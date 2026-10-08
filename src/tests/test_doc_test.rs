//! Tests of the Fix examples written in comments: the examples of `Std` itself, and what
//! `fix test` does with the examples of a project.

use crate::commands::test::{test_examples, ExampleOutcome};
use crate::configuration::Configuration;
use crate::doc_test::{collect_examples, examples_in_text, TextLine};
use crate::error::panic_if_err;
use crate::parse::parser::parse_file_path;
use crate::parse::sourcefile::{SourceFile, Span};
use crate::tests::test_util::fix_command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

/// The Fix examples of `Std` pass: those of the comments in `std.fix`, and those of the
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
        examples.extend(panic_if_err(examples_in_text(
            &doc_lines,
            &"Std".to_string(),
        )));
    }

    let mut failures = vec![];
    let mut tested = 0;
    let result = test_examples(&config, &examples, |example, outcome| match outcome {
        ExampleOutcome::Ignored => {}
        ExampleOutcome::Passed => tested += 1,
        ExampleOutcome::Failed(failure) => {
            tested += 1;
            failures.push(format!("{}:\n{}", example.location(), failure));
        }
    });
    panic_if_err(result);
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
fn markdown_file_lines(path: &Path) -> Vec<TextLine> {
    let source = SourceFile::from_file_path(path.to_path_buf());
    let content = panic_if_err(source.string());
    let mut lines = vec![];
    let mut line_start = 0;
    for line in content.split('\n') {
        let text = line.trim_end();
        lines.push(TextLine {
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

/// A module `Lib` whose comments hold a Fix example of each kind: one written as statements,
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
// ```fix no_run
// assert_eq(|_|"this example is compiled alone", double(1), 0)
// ```
//
// ```fix ignore
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

/// `fix test` runs `Test::test` and then each Fix example of the comments: an example written
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
/// place in the comment.
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
        "the compile error of a hidden line is reported at its place in the comment\n{}",
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

/// The info string of a Fix example carries `ignore`, `no_run` or neither, separated from `fix` by
/// spaces, an example written as statements has an expression after its import statements, and a
/// comment closes each example it opens. `fix test` rejects any other before it runs a test.
#[test]
fn test_malformed_examples_are_rejected() {
    let lib = r#"module Lib;

// ```fix no-run
// pure()
// ```
//
// ```fix ignore no_run
// pure()
// ```
//
// ```fix
// module Other;
// ```
//
// ```fix,no_run
// pure()
// ```
//
// ```fix
// import Std;
//
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
        ("Unknown mark `no-run`", "3 | // ```fix no-run"),
        (
            "is marked both `ignore` and `no_run`",
            "7 | // ```fix ignore no_run",
        ),
        ("declares the module `Other`", "12 | // module Other;"),
        (
            "Separate the marks of a Fix example from `fix` by spaces, as in `fix no_run`.",
            "15 | // ```fix,no_run",
        ),
        (
            "A Fix example has import statements and no expression after them.",
            "19 | // ```fix",
        ),
        (
            "The comment ends inside this Fix example. Close it by a line of ```.",
            "24 | // ```fix",
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

/// `fix test` rejects a module named `DocTest` or with a name beginning with `DocTest.`, the names
/// the Fix examples are compiled under, at its declaration. A module whose name ends in `DocTest`
/// and a namespace, a type or a trait named `DocTest` are accepted, and so is the module `DocTest`
/// where no example is compiled: under `--no-doc`, and in a project without an example.
#[test]
fn test_the_module_name_doc_test_is_reserved() {
    let module = "module DocTest;\nvalue : I64;\nvalue = 1;\n";
    for reserved in ["DocTest", "DocTest.Examples"] {
        let dir = project_dir(
            &[
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                (
                    "doc_test.fix",
                    &module.replace("module DocTest;", &format!("module {};", reserved)),
                ),
            ],
            &[],
        );
        let output = fix_test(&dir, &[]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success()
                && stderr.contains(&format!("The module name `{}` is reserved", reserved))
                && stderr.contains(&format!("1 | module {};", reserved))
                && !stderr.contains("doc test"),
            "the module `{}` is rejected at its declaration before any example runs\n{}",
            reserved,
            streams(&output)
        );
    }

    let lib_without_examples = "module Lib;\ndouble : I64 -> I64;\ndouble = |x| 2 * x;\n";
    for (build_files, args) in [
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                (
                    "other.fix",
                    "module Other.DocTest;\nvalue : I64;\nvalue = 1;\n",
                ),
            ],
            vec![],
        ),
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                ("other.fix", "module DocTests;\nvalue : I64;\nvalue = 1;\n"),
            ],
            vec![],
        ),
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                (
                    "other.fix",
                    "module Other;\nnamespace DocTest {\n    value : I64;\n    value = 1;\n}\n",
                ),
            ],
            vec![],
        ),
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                (
                    "other.fix",
                    "module Other;\ntype DocTest = struct { x : I64 };\n",
                ),
            ],
            vec![],
        ),
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                (
                    "other.fix",
                    "module Other;\ntrait a : DocTest {\n    describe : a -> String;\n}\n",
                ),
            ],
            vec![],
        ),
        (
            vec![
                ("lib.fix", LIB_WITH_PASSING_EXAMPLES),
                ("doc_test.fix", module),
            ],
            vec!["--no-doc"],
        ),
        (
            vec![("lib.fix", lib_without_examples), ("doc_test.fix", module)],
            vec![],
        ),
    ] {
        let dir = project_dir(&build_files, &[("test.fix", TEST_ANNOUNCING_ITSELF)]);
        let output = fix_test(&dir, &args);
        assert!(
            output.status.success(),
            "`fix test {}` accepts\n{:?}\n{}",
            args.join(" "),
            build_files,
            streams(&output)
        );
    }
}

