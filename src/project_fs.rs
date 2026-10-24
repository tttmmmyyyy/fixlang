//! Writing and removing the files of a project, in such a way that what the project directory
//! holds, such as a symbolic link committed to a repository, cannot lead fix to write or remove
//! anything outside the project.

use crate::{constants::DOT_FIXLANG, error::Errors};
use rand::random;
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
};

/// Fails when the `.fixlang` of the current directory is a symbolic link. Fix writes and removes
/// the files under `.fixlang` as files of the project, and a `.fixlang` that is a symbolic link,
/// such as one committed to a repository, would lead those writes and removals outside the project.
pub fn check_dot_fixlang_is_not_symlink() -> Result<(), Errors> {
    if Path::new(DOT_FIXLANG).is_symlink() {
        return Err(Errors::from_msg(format!(
            "\"{}\" is a symbolic link. Fix keeps the files it builds and installs in the directory \"{}\" of the project.\n\
             HINT: `fix clean` removes the link, leaving what it points to.",
            DOT_FIXLANG, DOT_FIXLANG
        )));
    }
    Ok(())
}

/// The directory of the project fix works on, which is the current directory, with symbolic links
/// resolved.
fn project_dir() -> io::Result<PathBuf> {
    env::current_dir()?.canonicalize()
}

/// Removes the file or directory at `path`, which must be the `.fixlang` directory of the current
/// directory or lie inside it. Fix removes the files and directories of a project through this
/// function, so that the contents of a project directory, such as a lock file or a `.fixlang` that
/// is a symbolic link, cannot lead fix to remove anything outside `.fixlang`.
///
/// The check applies to the entry the removal takes away: the last component of `path`, inside its
/// parent directory with symbolic links and `..` resolved. A symbolic link at `path` itself is
/// removed as a link, and what it points to is kept. So `.fixlang` is removed even when it is a
/// symbolic link, while anything under such a link is refused.
pub fn remove_inside_dot_fixlang(path: &Path) -> Result<(), Errors> {
    let failed = |e: io::Error| {
        Errors::from_msg(format!(
            "Failed to remove \"{}\": {}",
            path.to_string_lossy(),
            e
        ))
    };
    let dot_fixlang = project_dir().map_err(failed)?.join(DOT_FIXLANG);
    let entry = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => {
            let parent = if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            };
            Some(parent.canonicalize().map_err(failed)?.join(name))
        }
        // A path ending in `..` names no entry of its own.
        _ => None,
    };
    let Some(entry) = entry.filter(|entry| entry.starts_with(&dot_fixlang)) else {
        return Err(Errors::from_msg(format!(
            "Refused to remove \"{}\", which lies outside \"{}\".",
            path.to_string_lossy(),
            dot_fixlang.to_string_lossy()
        )));
    };
    // The removal acts on the entry judged above. `path` itself, written with a trailing `/` or
    // `/.`, leads the file system through a symbolic link at its last component.
    if fs::symlink_metadata(&entry).map_err(failed)?.is_dir() {
        fs::remove_dir_all(&entry)
    } else {
        fs::remove_file(&entry)
    }
    .map_err(failed)
}

/// Writes the file at `path` by having `write` write a new file beside it, at the path it is handed,
/// and renaming that onto `path`, so that `path` holds either what it held before or the whole of
/// what `write` wrote. A symbolic link at `path` is replaced by the new file, and what it pointed to
/// is kept.
///
/// The new file is named apart from `path`, so that a name of `path` as long as the file system
/// allows still leaves room for it.
pub fn replace_file(path: &Path, write: impl FnOnce(&Path) -> io::Result<()>) -> io::Result<()> {
    let tmp_path = path.with_file_name(format!(".{:016x}.tmp", random::<u64>()));
    write(&tmp_path)
        .and_then(|()| fs::rename(&tmp_path, path))
        .inspect_err(|_| {
            // The temporary path carries a random number drawn by this call, so removing it touches
            // nothing else.
            let _ = fs::remove_file(&tmp_path);
        })
}

