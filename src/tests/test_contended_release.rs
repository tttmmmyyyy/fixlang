//! Whether a release that turns out to be the last one destroys the value completely.
//!
//! Two threads can release one value at the same moment, and which of them holds the last reference
//! is decided as they run: a release that reads the count and finds the value shared can still be
//! the one that brings it to zero. So what a release does has to destroy the value completely
//! whichever answer the count gave.
//!
//! Each program here arranges exactly that interleaving and reports whether the value was left
//! whole: one shares an array whose storage a write copies, the other shares a `Std::Destructor`
//! that every thread lets go of at once.

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
const ARRAY_ELEMENT_COUNT: &str = "3000000";

/// How long, in microseconds, the thread that lets go of the array waits before doing so. Long
/// enough that the write finds the storage shared, short enough that the copy is still running.
const ARRAY_DROP_DELAY: &str = "1000";

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

/// Builds the case named `case` at `opt_level`, runs it with `program_args`, and returns what it
/// produced.
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

/// Runs the array case at `opt_level`.
///
/// A write to a shared array copies the elements into a storage of its own and lets go of the
/// shared storage. A `#ArrayStorage` carries no length, so nothing but the release that drops its
/// last reference can release the elements it holds -- and the other thread is what makes that
/// release the last one. The program keeps a reference to the object every slot holds and asks
/// afterwards whether anything else still holds it.
fn assert_cloning_a_shared_array_releases_its_elements(opt_level: &str) {
    assert_shared_value_is_destroyed_completely(
        "array_clone",
        opt_level,
        &[ARRAY_ELEMENT_COUNT, ARRAY_DROP_DELAY],
    );
}

/// Runs the destructor case at `opt_level`.
///
/// Every thread of a round holds one reference to a `Std::Destructor` and lets go of it at the same
/// moment, so more than one of them reads the count as shared. The destructor function counts its
/// own runs, and the program compares that count with the number of values it built.
fn assert_a_destructor_runs_for_every_value(opt_level: &str) {
    assert_shared_value_is_destroyed_completely(
        "destructor",
        opt_level,
        &[DESTRUCTOR_ROUNDS, DESTRUCTOR_THREADS],
    );
}

#[test]
fn test_cloning_a_shared_array_releases_its_elements_unoptimized() {
    assert_cloning_a_shared_array_releases_its_elements("none");
}

/// The same at `-O max`, where the optimizations that drop a uniqueness check or a reference count
/// run, so a release the compiler reshapes there is checked as well as the one it emits plainly.
#[test]
fn test_cloning_a_shared_array_releases_its_elements_optimized() {
    assert_cloning_a_shared_array_releases_its_elements("max");
}

#[test]
fn test_a_destructor_runs_for_every_value_unoptimized() {
    assert_a_destructor_runs_for_every_value("none");
}

/// The same at `-O max`, for the reason the array case is run there.
#[test]
fn test_a_destructor_runs_for_every_value_optimized() {
    assert_a_destructor_runs_for_every_value("max");
}
