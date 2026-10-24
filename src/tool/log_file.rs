use crate::constants::LOG_FILE_PATH;
use crate::error::panic_if_err;
use crate::misc::write_file_guarding_project_links;
use once_cell::sync::Lazy;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

pub const WRITE_LOG: bool = false;

static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| open_log_file());

/// Creates the log file empty and opens it for appending. A symbolic link at its path is replaced
/// by the new file. Panics when the file cannot be created, or when a symbolic link in the project
/// would place it outside the project.
fn open_log_file() -> Mutex<File> {
    let path = Path::new(LOG_FILE_PATH);
    panic_if_err(write_file_guarding_project_links(path, |tmp_path| {
        File::create(tmp_path).map(|_| ())
    }));
    let file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect(format!("Failed to open `{}` file.", LOG_FILE_PATH).as_str());
    Mutex::new(file)
}

#[doc(hidden)]
pub fn write_log_file_impl(message: &str) {
    // The log file is opened when the first message is written, so a `fix` run that writes no log
    // leaves the file system unchanged.
    if WRITE_LOG {
        let mut file = LOG_FILE.lock().expect("Failed to lock the log file.");
        let message = message.to_string() + "\n";
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

#[macro_export]
macro_rules! write_log {
    ($($arg:tt)*) => {
        $crate::tool::log_file::write_log_file_impl(&format!($($arg)*))
    };
}
