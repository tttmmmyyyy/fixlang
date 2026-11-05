// The type-check results of `fix build` and `fix check`, kept in one file for each module under
// `TYPED_MODULES_PATH`.
//
// A module's file is named after the module and its `module_dependency_hash`, so a file answers
// for exactly the sources and settings its module is checked from. It holds the typed expression of
// every body of the module that passed a type check producing no diagnostic, and a record of what
// later phases read off each of them:
//
//   [identity, typed expression] [identity, typed expression] ... [record] [footer]
//
// The record lists each body by its `EntityIdentity`, with where the body lies in the file, the
// concrete types of its opaque types, and its `WarningFact`s. The footer says where the record lies.
//
// A build reads the record of every module and type-checks only the bodies the record lacks. It
// reads a typed expression back only when instantiation asks for it. When the check passes bodies
// the record lacked, the file is written anew: the bodies it holds are copied as they are, at the
// same offsets, and each new body follows them as soon as it passes, so that it leaves memory then
// (`ModuleFileWriter`). When instantiation finds a body that the file fails to hold, it checks the
// body again and writes it the same way, after the bodies the file holds.
//
// The record is valid because a body is determined by its `EntityIdentity` together with what the
// module's `module_dependency_hash` covers. The set of bodies a module has can still differ between
// two builds under one hash (the implementations `Std` carries for tuples follow the sizes the
// program uses), which is why a build looks up each body in the record.

