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
of at the `let`. It fires only where that leaves every value's lifetime as it was, since a value
living longer can turn an in-place update of it into a copy: an update reads the value shared while
the longer-lived reference stands.

1. The condition of an `if`, or the matched value of a `match`, is evaluated where it was, and bound
   to a name, so the lambda reads only that name.
2. Each name a branch reads from outside it, of a type holding a reference count, is read by every
   branch. The lambda captures what all the branches read, while the closure of the branch taken
   captured only what that branch read.
3. Each branch is a chain of `let`s, each binding one name that the rest of the branch reads to a
   name, a constant, a lambda or an unboxed struct of those, and ends in a name or a lambda. What a
   branch evaluates before the lambda it ends in moves under the lambda, so it is limited to what
   costs nothing to evaluate again, and what it reads is kept by the lambda the branch ends in, as
   the closure of the branch kept it.
4. `m` is applied once, which is what lets `let_elimination` move it to the application. Without
   it, the rewrite leaves the closure as it was.

A `match` takes two more:

5. Each pattern binds every part of the matched value that holds a reference count, and the branch
   reads each name doing so. The lambda captures the matched value whole, where the closure of the
   branch taken captured only what its pattern bound and its body read: a part the pattern drops
   was released at the `match`, and would live under the lambda until the application.
6. The matched value is of an unboxed type. The box of a boxed value is an object of its own that
   other places may point at, and the lambda holding it would make an in-place update through
   them a copy.
*/

use crate::{
    ast::{
        expr::{expr_abs_typed, expr_app_typed, expr_let_typed, expr_var, var_var, Expr, ExprNode},
        name::FullName,
        pattern::{Pattern, PatternNode},
        program::TypeEnv,
        traverse::{EndVisitResult, ExprVisitor, VisitState},
        types::TypeNode,
    },
    misc::Set,
    optimization::{
        let_elimination,
        rename::{generate_new_names, NamedExpr},
    },
};
use std::sync::Arc;

/// Runs the transformation once over `expr`, and answers whether it rewrote anything.
pub fn run_on_expr_once(expr: &mut Arc<ExprNode>, type_env: &TypeEnv) -> bool {
    let mut expander = BranchEtaExpander { type_env };
    let res = expander.traverse(expr);
    *expr = res.expr;
    res.changed
}

