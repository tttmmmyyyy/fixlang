// LSP integration tests for "textDocument/codeAction" (quick fix) feature.
//
// Verifies that quick fix suggestions include import actions for associated types.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use serde_json::{json, Value};
    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };
    use tempfile::TempDir;

    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_case_src = get_test_cases_dir().join(project_name);
        let test_case_dst = temp_dir.path().join(project_name);
        copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
        let test_case_dst = test_case_dst
            .canonicalize()
            .expect("Failed to canonicalize test case path");
        (temp_dir, test_case_dst)
    }

    struct LspQuickFixCtx {
        client: LspClient,
        project_dir: PathBuf,
        _temp_dir: TempDir,
    }

    impl LspQuickFixCtx {
        fn setup(project_name: &str, files: &[&str]) -> Self {
            install_fix();
            let (temp_dir, project_dir) = setup_test_env(project_name);
            let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
            client
                .initialize(&project_dir, Duration::from_secs(5))
                .expect("Failed to initialize LSP");
            for f in files {
                client
                    .open_document(Path::new(f))
                    .expect(&format!("Failed to open {}", f));
            }
            let trigger_file = files.last().unwrap();
            client.trigger_and_wait_for_diagnostics(Path::new(trigger_file));
            Self {
                client,
                project_dir,
                _temp_dir: temp_dir,
            }
        }

        fn file_uri(&self, file: &str) -> String {
            format!("file://{}", self.project_dir.join(file).display())
        }

        /// Request code actions for a given range with the provided diagnostics.
        fn code_actions(
            &mut self,
            file: &str,
            diagnostics: Vec<Value>,
            start_line: u32,
            start_col: u32,
            end_line: u32,
            end_col: u32,
        ) -> Vec<Value> {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/codeAction",
                    json!({
                        "textDocument": { "uri": uri },
                        "range": {
                            "start": { "line": start_line, "character": start_col },
                            "end": { "line": end_line, "character": end_col }
                        },
                        "context": {
                            "diagnostics": diagnostics
                        }
                    }),
                )
                .expect("Failed to send codeAction request");
            self.client.wait_for_server(Duration::from_secs(10));
            let response = self.client.get_response(id);
            if response.is_none() {
                return vec![];
            }
            let response = response.unwrap();
            let result = response
                .get("result")
                .expect("Response should have a result field");
            if result.is_array() {
                result.as_array().unwrap().clone()
            } else {
                vec![]
            }
        }

        fn shutdown(mut self) {
            self.client
                .shutdown(Duration::from_millis(500))
                .expect("Failed to shutdown LSP");
            self.client
                .finish()
                .expect("Reader thread should not have errors");
        }
    }

    /// Test that quick fix suggests importing an associated type when it is unknown.
    ///
    /// The quickfix project compiles cleanly (all names are imported).
    /// We send fabricated diagnostics with code "unknown-name" to test
    /// that the code action handler can find the names in available_names.
    /// First, we verify traits work (baseline), then check associated types.
    #[test]
    fn test_quickfix_import_associated_type() {
        let mut ctx = LspQuickFixCtx::setup("quickfix", &["lib.fix", "main.fix"]);

        // Verify no real diagnostics (project compiles cleanly).
        let diagnostics = ctx.client.get_diagnostics(Path::new("main.fix"));
        assert!(
            diagnostics.is_empty(),
            "Project should compile cleanly. Got diagnostics: {:?}",
            diagnostics
        );

        // Baseline: verify that a fabricated diagnostic for a trait triggers a quick fix.
        let fake_trait_diag = json!({
            "code": "unknown-name",
            "data": "MyTrait",
            "message": "Unknown trait `MyTrait`.",
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 7 }
            },
            "severity": 1
        });
        let actions = ctx.code_actions("main.fix", vec![fake_trait_diag], 0, 0, 0, 7);
        let titles: Vec<String> = actions
            .iter()
            .filter_map(|a| a.get("title").and_then(|t| t.as_str()).map(String::from))
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("MyTrait")),
            "Quick fix should suggest importing `MyTrait`. Got actions: {:?}",
            titles
        );

        // Test: fabricated diagnostic for an unknown associated type.
        let fake_assoc_diag = json!({
            "code": "unknown-name",
            "data": "MyElem",
            "message": "Unknown associated type name `MyElem`.",
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 6 }
            },
            "severity": 1
        });
        let actions = ctx.code_actions("main.fix", vec![fake_assoc_diag], 0, 0, 0, 6);
        let titles: Vec<String> = actions
            .iter()
            .filter_map(|a| a.get("title").and_then(|t| t.as_str()).map(String::from))
            .collect();
        assert!(
            titles.iter().any(|t| t.contains("MyElem")),
            "Quick fix should suggest importing associated type `MyElem`. Got actions: {:?}",
            titles
        );

        ctx.shutdown();
    }
}
