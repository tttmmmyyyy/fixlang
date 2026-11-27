//! Tests of how `fix test` finds, runs and reports its tests: the values the `TEST[...]` pragma
//! declares, `Test::test`, and the Fix examples of the comments.

use crate::tests::test_util::{fix_command, fix_test, project_dir, streams};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// A module `Lib` declaring three tests: one that passes, one in a namespace that fails after
/// writing to the standard output and to the standard error, and one marked `ignore` that would
/// fail if it ran.
const LIB_WITH_TESTS: &str = r#"module Lib;

double : I64 -> I64;
double = |x| 2 * x;

double_is_even : IO ();
double_is_even = assert_eq(|_|"", double(3) % 2, 0);
TEST[double_is_even];

namespace Checks {
    double_of_two : IO ();
    double_of_two = (
        println("checking double(2)");;
        eprintln("about to assert");;
        assert_eq(|_|"double(2) is 4", double(2), 5)
    );
    TEST[double_of_two];

    never_run : IO ();
    never_run = undefined("an ignored test runs");
    TEST[never_run, ignore];
}
"#;

/// Each value a `TEST[...]` pragma names runs as a test of its own and is reported under its full
/// name: a failing test leaves the others running, what it wrote to its two streams is shown under
/// a heading naming where its pragma stands, after every test has ended, and a test marked `ignore`
/// is reported as ignored without running.
#[test]
fn test_declared_tests_run_and_are_reported_one_by_one() {
    let dir = project_dir(&[("lib.fix", LIB_WITH_TESTS)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "fix test fails\n{}",
        streams(&output)
    );
    for line in [
        "test Lib::double_is_even ... ok",
        "test Lib::Checks::double_of_two ... FAILED",
        "test Lib::Checks::never_run ... ignored",
        "---- Lib::Checks::double_of_two (lib.fix:17) ----",
        "checking double(2)\n",
        "about to assert\n",
        "double(2) is 4",
        "failures:\n    Lib::Checks::double_of_two\n",
        "test result: 1 passed, 1 failed, 1 ignored.\n    tests: 1 passed, 1 failed, 1 ignored.\n    \
         doc tests: 0 passed, 0 failed, 0 ignored.\n",
    ] {
        assert!(
            stderr.contains(line),
            "`fix test` reports `{}`\n{}",
            line,
            streams(&output)
        );
    }
    assert!(
        !stderr.contains("an ignored test runs"),
        "the test marked `ignore` does not run\n{}",
        streams(&output)
    );
    let last_progress_line = stderr
        .rfind(" ... ")
        .expect("`fix test` reports the tests");
    assert!(
        stderr.find("---- ").unwrap() > last_progress_line,
        "the failure is shown after every test has ended\n{}",
        streams(&output)
    );
}

/// `--filter` runs the tests whose names contain its text, or any of its texts where it is given
/// more than once, and a filter that no test matches is an error.
#[test]
fn test_filter_selects_the_tests_by_their_names() {
    let dir = project_dir(&[("lib.fix", LIB_WITH_TESTS)], &[]);
    let output = fix_test(&dir, &["--filter", "double_is", "--filter", "never"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.contains("test Lib::double_is_even ... ok")
            && stderr.contains("test Lib::Checks::never_run ... ignored")
            && !stderr.contains("double_of_two"),
        "the tests matching one of the filters run, and the other test does not\n{}",
        streams(&output)
    );

    let output = fix_test(&dir, &["--filter", "nothing_matches"]);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("No test to run has a name containing `nothing_matches`."),
        "a filter no test matches is an error\n{}",
        streams(&output)
    );
}

/// `--doc` leaves the declared tests out and `--no-doc` leaves the Fix examples out, and the counts
/// of the summary cover the kinds that run.
#[test]
fn test_doc_options_select_the_kinds_of_test() {
    let lib = r#"module Lib;

// ```fix
// assert_eq(|_|"", value, 1)
// ```
value : I64;
value = 1;

value_is_one : IO ();
value_is_one = assert_eq(|_|"", value, 1);
TEST[value_is_one];
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);

    let output = fix_test(&dir, &["--doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.contains("doc test lib.fix:3 ... ok")
            && !stderr.contains("value_is_one")
            && stderr.contains("    doc tests: 1 passed")
            && !stderr.contains("    tests:"),
        "`--doc` runs the example alone and counts the examples alone\n{}",
        streams(&output)
    );

    let output = fix_test(&dir, &["--no-doc"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.contains("test Lib::value_is_one ... ok")
            && !stderr.contains("doc test")
            && stderr.contains("    tests: 1 passed")
            && !stderr.contains("doc tests:"),
        "`--no-doc` runs the declared test alone and counts the declared tests alone\n{}",
        streams(&output)
    );
}

/// `Test::test` is a test without a `TEST[...]` pragma naming it, and a pragma naming it does not
/// make it run twice.
#[test]
fn test_test_function_named_by_a_pragma_runs_once() {
    let test = "module Test;\ntest : IO ();\ntest = pure();\nTEST[test];\n";
    let dir = project_dir(
        &[("lib.fix", "module Lib;\nvalue : I64 = 1;\n")],
        &[("test.fix", test)],
    );
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.matches("test Test::test ... ok").count() == 1
            && stderr.contains("test result: 1 passed, 0 failed, 0 ignored."),
        "`Test::test` runs once\n{}",
        streams(&output)
    );
}

/// The arguments after `--` reach every test: a declared test and a Fix example.
#[test]
fn test_arguments_reach_every_test() {
    let lib = r#"module Lib;

// ```fix
// let args = *IO::get_args;
// assert(|_|"the example gets the argument", args.find_by(|arg| arg == "hello").is_some)
// ```
gets_the_argument : IO ();
gets_the_argument = (
    let args = *IO::get_args;
    assert(|_|"the test gets the argument", args.find_by(|arg| arg == "hello").is_some)
);
TEST[gets_the_argument];
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--", "hello"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.contains("test Lib::gets_the_argument ... ok")
            && stderr.contains("doc test lib.fix:3 ... ok"),
        "both tests get the argument\n{}",
        streams(&output)
    );
}

