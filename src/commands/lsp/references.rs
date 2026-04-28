// This module implements "Find All References" and "Call Hierarchy" LSP features.

use super::server::{send_response, LatestContent};
use super::util::{
    find_local_occurrences, get_current_dir, path_to_uri, resolve_source_pos, span_to_range,
    spans_to_locations,
};
use crate::ast::expr::{Expr, ExprNode};
use crate::ast::import::ImportTreeNode;
use crate::ast::name::{FullName, Name};
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::program::{Program, SymbolExpr};
use crate::ast::qual_pred::QualPred;
use crate::ast::qual_type::QualType;
use crate::ast::traits::TraitId;
use crate::ast::typedecl::{Field, TypeDeclValue, TypeDefn};
use crate::constants::{
    STRUCT_ACT_SYMBOL, STRUCT_GETTER_SYMBOL, STRUCT_MODIFIER_SYMBOL, STRUCT_SETTER_SYMBOL,
    UNION_AS_SYMBOL, UNION_IS_SYMBOL, UNION_MOD_SYMBOL,
};
use crate::ast::equality::Equality;
use crate::ast::traits::AssocTypeImpl;
use crate::ast::types::{Scheme, AssocType, Type, TyCon, TypeNode};
use crate::misc::Map;
use crate::ast::program::EndNode;
use crate::parse::sourcefile::{SourcePos, Span};
use lsp_types::{
    CallHierarchyIncomingCall, CallHierarchyIncomingCallsParams, CallHierarchyItem,
    CallHierarchyOutgoingCall, CallHierarchyOutgoingCallsParams, CallHierarchyPrepareParams,
    ReferenceParams, SymbolKind,
};
use std::sync::Arc;

// Handle "textDocument/references" method.
pub(super) fn handle_references(
    id: u32,
    params: &ReferenceParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) {
    // Resolve the cursor into a source position, then look up the AST node.
    let Some(pos) = resolve_source_pos(
        &params.text_document_position,
        program,
        uri_to_content,
    ) else {
        send_response(id, Ok::<_, ()>(None::<Vec<lsp_types::Location>>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<Vec<lsp_types::Location>>));
        return;
    };

    let include_declaration = params.context.include_declaration;

    // Collect all reference spans.
    let spans = find_all_references(program, &node, &pos, include_declaration);

    // Convert spans to Locations.
    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<Vec<lsp_types::Location>>));
        return;
    };

    let locations = spans_to_locations(spans, &cdir);

    send_response(id, Ok::<_, ()>(locations));
}

// Find all references to the entity represented by `node` in the program.
fn find_all_references(
    program: &Program,
    node: &EndNode,
    pos: &SourcePos,
    include_declaration: bool,
) -> Vec<Span> {
    let mut refs = match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                return find_local_refs(program, pos, name, include_declaration);
            }
            find_global_value_references(program, name, include_declaration)
        }
        EndNode::ValueDecl(name) => {
            find_global_value_references(program, name, include_declaration)
        }
        EndNode::Type(tycon) => find_type_references(program, tycon, include_declaration),
        EndNode::Trait(trait_id) => find_trait_references(program, trait_id, include_declaration),
        EndNode::TypeOrTrait(name) => {
            let tycon = TyCon { name: name.clone() };
            if program.type_env.tycons.contains_key(&tycon)
                || program.type_env.aliases.contains_key(&tycon)
            {
                find_type_references(program, &tycon, include_declaration)
            } else {
                let trait_id = TraitId::from_fullname(name.clone());
                find_trait_references(program, &trait_id, include_declaration)
            }
        }
        EndNode::AssocType(assoc_type) => {
            find_assoc_type_references(program, assoc_type, include_declaration)
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
            find_field_occurrences(program, tc, name, include_declaration)
                .into_iter()
                .map(|o| o.span)
                .collect()
        }
        EndNode::Module(_) => {
            // Module references are not supported yet.
            vec![]
        }
    };

    // Deduplicate spans.
    refs.sort();
    refs.dedup();

    refs
}

// Find all references to the local symbol under the cursor. The symbol is
// the innermost binding of `name` that is visible at `pos`.
fn find_local_refs(
    program: &Program,
    pos: &SourcePos,
    name: &FullName,
    include_declaration: bool,
) -> Vec<Span> {
    let Some(occ) = find_local_occurrences(program, pos, name) else {
        return vec![];
    };
    let mut refs = occ.uses;
    if include_declaration {
        refs.push(occ.definition);
    }
    refs
}

// Find all references to a global value (function/constant).
pub(super) fn find_global_value_references(
    program: &Program,
    target: &FullName,
    include_declaration: bool,
) -> Vec<Span> {
    let mut refs = vec![];

    // Include the declaration/definition locations if requested.
    if include_declaration {
        if let Some(gv) = program.global_values.get(target) {
            if let Some(span) = &gv.decl_src {
                refs.push(span.clone());
            }
            if let Some(span) = &gv.defn_src {
                // Avoid a duplicate entry when decl_src and defn_src point to the same location
                // which can happen in Fix when the declaration and definition are merged, 
                // e.g., `main : IO () = ...`.
                if gv.decl_src.as_ref() != Some(span) {
                    refs.push(span.clone());
                }
            }
        }
    }

    // Walk all global values' expressions.
    for (_name, gv) in &program.global_values {
        collect_symbol_expr_var_refs(&gv.expr, target, &mut refs);
    }

    // Walk import statements.
    collect_import_refs(program, target, true, &mut refs);

    // Walk `FFI_EXPORT[...]` pragmas: include the value-name span if the
    // pragma targets `target`.
    for stmt in &program.export_statements {
        if &stmt.value_name == target {
            if let Some(span) = &stmt.value_name_src {
                refs.push(span.clone());
            }
        }
    }

    // Walk `DEPRECATED[...]` pragmas: include the target-name span if the
    // pragma targets `target`.
    for stmt in &program.deprecation_statements {
        if &stmt.target_path == target {
            if let Some(span) = &stmt.target_name_src {
                refs.push(span.clone());
            }
        }
    }

    refs
}

