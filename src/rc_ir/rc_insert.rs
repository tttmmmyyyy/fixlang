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
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{MatchArm, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcState, RcVar};

/// Insert explicit `Retain`/`Release` nodes into every function and global initializer of `prog`.
pub fn insert_rc(prog: &mut RcProgram, type_env: &TypeEnv) {
    let funcs = std::mem::take(&mut prog.funcs);
    let mut new_funcs = Map::default();
    for (fref, func) in funcs {
        let inserter = FuncRc::new(type_env, &func);
        new_funcs.insert(fref, inserter.insert_into_func(func));
    }
    prog.funcs = new_funcs;

    let globals = std::mem::take(&mut prog.globals);
    prog.globals = globals
        .into_iter()
        .map(|mut glob| {
            let inserter = FuncRc::new_for_expr(type_env, &glob.init);
            let (body, _live) = inserter.process(glob.init, &Set::default());
            glob.init = body;
            glob
        })
        .collect();
}

/// The reference-counting context for one function (or global initializer): the type environment
/// and a table from every local variable name to its `RcVar` (used to recover a variable's type and
/// span when placing a dead-branch release). Names are globally unique, so the table is unambiguous.
struct FuncRc<'a> {
    type_env: &'a TypeEnv,
    vars: Map<FullName, RcVar>,
}

impl<'a> FuncRc<'a> {
    fn new(type_env: &'a TypeEnv, func: &RcFunc) -> Self {
        let mut vars = Map::default();
        for p in &func.params {
            vars.insert(p.name.clone(), p.clone());
        }
        if let Some(cap) = &func.cap {
            vars.insert(cap.name.clone(), cap.clone());
        }
        collect_vars(&func.body, &mut vars);
        FuncRc { type_env, vars }
    }

    fn new_for_expr(type_env: &'a TypeEnv, expr: &RcExprNode) -> Self {
        let mut vars = Map::default();
        collect_vars(expr, &mut vars);
        FuncRc { type_env, vars }
    }

    /// Rewrite a function body, then release any parameter or capture that the body never uses.
    fn insert_into_func(&self, mut func: RcFunc) -> RcFunc {
        let (body, live) = self.process(func.body, &Set::default());

        let mut unused = vec![];
        for p in &func.params {
            if self.needs_rc(p) && !live.contains(&p.name) {
                unused.push(p.clone());
            }
        }
        if let Some(cap) = &func.cap {
            if self.needs_rc(cap) && !live.contains(&cap.name) {
                unused.push(cap.clone());
            }
        }
        func.body = build_releases(unused, body);
        func
    }

