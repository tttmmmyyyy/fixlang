// LSP "Rename Symbol" implementation.
//
// Phase C1 covers local variables and global values.
// Phase C2 adds type aliases, traits, trait aliases, associated types,
// struct fields, and union variants. Renaming a struct or union type
// itself is deferred to Phase D because it requires updating the
// auto-implemented method namespace.
// Phase C3 adds gating: stale-buffer detection, refusal to rename
// symbols defined outside the project, and refusal to rename
// auto-generated accessors directly.

use std::collections::HashMap;
use std::path::PathBuf;

use lsp_types::{
    PrepareRenameResponse, Range, RenameParams, TextDocumentPositionParams, TextEdit, Uri,
    WorkspaceEdit,
};
use serde::Serialize;

use super::references::{
    find_assoc_type_references, find_global_value_references, find_member_occurrences,
    find_trait_references, find_type_references,
};
use super::server::{send_response, LatestContent};
use super::util::{
    find_local_occurrences, get_current_dir, path_to_uri, resolve_source_pos, span_to_range,
};
use crate::ast::expr::{Expr, ExprNode};
use crate::ast::import::{ImportStatement, ImportTreeNode};
use crate::ast::name::{FullName, Name};
use crate::ast::program::{EndNode, Program, SymbolExpr};
use crate::ast::traits::TraitId;
use crate::ast::typedecl::TypeDeclValue;
use crate::ast::types::TyCon;
use crate::misc::{to_absolute_path, Map};
use crate::parse::parser::{
    parse_namespace_items_in_fullname, validate_token_str, TokenCategory,
};
use crate::parse::sourcefile::{SourcePos, Span};
use std::sync::Arc;

// LSP `ResponseError` shape: `{ code, message }`.
#[derive(Serialize)]
struct ResponseError {
    code: i64,
    message: String,
}

impl ResponseError {
    fn invalid_request(message: impl Into<String>) -> Self {
        // -32600 is the JSON-RPC reserved code for Invalid Request.
        ResponseError {
            code: -32600,
            message: message.into(),
        }
    }
}

// Handle "textDocument/prepareRename".
//
// LSP clients usually surface the result like this:
//   - `null`                     => "This element can't be renamed."
//                                   (a generic, not-very-helpful message)
//   - `ResponseError { message }` => the message verbatim, in a popup
//   - `DefaultBehavior { true }` => proceed to the rename input box
//
// So we use `ResponseError` whenever there is a clear, actionable reason
// to refuse (stale buffer, external symbol, auto-method click). We keep
// `null` for cases where the cursor is not on anything renameable in the
// first place — there's no element to talk about.
pub(super) fn handle_prepare_rename(
    id: u32,
    params: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    // Stale-buffer check first: if the editor buffer has drifted from the
    // AST the program was built against, even resolve_source_pos and
    // find_node_at can land on the wrong node, so bail out before any
    // node-shape analysis.
    if let Err(msg) = check_buffer_in_sync_with_program(program, uri_to_content) {
        send_response(id, Err::<(), _>(ResponseError::invalid_request(msg)));
        return;
    }

    let Some(pos) = resolve_source_pos(params, program, uri_to_content) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    };

    if !rename_target_supported(program, &node) {
        // Nothing renameable here at all (e.g. a module name) — the
        // generic "can't be renamed" message is acceptable.
        send_response(id, Ok::<_, ()>(None::<PrepareRenameResponse>));
        return;
    }
    if is_auto_method_var(program, &node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename an auto-generated accessor. \
                 Rename the field or variant declaration instead.",
            )),
        );
        return;
    }
    if !target_is_user_defined(program, &node, &pos) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename a symbol defined outside this project.",
            )),
        );
        return;
    }

    let resp = PrepareRenameResponse::DefaultBehavior {
        default_behavior: true,
    };
    send_response(id, Ok::<_, ()>(Some(resp)));
}

