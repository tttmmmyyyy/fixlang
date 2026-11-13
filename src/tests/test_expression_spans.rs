//! The span of an expression the parser builds from parts covers the source it was parsed from,
//! including the parentheses of a parenthesized part at either end.

use crate::{
    ast::{name::FullName, program::SymbolExpr},
    configuration::Configuration,
    parse::{parser::parse_source_file, sourcefile::SourceFile},
};
use std::path::PathBuf;

/// The source text the span of the expression `expr` covers, where `expr` is parsed as the body of
/// a global value.
fn span_text_of(expr: &str) -> String {
    let content = format!("module Main;\n\nv : I64 = {};\n", expr);
    let source = SourceFile::from_file_path_and_content(PathBuf::from("main.fix"), content.clone());
    let program = parse_source_file(source, &Configuration::develop_mode()).unwrap();
    let name = FullName::from_strs(&["Main"], "v");
    let SymbolExpr::Simple(value) = &program.global_values[&name].expr else {
        panic!("`v` is not a simple value");
    };
    let span = value.expr.source.clone().unwrap();
    content[span.start..span.end].to_string()
}

/// Each expression is built from parts, and has a parenthesized part or a closing parenthesis at an
/// end of its source.
#[test]
pub fn test_an_expression_built_from_parts_spans_the_parentheses_at_its_ends() {
    let exprs = [
        "f(1, 2)",
        "(f)(1)",
        "(f)()",
        "(1).f(2)",
        "(1).f",
        "(a).b[0]",
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
        .map(|expr| (expr, span_text_of(expr)))
        .filter(|(expr, span_text)| expr != span_text)
        .collect::<Vec<_>>();
    assert!(
        wrong_spans.is_empty(),
        "(expression, the text its span covers): {:?}",
        wrong_spans
    );
}
