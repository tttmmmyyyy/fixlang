use build_time::build_time_utc;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::BasicValue;
use inkwell::{
    passes::PassManager,
    targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
};
use inkwell::{AddressSpace, OptimizationLevel};
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::panic::panic_any;
use std::path::Path;
use std::{
    fs::{self, create_dir_all, remove_dir_all},
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};

use crate::ast::export_statement::ExportStatement;
use crate::compile_unit::CompileUnit;
use crate::cpu_features::CpuFeatures;
use crate::error::{any_to_string, panic_if_err, panic_with_err, Errors};
use crate::misc::{info_msg, save_temporary_source, temporary_source_path};
use crate::stopwatch::StopWatch;
use crate::ExprNode;
use crate::FixOptimizationLevel;
use crate::GenerationContext;
use crate::LinkType;
use crate::OutputFileType;
use crate::Program;
use crate::SubCommand;
use crate::TypeCheckContext;
use crate::ValgrindTool;
use crate::DOT_FIXLANG;
use crate::GLOBAL_VAR_NAME_ARGC;
use crate::GLOBAL_VAR_NAME_ARGV;
use crate::INTERMEDIATE_PATH;
use crate::{build_runtime, parse_file_path};
use crate::{llvm_passes, run_io_value};
use crate::{make_std_mod, runtime};
use crate::{make_tuple_traits_mod, BuildMode};
use crate::{optimization, Configuration};

// The result of `build_object_files` function.
pub struct BuildObjFilesResult {
    // Paths of object files generated.
    // If the function is running for language server, this will be empty.
    obj_paths: Vec<PathBuf>,

    // The program parsed.
    // This field is only set when the function is running for language server.
    program: Option<Program>,
}

// Compile the program, and returns the path of object files to be linked.
fn build_object_files<'c>(
    mut program: Program,
    config: Configuration,
) -> Result<BuildObjFilesResult, Errors> {
    let _sw = StopWatch::new("build_module", config.show_build_times);

    // Add tuple definitions.
    program.add_tuple_defns();

    // Add trait implementations for tuples such as ToString or Eq.
    program.link(
        make_tuple_traits_mod(&program.used_tuple_sizes, &config)?,
        true,
    )?;

    // Validate export statements.
    program.validate_export_statements()?;

    // Calculate list of type constructors.
    program.calculate_type_env()?;

    // Validate name confliction between types, traits and global values.
    program.validate_capital_name_confliction()?;

    // Infer namespaces of traits and types that appear in declarations and associated type implementations.
    program.resolve_namespace_in_type_signs()?;

    // Resolve type aliases that appear in declarations and associated type implementations.
    program.resolve_type_aliases_in_declaration()?;

    // Validate user-defined types.
    program.validate_type_defns()?;

    // Add struct / union methods
    program.add_methods()?;

    // Add `Std::Boxed` trait implementations.
    program.add_boxed_impls()?;

    // Validate trait env.
    program.validate_trait_env()?;

    // Create symbols.
    program.create_trait_method_symbols();

    // Validate constraints of global value type.
    program.validate_global_value_type_constraints()?;

    // Check if all items referred in import statements are defined.
    // This check should be done after `add_methods` and `create_trait_method_symbols`.
    program.validate_import_statements()?;

    // Set and check kinds that appear in type signatures.
    // NOTE: kinds of type variables appearing in type annotations in expressions are set not at this stage but at the type inference stage.
    program.set_kinds()?;

    // If typechecking is not needed, return here.
    if !config.subcommand.typecheck() {
        assert!(!config.subcommand.build_binary());
        return Ok(BuildObjFilesResult {
            obj_paths: vec![],
            program: Some(program),
        });
    }

    // Create typeckecker.
    let mut typechecker = TypeCheckContext::new(
        program.trait_env.clone(),
        program.type_env(),
        program.kind_env(),
        program.mod_to_import_stmts.clone(),
        config.type_check_cache.clone(),
        config.num_worker_thread,
    );

    // Register type declarations of global symbols to typechecker.
    {
        let globals = program
            .global_values
            .iter()
            .map(|(name, defn)| (name.clone(), defn.scm.clone()))
            .collect::<Vec<_>>();
        typechecker.scope.set_globals(globals);
    }

    // When running diagnostics, perform type checking of target modules and return here.
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        let _sw = StopWatch::new("typecheck", config.show_build_times);
        let modules = program.modules_from_files(&diag_config.files);
        program.resolve_namespace_and_check_type_in_modules(&typechecker, &modules)?;
        return Ok(BuildObjFilesResult {
            obj_paths: vec![],
            program: Some(program),
        });
    }

    // Instantiate Main::main (or Test::test).
    let main_expr = match config.output_file_type {
        OutputFileType::Executable => {
            let main_expr = program.instantiate_main_function(
                &typechecker,
                matches!(config.subcommand, SubCommand::Test),
            )?;
            Some(main_expr)
        }
        OutputFileType::DynamicLibrary => None,
    };

    // Instantiate all exported values and values called from them.
    program.instantiate_exported_values(&typechecker)?;

    optimization::root::optimize(&mut program, &config);

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

        let entry_expr = main_expr.clone();
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
                if let Some(main_expr) = entry_expr {
                    build_main_function(&mut gc, main_expr.clone());
                }
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
        if let Err(e) = t.join() {
            panic_any(e);
        }
    }

    Ok(BuildObjFilesResult {
        obj_paths,
        program: None,
    })
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
            llvm_passes::add_basic_optimization_passes(&passmgr);
            llvm_passes::add_optimized_optimization_passes(&passmgr, &config.llvm_passes_file);
        }
        FixOptimizationLevel::Default => {
            llvm_passes::add_basic_optimization_passes(&passmgr);
            llvm_passes::add_optimized_optimization_passes(&passmgr, &config.llvm_passes_file);
            llvm_passes::add_strip_passes(&passmgr);
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

fn build_main_function<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, main_expr: Arc<ExprNode>) {
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

    // Run main object.
    let main_obj = gc.eval_expr(main_expr, None); // `IO ()`
    run_io_value(gc, &main_obj, None);

    // Return main function.
    gc.builder()
        .build_return(Some(&gc.context.i32_type().const_int(0, false)));
}

