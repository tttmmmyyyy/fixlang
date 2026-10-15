// An `IO` action chosen by an `if` or a `match` in the body of a loop is run without building a
// closure for it. Once the loop is specialized for its body, the choice and the application of the
// action stand side by side, and the branch taken is reduced where the action runs. The choice is
// moved only where every value's lifetime stays as it was. A program sees this as an update in place
// staying in place, and these tests see it through `unsafe_is_unique`.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{build_run_and_read_rc_ir, build_within_and_run};
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
        assert_no_io_closure_is_built(
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
        assert_no_io_closure_is_built(
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
}
