use std::sync::Arc;

use crate::typecheck::Scope;

use super::*;

// First-order uncurrying optimizaion:
// Global closures are uncurried as long as possible, and converted to function pointers (= has no field for captured values).
// NOTE: I hope to implement higher-order uncurrying optimization (https://xavierleroy.org/publi/higher-order-uncurrying.pdf) in a future!

pub fn uncurry_optimization(fix_mod: &mut Program) {
    // First, define uncurried version of global symbols.
    let syms = std::mem::replace(&mut fix_mod.instantiated_symbols, Default::default());
    for (sym_name, sym) in syms {
        let typeresolver = &sym.substitution;

        fix_mod
            .instantiated_symbols
            .insert(sym_name.clone(), sym.clone());

        // Add function pointer version as long as possible.
        for arg_cnt in 1..(FUNPTR_ARGS_MAX + 1) {
            let mut expr = funptr_lambda(
                &sym.generic_name,
                sym.expr.as_ref().unwrap(),
                typeresolver,
                arg_cnt as usize,
            );
            if expr.is_none() {
                break;
            }
            let expr = calculate_free_vars(expr.take().unwrap());
            let ty = expr.ty.clone().unwrap();
            let mut name = sym_name.clone();
            convert_to_funptr_name(name.name_as_mut(), arg_cnt as usize);
            let mut generic_name = sym.generic_name.clone();
            convert_to_funptr_name(generic_name.name_as_mut(), arg_cnt as usize);
            fix_mod.instantiated_symbols.insert(
                name.clone(),
                InstantiatedSymbol {
                    instantiated_name: name.clone(),
                    generic_name: generic_name,
                    ty,
                    expr: Some(expr.clone()),
                    substitution: sym.substitution.clone(),
                },
            );
        }
    }

    // Then replace expressions in the global symbols.
    let mut symbol_names: HashSet<FullName> = Default::default();
    for (name, _sym) in &fix_mod.instantiated_symbols {
        symbol_names.insert(name.clone());
    }
    for (_name, sym) in &mut fix_mod.instantiated_symbols {
        let expr = replace_closure_call_to_funptr_call_subexprs(
            sym.expr.as_ref().unwrap(),
            &symbol_names,
            &sym.substitution,
        );
        let expr = calculate_free_vars(expr);
        sym.expr = Some(expr);
    }
}

// Global functions that cannot be uncurried.
pub fn exclude(name: &FullName) -> bool {
    let fix_name = FullName::from_strs(&[STD_NAME], FIX_NAME);
    if *name == fix_name
        || (name.to_string() + INSTANCIATED_NAME_SEPARATOR).starts_with(&fix_name.to_string())
    {
        // fix cannot be function ptr, because it calculates "fixf" in its implementation.
        return true;
    }
    return false;
}

pub fn convert_to_funptr_name(name: &mut Name, var_count: usize) {
    *name += &format!("#funptr{}", var_count);
}

// Convert lambda expression to function pointer.
fn funptr_lambda(
    generic_name: &FullName,
    expr: &Arc<ExprNode>,
    substitution: &Substitution, // for resolving types of expr
    vars_count: usize,
) -> Option<Arc<ExprNode>> {
    if exclude(generic_name) {
        return None;
    }

    let expr_type = substitution.substitute_type(expr.ty.as_ref().unwrap());
    if expr_type.is_funptr() {
        return None;
    }

    // Extract abstractions from expr.
    let expr = move_abs_front_let_all(expr);
    let (args, body) = collect_abs(&expr, vars_count);
    if args.len() != vars_count {
        return None;
    }

    // Collect types of argments.
    let (arg_types, body_ty) = collect_app_src(&expr_type, vars_count);
    assert_eq!(
        substitution.substitute_type(body.ty.as_ref().unwrap()),
        body_ty
    );

    // Construct function pointer.
    let funptr_ty = type_funptr(arg_types, body_ty);
    let funptr = expr_abs(args, body, None).set_inferred_type(funptr_ty);

    Some(funptr)
}

// Decompose expression |x, y| z to ([x, y], z).
fn collect_abs(expr: &Arc<ExprNode>, vars_limit: usize) -> (Vec<Arc<Var>>, Arc<ExprNode>) {
    fn collect_abs_inner(
        expr: &Arc<ExprNode>,
        vars: &mut Vec<Arc<Var>>,
        vars_limit: usize,
    ) -> Arc<ExprNode> {
        match &*expr.expr {
            Expr::Lam(vs, val) => {
                if vars.len() + vs.len() > vars_limit {
                    return expr.clone();
                }
                vars.append(&mut vs.clone());
                return collect_abs_inner(val, vars, vars_limit);
            }
            _ => expr.clone(),
        }
    }

    let mut vars: Vec<Arc<Var>> = vec![];
    let val = collect_abs_inner(expr, &mut vars, vars_limit);
    (vars, val)
}

