use crate::ast::import::{is_accessible, ImportStatement};
use crate::ast::name::{FullName, NameSpace};
use crate::ast::program::{GlobalValue, ModuleInfo, Program};
use crate::ast::traits::{Trait, TraitAlias, TraitInfo, TraitInstance};
use crate::ast::types::{TyAliasInfo, TyCon, TyConInfo, TyConVariant};
use crate::constants::{chars_allowed_in_identifiers, STD_NAME};
use crate::docgen::MarkdownSection;
use crate::misc::{to_absolute_path, Map, Set};
use crate::parser::{parse_str_import_statements, parse_str_module_defn};
use crate::typecheckcache::{self, SharedTypeCheckCache};
use crate::{
    constants::LOG_FILE_PATH,
    error::{any_to_string, Error, Errors},
    project_file::ProjectFile,
    runner::build_file,
    Configuration, Span,
};
use crate::{DiagnosticsConfig, EndNode, SourceFile, SourcePos, Var};
use difference::diff;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    CompletionParams, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentSymbol, DocumentSymbolParams, Documentation,
    GotoDefinitionParams, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, MarkupContent, ProgressParams, ProgressParamsValue,
    PublishDiagnosticsParams, SaveOptions, ServerCapabilities, SymbolKind,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Uri, WorkDoneProgressBegin,
    WorkDoneProgressCreateParams, WorkDoneProgressEnd, WorkDoneProgressOptions,
};
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::mem;
use std::path::Path;
use std::{
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

static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| open_log_file());

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

    fn get_import_stmts(&mut self) -> &Option<Vec<ImportStatement>> {
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

    fn get_module_info(&mut self) -> &Option<ModuleInfo> {
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
                handle_document_symbol(req.id, &req.params, program);
            }
        }

        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            let mut msg = "Failed to read a line: \n".to_string();
            msg.push_str(&format!("{:?}", res.unwrap_err()));
            write_log(msg.as_str());
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            let mut msg = "Expected `Content-Length:`. The line is: \n".to_string();
            msg.push_str(&format!("{:?}", content_length));
            write_log(msg.as_str());
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
            write_log(msg.as_str());
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
                write_log(msg.as_str());
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
            write_log(msg.as_str());
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log("Failed to parse the message as utf-8 string: ");
            write_log(&format!("{:?}", message.unwrap_err()));
            continue;
        }
        let message = message.unwrap();

        // Parse the message as JSONRPCMessage.
        let message: Result<JSONRPCMessage, _> = serde_json::from_str(&message);
        if message.is_err() {
            write_log("Failed to parse the message as JSONRPCMessage: ");
            write_log(&format!("{:?}", message.err().unwrap()));
            continue;
        }
        let message = message.unwrap();

        // Depending on the method, handle the message.
        if let Some(method) = message.method.as_ref() {
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
                    write_log(msg.as_str());
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
                write_log("Exiting the language server.");
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
                handle_completion(
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
                handle_completion_resolve_document(
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
                handle_hover(
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
                handle_goto_definition(
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
                handle_document_symbol(id.unwrap(), &params.unwrap(), program);
            }
        }
    }
}

fn parase_params<T: DeserializeOwned>(params: Value) -> Option<T> {
    let params: Result<T, _> = serde_json::from_value(params);
    if params.is_err() {
        let mut msg = "Failed to parse the params: \n".to_string();
        msg.push_str(&format!("{:?}", params.err().unwrap()));
        write_log(msg.as_str());
        return None;
    }
    params.ok()
}

fn parse_id(message: &JSONRPCMessage, method: &str) -> Option<u32> {
    if message.id.is_none() {
        write_log(
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

fn open_log_file() -> Mutex<File> {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LOG_FILE_PATH)
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
        .open(LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LOG_FILE_PATH).as_str());

    // Wrap it into a mutex.
    Mutex::new(file)
}

pub fn write_log(message: &str) {
    let mut file = LOG_FILE.lock().expect("Failed to lock the log file.");
    if WRITE_LOG {
        let message = message.to_string() + "\n";
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

// Handle "initialize" method.
fn handle_initialize(id: u32, _params: &InitializeParams) {
    // Return server capabilities.
    let result = InitializeResult {
        capabilities: ServerCapabilities {
            position_encoding: None,
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
            references_provider: None,
            document_highlight_provider: None,
            document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
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
            write_log(msg.as_str());
        }
    });

    // Send `Start` message to the diagnostics thread.
    if let Err(e) = diag_req_send.send(DiagnosticsMessage::Start) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log(msg.as_str());
    }
}

// Handle "shutdown" method.
fn handle_shutdown(id: u32, diag_send: Sender<DiagnosticsMessage>) {
    // Shutdown the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::Stop) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log(msg.as_str());
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
        write_log(msg.as_str());
    }

    // Get the path of the saved file.
    let path = uri_to_path(&params.text_document.uri);

    // Send a message to the diagnostics thread.
    if let Err(e) = diag_send.send(DiagnosticsMessage::OnSaveFile(path)) {
        let mut msg = "Failed to send a message to the diagnostics thread: \n".to_string();
        msg.push_str(&format!("{:?}", e));
        write_log(msg.as_str());
    }
}

fn uri_to_path(uri: &lsp_types::Uri) -> PathBuf {
    PathBuf::from(
        urlencoding::decode(&uri.path().to_string())
            .ok()
            .unwrap()
            .as_ref(),
    )
}

// Handle "textDocument/completion" method.
fn handle_completion(
    id: u32,
    params: &CompletionParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    let text_document_position = &params.text_document_position;
    let typing_text = get_typing_text(text_document_position, uri_to_content);

    let namespace = extract_namespace_from_typing_text(&typing_text);
    let is_in_namespace = |name: &FullName| namespace.is_suffix_of(&name.namespace);

    let mut items = vec![];

    fn create_item(
        name: &FullName,
        kind: CompletionItemKind,
        detail: Option<String>,
        end_node: &EndNode,
        typing_text: &str,
        text_document_position: &TextDocumentPositionParams,
    ) -> CompletionItem {
        CompletionItem {
            label: name.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: None,
            }),
            kind: Some(kind),
            detail,
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: Some(name.name.clone()),
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: None,
            data: Some(
                serde_json::to_value((end_node, typing_text, text_document_position)).unwrap(),
            ),
            tags: None,
        }
    }

    for (full_name, gv) in &program.global_values {
        // Skip compiler-defined entities
        if full_name.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(full_name) {
            continue;
        }
        let scheme = gv
            .syn_scm
            .clone()
            .unwrap_or(gv.scm.clone())
            .to_string_normalize();
        let item = create_item(
            full_name,
            CompletionItemKind::FUNCTION,
            Some(scheme),
            &EndNode::Expr(Var::create(full_name.clone()), None),
            &typing_text,
            &text_document_position,
        );
        items.push(item);
    }
    for (tycon, _kind) in program.type_env.kinds() {
        if tycon.name.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(&tycon.name) {
            continue;
        }
        let item = create_item(
            &tycon.name,
            CompletionItemKind::CLASS,
            None,
            &EndNode::Type(tycon.clone()),
            &typing_text,
            &text_document_position,
        );
        items.push(item);
    }
    for trait_ in program.traits_with_aliases() {
        if trait_.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(&trait_.name) {
            continue;
        }
        let item = create_item(
            &trait_.name,
            CompletionItemKind::INTERFACE,
            None,
            &EndNode::Trait(trait_.clone()),
            &typing_text,
            &text_document_position,
        );
        items.push(item);
    }
    send_response(id, Ok::<_, ()>(items));
}

