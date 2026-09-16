//! Where a build stops at a signed overflow is a setting of the build being made, and a test build
//! takes the `build` section's value where the `build.test` section names none.
//!
//! Each case under `test_check_signed_overflow_setting/cases` sums past the greatest value of
//! `I64`. The check stops the program there; without it the sum is taken modulo two to the width of
//! the type, so the program prints it and exits. So "the check was left out" is observable as a
//! completed run.

use crate::tests::test_util::{assert_succeeded, run_fix, setup_case_projects};
use std::process::Output;

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_check_signed_overflow_setting/cases";

/// What the check says when it stops the program at the sum the case projects ask for.
const SUM_STOPPED: &str = "Signed integer overflow: I64 addition";

/// Asserts that `output` failed with the check's report.
fn assert_stopped_by_the_check(output: &Output, what: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.contains(SUM_STOPPED),
        "{}\nstdout: {}\nstderr: {}",
        what,
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
}

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_turns_the_check_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build");
    assert_stopped_by_the_check(
        &run_fix(&project_dir, &["run"]),
        "`fix run` should stop at the sum, because the build section turns the check on.",
    );
}

/// A test build takes the `build` section's value where the `build.test` section names none, so a
/// project that asks for the check runs its tests under it.
#[test]
fn test_build_section_turns_the_check_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build");
    assert_stopped_by_the_check(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should stop at the sum, because the build section turns the check on and the \
         test section names no value of its own.",
    );
}

/// The `build.test` section decides the setting for a test build, and `false` is a value it names:
/// a project that asks for the check can still run its tests without it.
#[test]
fn test_test_section_turns_the_check_off_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build_off_in_test");
    assert_succeeded(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should succeed, because the test section turns the check off.",
    );
}

/// The `build.test` section covers the test build alone, so turning the check off there leaves the
/// program stopping at the sum.
#[test]
fn test_test_section_leaves_the_check_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build_off_in_test");
    assert_stopped_by_the_check(
        &run_fix(&project_dir, &["run"]),
        "`fix run` should stop at the sum, because the test section covers the test build alone.",
    );
}
