use std::{collections::HashMap, path::PathBuf};

use crate::{misc, sourcefile::Span};

pub struct Errors {
    errs: Vec<Error>,
}

impl Errors {
    pub fn from_msg(msg: &str) -> Errors {
        Errors {
            errs: vec![Error::from_msg(msg)],
        }
    }

    pub fn to_string(&self) -> String {
        let mut str = String::default();
        for err in &self.errs {
            str += &err.to_string();
            str += "\n\n";
        }
        str
    }

    // Organize all `Error`s by the path of its (first) `Span`.
    // If an `Error` has no `Span`, it will be considered as having a path `.`.
    pub fn organize_by_path(&self) -> Vec<(PathBuf, Vec<Error>)> {
        // Organize errors into a hashmap.
        let mut map: HashMap<PathBuf, Vec<Error>> = HashMap::default();
        for err in &self.errs {
            let path = match err.srcs.first() {
                None => PathBuf::from("."),
                Some(span) => span.input.file_path.clone(),
            };
            misc::insert_to_hashmap_vec(&mut map, &path, err.clone());
        }

        // Convert the hashmap into a vector.
        let mut res = map.into_iter().collect::<Vec<_>>();

        // Sort the vector by the path.
        res.sort_by(|a, b| a.0.cmp(&b.0));

        res
    }
}

#[derive(Clone)]
pub struct Error {
    pub msg: String,
    pub srcs: Vec<Span>,
}

impl Error {
    pub fn from_msg(msg: &str) -> Error {
        Error {
            msg: msg.to_string(),
            srcs: vec![],
        }
    }

    pub fn to_string(&self) -> String {
        let mut str = String::default();
        str += &self.msg;
        str += "\n";
        for src in &self.srcs {
            str += "\n";
            str += &src.to_string();
        }
        str
    }
}

pub fn error_exit(msg: &str) -> ! {
    // Default panic hook shows message such as "thread 'main' panicked at " or "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace".
    // We replace it to empty.
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or(format!("{:?}", payload));
        eprintln!("{}", msg);
    }));
    panic!("error: {}", msg);
}

pub fn exit_if_err<T>(err: Result<T, Errors>) -> T {
    err.unwrap_or_else(|errs| error_exit(&errs.to_string()))
}

pub fn error_exit_with_src(msg: &str, src: &Option<Span>) -> ! {
    let mut str: String = String::default();
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
