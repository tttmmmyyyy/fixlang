//! Tests of the Fix examples written in doc comments: the examples of `Std` itself, and what
//! `fix test` does with the examples of a project.

use crate::commands::run::{build_executable, run};
use crate::configuration::Configuration;
use crate::doc_test::{collect_examples, examples_in_document, ExampleTask};
use crate::error::panic_if_err;
use crate::parse::parser::parse_file_path;
use crate::parse::sourcefile::{DocLine, SourceFile, Span};
use crate::tests::test_util::fix_command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// The Fix examples of `Std` pass: those of the doc comments in `std.fix`, and those of the
/// documents under `src/docs/`, which `Std` carries for the values the compiler defines.
#[test]
fn test_std_doc_examples() {
    let config = Configuration::develop_mode();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let std_path = manifest_dir.join("src/fixstd/std.fix");
    let std_program = panic_if_err(parse_file_path(std_path.clone(), &config));
    let mut examples = panic_if_err(collect_examples(&std_program, &[std_path]));

    let mut documents = fs::read_dir(manifest_dir.join("src/docs"))
        .expect("the documents of `Std` are readable")
        .map(|entry| entry.expect("the documents of `Std` are listed").path())
        .collect::<Vec<_>>();
    documents.sort();
    for document in documents {
        let doc_lines = markdown_file_lines(&document);
        examples.extend(panic_if_err(examples_in_document(
            &doc_lines,
            &"Std".to_string(),
        )));
    }

    let mut failures = vec![];
    let mut tested = 0;
    for example in &examples {
        let failure = match &example.task {
            ExampleTask::Ignore => continue,
            ExampleTask::Compile(source) => {
                let mut config = config.clone();
                config.doc_test_example = Some(source.clone());
                build_executable(config)
                    .err()
                    .map(|errors| errors.to_string())
            }
            ExampleTask::Run(source) => {
                let mut config = config.clone();
                config.doc_test_example = Some(source.clone());
                match run(config, false) {
                    Err(errors) => Some(errors.to_string()),
                    Ok(Err(e)) => Some(format!("Failed to run the program: {}", e)),
                    Ok(Ok(output)) if output.status.success() => None,
                    Ok(Ok(output)) => Some(format!(
                        "The program ended with {}.\n{}",
                        output.status,
                        String::from_utf8_lossy(&output.stderr)
                    )),
                }
            }
        };
        tested += 1;
        if let Some(failure) = failure {
            failures.push(format!("{}:\n{}", example.location(), failure));
        }
    }
    assert!(
        tested > 0,
        "`Std` documents its values with Fix examples that are tested"
    );
    assert!(
        failures.is_empty(),
        "the Fix examples of `Std` pass, but these fail:\n{}",
        failures.join("\n")
    );
}

/// The lines of the Markdown file at `path`, each with the span it stands at, as the document of a
/// declaration whose comment is that file.
fn markdown_file_lines(path: &Path) -> Vec<DocLine> {
    let source = SourceFile::from_file_path(path.to_path_buf());
    let content = panic_if_err(source.string());
    let mut lines = vec![];
    let mut line_start = 0;
    for line in content.split('\n') {
        let text = line.trim_end();
        lines.push(DocLine {
            text: text.to_string(),
            span: Span {
                input: source.clone(),
                start: line_start,
                end: line_start + text.len(),
            },
        });
        line_start += line.len() + 1;
    }
    lines
}

