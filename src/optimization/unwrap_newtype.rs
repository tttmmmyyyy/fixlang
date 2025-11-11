// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// The boxed struct is not treated as a newtype pattern.
//
// If there is a circular definition of newtype, the optimization is skipped (it will result in an error in the code generation phase).
//
// This optimization should be run after the remove-hk-tyvar transform.
// The unwrap-newtype optimization cannot be applied to programs with generic type definitions such as `type [f : * -> *] Foo f = box struct { data : f () };`.
// This is because if there is an expression with a type like `Foo IO`, `IO` is a partially applied type and cannot be unwrapped.

use crate::{
    ast::{
        export_statement::IOType,
        expr::{expr_let_typed, expr_make_struct, expr_match_typed, expr_var, Expr, ExprNode},
        inline_llvm::LLVMGenerator,
        pattern::{Pattern, PatternInfo, PatternNode},
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{tycon, TyCon, TyConInfo, TyConVariant, Type, TypeNode},
    },
    builtin::{make_tuple_name, make_unit_ty},
    misc::{Map, Set},
};
use std::sync::Arc;

pub fn run(prg: &mut Program) {
    let mut env = prg.type_env.tycons.as_ref().clone();

    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, &env);
    }
    run_on_exported_statements(prg, &env);
    run_on_entry_io_value(prg, &env);
    unwrap_newtype_on_type_env(&mut env);

    prg.type_env.tycons = Arc::new(env);
}

fn run_on_exported_statements(prg: &mut Program, env: &Map<TyCon, TyConInfo>) {
    for export in &mut prg.export_statements {
        if let Some(expr) = &export.value_expr {
            let expr = run_on_inferred_type(expr, env);
            export.value_expr = Some(expr);
        }
        if let Some(ft) = &mut export.function_type {
            if matches!(ft.io_type, IOType::IO) {
                ft.io_type = IOType::IOState;
            }
        }
    }
}

fn run_on_entry_io_value(prg: &mut Program, env: &Map<TyCon, TyConInfo>) {
    if let Some(entry_io_value) = &mut prg.entry_io_value {
        let expr = run_on_inferred_type(entry_io_value, env);
        prg.entry_io_value = Some(expr);
    }
}

fn run_on_symbol(sym: &mut Symbol, env: &Map<TyCon, TyConInfo>) {
    let mut remover = NewtypeUnwrapper { type_env: env };
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.ty = unwrap_newtype_on_type(&sym.ty, env);
        sym.expr = Some(res.expr);
    }
}

fn run_on_inferred_type(expr: &Arc<ExprNode>, type_env: &Map<TyCon, TyConInfo>) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = unwrap_newtype_on_type(type_, type_env);
    expr.set_type(type_)
}

struct NewtypeUnwrapper<'a> {
    type_env: &'a Map<TyCon, TyConInfo>,
}

