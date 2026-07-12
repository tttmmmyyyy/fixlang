// Memory-safety tests for `match` on a boxed union with a boxed payload.
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
}
