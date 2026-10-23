//! Which subcommands run the `preliminary_commands` of a project and of its dependencies. `fix
//! build`, `fix run` and `fix test` run them without asking, on a terminal or off one; `fix check`,
//! `fix deps update` and `fix docs` run none.

use crate::tests::test_util::{create_git_repo, fix_command};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use tempfile::TempDir;

/// Runs `fix <args>` in `dir` with no terminal on its standard input.
fn run_fix(dir: &Path, args: &[&str]) -> Output {
    fix_command()
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .expect("Failed to run fix.")
}

fn assert_succeeded(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

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

/// A project in `root`/`proj` that depends on a git repository in `root`, whose project lists a
/// preliminary command creating the returned marker. The root project lists none.
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
    assert_succeeded(&run_fix(dir.path(), &["build"]));
    assert!(build_marker.exists(), "`fix build` did not run the command");
    assert!(
        !test_marker.exists(),
        "`fix build` ran a command of the `build.test` section"
    );
}

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
    assert_succeeded(&run_fix(dir.path(), &["test"]));
    assert!(build_marker.exists());
    assert!(test_marker.exists());
}

#[test]
fn build_runs_the_commands_of_a_git_dependency() {
    let dir = TempDir::new().unwrap();
    let (proj, marker) = project_with_git_dependency(dir.path());
    assert_succeeded(&run_fix(&proj, &["build"]));
    assert!(marker.exists(), "`fix build` did not run the dependency's command");
}

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
    assert_succeeded(&output);
    assert!(marker.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`--allow-preliminary-commands` has no effect"),
        "stderr: {}",
        stderr
    );
}

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
    assert_succeeded(&run_fix(dir.path(), &["check"]));
    assert!(!marker.exists(), "`fix check` ran a preliminary command");
}

#[test]
fn check_deps_update_and_docs_run_no_command_of_a_git_dependency() {
    let dir = TempDir::new().unwrap();
    let (proj, marker) = project_with_git_dependency(dir.path());
    assert_succeeded(&run_fix(&proj, &["deps", "update"]));
    assert_succeeded(&run_fix(&proj, &["check"]));
    assert_succeeded(&run_fix(&proj, &["docs", "-o", "out"]));
    assert!(
        !marker.exists(),
        "a subcommand that builds nothing ran the dependency's command"
    );
}
