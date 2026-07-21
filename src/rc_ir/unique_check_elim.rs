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

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{Map, Set};
use crate::rc_ir::ast::{
    FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs, RcVar,
};
use crate::rc_ir::provenance::{
    analyze_program, leaf_is_unique, resolve, ProvenanceAnalysis, Uniqueness,
};
use crate::rc_ir::rename::fresh_rename_function;
use std::collections::VecDeque;
use std::sync::Arc;

/// The input-uniqueness a function clone is specialized on: the uniqueness of each parameter.
type InputUniqueness = Vec<Uniqueness>;

/// Specialize `prog` and eliminate the unique checks each specialization makes provable.
pub fn specialize(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let analysis = analyze_program(prog, type_env);
    let mut spec = Specializer {
        prog,
        type_env,
        analysis,
        reaches_unique_check: funcs_reaching_unique_check(prog, type_env),
        clone_names: Map::default(),
        requested: Set::default(),
        worklist: VecDeque::new(),
        fresh_name_counter: 0,
        output_funcs: Map::default(),
    };

    // Keep every function's all-`Dynamic` version, since a program's entry points are not
    // identifiable from the RC IR. Specialization adds the more specific clones on top.
    let frefs: Vec<FuncRef> = prog.funcs.keys().cloned().collect();
    for fref in &frefs {
        let ck = spec.canonical_key(fref);
        spec.request_clone(fref, ck);
    }
    // A global initializer has no parameters, so its body resolves against no inputs.
    let globals: Vec<RcGlobalInit> = prog
        .globals
        .iter()
        .map(|g| RcGlobalInit {
            symbol: g.symbol.clone(),
            ty: g.ty.clone(),
            init: spec.rewrite_expr(&g.init, &[]),
        })
        .collect();

    // Materialize every requested clone; each materialization may request further clones.
    while let Some((fref, key)) = spec.worklist.pop_front() {
        let clone = spec.materialize_clone(&fref, &key);
        spec.output_funcs.insert(clone.name.clone(), clone);
    }

    RcProgram {
        funcs: spec.output_funcs,
        globals,
        entry: prog.entry.clone(),
    }
}

/// The mutable state of the specialization pass: the program and analysis it reads, and the clones
/// it accumulates as it walks the reachable `(function, key)` pairs.
struct Specializer<'a> {
    prog: &'a RcProgram,
    type_env: &'a TypeEnv,
    analysis: ProvenanceAnalysis,
    /// The functions worth specializing: those whose body reaches a uniqueness check (a force-unique
    /// op or `is_unique`), directly or through a direct call. A function that reaches none (a read-only
    /// function) is the same under every key, so specializing it would only make redundant clones; its
    /// calls route to its canonical version.
    reaches_unique_check: Set<FuncRef>,
    /// The fresh name of each non-canonical clone `(function, key)`.
    clone_names: Map<(FuncRef, InputUniqueness), FuncRef>,
    /// Every `(function, key)` already enqueued, so each is materialized once.
    requested: Set<(FuncRef, InputUniqueness)>,
    worklist: VecDeque<(FuncRef, InputUniqueness)>,
    fresh_name_counter: u64,
    /// The materialized clones, keyed by their output name.
    output_funcs: Map<FuncRef, RcFunc>,
}

impl<'a> Specializer<'a> {
    /// The all-`Dynamic` key of a function: nothing is known about its inputs' uniqueness. The clone
    /// on this key keeps the original name.
    fn canonical_key(&self, fref: &FuncRef) -> InputUniqueness {
        self.canonical_key_of(&self.prog.funcs[fref])
    }

    /// The all-`Dynamic` key built from a function's parameter types.
    fn canonical_key_of(&self, func: &RcFunc) -> InputUniqueness {
        func.params
            .iter()
            .map(|p| Uniqueness::all_dynamic(&p.ty, self.type_env))
            .collect()
    }

    /// The uniqueness of every input of the clone `(func, key)`: the key gives the parameters; a
    /// closure capture (the input past the parameters) is always all-`Dynamic`, since closures are not
    /// specialized.
    fn clone_inputs(&self, func: &RcFunc, key: &InputUniqueness) -> Vec<Uniqueness> {
        let mut inputs = key.clone();
        if let Some(cap) = &func.capture {
            inputs.push(Uniqueness::all_dynamic(&cap.ty, self.type_env));
        }
        inputs
    }

    /// The output name of the clone `(fref, key)`: the original name for the canonical (all-`Dynamic`)
    /// key, otherwise a fresh name minted (and memoized) once per key.
    fn clone_name(&mut self, fref: &FuncRef, key: &InputUniqueness) -> FuncRef {
        if *key == self.canonical_key(fref) {
            return fref.clone();
        }
        if let Some(n) = self.clone_names.get(&(fref.clone(), key.clone())) {
            return n.clone();
        }
        self.fresh_name_counter += 1;
        let mut n = fref.name.clone();
        n.name = format!("{}#u{}", n.name, self.fresh_name_counter);
        let nref = FuncRef { name: n };
        self.clone_names
            .insert((fref.clone(), key.clone()), nref.clone());
        nref
    }

