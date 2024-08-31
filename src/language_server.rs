use crate::constants::LSP_LOG_FILE_PATH;
use std::{
    fs::File,
    io::{Read, Write},
};

// Launch the language server
pub fn launch_language_server() {
    let mut log_file = open_log_file();
    write_log(&mut log_file, "Language server started.\n");

    let mut stdin = std::io::stdin();

    loop {
        // Read a line to get the content length.
        let mut content_length = String::new();
        let res = stdin.read_line(&mut content_length);
        if res.is_err() {
            write_log(&mut log_file, "Failed to read the content length: \n");
            write_log(&mut log_file, &format!("{:?}\n", res.unwrap_err()));
            continue;
        }
        if content_length.trim().is_empty() {
            continue;
        }

        // Check if the line starts with "Content-Length:".
        if !content_length.starts_with("Content-Length:") {
            write_log(&mut log_file, "Expected `Content-Length:`. The line is: \n");
            write_log(&mut log_file, &format!("{:?}\n", content_length));
            continue;
        }

        // Ignore the `Content-Length:` prefix and parse the rest as a number.
        let content_length: Result<usize, _> = content_length
            .split_off("Content-Length:".len())
            .trim()
            .parse();
        if content_length.is_err() {
            write_log(&mut log_file, "Failed to parse the content length: \n");
            write_log(
                &mut log_file,
                &format!("{:?}\n", content_length.unwrap_err()),
            );
            continue;
        }
        let content_length = content_length.unwrap();
        write_log(
            &mut log_file,
            &format!("Content-Length: {}\n", content_length),
        );

        // Read stdin upto an empty line.
        loop {
            let mut line = String::new();
            let res = stdin.read_line(&mut line);
            if res.is_err() {
                write_log(&mut log_file, "Failed to read a line: \n");
                write_log(&mut log_file, &format!("{:?}\n", res.unwrap_err()));
                continue;
            }
            if line.trim().is_empty() {
                break;
            }
        }

        // Read the content of the message.
        let mut message = vec![0; content_length];
        let res = stdin.read_exact(&mut message);
        if res.is_err() {
            write_log(&mut log_file, "Failed to read the message: \n");
            write_log(&mut log_file, &format!("{:?}\n", res.unwrap_err()));
            continue;
        }
        let message = String::from_utf8(message);
        if message.is_err() {
            write_log(
                &mut log_file,
                "Failed to parse the message as utf-8 string: \n",
            );
            write_log(&mut log_file, &format!("{:?}\n", message.unwrap_err()));
            continue;
        }
        let message = message.unwrap();

        // Write the message to the log file.
        write_log(&mut log_file, "Message: \n");
        write_log(&mut log_file, &message);
        write_log(&mut log_file, "\n");
    }
}

fn open_log_file() -> File {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LSP_LOG_FILE_PATH)
        .parent()
        .expect("Failed to get parent directory of LSP_LOG_FILE_PATH.");

    // Create directories to the parent directory.
    std::fs::create_dir_all(parent_dir)
        .expect("Failed to create directories to the parent directory.");

    // Create and open the log file.
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LSP_LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LSP_LOG_FILE_PATH).as_str())
}

fn write_log(file: &mut File, message: &str) {
    file.write_all(message.as_bytes())
        .expect("Failed to write a message to the log file.");
    file.flush().expect("Failed to flush the log file.");
}
