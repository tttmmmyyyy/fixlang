use super::*;
use core::panic;
use std::{fs, hash::Hash};

pub fn temporary_source_name(file_name: &str, hash: &str) -> String {
    format!("{}.{}.fix", file_name, hash)
}

pub fn temporary_source_path(file_name: &str, hash: &str) -> PathBuf {
    let file_name = temporary_source_name(file_name, hash);
    PathBuf::from(TEMPORARY_SRC_PATH).join(file_name)
}

pub fn check_temporary_source(file_name: &str, hash: &str) -> bool {
    temporary_source_path(file_name, hash).exists()
}

pub fn save_temporary_source(source: &str, file_name: &str, hash: &str) {
    let path = temporary_source_path(file_name, hash);
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).expect(
        format!(
            "Failed to create directory \"{}\".",
            parent.to_string_lossy().to_string()
        )
        .as_str(),
    );
    fs::write(path, source).expect(&format!("Failed to generate temporary file {}", file_name));
}

pub fn collect_results<T, E>(results: impl Iterator<Item = Result<T, E>>) -> Result<Vec<T>, E> {
    let mut ok_results = vec![];
    for result in results {
        match result {
            Ok(v) => ok_results.push(v),
            Err(e) => return Err(e),
        }
    }
    Ok(ok_results)
}

pub fn flatten_opt<T>(o: Option<Option<T>>) -> Option<T> {
    match o {
        Some(o) => o,
        None => None,
    }
}

pub fn nonempty_subsequences<T: Clone>(v: &Vec<T>) -> Vec<Vec<T>> {
    let mut result = vec![];
    for i in 0..v.len() {
        for j in i..v.len() {
            result.push(v[i..j + 1].to_vec());
        }
    }
    result
}

// Given a vector, split it into subvectors, each of which has at most `max_size` elements.
// Each subvector is nonempty.
pub fn split_by_max_size<T>(mut v: Vec<T>, max_size: usize) -> Vec<Vec<T>> {
    v.reverse();
    let mut result = vec![];
    while v.len() > 0 {
        let len = std::cmp::min(max_size, v.len());
        let mut w = v.split_off(v.len() - len);
        w.reverse();
        result.push(w);
    }
    result
}

pub fn insert_to_hashmap_vec<K: Clone + Eq + Hash, V>(
    map: &mut HashMap<K, Vec<V>>,
    key: &K,
    elem: V,
) {
    if let Some(vec) = map.get_mut(key) {
        vec.push(elem);
    } else {
        map.insert(key.clone(), vec![elem]);
    }
}

// A macro to get the name of a function.
#[allow(unused)]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        // Find and cut the rest of the path
        match &name[..name.len() - 3].rfind(':') {
            Some(pos) => &name[pos + 1..name.len() - 3],
            None => &name[..name.len() - 3],
        }
    }};
}
pub(crate) use function_name;

// Creates a variable name from a number.
pub fn number_to_varname(n: usize) -> String {
    let mut ret = "".to_string();
    let mut n = n;
    let c = (n % 26) as u8 + 'a' as u8;
    ret.push(c as char);
    n /= 26;
    if n == 0 {
        return ret;
    }
    ret += &n.to_string();
    ret
}

// Converts a path to an absolute path.
pub fn to_absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Err(e) => {
                panic!("Failed to get the current directory: {}", e);
            }
            Ok(cur_dir) => cur_dir.join(path),
        }
    }
}
