// LSP "Rename Symbol" implementation.
//
// Phase C1 covers local variables and global values; later phases extend
// to types, traits, associated types, struct fields, and union variants.

use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{
    PrepareRenameResponse, Range, RenameParams, TextDocumentPositionParams, TextEdit, Uri,
    WorkspaceEdit,
};
use serde::Serialize;

use super::references::find_global_value_references;
use super::server::{send_response, LatestContent};
use super::util::{
    find_local_occurrences, get_current_dir, path_to_uri, resolve_source_pos, span_to_range,
};
use crate::ast::program::{EndNode, Program};
use crate::misc::Map;
use crate::parse::parser::{validate_token_str, TokenCategory};
use crate::parse::sourcefile::Span;

// LSP `ResponseError` shape: `{ code, message }`.
#[derive(Serialize)]
struct ResponseError {
    code: i64,
    message: String,
}

impl ResponseError {
    fn invalid_request(message: impl Into<String>) -> Self {
        // -32600 is the JSON-RPC reserved code for Invalid Request.
        ResponseError {
            code: -32600,
            message: message.into(),
        }
    }
}

// Handle "textDocument/prepareRename".
//
// Returns `null` for positions where rename is not supported (the LSP
// client treats this as "rename not allowed at this position"), and
// `defaultBehavior: true` for supported positions (the client computes
// the token range itself using its own tokenizer, which is good enough
// for the simple identifier kinds Phase C1 supports).
pub(super) fn handle_prepare_rename(
    id: u32,
    params: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    let Some(pos) = resolve_source_pos(params, program, uri_to_content) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };

    if !is_rename_supported_in_phase_c1(&node) {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    }

    let resp = PrepareRenameResponse::DefaultBehavior {
        default_behavior: true,
    };
    send_response(id, Ok::<_, ()>(Some(resp)));
}

// Handle "textDocument/rename".
pub(super) fn handle_rename(
    id: u32,
    params: &RenameParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    let new_name = &params.new_name;

    let Some(pos) = resolve_source_pos(&params.text_document_position, program, uri_to_content)
    else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };

    if !is_rename_supported_in_phase_c1(&node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Rename is not yet supported for this kind of symbol.",
            )),
        );
        return;
    }

    // Validate new_name. Phase C1 only supports value-category names
    // (locals + globals), so we always use TokenCategory::Name.
    if let Err(msg) = validate_token_str(new_name, TokenCategory::Name) {
        send_response(id, Err::<(), _>(ResponseError::invalid_request(msg)));
        return;
    }

    // Collect (Span, new_text) pairs for all occurrences.
    let edits: Vec<(Span, String)> = match &node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                let Some(occ) = find_local_occurrences(program, &pos, name) else {
                    send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
                    return;
                };
                let mut spans = vec![occ.definition];
                spans.extend(occ.uses);
                spans
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            } else {
                find_global_value_references(program, name, true)
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            }
        }
        EndNode::ValueDecl(name) => find_global_value_references(program, name, true)
            .into_iter()
            .map(|s| (s, new_name.clone()))
            .collect(),
        _ => unreachable!("Unsupported EndNode passed the C1 filter"),
    };

    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };
    let workspace_edit = build_workspace_edit(edits, &cdir);
    send_response(id, Ok::<_, ()>(Some(workspace_edit)));
}

// Whether Phase C1 supports renaming the symbol at this EndNode.
fn is_rename_supported_in_phase_c1(node: &EndNode) -> bool {
    matches!(
        node,
        EndNode::Expr(_, _) | EndNode::Pattern(_, _) | EndNode::ValueDecl(_)
    )
}

// Group `(Span, new_text)` pairs by URI and produce a `WorkspaceEdit`.
// Spans whose paths cannot be converted to URIs are silently dropped.
// Within each URI, edits are deduplicated to avoid multiple TextEdits at
// the same range (which can happen when refs reports both decl_src and
// defn_src for a `name : T = ...` form).
fn build_workspace_edit(edits: Vec<(Span, String)>, cdir: &PathBuf) -> WorkspaceEdit {
    let mut by_uri: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    for (span, new_text) in edits {
        let uri = match path_to_uri(&cdir.join(&span.input.file_path)) {
            Ok(u) => u,
            Err(_) => continue,
        };
        let range = span_to_range(&span);
        by_uri.entry(uri).or_default().push(TextEdit { range, new_text });
    }
    // Deduplicate (range, new_text) pairs per URI.
    for edits in by_uri.values_mut() {
        edits.sort_by(|a, b| {
            (
                a.range.start.line,
                a.range.start.character,
                a.range.end.line,
                a.range.end.character,
            )
                .cmp(&(
                    b.range.start.line,
                    b.range.start.character,
                    b.range.end.line,
                    b.range.end.character,
                ))
        });
        edits.dedup_by(|a, b| range_eq(&a.range, &b.range) && a.new_text == b.new_text);
    }
    WorkspaceEdit {
        changes: Some(by_uri),
        document_changes: None,
        change_annotations: None,
    }
}

fn range_eq(a: &Range, b: &Range) -> bool {
    a.start.line == b.start.line
        && a.start.character == b.start.character
        && a.end.line == b.end.line
        && a.end.character == b.end.character
}

