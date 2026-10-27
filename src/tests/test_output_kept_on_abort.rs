//! What a program wrote before a runtime check stopped it reaches its destination.
//!
//! The tests run the program with its standard output captured into a pipe, where the C library
//! buffers it in blocks, so a printed line stays in the buffer until something flushes it.

use crate::{
    misc::function_name,
    tests::test_util::{
        assert_failed_with, run_source_capture, runtime_checked_config, source_with_a_runtime_zero,
        working_file_path,
    },
};
use std::fs;

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
    assert_failed_with(
        &output,
        "Index out of range: index=10, size=3",
        "the index check should stop the program",
    );
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
    assert_failed_with(
        &output,
        "undefined reached",
        "`undefined` should stop the program",
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "printed before undefined\n"
    );
}

/// What the program wrote to a file it opened and never closed is in the file after a check stops
/// the program.
#[test]
pub fn test_an_open_file_keeps_what_was_written_when_a_check_stops_the_program() {
    let file_path = working_file_path(function_name!(), "txt");
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
    assert_failed_with(
        &output,
        "Index out of range",
        "the index check should stop the program",
    );
    assert_eq!(
        fs::read_to_string(&file_path).unwrap(),
        "written before the check"
    );
    fs::remove_file(&file_path).unwrap();
}
