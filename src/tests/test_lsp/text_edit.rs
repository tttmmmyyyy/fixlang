//! The text edits a language server answers with, read from its JSON and applied to a text.

use lsp_types::{Position, TextEdit};
use serde_json::Value;
use std::cmp::Reverse;

/// Parse a list of LSP `TextEdit` JSON values into `lsp_types::TextEdit`s.
pub fn parse_text_edits(edits: &[Value]) -> Vec<TextEdit> {
    edits
        .iter()
        .map(|e| serde_json::from_value(e.clone()).expect("Failed to parse TextEdit"))
        .collect()
}

/// `content` with `edits` applied the way an editor applies them: each position's column is
/// counted in UTF-16 code units.
pub fn apply_lsp_edits(content: &str, edits: &[TextEdit]) -> String {
    let offset_of = |pos: Position| {
        let line_start: usize = content
            .split_inclusive('\n')
            .take(pos.line as usize)
            .map(str::len)
            .sum();
        let mut units = 0;
        let column = content[line_start..]
            .char_indices()
            .find(|&(_, c)| {
                let reached = units >= pos.character as usize;
                units += c.len_utf16();
                reached
            })
            .map_or(content.len() - line_start, |(i, _)| i);
        line_start + column
    };
    let mut edits = edits.to_vec();
    edits.sort_by_key(|edit| Reverse(offset_of(edit.range.start)));
    let mut result = content.to_string();
    for edit in edits {
        let (from, to) = (offset_of(edit.range.start), offset_of(edit.range.end));
        result.replace_range(from..to, &edit.new_text);
    }
    result
}
