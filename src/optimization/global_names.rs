// The names of the global functions the passes mint, and what they are minted against.

use crate::{
    ast::{name::FullName, program::Symbol},
    misc::{Map, Set},
};
use std::sync::Arc;

/// The names a walk over one symbol gives the global functions it creates, kept apart from the
/// globals of the program as the walk found them, which the walks over every symbol share.
///
/// A minted name is the walk's symbol, then the suffix, whose last character is not a digit, then a
/// counter. So the digits at its end are the counter, and what precedes the suffix is the symbol.
/// The counter only grows, so the names one walk mints differ from one another; names minted from
/// two different symbols differ as well, so each walk checks its names against the program's
/// globals alone, without seeing what the other walks minted.
pub struct GlobalNames {
    /// The globals of the program as the walks found them.
    existing: Arc<Set<FullName>>,
    /// The symbol the walk is over, which every minted name begins with.
    symbol: FullName,
    /// What follows the symbol in every minted name, before the counter.
    suffix: &'static str,
    /// The counter the next name is tried with.
    counter: u32,
}

impl GlobalNames {
    /// The names a walk over `symbol` mints, followed by `suffix`, in a program whose globals are
    /// `existing`.
    pub fn new(existing: Arc<Set<FullName>>, symbol: FullName, suffix: &'static str) -> Self {
        assert!(
            suffix.ends_with(|c: char| !c.is_ascii_digit()),
            "the suffix `{}` of a minted name ends in a digit, which runs it into the counter",
            suffix
        );
        GlobalNames {
            existing,
            symbol,
            suffix,
            counter: 0,
        }
    }

    /// A name for a new global function, which differs from the existing globals and from every
    /// name this value minted before it.
    ///
    /// # Examples
    /// The first two names minted for `Main::f` with the suffix `#lam` are `Main::f#lam0` and
    /// `Main::f#lam1`.
    pub fn fresh(&mut self) -> FullName {
        loop {
            let mut name = self.symbol.clone();
            *name.name_as_mut() += &format!("{}{}", self.suffix, self.counter);
            self.counter += 1;
            if !self.existing.contains(&name) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The first `n` names `GlobalNames::fresh` mints for `symbol` with the suffix `#lam`, in a
    /// program whose globals are `existing`.
    fn names_minted(existing: Set<FullName>, symbol: &FullName, n: usize) -> Vec<FullName> {
        let mut names = GlobalNames::new(Arc::new(existing), symbol.clone(), "#lam");
        (0..n).map(|_| names.fresh()).collect()
    }

    /// A name `fresh` would otherwise mint is passed over when a global already has it, and the
    /// names minted after it stay distinct from one another.
    #[test]
    fn fresh_passes_over_the_existing_globals() {
        let symbol = FullName::from_strs(&["Main"], "f");
        let existing: Set<FullName> = names_minted(Set::default(), &symbol, 2)
            .into_iter()
            .collect();
        let minted = names_minted(existing.clone(), &symbol, 3);
        for name in &minted {
            assert!(
                !existing.contains(name),
                "`{}` is minted although a global has it",
                name.to_string()
            );
        }
        let distinct: Set<FullName> = minted.iter().cloned().collect();
        assert_eq!(distinct.len(), minted.len());
    }

    /// Two walks over different symbols mint different names, though neither sees the other's, even
    /// where one symbol's name begins with the other's.
    #[test]
    fn names_minted_from_different_symbols_differ() {
        let symbols = [
            FullName::from_strs(&["Main"], "f"),
            FullName::from_strs(&["Main"], "f1"),
            FullName::from_strs(&["Main"], "f#lam1"),
        ];
        let mut minted: Set<FullName> = Set::default();
        for symbol in &symbols {
            for name in names_minted(Set::default(), symbol, 20) {
                assert!(
                    minted.insert(name.clone()),
                    "`{}` is minted from two symbols",
                    name.to_string()
                );
            }
        }
    }
}