use crate::ast::expr::ExprNode;
use crate::ast::name::{FullName, Name};
use crate::ast::pattern::Pattern;
use crate::ast::program::{BodyLocation, CheckedBody, Program, TypedExpr};
use crate::ast::types::{OpaqueTyConResolution, TyCon};
use crate::configuration::Configuration;
use crate::constants::TYPED_MODULES_PATH;
use crate::elaboration::typecheck::TypeCheckContext;
use crate::elaboration::typecheckcache::EntityIdentity;
use crate::error::Errors;
use crate::misc::{consume_on_compiler_threads, map_on_compiler_threads, warn_msg, Map};
use crate::parse::sourcefile::{SourceFile, Span};
use crate::project_fs::{
    create_dir_guarding_project_links, create_temporary_file_in, remove_abandoned_temporary_files,
    TemporaryFile, DOT_FIXLANG_LINK_HINT,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::take;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A fact about a typed expression that a warning after type checking may be derived from,
/// whatever the configuration. Whether a fact is reported depends on the configuration and on the
/// declarations, so it is decided from the facts at each build (`Program::collect_diagnostics_of_typed_program`).
#[derive(Clone, Serialize, Deserialize)]
pub enum WarningFact {
    /// The expression uses the global value `name` at `span`.
    GlobalUse { name: FullName, span: Span },
    /// A struct pattern at `span` names the fields `written` of `tycon`, and writes no `_` after
    /// them.
    StructPattern {
        span: Span,
        tycon: Arc<TyCon>,
        written: Vec<Name>,
    },
}

/// The facts about `expr` a warning after type checking may be derived from, in the order the
/// expression holds them.
pub fn warning_facts(expr: &Arc<ExprNode>) -> Vec<WarningFact> {
    let mut facts = vec![];
    expr.walk_var_uses(&mut |var, src| {
        // A use with no source location is synthetic (compiler-generated wrappers, builtin
        // bridges) and not something the user can act on.
        if let (true, Some(span)) = (var.name.is_global(), src) {
            facts.push(WarningFact::GlobalUse {
                name: var.name.clone(),
                span: span.clone(),
            });
        }
    });
    expr.walk_patterns(&mut |pat| {
        pat.walk_nodes(&mut |node| {
            let Pattern::Struct(tycon, fields, false) = &node.pattern else {
                return;
            };
            let Some(span) = &node.info.source else {
                return;
            };
            facts.push(WarningFact::StructPattern {
                span: span.clone(),
                tycon: tycon.clone(),
                written: fields.iter().map(|(name, _, _)| name.clone()).collect(),
            });
        });
    });
    facts
}

/// What a module's file records of one body.
#[derive(Clone, Serialize, Deserialize)]
struct EntityEntry {
    /// Where the body's identity and typed expression lie in the file.
    body: Range<u64>,
    /// The concrete type each opaque type constructor of the body stands for.
    opaque_types: Map<FullName, Vec<OpaqueTyConResolution>>,
    /// The facts about the body a warning after type checking may be derived from.
    warning_facts: Vec<WarningFact>,
}

/// The bodies a module's file holds, by their identity.
type ModuleRecord = Map<EntityIdentity, EntityEntry>;

/// The last bytes of a module's file: they mark the file as a file of type-checked modules.
const FILE_MAGIC: &[u8; 8] = b"FIXTYMOD";

/// The length of the footer: the offset of the record, its length, and `FILE_MAGIC`.
const FOOTER_LEN: u64 = 8 + 8 + FILE_MAGIC.len() as u64;

/// How many characters of a module's name the name of its file carries. With the two digests the
/// file name stays well within the 255 bytes file systems allow.
const READABLE_NAME_LENGTH: usize = 100;

/// The part every file of `module` begins its name with, whatever its hash.
///
/// The digest of the module's name is what tells the files of two modules apart. The part in front
/// of it names the module for someone reading the directory, and is filename-safe because it keeps
/// only ASCII alphanumeric characters; several modules can wear the same one.
///
/// # Examples
///
/// The files of `Main.Sub` are named `Main_Sub_<digest of the name>_<digest of the hash>`.
fn module_file_prefix(module: &Name) -> String {
    let readable: String = module
        .chars()
        .take(READABLE_NAME_LENGTH)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{}_{:x}_", readable, md5::compute(module))
}

/// The name of the file holding the bodies of `module` checked under `module_hash`, its
/// `module_dependency_hash`.
fn module_file_name(module: &Name, module_hash: &str) -> String {
    format!(
        "{}{:x}",
        module_file_prefix(module),
        md5::compute(format!("{}\n{}", module, module_hash))
    )
}

/// Where the record lies in a file of `len` bytes that ends in `footer`, or `None` when the footer
/// is not one a file of type-checked modules ends in.
fn record_range(footer: &[u8; FOOTER_LEN as usize], len: u64) -> Option<Range<u64>> {
    assert!(
        len >= FOOTER_LEN,
        "a footer of {} bytes in a file of {} bytes",
        FOOTER_LEN,
        len
    );
    if &footer[16..] != FILE_MAGIC {
        return None;
    }
    let offset = u64::from_le_bytes(footer[0..8].try_into().unwrap());
    let record_len = u64::from_le_bytes(footer[8..16].try_into().unwrap());
    if offset.checked_add(record_len) != Some(len - FOOTER_LEN) {
        return None;
    }
    Some(offset..offset + record_len)
}

/// The record serialized as `bytes`. A record that points past `bodies_end`, where the bodies of
/// its file end, answers the reason.
fn decode_record(bytes: &[u8], bodies_end: u64) -> Result<ModuleRecord, String> {
    let entries: Vec<(EntityIdentity, EntityEntry)> =
        postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    if entries
        .iter()
        .any(|(_, entry)| entry.body.start > entry.body.end || entry.body.end > bodies_end)
    {
        return Err("its record points past its bodies".to_string());
    }
    Ok(entries.into_iter().collect())
}

/// The typed expression of a body as a module's file writes it: the identity of the body, then
/// its typed expression. Bytes that are not the body of `identity` answer the reason.
fn decode_body(bytes: &[u8], identity: &EntityIdentity) -> Result<TypedExpr, String> {
    let ((found, expr), rest): ((EntityIdentity, TypedExpr), _) =
        postcard::take_from_bytes(bytes).map_err(|e| e.to_string())?;
    if !rest.is_empty() || &found != identity {
        return Err(format!(
            "it holds another body where the body of `{}` was recorded",
            identity.replace('\n', " : ")
        ));
    }
    Ok(expr)
}

/// The message for a file that is not one of type-checked modules.
const NOT_A_MODULE_FILE: &str = "it is not a file of type-checked modules";

/// The record of the file at `path`. A file that does not exist answers `None`; one that cannot be
/// read, or is not a file of type-checked modules, answers the reason.
///
/// It reads only the footer and the record, so that a build reads only the bodies it asks for
/// later.
fn read_record(path: &Path) -> Result<Option<ModuleRecord>, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    match read_record_of(&mut file) {
        Ok(Ok((record, _bodies_end))) => Ok(Some(record)),
        Ok(Err(why)) => Err(why),
        Err(e) => Err(e.to_string()),
    }
}

/// The record of the module file `file`, and the offset its bodies end at. The inner `Err` holds
/// the reason a file is not one of type-checked modules, and the outer one the error of a file that
/// cannot be read.
fn read_record_of(file: &mut File) -> io::Result<Result<(ModuleRecord, u64), String>> {
    let read = |file: &mut File| -> io::Result<Option<(Vec<u8>, u64)>> {
        let len = file.metadata()?.len();
        if len < FOOTER_LEN {
            return Ok(None);
        }
        let mut footer = [0u8; FOOTER_LEN as usize];
        file.seek(SeekFrom::Start(len - FOOTER_LEN))?;
        file.read_exact(&mut footer)?;
        let Some(range) = record_range(&footer, len) else {
            return Ok(None);
        };
        let mut record_bytes = vec![0u8; (range.end - range.start) as usize];
        file.seek(SeekFrom::Start(range.start))?;
        file.read_exact(&mut record_bytes)?;
        Ok(Some((record_bytes, range.start)))
    };
    Ok(match read(file)? {
        Some((record_bytes, bodies_end)) => {
            decode_record(&record_bytes, bodies_end).map(|record| (record, bodies_end))
        }
        None => Err(NOT_A_MODULE_FILE.to_string()),
    })
}

