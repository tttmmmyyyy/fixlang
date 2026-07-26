// The compiler compiles a function of many parameters in time proportional to
// the parameter count. Uncurrying eta-expands each global into a function
// pointer of every arity up to its own, and the application inlining that
// drives the expansion pushes each argument through the `let`s the parameters
// introduce; re-binding an argument at every level would double the chain per
// parameter, so the intermediate expression — and the recursion of every pass
// that walks it — would grow as `2^arity`.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_command;
    use std::fs;
    use std::process::Stdio;
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    // Compiling this arity takes a few seconds while the cost is linear in it, and
    // is out of reach while the cost doubles per parameter.
    const ARITY: usize = 25;

    // Generous next to the few seconds the build takes, and short enough to report
    // a regression as a failure instead of occupying the machine.
    const TIMEOUT: Duration = Duration::from_secs(180);

    /// Builds a global function of `ARITY` parameters and fails if the build does not finish
    /// within `TIMEOUT`, catching a compilation cost that grows with the parameter count faster
    /// than linearly.
    #[test]
    fn test_many_argument_function_compiles_in_reasonable_time() {
        let params = (0..ARITY)
            .map(|i| format!("x{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let signature = vec!["F32"; ARITY + 1].join(" -> ");
        let args = (0..ARITY)
            .map(|i| format!("{}.0_F32", i))
            .collect::<Vec<_>>()
            .join(", ");
        let source = format!(
            "module Main;\n\
             \n\
             g : {signature};\n\
             g = |{params}| x0;\n\
             \n\
             main : IO ();\n\
             main = println(g({args}).to_string);\n"
        );

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_path = temp_dir.path().join("many_args.fix");
        fs::write(&src_path, source).expect("Failed to write source file");

        let start = Instant::now();
        let mut child = fix_command()
            .arg("build")
            .arg("--file")
            .arg(&src_path)
            .arg("-O")
            .arg("basic")
            .current_dir(temp_dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to execute fix build");

        loop {
            match child.try_wait().expect("Failed to wait for fix build") {
                Some(status) => {
                    assert!(
                        status.success(),
                        "compiling a {}-argument function failed: {}",
                        ARITY,
                        status
                    );
                    break;
                }
                None => {
                    if start.elapsed() > TIMEOUT {
                        let _ = child.kill();
                        let _ = child.wait();
                        panic!(
                            "compiling a {}-argument function did not finish within {} seconds",
                            ARITY,
                            TIMEOUT.as_secs()
                        );
                    }
                    sleep(Duration::from_millis(100));
                }
            }
        }
    }
}
