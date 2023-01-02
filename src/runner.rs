use std::path::PathBuf;

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use super::*;

fn execute_main_module<'c>(ee: &ExecutionEngine<'c>) -> i64 {
    if SANITIZE_MEMORY {
        assert_eq!(
            load_library_permanently("sanitizer/libfixsanitizer.so"),
            false
        );
    }
    unsafe {
        let func = ee
            .get_function::<unsafe extern "C" fn() -> i64>("main")
            .unwrap();
        func.call()
    }
}

fn build_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    target: Either<TargetMachine, ExecutionEngine<'c>>,
    mut fix_mod: FixModule,
    result_as_main_return: bool,
) -> Either<TargetMachine, ExecutionEngine<'c>> {
    // Add built-in traits and types.
    fix_mod.add_builtin_traits_types();

    // Calculate list of type constructors.
    fix_mod.calculate_type_env();

    // Resolve namespaces to traits and types (not to variables).
    fix_mod.resolve_namespace();

    // Validate user-defined types.
    fix_mod.validate_user_defined_types();

    // Add global symbols
    fix_mod.add_builtin_symbols();

    // Validate trait env.
    fix_mod.validate_trait_env();

    // Create symbols.
    fix_mod.create_trait_method_symbols();

    // Set and check kinds that appear in the module.
    fix_mod.set_kinds();

    // Create typeckecker.
    let mut typechecker = TypeCheckContext::new(fix_mod.trait_env.clone(), fix_mod.type_env());

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &fix_mod.global_symbols {
        typechecker
            .scope
            .add_global(name.name.clone(), &name.namespace, &defn.ty);
    }

    // Check types.
    for (_name, sym) in &mut fix_mod.global_symbols {
        let mut tc = typechecker.clone();
        match &sym.expr {
            SymbolExpr::Simple(e) => {
                let e = tc.check_type(e.clone(), sym.ty.clone());
                sym.expr = SymbolExpr::Simple(e);
            }
            SymbolExpr::Method(methods) => {
                let mut methods = methods.clone();
                for m in &mut methods {
                    m.expr = tc.check_type(m.expr.clone(), m.ty.clone());
                }
                sym.expr = SymbolExpr::Method(methods);
            }
        }
        sym.typecheck_log = Some(tc);
    }

    // Calculate free variables of expressions.
    for (_name, sym) in &mut fix_mod.global_symbols {
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
    uncurry_optimization(&mut fix_mod);

    // Create GenerationContext.
    let mut gc = GenerationContext::new(&context, &module, target);

    // If use leaky allocator, prepare heap counter.
    if USE_LEAKY_ALLOCATOR {
        let leaky_heap_type = gc.context.i8_type().array_type(LEAKY_ALLOCATOR_HEAP_SIZE);
        let ptr_to_leaky_heap_type = leaky_heap_type.ptr_type(AddressSpace::from(0));
        let ptr_to_heap = gc
            .module
            .add_global(ptr_to_leaky_heap_type, None, LEAKY_HEAP_NAME);
        let null = ptr_to_leaky_heap_type.const_null().as_basic_value_enum();
        ptr_to_heap.set_initializer(&null);
    }

    // Build runtime functions.
    build_runtime(&mut gc);

    // Generate codes.
    fix_mod.generate_code(&mut gc);

    // Add main function.
    let main_fn_type = context.i64_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // If use leaky allocator, allocate heap.
    if USE_LEAKY_ALLOCATOR {
        let ptr_to_heap = gc
            .module
            .get_global(LEAKY_HEAP_NAME)
            .unwrap()
            .as_basic_value_enum()
            .into_pointer_value();
        // let leaky_heap_type = ptr_to_heap.get_type().get_element_type().into_array_type();
        let leaky_heap_type = gc.context.i8_type().array_type(LEAKY_ALLOCATOR_HEAP_SIZE);
        let leaky_heap = gc
            .builder()
            .build_malloc(leaky_heap_type, "leaky_heap")
            .unwrap();
        gc.builder().build_store(ptr_to_heap, leaky_heap);
    }

    // Evaluate program and extract int value from result.
    let result = gc.eval_expr(main_expr, None);
    let result = result.load_field_nocap(&mut gc, 0);

    // Perform leak check
    if SANITIZE_MEMORY {
        gc.call_runtime(RuntimeFunctions::CheckLeak, &[]);
    }

    // Print result if print_result and build return
    if let BasicValueEnum::IntValue(result) = result {
        if result_as_main_return {
            gc.builder().build_return(Some(&result));
        } else {
            let string_ptr = gc.builder().build_global_string_ptr("%d\n", "rust_str");
            gc.call_runtime(
                RuntimeFunctions::Printf,
                &[string_ptr.as_pointer_value().into(), result.into()],
            );
            gc.builder().build_return(Some(
                &gc.context.i64_type().const_zero().as_basic_value_enum(),
            ));
        }
    } else {
        panic!("Given program doesn't return int value!");
    }

    // Print LLVM bitcode to file
    module.print_to_file("main.ll").unwrap();

    // Verify LLVM module.
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }

    gc.target
}

pub fn run_source(source: &str, opt_level: OptimizationLevel, result_as_main_return: bool) -> i64 {
    let fix_mod = parse_source(source);
    let ctx = Context::create();
    let module = ctx.create_module(&fix_mod.name);
    let ee = module.create_jit_execution_engine(opt_level).unwrap();
    let ee = build_module(
        &ctx,
        &module,
        Either::Right(ee),
        fix_mod,
        result_as_main_return,
    )
    .unwrap_right();
    execute_main_module(&ee)
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

pub fn run_file(path: &Path, opt_level: OptimizationLevel, result_as_main_return: bool) -> i64 {
    run_source(read_file(path).as_str(), opt_level, result_as_main_return)
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

pub fn build_file(path: &Path, opt_level: OptimizationLevel, result_as_main_return: bool) {
    let mut out_path = PathBuf::from(path);
    out_path.set_extension("o");
    let tm = get_target_machine(opt_level);
    let fix_mod = parse_source(&read_file(path));
    let ctx = Context::create();
    let module = ctx.create_module(&fix_mod.name);
    let tm = build_module(
        &ctx,
        &module,
        Either::Left(tm),
        fix_mod,
        result_as_main_return,
    )
    .unwrap_left();
    tm.write_to_file(&module, inkwell::targets::FileType::Object, &out_path)
        .map_err(|e| error_exit(&format!("failed to write to file: {}", e)))
        .unwrap();
}
