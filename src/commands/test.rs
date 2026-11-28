use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::commands::run::TemporaryExecutable;
use crate::configuration::{Configuration, UserSourceFiles};
use crate::constants::{TEST_FUNCTION_NAME, TEST_INDEX_ENV_VAR, TEST_MODULE_NAME};
use crate::doc_test::{collect_examples, ExampleTask, FixExample};
use crate::elaboration::load_source_files;
use crate::error::{panic_if_err, Errors};
use crate::misc::info_msg;
use crate::parse::sourcefile::{SourceFile, Span};
use crate::test_build::{check_doc_test_module_names_are_free, DispatchedTest, TestBuild};
use colored::Colorize;
use std::io::{self, Read};
use std::mem::take;
use std::path::Path;
use std::process::{self, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Mutex};
use std::thread;

/// Which kinds of test `fix test` runs, as the options `--doc` and `--no-doc` select.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestSelection {
    /// Every kind.
    All,
    /// `Test::test` and the values the `TEST[...]` pragmas name, which `--no-doc` selects.
    Declared,
    /// The Fix examples of the comments, which `--doc` selects.
    DocTests,
}

impl TestSelection {
    /// Whether the selection runs the tests of the kind `kind`.
    fn includes(self, kind: TestKind) -> bool {
        match self {
            TestSelection::All => true,
            TestSelection::Declared => kind == TestKind::Declared,
            TestSelection::DocTests => kind == TestKind::DocTest,
        }
    }
}

/// The options of `fix test` that choose the tests and how they run.
pub struct TestOptions {
    /// The kinds of test that run.
    pub selection: TestSelection,
    /// The texts the `--filter` options give. Where there is one, a test runs when its name
    /// contains one of them.
    pub filters: Vec<String>,
    /// How many tests run at once.
    pub threads: usize,
}

impl TestOptions {
    /// Whether the options select the test `test`.
    fn selects(&self, test: &TestCase) -> bool {
        self.selection.includes(test.kind)
            && (self.filters.is_empty()
                || self
                    .filters
                    .iter()
                    .any(|filter| test.name.contains(filter.as_str())))
    }
}

/// A kind of test, which the summary counts on a line of its own.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TestKind {
    /// `Test::test`, or a value a `TEST[...]` pragma names.
    Declared,
    /// A Fix example of a comment.
    DocTest,
}

impl TestKind {
    /// Every kind, in the order the summary counts them.
    const ALL: [TestKind; 2] = [TestKind::Declared, TestKind::DocTest];

    /// The words the progress line of a test of this kind begins with.
    fn line_head(self) -> &'static str {
        match self {
            TestKind::Declared => "test",
            TestKind::DocTest => "doc test",
        }
    }

    /// The words the summary counts the tests of this kind under.
    fn summary_head(self) -> &'static str {
        match self {
            TestKind::Declared => "tests",
            TestKind::DocTest => "doc tests",
        }
    }
}

/// A test `fix test` can run.
pub struct TestCase {
    /// The kind of the test, which decides how its progress line begins and which line of the
    /// summary counts it.
    kind: TestKind,
    /// The name the test is reported under, which `--filter` is matched against: the full name of
    /// the value a declared test runs, and the place of a Fix example (see `FixExample::location`).
    name: String,
    /// Where the test is declared, which the heading of its failure shows beside its name: the
    /// `TEST[...]` pragma, or the declaration of `Test::test`. A Fix example is named by its place
    /// and has none.
    declared_at: Option<String>,
    /// What `fix test` does with the test.
    task: TestTask,
}

/// What `fix test` does with a test.
enum TestTask {
    /// Runs the global value of type `IO ()` the sources declare as a test.
    RunValue(FullName),
    /// Compiles the Fix example whose source this is and runs it.
    RunExample(SourceFile),
    /// Compiles the Fix example whose source this is, which is what the mark `no_run` asks for.
    CompileExample(SourceFile),
    /// Nothing, which is what the mark `ignore` asks for.
    Ignore,
}

