// Memory-safety tests for `as_<variant>` reading a fully-unboxed payload out of a union.
// Such a read borrows the union: the payload holds no reference, so the value read out takes nothing
// from the union, and reference-count insertion releases the union at its last use rather than at the
// read. The borrow matters most for a boxed union, whose container carries a reference count: an arm
// below reads a fully-unboxed variant of a boxed union, keeps the union shared, and reads it again;
// a reference dropped twice or too early is a use-after-free, checked under valgrind.

#[cfg(test)]
mod union_as_borrow_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const UNION_AS_BORROW_SOURCE: &str = r#"
module Main;

// A boxed union with a fully-unboxed, data-bearing variant (`pair`) and a boxed variant (`buf`).
type BoxOpt = box union { pair : (I64, I64), buf : Array I64 };

// Read the fully-unboxed variant's payload twice, keeping the boxed union alive across both reads.
read_pair_twice : BoxOpt -> I64 = |u| (
    let (a, b) = u.as_pair;
    let (c, d) = u.as_pair;
    a + b + c + d
);

main : IO () = (
    // A boxed union shared between `read_pair_twice` and a later read: the borrowed reads must leave
    // it usable, and it must be released exactly once.
    let u = BoxOpt::pair((10, 20));
    let s = read_pair_twice(u);
    assert_eq(|_|"boxed union, fully-unboxed payload read twice", s, 60);;
    assert_eq(|_|"boxed union still usable after the borrowed reads", u.as_pair.@0, 10);;

    // The boxed variant's payload is not fully unboxed, so its read takes the owning path.
    let v = BoxOpt::buf([1, 2, 3]);
    assert_eq(|_|"boxed variant read", v.as_buf.@size, 3);;

    // An unboxed union with the same shapes: the fully-unboxed variant read, union kept and reused.
    let w : UnboxOpt = UnboxOpt::pair((7, 8));
    let t = match w { pair(_) => w.as_pair.@0 + w.as_pair.@1, buf(_) => 0 };
    assert_eq(|_|"unboxed union, fully-unboxed payload reused", t, 15);;

    pure()
);

type UnboxOpt = unbox union { pair : (I64, I64), buf : Array I64 };
"#;

    /// The borrowed reads compute the right values, so the borrow leaves the payload intact.
    #[test]
    pub fn test_union_as_borrow_correctness() {
        test_source(UNION_AS_BORROW_SOURCE, Configuration::develop_mode());
    }

    /// The borrowed reads free the union exactly once and leak nothing, checked under Valgrind MemCheck.
    #[test]
    pub fn test_union_as_borrow_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(UNION_AS_BORROW_SOURCE, config);
    }
}
