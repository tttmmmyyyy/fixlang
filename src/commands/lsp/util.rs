// Utility functions shared across LSP feature modules.

use super::server::{get_file_content_at_previous_diagnostics, LatestContent};
use crate::ast::expr::{Expr, ExprNode};
use crate::ast::name::{FullName, Name};
use crate::ast::pattern::PatternNode;
use crate::ast::program::{Program, SymbolExpr};
use crate::ast::traits::TraitId;
use crate::ast::typedecl::TypeDeclValue;
use crate::ast::types::TyCon;
use crate::commands::docs::MarkdownSection;
use crate::constants::chars_allowed_in_identifiers;
use crate::misc::{char_pos_to_utf16_pos, to_absolute_path, utf16_pos_to_utf8_byte_pos, Map};
use crate::write_log;
use crate::ast::program::EndNode;
use crate::parse::sourcefile::{SourceFile, SourcePos, Span};
use std::sync::Arc;
use difference::diff;
use lsp_types::MarkupContent;
use lsp_types::TextDocumentPositionParams;
use std::path::{Component, PathBuf};
use std::str::FromStr;

// Convert a `lsp_types::Uri` into a `PathBuf`.
pub(super) fn uri_to_path(uri: &lsp_types::Uri) -> PathBuf {
    PathBuf::from(
        urlencoding::decode(&uri.path().to_string())
            .ok()
            .unwrap()
            .as_ref(),
    )
}

// Given two versions of a file content, find the line in `content1` that corresponds to
// `line0` in `content0`.
fn calculate_corresponding_line(content0: &str, content1: &str, line0: u32) -> Option<u32> {
    let (_, diffs) = diff(content0, content1, "\n");
    let mut line_cnt_0 = -1;
    let mut line_cnt_1 = -1;
    for diff in diffs {
        match diff {
            difference::Difference::Same(s) => {
                let lines = s.split("\n").count();
                for _ in 0..lines {
                    line_cnt_0 += 1;
                    line_cnt_1 += 1;
                    if line_cnt_0 == line0 as i32 {
                        return Some(line_cnt_1 as u32);
                    }
                }
            }
            difference::Difference::Add(s) => {
                line_cnt_1 += s.split("\n").count() as i32;
            }
            difference::Difference::Rem(s) => {
                line_cnt_0 += s.split("\n").count() as i32;
            }
        }
    }
    None
}

// Convert a `lsp_types::Position` into a byte offset in a string.
fn position_to_bytes(string: &str, position: lsp_types::Position) -> usize {
    let mut bytes = 0;
    let mut line = 0;
    let mut utf16_count = 0;

    for c in string.chars() {
        if line == position.line && utf16_count >= position.character as usize {
            break;
        }
        bytes += c.len_utf8();
        if c == '\n' {
            line += 1;
            utf16_count = 0;
        } else {
            utf16_count += c.len_utf16();
        }
    }
    bytes
}

// Translate an LSP cursor position into a `SourcePos` anchored to the
// diagnostics-time snapshot of the source file (which is what the AST was
// elaborated from).
pub(super) fn resolve_source_pos(
    text_position: &TextDocumentPositionParams,
    program: &Program,
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
) -> Option<SourcePos> {
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log!("{}", msg);
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();

    let path = uri_to_path(uri);

    let saved_content = get_file_content_at_previous_diagnostics(program, &path);
    if let Err(e) = saved_content {
        write_log!("{}", e);
        return None;
    }
    let saved_content = saved_content.ok().unwrap();

    // Map the cursor line from the latest buffer back to the snapshot.
    let pos_in_latest = text_position.position;
    let line_in_saved =
        calculate_corresponding_line(&latest_content.content, &saved_content, pos_in_latest.line)?;
    let pos_in_saved = lsp_types::Position {
        line: line_in_saved,
        character: pos_in_latest.character,
    };

    Some(SourcePos {
        input: SourceFile::from_file_path(path),
        pos: position_to_bytes(&saved_content, pos_in_saved),
    })
}

