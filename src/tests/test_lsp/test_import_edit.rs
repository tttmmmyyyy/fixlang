// Unit tests for the edits the language server makes to a file's import statements when a
// completion imports the completed name.

#[cfg(test)]
mod tests {
    use super::super::text_edit::apply_lsp_edits;
    use crate::ast::import::{is_accessible, ImportStatement};
    use crate::ast::name::FullName;
    use crate::commands::lsp::edit_import::create_text_edit_to_import;
    use crate::commands::lsp::server::LatestContent;
    use crate::constants::STD_NAME;
    use crate::error::panic_if_err;
    use crate::parse::parser::parse_str_import_statements;
    use std::path::PathBuf;

    /// The import statements `source` makes effective, the implicit import of `Std` included where
    /// the source writes none.
    fn effective_imports(source: &str) -> Vec<ImportStatement> {
        let mut stmts = panic_if_err(parse_str_import_statements(
            PathBuf::from("main.fix"),
            source,
        ));
        if !stmts.iter().any(|stmt| stmt.module_name == STD_NAME) {
            stmts.push(ImportStatement::implicit_std_import("Main".to_string()));
        }
        stmts
    }

    /// Verifies that the edits to import a name, applied to the source, make the name accessible and
    /// leave every other name as accessible as it was, for import statements of each shape: nested
    /// namespaces, `hiding`, several statements of one module, dotted module names, empty lists,
    /// lists long enough to wrap, statements sharing a line, statements after the code, a source
    /// ending in a statement, CRLF line breaks and characters outside the BMP.
    #[test]
    fn test_import_edits_add_the_name_and_keep_every_other_name() {
        let names = [
            "Lib::f",
            "Lib::g",
            "Lib::h",
            "Lib::T",
            "Lib::T::f",
            "Lib::T::g",
            "Lib::Ns::f",
            "Lib::Ns::g",
            "Lib::Ns::Sub::f",
            "Lib::Ns::Sub::g",
            "Lib::Ns::T",
            "Std::IO",
            "Std::IO::println",
            "Std::String",
            "Std::Monad::pure",
            "Lib.Sub::f",
            "Lib.Sub::g",
        ];
        let cases = [
            ("module Main;\nimport Lib::{Ns::f};\n\nx = 1;\n", "Lib::Ns::g"),
            ("module Main;\nimport Lib::{T};\n\nx = 1;\n", "Lib::T::f"),
            ("module Main;\nimport Lib::{T::f};\n\nx = 1;\n", "Lib::T"),
            ("module Main;\nimport Lib hiding f;\n\nx = 1;\n", "Lib::f"),
            (
                "module Main;\nimport Lib hiding f;\nimport Std::IO;\nx = 1;\n",
                "Std::String",
            ),
            ("module Main;\nimport Lib::{Ns::*} hiding Ns::f;\n\nx = 1;\n", "Lib::Ns::f"),
            ("module Main;\nimport Lib::f;\nimport Lib::g;\n\nx = 1;\n", "Lib::h"),
            ("module Main;\nimport Lib.Sub::f;\n\nx = 1;\n", "Lib.Sub::g"),
            ("module Main;\nimport Lib::Ns::{};\n\nx = 1;\n", "Lib::Ns::g"),
            ("module Main;\nimport Lib::{Ns::{Sub::f}, Ns::T};\n\nx = 1;\n", "Lib::Ns::Sub::g"),
            ("module Main;\nimport Lib::{Ns::Sub::f, Ns::Sub::g};\n\nx = 1;\n", "Lib::Ns::f"),
            ("module Main;import Lib::f;\nx = 1;\n", "Lib::g"),
            ("module Main;\r\nimport Lib::f;\r\n\r\nx = 1;\r\n", "Lib::g"),
            ("module Main;\nimport Lib::f;import Std::IO;\nx = 1;\n", "Std::String"),
            ("module Main;\n/* 日本語 */ import Lib::f;\nx = 1;\n", "Lib::g"),
            ("module Main;\nx = 1;\nimport Lib::f;\n", "Lib::g"),
            ("module Main;\nimport Lib::f;", "Lib::g"),
            ("module Main;\nimport Std::IO;\nimport Lib;\nx = 1;\n", "Std::Monad::pure"),
            ("module Main;\nimport Lib::{f, g} hiding T::*;\nx = 1;\n", "Lib::T::f"),
            (
                "// 😀\r\nmodule Main;\r\n\r\nimport /* 😀 */ Lib::f; // 😀\r\nimport Std::IO;\r\n\r\nx = 1; // 😀\r\n",
                "Lib::g",
            ),
            (
                "module Main;\nimport Lib::{aaaaaaaaaaaaaaaaaaaa1, aaaaaaaaaaaaaaaaaaaa2, aaaaaaaaaaaaaaaaaaaa3, aaaaaaaaaaaaaaaaaaaa4, Ns::{bbbbbbbbbbbbbbbbbbbbbbbbbbb1, bbbbbbbbbbbbbbbbbbbbbbbbbbb2, Sub::{ccccccccccccccccccccccccccc1, cccccccccccccccccccccccccccccccc2}}};\nx = 1;\n",
                "Lib::Ns::Sub::f",
            ),
            (
                "module Main;\nimport Std::{aaaaaaaaaaaaaaaaaaaa1} hiding {IO::eprintln, IO::ddddddddddddddddddddddddddddddddd, IO::eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee, IO::ffffffffffffffffffffffffffffffffffff};\nimport Std::IO;\nx = 1;\n",
                "Std::IO::println",
            ),
        ];
        for (source, item) in cases {
            let item = FullName::parse(item).unwrap();
            let mut content = LatestContent::new(PathBuf::from("main.fix"), source.to_string());
            let edits = create_text_edit_to_import(&item, &mut content);
            let edited = apply_lsp_edits(source, &edits);
            let (before, after) = (effective_imports(source), effective_imports(&edited));
            assert!(
                is_accessible(&after, &item),
                "{} is not accessible in:\n{}",
                item.to_string(),
                edited
            );
            for name in names {
                let name = FullName::parse(name).unwrap();
                if name == item {
                    continue;
                }
                assert_eq!(
                    is_accessible(&before, &name),
                    is_accessible(&after, &name),
                    "importing {} changed the accessibility of {} in:\n{}",
                    item.to_string(),
                    name.to_string(),
                    edited
                );
            }
        }
    }
}
