//! What a program wrote before a runtime check stopped it reaches its destination.
//!
//! The tests run the program with its standard output captured into a pipe, where the C library
//! buffers it in blocks rather than lines, so a line stays in the buffer until something flushes
//! it.

use crate::{
    configuration::Configuration,
    constants::COMPILER_TEST_WORKING_PATH,
    misc::function_name,
    tests::test_util::{run_source_capture, source_with_a_runtime_zero},
};
use std::{fs, process::Output};

/// A configuration whose runtime checks are on, so that the checks stopping the programs below are
/// part of them.
fn runtime_checked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    config
}

/// Fails the test unless the program behind `output` was stopped rather than exiting with code 0,
/// and unless its standard error contains `report`.
fn assert_stopped_with(output: &Output, report: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "the program should have been stopped.\nstderr: {}",
        stderr
    );
    assert!(
        stderr.contains(report),
        "the report should contain {:?}.\nstderr: {}",
        report,
        stderr
    );
}

/// A line printed to standard output before an index check stops the program is kept.
#[test]
pub fn test_stdout_is_kept_when_an_index_check_stops_the_program() {
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before the check");;
        let arr = Array::fill(3, 0);
        eval arr.@(zero + 10);
        println("not reached");;
        "#,
    );
    let output = run_source_capture(&source, runtime_checked_config());
    assert_stopped_with(&output, "Index out of range: index=10, size=3");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "printed before the check\n"
    );
}

/// A line printed to standard output before `undefined` stops the program is kept.
#[test]
pub fn test_stdout_is_kept_when_undefined_stops_the_program() {
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before undefined");;
        eval if zero == 0 { undefined("undefined reached") } else { 0 };
        println("not reached");;
        "#,
    );
    let output = run_source_capture(&source, runtime_checked_config());
    assert_stopped_with(&output, "undefined reached");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "printed before undefined\n"
    );
}

/// What the program wrote to a file it opened and never closed is in the file after a check stops
/// the program.
#[test]
pub fn test_an_open_file_keeps_what_was_written_when_a_check_stops_the_program() {
    let _ = fs::create_dir_all(COMPILER_TEST_WORKING_PATH);
    let file_path = format!("{}/{}.txt", COMPILER_TEST_WORKING_PATH, function_name!());
    let _ = fs::remove_file(&file_path);
    let source = source_with_a_runtime_zero(&format!(
        r#"
        let handle = *open_file("{}", "w").try(exit_with_msg(1));
        write_string(handle, "written before the check").try(exit_with_msg(1));;
        let arr = Array::fill(3, 0);
        eval arr.@(zero + 10);
        "#,
        file_path
    ));
    let output = run_source_capture(&source, runtime_checked_config());
    assert_stopped_with(&output, "Index out of range");
    assert_eq!(
        fs::read_to_string(&file_path).unwrap(),
        "written before the check"
    );
    fs::remove_file(&file_path).unwrap();
}
