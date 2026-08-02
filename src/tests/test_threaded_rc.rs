//! The memory orderings the reference counting of a multi-threaded object is built from.
//!
//! These orderings are what let a data race detector check the threaded path: ThreadSanitizer draws
//! no happens-before edge from a standalone `fence`, so a reference count whose acquire lives in a
//! fence is reported as racing even where it is correct. The orderings are read off the emitted
//! LLVM IR, since that is where the choice is made and where a change to it is visible.

use crate::tests::test_util::{emitted_llvm_ir, fix_build_source_command, EmittedIr};
use std::process::Command;
use tempfile::TempDir;

/// A program that writes through a multi-threaded value while it holds the only handle, and again
/// once a second handle exists, so that the build emits the retain, the release and both answers of
/// the uniqueness check of a multi-threaded object.
const SOURCE: &str = r#"
module Main;

main : IO ();
main = (
    let value = Box::make(21).mark_threaded;
    let value = value.set_value(value.@value + 1);
    let shared = value.mark_threaded;
    let value = shared.set_value(shared.@value * 2);
    println((value.@value + shared.@value).to_string)
);
"#;

/// What `SOURCE` prints: 21 becomes 22 through the handle that owns it alone, then 44 through a
/// handle that shares it, leaving the other handle at 22.
const EXPECTED_OUTPUT: &str = "66";

/// Builds `SOURCE` with multi-threading on at `opt_level`, runs it, and returns the LLVM IR of the
/// modules the build emitted.
///
/// Running the program is what covers the reference-counting paths as control flow rather than as
/// text: the orderings below are read off the same build that produced the answer.
///
/// Each call builds in a directory of its own, which is what makes the returned IR the work of this
/// build alone.
fn emit_and_run_threaded(opt_level: &str) -> String {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let build = fix_build_source_command(dir, SOURCE, opt_level)
        .arg("--emit-llvm")
        .arg("--threaded")
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );

    let run = Command::new(dir.join("a.out"))
        .current_dir(dir)
        .output()
        .expect("Failed to execute the built program");
    assert!(
        run.status.success(),
        "the program should run to completion at -O {}.\nstdout: {}\nstderr: {}",
        opt_level,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        EXPECTED_OUTPUT,
        "the program should answer the same with multi-threaded reference counting at -O {}",
        opt_level
    );

    emitted_llvm_ir(dir, EmittedIr::All)
}

/// Asserts that `ir` accesses a multi-threaded reference count with `instruction`, and that every
/// such access carries `ordering`. `build_description` names the build the IR came from, and is
/// what a failure reports.
fn assert_every_access_carries_ordering(
    ir: &str,
    instruction: &str,
    ordering: &str,
    build_description: &str,
) {
    let accesses = ir
        .lines()
        .filter(|line| line.contains(instruction))
        .collect::<Vec<_>>();
    assert!(
        !accesses.is_empty(),
        "the build should emit `{}` on a multi-threaded reference count {}",
        instruction,
        build_description
    );
    let ordering_token = format!(" {}", ordering);
    let without_ordering = accesses
        .iter()
        .filter(|line| !line.contains(&ordering_token))
        .collect::<Vec<_>>();
    assert!(
        without_ordering.is_empty(),
        "every `{}` on a multi-threaded reference count should be `{}` {}, but these are not: {:?}",
        instruction,
        ordering,
        build_description,
        without_ordering
    );
}

/// Verifies the orderings the threaded reference counting is built from: the decrement acquires as
/// well as releases, the uniqueness check acquires in its load, the increment stays relaxed, and
/// none of them leaves its ordering to a fence.
///
/// One build serves them all, since they are faces of one emitted artifact.
fn assert_orderings_are_checkable(opt_level: &str) {
    let ir = emit_and_run_threaded(opt_level);
    let build_description = format!("at -O {}", opt_level);

    // The thread that brings the count to zero has to see every write the other holders made, so
    // the decrement carries the acquire.
    assert_every_access_carries_ordering(&ir, "atomicrmw sub", "acq_rel", &build_description);

    // A thread that finds itself the only holder goes on to write through the value, and those
    // writes come after the reads the releasing threads did.
    assert_every_access_carries_ordering(&ir, "load atomic", "acquire", &build_description);

    // An increment hands nothing over and reads nothing another thread wrote, so it stays relaxed.
    // Strengthening it would cost every retain of a shared value and buy nothing.
    assert_every_access_carries_ordering(&ir, "atomicrmw add", "monotonic", &build_description);

    // A standalone fence is invisible to ThreadSanitizer, so an acquire moved into one would turn
    // the threaded path into a stream of false reports.
    let fences = ir
        .lines()
        .filter(|line| line.trim_start().starts_with("fence "))
        .collect::<Vec<_>>();
    assert!(
        fences.is_empty(),
        "synchronization should be carried by the accesses themselves, so that a race detector can \
         follow it, but the build {} emitted: {:?}",
        build_description,
        fences
    );
}

#[test]
fn test_threaded_orderings_are_checkable_unoptimized() {
    assert_orderings_are_checkable("none");
}

#[test]
fn test_threaded_orderings_are_checkable_optimized() {
    // The optimizer rewrites the reference-counting paths, so the orderings are read again on what
    // it leaves behind.
    assert_orderings_are_checkable("max");
}