/// The Fix examples of the files the `build.test` section alone lists are tested as well: a helper
/// module a project writes for its tests carries examples that run, and one written as statements
/// imports that module.
#[test]
fn test_examples_of_test_files_are_tested() {
    let test_util = r#"module TestUtil;

// ```fix
// assert_eq(|_|"", doubled_all([1, 2]), [2, 4])
// ```
doubled_all : Array I64 -> Array I64;
doubled_all = |xs| xs.map(|x| 2 * x);
"#;
    let dir = project_dir(
        &[("lib.fix", "module Lib;\nvalue : I64;\nvalue = 1;\n")],
        &[
            ("test_util.fix", test_util),
            ("test.fix", "module Test;\ntest : IO ();\ntest = pure();\n"),
        ],
    );
    let output = fix_test(&dir, &[]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("doc test test_util.fix:3 ... ok"),
        "the example of the test helper module runs and passes\n{}",
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

/// A module `Lib` that writes a Fix example in a comment of each kind and place: the doc comment
/// of the module, of a type, of a field and of a value, a comment inside the body of a value, a
/// comment a blank line keeps apart from the declaration below it, a comment standing between two
/// declarations, and a `/* */` comment.
const LIB_WITH_EXAMPLES_IN_EVERY_COMMENT: &str = r#"// ```fix
// assert_eq(|_|"", double(1), 2)
// ```
module Lib;

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
// assert_eq(|_|"", double(3), 6)
// ```

// A comment a blank line keeps apart from `double`.
double : I64 -> I64;
double = |x| (
    // ```fix
    // assert_eq(|_|"", double(4), 8)
    // ```
    2 * x
);

// ```fix
// assert_eq(|_|"", double(5), 10)
// ```

/*
```fix
let y = triple(4);
assert_eq(|_|"", y, 12)
```
*/
triple : I64 -> I64;
triple = |x| 3 * x;
"#;

/// A module `Util` whose comment holds a Fix example that uses `Util` by the short names.
const UTIL_WITH_AN_EXAMPLE: &str = r#"module Util;

// ```fix
// assert_eq(|_|"", quadruple(1), 4)
// ```
quadruple : I64 -> I64;
quadruple = |x| 4 * x;
"#;

/// The line numbers of the lines of `source` that open a Fix example.
fn fix_fence_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let text = line.trim_start();
            text.starts_with("// ```fix") || text.starts_with("```fix")
        })
        .map(|(index, _)| index + 1)
        .collect()
}

