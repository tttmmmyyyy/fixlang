// An array whose storage reaches `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes starts its element buffer on
// a 32-byte boundary. A 256-bit load or store that straddles a 64-byte cache line costs about 1.75x
// one that does not, and a vectorized array loop issues one per iteration, so a buffer off the
// boundary halves the throughput of exactly the loops that bounds- and uniqueness-check elimination
// made vectorizable. A smaller array takes `malloc` and whatever alignment it gives, since the bytes
// the alignment costs come out of every array while only the long loops win them back.
//
// The property is static, so these tests assert on the address of the element buffer: an
// instruction count sees neither the alignment nor its loss.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::constants::{ARRAY_ALIGNED_ALLOC_THRESHOLD, ARRAY_BUF_ALIGNMENT};
    use crate::tests::test_util::{test_source, test_source_with_c};

    /// Reports the address of an array's first element modulo the alignment, which the Fix side
    /// asserts is zero.
    const ADDR_MOD_ALIGNMENT: &str = r#"
        #include <stdint.h>
        int64_t fixtest_addr_mod_alignment(void* p, int64_t alignment) {
            return (int64_t)((uintptr_t)p % (uintptr_t)alignment);
        }
    "#;

    /// The Fix side of every test: the assertion, and the element count that clears the threshold.
    fn preamble() -> String {
        format!(
            r#"
        module Main;

        assert_aligned : String -> Array a -> IO ();
        assert_aligned = |label, arr| (
            let m = arr.borrow_elements(|p|
                FFI_CALL[I64 fixtest_addr_mod_alignment(Ptr, I64), p, {alignment}]
            );
            assert_eq(|_|label + ": the element buffer is off the {alignment}-byte boundary", m, 0)
        );

        // An element count that puts an I64 storage over the threshold, so its elements are aligned.
        i64_count : I64;
        i64_count = {i64_count};
"#,
            alignment = ARRAY_BUF_ALIGNMENT,
            i64_count = ARRAY_ALIGNED_ALLOC_THRESHOLD / 8 + 1,
        )
    }

    /// Every way of building an array over the threshold lands its elements on the boundary,
    /// whatever the element type.
    #[test]
    fn test_large_array_element_buffer_is_aligned() {
        let source = preamble()
            + &format!(
                r#"
        // An element count that puts a U8 storage over the threshold.
        u8_count : I64;
        u8_count = {u8_count};
"#,
                u8_count = ARRAY_ALIGNED_ALLOC_THRESHOLD + 1,
            )
            + r#"
        main : IO ();
        main = (
            assert_aligned("I64 fill", Array::fill(i64_count, 7));;
            assert_aligned("U8 fill", Array::fill(u8_count, 1_U8));;
            assert_aligned("F64 from_map", Array::from_map(i64_count, |i| i.to_F64));;
            assert_aligned("boxed elements", Array::fill(i64_count, Box::make(1)));;
            assert_aligned("string bytes", Iterator::range(0, u8_count).map(|_| 'a').to_array.push_back('\0').from_bytes.as_ok.get_bytes);;
            pure()
        );
        "#;
        test_source_with_c(&source, ADDR_MOD_ALIGNMENT, "array_alignment_build");
    }

    /// Whatever the element type, a storage that clears the threshold lands its elements on the
    /// boundary. `ARRAY_ALIGNED_ALLOC_THRESHOLD` elements clear it for every element type that
    /// occupies at least a byte.
    #[test]
    fn test_element_buffer_is_aligned_for_every_element_type() {
        let source = preamble()
            + &format!(
                r#"
        type BoxedUnion = box union {{ num : I64, text : String }};
        type UnboxUnion = union {{ num : I64, pair : (I64, I64) }};
        type UnboxStruct = unbox struct {{ p : I64, q : I64, r : U8 }};

        // An element count that puts a storage of any element type of at least one byte over the
        // threshold.
        count : I64;
        count = {threshold};

        main : IO ();
        main = (
            assert_aligned("F32", Array::from_map(count, |i| i.to_F32));;
            assert_aligned("Bool", Array::from_map(count, |i| i % 2 == 0));;
            assert_aligned("Ptr", Array::from_map(count, |_| nullptr));;
            assert_aligned("tuple", Array::from_map(count, |i| (i, i.to_U8)));;
            assert_aligned("unbox union", Array::from_map(count, |i|
                if i % 2 == 0 {{ UnboxUnion::num(i) }} else {{ UnboxUnion::pair((i, i)) }}));;
            assert_aligned("boxed union", Array::from_map(count, |i|
                if i % 2 == 0 {{ BoxedUnion::num(i) }} else {{ BoxedUnion::text("x") }}));;
            assert_aligned("unbox struct", Array::from_map(count, |i|
                UnboxStruct {{ p : i, q : i, r : i.to_U8 }}));;
            assert_aligned("String", Array::from_map(count, |i| i.to_string));;
            assert_aligned("nested array", Array::from_map(count, |i| Array::fill(i % 3, i)));;
            assert_aligned("closure", Array::from_map(count, |i| |x| x + i));;
            pure()
        );
"#,
                threshold = ARRAY_ALIGNED_ALLOC_THRESHOLD,
            );
        test_source_with_c(&source, ADDR_MOD_ALIGNMENT, "array_alignment_element_types");
    }

    /// Every capacity around the threshold, taken in both directions, keeps the array's elements.
    /// Each crossing moves the storage between a block that starts at its allocation's base and one
    /// that does not, and boxed elements make a lost or double-released element visible.
    #[test]
    fn test_every_capacity_across_the_threshold_keeps_the_elements() {
        let source = r#"
module Main;

set_cap : I64 -> Array a -> Array a;
set_cap = |c, a| a._unsafe_set_capacity_bounds_unchecked(c);

sweep : I64 -> I64;
sweep = |len| (
    let base = Array::from_map(len, |i| Box::make(i * 7 + 1));
    let expect = base.to_iter.map(|b| b.@value).sum;
    Iterator::range(0, 49).fold(0, |c, acc|
        let a = base.set_cap(max(c, len));
        let a = a.set_cap(max(2 * c, len));
        let a = a.set_cap(max(len, 1));
        acc + (if a.to_iter.map(|b| b.@value).sum == expect { 0 } else { 1 })
    )
);

main : IO ();
main = (
    let bad = Iterator::range(0, 40).fold(0, |len, acc| acc + sweep(len));
    assert_eq(|_|"an array resized across the threshold lost elements", bad, 0);;
    pure()
);
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An array that grows past the threshold lands on the boundary too, whether it grew by
    /// `push_back`, by `reserve` on a unique array, or by the clone `reserve` makes of a shared one,
    /// which leaves the array it was cloned from aligned as well.
    #[test]
    fn test_array_grown_past_the_threshold_is_aligned() {
        let source = preamble()
            + r#"
        grown : I64 -> Array I64;
        grown = |n| Iterator::range(0, n).fold([], |x, acc| acc.push_back(x));

        main : IO ();
        main = (
            assert_aligned("push_back past the threshold", grown(i64_count));;
            assert_aligned("reserve past the threshold", ([] : Array I64).reserve(3).push_back(1).reserve(i64_count));;
            let shared = grown(i64_count);
            assert_aligned("reserve of a shared array", shared.reserve(4 * i64_count));;
            assert_aligned("the array it was cloned from", shared);;
            pure()
        );
        "#;
        test_source_with_c(&source, ADDR_MOD_ALIGNMENT, "array_alignment_grow");
    }

    /// An array driven back and forth across the threshold keeps its elements: each crossing moves
    /// the storage between a block that starts at its allocation's base and one that does not, and
    /// the elements have to survive the move and the free that follows it.
    #[test]
    fn test_array_resized_across_the_threshold_keeps_its_elements() {
        let source = preamble()
            + r#"
        // `reserve` never shrinks, so shrink through the capacity primitive.
        set_capacity : I64 -> Array a -> Array a;
        set_capacity = |cap, arr| arr._unsafe_set_capacity_bounds_unchecked(cap);

        main : IO ();
        main = (
            let arr = Array::from_map(4, |i| Box::make(i));
            let arr = arr.reserve(i64_count);
            assert_aligned("grown over the threshold", arr);;
            let arr = arr.set_capacity(4);
            let arr = arr.reserve(2 * i64_count);
            assert_aligned("grown over the threshold again", arr);;
            let arr = arr.set_capacity(4);
            assert_eq(|_|"the elements did not survive the moves", arr.map(|b| b.@value), [0, 1, 2, 3]);;
            pure()
        );
        "#;
        test_source_with_c(&source, ADDR_MOD_ALIGNMENT, "array_alignment_resize");
    }
}