impl<'a> ExprVisitor for NewtypeUnwrapper<'a> {
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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let expr = run_on_inferred_type(&expr, self.type_env);
        EndVisitResult::changed(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let old_ty = expr.type_.as_ref().unwrap().clone();
        let mut expr = run_on_inferred_type(&expr, self.type_env);
        let new_ty = expr.type_.as_ref().unwrap().clone();

        let llvm = if let Expr::LLVM(llvm) = expr.expr.as_ref() {
            llvm.as_ref().clone()
        } else {
            unreachable!()
        };

        // We don't need to change the LLVM type: TODO: rename llvm.ty to llvm.generic_ty.

        // write_log(&format!(
        //     "UnwrapNewtype: LLVM expr with type {}, old_type = {}, new_type = {}, llvm_name = {}",
        //     llvm.ty.to_string(),
        //     old_ty.to_string(),
        //     new_ty.to_string(),
        //     llvm.generator.name()
        // ));
        // llvm.ty = unwrap_newtype_on_type(&llvm.ty, self.type_env);
        // expr = expr.set_llvm(llvm.clone());

        // Replace StructGetBody, StructSetBody, StructPunchBody, and StructPlugInBody for structures defined by the newtype pattern.
        match &llvm.generator {
            LLVMGenerator::StructGetBody(body) => {
                // @ : S -> F = |s| GetBody(s)
                // =>
                // @ : F -> F = |s| s
                let field_ty = new_ty;
                let struct_name = body.var_name.clone();
                assert!(struct_name.is_local());
                let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
                let struct_tc = struct_ty.toplevel_tycon().unwrap();
                if is_unwrappable_newtype(struct_tc.as_ref(), &self.type_env) {
                    expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
                }
            }
            LLVMGenerator::StructSetBody(body) => {
                // set : F -> S -> S = |f, s| SetBody(f)
                // =>
                // set : F -> F -> F = |f, s| f
                let field_ty = new_ty;
                let struct_ty = old_ty;
                let struct_tc = struct_ty.toplevel_tycon().unwrap();
                if is_unwrappable_newtype(struct_tc.as_ref(), &self.type_env) {
                    let field_name = body.value_name.clone();
                    expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
                }
            }
            LLVMGenerator::StructPunchBody(body) => {
                // punch : S -> (F, S*) = |s| Punch(s)
                // =>
                // punch : F -> (F, ()) = |s| (s, ())
                let field_unit_ty = new_ty;
                let struct_name = body.var_name.clone();
                assert!(struct_name.is_local());
                let struct_ty = state.scope.get_local(&struct_name.name).unwrap().unwrap();
                // let struct_ti = struct_ty.toplevel_tycon_info(&self.type_env);
                let struct_tc = struct_ty.toplevel_tycon().unwrap();
                if is_unwrappable_newtype(struct_tc.as_ref(), &self.type_env) {
                    let field_ty = field_unit_ty.collect_type_argments()[0].clone();
                    let unit_ty = make_unit_ty();
                    let struct_expr = expr_var(struct_name, expr.source.clone()).set_type(field_ty);
                    let unit_expr =
                        expr_make_struct(tycon(make_tuple_name(0)), vec![]).set_type(unit_ty);
                    expr = expr_make_struct(
                        tycon(make_tuple_name(2)),
                        vec![("0".to_string(), struct_expr), ("1".to_string(), unit_expr)],
                    )
                    .set_type(field_unit_ty);
                }
            }
            LLVMGenerator::StructPlugInBody(body) => {
                // plug_in : S* -> F -> S = |s, f| PlugIn(s, f)
                // =>
                // plug_in : () -> F -> F = |_, f| f
                let struct_ty = old_ty;
                let struct_tc = struct_ty.toplevel_tycon().unwrap();
                if is_unwrappable_newtype(struct_tc.as_ref(), &self.type_env) {
                    let field_ty = new_ty;
                    let field_name = body.field_name.clone();
                    assert!(field_name.is_local());
                    expr = expr_var(field_name, expr.source.clone()).set_type(field_ty);
                }
            }
            _ => {}
        }

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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, self.type_env);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = unwrap_newtype_on_pattern(pat, self.type_env);
            // let pat = pat
            //     .get_typed_matching(body.type_.as_ref().unwrap(), &self.type_env)
            //     .unwrap();
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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, self.type_env);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| {
                    let pat = unwrap_newtype_on_pattern(pat, self.type_env);
                    // let pat = pat
                    //     .get_typed_matching(scrut.type_.as_ref().unwrap(), &self.type_env)
                    //     .unwrap();
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
        let mut expr = run_on_inferred_type(&expr, self.type_env);
        if let Expr::MakeStruct(tycon, fields) = expr.expr.as_ref() {
            if is_unwrappable_newtype(tycon, &self.type_env) {
                expr = fields[0].1.clone();
            }
        } else {
            unreachable!()
        }
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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let expr = run_on_inferred_type(&expr, self.type_env);
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
        let expr = run_on_inferred_type(&expr, self.type_env);
        EndVisitResult::changed(expr)
    }
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function detects circular newtype patterns and avoids infinite loops.
//
// This function is supposed to be called after type aliases are resolved.
fn unwrap_newtype_on_type(ty: &Arc<TypeNode>, env: &Map<TyCon, TyConInfo>) -> Arc<TypeNode> {
    unwrap_newtype_on_type_internal(ty, env)
}

