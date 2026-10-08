//! The Fix examples written in comments.
//!
//! A comment is read as Markdown, and a fenced code block of it whose info string begins with `fix`
//! is a Fix example. A line of an example whose text begins with `# ` after its indentation is
//! hidden: it is compiled with the `# ` taken off, and left out when a doc comment is shown to a
//! reader. `fix test` compiles each example as the module `DocTest` and runs its `DocTest::main`.
//! It compiles the examples of a program together into one program where it can (see
//! `ExampleBuild`).

use crate::{
    ast::{
        name::{FullName, Name},
        program::Program,
    },
    constants::{DOC_TEST_EXAMPLE_ENV_VAR, DOC_TEST_MODULE_NAME, MAIN_FUNCTION_NAME},
    error::{Error, Errors},
    hash::md5_hex,
    misc::{save_temporary_source, to_absolute_path, Set},
    parse::{
        parser::{
            comment_ranges, code_after_import_statements, parse_source_module_defn,
            ModuleRenaming,
        },
        sourcefile::{line_comment_text, Insertion, LineOrigin, SourceFile, SourceOrigin, Span},
    },
};
use std::iter;
use std::path::PathBuf;

/// The language an info string names to make a code block a Fix example.
const FIX_LANGUAGE: &str = "fix";
/// The mark of an info string that leaves a Fix example out of the tests.
const IGNORE_MARK: &str = "ignore";
/// The mark of an info string that has a Fix example compiled but not run.
const NO_RUN_MARK: &str = "no_run";

/// `docstring` as it is shown to a reader. In each Fix example, the hidden lines are taken out and
/// the info string is replaced by `fix`, the name a Markdown renderer highlights the code by.
///
/// # Examples
/// ~~~text
/// "```fix no_run\n# import Foo;\nlet x = 1;\n#\n```\n"  ->  "```fix\nlet x = 1;\n```\n"
/// ~~~
pub fn docstring_for_display(docstring: &str) -> String {
    let lines = docstring.split('\n').collect::<Vec<_>>();
    let mut shown_lines = lines
        .iter()
        .map(|line| Some(line.to_string()))
        .collect::<Vec<_>>();
    for block in fix_example_blocks(&lines) {
        shown_lines[block.open] = Some(format!("{}fix", block.fence.prefix));
        for index in block.open + 1..block.close.unwrap_or(lines.len()) {
            if !matches!(ExampleLine::classify(lines[index]), ExampleLine::Shown(_)) {
                shown_lines[index] = None;
            }
        }
    }
    shown_lines
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
}

/// A line of a comment or of a Markdown document, with where its text stands in the file.
pub struct TextLine {
    /// The text of the line, without its line break and the white space it ends with. The text of
    /// a line of a `//` comment follows the `//` and one space after it.
    pub text: String,
    /// Where `text` stands in the file.
    pub span: Span,
}

/// The line that opens a fenced code block of Markdown: a run of three or more backticks, or of
/// three or more tildes, after the indentation, followed by the info string. A backtick fence has
/// no backtick in its info string, which is what tells it from a line of inline code.
///
/// # Examples
/// ```` ```fix no_run ```` opens a block with the info string `fix no_run`, which the line
/// ```` ``` ```` closes and the line ```` ```fix ```` does not.
pub struct CodeFence<'a> {
    /// The character the fence is made of, a backtick or a tilde.
    marker: char,
    /// How many times `marker` is repeated.
    length: usize,
    /// The indentation and the run of `marker`, as the line writes them.
    pub prefix: &'a str,
    /// The info string, trimmed.
    pub info: &'a str,
}

impl<'a> CodeFence<'a> {
    /// The fence the line `line` opens a code block with, or `None` for a line that opens none.
    pub fn opening(line: &'a str) -> Option<Self> {
        let (indent, after_indent) = split_indent(line);
        if indent_width(indent) >= CODE_INDENT {
            return None;
        }
        let marker = after_indent
            .chars()
            .next()
            .filter(|c| *c == '`' || *c == '~')?;
        let length = after_indent.len() - after_indent.trim_start_matches(marker).len();
        let info = after_indent[length..].trim();
        if length < 3 || (marker == '`' && info.contains('`')) {
            return None;
        }
        Some(CodeFence {
            marker,
            length,
            prefix: &line[..indent.len() + length],
            info,
        })
    }

