use crate::configuration::Configuration;
use crate::tests::test_util::{test_source, test_source_fail};

// The type of a trait member at an implementation that writes no type signature for it, which the
// compiler derives from the member's declaration and the implementation's head and context. Each
// test writes `{tv}` where the implementation names a type variable, and runs the program once
// with the name of a type variable of the declaration and once with a name of its own: the two
// programs are one program, so they have to give one result.

/// The two names each test gives the implementation's type variable: one the member's declaration
/// also uses, and one it does not.
const IMPL_TYVAR_NAMES: [&str; 2] = ["a", "z"];

/// A variable the implementation's context brings in through an equality stays apart from the
/// member's variable of the same name: the member's `a` stands for the type of the second
/// argument, and the context's `a` for `Elm I64`.
#[test]
pub fn test_member_variable_apart_from_a_context_variable_of_an_equality() {
    let source = r#"
        module Main;

        trait c : Bar {
            get : c -> a -> String;
        }

        trait c : Foo {
            type Elm c;
        }

        impl I64 : Foo {
            type Elm I64 = Bool;
        }

        type NoShow = unbox struct { v : I64 };

        impl [x : Foo, Foo::Elm x = {tv}, {tv} : ToString] Array x : Bar {
            get = |_, y| "ok";
        }

        main : IO ();
        main = (
            assert_eq(|_|"", Bar::get([1, 2], NoShow { v : 3 }), "ok");;
            pure()
        );
    "#;
    for name in IMPL_TYVAR_NAMES {
        test_source(&source.replace("{tv}", name), Configuration::develop_mode());
    }
}

/// A variable the implementation's context constrains without the head fixing it is reported at
/// the implementation, whether or not the member's declaration uses its name.
#[test]
pub fn test_unfixed_context_variable_named_as_a_member_variable_is_reported() {
    let source = r#"
        module Main;

        trait c : Bar {
            get : c -> a -> String;
        }

        type NoShow = unbox struct { v : I64 };
        type W = unbox struct { w : I64 };

        impl [{tv} : ToString] W : Bar {
            get = |_, _| "ok";
        }

        main : IO ();
        main = (
            assert_eq(|_|"", Bar::get(W { w : 1 }, NoShow { v : 3 }), "ok");;
            pure()
        );
    "#;
    for name in IMPL_TYVAR_NAMES {
        test_source_fail(
            &source.replace("{tv}", name),
            Configuration::develop_mode(),
            &format!(
                "Type variable `{}` is not fixed by this type signature",
                name
            ),
        );
    }
}

/// A kind signature of the member's declaration stays with the member's variable, apart from a
/// variable of the implementation's head of the same name and another kind.
#[test]
pub fn test_member_kind_signature_apart_from_a_head_variable() {
    let source = r#"
        module Main;

        type MyPair a b = box struct { p : a, q : b };

        type [f : *->*->*] Wrap2 f = box struct { x : f I64 I64 };

        trait c : Describe {
            describe : [g : *->*] g I64 -> c -> I64;
        }

        impl [{tv} : *->*->*] Wrap2 {tv} : Describe {
            describe = |_, _| 42;
        }

        impl Array {tv} : Describe {
            describe = |_, _| 43;
        }

        main : IO ();
        main = (
            let w : Wrap2 MyPair = Wrap2 { x : MyPair { p : 1, q : 2 } };
            assert_eq(|_|"", Describe::describe(Option::some(1), w), 42);;
            assert_eq(|_|"", Describe::describe(Option::some(1), [1, 2]), 43);;
            pure()
        );
    "#;
    for name in ["g", "h"] {
        test_source(&source.replace("{tv}", name), Configuration::develop_mode());
    }
}

/// The name a member's variable is renamed to, where it shares a name with a variable of the
/// implementation's head, stays apart from the variables of the implementation's context too.
#[test]
pub fn test_renamed_member_variable_apart_from_a_context_variable() {
    let source = r#"
        module Main;

        trait c : Bar {
            get : c -> a -> String;
        }

        trait c : Foo {
            type Elm c;
        }

        impl I64 : Foo {
            type Elm I64 = Bool;
        }

        type NoShow = unbox struct { v : I64 };

        impl [a : Foo, Foo::Elm a = {tv}, {tv} : ToString] Array a : Bar {
            get = |_, y| "ok";
        }

        main : IO ();
        main = (
            assert_eq(|_|"", Bar::get([1, 2], NoShow { v : 3 }), "ok");;
            pure()
        );
    "#;
    for name in ["b", "z"] {
        test_source(&source.replace("{tv}", name), Configuration::develop_mode());
    }
}

/// A kind signature a member's declaration writes for the trait's type variable leaves with that
/// variable when the implementation puts its type in the variable's place, apart from a variable
/// of the implementation's head of the same name and another kind.
#[test]
pub fn test_member_kind_signature_of_the_trait_variable_leaves_with_it() {
    let source = r#"
        module Main;

        trait [f : *->*] f : Desc {
            desc : [f : *->*] f I64 -> I64;
        }

        impl Arrow {tv} : Desc {
            desc = |_| 42;
        }

        main : IO ();
        main = (
            assert_eq(|_|"", Desc::desc(|x : I64| x + 1), 42);;
            pure()
        );
    "#;
    for name in ["f", "g"] {
        test_source(&source.replace("{tv}", name), Configuration::develop_mode());
    }
}

/// A variable the member's declaration names in a kind signature alone stays apart from a variable
/// of the implementation's head of the same name.
#[test]
pub fn test_member_variable_of_a_kind_signature_alone_apart_from_a_head_variable() {
    let source = r#"
        module Main;

        trait c : Bar {
            get : [b : *->*] c -> a -> String;
        }

        impl Array {tv} : Bar {
            get = |_, _| "ok";
        }

        main : IO ();
        main = (
            assert_eq(|_|"", Bar::get([1], 3), "ok");;
            pure()
        );
    "#;
    for name in ["a", "b", "x"] {
        test_source(&source.replace("{tv}", name), Configuration::develop_mode());
    }
}
