use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
};

#[test]
pub fn test_type_sign_in_trait_impl_0() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl Array a : MyToString {
        my_to_string : Array a -> String = |_|"my_to_string";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "my_to_string");;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_1() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string = |_|"my_to_string";
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "my_to_string");;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_use_in_annotation() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string = |arr : Array a| (
            arr.to_iter.map(to_string : a -> String).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_mismatch_type() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : b -> a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : a -> Array a -> String;
        my_to_string = |_: a, arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string(1), "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_redundant_kind_sign() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        my_member : (f, a);
    }

    type MyType = struct {};

    impl MyType : MyTrait {
        my_member : [f:*] (f, MyType);
        my_member = undefined("");
    }
    
    main : IO ();
    main = (
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_undefined_type_var_0() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [b : ToString] Array b : MyToString {
        my_to_string : Array b -> String;
        my_to_string = |arr : a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_undefined_type_var_1() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [b : ToString] Array b : MyToString {
        my_to_string = |arr : a| ( // `a` is defined in the trait, but not in this scope
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_duplicated_type_sig() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        my_to_string : Array a -> String;
        my_to_string : Array a -> String = |arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Duplicate the type signature of member `my_to_string`",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_type_sig_on_nonexistent_member() {
    let source = r#"
    module Main;

    trait a : MyToString {
        my_to_string : a -> String;
    }

    impl [a : ToString] Array a : MyToString {
        to_string : Array a -> String;

        my_to_string : Array a -> String = |arr : Array a| (
            arr.to_iter.map(to_string).join(", ")
        );
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", [1,2,3].my_to_string, "1, 2, 3");;
        pure()
    );
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`to_string` is not a member of trait `Main::MyToString`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_use_type_alias_and_trait_alias_0() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        member : [b : Zero, b : Add] b -> b -> () -> a;
    }

    impl I64 : MyTrait {
        member : [b : Additive] b -> b -> Lazy I64 = |x, y, _| eval x; eval y; 42;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", MyTrait::member(0, 0, ()), 42);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_use_type_alias_and_trait_alias_1() {
    let source = r#"
    module Main;

    trait a : MyTrait {
        member : [b : Additive] b -> b -> Lazy a;
    }

    impl I64 : MyTrait {
        member : [b : Zero, b : Add] b -> b -> () -> I64 = |x, y, _| eval x; eval y; 42;
    }
    
    main : IO ();
    main = (
        assert_eq(|_|"", MyTrait::member(0, 0, ()), 42);;
        pure()
    );
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_example_in_changelog_0() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap = |f : a -> b, x : MyType a| MyType{} : MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(u64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Unknown type variable `a`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_example_in_changelog_1() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap : (a -> b) -> MyType a -> MyType b = |f : a -> b, x : MyType a| MyType{} : MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(u64);
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_lacking_impl() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    mymap : (a -> b) -> MyType a -> MyType b;
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(u64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Lacking implementation of member `mymap`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_wrong_member_name() {
    let source = r#"
module Main;

trait [f : *->*] f : MyFunctor {
    mymap : (a -> b) -> f a -> f b;
}

type MyType a = struct {};

impl MyType : MyFunctor {
    my_map : (a -> b) -> MyType a -> MyType b;
    mymap = |f, x| MyType{};
}

main : IO () = (
    eval (MyType{} : MyType I64).mymap(u64);
    pure()
);
    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "`my_map` is not a member of trait `Main::MyFunctor`.",
    );
}

#[test]
pub fn test_type_sign_in_trait_impl_iterator_unuse_item() {
    // Verify that in the type signature of `advance`, we can write `()` instead of `Item MyIterator`.
    let source = r#"
module Main; 

type MyIterator = unbox struct {};

impl MyIterator : Iterator {
    type Item MyIterator = ();
    advance : MyIterator -> Option (MyIterator, ()) = |_| none();
}

main : IO ();
main = (
    assert(|_|"", MyIterator{}.advance.is_none);;
    pure()
);
    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_iterator_use_item() {
    // Verify that in the type signature of `advance`, we can write `Item MyIterator` instead of `()`.
    let source = r#"
module Main; 

type MyIterator = unbox struct {};

impl MyIterator : Iterator {
    type Item MyIterator = ();
    advance : MyIterator -> Option (MyIterator, Item MyIterator) = |_| none();
}

main : IO ();
main = (
    assert(|_|"", MyIterator{}.advance.is_none);;
    pure()
);

    "#;
    test_source(source, Configuration::develop_mode());
}

#[test]
pub fn test_type_sign_in_trait_impl_iterator_wrong_item() {
    let source = r#"
module Main; 

type MyIterator = unbox struct {};

impl MyIterator : Iterator {
    type Item MyIterator = ();
    advance : MyIterator -> Option (MyIterator, I64) = |_| none();
}

main : IO ();
main = (
    assert(|_|"", MyIterator{}.advance.is_none);;
    pure()
);

    "#;
    test_source_fail(
        source,
        Configuration::develop_mode(),
        "Type signature in implementation does not match trait definition.",
    );
}