    /// Whether the line `line` closes the block this fence opens: a run of the same character,
    /// at least as long, with nothing else on the line.
    pub fn is_closed_by(&self, line: &str) -> bool {
        let (indent, after_indent) = split_indent(line);
        if indent_width(indent) >= CODE_INDENT {
            return false;
        }
        let text = after_indent.trim_end();
        let length = text.len() - text.trim_start_matches(self.marker).len();
        length >= self.length && length == text.len()
    }
}

/// The width of indentation that makes a line of Markdown a line of an indented code block, so that
/// a fence indented this far is text of that block.
const CODE_INDENT: usize = 4;

/// The width of the indentation `indent`, where a tab reaches the next multiple of four as it does
/// in Markdown.
///
/// # Examples
/// `indent_width("  ")` is 2, and `indent_width(" \t")` is 4.
fn indent_width(indent: &str) -> usize {
    indent.chars().fold(0, |width, c| match c {
        '\t' => width + 4 - width % 4,
        _ => width + 1,
    })
}

/// A Fix example of a comment or of a Markdown document.
pub struct FixExample {
    /// Where the line opening the example stands, which names the example in what `fix test`
    /// reports.
    pub fence: Span,
    /// What `fix test` does with the example.
    pub task: ExampleTask,
}

/// What `fix test` does with a Fix example, as the marks of its info string decide.
pub enum ExampleTask {
    /// Compiles the source and runs it. The example passes when the program exits with status 0.
    Run(SourceFile),
    /// Compiles the source alone, which is what the mark `no_run` asks for.
    Compile(SourceFile),
    /// Nothing, which is what the mark `ignore` asks for.
    Ignore,
}

impl ExampleTask {
    /// The source the task compiles, or `None` for a task that compiles nothing.
    pub fn source(&self) -> Option<&SourceFile> {
        match self {
            ExampleTask::Run(source) | ExampleTask::Compile(source) => Some(source),
            ExampleTask::Ignore => None,
        }
    }
}

impl FixExample {
    /// The place the example is reported at: the file its comment is written in and the line that
    /// opens it.
    ///
    /// # Examples
    /// `src/geometry.fix:12`
    pub fn location(&self) -> String {
        format!(
            "{}:{}",
            self.fence.input.reported_path().to_string_lossy(),
            self.fence.start_line_no()
        )
    }
}

/// The Fix examples a build of `fix test` compiles beside the sources of the program, and the
/// value the program runs.
#[derive(Clone)]
pub struct ExampleBuild {
    /// The examples, each with the name of the module it is compiled as.
    examples: Vec<ExampleModule>,
    /// The source of the module whose `main` runs the example whose index in `examples` the
    /// environment variable `DOC_TEST_EXAMPLE_ENV_VAR` gives, where the build holds several
    /// examples. A build of one example has none, and runs its `DocTest::main`.
    dispatcher: Option<SourceFile>,
}

/// A Fix example as a build compiles it.
#[derive(Clone)]
struct ExampleModule {
    /// The source assembled from the example, which declares the module `DocTest`.
    source: SourceFile,
    /// The name the module is compiled as.
    name: Name,
}

/// The name of the module whose `main` runs one of the examples a build holds.
const DISPATCHER_MODULE_NAME: &str = "DocTest.Examples";

impl ExampleBuild {
    /// The build of the example whose source is `source` alone, compiled as the module `DocTest`.
    pub fn single(source: SourceFile) -> Self {
        ExampleBuild {
            examples: vec![ExampleModule {
                source,
                name: DOC_TEST_MODULE_NAME.to_string(),
            }],
            dispatcher: None,
        }
    }

