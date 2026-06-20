// LSP integration tests for `textDocument/semanticTokens/full`.
//
// Covers the two layers of the feature end-to-end, through a real
// `fix language-server` subprocess:
//   * the base lexical layer must respond even on a broken / drifted buffer
//     (highlighting never disappears mid-edit);
//   * the AST overlay must, once the file type-checks, distinguish local
//     identifiers (variable) from global ones (function).

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::{copy_dir_recursive, install_fix};
    use serde_json::json;
    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };
    use tempfile::TempDir;

    // Token-type indices, matching the legend in
    // `commands::lsp::semantic_tokens::legend`.
    const T_TYPE: u64 = 1;
    const T_VARIABLE: u64 = 2;
    const T_KEYWORD: u64 = 3;
    const T_STRING: u64 = 5;
    const T_COMMENT: u64 = 6;
    const T_FUNCTION: u64 = 8;
    const T_ENUM_MEMBER: u64 = 9;
    const T_PROPERTY: u64 = 10;
    const T_TYPE_PARAMETER: u64 = 11;
    const T_STRUCT: u64 = 12;
    const T_ENUM: u64 = 13;
    const T_INTERFACE: u64 = 14;

    /// The directory holding the LSP test-case projects.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    /// Copy the named test-case project into a fresh temp directory so tests can
    /// run in parallel. Returns the temp dir (kept alive for cleanup) and the
    /// canonical path to the copied project.
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

    /// A running language server connected to a copied test project.
    struct Ctx {
        /// The client driving the `fix language-server` subprocess.
        client: LspClient,
        /// The copied project's root directory.
        project_dir: PathBuf,
        /// Held only to keep the temp directory alive for the test's duration.
        _temp_dir: TempDir,
    }

    impl Ctx {
        /// Start a server on a fresh copy of the `semantic_tokens` project, open
        /// `main.fix`, and wait for an initial elaboration.
        fn setup() -> Self {
            install_fix();
            let (temp_dir, project_dir) = setup_test_env("semantic_tokens");
            let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
            client
                .initialize(&project_dir, Duration::from_secs(5))
                .expect("Failed to initialize LSP");
            client
                .open_document(Path::new("main.fix"))
                .expect("Failed to open main.fix");
            // Elaborate the project so the AST overlay has a program whose
            // snapshot matches the (unmodified) buffer.
            client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));
            Self {
                client,
                project_dir,
                _temp_dir: temp_dir,
            }
        }

        /// The `file://` URI for a project-relative file path.
        fn file_uri(&self, file: &str) -> String {
            format!("file://{}", self.project_dir.join(file).display())
        }

        /// Request semantic tokens and return the flat list of per-token type
        /// indices (the 4th element of each 5-tuple in the delta-encoded data).
        /// Polls for the response rather than sleeping a fixed time.
        fn token_types(&mut self, file: &str) -> Vec<u64> {
            let uri = self.file_uri(file);
            let id = self
                .client
                .send_request(
                    "textDocument/semanticTokens/full",
                    json!({ "textDocument": { "uri": uri } }),
                )
                .expect("Failed to send semanticTokens request");
            let mut response = None;
            for _ in 0..50 {
                if let Some(r) = self.client.get_response(id) {
                    response = Some(r);
                    break;
                }
                self.client.wait_for_server(Duration::from_millis(100));
            }
            let response = response.expect("Should receive a semanticTokens response");
            let data = response
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.as_array())
                .expect("Response should have result.data");
            let nums: Vec<u64> = data.iter().filter_map(|v| v.as_u64()).collect();
            assert_eq!(
                nums.len() % 5,
                0,
                "token data length must be a multiple of 5"
            );
            nums.chunks_exact(5).map(|c| c[3]).collect()
        }

        /// Like `token_types`, but waits for the AST overlay to be applied: the
        /// diagnostics result can be picked up by the server's main loop slightly
        /// after the progress-end notification, so retry until a typechecked
        /// token (a local variable, which only the overlay emits) appears.
        fn token_types_with_overlay(&mut self, file: &str) -> Vec<u64> {
            for _ in 0..40 {
                let types = self.token_types(file);
                if types.contains(&T_VARIABLE) {
                    return types;
                }
                self.client.wait_for_server(Duration::from_millis(250));
            }
            self.token_types(file)
        }

        /// Replace the whole content of `file` via a `didChange` notification.
        fn change_text(&mut self, file: &str, text: &str) {
            let uri = self.file_uri(file);
            self.client
                .send_notification(
                    "textDocument/didChange",
                    json!({
                        "textDocument": { "uri": uri, "version": 99 },
                        "contentChanges": [ { "text": text } ]
                    }),
                )
                .expect("Failed to send didChange");
            self.client.wait_for_server(Duration::from_millis(300));
        }

        /// Shut the server down cleanly and join its reader thread.
        fn shutdown(mut self) {
            self.client
                .shutdown(Duration::from_millis(500))
                .expect("Failed to shutdown LSP");
            self.client
                .finish()
                .expect("Reader thread should not error");
        }
    }

    /// Verifies that once the file type-checks, the overlay classifies symbols
    /// precisely: locals vs globals, struct/union/trait names, type variables,
    /// union variants and field accessors — while the base layer keeps coloring
    /// comments, strings, keywords and built-in types.
    #[test]
    fn semantic_tokens_overlay_precise_classification() {
        let mut ctx = Ctx::setup();
        let types = ctx.token_types_with_overlay("main.fix");

        let want = [
            (T_VARIABLE, "local variable"),      // p, s, n, x, y
            (T_FUNCTION, "global function"),     // println, size, pair
            (T_STRUCT, "struct type"),           // Point
            (T_ENUM, "union type"),              // Shape
            (T_INTERFACE, "trait"),              // Sizer
            (T_TYPE_PARAMETER, "type variable"), // a, b in `pair`
            (T_ENUM_MEMBER, "union variant"),    // dot, seg
            (T_PROPERTY, "field accessor"),      // @x, @y
            (T_KEYWORD, "keyword"),              // let, type, trait, ...
            (T_TYPE, "built-in type"),           // I64
            (T_STRING, "string"),                // "done"
            (T_COMMENT, "comment"),
        ];
        for (t, label) in want {
            assert!(
                types.contains(&t),
                "expected {} (type {}) token, got types: {:?}",
                label,
                t,
                types
            );
        }

        ctx.shutdown();
    }

    /// Verifies that on a broken / drifted buffer the server still responds with
    /// the base lexical layer, and does NOT emit the AST overlay (which would be
    /// misaligned), so no variable/function tokens appear.
    #[test]
    fn semantic_tokens_base_layer_survives_broken_buffer() {
        let mut ctx = Ctx::setup();

        // Drift the buffer away from the elaborated snapshot, with broken
        // syntax (unbalanced paren, unterminated string).
        ctx.change_text(
            "main.fix",
            "module Main;\nadd = |a, b| a + (\n  let q = \"unterminated\n",
        );
        let types = ctx.token_types("main.fix");

        assert!(
            !types.is_empty(),
            "base layer should still produce tokens on a broken buffer"
        );
        assert!(
            types.contains(&T_KEYWORD),
            "base layer should color keywords on a broken buffer, got: {:?}",
            types
        );
        // The overlay must be gated off when the buffer no longer matches the
        // snapshot, so no identifier coloring leaks through.
        assert!(
            !types.contains(&T_FUNCTION) && !types.contains(&T_VARIABLE),
            "overlay must not apply to a drifted buffer, got: {:?}",
            types
        );

        ctx.shutdown();
    }

    /// Verifies that editing one line does not drop the whole file to the base
    /// layer: the AST overlay is kept (per-line) on every line that is unchanged
    /// from the elaborated snapshot, even though the buffer no longer matches it
    /// exactly.
    #[test]
    fn semantic_tokens_overlay_survives_single_line_edit() {
        let mut ctx = Ctx::setup();
        // Apply the overlay first.
        let _ = ctx.token_types_with_overlay("main.fix");

        // Edit a single body line; the type/trait/struct definitions on other
        // lines are untouched.
        let original =
            std::fs::read_to_string(ctx.project_dir.join("main.fix")).expect("read main.fix");
        let edited = original.replace("let n = p.size;", "let n = p.size; // tweak");
        assert_ne!(original, edited, "the edit should change the buffer");
        ctx.change_text("main.fix", &edited);

        let types = ctx.token_types_with_overlay("main.fix");
        for (t, label) in [
            (T_STRUCT, "struct"),
            (T_ENUM, "enum"),
            (T_INTERFACE, "interface"),
            (T_TYPE_PARAMETER, "type parameter"),
            (T_VARIABLE, "variable"),
            (T_FUNCTION, "function"),
        ] {
            assert!(
                types.contains(&t),
                "overlay {} should survive a single-line edit, got: {:?}",
                label,
                types
            );
        }

        ctx.shutdown();
    }

    /// Verifies that when diagnostics finish, the server prompts the client to
    /// re-request semantic tokens (`workspace/semanticTokens/refresh`);
    /// otherwise the client keeps the base-layer-only result it fetched before
    /// elaboration completed and the overlay never appears.
    #[test]
    fn semantic_tokens_refresh_sent_after_diagnostics() {
        let mut ctx = Ctx::setup();

        let mut saw_refresh = false;
        while let Some(msg) = ctx.client.pop_message() {
            if msg.get("method").and_then(|m| m.as_str())
                == Some("workspace/semanticTokens/refresh")
            {
                saw_refresh = true;
                break;
            }
        }
        assert!(
            saw_refresh,
            "server should send workspace/semanticTokens/refresh after diagnostics complete"
        );

        ctx.shutdown();
    }
}