impl TestCase {
    /// The test of the Fix example `example`.
    pub fn of_example(example: &FixExample) -> Self {
        TestCase {
            kind: TestKind::DocTest,
            name: example.location(),
            declared_at: None,
            task: match &example.task {
                ExampleTask::Run(source) => TestTask::RunExample(source.clone()),
                ExampleTask::Compile(source) => TestTask::CompileExample(source.clone()),
                ExampleTask::Ignore => TestTask::Ignore,
            },
        }
    }

    /// Whether the test compiles a Fix example.
    fn compiles_an_example(&self) -> bool {
        matches!(
            self.task,
            TestTask::RunExample(_) | TestTask::CompileExample(_)
        )
    }

    /// The test that runs the global value `value`, which the source at `declared_at` declares as
    /// a test, and marks `ignore` where `ignore` is true.
    fn of_value(value: &FullName, declared_at: &Option<Span>, ignore: bool) -> Self {
        TestCase {
            kind: TestKind::Declared,
            name: value.to_string(),
            declared_at: declared_at.as_ref().map(Span::file_and_line),
            task: match ignore {
                true => TestTask::Ignore,
                false => TestTask::RunValue(value.clone()),
            },
        }
    }
}

/// What became of a test `fix test` tested.
pub enum TestOutcome {
    /// The test did what its task asks: it compiled, and where it was run, it exited with status 0.
    Passed,
    /// The test failed, for the reason given: the compile errors, or how the program ended and
    /// what it wrote.
    Failed(String),
    /// The test is marked `ignore`, so nothing was done with it.
    Ignored,
}

impl TestOutcome {
    /// The outcome of a test that failed for the reason `failure` gives, or passed where it gives
    /// none.
    fn of_failure(failure: Option<String>) -> Self {
        match failure {
            None => TestOutcome::Passed,
            Some(failure) => TestOutcome::Failed(failure),
        }
    }
}

/// Runs the tests `options` select and exits the `fix` process, with a status other than 0 when a
/// test failed.
///
/// The tests are those of the files the user writes (see `Configuration::root_source_files`):
/// `Test::test`, the values the `TEST[...]` pragmas name, and the Fix examples of the comments.
/// Each test runs in a process of its own, given the arguments `config.run_program_args` carries,
/// and what it writes is collected and shown where it fails. A progress line reports each test as
/// it ends, and the failures and the counts follow once every test has ended.
///
/// Where `config.out_file_path` names a path, the one test the options select is built alone, and
/// its executable is kept at that path.
pub fn test_command(mut config: Configuration, options: TestOptions) {
    let program = panic_if_err(load_source_files(&config));
    let tests = panic_if_err(collect_tests(&program, &config))
        .into_iter()
        .filter(|test| options.selects(test))
        .collect::<Vec<_>>();
    if tests.is_empty() {
        panic_if_err::<()>(Err(no_test_error(&options)));
    }
    // The examples are compiled as modules whose names are reserved for them (see
    // `is_reserved_module_name`), so those names are free where an example is compiled.
    if tests.iter().any(TestCase::compiles_an_example) {
        panic_if_err(check_doc_test_module_names_are_free(&program));
    }

    let args = take(&mut config.run_program_args);
    let mut report = Report::new(&tests);
    let result = match config.out_file_path.take() {
        Some(out_path) => {
            test_keeping_executable(config, &tests, &args, &out_path, |index, outcome| {
                report.add(index, outcome)
            })
        }
        None => run_tests(&config, &tests, &args, options.threads, |index, outcome| {
            report.add(index, outcome)
        }),
    };
    panic_if_err(result);
    let passed = report.finish(options.selection);
    process::exit(if passed { 0 } else { 1 });
}

