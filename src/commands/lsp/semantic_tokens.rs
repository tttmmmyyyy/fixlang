// LSP `textDocument/semanticTokens/full` support.
//
// Highlighting is produced in two layers:
//
//  * Base layer (always): a never-failing lexer over the live buffer
//    (`crate::parse::lexer`). It colors the things that are unambiguous from
//    syntax alone — comments, strings, numbers, keywords, capitalized
//    types/namespaces, operators. Lowercase identifiers are deliberately left
//    UNCOLORED here: whether a name is a local, a global, a function or a value
//    cannot be known lexically, and a positional guess colors a binding and its
//    use inconsistently.
//
//  * Overlay layer (when a successful elaboration is available): the AST of the
//    last successful elaboration is walked to classify each identifier
//    occurrence as a local (-> variable) or a global (-> function). The overlay
//    is computed against the snapshot the elaboration was built from, then
//    merged onto the live buffer per line via a snapshot-to-live line diff: a
//    line that is unchanged between the snapshot and the live buffer keeps its
//    overlay coloring, while a changed line falls back to the base layer. So
//    editing one line only degrades that line, instead of dropping the whole
//    file to the base layer.
//
// The net effect matches the desired UX: identifiers on an edited line are
// plain, and gain local-vs-global colors once the file type-checks.

use super::server::{send_response, DiagnosticsResult, LatestContent};
use super::util::{corresponding_line_map, uri_to_path};
use crate::ast::expr::{Expr, ExprNode};
use crate::ast::name::FullName;
use crate::ast::pattern::{Pattern, PatternNode};
use crate::ast::predicate::Predicate;
use crate::ast::program::{Program, SymbolExpr};
use crate::ast::typedecl::TypeDeclValue;
use crate::ast::types::{Scheme, TyCon, TyConVariant, Type, TypeNode};
use crate::misc::{to_absolute_path, Map, Set};
use crate::parse::lexer::{is_accessor_name, lex_tokens, LexTokenKind};
use crate::parse::sourcefile::Span;
use lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensParams,
    Uri,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Token type indices. These are the positions in the legend's `token_types`
// list; the wire format refers to a token type by this index, so the constants
// here and `legend()` below must stay in the same order.
const T_NAMESPACE: u32 = 0;
const T_TYPE: u32 = 1;
const T_VARIABLE: u32 = 2;
const T_KEYWORD: u32 = 3;
const T_NUMBER: u32 = 4;
const T_STRING: u32 = 5;
const T_COMMENT: u32 = 6;
const T_OPERATOR: u32 = 7;
const T_FUNCTION: u32 = 8;
// Boolean literals (`true` / `false`) are colored as `enumMember`: they are the
// nullary constructors of the `Bool` union, and `enumMember` is a standard
// token type that themes color distinctly from keywords.
const T_ENUM_MEMBER: u32 = 9;
const T_PROPERTY: u32 = 10;
const T_TYPE_PARAMETER: u32 = 11;
const T_STRUCT: u32 = 12;
const T_ENUM: u32 = 13;
const T_INTERFACE: u32 = 14;

/// The legend advertised in `ServerCapabilities` and used to interpret the
/// numeric token types in every response.
pub(super) fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,   // 0
            SemanticTokenType::TYPE,        // 1
            SemanticTokenType::VARIABLE,    // 2
            SemanticTokenType::KEYWORD,     // 3
            SemanticTokenType::NUMBER,      // 4
            SemanticTokenType::STRING,      // 5
            SemanticTokenType::COMMENT,     // 6
            SemanticTokenType::OPERATOR,    // 7
            SemanticTokenType::FUNCTION,    // 8
            SemanticTokenType::ENUM_MEMBER,    // 9 (boolean literals, union variants)
            SemanticTokenType::PROPERTY,       // 10 (field accessors)
            SemanticTokenType::TYPE_PARAMETER, // 11
            SemanticTokenType::STRUCT,         // 12
            SemanticTokenType::ENUM,           // 13
            SemanticTokenType::INTERFACE,      // 14 (traits)
        ],
        token_modifiers: vec![],
    }
}

