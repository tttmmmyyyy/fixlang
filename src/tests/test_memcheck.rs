use crate::{
    configuration::{Configuration, ValgrindTool},
    tests::test_util::test_source_fail,
};

#[test]
pub fn test_use_undefined_value() {
    // Test using undefined value.
    let source = r#"
        module Main;

        type MyOpt = union {
            my_none : (),
            my_some : U64,
        };
        
        main : IO ();
        main = (
            my_none().as_my_some.to_string.println
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = true;
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "uninitialised value");
}

#[test]
pub fn test_memory_leak() {
    // Test memory leak detection.
    let source = r#"
        module Main;

        main : IO ();
        main = (
            Array::fill(128, 42).boxed_to_retained_ptr;;
            pure()
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "definitely lost");
}
