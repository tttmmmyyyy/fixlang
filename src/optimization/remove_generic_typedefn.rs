/*
remove-generic-typedefn transform

Overview:
This transformation removes generic type definitions with type parameters from the program (expressions, patterns, and TypeEnv).
However, built-in types Arrow and Array, which do not have "definitions", are excluded from this transformation.

Example:
Suppose we have the following type definitions.
```
type [f : (* -> *) -> *] Foo f = struct { data : f IO };
type [f : * -> *] Bar f = struct { data : f () };
```
When the type `Foo Bar` appears in the program:
- Define `type #RGT<Foo Bar> = struct { data : Bar IO };`
- Define `type #RGT<Bar IO> = struct { data : IO () };`
And replace usages of `Foo Bar` and `Bar IO` with `#RGT<Foo Bar>` and `#RGT<Bar IO>` respectively.

Purpose:
- This transformation eliminates type variables with higher-kinded types from the program, simplifying the implementation of subsequent optimizations.
- This transformation is a prerequisite for applying the unwrap-newtype optimization. See the "unwrap-newtype.rs" for details.
*/

use std::sync::Arc;

use crate::{
    ast::{
        expr::{expr_let_typed, expr_match_typed, Expr, ExprNode},
        name::FullName,
        pattern::{Pattern, PatternInfo, PatternNode},
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        typedecl::Field,
        types::{kind_star, tycon, type_tyapp, type_tycon, TyCon, TyConInfo, TypeNode},
    },
    misc::{Map, Set},
};

pub fn run(prg: &mut Program) {
    // Run on all symbols.
    let mut env = prg.type_env.tycons.as_ref().clone();
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &mut env);
    }
    // Run on type environment.
    run_on_type_env(&mut env);

    prg.type_env.tycons = Arc::new(env);
}

fn run_on_type_env(env: &mut Map<TyCon, TyConInfo>) {
    let mut todo = Set::default();
    for (tc, _ti) in env.iter() {
        if is_subject_to_removal(&tc, env) {
            // Skip processing types that are scheduled for removal
            continue;
        }
        todo.insert(tc.clone());
    }
    while todo.len() > 0 {
        // Apply remove_generic_type_on_type to the right-hand side of the type definition
        for tc in &todo {
            let mut ti = env.get(tc).unwrap().clone();
            for field in &mut ti.fields {
                field.ty = remove_generic_type_on_type(&field.ty, env);
            }
            env.insert(tc.clone(), ti);
        }
        // Detect newly added types in the above loop
        let mut new_todo = Set::default();
        for (tc, _ti) in env.iter() {
            if todo.contains(tc) {
                continue;
            }
            if is_subject_to_removal(&tc, env) {
                // Skip processing types that are scheduled for removal
                continue;
            }
            new_todo.insert(tc.clone());
        }
        // Repeat the process for newly added types
        todo = new_todo;
    }
    // Remove types that are no longer needed
    let mut to_remove = vec![];
    for (tc, _ti) in env.iter() {
        if is_subject_to_removal(&tc, env) {
            to_remove.push(tc.clone());
        }
    }
    for tc in to_remove {
        env.remove(&tc);
    }
}

fn run_on_symbol(sym: &mut Symbol, env: &mut Map<TyCon, TyConInfo>) {
    let mut remover = RGT { env: env };
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.ty = remove_generic_type_on_type(&sym.ty, env);
        sym.expr = Some(res.expr);
    }
}

fn is_subject_to_removal(tc: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    if tc.is_array() || tc.is_arrow() {
        return false;
    }
    let ti = env.get(tc).unwrap();
    ti.tyvars.len() > 0
}

