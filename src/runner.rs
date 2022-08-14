use super::*;

fn execute_main_module<'c>(
    context: &'c Context,
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

fn run_ast(program: Arc<ExprInfo>, opt_level: OptimizationLevel) -> i64 {
    // Add library functions to program.
    let program = let_in(var_var("add"), add(), program);
    let program = let_in(var_var("eq"), eq(), program);
    let program = let_in(var_var("fix"), fix(), program);

    let program = calculate_aux_info(program);

    let context = Context::create();
    let module = context.create_module("main");

    let mut gc = GenerationContext::new(&context, &module);
    build_runtime(&mut gc);

    let main_fn_type = context.i64_type().fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);

    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    let program_result = gc.eval_expr(program);
    let int_obj_ptr = program_result;

    let int_obj_ty = ObjectType::int_obj_type().to_struct_type(&context);
    let value = gc.load_obj_field(int_obj_ptr, int_obj_ty, 1);
    gc.release(program_result);

    if SANITIZE_MEMORY {
        // Perform leak check
        gc.call_runtime(RuntimeFunctions::CheckLeak, &[]);
    }

    if let BasicValueEnum::IntValue(value) = value {
        gc.builder().build_return(Some(&value));
    } else {
        panic!("Given program doesn't return int value!");
    }

    module.print_to_file("ir").unwrap();
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }
    execute_main_module(&context, &module, opt_level)
}

pub fn run_source(source: &str, opt_level: OptimizationLevel) -> i64 {
    let ast = parse_source(source);
    run_ast(ast, opt_level)
}

pub fn run_file(path: &Path, opt_level: OptimizationLevel) -> i64 {
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    // ファイルの中身を文字列に読み込む。`io::Result<useize>`を返す。
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read {}: {}", display, why),
        Ok(_) => (),
    }

    run_source(s.as_str(), opt_level)
}
