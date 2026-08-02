use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::build::build_object_files::build_object_files;
use crate::configuration::{Configuration, LinkType, OutputFileType};
use crate::constants::{INTERMEDIATE_PATH, MARK_THREADED_NAME, STD_NAME};
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::misc::{info_msg, Set};
use crate::parse::sourcefile::Span;
use build_time::build_time_utc;
use rand::Rng;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Reports the calls of `Std::mark_threaded` a program makes when multi-threading is off.
///
/// Multi-threading is what gives an object a mode to be put into, so `Std::mark_threaded` has
/// nothing to work with without it. The setting belongs to the program being built, so a library
/// that needs multi-threading reaches the user through this: the calls reported are the ones asking
/// for the setting, in the files they were written in.
///
/// The program is checked before it is optimized, while each expression still carries the source it
/// came from.
fn check_multithreading_requirement(
    program: &Program,
    config: &Configuration,
) -> Result<(), Errors> {
    if config.threaded {
        return Ok(());
    }
    let mark_threaded = FullName::from_strs(&[STD_NAME], MARK_THREADED_NAME);
    // A generic value is instantiated once per type it is used at, and every instance answers to
    // the name it was written as.
    let instances = program
        .symbols
        .values()
        .filter(|symbol| symbol.generic_name == mark_threaded)
        .map(|symbol| symbol.name.clone())
        .collect::<Set<_>>();
    if instances.is_empty() {
        return Ok(());
    }
    let mut uses: Vec<(&FullName, Option<Span>)> = vec![];
    for symbol in program.symbols.values() {
        let expr = symbol.expr.as_ref().unwrap();
        expr.walk_var_uses(&mut |var, src| {
            if instances.contains(&var.name) {
                uses.push((&symbol.name, src.clone()));
            }
        });
    }
    // The symbols are held in a map, so an order is chosen here to keep the report the same from
    // one build to the next.
    uses.sort_by(|a, b| a.0.cmp(b.0));
    let srcs = uses.iter().map(|(_, src)| src).collect::<Vec<_>>();
    Err(Errors::from_msg_srcs(
        format!(
            "`{}` requires multi-threading. Enable it by `threaded = true` in the project file of \
             the program being built, or by the `--threaded` compiler option.",
            mark_threaded.to_string()
        ),
        &srcs,
    ))
}

// Run `gcc` as prepared in `com`, passing on whatever it writes to standard error.
fn run_gcc(com: &mut Command) {
    let output = com.output().expect("Failed to run gcc.");
    if output.stderr.len() > 0 {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

// Build the program specified in the configuration.
pub fn build(config: &Configuration) -> Result<(), Errors> {
    assert!(config.subcommand.build_binary());

    let mut config = config.clone();

    let out_path = config.get_output_file_path();

    // Run preliminary commands.
    if config.subcommand.run_preliminary_commands() {
        config.run_preliminary_commands()?;
    }

    let mut program = elaborate_via_config(&config)?;
    program.flush_warnings_to_stderr();
    // Surface any errors that were deferred to the diagnostic stage —
    // most importantly, deprecation diagnostics promoted to errors by
    // `--deny-deprecated`.
    if program.deferred_errors.has_error() {
        return Err(program.deferred_errors);
    }
    check_multithreading_requirement(&program, &config)?;
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
        fs::write(&runtime_c_path, include_str!("../fixstd/runtime.c")).expect(&format!(
            "Failed to generate \"{}\"",
            runtime_c_path.to_string_lossy().to_string()
        ));
        // Create library object file.
        let mut com = Command::new("gcc");
        let mut com = com.arg("-ffunction-sections").arg("-fdata-sections");
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
        run_gcc(com);

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
    run_gcc(&mut com);

    Ok(())
}
