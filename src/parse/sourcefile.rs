use crate::{elaboration::read_file, error::Errors, misc::to_absolute_path, parse::parser::Rule};
use colored::{Color, Colorize};
use pest::iterators::Pair;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// A file of Fix source code the compiler reads, named by its path.
///
/// The content and the hash are computed on the first request and kept, so a file that is asked
/// for many times is read once. The path is what a serialized `SourceFile` carries; the content
/// and the hash are read again wherever it is deserialized.
#[derive(Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// The path the file is read from. It names the file: two `SourceFile`s are equal, and are
    /// ordered, by this path alone.
    pub file_path: PathBuf,
    /// The content of the file, once it has been read.
    #[serde(skip)]
    string: Arc<Mutex<Option<String>>>,
    /// The value `hash` answers with, once it has been computed.
    #[serde(skip)]
    hash: Arc<Mutex<Option<String>>>,
    /// Where the lines of an assembled source were taken from, which the positions in it are
    /// reported against. `None` for a source whose positions are its own.
    ///
    /// The path of an assembled source is chosen for its origin, so two sources of one path share
    /// their origin, as equality by path requires.
    origin: Option<Arc<SourceOrigin>>,
}

impl PartialEq for SourceFile {
    /// Two source files are equal when their paths are equal. The content and the hash are what
    /// the path names, so they follow from it.
    fn eq(&self, other: &Self) -> bool {
        self.file_path == other.file_path
    }
}

impl Eq for SourceFile {}

impl PartialOrd for SourceFile {
    /// Source files are ordered by their paths, which orders every pair of them, so this always
    /// answers with an ordering.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceFile {
    /// Orders source files by their paths.
    fn cmp(&self, other: &Self) -> Ordering {
        self.file_path.cmp(&other.file_path)
    }
}

impl Hash for SourceFile {
    /// Hashes a source file by its path, which is what names it.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_path.hash(state);
    }
}

impl SourceFile {
    /// The content of the file. It is read from disk on the first request and kept for the later
    /// ones.
    pub fn string(&self) -> Result<String, Errors> {
        if self.string.lock().unwrap().is_none() {
            self.read_file()?;
        }
        Ok(self.string.lock().unwrap().as_ref().unwrap().clone())
    }

    /// The source file at `file_path`, whose content is read from disk when it is first asked for.
    pub fn from_file_path(file_path: PathBuf) -> Self {
        Self {
            string: Arc::new(Mutex::new(None)),
            hash: Arc::new(Mutex::new(None)),
            file_path,
            origin: None,
        }
    }

    /// The source file at `file_path` whose content is `content`, which stands in for what the
    /// path holds. The path names the file as it does for any other, so content that was never
    /// written to disk still belongs to the path it is given.
    pub fn from_file_path_and_content(file_path: PathBuf, content: String) -> Self {
        Self {
            string: Arc::new(Mutex::new(Some(content))),
            hash: Arc::new(Mutex::new(None)),
            file_path,
            origin: None,
        }
    }

    /// This source with `origin` as the place its lines were taken from, so that the positions in
    /// it are reported where they stand in the origin's file.
    pub fn with_origin(mut self, origin: SourceOrigin) -> Self {
        self.origin = Some(Arc::new(origin));
        self
    }

    /// The path the positions in this source are reported against: the file its lines were taken
    /// from, where it was assembled from another file, and its own path otherwise.
    pub fn reported_path(&self) -> &PathBuf {
        match &self.origin {
            Some(origin) => &origin.file_path,
            None => &self.file_path,
        }
    }

    /// Reads the file from disk and keeps its content for every later request.
    fn read_file(&self) -> Result<(), Errors> {
        match read_file(&self.file_path) {
            Ok(source) => {
                let mut string = self.string.lock().unwrap();
                *string = Some(source);
                Ok(())
            }
            Err(e) => Err(Errors::from_msg(e)),
        }
    }

