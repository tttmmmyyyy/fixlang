//! A term simplifier on the lowered RC IR, run before `insert_rc`.
//!
//! It removes the union / struct plumbing of Fix's functional style by two classic rewrites, iterated
//! to a fixpoint:
//!
//! - **case-of-known-constructor**: a `match` on a value known to be a specific constructor collapses
//!   to that constructor's arm, and a `destructure` of a just-built struct binds each field directly
//!   to the value that built it. The construction and the match/destructure both vanish.
//! - **case-of-case**: a `match` whose scrutinee is itself a `match` (in tail position) floats into
//!   the inner arms, bringing each inner arm's freshly built constructor next to the outer match so
//!   case-of-known-constructor can cancel it.
//!
//! Composed, they cancel the `Option`/`LoopState`/tuple union a loop builds and immediately matches
//! each iteration, exposing the scalar loop state underneath — which is what lets the back end form a
//! scalar induction variable, eliminate the bounds check, and vectorize (see
//! `dev-docs/2026-07-18-bounds-check-elim/`).
//!
//! Running before `insert_rc` keeps the rewrites free of reference-count bookkeeping: they only move
//! and drop plumbing, and `insert_rc` computes the reference counting afterward. Each rewrite fires
//! only when the value it removes is consumed exactly once, so no boxed payload gains a second
//! reference (which would force a copy). Every substitution renames variables only — no computation is
//! moved — so no boxed value's lifetime is extended.

use crate::ast::name::FullName;
use crate::fixstd::builtin::{InlineLLVMMakeStructBody, InlineLLVMMakeUnionBody};
use crate::misc::Map;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcProgram, RcRhs, RcVar};
use crate::rc_ir::rename::{clone_fresh, substitute_expr};

/// The marker for fresh names the case-of-case clone mints, keeping them distinct from other passes'.
const MARKER: &str = "cc";

/// Simplify every function body and global initializer of `prog` to a fixpoint.
pub fn simplify(prog: &mut RcProgram) {
    let mut counter: u64 = 0;
    for func in prog.funcs.values_mut() {
        func.body = simplify_to_fixpoint(&func.body, &mut counter);
    }
    for g in &mut prog.globals {
        g.init = simplify_to_fixpoint(&g.init, &mut counter);
    }
}

/// Apply the rewrites over a body until a pass makes no change.
fn simplify_to_fixpoint(node: &RcExprNode, counter: &mut u64) -> RcExprNode {
    let mut cur = node.clone();
    loop {
        let mut changed = false;
        cur = rewrite(&cur, counter, &mut changed);
        if !changed {
            return cur;
        }
    }
}

fn rewrite(node: &RcExprNode, counter: &mut u64, changed: &mut bool) -> RcExprNode {
    // The continuation chain recurses deeply for a large function; grow the stack on demand.
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        let node = rewrite_children(node, counter, changed);
        try_local(&node, counter, changed)
    })
}

/// Rebuild a node with `rewrite` applied to its sub-expressions (match arms and the continuation).
fn rewrite_children(node: &RcExprNode, counter: &mut u64, changed: &mut bool) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Ret(v) => RcExpr::Ret(v.clone()),
        RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    variant: arm.variant,
                    payload: arm.payload.clone(),
                    body: rewrite(&arm.body, counter, changed),
                })
                .collect();
            RcExpr::Let(
                x.clone(),
                RcRhs::Match(scrut.clone(), arms),
                rewrite(k, counter, changed),
            )
        }
        RcExpr::Let(x, rhs, k) => {
            RcExpr::Let(x.clone(), rhs.clone(), rewrite(k, counter, changed))
        }
        RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
            container.clone(),
            fields.clone(),
            rewrite(k, counter, changed),
        ),
        RcExpr::Eval(v, k) => RcExpr::Eval(v.clone(), rewrite(k, counter, changed)),
        RcExpr::Retain(v, path, state, k) => {
            RcExpr::Retain(v.clone(), path.clone(), *state, rewrite(k, counter, changed))
        }
        RcExpr::Release(v, path, state, k) => {
            RcExpr::Release(v.clone(), path.clone(), *state, rewrite(k, counter, changed))
        }
    };
    node_of(expr, &node.source)
}