/// Tests run at once: each of two tests waits for the file the other writes, which only a run of the
/// two at the same time lets both find.
#[test]
fn test_tests_run_at_once() {
    let lib = r#"module Lib;

// Writes the file `mine`, and waits for the file `theirs` to be written.
meet : Path -> Path -> IO ();
meet = |mine, theirs| (
    write_file_string(mine, "").try(|err| undefined(err));;
    wait_for(theirs, 3000)
);

// Waits for the file `path` to be written, looking for it `tries` times, 10 milliseconds apart.
wait_for : Path -> I64 -> IO ();
wait_for = |path, tries| (
    if tries == 0 { undefined("the other test does not run at the same time") };
    let found = *read_file_string(path).to_result;
    if found.is_ok { pure() };
    let _ = *FFI_CALL_IO[CInt usleep(CUnsignedInt), 10000.to_CUnsignedInt];
    wait_for(path, tries - 1)
);

first : IO ();
first = meet("first.txt", "second.txt");
TEST[first];

second : IO ();
second = meet("second.txt", "first.txt");
TEST[second];
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &["--test-threads", "2"]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("test result: 2 passed, 0 failed, 0 ignored."),
        "the two tests meet\n{}",
        streams(&output)
    );
}

/// `-o` builds the one test the options select, runs it, and keeps its executable, which runs the
/// test. It is an error where the options select several tests, or one marked `ignore`.
#[test]
fn test_output_option_keeps_the_executable_of_one_test() {
    let dir = project_dir(&[("lib.fix", LIB_WITH_TESTS)], &[]);
    let kept = dir.path().join("kept.out");
    let kept_arg = kept.to_str().unwrap();

    let output = fix_test(&dir, &["--filter", "double_is_even", "-o", kept_arg]);
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("test Lib::double_is_even ... ok"),
        "the selected test runs and passes\n{}",
        streams(&output)
    );
    let status = Command::new(&kept)
        .status()
        .expect("the executable is kept");
    assert!(status.success(), "the kept executable runs the test");

    fs::remove_file(&kept).expect("the executable is removable");
    let output = fix_test(&dir, &["-o", kept_arg]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && stderr.contains("`-o` keeps the executable of one test, and 3 tests are selected.")
            && stderr.contains("HINT: Select one of them with `--filter`")
            && !stderr.contains(" ... "),
        "`-o` with several tests is an error, and no test runs\n{}",
        streams(&output)
    );

    let output = fix_test(&dir, &["--filter", "never_run", "-o", kept_arg]);
    assert!(
        !output.status.success()
            && String::from_utf8_lossy(&output.stderr)
                .contains("the test `Lib::Checks::never_run` is marked `ignore`"),
        "`-o` with an ignored test is an error\n{}",
        streams(&output)
    );
    assert!(!kept.exists(), "no executable is kept where no test is built");
}