// Source spans of one specific local binding: its definition site, plus
// every use that resolves to it.
pub(super) struct LocalOccurrences {
    // The binder's source span (e.g. `x` in `let x = ...`).
    pub definition: Span,
    // Every use of the binder's name within its scope that still resolves
    // to this binder. Uses captured by an inner re-binding of the same
    // name are excluded.
    pub uses: Vec<Span>,
}

// At cursor position `pos`, resolve the local name `target` to its
// enclosing binding — the innermost `let` / lambda / match-arm binder of
// `target` whose scope contains `pos` — and return that binding's
// definition span together with every use that resolves to the *same*
// binding. Uses captured by an inner re-binding of `target` are excluded.
pub(super) fn find_local_occurrences(
    program: &Program,
    pos: &SourcePos,
    target: &FullName,
) -> Option<LocalOccurrences> {
    // Locate the global value whose body contains `pos`, then search its tree.
    for (name, gv) in &program.global_values {
        if gv.find_node_at(name, pos).is_none() {
            continue;
        }
        let roots: Vec<&Arc<ExprNode>> = match &gv.expr {
            SymbolExpr::Simple(te) => vec![&te.expr],
            SymbolExpr::Method(impls) => impls.iter().map(|m| &m.expr.expr).collect(),
        };
        for root in roots {
            let mut stack: Vec<(FullName, Span)> = vec![];
            let Some(def_span) = find_enclosing_binder(root, pos, target, &mut stack) else {
                continue;
            };
            let mut uses = vec![];
            let mut stack2: Vec<(FullName, Span)> = vec![];
            collect_uses_of_binding(root, target, &def_span, &mut stack2, &mut uses);
            return Some(LocalOccurrences {
                definition: def_span,
                uses,
            });
        }
    }
    None
}

// For each variable bound by `pat`, return its name paired with the
// source span of the binder. Variables from synthetic / desugared
// patterns (no source span) are omitted — a missing span cannot be the
// target of a jump-to-definition.
fn pat_name_spans(pat: &PatternNode) -> Vec<(FullName, Span)> {
    pat.var_infos()
        .into_iter()
        .filter_map(|(n, info)| info.source.map(|s| (n, s)))
        .collect()
}

