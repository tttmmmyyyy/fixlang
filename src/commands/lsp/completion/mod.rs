// LSP completion feature handlers.

mod index;
mod repair;
mod score;

use super::edit_import::create_text_edit_to_import;
use super::server::{send_response, LatestContent};
use super::util::{
    document_from_endnode, get_line_string_from_position, parameters_of_global_value,
    position_to_bytes,
};
use crate::ast::expr::{hole_full_name, Expr, ExprNode, Var};
use crate::ast::name::{FullName, NameSpace};
use crate::ast::pattern::PatternNode;
use crate::ast::program::{EndNode, Program, SymbolExpr};
use crate::ast::types::TypeNode;
use crate::configuration::{Configuration, DiagnosticsConfig};
use crate::constants::chars_allowed_in_identifiers;
use crate::dependency::lockfile::LockFileType;
use crate::elaboration::elaborate_via_config;
use crate::metafiles::project_file::ProjectFile;
use crate::misc::{to_absolute_path, Map};
use crate::parse::sourcefile::Span;
use crate::write_log;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
    CompletionParams, Documentation, InsertTextFormat, TextDocumentPositionParams, Uri,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Handles the `textDocument/completion` LSP request: collects
/// candidate symbols (globals, type constructors, traits, associated
/// types) visible at the cursor and replies with a list of
/// `CompletionItem`s.
pub(super) fn handle_completion(
    id: u32,
    params: &CompletionParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    let text_document_position = &params.text_document_position;
    let typing_text = get_typing_text(text_document_position, uri_to_content);

    // In dot-completion contexts, run the receiver-type extraction
    // pipeline so we can rank candidates by how well their receiver
    // position matches the typed receiver. On failure we silently fall
    // back to the legacy alphabetical list; no client-visible error.
    let dot_ranking = if is_dot_function(&typing_text) {
        let receiver_ty =
            extract_receiver_type_for_dot_completion(text_document_position, uri_to_content);
        match receiver_ty {
            Some(receiver_ty) => {
                write_log!(
                    "[completion] dot-context receiver type: {}",
                    receiver_ty.to_string()
                );
                Some(DotRanking {
                    receiver_type: receiver_ty,
                    index: index::CompletionIndex::build(program),
                })
            }
            None => None,
        }
    } else {
        None
    };

    let namespace = extract_namespace_from_typing_text(&typing_text);
    let is_in_namespace = |name: &FullName| namespace.is_suffix_of(&name.namespace);

    let mut items = vec![];

    /// Builds a `CompletionItem` for one symbol, stashing the data
    /// needed by `completionItem/resolve` (the `EndNode`, the typing
    /// text, and the original cursor position) into the item's `data`
    /// field.
    fn create_item(
        name: &FullName,
        kind: CompletionItemKind,
        detail: Option<String>,
        end_node: &EndNode,
        typing_text: &str,
        text_document_position: &TextDocumentPositionParams,
        deprecated: bool,
    ) -> CompletionItem {
        // Set both `deprecated` (LSP <3.15) and `tags` (LSP >=3.15) so older
        // and newer clients both render the strikethrough.
        let (deprecated_field, tags_field) = if deprecated {
            (Some(true), Some(vec![CompletionItemTag::DEPRECATED]))
        } else {
            (None, None)
        };
        CompletionItem {
            label: name.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: None,
                description: None,
            }),
            kind: Some(kind),
            detail,
            documentation: None,
            deprecated: deprecated_field,
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
            tags: tags_field,
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
        let mut item = create_item(
            full_name,
            CompletionItemKind::FUNCTION,
            Some(scheme),
            &EndNode::Expr(Var::create(full_name.clone()), None),
            &typing_text,
            &text_document_position,
            gv.deprecation.is_some(),
        );
        if let Some(ranking) = &dot_ranking {
            let tier = score::assign_tier(full_name, &ranking.index, &ranking.receiver_type);
            item.sort_text = Some(score::sort_text_for(tier, full_name));
        }
        items.push(item);
    }
    for (tycon, _kind) in program.type_env.kinds() {
        if tycon.name.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(&tycon.name) {
            continue;
        }
        let mut item = create_item(
            &tycon.name,
            CompletionItemKind::CLASS,
            None,
            &EndNode::Type(tycon.clone()),
            &typing_text,
            &text_document_position,
            false,
        );
        if dot_ranking.is_some() {
            // Types can't appear after a dot in Fix, so they shouldn't
            // outrank function candidates. Tier 3 keeps them present
            // (the user might still want them in a misclassified
            // context) but pushed to the bottom of the list.
            item.sort_text = Some(score::sort_text_for(score::Tier::Three, &tycon.name));
        }
        items.push(item);
    }
    for trait_ in program.traits_with_aliases() {
        if trait_.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(&trait_.name) {
            continue;
        }
        let mut item = create_item(
            &trait_.name,
            CompletionItemKind::INTERFACE,
            None,
            &EndNode::Trait(trait_.clone()),
            &typing_text,
            &text_document_position,
            false,
        );
        if dot_ranking.is_some() {
            item.sort_text = Some(score::sort_text_for(score::Tier::Three, &trait_.name));
        }
        items.push(item);
    }
    for (assoc_type, _kind_info) in program.trait_env.assoc_ty_kind_info() {
        if assoc_type.name.to_string().contains('#') {
            continue;
        }
        if !is_in_namespace(&assoc_type.name) {
            continue;
        }
        let mut item = create_item(
            &assoc_type.name,
            CompletionItemKind::CLASS,
            None,
            &EndNode::AssocType(assoc_type.clone()),
            &typing_text,
            &text_document_position,
            false,
        );
        if dot_ranking.is_some() {
            item.sort_text = Some(score::sort_text_for(score::Tier::Three, &assoc_type.name));
        }
        items.push(item);
    }
    send_response(id, Ok::<_, ()>(items));
}

