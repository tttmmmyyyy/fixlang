use build_time::build_time_utc;
use rand::Rng;
use std::{
    env,
    fs::create_dir_all,
    fs::{self, remove_dir_all},
    path::PathBuf,
    process::Command,
};

use inkwell::{
    module::Linkage,
    passes::PassManager,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetData, TargetMachine},
};
use stopwatch::StopWatch;

use super::*;

fn build_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    target_data: TargetData,
    mut fix_mod: Program,
    config: Configuration,
) {
    let _sw = StopWatch::new("build_module", config.show_build_times);

    // Add tuple types used in this program.
    let mut used_tuple_sizes = fix_mod.used_tuple_sizes.clone();
    // Make elements of used_tuple_sizes unique.
    used_tuple_sizes.sort();
    used_tuple_sizes.dedup();
    for tuple_size in used_tuple_sizes {
        fix_mod.add_tuple_defn(tuple_size);
    }

    // Calculate list of type constructors.
    fix_mod.calculate_type_env();

    // Infer namespaces of traits and types that appear in declarations (not in expressions).
    fix_mod.resolve_namespace_in_declaration();

    // Resolve type aliases that appear in declarations (not in expressions).
    fix_mod.resolve_type_aliases_in_declaration();

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
        fix_mod.visible_mods.clone(),
    );

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &fix_mod.global_values {
        typechecker
            .scope
            .add_global(name.name.clone(), &name.namespace, &defn.scm);
    }

    // Instantiate main function and all called functions.
    let main_expr = fix_mod.instantiate_main_function(&typechecker);

    // Perform uncurrying optimization.
    if config.get_uncurry_optimization() {
        uncurry_optimization(&mut fix_mod);
    }

    // Perform borrowing optimization.
    if config.get_borrowing_optimization() {
        borrowing_optimization(&mut fix_mod);
    }

    // Create GenerationContext.
    let mut gc = GenerationContext::new(
        &context,
        &module,
        target_data,
        config.clone(),
        fix_mod.type_env(),
    );

    // In debug mode, create debug infos.
    if config.debug_info {
        gc.create_debug_info();
    }

    // Build runtime functions.
    build_runtime(&mut gc);

    // Generate codes.
    fix_mod.generate_code(&mut gc);

    // Add main function.
    let main_fn_type = context.i32_type().fn_type(
        &[
            context.i32_type().into(), // argc
            context
                .i8_type()
                .ptr_type(AddressSpace::from(0))
                .ptr_type(AddressSpace::from(0))
                .into(), // argv
        ],
        false,
    );
    let main_function = module.add_function("main", main_fn_type, None);
    let entry_bb = context.append_basic_block(main_function, "entry");
    gc.builder().position_at_end(entry_bb);

    // Save argc and argv to global variables.
    for (i, arg) in [GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV]
        .iter()
        .enumerate()
    {
        let arg_val = main_function.get_nth_param(i as u32).unwrap();
        let gv_ptr = gc
            .module
            .get_global(arg)
            .unwrap()
            .as_basic_value_enum()
            .into_pointer_value();
        gc.builder().build_store(gv_ptr, arg_val);
    }

    // Store the pointer to `fixruntime_run_function` function defined in LLVM module to the `ptr_fixruntime_run_function` global variable defined in runtime.c.
    let run_function_func_ptr_ty = gc
        .context
        .i8_type()
        .ptr_type(AddressSpace::from(0))
        .fn_type(
            &[gc.context.i8_type().ptr_type(AddressSpace::from(0)).into()],
            false,
        )
        .ptr_type(AddressSpace::from(0));
    let run_task_func_ptr = gc.module.add_global(
        run_function_func_ptr_ty,
        Some(AddressSpace::from(0)),
        "ptr_fixruntime_run_function",
    );
    run_task_func_ptr.set_externally_initialized(true);
    run_task_func_ptr.set_linkage(Linkage::External);
    let run_function_func = gc.runtimes.get(&RuntimeFunctions::RunFunction).unwrap();
    gc.builder().build_store(
        run_task_func_ptr.as_pointer_value(),
        run_function_func.as_global_value().as_pointer_value(),
    );

    // If both of `AsyncTask` and sanitizer are used, prepare for terminating threads.
    if config.async_task && config.sanitize_memory {
        gc.call_runtime(RuntimeFunctions::ThreadPrepareTermination, &[]);
    }

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
    gc.check_leak();

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)));

    // If debug info is generated, finalize it.
    gc.finalize_di();

    // Print LLVM bitcode to file
    if config.emit_llvm {
        let path = config.get_output_llvm_ir_path(true);
        if let Err(e) = module.print_to_file(path) {
            error_exit(&format!("Failed to emit llvm: {}", e.to_string()));
        }
    }

    // Run optimization
    let passmgr = PassManager::create(());

    passmgr.add_verifier_pass();
    match config.fix_opt_level {
        FixOptimizationLevel::None => {}
        FixOptimizationLevel::Minimum => {
            passmgr.add_tail_call_elimination_pass();
        }
        FixOptimizationLevel::Default => {
            add_passes(&passmgr);
        }
    }

    passmgr.run_on(module);

    // Verify LLVM module.
    // Maybe not needed at now?
    let verify = module.verify();
    if verify.is_err() {
        print!("{}", verify.unwrap_err().to_str().unwrap());
        panic!("LLVM verify failed!");
    }

    // Print LLVM bitcode to file
    if config.emit_llvm {
        let path = config.get_output_llvm_ir_path(false);
        if let Err(e) = module.print_to_file(path) {
            error_exit(&format!("Failed to emit llvm: {}", e.to_string()));
        }
    }
}

