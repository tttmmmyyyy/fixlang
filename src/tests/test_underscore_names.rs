// A name starting with `_` is internal. The character after the `_` decides the kind of the name: a
// capital letter starts the name of a type, a type alias, a trait, a trait alias, an associated type,
// a namespace or a module, and anything else starts the name of a value, a field or a variant.

#[cfg(test)]
mod tests {
    use crate::{
        configuration::Configuration,
        tests::test_util::{
            assert_grammar_accepts, assert_grammar_rejects, run_source_assert_failed, test_source,
            test_source_fail, test_sources, test_sources_fail,
        },
    };

    /// Every kind of capital name can start with `_`: a struct, a union, a type alias, a namespace,
    /// a trait with an associated type, a trait alias. Struct literals and struct patterns of such
    /// a type parse, also at the head of an `if` in parentheses, while a value named with `__` or
    /// `_` and a lowercase letter stays a value, also at the head of an `if`.
    #[test]
    pub fn test_capital_names_starting_with_an_underscore() {
        let source = r##"
module Main;

type _Point = struct { x : I64, y : I64 };
type _Shape = union { _circle : I64, square : I64 };
type _Alias = _Point;

namespace _Internal {
    __NN : I64 = 624;
    _add_nn : I64 -> I64 = |x| x + __NN;
}

trait a : _Describe {
    type _Part a;
    _describe : a -> String;
    _part : a -> _Part a;
}
impl _Point : _Describe {
    type _Part _Point = I64;
    _describe = |p| p.@x.to_string + "," + p.@y.to_string;
    _part = |p| p.@x;
}
impl _Point : Eq {
    eq = |p, q| p.@x == q.@x && p.@y == q.@y;
}

trait _Both = Eq + _Describe;

_describe_if_eq : [a : _Both] a -> a -> String = |p, q| if p == q { p._describe } else { "" };

_debug : Bool = true;

main : IO () = (
    let x = 3;
    let p = _Point { x, y : 4 };
    let _Point { x : px, _ } = p;
    let q : _Alias = Main::_Point { x : 3, y : 4 };
    let r = match _Shape::_circle(5) { _circle(c) => c, square(d) => d };
    let in_if = if _debug { x } else { 0 };
    let in_if_parenthesized = if (_Point { x : 3, y : 4 }) == p { 1 } else { 0 };
    eval *assert_eq(|_|"pattern", px, 3);
    eval *assert_eq(|_|"trait alias", _describe_if_eq(p, q), "3,4");
    eval *assert_eq(|_|"associated type", p._part, 3);
    eval *assert_eq(|_|"union", r, 5);
    eval *assert_eq(|_|"namespace", _Internal::_add_nn(1), 625);
    eval *assert_eq(|_|"value at the head of if", in_if, 3);
    eval *assert_eq(|_|"struct literal at the head of if", in_if_parenthesized, 1);
    eval *assert_eq(|_|"getter", _Point::@y(p), 4);
    pure()
);
"##;
        test_source(source, Configuration::develop_mode());
    }

    /// A module whose name starts with `_`: an import statement brings in its type, the namespace of
    /// the type and a value of a namespace, whose names start with `_`, and an absolute path reaches
    /// its type, its namespace and a value named with `__` without an import.
    #[test]
    pub fn test_module_named_with_an_underscore() {
        let lib_source = r##"
module _Lib;

type _Foo = struct { v : I64 };

namespace _N {
    v : I64 = 2;
}

__x : I64 = 3;
"##;
        let main_source = r##"
module Main;

import _Lib::{_Foo, _Foo::*, _N::v};

main : IO () = (
    let foo : _Foo = _Foo { v : 1 };
    let abs_foo : ::_Lib::_Foo = ::_Lib::_Foo { v : 4 };
    eval *assert_eq(|_|"imported type", foo.@v, 1);
    eval *assert_eq(|_|"imported value of a namespace", _N::v, 2);
    eval *assert_eq(|_|"absolute path to a value", ::_Lib::__x, 3);
    eval *assert_eq(|_|"absolute path to a type", abs_foo.@v, 4);
    eval *assert_eq(|_|"absolute path to a namespace", ::_Lib::_N::v, 2);
    pure()
);
"##;
        test_sources(&[main_source, lib_source], Configuration::develop_mode());
    }

