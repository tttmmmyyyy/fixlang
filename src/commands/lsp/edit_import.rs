use super::server::LatestContent;
use super::util::span_to_range;
use crate::ast::import::{is_accessible, ImportStatement};
use crate::ast::name::FullName;
use crate::ast::program::ModuleInfo;
use crate::constants::STD_NAME;
use crate::write_log;
use lsp_types::TextEdit;

/// The edits that erase each of `import_stmts` written in `content`, together with the blank
/// lines that follow it.
pub fn create_text_edits_to_erase_imports(
    content: &str,
    import_stmts: &[ImportStatement],
) -> Vec<TextEdit> {
    let content_lines = content.lines().collect::<Vec<_>>();
    let mut text_edits = vec![];

    for import_stmt in import_stmts {
        if import_stmt.source.is_none() {
            continue;
        }
        let mut range = span_to_range(&import_stmt.source.as_ref().unwrap());
        // If there are no whitespace characters and line breaks after range.end,
        // expand the range to remove whitespace characters and line breaks as well.
        loop {
            let end_line_content = content_lines.get(range.end.line as usize);
            if let Some(end_line_content) = end_line_content {
                // Convert UTF-16 code unit position to UTF-8 byte position
                let byte_pos = crate::misc::utf16_pos_to_utf8_byte_pos(
                    end_line_content,
                    range.end.character as usize,
                );
                let end_col_content = &end_line_content[byte_pos..];
                if end_col_content.trim().is_empty() {
                    range.end.line += 1;
                    range.end.character = 0;
                    continue;
                }
            }
            break;
        }

        text_edits.push(TextEdit {
            range,
            new_text: "".to_string(),
        });
    }

    text_edits
}

/// The edit that writes `new_import_stmts` after the `module` declaration, one statement a line.
pub fn create_text_edit_to_insert_imports(
    module_info: &ModuleInfo,
    new_import_stmts: &[ImportStatement],
) -> TextEdit {
    let inserted_text = if new_import_stmts.is_empty() {
        String::new()
    } else {
        let import_text = new_import_stmts
            .iter()
            .map(|stmt| stmt.stringify())
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\n{}", import_text)
    };

    let span = module_info.source.to_end_position();
    let range = span_to_range(&span);

    TextEdit {
        range,
        new_text: inserted_text,
    }
}

pub(super) fn create_text_edit_to_import(
    item_name: &FullName,
    latest_content: &mut LatestContent,
) -> Vec<TextEdit> {
    if !item_name.is_global() {
        return vec![];
    }
    let Some(mod_info) = latest_content.get_module_info().clone() else {
        return vec![];
    };
    let mod_name = mod_info.name.clone();
    // No need to import if the item is defined in the same module.
    if item_name.module() == mod_name {
        return vec![];
    }
    let Some(import_stmts) = latest_content.get_import_stmts().clone() else {
        return vec![];
    };

    // Check if the standard library is imported explicitly.
    let import_std_explicitly = import_stmts
        .iter()
        .any(|imp: &ImportStatement| &imp.module_name == STD_NAME && !imp.implicit);

    // If the item is already accessible, we don't need to import it.
    if !import_std_explicitly && item_name.module() == STD_NAME {
        return vec![];
    }
    if is_accessible(&import_stmts, &item_name) {
        return vec![];
    }

    // If any import_statement's source is None, it is abnormal, so return.
    if import_stmts.iter().any(|imp| imp.source.is_none()) {
        let msg =
            format!("In create_text_edit_to_import, found an import statement with None source.",);
        write_log!("{}", msg);
        return vec![];
    }

    // Generate text for new import statements.
    let mut new_import_stmts = import_stmts.clone();
    ImportStatement::add_import(&mut new_import_stmts, mod_name, item_name.clone());
    ImportStatement::sort(&mut new_import_stmts);

    let mut text_edits = vec![];

    // Erase all existing import statements.
    text_edits.extend(create_text_edits_to_erase_imports(
        &latest_content.content,
        &import_stmts,
    ));

    // Insert the import statement at the end of the module definition.
    text_edits.push(create_text_edit_to_insert_imports(
        &mod_info,
        &new_import_stmts,
    ));

    text_edits
}
