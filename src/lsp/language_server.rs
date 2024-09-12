use crate::ast::program::Program;
use crate::constants::INSTANCIATED_NAME_SEPARATOR;
use crate::{
    constants::LSP_LOG_FILE_PATH,
    error::{any_to_string, Error, Errors},
    project::ProjectFile,
    runner::build_file,
    Configuration, Span,
};
use lsp_types::{
    CompletionItem, CompletionItemLabelDetails, CompletionOptions, CompletionParams,
    DiagnosticSeverity, InitializeParams, InitializeResult, InitializedParams,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri, WorkDoneProgressOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl JSONRPCMessage {
    pub fn new(
        id: Option<u32>,
        method: Option<String>,
        params: Option<Value>,
        result: Option<Value>,
        error: Option<Value>,
    ) -> Self {
        JSONRPCMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method,
            params,
            result,
            error,
        }
    }
}

// Requests sent to diagnostic thread.
enum DiagnosticsMessage {
    // Started the diagnostics thread.
    Start,
    // A file is saved.
    OnSaveFile,
    // Stop the diagnostics thread.
    Stop,
}

// The result of diagnostics.
pub struct DiagnosticsResult {
    pub prgoram: Program,
}

// Launch the language server
pub fn launch_language_server() {
    let mut stdin = std::io::stdin();

    // Prepare the log file.
    let log_file = open_log_file();
    write_log(log_file.clone(), "Language server started.\n");

    // Prepare a channel to send requests to the diagnostics thread.
    let (diag_req_send, diag_req_recv) = mpsc::channel::<DiagnosticsMessage>();
    let mut diag_req_recv = Some(diag_req_recv);

    // Prepare a channel to response from the diagnostics thread.
    let (diag_res_send, diag_res_recv) = mpsc::channel::<DiagnosticsResult>();

    // The last diagnostics result.
    let mut last_diag: Option<DiagnosticsResult> = None;

    loop {
        // If new diagnostics are available, send store it to `last_diag`.
        if let Ok(res) = diag_res_recv.try_recv() {
            last_diag = Some(res);
        }

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
                if diag_req_recv.is_none() {
                    let msg = "\"initialized\" method is sent twice.\n".to_string();
                    write_log(log_file.clone(), msg.as_str());
                    continue;
                }
                handle_initialized(
                    &params.unwrap(),
                    diag_req_send.clone(),
                    diag_req_recv.take().unwrap(),
                    diag_res_send.clone(),
                    log_file.clone(),
                );
            } else if method == "shutdown" {
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                handle_shutdown(id.unwrap(), diag_req_send.clone(), log_file.clone());
            } else if method == "exit" {
                write_log(log_file.clone(), "Exiting the language server.\n");
                break;
            } else if method == "textDocument/didSave" {
                handle_textdocument_did_save(diag_req_send.clone(), log_file.clone());
            } else if method == "textDocument/completion" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().prgoram;
                let id = parse_id(&message, method, log_file.clone());
                if id.is_none() {
                    continue;
                }
                let params: Option<CompletionParams> =
                    parase_params(message.params.unwrap(), log_file.clone());
                if params.is_none() {
                    continue;
                }
                handle_completion(id.unwrap(), &params.unwrap(), program, log_file.clone());
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

fn send_response<T: Serialize, E: Serialize>(id: u32, result: Result<T, E>) {
    let (res, err) = match result {
        Ok(res) => (Some(res), None),
        Err(err) => (None, Some(err)),
    };
    let msg = JSONRPCMessage::new(
        Some(id),
        None,
        None,
        res.map(|res| serde_json::to_value(res).unwrap()),
        err.map(|err| serde_json::to_value(err).unwrap()),
    );
    send_message(&msg);
}

fn send_notification<T: Serialize>(method: String, params: Option<T>) {
    let msg = JSONRPCMessage::new(
        None,
        Some(method),
        params.map(|params| serde_json::to_value(params).unwrap()),
        None,
        None,
    );
    send_message(&msg);
}

fn send_message(msg: &JSONRPCMessage) {
    let msg = serde_json::to_string(msg).unwrap();
    let content_length = msg.len();
    print!("Content-Length: {}\r\n\r\n{}", content_length, msg);
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
    // Return server capabilities.
    // - DidSaveTextDocument
    // - textDocument/completion
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
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![" ".to_string(), ".".to_string(), "(".to_string()]),
                all_commit_characters: None,
                resolve_provider: None,
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: None,
            }),
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
    send_response(id, Ok::<_, ()>(result))
}

