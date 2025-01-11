use std::{sync::Arc, usize};

use crate::{
    ast::name::{FullName, Name},
    collect_app, expr_abs, expr_app, expr_let, expr_var,
    misc::Set,
    type_funptr, Expr, ExprNode, InstantiatedSymbol, Program, Var, FIX_NAME, FUNPTR_ARGS_MAX,
    INSTANCIATED_NAME_SEPARATOR, STD_NAME,
};

use super::utils::replace_free_var_of_expr;

// First-order uncurrying optimizaion:
// Global closures are uncurried as long as possible, and converted to function pointers (= has no field for captured values).
// NOTE: I hope to implement higher-order uncurrying optimization (https://xavierleroy.org/publi/higher-order-uncurrying.pdf) in a future!

pub fn run(fix_mod: &mut Program) {
    // First, define uncurried version of global symbols.
    let syms = std::mem::replace(&mut fix_mod.instantiated_symbols, Default::default());
    for (sym_name, sym) in syms {
        fix_mod
            .instantiated_symbols
            .insert(sym_name.clone(), sym.clone());

        // Add function pointer version as long as possible.
        for arg_cnt in 1..(FUNPTR_ARGS_MAX + 1) {
            let mut expr = funptr_lambda(
                &sym.generic_name,
                sym.expr.as_ref().unwrap(),
                arg_cnt as usize,
            );
            if expr.is_none() {
                break;
            }
            let expr = expr.take().unwrap().calculate_free_vars();
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
                },
            );
        }
    }

    // Replace application expressions so that they use uncurried pointers.
    let mut symbol_names: Set<FullName> = Default::default();
    for (name, _sym) in &fix_mod.instantiated_symbols {
        symbol_names.insert(name.clone());
    }
    for (_name, sym) in &mut fix_mod.instantiated_symbols {
        let expr =
            replace_closure_call_to_funptr_call_subexprs(sym.expr.as_ref().unwrap(), &symbol_names);
        let expr = expr.calculate_free_vars();
        sym.expr = Some(expr);
    }

    // Replace export statements so that they use uncurried functions.
    for export in &mut fix_mod.export_statements {
        let exported_value = export.instantiated_value_expr.as_ref().unwrap();
        let exported_value_name = &exported_value.get_var().name;
        let exported_value_ty = exported_value.ty.as_ref().unwrap();
        if !exported_value_ty.is_closure() {
            continue;
        }
        let mut n_args = exported_value_ty.collect_app_src(usize::MAX).0.len();
        let uncurried_value = loop {
            if n_args == 0 {
                break None;
            }
            let mut name = exported_value_name.clone();
            convert_to_funptr_name(name.name_as_mut(), n_args);
            if let Some(sym) = fix_mod.instantiated_symbols.get(&name) {
                break Some(sym);
            }
            n_args -= 1;
        };
        if let None = uncurried_value {
            continue;
        }
        let uncurried_value = uncurried_value.unwrap();
        export.fix_value_name = uncurried_value.instantiated_name.clone();
        export.instantiated_value_expr = Some(
            expr_var(uncurried_value.instantiated_name.clone(), None)
                .set_inferred_type(uncurried_value.ty.clone()),
        );
    }
}

// Is this symbol a Std::fix or its instance?
pub fn is_std_fix(name: &FullName) -> bool {
    let fix_name = FullName::from_strs(&[STD_NAME], FIX_NAME);
    *name == fix_name
        || (name.to_string() + INSTANCIATED_NAME_SEPARATOR).starts_with(&fix_name.to_string())
}

fn convert_to_funptr_name(name: &mut Name, var_count: usize) {
    *name += &format!("#funptr{}", var_count);
}

