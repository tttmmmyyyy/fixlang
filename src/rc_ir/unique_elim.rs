//! Uniqueness-driven specialization and unique-check elimination on the RC IR.
//!
//! An operation that force-uniques its container before mutating it in place (`Array::set`, `swap`,
//! `mod`/`act` via punch and plug, struct `set`/`mod`) carries a runtime check that clones the
//! container when it is shared. Where the provenance analysis proves the container statically
//! `Unique` at the operation, that check is redundant and this pass drops it. The same holds for
//! `is_unique`, which reports a container's uniqueness at run time: where the container is provably
//! `Unique` the pass replaces it with the constant `true`, so the back end folds away the branch it
//! guarded (as in a generic `act`, whose unique arm then mutates in place without any check).
//!
//! A container's uniqueness usually depends on its function's inputs (an array threaded through a
//! loop is unique exactly when the loop was entered with a unique array). To resolve that, the pass
//! *specializes*: it clones a function per input-uniqueness key reaching it, starting from the entry
//! and the global initializers and following direct calls, so each clone knows its inputs' uniqueness
//! and can drop the checks that key makes provable. A call is routed to the clone for the argument
//! uniqueness it passes.
//!
//! The clone keyed on all-`Dynamic` inputs (nothing known) keeps the original function's name; every
//! other key gets a fresh name. Only funptr functions are specialized — a closure is reached only
//! indirectly, so it keeps its single all-`Dynamic` version. Every function keeps its all-`Dynamic`
//! version (the entry points a program is compiled for are not identifiable from the RC IR alone, so
//! no original is dropped, matching how code generation emits every function and lets the back end
//! remove the unreachable ones); specialization only adds the more specific clones the call sites
//! reach.

use crate::ast::inline_llvm::LLVMGenerator;
use crate::ast::program::TypeEnv;
use crate::misc::{Map, Set};
use crate::rc_ir::ast::{
    FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs, RcVar,
};
use crate::rc_ir::provenance::{analyze_program, leaf_is_unique, resolve, Analysis, Uniqueness};
use crate::rc_ir::rename::{collect_binders, fresh_rename, rename_expr, rename_var};
use std::collections::VecDeque;

/// The input-uniqueness a function clone is specialized on: the uniqueness of each parameter.
type Key = Vec<Uniqueness>;

/// Specialize `prog` and eliminate the unique checks each specialization makes provable.
pub fn specialize(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let analysis = analyze_program(prog, type_env);
    let mut spec = Specializer {
        prog,
        type_env,
        analysis,
        beneficial: beneficial_funcs(prog),
        clone_names: Map::default(),
        requested: Set::default(),
        worklist: VecDeque::new(),
        counter: 0,
        funcs: Map::default(),
    };

    // Keep every function's all-`Dynamic` version, since a program's entry points are not
    // identifiable from the RC IR. Specialization adds the more specific clones on top.
    let frefs: Vec<FuncRef> = prog.funcs.keys().cloned().collect();
    for fref in &frefs {
        let ck = spec.canonical_key(fref);
        spec.request(fref, ck);
    }
    // A global initializer has no parameters, so its body resolves against no inputs.
    let globals: Vec<RcGlobalInit> = prog
        .globals
        .iter()
        .map(|g| RcGlobalInit {
            symbol: g.symbol.clone(),
            ty: g.ty.clone(),
            init: spec.rewrite_body(&g.init, &[]),
        })
        .collect();

    // Materialize every requested clone; each materialization may request further clones.
    while let Some((fref, key)) = spec.worklist.pop_front() {
        let clone = spec.materialize(&fref, &key);
        spec.funcs.insert(clone.name.clone(), clone);
    }

    RcProgram {
        funcs: spec.funcs,
        globals,
        entry: prog.entry.clone(),
    }
}

struct Specializer<'a> {
    prog: &'a RcProgram,
    type_env: &'a TypeEnv,
    analysis: Analysis,
    /// The functions worth specializing: those whose body reaches a uniqueness check (a force-unique
    /// op or `is_unique`), directly or through a direct call. A function that reaches none (a read-only
    /// function) is the same under every key, so specializing it would only make redundant clones; its
    /// calls route to its canonical version.
    beneficial: Set<FuncRef>,
    /// The fresh name of each non-canonical clone `(function, key)`.
    clone_names: Map<(FuncRef, Key), FuncRef>,
    /// Every `(function, key)` already enqueued, so each is materialized once.
    requested: Set<(FuncRef, Key)>,
    worklist: VecDeque<(FuncRef, Key)>,
    counter: u64,
    /// The materialized clones, keyed by their output name.
    funcs: Map<FuncRef, RcFunc>,
}

impl<'a> Specializer<'a> {
    /// The all-`Dynamic` key of a function: nothing is known about its inputs' uniqueness. The clone
    /// on this key keeps the original name.
    fn canonical_key(&self, fref: &FuncRef) -> Key {
        self.canonical_key_of(&self.prog.funcs[fref])
    }

