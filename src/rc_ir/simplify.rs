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
//!   case-of-known-constructor can cancel it. It fires all-or-nothing — only when every inner arm's
//!   result is a union the arm builds and a specific outer arm matches, so the floated match cancels
//!   in every arm and the outer match is redistributed rather than duplicated.
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
//!
//! The fixpoint terminates: case-of-known-constructor and destructure-of-struct each remove a node,
//! and the all-or-nothing case-of-case guard keeps every float a net cancellation, so the term does
//! not grow without bound. A per-body rewrite budget (`rewrite_budget`) backs this up as a hard limit.

use crate::ast::name::FullName;
use crate::fixstd::builtin::{InlineLLVMMakeStructBody, InlineLLVMMakeUnionBody};
use crate::misc::Map;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcProgram, RcRhs, RcVar};
use crate::rc_ir::rename::{clone_fresh, substitute_expr};

/// The marker for fresh names the case-of-case clone mints, keeping them distinct from other passes'.
const MARKER: &str = "cc";

/// Rewriting state threaded through a body's fixpoint: a supply of fresh-name suffixes (unique across
/// the whole program) and a per-body rewrite budget. The budget is a halting backstop — the
/// constructor guard on case-of-case already keeps every rewrite productive, so the budget only
/// guarantees termination against an unforeseen non-terminating interaction, never limits real work.
struct Ctx {
    fresh: u64,
    budget: u64,
}

/// Simplify every function body and global initializer of `prog` to a fixpoint.
pub fn simplify(prog: &mut RcProgram) {
    let mut ctx = Ctx {
        fresh: 0,
        budget: 0,
    };
    for func in prog.funcs.values_mut() {
        func.body = simplify_to_fixpoint(&func.body, &mut ctx);
    }
    for g in &mut prog.globals {
        g.init = simplify_to_fixpoint(&g.init, &mut ctx);
    }
}

/// Apply the rewrites over a body until a pass makes no change (or the budget is spent).
fn simplify_to_fixpoint(node: &RcExprNode, ctx: &mut Ctx) -> RcExprNode {
    ctx.budget = rewrite_budget(node);
    let mut cur = node.clone();
    loop {
        let mut changed = false;
        cur = rewrite(&cur, ctx, &mut changed);
        if !changed {
            return cur;
        }
    }
}

fn rewrite(node: &RcExprNode, ctx: &mut Ctx, changed: &mut bool) -> RcExprNode {
    // The continuation chain recurses deeply for a large function; grow the stack on demand.
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        let node = rewrite_children(node, ctx, changed);
        try_local(&node, ctx, changed)
    })
}

/// Rebuild a node with `rewrite` applied to its sub-expressions (match arms and the continuation).
fn rewrite_children(node: &RcExprNode, ctx: &mut Ctx, changed: &mut bool) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Ret(v) => RcExpr::Ret(v.clone()),
        RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    tag: arm.tag,
                    payload: arm.payload.clone(),
                    body: rewrite(&arm.body, ctx, changed),
                })
                .collect();
            RcExpr::Let(
                x.clone(),
                RcRhs::Match(scrut.clone(), arms),
                rewrite(k, ctx, changed),
            )
        }
        RcExpr::Let(x, rhs, k) => RcExpr::Let(x.clone(), rhs.clone(), rewrite(k, ctx, changed)),
        RcExpr::Destructure(container, fields, k) => {
            RcExpr::Destructure(container.clone(), fields.clone(), rewrite(k, ctx, changed))
        }
        RcExpr::Eval(v, k) => RcExpr::Eval(v.clone(), rewrite(k, ctx, changed)),
        RcExpr::Retain(v, path, state, k) => {
            RcExpr::Retain(v.clone(), path.clone(), *state, rewrite(k, ctx, changed))
        }
        RcExpr::Release(v, path, state, k) => {
            RcExpr::Release(v.clone(), path.clone(), *state, rewrite(k, ctx, changed))
        }
    };
    node_of(expr, &node.source)
}

/// Try the local rewrites at `node`. Returns the rewritten node and sets `changed` if one fired. Once
/// the body's budget is spent no rewrite fires, so the fixpoint reaches a no-change pass and stops.
fn try_local(node: &RcExprNode, ctx: &mut Ctx, changed: &mut bool) -> RcExprNode {
    if ctx.budget == 0 {
        return node.clone();
    }
    if let Some(rewritten) = case_of_known_union(node) {
        ctx.budget -= 1;
        *changed = true;
        return rewritten;
    }
    if let Some(rewritten) = destructure_of_struct(node) {
        ctx.budget -= 1;
        *changed = true;
        return rewritten;
    }
    if let Some(rewritten) = case_of_case(node, &mut ctx.fresh) {
        ctx.budget -= 1;
        *changed = true;
        return rewritten;
    }
    node.clone()
}

/// The rewrite budget for a body: a halting backstop sized far above any legitimate need, so it never
/// limits real simplification but still bounds the total number of rewrites.
fn rewrite_budget(node: &RcExprNode) -> u64 {
    const BASE: u64 = 1024;
    const PER_NODE: u64 = 64;
    BASE + PER_NODE * node_count(node)
}

