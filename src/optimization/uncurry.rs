/*
Uncurrying optimizaion

Convert globally defined lambda expressions `|x1,...,xn| (...) : T1 -> ... -> Tn -> R` to function pointer expressions `[x1,...,xn] (...) : [T1,...,Tn] R`.
Also, convert lambda expression call expressions `f(a1, a2, ..., an)` to function pointer expression call expressions `f[a1,a2,...,an]`.

For each lambda expression, define multiple function pointer expressions such as one-variable function pointer expression, two-variable function pointer expression, etc.,
and select the appropriate one according to the call site.

NOTE: I hope to implement higher-order uncurrying optimization (https://xavierleroy.org/publi/higher-order-uncurrying.pdf) in a future!
*/

use super::rename::rename_lam_param_avoiding;
use crate::{
    ast::{
        expr::{collect_app, expr_abs, expr_app, expr_let_typed, expr_var, Expr, ExprNode, Var},
        name::{FullName, Name},
        program::{Program, Symbol},
        types::type_funptr,
    },
    constants::{FUNPTR_ARGS_MAX, INSTANCIATED_NAME_SEPARATOR, STD_NAME},
    fixstd::stdlib::FIX_NAME,
    misc::{Map, Set},
    optimization::eta_expansion,
};
use std::{mem, sync::Arc, usize};

/// Defines a function pointer version of each global for one, two, ... arguments, then rewrites the
/// calls, export statements and entry IO value of the program onto the version matching the number
/// of arguments they supply.
pub fn run(fix_mod: &mut Program) {
    // First, define uncurried version of global symbols.
    let syms = mem::replace(&mut fix_mod.symbols, Default::default());
    for (sym_name, sym) in syms {
        fix_mod.symbols.insert(sym_name.clone(), sym.clone());

        // Add function pointer version as long as possible.
        for n_args in 1..(FUNPTR_ARGS_MAX + 1) {
            let mut expr = funptr_lambda(
                &sym.generic_name,
                sym.expr.as_ref().unwrap(),
                n_args as usize,
            );
            if expr.is_none() {
                break;
            }
            let expr = expr.take().unwrap();
            let ty = expr.type_.clone().unwrap();
            let mut name = sym_name.clone();
            convert_to_funptr_name(name.name_as_mut(), n_args as usize);
            let mut generic_name = sym.generic_name.clone();
            convert_to_funptr_name(generic_name.name_as_mut(), n_args as usize);
            fix_mod.symbols.insert(
                name.clone(),
                Symbol {
                    name: name.clone(),
                    generic_name,
                    ty,
                    expr: Some(expr.clone()),
                    // The uncurried version holds the same body, reached the same way.
                    inline_into_callers: sym.inline_into_callers,
                },
            );
        }
    }

    // Replace application expressions so that they use uncurried pointers.
    let mut symbol_names: Set<FullName> = Default::default();
    for (name, _sym) in &fix_mod.symbols {
        symbol_names.insert(name.clone());
    }
    for (_name, sym) in &mut fix_mod.symbols {
        let expr =
            replace_closure_call_to_funptr_call_subexprs(sym.expr.as_ref().unwrap(), &symbol_names);
        sym.expr = Some(expr);
    }

    // Replace export statements so that they use uncurried functions.
    for export in &mut fix_mod.export_statements {
        let exported_value = export.value_expr.as_ref().unwrap();
        let n_args = exported_value
            .type_
            .as_ref()
            .unwrap()
            .collect_app_src(usize::MAX)
            .0
            .len();
        let Some(uncurried_value) = uncurried_symbol(&fix_mod.symbols, exported_value, n_args)
        else {
            continue;
        };
        export.value_name = uncurried_value.name.clone();
        export.value_expr =
            Some(expr_var(uncurried_value.name.clone(), None).set_type(uncurried_value.ty.clone()));
    }

    // Replace entry IO value so that it uses uncurried function.
    if let Some(entry_io_value) = &fix_mod.entry_io_value {
        // The entry IO value has the unwrapped `IO` type, i.e., the `IOState -> (IOState, a)` type,
        // so it takes one argument.
        if let Some(sym) = uncurried_symbol(&fix_mod.symbols, entry_io_value, 1) {
            fix_mod.entry_io_value =
                Some(expr_var(sym.name.clone(), None).set_type(sym.ty.clone()));
        }
    }
}

