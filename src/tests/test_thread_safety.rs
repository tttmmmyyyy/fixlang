//! Whether a multi-threaded program shares values between threads safely, checked by
//! ThreadSanitizer.
//!
//! `Std::FFI::pack_for_thread` is what puts a value's reference count into the mode that updates it
//! atomically, and the promise is that a program which hands its values to other threads through it
//! has no data race in its reference counting. The `hammer` program shares one value with eight
//! threads, and ThreadSanitizer reports whether the promise held. Its control hands the value over
//! through `Std::FFI::boxed_to_retained_ptr` instead, and ThreadSanitizer has to report a race
//! there: a check that cannot fail says nothing about the runs where it passes.
//!
//! A global value is computed on its first read, and the promise is that the threads reading it see
//! it computed once and read it whole. The `first_read` program reads one global value from two
//! groups of eight threads: one group arrives while the initializer is running and waits for it, and
//! the other reads the value after the initializer has finished, with nothing but the global's
//! accessor ordering its read after the initializing thread's writes.

use crate::misc::{function_name, platform_thread_sanitizer_supported};
use crate::tests::test_util::{copy_dir_recursive, fix_command};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// The argument that asks the `hammer` program to hand the value over the way the language says
/// to.
const MARK_ARGUMENT: &str = "mark";

/// What the `hammer` program prints once the threads have finished with the value.
const EXPECTED_OUTPUT: &str = "ok";

/// The project that shares one value with eight threads.
const HAMMER_PROJECT: &str = "hammer";

/// The project that reads one global value from threads arriving during and after its
/// initialization.
const FIRST_READ_PROJECT: &str = "first_read";

/// Copies the harness projects into a temporary directory and returns it with the path of the
/// project `project` inside it.
fn setup_test_env(project: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_thread_safety/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join(project);
    (temp_dir, project_dir)
}

/// Builds the project at `project_dir` at `opt_level` under the sanitizer named by `sanitizer`,
/// runs it with `program_args`, and returns what it produced, through `fix run`.
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
        .arg(sanitizer);
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

/// Builds the program in the case `project` at `opt_level` and asks ThreadSanitizer about both of its
/// runs: with `MARK_ARGUMENT` the program hands its value to the threads through
/// `Std::FFI::pack_for_thread`, and without it through `Std::FFI::boxed_to_retained_ptr`.
fn assert_shared_reference_counting_is_race_free(project: &str, opt_level: &str) {
    let (_temp_dir, project_dir) = setup_test_env(project);

    // The program the language asks for: every thread reaches the value through
    // `Std::FFI::pack_for_thread`, so the counts are updated atomically and nothing races.
    let marked = run_harness(&project_dir, opt_level, "thread", &[MARK_ARGUMENT]);
    assert_eq!(
        races_reported(&marked),
        0,
        "counting a value handed over through `Std::FFI::pack_for_thread` should race nowhere at \
         -O {}.\
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

    // The same program handing the value over through `Std::FFI::boxed_to_retained_ptr`, which leaves
    // it in single-threaded mode. A race reported here is what gives the run above its weight.
    let unmarked = run_harness(&project_dir, opt_level, "thread", &[]);
    assert!(
        races_reported(&unmarked) > 0,
        "sharing a value that was never handed over through `Std::FFI::pack_for_thread` should be \
         reported at -O {}, and nothing was.\nstdout: {}\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&unmarked.stdout),
        String::from_utf8_lossy(&unmarked.stderr),
    );
}

/// Verifies that a value handed to eight threads through `Std::FFI::pack_for_thread` is counted
/// without a race in a build with the optimizations off.
#[test]
fn test_shared_reference_counting_is_race_free_unoptimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free(HAMMER_PROJECT, "none");
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
    assert_shared_reference_counting_is_race_free(HAMMER_PROJECT, "max");
}

/// The project that shares one box holding an unboxed pair with eight threads.
const HAMMER_UNBOXED_PROJECT: &str = "hammer_unboxed";

/// Verifies that one box holding an unboxed value, handed to eight threads through
/// `Std::FFI::pack_for_thread` and `Std::FFI::get_funptr_retain_for_thread`, is counted without a
/// race in a build with the optimizations off.
#[test]
fn test_shared_unboxed_value_is_race_free_unoptimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free(HAMMER_UNBOXED_PROJECT, "none");
}

/// Verifies that the same sharing stays race-free at `-O max`.
#[test]
fn test_shared_unboxed_value_is_race_free_optimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free(HAMMER_UNBOXED_PROJECT, "max");
}

/// The project that shares a pair of a closure capturing an array and an optional box with eight
/// threads.
const HAMMER_CLOSURE_PROJECT: &str = "hammer_closure";

/// Verifies that the values a closure captures and the payload of an unboxed union, handed to eight
/// threads inside one value through `Std::FFI::pack_for_thread`, are counted without a race in a
/// build with the optimizations off.
#[test]
fn test_shared_closure_and_union_are_race_free_unoptimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free(HAMMER_CLOSURE_PROJECT, "none");
}

/// Verifies that the same sharing stays race-free at `-O max`.
#[test]
fn test_shared_closure_and_union_are_race_free_optimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_shared_reference_counting_is_race_free(HAMMER_CLOSURE_PROJECT, "max");
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
    let (_temp_dir, project_dir) = setup_test_env(HAMMER_PROJECT);

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

/// Builds the `first_read` program at `opt_level` under the sanitizer named by `sanitizer`, runs it,
/// and asserts that it ran to completion with no race reported. The program itself checks that the
/// initializer ran once and that every thread read the whole value.
fn assert_first_read_from_threads_is_safe(opt_level: &str, sanitizer: &str) {
    let (_temp_dir, project_dir) = setup_test_env(FIRST_READ_PROJECT);
    let output = run_harness(&project_dir, opt_level, sanitizer, &[]);
    assert_eq!(
        races_reported(&output),
        0,
        "reading a global value from threads during and after its initialization should race \
         nowhere at -O {}.\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "the program should run to completion at -O {}.\nstdout: {}\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Verifies that the threads reading a global value during and after its initialization run its
/// initializer once and each read the whole value.
#[test]
fn test_a_global_read_from_threads_is_initialized_once() {
    assert_first_read_from_threads_is_safe("max", "none");
}

/// Verifies that the threads reading a global value during and after its initialization see what
/// the initializing thread stored without a race, in a build with the optimizations off.
#[test]
fn test_a_global_read_from_threads_is_race_free_unoptimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_first_read_from_threads_is_safe("none", "thread");
}

/// Verifies the same at `-O max`, where the accessor of the global is inlined into its readers.
#[test]
fn test_a_global_read_from_threads_is_race_free_optimized() {
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    assert_first_read_from_threads_is_safe("max", "thread");
}
