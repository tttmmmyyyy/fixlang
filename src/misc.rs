use super::*;
use std::process;

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
            str += &v.to_string();
        }
    };
    error_exit(&str)
}
