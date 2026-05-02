// Negative tests: confirm the hole feature does not silently swallow
// what should be genuine syntax errors. Two flavours:
//
// 1. Constructs the parser would have rejected as an incomplete
//    expression even before holes existed (e.g. `if {`, `match x {`).
//    These must remain syntax errors and never reach ERR_HOLE.
//
// 2. Hole positions we explicitly chose NOT to support in this phase
//    (let bound, operator rhs, arg list, struct field). Holes there
//    should still be syntax errors so we don't accidentally widen the
//    feature without intent.
//
// We assert that compilation fails AND that the error text does not
// contain the ERR_HOLE marker `Expected expression of type` —
// the "of type" clause is mandatory in our hole message and is what
// keeps it distinct from pest's parse-time `Expected expression.`
// (which has the exact same prefix).

use crate::{configuration::Configuration, tests::test_util::test_source_fail_excludes};

const HOLE_ERR_MARKER: &str = "Expected expression of type";

// ----- Class 1: pre-existing syntax errors --------------------------

#[test]
pub fn no_hole_for_let_in_brace() {
    // `let x = 10 in {` — the `{` doesn't start any expression that the
    // grammar accepts here.
    let source = r#"
        module Main;
        f : I64 = let x = 10 in {
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_if_without_cond() {
    // `if {` — `if` requires a condition before the `{`.
    let source = r#"
        module Main;
        f : I64 = if {
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_truncated_match() {
    // `match x {` with no arms or closing brace.
    let source = r#"
        module Main;
        f : I64 = match 0 {
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_lone_let() {
    // `let` keyword on its own.
    let source = r#"
        module Main;
        f : I64 = let
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

// ----- Class 2: positions we did not support in this phase ----------

#[test]
pub fn no_hole_for_let_bound() {
    // `let x = ; body` — bound is empty. The let bound position is
    // intentionally NOT covered by `expr_hole` in this phase.
    let source = r#"
        module Main;
        f : I64 = let x = ; 0;
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_operator_rhs() {
    // `1 + ;` — operator right-hand side is intentionally NOT covered.
    let source = r#"
        module Main;
        f : I64 = 1 + ;
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_arg_list() {
    // `f(x, , z)` — argument list slot is intentionally NOT covered.
    let source = r#"
        module Main;
        g : (I64, I64, I64) -> I64 = |t| 0;
        f : I64 = g(1, , 3);
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_struct_field() {
    // `Foo { x: , y: 1 }` — struct field value is intentionally NOT
    // covered.
    let source = r#"
        module Main;
        type Pair = struct { x : I64, y : I64 };
        f : Pair = Pair { x: , y: 1 };
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}

#[test]
pub fn no_hole_for_array_lit() {
    // `[1, , 3]` — array literal element is intentionally NOT covered.
    let source = r#"
        module Main;
        f : Array I64 = [1, , 3];
        main : IO () = pure();
    "#;
    test_source_fail_excludes(source, Configuration::develop_mode(), HOLE_ERR_MARKER);
}
