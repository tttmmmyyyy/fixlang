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
// same offsets, and the new bodies follow them. A body that instantiation finds the file failing to
// hold is checked again and written the same way, after the bodies the file holds.
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
use crate::misc::{warn_msg, Map};
use crate::parse::sourcefile::{SourceFile, Span};
use crate::project_fs::{create_dir_guarding_project_links, replace_file, DOT_FIXLANG_LINK_HINT};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
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

/// The record serialized as `bytes`, in a file whose bodies end at `bodies_end`.
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

/// The typed expression `bytes` hold, which a module's file writes after the identity of its
/// body; the reason when the bytes are not the body of `identity`.
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
/// Only the footer and the record are read, so that a build reads of a module's bodies only those
/// it asks for later.
fn read_record(path: &Path) -> Result<Option<ModuleRecord>, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
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
    match read(&mut file) {
        Ok(Some((record_bytes, bodies_end))) => Ok(Some(decode_record(&record_bytes, bodies_end)?)),
        Ok(None) => Err(NOT_A_MODULE_FILE.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// The record of the module file `bytes`, and the bytes of its bodies.
fn decode_module_file(mut bytes: Vec<u8>) -> Result<(ModuleRecord, Vec<u8>), String> {
    let len = bytes.len() as u64;
    if len < FOOTER_LEN {
        return Err(NOT_A_MODULE_FILE.to_string());
    }
    let footer = bytes[(len - FOOTER_LEN) as usize..].try_into().unwrap();
    let range = record_range(footer, len).ok_or_else(|| NOT_A_MODULE_FILE.to_string())?;
    let record = decode_record(
        &bytes[range.start as usize..range.end as usize],
        range.start,
    )?;
    bytes.truncate(range.start as usize);
    Ok((record, bytes))
}

/// The typed expression of the body `identity` lying at `range` in the file at `path`. A file that
/// no longer exists answers `None`; one whose bytes there are not the body answers the reason.
///
/// A body the file holds intact stays at its offset when the file is written anew, so `range` read
/// from an earlier record of the file still finds it. The identity written in front of the body is
/// compared with `identity` all the same, so a file replaced by something else is never taken for
/// it.
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
    /// The entry of the body `identity` that `checked` passed.
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
    module: Name,
    /// The `module_dependency_hash` of `module`.
    module_hash: String,
}

impl ModuleFile {
    /// The path of the file, under `TYPED_MODULES_PATH`.
    fn path(&self) -> PathBuf {
        Path::new(TYPED_MODULES_PATH).join(module_file_name(&self.module, &self.module_hash))
    }
}

/// Adds `new` to the file of `module` checked under `module_hash`, in `dir`, and removes the files of
/// `module` checked under any other hash.
///
/// The file is read again here, so that the bodies another build added to it since this build read
/// it are kept. A body the file already holds is kept as it is, unless its bytes are not the body,
/// which `new` then replaces.
fn add_to_module_file(
    dir: &Path,
    module: &Name,
    module_hash: &str,
    new: Vec<NewEntry>,
) -> io::Result<()> {
    let file_name = module_file_name(module, module_hash);
    let path = dir.join(&file_name);
    let (mut record, mut bytes) = match fs::read(&path) {
        Ok(bytes) => decode_module_file(bytes).unwrap_or_default(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Default::default(),
        Err(e) => return Err(e),
    };
    for entry in new {
        if let Some(held) = record.get(&entry.identity) {
            let held_bytes = &bytes[held.body.start as usize..held.body.end as usize];
            if decode_body(held_bytes, &entry.identity).is_ok() {
                continue;
            }
        }
        let start = bytes.len() as u64;
        bytes.extend_from_slice(&entry.bytes);
        record.insert(
            entry.identity,
            EntityEntry {
                body: start..bytes.len() as u64,
                opaque_types: entry.opaque_types,
                warning_facts: entry.warning_facts,
            },
        );
    }
    let mut entries: Vec<(EntityIdentity, EntityEntry)> = record.into_iter().collect();
    entries.sort_by(|(lhs, _), (rhs, _)| lhs.cmp(rhs));
    let record = postcard::to_allocvec(&entries).unwrap();
    let offset = bytes.len() as u64;
    bytes.extend_from_slice(&record);
    bytes.extend_from_slice(&offset.to_le_bytes());
    bytes.extend_from_slice(&(record.len() as u64).to_le_bytes());
    bytes.extend_from_slice(FILE_MAGIC);
    // The name holds no path separator, and `replace_file` replaces a symbolic link at the path
    // instead of writing through it, so the file is written in `dir`.
    replace_file(&path, |tmp_path| fs::write(tmp_path, &bytes))?;

    // The temporary files `replace_file` writes begin with a dot, which no prefix does.
    let prefix = module_file_prefix(module);
    for dir_entry in fs::read_dir(dir)? {
        let entry_name = dir_entry?.file_name();
        let entry_name = entry_name.to_string_lossy();
        if entry_name.starts_with(&prefix) && entry_name != file_name.as_str() {
            let _ = fs::remove_file(dir.join(entry_name.as_ref()));
        }
    }
    Ok(())
}

/// Adds each list of bodies of `new_entries` to the file it is paired with. A file that cannot be
/// written is reported as a warning.
///
/// The bodies a file already holds are decoded to see whether they are intact, and the spans in
/// them share `source_files`.
fn add_to_module_files<'a>(
    new_entries: impl IntoIterator<Item = (&'a ModuleFile, Vec<NewEntry>)>,
    source_files: &mut Map<PathBuf, SourceFile>,
) {
    let dir = match create_dir_guarding_project_links(
        Path::new(TYPED_MODULES_PATH),
        DOT_FIXLANG_LINK_HINT,
    ) {
        Ok(dir) => dir,
        Err(errs) => {
            for err in errs.errors() {
                warn_msg(&err.msg);
            }
            return;
        }
    };
    for (file, entries) in new_entries {
        let added = SourceFile::with_shared_source_files(source_files, || {
            add_to_module_file(&dir, &file.module, &file.module_hash, entries)
        });
        if let Err(e) = added {
            warn_msg(&format!(
                "Failed to write cache file \"{}\": {}.",
                dir.join(module_file_name(&file.module, &file.module_hash))
                    .to_string_lossy(),
                e
            ));
        }
    }
}

/// Reports, as a warning, that the file at `path` could not be read for `why`.
fn warn_unreadable_file(path: &Path, why: &str) {
    warn_msg(&format!(
        "Failed to read cache file \"{}\": {}.",
        path.to_string_lossy(),
        why
    ));
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
    /// The bodies the files recorded when the program was checked, which the check skipped.
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
    /// A body the file of its module records is left as it was parsed, and its typed expression is
    /// read back when instantiation asks for it (`typed_body`).
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
                    Some(entry) => {
                        self.add_opaque_types(entry.opaque_types.clone());
                        self.typed_modules.stored.insert(
                            loc,
                            StoredBody {
                                file: file.clone(),
                                identity,
                                range: entry.body.clone(),
                                warning_facts: entry.warning_facts.clone(),
                            },
                        );
                    }
                    None => to_check.push((loc, file.clone(), identity)),
                }
            }
        }

        // Check the rest, and keep what passes with no diagnostic for the files.
        let mut new_entries: Map<Name, (Arc<ModuleFile>, Vec<NewEntry>)> = Map::default();
        let locs = to_check.iter().map(|(loc, _, _)| loc.clone()).collect();
        let results = self.check_bodies(locs, tc);
        for ((loc, file, identity), (checked_loc, checked)) in to_check.into_iter().zip(results) {
            assert!(loc == checked_loc, "`check_bodies` answered out of order");
            match checked {
                Ok(checked) => {
                    if checked.passed() {
                        new_entries
                            .entry(file.module.clone())
                            .or_insert_with(|| (file.clone(), vec![]))
                            .1
                            .push(NewEntry::of(identity, &checked));
                    }
                    errors.append(self.install_checked_body(&loc, checked));
                }
                Err(e) => errors.append(e),
            }
        }
        if !new_entries.is_empty() {
            add_to_module_files(
                new_entries
                    .values_mut()
                    .map(|(file, entries)| (&**file, std::mem::take(entries))),
                &mut self.typed_modules.source_files,
            );
        }

        // The concrete types of opaque types are known only once the values are checked, and a
        // cycle among them has to be reported before instantiation puts each in the other's place.
        errors.eat_err(self.validate_opaque_types_are_acyclic(tc));

        errors.to_result()
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
        let read =
            SourceFile::with_shared_source_files(&mut self.typed_modules.source_files, || {
                read_body(&stored.file.path(), &stored.range, &stored.identity)
            });
        let te = match read {
            Ok(Some(te)) => te,
            failed => {
                if let Err(why) = failed {
                    warn_unreadable_file(&stored.file.path(), &why);
                }
                // The opaque types of the body are already known from the record.
                let checked = self.check_body(loc, &self.create_name_resolution_env(), tc)?;
                if checked.diagnostics.has_error() {
                    return Err(checked.diagnostics);
                }
                if checked.passed() {
                    let stored = &self.typed_modules.stored[loc];
                    let entry = NewEntry::of(stored.identity.clone(), &checked);
                    add_to_module_files(
                        [(&*stored.file, vec![entry])],
                        &mut self.typed_modules.source_files,
                    );
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
