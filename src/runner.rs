use std::path::PathBuf;

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    passes::{PassManager, PassManagerSubType},
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use super::*;

fn execute_main_module<'c>(ee: &ExecutionEngine<'c>, config: &Configuration) -> i32 {
    if config.sanitize_memory {
        assert_eq!(
            load_library_permanently("sanitizer/libfixsanitizer.so"),
            false
        );
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
        fix_mod.imported_modules.clone(),
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
    let iostate = allocate_obj(
        iostate_lit_ty(),
        &vec![],
        None,
        &mut gc,
        Some("iostate_for_main"),
    );
    let ret = gc.apply_lambda(main_obj, vec![iostate], None);
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

    // Run optimization
    let passmgr = PassManager::create(());

    passmgr.add_verifier_pass();
    add_passes(&passmgr);

    passmgr.run_on(module);
    unsafe {
        module.run_in_pass_manager(&passmgr);
    }

    // Print LLVM bitcode to file
    module.print_to_file("main.ll").unwrap();

    // Verify LLVM module.
    // Now not needed?
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }

    gc.target
}

pub fn run_source(source: &str, config: Configuration) -> i32 {
    let mut fix_mod = parse_source(source);
    fix_mod.import(make_std_mod());

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

pub fn run_file(path: &Path, config: Configuration) -> i32 {
    run_source(read_file(path).as_str(), config)
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
        None => error_exit("failed to creeate target machine"),
    }
}

pub fn build_file(path: &Path, config: Configuration) {
    let mut out_path = PathBuf::from(path);
    out_path.set_extension("o");
    let tm = get_target_machine(config.llvm_opt_level);

    let mut fix_mod = parse_source(&read_file(path));
    fix_mod.import(make_std_mod());

    let ctx = Context::create();
    let module = ctx.create_module(&fix_mod.name);
    module.set_triple(&tm.get_triple());
    module.set_data_layout(&tm.get_target_data().get_data_layout());

    let tm = build_module(&ctx, &module, Either::Left(tm), fix_mod, config).unwrap_left();
    tm.write_to_file(&module, inkwell::targets::FileType::Object, &out_path)
        .map_err(|e| error_exit(&format!("failed to write to file: {}", e)))
        .unwrap();
}