    /// An import item starting with `_` and a capital letter names a type or a trait, so a missing
    /// one is reported as a missing entity.
    #[test]
    pub fn test_import_item_named_with_an_underscore_and_a_capital_is_a_type_or_trait() {
        let lib_source = r##"
module Lib;

_value : I64 = 1;
"##;
        let main_source = r##"
module Main;

import Lib::{_Missing};

main : IO () = pure();
"##;
        test_sources_fail(
            &[main_source, lib_source],
            Configuration::develop_mode(),
            "Cannot find entity named `Lib::_Missing`.",
        );
    }

    /// A value, a field or a variant named with `_` and a capital letter is rejected wherever it is
    /// written, every occurrence in one compilation: a global value's declaration and definition, a
    /// struct field, a union variant, a trait member's declaration and implementation, a reference
    /// alone, under a namespace and before a period, a `let` variable, an annotated lambda parameter,
    /// a `match` variable, a variant of a union pattern, a field of a struct literal, a field of a
    /// struct pattern, a field accessor of the index syntax, and the names in a `DEPRECATED` pragma
    /// and an `FFI_EXPORT` statement.
    #[test]
    pub fn test_value_side_names_starting_with_an_underscore_and_a_capital_are_rejected() {
        let source = r##"
module Main;

_Sig : I64;
_Sig = 1;

type S = struct { _Field : I64 };
type U = union { _Variant : I64 };

trait a : Tr {
    _Member : a;
}
impl I64 : Tr {
    _Member = 0;
}

DEPRECATED[_Dep, "deprecated"];
FFI_EXPORT[_Exp, fix_exp];

main : IO () = (
    let _Let = _Ref + Main::_Qualified + _Dot.to_string;
    let f = |_Param : I64| _Param;
    let x = match 1 { _Arm => 0 };
    let y = match u { _Variant(v) => v };
    let s = S { _Lit : 1 };
    let S { _Pat } = s;
    let z = s[^_Index];
    pure()
);
"##;
        let message = run_source_assert_failed(source, Configuration::develop_mode());
        for name in [
            "_Sig",
            "_Field",
            "_Variant",
            "_Member",
            "_Dep",
            "_Exp",
            "_Let",
            "_Ref",
            "_Qualified",
            "_Dot",
            "_Param",
            "_Arm",
            "_Lit",
            "_Pat",
            "_Index",
        ] {
            assert!(
                message.contains(&format!(
                    "A value, a field or a variant cannot be named `{}`: `_` followed by a capital \
                     letter starts the name of a type, a trait, a namespace or a module.\n\
                     HINT: write a lowercase letter or another `_` after the leading `_`, as in \
                     `_{}`.",
                    name, name
                )),
                "the error for `{}` is missing:\n{}",
                name,
                message
            );
        }
    }

    /// A parse error at a field name or at the name a pragma takes lists only the rules a correct
    /// program takes there, leaving out the alternative that catches a value named with `_` and a
    /// capital letter.
    #[test]
    pub fn test_parse_errors_at_value_names_list_the_rules_a_correct_program_takes() {
        let cases = [
            (
                "type S = struct { 1x : I64 };",
                "Expected field or variant name.",
            ),
            (
                "f : I64 = S { x : 1, 2 };",
                "Expected field or variant name.",
            ),
            (
                "f : I64 = match x { S { 1 } => 0 };",
                "Expected field or variant name.",
            ),
            ("DEPRECATED[1f, \"m\"];", "Expected fullname."),
            ("FFI_EXPORT[1f, g];", "Expected fullname."),
        ];
        for (line, expected) in cases {
            let source = format!("module Main;\n\n{}\n\nmain : IO () = pure();\n", line);
            test_source_fail(&source, Configuration::develop_mode(), expected);
        }
    }

    /// The parser's acceptance check used by the grammar tests counts a value named with `_` and a
    /// capital letter as rejected, as building the program does.
    #[test]
    pub fn test_grammar_check_rejects_a_value_named_with_an_underscore_and_a_capital() {
        assert_grammar_rejects("module Main;\nmain : IO () = let _X = 1; pure();\n");
        assert_grammar_accepts("module Main;\nmain : IO () = let __X = 1; pure();\n");
    }
}
