//! `fix build` reports a failed build through its exit status, so that a caller which chains on
//! success — a shell `&&`, a `make` rule, a CI step — stops rather than carrying on with an output
//! file the build never produced.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::fix_build_source_command;
    use tempfile::TempDir;

    const SOURCE: &str = r#"module Main;

main : IO ();
main = println("hi");
"#;

    /// Linking against a library that does not exist fails the build, and the same build without
    /// that library succeeds — so the failing case is a link failure rather than a program the
    /// compiler rejects for its own reasons.
    #[test]
    fn test_build_fails_when_linking_fails() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let out_path = temp_dir.path().join("out");

        let control = fix_build_source_command(temp_dir.path(), SOURCE, "none")
            .arg("-o")
            .arg(&out_path)
            .output()
            .expect("Failed to execute fix build");
        assert!(
            control.status.success() && out_path.exists(),
            "the control build failed, so the test below would prove nothing:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&control.stdout),
            String::from_utf8_lossy(&control.stderr)
        );
        std::fs::remove_file(&out_path).expect("Failed to remove the control build's output");

        let output = fix_build_source_command(temp_dir.path(), SOURCE, "none")
            .arg("-o")
            .arg(&out_path)
            .arg("-d")
            .arg("no_such_library_for_this_test")
            .output()
            .expect("Failed to execute fix build");
        assert!(
            !output.status.success(),
            "linking against a library that does not exist reported success, and left {}",
            if out_path.exists() {
                "an output file"
            } else {
                "no output file"
            }
        );
    }

    /// A program using a type whose unboxed fields lead back to itself is rejected, and the
    /// rejection reaches a caller of `fix build` the way any other does: the diagnostic on stderr,
    /// nothing after it, and a failing exit status.
    #[test]
    fn test_build_fails_with_a_status_when_a_type_has_no_size() {
        const CIRCULAR_SOURCE: &str = r#"module Main;

type A = unbox struct { b : B, n : I64 };
type B = unbox struct { a : A, m : I64 };

depth : A -> I64;
depth = |x| x.@n;

main : IO ();
main = println(depth(undefined("no value")).to_string);
"#;
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let output = fix_build_source_command(temp_dir.path(), CIRCULAR_SOURCE, "none")
            .arg("-o")
            .arg(temp_dir.path().join("out"))
            .output()
            .expect("Failed to execute fix build");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("its unboxed fields reach `Main::A` itself"),
            "the build did not report the type that has no layout:\nstderr: {}",
            stderr
        );
        assert!(
            output.status.code().is_some(),
            "the build was ended by a signal ({}) instead of an exit status:\nstderr: {}",
            output.status,
            stderr
        );
        assert!(
            !stderr.contains("(unknown error)"),
            "the diagnostic was followed by a second, contentless report:\nstderr: {}",
            stderr
        );
        assert!(
            !output.status.success(),
            "the build reported success for a program it could not lay out"
        );
    }
}
