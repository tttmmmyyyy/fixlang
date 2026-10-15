// An action chosen by an `if` or a `match` in the body of a loop is run without building a closure
// for it, for `IO` and for any other monad whose actions are functions. Once the loop is specialized
// for its body, the choice and the application of the action stand side by side, and the branch
// taken is reduced where the action runs. The choice is moved only where every value's lifetime stays
// as it was. A program sees this as an update in place staying in place, and these tests see it
// through `unsafe_is_unique`.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_run_and_read_rc_ir, build_within_and_run, rc_ir_function_bodies,
    };
    use std::time::Duration;

    /// Builds `source` at `-O max`, runs it, and asserts that it prints `expected_output` and that
    /// the loop of `Std::loop_m` builds no closure that stores something.
    fn assert_the_loop_builds_no_closure(source: &str, expected_output: &str, description: &str) {
        let dump = build_run_and_read_rc_ir(source, "max", expected_output, description);
        assert_loop_m_builds_no_closure(&dump);
    }

    /// An `if` choosing which of two actions to run, once per round of a loop.
    #[test]
    fn test_an_if_choosing_an_action_builds_no_closure() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*get_args).@size * 6;
            loop_m(0, |i|
                if i == n { break_m $ () };
                (if i % 3 == 0 { print("a") } else { print("bc") });;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        assert_the_loop_builds_no_closure(
            source,
            "abcbcabcbc",
            "an `if` choosing which action a round of a loop runs",
        );
    }

    /// A `match` on an `Option` choosing which of two actions to run. The `Option` is read out of an
    /// array, so the compiler cannot fold the `match` away.
    #[test]
    fn test_a_match_choosing_an_action_builds_no_closure() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*get_args).@size * 6;
            let picks = [some(0), none(), none()];
            loop_m(0, |i|
                if i == n { break_m $ () };
                let o = picks.@(i % 3);
                (match o { some(_) => print("a"), none() => print("bc") });;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        assert_the_loop_builds_no_closure(
            source,
            "abcbcabcbc",
            "a `match` choosing which action a round of a loop runs",
        );
    }

    /// The action of one branch prints a string, which holds a reference count, and the action of
    /// the other prints a literal.
    #[test]
    fn test_an_if_whose_one_branch_reads_a_string_builds_no_closure() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*get_args).@size * 6;
            let s = "a" + n.to_string;
            loop_m(0, |i|
                if i == n { break_m $ () };
                (if i % 3 == 0 { print(s) } else { print("_") });;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        assert_the_loop_builds_no_closure(
            source,
            "a6__a6__",
            "an `if` whose one branch prints a string",
        );
    }

    /// The arm taken prints the string the `Option` holds.
    #[test]
    fn test_a_match_printing_the_string_an_option_holds_builds_no_closure() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*get_args).@size * 6;
            let picks = [some("a"), none(), some("bc")];
            loop_m(0, |i|
                if i == n { break_m $ () };
                (match picks.@(i % 3) { some(s) => print(s), none() => print("_") });;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        assert_the_loop_builds_no_closure(
            source,
            "a_bca_bc",
            "a `match` whose arm prints the string an `Option` holds",
        );
    }

    /// Builds `source` at `-O max`, runs it, and asserts that it prints `expected_output`, one line
    /// of which says whether the value was unique where the program asked.
    fn assert_the_value_stays_unique(source: &str, expected_output: &str, description: &str) {
        assert_eq!(
            build_within_and_run(source, "max", Duration::from_secs(600), description),
            expected_output,
            "{} should leave the value unique where the program asks, since nothing the program \
             runs after that reads it",
            description
        );
    }

    /// In a round of a loop, one branch keeps a boxed value for its action to read, and the branch
    /// taken does not. The program asks whether the value is unique after choosing the action and
    /// before running it, so the choice stays where it is, and the value is not kept.
    #[test]
    fn test_what_only_the_branch_not_taken_reads_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            loop_m(0, |i|
                if i == 2 { break_m $ () };
                let cell = Cell { v : i };
                let action = if k == 1 { print("x") } else { IO::from_runner(|ios| (print(cell.@v.to_string).@runner)(ios)) };
                let (unique, cell) = cell.unsafe_is_unique;
                action;;
                println("");;
                println(unique.to_string);;
                continue_m $ i + 1
            )
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "x\ntrue\nx\ntrue",
            "an `if` in a loop where the branch not taken keeps a boxed value",
        );
    }

    /// In a round of a loop, the condition of the `if` reads a boxed value, and the actions do not.
    /// The condition is evaluated where the `if` is, so the value is not kept for the action.
    #[test]
    fn test_the_condition_is_evaluated_where_the_if_is() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            loop_m(0, |i|
                if i == 2 { break_m $ () };
                let cell = Cell { v : i };
                let action = if cell.@v == 0 { print("a") } else { print("b") };
                let (unique, cell) = cell.unsafe_is_unique;
                action;;
                println("");;
                println(unique.to_string);;
                continue_m $ i + 1
            )
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "a\ntrue\nb\ntrue",
            "an `if` in a loop whose condition reads a boxed value",
        );
    }

    /// A state monad whose `bind` threads a counter, looped over with `Std::loop_m`.
    ///
    /// `loop_m` answers with an action of the monad it loops in, so each round of it hands `bind`
    /// the action the body produced. Specializing the closure the body is puts that body into
    /// `loop_m`, where it answers with a lambda under each arm of an `if`, and the application of
    /// that lambda to the state is what has to reach the arms.
    const MONADIC_LOOP: &str = r#"
        module Main;

        type St a = unbox struct { _run : I64 -> (a, I64) };

        namespace St {
            run : I64 -> St a -> (a, I64);
            run = |s, m| (m.@_run)(s);
        }

        impl St : Functor {
            map = |f, m| St { _run : |s| let (v, s) = m.run(s); (f(v), s) };
        }

        impl St : Monad {
            pure = |v| St { _run : |s| (v, s) };
            bind = |f, m| St { _run : |s| let (v, s) = m.run(s); f(v).run(s) };
        }

        _tick : St I64;
        _tick = St { _run : |s| (s, s + 1) };

        main : IO () = (
            let counting = loop_m((0, 0), |(i, total)|
                if i == 100 { break_m $ total };
                let v = *_tick;
                continue_m $ (i + 1, total + v)
            );
            let (total, _) = counting.St::run(0);
            println(total.to_string)
        );
    "#;

    /// What `MONADIC_LOOP` prints: the sum of `0..99`.
    const MONADIC_LOOP_OUTPUT: &str = "4950";

    /// Asserts that `dump` holds the loop of `Std::loop_m`, and that no copy of it builds a
    /// closure that stores something.
    ///
    /// A closure storing nothing is a function pointer beside a null capture pointer, which costs
    /// no allocation: the function an `IO` action gives a built-in applying it in a scope is one,
    /// taking what it captures through the built-in's environment.
    ///
    /// The call the loop makes to itself is asserted alongside the closure's absence, so that a
    /// build where the loop no longer lives in `Std::loop_m` fails here rather than reading a
    /// function that has nothing left to build.
    fn assert_loop_m_builds_no_closure(dump: &str) {
        let bodies = rc_ir_function_bodies(dump, "Std::loop_m");
        assert!(
            !bodies.is_empty(),
            "the dump names no `Std::loop_m`:\n{}",
            dump
        );
        assert!(
            bodies.iter().any(|body| body
                .lines()
                .skip(1)
                .any(|line| line.contains("Std::loop_m"))),
            "no `Std::loop_m` calls itself, so the loop this pins is not in the dump:\n{}",
            bodies.join("\n")
        );
        for body in &bodies {
            assert!(
                body.lines()
                    .filter(|line| line.contains("= closure "))
                    .all(|line| line.trim_end().ends_with("[]")),
                "the loop should build no closure that stores something, but its body is:\n{}",
                body
            );
        }
    }

    /// A loop through a monad builds no closure: `loop_m` hands the state to the action its body
    /// answered with, and that application reaches the arms of the `if` the body ends in, so the
    /// lambda of the arm taken is never built. Left unreached, it is one heap allocation per round
    /// of the loop.
    ///
    /// The dump is what this asserts against because the program cannot observe it: the loop
    /// answers the same either way.
    #[test]
    fn test_a_loop_through_a_monad_builds_no_closure() {
        let dump = build_run_and_read_rc_ir(
            MONADIC_LOOP,
            "max",
            MONADIC_LOOP_OUTPUT,
            "a state monad looped over with `loop_m`",
        );
        assert_loop_m_builds_no_closure(&dump);
    }

    /// A loop in `IO`, performing an action each round, written the way `Std::loop_m` documents
    /// itself.
    ///
    /// `IO` reaches the shape by a route of its own: `IO a` is a one-field unboxed struct over
    /// `IOState -> (IOState, a)`, which unwrapping opens into that function type, and the loop
    /// threads the `IOState` through each round.
    const IO_LOOP: &str = r#"
        module Main;

        main : IO () = (
            let total = *loop_m((0, 0), |(i, total)|
                if i == 3 { break_m $ total };
                println("round " + i.to_string);;
                continue_m $ (i + 1, total + i)
            );
            println(total.to_string)
        );
    "#;

    /// What `IO_LOOP` prints: a line per round, then the sum of `0..2`.
    const IO_LOOP_OUTPUT: &str = "round 0\nround 1\nround 2\n3";

    /// A loop in `IO` builds no closure either, which is what `Std` itself pays: `read_string` and
    /// `get_args` both loop this way.
    #[test]
    fn test_a_loop_in_io_builds_no_closure() {
        let dump = build_run_and_read_rc_ir(
            IO_LOOP,
            "max",
            IO_LOOP_OUTPUT,
            "an `IO` action looped over with `loop_m`",
        );
        assert_loop_m_builds_no_closure(&dump);
    }

    /// The answer of a loop through a monad does not depend on the optimization level. The pass
    /// that flattens the application into the arms runs at `-O max` alone, so the levels below it
    /// are what says the flattening leaves the answer where it was.
    #[test]
    fn test_a_loop_through_a_monad_answers_the_same_at_every_level() {
        for opt_level in ["none", "basic", "max"] {
            assert_eq!(
                build_within_and_run(
                    MONADIC_LOOP,
                    opt_level,
                    Duration::from_secs(600),
                    "a state monad looped over with `loop_m`",
                ),
                MONADIC_LOOP_OUTPUT,
                "the loop should answer the same at -O {}",
                opt_level
            );
        }
    }
}