// Handle "textDocument/rename".
pub(super) fn handle_rename(
    id: u32,
    params: &RenameParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    let new_name = &params.new_name;

    // Stale-buffer check first (see prepareRename for reasoning).
    if let Err(msg) = check_buffer_in_sync_with_program(program, uri_to_content) {
        send_response(id, Err::<(), _>(ResponseError::invalid_request(msg)));
        return;
    }

    let Some(pos) = resolve_source_pos(&params.text_document_position, program, uri_to_content)
    else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };

    if !rename_target_supported(program, &node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                rename_unsupported_message(program, &node),
            )),
        );
        return;
    }

    // Reject auto-generated accessors before any further analysis. The
    // user must rename the field/variant itself instead, where they don't
    // have to think about whether the new name should include the `@`,
    // `set_`, etc. prefix.
    if is_auto_method_var(program, &node) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename an auto-generated accessor. \
                 Rename the field or variant declaration instead.",
            )),
        );
        return;
    }

    // Reject symbols whose declaration lives outside this project's source
    // tree (i.e. not listed in `fixproj.toml`'s `files` section).
    if !target_is_user_defined(program, &node, &pos) {
        send_response(
            id,
            Err::<(), _>(ResponseError::invalid_request(
                "Cannot rename a symbol defined outside this project.",
            )),
        );
        return;
    }

    // Validate new_name against the appropriate token category.
    let category = token_category_for(&node);
    if let Err(msg) = validate_token_str(new_name, category) {
        send_response(id, Err::<(), _>(ResponseError::invalid_request(msg)));
        return;
    }

    // Collect (Span, new_text) pairs for all occurrences.
    let edits: Vec<(Span, String)> = match &node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                let Some(occ) = find_local_occurrences(program, &pos, name) else {
                    send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
                    return;
                };
                let mut spans = vec![occ.definition];
                spans.extend(occ.uses);
                spans
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            } else {
                find_global_value_references(program, name, true)
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            }
        }
        EndNode::ValueDecl(name) => find_global_value_references(program, name, true)
            .into_iter()
            .map(|s| (s, new_name.clone()))
            .collect(),
        EndNode::Type(tycon) => collect_type_rename_edits(program, tycon, new_name),
        EndNode::TypeOrTrait(name) => {
            // Resolve to either a type or a trait. Type takes precedence
            // because in `program.type_env` aliases are also registered.
            let tycon = TyCon { name: name.clone() };
            if program.type_env.tycons.contains_key(&tycon)
                || program.type_env.aliases.contains_key(&tycon)
            {
                collect_type_rename_edits(program, &tycon, new_name)
            } else {
                let trait_id = TraitId::from_fullname(name.clone());
                find_trait_references(program, &trait_id, true)
                    .into_iter()
                    .map(|s| (s, new_name.clone()))
                    .collect()
            }
        }
        EndNode::Trait(trait_id) => find_trait_references(program, trait_id, true)
            .into_iter()
            .map(|s| (s, new_name.clone()))
            .collect(),
        EndNode::AssocType(assoc_type) => {
            find_assoc_type_references(program, assoc_type, true)
                .into_iter()
                .map(|s| (s, new_name.clone()))
                .collect()
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
            find_member_occurrences(program, tc, name, true)
                .into_iter()
                .map(|occ| (occ.span, format!("{}{}", occ.prefix, new_name)))
                .collect()
        }
        EndNode::Module(_) => unreachable!("Module rename is filtered out earlier"),
    };

    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<WorkspaceEdit>));
        return;
    };
    let workspace_edit = build_workspace_edit(edits, &cdir);
    send_response(id, Ok::<_, ()>(Some(workspace_edit)));
}

// Whether the symbol at this EndNode is renameable at all. Used by both
// prepareRename and rename to keep their answers consistent.
fn rename_target_supported(_program: &Program, node: &EndNode) -> bool {
    match node {
        EndNode::Expr(_, _) | EndNode::Pattern(_, _) | EndNode::ValueDecl(_) => true,
        EndNode::Trait(_) | EndNode::AssocType(_) => true,
        EndNode::Field(_, _) | EndNode::Variant(_, _) => true,
        EndNode::Type(_) | EndNode::TypeOrTrait(_) => true,
        EndNode::Module(_) => false,
    }
}

// Diagnostic message for the unsupported-target rejection. Keeps
// prepareRename and rename consistent in what they tell the user.
fn rename_unsupported_message(_program: &Program, node: &EndNode) -> String {
    match node {
        EndNode::Module(_) => "Renaming modules is not supported.".to_string(),
        _ => "Rename is not supported for this kind of symbol.".to_string(),
    }
}

// True iff `tc` is defined as a struct or union (as opposed to a type
// alias or a built-in TyCon). Struct/union types own an auto-namespace
// of compiler-generated methods and require Phase D handling, which is
// not yet implemented.
fn is_struct_or_union_type(program: &Program, tc: &TyCon) -> bool {
    program
        .type_defns
        .iter()
        .find(|td| td.tycon() == *tc)
        .map(|td| matches!(td.value, TypeDeclValue::Struct(_) | TypeDeclValue::Union(_)))
        .unwrap_or(false)
}

