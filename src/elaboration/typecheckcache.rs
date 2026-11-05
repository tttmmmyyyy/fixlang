use crate::{ast::name::FullName, ast::program::TypedExpr, ast::types::Scheme};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex, MutexGuard},
};

/// A type-check cache held by the threads that check a program together.
pub type SharedTypeCheckCache = Arc<MemoryCache>;

/// The entity a cache entry belongs to: a global value, together with the type it was checked
/// against. A cache that lets two entities meet in one entry hands one of them the typed
/// expression of the other, and the read installs it without checking it.
pub type EntityIdentity = String;

/// The hash of the sources the entity depends on, which tells the entries of one entity apart.
type VersionHash = String;

/// A name and a printed type each occupy one line, so joining the two with a newline gives a
/// distinct string to every distinct pair.
pub fn entity_identity(name: &FullName, type_: &Arc<Scheme>) -> EntityIdentity {
    let name = name.to_string();
    let type_ = type_.to_string_normalize();
    assert!(
        !name.contains('\n') && !type_.contains('\n'),
        "a component of a cache key spans two lines: name \"{}\", type \"{}\"",
        name,
        type_
    );
    format!("{}\n{}", name, type_)
}

/// How many versions of one entity `MemoryCache` keeps. Storing a further version drops the one
/// stored longest ago.
const CACHE_GENERATION: u64 = 3;

/// The stored expressions, grouped by the entity they belong to. Within a group the version stored
/// most recently comes first, and at most `CACHE_GENERATION` versions are held.
type CacheEntries = BTreeMap<EntityIdentity, VecDeque<(VersionHash, TypedExpr)>>;

/// A cache that holds its entries in memory, so they last as long as the process that filled it.
pub struct MemoryCache {
    /// The entries, behind a lock, since the threads that check a program together share them.
    data: Mutex<CacheEntries>,
}

impl MemoryCache {
    /// Creates a cache holding no entries.
    pub fn new() -> Self {
        MemoryCache {
            data: Mutex::new(CacheEntries::default()),
        }
    }

    /// Stores `expr` as the result of checking the global value `name` against `type_`, under the
    /// hash of the sources that value depends on. It goes in front of the entity's versions, and
    /// the versions stored longest ago are dropped to stay within `CACHE_GENERATION`.
    pub fn save(&self, expr: &TypedExpr, name: &FullName, type_: &Arc<Scheme>, version_hash: &str) {
        let entity_id = entity_identity(name, type_);
        let version_hash = version_hash.to_string();
        let mut entries = self.lock_data();
        let versions = entries.entry(entity_id).or_insert_with(|| VecDeque::new());
        // If the cache is full, remove the oldest entry.
        while versions.len() >= CACHE_GENERATION as usize {
            versions.pop_back();
        }
        versions.push_front((version_hash, expr.clone()));
    }

    /// The expression stored for the global value `name` checked against `type_` under
    /// `version_hash`.
    pub fn load(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr> {
        let entity_id = entity_identity(name, type_);
        let entries = self.lock_data();
        let versions = entries.get(&entity_id)?;
        let expr = versions
            .iter()
            .find(|(hash, _)| hash == version_hash)?
            .1
            .clone();
        Some(expr)
    }

    /// The entries, locked.
    ///
    /// A thread that panics while it holds the lock poisons it, and the entries are whole when
    /// that happens: the lock is held over the map alone, so a caller that goes on with them reads
    /// what the panicking thread had already stored. The language server type-checks a program the
    /// compiler can panic on, and keeps the cache across such a run.
    fn lock_data(&self) -> MutexGuard<'_, CacheEntries> {
        self.data
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::{entity_identity, MemoryCache};
    use crate::{
        ast::expr::expr_var, ast::name::FullName, ast::program::TypedExpr, ast::types::Scheme,
        fixstd::builtin::make_i64_ty,
    };
    use std::panic::{catch_unwind, AssertUnwindSafe};

    /// The namespace a value is written under is part of the entity its entry belongs to. Two
    /// values of one module, each under a namespace of its own, agree on their remaining name, on
    /// their type, and on the sources they are checked from, so the namespace alone tells their
    /// entries apart. Both caches read the entity through `entity_identity`, so pinning it here
    /// reaches the entries held in memory as well as those of the files of `TYPED_MODULES_PATH`.
    #[test]
    fn entities_of_names_differing_only_in_namespace_stay_apart() {
        let scheme = Scheme::from_type(make_i64_ty());
        let in_a = FullName::from_strs(&["Main", "A"], "answer");
        let in_b = FullName::from_strs(&["Main", "B"], "answer");

        assert_ne!(
            entity_identity(&in_a, &scheme),
            entity_identity(&in_b, &scheme),
            "two values whose names differ in the namespace alone meet in one entry",
        );
    }

    /// The entries answer after a thread panicked while it held them, and hold what was stored
    /// before the panic.
    ///
    /// The language server type-checks a program the compiler answers with a panic and goes on to
    /// the next program with the same cache, so a lock that answered a poisoning by panicking would
    /// turn one such program into a session whose every later run fails at the first entry it
    /// reaches.
    #[test]
    fn entries_answer_after_a_thread_panicked_holding_them() {
        let cache = MemoryCache::new();
        let name = FullName::from_strs(&["Main"], "answer");
        let scheme = Scheme::from_type(make_i64_ty());
        let expr = TypedExpr::from_expr(expr_var(name.clone(), None));
        cache.save(&expr, &name, &scheme, "0");

        let held = catch_unwind(AssertUnwindSafe(|| {
            let _entries = cache.lock_data();
            panic!("a thread that panics while it holds the entries");
        }));
        assert!(held.is_err(), "the panic is what poisons the lock");

        assert!(
            cache.load(&name, &scheme, "0").is_some(),
            "the entry stored before the panic is expected to answer",
        );
        cache.save(&expr, &name, &scheme, "1");
        assert!(
            cache.load(&name, &scheme, "1").is_some(),
            "an entry stored after the panic is expected to answer",
        );
    }
}
