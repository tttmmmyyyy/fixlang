use crate::{
    ast::name::Name,
    constants::{COMPILER_THREAD_STACK_SIZE, TEMPORARY_SRC_PATH},
    error::Errors,
    hash::md5_hex,
    parse::sourcefile::SourceFile,
};
use colored::{control, ColoredString, Colorize};
use fxhash::{FxHashMap, FxHashSet};
use std::{
    cmp, env, fs,
    hash::Hash,
    io::{self, ErrorKind, IsTerminal, Write},
    panic::resume_unwind,
    path::{Path, PathBuf},
    thread::{self, JoinHandle},
};

pub type Map<K, V> = FxHashMap<K, V>;

/// A map holding the given key-value pairs. When a key is given more than once, the value that
/// comes last is the one kept.
pub fn make_map<K: Eq + Hash, V>(kvs: impl IntoIterator<Item = (K, V)>) -> Map<K, V> {
    let mut map = Map::default();
    for (k, v) in kvs {
        map.insert(k, v);
    }
    map
}

pub type Set<T> = FxHashSet<T>;

/// A set holding the given elements, with an element that appears several times held once.
pub fn make_set<T: Eq + Hash>(iter: impl IntoIterator<Item = T>) -> Set<T> {
    let mut set = Set::default();
    for elem in iter {
        set.insert(elem);
    }
    set
}

/// Run `f` on a stack grown on demand, so a deeply recursive traversal — the RC IR passes over a
/// continuation chain, type checking over a nested expression — does not overflow the stack on a
/// deeply nested input.
pub fn grow_stack<R>(f: impl FnOnce() -> R) -> R {
    // Allocate another 1 MiB of stack whenever less than 64 KiB of it remains.
    stacker::maybe_grow(64 * 1024, 1024 * 1024, f)
}

/// Spawn a thread whose stack (`COMPILER_THREAD_STACK_SIZE`) is large enough for the
/// compiler's recursion over deeply nested user expressions. The program's expression tree has
/// unbounded depth, so a thread spawned with the default stack overflows on deep inputs.
pub fn spawn_compiler_thread<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    thread::Builder::new()
        .stack_size(COMPILER_THREAD_STACK_SIZE)
        .spawn(f)
        .expect("failed to spawn a compiler thread")
}

/// The values the given threads returned, in the order the threads are given.
///
/// A thread that panicked carries its panic on from here, once **every** thread has been joined:
/// unwinding tears down the state the compiler works on, and a thread still running under it reads
/// and writes memory that is being freed, which crashes the process on top of the panic that was
/// meant to be reported.
pub fn join_compiler_threads<T>(threads: Vec<JoinHandle<T>>) -> Vec<T> {
    let mut values = vec![];
    let mut panic_payload = None;
    for thread in threads {
        match thread.join() {
            Ok(value) => values.push(value),
            // Of the threads that panicked, the earliest in the list is the one whose payload is
            // carried on, by `resume_unwind`: that thread has already reported through the panic
            // hook, and a joined payload is an opaque `Box<dyn Any>`, so raising it as a fresh
            // panic would report a second time and call it an unknown error.
            Err(payload) => panic_payload = panic_payload.or(Some(payload)),
        }
    }
    if let Some(payload) = panic_payload {
        resume_unwind(payload);
    }
    values
}

/// The name a source is saved under in the temporary directory: `file_name` with `hash` — a digest
/// of the source's content — inserted before the `.fix` extension, so that two sources of the same
/// name and different content are saved side by side.
pub fn temporary_source_name(file_name: &str, hash: &str) -> String {
    format!("{}.{}.fix", file_name, hash)
}

/// The path a source is saved at: `TEMPORARY_SRC_PATH` joined with the name
/// `temporary_source_name` builds from `file_name` and `hash`.
pub fn temporary_source_path(file_name: &str, hash: &str) -> PathBuf {
    let file_name = temporary_source_name(file_name, hash);
    PathBuf::from(TEMPORARY_SRC_PATH).join(file_name)
}

/// Saves `source` in the temporary directory, under a name built from `file_name` and a digest of
/// the content, and answers with the source file it was saved as.
///
/// A file already saved at that path holds this same content, since the digest is taken over it, so
/// it is kept as it stands.
pub fn save_temporary_source(source: &str, file_name: &str) -> Result<SourceFile, Errors> {
    let hash = md5_hex(source);
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
            file.write_all(source.as_bytes()).map_err(|e| {
                Errors::from_msg(format!(
                    "Failed to write temporary file \"{}\": {}",
                    file_name, e
                ))
            })?;
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            // File already exists, which is fine
        }
        Err(e) => {
            return Err(Errors::from_msg(format!(
                "Failed to create temporary file \"{}\": {}",
                file_name, e
            )));
        }
    }

    let source_file = SourceFile::from_file_path_and_content(path, source.to_string());
    Ok(source_file)
}

