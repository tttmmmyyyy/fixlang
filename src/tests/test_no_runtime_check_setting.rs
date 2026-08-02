//! The run-time checks are a setting of the build being made, and a test build reads it from the
//! `build.test` section of the project file alone.
//!
//! Each case under `test_no_runtime_check_setting/cases` asks for an array whose element buffer
//! would overflow the address space. The run-time check rejects that capacity and aborts; with the
//! checks off the capacity is recorded as given, nothing writes to the buffer, and the program
//! prints the capacity and exits. So "the check was removed" is observable as a completed run.

use crate::tests::test_util::{copy_dir_recursive, fix_command};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// What the run-time check says when it rejects the capacity the case projects ask for.
const CAPACITY_REJECTED: &str = "Array size or capacity exceeds the address space";

/// Copies the case projects into a temporary directory and returns it with the path of `project`
/// inside it.
fn setup_test_env(project: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_no_runtime_check_setting/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join(project);
    (temp_dir, project_dir)
}

/// Runs `fix` with `args` in `project_dir`.
fn run_fix(project_dir: &Path, args: &[&str]) -> Output {
    fix_command()
        .args(args)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute fix")
}

/// Asserts that `output` succeeded, quoting both streams otherwise.
fn assert_succeeded(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{}\nstdout: {}\nstderr: {}",
        what,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Asserts that `output` failed with the run-time check's diagnostic.
fn assert_rejected_by_the_check(output: &Output, what: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.contains(CAPACITY_REJECTED),
        "{}\nstdout: {}\nstderr: {}",
        what,
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
}

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_disables_the_checks_for_the_program() {
    let (_temp_dir, project_dir) = setup_test_env("root_check_off_in_build");
    assert_succeeded(
        &run_fix(&project_dir, &["run"]),
        "`fix run` should succeed, because the build section turns the checks off.",
    );
}

/// The `build` section's setting stays out of a test build, so a project that turns the checks off
/// for its program still runs its tests with them.
#[test]
fn test_build_section_leaves_the_checks_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_check_off_in_build");
    assert_rejected_by_the_check(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should abort, because the test build keeps the checks.",
    );
}

/// The `build.test` section is what turns the checks off for a test build.
#[test]
fn test_test_section_disables_the_checks_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_check_off_in_test");
    assert_succeeded(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should succeed, because the test section turns the checks off.",
    );
}

/// The `build.test` section covers a test build alone, leaving the program with its checks.
#[test]
fn test_test_section_leaves_the_checks_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_test_env("root_check_off_in_test");
    assert_rejected_by_the_check(
        &run_fix(&project_dir, &["run"]),
        "`fix run` should abort, because the test section does not reach the program.",
    );
}

/// `--no-runtime-check` turns the checks off for a test build whose project file keeps them.
///
/// The two runs share a project directory, so the second one meets the object files the first one
/// cached. The setting therefore has to be part of what identifies them.
#[test]
fn test_option_disables_the_checks_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_check_off_in_build");
    assert_rejected_by_the_check(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should abort, because the test build keeps the checks.",
    );
    assert_succeeded(
        &run_fix(&project_dir, &["test", "--no-runtime-check"]),
        "`--no-runtime-check` should turn the checks off, whichever subcommand it is given to.",
    );
}