// Convert A -> B -> C to ([A, B], C)
fn collect_app_src(ty: &Arc<TypeNode>, vars_limit: usize) -> (Vec<Arc<TypeNode>>, Arc<TypeNode>) {
    fn collect_app_src_inner(
        ty: &Arc<TypeNode>,
        vars: &mut Vec<Arc<TypeNode>>,
        vars_limit: usize,
    ) -> Arc<TypeNode> {
        match &ty.ty {
            Type::FunTy(var, val) => {
                vars.push(var.clone());
                if vars.len() >= vars_limit {
                    return val.clone();
                }
                return collect_app_src_inner(&val, vars, vars_limit);
            }
            _ => {
                if ty.is_funptr() {
                    let mut vs = ty.get_lambda_srcs();
                    if vars.len() + vs.len() > vars_limit {
                        return ty.clone();
                    }
                    vars.append(&mut vs);
                    return collect_app_src_inner(&ty.get_lambda_dst(), vars, vars_limit);
                } else {
                    ty.clone()
                }
            }
        }
    }

    let mut vars: Vec<Arc<TypeNode>> = vec![];
    let val = collect_app_src_inner(ty, &mut vars, vars_limit);
    (vars, val)
}

// Replace "call closure" expression to "call function pointer" expression.
fn replace_closure_call_to_funptr_call(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<FullName>,
    substitution: &Substitution,
) -> Arc<ExprNode> {
    let (fun, args) = collect_app(expr);
    let fun_ty = substitution.substitute_type(fun.ty.as_ref().unwrap());
    if fun_ty.is_funptr() {
        return expr.clone();
    }
    match &*fun.expr {
        Expr::Var(v) => {
            if v.name.is_local() {
                // If fun is not global, do nothing.
                return expr.clone();
            }
            if args.is_empty() {
                // Currently, we cannot replace lambda value itself to function pointer,
                // because we need to re-instantiate the caller function.
                return expr.clone();
            }
            let mut f_funptr = v.as_ref().clone();
            convert_to_funptr_name(&mut f_funptr.name.name, args.len());
            if !symbols.contains(&f_funptr.name) {
                // If function pointer version is not defined, do not apply uncurry.
                return expr.clone();
            }
            let result_ty = expr.ty.clone().unwrap();
            let arg_tys = args
                .iter()
                .map(|arg| arg.ty.clone().unwrap())
                .collect::<Vec<_>>();
            let funptr_ty = type_funptr(arg_tys, result_ty.clone());
            let f_funptr = expr_var(f_funptr.name, None).set_inferred_type(funptr_ty);
            expr_app(f_funptr, args, None).set_inferred_type(result_ty)
        }
        _ => expr.clone(),
    }
}

// Replace all "call closure" subexpressions to "call function pointer" expression.
fn replace_closure_call_to_funptr_call_subexprs(
    expr: &Arc<ExprNode>,
    symbols: &HashSet<FullName>,
    typeresolver: &TypeResolver,
) -> Arc<ExprNode> {
    let expr = replace_closure_call_to_funptr_call(expr, symbols, typeresolver);
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::LLVM(_) => expr.clone(),
        Expr::App(fun, args) => {
            let args = args
                .iter()
                .map(|arg| replace_closure_call_to_funptr_call_subexprs(arg, symbols, typeresolver))
                .collect();
            expr.set_app_func(replace_closure_call_to_funptr_call_subexprs(
                fun,
                symbols,
                typeresolver,
            ))
            .set_app_args(args)
        }
        Expr::Lam(_, val) => expr.set_lam_body(replace_closure_call_to_funptr_call_subexprs(
            val,
            symbols,
            typeresolver,
        )),
        Expr::Let(_, bound, val) => expr
            .set_let_bound(replace_closure_call_to_funptr_call_subexprs(
                bound,
                symbols,
                typeresolver,
            ))
            .set_let_value(replace_closure_call_to_funptr_call_subexprs(
                val,
                symbols,
                typeresolver,
            )),
        Expr::If(c, t, e) => expr
            .set_if_cond(replace_closure_call_to_funptr_call_subexprs(
                c,
                symbols,
                typeresolver,
            ))
            .set_if_then(replace_closure_call_to_funptr_call_subexprs(
                t,
                symbols,
                typeresolver,
            ))
            .set_if_else(replace_closure_call_to_funptr_call_subexprs(
                e,
                symbols,
                typeresolver,
            )),
        Expr::TyAnno(e, _) => expr.set_tyanno_expr(replace_closure_call_to_funptr_call_subexprs(
            e,
            symbols,
            typeresolver,
        )),
        Expr::MakeStruct(_, fields) => {
            let fields = fields.clone();
            let mut expr = expr;
            for (field_name, field_expr) in fields {
                let field_expr = replace_closure_call_to_funptr_call_subexprs(
                    &field_expr,
                    symbols,
                    typeresolver,
                );
                expr = expr.set_make_struct_field(&field_name, field_expr);
            }
            expr
        }
        Expr::ArrayLit(elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                expr = expr.set_array_lit_elem(
                    replace_closure_call_to_funptr_call_subexprs(e, symbols, typeresolver),
                    i,
                )
            }
            expr
        }
        Expr::CallC(_, _, _, _, args) => {
            let mut expr = expr.clone();
            for (i, e) in args.iter().enumerate() {
                expr = expr.set_call_c_arg(
                    replace_closure_call_to_funptr_call_subexprs(e, symbols, typeresolver),
                    i,
                )
            }
            expr
        }
    }
}

