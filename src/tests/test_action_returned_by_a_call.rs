// An action that a call of a global returns, run where the call is made, is run without building a
// closure for it. Uncurrying gives a global a version per number of arguments, and a call supplying
// the arguments of the global and the state the action takes calls the version taking them all, which
// runs the action in place of answering with it.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, build_within_and_run, rc_ir_function_bodies,
    };
    use std::time::Duration;

    /// A function returning an `IO` action, too large to be inlined into its caller, called once per
    /// round of a loop and chained to the rest of the round with `;;`.
    ///
    /// The bind of `;;` names the action outside the lambda that takes the state, so the call and
    /// the application of the action to the state meet only once the local functions are inlined
    /// after uncurrying.
    const ACTION_RETURNED_BY_A_CALL: &str = r#"
        module Main;

        run : I64 -> IO () = |i| (
            print("a");; print("b");; print("c");;
            if i < 0 { print("d") } else { pure() }
        );

        main : IO () = (
            let n = (*get_args).@size * 2;
            loop_m(0, |i| if i == n { break_m $ () }; run(i);; continue_m $ i + 1)
        );
    "#;

    /// What `ACTION_RETURNED_BY_A_CALL` prints: a round per iteration.
    const ACTION_RETURNED_BY_A_CALL_OUTPUT: &str = "abcabc";

    /// The function returning the action builds no closure that stores something. Left built, it is
    /// one heap allocation per call, freed as soon as the caller has run it.
    ///
    /// The dump is what this asserts against because the program cannot observe it: the program
    /// prints the same either way.
    #[test]
    fn test_an_action_returned_by_a_call_builds_no_closure() {
        let dump = build_run_and_read_rc_ir(
            ACTION_RETURNED_BY_A_CALL,
            "max",
            ACTION_RETURNED_BY_A_CALL_OUTPUT,
            "a function returning an `IO` action, called once per round of a loop",
        );
        let bodies = rc_ir_function_bodies(&dump, "Main::run");
        assert!(
            !bodies.is_empty(),
            "the dump names no `Main::run`:\n{}",
            dump
        );
        for body in &bodies {
            assert!(
                body.lines()
                    .filter(|line| line.contains("= closure "))
                    .all(|line| line.trim_end().ends_with("[]")),
                "`Main::run` should build no closure that stores something, but its body is:\n{}",
                body
            );
        }
    }

    /// The program answers the same at every optimization level, uncurrying or not.
    #[test]
    fn test_an_action_returned_by_a_call_answers_the_same_at_every_level() {
        for opt_level in ["none", "basic", "max"] {
            assert_eq!(
                build_within_and_run(
                    ACTION_RETURNED_BY_A_CALL,
                    opt_level,
                    Duration::from_secs(600),
                    "a function returning an `IO` action, called once per round of a loop",
                ),
                ACTION_RETURNED_BY_A_CALL_OUTPUT,
                "the program should answer the same at -O {}",
                opt_level
            );
        }
    }
}
