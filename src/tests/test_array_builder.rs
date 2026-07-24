// Tests for the array builder primitives `append`, `reserve`, `resize`, and `push_back`. Each
// clones the array if it is shared, so building on a shared array must leave the original intact.
// The memory-safety test checks the boxed-element paths under valgrind: `append` moves the elements
// out of a unique source (with no reference counting) and copies them out of a shared one,
// `reserve` reallocs a unique array's block and copies a shared one, and each must neither leak an
// element nor free one twice.

#[cfg(test)]
mod array_builder_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    #[test]
    pub fn test_builder_correctness() {
        let source = r#"
module Main;

main : IO () = (
    // `append` on unboxed / boxed arrays.
    assert_eq(|_|"append unboxed", [1, 2].append([3, 4]), [1, 2, 3, 4]);;
    assert_eq(|_|"append boxed", [[1], [2]].append([[3]]), [[1], [2], [3]]);;
    assert_eq(|_|"append empty src", [1, 2].append([]), [1, 2]);;
    assert_eq(|_|"append empty dst", ([] : Array I64).append([3, 4]), [3, 4]);;

    // `append` of a shared array leaves both arguments intact.
    let a = [1, 2];
    let b = [3, 4];
    let c = a.append(b);
    assert_eq(|_|"append shared src", b, [3, 4]);;
    assert_eq(|_|"append shared dst", a, [1, 2]);;
    assert_eq(|_|"append shared result", c, [1, 2, 3, 4]);;

    // `reserve` grows the capacity while keeping the elements.
    let r = [1, 2, 3].reserve(16);
    assert_eq(|_|"reserve keeps elements", r, [1, 2, 3]);;
    assert_eq(|_|"reserve grows capacity", r.@capacity >= 16, true);;

    // `resize` grows with the fill value and truncates.
    assert_eq(|_|"resize grow", [1, 2].resize(4, 9), [1, 2, 9, 9]);;
    assert_eq(|_|"resize shrink", [1, 2, 3, 4].resize(2, 0), [1, 2]);;

    // `push_back` past the capacity reallocates.
    let p = Iterator::range(0, 100).fold(Array::empty(1), |i, arr| arr.push_back(i));
    assert_eq(|_|"push_back grow", p.@(99), 99);;
    assert_eq(|_|"push_back size", p.@size, 100);;

    // `unsafe_set_bounds_unchecked` writes an element in place, cloning a shared array.
    assert_eq(|_|"unsafe_set boxed", [[1], [2], [3]].unsafe_set_bounds_unchecked(1, [9]), [[1], [9], [3]]);;
    let sh = [[1], [2]];
    let s2 = sh.unsafe_set_bounds_unchecked(0, [9]);
    assert_eq(|_|"unsafe_set shared original", sh, [[1], [2]]);;
    assert_eq(|_|"unsafe_set shared result", s2, [[9], [2]]);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }

    #[test]
    pub fn test_builder_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let source = r#"
module Main;

main : IO () = (
    // `append` moves a unique boxed source's elements out with no reference counting; each must
    // end up owned by the result exactly once.
    eval [[1], [2]].append([[3], [4]]);

    // `append` copies a shared boxed source's elements with a retain each; the source and the result
    // must be released independently.
    let src = [[3], [4]];
    let dst = [[1], [2]];
    let both = dst.append(src);
    assert_eq(|_|"append shared src intact", src, [[3], [4]]);;
    assert_eq(|_|"append shared dst intact", dst, [[1], [2]]);;
    assert_eq(|_|"append shared result", both, [[1], [2], [3], [4]]);;

    // `reserve` reallocates a unique boxed array's block; the elements survive the move.
    eval [[1], [2], [3]].reserve(64);

    // `resize` grows a boxed array with a shared fill value and shrinks another, releasing the
    // dropped elements.
    eval [[1], [2]].resize(4, [9]);
    eval [[1], [2], [3], [4]].resize(2, [0]);

    // Growing a boxed array by repeated `push_back` reallocates several times.
    eval Iterator::range(0, 50).fold(Array::empty(1), |i, arr| arr.push_back([i]));

    // `sort_stable_by` merges runs into a working buffer, draining an exhausted run in one bulk
    // copy; on boxed elements the copies and the copy-back must not leak or double-free.
    assert_eq(|_|"sort_stable boxed",
        [[3], [1], [2], [1], [4], [0]].sort_stable_by(|(a, b)| a.@(0) < b.@(0)),
        [[0], [1], [1], [2], [3], [4]]);;

    // `get_sub` on a boxed array copies the range out; the source stays intact.
    let g = [[1], [2], [3], [4]];
    assert_eq(|_|"get_sub boxed", g.get_sub(1, 3), [[2], [3]]);;
    assert_eq(|_|"get_sub boxed src intact", g, [[1], [2], [3], [4]]);;

    // `unsafe_set_bounds_unchecked` on a boxed array releases the overwritten element and, on a
    // shared array, clones so the original keeps its element.
    eval [[1], [2], [3]].unsafe_set_bounds_unchecked(1, [9]);
    let base = [[1], [2], [3]];
    let overwritten = base.unsafe_set_bounds_unchecked(0, [9]);
    assert_eq(|_|"unsafe_set shared base intact", base, [[1], [2], [3]]);;
    assert_eq(|_|"unsafe_set shared overwritten", overwritten, [[9], [2], [3]]);;
    pure()
);
"#;
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(source, config);
    }
}