    /// Process one expression, given the set of local variables live *after* it. Returns the
    /// rewritten expression and the set of local variables live *before* it (at its entry).
    fn process(&self, node: RcExprNode, live_after: &Set<FullName>) -> (RcExprNode, Set<FullName>) {
        // The continuation chain recurses deeply for a large function (as lowering and code
        // generation do); grow the stack on demand so it does not overflow.
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.process_inner(node, live_after)
        })
    }

    fn process_inner(
        &self,
        node: RcExprNode,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let source = node.source.clone();
        match *node.expr {
            RcExpr::Ret(x) => {
                let mut live = live_after.clone();
                insert_local(&mut live, &x.name);
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
                self.process_match(x, scrut, arms, cont, source, live_after)
            }
            RcExpr::Let(x, rhs, cont) => {
                self.process_nonmatch_let(x, rhs, cont, source, live_after)
            }
            RcExpr::Destructure(container, fields, cont) => {
                self.process_destructure(container, fields, cont, source, live_after)
            }
            RcExpr::Retain(..) | RcExpr::Release(..) => {
                panic!("RC insertion runs on a skeleton that has no Retain/Release nodes yet")
            }
        }
    }

    /// A `let x = rhs; cont` whose `rhs` is not a `Match` (the `Match` case is `process_match`).
    fn process_nonmatch_let(
        &self,
        x: RcVar,
        rhs: RcRhs,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        assert!(
            !matches!(rhs, RcRhs::Match(..)),
            "process_nonmatch_let received a Match rhs; a Match is handled by process_match"
        );
        let (cont, live_cont) = self.process(cont, live_after);

        // Operand reference counting. Walk operands in reverse evaluation order so that, for a
        // variable used more than once, the last (right-most) use moves and the earlier uses retain.
        let operands = rhs_operands(&rhs);
        let mut running = live_cont.clone();
        let mut retains_before = vec![]; // Own operand used later -> retain before the statement.
        let mut releases_after = vec![]; // Borrow operand at its last use -> release after it.
        for (v, borrowed) in operands.iter().rev() {
            if v.name.is_local() {
                let used_later = running.contains(&v.name);
                if *borrowed {
                    if !used_later && self.needs_rc(v) {
                        releases_after.push(v.clone());
                    }
                } else if used_later && self.needs_rc(v) {
                    retains_before.push(v.clone());
                }
                running.insert(v.name.clone());
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
            insert_local(&mut live_before, &v.name);
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
    fn process_destructure(
        &self,
        container: RcVar,
        fields: Vec<(usize, RcVar)>,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let (cont, live_cont) = self.process(cont, live_after);

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
        insert_local(&mut live_before, &container.name);
        (node, live_before)
    }

    /// A `let x = match scrut { arms }; cont`.
    fn process_match(
        &self,
        x: RcVar,
        scrut: RcVar,
        arms: Vec<MatchArm>,
        cont: RcExprNode,
        source: Option<Span>,
        live_after: &Set<FullName>,
    ) -> (RcExprNode, Set<FullName>) {
        let (cont, live_cont) = self.process(cont, live_after);
        // Variables used after the match (excluding the match result `x`): live across every arm.
        let mut used_after = live_cont.clone();
        used_after.remove(&x.name);

        // Free local variables each arm uses (from the enclosing scope), for dead-branch releases.
        let arm_used: Vec<Set<FullName>> =
            arms.iter().map(|arm| self.arm_free_locals(arm)).collect();
        let mut used_in_any: Set<FullName> = Set::default();
        for u in &arm_used {
            for n in u {
                used_in_any.insert(n.clone());
            }
        }

        // A boxed union scrutinee's container is released in each arm (mirrors `get_union_value`);
        // an unbox union cancels (payload retain and container release cancel), so nothing.
        let release_container = scrut.ty.is_box(self.type_env);

        let mut new_arms = vec![];
        let mut live_before_arms: Set<FullName> = Set::default();
        for (arm, used) in arms.into_iter().zip(arm_used.iter()) {
            let payload = arm.payload.clone();
            let (body, body_live) = self.process(arm.body, &used_after);

            // Dead-branch (rule c): variables used in another arm but not this one, and dead after
            // the match, are released at this arm's head.
            let mut head = vec![];
            for n in &used_in_any {
                if !used.contains(n) && !used_after.contains(n) {
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
            if release_container && arm.variant.is_some() && self.needs_rc(&scrut) {
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
                variant: arm.variant,
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
        // The scrutinee is owned: retain it before the match if it is used after the match.
        let node = self.retain_if_live(&scrut, &used_after, node);

        let mut live_before = live_before_arms;
        live_before.remove(&x.name);
        insert_local(&mut live_before, &scrut.name);
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

/// The operands of a compound expression together with whether each is only borrowed, in evaluation
/// order (callee before arguments). A `Match` rhs never reaches here; it is handled by
/// `process_match`.
fn rhs_operands(rhs: &RcRhs) -> Vec<(RcVar, bool)> {
    match rhs {
        RcRhs::Var(v) => vec![(v.clone(), false)],
        RcRhs::App(callee, args) => {
            let mut ops = vec![(callee.clone(), false)];
            for a in args {
                ops.push((a.clone(), false));
            }
            ops
        }
        RcRhs::Closure(_, caps) => caps.iter().map(|c| (c.clone(), false)).collect(),
        RcRhs::Llvm(gen, args) => args
            .iter()
            .enumerate()
            .map(|(i, a)| (a.clone(), gen.borrows_operand(i)))
            .collect(),
        RcRhs::Match(..) => {
            unreachable!("a Match rhs is handled by process_match, not rhs_operands")
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

fn insert_local(set: &mut Set<FullName>, name: &FullName) {
    if name.is_local() {
        set.insert(name.clone());
    }
}

/// Returns the free local variables of `node`: the local names it references but does not itself
/// bind. Names are globally unique, so referenced-minus-bound is exact.
fn free_locals(node: &RcExprNode) -> Set<FullName> {
    let mut refs = Set::default();
    let mut bound = Set::default();
    collect_refs_bound(node, &mut refs, &mut bound);
    refs.retain(|n| !bound.contains(n));
    refs
}

/// The traversal behind `free_locals`: record into `refs` every local name `node` references and
/// into `bound` every local name it binds — a `Let` variable, a `Match` arm's payload variable, or
/// a `Destructure` field — descending through the continuation and match arms.
fn collect_refs_bound(node: &RcExprNode, refs: &mut Set<FullName>, bound: &mut Set<FullName>) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => insert_local(refs, &x.name),
        RcExpr::Let(x, rhs, k) => {
            bound.insert(x.name.clone());
            match rhs {
                RcRhs::Var(v) => insert_local(refs, &v.name),
                RcRhs::App(callee, args) => {
                    insert_local(refs, &callee.name);
                    for a in args {
                        insert_local(refs, &a.name);
                    }
                }
                RcRhs::Closure(_, caps) => {
                    for c in caps {
                        insert_local(refs, &c.name);
                    }
                }
                RcRhs::Llvm(_, args) => {
                    for a in args {
                        insert_local(refs, &a.name);
                    }
                }
                RcRhs::Match(scrut, arms) => {
                    insert_local(refs, &scrut.name);
                    for arm in arms {
                        bound.insert(arm.payload.name.clone());
                        collect_refs_bound(&arm.body, refs, bound);
                    }
                }
            }
            collect_refs_bound(k, refs, bound);
        }
        RcExpr::Destructure(container, fields, k) => {
            insert_local(refs, &container.name);
            for (_, fv) in fields {
                bound.insert(fv.name.clone());
            }
            collect_refs_bound(k, refs, bound);
        }
        RcExpr::Retain(v, _, _, k) | RcExpr::Release(v, _, _, k) => {
            insert_local(refs, &v.name);
            collect_refs_bound(k, refs, bound);
        }
    }
}

/// Record a local variable's `RcVar` in the table, keyed by name.
fn note_var(vars: &mut Map<FullName, RcVar>, v: &RcVar) {
    if v.name.is_local() {
        vars.insert(v.name.clone(), v.clone());
    }
}

/// Collect every local variable's `RcVar` in an expression, keyed by name.
fn collect_vars(node: &RcExprNode, vars: &mut Map<FullName, RcVar>) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => note_var(vars, x),
        RcExpr::Let(x, rhs, k) => {
            note_var(vars, x);
            match rhs {
                RcRhs::Var(v) => note_var(vars, v),
                RcRhs::App(callee, args) => {
                    note_var(vars, callee);
                    for a in args {
                        note_var(vars, a);
                    }
                }
                RcRhs::Closure(_, caps) => {
                    for c in caps {
                        note_var(vars, c);
                    }
                }
                RcRhs::Llvm(_, args) => {
                    for a in args {
                        note_var(vars, a);
                    }
                }
                RcRhs::Match(scrut, arms) => {
                    note_var(vars, scrut);
                    for arm in arms {
                        note_var(vars, &arm.payload);
                        collect_vars(&arm.body, vars);
                    }
                }
            }
            collect_vars(k, vars);
        }
        RcExpr::Destructure(container, fields, k) => {
            note_var(vars, container);
            for (_, fv) in fields {
                note_var(vars, fv);
            }
            collect_vars(k, vars);
        }
        RcExpr::Retain(v, _, _, k) | RcExpr::Release(v, _, _, k) => {
            note_var(vars, v);
            collect_vars(k, vars);
        }
    }
}