/// The typed expression of the body `identity` lying at `range` in the file at `path`. A file that
/// no longer exists answers `None`; one whose bytes there are not the body answers the reason.
///
/// A body the file holds intact stays at its offset when this build writes the file anew, so `range`
/// read from an earlier record of the file still finds it. A file another build began on its own
/// and put in place may hold the body elsewhere, which `Program::read_stored_body` handles. The
/// identity written in front of the body is compared with `identity`, so bytes of another body are
/// never taken for it.
fn read_body(
    path: &Path,
    range: &Range<u64>,
    identity: &EntityIdentity,
) -> Result<Option<TypedExpr>, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = vec![0u8; (range.end - range.start) as usize];
    file.seek(SeekFrom::Start(range.start))
        .and_then(|_| file.read_exact(&mut bytes))
        .map_err(|e| e.to_string())?;
    decode_body(&bytes, identity).map(Some)
}

/// A body passed by this build's type check, to be added to its module's file.
struct NewEntry {
    /// The entity the body belongs to, which the record lists the body by.
    identity: EntityIdentity,
    /// The identity and the typed expression, serialized as the file holds them.
    bytes: Vec<u8>,
    /// The concrete type each opaque type constructor of the body stands for.
    opaque_types: Map<FullName, Vec<OpaqueTyConResolution>>,
    /// The facts about the body a warning after type checking may be derived from.
    warning_facts: Vec<WarningFact>,
}

impl NewEntry {
    /// The entry of the body of `identity`, from the check `checked` that the body passed.
    fn of(identity: EntityIdentity, checked: &CheckedBody) -> Self {
        assert!(
            checked.passed(),
            "a cache entry for {:?}, a body that did not pass",
            identity
        );
        NewEntry {
            bytes: postcard::to_allocvec(&(&identity, &checked.te)).unwrap(),
            identity,
            opaque_types: checked.opaque_types.clone(),
            warning_facts: warning_facts(&checked.te.expr),
        }
    }
}

/// The file of a module checked under one `module_dependency_hash`.
struct ModuleFile {
    /// The module whose bodies the file holds.
    module: Name,
    /// The `module_dependency_hash` of `module`.
    module_hash: String,
}

impl ModuleFile {
    /// The name of the file, without its directory.
    fn name(&self) -> String {
        module_file_name(&self.module, &self.module_hash)
    }

    /// The path of the file, under `TYPED_MODULES_PATH`.
    fn path(&self) -> PathBuf {
        self.path_in(Path::new(TYPED_MODULES_PATH))
    }

    /// The path of the file in the directory `dir`.
    fn path_in(&self, dir: &Path) -> PathBuf {
        dir.join(self.name())
    }
}

/// The new version of a module's file, written while the bodies of the module are checked. It begins
/// with the bodies the file held, at the offsets they had, and each body `add`ed follows them;
/// `finish` puts the record and the footer after the bodies and replaces the file with it.
///
/// The bodies go to a temporary file beside the module's file as they are added, so that a body is
/// dropped from memory as soon as it is written. A writer dropped before `finish` removes the
/// temporary file, and the module's file stays as it was.
struct ModuleFileWriter {
    /// The directory the files of type-checked modules are written in.
    dir: PathBuf,
    /// The module file the new version replaces.
    file: Arc<ModuleFile>,
    /// The temporary file the new version is written to, until `finish` renames it.
    tmp: Option<TemporaryFile>,
    /// The bodies the new version holds so far.
    record: ModuleRecord,
    /// Where the bodies written so far end, which is where the next body goes.
    bodies_end: u64,
}