    /// A hash naming this source file: the path it is read from, together with its content.
    ///
    /// The caches of the compiler are keyed by this hash, and the path belongs in it because the
    /// path reaches what a cache entry carries. Every `Span` records the file it points into, so
    /// the file path travels with a cached typed expression into the diagnostics reported about
    /// it, and with a cached object file into its debug information. Two files of equal content
    /// are two files still, and the entry written for one names the other's file wrongly.
    pub fn hash(&self) -> Result<String, Errors> {
        if self.hash.lock().unwrap().is_none() {
            // The path goes in with its length in front of it, which fixes where it ends: every
            // pair of a path and a content gives a sequence of bytes of its own, and no other pair
            // gives that one. The path is taken as the bytes it is made of, so two paths that
            // differ outside what is spelled in UTF-8 differ here too.
            let mut context = md5::Context::new();
            let path = self.file_path.as_os_str().as_encoded_bytes();
            context.consume((path.len() as u64).to_le_bytes());
            context.consume(path);
            context.consume(self.string()?);
            let hash_str = format!("{:x}", context.compute());
            let mut hash = self.hash.lock().unwrap();
            *hash = Some(hash_str);
        }
        Ok(self.hash.lock().unwrap().as_ref().unwrap().clone())
    }

    /// The directory of the file the positions in this source are reported against, as the path
    /// spells it.
    pub fn get_file_dir(&self) -> String {
        self.reported_path()
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    /// The name of the file the positions in this source are reported against, in its directory.
    pub fn get_file_name(&self) -> String {
        self.reported_path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

/// Where the lines of a source assembled from parts of another file were taken from.
///
/// The Fix example of a comment is compiled from such a source: line `k` of it is line
/// `first_line + k - 1` of the file the comment is written in, with what precedes the comment's text
/// taken off the front, and the text the example is wrapped in is written on the lines of its
/// fences, or into the line its expression begins on.
///
/// # Examples
/// The source `"main : IO () = (\npure()\n);\n"` assembled from lines 7 to 9 of
/// ~~~text
/// // ```fix
/// // pure()
/// // ```
/// ~~~
/// has the origin `{ first_line: 7, lines: [Written { column: 4, width: 6 }, Taken { shift: 3,
/// inserted: None }, Written { column: 4, width: 3 }] }`, and the `p` of `pure` is reported at
/// line 8, column 4.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct SourceOrigin {
    /// The file the lines were taken from.
    pub file_path: PathBuf,
    /// The line of `file_path` the first line of the assembled source was taken from, counted from 1.
    pub first_line: usize,
    /// How each line of the assembled source relates to its line of `file_path`, in order. It is
    /// never empty. The end of the source, past the line break that ends its last line, stands
    /// where the last line does.
    pub lines: Vec<LineOrigin>,
}

/// How a line of an assembled source relates to the line of the origin's file it stands for.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub enum LineOrigin {
    /// The line holds text of the origin's line with `shift` characters taken off the front of it,
    /// and the text the assembler wrote into it at `inserted`, if any. A character taken stands at
    /// its column on the line plus `shift`, less the width of the text inserted before it.
    Taken {
        shift: usize,
        inserted: Option<Insertion>,
    },
    /// The line holds text the assembler wrote, which the origin's line does not have. A position
    /// on it is reported at the `width` characters beginning at `column` of the origin's line,
    /// which are what the text was written for.
    Written { column: usize, width: usize },
}

/// Text the assembler wrote into a line of taken text: `width` characters beginning at `column` of
/// the assembled line. A position in it is reported at the character taken that follows it.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Insertion {
    pub column: usize,
    pub width: usize,
}

impl SourceOrigin {
    /// The line and the column of the origin's file that the position at `line` and `column` of
    /// the assembled source stands at. All of them count from 1.
    pub fn position(&self, (line, column): (usize, usize)) -> (usize, usize) {
        let (origin_line, line_origin) = self.origin_of_line(line);
        match line_origin {
            LineOrigin::Taken { shift, inserted } => {
                let taken_column = match inserted {
                    Some(insertion) if column >= insertion.column => {
                        column.max(insertion.column + insertion.width) - insertion.width
                    }
                    _ => column,
                };
                (origin_line, taken_column + shift)
            }
            LineOrigin::Written { column, .. } => (origin_line, *column),
        }
    }

    /// The line of the origin's file that line `line` of the assembled source stands for, counted
    /// from 1, and how the two relate.
    fn origin_of_line(&self, line: usize) -> (usize, &LineOrigin) {
        assert!(
            !self.lines.is_empty(),
            "an assembled source taken from \"{}\" has a line",
            self.file_path.to_string_lossy()
        );
        assert!(
            line <= self.lines.len() + 1,
            "line {} of an assembled source of {} lines taken from \"{}\"",
            line,
            self.lines.len(),
            self.file_path.to_string_lossy()
        );
        let index = (line - 1).min(self.lines.len() - 1);
        (self.first_line + index, &self.lines[index])
    }

