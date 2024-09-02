use lsp_types::{InitializeParams, InitializeResult, InitializedParams};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::{constants::LSP_LOG_FILE_PATH, Configuration};
use std::{
    fs::File,
    io::{Read, Write},
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

// Launch the language server
pub fn launch_language_server() {
    let mut log_file = open_log_file();
    write_log(&mut log_file, "Language server started.\n");

    let mut stdin = std::io::stdin();

    loop {
        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            write_log(&mut log_file, "Failed to read the content length: \n");
            write_log(&mut log_file, &format!("{:?}\n", res.unwrap_err()));
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            write_log(&mut log_file, "Expected `Content-Length:`. The line is: \n");
            write_log(&mut log_file, &format!("{:?}\n", content_length));
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: Result<usize, _> = content_length
            .split_off("Content-Length:".len())
            .trim()
            .parse();
        if content_length.is_err() {
            write_log(&mut log_file, "Failed to parse the content length: \n");
            write_log(
                &mut log_file,
                &format!("{:?}\n", content_length.unwrap_err()),
            );
            continue;
        }
        let content_length = content_length.unwrap();

        // Read stdin upto an empty line.
        loop {
            let mut line = String::new();
            let res = stdin.read_line(&mut line);
            if res.is_err() {
                write_log(&mut log_file, "Failed to read a line: \n");
                let e = res.unwrap_err();
                write_log(&mut log_file, &format!("{:?}\n", e));
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
            write_log(&mut log_file, "Failed to read the message: \n");
            write_log(&mut log_file, &format!("{:?}\n", res.unwrap_err()));
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log(
                &mut log_file,
                "Failed to parse the message as utf-8 string: \n",
            );
            write_log(&mut log_file, &format!("{:?}\n", message.unwrap_err()));
            continue;
        }
        let message = message.unwrap();

        // Parse the message as JSONRPCMessage.
        let message: Result<JSONRPCMessage, _> = serde_json::from_str(&message);
        if message.is_err() {
            write_log(
                &mut log_file,
                "Failed to parse the message as JSONRPCMessage: \n",
            );
            write_log(&mut log_file, &format!("{:?}\n", message.err().unwrap()));
            continue;
        }
        let message = message.unwrap();

        // Depending on the method, handle the message.
        if let Some(method) = message.method.as_ref() {
            let id = message.id.unwrap();
            if method == "initialize" {
                let params: Option<InitializeParams> =
                    parase_params(message.params.unwrap(), &mut log_file);
                if params.is_none() {
                    continue;
                }
                handle_initialize(id, &params.unwrap(), &mut log_file);
            } else if method == "initialized" {
                let params: Option<InitializedParams> =
                    parase_params(message.params.unwrap(), &mut log_file);
                if params.is_none() {
                    continue;
                }
                handle_initialized(&params.unwrap(), &mut log_file);
            } else if method == "shutdown" {
                handle_shutdown(id, &mut log_file);
            } else if method == "exit" {
                break;
            }
        }
    }
}

fn parase_params<T: DeserializeOwned>(params: Value, log_file: &mut File) -> Option<T> {
    let params: Result<T, _> = serde_json::from_value(params);
    if params.is_err() {
        write_log(log_file, "Failed to parse the params: \n");
        write_log(log_file, &format!("{:?}\n", params.err().unwrap()));
        return None;
    }
    params.ok()
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

fn open_log_file() -> File {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LSP_LOG_FILE_PATH)
        .parent()
        .expect("Failed to get parent directory of LSP_LOG_FILE_PATH.");

    // Create directories to the parent directory.
    std::fs::create_dir_all(parent_dir)
        .expect("Failed to create directories to the parent directory.");

    // Create and open the log file.
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LSP_LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LSP_LOG_FILE_PATH).as_str())
}

fn write_log(file: &mut File, message: &str) {
    if WRITE_LOG {
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

// Handle "initialize" method.
fn handle_initialize(id: u32, _params: &InitializeParams, _log_file: &mut File) {
    // Return empty capabilities.
    let result = InitializeResult {
        capabilities: Default::default(),
        server_info: None,
    };
    send_response(id, Some(&result))
}

// Handle "initialized" method.
fn handle_initialized(_params: &InitializedParams, _log_file: &mut File) {
    // TODO: Launch the diagnostics thread.
}

// Handle "shutdown" method.
fn handle_shutdown(id: u32, _log_file: &mut File) {
    // TODO: Shutdown the diagnostics thread.
    let param: Option<&()> = None;
    send_response(id, param);
}
