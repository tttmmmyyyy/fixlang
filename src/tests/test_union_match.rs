// Memory-safety tests for `match`: on a boxed union with a boxed payload, and on a scrutinee an arm
// body reads.
// Matching a boxed union extracts the payload with `get_union_value`, whose boxed path retains the
// payload and releases the scrutinee container. With a boxed payload, an arm that drops the payload,
// and a scrutinee still used after the match, must leave every value released exactly once — checked
// under valgrind. `test_match_option` matches the unboxed `Std::Option`, so the boxed-union
// container's retain/release is never exercised; these tests use an explicit `box union`.

#[cfg(test)]
mod union_match_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const BOXED_UNION_MATCH_SOURCE: &str = r#"
module Main;

type BoxOpt = box union { some : Array I64, none : () };

main : IO () = (
    // scrutinee last-use, boxed payload used
    let u1 = BoxOpt::some([1, 2, 3]);
    let s1 = match u1 { some(a) => a.@(0), none(_) => -1 };
    assert_eq(|_|"some used", s1, 1);;

    // scrutinee last-use, boxed payload dropped in the taken arm (must be released)
    let u2 = BoxOpt::some([4, 5]);
    let s2 = match u2 { some(a) => 99, none(_) => -1 };
    assert_eq(|_|"payload dropped", s2, 99);;

    // scrutinee used after the match (retained before, released inside the arm)
    let u3 = BoxOpt::some([7, 8, 9]);
    let s3 = match u3 { some(a) => a.@size, none(_) => -1 };
    let s3b = match u3 { some(a) => a.@(2), none(_) => -1 };
    assert_eq(|_|"used after match", s3 + s3b, 12);;

    // none arm carries no payload; the scrutinee container is still released
    let u4 : BoxOpt = BoxOpt::none();
    let s4 = match u4 { some(a) => a.@size, none(_) => 0 };
    assert_eq(|_|"none arm", s4, 0);;

    pure()
);
"#;

    // A `match` consumes its scrutinee at the head of the arm it takes — a boxed union releases the
    // container there, an unboxed union lets the payload carry it away — so an arm body that reads
    // the scrutinee itself reads it after that consumption and needs its own reference. An arm that
    // does not read it must release the extra reference exactly once, on top of the container release.
    const SCRUTINEE_READ_IN_ARM_SOURCE: &str = r#"
module Main;

type BoxOpt = box union { some : Array I64, none : () };
type UnboxOpt = unbox union { some : Array I64, none : () };

// The arm body reads the scrutinee after the arm's head released the container.
boxed_in_arm : BoxOpt -> I64 = |u| (
    match u {
        some(a) => a.@(0) + (if u.is_some { 1 } else { 0 }),
        none(_) => 0
    }
);

// The scrutinee is read inside an arm and again after the match.
boxed_in_arm_and_after : BoxOpt -> I64 = |u| (
    let inner = match u {
        some(a) => a.@(0) + (if u.is_some { 1 } else { 0 }),
        none(_) => 0
    };
    inner + (if u.is_some { 100 } else { 0 })
);

// An unboxed union: the payload aliases the scrutinee, so a read of the scrutinee needs a reference
// of its own just the same.
unboxed_in_arm : UnboxOpt -> I64 = |u| (
    match u {
        some(a) => a.@size + (if u.is_some { 1 } else { 0 }),
        none(_) => 0
    }
);

main : IO () = (
    assert_eq(|_|"boxed, arm reads scrutinee", boxed_in_arm(BoxOpt::some([10, 20, 30])), 11);;
    assert_eq(|_|"boxed, arm without the read", boxed_in_arm(BoxOpt::none()), 0);;
    assert_eq(|_|"boxed, read in arm and after", boxed_in_arm_and_after(BoxOpt::some([10, 20, 30])), 111);;
    assert_eq(|_|"unboxed, arm reads scrutinee", unboxed_in_arm(UnboxOpt::some([10, 20, 30])), 4);;
    assert_eq(|_|"unboxed, arm without the read", unboxed_in_arm(UnboxOpt::none()), 0);;
    pure()
);
"#;

    #[test]
    pub fn test_boxed_union_match_correctness() {
        test_source(BOXED_UNION_MATCH_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_boxed_union_match_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(BOXED_UNION_MATCH_SOURCE, config);
    }

    #[test]
    pub fn test_scrutinee_read_in_arm_correctness() {
        test_source(SCRUTINEE_READ_IN_ARM_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_scrutinee_read_in_arm_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(SCRUTINEE_READ_IN_ARM_SOURCE, config);
    }
}
