//! Reference-counting insertion (Phase B of the AST->RC IR lowering).
//!
//! Phase A produces the RC IR skeleton with no explicit `Retain`/`Release` nodes (the only baked-in
//! reference counting is the boxed capture getter's retain and the `Destructure` node's extraction).
//! This pass adds the explicit nodes by a backward last-use analysis over each function, at
//! whole-value granularity (per-leaf paths and further precision come from later passes).
//!
//! Ownership of an operand is `Own` (the op consumes it: moves it into the result, releases it
//! internally, or force-unique-returns it) or `Borrow` (the op reads it without consuming it). Three
//! rules place the nodes: (a) before a non-last use of an `Own` operand, insert a `Retain`; (b) after
//! the last use of a `Borrow` operand, insert a `Release`; (c) a variable that becomes dead without
//! being consumed — an unused binding, or one a sibling match arm uses but this arm does not — is
//! released at the earliest point it is dead. Only the read-getters borrow (see
//! `LLVMGen::borrows_operand`); everything else owns. `RcState` is `Unknown` (always sound).
//! Reference counting is skipped for fully-unboxed values (they have no boxed leaf, so
//! `Retain`/`Release` would generate no code).

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    MatchArm, Ownership, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState, RcVar,
};
use std::sync::Arc;

/// Insert explicit `Retain`/`Release` nodes into every function and global initializer of `prog`.
pub fn insert_rc(prog: &mut RcProgram, type_env: &TypeEnv) {
    let funcs = std::mem::take(&mut prog.funcs);
    let mut new_funcs = Map::default();
    for (fref, func) in funcs {
        let inserter = RcInserter::new(type_env, &func);
        new_funcs.insert(fref, inserter.insert_into_func(func));
    }
    prog.funcs = new_funcs;

    let globals = std::mem::take(&mut prog.globals);
    prog.globals = globals
        .into_iter()
        .map(|mut glob| {
            let inserter = RcInserter::new_for_global_init(type_env, &glob.init);
            let (body, live) = inserter.insert_into_expr(glob.init, &Set::default());
            // A global initializer takes no parameter and no capture, so it has no way to receive a
            // value: a free local is a reference to a binding that lowering lost.
            assert!(
                live.is_empty(),
                "the initializer of `{}` reads local variables it does not bind: {:?}",
                glob.symbol.to_string(),
                live
            );
            glob.init = body;
            glob
        })
        .collect();
}

/// The reference-counting context for one function (or global initializer): the type environment
/// and a table from every local variable name to its `RcVar` (used to recover a variable's type and
/// span when placing a dead-branch release). Names are globally unique, so the table is unambiguous.
struct RcInserter<'a> {
    type_env: &'a TypeEnv,
    vars: Map<FullName, RcVar>,
}

impl<'a> RcInserter<'a> {
    fn new(type_env: &'a TypeEnv, func: &RcFunc) -> Self {
        let mut vars = Map::default();
        for p in &func.params {
            vars.insert(p.name.clone(), p.clone());
        }
        if let Some(cap) = &func.capture {
            vars.insert(cap.name.clone(), cap.clone());
        }
        collect_vars(&func.body, &mut vars);
        RcInserter { type_env, vars }
    }

    fn new_for_global_init(type_env: &'a TypeEnv, expr: &RcExprNode) -> Self {
        let mut vars = Map::default();
        collect_vars(expr, &mut vars);
        RcInserter { type_env, vars }
    }

    /// Rewrite a function body, then release any parameter or capture that the body never uses.
    fn insert_into_func(&self, mut func: RcFunc) -> RcFunc {
        let (body, live) = self.insert_into_expr(func.body, &Set::default());

        // The body is built under the parameters and the capture alone, so nothing else can be live
        // at its entry: a name that is would be a use with no binding to reach.
        for name in &live {
            assert!(
                func.params
                    .iter()
                    .chain(func.capture.iter())
                    .any(|input| &input.name == name),
                "`{}` is live at the entry of `{}`, which binds it neither as a parameter nor as its capture",
                name.to_string(),
                func.name.name.to_string()
            );
        }

        let mut unused = vec![];
        for p in &func.params {
            if self.needs_rc(p) && !live.contains(&p.name) {
                unused.push(p.clone());
            }
        }
        if let Some(cap) = &func.capture {
            if self.needs_rc(cap) && !live.contains(&cap.name) {
                unused.push(cap.clone());
            }
        }
        func.body = build_releases(unused, body);
        func
    }

