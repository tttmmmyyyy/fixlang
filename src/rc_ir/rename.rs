//! Fresh renaming of RC IR local variables, shared by the passes that clone functions. Because RC IR
//! names are globally unique, a clone must give every binder a fresh name so the clone's names do not
//! collide with the original's; `marker` distinguishes each cloning pass's fresh names from the
//! others'.

use crate::ast::name::FullName;
use crate::misc::Map;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcRhs, RcVar};

/// Assign `name` a fresh globally-unique name (unless it already has one), suffixed with `marker`
/// and a counter.
pub(crate) fn fresh_rename(
    name: &FullName,
    marker: &str,
    rename: &mut Map<FullName, FullName>,
    counter: &mut u64,
) {
    if rename.contains_key(name) {
        return;
    }
    *counter += 1;
    let mut fresh = name.clone();
    fresh.name = format!("{}#{}{}", fresh.name, marker, counter);
    rename.insert(name.clone(), fresh);
}

/// Record a fresh name for every variable bound in a function body.
pub(crate) fn collect_binders(
    node: &RcExprNode,
    marker: &str,
    rename: &mut Map<FullName, FullName>,
    counter: &mut u64,
) {
    match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => {
            fresh_rename(&x.name, marker, rename, counter);
            if let RcRhs::Match(_, arms) = rhs {
                for arm in arms {
                    fresh_rename(&arm.payload.name, marker, rename, counter);
                    collect_binders(&arm.body, marker, rename, counter);
                }
            }
            collect_binders(k, marker, rename, counter);
        }
        RcExpr::Destructure(_, fields, k) => {
            for (_, fv) in fields {
                fresh_rename(&fv.name, marker, rename, counter);
            }
            collect_binders(k, marker, rename, counter);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            collect_binders(k, marker, rename, counter)
        }
        RcExpr::Ret(_) => {}
    }
}

/// A variable with its name rewritten through `rename` (unchanged if it names a global rather than a
/// local binder).
pub(crate) fn rename_var(var: &RcVar, rename: &Map<FullName, FullName>) -> RcVar {
    let mut v = var.clone();
    if let Some(n) = rename.get(&var.name) {
        v.name = n.clone();
    }
    v
}

/// A deep clone of an expression with every variable occurrence rewritten through `rename`. The
/// operand names embedded in an `Llvm` generator are rewritten too, since they name the same locals.
pub(crate) fn rename_expr(node: &RcExprNode, rename: &Map<FullName, FullName>) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => RcExpr::Let(
            rename_var(x, rename),
            rename_rhs(rhs, rename),
            rename_expr(k, rename),
        ),
        RcExpr::Retain(v, path, state, k) => RcExpr::Retain(
            rename_var(v, rename),
            path.clone(),
            *state,
            rename_expr(k, rename),
        ),
        RcExpr::Release(v, path, state, k) => RcExpr::Release(
            rename_var(v, rename),
            path.clone(),
            *state,
            rename_expr(k, rename),
        ),
        RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
            rename_var(container, rename),
            fields
                .iter()
                .map(|(i, v)| (*i, rename_var(v, rename)))
                .collect(),
            rename_expr(k, rename),
        ),
        RcExpr::Eval(v, k) => RcExpr::Eval(rename_var(v, rename), rename_expr(k, rename)),
        RcExpr::Ret(v) => RcExpr::Ret(rename_var(v, rename)),
    };
    RcExprNode {
        expr: Box::new(expr),
        source: node.source.clone(),
    }
}

/// A right-hand side with every variable occurrence (including `Llvm` operand names) rewritten
/// through `rename`.
pub(crate) fn rename_rhs(rhs: &RcRhs, rename: &Map<FullName, FullName>) -> RcRhs {
    match rhs {
        RcRhs::Var(v) => RcRhs::Var(rename_var(v, rename)),
        RcRhs::App(callee, args) => RcRhs::App(
            rename_var(callee, rename),
            args.iter().map(|a| rename_var(a, rename)).collect(),
        ),
        RcRhs::Closure(fref, caps) => RcRhs::Closure(
            fref.clone(),
            caps.iter().map(|c| rename_var(c, rename)).collect(),
        ),
        RcRhs::Llvm(gen, args) => {
            let mut gen = gen.clone();
            for slot in gen.free_vars_mut() {
                if let Some(n) = rename.get(slot) {
                    *slot = n.clone();
                }
            }
            RcRhs::Llvm(gen, args.iter().map(|a| rename_var(a, rename)).collect())
        }
        RcRhs::Match(scrut, arms) => RcRhs::Match(
            rename_var(scrut, rename),
            arms.iter()
                .map(|arm| MatchArm {
                    variant: arm.variant,
                    payload: rename_var(&arm.payload, rename),
                    body: rename_expr(&arm.body, rename),
                })
                .collect(),
        ),
    }
}
