//! A term simplifier on the lowered RC IR, run before `insert_rc`.
//!
//! It removes the union / struct plumbing of Fix's functional style by two classic rewrites, iterated
//! to a fixpoint:
//!
//! - **case-of-known-constructor**: a `match` on a value known to be a specific constructor collapses
//!   to that constructor's arm, and a `destructure` of a just-built struct binds each field directly
//!   to the value that built it. The construction and the match/destructure both vanish.
//! - **case-of-case**: a `match` whose scrutinee is itself a `match` (in tail position) moves each
//!   outer arm into the inner arm that builds its constructor, so the construction and the outer match
//!   cancel together. It fires all-or-nothing — only when every inner arm's result is a union the arm
//!   builds and a specific outer arm matches — and only when the result is smaller than what it
//!   replaces, since an outer arm two inner arms reach is placed in both.
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
//! The fixpoint terminates because every rewrite makes the body strictly smaller: the two
//! case-of-known-constructor rewrites drop the construction they cancel, and case-of-case is taken
//! only when its result is smaller than what it replaces. A body of `n` nodes therefore admits at most
//! `n` rewrites, which is the budget it is given. `simplify_to_fixpoint` asserts both halves of that:
//! the simplified body is never larger than the one it started from, and the budget is never spent.

use crate::ast::name::FullName;
use crate::fixstd::builtin::{InlineLLVMMakeStructBody, InlineLLVMMakeUnionBody};
use crate::misc::{grow_stack, Map};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcProgram, RcRhs, RcVar};
use crate::rc_ir::rename::{clone_fresh, substitute_expr};
use std::sync::Arc;

/// The marker for fresh names the case-of-case move mints, keeping them distinct from other passes'.
const MARKER: &str = "cc";

/// Rewriting state threaded through a body's fixpoint: the supply of fresh-name suffixes, unique
/// across the whole program, and the rewrites this body has left.
struct Ctx<'a> {
    /// The source of the number that makes each minted name unique across the whole program.
    fresh: &'a mut u64,
    /// The rewrites left for the body; the fixpoint stops when it reaches zero.
    budget: u64,
}

/// Simplify every function body and global initializer of `prog` to a fixpoint.
pub fn simplify(prog: &mut RcProgram) {
    let mut fresh = 0;
    for func in prog.funcs.values_mut() {
        func.body = simplify_to_fixpoint(&func.body, &mut fresh);
    }
    for g in &mut prog.globals {
        g.init = simplify_to_fixpoint(&g.init, &mut fresh);
    }
}

/// Apply the rewrites over a body until a pass makes no change. The body's budget is its node count,
/// the number of rewrites it admits when each makes it smaller; spending it stops the fixpoint, so a
/// rewrite that broke that accounting halts here rather than looping forever, and the assertions below
/// say which half of it broke.
fn simplify_to_fixpoint(node: &RcExprNode, fresh: &mut u64) -> RcExprNode {
    let size = node_count(node);
    let mut ctx = Ctx {
        fresh,
        budget: size,
    };
    let mut cur = node.clone();
    loop {
        let mut changed = false;
        cur = rewrite(&cur, &mut ctx, &mut changed);
        if !changed {
            let simplified = node_count(&cur);
            assert!(
                simplified <= size,
                "the simplifier grew a body from {} nodes to {}",
                size,
                simplified
            );
            assert!(
                ctx.budget > 0,
                "the simplifier took {} rewrites on a body of {} nodes, so one of them did not make it smaller",
                size - ctx.budget,
                size
            );
            return cur;
        }
    }
}

