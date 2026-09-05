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
    /// Number of `$/progress` end notifications received so far.
    progress_end_count: Arc<Mutex<usize>>,
    /// The protocol error the reader thread stopped on, which `finish` hands to the test.
    reader_thread_error: Arc<Mutex<Option<String>>>,
}

impl SharedState {
    /// A state holding no messages, no responses, no diagnostics and no error.
    fn new() -> Self {
        SharedState {
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            responses: Arc::new(Mutex::new(Map::default())),
            diagnostics: Arc::new(Mutex::new(Map::default())),
            progress_end_count: Arc::new(Mutex::new(0)),
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

/// Process a received message and update internal state
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
        *shared.progress_end_count.lock().unwrap() += 1;
    }

    // Check if it's a publishDiagnostics notification
    process_publish_diagnostics(&message, shared);

    // Add to message queue for test code to inspect
    shared.message_queue.lock().unwrap().push_back(message);
}

impl LspClient {
    /// Start fix command in language server mode
    ///
    /// The working_dir can be either a relative or absolute path.
    /// It will be converted to an absolute path internally.
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

        // Start dedicated reader thread (detached - JoinHandle is not stored)
        // The thread will exit when stdout is closed (process termination) or on protocol error
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

        // Give the server a moment to initialize
        thread::sleep(Duration::from_millis(100));

        Ok(LspClient {
            process,
            stdin,
            working_dir: absolute_working_dir,
            document_versions: Map::default(),
            shared,
            next_id: 1,
        })
    }

    /// Send LSP request
    pub fn send_request(&mut self, method: &str, params: Value) -> Result<u32, String> {
        let id = self.next_id;
        self.next_id += 1;

        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let content = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize request: {:?}", e))?;

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

        Ok(id)
    }

    /// Send LSP notification
    pub fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let content = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize notification: {:?}", e))?;

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

    /// Sleep for `duration`, giving the reader thread that long to take in whatever the server
    /// sends meanwhile.
    pub fn wait_for_server(&mut self, duration: Duration) {
        thread::sleep(duration);
    }

    /// Pop one message from the message queue
    pub fn pop_message(&mut self) -> Option<Value> {
        self.shared.message_queue.lock().unwrap().pop_front()
    }

    /// The response to the request `id`, taken out of the responses so that it is handed over
    /// once. `None` says the response is yet to arrive.
    pub fn get_response(&mut self, id: u32) -> Option<Value> {
        self.shared.responses.lock().unwrap().remove(&id)
    }

    /// Return the number of `$/progress` end notifications received so far.
    pub fn count_progress_end_messages(&self) -> usize {
        *self.shared.progress_end_count.lock().unwrap()
    }

