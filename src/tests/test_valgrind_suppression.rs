use crate::{configuration::Configuration, tests::test_util::test_source};

// The `fix-iterator-advance-uninitialised-union-padding` entry in the repo-root `valgrind.supp`
// silences a benign uninitialised-value read. At `-O max` / `-O experimental` an iterator whose
// state holds a union (here a nested `flat_map`) is passed to `Iterator::advance` by value; the
// unused tail of a small variant's payload buffer is uninitialised, and LLVM speculatively reads it
// above the tag check. The `freeze` LLVM inserts makes the branch well-defined and the result
// correct, so the read is benign — valgrind reports it only because `freeze` lowers to reading that
// uninitialised stack slot. This runs the program under `develop_mode`'s memcheck, which loads that
// `valgrind.supp` (the test process runs from the repo root), and asserts it stays clean — so a
// change to `advance`'s mangled name that stops the suppression matching is caught here.
#[test]
pub fn test_iterator_advance_union_padding_suppressed() {
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