// Pick the pest grammar token category that the new name must satisfy.
fn token_category_for(node: &EndNode) -> TokenCategory {
    match node {
        EndNode::Expr(_, _) | EndNode::Pattern(_, _) | EndNode::ValueDecl(_) => TokenCategory::Name,
        EndNode::Type(_) | EndNode::TypeOrTrait(_) | EndNode::Trait(_) | EndNode::AssocType(_) => {
            TokenCategory::CapitalName
        }
        EndNode::Field(_, _) | EndNode::Variant(_, _) => TokenCategory::TypeFieldName,
        EndNode::Module(_) => TokenCategory::CapitalName,
    }
}

// Group `(Span, new_text)` pairs by URI and produce a `WorkspaceEdit`.
// Spans whose paths cannot be converted to URIs are silently dropped.
// Within each URI, edits are deduplicated to avoid multiple TextEdits at
// the same range (which can happen when refs reports both decl_src and
// defn_src for a `name : T = ...` form).
fn build_workspace_edit(edits: Vec<(Span, String)>, cdir: &PathBuf) -> WorkspaceEdit {
    let mut by_uri: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
    for (span, new_text) in edits {
        let uri = match path_to_uri(&cdir.join(&span.input.file_path)) {
            Ok(u) => u,
            Err(_) => continue,
        };
        let range = span_to_range(&span);
        by_uri.entry(uri).or_default().push(TextEdit { range, new_text });
    }
    // Deduplicate (range, new_text) pairs per URI.
    for edits in by_uri.values_mut() {
        edits.sort_by(|a, b| {
            (
                a.range.start.line,
                a.range.start.character,
                a.range.end.line,
                a.range.end.character,
            )
                .cmp(&(
                    b.range.start.line,
                    b.range.start.character,
                    b.range.end.line,
                    b.range.end.character,
                ))
        });
        edits.dedup_by(|a, b| range_eq(&a.range, &b.range) && a.new_text == b.new_text);
    }
    WorkspaceEdit {
        changes: Some(by_uri),
        document_changes: None,
        change_annotations: None,
    }
}

