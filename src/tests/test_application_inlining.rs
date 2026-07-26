// Application inlining moves an application into the subexpressions of the function it applies.
// An argument that is already a variable is moved in as it is, which is what keeps the cost linear
// in the number of arguments pushed through a chain of `let`s — the shape uncurrying's eta
// expansion builds for a function of many parameters. A variable that the target binds is bound to
// a fresh name first, so moving it in leaves it referring to the same value.

#[cfg(test)]
mod tests {
    use crate::configuration::Configuration;
    use crate::tests::test_util::{fix_build_source_command, test_source, wait_within};
    use std::fs::{self, File};
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tempfile::TempDir;

    /// The argument `x` of `(let x = ..; ..)(x)` denotes the outer `x` after the application is
    /// moved inside the `let`, where the name `x` denotes the bound lambda.
    #[test]
    fn test_variable_argument_pushed_into_a_let_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let x = 10;
            let y = (let x = |v : I64| v + 1; x)(x);
            assert_eq(|_|"the argument denotes the lambda bound by the inner `let`", y, 11);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    /// The argument `a` of `(match o { some(a) => a, .. })(a)` denotes the outer `a` after the
    /// application is moved into the arms, where the name `a` denotes what the pattern binds.
    #[test]
    fn test_variable_argument_pushed_into_a_match_arm_keeps_its_meaning() {
        let source = r#"
        module Main;

        main : IO ();
        main = (
            let a = 20;
            let o : Option (I64 -> I64) = Option::some(|w| w + 1);
            let z = (match o { some(a) => a, none(_) => |w| w + 100 })(a);
            assert_eq(|_|"the argument denotes the function bound by the arm", z, 21);;
            pure()
        );
        "#;
        test_source(source, Configuration::develop_mode());
    }

    // Compiling this arity takes a few seconds while the cost is linear in it, and is out of reach
    // while the cost doubles per parameter.
    const ARITY: usize = 25;

    // Generous next to the few seconds the build takes, and short enough to report a regression as
    // a failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(180);

    /// Builds and runs a global function of `ARITY` parameters, failing if the build does not
    /// finish within `TIMEOUT`. The body weights each parameter by its position, so the result also
    /// pins the order the arguments arrive in.
    ///
    /// The time bound catches the doubling at `Basic` and above, where uncurrying eta-expands a
    /// global into a function pointer per arity. At `None` uncurrying is off and the bound is slack,
    /// while building and running a function of this many parameters is a check of its own.
    #[test]
    fn test_many_parameter_function_compiles_in_reasonable_time() {
        let params = (0..ARITY)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let signature = vec!["I64"; ARITY + 1].join(" -> ");
        let body = (0..ARITY)
            .map(|i| format!("x{} * {}", i, i))
            .collect::<Vec<_>>()
            .join(" + ");
        let args = (0..ARITY)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let expected: usize = (0..ARITY).map(|i| i * i).sum();
        let source = format!(
            "module Main;\n\
             \n\
             g : {signature};\n\
             g = |{params}| {body};\n\
             \n\
             main : IO ();\n\
             main = println(g({args}).to_string);\n"
        );

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let program_path = temp_dir.path().join("many_params");

        // The compiler writes its diagnostics to a file rather than a pipe, which nothing reads
        // until the child exits — long enough to fill a pipe's buffer and block the very build
        // being timed.
        let log_path = temp_dir.path().join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(temp_dir.path(), &source, "basic");
        command
            .arg("-o")
            .arg(&program_path)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_for_stderr));
        let mut child = command.spawn().expect("Failed to execute fix build");
        let status = wait_within(
            &mut child,
            TIMEOUT,
            &format!("compiling a {}-parameter function", ARITY),
        );
        assert!(
            status.success(),
            "compiling a {}-parameter function failed: {}\n{}",
            ARITY,
            status,
            fs::read_to_string(&log_path).unwrap_or_default()
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
            expected.to_string(),
            "the {}-parameter function returned a wrong value",
            ARITY
        );
    }
}
