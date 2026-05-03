// Negative tests: confirm the parser still rejects constructs that
// the hole feature was never meant to accept. Two flavours:
//
// 1. Constructs the parser would have rejected as an incomplete
//    expression even before holes existed (e.g. `if {`, `match x {`).
//
// 2. Hole positions we explicitly chose NOT to support in this phase
//    (let bound, operator rhs, arg list, struct field, array
//    element). Holes there must remain syntax errors so the feature
//    does not silently widen.
//
// We assert grammar-level rejection directly via pest, without going
// through elaboration. That avoids any coupling to error message
// wording (notably the `Expected expression` prefix that pest uses
// for parse failures and our hole pass uses for ERR_HOLE).

use crate::tests::test_util::assert_grammar_rejects;

// ----- Class 1: pre-existing syntax errors --------------------------

#[test]
pub fn no_hole_for_let_in_brace() {
    // `let x = 10 in {` — the `{` doesn't start any expression that the
    // grammar accepts here.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let x = 10 in {
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_if_without_cond() {
    // `if {` — `if` requires a condition before the `{`.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = if {
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_truncated_match() {
    // `match x {` with no arms or closing brace.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = match 0 {
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_lone_let() {
    // `let` keyword on its own.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let
        main : IO () = pure();
    "#,
    );
}

// ----- Class 2: positions we did not support in this phase ----------

#[test]
pub fn no_hole_for_let_bound() {
    // `let x = ; body` — bound is empty. The let bound position is
    // intentionally NOT covered by `expr_hole` in this phase.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = let x = ; 0;
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_operator_rhs() {
    // `1 + ;` — operator right-hand side is intentionally NOT covered.
    assert_grammar_rejects(
        r#"
        module Main;
        f : I64 = 1 + ;
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_arg_list() {
    // `f(x, , z)` — argument list slot is intentionally NOT covered.
    assert_grammar_rejects(
        r#"
        module Main;
        g : (I64, I64, I64) -> I64 = |t| 0;
        f : I64 = g(1, , 3);
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_struct_field() {
    // `Foo { x: , y: 1 }` — struct field value is intentionally NOT
    // covered.
    assert_grammar_rejects(
        r#"
        module Main;
        type Pair = struct { x : I64, y : I64 };
        f : Pair = Pair { x: , y: 1 };
        main : IO () = pure();
    "#,
    );
}

#[test]
pub fn no_hole_for_array_lit() {
    // `[1, , 3]` — array literal element is intentionally NOT covered.
    assert_grammar_rejects(
        r#"
        module Main;
        f : Array I64 = [1, , 3];
        main : IO () = pure();
    "#,
    );
}
