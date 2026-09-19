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
    let compiler = com.get_program().to_string_lossy().to_string();
    let output = com.output().map_err(|e| {
        Errors::from_msg(format!(
            "Failed to {}: could not run `{}`: {}.",
            step, compiler, e
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
            compiler,
            output.status.code().unwrap_or(-1)
        )));
    }
    Ok(())
}

/// A header the build writes beside the sources, which include it by `path`.
struct RuntimeHeader {
    /// Where the header is written, relative to the directory the sources are compiled in, which
    /// is the path it carries in the compiler's tree.
    path: &'static str,
    /// The header's text.
    text: &'static str,
}

/// The headers of `src/fixstd/ryu/`, written beside the sources that include them.
///
/// Every header the directory holds is carried, whatever one configuration of Ryu reaches: `d2s.c`
/// and `f2s_intrinsics.h` choose between a tabulated and a computed table by a macro, so which
/// headers a build reads depends on the macros it is given.
/// `test_vendored_ryu_headers_are_all_carried` holds this list to the directory.
const RUNTIME_HEADERS: [RuntimeHeader; 8] = [
    RuntimeHeader {
        path: "ryu/ryu.h",
        text: include_str!("../fixstd/ryu/ryu.h"),
    },
    RuntimeHeader {
        path: "ryu/common.h",
        text: include_str!("../fixstd/ryu/common.h"),
    },
    RuntimeHeader {
        path: "ryu/digit_table.h",
        text: include_str!("../fixstd/ryu/digit_table.h"),
    },
    RuntimeHeader {
        path: "ryu/d2s_intrinsics.h",
        text: include_str!("../fixstd/ryu/d2s_intrinsics.h"),
    },
    RuntimeHeader {
        path: "ryu/d2s_full_table.h",
        text: include_str!("../fixstd/ryu/d2s_full_table.h"),
    },
    RuntimeHeader {
        path: "ryu/d2s_small_table.h",
        text: include_str!("../fixstd/ryu/d2s_small_table.h"),
    },
    RuntimeHeader {
        path: "ryu/f2s_intrinsics.h",
        text: include_str!("../fixstd/ryu/f2s_intrinsics.h"),
    },
    RuntimeHeader {
        path: "ryu/f2s_full_table.h",
        text: include_str!("../fixstd/ryu/f2s_full_table.h"),
    },
];

/// One of the C sources the runtime is built from.
struct RuntimeSource {
    /// Names the object this source compiles to, both in the build directory and in the cache.
    object_name: &'static str,
    /// The path this source is written to and compiled at, which is the one it carries in the
    /// compiler's tree.
    path: &'static str,
    /// The text of the source, carried in the compiler.
    text: &'static str,
    /// The flags this source alone is compiled with.
    flags: &'static [&'static str],
}

/// The C sources the runtime is built from.
///
/// `ryu/d2s.c` and `ryu/f2s.c` each define a `to_chars` of their own, so each is a translation unit
/// of its own.
///
/// They are all compiled with optimization. Writing a number as text is arithmetic, and
/// unoptimized arithmetic costs several times what optimized arithmetic costs: Ryu takes 214 ns to
/// write a floating point number unoptimized against 88 ns optimized, and one integer takes 188
/// instructions against 97. The whole runtime compiles in a few milliseconds, and a build reuses
/// the objects a previous build of the same compiler wrote.
const RUNTIME_SOURCES: [RuntimeSource; 4] = [
    RuntimeSource {
        object_name: "runtime",
        path: "runtime.c",
        text: include_str!("../fixstd/runtime.c"),
        flags: &["-O2"],
    },
    RuntimeSource {
        object_name: "float-text",
        path: "float_text.c",
        text: include_str!("../fixstd/float_text.c"),
        flags: &["-O2"],
    },
    RuntimeSource {
        object_name: "ryu-d2s",
        path: "ryu/d2s.c",
        text: include_str!("../fixstd/ryu/d2s.c"),
        flags: &["-O2"],
    },
    RuntimeSource {
        object_name: "ryu-f2s",
        path: "ryu/f2s.c",
        text: include_str!("../fixstd/ryu/f2s.c"),
        flags: &["-O2"],
    },
];

/// Removes the directory a runtime build wrote its copies of the sources into.
fn remove_build_dir(build_dir: &Path) {
    fs::remove_dir_all(build_dir).expect(&format!(
        "Failed to remove \"{}\"",
        build_dir.to_string_lossy().to_string()
    ));
}

