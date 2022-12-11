use super::*;
use std::process;

pub fn error_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1)
}

pub fn error_exit_with_src(msg: &str, src: &Option<Span>) -> ! {
    let mut str = String::default();
    str += "error: ";
    str += msg;
    str += "\n";
    match src {
        None => todo!(),
        Some(v) => {
            str += &v.to_string();
        }
    };
    error_exit(&str)
}

pub fn capitalize_head(str: &str) -> String {
    if str.len() > 0 {
        str.to_string()
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
            .collect()
    } else {
        str.to_string()
    }
}
