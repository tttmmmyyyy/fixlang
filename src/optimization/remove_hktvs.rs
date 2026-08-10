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
use std::sync::Arc;

struct Env {
    tycons: Map<TyCon, TyConInfo>,
    removed_tycons: Set<TyCon>,
}

impl Env {
    fn new(tycons: Map<TyCon, TyConInfo>) -> Self {
        let removed_tycons = calculate_removed_tycons(&tycons);
        Self {
            tycons,
            removed_tycons,
        }
    }

    fn is_removed(&self, tycon: &TyCon) -> bool {
        self.removed_tycons.contains(tycon)
    }
}

// The type constructors a declaration names: those appearing in the type of one of its fields, and,
// for a declaration holding a struct with one field punched out, that struct.
//
// A declaration written in terms of a type constructor stands or falls with it, which is the edge
// both `calculate_removed_tycons` and `assert_every_named_tycon_is_declared` are about, so they read
// it from here and agree on what naming is.
fn named_tycons(ti: &TyConInfo) -> Set<TyCon> {
    let mut named_tycons = Set::default();
    for field in &ti.fields {
        field.ty.collect_tycons(&mut named_tycons);
    }
    if let Some(struct_tc) = &ti.punched_from {
        named_tycons.insert(struct_tc.clone());
    }
    named_tycons
}

// The type constructors this transformation replaces by a copy per list of type arguments: a struct
// or a union declared with a higher-kinded type variable, and a struct or a union that names such a
// declaration, transitively.
//
// A declaration can name itself, directly or through other declarations, so "transitively" is taken
// over a graph that has cycles and the answer is its least fixed point. It is reached by propagating
// backwards along the naming edge, starting from the declarations that carry a higher-kinded type
// variable.
fn calculate_removed_tycons(tycons: &Map<TyCon, TyConInfo>) -> Set<TyCon> {
    // For each type constructor, the struct and union declarations naming it.
    let mut named_by: Map<TyCon, Vec<TyCon>> = Map::default();
    // The declarations to propagate from, which start as those carrying a higher-kinded type
    // variable.
    let mut pending = vec![];
    for (tc, ti) in tycons {
        match ti.variant {
            TyConVariant::Struct | TyConVariant::Union => {}
            _ => {
                continue;
            }
        }
        if ti.tyvars.iter().any(|tv| tv.kind != kind_star()) {
            pending.push(tc.clone());
        }
        for named_tycon in named_tycons(ti) {
            named_by.entry(named_tycon).or_default().push(tc.clone());
        }
    }

    let mut removed_tycons = Set::default();
    while let Some(tc) = pending.pop() {
        if !removed_tycons.insert(tc.clone()) {
            continue;
        }
        if let Some(namers) = named_by.get(&tc) {
            pending.extend(namers.iter().cloned());
        }
    }

    removed_tycons
}

// Every type constructor a remaining declaration names is itself declared.
//
// A declaration that takes type parameters is left as it is written, so the type constructors it
// names have to be ones this transformation keeps. A violation surfaces far from its origin, as a
// later pass looking a declaration up and finding nothing.
fn assert_every_named_tycon_is_declared(tycons: &Map<TyCon, TyConInfo>) {
    for (tc, ti) in tycons {
        for named_tycon in named_tycons(ti) {
            assert!(
                tycons.contains_key(&named_tycon),
                "The declaration of `{}` names `{}`, which is not declared.",
                tc.to_string(),
                named_tycon.to_string()
            );
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

    assert_every_named_tycon_is_declared(&env.tycons);

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
                // The type of a field of such a declaration has the declaration's type parameters
                // free in it, and `run_on_type` takes a type with no free type variable, so the
                // declaration is left as it is written. A declaration naming a type constructor
                // this transformation removes is removed as well, so one that stays names only
                // type constructors that stay, which
                // `assert_every_named_tycon_is_declared` checks.
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
            punched_from: None,
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

        // The copy being made punches the same field of the copy made for the struct at these same
        // type arguments, so it is paired with that one the way their originals are paired. The copy
        // is registered above first, so the walk for the struct stops here if it comes back.
        new_ti.punched_from = top_ti.punched_from.as_ref().map(|struct_tc| {
            let struct_ty = ty.set_toplevel_tycon(Arc::new(struct_tc.clone()));
            run_on_type(&struct_ty, env)
                .toplevel_tycon()
                .unwrap()
                .as_ref()
                .clone()
        });

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
                name_src: field.name_src.clone(),
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
            for (_field, _, subpat) in &mut field_to_pat {
                *subpat = run_on_pattern(subpat, env);
            }
            Arc::new(PatternNode {
                pattern: Pattern::Struct(new_tc.clone(), field_to_pat),
                info,
            })
        }
        Pattern::Union(variant, variant_src, subpat) => {
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
                pattern: Pattern::Union(variant, variant_src.clone(), run_on_pattern(subpat, env)),
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
