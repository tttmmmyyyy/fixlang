//! A diagnostic that quotes a type written in the program quotes it as written: its type variables
//! keep the names the program gives them, so the quoted type reads the same as the source the
//! diagnostic points at.

use crate::{configuration::Configuration, tests::test_util::test_source_fail};

/// A type applied to an argument its kind does not take.
#[test]
pub fn test_a_kind_mismatch_quotes_the_type_variables_as_written() {
    let source = r##"
module Main;

f : [t : ToString] Array t t -> String;
f = |x| "";

main : IO ();
main = println(f([1]));
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Kind mismatch in `Std::Array t t`. Type `Std::Array t` of kind `*` cannot be applied to type `t` of kind `*`.",
    );
}

/// A type alias given fewer arguments than it takes.
#[test]
pub fn test_an_alias_not_fully_applied_is_quoted_with_the_type_variables_as_written() {
    let source = r##"
module Main;

type Hoge q r = q -> r;

f : [k : ToString] Hoge k -> I64;
f = |x| 0;

main : IO ();
main = pure();
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Cannot resolve type alias `Main::Hoge` in `Main::Hoge k` because it is not fully applied.",
    );
}

/// Type aliases that expand into each other.
#[test]
pub fn test_a_circular_alias_is_quoted_with_the_type_variables_as_written() {
    let source = r##"
module Main;

type Hoge q = Fuga q;
type Fuga r = Hoge r;

f : [z : ToString] Hoge z -> I64;
f = |x| 0;

main : IO ();
main = pure();
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Circular type aliasing is found in `Main::Hoge z`.",
    );
}

/// A trait of another module implemented for a type of another module.
#[test]
pub fn test_an_implementation_against_the_orphan_rule_quotes_the_type_as_written() {
    let source = r##"
module Main;

impl Array elm : ToString {
    to_string = |_| "array!";
}

main : IO ();
main = pure();
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Implementing trait `Std::ToString` for type `Std::Array elm` in module `Main` is illegal;",
    );
}

/// An associated type given an argument of a kind its parameter does not take.
#[test]
pub fn test_a_kind_mismatch_in_an_associated_type_quotes_the_type_variables_as_written() {
    let source = r##"
module Main;

trait a : MyTrait {
    type [f : *->*] MyAssoc a f;
}

impl I64 : MyTrait {
    type MyAssoc I64 f = f I64;
}

func : [s : MyTrait] (s, MyAssoc s String) -> I64;
func = |x| 0;

main : IO ();
main = pure();
"##;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        ": Kind mismatch in `Main::MyTrait::MyAssoc s Std::String`. Expected `*->*`, found `*`.",
    );
}
