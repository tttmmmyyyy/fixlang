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
        expr::{collect_app, expr_abs, expr_app, expr_let_typed, expr_var, Expr, ExprNode, Var},
        name::{FullName, Name},
        program::{Program, Symbol},
        types::type_func,
    },
    constants::{FUNC_ARGS_MAX, INSTANCIATED_NAME_SEPARATOR, STD_NAME, UNCURRIED_FUNC_SUFFIX},
    fixstd::stdlib::FIX_NAME,
    misc::{for_each_on_compiler_threads, map_on_compiler_threads, Map},
    optimization::eta_expansion,
};
use std::{mem, sync::Arc};

/// Defines the uncurried functions of each global, taking one, two, ... arguments, then rewrites
/// the calls, export statements and entry IO value of the program onto the uncurried function
/// taking the number of arguments they supply. The symbols are rewritten on up to `num_threads`
/// threads.
///
/// Returns the uncurried functions it defined, which a later `replace_calls_with_funcs` takes.
pub fn run(prg: &mut Program, num_threads: usize) -> UncurriedFuncs {
    // Define the uncurried functions of each global.
    let syms = mem::replace(&mut prg.symbols, Default::default());
    let funcs_of_syms = map_on_compiler_threads(
        syms.into_iter().collect(),
        num_threads,
        |(sym_name, sym)| with_uncurried_funcs(sym_name, sym),
    );
    let mut funcs = UncurriedFuncs::default();
    for global_and_funcs in funcs_of_syms {
        let global = global_and_funcs[0].0.clone();
        for (n_args, (name, _)) in global_and_funcs.iter().enumerate().skip(1) {
            funcs.insert(name.clone(), (global.clone(), n_args));
        }
        prg.symbols.extend(global_and_funcs);
    }

    // Rewrite the calls onto the uncurried functions.
    replace_calls_with_funcs(prg, &funcs, num_threads);

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
        let Some(uncurried_value) = uncurried_symbol(&prg.symbols, exported_value, n_args) else {
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
        if let Some(sym) = uncurried_symbol(&prg.symbols, entry_io_value, 1) {
            prg.entry_io_value = Some(expr_var(sym.name.clone(), None).set_type(sym.ty.clone()));
        }
    }

    funcs
}

/// Rewrites every call of a global in the program onto the uncurried function matching the
/// number of arguments it supplies, on up to `num_threads` threads.
///
/// A call of an uncurried function whose result is applied to more arguments is merged into one
/// call of a longer one. So running this again after other passes rewrites the calls those passes
/// bring together.
///
/// # Examples
/// `g(a)(b)` becomes `g#func2(a, b)`, and so does `g#func1(a)(b)`.
///
/// # Arguments
/// * `funcs` — the uncurried functions `run` defined, each of which must still be in the program.
pub fn replace_calls_with_funcs(prg: &mut Program, funcs: &UncurriedFuncs, num_threads: usize) {
    for func in funcs.keys() {
        assert!(
            prg.symbols.contains_key(func),
            "the uncurried function `{}` is no longer in the program",
            func.to_string()
        );
    }
    for_each_on_compiler_threads(prg.symbols.values_mut().collect(), num_threads, |sym| {
        let expr = replace_closure_calls_with_func_calls(sym.expr.as_ref().unwrap(), funcs);
        sym.expr = Some(expr);
    });
}

/// The uncurried functions defined in a program: the name of each, mapped to the name of the global
/// it uncurries and the number of arguments it takes.
///
/// # Examples
/// `g#func2` maps to `(g, 2)`.
pub type UncurriedFuncs = Map<FullName, (FullName, usize)>;

/// The symbol `sym_name`, whose definition is `sym`, followed by its uncurried functions taking one
/// argument, two arguments, and so on, as many as its type allows.
fn with_uncurried_funcs(sym_name: FullName, sym: Symbol) -> Vec<(FullName, Symbol)> {
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
            name.clone(),
            Symbol {
                name,
                generic_name,
                ty,
                expr: Some(expr),
                inline_into_callers: false,
            },
        ));
    }
    funcs.insert(0, (sym_name, sym));
    funcs
}

/// The uncurried symbol to use in place of `value`, an expression referring to a global value.
/// The uncurried function taking as many arguments as possible, up to `max_args`, is chosen.
fn uncurried_symbol<'a>(
    symbols: &'a Map<FullName, Symbol>,
    value: &Arc<ExprNode>,
    max_args: usize,
) -> Option<&'a Symbol> {
    let value_name = &value.get_var().name;
    if !value.type_.as_ref().unwrap().is_arrow() {
        return None;
    }
    for n_args in (1..=max_args).rev() {
        let mut name = value_name.clone();
        convert_to_uncurried_func_name(name.name_as_mut(), n_args);
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
/// * `funcs` — the uncurried functions the program defines. A call is rewritten only when
///   the uncurried function it would name is among them.
fn replace_closure_call_with_func_call(
    expr: &Arc<ExprNode>,
    funcs: &UncurriedFuncs,
) -> Arc<ExprNode> {
    let (fun, args) = collect_app(expr);
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
            let mut f_func = v.as_ref().clone();
            if fun_ty.is_func() {
                // An uncurried function of a global, called with its arguments and then the
                // arguments of what it returns: `g#func{N}(a1, ..., aN)(b1, ..., bM)` becomes
                // `g#func{N+M}(a1, ..., aN, b1, ..., bM)`.
                let Some((global, n_args)) = funcs.get(&f_func.name) else {
                    unreachable!(
                        "the global `{}` of a func type is not an uncurried function `run` defined",
                        f_func.name.to_string()
                    );
                };
                let n_args = *n_args;
                assert!(
                    args.len() >= n_args,
                    "`{}` takes {} arguments but is applied to {}",
                    f_func.name.to_string(),
                    n_args,
                    args.len()
                );
                if args.len() == n_args {
                    return expr.clone();
                }
                f_func.name = global.clone();
            }
            convert_to_uncurried_func_name(&mut f_func.name.name, args.len());
            if !funcs.contains_key(&f_func.name) {
                // The global has no uncurried function taking this many arguments, so the call
                // stays as it is.
                return expr.clone();
            }
            let result_ty = expr.type_.clone().unwrap();
            let arg_tys = args
                .iter()
                .map(|arg| arg.type_.clone().unwrap())
                .collect::<Vec<_>>();
            let func_ty = type_func(arg_tys, result_ty.clone());
            let f_func = expr_var(f_func.name, None).set_type(func_ty);
            expr_app(f_func, args, None).set_type(result_ty)
        }
        _ => expr.clone(),
    }
}

/// `expr` with every call of a global in it rewritten onto the uncurried function taking as many
/// arguments as the call supplies.
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
    use super::{is_std_fix, FullName, FIX_NAME, INSTANCIATED_NAME_SEPARATOR, STD_NAME};

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
