// LSP "textDocument/definition" handler.

use super::server::{send_response, LatestContent};
use super::util::{
    find_field_def_src, find_local_occurrences, find_trait_or_alias_def_src, find_tycon_def_src,
    get_current_dir, resolve_source_pos, span_to_location,
};
use crate::ast::program::{EndNode, Program};
use crate::ast::traits::TraitId;
use crate::ast::types::TyCon;
use crate::misc::Map;
use lsp_types::GotoDefinitionParams;

// Handle "textDocument/definition" method.
pub(super) fn handle_goto_definition(
    id: u32,
    params: &GotoDefinitionParams,
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

    // The source location the name at the cursor is defined at. Every node the cursor can land on
    // is answered here, and a node that names nothing to jump to answers with `None`.
    let def_src = match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let full_name = &var.name;
            if full_name.is_local() {
                find_local_occurrences(program, &pos, full_name).map(|o| o.definition)
            } else {
                program
                    .global_values
                    .get(full_name)
                    .and_then(|gv| gv.decl_src.clone())
            }
        }
        EndNode::Type(tycon) => find_tycon_def_src(program, tycon),
        EndNode::Trait(trait_) => find_trait_or_alias_def_src(program, trait_),
        EndNode::TypeOrTrait(name) => find_tycon_def_src(program, TyCon { name: name.clone() })
            .or_else(|| find_trait_or_alias_def_src(program, TraitId::from_fullname(name))),
        EndNode::Module(mod_name) => program
            .modules
            .iter()
            .find(|mi| mi.name == mod_name)
            .map(|mi| mi.source.clone()),
        EndNode::AssocType(assoc_type) => {
            // The associated type is declared by the trait it belongs to.
            let trait_id = assoc_type.trait_id();
            program
                .trait_env
                .traits
                .get(&trait_id)
                .and_then(|ti| ti.assoc_types.get(&assoc_type.name.name))
                .and_then(|atd| atd.name_src.clone())
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
            find_field_def_src(program, &tc, &name)
        }
        // The cursor is on the declaration name itself, or on a `_` the compiler filled in.
        // Neither names a definition elsewhere.
        EndNode::ValueDecl(_) | EndNode::InferredType(_) => None,
    };

    // If the source is not found, respond with None.
    let Some(def_src) = def_src else {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    };

    // Create response value.
    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    };
    let location = span_to_location(&def_src, &cdir);
    if location.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    send_response(id, Ok::<_, ()>(location.unwrap()));
}
