use crate::{elaboration::read_file, error::Errors, misc::to_absolute_path, parse::parser::Rule};
use colored::{Color, Colorize};
use pest::iterators::Pair;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// A file of Fix source code the compiler reads, named by its path.
///
/// The content and the hash are computed on the first request and kept, so a file that is asked
/// for many times is read once. The path is what a serialized `SourceFile` carries; the content
/// and the hash are read again wherever it is deserialized.
// PROOF: D/A, P2a, P15, P16, P17, P18, P18c, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
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

    /// The directory the file lies in, as the path spells it.
    pub fn get_file_dir(&self) -> String {
        self.file_path
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    /// The last component of the path, the name the file carries in its directory.
    pub fn get_file_name(&self) -> String {
        self.file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

/// A single position in a source file, given as a byte offset into its content.
pub struct SourcePos {
    /// The file the position points into.
    pub input: SourceFile,
    /// The byte offset of the position from the beginning of the file's content.
    pub pos: usize,
}

/// A range of bytes of a source file, together with the file it points into.
///
/// It owns the file it points into, so it can be stored in the syntax tree and written into the
/// compiler's caches, where a `pest::Span` lives only as long as the content it borrows.
// PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    /// file the span points into.
    ///
    /// Returns `(0, 0)` when that file cannot be read.
    fn line_col(&self, of_position: impl FnOnce(&pest::Span) -> (usize, usize)) -> (usize, usize) {
        let source_string = self.input.string();
        if let Err(_e) = source_string {
            return (0, 0);
        }
        let source_string = source_string.ok().unwrap();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();
        of_position(&span)
    }

    /// The position and the file name of this span, followed by every source line it reaches, each
    /// carrying `^^^` markers under the part the span covers. The result is empty where the file
    /// cannot be read.
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

        let mut linenum_str_size = 0;
        for line_span in span.lines_span() {
            let linenum = line_span.start_pos().line_col().0;
            linenum_str_size = linenum_str_size.max(linenum.to_string().len());
        }

        let mut ret: String = String::default();
        ret += &format!(
            "{}:{}-{}:{} in \"{}\", \n",
            span.start_pos().line_col().0,
            span.start_pos().line_col().1,
            span.end_pos().line_col().0,
            span.end_pos().line_col().1,
            self.input.file_path.to_str().unwrap().to_string()
        );
        ret += &(" ".repeat(linenum_str_size) + &" | " + "\n");
        for line_span in span.lines_span() {
            let linenum_str = line_span.start_pos().line_col().0.to_string();
            ret +=
                &(linenum_str.clone() + &" ".repeat(linenum_str_size - linenum_str.len()) + &" | ");
            ret += String::from(line_span.as_str()).trim_end();
            ret += "\n";
            ret += &(" ".repeat(linenum_str_size) + &" | ");
            let start_pos = span.start_pos().max(line_span.start_pos());
            let end_pos = span.end_pos().min(line_span.end_pos());
            let start_col = start_pos.line_col().1;
            let span_len = (end_pos.pos() - start_pos.pos()).max(1);
            ret += &(" ".repeat(start_col - 1)
                + &"^".repeat(span_len).color(underline_color).to_string());
            ret += "\n";
        }
        ret
    }

    /// The document of the entity defined at this span: the content of the consecutive comment
    /// lines written just before the span begins, each stripped of its `//` and of one space after
    /// it. The document is empty where anything else stands on the line the definition begins on.
    pub fn get_document(&self) -> Result<String, Errors> {
        /// One line read backwards from `chars`, in reading order, together with whether the
        /// beginning of the content was reached while reading it.
        fn get_line(chars: &mut dyn Iterator<Item = char>) -> (String, bool) {
            let mut ret = String::default();
            let at_end = loop {
                let c = chars.next();
                if c.is_none() {
                    break true;
                }
                let c = c.unwrap();
                ret.push(c);
                if c == '\n' {
                    break false;
                }
            };
            (ret.chars().rev().collect::<String>(), at_end)
        }

        let mut lines_rev = vec![];
        let source_string = self.input.string()?;
        let mut chars = source_string[0..self.start].chars().rev();

        // Get the string ahead of the definition.
        let (string_before_defn, _) = get_line(&mut chars);

        // If some non-whitespace characters are found ahead of the definition, there is no document.
        if string_before_defn.trim().len() > 0 {
            return Ok(String::default());
        }

        loop {
            let (line, reached_start) = get_line(&mut chars);
            let line = line.trim();

            // Check if `line` is a comment line.
            if !line.starts_with("//") {
                break;
            }

            // If the comment starts with " ", remove it.
            let comment = if line.starts_with("// ") {
                line[3..].to_string()
            } else {
                line[2..].to_string()
            };

            lines_rev.push(comment);

            if reached_start {
                break;
            }
        }
        // Concatenate the lines in reverse order.
        let mut ret = String::default();
        for line in lines_rev.iter().rev() {
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