// Check if the user's typing text is in the form of a dot followed by namespaces or a function name
fn is_dot_function(typing_text: &str) -> bool {
    let mut chars = typing_text.chars().rev();
    let identifer_chars = chars_allowed_in_identifiers();
    while let Some(c) = chars.next() {
        if c == '.' {
            return true;
        }
        if !identifer_chars.contains(c) && c != ':' {
            return false;
        }
    }
    false
}

// Extract namespace from typing text string.
// This function performs string manipulation to extract namespace components from user input.
fn extract_namespace_from_typing_text(typing_text: &str) -> NameSpace {
    // Get the suffix of `typing_text` that consists of characters allowed in identifiers and colons.
    // Example: input "let x = Std::Array:" -> "Std::Array:"
    let mut suffix_len = 0;
    let identifer_chars = chars_allowed_in_identifiers();
    for c in typing_text.chars().rev() {
        if identifer_chars.contains(c) || c == ':' {
            suffix_len += 1;
        } else {
            break;
        }
    }
    let typing_text = typing_text.chars().collect::<Vec<_>>();
    let namespace_part = typing_text[typing_text.len() - suffix_len..typing_text.len()]
        .iter()
        .collect::<String>();

    // Remove the trailing colon
    // Example: "Std::Array:" -> "Std::Array"
    let namespace_part = namespace_part.trim_end_matches(':').to_string();

    // Split the text by "::". If the last component does not start with a uppercase letter, then drop it.
    let mut components = namespace_part.split("::").collect::<Vec<_>>();
    if let Some(last_component) = components.last() {
        let first_char = last_component.chars().nth(0);
        if let Some(first_char) = first_char {
            if !first_char.is_ascii_alphabetic() || !first_char.is_uppercase() {
                components.pop();
            }
        }
    }
    let namespace_str = components
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("::");

    // Convert the namespace string to a `NameSpace`.
    let namespace = NameSpace::parse(&namespace_str);
    if namespace.is_none() {
        return NameSpace::new(vec![]);
    }
    namespace.unwrap()
}

