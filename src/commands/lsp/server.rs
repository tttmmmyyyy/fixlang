use super::code_action;
use super::completion;
use super::document_symbol;
use super::goto_definition;
use super::hover;
use super::references;
use super::util::{get_current_dir, path_to_uri, span_to_location, span_to_range, uri_to_path};
use crate::ast::import::ImportStatement;
use crate::ast::program::{ModuleInfo, Program};
use crate::check_program::check_program_via_config;
use crate::configuration::BuildConfigType;
use crate::dependency_lockfile::LockFileType;
use crate::error::{any_to_string, Error, Errors};
use crate::misc::{to_absolute_path, Map, Set};
use crate::parser::{parse_str_import_statements, parse_str_module_defn};
use crate::project_file::ProjectFile;
use crate::typecheckcache::{self, SharedTypeCheckCache};
use crate::write_log;
use crate::Configuration;
use crate::DiagnosticsConfig;
use lsp_types::{
    CallHierarchyIncomingCallsParams, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    CallHierarchyServerCapability, CompletionItem,
    CompletionOptions, CompletionParams, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentSymbolParams,
    GotoDefinitionParams, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, PositionEncodingKind,
    ProgressParams, ProgressParamsValue, PublishDiagnosticsParams, ReferenceParams, SaveOptions,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Uri, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd, WorkDoneProgressOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::mem;
use std::path::Path;
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
};

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
    OnSaveFile(PathBuf),
    // Stop the diagnostics thread.
    Stop,
}

// The result of diagnostics.
pub struct DiagnosticsResult {
    pub program: Program,
}

// Document symbol request which are waiting for diagnostics.
pub struct PendingDocumentSymbolRequest {
    id: u32,
    params: DocumentSymbolParams,
}

// The latest content of each file (which may not have been saved to disk yet) and its associated information
pub struct LatestContent {
    // The path.
    path: PathBuf,
    // The latest content of the file.
    pub content: String,
    // Module name. None if not parsed yet or failed to parse.
    pub module_info: Option<ModuleInfo>,
    // Import statements. None if not parsed yet or failed to parse.
    pub import_stmts: Option<Vec<ImportStatement>>,
}

impl LatestContent {
    fn new(path: PathBuf, content: String) -> Self {
        LatestContent {
            path,
            content,
            module_info: None,
            import_stmts: None,
        }
    }

    pub(super) fn get_import_stmts(&mut self) -> &Option<Vec<ImportStatement>> {
        if self.import_stmts.is_none() {
            let import_stmts = parse_str_import_statements(self.path.clone(), &self.content);
            if let Ok(import_stmts) = import_stmts {
                self.import_stmts = Some(import_stmts);
            } else {
                self.import_stmts = None;
            }
        }
        &self.import_stmts
    }

    pub(super) fn get_module_info(&mut self) -> &Option<ModuleInfo> {
        if self.module_info.is_none() {
            let module_info = parse_str_module_defn(self.path.clone(), &self.content);
            if let Ok(module_info) = module_info {
                self.module_info = Some(module_info);
            } else {
                self.module_info = None;
            }
        }
        &self.module_info
    }
}

