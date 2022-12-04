use inkwell::module::Linkage;

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
    let module = context.create_module(&program.name);
    let mut gc = GenerationContext::new(&context, &module);

    // Build runtime functions.
    build_runtime(&mut gc);

    // Create global symbols.
    for (name, defn) in &program.global_symbol {
        let ptr_to_obj_ty = ptr_to_object_type(&gc.context);
        let ptr_name = format!("PtrTo{}", program.get_namespaced_name(name).to_string());
        let acc_fn_name = format!("Get{}", program.get_namespaced_name(name).to_string());

        // Add global pointer to the value of this symbol.
        let ptr_to_obj = gc.module.add_global(
            ptr_to_obj_ty.ptr_type(AddressSpace::Local),
            Some(AddressSpace::Local),
            &ptr_name,
        );
        let null = ptr_to_obj_ty.const_null().as_basic_value_enum();
        ptr_to_obj.set_initializer(&null);

        // Implement accessor function.
        let acc_fn_type = ptr_to_obj_ty.fn_type(&[], false);
        let acc_fn = gc
            .module
            .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::External));
        let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
        gc.builder().position_at_end(entry_bb);
        let is_null = gc.builder().build_int_compare(
            IntPredicate::EQ,
            ptr_to_obj.as_basic_value_enum().into_int_value(),
            null.into_int_value(),
            &format!("{}_is_null", ptr_name),
        );
        let init_bb = gc.context.append_basic_block(acc_fn, "ptr_is_null");
        let end_bb = gc.context.append_basic_block(acc_fn, "ptr_is_non_null");
        gc.builder()
            .build_conditional_branch(is_null, init_bb, end_bb);

        // If ptr is null, then create object and initialize the pointer.
        gc.builder().position_at_end(init_bb);
        let obj = gc.eval_expr(defn.expr.clone());
        gc.builder().build_store(ptr_to_obj.as_pointer_value(), obj);
        gc.builder().position_at_end(init_bb);
        gc.builder().build_unconditional_branch(end_bb);

        // Return object.
        gc.builder().position_at_end(end_bb);
        let ret = gc
            .builder()
            .build_load(ptr_to_obj.as_pointer_value(), "PtrToObj");
        gc.builder().build_return(Some(&ret));

        // Register the accessor function to gc.
        todo!()
    }

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
