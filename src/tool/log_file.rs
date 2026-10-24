use crate::constants::LOG_FILE_PATH;
use crate::error::panic_if_err;
use crate::project_fs::write_file_guarding_project_links;
use once_cell::sync::Lazy;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// Whether `write_log!` writes its messages to the log file at `LOG_FILE_PATH`. Turned on by hand
/// to debug a `fix` process, such as the language server, whose output a test cannot read.
pub const WRITE_LOG: bool = false;

/// The log file, created empty when the first message is written to it.
static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| open_log_file());

/// Creates the log file empty and opens it for appending. A symbolic link at its path is replaced
/// by the new file. Panics when the file cannot be created, or when a symbolic link in the project
/// would place it outside the project.
fn open_log_file() -> Mutex<File> {
    let path = panic_if_err(write_file_guarding_project_links(
        Path::new(LOG_FILE_PATH),
        |tmp_path| File::create(tmp_path).map(|_| ()),
    ));
    let file = OpenOptions::new()
        .append(true)
        .open(path)
        .expect(format!("Failed to open `{}` file.", LOG_FILE_PATH).as_str());
    Mutex::new(file)
}

/// Appends `message` and a newline to the log file when `WRITE_LOG` is on, and does nothing
/// otherwise. Called by `write_log!`.
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
