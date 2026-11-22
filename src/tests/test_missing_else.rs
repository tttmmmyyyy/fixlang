//! The `if` pointed at beside the parse error of a `let` left without its `;`, when the `if` is in
//! the bound expression of the `let` with no `else` block: its `;` makes the rest of the block its
//! else branch.

use crate::{
    configuration::Configuration,
    tests::test_util::{run_source_assert_failed, test_source},
};

/// The description of the location of an `if` that may be missing its `else`.
const MAY_BE_MISSING_ELSE: &str = "This `if` may be missing its `else`.";

/// Compiles `source`, which fails to parse, and asserts that the parse error is reported with the
/// `if`s at `if_spans` pointed at beside it, in that order and no other.
fn assert_ifs_are_pointed_at(source: &str, if_spans: &[&str]) {
    let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
    assert!(errmsg.contains("Expected `;`"), "{}", errmsg);
    let offsets_in_errmsg: Vec<usize> = if_spans
        .iter()
        .map(|span| {
            errmsg
                .find(&format!("{}\n\n{}", MAY_BE_MISSING_ELSE, span))
                .unwrap_or_else(|| panic!("{} is not pointed at:\n{}", span, errmsg))
        })
        .collect();
    assert!(offsets_in_errmsg.is_sorted(), "{}", errmsg);
    assert_eq!(
        errmsg.matches(MAY_BE_MISSING_ELSE).count(),
        if_spans.len(),
        "{}",
        errmsg
    );
}

/// Compiles `source`, which fails to parse, and asserts that the parse error is reported with no
/// `if` pointed at beside it.
fn assert_no_if_is_pointed_at(source: &str) {
    assert_ifs_are_pointed_at(source, &[]);
}

/// The `if` of `let x = if c { a };` followed by the rest of the block is pointed at beside the parse error.
#[test]
pub fn test_an_if_with_no_else_in_a_let_is_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// In an `if` ... `else if` chain with no last `else`, the last `if` is pointed at: an `if` whose else
/// branch follows `else` reads no `;`.
#[test]
pub fn test_the_last_if_of_an_else_if_chain_is_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 } else if n < 0 { -1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:33-5:35"]);
}

/// Of the `if`s in the form `if c { a }; b` on the right end of the bound expression, the first is
/// pointed at: the `if`s in the rest of the block it took are written as they are meant.
#[test]
pub fn test_the_first_if_with_no_else_is_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    if n > 9 { 0 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// The `if` of a `let` that fails to parse past the parse error reported is not pointed at: only
/// the first error is reported, and that `if` has no part in it.
#[test]
pub fn test_an_if_of_a_let_failing_past_the_reported_error_is_not_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    x + 2
);

g : I64 -> I64 = |n| (
    let y = if n < 0 { 1 };
    y + 2
);

main : IO () = println $ (f(3) + g(3)).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// A `let` ended by `;` right before the closing token, whose bound expression is an `if` with no
/// `else`, has that `if` pointed at: the `;` put in next to the written one stays a separate `;`.
#[test]
pub fn test_an_if_with_no_else_right_before_the_closing_token_is_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// An `if` with no `else` in a lambda that is the bound expression is pointed at.
#[test]
pub fn test_an_if_with_no_else_in_a_lambda_bound_by_a_let_is_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let g = |m| if m > 0 { 1 };
    g(n) + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:17-5:19"]);
}

/// The `if` is pointed at wherever the rest of the block ends: at a `}`, at a `,` and at a `]`, as well
/// as at a `)`.
#[test]
pub fn test_an_if_with_no_else_is_pointed_at_before_each_closing_token() {
    let before_brace = r##"
module Main;

f : I64 -> I64 = |n| if n > 0 {
    let x = if n > 1 { 1 };
    x + 2
} else { 0 };

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&before_brace, &["5:13-5:15"]);
    let before_comma = r##"
module Main;

f : I64 -> (I64, I64) = |n| (
    let x = if n > 0 { 1 };
    x + 2, 0
);

main : IO () = println $ f(3).@0.to_string;
    "##;
    assert_ifs_are_pointed_at(&before_comma, &["5:13-5:15"]);
    let before_bracket = r##"
module Main;

f : I64 -> Array I64 = |n| [
    let x = if n > 0 { 1 };
    x + 2
];

main : IO () = println $ f(3).@(0).to_string;
    "##;
    assert_ifs_are_pointed_at(&before_bracket, &["5:13-5:15"]);
}