// Find all references to a type constructor.
pub(super) fn find_type_references(
    program: &Program,
    target: &TyCon,
    include_declaration: bool,
) -> Vec<Span> {
    let mut refs = vec![];

    // Include the type name location in the definition if requested.
    if include_declaration {
        for td in &program.type_defns {
            if td.tycon() == *target {
                if let Some(span) = &td.name_src {
                    refs.push(span.clone());
                }
                break;
            }
        }
    }

    // Walk all global values' type signatures. Use `syn_scm` (which keeps
    // user-written aliases) when present so that `type T = U; f : T;` reports
    // the `T` occurrence in `f`'s signature when renaming the alias `T`.
    for (_name, gv) in &program.global_values {
        let scm_for_walk = gv.syn_scm.as_ref().unwrap_or(&gv.scm);
        collect_scheme_type_refs(scm_for_walk, target, &mut refs);
        // Walk expression trees for type annotations and patterns.
        collect_symbol_expr_type_refs(&gv.expr, target, &mut refs);
    }

    // Walk type definitions for type references in fields.
    for td in &program.type_defns {
        collect_typedecl_type_refs(td, target, &mut refs);
    }

    // Walk trait definitions.
    for (_trait_id, trait_defn) in &program.trait_env.traits {
        for member in &trait_defn.members {
            collect_qualtype_type_refs(&member.qual_ty, target, &mut refs);
        }
    }

    // Walk trait implementations.
    for (_trait_id, impls) in &program.trait_env.impls {
        for impl_ in impls {
            collect_qualpred_type_refs(&impl_.qual_pred, target, &mut refs);
            for (_member_name, member_sig) in &impl_.member_sigs {
                collect_qualtype_type_refs(member_sig, target, &mut refs);
            }
        }
    }

    // Walk import statements.
    collect_import_refs(program, &target.name, false, &mut refs);

    refs
}

// Find all references to a trait.
pub(super) fn find_trait_references(
    program: &Program,
    target: &TraitId,
    include_declaration: bool,
) -> Vec<Span> {
    let mut refs = vec![];

    // Include the trait name location in the definition if requested.
    if include_declaration {
        if let Some(ti) = program.trait_env.traits.get(target) {
            if let Some(span) = &ti.name_src {
                refs.push(span.clone());
            }
        }
        if let Some(ta) = program.trait_env.aliases.data.get(target) {
            if let Some(span) = &ta.name_src {
                refs.push(span.clone());
            }
        }
    }

    // Walk all global values' type signatures for trait predicates.
    for (_name, gv) in &program.global_values {
        collect_scheme_trait_refs(&gv.scm, target, &mut refs);
    }

    // Walk trait implementations.
    for (_trait_id, impls) in &program.trait_env.impls {
        for impl_ in impls {
            collect_qualpred_trait_refs(&impl_.qual_pred, target, &mut refs);
            // Walk member type signatures in the impl for trait constraints.
            for (_member_name, member_sig) in &impl_.member_sigs {
                collect_qualtype_trait_refs(member_sig, target, &mut refs);
            }
        }
    }

    // Walk trait definitions for predicates on members.
    for (_trait_id, trait_defn) in &program.trait_env.traits {
        for member in &trait_defn.members {
            collect_qualtype_trait_refs(&member.qual_ty, target, &mut refs);
        }
    }

    // Walk import statements.
    collect_import_refs(program, &target.name, false, &mut refs);

    refs
}

// Find all references to an associated type.
pub(super) fn find_assoc_type_references(
    program: &Program,
    target: &AssocType,
    include_declaration: bool,
) -> Vec<Span> {
    let mut refs = vec![];

    // Include the associated type definition location if requested.
    if include_declaration {
        let trait_id = target.trait_id();
        if let Some(ti) = program.trait_env.traits.get(&trait_id) {
            if let Some(atd) = ti.assoc_types.get(&target.name.name) {
                if let Some(span) = &atd.name_src {
                    refs.push(span.clone());
                }
            }
        }
    }

    // Walk all global values' type signatures for associated type refs.
    for (_name, gv) in &program.global_values {
        collect_scheme_assoc_type_refs(&gv.scm, target, &mut refs);
    }

    // Walk trait definitions for associated type refs in member signatures.
    for (_trait_id, trait_defn) in &program.trait_env.traits {
        for member in &trait_defn.members {
            collect_qualtype_assoc_type_refs(&member.qual_ty, target, &mut refs);
        }
    }

    // Walk trait implementations.
    for (_trait_id, impls) in &program.trait_env.impls {
        for impl_ in impls {
            // Walk impl declaration constraints.
            collect_qualpred_assoc_type_refs(&impl_.qual_pred, target, &mut refs);
            // Walk member type signatures.
            for (_member_name, member_sig) in &impl_.member_sigs {
                collect_qualtype_assoc_type_refs(member_sig, target, &mut refs);
            }
            // Walk associated type implementations.
            for (_name, assoc_impl) in &impl_.assoc_types {
                collect_assoc_type_impl_refs(assoc_impl, target, &mut refs);
            }
        }
    }

    refs
}

// Collect associated type references in a TypeNode tree.
fn collect_typenode_assoc_type_refs(
    ty: &Arc<TypeNode>,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    match &ty.ty {
        Type::TyCon(_) | Type::TyVar(_) => {}
        Type::TyApp(f, a) => {
            collect_typenode_assoc_type_refs(f, target, refs);
            collect_typenode_assoc_type_refs(a, target, refs);
        }
        Type::AssocTy(assoc_ty, args) => {
            if assoc_ty == target {
                if let Some(span) = &assoc_ty.src {
                    refs.push(span.clone());
                }
            }
            for arg in args {
                collect_typenode_assoc_type_refs(arg, target, refs);
            }
        }
    }
}