impl ModuleFileWriter {
    /// Begins a new version of `file` in `dir`, with the bodies the file holds now, so that the
    /// bodies another build added to it since this build read its record are kept. A file that is
    /// not one of type-checked modules is begun anew, and one that cannot be read answers the error.
    fn create(dir: &Path, file: Arc<ModuleFile>) -> io::Result<Self> {
        let path = file.path_in(dir);
        let tmp = create_temporary_file_in(dir)?;
        let mut writer = ModuleFileWriter {
            dir: dir.to_path_buf(),
            file,
            tmp: Some(tmp),
            record: ModuleRecord::default(),
            bodies_end: 0,
        };
        let mut held = match File::open(&path) {
            Ok(held) => held,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(writer),
            Err(e) => return Err(e),
        };
        let Ok((record, bodies_end)) = read_record_of(&mut held)? else {
            return Ok(writer);
        };
        held.seek(SeekFrom::Start(0))?;
        let copied = io::copy(&mut held.take(bodies_end), writer.tmp_file())?;
        if copied != bodies_end {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the file ended before its bodies did",
            ));
        }
        writer.record = record;
        writer.bodies_end = bodies_end;
        Ok(writer)
    }

    /// The temporary file the new version is written to.
    fn tmp_file(&mut self) -> &mut File {
        &mut self
            .tmp
            .as_mut()
            .expect("a module file written after it was finished")
            .file
    }

    /// Adds `entry` after the bodies written so far. A body the file held is kept as it is when its
    /// bytes decode to it, and is replaced by `entry` otherwise.
    ///
    /// The bytes of a body the file held are decoded with the source files their spans point into
    /// taken from `source_files`, which gains the paths it lacks.
    fn add(
        &mut self,
        entry: NewEntry,
        source_files: &mut Map<PathBuf, SourceFile>,
    ) -> io::Result<()> {
        if let Some(held) = self.record.get(&entry.identity) {
            let mut held_bytes = vec![0u8; (held.body.end - held.body.start) as usize];
            let start = held.body.start;
            let tmp = self.tmp_file();
            tmp.seek(SeekFrom::Start(start))?;
            tmp.read_exact(&mut held_bytes)?;
            let intact = SourceFile::with_shared_source_files(source_files, || {
                decode_body(&held_bytes, &entry.identity)
            })
            .is_ok();
            if intact {
                return Ok(());
            }
        }
        let start = self.bodies_end;
        let tmp = self.tmp_file();
        tmp.seek(SeekFrom::Start(start))?;
        tmp.write_all(&entry.bytes)?;
        self.bodies_end = start + entry.bytes.len() as u64;
        self.record.insert(
            entry.identity,
            EntityEntry {
                body: start..self.bodies_end,
                opaque_types: entry.opaque_types,
                warning_facts: entry.warning_facts,
            },
        );
        Ok(())
    }

    /// Puts the record and the footer after the bodies, replaces the module's file with the new
    /// version, removes the files of the module checked under any other hash, and answers the
    /// record the file holds now.
    fn finish(mut self) -> io::Result<ModuleRecord> {
        let mut entries: Vec<(EntityIdentity, EntityEntry)> =
            take(&mut self.record).into_iter().collect();
        entries.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
        let mut tail = postcard::to_allocvec(&entries).unwrap();
        let record_len = tail.len() as u64;
        tail.extend_from_slice(&self.bodies_end.to_le_bytes());
        tail.extend_from_slice(&record_len.to_le_bytes());
        tail.extend_from_slice(FILE_MAGIC);
        let bodies_end = self.bodies_end;
        let tmp = self.tmp_file();
        tmp.seek(SeekFrom::Start(bodies_end))?;
        tmp.write_all(&tail)?;

        // The file stays open, and so locked, until it is in place.
        let tmp = self.tmp.take().unwrap();
        let file_name = self.file.name();
        // The name holds no path separator, and a rename replaces a symbolic link at the path
        // instead of writing through it, so the file is written in `dir`.
        if let Err(e) = fs::rename(&tmp.path, self.dir.join(&file_name)) {
            let _ = fs::remove_file(&tmp.path);
            return Err(e);
        }
        drop(tmp);

        self.remove_stale_files(&file_name);
        Ok(entries.into_iter().collect())
    }

    /// Removes the files of the module checked under any other hash than the file named
    /// `file_name`, and the temporary files builds that were killed left in the directory. The file
    /// is in place already, so a file left behind costs only room on the disk, and the next write
    /// tries again.
    ///
    /// A temporary file another build created a moment ago may be unlocked still
    /// (`remove_abandoned_temporary_files`); removing it makes that build fail to put its file in
    /// place, which it reports, and its bodies are checked again.
    fn remove_stale_files(&self, file_name: &str) {
        remove_abandoned_temporary_files(&self.dir);
        let Ok(dir_entries) = fs::read_dir(&self.dir) else {
            return;
        };
        // The temporary files begin with a dot, which no prefix does.
        let prefix = module_file_prefix(&self.file.module);
        for dir_entry in dir_entries.flatten() {
            let entry_name = dir_entry.file_name();
            let entry_name = entry_name.to_string_lossy();
            if entry_name.starts_with(&prefix) && entry_name != file_name {
                let _ = fs::remove_file(self.dir.join(entry_name.as_ref()));
            }
        }
    }
}

impl Drop for ModuleFileWriter {
    /// Removes the temporary file of a new version that was not finished.
    fn drop(&mut self) {
        if let Some(tmp) = self.tmp.take() {
            let _ = fs::remove_file(&tmp.path);
        }
    }
}

