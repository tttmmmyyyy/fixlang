// Shared harness for LSP completion tests: starts a language server
// over a private copy of a test project and sends completion /
// resolve requests against it.

use super::lsp_client::LspClient;
use crate::tests::test_util::copy_dir_recursive;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tempfile::TempDir;

/// The directory holding the LSP test projects, one subdirectory per
/// project, named as the tests name it.
fn get_test_cases_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/tests/test_lsp/cases");
    path
}

/// Copy the test project `project_name` into a temporary directory of its
/// own, so tests that build and edit it can run in parallel.
///
/// # Returns
/// The guard whose drop deletes the copy, and the canonicalized path of
/// the copied project.
pub fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_case_src = get_test_cases_dir().join(project_name);
    let test_case_dst = temp_dir.path().join(project_name);
    copy_dir_recursive(&test_case_src, &test_case_dst).expect("Failed to copy test case");
    let test_case_dst = test_case_dst
        .canonicalize()
        .expect("Failed to canonicalize test case path");
    (temp_dir, test_case_dst)
}

/// Look up the `sortText` of the completion item whose `label` is `label`.
pub fn find_sort_text(items: &[Value], label: &str) -> Option<String> {
    items
        .iter()
        .find(|it| it.get("label").and_then(|l| l.as_str()) == Some(label))
        .and_then(|it| it.get("sortText"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Poll an in-flight `textDocument/completion` request until the
/// server replies or `timeout` elapses. Returns the completion items
/// (the response's `result` may be either an array or a
/// `CompletionList` — both shapes are unwrapped); returns `None`
/// when the timeout expires so the caller can format its own
/// diagnostic.
pub fn collect_completion_items(
    client: &mut LspClient,
    request_id: u32,
    timeout: Duration,
) -> Option<Vec<Value>> {
    let start = Instant::now();
    loop {
        client.wait_for_server(Duration::from_millis(500));
        if let Some(response) = client.get_response(request_id) {
            let result = response.get("result").expect("response has result");
            let items = if result.is_array() {
                result.as_array().unwrap().clone()
            } else {
                result
                    .get("items")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default()
            };
            return Some(items);
        }
        if start.elapsed() > timeout {
            return None;
        }
    }
}

/// A language server running over a private copy of one test project,
/// ready to answer completion requests against that copy's files.
pub struct LspCompletionCtx {
    /// The client end of the server's stdio connection.
    pub client: LspClient,
    /// Absolute path of the project copy, the base of every file URI.
    pub project_dir: PathBuf,
    /// Kept alive so the copy outlives the server; its drop deletes the
    /// copy.
    _temp_dir: TempDir,
}

impl LspCompletionCtx {
    /// Start a server over a fresh copy of `project_name` and open each of
    /// `files`, paths relative to the project root, in the given order.
    /// Returns once the server has published diagnostics for the last of
    /// them, so the project has been type-checked.
    pub fn setup(project_name: &str, files: &[&str]) -> Self {
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

    /// The `file://` URI the server knows `file` by, `file` being a path
    /// relative to the project root.
    pub fn file_uri(&self, file: &str) -> String {
        format!("file://{}", self.project_dir.join(file).display())
    }

    /// Send textDocument/completion and return the result items,
    /// waiting up to 5 seconds for the response.
    pub fn complete(&mut self, file: &str, line: u32, col: u32) -> Vec<Value> {
        self.complete_with_timeout(file, line, col, Duration::from_secs(5))
    }

    /// Send textDocument/completion and poll for the response with
    /// the given timeout. Use this in dot-completion tests where
    /// the server's first-time re-elaborate can take longer than
    /// `complete`'s 5-second wait on a cold cache.
    pub fn complete_with_timeout(
        &mut self,
        file: &str,
        line: u32,
        col: u32,
        timeout: Duration,
    ) -> Vec<Value> {
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
        collect_completion_items(&mut self.client, id, timeout)
            .unwrap_or_else(|| panic!("completion did not respond within {:?}", timeout))
    }

    /// Send completionItem/resolve and return the resolved item.
    pub fn resolve(&mut self, item: Value) -> Value {
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

    /// Shut the server down and assert its reader thread saw no errors.
    pub fn shutdown(mut self) {
        self.client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        self.client
            .finish()
            .expect("Reader thread should not have errors");
    }
}