/// The values of `results`, in the order they are produced, or the first error among them.
///
/// Iteration stops at that error, so the results behind it are never produced.
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

/// Every nonempty run of consecutive elements of `v`, ordered by where the run starts and then by
/// where it ends.
///
/// # Examples
/// `nonempty_subsequences(&vec![1, 2])` is `[[1], [1, 2], [2]]`.
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

/// Splits `v` into pieces of at most `max_size` elements, each piece holding at least one element
/// and the pieces read in order giving back `v`.
pub fn split_by_max_size<T>(mut v: Vec<T>, max_size: usize) -> Vec<Vec<T>> {
    v.reverse();
    let mut result = vec![];
    while v.len() > 0 {
        let len = cmp::min(max_size, v.len());
        let mut chunk = v.split_off(v.len() - len);
        chunk.reverse();
        result.push(chunk);
    }
    result
}

/// Appends `elem` to the vector `key` maps to, starting that vector when `key` maps to none.
pub fn insert_to_map_vec<K: Clone + Eq + Hash, V>(map: &mut Map<K, Vec<V>>, key: &K, elem: V) {
    if let Some(vec) = map.get_mut(key) {
        vec.push(elem);
    } else {
        map.insert(key.clone(), vec![elem]);
    }
}

/// Appends `elems`, in order, to the vector `key` maps to, starting that vector when `key` maps to
/// none.
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

#[allow(unused_imports)]
pub(crate) use function_name;

/// The variable name for `n`, a letter followed by a number where the letters run out. Each `n` has
/// a name of its own.
///
/// # Examples
/// `number_to_varname(0)` is `a`, `number_to_varname(25)` is `z`, and `number_to_varname(26)` is
/// `a1`.
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

/// `count` variable names, each differing from the others and from every name in `used_names`.
pub fn generate_fresh_varnames(count: usize, used_names: &Set<Name>) -> Vec<Name> {
    let mut result = Vec::with_capacity(count);
    let mut name_no = 0usize;
    for _ in 0..count {
        loop {
            let candidate = number_to_varname(name_no);
            name_no += 1;
            if !used_names.contains(&candidate) {
                result.push(candidate);
                break;
            }
        }
    }
    result
}

/// `path` taken against the current directory and canonicalized, so that two paths leading to one
/// file become one string and can be compared.
///
/// Canonicalization reads the file system, so the path has to lead to a file that exists.
pub fn to_absolute_path(path: &Path) -> Result<PathBuf, Errors> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        match env::current_dir() {
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

/// Works deferred to the moment this value is dropped, run latest first.
pub struct Finally {
    /// The works deferred so far, in the order they were deferred.
    works: Vec<Box<dyn FnOnce()>>,
}

impl Finally {
    /// A `Finally` with no work deferred.
    pub fn new() -> Self {
        Self { works: vec![] }
    }

    /// Defers `work` until this value is dropped.
    pub fn defer<F: FnOnce() + 'static>(&mut self, work: F) {
        self.works.push(Box::new(work));
    }
}

impl Drop for Finally {
    /// Runs the deferred works, latest first.
    fn drop(&mut self) {
        for work in self.works.drain(..).rev() {
            work();
        }
    }
}

/// Turns off the color of every message the compiler prints, when its error output goes somewhere
/// other than a terminal.
pub fn disable_colored_no_tty() {
    if !io::stderr().is_terminal() {
        control::set_override(false);
    }
}

/// Prints `msg` to standard error under an `info` label.
pub fn info_msg(msg: &str) {
    eprintln!("{}: {}", "info".bright_blue().bold(), msg);
}

/// Prints `msg` to standard error under a `warning` label.
pub fn warn_msg(msg: &str) {
    eprintln!("{}: {}", "warning".yellow().bold(), msg);
}

/// Styles `s` as a line of an interactive prompt that requires the user's attention, so that every
/// such prompt looks alike.
pub fn prompt_style(s: &str) -> ColoredString {
    s.bright_green().bold()
}

/// `text` cut short where it is too long for a report to carry, with an ellipsis marking the cut.
///
/// A type or a constraint that trips one of the compiler's bounds can be a term of any size, and
/// the whole of one says no more than its beginning does.
pub fn shorten_for_report(text: String) -> String {
    /// How much of one term a report shows.
    const MAX_SHOWN_CHARS: usize = 200;

    match text.char_indices().nth(MAX_SHOWN_CHARS) {
        Some((cut, _)) => format!("{}...", &text[..cut]),
        None => text,
    }
}

