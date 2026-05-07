// Source-text repair pre-pass used by dot-completion. Given the live
// editor buffer and the cursor position, produce a syntactically valid
// (or close-to-valid) source string that the parser can accept and a
// hole node can be located in.
//
// Two-stage shape (plan §A.4):
//   A0 — replace the post-dot identifier the cursor is in with `?`
//   A.4.2 — pest error-driven outer-source repair loop, splicing in
//           `;` or `?` to satisfy the surrounding grammar
//
// Both stages run unconditionally; outer repair is a no-op when A0's
// output already parses.

use crate::parse::parser::{
    probe_parse_for_completion_repair, RepairHint, RepairHintKind,
};

/// Result of `repair_for_completion`: the rewritten source plus the
/// adjusted byte offset that points to the hole the post-dot identifier
/// was rewritten into. The cursor offset is a byte offset into the
/// returned `String`, not into the original `live_buffer`.
pub(super) struct RepairOutput {
    pub source: String,
    pub cursor_byte: usize,
}

/// Repair `live_buffer` so the parser can chew through it and the
/// downstream hole-finder has a `Std::#hole` node at the cursor.
///
/// 1. Apply A0 — replace the post-dot identifier containing or ending
///    at the cursor with `?`.
/// 2. Loop: try to parse, and on each parse error splice in a `;` /
///    `?` per `RepairHint`. Bounded to `MAX_ATTEMPTS` iterations to
///    keep pathological inputs from looping forever.
///
/// Returns `None` (silent fallback to alphabetical-order completion)
/// when the cursor isn't in a `<id>.<post-dot>` shape, or the outer
/// repair can't make progress within the iteration budget.
pub(super) fn repair_for_completion(
    live_buffer: &str,
    cursor_byte: usize,
) -> Option<RepairOutput> {
    let RepairOutput {
        source,
        cursor_byte,
    } = apply_a0(live_buffer, cursor_byte)?;
    apply_outer_repair(source, cursor_byte)
}

/// Bound on the number of pest-error-driven splices the outer-repair
/// loop will attempt before giving up. Plan §A.4.2 sets it at 8 —
/// enough to fix several layered `let ... ;` / unclosed-call shapes
/// while still terminating quickly on hopeless inputs.
const MAX_ATTEMPTS: usize = 8;

fn apply_outer_repair(mut source: String, mut cursor_byte: usize) -> Option<RepairOutput> {
    let mut prev_insertion: Option<(usize, &'static str)> = None;
    for _ in 0..MAX_ATTEMPTS {
        match probe_parse_for_completion_repair(&source) {
            Ok(()) => {
                return Some(RepairOutput {
                    source,
                    cursor_byte,
                });
            }
            Err(hint) => {
                // Pick the splice based on the hint, with one
                // recovery: if the last splice was a `;` and pest now
                // wants another `;` at an adjacent byte (the source
                // grew by exactly one), the parser's `let` rule is
                // greedily eating semicolons without ever finding a
                // body. Insert `?` instead so the body slot fills and
                // the surrounding `;` can finally close the
                // definition.
                let mut inserted = match decide_insertion(&hint) {
                    Some(s) => s,
                    None => return None,
                };
                let pos = hint.insert_at.min(source.len());
                if let Some((prev_pos, prev_str)) = prev_insertion {
                    if prev_str == ";" && inserted == ";" && pos == prev_pos + 1 {
                        inserted = "?";
                    }
                }
                source.insert_str(pos, inserted);
                // Strict `<` so that an insertion *at* the cursor
                // sticks to its right; the cursor keeps marking the
                // byte immediately after the A0 hole (which is the
                // anchor the hole-finder uses).
                if pos < cursor_byte {
                    cursor_byte += inserted.len();
                }
                prev_insertion = Some((pos, inserted));
            }
        }
    }
    None
}

fn decide_insertion(hint: &RepairHint) -> Option<&'static str> {
    match hint.kind {
        RepairHintKind::Semicolon => Some(";"),
        RepairHintKind::Expression => Some("?"),
        RepairHintKind::Unknown => None,
    }
}

