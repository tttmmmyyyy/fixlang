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
}
