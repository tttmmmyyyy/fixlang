use super::edit_import;
use super::server::{send_response, LatestContent};
use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::constants::{ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME};
use crate::misc::Map;
use lsp_types::{CodeAction, CodeActionKind, CodeActionParams, NumberOrString, Uri, WorkspaceEdit};

// Handle "textDocument/codeAction" method.
pub(super) fn handle_code_action(
    id: u32,
    params: &CodeActionParams,
    program: &Program,
    uri_to_content: &mut Map<Uri, LatestContent>,
) {
    let mut actions: Vec<CodeAction> = vec![];
    for diag in &params.context.diagnostics {
        if diag.code == Some(NumberOrString::String(ERR_UNKNOWN_NAME.to_string()))
            || diag.code == Some(NumberOrString::String(ERR_NO_VALUE_MATCH.to_string()))
        {
            // Extract the name from the diagnostic data.
            if diag.data.is_none() {
                continue;
            }
            let name = serde_json::from_value::<String>(diag.data.as_ref().unwrap().clone());
            if name.is_err() {
                continue;
            }
            let name = FullName::parse(name.unwrap().as_str());
            if name.is_none() {
                continue;
            }
            let name = name.unwrap();
            let uri = &params.text_document.uri;
            let latest_content = uri_to_content.get_mut(uri);
            if latest_content.is_none() {
                continue;
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
    }
    send_response(id, Ok::<_, ()>(actions));
}