// =====================================================================
// Type rename: bare-name occurrences plus the struct/union auto-namespace
// rewrite (Phase D).
//
// For type aliases and other types without an auto-namespace, this is
// just `find_type_references`. For struct/union types whose
// auto-namespace owns user-callable methods (`@x`, `set_x`, ...), the
// auto-namespace path itself moves with the type, so:
//   - import-tree NameSpace components on that path are rewritten,
//     splitting the import into auto/user halves where necessary;
//   - inline qualified references like `Point::@x` and `[^Point::x]`
//     have just their `Point` sub-span rewritten.
// =====================================================================
fn collect_type_rename_edits(
    program: &Program,
    tc: &TyCon,
    new_name: &Name,
) -> Vec<(Span, String)> {
    // (A) Bare token spans: declaration, type annotations, MakeStruct,
    // Pattern::Struct, impl blocks, `import Foo::{Point}` (TypeOrTrait
    // import). Already handled by `find_type_references`.
    let mut edits: Vec<(Span, String)> = find_type_references(program, tc, true)
        .into_iter()
        .map(|s| (s, new_name.clone()))
        .collect();

    // For non-struct/non-union types (aliases, builtins) the auto-namespace
    // doesn't exist, so (B) and (C) are no-ops. Bail out early.
    if !is_struct_or_union_type(program, tc) {
        return edits;
    }

    // The auto-namespace path is the type's full name expanded into a
    // namespace, e.g. tc=`Foo::Point` -> auto_ns=["Foo", "Point"].
    let mut auto_ns: Vec<Name> = tc.name.namespace.names.clone();
    auto_ns.push(tc.name.name.clone());
    let type_old: &Name = &tc.name.name;

    // (B) import-tree namespace components. Collect rebuild edits
    // separately so we don't accidentally filter them out below.
    let mut rebuild_edits: Vec<(Span, String)> = vec![];
    for stmts in program.mod_to_import_stmts.values() {
        for stmt in stmts {
            collect_import_edits_for_type(
                program,
                &auto_ns,
                type_old,
                new_name,
                stmt,
                &mut edits,
                &mut rebuild_edits,
            );
        }
    }

    // (C) inline qualified Var references (`Point::@x`, `[^Point::x]`).
    for (_n, gv) in &program.global_values {
        collect_inline_qualified_edits(
            program,
            &gv.expr,
            &auto_ns,
            type_old,
            new_name,
            &mut edits,
        );
    }

    // Drop any individual edit whose span is fully contained in a
    // rebuild's span — the rebuild already covers it. Then concatenate
    // the rebuild edits.
    let rebuild_spans: Vec<Span> = rebuild_edits.iter().map(|(s, _)| s.clone()).collect();
    let mut filtered: Vec<(Span, String)> = edits
        .into_iter()
        .filter(|(s, _)| {
            !rebuild_spans
                .iter()
                .any(|rb| rb.input.file_path == s.input.file_path && rb.start <= s.start && s.end <= rb.end)
        })
        .collect();
    filtered.extend(rebuild_edits);
    filtered
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NamespaceClassification {
    AllAuto,
    AllUser,
    Mixed,
}

// Walk one ImportStatement and emit (B)-phase edits for it.
//
// - For each `NameSpace` whose resolved path equals `auto_ns` and whose
//   children are uniformly auto-method imports: emit a single
//   `(name_span, new_name)` edit.
// - When children are mixed (auto + user), rebuild the entire statement
//   via `stringify()` and record the original statement span as
//   "rebuilt" so other edits inside it can be filtered out.
// - All-user NameSpace nodes are left alone.
fn collect_import_edits_for_type(
    program: &Program,
    auto_ns: &[Name],
    type_old: &Name,
    type_new: &Name,
    stmt: &ImportStatement,
    edits: &mut Vec<(Span, String)>,
    rebuilt_stmt_spans: &mut Vec<(Span, String)>,
) {
    let module = &stmt.module.0;
    let stmt_classifications = scan_import_tree_for_type(
        program,
        auto_ns,
        &[module.clone()],
        &stmt.items,
    );
    let stmt_classifications_hiding =
        scan_import_tree_for_type(program, auto_ns, &[module.clone()], &stmt.hiding);
    let any_mixed = stmt_classifications
        .iter()
        .chain(stmt_classifications_hiding.iter())
        .any(|c| c.classification == NamespaceClassification::Mixed);

    if any_mixed {
        // Rebuild the whole statement via stringify().
        if let Some(stmt_span) = &stmt.source {
            let mut new_stmt = stmt.clone();
            rewrite_import_tree_for_type(
                program,
                auto_ns,
                type_old,
                type_new,
                &[module.clone()],
                &mut new_stmt.items,
            );
            rewrite_import_tree_for_type(
                program,
                auto_ns,
                type_old,
                type_new,
                &[module.clone()],
                &mut new_stmt.hiding,
            );
            rebuilt_stmt_spans.push((stmt_span.clone(), new_stmt.stringify()));
        }
    } else {
        // Emit individual NameSpace name-span edits for all-auto nodes.
        for c in stmt_classifications.iter().chain(stmt_classifications_hiding.iter()) {
            if c.classification == NamespaceClassification::AllAuto {
                if let Some(span) = &c.name_span {
                    edits.push((span.clone(), type_new.clone()));
                }
            }
        }
    }
}

struct NamespaceClassificationEntry {
    classification: NamespaceClassification,
    // The source span of just the NameSpace's name component; None if the
    // grammar didn't record one (only happens for synthetic imports).
    name_span: Option<Span>,
}

// Walk an import-tree, returning a classification entry for each
// `NameSpace` node whose resolved path equals `auto_ns`.
fn scan_import_tree_for_type(
    program: &Program,
    auto_ns: &[Name],
    parent_path: &[Name],
    nodes: &[ImportTreeNode],
) -> Vec<NamespaceClassificationEntry> {
    let mut out = vec![];
    for node in nodes {
        if let ImportTreeNode::NameSpace(name, children, span) = node {
            let mut path = parent_path.to_vec();
            path.push(name.clone());
            if path == auto_ns {
                let classification = classify_namespace_children(program, &path, children);
                out.push(NamespaceClassificationEntry {
                    classification,
                    name_span: span.clone(),
                });
            } else {
                out.extend(scan_import_tree_for_type(program, auto_ns, &path, children));
            }
        }
    }
    out
}

// Mutate an import-tree in place to apply the type rename:
//   - rewrite each `TypeOrTrait(type_old, _)` at parent_path == auto_ns[..-1]
//     to `TypeOrTrait(type_new, _)`;
//   - rewrite each `NameSpace(type_old, ...)` at the same depth, splitting
//     the children when mixed.
fn rewrite_import_tree_for_type(
    program: &Program,
    auto_ns: &[Name],
    type_old: &Name,
    type_new: &Name,
    parent_path: &[Name],
    nodes: &mut Vec<ImportTreeNode>,
) {
    let parent_of_type = &auto_ns[..auto_ns.len() - 1];
    let mut new_nodes: Vec<ImportTreeNode> = Vec::with_capacity(nodes.len());
    for node in nodes.drain(..) {
        match node {
            ImportTreeNode::NameSpace(name, mut children, span) => {
                let mut path = parent_path.to_vec();
                path.push(name.clone());
                if path == auto_ns {
                    match classify_namespace_children(program, &path, &children) {
                        NamespaceClassification::AllAuto => {
                            new_nodes.push(ImportTreeNode::NameSpace(
                                type_new.clone(),
                                children,
                                span,
                            ));
                        }
                        NamespaceClassification::AllUser => {
                            new_nodes.push(ImportTreeNode::NameSpace(name, children, span));
                        }
                        NamespaceClassification::Mixed => {
                            let (auto_children, user_children) =
                                split_children_auto_user(program, &path, children);
                            new_nodes.push(ImportTreeNode::NameSpace(
                                type_new.clone(),
                                auto_children,
                                None,
                            ));
                            if !user_children.is_empty() {
                                new_nodes.push(ImportTreeNode::NameSpace(
                                    type_old.clone(),
                                    user_children,
                                    None,
                                ));
                            }
                        }
                    }
                } else {
                    rewrite_import_tree_for_type(
                        program,
                        auto_ns,
                        type_old,
                        type_new,
                        &path,
                        &mut children,
                    );
                    new_nodes.push(ImportTreeNode::NameSpace(name, children, span));
                }
            }
            ImportTreeNode::TypeOrTrait(name, span) => {
                if &name == type_old && parent_path == parent_of_type {
                    new_nodes.push(ImportTreeNode::TypeOrTrait(type_new.clone(), span));
                } else {
                    new_nodes.push(ImportTreeNode::TypeOrTrait(name, span));
                }
            }
            other @ (ImportTreeNode::Symbol(_, _) | ImportTreeNode::Any(_)) => {
                new_nodes.push(other);
            }
        }
    }
    *nodes = new_nodes;
}

// Classify the children of a `NameSpace` whose path equals the type's
// auto-namespace into AllAuto / AllUser / Mixed.
fn classify_namespace_children(
    program: &Program,
    namespace_path: &[Name],
    children: &[ImportTreeNode],
) -> NamespaceClassification {
    let mut has_auto = false;
    let mut has_user = false;
    for child in children {
        let (a, u) = classify_child(program, namespace_path, child);
        has_auto |= a;
        has_user |= u;
    }
    match (has_auto, has_user) {
        (true, false) => NamespaceClassification::AllAuto,
        (false, true) => NamespaceClassification::AllUser,
        (true, true) => NamespaceClassification::Mixed,
        // No children resolved to either side (empty group); leave the
        // import alone.
        (false, false) => NamespaceClassification::AllUser,
    }
}

fn classify_child(
    program: &Program,
    namespace_path: &[Name],
    child: &ImportTreeNode,
) -> (bool /* auto */, bool /* user */) {
    match child {
        ImportTreeNode::Any(_) => scan_namespace_for_auto_user(program, namespace_path),
        ImportTreeNode::Symbol(name, _) => {
            let full = make_fullname(namespace_path, name);
            let is_auto = program
                .global_values
                .get(&full)
                .map(|gv| gv.compiler_defined_method)
                .unwrap_or(false);
            if is_auto { (true, false) } else { (false, true) }
        }
        // Nested types or namespaces inside the auto-namespace are unusual
        // — be conservative and treat them as user items so they're not
        // accidentally moved.
        ImportTreeNode::TypeOrTrait(_, _) | ImportTreeNode::NameSpace(_, _, _) => (false, true),
    }
}

// Scan `program.global_values` for entries whose namespace exactly equals
// `namespace_path`, returning whether any compiler-defined and any
// user-defined items live there.
fn scan_namespace_for_auto_user(program: &Program, namespace_path: &[Name]) -> (bool, bool) {
    let mut has_auto = false;
    let mut has_user = false;
    for (full, gv) in &program.global_values {
        if full.namespace.names == namespace_path {
            if gv.compiler_defined_method {
                has_auto = true;
            } else {
                has_user = true;
            }
            if has_auto && has_user {
                break;
            }
        }
    }
    (has_auto, has_user)
}

// Partition the children of a NameSpace into (auto_children, user_children)
// halves for the mixed-import rebuild.
//
// `Any(*)` is replicated into both halves so the wildcard semantics are
// preserved for both auto and user items.
fn split_children_auto_user(
    program: &Program,
    namespace_path: &[Name],
    children: Vec<ImportTreeNode>,
) -> (Vec<ImportTreeNode>, Vec<ImportTreeNode>) {
    let mut auto_children = vec![];
    let mut user_children = vec![];
    for child in children {
        let (a, u) = classify_child(program, namespace_path, &child);
        // `Any(*)` may produce both flags; in that case duplicate the node
        // so the wildcard appears in each half, matching what stringify
        // expects.
        if a && u {
            auto_children.push(child.clone());
            user_children.push(child);
        } else if a {
            auto_children.push(child);
        } else if u {
            user_children.push(child);
        }
    }
    (auto_children, user_children)
}

fn make_fullname(namespace_path: &[Name], name: &Name) -> FullName {
    let mut ns_names = namespace_path.to_vec();
    ns_names.push(name.clone());
    let last = ns_names.pop().unwrap();
    FullName {
        namespace: crate::ast::name::NameSpace::new(ns_names),
        name: last,
    }
}

// (C) Walk every Var node in `expr` and emit an edit when it is a
// qualified reference to an auto-method of the type identified by
// `auto_ns`. User-defined items that happen to live in the same
// namespace (because the user wrote `namespace Point { ... }` next to
// the type definition) are intentionally left alone — the type rename
// only moves compiler-generated accessors.
fn collect_inline_qualified_edits(
    program: &Program,
    expr: &SymbolExpr,
    auto_ns: &[Name],
    type_old: &Name,
    new_name: &Name,
    edits: &mut Vec<(Span, String)>,
) {
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            walk_expr_for_inline_qualified(
                program,
                &typed_expr.expr,
                auto_ns,
                type_old,
                new_name,
                edits,
            );
        }
        SymbolExpr::Method(impls) => {
            for impl_ in impls {
                walk_expr_for_inline_qualified(
                    program,
                    &impl_.expr.expr,
                    auto_ns,
                    type_old,
                    new_name,
                    edits,
                );
            }
        }
    }
}

