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

        /// Send completionItem/resolve and return the resolved item.
        fn resolve(&mut self, item: Value) -> Value {
            let id = self
                .client
                .send_request("completionItem/resolve", item)
                .expect("Failed to send resolve request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a resolve response");
            response
                .get("result")
                .cloned()
                .expect("Resolve response should have result")
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

    /// Snapshot of the completion *insertion* behavior. Pins down what
    /// the server sends back for `completionItem/resolve` for the four
    /// shapes that have so far been verified manually:
    ///
    ///   typing            expected insert_text         notes
    ///   ----              --------------------         -----
    ///   `func`            `func(${1:?x}, ${2:?y})`     plain identifier
    ///   `y.func`          `func(${1:?x})`              dot-call drops last param
    ///   `Hoge::func`      `func(${1:?x}, ${2:?y})`     qualified
    ///   `y.Hoge::func`    `func(${1:?x})`              dot-call + qualified
    ///
    /// Note the `insert_text` is just the part the client splices in over
    /// the completed identifier — the namespace prefix the user already
    /// typed stays as-is on the source side. Param names `x` / `y` come
    /// from the `# Parameters` section of the doc comment on
    /// `Hoge::func`. Each name is wrapped in `?` so the inserted text is
    /// a user-hole expression (`?x` / `?y`) — the source therefore
    /// elaborates with `Std::#hole` placeholders that produce ERR_HOLE
    /// rather than "undefined name `x`" diagnostics. The `${N:...}` LSP
    /// snippet syntax additionally tells supporting clients to put the
    /// cursor on the first hole and let Tab move it to the next one,
    /// with each placeholder pre-selected so typing overwrites it.
    #[test]
    fn test_completion_insert_text_for_function_with_two_params() {
        let mut ctx = LspCompletionCtx::setup("completion_insert", &["main.fix"]);

        // The fixture file `main.fix` contains the following lines (0-indexed):
        //   13:     let _ = func;            // cursor right after `func` -> col 16
        //   14:     let _ = y.func;          // cursor right after `func` -> col 18
        //   15:     let _ = Hoge::func;      // cursor right after `func` -> col 22
        //   16:     let _ = y.Hoge::func;    // cursor right after `func` -> col 24
        let cases = [
            (13u32, 16u32, "func(${1:?x}, ${2:?y})", "plain identifier"),
            (14, 18, "func(${1:?x})", "dot-call drops last param"),
            (15, 22, "func(${1:?x}, ${2:?y})", "qualified identifier"),
            (16, 24, "func(${1:?x})", "dot-call + qualified"),
        ];

        for (line, col, expected_insert, label) in cases {
            let items = ctx.complete("main.fix", line, col);

            // Find the candidate for `Main::Hoge::func`.
            let item = items
                .iter()
                .find(|it| {
                    it.get("label").and_then(|l| l.as_str()) == Some("Main::Hoge::func")
                })
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "[{}] Expected `Main::Hoge::func` in completion candidates at \
                         line {}, col {}. Got labels: {:?}",
                        label,
                        line,
                        col,
                        items
                            .iter()
                            .filter_map(|it| it.get("label").and_then(|l| l.as_str()))
                            .collect::<Vec<_>>()
                    )
                });

            // Resolve to fetch the final insert_text (the initial response only
            // sets it to the bare name; resolve is what appends the `(x, y)`).
            let resolved = ctx.resolve(item);
            let actual_insert = resolved
                .get("insertText")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| {
                    panic!(
                        "[{}] Resolved item missing insertText. Got: {}",
                        label, resolved
                    )
                });

            assert_eq!(
                actual_insert, expected_insert,
                "[{}] insertText mismatch at line {}, col {}",
                label, line, col
            );

            // `insertTextFormat` must be `Snippet` (= 2) so the editor
            // expands the `${N:?x}` tab-stops; otherwise the placeholder
            // syntax would be inserted as literal text.
            assert_eq!(
                resolved.get("insertTextFormat").and_then(|v| v.as_i64()),
                Some(2),
                "[{}] insertTextFormat should be Snippet (2) at line {}, col {}",
                label,
                line,
                col
            );
        }

        ctx.shutdown();
    }

    /// `Hoge::old_func` carries a `DEPRECATED[...]` pragma in the fixture.
    /// Its completion item should advertise that to the client via both the
    /// legacy `deprecated: true` boolean (LSP <3.15) and the modern
    /// `tags: [Deprecated]` field (LSP >=3.15) so editors render the
    /// strikethrough in the candidate list. The deprecation message must
    /// also reach the resolved item's documentation so users see *why* the
    /// symbol is discouraged.
    #[test]
    fn test_completion_marks_deprecated_symbols() {
        let mut ctx = LspCompletionCtx::setup("completion_deprecated", &["main.fix"]);

        // Position is irrelevant for "list everything"; use line 0, col 0.
        let items = ctx.complete("main.fix", 0, 0);

        let labels: Vec<String> = items
            .iter()
            .filter_map(|it| it.get("label").and_then(|l| l.as_str()).map(String::from))
            .collect();

        let deprecated_item = items
            .iter()
            .find(|it| {
                it.get("label").and_then(|l| l.as_str()) == Some("Main::Hoge::old_func")
            })
            .unwrap_or_else(|| {
                panic!(
                    "Expected `Main::Hoge::old_func` in completion candidates. \
                     Got labels: {:?}",
                    labels
                )
            });

        // Legacy field.
        assert_eq!(
            deprecated_item.get("deprecated").and_then(|v| v.as_bool()),
            Some(true),
            "`deprecated: true` should be set on the completion item for \
             `Main::Hoge::old_func`. Got: {}",
            deprecated_item
        );

        // Modern field. `CompletionItemTag::DEPRECATED` serialises to `1`.
        let tags = deprecated_item
            .get("tags")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| {
                panic!(
                    "`tags` array should be set on the deprecated completion \
                     item. Got: {}",
                    deprecated_item
                )
            });
        assert!(
            tags.iter().any(|t| t.as_i64() == Some(1)),
            "`tags` should contain `Deprecated` (=1) for \
             `Main::Hoge::old_func`. Got tags: {:?}",
            tags
        );

        // Sanity: a non-deprecated symbol from the same fixture must NOT
        // carry the deprecation markers, otherwise the test above would
        // pass even if we accidentally tagged everything.
        let live_item = items
            .iter()
            .find(|it| it.get("label").and_then(|l| l.as_str()) == Some("Main::Hoge::new_func"))
            .unwrap_or_else(|| panic!("Expected `Main::Hoge::new_func` in candidates"));
        assert_eq!(
            live_item.get("deprecated").and_then(|v| v.as_bool()),
            None,
            "Non-deprecated symbol must not have `deprecated: true`. Got: {}",
            live_item
        );
        assert!(
            live_item
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true),
            "Non-deprecated symbol must not have any tags. Got: {}",
            live_item
        );

        // The deprecation message itself must surface in the resolved
        // item's documentation so the user sees *why* the symbol is
        // discouraged in the candidate's detail panel.
        let resolved = ctx.resolve(deprecated_item.clone());
        let doc_value = resolved.get("documentation").unwrap_or_else(|| {
            panic!(
                "Resolved deprecated item should have documentation. Got: {}",
                resolved
            )
        });
        // documentation can be a plain string or `{ kind, value }` for
        // MarkupContent. We only care that the message is in there.
        let doc_text = doc_value
            .as_str()
            .map(String::from)
            .or_else(|| {
                doc_value
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_else(|| {
                panic!(
                    "Documentation should be a string or MarkupContent. Got: {}",
                    doc_value
                )
            });
        assert!(
            doc_text.contains("Use `new_func` instead."),
            "Documentation should contain the DEPRECATED message. Got: {}",
            doc_text
        );

        ctx.shutdown();
    }
}
