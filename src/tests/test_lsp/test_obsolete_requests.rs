//! Requests whose answers have lost their use by the time the server reaches them.
//!
//! The client goes on sending while the server works on a request, and what it sends behind a
//! request can leave that request's answer with no use: a cancellation of it, or a change to the
//! document it asks about. Such a request is answered with the error the protocol gives for the
//! case, without being carried out.

#[cfg(test)]
mod tests {
    use super::super::case_project::setup_test_env;
    use super::super::lsp_client::LspClient;
    use serde_json::{json, Value};
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use tempfile::TempDir;

    /// The error code of a request the client cancelled.
    const REQUEST_CANCELLED: i64 = -32800;

    /// The error code of a request whose document changed after the request was sent.
    const CONTENT_MODIFIED: i64 = -32801;

    /// A session over the case project `goto_local`, with `main.fix` and `lib.fix` opened and
    /// analyzed once.
    fn open_session() -> (TempDir, PathBuf, LspClient) {
        let (temp_dir, project_dir) = setup_test_env("goto_local");
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        client
            .open_document(Path::new("lib.fix"))
            .expect("Failed to open lib.fix");
        client.save_and_wait_for_the_program(Path::new("main.fix"));
        (temp_dir, project_dir, client)
    }

    /// Send a completion request in `lib.fix`, a semantic-tokens request for `main.fix`, and then
    /// `follow`. Returns the ids of the completion request and the semantic-tokens request.
    ///
    /// The completion request elaborates the program, which holds the server long enough for the
    /// semantic-tokens request and everything `follow` sends to arrive behind it; the server
    /// therefore reaches the semantic-tokens request with those messages queued.
    fn send_behind_a_completion(
        client: &mut LspClient,
        follow: impl FnOnce(&mut LspClient, u32),
    ) -> (u32, u32) {
        let completion = client
            .send_request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": client.file_uri(Path::new("lib.fix")) },
                    "position": { "line": 9, "character": 5 }
                }),
            )
            .expect("Failed to send completion");
        let tokens = client
            .send_request(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": client.file_uri(Path::new("main.fix")) } }),
            )
            .expect("Failed to send semanticTokens");
        follow(client, tokens);
        (completion, tokens)
    }

    /// Tell the server that the client now holds `text` for `file` under `version`.
    fn change(client: &mut LspClient, file: &Path, version: i32, text: &str) {
        client
            .send_notification(
                "textDocument/didChange",
                json!({
                    "textDocument": { "uri": client.file_uri(file), "version": version },
                    "contentChanges": [ { "text": text } ]
                }),
            )
            .expect("Failed to send didChange");
    }

    /// The error code a response carries, or `None` for a response carrying a result.
    fn error_code(response: &Value) -> Option<i64> {
        response.get("error").map(|error| {
            error["code"]
                .as_i64()
                .expect("an error carries its code as a number")
        })
    }

    /// A request is answered with `ContentModified` when a change to the document it asks about
    /// arrived behind it, and the request before it, on another document, is carried out.
    #[test]
    fn test_a_request_on_a_document_changed_behind_it_is_answered_content_modified() {
        let (_temp_dir, project_dir, mut client) = open_session();
        let main_fix = Path::new("main.fix");
        let text = std::fs::read_to_string(project_dir.join(main_fix)).unwrap();

        let (completion, tokens) = send_behind_a_completion(&mut client, |client, _| {
            change(client, main_fix, 2, &format!("{}\n", text));
        });

        assert_eq!(
            error_code(&client.expect_response(completion)),
            None,
            "the completion request asks about lib.fix, which no change arrived behind, so it is \
             expected to be carried out"
        );
        assert_eq!(
            error_code(&client.expect_response(tokens)),
            Some(CONTENT_MODIFIED),
            "a change to main.fix arrived behind the semantic-tokens request, so it is expected to \
             be answered ContentModified"
        );

        client.shutdown().expect("Failed to shutdown LSP");
        client
            .verify_no_protocol_error()
            .expect("Reader thread should not have errors");
    }

    /// A request is carried out when the change that arrived behind it is to another document.
    #[test]
    fn test_a_change_to_another_document_leaves_a_request_to_be_carried_out() {
        let (_temp_dir, project_dir, mut client) = open_session();
        let lib_fix = Path::new("lib.fix");
        let text = std::fs::read_to_string(project_dir.join(lib_fix)).unwrap();

        let (_, tokens) = send_behind_a_completion(&mut client, |client, _| {
            change(client, lib_fix, 2, &format!("{}\n", text));
        });

        let response = client.expect_response(tokens);
        assert_eq!(
            error_code(&response),
            None,
            "the change behind the semantic-tokens request is to lib.fix, and the request asks \
             about main.fix, so it is expected to be carried out"
        );
        assert!(
            !response["result"]["data"]
                .as_array()
                .expect("a semanticTokens response carries its data")
                .is_empty(),
            "the semantic tokens of main.fix are expected to be answered"
        );

        client.shutdown().expect("Failed to shutdown LSP");
        client
            .verify_no_protocol_error()
            .expect("Reader thread should not have errors");
    }

    /// A request the client cancelled before the server reached it is answered
    /// `RequestCancelled`.
    #[test]
    fn test_a_request_cancelled_before_it_is_reached_is_answered_request_cancelled() {
        let (_temp_dir, _project_dir, mut client) = open_session();

        let (_, tokens) = send_behind_a_completion(&mut client, |client, tokens| {
            client
                .send_notification("$/cancelRequest", json!({ "id": tokens }))
                .expect("Failed to send cancelRequest");
        });

        assert_eq!(
            error_code(&client.expect_response(tokens)),
            Some(REQUEST_CANCELLED),
            "the client cancelled the semantic-tokens request before the server reached it, so it \
             is expected to be answered RequestCancelled"
        );

        client.shutdown().expect("Failed to shutdown LSP");
        client
            .verify_no_protocol_error()
            .expect("Reader thread should not have errors");
    }
}
