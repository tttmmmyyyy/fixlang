// Utility functions shared across LSP feature modules.

use super::server::LatestContent;
use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::ast::traits::TraitId;
use crate::ast::types::TyCon;
use crate::commands::docs::MarkdownSection;
use crate::constants::chars_allowed_in_identifiers;
use crate::misc::{to_absolute_path, Map};
use crate::write_log;
use crate::EndNode;
use crate::SourceFile;
use crate::SourcePos;
use crate::Span;
use difference::diff;
use lsp_types::MarkupContent;
use lsp_types::TextDocumentPositionParams;
use std::path::PathBuf;
use std::str::FromStr;

// Convert a `lsp_types::Uri` into a `PathBuf`.
pub(super) fn uri_to_path(uri: &lsp_types::Uri) -> PathBuf {
    PathBuf::from(
        urlencoding::decode(&uri.path().to_string())
            .ok()
            .unwrap()
            .as_ref(),
    )
}

// Given two versions of a file content, find the line in `content1` that corresponds to
// `line0` in `content0`.
fn calculate_corresponding_line(content0: &str, content1: &str, line0: u32) -> Option<u32> {
    let (_, diffs) = diff(content0, content1, "\n");
    let mut line_cnt_0 = -1;
    let mut line_cnt_1 = -1;
    for diff in diffs {
        match diff {
            difference::Difference::Same(s) => {
                let lines = s.split("\n").count();
                for _ in 0..lines {
                    line_cnt_0 += 1;
                    line_cnt_1 += 1;
                    if line_cnt_0 == line0 as i32 {
                        return Some(line_cnt_1 as u32);
                    }
                }
            }
            difference::Difference::Add(s) => {
                line_cnt_1 += s.split("\n").count() as i32;
            }
            difference::Difference::Rem(s) => {
                line_cnt_0 += s.split("\n").count() as i32;
            }
        }
    }
    None
}

// Convert a `lsp_types::Position` into a byte offset in a string.
fn position_to_bytes(string: &str, position: lsp_types::Position) -> usize {
    let mut bytes = 0;
    let mut line = 0;
    let mut utf16_count = 0;

    for c in string.chars() {
        if line == position.line && utf16_count >= position.character as usize {
            break;
        }
        bytes += c.len_utf8();
        if c == '\n' {
            line += 1;
            utf16_count = 0;
        } else {
            utf16_count += c.len_utf16();
        }
    }
    bytes
}

// Resolve the AST node located at the cursor position given by `text_position`.
pub(super) fn get_node_at(
    text_position: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) -> Option<EndNode> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log!("{}", msg);
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();

    // Get the path of the file.
    let path = uri_to_path(uri);

    // Get the file content at the time of the last successful diagnostics.
    let saved_content = super::server::get_file_content_at_previous_diagnostics(program, &path);
    if let Err(e) = saved_content {
        write_log!("{}", e);
        return None;
    }
    let saved_content = saved_content.ok().unwrap();

    // Get the position of the cursor in `saved_content`.
    let pos_in_latest = text_position.position;
    let line_in_saved =
        calculate_corresponding_line(&latest_content.content, &saved_content, pos_in_latest.line);
    if line_in_saved.is_none() {
        return None;
    }
    let pos_in_saved = lsp_types::Position {
        line: line_in_saved.unwrap(),
        character: pos_in_latest.character,
    };

    // Get the node at the position.
    let pos = SourcePos {
        input: SourceFile::from_file_path(path),
        pos: position_to_bytes(&saved_content, pos_in_saved),
    };
    program.find_node_at(&pos)
}

// Get the current directory, logging an error and returning None if it fails.
pub(super) fn get_current_dir() -> Option<PathBuf> {
    match std::env::current_dir() {
        Ok(d) => Some(d),
        Err(e) => {
            write_log!("Failed to get the current directory: {:?}", e);
            None
        }
    }
}

