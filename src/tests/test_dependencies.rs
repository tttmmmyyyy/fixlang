// ==================== Integration Tests ====================
// These tests use actual Fix projects in src/tests/test_dependencies/cases/

#[cfg(test)]
mod integration_tests {
    use crate::misc::copy_dir_recursive;
    use crate::tests::test_util::install_fix;
    use crate::{LOCK_FILE_PATH, LOCK_FILE_TEST_PATH};
    use std::{fs, path::PathBuf, process::Command};
    use tempfile::TempDir;

    // Get the path to the test cases directory
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_dependencies/cases");
        path
    }

    // Create a temporary test environment with copied project files
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_cases_src = get_test_cases_dir();
        let test_cases_dst = temp_dir.path().to_path_buf();

        // Copy all test case directories
        copy_dir_recursive(&test_cases_src, &test_cases_dst).expect("Failed to copy test cases");

        let main_project_dir = test_cases_dst.join("dependencies_for_test/main_project");
        (temp_dir, main_project_dir)
    }

    // Clean up lock files and build artifacts before running test
    fn cleanup_test_project(project_dir: &PathBuf) {
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_PATH));
        let _ = fs::remove_file(project_dir.join(LOCK_FILE_TEST_PATH));
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(project_dir)
            .output();
    }

    #[test]
    fn test_dependencies_build_mode() {
        // This test verifies that in build mode:
        // 1. Only fixdeps.lock is created
        // 2. fixdeps.test.lock is NOT created
        // 3. Only normal dependencies are included
        // 4. Test dependencies of normal dependencies are NOT included

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_project(&project_dir);

        // Run `fix build` in the test project directory
        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build command failed");
        }

        // Verify fixdeps.lock exists
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            lock_file.exists(),
            "fixdeps.lock should be created in build mode"
        );

        // Verify fixdeps.test.lock does NOT exist
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            !test_lock_file.exists(),
            "fixdeps.test.lock should NOT be created in build mode"
        );

        // Read and verify lock file contents
        let lock_content = fs::read_to_string(&lock_file).expect("Failed to read lock file");

        // Check that normal-dep is included
        assert!(
            lock_content.contains("normal-dep"),
            "Lock file should contain normal-dep"
        );

        // Check that test-dep is NOT included (neither as main project's test dependency
        // nor as normal-dep's test dependency)
        assert!(
            !lock_content.contains("test-dep"),
            "Lock file should NOT contain test-dep in build mode (test dependencies of dependencies should also be excluded)"
        );
    }

    #[test]
    fn test_dependencies_test_mode() {
        // This test verifies that `fix test` automatically handles test dependencies:
        // 1. fixdeps.test.lock is created if not present
        // 2. Test dependencies are properly available during test execution
        // Note: test-dep appears in fixdeps.test.lock because main-project directly depends on it,
        // not because normal-dep has it as a test dependency (dependency's test dependencies don't propagate)

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_project(&project_dir);

        // Run `fix test` directly (should auto-generate lock file and install dependencies)
        let output = Command::new("fix")
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix test command failed");
        }

        // Verify fixdeps.test.lock was created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            test_lock_file.exists(),
            "fixdeps.test.lock should be created by `fix test`"
        );

        // Verify fixdeps.lock was NOT created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            !lock_file.exists(),
            "fixdeps.lock should NOT be created by `fix test`"
        );

        // Read and verify test lock file contents
        let test_lock_content =
            fs::read_to_string(&test_lock_file).expect("Failed to read test lock file");

        // Check that both dependencies are included in test lock file
        assert!(
            test_lock_content.contains("normal-dep"),
            "Test lock file should contain normal-dep"
        );
        assert!(
            test_lock_content.contains("test-dep"),
            "Test lock file should contain test-dep"
        );

        // Verify the test output shows success
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("PASS"),
            "Test should pass with correct output"
        );
    }

    #[test]
    fn test_dependencies_build_workflow() {
        // This test verifies the explicit build workflow:
        // `fix deps update` → `fix deps install` → `fix build`

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_project(&project_dir);

        // Step 1: Update dependencies
        let update_output = Command::new("fix")
            .args(&["deps", "update"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps update");

        if !update_output.status.success() {
            eprintln!("fix deps update failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&update_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&update_output.stderr));
            panic!("fix deps update command failed");
        }

        // Verify fixdeps.lock was created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            lock_file.exists(),
            "fixdeps.lock should be created by `fix deps update`"
        );

        // Verify fixdeps.test.lock was NOT created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            !test_lock_file.exists(),
            "fixdeps.test.lock should NOT be created by `fix deps update` (without --test)"
        );

        // Step 2: Install dependencies
        let install_output = Command::new("fix")
            .args(&["deps", "install"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps install");

        if !install_output.status.success() {
            eprintln!("fix deps install failed:");
            eprintln!(
                "stdout: {}",
                String::from_utf8_lossy(&install_output.stdout)
            );
            eprintln!(
                "stderr: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
            panic!("fix deps install command failed");
        }

        // Step 3: Build
        let build_output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        if !build_output.status.success() {
            eprintln!("fix build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&build_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&build_output.stderr));
            panic!("fix build command failed");
        }

        // Verify lock file contents
        let lock_content = fs::read_to_string(&lock_file).expect("Failed to read lock file");
        assert!(
            lock_content.contains("normal-dep"),
            "Lock file should contain normal-dep"
        );
        assert!(
            !lock_content.contains("test-dep"),
            "Lock file should NOT contain test-dep (neither as main project's test dependency nor as normal-dep's test dependency)"
        );
    }

    #[test]
    fn test_dependencies_test_workflow() {
        // This test verifies the explicit test workflow:
        // `fix deps update --test` → `fix deps install --test` → `fix test`

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_project(&project_dir);

        // Step 1: Update test dependencies
        let update_output = Command::new("fix")
            .args(&["deps", "update", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps update --test");

        if !update_output.status.success() {
            eprintln!("fix deps update --test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&update_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&update_output.stderr));
            panic!("fix deps update --test command failed");
        }

        // Verify fixdeps.test.lock was created
        let test_lock_file = project_dir.join(LOCK_FILE_TEST_PATH);
        assert!(
            test_lock_file.exists(),
            "fixdeps.test.lock should be created by `fix deps update --test`"
        );

        // Verify fixdeps.lock was NOT created
        let lock_file = project_dir.join(LOCK_FILE_PATH);
        assert!(
            !lock_file.exists(),
            "fixdeps.lock should NOT be created by `fix deps update --test`"
        );

        // Step 2: Install test dependencies
        let install_output = Command::new("fix")
            .args(&["deps", "install", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps install --test");

        if !install_output.status.success() {
            eprintln!("fix deps install --test failed:");
            eprintln!(
                "stdout: {}",
                String::from_utf8_lossy(&install_output.stdout)
            );
            eprintln!(
                "stderr: {}",
                String::from_utf8_lossy(&install_output.stderr)
            );
            panic!("fix deps install --test command failed");
        }

        // Step 3: Run test
        let test_output = Command::new("fix")
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix test");

        if !test_output.status.success() {
            eprintln!("fix test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&test_output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&test_output.stderr));
            panic!("fix test command failed");
        }

        // Verify test lock file contents
        let test_lock_content =
            fs::read_to_string(&test_lock_file).expect("Failed to read test lock file");
        assert!(
            test_lock_content.contains("normal-dep"),
            "Test lock file should contain normal-dep"
        );
        assert!(
            test_lock_content.contains("test-dep"),
            "Test lock file should contain test-dep"
        );

        // Verify test output
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        assert!(
            stdout.contains("PASS"),
            "Test should pass with correct output"
        );
    }
}