/// A raw, not-yet-encoded token: a half-open byte range plus its legend index.
type RawToken = (usize, usize, u32);

/// Base-layer color for a lexical token, or `None` to leave it uncolored.
/// Lowercase identifiers (`Variable`) are left to the AST overlay.
fn base_token_type(kind: LexTokenKind) -> Option<u32> {
    match kind {
        LexTokenKind::Comment => Some(T_COMMENT),
        LexTokenKind::String => Some(T_STRING),
        LexTokenKind::Number => Some(T_NUMBER),
        LexTokenKind::Keyword => Some(T_KEYWORD),
        LexTokenKind::Boolean => Some(T_ENUM_MEMBER),
        LexTokenKind::Type => Some(T_TYPE),
        LexTokenKind::Namespace => Some(T_NAMESPACE),
        LexTokenKind::Operator => Some(T_OPERATOR),
        LexTokenKind::Property => Some(T_PROPERTY),
        LexTokenKind::Variable => None,
    }
}

/// Lex `content` and keep only the tokens the base layer colors, as raw
/// byte-range tokens.
fn base_raw_tokens(content: &str) -> Vec<RawToken> {
    lex_tokens(content)
        .into_iter()
        .filter_map(|t| base_token_type(t.kind).map(|ty| (t.start, t.end, ty)))
        .collect()
}

/// Compute the delta-encoded base-layer tokens for `content`, without the AST
/// overlay.
#[cfg(test)]
pub(crate) fn compute_semantic_tokens(content: &str) -> Vec<SemanticToken> {
    pieces_to_tokens(raw_to_pieces(content, base_raw_tokens(content)))
}

/// A positioned token: 0-based line, UTF-16 start column, UTF-16 length, and the
/// legend token-type index.
type Piece = (u32, u32, u32, u32);

/// Turn raw byte-range tokens into positioned pieces. Multi-line tokens (block
/// comments) are split into one piece per line, since a semantic token may not
/// span a line break. Positions and lengths are in UTF-16 code units.
fn raw_to_pieces(content: &str, raw: Vec<RawToken>) -> Vec<Piece> {
    let line_starts = line_start_offsets(content);
    let mut pieces: Vec<Piece> = vec![];
    for (start, end, token_type) in raw {
        if start >= end || end > content.len() {
            continue;
        }
        let text = &content[start..end];
        let mut seg_start = start;
        for line_piece in text.split_inclusive('\n') {
            let piece = line_piece.strip_suffix('\n').unwrap_or(line_piece);
            let piece = piece.strip_suffix('\r').unwrap_or(piece);
            let length: u32 = piece.chars().map(|c| c.len_utf16() as u32).sum();
            if length > 0 {
                let (line, col) = byte_to_line_col_utf16(content, &line_starts, seg_start);
                pieces.push((line, col, length, token_type));
            }
            seg_start += line_piece.len();
        }
    }
    pieces
}

/// Sort pieces by position and delta-encode them into the LSP wire format.
fn pieces_to_tokens(mut pieces: Vec<Piece>) -> Vec<SemanticToken> {
    pieces.sort_by_key(|&(line, col, _, _)| (line, col));

    let mut data = Vec::with_capacity(pieces.len());
    let mut prev_line = 0u32;
    let mut prev_col = 0u32;
    for (line, col, length, token_type) in pieces {
        let delta_line = line - prev_line;
        let delta_start = if delta_line == 0 { col - prev_col } else { col };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        });
        prev_line = line;
        prev_col = col;
    }
    data
}

