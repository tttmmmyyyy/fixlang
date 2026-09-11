use crate::misc::{to_absolute_path, Map};
use crate::tests::test_util::fix_command;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// How long a wait sits between two looks at what has arrived.
const POLL_INTERVAL: Duration = Duration::from_millis(2);

/// Call `look` every `interval` until it answers `Some`, and hand that answer back. `None` says
/// `timeout` ran out with `look` still answering `None`.
///
/// `interval` is what one look costs: `POLL_INTERVAL` where a look reads what the reader thread
/// has already taken in, and a round trip's worth where each look asks the server again.
pub(super) fn poll_every<T>(
    interval: Duration,
    timeout: Duration,
    mut look: impl FnMut() -> Option<T>,
) -> Option<T> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(answer) = look() {
            return Some(answer);
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(interval);
    }
}

/// Call `look` every `POLL_INTERVAL` until it answers `Some`. The interval suits a look that
/// reads what the reader thread has already taken in.
pub(super) fn poll<T>(timeout: Duration, look: impl FnMut() -> Option<T>) -> Option<T> {
    poll_every(POLL_INTERVAL, timeout, look)
}

/// The `file://` URI naming `absolute_path`.
fn uri_of(absolute_path: &Path) -> String {
    format!("file://{}", absolute_path.display())
}

/// Shared state between `LspClient` and the background reader thread.
/// Each field is an `Arc<Mutex<T>>` so `SharedState` can be cheaply cloned
/// to share the same data across threads.
#[derive(Clone)]
struct SharedState {
    /// Every message the server sent, oldest first, for the test to look through.
    message_queue: Arc<Mutex<VecDeque<Value>>>,
    /// The response to each request the client sent, under the request's id.
    responses: Arc<Mutex<Map<u32, Value>>>,
    /// The diagnostics last published for each file, under the file's absolute path.
    diagnostics: Arc<Mutex<Map<PathBuf, Value>>>,
    /// How many diagnostics passes have ended, counted by the `$/progress` end notifications
    /// that mark them.
    ended_pass_count: Arc<Mutex<usize>>,
    /// The protocol error the reader thread stopped on, which `verify_no_protocol_error` hands
    /// to the test.
    reader_thread_error: Arc<Mutex<Option<String>>>,
}

impl SharedState {
    /// A state holding no messages, no responses, no diagnostics and no error.
    fn new() -> Self {
        SharedState {
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            responses: Arc::new(Mutex::new(Map::default())),
            diagnostics: Arc::new(Mutex::new(Map::default())),
            ended_pass_count: Arc::new(Mutex::new(0)),
            reader_thread_error: Arc::new(Mutex::new(None)),
        }
    }
}

/// A test's end of a session with `fix language-server`: it runs the server as a child process,
/// speaks the protocol to it over that process's pipes, and keeps what the server sent back so
/// that a test can assert on it.
pub struct LspClient {
    /// The server process, which `Drop` kills.
    process: Child,
    /// The pipe the client writes its messages into.
    stdin: ChildStdin,
    /// The project root, in absolute form. The paths a test passes are taken as relative to it.
    working_dir: PathBuf,
    /// The version last sent for each opened document, under the document's absolute path. The
    /// protocol asks each change to carry a version higher than the one before it.
    document_versions: Map<PathBuf, i32>,
    /// What the reader thread has taken in from the server.
    shared: SharedState,
    /// The id the next request the client sends will carry.
    next_id: u32,
}

/// Whether `message` is a `textDocument/publishDiagnostics` notification.
fn is_publish_diagnostics(message: &Value) -> bool {
    message.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics")
}

