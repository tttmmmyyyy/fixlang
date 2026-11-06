// What the files of `typed_modules` and of `parsed_modules` share: how a file is named, how it ends,
// how it is put in place, and how it is read.
//
// A cache file holds serialized entries, then an index of them, then a footer:
//
//   [entry] [entry] ... [index] [footer]
//
// The footer says where the index lies, and ends in eight bytes that tell which kind of cache file
// it is. A file is written to a temporary file in its directory and renamed over its name, and a
// build reads it through a mapping into memory, so a build that replaces or removes a file leaves
// the bytes another build has mapped as they were.

use crate::constants::CACHE_PATH;
use crate::misc::{warn_msg, Map};
use crate::project_fs::{
    create_dir_guarding_project_links, create_temporary_file_in, remove_abandoned_temporary_files,
    TemporaryFile, DOT_FIXLANG_LINK_HINT,
};
use memmap2::Mmap;
use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The eight bytes a cache file ends in, which tell its kind.
pub type FileMagic = [u8; 8];

/// The length of the footer: the offset of the index, its length, and the magic.
const FOOTER_LEN: u64 = 8 + 8 + size_of::<FileMagic>() as u64;

/// How many characters of the name a file is named after its file name carries. With the two
/// digests the file name stays well within the 255 bytes file systems allow.
const READABLE_NAME_LEN: usize = 100;

/// The part every file named after `name` begins its name with, whatever the digest after it.
///
/// The digest of `name` is what tells the files of two names apart. The part in front of it shows
/// the name to someone reading the directory, and is filename-safe because it keeps only ASCII
/// alphanumeric characters; several names can wear the same one.
///
/// # Examples
///
/// The files named after `Main.Sub` are named `Main_Sub_<digest of the name>_<digest>`.
pub fn readable_file_prefix(name: &str) -> String {
    let readable: String = name
        .chars()
        .take(READABLE_NAME_LEN)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{}_{:x}_", readable, md5::compute(name))
}

/// Where the index lies in a file of `len` bytes that ends in `footer`, or `None` when the footer is
/// not one a cache file ending in `magic` ends in.
fn index_range(footer: &[u8; FOOTER_LEN as usize], len: u64, magic: &FileMagic) -> Option<Range<u64>> {
    assert!(
        len >= FOOTER_LEN,
        "a footer of {} bytes in a file of {} bytes",
        FOOTER_LEN,
        len
    );
    if &footer[16..] != magic {
        return None;
    }
    let offset = u64::from_le_bytes(footer[0..8].try_into().unwrap());
    let index_len = u64::from_le_bytes(footer[8..16].try_into().unwrap());
    if offset.checked_add(index_len) != Some(len - FOOTER_LEN) {
        return None;
    }
    Some(offset..offset + index_len)
}

/// Where the index of the cache file `bytes` lies; its entries end where it begins. Bytes that are
/// not a cache file ending in `magic` answer `None`.
pub fn find_index(bytes: &[u8], magic: &FileMagic) -> Option<Range<u64>> {
    let len = bytes.len() as u64;
    if len < FOOTER_LEN {
        return None;
    }
    let footer = bytes[(len - FOOTER_LEN) as usize..].try_into().unwrap();
    index_range(footer, len, magic)
}

/// Whether `range`, which an index places an entry at, lies within the entries, which end at
/// `entries_end`.
pub fn lies_within_entries(range: &Range<u64>, entries_end: u64) -> bool {
    range.start <= range.end && range.end <= entries_end
}