fn walk_expr_for_inline_qualified(
    program: &Program,
    expr: &Arc<ExprNode>,
    auto_ns: &[Name],
    type_old: &Name,
    new_name: &Name,
    edits: &mut Vec<(Span, String)>,
) {
    match &*expr.expr {
        Expr::Var(v) => {
            // Only rewrite Vars that resolve to a compiler-defined method
            // sitting in the type's auto-namespace. Skipping the
            // `compiler_defined_method` check would also rewrite
            // qualified references to helper functions the user wrote
            // under `namespace Point { ... }`, breaking calls like
            // `MinCostFlowGraph::create(...)` after the type is renamed.
            if v.name.namespace.names == auto_ns {
                let is_auto = program
                    .global_values
                    .get(&v.name)
                    .map(|gv| gv.compiler_defined_method)
                    .unwrap_or(false);
                if is_auto {
                    if let Some(span) = &expr.source {
                        if let Some(edit) =
                            extract_inline_qualified_edit(span, type_old, new_name)
                        {
                            edits.push(edit);
                        }
                    }
                }
            }
        }
        Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            walk_expr_for_inline_qualified(program, func, auto_ns, type_old, new_name, edits);
            for a in args {
                walk_expr_for_inline_qualified(program, a, auto_ns, type_old, new_name, edits);
            }
        }
        Expr::Lam(_, body) => {
            walk_expr_for_inline_qualified(program, body, auto_ns, type_old, new_name, edits);
        }
        Expr::Let(_pat, bound, val) => {
            walk_expr_for_inline_qualified(program, bound, auto_ns, type_old, new_name, edits);
            walk_expr_for_inline_qualified(program, val, auto_ns, type_old, new_name, edits);
        }
        Expr::If(c, t, e) => {
            walk_expr_for_inline_qualified(program, c, auto_ns, type_old, new_name, edits);
            walk_expr_for_inline_qualified(program, t, auto_ns, type_old, new_name, edits);
            walk_expr_for_inline_qualified(program, e, auto_ns, type_old, new_name, edits);
        }
        Expr::Match(c, pat_vals) => {
            walk_expr_for_inline_qualified(program, c, auto_ns, type_old, new_name, edits);
            for (_p, v) in pat_vals {
                walk_expr_for_inline_qualified(program, v, auto_ns, type_old, new_name, edits);
            }
        }
        Expr::TyAnno(e, _) => {
            walk_expr_for_inline_qualified(program, e, auto_ns, type_old, new_name, edits);
        }
        Expr::MakeStruct(_, fields) => {
            for (_, _, val) in fields {
                walk_expr_for_inline_qualified(program, val, auto_ns, type_old, new_name, edits);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                walk_expr_for_inline_qualified(program, e, auto_ns, type_old, new_name, edits);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for a in args {
                walk_expr_for_inline_qualified(program, a, auto_ns, type_old, new_name, edits);
            }
        }
        Expr::Eval(side, main) => {
            walk_expr_for_inline_qualified(program, side, auto_ns, type_old, new_name, edits);
            walk_expr_for_inline_qualified(program, main, auto_ns, type_old, new_name, edits);
        }
    }
}

