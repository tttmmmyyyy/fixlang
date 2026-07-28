use crate::{
    configuration::{Configuration, ValgrindTool},
    misc::{function_name, platform_valgrind_supported},
    tests::test_util::test_source_fail,
};

#[test]
pub fn test_use_undefined_value() {
    // A read of memory that was never written reaches the test as a failure. This is what gives
    // weight to the tests that assert memory safety by running a program under memcheck and
    // expecting it to succeed.
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
    // Memory that is allocated and never freed reaches the test as a failure, so a program under
    // memcheck that leaks is caught even though it exits successfully.
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
            FFI_CALL_IO[Ptr malloc(CSizeT), 128.c_size_t];;
            pure()
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "definitely lost");
}

#[test]
pub fn test_use_after_free() {
    // A read of freed memory reaches the test as a failure. This is what gives weight to the tests
    // that assert memory safety by running a program under memcheck and expecting it to succeed.
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
            let ptr = *FFI_CALL_IO[Ptr malloc(CSizeT), 128.c_size_t];
            FFI_CALL_IO[Ptr memset(Ptr, CInt, CSizeT), ptr, 0.c_int, 128.c_size_t];;
            FFI_CALL_IO[() free(Ptr), ptr];;
            let found = *FFI_CALL_IO[Ptr memchr(Ptr, CInt, CSizeT), ptr, 0.c_int, 8.c_size_t];
            println((found != nullptr).to_string)
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "Invalid read");
}

#[test]
pub fn test_double_free() {
    // Freeing the same pointer twice reaches the test as a failure, so a program under memcheck
    // that frees twice is caught even though it runs to completion.
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
            let ptr = *FFI_CALL_IO[Ptr malloc(CSizeT), 128.c_size_t];
            FFI_CALL_IO[() free(Ptr), ptr];;
            FFI_CALL_IO[() free(Ptr), ptr];;
            pure()
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "Invalid free");
}

#[test]
pub fn test_leaked_array_is_an_error() {
    // An array large enough to have its elements aligned sits above the base of its allocation, so a
    // leaked one is reached only by a pointer into its block and the leak checker calls it
    // indirectly lost. The memcheck invocation takes that kind as an error, so a reference counting
    // mistake that strands an array reaches the test as a failure.
    if !platform_valgrind_supported() {
        eprintln!(
            "Skipping {}: Valgrind not available on this platform.",
            function_name!()
        );
        return;
    }
    let source = r#"
        module Main;

        type Big = box struct { xs : Array I64 };

        main : IO ();
        main = (
            let big = Big { xs : Array::from_map(1000, |i| i) };
            let ptr = *big.boxed_to_retained_ptr;
            println $ (ptr == nullptr).to_string
        );
    "#;
    let mut config = Configuration::develop_mode();
    config.set_valgrind(ValgrindTool::MemCheck);
    test_source_fail(&source, config, "indirectly lost");
}