/// A temporary directory holding a project whose `build` section lists `build_files` and whose
/// `build.test` section lists `test_files`, each given as its path and its content.
fn project_dir(build_files: &[(&str, &str)], test_files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp directory");
    let list = |files: &[(&str, &str)]| {
        files
            .iter()
            .map(|(path, _)| format!("\"{}\"", path))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut project_file = format!(
        "[general]\nname = \"doc-test-case\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [{}]\n",
        list(build_files)
    );
    if !test_files.is_empty() {
        project_file += &format!("\n[build.test]\nfiles = [{}]\n", list(test_files));
    }
    fs::write(dir.path().join("fixproj.toml"), project_file)
        .expect("Failed to write the project file");
    for (path, content) in build_files.iter().chain(test_files) {
        fs::write(dir.path().join(path), content).expect("Failed to write a source file");
    }
    dir
}

/// Runs `fix test` with `args` in `dir`.
fn fix_test(dir: &TempDir, args: &[&str]) -> Output {
    fix_command()
        .arg("test")
        .args(args)
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute fix test")
}

/// The standard error of `output`, followed by its standard output, for a failure message.
fn streams(output: &Output) -> String {
    format!(
        "stderr:\n{}\nstdout:\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    )
}

/// A module `Lib` whose doc comments hold a Fix example of each kind: one written as statements,
/// one written as a module, one marked `no_run` whose run would fail, and one marked `ignore` that
/// does not compile.
const LIB_WITH_PASSING_EXAMPLES: &str = r#"
// A library.
//
// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
module Lib;

// Doubles a number.
//
// ```fix
// # module DocTest;
// # import Lib;
// type Pair = (I64, I64);
// main : IO () = (
//     let pair : DocTest::Pair = (double(1), double(2));
//     assert_eq(|_|"", pair, (2, 4))
// );
// ```
//
// ```fix,no_run
// assert_eq(|_|"this example is compiled alone", double(1), 0)
// ```
//
// ```fix,ignore
// double(undefined_name)
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;

/// A module `Test` whose `Test::test` announces that it ran.
const TEST_ANNOUNCING_ITSELF: &str = r#"
module Test;
import Lib;

test : IO ();
test = (
    assert_eq(|_|"", double(2), 4);;
    println("Test::test ran")
);
"#;

/// `fix test` runs `Test::test` and then each Fix example of the doc comments: an example written
/// as statements and one written as a module run and pass, one marked `no_run` is compiled alone,
/// and one marked `ignore` is left out.
#[test]
fn test_fix_test_runs_the_test_function_and_then_the_examples() {
    let dir = project_dir(
        &[("lib.fix", LIB_WITH_PASSING_EXAMPLES)],
        &[("test.fix", TEST_ANNOUNCING_ITSELF)],
    );
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "fix test passes\n{}",
        streams(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Test::test ran"),
        "`Test::test` runs\n{}",
        streams(&output)
    );
    for line in [4, 11, 21] {
        assert!(
            stderr.contains(&format!("doc test lib.fix:{} ... ok", line)),
            "the example opened at line {} passes\n{}",
            line,
            streams(&output)
        );
    }
    assert!(
        stderr.contains("doc test lib.fix:25 ... ignored"),
        "the example marked `ignore` is reported as ignored\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc tests: 3 passed, 0 failed, 1 ignored."),
        "three examples pass and one is ignored\n{}",
        streams(&output)
    );
}

/// `--doc` runs the Fix examples alone, and `--no-doc` runs `Test::test` alone.
#[test]
fn test_fix_test_selects_the_tests_by_the_doc_options() {
    let dir = project_dir(
        &[("lib.fix", LIB_WITH_PASSING_EXAMPLES)],
        &[("test.fix", TEST_ANNOUNCING_ITSELF)],
    );

    let output = fix_test(&dir, &["--doc"]);
    assert!(
        output.status.success(),
        "fix test --doc passes\n{}",
        streams(&output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("Test::test ran"),
        "`--doc` leaves `Test::test` out\n{}",
        streams(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("doc tests: 3 passed"),
        "`--doc` runs the examples\n{}",
        streams(&output)
    );

    let output = fix_test(&dir, &["--no-doc"]);
    assert!(
        output.status.success(),
        "fix test --no-doc passes\n{}",
        streams(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Test::test ran"),
        "`--no-doc` runs `Test::test`\n{}",
        streams(&output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("doc test"),
        "`--no-doc` leaves the examples out\n{}",
        streams(&output)
    );
}

/// A project without `Test::test` has its Fix examples run alone, and a project with neither is
/// reported as missing `Test::test`.
#[test]
fn test_fix_test_without_the_test_function() {
    let dir = project_dir(&[("lib.fix", LIB_WITH_PASSING_EXAMPLES)], &[]);
    let output = fix_test(&dir, &[]);
    assert!(
        output.status.success(),
        "the examples run without `Test::test`\n{}",
        streams(&output)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("doc tests: 3 passed"),
        "the examples run without `Test::test`\n{}",
        streams(&output)
    );

    let dir = project_dir(
        &[("lib.fix", "module Lib;\nvalue : I64;\nvalue = 1;\n")],
        &[],
    );
    let output = fix_test(&dir, &[]);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("`Test::test` is not found"),
        "a project with neither `Test::test` nor an example reports the missing `Test::test`\n{}",
        streams(&output)
    );
}

/// A Fix example that fails is reported with what it wrote to the standard error, the examples
/// after it still run, and `fix test` fails. A compile error of an example is reported at its
/// place in the doc comment.
#[test]
fn test_fix_test_reports_a_failing_example_and_runs_the_rest() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"the first example fails", 1, 2)
// ```
//
// ```fix
//     # let x : I64 = "a string";
// pure()
// ```
//
// ```fix
// assert_eq(|_|"", 1, 1)
// ```
value : I64;
value = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "fix test fails\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:3 ... FAILED")
            && stderr.contains("the first example fails"),
        "the failing example is reported with its standard error\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("8:24-8:34 in \"lib.fix\"")
            && stderr.contains("8 | //     # let x : I64 = \"a string\";"),
        "the compile error of a hidden line is reported at its place in the doc comment\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:12 ... ok"),
        "the example after the failing ones runs\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc tests: 1 passed, 2 failed, 0 ignored.")
            && stderr.contains("failures:\n    lib.fix:3\n    lib.fix:7"),
        "the summary lists the failing examples\n{}",
        streams(&output)
    );
}

