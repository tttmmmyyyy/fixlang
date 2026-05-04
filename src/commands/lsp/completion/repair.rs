// Source-text repair pre-pass used by dot-completion. Given the live
// editor buffer and the cursor position, produce a syntactically valid
// (or close-to-valid) source string that the parser can accept and a
// hole node can be located in.
//
// See `logs/lsp-completion-type-filter.20260503/plan.md` §A.4 for the
// design.

/// Result of `repair_for_completion`: the rewritten source plus the
/// adjusted byte offset that points to the hole the post-dot identifier
/// was rewritten into. The cursor offset is a byte offset into the
/// returned `String`, not into the original `live_buffer`.
pub(super) struct RepairOutput {
    pub source: String,
    pub cursor_byte: usize,
}

/// Apply the A0 step of the repair: replace the post-dot identifier
/// containing or ending at the cursor with a single `?` hole. Returns
/// `None` if the cursor isn't in a `<identifier>.<post-dot>` shape.
///
/// **Step 1 implementation status**: only A0 is performed; the pest
/// error-driven outer-repair loop (A.4.2) lands in Step 4.
pub(super) fn repair_for_completion(
    live_buffer: &str,
    cursor_byte: usize,
) -> Option<RepairOutput> {
    let cursor_byte = cursor_byte.min(live_buffer.len());
    let bytes = live_buffer.as_bytes();

    // Walk back from the cursor to find the dot that introduces the
    // post-dot identifier. Skip identifier-character bytes (the partial
    // name the user is in the middle of typing) and stop at the dot.
    let mut i = cursor_byte;
    while i > 0 && is_name_char(bytes[i - 1]) {
        i -= 1;
    }
    if i == 0 || bytes[i - 1] != b'.' {
        return None;
    }
    // `i` is now the byte right after the dot — the start of the
    // post-dot identifier (possibly empty).
    //
    // Reject dots that belong to a numeric literal (e.g. `1.0`): the
    // byte before the dot must look like the tail of a real identifier,
    // not a digit. `_` and `@` are allowed because they're identifier
    // continuations.
    let dot_pos = i - 1;
    if dot_pos == 0 {
        return None;
    }
    let pre_dot = bytes[dot_pos - 1];
    if !pre_dot.is_ascii_alphabetic() && pre_dot != b'_' && pre_dot != b'@' && pre_dot != b')' && pre_dot != b']' {
        return None;
    }

    // Extend the post-dot identifier through any trailing identifier
    // characters past the cursor (e.g. `obj.foo<cursor>bar` → drop
    // `bar` along with `foo`).
    let mut end = cursor_byte;
    while end < bytes.len() && is_name_char(bytes[end]) {
        end += 1;
    }

    // Build the repaired source: prefix + "?" + suffix.
    let mut out = String::with_capacity(live_buffer.len() + 1);
    out.push_str(&live_buffer[..i]);
    out.push('?');
    out.push_str(&live_buffer[end..]);

    // Cursor lands right after the `?` we just inserted.
    Some(RepairOutput {
        source: out,
        cursor_byte: i + 1,
    })
}

fn is_name_char(b: u8) -> bool {
    // Mirror grammer.pest's `name_char = { ASCII_ALPHA | ASCII_DIGIT | "_" }`.
    // (`@` is only valid as a name *head*, not a continuation, so it isn't
    // part of the post-dot identifier we're replacing.)
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a0_replaces_cursor_at_end_of_post_dot_id() {
        // "let y = arr.foo" with cursor at end → "let y = arr.?"
        let src = "let y = arr.foo";
        let cursor = src.len();
        let out = repair_for_completion(src, cursor).unwrap();
        assert_eq!(out.source, "let y = arr.?");
        // cursor lands right after the `?`
        assert_eq!(out.cursor_byte, "let y = arr.?".len());
    }

    #[test]
    fn a0_inserts_hole_when_cursor_is_immediately_after_dot() {
        let src = "let y = arr.";
        let cursor = src.len();
        let out = repair_for_completion(src, cursor).unwrap();
        assert_eq!(out.source, "let y = arr.?");
        assert_eq!(out.cursor_byte, "let y = arr.?".len());
    }

    #[test]
    fn a0_drops_suffix_after_cursor() {
        // "obj.foobar" with cursor between "foo" and "bar" → "obj.?"
        let src = "obj.foobar";
        let cursor = "obj.foo".len();
        let out = repair_for_completion(src, cursor).unwrap();
        assert_eq!(out.source, "obj.?");
    }

    #[test]
    fn a0_returns_none_without_dot() {
        let src = "let y = arr";
        assert!(repair_for_completion(src, src.len()).is_none());
    }

    #[test]
    fn a0_rejects_numeric_dot() {
        // "1.0" at cursor right after the dot — `1` is digit, not an
        // identifier tail, so this isn't a dot-completion site.
        let src = "let y = 1.0";
        let cursor = "let y = 1.".len();
        assert!(repair_for_completion(src, cursor).is_none());
    }

    #[test]
    fn a0_accepts_dot_after_paren() {
        // "(f x).foo" — `)` is allowed before the dot.
        let src = "(f x).foo";
        let cursor = src.len();
        let out = repair_for_completion(src, cursor).unwrap();
        assert_eq!(out.source, "(f x).?");
    }
}