/// The tests of the user's files of `config` (see `UserSourceFiles`), which `program` is loaded from
/// among other files, in the order
/// `fix test` shows their failures in: `Test::test`, the values the `TEST[...]` pragmas name, and
/// the Fix examples of the comments. The tests of each kind are ordered by the file and then by
/// where they stand in it.
fn collect_tests(program: &Program, config: &Configuration) -> Result<Vec<TestCase>, Errors> {
    let user_files = UserSourceFiles::of(config);
    // The parser gives each `TEST[...]` pragma and each type signature its span.
    let in_user_files = |span: &Option<Span>| {
        let span = span
            .as_ref()
            .expect("a pragma or a declaration of a source has a span");
        user_files.contains_span(span)
    };

    let mut statements = program
        .test_statements
        .iter()
        .filter(|statement| in_user_files(&statement.src))
        .collect::<Vec<_>>();
    statements.sort_by(|lhs, rhs| lhs.src.cmp(&rhs.src));

    let mut tests = vec![];
    let test_function = FullName::from_strs(&[TEST_MODULE_NAME], TEST_FUNCTION_NAME);
    if let Some(gv) = program.global_values.get(&test_function) {
        let named_by_pragma = statements
            .iter()
            .any(|statement| statement.value_name == test_function);
        if !named_by_pragma && in_user_files(&gv.decl_src) {
            tests.push(TestCase::of_value(&test_function, &gv.decl_src, false));
        }
    }
    for statement in statements {
        tests.push(TestCase::of_value(
            &statement.value_name,
            &statement.src,
            statement.ignore,
        ));
    }
    for example in collect_examples(program, &config.root_source_files)? {
        tests.push(TestCase::of_example(&example));
    }
    Ok(tests)
}

/// The number of tests `fix test` runs at once where `--test-threads` is not given: the number of
/// CPUs, or 1 where the system does not tell it.
pub fn default_test_threads() -> usize {
    thread::available_parallelism().map_or(1, |threads| threads.get())
}

/// The error `fix test` reports where `options` select no test.
fn no_test_error(options: &TestOptions) -> Errors {
    if options.filters.is_empty() {
        return Errors::from_msg(
            "There is no test to run.\n\
             HINT: A test is `Test::test`, a global value a `TEST[...]` pragma names, or a Fix \
             example in a comment, written in a file the `build` or `build.test` section of the \
             project file lists."
                .to_string(),
        );
    }
    let filters = options
        .filters
        .iter()
        .map(|filter| format!("`{}`", filter))
        .collect::<Vec<_>>()
        .join(", ");
    Errors::from_msg(format!(
        "No test to run has a name containing {}.\n\
         HINT: `--filter` matches a part of the name a test is reported under, such as \
         `Lib::parse_empty` or `lib.fix:12`.",
        filters
    ))
}

