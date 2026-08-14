// A struct literal gives a value to each field of one struct type, so a name the type does not
// declare, a declared name left out, and a name given twice are all rejected with a source-level
// diagnostic. The literal may write the fields in any order; elaboration puts them back into
// declaration order for code generation.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{run_source_assert_failed, test_source, test_source_fail},
    };

    /// A literal that repeats a field is reported, with the field and the struct named.
    #[test]
    pub fn test_struct_literal_duplicate_field_rejected() {
        let source = r#"
module Main;

type S = struct { a : I64 };

main : IO ();
main = (
    let s = S { a : 1, a : 2 };
    println(s.@a.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Duplicate field `a` of struct `Main::S`.",
        );
    }

    /// A repeat is reported when the field list names every declared field, so that every declared
    /// field has a value.
    #[test]
    pub fn test_struct_literal_duplicate_field_with_every_field_given_rejected() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let s = S { a : 1, a : 2, b : 3 };
    println((s.@a + s.@b).to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Duplicate field `a` of struct `Main::S`.",
        );
    }

    /// Every repeated field name of one literal is reported.
    #[test]
    pub fn test_struct_literal_every_repeated_field_reported() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let s = S { a : 1, a : 2, b : 3, b : 4 };
    println((s.@a + s.@b).to_string)
);
"#;
        let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
        assert!(
            errmsg.contains("Duplicate field `a` of struct `Main::S`."),
            "the repeat of `a` is reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            errmsg.contains("Duplicate field `b` of struct `Main::S`."),
            "the repeat of `b` is reported, but the message is:\n{}",
            errmsg
        );
    }

    /// A literal wrong in each of the three ways at once — a field given twice, a declared field
    /// left out, and a field the struct does not declare — is reported for all three, so one
    /// compilation shows every way the field list is wrong.
    #[test]
    pub fn test_struct_literal_duplicate_missing_and_unknown_fields_all_reported() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let s = S { a : 1, a : 2, zz : 3 };
    println(s.@a.to_string)
);
"#;
        let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
        assert!(
            errmsg.contains("Duplicate field `a` of struct `Main::S`."),
            "the repeated field is reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            errmsg.contains("Missing field `b` of struct `Main::S`."),
            "the missing field is reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            errmsg.contains("Unknown field `zz` for struct `Main::S`."),
            "the unknown field is reported, but the message is:\n{}",
            errmsg
        );
    }

    /// A name the struct does not declare is an unknown field at each of its occurrences, even
    /// where the literal gives it twice: the struct has no such field for the second one to
    /// repeat.
    #[test]
    pub fn test_struct_literal_repeated_unknown_field_is_unknown_at_each_occurrence() {
        let source = r#"
module Main;

type S = struct { a : I64 };

main : IO ();
main = (
    let s = S { zz : 1, zz : 2 };
    println(s.@a.to_string)
);
"#;
        let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
        assert!(
            errmsg.contains("Unknown field `zz` for struct `Main::S`."),
            "the undeclared name is reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            !errmsg.contains("Duplicate field `zz`"),
            "a name the struct does not declare is not reported as a repeat of one of its fields, \
             but the message is:\n{}",
            errmsg
        );
    }

    /// A field name the struct does not declare is reported as an unknown field.
    #[test]
    pub fn test_struct_literal_unknown_field_rejected() {
        let source = r#"
module Main;

type S = struct { a : I64 };

main : IO ();
main = (
    let s = S { a : 1, zz : 2 };
    println(s.@a.to_string)
);
"#;
        test_source_fail(
            source,
            Configuration::develop_mode(),
            "Unknown field `zz` for struct `Main::S`.",
        );
    }

    /// Field expressions are type-checked in the order the literal writes them, so the first error
    /// reported is the first one the programmer reads.
    ///
    /// Elaborating one field leaves the type checker changed, so the order the fields are walked in
    /// decides which of several errors is reported and which candidate an overloaded name settles
    /// on. A literal is handed to code generation in declaration order, and this pins that the
    /// reordering happens after the walk rather than before it: the fields below are written in the
    /// reverse of the declaration's order, and both are ill-typed.
    #[test]
    pub fn test_struct_literal_fields_are_type_checked_in_source_order() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64 };

main : IO ();
main = (
    let s = S { b : "written first", a : true };
    println(s.@a.to_string)
);
"#;
        let errmsg = run_source_assert_failed(source, Configuration::develop_mode());
        assert!(
            errmsg.contains("Std::String"),
            "the field written first is the one reported, but the message is:\n{}",
            errmsg
        );
        assert!(
            !errmsg.contains("Std::Bool"),
            "the field written second is reported only after the first one is fixed, but the \
             message is:\n{}",
            errmsg
        );
    }

    /// Fields written in an order other than the declaration's still reach the field they name.
    #[test]
    pub fn test_struct_literal_fields_out_of_declaration_order() {
        let source = r#"
module Main;

type S = struct { a : I64, b : I64, c : I64 };

main : IO ();
main = (
    let s = S { c : 3, a : 1, b : 2 };
    assert_eq(|_|"a", s.@a, 1);;
    assert_eq(|_|"b", s.@b, 2);;
    assert_eq(|_|"c", s.@c, 3);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
