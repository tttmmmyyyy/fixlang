//! Acceptance tests for the `expr_or_hole` grammar / `Std::#hole`
//! builtin and the in-elaboration ERR_HOLE pass.
//!
//! Each program contains at least one hole. Elaboration types each
//! hole as `Std::#hole : a` (which unifies with whatever the
//! surrounding context expected); `check_type` then walks the
//! substituted AST, finds every hole, and emits "Expected expression
//! [of type `T`]." We verify the expected text reaches the user.

use crate::{configuration::Configuration, tests::test_util::test_source_fail};

/// Convenience: every hole-rejecting test expects the same prefix; use
/// this constant so the message wording lives in one place.
const HOLE_ERR_PREFIX: &str = "Expected expression";

// ----- A group: holes inside expressions -------------------------------

/// ERR_HOLE fires for an empty expression after a `let ... ;` inside parens.
#[test]
pub fn hole_in_let_paren() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = 10; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty expression after a `let ... ;` without parens.
#[test]
pub fn hole_in_let_bare() {
    let source = r#"
        module Main;
        hole_val : I64 = let x = 10; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the body of an `eval` is empty (parenthesised).
#[test]
pub fn hole_in_eval_paren() {
    let source = r#"
        module Main;
        hole_val : I64 = (eval 1; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the body of an `eval` is empty (bare).
#[test]
pub fn hole_in_eval_bare() {
    let source = r#"
        module Main;
        hole_val : I64 = eval 1; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when a lambda body is empty (parenthesised).
#[test]
pub fn hole_in_lam_paren() {
    let source = r#"
        module Main;
        hole_val : I64 -> I64 = (|x| );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when a lambda body is empty (bare).
#[test]
pub fn hole_in_lam_bare() {
    let source = r#"
        module Main;
        hole_val : I64 -> I64 = |x| ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the right operand of `;;` is empty (parenthesised).
#[test]
pub fn hole_in_and_then_paren() {
    let source = r#"
        module Main;
        hole_val : IO () = (println("hi") ;; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the right operand of `;;` is empty (bare).
#[test]
pub fn hole_in_and_then_bare() {
    let source = r#"
        module Main;
        hole_val : IO () = println("hi") ;; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- if -------------------------------------------------------------

/// ERR_HOLE fires when an `if`'s then-branch block is empty.
#[test]
pub fn hole_in_if_then_block() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { } else { 1 };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when an `if`'s else-branch block is empty.
#[test]
pub fn hole_in_if_else_block() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 } else { };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the `else <expr>` form has no expression.
#[test]
pub fn hole_in_if_else_word() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 } else ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for the `;` form of `else_of_if` (`;` then a `{ ... }` block).
#[test]
pub fn hole_in_if_else_semi_block() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 }; { };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for the `;` form of `else_of_if_with_space` with an
/// empty trailing expression (the global `;` closes the def).
#[test]
pub fn hole_in_if_else_semi_expr() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 }; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when the then-branch is empty and the else uses the `;` form.
#[test]
pub fn hole_in_if_then_semi() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { }; { 1 };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- match ----------------------------------------------------------

// `match` in Fix dispatches on union variants. Use `Option a` from Std
// (defined as `union { none: (), some: a }`) for exhaustiveness.

/// ERR_HOLE fires for an empty rhs in a `match`'s first arm.
#[test]
pub fn hole_in_match_first() {
    let source = r#"
        module Main;
        opt : Option I64 = Option::some(1);
        hole_val : I64 = match opt { none() => , some(x) => x };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty rhs in a `match`'s last arm.
#[test]
pub fn hole_in_match_last() {
    let source = r#"
        module Main;
        opt : Option I64 = Option::some(1);
        hole_val : I64 = match opt { none() => 0, some(x) => };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty rhs in a `match`'s last arm followed by a trailing comma.
#[test]
pub fn hole_in_match_last_trailing_comma() {
    let source = r#"
        module Main;
        opt : Option I64 = Option::some(1);
        hole_val : I64 = match opt { none() => 0, some(x) => , };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for every empty arm rhs (here, all of them).
#[test]
pub fn hole_in_match_all() {
    let source = r#"
        module Main;
        opt : Option I64 = Option::some(1);
        hole_val : I64 = match opt { none() => , some(x) => };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires when a single-arm `match` has an empty rhs.
/// (Single-variant union is required to satisfy exhaustiveness.)
#[test]
pub fn hole_in_match_only() {
    let source = r#"
        module Main;
        type Single = union { only: () };
        s : Single = Single::only();
        hole_val : I64 = match s { only(_) => };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- do -------------------------------------------------------------

/// ERR_HOLE fires when a `do { ... }` block contains no expression.
#[test]
pub fn hole_in_do() {
    let source = r#"
        module Main;
        hole_val : IO () = do { };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- monadic bind (`*`) interaction ---------------------------------

/// ERR_HOLE survives the `*`-desugaring rewrite: with `*some_io`
/// expanded to a `>>=` chain, the chain's innermost value (the empty
/// let body, parenthesised) is still recognised as a hole.
#[test]
pub fn hole_in_let_after_bind_paren() {
    let source = r#"
        module Main;
        some_io : IO String = pure("hi");
        hole_val : IO () = (let s = *some_io; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE survives the `*`-desugaring rewrite (bare form of the
/// previous test).
#[test]
pub fn hole_in_let_after_bind_bare() {
    let source = r#"
        module Main;
        some_io : IO String = pure("hi");
        hole_val : IO () = let s = *some_io; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE survives `*`-desugaring inside a `do` block.
#[test]
pub fn hole_in_do_after_bind() {
    let source = r#"
        module Main;
        some_io : IO String = pure("hi");
        hole_val : IO () = do { let s = *some_io; };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- B group: top-level definition rhs ------------------------------

/// ERR_HOLE fires for the `global_name_defn` parse path
/// (`name = expr_or_hole ;`) when the type is given via a separate
/// `global_name_type_sign`.
#[test]
pub fn hole_global_defn_with_separate_sign() {
    let source = r#"
        module Main;
        hole_val : I64;
        hole_val = ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}


/// ERR_HOLE fires for the rhs hole of `global_name_type_sign`
/// (combined `value : T = ;` form).
#[test]
pub fn hole_global_with_sign() {
    let source = r#"
        module Main;
        hole_val : I64 = ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty rhs in `trait_member_value_impl`
/// (`name = ;` inside `impl ... { ... }`).
#[test]
pub fn hole_trait_member_value_impl() {
    let source = r#"
        module Main;
        trait a : MyTrait {
            mymethod : a -> a;
        }
        impl I64 : MyTrait {
            mymethod = ;
        }
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty rhs in `trait_member_value_type_sign`
/// (`name : T = ;` inside `impl ... { ... }`).
#[test]
pub fn hole_trait_member_value_type_sign() {
    let source = r#"
        module Main;
        trait a : MyTrait {
            mymethod : a -> a;
        }
        impl I64 : MyTrait {
            mymethod : I64 -> I64 = ;
        }
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- Nested / combined --------------------------------------------

/// ERR_HOLE wins over the cannot-infer diagnostic when a nested let
/// has an empty body that leaves the outer pattern's type
/// indeterminate. The hole pass short-circuits the fixed-types check.
#[test]
pub fn hole_nested_let_let() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = (let y = 10; ); );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty let body nested inside an `if`.
#[test]
pub fn hole_nested_if_let() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { let x = 10; } else { 1 };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for an empty let body nested inside a `match` arm.
#[test]
pub fn hole_nested_match_let() {
    let source = r#"
        module Main;
        opt : Option I64 = Option::some(1);
        hole_val : I64 = match opt { none() => let y = 10; , some(x) => x };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE fires for the body of the inner lambda in a curried
/// `|x| |y|` form when the inner body is empty.
#[test]
pub fn hole_nested_lam_lam() {
    let source = r#"
        module Main;
        hole_val : I64 -> I64 -> I64 = |x| |y| ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- Whitespace / comment edges -----------------------------------

/// ERR_HOLE still fires when the hole position is occupied only by a block comment.
#[test]
pub fn hole_with_block_comment() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = 10; /* todo */ );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE still fires when the hole position is occupied only by a line comment.
#[test]
pub fn hole_with_line_comment() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = 10; // todo
        );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// ERR_HOLE still fires when the hole position is occupied only by blank lines.
#[test]
pub fn hole_with_many_newlines() {
    let source = "
        module Main;
        hole_val : I64 = (let x = 10;


        );
        main : IO () = pure();
    ";
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- EOF boundary --------------------------------------------------

/// Exercises the `EOI` branch of the `&(ANY | EOI)` lookahead in the
/// `hole` rule: the hole-bearing global is the last item in the file,
/// so nothing follows the closing `;` but whitespace and EOF.
#[test]
pub fn hole_at_end_of_file() {
    let source = "module Main;\nmain : IO () = pure();\nhole_val : I64 = (let x = 10; );";
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

// ----- C group: user-written `?` / `?label` syntax --------------------
//
// `?` (with or without a trailing label like `?x`) is a user-writable
// hole. It elaborates to the same `Std::#hole : a` builtin the parser
// inserts in empty `expr_or_hole` positions, and ERR_HOLE fires the same
// way. The label is parsed but discarded — it survives only as source
// text the user can read.

/// Bare `?` in expression position is accepted and produces ERR_HOLE.
#[test]
pub fn hole_user_written_anonymous() {
    let source = r#"
        module Main;
        hole_val : I64 = ?;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// `?label` is accepted; the label is discarded by the parser but must
/// not break anything. ERR_HOLE still fires.
#[test]
pub fn hole_user_written_with_label() {
    let source = r#"
        module Main;
        hole_val : I64 = ?x;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// Underscore-led labels (`?_unused`) are valid identifier-shaped
/// labels and parse the same way.
#[test]
pub fn hole_user_written_with_underscore_label() {
    let source = r#"
        module Main;
        hole_val : I64 = ?_x;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// Multiple user-written holes in a single expression each produce
/// their own ERR_HOLE — `?x` and `?y` here become two independent
/// holes typed by their argument position.
#[test]
pub fn hole_user_written_in_call() {
    let source = r#"
        module Main;
        // # Parameters
        // * `x`
        // * `y`
        myfn : I64 -> I64 -> I64;
        myfn = |x, y| x + y;
        hole_val : I64 = myfn(?x, ?y);
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

/// User-written hole in dot-call receiver-method form: `(?).myfn(1)`
/// elaborates as `myfn(1)(?)` and the receiver hole is reported.
#[test]
pub fn hole_user_written_as_receiver() {
    let source = r#"
        module Main;
        myfn : I64 -> I64 -> I64;
        myfn = |x, y| x + y;
        hole_val : I64 = (?).myfn(1);
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}
