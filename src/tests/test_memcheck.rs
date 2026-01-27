use crate::{
    configuration::{Configuration, ValgrindTool},
    misc::{function_name, platform_valgrind_supported},
    tests::test_util::test_source_fail,
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
        
        main : IO ();
        main = (
            let arr = Array::empty(2) : Array U64;
            arr.@(1).to_string.println
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
