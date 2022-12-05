use inkwell::{module::Linkage, values::InstructionOpcode};

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
            .add_global(name.name.clone(), &name.namespace, &defn.ty);
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

    // Create global objects, global variable and accessor function.
    let global_objs = program
        .global_symbol
        .iter()
        .map(|(name, defn)| {
            let ptr_to_obj_ty = ptr_to_object_type(&gc.context);
            let ptr_name = format!("PtrTo{}", name.to_string());
            let acc_fn_name = format!("Get{}", name.to_string());

            // Add global variable.
            let global_var = gc.module.add_global(ptr_to_obj_ty, None, &ptr_name);
            let null = ptr_to_obj_ty.const_null().as_basic_value_enum();
            global_var.set_initializer(&null);
            let global_var = global_var.as_basic_value_enum().into_pointer_value();

            // Add accessor function.
            let acc_fn_type = ptr_to_obj_ty.fn_type(&[], false);
            let acc_fn = gc
                .module
                .add_function(&acc_fn_name, acc_fn_type, Some(Linkage::External));

            // Register the accessor function to gc.
            gc.add_global_object(name.clone(), acc_fn);

            // Return global variable and accessor.
            (global_var, acc_fn, defn.clone())
        })
        .collect::<Vec<_>>();

    // Implement global accessor function.
    for (global_var, acc_fn, defn) in global_objs {
        let entry_bb = gc.context.append_basic_block(acc_fn, "entry");
        gc.builder().position_at_end(entry_bb);
        let ptr_to_obj = gc
            .builder()
            .build_load(global_var, "load_global_var")
            .into_pointer_value();
        let is_null = gc.builder().build_is_null(ptr_to_obj, "PtrToObjIsNull");
        let init_bb = gc.context.append_basic_block(acc_fn, "ptr_is_null");
        let end_bb = gc.context.append_basic_block(acc_fn, "ptr_is_non_null");
        gc.builder()
            .build_conditional_branch(is_null, init_bb, end_bb);

        // If ptr is null, then create object and initialize the pointer.
        gc.builder().position_at_end(init_bb);
        let obj = gc.eval_expr(defn.expr.clone());
        gc.builder().build_store(global_var, obj);
        gc.builder().position_at_end(init_bb);
        if SANITIZE_MEMORY {
            // Mark this object as global.
            let obj_id = gc.get_obj_id(obj);
            gc.call_runtime(RuntimeFunctions::MarkGlobal, &[obj_id.into()]);
        }
        gc.builder().build_unconditional_branch(end_bb);

        // Return object.
        gc.builder().position_at_end(end_bb);
        let ret = gc
            .builder()
            .build_load(global_var, "PtrToObj")
            .into_pointer_value();
        gc.builder().build_return(Some(&ret));
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
