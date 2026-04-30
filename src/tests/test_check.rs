#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_check/cases");
        path
    }

    fn setup_test_env(case_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_test_cases_dir().join(case_name);
        let dst = temp_dir.path().join(case_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        (temp_dir, dst)
    }

    #[test]
    fn test_check_valid_project() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("valid_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        if !output.status.success() {
            eprintln!("fix check failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix check should succeed on a valid project");
        }
    }

    #[test]
    fn test_check_type_error() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("type_error_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            !output.status.success(),
            "fix check should fail on a project with type errors"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Type mismatch"),
            "Error message should mention type mismatch, got: {}",
            stderr
        );
    }

    #[test]
    fn test_check_detects_type_error_in_test_code() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("test_type_error_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            !output.status.success(),
            "fix check should fail when test code has type errors"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Type mismatch"),
            "Error message should mention type mismatch in test code, got: {}",
            stderr
        );
    }

    /// `fix check` should surface `DEPRECATED[...]` warnings to stderr
    /// even though the project compiles successfully.
    #[test]
    fn test_check_emits_deprecation_warning() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_warning_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            output.status.success(),
            "fix check should succeed (warning-only): stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("warning")
                && stderr.contains("old_func")
                && stderr.contains("Use `new_func` instead."),
            "Expected deprecation warning in stderr, got: {}",
            stderr
        );
    }

    /// `--deny-deprecated` promotes the warning into a hard error and the
    /// build fails.
    #[test]
    fn test_build_deny_deprecated_promotes_to_error() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_warning_project");

        let output = Command::new("fix")
            .arg("build")
            .arg("--deny-deprecated")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        assert!(
            !output.status.success(),
            "fix build --deny-deprecated should fail: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("error")
                && stderr.contains("old_func")
                && stderr.contains("Use `new_func` instead."),
            "Expected deprecation error in stderr, got: {}",
            stderr
        );
    }

    /// `--allow-deprecated` suppresses the warning and produces a clean
    /// build.
    #[test]
    fn test_build_allow_deprecated_suppresses_warning() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_warning_project");

        let output = Command::new("fix")
            .arg("build")
            .arg("--allow-deprecated")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        assert!(
            output.status.success(),
            "fix build --allow-deprecated should succeed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("Use `new_func` instead."),
            "Expected no deprecation warning in stderr, got: {}",
            stderr
        );
    }

    /// A `DEPRECATED` trait member should warn when reached through a
    /// concrete impl: marking the trait member must propagate to each
    /// impl's derived `GlobalValue`.
    #[test]
    fn test_check_emits_trait_member_deprecation_warning() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_trait_warning_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            output.status.success(),
            "fix check should succeed (warning-only): stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("warning")
                && stderr.contains("old_greet")
                && stderr.contains("Use `greet` instead."),
            "Expected trait-member deprecation warning in stderr, got: {}",
            stderr
        );
    }

    /// Deprecation diagnostics must scope to the *root* project. When a
    /// dependency internally calls one of its own deprecated symbols, that
    /// is the dependency author's problem, not the consumer's — `fix check`
    /// on the root project must succeed and emit no deprecation warning.
    /// Mirrors how rustc/swiftc/javac/etc. only flag deprecated uses in
    /// the unit currently being compiled.
    #[test]
    fn test_check_does_not_warn_on_dependency_internal_deprecated_use() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_in_dependency_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            output.status.success(),
            "fix check should succeed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        // The resolver prints the dep's project name (`dep-with-deprecated`)
        // to stderr, so we can't search for the literal word "deprecated"
        // here — assert on the symbol name and message text instead.
        assert!(
            !stderr.contains("old_helper") && !stderr.contains("Use `new_helper` instead."),
            "Did not expect a deprecation warning for a use that lives \
             inside a dependency, got: {}",
            stderr
        );
    }

    /// "Deprecated context" rule: a deprecated helper calling another
    /// deprecated helper should not produce a warning for that internal
    /// call. Only the use from non-deprecated code (here, `main`) warns.
    #[test]
    fn test_check_deprecated_context_suppresses_inner_warning() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("deprecated_context_project");

        let output = Command::new("fix")
            .arg("check")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix check");

        assert!(
            output.status.success(),
            "fix check should succeed (warning-only): stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        // `main` uses `old_a`: that call must warn.
        assert!(
            stderr.contains("Use `new_a` instead."),
            "Expected warning for `old_a` from main, got: {}",
            stderr
        );
        // `old_a`'s body uses `old_b`, but `old_a` is itself deprecated,
        // so the inner call must NOT warn.
        assert!(
            !stderr.contains("Use `new_b` instead."),
            "Inner call from `old_a` to `old_b` should be suppressed, got: {}",
            stderr
        );
    }
}
