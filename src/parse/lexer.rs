// A never-failing lexical scanner used by the LSP semantic-tokens feature.
//
// Syntax highlighting must work even while the buffer is syntactically broken
// (the user is in the middle of an edit). The real parser (`parse_source_file`)
// fails fast on the first error and yields no AST, so it cannot drive
// highlighting in that state. This module instead runs the additive
// `lex_tokens` grammar rule, which is written so it can never fail: it
// classifies the source into a flat token stream and leaves anything it does
// not recognize uncolored.
//
// All knowledge about *what* a token looks like lives in `grammer.pest`; this
// module only maps the recognized grammar rules onto a small set of highlight
// categories and applies two purely lexical refinements (namespace-vs-type and
// field accessors).

use crate::parse::parser::{FixParser, Rule};
use pest::Parser;

/// A highlight category for a single lexical token. Intentionally coarse:
/// these are the distinctions that can be drawn purely lexically, without an
/// AST or type information. Finer distinctions (function vs. variable,
/// type vs. trait) would require the elaborated program and are left to a
/// future AST-based overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexTokenKind {
    /// `//` line comment or `/* */` block comment.
    Comment,
    /// `"..."` string literal or `'...'` character literal.
    String,
    /// Numeric literal, including any `_I64`-style suffix.
    Number,
    /// A language keyword, or the `nullptr` constant.
    Keyword,
    /// A boolean literal (`true` / `false`).
    Boolean,
    /// A capitalized name used as a type / trait / constructor (the final
    /// segment of a `::`-qualified path, or a standalone capitalized name).
    Type,
    /// A capitalized name immediately followed by `::` — i.e. used as a
    /// namespace qualifier rather than naming the entity itself.
    Namespace,
    /// A lowercase / `_` / `@`-prefixed identifier (value or field name).
    ///
    /// Identifiers are NOT distinguished further at the lexical level: whether
    /// a name is a local, a global, a function or a value cannot be known from
    /// position alone (a positional guess colors the binder and the use of the
    /// same name differently). The semantic-tokens layer leaves these uncolored
    /// and lets an AST overlay classify them once elaboration succeeds.
    Variable,
    /// A field accessor: a getter/setter/modifier/act function (`@x`, `set_x`,
    /// `mod_x`, `act_x`) or an index-syntax field/tuple accessor (`^field`,
    /// `^0`). Recognized purely lexically, so it colors before type checking.
    Property,
    /// An operator or separator (`::`, `->`, `+`, `.`, ...).
    Operator,
}

/// Whether an identifier denotes a field accessor function by its spelling:
/// a getter (`@x`), setter (`set_x`), modifier (`mod_x`) or act (`act_x`).
/// Shared by the lexer and the semantic-tokens overlay so they agree on what
/// the base layer already colors as a property.
pub fn is_accessor_name(s: &str) -> bool {
    s.starts_with('@') || s.starts_with("set_") || s.starts_with("mod_") || s.starts_with("act_")
}

/// One lexical token: a half-open byte range `[start, end)` into the source
/// string and its highlight category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexToken {
    pub start: usize,
    pub end: usize,
    pub kind: LexTokenKind,
}

// Map a grammar rule (the inner rule of a `lex_token`) to a highlight
// category. `None` means "do not color" (whitespace and unrecognized
// punctuation, which the grammar matches via `lex_other`).
fn kind_of_rule(rule: Rule) -> Option<LexTokenKind> {
    match rule {
        Rule::lex_line_comment | Rule::lex_block_comment => Some(LexTokenKind::Comment),
        Rule::lex_string | Rule::lex_char => Some(LexTokenKind::String),
        Rule::expr_number_lit => Some(LexTokenKind::Number),
        Rule::expr_bool_lit => Some(LexTokenKind::Boolean),
        Rule::lex_keyword | Rule::expr_nullptr_lit => Some(LexTokenKind::Keyword),
        Rule::lex_field_accessor => Some(LexTokenKind::Property),
        Rule::capital_name => Some(LexTokenKind::Type),
        Rule::name => Some(LexTokenKind::Variable),
        Rule::lex_operator => Some(LexTokenKind::Operator),
        _ => None,
    }
}

