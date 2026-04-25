// LSP integration tests for "Rename Symbol".

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

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

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

        // Send `textDocument/rename` and return the response value (full
        // JSON-RPC response object, including any error).
        fn rename_raw(&mut self, file: &str, line: u32, col: u32, new_name: &str) -> Value {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/rename",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col },
                        "newName": new_name,
                    }),
                )
                .expect("Failed to send rename request");
            self.client.wait_for_server(Duration::from_secs(5));
            self.client
                .get_response(id)
                .expect("Should receive a rename response")
        }

        // Send `textDocument/rename` and unwrap the `result` (asserting it
        // is a `WorkspaceEdit` rather than an error).
        fn rename(&mut self, file: &str, line: u32, col: u32, new_name: &str) -> Value {
            let resp = self.rename_raw(file, line, col, new_name);
            assert!(
                resp.get("error").is_none(),
                "rename returned error: {:?}",
                resp.get("error")
            );
            resp.get("result")
                .expect("rename response should have a result")
                .clone()
        }

        // Send `textDocument/prepareRename` and return the result value.
        fn prepare_rename(&mut self, file: &str, line: u32, col: u32) -> Value {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/prepareRename",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col },
                    }),
                )
                .expect("Failed to send prepareRename request");
            self.client.wait_for_server(Duration::from_secs(5));
            let resp = self
                .client
                .get_response(id)
                .expect("Should receive a prepareRename response");
            resp.get("result")
                .expect("prepareRename response should have a result")
                .clone()
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

    // Count the total number of TextEdits across every URI in a
    // WorkspaceEdit `result` value.
    fn count_edits(workspace_edit: &Value) -> usize {
        workspace_edit
            .get("changes")
            .and_then(|c| c.as_object())
            .map(|m| m.values().map(|v| v.as_array().map_or(0, |a| a.len())).sum())
            .unwrap_or(0)
    }

    // Collect (uri suffix, count) pairs for the changes in a WorkspaceEdit.
    fn changes_per_file(workspace_edit: &Value) -> Vec<(String, usize)> {
        workspace_edit
            .get("changes")
            .and_then(|c| c.as_object())
            .map(|m| {
                let mut v: Vec<(String, usize)> = m
                    .iter()
                    .map(|(k, v)| {
                        let count = v.as_array().map_or(0, |a| a.len());
                        let suffix = k.rsplit('/').next().unwrap_or(k).to_string();
                        (suffix, count)
                    })
                    .collect();
                v.sort();
                v
            })
            .unwrap_or_default()
    }

    fn assert_all_edits_have_new_text(workspace_edit: &Value, new_name: &str) {
        let changes = workspace_edit
            .get("changes")
            .and_then(|c| c.as_object())
            .expect("workspace_edit should have changes");
        for edits in changes.values() {
            for edit in edits.as_array().unwrap_or(&vec![]) {
                let nt = edit
                    .get("newText")
                    .and_then(|v| v.as_str())
                    .expect("edit should have newText");
                assert_eq!(nt, new_name, "all edits should use the same new_name");
            }
        }
    }

    // =======================================================================
    // rename_basic fixture lines (0-indexed):
    //
    // lib.fix:
    //   2: helper : I64 -> I64;
    //   3: helper = |x| x + 1;
    //   5: double : I64 -> I64;
    //   6: double = |x| helper(helper(x));
    //
    // main.fix:
    //   2: import Lib::{helper};
    //   4: use_helper : I64 -> I64;
    //   5: use_helper = |a| helper(a) + 2;
    //   8: local_demo = (
    //   9:     let y = 10 in
    //  10:     y + y + y
    //  11: );
    // =======================================================================

    /// RB-1: rename a global value across files. Cursor on the declaration
    /// LHS in lib.fix.
    #[test]
    fn test_rename_global_decl() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 0, "boost");

        // Expected edits:
        //   lib.fix : decl + def + 2 uses in `double` = 4
        //   main.fix: import-leaf + 1 use = 2
        // Total: 6
        assert_eq!(count_edits(&we), 6, "WorkspaceEdit: {:?}", we);

        let per_file = changes_per_file(&we);
        assert_eq!(per_file, vec![("lib.fix".to_string(), 4), ("main.fix".to_string(), 2)]);

        assert_all_edits_have_new_text(&we, "boost");
        ctx.shutdown();
    }

    /// RB-2: rename a global value, starting from a use site in lib.fix.
    #[test]
    fn test_rename_global_from_use_same_file() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        // Cursor on first `helper` in `double = |x| helper(helper(x));` (line 6, col 13).
        let we = ctx.rename("lib.fix", 6, 13, "boost");
        assert_eq!(count_edits(&we), 6);
        ctx.shutdown();
    }

    /// RB-3: rename a global value, starting from a use site in main.fix.
    #[test]
    fn test_rename_global_from_use_other_file() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        // Cursor on `helper` in `use_helper = |a| helper(a) + 2;` (line 5, col 17).
        let we = ctx.rename("main.fix", 5, 17, "boost");
        assert_eq!(count_edits(&we), 6);
        ctx.shutdown();
    }

    /// RB-4: rename a global value, starting from the import statement.
    #[test]
    fn test_rename_global_from_import() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        // Cursor on `helper` in `import Lib::{helper};` (line 2, col 13).
        let we = ctx.rename("main.fix", 2, 13, "boost");
        assert_eq!(count_edits(&we), 6);
        ctx.shutdown();
    }

    /// RB-5: rename a local let-bound variable.
    #[test]
    fn test_rename_local_let() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        // Cursor on the binder `y` in `let y = 10 in` (line 9, col 8).
        let we = ctx.rename("main.fix", 9, 8, "z");
        // Binder + 3 uses on the next line = 4 edits, all in main.fix.
        assert_eq!(count_edits(&we), 4);
        let per_file = changes_per_file(&we);
        assert_eq!(per_file, vec![("main.fix".to_string(), 4)]);
        assert_all_edits_have_new_text(&we, "z");
        ctx.shutdown();
    }

    /// RB-6: rename rejected on an invalid identifier (keyword).
    #[test]
    fn test_rename_reject_keyword() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        let resp = ctx.rename_raw("lib.fix", 2, 0, "let");
        assert!(
            resp.get("error").is_some(),
            "rename to keyword should be rejected, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RB-7: rename rejected on an invalid identifier (uppercase start).
    #[test]
    fn test_rename_reject_uppercase() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        let resp = ctx.rename_raw("lib.fix", 2, 0, "Boost");
        assert!(
            resp.get("error").is_some(),
            "rename of value to uppercase name should be rejected, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RB-8: prepareRename returns defaultBehavior for a global value.
    #[test]
    fn test_prepare_rename_global_value() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("lib.fix", 2, 0);
        // Should be `{ defaultBehavior: true }`, not null.
        assert!(
            result
                .get("defaultBehavior")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "expected defaultBehavior=true, got: {:?}",
            result
        );
        ctx.shutdown();
    }

    /// RB-9: prepareRename returns defaultBehavior for a local variable.
    #[test]
    fn test_prepare_rename_local() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("main.fix", 9, 8);
        assert!(
            result
                .get("defaultBehavior")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "expected defaultBehavior=true, got: {:?}",
            result
        );
        ctx.shutdown();
    }

    /// RB-10: prepareRename returns null for a position where rename isn't
    /// supported yet (Phase C1: types are not supported).
    #[test]
    fn test_prepare_rename_type_not_supported_yet() {
        let mut ctx = LspTestCtx::setup("rename_basic", &["lib.fix", "main.fix"]);
        // `helper : I64 -> I64;` — cursor on `I64` (the return type, after `> `).
        // line 2, col 14 = `I64` start.
        let result = ctx.prepare_rename("lib.fix", 2, 14);
        assert!(
            result.is_null(),
            "expected null (rename not allowed) for a type, got: {:?}",
            result
        );
        ctx.shutdown();
    }
}
