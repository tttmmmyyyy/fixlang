use std::process;

pub fn error_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    process::exit(1)
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