// Collect associated type references in an Equality.
fn collect_equality_assoc_type_refs(
    eq: &Equality,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    if &eq.assoc_type == target {
        if let Some(span) = &eq.assoc_type.src {
            refs.push(span.clone());
        }
    }
    for arg in &eq.args {
        collect_typenode_assoc_type_refs(arg, target, refs);
    }
    collect_typenode_assoc_type_refs(&eq.value, target, refs);
}

// Collect associated type references in a Scheme.
fn collect_scheme_assoc_type_refs(
    scheme: &Arc<Scheme>,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    collect_typenode_assoc_type_refs(&scheme.ty, target, refs);
    for pred in &scheme.predicates {
        collect_typenode_assoc_type_refs(&pred.ty, target, refs);
    }
    for eq in &scheme.equalities {
        collect_equality_assoc_type_refs(eq, target, refs);
    }
}

// Collect associated type references in a QualType.
fn collect_qualtype_assoc_type_refs(
    qt: &QualType,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    collect_typenode_assoc_type_refs(&qt.ty, target, refs);
    for pred in &qt.preds {
        collect_typenode_assoc_type_refs(&pred.ty, target, refs);
    }
    for eq in &qt.eqs {
        collect_equality_assoc_type_refs(eq, target, refs);
    }
}

// Collect associated type references in a QualPred.
fn collect_qualpred_assoc_type_refs(
    qp: &QualPred,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    collect_typenode_assoc_type_refs(&qp.predicate.ty, target, refs);
    for pred in &qp.pred_constraints {
        collect_typenode_assoc_type_refs(&pred.ty, target, refs);
    }
    for eq in &qp.eq_constraints {
        collect_equality_assoc_type_refs(eq, target, refs);
    }
}

// Collect associated type references in an AssocTypeImpl.
fn collect_assoc_type_impl_refs(
    assoc_impl: &AssocTypeImpl,
    target: &AssocType,
    refs: &mut Vec<Span>,
) {
    // Check if this impl is for the target associated type.
    // AssocTypeImpl.name is just the local name, so compare with the local part of target.
    if assoc_impl.name == target.name.name {
        if let Some(span) = &assoc_impl.name_src {
            refs.push(span.clone());
        }
    }
    // Also walk the value type for nested associated type references.
    collect_typenode_assoc_type_refs(&assoc_impl.value, target, refs);
}

// Collect variable references in a SymbolExpr.
fn collect_symbol_expr_var_refs(expr: &SymbolExpr, target: &FullName, refs: &mut Vec<Span>) {
    expr.walk_var_uses(&mut |var, src| {
        if &var.name == target {
            if let Some(span) = src {
                refs.push(span.clone());
            }
        }
    });
    expr.walk_patterns(&mut |pat| {
        collect_pattern_var_refs(pat, target, refs);
    });
}

// Collect variable references in a pattern.
fn collect_pattern_var_refs(pat: &Arc<PatternNode>, target: &FullName, refs: &mut Vec<Span>) {
    match &pat.pattern {
        Pattern::Var(v, _) => {
            if &v.name == target {
                if let Some(span) = &pat.info.source {
                    refs.push(span.clone());
                }
            }
        }
        Pattern::Struct(_, field_pats) => {
            for (_, _, sub_pat) in field_pats {
                collect_pattern_var_refs(sub_pat, target, refs);
            }
        }
        Pattern::Union(_, _, sub_pat) => {
            collect_pattern_var_refs(sub_pat, target, refs);
        }
    }
}

// Collect type references in a SymbolExpr.
fn collect_symbol_expr_type_refs(expr: &SymbolExpr, target: &TyCon, refs: &mut Vec<Span>) {
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            collect_exprnode_type_refs(&typed_expr.expr, target, refs);
        }
        SymbolExpr::Method(impls) => {
            for impl_ in impls {
                collect_exprnode_type_refs(&impl_.expr.expr, target, refs);
            }
        }
    }
}

// Collect type references in an expression tree (type annotations, patterns, MakeStruct).
fn collect_exprnode_type_refs(expr: &Arc<ExprNode>, target: &TyCon, refs: &mut Vec<Span>) {
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_exprnode_type_refs(func, target, refs);
            for arg in args {
                collect_exprnode_type_refs(arg, target, refs);
            }
        }
        Expr::Lam(_, body) => {
            collect_exprnode_type_refs(body, target, refs);
        }
        Expr::Let(pat, bound, val) => {
            collect_pattern_type_refs(pat, target, refs);
            collect_exprnode_type_refs(bound, target, refs);
            collect_exprnode_type_refs(val, target, refs);
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_exprnode_type_refs(cond, target, refs);
            collect_exprnode_type_refs(then_expr, target, refs);
            collect_exprnode_type_refs(else_expr, target, refs);
        }
        Expr::Match(cond, pat_vals) => {
            collect_exprnode_type_refs(cond, target, refs);
            for (pat, val) in pat_vals {
                collect_pattern_type_refs(pat, target, refs);
                collect_exprnode_type_refs(val, target, refs);
            }
        }
        Expr::TyAnno(e, ty) => {
            collect_exprnode_type_refs(e, target, refs);
            collect_typenode_type_refs(ty, target, refs);
        }
        Expr::MakeStruct(tc, fields) => {
            if tc.as_ref() == target {
                if let Some(span) = &expr.aux_src {
                    refs.push(span.clone());
                }
            }
            for (_, _, val) in fields {
                collect_exprnode_type_refs(val, target, refs);
            }
        }
        Expr::ArrayLit(elems) => {
            for elem in elems {
                collect_exprnode_type_refs(elem, target, refs);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for arg in args {
                collect_exprnode_type_refs(arg, target, refs);
            }
        }
        Expr::Eval(side, main) => {
            collect_exprnode_type_refs(side, target, refs);
            collect_exprnode_type_refs(main, target, refs);
        }
    }
}

