//! Whether a release that turns out to be the last one destroys the value completely.
//!
//! Two threads can release one value at the same moment, and which of them holds the last reference
//! is decided as they run: a release that reads the count and finds the value shared can still be
//! the one that brings it to zero. So what a release does has to destroy the value completely
//! whichever answer the count gave.
//!
//! Each program here arranges exactly that interleaving and reports whether the value was left
//! whole: one shares an array whose storage a write copies, the other shares a
//! `Std::FFI::Destructor` that every thread lets go of at once.

use crate::tests::test_util::{copy_dir_recursive, fix_command_at_opt_level};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// What a program prints when the value it shared was destroyed completely. One that finds
/// otherwise prints what it found instead.
const EXPECTED_OUTPUT: &str = "ok";

/// How many elements the shared array holds. The write copies them into a storage of its own, and
/// that copy is the stretch the other thread's release has to land inside, so it has to take long
/// enough to be aimed at.
const ELEMENT_COUNT: &str = "3000000";

/// How long the thread that lets go of the array waits before doing so, in microseconds. Long
/// enough that the write finds the storage shared, short enough that the copy is still running.
const DROP_DELAY_US: &str = "1000";

/// The write the array program makes in place: it copies the elements into a storage of the
/// array's own capacity, and has to release the elements the storage it lets go of held.
const ARRAY_WRITE_IN_PLACE: &str = "set";

/// The write the array program makes that grows the array: it copies the elements into a storage
/// of a larger capacity, which is a separate piece of generated code and has to release the
/// elements the storage it lets go of held on its own.
const ARRAY_WRITE_GROW: &str = "reserve";

/// What the array program prints when its two threads did not overlap: the thread that lets go of
/// the value did so before the write began, so the write found the storage its own and made no
/// copy. The run says nothing about the release under test, so it is taken again.
const MISSED_WINDOW: &str = "missed:";

/// How many times a run that missed the window is taken again before the machine is called unable
/// to produce the overlap.
const OVERLAP_ATTEMPTS: usize = 5;

/// How many values the destructor program builds and destroys. Whether the threads of a round meet
/// inside the window is decided by the machine, so the program asks many times.
const DESTRUCTOR_ROUNDS: &str = "5000";

/// How many threads share each of those values. Every one of them holds a reference and drops it at
/// the same moment, so any of them can be the one that finds the count already down to the last.
const DESTRUCTOR_THREADS: &str = "8";

/// Copies the case named `case` into a temporary directory and returns it with the project's path
/// inside it.
fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_contended_release/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join(case);
    (temp_dir, project_dir)
}

/// Builds the project in `project_dir` at `opt_level`, runs it with `program_args`, and returns
/// what it produced.
fn run_case(project_dir: &Path, opt_level: &str, program_args: &[&str]) -> Output {
    fix_command_at_opt_level("run", opt_level)
        .arg("--allow-preliminary-commands")
        .arg("--")
        .args(program_args)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute fix run")
}

/// Runs the case named `case` at `opt_level` and fails unless it reports that the shared value was
/// destroyed completely.
fn assert_shared_value_is_destroyed_completely(case: &str, opt_level: &str, program_args: &[&str]) {
    let (_temp_dir, project_dir) = setup_test_env(case);
    let output = run_case(&project_dir, opt_level, program_args);
    assert_program_reports_a_complete_destruction(case, opt_level, &output);
}