// Convert lambda expression to function pointer.
fn funptr_lambda(
    generic_name: &FullName,
    expr: &Arc<ExprNode>,
    vars_count: usize,
) -> Option<Arc<ExprNode>> {
    if is_std_fix(generic_name) {
        return None;
    }

    let expr_type = expr.ty.as_ref().unwrap();
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
    let (arg_types, body_ty) = expr_type.collect_app_src(vars_count);
    assert_eq!(*body.ty.as_ref().unwrap(), body_ty);

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

// Replace "call closure" expression to "call function pointer" expression.
pub fn replace_closure_call_to_funptr_call(
    expr: &Arc<ExprNode>,
    symbols: &Set<FullName>,
) -> Arc<ExprNode> {
    let (fun, args) = collect_app(expr);
    let fun_ty = fun.ty.as_ref().unwrap();
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
    symbols: &Set<FullName>,
) -> Arc<ExprNode> {
    let expr = replace_closure_call_to_funptr_call(expr, symbols);
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::LLVM(_) => expr.clone(),
        Expr::App(fun, args) => {
            let args = args
                .iter()
                .map(|arg| replace_closure_call_to_funptr_call_subexprs(arg, symbols))
                .collect();
            expr.set_app_func(replace_closure_call_to_funptr_call_subexprs(fun, symbols))
                .set_app_args(args)
        }
        Expr::Lam(_, val) => {
            expr.set_lam_body(replace_closure_call_to_funptr_call_subexprs(val, symbols))
        }
        Expr::Let(_, bound, val) => expr
            .set_let_bound(replace_closure_call_to_funptr_call_subexprs(bound, symbols))
            .set_let_value(replace_closure_call_to_funptr_call_subexprs(val, symbols)),
        Expr::If(c, t, e) => expr
            .set_if_cond(replace_closure_call_to_funptr_call_subexprs(c, symbols))
            .set_if_then(replace_closure_call_to_funptr_call_subexprs(t, symbols))
            .set_if_else(replace_closure_call_to_funptr_call_subexprs(e, symbols)),
        Expr::Match(cond, pat_vals) => {
            let cond = replace_closure_call_to_funptr_call_subexprs(cond, symbols);
            let mut new_pat_vals = vec![];
            for (pat, val) in pat_vals {
                let val = replace_closure_call_to_funptr_call_subexprs(val, symbols);
                new_pat_vals.push((pat.clone(), val));
            }
            expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
        }
        Expr::TyAnno(e, _) => {
            expr.set_tyanno_expr(replace_closure_call_to_funptr_call_subexprs(e, symbols))
        }
        Expr::MakeStruct(_, fields) => {
            let fields = fields.clone();
            let mut expr = expr;
            for (field_name, field_expr) in fields {
                let field_expr = replace_closure_call_to_funptr_call_subexprs(&field_expr, symbols);
                expr = expr.set_make_struct_field(&field_name, field_expr);
            }
            expr
        }
        Expr::ArrayLit(elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                expr = expr
                    .set_array_lit_elem(replace_closure_call_to_funptr_call_subexprs(e, symbols), i)
            }
            expr
        }
        Expr::FFICall(_, _, _, args, _) => {
            let mut expr = expr.clone();
            for (i, e) in args.iter().enumerate() {
                expr = expr
                    .set_ffi_call_arg(replace_closure_call_to_funptr_call_subexprs(e, symbols), i)
            }
            expr
        }
    }
}

// Convert `let a = x in |b| y` to `|b| let a = x in y` if possible.
// NOTE: if name `b` is contained in x, then first we need to replace `b` to another name.
// TODO: Refactor this using `rename_lam_param_avoiding`.
fn move_abs_front_let_one(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Let(let_var, let_bound, let_val) => {
            let let_val = move_abs_front_let_one(let_val);
            match &*let_val.expr {
                Expr::Lam(lam_vars, lam_val) => {
                    let ty = expr.ty.clone().unwrap();

                    // Replace lam_var and its appearance in lam_val to avoid confliction with free variables in let_bound.
                    let let_bound = let_bound.calculate_free_vars();
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
                                if lam_val
                                    .calculate_free_vars()
                                    .free_vars()
                                    .contains(&lam_var_name)
                                {
                                    continue;
                                }

                                // Replace original_name in lam_val.
                                let replaced = replace_free_var_of_expr(
                                    &lam_val,
                                    &original_name,
                                    &lam_var_name,
                                );

                                *lam_var = lam_var.set_name(lam_var_name.clone());
                                lam_val = replaced;
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
