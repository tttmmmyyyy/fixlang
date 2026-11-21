//! The error for an `if` with no `else` block in the bound expression of a `let`, whose `;` makes the
//! rest of the block the else branch of the `if` and leaves the `let` without its `;`.

use crate::{
    configuration::Configuration,
    tests::test_util::{run_source_assert_failed, test_source, test_source_fail},
};

/// The `if` of `let x = if c { a };` followed by the rest of the block is reported, at its `if`.
#[test]
pub fn test_an_if_with_no_else_in_a_let_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:13-5:15",
    );
}

/// In an `if` ... `else if` chain with no last `else`, the last `if` is reported: an `if` whose else
/// branch follows `else` reads no `;`.
#[test]
pub fn test_the_last_if_of_an_else_if_chain_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 } else if n < 0 { -1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:33-5:35",
    );
}

/// Of the `if`s in the form `if c { a }; b` on the right end of the bound expression, the first is
/// reported: the `if`s in the rest of the block it took are written as they are meant.
#[test]
pub fn test_the_first_if_with_no_else_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    if n > 9 { 0 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:13-5:15",
    );
}

/// The `if` of a second `let`, found after a `;` was put in for the `let` before it, is reported at
/// its position in the source as written.
#[test]
pub fn test_an_if_after_a_block_already_given_a_semicolon_is_reported_where_it_is_written() {
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
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:13-5:15",
    );
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n10:13-10:15",
    );
}

/// A `let` ended by `;` right before the closing token, whose bound expression is an `if` with no
/// `else`, reports that `if`: the `;` put in next to the written one stays a separate `;`.
#[test]
pub fn test_an_if_with_no_else_right_before_the_closing_token_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:13-5:15",
    );
}

/// An `if` with no `else` in a lambda that is the bound expression is reported.
#[test]
pub fn test_an_if_with_no_else_in_a_lambda_bound_by_a_let_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let g = |m| if m > 0 { 1 };
    g(n) + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else` block.\n\n5:17-5:19",
    );
}

/// The `if` is reported wherever the rest of the block ends: at a `}`, at a `,` and at a `]`, as well
/// as at a `)`.
#[test]
pub fn test_an_if_with_no_else_is_reported_before_each_closing_token() {
    let message = "This `if` has no `else` block.\n\n5:13-5:15";
    let before_brace = r##"
module Main;

f : I64 -> I64 = |n| if n > 0 {
    let x = if n > 1 { 1 };
    x + 2
} else { 0 };

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(&before_brace, Configuration::develop_mode(), message);
    let before_comma = r##"
module Main;

f : I64 -> (I64, I64) = |n| (
    let x = if n > 0 { 1 };
    x + 2, 0
);

main : IO () = println $ f(3).@0.to_string;
    "##;
    test_source_fail(&before_comma, Configuration::develop_mode(), message);
    let before_bracket = r##"
module Main;

f : I64 -> Array I64 = |n| [
    let x = if n > 0 { 1 };
    x + 2
];

main : IO () = println $ f(3).@(0).to_string;
    "##;
    test_source_fail(&before_bracket, Configuration::develop_mode(), message);
}

/// When the first expression on the right end of the bound expression that reads a `;` is an `eval`
/// or a `let`, the `;` meant for the outer `let` may be the one it reads, so the parse error is
/// reported as it is.
#[test]
pub fn test_a_let_or_an_eval_in_the_bound_expression_leaves_the_parse_error() {
    let message = "Expected `;`";
    let eval_in_else = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 } else eval n;
    if x > 9 { 0 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(&eval_in_else, Configuration::develop_mode(), message);
    let let_in_bound = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = let y = n; if y > 0 { 1 };
    x + 2
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(&let_in_bound, Configuration::develop_mode(), message);
}

/// An `if` in the form `if c { a }; { b }` closes its else branch with the brace, so the parse error
/// it leaves is reported as it is.
#[test]
pub fn test_an_if_whose_else_branch_is_in_braces_leaves_the_parse_error() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 }; { 0 }
);

main : IO () = println $ f(3).to_string;
    "##;
    test_source_fail(&source, Configuration::develop_mode(), "Expected `;`");
}

/// A `;` expected anywhere but before the token closing the block is reported as the parse error
/// it is, though the bound expression of the `let` before it ends with an `if` with no `else`.
#[test]
pub fn test_a_semicolon_expected_inside_the_block_leaves_the_parse_error() {
    let source = r##"
module Main;

main : IO () = (
    let x = if true { 1 };
    "x".println
    pure()
);
    "##;
    test_source_fail(&source, Configuration::develop_mode(), "Expected `;`");
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

/// The `if`s of two `let`s each left without its `;` are reported in the order they are written.
#[test]
pub fn test_the_ifs_of_two_lets_are_reported_in_the_order_they_are_written() {
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
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    let first = errmsg.find("5:13-5:15").unwrap();
    let second = errmsg.find("7:13-7:15").unwrap();
    assert!(first < second, "{}", errmsg);
}

/// When one `let` left without its `;` ends with an `if` with no `else` and the cause for another
/// `let` is not certain, the parse error is reported as it is for both.
#[test]
pub fn test_a_let_whose_cause_is_not_certain_leaves_the_parse_error_for_all() {
    let source = r##"
module Main;

f : I64 -> I64 = |n| (
    let x = if n > 0 { 1 };
    x + 2
);

g : I64 -> I64 = |n| (
    let y = let z = n; if z > 0 { 1 };
    y + 2
);

main : IO () = println $ (f(3) + g(3)).to_string;
    "##;
    test_source_fail(&source, Configuration::develop_mode(), "Expected `;`");
}

/// A `;` that an `eval` at the end of the block misses leaves the parse error, though the `let`
/// before it ends with an `if` with no `else`: the `eval` misses its `;` either way.
#[test]
pub fn test_an_eval_missing_its_semicolon_before_the_closing_token_leaves_the_parse_error() {
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
    test_source_fail(&source, Configuration::develop_mode(), "Expected `;`");
}

/// Whether the `if` is reported does not depend on the layout between its `;` and its else branch:
/// Fix's grammar reads no layout.
#[test]
pub fn test_whether_the_if_is_reported_does_not_depend_on_the_layout_after_its_semicolon() {
    let reported = |after_semicolon: &str| {
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
            .contains("This `if` has no `else` block.")
    };
    let reported_on_the_next_line = reported("\n    ");
    for after_semicolon in [" ", " /* one */ ", " // one\n    ", "\r\n    "] {
        assert_eq!(
            reported(after_semicolon),
            reported_on_the_next_line,
            "{:?}",
            after_semicolon
        );
    }
}