/// Builds the runtime into object files and answers where they are, reusing the objects a previous
/// build compiled.
///
/// An object is named by the hash of the settings it is compiled under, so a setting this
/// compilation reads belongs in `Configuration::runtime_object_hash`.
///
/// The sources are written into a directory of this build's own, so that builds running side by
/// side neither read nor overwrite each other's copies, and are compiled from inside it under the
/// paths they carry in the compiler's tree. What a compiled object records as the file it came
/// from is therefore that path, the same in every build.
fn build_runtime_objects(config: &Configuration) -> Result<Vec<PathBuf>, Errors> {
    let hash = config.runtime_object_hash();
    let objects: Vec<PathBuf> = RUNTIME_SOURCES
        .iter()
        .map(|source| {
            PathBuf::from(INTERMEDIATE_PATH)
                .join(format!("fixruntime.{}.{}.o", source.object_name, hash))
        })
        .collect();
    if objects.iter().all(|object| object.exists()) {
        return Ok(objects);
    }

    let build_dir = PathBuf::from(INTERMEDIATE_PATH).join(format!(
        "runtime.{}",
        rand::thread_rng().gen::<u64>().to_string()
    ));
    let write_file = |path: &str, text: &str| {
        let path = build_dir.join(path);
        fs::create_dir_all(path.parent().unwrap())
            .expect("Failed to create the directory the runtime is built in.");
        fs::write(&path, text).expect(&format!(
            "Failed to generate \"{}\"",
            path.to_string_lossy().to_string()
        ));
    };
    for header in RUNTIME_HEADERS {
        write_file(header.path, header.text);
    }
    for source in &RUNTIME_SOURCES {
        write_file(source.path, source.text);
    }

    for (source, object) in RUNTIME_SOURCES.iter().zip(objects.iter()) {
        // The compiler runs inside the build directory, so it is given the name of the object it
        // writes rather than a path reaching that directory.
        let compiled_name = format!("{}.o", source.object_name);
        let mut com = c_compiler_command(&config)?;
        // A source reaches the headers beside it by the path it includes them under, which is the
        // one it carries in the compiler's tree.
        com.current_dir(&build_dir).arg("-I.");
        com.args(source.flags);
        com.arg("-ffunction-sections").arg("-fdata-sections");
        // Keep frame pointers for better backtraces on macOS when backtrace is enabled
        if config.no_elim_frame_pointers() {
            com.arg("-fno-omit-frame-pointer");
        }
        com.arg("-o").arg(&compiled_name).arg("-c").arg(source.path);
        for m in &config.runtime_c_macro {
            com.arg(format!("-D{}", m));
        }
        if matches!(config.output_file_type, OutputFileType::DynamicLibrary) {
            com.arg("-fPIC");
        }
        if let Err(errors) =
            run_c_compiler(&mut com, &format!("compile the runtime's {}", source.path))
        {
            // The sources are this build's copies, which a failed compilation is as done with as
            // a finished one. A directory that resists removal is left where it is, since the
            // compilation's own failure is what the build reports.
            let _ = fs::remove_dir_all(&build_dir);
            return Err(errors);
        }

        let compiled_path = build_dir.join(&compiled_name);
        fs::rename(&compiled_path, object).expect(&format!(
            "Failed to rename \"{}\" to \"{}\"",
            compiled_path.to_string_lossy().to_string(),
            object.to_string_lossy().to_string()
        ));
    }
    remove_build_dir(&build_dir);

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
    let mut has_warned_on_mac = false;
    for (lib_name, link_type) in &config.linked_libraries {
        if env::consts::OS != "macos" {
            match link_type {
                LinkType::Static => libs_opts.push("-Wl,-Bstatic".to_string()),
                LinkType::Dynamic => libs_opts.push("-Wl,-Bdynamic".to_string()),
            }
        } else {
            if !has_warned_on_mac {
                info_msg("On MacOS, it is not possible to specify whether a library should be dynamically or statically linked. \
                If a dynamic library and a static library with the same name exist, the unintended one may be used.");
                has_warned_on_mac = true;
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

#[cfg(test)]
mod tests {
    use super::{RUNTIME_HEADERS, RUNTIME_SOURCES};
    use crate::misc::Set;
    use std::fs;
    use std::path::Path;

    /// The names of the files in `src/fixstd/ryu/` whose name ends in `extension`.
    fn vendored_ryu_files(extension: &str) -> Set<String> {
        let vendored_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixstd/ryu");
        fs::read_dir(&vendored_dir)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", vendored_dir.display(), e))
            .map(|entry| {
                entry
                    .expect("failed to read a directory entry")
                    .file_name()
                    .to_string_lossy()
                    .to_string()
            })
            .filter(|name| name.ends_with(extension))
            .collect()
    }

    /// Every header of `src/fixstd/ryu/` is carried into the directory a build compiles the runtime
    /// in. Taking a newer Ryu is a matter of replacing that directory's files, and a header it
    /// gained that nothing carried would leave the C compiler with nothing to include — at the
    /// user's build rather than at ours.
    #[test]
    fn test_vendored_ryu_headers_are_all_carried() {
        let carried: Set<String> = RUNTIME_HEADERS
            .iter()
            .map(|header| header.path.trim_start_matches("ryu/").to_string())
            .collect();
        assert_eq!(
            vendored_ryu_files(".h"),
            carried,
            "the headers of src/fixstd/ryu/ and the ones RUNTIME_HEADERS carries"
        );
    }

    /// Every source of `src/fixstd/ryu/` is compiled. The same reasoning as the headers': a source
    /// the directory gained and nothing compiled would be missing from the link.
    #[test]
    fn test_vendored_ryu_sources_are_all_compiled() {
        let compiled: Set<String> = RUNTIME_SOURCES
            .iter()
            .filter(|source| source.path.starts_with("ryu/"))
            .map(|source| source.path.trim_start_matches("ryu/").to_string())
            .collect();
        assert_eq!(
            vendored_ryu_files(".c"),
            compiled,
            "the sources of src/fixstd/ryu/ and the ones RUNTIME_SOURCES compiles"
        );
    }
}
