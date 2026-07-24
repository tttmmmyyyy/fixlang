// Memory-safety tests for `Array::swap` and `unsafe_swap_bounds_unchecked` on a shared array whose
// elements are boxed. A shared array (reference count >= 2) is cloned before the swap, and the clone
// retains every boxed element; the swapped result and the surviving alias must both stay valid and
// release each element exactly once — checked under valgrind. `test_array_swap` swaps boxed elements
// only in an unshared array (an in-place swap, no clone) and shares only an array of unboxed elements,
// so the shared-clone-with-boxed-elements retain path is exercised only here.

#[cfg(test)]
mod shared_boxed_swap_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const SHARED_BOXED_SWAP_SOURCE: &str = r#"
module Main;

main : IO () = (
    // shared array of boxed elements: `swap` clones, retaining the boxed elements; the alias stays
    // intact and the swapped clone reflects the swap
    let arr = [[1, 2], [3, 4], [5, 6]];
    let alias = arr;
    let swapped = arr.swap(0, 2);
    assert_eq(|_|"swapped[0]", swapped.@(0).@(0), 5);;
    assert_eq(|_|"swapped[2]", swapped.@(2).@(1), 2);;
    assert_eq(|_|"alias intact[0]", alias.@(0).@(0), 1);;
    assert_eq(|_|"alias intact[2]", alias.@(2).@(1), 6);;

    // same, through `unsafe_swap_bounds_unchecked`
    let arr2 = [[7], [8], [9]];
    let alias2 = arr2;
    let swapped2 = arr2.unsafe_swap_bounds_unchecked(0, 2);
    assert_eq(|_|"uswapped2[0]", swapped2.@(0).@(0), 9);;
    assert_eq(|_|"alias2 intact", alias2.@(0).@(0), 7);;

    pure()
);
"#;

    #[test]
    pub fn test_shared_boxed_swap_correctness() {
        test_source(SHARED_BOXED_SWAP_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_shared_boxed_swap_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(SHARED_BOXED_SWAP_SOURCE, config);
    }
}
