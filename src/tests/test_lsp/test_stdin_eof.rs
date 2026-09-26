//! LSP integration test for stdin EOF handling.
//!
//! When the parent editor process dies it closes the pipe connected to
//! the language server's stdin. `read_line` then returns `Ok(0)` on
//! every call, and the server's read loop must recognize that as EOF and
//! terminate, so that no orphaned process is left behind.

#[cfg(test)]
mod tests {
    use super::super::case_project::setup_test_env;
    use super::super::lsp_client::LspClient;
    use crate::tests::test_util::{fix_command, wait_within};
    use serde_json::{json, Value};
    use std::{iter, path::Path, process::Stdio, thread::sleep, time::Duration};

    /// Verifies that the language server terminates promptly once its
    /// stdin reaches EOF (parent editor closed the pipe).
    #[test]
    fn test_lsp_exits_on_stdin_eof() {
        let (_temp_dir, project_dir) = setup_test_env("completion");

        let mut child = fix_command()
            .arg("language-server")
            .current_dir(&project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn fix language-server");

        // Give the server time to start and block on its stdin read loop.
        sleep(Duration::from_millis(500));

        // Sanity check: the server should still be running here (blocked
        // waiting for input), not exited for some unrelated reason.
        assert!(
            child
                .try_wait()
                .expect("Failed to poll server status")
                .is_none(),
            "Server exited before stdin was closed"
        );

        // Simulate the parent editor process dying: close stdin so the
        // server observes EOF.
        drop(child.stdin.take().expect("stdin handle already taken"));

        // The server must terminate promptly once it observes the EOF.
        wait_within(
            &mut child,
            Duration::from_secs(10),
            "the LSP server after stdin reached EOF",
        );
    }

    /// The messages the server reads before stdin reaches EOF are all handled: a request still
    /// waiting its turn when the EOF arrives is answered before the server exits.
    #[test]
    fn test_lsp_answers_requests_queued_before_stdin_eof() {
        let (_temp_dir, project_dir) = setup_test_env("completion");
        let main_fix = Path::new("main.fix");
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("Failed to initialize LSP");
        client
            .open_document(main_fix)
            .expect("Failed to open main.fix");

        // The completion request elaborates the program, which holds the server while the EOF
        // arrives behind the semantic-tokens request.
        let uri = client.file_uri(main_fix);
        let completion_id = client
            .send_request(
                "textDocument/completion",
                json!({ "textDocument": { "uri": uri }, "position": { "line": 7, "character": 4 } }),
            )
            .expect("Failed to send completion");
        let tokens_id = client
            .send_request(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": uri } }),
            )
            .expect("Failed to send semanticTokens");
        client.close_stdin();

        client
            .wait_for_exit(Duration::from_secs(60))
            .expect("the LSP server is expected to exit after stdin reached EOF");
        client.expect_response(tokens_id);
        let answered: Vec<u32> = iter::from_fn(|| client.pop_message())
            .filter(|message| message.get("method").is_none())
            .filter_map(|message| message.get("id").and_then(Value::as_u64))
            .map(|id| id as u32)
            .filter(|id| [completion_id, tokens_id].contains(id))
            .collect();
        assert_eq!(
            answered,
            vec![completion_id, tokens_id],
            "every request sent before the EOF is expected to be answered, in order"
        );
    }
}
