use super::server::{send_response, LatestContent};
use super::util::{document_from_endnode, resolve_source_pos};
use crate::ast::program::Program;
use crate::misc::Map;
use lsp_types::HoverParams;

// Handle "textDocument/hover" method.
pub(super) fn handle_hover(
    id: u32,
    params: &HoverParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
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
    let content = document_from_endnode(&node, program);
    let hover = lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(content),
        range: None,
    };
    send_response(id, Ok::<_, ()>(hover))
}
