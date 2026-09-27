//! The diagnostics for an `if` missing its `else`, and for a `let` missing the `in` or `;` after its
//! value.

use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
};

/// A `let` whose value is an `if` with no `else` reports that `if`: the `;` after its first branch
/// took the rest of the enclosing expression as the else branch, leaving the `let` without its `;`.
#[test]
pub fn test_an_if_missing_else_in_a_let_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64;
f = |n| (
    let x = if n > 0 { n };
    x + 1
);

main : IO ();
main = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `if` has no `else`, so everything after its `;` up to the end of the enclosing \
         expression is its else branch, and the `let` whose value it is has no `in` or `;` after \
         that value.\nHINT: add `else { ... }` to this `if`.\n\n6:13-6:28",
    );
}

/// In an `if` ... `else if` chain missing its last `else`, the error points at the last `if`, which
/// is the one whose `;` took the rest of the enclosing expression.
#[test]
pub fn test_an_else_if_chain_missing_its_last_else_reports_the_last_if() {
    let source = r##"
module Main;

f : I64 -> I64;
f = |n| (
    let x = if n > 0 { n } else if n < 0 { -n };
    x + 1
);

main : IO ();
main = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "HINT: add `else { ... }` to this `if`.\n\n6:33-6:49",
    );
}

/// A `let` whose value runs up to a closing bracket with no `in` or `;` after it is reported as that
/// `let`. An `if` in the form `if c { a }; { b }` closes its else branch with the brace, so it takes
/// nothing past it and is not the one reported.
#[test]
pub fn test_a_let_missing_in_before_a_closing_bracket_is_reported() {
    let source = r##"
module Main;

f : I64 -> I64;
f = |n| (
    let x = if n > 0 { n }; { 0 }
);

main : IO ();
main = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "This `let` has no `in` or `;` after its value.\n\n6:5-6:12",
    );
}

/// An `if` missing its `else` right before a closing bracket names `else` among the tokens expected
/// there.
#[test]
pub fn test_an_if_missing_else_before_a_closing_bracket_expects_else() {
    let source = r##"
module Main;

f : I64 -> I64;
f = |n| (if n > 0 { n });

main : IO ();
main = println $ f(3).to_string;
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Expected `else` or `;`.",
    );
}

/// A `let` whose value is an `if` in the form `if c { a }; b`, ended by its own `;`, binds the value
/// of that `if`.
#[test]
pub fn test_a_let_binds_an_if_whose_else_follows_a_semicolon() {
    let source = r##"
module Main;

f : I64 -> I64;
f = |n| (
    let x = if n > 0 { n }; -n;
    x + 1
);

main : IO ();
main = (
    assert_eq(|_| "", f(3), 4);;
    assert_eq(|_| "", f(-5), 6);;
    pure()
);
    "##;
    test_source(&source, Configuration::develop_mode());
}
