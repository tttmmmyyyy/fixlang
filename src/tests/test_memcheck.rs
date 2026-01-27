use crate::{
    configuration::{Configuration, ValgrindTool},
    misc::{function_name, platform_valgrind_supported},
    tests::test_util::{test_source, test_source_fail},
};

#[test]
pub fn test_use_undefined_value() {
    // Test using undefined value.
    if !platform_valgrind_supported() {
        eprintln!(
            "Skipping {}: Valgrind not available on this platform.",
            function_name!()
        );
        return;
    }
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
    if !platform_valgrind_supported() {
        eprintln!(
            "Skipping {}: Valgrind not available on this platform.",
            function_name!()
        );
        return;
    }
    let source = r#"
        module Main;

        main : IO ();
        main = (
            println("");;
            FFI_CALL_IO[Ptr malloc(CInt), 128.c_int];;
            pure()
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "definitely lost");
}
