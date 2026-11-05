//! Writing and removing the files of a project, in such a way that what the project directory
//! holds, such as a symbolic link committed to a repository, cannot lead fix to write or remove
//! anything outside the project.

use crate::{constants::DOT_FIXLANG, error::Errors};
use rand::random;
use std::{
    env,
    fs::{self, File},
    io::{self, ErrorKind},
    path::{Component, Path, PathBuf},
    time::Duration,
};

/// The error reporting that an operation, named by `verb`, failed on `path` with `e`.
///
/// # Examples
/// `io_failure("write", Path::new("a.txt"))` turns a "Permission denied" error into
/// `Failed to write "a.txt": Permission denied (os error 13)`.
fn io_failure<'a>(verb: &'a str, path: &'a Path) -> impl Fn(io::Error) -> Errors + Copy + 'a {
    move |e| {
        Errors::from_msg(format!(
            "Failed to {} \"{}\": {}",
            verb,
            path.to_string_lossy(),
            e
        ))
    }
}

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

/// The `HINT:` line of an error that refuses a symbolic link under `.fixlang`, a directory that fix
/// creates again when it is missing.
pub const DOT_FIXLANG_LINK_HINT: &str =
    "`fix clean` removes the directory \".fixlang\", the link included, and fix creates it again.";

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
    let failed = io_failure("remove", path);
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
            "Refused to remove \"{}\", which lies outside \"{}\", where a symbolic link under it leads.\nHINT: {}",
            path.to_string_lossy(),
            dot_fixlang.to_string_lossy(),
            DOT_FIXLANG_LINK_HINT
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

/// The name of a new temporary file. It begins with a dot and carries a random number drawn by this
/// call, so that it names no file a project or another call has, and is no longer than the names
/// the file system allows.
///
/// # Examples
///
/// `.3f2a9c0d81b4e675.tmp`
fn temporary_file_name() -> String {
    format!(".{:016x}.tmp", random::<u64>())
}

/// Whether `name` is one `temporary_file_name` gives.
fn is_temporary_file_name(name: &str) -> bool {
    name.strip_prefix('.')
        .and_then(|name| name.strip_suffix(".tmp"))
        .is_some_and(|digits| digits.len() == 16 && digits.chars().all(|c| c.is_ascii_hexdigit()))
}

/// How long a temporary file (`create_temporary_file_in`) goes unmodified before it is taken for
/// one a build left when it was killed. A build puts each temporary file it writes in place, or
/// removes it, before it ends, so a file this old belongs to no build still running.
const ABANDONED_TEMPORARY_FILE_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// A temporary file `create_temporary_file_in` created.
pub struct TemporaryFile {
    /// The file, open for reading and writing.
    pub file: File,
    /// Where the file lies.
    pub path: PathBuf,
}

/// Creates a temporary file in `dir`, named by `temporary_file_name`.
pub fn create_temporary_file_in(dir: &Path) -> io::Result<TemporaryFile> {
    let path = dir.join(temporary_file_name());
    let file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)?;
    Ok(TemporaryFile { file, path })
}

/// Removes the temporary files in `dir` (`create_temporary_file_in`) that have gone unmodified for
/// `ABANDONED_TEMPORARY_FILE_AGE`.
pub fn remove_abandoned_temporary_files(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !is_temporary_file_name(&entry.file_name().to_string_lossy()) {
            continue;
        }
        let abandoned = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| {
                modified
                    .elapsed()
                    .is_ok_and(|age| age >= ABANDONED_TEMPORARY_FILE_AGE)
            });
        if abandoned {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Writes the file at `path` by having `write` write a new file beside it, at the path it is handed,
/// and renaming that onto `path`, so that `path` holds either what it held before or the whole of
/// what `write` wrote. A symbolic link at `path` is replaced by the new file, and what it pointed to
/// is kept.
///
/// The new file is named apart from `path`, so that a name of `path` as long as the file system
/// allows still leaves room for it.
pub fn replace_file(path: &Path, write: impl FnOnce(&Path) -> io::Result<()>) -> io::Result<()> {
    let tmp_path = path.with_file_name(temporary_file_name());
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
/// # Arguments
/// * `hint` — the `HINT:` line of the error that refuses a link: what the user can do about a link
///   at the place the caller writes, such as `DOT_FIXLANG_LINK_HINT` for a place under `.fixlang`.
///
/// # Examples
/// In a project at `/p`, `docs/Main.md` resolves to `/p/docs/Main.md`. With `docs` a link to
/// `/p/site`, it resolves to `/p/site/Main.md`, and with `docs` a link to `/home/u`, it fails.
pub fn resolve_guarding_project_links(path: &Path, hint: &str) -> Result<PathBuf, Errors> {
    let failed = io_failure("resolve", path);
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
                         HINT: {}",
                        path.to_string_lossy(),
                        next.to_string_lossy(),
                        target.to_string_lossy(),
                        hint
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
/// directory (the current directory) to a place outside it, with `hint` as the `HINT:` line, as
/// `resolve_guarding_project_links` does.
pub fn create_dir_guarding_project_links(path: &Path, hint: &str) -> Result<PathBuf, Errors> {
    let dir = resolve_guarding_project_links(path, hint)?;
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
/// when `path` names no file, as `..` does. `hint` is the `HINT:` line of a refusal, as in
/// `resolve_guarding_project_links`.
pub fn write_file_guarding_project_links(
    path: &Path,
    hint: &str,
    write: impl FnOnce(&Path) -> io::Result<()>,
) -> Result<PathBuf, Errors> {
    let failed = io_failure("write", path);
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        panic!("\"{}\" names no file to write.", path.to_string_lossy());
    };
    let file = create_dir_guarding_project_links(parent, hint)?.join(name);
    replace_file(&file, write).map_err(failed)?;
    Ok(file)
}

/// Resolves the symbolic links and `..` in the path of the existing file `path`, and returns the
/// result when it lies in the project directory (the current directory), and `None` when it lies
/// outside.
///
/// # Examples
/// In a project at `/p`, `src/main.fix` answers `Some("/p/src/main.fix")`, and `../lib/lib.fix`,
/// or `src/lib.fix` being a symbolic link to `/q/lib.fix`, answers `None`.
pub fn resolve_file_inside_project(path: &Path) -> Result<Option<PathBuf>, Errors> {
    let failed = io_failure("resolve", path);
    let project = project_dir().map_err(failed)?;
    let file = path.canonicalize().map_err(failed)?;
    Ok(file.starts_with(&project).then_some(file))
}
