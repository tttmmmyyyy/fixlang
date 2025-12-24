use crate::{test_source, test_source_fail, Configuration};

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

#[test]
pub fn test_import_required_using_string_literal() {
    // Using string literal requires neither `String`, `U8` nor `Array`.
    let source = r##"
module Main;

import Std::{Monad::pure, IO, Tuple0};

main : IO ();
main = eval "Hello, World!"; pure();
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_integer_literal() {
    // Using integer literal requires neither `I64` nor `U64`.
    let source = r##"
module Main;

import Std::{Monad::pure, IO, Tuple0};

main : IO ();
main = eval 42; eval 42_U64; pure();
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_array_literal() {
    // Using array literal does not require `Array`
    let source = r##"
module Main;

import Std::{Monad::pure, IO, Tuple0};

main : IO ();
main = eval [42]; pure();
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_struct_union_method() {
    // Using struct/union methods defined in the module itself does not require importing anything.
    let source = r##"
module Main;

import Std::{Monad::pure, Option::some, I64, IO, Tuple0};

type A = struct { x : I64 };
type B = union { x : I64 };

main : IO ();
main = (
    let a = A { x: 42 };
    eval a.@x;
    eval a.set_x(100);
    eval a.mod_x(|_| 31);
    eval a.act_x(|x| some(x));

    let b = B::x(42);
    eval b.is_x;
    eval b.as_x;
    eval b.mod_x(|_| 31);    

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_index_syntax() {
    // Using index syntax does not require importing anything.
    let source = r##"
module Main;

import Std::{Indexable::iget, Monad::pure, I64, IO, Tuple0};

type A = struct { x : I64 };

main : IO ();
main = (
    let a = A { x: 42 };
    eval [a][0][^x].iget;

    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_ffi_call_io() {
    // Using FFI_CALL_IO does not require importing anything.
    let source = r##"
module Main;

import Std::{Monad::pure, Tuple0};

main : ::Std::IO ();
main = (
    eval FFI_CALL_IO[CInt rand()];
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_ffi_call_ios() {
    // Using FFI_CALL_IOS does not require importing anything.
    let source = r##"
module Main;

import Std::{IO::IOState::_unsafe_create, Monad::pure, I32, I64, IO, Tuple0};

main : IO ();
main = (
    eval FFI_CALL_IOS[CInt rand(), IOState::_unsafe_create];
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_operators() {
    // Using operators does not require importing anything.
    let source = r##"
module Main;

import Std::{Monad::pure, IO, Tuple0};

main : IO ();
main = (
    let _ = *pure();
    let f = |x| x + 1; eval f >> f; eval f << f;
    let x = 42; eval -x;
    eval !true;
    eval 1 * 2;
    eval 2 / 1;
    eval 2 % 1;
    eval 2 + 1;
    eval 2 - 1;
    eval 2 == 1;
    eval 2 != 1;
    eval 2 <= 1;
    eval 2 >= 1;
    eval 2 < 1;
    eval 2 > 1;
    eval true && false;
    eval true || false;
    let f = |x| x + 1; eval f $ 1;
    pure();;
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_tuples() {
    // Using tuple literals does not require importing anything.
    let source = r##"
module Main;

import Std::{Monad::pure, IO};

main : IO ();
main = (
    eval ();
    eval (1,2);
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_arrow_syntax() {
    // Using tuple literals does not require importing anything.
    let source = r##"
module Main;

import Std::{Monad::pure, IO, I64};

f : I64 -> I64 = |x| x + 1;

main : IO ();
main = (
    eval f(41);
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_absolute_names() {
    // Using absolute does not require importing anything.
    let source = r##"
module Main;

import Std::{}; // Hide all standard library entities

main : ::Std::IO ();
main = ::Std::Monad::pure();
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_trait_alias() {
    // Using trait alias does not require importing the underlying traits.
    let source = r##"
module Main;

import Std::{IO, Additive, undefined, Monad::pure, I64}; // No need to import Std::Zero and Std::Add.

// Use in type signature
my_add : [a : Additive] a -> a -> a;
my_add = |_, _| undefined("");

// Use in trait alias definition
trait MyAdditive = Additive;

// Use in trait definition
trait a : MyTrait {
    add : [b : Additive] a -> b -> b -> b;
}

// Use in trait implementation type signature
impl I64 : MyTrait {
    add : [b : Additive] I64 -> b -> b -> b;
    add = |_, x, y| x + y;
}

// Use in trait implementation precondition
type MyType a = struct { data : a };
impl [a : Additive] MyType a : MyTrait {
    add = |_, x, y| x + y;
}

main : IO ();
main = (
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}

#[test]
pub fn test_import_required_using_type_alias() {
    // Using type alias does not require importing the underlying traits.
    let source = r##"
module Main;

import Std::{Monad::pure, IO, Path};

a : Path;
a = "hoge.fix";

trait a : ToPath {
    to_path : a -> Path;
}

impl Path : ToPath {
    to_path : Path -> Path = |p| p;
}

main: IO ();
main = (
    pure()
);
    "##;
    test_source(&source, Configuration::compiler_develop_mode());
}
