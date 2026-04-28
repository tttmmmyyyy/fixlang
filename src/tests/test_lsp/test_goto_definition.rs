// LSP integration tests for `textDocument/definition`, focused on
// jumping from a local-name use to its binder (let / match / lambda /
// struct-destructure), plus a regression on global values.

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

        /// Send textDocument/definition and return the result value (the LSP
        /// server returns either a single Location or null).
        fn goto_definition(&mut self, file: &str, line: u32, col: u32) -> Value {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/definition",
                    json!({
                        "textDocument": { "uri": uri },
                        "position": { "line": line, "character": col }
                    }),
                )
                .expect("Failed to send definition request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a definition response");
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

    /// Read the substring of `file` covered by an LSP range.
    fn read_text_at_range(file: &Path, range: &Value) -> String {
        let content = std::fs::read_to_string(file)
            .expect(&format!("Failed to read file: {:?}", file));
        let lines: Vec<&str> = content.lines().collect();
        let sl = range["start"]["line"].as_u64().unwrap() as usize;
        let sc = range["start"]["character"].as_u64().unwrap() as usize;
        let el = range["end"]["line"].as_u64().unwrap() as usize;
        let ec = range["end"]["character"].as_u64().unwrap() as usize;
        if sl == el {
            lines[sl][sc..ec].to_string()
        } else {
            let mut text = lines[sl][sc..].to_string();
            for i in (sl + 1)..el {
                text.push('\n');
                text.push_str(lines[i]);
            }
            text.push('\n');
            text.push_str(&lines[el][..ec]);
            text
        }
    }

    /// Assert the jump landed on the given file at the expected line/column,
    /// and that the range's text equals `expected_text`.
    fn assert_location(
        result: &Value,
        ctx: &LspTestCtx,
        file: &str,
        line: u32,
        col: u32,
        expected_text: &str,
    ) {
        assert!(
            result.is_object(),
            "Expected a Location, got {:?}",
            result
        );
        let uri = result["uri"].as_str().expect("Location should have uri");
        let expected_uri = ctx.file_uri(file);
        assert_eq!(
            uri, expected_uri,
            "Definition should be in {}, got {}",
            file, uri
        );

        let range = result.get("range").expect("Location should have range");
        let actual_line = range["start"]["line"].as_u64().unwrap() as u32;
        let actual_col = range["start"]["character"].as_u64().unwrap() as u32;
        assert_eq!(
            (actual_line, actual_col),
            (line, col),
            "Definition start should be at ({}, {}), got ({}, {})",
            line,
            col,
            actual_line,
            actual_col
        );

        let file_path = PathBuf::from(uri.strip_prefix("file://").unwrap());
        let text = read_text_at_range(&file_path, range);
        assert_eq!(
            text, expected_text,
            "Text at definition range should be `{}`, got `{}`",
            expected_text, text
        );
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Simple let: cursor on `x` in `x + 1` should jump to the `x` binder.
    #[test]
    fn test_goto_local_simple_let() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 10, col 4.
        let result = ctx.goto_definition("lib.fix", 10, 4);
        // Binder at line 9, col 8.
        assert_location(&result, &ctx, "lib.fix", 9, 8, "x");
        ctx.shutdown();
    }

    /// Lambda argument: cursor on `a` in `a + 1` should jump to `|a|`.
    #[test]
    fn test_goto_local_lambda_arg() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 16, col 14.
        let result = ctx.goto_definition("lib.fix", 16, 14);
        // Binder (the `a` inside `|a|`) at line 16, col 11.
        assert_location(&result, &ctx, "lib.fix", 16, 11, "a");
        ctx.shutdown();
    }

    /// Match arm (union payload): cursor on `h` after `=>` jumps to `one(h)` binder.
    #[test]
    fn test_goto_local_match_union_arm() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 27, col 18.
        let result = ctx.goto_definition("lib.fix", 27, 18);
        // Binder at line 27, col 12.
        assert_location(&result, &ctx, "lib.fix", 27, 12, "h");
        ctx.shutdown();
    }

    /// Match arm (tuple pattern): cursor on `hh` after `=>` jumps to tuple binder.
    #[test]
    fn test_goto_local_match_tuple_arm() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 28, col 24.
        let result = ctx.goto_definition("lib.fix", 28, 24);
        // Binder at line 28, col 13.
        assert_location(&result, &ctx, "lib.fix", 28, 13, "hh");
        ctx.shutdown();
    }

    /// Struct-destructuring `let Point { px : a, py : b }`: cursor on `a` use → struct-field binder.
    #[test]
    fn test_goto_local_struct_destructure() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 41, col 4.
        let result = ctx.goto_definition("lib.fix", 41, 4);
        // Binder at line 40, col 21.
        assert_location(&result, &ctx, "lib.fix", 40, 21, "a");
        ctx.shutdown();
    }

    /// Shadowing: cursor on `s` inside the inner scope should resolve to the inner `let s = 2`.
    #[test]
    fn test_goto_local_shadowing() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Use at line 52, col 4.
        let result = ctx.goto_definition("lib.fix", 52, 4);
        // Inner binder at line 51, col 8 (NOT the outer at line 50).
        assert_location(&result, &ctx, "lib.fix", 51, 8, "s");
        ctx.shutdown();
    }

    // --- Repro: source span missing on `&&`-desugared `if` ---
    //
    // `parse_expr_and` (parser.rs:1342) builds the synthesized
    // `expr_if(lhs, rhs, expr_bool_lit(false, None), None)` with
    // `source: None`. `ExprNode::find_node_at` short-circuits on a
    // None source, so anything underneath the synthesized If is
    // unreachable — including the LHS and RHS sub-expressions of
    // `&&`. Hover, goto-definition, and find-references all break
    // there. The outer `if`'s THEN/ELSE branches are not affected
    // because they sit on the outer (user-written) `if`.

    /// Cursor on `b` of `b >= 0` (LHS of `&&`).
    #[test]
    fn test_goto_local_and_lhs() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // First `b` in `    if b >= 0 && b < 10 { b } else { 0 - b }` (line 77, col 7).
        let result = ctx.goto_definition("lib.fix", 77, 7);
        // Should jump to `let b = 5 in` binder at line 76, col 8.
        assert_location(&result, &ctx, "lib.fix", 76, 8, "b");
        ctx.shutdown();
    }

    /// Cursor on `b` of `b < 10` (RHS of `&&`).
    #[test]
    fn test_goto_local_and_rhs() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // Second `b` (line 77, col 17).
        let result = ctx.goto_definition("lib.fix", 77, 17);
        assert_location(&result, &ctx, "lib.fix", 76, 8, "b");
        ctx.shutdown();
    }

    /// Sanity: cursor on `b` inside `{ b }` (outer If's THEN branch) — works.
    #[test]
    fn test_goto_local_and_then_branch() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // `b` in `{ b }` (line 77, col 26).
        let result = ctx.goto_definition("lib.fix", 77, 26);
        assert_location(&result, &ctx, "lib.fix", 76, 8, "b");
        ctx.shutdown();
    }

    /// Regression: clicking a global name still jumps via `decl_src`.
    #[test]
    fn test_goto_global_regression() {
        let mut ctx = LspTestCtx::setup("goto_local", &["lib.fix"]);
        // `simple_let` use at line 66, col 13.
        let result = ctx.goto_definition("lib.fix", 66, 13);
        // The global's declaration is on line 7 (`simple_let : I64;`) starting at col 0.
        assert!(result.is_object(), "Expected a Location, got {:?}", result);
        let uri = result["uri"].as_str().unwrap();
        assert_eq!(uri, ctx.file_uri("lib.fix"));
        let range = result.get("range").unwrap();
        let file_path = PathBuf::from(uri.strip_prefix("file://").unwrap());
        let text = read_text_at_range(&file_path, range);
        assert_eq!(text, "simple_let", "Expected to land on `simple_let` decl");
        ctx.shutdown();
    }

    /// Cursor on a name inside `DEPRECATED[...]` should jump to the decl.
    #[test]
    fn test_goto_from_deprecated_pragma_name() {
        let mut ctx = LspTestCtx::setup("rename_pragmas", &["main.fix"]);
        // Line 4: `DEPRECATED[old_func, "Use new_func."];`
        let result = ctx.goto_definition("main.fix", 4, 11);
        assert!(result.is_object(), "Expected a Location, got {:?}", result);
        let uri = result["uri"].as_str().unwrap();
        let range = result.get("range").unwrap();
        let file_path = PathBuf::from(uri.strip_prefix("file://").unwrap());
        let text = read_text_at_range(&file_path, range);
        assert_eq!(text, "old_func");
        ctx.shutdown();
    }

    /// Cursor on a name inside `FFI_EXPORT[...]` should jump to the decl.
    #[test]
    fn test_goto_from_ffi_export_pragma_name() {
        let mut ctx = LspTestCtx::setup("rename_pragmas", &["main.fix"]);
        // Line 5: `FFI_EXPORT[old_func, c_old_func];`
        let result = ctx.goto_definition("main.fix", 5, 11);
        assert!(result.is_object(), "Expected a Location, got {:?}", result);
        let uri = result["uri"].as_str().unwrap();
        let range = result.get("range").unwrap();
        let file_path = PathBuf::from(uri.strip_prefix("file://").unwrap());
        let text = read_text_at_range(&file_path, range);
        assert_eq!(text, "old_func");
        ctx.shutdown();
    }
}
