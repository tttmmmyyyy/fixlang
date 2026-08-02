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
}