    /// The build of the examples whose sources are `sources` in one program, which runs the
    /// example whose index in `sources` the environment variable `DOC_TEST_EXAMPLE_ENV_VAR` gives.
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
    pub fn merged(sources: Vec<SourceFile>) -> Result<Self, Errors> {
        let examples = sources
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
            "the modules a build of several examples adds have names reserved for the examples"
        );
        let dispatcher =
            save_temporary_source(&dispatcher_source(&examples), "doc_test_dispatcher")?;
        Ok(ExampleBuild {
            examples,
            dispatcher: Some(dispatcher),
        })
    }

    /// Whether the build reports the warning `warning`. A build of one example is made where the
    /// examples are tested one by one, after the sources were built alone and reported their
    /// warnings, so it reports only the warnings that lie in its example. A build of several
    /// examples reports every warning.
    pub fn reports_warning(&self, warning: &Error) -> bool {
        if self.dispatcher.is_some() {
            return true;
        }
        warning.srcs.iter().any(|(_, span)| {
            self.examples
                .iter()
                .any(|example| span.input == example.source)
        })
    }

    /// The `main` of each example the dispatcher runs. A build of one example has none.
    ///
    /// The dispatcher calls each of them as a value of type `IO ()`, which a `main` declared at a
    /// more general type, such as `[m : Monad] m ()`, also passes. A build checks each of them as it
    /// checks the entry point (see `Program::check_value_has_type`), so that an example passes or
    /// fails as it does in a build of its own.
    pub fn dispatched_mains(&self) -> Vec<FullName> {
        if self.dispatcher.is_none() {
            return vec![];
        }
        self.examples
            .iter()
            .map(|example| FullName::from_strs(&[&example.name], MAIN_FUNCTION_NAME))
            .collect()
    }

    /// The build of the program's sources with no example in them, whose entry runs nothing. It is
    /// the build `merged` gives for no example, so the sources build under the entry and the
    /// settings they build under with the examples.
    pub fn without_examples() -> Result<Self, Errors> {
        Self::merged(vec![])
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
        let dispatcher = self.dispatcher.iter().map(|source| (source.clone(), None));
        examples.chain(dispatcher).collect()
    }

    /// The value the program runs: `main` of the dispatcher, or `DocTest::main` of the one example.
    pub fn entry(&self) -> FullName {
        let module = match self.dispatcher {
            Some(_) => DISPATCHER_MODULE_NAME,
            None => DOC_TEST_MODULE_NAME,
        };
        FullName::from_strs(&[module], MAIN_FUNCTION_NAME)
    }
}

/// The source of the module `DocTest.Examples`, whose `main` runs the example of `examples` whose
/// index the environment variable `DOC_TEST_EXAMPLE_ENV_VAR` gives.
///
/// The `main` of each example is read inside a function, so that a run reads only the `main` of
/// the example it runs: a global value is evaluated when it is first read, and the value of a
/// `main` can panic before any I/O action of it runs.
fn dispatcher_source(examples: &[ExampleModule]) -> String {
    let mains = examples
        .iter()
        .map(|example| format!("        |_| ::{}::{}", example.name, MAIN_FUNCTION_NAME))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"module {module};

{main} : IO () = (
    let value = *"{var}".borrow_c_str_io(|name| FFI_CALL_IO[Ptr fixruntime_getenv(Ptr), name]);
    let index : I64 = String::unsafe_from_c_str_ptr(value).from_string.as_ok;
    let examples : Array (() -> IO ()) = [
{mains}
    ];
    let example = examples.@(index);
    example()
);
"#,
        module = DISPATCHER_MODULE_NAME,
        main = MAIN_FUNCTION_NAME,
        var = DOC_TEST_EXAMPLE_ENV_VAR,
        mains = mains,
    )
}

/// Whether `name` is a module name reserved for the Fix examples: `DocTest`, or a name beginning
/// with `DocTest.`. A build of several examples gives the modules it adds names of this kind.
///
/// # Examples
/// `DocTest` and `DocTest.Examples` are reserved, and `DocTests` and `Lib.DocTest` are not.
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

