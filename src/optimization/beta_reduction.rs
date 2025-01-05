/*
Beta reduction optimization.

This optimization tries to reduce cost of "create lambda and apply" expressions.

1. Moves application of variable into inner expression.

For example, if `v` is a variable,

```
(if c {{expr0}} else {{expr1}})(v)
```

is transformed into

```
if c {{expr0}(v)} else {{expr1}(v)}
```

2. Replaces application of lambda expression to a variable expression with substitution.

For example, if `v` is a variable,
```
(|x| {expr})(v)
```

is transformed into

```
{expr}[v/x]
```

Do the above transformations only if `v` is a variable, because if not, `v` is calculated many times and the optimization may increase the cost.
*/

use std::sync::Arc;

use crate::{
    ast::traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    expr_app_typed, expr_if_typed, expr_let_typed, expr_match_typed,
    optimization::utils::replace_free_var_of_expr,
    Expr, ExprNode, InstantiatedSymbol, Program,
};

use super::utils::rename_pattern_value_names;

#[allow(dead_code)]
pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.instantiated_symbols {
        run_on_symbol(sym);
    }
}

pub fn run_on_symbol(sym: &mut InstantiatedSymbol) {
    let mut optimizer = BetaReduction {};
    let res = optimizer.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.expr = Some(res.expr.calculate_free_vars());
    }
}

struct BetaReduction {}

impl ExprVisitor for BetaReduction {
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Get the argument of the application.
        let args = expr.get_app_args();
        if args.len() > 1 {
            // This optimiza does not support multiple arguments.
            return EndVisitResult::unchanged(expr);
        }
        let arg = args[0].clone();
        if !arg.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        let arg_var = arg.get_var();
        let arg_name = &arg_var.name;

        // Get the function applied to the argument.
        let func = expr.get_app_func();
        match &*func.expr {
            Expr::Lam(params, body) => {
                // The expression is of the form `(|x| {expr})(v)`.
                // Replace it with `{expr}[v/x]`.
                if params.len() != 1 {
                    // This optimization does not support multi-parameter lambdas.
                    return EndVisitResult::unchanged(expr);
                }
                let param_name = &params[0].name;
                let body = replace_free_var_of_expr(body, param_name, arg_name).unwrap();
                return EndVisitResult::changed(body).revisit();
            }
            Expr::Let(pattern, bound, value) => {
                // The expression is of the form `(let {pat} = {bound} in {value})(v)`.
                // Replace it with `let {pat} = {bound} in ({value}(v))`.

                // If `v` is in FV({pat}), we first rename `v` in `{pattern}` and `{value}` to a fresh variable.
                let (pattern, value) = rename_pattern_value_names(
                    &[arg_name.clone()].into_iter().collect(),
                    pattern.clone(),
                    value.clone(),
                );

                let value = expr_app_typed(value.clone(), vec![arg]);
                let expr = expr_let_typed(pattern, bound.clone(), value);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::If(cond, then, else_) => {
                // The expression is of the form `(if {cond} then {then} else {else})(v)`.
                // Replace it with `if {cond} then {then}(v) else {else}(v)`.
                let then = expr_app_typed(then.clone(), vec![arg.clone()]);
                let else_ = expr_app_typed(else_.clone(), vec![arg.clone()]);
                let expr = expr_if_typed(cond.clone(), then, else_);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Match(cond, pats_vals) => {
                // As with `let`, we need to rename `v` in each pattern and value to a fresh variable.
                let mut new_pats_vals = vec![];
                for (pat, val) in pats_vals {
                    let (pat, val) = rename_pattern_value_names(
                        &[arg_name.clone()].into_iter().collect(),
                        pat.clone(),
                        val.clone(),
                    );
                    let val = expr_app_typed(val, vec![arg.clone()]);
                    new_pats_vals.push((pat, val));
                }
                let expr = expr_match_typed(cond.clone(), new_pats_vals);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::App(_, _) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::Var(_) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::LLVM(_) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::TyAnno(_, _) => {
                // If remove tyanno optimization is done, this case should not happen.
                return EndVisitResult::unchanged(expr);
            }
            Expr::ArrayLit(_) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::MakeStruct(_, _) => {
                return EndVisitResult::unchanged(expr);
            }
            Expr::FFICall(_, _, _, _, _) => {
                return EndVisitResult::unchanged(expr);
            }
        }
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

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
        EndVisitResult::unchanged(expr)
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
        EndVisitResult::unchanged(expr)
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
        EndVisitResult::unchanged(expr)
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
        EndVisitResult::unchanged(expr)
    }
}
