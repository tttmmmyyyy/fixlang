use std::io::{BufRead, Write};

// Launch the language server
pub fn launch_language_server() {
    // Watch the standard input, and write it to the log file.

    // Create and open "ls.log" file
    let mut log_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("fixlang-language-server.log")
        .unwrap();

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();
    while handle.read_line(&mut buffer).unwrap() > 0 {
        log_file.write_all(buffer.as_bytes()).unwrap();
        buffer.clear();
    }
}