/// The Fix examples of the comments written in `files`, ordered by the file and then by where they
/// stand in it. `program` is a program loaded from sources that `files` are among, and each file is
/// read as the source of the module it declares.
pub fn collect_examples(program: &Program, files: &[PathBuf]) -> Result<Vec<FixExample>, Errors> {
    let files = files
        .iter()
        .map(|file| to_absolute_path(file))
        .collect::<Result<Set<_>, _>>()?;
    let mut examples = vec![];
    let mut errors = Errors::empty();
    for module in &program.modules {
        if !files.contains(&module.absolute_source_path()?) {
            continue;
        }
        for comment in comments_of(&module.source.input)? {
            errors.eat_err_or(examples_in_text(&comment, &module.name), |found| {
                examples.extend(found)
            });
        }
    }
    errors.to_result()?;
    examples.sort_by(|lhs, rhs| lhs.fence.cmp(&rhs.fence));
    Ok(examples)
}

/// The comments of `source`, each as its lines: every `/* */` comment, and every run of `//`
/// comments that stand alone on consecutive lines. A `//` comment written after code on its line is
/// a comment of its own. A line of a `//` comment is the text after the `//` and one space after it;
/// a line of a `/* */` comment is the text as it is written. The white space a line ends with is
/// left out.
///
/// # Examples
/// The source `"// a\n//  b\nx = 1; // c\n/* d\n e */"` has the three comments `["a", " b"]`,
/// `["c"]` and `[" d", " e"]`.
fn comments_of(source: &SourceFile) -> Result<Vec<Vec<TextLine>>, Errors> {
    let content = source.string()?;
    let line_starts = iter::once(0)
        .chain(content.match_indices('\n').map(|(newline, _)| newline + 1))
        .collect::<Vec<_>>();
    let line_of = |byte: usize| line_starts.partition_point(|start| *start <= byte) - 1;
    let text_line = |start: usize, text: &str| {
        let text = text.trim_end();
        TextLine {
            text: text.to_string(),
            span: Span {
                input: source.clone(),
                start,
                end: start + text.len(),
            },
        }
    };

    let mut comments: Vec<Vec<TextLine>> = vec![];
    // The line of the last `//` comment that stands alone on its line, which the next one on the
    // line below continues.
    let mut run_last_line: Option<usize> = None;
    for range in comment_ranges(&content) {
        let text = &content[range.clone()];
        if text.starts_with("//") {
            let body = line_comment_text(text);
            let line = line_of(range.start);
            let alone = content[line_starts[line]..range.start].trim().is_empty();
            let line_comment = text_line(range.end - body.len(), body);
            match comments.last_mut() {
                Some(comment) if alone && run_last_line.map(|l| l + 1) == Some(line) => {
                    comment.push(line_comment)
                }
                _ => comments.push(vec![line_comment]),
            }
            run_last_line = if alone { Some(line) } else { None };
        } else {
            let body = text[2..]
                .strip_suffix("*/")
                .expect("a `/* */` comment the scan finds is closed");
            let mut start = range.start + 2;
            let mut lines = vec![];
            for line_text in body.split('\n') {
                lines.push(text_line(start, line_text));
                start += line_text.len() + 1;
            }
            comments.push(lines);
            run_last_line = None;
        }
    }
    Ok(comments)
}

/// The Fix examples of the comment or the document whose lines are `lines`, in order. `module`
/// is the module it is written in, which an example written as statements imports.
///
/// An error reports each info string carrying a mark other than `ignore` and `no_run` or carrying
/// both of them, each example written as a module named other than `DocTest`, each example whose
/// import statements are followed by no expression, and each example the text ends inside.
pub fn examples_in_text(
    lines: &[TextLine],
    module: &Name,
) -> Result<Vec<FixExample>, Errors> {
    let texts = lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>();
    let mut examples = vec![];
    let mut errors = Errors::empty();
    for block in fix_example_blocks(&texts) {
        errors.eat_err_or(example_of_block(lines, &block, module), |example| {
            examples.push(example)
        });
    }
    errors.to_result()?;
    Ok(examples)
}

