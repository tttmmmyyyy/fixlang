use crate::ast::export_statement::ExportStatement;
use crate::error::error_exit;
use build_time::build_time_utc;
use error::Errors;
use rand::Rng;
use std::{
    fs::{self, create_dir_all, remove_dir_all},
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};

use inkwell::{
    passes::PassManager,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};
use stopwatch::StopWatch;

use self::{compile_unit::CompileUnit, cpu_features::CpuFeatures};

use super::*;

// Compile the program, and returns the path of object files to be linked.
fn build_object_files<'c>(
    mut program: Program,
    config: Configuration,
) -> Result<Vec<PathBuf>, Errors> {
    let _sw = StopWatch::new("build_module", config.show_build_times);

    // Add tuple definitions.
    program.add_tuple_defns();

    // Add trait implementations for tuples such as ToString or Eq.
    program.link(
        make_tuple_traits_mod(&program.used_tuple_sizes, &config),
        true,
    )?;

    // Validate export statements.
    program.validate_export_statements();

    // Calculate list of type constructors.
    program.calculate_type_env();

    // Infer namespaces of traits and types that appear in declarations and associated type implementations.
    program.resolve_namespace_capital_names_not_in_expression();

    // Resolve type aliases that appear in declarations and associated type implementations.
    program.resolve_type_aliases_in_declaration();

    // Validate user-defined types.
    program.validate_type_defns();

    // Add struct / union methods
    program.add_methods()?;

    // Validate trait env.
    program.validate_trait_env();

    // Create symbols.
    program.create_trait_method_symbols();

    // Validate constraints of global value type.
    program.validate_global_value_types();

    // Check if all items referred in import statements are defined.
    // This check should be done after `add_methods` and `create_trait_method_symbols`.
    program.validate_import_statements();

    // Set and check kinds that appear in the module.
    program.set_kinds();

    // Create typeckecker.
    let mut typechecker = TypeCheckContext::new(
        program.trait_env.clone(),
        program.type_env(),
        program.kind_env(),
        program.mod_to_import_stmts.clone(),
    );

    // Register type declarations of global symbols to typechecker.
    for (name, defn) in &program.global_values {
        typechecker
            .scope
            .add_global(name.name.clone(), &name.namespace, &defn.scm);
    }

    // Instantiate main function and all called functions.
    let main_expr = program.instantiate_main_function(&typechecker);

    // Instantiate exported functions and all called functions.
    program.instantiate_exported_values(&typechecker);

    // Perform uncurrying optimization.
    if config.perform_uncurry_optimization() {
        uncurry_optimization(&mut program);
    }

    // Perform borrowing optimization.
    if config.perform_borrowing_optimization() {
        borrowing_optimization(&mut program);
    }

    // Determine compilation units.
    let mut units = vec![];
    let mut instantiated_symbols = program
        .instantiated_symbols
        .values()
        .cloned()
        .collect::<Vec<_>>();
    instantiated_symbols.sort_by(|a, b| a.instantiated_name.cmp(&b.instantiated_name));
    let all_symbols = instantiated_symbols.clone();
    {
        let module_dependency_hash = program.module_dependency_hash_map();
        let module_dependency_map = program.module_dependency_map();
        if config.separate_compilation() {
            units = CompileUnit::split_symbols(
                instantiated_symbols,
                &module_dependency_hash,
                &module_dependency_map,
                &config,
            );
            // Also add main compilation unit.
            let mut main_unit = CompileUnit::new(vec![], vec![]);
            main_unit.set_random_unit_hash(); // Recompile main unit every time.
            units.push(main_unit);
        } else {
            // Add main compilation unit, which includes all symbols.
            let modules = program.linked_mods().iter().cloned().collect::<Vec<_>>();
            let mut main_unit = CompileUnit::new(instantiated_symbols, modules);
            main_unit.set_random_unit_hash(); // Recompile main unit every time.
            units.push(main_unit);
        }
    }

    // Paths of object files to be linked.
    let mut obj_paths = vec![];

    // Generate object files in parallel.
    let mut threads = vec![];
    let units_count = units.len();
    for (i, unit) in units.into_iter().enumerate() {
        // We generate the main unit in the last.
        let is_main_unit = i == units_count - 1;

        obj_paths.push(unit.object_file_path());
        // If the object file is cached, skip the generation.
        if unit.is_cached() {
            if config.verbose {
                eprintln!(
                    "Skipping generation of object file for {}.",
                    unit.to_string()
                );
            }
            continue;
        }
        if config.verbose {
            eprintln!("Generating object file for {}.", unit.to_string());
        }

        let all_symbols = all_symbols.clone();
        let config = config.clone();
        let type_env = program.type_env();

        let export_statements = if is_main_unit {
            // Export statements are only needed for the main unit.
            std::mem::replace(&mut program.export_statements, vec![])
        } else {
            vec![]
        };

        let main_expr = main_expr.clone();
        threads.push(std::thread::spawn(move || {
            // Create GenerationContext.
            let context = Context::create();
            let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
            let module = GenerationContext::create_module(
                &format!("Module-{}", unit.unit_hash()),
                &context,
                &target_machine,
            );
            let mut gc = GenerationContext::new(
                &context,
                &module,
                target_machine.get_target_data(),
                config.clone(),
                type_env,
            );

            // In debug mode, create debug infos.
            if config.debug_info {
                gc.create_debug_info();
            }

            // Declare runtime functions.
            runtime::build_runtime(&mut gc, BuildMode::Declare);

            // Declare all symbols in this program.
            // TODO: Optimize so that only necessary symbols are declared.
            for symbol in &all_symbols {
                gc.declare_symbol(symbol);
            }

            // Implement all symbols in this unit.
            for symbol in unit.symbols() {
                gc.implement_symbol(symbol);
            }

            if is_main_unit {
                assert!(!unit.is_cached()); // Main unit should not be cached.

                // Implement runtime functions.
                build_runtime(&mut gc, BuildMode::Implement);

                // Implement exported C functions.
                build_exported_c_functions(&mut gc, &export_statements);

                // Implement the `main()` function.
                build_main_function(&mut gc, main_expr.clone(), &config);
            }

            // If debug info is generated, finalize it.
            gc.finalize_di();

            if config.emit_llvm {
                // Print LLVM-IR to file before optimization.
                emit_llvm(gc.module, &config, false);
            }

            // LLVM level optimization.
            optimize_and_verify(gc.module, &config);

            if config.emit_llvm {
                // Print LLVM-IR to file after optimization.
                emit_llvm(gc.module, &config, true);
            }

            // Generate object file.
            write_to_object_file(gc.module, &target_machine, &unit.object_file_path());
        }));
    }
    // Wait for all threads to finish.
    for t in threads {
        t.join().unwrap();
    }

    Ok(obj_paths)
}