/// `fix test` runs the Fix example of every comment, each once, wherever the comment stands and
/// whatever its kind, and an example written as statements imports the module of the file its
/// comment is written in.
#[test]
fn test_examples_of_every_comment_run_once() {
    let dir = project_dir(
        &[
            ("lib.fix", LIB_WITH_EXAMPLES_IN_EVERY_COMMENT),
            ("util.fix", UTIL_WITH_AN_EXAMPLE),
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
        ("lib.fix", LIB_WITH_EXAMPLES_IN_EVERY_COMMENT),
        ("util.fix", UTIL_WITH_AN_EXAMPLE),
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

// ```fix no_run
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

// ```fix no_run
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

/// A compile error at the end of a Fix example's source is reported inside the comment, at the
/// closing fence: an unterminated string swallows the end of an example written as statements, and
/// an unclosed call ends an example written as a module.
#[test]
fn test_error_at_the_end_of_an_example_is_reported_at_the_closing_fence() {
    let lib = r#"module Lib;

// ```fix
// println("abc
// ```
value : I64;
value = 1;

// ```fix
// module DocTest;
// main : IO () = pure(
// ```
other : I64;
other = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for (position, fence) in [("5:4-5:4", "5 | // ```"), ("12:4-12:4", "12 | // ```")] {
        assert!(
            stderr.contains(&format!("{} in \"lib.fix\"", position)) && stderr.contains(fence),
            "the error is reported at the closing fence \"{}\"\n{}",
            fence,
            streams(&output)
        );
    }
}

/// A compile error of a Fix example is reported at its place in the comment, whatever the kind of
/// the comment: a `/* */` comment, whose text is read as it is written, and a `//` comment written
/// without a space after the `//`.
#[test]
fn test_error_in_an_example_is_reported_at_its_place_in_each_kind_of_comment() {
    let lib = r#"module Lib;

/*
```fix
  let x : I64 = "a string";
pure()
```
*/
value : I64;
value = 1;

//```fix
//let y : I64 = "another string";
//pure()
//```
other : I64;
other = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for (kind, position, line) in [
        ("`/* */`", "5:17-5:27", "5 |   let x : I64 = \"a string\";"),
        (
            "`//` without a space",
            "13:17-13:33",
            "13 | //let y : I64 = \"another string\";",
        ),
    ] {
        assert!(
            stderr.contains(&format!("{} in \"lib.fix\"", position)) && stderr.contains(line),
            "the error in the example of the {} comment is reported at {} on \"{}\"\n{}",
            kind,
            position,
            line,
            streams(&output)
        );
    }
}

/// `fix docs` reads a line beginning with `#` inside a fenced code block as a line of the block, as
/// CommonMark does: in a block of tildes, and in a block of backticks after a shorter fence it holds.
#[test]
fn test_fix_docs_keeps_a_hash_line_of_a_code_block_in_the_block() {
    let lib = r#"module Lib;

// ~~~sh
// # a shell comment
// ~~~
first : I64;
first = 1;

// ````text
// ```
// # a line after a shorter fence
// ````
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
    for block in [
        "~~~sh\n# a shell comment\n~~~",
        "````text\n```\n# a line after a shorter fence\n````",
    ] {
        assert!(
            document.contains(block),
            "the block\n{}\nis shown as it is written:\n{}",
            block,
            document
        );
    }
}

/// An example with no line between its fences is compiled as the empty expression it is, and the
/// error is reported once, at the closing fence.
#[test]
fn test_error_in_an_empty_example_is_reported_at_the_closing_fence() {
    let lib = "module Lib;\n\n// ```fix\n// ```\nvalue : I64;\nvalue = 1;\n";
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success() && stderr.matches("4 | // ```").count() == 1,
        "the error of the empty example quotes its closing fence once\n{}",
        streams(&output)
    );
}

/// An example written as statements sees the names its module defines, and the modules its
/// module imports only through the import statements it begins with: an import statement written
/// as a hidden line, and one listing some items, which leaves the others out.
#[test]
fn test_example_sees_what_its_import_statements_name() {
    let util = "module Util;\ntriple : I64 -> I64;\ntriple = |x| 3 * x;\nquadruple : I64 -> I64;\nquadruple = |x| 4 * x;\n";
    let lib = r#"module Lib;
import Util::{triple};

// ```fix
// # import Util::{triple};
// assert_eq(|_|"", sextuple(1), triple(2))
// ```
//
// ```fix no_run
// assert_eq(|_|"", sextuple(1), triple(2))
// ```
//
// ```fix no_run
// import Util::{triple};
//
// assert_eq(|_|"", sextuple(1), quadruple(1))
// ```
sextuple : I64 -> I64;
sextuple = |x| 2 * triple(x);
"#;
    let dir = project_dir(&[("lib.fix", lib), ("util.fix", util)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:4 ... ok"),
        "the example uses `triple`, which its hidden import statement names\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:9 ... FAILED")
            && stderr.contains("10 | // assert_eq(|_|\"\", sextuple(1), triple(2))"),
        "the example cannot use `triple`, which only its module imports\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:13 ... FAILED")
            && stderr.contains("16 | // assert_eq(|_|\"\", sextuple(1), quadruple(1))"),
        "the example cannot use `quadruple`, which its import statement leaves out\n{}",
        streams(&output)
    );
}

/// The Fix examples of a project are compiled together into one program, which is built once: the
/// warning a source raises is printed once, however many examples there are. In that program each
/// example still reaches its own module by `DocTest`, by a relative path and by an absolute one,
/// and an example that panics fails alone.
#[test]
fn test_examples_are_built_once_into_one_program() {
    let lib = r#"module Lib;

DEPRECATED[old_double, "Call `double` in place of `old_double`."];
old_double : I64 -> I64;
old_double = |x| 2 * x;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// # module DocTest;
// # import Lib;
// # type Pair = struct { fst : I64, snd : I64 };
// # half : I64 = 21;
// # main : IO () = (
// let pair : ::DocTest::Pair = DocTest::Pair { fst : ::DocTest::half, snd : DocTest::half };
// assert_eq(|_|"", double(pair.@fst), 42)
// # );
// ```
//
// ```fix no_run
// let x = double(1);
// assert_eq(|_|"", x, 3)
// ```
//
// ```fix
// let x = [1].@(5);
// assert_eq(|_|"", x, 1)
// ```
//
// ```fix
// assert_eq(|_|"", double(2), 4)
// ```
double : I64 -> I64;
double = |x| old_double(x);
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr
            .matches("Call `double` in place of `old_double`.")
            .count(),
        1,
        "the program of the examples is built once\n{}",
        streams(&output)
    );
    for line in [
        "doc test lib.fix:7 ... ok",
        "doc test lib.fix:11 ... ok",
        "doc test lib.fix:22 ... ok",
        "doc test lib.fix:27 ... FAILED",
        "doc test lib.fix:32 ... ok",
        "doc tests: 4 passed, 1 failed, 0 ignored.",
    ] {
        assert!(
            stderr.contains(line),
            "`fix test` reports `{}`\n{}",
            line,
            streams(&output)
        );
    }
    assert!(
        stderr.contains("Index out of range"),
        "the example that panics shows what it wrote\n{}",
        streams(&output)
    );
}