#[allow(dead_code)]
pub fn run_source(source: &str, mut config: Configuration) {
    const MAIN_RUN: &str = "main_run";
    let source_hash = format!("{:x}", md5::compute(source));
    save_temporary_source(source, MAIN_RUN, &source_hash);
    config.source_files = vec![temporary_source_path(MAIN_RUN, &source_hash)];
    run_file(config);
}

// Return file content and last modified.
pub fn read_file(path: &Path) -> Result<String, String> {
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => return Err(format!("Couldn't open {}: {}", display, why)),
        Ok(file) => file,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => return Err(format!("Couldn't read {}: {}", display, why)),
        Ok(_) => (),
    }
    Ok(s)
}

// Create a directory if it doesn't exist, and return its path.
pub fn touch_directory<P>(rel_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let cur_dir = match env::current_dir() {
        Err(why) => panic!("Failed to get current directory: {}", why),
        Ok(dir) => dir,
    };
    let res = cur_dir.join(rel_path);
    match create_dir_all(&res) {
        Err(why) => panic!("Failed to create directory {}: {}", res.display(), why),
        Ok(_) => {}
    };
    res
}

pub fn load_file(config: &mut Configuration) -> Program {
    // Link all modules specified in source_files.
    let mut target_mod = make_std_mod();
    for file_path in &config.source_files {
        let fix_mod = parse_file_path(file_path.clone());
        target_mod.link(fix_mod);
    }
    target_mod.resolve_imports(config);
    target_mod
}

pub fn run_file(mut config: Configuration) {
    fs::create_dir_all(DOT_FIXLANG).expect("Failed to create \".fixlang\" directory.");

    // For parallel execution, use different file name for each execution.
    let a_out_path: String = format!("./{}/a{}.out", DOT_FIXLANG, rand::thread_rng().gen::<u64>());
    config.out_file_path = Some(PathBuf::from(a_out_path.clone()));

    build_file(config.clone());

    let output = Command::new(a_out_path.clone())
        .output()
        .expect(&format!("Failed to run \"{}\".", a_out_path));
    if output.stdout.len() > 0 {
        print!(
            "{}",
            String::from_utf8(output.stdout)
                .unwrap_or("Failed to parse stdout as UTF8.".to_string()),
        );
    }
    if output.stderr.len() > 0 {
        eprint!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or("Failed to parse stderr as UTF8.".to_string()),
        );
    }
    // Remove the executable file.
    fs::remove_file(a_out_path.clone()).expect(&format!("Failed to remove \"{}\".", a_out_path));

    if output.status.code().is_none() {
        panic!("Program terminated abnormally.");
    }
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

