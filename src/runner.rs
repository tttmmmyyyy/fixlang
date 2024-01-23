use build_time::build_time_utc;
use chrono::{DateTime, Utc};
use std::{
    env,
    fs::create_dir_all,
    fs::{self, remove_dir_all},
    path::PathBuf,
    process::Command,
    ptr::null,
    time::SystemTime,
};

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    module::Linkage,
    passes::PassManager,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};

use super::*;

fn execute_main_module<'c>(ee: &ExecutionEngine<'c>, config: &Configuration) -> i32 {
    // If sanitize_memory, load `libfixsanitizer.so`.
    if config.sanitize_memory {
        let path = "./sanitizer/libfixsanitizer.so";
        let err = load_library_permanently(path);
        if err {
            error_exit(&format!("Failed to load \"{}\".", path));
        }
    }
    // Build and load runtime library.
    // First, determine the file name of cached runtime library.
    // The file name is determined by the list of linked libraries and the list of macro_runtime_c, and the build time of the compiler.
    let mut hash_source = "".to_string();
    hash_source += build_time_utc!();
    let linked_libs_list = config
        .linked_libraries
        .iter()
        .map(|(s, _)| s.clone())
        .collect::<Vec<_>>()
        .join("_");
    hash_source += &linked_libs_list;
    hash_source += &config.runtime_c_macro.join("_");
    let runtime_so_path = PathBuf::from(INTERMEDIATE_PATH)
        .join(format!("libfixruntime.{:x}.so", md5::compute(hash_source)));
    if !runtime_so_path.exists() {
        let runtime_c_path = PathBuf::from(INTERMEDIATE_PATH).join("fixruntime.c");
        fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate directory.");
        fs::write(&runtime_c_path, include_str!("runtime.c"))
            .expect(&format!("Failed to generate runtime.c"));
        // Create library binary file.
        let mut com = Command::new("gcc");
        com.arg("-shared")
            .arg("-fpic")
            .arg("-o")
            .arg(runtime_so_path.to_str().unwrap())
            .arg(runtime_c_path.to_str().unwrap());
        for m in &config.runtime_c_macro {
            com.arg(format!("-D{}", m));
        }
        // Load dynamically linked libraries specified by user.
        for (lib_name, _) in &config.linked_libraries {
            if std::env::consts::OS != "macos" {
                com.arg(format!("-Wl,--no-as-needed")); // Apple's ld command doesn't support --no-as-needed.
            }
            com.arg(format!("-l{}", lib_name));
        }
        let output = com.output().expect("Failed to run gcc.");
        if output.stderr.len() > 0 {
            eprintln!(
                "{}",
                String::from_utf8(output.stderr)
                    .unwrap_or("(failed to stringify error message of gcc.)".to_string())
            );
        }
    }
    load_library_permanently(runtime_so_path.to_str().unwrap());

    unsafe {
        let func = ee
            .get_function::<unsafe extern "C" fn(i32, *const *const i8) -> i32>("main")
            .unwrap();
        func.call(0, null())
    }
}