// Launch the language server
pub fn launch_language_server() {
    let mut stdin = std::io::stdin();

    // Prepare a channel to send requests to the diagnostics thread.
    let (diag_req_send, diag_req_recv) = mpsc::channel::<DiagnosticsMessage>();
    let mut diag_req_recv = Some(diag_req_recv);

    // Prepare a channel to response from the diagnostics thread.
    let (diag_res_send, diag_res_recv) = mpsc::channel::<DiagnosticsResult>();

    // The last diagnostics result.
    let mut last_diag: Option<DiagnosticsResult> = None;

    // Maps to get file contents from Uris.
    let mut uri_to_latest_content: Map<Uri, LatestContent> = Map::default();

    // The pending document symbol requests.
    let mut pending_document_symbol_requests: VecDeque<PendingDocumentSymbolRequest> =
        VecDeque::new();

    loop {
        // If new diagnostics are available, send store it to `last_diag`.
        while let Ok(res) = diag_res_recv.try_recv() {
            last_diag = Some(res);
        }
        if last_diag.is_some() {
            // If there are pending document symbol requests, process them.
            while let Some(req) = pending_document_symbol_requests.pop_front() {
                let program = &last_diag.as_ref().unwrap().program;
                document_symbol::handle_document_symbol(req.id, &req.params, program);
            }
        }

        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            let mut msg = "Failed to read a line: \n".to_string();
            msg.push_str(&format!("{:?}", res.unwrap_err()));
            write_log!("{}", msg);
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            let mut msg = "Expected `Content-Length:`. The line is: \n".to_string();
            msg.push_str(&format!("{:?}", content_length));
            write_log!("{}", msg);
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: Result<usize, _> = content_length
            .split_off("Content-Length:".len())
            .trim()
            .parse();
        if content_length.is_err() {
            let mut msg = "Failed to parse the content length: \n".to_string();
            msg.push_str(&format!("{:?}", content_length.err().unwrap()));
            write_log!("{}", msg);
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
                msg.push_str(&format!("{:?}", e));
                write_log!("{}", msg);
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
            msg.push_str(&format!("{:?}", res.unwrap_err()));
            write_log!("{}", msg);
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log!("Failed to parse the message as utf-8 string: ");
            write_log!("{:?}", message.unwrap_err());
            continue;
        }
        let message = message.unwrap();

        // Parse the message as JSONRPCMessage.
        let message: Result<JSONRPCMessage, _> = serde_json::from_str(&message);
        if message.is_err() {
            write_log!("Failed to parse the message as JSONRPCMessage: ");
            write_log!("{:?}", message.err().unwrap());
            continue;
        }
        let message = message.unwrap();
        write_log!(
            "Received message: {:?}",
            serde_json::to_string(&message).unwrap()
        );

        // Depending on the method, handle the message.
        if let Some(method) = message.method.as_ref() {
            write_log!("Handling method: {}", method);
            if method == "initialize" {
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<InitializeParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                handle_initialize(id.unwrap(), &params.unwrap());
            } else if method == "initialized" {
                let params: Option<InitializedParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                if diag_req_recv.is_none() {
                    let msg = "\"initialized\" method is sent twice.".to_string();
                    write_log!("{}", msg);
                    continue;
                }
                handle_initialized(
                    &params.unwrap(),
                    diag_req_send.clone(),
                    diag_req_recv.take().unwrap(),
                    diag_res_send.clone(),
                );
            } else if method == "shutdown" {
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                handle_shutdown(id.unwrap(), diag_req_send.clone());
            } else if method == "exit" {
                write_log!("Exiting the language server.");
                break;
            } else if method == "textDocument/didOpen" {
                let params: Option<DidOpenTextDocumentParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_open(&params.unwrap(), &mut uri_to_latest_content);
            } else if method == "textDocument/didChange" {
                let params: Option<DidChangeTextDocumentParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_change(&params.unwrap(), &mut uri_to_latest_content);
            } else if method == "textDocument/didSave" {
                let params: Option<DidSaveTextDocumentParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                handle_textdocument_did_save(
                    diag_req_send.clone(),
                    &params.unwrap(),
                    &mut uri_to_latest_content,
                );
            } else if method == "textDocument/completion" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<CompletionParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                completion::handle_completion(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                );
            } else if method == "completionItem/resolve" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<CompletionItem> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                completion::handle_completion_resolve_document(
                    id.unwrap(),
                    &params.unwrap(),
                    &mut uri_to_latest_content,
                    program,
                );
            } else if method == "textDocument/hover" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<HoverParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                hover::handle_hover(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                );
            } else if method == "textDocument/definition" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<GotoDefinitionParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                goto_definition::handle_goto_definition(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                );
            } else if method == "textDocument/documentSymbol" {
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<DocumentSymbolParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                if last_diag.is_none() {
                    pending_document_symbol_requests.push_back(PendingDocumentSymbolRequest {
                        id: id.unwrap(),
                        params: params.unwrap(),
                    });
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                document_symbol::handle_document_symbol(id.unwrap(), &params.unwrap(), program);
            } else if method == "textDocument/codeAction" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<lsp_types::CodeActionParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                code_action::handle_code_action(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &mut uri_to_latest_content,
                );
            } else if method == "textDocument/references" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<ReferenceParams> = parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                references::handle_references(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                );
            } else if method == "textDocument/prepareCallHierarchy" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<CallHierarchyPrepareParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                references::handle_call_hierarchy_prepare(
                    id.unwrap(),
                    &params.unwrap(),
                    program,
                    &uri_to_latest_content,
                );
            } else if method == "callHierarchy/incomingCalls" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<CallHierarchyIncomingCallsParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                references::handle_call_hierarchy_incoming(id.unwrap(), &params.unwrap(), program);
            } else if method == "callHierarchy/outgoingCalls" {
                if last_diag.is_none() {
                    continue;
                }
                let program = &last_diag.as_ref().unwrap().program;
                let id = parse_id(&message, method);
                if id.is_none() {
                    continue;
                }
                let params: Option<CallHierarchyOutgoingCallsParams> =
                    parase_params(message.params.unwrap());
                if params.is_none() {
                    continue;
                }
                references::handle_call_hierarchy_outgoing(id.unwrap(), &params.unwrap(), program);
            }
        }
    }
}

