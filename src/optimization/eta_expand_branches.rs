/*
# Eta expansion of a branch that chooses a function

## Overview

This transformation rewrites a `let` binding a function that an `if` or a `match` chooses, and that
is applied once:

```
let m = if c { {e0} } else { {e1} } in {e2}
```

is transformed into

```
let b = c in let m = |x| (if b { {e0} } else { {e1} })(x) in {e2}
```

and

```
let m = match s { p0 => {e0}, p1 => {e1} } in {e2}
```

into

```
let v = s in let m = |x| (match v { p0 => {e0}, p1 => {e1} })(x) in {e2}
```

## Purpose

A value of a monad such as `IO` is a function once `unwrap_newtype` has taken its struct apart, so
`if c { print("a") } else { print("b") }` is an `if` choosing between two lambdas. Bound by a `let`
and applied inside the lambda that `bind` builds for what follows, it is applied where neither
`application_inlining` nor `let_elimination` reaches: the application's function is a name, and
`let_elimination` moves a `let` into a lambda only where it binds a lambda. The `if` is left
building the closure of the branch taken, at every evaluation, and the application calls it through
a function pointer.

After this transformation, `m` binds a lambda applied once, which `let_elimination` moves to the
application (its condition 2-a), and `application_inlining` then pushes the application into the
branches and reduces it against the lambda each branch ends in. No closure is built.

## Conditions

The rewrite moves the branches under a lambda, where they are evaluated at the application instead
of at the `let`. It fires only where no value holding a reference count lives longer for it, since
such a value living longer can turn an in-place update of it into a copy: an update reads the value
shared while the longer-lived reference stands.

1. The condition of an `if`, or the matched value of a `match`, is evaluated where it was, and bound
   to a name, so the lambda reads only that name.
2. Nothing the lambda captures holds a reference count. The lambda captures every name the
   branches read from outside them, and the whole matched value of a `match`; each of these has a
   type that holds no reference count. The lambda keeps them until the application, while the
   closure of the branch taken kept only what that branch read, and only the parts of the matched
   value its pattern bound. Keeping a value that holds no reference count longer changes no
   reference count. The passes after this one only take reads away from a branch, so the lambda
   they leave still captures nothing that holds one.
3. Each branch is a chain of `let`s binding a name, a constant, a lambda or an unboxed struct of
   those, and ends in a name or a lambda. What a branch evaluates before the lambda it ends in
   moves under the lambda, so it is limited to what costs nothing to evaluate again. What it reads
   from outside the branch is what condition 2 covers.
4. `m` is applied once, which is what lets `let_elimination` move it to the application. Without
   it, the rewrite leaves the closure as it was.
*/

use crate::{
    ast::{
        expr::{expr_abs_typed, expr_app_typed, expr_let_typed, expr_var, var_var, Expr, ExprNode},
        name::FullName,
        program::TypeEnv,
        traverse::{EndVisitResult, ExprVisitor, VisitState},
        types::TypeNode,
    },
    misc::Set,
    optimization::{
        let_elimination,
        rename::{generate_new_names, names_bound_in, NamedExpr},
    },
};
use std::sync::Arc;

/// Runs the transformation once over `expr`, and answers whether it rewrote anything.
pub fn run_on_expr_once(expr: &mut Arc<ExprNode>, type_env: &TypeEnv) -> bool {
    let mut taken = names_bound_in(expr);
    taken.extend(expr.free_vars());
    let mut expander = BranchEtaExpander { type_env, taken };
    let res = expander.traverse(expr);
    *expr = res.expr;
    res.changed
}

/// The walk that rewrites each `let` the module documentation describes.
struct BranchEtaExpander<'a> {
    /// The types of the program, which say whether a value holds a reference count.
    type_env: &'a TypeEnv,
    /// Every name the expression binds or reads, and every name the walk has created. A new name
    /// avoids all of them, so the names the rewrites of a chain of `let`s create differ from one
    /// another, and moving one rewritten `let` past another renames nothing.
    taken: Set<FullName>,
}

impl<'a> ExprVisitor for BranchEtaExpander<'a> {
    /// Rewrites a `let` binding a function an `if` or a `match` chooses, where the conditions of the
    /// module documentation hold.
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        match self.expand(expr, state) {
            Some(expr) => EndVisitResult::changed(expr),
            None => EndVisitResult::unchanged(expr),
        }
    }
}

