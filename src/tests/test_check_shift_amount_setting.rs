//! Whether a build stops at a shift amount outside the width of its type is a setting of that
//! build. A test build takes the `build.test` section's value, and the `build` section's value
//! where `build.test` names none.
//!
//! The case project under `test_check_shift_amount_setting/cases` shifts an `I64` by 64. Under the
//! check the program stops there; without it the amount is taken modulo the width, so the shift
//! leaves the value where it is and the program prints it and exits. A run that completes
//! therefore shows the build was made without the check.

use crate::tests::test_util::{assert_failed_with, assert_succeeded, run_fix, setup_case_projects};

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_check_shift_amount_setting/cases";

/// What the check says when it stops the program at the shift the case project asks for.
const SHIFT_STOPPED: &str = "Shift amount outside the width of the type: I64 shift_left";

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_turns_the_check_on_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_check_on_in_build_off_in_test");
    assert_failed_with(
        &run_fix(&project_dir, &["run"]),
        SHIFT_STOPPED,
        "`fix run` should stop at the shift, because the build section turns the check on.",
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

/// `--check-shift-amount` turns the check on for a test build whose project file turns it off.
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
        &run_fix(&project_dir, &["test", "--check-shift-amount"]),
        SHIFT_STOPPED,
        "`--check-shift-amount` should turn the check on, over the value the test section names.",
    );
}