    /// The all-`Dynamic` key built from a function's parameter types.
    fn canonical_key_of(&self, func: &RcFunc) -> Key {
        func.params
            .iter()
            .map(|p| Uniqueness::all_dynamic(&p.ty, self.type_env))
            .collect()
    }

    /// The uniqueness of every input of the clone `(func, key)`: the key gives the parameters; a
    /// closure capture (the input past the parameters) is always all-`Dynamic`, since closures are not
    /// specialized.
    fn resolve_inputs(&self, func: &RcFunc, key: &Key) -> Vec<Uniqueness> {
        let mut inputs = key.clone();
        if let Some(cap) = &func.cap {
            inputs.push(Uniqueness::all_dynamic(&cap.ty, self.type_env));
        }
        inputs
    }

    /// The output name of the clone `(fref, key)`: the original name for the canonical (all-`Dynamic`)
    /// key, otherwise a fresh name minted (and memoized) once per key.
    fn name_of(&mut self, fref: &FuncRef, key: &Key) -> FuncRef {
        if *key == self.canonical_key(fref) {
            return fref.clone();
        }
        if let Some(n) = self.clone_names.get(&(fref.clone(), key.clone())) {
            return n.clone();
        }
        self.counter += 1;
        let mut n = fref.name.clone();
        n.name = format!("{}#u{}", n.name, self.counter);
        let nref = FuncRef { name: n };
        self.clone_names
            .insert((fref.clone(), key.clone()), nref.clone());
        nref
    }

    /// Request the clone `(fref, key)`: return its output name and, the first time, enqueue it for
    /// materialization.
    fn request(&mut self, fref: &FuncRef, key: Key) -> FuncRef {
        let name = self.name_of(fref, &key);
        if self.requested.insert((fref.clone(), key.clone())) {
            self.worklist.push_back((fref.clone(), key));
        }
        name
    }

    /// Materialize one clone: rewrite the original body under the clone's inputs (flipping the checks
    /// its key makes provable and routing its direct calls), then, for a fresh clone, give every local
    /// a fresh name so its names do not collide with the original's.
    fn materialize(&mut self, fref: &FuncRef, key: &Key) -> RcFunc {
        let func = self.prog.funcs[fref].clone();
        let inputs = self.resolve_inputs(&func, key);
        let body = self.rewrite_body(&func.body, &inputs);
        let name = self.name_of(fref, key);
        if name == *fref {
            return RcFunc { body, ..func };
        }
        let mut rename = Map::default();
        for p in func.params.iter().chain(func.cap.iter()) {
            fresh_rename(&p.name, "u", &mut rename, &mut self.counter);
        }
        collect_binders(&body, "u", &mut rename, &mut self.counter);
        RcFunc {
            name,
            fn_ty: func.fn_ty.clone(),
            params: func.params.iter().map(|p| rename_var(p, &rename)).collect(),
            cap: func.cap.as_ref().map(|c| rename_var(c, &rename)),
            ret_ty: func.ret_ty.clone(),
            body: rename_expr(&body, &rename),
            source: func.source.clone(),
        }
    }

