//! Negative tests: confirm the parser still rejects constructs that
//! the hole feature was never meant to accept. Two flavours:
//!
//! 1. Constructs the parser would have rejected as an incomplete
//!    expression even before holes existed (e.g. `if {`, `match x {`).
//!
//! 2. Hole positions we explicitly chose NOT to support in this phase
//!    (let bound, operator rhs, arg list, struct field, array
//!    element). Holes there must remain syntax errors so the feature
//!    does not silently widen.
//!
//! We assert grammar-level rejection directly via pest, without going
//! through elaboration. That avoids any coupling to error message
//! wording (notably the `Expected expression` prefix that pest uses
//! for parse failures and our hole pass uses for ERR_HOLE).

use crate::tests::test_util::assert_grammar_rejects;

// ----- Class 1: pre-existing syntax errors --------------------------

/// The parser still rejects `let x = 10 in {` — the `{` doesn't start
/// any expression accepted in this position.
#[test]
pub fn no_hole_for_let_in_brace() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let x = 10 in {
        main : IO () = pure();
    "#,
    );
}

/// The parser still rejects `if {` — `if` requires a condition before the `{`.
#[test]
pub fn no_hole_for_if_without_cond() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = if {
        main : IO () = pure();
    "#,
    );
}

/// The parser still rejects a `match x {` with no arms or closing brace.
#[test]
pub fn no_hole_for_truncated_match() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = match 0 {
        main : IO () = pure();
    "#,
    );
}

/// The parser still rejects a bare `let` keyword with no binding.
#[test]
pub fn no_hole_for_lone_let() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let
        main : IO () = pure();
    "#,
    );
}

// ----- Class 2: positions we did not support in this phase ----------

/// The let bound position is intentionally not covered by `expr_or_hole`;
/// `let x = ; body` must remain a syntax error.
#[test]
pub fn no_hole_for_let_bound() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let x = ; 0;
        main : IO () = pure();
    "#,
    );
}

/// The operator right-hand side is intentionally not covered;
/// `1 + ;` must remain a syntax error.
#[test]
pub fn no_hole_for_operator_rhs() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = 1 + ;
        main : IO () = pure();
    "#,
    );
}

/// Argument-list slots are intentionally not covered;
/// `f(x, , z)` must remain a syntax error.
#[test]
pub fn no_hole_for_arg_list() {
    assert_grammar_rejects(
        r#"
        module Main;
        g : (I64, I64, I64) -> I64 = |t| 0;
        f : I64 = g(1, , 3);
        main : IO () = pure();
    "#,
    );
}

/// Struct field values are intentionally not covered;
/// `Foo { x: , y: 1 }` must remain a syntax error.
#[test]
pub fn no_hole_for_struct_field() {
    assert_grammar_rejects(
        r#"
        module Main;
        type Pair = struct { x : I64, y : I64 };
        f : Pair = Pair { x: , y: 1 };
        main : IO () = pure();
    "#,
    );
}

/// Array literal elements are intentionally not covered;
/// `[1, , 3]` must remain a syntax error.
#[test]
pub fn no_hole_for_array_lit() {
    assert_grammar_rejects(
        r#"
        module Main;
        f : Array I64 = [1, , 3];
        main : IO () = pure();
    "#,
    );
}
