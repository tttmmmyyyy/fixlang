// Completion inside `import` statements.
//
// The generic completion flow offers symbols as they can appear in
// expressions, which is useless while writing an import statement. The
// functions here recognize that the cursor sits inside one, classify
// which part of the statement is being typed, and produce the
// candidates that can actually be written there: module names for the
// module position, and the imported module's namespaces / entities for
// the item positions.

use super::super::util::{scan_outside_comments, ScanState};
use super::{is_internal_name, ResolveData};
use crate::ast::expr::Var;
use crate::ast::name::{FullName, Name};
use crate::ast::program::{EndNode, Program};
use crate::constants::chars_allowed_in_identifiers;
use crate::misc::{to_absolute_path, Map, Set};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemTag, CompletionTextEdit, Position, Range,
    TextDocumentPositionParams, TextEdit,
};
use std::path::Path;

/// The part of an `import` statement the cursor sits in.
#[derive(Debug, PartialEq)]
pub(super) enum ImportContext {
    /// The module position: typing the module path right after
    /// `import`. `typed` is the (possibly empty) part typed so far,
    /// e.g. `Minilib.Te`.
    ModuleName { typed: String },
    /// Right after a complete module path or a complete item part; the
    /// only word that can start here is the `hiding` keyword.
    HidingKeyword,
    /// An item position: importing (or hiding) entities of `module`,
    /// with the cursor nested under the namespace path `namespace`
    /// relative to the module. E.g. in `import Std::{IO::<cursor>}`,
    /// the module is `Std` and the namespace is `[IO]`.
    Items { module: Name, namespace: Vec<Name> },
    /// Inside the statement, but at a position where no completable
    /// token can start (e.g. after a complete `hiding` part).
    Closed,
}

/// Classify the cursor position when it sits inside an `import`
/// statement; `None` when it doesn't (including inside comments and
/// string/char literals).
pub(super) fn import_context_at(content: &str, cursor_byte: usize) -> Option<ImportContext> {
    let stmt = statement_before_cursor(content, cursor_byte)?;
    let rest = stmt.trim_start().strip_prefix("import")?;
    // Require a separator after the keyword: without one the user is
    // typing an identifier that merely starts with "import".
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    Some(classify_fragment(rest))
}

/// The code text of the statement the cursor sits in: everything after
/// the last `;` outside comments and string/char literals, up to the
/// cursor, with each comment collapsed to a single space. `None` when
/// the cursor itself is inside a comment or a literal.
fn statement_before_cursor(content: &str, cursor_byte: usize) -> Option<String> {
    let mut out: Vec<u8> = vec![];
    let state = scan_outside_comments(content, cursor_byte, &mut |byte, in_literal| {
        if byte == b';' && !in_literal {
            out.clear();
        } else {
            out.push(byte);
        }
    });
    if state != ScanState::Normal {
        return None;
    }
    // Only whole ASCII spans are removed relative to `content`, so the
    // collected bytes are valid UTF-8.
    Some(String::from_utf8(out).unwrap())
}

