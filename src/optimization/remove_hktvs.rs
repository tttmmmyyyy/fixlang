/*
remove-hktvs transform

Overview:
This transformation removes type constructors with higher-kinded type variables from the program.

Example:
Suppose we have the following type definitions.
```
type [f : (* -> *) -> *] Foo f = struct { data : f IO };
type [f : * -> *] Bar f = struct { data : f () };
```
When the type `Foo Bar` appears in the program:
- Define `type #RHKTV<Foo Bar> = struct { data : Bar IO };`
- Define `type #RHKTV<Bar IO> = struct { data : IO () };`
And replace usages of `Foo Bar` and `Bar IO` with `#RHKTV<Foo Bar>` and `#RHKTV<Bar IO>` respectively.

Purpose:
- This transformation simplifies the implementation of subsequent optimizations.
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
        types::{
            kind_star, tycon, type_tyapp, type_tycon, TyCon, TyConInfo, TyConVariant, TypeNode,
        },
    },
    misc::{Map, Set},
};

struct Env {
    tycons: Map<TyCon, TyConInfo>,
    removed_tycons: Set<TyCon>,
}

impl Env {
    fn new(tycons: Map<TyCon, TyConInfo>) -> Self {
        let mut env = Self {
            tycons,
            removed_tycons: Set::default(),
        };
        let tycons = env.tycons.keys().cloned().collect::<Vec<_>>();
        let mut visited = Set::default();
        for tycon in tycons {
            env.calculate_removed_tycons_internal(&tycon, &mut visited);
        }
        env
    }

    fn is_removed(&self, tycon: &TyCon) -> bool {
        self.removed_tycons.contains(tycon)
    }

    fn calculate_removed_tycons_internal(&mut self, now: &TyCon, visited: &mut Set<TyCon>) {
        if visited.contains(now) {
            return;
        }
        visited.insert(now.clone());
        let ti = self.tycons.get(now).unwrap();
        match ti.variant {
            TyConVariant::Struct | TyConVariant::Union => {}
            _ => {
                return;
            }
        }
        if ti.tyvars.iter().any(|tv| tv.kind != kind_star()) {
            self.removed_tycons.insert(now.clone());
            return;
        }
        let field_tys = ti
            .fields
            .iter()
            .map(|field| field.ty.clone())
            .collect::<Vec<_>>();
        for field_ty in field_tys {
            let mut tycons = Set::default();
            field_ty.collect_tycons(&mut tycons);
            for tycon in tycons {
                self.calculate_removed_tycons_internal(&tycon, visited);
                if self.removed_tycons.contains(&tycon) {
                    self.removed_tycons.insert(now.clone());
                    return;
                }
            }
        }
    }
}

pub fn run(prg: &mut Program) {
    // Run on all symbols.
    let mut env = Env::new(prg.type_env.tycons.as_ref().clone());

    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &mut env);
    }
    run_on_exported_statements(prg, &mut env);
    run_on_entry_io_value(prg, &mut env);

    // Run on type environment.
    run_on_type_env(&mut env);

    prg.type_env.tycons = Arc::new(env.tycons);
}

fn run_on_exported_statements(prg: &mut Program, env: &mut Env) {
    for export in &mut prg.export_statements {
        if let Some(expr) = &export.value_expr {
            let expr = run_on_inferred_type(expr, env);
            export.value_expr = Some(expr);
        }
        if let Some(ft) = &mut export.function_type {
            for dom in &mut ft.doms {
                *dom = run_on_type(dom, env);
            }
            ft.codom = run_on_type(&ft.codom, env);
        }
    }
}

fn run_on_entry_io_value(prg: &mut Program, env: &mut Env) {
    if let Some(entry_io_value) = &mut prg.entry_io_value {
        let expr = run_on_inferred_type(entry_io_value, env);
        prg.entry_io_value = Some(expr);
    }
}

fn run_on_type_env(env: &mut Env) {
    let mut todo = Set::default();
    for (tc, _ti) in env.tycons.iter() {
        todo.insert(tc.clone());
    }
    let mut done = Set::default();
    while todo.len() > 0 {
        // Apply run_on_type to the right-hand side of the type definition
        for tc in &todo {
            done.insert(tc.clone());
            if env.is_removed(tc) {
                // Skip types that are scheduled for removal.
                continue;
            }
            let mut ti = env.tycons.get(tc).unwrap().clone();
            if ti.tyvars.len() > 0 {
                // If there are type variables, we cannot process it.
                continue;
            }
            for field in &mut ti.fields {
                field.ty = run_on_type(&field.ty, env);
            }
            env.tycons.insert(tc.clone(), ti);
        }
        // Detect newly added types in the above loop
        todo.clear();
        for (tc, _ti) in env.tycons.iter() {
            if done.contains(tc) {
                continue;
            }
            todo.insert(tc.clone());
        }
    }
    // Remove types that are no longer needed
    let mut to_remove = vec![];
    for (tc, _ti) in env.tycons.iter() {
        if env.is_removed(&tc) {
            to_remove.push(tc.clone());
        }
    }
    for tc in to_remove {
        env.tycons.remove(&tc);
    }
}

fn run_on_symbol(sym: &mut Symbol, env: &mut Env) {
    let mut remover = RGT { env: env };
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.ty = run_on_type(&sym.ty, env);
        sym.expr = Some(res.expr);
    }
}

// fn is_subject_to_removal(tc: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
//     let mut visited = Set::default();
//     is_subject_to_removal_internal(tc, env, &mut visited)
// }

// fn is_subject_to_removal_internal(
//     tc: &TyCon,
//     env: &Map<TyCon, TyConInfo>,
//     visited: &mut Set<TyCon>,
// ) -> bool {
//     let ti = env.get(tc).unwrap();
//     match ti.variant {
//         TyConVariant::Struct | TyConVariant::Union => {}
//         _ => {
//             return false;
//         }
//     }
//     if ti.tyvars.iter().any(|tv| tv.kind != kind_star()) {
//         return true;
//     }
//     visited.insert(tc.clone());
//     for field in &ti.fields {
//         let field_ty = &field.ty;
//         let mut tycons = Set::default();
//         field_ty.collect_tycons(&mut tycons);
//         for tycon in tycons {
//             if visited.contains(&tycon) {
//                 continue;
//             }
//             if is_subject_to_removal(&tycon, env) {
//                 return true;
//             }
//         }
//     }
//     return false;
// }

fn run_on_type(ty: &Arc<TypeNode>, env: &mut Env) -> Arc<TypeNode> {
    assert!(
        ty.free_vars_vec().is_empty(),
        "A type `{}` with free type variables.",
        ty.to_string()
    );
    let top_tc = ty.toplevel_tycon().as_ref().unwrap().clone();
    let top_ti = env.tycons.get(top_tc.as_ref()).unwrap();
    let is_fully_applied = top_ti.tyvars.len() == ty.collect_type_argments().len();
    assert!(
        is_fully_applied,
        "A type `{}` which is not fully applied.",
        ty.to_string()
    );
    if !env.is_removed(&top_tc) {
        let mut app_cmps = ty.flatten_type_application();
        if app_cmps.len() <= 1 {
            return ty.clone();
        }
        let fun = app_cmps.remove(0);
        let mut args = app_cmps;
        for arg in &mut args {
            *arg = run_on_type(arg, env);
        }
        let mut res = fun;
        for arg in args {
            res = type_tyapp(res, arg);
        }
        return res;
    }
    let top_ti = env.tycons.get(top_tc.as_ref()).unwrap().clone();
    let name = format!("#RHKTV<{}>", ty.to_string());
    let mut new_tc = top_tc.as_ref().clone();
    *new_tc.name.name_as_mut() = name;

    if !env.tycons.contains_key(&new_tc) {
        let mut new_ti = TyConInfo {
            kind: kind_star(),
            variant: top_ti.variant.clone(),
            is_unbox: top_ti.is_unbox,
            tyvars: vec![],
            fields: vec![],
            source: top_ti.source.clone(),
            document: top_ti.document.clone(),
        };
        // Register the new type constructor before processing field types to handle recursive types.
        env.tycons.insert(new_tc.clone(), new_ti.clone());

        let mut field_types = ty.field_types_via_tycons(&env.tycons);
        for field_type in &mut field_types {
            *field_type = run_on_type(field_type, env);
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
        env.tycons.insert(new_tc.clone(), new_ti.clone());
    }

    return type_tycon(&tycon(new_tc.name));
}

fn run_on_pattern(pat: &Arc<PatternNode>, env: &mut Env) -> Arc<PatternNode> {
    match &pat.pattern {
        Pattern::Var(v, ty) => {
            // Ignore the type annotation given by the user.
            let mut info = pat.info.clone();
            run_on_pattern_info(&mut info, env);
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), ty.clone()),
                info,
            })
        }
        Pattern::Struct(_tc, field_to_pat) => {
            let mut info = pat.info.clone();
            run_on_pattern_info(&mut info, env);
            let new_tc = info.type_.as_ref().unwrap().toplevel_tycon().unwrap();
            let mut field_to_pat = field_to_pat.clone();
            for (_field, subpat) in &mut field_to_pat {
                *subpat = run_on_pattern(subpat, env);
            }
            Arc::new(PatternNode {
                pattern: Pattern::Struct(new_tc.clone(), field_to_pat),
                info,
            })
        }
        Pattern::Union(variant, subpat) => {
            let mut info = pat.info.clone();
            run_on_pattern_info(&mut info, env);
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
                pattern: Pattern::Union(variant, run_on_pattern(subpat, env)),
                info,
            })
        }
    }
}

fn run_on_pattern_info(pat_info: &mut PatternInfo, env: &mut Env) {
    if let Some(ty) = &mut pat_info.type_ {
        *ty = run_on_type(ty, env);
    }
}

struct RGT<'a> {
    env: &'a mut Env,
}

fn run_on_inferred_type(expr: &Arc<ExprNode>, env: &mut Env) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = run_on_type(type_, env);
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
            let pat = run_on_pattern(pat, &mut self.env);
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
                    let pat = run_on_pattern(pat, &mut self.env);
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

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let expr = run_on_inferred_type(&expr, &mut self.env);
        EndVisitResult::changed(expr)
    }
}
