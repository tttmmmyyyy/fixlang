//! What a build produces — the kind of the output file and where it is written — is what `fix
//! build` is asked for. `fix run` and `fix test` build an executable in a temporary place, run it,
//! and remove it, so the two settings of the project file leave them alone.
//!
//! The kind is also part of the code that is generated, so two builds that disagree on it do not
//! share object files.
//!
//! Each case project under `test_output_file_setting/cases` prints a line naming which program is
//! running, which is how these tests read off what a run built.

use crate::configuration::OutputFileType;
use crate::tests::test_util::{assert_failed, assert_succeeded, run_fix, setup_case_projects};
use std::path::Path;
use std::process::Command;

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_output_file_setting/cases";

/// What `main.fix` prints.
const PROGRAM_OUTPUT: &str = "I am the program";

/// What `test.fix` prints.
const TEST_OUTPUT: &str = "I am the test suite";

/// The output file the `named_output` project asks for.
const NAMED_OUTPUT_FILE: &str = "myprogram";

/// The kind of output file the `unknown_output_type` project asks for, which the compiler does not
/// have.
const UNKNOWN_OUTPUT_TYPE: &str = "shared";

/// What a build calls a dynamic library when the project does not name the output file.
fn default_dynamic_library_name() -> &'static str {
    OutputFileType::DynamicLibrary.default_file_name()
}

/// What a build calls an executable when the project does not name the output file.
fn default_executable_name() -> &'static str {
    OutputFileType::Executable.default_file_name()
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
        project_dir.join(default_dynamic_library_name()).exists(),
        "`fix build` should produce \"{}\", which is the dynamic library the project file asks for.",
        default_dynamic_library_name(),
    );
    assert!(
        !project_dir.join(default_executable_name()).exists(),
        "`fix build` should leave \"{}\" alone, because the project file asks for a dynamic library.",
        default_executable_name(),
    );
}

/// The `--output-type` option decides what a build produces, whatever kind the project file names:
/// a compiler option overwrites the setting it stands against.
#[test]
fn test_the_output_type_option_overwrites_the_project_file() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "dylib_program");
    assert_succeeded(
        &run_fix(
            &project_dir,
            &["build", "-O", "none", "--output-type", "exe"],
        ),
        "`fix build --output-type exe` should succeed for a project asking for a dynamic library.",
    );
    assert!(
        run_program(&project_dir.join(default_executable_name())).contains(PROGRAM_OUTPUT),
        "`fix build --output-type exe` should produce \"{}\", the program the option asks for.",
        default_executable_name(),
    );
    assert!(
        !project_dir.join(default_dynamic_library_name()).exists(),
        "`fix build --output-type exe` should leave \"{}\" alone, because the option decides the kind.",
        default_dynamic_library_name(),
    );
}

/// A kind of output file the compiler does not have fails the command that read it, naming the kind
/// the project file asked for. A build, which produces the output file, and a run, which builds an
/// executable of its own, both report it.
#[test]
fn test_unknown_output_file_type_is_reported() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "unknown_output_type");
    for command in ["build", "run"] {
        let output = run_fix(&project_dir, &[command, "-O", "none"]);
        assert_failed(
            &output,
            &format!(
                "`fix {}` should fail for a project file naming a kind of output file the compiler \
                 does not have.",
                command
            ),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(UNKNOWN_OUTPUT_TYPE),
            "`fix {}` should name the kind the project file asked for.\nstderr: {}",
            command,
            stderr,
        );
    }
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
    let output_file = project_dir.join(NAMED_OUTPUT_FILE);
    assert!(
        run_program(&output_file).contains(PROGRAM_OUTPUT),
        "`fix build` should write the program to \"{}\", which the project file names.",
        NAMED_OUTPUT_FILE,
    );
    assert_succeeded(
        &run_fix(&project_dir, &["test", "-O", "none"]),
        "`fix test` should succeed for a project naming its output file.",
    );
    assert!(
        run_program(&output_file).contains(PROGRAM_OUTPUT),
        "`fix test` should leave \"{}\" as the program `fix build` wrote there.",
        NAMED_OUTPUT_FILE,
    );
}

/// A run writes no file where the project file names the output of a build: `fix run` and `fix
/// test` build an executable in a temporary place, run it, and remove it.
#[test]
fn test_a_run_writes_no_output_file() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "named_output");
    for command in ["run", "test"] {
        assert_succeeded(
            &run_fix(&project_dir, &[command, "-O", "none"]),
            &format!(
                "`fix {}` should succeed for a project naming its output file.",
                command
            ),
        );
        assert!(
            !project_dir.join(NAMED_OUTPUT_FILE).exists(),
            "`fix {}` should write no file at \"{}\", which names what `fix build` produces.",
            command,
            NAMED_OUTPUT_FILE,
        );
    }
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
    assert!(
        run_program(&project_dir.join(default_executable_name())).contains(PROGRAM_OUTPUT),
        "the program of the first build should stand beside the dynamic library.",
    );
    assert!(
        project_dir.join(default_dynamic_library_name()).exists(),
        "the second build should produce \"{}\", the dynamic library it was asked for.",
        default_dynamic_library_name(),
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
    assert!(
        project_dir.join(default_dynamic_library_name()).exists(),
        "the dynamic library of the first build should stand beside the program.",
    );
    assert!(
        run_program(&project_dir.join(default_executable_name())).contains(PROGRAM_OUTPUT),
        "the second build should produce \"{}\", the program it was asked for.",
        default_executable_name(),
    );
}
