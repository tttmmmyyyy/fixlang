// LSP "textDocument/definition" handler.

use super::server::{send_response, LatestContent};
use super::util::{find_trait_or_alias_def_src, find_tycon_def_src, get_current_dir, get_node_at, span_to_location};
use crate::ast::program::Program;
use crate::ast::traits::TraitId;
use crate::ast::types::TyCon;
use crate::misc::Map;
use crate::EndNode;
use lsp_types::GotoDefinitionParams;

// Handle "textDocument/definition" method.
pub(super) fn handle_goto_definition(
    id: u32,
    params: &GotoDefinitionParams,
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

    // The source location where the item is defined.
    let mut def_src;

    // First check if the node is an expression or a pattern.
    let var_name = match &node {
        EndNode::Expr(var, _) => Some(var.name.clone()),
        EndNode::Pattern(var, _) => Some(var.name.clone()),
        EndNode::Type(_) => None,
        EndNode::Trait(_) => None,
        EndNode::Module(_) => None,
        EndNode::TypeOrTrait(_) => None,
        EndNode::AssocType(_) => None,
        // The cursor is on the declaration name itself; there is no other definition to jump to.
        EndNode::ValueDecl(_) => None,
    };
    if let Some(var_name) = var_name {
        // If the variable is local, do nothing.
        let full_name = &var_name;
        if full_name.is_local() {
            def_src = None;
        } else {
            def_src = program
                .global_values
                .get(full_name)
                .and_then(|gv| gv.decl_src.clone());
        }
    } else {
        // Then handle the case of a type or a trait or a module.
        match node {
            EndNode::Expr(_, _) => {
                unreachable!()
            }
            EndNode::Pattern(_, _) => {
                unreachable!()
            }
            EndNode::Type(tycon) => {
                def_src = find_tycon_def_src(program, tycon);
            }
            EndNode::Trait(trait_) => {
                def_src = find_trait_or_alias_def_src(program, trait_);
            }
            EndNode::TypeOrTrait(name) => {
                def_src = find_tycon_def_src(program, TyCon { name: name.clone() });
                if def_src.is_none() {
                    def_src = find_trait_or_alias_def_src(program, TraitId::from_fullname(name));
                }
            }
            EndNode::Module(mod_name) => {
                if let Some(mi) = program.modules.iter().find(|mi| mi.name == mod_name) {
                    def_src = Some(mi.source.clone());
                } else {
                    def_src = None;
                }
            }
            EndNode::AssocType(assoc_type) => {
                // Find the associated type definition in the trait.
                let trait_id = assoc_type.trait_id();
                def_src = program
                    .trait_env
                    .traits
                    .get(&trait_id)
                    .and_then(|ti| ti.assoc_types.get(&assoc_type.name.name))
                    .and_then(|atd| atd.name_src.clone());
            }
            EndNode::ValueDecl(_) => {
                unreachable!()
            }
        }
    }

    // If the source is not found, respond with None.
    if def_src.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    let def_src = def_src.unwrap();

    // Create response value.
    let Some(cdir) = get_current_dir() else {
        return;
    };
    let location = span_to_location(&def_src, &cdir);
    if location.is_none() {
        send_response(id, Ok::<_, ()>(None::<()>));
        return;
    }
    send_response(id, Ok::<_, ()>(location.unwrap()));
}
