//! A dynamic library `fix build` produces, seen from the C program that uses it.
//!
//! The library offers the functions `FFI_EXPORT` names. Beside them stand the symbols that carry a
//! value from one compilation unit to another — a build below `-O max` splits its code into several
//! — and every one of them enters the library's symbol table, under the spelling
//! `object_symbol_name` gives a Fix name.
//!
//! A C program reaches the library either by opening it at run time or by naming it on its own link
//! line, and the case projects here cover one each.

use crate::configuration::OutputFileType;
use crate::tests::test_util::{assert_succeeded, fix_command, setup_case_projects};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_dynamic_library/cases";

/// The argument the driver of `exported_getter` calls the exported function with.
const DRIVER_ARGUMENT: i64 = 10;

/// What the exported function of `exported_getter` answers to `DRIVER_ARGUMENT`: the two fields of
/// the point it builds are the argument and its successor.
const DRIVER_OUTPUT: &str = "21";

/// Builds `project_dir` as a dynamic library at `opt_level` and returns the path of the library.
///
/// The suite runs under an optimization level taken from `FIX_MAX_OPT_LEVEL`, which caps the level
/// `-O` asks for, so the variable is pinned here to the level under test.
fn build_library(project_dir: &Path, opt_level: &str) -> PathBuf {
    let output = fix_command()
        .args(["build", "-O", opt_level])
        .env("FIX_MAX_OPT_LEVEL", opt_level)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute `fix build`");
    assert_succeeded(
        &output,
        &format!("`fix build -O {}` should build the library.", opt_level),
    );
    project_dir.join(OutputFileType::DynamicLibrary.default_file_name())
}

/// Builds the driver of `project_dir` with `link_arguments` on its link line, and returns the path
/// of the program.
fn build_driver(project_dir: &Path, link_arguments: &[&str]) -> PathBuf {
    let driver = project_dir.join("driver");
    let output = Command::new("gcc")
        .args(["-o", driver.to_str().unwrap()])
        .arg(project_dir.join("driver.c"))
        .args(link_arguments)
        .output()
        .expect("Failed to execute gcc");
    assert_succeeded(&output, "gcc should build the driver.");
    driver
}

/// Builds the `exported_getter` library at `opt_level`, opens it from the driver, and asserts that
/// the function it exports answers what the Fix source says it does.
fn assert_library_answers(opt_level: &str) {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "exported_getter");
    let library = build_library(&project_dir, opt_level);
    // Where the loader is a library of its own rather than a part of the C library.
    let link_arguments: &[&str] = if cfg!(target_os = "linux") {
        &["-ldl"]
    } else {
        &[]
    };
    let driver = build_driver(&project_dir, link_arguments);

    let output = Command::new(&driver)
        .arg(&library)
        .arg(DRIVER_ARGUMENT.to_string())
        .output()
        .expect("Failed to execute the driver");
    assert_succeeded(
        &output,
        &format!(
            "the driver should load \"{}\" and call into it.",
            library.display()
        ),
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        DRIVER_OUTPUT,
        "the exported function should answer {} to {}.",
        DRIVER_OUTPUT,
        DRIVER_ARGUMENT
    );
}

/// A library built without optimization, where the code is split into several compilation units and
/// the symbols carrying values between them enter the library's symbol table.
#[test]
fn test_a_library_built_at_none_is_loaded_and_called() {
    assert_library_answers("none");
}

/// The same at the level that optimizes while still splitting the code into compilation units.
#[test]
fn test_a_library_built_at_basic_is_loaded_and_called() {
    assert_library_answers("basic");
}

/// The same at the level that compiles the whole program as one unit, where the values it carries
/// need no symbol of their own.
#[test]
fn test_a_library_built_at_max_is_loaded_and_called() {
    assert_library_answers("max");
}

/// A C program built against the library calls the function it exports and gets the answer the Fix
/// source gives, which the program reports through its exit status. The library is on the driver's
/// link line, so the exported name has to be in the library's symbol table at link time as well.
#[test]
fn test_a_library_on_a_link_line_is_called() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "linked_program");
    let library = build_library(&project_dir, "max");
    let library_directory = library.parent().unwrap().to_str().unwrap().to_string();
    let driver = build_driver(
        &project_dir,
        &[
            library.to_str().unwrap(),
            &format!("-Wl,-rpath,{}", library_directory),
        ],
    );

    let output = Command::new(&driver)
        .output()
        .expect("Failed to execute the driver");
    assert_succeeded(
        &output,
        "the driver should get the answer of the exported function.",
    );
}
