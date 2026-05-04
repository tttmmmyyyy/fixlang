// Bucket index over the program's globals, keyed by the top-level
// `TyCon` of the receiver position (the last curried source argument
// of each global's scheme). Used by `score.rs` to assign Tier 1/2/3
// cheaply: looking up "globals whose receiver could fit this TyCon"
// is a single hashmap probe.
//
// See `logs/lsp-completion-type-filter.20260503/plan.md` §A.5.1.

use std::sync::Arc;

use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::ast::types::{TyCon, TypeNode};
use crate::misc::{Map, Set};

/// Buckets: the FullName of every global value whose receiver
/// position can plausibly be `tc`, indexed by `tc`. Globals whose
/// receiver position is itself a type variable (and thus unifies with
/// any concrete type) live in `wildcard` instead.
///
/// Membership of `by_receiver_tycon[tc]` says "the receiver position's
/// head TyCon matches `tc`". It does **not** assert full unifiability:
/// `Array I64` and `Array String` both bucket under `Array`, and the
/// final unify step (Step 3) is what decides Tier 0 vs Tier 1.
pub(super) struct CompletionIndex {
    pub(super) by_receiver_tycon: Map<TyCon, Set<FullName>>,
    pub(super) wildcard: Set<FullName>,
}

impl CompletionIndex {
    /// Walk every global value and assign it to a bucket.
    ///
    /// A global without a function-typed scheme can't be dot-called
    /// (no receiver argument exists), so it is skipped entirely —
    /// never appears in either bucket.
    pub(super) fn build(program: &Program) -> Self {
        let mut by_receiver_tycon: Map<TyCon, Set<FullName>> = Map::default();
        let mut wildcard: Set<FullName> = Set::default();

        for (name, gv) in &program.global_values {
            let Some(receiver_pos) = receiver_source_type(&gv.scm.ty) else {
                continue;
            };
            match receiver_pos.toplevel_tycon() {
                Some(tc) => {
                    by_receiver_tycon
                        .entry(tc.as_ref().clone())
                        .or_insert_with(Set::default)
                        .insert(name.clone());
                }
                None => {
                    wildcard.insert(name.clone());
                }
            }
        }

        CompletionIndex {
            by_receiver_tycon,
            wildcard,
        }
    }
}

/// Return the type at the receiver position of `ty` — i.e. the last
/// curried source argument — or `None` if `ty` is not a function type
/// at all.
///
/// Walks `ty` as long as it's a function (closure or function pointer)
/// to find the deepest "last source", which corresponds to the last
/// argument applied in dot syntax (`a.f(b1, b2)` applies `f` to `b1`,
/// then `b2`, then `a`, so the receiver lands on `f`'s last source).
fn receiver_source_type(ty: &Arc<TypeNode>) -> Option<Arc<TypeNode>> {
    if !(ty.is_funptr() || ty.is_closure()) {
        return None;
    }
    let srcs = ty.get_lambda_srcs();
    srcs.into_iter().last()
}