// Re-parse a Var's source span (covering whatever qualified name the
// user actually wrote) and return an edit that rewrites just the type
// component within it. Returns None if the source doesn't contain the
// type name (e.g. unqualified `act_x`) or can't be re-parsed.
fn extract_inline_qualified_edit(
    var_source: &Span,
    type_old: &Name,
    new_name: &Name,
) -> Option<(Span, String)> {
    let content = var_source.input.string().ok()?;
    let s_start = var_source.start;
    let s_end = var_source.end;
    if s_end > content.len() || s_start > s_end {
        return None;
    }
    let text_slice = &content[s_start..s_end];
    // Index syntax `[^field]` desugars to a Var whose source covers `^field`.
    let caret_skip = if text_slice.starts_with('^') { 1 } else { 0 };
    let inner = &text_slice[caret_skip..];
    let items = parse_namespace_items_in_fullname(inner)?;
    // Rewrite only the LAST namespace_item — that's the one immediately
    // before the simple name and corresponds to the type's position in
    // the auto-namespace path (see plan §4.10 (C)). If the user wrote
    // an unqualified reference, items is empty and there's nothing to do.
    let last = items.last()?;
    if &last.name != type_old {
        return None;
    }
    let abs_start = s_start + caret_skip + last.start;
    let abs_end = s_start + caret_skip + last.end;
    Some((
        Span {
            input: var_source.input.clone(),
            start: abs_start,
            end: abs_end,
        },
        new_name.clone(),
    ))
}