// Get the text of the line being typed by the user up to the cursor position.
fn get_typing_text(
    text_document_position: &lsp_types::TextDocumentPositionParams,
    uri_to_content: &std::collections::HashMap<
        Uri,
        LatestContent,
        std::hash::BuildHasherDefault<fxhash::FxHasher>,
    >,
) -> String {
    let current_line = get_line_string_from_position(uri_to_content, &text_document_position);
    let typing_text = current_line
        .map(|(line, pos)| {
            let mut line = line.chars().collect::<Vec<_>>();
            line.truncate(pos as usize);
            line.into_iter().collect::<String>()
        })
        .unwrap_or_default();
    typing_text
}

// Handle "textDocument/completion" method.
// Add documentation to the completion item.
fn handle_completion_resolve_document(
    id: u32,
    params: &CompletionItem,
    uri_to_content: &mut Map<Uri, LatestContent>,
    program: &Program,
) {
    if params.data.is_none() {
        let msg = "In textDocument/completion, params.data is null.".to_string();
        write_log(msg.as_str());
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }
    let data = params.data.as_ref().unwrap();
    let data =
        serde_json::from_value::<(EndNode, String, TextDocumentPositionParams)>(data.clone());
    if let Err(e) = data {
        let msg = format!(
            "In textDocument/completion, failed to parse params.data as EndNode: {}",
            e
        );
        write_log(msg.as_str());
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }

    let (node, typing_text, text_document_position) = data.unwrap();

    // Is the user completing a function call after a dot?
    let has_dot = is_dot_function(&typing_text);

    // Get the documentation.
    let docs = document_from_endnode(&node, program);

    // Set the documentation into the given completion item.
    let docs = Documentation::MarkupContent(docs);
    let mut item = params.clone();
    item.documentation = Some(docs);

    // If the node is a global value with parameters defined in the document, then add the parameters to the insert text.
    match &node {
        EndNode::Expr(var, _) => {
            if var.name.is_global() {
                let params = parameters_of_global_value(&var.name, program);
                if let Some(mut params) = params {
                    // If the trigger character is ".", then remove the last parameter.
                    if has_dot {
                        params.pop();
                    }

                    // Append argument list to the insert text.
                    if let Some(insert_text) = &mut item.insert_text {
                        if params.len() > 0 {
                            *insert_text += "(";
                            *insert_text += &params.join(", ");
                            *insert_text += ")";
                        }
                    }
                }
            }
        }
        _ => {}
    };

    // Create TextEdits of import statements to import the completion item.
    let import_item_name = match node {
        EndNode::Expr(var, _) => Some(var.name.clone()),
        EndNode::Pattern(_, _) => None,
        EndNode::Type(ty) => Some(ty.name.clone()),
        EndNode::Trait(trait_) => Some(trait_.name.clone()),
        EndNode::Module(_) => None, // Do not auto-import module name by completion.
    };
    if let Some(import_item_name) = import_item_name {
        if let Some(latest_content) =
            uri_to_content.get_mut(&text_document_position.text_document.uri)
        {
            let edits = create_text_edit_to_import(&import_item_name, latest_content);
            if edits.len() > 0 {
                // If the cursor position is included in or near to any of the range of the text edits, do not apply the edits.
                let cursor = &text_document_position.position;
                if !edits.iter().any(|edit| {
                    edit.range.start.line <= cursor.line && cursor.line <= edit.range.end.line
                }) {
                    item.additional_text_edits = Some(edits);
                }
            }
        }
    }

    // Send the completion item.
    send_response(id, Ok::<_, ()>(item));
}

fn create_text_edit_to_import(
    item_name: &FullName,
    latest_content: &mut LatestContent,
) -> Vec<TextEdit> {
    if !item_name.is_global() {
        return vec![];
    }
    let mod_info = latest_content.get_module_info();
    if mod_info.is_none() {
        return vec![];
    }
    let mod_info = mod_info.as_ref().unwrap().clone();
    let mod_name = mod_info.name.clone();
    // No need to import if the item is defined in the same module.
    if item_name.module() == mod_name {
        return vec![];
    }
    let import_stmts = latest_content.get_import_stmts();
    if import_stmts.is_none() {
        return vec![];
    }
    let import_stmts = import_stmts.as_ref().unwrap().clone();

    // Check if the standard library is imported explicitly.
    let import_std_explicitly = import_stmts
        .iter()
        .any(|imp: &ImportStatement| &imp.module.0 == STD_NAME && !imp.implicit);

    // If the item is already accessible, we don't need to import it.
    if !import_std_explicitly && item_name.module() == STD_NAME {
        return vec![];
    }
    if is_accessible(&import_stmts, &item_name) {
        return vec![];
    }

    // If any import_statement's source is None, it is abnormal, so return.
    if import_stmts.iter().any(|imp| imp.source.is_none()) {
        let msg = format!(
            "In create_text_edit_import_to_use, found an import statement with None source.",
        );
        write_log(msg.as_str());
        return vec![];
    }

    // Generate text for new import statements.
    let mut new_import_stmts = import_stmts.clone();
    ImportStatement::add_import(&mut new_import_stmts, mod_name, item_name.clone());
    ImportStatement::sort(&mut new_import_stmts);
    let inserted_text = new_import_stmts
        .iter()
        .map(|stmt| stmt.stringify())
        .collect::<Vec<_>>()
        .join("\n");

    let mut text_edits = vec![];

    // Insert the import statement at the end of the module definition.
    let inserted_text = format!("\n\n{}", inserted_text);
    let span = mod_info.source.to_end_position();
    let range = span_to_range(&span);
    text_edits.push(TextEdit {
        range,
        new_text: inserted_text,
    });

    // Erase all existing import statements.
    let content_lines = latest_content.content.lines().collect::<Vec<_>>();
    for import_stmt in &import_stmts {
        let mut range = span_to_range(&import_stmt.source.as_ref().unwrap());
        // If there are no whitespace characters and line breaks after range.end, expand the range to remove whitespace characters and line breaks as well.
        loop {
            let end_line_content = content_lines.get(range.end.line as usize);
            if let Some(end_line_content) = end_line_content {
                let end_col_content = &end_line_content[range.end.character as usize..];
                if end_col_content.trim().is_empty() {
                    range.end.line += 1;
                    range.end.character = 0;
                    continue;
                }
            }
            break;
        }

        text_edits.push(TextEdit {
            range,
            new_text: "".to_string(),
        });
    }
    text_edits
}