/// Bundle of the data Step 2 needs to assign tiers to candidates: the
/// receiver type extracted from the live buffer, plus the bucket index
/// over the snapshot's globals.
struct DotRanking {
    receiver_type: Arc<TypeNode>,
    index: index::CompletionIndex,
}

/// Run the dot-completion type-extraction pipeline for a single
/// completion request: repair the live buffer at the cursor (replacing
/// the post-dot identifier with `?`), re-elaborate via
/// `elaborate_via_config`, and return the receiver type read off the
/// inserted hole's inferred curried type.
///
/// **Step 1 prototype**: `n = 0` is hard-coded (the receiver is the
/// last element of the hole's curried sources), and only A0 of the
/// repair is run. Callers must check `is_dot_function` before invoking
/// this — non-dot contexts must not pay the elaborate cost.
///
/// Returns `None` (silent fallback to alphabetical-order completion)
/// when any step fails: the path can't be resolved, the live buffer
/// has no `<id>.<post-dot>` shape near the cursor, the elaborate fails,
/// the hole isn't located, or the hole's type isn't a curried function.
pub(super) fn extract_receiver_type_for_dot_completion(
    text_document_position: &TextDocumentPositionParams,
    uri_to_content: &Map<Uri, LatestContent>,
) -> Option<Arc<TypeNode>> {
    let uri = &text_document_position.text_document.uri;
    let latest = uri_to_content.get(uri)?;
    let live_buffer = &latest.content;

    // Cursor as a byte offset into the live buffer.
    let cursor_byte = position_to_bytes(live_buffer, text_document_position.position);

    // Step A0 of the repair (post-dot identifier → `?`). Outer-source
    // repair (A.4.2) is deferred to Step 4.
    let repaired = repair::repair_for_completion(live_buffer, cursor_byte)?;

    // Resolve to an absolute path so the override-key matches what
    // `parse_file_path` looks up. The completion thread's cwd is the
    // workspace root just like the diagnostics thread, so relative
    // resolution stays consistent between the two flows.
    let abs_path = to_absolute_path(&latest.path).ok()?;

    let program = run_completion_elaborate(&abs_path, repaired.source).ok()?;

    // Locate the hole node and read its inferred curried type. Step 1
    // hard-codes `n = 0`, so the hole's type should be `Self → Ret`
    // and the receiver is the only source argument.
    let cursor = SourcePosLite {
        path: abs_path,
        byte: repaired.cursor_byte,
    };
    let hole = find_innermost_hole_at(&program, &cursor)?;
    let hole_ty = hole.type_.as_ref()?;
    decompose_hole_type_n0(hole_ty)
}