/// A0: replace the post-dot identifier the cursor is in with a single
/// `?` hole. Returns `None` if the cursor isn't in a
/// `<identifier>.<post-dot>` shape.
fn apply_a0(live_buffer: &str, cursor_byte: usize) -> Option<RepairOutput> {
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
    if !pre_dot.is_ascii_alphabetic()
        && pre_dot != b'_'
        && pre_dot != b'@'
        && pre_dot != b')'
        && pre_dot != b']'
    {
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
        let out = apply_a0(src, cursor).unwrap();
        assert_eq!(out.source, "let y = arr.?");
        // cursor lands right after the `?`
        assert_eq!(out.cursor_byte, "let y = arr.?".len());
    }

    #[test]
    fn a0_inserts_hole_when_cursor_is_immediately_after_dot() {
        let src = "let y = arr.";
        let cursor = src.len();
        let out = apply_a0(src, cursor).unwrap();
        assert_eq!(out.source, "let y = arr.?");
        assert_eq!(out.cursor_byte, "let y = arr.?".len());
    }

    #[test]
    fn a0_drops_suffix_after_cursor() {
        // "obj.foobar" with cursor between "foo" and "bar" → "obj.?"
        let src = "obj.foobar";
        let cursor = "obj.foo".len();
        let out = apply_a0(src, cursor).unwrap();
        assert_eq!(out.source, "obj.?");
    }

    #[test]
    fn a0_returns_none_without_dot() {
        let src = "let y = arr";
        assert!(apply_a0(src, src.len()).is_none());
    }

    #[test]
    fn a0_rejects_numeric_dot() {
        // "1.0" at cursor right after the dot — `1` is digit, not an
        // identifier tail, so this isn't a dot-completion site.
        let src = "let y = 1.0";
        let cursor = "let y = 1.".len();
        assert!(apply_a0(src, cursor).is_none());
    }

    #[test]
    fn a0_accepts_dot_after_paren() {
        // "(f x).foo" — `)` is allowed before the dot.
        let src = "(f x).foo";
        let cursor = src.len();
        let out = apply_a0(src, cursor).unwrap();
        assert_eq!(out.source, "(f x).?");
    }

    /// `let y = arr.<cursor>` at file scope — A0 makes the cursor area
    /// `let y = arr.?`. Without outer repair, this fails to parse: the
    /// surrounding `let` needs a body. The pest-driven loop should
    /// splice in characters until a complete program is recovered, and
    /// the cursor should still land on the A0 hole afterwards.
    #[test]
    fn outer_repair_handles_unfinished_let_body() {
        let src = "module Main;\nmain : ();\nmain = let y = arr.";
        let cursor = src.len();
        let out = repair_for_completion(src, cursor).expect("outer repair should succeed");
        assert_eq!(
            out.source.as_bytes()[out.cursor_byte - 1],
            b'?',
            "cursor should land on the A0 hole; got source = {:?}, byte = {}",
            out.source,
            out.cursor_byte
        );
        assert!(
            probe_parse_for_completion_repair(&out.source).is_ok(),
            "repaired source should parse; got: {:?}",
            out.source
        );
    }

    /// Trailing comma right before `)` — A0 makes `f(arr.<cursor>, )`
    /// into `f(arr.?, )`. The parser needs an expression after the
    /// comma; the loop should splice in a `?`.
    #[test]
    fn outer_repair_handles_trailing_comma_in_arg_list() {
        let src = "module Main;\nmain : ();\nmain = f(arr., );";
        // Cursor immediately after the dot of `arr.`
        let cursor = "module Main;\nmain : ();\nmain = f(arr.".len();
        let out = repair_for_completion(src, cursor).expect("outer repair should succeed");
        assert!(
            probe_parse_for_completion_repair(&out.source).is_ok(),
            "repaired source should parse; got: {:?}",
            out.source
        );
        assert_eq!(out.source.as_bytes()[out.cursor_byte - 1], b'?');
    }
}
