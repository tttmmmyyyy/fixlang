// LSP "workspace/symbol" handler.
//
// Returns symbols defined in the user's project files (entries in
// `DiagnosticsResult::user_source_contents`). Symbols from the standard
// library and from dependencies are excluded so the picker stays useful
// in a real project.

use super::server::{send_response, DiagnosticsResult};
use super::util::{get_current_dir, span_to_location};
use crate::ast::types::{TyConInfo, TyConVariant};
use crate::misc::to_absolute_path;
use crate::parse::sourcefile::Span;
use lsp_types::{
    SymbolInformation, SymbolKind, WorkspaceSymbolParams, WorkspaceSymbolResponse,
};
use std::path::PathBuf;

/// Responds to an LSP `workspace/symbol` request with the user-defined
/// symbols (types, type aliases, traits, trait aliases, global values,
/// trait instances) whose names match `params.query` (case-insensitive
/// substring). Standard-library and dependency symbols are excluded so
/// the picker stays useful in a real project.
pub(super) fn handle_workspace_symbol(
    id: u32,
    params: &WorkspaceSymbolParams,
    diag: &DiagnosticsResult,
) {
    let cdir = match get_current_dir() {
        Some(d) => d,
        None => {
            send_response(
                id,
                Ok::<_, ()>(WorkspaceSymbolResponse::Flat(Vec::new())),
            );
            return;
        }
    };

    let program = &diag.program;
    let query = params.query.to_lowercase();
    let mut symbols: Vec<SymbolInformation> = Vec::new();

    // A symbol is included only if its definition span lives in one of
    // the project's user source files.
    let in_user_file = |span: &Span| -> bool {
        let Ok(abs) = to_absolute_path(&span.input.file_path) else {
            return false;
        };
        diag.user_source_contents.contains_key(&abs)
    };

    // Type constructors.
    for (tycon, tycon_info) in program.type_env.tycons.as_ref() {
        if tycon.name.to_string().contains('#') {
            continue;
        }
        let Some(span) = tycon_info.source.as_ref() else {
            continue;
        };
        if !in_user_file(span) {
            continue;
        }
        let name = tycon.name.to_string();
        if !matches_query(&name, &query) {
            continue;
        }
        if let Some(sym) = make_symbol(name, tycon_kind(tycon_info), span, &cdir) {
            symbols.push(sym);
        }
    }

    // Type aliases.
    for (tycon, alias_info) in program.type_env.aliases.as_ref() {
        if tycon.name.to_string().contains('#') {
            continue;
        }
        let Some(span) = alias_info.source.as_ref() else {
            continue;
        };
        if !in_user_file(span) {
            continue;
        }
        let name = tycon.name.to_string();
        if !matches_query(&name, &query) {
            continue;
        }
        if let Some(sym) = make_symbol(name, SymbolKind::CLASS, span, &cdir) {
            symbols.push(sym);
        }
    }

    // Global values (functions, constants, trait members).
    for (full_name, global_value) in &program.global_values {
        if full_name.to_string().contains('#') {
            continue;
        }
        let Some(span) = global_value.decl_src.as_ref() else {
            continue;
        };
        if !in_user_file(span) {
            continue;
        }
        let name = full_name.to_string();
        if !matches_query(&name, &query) {
            continue;
        }
        if let Some(sym) = make_symbol(name, SymbolKind::FUNCTION, span, &cdir) {
            symbols.push(sym);
        }
    }

    // Trait definitions.
    for (trait_, trait_info) in &program.trait_env.traits {
        if trait_.name.to_string().contains('#') {
            continue;
        }
        let Some(span) = trait_info.source.as_ref() else {
            continue;
        };
        if !in_user_file(span) {
            continue;
        }
        let name = trait_.name.to_string();
        if !matches_query(&name, &query) {
            continue;
        }
        if let Some(sym) = make_symbol(name, SymbolKind::INTERFACE, span, &cdir) {
            symbols.push(sym);
        }
    }

    // Trait aliases.
    for (trait_, trait_alias) in &program.trait_env.aliases.data {
        if trait_.name.to_string().contains('#') {
            continue;
        }
        let Some(span) = trait_alias.source.as_ref() else {
            continue;
        };
        if !in_user_file(span) {
            continue;
        }
        let name = trait_.name.to_string();
        if !matches_query(&name, &query) {
            continue;
        }
        if let Some(sym) = make_symbol(name, SymbolKind::INTERFACE, span, &cdir) {
            symbols.push(sym);
        }
    }

    // Trait instances.
    for (trait_, instances) in &program.trait_env.impls {
        if trait_.name.to_string().contains('#') {
            continue;
        }
        for instance in instances {
            if !instance.is_user_defined {
                continue;
            }
            let Some(span) = instance.source.as_ref() else {
                continue;
            };
            if !in_user_file(span) {
                continue;
            }
            let name = format!("impl {}", instance.qual_pred.to_string());
            if !matches_query(&name, &query) {
                continue;
            }
            if let Some(sym) = make_symbol(name, SymbolKind::MODULE, span, &cdir) {
                symbols.push(sym);
            }
        }
    }

    send_response(id, Ok::<_, ()>(WorkspaceSymbolResponse::Flat(symbols)));
}

/// Case-insensitive substring match between `name` and `query_lower`
/// (which the caller has already lowercased). An empty query matches
/// everything. The LSP spec lets clients (e.g. VSCode) do their own
/// fuzzy filter on top, so the server's job is just to keep the result
/// set reasonable.
fn matches_query(name: &str, query_lower: &str) -> bool {
    if query_lower.is_empty() {
        return true;
    }
    name.to_lowercase().contains(query_lower)
}

/// Builds an LSP `SymbolInformation` for `name`/`kind` whose definition
/// lives at `span`. Returns `None` when the span cannot be converted to
/// a workspace location (e.g. its file is outside `cdir`).
#[allow(deprecated)]
fn make_symbol(
    name: String,
    kind: SymbolKind,
    span: &Span,
    cdir: &PathBuf,
) -> Option<SymbolInformation> {
    let location = span_to_location(span, cdir)?;
    Some(SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: Some(false),
        location,
        container_name: None,
    })
}

/// Maps a Fix type-constructor variant to the LSP `SymbolKind` shown in
/// the workspace symbol picker.
fn tycon_kind(tycon_info: &TyConInfo) -> SymbolKind {
    match tycon_info.variant {
        TyConVariant::Struct => SymbolKind::STRUCT,
        TyConVariant::Union => SymbolKind::ENUM,
        TyConVariant::Primitive
        | TyConVariant::Arrow
        | TyConVariant::Array
        | TyConVariant::DynamicObject
        | TyConVariant::Opaque => SymbolKind::CLASS,
    }
}