/// A compile error on a line the tool wraps an example in is reported at the fence that line is
/// written for: an example ending in `;;` misses the expression the wrapper expects after it.
#[test]
fn test_error_on_the_wrapper_is_reported_at_the_fence() {
    let lib = "module Lib;\n\n// ```fix\n// println(\"a\");;\n// ```\nvalue : I64;\nvalue = 1;\n";
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("5:4-5:4 in \"lib.fix\"") && stderr.contains("5 | // ```"),
        "the error after the last action is reported at the closing fence\n{}",
        streams(&output)
    );
}

/// The info string of a Fix example carries `ignore`, `no_run` or neither, and a doc comment closes
/// each example it opens. `fix test` rejects any other before it runs a test.
#[test]
fn test_malformed_examples_are_rejected() {
    let lib = r#"module Lib;

// ```fix, no-run
// pure()
// ```
//
// ```fix,ignore,no_run
// pure()
// ```
//
// ```fix
// module Other;
// ```
//
// ```fix
value : I64;
value = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[("test.fix", TEST_ANNOUNCING_ITSELF)]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "fix test fails\n{}",
        streams(&output)
    );
    for (message, line) in [
        ("Unknown mark `no-run`", "3 | // ```fix, no-run"),
        (
            "is marked both `ignore` and `no_run`",
            "7 | // ```fix,ignore,no_run",
        ),
        ("declares the module `Other`", "12 | // module Other;"),
        (
            "The doc comment ends inside this Fix example",
            "15 | // ```fix",
        ),
    ] {
        assert!(
            stderr.contains(message) && stderr.contains(line),
            "\"{}\" is reported at \"{}\"\n{}",
            message,
            line,
            streams(&output)
        );
    }
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("Test::test ran"),
        "no test runs\n{}",
        streams(&output)
    );
}

/// `fix test` rejects a module named `DocTest`, the name each Fix example is compiled as.
#[test]
fn test_the_module_name_doc_test_is_reserved() {
    let dir = project_dir(
        &[
            ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
            ("doc_test.fix", "module DocTest;\n"),
        ],
        &[],
    );
    let output = fix_test(&dir, &[]);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("The module name `DocTest` is reserved"),
        "a module named `DocTest` is rejected\n{}",
        streams(&output)
    );
}