// Internal implementation of unwrap_newtype
fn unwrap_newtype_on_type_internal(
    ty: &Arc<TypeNode>,
    env: &Map<TyCon, TyConInfo>,
) -> Arc<TypeNode> {
    // First, replace the top-level type constructor if it is a newtype.
    // As an example, consider type alias `type Foo a = unbox struct { data : () -> a }`.
    // Then `Foo Bool` should be resolved to `() -> Bool`.
    if let Some(top_tc) = ty.toplevel_tycon().clone() {
        let top_ti = env.get(&top_tc).unwrap().clone();
        let is_fully_applied = top_ti.tyvars.len() == ty.collect_type_argments().len();
        if is_fully_applied && is_unwrappable_newtype(&top_tc, env) {
            // If the top-level tycon is a fully applied newtype, unwrap it.
            let ti = env.get(&top_tc).unwrap();
            // Check if this is a punched struct of a newtype pattern
            if ti.fields[0].is_punched {
                // Convert punched struct of newtype pattern to unit type
                return make_unit_ty();
            }
            let field_ty = ty.field_types_via_tycons(env)[0].clone();
            let result = unwrap_newtype_on_type_internal(&field_ty, env);

            return result;
        }
    }

    // If the top-level tycon is not a newtype, recursively process type arguments
    match &ty.ty {
        Type::TyVar(_) => ty.clone(),
        Type::TyCon(_) => ty.clone(),
        Type::TyApp(fun_ty, arg_ty) => ty
            .set_tyapp_fun(unwrap_newtype_on_type_internal(fun_ty, env))
            .set_tyapp_arg(unwrap_newtype_on_type_internal(arg_ty, env)),
        Type::AssocTy(_, _args) => {
            unimplemented!("AssocTy is not supported in unwrap_newtype_on_type")
        }
    }
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function does not detect circular newtype patterns. If a circular newtype pattern is included, it may fall into an infinite loop.
//
// This function is supposed to be called after type aliases are resolved.
fn unwrap_newtype_on_pattern(
    pat: &Arc<PatternNode>,
    env: &Map<TyCon, TyConInfo>,
) -> Arc<PatternNode> {
    match &pat.pattern {
        Pattern::Var(v, ty) => {
            // Ignore user-provided type annotation for variable patterns
            let mut info = pat.info.clone();
            unwrap_newtype_on_pattern_info(&mut info, env);
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), ty.clone()),
                info,
            })
        }
        Pattern::Struct(tc, field_to_pat) => {
            if is_unwrappable_newtype(tc, env) {
                assert_eq!(field_to_pat.len(), 1);
                let (_, pat) = &field_to_pat[0];
                unwrap_newtype_on_pattern(pat, env)
            } else {
                let mut field_to_pat = field_to_pat.clone();
                for (_, pat) in &mut field_to_pat {
                    *pat = unwrap_newtype_on_pattern(pat, env);
                }
                let mut info = pat.info.clone();
                unwrap_newtype_on_pattern_info(&mut info, env);
                Arc::new(PatternNode {
                    pattern: Pattern::Struct(tc.clone(), field_to_pat),
                    info,
                })
            }
        }
        Pattern::Union(variant, subpat) => {
            let mut info = pat.info.clone();
            unwrap_newtype_on_pattern_info(&mut info, env);
            Arc::new(PatternNode {
                pattern: Pattern::Union(variant.clone(), unwrap_newtype_on_pattern(subpat, env)),
                info,
            })
        }
    }
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function does not detect circular newtype patterns. If a circular newtype pattern is included, it may fall into an infinite loop.
//
// This function is supposed to be called after type aliases are resolved.
fn unwrap_newtype_on_pattern_info(pat_info: &mut PatternInfo, env: &Map<TyCon, TyConInfo>) {
    if let Some(ty) = &mut pat_info.type_ {
        *ty = unwrap_newtype_on_type(ty, env);
    }
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function does not detect circular newtype patterns. If a circular newtype pattern is included, it may fall into an infinite loop.
//
// This function is supposed to be called after type aliases are resolved.
pub fn unwrap_newtype_on_type_env(env: &mut Map<TyCon, TyConInfo>) {
    let tycons = env.clone();

    // Unwrap newtype patterns in the remaining types
    for (tycon, mut tycon_info) in tycons.into_iter() {
        for field in &mut tycon_info.fields {
            let new_ty = unwrap_newtype_on_type(&field.ty, env);
            field.ty = new_ty;
        }
        env.insert(tycon, tycon_info);
    }

    // Remove newtype pattern types from the tycons map
    let mut to_remove = Vec::new();
    for (name, _tycon_info) in env.iter() {
        if is_unwrappable_newtype(name, env) {
            to_remove.push(name.clone());
        }
    }
    for name in to_remove {
        env.remove(&name);
    }
}

// Is this type constructor a "newtype", i.e., is it an unbox struct type with only one field?
fn is_newtype(tycon: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    let ti = env.get(tycon).unwrap();
    ti.is_unbox && ti.variant == TyConVariant::Struct && ti.fields.len() == 1
}

// Is this type constructor a newtype and unwrappable?
fn is_unwrappable_newtype(tycon: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    is_unwrappable_newtype_internal(tycon, env)
}

// Is this type constructor a newtype and unwrappable?
fn is_unwrappable_newtype_internal(tc: &TyCon, env: &Map<TyCon, TyConInfo>) -> bool {
    // If this TyCon is not a newtype, return false.
    if !is_newtype(tc, env) {
        return false;
    }

    let mut visited = Set::default();
    let mut seed_tcs = vec![tc.clone()];
    while let Some(now_tc) = seed_tcs.pop() {
        visited.insert(now_tc.clone());
        if !is_newtype(&now_tc, env) {
            continue;
        }
        let ti = env.get(&now_tc).unwrap();
        let field_ty = &ti.fields[0].ty;
        let mut tycons = Set::default();
        field_ty.collect_tycons(&mut tycons);
        for tycon in tycons {
            if tycon == *tc {
                return false;
            }
            if visited.contains(&tycon) {
                continue;
            }
            seed_tcs.push(tycon);
        }
    }

    return true;
}
