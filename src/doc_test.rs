//! The Fix examples written in doc comments.
//!
//! A fenced code block of a doc comment whose info string begins with `fix` is a Fix example. A
//! line of it whose text begins with `# ` after its indentation is hidden: it is compiled with the
//! `# ` taken off, and left out when the doc comment is shown to a reader. `fix test` compiles each
//! example as the module `DocTest` and runs its `DocTest::main`.

use crate::{
    ast::{name::Name, program::Program},
    constants::{DOC_TEST_MODULE_NAME, MAIN_FUNCTION_NAME},
    error::Errors,
    hash::md5_hex,
    misc::{save_temporary_source, to_absolute_path, Set},
    parse::{
        lexer::{lex_tokens, LexTokenKind},
        parser::parse_source_module_defn,
        sourcefile::{line_comment_text, LineOrigin, SourceFile, SourceOrigin, Span},
    },
};
use std::iter;
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
/// ```` ```fix,no_run ```` opens a block with the info string `fix,no_run`, which the line
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
        let after_indent = line.trim_start();
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
            prefix: &line[..line.len() - after_indent.len() + length],
            info,
        })
    }

    /// Whether the line `line` closes the block this fence opens: a run of the same character,
    /// at least as long, with nothing else on the line.
    pub fn is_closed_by(&self, line: &str) -> bool {
        let text = line.trim();
        let length = text.len() - text.trim_start_matches(self.marker).len();
        length >= self.length && length == text.len()
    }
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

/// Reports the first declaration of `program` whose name has a component `DocTest`, the name each
/// Fix example is compiled as: a module of that name, a type or a trait of that name, or a
/// declaration inside a namespace of that name. A type or a trait opens a namespace of its name,
/// which holds its field accessors and its members.
///
/// A name refers to every declaration whose full name ends with it, so with no such declaration a
/// name beginning with `DocTest` in an example refers to the example itself.
pub fn check_doc_test_name_is_free(program: &Program) -> Result<(), Errors> {
    let modules = program
        .modules
        .iter()
        .map(|module| (vec![module.name.clone()], Some(module.source.clone())));
    let values = program.global_values.iter().map(|(name, value)| {
        (
            name.namespace.names.clone(),
            value.decl_src.clone().or(value.defn_src.clone()),
        )
    });
    let types = program.type_defns.iter().map(|type_defn| {
        (
            path_of(&type_defn.name.namespace.names, &type_defn.name.name),
            type_defn.source.clone(),
        )
    });
    let traits = program.trait_env.traits.iter().map(|(id, trait_defn)| {
        (
            path_of(&id.name.namespace.names, &id.name.name),
            trait_defn.source.clone(),
        )
    });
    let trait_aliases = program.trait_env.aliases.data.iter().map(|(id, alias)| {
        (
            path_of(&id.name.namespace.names, &id.name.name),
            alias.source.clone(),
        )
    });
    let taken = modules
        .chain(values)
        .chain(types)
        .chain(traits)
        .chain(trait_aliases)
        .find(|(path, _)| path.iter().any(|name| name == DOC_TEST_MODULE_NAME));
    match taken {
        Some((_, source)) => Err(Errors::from_msg_srcs(
            format!(
                "The name `{}` is reserved for the Fix examples of comments, which `fix test` \
                 compiles as the module `{}`. Give this module, namespace, type or trait another name.",
                DOC_TEST_MODULE_NAME, DOC_TEST_MODULE_NAME
            ),
            &[&source],
        )),
        None => Ok(()),
    }
}

/// The names of the namespace `namespace` followed by `name`: the path a type or a trait named
/// `name` opens as a namespace of its own.
fn path_of(namespace: &[Name], name: &Name) -> Vec<Name> {
    namespace.iter().chain([name]).cloned().collect()
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
    for token in lex_tokens(&content) {
        if token.kind != LexTokenKind::Comment {
            continue;
        }
        let text = &content[token.start..token.end];
        if text.starts_with("//") {
            let body = line_comment_text(text);
            let line = line_of(token.start);
            let alone = content[line_starts[line]..token.start].trim().is_empty();
            let line_comment = text_line(token.end - body.len(), body);
            match comments.last_mut() {
                Some(comment) if alone && run_last_line.map(|l| l + 1) == Some(line) => {
                    comment.push(line_comment)
                }
                _ => comments.push(vec![line_comment]),
            }
            run_last_line = if alone { Some(line) } else { None };
        } else {
            let body = &text[2..];
            let body = body.strip_suffix("*/").unwrap_or(body);
            let mut start = token.start + 2;
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
/// both of them, each example written as a module named other than `DocTest`, and each example the
/// doc comment ends inside.
pub fn examples_in_text(lines: &[TextLine], module: &Name) -> Result<Vec<FixExample>, Errors> {
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

/// The Fix example `block` of the doc comment whose lines are `lines`.
fn example_of_block(
    lines: &[TextLine],
    block: &FencedBlock,
    module: &Name,
) -> Result<FixExample, Errors> {
    let opening = &lines[block.open];
    let fence = Some(opening.span.clone());

    let mut ignore = false;
    let mut no_run = false;
    for mark in info_items(block.fence.info).skip(1) {
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
/// `lines` is compiled as.
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
    lines: &[TextLine],
    open: usize,
    close: usize,
    module: &Name,
) -> Result<SourceFile, Errors> {
    let (first_line, open_origin) = fence_origin(&lines[open]);
    let (_, close_origin) = fence_origin(&lines[close]);
    let mut code = vec![];
    let mut code_origins = vec![];
    for line in &lines[open + 1..close] {
        let (_, column) = line.span.start_line_col();
        let example_line = ExampleLine::classify(&line.text);
        code.push(example_line.compiled());
        code_origins.push(LineOrigin::Taken {
            shift: column - 1 + example_line.taken_off(),
        });
    }
    let origin = SourceOrigin {
        file_path: lines[open].span.input.file_path.clone(),
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
fn fence_origin(line: &TextLine) -> (usize, LineOrigin) {
    let (line_number, column) = line.span.start_line_col();
    let (indent, _) = split_indent(&line.text);
    let origin = LineOrigin::Written {
        column: column + indent.chars().count(),
        width: line.text.trim().chars().count(),
    };
    (line_number, origin)
}

/// A fenced code block of a docstring, by the indices of its lines.
struct FencedBlock<'a> {
    /// The index of the line opening the block.
    open: usize,
    /// The fence that line opens the block with.
    fence: CodeFence<'a>,
    /// The index of the line closing the block, or `None` for a block the docstring ends inside.
    close: Option<usize>,
}

/// The fenced code blocks of the docstring whose lines are `lines`, in order.
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

/// The fenced code blocks of the docstring whose lines are `lines` that are Fix examples, in order.
fn fix_example_blocks<'a>(lines: &[&'a str]) -> Vec<FencedBlock<'a>> {
    fenced_blocks(lines)
        .into_iter()
        .filter(|block| is_fix_example(block.fence.info))
        .collect()
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
            docstring_for_display("~~~fix,no_run\n# hidden\nshown\n~~~\n"),
            "~~~fix\nshown\n~~~\n",
            "a block of tildes whose info string is `fix` is a Fix example"
        );
    }
}
