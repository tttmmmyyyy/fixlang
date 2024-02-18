use super::*;
use std::{cell::Cell, fs, process, time::Instant};

pub fn error_exit(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    process::exit(1)
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
    fs::create_dir_all(DOT_FIXLANG).expect("Failed to create .fixlang directory.");
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

pub struct StopWatch {
    name: String,
    now: Instant,
    running: Cell<bool>,
}

impl StopWatch {
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            now: Instant::now(),
            running: Cell::new(true),
        }
    }

    pub fn end(&self) {
        if !self.running.get() {
            return;
        }
        let elapsed = self.now.elapsed();
        let time_str = format!("{}.{:03} sec", elapsed.as_secs(), elapsed.subsec_millis());
        eprintln!("{}: {}", self.name, time_str);
        self.running.set(false);
    }
}

impl Drop for StopWatch {
    fn drop(&mut self) {
        self.end();
    }
}