fn remove_generic_type_on_type(
    ty: &Arc<TypeNode>,
    env: &mut Map<TyCon, TyConInfo>,
) -> Arc<TypeNode> {
    assert!(ty.free_vars_vec().is_empty());
    let top_tc = ty.toplevel_tycon().as_ref().unwrap().clone();
    let top_ti = env.get(top_tc.as_ref()).unwrap();
    let is_fully_applied = top_ti.tyvars.len() == ty.collect_type_argments().len();
    assert!(
        is_fully_applied,
        "remove_generic_type_on_type called on a type `{}` which is not fully applied.",
        ty.to_string()
    );
    if !is_subject_to_removal(&top_tc, env) {
        let mut app_cmps = ty.flatten_type_application();
        if app_cmps.len() <= 1 {
            return ty.clone();
        }
        let fun = app_cmps.remove(0);
        let mut args = app_cmps;
        for arg in &mut args {
            *arg = remove_generic_type_on_type(arg, env);
        }
        let mut res = fun;
        for arg in args {
            res = type_tyapp(res, arg);
        }
        return res;
    }
    let top_ti = env.get(top_tc.as_ref()).unwrap().clone();
    let name = format!("#RGT<{}>", ty.to_string());
    let mut new_tc = top_tc.as_ref().clone();
    *new_tc.name.name_as_mut() = name;

    let mut new_ti = TyConInfo {
        kind: kind_star(),
        variant: top_ti.variant.clone(),
        is_unbox: top_ti.is_unbox,
        tyvars: vec![],
        fields: vec![],
        source: top_ti.source.clone(),
        document: top_ti.document.clone(),
    };
    let mut field_types = ty.field_types_via_tycons(env);
    for field_type in &mut field_types {
        *field_type = remove_generic_type_on_type(field_type, env);
    }
    for (i, field) in top_ti.fields.iter().enumerate() {
        let new_field = Field {
            name: field.name.clone(),
            ty: field_types[i].clone(),
            syn_ty: field.syn_ty.clone(),
            is_punched: field.is_punched,
            source: field.source.clone(),
        };
        new_ti.fields.push(new_field);
    }
    env.insert(new_tc.clone(), new_ti.clone());
    return type_tycon(&tycon(new_tc.name));
}

fn remove_generic_type_on_pattern(
    pat: &Arc<PatternNode>,
    env: &mut Map<TyCon, TyConInfo>,
) -> Arc<PatternNode> {
    match &pat.pattern {
        Pattern::Var(v, ty) => {
            let ty = ty.as_ref().map(|ty| remove_generic_type_on_type(ty, env));
            let mut info = pat.info.clone();
            remove_generic_type_on_pattern_info(&mut info, env);
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), ty),
                info,
            })
        }
        Pattern::Struct(_tc, field_to_pat) => {
            let mut info = pat.info.clone();
            remove_generic_type_on_pattern_info(&mut info, env);
            let new_tc = info.type_.as_ref().unwrap().toplevel_tycon().unwrap();
            let mut field_to_pat = field_to_pat.clone();
            for (_field, subpat) in &mut field_to_pat {
                *subpat = remove_generic_type_on_pattern(subpat, env);
            }
            Arc::new(PatternNode {
                pattern: Pattern::Struct(new_tc.clone(), field_to_pat),
                info,
            })
        }
        Pattern::Union(variant, subpat) => {
            let mut info = pat.info.clone();
            remove_generic_type_on_pattern_info(&mut info, env);
            let tc = info
                .type_
                .as_ref()
                .unwrap()
                .toplevel_tycon()
                .unwrap()
                .name
                .clone()
                .to_namespace();
            let variant = FullName::new(&tc, &variant.name.clone());
            Arc::new(PatternNode {
                pattern: Pattern::Union(variant, remove_generic_type_on_pattern(subpat, env)),
                info,
            })
        }
    }
}

fn remove_generic_type_on_pattern_info(
    pat_info: &mut PatternInfo,
    env: &mut Map<TyCon, TyConInfo>,
) {
    if let Some(ty) = &mut pat_info.type_ {
        *ty = remove_generic_type_on_type(ty, env);
    }
}

struct RGT<'a> {
    env: &'a mut Map<TyCon, TyConInfo>,
}

fn run_on_inferred_type(expr: &Arc<ExprNode>, env: &mut Map<TyCon, TyConInfo>) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = remove_generic_type_on_type(type_, env);
    expr.set_type(type_)
}

impl<'a> ExprVisitor for RGT<'a> {
    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);

        let ty = expr.get_tyanno_ty();
        let ty = remove_generic_type_on_type(&ty, &mut self.env);
        let expr = expr.set_tyanno_ty(ty);

        EndVisitResult::changed(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = run_on_inferred_type(&expr, &mut self.env);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = remove_generic_type_on_pattern(pat, &mut self.env);
            expr = expr_let_typed(pat, body.clone(), val.clone());
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let mut expr = run_on_inferred_type(&expr, &mut self.env);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| {
                    let pat = remove_generic_type_on_pattern(pat, &mut self.env);
                    (pat, arm_expr.clone())
                })
                .collect();
            expr = expr_match_typed(scrut.clone(), arms);
        } else {
            unreachable!()
        }
        EndVisitResult::changed(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        let new_tc = expr.type_.as_ref().unwrap().toplevel_tycon().unwrap();
        let expr = expr.set_make_struct_tycon(new_tc.clone());
        EndVisitResult::changed(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }
}
