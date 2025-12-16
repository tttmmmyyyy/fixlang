// Utility functions for editing text files, particularly for applying text edits.

use lsp_types::TextEdit;

/// Apply a list of TextEdits to a string content.
/// Edits are applied in reverse order of their positions to maintain correct offsets.
pub fn apply_text_edits(content: &str, edits: &[TextEdit]) -> String {
    if edits.is_empty() {
        return content.to_string();
    }

    // Sort edits by position (end position first, then start position) in reverse order.
    let mut sorted_edits = edits.to_vec();
    sorted_edits.sort_by(|a, b| {
        b.range
            .end
            .line
            .cmp(&a.range.end.line)
            .then_with(|| b.range.end.character.cmp(&a.range.end.character))
            .then_with(|| b.range.start.line.cmp(&a.range.start.line))
            .then_with(|| b.range.start.character.cmp(&a.range.start.character))
    });

    let lines: Vec<&str> = content.lines().collect();
    let mut result = content.to_string();

    // Apply edits in reverse order.
    for edit in sorted_edits {
        let start_line = edit.range.start.line as usize;
        let start_char = edit.range.start.character as usize;
        let end_line = edit.range.end.line as usize;
        let end_char = edit.range.end.character as usize;

        // Calculate byte offsets.
        let mut start_offset = 0;
        for i in 0..start_line {
            if i < lines.len() {
                start_offset += lines[i].len() + 1; // +1 for newline
            }
        }
        if start_line < lines.len() {
            start_offset += lines[start_line]
                .chars()
                .take(start_char)
                .map(|c| c.len_utf8())
                .sum::<usize>();
        }

        let mut end_offset = 0;
        for i in 0..end_line {
            if i < lines.len() {
                end_offset += lines[i].len() + 1; // +1 for newline
            }
        }
        if end_line < lines.len() {
            end_offset += lines[end_line]
                .chars()
                .take(end_char)
                .map(|c| c.len_utf8())
                .sum::<usize>();
        }

        result.replace_range(start_offset..end_offset, &edit.new_text);
    }

    result
}

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range, TextEdit};

    use crate::edit::edit_util::apply_text_edits;

    fn make_edit(
        start_line: u32,
        start_char: u32,
        end_line: u32,
        end_char: u32,
        new_text: &str,
    ) -> TextEdit {
        TextEdit {
            range: Range {
                start: Position {
                    line: start_line,
                    character: start_char,
                },
                end: Position {
                    line: end_line,
                    character: end_char,
                },
            },
            new_text: new_text.to_string(),
        }
    }

    #[test]
    fn test_empty_edits() {
        let content = "Hello, world!";
        let edits = vec![];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_single_line_replacement() {
        let content = "Hello, world!";
        let edits = vec![make_edit(0, 7, 0, 12, "Rust")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello, Rust!");
    }

    #[test]
    fn test_multiple_edits_on_same_line() {
        let content = "foo bar baz";
        let edits = vec![
            make_edit(0, 0, 0, 3, "FOO"),  // Replace "foo" with "FOO"
            make_edit(0, 8, 0, 11, "BAZ"), // Replace "baz" with "BAZ"
        ];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "FOO bar BAZ");
    }

    #[test]
    fn test_multiline_replacement() {
        let content = "line1\nline2\nline3";
        let edits = vec![make_edit(0, 5, 2, 0, "\ninserted\n")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "line1\ninserted\nline3");
    }

    #[test]
    fn test_insertion() {
        let content = "Hello world!";
        let edits = vec![make_edit(0, 5, 0, 5, ",")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_deletion() {
        let content = "Hello,, world!";
        let edits = vec![make_edit(0, 6, 0, 7, "")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_utf8_characters() {
        let content = "こんにちは世界";
        let edits = vec![make_edit(0, 5, 0, 7, "地球")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "こんにちは地球");
    }

    #[test]
    fn test_multiple_lines_with_edits() {
        let content = "line 1\nline 2\nline 3";
        let edits = vec![
            make_edit(0, 5, 0, 6, "A"), // Change "1" to "A"
            make_edit(2, 5, 2, 6, "C"), // Change "3" to "C"
        ];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "line A\nline 2\nline C");
    }

    #[test]
    fn test_append_at_end() {
        let content = "Hello";
        let edits = vec![make_edit(0, 5, 0, 5, " world!")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_insert_at_beginning() {
        let content = "world!";
        let edits = vec![make_edit(0, 0, 0, 0, "Hello ")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_replace_entire_content() {
        let content = "old content";
        let edits = vec![make_edit(0, 0, 0, 11, "new content")];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "new content");
    }

    #[test]
    fn test_overlapping_edits_order() {
        // Test that edits are applied in the correct order (reverse by position)
        let content = "abcdef";
        let edits = vec![
            make_edit(0, 0, 0, 3, "XYZ"), // Replace "abc" with "XYZ"
            make_edit(0, 3, 0, 6, "123"), // Replace "def" with "123"
        ];
        let result = apply_text_edits(content, &edits);
        assert_eq!(result, "XYZ123");
    }
}
