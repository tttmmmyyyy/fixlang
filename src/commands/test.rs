use crate::ast::name::FullName;
use crate::commands::run::{build_executable, run, run_command, with_temporary_executable};
use crate::configuration::{BuildConfigType, Configuration};
use crate::constants::{
    DOC_TEST_EXAMPLE_ENV_VAR, PROJECT_FILE_PATH, TEST_FUNCTION_NAME, TEST_MODULE_NAME,
};
use crate::doc_test::{
    check_doc_test_module_name_is_free, collect_examples, ExampleBuild, ExampleTask, FixExample,
};
use crate::elaboration::load_source_files;
use crate::error::{panic_if_err, Errors};
use crate::metafiles::project_file::ProjectFile;
use crate::misc::Set;
use crate::parse::sourcefile::SourceFile;
use colored::Colorize;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Output};

/// Which tests `fix test` runs, as its options select.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestSelection {
    /// `Test::test`, and then the Fix examples of the comments.
    All,
    /// `Test::test` alone, which `--no-doc` selects.
    TestFunction,
    /// The Fix examples of the comments alone, which `--doc` selects.
    DocTests,
}

/// Runs the tests `selection` names and exits the `fix` process.
///
/// `Test::test` runs with the terminal's streams attached. The Fix examples are then tested by
/// `test_examples`: each runs in a process of its own, whose output is collected and shown where the
/// example fails, and the examples that remain run after one fails. The process exits with a status
/// other than 0 when a test failed.
///
/// A program without Fix examples is tested as `Test::test` alone, which exits with the status
/// `Test::test` exits with, and reports the error of a program that does not define it.
pub fn test_command(mut config: Configuration, selection: TestSelection) {
    if selection == TestSelection::TestFunction {
        run_command(&config);
    }

    let program = panic_if_err(load_source_files(&config));
    let examples = panic_if_err(collect_examples(&program, &panic_if_err(doc_test_files())));
    // The module name `DocTest` is the one each example is compiled as, so it is free where an
    // example is compiled.
    if examples
        .iter()
        .any(|example| !matches!(example.task, ExampleTask::Ignore))
    {
        panic_if_err(check_doc_test_module_name_is_free(&program));
    }
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
    test_examples(&config, &examples, |example, outcome| {
        let location = example.location();
        match outcome {
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
    });

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

impl ExampleOutcome {
    /// The outcome of an example whose test failed for the reason `failure` gives, or passed where
    /// it gives none.
    fn of_failure(failure: Option<String>) -> Self {
        match failure {
            None => ExampleOutcome::Passed,
            Some(failure) => ExampleOutcome::Failed(failure),
        }
    }
}

/// Tests the Fix examples `examples` as their tasks ask, under `config` and beside the sources
/// `config` names, and hands `report` each example with what became of it, in the order of
/// `examples`.
///
/// The examples to compile are built together into one program, which is run once for each example
/// to run, in a process of its own with its output collected (see `ExampleBuild::merged`). What an
/// example does in that program is what it does in a program of its own, except where two examples
/// cannot share one program, as two that export functions under one C name cannot, or where one
/// example does not compile. So where the program fails to build, the examples the errors lie in
/// are taken out of it and tested alone, and the rest are built together again. Where an error lies in no example, as an error in
/// the sources does, each example is tested alone, which reports that error for each of them.
pub fn test_examples(
    config: &Configuration,
    examples: &[FixExample],
    mut report: impl FnMut(&FixExample, ExampleOutcome),
) {
    // The indices in `examples` of the examples built together.
    let mut together = (0..examples.len())
        .filter(|index| examples[*index].task.source().is_some())
        .collect::<Vec<_>>();
    while !together.is_empty() {
        let sources = together
            .iter()
            .map(|index| examples[*index].task.source().unwrap().clone())
            .collect::<Vec<_>>();
        let mut merged_config = config.clone();
        merged_config.doc_tests = Some(panic_if_err(ExampleBuild::merged(sources.clone())));
        let built = with_temporary_executable(merged_config, |merged_config, exec_path| {
            for (index, example) in examples.iter().enumerate() {
                // `together` is in ascending order, as `examples` is.
                let outcome = match together.binary_search(&index).ok() {
                    Some(position) => {
                        outcome_in_program(merged_config, exec_path, position, example)
                    }
                    None => test_example(config, example),
                };
                report(example, outcome);
            }
            Ok(())
        });
        let Err(errors) = built else {
            return;
        };
        match examples_errors_lie_in(&errors, &sources) {
            Some(blamed) if !blamed.is_empty() => {
                together = together
                    .into_iter()
                    .enumerate()
                    .filter(|(position, _)| !blamed.contains(position))
                    .map(|(_, index)| index)
                    .collect();
            }
            _ => break,
        }
    }
    for example in examples {
        report(example, test_example(config, example));
    }
}

/// What became of the example `example`, which the program built together at `exec_path` under
/// `config` holds at index `position`: an example to compile passed as the program was built, and
/// an example to run is run.
fn outcome_in_program(
    config: &Configuration,
    exec_path: &str,
    position: usize,
    example: &FixExample,
) -> ExampleOutcome {
    let failure = match &example.task {
        ExampleTask::Ignore => unreachable!(
            "an ignored example at {} is built into no program",
            example.location()
        ),
        ExampleTask::Compile(_) => None,
        ExampleTask::Run(_) => match config.program_run_command(exec_path) {
            Ok(mut command) => {
                command.env(DOC_TEST_EXAMPLE_ENV_VAR, position.to_string());
                run_failure(command.output())
            }
            Err(errors) => Some(errors.to_string()),
        },
    };
    ExampleOutcome::of_failure(failure)
}

/// The indices in `sources` of the sources the errors of `errors` lie in, or `None` where one of
/// them lies in none of `sources`. An error lies in each source one of its locations is in.
fn examples_errors_lie_in(errors: &Errors, sources: &[SourceFile]) -> Option<Set<usize>> {
    let mut blamed = Set::default();
    for error in errors.errors() {
        let lying_in = (0..sources.len())
            .filter(|index| {
                error
                    .srcs
                    .iter()
                    .any(|(_, span)| span.input == sources[*index])
            })
            .collect::<Vec<_>>();
        if lying_in.is_empty() {
            return None;
        }
        blamed.extend(lying_in);
    }
    Some(blamed)
}

/// Tests the Fix example `example` as its task asks: builds it alone under `config`, beside the
/// sources `config` names, and runs it in a process of its own with its output collected.
fn test_example(config: &Configuration, example: &FixExample) -> ExampleOutcome {
    let config_of = |source: &SourceFile| {
        let mut config = config.clone();
        config.doc_tests = Some(ExampleBuild::single(source.clone()));
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
    ExampleOutcome::of_failure(failure)
}

/// The files whose comments `fix test` takes the Fix examples of: those the `build` and the
/// `build.test` sections of the project file in the working directory list. A directory without a
/// project file has none.
fn doc_test_files() -> Result<Vec<PathBuf>, Errors> {
    if !Path::new(PROJECT_FILE_PATH).exists() {
        return Ok(vec![]);
    }
    Ok(ProjectFile::read_root_file()?.get_files(BuildConfigType::Test))
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
