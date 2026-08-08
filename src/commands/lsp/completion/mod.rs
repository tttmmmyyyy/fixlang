// LSP completion feature handlers.

mod import;
mod index;
mod repair;
mod score;
mod symbols;

use self::import::{import_completion_items, import_context_at};
use self::index::CompletionIndex;
use self::repair::repair_for_completion;
use self::score::{
    assign_tier, assign_tier_no_unify, dot_context_low_priority_sort_text, namespace_match,
    sort_text_for, PredMemo, Tier,
};
use self::symbols::{build_completion_item, type_symbols, value_symbols, CompletionSymbol};
use super::edit_import::create_text_edit_to_import;
use super::server::{send_response, LatestContent};
use super::util::{
    document_from_endnode, get_line_string_from_position, is_cursor_in_comment,
    parameters_of_global_value, position_to_bytes,
};
use crate::ast::expr::{hole_full_name, Expr, ExprNode};
use crate::ast::name::{FullName, NameSpace};
use crate::ast::program::{EndNode, Program, SymbolExpr};
use crate::ast::types::TypeNode;
use crate::configuration::{BuildConfigType, Configuration, DiagnosticsConfig, SubCommand};
use crate::constants::chars_allowed_in_identifiers;
use crate::dependency::lockfile::LockFileType;
use crate::elaboration::elaborate_via_config;
use crate::elaboration::typecheck::TypeCheckContext;
use crate::elaboration::typecheckcache::SharedTypeCheckCache;
use crate::error::Errors;
use crate::metafiles::project_file::ProjectFile;
use crate::misc::{to_absolute_path, Map};
use crate::parse::parser::parse_source_file;
use crate::parse::sourcefile::{SourceFile, Span};
use crate::write_log;
use lsp_types::{
    CompletionItem, CompletionParams, Documentation, InsertTextFormat, TextDocumentPositionParams,
    Uri,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Payload stashed in `CompletionItem::data`, carried from
/// `textDocument/completion` to `completionItem/resolve`.
#[derive(Serialize, Deserialize)]
struct ResolveData {
    /// The entity the item completes; resolve reads its documentation
    /// (and, for a global value, its parameter list) from it.
    node: EndNode,
    /// Where the completed name is being written; decides what resolve
    /// adds beyond the documentation.
    context: ResolveContext,
}

/// The syntactic position a completion item is offered in.
#[derive(Serialize, Deserialize)]
enum ResolveContext {
    /// An expression: resolve may append the argument snippet to the
    /// insert text and attach the auto-import edits, both of which
    /// need the typed line and the cursor position.
    Expression {
        /// The text of the cursor's line up to the cursor.
        typing_text: String,
        /// The document and cursor position of the completion request.
        position: TextDocumentPositionParams,
    },
    /// A component of an `import` statement: the completed name is
    /// written bare, so the documentation is all resolve adds.
    Import,
}

impl ResolveData {
    /// The serialized payload to stash in `CompletionItem::data`.
    fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

/// Handles the `textDocument/completion` LSP request: collects
/// candidate symbols (globals, type constructors, traits, associated
/// types) visible at the cursor and replies with a list of
/// `CompletionItem`s.
pub(super) fn handle_completion(
    id: u32,
    params: &CompletionParams,
    program: Option<&Program>,
    uri_to_content: &Map<Uri, LatestContent>,
    typecheck_cache: SharedTypeCheckCache,
) {
    let text_document_position = &params.text_document_position;

    // Don't offer completions while the cursor is inside a comment
    // (`//` or `/* */`): the user is writing prose, not code.
    if is_cursor_in_comment(uri_to_content, text_document_position) {
        send_response(id, Ok::<_, ()>(Vec::<CompletionItem>::new()));
        return;
    }

    let typing_text = get_typing_text(text_document_position, uri_to_content);

    // When the cursor is inside an `import` statement, complete module
    // names or the imported module's entities; the symbol candidates
    // collected below are for expressions and are meaningless there.
    if let Some(latest) = uri_to_content.get(&text_document_position.text_document.uri) {
        let cursor_byte = position_to_bytes(&latest.content, text_document_position.position);
        if let Some(import_ctx) = import_context_at(&latest.content, cursor_byte) {
            // Without a diagnostics snapshot there is nothing to
            // enumerate modules or entities from; reply empty and let
            // the next diagnostics run restore the candidates.
            let items = program
                .map(|program| {
                    import_completion_items(
                        &import_ctx,
                        program,
                        &latest.path,
                        text_document_position,
                    )
                })
                .unwrap_or_default();
            send_response(id, Ok::<_, ()>(items));
            return;
        }
    }

    // In dot-completion contexts, run the receiver-type extraction
    // pipeline so we can rank candidates by how well their receiver
    // position matches the typed receiver. On failure we silently fall
    // back to the legacy alphabetical list; no client-visible error.
    let dot_extract = if is_dot_function(&typing_text) {
        extract_receiver_type_and_program_for_dot_completion(
            text_document_position,
            uri_to_content,
            typecheck_cache,
        )
    } else {
        None
    };
    // The snapshot can be missing entirely when the last-saved file
    // failed to parse: `run_diagnostics` propagates that as `Err` and
    // the diagnostics thread never publishes a `DiagnosticsResult`, so
    // `last_diag` stays `None` in the server loop. In that state we
    // still need to reply — silently dropping the request makes the
    // client wait forever (looks like a server crash). The dot-context
    // pipeline does its own `error_tolerant` elaborate over the live
    // buffer, so it can still produce candidates; for non-dot contexts
    // without a snapshot we reply with an empty list and let the next
    // diagnostics run restore the full list.
    let Some(active_program) = dot_extract.as_ref().map(|d| &d.program).or(program) else {
        send_response(id, Ok::<_, ()>(Vec::<CompletionItem>::new()));
        return;
    };
    let dot_ranking = dot_extract.as_ref().map(|d| {
        write_log!(
            "[completion] dot-context receiver type: {}",
            d.receiver_type.to_string()
        );
        // Build a base typechecker over the active program; clone-
        // per-candidate inside `assign_tier` keeps unify substitutions
        // from leaking between candidates.
        let scratch_config = Configuration::diagnostics_mode(DiagnosticsConfig::default()).ok();
        let tc_template = scratch_config
            .as_ref()
            .map(|cfg| d.program.create_typechecker(cfg));
        DotRanking {
            receiver_type: d.receiver_type.clone(),
            index: CompletionIndex::build(&d.program),
            tc_template,
        }
    });

    let namespace = extract_namespace_from_typing_text(&typing_text);
    let is_in_namespace = |name: &FullName| namespace.is_suffix_of(&name.namespace);
    let expression_context = || ResolveContext::Expression {
        typing_text: typing_text.clone(),
        position: text_document_position.clone(),
    };

    let mut items = vec![];

    // Collect the visible global-value candidates once. This is also the
    // order they're sent in; the client re-sorts by `sortText`, so the
    // order only needs to be stable, not meaningful.
    let global_candidates: Vec<CompletionSymbol> = value_symbols(active_program)
        .into_iter()
        .filter(|symbol| is_in_namespace(&symbol.name))
        .collect();

    // In a dot context the per-candidate tier needs a unify probe
    // (`assign_tier` → `try_unify_receiver` → `reduce_predicates`) that
    // dominates the request's latency. Each probe clones the shared
    // `TypeCheckContext` and is otherwise independent, so compute the
    // tiers across worker threads. `None` outside dot contexts, where no
    // tier is assigned at all.
    let global_tiers: Option<Vec<Tier>> = dot_ranking.as_ref().map(|ranking| {
        let names: Vec<&FullName> = global_candidates.iter().map(|s| &s.name).collect();
        match &ranking.tc_template {
            Some(tc) => {
                // One memo per request, shared across all candidates (and
                // worker threads) so identical constraint sets are probed
                // only once.
                let memo = PredMemo::default();
                tiers_in_parallel(&names, ranking, active_program, tc, &memo)
            }
            None => names
                .iter()
                .map(|&n| assign_tier_no_unify(n, &ranking.index, &ranking.receiver_type))
                .collect(),
        }
    });

    for (idx, symbol) in global_candidates.into_iter().enumerate() {
        let sort_text = if let Some(ranking) = &dot_ranking {
            // `global_tiers` is `Some` whenever `dot_ranking` is, and is
            // indexed in lockstep with `global_candidates`.
            let tier = global_tiers.as_ref().unwrap()[idx];
            let ns_match = namespace_match(
                ranking.receiver_type.toplevel_tycon().as_deref(),
                &symbol.name,
            );
            Some(sort_text_for(
                tier,
                ns_match,
                symbol.deprecated,
                &symbol.name,
            ))
        } else if symbol.deprecated {
            // Non-dot context. Live items keep `sort_text = None` so the
            // client's default (label-based, possibly fuzzy) ordering
            // applies. Deprecated items get a `~` prefix on their sort
            // key — `~` (0x7E) is greater than every character that can
            // appear in a Fix identifier or namespace separator, so
            // deprecated items always sort below every live item whose
            // sort key is its label.
            Some(format!("~{}", symbol.name.to_string()))
        } else {
            None
        };
        let label = symbol.name.to_string();
        let mut item = build_completion_item(symbol, label, expression_context());
        item.sort_text = sort_text;
        items.push(item);
    }
    for symbol in type_symbols(active_program) {
        if !is_in_namespace(&symbol.name) {
            continue;
        }
        // Types can't appear after a dot in Fix, so they shouldn't
        // outrank function candidates.
        let sort_text = dot_ranking
            .as_ref()
            .map(|_| dot_context_low_priority_sort_text(&symbol.name));
        let label = symbol.name.to_string();
        let mut item = build_completion_item(symbol, label, expression_context());
        item.sort_text = sort_text;
        items.push(item);
    }
    send_response(id, Ok::<_, ()>(items));
}

/// Compute the dot-completion `Tier` for each candidate name, spreading
/// the work across worker threads. Each candidate's tier comes from an
/// independent unify probe (`assign_tier`, which clones `tc`), so there
/// is no shared mutable state and the probes parallelize cleanly. The
/// returned tiers are in the same order as `names`.
fn tiers_in_parallel(
    names: &[&FullName],
    ranking: &DotRanking,
    program: &Program,
    tc: &TypeCheckContext,
    memo: &PredMemo,
) -> Vec<Tier> {
    let n = names.len();
    // Below this many candidates the thread setup costs more than just
    // running the probes sequentially.
    const PARALLEL_THRESHOLD: usize = 64;
    let workers = num_cpus::get();
    let tier_of = |name: &FullName| {
        assign_tier(
            name,
            &ranking.index,
            &ranking.receiver_type,
            program,
            tc,
            memo,
        )
    };
    if workers <= 1 || n < PARALLEL_THRESHOLD {
        return names.iter().map(|&name| tier_of(name)).collect();
    }

    // Even chunks; the last chunk absorbs the remainder. `scope`
    // guarantees every worker finishes before the borrowed
    // `names`/`ranking`/`program`/`tc`/`memo` go out of scope, so plain
    // shared references suffice (no `Arc`/clone needed to hand them to
    // threads); the memo is a `Mutex`, so the workers share one cache.
    let chunk_size = n.div_ceil(workers);
    let mut tiers = Vec::with_capacity(n);
    std::thread::scope(|s| {
        let handles: Vec<_> = names
            .chunks(chunk_size)
            .map(|chunk| {
                let tier_of = &tier_of;
                s.spawn(move || {
                    chunk
                        .iter()
                        .map(|&name| tier_of(name))
                        .collect::<Vec<Tier>>()
                })
            })
            .collect();
        for h in handles {
            tiers.extend(h.join().expect("completion ranking worker thread panicked"));
        }
    });
    tiers
}

/// Bundle of the data the dot-completion ranking flow needs: the
/// receiver type extracted from the live buffer, the bucket index
/// over the snapshot's globals, and a scratch `TypeCheckContext`
/// (cloned per candidate inside `score::assign_tier`) for the Tier 1
/// → Tier 0 unify promotion.
///
/// `tc_template` may be `None` if the scratch `Configuration` couldn't
/// be built — e.g. the host environment can't satisfy
/// `CTypeSizes::load_or_check`. In that case the unify-based promotion
/// is silently skipped and `assign_tier_no_unify` is used, leaving the
/// ranking at the bucket-only Tier 1/2/3 level.
struct DotRanking {
    receiver_type: Arc<TypeNode>,
    index: CompletionIndex,
    tc_template: Option<TypeCheckContext>,
}

/// Bundle returned by the dot-completion extraction pipeline: the
/// receiver type read off the inserted hole's curried signature, plus
/// the fully-elaborated `Program` produced from the repaired live
/// buffer. The program is used to enumerate candidates and to
/// instantiate schemes for unify, so a stale or parse-erroring
/// snapshot doesn't suppress Main:: items.
pub(super) struct DotExtraction {
    pub receiver_type: Arc<TypeNode>,
    pub program: Program,
}

/// Run the dot-completion type-extraction pipeline for a single
/// completion request: repair the live buffer at the cursor (replacing
/// the post-dot identifier with `?`), re-elaborate via
/// `elaborate_via_config`, and return the receiver type read off the
/// inserted hole's inferred curried type.
///
/// `n = 0` is hard-coded: the receiver is treated as the last element
/// of the hole's curried sources. Callers must check `is_dot_function`
/// before invoking this — non-dot contexts must not pay the elaborate
/// cost.
pub(super) fn extract_receiver_type_and_program_for_dot_completion(
    text_document_position: &TextDocumentPositionParams,
    uri_to_content: &Map<Uri, LatestContent>,
    typecheck_cache: SharedTypeCheckCache,
) -> Option<DotExtraction> {
    let uri = &text_document_position.text_document.uri;
    let latest = uri_to_content.get(uri)?;
    let live_buffer = &latest.content;
    let cursor_byte = position_to_bytes(live_buffer, text_document_position.position);
    let repaired = repair_for_completion(live_buffer, cursor_byte)?;
    let abs_path = to_absolute_path(&latest.path).ok()?;
    let program = run_completion_elaborate(
        &abs_path,
        repaired.source,
        typecheck_cache,
        repaired.cursor_byte,
    )
    .ok()?;
    let cursor = SourcePosLite {
        path: abs_path,
        byte: repaired.cursor_byte,
    };
    let hole = find_innermost_hole_at(&program, &cursor)?;
    let hole_ty = hole.type_.as_ref()?;
    let (srcs, _) = hole_ty.collect_app_src(usize::MAX);
    let receiver_type = srcs.into_iter().last()?;
    Some(DotExtraction {
        receiver_type,
        program,
    })
}

/// Drive the elaborate pipeline against a configuration that swaps in
/// `repaired_content` for `path`'s on-disk contents. Mirrors the
/// initial setup in `run_diagnostics` — read the project file, build a
/// `DiagnosticsConfig`, apply lockfile — then plants the live override
/// just before invoking elaborate.
///
/// `typecheck_cache` is the diagnostics thread's in-memory cache,
/// shared in so unchanged modules' globals hit cached entries instead
/// of paying disk I/O via the default `FileCache`. Only the cursor's
/// file's dependency hash changes per request (the live override only
/// touches that one path), so every other module is a cache hit.
fn run_completion_elaborate(
    path: &PathBuf,
    repaired_content: String,
    typecheck_cache: SharedTypeCheckCache,
    cursor_byte: usize,
) -> Result<Program, Errors> {
    let proj_file = ProjectFile::read_root_file()?;
    let files = proj_file.get_files(BuildConfigType::Test);

    // Restrict the typecheck pass to the gv the cursor sits in. If
    // the live buffer doesn't parse, or the cursor isn't inside any
    // body, fall back to typing every gv in the target file (slower
    // but correct).
    let target_symbols =
        find_enclosing_gv(path, &repaired_content, cursor_byte).map(|name| vec![name]);

    let mut overrides = Map::default();
    overrides.insert(path.clone(), repaired_content);
    let diag_config = DiagnosticsConfig {
        files,
        live_source_overrides: Arc::new(overrides),
        target_symbols,
        error_tolerant: true,
    };
    let mut config = Configuration::diagnostics_mode(diag_config)?;
    config.type_check_cache = typecheck_cache;
    proj_file.set_config(&mut config)?;
    proj_file
        .open_or_auto_update_lock_file(LockFileType::Lsp)?
        .set_config(&mut config)?;

    elaborate_via_config(&config)
}

/// Locate the global value whose body contains the cursor.
///
/// Performs a one-shot parse of the live buffer (no typecheck) and
/// walks the parsed `global_values`, returning the `FullName` of the
/// first definition whose body span covers `cursor_byte`.
fn find_enclosing_gv(abs_path: &PathBuf, content: &str, cursor_byte: usize) -> Option<FullName> {
    // `parse_source_file` requires a `Configuration` so that
    // `parse_module` can inject FFI type aliases sized by
    // `c_type_sizes`. That injection does not affect the discovered
    // global-value names or source spans, so any successfully-built
    // configuration is fine here.
    let config = Configuration::release_mode(SubCommand::Build).ok()?;
    let source_file = SourceFile::from_file_path_and_content(abs_path.clone(), content.to_string());
    let program = parse_source_file(source_file, &config).ok()?;
    let covers = |span: Option<&Span>| span.map(|s| s.includes_byte(cursor_byte)).unwrap_or(false);
    for (name, gv) in program.global_values.iter() {
        let hit = match &gv.expr {
            SymbolExpr::Simple(e) => covers(e.expr.source.as_ref()),
            SymbolExpr::Method(impls) => impls.iter().any(|m| covers(m.expr.expr.source.as_ref())),
        };
        if hit {
            return Some(name.clone());
        }
    }
    None
}

/// Path-and-byte cursor pair used to identify the hole in the
/// re-elaborated AST. Kept as a path rather than a `SourcePos` so the
/// caller doesn't need to construct a `SourceFile` cache that nobody
/// reads.
struct SourcePosLite {
    path: PathBuf,
    byte: usize,
}

impl SourcePosLite {
    /// True when `span` belongs to the same file as `self.path` and
    /// covers `self.byte`. Inclusive on both ends, matching
    /// `Span::includes_pos_lsp`.
    fn includes(&self, span: &Span) -> bool {
        let span_path = to_absolute_path(&span.input.file_path).ok();
        let our_path = to_absolute_path(&self.path).ok();
        if span_path.is_none() || our_path.is_none() || span_path != our_path {
            return false;
        }
        span.includes_byte(self.byte)
    }
}

/// Find the innermost `Expr::Var(Std::#hole)` whose span contains the
/// cursor. "Innermost" = the match with the smallest span; this picks
/// the single hole the repair just inserted even when other holes
/// (e.g. ones the user wrote, or future repair-loop insertions) live
/// in nearby code.
fn find_innermost_hole_at(program: &Program, cursor: &SourcePosLite) -> Option<Arc<ExprNode>> {
    let target = hole_full_name();
    let mut best: Option<Arc<ExprNode>> = None;
    for (_name, gv) in &program.global_values {
        match &gv.expr {
            SymbolExpr::Simple(te) => {
                walk_for_hole(&te.expr, cursor, &target, &mut best);
            }
            SymbolExpr::Method(impls) => {
                for m in impls {
                    walk_for_hole(&m.expr.expr, cursor, &target, &mut best);
                }
            }
        }
    }
    best
}

/// Visit `expr` and update `best` if it is a `Var` reference to
/// `target` whose span contains `cursor` and is no larger than the
/// current best. Descends into children regardless, so deeper holes
/// can win over outer ones.
fn walk_for_hole(
    expr: &Arc<ExprNode>,
    cursor: &SourcePosLite,
    target: &FullName,
    best: &mut Option<Arc<ExprNode>>,
) {
    let Some(span) = expr.source.as_ref() else {
        // Synthetic / desugared nodes don't have a span; their
        // children might, so descend regardless.
        recurse_for_hole(expr, cursor, target, best);
        return;
    };
    if !cursor.includes(span) {
        return;
    }
    if let Expr::Var(v) = &*expr.expr {
        if v.name == *target {
            // Choose the smallest enclosing span — innermost wins.
            let take = match best {
                None => true,
                Some(prev) => {
                    // `best` is only ever assigned an expr whose `source`
                    // passed the let-else above, so it has a span.
                    let prev_span = prev
                        .source
                        .as_ref()
                        .expect("hole candidate recorded without a span");
                    let prev_len = prev_span.end - prev_span.start;
                    let cur_len = span.end - span.start;
                    cur_len <= prev_len
                }
            };
            if take {
                *best = Some(expr.clone());
            }
        }
    }
    recurse_for_hole(expr, cursor, target, best);
}

/// Walk every direct child of `expr`, calling `walk_for_hole` on each.
fn recurse_for_hole(
    expr: &Arc<ExprNode>,
    cursor: &SourcePosLite,
    target: &FullName,
    best: &mut Option<Arc<ExprNode>>,
) {
    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            walk_for_hole(func, cursor, target, best);
            for a in args {
                walk_for_hole(a, cursor, target, best);
            }
        }
        Expr::Lam(_, body) => walk_for_hole(body, cursor, target, best),
        Expr::Let(_pat, bound, val) => {
            walk_for_hole(bound, cursor, target, best);
            walk_for_hole(val, cursor, target, best);
        }
        Expr::If(c, t, e) => {
            walk_for_hole(c, cursor, target, best);
            walk_for_hole(t, cursor, target, best);
            walk_for_hole(e, cursor, target, best);
        }
        Expr::Match(c, arms) => {
            walk_for_hole(c, cursor, target, best);
            for (_, val) in arms {
                walk_for_hole(val, cursor, target, best);
            }
        }
        Expr::TyAnno(e, _) => walk_for_hole(e, cursor, target, best),
        Expr::MakeStruct(_, fields) => {
            for (_, _, e) in fields {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for e in args {
                walk_for_hole(e, cursor, target, best);
            }
        }
        Expr::Eval(side, main) => {
            walk_for_hole(side, cursor, target, best);
            walk_for_hole(main, cursor, target, best);
        }
    }
}

// Check if the user's typing text is in the form of a dot followed by namespaces or a function name
fn is_dot_function(typing_text: &str) -> bool {
    let mut chars = typing_text.chars().rev();
    let identifer_chars = chars_allowed_in_identifiers();
    while let Some(c) = chars.next() {
        if c == '.' {
            return true;
        }
        if !identifer_chars.contains(c) && c != ':' {
            return false;
        }
    }
    false
}

// Extract namespace from typing text string.
// This function performs string manipulation to extract namespace components from user input.
fn extract_namespace_from_typing_text(typing_text: &str) -> NameSpace {
    // Get the suffix of `typing_text` that consists of characters allowed in identifiers and colons.
    // Example: input "let x = Std::Array:" -> "Std::Array:"
    let identifer_chars = chars_allowed_in_identifiers();
    let suffix_byte_start = typing_text
        .char_indices()
        .rev()
        .find(|(_, c)| !(identifer_chars.contains(*c) || *c == ':'))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let namespace_part = &typing_text[suffix_byte_start..];

    // Remove the trailing colon
    // Example: "Std::Array:" -> "Std::Array"
    let namespace_part = namespace_part.trim_end_matches(':').to_string();

    // Split the text by "::". If the last component does not start with a uppercase letter, then drop it.
    let mut components = namespace_part.split("::").collect::<Vec<_>>();
    if let Some(last_component) = components.last() {
        let first_char = last_component.chars().nth(0);
        if let Some(first_char) = first_char {
            if !first_char.is_ascii_alphabetic() || !first_char.is_uppercase() {
                components.pop();
            }
        }
    }
    let namespace_str = components
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("::");

    // Convert the namespace string to a `NameSpace`.
    let namespace = NameSpace::parse(&namespace_str);
    if namespace.is_none() {
        return NameSpace::new(vec![]);
    }
    namespace.unwrap()
}

// Get the text of the line being typed by the user up to the cursor position.
fn get_typing_text(
    text_document_position: &TextDocumentPositionParams,
    uri_to_content: &Map<Uri, LatestContent>,
) -> String {
    let current_line = get_line_string_from_position(uri_to_content, text_document_position);
    let typing_text = current_line
        .map(|(line, byte_pos)| line[..byte_pos].to_string())
        .unwrap_or_default();
    typing_text
}

// Handle "completionItem/resolve" method.
// Add documentation to the completion item.
pub(super) fn handle_completion_resolve_document(
    id: u32,
    params: &CompletionItem,
    uri_to_content: &mut Map<Uri, LatestContent>,
    program: &Program,
) {
    let Some(data) = params.data.as_ref() else {
        // Items without stashed data (e.g. namespace components offered
        // inside an import statement) have no documentation to attach.
        send_response(id, Ok::<_, ()>(params.clone()));
        return;
    };
    let data = serde_json::from_value::<ResolveData>(data.clone());
    if let Err(e) = data {
        let msg = format!(
            "In textDocument/completion, failed to parse params.data as ResolveData: {}",
            e
        );
        write_log!("{}", msg);
        send_response(id, Err::<CompletionItem, String>(msg));
        return;
    }

    let ResolveData { node, context } = data.unwrap();

    // Get the documentation.
    let docs = document_from_endnode(&node, program);

    // Set the documentation into the given completion item.
    let docs = Documentation::MarkupContent(docs);
    let mut item = params.clone();
    item.documentation = Some(docs);

    let (typing_text, text_document_position) = match context {
        // In an import statement the completed name is written bare,
        // never as a call and never needing an import edit; the
        // documentation is all there is to add.
        ResolveContext::Import => {
            send_response(id, Ok::<_, ()>(item));
            return;
        }
        ResolveContext::Expression {
            typing_text,
            position,
        } => (typing_text, position),
    };

    // Is the user completing a function call after a dot?
    let has_dot = is_dot_function(&typing_text);

    // If the node is a global value with parameters defined in the document, then add the parameters to the insert text.
    match &node {
        EndNode::Expr(var, _) => {
            if var.name.is_global() {
                let params = parameters_of_global_value(&var.name, program);
                if let Some(mut params) = params {
                    // If the trigger character is ".", then remove the last parameter.
                    if has_dot {
                        params.pop();
                    }

                    // Append argument list to the insert text. Each parameter
                    // is wrapped in the user-hole syntax `?<name>` and turned
                    // into an LSP snippet tab-stop `${N:?<name>}` so editors
                    // that support snippets (VSCode, Neovim, Helix, …) put
                    // the cursor on the first hole, let the user tab through
                    // the rest, and pre-select each placeholder so typing
                    // overwrites it. The snippet text the editor expands to
                    // is still `?<name>`, which is a Fix hole expression —
                    // so even if the user dismisses the snippet without
                    // touching anything, the source type-checks (with hole
                    // diagnostics) instead of producing "undefined name `x`"
                    // or `f()` unit-call errors.
                    if let Some(insert_text) = &mut item.insert_text {
                        if params.len() > 0 {
                            let placeholders: Vec<String> = params
                                .iter()
                                .enumerate()
                                .map(|(i, p)| format!("${{{}:?{}}}", i + 1, p))
                                .collect();
                            *insert_text += "(";
                            *insert_text += &placeholders.join(", ");
                            *insert_text += ")";
                            item.insert_text_format = Some(InsertTextFormat::SNIPPET);
                        }
                    }
                }
            }
        }
        _ => {}
    };

    // Create TextEdits of import statements to import the completion item.
    let import_item_name = match node {
        EndNode::Expr(var, _) => Some(var.name.clone()),
        EndNode::Pattern(_, _) => None,
        EndNode::Type(ty) => Some(ty.name.clone()),
        EndNode::Trait(trait_) => Some(trait_.name.clone()),
        EndNode::Module(_) => None,
        EndNode::TypeOrTrait(name) => Some(name),
        EndNode::AssocType(assoc_type) => Some(assoc_type.name.clone()),
        EndNode::ValueDecl(name) => Some(name),
        EndNode::Field(_, _) | EndNode::Variant(_, _) => None,
        EndNode::InferredType(_) => None,
    };
    if let Some(import_item_name) = import_item_name {
        if let Some(latest_content) =
            uri_to_content.get_mut(&text_document_position.text_document.uri)
        {
            let edits = create_text_edit_to_import(&import_item_name, latest_content);
            if edits.len() > 0 {
                // If the cursor position is included in or near to any of the range of the text edits, do not apply the edits.
                let cursor = &text_document_position.position;
                if !edits.iter().any(|edit| {
                    edit.range.start.line <= cursor.line && cursor.line <= edit.range.end.line
                }) {
                    item.additional_text_edits = Some(edits);
                }
            }
        }
    }

    // Send the completion item.
    send_response(id, Ok::<_, ()>(item));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_namespace_from_typing_text_basic() {
        // Test case based on comment: "let x = Std::Array:"
        let result = extract_namespace_from_typing_text("let x = Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
        assert_eq!(result.is_absolute, false);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_simple() {
        // Test case: "Std::Array:"
        let result = extract_namespace_from_typing_text("Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_single() {
        // Test case: "Std:"
        let result = extract_namespace_from_typing_text("Std:");
        assert_eq!(result.names, vec!["Std".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_no_colon() {
        // Test case: "Std::Array" (no trailing colon)
        let result = extract_namespace_from_typing_text("Std::Array");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_lowercase_last() {
        // Test case: "Std::Array::get" - last component starts with lowercase, should be dropped
        let result = extract_namespace_from_typing_text("Std::Array::get");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_empty() {
        // Test case: empty string
        let result = extract_namespace_from_typing_text("");
        assert_eq!(result.names, Vec::<String>::new());
    }

    #[test]
    fn test_extract_namespace_from_typing_text_no_namespace() {
        // Test case: "SomeVariable" - no namespace separator
        let result = extract_namespace_from_typing_text("SomeVariable");
        assert_eq!(result.names, vec!["SomeVariable".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_with_special_chars() {
        // Test case: "func(Std::Array:" - function call with namespace
        let result = extract_namespace_from_typing_text("func(Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_nested() {
        // Test case: "A::B::C::D:" - deeply nested namespace
        let result = extract_namespace_from_typing_text("A::B::C::D:");
        assert_eq!(
            result.names,
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string()
            ]
        );
    }

    #[test]
    fn test_extract_namespace_from_typing_text_partial() {
        // Test case: "Std::arr" - partial typing with lowercase
        let result = extract_namespace_from_typing_text("Std::arr");
        assert_eq!(result.names, vec!["Std".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_with_operators() {
        // Test case: "x + Std::Array:" - with operators before
        let result = extract_namespace_from_typing_text("x + Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }

    #[test]
    fn test_extract_namespace_from_typing_text_whitespace() {
        // Test case: "    Std::Array:" - with leading whitespace
        let result = extract_namespace_from_typing_text("    Std::Array:");
        assert_eq!(result.names, vec!["Std".to_string(), "Array".to_string()]);
    }
}
