use crate::{
    tests::test_util::{
        run_source_assert_failed, test_source, test_source_fail, test_sources, test_sources_fail,
    },
    Configuration,
};

/// A module holding one value of every name shape an absolute path can end in: a name headed by a
/// lowercase letter, one headed by `_`, one headed by `@` (the getter the compiler defines for a
/// struct field), and a capitalized one (a type).
const LIB_OF_EVERY_NAME_SHAPE: &str = r##"
    module Lib;

    answer : I64;
    answer = 42;

    _answer : I64;
    _answer = 42;

    type Box2 = unbox struct { v : I64 };
"##;

#[test]
pub fn test_import_empty() {
    let source = r##"
    module Main;
    import Std::{};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type or associated type name",
    );
}

#[test]
pub fn test_import_any() {
    let source = r##"
    module Main;
    import Std::*;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_import_hiding_any() {
    let source = r##"
    module Main;
    import Std::* hiding *;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type or associated type name",
    );
}

#[test]
pub fn test_import_only_necessary() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::println, Array, U8};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_import_hierarchy() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, Array, U8, IO::{println, eprintln}};

    main : IO ();
    main = (
        eprintln("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_import_any_in_namespace() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::*, Array, U8};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_import_insufficient() {
    let source = r##"
    module Main;
    import Std::{Tuple0, IO::println};

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type or associated type name `IO`.",
    );
}

#[test]
pub fn test_import_hiding_necessary() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, String, IO::println} hiding IO;

    main : IO ();
    main = (
        println("Hello, World!")
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Unknown type or associated type name `IO`.",
    );
}

#[test]
pub fn test_import_hiding_unnecessary() {
    let source = r##"
    module Main;
    import Std hiding Tuple2;

    type Tuple2 a b = struct { fst : a, snd : b };

    impl [a : ToString, b : ToString] Tuple2 a b : ToString {
        to_string = |t| "(" + t.@fst.to_string + ", " + t.@snd.to_string + ")";
    }

    main : IO ();
    main = println $ Tuple2 { fst : "Hello", snd : "World!" }.to_string;
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_import_hiding_associated_type() {
    let source = r##"
    module Main;
    import Std hiding Iterator::Item;

    type Item = I64;

    main : IO ();
    main = (
        assert_eq(|_|"", 42 : Item, 42 : I64)
    );
    "##;
    test_source(&source, Configuration::develop_mode());
}

#[test]
pub fn test_type_and_trait_name_collision() {
    let source = r##"
    module Main;

    type Piyo = unbox struct { data : String };
    trait a : Piyo {
        val : a;
    }

    main : IO ();
    main = (
        pure()
    );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Name confliction: `Main::Piyo` is both a type and a trait.",
    );
}

/// A value written under a namespace named after a trait carries the name of that trait's member,
/// and one name defines one value, so the two are reported. Both bodies are wrong for the other's
/// type signature, so accepting either one silently would run a body against a signature it does
/// not have.
#[test]
pub fn test_trait_member_and_value_of_the_traits_namespace_collide() {
    let source = r##"
    module Main;

    trait c : Foo {
        bar : c -> I64;
    }

    impl I64 : Foo {
        bar = |x| x;
    }

    namespace Foo {
        bar : I64 -> String;
        bar = |_| "a value of the trait's namespace";
    }

    main : IO ();
    main = println(Foo::bar(1).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Duplicate definition for global value: `Main::Foo::bar`.",
    );
}

