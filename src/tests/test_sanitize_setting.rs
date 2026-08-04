//! The `sanitize` field of the project file: who it is read from, and what it refuses.
//!
//! Instrumenting a program is a property of the program that is built, so the project being built
//! decides it, as it does the optimization level. A library that asks for it in its own project file
//! leaves it off for anything that depends on it -- what makes that so is where the field is read in
//! `ProjectFile::set_config`, which nothing else would notice moving.

use crate::misc::{function_name, platform_thread_sanitizer_supported};
use crate::tests::test_util::{copy_dir_recursive, emitted_llvm_ir, fix_command, EmittedIr};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// Copies the case projects into a temporary directory and returns it with the path of `project`
/// inside it. The projects are copied together because they refer to each other by relative path.
fn setup_test_env(project: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_sanitize_setting/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join(project);
    (temp_dir, project_dir)
}

/// Runs `fix` with `args` in `project_dir`.
fn run_fix(project_dir: &Path, args: &[&str]) -> Output {
    fix_command()
        .args(args)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute fix")
}

/// Asserts that `output` succeeded, quoting both streams otherwise.
fn assert_succeeded(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{}\nstdout: {}\nstderr: {}",
        what,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Asserts that `output` failed, quoting both streams otherwise.
fn assert_failed(output: &Output, what: &str) {
    assert!(
        !output.status.success(),
        "{}\nstdout: {}\nstderr: {}",
        what,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Skips the caller unless this platform can build an instrumented program, saying so.
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

/// Whether the LLVM IR a build emitted into `project_dir` calls the sanitizer runtime.
fn is_instrumented(project_dir: &Path) -> bool {
    emitted_llvm_ir(project_dir, EmittedIr::All).contains("__tsan_")
}

#[test]
fn test_dependency_does_not_sanitize_the_project_being_built() {
    // A library that asks for instrumentation would otherwise impose it on every program that
    // depends on it, and a program several times slower than it was built to be is not what
    // depending on a library should mean.
    let (_temp_dir, project_dir) = setup_test_env("root_with_sanitizing_dep");
    let output = run_fix(&project_dir, &["build", "-O", "none", "--emit-llvm"]);
    assert_succeeded(&output, "the build should succeed.");
    assert!(
        !is_instrumented(&project_dir),
        "the dependency's `sanitize` should leave the program being built as it is built for use."
    );
}

#[test]
fn test_project_file_sanitizes_the_project_being_built() {
    // The other half of the pair above: the field does reach the build when the project being built
    // is the one carrying it, so the test above measures where the field is read rather than
    // whether it works at all.
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    let (_temp_dir, project_dir) = setup_test_env("root_sanitizes_itself");
    let output = run_fix(&project_dir, &["build", "-O", "none", "--emit-llvm"]);
    assert_succeeded(&output, "the build should succeed.");
    assert!(
        is_instrumented(&project_dir),
        "the project's own `sanitize` should instrument the program it builds."
    );
}

#[test]
fn test_unknown_sanitizer_is_reported() {
    // The command line offers its sanitizers as a fixed set, so a name outside it reaches the
    // compiler only through the project file.
    let (_temp_dir, project_dir) = setup_test_env("root_unknown_sanitizer");
    let output = run_fix(&project_dir, &["build"]);

    assert_failed(
        &output,
        "a sanitizer the compiler does not have should fail the build.",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown sanitizer") && stderr.contains("\"thread\""),
        "the failure should name the sanitizers there are.\nstderr: {}",
        stderr
    );
}

#[test]
fn test_sanitizer_and_valgrind_together_are_refused() {
    // An instrumented program brings its own runtime, and valgrind gives the program a machine of
    // its own; run together the program dies at startup with a message that names neither setting.
    // The project template offers both fields, so a project can ask for both.
    if !skip_unless_thread_sanitizer_available(function_name!()) {
        return;
    }
    let (_temp_dir, project_dir) = setup_test_env("root_sanitize_and_memcheck");
    let output = run_fix(&project_dir, &["test"]);

    assert_failed(&output, "asking for both should fail.");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot also be run under"),
        "the failure should say that the two settings are asking for different things.\
         \nstderr: {}",
        stderr
    );
}
