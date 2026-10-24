//! Which subcommands run the `preliminary_commands` of a project and of its dependencies. `fix
//! build`, `fix run` and `fix test` run them without asking; `fix check`, `fix deps install`, `fix
//! deps update`, `fix docs` and the language server run none.

use crate::tests::test_lsp::lsp_client::LspClient;
use crate::tests::test_util::{assert_failed, assert_succeeded, create_git_repo, run_fix};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

/// The argv of a preliminary command that creates `marker`, written as a TOML array literal.
fn touch_command(marker: &Path) -> String {
    format!("[\"touch\", \"{}\"]", marker.display())
}

/// Writes into `dir` a project named `name` whose `build` section lists `build_commands` and whose
/// `build.test` section lists `test_commands`, each a TOML array literal of argvs. `extra` is
/// appended to the project file.
fn write_project(dir: &Path, name: &str, build_commands: &str, test_commands: &str, extra: &str) {
    fs::write(
        dir.join("fixproj.toml"),
        format!(
            "[general]\nname = \"{name}\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
             [build]\nfiles = [\"main.fix\"]\npreliminary_commands = {build_commands}\n\n\
             [build.test]\nfiles = [\"test.fix\"]\npreliminary_commands = {test_commands}\n{extra}",
        ),
    )
    .unwrap();
    fs::write(
        dir.join("main.fix"),
        "module Main;\n\nmain : IO ();\nmain = println(\"ok\");\n",
    )
    .unwrap();
    fs::write(
        dir.join("test.fix"),
        "module Test;\n\ntest : IO ();\ntest = println(\"testing\");\n",
    )
    .unwrap();
}

/// Writes into `root` a git repository whose project lists a preliminary command creating a marker
/// file, and into `root/proj` a project that depends on it and lists no command. Returns the
/// project directory and the marker's path.
fn project_with_git_dependency(root: &Path) -> (PathBuf, PathBuf) {
    let marker = root.join("dep-ran");
    let repo = root.join("dep-repo");
    fs::create_dir(&repo).unwrap();
    create_git_repo(
        &repo,
        &[
            (
                "fixproj.toml",
                &format!(
                    "[general]\nname = \"dep\"\nversion = \"0.1.0\"\nfix_version = \"*\"\n\n\
                     [build]\nfiles = [\"lib.fix\"]\npreliminary_commands = [{}]\n",
                    touch_command(&marker)
                ),
            ),
            (
                "lib.fix",
                "module Dep;\n\ngreet : IO ();\ngreet = println(\"hi\");\n",
            ),
        ],
    );
    let proj = root.join("proj");
    fs::create_dir(&proj).unwrap();
    write_project(
        &proj,
        "root",
        "[]",
        "[]",
        &format!(
            "\n[[dependencies]]\nname = \"dep\"\ngit = {{ url = \"file://{}\" }}\n",
            repo.display()
        ),
    );
    (proj, marker)
}

/// Verifies that `fix build` runs the commands of the `build` section and none of the `build.test`
/// section.
#[test]
fn build_runs_the_build_section_commands() {
    let dir = TempDir::new().unwrap();
    let build_marker = dir.path().join("build-ran");
    let test_marker = dir.path().join("test-ran");
    write_project(
        dir.path(),
        "proj",
        &format!("[{}]", touch_command(&build_marker)),
        &format!("[{}]", touch_command(&test_marker)),
        "",
    );
    assert_succeeded(&run_fix(dir.path(), &["build"]), "`fix build` should succeed.");
    assert!(build_marker.exists(), "`fix build` did not run the command");
    assert!(
        !test_marker.exists(),
        "`fix build` ran a command of the `build.test` section"
    );
}

/// Verifies that `fix test` runs the commands of both the `build` and the `build.test` sections.
#[test]
fn test_runs_the_commands_of_both_sections() {
    let dir = TempDir::new().unwrap();
    let build_marker = dir.path().join("build-ran");
    let test_marker = dir.path().join("test-ran");
    write_project(
        dir.path(),
        "proj",
        &format!("[{}]", touch_command(&build_marker)),
        &format!("[{}]", touch_command(&test_marker)),
        "",
    );
    assert_succeeded(&run_fix(dir.path(), &["test"]), "`fix test` should succeed.");
    assert!(build_marker.exists());
    assert!(test_marker.exists());
}

/// Verifies that `fix build` runs the commands a git dependency lists, without asking for approval.
#[test]
fn build_runs_the_commands_of_a_git_dependency() {
    let dir = TempDir::new().unwrap();
    let (proj, marker) = project_with_git_dependency(dir.path());
    assert_succeeded(&run_fix(&proj, &["build"]), "`fix build` should succeed.");
    assert!(marker.exists(), "`fix build` did not run the dependency's command");
}

/// Verifies that `--allow-preliminary-commands` is still accepted, prints a warning, and leaves the
/// commands to run.
#[test]
fn allow_preliminary_commands_is_accepted_with_a_warning() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("ran");
    write_project(
        dir.path(),
        "proj",
        &format!("[{}]", touch_command(&marker)),
        "[]",
        "",
    );
    let output = run_fix(dir.path(), &["build", "--allow-preliminary-commands"]);
    assert_succeeded(
        &output,
        "`fix build --allow-preliminary-commands` should succeed.",
    );
    assert!(marker.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`--allow-preliminary-commands` has no effect"),
        "stderr: {}",
        stderr
    );
}

