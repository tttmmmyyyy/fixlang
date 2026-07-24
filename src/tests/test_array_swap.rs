// Tests for the `Array::swap` / `Array::unsafe_swap_bounds_unchecked` builtins.
//
// Both clone a shared array before mutating (checked here with the shared-value pattern) and
// swap in place otherwise. `swap` bounds-checks the indices; `unsafe_swap_bounds_unchecked` omits
// that check. Sort uses the unchecked variant, so its result is exercised too.

#[cfg(test)]
mod array_swap_tests {
    use crate::{configuration::Configuration, tests::test_util::test_source};

    #[test]
    pub fn test_swap() {
        let source = r#"
module Main;

main : IO ();
main = (
    // Unboxed elements, both variants.
    assert_eq(|_|"swap 0,3", [1,2,3,4].swap(0,3), [4,2,3,1]);;
    assert_eq(|_|"swap_bu 0,3", [1,2,3,4].unsafe_swap_bounds_unchecked(0,3), [4,2,3,1]);;
    assert_eq(|_|"swap i==j", [1,2,3].swap(1,1), [1,2,3]);;

    // Boxed elements only change places; reference counts stay balanced.
    assert_eq(|_|"swap boxed", [[1],[2],[3]].swap(0,2), [[3],[2],[1]]);;

    // Shared array: swap must clone, leaving the aliased copy intact.
    let a = [1,2,3,4];
    let keep = (a, a);
    let a2 = a.swap(0,3);
    assert_eq(|_|"shared keep", keep.@0, [1,2,3,4]);;
    assert_eq(|_|"shared result", a2, [4,2,3,1]);;

    // Sort (uses unsafe_swap_bounds_unchecked internally).
    assert_eq(|_|"sort", [3,1,4,1,5,9,2,6].sort, [1,1,2,3,4,5,6,9]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