/// Take `message` into `shared`: the response to a request under the request's id, the end of a
/// diagnostics pass into the count of the passes that have ended, the diagnostics of a file under
/// the file's path, and the message itself into the queue.
fn process_message(message: Value, shared: &SharedState) {
    /// Handle a `textDocument/publishDiagnostics` notification.
    fn process_publish_diagnostics(message: &Value, shared: &SharedState) {
        if !is_publish_diagnostics(message) {
            return;
        }
        let Some(params) = message.get("params") else {
            return;
        };
        let Some(uri_str) = params.get("uri").and_then(|u| u.as_str()) else {
            return;
        };
        // Extract file path from URI (file:///path/to/file)
        let Some(path_str) = uri_str.strip_prefix("file://") else {
            return;
        };
        let file_path = PathBuf::from(path_str);
        let Some(diagnostics_value) = params.get("diagnostics") else {
            return;
        };
        shared
            .diagnostics
            .lock()
            .unwrap()
            .insert(file_path, diagnostics_value.clone());
    }

    // Check if it's a response to one of our requests: it carries an `id` but
    // no `method`. Messages with a `method` and an `id` are server-initiated
    // requests (e.g. `workspace/semanticTokens/refresh`), whose ids live in a
    // separate space and must not clobber our response map.
    if message.get("method").is_none() {
        if let Some(id) = message.get("id") {
            if let Some(id_num) = id.as_u64() {
                shared
                    .responses
                    .lock()
                    .unwrap()
                    .insert(id_num as u32, message.clone());
            }
        }
    }

    // Check if it's a $/progress end notification
    if message.get("method").and_then(|m| m.as_str()) == Some("$/progress")
        && message
            .get("params")
            .and_then(|p| p.get("value"))
            .and_then(|v| v.get("kind"))
            .and_then(|k| k.as_str())
            == Some("end")
    {
        *shared.ended_pass_count.lock().unwrap() += 1;
    }

    // Check if it's a publishDiagnostics notification
    process_publish_diagnostics(&message, shared);

    // Add to message queue for test code to inspect
    shared.message_queue.lock().unwrap().push_back(message);
}

