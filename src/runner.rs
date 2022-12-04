use super::*;

fn execute_main_module<'c>(
    _context: &'c Context,
    module: &Module<'c>,
    opt_level: OptimizationLevel,
) -> i64 {
    if SANITIZE_MEMORY {
        assert_eq!(
            load_library_permanently("sanitizer/libfixsanitizer.so"),
            false
        );
    }
    let execution_engine = module.create_jit_execution_engine(opt_level).unwrap();
    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> i64>("main")
            .unwrap();
        func.call()
    }
}

fn run_module(mut program: FixModule, opt_level: OptimizationLevel) -> i64 {
    // Create typeckecker.
    let mut typechecker = TypeCheckContext::default();

    // Read type declarations to register user-defined types to typechecker.
    typechecker.add_tycons(&program.type_decls);

    // Add built-in functions to program.
    add_builtin_symbols(&mut program);

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &program.global_symbol {
        typechecker
            .scope
            .add_global(name.clone(), &program.get_namespace(), &defn.ty);
    }

    // Check types.
    for (_name, defn) in &mut program.global_symbol {
        let mut tc = typechecker.clone();
        defn.expr = tc.check_type_nofree(defn.expr.clone(), defn.ty.clone());
    }
    // Check types of root expression. Note: root expression will be removed in future.
    program.expr = typechecker.unify_type_of_expr(&program.expr, int_lit_ty());
    if !typechecker.reduce_predicates() || !typechecker.predicates.is_empty() {
        typechecker.error_exit_on_predicates();
    }

    // Calculate free variables of nodes.
    for (_name, defn) in &mut program.global_symbol {
        defn.expr = calculate_free_vars(defn.expr.clone());
    }
    program.expr = calculate_free_vars(program.expr);

    // Create GenerationContext.
    let context = Context::create();
    let module = context.create_module("main");
    let mut gc = GenerationContext::new(&context, &module);

    // Build runtime functions.
    build_runtime(&mut gc);

    // Add main function.
    let main_fn_type = context.i64_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Evaluate program and extract int value from result.
    let program_result = gc.eval_expr(program.expr);
    let result = gc.load_obj_field(program_result, int_type(&context), 1);
    gc.release(program_result);

    // Perform leak check
    if SANITIZE_MEMORY {
        gc.call_runtime(RuntimeFunctions::CheckLeak, &[]);
    }

    // Build return
    if let BasicValueEnum::IntValue(result) = result {
        gc.builder().build_return(Some(&result));
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

    // Run the module.
    execute_main_module(&context, &module, opt_level)
}

pub fn run_source(source: &str, opt_level: OptimizationLevel) -> i64 {
    let module = parse_source(source);
    run_module(module, opt_level)
}

pub fn run_file(path: &Path, opt_level: OptimizationLevel) -> i64 {
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

    run_source(s.as_str(), opt_level)
}
