// Acceptance tests for the `expr_hole` grammar / `Std::#hole` builtin
// and the in-elaboration ERR_HOLE pass.
//
// Each program contains at least one hole. Elaboration types each hole
// as `Std::#hole : a` (which unifies with whatever the surrounding
// context expected); `check_type` then walks the substituted AST,
// finds every hole, and emits "Expected expression [of type `T`]."
// We verify the expected text reaches the user.

use crate::{configuration::Configuration, tests::test_util::test_source_fail};

// Convenience: every hole-rejecting test expects the same prefix; use
// this constant so the message wording lives in one place.
const HOLE_ERR_PREFIX: &str = "Expected expression";

// ----- A group: holes inside expressions -------------------------------

#[test]
pub fn hole_in_let_paren() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = 10; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_let_bare() {
    let source = r#"
        module Main;
        hole_val : I64 = let x = 10; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_eval_paren() {
    let source = r#"
        module Main;
        hole_val : I64 = (eval 1; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_eval_bare() {
    let source = r#"
        module Main;
        hole_val : I64 = eval 1; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_lam_paren() {
    let source = r#"
        module Main;
        hole_val : I64 -> I64 = (|x| );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_lam_bare() {
    let source = r#"
        module Main;
        hole_val : I64 -> I64 = |x| ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_and_then_paren() {
    let source = r#"
        module Main;
        hole_val : IO () = (println("hi") ;; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

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

#[test]
pub fn hole_in_if_then_block() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { } else { 1 };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_if_else_block() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 } else { };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_if_else_word() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 } else ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_if_else_semi_block() {
    // Semicolon form of `else_of_if` (not `_with_space`): `;` then `{ ... }`.
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 }; { };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_if_else_semi_expr() {
    // Semicolon form of `else_of_if_with_space`: `;` then expression
    // (here, an empty expression — the global `;` closes the def).
    let source = r#"
        module Main;
        hole_val : I64 = if true { 1 }; ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_in_if_then_semi() {
    // Empty then-branch with `;` form of else.
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

#[test]
pub fn hole_in_match_only() {
    // Single-arm match needs a single-variant union to satisfy the
    // exhaustiveness check.
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

#[test]
pub fn hole_in_let_after_bind_paren() {
    // `*some_io` desugars to a >>= chain wrapping the let body. The body
    // is hole, which becomes the innermost value of the chain.
    let source = r#"
        module Main;
        some_io : IO String = pure("hi");
        hole_val : IO () = (let s = *some_io; );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

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

#[test]
pub fn hole_global_defn_with_separate_sign() {
    // Exercises the `global_name_defn` parser path (`name = expr_hole ;`)
    // by giving the type via a separate `global_name_type_sign` (no
    // rhs). The combined `name : T = ;` form below exercises the hole
    // path inside `global_name_type_sign` itself.
    let source = r#"
        module Main;
        hole_val : I64;
        hole_val = ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}


#[test]
pub fn hole_global_with_sign() {
    // `value : T = ;` — rhs of `global_name_type_sign` is a hole.
    let source = r#"
        module Main;
        hole_val : I64 = ;
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_trait_member_value_impl() {
    // `name = ;` inside `impl ... { ... }` — rhs of
    // `trait_member_value_impl` is a hole.
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

#[test]
pub fn hole_trait_member_value_type_sign() {
    // `name : T = ;` inside `impl ... { ... }` — rhs of
    // `trait_member_value_type_sign` is a hole.
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

#[test]
pub fn hole_nested_let_let() {
    // Inner let's body is a hole, so the inner let has type `a` (a
    // free type variable), making `x : a`. The hole-free analogue
    // `(let x = (let y = 10; undefined("")); undefined(""))` would
    // trip Fix's "Cannot infer the type of this pattern" check, but
    // here check_type's hole pass fires first and short-circuits the
    // fixed-types check (since the holes are the likely cause of the
    // indeterminacy). We therefore expect ERR_HOLE rather than the
    // cannot-infer message.
    let source = r#"
        module Main;
        hole_val : I64 = (let x = (let y = 10; ); );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

#[test]
pub fn hole_nested_if_let() {
    let source = r#"
        module Main;
        hole_val : I64 = if true { let x = 10; } else { 1 };
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

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

#[test]
pub fn hole_with_block_comment() {
    let source = r#"
        module Main;
        hole_val : I64 = (let x = 10; /* todo */ );
        main : IO () = pure();
    "#;
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}

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

#[test]
pub fn hole_at_end_of_file() {
    // The hole-bearing global is the last thing in the file. The closing
    // `;` of the global is followed only by whitespace and EOF, so the
    // `&(ANY | EOI)` lookahead in the `hole` rule has to use its `EOI`
    // branch.
    let source = "module Main;\nmain : IO () = pure();\nhole_val : I64 = (let x = 10; );";
    test_source_fail(source, Configuration::develop_mode(), HOLE_ERR_PREFIX);
}