/// A compile error in a value a `TEST[...]` pragma names is an error of the sources: it is reported
/// once, and no test runs.
#[test]
fn test_compile_error_of_a_declared_test_stops_every_test() {
    let lib = r#"module Lib;

passes : IO ();
passes = pure();
TEST[passes];

broken : IO ();
broken = assert_eq(|_|"", 1, "one");
TEST[broken];
"#;
    let dir = project_dir(&[("lib.fix", lib)], &[]);
    let output = fix_test(&dir, &[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success()
            && stderr
                .matches("Expected `Std::I64`, found `Std::String`.")
                .count()
                == 1
            && !stderr.contains(" ... "),
        "the error is reported once, and no test runs\n{}",
        streams(&output)
    );
}

/// The tests a dependency declares are not run.
#[test]
fn test_tests_of_a_dependency_are_not_run() {
    let root = TempDir::new().expect("Failed to create temp directory");
    let dep_dir = root.path().join("dep");
    let main_dir = root.path().join("main");
    fs::create_dir_all(&dep_dir).unwrap();
    fs::create_dir_all(&main_dir).unwrap();
    fs::write(
        dep_dir.join("fixproj.toml"),
        "[general]\nname = \"dep\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"dep.fix\"]\n",
    )
    .unwrap();
    fs::write(
        dep_dir.join("dep.fix"),
        "module Dep;\n\nfails : IO ();\nfails = undefined(\"the test of the dependency runs\");\n\
         TEST[fails];\n",
    )
    .unwrap();
    fs::write(
        main_dir.join("fixproj.toml"),
        "[general]\nname = \"main\"\nversion = \"0.1.0\"\n\n[build]\nfiles = [\"main.fix\"]\n\n\
         [[dependencies]]\nname = \"dep\"\npath = \"../dep\"\n",
    )
    .unwrap();
    fs::write(
        main_dir.join("main.fix"),
        "module Main;\nimport Dep;\n\npasses : IO ();\npasses = pure();\nTEST[passes];\n",
    )
    .unwrap();
    let output = fix_command()
        .arg("test")
        .current_dir(&main_dir)
        .output()
        .expect("Failed to execute fix test");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success()
            && stderr.contains("test Main::passes ... ok")
            && !stderr.contains("Dep::fails"),
        "the test of the project runs, and that of the dependency does not\n{}",
        streams(&output)
    );
}

/// A `TEST[...]` pragma is checked wherever the program is compiled, and `fix build` and `fix test`
/// report its errors at the pragma: it names a global value declared under where it is written, by
/// a relative path, at type `IO ()`, no other pragma names that value, and its mark is `ignore`.
#[test]
fn test_malformed_test_pragmas_are_rejected() {
    for (pragma, error) in [
        (
            "value : I64 = 1;\nTEST[value];",
            "The value `Main::value` should have type `Std::IO ()`.",
        ),
        (
            "TEST[missing];",
            "`TEST` names `Main::missing`, which is not a global value declared in `Main`.",
        ),
        (
            "ok : IO () = pure();\nTEST[ok];\nTEST[ok];",
            "Multiple `TEST` pragmas name the value `Main::ok`.",
        ),
        (
            "ok : IO () = pure();\nTEST[::Main::ok];",
            "`TEST` cannot take an absolute path.",
        ),
        (
            "ok : IO () = pure();\nTEST[ok, skip];",
            "`skip` is not a mark of `TEST`.",
        ),
    ] {
        let main = format!("module Main;\n{}\nmain : IO () = pure();\n", pragma);
        let dir = project_dir(&[("main.fix", &main)], &[]);
        for subcommand in ["build", "test"] {
            let output = fix_command()
                .arg(subcommand)
                .current_dir(dir.path())
                .output()
                .expect("Failed to execute fix");
            assert!(
                !output.status.success()
                    && String::from_utf8_lossy(&output.stderr).contains(error),
                "`fix {}` rejects\n{}\nwith `{}`\n{}",
                subcommand,
                pragma,
                error,
                streams(&output)
            );
        }
    }
}
