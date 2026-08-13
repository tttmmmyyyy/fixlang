//! The key of the type-check cache must identify the entry it names. Two global values that share
//! a key share the typed expression stored under it, so one of them is compiled from a body it
//! never declared — silently, since a cache hit skips type checking altogether.

use crate::{configuration::Configuration, tests::test_util::test_source};

/// Declaring a struct gives its field `b` an accessor named `@b`, so a value named `_b` in the
/// same namespace differs from the accessor only in a character no file name can carry. Both are
/// read here, and each must answer with its own body.
#[test]
fn test_a_field_accessor_and_a_value_named_alike_keep_their_own_bodies() {
    let source = r#"
        module Main;

        type S = unbox struct { b : I64 };

        namespace S {
            _b : S -> I64;
            _b = |_s| 999;
        }

        main : IO ();
        main = (
            let s = S { b : 1 };
            assert_eq(|_| "the field accessor", s.@b, 1);;
            assert_eq(|_| "the value named like the accessor", s._b, 999);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}
