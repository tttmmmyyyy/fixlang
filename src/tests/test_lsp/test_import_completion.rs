// LSP integration tests for completion inside `import` statements.
//
// The server must recognize that the cursor sits in an import statement
// and offer, instead of expression symbols: module names at the module
// position, the imported module's namespaces / entities at the item
// positions, and the `hiding` keyword after a complete module path.

#[cfg(test)]
mod tests {
    use super::super::completion_harness::LspCompletionCtx;
    use serde_json::Value;
    use std::{fs, path::Path};

    /// All files of the `completion-import` fixture; opening them all
    /// lets any of them be queried, and diagnostics are awaited on the
    /// last one.
    const FIXTURE_FILES: &[&str] = &[
        "lib.fix",
        "extra.fix",
        "hiding.fix",
        "multiline.fix",
        "main.fix",
    ];

    /// Start a language server over a fresh copy of the
    /// `completion-import` fixture, with all of `FIXTURE_FILES` open.
    fn setup() -> LspCompletionCtx {
        LspCompletionCtx::setup("completion-import", FIXTURE_FILES)
    }

    /// The (line, column) of the position right after the first
    /// occurrence of `upto` on the first line of `file` containing
    /// `line_pat`. The fixture is ASCII, so byte columns equal UTF-16
    /// columns.
    fn position_after(project_dir: &Path, file: &str, line_pat: &str, upto: &str) -> (u32, u32) {
        let content = fs::read_to_string(project_dir.join(file))
            .unwrap_or_else(|_| panic!("failed to read fixture file {}", file));
        let (line_idx, line) = content
            .lines()
            .enumerate()
            .find(|(_, l)| l.contains(line_pat))
            .unwrap_or_else(|| panic!("no line containing {:?} in {}", line_pat, file));
        let col = line
            .find(upto)
            .unwrap_or_else(|| panic!("no {:?} on line {:?} of {}", upto, line, file))
            + upto.len();
        (line_idx as u32, col as u32)
    }

    /// The `label` strings of `items`, in order.
    fn labels(items: &[Value]) -> Vec<String> {
        items
            .iter()
            .filter_map(|it| it.get("label").and_then(|l| l.as_str()).map(String::from))
            .collect()
    }

