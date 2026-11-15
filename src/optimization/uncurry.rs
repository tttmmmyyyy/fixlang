/*
Uncurrying optimization

Give each global `g : T1 -> ... -> Tn -> R` an uncurried function `g#func{k}` for each `k` up to
`n`: a function of type `#Func{k} T1 ... Tk (T{k+1} -> ... -> R)` that takes its first `k` arguments
at once and captures nothing. Then rewrite each call `g(a1)...(ak)` into the call
`g#func{k}(a1, ..., ak)`.

Higher-order uncurrying (https://xavierleroy.org/publi/higher-order-uncurrying.pdf) is not implemented.
*/

use super::rename::rename_lam_param_avoiding;
use crate::{
    ast::{
        expr::{expr_abs, expr_app, expr_let_typed, expr_var, Expr, ExprNode, Var},
        name::{FullName, Name},
        program::{Program, Symbol},
        types::type_func,
    },
    constants::{FUNC_ARGS_MAX, INSTANCIATED_NAME_SEPARATOR, UNCURRIED_FUNC_SUFFIX},
    fixstd::stdlib::is_std_fix,
    misc::{for_each_on_compiler_threads, map_on_compiler_threads, Map, Set},
    optimization::eta_expansion,
};
use std::{mem, sync::Arc};

/// Defines the uncurried functions of each global, taking one, two, ... arguments, then rewrites
/// the calls, export statements and entry IO value of the program onto the uncurried function
/// taking the number of arguments they supply. The symbols are rewritten on up to `num_threads`
/// threads.
///
/// The program may be the output of an earlier run, whose uncurried functions keep their names
/// and their calls. Calls are rewritten only onto the uncurried functions this run defines. The
/// passes between two runs, such as lambda set specialization, specialize each uncurried function
/// to its own callers, so the `g#func2` of an earlier run can take other types than a call
/// `g#func1(a)(b)` or `g(a)(b)` supplies.
pub fn run(prg: &mut Program, num_threads: usize) {
    // Define the uncurried functions of each global. An uncurried function gets none, since it
    // takes its arguments at once already.
    let syms = mem::take(&mut prg.symbols);
    let mut taken_names: Set<FullName> = syms.keys().cloned().collect();
    let funcs_of_syms = map_on_compiler_threads(
        syms.into_iter().collect(),
        num_threads,
        |(sym_name, sym)| {
            let sym_funcs = uncurried_funcs_of(&sym_name, &sym);
            (sym_name, sym, sym_funcs)
        },
    );
    let mut funcs = UncurriedFuncs::default();
    for (sym_name, sym, sym_funcs) in funcs_of_syms {
        for (n_args, mut func) in sym_funcs {
            func.name = unused_name(&func.name, &taken_names);
            let name = func.name.clone();
            taken_names.insert(name.clone());
            funcs.insert(sym_name.clone(), n_args, name.clone());
            assert!(
                prg.symbols.insert(name.clone(), func).is_none(),
                "`{}` is defined twice",
                name.to_string()
            );
        }
        assert!(
            prg.symbols.insert(sym_name.clone(), sym).is_none(),
            "`{}` is defined twice",
            sym_name.to_string()
        );
    }

    // Rewrite the calls onto the uncurried functions this run defined.
    for_each_on_compiler_threads(prg.symbols.values_mut().collect(), num_threads, |sym| {
        let expr = replace_closure_calls_with_func_calls(sym.expr.as_ref().unwrap(), &funcs);
        sym.expr = Some(expr);
    });

    // Replace export statements so that they use uncurried functions.
    for export in &mut prg.export_statements {
        let exported_value = export.value_expr.as_ref().unwrap();
        let n_args = exported_value
            .type_
            .as_ref()
            .unwrap()
            .collect_app_src(usize::MAX)
            .0
            .len();
        let Some(uncurried_value) = uncurried_symbol(&prg.symbols, &funcs, exported_value, n_args)
        else {
            continue;
        };
        export.value_name = uncurried_value.name.clone();
        export.value_expr =
            Some(expr_var(uncurried_value.name.clone(), None).set_type(uncurried_value.ty.clone()));
    }

    // Replace entry IO value so that it uses an uncurried function.
    if let Some(entry_io_value) = &prg.entry_io_value {
        // The entry IO value has the unwrapped `IO` type, i.e., the `IOState -> (IOState, a)` type,
        // so it takes one argument.
        if let Some(sym) = uncurried_symbol(&prg.symbols, &funcs, entry_io_value, 1) {
            prg.entry_io_value = Some(expr_var(sym.name.clone(), None).set_type(sym.ty.clone()));
        }
    }
}