/// Splits `s` at spaces, keeping a quoted run of characters as one word.
///
/// Single and double quotes both quote, and a backslash makes the character after it part of the
/// word it stands in. The quotes and backslashes themselves stay out of the words returned.
pub fn split_string_by_space_not_quated(s: &str) -> Vec<String> {
    let mut words = Vec::new();
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
                    words.push(current_word.clone());
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
        words.push(current_word);
    }

    words
}

/// Rewrites `s`, written in `UpperCamelCase`, as `lower_snake_case`.
///
/// Requires `s` to be ASCII alphanumeric.
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
    use super::{
        char_pos_to_utf16_pos, join_compiler_threads, spawn_compiler_thread, split_by_max_size,
        split_string_by_space_not_quated, upper_camel_to_lower_snake, utf16_pos_to_utf8_byte_pos,
    };
    use crate::error::any_to_string;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    /// Every thread has finished by the time a worker's panic is carried on, so unwinding never
    /// tears down state that a thread still running is working on.
    #[test]
    fn test_join_compiler_threads_joins_every_thread() {
        const SLOW_THREAD_COUNT: usize = 3;
        let finished_count = Arc::new(AtomicUsize::new(0));

        // The thread that panics is joined first, so a collector that carries the panic on at the
        // first `Err` it sees leaves the slow threads running.
        let mut threads = vec![spawn_compiler_thread(|| {
            panic!("this thread panics on purpose")
        })];
        for _ in 0..SLOW_THREAD_COUNT {
            let finished_count = finished_count.clone();
            threads.push(spawn_compiler_thread(move || {
                thread::sleep(Duration::from_millis(500));
                finished_count.fetch_add(1, Ordering::SeqCst);
            }));
        }

        let joined = catch_unwind(AssertUnwindSafe(|| join_compiler_threads(threads)));
        assert!(joined.is_err(), "the worker's panic is carried on");
        assert_eq!(
            finished_count.load(Ordering::SeqCst),
            SLOW_THREAD_COUNT,
            "every thread has finished by the time the panic is carried on"
        );
    }

    /// The panic carried on holds the worker's own payload, so the renderer the compiler reports
    /// through still finds the message in it. Raising a fresh panic around the joined
    /// `Box<dyn Any>` would report the error a second time, and as `(unknown error)`.
    #[test]
    fn test_join_compiler_threads_carries_the_workers_own_payload() {
        const MESSAGE: &str = "this thread panics on purpose";
        let threads: Vec<JoinHandle<()>> = vec![spawn_compiler_thread(|| panic!("{}", MESSAGE))];

        let payload = catch_unwind(AssertUnwindSafe(|| join_compiler_threads(threads)))
            .expect_err("the worker's panic is carried on");
        assert_eq!(any_to_string(&*payload), MESSAGE);
    }

    /// The values come back in the order the threads were given, whatever order they finish in.
    #[test]
    fn test_join_compiler_threads_returns_values_in_the_order_given() {
        let threads = vec![
            // The first thread finishes last, so the assertion below tells the order of the
            // threads apart from the order they finished in.
            spawn_compiler_thread(|| {
                thread::sleep(Duration::from_millis(200));
                "first"
            }),
            spawn_compiler_thread(|| "second"),
        ];

        assert_eq!(join_compiler_threads(threads), vec!["first", "second"]);
    }

    /// Every piece a split produces holds at least one element and at most `max_size` of them, and
    /// the pieces read back as the input. A consumer turns each piece into a unit of work, so an
    /// empty piece is a unit with nothing in it.
    #[test]
    fn test_split_by_max_size_pieces_are_nonempty() {
        for max_size in 1..=5 {
            for len in 0..=12 {
                let v: Vec<usize> = (0..len).collect();
                let pieces = split_by_max_size(v.clone(), max_size);
                assert!(
                    pieces.iter().all(|piece| !piece.is_empty()),
                    "max_size = {}, len = {}",
                    max_size,
                    len
                );
                assert!(
                    pieces.iter().all(|piece| piece.len() <= max_size),
                    "max_size = {}, len = {}",
                    max_size,
                    len
                );
                assert_eq!(pieces.concat(), v, "max_size = {}, len = {}", max_size, len);
            }
        }
    }

    /// Splitting a command line into words: a run of spaces separates words, a quoted run — single
    /// or double — stays one word, a backslash escapes the character after it, and an input of
    /// spaces alone yields no word at all.
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

/// Whether this platform can build and run a program instrumented with ThreadSanitizer.
///
/// The instrumented program needs the sanitizer runtime that ships with clang, and the address
/// space layout its shadow memory is mapped into; this compiler arranges both on Linux.
pub fn platform_thread_sanitizer_supported() -> bool {
    env::consts::OS == "linux"
}
