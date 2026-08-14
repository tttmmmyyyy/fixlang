// LSP integration tests for the "textDocument/codeAction" (quick fix) feature: importing the
// module that declares a name the file uses, stubbing the members an implementation leaves out,
// and filling in the fields a struct literal leaves out.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::edit::edit_util::apply_text_edits;
    use crate::tests::test_util::copy_dir_recursive;
    use lsp_types::TextEdit;
    use serde_json::{json, Value};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::Duration,
    };
    use tempfile::TempDir;

    /// Parse a list of LSP `TextEdit` JSON values into `lsp_types::TextEdit`s.
    fn parse_text_edits(edits: &[Value]) -> Vec<TextEdit> {
        edits
            .iter()
            .map(|e| serde_json::from_value(e.clone()).expect("Failed to parse TextEdit"))
            .collect()
    }

    /// Whether the diagnostic's code is `code`.
    fn has_code(diag: &Value, code: &str) -> bool {
        diag.get("code")
            .and_then(|c| c.as_str())
            .map_or(false, |c| c == code)
    }

    /// The first diagnostic whose code is `code`, of which the test expects one to be published.
    fn diagnostic_with_code<'a>(diagnostics: &'a [Value], code: &str) -> &'a Value {
        diagnostics
            .iter()
            .find(|diag| has_code(diag, code))
            .unwrap_or_else(|| {
                panic!(
                    "Should have a '{}' diagnostic. Got: {:?}",
                    code, diagnostics
                )
            })
    }

    /// The start and the end of the diagnostic's range, each as a line and a character.
    fn range_of(diag: &Value) -> (u32, u32, u32, u32) {
        let range = diag.get("range").expect("Diagnostic should have a range");
        let start = range.get("start").expect("Range should have a start");
        let end = range.get("end").expect("Range should have an end");
        (
            start.get("line").unwrap().as_u64().unwrap() as u32,
            start.get("character").unwrap().as_u64().unwrap() as u32,
            end.get("line").unwrap().as_u64().unwrap() as u32,
            end.get("character").unwrap().as_u64().unwrap() as u32,
        )
    }

    /// The titles of the code actions.
    fn action_titles(actions: &[Value]) -> Vec<String> {
        actions
            .iter()
            .filter_map(|a| a.get("title").and_then(|t| t.as_str()).map(String::from))
            .collect()
    }

    /// The directory holding the Fix projects these tests run the language server on.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    /// Copies the named case project into a temporary directory, so that tests editing their
    /// project's files run beside one another, and returns that directory with the canonical path
    /// of the copy inside it.
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

    /// A language server running on a copy of one case project, with its documents open and its
    /// first diagnostics published.
    struct LspQuickFixCtx {
        /// The connection to the running server.
        client: LspClient,
        /// The copy of the case project the server was started on.
        project_dir: PathBuf,
        /// Holds the temporary directory containing `project_dir` alive; dropping it deletes the
        /// copy.
        _temp_dir: TempDir,
    }

    impl LspQuickFixCtx {
        /// Starts a server on a copy of the named case project, opens each of `files`, and waits
        /// for the diagnostics of the last of them, which the tests read.
        ///
        /// # Arguments
        /// * `files` — paths relative to the project directory; the last of them is the one whose
        ///   diagnostics are awaited.
        fn setup(project_name: &str, files: &[&str]) -> Self {
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

        /// The URI the server names the project's `file` by, as the responses spell it.
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

        /// Ends the session and fails the test if the server's reader thread met an error.
        fn shutdown(mut self) {
            self.client
                .shutdown(Duration::from_millis(500))
                .expect("Failed to shutdown LSP");
            self.client
                .finish()
                .expect("Reader thread should not have errors");
        }
    }

    /// An `unknown-name` diagnostic naming an associated type draws a quick fix that imports the
    /// type, as one naming a trait does.
    ///
    /// The test writes the diagnostics itself and the case project compiles cleanly, so what is
    /// exercised is the search for the named entity among the ones the file could import.
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
        let titles = action_titles(&actions);
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
        let titles = action_titles(&actions);
        assert!(
            titles.iter().any(|t| t.contains("MyElem")),
            "Quick fix should suggest importing associated type `MyElem`. Got actions: {:?}",
            titles
        );

        ctx.shutdown();
    }

    /// Test that quick fix suggests inserting stub implementations for missing trait members
    /// and associated types.
    #[test]
    fn test_quickfix_missing_trait_impl() {
        let mut ctx = LspQuickFixCtx::setup("quickfix_trait_impl", &["main.fix"]);

        // The project has an incomplete impl, so we should get diagnostics.
        let diagnostics = ctx.client.get_diagnostics(Path::new("main.fix"));
        assert!(
            !diagnostics.is_empty(),
            "Project should have diagnostics for incomplete trait impl."
        );

        // Find the "missing-trait-impl" diagnostic.
        let missing_diag = diagnostic_with_code(&diagnostics, "missing-trait-impl");

        // Get the range of the diagnostic.
        let (start_line, start_col, end_line, end_col) = range_of(missing_diag);

        // Request code actions with the real diagnostic.
        let actions = ctx.code_actions(
            "main.fix",
            vec![missing_diag.clone()],
            start_line,
            start_col,
            end_line,
            end_col,
        );

        // Verify that we get a quick fix action.
        assert!(
            !actions.is_empty(),
            "Should have at least one quick fix action."
        );

        let titles = action_titles(&actions);
        assert!(
            titles.iter().any(|t| t.contains("stub")),
            "Quick fix should suggest inserting stub implementations. Got: {:?}",
            titles
        );

        // Verify the text edit content.
        let action = actions
            .iter()
            .find(|a| {
                a.get("title")
                    .and_then(|t| t.as_str())
                    .map(|t| t.contains("stub"))
                    .unwrap_or(false)
            })
            .expect("Should find the stub action");

        let edit = action.get("edit").expect("Action should have edit");
        let changes = edit.get("changes").expect("Edit should have changes");

        // Get the text edit for main.fix.
        let uri = ctx.file_uri("main.fix");
        let file_edits = changes.get(&uri).expect("Should have edits for main.fix");
        let file_edits = file_edits.as_array().expect("Edits should be an array");
        assert_eq!(file_edits.len(), 1, "Should have exactly one text edit");

        let new_text = file_edits[0]
            .get("newText")
            .and_then(|t| t.as_str())
            .expect("Edit should have newText");

        // The stub should contain associated type and member stubs.
        assert!(
            new_text.contains("type Elem"),
            "Stub should contain associated type Elem. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("get_elem"),
            "Stub should contain member get_elem. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("show_it"),
            "Stub should contain member show_it. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("= ?;"),
            "Member stubs should use a `?` hole. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("type Elem Main::MyData = ?;"),
            "Associated type stub should have the form 'type Elem Main::MyData = ?;'. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("get_elem : Main::MyData -> Main::MyTrait::Elem Main::MyData"),
            "Member get_elem should have the correct type. Got: {:?}",
            new_text
        );
        assert!(
            new_text.contains("show_it : Main::MyData -> Std::String"),
            "Member show_it should have the correct type. Got: {:?}",
            new_text
        );

        ctx.shutdown();
    }

    /// A `missing-struct-field` diagnostic draws a quick fix that inserts a `name: ?` placeholder
    /// for the field the literal leaves out, and applying that edit clears the diagnostic, leaving
    /// the inserted hole as the one thing reported.
    #[test]
    fn test_quickfix_missing_struct_field() {
        let mut ctx = LspQuickFixCtx::setup("quickfix_missing_struct_field", &["main.fix"]);

        let diagnostics = ctx.client.get_diagnostics(Path::new("main.fix"));
        let missing_diag = diagnostic_with_code(&diagnostics, "missing-struct-field");

        let (start_line, start_col, end_line, end_col) = range_of(missing_diag);

        let actions = ctx.code_actions(
            "main.fix",
            vec![missing_diag.clone()],
            start_line,
            start_col,
            end_line,
            end_col,
        );

        let action = actions
            .iter()
            .find(|a| {
                a.get("title")
                    .and_then(|t| t.as_str())
                    .map(|t| t.contains("missing field") && t.contains("`z`"))
                    .unwrap_or(false)
            })
            .expect("Should find an 'Add missing field `z`' action");

        let edit = action.get("edit").expect("Action should have edit");
        let changes = edit.get("changes").expect("Edit should have changes");
        let uri = ctx.file_uri("main.fix");
        let file_edits = changes
            .get(&uri)
            .expect("Should have edits for main.fix")
            .as_array()
            .expect("Edits should be an array");
        assert_eq!(file_edits.len(), 1, "Should have exactly one text edit");
        let new_text = file_edits[0]
            .get("newText")
            .and_then(|t| t.as_str())
            .expect("Edit should have newText");
        // The struct literal `Vector3 { x: 1.0, y: 2.0 }` has `y: 2.0` as
        // its last field with no trailing comma, so the quick fix should
        // insert `, z: ?` just before `}`.
        assert_eq!(
            new_text, ", z: ?",
            "Insertion should be a comma-separated placeholder. Got: {:?}",
            new_text
        );

        // Apply the edits to the file on disk, then re-trigger diagnostics
        // and verify that:
        //   - the `missing-struct-field` diagnostic is gone (the struct
        //     literal type-checks now), and
        //   - a `missing-expression` diagnostic appears at the inserted `?`
        //     (the only remaining issue is the unresolved hole).
        let main_path = ctx.project_dir.join("main.fix");
        let original = fs::read_to_string(&main_path).expect("Failed to read main.fix");
        let parsed_edits = parse_text_edits(file_edits);
        let updated = apply_text_edits(&original, &parsed_edits);
        assert!(
            updated.contains("Vector3 { x: 1.0, y: 2.0, z: ? }"),
            "Updated source should contain the patched literal. Got: {}",
            updated
        );
        fs::write(&main_path, &updated).expect("Failed to write main.fix");
        ctx.client
            .change_document(Path::new("main.fix"))
            .expect("Failed to send didChange");
        ctx.client
            .trigger_and_wait_for_diagnostics(Path::new("main.fix"));

        let diagnostics = ctx.client.get_diagnostics(Path::new("main.fix"));
        assert!(
            !diagnostics
                .iter()
                .any(|d| has_code(d, "missing-struct-field")),
            "missing-struct-field diagnostic should be gone after applying the quick fix. Got: {:?}",
            diagnostics
        );
        let hole_diag = diagnostics
            .iter()
            .find(|d| has_code(d, "missing-expression"));
        assert!(
            hole_diag.is_some(),
            "Expected a `missing-expression` diagnostic for the inserted `?`. Got: {:?}",
            diagnostics
        );

        ctx.shutdown();
    }

    /// Test that the quick fix for a missing struct field is offered when the same literal also
    /// gives one field twice: beside the report of the repeat, the missing-field diagnostic keeps
    /// the code and the data the quick fix reads.
    #[test]
    fn test_quickfix_missing_struct_field_beside_a_duplicate() {
        let mut ctx = LspQuickFixCtx::setup(
            "quickfix_missing_struct_field_with_duplicate",
            &["main.fix"],
        );

        let diagnostics = ctx.client.get_diagnostics(Path::new("main.fix"));
        assert!(
            diagnostics.iter().any(|d| d
                .get("message")
                .and_then(|m| m.as_str())
                .map(|m| m.contains("Duplicate field"))
                .unwrap_or(false)),
            "The repeated field should be reported as well. Got: {:?}",
            diagnostics
        );
        let missing_diag = diagnostic_with_code(&diagnostics, "missing-struct-field").clone();

        let (start_line, start_col, end_line, end_col) = range_of(&missing_diag);

        let actions = ctx.code_actions(
            "main.fix",
            vec![missing_diag],
            start_line,
            start_col,
            end_line,
            end_col,
        );

        let action = actions
            .iter()
            .find(|a| {
                a.get("title")
                    .and_then(|t| t.as_str())
                    .map(|t| t.contains("missing field") && t.contains("`z`"))
                    .unwrap_or(false)
            })
            .expect("Should find an 'Add missing field `z`' action");
        assert!(
            action.get("edit").is_some(),
            "The action should carry an edit. Got: {:?}",
            action
        );

        ctx.shutdown();
    }
}
