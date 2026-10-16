// The names of the global functions the passes mint, and what they are minted against.

use crate::{
    ast::{name::FullName, program::Symbol},
    misc::{Map, Set},
};
use std::sync::Arc;

/// The names a walk over one symbol must keep new global names apart from: the globals of the
/// program as the walk found them, shared by the walks over every symbol, and the names this walk
/// has minted.
///
/// A minted name is the walk's symbol, then a suffix whose last character is not a digit, then a
/// counter. So the digits at its end are the counter, and what precedes the suffix is the symbol.
/// Names minted from two different symbols therefore differ, and each walk checks its names against
/// the program's globals alone, without seeing what the other walks minted.
pub struct GlobalNames {
    /// The globals of the program as the walks found them.
    existing: Arc<Set<FullName>>,
    /// The names minted through this value.
    minted: Set<FullName>,
}

impl GlobalNames {
    /// The names to keep apart from, for a walk over a program whose globals are `existing`.
    pub fn new(existing: Arc<Set<FullName>>) -> Self {
        GlobalNames {
            existing,
            minted: Set::default(),
        }
    }

    /// A fresh global name for a lifted function, made of `base` (the symbol being processed),
    /// `suffix` and `counter`. `counter` advances and the chosen name joins the minted ones, so
    /// repeated calls give names that differ from the existing globals and from each other.
    ///
    /// # Examples
    /// `fresh(Main::f, "#lam", 0)` gives `Main::f#lam0` and leaves the counter at 1.
    pub fn fresh(&mut self, base: &FullName, suffix: &str, counter: &mut u32) -> FullName {
        assert!(
            suffix.ends_with(|c: char| !c.is_ascii_digit()),
            "the suffix `{}` of a minted name ends in a digit, which runs it into the counter",
            suffix
        );
        loop {
            let mut name = base.clone();
            *name.name_as_mut() += &format!("{}{}", suffix, *counter);
            *counter += 1;
            if !self.existing.contains(&name) && self.minted.insert(name.clone()) {
                return name;
            }
        }
    }
}

/// The names of the globals `symbols` defines, for the walks over them to share.
pub fn names_of_globals(symbols: &Map<FullName, Symbol>) -> Arc<Set<FullName>> {
    Arc::new(symbols.keys().cloned().collect())
}

/// Adds `sym`, a function named by `GlobalNames::fresh`, to `symbols`, which hold the globals the
/// names were minted against and the functions minted so far. Panics if `symbols` already holds a
/// global of that name.
pub fn insert_minted_symbol(symbols: &mut Map<FullName, Symbol>, sym: Symbol) {
    let name = sym.name.clone();
    assert!(
        symbols.insert(name.clone(), sym).is_none(),
        "the minted function `{}` is named like another global",
        name.to_string()
    );
}
