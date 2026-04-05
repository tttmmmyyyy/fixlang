use super::edit_import;
use super::server::{send_response, LatestContent};
use crate::ast::name::{FullName, Name};
use crate::ast::program::Program;
use crate::ast::traits::{MissingTraitImplInfo, MissingTraitImplItem};
use crate::constants::{ERR_LACKING_TRAIT_IMPL, ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME};
use crate::misc::{generate_fresh_varnames, Map, Set};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionParams, NumberOrString, Position, Range, TextEdit, Uri,
    WorkspaceEdit,
};

// Handle "textDocument/codeAction" method.
pub(super) fn handle_code_action(
    id: u32,
    params: &CodeActionParams,
    program: Option<&Program>,
    uri_to_content: &mut Map<Uri, LatestContent>,
) {
    let mut actions: Vec<CodeAction> = vec![];
    for diag in &params.context.diagnostics {
        if diag.code == Some(NumberOrString::String(ERR_UNKNOWN_NAME.to_string()))
            || diag.code == Some(NumberOrString::String(ERR_NO_VALUE_MATCH.to_string()))
        {
            if let Some(program) = program {
                handle_unknown_name(diag, params, program, uri_to_content, &mut actions);
            }
        } else if diag.code == Some(NumberOrString::String(ERR_LACKING_TRAIT_IMPL.to_string())) {
            handle_lacking_trait_impl(diag, params, uri_to_content, &mut actions);
        }
    }
    send_response(id, Ok::<_, ()>(actions));
}

fn handle_unknown_name(
    diag: &lsp_types::Diagnostic,
    params: &CodeActionParams,
    program: &Program,
    uri_to_content: &mut Map<Uri, LatestContent>,
    actions: &mut Vec<CodeAction>,
) {
    // Extract the name from the diagnostic data.
    if diag.data.is_none() {
        return;
    }
    let name = serde_json::from_value::<String>(diag.data.as_ref().unwrap().clone());
    if name.is_err() {
        return;
    }
    let name = FullName::parse(name.unwrap().as_str());
    if name.is_none() {
        return;
    }
    let name = name.unwrap();
    let uri = &params.text_document.uri;
    let latest_content = uri_to_content.get_mut(uri);
    if latest_content.is_none() {
        return;
    }
    let latest_content = latest_content.unwrap();
    let mut available_names = vec![];
    for symbol in program.global_values.keys() {
        available_names.push(symbol.clone());
    }
    for tycon in program.type_env.tycons.keys() {
        available_names.push(tycon.name.clone());
    }
    for ty_alias in program.type_env.aliases.keys() {
        available_names.push(ty_alias.name.clone());
    }
    for trait_ in program.trait_env.traits.keys() {
        available_names.push(trait_.name.clone());
    }
    for trait_alias in program.trait_env.aliases.data.keys() {
        available_names.push(trait_alias.name.clone());
    }
    for (assoc_type, _) in program.trait_env.assoc_ty_kind_info() {
        available_names.push(assoc_type.name.clone());
    }
    available_names.sort();
    available_names.dedup();
    // Search for the symbol in the program's global values.
    for symbol in &available_names {
        if name.name != symbol.name {
            continue;
        }
        if !name.namespace.is_suffix_of(&symbol.namespace) {
            continue;
        }
        // Suggest importing this symbol.
        let edits = edit_import::create_text_edit_to_import(symbol, latest_content);
        let action = CodeAction {
            title: format!("Import `{}`", symbol.to_string()),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![diag.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert(uri.clone(), edits);
                    map
                }),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        };
        actions.push(action);
    }
}

fn handle_lacking_trait_impl(
    diag: &lsp_types::Diagnostic,
    params: &CodeActionParams,
    uri_to_content: &mut Map<Uri, LatestContent>,
    actions: &mut Vec<CodeAction>,
) {
    if diag.data.is_none() {
        return;
    }
    let info = MissingTraitImplInfo::from_json(diag.data.as_ref().unwrap());
    if info.is_none() {
        return;
    }
    let info = info.unwrap();

    let uri = &params.text_document.uri;
    let latest_content = uri_to_content.get(uri);
    if latest_content.is_none() {
        return;
    }
    let content = &latest_content.unwrap().content;

    // The diagnostic range covers the entire impl block (from `impl` to `}`).
    // We need to find the position of `}` and insert before it.
    let end_line = diag.range.end.line as usize;
    let end_char = diag.range.end.character as usize; // UTF-16 position after `}`

    let lines: Vec<&str> = content.lines().collect();
    if end_line >= lines.len() {
        return;
    }
    if end_char == 0 {
        return;
    }

    // Determine the indentation of the impl block (the line where `impl` starts).
    let start_line = diag.range.start.line as usize;
    let impl_indent = if start_line < lines.len() {
        let line = lines[start_line];
        line.len() - line.trim_start().len()
    } else {
        0
    };

    // Generate the stub text using the structured type.
    let insert_text = quickfix_stub_text(&info, impl_indent);
    if insert_text.is_empty() {
        return;
    }

    // Insert position: just before the `}` on end_line.
    let insert_pos = Position {
        line: end_line as u32,
        character: (end_char - 1) as u32,
    };
    let edit = TextEdit {
        range: Range {
            start: insert_pos,
            end: insert_pos,
        },
        new_text: insert_text,
    };

    let action = CodeAction {
        title: "Insert stub implementations".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some({
                let mut map = std::collections::HashMap::new();
                map.insert(uri.clone(), vec![edit]);
                map
            }),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    };
    actions.push(action);
}

// Generate the stub text to insert into the impl block.
// `impl_indent` is the number of spaces of the `impl` line.
fn quickfix_stub_text(info: &MissingTraitImplInfo, impl_indent: usize) -> String {
    let impl_type_str = info.impl_type.to_string();

    // Collect free variable names from impl_type to avoid conflicts when generating param names.
    let used_names: Set<Name> = info.impl_type.free_vars().into_keys().collect();

    let member_indent = " ".repeat(impl_indent + 4);
    let mut stub_lines: Vec<String> = vec![];

    // Associated types first, then members.
    for item in &info.items {
        match item {
            MissingTraitImplItem::AssocType(a) => {
                let mut line =
                    format!("{}type {} {}", member_indent, a.name.name, impl_type_str);
                let fresh_names = generate_fresh_varnames(a.num_extra_params, &used_names);
                for name in &fresh_names {
                    line += &format!(" {}", name);
                }
                line += " = ?;";
                stub_lines.push(line);
            }
            MissingTraitImplItem::Member(_) => {}
        }
    }
    for item in &info.items {
        match item {
            MissingTraitImplItem::Member(m) => {
                stub_lines.push(format!(
                    "{}{} : {} = ::Std::undefined(\"unimplemented\");",
                    member_indent, m.name.name, m.ty.to_string()
                ));
            }
            MissingTraitImplItem::AssocType(_) => {}
        }
    }

    if stub_lines.is_empty() {
        return String::new();
    }

    stub_lines.join("\n") + "\n"
}
