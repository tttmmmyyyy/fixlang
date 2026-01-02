use std::{
    fs::{self, create_dir_all},
    panic::panic_any,
    path::{Path, PathBuf},
    sync::Arc,
};

use inkwell::{
    context::Context,
    module::Module,
    passes::PassBuilderOptions,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
    values::BasicValue,
    AddressSpace, OptimizationLevel,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    ast::{export_statement::ExportStatement, expr::ExprNode, program::Program},
    builtin::run_io_or_ios_runner,
    compile_unit::CompileUnit,
    configuration::{Configuration, FixOptimizationLevel, OutputFileType},
    constants::{GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV, UNITS_CACHE_PATH},
    cpu_features::CpuFeatures,
    error::{panic_with_err, Errors},
    generator::GenerationContext,
    misc::warn_msg,
    optimization,
    runtime::{self, BuildMode},
    stopwatch::StopWatch,
};

// The result of `build_object_files` function.
#[derive(Clone, Serialize, Deserialize)]
pub struct BuildObjFilesResult {
    // Paths of object files generated.
    // If the function is running for language server, this will be empty.
    pub obj_paths: Vec<PathBuf>,
}

// Compile the program, and returns the path of object files to be linked.
pub fn build_object_files<'c>(
    mut program: Program,
    config: &Configuration,
) -> Result<BuildObjFilesResult, Errors> {
    let _sw = StopWatch::new("build_object_files", config.show_build_times);

    // Return cached object files if available.
    // This cache is especially effective when running "fix run" repeatedly without editing the source code.
    if let Some(cached) = load_build_object_files_cache(&program, config) {
        if config.verbose {
            eprintln!("Using cached object files.");
        }
        return Ok(cached);
    }

    // Run optimizations.
    optimization::optimization::run(&mut program, &config);

    // Determine compilation units.
    let mut units = vec![];
    let mut symbols = program.symbols.values().cloned().collect::<Vec<_>>();
    symbols.sort_by(|a, b| a.name.cmp(&b.name));
    let all_symbols = symbols.clone();
    {
        let module_dependency_hash = program.module_dependency_hash_map();
        let module_dependency_map = program.module_dependency_map();
        let modules = program.linked_mods().iter().cloned().collect::<Vec<_>>();
        if config.enable_separated_compilation() {
            units = CompileUnit::split_symbols(
                symbols,
                &module_dependency_hash,
                &module_dependency_map,
                &config,
            );
            // Also add main compilation unit.
            // The main unit implements the entry point of exported functions.
            // Therefore, the main unit is treated as depending on all modules.
            let mut main_unit = CompileUnit::new(vec![], modules);
            main_unit.update_unit_hash(&module_dependency_hash, &config);
            units.push(main_unit);
        } else {
            // Add main compilation unit, which includes all symbols.
            let mut main_unit = CompileUnit::new(symbols, modules);
            main_unit.update_unit_hash(&module_dependency_hash, &config);
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

        let entry_io_value = program.entry_io_value.clone();
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
                // Implement runtime functions.
                runtime::build_runtime(&mut gc, BuildMode::Implement);

                // Implement exported C functions.
                build_exported_c_functions(&mut gc, &export_statements);

                // Implement the `main()` function.
                if let Some(main_expr) = entry_io_value {
                    build_main_function(&mut gc, main_expr.clone());
                }
            }

            // If debug info is generated, finalize it.
            gc.finalize_di();

            // Add frame-pointer attribute to all functions for better backtraces on macOS
            if config.no_elim_frame_pointers() {
                gc.add_frame_pointer_attribute_to_all_functions();
            }

            if config.emit_llvm {
                // Print LLVM-IR to file before optimization.
                emit_llvm(gc.module, &config, false);
            }

            // LLVM level optimization.
            optimize_and_verify(gc.module, &target_machine, &config);

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
        if let Err(e) = t.join() {
            panic_any(e);
        }
    }

    // Save object files cache.
    let result = BuildObjFilesResult { obj_paths };
    save_build_object_files_cache(&program, config, &result);

    Ok(result)
}

// Load cache of "build_object_files" function.
fn load_build_object_files_cache(
    program: &Program,
    config: &Configuration,
) -> Option<BuildObjFilesResult> {
    let hash = build_object_files_cache_hash(program, config);
    if let Err(e) = hash {
        warn_msg(&format!(
            "Failed to calculate hash of object files cache: {}.",
            e
        ));
        return None;
    }
    let hash = hash.ok().unwrap();
    let cache_path = format!("{}/{}.json", UNITS_CACHE_PATH, hash);
    if !Path::new(&cache_path).exists() {
        return None;
    }
    let file = std::fs::File::open(&cache_path);
    if let Err(e) = file {
        warn_msg(&format!(
            "Failed to open object files cache \"{}\": {}.",
            cache_path, e
        ));
        return None;
    }
    let file = file.ok().unwrap();
    let result = serde_json::from_reader(file);
    if let Err(e) = result {
        warn_msg(&format!(
            "Failed to read object files cache \"{}\": {}.",
            cache_path, e
        ));
        return None;
    }
    let cache: BuildObjFilesResult = result.ok().unwrap();
    // Check all files in the cache exist.
    for path in &cache.obj_paths {
        if !path.exists() {
            return None;
        }
    }
    Some(cache)
}

