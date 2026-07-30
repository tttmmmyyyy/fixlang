// Inlining reaches its result by rewriting the program until nothing changes. Globals that name
// each other in a cycle never get there — the rewriting goes around the cycle instead — so the
// number of rewrites is bounded. These tests hold that bound between the two things it sits
// between: high enough that a program of any ordinary depth still reaches its result, and low
// enough that a cycle is cut before what it builds does damage.

#[cfg(test)]
mod tests {
    use crate::tests::test_util::{fix_build_source_command, wait_within};
    use std::fs::{self, File};
    use std::path::Path;
    use std::process::Stdio;
    use std::time::Duration;
    use tempfile::TempDir;

    // Generous next to the seconds these builds take, and short enough to report a regression as a
    // failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(120);

    /// What a build produced: what the program printed, and the directory it was built in, which
    /// holds the symbol dumps when the build was asked for them.
    struct BuildResult {
        printed: String,
        _dir: TempDir,
        dir_path: std::path::PathBuf,
    }

    /// Builds and runs `source` at `opt_level`, failing if the build does not finish within
    /// `TIMEOUT`.
    fn build_and_run_within_timeout(
        source: &str,
        opt_level: &str,
        description: &str,
        emit_symbols: bool,
    ) -> BuildResult {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir_path = temp_dir.path().to_path_buf();
        let program_path = dir_path.join("program");

        // The compiler's diagnostics go to a file, which the test reads once the child has exited.
        // A pipe left unread that long fills its buffer and blocks the very build being timed.
        let log_path = dir_path.join("build.log");
        let log = File::create(&log_path).expect("Failed to create the build log");
        let log_for_stderr = log
            .try_clone()
            .expect("Failed to clone the build log handle");

        let mut command = fix_build_source_command(&dir_path, source, opt_level);
        command.arg("-o").arg(&program_path);
        if emit_symbols {
            command.arg("--emit-symbols");
        }
        command
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
        BuildResult {
            printed: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            _dir: temp_dir,
            dir_path,
        }
    }

    /// The symbols of the program as it went to code generation, from the dumps `--emit-symbols`
    /// left in `dir`.
    fn final_symbols(dir: &Path) -> String {
        let dumps =
            fs::read_dir(dir.join(".fixlang")).expect("Failed to read the .fixlang directory");
        for entry in dumps {
            let path = entry.expect("Failed to read a directory entry").path();
            if path.to_string_lossy().ends_with("final.symbols.fix") {
                return fs::read_to_string(&path).expect("Failed to read the symbol dump");
            }
        }
        panic!("The build left no final symbol dump in {}", dir.display());
    }

    /// A program defining a cycle of `length` globals, each naming the next, reached from `main`
    /// down a branch the guard never takes. Evaluating such a cycle diverges, so `main` names it
    /// without running it: what is under test is that the compiler finishes, and naming it from
    /// `main` is what keeps dead-symbol elimination from discarding it first.
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

    /// A cycle of globals that name each other compiles, whatever its length. Inlining runs from
    /// `Max`, so `basic` is the control that the program itself is fine; a cycle whose length is a
    /// power of two settles on its own, so the other lengths are what the check rests on.
    #[test]
    fn test_a_cycle_of_aliases_compiles() {
        for length in [2, 3, 5, 6] {
            for opt_level in ["basic", "max"] {
                let description = format!("a cycle of {} aliases at -O {}", length, opt_level);
                let result = build_and_run_within_timeout(
                    &alias_cycle_source(length),
                    opt_level,
                    &description,
                    false,
                );
                assert_eq!(
                    result.printed, "reached",
                    "{} printed the wrong value",
                    description
                );
            }
        }
    }

    /// A cycle of functions that call each other compiles. Each round inlines one into the next, so
    /// their bodies double while the rewriting goes around: this is the shape that makes the bound
    /// worth keeping low, and it is the one that used to exhaust the compiler's stack.
    #[test]
    fn test_a_cycle_of_calling_functions_compiles() {
        let source = r#"
        module Main;

        f : I64 -> I64;
        f = |x| g(x);
        g : I64 -> I64;
        g = |x| h(x);
        h : I64 -> I64;
        h = |x| f(x);

        guard : I64;
        guard = "abc".@size;

        main : IO ();
        main = if guard > 100 { println $ f(0).to_string } else { println $ "reached" };
        "#;
        let result =
            build_and_run_within_timeout(source, "max", "a cycle of three functions", false);
        assert_eq!(result.printed, "reached");
    }

    /// A chain of globals naming each other arrives at the definition at its end.
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
            let result = build_and_run_within_timeout(source, opt_level, &description, false);
            assert_eq!(
                result.printed, "42",
                "{} printed the wrong value",
                description
            );
        }
    }

    // A chain this long takes eight rounds to follow, since a round doubles how far each name has
    // been followed. It is what holds the bound up from below: reaching the end of this chain is
    // what a program of an ordinary depth needs.
    const DEEP_CHAIN: usize = 50;

    /// A deep chain of globals naming each other is still followed to its end, so that none of them
    /// survives into the program that goes to code generation.
    #[test]
    fn test_a_deep_chain_of_aliases_is_followed_to_its_end() {
        let chain = (0..DEEP_CHAIN)
            .map(|i| {
                let target = if i + 1 < DEEP_CHAIN {
                    format!("x{}", i + 1)
                } else {
                    "real".to_string()
                };
                format!("x{} : I64 -> I64;\nx{} = {};\n", i, i, target)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let source = format!(
            "module Main;\n\
             \n\
             real : I64 -> I64;\n\
             real = |x| x + 1;\n\
             \n\
             {chain}\n\
             main : IO ();\n\
             main = println $ x0(41).to_string;\n"
        );
        let description = format!("a chain of {} aliases", DEEP_CHAIN);
        let result = build_and_run_within_timeout(&source, "max", &description, true);
        assert_eq!(
            result.printed, "42",
            "{} printed the wrong value",
            description
        );

        let symbols = final_symbols(&result.dir_path);
        let left = (0..DEEP_CHAIN)
            .filter(|i| symbols.contains(&format!("Main::x{}#", i)))
            .count();
        assert_eq!(
            left, 0,
            "{} of the {} names in the chain reached code generation, so the chain was followed only part of the way",
            left, DEEP_CHAIN
        );
    }
}
