// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.

use crate::{
    ast::{
        export_statement::IOType,
        expr::{expr_let_typed, expr_make_struct, expr_match_typed, expr_var, Expr, ExprNode},
        inline_llvm::LLVMGenerator,
        pattern::{Pattern, PatternInfo, PatternNode},
        program::{Program, Symbol, TypeEnv},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        typedecl::Field,
        types::{kind_star, tycon, type_tycon, TyCon, TyConInfo, TyConVariant, Type, TypeNode},
    },
    builtin::{make_tuple_name, make_unit_ty},
    misc::{Map, Set},
};
use std::sync::Arc;

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym, prg.type_env.clone());
    }
    run_on_exported_statements(prg);
    run_on_entry_io_value(prg);
    unwrap_newtype_on_type_env(&mut prg.type_env);
}

fn run_on_exported_statements(prg: &mut Program) {
    for export in &mut prg.export_statements {
        if let Some(expr) = &export.value_expr {
            let expr = run_on_inferred_type(expr, &prg.type_env);
            export.value_expr = Some(expr);
        }
        if let Some(ft) = &mut export.function_type {
            if matches!(ft.io_type, IOType::IO) {
                ft.io_type = IOType::IOState;
            }
        }
    }
}

fn run_on_entry_io_value(prg: &mut Program) {
    if let Some(entry_io_value) = &mut prg.entry_io_value {
        let expr = run_on_inferred_type(entry_io_value, &prg.type_env);
        prg.entry_io_value = Some(expr);
    }
}

fn run_on_symbol(sym: &mut Symbol, type_env: TypeEnv) {
    let mut remover = NewtypeUnwrapper { type_env: type_env };
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.ty = unwrap_newtype_on_type(&sym.ty, &remover.type_env);
        sym.expr = Some(res.expr);
    }
}

fn run_on_inferred_type(expr: &Arc<ExprNode>, type_env: &TypeEnv) -> Arc<ExprNode> {
    let type_ = expr.type_.as_ref().unwrap();
    let type_ = unwrap_newtype_on_type(type_, type_env);
    expr.set_type(type_)
}

struct NewtypeUnwrapper {
    type_env: TypeEnv,
}

impl ExprVisitor for NewtypeUnwrapper {
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
        let expr = run_on_inferred_type(&expr, &self.type_env);

