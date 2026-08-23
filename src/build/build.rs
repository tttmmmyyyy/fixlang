use crate::build::build_object_files::build_object_files;
use crate::configuration::{Configuration, LinkType, OutputFileType, Sanitizer};
use crate::constants::INTERMEDIATE_PATH;
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::hash::HashSource;
use crate::misc::info_msg;
use build_time::build_time_utc;
use rand::Rng;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The C compiler a build drives, prepared with the flags the configuration calls for.
///
/// A sanitized build goes through clang. The instrumentation the code generator inserts calls into
/// the sanitizer runtime, which ships with clang, and clang is what knows where to find it and how
/// to link it. Every other build goes through gcc.
fn c_compiler_command(config: &Configuration) -> Result<Command, Errors> {
    match config.sanitizer {
        Sanitizer::None => Ok(Command::new("gcc")),
        Sanitizer::Thread => {
            let mut com = Command::new(clang_path()?);
            com.arg("-fsanitize=thread");
            Ok(com)
        }
    }
}

/// The clang a sanitized build is compiled and linked by.
///
/// The instrumentation the code generator inserts calls into the sanitizer runtime, which is
/// distributed with clang. Taking the clang that sits beside the LLVM this compiler was built
/// against is what pairs the two: the instrumentation and the runtime answering it come from one
/// release.
fn clang_path() -> Result<PathBuf, Errors> {
    // `llvm-sys` names this after the LLVM release it links, which `Cargo.toml` pins through
    // inkwell's `llvm17-0` feature. Raising one without the other leaves this looking for a prefix
    // nothing sets, so say so rather than reach for whatever clang the path happens to hold.
    let Some(prefix) = option_env!("LLVM_SYS_170_PREFIX") else {
        return Err(Errors::from_msg(
            "This compiler was built without recording where its LLVM lives, so the clang a \
             sanitized build needs cannot be found. Build it with `LLVM_SYS_170_PREFIX` set."
                .to_string(),
        ));
    };
    let clang_beside_llvm = Path::new(prefix).join("bin").join("clang");
    if !clang_beside_llvm.exists() {
        return Err(Errors::from_msg(format!(
            "A sanitized build is compiled and linked by the clang beside the LLVM this compiler \
             was built against, and there is none at `{}`. The sanitizer runtime the \
             instrumentation calls into is distributed with clang, so the two have to come from \
             one release.",
            clang_beside_llvm.display()
        )));
    }
    Ok(clang_beside_llvm)
}

/// Runs a prepared C compiler command, passing on what it writes to standard error and reporting a
/// non-zero exit as a failure of `step`.
///
/// # Arguments
/// * `step` — what the invocation is for, as a verb phrase that completes "Failed to ...", so that
///   a failure says which of the build's several C compiler calls it was.
fn run_c_compiler(com: &mut Command, step: &str) -> Result<(), Errors> {
    let program = com.get_program().to_string_lossy().to_string();
    let output = com.output().map_err(|e| {
        Errors::from_msg(format!(
            "Failed to {}: could not run `{}`: {}.",
            step, program, e
        ))
    })?;
    if output.stderr.len() > 0 {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if !output.status.success() {
        return Err(Errors::from_msg(format!(
            "Failed to {}: {} exited with code {}.",
            step,
            program,
            output.status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

/// Builds the program specified in the configuration, linking the object files and the runtime into
/// the output file.
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
    program.check_multi_threading_requirement(&config)?;
    let obj_files = build_object_files(program, &config)?;

    let mut library_search_path_opts: Vec<String> = vec![];
    for path in &config.library_search_paths {
        library_search_path_opts.push(format!("-L{}", path.to_str().unwrap()));
    }
    let mut libs_opts = vec![];
    let mut warned_on_mac = false;
    for (lib_name, link_type) in &config.linked_libraries {
        if env::consts::OS != "macos" {
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
    let mut runtime_obj_hash_source = HashSource::default();
    runtime_obj_hash_source.push_text(build_time_utc!());
    runtime_obj_hash_source.push_list(&config.runtime_c_macro);
    runtime_obj_hash_source.push_text(config.output_file_type.to_str());
    // A sanitized build compiles the runtime with the instrumentation, so an object built without it
    // is a different object.
    runtime_obj_hash_source.push_text(&config.sanitizer.to_string());
    // A build keeping the frame pointers compiles the runtime keeping them too.
    runtime_obj_hash_source.push_text(&config.no_elim_frame_pointers().to_string());
    let runtime_obj_path = PathBuf::from(INTERMEDIATE_PATH)
        .join(format!("fixruntime.{}.o", runtime_obj_hash_source.finish()));
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
        let mut com = c_compiler_command(&config)?;
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
        run_c_compiler(com, "compile the runtime")?;

        // Rename the temporary file to the final file.
        fs::rename(&runtime_tmp_path, &runtime_obj_path).expect(&format!(
            "Failed to rename \"{}\" to \"{}\"",
            runtime_tmp_path.to_string_lossy().to_string(),
            runtime_obj_path.to_string_lossy().to_string()
        ));
    }

    let mut com = c_compiler_command(&config)?;
    com.arg("-Wno-unused-command-line-argument");
    if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
        com.arg("-shared");
    } else {
        com.arg("-no-pie");
    }
    if env::consts::OS == "macos" {
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
    run_c_compiler(&mut com, "link the output file")?;

    Ok(())
}
