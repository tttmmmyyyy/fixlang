//! `skip_eval` compiles `eval {side}; {main}` as `{main}`, so the effect of `{side}` is left out of
//! the program.

use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::tests::test_util::{assert_succeeded, run_fix, run_source_capture, setup_case_projects};
use std::path::Path;

/// The directory holding this module's case projects.
const CASES: &str = "src/tests/test_skip_eval/cases";

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

/// Compiles and runs `source` with the setting as given, and returns its stdout and stderr.
///
/// `develop_mode` runs the program under valgrind's memcheck with `--error-exitcode=1`, so the
/// status this asserts on is what carries the memory check: dropping an `eval` changes which values
/// the program produces and releases, `Std`'s own `eval` among them.
fn run_with(source: &str, opt_level: FixOptimizationLevel, skip_eval: bool) -> (String, String) {
    let mut config = Configuration::develop_mode();
    config.set_fix_opt_level(opt_level);
    config.skip_eval = skip_eval;
    let output = run_source_capture(source, config);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "the program should exit cleanly at {} with skip_eval={}.\nstdout: {}\nstderr: {}",
        opt_level,
        skip_eval,
        stdout,
        stderr
    );
    (stdout, stderr)
}

/// Runs `SOURCE` at `opt_level` and returns its stdout and stderr.
fn run_at(opt_level: FixOptimizationLevel, skip_eval: bool) -> (String, String) {
    run_with(SOURCE, opt_level, skip_eval)
}

/// Every optimization level, since dropping the side expression is what the setting asks for.
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
    let (stdout, _stderr) = run_with(source, FixOptimizationLevel::Max, true);
    assert!(
        stdout.contains("from the bound action") && stdout.contains("done"),
        "the bound action should be performed.\nstdout: {}",
        stdout
    );
}

/// An `eval` is dropped wherever it sits in an expression, so the traversal has to reach under every
/// kind of expression that can contain one. Each `eval` here is nested under a different kind — a
/// `let` value, an `if` branch, a `match` arm, a struct-literal field, an array-literal element, a
/// lambda body, an application argument, a type annotation — and a traversal that stopped at any of
/// them would leave that line printed.
#[test]
fn test_a_nested_side_expression_is_skipped() {
    let source = r#"
            module Main;

            type Wrapper = struct { value : I64 };

            add_six : I64 -> I64;
            add_six = |x| x + 6;

            nested : I64;
            nested = (
                let from_let = (eval debug_eprintln("in let"); 1);
                let from_if = if from_let == 1 { eval debug_eprintln("in if"); 2 } else { 0 };
                let from_match = match some(from_if) {
                    some(v) => (eval debug_eprintln("in match"); v),
                    none(_) => 0
                };
                let from_struct = Wrapper { value : (eval debug_eprintln("in struct"); 3) }.@value;
                let from_array = [(eval debug_eprintln("in array"); 4)].@(0);
                let from_lambda = (|x| (eval debug_eprintln("in lambda"); x))(5);
                let from_arg = add_six((eval debug_eprintln("in argument"); 0));
                let from_anno = ((eval debug_eprintln("in annotation"); 7) : I64);
                from_let + from_if + from_match + from_struct + from_array
                    + from_lambda + from_arg + from_anno
            );

            main : IO ();
            main = println("nested is " + nested.to_string);
        "#;
    let nested_evals = [
        "in let",
        "in if",
        "in match",
        "in struct",
        "in array",
        "in lambda",
        "in argument",
        "in annotation",
    ];

    let (stdout, stderr) = run_with(source, FixOptimizationLevel::Max, false);
    for message in nested_evals {
        assert!(
            stderr.contains(message),
            "`{}` should be printed with the setting off.\nstderr: {}",
            message,
            stderr
        );
    }
    assert!(stdout.contains("nested is 30"), "stdout: {}", stdout);

    let (stdout, stderr) = run_with(source, FixOptimizationLevel::Max, true);
    for message in nested_evals {
        assert!(
            !stderr.contains(message),
            "`{}` should be dropped with the setting on.\nstderr: {}",
            message,
            stderr
        );
    }
    assert!(
        stdout.contains("nested is 30"),
        "the values should be unchanged.\nstdout: {}",
        stdout
    );
}

/// Consecutive `eval` expressions are each dropped: the traversal rebuilds a node from its visited
/// children before replacing it, so the inner `eval` of a chain is gone by the time the outer one is
/// replaced by its main expression.
#[test]
fn test_a_chain_of_side_expressions_is_skipped() {
    let source = r#"
            module Main;

            answer : I64;
            answer = (
                eval debug_eprintln("first");
                eval debug_eprintln("second");
                eval debug_eprintln("third");
                42
            );

            main : IO ();
            main = println("answer is " + answer.to_string);
        "#;

    let (_stdout, stderr) = run_with(source, FixOptimizationLevel::Max, false);
    assert!(
        stderr.contains("first") && stderr.contains("second") && stderr.contains("third"),
        "every side expression should be evaluated with the setting off.\nstderr: {}",
        stderr
    );

    let (stdout, stderr) = run_with(source, FixOptimizationLevel::Max, true);
    assert!(
        !stderr.contains("first") && !stderr.contains("second") && !stderr.contains("third"),
        "every side expression should be dropped.\nstderr: {}",
        stderr
    );
    assert!(stdout.contains("answer is 42"), "stdout: {}", stdout);
}

// The setting also comes from the project file, whose `build.test` section decides it for a test
// build. The cases under `test_skip_eval/cases` write the same program into `main.fix` and
// `test.fix`, and differ in which section turns the setting on.

/// What the case projects print through `eval`.
const FROM_EVAL: &str = "from eval";

/// Runs `fix` with `args` in `project_dir` and returns what the program printed to stderr, having
/// asserted that the run succeeded.
fn stderr_of_run(project_dir: &Path, args: &[&str]) -> String {
    let output = run_fix(project_dir, args);
    assert_succeeded(
        &output,
        &format!("`fix {}` should succeed.", args.join(" ")),
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// The `build` section decides the setting for the program.
#[test]
fn test_build_section_skips_the_evaluation_for_the_program() {
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_skip_in_build");
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
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_skip_in_build");
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
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_skip_in_test");
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
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_skip_in_test");
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
    let (_temp_dir, project_dir) = setup_case_projects(CASES, "root_skip_in_build");
    let stderr_without_option = stderr_of_run(&project_dir, &["test"]);
    assert!(
        stderr_without_option.contains(FROM_EVAL),
        "a test build should keep the evaluation.\nstderr: {}",
        stderr_without_option
    );
    let stderr_with_option = stderr_of_run(&project_dir, &["test", "--skip-eval"]);
    assert!(
        !stderr_with_option.contains(FROM_EVAL),
        "`--skip-eval` should skip the evaluation, whichever subcommand it is given to.\nstderr: {}",
        stderr_with_option
    );
}
