// The names of the global functions the passes mint, and what they are minted against.

use crate::{
    ast::{name::FullName, program::Symbol},
    misc::{Map, Set},
};
use std::sync::Arc;

// The global names a walk over one symbol mints new ones against: the globals of the program as the
// walk found them, shared by the walks over every symbol, and the names this walk has minted.
//
// A minted name is the walk's symbol followed by a suffix ending in a letter and a counter, so the
// digits at its end are the counter and what precedes the suffix is the symbol. Names minted from
// two different symbols therefore differ, and the walks over different symbols mint against the
// program's globals alone, without seeing what one another minted.
pub struct GlobalNames {
    // The globals of the program as the walks found them.
    existing: Arc<Set<FullName>>,
    // The names minted through this value.
    minted: Set<FullName>,
}

impl GlobalNames {
    // The names of the globals `symbols` defines, which the walks over them share as what they mint
    // against.
    pub fn existing_in(symbols: &Map<FullName, Symbol>) -> Arc<Set<FullName>> {
        Arc::new(symbols.keys().cloned().collect())
    }

    // Add `sym`, a function named by `fresh`, to `symbols`, which hold the globals the names were
    // minted against and the functions minted from them so far.
    pub fn insert_minted(symbols: &mut Map<FullName, Symbol>, sym: Symbol) {
        let name = sym.name.clone();
        assert!(
            symbols.insert(name.clone(), sym).is_none(),
            "the minted function `{}` is named like another global",
            name.to_string()
        );
    }

    // The names to mint against for a walk over a program whose globals are `existing`.
    pub fn new(existing: Arc<Set<FullName>>) -> Self {
        GlobalNames {
            existing,
            minted: Set::default(),
        }
    }

    // A fresh global name for a lifted function, derived from `base` (the symbol being processed)
    // plus `suffix` and `counter`. `counter` advances and the chosen name joins the minted ones, so
    // repeated calls stay collision-free against existing globals and against each other.
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