/// The uncurried symbol to use in place of `value`, an expression referring to a global value.
/// The uncurried version taking as many arguments as possible, up to `max_args`, is chosen.
fn uncurried_symbol<'a>(
    symbols: &'a Map<FullName, Symbol>,
    value: &Arc<ExprNode>,
    max_args: usize,
) -> Option<&'a Symbol> {
    let value_name = &value.get_var().name;
    if !value.type_.as_ref().unwrap().is_closure() {
        return None;
    }
    for n_args in (1..=max_args).rev() {
        let mut name = value_name.clone();
        convert_to_funptr_name(name.name_as_mut(), n_args);
        if let Some(sym) = symbols.get(&name) {
            return Some(sym);
        }
    }
    None
}

/// Is this symbol `Std::fix` or an instance of it? `Program::determine_symbol_name` names an
/// instance after the original, followed by the separator and the hash of the type it was
/// instantiated at, so the separator is what tells `fix#<hash>` apart from a name that merely begins
/// with `fix`.
pub fn is_std_fix(name: &FullName) -> bool {
    if name.namespace.names != [STD_NAME] {
        return false;
    }
    match name.name.strip_prefix(FIX_NAME) {
        Some(suffix) => suffix.is_empty() || suffix.starts_with(INSTANCIATED_NAME_SEPARATOR),
        None => false,
    }
}

/// Rewrites `name` in place into the name carried by the uncurried version taking `n_args`
/// arguments.
fn convert_to_funptr_name(name: &mut Name, n_args: usize) {
    *name += &format!("#funptr{}", n_args);
}

/// Convert lambda expression to function pointer taking `n_args` arguments.
///
/// # Arguments
/// * `generic_name` — the name of the global `expr` defines, before instantiation. `Std::fix` is
///   identified by it, since that global has no function pointer version.
fn funptr_lambda(
    generic_name: &FullName,
    expr: &Arc<ExprNode>,
    n_args: usize,
) -> Option<Arc<ExprNode>> {
    if is_std_fix(generic_name) {
        return None;
    }

    let expr_type = expr.type_.as_ref().unwrap();
    if expr_type.is_funptr() {
        return None;
    }

    // Eta expand the expression to take `n_args` arguments.
    let expr = eta_expansion::run_on_expr(expr.clone(), n_args)?;
    let (args, body) = collect_abs(&expr, n_args);

    // Collect types of argments.
    let (arg_types, body_ty) = expr_type.collect_app_src(n_args);
    assert_eq!(*body.type_.as_ref().unwrap(), body_ty);

    // Construct function pointer expression.
    let funptr_ty = type_funptr(arg_types, body_ty);
    let funptr = expr_abs(args, body, None).set_type(funptr_ty);

    Some(funptr)
}

