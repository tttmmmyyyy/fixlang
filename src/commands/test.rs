use crate::ast::name::FullName;
use crate::commands::run::{build_executable, run, run_command};
use crate::configuration::{BuildConfigType, Configuration};
use crate::constants::{PROJECT_FILE_PATH, TEST_FUNCTION_NAME, TEST_MODULE_NAME};
use crate::doc_test::{check_doc_test_name_is_free, collect_examples, ExampleTask, FixExample};
use crate::elaboration::load_source_files;
use crate::error::{panic_if_err, Errors};
use crate::metafiles::project_file::ProjectFile;
use crate::parse::sourcefile::SourceFile;
use colored::Colorize;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Output};

/// Which tests `fix test` runs, as its options select.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestSelection {
    /// `Test::test`, and then the Fix examples of the doc comments.
    All,
    /// `Test::test` alone, which `--no-doc` selects.
    TestFunction,
    /// The Fix examples of the doc comments alone, which `--doc` selects.
    DocTests,
}

/// Runs the tests `selection` names and exits the `fix` process.
///
/// `Test::test` runs with the terminal's streams attached. Each Fix example is then built and run
/// in a process of its own, whose output is collected and shown where the example fails, and the
/// examples that remain run after one fails. The process exits with a status other than 0 when a
/// test failed.
///
/// A program without Fix examples is tested as `Test::test` alone, which exits with the status
/// `Test::test` exits with, and reports the error of a program that does not define it.
pub fn test_command(mut config: Configuration, selection: TestSelection) {
    if selection == TestSelection::TestFunction {
        run_command(&config);
    }

    let program = panic_if_err(load_source_files(&config));
    panic_if_err(check_doc_test_name_is_free(&program));
    let examples = panic_if_err(collect_examples(&program, &panic_if_err(doc_test_files())));
    if selection == TestSelection::All && examples.is_empty() {
        run_command(&config);
    }

    // The tests that failed, each by the name it is reported under.
    let mut failures = vec![];
    let test_function = FullName::from_strs(&[TEST_MODULE_NAME], TEST_FUNCTION_NAME);
    if selection == TestSelection::All && program.global_values.contains_key(&test_function) {
        if let Some(failure) = run_failure(panic_if_err(run(config.clone(), true))) {
            eprintln!("{}", failure);
            failures.push(test_function.to_string());
        }
    }
    let test_function_failure_count = failures.len();

    // A Fix example is built without the arguments and the output path given for `Test::test`.
    config.run_program_args.clear();
    config.out_file_path = None;
    let mut passed = 0;
    let mut ignored = 0;
    for example in &examples {
        let location = example.location();
        match test_example(&config, example) {
            ExampleOutcome::Ignored => {
                eprintln!("doc test {} ... ignored", location);
                ignored += 1;
            }
            ExampleOutcome::Passed => {
                eprintln!("doc test {} ... {}", location, "ok".green());
                passed += 1;
            }
            ExampleOutcome::Failed(failure) => {
                eprintln!("doc test {} ... {}", location, "FAILED".red());
                eprintln!("{}", failure);
                failures.push(location);
            }
        }
    }

    eprintln!(
        "doc tests: {} passed, {} failed, {} ignored.",
        passed,
        failures.len() - test_function_failure_count,
        ignored
    );
    if failures.is_empty() {
        process::exit(0);
    }
    eprintln!("failures:");
    for failure in &failures {
        eprintln!("    {}", failure);
    }
    process::exit(1);
}

/// What became of a Fix example `fix test` tested.
pub enum ExampleOutcome {
    /// The example did what its task asks: it compiled, and where it was run, it exited with status
    /// 0.
    Passed,
    /// The example failed, for the reason given: the compile errors, or how the program ended and
    /// what it wrote to the standard error.
    Failed(String),
    /// The example is marked `ignore`, so nothing was done with it.
    Ignored,
}

/// Tests the Fix example `example` as its task asks: builds it under `config`, beside the sources
/// `config` names, and runs it in a process of its own with its output collected.
pub fn test_example(config: &Configuration, example: &FixExample) -> ExampleOutcome {
    let config_of = |source: &SourceFile| {
        let mut config = config.clone();
        config.doc_test_example = Some(source.clone());
        config
    };
    let failure = match &example.task {
        ExampleTask::Ignore => return ExampleOutcome::Ignored,
        ExampleTask::Compile(source) => build_executable(config_of(source))
            .err()
            .map(|errors| errors.to_string()),
        ExampleTask::Run(source) => match run(config_of(source), false) {
            Ok(output) => run_failure(output),
            Err(errors) => Some(errors.to_string()),
        },
    };
    match failure {
        None => ExampleOutcome::Passed,
        Some(failure) => ExampleOutcome::Failed(failure),
    }
}

/// The files whose doc comments `fix test` takes the Fix examples of: those the `build` section of
/// the project file in the working directory lists. A directory without a project file has none.
fn doc_test_files() -> Result<Vec<PathBuf>, Errors> {
    if !Path::new(PROJECT_FILE_PATH).exists() {
        return Ok(vec![]);
    }
    Ok(ProjectFile::read_root_file()?.get_files(BuildConfigType::Build))
}

/// What went wrong with the run of a built program whose result is `output`, including what it
/// wrote to the standard error where that was collected, or `None` where it exited with status 0.
fn run_failure(output: Result<Output, io::Error>) -> Option<String> {
    match output {
        Err(e) => Some(format!("Failed to run the program: {}", e)),
        Ok(output) if output.status.success() => None,
        Ok(output) => Some(format!(
            "The program ended with {}.\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )),
    }
}
