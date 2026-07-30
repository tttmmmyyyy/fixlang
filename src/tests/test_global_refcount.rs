// A value reachable from a global is reference-counted like any other value: the reference a caller
// hands to a consuming callee is one it retained, so reading a global leaves its count where it was.
// Its count is also permanent, which is what keeps a global from being freed or mutated in place —
// but a permanent count is exactly what hides an unbalanced read, since millions of unmatched
// releases still leave the object alive. These tests therefore assert on the count itself.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::test_source_with_c;

    /// Reports the reference count of a boxed value, which its pointer points at.
    const READ_REFCNT: &str = r#"
        #include <stdint.h>
        int64_t fixtest_refcnt(void* p) {
            return (int64_t)*(int32_t*)p;
        }
    "#;

    /// The Fix side of every test: a boxed global, and its reference count read through FFI.
    const PREAMBLE: &str = r#"
        module Main;

        type Big = box struct { v : I64 };

        table : Big;
        table = Big { v : 42 };

        // The reference count of `table`. `boxed_to_retained_ptr` adds a reference of its own, which
        // is constant across the calls the tests compare, and `boxed_from_retained_ptr` gives it back.
        table_refcnt : IO I64;
        table_refcnt = (
            let ptr = *table.boxed_to_retained_ptr;
            let n = FFI_CALL[I64 fixtest_refcnt(Ptr), ptr];
            let _ = *(ptr.boxed_from_retained_ptr : IO Big);
            pure $ n
        );
    "#;

    /// Reading a global in a loop leaves its reference count unchanged. Each iteration hands the
    /// global to a consuming callee, which releases it; without the matching retain the count falls
    /// by one per iteration.
    #[test]
    fn test_reading_a_global_does_not_change_its_reference_count() {
        let source = PREAMBLE.to_string()
            + r#"
        // Building an array from `table` takes ownership of the reference it is given, so this is a
        // consuming use however far the optimizer sees into it.
        sum_through_array : I64 -> I64;
        sum_through_array = |n| loop((0, 0), |(i, acc)| (
            if i == n { break $ acc };
            let boxed = Array::fill(1, table);
            continue $ (i + 1, acc + boxed.@(0).@v)
        ));

        main : IO ();
        main = (
            let before = *table_refcnt;
            assert_eq(|_|"the sum through an array", sum_through_array(1000), 42000);;
            let after = *table_refcnt;
            assert_eq(|_|"the reference count of a global after 1000 reads", after, before);;
            pure()
        );
        "#;
        test_source_with_c(&source, READ_REFCNT, "global_refcount_balance");
    }

    /// A global's reference count is far enough from one that no uniqueness test calls it unique, so
    /// writing through it copies rather than mutating the global in place.
    #[test]
    fn test_a_global_is_never_unique() {
        let source = PREAMBLE.to_string()
            + r#"
        main : IO ();
        main = (
            let n = *table_refcnt;
            assert(|_|"a global's reference count reached one", n != 1);;
            // A `set` on a value the compiler called unique would write the global itself.
            let copy = table.set_v(7);
            assert_eq(|_|"the copy", copy.@v, 7);;
            assert_eq(|_|"the global", table.@v, 42);;
            pure()
        );
        "#;
        test_source_with_c(&source, READ_REFCNT, "global_refcount_uniqueness");
    }
}
