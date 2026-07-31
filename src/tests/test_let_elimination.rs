// Let-elimination replaces `let x = {e0}; {e1}` with `{e1}[x := {e0}]` once it has inspected the
// occurrences of `x` in `{e1}`. Which occurrences belong to that binding is what these tests pin:
// an occurrence under a binder that gives the name to something else belongs to that binder, and an
// occurrence the inspection reaches only through a lambda or a match arm still keeps the binding
// alive.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::{fix_build_source_command, test_source, wait_within};
    use std::fs::{self, File};
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tempfile::TempDir;

    /// The only occurrence of `x` is inside a lambda, which captures the binding.
    #[test]
    fn test_name_used_only_inside_a_lambda_keeps_its_binding() {
        let source = r#"
        module Main;

        used_in_lambda : I64 -> I64;
        used_in_lambda = |a| (
            let x = a + 1;
            let f = |v : I64| v + x;
            f(10)
        );

        main : IO ();
        main = (
            assert_eq(|_|"the lambda adds what `x` is bound to", used_in_lambda(5), 16);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The only occurrence of `x` is inside the value expression of a match arm.
    #[test]
    fn test_name_used_only_inside_a_match_arm_keeps_its_binding() {
        let source = r#"
        module Main;

        used_in_match_arm : I64 -> I64;
        used_in_match_arm = |a| (
            let x = a + 1;
            match Option::some(3) {
                some(v) => v + x,
                none(_) => 0
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the arm adds what `x` is bound to", used_in_match_arm(5), 9);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A lambda parameter of the same name gives `x` inside the lambda to the parameter.
    #[test]
    fn test_name_rebound_by_a_lambda_parameter_denotes_the_parameter() {
        let source = r#"
        module Main;

        rebound_by_lambda_param : I64 -> I64;
        rebound_by_lambda_param = |a| (
            let x = a + 1;
            let g = |x : I64| x * 2;
            g(10) + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the lambda is its parameter", rebound_by_lambda_param(5), 26);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// An inner `let` of the same name gives `x` in its value expression to the inner binding.
    #[test]
    fn test_name_rebound_by_an_inner_let_denotes_the_inner_binding() {
        let source = r#"
        module Main;

        rebound_by_inner_let : I64 -> I64;
        rebound_by_inner_let = |a| (
            let x = a + 1;
            let y = (let x = 100; x * 2);
            y + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the inner `let` is what that `let` binds", rebound_by_inner_let(5), 206);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The bound expression of a `let` is evaluated in the enclosing scope, so it reaches the outer
    /// binding even when the pattern gives the same name to what it binds.
    #[test]
    fn test_name_read_by_the_bound_expression_of_a_rebinding_let_keeps_its_binding() {
        let source = r#"
        module Main;

        rebinding_let_reads_the_outer_name : I64 -> I64;
        rebinding_let_reads_the_outer_name = |a| (
            let x = a + 1;
            let x = x * 2;
            x + x
        );

        main : IO ();
        main = (
            assert_eq(|_|"the bound expression doubles what the outer `x` is bound to", rebinding_let_reads_the_outer_name(5), 24);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The expression a match inspects is evaluated in the enclosing scope, so it reaches the outer
    /// binding even when every arm gives the same name to what its pattern binds.
    #[test]
    fn test_name_read_by_a_match_condition_keeps_its_binding() {
        let source = r#"
        module Main;

        match_condition_reads_the_outer_name : I64 -> I64;
        match_condition_reads_the_outer_name = |a| (
            let x = a + 1;
            match Option::some(x) {
                some(x) => x * 3,
                none(_) => 0
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"the match inspects what the outer `x` is bound to", match_condition_reads_the_outer_name(5), 18);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// A match pattern of the same name gives `x` in that arm to the pattern, while the arm whose
    /// pattern binds another name reaches the outer binding.
    #[test]
    fn test_name_rebound_by_a_match_pattern_denotes_the_pattern() {
        let source = r#"
        module Main;

        rebound_by_match_pattern : I64 -> I64;
        rebound_by_match_pattern = |a| (
            let x = a + 1;
            match Option::some(7) {
                some(x) => x * 3,
                none(_) => x
            }
        );

        main : IO ();
        main = (
            assert_eq(|_|"`x` in the arm is what the pattern binds", rebound_by_match_pattern(5), 21);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    // A chain this long builds in a few seconds while the inspection of a binding is proportional
    // to where its name is read, and in several minutes while it walks the whole body of every
    // binding.
    const CHAIN_LENGTH: usize = 2400;

    // Generous next to the few seconds the build takes, with room for a machine several times
    // slower running the rest of the suite beside it, and well short of the minutes the build takes
    // once the inspection walks whole bodies again.
    const TIMEOUT: Duration = Duration::from_secs(120);

    /// Builds and runs a global whose body is a chain of `CHAIN_LENGTH` `let`s, failing if the
    /// build does not finish within `TIMEOUT`. `-O max` is the level that inspects the chain twice:
    /// once for the local inlining that eliminates `let`s, and again for the eta expansion that
    /// uncurrying runs the same pass on.
    ///
    /// Each binding calls a global of one parameter rather than writing the arithmetic out, which
    /// keeps type checking a small part of the budget: an operator resolved through a trait carries
    /// a predicate per occurrence, and a chain of those spends most of the build before the
    /// inspection under test even runs.
    #[test]
    fn test_long_let_chain_compiles_in_reasonable_time() {
        let mut body = String::from("    let a0 = succ(x);\n");
        for i in 1..CHAIN_LENGTH {
            body.push_str(&format!("    let a{} = succ(a{});\n", i, i - 1));
        }
        body.push_str(&format!("    a{}\n", CHAIN_LENGTH - 1));
        let source = format!(
            "module Main;\n\
             \n\
             succ : I64 -> I64;\n\
             succ = |x| x + 1;\n\
             \n\
             let_chain : I64 -> I64;\n\
             let_chain = |x| (\n{body});\n\
             \n\
             main : IO ();\n\
             main = println(let_chain(0).to_string);\n"
        );

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let program_path = temp_dir.path().join("long_let_chain");

        // The compiler's diagnostics go to a file, which the test reads once the child has exited.
        // A pipe left unread that long fills its buffer and blocks the very build being timed.
        let log_path = temp_dir.path().join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(temp_dir.path(), &source, "max");
        command
            .arg("-o")
            .arg(&program_path)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_for_stderr));
        let mut child = command.spawn().expect("Failed to execute fix build");
        let status = wait_within(
            &mut child,
            TIMEOUT,
            &format!("compiling a chain of {} `let`s", CHAIN_LENGTH),
        );
        assert!(
            status.success(),
            "compiling a chain of {} `let`s failed: {}\n{}",
            CHAIN_LENGTH,
            status,
            fs::read_to_string(&log_path).expect("Failed to read the build log")
        );

        let output = Command::new(&program_path)
            .output()
            .expect("Failed to run the compiled program");
        assert!(
            output.status.success(),
            "the compiled program exited with {}",
            output.status
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            CHAIN_LENGTH.to_string(),
            "the chain of {} `let`s returned a wrong value",
            CHAIN_LENGTH
        );
    }
}