// Get the content of the file at the specified path at the time of the last diagnostics
//
// - `program`: The `Program` obtained from the last diagnostics result
fn get_file_content_at_previous_diagnostics(
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

fn calculate_corresponding_line(content0: &str, content1: &str, line0: u32) -> Option<u32> {
    let (_, diffs) = diff(content0, content1, "\n");
    let mut line_cnt_0 = -1;
    let mut line_cnt_1 = -1;
    for diff in diffs {
        match diff {
            difference::Difference::Same(s) => {
                let lines = s.split("\n").count();
                for _ in 0..lines {
                    line_cnt_0 += 1;
                    line_cnt_1 += 1;
                    if line_cnt_0 == line0 as i32 {
                        return Some(line_cnt_1 as u32);
                    }
                }
            }
            difference::Difference::Add(s) => {
                line_cnt_1 += s.split("\n").count() as i32;
            }
            difference::Difference::Rem(s) => {
                line_cnt_0 += s.split("\n").count() as i32;
            }
        }
    }
    None
}

// Given a `TextDocumentPositionParams`, get the line string and the line number.
// `uri_to_content` is a map to get the source string from the uri of the source file.
fn get_line_string_from_position(
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
    text_position: &TextDocumentPositionParams,
) -> Option<(String, u32)> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log(msg.as_str());
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();
    let latest_content = &latest_content.content;

    // Get the string at line `line` in `latest_content`.
    let line = text_position.position.line;
    let line = line as usize;
    let line_str = latest_content.lines().nth(line).unwrap_or("").to_string();
    Some((line_str, text_position.position.character as u32))
}

fn get_node_at(
    text_position: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) -> Option<EndNode> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log(msg.as_str());
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();

    // Get the path of the file.
    let path = uri_to_path(uri);

    // Get the file content at the time of the last successful diagnostics.
    let saved_content = get_file_content_at_previous_diagnostics(program, &path);
    if let Err(e) = saved_content {
        write_log(&e);
        return None;
    }
    let saved_content = saved_content.ok().unwrap();

    // Get the position of the cursor in `saved_content`.
    let pos_in_latest = text_position.position;
    let line_in_saved =
        calculate_corresponding_line(&latest_content.content, &saved_content, pos_in_latest.line);
    if line_in_saved.is_none() {
        return None;
    }
    let pos_in_saved = lsp_types::Position {
        line: line_in_saved.unwrap(),
        character: pos_in_latest.character,
    };

    // Get the node at the position.
    let pos = SourcePos {
        input: SourceFile::from_file_path(path),
        pos: position_to_bytes(&saved_content, pos_in_saved),
    };
    program.find_node_at(&pos)
}