/// Merge the AST overlay onto the live-buffer base pieces.
///
/// Overlay pieces are positioned against the snapshot; `line_map` maps each
/// snapshot line to the corresponding live line (or `None` if it changed). An
/// overlay piece is remapped onto the live buffer only when its snapshot line is
/// unchanged; on a changed line the base layer stands in. At a shared position
/// the overlay wins (e.g. a struct name colored `struct` rather than the base
/// `type`). This keeps the rich coloring on every untouched line while the user
/// edits, instead of dropping the whole file to the base layer.
fn merge_overlay(base: Vec<Piece>, overlay_snapshot: Vec<Piece>, line_map: &[Option<u32>]) -> Vec<Piece> {
    let mut by_pos: Map<(u32, u32), (u32, u32)> = Map::default();
    for (line, col, len, ty) in base {
        by_pos.insert((line, col), (len, ty));
    }
    for (snap_line, col, len, ty) in overlay_snapshot {
        if let Some(Some(live_line)) = line_map.get(snap_line as usize) {
            by_pos.insert((*live_line, col), (len, ty));
        }
    }
    by_pos
        .into_iter()
        .map(|((line, col), (len, ty))| (line, col, len, ty))
        .collect()
}


/// The byte offset at which each line of `content` begins (index `i` is line
/// `i`, 0-based).
fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in content.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Convert a byte offset into a 0-based (line, UTF-16 column) position.
///
/// # Arguments
/// * `line_starts` — the per-line byte offsets from `line_start_offsets`,
///   reused so each conversion avoids a fresh scan of `content`.
fn byte_to_line_col_utf16(content: &str, line_starts: &[usize], byte: usize) -> (u32, u32) {
    let line = match line_starts.binary_search(&byte) {
        Ok(i) => i,
        Err(i) => i - 1,
    };
    let line_start = line_starts[line];
    let col: usize = content[line_start..byte]
        .chars()
        .map(|c| c.len_utf16())
        .sum();
    (line as u32, col as u32)
}

// --- AST overlay --------------------------------------------------------

/// Collect the richer, type-checked tokens for the file at `abs_file` by walking
/// the elaborated program. Only spans belonging to `abs_file` are emitted, with
/// byte offsets that index into `content` (the snapshot the program was
/// elaborated from).
fn collect_overlay(program: &Program, abs_file: &Path, content: &str, out: &mut Vec<RawToken>) {
    out.extend(Overlay::new(program, content, abs_file).run());
}

/// Walks the program and accumulates overlay tokens, filtering to one file.
struct Overlay<'a> {
    /// The elaborated program being walked.
    program: &'a Program,
    /// The source text the spans index into (the elaboration snapshot).
    content: &'a str,
    /// The file whose tokens are collected; spans from other files are skipped.
    abs_file: &'a Path,
    /// Full names of every union variant constructor in the program, so a
    /// reference to one can be colored as an enum member rather than a function.
    variant_ctors: Set<FullName>,
    /// Memoizes `file_path -> belongs to abs_file?` so the (filesystem-touching)
    /// absolute-path resolution runs once per distinct file, not once per symbol.
    file_cache: RefCell<Map<PathBuf, bool>>,
    /// The accumulated overlay tokens.
    out: Vec<RawToken>,
}

impl<'a> Overlay<'a> {
    /// Build an overlay walker, precomputing the set of union variant
    /// constructors so references to them can be colored as enum members.
    fn new(program: &'a Program, content: &'a str, abs_file: &'a Path) -> Self {
        let mut variant_ctors = Set::default();
        for td in &program.type_defns {
            if let TypeDeclValue::Union(u) = &td.value {
                let ns = td.name.to_namespace();
                for f in &u.fields {
                    variant_ctors.insert(FullName::new(&ns, &f.name));
                }
            }
        }
        Overlay {
            program,
            content,
            abs_file,
            variant_ctors,
            file_cache: RefCell::new(Map::default()),
            out: vec![],
        }
    }

    /// Walk the whole program and return the collected overlay tokens.
    fn run(mut self) -> Vec<RawToken> {
        self.collect_global_values();
        self.collect_type_defns();
        self.collect_traits();
        self.out
    }