/// Resolves `path` as the file system does, and fails when it follows a symbolic link in the
/// project directory (the current directory) to a place outside it. Fix writes the files of a
/// project at paths resolved here, so a link committed to a repository, such as `.fixlang/cache`
/// or `docs`, cannot make fix write outside the project. A link outside the project belongs to the
/// user, and is followed wherever it leads.
///
/// The components from the first missing one onward are joined as they are, which is what
/// `create_dir_all` makes of them. The result is an absolute path.
///
/// # Examples
/// In a project at `/p`, `docs/Main.md` resolves to `/p/docs/Main.md`. With `docs` a link to
/// `/p/site`, it resolves to `/p/site/Main.md`, and with `docs` a link to `/home/u`, it fails.
pub fn resolve_guarding_project_links(path: &Path) -> Result<PathBuf, Errors> {
    let failed = |e: io::Error| {
        Errors::from_msg(format!(
            "Failed to resolve \"{}\": {}",
            path.to_string_lossy(),
            e
        ))
    };
    let project = project_dir().map_err(failed)?;
    // `resolved` holds no symbolic link up to its first missing component, so that popping its last
    // component reaches the directory `..` names. Past a missing component, `..` is taken as it is
    // spelled.
    let mut resolved = project.clone();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => resolved.push(component),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(name) => {
                let next = resolved.join(name);
                let is_link = match fs::symlink_metadata(&next) {
                    Ok(meta) => meta.is_symlink(),
                    Err(e) if e.kind() == ErrorKind::NotFound => false,
                    Err(e) => return Err(failed(e)),
                };
                if !is_link {
                    resolved = next;
                    continue;
                }
                let target = next.canonicalize().map_err(failed)?;
                if resolved.starts_with(&project) && !target.starts_with(&project) {
                    return Err(Errors::from_msg(format!(
                        "Refused to write \"{}\": \"{}\" is a symbolic link in the project leading outside it, to \"{}\".\n\
                         HINT: Remove the link, and fix writes in its place. To write where the link leads, name that place on the command line, as `fix docs --out-dir <dir>` does.",
                        path.to_string_lossy(),
                        next.to_string_lossy(),
                        target.to_string_lossy()
                    )));
                }
                resolved = target;
            }
        }
    }
    Ok(resolved)
}

/// Creates the directory at `path` and its missing ancestors, and returns the absolute path of the
/// directory, with symbolic links resolved. Fails when `path` follows a symbolic link in the project
/// directory (the current directory) to a place outside it.
pub fn create_dir_guarding_project_links(path: &Path) -> Result<PathBuf, Errors> {
    let dir = resolve_guarding_project_links(path)?;
    fs::create_dir_all(&dir).map_err(|e| {
        Errors::from_msg(format!(
            "Failed to create directory \"{}\": {}",
            path.to_string_lossy(),
            e
        ))
    })?;
    Ok(dir)
}

/// Writes the file at `path` by having `write` write a new file at the path it is handed, and
/// renaming that onto `path`. Creates the parent directory of `path` when it is missing, and fails
/// when `path` follows a symbolic link in the project directory (the current directory) to a place
/// outside it. A symbolic link at `path` itself is replaced by the new file, and what it pointed to
/// is kept. Returns the absolute path of the file written, with symbolic links resolved. Panics
/// when `path` names no file, as `..` does.
pub fn write_file_guarding_project_links(
    path: &Path,
    write: impl FnOnce(&Path) -> io::Result<()>,
) -> Result<PathBuf, Errors> {
    let failed = |e: io::Error| {
        Errors::from_msg(format!(
            "Failed to write \"{}\": {}",
            path.to_string_lossy(),
            e
        ))
    };
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        panic!("\"{}\" names no file to write.", path.to_string_lossy());
    };
    let file = create_dir_guarding_project_links(parent)?.join(name);
    replace_file(&file, write).map_err(failed)?;
    Ok(file)
}
