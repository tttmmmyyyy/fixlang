use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
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
    test_source_fail(source, Configuration::develop_mode(), "Multiple `DEPRECATED`");
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
