use crate::build::build_object_files::build_object_files;
use crate::configuration::{Configuration, LinkType, OutputFileType, Sanitizer};
use crate::constants::INTERMEDIATE_PATH;
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::misc::info_msg;
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
    // inkwell's `llvm22-1` feature. Raising one without the other leaves this looking for a prefix
    // nothing sets, so say so rather than reach for whatever clang the path happens to hold.
    let Some(prefix) = option_env!("LLVM_SYS_221_PREFIX") else {
        return Err(Errors::from_msg(
            "This compiler was built without recording where its LLVM lives, so the clang a \
             sanitized build needs cannot be found. Build it with `LLVM_SYS_221_PREFIX` set."
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
        // A process a signal ends carries no exit code, and `-1` stands for that case: the C
        // compiler crashed, or the system killed it under memory pressure.
        return Err(Errors::from_msg(format!(
            "Failed to {}: {} exited with code {}.",
            step,
            program,
            output.status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

/// The headers the runtime's sources include, each written beside the source that includes it.
const RUNTIME_HEADERS: [(&str, &str); 8] = [
    ("ryu/ryu.h", include_str!("../fixstd/ryu/ryu.h")),
    ("ryu/common.h", include_str!("../fixstd/ryu/common.h")),
    (
        "ryu/digit_table.h",
        include_str!("../fixstd/ryu/digit_table.h"),
    ),
    (
        "ryu/d2s_intrinsics.h",
        include_str!("../fixstd/ryu/d2s_intrinsics.h"),
    ),
    (
        "ryu/d2s_full_table.h",
        include_str!("../fixstd/ryu/d2s_full_table.h"),
    ),
    (
        "ryu/d2s_small_table.h",
        include_str!("../fixstd/ryu/d2s_small_table.h"),
    ),
    (
        "ryu/f2s_intrinsics.h",
        include_str!("../fixstd/ryu/f2s_intrinsics.h"),
    ),
    (
        "ryu/f2s_full_table.h",
        include_str!("../fixstd/ryu/f2s_full_table.h"),
    ),
];

/// The C sources the runtime is built from: the name the object each one compiles to is cached
/// under, the path the source is written to and compiled by, its text, and the flags that source
/// alone is compiled with.
///
/// `ryu/d2s.c` and `ryu/f2s.c` each define a `to_chars` of their own, so each is a translation unit
/// of its own. They are the part of the runtime that computes rather than calls out, and they take
/// 214 ns to write a number unoptimized against the 88 ns they take optimized, so they are the part
/// the C compiler is asked to optimize.
const RUNTIME_SOURCES: [(&str, &str, &str, &[&str]); 3] = [
    (
        "runtime",
        "runtime.c",
        include_str!("../fixstd/runtime.c"),
        &[],
    ),
    (
        "ryu-d2s",
        "ryu/d2s.c",
        include_str!("../fixstd/ryu/d2s.c"),
        &["-O2"],
    ),
    (
        "ryu-f2s",
        "ryu/f2s.c",
        include_str!("../fixstd/ryu/f2s.c"),
        &["-O2"],
    ),
];

/// Builds the runtime into object files and answers where they are, reusing the objects a previous
/// build compiled.
///
/// An object is named by the hash of the settings it is compiled under, so a setting the
/// compilation below reads belongs in `Configuration::runtime_object_hash`.
///
/// The sources are written into a directory of this build's own, so that builds running side by
/// side neither read nor overwrite each other's copies, and are compiled from inside it under the
/// paths they carry in the compiler's tree. What a compiled object records as the file it came
/// from is therefore `runtime.c`, the same in every build.
fn build_runtime_objects(config: &Configuration) -> Result<Vec<PathBuf>, Errors> {
    let hash = config.runtime_object_hash();
    let objects: Vec<PathBuf> = RUNTIME_SOURCES
        .iter()
        .map(|(name, _, _, _)| {
            PathBuf::from(INTERMEDIATE_PATH).join(format!("fixruntime.{}.{}.o", name, hash))
        })
        .collect();
    if objects.iter().all(|object| object.exists()) {
        return Ok(objects);
    }

    let build_dir = PathBuf::from(INTERMEDIATE_PATH).join(format!(
        "runtime.{}",
        rand::thread_rng().gen::<u64>().to_string()
    ));
    let write_source = |path: &str, text: &str| {
        let path = build_dir.join(path);
        fs::create_dir_all(path.parent().unwrap())
            .expect("Failed to create the directory the runtime is built in.");
        fs::write(&path, text).expect(&format!(
            "Failed to generate \"{}\"",
            path.to_string_lossy().to_string()
        ));
    };
    for (path, text) in RUNTIME_HEADERS {
        write_source(path, text);
    }
    for (_, path, text, _) in RUNTIME_SOURCES {
        write_source(path, text);
    }

    for ((name, source, _, flags), object) in RUNTIME_SOURCES.iter().zip(objects.iter()) {
        let compiled = format!("{}.o", name);
        let mut com = c_compiler_command(&config)?;
        // A source reaches the headers beside it by the path it includes them under, which is the
        // one it carries in the compiler's tree.
        com.current_dir(&build_dir).arg("-I.");
        com.args(*flags);
        com.arg("-ffunction-sections").arg("-fdata-sections");
        // Keep frame pointers for better backtraces on macOS when backtrace is enabled
        if config.no_elim_frame_pointers() {
            com.arg("-fno-omit-frame-pointer");
        }
        com.arg("-o").arg(&compiled).arg("-c").arg(source);
        for m in &config.runtime_c_macro {
            com.arg(format!("-D{}", m));
        }
        if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
            com.arg("-fPIC");
        }
        run_c_compiler(&mut com, "compile the runtime")?;

        fs::rename(build_dir.join(&compiled), object).expect(&format!(
            "Failed to rename \"{}\" to \"{}\"",
            build_dir.join(&compiled).to_string_lossy().to_string(),
            object.to_string_lossy().to_string()
        ));
    }
    fs::remove_dir_all(&build_dir).expect(&format!(
        "Failed to remove \"{}\"",
        build_dir.to_string_lossy().to_string()
    ));

    Ok(objects)
}

/// Builds the program specified in the configuration, linking the object files and the runtime into
/// the output file.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
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

    let runtime_obj_paths = build_runtime_objects(&config)?;

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
    for runtime_obj_path in &runtime_obj_paths {
        com.arg(runtime_obj_path.to_str().unwrap());
    }
    com.args(library_search_path_opts).args(libs_opts);
    run_c_compiler(&mut com, "link the output file")?;

    Ok(())
}
