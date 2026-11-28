//! The programs `fix test` builds to run its tests.
//!
//! `fix test` builds the tests of a program into one program where it can: the values the
//! `TEST[...]` pragmas name, `Test::test`, and the Fix examples of the comments. The program holds a
//! dispatcher, whose `main` runs the test whose index an environment variable gives, so that each
//! test runs in a process of its own.

use crate::{
    ast::{
        name::{FullName, Name},
        program::Program,
    },
    constants::{DOC_TEST_MODULE_NAME, MAIN_FUNCTION_NAME, TEST_INDEX_ENV_VAR},
    error::{Error, Errors},
    hash::md5_hex,
    misc::save_temporary_source,
    parse::{parser::ModuleRenaming, sourcefile::SourceFile},
};
use std::iter;

/// What a build of `fix test` compiles beside the sources of the program, and the value the program
/// runs.
#[derive(Clone)]
pub struct TestBuild {
    /// The Fix examples, each with the name of the module it is compiled as.
    examples: Vec<ExampleModule>,
    /// What the entry point of the program runs.
    entry: TestEntry,
    /// Whether the build reports the warnings that lie outside its examples.
    reports_source_warnings: bool,
}

/// What the entry point of a program `fix test` builds runs.
#[derive(Clone)]
enum TestEntry {
    /// One test, the value of type `IO ()` that is the entry point of the program.
    Alone(FullName),
    /// Several tests, one in each run, which the module `DocTest.Dispatcher` chooses.
    Dispatched {
        /// The values of type `IO ()` the dispatcher runs. The values the sources declare as tests
        /// come first, in the order they were given, and the `main` of each example follows them,
        /// in the order of `TestBuild::examples`.
        values: Vec<FullName>,
        /// The source of the module whose `main` runs the value whose index in `values` the
        /// environment variable `TEST_INDEX_ENV_VAR` gives.
        dispatcher: SourceFile,
    },
}

/// A Fix example as a build compiles it.
#[derive(Clone)]
struct ExampleModule {
    /// The source assembled from the example, which declares the module `DocTest`.
    source: SourceFile,
    /// The name the module is compiled as.
    name: Name,
}

impl ExampleModule {
    /// The `main` the example defines.
    fn main(&self) -> FullName {
        FullName::from_strs(&[&self.name], MAIN_FUNCTION_NAME)
    }
}

/// The name of the module whose `main` runs one of the tests a build holds.
const DISPATCHER_MODULE_NAME: &str = "DocTest.Dispatcher";

impl TestBuild {
    /// The build of the tests `tests` the sources declare and of the Fix examples whose sources are
    /// `examples`, in one program. A run of it runs the test whose index the environment variable
    /// `TEST_INDEX_ENV_VAR` gives: the index of a test of `tests` is its position there, and the
    /// index of an example is the number of `tests` added to its position in `examples`.
    ///
    /// Each example is compiled as a module `DocTest.Example<hash>.DocTest`, where `<hash>` is
    /// taken from the path of its source. The path is decided by where the example stands and what
    /// it holds, so an example keeps its name as other examples come and go, and the caches keyed
    /// by the names of what it defines still answer for it. The name ends in `DocTest`, so a path
    /// the example writes relative, such as `DocTest::helper`, reaches the module as it reaches
    /// the module `DocTest` (see `NameSpace::is_suffix_of`). The parser renames the module at each
    /// place the source names it (see `ModuleRenaming`).
    ///
    /// # Examples
    /// An example whose source is at `.fixlang/tmp/src/doc_test.<origin>.<content>.fix` is
    /// compiled as a module such as `DocTest.Example3f9c0a1b2d4e5f60.DocTest`.
    pub fn dispatching(tests: Vec<FullName>, examples: Vec<SourceFile>) -> Result<Self, Errors> {
        let examples = examples
            .into_iter()
            .map(|source| {
                let hash = md5_hex(&source.file_path.to_string_lossy());
                ExampleModule {
                    name: format!("{0}.Example{1}.{0}", DOC_TEST_MODULE_NAME, &hash[..16]),
                    source,
                }
            })
            .collect::<Vec<_>>();
        assert!(
            examples
                .iter()
                .map(|example| example.name.as_str())
                .chain(iter::once(DISPATCHER_MODULE_NAME))
                .all(is_reserved_module_name),
            "the modules a build adds have names reserved for the Fix examples"
        );
        let values = tests
            .into_iter()
            .chain(examples.iter().map(ExampleModule::main))
            .collect::<Vec<_>>();
        let dispatcher = save_temporary_source(&dispatcher_source(&values), "test_dispatcher")?;
        Ok(TestBuild {
            examples,
            entry: TestEntry::Dispatched { values, dispatcher },
            reports_source_warnings: true,
        })
    }

    /// The build of the test `test` the sources declare alone, whose entry point is `test`.
    pub fn of_value(test: FullName) -> Self {
        TestBuild {
            examples: vec![],
            entry: TestEntry::Alone(test),
            reports_source_warnings: true,
        }
    }

