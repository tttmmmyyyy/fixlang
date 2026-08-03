//! Whether a value shared between threads is counted safely, checked by ThreadSanitizer.
//!
//! `Std::mark_threaded` is what puts a value's reference count into the mode that updates it
//! atomically, and the promise is that a program which uses it has no data race in its reference
//! counting. This runs a program that shares one value with eight threads and asks ThreadSanitizer
//! whether the promise held.
//!
//! The control is the same program run without the call. It has to be reported: a check that cannot
//! fail says nothing about the runs where it passes.

use crate::misc::{function_name, platform_thread_sanitizer_supported};
use crate::tests::test_util::{copy_dir_recursive, fix_command};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// The argument that asks the program to hand the value over the way the language says to.
const MARK_ARGUMENT: &str = "mark";

/// What the program prints once the threads have finished with the value.
const EXPECTED_OUTPUT: &str = "ok";

/// Copies the harness project into a temporary directory and returns it with the project's path
/// inside it.
fn setup_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_thread_safety/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join("hammer");
    (temp_dir, project_dir)
}

/// Builds the harness at `opt_level` under the sanitizer named by `sanitizer`, runs it with
/// `program_args`, and returns what it produced, through `fix run`.
///
/// Going through `fix run` puts the compiler's own way of starting an instrumented program under
/// test: the sanitizer maps its shadow memory to addresses derived from the program's own, so it
/// needs the address space laid out the same way on every run, and `fix run` is what arranges that.
fn run_harness(
    project_dir: &Path,
    opt_level: &str,
    sanitizer: &str,
    program_args: &[&str],
) -> Output {
    let mut command = fix_command();
    command
        .arg("run")
        .arg("-O")
        .arg(opt_level)
        .arg("--sanitize")
        .arg(sanitizer)
        .arg("--allow-preliminary-commands");
    if !program_args.is_empty() {
        command.arg("--").args(program_args);
    }
    command
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute fix run")
}

/// How many data races ThreadSanitizer reported in `output`.
fn races_reported(output: &Output) -> usize {
    String::from_utf8_lossy(&output.stderr)
        .matches("WARNING: ThreadSanitizer: data race")
        .count()
}

/// Whether an instrumented program can run on this platform, printing that `test_name` was skipped
/// when it cannot.
fn skip_unless_thread_sanitizer_available(test_name: &str) -> bool {
    if platform_thread_sanitizer_supported() {
        return true;
    }
    eprintln!(
        "Skipping {}: ThreadSanitizer is not available on this platform.",
        test_name
    );
    false
}

/// Builds the harness at `opt_level` and asks ThreadSanitizer about both of its runs.
fn assert_shared_reference_counting_is_race_free(opt_level: &str) {
    let (_temp_dir, project_dir) = setup_test_env();

    // The program the language asks for: every thread reaches the value through
    // `Std::mark_threaded`, so the counts are updated atomically and nothing races.
    let marked = run_harness(&project_dir, opt_level, "thread", &[MARK_ARGUMENT]);
    assert_eq!(
        races_reported(&marked),
        0,
        "counting a value handed over through `Std::mark_threaded` should race nowhere at -O {}.\
         \nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&marked.stderr)
    );
    assert!(
        marked.status.success()
            && String::from_utf8_lossy(&marked.stdout).contains(EXPECTED_OUTPUT),
        "the program should run to completion at -O {}.\nstdout: {}\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&marked.stdout),
        String::from_utf8_lossy(&marked.stderr),
    );

    // The same program without the call. Reporting this is what gives the run above its weight.
    let unmarked = run_harness(&project_dir, opt_level, "thread", &[]);
    assert!(
        races_reported(&unmarked) > 0,
        "sharing a value that was never handed over through `Std::mark_threaded` should be \
         reported at -O {}, and nothing was.\nstdout: {}\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&unmarked.stdout),
        String::from_utf8_lossy(&unmarked.stderr),
    );
}

/// Verifies that a value handed to eight threads through `Std::mark_threaded` is counted without a
/// race in a build with the optimizations off.
#[test]
fn test_shared_reference_counting_is_race_free_unoptimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free("none");
}

/// Verifies that the same sharing stays race-free at `-O max`, where the optimizations that elide a
/// reference count run -- the uniqueness checks specialization removes, the counting
/// borrow-ification cancels -- so a mistake in one of them shows up as a race the unoptimized build
/// never sees.
#[test]
fn test_shared_reference_counting_is_race_free_optimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free("max");
}

/// Verifies that a build asking for the instrumentation carries it even after the same program was
/// built without it. The objects a build leaves behind are reused by the next build of the same
/// program, so the instrumentation has to be part of what names them; otherwise the checked build
/// would reuse the uninstrumented objects and report a clean run.
#[test]
fn test_instrumentation_is_not_taken_from_an_uninstrumented_build() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    let (_temp_dir, project_dir) = setup_test_env();

    // Warmed by the same subcommand the checked run uses: what names an object includes which
    // subcommand asked for it, so a build and a run never share one to begin with.
    let plain = run_harness(&project_dir, "none", "none", &[MARK_ARGUMENT]);
    assert!(
        plain.status.success(),
        "the run without instrumentation should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&plain.stdout),
        String::from_utf8_lossy(&plain.stderr),
    );
    assert_eq!(
        races_reported(&plain),
        0,
        "a program built without the instrumentation reports nothing, whatever it does.\nstderr: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let checked = run_harness(&project_dir, "none", "thread", &[]);
    assert!(
        races_reported(&checked) > 0,
        "a build that asks for the instrumentation after one built without it should carry the \
         instrumentation, and this one reported nothing.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr),
    );
}
