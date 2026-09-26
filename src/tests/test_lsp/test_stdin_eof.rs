//! LSP integration test for stdin EOF handling.
//!
//! When the parent editor process dies it closes the pipe connected to
//! the language server's stdin. `read_line` then returns `Ok(0)` on
//! every call, and the server's read loop must recognize that as EOF and
//! terminate, so that no orphaned process is left behind.

#[cfg(test)]
mod tests {
    use super::super::case_project::setup_test_env;
    use crate::tests::test_util::{fix_command, wait_within};
    use serde_json::{json, Value};
    use std::{
        fs,
        io::{Read, Write},
        process::Stdio,
        thread::{self, sleep},
        time::Duration,
    };

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
        let uri = format!("file://{}", project_dir.join("main.fix").display());
        let text = fs::read_to_string(project_dir.join("main.fix")).unwrap();

        let mut child = fix_command()
            .arg("language-server")
            .current_dir(&project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn fix language-server");
        let mut stdout = child.stdout.take().unwrap();
        let reader = thread::spawn(move || {
            let mut out = String::new();
            stdout.read_to_string(&mut out).unwrap();
            out
        });

        // The completion request elaborates the program, which holds the server while the EOF
        // arrives behind the semantic-tokens request.
        let messages = [
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": { "processId": null, "capabilities": {} } }),
            json!({ "jsonrpc": "2.0", "method": "textDocument/didOpen",
                    "params": { "textDocument": { "uri": uri, "languageId": "fix",
                                                  "version": 1, "text": text } } }),
            json!({ "jsonrpc": "2.0", "id": 2, "method": "textDocument/completion",
                    "params": { "textDocument": { "uri": uri },
                                "position": { "line": 7, "character": 4 } } }),
            json!({ "jsonrpc": "2.0", "id": 3, "method": "textDocument/semanticTokens/full",
                    "params": { "textDocument": { "uri": uri } } }),
        ];
        let mut bytes = vec![];
        for message in &messages {
            let body = message.to_string();
            write!(bytes, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        }
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&bytes).unwrap();
        drop(stdin);

        wait_within(
            &mut child,
            Duration::from_secs(60),
            "the LSP server after stdin reached EOF",
        );
        let out = reader.join().unwrap();
        let answered: Vec<u64> = out
            .split("Content-Length:")
            .filter_map(|frame| frame.split_once("\r\n\r\n"))
            .filter_map(|(_, body)| serde_json::from_str::<Value>(body).ok())
            .filter(|message| message.get("method").is_none())
            .filter_map(|message| message.get("id").and_then(Value::as_u64))
            .collect();
        assert_eq!(
            answered,
            vec![1, 2, 3],
            "every request sent before the EOF is expected to be answered, in order"
        );
    }
}