    /// Request the clone `(fref, key)`: return its output name and, the first time, enqueue it for
    /// materialization.
    fn request_clone(&mut self, fref: &FuncRef, key: InputUniqueness) -> FuncRef {
        let name = self.clone_name(fref, &key);
        if self.requested.insert((fref.clone(), key.clone())) {
            self.worklist.push_back((fref.clone(), key));
        }
        name
    }

    /// Materialize one clone: rewrite the original body under the clone's inputs (flipping the checks
    /// its key makes provable and routing its direct calls), then, for a fresh clone, give every local
    /// a fresh name so its names do not collide with the original's.
    fn materialize_clone(&mut self, fref: &FuncRef, key: &InputUniqueness) -> RcFunc {
        let func = self.prog.funcs[fref].clone();
        let inputs = self.clone_inputs(&func, key);
        let body = self.rewrite_expr(&func.body, &inputs);
        let name = self.clone_name(fref, key);
        if name == *fref {
            return RcFunc { body, ..func };
        }
        let (params, capture, body, rename) = fresh_rename_function(
            &func.params,
            &func.capture,
            &body,
            "u",
            &mut self.fresh_name_counter,
        );
        RcFunc {
            name,
            fn_ty: func.fn_ty.clone(),
            params,
            capture,
            ret_ty: func.ret_ty.clone(),
            body,
            source: func.source.clone(),
            // Carry the ownership annotation, remapping its parameter keys through the same renaming.
            borrowed_units: func
                .borrowed_units
                .iter()
                .map(|(n, unit)| {
                    // Every `borrowed_units` key is a parameter or capture name, and
                    // `fresh_rename_function` renames all of those, so the lookup always hits.
                    let renamed = rename.get(n).cloned().unwrap_or_else(|| {
                        unreachable!(
                            "borrowed_units key {:?} is not a renamed parameter/capture",
                            n
                        )
                    });
                    (renamed, unit.clone())
                })
                .collect(),
        }
    }

    /// Rewrite a function body under `inputs` (the uniqueness of the enclosing clone's inputs),
    /// growing the stack for deeply nested bodies.
    fn rewrite_expr(&mut self, node: &RcExprNode, inputs: &[Uniqueness]) -> RcExprNode {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.rewrite_expr_inner(node, inputs)
        })
    }

    /// Rewrite one expression node under `inputs`: route its direct calls to specialized clones and
    /// elide the uniqueness checks `inputs` make provable, recursing into continuations and match arms.
    fn rewrite_expr_inner(&mut self, node: &RcExprNode, inputs: &[Uniqueness]) -> RcExprNode {
        let expr = match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let callee = self.retarget_call(x, callee, inputs);
                RcExpr::Let(
                    x.clone(),
                    RcRhs::App(callee, args.clone()),
                    self.rewrite_expr(k, inputs),
                )
            }
            RcExpr::Let(x, RcRhs::Llvm(llvm_gen, args), k) => {
                let llvm_gen = self.elide_unique_check_if_provable(x, llvm_gen, args, inputs);
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Llvm(llvm_gen, args.clone()),
                    self.rewrite_expr(k, inputs),
                )
            }
            RcExpr::Let(x, RcRhs::Match(scrutinee, arms), k) => {
                let arms = arms
                    .iter()
                    .map(|arm| MatchArm {
                        tag: arm.tag,
                        payload: arm.payload.clone(),
                        body: self.rewrite_expr(&arm.body, inputs),
                    })
                    .collect();
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrutinee.clone(), arms),
                    self.rewrite_expr(k, inputs),
                )
            }
            // `Var` and `Closure` need no routing (a closure's target keeps its original name, whose
            // all-`Dynamic` version is always kept), so their right-hand sides pass through unchanged.
            // Listed explicitly (not a catch-all) so a new `RcRhs` that might need routing fails to
            // compile here instead of silently passing through.
            RcExpr::Let(x, rhs @ (RcRhs::Var(_) | RcRhs::Closure(_, _)), k) => {
                RcExpr::Let(x.clone(), rhs.clone(), self.rewrite_expr(k, inputs))
            }
            RcExpr::Retain(v, path, state, k) => RcExpr::Retain(
                v.clone(),
                path.clone(),
                *state,
                self.rewrite_expr(k, inputs),
            ),
            RcExpr::Release(v, path, state, k) => RcExpr::Release(
                v.clone(),
                path.clone(),
                *state,
                self.rewrite_expr(k, inputs),
            ),
            RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
                container.clone(),
                fields.clone(),
                self.rewrite_expr(k, inputs),
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
    fn retarget_call(&mut self, call: &RcVar, callee: &RcVar, inputs: &[Uniqueness]) -> RcVar {
        let cref = FuncRef {
            name: callee.name.clone(),
        };
        let Some(g) = self.prog.funcs.get(&cref) else {
            return callee.clone();
        };
        // Only funptr functions worth specializing are cloned; a closure named directly (unusual) and
        // a read-only function keep their always-present all-`Dynamic` version.
        if g.capture.is_some() || !self.reaches_unique_check.contains(&cref) {
            return callee.clone();
        }
        let key = self.callee_key(call, g, inputs);
        let name = self.request_clone(&cref, key);
        let mut c = callee.clone();
        c.name = name.name;
        c
    }

    /// The key of a direct callee `g`: the uniqueness of each argument at the call, resolved against
    /// the caller's own input uniqueness.
    fn callee_key(&self, call: &RcVar, g: &RcFunc, inputs: &[Uniqueness]) -> InputUniqueness {
        // `interpret_app` records `call_arg_provs` for every call, so the entry always exists.
        let arg_provs = self
            .analysis
            .call_arg_provs
            .get(&call.name)
            .unwrap_or_else(|| {
                unreachable!("call_arg_provs has no entry for the call {:?}", call.name)
            });
        // Code generation requires every call to supply one argument per parameter, so the key has one
        // entry per parameter.
        assert_eq!(
            arg_provs.len(),
            g.params.len(),
            "call to `{}` supplies {} arguments to {} parameters",
            g.name.name.to_string(),
            arg_provs.len(),
            g.params.len()
        );
        arg_provs.iter().map(|prov| resolve(prov, inputs)).collect()
    }

    /// Drop the runtime uniqueness check from an operation whose checked container this clone's inputs
    /// make unique — a force-unique mutation loses its clone-when-shared, and an `is_unique` becomes
    /// the constant `true`, which lets the back end fold the branch it guarded.
    fn elide_unique_check_if_provable(
        &self,
        result: &RcVar,
        llvm_gen: &Box<dyn LLVMGen>,
        args: &[RcVar],
        inputs: &[Uniqueness],
    ) -> Box<dyn LLVMGen> {
        let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
        let Some(check) = llvm_gen.unique_check_operand(&arg_tys, self.type_env) else {
            return llvm_gen.clone();
        };
        // `interpret_rhs` records `unique_check_operand_provs` for exactly the ops that carry a `unique_check_operand`
        // — the same condition the `let Some(check)` guard above passed — so the entry always exists.
        let container_prov = self
            .analysis
            .unique_check_operand_provs
            .get(&result.name)
            .unwrap_or_else(|| {
                unreachable!(
                    "unique_check_operand_provs has no entry for the unique-check op {:?}",
                    result.name
                )
            });
        let unique = leaf_is_unique(container_prov, &check.path, inputs);
        if unique {
            llvm_gen.assuming_unique()
        } else {
            llvm_gen.clone()
        }
    }
}

