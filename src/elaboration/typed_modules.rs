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
// same offsets, and the new bodies follow them.
//
// The record is valid because a body is determined by its `EntityIdentity` together with what the
// module's `module_dependency_hash` covers. The set of bodies a module has can still differ between
// two builds under one hash (the implementations `Std` carries for tuples follow the sizes the
// program uses), which is why a build compares bodies, not modules, against the record.

use crate::ast::expr::ExprNode;
use crate::ast::name::{FullName, Name};
use crate::ast::pattern::Pattern;
use crate::ast::program::{BodyLocation, Program, SymbolExpr, TypedExpr};
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

/// The last bytes of a module's file: they mark the file as one this module wrote.
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

/// The record of the file at `path`, and where its bodies end. A file that does not exist answers
/// `None`; one that cannot be read, or is not a file this module wrote, answers the reason.
fn read_record(path: &Path) -> Result<Option<(ModuleRecord, u64)>, String> {
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
        if &footer[16..] != FILE_MAGIC {
            return Ok(None);
        }
        let offset = u64::from_le_bytes(footer[0..8].try_into().unwrap());
        let record_len = u64::from_le_bytes(footer[8..16].try_into().unwrap());
        if offset.checked_add(record_len) != Some(len - FOOTER_LEN) {
            return Ok(None);
        }
        let mut record = vec![0u8; record_len as usize];
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut record)?;
        Ok(Some((record, offset)))
    };
    let (record, bodies_end) = match read(&mut file) {
        Ok(Some(read)) => read,
        Ok(None) => return Err("it is not a file of type-checked modules".to_string()),
        Err(e) => return Err(e.to_string()),
    };
    let entries: Vec<(EntityIdentity, EntityEntry)> =
        postcard::from_bytes(&record).map_err(|e| e.to_string())?;
    if entries.iter().any(|(_, entry)| entry.body.end > bodies_end) {
        return Err("its record points past its bodies".to_string());
    }
    Ok(Some((entries.into_iter().collect(), bodies_end)))
}

/// The typed expression of the body `identity` lying at `range` in the file at `path`. A file that
/// no longer exists answers `None`; one whose bytes there are not the body answers the reason.
///
/// The bodies a file holds stay at their offsets when the file is written anew, so `range` read
/// from an earlier record of the file still finds the body. The identity written in front of the
/// body is compared with `identity` all the same, so a file replaced by something else is never
/// taken for it.
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
    let ((found, expr), rest): ((EntityIdentity, TypedExpr), _) =
        postcard::take_from_bytes(&bytes).map_err(|e| e.to_string())?;
    if !rest.is_empty() || &found != identity {
        return Err(format!(
            "it holds another body where the body of `{}` was recorded",
            identity.replace('\n', " : ")
        ));
    }
    Ok(Some(expr))
}

/// A body passed by this build's type check, to be added to its module's file.
struct NewEntry {
    identity: EntityIdentity,
    /// The identity and the typed expression, serialized as the file holds them.
    bytes: Vec<u8>,
    opaque_types: Map<FullName, Vec<OpaqueTyConResolution>>,
    warning_facts: Vec<WarningFact>,
}