// Convert `let a = x in |b| y` to `|b| let a = x in y` if possible.
// NOTE: if name `b` is contained in x, then first we need to replace `b` to another name.
fn move_abs_front_let_one(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(let_var, let_bound, let_val) => {
            let let_val = move_abs_front_let_one(let_val);
            match &*let_val.expr {
                Expr::Lam(lam_vars, lam_val) => {
                    let ty = expr.ty.clone().unwrap();

                    // Replace lam_var and its appearance in lam_val to avoid confliction with free variables in let_bound.
                    let let_bound = calculate_free_vars(let_bound.clone());
                    let let_bound_free_vars = let_bound.free_vars();

                    let mut lam_vars = lam_vars.clone();
                    let mut lam_val = lam_val.clone();

                    for lam_var in &mut lam_vars {
                        let original_name = lam_var.name.clone();
                        let mut lam_var_name = original_name.clone();
                        if let_bound_free_vars.contains(&lam_var_name) {
                            // If replace is necessary,
                            let mut counter = -1;
                            loop {
                                counter += 1;
                                // Make a candidate for the new name.
                                *lam_var_name.name_as_mut() =
                                    format!("{}@{}", original_name.name, counter);

                                // If it is still appears in let_bound, try another name.
                                if let_bound_free_vars.contains(&lam_var_name) {
                                    continue;
                                }

                                // If new name is already appears freely in lam_val, it cannot be used.
                                let lam_val_frees =
                                    calculate_free_vars(lam_val.clone()).free_vars().clone();
                                if lam_val_frees.contains(&lam_var_name) {
                                    continue;
                                }

                                // Replace original_name in lam_val.
                                let replaced = replace_free_var(
                                    &lam_val,
                                    &original_name,
                                    &lam_var_name,
                                    &mut Scope::default(),
                                );
                                // If replacement to lam_var_name fails, try another name.
                                if replaced.is_err() {
                                    continue;
                                }

                                *lam_var = lam_var.set_name(lam_var_name.clone());
                                lam_val = replaced.unwrap();
                                break;
                            }
                        }
                    }

                    // Construct the expression.
                    let expr = expr_let(let_var.clone(), let_bound.clone(), lam_val.clone(), None)
                        .set_inferred_type(lam_val.ty.clone().unwrap());
                    let expr = expr_abs(lam_vars, expr, None).set_inferred_type(ty);
                    expr
                }
                _ => expr.clone(),
            }
        }
        _ => expr.clone(),
    }
}

// apply move_abs_front_let_one repeatedly at the head.
fn move_abs_front_let_all(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Lam(_, val) => {
            let val = move_abs_front_let_all(val);
            expr.set_lam_body(val)
        }
        Expr::Let(_, _, _) => {
            let expr = move_abs_front_let_one(&expr);
            match &*expr.expr {
                Expr::Lam(_, _) => move_abs_front_let_all(&expr),
                _ => expr,
            }
        }
        _ => expr.clone(),
    }
}

