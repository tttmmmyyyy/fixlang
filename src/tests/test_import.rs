use crate::{
    tests::util::{test_source, test_source_fail},
    Configuration,
};

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
        Configuration::compiler_develop_mode(),
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
    test_source(&source, Configuration::compiler_develop_mode());
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
        Configuration::compiler_develop_mode(),
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
    test_source(&source, Configuration::compiler_develop_mode());
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
    test_source(&source, Configuration::compiler_develop_mode());
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
    test_source(&source, Configuration::compiler_develop_mode());
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
        Configuration::compiler_develop_mode(),
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
        Configuration::compiler_develop_mode(),
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
    test_source(&source, Configuration::compiler_develop_mode());
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
    test_source(&source, Configuration::compiler_develop_mode());
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
        Configuration::compiler_develop_mode(),
        "Name confliction: `Main::Piyo` is both a type and a trait.",
    );
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
        Configuration::compiler_develop_mode(),
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
        Configuration::compiler_develop_mode(),
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
        Configuration::compiler_develop_mode(),
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
        Configuration::compiler_develop_mode(),
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
        Configuration::compiler_develop_mode(),
        "Namespace `Std::Piyo` is not defined or empty.",
    );
}