/// The Fix example `block` of the comment or the document whose lines are `lines`.
fn example_of_block(
    lines: &[TextLine],
    block: &FencedBlock,
    module: &Name,
) -> Result<FixExample, Errors> {
    let opening = &lines[block.open];
    let fence = Some(opening.span.clone());

    let mut words = info_words(block.fence.info);
    if words.next() != Some(FIX_LANGUAGE) {
        return Err(Errors::from_msg_srcs(
            format!(
                "Separate the marks of a Fix example from `{}` by spaces, as in `{} {}`.",
                FIX_LANGUAGE, FIX_LANGUAGE, NO_RUN_MARK
            ),
            &[&fence],
        ));
    }
    let mut ignore = false;
    let mut no_run = false;
    for mark in words {
        match mark {
            IGNORE_MARK => ignore = true,
            NO_RUN_MARK => no_run = true,
            _ => {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Unknown mark `{}` in the info string of a Fix example. The marks are `{}` \
                         and `{}`, separated by spaces.",
                        mark, IGNORE_MARK, NO_RUN_MARK
                    ),
                    &[&fence],
                ))
            }
        }
    }
    if ignore && no_run {
        return Err(Errors::from_msg_srcs(
            format!(
                "A Fix example is marked both `{}` and `{}`.",
                IGNORE_MARK, NO_RUN_MARK
            ),
            &[&fence],
        ));
    }
    let Some(close) = block.close else {
        return Err(Errors::from_msg_srcs(
            format!(
                "The comment ends inside this Fix example. Close it by a line of {}.",
                block.fence.prefix.trim_start()
            ),
            &[&fence],
        ));
    };

    let task = if ignore {
        ExampleTask::Ignore
    } else {
        let source = assemble_example(lines, block.open, close, module)?;
        if no_run {
            ExampleTask::Compile(source)
        } else {
            ExampleTask::Run(source)
        }
    };
    Ok(FixExample {
        fence: opening.span.clone(),
        task,
    })
}

