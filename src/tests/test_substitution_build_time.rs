// Putting an expression where a name stands walks only the subexpressions the name is free in, and
// the bodies a symbol receives go in together, so the cost of a substitution follows the places it
// changes rather than the size of the expression it is made in. A function calling a scope builtin
// many times is where that matters: the optimizer moves the body of each call's function into the
// caller and looks for each name's uses there, and a walk over all of the caller for each of them
// takes a time that grows with the square of the calls. The deadline on the build is what measures
// that.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// How many times the function calls `borrow_c_str`. Long enough that walking the whole function
    /// for each call's body takes more than `TIMEOUT`, and short enough that the build walking only
    /// where the names are free finishes well within it.
    const CALLS: usize = 1200;

    /// Several times what the build takes when each substitution walks only where its names are
    /// free, so that a slower machine passes, and short enough to report a substitution walking the
    /// whole function as a failure.
    const TIMEOUT: Duration = Duration::from_secs(45);

    /// A function binding the results of `CALLS` calls of `borrow_c_str` in a chain, each call's
    /// function reading the binding before it, builds within `TIMEOUT`. The string comes from the
    /// program's arguments, which the compiler cannot fold away.
    #[test]
    fn test_a_function_calling_a_scope_builtin_many_times_builds_in_time() {
        let mut source = "module Main;\n\n\
                          f : String -> I64;\n\
                          f = |s| (\n\
                          \x20   let a0 = 0;\n"
            .to_string();
        for i in 1..=CALLS {
            source += &format!(
                "    let a{i} = s.borrow_c_str(|p| a{} + (p.offset_from(p) + {}));\n",
                i - 1,
                i + 1
            );
        }
        source += &format!(
            "    a{CALLS}\n\
             );\n\
             \n\
             main : IO ();\n\
             main = println(f(\"x\" + (*get_args).@size.to_string).to_string);\n"
        );
        // Each call adds `i + 1` for its index `i`.
        let expected = (1..=CALLS).map(|i| i + 1).sum::<usize>().to_string();

        let output = build_within_and_run(
            &source,
            "max",
            TIMEOUT,
            &format!("a function calling `borrow_c_str` {} times", CALLS),
        );
        assert_eq!(output, expected);
    }
}