    /// Wait until the total number of `$/progress` end notifications
    /// reaches at least `target_count`.
    ///
    /// This is used to detect when diagnostics have completed, since the
    /// server sends `$/progress` with `kind: "end"` after each diagnostics run.
    pub fn wait_for_progress_end_count(
        &self,
        target_count: usize,
        timeout: Duration,
    ) -> Result<(), String> {
        let start = Instant::now();
        loop {
            if self.count_progress_end_messages() >= target_count {
                return Ok(());
            }
            if start.elapsed() > timeout {
                return Err(format!(
                    "Timeout ({:?}) waiting for progress end count to reach {}. Current: {}",
                    timeout,
                    target_count,
                    self.count_progress_end_messages()
                ));
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    /// Trigger diagnostics for `file` and wait until the server has processed the results.
    ///
    /// The server sends a `$/progress` notification with `kind: "end"` when diagnostics
    /// complete, and the wait ends on that notification.
    ///
    /// After diagnostics complete, the result is on the internal channel
    /// but the server's main loop must call `try_recv` to pick it up.
    /// Sending a `$/ping` notification (unknown to the server, safely ignored)
    /// forces the main loop to iterate, executing `try_recv`.
    pub fn trigger_and_wait_for_diagnostics(&mut self, file: &Path) {
        let count_before = self.count_progress_end_messages();

        // Save triggers diagnostics.
        self.save_document(file).expect("Failed to save document");

        // Wait for diagnostics to complete ($/progress end notification).
        self.wait_for_progress_end_count(count_before + 1, Duration::from_secs(60))
            .expect("Diagnostics did not complete in time");

        // Force the server's main loop to iterate, ensuring it picks up
        // the diagnostics result via try_recv.
        self.send_notification("$/ping", json!(null))
            .expect("Failed to send flush notification");
        self.wait_for_server(Duration::from_secs(1));
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

    /// Checks that the diagnostics of every file are empty, answering with an error that names a
    /// file carrying any and shows what it carries.
    pub fn verify_no_diagnostic_errors(&self) -> Result<(), String> {
        let diagnostics = self.shared.diagnostics.lock().unwrap();
        for (file_path, diagnostics_value) in diagnostics.iter() {
            if let Some(diag_array) = diagnostics_value.as_array() {
                if !diag_array.is_empty() {
                    return Err(format!(
                        "Expected no diagnostic errors but found errors in {:?}: {:?}",
                        file_path, diag_array
                    ));
                }
            }
        }
        Ok(())
    }

    /// Run the initialization handshake: send the `initialize` request, wait for its response,
    /// then send the `initialized` notification the server starts its diagnostics on.
    ///
    /// # Arguments
    /// * `root_path` - Project root directory path (can be relative or absolute)
    /// * `timeout` - Maximum time to wait for initialize response
    pub fn initialize(&mut self, root_path: &Path, timeout: Duration) -> Result<(), String> {
        // Convert to absolute path
        let absolute_root = to_absolute_path(root_path)
            .map_err(|e| format!("Failed to convert root_path to absolute path: {}", e))?;
        let root_uri = format!("file://{}", absolute_root.display());

        let params = json!({
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {}
        });

        let id = self.send_request("initialize", params)?;

        // Wait for response
        self.wait_for_server(timeout);
        if let Some(response) = self.get_response(id) {
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
        let uri = format!("file://{}", absolute_path.display());
        Ok((text, uri))
    }

    /// Send didOpen notification for a document
    ///
    /// Takes a file path relative to the project root, reads the file content,
    /// and sends a didOpen notification to the language server.
    /// Initializes the document version to 1.
    ///
    /// Returns an error if the document is already opened.
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

    /// Send didChange notification for a document
    ///
    /// Takes a file path relative to the project root, reads the file content,
    /// increments the document version, and sends a didChange notification to the language server.
    /// The document must have been opened with open_document first.
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

    /// Send didSave notification for a document
    ///
    /// Takes a file path relative to the project root, reads the file content,
    /// and sends a didSave notification to the language server.
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

    /// Ask the server to shut down and exit, and wait for its process to end.
    ///
    /// # Arguments
    /// * `exit_timeout` - Maximum time to wait for the process to exit after sending exit notification
    pub fn shutdown(&mut self, exit_timeout: Duration) -> Result<(), String> {
        let id = self.send_request("shutdown", json!(null))?;

        // Wait for response with 5 second timeout
        let response_timeout = Duration::from_secs(5);
        self.wait_for_server(response_timeout);
        let _ = self.get_response(id);

        self.send_notification("exit", json!(null))?;

        // Wait for process to exit with timeout to avoid freezing tests
        // If the process doesn't exit within the timeout, return error (Drop will kill it)
        thread::sleep(exit_timeout);

        match self.process.try_wait() {
            Ok(Some(_status)) => {
                // Process has already exited
            }
            Ok(None) => {
                // Process is still running - return error
                // Drop will kill the process when LspClient is dropped
                return Err("LSP server did not exit gracefully within timeout".to_string());
            }
            Err(e) => {
                return Err(format!("Failed to check process status: {:?}", e));
            }
        }

        Ok(())
    }

    /// The protocol error the reader thread met, as an `Err`. Called at the end of a test, so
    /// that an error met on a thread of its own reaches the test's result.
    pub fn finish(&self) -> Result<(), String> {
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