fn range_eq(a: &Range, b: &Range) -> bool {
    a.start.line == b.start.line
        && a.start.character == b.start.character
        && a.end.line == b.end.line
        && a.end.character == b.end.character
}

// Compare each user source file's current content (from the editor buffer
// if open, otherwise from disk) against the content recorded at
// elaboration time. If any mismatch is found, return an error message
// suitable for surfacing to the user.
//
// The strict whole-project policy (per the rename plan) is intentional:
// rename touches AST spans, and any drift between the AST and the buffer
// can produce silently corrupt edits.
fn check_buffer_in_sync_with_program(
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) -> Result<(), String> {
    if program.source_contents.is_empty() {
        // The diagnostics thread didn't (or couldn't) record source
        // contents; treat that as "we don't know" and refuse rather than
        // produce edits we can't validate.
        return Err(
            "Cannot rename: source contents from the last build are not available. \
             Save the file and wait for diagnostics to refresh."
                .to_string(),
        );
    }

    // Collect (absolute path -> current buffer content) from `uri_to_content`
    // so we can look up by path below.
    let mut buffer_by_path: Map<PathBuf, String> = Map::default();
    for lc in uri_to_content.values() {
        if let Ok(abs) = to_absolute_path(&lc.path) {
            buffer_by_path.insert(abs, lc.content.clone());
        }
    }

    for path in &program.user_source_files {
        let elaborated = match program.source_contents.get(path) {
            Some(c) => c,
            // No record of this file; can't verify, so be safe.
            None => {
                return Err(format!(
                    "Cannot rename: source contents missing for `{}`. \
                     Save the file and wait for diagnostics to refresh.",
                    path.display()
                ));
            }
        };

        let current_owned: String;
        let current: &str = if let Some(buf) = buffer_by_path.get(path) {
            buf.as_str()
        } else {
            // No editor buffer — read from disk.
            current_owned = std::fs::read_to_string(path).unwrap_or_default();
            current_owned.as_str()
        };

        if current != elaborated {
            return Err(format!(
                "Cannot rename: `{}` has been edited since the last successful build. \
                 Save the file and wait for diagnostics to refresh.",
                path.display()
            ));
        }
    }
    Ok(())
}

