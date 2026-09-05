//! What the language server does with a request it cannot carry out.
//!
//! A client sends what its own editor holds, and an editor holds whatever the file system gave it.
//! A request the server cannot answer is answered as far as it can be, and the session goes on: the
//! next request is served, so one such message costs the programmer nothing.

#[cfg(test)]
mod tests {
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::copy_dir_recursive;
    use serde_json::json;
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use tempfile::TempDir;

    /// Copies the named case project into a temporary directory and returns it with the project's
    /// path inside it.
    fn setup_test_env(project_name: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/test_lsp/cases");
        let project_dir = temp_dir.path().join(project_name);
        copy_dir_recursive(&cases_dir.join(project_name), &project_dir)
            .expect("Failed to copy test case");
        (temp_dir, project_dir)
    }

    /// A URI naming a file whose name is not UTF-8 leaves the server serving.
    ///
    /// A file name on Linux is any sequence of bytes, and an editor holding one percent-encodes it
    /// byte by byte, which puts escapes such as `%FF` — the start of no UTF-8 sequence — in the URI
    /// it sends. The server has nothing to say about such a file, since the URI it would answer
    /// under cannot be built either; what it must do is say nothing about that file alone.
    #[test]
    fn test_a_uri_whose_escapes_are_not_utf8_leaves_the_server_serving() {
        let (_temp_dir, project_dir) = setup_test_env("goto_local");
        let lib_fix = Path::new("lib.fix");

        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("Failed to initialize LSP");
        client
            .open_document(lib_fix)
            .expect("Failed to open document");
        client.trigger_and_wait_for_diagnostics(lib_fix);

        let undecodable_uri = format!("file://{}/%FF.fix", project_dir.display());
        client
            .send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": undecodable_uri,
                        "languageId": "fix",
                        "version": 1,
                        "text": "module A;\n"
                    }
                }),
            )
            .expect("Failed to send didOpen");
        let id = client
            .send_request(
                "textDocument/documentSymbol",
                json!({ "textDocument": { "uri": undecodable_uri } }),
            )
            .expect("Failed to send documentSymbol");
        client.wait_for_server(Duration::from_secs(2));
        let _ = client.get_response(id);

        // The pass this asks for is what says the server is still running: a server that ended on
        // the message above sends no progress, and the wait times out.
        client.trigger_and_wait_for_diagnostics(lib_fix);

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }
}
