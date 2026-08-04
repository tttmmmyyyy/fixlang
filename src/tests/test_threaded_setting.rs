//! Multi-threading is a setting of the project being built, and a dependency asking for it in its
//! own project file leaves it off.
//!
//! The cases under `test_threaded_setting/cases` are a library that calls `Std::mark_threaded` and
//! sets `threaded = true`, together with the root projects that depend on it.

use crate::tests::test_util::{assert_succeeded, run_fix, setup_case_projects};

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_threaded_setting/cases";

/// The lines of a diagnostic that quote source, which is where the reported call sites appear.
fn quoted_source(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| line.contains('|'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A dependency's `threaded = true` leaves multi-threading off, so the calls of
/// `Std::mark_threaded` the program reaches fail the build, each quoted at its own call site.
#[test]
fn test_dependency_does_not_enable_threaded() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_without_threaded");
    let output = run_fix(&project_dir, &["build"]);

    assert!(
        !output.status.success(),
        "the build should fail, because the dependency's `Std::mark_threaded` has no \
         multi-threading to work with.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`Std::mark_threaded` requires multi-threading"),
        "the failure should say what is missing.\nstderr: {}",
        stderr
    );

    // Every call is reported, each quoted at the call site alone, and the calls in the library are
    // what name the library that needs the setting. The program makes three: two through the
    // library at two different types, so that `Std::mark_threaded` is instantiated twice, and one
    // directly.
    let quoted = quoted_source(&stderr);
    assert_eq!(
        quoted.matches("mark_threaded").count(),
        3,
        "all three calls should be reported.\nquoted: {}",
        quoted
    );
    assert!(
        !quoted.contains("contents"),
        "a call should be quoted on its own, leaving the rest of the definition out.\nquoted: {}",
        quoted
    );
    assert!(
        stderr.contains("lib.fix") && stderr.contains("main.fix"),
        "the calls in the library and in the root project should both be reported.\nstderr: {}",
        stderr
    );
}

/// The root project's own `threaded = true` covers the calls of `Std::mark_threaded` its
/// dependency makes.
#[test]
fn test_root_enables_threaded() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_with_threaded");
    let output = run_fix(&project_dir, &["build"]);
    assert_succeeded(
        &output,
        "the build should succeed, because the root project turns multi-threading on.",
    );
}

/// `--threaded` turns multi-threading on for a project whose project file leaves it off.
#[test]
fn test_threaded_option_enables_multi_threading() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_without_threaded");
    let output = run_fix(&project_dir, &["build", "--threaded"]);
    assert_succeeded(
        &output,
        "`--threaded` should enable multi-threading, as the diagnostic says it does.",
    );
}

/// A program that reaches no call of `Std::mark_threaded` builds with multi-threading off, even
/// though it depends on a library that calls it.
///
/// A program is built from the definitions it reaches, so depending on a library that calls
/// `Std::mark_threaded` somewhere costs nothing until the program reaches such a definition. This
/// is what lets one library serve programs that want multi-threading and programs that do not.
#[test]
fn test_unreached_library_call_needs_no_threading() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_reaches_no_call");
    let output = run_fix(&project_dir, &["build"]);
    assert_succeeded(
        &output,
        "the build should succeed, because the program reaches no call of `Std::mark_threaded`.",
    );
}

/// `fix test` builds the test program, so the `[build.test]` section of the root project decides
/// the setting for it.
///
/// The case calls `Std::mark_threaded` from `Test::test` alone, so `fix build` succeeds with the
/// `[build]` section leaving multi-threading off, and `fix test` succeeds with `[build.test]`
/// turning it on.
#[test]
fn test_test_section_enables_threaded() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_threaded_in_test");
    assert_succeeded(
        &run_fix(&project_dir, &["build"]),
        "`fix build` should succeed, because `main` reaches no call of `Std::mark_threaded`.",
    );
    assert_succeeded(
        &run_fix(&project_dir, &["test"]),
        "`fix test` should succeed, because the test section turns multi-threading on.",
    );
}
