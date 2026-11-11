//! How a type error reads: the types that do not match, or the constraint that cannot be deduced,
//! open the report, and the type variables the inference created are printed as `t1`, `t2`, ... in
//! the order the report mentions them.
//!
//! An expected text that starts with `: ` is matched right after the label `error`, so it pins the
//! sentence that opens the report.

use crate::{
    configuration::Configuration,
    tests::test_util::{test_source_fail, test_source_fail_excludes},
};

/// A lambda whose parameter pattern destructures a tuple, passed where the lambda takes `I64` at
/// that parameter.
const PARAM_PATTERN_ON_FOLD_ELEMENT: &str = r##"
module Main;

decode : I64 -> I64 -> Array I64 = |flag, len| (
    range(0, len).fold(([], flag), |(arr, flag)|
        let arr = arr.push_back(flag % 3 - 1);
        (arr, flag / 3)
    ).@0
);

main : IO () = println(decode(5, 3).to_string);
"##;

/// A parameter pattern that does not match the parameter is reported on the pattern, with the
/// pattern's type and the parameter's, and the type the lambda is expected to have from that
/// parameter on, which shows the parameters the lambda is missing.
#[test]
pub fn test_a_parameter_pattern_mismatch_names_the_pattern_and_the_lambda_type() {
    test_source_fail(
        PARAM_PATTERN_ON_FOLD_ELEMENT,
        Configuration::develop_mode(),
        ": The pattern `(arr, flag)` matches a value of type `(t1, t2)`, but the parameter of the lambda has type `Std::I64`.\n\
         From this parameter on, the lambda is expected to have type `Std::I64 -> (Std::Array t3, Std::I64) -> (Std::Array t3, Std::I64)`.",
    );
    test_source_fail(
        PARAM_PATTERN_ON_FOLD_ELEMENT,
        Configuration::develop_mode(),
        "`t1` is the type for `arr`.",
    );
}

/// The name the parser gives a lambda's parameter, which the program does not write, stays out of
/// the report on a parameter pattern.
#[test]
pub fn test_a_parameter_pattern_mismatch_leaves_out_the_parameter_name_the_parser_gives() {
    test_source_fail_excludes(
        PARAM_PATTERN_ON_FOLD_ELEMENT,
        Configuration::develop_mode(),
        "#param",
    );
}

/// A parameter pattern whose type differs from the parameter's inside the two opens the report with
/// the two types that do not match.
#[test]
pub fn test_a_parameter_pattern_mismatch_inside_the_types_opens_with_the_mismatched_pair() {
    let source = r##"
module Main;

second : (I64, I64) -> I64 = |(a : String, b)| b;

main : IO () = println(second((1, 2)).to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::String` and `Std::I64` do not match.\n\
         The pattern `(a: Std::String, b)` matches a value of type `(Std::String, t1)`, but the parameter of the lambda has type `(Std::I64, Std::I64)`.",
    );
}

/// A name the program gives a type variable is not reused for one the inference created, so the
/// two print apart.
#[test]
pub fn test_a_created_type_variable_skips_a_name_the_program_writes() {
    let source = r##"
module Main;

trait t1 : Foo {
    foo : t1 -> IO ();
}

main : IO () = (
    let x = [];
    x.foo
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::Array t2 : Main::Foo` cannot be deduced.\n\
         `Main::Foo::foo` of type `[t1 : Main::Foo] t1 -> Std::IO ()` does not match the expected type `Std::Array t2 -> Std::IO ()`.",
    );
}

/// A name of several values none of which matches lists each value with the reason it does not.
#[test]
pub fn test_each_value_of_an_overloaded_name_is_listed_with_its_mismatch() {
    let source = r##"
module Main;

namespace A {
    f : I64 -> I64 = |x| x;
}

namespace B {
    f : Bool -> Bool = |x| x;
}

main : IO () = println("a".f);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": No value named `f` matches the expected type `Std::String -> Std::String`.\n\
         - (1) `Main::A::f` of type `Std::I64 -> Std::I64`: `Std::I64` and `Std::String` do not match.\n\
         - (2) `Main::B::f` of type `Std::Bool -> Std::Bool`: `Std::Bool` and `Std::String` do not match.",
    );
}

/// A local value whose type mismatches the expected one as a whole is reported by the one
/// sentence on the two types.
#[test]
pub fn test_a_whole_mismatch_of_a_local_value_is_one_sentence() {
    let source = r##"
module Main;

main : IO () = (
    let n = 3;
    let s : String = n;
    println(s)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `n` of type `Std::I64` does not match the expected type `Std::String`.\n",
    );
}

/// An expression whose type mismatches the expected one as a whole is reported by the one
/// sentence on the two types.
#[test]
pub fn test_a_whole_mismatch_of_an_expression_is_one_sentence() {
    let source = r##"
module Main;

main : IO () = (
    let a : Array String = [1, 2];
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Type mismatch. Expected `Std::String`, found `Std::I64`.\n",
    );
}

/// An expression whose type differs from the expected one inside the two opens the report with the
/// two types that do not match, followed by the expected type and the found one.
#[test]
pub fn test_a_mismatch_inside_the_types_of_an_expression_opens_with_the_mismatched_pair() {
    let source = r##"
module Main;

main : IO () = (
    let n = match (1, "a") { (a, b : I64) => a };
    println(n.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::I64` and `Std::String` do not match.\n\
         Expected `(Std::I64, Std::I64)`, found `(Std::I64, Std::String)`.",
    );
}

/// A type the inference cannot settle is printed, and its unresolved variable named, with the name
/// the report gives that variable.
#[test]
pub fn test_an_unresolved_type_variable_is_named_as_the_inferred_type_prints_it() {
    let source = r##"
module Main;

main : IO () = (
    let x = [];
    assert_eq(|_|"", x.@size, 0)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Cannot infer the type of this pattern: inferred as `Std::Array t1`, but the type variable `t1` is unresolved.",
    );
}

/// A hole whose type the inference leaves open is reported with that type under the name the
/// report gives it, the same name the pointer to its origin uses.
#[test]
pub fn test_a_hole_of_an_open_type_names_the_type_variable() {
    let source = r##"
module Main;

main : IO () = (
    let y : I64 = [?].@size;
    println(y.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Expected expression of type `t1`.",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`t1` is the type for `?`.",
    );
}
