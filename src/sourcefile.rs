use std::{path::PathBuf, rc::Rc};

use pest::iterators::Pair;
use serde::{Deserialize, Serialize};

use crate::{parser::Rule, runner::read_file};

#[derive(Clone, Serialize, Deserialize)]
pub struct SourceFile {
    #[serde(skip)]
    pub string: Option<Rc<String>>,
    #[serde(skip)]
    pub hash: Option<String>,
    pub file_path: PathBuf,
}

impl SourceFile {
    pub fn string(&self) -> String {
        match &self.string {
            Some(s) => s.as_str().to_string(),
            None => match read_file(&self.file_path) {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            },
        }
    }

    pub fn from_file_path(file_path: PathBuf) -> Self {
        let mut src_file = Self {
            string: None,
            hash: None,
            file_path,
        };
        src_file.read_file();
        src_file.set_hash();
        src_file
    }

    // Set the values of uninitialized fields.
    pub fn read_file(&mut self) {
        if self.string.is_none() {
            self.string = match read_file(&self.file_path) {
                Ok(source) => Option::Some(Rc::new(source)),
                Err(e) => panic!("{}", e),
            };
        }
    }

    pub fn set_hash(&mut self) {
        if self.hash.is_some() {
            return;
        }
        self.hash = Option::Some(format!("{:x}", md5::compute(self.string())));
    }

    pub fn hash(&self) -> String {
        match &self.hash {
            Some(h) => h.clone(),
            None => {
                format!("{:x}", md5::compute(self.string()))
            }
        }
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

    pub fn to_single_character(&self) -> Self {
        Self {
            input: self.input.clone(),
            start: self.start,
            end: self.start + 1,
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

    // Get line and column number of start.
    pub fn start_line_col(&self) -> (usize, usize) {
        let source_string = self.input.string();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();
        span.start_pos().line_col()
    }

    // Show source codes around this span.
    pub fn to_string(&self) -> String {
        let source_string = self.input.string();
        let span = pest::Span::new(&source_string, self.start, self.end).unwrap();

        let mut linenum_str_size = 0;
        for line_span in span.lines_span() {
            let linenum = line_span.start_pos().line_col().0;
            linenum_str_size = linenum_str_size.max(linenum.to_string().len());
        }

        let mut ret: String = String::default();
        ret += &format!(
            "At {}:{}-{}:{} in \"{}\", \n",
            span.start_pos().line_col().0,
            span.start_pos().line_col().1,
            span.end_pos().line_col().0,
            span.end_pos().line_col().1,
            self.input.file_path.to_str().unwrap().to_string()
        );
        ret += &(" ".repeat(linenum_str_size) + " | " + "\n");
        for line_span in span.lines_span() {
            let linenum_str = line_span.start_pos().line_col().0.to_string();
            ret +=
                &(linenum_str.clone() + &" ".repeat(linenum_str_size - linenum_str.len()) + " | ");
            ret += String::from(line_span.as_str()).trim_end();
            ret += "\n";
            ret += &(" ".repeat(linenum_str_size) + " | ");
            let start_pos = span.start_pos().max(line_span.start_pos());
            let end_pos = span.end_pos().min(line_span.end_pos());
            let start_col = start_pos.line_col().1;
            let span_len = end_pos.pos() - start_pos.pos();
            ret += &(" ".repeat(start_col - 1) + &"^".repeat(span_len));
            ret += "\n";
        }
        ret
    }
}
