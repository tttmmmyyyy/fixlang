// Integration tests for git ref (rev/tag) pinning in dependencies.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use crate::LOCK_FILE_PATH;
    use std::{fs, path::PathBuf, process::Command};
    use tempfile::TempDir;

    // Get the path to the git_ref_tests directory.
    fn get_git_ref_test_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_dependencies/cases/git_ref_tests");
        path
    }

    // Create a temporary test environment with copied project files for a specific test case.
    fn setup_git_ref_test_env(case_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src = get_git_ref_test_dir().join(case_name);
        let dst = temp_dir.path().join(case_name);
        copy_dir_recursive(&src, &dst).expect("Failed to copy test case");
        (temp_dir, dst)
    }

    // Clean up lock files and build artifacts before running test.
    fn cleanup_test_project(project_dir: &PathBuf) {
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_PATH));
        let _ = fs::remove_dir_all(project_dir.join(".fix"));
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(project_dir)
            .output();
    }

    // Test 1: rev pinning builds successfully.
    #[test]
    fn test_git_rev_basic() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_basic");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("7602fba"),
            "Lock file should contain the pinned rev"
        );
        assert!(
            lock_content.contains("version = \"1.1.0\""),
            "Lock file should show version 1.1.0"
        );
    }

    // Test 2: tag pinning builds successfully.
    #[test]
    fn test_git_tag_basic() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("tag_basic");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("6b1c381"),
            "Lock file should contain the rev for tag v1.0.0"
        );
        assert!(
            lock_content.contains("version = \"1.0.0\""),
            "Lock file should show version 1.0.0"
        );
    }

    // Test 3: rev + version (version requirement satisfied).
    #[test]
    fn test_git_rev_with_version_ok() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_with_version_ok");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test 4: rev + version (version requirement NOT satisfied → error).
    #[test]
    fn test_git_rev_with_version_fail() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_with_version_fail");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to version mismatch"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("not satisfied"),
            "Error should mention version not satisfied: {}",
            stderr
        );
    }

    // Test 5: rev and tag both specified → validation error.
    #[test]
    fn test_git_rev_and_tag_conflict() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("rev_and_tag_conflict");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to rev+tag conflict"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Only one of"),
            "Error should mention mutual exclusion: {}",
            stderr
        );
    }

    // Test 6: fix deps update does not change tag-pinned dependency.
    #[test]
    fn test_git_tag_update_stable() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("tag_update_stable");
        cleanup_test_project(&project_dir);

        // First build.
        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(lock_content.contains("6b1c381"));
        assert!(lock_content.contains("version = \"1.0.0\""));

        // Run fix deps update.
        let output = Command::new("fix")
            .args(&["deps", "update"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix deps update");
        assert!(
            output.status.success(),
            "fix deps update failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Verify lock file is unchanged (still pinned to v1.0.0, not updated to 1.2.1).
        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("6b1c381"),
            "Lock file should still contain the rev for tag v1.0.0 after deps update"
        );
        assert!(
            lock_content.contains("version = \"1.0.0\""),
            "Lock file should still show version 1.0.0 after deps update"
        );
    }

    // Test 7: Root pins with tag, transitive dep A requires compatible version range → success.
    #[test]
    fn test_transitive_root_pins_ok() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_root_pins_ok");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("6b1c381"),
            "Lock file should contain rev for tag v1.0.0"
        );
        assert!(
            lock_content.contains("version = \"1.0.0\""),
            "Lock file should show version 1.0.0"
        );
    }

    // Test 8: Root pins with tag, transitive dep A requires incompatible version range → error.
    #[test]
    fn test_transitive_root_pins_fail() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_root_pins_fail");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            !output.status.success(),
            "fix build should fail due to version incompatibility"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Failed to resolve dependencies"),
            "Error should mention failed resolution: {}",
            stderr
        );
    }

    // Test 9: Root uses version range, A pins with tag → A's pin is ignored, SemVer resolution used.
    // A pins math to tag v0.1.3 which is OUTSIDE root's ^1.0 range.
    // If A's pin were respected, the build would fail. Success proves the pin is ignored.
    #[test]
    fn test_transitive_local_pins_ignored() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_local_pins");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        // A pins math to v0.1.3 (outside root's ^1.0), but the pin is ignored for transitive deps.
        // The build succeeds because the resolver uses root's ^1.0 range via normal SemVer resolution.
        assert!(
            output.status.success(),
            "fix build failed (A's pin should be ignored): {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        // The resolved version should be in the ^1.0 range, not 0.1.3.
        assert!(
            !lock_content.contains("version = \"0.1.3\""),
            "Lock file should NOT show version 0.1.3 (A's pin should be ignored): {}",
            lock_content
        );
    }

    // Test 10: Both root and A pin with different tags → Root's pin is used, A's pin is ignored.
    #[test]
    fn test_transitive_both_pin() {
        install_fix();
        let (_temp_dir, project_dir) = setup_git_ref_test_env("transitive_both_pin");
        cleanup_test_project(&project_dir);

        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run fix build");
        assert!(
            output.status.success(),
            "fix build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let lock_content =
            fs::read_to_string(project_dir.join(LOCK_FILE_PATH)).expect("Lock file not found");
        assert!(
            lock_content.contains("6b1c381"),
            "Lock file should contain rev for Root's tag v1.0.0"
        );
        assert!(
            lock_content.contains("version = \"1.0.0\""),
            "Lock file should show version 1.0.0 (Root's pin)"
        );
    }
}
