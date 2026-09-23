//! Whether a build stops on a signed integer overflow, on a shift by an amount outside the width
//! of its type, or on a conversion of a floating-point value to an integer type that does not hold
//! it, is a setting of that build. A test build takes the `build.test` section's value, and falls
//! back to the `build` section when `build.test` does not set it.
//!
//! Each case under `test_check_integer_operations_setting/cases` sums past the greatest value of
//! `I64`. Under the check the program stops there; without it the sum wraps, so the program prints
//! it and exits. A run that completes therefore shows the build was made without the check.

use crate::tests::test_util::{assert_failed_with, assert_succeeded, run_fix, setup_case_projects};

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_check_integer_operations_setting/cases";

/// What the checks say when they stop the program at the sum the case projects ask for.
/// The setting covers the shift amount and the float-to-integer conversion as well, which
/// `test_shift_amount` and `test_float_to_integer_conversion` exercise.
const SUM_STOPPED: &str = "Signed integer overflow: I64 addition";

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_turns_the_check_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build");
    assert_failed_with(
        &run_fix(&project_dir, &["run"]),
        SUM_STOPPED,
        "`fix run` should stop at the sum, because the build section turns the check on.",
    );
}

/// A test build falls back to the `build` section when the `build.test` section does not set the
/// value, so a project that asks for the check runs its tests under it.
#[test]
fn test_build_section_turns_the_check_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build");
    assert_failed_with(
        &run_fix(&project_dir, &["test"]),
        SUM_STOPPED,
        "`fix test` should stop at the sum, because the build section turns the check on and the \
         test section does not set a value of its own.",
    );
}

/// The `build.test` section decides the setting for a test build, and `false` counts as setting
/// it: a project that asks for the check can still run its tests without it.
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
    assert_failed_with(
        &run_fix(&project_dir, &["run"]),
        SUM_STOPPED,
        "`fix run` should stop at the sum, because the test section covers the test build alone.",
    );
}

/// The `build.test` section decides the setting for a test build when the `build` section does
/// not set it, so a project can run its tests under the check while its program runs on without
/// it.
#[test]
fn test_test_section_turns_the_check_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_test");
    assert_failed_with(
        &run_fix(&project_dir, &["test"]),
        SUM_STOPPED,
        "`fix test` should stop at the sum, because the test section turns the check on.",
    );
}

/// `--check-integer-operations` turns the check on for a test build whose project file turns it off.
///
/// The two runs share a project directory, so the second one meets the object files the first one
/// cached. The setting therefore has to be part of what identifies them.
#[test]
fn test_option_turns_the_check_on_for_a_test() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build_off_in_test");
    assert_succeeded(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should succeed, because the test section turns the check off.",
    );
    assert_failed_with(
        &run_fix(&project_dir, &["test", "--check-integer-operations"]),
        SUM_STOPPED,
        "`--check-integer-operations` should turn the check on, over the value the test section names.",
    );
}