impl LspClient {
    /// Run `fix language-server` in `working_dir`, which is the project root the paths a test
    /// passes are taken as relative to, and read what it sends on a thread of its own.
    /// `working_dir` itself may be relative to the directory the test runs in.
    pub fn new(working_dir: &Path) -> Result<Self, String> {
        // Convert to absolute path
        let absolute_working_dir = to_absolute_path(working_dir)
            .map_err(|e| format!("Failed to convert to absolute path: {}", e))?;

        let mut process = fix_command()
            .arg("language-server")
            .current_dir(&absolute_working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn fix language-server: {:?}", e))?;

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        // Create shared data structures
        let shared = SharedState::new();
        let shared_clone = shared.clone();

        // The reader thread runs on its own, and ends when the process closes its stdout or when
        // it meets a protocol error.
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let read_message: Result<Value, String> = (|| {
                    // Read Content-Length header
                    let mut header_line = String::new();
                    reader
                        .read_line(&mut header_line)
                        .map_err(|e| format!("Failed to read header: {:?}", e))?;

                    if header_line.is_empty() {
                        return Err("EOF reached while reading header".to_string());
                    }

                    let trimmed = header_line.trim();
                    if !trimmed.starts_with("Content-Length: ") {
                        return Err(format!(
                            "Invalid header format. Expected 'Content-Length: ...', but got: {:?}",
                            header_line
                        ));
                    }
                    let content_length: usize = trimmed
                        .strip_prefix("Content-Length: ")
                        .unwrap()
                        .parse()
                        .map_err(|e| format!("Failed to parse content length: {:?}", e))?;

                    // Skip empty line
                    let mut empty_line = String::new();
                    reader
                        .read_line(&mut empty_line)
                        .map_err(|e| format!("Failed to read empty line: {:?}", e))?;

                    // Read content
                    let mut content = vec![0u8; content_length];
                    reader
                        .read_exact(&mut content)
                        .map_err(|e| format!("Failed to read content: {:?}", e))?;

                    let message: Value = serde_json::from_slice(&content)
                        .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

                    Ok(message)
                })();

                match read_message {
                    Ok(message) => {
                        process_message(message, &shared_clone);
                    }
                    Err(e) => {
                        // EOF or protocol error - exit the loop
                        if e.contains("EOF") {
                            break;
                        }
                        // Store error before panicking
                        *shared_clone.reader_thread_error.lock().unwrap() = Some(e.clone());
                        panic!("LSP protocol error: {}", e);
                    }
                }
            }
        });

        Ok(LspClient {
            process,
            stdin,
            working_dir: absolute_working_dir,
            document_versions: Map::default(),
            shared,
            next_id: 1,
        })
    }

    /// Write `message` to the server, headed by the `Content-Length` the protocol frames a
    /// message with.
    fn send_message(&mut self, message: &Value) -> Result<(), String> {
        let content = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {:?}", e))?;

        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        self.stdin
            .write_all(header.as_bytes())
            .map_err(|e| format!("Failed to write header: {:?}", e))?;
        self.stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write content: {:?}", e))?;
        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush: {:?}", e))?;

        Ok(())
    }

    /// Send the request `method` with `params`, and hand back the id it was sent under, which the
    /// response to it carries.
    pub fn send_request(&mut self, method: &str, params: Value) -> Result<u32, String> {
        let id = self.next_id;
        self.next_id += 1;

        self.send_message(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))?;

        Ok(id)
    }

    /// Send the notification `method` with `params`, which the server answers nothing to.
    pub fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    /// The oldest message the server sent that the queue still holds, taken out of it.
    pub fn pop_message(&mut self) -> Option<Value> {
        self.shared.message_queue.lock().unwrap().pop_front()
    }

    /// The response to the request `id`, waited for until it arrives or `timeout` runs out.
    /// `None` says the wait ran out.
    pub fn wait_for_response(&mut self, id: u32, timeout: Duration) -> Option<Value> {
        poll(timeout, || self.take_response(id))
    }

    /// The response to the request `id`, which is expected to arrive within `RESPONSE_TIMEOUT`.
    pub fn expect_response(&mut self, id: u32) -> Value {
        self.wait_for_response(id, Self::RESPONSE_TIMEOUT)
            .unwrap_or_else(|| panic!("the request {} is expected to be answered", id))
    }

    /// The response to the request `id`, taken out of the responses so that it is handed over
    /// once. `None` says the response is yet to arrive.
    pub fn take_response(&mut self, id: u32) -> Option<Value> {
        self.shared.responses.lock().unwrap().remove(&id)
    }

    /// How many diagnostics passes have ended so far.
    pub fn ended_pass_count(&self) -> usize {
        *self.shared.ended_pass_count.lock().unwrap()
    }

    /// Wait until at least `target_count` diagnostics passes have ended. The server marks the
    /// end of each pass with a `$/progress` notification carrying `kind: "end"`, which is what
    /// the count is taken from.
    pub fn wait_for_ended_passes(
        &self,
        target_count: usize,
        timeout: Duration,
    ) -> Result<(), String> {
        poll(timeout, || {
            (self.ended_pass_count() >= target_count).then_some(())
        })
        .ok_or_else(|| {
            format!(
                "Timeout ({:?}) waiting for {} passes to end. Ended: {}",
                timeout,
                target_count,
                self.ended_pass_count()
            )
        })
    }

    /// How long a request is given to be answered.
    pub const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

    /// How long a diagnostics pass is given to end.
    pub const PASS_TIMEOUT: Duration = Duration::from_secs(180);

    /// Run `trigger`, which asks the server for a diagnostics pass, and return once one more pass
    /// has ended than had ended before it ran, so that the reports that pass publishes have
    /// arrived.
    pub fn wait_for_one_more_pass(&mut self, trigger: impl FnOnce(&mut Self)) {
        let passes_before = self.ended_pass_count();
        trigger(self);
        self.wait_for_ended_passes(passes_before + 1, Self::PASS_TIMEOUT)
            .expect("the pass the buffer asks for is expected to end");
    }

    /// Save `file` and return once the pass it triggers has ended, so that the reports the pass
    /// publishes have arrived.
    pub fn save_and_wait_for_a_pass(&mut self, file: &Path) {
        self.wait_for_one_more_pass(|client| {
            client.save_document(file).expect("Failed to save document")
        });
    }

    /// Save `file` and return once the server's main loop holds what the pass it triggers
    /// produced, so that a request answered out of the elaborated program sees this pass's
    /// program.
    ///
    /// The main loop takes a pass's result in at the top of an iteration, before it blocks
    /// reading the next message, so the result of a pass that has just ended reaches it only once
    /// it has read one more message. The wait therefore ends the pass, sends a notification the
    /// loop reads and ignores, and then waits out one request: the notification is what the loop
    /// reads before it takes the result in, and the answer to the request is what says it has.
    ///
    /// The request is `textDocument/semanticTokens/full` because it is the one the server answers
    /// whether or not it holds a program — which is what a project whose source fails to
    /// elaborate needs.
    pub fn save_and_wait_for_the_program(&mut self, file: &Path) {
        self.save_and_wait_for_a_pass(file);
        self.send_notification("$/ping", json!(null))
            .expect("Failed to send the notification the main loop reads before the result");
        let id = self
            .send_request(
                "textDocument/semanticTokens/full",
                json!({ "textDocument": { "uri": self.file_uri(file) } }),
            )
            .expect("Failed to send the request that waits the main loop out");
        self.wait_for_response(id, Self::RESPONSE_TIMEOUT)
            .expect("the request that waits the main loop out is expected to be answered");
    }

    /// The `file://` URI the server knows `file` by, `file` being taken as relative to the
    /// project root.
    pub fn file_uri(&self, file: &Path) -> String {
        uri_of(&self.working_dir.join(file))
    }

    /// The diagnostics the server last published for `file_path`, which is taken as relative to
    /// the project root. A file the server has published nothing for carries an empty vector.
    pub fn get_diagnostics(&self, file_path: &Path) -> Vec<Value> {
        let absolute_path = self.working_dir.join(file_path);
        let diagnostics = self.shared.diagnostics.lock().unwrap();
        if let Some(diagnostics_value) = diagnostics.get(&absolute_path) {
            if let Some(arr) = diagnostics_value.as_array() {
                return arr.clone();
            }
        }
        Vec::new()
    }

    /// The diagnostics the server last published for each file, under the file's absolute path.
    pub fn get_all_diagnostics(&self) -> Map<PathBuf, Vec<Value>> {
        let diagnostics = self.shared.diagnostics.lock().unwrap();
        let mut diagnostics_by_path = Map::default();
        for (file_path, diagnostics_value) in diagnostics.iter() {
            if let Some(arr) = diagnostics_value.as_array() {
                diagnostics_by_path.insert(file_path.clone(), arr.clone());
            }
        }
        diagnostics_by_path
    }

    /// Checks that no file carries a diagnostic of any severity, answering with an error that
    /// names a file carrying one and shows what it carries.
    pub fn verify_no_diagnostics(&self) -> Result<(), String> {
        let diagnostics = self.shared.diagnostics.lock().unwrap();
        for (file_path, diagnostics_value) in diagnostics.iter() {
            if let Some(diag_array) = diagnostics_value.as_array() {
                if !diag_array.is_empty() {
                    return Err(format!(
                        "Expected no diagnostics but found some in {:?}: {:?}",
                        file_path, diag_array
                    ));
                }
            }
        }
        Ok(())
    }

    /// Run the initialization handshake: send the `initialize` request naming `root_path` as the
    /// project root, wait up to `timeout` for its response, then send the `initialized`
    /// notification the server starts its diagnostics on. `root_path` may itself be relative to
    /// the directory the test runs in.
    pub fn initialize(&mut self, root_path: &Path, timeout: Duration) -> Result<(), String> {
        // Convert to absolute path
        let absolute_root = to_absolute_path(root_path)
            .map_err(|e| format!("Failed to convert root_path to absolute path: {}", e))?;
        let root_uri = uri_of(&absolute_root);

        let params = json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {}
        });

        let id = self.send_request("initialize", params)?;

        if let Some(response) = self.wait_for_response(id, timeout) {
            if response.get("error").is_some() {
                return Err(format!("Initialize failed: {:?}", response));
            }
            self.send_notification("initialized", json!({}))?;
            return Ok(());
        }

        Err(format!(
            "Timeout ({:?}) waiting for initialize response",
            timeout
        ))
    }

    /// The content of the document at `absolute_path`, and the URI naming it.
    fn read_document(absolute_path: &Path) -> Result<(String, String), String> {
        let text = fs::read_to_string(absolute_path)
            .map_err(|e| format!("Failed to read file {:?}: {:?}", absolute_path, e))?;
        Ok((text, uri_of(absolute_path)))
    }

    /// Tell the server that the client now holds `file_path`, whose content it reads from disk and
    /// sends along. `file_path` is taken as relative to the project root, and the version the
    /// client counts its changes from starts here.
    pub fn open_document(&mut self, file_path: &Path) -> Result<(), String> {
        /// The version the protocol counts an opened document from.
        const INITIAL_VERSION_NUMBER: i32 = 1;

        let absolute_path = self.working_dir.join(file_path);

        // Check if already opened
        if self.document_versions.contains_key(&absolute_path) {
            return Err(format!("Document {:?} is already opened", file_path));
        }

        let (text, uri) = Self::read_document(&absolute_path)?;

        // Set initial version
        self.document_versions
            .insert(absolute_path, INITIAL_VERSION_NUMBER);

        self.send_notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "fix",
                    "version": INITIAL_VERSION_NUMBER,
                    "text": text
                }
            }),
        )
    }

    /// Tell the server that what the client holds for `file_path` is now the content on disk,
    /// under the next version. `file_path` is taken as relative to the project root, and is
    /// expected to have been opened by `open_document`.
    pub fn change_document(&mut self, file_path: &Path) -> Result<(), String> {
        let absolute_path = self.working_dir.join(file_path);
        let (text, uri) = Self::read_document(&absolute_path)?;

        // Increment version
        let version = self
            .document_versions
            .get_mut(&absolute_path)
            .ok_or_else(|| format!("Document {:?} has not been opened yet", file_path))?;
        *version += 1;
        let current_version = *version;

        self.send_notification(
            "textDocument/didChange",
            json!({
                "textDocument": {
                    "uri": uri,
                    "version": current_version
                },
                "contentChanges": [
                    {
                        "text": text
                    }
                ]
            }),
        )
    }

    /// Tell the server that `file_path` has been saved, along with the content on disk, which is
    /// what asks it for a diagnostics pass. `file_path` is taken as relative to the project root.
    pub fn save_document(&mut self, file_path: &Path) -> Result<(), String> {
        let absolute_path = self.working_dir.join(file_path);
        let (text, uri) = Self::read_document(&absolute_path)?;

        self.send_notification(
            "textDocument/didSave",
            json!({
                "textDocument": {
                    "uri": uri
                },
                "text": text
            }),
        )
    }

    /// Ask the server to shut down and exit, and wait up to `exit_timeout` for its process to end
    /// once the `exit` notification has been sent.
    pub fn shutdown(&mut self, exit_timeout: Duration) -> Result<(), String> {
        let id = self.send_request("shutdown", json!(null))?;
        let _ = self.wait_for_response(id, Self::RESPONSE_TIMEOUT);

        self.send_notification("exit", json!(null))?;

        // A process still running when `exit_timeout` runs out is an error; `Drop` kills it.
        match poll(exit_timeout, || match self.process.try_wait() {
            Ok(Some(_status)) => Some(Ok(())),
            Ok(None) => None,
            Err(e) => Some(Err(format!("Failed to check process status: {:?}", e))),
        }) {
            Some(result) => result,
            None => Err("LSP server did not exit gracefully within timeout".to_string()),
        }
    }

    /// Checks that the reader thread met no protocol error, answering with the error it met.
    /// Called at the end of a test, so that an error met on a thread of its own reaches the
    /// test's result.
    pub fn verify_no_protocol_error(&self) -> Result<(), String> {
        let error = self.shared.reader_thread_error.lock().unwrap();
        if let Some(err_msg) = error.as_ref() {
            return Err(format!("LSP protocol error occurred: {}", err_msg));
        }
        Ok(())
    }
}

impl Drop for LspClient {
    /// Kills the server process, ending the session however the test left it.
    ///
    /// The reader thread is left to itself: it ends once the process it reads from closes its
    /// stdout, and a join here would block for as long as the process lives.
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