fn build_module<'c>(
    context: &'c Context,
    module: &Module<'c>,
    target: Either<TargetMachine, ExecutionEngine<'c>>,
    mut fix_mod: Program,
    config: Configuration,
) -> Either<TargetMachine, ExecutionEngine<'c>> {
    // Calculate last affected dates.
    fix_mod.set_last_affected_dates();

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
        fix_mod.visible_namespaces.clone(),
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
    if config.uncurry_optimization {
        uncurry_optimization(&mut fix_mod);
    }

    // Perform borrowing optimization.
    if config.borrowing_optimization {
        borrowing_optimization(&mut fix_mod);
    }

    // Create GenerationContext.
    let mut gc = GenerationContext::new(
        &context,
        &module,
        target,
        config.clone(),
        fix_mod.type_env(),
    );

    // In debug mode, create debug infos.
    if config.debug_mode {
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

    // If AsyncTask is used, initialize thread pool.
    if config.async_task {
        // Store the pointer to `fixruntime_threadpool_run_task` function defined in LLVM module to the `fixruntime_threadpool_run_task` global variable defined in runtime.c.
        let run_task_func_ptr_ty = gc
            .context
            .void_type()
            .fn_type(
                &[gc.context.i8_type().ptr_type(AddressSpace::from(0)).into()],
                false,
            )
            .ptr_type(AddressSpace::from(0));
        let run_task_func_ptr = gc.module.add_global(
            run_task_func_ptr_ty,
            Some(AddressSpace::from(0)),
            "ptr_fixruntime_threadpool_run_task",
        );
        run_task_func_ptr.set_externally_initialized(true);
        run_task_func_ptr.set_linkage(Linkage::External);
        let run_task_func = gc
            .runtimes
            .get(&RuntimeFunctions::ThreadPoolRunTask)
            .unwrap();
        gc.builder().build_store(
            run_task_func_ptr.as_pointer_value(),
            run_task_func.as_global_value().as_pointer_value(),
        );
        // Initialize thread pool.
        gc.call_runtime(RuntimeFunctions::ThreadPoolInitialize, &[]);
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

    // If debug inf generated, finalize it.
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
    if !config.debug_mode {
        add_passes(&passmgr);
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

    gc.target
}

#[allow(dead_code)]
pub fn run_source(source: &str, mut config: Configuration) {
    const MAIN_RUN: &str = "main_run";
    let datetime: DateTime<Utc> = SystemTime::now().into();
    let file_hash = format!("{:x}", md5::compute(datetime.to_rfc3339()));

    if config.run_by_build {
        save_temporary_source(source, MAIN_RUN, &file_hash);
        config.source_files = vec![temporary_source_path(MAIN_RUN, &file_hash)];
        build_file(config);
        let output = Command::new("./a.out")
            .output()
            .expect("Failed to run a.out.");
        if output.status.code().is_none() {
            panic!("a.out crashed!");
        }
        if output.stdout.len() > 0 {
            println!(
                "{}",
                String::from_utf8(output.stdout)
                    .unwrap_or("(failed to parse stdout from a.out as UTF8.)".to_string()),
            );
        }
        if output.stderr.len() > 0 {
            eprintln!(
                "{}",
                String::from_utf8(output.stderr)
                    .unwrap_or("(failed to parse stderr from a.out as UTF8.)".to_string())
            );
        }
    } else {
        let source_mod = parse_source_temporary_file(source, MAIN_RUN, &file_hash);
        let mut target_mod = make_std_mod();
        target_mod.link(source_mod);
        target_mod.resolve_imports(&mut config);
        run_module(target_mod, config);
    }
}

pub fn run_module(fix_mod: Program, config: Configuration) -> i32 {
    let ctx = Context::create();
    let module = ctx.create_module("Main");
    let ee = module
        .create_jit_execution_engine(config.llvm_opt_level)
        .unwrap();
    let ee = build_module(&ctx, &module, Either::Right(ee), fix_mod, config.clone()).unwrap_right();
    execute_main_module(&ee, &config)
}

// Return file content and last modified.
pub fn read_file(path: &Path) -> Result<(String, UpdateDate), String> {
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
    let last_modified: Option<UpdateDate> = match file.metadata() {
        Err(why) => {
            println!("Failed to get last modified date of {}: {}", display, why);
            None
        }
        Ok(md) => match md.modified() {
            Err(why) => {
                println!("Failed to get last modified date of {}: {}", display, why);
                None
            }
            Ok(time) => Some(UpdateDate(time.into())),
        },
    };
    if last_modified.is_none() {
        println!("Build cache for {} will be ignored", display);
    }
    let last_modified = last_modified.unwrap_or(UpdateDate(SystemTime::now().into()));
    Ok((s, last_modified))
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
        let (content, last_modified) = match read_file(file_path) {
            Ok(p) => p,
            Err(e) => {
                panic!("{}", e)
            }
        };
        let mut fix_mod = parse_source(&content, file_path.to_str().unwrap());
        fix_mod.set_last_update(fix_mod.get_name_if_single_module(), last_modified);
        target_mod.link(fix_mod);
    }
    target_mod.resolve_imports(config);
    target_mod
}

pub fn run_file(mut config: Configuration) -> i32 {
    run_module(load_file(&mut config), config)
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
    let obj_path = PathBuf::from(INTERMEDIATE_PATH).join("a.o");
    let exec_path = config.get_output_executable_file_path();

    // Create intermediate directory.
    fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate .");

    let tm = get_target_machine(config.llvm_opt_level);

    let fix_mod = load_file(&mut config);

    let ctx = Context::create();
    let module = ctx.create_module("Main");
    module.set_triple(&tm.get_triple());
    module.set_data_layout(&tm.get_target_data().get_data_layout());

    let tm = build_module(&ctx, &module, Either::Left(tm), fix_mod, config.clone()).unwrap_left();
    tm.write_to_file(&module, inkwell::targets::FileType::Object, &obj_path)
        .map_err(|e| error_exit(&format!("Failed to write to file: {}", e)))
        .unwrap();

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
