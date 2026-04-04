// LSP integration tests for "textDocument/completion" feature.
//
// Verifies that associated types appear in completion candidates.

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

    struct LspCompletionCtx {
        client: LspClient,
        project_dir: PathBuf,
        _temp_dir: TempDir,
    }

    impl LspCompletionCtx {
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

        /// Send textDocument/completion and return the result items.
        fn complete(&mut self, file: &str, line: u32, col: u32) -> Vec<Value> {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/completion",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col }
                    }),
                )
                .expect("Failed to send completion request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a completion response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            // The result can be either an array or a CompletionList object.
            if result.is_array() {
                result.as_array().unwrap().clone()
            } else {
                result
                    .get("items")
                    .and_then(|items| items.as_array())
                    .cloned()
                    .unwrap_or_default()
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

    /// Test that associated types appear in completion candidates.
    /// The completion project defines various entities:
    /// - Type (struct): MyData
    /// - Type alias: MyInt
    /// - Value (function): my_func
    /// - Trait: MyTrait
    /// - Trait alias: MyTraitAlias
    /// - Trait member: MyTrait::get_elem
    /// - Associated type: MyTrait::Elem
    #[test]
    fn test_completion_includes_all_entity_kinds() {
        let mut ctx = LspCompletionCtx::setup("completion", &["lib.fix", "main.fix"]);

        // Request completion at line 0, col 0 to get all available completions.
        let items = ctx.complete("main.fix", 0, 0);

        // Extract labels from completion items.
        let labels: Vec<String> = items
            .iter()
            .filter_map(|item| item.get("label").and_then(|l| l.as_str()).map(String::from))
            .collect();

        // Type (struct)
        assert!(
            labels.iter().any(|l| l == "Lib::MyData"),
            "Type `MyData` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Type alias
        assert!(
            labels.iter().any(|l| l == "Lib::MyInt"),
            "Type alias `MyInt` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Value (function)
        assert!(
            labels.iter().any(|l| l == "Lib::my_func"),
            "Value `my_func` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Trait
        assert!(
            labels.iter().any(|l| l == "Lib::MyTrait"),
            "Trait `MyTrait` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Trait alias
        assert!(
            labels.iter().any(|l| l == "Lib::MyTraitAlias"),
            "Trait alias `MyTraitAlias` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Trait member (value)
        assert!(
            labels.iter().any(|l| l == "Lib::MyTrait::get_elem"),
            "Trait member `get_elem` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        // Associated type
        assert!(
            labels.iter().any(|l| l.contains("Elem")),
            "Associated type `Elem` should appear in completion candidates. Got labels: {:?}",
            labels
        );

        ctx.shutdown();
    }
}
