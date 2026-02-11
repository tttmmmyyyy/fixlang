// ==================== Integration Tests for `fix docs` Command ====================
// These tests use actual Fix projects in src/tests/test_docs/

#[cfg(test)]
mod integration_tests {
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use std::{fs, path::PathBuf, process::Command};
    use tempfile::TempDir;

    // Get the path to the test project directory
    fn get_test_project_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_docs");
        path
    }

    // Create a temporary test environment with copied project files
    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_project_src = get_test_project_dir();
        let test_project_dst = temp_dir.path().join("test_docs_project");

        // Copy test project directory
        copy_dir_recursive(&test_project_src, &test_project_dst)
            .expect("Failed to copy test project");

        (temp_dir, test_project_dst)
    }

    // Clean up generated documentation before running test
    fn cleanup_test_docs(project_dir: &PathBuf) {
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            let _ = fs::remove_dir_all(&docs_dir);
        }
    }

    #[test]
    fn test_docs_default_mode() {
        // This test verifies that `fix docs` (without --test flag):
        // 1. Generates documentation only for Main module
        // 2. Does NOT generate documentation for Test module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs` in the test project directory
        let output = Command::new("fix")
            .arg("docs")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs`"
        );

        // Verify Main.md exists
        let main_md = docs_dir.join("Main.md");
        assert!(
            main_md.exists(),
            "Main.md should be generated in default mode"
        );

        // Verify Test.md does NOT exist
        let test_md = docs_dir.join("Test.md");
        assert!(
            !test_md.exists(),
            "Test.md should NOT be generated in default mode (without --test flag)"
        );

        // Verify Main.md contains expected content
        let main_content = fs::read_to_string(&main_md).expect("Failed to read Main.md");
        assert!(
            main_content.contains("hello"),
            "Main.md should contain 'hello' function"
        );
    }

    #[test]
    fn test_docs_test_mode() {
        // This test verifies that `fix docs --test`:
        // 1. Generates documentation for Main module
        // 2. Also generates documentation for Test module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test` in the test project directory
        let output = Command::new("fix")
            .args(&["docs", "--test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs --test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs --test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --test command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs --test`"
        );

        // Verify Main.md exists
        let main_md = docs_dir.join("Main.md");
        assert!(
            main_md.exists(),
            "Main.md should be generated with --test flag"
        );

        // Verify Test.md exists
        let test_md = docs_dir.join("Test.md");
        assert!(
            test_md.exists(),
            "Test.md should be generated with --test flag"
        );

        // Verify Main.md contains expected content
        let main_content = fs::read_to_string(&main_md).expect("Failed to read Main.md");
        assert!(
            main_content.contains("hello"),
            "Main.md should contain 'hello' function"
        );

        // Verify Test.md contains expected content
        let test_content = fs::read_to_string(&test_md).expect("Failed to read Test.md");
        assert!(
            test_content.contains("test_helper"),
            "Test.md should contain 'test_helper' function"
        );
    }

    #[test]
    fn test_docs_test_mode_specific_module() {
        // This test verifies that `fix docs --test --mods Test`:
        // 1. Generates documentation only for Test module
        // 2. Does NOT generate documentation for Main module

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env();
        cleanup_test_docs(&project_dir);

        // Run `fix docs --test --mods Test` in the test project directory
        let output = Command::new("fix")
            .args(&["docs", "--test", "--mods", "Test"])
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix docs --test --mods Test");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs --test --mods Test failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs --test --mods Test command failed");
        }

        // Verify docs directory exists
        let docs_dir = project_dir.join("docs");
        assert!(
            docs_dir.exists(),
            "docs directory should be created by `fix docs --test --mods Test`"
        );

        // Verify Test.md exists
        let test_md = docs_dir.join("Test.md");
        assert!(
            test_md.exists(),
            "Test.md should be generated when specified with --mods"
        );

        // Verify Main.md does NOT exist
        let main_md = docs_dir.join("Main.md");
        assert!(
            !main_md.exists(),
            "Main.md should NOT be generated when only Test is specified with --mods"
        );

        // Verify Test.md contains expected content
        let test_content = fs::read_to_string(&test_md).expect("Failed to read Test.md");
        assert!(
            test_content.contains("test_helper"),
            "Test.md should contain 'test_helper' function"
        );
    }

    #[test]
    fn test_docs_comprehensive_output() {
        // This test verifies that `fix docs` generates documentation
        // that matches the expected output for a comprehensive test case
        // containing various language features (structs, unions, traits, type aliases, etc.)

        install_fix();

        // Set up test environment with comprehensive test case
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_project_dir().join("cases/comprehensive_docs");
        let test_case_dst = temp_dir.path().join("comprehensive_docs");

        // Copy test case directory
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");

        // Clean up any existing generated documentation
        cleanup_test_docs(&test_case_dst);

        // Run `fix docs` in the test case directory
        let output = Command::new("fix")
            .arg("docs")
            .current_dir(&test_case_dst)
            .output()
            .expect("Failed to execute fix docs");

        // Check that the command succeeded
        if !output.status.success() {
            eprintln!("fix docs failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("fix docs command failed");
        }

        // Read expected and actual documentation
        let expected_doc_path = test_case_dst.join("expected_docs/Main.md");
        let actual_doc_path = test_case_dst.join("docs/Main.md");

        assert!(
            expected_doc_path.exists(),
            "Expected documentation file should exist at {:?}",
            expected_doc_path
        );
        assert!(
            actual_doc_path.exists(),
            "Generated documentation file should exist at {:?}",
            actual_doc_path
        );

        let expected_content =
            fs::read_to_string(&expected_doc_path).expect("Failed to read expected documentation");
        let actual_content =
            fs::read_to_string(&actual_doc_path).expect("Failed to read generated documentation");

        // Compare the contents
        assert_eq!(
            actual_content, expected_content,
            "Generated documentation does not match expected output.\n\
            Expected file: {:?}\n\
            Actual file: {:?}\n\
            \n\
            If the difference is intentional, update the expected documentation by running:\n\
            cd src/tests/test_docs/cases/comprehensive_docs && fix docs && cp docs/Main.md expected_docs/Main.md",
            expected_doc_path, actual_doc_path
        );
    }
}