impl<'a> BranchEtaExpander<'a> {
    /// The rewritten `let`, for a `let` of the form the module documentation describes that meets
    /// its conditions.
    fn expand(&mut self, expr: &Arc<ExprNode>, state: &VisitState) -> Option<Arc<ExprNode>> {
        let pat = expr.get_let_pat();
        if !pat.is_var() {
            return None;
        }
        let bound = expr.get_let_bound();
        let ty = bound.type_.clone().unwrap();
        if !ty.is_closure() {
            return None;
        }
        let (subject, set_subject): (_, fn(&ExprNode, Arc<ExprNode>) -> Arc<ExprNode>) =
            match &*bound.expr {
                Expr::If(cond, then_expr, else_expr) => {
                    for branch in [then_expr, else_expr] {
                        if !self.is_cheap(branch)
                            || self.reads_counted_name(branch, &Set::default(), state)
                        {
                            return None;
                        }
                    }
                    (cond, ExprNode::set_if_cond)
                }
                Expr::Match(scrutinee, arms) => {
                    if self.holds_count(scrutinee.type_.as_ref().unwrap()) {
                        return None;
                    }
                    for (arm_pat, arm_val) in arms {
                        if !self.is_cheap(arm_val)
                            || self.reads_counted_name(arm_val, &arm_pat.pattern.vars(), state)
                        {
                            return None;
                        }
                    }
                    (scrutinee, ExprNode::set_match_cond)
                }
                _ => return None,
            };
        // Condition 4.
        if !let_elimination::is_applied_once(&pat.get_var().name, &expr.get_let_value()) {
            return None;
        }

        let subject = NamedExpr::new(subject, &self.taken);
        self.taken.insert(subject.value.get_var().name.clone());
        let chosen = set_subject(&bound, subject.value.clone());
        let param = generate_new_names(&self.taken, 1)[0].clone();
        self.taken.insert(param.clone());
        Some(subject.wrap(expanded_let(expr, chosen, &ty, param)))
    }

    /// Does `branch` read, from outside it, a local name whose type holds a reference count? The
    /// names in `bound_by_pattern` are bound for the branch, so they are left out.
    fn reads_counted_name(
        &self,
        branch: &Arc<ExprNode>,
        bound_by_pattern: &Set<FullName>,
        state: &VisitState,
    ) -> bool {
        branch
            .free_vars()
            .into_iter()
            .filter(|name| name.is_local() && !bound_by_pattern.contains(name))
            .any(|name| {
                let ty = state.scope.get_local(&name.name).unwrap().unwrap();
                self.holds_count(&ty)
            })
    }

    /// Is `branch` a chain of `let`s binding a name, a constant, a lambda or an unboxed struct of
    /// those, ending in a name or a lambda? What such a branch evaluates is free to evaluate again
    /// (condition 3).
    fn is_cheap(&self, branch: &Arc<ExprNode>) -> bool {
        match &*branch.expr {
            Expr::Var(_) | Expr::Lam(_, _) => true,
            Expr::Let(_, bound, value) => self.is_cheap_bound(bound) && self.is_cheap(value),
            _ => false,
        }
    }

    /// Is `bound` a name, a constant, a lambda, an unboxed struct of those, or a chain of `let`s of
    /// those ending in one?
    fn is_cheap_bound(&self, bound: &Arc<ExprNode>) -> bool {
        match &*bound.expr {
            Expr::Var(_) | Expr::Lam(_, _) => true,
            Expr::Builtin(builtin) => builtin.op.is_free_to_duplicate(),
            Expr::MakeStruct(_, fields) => {
                bound.type_.as_ref().unwrap().is_unbox(self.type_env)
                    && fields
                        .iter()
                        .all(|(_name, _src, field)| self.is_cheap_bound(field))
            }
            Expr::Let(_, inner_bound, value) => {
                self.is_cheap_bound(inner_bound) && self.is_cheap_bound(value)
            }
            _ => false,
        }
    }

    /// Does a value of `ty` hold a reference count?
    fn holds_count(&self, ty: &Arc<TypeNode>) -> bool {
        !ty.is_fully_unboxed(self.type_env)
    }
}

/// `let m = |x| {chosen}(x) in {value}`, for the `let m = .. in {value}` that `expr` is, where `m`
/// has type `ty` and `x` is `param`.
fn expanded_let(
    expr: &Arc<ExprNode>,
    chosen: Arc<ExprNode>,
    ty: &Arc<TypeNode>,
    param: FullName,
) -> Arc<ExprNode> {
    let param_ty = ty.get_lambda_srcs()[0].clone();
    let arg = expr_var(param.clone(), None).set_type(param_ty.clone());
    let lam = expr_abs_typed(var_var(param), param_ty, expr_app_typed(chosen, vec![arg]));
    expr_let_typed(expr.get_let_pat(), lam, expr.get_let_value())
}