/// Adds `new` to the file of `module` checked under `module_hash`, in `dir`, and removes the files of
/// `module` checked under any other hash.
///
/// The file is read again here, so that the bodies another build added to it since this build read
/// it are kept. A body the file already holds is kept as it is.
fn add_to_module_file(
    dir: &Path,
    module: &Name,
    module_hash: &str,
    new: Vec<NewEntry>,
) -> io::Result<()> {
    let name = module_file_name(module, module_hash);
    let path = dir.join(&name);
    let (mut bytes, mut record) = match read_record(&path) {
        Ok(Some((record, bodies_end))) => {
            let mut bytes = fs::read(&path)?;
            bytes.truncate(bodies_end as usize);
            (bytes, record)
        }
        Ok(None) | Err(_) => (vec![], ModuleRecord::default()),
    };
    for entry in new {
        if record.contains_key(&entry.identity) {
            continue;
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
        let file_name = dir_entry?.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with(&prefix) && file_name != name.as_str() {
            let _ = fs::remove_file(dir.join(file_name.as_ref()));
        }
    }
    Ok(())
}

/// A body whose typed expression lies in its module's file, and which `Program::global_values`
/// holds as it was parsed.
struct StoredBody {
    /// The file.
    path: Arc<PathBuf>,
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
    read: Map<BodyLocation, TypedExpr>,
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
    /// read back when instantiation asks for it (`typed_expr_at`).
    pub fn check_bodies_through_typed_modules(
        &mut self,
        tc: &TypeCheckContext,
        modules: &[Name],
        config: &Configuration,
    ) -> Result<(), Errors> {
        // A check that swallows type errors reports none, so what it passes is not known to pass.
        assert!(!tc.error_tolerant);
        let mut errors = Errors::empty();

        // The spans read back share the source files the parser made.
        for module in &self.modules {
            let input = &module.source.input;
            self.typed_modules
                .source_files
                .insert(input.file_path.clone(), input.clone());
        }

        // The bodies of `modules`, by the module each is defined in.
        let names: Vec<FullName> = self
            .global_values
            .iter()
            .filter(|(name, gv)| match gv.expr {
                SymbolExpr::Simple(_) => modules.contains(&name.module()),
                SymbolExpr::Method(_) => true,
            })
            .map(|(name, _)| name.clone())
            .collect();
        let mut bodies_by_module: Map<Name, Vec<BodyLocation>> = Map::default();
        for loc in self.body_locations(&names, |method| modules.contains(&method.define_module)) {
            let (_te, _scm, module) = self.body_at(&loc);
            bodies_by_module.entry(module).or_default().push(loc);
        }

        // Skip the bodies the files record.
        let mut module_hashes: Map<Name, String> = Map::default();
        let mut to_check: Vec<(BodyLocation, Name, EntityIdentity)> = vec![];
        for (module, locs) in bodies_by_module {
            let module_hash = self.module_dependency_hash(&module, config)?;
            let path = Arc::new(
                Path::new(TYPED_MODULES_PATH).join(module_file_name(&module, &module_hash)),
            );
            let record =
                SourceFile::sharing(&mut self.typed_modules.source_files, || read_record(&path));
            let record = match record {
                Ok(Some((record, _bodies_end))) => record,
                Ok(None) => ModuleRecord::default(),
                Err(why) => {
                    warn_msg(&format!(
                        "Failed to read cache file \"{}\": {}.",
                        path.to_string_lossy(),
                        why
                    ));
                    ModuleRecord::default()
                }
            };
            for loc in locs {
                let identity = self.entity_identity_at(&loc);
                match record.get(&identity) {
                    Some(entry) => {
                        for (k, v) in &entry.opaque_types {
                            self.opaque_types
                                .entry(k.clone())
                                .or_default()
                                .extend(v.iter().cloned());
                        }
                        self.typed_modules.stored.insert(
                            loc,
                            StoredBody {
                                path: path.clone(),
                                identity,
                                range: entry.body.clone(),
                                warning_facts: entry.warning_facts.clone(),
                            },
                        );
                    }
                    None => to_check.push((loc, module.clone(), identity)),
                }
            }
            module_hashes.insert(module, module_hash);
        }

        // Check the rest, and keep what passes with no diagnostic for the files.
        let mut new_entries: Map<Name, Vec<NewEntry>> = Map::default();
        let locs = to_check.iter().map(|(loc, _, _)| loc.clone()).collect();
        let results = self.check_bodies(locs, tc, None, config)?;
        for ((loc, module, identity), (_loc, checked)) in to_check.into_iter().zip(results) {
            match checked {
                Ok(checked) => {
                    if !checked.diagnostics.has_diagnostics() {
                        new_entries.entry(module).or_default().push(NewEntry {
                            bytes: postcard::to_allocvec(&(&identity, &checked.te)).unwrap(),
                            identity,
                            opaque_types: checked.opaque_types.clone(),
                            warning_facts: warning_facts(&checked.te.expr),
                        });
                    }
                    errors.append(self.install_checked_body(&loc, checked));
                }
                Err(e) => errors.append(e),
            }
        }
        if !new_entries.is_empty() {
            match create_dir_guarding_project_links(
                Path::new(TYPED_MODULES_PATH),
                DOT_FIXLANG_LINK_HINT,
            ) {
                Ok(dir) => {
                    for (module, entries) in new_entries {
                        if let Err(e) =
                            add_to_module_file(&dir, &module, &module_hashes[&module], entries)
                        {
                            warn_msg(&format!(
                                "Failed to write cache file \"{}\": {}.",
                                dir.join(module_file_name(&module, &module_hashes[&module]))
                                    .to_string_lossy(),
                                e
                            ));
                        }
                    }
                }
                Err(errs) => {
                    for err in errs.errors() {
                        warn_msg(&err.msg);
                    }
                }
            }
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
    /// A body the file no longer holds is type-checked again.
    pub fn typed_expr_at(
        &mut self,
        loc: &BodyLocation,
        tc: &TypeCheckContext,
    ) -> Result<TypedExpr, Errors> {
        let Some(stored) = self.typed_modules.stored.get(loc) else {
            let (te, _scm, _module) = self.body_at(loc);
            return Ok(te.clone());
        };
        if let Some(te) = self.typed_modules.read.get(loc) {
            return Ok(te.clone());
        }
        let read = SourceFile::sharing(&mut self.typed_modules.source_files, || {
            read_body(&stored.path, &stored.range, &stored.identity)
        });
        let te = match read {
            Ok(Some(te)) => te,
            failed => {
                if let Err(why) = failed {
                    warn_msg(&format!(
                        "Failed to read cache file \"{}\": {}.",
                        stored.path.to_string_lossy(),
                        why
                    ));
                }
                // The opaque types of the body are already known from the record.
                let checked = self.check_body(loc, &self.create_name_resolution_env(), tc, None)?;
                if checked.diagnostics.has_error() {
                    return Err(checked.diagnostics);
                }
                checked.te
            }
        };
        self.typed_modules.read.insert(loc.clone(), te.clone());
        Ok(te)
    }
}