    /// `quoted`, a line of the assembled source quoted under a diagnostic, as the line of the
    /// origin's file it stands for: that line's number and text, and the columns the underline
    /// covers there. `origin_lines` are the lines of the origin's file, or `None` where the file
    /// cannot be read, in which case the text quoted is the assembled one.
    fn quote(&self, quoted: QuotedLine, origin_lines: Option<&Vec<&str>>) -> QuotedLine {
        let (line, column) = self.position((quoted.line, quoted.column));
        let width = match self.origin_of_line(quoted.line).1 {
            LineOrigin::Written { width, .. } => *width,
            LineOrigin::Taken { inserted: None, .. } => quoted.width,
            // The underline loses the inserted characters it covers.
            LineOrigin::Taken {
                inserted: Some(insertion),
                ..
            } => {
                let covered_end = (quoted.column + quoted.width).min(insertion.column + insertion.width);
                let covered = covered_end.saturating_sub(quoted.column.max(insertion.column));
                (quoted.width - covered).max(1)
            }
        };
        match origin_lines.and_then(|lines| lines.get(line - 1)) {
            Some(text) => QuotedLine {
                line,
                text: text.trim_end().to_string(),
                column,
                width,
            },
            None => QuotedLine { line, ..quoted },
        }
    }
}

/// A line of a source quoted under a diagnostic, with the part of it the span covers underlined.
struct QuotedLine {
    /// The number of the line, counted from 1.
    line: usize,
    /// The text of the line, without its line break.
    text: String,
    /// The column the underline begins at, counted from 1.
    column: usize,
    /// How many characters the underline covers, at least 1.
    width: usize,
}

/// A single position in a source file, given as a byte offset into its content.
pub struct SourcePos {
    /// The file the position points into.
    pub input: SourceFile,
    /// The byte offset of the position from the beginning of the file's content.
    pub pos: usize,
}

/// The text of the `//` comment `comment`, which begins with its `//`: what follows the `//` and
/// one space after it.
///
/// # Examples
/// `line_comment_text("// a b")` is `"a b"`, and `line_comment_text("//  a")` is `" a"`.
pub fn line_comment_text(comment: &str) -> &str {
    let after_slashes = &comment[2..];
    after_slashes.strip_prefix(' ').unwrap_or(after_slashes)
}

/// A range of bytes of a source file, together with the file it points into.
///
/// It owns the file it points into, so it can be stored in the syntax tree and written into the
/// compiler's caches, where a `pest::Span` lives only as long as the content it borrows.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    /// The file the range lies in.
    pub input: SourceFile,
    /// Start byte index (inclusive).
    pub start: usize,
    /// End byte index (exclusive).
    pub end: usize,
}

impl Span {
    /// A span over `src` that covers nothing: it begins past every position of the file and ends
    /// before every one, so uniting it with a span answers with that span itself.
    #[allow(dead_code)]
    pub fn empty(src: &SourceFile) -> Self {
        Self {
            input: src.clone(),
            start: usize::max_value(),
            end: 0,
        }
    }

    /// The range of `src` that the parsed `pair` was matched over.
    pub fn from_pair(src: &SourceFile, pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Self {
            input: src.clone(),
            start: span.start(),
            end: span.end(),
        }
    }

    /// The smallest span covering both spans, which also covers whatever lies between them. It
    /// points into this span's file, so the two are expected to lie in one file.
    pub fn unite(&self, other: &Self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// The span of `self[start..end]`, where both are byte offsets from the beginning of this span.
    ///
    /// # Examples
    /// The part `(3, 9)` of the span of `abc\u0000def` is the span of its `\u0000`.
    pub fn part(&self, start: usize, end: usize) -> Self {
        assert!(
            start <= end && self.start + end <= self.end,
            "a part lies within its span, but the bytes `{}..{}` from its beginning are asked of \
             the span at `{}..{}`",
            start,
            end,
            self.start,
            self.end
        );
        Self {
            input: self.input.clone(),
            start: self.start + start,
            end: self.start + end,
        }
    }

    /// The span of the single byte this span begins at.
    pub fn to_head_character(&self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start,
            end: self.start + 1,
        }
    }