fn parase_params<T: DeserializeOwned>(params: Value) -> Option<T> {
    let params: Result<T, _> = serde_json::from_value(params);
    if params.is_err() {
        let mut msg = "Failed to parse the params: \n".to_string();
        msg.push_str(&format!("{:?}", params.err().unwrap()));
        write_log!("{}", msg);
        return None;
    }
    params.ok()
}

fn parse_id(message: &JSONRPCMessage, method: &str) -> Option<u32> {
    if message.id.is_none() {
        write_log!(
            "Failed to get \"id\" from the message for method \"{}\".\n",
            method
        );
        return None;
    }
    message.id
}

#[allow(dead_code)]
fn send_request<T: Serialize>(id: u32, method: String, params: Option<T>) {
    let msg = JSONRPCMessage::new(
        Some(id),
        Some(method),
        params.map(|params| serde_json::to_value(params).unwrap()),
        None,
        None,
    );
    send_message(&msg);
}

pub(super) fn send_response<T: Serialize, E: Serialize>(id: u32, result: Result<T, E>) {
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
    write_log!("Sending message: {}", msg);
    print!("Content-Length: {}\r\n\r\n{}", content_length, msg);
    std::io::stdout()
        .flush()
        .expect("Failed to flush the stdout.");
}

// Handle "initialize" method.
fn handle_initialize(id: u32, _params: &InitializeParams) {
    // Return server capabilities.
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: Some(PositionEncodingKind::UTF16),
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                        include_text: Some(true),
                    })),
                },
            )),
            notebook_document_sync: None,
            selection_range_provider: None,
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec![
                    " ".to_string(),
                    ".".to_string(),
                    "(".to_string(),
                    ":".to_string(),
                ]),
                all_commit_characters: None,
                resolve_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                completion_item: None,
            }),
            signature_help_provider: None,
            definition_provider: Some(lsp_types::OneOf::Left(true)),
            type_definition_provider: None,
            implementation_provider: None,
            references_provider: Some(lsp_types::OneOf::Left(true)),
            document_highlight_provider: None,
            document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
            workspace_symbol_provider: None,
            code_action_provider: Some(lsp_types::CodeActionProviderCapability::Simple(true)),
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
            call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
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
) {
    // Launch the diagnostics thread.
    std::thread::spawn(|| {
        let res = std::panic::catch_unwind(|| {
            diagnostics_thread(diag_req_recv, diag_res_send);
        });
        if res.is_err() {
            // If a panic occurs in the diagnostics thread,
            send_diagnostics_error_message(
                "Diagnostics stopped. This may be a bug of \"fix\" command. I would be happy if you report how to reproduce this at https://github.com/tttmmmyyyy/fixlang".to_string(),
            );
            let mut msg = "Panic occurred in the diagnostics thread: \n".to_string();
            msg.push_str(&format!("{}", any_to_string(res.err().as_ref().unwrap())));
            write_log!("{}", msg);
        }
    });

    // Send `Start` message to the diagnostics thread.
    if let Err(e) = diag_req_send.send(DiagnosticsMessage::Start) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log!("{}", msg);
    }
}

// Handle "shutdown" method.
fn handle_shutdown(id: u32, diag_send: Sender<DiagnosticsMessage>) {
    // Shutdown the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::Stop) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log!("{}", msg);
    }

    // Respond to the client.
    let param = Ok::<_, ()>(serde_json::to_value(None::<()>).unwrap());
    send_response(id, param);
}

// Handle "textDocument/didOpen" method.
fn handle_textdocument_did_open(
    params: &DidOpenTextDocumentParams,
    uri_to_latest_content: &mut Map<lsp_types::Uri, LatestContent>,
) {
    // Store the content of the file into the maps.
    let path = uri_to_path(&params.text_document.uri);
    uri_to_latest_content.insert(
        params.text_document.uri.clone(),
        LatestContent::new(path, params.text_document.text.clone()),
    );
}

// Handle "textDocument/didChange" method.
fn handle_textdocument_did_change(
    params: &DidChangeTextDocumentParams,
    uri_to_latest_content: &mut Map<lsp_types::Uri, LatestContent>,
) {
    // Store the content of the file into `uri_to_content`.
    if let Some(change) = params.content_changes.last() {
        let path = uri_to_path(&params.text_document.uri);
        uri_to_latest_content.insert(
            params.text_document.uri.clone(),
            LatestContent::new(path, change.text.clone()),
        );
    }
}