    /// Process one expression, given the set of local variables live *after* it. Returns the
    /// rewritten expression and the set of local variables live *before* it (at its entry).
    fn insert_into_expr(
        &self,
        node: RcExprNode,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        // The continuation chain recurses deeply for a large function (as lowering and code
        // generation do); grow the stack on demand so it does not overflow.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.insert_into_expr_inner(node, live_after)
        })
    }

    fn insert_into_expr_inner(
        &self,
        node: RcExprNode,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let source = node.source.clone();
        match *node.expr {
            RcExpr::Ret(x) => {
                let mut live = live_after.clone();
                insert_if_local(&mut live, &x.name);
                // Returning `x` consumes it. If `x` is also live after this expression — e.g. a match
                // arm returns a variable that is used again after the match — it is consumed twice, so
                // retain it here to provide the extra reference.
                let ret = RcExprNode {
                    expr: Box::new(RcExpr::Ret(x.clone())),
                    source,
                };
                let node = self.retain_if_live(&x, live_after, ret);
                (node, live)
            }
            RcExpr::Let(x, RcRhs::Match(scrut, arms), cont) => {
                self.insert_into_match(x, scrut, arms, cont, source, live_after)
            }
            RcExpr::Let(x, rhs, cont) => {
                self.insert_into_operation_let(x, rhs, cont, source, live_after)
            }
            RcExpr::Destructure(container, fields, cont) => {
                self.insert_into_destructure(container, fields, cont, source, live_after)
            }
            RcExpr::Eval(x, cont) => self.insert_into_eval(x, cont, source, live_after),
            RcExpr::Retain(..) | RcExpr::Release(..) => {
                panic!("RC insertion runs on a skeleton that has no Retain/Release nodes yet")
            }
        }
    }

    /// An `eval x; cont`. `Eval` observes `x` without consuming it (a borrow), so — like a borrowed
    /// operand at its last use — `x` is released after the eval iff it is a local that needs reference
    /// counting and is dead in the continuation.
    fn insert_into_eval(
        &self,
        x: RcVar,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let (cont, live_cont) = self.insert_into_expr(cont, live_after);
        let cont = if x.name.is_local() && !live_cont.contains(&x.name) && self.needs_rc(&x) {
            build_releases(vec![x.clone()], cont)
        } else {
            cont
        };
        let node = RcExprNode {
            expr: Box::new(RcExpr::Eval(x.clone(), cont)),
            source,
        };
        let mut live_before = live_cont;
        insert_if_local(&mut live_before, &x.name);
        (node, live_before)
    }

    /// A `let x = rhs; cont` whose `rhs` is not a `Match` (the `Match` case is `insert_into_match`).
    fn insert_into_operation_let(
        &self,
        x: RcVar,
        rhs: RcRhs,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        assert!(
            !matches!(rhs, RcRhs::Match(..)),
            "insert_into_operation_let received a Match rhs; a Match is handled by insert_into_match"
        );
        let (cont, live_cont) = self.insert_into_expr(cont, live_after);

        // Operand reference counting. Walk operands in reverse evaluation order so that, for a
        // variable used more than once, the last (right-most) use moves and the earlier uses retain.
        let operands = rhs_operands(&rhs, self.type_env);
        let mut live_after_operand = live_cont.clone();
        let mut retains_before = vec![]; // Own operand used later -> retain before the statement.
        let mut releases_after = vec![]; // Borrow operand at its last use -> release after it.
        for (v, ownership) in operands.iter().rev() {
            if v.name.is_local() {
                let used_later = live_after_operand.contains(&v.name);
                if *ownership == Ownership::Borrow {
                    if !used_later && self.needs_rc(v) {
                        releases_after.push(v.clone());
                    }
                } else if used_later && self.needs_rc(v) {
                    retains_before.push(v.clone());
                }
                live_after_operand.insert(v.name.clone());
            }
        }

        // After-statement releases: the borrowed operands at their last use, then `x` if it is a
        // dead binding (unused in the continuation).
        let mut after = releases_after; // release order among borrows is immaterial
        if !live_cont.contains(&x.name) && self.needs_rc(&x) {
            after.push(x.clone());
        }
        let cont = build_releases(after, cont);

        let node = RcExprNode {
            expr: Box::new(RcExpr::Let(x.clone(), rhs, cont)),
            source,
        };
        let node = build_retains(retains_before, node);

        let mut live_before = live_cont;
        live_before.remove(&x.name);
        for (v, _) in &operands {
            insert_if_local(&mut live_before, &v.name);
        }
        (node, live_before)
    }

    /// A `destructure container into fields; cont`. The container is consumed: RC insertion retains
    /// it beforehand iff it is used after the destructure (mirroring `get_scoped_obj`'s
    /// retain-if-used-later before `get_struct_fields`), and each field the continuation never uses is
    /// released. Code generation performs the extraction itself: a boxed container retains the fields
    /// and releases the container; an unboxed container moves the fields out and drops the fields not
    /// named here. So the field retains, the container release, and the dropped-field releases are not
    /// emitted here.
    fn insert_into_destructure(
        &self,
        container: RcVar,
        fields: Vec<(usize, RcVar)>,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let (cont, live_cont) = self.insert_into_expr(cont, live_after);

        // A field the continuation never uses is dead: release it after the extraction.
        let mut dead = vec![];
        for (_, fv) in &fields {
            if !live_cont.contains(&fv.name) && self.needs_rc(fv) {
                dead.push(fv.clone());
            }
        }
        let cont = build_releases(dead, cont);

        let node = RcExprNode {
            expr: Box::new(RcExpr::Destructure(container.clone(), fields.clone(), cont)),
            source,
        };
        // Retain the container before the destructure iff it is used afterward, so both the extracted
        // fields and the later use are covered.
        let node = self.retain_if_live(&container, &live_cont, node);

        let mut live_before = live_cont;
        for (_, fv) in &fields {
            live_before.remove(&fv.name);
        }
        insert_if_local(&mut live_before, &container.name);
        (node, live_before)
    }

    /// A `let x = match scrut { arms }; cont`.
    fn insert_into_match(
        &self,
        x: RcVar,
        scrut: RcVar,
        arms: Vec<MatchArm>,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        // The liveness this returns is the union over the arms, so with no arm every variable live
        // after the match would be reported dead before it and released early.
        assert!(!arms.is_empty(), "a match has at least one arm");
        let (cont, live_cont) = self.insert_into_expr(cont, live_after);
        // Variables used after the match (excluding the match result `x`): live across every arm.
        let mut live_after_match = live_cont.clone();
        live_after_match.remove(&x.name);

        // The local variables live at an arm's head: those some arm uses from the enclosing scope,
        // plus those live after the match. The match consumes the scrutinee at an arm's head — a
        // variant arm releases the container, and otherwise the payload carries the scrutinee away —
        // so this, rather than the liveness after the match, is the liveness that follows that
        // consumption. Both the dead-branch releases and the scrutinee retain read it.
        let free_in_arms: Vec<Set<FullName>> =
            arms.iter().map(|arm| self.arm_free_locals(arm)).collect();
        let mut live_at_arm_head: Set<FullName> = live_after_match.clone();
        for u in &free_in_arms {
            for n in u {
                live_at_arm_head.insert(n.clone());
            }
        }

        // A boxed union scrutinee's container is released in each arm (mirrors `get_union_value`);
        // an unbox union cancels (payload retain and container release cancel), so nothing.
        let release_container = scrut.ty.is_box(self.type_env);

        let mut new_arms = vec![];
        let mut live_before_arms: Set<FullName> = Set::default();
        for (arm, used) in arms.into_iter().zip(free_in_arms.iter()) {
            let payload = arm.payload.clone();
            let (body, body_live) = self.insert_into_expr(arm.body, &live_after_match);

            // Dead-branch (rule c): variables used in another arm but not this one, and dead after
            // the match, are released at this arm's head.
            let mut head = vec![];
            for n in &live_at_arm_head {
                if !used.contains(n) && !live_after_match.contains(n) {
                    // A free local of an arm is bound in the enclosing scope, so it is a known variable.
                    let v = self
                        .vars
                        .get(n)
                        .expect("a free local of a match arm is bound in the enclosing scope");
                    if self.needs_rc(v) {
                        head.push(v.clone());
                    }
                }
            }
            // Then the scrutinee container release (boxed union), for a variant arm only. A
            // catch-all arm binds the whole scrutinee as its payload, so the scrutinee flows into the
            // arm and is disposed through the payload, not by a container release here.
            if release_container && arm.tag.is_some() && self.needs_rc(&scrut) {
                head.push(scrut.clone());
            }
            // Then release the payload if the arm body never uses it.
            if !body_live.contains(&payload.name) && self.needs_rc(&payload) {
                head.push(payload.clone());
            }
            let body = build_releases(head, body);

            for n in &body_live {
                live_before_arms.insert(n.clone());
            }
            live_before_arms.remove(&payload.name);

            new_arms.push(MatchArm {
                tag: arm.tag,
                payload,
                body,
            });
        }

        // After-match release of a dead match result.
        let cont = if !live_cont.contains(&x.name) && self.needs_rc(&x) {
            build_releases(vec![x.clone()], cont)
        } else {
            cont
        };

        let node = RcExprNode {
            expr: Box::new(RcExpr::Let(
                x.clone(),
                RcRhs::Match(scrut.clone(), new_arms),
                cont,
            )),
            source,
        };
        // The scrutinee is owned, and the match consumes it at an arm's head, so a use of it inside an
        // arm body is a use after that consumption just as a use after the match is: retain it when
        // either occurs. One reference covers both — an arm that uses the scrutinee consumes the extra
        // reference, an arm that does not releases it by the dead-branch rule above, and an arm whose
        // body uses a scrutinee that also outlives the match retains it again at that use.
        let node = self.retain_if_live(&scrut, &live_at_arm_head, node);

        let mut live_before = live_before_arms;
        live_before.remove(&x.name);
        insert_if_local(&mut live_before, &scrut.name);
        (node, live_before)
    }

    /// The local variables an arm references from the enclosing scope (its free locals minus the
    /// payload the arm binds).
    fn arm_free_locals(&self, arm: &MatchArm) -> Set<FullName> {
        let mut free = free_locals(&arm.body);
        free.remove(&arm.payload.name);
        free
    }

    /// Whether a variable needs reference counting: a fully-unboxed value has no boxed leaf, so its
    /// `Retain`/`Release` would generate no code and is omitted.
    fn needs_rc(&self, var: &RcVar) -> bool {
        !var.ty.is_fully_unboxed(self.type_env)
    }

    /// Wrap `node` in a `Retain` of `var` iff `var` is a local that needs reference counting and is
    /// live in `live` — the owned-operand rule for a variable a consuming construct (a `Ret`, a
    /// `Destructure`, or a `Match` scrutinee) uses, when it is still live afterward.
    fn retain_if_live(&self, var: &RcVar, live: &Set<FullName>, node: RcExprNode) -> RcExprNode {
        if var.name.is_local() && live.contains(&var.name) && self.needs_rc(var) {
            build_retains(vec![var.clone()], node)
        } else {
            node
        }
    }
}

