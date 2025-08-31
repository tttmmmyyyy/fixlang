use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::{error::Errors, misc::to_absolute_path, parser::Rule, runner::read_file};

#[derive(Clone, Serialize, Deserialize)]
pub struct SourceFile {
    // The file path.
    pub file_path: PathBuf,
    #[serde(skip)]
    // Cached content of the file.
    string: Arc<Mutex<Option<String>>>,
    // Hash of the file content.
    #[serde(skip)]
    hash: Arc<Mutex<Option<String>>>,
}

impl SourceFile {
    pub fn string(&self) -> Result<String, Errors> {
        if self.string.lock().unwrap().is_none() {
            self.read_file()?;
        }
        Ok(self.string.lock().unwrap().as_ref().unwrap().clone())
    }

    pub fn from_file_path(file_path: PathBuf) -> Self {
        Self {
            string: Arc::new(Mutex::new(None)),
            hash: Arc::new(Mutex::new(None)),
            file_path,
        }
    }

    pub fn from_file_path_and_content(file_path: PathBuf, content: String) -> Self {
        Self {
            string: Arc::new(Mutex::new(Some(content))),
            hash: Arc::new(Mutex::new(None)),
            file_path,
        }
    }

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

    pub fn hash(&self) -> Result<String, Errors> {
        if self.hash.lock().unwrap().is_none() {
            let hash = md5::compute(self.string()?);
            let hash_str = format!("{:x}", hash);
            let mut hash = self.hash.lock().unwrap();
            *hash = Some(hash_str);
        }
        Ok(self.hash.lock().unwrap().as_ref().unwrap().clone())
    }

    pub fn get_file_dir(&self) -> String {
        self.file_path
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn get_file_name(&self) -> String {
        self.file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

pub struct SourcePos {
    pub input: SourceFile,
    pub pos: usize,
}

// lifetime-free version of pest::Span
#[derive(Clone, Serialize, Deserialize)]
pub struct Span {
    pub input: SourceFile,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[allow(dead_code)]
    pub fn empty(src: &SourceFile) -> Self {
        Self {
            input: src.clone(),
            start: usize::max_value(),
            end: 0,
        }
    }

    pub fn from_pair(src: &SourceFile, pair: &Pair<Rule>) -> Self {
        let span = pair.as_span();
        Self {
            input: src.clone(),
            start: span.start(),
            end: span.end(),
        }
    }

    pub fn unite(&self, other: &Self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn to_head_character(&self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start,
            end: self.start + 1,
        }
    }

    pub fn offset(&self, offset: usize) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start + offset,
            end: self.end + offset,
        }
    }

    pub fn unite_opt(lhs: &Option<Span>, rhs: &Option<Span>) -> Option<Span> {
        if lhs.is_none() {
            return None;
        }
        if rhs.is_none() {
            return None;
        }
        Some(lhs.clone().unwrap().unite(rhs.as_ref().unwrap()))
    }

    // Get line number of start.
    pub fn start_line_no(&self) -> usize {
        self.start_line_col().0
    }

    #[allow(dead_code)]
    pub fn range(&self) -> (usize, usize) {
        (self.start, self.end)
    }

    // Get line and column number of start.
    pub fn start_line_col(&self) -> (usize, usize) {
        let source_string = self.input.string();
        if let Err(_e) = source_string {
            return (0, 0);
        }
        let source_string = source_string.ok().unwrap();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();
        span.start_pos().line_col()
    }

    // Get line and column number of end.
    pub fn end_line_col(&self) -> (usize, usize) {
        let source_string = self.input.string();
        if let Err(_e) = source_string {
            return (0, 0);
        }
        let source_string = source_string.ok().unwrap();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();
        span.end_pos().line_col()
    }

    // Show source codes around this span.
    pub fn to_string(&self) -> String {
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
            let span_len = end_pos.pos() - start_pos.pos();
            ret += &(" ".repeat(start_col - 1) + &"^".repeat(span_len).red().to_string());
            ret += "\n";
        }
        ret
    }

    // Get the document of the entity defined at this span.
    // More specifically, this function returns the content of the consecutive comment lines just before the start of the span.
    pub fn get_document(&self) -> Result<String, Errors> {
        // Get a line from the reversed iterator.
        // Returns the line and whether the end of the iterator is reached.
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

    // Check if the position is included in the span.
    pub fn includes_pos(&self, pos: &SourcePos) -> bool {
        let file_path_abs = to_absolute_path(&self.input.file_path);
        let pos_file_path_abs = to_absolute_path(&pos.input.file_path);
        if file_path_abs.is_err() || pos_file_path_abs.is_err() {
            return false;
        }
        if file_path_abs.ok().unwrap() != pos_file_path_abs.ok().unwrap() {
            return false;
        }
        // Like rust-analyzer, we use not `pos.pos < self.end` but `pos.pos <= self.end`, because VSCode sometimes sends a position that is one character beyond the end of the symbol string.
        self.start <= pos.pos && pos.pos <= self.end
    }
}
