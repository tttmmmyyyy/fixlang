use crate::{configuration::Configuration, tests::test_util::test_source_fail};

#[test]
pub fn test_impl_for_type_contains_associated_type() {
    let source = r##"
module Main;

trait a : MyTraitA {
    type MyTypeA a;
}

trait a : MyTraitB {
}

impl [a : MyTraitA] Array (MyTypeA a) : MyTraitB {
}

main: IO () = (
    pure()
);

    "##;
    test_source_fail(&source, Configuration::develop_mode(), "Implementing trait for type `Std::Array (Main::MyTraitA::MyTypeA a)` is not allowed. Associated types cannot appear in the type.");
}

#[test]
pub fn test_impl_for_associated_type() {
    let source = r##"
module Main;

trait a : MyTraitA {
    type MyTypeA a;
}

trait a : MyTraitB {
}

impl [a : MyTraitA] MyTypeA a : MyTraitB {
}

main: IO () = (
    pure()
);

    "##;
    test_source_fail(&source, Configuration::develop_mode(), "Implementing trait for type `Main::MyTraitA::MyTypeA a` is not allowed. The head (in this case, `Main::MyTraitA::MyTypeA`) of the type should be a type constructor.");
}

#[test]
pub fn test_impl_for_non_specific_type() {
    let source = r##"
module Main;

trait a : MyTrait {
}

impl [f : * -> *] f I64 : MyTrait {
}

main: IO () = (
    pure()
);
    "##;
    test_source_fail(&source, Configuration::develop_mode(), 
    "Implementing trait for type `f Std::I64` is not allowed. The head (in this case, `f`) of the type should be a type constructor.");
}
