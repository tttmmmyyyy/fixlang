// The syntax trees of the bodies `fix build` and `fix check` parse, kept in one file for each source
// file under `PARSED_MODULES_PATH`.
//
// A build parses every source file before it type-checks any body, and type-checks only the bodies
// the files of `typed_modules` lack. So that the syntax tree of a body is not held in memory from
// the parse until then, the bodies of a source file are written to its file as soon as the source
// file is parsed, and each is read back when it is type-checked (`ParsedBody::expr`):
//
//   [expression] [expression] ... [index] [footer]
//
// The index lists each body by where its expression is written in the source file, and the footer
// is that of every cache file (`cache_files`). The expressions are serialized inside
// `SourceFile::with_spans_within`, so a span is written as a range of the source file alone, and a
// type annotation keeps the position its diagnostics point at.
//
// A file is named after its source, by the path of a source file or the name of a source the
// compiler writes, and after the digest of what the parse is made from (`ParseOrigin`), so a file
// answers for exactly one parse, and replaces the files of the other contents of its source. A build that finds the file of a source
// file it parses reads only the index, and leaves the file as it is. When a body fails to read back
// from the file, the source file is parsed again, and the bodies are taken from that parse.

use crate::ast::expr::ExprNode;
use crate::ast::program::{Body, ParsedBody, Program, SymbolExpr};
use crate::constants::PARSED_MODULES_PATH;
use crate::elaboration::cache_files::{
    find_index, lies_within_entries, map_file, readable_file_prefix, warn_unreadable_file, CacheDirs,
    warn_unwritable_file, CacheFileWriter, FileMagic,
};
use crate::hash::HashSource;
use crate::misc::{Map, Set};
use crate::parse::parser::{parse_origin, ParseOrigin};
use crate::parse::sourcefile::Span;
use build_time::build_time_utc;
use memmap2::Mmap;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

/// The last bytes of a file of parsed bodies: they mark the file as one.
const FILE_MAGIC: &FileMagic = b"FIXPAMOD";

/// Where an expression is written in its source file, by which a file of parsed bodies lists the
/// expression.
type ExprKey = (usize, usize);

/// The key of the expression written at `span`.
fn expr_key(span: &Span) -> ExprKey {
    (span.start, span.end)
}

/// The bodies a file of parsed bodies holds: where each lies in the file, by the key of its
/// expression.
type ParsedIndex = Map<ExprKey, Range<u64>>;

/// A body written to the file of parsed bodies of the source file it was parsed from.
#[derive(Clone)]
pub struct StoredParsedBody {
    /// The file, as this build found it or wrote it.
    file: Arc<MappedParsedFile>,
    /// Where the expression lies in the file.
    range: Range<u64>,
    /// Where the expression is written in the source file.
    source: Span,
}

impl StoredParsedBody {
    /// Where the expression is written.
    pub fn source(&self) -> &Span {
        &self.source
    }

    /// The expression, read back from the file. An expression the file fails to hold is taken from
    /// a parse of the source file made again, which then replaces the file, so that the next build
    /// reads every body back.
    pub fn expr(&self) -> Arc<ExprNode> {
        let bytes = &self.file.bytes[self.range.start as usize..self.range.end as usize];
        let read = self
            .file
            .origin
            .source
            .with_spans_within(|| postcard::from_bytes::<Arc<ExprNode>>(bytes));
        let why = match read {
            Ok(expr) if expr.source.as_ref() == Some(&self.source) => return expr,
            Ok(_) => "it holds another body where a body was recorded".to_string(),
            Err(e) => e.to_string(),
        };
        let reparsed = self.file.reparsed.get_or_init(|| {
            warn_unreadable_file(&parsed_file_path(&self.file.file_name), &why);
            let origin = &self.file.origin;
            // The source parsed into the program this build holds, from the content it keeps.
            let Ok(mut program) = parse_origin(origin) else {
                panic!(
                    "a source parsed once fails to parse again: \"{}\"",
                    origin.source.file_path.to_string_lossy()
                )
            };
            let bodies = bodies_to_store(&mut program, origin);
            let file = &self.file;
            write_parsed_file(&file.cache_dirs, &file.file_name, &file.prefix, origin, &bodies);
            bodies
                .into_iter()
                .map(|(key, body)| (key, body.expr()))
                .collect()
        });
        reparsed
            .get(&expr_key(&self.source))
            .unwrap_or_else(|| {
                panic!(
                    "a parse of \"{}\" made again lacks the body written at {}..{}",
                    self.file.origin.source.file_path.to_string_lossy(),
                    self.source.start,
                    self.source.end
                )
            })
            .clone()
    }
}

