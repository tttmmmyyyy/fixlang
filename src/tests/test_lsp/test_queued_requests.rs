//! Requests the client cancels, or edits behind, before the server reaches them.
//!
//! The client goes on sending while the server works on a request. A cancellation of a request
//! still waiting its turn is answered with `RequestCancelled`, without carrying the request out. A
//! change to the document a waiting request asks about leaves the request to be carried out: the
//! client decides whether an answer computed on the older text is of use, and cancels it if not.

#[cfg(test)]
mod tests {
    use super::super::case_project::setup_test_env;
    use super::super::lsp_client::LspClient;
    use lsp_types::error_codes::REQUEST_CANCELLED;
    use serde_json::{json, Value};
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use tempfile::TempDir;

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

    /// Send a semantic-tokens request for `main.fix`, and return its id.
    fn request_tokens(client: &mut LspClient) -> u32 {
        client
            .send_request(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": client.file_uri(Path::new("main.fix")) } }),
            )
            .expect("Failed to send semanticTokens")
    }

    /// Assert that the server sends no response to the request `id` beyond the one the test has
    /// taken. The server handles the client's messages in the order they arrive, so once a request
    /// sent now is answered, every response to `id` the server was going to send has arrived.
    fn assert_answered_once(client: &mut LspClient, id: u32) {
        let probe = request_tokens(client);
        client.expect_response(probe);
        assert!(
            client.take_response(id).is_none(),
            "the request {} is expected to be answered once",
            id
        );
    }

    /// A request is carried out when a change to the document it asks about arrived behind it.
    #[test]
    fn test_a_change_behind_a_request_leaves_it_to_be_carried_out() {
        let (_temp_dir, project_dir, mut client) = open_session();
        let main_fix = Path::new("main.fix");
        let text = std::fs::read_to_string(project_dir.join(main_fix)).unwrap();

        let (_, tokens) = send_behind_a_completion(&mut client, |client, _| {
            change(client, main_fix, 2, &format!("{}\n", text));
        });

        let response = client.expect_response(tokens);
        assert_eq!(
            error_code(&response),
            None,
            "a change to main.fix arrived behind the semantic-tokens request, which the client \
             did not cancel, so it is expected to be carried out"
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

    /// A cancellation of a request the server has already answered leaves that answer the only
    /// one.
    #[test]
    fn test_a_cancellation_after_the_answer_sends_nothing_more() {
        let (_temp_dir, _project_dir, mut client) = open_session();

        let tokens = request_tokens(&mut client);
        client.expect_response(tokens);
        client
            .send_notification("$/cancelRequest", json!({ "id": tokens }))
            .expect("Failed to send cancelRequest");
        assert_answered_once(&mut client, tokens);

        client.shutdown().expect("Failed to shutdown LSP");
        client
            .verify_no_protocol_error()
            .expect("Reader thread should not have errors");
    }

    /// A cancellation names a request of the client: a response of the client queued ahead of that
    /// request under the same id is left to be handled as a response.
    #[test]
    fn test_a_cancellation_passes_over_a_response_carrying_the_same_id() {
        let (_temp_dir, _project_dir, mut client) = open_session();

        let completion = client
            .send_request(
                "textDocument/completion",
                json!({
                    "textDocument": { "uri": client.file_uri(Path::new("lib.fix")) },
                    "position": { "line": 9, "character": 5 }
                }),
            )
            .expect("Failed to send completion");
        // The client numbers its requests in order, so the next request carries `completion + 1`.
        client
            .send_response(completion + 1, Value::Null)
            .expect("Failed to send a response");
        let tokens = request_tokens(&mut client);
        assert_eq!(tokens, completion + 1);
        client
            .send_notification("$/cancelRequest", json!({ "id": tokens }))
            .expect("Failed to send cancelRequest");

        assert_eq!(
            error_code(&client.expect_response(tokens)),
            Some(REQUEST_CANCELLED),
            "the client cancelled the semantic-tokens request before the server reached it, so it \
             is expected to be answered RequestCancelled"
        );
        assert_answered_once(&mut client, tokens);

        client.shutdown().expect("Failed to shutdown LSP");
        client
            .verify_no_protocol_error()
            .expect("Reader thread should not have errors");
    }
}
