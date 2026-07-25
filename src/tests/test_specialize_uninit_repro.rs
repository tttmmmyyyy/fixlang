use crate::{configuration::Configuration, tests::test_util::test_source};

// Constructing a union whose active variant is smaller than the union's payload buffer must leave no
// uninitialised bytes in the buffer. A nested `flat_map` folded into a growing array builds such
// unions and passes them across a function boundary as scalar leaves; at `-O max` / `experimental`
// the consumer (`Iterator::advance`) speculatively reads the payload before checking its tag, so an
// undef tail byte surfaces as a valgrind "Conditional jump or move depends on uninitialised
// value(s)". The computed result is correct either way, so this guards against a memory-safety
// (undefined-behavior) regression rather than a miscompilation. Runs under `develop_mode`'s memcheck.
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