/// The operands of a compound expression together with how each is taken, in evaluation order
/// (callee before arguments). A `Match` rhs never reaches here; it is handled by
/// `insert_into_match`.
fn rhs_operands(rhs: &RcRhs, type_env: &TypeEnv) -> Vec<(RcVar, Ownership)> {
    match rhs {
        RcRhs::Var(v) => vec![(v.clone(), Ownership::Own)],
        RcRhs::App(callee, args) => {
            let mut ops = vec![(callee.clone(), Ownership::Own)];
            for a in args {
                ops.push((a.clone(), Ownership::Own));
            }
            ops
        }
        RcRhs::Closure(_, caps) => caps.iter().map(|c| (c.clone(), Ownership::Own)).collect(),
        RcRhs::Llvm(llvm_gen, args) => {
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            args.iter()
                .enumerate()
                .map(|(i, a)| {
                    let ownership = if llvm_gen.borrows_operand(i, &arg_tys, type_env) {
                        Ownership::Borrow
                    } else {
                        Ownership::Own
                    };
                    (a.clone(), ownership)
                })
                .collect()
        }
        RcRhs::Match(..) => {
            unreachable!("a Match rhs is handled by insert_into_match, not rhs_operands")
        }
    }
}

/// Wrap `cont` in a chain of `Retain` nodes (the first element outermost).
fn build_retains(vars: Vec<RcVar>, cont: RcExprNode) -> RcExprNode {
    vars.into_iter().rev().fold(cont, |c, v| {
        let source = v.source.clone();
        RcExprNode {
            expr: Box::new(RcExpr::Retain(v, vec![], RcState::Unknown, c)),
            source,
        }
    })
}