    /// Collect tokens for global values: declaration / definition names,
    /// signatures, and bodies. Compiler-generated accessors (getters etc.) are
    /// skipped here; their *uses* are colored as properties by the base layer.
    fn collect_global_values(&mut self) {
        for (_name, gv) in &self.program.global_values {
            if gv.compiler_defined_method {
                continue;
            }
            if let Some(decl) = gv.decl_src.clone() {
                if self.in_file(&decl) {
                    self.push_value(&decl, T_FUNCTION);
                    let scm = gv.syn_scm.as_ref().unwrap_or(&gv.scm);
                    self.collect_scheme(scm);
                }
            }
            if let Some(defn) = gv.defn_src.clone() {
                if Some(&defn) != gv.decl_src.as_ref() && self.in_file(&defn) {
                    self.push_value(&defn, T_FUNCTION);
                }
            }
            let roots: Vec<Arc<ExprNode>> = match &gv.expr {
                SymbolExpr::Simple(te) => vec![te.expr.clone()],
                SymbolExpr::Method(impls) => impls.iter().map(|m| m.expr.expr.clone()).collect(),
            };
            for root in &roots {
                if let Some(src) = &root.source {
                    if self.in_file(src) {
                        self.collect_expr(root);
                    }
                }
            }
        }
    }

    /// Collect tokens for type definitions: the type name (by kind), variant
    /// names, and field types.
    fn collect_type_defns(&mut self) {
        for td in &self.program.type_defns {
            let Some(name_src) = td.name_src.clone() else {
                continue;
            };
            if !self.in_file(&name_src) {
                continue;
            }
            let name_cat = match &td.value {
                TypeDeclValue::Struct(_) => T_STRUCT,
                TypeDeclValue::Union(_) => T_ENUM,
                TypeDeclValue::Alias(_) => T_TYPE,
            };
            self.push_type(&name_src, name_cat);
            match &td.value {
                TypeDeclValue::Struct(s) => {
                    for f in &s.fields {
                        if let Some(fsrc) = &f.name_src {
                            self.push_value(fsrc, T_PROPERTY);
                        }
                        self.collect_type(&f.syn_ty);
                    }
                }
                TypeDeclValue::Union(u) => {
                    for f in &u.fields {
                        if let Some(fsrc) = &f.name_src {
                            self.push_value(fsrc, T_ENUM_MEMBER);
                        }
                        self.collect_type(&f.syn_ty);
                    }
                }
                TypeDeclValue::Alias(a) => self.collect_type(&a.value),
            }
        }
    }

    /// Collect tokens for trait definitions: the trait name. Member signatures
    /// are reached via their own global values (trait methods), so they are not
    /// walked here.
    fn collect_traits(&mut self) {
        for (_tid, tdef) in self.program.trait_env.traits.iter() {
            if let Some(name_src) = &tdef.name_src {
                if self.in_file(name_src) {
                    self.push_type(name_src, T_INTERFACE);
                }
            }
        }
    }

    /// Whether `span` belongs to the file being collected. Results are cached
    /// per source file path.
    fn in_file(&self, span: &Span) -> bool {
        let key = &span.input.file_path;
        if let Some(b) = self.file_cache.borrow().get(key) {
            return *b;
        }
        let b = to_absolute_path(key)
            .map(|p| p.as_path() == self.abs_file)
            .unwrap_or(false);
        self.file_cache.borrow_mut().insert(key.clone(), b);
        b
    }

    /// The token type for a type constructor: `struct` / `enum` by its variant,
    /// otherwise the generic `type`.
    fn classify_tycon(&self, tc: &TyCon) -> u32 {
        match self.program.type_env.tycons.get(tc) {
            Some(info) => match info.variant {
                TyConVariant::Struct => T_STRUCT,
                TyConVariant::Union => T_ENUM,
                _ => T_TYPE,
            },
            None => T_TYPE,
        }
    }

    /// The token type for a global name: `enumMember` for a union variant
    /// constructor, otherwise `function`.
    fn classify_global(&self, name: &FullName) -> u32 {
        if self.variant_ctors.contains(name) {
            T_ENUM_MEMBER
        } else {
            T_FUNCTION
        }
    }

