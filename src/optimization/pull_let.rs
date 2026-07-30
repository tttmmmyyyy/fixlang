/*
# Overview

`Pulls let` transformation.
This transformation is used to increase the number of places where decapturing optimization can be applied.

# Transformation

This pass performs the following transformations:

(1)
Transforms

```
let {pat0} = (
    let {pat1} = {expr0};
    {expr1}
);
{expr2}
```

to

```
let {pat1'} = {expr0};
let {pat0} = {expr1'};
{expr2}
```

Here, `{pat1'}` and `{expr1'}` are the same as `{pat1}` and `{expr1}`, but with all variables in `{pat1}` renamed to avoid conflicts with free variables in `{expr2}`,

(2)
Transforms

```
{expr0}({non-variable-expr})
```

to

```
let f = {non-variable-expr};
{expr0}(f)
```

where `f` is a new name that does not conflict with any free variables in `{expr0}`.

(3)

Transforms

```
(let {pat} = {expr0}; {expr1})({expr2})
```

to

```
let {pat'} = {expr0};
{expr1'}({expr2})
```

where `{pat'}` and `{expr1'}` are the same as `{pat}` and `{expr1}`, but with all variables in `{pat}` renamed to avoid conflicts with free variables in `{expr2}`.

# Expected Effects

(1)
As described in the comment for decapturing optimization, the following code can be optimized with decapturing:

```
let f = |x| x + n;
```

On the other hand, decapturing optimization cannot be applied to the following code:

```
let f = (
    let n = m;
    |x| x + n
);
```

After applying the pull-let transformation (1), the second code can be transformed into a form that can be applied with decapturing optimization.

(2) and (3)
The following code can be optimized with decapturing optimization

```
let f = |i, s| s + n;
it.fold(s0, f)
```

into

```
let f = #DecapF { n : n };
it.fold#lamf(s0, f)
```

On the other hand, decapturing optimization cannot be applied to the following code:

```
it.fold(s0, |i, s| s + n)
```

After applying the pull-let transformation (2) and (3), the second code can be transformed into a form that can be applied with decapturing optimization.

# Evaluation order

Transformations (2) and (3) also set the order in which a call evaluates its arguments: the `let`s
leave an application from the outside in, and the argument bound by the outermost `let` is evaluated
first. A call written `f(x, y)` nests as `f(x)(y)` and one written `x.f(y)` nests as `f(y)(x)`, so
`ExprNode::app_order` is what says whether the function position or the argument carries the
argument written first. This pass takes that one first, so the `let` chain it leaves behind holds the
arguments in the order they are written. A pass that moves an expression afterwards can still change
the order the program ends up evaluating them in.

*/

use crate::{
    ast::{
        expr::{expr_let_typed, expr_var, var_var, AppSourceCodeOrderType, Expr, ExprNode},
        pattern::PatternNode,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    optimization::rename::{generate_new_names, rename_pattern_value_avoiding},
};
use std::sync::Arc;

/// Transformation (2): `{expr0}({non-variable-expr})` to `let f = {non-variable-expr}; {expr0}(f)`.
fn pull_argument_into_let(app: &Arc<ExprNode>) -> Arc<ExprNode> {
    let fun = app.get_app_func();
    let arg = app.get_app_args()[0].clone();

    let f_name = generate_new_names(&fun.free_vars(), 1)[0].clone();
    let arg_ty = arg.type_.as_ref().unwrap();
    let f_pat = PatternNode::make_var(var_var(f_name.clone()), None).set_type(arg_ty.clone());
    let f_var = expr_var(f_name, None).set_type(arg_ty.clone());

    expr_let_typed(f_pat, arg, app.set_app_args(vec![f_var]))
}

/// Transformation (3): `(let {pat} = {expr0}; {expr1})({expr2})` to
/// `let {pat'} = {expr0}; {expr1'}({expr2})`.
fn pull_let_out_of_function(app: &Arc<ExprNode>) -> Arc<ExprNode> {
    let fun = app.get_app_func();
    let arg = app.get_app_args()[0].clone();

    let expr0 = fun.get_let_bound();
    let expr1 = fun.get_let_value();
    let pat = fun.get_let_pat();

    // Rename `pat` and `expr1` to avoid conflicts with free variables in the argument.
    let black_list = arg.free_vars();
    let (pat, expr1) = rename_pattern_value_avoiding(&black_list, pat, expr1);

    expr_let_typed(pat, expr0, app.set_app_func(expr1))
}

/// Whether the function position of an application still holds a `let` for this pass to pull out:
/// an application on its spine whose argument is not a variable yet, or a `let` that transformation
/// (3) hoists.
fn has_something_to_pull(fun: &Arc<ExprNode>) -> bool {
    match fun.expr.as_ref() {
        Expr::App(inner_fun, args) => {
            args.iter().any(|arg| !arg.is_var()) || has_something_to_pull(inner_fun)
        }
        Expr::Let(_, _, _) => true,
        _ => false,
    }
}

pub fn run_on_expr(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let mut expr = expr.clone();
    while run_on_expr_once(&mut expr) {}
    expr
}

// Run pull-let transformation once on the given expression.
//
// If any transformation is applied, returns true.
pub fn run_on_expr_once(expr: &mut Arc<ExprNode>) -> bool {
    let mut pull_let = PullLet {};
    let res = pull_let.traverse(expr);
    *expr = res.expr;
    res.changed
}

struct PullLet {}

impl ExprVisitor for PullLet {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let fun = expr.get_app_func();
        let args = expr.get_app_args();
        assert_eq!(
            args.len(),
            1,
            "an application of {} arguments reached the pull-let transformation",
            args.len()
        );
        let arg = &args[0];

        // The `let`s leave an application in the order the call evaluates its arguments in, so
        // which of the function position and this argument is taken first is what sets that order.
        // `f(x, y)` nests as `f(x)(y)`, which puts the argument written first on the inner
        // application: settle the function position, then pull this one. `x.f(y)` nests as
        // `f(y)(x)`, which puts the argument written first on this application.
        if expr.app_order == AppSourceCodeOrderType::FX {
            if fun.is_let() {
                return StartVisitResult::ReplaceAndRevisit(pull_let_out_of_function(expr));
            }
            if has_something_to_pull(&fun) {
                return StartVisitResult::VisitChildren;
            }
        }

        if !arg.is_var() {
            return StartVisitResult::ReplaceAndRevisit(pull_argument_into_let(expr));
        }

        if fun.is_let() {
            return StartVisitResult::ReplaceAndRevisit(pull_let_out_of_function(expr));
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Check if the transformation (1) can be applied.
        let bound = expr.get_let_bound();
        if !bound.is_let() {
            return StartVisitResult::VisitChildren;
        }

        // Then, the transformation (1) can be applied.
        let pat0 = expr.get_let_pat();
        let pat1 = bound.get_let_pat();
        let expr0 = bound.get_let_bound();
        let expr1 = bound.get_let_value();
        let expr2 = expr.get_let_value();

        // Rename `pat1` and `expr1` to avoid conflicts with free variables in `expr2`.
        let black_list = expr2.free_vars();
        let (pat1, expr1) = rename_pattern_value_avoiding(&black_list, pat1, expr1);

        // Construct the new expression.
        let expr = expr_let_typed(pat1, expr0, expr_let_typed(pat0, expr1, expr2));

        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
