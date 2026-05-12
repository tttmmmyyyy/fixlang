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

    /// `42.<cursor>` in a body that mentions both `myfunc1 : U32 -> U32 -> U32`
    /// and `myfunc2 : I64 -> I64 -> I64` — `42` is `I64` so `myfunc2` should
    /// outrank `myfunc1` in the completion list. Verifies the dot-completion
    /// type-aware ranking pipeline (Steps 1-4).
    #[test]
    fn test_completion_dot_sort_ranks_matching_receiver_above_others() {
        let mut ctx = LspCompletionCtx::setup("completion-dot-sort", &["main.fix"]);

        // Cursor right after the dot in `    42.` on line 13 (0-indexed),
        // column 7 (= byte right after `.`).
        // Use a polling wait — Step 1's full re-elaborate can take longer
        // than `complete`'s hard-coded 5s sleep on a cold cache.
        let id = ctx
            .client
            .send_request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": ctx.file_uri("main.fix") },
                    "position": { "line": 13, "character": 7 }
                }),
            )
            .expect("send completion");
        let mut items: Vec<Value> = vec![];
        let start = std::time::Instant::now();
        loop {
            ctx.client.wait_for_server(Duration::from_millis(500));
            if let Some(response) = ctx.client.get_response(id) {
                let result = response.get("result").expect("response has result");
                items = if result.is_array() {
                    result.as_array().unwrap().clone()
                } else {
                    result
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                };
                break;
            }
            if start.elapsed() > Duration::from_secs(60) {
                panic!("completion did not respond within 60s");
            }
        }

        // Each item should carry a sortText derived from its tier.
        let find_sort = |label: &str| -> String {
            let it = items
                .iter()
                .find(|it| it.get("label").and_then(|l| l.as_str()) == Some(label))
                .unwrap_or_else(|| {
                    panic!("expected {} in completion items; got {:?}", label, items)
                });
            it.get("sortText")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| {
                    panic!(
                        "expected sortText on dot-completion item {}; got {}",
                        label, it
                    )
                })
        };

        let sort_myfunc1 = find_sort("Main::myfunc1");
        let sort_myfunc2 = find_sort("Main::myfunc2");
        assert!(
            sort_myfunc2 < sort_myfunc1,
            "myfunc2 (I64 receiver) should sort before myfunc1 (U32 receiver); \
             got myfunc2={:?}, myfunc1={:?}",
            sort_myfunc2,
            sort_myfunc1
        );
        // Stronger: I64 unify should land myfunc2 in Tier 0. The
        // namespace-match sub-tier is encoded as a single letter
        // following the digit (`0a` / `0b` / `0c`); for `Main::myfunc2`
        // with an `I64` receiver the namespace `Main` is unrelated to
        // `Std::I64`, so the sub-tier is `c`.
        assert!(
            sort_myfunc2.starts_with('0'),
            "myfunc2 should be Tier 0 (sortText `0…`); got {:?}",
            sort_myfunc2
        );

        ctx.shutdown();
    }

    /// Scenario B: the on-disk file has a parse error (`42` with no
    /// dot inside `(...)`), so the snapshot Program built at LSP
    /// startup may be missing the user's module entirely. The user
    /// then types `.` (live buffer becomes parseable as `42.pure()`)
    /// and triggers completion.
    ///
    /// This reproduces the user's report that priority ranking
    /// doesn't apply after a "save with parse error → close → reopen
    /// → type the dot" round trip. We expect the test to fail before
    /// any fix lands; once it passes the regression is closed.
    #[test]
    fn test_completion_dot_sort_stale_snapshot_after_dot_added() {
        use crate::tests::test_lsp::lsp_client::LspClient;
        use std::fs;

        install_fix();
        let (temp_dir, project_dir) = setup_test_env("completion-dot-sort-stale");
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(5))
            .expect("Failed to initialize LSP");

        // Open the parse-erroring file. didOpen sends the on-disk
        // content; we don't save (which is what triggers diagnostics
        // in this server) — the LSP startup `Start` message already
        // ran diagnostics once before the open.
        client
            .open_document(Path::new("main.fix"))
            .expect("open main.fix");
        client.trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        // Replay: the user types `.` after `42`, turning the line
        // into `    42.`. didChange notifications carry the full text
        // (the LSP capability advertised `change: 1` = full sync).
        let abs_path = project_dir.join("main.fix");
        let dot_added = fs::read_to_string(&abs_path)
            .expect("read main.fix")
            .replace("    42\n", "    42.\n");
        let uri = format!("file://{}", abs_path.display());
        client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": uri, "version": 2 },
                    "contentChanges": [ { "text": dot_added } ]
                }),
            )
            .expect("send didChange");

        // Trigger completion right after the inserted dot. Find the
        // line that ends with `42.` so the test stays robust to fixture
        // edits.
        let line = dot_added
            .lines()
            .position(|l| l.trim_end().ends_with("42."))
            .expect("find `42.` line in dot_added") as u32;
        let col = dot_added
            .lines()
            .nth(line as usize)
            .map(|l| l.find('.').unwrap() as u32 + 1)
            .expect("find `.` column");

        let id = client
            .send_request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": col }
                }),
            )
            .expect("send completion");

        let mut items: Vec<Value> = vec![];
        let start = std::time::Instant::now();
        loop {
            client.wait_for_server(Duration::from_millis(500));
            if let Some(response) = client.get_response(id) {
                let result = response
                    .get("result")
                    .expect("response has result");
                items = if result.is_array() {
                    result.as_array().unwrap().clone()
                } else {
                    result
                        .get("items")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default()
                };
                break;
            }
            if start.elapsed() > Duration::from_secs(60) {
                let log_path = project_dir.join(".fixlang/fix.log");
                let log_content =
                    fs::read_to_string(&log_path).unwrap_or_else(|_| "<no log>".into());
                let completion_log: String = log_content
                    .lines()
                    .filter(|l| l.contains("[completion]"))
                    .collect::<Vec<_>>()
                    .join("\n");
                panic!(
                    "completion did not respond within 60s.\n\
                     [completion] log lines:\n{}",
                    if completion_log.is_empty() {
                        "<none>".to_string()
                    } else {
                        completion_log
                    }
                );
            }
        }

        let log_path = project_dir.join(".fixlang/fix.log");
        let log_content = fs::read_to_string(&log_path).unwrap_or_default();
        let completion_log: String = log_content
            .lines()
            .filter(|l| l.contains("[completion]"))
            .collect::<Vec<_>>()
            .join("\n");
        eprintln!(
            "===== [completion] log lines (scenario B) =====\n{}\n=========================================",
            if completion_log.is_empty() {
                "<none>".to_string()
            } else {
                completion_log
            }
        );

        let item_summary: Vec<String> = items
            .iter()
            .filter_map(|it| {
                let label = it.get("label")?.as_str()?;
                let sort = it
                    .get("sortText")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<none>");
                Some(format!("{:>40}  sort={}", label, sort))
            })
            .filter(|s| s.contains("Main::") || s.contains("myfunc"))
            .collect();
        eprintln!(
            "===== Main:: items (scenario B) =====\n{}\n=====================================",
            item_summary.join("\n")
        );

        let _ = client.shutdown(Duration::from_millis(500));
        let _ = client.finish();
        drop(temp_dir);

        // Assertion: myfunc2 should out-rank myfunc1 even when the
        // snapshot Program was built from a parse-erroring source.
        let find_sort = |label: &str| -> Option<String> {
            items
                .iter()
                .find(|it| it.get("label").and_then(|l| l.as_str()) == Some(label))
                .and_then(|it| it.get("sortText"))
                .and_then(|v| v.as_str())
                .map(String::from)
        };
        match (find_sort("Main::myfunc1"), find_sort("Main::myfunc2")) {
            (Some(s1), Some(s2)) => {
                assert!(
                    s2 < s1,
                    "scenario B: myfunc2 should sort before myfunc1; got myfunc2={:?}, myfunc1={:?}",
                    s2, s1
                );
            }
            (s1, s2) => panic!(
                "scenario B: missing sortText for Main::myfunc1 ({:?}) or Main::myfunc2 ({:?})",
                s1, s2
            ),
        }
    }

    /// Reproduces the user-report: `let n = range(50, 101).<cursor>`
    /// with the cursor right after the dot at end of line. We expect
    /// some `Std::Iterator::*` method (e.g. `fold`) to be ranked
    /// strictly above an alphabetically-earlier candidate like
    /// `Std::Add::add` — i.e. the dot-completion ranker must classify
    /// the receiver as a `RangeIterator`-like type and place Iterator
    /// methods in a lower-numbered tier.
    #[test]
    fn test_completion_dot_sort_iterator_at_end_of_line() {
        let mut ctx = LspCompletionCtx::setup("completion-dot-sort-iterator", &["main.fix"]);

        // main.fix layout (0-indexed):
        //   0: module Main;
        //   1: (blank)
        //   2: main : IO () = (
        //   3:     let n = range(50, 101).
        //   4:     pure()
        //   5: );
        //
        // The `.` is at column 26 (4 spaces + "let n = range(50, 101)" =
        // 22 chars + `.` = 27 chars total; the dot is the 27th char,
        // so byte-after-dot is column 27).
        let items = ctx.complete("main.fix", 3, 27);

        // Dump the [completion] log lines so we can see what the
        // dot-context extractor actually observed as the receiver
        // type (it logs `dot-context receiver type: <ty>`).
        let log_path = ctx.project_dir.join(".fixlang/fix.log");
        if let Ok(log_content) = std::fs::read_to_string(&log_path) {
            let completion_log: String = log_content
                .lines()
                .filter(|l| l.contains("[completion]"))
                .collect::<Vec<_>>()
                .join("\n");
            eprintln!(
                "===== [completion] log lines =====\n{}\n==================================",
                if completion_log.is_empty() {
                    "<none>".to_string()
                } else {
                    completion_log
                }
            );
        } else {
            eprintln!("===== [completion] log =====\n<no log file at {}>\n=============================", log_path.display());
        }

        let dump_top: Vec<String> = items
            .iter()
            .filter_map(|it| {
                let label = it.get("label")?.as_str()?;
                let sort = it
                    .get("sortText")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<none>");
                Some(format!("{:>50}  sort={}", label, sort))
            })
            .take(20)
            .collect();
        eprintln!(
            "===== first 20 completion items =====\n{}\n=====================================",
            dump_top.join("\n")
        );

        let find_sort = |label: &str| -> Option<String> {
            items
                .iter()
                .find(|it| it.get("label").and_then(|l| l.as_str()) == Some(label))
                .and_then(|it| it.get("sortText"))
                .and_then(|v| v.as_str())
                .map(String::from)
        };

        let sort_fold = find_sort("Std::Iterator::fold");
        let sort_add = find_sort("Std::Add::add");
        eprintln!(
            "Std::Iterator::fold sort = {:?}, Std::Add::add sort = {:?}",
            sort_fold, sort_add
        );

        let s_fold = sort_fold.expect("Std::Iterator::fold should be a candidate");
        let s_add = sort_add.expect("Std::Add::add should be a candidate");
        assert!(
            s_fold < s_add,
            "Std::Iterator::fold should rank above Std::Add::add for a RangeIterator receiver; \
             got fold={:?}, add={:?}",
            s_fold,
            s_add
        );

        // No candidate label should contain `?` — those name the
        // opaque tycons introduced by opaque-tyvar desugar
        // (e.g. `Std::Iterator::range::?it`) and aren't anything
        // the user can write.
        let opaque_leaks: Vec<String> = items
            .iter()
            .filter_map(|it| it.get("label").and_then(|l| l.as_str()).map(String::from))
            .filter(|l| l.contains('?'))
            .collect();
        assert!(
            opaque_leaks.is_empty(),
            "completion list contains opaque-tycon labels: {:?}",
            opaque_leaks
        );

        ctx.shutdown();
    }
}
