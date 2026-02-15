// LSP integration tests module
pub mod lsp_client;

// LSP Integration Tests
// Tests for automatic lock file management in language server mode

#[cfg(test)]
mod tests {
    use super::lsp_client::LspClient;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use crate::LOCK_FILE_LSP_PATH;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::Duration,
    };
    use tempfile::TempDir;

    // Get the path to the test cases directory
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    // Create a temporary test environment with copied project files
    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_cases_dir().join(project_name);
        let test_case_dst = temp_dir.path().join(project_name);

        // Copy test case directory
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");

        (temp_dir, test_case_dst)
    }

    #[test]
    fn test_lsp_auto_lockfile_generation() {
        // Test: Verify that LSP automatically generates lock file with dependencies

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_with_deps");

        // Clean up before test
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(&project_dir)
            .output();

        // Start LSP client
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");

        // Initialize LSP
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");

        // Open main.fix (which imports Character only)
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");

        // Send didSave to trigger diagnostics
        client
            .save_document(Path::new("main.fix"))
            .expect("Failed to save main.fix");

        // Wait for initial diagnostics
        client.wait_for_server(Duration::from_secs(5));

        // Verify that main.fix has the specific error message
        let main_diagnostics = client.get_diagnostics(Path::new("main.fix"));
        let main_has_character_error = main_diagnostics.iter().any(|diag| {
            if let Some(message) = diag.get("message").and_then(|m| m.as_str()) {
                message.contains("Cannot find module") && message.contains("Character")
            } else {
                false
            }
        });

        assert!(
            main_has_character_error,
            "main.fix should have 'Cannot find module Character' error, but found: {:?}",
            main_diagnostics
        );

        // Add normal dependency
        let output = Command::new("fix")
            .arg("deps")
            .arg("add")
            .arg("character")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to add character dependency");

        assert!(
            output.status.success(),
            "Failed to add character dependency: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Send didSave to trigger diagnostics and LSP lock file generation
        client
            .save_document(Path::new("main.fix"))
            .expect("Failed to save main.fix");

        // Wait for LSP to process and generate lock file
        client.wait_for_server(Duration::from_secs(10));

        // Check if LSP lock file was generated
        let lsp_lock_file = project_dir.join(LOCK_FILE_LSP_PATH);
        assert!(
            lsp_lock_file.exists(),
            "LSP lock file should be created at {:?}",
            lsp_lock_file
        );

        // Verify lock file contains the dependency
        let content = fs::read_to_string(&lsp_lock_file).expect("Failed to read LSP lock file");
        assert!(
            content.contains("character"),
            "LSP lock file should include dependency (character)"
        );

        // Verify that all diagnostic errors have been resolved
        client
            .verify_no_diagnostic_errors()
            .expect("All diagnostic errors should be resolved after adding dependencies");

        // Shutdown
        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");

        // Check for reader thread errors
        client
            .finish()
            .expect("Reader thread should not have errors");
    }
}
