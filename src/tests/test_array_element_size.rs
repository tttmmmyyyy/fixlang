// The element buffer of an `Array` holds each element as the buffer stores it: a pointer where the
// element type is boxed. An array of a boxed type therefore takes one pointer per element, however
// large the element's own object is.
//
// The property is a memory size, so the tests measure the block the element buffer lives in and
// compare it with the same measurement for an array of `I64`. The arrays stay below
// `ARRAY_ALIGNED_ALLOC_THRESHOLD`, where a storage starts at the base of its allocation, so stepping
// back over the control block from the buffer reaches the pointer the allocator returned.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::constants::ARRAY_ALIGNED_ALLOC_THRESHOLD;
    use crate::tests::test_util::{test_source, test_source_with_c};

    /// Reports the size of the allocation an array's element buffer lives in, given the distance
    /// from the base of that allocation to the buffer.
    const BLOCK_SIZE: &str = r#"
        #include <stdint.h>
        #ifdef __APPLE__
        #include <malloc/malloc.h>
        #define fixtest_usable_size(p) malloc_size(p)
        #else
        #include <malloc.h>
        #define fixtest_usable_size(p) malloc_usable_size(p)
        #endif
        int64_t fixtest_block_size(void *p, int64_t back) {
            return (int64_t)fixtest_usable_size((char *)p - back);
        }
    "#;

    /// An array of a boxed element type reserves a reference per element rather than the whole of
    /// each element, so its buffer is the size of an array of `I64` of the same length.
    #[test]
    fn test_array_of_a_boxed_type_takes_one_pointer_per_element() {
        let source = format!(
            r#"
module Main;

// A boxed element whose own object is far larger than the pointer the buffer stores.
type Big = box struct {{ a : I64, b : I64, c : I64, d : I64, e : I64, f : I64, g : I64, h : I64 }};

// The distance from the base of a storage's allocation to its element buffer: the control block,
// which the elements follow immediately.
header : I64 = 8;

block_size : Array a -> I64;
block_size = |arr| arr.borrow_elements(|p| FFI_CALL[I64 fixtest_block_size(Ptr, I64), p, header]);

// An element count whose storage stays below the size from which a storage is placed off the base
// of its allocation, so that stepping back over the control block reaches that base.
count : I64 = {count};

main : IO ();
main = (
    let boxed = Array::from_map(count, |i| Big {{ a : i, b : i, c : i, d : i, e : i, f : i, g : i, h : i }});
    let unboxed = Array::from_map(count, |i| i);
    assert(|_|"the block holding the elements was not measured", block_size(unboxed) >= count * 8);;
    assert_eq(
        |_|"an array of a boxed element type does not take one pointer per element",
        block_size(boxed),
        block_size(unboxed)
    );;
    pure()
);
"#,
            count = ARRAY_ALIGNED_ALLOC_THRESHOLD / 8 - 4,
        );
        test_source_with_c(&source, BLOCK_SIZE, "array_element_size");
    }

    /// Every array primitive over a boxed element type, whose buffer holds one pointer per element
    /// and so is exactly as long as its elements need: an access past them leaves the block, which
    /// the memcheck run of the test suite reports.
    #[test]
    fn test_array_primitives_over_a_boxed_element_type() {
        let source = r#"
module Main;

type Big = box struct { a : I64, b : I64, c : I64, d : I64, e : I64, f : I64, g : I64, h : I64 };

mk : I64 -> Big;
mk = |i| Big { a : i, b : i+1, c : i+2, d : i+3, e : i+4, f : i+5, g : i+6, h : i+7 };

// The elements of an array of `Big`, as the numbers they were made from.
ids : Array Big -> Array I64;
ids = |arr| arr.map(|x| x.@a);

main : IO ();
main = (
    assert_eq(|_|"fill", ids(Array::fill(3, mk(7))), [7, 7, 7]);;
    assert_eq(|_|"from_map", ids(Array::from_map(4, mk)), [0, 1, 2, 3]);;
    assert_eq(|_|"literal", ids([mk(1), mk(2)]), [1, 2]);;
    assert_eq(|_|"push_back", ids(Iterator::range(0, 5).fold([], |i, acc| acc.push_back(mk(i)))), [0, 1, 2, 3, 4]);;
    assert_eq(|_|"set", ids(Array::from_map(4, mk).set(1, mk(9))), [0, 9, 2, 3]);;
    assert_eq(|_|"mod", ids(Array::from_map(4, mk).mod(2, |x| mk(x.@a * 10))), [0, 1, 20, 3]);;
    assert_eq(|_|"swap", ids(Array::from_map(4, mk).swap(0, 3)), [3, 1, 2, 0]);;
    assert_eq(|_|"truncate", ids(Array::from_map(4, mk).truncate(2)), [0, 1]);;
    assert_eq(|_|"pop_back", ids(Array::from_map(4, mk).pop_back), [0, 1, 2]);;
    assert_eq(|_|"get_sub", ids(Array::from_map(6, mk).get_sub(2, 5)), [2, 3, 4]);;
    assert_eq(|_|"append", ids(Array::from_map(2, mk).append(Array::from_map(3, mk))), [0, 1, 0, 1, 2]);;
    assert_eq(|_|"reserve", ids(Array::from_map(3, mk).reserve(64).push_back(mk(3))), [0, 1, 2, 3]);;
    assert_eq(|_|"resize", ids(Array::from_map(2, mk).resize(4, mk(8))), [0, 1, 8, 8]);;
    assert_eq(|_|"sort", ids(Array::from_map(5, |i| mk((i * 2) % 5)).sort_by(|(x, y)| x.@a < y.@a)), [0, 1, 2, 3, 4]);;
    // Long enough that the stable sort merges the range rather than sorting it by insertion, so
    // that the merge's element-by-element writes land on this element size too.
    assert_eq(|_|"sort_stable", ids(Array::from_map(20, |i| mk((i * 3) % 20)).sort_stable_by(|(x, y)| x.@a < y.@a)), Array::from_map(20, |i| i));;
    assert_eq(|_|"reverse", ids(Array::from_map(4, mk).reverse), [3, 2, 1, 0]);;
    assert_eq(|_|"to_iter", ids(Array::from_map(4, mk).to_iter.to_array), [0, 1, 2, 3]);;

    // The same operations on a shared array, which take the clone-if-shared path.
    let shared = Array::from_map(4, mk);
    assert_eq(|_|"set on a shared array", ids(shared.set(0, mk(9))), [9, 1, 2, 3]);;
    assert_eq(|_|"mod on a shared array", ids(shared.mod(0, |x| mk(x.@h))), [7, 1, 2, 3]);;
    assert_eq(|_|"append to a shared array", ids(shared.append(Array::from_map(1, mk))), [0, 1, 2, 3, 0]);;
    assert_eq(|_|"reserve on a shared array", ids(shared.reserve(64)), [0, 1, 2, 3]);;
    assert_eq(|_|"the array they were cloned from", ids(shared), [0, 1, 2, 3]);;

    // Degenerate shapes.
    assert_eq(|_|"empty", ids([]), []);;
    assert_eq(|_|"one element", ids(Array::fill(1, mk(5))), [5]);;
    assert_eq(|_|"capacity without elements", ids(Array::empty(8)), []);;
    pure()
);
"#;
        test_source(source, Configuration::develop_mode());
    }
}