/// Decompose expression `|x, y| z` to `([x, y], z)`.
///
/// # Arguments
/// * `vars_limit` — the largest number of parameters to collect. A nested lambda that would carry
///   the count past it is left in the returned body, parameters and all.
fn collect_abs(expr: &Arc<ExprNode>, vars_limit: usize) -> (Vec<Arc<Var>>, Arc<ExprNode>) {
    /// Appends the parameters of the leading lambdas of `expr` to `vars` and returns the body they
    /// wrap.
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

/// Replace "call closure" expression to "call function pointer" expression.
///
/// # Arguments
/// * `symbol_names` — the names of every global defined in the program, including the uncurried
///   ones. A call is rewritten only when the uncurried version it would name is among them.
fn replace_closure_call_to_funptr_call(
    expr: &Arc<ExprNode>,
    symbol_names: &Set<FullName>,
) -> Arc<ExprNode> {
    let (fun, args) = collect_app(expr);
    let fun_ty = fun.type_.as_ref().unwrap();
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
            if !symbol_names.contains(&f_funptr.name) {
                // If function pointer version is not defined, do not apply uncurry.
                return expr.clone();
            }
            let result_ty = expr.type_.clone().unwrap();
            let arg_tys = args
                .iter()
                .map(|arg| arg.type_.clone().unwrap())
                .collect::<Vec<_>>();
            let funptr_ty = type_funptr(arg_tys, result_ty.clone());
            let f_funptr = expr_var(f_funptr.name, None).set_type(funptr_ty);
            expr_app(f_funptr, args, None).set_type(result_ty)
        }
        _ => expr.clone(),
    }
}

/// Replace all "call closure" subexpressions to "call function pointer" expression.
fn replace_closure_call_to_funptr_call_subexprs(
    expr: &Arc<ExprNode>,
    symbol_names: &Set<FullName>,
) -> Arc<ExprNode> {
    let expr = replace_closure_call_to_funptr_call(expr, symbol_names);
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::LLVM(_) => expr.clone(),
        Expr::App(fun, args) => {
            let args = args
                .iter()
                .map(|arg| replace_closure_call_to_funptr_call_subexprs(arg, symbol_names))
                .collect();
            expr.set_app_func(replace_closure_call_to_funptr_call_subexprs(
                fun,
                symbol_names,
            ))
            .set_app_args(args)
        }
        Expr::Lam(_, val) => expr.set_lam_body(replace_closure_call_to_funptr_call_subexprs(
            val,
            symbol_names,
        )),
        Expr::Let(_, bound, val) => expr
            .set_let_bound(replace_closure_call_to_funptr_call_subexprs(
                bound,
                symbol_names,
            ))
            .set_let_value(replace_closure_call_to_funptr_call_subexprs(
                val,
                symbol_names,
            )),
        Expr::If(c, t, e) => expr
            .set_if_cond(replace_closure_call_to_funptr_call_subexprs(
                c,
                symbol_names,
            ))
            .set_if_then(replace_closure_call_to_funptr_call_subexprs(
                t,
                symbol_names,
            ))
            .set_if_else(replace_closure_call_to_funptr_call_subexprs(
                e,
                symbol_names,
            )),
        Expr::Match(cond, pat_vals) => {
            let cond = replace_closure_call_to_funptr_call_subexprs(cond, symbol_names);
            let mut new_pat_vals = vec![];
            for (pat, val) in pat_vals {
                let val = replace_closure_call_to_funptr_call_subexprs(val, symbol_names);
                new_pat_vals.push((pat.clone(), val));
            }
            expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
        }
        Expr::TyAnno(e, _) => expr.set_tyanno_expr(replace_closure_call_to_funptr_call_subexprs(
            e,
            symbol_names,
        )),
        Expr::MakeStruct(_, fields) => {
            let fields = fields.clone();
            let mut expr = expr;
            for (field_name, _, field_expr) in fields {
                let field_expr =
                    replace_closure_call_to_funptr_call_subexprs(&field_expr, symbol_names);
                expr = expr.set_make_struct_field(&field_name, field_expr);
            }
            expr
        }
        Expr::ArrayLit(elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                expr = expr.set_array_lit_elem(
                    replace_closure_call_to_funptr_call_subexprs(e, symbol_names),
                    i,
                )
            }
            expr
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            let mut expr = expr.clone();
            for (i, e) in args.iter().enumerate() {
                expr = expr.set_ffi_call_arg(
                    replace_closure_call_to_funptr_call_subexprs(e, symbol_names),
                    i,
                )
            }
            expr
        }
        Expr::Eval(side, main) => expr
            .set_eval_side(replace_closure_call_to_funptr_call_subexprs(
                side,
                symbol_names,
            ))
            .set_eval_main(replace_closure_call_to_funptr_call_subexprs(
                main,
                symbol_names,
            )),
    }
}

