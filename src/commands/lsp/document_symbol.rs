// LSP "textDocument/documentSymbol" handler.

use super::server::send_response;
use super::util::{span_to_range, uri_to_path};
use crate::ast::name::FullName;
use crate::ast::program::{GlobalValue, Program};
use crate::ast::traits::{TraitAlias, TraitDefn, TraitId, TraitImpl};
use crate::ast::types::{TyAliasInfo, TyCon, TyConInfo, TyConVariant};
use crate::misc::to_absolute_path;
use crate::write_log;
use lsp_types::{DocumentSymbol, DocumentSymbolParams, SymbolKind};

// Handle "textDocument/documentSymbol" method.
pub(super) fn handle_document_symbol(id: u32, params: &DocumentSymbolParams, program: &Program) {
    let canonicalize_path = |path| {
        let path = to_absolute_path(path);
        if let Err(e) = path {
            let msg = e.to_string();
            write_log!("{}", msg);
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
        if let Some(span) = &global_value.decl_src {
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
    for (trait_, trait_alias) in &program.trait_env.aliases.data {
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
    for (trait_, instances) in &program.trait_env.impls {
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
        .decl_src
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
fn create_symbol_from_trait_info(trait_: &TraitId, trait_info: &TraitDefn) -> DocumentSymbol {
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
    trait_: &TraitId,
    trait_instance: &TraitImpl,
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