// Collect type references in a pattern.
fn collect_pattern_type_refs(pat: &Arc<PatternNode>, target: &TyCon, refs: &mut Vec<Span>) {
    match &pat.pattern {
        Pattern::Var(_, opt_ty) => {
            if let Some(ty) = opt_ty {
                collect_typenode_type_refs(ty, target, refs);
            }
        }
        Pattern::Struct(tc, field_pats) => {
            if tc.as_ref() == target {
                if let Some(span) = &pat.info.aux_src {
                    refs.push(span.clone());
                }
            }
            for (_, _, sub_pat) in field_pats {
                collect_pattern_type_refs(sub_pat, target, refs);
            }
        }
        Pattern::Union(_, _, sub_pat) => {
            collect_pattern_type_refs(sub_pat, target, refs);
        }
    }
}

// Collect type constructor references in a TypeNode tree.
fn collect_typenode_type_refs(ty: &Arc<TypeNode>, target: &TyCon, refs: &mut Vec<Span>) {
    match &ty.ty {
        Type::TyCon(tc) => {
            if tc.as_ref() == target {
                if let Some(span) = ty.get_source() {
                    refs.push(span.clone());
                }
            }
        }
        Type::TyVar(_) => {}
        Type::TyApp(f, a) => {
            collect_typenode_type_refs(f, target, refs);
            collect_typenode_type_refs(a, target, refs);
        }
        Type::AssocTy(_, args) => {
            for arg in args {
                collect_typenode_type_refs(arg, target, refs);
            }
        }
    }
}

// Collect type references in a Scheme (type signature).
fn collect_scheme_type_refs(scheme: &Arc<Scheme>, target: &TyCon, refs: &mut Vec<Span>) {
    collect_typenode_type_refs(&scheme.ty, target, refs);
    for pred in &scheme.predicates {
        collect_typenode_type_refs(&pred.ty, target, refs);
    }
    for eq in &scheme.equalities {
        for arg in &eq.args {
            collect_typenode_type_refs(arg, target, refs);
        }
        collect_typenode_type_refs(&eq.value, target, refs);
    }
}

// Collect type references in a QualPred.
fn collect_qualpred_type_refs(qp: &QualPred, target: &TyCon, refs: &mut Vec<Span>) {
    collect_typenode_type_refs(&qp.predicate.ty, target, refs);
    for pred in &qp.pred_constraints {
        collect_typenode_type_refs(&pred.ty, target, refs);
    }
    for eq in &qp.eq_constraints {
        for arg in &eq.args {
            collect_typenode_type_refs(arg, target, refs);
        }
        collect_typenode_type_refs(&eq.value, target, refs);
    }
}

// Collect type references in a TypeDefn.
fn collect_typedecl_type_refs(td: &TypeDefn, target: &TyCon, refs: &mut Vec<Span>) {
    match &td.value {
        TypeDeclValue::Struct(s) => {
            for field in &s.fields {
                collect_typenode_type_refs(&field.ty, target, refs);
            }
        }
        TypeDeclValue::Union(u) => {
            for field in &u.fields {
                collect_typenode_type_refs(&field.ty, target, refs);
            }
        }
        TypeDeclValue::Alias(a) => {
            collect_typenode_type_refs(&a.value, target, refs);
        }
    }
}

// Collect type references in a QualType (used for trait members).
fn collect_qualtype_type_refs(qt: &QualType, target: &TyCon, refs: &mut Vec<Span>) {
    collect_typenode_type_refs(&qt.ty, target, refs);
    for pred in &qt.preds {
        collect_typenode_type_refs(&pred.ty, target, refs);
    }
    for eq in &qt.eqs {
        for arg in &eq.args {
            collect_typenode_type_refs(arg, target, refs);
        }
        collect_typenode_type_refs(&eq.value, target, refs);
    }
}

// Collect trait references in a Scheme (type signature).
fn collect_scheme_trait_refs(scheme: &Arc<Scheme>, target: &TraitId, refs: &mut Vec<Span>) {
    for pred in &scheme.predicates {
        if &pred.trait_id == target {
            if let Some(span) = &pred.trait_src {
                refs.push(span.clone());
            }
        }
    }
}

// Collect trait references in a QualPred.
fn collect_qualpred_trait_refs(qp: &QualPred, target: &TraitId, refs: &mut Vec<Span>) {
    if &qp.predicate.trait_id == target {
        if let Some(span) = &qp.predicate.trait_src {
            refs.push(span.clone());
        }
    }
    for pred in &qp.pred_constraints {
        if &pred.trait_id == target {
            if let Some(span) = &pred.trait_src {
                refs.push(span.clone());
            }
        }
    }
}

// Collect trait references in a QualType (used for trait members).
fn collect_qualtype_trait_refs(qt: &QualType, target: &TraitId, refs: &mut Vec<Span>) {
    for pred in &qt.preds {
        if &pred.trait_id == target {
            if let Some(span) = &pred.trait_src {
                refs.push(span.clone());
            }
        }
    }
}

// A reference to a struct field or union variant. `prefix` records the
// literal text that precedes the field/variant name at the source-level
// occurrence: "" for a bare-name occurrence (declaration, MakeStruct
// field, Pattern::Struct/Union), the auto-method's literal prefix
// (`@`/`set_`/`mod_`/`act_`/`as_`/`is_`) for an auto-method call site, or
// `^` for an `act_` Var desugared from `[^field]` index syntax.
pub(super) struct FieldOccurrence {
    pub span: Span,
    pub prefix: &'static str,
}