// True if the EndNode at `pos` resolves to an auto-generated accessor
// (a global value with `compiler_defined_method == true`). Used to reject
// rename starting on `@x`, `set_x`, `act_x`, `[^x]`, `as_v`, `is_v`,
// `mod_v` and the union variant constructor — the user should rename the
// field or variant declaration itself.
fn is_auto_method_var(program: &Program, node: &EndNode) -> bool {
    let name = match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => &var.name,
        EndNode::ValueDecl(name) => name,
        _ => return false,
    };
    if name.is_local() {
        return false;
    }
    program
        .global_values
        .get(name)
        .map(|gv| gv.compiler_defined_method)
        .unwrap_or(false)
}

// True if the symbol at `node` is declared in a file listed in
// `fixproj.toml`'s `files` section. `pos` is the cursor position; for
// local variables we use the scope walker to find the binder span.
fn target_is_user_defined(program: &Program, node: &EndNode, pos: &SourcePos) -> bool {
    let decl_span = declaration_span(program, node, pos);
    let Some(span) = decl_span else {
        // No declaration span => can't verify => be conservative and refuse.
        return false;
    };
    let Ok(abs) = to_absolute_path(&span.input.file_path) else {
        return false;
    };
    program.user_source_files.contains(&abs)
}

// Return a span pointing to where the symbol at `node` is declared
// (defined) in source, used for the user-defined check. May return None
// if the symbol has no recorded declaration span (e.g. compiler builtins
// or a local that fails the scope lookup).
fn declaration_span(program: &Program, node: &EndNode, pos: &SourcePos) -> Option<Span> {
    match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                find_local_occurrences(program, pos, name).map(|o| o.definition)
            } else {
                program
                    .global_values
                    .get(name)
                    .and_then(|gv| gv.decl_src.clone().or_else(|| gv.defn_src.clone()))
            }
        }
        EndNode::ValueDecl(name) => program
            .global_values
            .get(name)
            .and_then(|gv| gv.decl_src.clone().or_else(|| gv.defn_src.clone())),
        EndNode::Type(tc) => program
            .type_defns
            .iter()
            .find(|td| td.tycon() == *tc)
            .and_then(|td| td.name_src.clone()),
        EndNode::TypeOrTrait(name) => {
            let tc = TyCon { name: name.clone() };
            if let Some(td) = program.type_defns.iter().find(|td| td.tycon() == tc) {
                td.name_src.clone()
            } else {
                let trait_id = TraitId::from_fullname(name.clone());
                program
                    .trait_env
                    .traits
                    .get(&trait_id)
                    .and_then(|ti| ti.name_src.clone())
                    .or_else(|| {
                        program
                            .trait_env
                            .aliases
                            .data
                            .get(&trait_id)
                            .and_then(|ta| ta.name_src.clone())
                    })
            }
        }
        EndNode::Trait(trait_id) => program
            .trait_env
            .traits
            .get(trait_id)
            .and_then(|ti| ti.name_src.clone())
            .or_else(|| {
                program
                    .trait_env
                    .aliases
                    .data
                    .get(trait_id)
                    .and_then(|ta| ta.name_src.clone())
            }),
        EndNode::AssocType(at) => {
            let trait_id = at.trait_id();
            program
                .trait_env
                .traits
                .get(&trait_id)
                .and_then(|ti| ti.assoc_types.get(&at.name.name))
                .and_then(|atd| atd.name_src.clone())
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => program
            .type_defns
            .iter()
            .find(|td| td.tycon() == *tc)
            .and_then(|td| {
                let fields = match &td.value {
                    TypeDeclValue::Struct(s) => &s.fields,
                    TypeDeclValue::Union(u) => &u.fields,
                    TypeDeclValue::Alias(_) => return None,
                };
                fields
                    .iter()
                    .find(|f| &f.name == name)
                    .and_then(|f| f.name_src.clone())
            }),
        EndNode::Module(_) => None,
    }
}

