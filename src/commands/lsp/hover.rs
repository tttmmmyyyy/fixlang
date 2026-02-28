use super::server::{send_response, LatestContent};
use super::util::{document_from_endnode, get_node_at};
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
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let node = node.unwrap();
    let content = document_from_endnode(&node, program);
    let hover = lsp_types::Hover {
        contents: lsp_types::HoverContents::Markup(content),
        range: None,
    };
    send_response(id, Ok::<_, ()>(hover))
}