    /// The build of the Fix example whose source is `source` alone, compiled as the module
    /// `DocTest`, whose entry point is `DocTest::main`. `reports_source_warnings` tells whether
    /// the build reports the warnings that lie outside the example: a build made after the sources
    /// were built and reported their warnings leaves them out, so that each is reported once.
    pub fn of_example(source: SourceFile, reports_source_warnings: bool) -> Self {
        let example = ExampleModule {
            source,
            name: DOC_TEST_MODULE_NAME.to_string(),
        };
        TestBuild {
            entry: TestEntry::Alone(example.main()),
            examples: vec![example],
            reports_source_warnings,
        }
    }

    /// Whether the build reports the warning `warning`.
    pub fn reports_warning(&self, warning: &Error) -> bool {
        self.reports_source_warnings
            || warning.srcs.iter().any(|(_, span)| {
                self.examples
                    .iter()
                    .any(|example| span.input == example.source)
            })
    }

    /// The values the dispatcher runs. A build of one test has none.
    ///
    /// The dispatcher calls each of them as a value of type `IO ()`, which a value declared at a
    /// more general type, such as `[m : Monad] m ()`, also passes. A build checks each of them as it
    /// checks the entry point (see `Program::check_value_has_type`), so that a test passes or fails
    /// as it does in a build of its own.
    pub fn dispatched_values(&self) -> &[FullName] {
        match &self.entry {
            TestEntry::Dispatched { values, .. } => values,
            TestEntry::Alone(_) => &[],
        }
    }

    /// The sources the build adds to the program, each with the renaming its module is compiled
    /// under.
    pub fn sources(&self) -> Vec<(SourceFile, Option<ModuleRenaming>)> {
        let examples = self.examples.iter().map(|example| {
            let renaming = ModuleRenaming {
                written: DOC_TEST_MODULE_NAME.to_string(),
                compiled: example.name.clone(),
            };
            (example.source.clone(), Some(renaming))
        });
        let dispatcher = match &self.entry {
            TestEntry::Dispatched { dispatcher, .. } => Some((dispatcher.clone(), None)),
            TestEntry::Alone(_) => None,
        };
        examples.chain(dispatcher).collect()
    }

    /// The value the program runs: `main` of the dispatcher, or the one test of the build.
    pub fn entry(&self) -> FullName {
        match &self.entry {
            TestEntry::Dispatched { .. } => {
                FullName::from_strs(&[DISPATCHER_MODULE_NAME], MAIN_FUNCTION_NAME)
            }
            TestEntry::Alone(value) => value.clone(),
        }
    }
}

/// The source of the module `DocTest.Dispatcher`, whose `main` runs the value of `values` whose
/// index the environment variable `TEST_INDEX_ENV_VAR` gives.
///
/// Each value is read inside a function, so that a run reads only the value it runs: a global value
/// is evaluated when it is first read, and the evaluation of a value of type `IO ()` can panic
/// before any I/O action of it runs.
fn dispatcher_source(values: &[FullName]) -> String {
    let entries = values
        .iter()
        .map(|entry| format!("        |_| ::{}", entry.to_string()))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"module {module};

{main} : IO () = (
    let value = *"{var}".borrow_c_str_io(|name| FFI_CALL_IO[Ptr fixruntime_getenv(Ptr), name]);
    let index : I64 = String::unsafe_from_c_str_ptr(value).from_string.as_ok;
    let entries : Array (() -> IO ()) = [
{entries}
    ];
    let entry = entries.@(index);
    entry()
);
"#,
        module = DISPATCHER_MODULE_NAME,
        main = MAIN_FUNCTION_NAME,
        var = TEST_INDEX_ENV_VAR,
        entries = entries,
    )
}

/// Whether `name` is a module name reserved for the Fix examples: `DocTest`, or a name beginning
/// with `DocTest.`. A build of `fix test` gives the modules it adds names of this kind.
///
/// # Examples
/// `DocTest` and `DocTest.Dispatcher` are reserved, and `DocTests` and `Lib.DocTest` are not.
fn is_reserved_module_name(name: &str) -> bool {
    name == DOC_TEST_MODULE_NAME || name.starts_with(&format!("{}.", DOC_TEST_MODULE_NAME))
}

/// Reports a module of `program` whose name is reserved for the Fix examples (see
/// `is_reserved_module_name`). A namespace, a type or a trait named `DocTest` stays free; an
/// example that refers to one of them by a name beginning with `DocTest` is reported as ambiguous,
/// with the absolute path that tells them apart.
pub fn check_doc_test_module_names_are_free(program: &Program) -> Result<(), Errors> {
    match program
        .modules
        .iter()
        .find(|module| is_reserved_module_name(&module.name))
    {
        Some(module) => Err(Errors::from_msg_srcs(
            format!(
                "The module name `{}` is reserved for the Fix examples of comments, which \
                 `fix test` compiles as modules named `{}` or with names beginning with `{}.`. \
                 Give this module another name.",
                module.name, DOC_TEST_MODULE_NAME, DOC_TEST_MODULE_NAME
            ),
            &[&Some(module.source.clone())],
        )),
        None => Ok(()),
    }
}