// Descend into `expr` along the path that contains `pos`, maintaining a scope
// stack of `(name, span)` bindings. When the cursor is inside a subtree whose
// scope binds `target`, return the innermost match from the stack.
fn find_enclosing_binder(
    expr: &Arc<ExprNode>,
    pos: &SourcePos,
    target: &FullName,
    stack: &mut Vec<(FullName, Span)>,
) -> Option<Span> {
    let span = expr.source.as_ref()?;
    if !span.includes_pos_lsp(pos) {
        return None;
    }

    // Look up the innermost binding for `target` that is currently in scope.
    let lookup = |stack: &Vec<(FullName, Span)>| -> Option<Span> {
        stack.iter().rev().find(|(n, _)| n == target).map(|(_, s)| s.clone())
    };

    match &*expr.expr {
        Expr::Var(_) | Expr::LLVM(_) => lookup(stack),
        Expr::App(func, args) => {
            if let Some(s) = find_enclosing_binder(func, pos, target, stack) {
                return Some(s);
            }
            for a in args {
                if let Some(s) = find_enclosing_binder(a, pos, target, stack) {
                    return Some(s);
                }
            }
            lookup(stack)
        }
        Expr::Lam(args, body) => {
            // Lambdas are desugared to `\#param. let pat = #param in body`, so
            // the user-visible binders appear inside the body's `let`. The Lam
            // itself binds only the synthesized `#param`; we still push it to
            // keep the scope stack faithful, using the body's own span as a
            // placeholder (never user-visible).
            let body_span = body.source.clone();
            let pushed = args.len();
            for v in args {
                let span = body_span.clone().unwrap_or_else(|| expr.source.clone().unwrap());
                stack.push((v.name.clone(), span));
            }
            let res = find_enclosing_binder(body, pos, target, stack);
            for _ in 0..pushed {
                stack.pop();
            }
            res.or_else(|| lookup(stack))
        }
        Expr::Let(pat, bound, val) => {
            // `let` is non-recursive: `bound` is outside the new scope.
            if let Some(s) = find_enclosing_binder(bound, pos, target, stack) {
                return Some(s);
            }
            // Cursor directly on the binder pattern: if the pattern itself binds
            // `target`, return that binder's span. Otherwise fall back to any
            // outer binding.
            if let Some(pat_span) = &pat.info.source {
                if pat_span.includes_pos_lsp(pos) {
                    for (n, info) in pat.var_infos() {
                        if &n == target {
                            if let Some(s) = info.source {
                                return Some(s);
                            }
                        }
                    }
                    return lookup(stack);
                }
            }
            let bindings = pat_name_spans(pat);
            let pushed = bindings.len();
            stack.extend(bindings);
            let res = find_enclosing_binder(val, pos, target, stack);
            for _ in 0..pushed {
                stack.pop();
            }
            res.or_else(|| lookup(stack))
        }
        Expr::If(cond, then_e, else_e) => {
            if let Some(s) = find_enclosing_binder(cond, pos, target, stack) {
                return Some(s);
            }
            if let Some(s) = find_enclosing_binder(then_e, pos, target, stack) {
                return Some(s);
            }
            if let Some(s) = find_enclosing_binder(else_e, pos, target, stack) {
                return Some(s);
            }
            lookup(stack)
        }
        Expr::Match(cond, arms) => {
            if let Some(s) = find_enclosing_binder(cond, pos, target, stack) {
                return Some(s);
            }
            for (pat, val) in arms {
                if let Some(pat_span) = &pat.info.source {
                    if pat_span.includes_pos_lsp(pos) {
                        for (n, info) in pat.var_infos() {
                            if &n == target {
                                if let Some(s) = info.source {
                                    return Some(s);
                                }
                            }
                        }
                        return lookup(stack);
                    }
                }
                let bindings = pat_name_spans(pat);
                let pushed = bindings.len();
                stack.extend(bindings);
                let res = find_enclosing_binder(val, pos, target, stack);
                for _ in 0..pushed {
                    stack.pop();
                }
                if let Some(s) = res {
                    return Some(s);
                }
            }
            lookup(stack)
        }
        Expr::TyAnno(e, _) => {
            find_enclosing_binder(e, pos, target, stack).or_else(|| lookup(stack))
        }
        Expr::MakeStruct(_, fields) => {
            for (_, _, e) in fields {
                if let Some(s) = find_enclosing_binder(e, pos, target, stack) {
                    return Some(s);
                }
            }
            lookup(stack)
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                if let Some(s) = find_enclosing_binder(e, pos, target, stack) {
                    return Some(s);
                }
            }
            lookup(stack)
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for e in args {
                if let Some(s) = find_enclosing_binder(e, pos, target, stack) {
                    return Some(s);
                }
            }
            lookup(stack)
        }
        Expr::Eval(side, main) => {
            if let Some(s) = find_enclosing_binder(side, pos, target, stack) {
                return Some(s);
            }
            find_enclosing_binder(main, pos, target, stack).or_else(|| lookup(stack))
        }
    }
}