// Convert a `Span` into an LSP `Range`.
pub(super) fn span_to_range(span: &Span) -> lsp_types::Range {
    fn pair_to_zero_indexed((x, y): (usize, usize)) -> (usize, usize) {
        (x - 1, y - 1)
    }

    let (start_line, start_column) = pair_to_zero_indexed(span.start_line_col());
    let (end_line, end_column) = pair_to_zero_indexed(span.end_line_col());

    // Convert character-based column positions to UTF-16 code unit positions
    let source_string = span.input.string();
    let (start_utf16_col, end_utf16_col) = if let Ok(source_string) = source_string {
        let start_utf16 =
            crate::misc::char_pos_to_utf16_pos(&source_string, start_line, start_column);
        let end_utf16 = crate::misc::char_pos_to_utf16_pos(&source_string, end_line, end_column);
        (start_utf16, end_utf16)
    } else {
        (start_column, end_column)
    };

    lsp_types::Range {
        start: lsp_types::Position {
            line: start_line as u32,
            character: start_utf16_col as u32,
        },
        end: lsp_types::Position {
            line: end_line as u32,
            character: end_utf16_col as u32,
        },
    }
}

// Convert a `Span` into an `lsp_types::Location` using `cdir` as the base directory.
// Returns `None` if the path cannot be converted to a URI.
pub(super) fn span_to_location(span: &Span, cdir: &PathBuf) -> Option<lsp_types::Location> {
    let uri = path_to_uri(&cdir.join(&span.input.file_path));
    match uri {
        Ok(uri) => Some(lsp_types::Location {
            uri,
            range: span_to_range(span),
        }),
        Err(e) => {
            write_log!("Failed to convert path to URI: {}", e);
            None
        }
    }
}

// Convert a collection of `Span`s into `lsp_types::Location`s using `cdir` as the base directory.
// Spans that cannot be converted are silently dropped (with a log message).
pub(super) fn spans_to_locations(spans: Vec<Span>, cdir: &PathBuf) -> Vec<lsp_types::Location> {
    spans
        .into_iter()
        .filter_map(|span| span_to_location(&span, cdir))
        .collect()
}

// Convert a file path into an LSP URI.
pub(super) fn path_to_uri(path: &PathBuf) -> Result<lsp_types::Uri, String> {
    // URI-encode each component of the path.
    let path = to_absolute_path(path).map_err(|_| {
        format!(
            "Failed to get the absolute path of the file: \"{}\"",
            path.to_string_lossy().to_string()
        )
    })?;
    let mut components = vec![];
    for comp in path.components() {
        match comp {
            std::path::Component::Normal(comp) => {
                let comp = comp.to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                let comp = urlencoding::encode(comp.unwrap()).to_string();
                components.push(comp);
            }
            std::path::Component::Prefix(prefix) => {
                let comp = prefix.as_os_str().to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                components.push(comp.unwrap().to_string());
            }
            std::path::Component::RootDir => {}
            std::path::Component::CurDir => unreachable!(),
            std::path::Component::ParentDir => unreachable!(),
        }
    }
    let path = "file:///".to_string() + components.join("/").as_str();
    let uri = lsp_types::Uri::from_str(&path);
    if uri.is_err() {
        return Err(format!("Failed to convert a path into Uri: {:?}", path));
    }
    Ok(uri.unwrap())
}

// Given a `TextDocumentPositionParams`, get the line string and the byte position in that line.
// `uri_to_content` is a map to get the source string from the uri of the source file.
// The returned byte position is converted from the UTF-16 code unit position in the text_position.
pub(super) fn get_line_string_from_position(
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
    text_position: &TextDocumentPositionParams,
) -> Option<(String, usize)> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log!("{}", msg);
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();
    let latest_content = &latest_content.content;

    // Get the string at line `line` in `latest_content`.
    let line = text_position.position.line;
    let line = line as usize;
    let line_str = latest_content.lines().nth(line).unwrap_or("").to_string();

    // Convert UTF-16 code unit position to byte position
    let byte_pos = crate::misc::utf16_pos_to_utf8_byte_pos(
        &line_str,
        text_position.position.character as usize,
    );

    Some((line_str, byte_pos))
}