// Generate the (prefix, fullname) pairs for each user-callable auto-method
// of a struct field or union variant. Internal helpers like
// `_act_x_identity` and `#punch_x` are intentionally excluded (they are
// not user-visible). Only entries that actually exist in
// `program.global_values` are returned.
fn auto_methods_for(
    program: &Program,
    tc: &TyCon,
    name: &Name,
) -> Vec<(&'static str, FullName)> {
    let Some(td) = program.type_defns.iter().find(|td| td.tycon() == *tc) else {
        return vec![];
    };
    let prefixes: &[&'static str] = match &td.value {
        TypeDeclValue::Struct(_) => &[
            STRUCT_GETTER_SYMBOL,
            STRUCT_SETTER_SYMBOL,
            STRUCT_MODIFIER_SYMBOL,
            STRUCT_ACT_SYMBOL,
        ],
        // Union: bare variant name is the constructor; `as_/is_/mod_` are
        // helper functions.
        TypeDeclValue::Union(_) => &["", UNION_AS_SYMBOL, UNION_IS_SYMBOL, UNION_MOD_SYMBOL],
        TypeDeclValue::Alias(_) => return vec![],
    };
    let ns = td.name.to_namespace();
    let mut result = vec![];
    for &prefix in prefixes {
        let fullname = FullName::new(&ns, &format!("{}{}", prefix, name));
        if program.global_values.contains_key(&fullname) {
            result.push((prefix, fullname));
        }
    }
    result
}

// Collect all references to a struct field or union variant. The result
// covers: the bare-name declaration span (when `include_declaration` is
// true, with `prefix = ""`), every Var occurrence of each user-callable
// auto-method (`@x`, `set_x`, ...), each import-leaf naming such an
// auto-method, and bare field-name use sites in `MakeStruct`,
// `Pattern::Struct`, and `Pattern::Union`.
pub(super) fn find_field_occurrences(
    program: &Program,
    tc: &TyCon,
    name: &Name,
    include_declaration: bool,
) -> Vec<FieldOccurrence> {
    let mut occs: Vec<FieldOccurrence> = vec![];

    // (1) Declaration span.
    if include_declaration {
        if let Some(td) = program.type_defns.iter().find(|td| td.tycon() == *tc) {
            let fields: Option<&[Field]> = match &td.value {
                TypeDeclValue::Struct(s) => Some(&s.fields),
                TypeDeclValue::Union(u) => Some(&u.fields),
                TypeDeclValue::Alias(_) => None,
            };
            if let Some(fields) = fields {
                if let Some(f) = fields.iter().find(|f| &f.name == name) {
                    if let Some(span) = &f.name_src {
                        occs.push(FieldOccurrence {
                            span: span.clone(),
                            prefix: "",
                        });
                    }
                }
            }
        }
    }

    // (2) For each user-callable auto-method, walk all Var nodes in the
    // program and emit an occurrence at each matching call site, plus any
    // import-leaf that names that auto-method.
    for (prefix, fullname) in auto_methods_for(program, tc, name) {
        for (_n, gv) in &program.global_values {
            collect_field_var_occs(&gv.expr, prefix, &fullname, &mut occs);
        }
        collect_field_import_occs(program, prefix, &fullname, &mut occs);
    }

    // (3) Bare field-name uses in MakeStruct, Pattern::Struct, Pattern::Union.
    for (_n, gv) in &program.global_values {
        collect_field_bare_occs(&gv.expr, tc, name, &mut occs);
    }

    occs
}

// Walk a SymbolExpr and emit occurrences at every bare field-name source
// position: MakeStruct field-name spans (struct case), Pattern::Struct
// field-name spans (struct case), and Pattern::Union variant-name spans
// (union case). The TyCon stored in the expression/pattern node must match
// `tc` for an occurrence to be emitted.
fn collect_field_bare_occs(
    expr: &SymbolExpr,
    tc: &TyCon,
    name: &Name,
    occs: &mut Vec<FieldOccurrence>,
) {
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            collect_exprnode_bare_field_occs(&typed_expr.expr, tc, name, occs);
        }
        SymbolExpr::Method(impls) => {
            for impl_ in impls {
                collect_exprnode_bare_field_occs(&impl_.expr.expr, tc, name, occs);
            }
        }
    }
}

// Recursive walker for `collect_field_bare_occs` over an expression tree.
fn collect_exprnode_bare_field_occs(
    expr: &Arc<ExprNode>,
    tc: &TyCon,
    name: &Name,
    occs: &mut Vec<FieldOccurrence>,
) {
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_exprnode_bare_field_occs(func, tc, name, occs);
            for arg in args {
                collect_exprnode_bare_field_occs(arg, tc, name, occs);
            }
        }
        Expr::Lam(_, body) => {
            collect_exprnode_bare_field_occs(body, tc, name, occs);
        }
        Expr::Let(pat, bound, val) => {
            collect_pattern_bare_field_occs(pat, tc, name, occs);
            collect_exprnode_bare_field_occs(bound, tc, name, occs);
            collect_exprnode_bare_field_occs(val, tc, name, occs);
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_exprnode_bare_field_occs(cond, tc, name, occs);
            collect_exprnode_bare_field_occs(then_expr, tc, name, occs);
            collect_exprnode_bare_field_occs(else_expr, tc, name, occs);
        }
        Expr::Match(cond, pat_vals) => {
            collect_exprnode_bare_field_occs(cond, tc, name, occs);
            for (pat, val) in pat_vals {
                collect_pattern_bare_field_occs(pat, tc, name, occs);
                collect_exprnode_bare_field_occs(val, tc, name, occs);
            }
        }
        Expr::TyAnno(e, _) => {
            collect_exprnode_bare_field_occs(e, tc, name, occs);
        }
        Expr::MakeStruct(expr_tc, fields) => {
            if expr_tc.as_ref() == tc {
                for (fname, fname_src, _) in fields {
                    if fname == name {
                        if let Some(span) = fname_src {
                            occs.push(FieldOccurrence {
                                span: span.clone(),
                                prefix: "",
                            });
                        }
                    }
                }
            }
            for (_, _, val) in fields {
                collect_exprnode_bare_field_occs(val, tc, name, occs);
            }
        }
        Expr::ArrayLit(elems) => {
            for elem in elems {
                collect_exprnode_bare_field_occs(elem, tc, name, occs);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for arg in args {
                collect_exprnode_bare_field_occs(arg, tc, name, occs);
            }
        }
        Expr::Eval(side, main) => {
            collect_exprnode_bare_field_occs(side, tc, name, occs);
            collect_exprnode_bare_field_occs(main, tc, name, occs);
        }
    }
}