/// Classify the cursor position from the statement text between the
/// `import` keyword and the cursor.
///
/// The text is scanned as a token stream. The module path is the first
/// token; after it, `frames` accumulates the namespace segments
/// captured by each unclosed `{`, and `segments` the ones completed by
/// `::` since, so that at the end of the text their concatenation is
/// the namespace path enclosing the cursor. Malformed text (this runs
/// on half-typed statements) degrades to a nearby context rather than
/// to an error.
fn classify_fragment(rest: &str) -> ImportContext {
    let ident_chars = chars_allowed_in_identifiers();
    let is_ident_char = |c: char| ident_chars.contains(c);

    let mut chars = rest.chars().peekable();

    // The module path: identifier characters and the `.` separator.
    while chars.peek().map_or(false, |c| c.is_whitespace()) {
        chars.next();
    }
    let mut module = String::new();
    while let Some(&c) = chars.peek() {
        if is_ident_char(c) || c == '.' {
            module.push(c);
            chars.next();
        } else {
            break;
        }
    }
    if chars.peek().is_none() {
        // The cursor is right at the end of the module path: it is
        // still being typed.
        return ImportContext::ModuleName { typed: module };
    }

    // The item part(s) after the module path.
    let mut frames: Vec<Vec<Name>> = vec![];
    let mut segments: Vec<Name> = vec![];
    // The identifier currently being typed (not yet terminated by
    // `::`, whitespace, etc.).
    let mut pending = String::new();
    // True right after a complete element at brace depth 0 — the
    // module path, a braceless single item, or a closed `{...}` — where
    // the only word that can follow is the `hiding` keyword.
    let mut expects_hiding_keyword = true;
    let mut saw_hiding = false;

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            if !pending.is_empty() {
                if frames.is_empty() && pending == "hiding" {
                    saw_hiding = true;
                    segments.clear();
                    expects_hiding_keyword = false;
                } else {
                    expects_hiding_keyword = frames.is_empty();
                }
                pending.clear();
            }
            continue;
        }
        match c {
            ':' => {
                // `::`, or its first `:` still being typed.
                if chars.peek() == Some(&':') {
                    chars.next();
                }
                if !pending.is_empty() {
                    segments.push(std::mem::take(&mut pending));
                }
                expects_hiding_keyword = false;
            }
            '{' => {
                frames.push(std::mem::take(&mut segments));
                pending.clear();
                expects_hiding_keyword = false;
            }
            '}' => {
                // The brace's element is complete; back to its parent
                // frame, with no partial item there.
                frames.pop();
                segments.clear();
                pending.clear();
                expects_hiding_keyword = frames.is_empty();
            }
            ',' => {
                segments.clear();
                pending.clear();
                expects_hiding_keyword = false;
            }
            c if is_ident_char(c) => {
                pending.push(c);
            }
            _ => {
                // `*`, or a character no well-formed import statement
                // contains: a complete (or hopeless) element.
                pending.clear();
                expects_hiding_keyword = frames.is_empty();
            }
        }
    }

    if expects_hiding_keyword {
        // `pending` may hold a partially-typed `hiding`; the client
        // filters the offered keyword against it.
        return if saw_hiding {
            ImportContext::Closed
        } else {
            ImportContext::HidingKeyword
        };
    }
    let mut namespace: Vec<Name> = frames.into_iter().flatten().collect();
    namespace.append(&mut segments);
    ImportContext::Items { module, namespace }
}

/// Build the completion items for a cursor inside an `import`
/// statement. `file_path` is the file being edited (its own module is
/// not offered as a module candidate); `typing_text` and `position`
/// flow into each item's resolve data.
pub(super) fn import_completion_items(
    ctx: &ImportContext,
    program: &Program,
    file_path: &Path,
    typing_text: &str,
    position: &TextDocumentPositionParams,
) -> Vec<CompletionItem> {
    match ctx {
        ImportContext::ModuleName { typed } => {
            module_name_items(typed, program, file_path, typing_text, position)
        }
        ImportContext::HidingKeyword => vec![hiding_keyword_item()],
        ImportContext::Items { module, namespace } => {
            member_items(module, namespace, program, typing_text, position)
        }
        ImportContext::Closed => vec![],
    }
}

/// One completion item per module of the program, except the edited
/// file's own module (importing it is implicit).
///
/// A module name can contain `.`, which LSP clients treat as a word
/// boundary when splicing in a completion — inserting
/// `Minilib.Text.StringEx` over the typed `Minilib.Te` would only
/// replace the `Te` "word". Each item therefore carries a `TextEdit`
/// replacing the whole typed module path, and a `filter_text` holding
/// the full module name so clients match the typed path against it.
fn module_name_items(
    typed: &str,
    program: &Program,
    file_path: &Path,
    typing_text: &str,
    position: &TextDocumentPositionParams,
) -> Vec<CompletionItem> {
    let file_abs = to_absolute_path(file_path).ok();
    let self_module: Option<Name> = program
        .modules
        .iter()
        .find(|mi| {
            file_abs.is_some() && to_absolute_path(&mi.source.input.file_path).ok() == file_abs
        })
        .map(|mi| mi.name.clone());

    // The typed module path consists of identifier characters and `.`,
    // all ASCII, so its UTF-16 length equals its byte length.
    let end = position.position;
    let start = Position {
        line: end.line,
        character: end.character.saturating_sub(typed.len() as u32),
    };
    let range = Range { start, end };

    let mut names: Vec<Name> = program
        .modules
        .iter()
        .map(|mi| mi.name.clone())
        .filter(|name| Some(name) != self_module.as_ref())
        .collect();
    names.sort();
    names.dedup();

    names
        .into_iter()
        .map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::MODULE),
            filter_text: Some(name.clone()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: name.clone(),
            })),
            data: Some(resolve_data(EndNode::Module(name), typing_text, position)),
            ..CompletionItem::default()
        })
        .collect()
}