/// The file at `path` mapped into memory, or `None` when there is no file there.
pub fn map_file(path: &Path) -> io::Result<Option<Mmap>> {
    match File::open(path) {
        Ok(file) => map_into_memory(&file).map(Some),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// `file` mapped into memory. `file` must be a cache file, or a temporary file that becomes one,
/// since the mapping stays sound only while nothing writes the file in place.
fn map_into_memory(file: &File) -> io::Result<Mmap> {
    // SAFETY: the mapping is read for as long as the build runs, and a cache file is never written
    // in place: a build writes a new version under another name and renames it over the file, or
    // removes the file, and either leaves the mapped bytes as they were. A read of a page that
    // cannot be loaded stops the process with SIGBUS: a file truncated in place by another program,
    // an I/O error of the disk, or a file that a build on another host removed from a network file
    // system.
    unsafe { Mmap::map(file) }
}

/// A new cache file, written to a temporary file in its directory and put in place by `finish`.
/// A writer dropped before `finish` removes the temporary file.
pub struct CacheFileWriter {
    /// The directory the file is written in.
    dir: PathBuf,
    /// The temporary file the new file is written to, until `finish` renames it.
    tmp: Option<TemporaryFile>,
}

impl CacheFileWriter {
    /// Begins a new cache file in `dir`.
    pub fn create(dir: &Path) -> io::Result<Self> {
        Ok(CacheFileWriter {
            dir: dir.to_path_buf(),
            tmp: Some(create_temporary_file_in(dir)?),
        })
    }

    /// Writes `bytes` at `offset`.
    pub fn write_at(&mut self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        let tmp = &mut self
            .tmp
            .as_mut()
            .expect("a cache file written after it was finished")
            .file;
        tmp.seek(SeekFrom::Start(offset))?;
        tmp.write_all(bytes)
    }

    /// Writes `index` at `entries_end`, where the entries end, and the footer after it, renames the
    /// file to `file_name` in its directory, and answers the file mapped into memory. Then removes
    /// the other files whose names begin with `prefix`, and the temporary files left in the
    /// directory by builds that were killed.
    pub fn finish(
        mut self,
        entries_end: u64,
        index: &[u8],
        magic: &FileMagic,
        file_name: &str,
        prefix: &str,
    ) -> io::Result<Mmap> {
        let mut tail = index.to_vec();
        tail.extend_from_slice(&entries_end.to_le_bytes());
        tail.extend_from_slice(&(index.len() as u64).to_le_bytes());
        tail.extend_from_slice(magic);
        self.write_at(entries_end, &tail)?;
        let tmp = self.tmp.take().unwrap();
        let bytes = map_into_memory(&tmp.file)?;

        // The name holds no path separator, and a rename replaces a symbolic link at the path
        // instead of writing through it, so the file is written in `dir`.
        if let Err(e) = fs::rename(&tmp.path, self.dir.join(file_name)) {
            let _ = fs::remove_file(&tmp.path);
            return Err(e);
        }
        remove_stale_files(&self.dir, prefix, file_name);
        Ok(bytes)
    }
}

impl Drop for CacheFileWriter {
    /// Removes the temporary file of a cache file that was not finished.
    fn drop(&mut self) {
        if let Some(tmp) = self.tmp.take() {
            let _ = fs::remove_file(&tmp.path);
        }
    }
}

/// Removes the files in `dir` whose names begin with `prefix`, except the one named `file_name`,
/// and the temporary files left in `dir` by builds that were killed. The file named `file_name` is
/// in place already, so a file this fails to remove costs only disk space, and the next write tries
/// again.
fn remove_stale_files(dir: &Path, prefix: &str, file_name: &str) {
    remove_abandoned_temporary_files(dir);
    let Ok(dir_entries) = fs::read_dir(dir) else {
        return;
    };
    // The temporary files begin with a dot, which no prefix does.
    for dir_entry in dir_entries.flatten() {
        let entry_name = dir_entry.file_name();
        let entry_name = entry_name.to_string_lossy();
        if entry_name.starts_with(prefix) && entry_name != file_name {
            let _ = fs::remove_file(dir.join(entry_name.as_ref()));
        }
    }
}

/// The directories of the cache files of one build, each made ready the first time it is asked for
/// (`writable_dir`), and answered from then on as it was found then. Clones share what they found.
#[derive(Clone, Default)]
pub struct CacheDirs(Arc<Mutex<Map<&'static str, Option<PathBuf>>>>);

impl CacheDirs {
    /// The directory `path` under `CACHE_PATH`, created when it is missing, when files can be
    /// created in it; `None` otherwise, which is reported as a warning the first time.
    ///
    /// `CACHE_PATH` itself is made ready first, so that a `CACHE_PATH` the build may not write
    /// through is reported once for all the directories under it.
    pub fn writable_dir(&self, path: &'static str) -> Option<PathBuf> {
        assert!(
            Path::new(path).parent() == Some(Path::new(CACHE_PATH)),
            "\"{}\" is not a directory of `CACHE_PATH`",
            path
        );
        self.made_ready(CACHE_PATH, create_cache_dir)?;
        self.made_ready(path, |path| {
            create_cache_dir(path).filter(|dir| probe_file_creation(dir))
        })
    }

    /// The directory `path` as `make_ready` made it ready, the first time it is asked for.
    fn made_ready(
        &self,
        path: &'static str,
        make_ready: impl FnOnce(&str) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let mut dirs = self.0.lock().unwrap();
        dirs.entry(path).or_insert_with(|| make_ready(path)).clone()
    }
}

/// The directory `path`, created when it is missing. A directory that cannot be created is reported
/// as a warning, and answers `None`.
fn create_cache_dir(path: &str) -> Option<PathBuf> {
    match create_dir_guarding_project_links(Path::new(path), DOT_FIXLANG_LINK_HINT) {
        Ok(dir) => Some(dir),
        Err(errs) => {
            for err in errs.errors() {
                warn_msg(&err.msg);
            }
            None
        }
    }
}

/// Whether a file can be created in `dir`. When it cannot, that is reported as a warning.
fn probe_file_creation(dir: &Path) -> bool {
    match create_temporary_file_in(dir) {
        Ok(probe) => {
            let _ = fs::remove_file(&probe.path);
            true
        }
        Err(e) => {
            warn_msg(&format!(
                "Failed to write cache files in \"{}\": {}.",
                dir.to_string_lossy(),
                e
            ));
            false
        }
    }
}

/// Reports, as a warning, that the cache file at `path` could not be written for `e`.
pub fn warn_unwritable_file(path: &Path, e: &io::Error) {
    warn_msg(&format!(
        "Failed to write cache file \"{}\": {}.",
        path.to_string_lossy(),
        e
    ));
}

/// Reports, as a warning, that the cache file at `path` could not be read for `why`.
pub fn warn_unreadable_file(path: &Path, why: &str) {
    warn_msg(&format!(
        "Failed to read cache file \"{}\": {}.",
        path.to_string_lossy(),
        why
    ));
}
