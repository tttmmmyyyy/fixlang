// LSP integration tests for "workspace/symbol" feature.
//
// Verifies that user-defined symbols (types, type aliases, traits, trait
// aliases, trait members, global values, trait instances) appear in the
// workspace symbol picker, that std-library symbols don't pollute the
// list, and that the `query` field filters by name.

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

    /// Absolute path to the directory containing LSP test fixture projects.
    fn get_test_cases_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/tests/test_lsp/cases");
        path
    }

    /// Copies the named fixture project into a fresh temporary directory
    /// and returns the temp dir handle plus the canonicalized project path.
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

    /// Test fixture that owns an initialized `LspClient` together with
    /// the temporary project directory it operates on.
    struct LspWorkspaceSymbolCtx {
        client: LspClient,
        _project_dir: PathBuf,
        _temp_dir: TempDir,
    }

    impl LspWorkspaceSymbolCtx {
        /// Boots the LSP server against a temp copy of the named fixture,
        /// opens each file in `files`, and waits for the first round of
        /// diagnostics on the last opened file before returning.
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
                _project_dir: project_dir,
                _temp_dir: temp_dir,
            }
        }

        /// Send workspace/symbol with the given query and return the
        /// resulting array (the response's `result` field).
        fn workspace_symbols(&mut self, query: &str) -> Vec<Value> {
            let id = self
                .client
                .send_request(
                    "workspace/symbol",
                    json!({
                        "query": query,
                    }),
                )
                .expect("Failed to send workspace/symbol request");
            self.client.wait_for_server(Duration::from_secs(5));
            let response = self
                .client
                .get_response(id)
                .expect("Should receive a workspace/symbol response");
            let result = response
                .get("result")
                .expect("Response should have a result field");
            result.as_array().cloned().unwrap_or_default()
        }

        /// Performs a clean LSP shutdown and joins the reader thread.
        fn shutdown(mut self) {
            self.client
                .shutdown(Duration::from_millis(500))
                .expect("Failed to shutdown LSP");
            self.client
                .finish()
                .expect("Reader thread should not have errors");
        }
    }

    /// Extracts the `name` field from each `SymbolInformation` value.
    fn names(symbols: &[Value]) -> Vec<String> {
        symbols
            .iter()
            .filter_map(|s| s.get("name").and_then(|n| n.as_str()).map(String::from))
            .collect()
    }

    /// Empty query should return all user-defined symbols, covering each
    /// entity kind defined by the `completion` test project.
    #[test]
    fn test_workspace_symbol_empty_query_returns_all_user_symbols() {
        let mut ctx = LspWorkspaceSymbolCtx::setup("completion", &["lib.fix", "main.fix"]);

        let symbols = ctx.workspace_symbols("");
        let names = names(&symbols);

        // Type (struct)
        assert!(
            names.iter().any(|n| n == "Lib::MyData"),
            "Type `MyData` missing. Got: {:?}",
            names
        );
        // Type alias
        assert!(
            names.iter().any(|n| n == "Lib::MyInt"),
            "Type alias `MyInt` missing. Got: {:?}",
            names
        );
        // Trait
        assert!(
            names.iter().any(|n| n == "Lib::MyTrait"),
            "Trait `MyTrait` missing. Got: {:?}",
            names
        );
        // Trait alias
        assert!(
            names.iter().any(|n| n == "Lib::MyTraitAlias"),
            "Trait alias `MyTraitAlias` missing. Got: {:?}",
            names
        );
        // Trait member
        assert!(
            names.iter().any(|n| n == "Lib::MyTrait::get_elem"),
            "Trait member `get_elem` missing. Got: {:?}",
            names
        );
        // Global value
        assert!(
            names.iter().any(|n| n == "Lib::my_func"),
            "Global value `my_func` missing. Got: {:?}",
            names
        );
        // Trait implementation
        assert!(
            names.iter().any(|n| n.starts_with("impl ") && n.contains("MyData")),
            "Trait impl for MyData missing. Got: {:?}",
            names
        );
        // Main from main.fix
        assert!(
            names.iter().any(|n| n == "Main::main"),
            "`Main::main` missing. Got: {:?}",
            names
        );

        // Std-library symbols must not leak in.
        assert!(
            !names.iter().any(|n| n.starts_with("Std::")),
            "Std symbols should be filtered out, but found some. Got: {:?}",
            names
        );

        ctx.shutdown();
    }

    /// A non-empty query filters the result set to symbols whose name
    /// contains the query (case-insensitive).
    #[test]
    fn test_workspace_symbol_query_filters_by_name() {
        let mut ctx = LspWorkspaceSymbolCtx::setup("completion", &["lib.fix", "main.fix"]);

        let symbols = ctx.workspace_symbols("MyData");
        let names = names(&symbols);

        assert!(
            !names.is_empty(),
            "Query `MyData` should return at least one symbol",
        );
        for n in &names {
            assert!(
                n.to_lowercase().contains("mydata"),
                "Result `{}` does not contain query `MyData`. All results: {:?}",
                n,
                names,
            );
        }

        // The struct itself must be present.
        assert!(
            names.iter().any(|n| n == "Lib::MyData"),
            "Type `MyData` missing from filtered query. Got: {:?}",
            names
        );

        ctx.shutdown();
    }

    /// Each returned symbol's `location.uri` should point into the
    /// project directory (not into std lib or dependency caches).
    #[test]
    fn test_workspace_symbol_locations_are_in_project() {
        let mut ctx = LspWorkspaceSymbolCtx::setup("completion", &["lib.fix", "main.fix"]);
        let project_dir_str = ctx._project_dir.display().to_string();

        let symbols = ctx.workspace_symbols("");
        assert!(!symbols.is_empty(), "Expected at least one symbol");

        for sym in &symbols {
            let uri = sym
                .get("location")
                .and_then(|l| l.get("uri"))
                .and_then(|u| u.as_str())
                .expect("Symbol should have location.uri");
            assert!(
                uri.contains(&project_dir_str),
                "Symbol URI `{}` is not inside project dir `{}`",
                uri,
                project_dir_str,
            );
        }

        ctx.shutdown();
    }
}