/// Two Fix examples that cannot be compiled into one program, as two that export functions under
/// one C name, are tested one by one, where each passes: the program of the examples fails to
/// build, and the sources build alone. An error of the sources is reported once, and no example is
/// reported.
#[test]
fn test_examples_that_cannot_share_a_program_are_tested_alone() {
    let example_exporting = |value: &str| {
        format!(
            "// ```fix\n\
             // # module DocTest;\n\
             // # {value} : CInt -> CInt = |x| x;\n\
             // # FFI_EXPORT[{value}, doc_test_exported];\n\
             // # main : IO () = (\n\
             // pure()\n\
             // # );\n\
             // ```\n"
        )
    };
    let lib = format!(
        "module Lib;\n\n{}//\n{}//\n// ```fix\n// assert_eq(|_|\"\", value, 1)\n// ```\nvalue : I64 = 1;\n",
        example_exporting("first"),
        example_exporting("second"),
    );
    let dir = project_dir(&[("lib.fix", &lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("doc tests: 3 passed, 0 failed, 0 ignored."),
        "each example passes\n{}",
        streams(&output)
    );

    let broken_lib = lib.replace("value : I64 = 1;", "value : I64 = \"one\";");
    let dir = project_dir(&[("lib.fix", &broken_lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && stderr.matches("Type mismatch").count() == 1
            && !stderr.contains("doc test"),
        "the error of the source is reported once, and no example is reported\n{}",
        streams(&output)
    );
}

/// An example written as statements sees the whole of `Std`, however its module narrows it, and
/// it narrows `Std` by the import statements it begins with, as any module does. The `main` the
/// example is wrapped into is of type `IO ()` however the example narrows `Std`.
#[test]
fn test_example_sees_std_as_its_import_statements_narrow_it() {
    let lib = r#"module Lib;
import Std::{I64, Monad::pure};

// ```fix
// assert_eq(|_|"", [value, value].get_size, 2)
// ```
//
// ```fix
// # import Std::{Monad::pure};
// pure()
// ```
//
// ```fix no_run
// import Std::{Monad::pure};
// assert_eq(|_|"", value, 1)
// ```
value : I64 = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:4 ... ok"),
        "the example uses `assert_eq` and `get_size`, which its module does not import\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:8 ... ok"),
        "the example's `main` is of type `IO ()` where the example imports no `IO`\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("doc test lib.fix:13 ... FAILED") && stderr.contains("`assert_eq`"),
        "the example cannot use `assert_eq`, which its import statement leaves out of `Std`\n{}",
        streams(&output)
    );

    let lib = r#"module Lib;
import Std hiding Tuple2;

// ```fix
// # import Std hiding Tuple2;
// let pair = Tuple2 { fst : 1, snd : 2 };
// assert_eq(|_|"", pair.@snd, 2)
// ```
type Tuple2 = struct { fst : I64, snd : I64 };
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr).contains("doc test lib.fix:4 ... ok"),
        "the example's `Tuple2` is `Lib::Tuple2`, as the example hides `Std`'s\n{}",
        streams(&output)
    );
}

