//! Fresh renaming of RC IR local variables for the passes that copy code. RC IR names are globally
//! unique, so a copy gives every binder a fresh name, which it makes by appending a suffix to the
//! binder's name. The caller chooses the suffix, and keeps the names unique by giving each copy of
//! one binder a suffix of its own. The suffix depends only on the function the copy belongs to and
//! on what the copy is, so editing one function leaves the names in every other function, and the
//! code generated for them, as they were.

use crate::ast::name::FullName;
use crate::misc::{grow_stack, Map};
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcRhs, RcVar};
use std::sync::Arc;

/// The suffix a copy's names take: `#`, the tag of the pass that makes the copy, and
/// `distinguisher`, which tells the copy apart from the pass's other copies of the same code. No
/// lowered name ends in this shape, which is what keeps the names the suffix makes fresh.
pub(crate) fn copy_suffix(tag: &str, distinguisher: &str) -> String {
    format!("#{}{}", tag, distinguisher)
}

/// Clone a function's parameters, capture, and body, giving every bound variable (parameters,
/// capture, `let` bindings, destructure fields, match-arm payloads) its name followed by `suffix`, and
/// rewriting every occurrence. Returns the renamed pieces together with the binder renaming, which
/// callers use to remap side tables and to route recursive references.
pub(crate) fn fresh_rename_function(
    params: &[RcVar],
    cap: &Option<RcVar>,
    body: &RcExprNode,
    suffix: &str,
) -> (
    Vec<RcVar>,
    Option<RcVar>,
    RcExprNode,
    Map<FullName, FullName>,
) {
    let mut renaming: Map<FullName, FullName> = Map::default();
    for p in params.iter().chain(cap.iter()) {
        assign_fresh_name(&p.name, suffix, &mut renaming);
    }
    assign_fresh_names_to_binders(body, suffix, &mut renaming);
    let new_params = params.iter().map(|p| rename_var(p, &renaming)).collect();
    let new_cap = cap.as_ref().map(|c| rename_var(c, &renaming));
    let new_body = rename_expr(body, &renaming);
    (new_params, new_cap, new_body, renaming)
}

/// Assign `name` the fresh name `name` followed by `suffix`.
fn assign_fresh_name(name: &FullName, suffix: &str, renaming: &mut Map<FullName, FullName>) {
    let mut fresh = name.clone();
    fresh.name.push_str(suffix);
    // Names are unique within a function, so a second fresh name for one binder would merge two
    // variables of the clone into one.
    let previous = renaming.insert(name.clone(), fresh);
    assert!(
        previous.is_none(),
        "`{}` is bound twice in one function",
        name.to_string()
    );
}

/// Record a fresh name for every variable bound in a function body.
fn assign_fresh_names_to_binders(
    node: &RcExprNode,
    suffix: &str,
    renaming: &mut Map<FullName, FullName>,
) {
    grow_stack(|| assign_fresh_names_to_binders_inner(node, suffix, renaming))
}

/// Record the fresh name of every binder of one node, then descend into its continuation and arms.
fn assign_fresh_names_to_binders_inner(
    node: &RcExprNode,
    suffix: &str,
    renaming: &mut Map<FullName, FullName>,
) {
    match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => {
            assign_fresh_name(&x.name, suffix, renaming);
            // Every `RcRhs` variant is listed, so a new one that binds a name fails to compile here
            // instead of leaving that binder pointing at the original function's variable.
            match rhs {
                RcRhs::Match(_, arms) => {
                    for arm in arms {
                        assign_fresh_name(&arm.payload.name, suffix, renaming);
                        assign_fresh_names_to_binders(&arm.body, suffix, renaming);
                    }
                }
                RcRhs::Var(_) | RcRhs::App(..) | RcRhs::Closure(..) | RcRhs::Builtin(..) => {}
            }
            assign_fresh_names_to_binders(k, suffix, renaming);
        }
        RcExpr::Destructure(_, fields, _state, k) => {
            for (_, fv) in fields {
                assign_fresh_name(&fv.name, suffix, renaming);
            }
            assign_fresh_names_to_binders(k, suffix, renaming);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            assign_fresh_names_to_binders(k, suffix, renaming)
        }
        RcExpr::Ret(_) => {}
    }
}