// Recursive walker for `collect_field_bare_occs` over a pattern tree.
fn collect_pattern_bare_field_occs(
    pat: &Arc<PatternNode>,
    tc: &TyCon,
    name: &Name,
    occs: &mut Vec<FieldOccurrence>,
) {
    match &pat.pattern {
        Pattern::Var(_, _) => {}
        Pattern::Struct(pat_tc, fields) => {
            if pat_tc.as_ref() == tc {
                for (fname, fname_src, _) in fields {
                    if fname == name {
                        if let Some(span) = fname_src {
                            occs.push(FieldOccurrence {
                                span: span.clone(),
                                prefix: "",
                            });
                        }
                    }
                }
            }
            for (_, _, sub) in fields {
                collect_pattern_bare_field_occs(sub, tc, name, occs);
            }
        }
        Pattern::Union(variant, variant_src, sub) => {
            // The variant FullName has the namespace `tc.name::*`.
            // Compare via the TyCon constructed from the variant's namespace.
            // After elaboration the namespace is populated (validate_variant_name
            // sets it from the matched union); guard against the unresolved
            // pre-elaboration shape just in case.
            if !variant.namespace.names.is_empty() {
                let variant_tc = TyCon::new(variant.namespace.clone().to_fullname());
                if &variant_tc == tc && &variant.name == name {
                    if let Some(span) = variant_src {
                        occs.push(FieldOccurrence {
                            span: span.clone(),
                            prefix: "",
                        });
                    }
                }
            }
            collect_pattern_bare_field_occs(sub, tc, name, occs);
        }
    }
}

// Walk a SymbolExpr's expression trees and emit a `FieldOccurrence` for
// every Var whose `FullName` equals `target`, tagging it with `prefix`.
fn collect_field_var_occs(
    expr: &SymbolExpr,
    prefix: &'static str,
    target: &FullName,
    occs: &mut Vec<FieldOccurrence>,
) {
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            collect_exprnode_field_occs(&typed_expr.expr, prefix, target, occs);
        }
        SymbolExpr::Method(impls) => {
            for impl_ in impls {
                collect_exprnode_field_occs(&impl_.expr.expr, prefix, target, occs);
            }
        }
    }
}

// Recursive walker for `collect_field_var_occs` over an expression tree.
fn collect_exprnode_field_occs(
    expr: &Arc<ExprNode>,
    prefix: &'static str,
    target: &FullName,
    occs: &mut Vec<FieldOccurrence>,
) {
    match &*expr.expr {
        Expr::Var(v) => {
            if &v.name == target {
                if let Some(span) = &expr.source {
                    // For `act_<field>` Vars desugared from `[^field]`,
                    // the source span covers `^field` and the source-level
                    // prefix is `^` rather than `act_`.
                    let effective_prefix = if prefix == STRUCT_ACT_SYMBOL
                        && expr.struct_act_func_in_index_syntax
                    {
                        "^"
                    } else {
                        prefix
                    };
                    occs.push(FieldOccurrence {
                        span: span.clone(),
                        prefix: effective_prefix,
                    });
                }
            }
        }
        Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_exprnode_field_occs(func, prefix, target, occs);
            for arg in args {
                collect_exprnode_field_occs(arg, prefix, target, occs);
            }
        }
        Expr::Lam(_, body) => {
            collect_exprnode_field_occs(body, prefix, target, occs);
        }
        Expr::Let(_pat, bound, val) => {
            collect_exprnode_field_occs(bound, prefix, target, occs);
            collect_exprnode_field_occs(val, prefix, target, occs);
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_exprnode_field_occs(cond, prefix, target, occs);
            collect_exprnode_field_occs(then_expr, prefix, target, occs);
            collect_exprnode_field_occs(else_expr, prefix, target, occs);
        }
        Expr::Match(cond, pat_vals) => {
            collect_exprnode_field_occs(cond, prefix, target, occs);
            for (_, val) in pat_vals {
                collect_exprnode_field_occs(val, prefix, target, occs);
            }
        }
        Expr::TyAnno(e, _) => {
            collect_exprnode_field_occs(e, prefix, target, occs);
        }
        Expr::MakeStruct(_, fields) => {
            for (_, _, val) in fields {
                collect_exprnode_field_occs(val, prefix, target, occs);
            }
        }
        Expr::ArrayLit(elems) => {
            for elem in elems {
                collect_exprnode_field_occs(elem, prefix, target, occs);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for arg in args {
                collect_exprnode_field_occs(arg, prefix, target, occs);
            }
        }
        Expr::Eval(side, main) => {
            collect_exprnode_field_occs(side, prefix, target, occs);
            collect_exprnode_field_occs(main, prefix, target, occs);
        }
    }
}