fn hiding_keyword_item() -> CompletionItem {
    CompletionItem {
        label: "hiding".to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        ..CompletionItem::default()
    }
}

/// Completion items for an item position under `module` / `namespace`:
/// the entities (values, types, traits, associated types) living
/// directly at that namespace, and the child namespaces one component
/// below it. A name that is both an entity and a namespace (e.g. a
/// type `IO` and the namespace `IO` of its methods) is offered once,
/// as the entity.
fn member_items(
    module: &Name,
    namespace: &[Name],
    program: &Program,
    typing_text: &str,
    position: &TextDocumentPositionParams,
) -> Vec<CompletionItem> {
    let mut prefix: Vec<&Name> = Vec::with_capacity(namespace.len() + 1);
    prefix.push(module);
    prefix.extend(namespace.iter());

    let mut leaves: Map<Name, CompletionItem> = Map::default();
    let mut child_namespaces: Set<Name> = Set::default();

    let add = |full_name: &FullName,
               leaves: &mut Map<Name, CompletionItem>,
               child_namespaces: &mut Set<Name>,
               make_leaf: &dyn Fn(&Name) -> CompletionItem| {
        if is_internal_name(&full_name.to_string()) {
            return;
        }
        let Some((next, is_leaf)) = next_component(full_name, &prefix) else {
            return;
        };
        if is_leaf {
            leaves.insert(next.clone(), make_leaf(next));
        } else {
            child_namespaces.insert(next.clone());
        }
    };

    for (full_name, gv) in &program.global_values {
        add(full_name, &mut leaves, &mut child_namespaces, &|name| {
            let scheme = gv
                .syn_scm
                .clone()
                .unwrap_or(gv.scm.clone())
                .to_string_normalize();
            leaf_item(
                name,
                CompletionItemKind::FUNCTION,
                Some(scheme),
                EndNode::Expr(Var::create(full_name.clone()), None),
                gv.deprecation.is_some(),
                typing_text,
                position,
            )
        });
    }
    for (tycon, _kind) in program.type_env.kinds() {
        add(&tycon.name, &mut leaves, &mut child_namespaces, &|name| {
            leaf_item(
                name,
                CompletionItemKind::CLASS,
                None,
                EndNode::Type(tycon.clone()),
                false,
                typing_text,
                position,
            )
        });
    }
    for trait_ in program.traits_with_aliases() {
        add(&trait_.name, &mut leaves, &mut child_namespaces, &|name| {
            leaf_item(
                name,
                CompletionItemKind::INTERFACE,
                None,
                EndNode::Trait(trait_.clone()),
                false,
                typing_text,
                position,
            )
        });
    }
    for (assoc_type, _kind_info) in program.trait_env.assoc_ty_kind_info() {
        add(
            &assoc_type.name,
            &mut leaves,
            &mut child_namespaces,
            &|name| {
                leaf_item(
                    name,
                    CompletionItemKind::CLASS,
                    None,
                    EndNode::AssocType(assoc_type.clone()),
                    false,
                    typing_text,
                    position,
                )
            },
        );
    }

    let mut items: Vec<CompletionItem> = vec![];
    for name in child_namespaces {
        if leaves.contains_key(&name) {
            continue;
        }
        items.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::MODULE),
            filter_text: Some(name.clone()),
            insert_text: Some(name),
            ..CompletionItem::default()
        });
    }
    items.extend(leaves.into_iter().map(|(_, item)| item));
    items
}

/// The component that follows `prefix` in `full_name`'s full path
/// (namespace components followed by the bare name), when `full_name`
/// lies strictly under `prefix`: the component, and whether it is the
/// bare name (rather than a namespace component).
fn next_component<'a>(full_name: &'a FullName, prefix: &[&Name]) -> Option<(&'a Name, bool)> {
    let ns = &full_name.namespace.names;
    if prefix.len() > ns.len() {
        return None;
    }
    if !prefix.iter().zip(ns.iter()).all(|(p, n)| *p == n) {
        return None;
    }
    if prefix.len() == ns.len() {
        Some((&full_name.name, true))
    } else {
        Some((&ns[prefix.len()], false))
    }
}

