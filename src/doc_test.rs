//! The Fix examples written in doc comments.
//!
//! A fenced code block of a doc comment whose info string begins with `fix` is a Fix example. A
//! line of it whose text begins with `# ` after its indentation is hidden: it is compiled with the
//! `# ` taken off, and left out when the doc comment is shown to a reader. `fix test` compiles each
//! example as the module `DocTest` and runs its `DocTest::main`.

use crate::{
    ast::{name::Name, program::Program},
    commands::docs::is_fence_line,
    constants::{DOC_TEST_MODULE_NAME, MAIN_FUNCTION_NAME},
    error::Errors,
    hash::md5_hex,
    misc::{save_temporary_source, to_absolute_path, Map, Set},
    parse::{
        parser::parse_source_module_defn,
        sourcefile::{DocLine, LineOrigin, SourceFile, SourceOrigin, Span},
    },
};
use std::path::PathBuf;

/// The mark of an info string that leaves a Fix example out of the tests.
const IGNORE_MARK: &str = "ignore";
/// The mark of an info string that has a Fix example compiled but not run.
const NO_RUN_MARK: &str = "no_run";

/// `docstring` as it is shown to a reader. In each Fix example, the hidden lines are taken out and
/// the info string is replaced by `fix`, the name a Markdown renderer highlights the code by.
///
/// # Examples
/// ~~~text
/// "```fix,no_run\n# import Foo;\nlet x = 1;\n#\n```\n"  ->  "```fix\nlet x = 1;\n```\n"
/// ~~~
pub fn docstring_for_display(docstring: &str) -> String {
    let lines = docstring.split('\n').collect::<Vec<_>>();
    let mut shown_lines = lines
        .iter()
        .map(|line| Some(line.to_string()))
        .collect::<Vec<_>>();
    for block in fix_example_blocks(&lines) {
        shown_lines[block.open] = Some(fence_with_info(lines[block.open], "fix"));
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

/// A Fix example of a doc comment.
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

impl FixExample {
    /// The place the example is reported at: the file its doc comment is written in and the line
    /// that opens it.
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

/// Reports a module of `program` named `DocTest`, the name each Fix example is compiled as. With
/// no such module, a name beginning with `DocTest` in an example refers to the example itself.
pub fn check_doc_test_module_name_is_free(program: &Program) -> Result<(), Errors> {
    match program
        .modules
        .iter()
        .find(|module| module.name == DOC_TEST_MODULE_NAME)
    {
        Some(module) => Err(Errors::from_msg_srcs(
            format!(
                "The module name `{}` is reserved for the Fix examples of doc comments, which \
                 `fix test` compiles as the module `{}`.",
                DOC_TEST_MODULE_NAME, DOC_TEST_MODULE_NAME
            ),
            &[&Some(module.source.clone())],
        )),
        None => Ok(()),
    }
}

/// The Fix examples of the doc comments written in `files`, ordered by the file and then by where
/// they stand in it. `program` is a program loaded from sources that `files` are among.
pub fn collect_examples(program: &Program, files: &[PathBuf]) -> Result<Vec<FixExample>, Errors> {
    let files = files
        .iter()
        .map(|file| to_absolute_path(file))
        .collect::<Result<Set<_>, _>>()?;
    let mut module_of_file: Map<PathBuf, Name> = Map::default();
    for module in &program.modules {
        module_of_file.insert(module.absolute_source_path()?, module.name.clone());
    }

    let mut examples = vec![];
    let mut errors = Errors::empty();
    for span in program.documentable_declaration_spans() {
        let path = to_absolute_path(&span.input.file_path)?;
        if !files.contains(&path) {
            continue;
        }
        let module = module_of_file.get(&path).unwrap_or_else(|| {
            panic!(
                "the declaration in \"{}\" belongs to the module the file declares",
                path.to_string_lossy()
            )
        });
        errors.eat_err_or(
            examples_in_document(&span.document_lines()?, module),
            |found| examples.extend(found),
        );
    }
    errors.to_result()?;
    examples.sort_by(|lhs, rhs| lhs.fence.cmp(&rhs.fence));
    Ok(examples)
}

/// The Fix examples of the doc comment whose lines are `doc_lines`, in order. `module` is the
/// module the doc comment is written in, which an example written as statements imports.
///
/// An error reports each info string carrying a mark other than `ignore` and `no_run` or carrying
/// both of them, each example written as a module named other than `DocTest`, and each example the
/// doc comment ends inside.
pub fn examples_in_document(
    doc_lines: &[DocLine],
    module: &Name,
) -> Result<Vec<FixExample>, Errors> {
    let texts = doc_lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>();
    let mut examples = vec![];
    let mut errors = Errors::empty();
    for block in fix_example_blocks(&texts) {
        errors.eat_err_or(example_of_block(doc_lines, &block, module), |example| {
            examples.push(example)
        });
    }
    errors.to_result()?;
    Ok(examples)
}

/// The Fix example `block` of the doc comment whose lines are `doc_lines`.
fn example_of_block(
    doc_lines: &[DocLine],
    block: &FencedBlock,
    module: &Name,
) -> Result<FixExample, Errors> {
    let opening = &doc_lines[block.open];
    let fence = Some(opening.span.clone());

    let mut ignore = false;
    let mut no_run = false;
    for mark in info_items(info_string(&opening.text)).skip(1) {
        match mark {
            IGNORE_MARK => ignore = true,
            NO_RUN_MARK => no_run = true,
            _ => {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "Unknown mark `{}` in the info string of a Fix example. The marks are `{}` \
                         and `{}`.",
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
            "The doc comment ends inside this Fix example. Close it by a line of ```.".to_string(),
            &[&fence],
        ));
    };

    let task = if ignore {
        ExampleTask::Ignore
    } else {
        let source = assemble_example(doc_lines, block.open, close, module)?;
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
/// `doc_lines` is compiled as.
///
/// An example that begins with a `module` declaration is the source of the module as it stands,
/// and it has to declare the module `DocTest`. Any other example is an expression of type `IO ()`,
/// which is wrapped into the module as the value `DocTest::main`, with `module` imported.
///
/// Each line of the example stays on the line of the doc comment it is written on, and the lines
/// the example is wrapped in are written on the lines of its fences, so the positions in the
/// source are reported where they stand in the doc comment (see `SourceOrigin`).
///
/// # Examples
/// The example `let x = 1;` / `assert_eq(|_|"", x, 1)` of a doc comment in the module
/// `Geometry` is compiled as
/// ~~~text
/// module DocTest; import Geometry; main : IO () = (
/// let x = 1;
/// assert_eq(|_|"", x, 1)
/// );
/// ~~~
fn assemble_example(
    doc_lines: &[DocLine],
    open: usize,
    close: usize,
    module: &Name,
) -> Result<SourceFile, Errors> {
    let (first_line, open_origin) = fence_origin(&doc_lines[open]);
    let (_, close_origin) = fence_origin(&doc_lines[close]);
    let mut code = vec![];
    let mut code_origins = vec![];
    for line in &doc_lines[open + 1..close] {
        let (_, column) = line.span.start_line_col();
        let example_line = ExampleLine::classify(&line.text);
        code.push(example_line.compiled());
        code_origins.push(LineOrigin::Taken {
            shift: column - 1 + example_line.taken_off(),
        });
    }
    let origin = SourceOrigin {
        file_path: doc_lines[open].span.input.file_path.clone(),
        first_line,
        lines: [vec![open_origin], code_origins, vec![close_origin]].concat(),
    };
    let assemble =
        |header: &str, footer: &str| format!("{}\n{}\n{}\n", header, code.join("\n"), footer);

    // The source is saved under a name the origin decides, so that a source read back from a
    // cache, which carries its path, finds this content and this origin at that path.
    let origin_text = serde_json::to_string(&origin).expect("a `SourceOrigin` is written as JSON");
    let file_name = format!("doc_test.{}", md5_hex(&origin_text));
    let save = |content: String| -> Result<SourceFile, Errors> {
        Ok(save_temporary_source(&content, &file_name)?.with_origin(origin.clone()))
    };

    // An example that begins with a `module` declaration is the source of the module.
    let module_source = save(assemble("", ""))?;
    match parse_source_module_defn(module_source.clone()) {
        Ok(module_info) => {
            if module_info.name != DOC_TEST_MODULE_NAME {
                return Err(Errors::from_msg_srcs(
                    format!(
                        "A Fix example written as a module declares the module `{}`. The module \
                         of a Fix example is `{}`.",
                        module_info.name, DOC_TEST_MODULE_NAME
                    ),
                    &[&Some(module_info.source)],
                ));
            }
            Ok(module_source)
        }
        Err(_) => save(assemble(
            &format!(
                "module {}; import {}; {} : IO () = (",
                DOC_TEST_MODULE_NAME, module, MAIN_FUNCTION_NAME
            ),
            ");",
        )),
    }
}

/// The line number of the fence `line` stands on, and how the line written on it in place of the
/// fence relates to the fence: a position on it is reported at the fence.
fn fence_origin(line: &DocLine) -> (usize, LineOrigin) {
    let (line_number, column) = line.span.start_line_col();
    let (indent, _) = split_indent(&line.text);
    let origin = LineOrigin::Written {
        column: column + indent.chars().count(),
        width: line.text.trim().chars().count(),
    };
    (line_number, origin)
}

/// A fenced code block of a docstring, by the indices of its lines.
struct FencedBlock {
    /// The index of the line opening the block.
    open: usize,
    /// The index of the line closing the block, or `None` for a block the docstring ends inside.
    close: Option<usize>,
}

/// The fenced code blocks of the docstring whose lines are `lines`, in order. A fence line opens a
/// block, and the next fence line closes it.
fn fenced_blocks(lines: &[&str]) -> Vec<FencedBlock> {
    let mut blocks = vec![];
    let mut open = None;
    for (index, line) in lines.iter().enumerate() {
        if !is_fence_line(line) {
            continue;
        }
        match open.take() {
            None => open = Some(index),
            Some(open) => blocks.push(FencedBlock {
                open,
                close: Some(index),
            }),
        }
    }
    if let Some(open) = open {
        blocks.push(FencedBlock { open, close: None });
    }
    blocks
}

/// The fenced code blocks of the docstring whose lines are `lines` that are Fix examples, in order.
fn fix_example_blocks(lines: &[&str]) -> Vec<FencedBlock> {
    fenced_blocks(lines)
        .into_iter()
        .filter(|block| is_fix_example(info_string(lines[block.open])))
        .collect()
}

/// The info string of the block `fence` opens: what follows its backticks, trimmed.
fn info_string(fence: &str) -> &str {
    fence.trim_start().trim_start_matches('`').trim()
}

/// The items of the info string `info`, separated by `,` and trimmed. The first is the language of
/// the block, and the rest are marks.
fn info_items(info: &str) -> impl Iterator<Item = &str> {
    info.split(',').map(str::trim)
}

/// Whether the block whose info string is `info` is a Fix example: the first item of the info
/// string is `fix`.
fn is_fix_example(info: &str) -> bool {
    info_items(info).next() == Some("fix")
}

/// The fence `fence` with its info string replaced by `info`.
fn fence_with_info(fence: &str, info: &str) -> String {
    let (indent, after_indent) = split_indent(fence);
    let backticks =
        &after_indent[..after_indent.len() - after_indent.trim_start_matches('`').len()];
    format!("{}{}{}", indent, backticks, info)
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
    use super::docstring_for_display;

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
}
