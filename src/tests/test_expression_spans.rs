//! The span of an expression the parser builds from parts covers the source it was parsed from,
//! including the parentheses of a parenthesized part at either end.

use crate::{
    ast::{
        expr::{Expr, ExprNode},
        name::FullName,
        program::{EndNode, SymbolExpr},
    },
    configuration::Configuration,
    parse::{
        parser::parse_source_file,
        sourcefile::{SourceFile, SourcePos},
    },
};
use std::{fs, sync::Arc};
use tempfile::TempDir;

/// A global value parsed from a file of its own.
struct ParsedValue {
    /// The directory holding the file. It keeps the file on disk while the value is examined,
    /// since looking up a position resolves the path of the file, which must exist.
    _dir: TempDir,
    /// The expression the value is defined by.
    expr: Arc<ExprNode>,
    /// The content of the file.
    content: String,
}

/// Parses `expr` as the body of a global value.
fn parse_value(expr: &str) -> ParsedValue {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("main.fix");
    let content = format!("module Main;\n\nv : I64 = {};\n", expr);
    fs::write(&path, &content).unwrap();
    let source = SourceFile::from_file_path_and_content(path, content.clone());
    let program = parse_source_file(source, &Configuration::develop_mode()).unwrap();
    let name = FullName::from_strs(&["Main"], "v");
    let SymbolExpr::Simple(value) = &program.global_values[&name].expr else {
        panic!("`v` is not a simple value");
    };
    ParsedValue {
        _dir: dir,
        expr: value.expr.clone(),
        content,
    }
}

/// The text of `content` that the span of `expr` covers.
fn span_text(expr: &ExprNode, content: &str) -> String {
    let span = expr.source.clone().unwrap();
    content[span.start..span.end].to_string()
}

/// Verifies that an expression built from parts spans its whole source, for each way of building
/// one, when a parenthesized part or a closing parenthesis lies at an end of the source.
#[test]
pub fn test_an_expression_built_from_parts_spans_the_parentheses_at_its_ends() {
    let exprs = [
        "f(1, 2)",
        "(f)(1)",
        "(f)()",
        "(1).f(2)",
        "(1).f",
        "(a).b[0]",
        "(a).b.c",
        "f $ (1)",
        "(1) + (2)",
        "(1) == (2)",
        "-(1)",
        "(f) >> (g)",
        "(f) << (g)",
        "(a) && (b)",
        "(a) || (b)",
        "(a) ;; (b)",
        "*(a)",
        "let x = 1; (x)",
    ];
    let wrong_spans = exprs
        .into_iter()
        .map(|expr| {
            let value = parse_value(expr);
            (expr, span_text(&value.expr, &value.content))
        })
        .filter(|(expr, span_text)| expr != span_text)
        .collect::<Vec<_>>();
    assert!(
        wrong_spans.is_empty(),
        "(expression, the text its span covers): {:?}",
        wrong_spans
    );
}

/// The value of a `let` that holds a `*` spans its source, the operand's parentheses included.
#[test]
pub fn test_the_value_of_a_let_spans_the_parentheses_of_a_bind_operand() {
    let value = parse_value("let x = 1; *(m)");
    let Expr::Let(_, _, let_value) = &*value.expr.expr else {
        panic!("`v` is not a `let`");
    };
    assert_eq!(span_text(let_value, &value.content), "*(m)");
}

/// Verifies that a position in the operand of a `*` finds the name written there, after the `*`
/// has been turned into a bind.
#[test]
pub fn test_a_name_in_the_operand_of_a_bind_is_found_at_its_position() {
    let value = parse_value("*act");
    let pos = SourcePos {
        input: value.expr.source.as_ref().unwrap().input.clone(),
        // The middle of the name, since a position just after a name also counts as in it.
        pos: value.content.find("*act").unwrap() + 2,
    };
    let Some(EndNode::Expr(var, _)) = value.expr.find_node_at(&pos) else {
        panic!("no name is found at the operand");
    };
    assert_eq!(var.name.name, "act");
}

/// Verifies that the binds the `*`s in a parenthesized expression make span the text inside the
/// parentheses, as the expression does, both for the body of a value and for the value of a `let`.
#[test]
pub fn test_the_binds_in_a_parenthesized_expression_span_the_text_inside() {
    let value = parse_value("(*(m))");
    assert_eq!(span_text(&value.expr, &value.content), "*(m)");

    let value = parse_value("let x = 1; (*(m))");
    let Expr::Let(_, _, let_value) = &*value.expr.expr else {
        panic!("`v` is not a `let`");
    };
    assert_eq!(span_text(let_value, &value.content), "*(m)");
}

/// Verifies that no position in an expression holding `*`s finds a name the compiler generated,
/// such as the variable a `*x` binds its result to, which the source never writes.
#[test]
pub fn test_no_position_finds_a_name_the_source_never_writes() {
    let value = parse_value("let x = *(m); f(*(g)(x), *n)");
    let span = value.expr.source.clone().unwrap();
    let generated = (span.start..=span.end)
        .filter_map(|pos| {
            let pos = SourcePos {
                input: span.input.clone(),
                pos,
            };
            match value.expr.find_node_at(&pos) {
                Some(EndNode::Expr(var, _)) if var.name.name.starts_with('#') => {
                    Some((pos.pos, var.name.name))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert!(
        generated.is_empty(),
        "(position, name found): {:?}",
        generated
    );
}