// Walk `expr` collecting source spans of every `Expr::Var(target)` whose
// innermost enclosing binding equals `def_span`. Uses shadowed by a nested
// binder of the same name are excluded. The scope stack is threaded in the
// same way as `find_enclosing_binder`.
fn collect_uses_of_binding(
    expr: &Arc<ExprNode>,
    target: &FullName,
    def_span: &Span,
    stack: &mut Vec<(FullName, Span)>,
    out: &mut Vec<Span>,
) {
    let current_binder = |stack: &Vec<(FullName, Span)>| -> Option<Span> {
        stack.iter().rev().find(|(n, _)| n == target).map(|(_, s)| s.clone())
    };

    match &*expr.expr {
        Expr::Var(v) => {
            if &v.name == target {
                if let Some(binder) = current_binder(stack) {
                    if &binder == def_span {
                        if let Some(src) = &expr.source {
                            out.push(src.clone());
                        }
                    }
                }
            }
        }
        Expr::LLVM(_) => {}
        Expr::App(func, args) => {
            collect_uses_of_binding(func, target, def_span, stack, out);
            for a in args {
                collect_uses_of_binding(a, target, def_span, stack, out);
            }
        }
        Expr::Lam(args, body) => {
            // See `find_enclosing_binder` for why we push synthetic `#param`
            // entries using the body's span.
            let body_span = body.source.clone();
            let pushed = args.len();
            for v in args {
                let span = body_span.clone().unwrap_or_else(|| expr.source.clone().unwrap());
                stack.push((v.name.clone(), span));
            }
            collect_uses_of_binding(body, target, def_span, stack, out);
            for _ in 0..pushed {
                stack.pop();
            }
        }
        Expr::Let(pat, bound, val) => {
            // Non-recursive `let`: bound sees the outer scope.
            collect_uses_of_binding(bound, target, def_span, stack, out);
            let bindings = pat_name_spans(pat);
            let pushed = bindings.len();
            stack.extend(bindings);
            collect_uses_of_binding(val, target, def_span, stack, out);
            for _ in 0..pushed {
                stack.pop();
            }
        }
        Expr::If(cond, then_e, else_e) => {
            collect_uses_of_binding(cond, target, def_span, stack, out);
            collect_uses_of_binding(then_e, target, def_span, stack, out);
            collect_uses_of_binding(else_e, target, def_span, stack, out);
        }
        Expr::Match(cond, arms) => {
            collect_uses_of_binding(cond, target, def_span, stack, out);
            for (pat, val) in arms {
                let bindings = pat_name_spans(pat);
                let pushed = bindings.len();
                stack.extend(bindings);
                collect_uses_of_binding(val, target, def_span, stack, out);
                for _ in 0..pushed {
                    stack.pop();
                }
            }
        }
        Expr::TyAnno(e, _) => {
            collect_uses_of_binding(e, target, def_span, stack, out);
        }
        Expr::MakeStruct(_, fields) => {
            for (_, _, e) in fields {
                collect_uses_of_binding(e, target, def_span, stack, out);
            }
        }
        Expr::ArrayLit(elems) => {
            for e in elems {
                collect_uses_of_binding(e, target, def_span, stack, out);
            }
        }
        Expr::FFICall(_, _, _, _, args, _) => {
            for a in args {
                collect_uses_of_binding(a, target, def_span, stack, out);
            }
        }
        Expr::Eval(side, main) => {
            collect_uses_of_binding(side, target, def_span, stack, out);
            collect_uses_of_binding(main, target, def_span, stack, out);
        }
    }
}

// Get the current directory, logging an error and returning None if it fails.
pub(super) fn get_current_dir() -> Option<PathBuf> {
    match std::env::current_dir() {
        Ok(d) => Some(d),
        Err(e) => {
            write_log!("Failed to get the current directory: {:?}", e);
            None
        }
    }
}

// Convert a `Span` into an LSP `Range`.
pub(super) fn span_to_range(span: &Span) -> lsp_types::Range {
    fn pair_to_zero_indexed((x, y): (usize, usize)) -> (usize, usize) {
        (x - 1, y - 1)
    }

    let (start_line, start_column) = pair_to_zero_indexed(span.start_line_col());
    let (end_line, end_column) = pair_to_zero_indexed(span.end_line_col());

    // Convert character-based column positions to UTF-16 code unit positions
    let source_string = span.input.string();
    let (start_utf16_col, end_utf16_col) = if let Ok(source_string) = source_string {
        let start_utf16 =
            char_pos_to_utf16_pos(&source_string, start_line, start_column);
        let end_utf16 = char_pos_to_utf16_pos(&source_string, end_line, end_column);
        (start_utf16, end_utf16)
    } else {
        (start_column, end_column)
    };

    lsp_types::Range {
        start: lsp_types::Position {
            line: start_line as u32,
            character: start_utf16_col as u32,
        },
        end: lsp_types::Position {
            line: end_line as u32,
            character: end_utf16_col as u32,
        },
    }
}

// Convert a `Span` into an `lsp_types::Location` using `cdir` as the base directory.
// Returns `None` if the path cannot be converted to a URI.
pub(super) fn span_to_location(span: &Span, cdir: &PathBuf) -> Option<lsp_types::Location> {
    let uri = path_to_uri(&cdir.join(&span.input.file_path));
    match uri {
        Ok(uri) => Some(lsp_types::Location {
            uri,
            range: span_to_range(span),
        }),
        Err(e) => {
            write_log!("Failed to convert path to URI: {}", e);
            None
        }
    }
}

