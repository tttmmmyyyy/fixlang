use std::{
    collections::{BTreeMap, VecDeque},
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{touch_directory, FullName, Scheme, TypedExpr, TYPE_CHECK_CACHE_PATH};

pub type SharedTypeCheckCache = Arc<dyn TypeCheckCache + Send + Sync>;

// A trait for objects which manage caching of typechecked expressions.
pub trait TypeCheckCache {
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
    fn cache_file_name(&self, name: &FullName, type_: &Arc<Scheme>, version_hash: &str) -> String {
        let name = name.to_string();
        // To make it filename-safe, replace all non-alphanumeric characters with underscores.
        let name = name.replace(|c: char| !c.is_alphanumeric(), "_");

        let type_ = type_.to_string_normalize();
        // To make it filename-safe, take md5 hash.
        let type_ = format!("{:x}", md5::compute(type_));

        format!("{}_{}_{}.cache", name, type_, version_hash)
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
                eprintln!(
                    "warning: Failed to create cache file \"{}\".",
                    cache_file_str
                );
                return;
            }
            Ok(file) => file,
        };
        let serialized = serde_pickle::to_vec(&expr, Default::default()).unwrap();
        match cache_file.write_all(&serialized) {
            Ok(_) => {}
            Err(_) => {
                eprintln!(
                    "warning: Failed to write cache file \"{}\".",
                    cache_file_str
                );
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
                eprintln!(
                    "warning: Failed to read cache file \"{}\": {}.",
                    cache_file_str, why
                );
                return None;
            }
        }
        let expr: TypedExpr = match serde_pickle::from_slice(&cache_bytes, Default::default()) {
            Ok(res) => res,
            Err(why) => {
                eprintln!(
                    "warning: Failed to parse content of cache file \"{}\": {}.",
                    cache_file_str, why
                );
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
            data: Mutex::new(BTreeMap::new()),
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
