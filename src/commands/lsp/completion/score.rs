// Tier classification for dot-completion candidates and the
// `sort_text` they map to.
//
// Tiers:
//
// | 0 | candidate's receiver position fully unifies with the receiver type |
// | 1 | TyCon of receiver matches but unify failed / wasn't tried          |
// | 2 | candidate's receiver position is a type variable (wildcard)        |
// | 3 | other                                                              |
//
// Within each tier, candidates are further split by namespace
// proximity to the receiver TyCon — see `NamespaceMatch`. For a
// receiver of type `Std::Array`, a function in `Std::Array::*` ranks
// above one in `Std::*`, which ranks above one in an unrelated
// namespace.

use std::sync::Arc;
use crate::ast::name::FullName;
use crate::ast::program::Program;
use crate::ast::types::{TyCon, TypeNode};
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

/// Sub-tier within a `Tier`, derived from how close the candidate's
/// namespace is to the receiver's TyCon. Lower variants rank higher.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NamespaceMatch {
    /// Candidate lives directly in the receiver TyCon's namespace —
    /// e.g., `Std::Array::push_back` for a `Std::Array` receiver. The
    /// strongest signal that this function is a "method" of the type.
    InsideTyCon,
    /// Candidate lives in the receiver TyCon's parent namespace —
    /// e.g., `Std::push_back` for a `Std::Array` receiver. Useful for
    /// helpers grouped alongside the type at the parent level.
    InParent,
    /// Neither — a function from an unrelated namespace that happens
    /// to take this type as its receiver position.
    Unrelated,
}

impl NamespaceMatch {
    /// Single-letter encoding for `sort_text`. Order: `'a' < 'b' < 'c'`
    /// matches the variant ordering above.
    fn as_letter(self) -> char {
        match self {
            NamespaceMatch::InsideTyCon => 'a',
            NamespaceMatch::InParent => 'b',
            NamespaceMatch::Unrelated => 'c',
        }
    }
}

/// Classify a candidate's namespace against the receiver TyCon. The
/// receiver TyCon's full name (`Std::Array`) plays the role of a
/// virtual "method namespace" the user expects functions to live in.
pub(super) fn namespace_match(
    receiver_tc: Option<&TyCon>,
    candidate: &FullName,
) -> NamespaceMatch {
    let Some(tc) = receiver_tc else {
        return NamespaceMatch::Unrelated;
    };
    // Receiver TyCon as a namespace: the TyCon's own namespace path
    // followed by the type name. e.g. for `Std::Array`,
    // `[Std] + ["Array"] = [Std, Array]`.
    if candidate.namespace == tc.name.to_namespace() {
        return NamespaceMatch::InsideTyCon;
    }
    if candidate.namespace == tc.name.namespace {
        return NamespaceMatch::InParent;
    }
    NamespaceMatch::Unrelated
}

/// Format a completion item's `sort_text` so the LSP client orders by
/// tier first, then by namespace match within the tier, and finally
/// alphabetically by name.
///
/// Encoding: `<tier_digit><ns_letter>_<name>` — e.g. `0a_push_back`,
/// `0c_build`. Strings sort lexicographically; `0a_…` < `0c_…` puts
/// in-namespace candidates above unrelated ones at the same tier.
pub(super) fn sort_text_for(tier: Tier, ns_match: NamespaceMatch, name: &FullName) -> String {
    format!(
        "{}{}_{}",
        tier.as_digit(),
        ns_match.as_letter(),
        name.to_string()
    )
}

/// `sort_text` for completion items that can never satisfy a dot
/// expression (types / traits / assoc types). They land at the bottom
/// of the dot-context list — `Tier::Three` + `NamespaceMatch::Unrelated`
/// — so they don't outrank function candidates but are still present
/// when the user is working in a misclassified context.
pub(super) fn dot_context_low_priority_sort_text(name: &FullName) -> String {
    sort_text_for(Tier::Three, NamespaceMatch::Unrelated, name)
}

/// Assign a tier to one candidate. Walks the bucket index for the
/// cheap shape match (Tier 1/2/3); for candidates in either the
/// receiver-tycon bucket (Tier 1) or the wildcard bucket (Tier 2),
/// additionally tries to unify the candidate's receiver position with
/// `receiver_type` *and* check that its trait constraints stay
/// satisfiable. A successful probe promotes the candidate to Tier 0.
///
/// Promoting Tier 2 candidates matters when the receiver is, e.g.,
/// an opaque iterator type returned by `Std::Iterator::range`: the
/// iterator methods all have a tyvar receiver constrained by
/// `Iterator`, so they sit in the wildcard bucket, but they should
/// still rank above unrelated trait methods (`Add::add`,
/// `ToString::to_string`, …) whose constraints (`Add a`, `ToString a`)
/// don't hold for the iterator.
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
    if bucket_tier == Tier::Three {
        return Tier::Three;
    }
    if try_unify_receiver(name, receiver_type, program, tc_template).is_ok() {
        Tier::Zero
    } else {
        bucket_tier
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
/// source argument of its scheme) with `receiver_type`, and verify
/// that the candidate's trait constraints stay satisfiable under that
/// binding.
///
/// `n = 0` is hard-coded: the candidate's `S_{m-1}` must unify with
/// the typed receiver. After unification, `reduce_predicates` is run
/// so that constraints like `[a : Add] a -> a -> a` fail when no
/// `Add` instance exists for the concrete receiver — otherwise every
/// trait method would pass the probe (since their tyvar receiver
/// unifies with anything) and Tier 0 would lose discriminating
/// power.
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
    /// Collapses the two-level `Result<Result<T, UnificationErr>, Errors>`
    /// returned by `extract_others` down to `Result<(), ()>`: any failure
    /// (unification mismatch or unsatisfiable predicate) means the
    /// candidate doesn't fit the receiver.
    fn flatten<T>(r: Result<T, UnifOrOtherErr>) -> Result<(), ()> {
        UnifOrOtherErr::extract_others(r)
            .map_err(|_| ())?
            .map(|_| ())
            .map_err(|_| ())
    }
    flatten(tc.unify(recv_pos, receiver_type))?;
    flatten(tc.reduce_predicates())
}
