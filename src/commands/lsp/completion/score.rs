// Tier classification for dot-completion candidates and the
// `sort_text` they map to.
//
// Tiers (plan §A.2):
//
// | 0 | candidate's receiver position fully unifies with the receiver type |
// | 1 | TyCon of receiver matches but unify failed / wasn't tried |
// | 2 | candidate's receiver position is a type variable (wildcard)        |
// | 3 | other                                                              |
//
// Step 2 lands Tier 1/2/3. Tier 0 (the unify-based promotion) lands in
// Step 3.

use std::sync::Arc;

use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::ast::types::TypeNode;
use crate::elaboration::typecheck::{ConstraintInstantiationMode, TypeCheckContext, UnifOrOtherErr};

use super::index::CompletionIndex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tier {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

impl Tier {
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
/// **Step 1 scope** — `n = 0` is hard-coded: the candidate's
/// `S_{m-1}` must unify with the typed receiver. Trait constraints
/// from the scheme's `predicates` are added to the typechecker but
/// their satisfiability is not enforced beyond what `unify` does
/// automatically (per plan §A.5.3 "trait constraint は MVP では無視").
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
    let srcs = collect_lambda_srcs_iter(&inst_ty);
    if srcs.is_empty() {
        return Err(());
    }
    let recv_pos = srcs.last().ok_or(())?;
    UnifOrOtherErr::extract_others(tc.unify(recv_pos, receiver_type))
        .map_err(|_| ())?
        .map_err(|_| ())
}

/// Walk a function-typed `TypeNode` peeling off its source arguments
/// in curry order. Stops as soon as the type ceases to be a closure
/// or function pointer. Empty result means `ty` wasn't a function.
fn collect_lambda_srcs_iter(ty: &Arc<TypeNode>) -> Vec<Arc<TypeNode>> {
    let mut out = vec![];
    let mut cur = ty.clone();
    while cur.is_funptr() || cur.is_closure() {
        let srcs = cur.get_lambda_srcs();
        let dst = cur.get_lambda_dst();
        out.extend(srcs);
        cur = dst;
    }
    out
}
