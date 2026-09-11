//! What the waits the harness is built on promise.
//!
//! A test in this directory reads what the server sent once a wait has returned, so what the wait
//! promises is what that reading means.

#[cfg(test)]
mod tests {
    use super::super::completion_harness::setup_test_env;
    use super::super::lsp_client::{poll_every, LspClient};
    use serde_json::{json, Value};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    /// A wait whose condition already holds hands that look's answer back and stops, so that a
    /// test asking for what the server has already sent pays nothing for the asking.
    ///
    /// The timeout is empty, which leaves the wait with no time to spend, and the interval is
    /// long enough that a wait spending any shows it.
    #[test]
    fn test_a_wait_answers_a_condition_that_already_holds() {
        /// The gap between two looks.
        const INTERVAL: Duration = Duration::from_secs(5);

        let mut looks = 0;
        let started = Instant::now();
        let answer = poll_every(INTERVAL, Duration::ZERO, || {
            looks += 1;
            Some(looks)
        });

        assert_eq!(
            answer,
            Some(1),
            "the wait is expected to hand back the answer of the look that answered"
        );
        assert_eq!(
            looks, 1,
            "the wait is expected to stop at the look that answered, but it looked {} times",
            looks
        );
        assert!(
            started.elapsed() < INTERVAL,
            "the wait took {:?}, which is the interval it is expected to spend none of",
            started.elapsed()
        );
    }

    /// A wait looks once more when its time runs out, so that a condition turning true while the
    /// wait sleeps is answered rather than missed.
    ///
    /// The interval is the whole timeout, so the wait sleeps once and the look after that sleep
    /// is the one taken at the deadline. The condition answers nothing on the first look, so a
    /// wait that gives up without taking the second one answers nothing at all.
    #[test]
    fn test_a_wait_looks_once_more_when_its_time_runs_out() {
        let span = Duration::from_millis(100);
        let mut looks = 0;
        let answer = poll_every(span, span, || {
            looks += 1;
            (looks > 1).then_some(looks)
        });

        assert!(
            answer.is_some(),
            "the wait gave up after {} look(s), without looking again when its time ran out",
            looks
        );
    }

    /// The project the session tests run over, whose `main.fix` they write.
    const PROJECT: &str = "goto_local";

    /// A program carrying one error, which the analysis finishes and reports.
    const PROGRAM_WITH_AN_ERROR: &str =
        "module Main;\n\nmain : IO ();\nmain = println(nonexistent_name);\n";

    /// A program declaring `main` alone.
    const PROGRAM_WITHOUT_THE_GLOBAL: &str =
        "module Main;\n\nmain : IO ();\nmain = println(\"\");\n";

    /// A program declaring the global `answer` besides `main`.
    const PROGRAM_WITH_THE_GLOBAL: &str =
        "module Main;\n\nanswer : I64;\nanswer = 42;\n\nmain : IO ();\nmain = println(answer.to_string);\n";

    /// A session over a copy of `PROJECT` whose `main.fix` carries `program`, with `main.fix`
    /// opened.
    fn open_session(program: &str) -> (TempDir, PathBuf, LspClient) {
        let (temp_dir, project_dir) = setup_test_env(PROJECT);
        fs::write(project_dir.join("main.fix"), program).expect("Failed to write main.fix");
        let mut client = LspClient::new(&project_dir).expect("Failed to start LSP");
        client
            .initialize(&project_dir, Duration::from_secs(10))
            .expect("Failed to initialize LSP");
        client
            .open_document(Path::new("main.fix"))
            .expect("Failed to open main.fix");
        (temp_dir, project_dir, client)
    }

    /// The name of each symbol the server answers for the file `uri` names.
    fn document_symbol_names(client: &mut LspClient, uri: &str) -> Vec<String> {
        let id = client
            .send_request(
                "textDocument/documentSymbol",
                json!({ "textDocument": { "uri": uri } }),
            )
            .expect("Failed to send documentSymbol");
        client.response_of(id)["result"]
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

    /// Every message the server has sent that the client has yet to look at, oldest first.
    fn recorded_messages(client: &mut LspClient) -> Vec<Value> {
        let mut messages = Vec::new();
        while let Some(message) = client.pop_message() {
            messages.push(message);
        }
        messages
    }

    /// Whether `message` publishes at least one report.
    fn publishes_a_report(message: &Value) -> bool {
        message["method"] == json!("textDocument/publishDiagnostics")
            && message["params"]["diagnostics"]
                .as_array()
                .is_some_and(|reports| !reports.is_empty())
    }

    /// Whether `message` is the `$/progress` notification a pass ends with.
    fn ends_a_pass(message: &Value) -> bool {
        message["method"] == json!("$/progress")
            && message["params"]["value"]["kind"] == json!("end")
    }

    /// The wait for a pass returns with the reports that pass published, which rests on the pass
    /// sending its end after everything it publishes.
    #[test]
    fn test_the_wait_for_a_pass_returns_with_the_reports_the_pass_published() {
        let main_fix = Path::new("main.fix");
        let (_temp_dir, _project_dir, mut client) = open_session(PROGRAM_WITH_AN_ERROR);

        client.save_and_wait_for_a_pass(main_fix);

        assert!(
            !client.get_diagnostics(main_fix).is_empty(),
            "the wait ended on the pass over a program carrying an error, so `main.fix` is \
             expected to carry that pass's report"
        );

        let messages = recorded_messages(&mut client);
        let published = messages
            .iter()
            .position(publishes_a_report)
            .expect("the pass over a program carrying an error is expected to publish a report");
        let ended = messages
            .iter()
            .position(ends_a_pass)
            .expect("the pass is expected to end");
        assert!(
            published < ended,
            "a pass is expected to publish its reports before it ends, but it ended at message {} \
             and published its first report at message {}",
            ended,
            published
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }

    /// The wait for the program returns with the server's main loop holding the program of the
    /// pass the save asked for, so the request sent next is answered out of that program.
    #[test]
    fn test_the_wait_for_the_program_returns_with_this_passs_program_in_hand() {
        let main_fix = Path::new("main.fix");
        let (_temp_dir, project_dir, mut client) = open_session(PROGRAM_WITHOUT_THE_GLOBAL);
        client.save_and_wait_for_the_program(main_fix);

        // The second pass, over a program declaring a global the first pass's program has none of.
        fs::write(project_dir.join(main_fix), PROGRAM_WITH_THE_GLOBAL)
            .expect("Failed to write the program declaring the global");
        client.save_and_wait_for_the_program(main_fix);

        let uri = client.file_uri(main_fix);
        let names = document_symbol_names(&mut client, &uri);
        assert!(
            names.contains(&"Main::answer".to_string()),
            "the symbols of `main.fix` are answered out of the program the main loop holds, which \
             is expected to be the one declaring `Main::answer`, but the answer carries {:?}",
            names
        );

        client
            .shutdown(Duration::from_millis(500))
            .expect("Failed to shutdown LSP");
        client
            .finish()
            .expect("Reader thread should not have errors");
    }
}
