// This module implements "Find All References" and "Call Hierarchy" LSP features.

use super::server::{send_response, LatestContent};
use super::util::{get_current_dir, get_node_at, path_to_uri, span_to_range, spans_to_locations};
use crate::ast::expr::{Expr, ExprNode};
use crate::ast::name::FullName;
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::program::{Program, SymbolExpr};
use crate::ast::qual_pred::QualPred;
use crate::ast::qual_type::QualType;
use crate::ast::traits::TraitId;
use crate::ast::typedecl::{TypeDeclValue, TypeDefn};
use crate::ast::equality::Equality;
use crate::ast::traits::AssocTypeImpl;
use crate::ast::types::{Scheme, AssocType, Type, TyCon, TypeNode};
use crate::misc::Map;
use crate::EndNode;
use crate::Span;
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
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position,
        program,
        uri_to_content,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<Vec<lsp_types::Location>>));
        return;
    }
    let node = node.unwrap();

    let include_declaration = params.context.include_declaration;

    // Collect all reference spans.
    let spans = find_all_references(program, &node, include_declaration);

    // Convert spans to Locations.
    let Some(cdir) = get_current_dir() else {
        send_response(id, Ok::<_, ()>(None::<Vec<lsp_types::Location>>));
        return;
    };

    let locations = spans_to_locations(spans, &cdir);

    send_response(id, Ok::<_, ()>(locations));
}

// Find all references to the entity represented by `node` in the program.
fn find_all_references(program: &Program, node: &EndNode, include_declaration: bool) -> Vec<Span> {
    match node {
        EndNode::Expr(var, _) | EndNode::Pattern(var, _) => {
            let name = &var.name;
            if name.is_local() {
                // Local variables: no cross-file reference search supported.
                return vec![];
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
        EndNode::Module(_) => {
            // Module references are not supported yet.
            vec![]
        }
    }
}

// Find all references to a global value (function/constant).
fn find_global_value_references(
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

    refs
}

// Find all references to a type constructor.
fn find_type_references(
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

    // Walk all global values' type signatures.
    for (_name, gv) in &program.global_values {
        collect_scheme_type_refs(&gv.scm, target, &mut refs);
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

    refs
}

// Find all references to a trait.
fn find_trait_references(
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

    refs
}

// Find all references to an associated type.
fn find_assoc_type_references(
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
                if let Some(span) = &assoc_ty.source {
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
        if let Some(span) = &eq.assoc_type.source {
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
        if let Some(span) = &assoc_impl.source {
            refs.push(span.clone());
        }
    }
    // Also walk the value type for nested associated type references.
    collect_typenode_assoc_type_refs(&assoc_impl.value, target, refs);
}

// Collect variable references in a SymbolExpr.
fn collect_symbol_expr_var_refs(expr: &SymbolExpr, target: &FullName, refs: &mut Vec<Span>) {
    match expr {
        SymbolExpr::Simple(typed_expr) => {
            collect_exprnode_var_refs(&typed_expr.expr, target, refs);
        }
        SymbolExpr::Method(impls) => {
            for impl_ in impls {
                collect_exprnode_var_refs(&impl_.expr.expr, target, refs);
            }
        }
    }
}

// Recursively collect all references to `target` FullName in an expression tree.
fn collect_exprnode_var_refs(expr: &Arc<ExprNode>, target: &FullName, refs: &mut Vec<Span>) {
    match &*expr.expr {
        Expr::Var(v) => {
            if &v.name == target {
                if let Some(span) = &expr.source {
                    refs.push(span.clone());
                }
            }
        }
        Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_exprnode_var_refs(func, target, refs);
            for arg in args {
                collect_exprnode_var_refs(arg, target, refs);
            }
        }
        Expr::Lam(_, body) => {
            collect_exprnode_var_refs(body, target, refs);
        }
        Expr::Let(pat, bound, val) => {
            collect_pattern_var_refs(pat, target, refs);
            collect_exprnode_var_refs(bound, target, refs);
            collect_exprnode_var_refs(val, target, refs);
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_exprnode_var_refs(cond, target, refs);
            collect_exprnode_var_refs(then_expr, target, refs);
            collect_exprnode_var_refs(else_expr, target, refs);
        }
        Expr::Match(cond, pat_vals) => {
            collect_exprnode_var_refs(cond, target, refs);
            for (pat, val) in pat_vals {
                collect_pattern_var_refs(pat, target, refs);
                collect_exprnode_var_refs(val, target, refs);
            }
        }
        Expr::TyAnno(e, _) => {
            collect_exprnode_var_refs(e, target, refs);
        }
        Expr::MakeStruct(_, fields) => {
            for (_, val) in fields {
                collect_exprnode_var_refs(val, target, refs);
            }
        }
        Expr::ArrayLit(elems) => {
            for elem in elems {
                collect_exprnode_var_refs(elem, target, refs);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for arg in args {
                collect_exprnode_var_refs(arg, target, refs);
            }
        }
        Expr::Eval(side, main) => {
            collect_exprnode_var_refs(side, target, refs);
            collect_exprnode_var_refs(main, target, refs);
        }
    }
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
            for (_, sub_pat) in field_pats {
                collect_pattern_var_refs(sub_pat, target, refs);
            }
        }
        Pattern::Union(_, sub_pat) => {
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
                if let Some(span) = &expr.source {
                    refs.push(span.clone());
                }
            }
            for (_, val) in fields {
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
                if let Some(span) = &pat.info.source {
                    refs.push(span.clone());
                }
            }
            for (_, sub_pat) in field_pats {
                collect_pattern_type_refs(sub_pat, target, refs);
            }
        }
        Pattern::Union(_, sub_pat) => {
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
            if let Some(span) = &pred.source {
                refs.push(span.clone());
            }
        }
    }
}

// Collect trait references in a QualPred.
fn collect_qualpred_trait_refs(qp: &QualPred, target: &TraitId, refs: &mut Vec<Span>) {
    if &qp.predicate.trait_id == target {
        if let Some(span) = &qp.predicate.source {
            refs.push(span.clone());
        }
    }
    for pred in &qp.pred_constraints {
        if &pred.trait_id == target {
            if let Some(span) = &pred.source {
                refs.push(span.clone());
            }
        }
    }
}

// Collect trait references in a QualType (used for trait members).
fn collect_qualtype_trait_refs(qt: &QualType, target: &TraitId, refs: &mut Vec<Span>) {
    for pred in &qt.preds {
        if &pred.trait_id == target {
            if let Some(span) = &pred.source {
                refs.push(span.clone());
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
    // Get the node at the cursor position.
    let node = get_node_at(
        &params.text_document_position_params,
        program,
        uri_to_content,
    );
    if node.is_none() {
        send_response(id, Ok::<_, ()>(None::<Vec<CallHierarchyItem>>));
        return;
    }
    let node = node.unwrap();

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
            for (_, val) in fields {
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