/// Wrap `cont` in a chain of `Release` nodes (the first element outermost).
fn build_releases(vars: Vec<RcVar>, cont: RcExprNode) -> RcExprNode {
    vars.into_iter().rev().fold(cont, |c, v| {
        let source = v.source.clone();
        RcExprNode {
            expr: Box::new(RcExpr::Release(v, vec![], RcState::Unknown, c)),
            source,
        }
    })
}

fn insert_if_local(set: &mut Set<FullName>, name: &FullName) {
    if name.is_local() {
        set.insert(name.clone());
    }
}

/// Returns the free local variables of `node`: the local names it references but does not itself
/// bind. Names are globally unique, so referenced-minus-bound is exact.
fn free_locals(node: &RcExprNode) -> Set<FullName> {
    let mut refs = Set::default();
    let mut bound = Set::default();
    collect_referenced_and_bound(node, &mut refs, &mut bound);
    refs.retain(|n| !bound.contains(n));
    refs
}

/// The traversal behind `free_locals`: record into `refs` every local name `node` references and
/// into `bound` every local name it binds — a `Let` variable, a `Match` arm's payload variable, or
/// a `Destructure` field — descending through the continuation and match arms.
fn collect_referenced_and_bound(
    node: &RcExprNode,
    refs: &mut Set<FullName>,
    bound: &mut Set<FullName>,
) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => insert_if_local(refs, &x.name),
        RcExpr::Let(x, rhs, k) => {
            bound.insert(x.name.clone());
            match rhs {
                RcRhs::Var(v) => insert_if_local(refs, &v.name),
                RcRhs::App(callee, args) => {
                    insert_if_local(refs, &callee.name);
                    for a in args {
                        insert_if_local(refs, &a.name);
                    }
                }
                RcRhs::Closure(_, caps) => {
                    for c in caps {
                        insert_if_local(refs, &c.name);
                    }
                }
                RcRhs::Llvm(_, args) => {
                    for a in args {
                        insert_if_local(refs, &a.name);
                    }
                }
                RcRhs::Match(scrut, arms) => {
                    insert_if_local(refs, &scrut.name);
                    for arm in arms {
                        bound.insert(arm.payload.name.clone());
                        collect_referenced_and_bound(&arm.body, refs, bound);
                    }
                }
            }
            collect_referenced_and_bound(k, refs, bound);
        }
        RcExpr::Destructure(container, fields, k) => {
            insert_if_local(refs, &container.name);
            for (_, fv) in fields {
                bound.insert(fv.name.clone());
            }
            collect_referenced_and_bound(k, refs, bound);
        }
        RcExpr::Retain(v, _, _, k) | RcExpr::Release(v, _, _, k) | RcExpr::Eval(v, k) => {
            insert_if_local(refs, &v.name);
            collect_referenced_and_bound(k, refs, bound);
        }
    }
}