    /// The item whose `label` is `label`; panics, listing the labels
    /// present, when there is none.
    fn find_item<'a>(items: &'a [Value], label: &str) -> &'a Value {
        items
            .iter()
            .find(|it| it.get("label").and_then(|l| l.as_str()) == Some(label))
            .unwrap_or_else(|| panic!("expected item {:?}; got labels {:?}", label, labels(items)))
    }

    /// The numeric `CompletionItemKind` of `item` (0 when absent).
    fn kind_of(item: &Value) -> i64 {
        item.get("kind").and_then(|k| k.as_i64()).unwrap_or(0)
    }

    /// At the module position (`import Li<cursor>`) the candidates are
    /// exactly the other modules of the program, offered as
    /// `Module`-kind items whose `TextEdit` replaces the typed module
    /// path (a module name can contain `.`, which clients treat as a
    /// word boundary, so plain insertion would mangle dotted names).
    #[test]
    fn test_import_completion_module_names() {
        let mut ctx = setup();
        let (line, col) =
            position_after(&ctx.project_dir, "main.fix", "import Lib::{", "import Li");
        let items = ctx.complete("main.fix", line, col);
        let labels = labels(&items);

        for expected in ["Lib", "Std", "HidingDemo", "MultiLine"] {
            assert!(
                labels.iter().any(|l| l == expected),
                "module candidate {:?} missing; got {:?}",
                expected,
                labels
            );
        }
        // The file's own module: importing it is implicit.
        assert!(
            !labels.iter().any(|l| l == "Main"),
            "the edited file's own module must not be offered; got {:?}",
            labels
        );
        // No expression symbols (their labels carry `::`).
        assert!(
            !labels.iter().any(|l| l.contains("::")),
            "expression symbols leaked into module-position completion: {:?}",
            labels
        );
        for item in &items {
            // CompletionItemKind::MODULE = 9.
            assert_eq!(kind_of(item), 9, "non-module item offered: {}", item);
        }

        let lib = find_item(&items, "Lib");
        let edit = lib
            .get("textEdit")
            .unwrap_or_else(|| panic!("module item should carry a textEdit: {}", lib));
        assert_eq!(
            edit.get("newText").and_then(|v| v.as_str()),
            Some("Lib"),
            "textEdit should insert the full module name: {}",
            edit
        );
        let range = edit.get("range").expect("textEdit has range");
        let start = range.get("start").expect("range has start");
        let end = range.get("end").expect("range has end");
        assert_eq!(
            start.get("line").and_then(|v| v.as_u64()),
            Some(line as u64)
        );
        // The typed path is `Li`, 2 characters before the cursor.
        assert_eq!(
            start.get("character").and_then(|v| v.as_u64()),
            Some((col - 2) as u64),
            "textEdit should start where the typed module path starts: {}",
            edit
        );
        assert_eq!(
            end.get("character").and_then(|v| v.as_u64()),
            Some(col as u64)
        );

        ctx.shutdown();
    }

    /// At an item position (`import Lib::{<cursor>`) the candidates are
    /// the module's root-level entities and child namespaces, as bare
    /// names.
    #[test]
    fn test_import_completion_module_members() {
        let mut ctx = setup();
        let (line, col) = position_after(
            &ctx.project_dir,
            "main.fix",
            "import Lib::{",
            "import Lib::{",
        );
        let items = ctx.complete("main.fix", line, col);
        let labels = labels(&items);

        for expected in ["lib_value", "lib_func", "LibType", "LibTrait", "Sub"] {
            assert!(
                labels.iter().any(|l| l == expected),
                "member candidate {:?} missing; got {:?}",
                expected,
                labels
            );
        }
        // Entities nested under `Sub` sit one level deeper.
        assert!(
            !labels.iter().any(|l| l == "sub_value"),
            "nested entity must not be offered at the module root; got {:?}",
            labels
        );
        // No expression symbols and no symbols of other modules.
        assert!(
            !labels.iter().any(|l| l.contains("::")),
            "expression symbols leaked into item-position completion: {:?}",
            labels
        );
        assert!(
            !labels.iter().any(|l| l == "Std" || l == "Main"),
            "module names must not be offered at an item position; got {:?}",
            labels
        );

        // FUNCTION = 3, CLASS = 7, INTERFACE = 8, MODULE = 9.
        assert_eq!(kind_of(find_item(&items, "lib_value")), 3);
        assert_eq!(kind_of(find_item(&items, "LibType")), 7);
        assert_eq!(kind_of(find_item(&items, "LibTrait")), 8);
        assert_eq!(kind_of(find_item(&items, "LibTraitAlias")), 8);
        assert_eq!(kind_of(find_item(&items, "Sub")), 9);

        // A deprecated entity carries both the legacy `deprecated`
        // field and the modern `tags` (DEPRECATED = 1); a live one
        // carries neither.
        let old_value = find_item(&items, "old_value");
        assert_eq!(
            old_value.get("deprecated").and_then(|v| v.as_bool()),
            Some(true),
            "deprecated entity should carry `deprecated: true`: {}",
            old_value
        );
        assert!(
            old_value
                .get("tags")
                .and_then(|v| v.as_array())
                .map_or(false, |tags| tags.iter().any(|t| t.as_i64() == Some(1))),
            "deprecated entity should carry the DEPRECATED tag: {}",
            old_value
        );
        let live = find_item(&items, "lib_value");
        assert!(
            live.get("deprecated").is_none() && live.get("tags").is_none(),
            "live entity must not carry deprecation fields: {}",
            live
        );

        // Values carry their type signature as the detail.
        assert_eq!(
            find_item(&items, "lib_func")
                .get("detail")
                .and_then(|v| v.as_str()),
            Some("Std::I64 -> Std::I64 -> Std::I64"),
        );

        ctx.shutdown();
    }

    /// A module name containing `.` completes as one item. The typed
    /// dotted path must be replaced whole by the item's `TextEdit`
    /// (clients treat `.` as a word boundary, so a plain insertion
    /// would splice the full name over its last component only), and
    /// the `filterText` must hold the full dotted name so the client
    /// matches the typed path against it.
    #[test]
    fn test_import_completion_dotted_module_name() {
        let mut ctx = setup();
        let (line, col) = position_after(
            &ctx.project_dir,
            "main.fix",
            "import Lib.Extra;",
            "import Lib.Ex",
        );
        let items = ctx.complete("main.fix", line, col);

        let item = find_item(&items, "Lib.Extra");
        assert_eq!(
            item.get("filterText").and_then(|v| v.as_str()),
            Some("Lib.Extra"),
            "filterText should be the full dotted module name: {}",
            item
        );
        let edit = item
            .get("textEdit")
            .unwrap_or_else(|| panic!("dotted module item should carry a textEdit: {}", item));
        assert_eq!(
            edit.get("newText").and_then(|v| v.as_str()),
            Some("Lib.Extra")
        );
        let range = edit.get("range").expect("textEdit has range");
        // The typed path is `Lib.Ex`, 6 characters before the cursor;
        // the range must span it whole, `.` included.
        assert_eq!(
            range
                .get("start")
                .and_then(|p| p.get("character"))
                .and_then(|v| v.as_u64()),
            Some((col - 6) as u64),
            "textEdit must start where the typed dotted path starts: {}",
            edit
        );
        assert_eq!(
            range
                .get("end")
                .and_then(|p| p.get("character"))
                .and_then(|v| v.as_u64()),
            Some(col as u64)
        );

        ctx.shutdown();
    }

    /// Under a trait's namespace (`import Lib::{..., LibTrait::<cursor>`)
    /// the candidates are the trait's members and associated types.
    #[test]
    fn test_import_completion_trait_namespace() {
        let mut ctx = setup();
        let (line, col) = position_after(
            &ctx.project_dir,
            "multiline.fix",
            "LibTrait::lib_method",
            "LibTrait::",
        );
        let items = ctx.complete("multiline.fix", line, col);
        let got = labels(&items);

        // The trait method is a value (FUNCTION = 3); the associated
        // type is type-level (CLASS = 7).
        assert_eq!(kind_of(find_item(&items, "lib_method")), 3);
        assert_eq!(kind_of(find_item(&items, "Elem")), 7);
        assert!(
            !got.iter().any(|l| l == "lib_value" || l == "LibTrait"),
            "candidates outside the trait's namespace offered: {:?}",
            got
        );

        ctx.shutdown();
    }

    /// Under a namespace path (`import Lib::{..., Sub::<cursor>`) the
    /// candidates are the entities of that namespace.
    #[test]
    fn test_import_completion_nested_namespace() {
        let mut ctx = setup();
        let (line, col) = position_after(&ctx.project_dir, "main.fix", "import Lib::{", "Sub::");
        let items = ctx.complete("main.fix", line, col);
        let labels = labels(&items);

        assert!(
            labels.iter().any(|l| l == "sub_value"),
            "expected `sub_value` under `Sub`; got {:?}",
            labels
        );
        assert!(
            !labels.iter().any(|l| l == "lib_value" || l == "Sub"),
            "candidates outside the `Sub` namespace offered: {:?}",
            labels
        );

        ctx.shutdown();
    }

    /// Right after a complete module path (`import Lib <cursor>`) the
    /// only word that can follow is the `hiding` keyword.
    #[test]
    fn test_import_completion_hiding_keyword() {
        let mut ctx = setup();
        let (line, col) = position_after(
            &ctx.project_dir,
            "hiding.fix",
            "import Lib hiding",
            "import Lib ",
        );
        let items = ctx.complete("hiding.fix", line, col);
        let labels = labels(&items);

        assert_eq!(
            labels,
            vec!["hiding".to_string()],
            "only the `hiding` keyword can follow a complete module path"
        );
        // CompletionItemKind::KEYWORD = 14.
        assert_eq!(kind_of(&items[0]), 14);

        ctx.shutdown();
    }

    /// After `hiding` the same member candidates as at an import item
    /// position are offered.
    #[test]
    fn test_import_completion_hiding_items() {
        let mut ctx = setup();
        let (line, col) = position_after(
            &ctx.project_dir,
            "hiding.fix",
            "import Lib hiding",
            "import Lib hiding ",
        );
        let items = ctx.complete("hiding.fix", line, col);
        let labels = labels(&items);

        for expected in ["lib_value", "Sub"] {
            assert!(
                labels.iter().any(|l| l == expected),
                "hiding candidate {:?} missing; got {:?}",
                expected,
                labels
            );
        }

        ctx.shutdown();
    }

    /// An import statement spanning several lines is still recognized:
    /// the statement, not the cursor's line, determines the context.
    #[test]
    fn test_import_completion_multiline_statement() {
        let mut ctx = setup();

        // After `lib_value,` inside the braces: back at the module root.
        let (line, col) = position_after(
            &ctx.project_dir,
            "multiline.fix",
            "    lib_value,",
            "lib_value,",
        );
        let items = ctx.complete("multiline.fix", line, col);
        let got = labels(&items);
        for expected in ["Sub", "lib_func"] {
            assert!(
                got.iter().any(|l| l == expected),
                "candidate {:?} missing in multi-line import; got {:?}",
                expected,
                got
            );
        }

        // After `Sub::` on the next line: inside the namespace.
        let (line, col) = position_after(
            &ctx.project_dir,
            "multiline.fix",
            "    Sub::sub_value",
            "Sub::",
        );
        let items = ctx.complete("multiline.fix", line, col);
        let got = labels(&items);
        assert!(
            got.iter().any(|l| l == "sub_value"),
            "expected `sub_value` under `Sub` in multi-line import; got {:?}",
            got
        );
        assert!(
            !got.iter().any(|l| l == "lib_value"),
            "candidates outside `Sub` offered in multi-line import: {:?}",
            got
        );

        ctx.shutdown();
    }

    /// Resolving import-position items adds documentation but must not
    /// append an argument snippet (the completed name is written bare in
    /// an import statement) nor auto-import edits.
    #[test]
    fn test_import_completion_resolve() {
        let mut ctx = setup();

        // A function entity: insert text stays the bare name.
        let (line, col) = position_after(
            &ctx.project_dir,
            "main.fix",
            "import Lib::{",
            "import Lib::{",
        );
        let items = ctx.complete("main.fix", line, col);
        let resolved = ctx.resolve(find_item(&items, "lib_func").clone());
        assert_eq!(
            resolved.get("insertText").and_then(|v| v.as_str()),
            Some("lib_func"),
            "no argument snippet may be appended in an import statement: {}",
            resolved
        );
        assert!(
            resolved.get("additionalTextEdits").is_none(),
            "no auto-import edits may be attached in an import statement: {}",
            resolved
        );
        let doc = resolved
            .get("documentation")
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("resolved entity should carry documentation: {}", resolved));
        assert!(
            doc.contains("Adds two numbers."),
            "documentation should come from the entity's doc comment; got {:?}",
            doc
        );

        // A namespace item has no resolve data; resolving it must
        // still succeed and return the item unchanged.
        let resolved = ctx.resolve(find_item(&items, "Sub").clone());
        assert_eq!(
            resolved.get("label").and_then(|v| v.as_str()),
            Some("Sub"),
            "resolving a namespace item should return it unchanged: {}",
            resolved
        );

        // A module item resolves to the module's documentation.
        let (line, col) =
            position_after(&ctx.project_dir, "main.fix", "import Lib::{", "import Li");
        let items = ctx.complete("main.fix", line, col);
        let resolved = ctx.resolve(find_item(&items, "Lib").clone());
        let doc = resolved
            .get("documentation")
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("resolved module should carry documentation: {}", resolved));
        assert!(
            doc.contains("module Lib"),
            "module documentation should show the module header; got {:?}",
            doc
        );

        ctx.shutdown();
    }
}