/// A namespace named after a trait holds values of its own, as long as no member of the trait
/// carries the name.
#[test]
pub fn test_value_of_a_traits_namespace_named_after_no_member_is_accepted() {
    let source = r##"
    module Main;

    trait c : Foo {
        bar : c -> I64;
    }

    impl I64 : Foo {
        bar = |x| x;
    }

    namespace Foo {
        baz : I64;
        baz = 1;
    }

    main : IO ();
    main = println((Foo::baz + Foo::bar(2)).to_string);
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A trait member and a value of another namespace carrying the member's name are two values: a
/// member is registered under the trait's namespace followed by the member's name, so only a value
/// of that same namespace meets it.
#[test]
pub fn test_trait_member_and_value_of_another_namespace_are_two_values() {
    let source = r##"
    module Main;

    trait c : Foo {
        bar : c -> I64;
    }

    impl I64 : Foo {
        bar = |x| x;
    }

    namespace Baz {
        bar : I64 -> String;
        bar = |_| "a value of another namespace";
    }

    main : IO ();
    main = println(Foo::bar(1).to_string + Baz::bar(2));
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A value of a namespace under the trait's carries a name of its own: a member is registered under
/// the trait's namespace followed by the member's name, and a deeper namespace is another one.
#[test]
pub fn test_value_of_a_namespace_under_the_traits_namespace_is_a_value_of_its_own() {
    let source = r##"
    module Main;

    trait c : Foo {
        bar : c -> I64;
    }

    impl I64 : Foo {
        bar = |x| x;
    }

    namespace Foo::Inner {
        bar : I64 -> String;
        bar = |_| "a value of a namespace under the trait's";
    }

    main : IO ();
    main = println(Foo::bar(1).to_string + Foo::Inner::bar(2));
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Every member of a trait whose name a value of the trait's namespace also carries is reported,
/// so one compilation shows each name to change instead of one per run.
#[test]
pub fn test_every_trait_member_colliding_with_a_value_is_reported_in_one_compilation() {
    let source = r##"
    module Main;

    trait c : Foo {
        bar : c -> I64;
        baz : c -> I64;
    }

    namespace Foo {
        bar : I64 -> String;
        bar = |_| "a value named after the first member";

        baz : I64 -> String;
        baz = |_| "a value named after the second member";
    }

    main : IO ();
    main = println(Foo::bar(1) + Foo::baz(2));
    "##;
    let errmsg = run_source_assert_failed(&source, Configuration::develop_mode());
    for colliding_name in ["Main::Foo::bar", "Main::Foo::baz"] {
        let expected_report = format!(
            "Duplicate definition for global value: `{}`.",
            colliding_name
        );
        assert!(
            errmsg.contains(&expected_report),
            "`{}` is expected to be reported, but the message is:\n{}",
            colliding_name,
            errmsg
        );
    }
}

#[test]
pub fn test_import_unknown_module() {
    let source = r##"
    module Main;

    import Piyo;

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Cannot find module `Piyo`.",
    );
}

#[test]
pub fn test_import_unknown_symbol() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, piyo};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Cannot find value named `Std::piyo`.",
    );
}

#[test]
pub fn test_import_unknown_symbol_hiding() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure} hiding piyo;

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Cannot find value named `Std::piyo`.",
    );
}

#[test]
pub fn test_import_unknown_type_or_trait() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, Piyo};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Cannot find entity named `Std::Piyo`.",
    );
}

#[test]
pub fn test_import_unknown_namespace() {
    let source = r##"
    module Main;
    import Std::{IO, Tuple0, Monad::pure, Piyo::*};

    main : IO ();
    main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Namespace `Std::Piyo` is not defined or empty.",
    );
}

#[test]
pub fn test_absolute_path_reaches_every_value_name_shape_without_an_import() {
    let main = r##"
    module Main;

    main : IO ();
    main = (
        eval *assert_eq(|_|"lowercase", ::Lib::answer, 42);
        eval *assert_eq(|_|"underscore", ::Lib::_answer, 42);
        eval *assert_eq(|_|"getter", ::Lib::Box2::@v(::Lib::Box2 { v : 7 }), 7);
        pure()
    );
    "##;
    test_sources(
        &[main, LIB_OF_EVERY_NAME_SHAPE],
        Configuration::develop_mode(),
    );
}

#[test]
pub fn test_absolute_path_to_an_undefined_value_of_another_module_is_reported_as_a_value() {
    let main = r##"
    module Main;

    main : IO ();
    main = println(::Lib::_missing.to_string);
    "##;
    test_sources_fail(
        &[main, LIB_OF_EVERY_NAME_SHAPE],
        Configuration::develop_mode(),
        "Cannot find value named `Lib::_missing`.",
    );
}

#[test]
pub fn test_absolute_path_to_an_undefined_type_of_another_module_is_reported_as_an_entity() {
    let main = r##"
    module Main;

    main : IO ();
    main = (
        let x : ::Lib::Missing = 0;
        println(x.to_string)
    );
    "##;
    test_sources_fail(
        &[main, LIB_OF_EVERY_NAME_SHAPE],
        Configuration::develop_mode(),
        "Cannot find entity named `Lib::Missing`.",
    );
}