// Convert a collection of `Span`s into `lsp_types::Location`s using `cdir` as the base directory.
// Spans that cannot be converted are silently dropped (with a log message).
pub(super) fn spans_to_locations(spans: Vec<Span>, cdir: &PathBuf) -> Vec<lsp_types::Location> {
    spans
        .into_iter()
        .filter_map(|span| span_to_location(&span, cdir))
        .collect()
}

// Convert a file path into an LSP URI.
pub(super) fn path_to_uri(path: &PathBuf) -> Result<lsp_types::Uri, String> {
    // URI-encode each component of the path.
    let path = to_absolute_path(path).map_err(|_| {
        format!(
            "Failed to get the absolute path of the file: \"{}\"",
            path.to_string_lossy().to_string()
        )
    })?;
    let mut components = vec![];
    for comp in path.components() {
        match comp {
            Component::Normal(comp) => {
                let comp = comp.to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                let comp = urlencoding::encode(comp.unwrap()).to_string();
                components.push(comp);
            }
            Component::Prefix(prefix) => {
                let comp = prefix.as_os_str().to_str();
                if comp.is_none() {
                    return Err(format!("Failed to convert a path into string: {:?}", path));
                }
                components.push(comp.unwrap().to_string());
            }
            Component::RootDir => {}
            Component::CurDir => unreachable!(),
            Component::ParentDir => unreachable!(),
        }
    }
    let path = "file:///".to_string() + components.join("/").as_str();
    let uri = lsp_types::Uri::from_str(&path);
    if uri.is_err() {
        return Err(format!("Failed to convert a path into Uri: {:?}", path));
    }
    Ok(uri.unwrap())
}

// Given a `TextDocumentPositionParams`, get the line string and the byte position in that line.
// `uri_to_content` is a map to get the source string from the uri of the source file.
// The returned byte position is converted from the UTF-16 code unit position in the text_position.
pub(super) fn get_line_string_from_position(
    uri_to_content: &Map<lsp_types::Uri, LatestContent>,
    text_position: &TextDocumentPositionParams,
) -> Option<(String, usize)> {
    // Get the latest file content.
    let uri = &text_position.text_document.uri;
    if !uri_to_content.contains_key(uri) {
        let msg = format!("No stored content for the uri \"{}\".", uri.to_string());
        write_log!("{}", msg);
        return None;
    }
    let latest_content = uri_to_content.get(uri).unwrap();
    let latest_content = &latest_content.content;

    // Get the string at line `line` in `latest_content`.
    let line = text_position.position.line;
    let line = line as usize;
    let line_str = latest_content.lines().nth(line).unwrap_or("").to_string();

    // Convert UTF-16 code unit position to byte position
    let byte_pos = utf16_pos_to_utf8_byte_pos(
        &line_str,
        text_position.position.character as usize,
    );

    Some((line_str, byte_pos))
}