    /// Collect tokens for a type scheme: its type and its predicates.
    fn collect_scheme(&mut self, scm: &Scheme) {
        self.collect_type(&scm.ty);
        for p in &scm.predicates {
            self.collect_pred(p);
        }
    }

    /// Collect tokens for a predicate: the trait name and the constrained type.
    fn collect_pred(&mut self, p: &Predicate) {
        if let Some(s) = &p.trait_src {
            self.push_type(s, T_INTERFACE);
        }
        self.collect_type(&p.ty);
    }

    /// Walk a type tree, emitting a token per source-backed node: type
    /// constructors as type/struct/enum, type variables as typeParameter.
    fn collect_type(&mut self, ty: &Arc<TypeNode>) {
        match &ty.ty {
            Type::TyVar(_) => {
                if let Some(s) = ty.get_source() {
                    self.push_value(s, T_TYPE_PARAMETER);
                }
            }
            Type::TyCon(tc) => {
                if let Some(s) = ty.get_source() {
                    let cat = self.classify_tycon(tc);
                    self.push_type(s, cat);
                }
            }
            Type::TyApp(f, a) => {
                self.collect_type(f);
                self.collect_type(a);
            }
            Type::AssocTy(_, args) => {
                for a in args {
                    self.collect_type(a);
                }
            }
        }
    }

    /// Walk an expression tree, emitting a token per identifier occurrence and
    /// recursing into subexpressions.
    fn collect_expr(&mut self, expr: &Arc<ExprNode>) {
        if let Expr::Var(v) = &*expr.expr {
            if let Some(span) = &expr.source {
                let cat = if v.name.is_local() {
                    T_VARIABLE
                } else {
                    self.classify_global(&v.name)
                };
                self.push_value(span, cat);
            }
        }
        match &*expr.expr {
            Expr::Var(_) | Expr::LLVM(_) => {}
            Expr::App(func, args) => {
                self.collect_expr(func);
                for a in args {
                    self.collect_expr(a);
                }
            }
            // Lambdas desugar to `\#param. let pat = #param in body`; the
            // user-written parameter binders therefore appear as a `let`
            // pattern inside the body, so recursing into the body is enough.
            Expr::Lam(_args, body) => self.collect_expr(body),
            Expr::Let(pat, bound, val) => {
                self.collect_pattern(pat);
                self.collect_expr(bound);
                self.collect_expr(val);
            }
            Expr::If(cond, then_e, else_e) => {
                self.collect_expr(cond);
                self.collect_expr(then_e);
                self.collect_expr(else_e);
            }
            Expr::Match(cond, arms) => {
                self.collect_expr(cond);
                for (pat, val) in arms {
                    self.collect_pattern(pat);
                    self.collect_expr(val);
                }
            }
            Expr::TyAnno(e, ty) => {
                self.collect_expr(e);
                self.collect_type(ty);
            }
            Expr::MakeStruct(tc, fields) => {
                // The struct type-constructor name is in `aux_src`.
                if let Some(aux) = &expr.aux_src {
                    let cat = self.classify_tycon(tc);
                    self.push_type(aux, cat);
                }
                for (_name, name_span, e) in fields {
                    if let Some(s) = name_span {
                        self.push_value(s, T_PROPERTY);
                    }
                    self.collect_expr(e);
                }
            }
            Expr::ArrayLit(elems) => {
                for e in elems {
                    self.collect_expr(e);
                }
            }
            Expr::FFICall(_, _, _, _, args, _) => {
                for e in args {
                    self.collect_expr(e);
                }
            }
            Expr::Eval(side, main) => {
                self.collect_expr(side);
                self.collect_expr(main);
            }
        }
    }