    /// The empty span at the position this span ends at, which points just past its last byte.
    pub fn to_end_position(&self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.end,
            end: self.end,
        }
    }

    /// The empty span just past the byte this span begins at.
    pub fn after_head_character(&self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start + 1,
            end: self.start + 1,
        }
    }

    /// The smallest span covering both, where both spans are there to be united.
    pub fn unite_opt(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
        if lhs.is_none() {
            return None;
        }
        if rhs.is_none() {
            return None;
        }
        Some(lhs.clone().unwrap().unite(rhs.as_ref().unwrap()))
    }

    /// The line this span begins on, counted from 1.
    pub fn start_line_no(&self) -> usize {
        self.start_line_col().0
    }

    /// The line and the column this span begins at. Both count from 1, and the column counts the
    /// characters of the line.
    pub fn start_line_col(&self) -> (usize, usize) {
        self.line_col(|span| span.start_pos().line_col())
    }

    /// The line and the column this span ends at. Both count from 1, and the column counts the
    /// characters of the line.
    pub fn end_line_col(&self) -> (usize, usize) {
        self.line_col(|span| span.end_pos().line_col())
    }

    /// The line and column number `of_position` reads off this span, taken over the content of the
    /// file the span points into and reported where the origin of that file puts it.
    ///
    /// Returns `(0, 0)` when that file cannot be read.
    fn line_col(&self, of_position: impl FnOnce(&pest::Span) -> (usize, usize)) -> (usize, usize) {
        let source_string = self.input.string();
        if let Err(_e) = source_string {
            return (0, 0);
        }
        let source_string = source_string.ok().unwrap();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();
        let line_col = of_position(&span);
        match &self.input.origin {
            Some(origin) => origin.position(line_col),
            None => line_col,
        }
    }

    /// The position and the file name of this span, followed by every source line it reaches, each
    /// carrying `^^^` markers under the part the span covers. A span of an assembled source is
    /// shown where its origin puts it, quoting the lines of the origin's file. The result is empty
    /// where the file cannot be read.
    ///
    /// # Arguments
    ///
    /// * `underline_color` - The color of the `^^^` markers, typically red for an error and yellow
    ///   for a warning.
    pub fn to_string(&self, underline_color: Color) -> String {
        let source_string = self.input.string();
        if let Err(_e) = source_string {
            return "".to_string();
        }
        let source_string = source_string.ok().unwrap();
        let opt_span = pest::Span::new(&source_string, self.start, self.end);
        if opt_span.is_none() {
            return "".to_string();
        }
        let span = opt_span.unwrap();

        let mut start = span.start_pos().line_col();
        let mut end = span.end_pos().line_col();
        let mut quoted_lines = span
            .lines_span()
            .map(|line_span| {
                let start_pos = span.start_pos().max(line_span.start_pos());
                let end_pos = span.end_pos().min(line_span.end_pos());
                QuotedLine {
                    line: line_span.start_pos().line_col().0,
                    text: String::from(line_span.as_str()).trim_end().to_string(),
                    column: start_pos.line_col().1,
                    width: (end_pos.pos() - start_pos.pos()).max(1),
                }
            })
            .collect::<Vec<_>>();
        if let Some(origin) = &self.input.origin {
            // A span at the end of the source reaches no line of it, and the origin still has the
            // line that end stands for, so that line is quoted.
            if quoted_lines.is_empty() {
                quoted_lines.push(QuotedLine {
                    line: start.0,
                    text: String::new(),
                    column: start.1,
                    width: 1,
                });
            }
            start = origin.position(start);
            end = origin.position(end);
            let origin_string = SourceFile::from_file_path(origin.file_path.clone()).string();
            let origin_lines = origin_string
                .as_ref()
                .ok()
                .map(|content| content.lines().collect::<Vec<_>>());
            quoted_lines = quoted_lines
                .into_iter()
                .map(|quoted| origin.quote(quoted, origin_lines.as_ref()))
                .collect();
        }

        let linenum_str_size = quoted_lines
            .iter()
            .map(|quoted| quoted.line.to_string().len())
            .max()
            .unwrap_or(0);

        let mut ret: String = String::default();
        ret += &format!(
            "{}:{}-{}:{} in \"{}\", \n",
            start.0,
            start.1,
            end.0,
            end.1,
            self.input.reported_path().to_str().unwrap().to_string()
        );
        ret += &(" ".repeat(linenum_str_size) + &" | " + "\n");
        for quoted in quoted_lines {
            let linenum_str = quoted.line.to_string();
            ret +=
                &(linenum_str.clone() + &" ".repeat(linenum_str_size - linenum_str.len()) + &" | ");
            ret += &quoted.text;
            ret += "\n";
            ret += &(" ".repeat(linenum_str_size) + &" | ");
            ret += &(" ".repeat(quoted.column - 1)
                + &"^".repeat(quoted.width).color(underline_color).to_string());
            ret += "\n";
        }
        ret
    }

    /// The document of a declaration: the comment written above `src` where the declaration comes
    /// from a source, and `fallback` otherwise. A document with no text in it is answered as `None`.
    pub fn document_of_declaration(
        src: &Option<Span>,
        fallback: &Option<String>,
    ) -> Option<String> {
        src.as_ref()
            .and_then(|src| src.get_document().ok())
            .filter(|docs| !docs.is_empty())
            .or_else(|| fallback.clone())
            .filter(|docs| !docs.is_empty())
    }

    /// The document of the entity defined at this span: the content of the consecutive comment
    /// lines written just before the span begins, each stripped of its `//` and of one space after
    /// it. The document is empty where anything else stands on the line the definition begins on.
    pub fn get_document(&self) -> Result<String, Errors> {
        let source_string = self.input.string()?;

        // The line the definition begins on. Anything written ahead of the definition on it means
        // there is no document.
        let definition_line_start = source_string[..self.start]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        if !source_string[definition_line_start..self.start]
            .trim()
            .is_empty()
        {
            return Ok(String::default());
        }

        // Read the lines above it, from the nearest one up, while they are comment lines.
        let mut lines = vec![];
        let mut next_line_start = definition_line_start;
        while next_line_start > 0 {
            let line_end = next_line_start - 1;
            let line_start = source_string[..line_end]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            let comment = source_string[line_start..line_end].trim();
            if !comment.starts_with("//") {
                break;
            }
            lines.push(line_comment_text(comment));
            next_line_start = line_start;
        }
        let mut ret = String::default();
        for line in lines.iter().rev() {
            ret += line;
            ret += "\n";
        }
        Ok(ret)
    }

    /// Whether `byte` falls within this span, both ends included.
    ///
    /// The byte is taken as an offset into this span's own file, so the file it belongs to is
    /// settled before the call.
    pub fn includes_byte(&self, byte: usize) -> bool {
        self.start <= byte && byte <= self.end
    }

    /// Whether `pos` points into the same file as this span and falls within it, both ends
    /// included.
    ///
    /// This answers the position an LSP (Language Server Protocol) client sends.
    pub fn includes_pos_lsp(&self, pos: &SourcePos) -> bool {
        let file_path_abs = to_absolute_path(&self.input.file_path);
        let pos_file_path_abs = to_absolute_path(&pos.input.file_path);
        if file_path_abs.is_err() || pos_file_path_abs.is_err() {
            return false;
        }
        if file_path_abs.ok().unwrap() != pos_file_path_abs.ok().unwrap() {
            return false;
        }
        // The end of the span counts as inside it: when you double-click a symbol in VSCode to
        // select it and then right-click to choose "Go to Definition", the LSP client sends the
        // position next to the last character of the symbol, so including the end is what carries
        // "Go to Definition" to the symbol.
        //
        // A symbol therefore also answers a Ctrl-click made with the cursor just past its last
        // character, as it does in Rust-analyzer.
        self.start <= pos.pos && pos.pos <= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::SourceFile;
    use std::path::PathBuf;

    /// The caches of the compiler are named by the hash of a source file, so two files that differ
    /// get two names. The path and the content are hashed one after the other, and a boundary
    /// between them that can move gives one name to a pair of files: `("ab", "c")` and
    /// `("a", "bc")` are two files, and each keeps a name of its own.
    #[test]
    fn test_the_hash_separates_the_path_from_the_content() {
        let hash_of = |path: &str, content: &str| {
            SourceFile::from_file_path_and_content(PathBuf::from(path), content.to_string())
                .hash()
                .unwrap_or_else(|errs| panic!("Failed to hash a source file: {}", errs))
        };
        assert_ne!(
            hash_of("ab", "c"),
            hash_of("a", "bc"),
            "two files whose path and content run together alike keep their own hashes"
        );
    }
}
