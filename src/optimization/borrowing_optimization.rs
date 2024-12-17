use std::sync::Arc;

use crate::{
    ast::name::FullName,
    misc::{nonempty_subsequences, Map},
    Expr, ExprNode, GenerationContext, InstantiatedSymbol, Program,
};

// Borrowing optimization.
// Consider an application `f(x)`, where `f` is a function which takes the ownership of `x` but releases it in its body.
// Assume that the expression `x` is a variable, i.e., it is not a temporary value.
// Furthermore, assume that the variable `x` is used after `f(x)`, i.e., `x` is not released after `f(x)`.
// Borrowing optimization replaces `f(x)` to `f1(x1)`, where
// - `f1` is a the same function as `f` but does not release `x`.
// - `x1` is a borrowed (i.e., not retained) value for the variable `x`.

// Perform borrowing optimization.
pub fn borrowing_optimization(program: &mut Program) {
    // Define borrowing versions of each function if possible.
    define_borrowing_functions(program);

    // Set `released_params_indices` field for the function of each application expression.
    let instantiated_global_symbols = program.instantiated_symbols.clone();
    for (name, sym) in instantiated_global_symbols {
        let expr = sym.expr.as_ref().unwrap();
        let expr = set_released_param_indices(expr, program);
        program.instantiated_symbols.get_mut(&name).unwrap().expr = Some(expr);
    }

    // NOTE: Replacement of call expressions is handled in `borrowing_optimization_evaluating_application` which is called from `Generator::eval_app`.
}

// This function converts a name of a function to the name of a same function but it only borrows its argument (i.e., does not release the argument in its body).
pub fn convert_to_borrowing_function_name(
    name: &mut FullName,
    mut borrowed_params_indices: Vec<usize>,
) {
    borrowed_params_indices.sort();
    let name = name.name_as_mut();
    *name = name.clone()
        + "#borrowing_"
        + &borrowed_params_indices
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("_");
}

// Creates a borrowing version of a function expression.
// Returns None if the borrowing version cannot be created.
fn create_borrowing_function(
    function: &ExprNode,
    borrowed_args: Vec<FullName>,
) -> Option<Arc<ExprNode>> {
    let body = function.get_lam_body();
    // Currently, we handle only when `expr_body` is InlineLLVM.
    if !body.is_llvm() {
        return None;
    }
    let body = body.set_llvm_borrowed_vars(borrowed_args);
    let expr = function.set_lam_body(body);
    Some(expr)
}

// Adds a borrowing version of functions in a program.
pub fn define_borrowing_functions(program: &mut Program) {
    let mut new_functions: Map<FullName, InstantiatedSymbol> = Default::default();
    for (sym_name, sym) in &program.instantiated_symbols {
        let expr = sym.expr.as_ref().unwrap();
        if !expr.is_lam() {
            continue;
        }
        let params = expr.get_lam_params();
        let expr_body = expr.get_lam_body();
        // Currently, we handle only when `expr_body` is InlineLLVM.
        if !expr_body.is_llvm() {
            continue;
        }
        let llvm = expr_body.get_llvm();
        let released_vars = llvm.generator.released_vars();
        // If `released_vars` is not provided by the generator, we do not add borrowing function for it.
        if released_vars.is_none() {
            continue;
        }
        // For any subsequences of `released_vars`, try to define a borrowing version of the function.
        let released_vars = released_vars.unwrap();
        for borrowed_vars in nonempty_subsequences(&released_vars) {
            let expr = create_borrowing_function(expr, borrowed_vars.clone());
            if expr.is_none() {
                continue;
            }
            let expr = expr.unwrap();
            let mut name = sym_name.clone();
            let borrowed_params_indices = (0..params.len())
                .filter(|i| borrowed_vars.contains(&params[*i].name))
                .collect::<Vec<_>>();
            convert_to_borrowing_function_name(&mut name, borrowed_params_indices.clone());
            let mut generic_name = sym.generic_name.clone();
            convert_to_borrowing_function_name(&mut generic_name, borrowed_params_indices);
            new_functions.insert(
                name.clone(),
                InstantiatedSymbol {
                    instantiated_name: name.clone(),
                    generic_name: generic_name,
                    ty: sym.ty.clone(),
                    expr: Some(expr),
                },
            );
        }
    }
    program.instantiated_symbols.extend(new_functions);
}