/// The directory the files of type-checked modules are written in, `TYPED_MODULES_PATH`, created
/// when it is missing. A directory that cannot be created is reported as a warning, and answers
/// `None`.
fn create_typed_modules_dir() -> Option<PathBuf> {
    match create_dir_guarding_project_links(Path::new(TYPED_MODULES_PATH), DOT_FIXLANG_LINK_HINT) {
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
fn accepts_new_files(dir: &Path) -> bool {
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

/// Reports, as a warning, that the file of `file` in `dir` could not be written for `e`.
fn warn_unwritable_file(dir: &Path, file: &ModuleFile, e: &io::Error) {
    warn_msg(&format!(
        "Failed to write cache file \"{}\": {}.",
        file.path_in(dir).to_string_lossy(),
        e
    ));
}

/// Reports, as a warning, that the file at `path` could not be read for `why`.
fn warn_unreadable_file(path: &Path, why: &str) {
    warn_msg(&format!(
        "Failed to read cache file \"{}\": {}.",
        path.to_string_lossy(),
        why
    ));
}

/// The writing of one module's file while the bodies of the module are checked.
struct ModuleWriting {
    /// The new version of the file, or why writing it failed once it has.
    writer: Result<ModuleFileWriter, io::Error>,
    /// The bodies written to `writer`, each with its index among the bodies checked, its identity,
    /// and the names it referred to that the module it is defined in has to import.
    written: Vec<(usize, EntityIdentity, Map<Name, Vec<FullName>>)>,
    /// The bodies that passed and that the file will not hold, by their index among the bodies
    /// checked.
    unwritten: Vec<usize>,
}

/// What became of the bodies of a module `ModuleWriting` wrote.
struct WrittenModule {
    /// The bodies the file holds, each with its index among the bodies checked.
    stored: Vec<(usize, StoredOutcome)>,
    /// The bodies that passed and that the file does not hold, by their index among the bodies
    /// checked.
    unwritten: Vec<usize>,
    /// Why writing the file failed, when it did.
    failure: Option<io::Error>,
}

impl ModuleWriting {
    /// Begins writing the file of `file` in `dir`.
    fn begin(dir: &Path, file: Arc<ModuleFile>) -> Self {
        ModuleWriting {
            writer: ModuleFileWriter::create(dir, file),
            written: vec![],
            unwritten: vec![],
        }
    }

    /// Writes `body`, the body at index `i` among the bodies checked, to the file. When a write
    /// fails, the bodies written before it, `body` with them, are left unwritten.
    fn add(&mut self, i: usize, body: PassedBody, source_files: &mut Map<PathBuf, SourceFile>) {
        let Ok(writer) = &mut self.writer else {
            self.unwritten.push(i);
            return;
        };
        let identity = body.entry.identity.clone();
        match writer.add(body.entry, source_files) {
            Ok(()) => self.written.push((i, identity, body.import_required)),
            Err(e) => {
                self.writer = Err(e);
                self.unwritten
                    .extend(self.written.drain(..).map(|(i, _, _)| i));
                self.unwritten.push(i);
            }
        }
    }

    /// Finishes the file. When that fails, the bodies written to it are left unwritten.
    fn finish(self) -> WrittenModule {
        let mut unwritten = self.unwritten;
        let writer = match self.writer {
            Ok(writer) => writer,
            Err(e) => {
                return WrittenModule {
                    stored: vec![],
                    unwritten,
                    failure: Some(e),
                }
            }
        };
        match writer.finish() {
            Ok(mut record) => {
                let stored = self
                    .written
                    .into_iter()
                    .map(|(i, identity, import_required)| {
                        let entry = record
                            .remove(&identity)
                            .expect("the file written lacks a body written to it");
                        (
                            i,
                            StoredOutcome {
                                entry,
                                import_required,
                            },
                        )
                    })
                    .collect();
                WrittenModule {
                    stored,
                    unwritten,
                    failure: None,
                }
            }
            Err(e) => {
                unwritten.extend(self.written.into_iter().map(|(i, _, _)| i));
                WrittenModule {
                    stored: vec![],
                    unwritten,
                    failure: Some(e),
                }
            }
        }
    }
}

/// A body that passed this build's type check, to be written to the file of its module.
struct PassedBody {
    /// The body, serialized for the file.
    entry: NewEntry,
    /// The names the body referred to that the module it is defined in has to import.
    import_required: Map<Name, Vec<FullName>>,
}

/// What a compiler thread answers for a body it type-checked.
enum CheckedOnThread {
    /// The body passed, and the file of its module is to hold it.
    Passed(PassedBody),
    /// The body is to be held in memory: what `Program::check_body` answered for a body that did
    /// not pass, or for any body when the directory of the files cannot be written in.
    InMemory(Result<CheckedBody, Errors>),
}

/// A body this build checked that passed and that the file of its module now records.
struct StoredOutcome {
    /// What the file records of the body.
    entry: EntityEntry,
    /// The names the body referred to that the module it is defined in has to import.
    import_required: Map<Name, Vec<FullName>>,
}

/// What became of a body this build checked.
enum CheckOutcome {
    /// The body passed, and the file of its module records it.
    Stored(StoredOutcome),
    /// The body is held in memory: what `Program::check_body` answered for a body that did not
    /// pass, or for one that passed and that the file of its module could not take.
    InMemory(Result<CheckedBody, Errors>),
}

/// A body whose typed expression lies in its module's file, and which `Program::global_values`
/// holds as it was parsed.
struct StoredBody {
    /// The file of the module the body is defined in.
    file: Arc<ModuleFile>,
    /// The entity the body belongs to, which the file writes in front of the body.
    identity: EntityIdentity,
    /// Where the body lies in the file.
    range: Range<u64>,
    /// The facts about the body a warning after type checking may be derived from.
    warning_facts: Vec<WarningFact>,
}

/// The bodies of a program whose typed expressions lie in the files of their modules.
#[derive(Default)]
pub struct TypedModules {
    /// The bodies whose typed expressions lie in the files, by where each lies in the program.
    stored: Map<BodyLocation, StoredBody>,
    /// The typed expressions read back so far, so that a value instantiated at several types is
    /// read once.
    read_back: Map<BodyLocation, TypedExpr>,
    /// The source files the spans read back point into, by path.
    source_files: Map<PathBuf, SourceFile>,
}

impl TypedModules {
    /// Takes the ranges of the stored bodies of the file the body at `loc` lies in anew, from the
    /// record the file holds now, and answers whether the range of the body at `loc` changed.
    fn relocate_stored_bodies(&mut self, loc: &BodyLocation) -> bool {
        let file = self.stored[loc].file.clone();
        let path = file.path();
        let Ok(Some(record)) =
            SourceFile::with_shared_source_files(&mut self.source_files, || read_record(&path))
        else {
            return false;
        };
        let mut moved = false;
        for (body_loc, body) in self.stored.iter_mut() {
            if !Arc::ptr_eq(&body.file, &file) {
                continue;
            }
            if let Some(entry) = record.get(&body.identity) {
                if entry.body != body.range {
                    body.range = entry.body.clone();
                    moved |= body_loc == loc;
                }
            }
        }
        moved
    }

    /// The facts a warning after type checking may be derived from about the body at `loc`, when
    /// its typed expression lies in its module's file.
    pub fn stored_warning_facts(&self, loc: &BodyLocation) -> Option<&[WarningFact]> {
        self.stored
            .get(loc)
            .map(|body| body.warning_facts.as_slice())
    }
}

impl Program {
    /// Resolves namespaces in, and type-checks, the bodies of `modules` that the files of
    /// `TYPED_MODULES_PATH` do not record, and adds those that pass to the files.
    ///
    /// A body the file of its module holds, whether recorded before or added here, is left as it
    /// was parsed, and its typed expression is read back when instantiation asks for it
    /// (`typed_body`). The typed expressions of the other bodies are put in place of their bodies.
    pub fn check_bodies_through_typed_modules(
        &mut self,
        tc: &TypeCheckContext,
        modules: &[Name],
        config: &Configuration,
    ) -> Result<(), Errors> {
        let mut errors = Errors::empty();

        // The spans read back share the source files the parser made.
        for module in &self.modules {
            let input = &module.source.input;
            self.typed_modules
                .source_files
                .insert(input.file_path.clone(), input.clone());
        }

        // The bodies of `modules`, by the module each is defined in.
        let mut bodies_by_module: Map<Name, Vec<BodyLocation>> = Map::default();
        for loc in self.body_locations_in_modules(modules, None) {
            let (_te, _scm, module) = self.body_at(&loc);
            bodies_by_module.entry(module).or_default().push(loc);
        }

        // Skip the bodies the files record.
        let mut to_check: Vec<(BodyLocation, Arc<ModuleFile>, EntityIdentity)> = vec![];
        for (module, locs) in bodies_by_module {
            let module_hash = self.module_dependency_hash(&module, config)?;
            let file = Arc::new(ModuleFile {
                module,
                module_hash,
            });
            let path = file.path();
            let record =
                SourceFile::with_shared_source_files(&mut self.typed_modules.source_files, || {
                    read_record(&path)
                });
            let record = match record {
                Ok(Some(record)) => record,
                Ok(None) => ModuleRecord::default(),
                Err(why) => {
                    warn_unreadable_file(&path, &why);
                    ModuleRecord::default()
                }
            };
            for loc in locs {
                let identity = self.entity_identity_at(&loc);
                match record.get(&identity) {
                    Some(entry) => self.store_body(loc, file.clone(), identity, entry.clone()),
                    None => to_check.push((loc, file.clone(), identity)),
                }
            }
        }

        // Check the rest.
        let mut source_files = take(&mut self.typed_modules.source_files);
        let outcomes = self.check_and_write_bodies(&to_check, tc, &mut source_files);
        self.typed_modules.source_files = source_files;
        for ((loc, file, identity), outcome) in to_check.into_iter().zip(outcomes) {
            match outcome {
                CheckOutcome::Stored(StoredOutcome {
                    entry,
                    import_required,
                }) => {
                    self.merge_import_required(import_required);
                    self.store_body(loc, file, identity, entry);
                }
                CheckOutcome::InMemory(Ok(checked)) => {
                    errors.append(self.install_checked_body(&loc, checked))
                }
                CheckOutcome::InMemory(Err(e)) => errors.append(e),
            }
        }

        // The concrete types of opaque types are known only once the values are checked, and a
        // cycle among them has to be reported before instantiation puts each in the other's place.
        errors.eat_err(self.validate_opaque_types_are_acyclic(tc));

        errors.to_result()
    }

    /// Type-checks the bodies `to_check` on the compiler threads, writes each that passes to the file
    /// of its module, and answers what became of each, in the order of `to_check`.
    ///
    /// A body that passes is written as soon as it is checked, and leaves memory then; the file of
    /// a module is put in place once the last body of the module is checked. A body the file could
    /// not take is checked again, to be held in memory.
    ///
    /// The bodies a file already held are decoded to see whether they are intact, with the source
    /// files their spans point into taken from `source_files`, which gains the paths it lacks.
    fn check_and_write_bodies(
        &self,
        to_check: &[(BodyLocation, Arc<ModuleFile>, EntityIdentity)],
        tc: &TypeCheckContext,
        source_files: &mut Map<PathBuf, SourceFile>,
    ) -> Vec<CheckOutcome> {
        let dir = if to_check.is_empty() {
            None
        } else {
            create_typed_modules_dir().filter(|dir| accepts_new_files(dir))
        };
        let mut outcomes: Vec<Option<CheckOutcome>> = to_check.iter().map(|_| None).collect();
        let mut unchecked_of_module: Map<Name, usize> = Map::default();
        for (_loc, file, _identity) in to_check {
            *unchecked_of_module.entry(file.module.clone()).or_default() += 1;
        }
        let mut writing_of_module: Map<Name, ModuleWriting> = Map::default();
        let mut unwritten: Vec<usize> = vec![];
        let mut failures: Vec<(Arc<ModuleFile>, io::Error)> = vec![];
        let nrenv = self.create_name_resolution_env();
        consume_on_compiler_threads(
            (0..to_check.len()).collect(),
            tc.num_worker_threads,
            |i| {
                let (loc, _file, identity) = &to_check[i];
                let checked = match self.check_body(loc, &nrenv, tc) {
                    Ok(checked) if checked.passed() && dir.is_some() => {
                        CheckedOnThread::Passed(PassedBody {
                            entry: NewEntry::of(identity.clone(), &checked),
                            import_required: checked.import_required,
                        })
                    }
                    checked => CheckedOnThread::InMemory(checked),
                };
                (i, checked)
            },
            |(i, checked)| {
                let file = &to_check[i].1;
                match checked {
                    CheckedOnThread::Passed(body) => {
                        let dir = dir
                            .as_deref()
                            .expect("a body passed to be written with no directory to write in");
                        writing_of_module
                            .entry(file.module.clone())
                            .or_insert_with(|| ModuleWriting::begin(dir, file.clone()))
                            .add(i, body, source_files);
                    }
                    CheckedOnThread::InMemory(checked) => {
                        outcomes[i] = Some(CheckOutcome::InMemory(checked))
                    }
                }
                let unchecked = unchecked_of_module
                    .get_mut(&file.module)
                    .expect("a body of a module whose bodies were not counted");
                *unchecked -= 1;
                if *unchecked == 0 {
                    if let Some(writing) = writing_of_module.remove(&file.module) {
                        let written = writing.finish();
                        for (stored_i, outcome) in written.stored {
                            outcomes[stored_i] = Some(CheckOutcome::Stored(outcome));
                        }
                        unwritten.extend(written.unwritten);
                        failures.extend(written.failure.map(|e| (file.clone(), e)));
                    }
                }
            },
        );
        // The modules finish in the order their bodies happen to be checked, so the failures are
        // reported in the order of the modules.
        failures.sort_by(|(lhs, _), (rhs, _)| lhs.module.cmp(&rhs.module));
        for (file, e) in &failures {
            let dir = dir
                .as_deref()
                .expect("a module file failed to be written with no directory to write in");
            warn_unwritable_file(dir, file, e);
        }

        let checked_again =
            map_on_compiler_threads(unwritten.clone(), tc.num_worker_threads, |i| {
                self.check_body(&to_check[i].0, &nrenv, tc)
            });
        for (i, checked) in unwritten.into_iter().zip(checked_again) {
            outcomes[i] = Some(CheckOutcome::InMemory(checked));
        }
        outcomes
            .into_iter()
            .map(|outcome| outcome.expect("a body left unchecked"))
            .collect()
    }

    /// Takes the body at `loc`, of the entity `identity`, as the one `file` records as `entry`: its
    /// typed expression is read back from the file when instantiation asks for it.
    fn store_body(
        &mut self,
        loc: BodyLocation,
        file: Arc<ModuleFile>,
        identity: EntityIdentity,
        entry: EntityEntry,
    ) {
        self.add_opaque_types(entry.opaque_types);
        self.typed_modules.stored.insert(
            loc,
            StoredBody {
                file,
                identity,
                range: entry.body,
                warning_facts: entry.warning_facts,
            },
        );
    }

    /// The typed expression of the stored body at `loc`, read from the file of its module as
    /// `read_body` reads it.
    ///
    /// Another build may have put a file of its own in place of the one this build read the range
    /// from, holding the same bodies in another order. So when the bytes at the range are not the
    /// body, the ranges of the bodies of the file are taken anew from the record it holds now
    /// (`relocate_stored_bodies`), and the body is read once more.
    fn read_stored_body(&mut self, loc: &BodyLocation) -> Result<Option<TypedExpr>, String> {
        let read = |typed_modules: &mut TypedModules| {
            let stored = &typed_modules.stored[loc];
            SourceFile::with_shared_source_files(&mut typed_modules.source_files, || {
                read_body(&stored.file.path(), &stored.range, &stored.identity)
            })
        };
        match read(&mut self.typed_modules) {
            Err(_) if self.typed_modules.relocate_stored_bodies(loc) => {
                read(&mut self.typed_modules)
            }
            read => read,
        }
    }

    /// The typed expression of the body at `loc`: read from the file of its module when the
    /// program was checked through `check_bodies_through_typed_modules` and the file recorded the
    /// body, and the one `Program::global_values` holds otherwise.
    ///
    /// A body the file no longer holds is type-checked again, and written back to the file, so that
    /// the next build reads it.
    pub fn typed_body(
        &mut self,
        loc: &BodyLocation,
        tc: &TypeCheckContext,
    ) -> Result<TypedExpr, Errors> {
        let Some(stored) = self.typed_modules.stored.get(loc) else {
            let (te, _scm, _module) = self.body_at(loc);
            return Ok(te.clone());
        };
        if let Some(te) = self.typed_modules.read_back.get(loc) {
            return Ok(te.clone());
        }
        let path = stored.file.path();
        let te = match self.read_stored_body(loc) {
            Ok(Some(te)) => te,
            failed => {
                if let Err(why) = failed {
                    warn_unreadable_file(&path, &why);
                }
                // The opaque types of the body are already known from the record.
                let checked = self.check_body(loc, &self.create_name_resolution_env(), tc)?;
                if checked.diagnostics.has_error() {
                    return Err(checked.diagnostics);
                }
                if checked.passed() {
                    if let Some(dir) = create_typed_modules_dir() {
                        let stored = &self.typed_modules.stored[loc];
                        let entry = NewEntry::of(stored.identity.clone(), &checked);
                        let source_files = &mut self.typed_modules.source_files;
                        let written = ModuleFileWriter::create(&dir, stored.file.clone()).and_then(
                            |mut writer| {
                                writer.add(entry, source_files)?;
                                writer.finish()
                            },
                        );
                        if let Err(e) = written {
                            warn_unwritable_file(&dir, &stored.file, &e);
                        }
                    }
                }
                checked.te
            }
        };
        self.typed_modules.read_back.insert(loc.clone(), te.clone());
        Ok(te)
    }
}

#[cfg(test)]
mod tests {
    use super::decode_body;
    use crate::ast::program::TypedExpr;
    use crate::elaboration::typecheckcache::EntityIdentity;
    use crate::fixstd::builtin::{expr_int_lit, make_u128_ty};

    /// An integer literal wider than 64 bits reads back from the form a body takes in a module's
    /// file with every bit it was written with.
    #[test]
    fn an_integer_literal_wider_than_64_bits_reads_back_from_a_module_file() {
        let identity: EntityIdentity = "Main::wide\nU128".to_string();
        let expr = TypedExpr::from_expr(expr_int_lit(u128::MAX, make_u128_ty(), None));
        let bytes = postcard::to_allocvec(&(&identity, &expr)).unwrap();
        let read = decode_body(&bytes, &identity).unwrap();
        assert_eq!(
            read.expr.get_builtin().op.name(),
            format!("int({})", u128::MAX)
        );
    }
}