/// The Fix examples of the files the `build.test` section alone lists are not tested.
#[test]
fn test_examples_of_test_files_are_not_tested() {
    let test = r#"
module Test;

// ```fix
// assert_eq(|_|"an example of a test file runs", 1, 2)
// ```
test : IO ();
test = pure();
"#;
    let dir = project_dir(
        &[("lib.fix", "module Lib;\nvalue : I64;\nvalue = 1;\n")],
        &[("test.fix", test)],
    );
    let output = fix_test(&dir, &[]);
    assert!(
        output.status.success(),
        "the example of the test file is left out\n{}",
        streams(&output)
    );
}

/// `fix docs` shows a Fix example without its hidden lines, and with its info string replaced by
/// `fix`.
#[test]
fn test_fix_docs_hides_the_hidden_lines() {
    let dir = project_dir(&[("lib.fix", LIB_WITH_PASSING_EXAMPLES)], &[]);
    let output = fix_command()
        .arg("docs")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute fix docs");
    assert!(
        output.status.success(),
        "fix docs passes\n{}",
        streams(&output)
    );
    let document =
        fs::read_to_string(dir.path().join("docs/Lib.md")).expect("fix docs writes Lib.md");
    assert!(
        document.contains(
            "```fix\ntype Pair = (I64, I64);\nmain : IO () = (\n    let pair : DocTest::Pair"
        ),
        "the hidden lines are left out of the document:\n{}",
        document
    );
    assert!(
        document.contains("```fix\nassert_eq(|_|\"this example is compiled alone\"")
            && document.contains("```fix\ndouble(undefined_name)")
            && !document.contains("no_run")
            && !document.contains("ignore"),
        "the marks are left out of the document:\n{}",
        document
    );
}

/// A module `Lib` that documents a declaration of each kind with a Fix example: the module, a union
/// and its variant, a struct and its field, a trait with its associated type and its member, a
/// trait implementation, a trait alias and a global value.
const LIB_DOCUMENTING_EVERY_KIND: &str = r#"// ```fix
// assert_eq(|_|"", Shape::circle(1).is_circle, true)
// ```
module Lib;

// ```fix
// assert_eq(|_|"", Shape::square(2).as_square, 2)
// ```
type Shape = union {
    // ```fix
    // assert_eq(|_|"", Shape::circle(3).as_circle, 3)
    // ```
    circle : I64,
    square : I64
};

// ```fix
// assert_eq(|_|"", Point { x : 1 }.@x, 1)
// ```
type Point = struct {
    // ```fix
    // assert_eq(|_|"", Point { x : 2 }.@x, 2)
    // ```
    x : I64
};

// ```fix
// assert_eq(|_|"", 3.describe, "a number")
// ```
trait a : Describe {
    // ```fix
    // assert_eq(|_|"", 4.describe, "a number")
    // ```
    type Kind a;
    // ```fix
    // assert_eq(|_|"", 5.describe, "a number")
    // ```
    describe : a -> String;
}

// ```fix
// assert_eq(|_|"", 6.describe, "a number")
// ```
impl I64 : Describe {
    type Kind I64 = Bool;
    describe = |_| "a number";
}

// ```fix
// assert_eq(|_|"", 7.describe, "a number")
// ```
trait Numeric = Describe + Add;

// ```fix
// assert_eq(|_|"", double(4), 8)
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;

/// A module `Util` whose doc comment holds a Fix example that uses `Util` by the short names.
const UTIL_DOCUMENTING_A_VALUE: &str = r#"module Util;

// ```fix
// assert_eq(|_|"", triple(1), 3)
// ```
triple : I64 -> I64;
triple = |x| 3 * x;
"#;

/// The line numbers of the lines of `source` that open a Fix example.
fn fix_fence_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with("// ```fix"))
        .map(|(index, _)| index + 1)
        .collect()
}

