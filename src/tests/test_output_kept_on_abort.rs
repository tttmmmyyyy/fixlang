//! What a program wrote before a runtime check stopped it reaches its destination.
//!
//! The tests run the program with its standard output captured into a pipe, where the C library
//! buffers it in blocks, so a printed line stays in the buffer until something flushes it.

use crate::{
    constants::COMPILER_TEST_WORKING_PATH,
    misc::function_name,
    tests::test_util::{
        assert_failed_with, build_program, run_source_capture, runtime_checked_config,
        source_with_a_runtime_zero,
    },
};
use std::{
    fs,
    io::{Read, Write},
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{Command, Stdio},
};

/// The signal `abort` stops a program with.
const SIGABRT: i32 = 6;

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

/// A line printed to standard output before a signed overflow check stops the program is kept.
#[test]
pub fn test_stdout_is_kept_when_an_overflow_check_stops_the_program() {
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before the check");;
        eval I64::maximum + (zero + 1);
        println("not reached");;
        "#,
    );
    let mut config = runtime_checked_config();
    config.check_integer_operations = true;
    let output = run_source_capture(&source, config);
    assert_failed_with(
        &output,
        "Signed integer overflow",
        "the overflow check should stop the program",
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "printed before the check\n"
    );
}

/// What the program wrote to a file it opened and never closed is in the file after a check stops
/// the program.
#[test]
pub fn test_an_open_file_keeps_what_was_written_when_a_check_stops_the_program() {
    fs::create_dir_all(COMPILER_TEST_WORKING_PATH).unwrap();
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

/// Where standard output and standard error go to one pipe, the report of a check that stops the
/// program comes after the lines the program printed before it.
#[test]
pub fn test_the_report_follows_what_was_printed_before_the_check() {
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before the check");;
        let arr = Array::fill(3, 0);
        eval arr.@(zero + 10);
        "#,
    );
    let (_dir, program) = build_program(&source, "basic", &[], None, "an index check");
    assert_aborts_writing_to_one_pipe(
        &program,
        "printed before the check\nIndex out of range: index=10, size=3\n",
    );
}

/// A program whose standard output goes to a pipe its reader has closed is still stopped by
/// `abort` when a check fails, and still writes the report, although flushing what it printed
/// fails.
#[test]
pub fn test_a_check_reports_and_aborts_when_stdout_is_a_closed_pipe() {
    // The program prints a line, which stays in its buffer, and waits for a line on standard input
    // before the check, so that the reader of its standard output is gone by the time it flushes.
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before the check");;
        let _ = *read_line(stdin).to_result;
        let arr = Array::fill(3, 0);
        eval arr.@(zero + 10);
        "#,
    );
    let (_dir, program) = build_program(&source, "basic", &[], None, "an index check");
    let mut child = Command::new(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    child.stdin.take().unwrap().write_all(b"go\n").unwrap();
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    let status = child.wait().unwrap();
    assert_eq!(status.signal(), Some(SIGABRT), "stderr: {}", stderr);
    assert!(
        stderr.contains("Index out of range: index=10, size=3"),
        "stderr: {}",
        stderr
    );
}

/// Where standard output and standard error go to one pipe, the message of `undefined` comes after
/// the lines the program printed before it.
#[test]
pub fn test_the_message_of_undefined_follows_what_was_printed_before_it() {
    let source = source_with_a_runtime_zero(
        r#"
        println("printed before undefined");;
        eval if zero == 0 { undefined("undefined reached") } else { 0 };
        "#,
    );
    let (_dir, program) = build_program(&source, "basic", &[], None, "undefined");
    assert_aborts_writing_to_one_pipe(&program, "printed before undefined\nundefined reached\n");
}

/// Run `program` with its standard output and standard error going to one pipe, and assert that
/// `abort` stops it and that the pipe receives `expected`.
fn assert_aborts_writing_to_one_pipe(program: &Path, expected: &str) {
    let output = Command::new("sh")
        .args(["-c", "exec \"$0\" 2>&1"])
        .arg(program)
        .output()
        .unwrap();
    assert_eq!(output.status.signal(), Some(SIGABRT));
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
}

/// `undefined` writes its message as it is, a `%` in it included.
#[test]
pub fn test_undefined_writes_a_message_holding_a_percent_sign_as_it_is() {
    let source = source_with_a_runtime_zero(
        r#"
        eval if zero == 0 { undefined("100%% done") } else { 0 };
        "#,
    );
    let output = run_source_capture(&source, runtime_checked_config());
    assert_failed_with(
        &output,
        "100%% done",
        "`undefined` should stop the program with its message",
    );
}