/// Convert `let a = x in |b| y` to `|b| let a = x in y` if `x` is a variable expression.
fn internalize_let_to_var_one(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    // Check if the expression is in the form of `let a = x in |b| y`.
    if !expr.is_let() {
        return expr.clone();
    }
    let lam = expr.get_let_value();
    if !lam.is_lam() {
        return expr.clone();
    }
    let pat_a = expr.get_let_pat();
    let bound_x = expr.get_let_bound();
    if !bound_x.is_var() {
        return expr.clone();
    }

    // Rename the parameter of the lambda so that it is not contained in `FV(bound_x) + FV(pat_a)`.
    let mut black_list = pat_a.pattern.vars();
    black_list.extend(&mut bound_x.free_vars().into_iter());
    let lam = rename_lam_param_avoiding(&black_list, lam);

    // Construct the expression.
    let params_b = lam.get_lam_params();
    let body_y = lam.get_lam_body();
    let new_expr = expr_let_typed(pat_a.clone(), bound_x.clone(), body_y.clone());
    let new_expr = expr_abs(params_b, new_expr, None);
    new_expr.set_type(expr.type_.clone().unwrap())
}

/// Rewrites the head of `expr` to begin with as many nested lambdas as possible, by moving each
/// `let` binding of a variable that stands between them inside the lambdas that follow it.
pub fn internalize_let_to_var_at_head(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Lam(_, body) => {
            let body = internalize_let_to_var_at_head(body);
            expr.set_lam_body(body)
        }
        Expr::Let(_, _, val) => {
            // Before applying `internalize_let_to_var_one` into the whole let expression,
            // apply it to the value of the let expression.
            // This increases the chance of applying `internalize_let_to_var_one` by changing the value to a lambda expression.
            let val = internalize_let_to_var_at_head(val);
            let expr = expr.set_let_value_typed(val);

            // Apply `internalize_let_to_var_one` to the whole let expression.
            let expr = internalize_let_to_var_one(&expr);

            // If the whole expression changed into a lambda expression, apply `internalize_let_to_var_at_head` again.
            match &*expr.expr {
                Expr::Lam(_, _) => internalize_let_to_var_at_head(&expr),
                _ => expr,
            }
        }
        _ => expr.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name a global of `namespace` called `name` carries, once instantiated at some type.
    fn instance_of(namespace: &str, name: &str) -> FullName {
        FullName::from_strs(
            &[namespace],
            &format!("{}{}0123abcd", name, INSTANCIATED_NAME_SEPARATOR),
        )
    }

    /// A global reaches the predicate both as declared and as instantiated at a type, so both forms
    /// have to be recognized.
    #[test]
    fn std_fix_and_its_instances_are_recognized() {
        assert!(is_std_fix(&FullName::from_strs(&[STD_NAME], FIX_NAME)));
        assert!(is_std_fix(&instance_of(STD_NAME, FIX_NAME)));
    }

    /// A name sharing `fix` as a prefix belongs to a distinct global, whose application would be
    /// rewritten into a fixed-point computation were the predicate to accept it.
    #[test]
    fn a_name_that_merely_begins_with_fix_is_not_std_fix() {
        assert!(!is_std_fix(&FullName::from_strs(&[STD_NAME], "fixup")));
        assert!(!is_std_fix(&instance_of(STD_NAME, "fixup")));
    }

    /// A module's own `fix` is an ordinary global, so recognition demands the `Std` namespace as
    /// well as the name.
    #[test]
    fn a_global_named_fix_outside_std_is_not_std_fix() {
        assert!(!is_std_fix(&FullName::from_strs(&["Main"], FIX_NAME)));
        assert!(!is_std_fix(&instance_of("Main", FIX_NAME)));
    }
}