    /// Walk a pattern tree, emitting a token per binder, struct field and union
    /// variant name.
    fn collect_pattern(&mut self, pat: &PatternNode) {
        match &pat.pattern {
            Pattern::Var(_v, ty) => {
                // The pattern span covers `name : Type`; color only the name.
                if let Some(span) = &pat.info.source {
                    let (s, e) = self.leading_name_range(span.start, span.end);
                    self.push_value_range(s, e, T_VARIABLE);
                }
                if let Some(ty) = ty {
                    self.collect_type(ty);
                }
            }
            Pattern::Struct(tc, fields) => {
                // The struct type-constructor name (`info.aux_src`).
                if let Some(aux) = &pat.info.aux_src {
                    let cat = self.classify_tycon(tc);
                    self.push_type(aux, cat);
                }
                for (_name, name_span, sub) in fields {
                    if let Some(s) = name_span {
                        self.push_value(s, T_PROPERTY);
                    }
                    self.collect_pattern(sub);
                }
            }
            Pattern::Union(_full, name_span, sub) => {
                if let Some(s) = name_span {
                    self.push_value(s, T_ENUM_MEMBER);
                }
                self.collect_pattern(sub);
            }
        }
    }

    /// Emit a token for a capitalized name (type / trait / namespace). The
    /// uppercase-head check skips synthetic constructors whose span covers
    /// punctuation (e.g. the tuple/arrow type cons).
    fn push_type(&mut self, span: &Span, cat: u32) {
        if let Some(text) = self.span_text(span.start, span.end) {
            if matches!(text.chars().next(), Some('A'..='Z')) {
                self.out.push((span.start, span.end, cat));
            }
        }
    }

    /// Emit a token for a value-identifier-shaped name (lowercase / `_` / `@`).
    /// Field accessors are skipped: the base layer already colors them.
    fn push_value(&mut self, span: &Span, cat: u32) {
        self.push_value_range(span.start, span.end, cat);
    }

    /// Emit a value-identifier token for the byte range `[start, end)`, applying
    /// the same shape and field-accessor filtering as `push_value`.
    fn push_value_range(&mut self, start: usize, end: usize, cat: u32) {
        if let Some(text) = self.span_text(start, end) {
            if !matches!(text.chars().next(), Some('a'..='z') | Some('_') | Some('@')) {
                return;
            }
            if is_accessor_name(text) {
                return;
            }
            self.out.push((start, end, cat));
        }
    }

    /// The byte range of the leading identifier within `[start, end)` — used to
    /// color only the name of a binder whose span also covers a `: Type`
    /// annotation. Returns the original range if it does not start with an
    /// identifier character.
    fn leading_name_range(&self, start: usize, end: usize) -> (usize, usize) {
        let Some(text) = self.span_text(start, end) else {
            return (start, end);
        };
        let mut name_end = start;
        for (i, c) in text.char_indices() {
            let is_name = c.is_ascii_alphanumeric() || c == '_' || c == '@';
            if i == 0 && !is_name {
                return (start, end);
            }
            if !is_name {
                return (start, name_end);
            }
            name_end = start + i + c.len_utf8();
        }
        (start, end)
    }

    /// The source text of `[start, end)`, or `None` if it is empty or out of
    /// bounds for the buffer.
    fn span_text(&self, start: usize, end: usize) -> Option<&'a str> {
        if start >= end
            || end > self.content.len()
            || !self.content.is_char_boundary(start)
            || !self.content.is_char_boundary(end)
        {
            return None;
        }
        Some(&self.content[start..end])
    }
}