fn write_to_object_file<'c>(module: &Module<'c>, target_machine: &TargetMachine, obj_path: &Path) {
    // Create directory if it doesn't exist.
    let dir_path = obj_path.parent().unwrap();
    match fs::create_dir_all(dir_path) {
        Err(e) => {
            error_exit(&format!(
                "Failed to create directory {}: {}",
                dir_path.display(),
                e
            ));
        }
        Ok(_) => {}
    }
    // Write to a temporary file.
    let tmp_file_path =
        obj_path.with_extension(rand::thread_rng().gen::<u64>().to_string() + ".tmp");
    target_machine
        .write_to_file(&module, inkwell::targets::FileType::Object, &tmp_file_path)
        .map_err(|e| {
            error_exit(&format!(
                "Failed to write to file {}: {}",
                obj_path.display(),
                e
            ))
        })
        .unwrap();

    // Rename the temporary file to the final file.
    match fs::rename(&tmp_file_path, obj_path) {
        Err(e) => {
            error_exit(&format!(
                "Failed to rename {} to {}: {}",
                tmp_file_path.display(),
                obj_path.display(),
                e
            ));
        }
        Ok(_) => {}
    }
}

fn emit_llvm<'c>(module: &Module<'c>, config: &Configuration, optimized: bool) {
    let unit_name = module.get_name().to_str().unwrap();
    let path = config.get_output_llvm_ir_path(optimized, unit_name);
    if let Err(e) = module.print_to_file(path.clone()) {
        error_exit(&format!("Failed to emit LLVM-IR: {}", e.to_string()));
    }
}

fn optimize_and_verify<'c>(module: &Module<'c>, config: &Configuration) {
    // Run optimization
    let passmgr = PassManager::create(());

    passmgr.add_verifier_pass(); // Verification before optimization.
    match config.fix_opt_level {
        FixOptimizationLevel::None => {}
        FixOptimizationLevel::Minimum => {
            passmgr.add_tail_call_elimination_pass();
        }
        FixOptimizationLevel::Separated => {
            llvm_passes::add_optimization_passes(&passmgr);
        }
        FixOptimizationLevel::Default => {
            llvm_passes::add_internalize_and_strip_passes(&passmgr);
            llvm_passes::add_optimization_passes(&passmgr);
            llvm_passes::add_internalize_and_strip_passes(&passmgr);
        }
    }
    passmgr.add_verifier_pass(); // Verification after optimization.
    passmgr.run_on(module);
}