/// A variable with its name rewritten through `renaming`. A name `renaming` leaves out, such as a
/// global's, stays as it is.
fn rename_var(var: &RcVar, renaming: &Map<FullName, FullName>) -> RcVar {
    let mut v = var.clone();
    if let Some(n) = renaming.get(&var.name) {
        v.name = n.clone();
    }
    v
}

/// A deep clone of an expression with every variable occurrence rewritten through `renaming`. The
/// operand names embedded in a `Builtin` operation are rewritten too, since they name the same
/// locals.
fn rename_expr(node: &RcExprNode, renaming: &Map<FullName, FullName>) -> RcExprNode {
    grow_stack(|| rename_expr_inner(node, renaming))
}

/// Rebuild one node with its variable occurrences rewritten, over its rewritten continuation.
fn rename_expr_inner(node: &RcExprNode, renaming: &Map<FullName, FullName>) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => RcExpr::Let(
            rename_var(x, renaming),
            rename_rhs(rhs, renaming),
            rename_expr(k, renaming),
        ),
        RcExpr::Retain(v, path, state, k) => RcExpr::Retain(
            rename_var(v, renaming),
            path.clone(),
            *state,
            rename_expr(k, renaming),
        ),
        RcExpr::Release(v, path, state, k) => RcExpr::Release(
            rename_var(v, renaming),
            path.clone(),
            *state,
            rename_expr(k, renaming),
        ),
        RcExpr::Destructure(container, fields, state, k) => RcExpr::Destructure(
            rename_var(container, renaming),
            fields
                .iter()
                .map(|(i, v)| (*i, rename_var(v, renaming)))
                .collect(),
            *state,
            rename_expr(k, renaming),
        ),
        RcExpr::Eval(v, k) => RcExpr::Eval(rename_var(v, renaming), rename_expr(k, renaming)),
        RcExpr::Ret(v) => RcExpr::Ret(rename_var(v, renaming)),
    };
    RcExprNode {
        expr: Arc::new(expr),
        source: node.source.clone(),
    }
}

/// A right-hand side with every variable occurrence (including `Builtin` operand names) rewritten
/// through `renaming`.
fn rename_rhs(rhs: &RcRhs, renaming: &Map<FullName, FullName>) -> RcRhs {
    match rhs {
        RcRhs::Var(v) => RcRhs::Var(rename_var(v, renaming)),
        RcRhs::App(callee, args) => RcRhs::App(
            rename_var(callee, renaming),
            args.iter().map(|a| rename_var(a, renaming)).collect(),
        ),
        RcRhs::Closure(fref, caps) => RcRhs::Closure(
            fref.clone(),
            caps.iter().map(|c| rename_var(c, renaming)).collect(),
        ),
        RcRhs::Builtin(op, args) => {
            let mut op = op.clone();
            for slot in op.free_vars_mut() {
                if let Some(n) = renaming.get(slot) {
                    *slot = n.clone();
                }
            }
            RcRhs::Builtin(op, args.iter().map(|a| rename_var(a, renaming)).collect())
        }
        RcRhs::Match(scrut, arms) => RcRhs::Match(
            rename_var(scrut, renaming),
            arms.iter()
                .map(|arm| MatchArm {
                    payload_state: arm.payload_state,
                    tag: arm.tag,
                    payload: rename_var(&arm.payload, renaming),
                    body: rename_expr(&arm.body, renaming),
                })
                .collect(),
        ),
    }
}

/// Substitute variable occurrences through `subst` in a deep clone of `node`, leaving binders and
/// structure otherwise intact. A name `subst` maps must not be re-bound within `node`, so a map
/// covering only some of the variables suffices.
pub(crate) fn substitute_expr(node: &RcExprNode, subst: &Map<FullName, FullName>) -> RcExprNode {
    rename_expr(node, subst)
}

/// A deep clone of an arbitrary expression with every bound variable given its name followed by
/// `suffix`. Free variables — those bound outside `node` — are left unchanged.
pub(crate) fn clone_fresh(node: &RcExprNode, suffix: &str) -> RcExprNode {
    let mut rename: Map<FullName, FullName> = Map::default();
    assign_fresh_names_to_binders(node, suffix, &mut rename);
    rename_expr(node, &rename)
}