// Handle "initialized" method.
fn handle_initialized(
    _params: &InitializedParams,
    diag_req_send: Sender<DiagnosticsMessage>,
    diag_req_recv: Receiver<DiagnosticsMessage>,
    diag_res_send: Sender<DiagnosticsResult>,
    log_file: Arc<Mutex<File>>,
) {
    // Launch the diagnostics thread.
    let log_file_cloned = log_file.clone();
    std::thread::spawn(|| {
        let res = std::panic::catch_unwind(|| {
            diagnostics_thread(diag_req_recv, diag_res_send, log_file_cloned.clone());
        });
        if res.is_err() {
            // If a panic occurs in the diagnostics thread,
            send_diagnostics_error_message(
                "Diagnostics stopped. This may be a bug of \"fix\" command. I would be happy if you report how to reproduce this! Repo: https://github.com/tttmmmyyyy/fixlang".to_string(),
                log_file_cloned.clone(),
            );
            let mut msg = "Panic occurred in the diagnostics thread: \n".to_string();
            msg.push_str(&format!("{}\n", any_to_string(res.err().as_ref().unwrap())));
            write_log(log_file_cloned, msg.as_str());
        }
    });

    // Send `Start` message to the diagnostics thread.
    if let Err(e) = diag_req_send.send(DiagnosticsMessage::Start) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}\n", e));
        write_log(log_file.clone(), msg.as_str());
    }
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
    let param = Ok::<_, ()>(serde_json::to_value(None::<()>).unwrap());
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

// Handle "textDocument/completion" method.
fn handle_completion(
    id: u32,
    _params: &CompletionParams,
    program: &Program,
    _log_file: Arc<Mutex<File>>,
) {
    let mut items = vec![];
    for (name, gv) in &program.global_values {
        let label = name.name.clone();
        // Skip compiler-defined values.
        if label.starts_with(INSTANCIATED_NAME_SEPARATOR) {
            continue;
        }
        let in_namespace = " in ".to_string() + &name.namespace.to_string();
        let scheme = gv.scm.to_string();
        items.push(CompletionItem {
            label,
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(in_namespace),
                description: None,
            }),
            kind: None,
            detail: Some(scheme),
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: None,
            tags: None,
        });
    }
    send_response(id, Ok::<_, ()>(items));
}

// The entry point of the diagnostics thread.
fn diagnostics_thread(
    req_recv: Receiver<DiagnosticsMessage>,
    res_send: Sender<DiagnosticsResult>,
    log_file: Arc<Mutex<File>>,
) {
    let mut prev_err_paths = HashSet::new();

    loop {
        // Wait for a message.
        let msg = req_recv.recv();
        if msg.is_err() {
            // If the sender is dropped, stop the diagnostics thread.
            break;
        }

        // Run diagnostics.
        let res = match msg.unwrap() {
            DiagnosticsMessage::Stop => {
                // Stop the diagnostics thread.
                break;
            }
            DiagnosticsMessage::OnSaveFile => run_diagnostics(log_file.clone()),
            DiagnosticsMessage::Start => run_diagnostics(log_file.clone()),
        };

        // Send the result to the main thread and language clinent.
        let errs = match res {
            Ok(res) => {
                res_send.send(res).unwrap();
                Errors::empty()
            }
            Err(errs) => errs,
        };
        prev_err_paths = send_diagnostics_notification(
            errs,
            std::mem::replace(&mut prev_err_paths, HashSet::new()),
            log_file.clone(),
        );
    }
}

// Convert a `Span` into a `Range`.
fn span_to_range(span: &Span) -> lsp_types::Range {
    fn pair_to_zero_indexed((x, y): (usize, usize)) -> (usize, usize) {
        (x - 1, y - 1)
    }

    let (start_line, start_column) = pair_to_zero_indexed(span.start_line_col());
    let (end_line, end_column) = pair_to_zero_indexed(span.end_line_col());
    lsp_types::Range {
        start: lsp_types::Position {
            line: start_line as u32,
            character: start_column as u32,
        },
        end: lsp_types::Position {
            line: end_line as u32,
            character: end_column as u32,
        },
    }
}

