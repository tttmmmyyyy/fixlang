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
        let root_uri = format!("file://{}", project_dir.display());
        client
            .initialize(&root_uri, Duration::from_secs(5))
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
        eprintln!("Waiting for diagnostics messages...");
        client.wait_for_server(Duration::from_secs(5));

        // Debug: print all diagnostics
        eprintln!("Diagnostics after opening main.fix:");
        let main_diagnostics = client.get_diagnostics(Path::new("main.fix"));
        eprintln!("  main.fix diagnostics: {} errors", main_diagnostics.len());

        // Print detailed diagnostic messages for main.fix
        for (i, diag) in main_diagnostics.iter().enumerate() {
            if let Some(message) = diag.get("message") {
                eprintln!("  main.fix error {}: {}", i, message);
            }
        }

        // Verify that main.fix has the specific error message
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
        eprintln!("Waiting for LSP to generate lock file and install dependencies...");
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
        eprintln!("LSP lock file full content:\n{}", content);

        assert!(
            content.contains("character"),
            "LSP lock file should include dependency (character)"
        );

        eprintln!("✓ LSP lock file generated with correct dependencies");

        // Note: Diagnostic error resolution after LSP lock file generation
        // should resolve import error in main.fix
        eprintln!("Checking final diagnostic state...");
        
        // Verify that all diagnostic errors have been resolved
        client
            .verify_no_diagnostic_errors()
            .expect("All diagnostic errors should be resolved after adding dependencies");

        eprintln!("✓ All diagnostic errors resolved");

        // For now, we've successfully verified that:
        // 1. Initial diagnostic errors are detected for main.fix ✓
        // 2. LSP lock file is generated ✓
        // 3. Lock file contains expected dependency (character) ✓
        // The automatic diagnostic refresh after lock file generation
        // may require additional LSP implementation or timing.
        eprintln!("✓ LSP auto-lockfile generation test completed successfully");

        // Shutdown
        client.shutdown().expect("Failed to shutdown LSP");
        
        // Check for reader thread errors
        client.finish().expect("Reader thread should not have errors");

        println!("✓ LSP auto-lock file generation test passed");
    }

    #[test]
    fn test_dependency_resolution_failure() {
        // Test: Verify that dependency resolution failures are handled properly

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("invalid_dependency_project");

        // Clean up before test
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(&project_dir)
            .output();

        // Try to update dependencies (should fail)
        let output = Command::new("fix")
            .arg("deps")
            .arg("update")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to execute fix deps update");

        // Verify that the command failed
        assert!(
            !output.status.success(),
            "Dependency resolution should fail for invalid dependencies"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Failed") || stderr.contains("error") || stderr.contains("Error"),
            "Error message should be present: {}",
            stderr
        );

        println!("✓ Dependency resolution failure test passed");
    }

    #[test]
    fn test_lsp_diagnostics_without_dependencies() {
        // Test: Verify that LSP diagnostics work even without dependencies

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("simple_project");

        // Clean up before test
        let _ = Command::new("fix")
            .arg("clean")
            .current_dir(&project_dir)
            .output();

        // Start LSP client
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");

        // Initialize LSP with shorter timeout for testing
        let root_uri = format!("file://{}", project_dir.display());

        match client.initialize(&root_uri, Duration::from_secs(5)) {
            Ok(_) => {}
            Err(e) => {
                // Check the log file for diagnostic information
                let log_path = project_dir.join(".fixlang/fix.log");
                if let Ok(log_content) = fs::read_to_string(&log_path) {
                    eprintln!("LSP log content:\n{}", log_content);
                }
                panic!("Failed to initialize LSP: {:?}", e);
            }
        }

        // Open main.fix
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to send didOpen");

        // Wait for server to process and send diagnostics
        client.wait_for_server(Duration::from_secs(5));

        // Verify no diagnostic errors for any file
        eprintln!("Verifying diagnostics...");
        client
            .verify_no_diagnostic_errors()
            .expect("Should have no diagnostic errors");

        // Shutdown (best effort - may timeout if server is still busy)
        let _ = client.shutdown();

        println!("✓ LSP diagnostics test passed");
    }
}