/// The source of the module `DocTest` the Fix example between the fences `open` and `close` of
/// `lines` is compiled as. `module` is the module the comment is written in.
///
/// An example that begins with a `module` declaration is the source of the module as it stands,
/// and it has to declare the module `DocTest`. Any other example is an expression of type `IO ()`,
/// which may be preceded by import statements. It is wrapped into the module as the value
/// `DocTest::main`, and the module imports `module` and what the import statements name. As in any
/// module, `Std` is imported whole unless an import statement names it.
///
/// Each line of the example stays on the line of the comment it is written on, and the text the
/// example is wrapped in is written on the lines of its fences, or into the line where its
/// expression begins when import statements precede it, so the positions in the source are
/// reported where they stand in the comment (see `SourceOrigin`).
///
/// # Examples
/// The example `let x = 1;` / `assert_eq(|_|"", x, 1)` of a comment in the module
/// `Geometry` is compiled as
/// ~~~text
/// module DocTest; import Geometry; main : ::Std::IO () = (
/// let x = 1;
/// assert_eq(|_|"", x, 1)
/// );
/// ~~~
/// and the example `import Math;` / `assert_eq(|_|"", sqrt(4.0), 2.0)` as
/// ~~~text
/// module DocTest; import Geometry;
/// import Math;
/// main : ::Std::IO () = (assert_eq(|_|"", sqrt(4.0), 2.0)
/// );
/// ~~~
fn assemble_example(
    lines: &[TextLine],
    open: usize,
    close: usize,
    module: &Name,
) -> Result<SourceFile, Errors> {
    let (first_line, open_origin) = fence_origin(&lines[open]);
    let (_, close_origin) = fence_origin(&lines[close]);
    let mut code = vec![];
    let mut shifts = vec![];
    for line in &lines[open + 1..close] {
        let (_, column) = line.span.start_line_col();
        let example_line = ExampleLine::classify(&line.text);
        code.push(example_line.compiled());
        shifts.push(column - 1 + example_line.taken_off());
    }

    // Saves the source whose first line is `header`, whose last line is `footer`, and whose other
    // lines are those of the example, with `insertion`'s text written into the line of the example
    // it names, at the byte offset it names.
    let save = |header: String,
                insertion: Option<(usize, usize, &str)>,
                footer: &str|
     -> Result<SourceFile, Errors> {
        let mut body = code.clone();
        let mut body_origins = shifts
            .iter()
            .map(|&shift| LineOrigin::Taken {
                shift,
                inserted: None,
            })
            .collect::<Vec<_>>();
        if let Some((line, offset, text)) = insertion {
            body_origins[line] = LineOrigin::Taken {
                shift: shifts[line],
                inserted: Some(Insertion {
                    column: body[line][..offset].chars().count() + 1,
                    width: text.chars().count(),
                }),
            };
            body[line].insert_str(offset, text);
        }
        let origin = SourceOrigin {
            file_path: lines[open].span.input.file_path.clone(),
            first_line,
            lines: [vec![open_origin.clone()], body_origins, vec![close_origin.clone()]].concat(),
        };
        let source_lines = iter::once(header)
            .chain(body)
            .chain(iter::once(footer.to_string()))
            .collect::<Vec<_>>();
        assert_eq!(
            source_lines.len(),
            origin.lines.len(),
            "an assembled example has a line for each line its origin records"
        );
        // The source is saved under a name the origin decides, so that a source read back from a
        // cache, which carries its path, finds this content and this origin at that path.
        let origin_text =
            serde_json::to_string(&origin).expect("a `SourceOrigin` is written as JSON");
        let file_name = format!("doc_test.{}", md5_hex(&origin_text));
        let content = format!("{}\n", source_lines.join("\n"));
        Ok(save_temporary_source(&content, &file_name)?.with_origin(origin))
    };

    // An example that begins with a `module` declaration is the source of the module.
    let module_source = save(String::new(), None, "")?;
    if let Ok(module_info) = parse_source_module_defn(module_source.clone()) {
        if module_info.name != DOC_TEST_MODULE_NAME {
            return Err(Errors::from_msg_srcs(
                format!(
                    "A Fix example written as a module declares the module `{}`. The module of a \
                     Fix example is `{}`.",
                    module_info.name, DOC_TEST_MODULE_NAME
                ),
                &[&Some(module_info.source)],
            ));
        }
        return Ok(module_source);
    }

    let module_header = format!("module {}; import {};", DOC_TEST_MODULE_NAME, module);
    let main_head = format!("{} : ::Std::IO () = (", MAIN_FUNCTION_NAME);
    let joined = code.join("\n");
    let Some(expression_start) = code_after_import_statements(&joined) else {
        return save(format!("{} {}", module_header, main_head), None, ");");
    };
    if expression_start == joined.len() {
        return Err(Errors::from_msg_srcs(
            "A Fix example has import statements and no expression after them.".to_string(),
            &[&Some(lines[open].span.clone())],
        ));
    }
    let expression_line = joined[..expression_start].matches('\n').count();
    let line_start = joined[..expression_start].rfind('\n').map_or(0, |at| at + 1);
    save(
        module_header,
        Some((expression_line, expression_start - line_start, &main_head)),
        ");",
    )
}

/// The line number of the fence `line` stands on, and how the line written on it in place of the
/// fence relates to the fence: a position on it is reported at the fence.
fn fence_origin(line: &TextLine) -> (usize, LineOrigin) {
    let (line_number, column) = line.span.start_line_col();
    let (indent, _) = split_indent(&line.text);
    let origin = LineOrigin::Written {
        column: column + indent.chars().count(),
        width: line.text.trim().chars().count(),
    };
    (line_number, origin)
}

/// A fenced code block of a Markdown text, by the indices of its lines.
struct FencedBlock<'a> {
    /// The index of the line opening the block.
    open: usize,
    /// The fence that line opens the block with.
    fence: CodeFence<'a>,
    /// The index of the line closing the block, or `None` for a block the text ends inside.
    close: Option<usize>,
}

/// The fenced code blocks of the Markdown text whose lines are `lines`, in order.
fn fenced_blocks<'a>(lines: &[&'a str]) -> Vec<FencedBlock<'a>> {
    let mut blocks = vec![];
    let mut open: Option<(usize, CodeFence)> = None;
    for (index, line) in lines.iter().enumerate() {
        match open.take() {
            None => open = CodeFence::opening(line).map(|fence| (index, fence)),
            Some((open_index, fence)) if fence.is_closed_by(line) => blocks.push(FencedBlock {
                open: open_index,
                fence,
                close: Some(index),
            }),
            Some(still_open) => open = Some(still_open),
        }
    }
    if let Some((open, fence)) = open {
        blocks.push(FencedBlock {
            open,
            fence,
            close: None,
        });
    }
    blocks
}

