/*
Eta expansion optimization.

Consider following global definition:

```
f : T -> S;
f = {expr};
```

Eta expansion optimization replaces the above definition with the following definition:

```
f : T -> S;
f = |x| {expr}(x);
```

More generally, if the type signature of `f` has n inputs, but the right hand side of the definition has m inputs, where m < n,
then this optimization replaces the definition with a lambda expression with n parameters.

This allows other optimizations such as uncurrying to be applied well.

This optimization is not applied to multi-parameter lambdas.
Since multi-parameter lambdas are generated by the uncurrying optimization, this optimization should be applied before the uncurrying optimization.
*/

use crate::{
    ast::name::FullName, expr_abs_typed, expr_app_typed, expr_var, misc::warn_msg, var_local,
    Program, Symbol,
};

use super::uncurry::is_std_fix;

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

// Run the optimization on a symbol.
fn run_on_symbol(sym: &mut Symbol) {
    if is_std_fix(&sym.name) {
        // We currently exclude `Std::fix` function.
        // `fix` is defined as `|f||x| LLVM[fix(f,x)]`.
        // `LLVM[fix(f,x)]` uses `get_insert_block().get_parent()` to get the function of `|x| LLVM[fix(f,x)]`, which is the function of `fix(f)`.
        // If we apply eta expansion to `fix : ((S -> T -> U) -> (S -> T -> U)) -> (S -> T -> U)`,
        // then the definition is changed to `fix = |f||x||p| LLVM[fix(f,x)](p)`.
        // Then the result of `get_insert_block().get_parent()` becomes the function of `|p| LLVM[fix(f,x)](p)`, which is not the function of `fix(f)`.
        return;
    }

    let ty = sym.ty.clone();
    let expr = sym.expr.clone().unwrap();

    // Count the number of parameters in the lambda expression and in the type signature.
    let (params, body) = expr.destructure_lam_sequence();
    // Since this opt does not support multi-parameter lambdas, we check if each lambda has only one parameter.
    let mut params_new = vec![];
    for param in params {
        if param.len() != 1 {
            warn_msg("Eta expansion found multi-parameter lambda. Skipping the optimization. Please report this issue.");
            return;
        }
        params_new.push(param[0].clone());
    }
    let mut params = params_new;
    let (doms_tys, _codom_ty) = ty.collect_app_src(usize::MAX);

    // If there is enough parameters, then we do not need to do anything.
    if params.len() >= doms_tys.len() {
        return;
    }

    // Determine the names of the additional parameters.
    // We need to avoid the name conflicts with existing parameters.
    let mut names_set = params
        .iter()
        .map(|param| param.name.to_string())
        .collect::<std::collections::HashSet<_>>();
    let num_additional_params = doms_tys.len() - params.len();
    let mut additional_params = vec![];

    // TODO: refactor this part to use `generate_new_names` function.
    let mut var_name_no = 0;
    for _ in 0..num_additional_params {
        let var_name = loop {
            let var_name = format!("#v{}", var_name_no);
            if !names_set.contains(&var_name) {
                break var_name;
            }
            var_name_no += 1;
        };
        additional_params.push(FullName::local(&var_name));
        names_set.insert(var_name);
    }

    // Create the new lambda expression.
    let mut body = body;
    for (i, param) in additional_params.iter().enumerate() {
        // Get the type of the additional parameter.
        let var_ty = doms_tys[params.len() + i].clone();

        // Create the variable expression of the additional parameter.
        let var = expr_var(param.clone(), None).set_inferred_type(var_ty);

        // Create the application expression `{body}({var})`.
        body = expr_app_typed(body, vec![var]);
    }

    // Create the list of all parameters by concatenating `params` and `additional_params`.
    params.extend(
        additional_params
            .into_iter()
            .map(|param| var_local(&param.to_string())),
    );

    // Abstract `body` by `params` in reverse order.
    let mut expr = body;
    for (var, ty) in params.into_iter().zip(doms_tys.into_iter()).rev() {
        expr = expr_abs_typed(var, ty, expr);
    }

    let expr = expr.calculate_free_vars();
    sym.expr = Some(expr);
}