/// A compile error of an example written as statements after import statements is reported at
/// its place in the comment: on the line the expression begins on, into which the head of `main`
/// is written, whether the expression begins the line or follows an import statement and a comment
/// on it, and on the lines after it.
#[test]
fn test_error_in_an_example_after_import_statements_is_reported_at_its_place() {
    let lib = r#"module Lib;

// ```fix
// import Std;
// let x : I64 = "a string";
// pure()
// ```
value : I64 = 1;

// ```fix
// # import Std;
// let x = 1;
// let y : I64 = "a string";
// pure()
// ```
other : I64 = 1;

// ```fix
// import Std; /* the types */ let z : I64 = "a string";
// pure()
// ```
third : I64 = 1;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for (position, line) in [
        ("5:18-5:28", "5 | // let x : I64 = \"a string\";"),
        ("13:18-13:28", "13 | // let y : I64 = \"a string\";"),
        (
            "19:46-19:56",
            "19 | // import Std; /* the types */ let z : I64 = \"a string\";",
        ),
    ] {
        assert!(
            stderr.contains(&format!("{} in \"lib.fix\"", position)) && stderr.contains(line),
            "the error is reported at \"{}\" of \"{}\"\n{}",
            position,
            line,
            streams(&output)
        );
    }
}

/// A Fix example written as a module that defines no `main` fails as it does built alone. The
/// program of the examples does not build, so `fix test` says that it tests each example alone,
/// which is slower, and the other example passes.
#[test]
fn test_example_without_main_fails() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// # module DocTest;
// # import Lib;
// # value : I64 = double(1);
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("so the examples will be compiled one by one. This may take longer than usual.")
            && stderr.contains("doc test lib.fix:3 ... ok")
            && stderr.contains("doc test lib.fix:7 ... FAILED")
            && stderr.contains("doc tests: 1 passed, 1 failed, 0 ignored."),
        "`fix test` says that testing each example alone is slower, and only the example without `main` fails\n{}",
        streams(&output)
    );
}

