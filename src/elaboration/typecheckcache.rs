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

/// A type-check cache held by the threads that check a program together.
pub type SharedTypeCheckCache = Arc<dyn TypeCheckCache + Send + Sync>;

/// Keeps the expressions produced by type checking, so that a value asked for again under the same
/// type and the same sources is taken back instead of checked afresh.
///
/// `RefUnwindSafe` is a supertrait so that `Arc<dyn TypeCheckCache + Send + Sync>` satisfies
/// `UnwindSafe`, which a closure carrying one across a `catch_unwind` boundary requires.
pub trait TypeCheckCache: RefUnwindSafe {
    /// Stores `expr` as the result of checking the global value `name` against `type_`, under the
    /// hash of the sources that value depends on.
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    );
    /// Answers with the expression stored for the global value `name` checked against `type_`
    /// under `version_hash`.
    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr>;
}

/// The entity a cache entry belongs to: a global value, together with the type it was checked
/// against. A cache that lets two entities meet in one entry hands one of them the typed
/// expression of the other, and the read installs it without checking it.
type EntityIdentity = String;

/// The hash of the sources the entity depends on, which tells the entries of one entity apart.
type VersionHash = String;

/// A name and a printed type each occupy one line, so joining the two with a newline gives a
/// distinct string to every distinct pair.
fn entity_identity(name: &FullName, type_: &Arc<Scheme>) -> EntityIdentity {
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

/// A cache that gives every entry a file of its own, so the entries outlive the run that wrote
/// them.
pub struct FileCache {}

impl FileCache {
    /// Creates a handle to the cache. Every handle reaches the same entries, which live under
    /// `TYPE_CHECK_CACHE_PATH` relative to the working directory.
    pub fn new() -> Self {
        FileCache {}
    }

    /// The name of the file that holds the cache entry for a value.
    ///
    /// The digest is what identifies the entry: it is taken over the entity and the version hash at
    /// once, so two entries meet in one file only when both agree. The part in front of it names
    /// the value for someone reading the cache directory, and is filename-safe because it keeps
    /// only alphanumeric characters; several values can wear the same one.
    ///
    /// # Examples
    ///
    /// The entries of `Main::hole_val` are filed under names of the form `Main__hole_val_<digest>`.
    fn cache_file_name(&self, name: &FullName, type_: &Arc<Scheme>, version_hash: &str) -> String {
        let key = format!("{}\n{}", entity_identity(name, type_), version_hash);
        let digest = format!("{:x}", md5::compute(key));

        let readable_name = name
            .to_string()
            .replace(|c: char| !c.is_alphanumeric(), "_");
        format!("{}_{}", readable_name, digest)
    }

    /// The path of the file that holds the cache entry for a value, creating the cache directory if
    /// it is absent.
    fn cache_file_path(&self, name: &FullName, type_: &Arc<Scheme>, version_hash: &str) -> PathBuf {
        let cache_file_name = self.cache_file_name(name, type_, version_hash);
        touch_directory(TYPE_CHECK_CACHE_PATH).join(cache_file_name)
    }
}

impl TypeCheckCache for FileCache {
    /// Writes the expression into the entry's own file. A file that cannot be created or written
    /// is reported as a warning and leaves the entry absent.
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) {
        let cache_file_path = self.cache_file_path(name, type_, version_hash);
        let cache_file_path_str = cache_file_path.to_string_lossy().to_string();
        let mut cache_file = match File::create(&cache_file_path) {
            Err(_) => {
                warn_msg(&format!(
                    "Failed to create cache file \"{}\".",
                    cache_file_path_str
                ));
                return;
            }
            Ok(file) => file,
        };
        let serialized = postcard::to_allocvec(expr).unwrap();
        match cache_file.write_all(&serialized) {
            Ok(_) => {}
            Err(_) => {
                warn_msg(&format!(
                    "Failed to write cache file \"{}\".",
                    cache_file_path_str
                ));
            }
        }
    }

    /// Reads the entry's own file and answers with the expression it holds. A file that cannot be
    /// read or parsed is reported as a warning and answers as an absent entry does.
    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr> {
        let cache_file_path = self.cache_file_path(name, type_, version_hash);
        let cache_file_path_str = cache_file_path.to_string_lossy().to_string();
        if !cache_file_path.exists() {
            return None;
        }
        let mut cache_file = match File::open(&cache_file_path) {
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
                    cache_file_path_str, why
                ));
                return None;
            }
        }
        // The format carries no end marker. A file that lost its tail fails to parse because the
        // expression wants the bytes that went missing, but a file with bytes past the expression
        // parses without them ever being looked at. An entry this cache wrote holds one expression
        // and nothing else, so bytes left over say the file is not one.
        let (expr, rest): (TypedExpr, _) = match postcard::take_from_bytes(&cache_bytes) {
            Ok(res) => res,
            Err(why) => {
                warn_msg(&format!(
                    "Failed to parse content of cache file \"{}\": {}.",
                    cache_file_path_str, why
                ));
                return None;
            }
        };
        if !rest.is_empty() {
            warn_msg(&format!(
                "Failed to parse content of cache file \"{}\": bytes follow the expression it holds.",
                cache_file_path_str
            ));
            return None;
        }
        Some(expr)
    }
}

