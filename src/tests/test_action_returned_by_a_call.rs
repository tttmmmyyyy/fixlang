// A call of a global that returns an action, run right where the call is made, builds no closure
// for the action. Uncurrying gives a global one version for each number of arguments. A call that
// supplies both the arguments of the global and the state the action takes calls the version taking
// all of them, and that version runs the action directly.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, build_within_and_run, builds_no_closure_storing_anything,
        rc_ir_function_bodies,
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

    /// The function returning the action builds no closure that stores something. Such a closure
    /// costs one heap allocation per call, freed as soon as the caller has run it.
    ///
    /// The test reads the RC IR dump, because the program prints the same output either way.
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
                builds_no_closure_storing_anything(body),
                "`Main::run` should build no closure that stores something, but its body is:\n{}",
                body
            );
        }
    }

    /// The program prints the same output at every optimization level, with or without uncurrying.
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
