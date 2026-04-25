// LSP "Rename Symbol" implementation.
//
// Phase C1 covers local variables and global values.
// Phase C2 adds type aliases, traits, trait aliases, associated types,
// struct fields, and union variants. Renaming a struct or union type
// itself is deferred to Phase D because it requires updating the
// auto-implemented method namespace.
// Phase C3 adds gating: stale-buffer detection, refusal to rename
// symbols defined outside the project, and refusal to rename
// auto-generated accessors directly.

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
use crate::misc::{to_absolute_path, Map};
use crate::parse::parser::{validate_token_str, TokenCategory};
use crate::parse::sourcefile::{SourcePos, Span};

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
    // Stale-buffer check first: if the editor buffer has drifted from the
    // AST the program was built against, even resolve_source_pos and
    // find_node_at can land on the wrong node, so bail out before any
    // node-shape analysis.
    if check_buffer_in_sync_with_program(program, uri_to_content).is_err() {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    }

    let Some(pos) = resolve_source_pos(params, program, uri_to_content) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };

    if !rename_target_supported(program, &node)
        || is_auto_method_var(program, &node)
        || !target_is_user_defined(program, &node, &pos)
    {
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

    // Stale-buffer check first (see prepareRename for reasoning).
    if let Err(msg) = check_buffer_in_sync_with_program(program, uri_to_content) {
        send_response(id, Err::<(), _>(ResponseError::invalid_request(msg)));
        return;
    }

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

    // Reject auto-generated accessors before any further analysis. The
    // user must rename the field/variant itself instead, where they don't
    // have to think about whether the new name should include the `@`,
    // `set_`, etc. prefix.
    if is_auto_method_var(program, &node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename an auto-generated accessor. \
                 Rename the field or variant declaration instead.",
            )),
        );
        return;
    }

    // Reject symbols whose declaration lives outside this project's source
    // tree (i.e. not listed in `fixproj.toml`'s `files` section).
    if !target_is_user_defined(program, &node, &pos) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename a symbol defined outside this project \
                 (e.g. Std or a dependency).",
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

// Compare each user source file's current content (from the editor buffer
// if open, otherwise from disk) against the content recorded at
// elaboration time. If any mismatch is found, return an error message
// suitable for surfacing to the user.
//
// The strict whole-project policy (per the rename plan) is intentional:
// rename touches AST spans, and any drift between the AST and the buffer
// can produce silently corrupt edits.
fn check_buffer_in_sync_with_program(
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) -> Result<(), String> {
    if program.source_contents.is_empty() {
        // The diagnostics thread didn't (or couldn't) record source
        // contents; treat that as "we don't know" and refuse rather than
        // produce edits we can't validate.
        return Err(
            "Cannot rename: source contents from the last build are not available. \
             Save the file and wait for diagnostics to refresh."
                .to_string(),
        );
    }

    // Collect (absolute path -> current buffer content) from `uri_to_content`
    // so we can look up by path below.
    let mut buffer_by_path: Map<PathBuf, String> = Map::default();
    for lc in uri_to_content.values() {
        if let Ok(abs) = to_absolute_path(&lc.path) {
            buffer_by_path.insert(abs, lc.content.clone());
        }
    }

    for path in &program.user_source_files {
        let elaborated = match program.source_contents.get(path) {
            Some(c) => c,
            // No record of this file; can't verify, so be safe.
            None => {
                return Err(format!(
                    "Cannot rename: source contents missing for `{}`. \
                     Save the file and wait for diagnostics to refresh.",
                    path.display()
                ));
            }
        };

        let current_owned: String;
        let current: &str = if let Some(buf) = buffer_by_path.get(path) {
            buf.as_str()
        } else {
            // No editor buffer — read from disk.
            current_owned = std::fs::read_to_string(path).unwrap_or_default();
            current_owned.as_str()
        };

        if current != elaborated {
            return Err(format!(
                "Cannot rename: `{}` has been edited since the last successful build. \
                 Save the file and wait for diagnostics to refresh.",
                path.display()
            ));
        }
    }
    Ok(())
}

