use crate::constants::DOT_FIXLANG;
use crate::misc::remove_inside_dot_fixlang;
use std::path::Path;

// A function implementing `fix clean` command.
pub fn clean_command() {
    // Delete `.fixlang` directory.
    let _ = remove_inside_dot_fixlang(Path::new(DOT_FIXLANG));
}