    fn rewrite_body(&mut self, node: &RcExprNode, inputs: &[Uniqueness]) -> RcExprNode {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.rewrite_body_inner(node, inputs)
        })
    }

    fn rewrite_body_inner(&mut self, node: &RcExprNode, inputs: &[Uniqueness]) -> RcExprNode {
        let expr = match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let callee = self.route(x, callee, inputs);
                RcExpr::Let(
                    x.clone(),
                    RcRhs::App(callee, args.clone()),
                    self.rewrite_body(k, inputs),
                )
            }
            RcExpr::Let(x, RcRhs::Llvm(gen, args), k) => {
                let gen = self.maybe_elide(x, gen, inputs);
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Llvm(gen, args.clone()),
                    self.rewrite_body(k, inputs),
                )
            }
            RcExpr::Let(x, RcRhs::Match(scrutinee, arms), k) => {
                let arms = arms
                    .iter()
                    .map(|arm| MatchArm {
                        variant: arm.variant,
                        payload: arm.payload.clone(),
                        body: self.rewrite_body(&arm.body, inputs),
                    })
                    .collect();
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrutinee.clone(), arms),
                    self.rewrite_body(k, inputs),
                )
            }
            // `Var` and `Closure` need no routing (a closure's target keeps its original name, whose
            // all-`Dynamic` version is always kept), so their right-hand sides pass through unchanged.
            RcExpr::Let(x, rhs, k) => {
                RcExpr::Let(x.clone(), rhs.clone(), self.rewrite_body(k, inputs))
            }
            RcExpr::Retain(v, path, state, k) => {
                RcExpr::Retain(v.clone(), path.clone(), *state, self.rewrite_body(k, inputs))
            }
            RcExpr::Release(v, path, state, k) => {
                RcExpr::Release(v.clone(), path.clone(), *state, self.rewrite_body(k, inputs))
            }
            RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
                container.clone(),
                fields.clone(),
                self.rewrite_body(k, inputs),
            ),
            RcExpr::Ret(v) => RcExpr::Ret(v.clone()),
        };
        RcExprNode {
            expr: Box::new(expr),
            source: node.source.clone(),
        }
    }

    /// Route a direct call: retarget the callee to the clone for the argument uniqueness this call
    /// passes, requesting that clone. An indirect call (the callee is a closure value, not a function
    /// name) is left as is. A funptr function is specialized; a closure named directly is not.
    fn route(&mut self, call: &RcVar, callee: &RcVar, inputs: &[Uniqueness]) -> RcVar {
        let cref = FuncRef {
            name: callee.name.clone(),
        };
        let Some(g) = self.prog.funcs.get(&cref) else {
            return callee.clone();
        };
        // Only funptr functions worth specializing are cloned; a closure named directly (unusual) and
        // a read-only function keep their always-present all-`Dynamic` version.
        if g.cap.is_some() || !self.beneficial.contains(&cref) {
            return callee.clone();
        }
        let key = self.callee_key(call, g, inputs);
        let name = self.request(&cref, key);
        let mut c = callee.clone();
        c.name = name.name;
        c
    }

    /// The key of a direct callee `g`: the uniqueness of each argument at the call, resolved against
    /// the caller's own input uniqueness. An arity mismatch (a partial application) resolves to the
    /// canonical key, leaving the callee unspecialized.
    fn callee_key(&self, call: &RcVar, g: &RcFunc, inputs: &[Uniqueness]) -> Key {
        match self.analysis.call_args.get(&call.name) {
            Some(arg_provs) if arg_provs.len() == g.params.len() => arg_provs
                .iter()
                .map(|prov| resolve(prov, inputs))
                .collect(),
            _ => self.canonical_key_of(g),
        }
    }

    /// Drop the runtime uniqueness check from an operation whose checked container this clone's inputs
    /// make unique — a force-unique mutation loses its clone-when-shared, and an `is_unique` becomes
    /// the constant `true`, which lets the back end fold the branch it guarded.
    fn maybe_elide(
        &self,
        result: &RcVar,
        gen: &LLVMGenerator,
        inputs: &[Uniqueness],
    ) -> LLVMGenerator {
        let Some((_, path)) = gen.unique_check_operand() else {
            return gen.clone();
        };
        let unique = self
            .analysis
            .op_containers
            .get(&result.name)
            .map_or(false, |prov| leaf_is_unique(prov, &path, inputs));
        if unique {
            gen.assuming_unique()
        } else {
            gen.clone()
        }
    }
}

/// The functions whose body reaches a uniqueness check (a force-unique op or `is_unique`) — directly,
/// or through a direct call to another such function. Only these are worth specializing; the rest are
/// the same under every key. A least fixed point over the direct-call graph.
fn beneficial_funcs(prog: &RcProgram) -> Set<FuncRef> {
    // Each function's direct callees, and whether its own body performs a uniqueness check.
    let mut callees: Map<FuncRef, Vec<FuncRef>> = Map::default();
    let mut beneficial: Set<FuncRef> = Set::default();
    for (fref, func) in &prog.funcs {
        let mut cs = vec![];
        let mut has_unique_check = false;
        scan_body(&func.body, prog, &mut cs, &mut has_unique_check);
        if has_unique_check {
            beneficial.insert(fref.clone());
        }
        callees.insert(fref.clone(), cs);
    }
    // A function that calls a beneficial function is itself beneficial.
    loop {
        let mut changed = false;
        for (fref, cs) in &callees {
            if !beneficial.contains(fref) && cs.iter().any(|c| beneficial.contains(c)) {
                beneficial.insert(fref.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    beneficial
}

/// Collect a body's direct callees (functions of `prog`) and whether it performs a uniqueness check.
fn scan_body(
    node: &RcExprNode,
    prog: &RcProgram,
    callees: &mut Vec<FuncRef>,
    has_unique_check: &mut bool,
) {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        match node.expr.as_ref() {
            RcExpr::Let(_, rhs, k) => {
                match rhs {
                    RcRhs::Llvm(gen, _) => {
                        if gen.unique_check_operand().is_some() {
                            *has_unique_check = true;
                        }
                    }
                    RcRhs::App(callee, _) => {
                        let cref = FuncRef {
                            name: callee.name.clone(),
                        };
                        if prog.funcs.contains_key(&cref) {
                            callees.push(cref);
                        }
                    }
                    RcRhs::Match(_, arms) => {
                        for arm in arms {
                            scan_body(&arm.body, prog, callees, has_unique_check);
                        }
                    }
                    RcRhs::Var(_) | RcRhs::Closure(..) => {}
                }
                scan_body(k, prog, callees, has_unique_check);
            }
            RcExpr::Retain(_, _, _, k)
            | RcExpr::Release(_, _, _, k)
            | RcExpr::Destructure(_, _, k) => scan_body(k, prog, callees, has_unique_check),
            RcExpr::Ret(_) => {}
        }
    })
}