/// Fails unless the run of `case` that produced `output` finished and reported that the shared
/// value was destroyed completely.
fn assert_program_reports_a_complete_destruction(case: &str, opt_level: &str, output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "the `{}` program should run to completion at -O {}.\nstdout: {}\nstderr: {}",
        case,
        opt_level,
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        stdout.contains(EXPECTED_OUTPUT),
        "the `{}` program reported that the value it shared was not destroyed completely at -O {}.\
         \nstdout: {}\nstderr: {}",
        case,
        opt_level,
        stdout,
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Runs `case` at `opt_level` with `program_args` until its two threads overlap, and fails unless
/// the run they overlapped in reports that the shared value was destroyed completely.
///
/// The copy the release under test belongs to is made only where the two threads overlap, and which
/// of them the machine runs first is not ours to decide, so a run that reports the overlap was
/// missed is taken again.
fn assert_an_overlapping_run_destroys_the_value_completely(
    case: &str,
    opt_level: &str,
    program_args: &[&str],
) {
    let (_temp_dir, project_dir) = setup_test_env(case);
    for _ in 0..OVERLAP_ATTEMPTS {
        let output = run_case(&project_dir, opt_level, program_args);
        if String::from_utf8_lossy(&output.stdout).contains(MISSED_WINDOW) {
            continue;
        }
        assert_program_reports_a_complete_destruction(case, opt_level, &output);
        return;
    }
    panic!(
        "the two threads of the `{}` program never overlapped in {} runs at -O {}, so the release \
         under test was never reached.",
        case, OVERLAP_ATTEMPTS, opt_level,
    );
}

/// Runs the array case at `opt_level`, with the array written by `write`.
///
/// A write to a shared array copies the elements into a storage of its own and lets go of the
/// shared storage. A `#ArrayStorage` carries no length, so nothing but the release that drops its
/// last reference can release the elements it holds -- and the other thread is what makes that
/// release the last one. The program keeps a reference to the object every slot holds and asks
/// afterwards whether anything else still holds it.
fn assert_cloning_a_shared_array_releases_its_elements(opt_level: &str, write: &str) {
    assert_an_overlapping_run_destroys_the_value_completely(
        "array_clone",
        opt_level,
        &[ELEMENT_COUNT, DROP_DELAY_US, write],
    );
}

/// Runs the destructor case at `opt_level`.
///
/// Every thread of a round holds one reference to a `Std::FFI::Destructor` and lets go of it at
/// the same moment, so more than one of them reads the count as shared. The destructor function
/// counts its own runs, and the program compares that count with the number of values it built.
fn assert_a_destructor_runs_for_every_value(opt_level: &str) {
    assert_shared_value_is_destroyed_completely(
        "destructor",
        opt_level,
        &[DESTRUCTOR_ROUNDS, DESTRUCTOR_THREADS],
    );
}

/// A `set` on an array another thread shares releases the elements of the storage it copied out
/// of, where the other thread's release left this one holding the last reference. `-O none` keeps
/// every uniqueness check and reference count the compiler emits.
#[test]
fn test_cloning_a_shared_array_releases_its_elements_unoptimized() {
    assert_cloning_a_shared_array_releases_its_elements("none", ARRAY_WRITE_IN_PLACE);
}

/// A `set` on an array another thread shares releases the elements of the storage it copied out
/// of, at `-O max`, where the optimizations that drop a uniqueness check or a reference count
/// reshape the release under test.
#[test]
fn test_cloning_a_shared_array_releases_its_elements_optimized() {
    assert_cloning_a_shared_array_releases_its_elements("max", ARRAY_WRITE_IN_PLACE);
}

/// A `reserve` on an array another thread shares releases the elements of the storage it copied
/// out of. The copy goes into a storage of a larger capacity, which is generated separately from
/// the copy a write in place makes. `-O none` keeps every uniqueness check and reference count the
/// compiler emits.
#[test]
fn test_growing_a_shared_array_releases_its_elements_unoptimized() {
    assert_cloning_a_shared_array_releases_its_elements("none", ARRAY_WRITE_GROW);
}

/// A `reserve` on an array another thread shares releases the elements of the storage it copied
/// out of, at `-O max`, where the optimizations that drop a uniqueness check or a reference count
/// reshape the release under test.
#[test]
fn test_growing_a_shared_array_releases_its_elements_optimized() {
    assert_cloning_a_shared_array_releases_its_elements("max", ARRAY_WRITE_GROW);
}

/// Runs the punched-array case at `opt_level`.
///
/// A plug into a shared punched array copies the elements into a storage of its own, leaving out
/// the slot whose element was moved out of the array, and lets go of the shared storage. The other
/// thread is what makes that release the last one, and what it releases is every element outside
/// the hole: the element in the hole belongs to what the punch handed back.
fn assert_plugging_a_shared_punched_array_releases_its_elements(opt_level: &str) {
    assert_an_overlapping_run_destroys_the_value_completely(
        "punched_clone",
        opt_level,
        &[ELEMENT_COUNT, DROP_DELAY_US],
    );
}

/// A plug into a punched array another thread shares releases the elements of the storage it
/// copied out of, apart from the slot whose element the punch handed back. `-O none` keeps every
/// uniqueness check and reference count the compiler emits.
#[test]
fn test_plugging_a_shared_punched_array_releases_its_elements_unoptimized() {
    assert_plugging_a_shared_punched_array_releases_its_elements("none");
}

/// A plug into a punched array another thread shares releases the elements of the storage it
/// copied out of, at `-O max`, where the optimizations that drop a uniqueness check or a reference
/// count reshape the release under test.
#[test]
fn test_plugging_a_shared_punched_array_releases_its_elements_optimized() {
    assert_plugging_a_shared_punched_array_releases_its_elements("max");
}

/// A `Std::FFI::Destructor` that many threads let go of at the same moment runs its destructor
/// function exactly once. `-O none` keeps every uniqueness check and reference count the compiler
/// emits.
#[test]
fn test_a_destructor_runs_for_every_value_unoptimized() {
    assert_a_destructor_runs_for_every_value("none");
}

/// A `Std::FFI::Destructor` that many threads let go of at the same moment runs its destructor
/// function exactly once, at `-O max`, where the optimizations that drop a uniqueness check or a
/// reference count reshape the release under test.
#[test]
fn test_a_destructor_runs_for_every_value_optimized() {
    assert_a_destructor_runs_for_every_value("max");
}