/// A file of parsed bodies mapped into memory, as this build found it or wrote it. Each body read
/// back from the file is read from the mapping, so a file another build puts in place meanwhile, or
/// removes, leaves the body where it was.
struct MappedParsedFile {
    /// What the bodies of the file were parsed from.
    origin: ParseOrigin,
    /// The name of the file, without its directory.
    file_name: String,
    /// The part the file of every content of its source begins its name with.
    prefix: String,
    /// The bytes of the file.
    bytes: Mmap,
    /// The cache directories of the build, which a file written anew is written in.
    cache_dirs: CacheDirs,
    /// The bodies of a parse of the source file made again, by the keys of their expressions, once
    /// a body has failed to read back from the file.
    reparsed: OnceLock<Map<ExprKey, Arc<ExprNode>>>,
}

/// The path of the file of parsed bodies named `file_name`.
fn parsed_file_path(file_name: &str) -> PathBuf {
    Path::new(PARSED_MODULES_PATH).join(file_name)
}

/// The name of the file of parsed bodies of `origin`, a content of the source named `source_name`,
/// and the part the file of every content of that source begins its name with.
///
/// `origin.source` holds its content, as it does once it has been parsed.
fn parsed_file_name(origin: &ParseOrigin, source_name: &str) -> (String, String) {
    let mut hash_source = HashSource::default();
    let source_hash = origin.source.hash().unwrap_or_else(|errs| {
        panic!(
            "a source parsed fails to give its content: \"{}\": {}",
            origin.source.file_path.to_string_lossy(),
            errs
        )
    });
    hash_source.push_text(&source_hash);
    match &origin.renaming {
        Some(renaming) => hash_source.push_list([&renaming.written, &renaming.compiled]),
        None => hash_source.push_list::<[&str; 0]>([]),
    }
    hash_source.push_text(&origin.c_numeric_types.to_string());
    hash_source.push_text(build_time_utc!());
    let prefix = readable_file_prefix(source_name);
    (format!("{}{}", prefix, hash_source.finish()), prefix)
}

/// The index of the file of parsed bodies whose bytes are `bytes`, or `None` when the bytes are
/// not such a file.
fn decode_parsed_file(bytes: &[u8]) -> Option<ParsedIndex> {
    let range = find_index(bytes, FILE_MAGIC)?;
    let entries: Vec<(ExprKey, Range<u64>)> =
        postcard::from_bytes(&bytes[range.start as usize..range.end as usize]).ok()?;
    if entries
        .iter()
        .any(|(_, body)| !lies_within_entries(body, range.start))
    {
        return None;
    }
    Some(entries.into_iter().collect())
}

/// Writes the bodies of `program`, parsed from `origin`, to the file of parsed bodies of `origin`
/// in the directory `cache_dirs` gives, or finds them in the file an earlier build wrote, and holds
/// each of them as one read back from the file (see `bodies_to_store` for which bodies). The bodies
/// stay in memory when the file can be neither found nor written.
///
/// A file found that cannot be read, or lacks a body, is reported as a warning, and written anew.
pub fn store_parsed_bodies(
    program: &mut Program,
    origin: ParseOrigin,
    source_name: &str,
    cache_dirs: &CacheDirs,
) {
    let bodies = bodies_to_store(program, &origin);
    if bodies.is_empty() {
        return;
    }
    let (file_name, prefix) = parsed_file_name(&origin, source_name);
    let found = find_parsed_file(&file_name, &bodies)
        .or_else(|| write_parsed_file(cache_dirs, &file_name, &prefix, &origin, &bodies));
    let Some((bytes, index)) = found else {
        return;
    };
    let file = Arc::new(MappedParsedFile {
        origin,
        file_name,
        prefix,
        bytes,
        cache_dirs: cache_dirs.clone(),
        reparsed: OnceLock::new(),
    });
    for (key, body) in bodies {
        let ParsedBody::InMemory(expr) = body else {
            unreachable!("a body to store held outside memory")
        };
        let source = expr.source.clone().unwrap();
        *body = ParsedBody::Stored(StoredParsedBody {
            file: file.clone(),
            range: index[&key].clone(),
            source,
        });
    }
}

