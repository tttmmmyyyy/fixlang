// Inlining expands a global whose body is another global's name into the definition that chain of
// names arrives at. A chain that runs back into itself arrives nowhere, and the pass has to notice,
// because it reaches its result by rewriting until nothing changes and going around a cycle is a
// change every time.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{fix_build_source_command, wait_within};
    use std::fs::{self, File};
    use std::process::Stdio;
    use std::time::Duration;
    use tempfile::TempDir;

    // Generous next to the second the build takes, and short enough to report a regression as a
    // failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(120);

    /// A program defining a cycle of `length` aliases, reached from `main` down a branch the guard
    /// never takes. Evaluating such a cycle diverges, so `main` names it without running it: what is
    /// under test is that the compiler finishes, and reaching the cycle from `main` is what keeps
    /// dead-symbol elimination from discarding it first.
    fn alias_cycle_source(length: usize) -> String {
        let cycle = (0..length)
            .map(|i| format!("a{} : I64 -> I64;\na{} = a{};\n", i, i, (i + 1) % length))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "module Main;\n\
             \n\
             {cycle}\n\
             guard : I64;\n\
             guard = \"abc\".@size;\n\
             \n\
             main : IO ();\n\
             main = if guard > 100 {{ println $ a0(0).to_string }} else {{ println $ \"reached\" }};\n"
        )
    }

    /// Builds and runs `source` at `opt_level`, failing if the build does not finish within
    /// `TIMEOUT`, and returns what the program printed.
    fn build_and_run_within_timeout(source: &str, opt_level: &str, description: &str) -> String {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let program_path = temp_dir.path().join("program");

        // The compiler's diagnostics go to a file, which the test reads once the child has exited.
        // A pipe left unread that long fills its buffer and blocks the very build being timed.
        let log_path = temp_dir.path().join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(temp_dir.path(), source, opt_level);
        command
            .arg("-o")
            .arg(&program_path)
            .stdout(Stdio::from(log))
            .stderr(Stdio::from(log_for_stderr));
        let mut child = command.spawn().expect("Failed to execute fix build");
        let status = wait_within(&mut child, TIMEOUT, description);
        assert!(
            status.success(),
            "building {} failed: {}\n{}",
            description,
            status,
            fs::read_to_string(&log_path).expect("Failed to read the build log")
        );

        let output = std::process::Command::new(&program_path)
            .output()
            .expect("Failed to run the compiled program");
        assert!(
            output.status.success(),
            "{} exited with {}",
            description,
            output.status
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    /// A cycle of aliases compiles, whatever its length. Inlining runs from `Max`, so `basic` is the
    /// control that the program itself is fine; a cycle whose length is a power of two converges on
    /// its own, so the other lengths are what the check rests on.
    #[test]
    fn test_a_cycle_of_aliases_compiles() {
        for length in [2, 3, 5, 6] {
            for opt_level in ["basic", "max"] {
                let description = format!("a cycle of {} aliases at -O {}", length, opt_level);
                let printed = build_and_run_within_timeout(
                    &alias_cycle_source(length),
                    opt_level,
                    &description,
                );
                assert_eq!(
                    printed, "reached",
                    "{} printed the wrong value",
                    description
                );
            }
        }
    }

    /// A chain of aliases that does end still arrives at the definition it names.
    #[test]
    fn test_a_chain_of_aliases_reaches_its_definition() {
        let source = r#"
        module Main;

        real : I64 -> I64;
        real = |x| x + 1;

        x0 : I64 -> I64;
        x0 = x1;
        x1 : I64 -> I64;
        x1 = x2;
        x2 : I64 -> I64;
        x2 = real;

        main : IO ();
        main = println $ x0(41).to_string;
        "#;
        for opt_level in ["none", "basic", "max"] {
            let description = format!("a chain of aliases at -O {}", opt_level);
            let printed = build_and_run_within_timeout(source, opt_level, &description);
            assert_eq!(printed, "42", "{} printed the wrong value", description);
        }
    }
}