/// The uncurried functions one run of `run` defines.
#[derive(Default)]
struct UncurriedFuncs {
    /// The name of the uncurried function of each global taking each number of arguments.
    ///
    /// # Examples
    /// `(g, 2)` maps to `g#func2`.
    of_global: Map<(FullName, usize), FullName>,
    /// The inverse of `of_global`: the name of each uncurried function, mapped to the global it
    /// uncurries and the number of arguments it takes.
    global_of: Map<FullName, (FullName, usize)>,
}

impl UncurriedFuncs {
    /// Records `func` as the uncurried function of `global` taking `n_args` arguments.
    fn insert(&mut self, global: FullName, n_args: usize, func: FullName) {
        assert!(
            self.global_of
                .insert(func.clone(), (global.clone(), n_args))
                .is_none(),
            "`{}` is recorded twice",
            func.to_string()
        );
        assert!(
            self.of_global
                .insert((global.clone(), n_args), func)
                .is_none(),
            "`{}` has two uncurried functions taking {} arguments",
            global.to_string(),
            n_args
        );
    }

    /// The global `func` uncurries and the number of arguments it takes, when `func` is an
    /// uncurried function this run defines.
    fn global_of(&self, func: &FullName) -> Option<&(FullName, usize)> {
        self.global_of.get(func)
    }

    /// The uncurried function of `global` taking `n_args` arguments.
    fn of(&self, global: &FullName, n_args: usize) -> Option<&FullName> {
        self.of_global.get(&(global.clone(), n_args))
    }
}

/// The uncurried functions of the symbol `sym_name`, whose definition is `sym`, each with the
/// number of arguments it takes: one argument, two arguments, and so on, as many as its type
/// allows.
fn uncurried_funcs_of(sym_name: &FullName, sym: &Symbol) -> Vec<(usize, Symbol)> {
    let mut funcs = vec![];
    for n_args in 1..=FUNC_ARGS_MAX as usize {
        let Some(expr) = uncurried_func(&sym.generic_name, sym.expr.as_ref().unwrap(), n_args)
        else {
            break;
        };
        let ty = expr.type_.clone().unwrap();
        let mut name = sym_name.clone();
        convert_to_uncurried_func_name(name.name_as_mut(), n_args);
        let mut generic_name = sym.generic_name.clone();
        convert_to_uncurried_func_name(generic_name.name_as_mut(), n_args);
        funcs.push((
            n_args,
            Symbol {
                name,
                generic_name,
                ty,
                expr: Some(expr),
                inline_into_callers: false,
            },
        ));
    }
    funcs
}

/// The uncurried symbol to use in place of `value`, an expression referring to a global value.
/// The uncurried function in `funcs` taking as many arguments as possible, up to `max_args`, is
/// chosen.
fn uncurried_symbol<'a>(
    symbols: &'a Map<FullName, Symbol>,
    funcs: &UncurriedFuncs,
    value: &Arc<ExprNode>,
    max_args: usize,
) -> Option<&'a Symbol> {
    let value_name = &value.get_var().name;
    if !value.type_.as_ref().unwrap().is_arrow() {
        return None;
    }
    (1..=max_args)
        .rev()
        .find_map(|n_args| funcs.of(value_name, n_args))
        .map(|name| &symbols[name])
}