/// The file of parsed bodies named `file_name` an earlier build wrote, mapped into memory, with its
/// index, when it holds every one of `bodies`. A file that cannot be read, or lacks one of
/// `bodies`, is reported as a warning, and answers `None`, as no file does.
fn find_parsed_file(
    file_name: &str,
    bodies: &[(ExprKey, &mut ParsedBody)],
) -> Option<(Mmap, ParsedIndex)> {
    let path = parsed_file_path(file_name);
    let bytes = match map_file(&path) {
        Ok(bytes) => bytes?,
        Err(e) => {
            warn_unreadable_file(&path, &e.to_string());
            return None;
        }
    };
    match decode_parsed_file(&bytes) {
        Some(index) if bodies.iter().all(|(key, _)| index.contains_key(key)) => Some((bytes, index)),
        Some(_) => {
            warn_unreadable_file(&path, "it lacks a body of its source file");
            None
        }
        None => {
            warn_unreadable_file(&path, "it is not a file of parsed bodies");
            None
        }
    }
}

/// The bodies of `program`, parsed from `origin`, that its file of parsed bodies holds, each with
/// the key of its expression: those held in memory whose expressions are written in
/// `origin.source`.
///
/// Two bodies of one source are two definitions, written at two places, so a key names one body.
fn bodies_to_store<'p>(
    program: &'p mut Program,
    origin: &ParseOrigin,
) -> Vec<(ExprKey, &'p mut ParsedBody)> {
    let mut bodies = program.parsed_bodies_mut();
    bodies.retain(|(_, body)| {
        matches!(body, ParsedBody::InMemory(expr)
            if expr.source.as_ref().is_some_and(|span| span.input == origin.source))
    });
    let mut keys: Set<ExprKey> = Set::default();
    for (key, _) in &bodies {
        assert!(
            keys.insert(*key),
            "two bodies of \"{}\" written at {}..{}",
            origin.source.file_path.to_string_lossy(),
            key.0,
            key.1
        );
    }
    bodies
}

/// Writes the file of parsed bodies `file_name` in the directory `cache_dirs` gives, holding the
/// expressions of `bodies`, parsed from `origin`, and answers it mapped into memory, with its index.
/// A file that cannot be written is reported as a warning, and answers `None`.
fn write_parsed_file(
    cache_dirs: &CacheDirs,
    file_name: &str,
    prefix: &str,
    origin: &ParseOrigin,
    bodies: &[(ExprKey, &mut ParsedBody)],
) -> Option<(Mmap, ParsedIndex)> {
    let dir = cache_dirs.writable_dir(PARSED_MODULES_PATH)?;
    match write_parsed_file_in(&dir, file_name, prefix, origin, bodies) {
        Ok(written) => Some(written),
        Err(e) => {
            warn_unwritable_file(&dir.join(file_name), &e);
            None
        }
    }
}

/// Writes the file of parsed bodies `file_name` in `dir`, holding the expressions of `bodies`,
/// parsed from `origin`, and answers it mapped into memory, with its index.
fn write_parsed_file_in(
    dir: &Path,
    file_name: &str,
    prefix: &str,
    origin: &ParseOrigin,
    bodies: &[(ExprKey, &mut ParsedBody)],
) -> io::Result<(Mmap, ParsedIndex)> {
    let mut writer = CacheFileWriter::create(dir)?;
    let mut entries: Vec<(ExprKey, Range<u64>)> = vec![];
    let mut end = 0;
    for (key, body) in bodies {
        let ParsedBody::InMemory(expr) = body else {
            unreachable!("a body to store held outside memory")
        };
        let bytes = origin
            .source
            .with_spans_within(|| postcard::to_allocvec(expr).unwrap());
        writer.write_at(end, &bytes)?;
        let start = end;
        end += bytes.len() as u64;
        entries.push((*key, start..end));
    }
    entries.sort_by_key(|(key, _)| *key);
    let bytes = writer.finish(
        end,
        &postcard::to_allocvec(&entries).unwrap(),
        FILE_MAGIC,
        file_name,
        prefix,
    )?;
    Ok((bytes, entries.into_iter().collect()))
}

