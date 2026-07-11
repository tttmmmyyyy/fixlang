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