fn rewrite(node: &RcExprNode, ctx: &mut Ctx, changed: &mut bool) -> RcExprNode {
    // The continuation chain recurses deeply for a large function; grow the stack on demand.
    grow_stack(|| {
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
                    payload_state: arm.payload_state,
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
        RcExpr::Destructure(container, fields, state, k) => RcExpr::Destructure(
            container.clone(),
            fields.clone(),
            *state,
            rewrite(k, ctx, changed),
        ),
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
    let rewritten = case_of_known_union(node)
        .or_else(|| destructure_of_struct(node))
        .or_else(|| case_of_case(node, ctx.fresh));
    match rewritten {
        Some(rewritten) => {
            ctx.budget -= 1;
            *changed = true;
            rewritten
        }
        None => node.clone(),
    }
}

/// The number of expression nodes in `node`.
fn node_count(node: &RcExprNode) -> u64 {
    // A deep continuation chain recurses to its full depth here; grow the stack on demand.
    grow_stack(|| {
        let cont = match node.expr.as_ref() {
            RcExpr::Ret(_) => return 1,
            RcExpr::Let(_, RcRhs::Match(_, arms), k) => {
                return 1 + arms.iter().map(|a| node_count(&a.body)).sum::<u64>() + node_count(k);
            }
            RcExpr::Let(_, _, k)
            | RcExpr::Destructure(_, _, _, k)
            | RcExpr::Eval(_, k)
            | RcExpr::Retain(_, _, _, k)
            | RcExpr::Release(_, _, _, k) => k,
        };
        1 + node_count(cont)
    })
}

/// The variant number and payload operand of a union construction, and `None` where `rhs` builds no
/// union.
fn union_construction(rhs: &RcRhs) -> Option<(usize, &RcVar)> {
    let RcRhs::Llvm(gen, args) = rhs else {
        return None;
    };
    let make = gen.as_any().downcast_ref::<InlineLLVMMakeUnionBody>()?;
    // An operation's operands are its free variables, of which a union construction has the payload
    // alone.
    assert_eq!(
        args.len(),
        1,
        "a union construction takes its payload alone"
    );
    Some((make.variant_index(), &args[0]))
}

/// case-of-known-constructor on a union: `let x = union_tag(payload); let m = match x { .. }; k`,
/// where `x` is consumed only by the match, collapses to the `tag` arm — its payload bound to the
/// construction's operand, its result flowing into `m` — dropping both the construction and the match.
fn case_of_known_union(node: &RcExprNode) -> Option<RcExprNode> {
    let RcExpr::Let(x, rhs, k) = node.expr.as_ref() else {
        return None;
    };
    let (variant, payload) = union_construction(rhs)?;
    // The continuation must be exactly a match on `x`, and `x` must be used nowhere else.
    let RcExpr::Let(m, RcRhs::Match(scrut, arms), k2) = k.expr.as_ref() else {
        return None;
    };
    if scrut.name != x.name || count_value_uses(&x.name, k) != 1 {
        return None;
    }
    // Pick the arm for the known tag. A catch-all arm binds the whole union (not the payload), so it
    // would not remove the construction; skip when only a catch-all matches.
    let arm = arms.iter().find(|a| a.tag == Some(variant))?;
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
    let RcExpr::Destructure(container, fields, _, k2) = k.expr.as_ref() else {
        return None;
    };
    if container.name != x.name || count_value_uses(&x.name, k) != 1 {
        return None;
    }
    let mut subst: Map<FullName, FullName> = Map::default();
    for (idx, fv) in fields {
        // A struct construction takes one operand per field of its type, and a destructure of that
        // value names fields of the same type, so every field index is an operand of the
        // construction.
        assert!(
            *idx < args.len(),
            "a destructure names field {} of a struct built from {} operands",
            idx,
            args.len()
        );
        subst.insert(fv.name.clone(), args[*idx].name.clone());
    }
    Some(substitute_expr(k2, &subst))
}

/// case-of-case (tail form): `let s = match iScrut { iArms }; let m = match s { oArms }; ret m`, where
/// `s` is consumed only by the outer match, moves each outer arm into the inner arm whose result it
/// matches. An inner arm ends in a union it builds and immediately returns, so the outer arm for that
/// constructor is the one that arm reaches; putting that arm in place of the construction cancels the
/// two against each other, with the outer arm's payload binder becoming the construction's operand.
/// The inner match then produces what the outer match did, so it binds the outer match's variable.
///
/// It fires all-or-nothing — every inner arm must end in such a construction and a specific outer arm
/// must match it — and only when the result is smaller than what it replaces. The result is built and
/// then measured, so what bounds the term is the term itself rather than a rule about when it grows.
/// It grows where two inner arms build one constructor, which puts that outer arm in both: a nest of
/// matches doing so at every level would double the term at every level. Where the inner arms build
/// pairwise distinct constructors, each outer arm moves to one inner arm and the result always
/// shrinks, by the constructions and the outer match that go away.
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
    let mut new_arms = Vec::with_capacity(inner_arms.len());
    for arm in inner_arms {
        let body = replace_tail_construction(&arm.body, &mut |variant, operand| {
            let outer = outer_arms.iter().find(|a| a.tag == Some(variant))?;
            // Fresh binders per arm, so an outer arm two inner arms reach does not put one name in
            // two places.
            let moved = clone_fresh(&outer.body, MARKER, counter);
            Some(substitute_expr(
                &moved,
                &single(&outer.payload.name, &operand.name),
            ))
        })?;
        new_arms.push(MatchArm {
            payload_state: arm.payload_state,
            tag: arm.tag,
            payload: arm.payload.clone(),
            body,
        });
    }
    let rewritten = node_of(
        RcExpr::Let(
            m.clone(),
            RcRhs::Match(inner_scrut.clone(), new_arms),
            node_of(RcExpr::Ret(m.clone()), &node.source),
        ),
        &node.source,
    );
    (node_count(&rewritten) < node_count(node)).then_some(rewritten)
}

/// Whether `node` is exactly `ret name`.
fn is_ret_of(node: &RcExprNode, name: &FullName) -> bool {
    matches!(node.expr.as_ref(), RcExpr::Ret(v) if v.name == *name)
}

/// Replace the union construction at `node`'s tail — `let r = make_union(operand); ret r` — with
/// `f(variant, operand)`, and give `None` when the tail is not such a construction or `f` declines.
/// One walk therefore both decides whether the arm cancels and performs the cancellation.
///
/// Requiring the construction to abut the `ret` makes `r` single-use — bound and immediately returned
/// — so whatever consumed the arm's result consumed that union linearly.
fn replace_tail_construction(
    node: &RcExprNode,
    f: &mut dyn FnMut(usize, &RcVar) -> Option<RcExprNode>,
) -> Option<RcExprNode> {
    // An arm body is a continuation chain that recurses to its full depth here; grow the stack.
    grow_stack(|| {
        let expr = match node.expr.as_ref() {
            RcExpr::Ret(_) => return None,
            RcExpr::Let(r, rhs, k) => {
                if is_ret_of(k, &r.name) {
                    let (variant, operand) = union_construction(rhs)?;
                    return f(variant, operand);
                }
                RcExpr::Let(r.clone(), rhs.clone(), replace_tail_construction(k, f)?)
            }
            RcExpr::Destructure(c, fields, state, k) => RcExpr::Destructure(
                c.clone(),
                fields.clone(),
                *state,
                replace_tail_construction(k, f)?,
            ),
            RcExpr::Eval(v, k) => RcExpr::Eval(v.clone(), replace_tail_construction(k, f)?),
            RcExpr::Retain(v, p, st, k) => {
                RcExpr::Retain(v.clone(), p.clone(), *st, replace_tail_construction(k, f)?)
            }
            RcExpr::Release(v, p, st, k) => {
                RcExpr::Release(v.clone(), p.clone(), *st, replace_tail_construction(k, f)?)
            }
        };
        Some(node_of(expr, &node.source))
    })
}

/// Replace the terminal `ret r` of `node` with `f(r)`, threading through the continuation chain. A
/// `Match` is a right-hand side, so its arms are not the expression's tail — the tail is the final
/// `Ret` reached through the `Let`/`Destructure`/`Eval`/`Retain`/`Release` continuations.
fn replace_tail(node: &RcExprNode, f: &mut dyn FnMut(&RcVar) -> RcExprNode) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Ret(r) => return f(r),
        RcExpr::Let(x, rhs, k) => RcExpr::Let(x.clone(), rhs.clone(), replace_tail(k, f)),
        RcExpr::Destructure(c, fields, state, k) => {
            RcExpr::Destructure(c.clone(), fields.clone(), *state, replace_tail(k, f))
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
    grow_stack(|| {
        let hit = |v: &RcVar| (v.name == *name) as usize;
        match node.expr.as_ref() {
            RcExpr::Ret(v) => hit(v),
            RcExpr::Let(_, rhs, k) => rhs_value_uses(name, rhs) + count_value_uses(name, k),
            RcExpr::Destructure(c, _, _state, k) => hit(c) + count_value_uses(name, k),
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

/// A one-entry substitution map.
fn single(from: &FullName, to: &FullName) -> Map<FullName, FullName> {
    let mut m: Map<FullName, FullName> = Map::default();
    m.insert(from.clone(), to.clone());
    m
}

fn node_of(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Arc::new(expr),
        source: source.clone(),
    }
}