// Get the parameters of a global value from its documentation.
pub(super) fn parameters_of_global_value(full_name: &FullName, program: &Program) -> Option<Vec<String>> {
    // Get the document of the global value, which is a markdown string.
    let opt_gv = program.global_values.get(full_name);
    if opt_gv.is_none() {
        return None;
    }
    let gv = opt_gv.unwrap();
    let opt_docs = gv.get_document();
    if opt_docs.is_none() {
        return None;
    }
    let docs = opt_docs.unwrap();
    let sections = MarkdownSection::parse_many(docs.lines().collect());

    // Find the first top-level or second-level section named "Parameters".
    // let param_section = sections.iter().find(|sec| sec.title.trim() == "Parameters");
    let param_section = sections.iter().find_map(|sec| {
        if sec.title.trim() == "Parameters" {
            Some(sec)
        } else {
            sec.subsections
                .iter()
                .find(|subsec| subsec.title.trim() == "Parameters")
        }
    });
    if param_section.is_none() {
        return None;
    }
    let param_section = param_section.unwrap();

    // Collect the top-level list items.
    let mut params = vec![];
    for paragraph in &param_section.paragraphs {
        for line in paragraph.lines() {
            if line.starts_with("- ") || line.starts_with("* ") {
                // Find the first backquoted sequence of characters.
                let mut quoted_str = String::new();
                let mut in_backquote = false;
                for c in line.chars() {
                    if c == '`' {
                        if in_backquote {
                            in_backquote = false;
                            break;
                        } else {
                            in_backquote = true;
                            continue;
                        }
                    } else if in_backquote {
                        quoted_str.push(c);
                    }
                }
                // If the line ends in backquote, then skip it.
                if in_backquote {
                    continue;
                }

                // Find the first continuous sequence of characters that are allowed in identifiers.
                let name_chars = chars_allowed_in_identifiers();
                let mut param = String::new();
                for c in quoted_str.chars() {
                    if name_chars.contains(c) {
                        param.push(c);
                    } else if !param.is_empty() {
                        break;
                    }
                }

                // If the parameter is empty, skip it.
                if param.is_empty() {
                    continue;
                }

                params.push(param.to_string());
            }
        }
    }

    Some(params)
}

pub(super) fn find_trait_or_alias_def_src(program: &Program, trait_: TraitId) -> Option<Span> {
    let mut def_src = program
        .trait_env
        .traits
        .get(&trait_)
        .and_then(|ti| ti.source.clone());
    if def_src.is_none() {
        def_src = program
            .trait_env
            .aliases
            .data
            .get(&trait_)
            .and_then(|ta| ta.source.clone());
    }
    def_src
}

// Find the source location where the type constructor is defined.
pub(super) fn find_tycon_def_src(program: &Program, tycon: TyCon) -> Option<Span> {
    program
        .type_env
        .tycons
        .get(&tycon)
        .and_then(|ti| ti.source.clone())
}