/// Try the local rewrites at `node`. Returns the rewritten node and sets `changed` if one fired.
fn try_local(node: &RcExprNode, counter: &mut u64, changed: &mut bool) -> RcExprNode {
    if let Some(rewritten) = case_of_known_union(node) {
        *changed = true;
        return rewritten;
    }
    if let Some(rewritten) = destructure_of_struct(node) {
        *changed = true;
        return rewritten;
    }
    if let Some(rewritten) = case_of_case(node, counter) {
        *changed = true;
        return rewritten;
    }
    node.clone()
}

/// case-of-known-constructor on a union: `let x = union_tag(payload); let m = match x { .. }; k`,
/// where `x` is consumed only by the match, collapses to the `tag` arm — its payload bound to the
/// construction's operand, its result flowing into `m` — dropping both the construction and the match.
fn case_of_known_union(node: &RcExprNode) -> Option<RcExprNode> {
    let RcExpr::Let(x, RcRhs::Llvm(gen, args), k) = node.expr.as_ref() else {
        return None;
    };
    let make = gen.as_any().downcast_ref::<InlineLLVMMakeUnionBody>()?;
    if args.len() != 1 {
        return None;
    }
    let payload = &args[0];
    // The continuation must be exactly a match on `x`, and `x` must be used nowhere else.
    let RcExpr::Let(m, RcRhs::Match(scrut, arms), k2) = k.expr.as_ref() else {
        return None;
    };
    if scrut.name != x.name || count_value_uses(&x.name, k) != 1 {
        return None;
    }
    // Pick the arm for the known tag. A catch-all arm binds the whole union (not the payload), so it
    // would not remove the construction; skip when only a catch-all matches.
    let arm = arms.iter().find(|a| a.variant == Some(make.variant_index()))?;
    let body = substitute_expr(&arm.body, &single(&arm.payload.name, &payload.name));
    Some(replace_tail(&body, &mut |result| {
        substitute_expr(k2, &single(&m.name, &result.name))
    }))
}

/// case-of-known-constructor on a struct: `let x = make_struct(a, b, ..); destructure x { .i -> fi };
/// k`, where `x` is consumed only by the destructure, binds each field variable directly to the
/// operand that built that field, dropping both the construction and the destructure.
fn destructure_of_struct(node: &RcExprNode) -> Option<RcExprNode> {
    let RcExpr::Let(x, RcRhs::Llvm(gen, args), k) = node.expr.as_ref() else {
        return None;
    };
    gen.as_any().downcast_ref::<InlineLLVMMakeStructBody>()?;
    let RcExpr::Destructure(container, fields, k2) = k.expr.as_ref() else {
        return None;
    };
    if container.name != x.name || count_value_uses(&x.name, k) != 1 {
        return None;
    }
    let mut subst: Map<FullName, FullName> = Map::default();
    for (idx, fv) in fields {
        let operand = args.get(*idx)?;
        subst.insert(fv.name.clone(), operand.name.clone());
    }
    Some(substitute_expr(k2, &subst))
}

/// case-of-case (tail form): `let s = match iScrut { iArms }; let m = match s { oArms }; ret m`, where
/// `s` is consumed only by the outer match, floats the outer match into each inner arm's tail. The
/// outer match is cloned with fresh binders per arm (so names stay unique) and its scrutinee replaced
/// by the value that inner arm produces — bringing that value (a freshly built constructor) next to
/// the outer match for case-of-known-constructor to cancel on the next pass.
fn case_of_case(node: &RcExprNode, counter: &mut u64) -> Option<RcExprNode> {
    let RcExpr::Let(s, RcRhs::Match(inner_scrut, inner_arms), k) = node.expr.as_ref() else {
        return None;
    };
    // The continuation must be exactly a tail match on `s`: `let m = match s {..}; ret m`.
    let RcExpr::Let(m, RcRhs::Match(outer_scrut, _), k2) = k.expr.as_ref() else {
        return None;
    };
    if outer_scrut.name != s.name || !is_ret_of(k2, &m.name) || count_value_uses(&s.name, k) != 1 {
        return None;
    }
    // Float the outer match `k` into each inner arm's tail, cloned with fresh binders per arm and its
    // scrutinee `s` replaced by the value that arm produces. A `for` loop keeps `counter`'s borrow
    // sequential across arms.
    let mut new_arms = Vec::with_capacity(inner_arms.len());
    for arm in inner_arms {
        let body = replace_tail(&arm.body, &mut |produced| {
            let fresh = clone_fresh(k, MARKER, counter);
            substitute_expr(&fresh, &single(&s.name, &produced.name))
        });
        new_arms.push(MatchArm {
            variant: arm.variant,
            payload: arm.payload.clone(),
            body,
        });
    }
    // Bind the floated match to a fresh result and return it, preserving the original value (`m`).
    let result = fresh_var(m, counter);
    let matched = node_of(
        RcExpr::Let(
            result.clone(),
            RcRhs::Match(inner_scrut.clone(), new_arms),
            node_of(RcExpr::Ret(result.clone()), &node.source),
        ),
        &node.source,
    );
    Some(matched)
}

