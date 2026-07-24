// Memory-safety tests for a `match` on a boxed union with a bare catch-all arm — a variable pattern
// that binds the whole scrutinee (here written `rest`). Such an arm carries no variant, so the
// boxed-union container has no separate release; the scrutinee flows into the arm as the payload and
// is disposed through it — released once when the arm drops it, or consumed when the arm uses it. With
// a boxed payload, the catch-all taken and not taken, and the payload both dropped and used, every
// value must be released exactly once — checked under valgrind. `test_union_match` matches only
// variant arms, so this variant-less catch-all arm on a boxed scrutinee is exercised only here.

#[cfg(test)]
mod union_catchall_match_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const UNION_CATCHALL_MATCH_SOURCE: &str = r#"
module Main;

type BoxOpt = box union { some : Array I64, none : () };

main : IO () = (
    // catch-all arm not taken (a variant arm is taken); scrutinee released in the variant arm
    let u1 = BoxOpt::some([1, 2, 3]);
    let r1 = match u1 {
        some(a) => a.@(0),
        rest => -1
    };
    assert_eq(|_|"variant taken", r1, 1);;

    // catch-all arm taken, its whole-union payload dropped (disposed through the payload)
    let u2 : BoxOpt = BoxOpt::none();
    let r2 = match u2 {
        some(a) => a.@(0),
        rest => 99
    };
    assert_eq(|_|"catchall taken, payload dropped", r2, 99);;

    // catch-all arm taken, its whole-union payload used (re-matched), disposing it there
    let u3 = BoxOpt::some([7, 8]);
    let r3 = match u3 {
        none(_) => 0,
        rest => (
            match rest {
                some(a) => a.@size,
                none(_) => -1
            }
        )
    };
    assert_eq(|_|"catchall taken, payload used", r3, 2);;

    pure()
);
"#;

    #[test]
    pub fn test_union_catchall_match_correctness() {
        test_source(UNION_CATCHALL_MATCH_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_union_catchall_match_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(UNION_CATCHALL_MATCH_SOURCE, config);
    }
}
