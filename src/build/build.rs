use crate::build::build_object_files::build_object_files;
use crate::configuration::{Configuration, LinkType, OutputFileType, Sanitizer};
use crate::constants::INTERMEDIATE_PATH;
use crate::elaboration::elaborate_via_config;
use crate::error::Errors;
use crate::misc::info_msg;
use rand::{thread_rng, Rng};
use std::env::consts::OS;
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
    // nothing sets, which is reported as an error.
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

/// A file the build writes beside the sources, which a source includes by `path`.
struct IncludedFile {
    /// Where the file is written, relative to the directory the sources are compiled in, which is
    /// the path it carries in the compiler's tree.
    path: &'static str,
    /// The file's text.
    text: &'static str,
}

/// The files of the libraries under `src/fixstd/` that the runtime carries — Ryu in `ryu/` and
/// fast_float in `ffc/` — written beside the sources. `float_text.c` includes the sources of both,
/// so that it compiles every function they define as `static`.
///
/// Every file those directories hold is carried, whatever one configuration of the libraries
/// reaches: `d2s.c` and `f2s_intrinsics.h` choose between a tabulated and a computed table by a
/// macro, so which headers a build reads depends on the macros it is given.
/// `test_vendored_files_are_all_carried` holds this list to the directories.
const RUNTIME_INCLUDED_FILES: [IncludedFile; 13] = [
    IncludedFile {
        path: "ryu/ryu.h",
        text: include_str!("../fixstd/ryu/ryu.h"),
    },
    IncludedFile {
        path: "ryu/common.h",
        text: include_str!("../fixstd/ryu/common.h"),
    },
    IncludedFile {
        path: "ryu/digit_table.h",
        text: include_str!("../fixstd/ryu/digit_table.h"),
    },
    IncludedFile {
        path: "ryu/d2s_intrinsics.h",
        text: include_str!("../fixstd/ryu/d2s_intrinsics.h"),
    },
    IncludedFile {
        path: "ryu/d2s_full_table.h",
        text: include_str!("../fixstd/ryu/d2s_full_table.h"),
    },
    IncludedFile {
        path: "ryu/d2s_small_table.h",
        text: include_str!("../fixstd/ryu/d2s_small_table.h"),
    },
    IncludedFile {
        path: "ryu/f2s_intrinsics.h",
        text: include_str!("../fixstd/ryu/f2s_intrinsics.h"),
    },
    IncludedFile {
        path: "ryu/f2s_full_table.h",
        text: include_str!("../fixstd/ryu/f2s_full_table.h"),
    },
    IncludedFile {
        path: "ryu/d2fixed_full_table.h",
        text: include_str!("../fixstd/ryu/d2fixed_full_table.h"),
    },
    IncludedFile {
        path: "ryu/d2s.c",
        text: include_str!("../fixstd/ryu/d2s.c"),
    },
    IncludedFile {
        path: "ryu/f2s.c",
        text: include_str!("../fixstd/ryu/f2s.c"),
    },
    IncludedFile {
        path: "ryu/d2fixed.c",
        text: include_str!("../fixstd/ryu/d2fixed.c"),
    },
    IncludedFile {
        path: "ffc/ffc.h",
        text: include_str!("../fixstd/ffc/ffc.h"),
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
}

/// The C sources the runtime is built from.
const RUNTIME_SOURCES: [RuntimeSource; 2] = [
    RuntimeSource {
        object_name: "runtime",
        path: "runtime.c",
        text: include_str!("../fixstd/runtime.c"),
    },
    RuntimeSource {
        object_name: "float-text",
        path: "float_text.c",
        text: include_str!("../fixstd/float_text.c"),
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

    let build_dir = PathBuf::from(INTERMEDIATE_PATH)
        .join(format!("runtime.{}", thread_rng().gen::<u64>().to_string()));
    let write_file = |path: &str, text: &str| {
        let path = build_dir.join(path);
        fs::create_dir_all(path.parent().unwrap())
            .expect("Failed to create the directory the runtime is built in.");
        fs::write(&path, text).expect(&format!(
            "Failed to generate \"{}\"",
            path.to_string_lossy().to_string()
        ));
    };
    for included in RUNTIME_INCLUDED_FILES {
        write_file(included.path, included.text);
    }
    for source in &RUNTIME_SOURCES {
        write_file(source.path, source.text);
    }

    for (source, object) in RUNTIME_SOURCES.iter().zip(objects.iter()) {
        // The compiler runs inside the build directory, so it is given the plain name of the
        // object it writes.
        let compiled_name = format!("{}.o", source.object_name);
        let mut com = c_compiler_command(&config)?;
        // A source reaches the headers beside it by the path it includes them under, which is the
        // one it carries in the compiler's tree.
        com.current_dir(&build_dir).arg("-I.");
        // Writing a number as text is arithmetic, and unoptimized arithmetic costs several times
        // what optimized arithmetic costs: Ryu takes 214 ns to write a floating point number
        // unoptimized against 88 ns optimized, and one integer takes 188 instructions against 97.
        // The whole runtime compiles in a few milliseconds, and a build reuses the objects a
        // previous build of the same compiler wrote.
        com.arg("-O2")
            .arg("-ffunction-sections")
            .arg("-fdata-sections");
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
        if OS != "macos" {
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
    if OS == "macos" {
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
    use super::{RUNTIME_INCLUDED_FILES, RUNTIME_SOURCES};
    use crate::misc::Set;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    /// The directories under `src/fixstd/` that hold a library the runtime carries.
    const VENDORED_DIRS: [&str; 2] = ["ryu", "ffc"];

    /// The C files — headers and sources — in the directories of `VENDORED_DIRS`, each named by its
    /// path under `src/fixstd/`, such as `ryu/d2s.c`.
    fn vendored_c_files() -> Set<String> {
        let mut files = Set::default();
        for dir in VENDORED_DIRS {
            let vendored_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/fixstd")
                .join(dir);
            for entry in fs::read_dir(&vendored_dir)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", vendored_dir.display(), e))
            {
                let name = entry
                    .expect("failed to read a directory entry")
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                if name.ends_with(".h") || name.ends_with(".c") {
                    files.insert(format!("{}/{}", dir, name));
                }
            }
        }
        files
    }

    /// Every C file of the vendored libraries is carried into the directory a build compiles the
    /// runtime in. Taking a newer version of one is a matter of replacing its directory's files,
    /// and a file it gained that nothing carried would leave the C compiler with nothing to
    /// include — at the user's build rather than at ours.
    #[test]
    fn test_vendored_files_are_all_carried() {
        let carried: Set<String> = RUNTIME_INCLUDED_FILES
            .iter()
            .map(|file| file.path.to_string())
            .collect();
        assert_eq!(
            vendored_c_files(),
            carried,
            "the C files of the vendored libraries and the ones RUNTIME_INCLUDED_FILES carries"
        );
    }

    /// Every source of the vendored libraries is included by `float_text.c`, which is how it is
    /// compiled into the runtime. A source a newer version gained that nothing included would be
    /// missing from the link.
    #[test]
    fn test_vendored_sources_are_all_included() {
        let float_text = RUNTIME_SOURCES
            .iter()
            .find(|source| source.path == "float_text.c")
            .expect("float_text.c is one of the runtime's sources")
            .text;
        for source in vendored_c_files()
            .iter()
            .filter(|path| path.ends_with(".c"))
        {
            assert!(
                float_text.contains(&format!("#include \"{}\"", source)),
                "float_text.c does not include {}",
                source
            );
        }
    }

    /// Every name the runtime's objects define for the linker begins with `fixruntime_`, a prefix
    /// `FFI_EXPORT` rejects. So the runtime's names never meet a name a program or a library it
    /// links defines: the libraries the runtime carries define all their functions as `static`,
    /// and a program may carry the same libraries on its own.
    #[test]
    fn test_runtime_defines_only_fixruntime_names() {
        let build_dir = tempfile::tempdir().expect("failed to create a temporary directory");
        for file in RUNTIME_INCLUDED_FILES.iter() {
            let path = build_dir.path().join(file.path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, file.text).unwrap();
        }
        for source in RUNTIME_SOURCES.iter() {
            fs::write(build_dir.path().join(source.path), source.text).unwrap();
            let object = format!("{}.o", source.object_name);
            let output = Command::new("gcc")
                .current_dir(build_dir.path())
                .args(["-I.", "-O2", "-c", "-o", &object, source.path])
                .output()
                .expect("failed to run gcc");
            assert!(
                output.status.success(),
                "gcc failed on {}: {}",
                source.path,
                String::from_utf8_lossy(&output.stderr)
            );
            let output = Command::new("nm")
                .current_dir(build_dir.path())
                .args(["-g", &object])
                .output()
                .expect("failed to run nm");
            assert!(output.status.success(), "nm failed on {}", object);
            // Each line of `nm -g` is an address, a letter for the kind of the symbol and its name;
            // an undefined symbol has no address and the letter `U`. Mach-O puts `_` before a C name.
            let defined: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    match fields.as_slice() {
                        [_, kind, name] if *kind != "U" => Some(name.to_string()),
                        _ => None,
                    }
                })
                .collect();
            let foreign: Vec<&String> = defined
                .iter()
                .filter(|name| {
                    let name = if cfg!(target_os = "macos") {
                        name.strip_prefix('_').unwrap_or(name)
                    } else {
                        name
                    };
                    !name.starts_with("fixruntime_")
                })
                .collect();
            assert!(!defined.is_empty(), "nm found no name {} defines", object);
            assert!(
                foreign.is_empty(),
                "{} defines names without the `fixruntime_` prefix: {:?}",
                object,
                foreign
            );
        }
    }
}