pub fn borrowing_optimization_evaluating_application(
    gc: &mut GenerationContext,
    fun: Arc<ExprNode>,
    args: &Vec<Arc<ExprNode>>,
) -> Option<(Arc<ExprNode>, Vec<usize>)> {
    if fun.released_params_indices.is_none() {
        return None; // When `released_params_indices` is unknown, we can not perform borrowing optimization.
    }

    // Emulate the state of used_later values at the time when the argument is evaluated.
    for arg in args {
        gc.scope_lock_as_used_later(arg.free_vars());
    }

    // Get a list of arguments which CAN be borrowed here.
    // That is, arguments which are variables and are also used after evaluating the argument.
    let mut borrowable_args_indices: Vec<usize> = Default::default();
    for (i, arg) in args.iter().enumerate() {
        // Emulate the state of used_later values at the time when the argument is evaluated.
        gc.scope_unlock_as_used_later(arg.free_vars());
        // Proceed only for variables.
        if !arg.is_var() {
            continue;
        }
        let var_name = arg.get_var().name.clone();
        if !var_name.is_local() || gc.is_var_used_later(&var_name) {
            borrowable_args_indices.push(i);
        }
    }

    // Get a list of arguments which SHOULD be borrowed here.
    // Filter out arguments in `borrowed_args_indices` which are not released by the function.
    let borrowed_args_indices = borrowable_args_indices
        .iter()
        .cloned()
        .filter(|i| fun.released_params_indices.as_ref().unwrap().contains(i))
        .collect::<Vec<_>>();

    // Get borrowing version of the function.
    let mut borrowing_fun_name = fun.get_var().name.clone();
    convert_to_borrowing_function_name(&mut borrowing_fun_name, borrowed_args_indices.clone());

    // Check whether the borrowing version of the function is defined.
    if !gc.global.contains_key(&borrowing_fun_name) {
        return None;
    }

    // Return the borrowing version of the given function and the list of arguments which should be borrowed.
    let borrowing_fun_var = fun.get_var().set_name(borrowing_fun_name);
    let borrowing_fun = fun.set_var_var(borrowing_fun_var);
    Some((borrowing_fun, borrowed_args_indices))
}

// Set `released_params_indices` field for the function of each application expression.
fn set_released_param_indices(expr: &Arc<ExprNode>, program: &Program) -> Arc<ExprNode> {
    match &*expr.expr {
        Expr::Var(_) => expr.clone(),
        Expr::LLVM(_) => expr.clone(),
        Expr::App(fun, args) => {
            let args = args
                .iter()
                .map(|arg| set_released_param_indices(arg, program))
                .collect::<Vec<_>>();
            let mut fun = set_released_param_indices(fun, program);
            if fun.is_var() {
                let fun_name = fun.get_var().name.clone();
                if fun_name.is_global() {
                    let lam_expr = program
                        .instantiated_symbols
                        .get(&fun_name)
                        .unwrap()
                        .expr
                        .as_ref()
                        .unwrap()
                        .clone();
                    if lam_expr.is_lam() {
                        let (params, body) = lam_expr.destructure_lam();
                        assert_eq!(args.len(), params.len());
                        let released_params = body.released_vars();
                        if released_params.is_some() {
                            let released_params = released_params.unwrap();
                            let released_params_indices = (0..params.len())
                                .filter(|i| released_params.contains(&params[*i].name))
                                .collect::<Vec<_>>();
                            fun = fun.set_released_params_indices(released_params_indices);
                        }
                    }
                }
            }
            expr.set_app_func(fun).set_app_args(args)
        }
        Expr::Lam(_, body) => expr.set_lam_body(set_released_param_indices(body, program)),
        Expr::Let(_, bound, value) => expr
            .set_let_bound(set_released_param_indices(bound, program))
            .set_let_value(set_released_param_indices(value, program)),
        Expr::If(c, t, e) => expr
            .set_if_cond(set_released_param_indices(c, program))
            .set_if_then(set_released_param_indices(t, program))
            .set_if_else(set_released_param_indices(e, program)),
        Expr::Match(cond, pat_vals) => {
            let cond = set_released_param_indices(cond, program);
            let mut new_pat_vals = vec![];
            for (pat, val) in pat_vals {
                let val = set_released_param_indices(val, program);
                new_pat_vals.push((pat.clone(), val));
            }
            expr.set_match_cond(cond).set_match_pat_vals(new_pat_vals)
        }
        Expr::TyAnno(e, _) => expr.set_tyanno_expr(set_released_param_indices(e, program)),
        Expr::ArrayLit(elems) => {
            let mut expr = expr.clone();
            for (i, e) in elems.iter().enumerate() {
                expr = expr.set_array_lit_elem(set_released_param_indices(e, program), i)
            }
            expr
        }
        Expr::MakeStruct(_, fields) => {
            let fields = fields.clone();
            let mut expr = expr.clone();
            for (field_name, field_expr) in fields {
                let field_expr = set_released_param_indices(&field_expr, program);
                expr = expr.set_make_struct_field(&field_name, field_expr);
            }
            expr
        }
        Expr::FFICall(_, _, _, args, _) => {
            let mut expr = expr.clone();
            for (i, e) in args.iter().enumerate() {
                expr = expr.set_ffi_call_arg(set_released_param_indices(e, program), i)
            }
            expr
        }
    }
}
