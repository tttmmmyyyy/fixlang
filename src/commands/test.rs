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
    // The examples are compiled as modules whose names are reserved for them (see
    // `is_reserved_module_name`), so those names are free where an example is compiled.
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
/// to run, in a process of its own with its output collected (see `ExampleBuild::merged`). Where
/// that program fails to build, as it does when one example does not compile, the sources are built
/// without the examples. If they fail to build too, the error is the sources': it is returned, and
/// no example is reported. Otherwise each example is built and tested alone, which reports what is
/// wrong with each.
pub fn test_examples(
    config: &Configuration,
    examples: &[FixExample],
    mut report: impl FnMut(&FixExample, ExampleOutcome),
) -> Result<(), Errors> {
    let sources = examples
        .iter()
        .filter_map(|example| example.task.source().cloned())
        .collect::<Vec<_>>();
    let source_count = sources.len();
    if source_count > 0 {
        let mut merged_config = config.clone();
        merged_config.example_build = Some(ExampleBuild::merged(sources)?);
        let built = with_temporary_executable(merged_config, |merged_config, exec_path| {
            // The position in the program of the next example it holds.
            let mut position = 0;
            for example in examples {
                let outcome = match example.task.source() {
                    None => ExampleOutcome::Ignored,
                    Some(_) => {
                        position += 1;
                        outcome_in_program(merged_config, exec_path, position - 1, example)
                    }
                };
                report(example, outcome);
            }
            assert_eq!(
                position, source_count,
                "each example the program holds is reported once"
            );
            Ok(())
        });
        if built.is_ok() {
            return Ok(());
        }
        let mut sources_config = config.clone();
        sources_config.example_build = Some(ExampleBuild::without_examples()?);
        build_executable(sources_config)?;
        if source_count > 1 {
            eprintln!(
                "The {} Fix examples do not build together into one program, which happens when \
                 one of them does not compile. Each of them is now built alone, which takes {} \
                 builds instead of one and is slower.",
                source_count, source_count
            );
        }
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
