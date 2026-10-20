//! The clone bookkeeping shared by the specializing passes of the RC IR.
//!
//! A specializing pass clones a function once per key its callers reach it with, so that each clone
//! knows something about its inputs and can rewrite its body under that knowledge. Which key type
//! that is, how a call computes its callee's key, and what the rewriting does are the pass's own;
//! what every such pass does the same way is here — minting and memoizing a clone's name, driving
//! the worklist so each `(function, key)` is materialized once, and giving a fresh clone fresh local
//! names without losing its ownership annotation.
//!
//! A clone's name is its function's name followed by the pass's tag and the key's spelling
//! (`CloneKey::spelling`), and the clone's local names take the same suffix. The name depends on
//! nothing but the function and the key, so editing one function leaves the clones of every other
//! function named as they were.
//!
//! The clone keyed on the least informative inputs keeps the original function's name and is called
//! the *canonical* one. Every function keeps its canonical version, since a pass that routes calls
//! reads no root set and so takes any function to be callable; specialization only adds the more
//! specific clones the call sites reach. `dead_code_elim::eliminate_unreachable` then drops the
//! versions nothing calls.

use crate::misc::{Map, Set};
use crate::rc_ir::ast::{FuncRef, RcExprNode, RcFunc, RcProgram, RcVar};
use crate::rc_ir::rename::{copy_suffix, fresh_rename_function};
use std::collections::VecDeque;
use std::hash::Hash;

/// How many clones one function may have, past its canonical version.
///
/// A key space is a product: one component per input, and one value per boxed leaf of each. A
/// function that hands its arguments on in a different arrangement each time reaches the whole
/// product, so a body of a few lines can name thousands of clones and a caller chain multiplies
/// them. Nothing in a key's meaning bounds this, and a pass whose lattice happens to collapse — as
/// uniqueness does, since duplicating a reference costs a value its claim to be the only one — is
/// bounded only by luck.
///
/// So the count is bounded outright. Past it, a call routes to the canonical version, which is keyed
/// on the least informative inputs and therefore carries no annotation at all: exceeding the budget
/// costs precision and can never cost soundness, whatever the bound is set to. The value is chosen
/// against the corpus, where the most-cloned function has four.
const MAX_CLONES_PER_FUNCTION: usize = 16;

/// A key a specializing pass clones a function on, which spells itself into the clone's name.
pub trait CloneKey: Clone + Eq + Hash {
    /// The key written as letters and digits. Two different keys of one function spell differently.
    fn spelling(&self) -> String;
}

/// The clones a specializing pass has named and has yet to materialize.
pub struct CloneRegistry<K> {
    /// The canonical key of each function: the key on the least informative inputs, whose clone
    /// keeps the function's name.
    canonical_keys: Map<FuncRef, K>,
    /// The fresh name of each non-canonical clone `(function, key)`.
    clone_names: Map<(FuncRef, K), FuncRef>,
    /// Every fresh name in `clone_names`, so that `request` catches a second key that spells a name
    /// already given out.
    minted_names: Set<FuncRef>,
    /// How many clones each function has been given, against `MAX_CLONES_PER_FUNCTION`.
    clone_counts: Map<FuncRef, usize>,
    /// Every `(function, key)` already enqueued, so each is materialized once.
    requested: Set<(FuncRef, K)>,
    /// The requested clones not yet handed out for materialization.
    worklist: VecDeque<(FuncRef, K)>,
    /// The letter marking this pass's fresh names apart from another pass's.
    tag: &'static str,
}

impl<K: CloneKey> CloneRegistry<K> {
    /// An empty registry whose fresh names carry `pass_tag`, which tells this pass's names apart
    /// from those another specializing pass minted.
    pub fn new(pass_tag: &'static str) -> CloneRegistry<K> {
        CloneRegistry {
            canonical_keys: Map::default(),
            clone_names: Map::default(),
            minted_names: Set::default(),
            clone_counts: Map::default(),
            requested: Set::default(),
            worklist: VecDeque::new(),
            tag: pass_tag,
        }
    }

    /// Record `key` as the canonical key of `fref` and enqueue the canonical version. A pass calls
    /// this for every function of its program before it requests any clone, since every function
    /// keeps its canonical version.
    pub fn keep_canonical(&mut self, fref: &FuncRef, key: K) {
        let previous = self.canonical_keys.insert(fref.clone(), key.clone());
        assert!(
            previous.is_none(),
            "`{}` is given a canonical key twice",
            fref.name.to_string()
        );
        self.enqueue(fref, key);
    }