/// Tests `tests` under `config` and beside the sources `config` names, and hands `report` the index
/// in `tests` of each test with its outcome, as the outcome is known. The tests that run, run
/// `threads` at a time, each in a process of its own given the arguments `args`.
///
/// The tests are built together into one program, which is run once for each test to run (see
/// `TestBuild::dispatching`). Where that program fails to build and holds Fix examples, the sources
/// are built with the declared tests and without the examples. If they fail to build too, the error
/// is the sources': it is returned, and no test that would be built is reported. Otherwise each
/// example is built alone, which reports what is wrong with each, and the declared tests run from
/// the program of the sources. Where the program holds no example, the error of its build is
/// returned.
pub fn run_tests(
    config: &Configuration,
    tests: &[TestCase],
    args: &[String],
    threads: usize,
    mut report: impl FnMut(usize, TestOutcome),
) -> Result<(), Errors> {
    // Each test to build by its index, with whether it is run.
    let mut built = vec![];
    let mut ignored = vec![];
    for (index, test) in tests.iter().enumerate() {
        match &test.task {
            TestTask::RunValue(value) => {
                built.push((index, DispatchedTest::Value(value.clone()), true))
            }
            TestTask::RunExample(source) => {
                built.push((index, DispatchedTest::Example(source.clone()), true))
            }
            TestTask::CompileExample(source) => {
                built.push((index, DispatchedTest::Example(source.clone()), false))
            }
            TestTask::Ignore => ignored.push(index),
        }
    }
    let config_of = |build: TestBuild| {
        let mut config = config.clone();
        config.test_build = Some(build);
        config
    };
    let mut executables = vec![];
    let mut runs = vec![];
    // Builds the tests `tests` in one program, and adds the runs of those to run by it.
    let mut build_dispatching = |tests: Vec<(usize, DispatchedTest, bool)>| -> Result<(), Errors> {
        let build =
            TestBuild::dispatching(tests.iter().map(|(_, test, _)| test.clone()).collect())?;
        executables.push(TemporaryExecutable::build(config_of(build))?);
        for (dispatch_index, (index, _, to_run)) in tests.into_iter().enumerate() {
            if to_run {
                runs.push(Run {
                    test_index: index,
                    executable_index: executables.len() - 1,
                    dispatch_index: Some(dispatch_index),
                });
            } else {
                report(index, TestOutcome::Passed);
            }
        }
        Ok(())
    };

    let has_examples = built
        .iter()
        .any(|(_, test, _)| matches!(test, DispatchedTest::Example(_)));
    match build_dispatching(built.clone()) {
        Ok(()) => {}
        Err(errors) if !has_examples => return Err(errors),
        Err(_) => {
            let (values, examples): (Vec<_>, Vec<_>) = built
                .into_iter()
                .partition(|(_, test, _)| matches!(test, DispatchedTest::Value(_)));
            build_dispatching(values)?;
            if examples.len() > 1 {
                info_msg(
                    "One of the Fix examples (i.e., the code blocks marked `fix` in the comments) \
                     does not compile, so the examples will be compiled one by one. This may take \
                     longer than usual.",
                );
            }
            for (index, example, to_run) in examples {
                let DispatchedTest::Example(source) = example else {
                    unreachable!("the tests left after the values are the examples")
                };
                match TemporaryExecutable::build(config_of(TestBuild::of_example(source, false))) {
                    Err(errors) => report(index, TestOutcome::Failed(errors.to_string())),
                    Ok(_) if !to_run => report(index, TestOutcome::Passed),
                    Ok(executable) => {
                        executables.push(executable);
                        runs.push(Run {
                            test_index: index,
                            executable_index: executables.len() - 1,
                            dispatch_index: None,
                        });
                    }
                }
            }
        }
    }

    // The sources have built, so the ignored tests are reported beside the tests that run.
    for index in ignored {
        report(index, TestOutcome::Ignored);
    }
    run_in_parallel(&executables, &runs, args, threads, report);
    Ok(())
}

/// A run of a test by one of the executables built.
struct Run {
    /// The index of the test.
    test_index: usize,
    /// The index of the executable among those built.
    executable_index: usize,
    /// The index the dispatcher of the executable runs the test by, or `None` where the executable
    /// has no dispatcher and runs the test alone.
    dispatch_index: Option<usize>,
}

/// Runs each run of `runs` by the executables `executables`, `threads` at a time, giving each the
/// arguments `args`, and hands `report` the index of its test with its outcome, in the order the
/// runs end.
fn run_in_parallel(
    executables: &[TemporaryExecutable],
    runs: &[Run],
    args: &[String],
    threads: usize,
    mut report: impl FnMut(usize, TestOutcome),
) {
    assert!(threads > 0, "the tests run on at least one thread");
    // The commands are made here, so that the threads below share the commands alone.
    let commands = runs
        .iter()
        .map(|run| {
            let command = run_command(&executables[run.executable_index], run.dispatch_index, args)
                .map_err(|errors| errors.to_string());
            (run.test_index, command)
        })
        .collect::<Vec<_>>();
    let commands = Mutex::new(commands.into_iter());
    let (sender, receiver) = mpsc::channel();
    thread::scope(|scope| {
        for _ in 0..threads.min(runs.len()) {
            let sender = sender.clone();
            let commands = &commands;
            scope.spawn(move || loop {
                let Some((index, command)) = commands.lock().unwrap().next() else {
                    break;
                };
                let failure = match command {
                    Ok(command) => run_failure(command),
                    Err(failure) => Some(failure),
                };
                sender
                    .send((index, TestOutcome::of_failure(failure)))
                    .expect("the outcomes are received until every run has ended");
            });
        }
        // The loop below ends once every thread has dropped its sender.
        drop(sender);
        for (index, outcome) in receiver {
            report(index, outcome);
        }
    });
}

