use serde_json::{json, Value};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::misc::{to_absolute_path, Map};

pub struct LspClient {
    process: Child,
    stdin: ChildStdin,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    working_dir: PathBuf,
    document_versions: Map<PathBuf, i32>,
    message_queue: VecDeque<Value>,
    responses: Map<u32, Value>,
    diagnostics: Map<PathBuf, Value>,
    next_id: u32,
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
        let stdout = Arc::new(Mutex::new(BufReader::new(process.stdout.take().unwrap())));

        // Give the server a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(LspClient {
            process,
            stdin,
            stdout,
            working_dir: absolute_working_dir,
            document_versions: Map::default(),
            message_queue: VecDeque::new(),
            responses: Map::default(),
            diagnostics: Map::default(),
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

    /// Try to receive one message from the server without blocking
    fn try_receive_one_message(&mut self) -> Option<Value> {
        let (tx, rx) = mpsc::channel();
        let stdout_clone = Arc::clone(&self.stdout);

        std::thread::spawn(move || {
            let result: Result<Value, String> = (|| {
                let mut stdout = stdout_clone.lock().unwrap();

                // Read Content-Length header
                let mut header_line = String::new();
                stdout
                    .read_line(&mut header_line)
                    .map_err(|e| format!("Failed to read header: {:?}", e))?;

                if header_line.is_empty() {
                    return Err("EOF reached while reading header".to_string());
                }

                let content_length: usize = header_line
                    .trim()
                    .strip_prefix("Content-Length: ")
                    .ok_or("Invalid header format")?
                    .parse()
                    .map_err(|e| format!("Failed to parse content length: {:?}", e))?;

                // Skip empty line
                let mut empty_line = String::new();
                stdout
                    .read_line(&mut empty_line)
                    .map_err(|e| format!("Failed to read empty line: {:?}", e))?;

                // Read content
                let mut content = vec![0u8; content_length];
                stdout
                    .read_exact(&mut content)
                    .map_err(|e| format!("Failed to read content: {:?}", e))?;

                let message: Value = serde_json::from_slice(&content)
                    .map_err(|e| format!("Failed to parse JSON: {:?}", e))?;

                Ok(message)
            })();

            let _ = tx.send(result);
        });

        // Use a short timeout for non-blocking behavior
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(message)) => Some(message),
            Ok(Err(_)) => None,
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(_) => None,
        }
    }

    /// Process a received message and update internal state
    fn process_message(&mut self, message: Value) {
        // Check if it's a response (has an id field)
        if let Some(id) = message.get("id") {
            if let Some(id_num) = id.as_u64() {
                self.responses.insert(id_num as u32, message.clone());
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
                                if let Some(diagnostics) = params.get("diagnostics") {
                                    self.diagnostics.insert(file_path, diagnostics.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add to message queue for test code to inspect
        self.message_queue.push_back(message);
    }

    /// Wait for server messages for the specified duration and buffer them
    pub fn wait_for_server(&mut self, duration: Duration) {
        let start_time = Instant::now();

        while start_time.elapsed() < duration {
            if let Some(message) = self.try_receive_one_message() {
                self.process_message(message);
            }
        }
    }

    /// Pop one message from the message queue
    #[allow(dead_code)]
    pub fn pop_message(&mut self) -> Option<Value> {
        self.message_queue.pop_front()
    }

    /// Get a response for a specific request ID
    /// Returns None if the response has not been received yet
    /// Removes the response from the internal map when retrieved
    pub fn get_response(&mut self, id: u32) -> Option<Value> {
        self.responses.remove(&id)
    }

    /// Get diagnostics for a specific file
    /// Returns a vector of diagnostic messages (empty if no diagnostics)
    #[allow(dead_code)]
    pub fn get_diagnostics(&self, file_path: &Path) -> Vec<Value> {
        let absolute_path = self.working_dir.join(file_path);
        if let Some(diagnostics) = self.diagnostics.get(&absolute_path) {
            if let Some(arr) = diagnostics.as_array() {
                return arr.clone();
            }
        }
        Vec::new()
    }

    /// Verify that there are no diagnostic errors for any file
    /// Returns an error if any diagnostics contain errors
    pub fn verify_no_diagnostic_errors(&self) -> Result<(), String> {
        for (file_path, diagnostics) in &self.diagnostics {
            if let Some(diag_array) = diagnostics.as_array() {
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
    pub fn initialize(&mut self, root_uri: &str, timeout: Duration) -> Result<(), String> {
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
    pub fn shutdown(&mut self) -> Result<(), String> {
        let id = self.send_request("shutdown", json!(null))?;

        // Wait for response with 5 second timeout
        let timeout = Duration::from_secs(5);
        self.wait_for_server(timeout);
        let _ = self.get_response(id);

        self.send_notification("exit", json!(null))?;

        // Wait for process to exit
        let _ = self.process.wait();

        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Kill process if still running
        let _ = self.process.kill();
    }
}
