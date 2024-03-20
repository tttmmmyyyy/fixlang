use super::*;
use std::fs;

pub fn error_exit(msg: &str) -> ! {
    // Default panic hook shows message such as "thread 'main' panicked at " or "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace".
    // We replace it to empty.
    std::panic::set_hook(Box::new(move |_info| {}));

    eprintln!("error: {}", msg);
    panic!(""); // We dont use here `process::exit()` because we are using `#[should_panic]` attribute in tests.
}

pub fn error_exit_with_src(msg: &str, src: &Option<Span>) -> ! {
    let mut str = String::default();
    str += msg;
    str += "\n";
    match src {
        None => {}
        Some(v) => {
            str += "\n";
            str += &v.to_string();
        }
    };
    error_exit(&str)
}

pub fn error_exit_with_srcs(msg: &str, srcs: &[&Option<Span>]) -> ! {
    let mut str = String::default();
    str += msg;
    str += "\n";
    for src in srcs {
        match src {
            None => {}
            Some(v) => {
                str += "\n";
                str += &v.to_string();
            }
        }
    }

    error_exit(&str)
}

pub fn temporary_source_name(file_name: &str, hash: &str) -> String {
    format!("{}.{}.fix", file_name, hash)
}

pub fn temporary_source_path(file_name: &str, hash: &str) -> PathBuf {
    let file_name = temporary_source_name(file_name, hash);
    PathBuf::from(DOT_FIXLANG).join(file_name)
}

pub fn check_temporary_source(file_name: &str, hash: &str) -> bool {
    temporary_source_path(file_name, hash).exists()
}

pub fn save_temporary_source(source: &str, file_name: &str, hash: &str) {
    let path = temporary_source_path(file_name, hash);
    fs::create_dir_all(DOT_FIXLANG).expect("Failed to create \".fixlang\" directory.");
    fs::write(path, source).expect(&format!("Failed to generate temporary file {}", file_name));
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