// Save cache of "build_object_files" function.
fn save_build_object_files_cache(
    program: &Program,
    config: &Configuration,
    result: &BuildObjFilesResult,
) {
    let hash = build_object_files_cache_hash(program, config);
    if let Err(e) = hash {
        warn_msg(&format!(
            "Failed to calculate hash of object files cache: {}.",
            e
        ));
        return;
    }
    let hash = hash.ok().unwrap();
    if let Err(e) = create_dir_all(UNITS_CACHE_PATH) {
        warn_msg(&format!(
            "Failed to create directory for object files cache: {}.",
            e
        ));
        return;
    }
    let cache_path = format!("{}/{}.json", UNITS_CACHE_PATH, hash);
    let file = std::fs::File::create(&cache_path);
    if let Err(e) = file {
        warn_msg(&format!(
            "Failed to create object files cache \"{}\": {}.",
            cache_path, e
        ));
        return;
    }
    let file = file.ok().unwrap();
    let res = serde_json::to_writer_pretty(file, result);
    if let Err(e) = res {
        warn_msg(&format!(
            "Failed to write object files cache \"{}\": {}.",
            cache_path, e
        ));
        return;
    }
}

// Calculate hash used for cache of "build_object_files" function.
fn build_object_files_cache_hash(
    program: &Program,
    config: &Configuration,
) -> Result<String, Errors> {
    let mut hash_source = "".to_string();
    hash_source += "<configuration>";
    hash_source += &config.object_generation_hash();

    hash_source += "<sources>";
    for mi in &program.modules {
        hash_source += &mi.source.input.hash()?;
    }

    Ok(format!("{:x}", md5::compute(hash_source)))
}

fn get_target_machine(opt_level: OptimizationLevel, config: &Configuration) -> TargetMachine {
    let _native = Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| panic_with_err(&format!("failed to initialize native: {}", e)))
        .unwrap();
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple)
        .map_err(|e| {
            panic_with_err(&format!("failed to create target: {}", e));
        })
        .unwrap();
    let cpu_name = TargetMachine::get_host_cpu_name();
    let mut features = CpuFeatures::parse(TargetMachine::get_host_cpu_features().to_str().unwrap());
    config.edit_cpu_features(&mut features);
    let reloc_mode = if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
        RelocMode::PIC
    } else {
        RelocMode::Default
    };
    let target_machine = target.create_target_machine(
        &triple,
        cpu_name.to_str().unwrap(),
        &features.to_string(),
        opt_level,
        reloc_mode,
        CodeModel::Default,
    );
    match target_machine {
        Some(tm) => tm,
        None => panic_with_err("Failed to create target machine."),
    }
}

fn write_to_object_file<'c>(module: &Module<'c>, target_machine: &TargetMachine, obj_path: &Path) {
    // Create directory if it doesn't exist.
    let dir_path = obj_path.parent().unwrap();
    match fs::create_dir_all(dir_path) {
        Err(e) => {
            panic_with_err(&format!(
                "Failed to create directory \"{}\": {}",
                dir_path.to_string_lossy().to_string(),
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
            panic_with_err(&format!(
                "Failed to write to file \"{}\": {}",
                obj_path.to_string_lossy().to_string(),
                e
            ))
        })
        .unwrap();

    // Rename the temporary file to the final file.
    match fs::rename(&tmp_file_path, obj_path) {
        Err(e) => {
            panic_with_err(&format!(
                "Failed to rename \"{}\" to \"{}\": {}",
                tmp_file_path.to_string_lossy().to_string(),
                obj_path.to_string_lossy().to_string(),
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
        panic_with_err(&format!("Failed to emit LLVM-IR: {}", e.to_string()));
    }
}

fn optimize_and_verify<'c>(
    module: &Module<'c>,
    target_machine: &TargetMachine,
    config: &Configuration,
) {
    fn run_passes_or_panic(module: &Module, passes: &[&str], target_machine: &TargetMachine) {
        for pass in passes {
            if let Err(e) = module.run_passes(pass, target_machine, PassBuilderOptions::create()) {
                panic_with_err(&format!(
                    "Failed to run pass \"{}\": {}",
                    pass,
                    e.to_string()
                ));
            }
        }
    }

    // Get passes.
    let passes = match &config.llvm_passes_file {
        None => include_str!("llvm_passes.txt").to_string(),
        Some(file) => std::fs::read_to_string(file).unwrap(),
    };
    let passes = passes
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    // Run optimization
    run_passes_or_panic(module, &["verify"], target_machine);

    match config.fix_opt_level {
        FixOptimizationLevel::None => {}
        FixOptimizationLevel::Basic => {
            run_passes_or_panic(module, &passes, target_machine);
        }
        FixOptimizationLevel::Max => {
            run_passes_or_panic(module, &["default<O3>"], target_machine);
            run_passes_or_panic(module, &passes, target_machine);
        }
        FixOptimizationLevel::Experimental => {
            run_passes_or_panic(module, &["default<O3>"], target_machine);
            run_passes_or_panic(module, &passes, target_machine);
        }
    }
    run_passes_or_panic(module, &["verify"], target_machine);
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

fn build_main_function<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, main_expr: Arc<ExprNode>) {
    let main_fn_type = gc.context.i32_type().fn_type(
        &[
            gc.context.i32_type().into(),                      // argc
            gc.context.ptr_type(AddressSpace::from(0)).into(), // argv
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
        gc.builder().build_store(gv_ptr, arg_val).unwrap();
    }

    // Run main object.
    let main_obj = gc.eval_expr(main_expr, false).unwrap(); // A value of type `IO ()`.
    run_io_or_ios_runner(gc, &main_obj);

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)))
        .unwrap();
}
