//! How a type error reads: the types that do not match, or the constraint that cannot be deduced,
//! open the report, and the type variables the inference created are printed as `#1`, `#2`, ... in
//! the order the report mentions them.
//!
//! An expected text that starts with `: ` is matched right after the label `error`, so it pins the
//! sentence that opens the report.

use crate::{
    configuration::Configuration,
    constants::BIND_RESULT_VAR_PREFIX,
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
/// pattern's type and the parameter's, and the type the lambda is expected to have, which shows the
/// parameters the lambda is missing.
#[test]
pub fn test_a_parameter_pattern_mismatch_names_the_pattern_and_the_lambda_type() {
    test_source_fail(
        PARAM_PATTERN_ON_FOLD_ELEMENT,
        Configuration::develop_mode(),
        ": This pattern matches a value of type `(#1, #2)`, but the parameter of the lambda has type `Std::I64`.\n\
         The lambda is expected to have type `Std::I64 -> (Std::Array #3, Std::I64) -> (Std::Array #3, Std::I64)`.",
    );
    test_source_fail(
        PARAM_PATTERN_ON_FOLD_ELEMENT,
        Configuration::develop_mode(),
        "`#1` is the type expected for `arr`.",
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
         This pattern matches a value of type `(Std::String, #1)`, but the parameter of the lambda has type `(Std::I64, Std::I64)`.",
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
        ": Expected `Std::String`, found `Std::I64`.\n",
    );
}

/// A boolean literal used where another type is expected is reported as a `Std::Bool` at the
/// literal, the way a number literal is.
#[test]
pub fn test_a_mismatched_boolean_literal_is_one_sentence_finding_bool_at_the_literal() {
    let source = r##"
module Main;

main : IO () = (
    let x = [1];
    let y = [x.@size, true];
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Expected `Std::I64`, found `Std::Bool`.\n\n6:23-6:27 in ",
    );
    let source = r##"
module Main;

main : IO () = (
    let f = |x : I64| x;
    let y = f(false);
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Expected `Std::I64`, found `Std::Bool`.\n\n6:15-6:20 in ",
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
        ": Cannot infer the type of this pattern: inferred as `Std::Array #1`, but the type variable `#1` is unresolved.",
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
        ": Expected expression of type `#1`.",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `?`.",
    );
}

/// A type variable of the expected type is printed under one name in every line of a report
/// listing the values of a name.
#[test]
pub fn test_a_type_variable_of_the_expected_type_is_named_alike_in_every_listed_line() {
    let source = r##"
module Main;

trait a : Foo {
    describe : a -> String;
}

namespace A {
    f : I64 -> String = |x| x.to_string;
}

namespace B {
    f : [a : Foo] a -> String = |x| x.describe;
}

main : IO () = (
    let x = [];
    println(x.f)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": No value named `f` matches the expected type `Std::Array #1 -> Std::String`.\n\
         - (1) `Main::A::f` of type `Std::I64 -> Std::String`: `Std::I64` and `Std::Array #1` do not match.\n\
         - (2) `Main::B::f` of type `[a : Main::Foo] a -> Std::String`: `Std::Array #1 : Main::Foo` cannot be deduced.",
    );
}

/// A type variable a value's scheme generalizes gets no pointer to a source, even where a type
/// variable of the enclosing signature has its name.
#[test]
pub fn test_a_generalized_type_variable_of_a_value_is_not_located() {
    let source = r##"
module Main;

h : [a : ToString] a -> I64 = |x| (
    let y : String = Option::is_some;
    x.to_string.get_size
);

main : IO () = println(h(1).to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`Std::Option::is_some` of type `Std::Option a -> Std::Bool` does not match the expected type `Std::String`.",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "`a` is the ");
}

/// A type variable the signature writes gets no pointer to a source where a parameter pattern of
/// another type is matched against it, while the type variables the inference created for the
/// pattern still get theirs.
#[test]
pub fn test_a_type_variable_of_the_signature_is_not_located_at_a_mismatched_parameter_pattern() {
    let source = r##"
module Main;

g : [a : ToString] a -> String;
g = |(x, y)| x.to_string;

main : IO () = println(g(1));
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `x`.",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "`a` is the ");
}

/// A type variable the signature writes gets no pointer to a source, even at the parameter whose
/// type it is.
#[test]
pub fn test_a_type_variable_of_the_signature_is_not_located_at_its_parameter() {
    let source = r##"
module Main;

g : [a : ToString] a -> String;
g = |x| (x + x).to_string;

main : IO () = println(g(1));
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`a : Std::Add` cannot be deduced.",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "`a` is the ");
}

/// A parameter pattern written with a type wildcard is reported without the name the parser gives
/// the wildcard.
#[test]
pub fn test_a_parameter_pattern_with_a_type_wildcard_is_reported_without_its_inner_name() {
    let source = r##"
module Main;

f : I64 -> I64 = |(x : _, y)| y;

main : IO () = println(f(1).to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": This pattern matches a value of type `(#1, #2)`, but the parameter of the lambda has type `Std::I64`.",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "#typewildcard");
}

/// The result of a `*x` that does not match the expected type is reported over the `*x`, quoting
/// it, without the name of the variable the parser binds the result to.
#[test]
pub fn test_a_mismatched_result_of_a_bind_is_reported_by_its_source() {
    let source = r##"
module Main;

main : IO () = (
    let (a, b) = *pure((1, 2, 3));
    println(a.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`*pure((1, 2, 3))` of type `(Std::I64, Std::I64, Std::I64)` does not match the expected type `(#1, #2)`.\n\n5:18-5:34 in ",
    );
    test_source_fail_excludes(
        source,
        Configuration::develop_mode(),
        BIND_RESULT_VAR_PREFIX,
    );
}

/// The result of a `*x` too long to quote is reported as the expression the source pointer marks.
#[test]
pub fn test_a_mismatched_result_of_a_long_bind_is_reported_as_this_expression() {
    let source = r##"
module Main;

main : IO () = (
    let s : String = *pure((1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12));
    println(s)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": This expression of type `(Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64, Std::I64)` does not match the expected type `Std::String`.\n\n5:22-5:68 in ",
    );
}

/// When the function of an application fails to check before its argument is checked, the type
/// variable for the argument still points at the argument, as the type expected for it.
#[test]
pub fn test_an_argument_unchecked_at_the_error_is_located_as_the_expression_its_type_variable_is_expected_for(
) {
    let source = r##"
module Main;

g : I64 -> Bool;
g = |x| let h = 1; h(x);

main : IO () = println(g(1).to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`h` of type `Std::I64` does not match the expected type `#1 -> Std::Bool`.",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `x`.\n\n5:22-5:23 in ",
    );
}

/// The type variable for a variadic argument of `FFI_CALL` points at that argument.
#[test]
pub fn test_the_type_expected_for_a_variadic_argument_of_an_ffi_call_is_located_at_the_argument() {
    let source = r##"
module Main;

main : IO () = (
    let v = FFI_CALL[I32 printf(Ptr, ...), nullptr, undefined("")];
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `undefined(\"\")`.\n\n5:53-5:66 in ",
    );
}

/// A type variable that replaces a variable of a global value's type, where an expression uses the
/// value, is described by the variable it replaces and the value, and points at the use. Here the
/// lambda returns the state where `fold_m` expects it wrapped in its monad `m`.
#[test]
pub fn test_a_type_variable_from_the_type_of_a_global_value_names_the_variable_and_the_value() {
    let source = r##"
module Main;

type H k v = box struct { ks : Array k, vs : Array v };

new_h : I64 -> H k v;
new_h = |_| H { ks : [], vs : [] };

insert : k -> v -> H k v -> H k v;
insert = |k, v, h| h.mod_ks(push_back(k)).mod_vs(push_back(v));

main : IO ();
main = (
    let memo = new_h(10);
    let memo = *range(2, 11).fold_m(memo, |n, memo|
        range(n+1, 11).fold(memo, |m, memo|
            let memo = memo.insert(n, m);
            memo
        )
    );
    println(memo.@ks.get_size.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`memo` of type `Main::H #2 #1` does not match the expected type `#3 (Main::H #2 #1)`.",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is `v` in the type of `Main::new_h`.\n\n14:16-14:21 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#2` is `k` in the type of `Main::new_h`.\n\n14:16-14:21 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#3` is `m` in the type of `Std::Iterator::fold_m`.\n\n15:30-15:36 in ",
    );
}

/// A type variable that replaces a variable the compiler named in a global value's type, such as
/// the functor of a field's `act_` function, gets no sentence about that variable, since the
/// sentence would quote a name the source never writes.
#[test]
pub fn test_a_type_variable_from_a_variable_the_compiler_named_is_not_explained() {
    let source = r##"
module Main;

type H = box struct { n : I64 };

main : IO () = (
    let h = H { n : 1 };
    let h = h.act_n(|n| n + 1);
    println(h.@n.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`n` of type `Std::I64` does not match the expected type `#1 Std::I64`.",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "` is `#");
}

/// Verifies that where the type of a global value an expression uses does not match the expected
/// type, the error explains the type variables of the value's type.
#[test]
pub fn test_a_type_variable_from_the_type_of_a_global_value_that_does_not_match_is_explained() {
    let source = r##"
module Main;

main : IO ();
main = (
    let n = true.Std::Array::get_size;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is `a` in the type of `Std::Array::get_size`.\n\n6:18-6:38 in ",
    );
}

/// Verifies that where no value of an overloaded name matches, the error explains a type variable
/// of a value's type with the number of that value's line, as in `in (1)`.
#[test]
pub fn test_a_type_variable_from_the_type_of_a_global_value_is_explained_in_its_line_of_the_overloads(
) {
    let source = r##"
module Main;

get_size : I64 -> I64;
get_size = |x| x;

main : IO ();
main = (
    let n = true.get_size;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "- (1) `Std::Array::get_size` of type `Std::Array a -> Std::I64`: `Std::Array #2` and `Std::Bool` do not match.",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#2` in (1) is `a` in the type of `Std::Array::get_size`.\n\n9:18-9:26 in ",
    );
}

/// Verifies that two type variables of the expected type that the failed check of a value unified
/// with each other still point at their own expressions, since the report prints them apart.
#[test]
pub fn test_type_variables_the_failed_check_unified_point_at_their_own_expressions() {
    let source = r##"
module Main;

same2 : (Option a, Option a) -> I64;
same2 = |_| 0;

main : IO ();
main = (
    let s : String = (Option::none(), Option::none()).same2;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is `a` in the type of `Std::Option::none`.\n\n9:23-9:35 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#2` is `a` in the type of `Std::Option::none`.\n\n9:39-9:51 in ",
    );
}

/// Verifies that the type variables of the expected type of a value whose check failed keep the
/// sources the outer context records for them, where the check unified one with another.
#[test]
pub fn test_type_variables_the_failed_check_unified_keep_the_sources_of_the_outer_context() {
    let source = r##"
module Main;

same3 : (a, a) -> I64;
same3 = |_| 0;

f : I64 -> String;
f = |n| ( let g = |x, y| ( let s : String = (x, y).same3; s ); g(1, 2) );

main : IO ();
main = println(f(0));
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `x`.\n\n8:20-8:21 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#2` is the type expected for `y`.\n\n8:23-8:24 in ",
    );
}

/// Verifies that where no value of an overloaded name matches and two of the values have type
/// variables of their own, each line's variable is explained by the value of that line, even though
/// the two checks name their variables alike.
#[test]
pub fn test_type_variables_of_two_overloaded_values_are_each_explained_in_their_own_line() {
    let source = r##"
module Main;

namespace A {
    conv : Array a -> a;
    conv = |xs| xs.@(0);
}

namespace B {
    conv : Option b -> b;
    conv = |o| o.as_some;
}

main : IO () = (
    let n : I64 = true.conv;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` in (1) is `a` in the type of `Main::A::conv`.\n\n15:24-15:28 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` in (2) is `b` in the type of `Main::B::conv`.\n\n15:24-15:28 in ",
    );
}

/// A trait constraint that the type of a value required, and that is left undeduced until the next
/// name is checked, is reported at the use of that value, with that value's type.
#[test]
pub fn test_a_constraint_failing_at_a_later_name_is_reported_at_the_value_that_required_it() {
    let source = r##"
module Main;

main : IO () = (
    let x = 1.0 % 2.0;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` cannot be deduced.\n\
         `Std::Rem::rem` of type `[a : Std::Rem] a -> a -> a` does not match the expected type `Std::F64 -> Std::F64 -> Std::F64`.\n\n\
         5:17-5:18 in ",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "Std::Monad::pure");
}

/// A constraint that an instance asks for, on the way to deducing a constraint a value required, is
/// reported at the use of that value.
#[test]
pub fn test_a_constraint_an_instance_asks_for_is_reported_at_the_value_that_required_the_first() {
    let source = r##"
module Main;

trait a : Foo {
    foo : a -> I64;
}

impl [a : Foo] Array a : Foo {
    foo = |xs| xs.get_size;
}

main : IO () = (
    let n = foo([1.0]);
    println(n.to_string)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Main::Foo` cannot be deduced.\n\
         `Main::Foo::foo` of type `[a : Main::Foo] a -> Std::I64` does not match the expected type `Std::Array Std::F64 -> Std::I64`.\n\n\
         13:13-13:16 in ",
    );
}

/// A constraint left from an earlier expression that fails for every value an overloaded name
/// stands for is reported once, at the use of the value that required it.
#[test]
pub fn test_a_constraint_every_value_of_an_overloaded_name_fails_on_is_reported_at_its_value() {
    let source = r##"
module Main;

namespace A {
    g : I64 -> IO ();
    g = |n| println(n.to_string);
}

namespace B {
    g : I64 -> IO ();
    g = |n| println((n + 1).to_string);
}

main : IO () = (
    let x = 1.0 % 2.0;
    g(1)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` cannot be deduced.\n\
         `Std::Rem::rem` of type `[a : Std::Rem] a -> a -> a` does not match the expected type `Std::F64 -> Std::F64 -> Std::F64`.\n\n\
         15:17-15:18 in ",
    );
}

/// A trait constraint left unsettled to the end of the body it is checked in is reported at the use
/// of the value that required it.
#[test]
pub fn test_a_constraint_failing_at_the_end_of_the_body_is_reported_at_the_value_that_required_it()
{
    let source = r##"
module Main;

f : I64 = (
    let x = 1.0 % 2.0;
    0
);

main : IO () = println(f.to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` is required in the type inference of this expression but cannot be deduced from assumptions.\n\n\
         5:17-5:18 in ",
    );
}

/// A trait constraint that the type of a value states, and that an associated type in the same type
/// asks for as well, is reported at the use of that value.
#[test]
pub fn test_a_constraint_an_associated_type_also_asks_for_is_reported_at_the_value_that_states_it()
{
    let source = r##"
module Main;

main : IO () = (
    let x = Iterator::get_first(1.0);
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Iterator` cannot be deduced.\n\
         `Std::Iterator::get_first` of type `[iter : Std::Iterator] iter -> Std::Option (Std::Iterator::Item iter)` does not match the expected type `Std::F64 -> Std::Option (Std::Iterator::Item Std::F64)`.\n\n\
         5:13-5:32 in ",
    );
}

/// A trait constraint that the type of a value states, and that an associated type in that type asks
/// for as well, is reported at the use of that value when it fails at a later name, whichever of the
/// two asks for the constraint first.
#[test]
pub fn test_a_constraint_an_associated_type_asks_for_failing_at_a_later_name_is_reported_at_its_value(
) {
    let source = r##"
module Main;

h : [a : Iterator] a -> Option (Iterator::Item a) -> I64;
h = |_, _| 0;

main : IO () = (
    let n = h(1.0, none());
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Iterator` cannot be deduced.\n`Main::h` of type ",
    );
    test_source_fail(source, Configuration::develop_mode(), "\n\n8:13-8:14 in ");
    test_source_fail_excludes(source, Configuration::develop_mode(), "Std::Option::none");
}

/// A trait constraint that the type of a value states, and that an associated type in that type asks
/// for as well, is reported at the use of that value when it is left unsettled to the end of the
/// body.
#[test]
pub fn test_a_constraint_an_associated_type_asks_for_failing_at_the_end_of_the_body_is_reported_at_its_value(
) {
    let source = r##"
module Main;

f : I64 = (
    let x = Iterator::get_first(1.0);
    0
);

main : IO () = println(f.to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Iterator` is required in the type inference of this expression but cannot be deduced from assumptions.\n\n\
         5:13-5:32 in ",
    );
}

/// A trait constraint whose deduction is circular, left from an earlier expression, is reported at
/// the use of the value that required it.
#[test]
pub fn test_a_circular_constraint_failing_at_a_later_name_is_reported_at_the_value_that_required_it(
) {
    let source = r##"
module Main;

trait a : Marker {}

type Foo a = unbox struct { x : a };

impl [Foo a : Marker] Foo a : Marker {}

need_marker : [a : Marker] a -> I64;
need_marker = |_| 0;

main : IO ();
main = println(need_marker(Foo { x : 42 }).to_string);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Main::Foo Std::I64 : Main::Marker` cannot be deduced. The inference is circular.\n\
         `Main::need_marker` of type ",
    );
    test_source_fail(source, Configuration::develop_mode(), "\n\n14:16-14:27 in ");
}

/// A trait constraint whose deduction never ends, left from an earlier expression, is
/// reported at the use of the value that required it.
#[test]
pub fn test_an_endless_constraint_failing_at_a_later_name_is_reported_at_the_value_that_required_it(
) {
    let source = r##"
module Main;

trait a : Grow {}

type W a = unbox struct { x : a };

impl [W (W a) : Grow] W a : Grow {}

need_grow : [a : Grow] a -> I64;
need_grow = |_| 0;

main : IO () = (
    let h = |w| need_grow(w);
    let n = h(W { x : 1 });
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Main::W Std::I64 : Main::Grow` cannot be deduced. The inference is too long: ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`Main::need_grow` of type `[a : Main::Grow] a -> Std::I64` does not match the expected type `Main::W Std::I64 -> Std::I64`.\n\n\
         14:17-14:26 in ",
    );
}

/// A trait constraint that the type of a name's value requires, and that fails at that name, is
/// reported with the type the name's context expected before the name was checked.
#[test]
pub fn test_a_constraint_failing_at_the_name_that_required_it_quotes_the_type_its_context_expects()
{
    let source = r##"
module Main;

main : IO () = (
    let r = 1.0.rem;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` cannot be deduced.\n\
         `Std::Rem::rem` of type `[a : Std::Rem] a -> a -> a` does not match the expected type `Std::F64 -> #1`.\n\n\
         5:17-5:20 in ",
    );
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`#1` is the type expected for `r`.\n\n5:9-5:10 in ",
    );
}

/// A trait constraint that an instance asks for, on the way to a constraint that both the type of a
/// value and an associated type in it require, is reported at the use of that value, whichever of
/// the two asks for the constraint first.
#[test]
pub fn test_a_constraint_an_instance_asks_for_after_an_associated_type_is_reported_at_the_value() {
    let source = r##"
module Main;

trait c : Container {
    type Content c;
    size_of : c -> I64;
}

type Bag a = box struct { items : Array a };

impl [a : ToString] Bag a : Container {
    type Content (Bag a) = a;
    size_of = |b| b.@items.get_size;
}

type P = struct { v : I64 };

impl P : Eq {
    eq = |x, y| x.@v == y.@v;
}

get : [c : Container] c -> Content c;
get = |_| undefined("");

same : [d : Eq] d -> d -> Bool;
same = |x, y| x == y;

main : IO () = (
    let f = |b| (
        let x = get(b);
        let y = same(x, x);
        let z = 0;
        let _ = z;
        0
    );
    let bag = Bag { items : [] };
    let n = f(bag);
    let w : Bag P = bag;
    pure()
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Main::P : Std::ToString` cannot be deduced.\n\
         `Main::get` of type `[c : Main::Container] c -> Main::Container::Content c` does not match ",
    );
    test_source_fail(source, Configuration::develop_mode(), "\n\n30:17-30:20 in ");
}

/// Of the trait constraints left unsettled to the end of a body, one that the type of a value
/// required is reported, at the use of that value.
#[test]
pub fn test_of_the_constraints_failing_at_the_end_of_the_body_one_a_value_required_is_reported() {
    let source = r##"
module Main;

f : [a : Iterator] a -> Bool;
f = |it| (
    let x = it.get_first.as_some;
    let b = x == x;
    let y : Option (Iterator::Item (Iterator::Item a)) = none();
    b
);

main : IO () = pure();
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::Iterator::Item a : Std::Eq` is required in the type inference of this expression but cannot be deduced from assumptions.\n\n\
         7:15-7:17 in ",
    );
}

/// A trait constraint left from an earlier expression, whose type an earlier expression settled, is
/// reported at the use of the value that required it when the next name is overloaded and one of
/// its values fails to match for a reason of its own.
#[test]
pub fn test_a_constraint_failing_before_an_overloaded_name_is_checked_is_reported_at_the_value_that_required_it(
) {
    let source = r##"
module Main;

namespace A {
    g : I64 -> IO ();
    g = |n| println(n.to_string);
}

namespace B {
    g : I64 -> I64;
    g = |n| n;
}

main : IO () = (
    let x = 1.0 % 2.0;
    g(1)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` cannot be deduced.\n\
         `Std::Rem::rem` of type `[a : Std::Rem] a -> a -> a` does not match the expected type ",
    );
    test_source_fail(source, Configuration::develop_mode(), "\n\n15:17-15:18 in ");
    test_source_fail_excludes(source, Configuration::develop_mode(), "No value named `g`");
}

/// A trait constraint left from an earlier expression, whose type an earlier expression settled, is
/// reported at the use of the value that required it when the next name has one value, which fails
/// to match for a reason of its own.
#[test]
pub fn test_a_constraint_failing_before_a_name_of_one_value_is_checked_is_reported_at_the_value_that_required_it(
) {
    let source = r##"
module Main;

g : I64 -> I64;
g = |n| n;

main : IO () = (
    let x = 1.0 % 2.0;
    g(1)
);
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": `Std::F64 : Std::Rem` cannot be deduced.\n\
         `Std::Rem::rem` of type `[a : Std::Rem] a -> a -> a` does not match the expected type `Std::F64 -> Std::F64 -> Std::F64`.\n\n\
         8:17-8:18 in ",
    );
    test_source_fail_excludes(source, Configuration::develop_mode(), "Main::g");
}
