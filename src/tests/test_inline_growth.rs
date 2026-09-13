// A symbol grows by what is substituted into it, and nothing in the cost of a body says how large
// the symbol receiving it has become. Globals that name each other in a cycle are what needs it:
// none of them calls itself, so the substitution never stops on its own, and each round puts the
// whole cycle into each member again. What accumulates is the renaming a substitution leaves
// behind, which generates no code and so costs nothing by the measure that decides whether to
// inline; only a count of the nodes themselves sees it.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::build_within_and_run;
    use std::time::Duration;

    /// Three globals calling each other in a ring, each carrying two thousand renamings, compile
    /// and answer. Without a ceiling on the nodes a symbol may reach, the rewriting doubles each of
    /// them every round until the compiler exhausts its stack and aborts; with one it builds in
    /// under a second.
    ///
    /// The ring is reached only through a condition the argument count decides, so the program
    /// never calls it; what is under test is that the compiler arrives at a program at all.
    #[test]
    fn test_a_ring_of_globals_carrying_renamings_compiles() {
        let mut source = "module Main;\n\n".to_string();
        for (name, next) in [("f", "g"), ("g", "h"), ("h", "f")] {
            source += &format!(
                "{} : I64 -> I64;\n{} = |x| (\n    let a0 = x;\n",
                name, name
            );
            for i in 1..2000 {
                source += &format!("    let a{} = a{};\n", i, i - 1);
            }
            source += &format!("    {}(a1999)\n);\n\n", next);
        }
        source += r#"
        main : IO ();
        main = (
            // The command line decides the branch, so the call to the ring stands in the program
            // and the run never takes it.
            let args = *IO::get_args;
            if args.@size > 100 { println $ f(0).to_string } else { println $ "reached" }
        );
        "#;

        let output = build_within_and_run(
            &source,
            "max",
            Duration::from_secs(60),
            "a ring of three globals, each carrying two thousand renamings",
        );
        assert_eq!(output.trim(), "reached");
    }

    /// The same ring with bodies of one expression apiece, which is the smallest shape that never
    /// stops on its own: each global is small enough to be put where it is called, and none of them
    /// calls itself.
    #[test]
    fn test_a_ring_of_one_line_globals_compiles() {
        let source = r#"
        module Main;

        f : I64 -> I64;
        f = |x| g(x);
        g : I64 -> I64;
        g = |x| h(x);
        h : I64 -> I64;
        h = |x| f(x);

        main : IO ();
        main = (
            // The command line decides the branch, so the call to the ring stands in the program
            // and the run never takes it.
            let args = *IO::get_args;
            if args.@size > 100 { println $ f(0).to_string } else { println $ "reached" }
        );
        "#;

        let output = build_within_and_run(
            &source,
            "max",
            Duration::from_secs(60),
            "a ring of three globals of one expression apiece",
        );
        assert_eq!(output.trim(), "reached");
    }
}
