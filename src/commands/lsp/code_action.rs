use super::edit_import;
use super::server::{send_response, LatestContent};
use super::util::{bytes_to_position, parameters_in_document, position_to_bytes};
use crate::ast::name::{FullName, Name};
use crate::ast::program::Program;
use crate::ast::traits::{MissingTraitImplInfo, MissingTraitImplItem};
use crate::ast::typedecl::describe_field_names;
use crate::ast::types::{type_assocty, type_tyvar_star, AssocType};
use crate::constants::{
    ERR_MISSING_STRUCT_FIELD, ERR_MISSING_TRAIT_IMPL, ERR_NO_VALUE_MATCH, ERR_UNKNOWN_NAME,
};
use crate::error::WARN_MISSING_PATTERN_FIELD;
use crate::misc::{generate_fresh_varnames, Map, Set};
use crate::parse::lexer::{lex_tokens, LexTokenKind};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionParams, Diagnostic, NumberOrString, Position, Range,
    TextEdit, Uri, WorkspaceEdit,
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
        } else if diag.code
            == Some(NumberOrString::String(
                WARN_MISSING_PATTERN_FIELD.to_string(),
            ))
        {
            handle_missing_pattern_field(diag, params, uri_to_content, &mut actions);
        }
    }
    send_response(id, Ok::<_, ()>(actions));
}