#[allow(dead_code)]
pub fn test_source(source: &str, mut config: Configuration) {
    const MAIN_RUN: &str = "main_run";
    let source_hash = format!("{:x}", md5::compute(source));
    save_temporary_source(source, MAIN_RUN, &source_hash);
    config
        .source_files
        .push(temporary_source_path(MAIN_RUN, &source_hash));
    assert_eq!(run_file(config), 0);
}

#[allow(dead_code)]
pub fn test_source_fail(source: &str, config: Configuration, contained_msg: &str) {
    let any = catch_unwind(AssertUnwindSafe(|| {
        test_source(source, config);
    }))
    .unwrap_err();
    let msg = any_to_string(any.as_ref());
    assert!(msg.contains(contained_msg));
}

// Return file content and last modified.
pub fn read_file(path: &Path) -> Result<String, String> {
    let mut file = match File::open(&path) {
        Err(why) => {
            return Err(format!(
                "Couldn't open \"{}\": {}",
                path.to_string_lossy().to_string(),
                why
            ))
        }
        Ok(file) => file,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => {
            return Err(format!(
                "Couldn't read \"{}\": {}",
                path.to_string_lossy().to_string(),
                why
            ))
        }
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
        Err(why) => panic!(
            "Failed to create directory \"{}\": {}",
            res.to_string_lossy().to_string(),
            why
        ),
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
    target_mod.check_imports()?;

    Ok(target_mod)
}

// Run the program specified in the configuration, and return the exit code.
pub fn run_file(mut config: Configuration) -> i32 {
    fs::create_dir_all(DOT_FIXLANG).expect("Failed to create \".fixlang\" directory.");

    // For parallel execution, use different file name for each execution.
    let a_out_path: String = format!("./{}/a{}.out", DOT_FIXLANG, rand::thread_rng().gen::<u64>());
    config.out_file_path = Some(PathBuf::from(a_out_path.clone()));

    // Build executable file.
    panic_if_err(build_file(&mut config));

    // Run the executable file.
    let mut com = if config.valgrind_tool == ValgrindTool::None {
        Command::new(a_out_path.clone())
    } else {
        let mut com = config.valgrind_command();
        com.arg(a_out_path.clone());
        com
    };
    for arg in &config.run_program_args {
        com.arg(arg);
    }
    com.stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit());
    let output = com.output();

    // Remove the executable file.
    let _ = fs::remove_file(a_out_path.clone()); // Ignore the error.

    if let Err(e) = output {
        panic_with_err(&format!("Failed to run \"{}\": {}", a_out_path, e));
    }
    let output = output.unwrap();

    if let Some(code) = output.status.code() {
        code
    } else {
        panic_with_err("Program terminated abnormally.")
    }
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
    config.edit_features(&mut features);
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

// The result of `build_file` function.
#[derive(Default)]
pub struct BuildFileResult {
    // The program parsed.
    // This field is only set when the function is running for language server.
    pub program: Option<Program>,
}

pub fn build_file(config: &mut Configuration) -> Result<BuildFileResult, Errors> {
    let out_path = config.get_output_file_path();

    // Run extra commands.
    if config.subcommand.run_preliminary_commands() {
        config.run_extra_commands()?;
    }

    // Create intermediate directory.
    fs::create_dir_all(INTERMEDIATE_PATH).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to create directory \"{}\": {:?}",
            INTERMEDIATE_PATH, e
        ))
    })?;

    let program = load_source_files(config)?;
    let build_res = build_object_files(program, config.clone())?;

    // In case we don't need to build binary file, return here.
    if !config.subcommand.build_binary() {
        let program = build_res.program.unwrap();
        return Ok(BuildFileResult {
            program: Some(program),
        });
    }

    let mut library_search_path_opts: Vec<String> = vec![];
    for path in &config.library_search_paths {
        library_search_path_opts.push(format!("-L{}", path.to_str().unwrap()));
    }
    let mut libs_opts = vec![];
    let mut warned_on_mac = false;
    for (lib_name, link_type) in &config.linked_libraries {
        if std::env::consts::OS != "macos" {
            match link_type {
                LinkType::Static => libs_opts.push("-Wl,-Bstatic".to_string()),
                LinkType::Dynamic => libs_opts.push("-Wl,-Bdynamic".to_string()),
            }
        } else {
            if !warned_on_mac {
                info_msg("on macOS, it is not possible to specify whether a library should be dynamically or statically linked. \
                If a dynamic library and a static library with the same name exist, the unintended one may be used.");
                warned_on_mac = true;
            }
        }
        libs_opts.push(format!("-l{}", lib_name));
    }

    // Build runtime.c to object file.
    let mut runtime_obj_hash_source = "".to_string();
    runtime_obj_hash_source += build_time_utc!();
    runtime_obj_hash_source += &config.runtime_c_macro.join("_");
    runtime_obj_hash_source += config.output_file_type.to_str();
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
        fs::write(&runtime_c_path, include_str!("runtime.c")).expect(&format!(
            "Failed to generate \"{}\"",
            runtime_c_path.to_string_lossy().to_string()
        ));
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
        if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
            com = com.arg("-fPIC");
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
            "Failed to rename \"{}\" to \"{}\"",
            runtime_tmp_path.to_string_lossy().to_string(),
            runtime_obj_path.to_string_lossy().to_string()
        ));
    }

    let mut com = Command::new("gcc");
    com.arg("-Wno-unused-command-line-argument");
    if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
        com.arg("-shared");
    } else {
        com.arg("-no-pie");
    }
    if std::env::consts::OS == "macos" {
        com.arg("-Wl,-dead_strip");
    } else {
        com.arg("-Wl,--gc-sections");
    }
    com.arg("-o").arg(out_path.to_str().unwrap());

    let mut obj_paths = build_res.obj_paths;
    obj_paths.append(&mut config.object_files.clone());
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

    Ok(BuildFileResult::default())
}

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    let _ = remove_dir_all(DOT_FIXLANG);
}
