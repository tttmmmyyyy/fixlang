//! The clone bookkeeping shared by the specializing passes of the RC IR.
//!
//! A specializing pass clones a function once per key its callers reach it with, so that each clone
//! knows something about its inputs and can rewrite its body under that knowledge. Which key type
//! that is, how a call computes its callee's key, and what the rewriting does are the pass's own;
//! what every such pass does the same way is here — minting and memoizing a clone's name, driving
//! the worklist so each `(function, key)` is materialized once, and giving a fresh clone fresh local
//! names without losing its ownership annotation.
//!
//! The clone keyed on the least informative inputs keeps the original function's name and is called
//! the *canonical* one. Every function keeps its canonical version, since a pass that routes calls
//! reads no root set and so takes any function to be callable; specialization only adds the more
//! specific clones the call sites reach. `dce::eliminate_unreachable` then drops the versions
//! nothing calls.

use crate::misc::{Map, Set};
use crate::rc_ir::ast::{FuncRef, RcExprNode, RcFunc, RcProgram, RcVar};
use crate::rc_ir::rename::fresh_rename_function;
use std::collections::VecDeque;
use std::hash::Hash;

/// How many clones one function may have, past its canonical version.
///
/// A key space is a product: one component per input, and one value per boxed leaf of each. A
/// function that hands its arguments on in a different arrangement each time reaches the whole
/// product, so a body of a few lines can name thousands of clones and a caller chain multiplies
/// them. Nothing in a key's meaning bounds this, and a pass whose lattice happens to collapse — as
/// uniqueness does, since duplicating a reference costs a value its claim to be the only one — is
/// bounded by luck rather than by design.
///
/// So the count is bounded outright. Past it, a call routes to the canonical version, which is keyed
/// on the least informative inputs and therefore carries no annotation at all: exceeding the budget
/// costs precision and can never cost soundness, whatever the bound is set to. The value is chosen
/// against the corpus, where the most-cloned function has four.
const MAX_CLONES_PER_FUNCTION: usize = 16;

/// The clones a specializing pass has named and has yet to materialize.
pub struct CloneRegistry<K> {
    /// The fresh name of each non-canonical clone `(function, key)`.
    clone_names: Map<(FuncRef, K), FuncRef>,
    /// How many clones each function has been given, against `MAX_CLONES_PER_FUNCTION`.
    clone_counts: Map<FuncRef, usize>,
    /// Every `(function, key)` already enqueued, so each is materialized once.
    requested: Set<(FuncRef, K)>,
    /// The requested clones not yet handed out for materialization.
    worklist: VecDeque<(FuncRef, K)>,
    /// The source of the number distinguishing one fresh name from the next, shared by the clone
    /// names and the local names a fresh clone's body is renamed to.
    fresh_name_counter: u64,
    /// The letter marking this pass's fresh names apart from another pass's.
    tag: &'static str,
}

impl<K: Clone + Eq + Hash> CloneRegistry<K> {
    /// An empty registry whose fresh names carry `pass_tag`, which tells this pass's names apart
    /// from those another specializing pass minted.
    pub fn new(pass_tag: &'static str) -> CloneRegistry<K> {
        CloneRegistry {
            clone_names: Map::default(),
            clone_counts: Map::default(),
            requested: Set::default(),
            worklist: VecDeque::new(),
            fresh_name_counter: 0,
            tag: pass_tag,
        }
    }

    /// The output name of the clone `(fref, key)`, enqueuing it for materialization the first time
    /// it is asked for. The canonical clone keeps the original name; every other key gets a fresh
    /// name, minted once per key, until the function has as many as `MAX_CLONES_PER_FUNCTION`
    /// allows — past that the answer is the canonical name, which is always available and proves
    /// nothing, so the call it routes keeps every dispatch it had.
    pub fn request(&mut self, fref: &FuncRef, key: K, is_canonical: bool) -> FuncRef {
        if is_canonical {
            self.enqueue(fref, key);
            return fref.clone();
        }
        if let Some(name) = self.clone_names.get(&(fref.clone(), key.clone())) {
            let name = name.clone();
            self.enqueue(fref, key);
            return name;
        }
        let count = self.clone_counts.entry(fref.clone()).or_insert(0);
        if *count >= MAX_CLONES_PER_FUNCTION {
            return fref.clone();
        }
        *count += 1;
        self.fresh_name_counter += 1;
        let mut name = fref.name.clone();
        name.name = format!("{}#{}{}", name.name, self.tag, self.fresh_name_counter);
        let nref = FuncRef { name };
        self.clone_names
            .insert((fref.clone(), key.clone()), nref.clone());
        self.enqueue(fref, key);
        nref
    }

    /// Enqueue `(fref, key)` for materialization, the first time it is asked for.
    fn enqueue(&mut self, fref: &FuncRef, key: K) {
        if self.requested.insert((fref.clone(), key.clone())) {
            self.worklist.push_back((fref.clone(), key));
        }
    }

    /// The next requested clone to materialize. Materializing one may request further clones, so the
    /// caller drains this until it is empty.
    pub fn pop(&mut self) -> Option<(FuncRef, K)> {
        self.worklist.pop_front()
    }

    /// Assemble the clone of `func` named `name` from its rewritten `body`. The canonical clone is
    /// the original function carrying the new body; a fresh clone additionally gets fresh local
    /// names, so that its names do not collide with the original's.
    pub fn finish_clone(&mut self, func: &RcFunc, name: FuncRef, body: RcExprNode) -> RcFunc {
        if name == func.name {
            return RcFunc {
                body,
                ..func.clone()
            };
        }
        let (params, capture, body, rename) = fresh_rename_function(
            &func.params,
            &func.capture,
            &body,
            self.tag,
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
            inline_into_callers: func.inline_into_callers,
        }
    }
}

/// The functions that reach a seed through the calls of `callees`: the least set containing every
/// seed and closed under "calls a member".
///
/// A specializing pass gates its clones on this. Whatever makes one function worth cloning — a
/// reference-counting site whose answer depends on the inputs, a uniqueness check in the body —
/// travels to its callers, because a caller cloned for its own inputs is what lets the callee be
/// called with more informative ones.
pub fn callers_of(seeds: Set<FuncRef>, callees: &Map<FuncRef, Vec<FuncRef>>) -> Set<FuncRef> {
    let mut reaching = seeds;
    loop {
        let mut changed = false;
        for (fref, cs) in callees {
            if !reaching.contains(fref) && cs.iter().any(|c| reaching.contains(c)) {
                reaching.insert(fref.clone());
                changed = true;
            }
        }
        if !changed {
            return reaching;
        }
    }
}

/// The function a direct call names, when routing that call to a clone is possible: the callee
/// resolves to a function of this program, it takes no capture (a closure is reached indirectly, so
/// it keeps its single canonical version), and it is one the pass's gate passes — a function the key
/// cannot change is the same under every key, so cloning it would only make redundant copies.
pub fn specializable_callee(
    prog: &RcProgram,
    callee: &RcVar,
    gate: &Set<FuncRef>,
) -> Option<FuncRef> {
    let cref = FuncRef {
        name: callee.name.clone(),
    };
    let g = prog.funcs.get(&cref)?;
    if g.capture.is_some() || !gate.contains(&cref) {
        return None;
    }
    Some(cref)
}