// Build exported c functions.
fn build_exported_c_functions<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    export_stmts: &[ExportStatement],
) {
    for export_stmt in export_stmts {
        export_stmt.implement(gc);
    }
}

fn build_main_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    main_expr: Arc<ExprNode>,
    config: &Configuration,
) {
    let main_fn_type = gc.context.i32_type().fn_type(
        &[
            gc.context.i32_type().into(), // argc
            gc.context
                .i8_type()
                .ptr_type(AddressSpace::from(0))
                .ptr_type(AddressSpace::from(0))
                .into(), // argv
        ],
        false,
    );
    let main_function = gc.module.add_function("main", main_fn_type, None);
    let entry_bb = gc.context.append_basic_block(main_function, "entry");
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
    // let run_function_func_ptr_ty: PointerType = gc
    //     .context
    //     .i8_type()
    //     .ptr_type(AddressSpace::from(0))
    //     .fn_type(
    //         &[gc.context.i8_type().ptr_type(AddressSpace::from(0)).into()],
    //         false,
    //     )
    //     .ptr_type(AddressSpace::from(0));
    // let run_task_func_ptr: inkwell::values::GlobalValue = gc.module.add_global(
    //     run_function_func_ptr_ty,
    //     Some(AddressSpace::from(0)),
    //     "ptr_fixruntime_run_function",
    // );
    // run_task_func_ptr.set_externally_initialized(true);
    // run_task_func_ptr.set_linkage(Linkage::External);
    // let run_function_func = gc.module.get_function(RUNTIME_RUN_FUNCTION).unwrap();
    // gc.builder().build_store(
    //     run_task_func_ptr.as_pointer_value(),
    //     run_function_func.as_global_value().as_pointer_value(),
    // );

    // If both of `AsyncTask` and sanitizer are used, prepare for terminating threads.
    if config.async_task && config.sanitize_memory {
        gc.call_runtime(RUNTIME_THREAD_PREPARE_TERMINATION, &[]);
    }

    // Run main object.
    let main_obj = gc.eval_expr(main_expr, None); // `IO ()`
    let main_lambda_val = main_obj.load_field_nocap(gc, 0);
    let main_lambda_ty = type_fun(make_tuple_ty(vec![]), make_tuple_ty(vec![]));
    let main_lambda = Object::create_from_value(main_lambda_val, main_lambda_ty, gc);
    let unit = allocate_obj(
        make_tuple_ty(vec![]),
        &vec![],
        None,
        gc,
        Some("unit_for_main_io"),
    );
    let ret = gc.apply_lambda(main_lambda, vec![unit], None);
    gc.release(ret);

    // Perform leak check
    if config.should_terminate_tasks() {
        gc.call_runtime(RUNTIME_THREAD_TERMINATE, &[]);
    }
    gc.check_leak();

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)));
}

#[allow(dead_code)]
pub fn test_source(source: &str, mut config: Configuration) {
    const MAIN_RUN: &str = "main_run";
    let source_hash = format!("{:x}", md5::compute(source));
    save_temporary_source(source, MAIN_RUN, &source_hash);
    config.source_files = vec![temporary_source_path(MAIN_RUN, &source_hash)];
    assert_eq!(run_file(config), 0);
}

#[allow(dead_code)]
pub fn test_source_fail(source: &str, config: Configuration, contained_msg: &str) {
    let msg = catch_unwind(AssertUnwindSafe(|| {
        test_source(source, config);
    }))
    .unwrap_err()
    .downcast_ref::<String>()
    .unwrap()
    .clone();
    assert!(msg.contains(contained_msg));
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
    let res = PathBuf::new().join(rel_path);
    match create_dir_all(&res) {
        Err(why) => panic!("Failed to create directory {}: {}", res.display(), why),
        Ok(_) => {}
    };
    res
}

// Load all source files specified in the configuration, link them, and return the resulting `Program`.
pub fn load_source_files(config: &mut Configuration) -> Result<Program, Errors> {
    // Load all source files.
    let mut modules = vec![];
    let mut errors = Errors::empty();

    for file_path in &config.source_files {
        let res = parse_file_path(file_path.clone(), config);
        errors.eat_err_or(res, |prog| modules.push(prog));
    }

    // If an error occurres in parsing, return the error.
    errors.to_result()?;

    // Create `Std` module.
    let mut target_mod = make_std_mod(config)?;

    // Link all modules.
    for mod_ in modules {
        target_mod.link(mod_, false)?; // If an error occurres in linking, return the error.
    }

    // Resolve imports.
    target_mod.resolve_imports(config)?;

    Ok(target_mod)
}