// Send the diagnostics notification to the client.
// Return the paths of the files that have errors.
// - `prev_err_paths`: The paths of the files that have errors in the previous diagnostics. This is used to clear the diagnostics for the files that have no errors.
fn send_diagnostics_notification(
    errs: Errors,
    mut prev_err_paths: HashSet<PathBuf>,
    log_file: Arc<Mutex<File>>,
) -> HashSet<PathBuf> {
    let mut err_paths = HashSet::new();

    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}\n", cdir.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return err_paths;
    }
    let cdir = cdir.unwrap();

    // Send the diagnostics notification for each file that has errors.
    for (path, errs) in errs.organize_by_path() {
        err_paths.insert(path.clone());
        prev_err_paths.remove(&path);

        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(path));
        if uri.is_err() {
            write_log(
                log_file.clone(),
                &format!("Failed to convert path to uri: {:?}\n", uri.unwrap_err()),
            );
            continue;
        }
        let uri = uri.unwrap();

        // Send the diagnostics notification for each file.
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics: errs
                .iter()
                .map(|err| error_to_diagnostics(err, &cdir, log_file.clone()))
                .collect(),
            version: None,
        };
        send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
    }

    // Clear the diagnostics for the files that have no errors.
    for path in prev_err_paths {
        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(path));
        if uri.is_err() {
            write_log(log_file.clone(), &(uri.unwrap_err() + "\n"));
            continue;
        }
        let uri = uri.unwrap();

        // Send the empty diagnostics notification for each file.
        let params = lsp_types::PublishDiagnosticsParams {
            uri,
            diagnostics: vec![],
            version: None,
        };
        send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
    }

    err_paths
}

// Send the diagnostics notification to the client which informs that an error occurred.
fn send_diagnostics_error_message(msg: String, log_file: Arc<Mutex<File>>) {
    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}\n", cdir.err().unwrap()));
        write_log(log_file.clone(), msg.as_str());
        return;
    }
    let cdir = cdir.unwrap();
    // Convert path to uri.
    let cdir_uri = path_to_uri(&cdir);
    if cdir_uri.is_err() {
        write_log(
            log_file.clone(),
            &format!(
                "Failed to convert path to uri: {:?}\n",
                cdir_uri.unwrap_err()
            ),
        );
        return;
    }
    let cdir_uri = cdir_uri.unwrap();

    // Send the diagnostics notification for each file.
    let params = lsp_types::PublishDiagnosticsParams {
        uri: cdir_uri,
        diagnostics: vec![lsp_types::Diagnostic {
            range: lsp_types::Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: None,
            message: msg,
            tags: None,
            related_information: None,
            data: None,
        }],
        version: None,
    };
    send_notification("textDocument/publishDiagnostics".to_string(), Some(&params));
}

// Convert an `Error` into a diagnostic message.
fn error_to_diagnostics(
    err: &Error,
    cdir: &PathBuf,
    log_file: Arc<Mutex<File>>,
) -> lsp_types::Diagnostic {
    // Show error at the first span in `err`.
    let range = err
        .srcs
        .first()
        .map(|span| span_to_range(span))
        .unwrap_or_default();

    // Other spans are shown in related informations.
    let mut related_information = vec![];
    for span in err.srcs.iter().skip(1) {
        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(&span.input.file_path));
        if uri.is_err() {
            write_log(
                log_file.clone(),
                &format!("Failed to convert path to uri: {:?}\n", uri.unwrap_err()),
            );
            continue;
        }
        let uri = uri.unwrap();

        // Create related informations.
        let related = lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri,
                range: span_to_range(span),
            },
            message: "see also here".to_string(),
        };
        related_information.push(related);
    }
    let related_information = if related_information.is_empty() {
        None
    } else {
        Some(related_information)
    };

    lsp_types::Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: None,
        message: err.msg.clone(),
        tags: None,
        related_information,
        data: None,
    }
}

fn path_to_uri(path: &PathBuf) -> Result<Uri, String> {
    let path = path.to_str();
    if path.is_none() {
        return Err(format!("Failed to convert a path into string: {:?}", path));
    }
    let path = "file://".to_string() + path.unwrap();
    let uri = Uri::from_str(&path);
    if uri.is_err() {
        return Err(format!("Failed to convert a path into Uri: {:?}", path));
    }
    Ok(uri.unwrap())
}

fn run_diagnostics(_log_file: Arc<Mutex<File>>) -> Result<DiagnosticsResult, Errors> {
    // TODO: maybe we should check if the file has been changed actually after previous diagnostics?

    // Read the project file.
    let project_file = ProjectFile::read_file(true)?;

    // Create the configuration.
    let mut config = Configuration::language_server();
    ProjectFile::set_config_from_proj_file(&mut config, &project_file);

    // Build the file and get the errors.
    let program = build_file(&mut config)?.program.unwrap();

    Ok(DiagnosticsResult { prgoram: program })
}