/// Extract the receiver (`Self`) type from a hole whose inferred type
/// is `Self → Ret` (the `n = 0` case). Returns `None` if the hole
/// type isn't a function — typically because elaborate gave the hole
/// a fresh type variable when it couldn't be constrained from context.
fn decompose_hole_type_n0(hole_ty: &Arc<TypeNode>) -> Option<Arc<TypeNode>> {
    if !(hole_ty.is_funptr() || hole_ty.is_closure()) {
        return None;
    }
    let srcs = hole_ty.get_lambda_srcs();
    // For `n = 0` the receiver is the last (and typically only)
    // curried source. Mirrors plan §A.7's `S_{m-n-1}` with `n = 0`.
    srcs.into_iter().last()
}

/// Drive the elaborate pipeline against a configuration that swaps in
/// `repaired_content` for `path`'s on-disk contents. Mirrors the
/// initial setup in `run_diagnostics` — read the project file, build a
/// `DiagnosticsConfig`, apply lockfile — then plants the live override
/// just before invoking elaborate.
fn run_completion_elaborate(
    path: &PathBuf,
    repaired_content: String,
) -> Result<Program, crate::error::Errors> {
    let proj_file = ProjectFile::read_root_file()?;
    let files = proj_file.get_files(crate::configuration::BuildConfigType::Test);
    let mut config = Configuration::diagnostics_mode(DiagnosticsConfig { files })?;
    proj_file.set_config(&mut config)?;
    proj_file
        .open_or_auto_update_lock_file(LockFileType::Lsp)?
        .set_config(&mut config)?;

    let mut overrides = Map::default();
    overrides.insert(path.clone(), repaired_content);
    config.live_source_overrides = Arc::new(overrides);

    elaborate_via_config(&config)
}

/// Path-and-byte cursor pair used to identify the hole in the
/// re-elaborated AST. Kept as a path rather than a `SourcePos` so the
/// caller doesn't need to construct a `SourceFile` cache that nobody
/// reads.
struct SourcePosLite {
    path: PathBuf,
    byte: usize,
}

impl SourcePosLite {
    fn includes(&self, span: &Span) -> bool {
        let span_path = to_absolute_path(&span.input.file_path).ok();
        let our_path = to_absolute_path(&self.path).ok();
        if span_path.is_none() || our_path.is_none() || span_path != our_path {
            return false;
        }
        // Inclusive on both ends to match `Span::includes_pos_lsp`.
        span.start <= self.byte && self.byte <= span.end
    }
}

/// Find the innermost `Expr::Var(Std::#hole)` whose span contains the
/// cursor. "Innermost" = the match with the smallest span; this picks
/// the single hole the repair just inserted even when other holes
/// (e.g. ones the user wrote, or future repair-loop insertions) live
/// in nearby code.
fn find_innermost_hole_at(
    program: &Program,
    cursor: &SourcePosLite,
) -> Option<Arc<ExprNode>> {
    let target = hole_full_name();
    let mut best: Option<Arc<ExprNode>> = None;
    for (_name, gv) in &program.global_values {
        match &gv.expr {
            SymbolExpr::Simple(te) => {
                walk_for_hole(&te.expr, cursor, &target, &mut best);
            }
            SymbolExpr::Method(impls) => {
                for m in impls {
                    walk_for_hole(&m.expr.expr, cursor, &target, &mut best);
                }
            }
        }
    }
    best
}

