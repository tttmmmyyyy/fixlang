use crate::constants::DOT_FIXLANG;
use crate::project_fs::remove_inside_dot_fixlang;
use std::path::Path;

/// Removes the project's `.fixlang` directory, which holds the build artifacts, the caches and
/// the installed git dependencies.
pub fn clean_command() {
    let _ = remove_inside_dot_fixlang(Path::new(DOT_FIXLANG));
}
