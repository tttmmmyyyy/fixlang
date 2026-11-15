//! A term simplifier on the lowered RC IR, run before `insert_rc`.
//!
//! It removes the union / struct plumbing of Fix's functional style by two classic rewrites, iterated
//! to a fixpoint:
//!
//! - **case-of-known-constructor**: a `match` on a value known to be a specific constructor collapses
//!   to that constructor's arm, and a `destructure` of a just-built struct binds each field directly
//!   to the value that built it. The construction and the match/destructure both vanish.
//! - **case-of-case**: a `match` whose scrutinee is itself a `match` moves each outer arm into the
//!   inner arm that builds its constructor, so the construction and the outer match cancel together.
//!   The walk over an inner arm follows into a `match` that arm ends in, so it reaches the
//!   constructions the arms of that one build as well. It fires all-or-nothing — only when a specific
//!   outer arm matches every construction the walk reaches — and only when the result is smaller than
//!   what it replaces, since an outer arm two inner arms reach is placed in both. The walk that finds
//!   those constructions measures that size first, so the rewrite is decided before an outer arm is
//!   copied anywhere.
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
//! Every rewrite makes the body strictly smaller: the two case-of-known-constructor rewrites drop the
//! construction they cancel, and case-of-case is taken only when the size it measures beforehand is
//! smaller than what it replaces. `simplify_to_fixpoint` measures each pass against that, which is
//! what makes the fixpoint terminate — a body of `n` nodes admits at most `n` passes that change it.

use crate::ast::name::FullName;
use crate::ast::types::TypeNode;
use crate::configuration::Configuration;
use crate::fixstd::builtin::{MakeStructOp, MakeUnionOp};
use crate::misc::{grow_stack, Map};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    count_value_uses, map_children, map_continuation, MatchArm, RcExpr, RcExprNode, RcProgram,
    RcRhs, RcVar,
};
use crate::rc_ir::rename::{clone_fresh, copy_suffix, substitute_expr};
use std::sync::Arc;

/// The tag on the fresh names the case-of-case move mints, keeping them distinct from other passes'.
const PASS_TAG: &str = "case";

/// Simplify every function body and global initializer of `prog` to a fixpoint.
pub fn simplify(prog: &mut RcProgram, config: &Configuration) {
    for func in prog.funcs.values_mut() {
        func.body = simplify_to_fixpoint(&func.body, config);
    }
    for g in &mut prog.globals {
        g.init = simplify_to_fixpoint(&g.init, config);
    }
}

/// Apply the rewrites over a body until a pass makes no change. A pass that fired a rewrite leaves
/// fewer nodes than it found; a pass that leaves as many or more has broken that, and the body it
/// produced is dropped in favour of the one before it, which stops the fixpoint. Development mode
/// stops loudly instead, at the pass that broke it.
fn simplify_to_fixpoint(node: &RcExprNode, config: &Configuration) -> RcExprNode {
    // Numbers the copies the case-of-case move makes in this body. The binders of the body are bound
    // nowhere else in the program, so numbering the copies per body keeps the names unique.
    let mut counter = 0;
    let mut cur = node.clone();
    let mut size = node_count(&cur);
    loop {
        let mut changed = false;
        let next = rewrite(&cur, &mut counter, &mut changed);
        if !changed {
            return cur;
        }
        let next_size = node_count(&next);
        if next_size >= size {
            if config.develop_mode {
                panic!(
                    "a simplifier pass grew a body of {} nodes to {}, so a rewrite it fired did not make it smaller",
                    size, next_size
                );
            }
            return cur;
        }
        cur = next;
        size = next_size;
    }
}

/// One rewriting pass over `node`: its sub-expressions first, then the local rewrites at `node`
/// itself, so a rewrite at `node` sees its sub-expressions already simplified. `changed` is set when
/// any rewrite fires, which is what tells the fixpoint another pass is due.
fn rewrite(node: &RcExprNode, counter: &mut u64, changed: &mut bool) -> RcExprNode {
    // The continuation chain recurses deeply for a large function; grow the stack on demand.
    grow_stack(|| {
        let node = map_children(node, |child| rewrite(child, counter, changed));
        try_local(&node, counter, changed)
    })
}

/// Try the local rewrites at `node`, in order. Returns the rewritten node and sets `changed` if one
/// fired, which is what tells the fixpoint another pass is due.
fn try_local(node: &RcExprNode, counter: &mut u64, changed: &mut bool) -> RcExprNode {
    let rewritten = case_of_known_union(node)
        .or_else(|| destructure_of_struct(node))
        .or_else(|| case_of_case(node, counter));
    match rewritten {
        Some(rewritten) => {
            *changed = true;
            rewritten
        }
        None => node.clone(),
    }
}

