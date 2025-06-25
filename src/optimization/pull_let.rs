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

*/

use std::sync::Arc;

use crate::{
    ast::{
        expr::{expr_app_typed, expr_let_typed, expr_var, var_var, ExprNode},
        pattern::PatternNode,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    optimization::utils::{generate_new_names, rename_pattern_value_avoiding},
};

pub fn run_on_expr(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let mut pull_let = PullLet {};
    let mut expr = expr.calculate_free_vars();
    loop {
        let res = pull_let.traverse(&expr.calculate_free_vars());
        if !res.changed {
            return expr;
        }
        expr = res.expr;
    }
}

struct PullLet {}

impl ExprVisitor for PullLet {
    fn start_visit_var(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let fun = expr.get_app_func();
        let args = expr.get_app_args();
        assert_eq!(args.len(), 1);
        let arg = &args[0];

        if !arg.is_var() {
            // Apply the transformation (2).
            let f_name = generate_new_names(fun.free_vars(), 1)[0].clone();
            let arg_ty = arg.ty.as_ref().unwrap();
            let f_pat =
                PatternNode::make_var(var_var(f_name.clone()), None).set_type(arg_ty.clone());
            let f_var = expr_var(f_name, None).set_inferred_type(arg_ty.clone());
            let expr = expr_let_typed(f_pat, arg.clone(), expr_app_typed(fun, vec![f_var]))
                .calculate_free_vars();
            return StartVisitResult::ReplaceAndRevisit(expr);
        }

        if fun.is_let() {
            // Apply the transformation (3).
            let expr0 = fun.get_let_bound();
            let expr1 = fun.get_let_value();
            let pat = fun.get_let_pat();
            let expr2 = arg.clone();

            // Rename `pat` and `expr1` to avoid conflicts with free variables in `expr2`.
            let ng_names = expr2.free_vars();
            let (pat, expr1) = rename_pattern_value_avoiding(ng_names, pat, expr1);

            // Construct the new expression.
            let expr = expr_let_typed(pat, expr0, expr_app_typed(expr1, vec![expr2]))
                .calculate_free_vars();

            return StartVisitResult::ReplaceAndRevisit(expr);
        }

        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
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
        let ng_names = expr2.free_vars();
        let (pat1, expr1) = rename_pattern_value_avoiding(ng_names, pat1, expr1);

        // Construct the new expression.
        let expr =
            expr_let_typed(pat1, expr0, expr_let_typed(pat0, expr1, expr2)).calculate_free_vars();

        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
