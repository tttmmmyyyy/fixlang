// Memory-safety test for a `match` arm that returns an outer-scope boxed local which is also live
// after the match. Returning the local consumes it, and the later use consumes it again, so RC
// insertion retains it at the return to supply the extra reference (a missing retain would double-free
// the array under valgrind). Both arms return the same outer `Array I64`, and it is read after the
// match, so the retain is required on whichever arm runs. `test_match_option` returns values derived
// from the payload, never an outer local live past the match, so this double-consume retain at a `Ret`
// is exercised only here.

#[cfg(test)]
mod match_return_outer_tests {
    use crate::{
        configuration::{Configuration, ValgrindTool},
        misc::{function_name, platform_valgrind_supported},
        tests::test_util::test_source,
    };

    const MATCH_RETURN_OUTER_SOURCE: &str = r#"
module Main;

main : IO () = (
    // some arm taken: it returns the outer `buf`, which is read again after the match
    let buf = [1, 2, 3];
    let opt : Option I64 = Option::some(0);
    let r = match opt {
        some(x) => buf,
        none(_) => buf
    };
    assert_eq(|_|"returned buf", r.@(0), 1);;
    assert_eq(|_|"buf still live", buf.@(2), 3);;

    // none arm taken: same double-consume of the returned outer local
    let buf2 = [10, 20];
    let opt2 : Option I64 = Option::none();
    let r2 = match opt2 {
        some(x) => buf2,
        none(_) => buf2
    };
    assert_eq(|_|"returned buf2", r2.@(1), 20);;
    assert_eq(|_|"buf2 still live", buf2.@(0), 10);;

    pure()
);
"#;

    #[test]
    pub fn test_match_return_outer_correctness() {
        test_source(MATCH_RETURN_OUTER_SOURCE, Configuration::develop_mode());
    }

    #[test]
    pub fn test_match_return_outer_memory_safety() {
        if !platform_valgrind_supported() {
            eprintln!(
                "Skipping {}: Valgrind not available on this platform.",
                function_name!()
            );
            return;
        }
        let mut config = Configuration::develop_mode();
        config.set_valgrind(ValgrindTool::MemCheck);
        test_source(MATCH_RETURN_OUTER_SOURCE, config);
    }
}