/// The number of expression nodes in `node`.
pub(crate) fn node_count(node: &RcExprNode) -> u64 {
    // A deep continuation chain recurses to its full depth here; grow the stack on demand.
    grow_stack(|| {
        let cont = match node.expr.as_ref() {
            RcExpr::Ret(_) => return 1,
            RcExpr::Let(_, RcRhs::Match(_, arms), k) => {
                return 1 + arms_size(arms) + node_count(k);
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

/// The number of expression nodes in the bodies of `arms`.
fn arms_size(arms: &[MatchArm]) -> u64 {
    arms.iter().map(|arm| node_count(&arm.body)).sum()
}

/// The variant number and payload operand of the union construction `rhs` builds.
fn union_construction(rhs: &RcRhs) -> Option<(usize, &RcVar)> {
    let RcRhs::Builtin(op, args) = rhs else {
        return None;
    };
    let construction = op.as_any().downcast_ref::<MakeUnionOp>()?;
    // An operation's operands are its free variables, of which a union construction has the payload
    // alone.
    assert_eq!(
        args.len(),
        1,
        "a union construction takes its payload alone"
    );
    Some((construction.variant_index(), &args[0]))
}

/// case-of-known-constructor on a union: `let x = union_tag(payload); let m = match x { .. }; k`,
/// where `x` is consumed only by the match, collapses to the `tag` arm — its payload bound to the
/// construction's operand, its result flowing into `m` — dropping both the construction and the match.
fn case_of_known_union(node: &RcExprNode) -> Option<RcExprNode> {
    let RcExpr::Let(x, rhs, k) = node.expr.as_ref() else {
        return None;
    };
    let (variant, operand) = union_construction(rhs)?;
    let (m, arms, k2) = match_on(x, k)?;
    // Pick the arm for the known tag. A catch-all arm binds the whole union, so the construction
    // stays to build it; skip when only a catch-all matches.
    let arm = arms.iter().find(|arm| arm.tag == Some(variant))?;
    if !consumed_only_by_its_match(x, k) {
        return None;
    }
    let body = substitute_expr(&arm.body, &single_subst(&arm.payload.name, &operand.name));
    Some(replace_tail(&body, &mut |arm_result| {
        substitute_expr(k2, &single_subst(&m.name, &arm_result.name))
    }))
}

/// The match on `x` that `k` starts with, `let m = match x { arms }; k2`, as `m`, `arms` and `k2`.
fn match_on<'a>(
    x: &RcVar,
    k: &'a RcExprNode,
) -> Option<(&'a RcVar, &'a [MatchArm], &'a RcExprNode)> {
    let RcExpr::Let(m, RcRhs::Match(scrut, arms), k2) = k.expr.as_ref() else {
        return None;
    };
    if scrut.name != x.name {
        return None;
    }
    Some((m, arms, k2))
}

/// Whether the match `k` starts with is the only consumer of `x` in `k`, so that a rewrite dropping
/// what built `x` drops that match with it and leaves nothing else reading `x`.
///
/// This walks the whole of `k`, which is the rest of the function, so a rewrite asks it last: asked
/// first at every match on a match, it would make a pass over a function quadratic in its length.
fn consumed_only_by_its_match(x: &RcVar, k: &RcExprNode) -> bool {
    count_value_uses(&x.name, k) == 1
}

/// case-of-known-constructor on a struct: `let x = make_struct(a, b, ..); destructure x { .i -> fi };
/// k`, where `x` is consumed only by the destructure, binds each field variable directly to the
/// operand that built that field, dropping both the construction and the destructure.
fn destructure_of_struct(node: &RcExprNode) -> Option<RcExprNode> {
    let RcExpr::Let(x, RcRhs::Builtin(op, args), k) = node.expr.as_ref() else {
        return None;
    };
    op.as_any().downcast_ref::<MakeStructOp>()?;
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

/// case-of-case: `let s = match iScrut { iArms }; let m = match s { oArms }; k2`, where `s` is
/// consumed only by the outer match, moves each outer arm into the inner arm whose result it
/// matches. An inner arm ends in a union it builds and immediately returns — or in a `match` whose own
/// arms do, which `replace_tail_union` follows into — so the outer arm for that constructor is the one
/// that arm reaches; putting that arm in place of the construction cancels the two against each other,
/// with the outer arm's payload binder becoming the construction's operand. The inner match then
/// produces what the outer match did, so it binds the outer match's variable, and `k2` follows it
/// once, as it followed the outer match.
///
/// It fires all-or-nothing — every tail the walk reaches must build a union a specific outer arm
/// matches — and only when the result is smaller than what it replaces, a size
/// `size_after_replacing_tails` gives before an outer arm is copied anywhere. It grows where two
/// inner arms build one constructor, which puts that outer arm in both: a nest of matches doing so
/// at every level would double the term at every level. Where the inner arms build pairwise
/// distinct constructors, each outer arm moves to one inner arm and the result always shrinks, by
/// the constructions and the outer match that go away.
fn case_of_case(node: &RcExprNode, counter: &mut u64) -> Option<RcExprNode> {
    let RcExpr::Let(s, RcRhs::Match(inner_scrut, inner_arms), k) = node.expr.as_ref() else {
        return None;
    };
    let (m, outer_arms, k2) = match_on(s, k)?;
    // The arm answering each variant, with the size of its body, so that a tail costs one lookup.
    let mut outer_arm_of_variant: Map<usize, (&MatchArm, u64)> = Map::default();
    for arm in outer_arms {
        if let Some(tag) = arm.tag {
            outer_arm_of_variant
                .entry(tag)
                .or_insert_with(|| (arm, node_count(&arm.body)));
        }
    }
    // Measure before building. Copying an outer arm into every tail is what this rewrite spends, so
    // the copies below are made only for a rewrite that is kept. One binding of a match and `k2`
    // stand on both sides, so the rewrite is measured on the arms alone: the new inner arms against
    // the inner arms, the outer match and its arms they replace.
    let replaced_size = 1 + arms_size(inner_arms) + arms_size(outer_arms);
    let mut new_arms_size = 0;
    for arm in inner_arms {
        new_arms_size += size_after_replacing_tails(&arm.body, &mut |variant| {
            outer_arm_of_variant.get(&variant).map(|(_, size)| *size)
        })?;
    }
    // Equality declines too: `simplify_to_fixpoint` counts a pass that leaves as many nodes as it
    // found as broken, and stops the fixpoint there.
    if new_arms_size >= replaced_size {
        return None;
    }
    if !consumed_only_by_its_match(s, k) {
        return None;
    }
    let mut new_arms = Vec::with_capacity(inner_arms.len());
    for arm in inner_arms {
        let body = replace_tail_union(&arm.body, &m.ty, &mut |variant, operand| {
            let (outer, _) = outer_arm_of_variant.get(&variant).unwrap_or_else(|| {
                panic!(
                    "no outer arm answers variant {}, which a tail builds",
                    variant
                )
            });
            // Fresh binders per arm, so an outer arm two inner arms reach does not put one name in
            // two places.
            *counter += 1;
            let moved = clone_fresh(&outer.body, &copy_suffix(PASS_TAG, &counter.to_string()));
            substitute_expr(&moved, &single_subst(&outer.payload.name, &operand.name))
        });
        new_arms.push(arm.with_body(body));
    }
    // Renaming a binder and substituting a variable both leave the term's shape alone, so placing
    // the outer arms leaves exactly the nodes `new_arms_size` counted.
    assert_eq!(
        arms_size(&new_arms),
        new_arms_size,
        "the rewrite left arms of a size other than the one measured before building them"
    );
    Some(node_of(
        RcExpr::Let(
            m.clone(),
            RcRhs::Match(inner_scrut.clone(), new_arms),
            k2.clone(),
        ),
        &node.source,
    ))
}

/// The number of nodes `node` holds once the union construction at each of its tails is replaced by
/// what `f` sizes — the size `replace_tail_union` leaves. `None` where a tail builds no union, or
/// where `f` has nothing for the variant one builds — the all-or-nothing condition the move fires
/// under, decided with nothing built.
fn size_after_replacing_tails(
    node: &RcExprNode,
    f: &mut dyn FnMut(usize) -> Option<u64>,
) -> Option<u64> {
    // An arm body is a continuation chain that recurses to its full depth here; grow the stack.
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Ret(_) => None,
        RcExpr::Let(r, rhs, k) if is_ret_of(k, &r.name) => {
            if let RcRhs::Match(_, arms) = rhs {
                // The binding of the match and the `ret` after it stand where they stood.
                let mut size = 2;
                for arm in arms {
                    size += size_after_replacing_tails(&arm.body, f)?;
                }
                return Some(size);
            }
            f(union_construction(rhs)?.0)
        }
        RcExpr::Let(_, RcRhs::Match(_, arms), _) => {
            // A match anywhere but the tail stands where it stood, arms and all.
            Some(1 + arms_size(arms) + size_after_replacing_tails(continuation_of(node), f)?)
        }
        _ => Some(1 + size_after_replacing_tails(continuation_of(node), f)?),
    })
}

/// Whether `node` is exactly `ret name`.
fn is_ret_of(node: &RcExprNode, name: &FullName) -> bool {
    matches!(node.expr.as_ref(), RcExpr::Ret(v) if v.name == *name)
}

/// Replace the union construction at `node`'s tail — `let r = make_union(operand); ret r` — with
/// `f(variant, operand)`. Where a `match` stands in that tail position, the walk continues into its
/// arms and replaces the tail of each. `size_after_replacing_tails` walks the same shapes and has
/// already answered for every tail this reaches, so a tail that builds nothing to replace stops the
/// compiler where the two walks part ways.
///
/// Requiring the construction to abut the `ret` makes `r` single-use — bound and immediately returned
/// — so whatever consumed the arm's result consumed that union linearly.
///
/// # Arguments
/// * `result_ty` - the type of what `f` produces, which a `match` walked into now yields.
/// * `f` - what replaces the tail, given the variant number and the payload operand of the union
///   construction found there.
fn replace_tail_union(
    node: &RcExprNode,
    result_ty: &Arc<TypeNode>,
    f: &mut dyn FnMut(usize, &RcVar) -> RcExprNode,
) -> RcExprNode {
    // An arm body is a continuation chain that recurses to its full depth here; grow the stack.
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Ret(v) => unreachable!(
            "the tail returns {}, which it did not build",
            v.name.to_string()
        ),
        RcExpr::Let(r, rhs, k) if is_ret_of(k, &r.name) => {
            if let RcRhs::Match(scrut, arms) = rhs {
                return replace_tail_union_of_match(node, r, scrut, arms, result_ty, f);
            }
            let (variant, operand) = union_construction(rhs).expect("the tail builds no union");
            f(variant, operand)
        }
        _ => map_continuation(node, |k| replace_tail_union(k, result_ty, f)),
    })
}

/// Replace the tail union of every arm of a `match` standing at `node`'s tail — `let r = match scrut
/// { arms }; ret r` — with `f(variant, operand)`. The rebuilt binding of `r` takes `result_ty`, which
/// is what those arms now produce.
///
/// A body deciding between two constructors this way — read one condition, then the next — is the
/// ordinary shape of a two-branch loop body, so this is where the union of such a loop is built.
fn replace_tail_union_of_match(
    node: &RcExprNode,
    r: &RcVar,
    scrut: &RcVar,
    arms: &[MatchArm],
    result_ty: &Arc<TypeNode>,
    f: &mut dyn FnMut(usize, &RcVar) -> RcExprNode,
) -> RcExprNode {
    let new_arms = arms
        .iter()
        .map(|arm| arm.with_body(replace_tail_union(&arm.body, result_ty, f)))
        .collect::<Vec<_>>();
    let new_r = RcVar {
        ty: result_ty.clone(),
        ..r.clone()
    };
    tail_match(new_r, scrut, new_arms, &node.source)
}

/// `let x = match scrut { arms }; ret x` — the shape a `match` takes where it stands in tail
/// position.
fn tail_match(x: RcVar, scrut: &RcVar, arms: Vec<MatchArm>, source: &Option<Span>) -> RcExprNode {
    let ret = node_of(RcExpr::Ret(x.clone()), source);
    node_of(
        RcExpr::Let(x, RcRhs::Match(scrut.clone(), arms), ret),
        source,
    )
}

/// Replace the terminal `ret r` of `node` with `f(r)`, threading through the continuation chain. A
/// `Match` is a right-hand side, so the tail is the final `Ret` reached through the
/// `Let`/`Destructure`/`Eval`/`Retain`/`Release` continuations, and the arms of a `Match` are left as
/// they are.
fn replace_tail(node: &RcExprNode, f: &mut dyn FnMut(&RcVar) -> RcExprNode) -> RcExprNode {
    // An arm body is a continuation chain that recurses to its full depth here; grow the stack.
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Ret(r) => f(r),
        _ => map_continuation(node, |k| replace_tail(k, f)),
    })
}

/// What `node` evaluates after its own step. A `ret` ends the chain and has none, so a caller answers
/// for that case before asking.
fn continuation_of(node: &RcExprNode) -> &RcExprNode {
    match node.expr.as_ref() {
        RcExpr::Ret(v) => unreachable!("`ret {}` ends the chain", v.name.to_string()),
        RcExpr::Let(_, _, k)
        | RcExpr::Destructure(_, _, _, k)
        | RcExpr::Eval(_, k)
        | RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k) => k,
    }
}

/// A one-entry substitution map.
fn single_subst(from: &FullName, to: &FullName) -> Map<FullName, FullName> {
    let mut subst: Map<FullName, FullName> = Map::default();
    subst.insert(from.clone(), to.clone());
    subst
}

/// A node holding `expr` and reporting `source` as the place it comes from. A rewritten node carries
/// the span of the node it replaces, so the simplified body still points into the source program.
fn node_of(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Arc::new(expr),
        source: source.clone(),
    }
}
