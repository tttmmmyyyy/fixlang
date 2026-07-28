// An array whose storage reaches `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes starts its element buffer on
// a 32-byte boundary. A 256-bit load or store that straddles a 64-byte cache line costs about 1.75x
// one that does not, and a vectorized array loop issues one per iteration, so a buffer off the
// boundary halves the throughput of exactly the loops that bounds- and uniqueness-check elimination
// made vectorizable. A smaller array takes `malloc` and whatever alignment it gives, since the bytes
// the alignment costs come out of every array while only the long loops win them back.
//
// The property is static, so these tests read the address rather than the clock: an instruction
// count sees neither the alignment nor its loss.

#[cfg(test)]
mod tests {
    use crate::constants::{ARRAY_ALIGNED_ALLOC_THRESHOLD, ARRAY_BUF_ALIGNMENT};
    use crate::tests::test_util::test_source_with_c;

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

    /// An array that grows past the threshold lands on the boundary too: over it, `reserve` gives a
    /// unique array a fresh block and moves the elements over rather than resizing in place, because
    /// a block from `realloc` starts where the allocator put it.
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
