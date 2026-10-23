//! The `objects` of a project file reach the linker as files to link. An entry that looks like an
//! option of the C compiler (`-wrapper`, which runs a program of the project's choosing) or like a
//! file of further options (`@file`) is still read as the path of a file, so a project file cannot
//! hand the linker options through `objects` without the approval `ld_flags` need.

use crate::tests::test_util::{assert_failed, run_fix};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;

/// Writes into `dir` a script `wrap.sh` that, given as `gcc`'s `-wrapper`, records that it ran by
/// creating `dir/wrapper-ran` and then runs the command `gcc` handed it. Returns the path of the
/// file it creates.
fn write_wrapper_script(dir: &Path) -> std::path::PathBuf {
    let marker = dir.join("wrapper-ran");
    let script = dir.join("wrap.sh");
    fs::write(
        &script,
        format!("#!/bin/sh\ntouch '{}'\nexec \"$@\"\n", marker.display()),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    marker
}

/// Writes into `dir` a project whose `build` section lists `objects` (a TOML array literal).
fn write_project(dir: &Path, objects: &str) {
    fs::write(
        dir.join("fixproj.toml"),
        format!(
            "[general]\nname = \"objects\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\nobjects = {}\n",
            objects
        ),
    )
    .unwrap();
    fs::write(
        dir.join("main.fix"),
        "module Main;\n\nmain : IO ();\nmain = println(\"ok\");\n",
    )
    .unwrap();
}

/// An entry of `objects` written as the option `-wrapper` is a file named `-wrapper`, which does
/// not exist: the build fails without running the program the entry after it names.
#[test]
fn test_an_object_named_like_an_option_is_not_an_option() {
    let dir = TempDir::new().unwrap();
    let marker = write_wrapper_script(dir.path());
    write_project(dir.path(), "[\"-wrapper\", \"./wrap.sh\"]");
    assert_failed(
        &run_fix(dir.path(), &["build", "-O", "none"]),
        "`fix build` should fail to link a file named `-wrapper`.",
    );
    assert!(
        !marker.exists(),
        "the object `-wrapper` was read as an option"
    );
}

/// An entry of `objects` starting with `@` is a file to link, not a file of further options.
#[test]
fn test_an_object_named_like_an_options_file_is_not_read_for_options() {
    let dir = TempDir::new().unwrap();
    let marker = write_wrapper_script(dir.path());
    // `gcc` reads `@opts` as the file `opts` of further options.
    fs::write(dir.path().join("opts"), "-wrapper ./wrap.sh\n").unwrap();
    write_project(dir.path(), "[\"@opts\"]");
    assert_failed(
        &run_fix(dir.path(), &["build", "-O", "none"]),
        "`fix build` should fail to link `@opts`, which is no file.",
    );
    assert!(!marker.exists(), "the object `@opts` was read for options");
}
