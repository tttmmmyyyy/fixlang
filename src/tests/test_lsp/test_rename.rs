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

        // Send `textDocument/prepareRename` and return the full response
        // value (so tests can inspect `result` and `error` independently).
        fn prepare_rename_raw(&mut self, file: &str, line: u32, col: u32) -> Value {
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
            self.client
                .get_response(id)
                .expect("Should receive a prepareRename response")
        }

        // Send `textDocument/prepareRename` and return the `result`
        // value; panics if the server returned a `ResponseError`.
        fn prepare_rename(&mut self, file: &str, line: u32, col: u32) -> Value {
            let resp = self.prepare_rename_raw(file, line, col);
            assert!(
                resp.get("error").is_none(),
                "prepareRename returned error: {:?}",
                resp.get("error")
            );
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

    /// RB-10: prepareRename returns defaultBehavior on a struct type.
    #[test]
    fn test_prepare_rename_struct_type() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // `type Point = ...;` — cursor on `Point` at line 9, col 5.
        let result = ctx.prepare_rename("lib.fix", 9, 5);
        assert!(
            result
                .get("defaultBehavior")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "expected defaultBehavior=true on a struct type, got: {:?}",
            result
        );
        ctx.shutdown();
    }

    // =======================================================================
    // rename_types fixture (lines are 0-indexed):
    //
    // lib.fix:
    //   3: type MyInt = I64;                (col 5 = `MyInt` decl)
    //   5: inc : MyInt -> MyInt;            (cols 6, 15)
    //   9: type Point = unbox struct { x : I64, y : I64 };
    //                                        (col 5 = `Point`, col 28 = `x`)
    //  12: mk_point = Point { x : 1, y : 2 };  (col 19 = `x`)
    //  15: get_x = |p| p.@x;                  (col 14 = `@x`)
    //  18: type Maybe a = box union {
    //  19:     some : a,                      (col 4 = `some` decl)
    //  25:     some(v) => v,                  (col 4 = `some` pattern)
    //  30: trait a : Greeter {                (col 10 = `Greeter` decl)
    //  34: impl Point : Greeter {             (col 13 = `Greeter` use)
    //  35:     greet = |p| p.@x;              (col 18 = `@x`)
    //
    // main.fix:
    //   2: import Lib::{MyInt, Greeter};    (col 13 = `MyInt`, col 20 = `Greeter`)
    //   4: bump : MyInt -> MyInt;           (cols 7, 16)
    // =======================================================================

    /// RT-1: rename a type alias from its declaration.
    #[test]
    fn test_rename_type_alias_decl() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 3, 5, "Counter");
        // decl + 2 in lib.fix type sig + 1 import + 2 in main.fix type sig = 6
        assert_eq!(count_edits(&we), 6, "WorkspaceEdit: {:?}", we);
        let per_file = changes_per_file(&we);
        assert_eq!(
            per_file,
            vec![("lib.fix".to_string(), 3), ("main.fix".to_string(), 3)]
        );
        assert_all_edits_have_new_text(&we, "Counter");
        ctx.shutdown();
    }

    /// RT-2: rename a trait.
    #[test]
    fn test_rename_trait() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Cursor on `Greeter` in the trait declaration (line 33, col 10).
        let we = ctx.rename("lib.fix", 33, 10, "Speaker");
        // decl + impl + import = 3
        assert_eq!(count_edits(&we), 3, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "Speaker");
        ctx.shutdown();
    }

    /// RT-3: rename a struct field. Auto-method occurrences (`@x`,
    /// `[^x]`) must switch to `@new_name` / `^new_name`, and the bare
    /// field-name (decl + MakeStruct) edits must use just `new_name`.
    #[test]
    fn test_rename_struct_field() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Cursor on the field-name `x` in the struct declaration (line 9, col 28).
        let we = ctx.rename("lib.fix", 9, 28, "horiz");
        // decl + MakeStruct + 2 getter calls + 1 index-syntax = 5.
        assert_eq!(count_edits(&we), 5, "WorkspaceEdit: {:?}", we);

        let changes = we.get("changes").unwrap().as_object().unwrap();
        let edits: Vec<&Value> = changes
            .values()
            .flat_map(|arr| arr.as_array().unwrap().iter())
            .collect();
        let new_texts: Vec<&str> = edits
            .iter()
            .map(|e| e.get("newText").and_then(|n| n.as_str()).unwrap())
            .collect();
        let at_count = new_texts.iter().filter(|s| **s == "@horiz").count();
        let caret_count = new_texts.iter().filter(|s| **s == "^horiz").count();
        let bare_count = new_texts.iter().filter(|s| **s == "horiz").count();
        assert_eq!(
            at_count, 2,
            "expected 2 `@horiz` edits, got {}: {:?}",
            at_count, new_texts
        );
        assert_eq!(
            caret_count, 1,
            "expected 1 `^horiz` edit (index syntax), got {}: {:?}",
            caret_count, new_texts
        );
        assert_eq!(
            bare_count, 2,
            "expected 2 bare `horiz` edits, got {}: {:?}",
            bare_count, new_texts
        );
        ctx.shutdown();
    }

    /// RT-4: rename a union variant. Pattern::Union and bare-name
    /// occurrences both update.
    #[test]
    fn test_rename_union_variant() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Cursor on the variant `some` declaration (line 22, col 4).
        let we = ctx.rename("lib.fix", 22, 4, "present");
        // decl + Pattern::Union usage = 2
        assert_eq!(count_edits(&we), 2, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "present");
        ctx.shutdown();
    }

    /// RT-5: rename a type alias from a use site in another file.
    #[test]
    fn test_rename_type_alias_from_other_file() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // `bump : MyInt -> MyInt;` — `MyInt` at line 4, col 7 in main.fix.
        let we = ctx.rename("main.fix", 4, 7, "Counter");
        assert_eq!(count_edits(&we), 6);
        ctx.shutdown();
    }

    /// RT-6: renaming a struct type renames every bare-name occurrence
    /// (declaration, MakeStruct, type sigs, impl blocks).
    #[test]
    fn test_rename_struct_type() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Cursor on `Point` at the struct declaration (line 9, col 5).
        let we = ctx.rename("lib.fix", 9, 5, "Pixel");
        // Bare token occurrences in lib.fix:
        //   line 9 decl, line 11 sig, line 12 MakeStruct ctor, line 14 sig (2),
        //   line 17 sig (2), line 37 impl. = 8
        assert!(count_edits(&we) >= 7, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "Pixel");
        ctx.shutdown();
    }

    /// RT-7: renaming a union type renames every bare-name occurrence.
    #[test]
    fn test_rename_union_type() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Cursor on `Maybe` at the union declaration (line 21, col 5).
        let we = ctx.rename("lib.fix", 21, 5, "Optional");
        // Bare-name occurrences: decl + 1 in `unwrap_default`'s sig = 2.
        assert_eq!(count_edits(&we), 2, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "Optional");
        ctx.shutdown();
    }

    /// RT-8: renaming a struct field to `@y` is rejected by the
    /// `type_field_name` rule (no leading `@`).
    #[test]
    fn test_rename_field_reject_at_prefix() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let resp = ctx.rename_raw("lib.fix", 9, 28, "@horiz");
        assert!(
            resp.get("error").is_some(),
            "expected rejection of `@`-prefixed field name, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RT-9: renaming a type alias to a lowercase name is rejected by the
    /// `capital_name` rule.
    #[test]
    fn test_rename_type_alias_reject_lowercase() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let resp = ctx.rename_raw("lib.fix", 3, 5, "counter");
        assert!(
            resp.get("error").is_some(),
            "expected rejection of lowercase name for a type, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RT-10: prepareRename returns defaultBehavior for a type alias.
    #[test]
    fn test_prepare_rename_type_alias() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("lib.fix", 3, 5);
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

    /// RT-11: prepareRename returns defaultBehavior for a trait.
    #[test]
    fn test_prepare_rename_trait() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("lib.fix", 33, 10);
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

    /// RT-12: prepareRename returns defaultBehavior for a struct field.
    #[test]
    fn test_prepare_rename_field() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("lib.fix", 9, 28);
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

    // =======================================================================
    // Gating: auto-method rejection, external-symbol rejection,
    // stale-buffer rejection.
    // =======================================================================

    /// RG-1: rename rejected on `@x` (auto-generated getter).
    #[test]
    fn test_rename_reject_at_accessor() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // `get_x = |p| p.@x;` — `@x` at line 15 col 14.
        let resp = ctx.rename_raw("lib.fix", 15, 14, "horiz");
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("auto-generated"),
            "expected auto-generated rejection, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-2: rename rejected on `[^x]` index syntax (the Var the parser
    /// generates is `Point::act_x`, also auto-generated).
    #[test]
    fn test_rename_reject_index_syntax() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // `set_x_zero = |p| p[^x].iset(0);` — `^x` at line 18 col 19.
        let resp = ctx.rename_raw("lib.fix", 18, 19, "horiz");
        assert!(
            resp.get("error").is_some(),
            "expected rejection on index-syntax cursor, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-3: prepareRename returns a ResponseError with an explanatory
    /// message on an auto-generated accessor (so the editor can surface
    /// a useful message instead of the generic "can't be renamed").
    #[test]
    fn test_prepare_rename_reject_at_accessor() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let resp = ctx.prepare_rename_raw("lib.fix", 15, 14);
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("auto-generated"),
            "expected auto-generated rejection on prepareRename, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-4: rename rejected on a Std symbol (defined outside the project).
    /// The cursor on `I64` in `type MyInt = I64;` resolves to `Std::I64`,
    /// which is not in the diagnostics result's `user_source_contents`.
    #[test]
    fn test_rename_reject_external_type() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // `type MyInt = I64;` — `I64` starts at line 3 col 13.
        let resp = ctx.rename_raw("lib.fix", 3, 13, "MyI64");
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("outside this project"),
            "expected external-symbol rejection, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-5: prepareRename returns a ResponseError on an external symbol.
    #[test]
    fn test_prepare_rename_reject_external() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let resp = ctx.prepare_rename_raw("lib.fix", 3, 13);
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("outside this project"),
            "expected external-symbol rejection on prepareRename, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-6: rename rejected after the buffer drifts from the AST.
    /// We send a didChange with modified text but don't trigger a rebuild,
    /// so the recorded `user_source_contents[lib.fix]` is now out of sync.
    #[test]
    fn test_rename_reject_stale_buffer() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        // Send a didChange that mutates lib.fix in memory.
        let uri = ctx.file_uri("lib.fix");
        let new_content =
            "module Lib;\n\n// stale-test\ntype MyInt = I64;\ninc : MyInt -> MyInt;\ninc = |x| x;\n";
        ctx.client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": 99 },
                    "contentChanges": [{ "text": new_content }],
                }),
            )
            .expect("Failed to send didChange");
        // Give the server a moment to process the notification (no
        // diagnostic re-run is triggered, so the AST stays stale).
        ctx.client.wait_for_server(std::time::Duration::from_millis(300));

        let resp = ctx.rename_raw("lib.fix", 3, 5, "Counter");
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("edited since the last successful build"),
            "expected stale-buffer rejection, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    /// RG-7: prepareRename returns a ResponseError when the buffer is
    /// stale, so the editor can show the actionable message ("save and
    /// wait for diagnostics").
    #[test]
    fn test_prepare_rename_reject_stale_buffer() {
        let mut ctx = LspTestCtx::setup("rename_types", &["lib.fix", "main.fix"]);
        let uri = ctx.file_uri("lib.fix");
        let new_content = "module Lib;\n";
        ctx.client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": 99 },
                    "contentChanges": [{ "text": new_content }],
                }),
            )
            .expect("Failed to send didChange");
        ctx.client.wait_for_server(std::time::Duration::from_millis(300));

        let resp = ctx.prepare_rename_raw("lib.fix", 3, 5);
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");
        assert!(
            msg.contains("edited since the last successful build"),
            "expected stale-buffer rejection on prepareRename, got: {:?}",
            resp
        );
        ctx.shutdown();
    }

    // =======================================================================
    // Struct/union type rename with auto-namespace coupling.
    //
    // rename_struct_type fixture (0-indexed):
    // lib.fix:
    //   2: type Point = unbox struct { x : I64, y : I64 };
    //   7: namespace Point {                       (col 10 = `Point`)
    //   8:     user_helper : Point -> I64;         (col 18 = `Point`)
    //   9:     user_helper = |p| p.@x + p.@y;
    //  10: }
    //  12: mk_point : Point;                       (col 11 = `Point`)
    //  13: mk_point = Point { x : 1, y : 2 };      (col 11 = `Point`)
    //
    // main.fix:
    //   5: import Lib::{Point, Point::{@x, act_x}};
    //                  ^col 14 = `Point` (TypeOrTrait)
    //                         ^col 21 = `Point` (NameSpace)
    //   8: qualified_get : Point -> I64;           (col 17 = `Point`)
    //   9: qualified_get = |p| Point::@x(p);       (col 22 = inline `Point`)
    //  12: qualified_idx : Point -> Point;
    //  13: qualified_idx = |p| p[^Point::x].iset(0);  (col 23 = inline `Point`)
    // =======================================================================

    /// RD-1: rename a struct type and observe that all bare uses, the
    /// auto-namespace component in the all-auto import, and the inline
    /// qualified Var references are all rewritten.
    #[test]
    fn test_rename_struct_type_phase_d() {
        let mut ctx =
            LspTestCtx::setup("rename_struct_type", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 5, "Pixel");

        let per_file = changes_per_file(&we);
        assert_eq!(
            per_file.iter().map(|(_, n)| n).sum::<usize>(),
            count_edits(&we)
        );

        let lib_count = per_file
            .iter()
            .find(|(f, _)| f == "lib.fix")
            .map(|(_, n)| *n)
            .unwrap_or(0);
        let main_count = per_file
            .iter()
            .find(|(f, _)| f == "main.fix")
            .map(|(_, n)| *n)
            .unwrap_or(0);

        // lib.fix bare-name occurrences: decl + user_helper sig + mk_point
        // sig + MakeStruct = 4.
        assert_eq!(lib_count, 4, "lib.fix edits: {:?}", we);

        // main.fix: 4 bare (one TypeOrTrait import + 3 type sigs) +
        // 1 NameSpace import + 2 inline qualified = 7.
        assert_eq!(main_count, 7, "main.fix edits: {:?}", we);

        // Every edit's new_text should be just `Pixel` (the rebuilt-import
        // case is not triggered here because the namespace import is all-auto).
        assert_all_edits_have_new_text(&we, "Pixel");
        ctx.shutdown();
    }

    /// RD-2: the user-defined namespace block in lib.fix
    /// (`namespace Point { ... }`) must NOT be touched, because its
    /// `Point` is a user-written namespace name, independent of the type.
    #[test]
    fn test_rename_struct_type_skips_user_namespace_block() {
        let mut ctx =
            LspTestCtx::setup("rename_struct_type", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 5, "Pixel");

        // Verify no edit lands on line 7 col 10 (the `namespace Point {`
        // declaration).
        let changes = we.get("changes").unwrap().as_object().unwrap();
        for (_uri, arr) in changes {
            for edit in arr.as_array().unwrap() {
                let start = edit.get("range").unwrap().get("start").unwrap();
                let line = start.get("line").unwrap().as_u64().unwrap();
                let ch = start.get("character").unwrap().as_u64().unwrap();
                assert!(
                    !(line == 7 && ch == 10),
                    "should not rewrite the `namespace Point {{` declaration"
                );
            }
        }
        ctx.shutdown();
    }

    /// RD-3: the inline qualified reference `Point::@x` in main.fix has
    /// just its `Point` sub-span rewritten to `Pixel`, leaving `::@x`.
    /// We verify by reading the post-edit text at the reported range.
    #[test]
    fn test_rename_struct_type_inline_qualified_var() {
        let mut ctx =
            LspTestCtx::setup("rename_struct_type", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 5, "Pixel");

        // Find an edit on main.fix line 9 (qualified_get's body) that
        // covers exactly 5 chars (= length of "Point").
        let changes = we.get("changes").unwrap().as_object().unwrap();
        let mut found = false;
        for (uri, arr) in changes {
            if !uri.contains("main.fix") {
                continue;
            }
            for edit in arr.as_array().unwrap() {
                let r = edit.get("range").unwrap();
                let s = r.get("start").unwrap();
                let e = r.get("end").unwrap();
                if s.get("line").unwrap().as_u64() == Some(9)
                    && e.get("line").unwrap().as_u64() == Some(9)
                {
                    let s_ch = s.get("character").unwrap().as_u64().unwrap();
                    let e_ch = e.get("character").unwrap().as_u64().unwrap();
                    if e_ch - s_ch == 5 {
                        found = true;
                        assert_eq!(
                            edit.get("newText").unwrap().as_str(),
                            Some("Pixel"),
                            "inline qualified Point edit should be 'Pixel'"
                        );
                    }
                }
            }
        }
        assert!(found, "expected inline qualified `Point` edit on line 9");
        ctx.shutdown();
    }

    /// RD-4: the qualified index syntax `[^Point::x]` has its `Point`
    /// sub-span rewritten too — Var.source covers `^Point::x`, so the
    /// `^` is skipped during sub-span extraction.
    #[test]
    fn test_rename_struct_type_inline_index_syntax() {
        let mut ctx =
            LspTestCtx::setup("rename_struct_type", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 5, "Pixel");

        // Look for an edit on main.fix line 13 covering 5 chars
        // (= length of "Point") with new_text "Pixel".
        let changes = we.get("changes").unwrap().as_object().unwrap();
        let mut found = false;
        for (uri, arr) in changes {
            if !uri.contains("main.fix") {
                continue;
            }
            for edit in arr.as_array().unwrap() {
                let r = edit.get("range").unwrap();
                let s = r.get("start").unwrap();
                let e = r.get("end").unwrap();
                if s.get("line").unwrap().as_u64() == Some(13)
                    && e.get("line").unwrap().as_u64() == Some(13)
                    && e.get("character").unwrap().as_u64().unwrap()
                        - s.get("character").unwrap().as_u64().unwrap()
                        == 5
                    && edit.get("newText").unwrap().as_str() == Some("Pixel")
                {
                    found = true;
                }
            }
        }
        assert!(
            found,
            "expected inline `^Point::x` -> `^Pixel::x` edit on line 13"
        );
        ctx.shutdown();
    }

    /// RD-5: prepareRename returns defaultBehavior on a struct type.
    #[test]
    fn test_prepare_rename_struct_type_phase_d() {
        let mut ctx =
            LspTestCtx::setup("rename_struct_type", &["lib.fix", "main.fix"]);
        let result = ctx.prepare_rename("lib.fix", 2, 5);
        assert!(
            result
                .get("defaultBehavior")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "expected defaultBehavior=true on struct type, got: {:?}",
            result
        );
        ctx.shutdown();
    }

    /// RD-6 regression: a qualified call to a user-defined helper
    /// (`MinCostFlowGraph::create(...)`) sitting in the type's
    /// namespace must NOT have its `MinCostFlowGraph::` prefix rewritten
    /// when the type is renamed. Only auto-generated accessors travel
    /// with the type.
    #[test]
    fn test_rename_struct_type_skips_user_helper_qualified_call() {
        let mut ctx = LspTestCtx::setup(
            "rename_user_helper_qualified",
            &["lib.fix", "main.fix"],
        );
        let we = ctx.rename("lib.fix", 2, 5, "MCFGraph");

        // Inspect every edit: any change inside main.fix's body must
        // be a bare-token rename, never a sub-span rewrite of
        // `MinCostFlowGraph::create`.
        let changes = we.get("changes").unwrap().as_object().unwrap();
        for (uri, arr) in changes {
            if !uri.contains("main.fix") {
                continue;
            }
            for edit in arr.as_array().unwrap() {
                let r = edit.get("range").unwrap();
                let line = r["start"]["line"].as_u64().unwrap();
                let col = r["start"]["character"].as_u64().unwrap();
                let new_text = edit.get("newText").and_then(|v| v.as_str()).unwrap();
                // The `make_graph = MinCostFlowGraph::create(10);` line
                // is line 9. The qualifier `MinCostFlowGraph` here must
                // not be touched, so no edit should land on its column
                // (col 13). The only valid edits on line 9 are on the
                // type-annotation `MinCostFlowGraph` of line 8.
                assert!(
                    !(line == 9 && col == 13),
                    "should not rewrite `MinCostFlowGraph::` qualifier in user-helper call, got edit: {:?}",
                    edit
                );
                // And we definitely shouldn't be writing `MCFGraph::create`
                // anywhere as a single new_text.
                assert!(
                    !new_text.contains("MCFGraph::create"),
                    "edit should not produce `MCFGraph::create` text: {:?}",
                    edit
                );
            }
        }
        ctx.shutdown();
    }

    /// RD-7: a mixed import (`Lib::{Point::{act_x, user_helper}}`) is
    /// rebuilt as a single TextEdit covering the entire import statement.
    /// The new text must contain both `Pixel::` (for the auto-method
    /// half) and `Point::` (for the user-defined half).
    #[test]
    fn test_rename_struct_type_mixed_import_split() {
        let mut ctx =
            LspTestCtx::setup("rename_mixed_import", &["lib.fix", "main.fix"]);
        let we = ctx.rename("lib.fix", 2, 5, "Pixel");

        // Find the rebuilt import edit on main.fix line 6 (the import
        // statement). It should cover the whole `import ...;` line.
        let changes = we.get("changes").unwrap().as_object().unwrap();
        let main_edits = changes
            .iter()
            .find(|(k, _)| k.contains("main.fix"))
            .map(|(_, v)| v.as_array().unwrap().clone())
            .expect("main.fix should have edits");

        // At least one edit on line 6 must be the whole-import rebuild
        // and contain both `Pixel::` and `Point::` in its newText.
        let mut found_split = false;
        for edit in main_edits {
            let r = edit.get("range").unwrap();
            let s_line = r["start"]["line"].as_u64().unwrap();
            let e_line = r["end"]["line"].as_u64().unwrap();
            let text = edit.get("newText").unwrap().as_str().unwrap();
            if s_line == 6 && e_line == 6 && text.contains("Pixel") && text.contains("Point") {
                // Must contain both halves.
                assert!(
                    text.contains("act_x"),
                    "expected `act_x` under `Pixel::` in {:?}",
                    text
                );
                assert!(
                    text.contains("user_helper"),
                    "expected `user_helper` under `Point::` in {:?}",
                    text
                );
                found_split = true;
            }
        }
        assert!(
            found_split,
            "did not find a split-rebuilt import on line 6 in {:?}",
            we
        );
        ctx.shutdown();
    }

    /// Renaming a global value should also rewrite the name token inside
    /// `FFI_EXPORT[...]` and `DEPRECATED[...]` pragmas. Cursor on the
    /// declaration LHS at line 2, col 0.
    #[test]
    fn test_rename_pragma_names() {
        let mut ctx = LspTestCtx::setup("rename_pragmas", &["main.fix"]);
        let we = ctx.rename("main.fix", 2, 0, "fresh_func");

        // Expected edits:
        //   line 2: decl LHS
        //   line 3: defn LHS
        //   line 4: DEPRECATED target name
        //   line 5: FFI_EXPORT value name
        // = 4 edits, all in main.fix.
        assert_eq!(count_edits(&we), 4, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "fresh_func");
        ctx.shutdown();
    }

    /// Cursor on the name inside `DEPRECATED[old_func, ...]` should be a
    /// rename target equivalent to renaming the declaration itself.
    #[test]
    fn test_rename_from_deprecated_pragma_name() {
        let mut ctx = LspTestCtx::setup("rename_pragmas", &["main.fix"]);
        // Line 4: `DEPRECATED[old_func, "Use new_func."];`
        // Cursor on `old_func` at col 11 (after `DEPRECATED[`).
        let we = ctx.rename("main.fix", 4, 11, "fresh_func");
        assert_eq!(count_edits(&we), 4, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "fresh_func");
        ctx.shutdown();
    }

    /// Same as above but cursor on the name inside `FFI_EXPORT[...]`.
    #[test]
    fn test_rename_from_ffi_export_pragma_name() {
        let mut ctx = LspTestCtx::setup("rename_pragmas", &["main.fix"]);
        // Line 5: `FFI_EXPORT[old_func, c_old_func];`
        // Cursor on `old_func` at col 11.
        let we = ctx.rename("main.fix", 5, 11, "fresh_func");
        assert_eq!(count_edits(&we), 4, "WorkspaceEdit: {:?}", we);
        assert_all_edits_have_new_text(&we, "fresh_func");
        ctx.shutdown();
    }
}
