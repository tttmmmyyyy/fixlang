use std::io::{Read, Write};

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
    let mut stdin = stdin.lock();

    let mut buffer = [0; 1];
    while stdin.read_exact(&mut buffer).is_ok() {
        log_file.write_all(&buffer).unwrap();
        log_file.flush().unwrap();
    }
}
