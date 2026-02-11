// ==================== Integration Tests ====================
// These tests verify that command-line arguments can be passed to Fix programs
// using the `fix run -- args` syntax and that get_args works correctly.

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::{path::PathBuf, process::Command};
    use tempfile::TempDir;

    // Get the path to the test cases directory
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_get_args/cases");
        path
    }

    // Create a temporary test environment with copied project files
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_cases_src = get_test_cases_dir();
        let test_cases_dst = temp_dir.path().to_path_buf();

        // Copy all test case directories
        copy_dir_recursive(&test_cases_src, &test_cases_dst).expect("Failed to copy test cases");

        let project_dir = test_cases_dst.join("simple_args");
        (temp_dir, project_dir)
    }

    #[test]
    fn test_get_args_with_fix_run() {
        // Test: fix run -- arg0 arg1
        // Verify that arguments passed via `fix run --` are correctly received by get_args

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();

        // Run `fix run -- arg0 arg1` in the test project directory
        let output = Command::new("fix")
            .arg("run")
            .arg("--")
            .arg("arg0")
            .arg("arg1")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix run");

        // Check that the command succeeded
        // The Fix program uses assert_eq to verify arguments, so if it exits with code 0,
        // the arguments were correctly received
        if !output.status.success() {
            eprintln!("fix run failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix run command failed - arguments were not received correctly");
        }

        // Verify the success message is printed
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Test passed"),
            "Expected success message in output, got: {}",
            stdout
        );
    }

    #[test]
    fn test_get_args_with_built_executable() {
        // Test: Build with `fix build`, then run the executable with arguments
        // Verify that arguments passed directly to the executable are correctly received

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();

        // Clean up any existing executable
        let executable_path = project_dir.join("a.out");
        let _ = std::fs::remove_file(&executable_path);

        // Build the project
        let output = Command::new("fix")
            .arg("build")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix build");

        // Check that the build succeeded
        if !output.status.success() {
            eprintln!("fix build failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix build command failed");
        }

        // Verify that a.out was created
        assert!(
            executable_path.exists(),
            "Expected a.out to be created after fix build"
        );

        // Run the executable with arguments
        let output = Command::new(&executable_path)
            .arg("arg0")
            .arg("arg1")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute a.out");

        // Check that the execution succeeded
        // The Fix program uses assert_eq to verify arguments, so if it exits with code 0,
        // the arguments were correctly received
        if !output.status.success() {
            eprintln!("a.out execution failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("a.out execution failed - arguments were not received correctly");
        }

        // Verify the success message is printed
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Test passed"),
            "Expected success message in output, got: {}",
            stdout
        );

        // Clean up
        let _ = std::fs::remove_file(&executable_path);
    }
}