// Get the parameters of a global value from its documentation.
pub(super) fn parameters_of_global_value(full_name: &FullName, program: &Program) -> Option<Vec<String>> {
    // Get the document of the global value, which is a markdown string.
    let opt_gv = program.global_values.get(full_name);
    if opt_gv.is_none() {
        return None;
    }
    let gv = opt_gv.unwrap();
    let opt_docs = gv.get_document();
    if opt_docs.is_none() {
        return None;
    }
    let docs = opt_docs.unwrap();
    let sections = MarkdownSection::parse_many(docs.lines().collect());

    // Find the first top-level or second-level section named "Parameters".
    // let param_section = sections.iter().find(|sec| sec.title.trim() == "Parameters");
    let param_section = sections.iter().find_map(|sec| {
        if sec.title.trim() == "Parameters" {
            Some(sec)
        } else {
            sec.subsections
                .iter()
                .find(|subsec| subsec.title.trim() == "Parameters")
        }
    });
    if param_section.is_none() {
        return None;
    }
    let param_section = param_section.unwrap();

    // Collect the top-level list items.
    let mut params = vec![];
    for paragraph in &param_section.paragraphs {
        for line in paragraph.lines() {
            if line.starts_with("- ") || line.starts_with("* ") {
                // Find the first backquoted sequence of characters.
                let mut quoted_str = String::new();
                let mut in_backquote = false;
                for c in line.chars() {
                    if c == '`' {
                        if in_backquote {
                            in_backquote = false;
                            break;
                        } else {
                            in_backquote = true;
                            continue;
                        }
                    } else if in_backquote {
                        quoted_str.push(c);
                    }
                }
                // If the line ends in backquote, then skip it.
                if in_backquote {
                    continue;
                }

                // Find the first continuous sequence of characters that are allowed in identifiers.
                let name_chars = chars_allowed_in_identifiers();
                let mut param = String::new();
                for c in quoted_str.chars() {
                    if name_chars.contains(c) {
                        param.push(c);
                    } else if !param.is_empty() {
                        break;
                    }
                }

                // If the parameter is empty, skip it.
                if param.is_empty() {
                    continue;
                }

                params.push(param.to_string());
            }
        }
    }

    Some(params)
}

pub(super) fn find_trait_or_alias_def_src(program: &Program, trait_: TraitId) -> Option<Span> {
    let mut def_src = program
        .trait_env
        .traits
        .get(&trait_)
        .and_then(|ti| ti.source.clone());
    if def_src.is_none() {
        def_src = program
            .trait_env
            .aliases
            .data
            .get(&trait_)
            .and_then(|ta| ta.source.clone());
    }
    def_src
}

// Find the source location where the type constructor is defined.
pub(super) fn find_tycon_def_src(program: &Program, tycon: TyCon) -> Option<Span> {
    program
        .type_env
        .tycons
        .get(&tycon)
        .and_then(|ti| ti.source.clone())
}

// Find the source location of a struct field or union variant declaration
// (the bare name span in the type definition).
pub(super) fn find_field_def_src(program: &Program, tc: &TyCon, name: &Name) -> Option<Span> {
    let td = program.type_defns.iter().find(|td| td.tycon() == *tc)?;
    let fields = match &td.value {
        TypeDeclValue::Struct(s) => &s.fields,
        TypeDeclValue::Union(u) => &u.fields,
        TypeDeclValue::Alias(_) => return None,
    };
    let field = fields.iter().find(|f| &f.name == name)?;
    field.name_src.clone()
}