pub fn build_file(mut config: Configuration) {
    let obj_path =
        PathBuf::from(INTERMEDIATE_PATH).join(format!("a{}.o", rand::thread_rng().gen::<u64>())); // Add randomness for parallel execution.
    let exec_path = config.get_output_executable_file_path();

    // Create intermediate directory.
    fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate .");

    let target_machine = get_target_machine(config.get_llvm_opt_level());

    let fix_mod = load_file(&mut config);

    let ctx = Context::create();
    let module = ctx.create_module("Main");
    module.set_triple(&target_machine.get_triple());
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());

    build_module(
        &ctx,
        &module,
        target_machine.get_target_data(),
        fix_mod,
        config.clone(),
    );

    {
        let _sw = StopWatch::new("write_to_file", config.show_build_times);
        target_machine
            .write_to_file(&module, inkwell::targets::FileType::Object, &obj_path)
            .map_err(|e| error_exit(&format!("Failed to write to file: {}", e)))
            .unwrap();
    }

    let mut libs_opts = vec![];
    for (lib_name, link_type) in &config.linked_libraries {
        if std::env::consts::OS != "macos" {
            match link_type {
                LinkType::Static => libs_opts.push("-Wl,-Bstatic".to_string()),
                LinkType::Dynamic => libs_opts.push("-Wl,-Bdynamic".to_string()),
            }
        } else {
            match link_type {
                LinkType::Static => error_exit("Static linking is not supported on macOS."),
                _ => {}
            }
        }
        libs_opts.push(format!("-l{}", lib_name));
    }
    if config.sanitize_memory {
        libs_opts.push("-Wl,-rpath=./sanitizer".to_string());
        libs_opts.push("-Wl,-Bdynamic".to_string());
        libs_opts.push("-lfixsanitizer".to_string());
    }

    // Build runtime.c to object file.
    let mut runtime_obj_hash_source = "".to_string();
    runtime_obj_hash_source += build_time_utc!();
    runtime_obj_hash_source += &config.runtime_c_macro.join("_");
    let runtime_obj_path = PathBuf::from(INTERMEDIATE_PATH).join(format!(
        "fixruntime.{:x}.o",
        md5::compute(runtime_obj_hash_source)
    ));
    if !runtime_obj_path.exists() {
        let runtime_c_path = PathBuf::from(INTERMEDIATE_PATH).join("fixruntime.c");
        fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate directory.");
        fs::write(&runtime_c_path, include_str!("runtime.c"))
            .expect(&format!("Failed to generate runtime.c"));
        // Create library object file.
        let mut com = Command::new("gcc");
        let mut com = com
            .arg("-ffunction-sections")
            .arg("-fdata-sections")
            .arg("-o")
            .arg(runtime_obj_path.to_str().unwrap())
            .arg("-c")
            .arg(runtime_c_path.to_str().unwrap());
        for m in &config.runtime_c_macro {
            com = com.arg(format!("-D{}", m));
        }
        let output = com.output().expect("Failed to run gcc.");

        if output.stderr.len() > 0 {
            eprintln!(
                "{}",
                String::from_utf8(output.stderr)
                    .unwrap_or("(failed to parse stderr from gcc as UTF8.)".to_string())
            );
        }
    }

    let mut com = Command::new("gcc");
    com.arg("-Wno-unused-command-line-argument").arg("-no-pie");
    if std::env::consts::OS == "macos" {
        com.arg("-Wl,-dead_strip");
    } else {
        com.arg("-Wl,--gc-sections");
    }
    com.arg("-o")
        .arg(exec_path.to_str().unwrap())
        .arg(obj_path.to_str().unwrap())
        .arg(runtime_obj_path.to_str().unwrap())
        .args(libs_opts);
    let output = com.output().expect("Failed to run gcc.");
    if output.stderr.len() > 0 {
        eprintln!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or("(failed to parse stderr from gcc as UTF8.)".to_string())
        );
    }
}

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    remove_dir_all(DOT_FIXLANG).expect(&format!("Failed to remove `{}` directory.", DOT_FIXLANG));
}
