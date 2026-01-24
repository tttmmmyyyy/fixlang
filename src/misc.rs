use super::*;
use std::{env, fs, hash::Hash};

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

// Save a file with the specified content in a temporary directory with the specified name (with a hash value added to avoid collisions).
pub fn save_temporary_source(source: &str, file_name: &str) -> Result<SourceFile, Errors> {
    let hash = format!("{:x}", md5::compute(source));
    let path = temporary_source_path(file_name, &hash);
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to create directory \"{}\": {}",
            parent.to_string_lossy().to_string(),
            e
        ))
    })?;

    // Use create_new(true) for atomic check-and-create operation
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            use std::io::Write;
            // file.write_all(source.as_bytes())
            //     .expect(&format!("Failed to write temporary file {}", file_name));
            file.write_all(source.as_bytes()).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to write temporary file \"{}\": {}",
                    file_name, e
                ))
            })?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // File already exists, which is fine
        }
        Err(e) => {
            return Err(Errors::from_msg(format!(
                "Failed to create temporary file \"{}\": {}",
                file_name, e
            )));
        }
    }

    let source = SourceFile::from_file_path_and_content(path, source.to_string());
    Ok(source)
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

pub fn insert_to_map_vec_many<K: Clone + Eq + Hash, V>(
    map: &mut Map<K, Vec<V>>,
    key: &K,
    elems: Vec<V>,
) {
    if let Some(vec) = map.get_mut(key) {
        vec.extend(elems);
    } else {
        map.insert(key.clone(), elems);
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
pub fn to_absolute_path(path: &Path) -> Result<PathBuf, Errors> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match std::env::current_dir() {
            Err(e) => {
                return Err(Errors::from_msg(format!(
                    "Failed to get the current directory: {}",
                    e
                )));
            }
            Ok(cur_dir) => cur_dir.join(path),
        }
    };
    let abs = abs.canonicalize();
    if let Err(e) = abs {
        return Err(Errors::from_msg(format!(
            "Failed to canonicalize path \"{}\": {}",
            path.to_string_lossy(),
            e
        )));
    }
    Ok(abs.unwrap())
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

pub fn disable_colored_no_tty() {
    if !atty::is(Stream::Stderr) {
        colored::control::set_override(false);
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

// Upper CamelCase to lower_snake_case
pub fn upper_camel_to_lower_snake(s: &str) -> String {
    assert!(
        s.chars().all(|c| c.is_ascii_alphanumeric()),
        "Input must contain only ASCII alphanumeric characters"
    );

    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
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

    #[test]
    fn test_upper_camel_to_lower_snake() {
        assert_eq!(upper_camel_to_lower_snake("HelloWorld"), "hello_world");
        assert_eq!(upper_camel_to_lower_snake("MyClass"), "my_class");
        assert_eq!(upper_camel_to_lower_snake("MyClass123"), "my_class123");
        assert_eq!(upper_camel_to_lower_snake("ABC"), "a_b_c");
        assert_eq!(upper_camel_to_lower_snake("Hello"), "hello");
        assert_eq!(upper_camel_to_lower_snake("A"), "a");
        assert_eq!(upper_camel_to_lower_snake("IOError"), "i_o_error");
        assert_eq!(upper_camel_to_lower_snake("HTTPServer"), "h_t_t_p_server");
        assert_eq!(upper_camel_to_lower_snake("I64"), "i64");
        assert_eq!(upper_camel_to_lower_snake("CUnsignedInt"), "c_unsigned_int");
    }

    #[test]
    fn test_utf16_pos_to_utf8_byte_pos() {
        // ASCII only
        assert_eq!(utf16_pos_to_utf8_byte_pos("hello", 0), 0);
        assert_eq!(utf16_pos_to_utf8_byte_pos("hello", 3), 3);
        assert_eq!(utf16_pos_to_utf8_byte_pos("hello", 5), 5);

        // Japanese characters (3 bytes in UTF-8, 1 code unit in UTF-16)
        assert_eq!(utf16_pos_to_utf8_byte_pos("こんにちは", 0), 0);
        assert_eq!(utf16_pos_to_utf8_byte_pos("こんにちは", 1), 3);
        assert_eq!(utf16_pos_to_utf8_byte_pos("こんにちは", 2), 6);

        // Emoji (4 bytes in UTF-8, 2 code units in UTF-16)
        assert_eq!(utf16_pos_to_utf8_byte_pos("😀", 0), 0);
        assert_eq!(utf16_pos_to_utf8_byte_pos("😀", 2), 4);
        assert_eq!(utf16_pos_to_utf8_byte_pos("a😀b", 0), 0);
        assert_eq!(utf16_pos_to_utf8_byte_pos("a😀b", 1), 1);
        assert_eq!(utf16_pos_to_utf8_byte_pos("a😀b", 3), 5);
        assert_eq!(utf16_pos_to_utf8_byte_pos("a😀b", 4), 6);
    }

    #[test]
    fn test_char_pos_to_utf16_pos() {
        // ASCII only - single line
        assert_eq!(char_pos_to_utf16_pos("hello", 0, 0), 0);
        assert_eq!(char_pos_to_utf16_pos("hello", 0, 3), 3);
        assert_eq!(char_pos_to_utf16_pos("hello", 0, 5), 5);

        // ASCII only - multiple lines
        let multiline = "line1\nline2\nline3";
        assert_eq!(char_pos_to_utf16_pos(multiline, 0, 0), 0);
        assert_eq!(char_pos_to_utf16_pos(multiline, 0, 3), 3);
        assert_eq!(char_pos_to_utf16_pos(multiline, 1, 0), 0);
        assert_eq!(char_pos_to_utf16_pos(multiline, 1, 3), 3);
        assert_eq!(char_pos_to_utf16_pos(multiline, 2, 0), 0);
        assert_eq!(char_pos_to_utf16_pos(multiline, 2, 3), 3);

        // Japanese characters (1 character = 1 code unit in UTF-16)
        let japanese = "こんにちは\n世界";
        assert_eq!(char_pos_to_utf16_pos(japanese, 0, 0), 0);
        assert_eq!(char_pos_to_utf16_pos(japanese, 0, 2), 2);
        assert_eq!(char_pos_to_utf16_pos(japanese, 0, 5), 5);
        assert_eq!(char_pos_to_utf16_pos(japanese, 1, 0), 0);
        assert_eq!(char_pos_to_utf16_pos(japanese, 1, 2), 2);

        // Emoji (1 character = 2 code units in UTF-16)
        let emoji = "a😀b\nc😀d";
        assert_eq!(char_pos_to_utf16_pos(emoji, 0, 0), 0); // 'a'
        assert_eq!(char_pos_to_utf16_pos(emoji, 0, 1), 1); // before '😀'
        assert_eq!(char_pos_to_utf16_pos(emoji, 0, 2), 3); // after '😀', before 'b'
        assert_eq!(char_pos_to_utf16_pos(emoji, 0, 3), 4); // 'b'
        assert_eq!(char_pos_to_utf16_pos(emoji, 1, 0), 0); // 'c'
        assert_eq!(char_pos_to_utf16_pos(emoji, 1, 1), 1); // before '😀'
        assert_eq!(char_pos_to_utf16_pos(emoji, 1, 2), 3); // after '😀', before 'd'

        // Mixed content
        let mixed = "ASCII\nこんにちは\na😀b";
        assert_eq!(char_pos_to_utf16_pos(mixed, 0, 3), 3);
        assert_eq!(char_pos_to_utf16_pos(mixed, 1, 2), 2);
        assert_eq!(char_pos_to_utf16_pos(mixed, 2, 1), 1);
        assert_eq!(char_pos_to_utf16_pos(mixed, 2, 2), 3);
    }
}

// Convert a UTF-16 code unit position to a UTF-8 byte position in a string.
// This is useful for converting LSP positions (which use UTF-16) to Rust string indices (which use UTF-8).
pub fn utf16_pos_to_utf8_byte_pos(s: &str, utf16_pos: usize) -> usize {
    let mut utf16_count = 0;

    for (byte_idx, ch) in s.char_indices() {
        if utf16_count >= utf16_pos {
            return byte_idx;
        }
        utf16_count += ch.len_utf16();
    }

    // If we reach here, utf16_pos is at or beyond the end of the string
    s.len()
}

// Convert character position to UTF-16 code unit position
// This is useful for converting source span positions (which use character counts) to LSP positions (which use UTF-16).
pub fn char_pos_to_utf16_pos(source: &str, line: usize, char_col: usize) -> usize {
    let mut current_line = 0;
    let mut char_count = 0;
    let mut utf16_count = 0;

    for c in source.chars() {
        if c == '\n' {
            if current_line == line {
                // We've reached the end of the target line
                break;
            }
            current_line += 1;
            char_count = 0;
            utf16_count = 0;
            continue;
        }

        if current_line == line {
            if char_count >= char_col {
                break;
            }
            char_count += 1;
            utf16_count += c.len_utf16();
        }
    }

    utf16_count
}

pub fn platform_valgrind_supported() -> bool {
    env::consts::OS == "linux"
}