/// Scan `content` into a flat stream of highlight tokens. Whitespace and
/// uncolored punctuation are omitted. The returned tokens are ordered by, and
/// do not overlap in, their source position. This function never fails: on the
/// (theoretically impossible) event that the scanner rule does not match, it
/// returns an empty vector rather than erroring.
pub fn lex_tokens(content: &str) -> Vec<LexToken> {
    let mut pairs = match FixParser::parse(Rule::lex_tokens, content) {
        Ok(pairs) => pairs,
        Err(_) => return vec![],
    };
    // `Rule::lex_tokens` always yields exactly one top pair.
    let Some(top) = pairs.next() else {
        return vec![];
    };

    let mut tokens = vec![];
    for lex_token in top.into_inner() {
        // Skip the trailing `EOI` pair produced by pest.
        if lex_token.as_rule() != Rule::lex_token {
            continue;
        }
        // `lex_token` wraps exactly one category rule.
        let Some(inner) = lex_token.into_inner().next() else {
            continue;
        };
        let Some(kind) = kind_of_rule(inner.as_rule()) else {
            continue;
        };
        let span = inner.as_span();
        tokens.push(LexToken {
            start: span.start(),
            end: span.end(),
            kind,
        });
    }

    refine_namespaces(content, &mut tokens);
    refine_accessors(content, &mut tokens);
    tokens
}

// Reclassify identifiers spelled like a field accessor (`@x`, `set_x`, `mod_x`,
// `act_x`) as properties. Purely lexical, so it applies before type checking.
// (`^field` accessors are recognized directly by the grammar.)
fn refine_accessors(content: &str, tokens: &mut [LexToken]) {
    for tok in tokens.iter_mut() {
        if tok.kind == LexTokenKind::Variable && is_accessor_name(&content[tok.start..tok.end]) {
            tok.kind = LexTokenKind::Property;
        }
    }
}