/// The functions whose body reaches a uniqueness check (a force-unique op or `is_unique`) — directly,
/// or through a direct call to another such function. Only these are worth specializing; the rest are
/// the same under every key. A least fixed point over the direct-call graph.
fn funcs_reaching_unique_check(prog: &RcProgram, type_env: &TypeEnv) -> Set<FuncRef> {
    // Each function's direct callees, and whether its own body performs a uniqueness check.
    let mut callees: Map<FuncRef, Vec<FuncRef>> = Map::default();
    let mut reaches_unique_check: Set<FuncRef> = Set::default();
    for (fref, func) in &prog.funcs {
        let mut cs = vec![];
        let mut has_unique_check = false;
        collect_callees_and_unique_check(
            &func.body,
            prog,
            type_env,
            &mut cs,
            &mut has_unique_check,
        );
        if has_unique_check {
            reaches_unique_check.insert(fref.clone());
        }
        callees.insert(fref.clone(), cs);
    }
    // A function that calls a reaches_unique_check function is itself reaches_unique_check.
    loop {
        let mut changed = false;
        for (fref, cs) in &callees {
            if !reaches_unique_check.contains(fref)
                && cs.iter().any(|c| reaches_unique_check.contains(c))
            {
                reaches_unique_check.insert(fref.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    reaches_unique_check
}

/// Collect a body's direct callees (functions of `prog`) and whether it performs a uniqueness check.
fn collect_callees_and_unique_check(
    node: &RcExprNode,
    prog: &RcProgram,
    type_env: &TypeEnv,
    callees: &mut Vec<FuncRef>,
    has_unique_check: &mut bool,
) {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || match node.expr.as_ref() {
        RcExpr::Let(_, rhs, k) => {
            match rhs {
                RcRhs::Llvm(llvm_gen, args) => {
                    let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
                    if llvm_gen.unique_check_operand(&arg_tys, type_env).is_some() {
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
                        collect_callees_and_unique_check(
                            &arm.body,
                            prog,
                            type_env,
                            callees,
                            has_unique_check,
                        );
                    }
                }
                RcRhs::Var(_) | RcRhs::Closure(..) => {}
            }
            collect_callees_and_unique_check(k, prog, type_env, callees, has_unique_check);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Destructure(_, _, k) => {
            collect_callees_and_unique_check(k, prog, type_env, callees, has_unique_check)
        }
        RcExpr::Ret(_) => {}
    })
}
