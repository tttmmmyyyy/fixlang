//! What the language server does with a request it cannot carry out.
//!
//! A client sends what its own editor holds, and an editor holds whatever the file system gave it.
//! A request the server cannot answer is answered as far as it can be, and the session goes on: the
//! next request is served, so one such message costs the programmer nothing.

#[cfg(test)]
mod tests {
    use super::super::completion_harness::setup_test_env;
    use super::super::lsp_client::LspClient;
    use serde_json::{json, Value};
    use std::path::Path;
    use std::time::Duration;

    /// The response to the request `id`, waited for until it arrives.
    fn wait_for_response(client: &mut LspClient, id: u32) -> Value {
        for _ in 0..50 {
            if let Some(response) = client.get_response(id) {
                return response;
            }
            client.wait_for_server(Duration::from_millis(100));
        }
        panic!("the request {} is expected to be answered", id);
    }

    /// The delta-encoded semantic tokens the server answers for the buffer `uri` names.
    fn semantic_token_data(client: &mut LspClient, uri: &str) -> Vec<u64> {
        let id = client
            .send_request(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": uri } }),
            )
            .expect("Failed to send semanticTokens");
        wait_for_response(client, id)["result"]["data"]
            .as_array()
            .expect("a semanticTokens response carries its data")
            .iter()
            .filter_map(|number| number.as_u64())
            .collect()
    }

    /// The name of each symbol the server answers for the file `uri` names.
    fn document_symbol_names(client: &mut LspClient, uri: &str) -> Vec<String> {
        let id = client
            .send_request(
                "textDocument/documentSymbol",
                json!({ "textDocument": { "uri": uri } }),
            )
            .expect("Failed to send documentSymbol");
        wait_for_response(client, id)["result"]
            .as_array()
            .expect("a documentSymbol response carries an array of symbols")
            .iter()
            .map(|symbol| {
                symbol["name"]
                    .as_str()
                    .expect("a symbol carries its name")
                    .to_string()
            })
            .collect()
    }

    /// A session over the case project, with `file` opened and analyzed once.
    fn open_session(
        project: &str,
        file: &Path,
    ) -> (tempfile::TempDir, std::path::PathBuf, LspClient) {
        let (temp_dir, project_dir) = setup_test_env(project);
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("Failed to initialize LSP");
        client.open_document(file).expect("Failed to open document");
        client.trigger_and_wait_for_diagnostics(file);
        (temp_dir, project_dir, client)
    }

    /// A URI naming a file whose name is not UTF-8 is answered, and leaves the server serving.
    ///
    /// A file name on Linux is any sequence of bytes, and an editor holding one percent-encodes it
    /// byte by byte, which puts escapes such as `%FF` — the start of no UTF-8 sequence — in the URI
    /// it sends. The server has nothing to say about such a file, since the URI it would answer
    /// under cannot be built either; what it must do is say nothing about that file alone.
    #[test]
    fn test_a_uri_whose_escapes_are_not_utf8_leaves_the_server_serving() {
        let lib_fix = Path::new("lib.fix");
        let (_temp_dir, project_dir, mut client) = open_session("goto_local", lib_fix);

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
        assert_eq!(
            document_symbol_names(&mut client, &undecodable_uri),
            Vec::<String>::new(),
            "a file the server has no path for is expected to be answered with no symbols"
        );

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

    /// A URI whose escapes do not decode leaves no buffer recorded.
    ///
    /// `didOpen`, `didChange` and `didSave` each record what the client holds under the path its
    /// URI names, and a URI naming no path is left out of that record, which is what leaves every
    /// buffer the record holds with a path. What tells a buffer the record holds from one it does
    /// not is the semantic tokens of it: the tokens of its content are answered for the first, and
    /// none for the second.
    #[test]
    fn test_a_uri_whose_escapes_are_not_utf8_records_no_buffer() {
        let lib_fix = Path::new("lib.fix");
        let (_temp_dir, project_dir, mut client) = open_session("goto_local", lib_fix);

        // One escape per notification that records a buffer, each a byte that starts no UTF-8
        // sequence: a lone continuation byte, a three-byte sequence cut short, and a byte no
        // sequence uses.
        let opened = format!("file://{}/%80.fix", project_dir.display());
        let changed = format!("file://{}/%E3%81.fix", project_dir.display());
        let saved = format!("file://{}/%FF.fix", project_dir.display());
        client
            .send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": opened,
                        "languageId": "fix",
                        "version": 1,
                        "text": "module A; // opened\n"
                    }
                }),
            )
            .expect("Failed to send didOpen");
        client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": changed, "version": 2 },
                    "contentChanges": [ { "text": "module B; // changed\n" } ]
                }),
            )
            .expect("Failed to send didChange");
        client
            .send_notification(
                "textDocument/didSave",
                json!({
                    "textDocument": { "uri": saved },
                    "text": "module C; // saved\n"
                }),
            )
            .expect("Failed to send didSave");

        // A URI whose escapes do decode names a buffer, whatever bytes those escapes carry, and
        // the buffer it names is one the client holds rather than a file on disk.
        let decodable = format!("file://{}/caf%C3%A9.fix", project_dir.display());
        client
            .send_notification(
                "textDocument/didOpen",
                json!({
                    "textDocument": {
                        "uri": decodable,
                        "languageId": "fix",
                        "version": 1,
                        "text": "module D; // decodable\n"
                    }
                }),
            )
            .expect("Failed to send didOpen");

        for uri in [&opened, &changed, &saved] {
            assert!(
                semantic_token_data(&mut client, uri).is_empty(),
                "the uri \"{}\" decodes to no path, so the buffer it names is expected to be left \
                 unrecorded and to carry no semantic token",
                uri
            );
        }
        assert!(
            !semantic_token_data(&mut client, &decodable).is_empty(),
            "the buffer the uri \"{}\" names is expected to be recorded, and so to carry the \
             semantic tokens of its content",
            decodable
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// The symbols answered for a URI are those of the file its escapes decode to, and a URI
    /// naming no file on disk is answered with no symbols.
    #[test]
    fn test_document_symbol_answers_for_the_path_the_uri_decodes_to() {
        let lib_fix = Path::new("lib.fix");
        let (_temp_dir, project_dir, mut client) = open_session("goto_local", lib_fix);

        // `%6C%69%62` is `lib` written escape by escape, which a client is free to send.
        let escaped = format!("file://{}/%6C%69%62.fix", project_dir.display());
        let names = document_symbol_names(&mut client, &escaped);
        assert!(
            names.contains(&"Lib::simple_let".to_string()),
            "the escapes of \"{}\" name `lib.fix`, whose symbols are expected to carry \
             `Lib::simple_let`, but the answer carries {:?}",
            escaped,
            names
        );

        let absent = format!("file://{}/absent.fix", project_dir.display());
        assert_eq!(
            document_symbol_names(&mut client, &absent),
            Vec::<String>::new(),
            "a uri naming no file on disk is expected to be answered with no symbols"
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }
}