/// A completion item for an entity offered at an item position. The
/// label and inserted text are the bare name — the qualifying path is
/// already in the source around the cursor.
fn leaf_item(
    name: &Name,
    kind: CompletionItemKind,
    detail: Option<String>,
    node: EndNode,
    deprecated: bool,
    typing_text: &str,
    position: &TextDocumentPositionParams,
) -> CompletionItem {
    // Set both `deprecated` (LSP <3.15) and `tags` (LSP >=3.15) so older
    // and newer clients both render the strikethrough.
    let (deprecated_field, tags_field) = if deprecated {
        (Some(true), Some(vec![CompletionItemTag::DEPRECATED]))
    } else {
        (None, None)
    };
    CompletionItem {
        label: name.clone(),
        kind: Some(kind),
        detail,
        deprecated: deprecated_field,
        tags: tags_field,
        filter_text: Some(name.clone()),
        insert_text: Some(name.clone()),
        data: Some(resolve_data(node, typing_text, position)),
        ..CompletionItem::default()
    }
}

fn resolve_data(
    node: EndNode,
    typing_text: &str,
    position: &TextDocumentPositionParams,
) -> serde_json::Value {
    serde_json::to_value(ResolveData {
        node,
        typing_text: typing_text.to_string(),
        position: position.clone(),
        in_import: true,
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(content: &str) -> Option<ImportContext> {
        import_context_at(content, content.len())
    }

    fn items(module: &str, namespace: &[&str]) -> ImportContext {
        ImportContext::Items {
            module: module.to_string(),
            namespace: namespace.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_module_name_context() {
        assert_eq!(
            context("module Main;\n\nimport Li"),
            Some(ImportContext::ModuleName {
                typed: "Li".to_string()
            })
        );
        assert_eq!(
            context("module Main;\nimport "),
            Some(ImportContext::ModuleName {
                typed: String::new()
            })
        );
        assert_eq!(
            context("import Minilib.Te"),
            Some(ImportContext::ModuleName {
                typed: "Minilib.Te".to_string()
            })
        );
    }

    #[test]
    fn test_items_context() {
        assert_eq!(context("import Std:"), Some(items("Std", &[])));
        assert_eq!(context("import Std::"), Some(items("Std", &[])));
        assert_eq!(context("import Std::IO::"), Some(items("Std", &["IO"])));
        assert_eq!(context("import Std::{IO, "), Some(items("Std", &[])));
        assert_eq!(context("import Std::{IO, Str"), Some(items("Std", &[])));
        assert_eq!(
            context("import Std::{IO::{println, "),
            Some(items("Std", &["IO"]))
        );
        assert_eq!(
            context("import Std::{IO::{println}, "),
            Some(items("Std", &[]))
        );
        assert_eq!(
            context("import Std::{IO, String, IO::"),
            Some(items("Std", &["IO"]))
        );
    }

    #[test]
    fn test_hiding_contexts() {
        assert_eq!(context("import Std "), Some(ImportContext::HidingKeyword));
        assert_eq!(
            context("import Std::{IO} "),
            Some(ImportContext::HidingKeyword)
        );
        assert_eq!(context("import Std hi"), Some(ImportContext::HidingKeyword));
        assert_eq!(context("import Std hiding "), Some(items("Std", &[])));
        assert_eq!(context("import Std hiding {IO, "), Some(items("Std", &[])));
        assert_eq!(
            context("import Std hiding Sub::"),
            Some(items("Std", &["Sub"]))
        );
        assert_eq!(
            context("import Std hiding IO "),
            Some(ImportContext::Closed)
        );
        assert_eq!(
            context("import Std hiding {IO} "),
            Some(ImportContext::Closed)
        );
    }

    #[test]
    fn test_multiline_and_comments() {
        assert_eq!(
            context("import Std::{\n    IO,\n    "),
            Some(items("Std", &[]))
        );
        assert_eq!(
            context("import Std::{\n    IO,\n    String::"),
            Some(items("Std", &["String"]))
        );
        assert_eq!(context("import /* which */ Std::"), Some(items("Std", &[])));
        assert_eq!(
            context("import Std::{ // items\n    IO::"),
            Some(items("Std", &["IO"]))
        );
    }

    #[test]
    fn test_non_import_contexts() {
        assert_eq!(context(""), None);
        assert_eq!(context("module Main;\n"), None);
        assert_eq!(context("main : IO () = (\n    let x = f("), None);
        // A statement boundary ends the import statement.
        assert_eq!(context("import Std; "), None);
        // Still typing the keyword itself.
        assert_eq!(context("impor"), None);
        assert_eq!(context("import"), None);
        // An identifier that merely starts with "import".
        assert_eq!(context("importer.foo("), None);
        // "import" inside a string literal or spelled after one.
        assert_eq!(context("main = (\n    let s = \"import \""), None);
        assert_eq!(context("let s = \"a;b\"; let t = s"), None);
    }
}