/// `fix test` runs the Fix example of every declaration that carries a doc comment, each once, and
/// an example written as statements imports the module of the file its doc comment is written in.
#[test]
fn test_examples_of_every_kind_of_declaration_run_once() {
    let dir = project_dir(
        &[
            ("lib.fix", LIB_DOCUMENTING_EVERY_KIND),
            ("util.fix", UTIL_DOCUMENTING_A_VALUE),
        ],
        &[],
    );
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "fix test passes\n{}",
        streams(&output)
    );
    let mut example_count = 0;
    for (file, source) in [
        ("lib.fix", LIB_DOCUMENTING_EVERY_KIND),
        ("util.fix", UTIL_DOCUMENTING_A_VALUE),
    ] {
        for line in fix_fence_lines(source) {
            let report = format!("doc test {}:{} ... ok", file, line);
            assert_eq!(
                stderr.matches(&report).count(),
                1,
                "the example opened at {}:{} runs once and passes\n{}",
                file,
                line,
                streams(&output)
            );
            example_count += 1;
        }
    }
    assert!(
        stderr.contains(&format!(
            "doc tests: {} passed, 0 failed, 0 ignored.",
            example_count
        )),
        "each example runs once\n{}",
        streams(&output)
    );
}

/// A failure of `Test::test` makes `fix test` fail and is listed among the failures, and the Fix
/// examples still run after it.
#[test]
fn test_failing_test_function_is_reported_and_the_examples_run() {
    let test = "module Test;\ntest : IO ();\ntest = assert_eq(|_|\"Test::test fails\", 1, 2);\n";
    let dir = project_dir(
        &[("lib.fix", LIB_WITH_PASSING_EXAMPLES)],
        &[("test.fix", test)],
    );
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "fix test fails\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc tests: 3 passed, 0 failed, 1 ignored."),
        "the examples run after `Test::test` fails\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("failures:\n    Test::test\n"),
        "`Test::test` is listed among the failures\n{}",
        streams(&output)
    );
}

/// A compile error of `Test::test` stops `fix test` before it runs a Fix example.
#[test]
fn test_compile_error_of_the_test_function_stops_before_the_examples() {
    let test = "module Test;\ntest : IO ();\ntest = pure(1);\n";
    let dir = project_dir(
        &[("lib.fix", LIB_WITH_PASSING_EXAMPLES)],
        &[("test.fix", test)],
    );
    let output = fix_test(&dir, &[]);
    assert!(
        !output.status.success(),
        "fix test fails\n{}",
        streams(&output)
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("doc test"),
        "no example runs\n{}",
        streams(&output)
    );
}

/// A Fix example marked `no_run` that does not compile fails.
#[test]
fn test_no_run_example_that_does_not_compile_fails() {
    let lib = r#"module Lib;

// ```fix,no_run
// let x : I64 = "a string";
// pure()
// ```
value : I64;
value = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.contains("doc test lib.fix:3 ... FAILED"),
        "the example that does not compile fails\n{}",
        streams(&output)
    );
}

/// `fix docs` hides the hidden lines of each doc comment on its own: a Fix example one doc comment
/// leaves unclosed does not change how the examples of the doc comments after it are shown.
#[test]
fn test_fix_docs_reads_each_doc_comment_on_its_own() {
    let lib = r#"module Lib;

// ```fix
// unclosed(1)
first : I64;
first = 1;

// ```fix,no_run
// # let hidden = 1;
// let shown = 2;
// pure()
// ```
second : I64;
second = 2;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_command()
        .arg("docs")
        .current_dir(dir.path())
        .output()
        .expect("Failed to execute fix docs");
    assert!(
        output.status.success(),
        "fix docs passes\n{}",
        streams(&output)
    );
    let document =
        fs::read_to_string(dir.path().join("docs/Lib.md")).expect("fix docs writes Lib.md");
    assert!(
        document.contains("```fix\nlet shown = 2;\npure()\n```") && !document.contains("hidden"),
        "the example of the second doc comment is shown without its hidden line:\n{}",
        document
    );
}
