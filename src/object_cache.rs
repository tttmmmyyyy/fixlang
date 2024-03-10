/*
Cache system for object (*.o) files.
*/

use std::collections::HashMap;
use std::path::PathBuf;

use crate::ast::name::FullName;
use crate::ast::name::Name;
use crate::configuration::Configuration;
use crate::constants::INTERMEDIATE_PATH;

// Get cache file name for the object file consists from the given symbols.
// - names: Sequence of symbols. This should be sorted.
// - mod_to_hash: A map from module name to the hash of dependency files.
fn cache_file_name(
    symbol_names: &[FullName],
    mod_to_hash: HashMap<Name, String>,
    config: &Configuration,
) -> String {
    let mut data = config.object_generation_hash();
    for name in symbol_names {
        data.push_str(&name.to_string());
        data.push_str(&mod_to_hash[&name.module()]);
    }
    format!("symbols_{:x}.o", md5::compute(data))
}

fn cache_file_path(
    symbol_names: &[FullName],
    mod_to_hash: HashMap<Name, String>,
    config: &Configuration,
) -> PathBuf {
    let mut path = PathBuf::from(INTERMEDIATE_PATH);
    path.push(cache_file_name(symbol_names, mod_to_hash, config));
    path
}

pub fn is_cached(
    symbol_names: &[FullName],
    mod_to_hash: HashMap<Name, String>,
    config: &Configuration,
) -> bool {
    cache_file_path(symbol_names, mod_to_hash, config).exists()
}
