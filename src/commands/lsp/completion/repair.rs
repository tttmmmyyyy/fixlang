// Source-text repair pre-pass used by dot-completion. Given the live
// editor buffer and the cursor position, produce a syntactically valid
// (or close-to-valid) source string that the parser can accept and a
// hole node can be located in.
//
// Two stages:
//   A0     — replace the post-dot identifier the cursor is in with `?`
//   outer  — pest error-driven outer-source repair loop, splicing in
//            `;` or `?` to satisfy the surrounding grammar
//
// Both stages run unconditionally; outer repair is a no-op when A0's
// output already parses.

use crate::parse::parser::{probe_parse_for_completion_repair, RepairHintKind};

/// Insertion candidates for the outer-repair loop, ranked by how
/// often they appear in the failure modes we want to recover from.
///
/// The order matters: when the classified `RepairHintKind` is
/// `Unknown` (= pest didn't surface a named token we recognise), we
/// try each in turn and keep the first one whose parse either fully
/// succeeds or moves the failure point forward.
///
/// `{};` covers the `if cond <missing>` / `match e <missing>` shape,
/// where pest's `positives` only mention more-expression rules
/// (`arg_list`, `operator_*`, …) instead of the literal `{` it
/// actually wants. `}` / `)` / `]` cover the closing-bracket cases
/// pest likewise can't surface as a positive — the inline `"}"` /
/// `")"` / `"]"` literals in the grammar aren't pest `Rule`s, so they
/// never appear in the positive set.
const FALLBACK_INSERTIONS: &[&str] = &["{};", "{}", ")", "}", "]", ";", ",", "?"];

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
pub(super) fn repair_for_completion(live_buffer: &str, cursor_byte: usize) -> Option<RepairOutput> {
    let RepairOutput {
        source,
        cursor_byte,
    } = apply_a0(live_buffer, cursor_byte)?;
    apply_outer_repair(source, cursor_byte)
}

/// Bound on the number of pest-error-driven splices the outer-repair
/// loop will attempt before giving up. 8 is enough to fix several
/// layered `let ... ;` / unclosed-call shapes while still terminating
/// quickly on hopeless inputs.
const MAX_ATTEMPTS: usize = 8;

/// Run the pest-error-driven splice loop on `source`, advancing
/// `cursor_byte` whenever an insertion lands before it. Returns the
/// repaired source on success, or `None` if no progress can be made
/// within `MAX_ATTEMPTS` iterations.
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
                // Build an ordered candidate list:
                //   - the splice the classifier suggests (`;` or `?`),
                //     adjusted by the let-body anti-loop heuristic;
                //   - then the structural fallbacks for cases pest
                //     can't surface as positives (closing brackets,
                //     `if`/`match` bodies — see `FALLBACK_INSERTIONS`).
                // `pick_insertion` then trial-parses each in order and
                // keeps the first that succeeds or makes the parse
                // failure move forward.
                let mut primary = match hint.kind {
                    RepairHintKind::Semicolon => Some(";"),
                    RepairHintKind::Expression => Some("?"),
                    RepairHintKind::Unknown => None,
                };
                if let (Some(p), Some((prev_pos, prev_str))) = (primary, prev_insertion) {
                    // Anti-loop: when the previous splice was `;` and
                    // pest is asking for another `;` at the immediately
                    // adjacent byte, the let-rule's `+` is eating
                    // semicolons without ever finding a body. Force
                    // `?` for one round so the body slot fills.
                    if prev_str == ";" && p == ";" && hint.insert_at == prev_pos + 1 {
                        primary = Some("?");
                    }
                }
                let mut candidates: Vec<&'static str> = Vec::new();
                if let Some(c) = primary {
                    candidates.push(c);
                }
                for c in FALLBACK_INSERTIONS {
                    if !candidates.contains(c) {
                        candidates.push(*c);
                    }
                }

                let pos = hint.insert_at.min(source.len());
                let chosen = pick_insertion(&source, pos, &candidates)?;
                source.insert_str(pos, chosen);
                // Strict `<` so an insertion *at* the cursor sticks to
                // its right; the cursor keeps marking the byte right
                // after the A0 hole (the anchor the hole-finder uses).
                if pos < cursor_byte {
                    cursor_byte += chosen.len();
                }
                prev_insertion = Some((pos, chosen));
            }
        }
    }
    None
}

