// Enumeration of the program's completable symbols, and the assembly
// of a `CompletionItem` from one. The expression flow
// (`handle_completion`) and the import-statement flow both draw their
// candidates from these, so a change in which symbols are offered or
// how one renders lands in one place.

use super::{ResolveContext, ResolveData};
use crate::ast::expr::Var;
use crate::ast::name::FullName;
use crate::ast::program::{EndNode, Program};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionItemTag,
};

/// One symbol of the program that completion can offer.
pub(super) struct CompletionSymbol {
    pub(super) name: FullName,
    pub(super) kind: CompletionItemKind,
    /// The type signature, for a global value.
    pub(super) detail: Option<String>,
    /// The node stashed into the item's resolve data, from which
    /// `completionItem/resolve` derives the documentation.
    pub(super) node: EndNode,
    pub(super) deprecated: bool,
}

/// The program's global values: functions and values, trait methods
/// included.
pub(super) fn value_symbols(program: &Program) -> Vec<CompletionSymbol> {
    let mut symbols = vec![];
    for (full_name, gv) in &program.global_values {
        if is_internal_name(&full_name.to_string()) {
            continue;
        }
        let scheme = gv
            .syn_scm
            .clone()
            .unwrap_or(gv.scm.clone())
            .to_string_normalize();
        symbols.push(CompletionSymbol {
            name: full_name.clone(),
            kind: CompletionItemKind::FUNCTION,
            detail: Some(scheme),
            node: EndNode::Expr(Var::create(full_name.clone()), None),
            deprecated: gv.deprecation.is_some(),
        });
    }
    symbols
}

/// The program's type-level symbols: type constructors and aliases,
/// traits and trait aliases, and associated types.
pub(super) fn type_symbols(program: &Program) -> Vec<CompletionSymbol> {
    let mut symbols = vec![];
    for (tycon, _kind) in program.type_env.kinds() {
        if is_internal_name(&tycon.name.to_string()) {
            continue;
        }
        symbols.push(CompletionSymbol {
            name: tycon.name.clone(),
            kind: CompletionItemKind::CLASS,
            detail: None,
            node: EndNode::Type(tycon.clone()),
            deprecated: false,
        });
    }
    for trait_ in program.traits_with_aliases() {
        if is_internal_name(&trait_.to_string()) {
            continue;
        }
        symbols.push(CompletionSymbol {
            name: trait_.name.clone(),
            kind: CompletionItemKind::INTERFACE,
            detail: None,
            node: EndNode::Trait(trait_),
            deprecated: false,
        });
    }
    for (assoc_type, _kind_info) in program.trait_env.assoc_ty_kind_info() {
        if is_internal_name(&assoc_type.name.to_string()) {
            continue;
        }
        symbols.push(CompletionSymbol {
            name: assoc_type.name.clone(),
            kind: CompletionItemKind::CLASS,
            detail: None,
            node: EndNode::AssocType(assoc_type),
            deprecated: false,
        });
    }
    symbols
}

/// True for names that refer to compiler-internal entities and
/// shouldn't appear in user-facing completion. `#` marks
/// compiler-defined values/types (`Std::#hole`, …); `?` marks
/// opaque type variables turned into TyCons by opaque desugar
/// (`Std::Iterator::range::?it`, …). Neither character is legal in
/// a user-written identifier, so plain substring checks suffice.
fn is_internal_name(rendered: &str) -> bool {
    rendered.contains('#') || rendered.contains('?')
}

/// Builds the `CompletionItem` offering `symbol`, shown as `label` and
/// resolved in `context`.
pub(super) fn build_completion_item(
    symbol: CompletionSymbol,
    label: String,
    context: ResolveContext,
) -> CompletionItem {
    // Set both `deprecated` (LSP <3.15) and `tags` (LSP >=3.15) so older
    // and newer clients both render the strikethrough.
    let (deprecated_field, tags_field) = if symbol.deprecated {
        (Some(true), Some(vec![CompletionItemTag::DEPRECATED]))
    } else {
        (None, None)
    };
    CompletionItem {
        label,
        label_details: Some(CompletionItemLabelDetails {
            detail: None,
            description: None,
        }),
        kind: Some(symbol.kind),
        detail: symbol.detail,
        deprecated: deprecated_field,
        tags: tags_field,
        // Filter by the bare name, not the rendered label
        // (which may include the namespace). The label keeps the full
        // qualified path for display; the bare-name filter makes
        // typing `mpq` match `GMP.Q::mpq` with a top-tier fuzzy
        // score. Namespace-prefix typing is unaffected because the
        // `:` trigger character re-fires completion, after which
        // the server has already restricted the candidate set to the
        // typed namespace's members.
        filter_text: Some(symbol.name.name.clone()),
        insert_text: Some(symbol.name.name.clone()),
        data: Some(
            ResolveData {
                node: symbol.node,
                context,
            }
            .to_value(),
        ),
        ..CompletionItem::default()
    }
}