/// A Fix example written as a module whose `main` has a type more general than `IO ()` fails as it
/// does built alone, though the program of the examples could run it as an `IO ()`.
#[test]
fn test_example_whose_main_is_of_a_more_general_type_fails() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// # module DocTest;
// # main : [m : Monad] m ();
// # main = pure();
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:3 ... ok")
            && stderr.contains("doc test lib.fix:7 ... FAILED")
            && stderr.contains("should have type `Std::IO ()`")
            && stderr.contains("doc tests: 1 passed, 1 failed, 0 ignored."),
        "the example whose `main` is of a more general type fails, and the other passes\n{}",
        streams(&output)
    );
}

/// A library that calls the C function `getenv` at a signature of its own has its Fix examples
/// tested: the program the examples are built into reads which example to run without declaring
/// `getenv` itself.
#[test]
fn test_examples_of_a_library_declaring_getenv_its_own_way() {
    let lib = r#"module Lib;

// Whether the environment variable `PATH` is set.
//
// ```fix
// assert_eq(|_|"", *path_is_set, true)
// ```
path_is_set : IO Bool = (
    let value = *"PATH".borrow_c_str_io(|name| FFI_CALL_IO[U64 getenv(Ptr), name]);
    pure(value != 0_U64)
);

// ```fix
// assert_eq(|_|"", 1 + 1, 2)
// ```
two : I64 = 2;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("doc tests: 2 passed, 0 failed, 0 ignored."),
        "both examples pass\n{}",
        streams(&output)
    );
}

/// A Fix example that fails to link, by calling a C function nothing defines, fails as it does
/// built alone, and the other example still passes: the program of the examples fails to link, the
/// sources build alone, and so each example is tested one by one.
#[test]
fn test_example_failing_to_link_fails_alone() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// let r = FFI_CALL[CInt doc_test_undefined_function(CInt), 1.to_CInt];
// assert_eq(|_|"", r.to_I64, 0)
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:3 ... ok")
            && stderr.contains("doc test lib.fix:7 ... FAILED")
            && stderr.contains("doc tests: 1 passed, 1 failed, 0 ignored."),
        "the example that fails to link fails, and the other passes\n{}",
        streams(&output)
    );
}

/// A Fix example written as a module that declares a type and a trait of one name fails, and the
/// error is reported at the two declarations in the comment.
#[test]
fn test_name_confliction_in_an_example_is_reported_at_the_declarations() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// # module DocTest;
// # type Piyo = struct { data : I64 };
// # trait a : Piyo {
// #     val : a;
// # }
// # main : IO ();
// # main = pure();
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:7 ... FAILED")
            && stderr.contains("is both a type and a trait")
            && stderr.contains("doc tests: 1 passed, 1 failed, 0 ignored."),
        "the example declaring `Piyo` twice fails, and the others pass\n{}",
        streams(&output)
    );
    assert!(
        stderr.contains("9 | // # type Piyo = struct { data : I64 };")
            && stderr.contains("10 | // # trait a : Piyo {"),
        "the error quotes the two declarations\n{}",
        streams(&output)
    );
}