// Handle "textDocument/didSave" method.
fn handle_textdocument_did_save(
    diag_send: Sender<DiagnosticsMessage>,
    params: &DidSaveTextDocumentParams,
    uri_to_latest_content: &mut Map<lsp_types::Uri, LatestContent>,
) {
    // Store the content of the file into maps.
    if let Some(text) = &params.text {
        let path = uri_to_path(&params.text_document.uri);
        uri_to_latest_content.insert(
            params.text_document.uri.clone(),
            LatestContent::new(path, text.clone()),
        );
    } else {
        let msg = "No text data in \"textDocument/didSave\" notification.".to_string();
        write_log!("{}", msg);
    }

    // Get the path of the saved file.
    let path = uri_to_path(&params.text_document.uri);

    // Send a message to the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::OnSaveFile(path)) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log!("{}", msg);
    }
}




// The entry point of the diagnostics thread.
fn diagnostics_thread(req_recv: Receiver<DiagnosticsMessage>, res_send: Sender<DiagnosticsResult>) {
    let mut prev_err_paths = Set::default();
    let typecheck_cache = Arc::new(typecheckcache::MemoryCache::new());

    loop {
        // Wait for a message.
        let msg = req_recv.recv();
        if msg.is_err() {
            // If the sender is dropped, stop the diagnostics thread.
            break;
        }

        // Create and begin work done progress.
        const WORK_DONE_PROGRESS_TOKEN: &str = "diagnostics";
        send_work_done_progress_create(WORK_DONE_PROGRESS_TOKEN, 0);

        // Run diagnostics.
        let res = match msg.unwrap() {
            DiagnosticsMessage::Stop => {
                // Stop the diagnostics thread.
                break;
            }
            DiagnosticsMessage::OnSaveFile(_path) => {
                // TODO: we should run diagnostics only for the saved file and its dependents.
                // To achieve it, we need to write a mechanism to update diagnostics result (i.e. the Program and the Errors) incrementally.
                send_work_done_progress_begin(WORK_DONE_PROGRESS_TOKEN, "Running diagnostics");
                run_diagnostics(typecheck_cache.clone())
            }
            DiagnosticsMessage::Start => {
                send_work_done_progress_begin(WORK_DONE_PROGRESS_TOKEN, "Running diagnostics");
                run_diagnostics(typecheck_cache.clone())
            }
        };

        // End work done progress.
        send_work_done_progress_end(WORK_DONE_PROGRESS_TOKEN);

        // Send the result to the main thread and language clinent.
        let errs = match res {
            Ok(mut res) => {
                let errs = mem::replace(&mut res.program.deferred_errors, Errors::empty());
                res_send.send(res).unwrap();
                errs
            }
            Err(errs) => errs,
        };
        prev_err_paths = send_diagnostics_notification(errs, std::mem::take(&mut prev_err_paths));
    }
}