pub(super) fn document_from_endnode(node: &EndNode, program: &Program) -> MarkupContent {
    fn document_tycon_or_alias(program: &Program, docs: &mut String, tycon: &TyCon) {
        *docs += &format!("```\n{}\n```", tycon.to_string());
        if let Some(ti) = program.type_env.tycons.get(&tycon) {
            if let Some(document) = ti.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        } else if let Some(ta) = program.type_env.aliases.get(&tycon) {
            if let Some(document) = ta.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        }
    }

    fn document_trait_or_alias(program: &Program, docs: &mut String, trait_id: &TraitId) {
        *docs += &format!("```\n{}\n```", trait_id.to_string());
        if let Some(ti) = program.trait_env.traits.get(&trait_id) {
            if let Some(document) = ti.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        } else if let Some(ta) = program.trait_env.aliases.data.get(&trait_id) {
            if let Some(document) = ta.get_document() {
                *docs += &format!("\n\n{}", document);
            }
        }
    }

    // Create a hover message.
    let mut docs = String::new();
    match node {
        EndNode::Expr(var, ty) => {
            // Get informations of the variable which are needed to show in the hover.
            let full_name = &var.name;

            if full_name.is_local() {
                // In case the variable is local, show the name and type of the variable.
                if let Some(ty) = ty.as_ref() {
                    docs += &format!(
                        "```\n{} : {}\n```",
                        full_name.to_string(),
                        ty.to_string_normalize()
                    );
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
            } else {
                // In case the variable is global, show the documentation of the global value.
                let mut scm_string = String::new();
                if let Some(gv) = program.global_values.get(full_name) {
                    scm_string = gv
                        .syn_scm
                        .clone()
                        .unwrap_or(gv.scm.clone())
                        .to_string_normalize();
                    docs += &format!("```\n{} : {}\n```", full_name.to_string(), scm_string);
                } else {
                    docs += &format!("```\n{}\n```", full_name.to_string());
                }
                if let Some(ty) = ty.as_ref() {
                    let ty_string = ty.to_string_normalize();
                    if scm_string != ty_string {
                        docs += &format!("\nInstantiated as:\n```\n{}\n```", ty_string);
                    }
                }
                if let Some(gv) = program.global_values.get(full_name) {
                    if let Some(document) = gv.get_document() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            };
        }
        EndNode::Pattern(var, ty) => {
            // In case the node is a variable, show the name and type of the variable.
            if let Some(ty) = ty.as_ref() {
                docs += &format!(
                    "```\n{} : {}\n```",
                    var.name.to_string(),
                    ty.to_string_normalize()
                );
            } else {
                docs += &format!("```\n{}\n```", var.name.to_string());
            }
        }
        EndNode::Type(tycon) => {
            document_tycon_or_alias(program, &mut docs, tycon);
        }
        EndNode::Trait(trait_id) => {
            document_trait_or_alias(program, &mut docs, trait_id);
        }
        EndNode::Module(mod_name) => {
            docs += &format!("```\nmodule {}\n```", mod_name.to_string());
            if let Some(mi) = program.modules.iter().find(|mi| &mi.name == mod_name) {
                if let Some(document) = mi.source.get_document().ok() {
                    if !document.trim().is_empty() {
                        docs += &format!("\n\n{}", document);
                    }
                }
            }
        }
        EndNode::TypeOrTrait(name) => {
            let tycon = TyCon { name: name.clone() };
            let trait_ = TraitId::from_fullname(name.clone());
            if program.type_env.tycons.contains_key(&tycon) {
                document_tycon_or_alias(program, &mut docs, &tycon);
            } else if program.trait_env.traits.contains_key(&trait_) {
                document_trait_or_alias(program, &mut docs, &trait_);
            }
        }
        EndNode::AssocType(assoc_type) => {
            let trait_id = assoc_type.trait_id();
            docs += &format!(
                "```\nassociated type {} (trait {})\n```",
                assoc_type.name.to_string(),
                trait_id.to_string()
            );
            // Try to show the documentation comment of the associated type definition.
            if let Some(ti) = program.trait_env.traits.get(&trait_id) {
                if let Some(atd) = ti.assoc_types.get(&assoc_type.name.name) {
                    if let Some(doc) = atd.src.as_ref().and_then(|src| src.get_document().ok()) {
                        if !doc.is_empty() {
                            docs += &format!("\n\n{}", doc);
                        }
                    }
                }
            }
        }
        EndNode::ValueDecl(name) => {
            // Show the type signature and documentation of the declared global value.
            if let Some(gv) = program.global_values.get(name) {
                let scm_string = gv
                    .syn_scm
                    .clone()
                    .unwrap_or(gv.scm.clone())
                    .to_string_normalize();
                docs += &format!("```\n{} : {}\n```", name.to_string(), scm_string);
                if let Some(document) = gv.get_document() {
                    docs += &format!("\n\n{}", document);
                }
            } else {
                docs += &format!("```\n{}\n```", name.to_string());
            }
        }
        EndNode::Field(tc, name) | EndNode::Variant(tc, name) => {
            // Show the field/variant name and its type.
            let ty_str = program
                .type_defns
                .iter()
                .find(|td| td.tycon() == *tc)
                .and_then(|td| {
                    let fields = match &td.value {
                        TypeDeclValue::Struct(s) => &s.fields,
                        TypeDeclValue::Union(u) => &u.fields,
                        TypeDeclValue::Alias(_) => return None,
                    };
                    fields
                        .iter()
                        .find(|f| &f.name == name)
                        .map(|f| f.syn_ty.to_string_normalize())
                });
            match ty_str {
                Some(t) => docs += &format!("```\n{} : {}\n```", name, t),
                None => docs += &format!("```\n{}\n```", name),
            }
        }
    }
    let content = MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: docs,
    };
    content
}
