//! The run-time checks are a setting of the build being made, and a test build reads it from the
//! `build.test` section of the project file alone.
//!
//! Each case under `test_no_runtime_check_setting/cases` asks for an array whose element buffer
//! would overflow the address space. The run-time check rejects that capacity and aborts; with the
//! checks off the capacity is recorded as given, nothing writes to the buffer, and the program
//! prints the capacity and exits. So "the check was removed" is observable as a completed run.

use crate::tests::test_util::{assert_failed_with, assert_succeeded, run_fix, setup_case_projects};

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_no_runtime_check_setting/cases";

/// What the run-time check says when it rejects the capacity the case projects ask for.
const CAPACITY_REJECTED: &str = "Array size or capacity exceeds the address space";

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_disables_the_checks_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_off_in_build");
    assert_succeeded(
        &run_fix(&project_dir, &["run"]),
        "`fix run` should succeed, because the build section turns the checks off.",
    );
}

/// The `build` section's setting stays out of a test build, so a project that turns the checks off
/// for its program still runs its tests with them.
#[test]
fn test_build_section_leaves_the_checks_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_off_in_build");
    assert_failed_with(
        &run_fix(&project_dir, &["test"]),
        CAPACITY_REJECTED,
        "`fix test` should abort, because the test build keeps the checks.",
    );
}

/// The `build.test` section is what turns the checks off for a test build.
#[test]
fn test_test_section_disables_the_checks_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_off_in_test");
    assert_succeeded(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should succeed, because the test section turns the checks off.",
    );
}

/// The `build.test` section covers a test build alone, leaving the program with its checks.
#[test]
fn test_test_section_leaves_the_checks_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_off_in_test");
    assert_failed_with(
        &run_fix(&project_dir, &["run"]),
        CAPACITY_REJECTED,
        "`fix run` should abort, because the test section does not reach the program.",
    );
}

/// `--no-runtime-check` turns the checks off for a test build whose project file keeps them.
///
/// The two runs share a project directory, so the second one meets the object files the first one
/// cached. The setting therefore has to be part of what identifies them.
#[test]
fn test_option_disables_the_checks_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_off_in_build");
    assert_failed_with(
        &run_fix(&project_dir, &["test"]),
        CAPACITY_REJECTED,
        "`fix test` should abort, because the test build keeps the checks.",
    );
    assert_succeeded(
        &run_fix(&project_dir, &["test", "--no-runtime-check"]),
        "`--no-runtime-check` should turn the checks off, whichever subcommand it is given to.",
    );
}
