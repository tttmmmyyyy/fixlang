use crate::DOT_FIXLANG;
use std::fs::remove_dir_all;

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    let _ = remove_dir_all(DOT_FIXLANG);
}