/// The command that runs the test the executable `executable` runs by the index `dispatch_index`,
/// or the one test it runs where it has no dispatcher, given the arguments `args`.
fn run_command(
    executable: &TemporaryExecutable,
    dispatch_index: Option<usize>,
    args: &[String],
) -> Result<Command, Errors> {
    let mut command = executable.config().program_run_command(executable.path())?;
    command.args(args);
    if let Some(index) = dispatch_index {
        command.env(TEST_INDEX_ENV_VAR, index.to_string());
    }
    Ok(command)
}

/// Runs `command` and tells what went wrong with the run, including what the program wrote, or
/// `None` where it exited with status 0.
fn run_failure(command: Command) -> Option<String> {
    match run_collecting_output(command) {
        Err(e) => Some(format!("Failed to run the program: {}", e)),
        Ok((status, _)) if status.success() => None,
        Ok((status, output)) => Some(format!(
            "The program ended with {}.\n{}",
            status,
            String::from_utf8_lossy(&output)
        )),
    }
}

/// Runs `command` with no standard input, and returns how it ended and what it wrote to its
/// standard output and its standard error. The two are written into one pipe, so what they hold
/// stands in the order the program wrote it.
fn run_collecting_output(mut command: Command) -> io::Result<(ExitStatus, Vec<u8>)> {
    let (mut reader, writer) = io::pipe()?;
    command
        .stdin(Stdio::null())
        .stdout(writer.try_clone()?)
        .stderr(writer);
    let mut child = command.spawn()?;
    // The command holds the write ends it handed the program, and dropping it closes them, so that
    // the read below ends when the program has closed its own.
    drop(command);
    let mut output = vec![];
    reader.read_to_end(&mut output)?;
    Ok((child.wait()?, output))
}

/// Tests the one test of `tests` under `config`, built alone, and keeps the executable it was built
/// into at `out_path`. `report` is handed the index of the test with its outcome. A Fix example
/// that fails to compile is reported as a failed test; a declared test whose sources fail to
/// compile returns the compile error.
fn test_keeping_executable(
    mut config: Configuration,
    tests: &[TestCase],
    args: &[String],
    out_path: &Path,
    mut report: impl FnMut(usize, TestOutcome),
) -> Result<(), Errors> {
    let [test] = tests else {
        return Err(Errors::from_msg(format!(
            "`-o` keeps the executable of one test, and {} tests are selected.\n\
             HINT: Select one of them with `--filter`, as in `fix test --filter {} -o {}`.",
            tests.len(),
            tests[0].name,
            out_path.display()
        )));
    };
    config.test_build = Some(match &test.task {
        TestTask::RunValue(value) => TestBuild::of_value(value.clone()),
        TestTask::RunExample(source) | TestTask::CompileExample(source) => {
            TestBuild::of_example(source.clone(), true)
        }
        TestTask::Ignore => {
            return Err(Errors::from_msg(format!(
                "`-o` keeps the executable of the test it builds, and the test `{}` is marked \
                 `ignore`, so it is not built.",
                test.name
            )))
        }
    });
    let executable = match TemporaryExecutable::build(config) {
        Ok(executable) => executable,
        Err(errors) if matches!(test.task, TestTask::RunValue(_)) => return Err(errors),
        Err(errors) => {
            report(0, TestOutcome::Failed(errors.to_string()));
            return Ok(());
        }
    };
    let outcome = match test.task {
        TestTask::CompileExample(_) => TestOutcome::Passed,
        _ => TestOutcome::of_failure(match run_command(&executable, None, args) {
            Ok(command) => run_failure(command),
            Err(errors) => Some(errors.to_string()),
        }),
    };
    executable.keep_at(out_path);
    report(0, outcome);
    Ok(())
}

