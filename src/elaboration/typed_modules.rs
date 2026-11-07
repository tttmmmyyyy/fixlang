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
// reads a typed expression back only when instantiation asks for it, from the file it found the
// body in, which it keeps mapped into memory (`ModuleFileContents`). When the check passes bodies
// the record lacked, the file is written anew (`ModuleFileWriter`): the bodies it holds are copied
// as they are, at the same offsets, and each new body follows them. A new body is serialized as soon
// as it passes, so that its typed expression leaves memory then, and the file is written once the
// last body of the module is checked. When instantiation finds a body that the file fails to hold,
// it checks the body again and writes it the same way, after the bodies the file holds.
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
use crate::misc::{consume_on_compiler_threads, warn_msg, Map};
use crate::parse::sourcefile::{SourceFile, Span};
use crate::project_fs::{
    create_dir_guarding_project_links, remove_abandoned_temporary_files, replace_file,
    DOT_FIXLANG_LINK_HINT,
};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::ops::{Deref, Range};
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

/// `name` without its last 32 characters, which is the `module_file_prefix` of a name that
/// `module_file_name` gives.
///
/// # Examples
///
/// `Main_Sub_<digest of the name>_<digest of the hash>` gives `Main_Sub_<digest of the name>_`.
fn module_file_prefix_of(name: &str) -> Option<&str> {
    // `module_file_name` ends the name in the 32 hexadecimal digits of an MD5 digest.
    name.get(..name.len().checked_sub(32)?)
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

/// The record of the module file `file`, and the file mapped into memory, to read the bodies from.
/// A file that does not exist answers `None`; one that cannot be read, or is not a file of
/// type-checked modules, answers the reason.
///
/// Only the pages of the footer and the record are read here, so that a build reads only the
/// bodies it asks for later.
fn map_module_file(
    file: &Arc<ModuleFile>,
) -> Result<Option<(ModuleRecord, ModuleFileContents)>, String> {
    let Some(bytes) = map_file(&file.path()).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let (record, _bodies_end) = decode_module_file(&bytes)?;
    let contents = ModuleFileContents {
        file: file.clone(),
        bytes: ModuleFileBytes::Mapped(bytes),
    };
    Ok(Some((record, contents)))
}

/// The file at `path` mapped into memory, or `None` when there is no file there.
fn map_file(path: &Path) -> io::Result<Option<Mmap>> {
    match File::open(path) {
        Ok(file) => map_into_memory(&file).map(Some),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// `file` mapped into memory. `file` must be a file of type-checked modules, or the temporary file
/// of a new version of one, since the mapping stays sound only while nothing writes the file in place.
fn map_into_memory(file: &File) -> io::Result<Mmap> {
    // SAFETY: the mapping is read for as long as the build runs, and a file of type-checked modules
    // is never written in place: a build writes a new version under another name and renames it
    // over the file, or removes the file, and either leaves the mapped bytes as they were. A read
    // of a page that cannot be loaded stops the process with SIGBUS: a file truncated in place by
    // another program, an I/O error of the disk, or a file that a build on another host removed
    // from a network file system.
    unsafe { Mmap::map(file) }
}

/// The record of the module file whose bytes are `bytes`, and the offset its bodies end at. Bytes
/// that are not a file of type-checked modules answer the reason.
fn decode_module_file(bytes: &[u8]) -> Result<(ModuleRecord, u64), String> {
    let len = bytes.len() as u64;
    if len < FOOTER_LEN {
        return Err(NOT_A_MODULE_FILE.to_string());
    }
    let footer = bytes[(len - FOOTER_LEN) as usize..].try_into().unwrap();
    let Some(range) = record_range(footer, len) else {
        return Err(NOT_A_MODULE_FILE.to_string());
    };
    let record = decode_record(
        &bytes[range.start as usize..range.end as usize],
        range.start,
    )?;
    Ok((record, range.start))
}

/// The typed expression of the body `identity` lying at `range` in `bytes`, the bytes of a module's
/// file whose record places the body there. Bytes there that are not the body answer the reason.
fn read_body(
    bytes: &[u8],
    range: &Range<u64>,
    identity: &EntityIdentity,
) -> Result<TypedExpr, String> {
    decode_body(&bytes[range.start as usize..range.end as usize], identity)
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

/// The bytes of a module's file that a build reads bodies from.
enum ModuleFileBytes {
    /// The file as this build found it or wrote it, mapped into memory.
    Mapped(Mmap),
    /// The bytes this build meant the file to hold, kept in memory when the file was not written.
    Unwritten(Vec<u8>),
}

impl Deref for ModuleFileBytes {
    type Target = [u8];

    /// The bytes, whether mapped or kept in memory.
    fn deref(&self) -> &[u8] {
        match self {
            ModuleFileBytes::Mapped(bytes) => bytes,
            ModuleFileBytes::Unwritten(bytes) => bytes,
        }
    }
}

/// The contents of a module's file as this build found it, wrote it, or meant to write it. Each
/// body the build takes from the file is read from `bytes`, so a file another build puts in place
/// meanwhile, or removes, leaves the body where it was.
struct ModuleFileContents {
    /// The module and hash the file is named after.
    file: Arc<ModuleFile>,
    /// The bytes of the file.
    bytes: ModuleFileBytes,
}

/// The new version of a module's file, built in memory while the bodies of the module are checked.
/// It begins with the bodies the file held, at the offsets they had, and each body `add`ed follows
/// them; `finish` puts the record and the footer after the bodies.
struct ModuleFileWriter {
    /// The module file the new version replaces.
    file: Arc<ModuleFile>,
    /// The bodies the new version holds so far, serialized as the file holds them.
    bodies: Vec<u8>,
    /// Where the bodies taken from the file the new version replaces end.
    held_end: u64,
    /// Where each body of the new version lies in `bodies`, and what the file records of it.
    record: ModuleRecord,
}

impl ModuleFileWriter {
    /// Begins a new version of `file` with the bodies the file holds now, so that the bodies another
    /// build added to it since this build read its record are kept. A file that cannot be read, or
    /// does not decode as the file of a type-checked module, is begun anew, since the new version
    /// replaces it either way.
    fn begin(file: Arc<ModuleFile>) -> Self {
        let mut writer = ModuleFileWriter {
            file,
            bodies: vec![],
            held_end: 0,
            record: ModuleRecord::default(),
        };
        let Ok(Some(held)) = map_file(&writer.file.path()) else {
            return writer;
        };
        let Ok((record, bodies_end)) = decode_module_file(&held) else {
            return writer;
        };
        writer.bodies = held[..bodies_end as usize].to_vec();
        writer.held_end = bodies_end;
        writer.record = record;
        writer
    }

    /// Adds `entry` after the bodies so far. A body the file held under the same identity is
    /// replaced by `entry`; its bytes stay in the file, recorded by nothing.
    fn add(&mut self, entry: NewEntry) {
        if let Some(recorded) = self.record.get(&entry.identity) {
            assert!(
                recorded.body.end <= self.held_end,
                "the body of {:?} added twice to the file of `{}`",
                entry.identity,
                self.file.module
            );
        }
        let start = self.bodies.len() as u64;
        self.bodies.extend_from_slice(&entry.bytes);
        self.record.insert(
            entry.identity,
            EntityEntry {
                body: start..self.bodies.len() as u64,
                opaque_types: entry.opaque_types,
                warning_facts: entry.warning_facts,
            },
        );
    }

    /// The bytes of the new version: the bodies, then the record and the footer. The record is
    /// answered beside them.
    fn finish(self) -> (ModuleRecord, Vec<u8>) {
        let mut entries: Vec<(EntityIdentity, EntityEntry)> = self.record.into_iter().collect();
        entries.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
        let mut bytes = self.bodies;
        let bodies_end = bytes.len() as u64;
        let record = postcard::to_allocvec(&entries).unwrap();
        bytes.extend_from_slice(&record);
        bytes.extend_from_slice(&bodies_end.to_le_bytes());
        bytes.extend_from_slice(&(record.len() as u64).to_le_bytes());
        bytes.extend_from_slice(FILE_MAGIC);
        (entries.into_iter().collect(), bytes)
    }
}

/// Writes `bytes` in place of the file of `file` in `dir`, and answers the written file mapped into
/// memory.
fn write_module_file(dir: &Path, file: &ModuleFile, bytes: &[u8]) -> io::Result<Mmap> {
    let mut mapped = None;
    // The name holds no path separator, and `replace_file` replaces a symbolic link at the path
    // instead of writing through it, so the file is written in `dir`.
    replace_file(&file.path_in(dir), |tmp_path| {
        let mut tmp = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(tmp_path)?;
        tmp.write_all(bytes)?;
        mapped = Some(map_into_memory(&tmp)?);
        Ok(())
    })?;
    Ok(mapped.expect("a module file written without being mapped"))
}

/// Removes from `dir` the files of the modules of `written` checked under any other hash than the
/// one each was written under, and the temporary files that builds which were killed left there.
/// The files of `written` are in place already, so a file this fails to remove costs only disk
/// space, and a later build tries again.
fn remove_stale_files(dir: &Path, written: &[Arc<ModuleFile>]) {
    remove_abandoned_temporary_files(dir);
    let Ok(dir_entries) = fs::read_dir(dir) else {
        return;
    };
    let kept: Map<String, String> = written
        .iter()
        .map(|file| (module_file_prefix(&file.module), file.name()))
        .collect();
    for dir_entry in dir_entries.flatten() {
        let entry_name = dir_entry.file_name();
        let entry_name = entry_name.to_string_lossy();
        let Some(prefix) = module_file_prefix_of(&entry_name) else {
            continue;
        };
        if kept
            .get(prefix)
            .is_some_and(|kept_name| *kept_name != entry_name)
        {
            let _ = fs::remove_file(dir.join(entry_name.as_ref()));
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

/// What a compiler thread answers for a body it type-checked.
enum CheckedOnThread {
    /// The body passed, and the file of its module is to hold it.
    Passed(NewEntry),
    /// What `Program::check_body` answered for a body that did not pass.
    NotPassed(Result<CheckedBody, Errors>),
}

/// What became of a body this build checked.
enum CheckOutcome {
    /// The body passed, and the new version of its module's file records it as `entry`.
    Stored {
        /// The contents of the new version.
        contents: Arc<ModuleFileContents>,
        /// What the new version records of the body.
        entry: EntityEntry,
    },
    /// What `Program::check_body` answered for a body that did not pass.
    NotPassed(Result<CheckedBody, Errors>),
}

/// A body whose typed expression lies in its module's file, and which `Program::global_values`
/// holds as it was parsed.
struct StoredBody {
    /// The contents of the file of the module the body is defined in, which the body is read back
    /// from.
    contents: Arc<ModuleFileContents>,
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
    /// A body that passes, whether recorded before or added here, is left as it was parsed, and its
    /// typed expression is read back when instantiation asks for it (`typed_body`), from the file
    /// or, when the file could not be written, from the bytes meant for it. The typed
    /// expressions of the other bodies are put in place of their bodies.
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

        // Skip the bodies the files record. The bodies left to check stay grouped by module, so
        // that the new versions of few files are held in memory at once.
        let mut to_check: Vec<(BodyLocation, Arc<ModuleFile>, EntityIdentity)> = vec![];
        for (module, locs) in bodies_by_module {
            let module_hash = self.module_dependency_hash(&module, config)?;
            let file = Arc::new(ModuleFile {
                module,
                module_hash,
            });
            let found =
                SourceFile::with_shared_source_files(&mut self.typed_modules.source_files, || {
                    map_module_file(&file)
                });
            let found = match found {
                Ok(found) => found.map(|(record, contents)| (record, Arc::new(contents))),
                Err(why) => {
                    warn_unreadable_file(&file.path(), &why);
                    None
                }
            };
            for loc in locs {
                let identity = self.entity_identity_at(&loc);
                let recorded = found.as_ref().and_then(|(record, contents)| {
                    record
                        .get(&identity)
                        .map(|entry| (contents.clone(), entry.clone()))
                });
                match recorded {
                    Some((contents, entry)) => self.store_body(loc, contents, identity, entry),
                    None => to_check.push((loc, file.clone(), identity)),
                }
            }
        }

        // Check the rest.
        let outcomes = self.check_and_write_bodies(&to_check, tc);
        for ((loc, _file, identity), outcome) in to_check.into_iter().zip(outcomes) {
            match outcome {
                CheckOutcome::Stored { contents, entry } => {
                    self.store_body(loc, contents, identity, entry)
                }
                CheckOutcome::NotPassed(Ok(checked)) => {
                    errors.append(self.install_checked_body(&loc, checked))
                }
                CheckOutcome::NotPassed(Err(e)) => errors.append(e),
            }
        }

        // The concrete types of opaque types are known only once the values are checked, and a
        // cycle among them has to be reported before instantiation puts each in the other's place.
        errors.eat_err(self.validate_opaque_types_are_acyclic(tc));

        errors.to_result()
    }

    /// Type-checks the bodies `to_check` on the compiler threads, adds each that passes to the file
    /// of its module, and answers what became of each, in the order of `to_check`.
    ///
    /// A body that passes is serialized on the thread that checked it, and its typed expression
    /// leaves memory then; the file of a module is written once the last body of the module is
    /// checked. When a file cannot be written, that is reported as a warning, no file is written
    /// after it, and the bodies of that module and of the modules after it are read from the bytes
    /// meant for their files.
    fn check_and_write_bodies(
        &self,
        to_check: &[(BodyLocation, Arc<ModuleFile>, EntityIdentity)],
        tc: &TypeCheckContext,
    ) -> Vec<CheckOutcome> {
        let dir = if to_check.is_empty() {
            None
        } else {
            create_typed_modules_dir()
        };
        let mut write_failed = false;
        let mut outcomes: Vec<Option<CheckOutcome>> = to_check.iter().map(|_| None).collect();
        let mut unchecked_of_module: Map<Name, usize> = Map::default();
        for (_loc, file, _identity) in to_check {
            *unchecked_of_module.entry(file.module.clone()).or_default() += 1;
        }
        // The new version of each module's file being built, with the indices in `to_check` of the
        // bodies added to it.
        let mut writing_of_module: Map<Name, (ModuleFileWriter, Vec<usize>)> = Map::default();
        let mut written: Vec<Arc<ModuleFile>> = vec![];
        let nrenv = self.create_name_resolution_env();
        consume_on_compiler_threads(
            (0..to_check.len()).collect(),
            tc.num_worker_threads,
            |i| {
                let (loc, _file, identity) = &to_check[i];
                let checked = match self.check_body(loc, &nrenv, tc) {
                    Ok(checked) if checked.passed() => {
                        CheckedOnThread::Passed(NewEntry::of(identity.clone(), &checked))
                    }
                    checked => CheckedOnThread::NotPassed(checked),
                };
                (i, checked)
            },
            |(i, checked)| {
                let file = &to_check[i].1;
                match checked {
                    CheckedOnThread::Passed(entry) => {
                        let (writer, added) = writing_of_module
                            .entry(file.module.clone())
                            .or_insert_with(|| (ModuleFileWriter::begin(file.clone()), vec![]));
                        writer.add(entry);
                        added.push(i);
                    }
                    CheckedOnThread::NotPassed(checked) => {
                        outcomes[i] = Some(CheckOutcome::NotPassed(checked))
                    }
                }
                let unchecked = unchecked_of_module
                    .get_mut(&file.module)
                    .expect("a body of a module whose bodies were not counted");
                *unchecked -= 1;
                if *unchecked > 0 {
                    return;
                }
                let Some((writer, added)) = writing_of_module.remove(&file.module) else {
                    return;
                };
                let (mut record, bytes) = writer.finish();
                let bytes = match &dir {
                    Some(dir) if !write_failed => match write_module_file(dir, file, &bytes) {
                        Ok(mapped) => {
                            written.push(file.clone());
                            ModuleFileBytes::Mapped(mapped)
                        }
                        Err(e) => {
                            warn_unwritable_file(dir, file, &e);
                            write_failed = true;
                            ModuleFileBytes::Unwritten(bytes)
                        }
                    },
                    _ => ModuleFileBytes::Unwritten(bytes),
                };
                let contents = Arc::new(ModuleFileContents {
                    file: file.clone(),
                    bytes,
                });
                for added_i in added {
                    let entry = record
                        .remove(&to_check[added_i].2)
                        .expect("the new version of a file lacks a body added to it");
                    outcomes[added_i] = Some(CheckOutcome::Stored {
                        contents: contents.clone(),
                        entry,
                    });
                }
            },
        );
        if let (Some(dir), false) = (&dir, written.is_empty()) {
            remove_stale_files(dir, &written);
        }
        outcomes
            .into_iter()
            .map(|outcome| outcome.expect("a body left unchecked"))
            .collect()
    }

    /// Takes the body at `loc`, of the entity `identity`, as the one the module file of `contents`
    /// records as `entry`: its typed expression is read back from the file when instantiation asks
    /// for it.
    fn store_body(
        &mut self,
        loc: BodyLocation,
        contents: Arc<ModuleFileContents>,
        identity: EntityIdentity,
        entry: EntityEntry,
    ) {
        self.add_opaque_types(entry.opaque_types);
        self.typed_modules.stored.insert(
            loc,
            StoredBody {
                contents,
                identity,
                range: entry.body,
                warning_facts: entry.warning_facts,
            },
        );
    }

    /// The typed expression of the body at `loc`: read from the file of its module when the
    /// program was checked through `check_bodies_through_typed_modules` and the file recorded the
    /// body, and the one `Program::global_values` holds otherwise.
    ///
    /// A body the file fails to hold intact is type-checked again, and written back to the file, so
    /// that the next build reads it.
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
        let read =
            SourceFile::with_shared_source_files(&mut self.typed_modules.source_files, || {
                read_body(&stored.contents.bytes, &stored.range, &stored.identity)
            });
        let te = match read {
            Ok(te) => te,
            Err(why) => {
                warn_unreadable_file(&stored.contents.file.path(), &why);
                // The opaque types of the body are already known from the record.
                let checked = self.check_body(loc, &self.create_name_resolution_env(), tc)?;
                if checked.diagnostics.has_error() {
                    return Err(checked.diagnostics);
                }
                if checked.passed() {
                    if let Some(dir) = create_typed_modules_dir() {
                        let stored = &self.typed_modules.stored[loc];
                        let file = &stored.contents.file;
                        let mut writer = ModuleFileWriter::begin(file.clone());
                        writer.add(NewEntry::of(stored.identity.clone(), &checked));
                        let (_record, bytes) = writer.finish();
                        if let Err(e) = write_module_file(&dir, file, &bytes) {
                            warn_unwritable_file(&dir, file, &e);
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
