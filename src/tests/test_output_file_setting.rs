//! What a build produces — the kind of the output file and where it is written — is what `fix
//! build` is asked for. `fix run` and `fix test` build an executable in a temporary place, run it,
//! and remove it, so the two settings of the project file leave them alone.
//!
//! The kind is also part of the code that is generated, so two builds that disagree on it do not
//! share object files.
//!
//! Each case project under `test_output_file_setting/cases` prints a line naming which program is
//! running, which is how these tests read off what a run built.

use crate::tests::test_util::{assert_succeeded, run_fix, setup_case_projects};
use std::path::Path;
use std::process::Command;

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_output_file_setting/cases";

/// What `main.fix` prints.
const PROGRAM_OUTPUT: &str = "I am the program";

/// What `test.fix` prints.
const TEST_OUTPUT: &str = "I am the test suite";

/// The output file the `named_output` project asks for.
const NAMED_OUTPUT: &str = "myprogram";

/// What a build calls a dynamic library when the project does not name the output file, following
/// `Configuration::get_output_file_path`.
fn dynamic_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "lib.dll"
    } else if cfg!(target_os = "macos") {
        "lib.dylib"
    } else {
        "lib.so"
    }
}

/// What a build calls an executable when the project does not name the output file, following
/// `Configuration::get_output_file_path`.
fn executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "a.exe"
    } else {
        "a.out"
    }
}

/// Runs the program at `path` and returns what it printed on its standard output.
fn run_program(path: &Path) -> String {
    let output = Command::new(path)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run \"{}\": {}", path.display(), e));
    assert_succeeded(
        &output,
        &format!("\"{}\" should run successfully.", path.display()),
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// `fix build` produces the kind of file the project file names. The kind decides what the output
/// file is called, so which file the build left behind says which kind it produced.
#[test]
fn test_build_produces_the_kind_the_project_file_names() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "dylib_program");
    assert_succeeded(
        &run_fix(&project_dir, &["build", "-O", "none"]),
        "`fix build` should succeed for a project asking for a dynamic library.",
    );
    assert!(
        project_dir.join(dynamic_library_name()).exists(),
        "`fix build` should produce \"{}\", which is the dynamic library the project file asks for.",
        dynamic_library_name(),
    );
    assert!(
        !project_dir.join(executable_name()).exists(),
        "`fix build` should leave \"{}\" alone, because the project file asks for a dynamic library.",
        executable_name(),
    );
}

/// `fix run` builds an executable and runs it, whatever kind of file the project file asks a build
/// to produce.
#[test]
fn test_run_builds_an_executable_for_a_project_asking_for_a_library() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "dylib_program");
    let output = run_fix(&project_dir, &["run", "-O", "none"]);
    assert_succeeded(
        &output,
        "`fix run` should run the program of a project asking a build for a dynamic library.",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(PROGRAM_OUTPUT),
        "`fix run` should print what the program prints.\nstdout: {}",
        stdout,
    );
}

/// `fix test` builds an executable and runs it, whatever kind of file the project file asks a build
/// to produce.
#[test]
fn test_test_builds_an_executable_for_a_project_asking_for_a_library() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "dylib_program");
    let output = run_fix(&project_dir, &["test", "-O", "none"]);
    assert_succeeded(
        &output,
        "`fix test` should run the test suite of a project asking a build for a dynamic library.",
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(TEST_OUTPUT),
        "`fix test` should print what the test suite prints.\nstdout: {}",
        stdout,
    );
}

/// A test run leaves the output file the project names as `fix build` wrote it: the executable a
/// test run makes is a temporary one, and the path in the project file describes what `fix build`
/// produces.
#[test]
fn test_a_test_run_leaves_the_output_file_of_a_build() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "named_output");
    assert_succeeded(
        &run_fix(&project_dir, &["build", "-O", "none"]),
        "`fix build` should succeed for a project naming its output file.",
    );
    let output_file = project_dir.join(NAMED_OUTPUT);
    assert!(
        run_program(&output_file).contains(PROGRAM_OUTPUT),
        "`fix build` should write the program to \"{}\", which the project file names.",
        NAMED_OUTPUT,
    );
    assert_succeeded(
        &run_fix(&project_dir, &["test", "-O", "none"]),
        "`fix test` should succeed for a project naming its output file.",
    );
    assert!(
        run_program(&output_file).contains(PROGRAM_OUTPUT),
        "`fix test` should leave \"{}\" as the program `fix build` wrote there.",
        NAMED_OUTPUT,
    );
}

/// A build of a dynamic library rejects the object files of a build of a program: the two ask for
/// different code, so the objects of one cannot be linked into the other.
#[test]
fn test_a_library_build_does_not_reuse_the_objects_of_a_program_build() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "plain_program");
    assert_succeeded(
        &run_fix(&project_dir, &["build", "-O", "none"]),
        "`fix build` should produce the program.",
    );
    assert_succeeded(
        &run_fix(
            &project_dir,
            &["build", "-O", "none", "--output-type", "dylib"],
        ),
        "`fix build --output-type dylib` should produce a dynamic library beside the program.",
    );
}

/// A build of a program rejects the object files of a build of a dynamic library, which is the
/// same rule seen from the other side.
#[test]
fn test_a_program_build_does_not_reuse_the_objects_of_a_library_build() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "plain_program");
    assert_succeeded(
        &run_fix(
            &project_dir,
            &["build", "-O", "none", "--output-type", "dylib"],
        ),
        "`fix build --output-type dylib` should produce a dynamic library.",
    );
    assert_succeeded(
        &run_fix(&project_dir, &["build", "-O", "none"]),
        "`fix build` should produce the program beside the dynamic library.",
    );
}
