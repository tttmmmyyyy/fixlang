use crate::{
    ast::name::FullName, ast::program::TypedExpr, ast::types::Scheme,
    constants::TYPE_CHECK_CACHE_PATH, elaboration::touch_directory, misc::warn_msg,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{Read, Write},
    panic::RefUnwindSafe,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub type SharedTypeCheckCache = Arc<dyn TypeCheckCache + Send + Sync>;

// A trait for objects which manage caching of typechecked expressions.
//
// `RefUnwindSafe` is a supertrait so that `Arc<dyn TypeCheckCache + Send + Sync>`
// satisfies `UnwindSafe`. The diagnostics thread captures one such `Arc` and
// runs the captured closure under `catch_unwind`; without this bound the
// closure isn't unwind-safe and the caller has to wrap it in
// `AssertUnwindSafe`. Both built-in impls (`FileCache`, `MemoryCache`) are
// `RefUnwindSafe` for free — `FileCache` is empty and `MemoryCache`'s only
// field is a `Mutex`, whose poisoning protocol makes it unconditionally
// `RefUnwindSafe`.
pub trait TypeCheckCache: RefUnwindSafe {
    // Saves a typechecked expression to the cache.
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    );
    // Loads a typechecked expression from the cache.
    // Returns None if the cache is not found.
    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr>;
}

// A cache implementation that stores cache in files.
pub struct FileCache {}

impl FileCache {
    pub fn new() -> Self {
        FileCache {}
    }

    // Determine the filename for a cache file.
    //
    // The digest is what identifies the entry: it is taken over the three components at once, so
    // two entries meet in one file only when all three agree. The part in front of it names the
    // value for someone reading the cache directory, and is filename-safe because it keeps only
    // alphanumeric characters; several values can wear the same one.
    fn cache_file_name(&self, name: &FullName, type_: &Arc<Scheme>, version_hash: &str) -> String {
        let name = name.to_string();
        let type_ = type_.to_string_normalize();
        // A name, a printed type and a hash each occupy one line, so joining the three with a
        // newline gives a distinct string to every distinct triple.
        let key = format!("{}\n{}\n{}", name, type_, version_hash);
        let digest = format!("{:x}", md5::compute(key));

        let readable = name.replace(|c: char| !c.is_alphanumeric(), "_");
        format!("{}_{}", readable, digest)
    }
}

impl TypeCheckCache for FileCache {
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) {
        let cache_file_name: String = self.cache_file_name(name, type_, version_hash);
        let cache_dir = touch_directory(TYPE_CHECK_CACHE_PATH);
        let cache_file = cache_dir.join(cache_file_name);
        let cache_file_str = cache_file.to_string_lossy().to_string();
        let mut cache_file = match File::create(&cache_file) {
            Err(_) => {
                warn_msg(&format!(
                    "Failed to create cache file \"{}\".",
                    cache_file_str
                ));
                return;
            }
            Ok(file) => file,
        };
        let serialized = serde_pickle::to_vec(&expr, Default::default()).unwrap();
        match cache_file.write_all(&serialized) {
            Ok(_) => {}
            Err(_) => {
                warn_msg(&format!(
                    "Failed to write cache file \"{}\".",
                    cache_file_str
                ));
            }
        }
    }

    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr> {
        let cache_file_name: String = self.cache_file_name(name, type_, version_hash);
        let cache_dir: PathBuf = touch_directory(TYPE_CHECK_CACHE_PATH);
        let cache_file = cache_dir.join(cache_file_name);
        let cache_file_str = cache_file.to_string_lossy().to_string();
        if !cache_file.exists() {
            return None;
        }
        let mut cache_file = match File::open(&cache_file) {
            Err(_) => {
                return None;
            }
            Ok(file) => file,
        };
        let mut cache_bytes = vec![];
        match cache_file.read_to_end(&mut cache_bytes) {
            Ok(_) => {}
            Err(why) => {
                warn_msg(&format!(
                    "Failed to read cache file \"{}\": {}.",
                    cache_file_str, why
                ));
                return None;
            }
        }
        let expr: TypedExpr = match serde_pickle::from_slice(&cache_bytes, Default::default()) {
            Ok(res) => res,
            Err(why) => {
                warn_msg(&format!(
                    "Failed to parse content of cache file \"{}\": {}.",
                    cache_file_str, why
                ));
                return None;
            }
        };
        Some(expr)
    }
}

type EntityIdentity = String;
type VersionHash = String;

const CACHE_GENERATION: u64 = 3;

// Memory Cache.
pub struct MemoryCache {
    data: Mutex<BTreeMap<EntityIdentity, VecDeque<(VersionHash, TypedExpr)>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        MemoryCache {
            data: Mutex::new(BTreeMap::default()),
        }
    }

    fn entity_identity(name: &FullName, type_: &Arc<Scheme>) -> EntityIdentity {
        format!("{}_{}", name.to_string(), type_.to_string_normalize())
    }
}

impl TypeCheckCache for MemoryCache {
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) {
        let mut data = self.data.lock().unwrap();
        let entity_id = MemoryCache::entity_identity(name, type_);
        let version_hash = version_hash.to_string();
        let entry = data.entry(entity_id).or_insert_with(|| VecDeque::new());
        // If the cache is full, remove the oldest entry.
        while entry.len() >= CACHE_GENERATION as usize {
            entry.pop_back();
        }
        entry.push_front((version_hash, expr.clone()));
    }

    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr> {
        let data = self.data.lock().unwrap();
        let entity_id = MemoryCache::entity_identity(name, type_);
        let version_hash = version_hash.to_string();
        let entry = data.get(&entity_id)?;
        let expr = entry
            .iter()
            .find(|(hash, _)| hash == &version_hash)?
            .1
            .clone();
        Some(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::FileCache;
    use crate::ast::name::FullName;
    use crate::ast::types::{type_tyvar_star, Scheme};

    // A field accessor and a value the user writes are two entities whose names differ only in a
    // character a file name cannot carry. Their cache files must stay apart: a shared file hands
    // one entity the body of the other, and the read skips type checking, so nothing reports it.
    #[test]
    fn cache_files_of_names_differing_only_in_punctuation_stay_apart() {
        let cache = FileCache::new();
        let scheme = Scheme::from_type(type_tyvar_star("a"));
        let accessor = FullName::from_strs(&["Main", "S"], "@b");
        let value = FullName::from_strs(&["Main", "S"], "_b");

        assert_ne!(
            cache.cache_file_name(&accessor, &scheme, "0"),
            cache.cache_file_name(&value, &scheme, "0"),
        );
    }
}