fn walk_for_hole(
    expr: &Arc<ExprNode>,
    cursor: &SourcePosLite,
    target: &FullName,
    best: &mut Option<Arc<ExprNode>>,
) {
    let Some(span) = expr.source.as_ref() else {
        // Synthetic / desugared nodes don't have a span; their
        // children might, so descend regardless.
        recurse_for_hole(expr, cursor, target, best);
        return;
    };
    if !cursor.includes(span) {
        return;
    }
    if let Expr::Var(v) = &*expr.expr {
        if v.name == *target {
            // Choose the smallest enclosing span — innermost wins.
            let take = match best {
                None => true,
                Some(prev) => {
                    let prev_len = prev
                        .source
                        .as_ref()
                        .map(|s| s.end - s.start)
                        .unwrap_or(usize::MAX);
                    let cur_len = span.end - span.start;
                    cur_len <= prev_len
                }
            };
            if take {
                *best = Some(expr.clone());
            }
        }
    }
    recurse_for_hole(expr, cursor, target, best);
}

fn recurse_for_hole(
    expr: &Arc<ExprNode>,
    cursor: &SourcePosLite,
    target: &FullName,
    best: &mut Option<Arc<ExprNode>>,
) {
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            walk_for_hole(func, cursor, target, best);
            for a in args {
                walk_for_hole(a, cursor, target, best);
            }
        }
        Expr::Lam(_, body) => walk_for_hole(body, cursor, target, best),
        Expr::Let(pat, bound, val) => {
            let _ = pat as &Arc<PatternNode>;
            walk_for_hole(bound, cursor, target, best);
            walk_for_hole(val, cursor, target, best);
        }
        Expr::If(c, t, e) => {
            walk_for_hole(c, cursor, target, best);
            walk_for_hole(t, cursor, target, best);
            walk_for_hole(e, cursor, target, best);
        }
        Expr::Match(c, arms) => {
            walk_for_hole(c, cursor, target, best);
            for (_, val) in arms {
                walk_for_hole(val, cursor, target, best);
            }
        }
        Expr::TyAnno(e, _) => walk_for_hole(e, cursor, target, best),
        Expr::MakeStruct(_, fields) => {
            for (_, _, e) in fields {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for e in args {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::Eval(side, main) => {
            walk_for_hole(side, cursor, target, best);
            walk_for_hole(main, cursor, target, best);
        }
    }
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

                    // Append argument list to the insert text. Each parameter
                    // is wrapped in the user-hole syntax `?<name>` and turned
                    // into an LSP snippet tab-stop `${N:?<name>}` so editors
                    // that support snippets (VSCode, Neovim, Helix, …) put
                    // the cursor on the first hole, let the user tab through
                    // the rest, and pre-select each placeholder so typing
                    // overwrites it. The snippet text the editor expands to
                    // is still `?<name>`, which is a Fix hole expression —
                    // so even if the user dismisses the snippet without
                    // touching anything, the source type-checks (with hole
                    // diagnostics) instead of producing "undefined name `x`"
                    // or `f()` unit-call errors.
                    if let Some(insert_text) = &mut item.insert_text {
                        if params.len() > 0 {
                            let placeholders: Vec<String> = params
                                .iter()
                                .enumerate()
                                .map(|(i, p)| format!("${{{}:?{}}}", i + 1, p))
                                .collect();
                            *insert_text += "(";
                            *insert_text += &placeholders.join(", ");
                            *insert_text += ")";
                            item.insert_text_format = Some(InsertTextFormat::SNIPPET);
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
        EndNode::AssocType(assoc_type) => Some(assoc_type.name.clone()),
        EndNode::ValueDecl(name) => Some(name), // Should not be used for completion, but just in case.
        EndNode::Field(_, _) | EndNode::Variant(_, _) => None,
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