/// When the first expression on the right end of the bound expression that reads a `;` is an `eval`
/// or a `let`, the `;` meant for the outer `let` may be the one it reads, so no `if` is pointed at
/// beside the parse error.
#[test]
pub fn test_a_let_or_an_eval_in_the_bound_expression_points_at_no_if() {
    let eval_in_else = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 } else eval n;
    if x > 9 { 0 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_no_if_is_pointed_at(&eval_in_else);
    let let_in_bound = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = let y = n; if y > 0 { 1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_no_if_is_pointed_at(&let_in_bound);
}

/// An `if` in the form `if c { a }; { b }` closes its else branch with the brace, so no `if` is pointed at
/// beside the parse error it leaves.
#[test]
pub fn test_an_if_whose_else_branch_is_in_braces_points_at_no_if() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 }; { 0 }
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_no_if_is_pointed_at(&source);
}

/// A `;` expected anywhere but before the token closing the block has no `if` pointed at
/// beside it, though the bound expression of the `let` before it ends with an `if` with no `else`.
#[test]
pub fn test_a_semicolon_expected_inside_the_block_points_at_no_if() {
    let source = r##"
module Main;

main : IO () = (
    let x = if true { 1 };
    "x".println
    pure()
);
    "##;
    assert_no_if_is_pointed_at(&source);
}

/// A `let` whose bound expression is an `if` in the form `if c { a }; b`, ended by its own `;`, binds
/// the value of that `if`.
#[test]
pub fn test_a_let_binds_an_if_whose_else_branch_follows_a_semicolon() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { n }; -n;
    x + 1
);

main : IO () = (
    assert_eq(|_| "", f(3), 4);;
    assert_eq(|_| "", f(-5), 6);;
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The `if`s of two `let`s each left without its `;` are pointed at in the order they are written.
#[test]
pub fn test_the_ifs_of_two_lets_are_pointed_at_in_the_order_they_are_written() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    if n > 9 { 0 };
    let y = if n < 0 { 1 };
    x + y
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15", "7:13-7:15"]);
}

/// Of two `let`s left without their `;` at the reported parse error, the one whose cause is not
/// certain has no `if` pointed at, and the other still has its `if` pointed at.
#[test]
pub fn test_a_let_whose_cause_is_not_certain_leaves_the_if_of_the_other_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    let y = let z = n; if z > 0 { 1 };
    x + y
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// A `;` put in that ends an `eval` at the end of the block points at no `if`, and the `let` before
/// the `eval`, whose `;` is put in next, still has its `if` pointed at.
#[test]
pub fn test_a_semicolon_put_in_for_an_eval_leaves_the_if_of_the_let_pointed_at() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    eval x
);

main : IO () = (
    let m = f(3);
    println $ m.to_string
);
    "##;
    assert_ifs_are_pointed_at(&source, &["5:13-5:15"]);
}

/// Whether the `if` is pointed at does not depend on the layout between its `;` and its else branch:
/// Fix's grammar ignores layout.
#[test]
pub fn test_whether_the_if_is_pointed_at_does_not_depend_on_the_layout_after_its_semicolon() {
    let pointed_at = |after_semicolon: &str| {
        let source = format!(
            r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 {{ 1 }};{after_semicolon}x + 2
);

main : IO () = println $ f(3).to_string;
    "##
        );
        run_source_assert_failed(&source, Configuration::develop_mode())
            .contains(MAY_BE_MISSING_ELSE)
    };
    let pointed_at_on_the_next_line = pointed_at("\n    ");
    for after_semicolon in [" ", " /* one */ ", " // one\n    ", "\r\n    "] {
        assert_eq!(
            pointed_at(after_semicolon),
            pointed_at_on_the_next_line,
            "{:?}",
            after_semicolon
        );
    }
}

/// The `if` of the `let` is pointed at when the `;` put in before it ends an `eval` or an `if` in the
/// form `if c { a }; b`, though the text holds no other `let`.
#[test]
pub fn test_the_if_of_the_only_let_is_pointed_at_after_a_semicolon_put_in_for_an_eval_or_an_if() {
    let after_eval = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    eval x
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&after_eval, &["5:13-5:15"]);
    let after_if = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    if x > 9 { 0 }
);

main : IO () = println $ f(3).to_string;
    "##;
    assert_ifs_are_pointed_at(&after_if, &["5:13-5:15"]);
}