// Emit a `FieldOccurrence` for each import-statement leaf that names the
// auto-method `target`, tagging it with `prefix`.
fn collect_field_import_occs(
    program: &Program,
    prefix: &'static str,
    target: &FullName,
    occs: &mut Vec<FieldOccurrence>,
) {
    if target.namespace.names.is_empty() {
        return;
    }
    let target_module = target.module();
    let target_ns_after_module: &[Name] = &target.namespace.names[1..];
    let target_name = &target.name;

    for (_importer, stmts) in &program.mod_to_import_stmts {
        for stmt in stmts {
            if stmt.module.0 != target_module {
                continue;
            }
            for item in &stmt.items {
                walk_import_node_for_field(
                    item,
                    &[],
                    target_ns_after_module,
                    target_name,
                    prefix,
                    occs,
                );
            }
            for item in &stmt.hiding {
                walk_import_node_for_field(
                    item,
                    &[],
                    target_ns_after_module,
                    target_name,
                    prefix,
                    occs,
                );
            }
        }
    }
}

// Recursive walker for `collect_field_import_occs` over a single
// import-tree node, with `traversed` accumulating the namespace path so
// far.
fn walk_import_node_for_field(
    node: &ImportTreeNode,
    traversed: &[Name],
    target_ns: &[Name],
    target_name: &str,
    prefix: &'static str,
    occs: &mut Vec<FieldOccurrence>,
) {
    match node {
        ImportTreeNode::Any(_) => {}
        ImportTreeNode::Symbol(name, span) => {
            if traversed == target_ns && name == target_name {
                if let Some(span) = span {
                    occs.push(FieldOccurrence {
                        span: span.clone(),
                        prefix,
                    });
                }
            }
        }
        ImportTreeNode::TypeOrTrait(_, _) => {}
        ImportTreeNode::NameSpace(ns_name, children, _) => {
            let mut new_traversed = traversed.to_vec();
            new_traversed.push(ns_name.clone());
            for child in children {
                walk_import_node_for_field(
                    child,
                    &new_traversed,
                    target_ns,
                    target_name,
                    prefix,
                    occs,
                );
            }
        }
    }
}

// Collect spans of import-statement leaves that refer to `target`.
//
// `is_value`: when true, match `ImportTreeNode::Symbol` (lowercase value
// names like `bar` or `act_x`); when false, match
// `ImportTreeNode::TypeOrTrait` (uppercase type/trait names).
//
// Wildcards (`Any(*)`) are not matched since they do not name `target`
// explicitly.
fn collect_import_refs(
    program: &Program,
    target: &FullName,
    is_value: bool,
    refs: &mut Vec<Span>,
) {
    if target.namespace.names.is_empty() {
        return;
    }
    let target_module = target.module();
    let target_ns_after_module: &[Name] = &target.namespace.names[1..];
    let target_name = &target.name;

    for (_importer, stmts) in &program.mod_to_import_stmts {
        for stmt in stmts {
            if stmt.module.0 != target_module {
                continue;
            }
            for item in &stmt.items {
                walk_import_node_for_refs(
                    item,
                    &[],
                    target_ns_after_module,
                    target_name,
                    is_value,
                    refs,
                );
            }
            for item in &stmt.hiding {
                walk_import_node_for_refs(
                    item,
                    &[],
                    target_ns_after_module,
                    target_name,
                    is_value,
                    refs,
                );
            }
        }
    }
}

fn walk_import_node_for_refs(
    node: &ImportTreeNode,
    traversed: &[Name],
    target_ns: &[Name],
    target_name: &str,
    is_value: bool,
    refs: &mut Vec<Span>,
) {
    match node {
        ImportTreeNode::Any(_) => {}
        ImportTreeNode::Symbol(name, span) => {
            if !is_value {
                return;
            }
            if traversed == target_ns && name == target_name {
                if let Some(span) = span {
                    refs.push(span.clone());
                }
            }
        }
        ImportTreeNode::TypeOrTrait(name, span) => {
            if is_value {
                return;
            }
            if traversed == target_ns && name == target_name {
                if let Some(span) = span {
                    refs.push(span.clone());
                }
            }
        }
        ImportTreeNode::NameSpace(name, children, _span) => {
            let mut new_traversed = traversed.to_vec();
            new_traversed.push(name.clone());
            for child in children {
                walk_import_node_for_refs(
                    child,
                    &new_traversed,
                    target_ns,
                    target_name,
                    is_value,
                    refs,
                );
            }
        }
    }
}

// Handle "textDocument/prepareCallHierarchy" method.
pub(super) fn handle_call_hierarchy_prepare(
    id: u32,
    params: &CallHierarchyPrepareParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) {
    // Resolve the cursor into a source position, then look up the AST node.
    let Some(pos) = resolve_source_pos(
        &params.text_document_position_params,
        program,
        uri_to_content,
    ) else {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyItem>>));
        return;
    };
    let Some(node) = program.find_node_at(&pos) else {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyItem>>));
        return;
    };

    // Only support call hierarchy for global values (functions).
    let full_name = match &node {
        EndNode::Expr(var, _) if var.name.is_global() => var.name.clone(),
        EndNode::ValueDecl(name) => name.clone(),
        _ => {
            send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyItem>>));
            return;
        }
    };

    let item = create_call_hierarchy_item(&full_name, program);
    match item {
        Some(item) => send_response(id, Ok::<_, ()>(vec![item])),
        None => send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyItem>>)),
    }
}

