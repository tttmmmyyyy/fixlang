use std::{path::PathBuf, process::Command};

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    passes::{PassManager, PassManagerSubType},
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use super::*;

fn execute_main_module<'c>(ee: &ExecutionEngine<'c>, config: &Configuration) -> i32 {
    if config.sanitize_memory {
        let path = "./sanitizer/libfixsanitizer.so";
        let err = load_library_permanently(path);
        if err {
            error_exit(&format!("Failed to load \"{}\".", path));
        }
    }
    for (lib_name, _) in &config.linked_libraries {
        let lib_name = format!("lib{}.so", lib_name);
        let err = load_library_permanently(&lib_name);
        if err {
            error_exit(&format!("Failed to load \"{}\".", lib_name));
        }
    }
    unsafe {
        let func = ee
            .get_function::<unsafe extern "C" fn() -> i32>("main")
            .unwrap();
        func.call()
    }
}

fn build_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    target: Either<TargetMachine, ExecutionEngine<'c>>,
    mut fix_mod: FixModule,
    config: Configuration,
) -> Either<TargetMachine, ExecutionEngine<'c>> {
    // Calculate list of type constructors.
    fix_mod.calculate_type_env();

    // Resolve namespaces to traits and types (not to variables).
    fix_mod.resolve_namespace();

    // Validate user-defined types.
    fix_mod.validate_type_defns();

    // Add struct / union methods
    fix_mod.add_methods();

    // Validate trait env.
    fix_mod.validate_trait_env();

    // Create symbols.
    fix_mod.create_trait_method_symbols();

    // Set and check kinds that appear in the module.
    fix_mod.set_kinds();

    // Create typeckecker.
    let mut typechecker = TypeCheckContext::new(
        fix_mod.trait_env.clone(),
        fix_mod.type_env(),
        fix_mod.imported_mod_map.clone(),
    );

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &fix_mod.global_values {
        typechecker
            .scope
            .add_global(name.name.clone(), &name.namespace, &defn.ty);
    }

    // Check types.
    for (name, sym) in &mut fix_mod.global_values {
        let mut tc = typechecker.clone();
        match &sym.expr {
            SymbolExpr::Simple(e) => {
                tc.current_module = Some(name.module());
                let e = tc.check_type(e.clone(), sym.ty.clone());
                sym.expr = SymbolExpr::Simple(e);
            }
            SymbolExpr::Method(methods) => {
                let mut methods = methods.clone();
                for m in &mut methods {
                    tc.current_module = Some(m.define_module.clone());
                    m.expr = tc.check_type(m.expr.clone(), m.ty.clone());
                }
                sym.expr = SymbolExpr::Method(methods);
            }
        }
        sym.typecheck_log = Some(tc);
    }

    // Calculate free variables of expressions.
    for (_name, sym) in &mut fix_mod.global_values {
        match &sym.expr {
            SymbolExpr::Simple(e) => {
                let e = calculate_free_vars(e.clone());
                sym.expr = SymbolExpr::Simple(e);
            }
            SymbolExpr::Method(methods) => {
                let mut methods = methods.clone();
                for m in &mut methods {
                    m.expr = calculate_free_vars(m.expr.clone());
                }
                sym.expr = SymbolExpr::Method(methods);
            }
        }
    }

    // Instanciate main function and all called functions.
    let main_expr = fix_mod.instantiate_main_function();
    if config.funptr_optimization {
        funptr_optimization(&mut fix_mod);
    }

    // Create GenerationContext.
    let mut gc = GenerationContext::new(&context, &module, target, config);

    // Build runtime functions.
    build_runtime(&mut gc);

    // Generate codes.
    fix_mod.generate_code(&mut gc);

    // Add main function.
    let main_fn_type = context.i32_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Run main object.
    let main_obj = gc.eval_expr(main_expr, None); // `IO ()`
    let main_lambda_val = main_obj.load_field_nocap(&mut gc, 0);
    let main_lambda_ty = type_fun(make_tuple_ty(vec![]), make_tuple_ty(vec![]));
    let main_lambda = Object::create_from_value(main_lambda_val, main_lambda_ty, &mut gc);
    let unit = allocate_obj(
        make_tuple_ty(vec![]),
        &vec![],
        None,
        &mut gc,
        Some("unit_for_main_io"),
    );
    let ret = gc.apply_lambda(main_lambda, vec![unit], None);
    gc.release(ret);

    // Perform leak check
    if gc.config.sanitize_memory {
        // Deallocate all global objects.
        let mut global_names = vec![];
        for (name, _) in &gc.global {
            global_names.push(name.clone());
        }
        for name in global_names {
            let obj = gc.get_var(&name).ptr.get(&gc);
            gc.release(obj);
        }

        gc.call_runtime(RuntimeFunctions::CheckLeak, &[]);
    }

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)));

    // Print LLVM bitcode to file
    // module.print_to_file("main.ll").unwrap();

    // Run optimization
    let passmgr = PassManager::create(());

    passmgr.add_verifier_pass();
    add_passes(&passmgr);

    passmgr.run_on(module);
    unsafe {
        module.run_in_pass_manager(&passmgr);
    }

    // Verify LLVM module.
    // Maybe not needed at now?
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }

    gc.target
}