/// Whether `node` is exactly `ret name`.
fn is_ret_of(node: &RcExprNode, name: &FullName) -> bool {
    matches!(node.expr.as_ref(), RcExpr::Ret(v) if v.name == *name)
}

/// Replace the terminal `ret r` of `node` with `f(r)`, threading through the continuation chain. A
/// `Match` is a right-hand side, so its arms are not the expression's tail — the tail is the final
/// `Ret` reached through the `Let`/`Destructure`/`Eval`/`Retain`/`Release` continuations.
fn replace_tail(node: &RcExprNode, f: &mut dyn FnMut(&RcVar) -> RcExprNode) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Ret(r) => return f(r),
        RcExpr::Let(x, rhs, k) => RcExpr::Let(x.clone(), rhs.clone(), replace_tail(k, f)),
        RcExpr::Destructure(c, fields, k) => {
            RcExpr::Destructure(c.clone(), fields.clone(), replace_tail(k, f))
        }
        RcExpr::Eval(v, k) => RcExpr::Eval(v.clone(), replace_tail(k, f)),
        RcExpr::Retain(v, p, st, k) => RcExpr::Retain(v.clone(), p.clone(), *st, replace_tail(k, f)),
        RcExpr::Release(v, p, st, k) => RcExpr::Release(v.clone(), p.clone(), *st, replace_tail(k, f)),
    };
    node_of(expr, &node.source)
}

/// The number of times `name` occurs as a value in `node`: a move, a call callee or argument, an
/// inline-LLVM operand, a closure capture, a match scrutinee, a destructured container, an `eval`, or
/// the returned variable. Binders do not count. `Retain`/`Release` name a variable only for reference
/// counting, so they are transparent (and do not occur before `insert_rc` anyway).
fn count_value_uses(name: &FullName, node: &RcExprNode) -> usize {
    let hit = |v: &RcVar| (v.name == *name) as usize;
    match node.expr.as_ref() {
        RcExpr::Ret(v) => hit(v),
        RcExpr::Let(_, rhs, k) => rhs_value_uses(name, rhs) + count_value_uses(name, k),
        RcExpr::Destructure(c, _, k) => hit(c) + count_value_uses(name, k),
        RcExpr::Eval(v, k) => hit(v) + count_value_uses(name, k),
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => count_value_uses(name, k),
    }
}

fn rhs_value_uses(name: &FullName, rhs: &RcRhs) -> usize {
    let hit = |v: &RcVar| (v.name == *name) as usize;
    match rhs {
        RcRhs::Var(v) => hit(v),
        RcRhs::App(callee, args) => hit(callee) + args.iter().map(hit).sum::<usize>(),
        RcRhs::Closure(_, caps) => caps.iter().map(hit).sum(),
        RcRhs::Llvm(_, args) => args.iter().map(hit).sum(),
        RcRhs::Match(scrut, arms) => {
            hit(scrut)
                + arms
                    .iter()
                    .map(|arm| count_value_uses(name, &arm.body))
                    .sum::<usize>()
        }
    }
}

/// A fresh variable like `base` but with a globally-unique name.
fn fresh_var(base: &RcVar, counter: &mut u64) -> RcVar {
    *counter += 1;
    let mut v = base.clone();
    v.name.name = format!("{}#{}{}", base.name.name, MARKER, counter);
    v
}

/// A one-entry substitution map.
fn single(from: &FullName, to: &FullName) -> Map<FullName, FullName> {
    let mut m: Map<FullName, FullName> = Map::default();
    m.insert(from.clone(), to.clone());
    m
}

fn node_of(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Box::new(expr),
        source: source.clone(),
    }
}
