use super::*;
use core::panic;
use std::{fs, hash::Hash};

pub type Map<K, V> = fxhash::FxHashMap<K, V>;

pub fn make_map<K: Eq + Hash, V>(kvs: impl IntoIterator<Item = (K, V)>) -> Map<K, V> {
    let mut map = Map::default();
    for (k, v) in kvs {
        map.insert(k, v);
    }
    map
}

pub type Set<T> = fxhash::FxHashSet<T>;

pub fn make_set<T: Eq + Hash>(iter: impl IntoIterator<Item = T>) -> Set<T> {
    let mut set = Set::default();
    for elem in iter {
        set.insert(elem);
    }
    set
}

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

#[allow(unused)]
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

pub fn insert_to_map_vec<K: Clone + Eq + Hash, V>(map: &mut Map<K, Vec<V>>, key: &K, elem: V) {
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
use colored::Colorize;
#[allow(unused_imports)]
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
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Err(e) => {
                panic!("Failed to get the current directory: {}", e);
            }
            Ok(cur_dir) => cur_dir.join(path),
        }
    };
    abs.canonicalize().unwrap_or_else(|e| {
        panic!(
            "Failed to canonicalize path \"{}\": {}",
            abs.to_string_lossy(),
            e
        )
    })
}

pub struct Finally {
    works: Vec<Box<dyn FnOnce()>>,
}

impl Finally {
    pub fn new() -> Self {
        Self { works: vec![] }
    }

    pub fn defer<F: FnOnce() + 'static>(&mut self, work: F) {
        self.works.push(Box::new(work));
    }
}

impl Drop for Finally {
    fn drop(&mut self) {
        for work in self.works.drain(..).rev() {
            work();
        }
    }
}

pub fn info_msg(msg: &str) {
    println!("{}: {}", "info".bright_blue(), msg);
}

pub fn warn_msg(msg: &str) {
    println!("{}: {}", "warning".yellow(), msg);
}

// Splits a string by spaces, but keeps the words in quotes as a single word.
pub fn split_string_by_space_not_quated(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut in_quotes = None; // None if not in quotes, Some(') if in single quotes, Some(") if in double quotes
    let mut escaped = false; // true if the previous character is an escape character

    for c in s.chars() {
        if escaped {
            current_word.push(c);
            escaped = false;
            continue;
        }

        match c {
            ' ' if in_quotes.is_none() => {
                if !current_word.is_empty() {
                    result.push(current_word.clone());
                    current_word.clear();
                }
            }
            '"' if in_quotes.is_none() => in_quotes = Some('"'),
            '"' if in_quotes == Some('"') => in_quotes = None,
            '\'' if in_quotes.is_none() => in_quotes = Some('\''),
            '\'' if in_quotes == Some('\'') => in_quotes = None,
            '\\' => escaped = true, // The next character is escaped
            _ => current_word.push(c),
        }
    }

    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_string() {
        assert_eq!(
            split_string_by_space_not_quated("hello world"),
            vec!["hello", "world"]
        );
        assert_eq!(
            split_string_by_space_not_quated("hello   world"),
            vec!["hello", "world"]
        );
        assert_eq!(
            split_string_by_space_not_quated(" \"hello world\" "),
            vec!["hello world"]
        );
        assert_eq!(
            split_string_by_space_not_quated(" 'hello world' "),
            vec!["hello world"]
        );
        assert_eq!(
            split_string_by_space_not_quated("hello \"big world\""),
            vec!["hello", "big world"]
        );
        assert_eq!(
            split_string_by_space_not_quated("'it\\'s a beautiful day'"),
            vec!["it's a beautiful day"]
        );
        assert_eq!(
            split_string_by_space_not_quated("\"this has \\\"escaped quotes\\\"\""),
            vec!["this has \"escaped quotes\""]
        );
        assert_eq!(
            split_string_by_space_not_quated("混合 \"日本語 の テスト\""),
            vec!["混合", "日本語 の テスト"]
        );
        assert_eq!(split_string_by_space_not_quated(""), Vec::<String>::new());
        assert_eq!(
            split_string_by_space_not_quated("   "),
            Vec::<String>::new()
        );
    }
}
