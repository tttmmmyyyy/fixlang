use crate::{
    configuration::Configuration,
    tests::test_util::{test_source, test_source_fail},
};

#[test]
pub fn test_get() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].@(3);
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Index out of range: index=3, size=3");
}

#[test]
pub fn test_set() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].set(3, 42);
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Index out of range");
}

#[test]
pub fn test_mod() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].mod(3, |_| 42);
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Index out of range");
}

#[test]
pub fn test_act() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2].act(3, |_| some(42));
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Index out of range");
}

#[test]
pub fn test_index_syntax() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval [0,1,2][3].iget;
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Index out of range");
}

#[test]
pub fn test_empty_negative_capacity() {
    let source = r#"    
            module Main;
            
            main : IO ();
            main = (
                eval Array::empty(-1) : Array I64; 
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Negative array size or capacity: -1");
}

#[test]
pub fn test_fill_negative_size() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                eval Array::fill(-1, 42);
                pure()
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(&source, config, "Negative array size or capacity");
}

// The bytes an array's elements need must fit in the address space. A capacity whose byte count
// exceeds it wraps around to a small number, which `malloc` supplies, leaving an array whose block
// has no room for the capacity it claims; the first write past the block corrupts the heap. The two
// capacities below are caught by different halves of the check.

// A capacity of 2^61 elements of 8 bytes comes to exactly 2^64 bytes, which wraps to zero.
#[test]
pub fn test_empty_capacity_byte_count_wraps() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693952);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(
        &source,
        config,
        "Array size or capacity too large: 2305843009213693952",
    );
}

// A capacity of 2^61-3 elements of 8 bytes comes to 24 bytes short of 2^64: the elements fit, while
// the header and the padding that puts the element buffer on its alignment do not.
#[test]
pub fn test_empty_capacity_byte_count_leaves_no_room_for_the_header() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(2305843009213693949);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(
        &source,
        config,
        "Array size or capacity too large: 2305843009213693949",
    );
}

// A capacity the compiler cannot see the value of is checked where the program allocates, rather
// than by folding the check away at the capacity a literal gives.
#[test]
pub fn test_capacity_byte_count_is_checked_at_run_time() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let args = *get_args;
                // A program run with no argument gets one element in `args`, so this is 2^61-3.
                let arr : Array I64 = Array::empty(2305843009213693950 - args.@size);
                println(arr.push_back(42).@(0).to_string)
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(
        &source,
        config,
        "Array size or capacity too large: 2305843009213693949",
    );
}

// Growing an array's capacity reallocates its storage, and the byte count of the new capacity is
// checked as the one an array is created with is.
#[test]
pub fn test_reserve_capacity_byte_count_wraps() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = [1, 2, 3];
                println(arr.reserve(2305843009213693952).@(0).to_string)
            );
        "#;
    let mut config = Configuration::develop_mode();
    config.no_runtime_check = false;
    test_source_fail(
        &source,
        config,
        "Array size or capacity too large: 2305843009213693952",
    );
}

// `--no-runtime-check` disables array bounds checks (documented in the CLI help), so `set`
// and `swap` must honor it like `@` / `mod` / `act`. An index within the array's capacity but
// past its size stays inside the allocated buffer, so the access itself is memory-safe; only
// the bounds check decides whether it aborts. That makes "the check was removed" observable
// as a completed run rather than an out-of-range abort.

#[test]
pub fn test_set_bounds_check_respects_no_runtime_check() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.set(5, 42);                    // in capacity, past size
                pure()
            );
        "#;
    // Checks on: the out-of-range index aborts.
    let mut checked = Configuration::develop_mode();
    checked.no_runtime_check = false;
    test_source_fail(&source, checked, "Index out of range");
    // Checks off: no bounds check, so the (memory-safe) access completes.
    let mut unchecked = Configuration::develop_mode();
    unchecked.no_runtime_check = true;
    test_source(&source, unchecked);
}

#[test]
pub fn test_swap_bounds_check_respects_no_runtime_check() {
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.swap(3, 5);                    // both in capacity, past size
                pure()
            );
        "#;
    let mut checked = Configuration::develop_mode();
    checked.no_runtime_check = false;
    test_source_fail(&source, checked, "Index out of range");
    let mut unchecked = Configuration::develop_mode();
    unchecked.no_runtime_check = true;
    test_source(&source, unchecked);
}

#[test]
pub fn test_unsafe_swap_bounds_unchecked_skips_check() {
    // `unsafe_swap_bounds_unchecked` never bounds-checks, even with runtime checks on.
    let source = r#"
            module Main;

            main : IO ();
            main = (
                let arr : Array I64 = Array::empty(10); // size 0, capacity 10
                eval arr.unsafe_swap_bounds_unchecked(3, 5);   // in capacity, past size
                pure()
            );
        "#;
    let mut checked = Configuration::develop_mode();
    checked.no_runtime_check = false;
    test_source(&source, checked);
}
