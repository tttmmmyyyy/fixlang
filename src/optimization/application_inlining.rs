/*
Application inlining.

This optimization tries to reduce cost of "create lambda and apply" expressions.

1. Moves application into inner.

For example, in case of `if` expression,

```
(if c {{expr0}} else {{expr1}})({expr2})
```

is transformed into

```
let v = {expr2} in if c {{expr0}(v)} else {{expr1}(v)}
```

2. Replaces application of lambda expression to an expression with let binding.

The expression

```
(|x| {expr0})({expr1})
```

is transformed into

```
let x = {expr1} in {expr0}
```
*/

use std::sync::Arc;

use crate::{
    ast::{
        expr::expr_eval_typed,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    expr_app_typed, expr_if_typed, expr_let_typed, expr_match_typed, expr_var, var_var, Expr,
    ExprNode, PatternNode, Program, Symbol,
};

use super::utils::generate_new_names;

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

pub fn run_on_symbol(sym: &mut Symbol) {
    let mut expr = sym.expr.as_ref().unwrap().clone();
    while run_on_expr_once(&mut expr) {}
    sym.expr = Some(expr);
}

pub fn run_on_expr_once(expr: &mut Arc<ExprNode>) -> bool {
    let mut optimizer = AppInliner {};
    let res = optimizer.traverse(expr);
    *expr = res.expr;
    res.changed
}

struct AppInliner {}

impl ExprVisitor for AppInliner {
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Get the argument of the application.
        let args = expr.get_app_args();
        if args.len() > 1 {
            // This optimiza does not support multiple arguments.
            return EndVisitResult::unchanged(expr);
        }
        let arg = args[0].clone();

        // Get the function applied to the argument.
        let func = expr.get_app_func();
        match &*func.expr {
            Expr::Lam(params, body) => {
                // The expression is of the form `(|x| {expr})({a})`.
                // Replace it with `let x = {a} in {expr}`.
                if params.len() != 1 {
                    // This optimization does not support multi-parameter lambdas.
                    return EndVisitResult::unchanged(expr);
                }
                let param = &params[0];
                let pat = PatternNode::make_var(param.clone(), None)
                    .set_type(arg.type_.as_ref().unwrap().clone());
                let expr = expr_let_typed(pat, arg, body.clone());
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Let(pattern, bound, value) => {
                // The expression is of the form `(let {pat} = {bound} in {value})({a})`.
                // Replace it with `let x = {a} in let {pat} = {bound} in {value}(x)`.

                let mut black_list = pattern.pattern.vars();
                black_list.extend(bound.free_vars().into_iter());
                black_list.extend(value.free_vars().into_iter());

                let x_name = generate_new_names(&black_list, 1)[0].clone();
                let x_pat = PatternNode::make_var(var_var(x_name.clone()), None)
                    .set_type(arg.type_.as_ref().unwrap().clone());
                let x = expr_var(x_name, None).set_type(arg.type_.as_ref().unwrap().clone());

                let expr = expr_app_typed(value.clone(), vec![x]); // {value}(x)
                let expr = expr_let_typed(pattern.clone(), bound.clone(), expr); // let {pat} = {bound} in {value}(x)
                let expr = expr_let_typed(x_pat, arg.clone(), expr); // let x = {a} in let {pat} = {bound} in {value}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::If(cond, then, else_) => {
                // The expression is of the form `(if {cond} then {then} else {else})({a})`.
                // Replace it with `let x = {a} in if {cond} then {then}(x) else {else}(x)`.
                let mut black_list = cond.free_vars().clone();
                black_list.extend(then.free_vars().into_iter());
                black_list.extend(else_.free_vars().into_iter());

                let x_name = generate_new_names(&black_list, 1)[0].clone();
                let x_pat = PatternNode::make_var(var_var(x_name.clone()), None)
                    .set_type(arg.type_.as_ref().unwrap().clone());
                let x = expr_var(x_name, None).set_type(arg.type_.as_ref().unwrap().clone());

                let then = expr_app_typed(then.clone(), vec![x.clone()]); // {then}(x)
                let else_ = expr_app_typed(else_.clone(), vec![x.clone()]); // {else}(x)
                let expr = expr_if_typed(cond.clone(), then, else_); // if {cond} then {then}(x) else {else}(x)
                let expr = expr_let_typed(x_pat, arg.clone(), expr); // let x = {a} in if {cond} then {then}(x) else {else}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Match(cond, pats_vals) => {
                // Similar to `if` and `let` cases.
                let mut black_list = cond.free_vars().clone();
                for (pat, val) in pats_vals {
                    black_list.extend(pat.pattern.vars());
                    black_list.extend(val.free_vars().into_iter());
                }

                let x_name = generate_new_names(&black_list, 1)[0].clone();
                let x_pat = PatternNode::make_var(var_var(x_name.clone()), None)
                    .set_type(arg.type_.as_ref().unwrap().clone());
                let x = expr_var(x_name, None).set_type(arg.type_.as_ref().unwrap().clone());

                let mut pats_vals = pats_vals.clone();
                for (_pat, val) in &mut pats_vals {
                    let new_val = expr_app_typed(val.clone(), vec![x.clone()]);
                    *val = new_val;
                }
                let expr = expr_match_typed(cond.clone(), pats_vals);
                let expr = expr_let_typed(x_pat, arg.clone(), expr);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Eval(side, main) => {
                // The expression is of the form `(eval {side} in {main})({a})`.
                // Replace it with `let x = {a} in eval {side} in {main}(x)`.
                let mut black_list = side.free_vars().clone();
                black_list.extend(main.free_vars().into_iter());
                let x_name = generate_new_names(&black_list, 1)[0].clone();
                let x_pat = PatternNode::make_var(var_var(x_name.clone()), None)
                    .set_type(arg.type_.as_ref().unwrap().clone());
                let x = expr_var(x_name, None).set_type(arg.type_.as_ref().unwrap().clone());
                let main_x = expr_app_typed(main.clone(), vec![x]); // {main}(x)
                let eval_expr = expr_eval_typed(side.clone(), main_x); // eval {side} in {main}(x)
                let expr = expr_let_typed(x_pat, arg.clone(), eval_expr); // let x = {a} in eval {side} in {main}(x)
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

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