// Replace the name of a free variable in an expression.
// If the name `to` is bound at the place `from` appears, returns Err.
fn replace_free_var(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &FullName,
    scope: &mut Scope<()>,
) -> Result<Arc<ExprNode>, ()> {
    match &*expr.expr {
        Expr::Var(v) => {
            if v.name == *from {
                if scope.local_names().contains(&to.name) {
                    Err(())
                } else {
                    Ok(expr.clone().set_var_var(v.set_name(to.clone())))
                }
            } else {
                Ok(expr.clone())
            }
        }
        Expr::LLVM(_) => Ok(expr.clone()),
        Expr::App(func, args) => {
            let func = replace_free_var(func, from, to, scope)?;
            let args = args
                .iter()
                .map(|arg| replace_free_var(arg, from, to, scope))
                .collect::<Result<_, ()>>()?;
            Ok(expr.set_app_func(func).set_app_args(args))
        }
        Expr::Lam(vs, val) => {
            let val = if vs.iter().any(|v| v.name == *from) {
                // then, the from-name is shadowed in val, so we should not replace val.
                val.clone()
            } else {
                for v in vs {
                    scope.push(&v.name.name, &());
                }
                let res = replace_free_var(val, from, to, scope)?;
                for v in vs {
                    scope.pop(&v.name.name);
                }
                res
            };
            Ok(expr.set_lam_body(val))
        }
        Expr::Let(pat, bound, val) => {
            let bound = replace_free_var(bound, from, to, scope)?;
            let val = if pat.pattern.vars().contains(from) {
                // then, the from-name is shadowed in val, so we should not replace val.
                val.clone()
            } else {
                for v in pat.pattern.vars() {
                    scope.push(&v.name, &());
                }
                let res = replace_free_var(val, from, to, scope)?;
                for v in pat.pattern.vars() {
                    scope.pop(&v.name);
                }
                res
            };
            Ok(expr.set_let_bound(bound).set_let_value(val))
        }
        Expr::If(c, t, e) => {
            let c = replace_free_var(c, from, to, scope)?;
            let t = replace_free_var(t, from, to, scope)?;
            let e = replace_free_var(e, from, to, scope)?;
            Ok(expr.set_if_cond(c).set_if_then(t).set_if_else(e))
        }
        Expr::TyAnno(e, _) => {
            let e = replace_free_var(e, from, to, scope)?;
            Ok(expr.set_tyanno_expr(e))
        }
        Expr::MakeStruct(_, fields) => {
            let mut expr = expr.clone();
            for (field_name, field_expr) in fields {
                let field_expr = replace_free_var(field_expr, from, to, scope)?;
                expr = expr.set_make_struct_field(field_name, field_expr);
            }
            Ok(expr)
        }
        Expr::ArrayLit(elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                let e = replace_free_var(e, from, to, scope)?;
                expr = expr.set_array_lit_elem(e, i);
            }
            Ok(expr)
        }
        Expr::CallC(_, _, _, _, elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                let e = replace_free_var(e, from, to, scope)?;
                expr = expr.set_call_c_arg(e, i);
            }
            Ok(expr)
        }
    }
}

// fn replace_travarsally(
//     expr: Arc<ExprNode>,
//     replace: &impl Fn(Arc<ExprNode>) -> Arc<ExprNode>,
// ) -> Arc<ExprNode> {
//     match &*expr.expr {
//         Expr::Var(_) => replace(expr.clone()),
//         Expr::Lit(_) => replace(expr.clone()),
//         Expr::App(fun, arg) => {
//             let expr = expr
//                 .set_app_func(replace_travarsally(fun.clone(), replace))
//                 .set_app_arg(replace_travarsally(arg.clone(), replace));
//             replace(expr)
//         }
//         Expr::Lam(_, val) => {
//             let expr = expr.set_lam_body(replace_travarsally(val.clone(), replace));
//             replace(expr)
//         }
//         Expr::Let(_, bound, val) => {
//             let expr = expr
//                 .set_let_bound(replace_travarsally(bound.clone(), replace))
//                 .set_let_value(replace_travarsally(val.clone(), replace));
//             replace(expr)
//         }
//         Expr::If(c, t, e) => {
//             let expr = expr
//                 .set_if_cond(replace_travarsally(c.clone(), replace))
//                 .set_if_then(replace_travarsally(t.clone(), replace))
//                 .set_if_else(replace_travarsally(e.clone(), replace));
//             replace(expr)
//         }
//         Expr::TyAnno(e, _) => {
//             let expr = expr.set_tyanno_expr(replace_travarsally(e.clone(), replace));
//             replace(expr)
//         }
//         Expr::MakePair(lhs, rhs) => {
//             let expr = expr
//                 .set_make_pair_lhs(replace_travarsally(lhs.clone(), replace))
//                 .set_make_pair_rhs(replace_travarsally(rhs.clone(), replace));
//             replace(expr)
//         }
//     }
// }
