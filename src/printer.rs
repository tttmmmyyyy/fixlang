use std::mem::take;

pub struct Line {
    content: String,
    indent: u32,
}

impl Line {
    pub fn new(content: String, indent: u32) -> Line {
        Line { content, indent }
    }

    pub fn from_string(content: String) -> Line {
        Line::new(content, 0)
    }
}

pub struct Text {
    pub lines: Vec<Line>,
}

impl Default for Text {
    fn default() -> Self {
        Text::empty()
    }
}

impl Text {
    pub fn to_string(&self) -> String {
        self.lines
            .iter()
            .map(|line| {
                let indent = " ".repeat((line.indent * 4) as usize);
                format!("{}{}", indent, line.content)
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn empty() -> Self {
        Text { lines: Vec::new() }
    }

    pub fn line(line: Line) -> Self {
        Text { lines: vec![line] }
    }

    pub fn from_string(content: String) -> Self {
        Text::line(Line::from_string(content))
    }

    pub fn from_str(content: &str) -> Self {
        Text::from_string(content.to_string())
    }

    pub fn first_line_mut(&mut self) -> &mut String {
        &mut self.lines[0].content
    }

    pub fn last_line_mut(&mut self) -> &mut String {
        let last_index = self.lines.len() - 1;
        &mut self.lines[last_index].content
    }

    pub fn insert_to_first_line(mut self, string: &str) -> Text {
        self.first_line_mut().insert_str(0, string);
        self
    }

    pub fn append_to_last_line(mut self, string: &str) -> Text {
        self.last_line_mut().push_str(string);
        self
    }

    pub fn num_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn append(mut self, mut other: Text) -> Text {
        self.lines.append(&mut other.lines);
        self
    }

    // Append `other` to `self`.
    //
    // If the first line of `other` has no indent, this function appends `other` to the last line of `self`.
    pub fn append_nobreak(mut self: Text, other: Text) -> Text {
        if other.num_lines() == 0 {
            return self;
        }
        for (i, line) in other.lines.into_iter().enumerate() {
            if i == 0 && line.indent == 0 {
                self = self.append_to_last_line(&line.content);
            } else {
                self.lines.push(line);
            }
        }
        self
    }

    pub fn append_front(self, other: Text) -> Text {
        other.append(self)
    }

    pub fn add_indent(mut self, indent: u32) -> Text {
        for line in self.lines.iter_mut() {
            line.indent += indent;
        }
        self
    }

    // Join a list of texts with a separator.
    //
    // If consecutive texts A, B in `texts` satisfies the any of following conditions, this function puts a line break between A (+ sep) and B.
    // - Any of A and B has multiple lines.
    // - Any of A and B has a non-zero indent.
    pub fn join(mut texts: Vec<Text>, sep: &str) -> Text {
        // If `texts` is empty or has only one element, return it as is.
        if texts.is_empty() {
            return Text::empty();
        }
        if texts.len() <= 1 {
            return take(&mut texts[0]);
        }

        // A function to check if a text has multiple lines or a non-zero indent.
        fn should_break(text: &Text) -> bool {
            text.num_lines() > 1 || text.lines.iter().any(|line| line.indent > 0)
        }

        let mut res = Text::empty();
        let mut prev_should_break = false;
        for (i, text) in texts.into_iter().enumerate() {
            if i == 0 {
                prev_should_break = should_break(&text);
                res = text;
            } else {
                res = res.append_to_last_line(sep);

                let this_should_break = should_break(&text);
                let should_break = prev_should_break || this_should_break;
                prev_should_break = this_should_break;

                if should_break {
                    res = res.append(text);
                } else {
                    res = res.append_to_last_line(&text.lines[0].content);
                }
            }
        }
        res
    }

    // Add braces to the text.
    //
    // If the text has only one line, this function adds braces to the line.
    // Otherwise, this function adds braces to the first and last lines of the text, and indents all lines in the text.
    fn brace_inner(self, open: &str, close: &str) -> Text {
        if self.num_lines() == 0 {
            Text::from_str(&format!("{}{}", open, close))
        } else if self.num_lines() == 1 {
            self.insert_to_first_line(open).append_to_last_line(close)
        } else {
            self.add_indent(1)
                .append_front(Text::from_str(open))
                .append(Text::from_str(close))
        }
    }

    pub fn brace(self) -> Text {
        self.brace_inner("(", ")")
    }

    pub fn brace_if_multiline(self) -> Text {
        if self.num_lines() > 1 {
            return self.brace();
        }
        self
    }

    pub fn curly_brace(self) -> Text {
        self.brace_inner("{", "}")
    }

    pub fn square_brace(self) -> Text {
        self.brace_inner("[", "]")
    }
}
