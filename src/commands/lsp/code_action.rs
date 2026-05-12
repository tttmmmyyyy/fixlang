use super::edit_import;
use super::server::{send_response, LatestContent};
use crate::ast::name::{FullName, Name};
use crate::ast::program::Program;
use crate::ast::traits::{MissingTraitImplInfo, MissingTraitImplItem};
use crate::ast::types::{type_assocty, type_tyvar_star, AssocType};
use crate::constants::{
    ERR_MISSING_STRUCT_FIELD, ERR_MISSING_TRAIT_IMPL, ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME,
};
use crate::misc::{generate_fresh_varnames, Map, Set};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionParams, NumberOrString, Position, Range, TextEdit, Uri,
    WorkspaceEdit,
};
use std::collections::HashMap;

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
        } else if diag.code == Some(NumberOrString::String(ERR_MISSING_TRAIT_IMPL.to_string())) {
            handle_missing_trait_impl(diag, params, uri_to_content, &mut actions);
        } else if diag.code == Some(NumberOrString::String(ERR_MISSING_STRUCT_FIELD.to_string())) {
            handle_missing_struct_field(diag, params, uri_to_content, &mut actions);
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
                    let mut map = HashMap::new();
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

fn handle_missing_trait_impl(
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
                let mut map = HashMap::new();
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

/// Offer a quick fix that inserts `name: ?` placeholders for each missing
/// field of a struct literal (e.g. `Vector3 { x: 1.0, y: 2.0 }` missing `z`).
///
/// The diagnostic's `data` carries a JSON array of missing field names, and
/// its `range` covers the whole MakeStruct expression — so `range.end` sits
/// just past the closing `}`. Field/expression names in Fix are ASCII, so
/// LSP UTF-16 columns coincide with char/byte columns at the positions we
/// care about.
fn handle_missing_struct_field(
    diag: &lsp_types::Diagnostic,
    params: &CodeActionParams,
    uri_to_content: &mut Map<Uri, LatestContent>,
    actions: &mut Vec<CodeAction>,
) {
    if diag.data.is_none() {
        return;
    }
    let missing: Vec<String> =
        match serde_json::from_value(diag.data.as_ref().unwrap().clone()) {
            Ok(v) => v,
            Err(_) => return,
        };
    if missing.is_empty() {
        return;
    }

    let uri = &params.text_document.uri;
    let latest_content = uri_to_content.get(uri);
    if latest_content.is_none() {
        return;
    }
    let content = &latest_content.unwrap().content;
    let lines: Vec<&str> = content.lines().collect();

    let start_line = diag.range.start.line as usize;
    let end_line = diag.range.end.line as usize;
    let end_char = diag.range.end.character as usize;
    if end_line >= lines.len() || end_char == 0 {
        return;
    }

    let end_line_chars: Vec<char> = lines[end_line].chars().collect();
    let brace_col = end_char - 1;
    if brace_col > end_line_chars.len() {
        return;
    }
    let before_brace: String = end_line_chars[..brace_col].iter().collect();
    let is_multiline = before_brace.trim().is_empty() && start_line != end_line;

    let last_nonws = find_last_nonws_before(&lines, start_line, end_line, brace_col);

    let mut edits: Vec<TextEdit> = vec![];

    if !is_multiline {
        // Single line: insert right after the previous non-whitespace
        // character so any trailing whitespace before `}` (e.g. the space in
        // `Vector3 { x: 1.0 }`) becomes the natural padding after the new
        // field. Prefix with a separator that matches what precedes us.
        let (insert_line, insert_col, prefix) = match last_nonws {
            Some((line, col_after, '{')) => (line, col_after, ""),
            Some((line, col_after, ',')) => (line, col_after, " "),
            Some((line, col_after, _)) => (line, col_after, ", "),
            None => (end_line, brace_col, ""),
        };
        let fields_str = missing
            .iter()
            .map(|n| format!("{}: ?", n))
            .collect::<Vec<_>>()
            .join(", ");
        let new_text = format!("{}{}", prefix, fields_str);
        let pos = Position {
            line: insert_line as u32,
            character: insert_col as u32,
        };
        edits.push(TextEdit {
            range: Range { start: pos, end: pos },
            new_text,
        });
    } else {
        // Multi-line: emit each new field on its own indented line ending in
        // a trailing comma. If the previous field has no trailing comma, add
        // one as a separate, non-overlapping edit.
        let field_indent = compute_field_indent(&lines, start_line, end_line);
        let mut body = String::new();
        for name in &missing {
            body.push_str(&field_indent);
            body.push_str(&format!("{}: ?,\n", name));
        }

        let need_trailing_comma =
            matches!(last_nonws, Some((_, _, c)) if c != '{' && c != ',');
        if need_trailing_comma {
            if let Some((lline, lcol, _)) = last_nonws {
                let pos = Position {
                    line: lline as u32,
                    character: lcol as u32,
                };
                edits.push(TextEdit {
                    range: Range { start: pos, end: pos },
                    new_text: ",".to_string(),
                });
            }
        }

        // Insert at column 0 of the `}` line so that the existing indent of
        // `}` is preserved after the inserted lines.
        let pos = Position {
            line: end_line as u32,
            character: 0,
        };
        edits.push(TextEdit {
            range: Range { start: pos, end: pos },
            new_text: body,
        });
    }

    let title = if missing.len() == 1 {
        format!("Add missing field `{}`", missing[0])
    } else {
        let list = missing
            .iter()
            .map(|n| format!("`{}`", n))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Add missing fields {}", list)
    };

    let action = CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some({
                let mut map = HashMap::new();
                map.insert(uri.clone(), edits);
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

/// Find the last non-whitespace char in `lines` that sits strictly before
/// `(end_line, end_col)`. Returns `(line, col_after_char, char)` where
/// `col_after_char` is the column immediately following the character (a
/// natural insertion point for, e.g., a trailing comma).
fn find_last_nonws_before(
    lines: &[&str],
    start_line: usize,
    end_line: usize,
    end_col: usize,
) -> Option<(usize, usize, char)> {
    for line_idx in (start_line..=end_line).rev() {
        let chars: Vec<char> = lines[line_idx].chars().collect();
        let limit = if line_idx == end_line {
            end_col.min(chars.len())
        } else {
            chars.len()
        };
        for col in (0..limit).rev() {
            let c = chars[col];
            if !c.is_whitespace() {
                return Some((line_idx, col + 1, c));
            }
        }
    }
    None
}

/// Choose the column prefix to use for inserted field lines. Prefer the
/// indent of an existing field line; if there is none, fall back to the
/// indent of the `}` line plus four spaces.
fn compute_field_indent(lines: &[&str], start_line: usize, end_line: usize) -> String {
    for line_idx in (start_line + 1)..end_line {
        let l = lines[line_idx];
        if !l.trim().is_empty() {
            return l[..l.len() - l.trim_start().len()].to_string();
        }
    }
    let l = lines[end_line];
    let base = l.len() - l.trim_start().len();
    " ".repeat(base + 4)
}

// Generate the stub text to insert into the impl block.
// `impl_indent` is the number of spaces of the `impl` line.
fn quickfix_stub_text(info: &MissingTraitImplInfo, impl_indent: usize) -> String {
    // Collect free variable names from impl_type to avoid conflicts when generating param names.
    let used_names: Set<Name> = info.impl_type.free_vars().into_keys().collect();

    let member_indent = " ".repeat(impl_indent + 4);
    let mut stub_lines: Vec<String> = vec![];

    // Associated types first, then members.
    for item in &info.items {
        match item {
            MissingTraitImplItem::AssocType(a) => {
                let fresh_names = generate_fresh_varnames(a.num_extra_params, &used_names);
                let mut args = vec![info.impl_type.clone()];
                for name in &fresh_names {
                    args.push(type_tyvar_star(name));
                }
                let assoc_ty = AssocType {
                    name: FullName::local(&a.name.name),
                    src: None,
                };
                let assoc_type_node = type_assocty(assoc_ty, args);
                stub_lines.push(format!(
                    "{}type {} = ?;",
                    member_indent,
                    assoc_type_node.to_string()
                ));
            }
            MissingTraitImplItem::Member(_) => {}
        }
    }
    for item in &info.items {
        match item {
            MissingTraitImplItem::Member(m) => {
                stub_lines.push(format!(
                    "{}{} : {} = ?;",
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