/// Trial-insert each candidate at `pos` and return the one whose
/// resulting source either parses cleanly or moves the parser's
/// failure position the furthest forward. `None` if no candidate
/// makes progress.
fn pick_insertion(source: &str, pos: usize, candidates: &[&'static str]) -> Option<&'static str> {
    let mut best_full: Option<&'static str> = None;
    let mut best_progress: Option<(&'static str, usize)> = None;
    for cand in candidates {
        let mut trial = String::with_capacity(source.len() + cand.len());
        trial.push_str(&source[..pos]);
        trial.push_str(cand);
        trial.push_str(&source[pos..]);
        match probe_parse_for_completion_repair(&trial) {
            Ok(()) => {
                if best_full.is_none() {
                    best_full = Some(*cand);
                }
            }
            Err(new_hint) => {
                // Translate the new error position back to pre-
                // insertion source coordinates. Errors that fall
                // inside the inserted text count as "no progress".
                if new_hint.insert_at >= pos + cand.len() {
                    // `np == pos` is still progress: pest accepted the
                    // inserted text and stopped at the byte immediately
                    // after it. The let-body recovery relies on this
                    // (insert `;`, then on the next iteration insert
                    // `?` to fill the body).
                    let np = new_hint.insert_at - cand.len();
                    if best_progress.map_or(true, |(_, p)| np > p) {
                        best_progress = Some((*cand, np));
                    }
                }
            }
        }
    }
    best_full.or(best_progress.map(|(c, _)| c))
}

/// A0: locate the post-dot identifier and replace it with `?`.
///
/// The cursor anchor returned points at the byte immediately after
/// the inserted `?`, regardless of where the user's actual cursor
/// landed (in the partial id, between dot and id, before whitespace,
/// etc.).
///
/// Fix's `expr_dot_seq` allows `sep*` (whitespace including newlines)
/// on both sides of the dot, so the search walks across whitespace in
/// both directions:
///
///   - back from the cursor: any name chars (the partial id), then
///     whitespace, then the dot we anchor on
///   - forward from the dot: any whitespace, then the name chars that
///     form the post-dot identifier (which might be on a later line)
fn apply_a0(live_buffer: &str, cursor_byte: usize) -> Option<RepairOutput> {
    let cursor_byte = cursor_byte.min(live_buffer.len());
    let bytes = live_buffer.as_bytes();

    // Walk back from the cursor through name chars (any partial id
    // the user has already typed past the dot).
    let mut back = cursor_byte;
    while back > 0 && is_name_char(bytes[back - 1]) {
        back -= 1;
    }
    // Skip whitespace.
    while back > 0 && is_ascii_whitespace(bytes[back - 1]) {
        back -= 1;
    }
    if back == 0 || bytes[back - 1] != b'.' {
        return None;
    }
    let dot_pos = back - 1;
    if dot_pos == 0 {
        return None;
    }

    // Reject the dot in a numeric literal (`1.0`): unambiguously
    // numeric only when both sides of the dot are digits. `42.foo`,
    // `42.<cursor>`, `42.\n  pure` are dot syntax even though the
    // receiver happens to be a digit.
    let pre_dot = bytes[dot_pos - 1];
    let post_dot_byte = bytes.get(dot_pos + 1).copied();
    if pre_dot.is_ascii_digit() && post_dot_byte.map_or(false, |b| b.is_ascii_digit()) {
        return None;
    }

    // Walk forward from the dot through whitespace, then through name
    // chars, to find the full post-dot identifier the user wrote
    // (possibly on a later line).
    let mut id_start = dot_pos + 1;
    while id_start < bytes.len() && is_ascii_whitespace(bytes[id_start]) {
        id_start += 1;
    }
    let mut id_end = id_start;
    while id_end < bytes.len() && is_name_char(bytes[id_end]) {
        id_end += 1;
    }

    // Replace [id_start..id_end] with `?`. Whitespace between the dot
    // and the id stays in place — `obj.\n    ?` parses the same as
    // `obj.?` and preserving the indentation keeps every other span
    // unchanged.
    let mut out = String::with_capacity(live_buffer.len() + 1);
    out.push_str(&live_buffer[..id_start]);
    out.push('?');
    out.push_str(&live_buffer[id_end..]);

    Some(RepairOutput {
        source: out,
        cursor_byte: id_start + 1,
    })
}

/// True for bytes that pest's `name_char` rule accepts as a
/// continuation of an identifier: ASCII letters, digits, and `_`.
/// (`@` is only valid as a name *head*, not a continuation, so it
/// isn't part of the post-dot identifier we're replacing.)
fn is_name_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// True for ASCII whitespace bytes — spaces, tabs, CR, LF, vertical
/// tab, form feed. Matches what pest's `sep` rule covers in terms of
/// straight whitespace; comments aren't handled here, so A0 bails
/// when the surrounding shape is unusual.
fn is_ascii_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
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

    #[test]
    fn a0_accepts_digit_dot_when_post_dot_is_not_digit() {
        // `42.foo` is dot syntax (`App(foo, [42])`), not a numeric
        // literal. The earlier "reject any digit before the dot"
        // check was too strict for receivers that happen to be
        // integer literals.
        let src = "    42.";
        let cursor = src.len();
        let out = apply_a0(src, cursor).expect("digit-before-dot is OK when no digit follows");
        assert_eq!(out.source, "    42.?");
    }

    #[test]
    fn a0_replaces_post_dot_id_across_newline() {
        // Cursor right after the dot on one line; the post-dot id
        // lives on the next line. Fix's `expr_dot_seq` allows `sep*`
        // between the dot and the next component, so this is `42.pure`
        // to the parser, and A0 should replace `pure` with `?`.
        let src = "    42.\n    pure()";
        let cursor = "    42.".len();
        let out = apply_a0(src, cursor).expect("multi-line post-dot id");
        assert_eq!(out.source, "    42.\n    ?()");
        // Cursor anchor lands just after the inserted `?`.
        assert_eq!(
            out.source.as_bytes()[out.cursor_byte - 1],
            b'?',
            "cursor should land on the inserted `?`; got source = {:?}, byte = {}",
            out.source,
            out.cursor_byte
        );
    }

    /// Cursor inside an `if` body that itself contains a dot
    /// expression: `if 42.myfunc2(7) { 42.<cursor> } pure()`. A0
    /// should fire on the inner `42.` and the surrounding
    /// `if cond { body }` shape (no `else`, body terminated by the
    /// outer `pure()` expression) should still let outer repair
    /// produce a parseable program.
    #[test]
    fn outer_repair_handles_dot_inside_if_body_with_outer_else() {
        let src = "module Main;\n\
                   myfunc2 : I64 -> I64 -> I64 = |x, y| x + y;\n\
                   main : IO () = (\n\
                   \x20   if 42.myfunc2(7) {\n\
                   \x20       42.\n\
                   \x20   }\n\
                   \x20   pure()\n\
                   );";
        // Cursor right after the inner `42.` dot.
        let prefix = "module Main;\n\
                      myfunc2 : I64 -> I64 -> I64 = |x, y| x + y;\n\
                      main : IO () = (\n\
                      \x20   if 42.myfunc2(7) {\n\
                      \x20       42.";
        let cursor = prefix.len();
        match repair_for_completion(src, cursor) {
            Some(out) => {
                assert_eq!(
                    out.source.as_bytes()[out.cursor_byte - 1],
                    b'?',
                    "cursor anchor should mark the A0 hole; source = {:?}, byte = {}",
                    out.source,
                    out.cursor_byte
                );
                assert!(
                    probe_parse_for_completion_repair(&out.source).is_ok(),
                    "repaired source should parse; got: {:?}",
                    out.source
                );
            }
            None => panic!(
                "repair_for_completion returned None for nested if-body case; \
                 src = {:?}",
                src
            ),
        }
    }

    /// `if 42.<cursor>\n    pure()` inside a tuple-style body. After
    /// A0 the post-dot identifier `pure` becomes `?`, leaving an
    /// `if cond` shape with no body braces. Pest can't surface the
    /// missing `{` as a positive (the `"{"` literal in `expr_if`
    /// isn't a named `Rule`), so `classify_repair_hint` returns
    /// `Unknown` and the trial-insertion fallback in `pick_insertion`
    /// has to try `{};` to make the parser succeed.
    #[test]
    fn outer_repair_handles_if_body_missing_braces() {
        let src = "module Main;\nmain : IO () = (\n    if 42.\n    pure()\n);";
        // Cursor right after the dot (= where the user typed `.` to
        // open dot-completion).
        let cursor = "module Main;\nmain : IO () = (\n    if 42.".len();
        let out = repair_for_completion(src, cursor)
            .expect("outer repair should succeed for `if cond.<cursor>` shape");
        // Cursor anchor still points at the `?` A0 inserted; the
        // hole-finder relies on this.
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
