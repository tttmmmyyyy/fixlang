// LSP integration tests for `textDocument/hover`. Right now the focus
// is on hover behaviour around expressions that contain `Std::#hole`
// placeholders: even when ERR_HOLE rejects the value, hover on the
// surrounding local variables should still show their inferred types.

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

    struct LspTestCtx {
        client: LspClient,
        project_dir: PathBuf,
        _temp_dir: TempDir,
    }

    impl LspTestCtx {
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

        /// Send textDocument/hover and return the result value (the LSP
        /// server returns either a Hover object or null).
        fn hover(&mut self, file: &str, line: u32, col: u32) -> Value {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/hover",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col }
                    }),
                )
                .expect("Failed to send hover request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a hover response");
            response
                .get("result")
                .cloned()
                .expect("Response should have a result field")
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

    fn hover_text(hover: &Value) -> Option<String> {
        let contents = hover.get("contents")?;
        // hover.contents is `MarkupContent { kind, value }`.
        contents
            .get("value")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Hover on local variables inside a hole-bearing expression must
    /// still show their inferred types. The check_type pass is allowed
    /// to fail (with ERR_HOLE), but the typed expression must be
    /// preserved so the LSP can serve hover.
    #[test]
    fn test_hover_local_in_hole_expression() {
        let mut ctx = LspTestCtx::setup("hover_with_hole", &["main.fix"]);

        // Source layout (1-based for human reading):
        //
        //   8: fact : I64 -> I64 = |n| (
        //   9:     Iterator::range(0, n).fold(1, |i, acc| )
        //  10: );
        //
        // Hover on `i` (line index 8, char 35) — should show `I64`.
        // Hover on `acc` (line index 8, char 38) — should show `I64`.

        let i_hover = ctx.hover("main.fix", 8, 35);
        let i_text = hover_text(&i_hover).expect("hover on `i` should return content");
        assert!(
            i_text.contains("I64"),
            "hover on `i` should mention I64. Got: {:?}",
            i_text
        );

        let acc_hover = ctx.hover("main.fix", 8, 38);
        let acc_text = hover_text(&acc_hover).expect("hover on `acc` should return content");
        assert!(
            acc_text.contains("I64"),
            "hover on `acc` should mention I64. Got: {:?}",
            acc_text
        );

        ctx.shutdown();
    }

    /// Hover on or near the hole position itself should NOT leak the
    /// internal `Std::#hole` name. The user has no way to spell `#hole`
    /// in their own code (the parser's `name` rule rejects `#`), so
    /// surfacing it would just expose an implementation detail.
    #[test]
    fn test_hover_on_hole_does_not_leak_internal_name() {
        let mut ctx = LspTestCtx::setup("hover_with_hole", &["main.fix"]);

        // The hole sits where the lambda body would have been:
        //   Iterator::range(0, n).fold(1, |i, acc| )
        //                                          ^^ position of the empty body
        // Hover at the closing `)` of the fold call (line 8, char 43-44)
        // and a few characters around — none should mention `#hole`.
        for col in 39..=44 {
            let hov = ctx.hover("main.fix", 8, col);
            if let Some(text) = hover_text(&hov) {
                assert!(
                    !text.contains("#hole"),
                    "hover at col {} leaked the internal `#hole` name. Got: {:?}",
                    col,
                    text
                );
            }
        }

        ctx.shutdown();
    }
}
