// Putting an expression where a name stands walks only the subexpressions the name is free in, and
// the bodies a symbol receives go in together, so the cost of a substitution follows the places it
// changes. A function calling a scope builtin many times is where that matters: the optimizer moves
// the body of each call's function into the caller and looks for each name's uses there, and a walk
// over all of the caller for each of them does work that grows with the square of the calls. The
// bound on the instructions the build executes is what measures that.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_counting_instructions_and_run;

    /// How many times the function calls `borrow_c_str`. Long enough that walking the whole function
    /// for each call's body executes more than `MAX_INSTRUCTIONS`.
    const CALLS: usize = 1200;

    /// About twice what the build executes when each substitution walks only where its names are
    /// free (98 billion on x86-64), and a fifth of what it executes when each walks the whole
    /// function (506 billion).
    const MAX_INSTRUCTIONS: u64 = 200_000_000_000;

    /// A function binding the results of `CALLS` calls of `borrow_c_str` in a chain, each call's
    /// function reading the binding before it, builds in fewer than `MAX_INSTRUCTIONS` instructions
    /// and computes the sum. The string comes from the program's arguments, which the compiler
    /// cannot fold away. Where the machine does not let the test count instructions, the test
    /// checks the sum alone.
    #[test]
    fn test_a_function_calling_a_scope_builtin_many_times_builds_in_bounded_work() {
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

        let description = format!("a function calling `borrow_c_str` {} times", CALLS);
        let (output, instructions) =
            build_counting_instructions_and_run(&source, "max", &description);
        assert_eq!(output, expected);
        match instructions {
            Some(instructions) => assert!(
                instructions < MAX_INSTRUCTIONS,
                "compiling {} executed {} instructions, beyond the bound of {}",
                description,
                instructions,
                MAX_INSTRUCTIONS
            ),
            None => eprintln!(
                "compiling {}: this machine does not let the test count instructions, so the bound \
                 went unchecked",
                description
            ),
        }
    }
}