pub(super) fn document_from_endnode(node: &EndNode, program: &Program) -> MarkupContent {
    fn document_tycon_or_alias(program: &Program, docs: &mut String, tycon: &TyCon) {
        *docs += &format!("```\n{}\n```", tycon.to_string());
        if let Some(ti) = program.type_env.tycons.get(&tycon) {
            if let Some(document) = ti.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        } else if let Some(ta) = program.type_env.aliases.get(&tycon) {
            if let Some(document) = ta.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        }
    }

    fn document_trait_or_alias(program: &Program, docs: &mut String, trait_id: &TraitId) {
        *docs += &format!("```\n{}\n```", trait_id.to_string());
        if let Some(ti) = program.trait_env.traits.get(&trait_id) {
            if let Some(document) = ti.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        } else if let Some(ta) = program.trait_env.aliases.data.get(&trait_id) {
            if let Some(document) = ta.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        }
    }

    // Create a hover message.
    let mut docs = String::new();
    match node {
        EndNode::Expr(var, ty) => {
            // Get informations of the variable which are needed to show in the hover.
            let full_name = &var.name;

            if full_name.is_local() {
                // In case the variable is local, show the name and type of the variable.
                if let Some(ty) = ty.as_ref() {
                    docs += &format!(
                        "```\n{} : {}\n```",
                        full_name.to_string(),
                        ty.to_string_normalize()
                    );
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
            } else {
                // In case the variable is global, show the documentation of the global value.
                let mut scm_string = String::new();
                if let Some(gv) = program.global_values.get(full_name) {
                    scm_string = gv
                        .syn_scm
                        .clone()
                        .unwrap_or(gv.scm.clone())
                        .to_string_normalize();
                    docs += &format!("```\n{} : {}\n```", full_name.to_string(), scm_string);
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
                if let Some(ty) = ty.as_ref() {
                    let ty_string = ty.to_string_normalize();
                    if scm_string != ty_string {
                        docs += &format!("\nInstantiated as:\n```\n{}\n```", ty_string);
                    }
                }
                if let Some(gv) = program.global_values.get(full_name) {
                    if let Some(document) = gv.get_document() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            };
        }
        EndNode::Pattern(var, ty) => {
            // In case the node is a variable, show the name and type of the variable.
            if let Some(ty) = ty.as_ref() {
                docs += &format!(
                    "```\n{} : {}\n```",
                    var.name.to_string(),
                    ty.to_string_normalize()
                );
            } else {
                docs += &format!("```\n{}\n```", var.name.to_string());
            }
        }
        EndNode::Type(tycon) => {
            document_tycon_or_alias(program, &mut docs, tycon);
        }
        EndNode::Trait(trait_id) => {
            document_trait_or_alias(program, &mut docs, trait_id);
        }
        EndNode::Module(mod_name) => {
            docs += &format!("```\nmodule {}\n```", mod_name.to_string());
            if let Some(mi) = program.modules.iter().find(|mi| &mi.name == mod_name) {
                if let Some(document) = mi.source.get_document().ok() {
                    if !document.trim().is_empty() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            }
        }
        EndNode::TypeOrTrait(name) => {
            let tycon = TyCon { name: name.clone() };
            let trait_ = TraitId::from_fullname(name.clone());
            if program.type_env.tycons.contains_key(&tycon) {
                document_tycon_or_alias(program, &mut docs, &tycon);
            } else if program.trait_env.traits.contains_key(&trait_) {
                document_trait_or_alias(program, &mut docs, &trait_);
            }
        }
        EndNode::AssocType(assoc_type) => {
            let trait_id = assoc_type.trait_id();
            docs += &format!(
                "```\nassociated type {} (trait {})\n```",
                assoc_type.name.to_string(),
                trait_id.to_string()
            );
            // Try to show the documentation comment of the associated type definition.
            if let Some(ti) = program.trait_env.traits.get(&trait_id) {
                if let Some(atd) = ti.assoc_types.get(&assoc_type.name.name) {
                    if let Some(doc) = atd.src.as_ref().and_then(|src| src.get_document().ok()) {
                        if !doc.is_empty() {
                            docs += &format!("\n\n{}", doc);
                        }
                    }
                }
            }
        }
        EndNode::ValueDecl(name) => {
            // Show the type signature and documentation of the declared global value.
            if let Some(gv) = program.global_values.get(name) {
                let scm_string = gv
                    .syn_scm
                    .clone()
                    .unwrap_or(gv.scm.clone())
                    .to_string_normalize();
                docs += &format!("```\n{} : {}\n```", name.to_string(), scm_string);
                if let Some(document) = gv.get_document() {
                    docs += &format!("\n\n{}", document);
                }
            } else {
                docs += &format!("```\n{}\n```", name.to_string());
            }
        }
    }
    let content = MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: docs,
    };
    content
}