/// Handle a `textDocument/semanticTokens/full` request.
///
/// Always returns the base lexical layer (so highlighting works on a broken
/// buffer). When a successful elaboration is available, its AST overlay is
/// merged in for every line that is unchanged between the elaborated snapshot
/// and the live buffer — so editing one line only degrades that line to the
/// base layer, instead of dropping the whole file. Replies with an empty token
/// set rather than an error when the buffer is unknown, so the client never
/// hangs.
pub(super) fn handle_semantic_tokens_full(
    id: u32,
    params: &SemanticTokensParams,
    uri_to_content: &Map<Uri, LatestContent>,
    last_diag: Option<&DiagnosticsResult>,
) {
    let uri = &params.text_document.uri;
    let Some(latest) = uri_to_content.get(uri) else {
        send_response(
            id,
            Ok::<_, ()>(SemanticTokens {
                result_id: None,
                data: vec![],
            }),
        );
        return;
    };
    let live = &latest.content;

    let base = raw_to_pieces(live, base_raw_tokens(live));

    // Look up the elaborated snapshot for this file (if any) and overlay it.
    let overlay = last_diag.and_then(|diag| {
        let abs = to_absolute_path(&uri_to_path(uri)).ok()?;
        let snapshot = diag.user_source_contents.get(&abs)?;
        let mut raw = vec![];
        collect_overlay(&diag.program, &abs, snapshot, &mut raw);
        let snapshot_pieces = raw_to_pieces(snapshot, raw);
        let line_map = corresponding_line_map(snapshot, live);
        Some((snapshot_pieces, line_map))
    });

    let pieces = match overlay {
        Some((snapshot_pieces, line_map)) => merge_overlay(base, snapshot_pieces, &line_map),
        None => base,
    };

    send_response(
        id,
        Ok::<_, ()>(SemanticTokens {
            result_id: None,
            data: pieces_to_tokens(pieces),
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `SemanticToken` from its delta-encoded fields, with no modifiers.
    fn tok(delta_line: u32, delta_start: u32, length: u32, token_type: u32) -> SemanticToken {
        SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        }
    }

    /// Verifies that each legend entry sits at the index its `T_*` constant
    /// names, since the wire format refers to token types by index.
    #[test]
    fn legend_has_expected_order() {
        let legend = legend();
        assert_eq!(legend.token_types[T_NAMESPACE as usize], SemanticTokenType::NAMESPACE);
        assert_eq!(legend.token_types[T_TYPE as usize], SemanticTokenType::TYPE);
        assert_eq!(legend.token_types[T_VARIABLE as usize], SemanticTokenType::VARIABLE);
        assert_eq!(legend.token_types[T_KEYWORD as usize], SemanticTokenType::KEYWORD);
        assert_eq!(legend.token_types[T_NUMBER as usize], SemanticTokenType::NUMBER);
        assert_eq!(legend.token_types[T_STRING as usize], SemanticTokenType::STRING);
        assert_eq!(legend.token_types[T_COMMENT as usize], SemanticTokenType::COMMENT);
        assert_eq!(legend.token_types[T_OPERATOR as usize], SemanticTokenType::OPERATOR);
        assert_eq!(legend.token_types[T_FUNCTION as usize], SemanticTokenType::FUNCTION);
        assert_eq!(
            legend.token_types[T_ENUM_MEMBER as usize],
            SemanticTokenType::ENUM_MEMBER
        );
        assert_eq!(
            legend.token_types[T_PROPERTY as usize],
            SemanticTokenType::PROPERTY
        );
        assert_eq!(
            legend.token_types[T_TYPE_PARAMETER as usize],
            SemanticTokenType::TYPE_PARAMETER
        );
        assert_eq!(legend.token_types[T_STRUCT as usize], SemanticTokenType::STRUCT);
        assert_eq!(legend.token_types[T_ENUM as usize], SemanticTokenType::ENUM);
        assert_eq!(
            legend.token_types[T_INTERFACE as usize],
            SemanticTokenType::INTERFACE
        );
    }

    /// Verifies that field accessors (`@x`, `set_x`) are colored as properties
    /// by the base layer, without needing type information.
    #[test]
    fn field_accessors_color_as_property_in_base_layer() {
        // Accessor functions and index `^field` are colored without needing
        // type information (base layer).
        assert_eq!(
            compute_semantic_tokens("@x"),
            vec![tok(0, 0, 2, T_PROPERTY)]
        );
        assert_eq!(
            compute_semantic_tokens("set_x"),
            vec![tok(0, 0, 5, T_PROPERTY)]
        );
    }

    /// Verifies that `true` / `false` color as enumMember while `nullptr` stays
    /// a keyword.
    #[test]
    fn boolean_literals_get_enum_member_type() {
        // `true` / `false` color as enumMember; `nullptr` stays a keyword.
        assert_eq!(
            compute_semantic_tokens("true false nullptr"),
            vec![
                tok(0, 0, 4, T_ENUM_MEMBER), // true
                tok(0, 5, 5, T_ENUM_MEMBER), // false
                tok(0, 6, 7, T_KEYWORD),     // nullptr
            ]
        );
    }

    /// Verifies that lowercase identifiers get no base-layer token (they are
    /// left to the AST overlay), while a keyword next to one is still colored.
    #[test]
    fn identifiers_are_uncolored_in_base_layer() {
        // Lowercase identifiers get no base token (the AST overlay colors them).
        assert_eq!(compute_semantic_tokens("foo bar"), vec![]);
        // `let x` only colors the keyword; `x` is left to the overlay.
        assert_eq!(compute_semantic_tokens("let x"), vec![tok(0, 0, 3, T_KEYWORD)]);
    }

    /// Verifies that in `Std::Array` the qualifier is a namespace, `::` an
    /// operator, and the final segment a type — all from the base layer.
    #[test]
    fn namespace_qualifier_colored() {
        // `Std::Array`: namespace, operator, type (all base-layer).
        assert_eq!(
            compute_semantic_tokens("Std::Array"),
            vec![
                tok(0, 0, 3, T_NAMESPACE), // Std at col 0
                tok(0, 3, 2, T_OPERATOR),  // ::  at col 3
                tok(0, 2, 5, T_TYPE),      // Array at col 5 (delta 2 from col 3)
            ]
        );
    }

    /// Verifies that `delta_start` is encoded absolutely (not relative to the
    /// previous token) on the first token of a new line.
    #[test]
    fn newline_resets_delta_start() {
        // Two capitalized (type) tokens across lines: delta_start is absolute
        // again on the new line.
        assert_eq!(
            compute_semantic_tokens("Foo\n  Bar"),
            vec![
                tok(0, 0, 3, T_TYPE), // Foo on line 0, col 0
                tok(1, 2, 3, T_TYPE), // Bar on line 1, col 2
            ]
        );
    }

    /// Verifies that a multi-line block comment is split into one token per
    /// line, since a semantic token may not span a line break.
    #[test]
    fn block_comment_splits_per_line() {
        assert_eq!(
            compute_semantic_tokens("/*a\nbb*/"),
            vec![
                tok(0, 0, 3, T_COMMENT), // "/*a" on line 0
                tok(1, 0, 4, T_COMMENT), // "bb*/" on line 1
            ]
        );
    }

    /// Verifies that token columns and lengths are measured in UTF-16 code
    /// units, so positions stay correct after a multi-unit character.
    #[test]
    fn utf16_columns_and_lengths() {
        // The emoji is 2 UTF-16 units; the string `"😀"` is length 4, and the
        // following capitalized `Foo` sits at UTF-16 column 5.
        assert_eq!(
            compute_semantic_tokens("\"😀\" Foo"),
            vec![
                tok(0, 0, 4, T_STRING), // "😀" length 4 in UTF-16
                tok(0, 5, 3, T_TYPE),   // Foo at col 5
            ]
        );
    }

    /// Verifies that empty input yields no tokens (rather than panicking).
    #[test]
    fn empty_content_yields_no_tokens() {
        assert_eq!(compute_semantic_tokens(""), vec![]);
    }

    /// Verifies that even on broken input the emitted tokens decode to
    /// non-decreasing positions with positive lengths (a valid wire stream).
    #[test]
    fn broken_input_produces_monotonic_deltas() {
        let src = "module Main; main = (\n  let = => Foo::\n  \"unterminated";
        let data = compute_semantic_tokens(src);
        let mut line = 0u32;
        let mut col = 0u32;
        let mut prev = (0u32, 0u32);
        for (i, t) in data.iter().enumerate() {
            if t.delta_line == 0 {
                col += t.delta_start;
            } else {
                line += t.delta_line;
                col = t.delta_start;
            }
            if i > 0 {
                assert!((line, col) >= prev, "tokens must be non-decreasing");
            }
            prev = (line, col);
            assert!(t.length > 0, "zero-length token emitted");
        }
    }
}
