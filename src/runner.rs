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

fn run_module(mut fix_mod: FixModule, opt_level: OptimizationLevel) -> i64 {
    // Add create global symbols
    add_builtin_symbols(&mut fix_mod);
    fix_mod.create_trait_method_symbols();

    // Calculate list of type constructors.
    fix_mod.calculate_type_env();

    // Set Namespaces to tycons and traits that appear in the module.
    fix_mod.set_namespace_of_tycons_and_traits();

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
        match &sym.expr {
            SymbolExpr::Simple(e) => {
                let e = typechecker.check_type(e.clone(), sym.ty.clone());
                sym.expr = SymbolExpr::Simple(e);
            }
            SymbolExpr::Method(methods) => {
                let mut methods = methods.clone();
                for m in &mut methods {
                    m.expr = typechecker.check_type(m.expr.clone(), m.ty.clone());
                }
                sym.expr = SymbolExpr::Method(methods);
            }
        }
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

    // Create GenerationContext.
    let context = Context::create();
    let module = context.create_module(&fix_mod.name);
    let mut gc = GenerationContext::new(&context, &module);

    // Build runtime functions.
    build_runtime(&mut gc);

    // Instanciate main function and all called functions.
    let main_expr = fix_mod.instantiate_main_function(&typechecker);

    // Generate codes.
    fix_mod.generate_code(&mut gc);

    // Add main function.
    let main_fn_type = context.i64_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Evaluate program and extract int value from result.
    let program_result = gc.eval_expr(main_expr);
    let result = gc.load_obj_field(program_result, int_type(&context), 1);

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
