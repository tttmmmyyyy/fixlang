//! Fresh renaming of RC IR local variables for the passes that clone functions (`borrow`,
//! `unique_check_elim`). Because RC IR names are globally unique, a clone must give every binder a fresh
//! name so the clone's names do not collide with the original's. The single entry point,
//! `fresh_rename_function`, clones a function's parameters, capture, and body this way; `pass_tag`
//! distinguishes each cloning pass's fresh names from the others'.

use crate::ast::name::FullName;
use crate::misc::Map;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcRhs, RcVar};
use std::sync::Arc;

/// Clone a function's parameters, capture, and body, giving every bound variable (parameters,
/// capture, `let` bindings, destructure fields, match-arm payloads) a fresh globally-unique name and
/// rewriting every occurrence. Returns the renamed pieces together with the binder renaming, which
/// callers use to remap side tables and to route recursive references. `pass_tag` distinguishes this
/// cloning pass's fresh names from the others'.
pub(crate) fn fresh_rename_function(
    params: &[RcVar],
    cap: &Option<RcVar>,
    body: &RcExprNode,
    pass_tag: &str,
    counter: &mut u64,
) -> (
    Vec<RcVar>,
    Option<RcVar>,
    RcExprNode,
    Map<FullName, FullName>,
) {
    let mut renaming: Map<FullName, FullName> = Map::default();
    for p in params.iter().chain(cap.iter()) {
        assign_fresh_name(&p.name, pass_tag, &mut renaming, counter);
    }
    assign_fresh_names_to_binders(body, pass_tag, &mut renaming, counter);
    let new_params = params.iter().map(|p| rename_var(p, &renaming)).collect();
    let new_cap = cap.as_ref().map(|c| rename_var(c, &renaming));
    let new_body = rename_expr(body, &renaming);
    (new_params, new_cap, new_body, renaming)
}

/// Assign `name` a fresh globally-unique name, suffixed with `pass_tag` and a counter.
fn assign_fresh_name(
    name: &FullName,
    pass_tag: &str,
    renaming: &mut Map<FullName, FullName>,
    counter: &mut u64,
) {
    *counter += 1;
    let mut fresh = name.clone();
    fresh.name = format!("{}#{}{}", fresh.name, pass_tag, counter);
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
    pass_tag: &str,
    renaming: &mut Map<FullName, FullName>,
    counter: &mut u64,
) {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        assign_fresh_names_to_binders_inner(node, pass_tag, renaming, counter)
    })
}

fn assign_fresh_names_to_binders_inner(
    node: &RcExprNode,
    pass_tag: &str,
    renaming: &mut Map<FullName, FullName>,
    counter: &mut u64,
) {
    match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => {
            assign_fresh_name(&x.name, pass_tag, renaming, counter);
            // Listed explicitly (not a catch-all) so a new `RcRhs` that binds a name fails to compile
            // here instead of leaving that binder pointing at the original function's variable.
            match rhs {
                RcRhs::Match(_, arms) => {
                    for arm in arms {
                        assign_fresh_name(&arm.payload.name, pass_tag, renaming, counter);
                        assign_fresh_names_to_binders(&arm.body, pass_tag, renaming, counter);
                    }
                }
                RcRhs::Var(_) | RcRhs::App(..) | RcRhs::Closure(..) | RcRhs::Llvm(..) => {}
            }
            assign_fresh_names_to_binders(k, pass_tag, renaming, counter);
        }
        RcExpr::Destructure(_, fields, k) => {
            for (_, fv) in fields {
                assign_fresh_name(&fv.name, pass_tag, renaming, counter);
            }
            assign_fresh_names_to_binders(k, pass_tag, renaming, counter);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            assign_fresh_names_to_binders(k, pass_tag, renaming, counter)
        }
        RcExpr::Ret(_) => {}
    }
}

/// A variable with its name rewritten through `renaming` (unchanged if it names a global rather than a
/// local binder).
fn rename_var(var: &RcVar, renaming: &Map<FullName, FullName>) -> RcVar {
    let mut v = var.clone();
    if let Some(n) = renaming.get(&var.name) {
        v.name = n.clone();
    }
    v
}

/// A deep clone of an expression with every variable occurrence rewritten through `renaming`. The
/// operand names embedded in an `Llvm` generator are rewritten too, since they name the same locals.
fn rename_expr(node: &RcExprNode, renaming: &Map<FullName, FullName>) -> RcExprNode {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || rename_expr_inner(node, renaming))
}

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
        RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
            rename_var(container, renaming),
            fields
                .iter()
                .map(|(i, v)| (*i, rename_var(v, renaming)))
                .collect(),
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

/// A right-hand side with every variable occurrence (including `Llvm` operand names) rewritten
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
        RcRhs::Llvm(llvm_gen, args) => {
            let mut llvm_gen = llvm_gen.clone();
            for slot in llvm_gen.free_vars_mut() {
                if let Some(n) = renaming.get(slot) {
                    *slot = n.clone();
                }
            }
            RcRhs::Llvm(
                llvm_gen,
                args.iter().map(|a| rename_var(a, renaming)).collect(),
            )
        }
        RcRhs::Match(scrut, arms) => RcRhs::Match(
            rename_var(scrut, renaming),
            arms.iter()
                .map(|arm| MatchArm {
                    tag: arm.tag,
                    payload: rename_var(&arm.payload, renaming),
                    body: rename_expr(&arm.body, renaming),
                })
                .collect(),
        ),
    }
}

/// Substitute variable occurrences through `subst` in a deep clone of `node`, leaving binders and
/// structure otherwise intact — a partial-map application of the same rewrite `rename_expr` performs.
/// The simplifier uses it to replace a match-arm payload with the operands of the constructor it
/// matched (case-of-known-constructor) and a match result with an inner arm's value (case-of-case);
/// those substituends are never re-bound within the substituted expression, so a partial map suffices.
pub(crate) fn substitute_expr(node: &RcExprNode, subst: &Map<FullName, FullName>) -> RcExprNode {
    rename_expr(node, subst)
}

/// A deep clone of an arbitrary expression with every bound variable given a fresh globally-unique
/// name (like `fresh_rename_function`, but for a sub-expression rather than a whole function). Free
/// variables — those bound outside `node` — are left unchanged. The simplifier uses it when
/// case-of-case duplicates a match into several arms, so each copy's binders stay unique.
pub(crate) fn clone_fresh(node: &RcExprNode, marker: &str, counter: &mut u64) -> RcExprNode {
    let mut rename: Map<FullName, FullName> = Map::default();
    assign_fresh_names_to_binders(node, marker, &mut rename, counter);
    rename_expr(node, &rename)
}