/// Record a local variable's `RcVar` in the table, keyed by name.
fn record_if_local(vars: &mut Map<FullName, RcVar>, v: &RcVar) {
    if v.name.is_local() {
        vars.insert(v.name.clone(), v.clone());
    }
}

/// Collect every local variable's `RcVar` in an expression, keyed by name.
fn collect_vars(node: &RcExprNode, vars: &mut Map<FullName, RcVar>) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => record_if_local(vars, x),
        RcExpr::Let(x, rhs, k) => {
            record_if_local(vars, x);
            match rhs {
                RcRhs::Var(v) => record_if_local(vars, v),
                RcRhs::App(callee, args) => {
                    record_if_local(vars, callee);
                    for a in args {
                        record_if_local(vars, a);
                    }
                }
                RcRhs::Closure(_, caps) => {
                    for c in caps {
                        record_if_local(vars, c);
                    }
                }
                RcRhs::Llvm(_, args) => {
                    for a in args {
                        record_if_local(vars, a);
                    }
                }
                RcRhs::Match(scrut, arms) => {
                    record_if_local(vars, scrut);
                    for arm in arms {
                        record_if_local(vars, &arm.payload);
                        collect_vars(&arm.body, vars);
                    }
                }
            }
            collect_vars(k, vars);
        }
        RcExpr::Destructure(container, fields, k) => {
            record_if_local(vars, container);
            for (_, fv) in fields {
                record_if_local(vars, fv);
            }
            collect_vars(k, vars);
        }
        RcExpr::Retain(v, _, _, k) | RcExpr::Release(v, _, _, k) | RcExpr::Eval(v, k) => {
            record_if_local(vars, v);
            collect_vars(k, vars);
        }
    }
}
