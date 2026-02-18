use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::misc::{to_absolute_path, Map};

pub struct LspClient {
    process: Child,
    stdin: ChildStdin,
    working_dir: PathBuf,
    document_versions: Map<PathBuf, i32>,
    message_queue: Arc<Mutex<VecDeque<Value>>>,
    responses: Arc<Mutex<Map<u32, Value>>>,
    diagnostics: Arc<Mutex<Map<PathBuf, Value>>>,
    next_id: u32,
    reader_thread_error: Arc<Mutex<Option<String>>>,
}

/// Process a received message and update internal state
fn process_message(
    message: Value,
    responses: &Arc<Mutex<Map<u32, Value>>>,
    diagnostics: &Arc<Mutex<Map<PathBuf, Value>>>,
    message_queue: &Arc<Mutex<VecDeque<Value>>>,
) {
    // Check if it's a response (has an id field)
    if let Some(id) = message.get("id") {
        if let Some(id_num) = id.as_u64() {
            responses
                .lock()
                .unwrap()
                .insert(id_num as u32, message.clone());
        }
    }

    // Check if it's a publishDiagnostics notification
    if let Some(method) = message.get("method") {
        if method.as_str() == Some("textDocument/publishDiagnostics") {
            if let Some(params) = message.get("params") {
                if let Some(uri) = params.get("uri") {
                    if let Some(uri_str) = uri.as_str() {
                        // Extract file path from URI (file:///path/to/file)
                        if let Some(path_str) = uri_str.strip_prefix("file://") {
                            let file_path = PathBuf::from(path_str);
                            if let Some(diagnostics_value) = params.get("diagnostics") {
                                diagnostics
                                    .lock()
                                    .unwrap()
                                    .insert(file_path, diagnostics_value.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Add to message queue for test code to inspect
    message_queue.lock().unwrap().push_back(message);
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

        let mut process = Command::new("fix")
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
        let message_queue = Arc::new(Mutex::new(VecDeque::new()));
        let responses = Arc::new(Mutex::new(Map::default()));
        let diagnostics = Arc::new(Mutex::new(Map::default()));
        let reader_thread_error = Arc::new(Mutex::new(None));

        // Clone Arcs for the reader thread
        let message_queue_clone = Arc::clone(&message_queue);
        let responses_clone = Arc::clone(&responses);
        let diagnostics_clone = Arc::clone(&diagnostics);
        let reader_thread_error_clone = Arc::clone(&reader_thread_error);

        // Start dedicated reader thread (detached - JoinHandle is not stored)
        // The thread will exit when stdout is closed (process termination) or on protocol error
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let result: Result<Value, String> = (|| {
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

                match result {
                    Ok(message) => {
                        process_message(
                            message,
                            &responses_clone,
                            &diagnostics_clone,
                            &message_queue_clone,
                        );
                    }
                    Err(e) => {
                        // EOF or protocol error - exit the loop
                        if e.contains("EOF") {
                            break;
                        }
                        // Store error before panicking
                        *reader_thread_error_clone.lock().unwrap() = Some(e.clone());
                        panic!("LSP protocol error: {}", e);
                    }
                }
            }
        });

        // Give the server a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(LspClient {
            process,
            stdin,
            working_dir: absolute_working_dir,
            document_versions: Map::default(),
            message_queue,
            responses,
            diagnostics,
            next_id: 1,
            reader_thread_error,
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

    /// Wait for server messages for the specified duration
    pub fn wait_for_server(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }

    /// Pop one message from the message queue
    #[allow(dead_code)]
    pub fn pop_message(&mut self) -> Option<Value> {
        self.message_queue.lock().unwrap().pop_front()
    }

    /// Get a response for a specific request ID
    /// Returns None if the response has not been received yet
    /// Removes the response from the internal map when retrieved
    pub fn get_response(&mut self, id: u32) -> Option<Value> {
        self.responses.lock().unwrap().remove(&id)
    }

    /// Get diagnostics for a specific file
    /// Returns a vector of diagnostic messages (empty if no diagnostics)
    #[allow(dead_code)]
    pub fn get_diagnostics(&self, file_path: &Path) -> Vec<Value> {
        let absolute_path = self.working_dir.join(file_path);
        let diagnostics = self.diagnostics.lock().unwrap();
        if let Some(diagnostics_value) = diagnostics.get(&absolute_path) {
            if let Some(arr) = diagnostics_value.as_array() {
                return arr.clone();
            }
        }
        Vec::new()
    }

    /// Get all diagnostics for all files
    /// Returns a map of file paths to their diagnostic messages
    pub fn get_all_diagnostics(&self) -> Map<PathBuf, Vec<Value>> {
        let diagnostics = self.diagnostics.lock().unwrap();
        let mut result = Map::default();
        for (file_path, diagnostics_value) in diagnostics.iter() {
            if let Some(arr) = diagnostics_value.as_array() {
                result.insert(file_path.clone(), arr.clone());
            }
        }
        result
    }

    /// Verify that there are no diagnostic errors for any file
    /// Returns an error if any diagnostics contain errors
    pub fn verify_no_diagnostic_errors(&self) -> Result<(), String> {
        let diagnostics = self.diagnostics.lock().unwrap();
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

    /// Execute initialization sequence with custom timeout
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

    /// Send didOpen notification for a document
    ///
    /// Takes a file path relative to the project root, reads the file content,
    /// and sends a didOpen notification to the language server.
    /// Initializes the document version to 1.
    ///
    /// Returns an error if the document is already opened.
    pub fn open_document(&mut self, file_path: &Path) -> Result<(), String> {
        const INITIAL_VERSION_NUMBER: i32 = 1;

        let absolute_path = self.working_dir.join(file_path);

        // Check if already opened
        if self.document_versions.contains_key(&absolute_path) {
            return Err(format!("Document {:?} is already opened", file_path));
        }

        let text = fs::read_to_string(&absolute_path)
            .map_err(|e| format!("Failed to read file {:?}: {:?}", absolute_path, e))?;
        let uri = format!("file://{}", absolute_path.display());

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
    #[allow(dead_code)]
    pub fn change_document(&mut self, file_path: &Path) -> Result<(), String> {
        let absolute_path = self.working_dir.join(file_path);
        let text = fs::read_to_string(&absolute_path)
            .map_err(|e| format!("Failed to read file {:?}: {:?}", absolute_path, e))?;
        let uri = format!("file://{}", absolute_path.display());

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
        let text = fs::read_to_string(&absolute_path)
            .map_err(|e| format!("Failed to read file {:?}: {:?}", absolute_path, e))?;
        let uri = format!("file://{}", absolute_path.display());

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

    /// Shutdown
    ///
    /// # Arguments
    /// * `exit_timeout` - Maximum time to wait for the process to exit after sending exit notification
    pub fn shutdown(&mut self, exit_timeout: Duration) -> Result<(), String> {
        let id = self.send_request("shutdown", json!(null))?;

        // Wait for response with 5 second timeout
        let timeout = Duration::from_secs(5);
        self.wait_for_server(timeout);
        let _ = self.get_response(id);

        self.send_notification("exit", json!(null))?;

        // Wait for process to exit with timeout to avoid freezing tests
        // If the process doesn't exit within the timeout, return error (Drop will kill it)
        std::thread::sleep(exit_timeout);

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

    /// Check for reader thread errors and panic if any occurred
    ///
    /// This should be called at the end of tests to ensure that
    /// LSP protocol errors in the reader thread are properly reported.
    pub fn finish(&self) -> Result<(), String> {
        let error = self.reader_thread_error.lock().unwrap();
        if let Some(err_msg) = error.as_ref() {
            return Err(format!("LSP protocol error occurred: {}", err_msg));
        }
        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Kill process if still running
        let _ = self.process.kill();
        // Note: We don't join the reader thread here as it may block.
        // The thread will be detached and will exit when stdout is closed.
    }
}