/// `name`, or when `taken` has it, `name` followed by `#1`, `#2`, ..., whichever comes first that
/// `taken` does not have. An earlier run of `run` leaves its uncurried functions under the names
/// this run gives.
///
/// # Examples
/// `g#func2` gives `g#func2#1` when `taken` has `g#func2` only.
fn unused_name(name: &FullName, taken: &Set<FullName>) -> FullName {
    let mut candidate = name.clone();
    let mut index = 0;
    while taken.contains(&candidate) {
        index += 1;
        candidate = name.clone();
        *candidate.name_as_mut() += &format!("{}{}", INSTANCIATED_NAME_SEPARATOR, index);
    }
    candidate
}

/// Rewrites `name` in place into the name carried by the uncurried function taking `n_args`
/// arguments.
///
/// # Examples
/// `g` with 2 arguments becomes `g#func2`.
fn convert_to_uncurried_func_name(name: &mut Name, n_args: usize) {
    *name += &format!("{}{}", UNCURRIED_FUNC_SUFFIX, n_args);
}

/// The uncurried function taking `n_args` arguments that `expr`, the body of a global, becomes.
///
/// # Arguments
/// * `generic_name` — the name of the global `expr` defines, before instantiation. `Std::fix` is
///   identified by it, since that global has no uncurried function.
fn uncurried_func(
    generic_name: &FullName,
    expr: &Arc<ExprNode>,
    n_args: usize,
) -> Option<Arc<ExprNode>> {
    if is_std_fix(generic_name) {
        return None;
    }

    let expr_type = expr.type_.as_ref().unwrap();
    if expr_type.is_func() {
        return None;
    }

    // Eta expand the expression to take `n_args` arguments.
    let expr = eta_expansion::run_on_expr(expr.clone(), n_args)?;
    let (args, body) = collect_abs(&expr, n_args);

    // Collect types of argments.
    let (arg_types, body_ty) = expr_type.collect_app_src(n_args);
    assert_eq!(*body.type_.as_ref().unwrap(), body_ty);

    // Construct the uncurried function.
    let func_ty = type_func(arg_types, body_ty);
    let func = expr_abs(args, body, None).set_type(func_ty);

    Some(func)
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

/// `expr`, when it is a call of a global, rewritten onto the uncurried function taking as many
/// arguments as the call supplies.
///
/// # Arguments
/// * `funcs` — the uncurried functions this run of `run` defines. A call is rewritten only when
///   the uncurried function it would name is among them.
fn replace_closure_call_with_func_call(
    expr: &Arc<ExprNode>,
    funcs: &UncurriedFuncs,
) -> Arc<ExprNode> {
    let (fun, args) = expr.destructure_app();
    let fun_ty = fun.type_.as_ref().unwrap();
    match &*fun.expr {
        Expr::Var(v) => {
            if v.name.is_local() {
                // If fun is not global, do nothing.
                return expr.clone();
            }
            if args.is_empty() {
                // Currently, we cannot replace the lambda value itself by an uncurried function,
                // because we need to re-instantiate the caller function.
                return expr.clone();
            }
            let mut global = &v.name;
            if fun_ty.is_func() {
                // An uncurried function of a global, called with its arguments and then the
                // arguments of what it returns: `g#func{N}(a1, ..., aN)(b1, ..., bM)` becomes
                // `g#func{N+M}(a1, ..., aN, b1, ..., bM)`.
                // A call of an uncurried function an earlier run defined finds no entry in
                // `funcs`, and stays as it is.
                let Some((func_global, n_args)) = funcs.global_of(&v.name) else {
                    return expr.clone();
                };
                let n_args = *n_args;
                assert!(
                    args.len() >= n_args,
                    "`{}` takes {} arguments but is applied to {}",
                    v.name.to_string(),
                    n_args,
                    args.len()
                );
                if args.len() == n_args {
                    return expr.clone();
                }
                global = func_global;
            }
            let Some(func) = funcs.of(global, args.len()) else {
                // The global has no uncurried function taking this many arguments, so the call
                // stays as it is.
                return expr.clone();
            };
            let result_ty = expr.type_.clone().unwrap();
            let arg_tys = args
                .iter()
                .map(|arg| arg.type_.clone().unwrap())
                .collect::<Vec<_>>();
            let func_ty = type_func(arg_tys, result_ty.clone());
            let f_func = expr_var(func.clone(), None).set_type(func_ty);
            expr_app(f_func, args, None).set_type(result_ty)
        }
        _ => expr.clone(),
    }
}

/// `expr` with every call of a global in it rewritten onto the uncurried function taking as many
/// arguments as the call supplies.
///
/// # Examples
/// `g(a)(b)` becomes `g#func2(a, b)`, and so does `g#func1(a)(b)`.
fn replace_closure_calls_with_func_calls(
    expr: &Arc<ExprNode>,
    funcs: &UncurriedFuncs,
) -> Arc<ExprNode> {
    let expr = replace_closure_call_with_func_call(expr, funcs);
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::Builtin(_) => expr.clone(),
        Expr::App(fun, args) => {
            let args = args
                .iter()
                .map(|arg| replace_closure_calls_with_func_calls(arg, funcs))
                .collect();
            expr.set_app_func(replace_closure_calls_with_func_calls(fun, funcs))
                .set_app_args(args)
        }
        Expr::Lam(_, val) => expr.set_lam_body(replace_closure_calls_with_func_calls(val, funcs)),
        Expr::Let(_, bound, val) => expr
            .set_let_bound(replace_closure_calls_with_func_calls(bound, funcs))
            .set_let_value(replace_closure_calls_with_func_calls(val, funcs)),
        Expr::If(c, t, e) => expr
            .set_if_cond(replace_closure_calls_with_func_calls(c, funcs))
            .set_if_then(replace_closure_calls_with_func_calls(t, funcs))
            .set_if_else(replace_closure_calls_with_func_calls(e, funcs)),
        Expr::Match(cond, pat_vals) => {
            let cond = replace_closure_calls_with_func_calls(cond, funcs);
            let mut new_pat_vals = vec![];
            for (pat, val) in pat_vals {
                let val = replace_closure_calls_with_func_calls(val, funcs);
                new_pat_vals.push((pat.clone(), val));
            }
            expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
        }
        Expr::TyAnno(e, _) => expr.set_tyanno_expr(replace_closure_calls_with_func_calls(e, funcs)),
        Expr::MakeStruct(_, fields) => expr.set_make_struct_fields(
            fields
                .iter()
                .map(|(field_name, field_src, field_expr)| {
                    (
                        field_name.clone(),
                        field_src.clone(),
                        replace_closure_calls_with_func_calls(field_expr, funcs),
                    )
                })
                .collect(),
        ),
        Expr::ArrayLit(elems) => expr.set_array_lit_elems(
            elems
                .iter()
                .map(|e| replace_closure_calls_with_func_calls(e, funcs))
                .collect(),
        ),
        Expr::FFICall(_, _, _, _, args, _) => expr.set_ffi_call_args(
            args.iter()
                .map(|e| replace_closure_calls_with_func_calls(e, funcs))
                .collect(),
        ),
        Expr::Eval(side, main) => expr
            .set_eval_side(replace_closure_calls_with_func_calls(side, funcs))
            .set_eval_main(replace_closure_calls_with_func_calls(main, funcs)),
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
    use super::unused_name;
    use crate::{ast::name::FullName, misc::Set};

    /// `unused_name` gives a name that `taken` does not have: `name` itself while it is free, and
    /// one that differs from every taken name however many of the names it tries first are taken.
    #[test]
    fn test_unused_name_avoids_every_taken_name() {
        let name = FullName::from_strs(&["Main"], "g#func2");
        let mut taken: Set<FullName> = Set::default();
        assert!(unused_name(&name, &taken) == name, "a free name should be kept");
        for _ in 0..3 {
            let fresh = unused_name(&name, &taken);
            assert!(
                !taken.contains(&fresh),
                "`{}` is taken already",
                fresh.to_string()
            );
            taken.insert(fresh);
        }
    }
}