    /// Whether `key` is the canonical key of `fref`, which `keep_canonical` recorded.
    fn is_canonical(&self, fref: &FuncRef, key: &K) -> bool {
        let canonical = self.canonical_keys.get(fref).unwrap_or_else(|| {
            panic!(
                "`{}` was given no canonical key before its clones were asked for",
                fref.name.to_string()
            )
        });
        canonical == key
    }

    /// The output name of the clone `(fref, key)`, enqueuing it for materialization the first time
    /// it is asked for. `keep_canonical` must have recorded `fref` first. The canonical clone keeps
    /// the original name; every other key gets a fresh
    /// name, minted once per key, until the function has as many as `MAX_CLONES_PER_FUNCTION`
    /// allows — past that the answer is the canonical name, which is always available and proves
    /// nothing, so the call it routes keeps every dispatch it had.
    pub fn request(&mut self, fref: &FuncRef, key: K) -> FuncRef {
        if self.is_canonical(fref, &key) {
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
        let mut name = fref.name.clone();
        name.name.push_str(&self.suffix(&key));
        let nref = FuncRef { name };
        assert!(
            self.minted_names.insert(nref.clone()),
            "two keys of `{}` spell the same clone name `{}`",
            fref.name.to_string(),
            nref.name.to_string()
        );
        self.clone_names
            .insert((fref.clone(), key.clone()), nref.clone());
        self.enqueue(fref, key);
        nref
    }

    /// The suffix the clone keyed on `key` adds to its function's name and to each of its local names.
    fn suffix(&self, key: &K) -> String {
        copy_suffix(self.tag, &key.spelling())
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

    /// Assemble the clone of `func` keyed on `key`, which `pop` handed out, from its rewritten
    /// `body`. The canonical clone is the original function carrying the new body; a fresh clone
    /// additionally gets fresh local names, so that its names do not collide with the original's.
    pub fn finish_clone(&self, func: &RcFunc, key: &K, body: RcExprNode) -> RcFunc {
        if self.is_canonical(&func.name, key) {
            return RcFunc {
                body,
                ..func.clone()
            };
        }
        // `pop` hands out the canonical keys and the keys `request` gave a fresh name; a key past
        // the clone budget is never enqueued.
        let name = self
            .clone_names
            .get(&(func.name.clone(), key.clone()))
            .unwrap_or_else(|| {
                panic!(
                    "`{}` is finished under a key `request` gave no fresh name",
                    func.name.name.to_string()
                )
            });
        let (params, capture, body, rename) =
            fresh_rename_function(&func.params, &func.capture, &body, &self.suffix(key));
        RcFunc {
            name: name.clone(),
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

#[cfg(test)]
mod tests {
    use super::{CloneKey, CloneRegistry};
    use crate::rc_ir::ast::FuncRef;
    use crate::rc_ir::test_program::global_name;

    /// A key of one boolean fact per parameter, spelled `T` or `F` per parameter.
    impl CloneKey for Vec<bool> {
        fn spelling(&self) -> String {
            self.iter()
                .map(|fact| if *fact { 'T' } else { 'F' })
                .collect()
        }
    }

    /// A reference to the function named `name` in the namespace `Main`.
    fn fref(name: &str) -> FuncRef {
        FuncRef {
            name: global_name(name),
        }
    }

    /// The name of a clone is decided by its function and its key, whatever the registry named
    /// before it — so the clones of one function keep their names when an edit elsewhere in the
    /// program asks for more clones, or for fewer, ahead of them.
    #[test]
    fn test_a_clone_is_named_by_its_function_and_key_alone() {
        let mut alone: CloneRegistry<Vec<bool>> = CloneRegistry::new("t");
        alone.keep_canonical(&fref("f"), vec![false, false]);
        let named_alone = alone.request(&fref("f"), vec![true, false]);

        let mut after_others: CloneRegistry<Vec<bool>> = CloneRegistry::new("t");
        after_others.keep_canonical(&fref("g"), vec![false]);
        after_others.keep_canonical(&fref("f"), vec![false, false]);
        after_others.request(&fref("g"), vec![true]);
        after_others.request(&fref("f"), vec![false, true]);
        let named_after_others = after_others.request(&fref("f"), vec![true, false]);

        assert_eq!(
            named_alone.name.to_string(),
            named_after_others.name.to_string()
        );
        assert_eq!(
            named_alone.name.to_string(),
            global_name("f#tTF").to_string()
        );
    }
}