/// A Fix example written as a module whose `main` is declared through a type alias of `IO ()`
/// passes in the program of the examples, as it does built alone, and the examples are built
/// together once.
#[test]
fn test_example_whose_main_is_declared_through_an_alias_of_io() {
    let lib = r#"module Lib;

DEPRECATED[old_double, "Call `double` in place of `old_double`."];
old_double : I64 -> I64;
old_double = |x| 2 * x;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// # module DocTest;
// # import Lib;
// # type Action = IO ();
// # main : Action;
// # main = assert_eq(|_|"", double(2), 4);
// ```
double : I64 -> I64;
double = |x| old_double(x);
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stderr.contains("doc tests: 2 passed, 0 failed, 0 ignored."),
        "both examples pass\n{}",
        streams(&output)
    );
    assert_eq!(
        stderr
            .matches("Call `double` in place of `old_double`.")
            .count(),
        1,
        "the examples are built together once\n{}",
        streams(&output)
    );
}

/// An example marked `ignore` is left out of the program of the Fix examples, and each example
/// after it runs as itself.
#[test]
fn test_examples_after_an_ignored_example_run_as_themselves() {
    let lib = r#"module Lib;

// ```fix ignore
// double(undefined_name)
// ```
//
// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// assert_eq(|_|"the third example fails", double(2), 5)
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:3 ... ignored")
            && stderr.contains("doc test lib.fix:7 ... ok")
            && stderr.contains("doc test lib.fix:11 ... FAILED")
            && stderr.contains("the third example fails")
            && stderr.contains("doc tests: 1 passed, 1 failed, 1 ignored."),
        "each example after the ignored one runs as itself\n{}",
        streams(&output)
    );
}

/// Where the program of the Fix examples holds one example and does not build, `fix test` tests
/// that example alone and prints no message that this is slower: testing it alone takes one build,
/// as the program does. An example marked `ignore` is not built, so it does not count.
#[test]
fn test_one_example_that_does_not_compile_is_tested_without_saying_it_is_slower() {
    let lib = r#"module Lib;

// ```fix
// let x : I64 = "a string";
// pure()
// ```
//
// ```fix ignore
// double(undefined_name)
// ```
double : I64 -> I64;
double = |x| 2 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("doc test lib.fix:3 ... FAILED")
            && stderr.contains("doc tests: 0 passed, 1 failed, 1 ignored.")
            && !stderr.contains("compiled one by one"),
        "the example fails, and `fix test` says nothing about building each example alone\n{}",
        streams(&output)
    );
}

/// Where the Fix examples are tested one by one, a warning of the sources is reported once, by the
/// build of the sources alone, and a warning in an example is reported by the build of that
/// example.
#[test]
fn test_examples_tested_one_by_one_report_each_warning_once() {
    let lib = r#"module Lib;

DEPRECATED[old_double, "Call `double` in place of `old_double`."];
old_double : I64 -> I64;
old_double = |x| 2 * x;

DEPRECATED[old_triple, "Call `triple` in place of `old_triple`."];
old_triple : I64 -> I64;
old_triple = |x| 3 * x;

// ```fix
// assert_eq(|_|"", double(21), 42)
// ```
//
// ```fix
// assert_eq(|_|"", old_triple(1), 3)
// ```
//
// ```fix
// # module DocTest;
// # import Lib;
// # value : I64 = double(1);
// ```
double : I64 -> I64;
double = |x| old_double(x);

triple : I64 -> I64;
triple = |x| 3 * x;
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("compiled one by one")
            && stderr.contains("doc tests: 2 passed, 1 failed, 0 ignored."),
        "the examples are tested one by one, and the one without `main` fails\n{}",
        streams(&output)
    );
    assert_eq!(
        stderr
            .matches("Call `double` in place of `old_double`.")
            .count(),
        1,
        "the warning of the sources is reported once\n{}",
        streams(&output)
    );
    assert_eq!(
        stderr
            .matches("Call `triple` in place of `old_triple`.")
            .count(),
        1,
        "the warning in an example is reported once\n{}",
        streams(&output)
    );
}