fn handle_unknown_name(
    diag: &Diagnostic,
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
    for tycon in program.type_env.tycons().keys() {
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
    diag: &Diagnostic,
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
/// its `range` covers the whole MakeStruct expression.
fn handle_missing_struct_field(
    diag: &Diagnostic,
    params: &CodeActionParams,
    uri_to_content: &mut Map<Uri, LatestContent>,
    actions: &mut Vec<CodeAction>,
) {
    let Some(missing) = missing_fields_of(diag) else {
        return;
    };
    let uri = &params.text_document.uri;
    let Some(latest_content) = uri_to_content.get(uri) else {
        return;
    };
    let items = missing
        .iter()
        .map(|name| format!("{}: ?", name))
        .collect::<Vec<_>>();
    let Some(edits) = insert_fields_edits(&latest_content.content, &diag.range, &items) else {
        return;
    };
    actions.push(quick_fix(
        missing_fields_fix_title(&missing),
        diag,
        uri,
        edits,
        true,
    ));
}

/// Offer two quick fixes for a struct pattern that leaves out fields without `_`: one writes each
/// missing field as `name: _`, which matches it and binds nothing, and the other writes `_` after
/// the fields to leave out the rest.
///
/// A missing field is written with `_` rather than by its name alone, since `name` would bind a
/// variable that can hide one of the same name the pattern's scope uses.
///
/// The diagnostic's `data` carries a JSON array of missing field names, and
/// its `range` covers the whole struct pattern.
fn handle_missing_pattern_field(
    diag: &Diagnostic,
    params: &CodeActionParams,
    uri_to_content: &mut Map<Uri, LatestContent>,
    actions: &mut Vec<CodeAction>,
) {
    let Some(missing) = missing_fields_of(diag) else {
        return;
    };
    let uri = &params.text_document.uri;
    let Some(latest_content) = uri_to_content.get(uri) else {
        return;
    };
    let content = &latest_content.content;

    let items = missing
        .iter()
        .map(|name| format!("{}: _", name))
        .collect::<Vec<_>>();
    if let Some(edits) = insert_fields_edits(content, &diag.range, &items) {
        actions.push(quick_fix(
            missing_fields_fix_title(&missing),
            diag,
            uri,
            edits,
            true,
        ));
    }
    if let Some(edits) = insert_fields_edits(content, &diag.range, &["_".to_string()]) {
        let title = "Leave out the other fields with `_`".to_string();
        actions.push(quick_fix(title, diag, uri, edits, false));
    }
}

/// The names of the missing fields a diagnostic carries in its `data`, when it carries any.
fn missing_fields_of(diag: &Diagnostic) -> Option<Vec<String>> {
    let missing: Vec<String> = serde_json::from_value(diag.data.clone()?).ok()?;
    if missing.is_empty() {
        None
    } else {
        Some(missing)
    }
}

/// The title of a quick fix that adds the fields `missing`.
fn missing_fields_fix_title(missing: &[Name]) -> String {
    format!("Add missing {}", describe_field_names(missing))
}

/// A quick fix titled `title` for `diag` that applies `edits` to the document at `uri`.
fn quick_fix(
    title: String,
    diag: &Diagnostic,
    uri: &Uri,
    edits: Vec<TextEdit>,
    is_preferred: bool,
) -> CodeAction {
    CodeAction {
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
        is_preferred: Some(is_preferred),
        disabled: None,
        data: None,
    }
}

/// The edits that append `items` to the fields of the struct literal or struct pattern that
/// `range` covers, from its head to its closing `}`.
///
/// On one line, the items go after the last field, joined by `, `. When the `}` stands on a line
/// of its own, each item goes on its own line, indented like the fields and ending in a comma, and
/// a comma is added after the last field when it has none. Comments between the fields and the
/// `}` are left where they are.
///
/// # Examples
/// For `S { x }` and the items `["y: _"]`, the edit inserts `, y: _` after `x`.
fn insert_fields_edits(content: &str, range: &Range, items: &[String]) -> Option<Vec<TextEdit>> {
    let start = position_to_bytes(content, range.start);
    let end = position_to_bytes(content, range.end);
    if end <= start || !content[..end].ends_with('}') {
        return None;
    }
    let brace = end - 1;
    let last_code_char = last_code_char_before(content, start, brace);
    let insert_at = |offset: usize, new_text: String| {
        let pos = bytes_to_position(content, offset);
        TextEdit {
            range: Range {
                start: pos,
                end: pos,
            },
            new_text,
        }
    };

    let brace_line_start = content[..brace].rfind('\n').map_or(0, |i| i + 1);
    let is_multiline =
        brace_line_start > start && content[brace_line_start..brace].trim().is_empty();
    if !is_multiline {
        // Insert right after the last character of code, so that any whitespace before `}` (the
        // space in `S { x }`) becomes the padding after the new fields. The separator matches
        // what precedes the insertion.
        let (offset, prefix) = match last_code_char {
            Some((offset, '{')) => (offset, ""),
            Some((offset, ',')) => (offset, " "),
            Some((offset, _)) => (offset, ", "),
            None => (brace, ""),
        };
        return Some(vec![insert_at(
            offset,
            format!("{}{}", prefix, items.join(", ")),
        )]);
    }

    // Each new field goes on its own line before the `}` line, so that the indent of `}` stays.
    // When the last field has no trailing comma, one is added as a separate edit.
    let mut edits = vec![];
    if let Some((offset, c)) = last_code_char {
        if c != '{' && c != ',' {
            edits.push(insert_at(offset, ",".to_string()));
        }
    }
    let indent = field_indent(content, start, brace_line_start, brace);
    let body = items
        .iter()
        .map(|item| format!("{}{},\n", indent, item))
        .collect::<String>();
    edits.push(insert_at(brace_line_start, body));
    Some(edits)
}

/// The last character of code in `content[start..end]` that is neither whitespace nor part of a
/// comment, with the byte offset just past it.
fn last_code_char_before(content: &str, start: usize, end: usize) -> Option<(usize, char)> {
    let text = &content[start..end];
    let comments: Vec<(usize, usize)> = lex_tokens(text)
        .into_iter()
        .filter(|token| token.kind == LexTokenKind::Comment)
        .map(|token| (token.start, token.end))
        .collect();
    text.char_indices()
        .rev()
        .find(|&(i, c)| {
            !c.is_whitespace() && !comments.iter().any(|&(from, to)| from <= i && i < to)
        })
        .map(|(i, c)| (start + i + c.len_utf8(), c))
}

/// The indent to give a field inserted on a line of its own: that of the first non-blank line
/// after the head of the literal or pattern, or, when the fields share the head's line, the
/// indent of the `}` line plus four spaces.
///
/// # Arguments
/// * `start` — the byte offset of the head.
/// * `brace_line_start` — the byte offset of the start of the line holding the closing `}`.
/// * `brace` — the byte offset of the closing `}`.
fn field_indent(content: &str, start: usize, brace_line_start: usize, brace: usize) -> String {
    let leading_whitespace = |line: &str| line[..line.len() - line.trim_start().len()].to_string();
    let field_lines = content[start..brace_line_start].lines().skip(1);
    for line in field_lines {
        if !line.trim().is_empty() {
            return leading_whitespace(line);
        }
    }
    format!(
        "{}    ",
        leading_whitespace(&content[brace_line_start..brace])
    )
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
                // Write the parameters the member's document lists as `|x, y| ?`.
                let params = m
                    .document
                    .as_deref()
                    .and_then(parameters_in_document)
                    .unwrap_or_default();
                let lambda_head = if params.is_empty() {
                    String::new()
                } else {
                    format!("|{}| ", params.join(", "))
                };
                stub_lines.push(format!(
                    "{}{} : {} = {}?;",
                    member_indent,
                    m.name.name,
                    m.ty.to_string(),
                    lambda_head
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
