use crate::{
    configuration::{Configuration, DeprecationMode},
    tests::test_util::{run_source_assert_failed, test_source, test_source_fail},
};

/// Verifies that a `DEPRECATED` pragma on a top-level global value
/// compiles successfully and that calls to the deprecated symbol are
/// accepted (warning-only by default).
#[test]
pub fn test_deprecated_global_value() {
    let source = r##"
        module Main;

        old_func : I64 -> I64;
        old_func = |x| x + 1;
        DEPRECATED[old_func, "Use `new_func` instead."];

        new_func : I64 -> I64;
        new_func = |x| x + 2;

        main : IO ();
        main = (
            let _ = old_func(10);
            let _ = new_func(10);
            pure()
        );
    "##;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that a `DEPRECATED` pragma written inside a `namespace { ... }`
/// resolves its target relative to the surrounding namespace.
#[test]
pub fn test_deprecated_in_namespace() {
    let source = r##"
        module Main;

        namespace Foo {
            bar : I64 -> I64;
            bar = |x| x + 1;
            DEPRECATED[bar, "Removed in next release."];
        }

        main : IO ();
        main = (
            let _ = Foo::bar(3);
            pure()
        );
    "##;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that a `DEPRECATED` pragma written inside a trait body targets
/// the named trait member (resolved against the enclosing trait's namespace).
#[test]
pub fn test_deprecated_trait_member_inner() {
    let source = r##"
        module Main;

        trait a : Greeter {
            old_greet : a -> String;
            DEPRECATED[old_greet, "Use `greet` instead."];

            greet : a -> String;
        }

        impl I64 : Greeter {
            old_greet = |_| "hi";
            greet = |_| "hello";
        }

        main : IO ();
        main = (
            let _ = (1).old_greet;
            let _ = (1).greet;
            pure()
        );
    "##;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that a `DEPRECATED` pragma whose target does not resolve to any
/// global or trait member is rejected with a diagnostic.
#[test]
pub fn test_deprecated_unknown_target_fails() {
    let source = r##"
        module Main;

        DEPRECATED[no_such_func, "Wrong"];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(source, Configuration::develop_mode(), "DEPRECATED");
}

/// Verifies that an absolute path inside `DEPRECATED[...]` is rejected:
/// the target must be written as a path relative to where the pragma sits.
#[test]
pub fn test_deprecated_absolute_path_fails() {
    let source = r##"
        module Main;

        old_func : I64 -> I64;
        old_func = |x| x;
        DEPRECATED[::Main::old_func, "Use new"];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(source, Configuration::develop_mode(), "absolute path");
}

/// Verifies that two `DEPRECATED` pragmas pointing at the same target are
/// rejected as a duplicate.
#[test]
pub fn test_deprecated_duplicate_fails() {
    let source = r##"
        module Main;

        old_func : I64 -> I64;
        old_func = |x| x;
        DEPRECATED[old_func, "first"];
        DEPRECATED[old_func, "second"];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Multiple `DEPRECATED`",
    );
}

/// Verifies that `FFI_EXPORT` accepts a qualified path (e.g. `Foo::bar`)
/// for the Fix value being exported, not just a bare name.
#[test]
pub fn test_ffi_export_with_path() {
    let source = r##"
        module Main;

        namespace Foo {
            bar : CInt -> CInt;
            bar = |x| x + 1.c_int;
        }
        FFI_EXPORT[Foo::bar, c_bar_path];

        main : IO ();
        main = pure();
    "##;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that backslash escape sequences inside the `DEPRECATED`
/// message string (`\\`, `\"`, `\n`, `\t`, `\uXXXX`) are decoded the same
/// way as inside `expr_string_lit`.
#[test]
pub fn test_deprecated_message_escape_sequences() {
    let source = r##"
        module Main;

        old_func : I64 -> I64;
        old_func = |x| x;
        DEPRECATED[old_func, "line1\nline2 \"quoted\" \\ あ"];

        main : IO ();
        main = pure();
    "##;
    test_source(source, Configuration::develop_mode());
}

/// Verifies that a `DEPRECATED` pragma written outside a trait body can
/// target a member of that trait via a qualified path
/// (`DEPRECATED[Greeter::old_greet, "..."]`). The inner-form is already
/// covered by `test_deprecated_trait_member_inner`.
#[test]
pub fn test_deprecated_outer_pragma_targets_trait_member() {
    let source = r##"
        module Main;

        trait a : Greeter {
            old_greet : a -> String;
            greet : a -> String;
        }
        DEPRECATED[Greeter::old_greet, "Use `greet` instead."];

        impl I64 : Greeter {
            old_greet = |_| "hi";
            greet = |_| "hello";
        }

        main : IO ();
        main = (
            let _ = (1).old_greet;
            pure()
        );
    "##;
    test_source(source, Configuration::develop_mode());
}

/// A `DEPRECATED` pragma's path is interpreted *relative to its enclosing
/// container*. Inside `namespace Foo { ... }`, `DEPRECATED[Bar::baz, ..]`
/// resolves to `Foo::Bar::baz`; if no such global exists we expect a
/// "not found" diagnostic — never a fallthrough match against an unrelated
/// `Bar::baz` defined elsewhere.
#[test]
pub fn test_deprecated_namespace_container_miss_fails() {
    let source = r##"
        module Main;

        namespace Foo {
            DEPRECATED[Bar::baz, "Removed."];
        }

        // `Bar::baz` exists at the top level, but the pragma above sits
        // inside `Foo`, so it must look up `Foo::Bar::baz` (which doesn't
        // exist) — not this one.
        namespace Bar {
            baz : I64;
            baz = 0;
        }

        main : IO ();
        main = pure();
    "##;
    test_source_fail(source, Configuration::develop_mode(), "not found under");
}

/// `FFI_EXPORT[::Foo::bar, c_bar];` is rejected for the same reason
/// `DEPRECATED[::Foo::bar, ..]` is: the path must be relative to the
/// surrounding container.
#[test]
pub fn test_ffi_export_absolute_path_fails() {
    let source = r##"
        module Main;

        bar : CInt -> CInt;
        bar = |x| x;
        FFI_EXPORT[::Main::bar, c_bar_abs];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(source, Configuration::develop_mode(), "absolute path");
}

/// Verifies that the auto-generated `Std::<Type>::to_<type>` cast
/// functions registered programmatically by `make_std_mod` carry their
/// `DEPRECATED` entries. Compilation succeeds (warning-only) but the
/// `--deny-deprecated` mode would convert this to a hard error.
#[test]
pub fn test_stdlib_to_cast_is_deprecated() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            // `Std::I64::to_F64` is now a deprecated alias for
            // `ToF64::to_f64`. The build succeeds with a warning.
            let _ : F64 = 3.to_F64;
            pure()
        );
    "##;
    test_source(source, Configuration::develop_mode());
}

/// A deprecated value is reported wherever it is named, whichever kind of expression holds the use.
/// The report walks every node of the expression tree, so this pins that the walk reaches them all.
#[test]
pub fn test_deprecated_use_is_reported_in_every_expression_form() {
    let source = r##"
        module Main;

        old : I64 -> I64;
        old = |x| x;
        DEPRECATED[old, "gone"];

        apply : (I64 -> I64) -> I64 -> I64;
        apply = |f, x| f(x);

        type S = unbox struct { v : I64 };

        in_app_func : I64;
        in_app_func = old(1);

        in_app_arg : I64;
        in_app_arg = apply(old, 1);

        in_lam_body : I64 -> I64;
        in_lam_body = |x| old(x);

        in_let_bound : I64;
        in_let_bound = ( let y = old(1); y );

        in_let_value : I64;
        in_let_value = ( let y = 1; old(y) );

        in_if_cond : I64;
        in_if_cond = if old(0) == 0 { 1 } else { 2 };

        in_if_then : I64;
        in_if_then = if true { old(3) } else { 2 };

        in_if_else : I64;
        in_if_else = if true { 1 } else { old(4) };

        in_match_cond : I64;
        in_match_cond = match Option::some(old(5)) { some(v) => v, none(_) => 0 };

        in_match_arm : I64;
        in_match_arm = match Option::some(6) { some(v) => old(v), none(_) => 0 };

        in_tyanno : I64;
        in_tyanno = (old(7) : I64);

        in_make_struct : S;
        in_make_struct = S { v : old(8) };

        in_array_lit : Array I64;
        in_array_lit = [old(9)];

        in_ffi_call : I32;
        in_ffi_call = FFI_CALL[I32 abs(I32), old(10).to_I32];

        in_eval_side : I64;
        in_eval_side = ( eval old(11); 0 );

        in_eval_main : I64;
        in_eval_main = ( eval 12; old(13) );

        main : IO ();
        main = (
            eval in_app_func;
            eval in_app_arg;
            eval in_lam_body;
            eval in_let_bound;
            eval in_let_value;
            eval in_if_cond;
            eval in_if_then;
            eval in_if_else;
            eval in_match_cond;
            eval in_match_arm;
            eval in_tyanno;
            eval in_make_struct;
            eval in_array_lit;
            eval in_ffi_call;
            eval in_eval_side;
            eval in_eval_main;
            pure()
        );
    "##;
    let mut config = Configuration::develop_mode();
    config.deprecation_mode = DeprecationMode::Deny;
    let errmsg = run_source_assert_failed(source, config);
    // One line per kind of expression, each holding this program's only use of `old` in that kind.
    for use_site in [
        "in_app_func = old(1);",
        "in_app_arg = apply(old, 1);",
        "in_lam_body = |x| old(x);",
        "let y = old(1); y",
        "let y = 1; old(y)",
        "in_if_cond = if old(0) == 0",
        "in_if_then = if true { old(3) }",
        "else { old(4) }",
        "match Option::some(old(5))",
        "some(v) => old(v)",
        "in_tyanno = (old(7) : I64);",
        "in_make_struct = S { v : old(8) };",
        "in_array_lit = [old(9)];",
        "old(10).to_I32",
        "eval old(11); 0",
        "eval 12; old(13)",
    ] {
        assert!(
            errmsg.contains(use_site),
            "the deprecated use in `{}` was not reported.\nReport:\n{}",
            use_site,
            errmsg
        );
    }
}
