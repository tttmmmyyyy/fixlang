use crate::ast::name::FullName;
use crate::commands::run::{build_executable, run, run_command, with_temporary_executable};
use crate::configuration::{BuildConfigType, Configuration};
use crate::constants::{
    DOC_TEST_EXAMPLE_ENV_VAR, PROJECT_FILE_PATH, TEST_FUNCTION_NAME, TEST_MODULE_NAME,
};
use crate::doc_test::{
    check_doc_test_module_names_are_free, collect_examples, ExampleBuild, ExampleTask, FixExample,
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
/// `test_examples`: each runs in a process of its own, whose output is collected and shown where
/// the example fails, and the examples that remain run after one fails. The process exits with a
/// status other than 0 when a test failed.
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
        .any(|example| example.task.source().is_some())
    {
        panic_if_err(check_doc_test_module_names_are_free(&program));
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
    let result = test_examples(&config, &examples, |example, outcome| {
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
    panic_if_err(result);

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
/// `config` names, and hands `report` each example with its outcome, in the order of `examples`.
///
/// The examples to compile are built together into one program, which is run once for each example
/// to run, in a process of its own with its output collected (see `ExampleBuild::merged`). An
/// example behaves in that program as it does in a program of its own, except an example that does
/// not compile and two examples that cannot share a program, such as two that export functions
/// under one C name. So where the program fails to build, the examples the errors lie in are taken
/// out of it and tested alone, and the rest are built together again.
///
/// Where an error lies in no example, the sources are built without the examples. If they fail to
/// build, the error is theirs: it is returned, and no example is reported. If they build, one of the
/// examples caused the error but nothing tells which, as when an example calls a C function that
/// nothing defines and the link fails. Each example left is then tested alone.
pub fn test_examples(
    config: &Configuration,
    examples: &[FixExample],
    mut report: impl FnMut(&FixExample, ExampleOutcome),
) -> Result<(), Errors> {
    // The indices in `examples` of the examples built together.
    let mut together = (0..examples.len())
        .filter(|index| examples[*index].task.source().is_some())
        .collect::<Vec<_>>();
    while !together.is_empty() {
        let sources = together
            .iter()
            .map(|index| examples[*index].task.source().unwrap().clone())
            .collect::<Vec<_>>();
        let example_build = panic_if_err(ExampleBuild::merged(sources));
        let mut merged_config = config.clone();
        merged_config.example_build = Some(example_build.clone());
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
            return Ok(());
        };
        let Some(blamed) = examples_errors_lie_in(&errors, &example_build) else {
            let mut sources_config = config.clone();
            sources_config.example_build = Some(panic_if_err(ExampleBuild::merged(vec![])));
            build_executable(sources_config)?;
            break;
        };
        assert!(
            !blamed.is_empty(),
            "a build that failed reports an error\n{}",
            errors.to_string()
        );
        together = together
            .into_iter()
            .enumerate()
            .filter(|(position, _)| !blamed.contains(position))
            .map(|(_, index)| index)
            .collect();
    }
    for example in examples {
        report(example, test_example(config, example));
    }
    Ok(())
}

/// The outcome of `example`, which is at index `position` in the program built at `exec_path` under
/// `config`: an example to compile passed when the program was built, and an example to run is run.
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

/// The indices in `example_build` of the examples the errors of `errors` belong to, or `None` where
/// one of them belongs to none. An error belongs to each example one of its locations belongs to
/// (see `ExampleBuild::example_at`).
fn examples_errors_lie_in(errors: &Errors, example_build: &ExampleBuild) -> Option<Set<usize>> {
    let mut blamed = Set::default();
    for error in errors.errors() {
        let lying_in = error
            .srcs
            .iter()
            .filter_map(|(_, span)| example_build.example_at(span))
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
        config.example_build = Some(ExampleBuild::single(source.clone()));
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