#[allow(dead_code)]
pub fn run_source(source: &str, config: Configuration) -> i32 {
    // When run source string, import only "Std".
    let mut imports: Vec<ImportStatement> = vec![];

    let std_mod = make_std_mod();
    imports.append(&mut std_mod.import_statements.clone());
    let mut target_mod = std_mod;

    let source_mod = parse_source(source, "{filename unknown}");
    imports.append(&mut source_mod.import_statements.clone());
    target_mod.link(source_mod);

    resolve_imports(&mut target_mod, &mut imports);

    run_module(target_mod, config)
}

pub fn run_module(fix_mod: FixModule, config: Configuration) -> i32 {
    let ctx = Context::create();
    let module = ctx.create_module(&fix_mod.name);
    let ee = module
        .create_jit_execution_engine(config.llvm_opt_level)
        .unwrap();
    let ee = build_module(&ctx, &module, Either::Right(ee), fix_mod, config.clone()).unwrap_right();
    execute_main_module(&ee, &config)
}

pub fn read_file(path: &Path) -> String {
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        Ok(_) => (),
    }

    s
}

fn resolve_imports(target_mod: &mut FixModule, imports: &mut Vec<ImportStatement>) {
    while imports.len() > 0 {
        let import = imports.pop().unwrap();

        // If import is already resolved, do nothing.
        if target_mod.imported_mod_map.contains_key(&import.module) {
            continue;
        }

        // TODO: search for library here.

        error_exit_with_src(
            &format!("Cannot find module `{}`", import.module),
            &import.source,
        );
    }
}

pub fn load_file(config: &Configuration) -> FixModule {
    let mut imports: Vec<ImportStatement> = vec![];

    // Link all modules specified in source_files.
    let std_mod = make_std_mod();
    imports.append(&mut std_mod.import_statements.clone());
    let mut target_mod = std_mod;
    for file_path in &config.source_files {
        let fix_mod = parse_source(&read_file(file_path), file_path.to_str().unwrap());
        imports.append(&mut fix_mod.import_statements.clone());
        target_mod.link(fix_mod);
    }

    resolve_imports(&mut target_mod, &mut imports);
    target_mod
}

pub fn run_file(config: Configuration) -> i32 {
    run_module(load_file(&config), config)
}

fn get_target_machine(opt_level: OptimizationLevel) -> TargetMachine {
    let _native = Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| error_exit(&format!("failed to initialize native: {}", e)))
        .unwrap();
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| {
            error_exit(&format!("failed to create target: {}", e));
        })
        .unwrap();
    let cpu_name = TargetMachine::get_host_cpu_name();
    let target_machine = target.create_target_machine(
        &triple,
        cpu_name.to_str().unwrap(),
        TargetMachine::get_host_cpu_features().to_str().unwrap(),
        opt_level,
        RelocMode::Default,
        CodeModel::Default,
    );
    match target_machine {
        Some(tm) => tm,
        None => error_exit("Failed to creeate target machine."),
    }
}

pub fn build_file(config: Configuration) {
    let obj_path = PathBuf::from("a.o");
    let exec_path = PathBuf::from("a.out");

    let tm = get_target_machine(config.llvm_opt_level);

    let fix_mod = load_file(&config);

    let ctx = Context::create();
    let module = ctx.create_module(&fix_mod.name);
    module.set_triple(&tm.get_triple());
    module.set_data_layout(&tm.get_target_data().get_data_layout());

    let tm = build_module(&ctx, &module, Either::Left(tm), fix_mod, config.clone()).unwrap_left();
    tm.write_to_file(&module, inkwell::targets::FileType::Object, &obj_path)
        .map_err(|e| error_exit(&format!("Failed to write to file: {}", e)))
        .unwrap();

    let mut libs_opts = vec![];
    for (lib_name, link_type) in &config.linked_libraries {
        match link_type {
            LinkType::Static => libs_opts.push("-Wl,-Bstatic".to_string()),
            LinkType::Dynamic => libs_opts.push("-Wl,-Bdynamic".to_string()),
        }
        libs_opts.push(format!("-l{}", lib_name));
    }

    let _link_res = Command::new("gcc")
        .args(libs_opts)
        .arg("-o")
        .arg(exec_path.to_str().unwrap())
        .arg(obj_path.to_str().unwrap())
        .output()
        .expect("Failed to run gcc.");
}
