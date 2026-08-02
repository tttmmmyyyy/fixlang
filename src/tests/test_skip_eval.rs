//! `skip_eval` compiles `eval {side}; {main}` as `{main}`, so the effect of `{side}` is left out of
//! the program.

use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::tests::test_util::{copy_dir_recursive, fix_command, run_source_capture};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// Prints one line through the `IO` monad and one through `eval`, so that a run shows which of the
/// two the build kept.
const SOURCE: &str = r#"
        module Main;

        answer : I64;
        answer = (
            eval debug_eprintln("from eval");
            42
        );

        main : IO ();
        main = println("answer is " + answer.to_string);
    "#;

/// Runs `SOURCE` at `opt_level` and returns its stdout and stderr.
fn run_at(opt_level: FixOptimizationLevel, skip_eval: bool) -> (String, String) {
    let mut config = Configuration::develop_mode();
    config.set_fix_opt_level(opt_level);
    config.skip_eval = skip_eval;
    let output = run_source_capture(SOURCE, config);
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Every optimization level, since dropping the side expression is what the setting asks for rather
/// than a cost the optimizer is trading away.
const OPT_LEVELS: [FixOptimizationLevel; 3] = [
    FixOptimizationLevel::None,
    FixOptimizationLevel::Basic,
    FixOptimizationLevel::Max,
];

/// `eval` evaluates its side expression, which is what the setting turns off.
#[test]
fn test_the_side_expression_is_evaluated_by_default() {
    for opt_level in OPT_LEVELS {
        let (stdout, stderr) = run_at(opt_level, false);
        assert!(
            stderr.contains("from eval"),
            "`eval` should evaluate its side expression at {}.\nstderr: {}",
            opt_level.to_string(),
            stderr
        );
        assert!(stdout.contains("answer is 42"));
    }
}

/// The side expression is dropped, and the value of the whole expression is the one `{main}` has.
#[test]
fn test_the_side_expression_is_skipped() {
    for opt_level in OPT_LEVELS {
        let (stdout, stderr) = run_at(opt_level, true);
        assert!(
            !stderr.contains("from eval"),
            "the side expression should be dropped at {}.\nstderr: {}",
            opt_level.to_string(),
            stderr
        );
        assert!(
            stdout.contains("answer is 42"),
            "the value of `{{main}}` should be unchanged at {}.\nstdout: {}",
            opt_level.to_string(),
            stdout
        );
    }
}

/// A monadic action bound with `*` inside the side expression is still performed: `*` desugars into
/// a `bind` that sits outside the `eval` expression, so only the use of the value it produced is
/// dropped. This is what keeps code that chains actions with `eval` working under the setting.
#[test]
fn test_an_action_bound_in_the_side_expression_is_performed() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                eval *println("from the bound action");
                println("done")
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.skip_eval = true;
    let output = run_source_capture(source, config);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("from the bound action") && stdout.contains("done"),
        "the bound action should be performed.\nstdout: {}",
        stdout
    );
}

// The setting also comes from the project file, whose `build.test` section decides it for a test
// build. The cases under `test_skip_eval/cases` write the same program into `main.fix` and
// `test.fix`, and differ in which section turns the setting on.

/// What the case projects print through `eval`.
const FROM_EVAL: &str = "from eval";

/// Copies the case projects into a temporary directory and returns it with the path of `project`
/// inside it.
fn setup_test_env(project: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_skip_eval/cases");
    copy_dir_recursive(&cases, &temp_dir.path().to_path_buf()).expect("Failed to copy test cases");
    let project_dir = temp_dir.path().join(project);
    (temp_dir, project_dir)
}

/// Runs `fix` with `args` in `project_dir` and returns what the program printed to stderr, having
/// asserted that the run succeeded.
fn stderr_of_run(project_dir: &Path, args: &[&str]) -> String {
    let output: Output = fix_command()
        .args(args)
        .current_dir(project_dir)
        .output()
        .expect("Failed to execute fix");
    assert!(
        output.status.success(),
        "`fix {}` should succeed.\nstdout: {}\nstderr: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_skips_the_evaluation_for_the_program() {
    let (_temp_dir, project_dir) = setup_test_env("root_skip_in_build");
    let stderr = stderr_of_run(&project_dir, &["run"]);
    assert!(
        !stderr.contains(FROM_EVAL),
        "the build section should skip the evaluation.\nstderr: {}",
        stderr
    );
}

/// The `build` section's setting stays out of a test build.
#[test]
fn test_build_section_keeps_the_evaluation_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_skip_in_build");
    let stderr = stderr_of_run(&project_dir, &["test"]);
    assert!(
        stderr.contains(FROM_EVAL),
        "a test build should keep the evaluation.\nstderr: {}",
        stderr
    );
}

/// The `build.test` section is what skips the evaluation for a test build.
#[test]
fn test_test_section_skips_the_evaluation_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_skip_in_test");
    let stderr = stderr_of_run(&project_dir, &["test"]);
    assert!(
        !stderr.contains(FROM_EVAL),
        "the test section should skip the evaluation.\nstderr: {}",
        stderr
    );
}

/// The `build.test` section covers a test build alone, leaving the program's `eval` in place.
#[test]
fn test_test_section_keeps_the_evaluation_for_the_program() {
    let (_temp_dir, project_dir) = setup_test_env("root_skip_in_test");
    let stderr = stderr_of_run(&project_dir, &["run"]);
    assert!(
        stderr.contains(FROM_EVAL),
        "the test section should not reach the program.\nstderr: {}",
        stderr
    );
}

/// `--skip-eval` skips the evaluation for a test build whose project file keeps it.
///
/// The two runs share a project directory, so the second one meets the object files the first one
/// cached. The setting therefore has to be part of what identifies them.
#[test]
fn test_option_skips_the_evaluation_for_a_test() {
    let (_temp_dir, project_dir) = setup_test_env("root_skip_in_build");
    let kept = stderr_of_run(&project_dir, &["test"]);
    assert!(
        kept.contains(FROM_EVAL),
        "a test build should keep the evaluation.\nstderr: {}",
        kept
    );
    let skipped = stderr_of_run(&project_dir, &["test", "--skip-eval"]);
    assert!(
        !skipped.contains(FROM_EVAL),
        "`--skip-eval` should skip the evaluation, whichever subcommand it is given to.\nstderr: {}",
        skipped
    );
}
