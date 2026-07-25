use crate::{configuration::Configuration, tests::test_util::test_source};

// A fold over a nested `flat_map` iterator into a growing array reads an uninitialised stack value
// at `-O max` / `experimental`. The `specialize` pass clones the fold on input uniqueness and the
// specialized clone leaves the value undef in a `MapIterator::advance`, so valgrind reports a
// "Conditional jump or move depends on uninitialised value(s)". The computed result is correct, so
// this is a memory-safety (undefined-behavior) defect rather than a miscompilation. Runs under
// `develop_mode`'s memcheck; it fails until the `specialize` pass no longer emits the undef.
#[test]
pub fn test_specialize_nested_flat_map_uninitialised_value() {
    let source = r#"
        module Main;

        main : IO ();
        main = (
            let table : Array (Array I64) = Iterator::range(0, 8).map(|i|
                Iterator::range(0, i).to_array
            ).to_array;
            let flattened = Iterator::range(0, table.@size).flat_map(|ti|
                Iterator::range(0, table.@(ti).@size).flat_map(|bi|
                    Iterator::range(0, table.@(ti).@(bi) + 1)
                )
            ).fold(Array::empty(0), |x, acc| acc.push_back(x));
            assert_eq(|_| "flattened size", flattened.@size, 84);;
            pure()
        );
    "#;
    test_source(&source, Configuration::develop_mode());
}
