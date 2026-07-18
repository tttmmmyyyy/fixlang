// Integration tests for the RC IR term simplifier, checked through the `--emit-rc-ir` dump.
// A read loop over `range(0, size).fold` lowers to a specialized fold driver whose loop-carried state
// is the `Option` that `range`'s `advance` builds and `fold` immediately matches. The simplifier
// cancels that union (case-of-case + case-of-known-constructor), so the driver keeps only the plain
// `RangeIterator` two-scalar state and no union construction — the property these tests assert.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_simplify/cases");
        path
    }

    // Copy the test cases into a fresh temporary directory so parallel test runs do not conflict, and
    // return the directory of the named case project.
    fn setup_test_env(case: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &dst).expect("Failed to copy test cases");
        let project_dir = dst.join(case);
        (temp_dir, project_dir)
    }

    // Build the case project at `max` (where the simplifier runs) with `--emit-rc-ir all`, returning
    // the dumped RC IR of every module. The `range.fold` driver is a specialized `Std::Iterator`
    // symbol, so the whole-program dump is needed to see it. Also leaves a runnable executable.
    fn emit_all_rc_ir(project_dir: &Path) -> String {
        let output = fix_command()
            .arg("build")
            .arg("--emit-rc-ir")
            .arg("all")
            .env("FIX_MAX_OPT_LEVEL", "max")
            .current_dir(project_dir)
            .output()
            .expect("Failed to execute fix build --emit-rc-ir");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build --emit-rc-ir failed");
        }

        let dump_path = project_dir.join(".fixlang/rc_ir.post.txt");
        std::fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e))
    }

    // The body of each `fn` block in the dump whose header line contains all of `needles`.
    fn fn_bodies_matching<'a>(dump: &'a str, needles: &[&str]) -> Vec<String> {
        let mut bodies = Vec::new();
        let mut current: Option<String> = None;
        for line in dump.lines() {
            if line.starts_with("fn ") {
                if let Some(body) = current.take() {
                    bodies.push(body);
                }
                if needles.iter().all(|n| line.contains(n)) {
                    current = Some(String::new());
                }
            }
            if let Some(body) = current.as_mut() {
                body.push_str(line);
                body.push('\n');
            }
        }
        if let Some(body) = current.take() {
            bodies.push(body);
        }
        bodies
    }

    #[test]
    fn test_range_fold_union_removed() {
        let (_temp_dir, project_dir) = setup_test_env("read_fold");
        let dump = emit_all_rc_ir(&project_dir);

        // The `range.fold` drivers (own and borrow version) — identified by the `RangeIterator` loop
        // state in their signature.
        let drivers = fn_bodies_matching(&dump, &["Iterator::fold", "RangeIterator"]);
        assert!(
            !drivers.is_empty(),
            "no `range.fold` driver (an `Iterator::fold` over a `RangeIterator`) in the RC IR dump:\n{}",
            dump
        );
        for driver in &drivers {
            // The simplifier cancelled the `Option` union: the driver builds no union, so its
            // loop-carried state is just the plain `RangeIterator` named in its signature.
            assert!(
                !driver.contains("union_"),
                "the `range.fold` driver still builds a union — the simplifier did not cancel it:\n{}",
                driver
            );
        }

        // The program still computes 0 + 1 + .. + 99 = 4950.
        let run = std::process::Command::new(project_dir.join("a.out"))
            .output()
            .expect("failed to run the built executable");
        assert!(run.status.success(), "the built executable did not run cleanly");
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "4950");
    }
}
