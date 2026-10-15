// Putting an expression where a name stands walks only the subexpressions the name is free in, so
// each substitution costs the path down to the places it changes rather than the whole expression
// it is made in. A `main` running many actions one after another is where that matters: the
// optimizer moves a body into `main` for each action and looks for each name's uses there, and a
// walk over all of `main` for each of them takes a time that grows with the square of the actions.
// The deadline on the build is what measures that.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// How many actions `main` runs. Long enough that walking all of `main` for each action takes
    /// several times `TIMEOUT`, and short enough that the build walking only where the names are
    /// free finishes well within it.
    const ACTIONS: usize = 400;

    /// Several times what the build takes when each substitution walks only where its names are
    /// free, so that a slower machine passes, and short enough to report a substitution walking all
    /// of `main` as a failure.
    const TIMEOUT: Duration = Duration::from_secs(90);

    /// A `main` that binds each of `ACTIONS` actions, chosen by an `if`, to a name and runs it right
    /// after, builds within `TIMEOUT`. Every condition reads the number of arguments, which the
    /// compiler cannot fold away, and is false for a run given none: `1 % (i + 2)` is `1`.
    #[test]
    fn test_a_main_running_many_chosen_actions_builds_in_time() {
        let mut source = "module Main;\n\n\
                          main : IO ();\n\
                          main = (\n\
                          \x20   let k = (*get_args).@size;\n"
            .to_string();
        let mut expected = String::new();
        for i in 0..ACTIONS {
            source += &format!(
                "    let a{i} = if k % {} == 0 {{ print(\"x{i}\") }} else {{ print(\"y{i}\") }};\n\
                 \x20   a{i};;\n",
                i + 2
            );
            expected += &format!("y{i}");
        }
        source += "    pure()\n);\n";

        let output = build_within_and_run(
            &source,
            "max",
            TIMEOUT,
            &format!("a main running {} actions chosen by an `if`", ACTIONS),
        );
        assert_eq!(output, expected);
    }
}