// Run the program specified in the configuration, and return the exit code.
pub fn run_file(mut config: Configuration) -> i32 {
    fs::create_dir_all(DOT_FIXLANG).expect("Failed to create \".fixlang\" directory.");

    // For parallel execution, use different file name for each execution.
    let a_out_path: String = format!("./{}/a{}.out", DOT_FIXLANG, rand::thread_rng().gen::<u64>());
    config.out_file_path = Some(PathBuf::from(a_out_path.clone()));

    // Build executable file.
    exit_if_err(build_file(&mut config));

    // Run the executable file.
    let mut com = if config.valgrind_tool == ValgrindTool::None {
        Command::new(a_out_path.clone())
    } else {
        let mut com = config.valgrind_command();
        com.arg(a_out_path.clone());
        com
    };
    com.stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit());
    let output = com
        .output()
        .expect(&format!("Failed to run \"{}\".", a_out_path));

    // Remove the executable file.
    fs::remove_file(a_out_path.clone()).expect(&format!("Failed to remove \"{}\".", a_out_path));

    if let Some(code) = output.status.code() {
        code
    } else {
        error_exit("Program terminated abnormally.")
    }
}

fn get_target_machine(opt_level: OptimizationLevel, config: &Configuration) -> TargetMachine {
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
    let mut features = CpuFeatures::parse(TargetMachine::get_host_cpu_features().to_str().unwrap());
    config.edit_features(&mut features);
    let target_machine = target.create_target_machine(
        &triple,
        cpu_name.to_str().unwrap(),
        &features.to_string(),
        opt_level,
        RelocMode::Default,
        CodeModel::Default,
    );
    match target_machine {
        Some(tm) => tm,
        None => error_exit("Failed to create target machine."),
    }
}

pub fn build_file(config: &mut Configuration) -> Result<(), Errors> {
    let exec_path = config.get_output_executable_file_path();

    // Create intermediate directory.
    fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate .");

    let program = load_source_files(config)?;

    let obj_paths = build_object_files(program, config.clone())?;

    // If the program is for language server, we don't need to build binary file.
    if config.language_server_mode {
        return Ok(());
    }

    let mut library_search_path_opts: Vec<String> = vec![];
    for path in &config.library_search_paths {
        library_search_path_opts.push(format!("-L{}", path.to_str().unwrap()));
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
        // Random number for temporary file name.
        // This is necessary to avoid confliction when multiple compilation processes are running in parallel.
        let rand_num = rand::thread_rng().gen::<u64>();

        // Create temporary file.
        let runtime_tmp_path = runtime_obj_path.with_extension(rand_num.to_string() + ".tmp");

        let runtime_c_path =
            PathBuf::from(INTERMEDIATE_PATH).join(format!("fixruntime.{}.c", rand_num.to_string()));
        fs::create_dir_all(INTERMEDIATE_PATH).expect("Failed to create intermediate directory.");
        fs::write(&runtime_c_path, include_str!("runtime.c"))
            .expect(&format!("Failed to generate {}", runtime_c_path.display()));
        // Create library object file.
        let mut com = Command::new("gcc");
        let mut com = com
            .arg("-ffunction-sections")
            .arg("-fdata-sections")
            .arg("-o")
            .arg(runtime_tmp_path.to_str().unwrap())
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

        // Rename the temporary file to the final file.
        fs::rename(&runtime_tmp_path, &runtime_obj_path).expect(&format!(
            "Failed to rename {} to {}",
            runtime_tmp_path.display(),
            runtime_obj_path.display()
        ));
    }

    let mut com = Command::new("gcc");
    com.arg("-Wno-unused-command-line-argument").arg("-no-pie");
    if std::env::consts::OS == "macos" {
        com.arg("-Wl,-dead_strip");
    } else {
        com.arg("-Wl,--gc-sections");
    }
    com.arg("-o").arg(exec_path.to_str().unwrap());
    for obj_path in obj_paths {
        com.arg(obj_path.to_str().unwrap());
    }
    com.arg(runtime_obj_path.to_str().unwrap())
        .args(library_search_path_opts)
        .args(libs_opts);
    let output = com.output().expect("Failed to run gcc.");
    if output.stderr.len() > 0 {
        eprintln!(
            "{}",
            String::from_utf8(output.stderr)
                .unwrap_or("(failed to parse stderr from gcc as UTF8.)".to_string())
        );
    }

    Ok(())
}

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    let _ = remove_dir_all(DOT_FIXLANG);
}
