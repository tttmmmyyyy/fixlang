//! The span of an expression the parser builds from parts covers the source it was parsed from,
//! including the parentheses of a parenthesized part at either end.

use crate::{
    ast::{
        expr::{Expr, ExprNode},
        name::FullName,
        program::SymbolExpr,
    },
    ast::program::EndNode,
    configuration::Configuration,
    parse::{
        parser::parse_source_file,
        sourcefile::{SourceFile, SourcePos},
    },
};
use std::{fs, path::PathBuf, sync::Arc};
use tempfile::TempDir;

/// Parses `expr` as the body of a global value, and returns the expression with the source of the
/// file it was parsed from.
fn parse_value(expr: &str) -> (Arc<ExprNode>, String) {
    let content = format!("module Main;\n\nv : I64 = {};\n", expr);
    let source = SourceFile::from_file_path_and_content(PathBuf::from("main.fix"), content.clone());
    let program = parse_source_file(source, &Configuration::develop_mode()).unwrap();
    let name = FullName::from_strs(&["Main"], "v");
    let SymbolExpr::Simple(value) = &program.global_values[&name].expr else {
        panic!("`v` is not a simple value");
    };
    (value.expr.clone(), content)
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
            let (value, content) = parse_value(expr);
            (expr, span_text(&value, &content))
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
    let (value, content) = parse_value("let x = 1; *(m)");
    let Expr::Let(_, _, let_value) = &*value.expr else {
        panic!("`v` is not a `let`");
    };
    assert_eq!(span_text(let_value, &content), "*(m)");
}

/// Verifies that a position in the operand of a `*` finds the name written there, which the binds
/// the `*` makes leave in place of the operand.
#[test]
pub fn test_a_name_in_the_operand_of_a_bind_is_found_at_its_position() {
    // A position is looked up in a file that exists.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("main.fix");
    let content = "module Main;\n\nv : IO () = *act;\n";
    fs::write(&path, content).unwrap();
    let source = SourceFile::from_file_path_and_content(path, content.to_string());
    let program = parse_source_file(source, &Configuration::develop_mode()).unwrap();
    let SymbolExpr::Simple(value) = &program.global_values[&FullName::from_strs(&["Main"], "v")].expr
    else {
        panic!("`v` is not a simple value");
    };
    let pos = SourcePos {
        input: value.expr.source.as_ref().unwrap().input.clone(),
        // The middle of the name, since a position just after a name also counts as in it.
        pos: content.find("*act").unwrap() + 2,
    };
    let Some(EndNode::Expr(var, _)) = value.expr.find_node_at(&pos) else {
        panic!("no name is found at the operand");
    };
    assert_eq!(var.name.name, "act");
}