// True if the EndNode at `pos` resolves to an auto-generated accessor
// (a global value with `compiler_defined_method == true`). Used to reject
// rename starting on `@x`, `set_x`, `act_x`, `[^x]`, `as_v`, `is_v`,
// `mod_v` and the union variant constructor — the user should rename the
// field or variant declaration itself.
fn is_auto_method_var(program: &Program, node: &EndNode) -> bool {
    let name = match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => &var.name,
        EndNode::ValueDecl(name) => name,
        _ => return false,
    };
    if name.is_local() {
        return false;
    }
    program
        .global_values
        .get(name)
        .map(|gv| gv.compiler_defined_method)
        .unwrap_or(false)
}

// True if the symbol at `node` is declared in a file listed in
// `fixproj.toml`'s `files` section. `pos` is the cursor position; for
// local variables we use the scope walker to find the binder span.
fn target_is_user_defined(program: &Program, node: &EndNode, pos: &SourcePos) -> bool {
    let decl_span = declaration_span(program, node, pos);
    let Some(span) = decl_span else {
        // No declaration span => can't verify => be conservative and refuse.
        return false;
    };
    let Ok(abs) = to_absolute_path(&span.input.file_path) else {
        return false;
    };
    program.user_source_files.contains(&abs)
}

// Return a span pointing to where the symbol at `node` is declared
// (defined) in source, used for the user-defined check. May return None
// if the symbol has no recorded declaration span (e.g. compiler builtins
// or a local that fails the scope lookup).
fn declaration_span(program: &Program, node: &EndNode, pos: &SourcePos) -> Option<Span> {
    match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                find_local_occurrences(program, pos, name).map(|o| o.definition)
            } else {
                program
                    .global_values
                    .get(name)
                    .and_then(|gv| gv.decl_src.clone().or_else(|| gv.defn_src.clone()))
            }
        }
        EndNode::ValueDecl(name) => program
            .global_values
            .get(name)
            .and_then(|gv| gv.decl_src.clone().or_else(|| gv.defn_src.clone())),
        EndNode::Type(tc) => program
            .type_defns
            .iter()
            .find(|td| td.tycon() == *tc)
            .and_then(|td| td.name_src.clone()),
        EndNode::TypeOrTrait(name) => {
            let tc = TyCon { name: name.clone() };
            if let Some(td) = program.type_defns.iter().find(|td| td.tycon() == tc) {
                td.name_src.clone()
            } else {
                let trait_id = TraitId::from_fullname(name.clone());
                program
                    .trait_env
                    .traits
                    .get(&trait_id)
                    .and_then(|ti| ti.name_src.clone())
                    .or_else(|| {
                        program
                            .trait_env
                            .aliases
                            .data
                            .get(&trait_id)
                            .and_then(|ta| ta.name_src.clone())
                    })
            }
        }
        EndNode::Trait(trait_id) => program
            .trait_env
            .traits
            .get(trait_id)
            .and_then(|ti| ti.name_src.clone())
            .or_else(|| {
                program
                    .trait_env
                    .aliases
                    .data
                    .get(trait_id)
                    .and_then(|ta| ta.name_src.clone())
            }),
        EndNode::AssocType(at) => {
            let trait_id = at.trait_id();
            program
                .trait_env
                .traits
                .get(&trait_id)
                .and_then(|ti| ti.assoc_types.get(&at.name.name))
                .and_then(|atd| atd.name_src.clone())
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => program
            .type_defns
            .iter()
            .find(|td| td.tycon() == *tc)
            .and_then(|td| {
                let fields = match &td.value {
                    TypeDeclValue::Struct(s) => &s.fields,
                    TypeDeclValue::Union(u) => &u.fields,
                    TypeDeclValue::Alias(_) => return None,
                };
                fields
                    .iter()
                    .find(|f| &f.name == name)
                    .and_then(|f| f.name_src.clone())
            }),
        EndNode::Module(_) => None,
    }
}

