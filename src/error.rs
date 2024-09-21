use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{misc, sourcefile::Span};

pub struct Errors {
    errs: Vec<Error>,
}

impl Errors {
    pub fn empty() -> Errors {
        Errors { errs: vec![] }
    }

    fn has_error(&self) -> bool {
        !self.errs.is_empty()
    }

    pub fn to_result(&mut self) -> Result<(), Errors> {
        if self.has_error() {
            Err(std::mem::replace(self, Errors::empty()))
        } else {
            Ok(())
        }
    }

    pub fn append(&mut self, mut other: Errors) {
        self.errs.append(&mut other.errs);
    }

    // Append the error in `res` if it is an error.
    pub fn eat_err(&mut self, res: Result<(), Errors>) {
        match res {
            Ok(_v) => {}
            Err(errs) => {
                self.append(errs);
            }
        }
    }

    // Otherwise, append the error in `res` if it is an error.
    pub fn eat_err_or<T>(&mut self, res: Result<T, Errors>, act_if_ok: impl FnOnce(T)) {
        match res {
            Ok(v) => act_if_ok(v),
            Err(errs) => {
                self.append(errs);
            }
        }
    }

    pub fn from_msg(msg: String) -> Errors {
        Errors {
            errs: vec![Error::from_msg(msg)],
        }
    }

    pub fn from_msg_srcs(msg: String, srcs: &[&Option<Span>]) -> Errors {
        Errors {
            errs: vec![Error::from_msg_srcs(msg, srcs)],
        }
    }

    pub fn from_msg_err<E>(msg: &str, err: E) -> Errors
    where
        E: Display,
    {
        Errors::from_msg(format!("{}: {}", msg, err))
    }

    pub fn to_string(&self) -> String {
        let mut str = String::default();
        for err in &self.errs {
            str += "- ";
            str += &err.to_string();
            str += "\n";
        }
        str
    }

    // Organize all `Error`s by the path of its (first) `Span`.
    // If an `Error` has no `Span`, it will be considered as having a default PathBuf.
    pub fn organize_by_path(&self) -> Vec<(PathBuf, Vec<Error>)> {
        // Organize errors into a hashmap.
        let mut map: HashMap<PathBuf, Vec<Error>> = HashMap::default();
        for err in &self.errs {
            let path = match err.srcs.first() {
                None => PathBuf::new(),
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
    pub fn from_msg(msg: String) -> Error {
        Error { msg, srcs: vec![] }
    }

    pub fn from_msg_srcs(msg: String, srcs: &[&Option<Span>]) -> Error {
        Error {
            msg,
            srcs: srcs.iter().filter_map(|x| (*x).clone()).collect(),
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
        let msg = any_to_string(info.payload());
        eprintln!("{}", msg);
    }));
    panic!("The following error(s) has occurred: \n\n{}", msg);
}

pub fn any_to_string(any: &dyn std::any::Any) -> String {
    if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "(unknown error)".to_string()
    }
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

// pub fn error_exit_with_srcs(msg: &str, srcs: &[&Option<Span>]) -> ! {
//     let mut str = String::default();
//     str += msg;
//     str += "\n";
//     for src in srcs {
//         match src {
//             None => {}
//             Some(v) => {
//                 str += "\n";
//                 str += &v.to_string();
//             }
//         }
//     }
//     error_exit(&str)
// }
