// Tier classification for dot-completion candidates and the
// `sort_text` they map to.
//
// Tiers:
//
// | 0 | candidate's receiver position fully unifies with the receiver type |
// | 1 | TyCon of receiver matches but unify failed / wasn't tried          |
// | 2 | candidate's receiver position is a type variable (wildcard)        |
// | 3 | other                                                              |

use std::sync::Arc;
use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::ast::types::TypeNode;
use crate::elaboration::typecheck::{ConstraintInstantiationMode, TypeCheckContext, UnifOrOtherErr};
use super::index::CompletionIndex;

/// Bucket a dot-completion candidate falls into; lower values rank
/// higher in the completion list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tier {
    /// Receiver position unifies with the typed receiver.
    Zero = 0,
    /// Receiver position's head TyCon matches but unify wasn't run or
    /// didn't succeed.
    One = 1,
    /// Receiver position is a type variable (matches anything).
    Two = 2,
    /// None of the above.
    Three = 3,
}

impl Tier {
    /// Numeric value of the tier as a single digit, suitable for
    /// embedding in `sort_text`.
    pub(super) fn as_digit(self) -> u8 {
        self as u8
    }
}

/// Format a completion item's `sort_text` so the LSP client orders by
/// tier first, then alphabetically by name within each tier.
///
/// Single-digit tier is enough — strings sort lexicographically and
/// `0_…` < `1_…` < `2_…` < `3_…` is the natural collation.
pub(super) fn sort_text_for(tier: Tier, name: &FullName) -> String {
    format!("{}_{}", tier.as_digit(), name.to_string())
}

/// Assign a tier to one candidate. Walks the bucket index for the
/// cheap shape match (Tier 1/2/3); for candidates already in the
/// receiver-tycon bucket, additionally tries to unify the candidate's
/// receiver position with `receiver_type` and promotes to Tier 0 on
/// success.
///
/// `tc_template` is a typechecker created from the snapshot `program`;
/// it gets cloned per unify attempt so substitutions accumulated by
/// one candidate don't leak into the next.
pub(super) fn assign_tier(
    name: &FullName,
    index: &CompletionIndex,
    receiver_type: &Arc<TypeNode>,
    program: &Program,
    tc_template: &TypeCheckContext,
) -> Tier {
    let bucket_tier = bucket_tier(name, index, receiver_type);
    if bucket_tier != Tier::One {
        return bucket_tier;
    }
    if try_unify_receiver(name, receiver_type, program, tc_template).is_ok() {
        Tier::Zero
    } else {
        Tier::One
    }
}

/// Bucket-only tier assignment: the cheap Tier 1/2/3 classification
/// without the unify-based Tier 0 promotion. Used as a fallback when
/// the unify step's prerequisites (a scratch `Configuration` for
/// building a `TypeCheckContext`) couldn't be set up.
pub(super) fn assign_tier_no_unify(
    name: &FullName,
    index: &CompletionIndex,
    receiver_type: &Arc<TypeNode>,
) -> Tier {
    bucket_tier(name, index, receiver_type)
}

/// Cheap shape-only tier assignment: probe `index` for `name` against
/// `receiver_type`'s head TyCon, returning Tier 1 / 2 / 3 without any
/// unify work.
fn bucket_tier(
    name: &FullName,
    index: &CompletionIndex,
    receiver_type: &Arc<TypeNode>,
) -> Tier {
    let receiver_tc = receiver_type.toplevel_tycon();

    let in_tycon_bucket = receiver_tc
        .as_ref()
        .and_then(|tc| index.by_receiver_tycon.get(tc.as_ref()))
        .map(|b| b.contains(name))
        .unwrap_or(false);
    if in_tycon_bucket {
        return Tier::One;
    }
    if index.wildcard.contains(name) {
        return Tier::Two;
    }
    Tier::Three
}

/// Try to unify the candidate's receiver position (the last curried
/// source argument of its scheme) with `receiver_type`.
///
/// `n = 0` is hard-coded: the candidate's `S_{m-1}` must unify with
/// the typed receiver. Trait constraints from the scheme's
/// `predicates` are added to the typechecker but their satisfiability
/// is not enforced beyond what `unify` does automatically.
fn try_unify_receiver(
    name: &FullName,
    receiver_type: &Arc<TypeNode>,
    program: &Program,
    tc_template: &TypeCheckContext,
) -> Result<(), ()> {
    let gv = program.global_values.get(name).ok_or(())?;
    let mut tc = tc_template.clone();
    let inst_ty = tc
        .instantiate_scheme(&gv.scm, ConstraintInstantiationMode::Require)
        .map_err(|_| ())?;
    let (srcs, _) = inst_ty.collect_app_src(usize::MAX);
    let recv_pos = srcs.last().ok_or(())?;
    UnifOrOtherErr::extract_others(tc.unify(recv_pos, receiver_type))
        .map_err(|_| ())?
        .map_err(|_| ())
}
