// LSP "Rename Symbol" implementation.
//
// Phase C1 covers local variables and global values.
// Phase C2 adds type aliases, traits, trait aliases, associated types,
// struct fields, and union variants. Renaming a struct or union type
// itself is deferred to Phase D because it requires updating the
// auto-implemented method namespace.

use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{
    PrepareRenameResponse, Range, RenameParams, TextDocumentPositionParams, TextEdit, Uri,
    WorkspaceEdit,
};
use serde::Serialize;

use super::references::{
    find_assoc_type_references, find_global_value_references, find_member_occurrences,
    find_trait_references, find_type_references,
};
use super::server::{send_response, LatestContent};
use super::util::{
    find_local_occurrences, get_current_dir, path_to_uri, resolve_source_pos, span_to_range,
};
use crate::ast::program::{EndNode, Program};
use crate::ast::traits::TraitId;
use crate::ast::typedecl::TypeDeclValue;
use crate::ast::types::TyCon;
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
// the token range itself).
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

    if !rename_target_supported(program, &node) {
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

    if !rename_target_supported(program, &node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                rename_unsupported_message(program, &node),
            )),
        );
        return;
    }

    // Validate new_name against the appropriate token category.
    let category = token_category_for(&node);
    if let Err(msg) = validate_token_str(new_name, category) {
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
        EndNode::Type(tycon) => find_type_references(program, tycon, true)
            .into_iter()
            .map(|s| (s, new_name.clone()))
            .collect(),
        EndNode::TypeOrTrait(name) => {
            // Resolve to either a type or a trait. Type takes precedence
            // because in `program.type_env` aliases are also registered.
            let tycon = TyCon { name: name.clone() };
            if program.type_env.tycons.contains_key(&tycon)
                || program.type_env.aliases.contains_key(&tycon)
            {
                find_type_references(program, &tycon, true)
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            } else {
                let trait_id = TraitId::from_fullname(name.clone());
                find_trait_references(program, &trait_id, true)
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            }
        }
        EndNode::Trait(trait_id) => find_trait_references(program, trait_id, true)
            .into_iter()
            .map(|s| (s, new_name.clone()))
            .collect(),
        EndNode::AssocType(assoc_type) => {
            find_assoc_type_references(program, assoc_type, true)
                .into_iter()
                .map(|s| (s, new_name.clone()))
                .collect()
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
            find_member_occurrences(program, tc, name, true)
                .into_iter()
                .map(|occ| (occ.span, format!("{}{}", occ.prefix, new_name)))
                .collect()
        }
        EndNode::Module(_) => unreachable!("Module rename is filtered out earlier"),
    };

    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };
    let workspace_edit = build_workspace_edit(edits, &cdir);
    send_response(id, Ok::<_, ()>(Some(workspace_edit)));
}

// Whether the symbol at this EndNode is renameable at all (in any phase
// the implementation has reached so far). Used by both prepareRename and
// rename to keep their answers consistent.
fn rename_target_supported(program: &Program, node: &EndNode) -> bool {
    match node {
        EndNode::Expr(_, _) | EndNode::Pattern(_, _) | EndNode::ValueDecl(_) => true,
        EndNode::Trait(_) | EndNode::AssocType(_) => true,
        EndNode::Field(_, _) | EndNode::Variant(_, _) => true,
        EndNode::Type(tc) => !is_struct_or_union_type(program, tc),
        EndNode::TypeOrTrait(name) => {
            let tc = TyCon { name: name.clone() };
            if program.type_env.tycons.contains_key(&tc)
                || program.type_env.aliases.contains_key(&tc)
            {
                !is_struct_or_union_type(program, &tc)
            } else {
                // Treated as a trait — supported.
                true
            }
        }
        EndNode::Module(_) => false,
    }
}

// Diagnostic message for the unsupported-target rejection. Keeps
// prepareRename and rename consistent in what they tell the user.
fn rename_unsupported_message(program: &Program, node: &EndNode) -> String {
    match node {
        EndNode::Type(tc) if is_struct_or_union_type(program, tc) => {
            "Renaming struct or union types is not yet supported.".to_string()
        }
        EndNode::TypeOrTrait(name) => {
            let tc = TyCon { name: name.clone() };
            if is_struct_or_union_type(program, &tc) {
                "Renaming struct or union types is not yet supported.".to_string()
            } else {
                "Rename is not supported for this kind of symbol.".to_string()
            }
        }
        EndNode::Module(_) => "Renaming modules is not supported.".to_string(),
        _ => "Rename is not supported for this kind of symbol.".to_string(),
    }
}

// True iff `tc` is defined as a struct or union (as opposed to a type
// alias or a built-in TyCon). Struct/union types own an auto-namespace
// of compiler-generated methods and require Phase D handling, which is
// not yet implemented.
fn is_struct_or_union_type(program: &Program, tc: &TyCon) -> bool {
    program
        .type_defns
        .iter()
        .find(|td| td.tycon() == *tc)
        .map(|td| matches!(td.value, TypeDeclValue::Struct(_) | TypeDeclValue::Union(_)))
        .unwrap_or(false)
}

// Pick the pest grammar token category that the new name must satisfy.
fn token_category_for(node: &EndNode) -> TokenCategory {
    match node {
        EndNode::Expr(_, _) | EndNode::Pattern(_, _) | EndNode::ValueDecl(_) => TokenCategory::Name,
        EndNode::Type(_) | EndNode::TypeOrTrait(_) | EndNode::Trait(_) | EndNode::AssocType(_) => {
            TokenCategory::CapitalName
        }
        EndNode::Field(_, _) | EndNode::Variant(_, _) => TokenCategory::TypeFieldName,
        EndNode::Module(_) => TokenCategory::CapitalName,
    }
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

