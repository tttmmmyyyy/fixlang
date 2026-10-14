// An `IO` action chosen by an `if` or a `match` is run without building a closure for it: the branch
// is eta expanded into a lambda, which goes to where the action is run and is reduced there. The
// expansion moves the branches to where the action runs, so it is held back wherever that would keep
// a value alive longer than before — which a program observes as an update in place turning into a
// copy, and these tests observe through `unsafe_is_unique`.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{
        build_and_run_within, build_run_and_read_rc_ir, build_within_and_run,
    };
    use std::time::Duration;

    /// What a closure holding an `IO ()` action is declared as in the RC IR dump.
    const IO_UNIT_CLOSURE: &str = ": Std::IO::IOState -> (Std::IO::IOState, ()) ";

    /// The lines of `dump` that build a closure holding an `IO ()` action.
    fn io_closures_built(dump: &str) -> Vec<&str> {
        dump.lines()
            .filter(|line| line.contains(IO_UNIT_CLOSURE) && line.contains("= closure "))
            .collect()
    }

    /// Builds `source` at `-O max`, runs it, and asserts that it prints `expected_output` and builds
    /// no closure holding an `IO ()` action.
    fn assert_no_io_closure_is_built(source: &str, expected_output: &str, description: &str) {
        let dump = build_run_and_read_rc_ir(source, "max", expected_output, description);
        let built = io_closures_built(&dump);
        assert!(
            built.is_empty(),
            "{} should run the action it chooses without building a closure for it, and builds:\n{}",
            description,
            built.join("\n")
        );
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
        assert_no_io_closure_is_built(
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
        assert_no_io_closure_is_built(
            source,
            "abcbcabcbc",
            "a `match` choosing which action a round of a loop runs",
        );
    }

    /// One branch reads the counter of the loop and the other does not. A value without a reference
    /// count costs nothing to keep longer, so the branches may read different values of that kind.
    #[test]
    fn test_branches_reading_different_uncounted_values_build_no_closure() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let n = (*get_args).@size * 6;
            loop_m(0, |i|
                if i == n { break_m $ () };
                (if i % 3 == 0 { IO::from_runner(|ios| (print(i.to_string).@runner)(ios)) } else { print("_") });;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        assert_no_io_closure_is_built(
            source,
            "0__3__",
            "an `if` whose branches read different values without a reference count",
        );
    }

    /// The action is chosen once, before a loop of ten thousand rounds runs it, and the branch taken
    /// computes the text it prints by a fold over ten million numbers before its action. That
    /// computation stays where the action is chosen, so it runs once; moved under the lambda the
    /// action becomes, it would run every round and take minutes.
    #[test]
    fn test_what_a_branch_computes_before_its_action_runs_once() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let action = if k == 1 { print(Iterator::range(0, 10000000).fold(0, |i, s| s + i % 7).to_string) } else { print("y") };
            loop_m(0, |i|
                if i == 10000 { break_m $ () };
                action;;
                continue_m $ i + 1
            );;
            println("")
        );
        "#;
        let printed = build_and_run_within(
            source,
            "max",
            &[],
            Duration::from_secs(10),
            "an action whose branch computes its text before the action, run ten thousand times",
        );
        assert_eq!(printed, "29999994".repeat(10000));
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

    /// The condition of the `if` reads a boxed value, and the actions do not. The condition is
    /// evaluated where the `if` is, so the value is not kept for the action to read it.
    #[test]
    fn test_the_condition_is_evaluated_where_the_if_is() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let cell = Cell { v : 0 };
            let action = if cell.@v == 0 { print("a") } else { print("b") };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "a\ntrue",
            "an `if` whose condition reads a boxed value",
        );
    }

    /// One branch keeps a boxed value for its action to read, and the branch taken does not. The
    /// action of the branch taken keeps nothing, so the value is not kept.
    #[test]
    fn test_what_only_the_branch_not_taken_reads_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let action = if k == 1 { print("x") } else { IO::from_runner(|ios| (print(cell.@v.to_string).@runner)(ios)) };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "x\ntrue",
            "an `if` where the branch not taken keeps a boxed value",
        );
    }

    /// Each branch reads a boxed value to make the text its action prints, before the action. What
    /// the action keeps is the text, so the value is not kept.
    #[test]
    fn test_what_a_branch_reads_before_its_action_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let action = if k == 1 { print(cell.@v.to_string) } else { print((cell.@v + 1).to_string) };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "0\ntrue",
            "an `if` whose branches read a boxed value before their actions",
        );
    }

    /// The `Option` holds a boxed value, and the pattern of the arm taken drops it. The value is not
    /// kept by the action.
    #[test]
    fn test_what_a_pattern_drops_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let o = if k == 1 { some(cell) } else { none() };
            let action = match o { some(_) => print("x"), none() => print("y") };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "x\ntrue",
            "a `match` whose pattern drops the boxed value the `Option` holds",
        );
    }

    /// One arm of a `match` keeps a boxed value for its action to read, and the arm taken does not.
    /// The action of the arm taken keeps nothing, so the value is not kept.
    #[test]
    fn test_what_only_the_arm_not_taken_reads_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let o = if k == 1 { some(1) } else { none() };
            let action = match o { some(_) => print("x"), none() => IO::from_runner(|ios| (print(cell.@v.to_string).@runner)(ios)) };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "x\ntrue",
            "a `match` where the arm not taken keeps a boxed value",
        );
    }

    /// The struct matched holds a boxed value, and the pattern leaves out the field holding it. The
    /// value is not kept by the action.
    #[test]
    fn test_what_a_struct_pattern_leaves_out_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };
        type Pair = struct { c : Cell, n : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let p = Pair { c : cell, n : k };
            let action = match p { Pair { n, _ } => IO::from_runner(|ios| (print(n.to_string).@runner)(ios)) };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "1\ntrue",
            "a `match` whose struct pattern leaves out the field holding a boxed value",
        );
    }

    /// The pattern takes apart a boxed struct the `Option` holds, and binds only a field without a
    /// reference count. The box is released where the `match` stands, so it is not kept by the
    /// action.
    #[test]
    fn test_a_box_a_pattern_takes_apart_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 5 };
            let o = if k == 1 { some(cell) } else { none() };
            let action = match o { some(Cell { v : v }) => IO::from_runner(|ios| (print(v.to_string).@runner)(ios)), none() => print("y") };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "5\ntrue",
            "a `match` whose pattern takes apart a boxed struct",
        );
    }

    /// A struct pattern naming some of the fields without `_` drops the others, as one with `_`
    /// does. The field it drops is not kept by the action.
    #[test]
    fn test_what_a_struct_pattern_without_a_wildcard_leaves_out_is_not_kept() {
        let source = r#"
        module Main;

        type Cell = box struct { v : I64 };
        type Pair = struct { c : Cell, n : I64 };

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 0 };
            let p = Pair { c : cell, n : k };
            let action = match p { Pair { n } => IO::from_runner(|ios| (print(n.to_string).@runner)(ios)) };
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "1\ntrue",
            "a `match` whose struct pattern names some of the fields without `_`",
        );
    }

    /// The source of a program whose branches both read a boxed value when the action is chosen, and
    /// in which the read of the branch taken goes away once `second6` is inlined: `second6`
    /// answers with its second argument, through seven calls the inliner takes one round each.
    /// `{action}` is replaced with the expression choosing the action.
    const READ_THAT_GOES_AWAY_SOURCE: &str = r#"
        module Main;

        type Cell = box struct { v : I64 };

        second0 : a -> b -> b;
        second0 = |a, b| b;
        second1 : a -> b -> b;
        second1 = |a, b| second0(a, b);
        second2 : a -> b -> b;
        second2 = |a, b| second1(a, b);
        second3 : a -> b -> b;
        second3 = |a, b| second2(a, b);
        second4 : a -> b -> b;
        second4 = |a, b| second3(a, b);
        second5 : a -> b -> b;
        second5 = |a, b| second4(a, b);
        second6 : a -> b -> b;
        second6 = |a, b| second5(a, b);

        main : IO ();
        main = (
            let k = (*get_args).@size;
            let cell = Cell { v : 5 };
            let action = {action};
            let (unique, cell) = cell.unsafe_is_unique;
            action;;
            println("");;
            println(unique.to_string)
        );
    "#;

    /// The branch taken reads the boxed value when the action is chosen, and the read goes away once
    /// the program is optimized, so the action keeps nothing. The value is not kept for the branch
    /// not taken.
    #[test]
    fn test_a_read_that_goes_away_keeps_nothing() {
        let if_source = READ_THAT_GOES_AWAY_SOURCE.replace(
            "{action}",
            "if k == 1 { IO::from_runner(|ios| (second6(cell, print(\"x\")).@runner)(ios)) } else { IO::from_runner(|ios| (print(cell.@v.to_string).@runner)(ios)) }",
        );
        assert_the_value_stays_unique(
            &if_source,
            "x\ntrue",
            "an `if` whose branch taken reads a boxed value until it is optimized",
        );
        let match_source = READ_THAT_GOES_AWAY_SOURCE.replace(
            "{action}",
            "match (if k == 1 { some(cell) } else { none() }) { some(c) => IO::from_runner(|ios| (second6(c, print(\"x\")).@runner)(ios)), none() => print(\"y\") }",
        );
        assert_the_value_stays_unique(
            &match_source,
            "x\ntrue",
            "a `match` whose arm taken reads the boxed value it binds until it is optimized",
        );
    }

    /// The `match` is on a boxed union, which an array holds as well. The box is not kept by the
    /// action, so it stays unique in the array, and an update of it through the array is made in
    /// place.
    #[test]
    fn test_a_boxed_value_matched_is_not_kept() {
        let source = r#"
        module Main;

        type B = box union { a : I64, b : I64 };

        main : IO ();
        main = (
            let xs = Array::fill(1, B::a(7));
            let o = xs.@(0);
            let action = match o { a(_) => print("a"), b(_) => print("b") };
            let xs = *xs.act(0, |o| let (unique, o) = o.unsafe_is_unique; println(unique.to_string);; pure(o));
            action;;
            println("")
        );
        "#;
        assert_the_value_stays_unique(
            source,
            "true\na",
            "a `match` on a boxed union an array holds",
        );
    }
}