/// How many versions of one entity `MemoryCache` keeps. Storing a further version drops the one
/// stored longest ago.
const CACHE_GENERATION: u64 = 3;

/// A cache that holds its entries in memory, so they last as long as the process that filled it.
pub struct MemoryCache {
    /// The stored expressions, grouped by the entity they belong to. Within a group the version
    /// stored most recently comes first, and at most `CACHE_GENERATION` versions are held.
    data: Mutex<BTreeMap<EntityIdentity, VecDeque<(VersionHash, TypedExpr)>>>,
}

impl MemoryCache {
    /// Creates a cache holding no entries.
    pub fn new() -> Self {
        MemoryCache {
            data: Mutex::new(BTreeMap::default()),
        }
    }
}

impl TypeCheckCache for MemoryCache {
    /// Puts the expression at the front of the entity's versions, dropping the versions stored
    /// longest ago to stay within `CACHE_GENERATION`.
    fn save_cache(
        &self,
        expr: &TypedExpr,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) {
        let mut data = self.data.lock().unwrap();
        let entity_id = entity_identity(name, type_);
        let version_hash = version_hash.to_string();
        let entries = data.entry(entity_id).or_insert_with(|| VecDeque::new());
        // If the cache is full, remove the oldest entry.
        while entries.len() >= CACHE_GENERATION as usize {
            entries.pop_back();
        }
        entries.push_front((version_hash, expr.clone()));
    }

    /// Searches the entity's versions for the one stored under `version_hash`.
    fn load_cache(
        &self,
        name: &FullName,
        type_: &Arc<Scheme>,
        version_hash: &str,
    ) -> Option<TypedExpr> {
        let data = self.data.lock().unwrap();
        let entity_id = entity_identity(name, type_);
        let version_hash = version_hash.to_string();
        let entries = data.get(&entity_id)?;
        let expr = entries
            .iter()
            .find(|(hash, _)| hash == &version_hash)?
            .1
            .clone();
        Some(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::{entity_identity, FileCache};
    use crate::{
        ast::name::FullName,
        ast::types::{type_tyvar_star, Scheme},
        fixstd::builtin::{make_bool_ty, make_i64_ty},
    };

    /// A field accessor and a value the user writes are two entities whose names differ only in a
    /// character a file name cannot carry. Their cache files must stay apart: a shared file hands
    /// one entity the body of the other, and the read skips type checking, so nothing reports it.
    #[test]
    fn cache_files_of_names_differing_only_in_punctuation_stay_apart() {
        let cache = FileCache::new();
        let scheme = Scheme::from_type(type_tyvar_star("a"));
        let accessor_name = FullName::from_strs(&["Main", "S"], "@b");
        let value_name = FullName::from_strs(&["Main", "S"], "_b");

        assert_ne!(
            cache.cache_file_name(&accessor_name, &scheme, "0"),
            cache.cache_file_name(&value_name, &scheme, "0"),
        );
    }

    /// The key has three components — the name of the value, the type it is checked against, and
    /// the hash of the sources it depends on — and each one alone tells two entries apart. The
    /// implementations of one trait method defined in a single module share the name and the hash
    /// and differ in the type; an edit to a source the value depends on changes the hash alone.
    #[test]
    fn cache_files_of_entries_differing_in_one_component_stay_apart() {
        let cache = FileCache::new();
        let name = FullName::from_strs(&["Std", "ToString"], "to_string");
        let i64_scheme = Scheme::from_type(make_i64_ty());
        let bool_scheme = Scheme::from_type(make_bool_ty());

        assert_ne!(
            cache.cache_file_name(&name, &i64_scheme, "0"),
            cache.cache_file_name(&name, &bool_scheme, "0"),
            "two entries of one name whose types differ share a file",
        );
        assert_ne!(
            cache.cache_file_name(&name, &i64_scheme, "0"),
            cache.cache_file_name(&name, &i64_scheme, "1"),
            "an entry keeps its file when the sources it depends on change",
        );
    }

    /// The namespace a value is written under is part of the entity its entry belongs to. Two
    /// values of one module, each under a namespace of its own, agree on their remaining name, on
    /// their type, and on the sources they are checked from, so the namespace alone tells their
    /// entries apart. Every cache reads the entity through `entity_identity`, so pinning it here
    /// reaches the entries held in memory as well as those given a file.
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
}
