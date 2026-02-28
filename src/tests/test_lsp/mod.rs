// LSP integration tests module
pub mod lsp_client;

// LSP Integration Tests
// Tests for automatic lock file management in language server mode

#[cfg(test)]
mod tests {
    use super::lsp_client::LspClient;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use crate::LOCK_FILE_LSP_PATH;
    use serde_json::json;
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

    #[test]
    fn test_lsp_find_all_references_value_from_usage() {
        // Test: Verify that the LSP server correctly finds all references
        // to a global value across multiple files.

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        // Start LSP client
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");

        // Initialize LSP
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");

        // Open both files
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");

        // Trigger diagnostics and wait until the server is ready to handle requests.
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // --- Find references for `helper` function ---
        // Place cursor on a USAGE of `helper` in lib.fix line 13 (0-indexed):
        //   `double_helper = |x| helper(helper(x));`
        //                        ^-- column 20
        // Note: the cursor must be on a usage site, not the definition LHS.
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 13, "character": 20 },
                    "context": { "includeDeclaration": true }
                }),
            )
            .expect("Failed to send references request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a references response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let locations = result.as_array().unwrap();

        // `helper` is defined in lib.fix and used in:
        //   - lib.fix line 9: definition (included because includeDeclaration=true)
        //   - lib.fix line 13: double_helper calls helper twice
        //   - main.fix line 6: use_helper calls Lib::helper twice
        // Total: at least 3 distinct locations
        assert!(
            locations.len() >= 3,
            "Expected at least 3 references to `helper`, got {}. Locations: {:?}",
            locations.len(),
            locations
        );

        // Verify references span across both files
        let has_lib_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("lib.fix"))
        });
        let has_main_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("main.fix"))
        });

        assert!(has_lib_ref, "Should have references in lib.fix");
        assert!(has_main_ref, "Should have references in main.fix");

        // Shutdown
        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    #[test]
    fn test_lsp_call_hierarchy_value_from_usage() {
        // Test: Verify that the LSP server supports call hierarchy
        // (prepare, incoming calls, outgoing calls).

        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        // Start LSP client
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");

        // Initialize LSP
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");

        // Open both files
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");

        // Trigger diagnostics and wait until the server is ready to handle requests.
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // --- Prepare call hierarchy for `helper` ---
        // Place cursor on a USAGE of `helper` in lib.fix line 13 (0-indexed):
        //   `double_helper = |x| helper(helper(x));`
        //                        ^-- column 20
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/prepareCallHierarchy",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 13, "character": 20 }
                }),
            )
            .expect("Failed to send prepareCallHierarchy request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a prepareCallHierarchy response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let items = result.as_array().unwrap();
        assert_eq!(
            items.len(),
            1,
            "Should return exactly one CallHierarchyItem, got: {:?}",
            items
        );

        let helper_item = &items[0];
        let name = helper_item
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Item should have a name");
        assert!(
            name.contains("helper"),
            "Item name should contain 'helper', got: {}",
            name
        );

        // --- Incoming calls for `helper` ---
        let id = client
            .send_request(
                "callHierarchy/incomingCalls",
                json!({ "item": helper_item }),
            )
            .expect("Failed to send incomingCalls request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive an incomingCalls response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let incoming = result.as_array().unwrap();

        // `helper` is called by `double_helper` (in lib.fix) and `use_helper` (in main.fix)
        assert!(
            incoming.len() >= 2,
            "Expected at least 2 incoming callers, got {}. Callers: {:?}",
            incoming.len(),
            incoming
        );

        let caller_names: Vec<String> = incoming
            .iter()
            .filter_map(|call| {
                call.get("from")
                    .and_then(|f| f.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from)
            })
            .collect();

        assert!(
            caller_names.iter().any(|n| n.contains("double_helper")),
            "Incoming callers should include double_helper. Found: {:?}",
            caller_names
        );
        assert!(
            caller_names.iter().any(|n| n.contains("use_helper")),
            "Incoming callers should include use_helper. Found: {:?}",
            caller_names
        );

        // --- Outgoing calls from `use_helper` ---
        // Prepare call hierarchy for `use_helper` at a USAGE site in main.fix line 14:
        //   `caller = |x| use_helper(x) + use_helper(x + 1);`
        //                 ^-- column 13
        let main_uri = format!("file://{}", project_dir.join("main.fix").display());

        let id = client
            .send_request(
                "textDocument/prepareCallHierarchy",
                json!({
                    "textDocument": { "uri": main_uri },
                    "position": { "line": 14, "character": 13 }
                }),
            )
            .expect("Failed to send prepareCallHierarchy for use_helper");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a prepareCallHierarchy response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let items = result.as_array().unwrap();
        assert_eq!(
            items.len(),
            1,
            "Should return exactly one item for use_helper. Got: {:?}",
            items
        );

        let use_helper_item = &items[0];
        let use_helper_name = use_helper_item
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Item should have a name");
        assert!(
            use_helper_name.contains("use_helper"),
            "Item name should contain 'use_helper', got: {}",
            use_helper_name
        );

        // Get outgoing calls from use_helper
        let id = client
            .send_request(
                "callHierarchy/outgoingCalls",
                json!({ "item": use_helper_item }),
            )
            .expect("Failed to send outgoingCalls request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive an outgoingCalls response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let outgoing = result.as_array().unwrap();

        // `use_helper` calls `Lib::helper`
        assert!(
            !outgoing.is_empty(),
            "Expected at least 1 outgoing call from use_helper, got 0"
        );

        let callee_names: Vec<String> = outgoing
            .iter()
            .filter_map(|call| {
                call.get("to")
                    .and_then(|t| t.get("name"))
                    .and_then(|n| n.as_str())
                    .map(String::from)
            })
            .collect();

        assert!(
            callee_names.iter().any(|n| n.contains("helper")),
            "Outgoing calls from use_helper should include helper. Found: {:?}",
            callee_names
        );

        // Shutdown
        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// Regression test: "Find All References" should work when the cursor is on the
    /// **declaration** (left-hand side) of a symbol, not just on usage sites.
    #[test]
    fn test_lsp_find_all_references_value_from_declaration() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // Place cursor on the DECLARATION of `helper` in lib.fix line 8 (0-indexed):
        //   `helper : I64 -> I64;`
        //     ^-- column 1 (inside the declaration LHS name)
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 8, "character": 1 },
                    "context": { "includeDeclaration": true }
                }),
            )
            .expect("Failed to send references request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a references response from definition site");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let locations = result.as_array().unwrap();

        // Same references as from a usage site: at least 3.
        assert!(
            locations.len() >= 3,
            "Expected at least 3 references to `helper` from definition site, got {}. Locations: {:?}",
            locations.len(),
            locations
        );

        // Verify references span across both files
        let has_lib_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("lib.fix"))
        });
        let has_main_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("main.fix"))
        });
        assert!(has_lib_ref, "Should have references in lib.fix");
        assert!(has_main_ref, "Should have references in main.fix");

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// Regression test: "Call Hierarchy" should work when the cursor is on the
    /// **declaration** (left-hand side) of a function, not just on usage sites.
    #[test]
    fn test_lsp_call_hierarchy_value_from_declaration() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // Place cursor on the DECLARATION of `helper` in lib.fix line 8 (0-indexed):
        //   `helper : I64 -> I64;`
        //     ^-- column 1 (inside the declaration LHS name)
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/prepareCallHierarchy",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 8, "character": 1 }
                }),
            )
            .expect("Failed to send prepareCallHierarchy request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a prepareCallHierarchy response from definition site");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let items = result.as_array().unwrap();
        assert_eq!(
            items.len(),
            1,
            "Should return exactly one CallHierarchyItem from definition site, got: {:?}",
            items
        );

        let helper_item = &items[0];
        let name = helper_item
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Item should have a name");
        assert!(
            name.contains("helper"),
            "Item name should contain 'helper', got: {}",
            name
        );

        // Incoming calls should still work
        let id = client
            .send_request(
                "callHierarchy/incomingCalls",
                json!({ "item": helper_item }),
            )
            .expect("Failed to send incomingCalls request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive an incomingCalls response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let incoming = result.as_array().unwrap();
        assert!(
            incoming.len() >= 2,
            "Expected at least 2 incoming callers from definition site, got {}. Callers: {:?}",
            incoming.len(),
            incoming
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// Regression test: "Find All References" should work when the cursor is on the
    /// **definition** (the `hoge = value;` line) of a symbol, not just on usage sites.
    #[test]
    fn test_lsp_find_all_references_value_from_definition() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // Place cursor on the DEFINITION of `helper` in lib.fix line 9 (0-indexed):
        //   `helper = |x| x + 1;`
        //     ^-- column 1 (inside the definition LHS name)
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/references",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 9, "character": 1 },
                    "context": { "includeDeclaration": true }
                }),
            )
            .expect("Failed to send references request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a references response from definition site");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let locations = result.as_array().unwrap();

        // Same references as from a usage/declaration site: at least 3.
        assert!(
            locations.len() >= 3,
            "Expected at least 3 references to `helper` from definition site, got {}. Locations: {:?}",
            locations.len(),
            locations
        );

        // Verify references span across both files
        let has_lib_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("lib.fix"))
        });
        let has_main_ref = locations.iter().any(|loc| {
            loc.get("uri")
                .and_then(|u| u.as_str())
                .map_or(false, |u| u.contains("main.fix"))
        });
        assert!(has_lib_ref, "Should have references in lib.fix");
        assert!(has_main_ref, "Should have references in main.fix");

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// Regression test: "Call Hierarchy" should work when the cursor is on the
    /// **definition** (the `hoge = value;` line) of a function, not just on usage sites.
    #[test]
    fn test_lsp_call_hierarchy_value_from_definition() {
        install_fix();
        let (_temp_dir, project_dir) = setup_test_env("project_references");

        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // Place cursor on the DEFINITION of `helper` in lib.fix line 9 (0-indexed):
        //   `helper = |x| x + 1;`
        //     ^-- column 1 (inside the definition LHS name)
        let lib_uri = format!("file://{}", project_dir.join("lib.fix").display());

        let id = client
            .send_request(
                "textDocument/prepareCallHierarchy",
                json!({
                    "textDocument": { "uri": lib_uri },
                    "position": { "line": 9, "character": 1 }
                }),
            )
            .expect("Failed to send prepareCallHierarchy request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive a prepareCallHierarchy response from definition site");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let items = result.as_array().unwrap();
        assert_eq!(
            items.len(),
            1,
            "Should return exactly one CallHierarchyItem from definition site, got: {:?}",
            items
        );

        let helper_item = &items[0];
        let name = helper_item
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Item should have a name");
        assert!(
            name.contains("helper"),
            "Item name should contain 'helper', got: {}",
            name
        );

        // Incoming calls should still work
        let id = client
            .send_request(
                "callHierarchy/incomingCalls",
                json!({ "item": helper_item }),
            )
            .expect("Failed to send incomingCalls request");

        client.wait_for_server(Duration::from_secs(5));

        let response = client
            .get_response(id)
            .expect("Should receive an incomingCalls response");
        let result = response
            .get("result")
            .expect("Response should have a result field");
        assert!(result.is_array(), "Result should be an array");
        let incoming = result.as_array().unwrap();
        assert!(
            incoming.len() >= 2,
            "Expected at least 2 incoming callers from definition site, got {}. Callers: {:?}",
            incoming.len(),
            incoming
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }
}