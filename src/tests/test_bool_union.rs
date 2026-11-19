// `Bool` is an unbox union `{ _false : (), _true : () }`.
//
// Comparison, `if`, `&&`, `||`, and `not` are covered by test_basic; this test covers matching on
// a `Bool` with its variant patterns. (Its debug type being `DW_ATE_boolean` is checked in
// test_debug_info.)

#[cfg(test)]
mod bool_union_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

    // `Bool` is a union, so it can be matched on with the `_true()` / `_false()` patterns.
    #[test]
    pub fn test_match_on_bool() {
        let source = r#"
module Main;

bool_to_int : Bool -> I64;
bool_to_int = |b| match b {
    _true() => 1,
    _false() => 0
};

main : IO ();
main = (
    assert_eq(|_|"match true", bool_to_int(true), 1);;
    assert_eq(|_|"match false", bool_to_int(false), 0);;
    assert_eq(|_|"match cmp true", bool_to_int(1 < 2), 1);;
    assert_eq(|_|"match cmp false", bool_to_int(2 < 1), 0);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    /// `true` and `false` are the `Std::Bool` values even where the module declares its own
    /// `Std::Bool` with the same variants.
    #[test]
    pub fn test_bool_literal_beside_a_user_type_named_std_bool() {
        let source = r#"
module Main;

namespace Std {
    type Bool = unbox union { _false : (), _true : () };
}

main : IO ();
main = (
    assert_eq(|_|"true", if true { 1 } else { 0 }, 1);;
    assert_eq(|_|"false", if false { 1 } else { 0 }, 0);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