// Send the diagnostics notification to the client.
// Return the paths of the files that have errors.
// - `prev_err_paths`: The paths of the files that have errors in the previous diagnostics. This is used to clear the diagnostics for the files that have no errors.
fn send_diagnostics_notification(errs: Errors, mut prev_err_paths: Set<PathBuf>) -> Set<PathBuf> {
    let mut err_paths = Set::default();

    let Some(cdir) = get_current_dir() else {
        return err_paths;
    };

    // Send the diagnostics notification for each file that has errors.
    for (path, errs) in errs.organize_by_path() {
        err_paths.insert(path.clone());
        prev_err_paths.remove(&path);

        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(path));
        if uri.is_err() {
            write_log!("Failed to convert path to uri: {:?}", uri.unwrap_err());
            continue;
        }
        let uri = uri.unwrap();

        // Send the diagnostics notification for each file.
        let params = PublishDiagnosticsParams {
            uri,
            diagnostics: errs
                .iter()
                .map(|err| error_to_diagnostics(err, &cdir))
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
            write_log!("{}", (uri.unwrap_err()));
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
fn send_diagnostics_error_message(msg: String) {
    let Some(cdir) = get_current_dir() else {
        return;
    };
    // Convert path to uri.
    let cdir_uri = path_to_uri(&cdir);
    if cdir_uri.is_err() {
        write_log!("Failed to convert path to uri: {:?}", cdir_uri.unwrap_err());
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
fn error_to_diagnostics(err: &Error, cdir: &PathBuf) -> lsp_types::Diagnostic {
    // Show error at the first span in `err`.
    let range = err
        .srcs
        .first()
        .map(|(_, span)| span_to_range(span))
        .unwrap_or_default();

    // Other spans are shown in related informations.
    let mut related_information = vec![];
    for (msg, span) in err.srcs.iter().skip(1) {
        // Convert span to location.
        let location = span_to_location(span, cdir);
        if location.is_none() {
            continue;
        }
        let location = location.unwrap();

        // Create related informations.
        let related = lsp_types::DiagnosticRelatedInformation {
            location,
            message: if msg.len() > 0 {
                msg.to_string()
            } else {
                "See also here.".to_string()
            },
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
        code: err
            .code
            .map(|c| lsp_types::NumberOrString::String(c.to_string())),
        code_description: None,
        source: None,
        message: err.msg.clone(),
        tags: None,
        related_information,
        data: err.data.clone(),
    }
}

// Get the file content at the specified path at the time of the last diagnostics.
//
// - `program`: The `Program` obtained from the last diagnostics result
pub(super) fn get_file_content_at_previous_diagnostics(
    program: &Program,
    path: &Path,
) -> Result<String, String> {
    for mi in &program.modules {
        let src = &mi.source.input;
        let path_abs = to_absolute_path(&path);
        if path_abs.is_err() {
            let msg = format!(
                "Failed to get the absolute path of the file: \"{}\"",
                path.to_string_lossy().to_string()
            );
            return Err(msg);
        }
        let path = path_abs.ok().unwrap();
        let src_file_path_abs = to_absolute_path(&src.file_path);
        if src_file_path_abs.is_err() {
            let msg = format!(
                "Failed to get the absolute path of the source file: \"{}\"",
                src.file_path.to_string_lossy().to_string()
            );
            return Err(msg);
        }
        let src_file_path = src_file_path_abs.ok().unwrap();
        if src_file_path == path {
            let content = src.string();
            if let Err(_e) = content {
                let msg = format!(
                    "Failed to get the content of the file: \"{}\"",
                    src.file_path.to_string_lossy().to_string()
                );
                return Err(msg);
            }
            return Ok(content.ok().unwrap());
        }
    }
    let msg = format!(
        "No saved content for the file: \"{}\"\n",
        path.to_string_lossy().to_string()
    );
    return Err(msg);
}

pub fn run_diagnostics(typecheck_cache: SharedTypeCheckCache) -> Result<DiagnosticsResult, Errors> {
    // TODO: maybe we should check if the file has been changed actually after previous diagnostics?

    // Read the project file.
    let proj_file = ProjectFile::read_root_file()?;

    // Determine the source files for which diagnostics are run.
    let files = proj_file.get_files(BuildConfigType::Test);

    // Create the configuration.
    let mut config = Configuration::diagnostics_mode(DiagnosticsConfig { files })?;
    config.type_check_cache = typecheck_cache;

    // Set up the configuration by the project file.
    proj_file.set_config(&mut config, false)?;

    // Set up the configuration by the lock file.
    // Automatically create/update the lock file if necessary.
    proj_file
        .open_or_auto_update_lock_file(LockFileType::Lsp)?
        .set_config(&mut config)?;

    // Build the file and get the errors.
    let program = check_program_via_config(&config)?;

    Ok(DiagnosticsResult { program })
}

// Create work done progress.
pub fn send_work_done_progress_create(token: &str, id: u32) {
    let progress = WorkDoneProgressCreateParams {
        token: lsp_types::ProgressToken::String(token.to_string()),
    };
    send_request(
        id,
        "window/workDoneProgress/create".to_string(),
        Some(progress),
    );
}

// Begin work done progress.
pub fn send_work_done_progress_begin(token: &str, title: &str) {
    let begin = WorkDoneProgressBegin {
        title: title.to_string(),
        cancellable: Some(false),
        message: None,
        percentage: None,
    };
    let params = ProgressParams {
        token: lsp_types::ProgressToken::String(token.to_string()),
        value: ProgressParamsValue::WorkDone(lsp_types::WorkDoneProgress::Begin(begin)),
    };
    send_notification("$/progress".to_string(), Some(params));
}

// End work done progress.
pub fn send_work_done_progress_end(token: &str) {
    let end = WorkDoneProgressEnd { message: None };
    let params = ProgressParams {
        token: lsp_types::ProgressToken::String(token.to_string()),
        value: ProgressParamsValue::WorkDone(lsp_types::WorkDoneProgress::End(end)),
    };
    send_notification("$/progress".to_string(), Some(params));
}


