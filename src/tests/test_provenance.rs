// Integration tests for the RC IR provenance analysis, checked through the `--emit-rc-ir` dump.
// The dump annotates each variable binding with the provenance the analysis computed, so a small
// program with named `let`s lets us assert the analysis end to end: allocators produce `fresh`
// values, reading a boxed element out of a boxed container is `dyn`, and constructing an unboxed
// tuple carries each component's provenance through.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, fix_command};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_provenance/cases");
        path
    }

    // Copy the test cases into a fresh temporary directory so parallel test runs do not conflict.
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dst = temp_dir.path().to_path_buf();
        copy_dir_recursive(&get_test_cases_dir(), &dst).expect("Failed to copy test cases");
        let project_dir = dst.join("basic");
        (temp_dir, project_dir)
    }

    /// Assert that the binding named `source_name` (its `(as ...)` annotation) is annotated with the
    /// given provenance in the dump.
    fn assert_binding_prov(dump: &str, source_name: &str, expected_prov: &str) {
        let marker = format!("(as {})", source_name);
        let line = dump
            .lines()
            .find(|l| l.contains(&marker))
            .unwrap_or_else(|| {
                panic!(
                    "no binding `(as {})` in the RC IR dump:\n{}",
                    source_name, dump
                )
            });
        assert!(
            line.contains(expected_prov),
            "binding `(as {})` should have provenance `{}`, but its line is:\n{}",
            source_name,
            expected_prov,
            line
        );
    }

    #[test]
    fn test_provenance_dump_basic() {
        let (_temp_dir, project_dir) = setup_test_env();

        let output = fix_command()
            .arg("build")
            .arg("--emit-rc-ir")
            .arg("Main")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build --emit-rc-ir");

        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build --emit-rc-ir failed");
        }

        let dump_path = project_dir.join(".fixlang/rc_ir.Main.txt");
        let dump = std::fs::read_to_string(&dump_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", dump_path.display(), e));

        // `Array::fill` and an array literal produce a fresh array.
        assert_binding_prov(&dump, "arr", "[fresh]");
        assert_binding_prov(&dump, "strs", "[fresh]");
        // Reading a boxed element out of a boxed container yields an unknown value.
        assert_binding_prov(&dump, "s0", "[dyn]");
        // Constructing an unboxed tuple carries each component's provenance through: `arr` is fresh,
        // `s0` is dyn.
        assert_binding_prov(&dump, "pair", "[(fresh, dyn)]");
    }
}