/// The number of expression nodes in `node`.
fn node_count(node: &RcExprNode) -> u64 {
    // A deep continuation chain recurses to its full depth here; grow the stack on demand.
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        let cont = match node.expr.as_ref() {
            RcExpr::Ret(_) => return 1,
            RcExpr::Let(_, RcRhs::Match(_, arms), k) => {
                return 1 + arms.iter().map(|a| node_count(&a.body)).sum::<u64>() + node_count(k);
            }
            RcExpr::Let(_, _, k)
            | RcExpr::Destructure(_, _, k)
            | RcExpr::Eval(_, k)
            | RcExpr::Retain(_, _, _, k)
            | RcExpr::Release(_, _, _, k) => k,
        };
        1 + node_count(cont)
    })
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
    let arm = arms.iter().find(|a| a.tag == Some(make.variant_index()))?;
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
///
/// It fires all-or-nothing: only when *every* inner arm's result is a union built in that arm and
/// matched by a specific outer arm, so case-of-known-constructor cancels the floated match in every
/// arm and the outer match is redistributed rather than left duplicated. That keeps each rewrite a net
/// simplification and bounds the total, so the fixpoint cannot diverge.
fn case_of_case(node: &RcExprNode, counter: &mut u64) -> Option<RcExprNode> {
    let RcExpr::Let(s, RcRhs::Match(inner_scrut, inner_arms), k) = node.expr.as_ref() else {
        return None;
    };
    // The continuation must be exactly a tail match on `s`: `let m = match s {..}; ret m`.
    let RcExpr::Let(m, RcRhs::Match(outer_scrut, outer_arms), k2) = k.expr.as_ref() else {
        return None;
    };
    if outer_scrut.name != s.name || !is_ret_of(k2, &m.name) || count_value_uses(&s.name, k) != 1 {
        return None;
    }
    if !inner_arms
        .iter()
        .all(|arm| arm_result_cancels_outer(&arm.body, outer_arms))
    {
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
            tag: arm.tag,
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

/// Whether floating `outer_arms` (a union match) into an inner arm whose body is `body` will cancel:
/// the arm must end in a union it builds and immediately returns, and `outer_arms` must have a
/// specific arm for that union's tag. Together these are exactly what case-of-known-constructor needs
/// to collapse the floated match, so the guard predicts the cancellation faithfully.
fn arm_result_cancels_outer(body: &RcExprNode, outer_arms: &[MatchArm]) -> bool {
    match arm_tail_union_tag(body) {
        Some(tag) => outer_arms.iter().any(|a| a.tag == Some(tag)),
        None => false,
    }
}

/// If `body`'s tail is `let r = make_union(payload); ret r` — a union built and immediately returned —
/// the constructor's tag. Requiring the construction to abut the `ret` is what makes the union abut the
/// floated match, which is the adjacency case-of-known-constructor needs; it also makes `r` single-use
/// (bound then returned), so the floated match consumes it linearly.
fn arm_tail_union_tag(body: &RcExprNode) -> Option<usize> {
    match body.expr.as_ref() {
        RcExpr::Let(r, rhs, k) => {
            if !is_ret_of(k, &r.name) {
                return arm_tail_union_tag(k);
            }
            let RcRhs::Llvm(gen, args) = rhs else {
                return None;
            };
            let make = gen.as_any().downcast_ref::<InlineLLVMMakeUnionBody>()?;
            (args.len() == 1).then(|| make.variant_index())
        }
        RcExpr::Destructure(_, _, k)
        | RcExpr::Eval(_, k)
        | RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k) => arm_tail_union_tag(k),
        RcExpr::Ret(_) => None,
    }
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
        RcExpr::Retain(v, p, st, k) => {
            RcExpr::Retain(v.clone(), p.clone(), *st, replace_tail(k, f))
        }
        RcExpr::Release(v, p, st, k) => {
            RcExpr::Release(v.clone(), p.clone(), *st, replace_tail(k, f))
        }
    };
    node_of(expr, &node.source)
}

/// The number of times `name` occurs as a value in `node`: a move, a call callee or argument, an
/// inline-LLVM operand, a closure capture, a match scrutinee, a destructured container, an `eval`, or
/// the returned variable. Binders do not count. `Retain`/`Release` name a variable only for reference
/// counting, so they are transparent (and do not occur before `insert_rc` anyway).
fn count_value_uses(name: &FullName, node: &RcExprNode) -> usize {
    // A deep continuation chain recurses to its full depth here; grow the stack on demand.
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        let hit = |v: &RcVar| (v.name == *name) as usize;
        match node.expr.as_ref() {
            RcExpr::Ret(v) => hit(v),
            RcExpr::Let(_, rhs, k) => rhs_value_uses(name, rhs) + count_value_uses(name, k),
            RcExpr::Destructure(c, _, k) => hit(c) + count_value_uses(name, k),
            RcExpr::Eval(v, k) => hit(v) + count_value_uses(name, k),
            RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => count_value_uses(name, k),
        }
    })
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
