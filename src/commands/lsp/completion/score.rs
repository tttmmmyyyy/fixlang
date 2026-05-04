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
use crate::ast::types::TypeNode;

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

/// Assign a tier to one candidate. Step 2 returns Tier 1/2/3 only:
/// any candidate in the receiver-tycon bucket gets Tier 1 (the
/// unify-based promotion to Tier 0 lands in Step 3).
pub(super) fn assign_tier(
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
