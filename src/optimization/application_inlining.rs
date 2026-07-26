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

An argument that is already a variable is pushed in as it is, so the `let` above appears only for an
argument that has something to evaluate. See `PushedArg`.

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
        expr::{
            expr_app_typed, expr_eval_typed, expr_if_typed, expr_let_typed, expr_match_typed,
            expr_var, var_var, Expr, ExprNode,
        },
        name::FullName,
        pattern::PatternNode,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::Set,
};

use super::rename::generate_new_names;

#[allow(dead_code)]
pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

pub fn run_on_symbol(sym: &mut Symbol) {
    let expr = sym.expr.as_ref().unwrap().clone();
    let expr = run_on_expr(expr);
    sym.expr = Some(expr);
}

pub fn run_on_expr(mut expr: Arc<ExprNode>) -> Arc<ExprNode> {
    while run_on_expr_once(&mut expr) {}
    expr
}

pub fn run_on_expr_once(expr: &mut Arc<ExprNode>) -> bool {
    let mut inliner = AppInliner {};
    let res = inliner.traverse(expr);
    *expr = res.expr;
    res.changed
}

struct AppInliner {}

// The value that the subexpressions of a function receive when an application is pushed into them,
// together with the binding, if any, that the rewritten expression must be wrapped in.
//
// A variable argument is handed over as it is: a variable reference has nothing to evaluate, so
// mentioning it once per branch keeps both the work done and the order it is done in. Any other
// argument is bound to a fresh name first, so it is evaluated exactly once, before the function's
// own body.
//
// Handing a variable over unchanged is what keeps this transformation linear. Binding every argument
// would add one binding for each `let` an application is pushed through, so pushing `n` arguments
// into a chain of `let`s — what uncurrying's eta expansion does to a function of `n` parameters —
// would grow the chain as `2^n`.
struct PushedArg {
    value: Arc<ExprNode>,
    binding: Option<(Arc<PatternNode>, Arc<ExprNode>)>,
}

impl PushedArg {
    // `func` is the function the application is pushed into, and `shadowed` are the names its
    // subexpressions are reached under; a variable argument among the latter would be captured
    // there, so it gets a binding of its own. A fresh binding avoids every name `func` mentions,
    // free or shadowing.
    fn new(arg: &Arc<ExprNode>, func: &Arc<ExprNode>, shadowed: &Set<FullName>) -> Self {
        if arg.is_var() && !shadowed.contains(&arg.get_var().name) {
            return PushedArg {
                value: arg.clone(),
                binding: None,
            };
        }
        let mut black_list = func.free_vars();
        black_list.extend(shadowed.iter().cloned());
        let ty = arg.type_.as_ref().unwrap().clone();
        let name = generate_new_names(&black_list, 1)[0].clone();
        let pat = PatternNode::make_var(var_var(name.clone()), None).set_type(ty.clone());
        PushedArg {
            value: expr_var(name, None).set_type(ty),
            binding: Some((pat, arg.clone())),
        }
    }

    // Wrap the rewritten expression in the argument's binding, for an argument that needed one.
    fn wrap(self, expr: Arc<ExprNode>) -> Arc<ExprNode> {
        match self.binding {
            Some((pat, bound)) => expr_let_typed(pat, bound, expr),
            None => expr,
        }
    }
}

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
                let shadowed = pattern.pattern.vars();
                let pushed = PushedArg::new(&arg, &func, &shadowed);

                let expr = expr_app_typed(value.clone(), vec![pushed.value.clone()]); // {value}(x)
                let expr = expr_let_typed(pattern.clone(), bound.clone(), expr); // let {pat} = {bound} in {value}(x)
                let expr = pushed.wrap(expr); // let x = {a} in let {pat} = {bound} in {value}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::If(cond, then, else_) => {
                // The expression is of the form `(if {cond} then {then} else {else})({a})`.
                // Replace it with `let x = {a} in if {cond} then {then}(x) else {else}(x)`.
                let pushed = PushedArg::new(&arg, &func, &Set::default());

                let then = expr_app_typed(then.clone(), vec![pushed.value.clone()]); // {then}(x)
                let else_ = expr_app_typed(else_.clone(), vec![pushed.value.clone()]); // {else}(x)
                let expr = expr_if_typed(cond.clone(), then, else_); // if {cond} then {then}(x) else {else}(x)
                let expr = pushed.wrap(expr); // let x = {a} in if {cond} then {then}(x) else {else}(x)
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Match(cond, pats_vals) => {
                // Similar to `if` and `let` cases. The argument is pushed under the patterns of the
                // arms, so the names they bind are what it must not be captured by.
                let mut shadowed = Set::default();
                for (pat, _val) in pats_vals {
                    shadowed.extend(pat.pattern.vars());
                }
                let pushed = PushedArg::new(&arg, &func, &shadowed);

                let mut pats_vals = pats_vals.clone();
                for (_pat, val) in &mut pats_vals {
                    let new_val = expr_app_typed(val.clone(), vec![pushed.value.clone()]);
                    *val = new_val;
                }
                let expr = expr_match_typed(cond.clone(), pats_vals);
                let expr = pushed.wrap(expr);
                return EndVisitResult::changed(expr).revisit();
            }
            Expr::Eval(side, main) => {
                // The expression is of the form `(eval {side} in {main})({a})`.
                // Replace it with `let x = {a} in eval {side} in {main}(x)`.
                let pushed = PushedArg::new(&arg, &func, &Set::default());

                let main_x = expr_app_typed(main.clone(), vec![pushed.value.clone()]); // {main}(x)
                let eval_expr = expr_eval_typed(side.clone(), main_x); // eval {side} in {main}(x)
                let expr = pushed.wrap(eval_expr); // let x = {a} in eval {side} in {main}(x)
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
            Expr::FFICall(_, _, _, _, _, _) => {
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