impl Program {
    /// Every body of this program as the parser produced it or the compiler built it, each with the
    /// key of where its expression is written: the bodies of the simple values not yet type-checked,
    /// and what the implementations of traits write for their members. A body whose expression is
    /// written nowhere is left out.
    fn parsed_bodies_mut(&mut self) -> Vec<(ExprKey, &mut ParsedBody)> {
        let mut bodies = vec![];
        for gv in self.global_values.values_mut() {
            if let SymbolExpr::Simple(Body::Unchecked(body)) = &mut gv.expr {
                bodies.push(&mut body.expr);
            }
        }
        for impls in self.trait_env.impls.values_mut() {
            for impl_ in impls {
                for member in impl_.members.values_mut() {
                    bodies.push(&mut member.body);
                }
            }
        }
        bodies
            .into_iter()
            .filter_map(|body| Some((expr_key(&body.source()?), body)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::expr::{Expr, ExprNode};
    use crate::ast::name::FullName;
    use crate::ast::program::{Body, ParsedBody, SymbolExpr};
    use crate::ffi::CNumericTypes;
    use super::parsed_file_name;
    use crate::parse::parser::{parse_origin, ModuleRenaming, ParseOrigin};
    use crate::parse::sourcefile::{SourceFile, Span};
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Every place an expression or a type written in `expr` is written at: the span of each node,
    /// and of each type a type annotation writes.
    fn positions(expr: &Arc<ExprNode>) -> Vec<Option<Span>> {
        let mut positions = vec![];
        expr.walk_nodes(&mut |node| {
            positions.push(node.source.clone());
            if let Expr::TyAnno(_, ty) = &*node.expr {
                positions.push(ty.get_source().clone());
            }
        });
        positions
    }

    /// A body written to its file of parsed bodies reads back with every position its source gives
    /// it: those of its expressions, and those of the types its annotations write, which the
    /// diagnostics of a type annotation point at.
    #[test]
    fn a_parsed_body_reads_back_with_the_positions_of_its_expressions_and_types() {
        let source = SourceFile::from_file_path_and_content(
            PathBuf::from("main.fix"),
            "module Main;\n\nmain : IO () = println((1 : I64).to_string);\n".to_string(),
        );
        let origin = ParseOrigin {
            source: source.clone(),
            renaming: None,
            c_numeric_types: CNumericTypes::of_target("x86_64-unknown-linux-gnu"),
        };
        let program = parse_origin(&origin).unwrap();
        let SymbolExpr::Simple(Body::Unchecked(body)) =
            &program.global_values[&FullName::from_strs(&["Main"], "main")].expr
        else {
            panic!("`main` was parsed as another kind of body")
        };
        let ParsedBody::InMemory(expr) = &body.expr else {
            panic!("a body parsed alone was held outside memory")
        };

        let bytes = source.with_spans_within(|| postcard::to_allocvec(expr).unwrap());
        let read: Arc<ExprNode> =
            source.with_spans_within(|| postcard::from_bytes(&bytes).unwrap());

        let start = source.string().unwrap().find("I64)").unwrap();
        let annotated = Span {
            input: source.clone(),
            start,
            end: start + 3,
        };
        assert!(
            positions(expr).contains(&Some(annotated)),
            "the parse gave the annotated type no position"
        );
        assert!(
            positions(&read) == positions(expr),
            "the body read back is written at other places than the body parsed"
        );
    }

    /// The name of a file of parsed bodies follows what the parse is made from beside the source:
    /// the C numeric types, which decide the Fix types the C types of an `FFI_CALL` stand for, and
    /// the renaming of the module, which decides the names the bodies are parsed under.
    #[test]
    fn the_name_of_a_file_of_parsed_bodies_follows_the_c_numeric_types_and_the_renaming() {
        let origin = ParseOrigin {
            source: SourceFile::from_file_path_and_content(
                PathBuf::from("main.fix"),
                "module Main;\n\nmain : IO () = pure();\n".to_string(),
            ),
            renaming: None,
            c_numeric_types: CNumericTypes::of_target("x86_64-unknown-linux-gnu"),
        };
        let name = |origin: &ParseOrigin| parsed_file_name(origin, "main.fix").0;

        let mut wider_int = origin.clone();
        wider_int.c_numeric_types.int_bits = origin.c_numeric_types.int_bits * 2;
        assert_ne!(
            name(&origin),
            name(&wider_int),
            "a C `int` of another width gives the parse other types, and the name of its file \
             stayed where it was"
        );

        let mut renamed = origin.clone();
        renamed.renaming = Some(ModuleRenaming {
            written: "Main".to_string(),
            compiled: "Other.Main".to_string(),
        });
        assert_ne!(
            name(&origin),
            name(&renamed),
            "a module compiled under another name gives the parse other names, and the name of its \
             file stayed where it was"
        );
    }
}
