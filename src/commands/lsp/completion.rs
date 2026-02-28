// LSP completion feature handlers.

use super::edit_import::create_text_edit_to_import;
use super::server::{send_response, LatestContent};
use super::util::{document_from_endnode, get_line_string_from_position, parameters_of_global_value};
use crate::ast::name::{FullName, NameSpace};
use crate::ast::program::Program;
use crate::constants::chars_allowed_in_identifiers;
use crate::misc::Map;
use crate::write_log;
use crate::EndNode;
use crate::Var;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, Documentation,
    TextDocumentPositionParams, Uri,
};

// Handle "textDocument/completion" method.
pub(super) fn handle_completion(
    id: u32,
    params: &lsp_types::CompletionParams,
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
    text_document_position: &TextDocumentPositionParams,
    uri_to_content: &Map<Uri, LatestContent>,
) -> String {
    let current_line = get_line_string_from_position(uri_to_content, text_document_position);
    let typing_text = current_line
        .map(|(line, byte_pos)| line[..byte_pos].to_string())
        .unwrap_or_default();
    typing_text
}

// Handle "completionItem/resolve" method.
// Add documentation to the completion item.
pub(super) fn handle_completion_resolve_document(
    id: u32,
    params: &CompletionItem,
    uri_to_content: &mut Map<Uri, LatestContent>,
    program: &Program,
) {
    if params.data.is_none() {
        let msg = "In textDocument/completion, params.data is null.".to_string();
        write_log!("{}", msg);
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
        write_log!("{}", msg);
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
        EndNode::Module(_) => None,
        EndNode::TypeOrTrait(name) => Some(name),
        EndNode::ValueDecl(name) => Some(name), // Should not be used for completion, but just in case.
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