/// The outcomes of the tests `fix test` runs, which it reports as they are known and sums up once
/// every test has ended.
struct Report<'a> {
    /// The tests, in the order their failures are shown in.
    tests: &'a [TestCase],
    /// The outcome of each test of `tests`, at its index there, once it is known.
    outcomes: Vec<Option<TestOutcome>>,
}

/// How many tests passed, failed and were ignored.
#[derive(Default)]
struct Counts {
    passed: usize,
    failed: usize,
    ignored: usize,
}

impl Counts {
    /// The counts of `outcomes`.
    fn of<'b>(outcomes: impl Iterator<Item = &'b TestOutcome>) -> Self {
        let mut counts = Counts::default();
        for outcome in outcomes {
            match outcome {
                TestOutcome::Passed => counts.passed += 1,
                TestOutcome::Failed(_) => counts.failed += 1,
                TestOutcome::Ignored => counts.ignored += 1,
            }
        }
        counts
    }

    /// The counts as the summary writes them.
    ///
    /// # Examples
    /// `3 passed, 1 failed, 0 ignored`
    fn to_string(&self) -> String {
        format!(
            "{} passed, {} failed, {} ignored",
            self.passed, self.failed, self.ignored
        )
    }
}

impl<'a> Report<'a> {
    /// The report of `tests`, none of whose outcomes is known yet.
    fn new(tests: &'a [TestCase]) -> Self {
        Report {
            tests,
            outcomes: tests.iter().map(|_| None).collect(),
        }
    }

    /// Records `outcome` as the outcome of the test at `index` and writes its progress line.
    ///
    /// # Examples
    /// `test Lib::parse_empty ... ok`, and `doc test lib.fix:9 ... FAILED`
    fn add(&mut self, index: usize, outcome: TestOutcome) {
        let test = &self.tests[index];
        let status = match &outcome {
            TestOutcome::Passed => "ok".green(),
            TestOutcome::Failed(_) => "FAILED".red(),
            TestOutcome::Ignored => "ignored".normal(),
        };
        eprintln!("{} {} ... {}", test.kind.line_head(), test.name, status);
        assert!(
            self.outcomes[index].is_none(),
            "the test {} is reported once",
            test.name
        );
        self.outcomes[index] = Some(outcome);
    }

    /// Writes what each failed test wrote, the names of the failed tests, and the counts of the
    /// tests: all of them, and those of each kind `selection` runs. Returns whether no test failed.
    fn finish(self, selection: TestSelection) -> bool {
        let outcomes = self
            .tests
            .iter()
            .zip(&self.outcomes)
            .map(|(test, outcome)| {
                let outcome = outcome
                    .as_ref()
                    .unwrap_or_else(|| panic!("the test {} is reported", test.name));
                (test, outcome)
            })
            .collect::<Vec<_>>();
        let failures = outcomes
            .iter()
            .filter_map(|(test, outcome)| match outcome {
                TestOutcome::Failed(failure) => Some((*test, failure)),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (test, failure) in &failures {
            eprintln!();
            match &test.declared_at {
                Some(declared_at) => eprintln!("---- {} ({}) ----", test.name, declared_at),
                None => eprintln!("---- {} ----", test.name),
            }
            eprintln!("{}", failure.trim_end());
        }
        if !failures.is_empty() {
            eprintln!();
            eprintln!("failures:");
            for (test, _) in &failures {
                eprintln!("    {}", test.name);
            }
        }
        eprintln!();
        eprintln!(
            "test result: {}.",
            Counts::of(outcomes.iter().map(|(_, outcome)| *outcome)).to_string()
        );
        for kind in TestKind::ALL {
            if selection.includes(kind) {
                let counts = Counts::of(
                    outcomes
                        .iter()
                        .filter(|(test, _)| test.kind == kind)
                        .map(|(_, outcome)| *outcome),
                );
                eprintln!("    {}: {}.", kind.summary_head(), counts.to_string());
            }
        }
        failures.is_empty()
    }
}