// Handle "callHierarchy/incomingCalls" method.
pub(super) fn handle_call_hierarchy_incoming(
    id: u32,
    params: &CallHierarchyIncomingCallsParams,
    program: &Program,
) {
    // Extract the target function name from the CallHierarchyItem data.
    let target_name = match &params.item.data {
        Some(data) => {
            let name: Result<String, _> = serde_json::from_value(data.clone());
            match name {
                Ok(s) => FullName::parse(&s),
                Err(_) => None,
            }
        }
        None => None,
    };

    if target_name.is_none() {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyIncomingCall>>));
        return;
    }
    let target_name = target_name.unwrap();

    let mut incoming_calls: Vec<CallHierarchyIncomingCall> = vec![];

    // For each global value, check if its expression references the target.
    for (caller_name, gv) in &program.global_values {
        if caller_name.to_string().contains('#') {
            continue;
        }
        let mut call_spans = vec![];
        collect_symbol_expr_var_refs(&gv.expr, &target_name, &mut call_spans);
        if !call_spans.is_empty() {
            if let Some(caller_item) = create_call_hierarchy_item(caller_name, program) {
                let from_ranges: Vec<lsp_types::Range> = call_spans
                    .iter()
                    .map(|span| span_to_range(span))
                    .collect();
                incoming_calls.push(CallHierarchyIncomingCall {
                    from: caller_item,
                    from_ranges,
                });
            }
        }
    }

    send_response(id, Ok::<_, ()>(incoming_calls));
}

// Handle "callHierarchy/outgoingCalls" method.
pub(super) fn handle_call_hierarchy_outgoing(
    id: u32,
    params: &CallHierarchyOutgoingCallsParams,
    program: &Program,
) {
    // Extract the source function name from the CallHierarchyItem data.
    let source_name = match &params.item.data {
        Some(data) => {
            let name: Result<String, _> = serde_json::from_value(data.clone());
            match name {
                Ok(s) => FullName::parse(&s),
                Err(_) => None,
            }
        }
        None => None,
    };

    if source_name.is_none() {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyOutgoingCall>>));
        return;
    }
    let source_name = source_name.unwrap();

    // Get the expression of the source function.
    let gv = program.global_values.get(&source_name);
    if gv.is_none() {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyOutgoingCall>>));
        return;
    }
    let gv = gv.unwrap();

    // Collect all global function calls from this function's expression.
    let called_names = collect_symbol_expr_called_globals(&gv.expr);

    let mut outgoing_calls: Vec<CallHierarchyOutgoingCall> = vec![];

    for (callee_name, call_spans) in &called_names {
        if callee_name.to_string().contains('#') {
            continue;
        }
        if let Some(callee_item) = create_call_hierarchy_item(callee_name, program) {
            let from_ranges: Vec<lsp_types::Range> = call_spans
                .iter()
                .map(|span| span_to_range(span))
                .collect();
            outgoing_calls.push(CallHierarchyOutgoingCall {
                to: callee_item,
                from_ranges,
            });
        }
    }

    send_response(id, Ok::<_, ()>(outgoing_calls));
}

// Create a CallHierarchyItem for a global value.
fn create_call_hierarchy_item(
    full_name: &FullName,
    program: &Program,
) -> Option<CallHierarchyItem> {
    let gv = program.global_values.get(full_name)?;
    let span = gv.decl_src.as_ref()?;

    let cdir = get_current_dir()?;
    let uri = path_to_uri(&cdir.join(&span.input.file_path)).ok()?;

    let range = span_to_range(span);
    let detail = gv
        .syn_scm
        .clone()
        .unwrap_or(gv.scm.clone())
        .to_string_normalize();

    Some(CallHierarchyItem {
        name: full_name.to_string(),
        kind: SymbolKind::FUNCTION,
        tags: None,
        detail: Some(detail),
        uri,
        range: range.clone(),
        selection_range: range,
        data: Some(serde_json::to_value(full_name.to_string()).unwrap()),
    })
}

// Collect all global function names called from a SymbolExpr, with their call site spans.
// Note: for trait method implementations (SymbolExpr::Method), outgoing calls are intentionally
// not searched.
// Searching all known implementations is possible in Fix,
// but it is not standard in other languages and may produce too many results.
fn collect_symbol_expr_called_globals(expr: &SymbolExpr) -> Map<FullName, Vec<Span>> {
    let mut result: Map<FullName, Vec<Span>> = Map::default();
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            collect_exprnode_called_globals(&typed_expr.expr, &mut result);
        }
        SymbolExpr::Method(_) => {
            // Intentionally not supported. See the comment above.
        }
    }
    result
}

// Recursively collect all global function names referenced in an expression tree.
fn collect_exprnode_called_globals(
    expr: &Arc<ExprNode>,
    result: &mut Map<FullName, Vec<Span>>,
) {
    match &*expr.expr {
        Expr::Var(v) => {
            if v.name.is_global() {
                if let Some(span) = &expr.source {
                    result
                        .entry(v.name.clone())
                        .or_insert_with(Vec::new)
                        .push(span.clone());
                }
            }
        }
        Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_exprnode_called_globals(func, result);
            for arg in args {
                collect_exprnode_called_globals(arg, result);
            }
        }
        Expr::Lam(_, body) => {
            collect_exprnode_called_globals(body, result);
        }
        Expr::Let(_, bound, val) => {
            collect_exprnode_called_globals(bound, result);
            collect_exprnode_called_globals(val, result);
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_exprnode_called_globals(cond, result);
            collect_exprnode_called_globals(then_expr, result);
            collect_exprnode_called_globals(else_expr, result);
        }
        Expr::Match(cond, pat_vals) => {
            collect_exprnode_called_globals(cond, result);
            for (_, val) in pat_vals {
                collect_exprnode_called_globals(val, result);
            }
        }
        Expr::TyAnno(e, _) => {
            collect_exprnode_called_globals(e, result);
        }
        Expr::MakeStruct(_, fields) => {
            for (_, _, val) in fields {
                collect_exprnode_called_globals(val, result);
            }
        }
        Expr::ArrayLit(elems) => {
            for elem in elems {
                collect_exprnode_called_globals(elem, result);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for arg in args {
                collect_exprnode_called_globals(arg, result);
            }
        }
        Expr::Eval(side, main) => {
            collect_exprnode_called_globals(side, result);
            collect_exprnode_called_globals(main, result);
        }
    }
}