// Handle "textDocument/definition" method.
fn handle_goto_definition(
    id: u32,
    params: &GotoDefinitionParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) {
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let node = node.unwrap();

    // The source location where the item is defined.
    let mut def_src;

    // First check if the node is an expression or a pattern.
    let var_name = match &node {
        EndNode::Expr(var, _) => Some(var.name.clone()),
        EndNode::Pattern(var, _) => Some(var.name.clone()),
        EndNode::Type(_) => None,
        EndNode::Trait(_) => None,
        EndNode::Module(_) => None,
    };
    if let Some(var_name) = var_name {
        // If the variable is local, do nothing.
        let full_name = &var_name;
        if full_name.is_local() {
            def_src = None;
        } else {
            def_src = program
                .global_values
                .get(full_name)
                .and_then(|gv| gv.def_src.clone());
        }
    } else {
        // Then handle the case of a type or a trait or a module.
        match node {
            EndNode::Expr(_, _) => {
                unreachable!()
            }
            EndNode::Pattern(_, _) => {
                unreachable!()
            }
            EndNode::Type(tycon) => {
                def_src = program
                    .type_env
                    .tycons
                    .get(&tycon)
                    .and_then(|ti| ti.source.clone());
            }
            EndNode::Trait(trait_) => {
                def_src = program
                    .trait_env
                    .traits
                    .get(&trait_)
                    .and_then(|ti| ti.source.clone());
                if def_src.is_none() {
                    def_src = program
                        .trait_env
                        .aliases
                        .get(&trait_)
                        .and_then(|ta| ta.source.clone());
                }
            }
            EndNode::Module(mod_name) => {
                if let Some(mi) = program.modules.iter().find(|mi| mi.name == mod_name) {
                    def_src = Some(mi.source.clone());
                } else {
                    def_src = None;
                }
            }
        }
    }

    // If the source is not found, respond with None.
    if def_src.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let def_src = def_src.unwrap();

    // Create response value.
    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}", cdir.err().unwrap()));
        write_log(msg.as_str());
        return;
    }
    let cdir = cdir.unwrap();
    let uri = path_to_uri(&cdir.join(&def_src.input.file_path));
    if let Err(e) = uri {
        send_response(id, Err::<(), String>(e));
        return;
    }
    let uri = uri.ok().unwrap();
    let location = lsp_types::Location {
        uri,
        range: span_to_range(&def_src),
    };
    send_response(id, Ok::<_, ()>(location));
}

// Handle "textDocument/documentSymbol" method.
fn handle_document_symbol(id: u32, params: &DocumentSymbolParams, program: &Program) {
    let canonicalize_path = |path| {
        let path = to_absolute_path(path);
        if let Err(e) = path {
            let msg = e.to_string();
            write_log(&msg);
            return None;
        }
        path.ok()
    };

    let path = uri_to_path(&params.text_document.uri);
    let path = match canonicalize_path(&path) {
        Some(path) => path,
        None => return,
    };

    let mut symbols = Vec::new();

    // Extract type constructors from type environment
    for (tycon, tycon_info) in program.type_env.tycons.as_ref() {
        // Skip compiler-defined entities
        if tycon.name.to_string().contains('#') {
            continue;
        }
        if let Some(span) = &tycon_info.source {
            let sym_path = canonicalize_path(&span.input.file_path);
            if sym_path.is_none() {
                continue;
            }
            let sym_path = sym_path.unwrap();
            if sym_path == path {
                let symbol = create_symbol_from_tycon(tycon, tycon_info);
                symbols.push(symbol);
            }
        }
    }

    // Extract type aliases from type environment
    for (tycon, alias_info) in program.type_env.aliases.as_ref() {
        // Skip compiler-defined entities
        if tycon.name.to_string().contains('#') {
            continue;
        }
        if let Some(span) = &alias_info.source {
            let sym_path = canonicalize_path(&span.input.file_path);
            if sym_path.is_none() {
                continue;
            }
            let sym_path = sym_path.unwrap();
            if sym_path == path {
                let symbol = create_symbol_from_type_alias(tycon, alias_info);
                symbols.push(symbol);
            }
        }
    }

    // Extract global values (functions, constants)
    for (full_name, global_value) in &program.global_values {
        // Skip compiler-defined entities
        if full_name.to_string().contains('#') {
            continue;
        }
        if let Some(span) = &global_value.def_src {
            let sym_path = canonicalize_path(&span.input.file_path);
            if sym_path.is_none() {
                continue;
            }
            let sym_path = sym_path.unwrap();
            if sym_path == path {
                let symbol = create_symbol_from_global_value(full_name, global_value);
                symbols.push(symbol);
            }
        }
    }

    // Extract trait definitions from trait environment
    for (trait_, trait_info) in &program.trait_env.traits {
        // Skip compiler-defined entities
        if trait_.name.to_string().contains('#') {
            continue;
        }
        if let Some(span) = &trait_info.source {
            let sym_path = canonicalize_path(&span.input.file_path);
            if sym_path.is_none() {
                continue;
            }
            let sym_path = sym_path.unwrap();
            if sym_path == path {
                let symbol = create_symbol_from_trait_info(trait_, trait_info);
                symbols.push(symbol);
            }
        }
    }

    // Extract trait aliases from trait environment
    for (trait_, trait_alias) in &program.trait_env.aliases {
        // Skip compiler-defined entities
        if trait_.name.to_string().contains('#') {
            continue;
        }
        if let Some(span) = &trait_alias.source {
            let sym_path = canonicalize_path(&span.input.file_path);
            if sym_path.is_none() {
                continue;
            }
            let sym_path = sym_path.unwrap();
            if sym_path == path {
                let symbol = create_symbol_from_trait_alias(trait_alias);
                symbols.push(symbol);
            }
        }
    }

    // Extract trait instances from trait environment
    for (trait_, instances) in &program.trait_env.instances {
        // Skip compiler-defined entities
        if trait_.name.to_string().contains('#') {
            continue;
        }
        for instance in instances {
            // Only include user-defined instances
            if !instance.is_user_defined {
                continue;
            }
            if let Some(span) = &instance.source {
                let sym_path = canonicalize_path(&span.input.file_path);
                if sym_path.is_none() {
                    continue;
                }
                let sym_path = sym_path.unwrap();
                if sym_path == path {
                    let symbol = create_symbol_from_trait_instance(trait_, instance);
                    symbols.push(symbol);
                }
            }
        }
    }

    send_response(id, Ok::<_, ()>(symbols));
}