        let ty = expr.get_tyanno_ty();
        let ty = unwrap_newtype_on_type(&ty, &self.type_env);
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
        let expr = run_on_inferred_type(&expr, &self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        let new_ty = expr.type_.as_ref().unwrap().clone();

        let mut llvm = if let Expr::LLVM(llvm) = expr.expr.as_ref() {
            llvm.as_ref().clone()
        } else {
            unreachable!()
        };
        llvm.ty = unwrap_newtype_on_type(&llvm.ty, &self.type_env);
        expr = expr.set_llvm(llvm.clone());

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
        let expr = run_on_inferred_type(&expr, &self.type_env);
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
        let expr = run_on_inferred_type(&expr, &self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        if let Expr::Let(pat, body, val) = expr.expr.as_ref() {
            let pat = unwrap_newtype_on_pattern(pat, &self.type_env);
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
        let expr = run_on_inferred_type(&expr, &self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
        if let Expr::Match(scrut, arms) = expr.expr.as_ref() {
            let arms = arms
                .iter()
                .map(|(pat, arm_expr)| {
                    let pat = unwrap_newtype_on_pattern(pat, &self.type_env);
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
        let mut expr = run_on_inferred_type(&expr, &self.type_env);
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
        let expr = run_on_inferred_type(&expr, &self.type_env);
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
        let expr = run_on_inferred_type(&expr, &self.type_env);
        EndVisitResult::changed(expr)
    }
}

// Determine whether the type is to be replaced with a new type constructor, such as `Foo IO` in the comment of `unwrap_newtype_on_type`.
fn should_converted_to_tycon(ty: &Arc<TypeNode>, env: &TypeEnv) -> Option<(TyCon, TyConInfo)> {
    // This function supposes that `ty` is fully applied.
    let top_tc = ty.toplevel_tycon().unwrap();
    let top_ti = env.tycons.get(&top_tc).unwrap();
    assert_eq!(top_ti.tyvars.len(), ty.collect_type_argments().len());

    // If the top-level type constructor is a newtype pattern, it should not be converted to a new type constructor.
    if is_unwrappable_newtype(&top_tc, env) {
        return None;
    }

    let type_args = ty.collect_type_argments();
    let mut has_arg_partially_applied_newtype = false;
    for arg in type_args {
        let arg_top_tc = arg.toplevel_tycon().unwrap();
        if !is_unwrappable_newtype(&arg_top_tc, env) {
            continue;
        }
        let arg_top_ti = env.tycons.get(&arg_top_tc).unwrap();
        let arg_top_arity = arg_top_ti.tyvars.len();
        let arg_top_args = arg.collect_type_argments();
        assert!(arg_top_arity <= arg_top_args.len());
        if arg_top_arity < arg_top_args.len() {
            // If the type argument is a partially applied newtype pattern, it should be converted to a new type constructor.
            has_arg_partially_applied_newtype = true;
            break;
        }
    }
    if !has_arg_partially_applied_newtype {
        return None;
    }

    // In this case, convert `ty` to a new type constructor.
    let name = format!("#UnwrapNewtype<{}>", ty.to_string());
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
    let field_types = ty.field_types(env);
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
    Some((new_tc, new_ti))
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function detects circular newtype patterns and avoids infinite loops.
//
// This function is supposed to be called after type aliases are resolved.
//
// If there is a type to be newly defined, add it to `types_to_add`.
// For example, consider `type [f : *->*] Foo f = struct { data : f (), dummy : () };`.
// Note that `Foo` itself is not a newtype, so it is not unwrapped.
// When the type `Foo IO` is used in the program, the `IO` in this type cannot be unwrapped because there is no type like `Foo (IOState -> (IOState, *))`.
// Therefore, define a new type `type #UnwrapNewtype<Foo IO> = struct { data : IO (), dummy : () }` and convert `Foo IO` into `#UnwrapNewtype<Foo IO>`.
fn unwrap_newtype_on_type(
    ty: &Arc<TypeNode>,
    env: &TypeEnv,
    types_to_add: &mut Map<TyCon, TyConInfo>,
) -> Arc<TypeNode> {
    unwrap_newtype_on_type_internal(ty, env, types_to_add)
}

// Internal implementation of unwrap_newtype
fn unwrap_newtype_on_type_internal(
    ty: &Arc<TypeNode>,
    env: &TypeEnv,
    types_to_add: &mut Map<TyCon, TyConInfo>,
) -> Arc<TypeNode> {
    // First, replace the top-level type constructor if it is a newtype.
    // As an example, consider type alias `type Foo a = unbox struct { data : () -> a }`.
    // Then `Foo Bool` should be resolved to `() -> Bool`.
    let top_tc = ty.toplevel_tycon().unwrap();
    let top_tc = top_tc.as_ref();
    let top_ti = env.tycons.get(top_tc).unwrap();
    let is_fully_applied = top_ti.tyvars.len() == ty.collect_type_argments().len();

    if is_fully_applied && is_unwrappable_newtype(top_tc, env) {
        // If the top-level tycon is a fully applied newtype, unwrap it.
        let ti = env.tycons.get(top_tc).unwrap();
        // Check if this is a punched struct of a newtype pattern
        if ti.fields[0].is_punched {
            // Convert punched struct of newtype pattern to unit type
            return make_unit_ty();
        }
        let field_ty = ty.field_types(env)[0].clone();
        let result = unwrap_newtype_on_type_internal(&field_ty, env, types_to_add);

        return result;
    }

    // If the top-level tycon is not a newtype, recursively process type arguments
    let mut ty = match &ty.ty {
        Type::TyVar(_) => {
            unimplemented!("TyVar is not supported in unwrap_newtype_on_type")
        }
        Type::TyCon(_) => ty.clone(),
        Type::TyApp(fun_ty, arg_ty) => ty
            .set_tyapp_fun(unwrap_newtype_on_type_internal(fun_ty, env, types_to_add))
            .set_tyapp_arg(unwrap_newtype_on_type_internal(arg_ty, env, types_to_add)),
        Type::AssocTy(_, _args) => {
            unimplemented!("AssocTy is not supported in unwrap_newtype_on_type")
        }
    };

    if is_fully_applied {
        // Check if there is still an unwrappable newtype being used.
        let mut tycons = Set::default();
        ty.collect_tycons(&mut tycons);
        if tycons.iter().any(|tc| is_unwrappable_newtype(tc, env)) {
            // In this case, there is a newtype that cannot be resolved any further, such as `Foo IO` mentioned in the comment.
            // In this case, convert `ty` to a new type constructor.
            let name = format!("#UnwrapNewtype<{}>", ty.to_string());
            let mut new_tc = top_tc.clone();
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
            let field_types = ty.field_types(env);
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
            types_to_add.insert(new_tc.clone(), new_ti.clone());
            ty = type_tycon(&tycon(new_tc.name));
        }
    }

    ty
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function does not detect circular newtype patterns. If a circular newtype pattern is included, it may fall into an infinite loop.
//
// This function is supposed to be called after type aliases are resolved.
fn unwrap_newtype_on_pattern(pat: &Arc<PatternNode>, env: &TypeEnv) -> Arc<PatternNode> {
    match &pat.pattern {
        Pattern::Var(v, ty) => {
            let ty = ty.as_ref().map(|ty| unwrap_newtype_on_type(ty, env));
            let mut info = pat.info.clone();
            unwrap_newtype_on_pattern_info(&mut info, env);
            Arc::new(PatternNode {
                pattern: Pattern::Var(v.clone(), ty),
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
fn unwrap_newtype_on_pattern_info(pat_info: &mut PatternInfo, env: &TypeEnv) {
    if let Some(ty) = &mut pat_info.type_ {
        *ty = unwrap_newtype_on_type(ty, env);
    }
}

// Unwrap newtype pattern, i.e., type A = unbox struct { data : B } to B.
//
// This function does not detect circular newtype patterns. If a circular newtype pattern is included, it may fall into an infinite loop.
//
// This function is supposed to be called after type aliases are resolved.
pub fn unwrap_newtype_on_type_env(env: &mut TypeEnv) {
    let mut new_tycons = (*env.tycons).clone();

    // Unwrap newtype patterns in the remaining types
    for (_name, tycon_info) in new_tycons.iter_mut() {
        for field in &mut tycon_info.fields {
            let new_ty = unwrap_newtype_on_type(&field.ty, &env);
            field.ty = new_ty;
        }
    }

    // Remove newtype pattern types from the tycons map
    let mut to_remove = Vec::new();
    for (name, _tycon_info) in new_tycons.iter() {
        if is_unwrappable_newtype(name, env) {
            to_remove.push(name.clone());
        }
    }
    for name in to_remove {
        new_tycons.remove(&name);
    }

    env.tycons = Arc::new(new_tycons);
}

// Is this type constructor a "newtype pattern", i.e., is it an unbox struct type with only one field?
fn is_newtype(tycon: &TyCon, env: &TypeEnv) -> bool {
    let ti = env.tycons.get(tycon).unwrap();
    ti.is_unbox && ti.variant == TyConVariant::Struct && ti.fields.len() == 1
}

// Is this type constructor a newtype pattern and unwrappable?
fn is_unwrappable_newtype(tycon: &TyCon, env: &TypeEnv) -> bool {
    let mut visited: Set<TyCon> = Set::default();
    is_unwrappable_newtype_internal(tycon, env, &mut visited)
}

// Is this type constructor a newtype pattern and unwrappable?
fn is_unwrappable_newtype_internal(tc: &TyCon, env: &TypeEnv, visited: &mut Set<TyCon>) -> bool {
    // If this TyCon is not a newtype pattern, return false.
    if !is_newtype(tc, env) {
        return false;
    }

    // If this TyCon is already being processed (circular dependency), return false
    if visited.contains(tc) {
        return false;
    }

    // Add this TyCon to the visited set
    visited.insert(tc.clone());

    let ti = env.tycons.get(tc).unwrap();

    // Check the field type of the newtype pattern
    let field_ty = &ti.fields[0].ty;

    // Collect all TyCons that appear in the field type, which is not punched.
    let referenced_tycons = if ti.fields[0].is_punched {
        Set::default()
    } else {
        let mut s = Set::default();
        field_ty.collect_tycons(&mut s);
        s
    };

    // All referenced TyCons should satisfy the unwrappability conditions
    let result = referenced_tycons.iter().all(|referenced_tycon| {
        if !is_newtype(referenced_tycon, env) {
            // If it's not a newtype pattern, it's always safe
            return true;
        }
        // It's a newtype pattern - check if it's unwrappable and not already in the chain
        is_unwrappable_newtype_internal(referenced_tycon, env, visited)
    });

    // Remove this TyCon from the visited set before returning
    visited.remove(tc);

    result
}