/// The fenced code blocks of the Markdown text whose lines are `lines` that are Fix examples, in
/// order.
fn fix_example_blocks<'a>(lines: &[&'a str]) -> Vec<FencedBlock<'a>> {
    fenced_blocks(lines)
        .into_iter()
        .filter(|block| is_fix_example(block.fence.info))
        .collect()
}

/// The words of the info string `info`, separated by white space. The first is the language of the
/// block, as Markdown reads it, and the rest are marks.
fn info_words(info: &str) -> impl Iterator<Item = &str> {
    info.split_whitespace()
}

/// Whether the block whose info string is `info` is a Fix example: the first word of the info
/// string is `fix`, or begins with `fix,`. A block of the second kind is taken as a Fix example so
/// that its marks, written after commas, are reported as an error.
///
/// # Examples
/// `fix`, `fix no_run` and `fix,no_run` are Fix examples, and `fixme` and `rust` are not.
fn is_fix_example(info: &str) -> bool {
    info_words(info)
        .next()
        .and_then(|language| language.strip_prefix(FIX_LANGUAGE))
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(','))
}

/// `line` split into its indentation and the text that follows it.
fn split_indent(line: &str) -> (&str, &str) {
    line.split_at(line.len() - line.trim_start().len())
}

/// A line of a Fix example, by the part it takes in the program and in what a reader is shown.
enum ExampleLine<'a> {
    /// A line shown and compiled as it is written.
    Shown(&'a str),
    /// A line whose text begins with `# ` after its indentation: it is compiled as `indent`
    /// followed by `code`, and not shown.
    Hidden { indent: &'a str, code: &'a str },
    /// A line of `#` alone: it is compiled as an empty line, and not shown.
    HiddenBlank,
}

impl<'a> ExampleLine<'a> {
    /// The kind of the line `line` of a Fix example.
    fn classify(line: &'a str) -> Self {
        let (indent, after_indent) = split_indent(line);
        if after_indent == "#" {
            ExampleLine::HiddenBlank
        } else if let Some(code) = after_indent.strip_prefix("# ") {
            ExampleLine::Hidden { indent, code }
        } else {
            ExampleLine::Shown(line)
        }
    }

    /// The text of the line that is compiled.
    fn compiled(&self) -> String {
        match self {
            ExampleLine::Shown(line) => line.to_string(),
            ExampleLine::Hidden { indent, code } => format!("{}{}", indent, code),
            ExampleLine::HiddenBlank => String::new(),
        }
    }