/// The walk that rewrites each `let` the module documentation describes.
struct BranchEtaExpander<'a> {
    /// The types of the program, which say whether a value holds a reference count.
    type_env: &'a TypeEnv,
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
    /// The rewritten `let`, or `None` where `expr` is not of the form the module documentation
    /// describes or a condition fails.
    fn expand(&self, expr: &Arc<ExprNode>, state: &VisitState) -> Option<Arc<ExprNode>> {
        let pat = expr.get_let_pat();
        if !pat.is_var() {
            return None;
        }
        let bound = expr.get_let_bound();
        let ty = bound.type_.clone().unwrap();
        if !ty.is_closure() {
            return None;
        }
        let (subject, read_by_branch, set_subject): (
            _,
            _,
            fn(&ExprNode, Arc<ExprNode>) -> Arc<ExprNode>,
        ) = match &*bound.expr {
            Expr::If(cond, then_expr, else_expr) => {
                let branches = [then_expr, else_expr];
                if !branches.iter().all(|branch| self.is_cheap(branch)) {
                    return None;
                }
                let read_by_branch = branches
                    .iter()
                    .map(|branch| self.counted_names_read(branch, &Set::default(), state))
                    .collect::<Vec<_>>();
                (cond, read_by_branch, ExprNode::set_if_cond)
            }
            Expr::Match(scrutinee, arms) => {
                // Condition 6.
                if !scrutinee.type_.as_ref().unwrap().is_unbox(self.type_env) {
                    return None;
                }
                let mut read_by_branch = vec![];
                for (arm_pat, arm_val) in arms {
                    if !self.is_cheap(arm_val) {
                        return None;
                    }
                    // Condition 5.
                    if !self.binds_every_counted_part(arm_pat, arm_val) {
                        return None;
                    }
                    read_by_branch.push(self.counted_names_read(
                        arm_val,
                        &arm_pat.pattern.vars(),
                        state,
                    ));
                }
                (scrutinee, read_by_branch, ExprNode::set_match_cond)
            }
            _ => return None,
        };
        // Condition 2.
        if !all_equal(&read_by_branch) {
            return None;
        }
        // Condition 4.
        if !let_elimination::is_applied_once(&pat.get_var().name, &expr.get_let_value()) {
            return None;
        }

        // The names the new binders avoid: every name free in the `let`, and the name it binds.
        let mut black_list = expr.free_vars();
        black_list.insert(pat.get_var().name.clone());
        let subject = NamedExpr::new(subject, &black_list);
        black_list.insert(subject.value.get_var().name.clone());
        let chosen = set_subject(&bound, subject.value.clone());
        Some(subject.wrap(expanded_let(expr, chosen, &ty, &black_list)))
    }

    /// The local names `branch` reads from outside it whose type holds a reference count, leaving
    /// out `bound_by_pattern`.
    fn counted_names_read(
        &self,
        branch: &Arc<ExprNode>,
        bound_by_pattern: &Set<FullName>,
        state: &VisitState,
    ) -> Set<FullName> {
        branch
            .free_vars()
            .into_iter()
            .filter(|name| name.is_local() && !bound_by_pattern.contains(name))
            .filter(|name| {
                let ty = state.scope.get_local(&name.name).unwrap().unwrap();
                !self.holds_no_count(&ty)
            })
            .collect()
    }

    /// Does `pat` bind every part of what it matches that holds a reference count, to a name that
    /// `val` reads?
    fn binds_every_counted_part(&self, pat: &Arc<PatternNode>, val: &Arc<ExprNode>) -> bool {
        let names_read = val.free_vars();
        let mut binds_all = true;
        pat.walk_nodes(&mut |node| {
            let counted = !self.holds_no_count(node.info.type_.as_ref().unwrap());
            match &node.pattern {
                Pattern::Var(var, _) => {
                    if counted && !names_read.contains(&var.name) {
                        binds_all = false;
                    }
                }
                // A struct pattern written with `_` after its fields drops the fields it does
                // not name.
                Pattern::Struct(_, _, leaves_out_fields) => {
                    if counted && *leaves_out_fields {
                        binds_all = false;
                    }
                }
                Pattern::Union(..) => {}
            }
        });
        binds_all
    }

    /// Is `branch` a chain of `let`s, each binding one name that the rest of the branch reads to a
    /// name, a constant, a lambda or an unboxed struct of those, ending in a name or a lambda? What
    /// such a branch evaluates is free to evaluate again, and reads no value it does not keep
    /// (condition 3).
    fn is_cheap(&self, branch: &Arc<ExprNode>) -> bool {
        match &*branch.expr {
            Expr::Var(_) | Expr::Lam(_, _) => true,
            Expr::Let(pat, bound, value) => {
                binds_a_name_read_by(pat, value)
                    && self.is_cheap_bound(bound)
                    && self.is_cheap(value)
            }
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
            Expr::Let(pat, inner_bound, value) => {
                binds_a_name_read_by(pat, value)
                    && self.is_cheap_bound(inner_bound)
                    && self.is_cheap_bound(value)
            }
            _ => false,
        }
    }

    /// Does a value of `ty` hold no reference count?
    fn holds_no_count(&self, ty: &Arc<TypeNode>) -> bool {
        ty.is_fully_unboxed(self.type_env)
    }
}

/// `let m = |x| {chosen}(x) in {value}`, for the `let m = .. in {value}` that `expr` is, where `m`
/// has type `ty` and `x` is a fresh name avoiding `black_list`.
fn expanded_let(
    expr: &Arc<ExprNode>,
    chosen: Arc<ExprNode>,
    ty: &Arc<TypeNode>,
    black_list: &Set<FullName>,
) -> Arc<ExprNode> {
    let param_ty = ty.get_lambda_srcs()[0].clone();
    let param = generate_new_names(black_list, 1)[0].clone();
    let arg = expr_var(param.clone(), None).set_type(param_ty.clone());
    let lam = expr_abs_typed(var_var(param), param_ty, expr_app_typed(chosen, vec![arg]));
    expr_let_typed(expr.get_let_pat(), lam, expr.get_let_value())
}

/// Does `pat` bind one name, which `value` reads? A binding nothing reads would release what it
/// reads where it stands, and keep it under the lambda once moved there.
fn binds_a_name_read_by(pat: &Arc<PatternNode>, value: &Arc<ExprNode>) -> bool {
    pat.is_var() && value.has_free_var(&pat.get_var().name)
}

/// Is every set of `sets` the same?
fn all_equal(sets: &[Set<FullName>]) -> bool {
    sets.windows(2).all(|pair| pair[0] == pair[1])
}
