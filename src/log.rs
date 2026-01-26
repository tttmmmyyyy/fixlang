use crate::constants::LOG_FILE_PATH;
use once_cell::sync::Lazy;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

pub const WRITE_LOG: bool = false;

static LOG_FILE: Lazy<Mutex<File>> = Lazy::new(|| open_log_file());

fn open_log_file() -> Mutex<File> {
    // Get parent directory of path `LSP_LOG_FILE_PATH`.
    let parent_dir = std::path::Path::new(LOG_FILE_PATH)
        .parent()
        .expect("Failed to get parent directory of LSP_LOG_FILE_PATH.");

    // Create directories to the parent directory.
    std::fs::create_dir_all(parent_dir)
        .expect("Failed to create directories to the parent directory.");

    // Create and open the log file.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(LOG_FILE_PATH)
        .expect(format!("Failed to open `{}` file.", LOG_FILE_PATH).as_str());

    // Wrap it into a mutex.
    Mutex::new(file)
}

#[doc(hidden)]
pub fn write_log_impl(message: &str) {
    let mut file = LOG_FILE.lock().expect("Failed to lock the log file.");
    if WRITE_LOG {
        let message = message.to_string() + "\n";
        file.write_all(message.as_bytes())
            .expect("Failed to write a message to the log file.");
        file.flush().expect("Failed to flush the log file.");
    }
}

#[macro_export]
macro_rules! write_log {
    ($($arg:tt)*) => {
        $crate::log::write_log_impl(&format!($($arg)*))
    };
}
