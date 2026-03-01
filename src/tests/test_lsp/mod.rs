// LSP integration tests module
pub mod lsp_client;
pub mod test_references;

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

    #[test]
    fn test_lsp_auto_lockfile_generation_for_test_deps() {
        // Test: Verify that LSP automatically generates lock file with test-dependencies
        // when test.fix imports an external library that is listed in test_dependencies
        // but not yet in fixproj.toml.

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_with_test_deps");

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

        // Open test.fix (which imports Character but has no test_dependencies)
        client
            .open_document(Path::new("test.fix"))
            .expect("Failed to open test.fix");

        // Send didSave to trigger diagnostics
        client
            .save_document(Path::new("test.fix"))
            .expect("Failed to save test.fix");

        // Wait for initial diagnostics
        client.wait_for_server(Duration::from_secs(5));

        // Verify that test.fix has the specific error message about missing Character module
        let test_diagnostics = client.get_diagnostics(Path::new("test.fix"));
        let has_character_error = test_diagnostics.iter().any(|diag| {
            if let Some(message) = diag.get("message").and_then(|m| m.as_str()) {
                message.contains("Cannot find module") && message.contains("Character")
            } else {
                false
            }
        });

        assert!(
            has_character_error,
            "test.fix should have 'Cannot find module Character' error, but found: {:?}",
            test_diagnostics
        );

        // Add character as a test dependency
        let output = Command::new("fix")
            .arg("deps")
            .arg("add")
            .arg("--test")
            .arg("character")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to add character test dependency");

        assert!(
            output.status.success(),
            "Failed to add character test dependency: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // Send didSave to trigger diagnostics and LSP lock file generation
        client
            .save_document(Path::new("test.fix"))
            .expect("Failed to save test.fix");

        // Wait for LSP to process and generate lock file
        client.wait_for_server(Duration::from_secs(10));

        // Check if LSP lock file was generated
        let lsp_lock_file = project_dir.join(LOCK_FILE_LSP_PATH);
        assert!(
            lsp_lock_file.exists(),
            "LSP lock file should be created at {:?}",
            lsp_lock_file
        );

        // Verify lock file contains the test dependency
        let content = fs::read_to_string(&lsp_lock_file).expect("Failed to read LSP lock file");
        assert!(
            content.contains("character"),
            "LSP lock file should include test dependency (character)"
        );

        // Verify that all diagnostic errors have been resolved
        client
            .verify_no_diagnostic_errors()
            .expect("All diagnostic errors should be resolved after adding test dependencies");

        // Shutdown
        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");

        // Check for reader thread errors
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    #[test]
    fn test_lsp_dependency_resolution_failure() {
        // Test: Verify that LSP shows diagnostic errors when dependency resolution fails

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_with_invalid_dep");

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

        // Open main.fix
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");

        // Send didSave to trigger diagnostics and dependency resolution
        client
            .save_document(Path::new("main.fix"))
            .expect("Failed to save main.fix");

        // Wait for LSP to process
        client.wait_for_server(Duration::from_secs(10));

        // Get all diagnostics (dependency resolution errors may not be tied to main.fix)
        let all_diagnostics = client.get_all_diagnostics();
        
        // Verify that diagnostic errors are present somewhere
        // When dependency resolution fails, the error should be visible to the user
        let total_diagnostics: usize = all_diagnostics.values().map(|v| v.len()).sum();
        assert!(
            total_diagnostics > 0,
            "Should have diagnostic errors when dependency resolution fails"
        );

        // Collect all diagnostic messages from all files
        let all_messages: Vec<String> = all_diagnostics
            .values()
            .flat_map(|diagnostics| {
                diagnostics.iter().filter_map(|diag| {
                    diag.get("message").and_then(|m| m.as_str()).map(String::from)
                })
            })
            .collect();

        // Verify the error message contains "Failed to clone the repository"
        let has_clone_error = all_messages.iter().any(|message| {
            message.contains("Failed to clone the repository")
        });

        assert!(
            has_clone_error,
            "Should have 'Failed to clone the repository' error. Found messages: {:?}",
            all_messages
        );

        // Verify that at least one error has Error severity (not just warning)
        // This ensures the failure is not silent - user can see it as an error
        let has_error_severity = all_diagnostics.values().flatten().any(|diag| {
            if let Some(severity) = diag.get("severity").and_then(|s| s.as_u64()) {
                severity == 1 // 1 = Error in LSP protocol
            } else {
                false
            }
        });

        assert!(
            has_error_severity,
            "Dependency resolution failure should be shown as an error (not just warning). Diagnostics: {:?}",
            all_diagnostics
        );

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