/// Verifies that `fix check` runs no command of either section.
#[test]
fn check_runs_no_command() {
    let dir = TempDir::new().unwrap();
    let marker = dir.path().join("ran");
    write_project(
        dir.path(),
        "proj",
        &format!("[{}]", touch_command(&marker)),
        &format!("[{}]", touch_command(&marker)),
        "",
    );
    assert_succeeded(&run_fix(dir.path(), &["check"]), "`fix check` should succeed.");
    assert!(!marker.exists(), "`fix check` ran a preliminary command");
}

/// Verifies that `fix deps update`, `fix deps install`, `fix check` and `fix docs` run no command
/// of a git dependency.
#[test]
fn subcommands_that_build_nothing_run_no_command_of_a_git_dependency() {
    let dir = TempDir::new().unwrap();
    let (proj, marker) = project_with_git_dependency(dir.path());
    assert_succeeded(
        &run_fix(&proj, &["deps", "update"]),
        "`fix deps update` should succeed.",
    );
    // `fix clean` leaves the lock file, so that `fix deps install` clones the dependency afresh.
    assert_succeeded(&run_fix(&proj, &["clean"]), "`fix clean` should succeed.");
    assert_succeeded(
        &run_fix(&proj, &["deps", "install"]),
        "`fix deps install` should succeed.",
    );
    assert_succeeded(&run_fix(&proj, &["check"]), "`fix check` should succeed.");
    assert_succeeded(
        &run_fix(&proj, &["docs", "-o", "out"]),
        "`fix docs` should succeed.",
    );
    assert!(
        !marker.exists(),
        "a subcommand that builds nothing ran the dependency's command"
    );
}

/// Verifies that a command exiting with a nonzero status fails `fix build`.
#[test]
fn a_failing_command_fails_the_build() {
    let dir = TempDir::new().unwrap();
    write_project(dir.path(), "proj", "[[\"false\"]]", "[]", "");
    assert_failed(
        &run_fix(dir.path(), &["build"]),
        "`fix build` succeeded although a preliminary command failed",
    );
}

/// Verifies that the commands of one section run in the order the project file lists them.
#[test]
fn commands_run_in_the_order_the_project_file_lists_them() {
    let dir = TempDir::new().unwrap();
    let log = dir.path().join("log");
    let append = |word: &str| {
        format!(
            "[\"sh\", \"-c\", \"echo {} >> '{}'\"]",
            word,
            log.display()
        )
    };
    write_project(
        dir.path(),
        "proj",
        &format!("[{}, {}]", append("first"), append("second")),
        "[]",
        "",
    );
    assert_succeeded(&run_fix(dir.path(), &["build"]), "`fix build` should succeed.");
    // Consecutive repeats are merged, since a command runs twice (#949): the order is what this
    // test pins.
    let mut lines: Vec<String> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    lines.dedup();
    assert_eq!(lines, ["first", "second"]);
}

/// Verifies that the language server, elaborating a project with a git dependency, runs no command
/// of that dependency.
#[test]
fn the_language_server_runs_no_command_of_a_git_dependency() {
    let dir = TempDir::new().unwrap();
    let (proj, marker) = project_with_git_dependency(dir.path());
    // A program that calls the dependency, so that a pass reporting nothing is one that read the
    // dependency's project file.
    fs::write(
        proj.join("main.fix"),
        "module Main;\n\nimport Dep;\n\nmain : IO ();\nmain = Dep::greet;\n",
    )
    .unwrap();
    let mut client = LspClient::new(&proj).expect("Failed to start LSP");
    client
        .initialize(&proj, Duration::from_secs(5))
        .expect("Failed to initialize LSP");
    client
        .open_document(Path::new("main.fix"))
        .expect("Failed to open main.fix");
    client.save_and_wait_for_a_pass(Path::new("main.fix"));
    client
        .verify_no_diagnostics()
        .expect("the project is expected to elaborate with its dependency");
    client.shutdown().expect("Failed to shut down LSP");
    assert!(
        !marker.exists(),
        "the language server ran the dependency's command"
    );
}