    /// How many characters in front of the code of the line are taken off to compile it, which
    /// moves the code that many columns left.
    fn taken_off(&self) -> usize {
        match self {
            ExampleLine::Hidden { .. } => 2,
            ExampleLine::Shown(_) | ExampleLine::HiddenBlank => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{comments_of, docstring_for_display};
    use crate::error::panic_if_err;
    use crate::parse::sourcefile::SourceFile;
    use std::path::PathBuf;

    /// Each `/* */` comment is one comment, and so is each run of `//` comments standing alone on
    /// consecutive lines. A `//` comment after code on its line stands on its own, and a line that
    /// holds no comment ends a run.
    #[test]
    fn test_comments_of() {
        let source = SourceFile::from_file_path_and_content(
            PathBuf::from("lib.fix"),
            "// a\n//  b\nx = 1; // c\n// d\n\n// e\ns = \"// f\";\n/* g\n h */".to_string(),
        );
        let comments = panic_if_err(comments_of(&source))
            .into_iter()
            .map(|comment| {
                comment
                    .into_iter()
                    .map(|line| line.text)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            comments,
            vec![
                vec!["a", " b"],
                vec!["c"],
                vec!["d"],
                vec!["e"],
                vec![" g", " h"],
            ],
            "the comments are split as their lines and their kinds say, and a `//` in a string \
             literal is no comment"
        );

        let source = SourceFile::from_file_path_and_content(
            PathBuf::from("lib.fix"),
            "s = \"a\n// b\n/* c\";\n// d\nx = 1;".to_string(),
        );
        let comments = panic_if_err(comments_of(&source))
            .into_iter()
            .map(|comment| {
                comment
                    .into_iter()
                    .map(|line| line.text)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            comments,
            vec![vec!["d"]],
            "a string literal that spans lines holds no comment, and the comment after it is found"
        );
    }

    /// A reader is shown a Fix example without its hidden lines and with `fix` as its info string.
    /// A line beginning with `#` outside a Fix example, such as a heading or a line of a block in
    /// another language, is shown as it is written.
    #[test]
    fn test_docstring_for_display() {
        assert_eq!(
            docstring_for_display(
                "# Examples\n  ```` fix , no_run\n  # import Lib;\n#\n  let x = 1;\n##\n  ````\n"
            ),
            "# Examples\n  ````fix\n  let x = 1;\n##\n  ````\n",
            "the hidden lines and the marks of a Fix example are left out"
        );
        assert_eq!(
            docstring_for_display("```sh\n# a comment\n```\n```fixme\n# kept\n```\n"),
            "```sh\n# a comment\n```\n```fixme\n# kept\n```\n",
            "a block whose info string does not begin with the item `fix` is shown as it is"
        );
        assert_eq!(
            docstring_for_display("```fix\n# hidden\nshown"),
            "```fix\nshown",
            "an example the docstring ends inside is shown up to the end"
        );
    }

    /// A fenced code block is closed by a fence of its own character, at least as long as the one
    /// that opened it, with nothing else on the line, as in CommonMark. A Fix example can therefore
    /// hold a shorter fence, a block of tildes can hold a Fix example as text, and a line of inline
    /// code opens nothing.
    #[test]
    fn test_code_fences_pair_as_in_commonmark() {
        assert_eq!(
            docstring_for_display("````fix\n# hidden\n```\n# hidden after the inner fence\n````\n"),
            "````fix\n```\n````\n",
            "a shorter fence inside a Fix example belongs to the example"
        );
        assert_eq!(
            docstring_for_display("~~~text\n```fix\n# shown\n```\n~~~\n"),
            "~~~text\n```fix\n# shown\n```\n~~~\n",
            "a Fix example written inside a block of tildes is text of that block"
        );
        assert_eq!(
            docstring_for_display("```x```\n```fix\n# hidden\n```\n"),
            "```x```\n```fix\n```\n",
            "a line of inline code opens no block"
        );
        assert_eq!(
            docstring_for_display("```fix\n# hidden\n``` text\n# hidden after the line\n```\n"),
            "```fix\n``` text\n```\n",
            "a fence followed by text on its line closes no block"
        );
        assert_eq!(
            docstring_for_display("~~~fix no_run\n# hidden\nshown\n~~~\n"),
            "~~~fix\nshown\n~~~\n",
            "a block of tildes whose info string is `fix` is a Fix example"
        );
        assert_eq!(
            docstring_for_display("    ```fix\n    # shown\n    ```\n\t```fix\n\t# shown\n\t```\n"),
            "    ```fix\n    # shown\n    ```\n\t```fix\n\t# shown\n\t```\n",
            "a fence indented four columns or more is text of an indented code block"
        );
        assert_eq!(
            docstring_for_display("```fix\n# hidden\n    ```\n# hidden after the line\n```\n"),
            "```fix\n    ```\n```\n",
            "a fence indented four columns or more closes no block"
        );
        assert_eq!(
            docstring_for_display("- item\n\n  ```fix\n  # hidden\n  ```\n"),
            "- item\n\n  ```fix\n  ```\n",
            "a fence on its own line inside a list item opens a Fix example"
        );
        assert_eq!(
            docstring_for_display("- ```fix\n  # shown\n  ```\n> ```fix\n> # shown\n> ```\n"),
            "- ```fix\n  # shown\n  ```\n> ```fix\n> # shown\n> ```\n",
            "a fence after a list marker or in a block quote opens no Fix example"
        );
    }
}
