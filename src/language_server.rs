use lsp_types::{
    InitializeParams, InitializeResult, InitializedParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{constants::LSP_LOG_FILE_PATH, Configuration};
use std::{
    fs::File,
    io::{Read, Write},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

pub const WRITE_LOG: bool = true;

#[derive(Deserialize, Serialize)]
pub struct JSONRPCMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

impl JSONRPCMessage {
    pub fn new(
        id: Option<u32>,
        method: Option<String>,
        params: Option<Value>,
        result: Option<Value>,
    ) -> Self {
        JSONRPCMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
            result,
        }
    }
}

// Messaages sent for diagnostic thread.
enum DiagnosticsMessage {
    // A file is saved.
    OnSaveFile,
    // Stop the diagnostics thread.
    Stop,
}

// Launch the language server
pub fn launch_language_server() {
    let mut stdin = std::io::stdin();

    // Prepare the log file.
    let log_file = open_log_file();
    write_log(log_file.clone(), "Language server started.\n");

    // Prepare a channel to send messages from the main thread to the diagnostics thread.
    let (diag_send, diag_recv) = mpsc::channel::<DiagnosticsMessage>();
    let mut diag_recv = Some(diag_recv);

    loop {
        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            let mut msg = "Failed to read a line: \n".to_string();
            msg.push_str(&format!("{:?}\n", res.unwrap_err()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            let mut msg = "Expected `Content-Length:`. The line is: \n".to_string();
            msg.push_str(&format!("{:?}\n", content_length));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: Result<usize, _> = content_length
            .split_off("Content-Length:".len())
            .trim()
            .parse();
        if content_length.is_err() {
            let mut msg = "Failed to parse the content length: \n".to_string();
            msg.push_str(&format!("{:?}\n", content_length.err().unwrap()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        let content_length = content_length.unwrap();

        // Read stdin upto an empty line.
        loop {
            let mut line = String::new();
            let res = stdin.read_line(&mut line);
            if res.is_err() {
                let e = res.unwrap_err();
                let mut msg = "Failed to read a line: \n".to_string();
                msg.push_str(&format!("{:?}\n", e));
                write_log(log_file.clone(), msg.as_str());
                continue;
            }
            if line.trim().is_empty() {
                break;
            }
        }

        // Read the content of the message.
        let mut message = vec![0; content_length];
        let res = stdin.read_exact(&mut message);
        if res.is_err() {
            let mut msg = "Failed to read the message: \n".to_string();
            msg.push_str(&format!("{:?}\n", res.unwrap_err()));
            write_log(log_file.clone(), msg.as_str());
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log(
                log_file.clone(),
                "Failed to parse the message as utf-8 string: \n",
            );
            write_log(log_file.clone(), &format!("{:?}\n", message.unwrap_err()));
            continue;
        }
        let message = message.unwrap();

        // Parse the message as JSONRPCMessage.
        let message: Result<JSONRPCMessage, _> = serde_json::from_str(&message);
        if message.is_err() {
            write_log(
                log_file.clone(),
                "Failed to parse the message as JSONRPCMessage: \n",
            );
            write_log(log_file.clone(), &format!("{:?}\n", message.err().unwrap()));
            continue;
        }
        let message = message.unwrap();

        // Depending on the method, handle the message.
        if let Some(method) = message.method.as_ref() {
            if method == "initialize" {
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<InitializeParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_initialize(id.unwrap(), &params.unwrap(), log_file.clone());
            } else if method == "initialized" {
                let params: Option<InitializedParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                if diag_recv.is_none() {
                    let msg = "\"initialized\" method is sent twice.\n".to_string();
                    write_log(log_file.clone(), msg.as_str());
                    continue;
                }
                handle_initialized(
                    &params.unwrap(),
                    diag_recv.take().unwrap(),
                    log_file.clone(),
                );
            } else if method == "shutdown" {
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                handle_shutdown(id.unwrap(), diag_send.clone(), log_file.clone());
            } else if method == "exit" {
                write_log(log_file.clone(), "Exiting the language server.\n");
                break;
            } else if method == "textDocument/didSave" {
                handle_textdocument_did_save(diag_send.clone(), log_file.clone());
            }
        }
    }
}

fn parase_params<T: DeserializeOwned>(params: Value, log_file: Arc<Mutex<File>>) -> Option<T> {
    let params: Result<T, _> = serde_json::from_value(params);
    if params.is_err() {
        let mut msg = "Failed to parse the params: \n".to_string();
        msg.push_str(&format!("{:?}\n", params.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return None;
    }
    params.ok()
}

fn parse_id(message: &JSONRPCMessage, method: &str, log_file: Arc<Mutex<File>>) -> Option<u32> {
    if message.id.is_none() {
        write_log(
            log_file,
            format!(
                "Failed to get \"id\" from the message for method \"{}\".\n",
                method
            )
            .as_str(),
        );
        return None;
    }
    message.id
}

fn send_response<T: Serialize>(id: u32, result: Option<&T>) {
    let msg = JSONRPCMessage::new(
        Some(id),
        None,
        None,
        result.map(|result| serde_json::to_value(result).unwrap()),
    );
    send_message(&msg);
}

fn send_notification<T: Serialize>(method: String, params: Option<&T>) {
    let msg = JSONRPCMessage::new(
        None,
        Some(method),
        params.map(|params| serde_json::to_value(params).unwrap()),
        None,
    );
    send_message(&msg);
}

fn send_message(msg: &JSONRPCMessage) {
    let msg = serde_json::to_string(msg).unwrap();
    let content_length = msg.len();
    println!("Content-Length: {}\r\n\r\n{}", content_length, msg);
    std::io::stdout()
        .flush()
        .expect("Failed to flush the stdout.");
}

fn open_log_file() -> Arc<Mutex<File>> {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LSP_LOG_FILE_PATH)
        .parent()
        .expect("Failed to get parent directory of LSP_LOG_FILE_PATH.");

    // Create directories to the parent directory.
    std::fs::create_dir_all(parent_dir)
        .expect("Failed to create directories to the parent directory.");

    // Create and open the log file.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LSP_LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LSP_LOG_FILE_PATH).as_str());

    // Wrap it into a mutex.
    Arc::new(Mutex::new(file))
}

fn write_log(file: Arc<Mutex<File>>, message: &str) {
    let mut file = file.lock().expect("Failed to lock the log file.");
    if WRITE_LOG {
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

// Handle "initialize" method.
fn handle_initialize(id: u32, _params: &InitializeParams, _log_file: Arc<Mutex<File>>) {
    // Return empty capabilities.
    // - The server can respond to DidSaveTextDocument notification.
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: None,
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: None,
                    change: None,
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                },
            )),
            notebook_document_sync: None,
            selection_range_provider: None,
            hover_provider: None,
            completion_provider: None,
            signature_help_provider: None,
            definition_provider: None,
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: None,
            document_highlight_provider: None,
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: None,
            code_lens_provider: None,
            document_formatting_provider: None,
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: None,
            document_link_provider: None,
            color_provider: None,
            folding_range_provider: None,
            declaration_provider: None,
            execute_command_provider: None,
            workspace: None,
            call_hierarchy_provider: None,
            semantic_tokens_provider: None,
            moniker_provider: None,
            linked_editing_range_provider: None,
            inline_value_provider: None,
            inlay_hint_provider: None,
            diagnostic_provider: None,
            experimental: None,
        },
        server_info: None,
    };
    send_response(id, Some(&result))
}

// Handle "initialized" method.
fn handle_initialized(
    _params: &InitializedParams,
    diag_recv: Receiver<DiagnosticsMessage>,
    log_file: Arc<Mutex<File>>,
) {
    // Launch the diagnostics thread.
    // TODO: launch the diagnostics thread only when `publishDiagnostics` is supported in client.
    std::thread::spawn(|| {
        diagnostics_thread(diag_recv, log_file);
    });
}

// Handle "shutdown" method.
fn handle_shutdown(id: u32, diag_send: Sender<DiagnosticsMessage>, _log_file: Arc<Mutex<File>>) {
    // Shutdown the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::Stop) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(_log_file.clone(), msg.as_str());
    }

    // Respond to the client.
    let param: Option<&()> = None;
    send_response(id, param);
}

// Handle "textDocument/didSave" method.
fn handle_textdocument_did_save(diag_send: Sender<DiagnosticsMessage>, log_file: Arc<Mutex<File>>) {
    // Send a message to the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::OnSaveFile) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(log_file.clone(), msg.as_str());
    }
}

// The entry point of the diagnostics thread.
fn diagnostics_thread(msg_recv: Receiver<DiagnosticsMessage>, log_file: Arc<Mutex<File>>) {
    loop {
        let msg = msg_recv.recv();
        if msg.is_err() {
            // If the sender is dropped, stop the diagnostics thread.
            break;
        }
        match msg.unwrap() {
            DiagnosticsMessage::Stop => {
                // Stop the diagnostics thread.
                break;
            }
            DiagnosticsMessage::OnSaveFile => {
                run_diagnostics(log_file.clone());
            }
        }
    }
}

fn run_diagnostics(log_file: Arc<Mutex<File>>) {
    // TODO: check if the file has been changed actually after previous diagnostics.
}