#[allow(deprecated)]
fn create_symbol_from_tycon(tycon: &TyCon, tycon_info: &TyConInfo) -> DocumentSymbol {
    let name = tycon.name.to_string();
    let range = tycon_info
        .source
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    let (kind, detail) = match &tycon_info.variant {
        TyConVariant::Struct => (SymbolKind::STRUCT, Some("struct".to_string())),
        TyConVariant::Union => (SymbolKind::ENUM, Some("union".to_string())),
        TyConVariant::Primitive => (SymbolKind::CLASS, Some("primitive type".to_string())),
        TyConVariant::Arrow => (SymbolKind::CLASS, Some("function type".to_string())),
        TyConVariant::Array => (SymbolKind::CLASS, Some("array type".to_string())),
        TyConVariant::DynamicObject => (SymbolKind::CLASS, Some("dynamic object type".to_string())),
    };

    DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

#[allow(deprecated)]
fn create_symbol_from_type_alias(tycon: &TyCon, alias_info: &TyAliasInfo) -> DocumentSymbol {
    let name = tycon.name.to_string();
    let range = alias_info
        .source
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    DocumentSymbol {
        name,
        detail: Some("type alias".to_string()),
        kind: SymbolKind::CLASS,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

#[allow(deprecated)]
fn create_symbol_from_global_value(
    full_name: &FullName,
    global_value: &GlobalValue,
) -> DocumentSymbol {
    let name = full_name.to_string();
    let range = global_value
        .def_src
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    let detail = Some(global_value.scm.to_string_normalize());

    DocumentSymbol {
        name,
        detail,
        kind: SymbolKind::FUNCTION,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

#[allow(deprecated)]
fn create_symbol_from_trait_info(trait_: &Trait, trait_info: &TraitInfo) -> DocumentSymbol {
    let name = trait_.name.to_string();
    let range = trait_info
        .source
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    DocumentSymbol {
        name,
        detail: Some("trait".to_string()),
        kind: SymbolKind::INTERFACE,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

#[allow(deprecated)]
fn create_symbol_from_trait_alias(trait_alias: &TraitAlias) -> DocumentSymbol {
    let name = trait_alias.id.name.to_string();
    let range = trait_alias
        .source
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    DocumentSymbol {
        name,
        detail: Some("trait".to_string()),
        kind: SymbolKind::INTERFACE,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

#[allow(deprecated)]
fn create_symbol_from_trait_instance(
    trait_: &Trait,
    trait_instance: &TraitInstance,
) -> DocumentSymbol {
    let name = format!("impl {}", trait_instance.qual_pred.to_string());
    let range = trait_instance
        .source
        .as_ref()
        .map(span_to_range)
        .unwrap_or_default();
    let selection_range = range.clone();

    let detail = Some(format!(
        "trait implementation for {}",
        trait_.name.to_string()
    ));

    DocumentSymbol {
        name,
        detail,
        kind: SymbolKind::MODULE,
        tags: None,
        deprecated: Some(false),
        range,
        selection_range,
        children: None,
    }
}

// Handle "textDocument/hover" method.
fn handle_hover(
    id: u32,
    params: &HoverParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) {
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let node = node.unwrap();
    let content = document_from_endnode(&node, program);
    let hover = lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(content),
        range: None,
    };
    send_response(id, Ok::<_, ()>(hover))
}

// Get the parameters of a global value from its documentation.
fn parameters_of_global_value(full_name: &FullName, program: &Program) -> Option<Vec<String>> {
    // Get the document of the global value, which is a markdown string.
    let opt_gv = program.global_values.get(full_name);
    if opt_gv.is_none() {
        return None;
    }
    let gv = opt_gv.unwrap();
    let opt_docs = gv.get_document();
    if opt_docs.is_none() {
        return None;
    }
    let docs = opt_docs.unwrap();
    let sections = MarkdownSection::parse_many(docs.lines().collect());

    // Find the first top-level or second-level section named "Parameters".
    // let param_section = sections.iter().find(|sec| sec.title.trim() == "Parameters");
    let param_section = sections.iter().find_map(|sec| {
        if sec.title.trim() == "Parameters" {
            Some(sec)
        } else {
            sec.subsections
                .iter()
                .find(|subsec| subsec.title.trim() == "Parameters")
        }
    });
    if param_section.is_none() {
        return None;
    }
    let param_section = param_section.unwrap();

    // Collect the top-level list items.
    let mut params = vec![];
    for paragraph in &param_section.paragraphs {
        for line in paragraph.lines() {
            if line.starts_with("- ") || line.starts_with("* ") {
                // Find the first backquoted sequence of characters.
                let mut quoted_str = String::new();
                let mut in_backquote = false;
                for c in line.chars() {
                    if c == '`' {
                        if in_backquote {
                            in_backquote = false;
                            break;
                        } else {
                            in_backquote = true;
                            continue;
                        }
                    } else if in_backquote {
                        quoted_str.push(c);
                    }
                }
                // If the line ends in backquote, then skip it.
                if in_backquote {
                    continue;
                }

                // Find the first continuous sequence of characters that are allowed in identifiers.
                let name_chars = chars_allowed_in_identifiers();
                let mut param = String::new();
                for c in quoted_str.chars() {
                    if name_chars.contains(c) {
                        param.push(c);
                    } else if !param.is_empty() {
                        break;
                    }
                }

                // If the parameter is empty, skip it.
                if param.is_empty() {
                    continue;
                }

                params.push(param.to_string());
            }
        }
    }

    Some(params)
}

fn document_from_endnode(node: &EndNode, program: &Program) -> MarkupContent {
    // Create a hover message.
    let mut docs = String::new();
    match node {
        EndNode::Expr(var, ty) => {
            // Get informations of the variable which are needed to show in the hover.
            let full_name = &var.name;

            if full_name.is_local() {
                // In case the variable is local, show the name and type of the variable.
                if let Some(ty) = ty.as_ref() {
                    docs += &format!(
                        "```\n{} : {}\n```",
                        full_name.to_string(),
                        ty.to_string_normalize()
                    );
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
            } else {
                // In case the variable is global, show the documentation of the global value.
                let mut scm_string = String::new();
                if let Some(gv) = program.global_values.get(full_name) {
                    scm_string = gv
                        .syn_scm
                        .clone()
                        .unwrap_or(gv.scm.clone())
                        .to_string_normalize();
                    docs += &format!("```\n{} : {}\n```", full_name.to_string(), scm_string);
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
                if let Some(ty) = ty.as_ref() {
                    let ty_string = ty.to_string_normalize();
                    if scm_string != ty_string {
                        docs += &format!("\nInstantiated as:\n```\n{}\n```", ty_string);
                    }
                }
                if let Some(gv) = program.global_values.get(full_name) {
                    if let Some(document) = gv.get_document() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            };
        }
        EndNode::Pattern(var, ty) => {
            // In case the node is a variable, show the name and type of the variable.
            if let Some(ty) = ty.as_ref() {
                docs += &format!(
                    "```\n{} : {}\n```",
                    var.name.to_string(),
                    ty.to_string_normalize()
                );
            } else {
                docs += &format!("```\n{}\n```", var.name.to_string());
            }
        }
        EndNode::Type(tycon) => {
            docs += &format!("```\n{}\n```", tycon.to_string());
            if let Some(ti) = program.type_env.tycons.get(&tycon) {
                if let Some(document) = ti.get_document() {
                    docs += &format!("\n\n{}", document);
                }
            }
        }
        EndNode::Trait(trait_id) => {
            docs += &format!("```\n{}\n```", trait_id.to_string());
            if let Some(ti) = program.trait_env.traits.get(&trait_id) {
                if let Some(document) = ti.get_document() {
                    docs += &format!("\n\n{}", document);
                }
            }
        }
        EndNode::Module(mod_name) => {
            docs += &format!("```\nmodule {}\n```", mod_name.to_string());
            if let Some(mi) = program.modules.iter().find(|mi| &mi.name == mod_name) {
                if let Some(document) = mi.source.get_document().ok() {
                    if !document.trim().is_empty() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            }
        }
    }
    let content = MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: docs,
    };
    content
}

// Convert a `lsp_types::Position` into a bytes position in a string.
fn position_to_bytes(string: &str, position: lsp_types::Position) -> usize {
    let mut bytes = 0;
    let mut line = 0;
    let mut pos = 0;
    for c in string.chars() {
        bytes += c.len_utf8();
        pos += 1;
        if c == '\n' {
            line += 1;
            pos = 0;
        }
        if line == position.line && pos == position.character as usize {
            break;
        }
    }
    bytes
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
fn send_diagnostics_notification(errs: Errors, mut prev_err_paths: Set<PathBuf>) -> Set<PathBuf> {
    let mut err_paths = Set::default();

    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}", cdir.err().unwrap()));
        write_log(msg.as_str());
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
            write_log(&format!(
                "Failed to convert path to uri: {:?}",
                uri.unwrap_err()
            ));
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
            write_log(&(uri.unwrap_err()));
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
    // Get the current directory.
    let cdir = std::env::current_dir();
    if cdir.is_err() {
        let mut msg = "Failed to get the current directory: \n".to_string();
        msg.push_str(&format!("{:?}", cdir.err().unwrap()));
        write_log(msg.as_str());
        return;
    }
    let cdir = cdir.unwrap();
    // Convert path to uri.
    let cdir_uri = path_to_uri(&cdir);
    if cdir_uri.is_err() {
        write_log(&format!(
            "Failed to convert path to uri: {:?}",
            cdir_uri.unwrap_err()
        ));
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
        // Convert path to uri.
        let uri = path_to_uri(&cdir.join(&span.input.file_path));
        if uri.is_err() {
            write_log(&format!(
                "Failed to convert path to uri: {:?}",
                uri.unwrap_err()
            ));
            continue;
        }
        let uri = uri.unwrap();

        // Create related informations.
        let related = lsp_types::DiagnosticRelatedInformation {
            location: lsp_types::Location {
                uri,
                range: span_to_range(span),
            },
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
        code: None,
        code_description: None,
        source: None,
        message: err.msg.clone(),
        tags: None,
        related_information,
        data: None,
    }
}

fn path_to_uri(path: &PathBuf) -> Result<lsp_types::Uri, String> {
    // URI-encode each component of the path.
    let path = to_absolute_path(path).map_err(|_| {
        format!(
            "Failed to get the absolute path of the file: \"{}\"",
            path.to_string_lossy().to_string()
        )
    })?;
    let mut components = vec![];
    for comp in path.components() {
        match comp {
            std::path::Component::Normal(comp) => {
                let comp = comp.to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                let comp = urlencoding::encode(comp.unwrap()).to_string();
                components.push(comp);
            }
            std::path::Component::Prefix(prefix) => {
                let comp = prefix.as_os_str().to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                components.push(comp.unwrap().to_string());
            }
            std::path::Component::RootDir => {}
            std::path::Component::CurDir => unreachable!(),
            std::path::Component::ParentDir => unreachable!(),
        }
    }
    let path = "file:///".to_string() + components.join("/").as_str();
    let uri = lsp_types::Uri::from_str(&path);
    if uri.is_err() {
        return Err(format!("Failed to convert a path into Uri: {:?}", path));
    }
    Ok(uri.unwrap())
}

pub fn run_diagnostics(typecheck_cache: SharedTypeCheckCache) -> Result<DiagnosticsResult, Errors> {
    // TODO: maybe we should check if the file has been changed actually after previous diagnostics?

    // Read the project file.
    let proj_file = ProjectFile::read_root_file()?;

    // Determine the source files for which diagnostics are run.
    let files = proj_file.get_files(true);

    // Create the configuration.
    let mut config = Configuration::diagnostics_mode(DiagnosticsConfig { files })?;
    config.type_check_cache = typecheck_cache;

    // Set up the configuration by the project file.
    proj_file.set_config(&mut config, false)?;

    // Set up the configuration by the lock file.
    proj_file.open_lock_file()?.set_config(&mut config)?;

    // Build the file and get the errors.
    let program = build_file(&mut config)?.program.unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_namespace_from_typing_text_basic() {
        // Test case based on comment: "let x = Std::Array:"
        let result = extract_namespace_from_typing_text("let x = Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
        assert_eq!(result.is_absolute, false);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_simple() {
        // Test case: "Std::Array:"
        let result = extract_namespace_from_typing_text("Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_single() {
        // Test case: "Std:"
        let result = extract_namespace_from_typing_text("Std:");
        assert_eq!(result.names, vec!["Std".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_no_colon() {
        // Test case: "Std::Array" (no trailing colon)
        let result = extract_namespace_from_typing_text("Std::Array");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_lowercase_last() {
        // Test case: "Std::Array::get" - last component starts with lowercase, should be dropped
        let result = extract_namespace_from_typing_text("Std::Array::get");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_empty() {
        // Test case: empty string
        let result = extract_namespace_from_typing_text("");
        assert_eq!(result.names, Vec::<String>::new());
    }

    #[test]
    fn test_extract_namespace_from_typing_text_no_namespace() {
        // Test case: "SomeVariable" - no namespace separator
        let result = extract_namespace_from_typing_text("SomeVariable");
        assert_eq!(result.names, vec!["SomeVariable".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_with_special_chars() {
        // Test case: "func(Std::Array:" - function call with namespace
        let result = extract_namespace_from_typing_text("func(Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_nested() {
        // Test case: "A::B::C::D:" - deeply nested namespace
        let result = extract_namespace_from_typing_text("A::B::C::D:");
        assert_eq!(
            result.names,
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string()
            ]
        );
    }

    #[test]
    fn test_extract_namespace_from_typing_text_partial() {
        // Test case: "Std::arr" - partial typing with lowercase
        let result = extract_namespace_from_typing_text("Std::arr");
        assert_eq!(result.names, vec!["Std".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_with_operators() {
        // Test case: "x + Std::Array:" - with operators before
        let result = extract_namespace_from_typing_text("x + Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_whitespace() {
        // Test case: "    Std::Array:" - with leading whitespace
        let result = extract_namespace_from_typing_text("    Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }
}
