// Inlining substitutes the body of a global into the places that name it, and what a body costs
// says nothing about how large the symbol receiving it has grown. That matters for globals that
// name each other in a cycle: none of them calls itself, so the substitution never stops on its
// own, and each round puts the whole cycle into each member again. What accumulates is the renaming
// a substitution leaves behind, which generates no code and so costs nothing by the measure that
// decides whether to inline; only a count of the nodes sees it. That such a program compiles is
// what these tests pin.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// How many renamings each member of the ring carries. A renaming generates no code, so the
    /// measure that decides whether to inline counts such a body as small however many it holds.
    const RENAMINGS_PER_GLOBAL: usize = 2000;

    /// The entry point that calls a ring of globals named `f`, `g` and `h`. The command line
    /// decides the branch, so the call to the ring stands in the program while the run never takes
    /// it: what is under test is that the compiler arrives at a program at all.
    const MAIN_CALLING_THE_RING: &str = r#"
        main : IO ();
        main = (
            let args = *IO::get_args;
            if args.@size > 100 { println $ f(0).to_string } else { println $ "reached" }
        );
        "#;

    /// Builds `source` at `-O max` and runs it, asserting that it reached the branch
    /// `MAIN_CALLING_THE_RING` takes. Without a ceiling on the nodes a symbol may gain, the
    /// rewriting doubles each member of the ring every round until the compiler exhausts its stack
    /// and aborts; with one the build finishes in under a second.
    ///
    /// # Arguments
    /// * `description` - what is being compiled, as a phrase that reads after "compiling".
    fn assert_ring_compiles_and_runs(source: &str, description: &str) {
        let output = build_within_and_run(source, "max", Duration::from_secs(60), description);
        assert_eq!(output, "reached");
    }

    /// Three globals calling each other in a ring, each carrying `RENAMINGS_PER_GLOBAL` renamings,
    /// compile and answer.
    #[test]
    fn test_a_ring_of_globals_carrying_renamings_compiles() {
        let mut source = "module Main;\n\n".to_string();
        for (name, next) in [("f", "g"), ("g", "h"), ("h", "f")] {
            source += &format!(
                "{} : I64 -> I64;\n{} = |x| (\n    let a0 = x;\n",
                name, name
            );
            for i in 1..RENAMINGS_PER_GLOBAL {
                source += &format!("    let a{} = a{};\n", i, i - 1);
            }
            source += &format!("    {}(a{})\n);\n\n", next, RENAMINGS_PER_GLOBAL - 1);
        }
        source += MAIN_CALLING_THE_RING;

        assert_ring_compiles_and_runs(
            &source,
            &format!(
                "a ring of three globals, each carrying {} renamings",
                RENAMINGS_PER_GLOBAL
            ),
        );
    }

    /// The same ring with bodies of one expression apiece, which is the smallest shape that never
    /// stops on its own: each global is small enough to be put where it is called, and none of them
    /// calls itself.
    #[test]
    fn test_a_ring_of_one_expression_globals_compiles() {
        let mut source = r#"
        module Main;

        f : I64 -> I64;
        f = |x| g(x);
        g : I64 -> I64;
        g = |x| h(x);
        h : I64 -> I64;
        h = |x| f(x);
        "#
        .to_string();
        source += MAIN_CALLING_THE_RING;

        assert_ring_compiles_and_runs(&source, "a ring of three globals of one expression apiece");
    }
}