// Lexical refinement: a `Type` token immediately followed by a `::` operator
// is acting as a namespace qualifier (e.g. `Std` in `Std::Array`), so recolor
// it as `Namespace`. The final capitalized segment (not followed by `::`)
// stays a `Type`. This gives qualifiers a distinct color from the entity they
// qualify, which is a purely positional decision and so safe even on broken
// input.
fn refine_namespaces(content: &str, tokens: &mut [LexToken]) {
    for i in 0..tokens.len() {
        if tokens[i].kind != LexTokenKind::Type {
            continue;
        }
        let Some(next) = tokens.get(i + 1) else {
            continue;
        };
        if next.kind == LexTokenKind::Operator && &content[next.start..next.end] == "::" {
            tokens[i].kind = LexTokenKind::Namespace;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render the token stream as (kind, matched-text) pairs.
    ///
    /// `String` is fully qualified here because `use LexTokenKind::*` below
    /// brings the `String` *variant* into scope (which is what we want in the
    /// assertions, but not for this return type).
    fn lex(content: &str) -> Vec<(LexTokenKind, std::string::String)> {
        lex_tokens(content)
            .into_iter()
            .map(|t| (t.kind, content[t.start..t.end].to_string()))
            .collect()
    }

    use LexTokenKind::*;

    /// Verifies that keywords are recognized only as whole words, so an
    /// identifier that merely starts with a keyword stays a variable.
    #[test]
    fn keywords_vs_identifiers() {
        assert_eq!(
            lex("let x = 1"),
            vec![
                (Keyword, "let".into()),
                (Variable, "x".into()),
                (Operator, "=".into()),
                (Number, "1".into()),
            ]
        );
        // A keyword is only a keyword when it is a whole word.
        assert_eq!(lex("letx"), vec![(Variable, "letx".into())]);
        assert_eq!(lex("ifce"), vec![(Variable, "ifce".into())]);
        // Declaration keywords spelled as inline literals in the grammar are
        // still recognized as keywords here.
        assert_eq!(
            lex("type module trait impl struct union namespace import"),
            vec![
                (Keyword, "type".into()),
                (Keyword, "module".into()),
                (Keyword, "trait".into()),
                (Keyword, "impl".into()),
                (Keyword, "struct".into()),
                (Keyword, "union".into()),
                (Keyword, "namespace".into()),
                (Keyword, "import".into()),
            ]
        );
    }

    /// Verifies that `true`/`false`/`nullptr` are recognized as whole-word
    /// constants, while a longer identifier sharing the prefix is a variable.
    #[test]
    fn boolean_and_nullptr_constants() {
        assert_eq!(
            lex("true false nullptr"),
            vec![
                (Boolean, "true".into()),
                (Boolean, "false".into()),
                (Keyword, "nullptr".into()),
            ]
        );
        // ...but a longer identifier with that prefix is an identifier.
        assert_eq!(lex("trueish"), vec![(Variable, "trueish".into())]);
        assert_eq!(lex("nullptr_x"), vec![(Variable, "nullptr_x".into())]);
    }

    /// Verifies that a capital name before `::` is recolored as a namespace
    /// while the final segment stays a type, and that `.` does not trigger this.
    #[test]
    fn capital_names_and_namespaces() {
        // A standalone capital name is a type.
        assert_eq!(lex("Array"), vec![(Type, "Array".into())]);
        // Qualifiers before `::` become namespaces; the last segment is the type.
        assert_eq!(
            lex("Std::Array"),
            vec![
                (Namespace, "Std".into()),
                (Operator, "::".into()),
                (Type, "Array".into()),
            ]
        );
        assert_eq!(
            lex("Std::Iterator::map"),
            vec![
                (Namespace, "Std".into()),
                (Operator, "::".into()),
                (Namespace, "Iterator".into()),
                (Operator, "::".into()),
                (Variable, "map".into()),
            ]
        );
        // `.` (field/dot) does NOT turn the capital into a namespace. The
        // dotted lowercase name stays a plain identifier — its function/value
        // role is decided by the AST overlay, not lexically.
        assert_eq!(
            lex("Foo.bar"),
            vec![
                (Type, "Foo".into()),
                (Operator, ".".into()),
                (Variable, "bar".into()),
            ]
        );
    }

    /// Verifies number lexing: type suffixes, hex, exponents, a leading minus
    /// as part of the literal, but a binary minus on identifiers as an operator.
    #[test]
    fn numbers() {
        assert_eq!(lex("42"), vec![(Number, "42".into())]);
        assert_eq!(lex("42_I64"), vec![(Number, "42_I64".into())]);
        assert_eq!(lex("0xFF_U8"), vec![(Number, "0xFF_U8".into())]);
        assert_eq!(lex("3.14e-2"), vec![(Number, "3.14e-2".into())]);
        // A leading minus is part of the literal (matches the real grammar).
        assert_eq!(lex("-1"), vec![(Number, "-1".into())]);
        // Binary minus with a non-digit operand stays an operator.
        assert_eq!(
            lex("a - b"),
            vec![
                (Variable, "a".into()),
                (Operator, "-".into()),
                (Variable, "b".into()),
            ]
        );
    }

    /// Verifies that string and char literals lex as one token, and an escaped
    /// quote inside a string does not terminate it.
    #[test]
    fn strings_and_chars() {
        assert_eq!(lex("\"hello\""), vec![(String, "\"hello\"".into())]);
        assert_eq!(lex("'a'"), vec![(String, "'a'".into())]);
        // Escaped quote inside a string does not terminate it.
        assert_eq!(lex("\"a\\\"b\""), vec![(String, "\"a\\\"b\"".into())]);
    }

    /// Verifies that a `//` line comment runs to end of line and the next line
    /// resumes normal lexing.
    #[test]
    fn line_comment() {
        assert_eq!(
            lex("x // tail\ny"),
            vec![
                (Variable, "x".into()),
                (Comment, "// tail".into()),
                (Variable, "y".into()),
            ]
        );
    }

    /// Verifies that the lexer keeps a multi-line block comment as a single
    /// token (per-line splitting happens later, in the LSP encoder).
    #[test]
    fn block_comment_multiline_is_one_token() {
        // The lexer keeps a block comment as a single (possibly multi-line)
        // token; splitting per line for the LSP happens in the encoder.
        assert_eq!(
            lex("/* a\n b */x"),
            vec![(Comment, "/* a\n b */".into()), (Variable, "x".into()),]
        );
    }

    /// Verifies that multi-character operators (`>>`, `->`, `=>`, `$`) lex as
    /// single operator tokens between identifiers.
    #[test]
    fn operators() {
        assert_eq!(
            lex("f >> g $ x"),
            vec![
                (Variable, "f".into()),
                (Operator, ">>".into()),
                (Variable, "g".into()),
                (Operator, "$".into()),
                (Variable, "x".into()),
            ]
        );
        assert_eq!(
            lex("a -> b => c"),
            vec![
                (Variable, "a".into()),
                (Operator, "->".into()),
                (Variable, "b".into()),
                (Operator, "=>".into()),
                (Variable, "c".into()),
            ]
        );
    }

    /// Verifies that accessor-shaped names (`@x`, `set_x`, `^field`, `^0`) lex
    /// as properties, while a name that only contains a prefix stays a variable.
    #[test]
    fn field_accessors_are_properties() {
        // Getter / setter / modifier / act functions, by spelling.
        assert_eq!(lex("@value"), vec![(Property, "@value".into())]);
        assert_eq!(lex("set_value"), vec![(Property, "set_value".into())]);
        assert_eq!(lex("mod_value"), vec![(Property, "mod_value".into())]);
        assert_eq!(lex("act_value"), vec![(Property, "act_value".into())]);
        // Index-syntax field / tuple accessors.
        assert_eq!(
            lex("arr[^field]"),
            vec![(Variable, "arr".into()), (Property, "^field".into()),]
        );
        assert_eq!(
            lex("t[^0]"),
            vec![(Variable, "t".into()), (Property, "^0".into()),]
        );
        // A plain identifier that merely contains, but does not start with, an
        // accessor prefix stays a variable.
        assert_eq!(lex("settings"), vec![(Variable, "settings".into())]);
    }

    // --- Robustness: the scanner must never fail or panic on broken input. ---

    /// Verifies that an unterminated string stops at end of line rather than
    /// swallowing the next line.
    #[test]
    fn unterminated_string_is_tolerated() {
        // Stops at end of line, does not swallow the next line.
        assert_eq!(
            lex("\"oops\nx"),
            vec![(String, "\"oops".into()), (Variable, "x".into())]
        );
    }

    /// Verifies that an unterminated block comment lexes as a comment to end of
    /// input rather than failing.
    #[test]
    fn unterminated_block_comment_is_tolerated() {
        assert_eq!(
            lex("/* never closed"),
            vec![(Comment, "/* never closed".into())]
        );
    }

    /// Verifies that mid-edit garbage never panics and always yields ordered,
    /// in-bounds, char-boundary-aligned tokens.
    #[test]
    fn broken_syntax_does_not_panic() {
        // Mid-edit garbage: unbalanced delimiters, stray symbols, partial tokens.
        let inputs = [
            "module Main; main = (",
            "let = = => |] # ^ & ~",
            "type Foo struct { x : ",
            "🎉 unicode \"半角\" 漢字 = 42",
            "",
            "                 ",
            "}}}]]])))",
        ];
        for input in inputs {
            // Must return without panicking. Tokens must stay ordered and
            // within bounds.
            let toks = lex_tokens(input);
            let mut prev_end = 0;
            for t in &toks {
                assert!(t.start >= prev_end, "tokens overlap or are unordered");
                assert!(t.end <= input.len(), "token end out of bounds");
                assert!(t.start < t.end, "empty token");
                // Slicing must land on char boundaries (no panic).
                let _ = &input[t.start..t.end];
                prev_end = t.end;
            }
        }
    }

    /// Verifies that token offsets remain valid byte positions across multi-byte
    /// characters, so tokens after them slice correctly.
    #[test]
    fn unicode_positions_are_byte_offsets() {
        // Identifiers are ASCII-only, but strings/comments carry multi-byte
        // text. Byte offsets must stay correct across those characters so the
        // tokens after them are sliced correctly.
        let src = "x = \"漢字\" + y";
        let toks = lex_tokens(src);
        let rendered: Vec<(LexTokenKind, &str)> = toks
            .iter()
            .map(|t| (t.kind, &src[t.start..t.end]))
            .collect();
        assert_eq!(
            rendered,
            vec![
                (Variable, "x"),
                (Operator, "="),
                (String, "\"漢字\""),
                (Operator, "+"),
                (Variable, "y"),
            ]
        );
    }
}
