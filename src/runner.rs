use crate::build_object_files::build_object_files;
use crate::constants::RUN_PATH;
use crate::error::{any_to_string, panic_if_err, panic_with_err, Errors};
use crate::make_std_mod;
use crate::make_tuple_traits_mod;
use crate::misc::{info_msg, save_temporary_source};
use crate::parse_file_path;
use crate::stopwatch::StopWatch;
use crate::Configuration;
use crate::LinkType;
use crate::OutputFileType;
use crate::Program;
use crate::SubCommand;
use crate::ValgrindTool;
use crate::DOT_FIXLANG;
use crate::INTERMEDIATE_PATH;
use build_time::build_time_utc;
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{
    fs::{self, create_dir_all, remove_dir_all},
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    process::{Command, Stdio},
};

// Perform validations and type checking on the program, and return the updated program.
// Changes made to the program include instantiation of symbols and setting of entry points.
fn check_program(mut program: Program, config: &Configuration) -> Result<Program, Errors> {
    let _sw = StopWatch::new("check_program", config.show_build_times);

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
    program.resolve_namespace_not_in_expr()?;

    // Resolve type aliases that appear in declarations and associated type implementations.
    program.resolve_type_aliases_not_in_expr()?;

    // Validate user-defined types.
    program.validate_type_defns()?;

    // Add struct / union methods
    program.add_methods()?;

    // Add `Std::Boxed` trait implementations.
    program.add_boxed_impls()?;

    // Validate trait env.
    program.validate_trait_env()?;

    // Create symbols.
    program.create_trait_member_symbols();

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
        return Ok(program);
    }

    let typechecker = program.create_typechecker(config);

    // When running diagnostics, perform type checking of target modules and return here.
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        let _sw = StopWatch::new("typecheck", config.show_build_times);
        let modules = program.modules_from_files(&diag_config.files)?;
        let mut errors = Errors::empty();
        errors.eat_err(program.resolve_namespace_and_check_type_in_modules(&typechecker, &modules));
        program.deferred_errors.append(errors);
        return Ok(program);
    }

    // Instantiate Main::main (or Test::test).
    match config.output_file_type {
        OutputFileType::Executable => program.instantiate_entry_io_value(
            &typechecker,
            matches!(config.subcommand, SubCommand::Test),
        )?,
        OutputFileType::DynamicLibrary => {}
    };

    // Instantiate all exported values and values called from them.
    program.instantiate_exported_values(&typechecker)?;

    Ok(program)
}

#[allow(dead_code)]
pub fn test_source(source: &str, mut config: Configuration) {
    const MAIN_RUN: &str = "main_run";
    let src = save_temporary_source(source, MAIN_RUN).ok().unwrap();
    config.source_files.push(src.file_path);
    assert_eq!(run(config), 0);
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
pub fn load_source_files(config: &Configuration) -> Result<Program, Errors> {
    // Create `Std` module.
    let mut program = make_std_mod(config)?;

    // Parse all source files.
    let mut modules = vec![];
    let mut errors = Errors::empty();
    for file_path in &config.source_files {
        let res = parse_file_path(file_path.clone(), config);
        errors.eat_err_or(res, |mod_| modules.push(mod_));
    }

    // If an error occurres in parsing,
    if let SubCommand::Diagnostics(diag_config) = &config.subcommand {
        // In eny parsing error occurres in diagnostics mode, delay the error and remove the root project from modules.
        // In other words, in the following diagnostic process, only the dependent projects are targeted.
        // This allows us to give the language server the information it needs for code completion, even if there is a parse error in the root project.
        if errors.has_error() {
            let mut dependent_projects = vec![];
            for mod_ in modules {
                let mods = mod_.modules_from_files(&diag_config.files)?;
                if mods.is_empty() {
                    dependent_projects.push(mod_);
                }
            }
            modules = dependent_projects;
        }
        program.deferred_errors.append(errors);
    } else {
        // In usual compilation, raise the error.
        errors.to_result()?;
    }

    // Link all modules.
    for mod_ in modules {
        program.link(mod_, false)?; // If an error occurres in linking, return the error.
    }

    // Resolve imports.
    program.check_imports()?;

    Ok(program)
}

// Run the program specified in the configuration, and return the exit code.
pub fn run(mut config: Configuration) -> i32 {
    fs::create_dir_all(DOT_FIXLANG)
        .expect(format!("Failed to create \"{}\" directory.", DOT_FIXLANG).as_str());
    fs::create_dir_all(RUN_PATH)
        .expect(format!("Failed to create \"{}\" directory.", RUN_PATH).as_str());

    // For parallel execution, use different file name for each execution.
    let exec_path: String = format!("{}/a{}.out", RUN_PATH, rand::thread_rng().gen::<u64>());
    let user_specified_out_path = std::mem::replace(
        &mut config.out_file_path,
        Some(PathBuf::from(exec_path.clone())),
    );

    // Build executable file.
    panic_if_err(build(&mut config));

    // Run the executable file.
    let mut com = if config.valgrind_tool == ValgrindTool::None {
        Command::new(exec_path.clone())
    } else {
        let mut com = config.valgrind_command();
        com.arg(exec_path.clone());
        com
    };
    for arg in &config.run_program_args {
        com.arg(arg);
    }
    com.stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit());
    let output = com.output();

    // Clean up the temporary executable file.
    match user_specified_out_path {
        Some(out_path) => {
            // Move the temporary executable file to the specified output file.
            if let Err(e) = fs::rename(exec_path.clone(), out_path.clone()) {
                let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
                panic_with_err(&format!(
                    "Failed to rename \"{}\" to \"{}\": {}",
                    exec_path,
                    out_path.display(),
                    e
                ));
            }
        }
        None => {
            // If the output file is not specified, remove the temporary executable file.
            let _ = fs::remove_file(exec_path.clone()); // Ignore the error.
        }
    }

    if let Err(e) = output {
        panic_with_err(&format!("Failed to run \"{}\": {}", exec_path, e));
    }
    let output = output.unwrap();

    if let Some(code) = output.status.code() {
        code
    } else {
        panic_with_err("Program terminated abnormally.")
    }
}

// Load the program specified by the Configuration, perform validations and type checking.
pub fn check_program_via_config(config: &Configuration) -> Result<Program, Errors> {
    let program = load_source_files(&config)?;
    let program = check_program(program, config)?;
    Ok(program)
}

// Build the program specified in the configuration.
pub fn build(config: &Configuration) -> Result<(), Errors> {
    assert!(config.subcommand.build_binary());

    let mut config = config.clone();

    let out_path = config.get_output_file_path();

    // Run extra commands.
    if config.subcommand.run_preliminary_commands() {
        config.run_extra_commands()?;
    }

    let program = check_program_via_config(&config)?;
    let obj_files = build_object_files(program, &config)?;

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
                info_msg("On MacOS, it is not possible to specify whether a library should be dynamically or statically linked. \
                If a dynamic library and a static library with the same name exist, the unintended one may be used.");
                warned_on_mac = true;
            }
        }
        libs_opts.push(format!("-l{}", lib_name));
    }
    for ld_flag in &config.ld_flags {
        libs_opts.push(ld_flag.clone());
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
            .arg("-fdata-sections");
        // Keep frame pointers for better backtraces on macOS when backtrace is enabled
        if config.no_elim_frame_pointers() {
            com = com.arg("-fno-omit-frame-pointer");
        }
        let mut com = com
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

    let mut obj_paths = obj_files.obj_paths;
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

    Ok(())
}

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    let _ = remove_dir_all(DOT_FIXLANG);
}
