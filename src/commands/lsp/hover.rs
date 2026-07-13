use super::server::{send_response, LatestContent};
use super::util::{document_from_endnode, resolve_source_pos};
use crate::ast::program::{EndNode, Program};
use crate::misc::Map;
use lsp_types::{Hover, HoverContents, HoverParams, Uri};

// Handle "textDocument/hover" method.
pub(super) fn handle_hover(
    id: u32,
    params: &HoverParams,
    program: &Program,
    uri_to_content: &Map<Uri, LatestContent>,
) {
    // Resolve the cursor into a source position, then look up the AST node.
    let Some(pos) = resolve_source_pos(
        &params.text_document_position_params,
        program,
        uri_to_content,
    ) else {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    };
    if is_internal_name_node(&node) {
        // Suppress hover for compiler-internal names (e.g. the
        // `Std::#hole` placeholder synthesised by the parser when an
        // expression position is left empty). Their leading `#` is not
        // a valid identifier head, so they cannot collide with
        // user-defined names; surfacing them in the hover would expose
        // an implementation detail. Wildcard binders are the exception:
        // they are rendered as `_` (see `document_from_endnode`) so the
        // user can inspect the type a `_` matched.
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let content = document_from_endnode(&node, program);
    let hover = Hover {
        contents: HoverContents::Markup(content),
        range: None,
    };
    send_response(id, Ok::<_, ()>(hover))
}

/// Return true when `node` refers to a compiler-internal name (one
/// whose local name starts with `#`, e.g. `Std::#hole`). User
/// identifiers cannot start with `#`, so this never matches anything
/// the user wrote. Used to suppress hover content that would expose
/// internal placeholders. Wildcard binders (`#wildcard{N}`) are excluded:
/// they are shown as `_ : <type>`.
fn is_internal_name_node(node: &EndNode) -> bool {
    let name = match node {
        EndNode::Expr(var, _) => &var.name,
        EndNode::Pattern(var, _) => &var.name,
        _ => return false,
    };
    name.name.starts_with('#') && !name.is_wildcard()
}
