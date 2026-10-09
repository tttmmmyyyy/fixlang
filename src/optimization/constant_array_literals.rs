/*
Turn each array literal whose elements are all number literals into the operation that evaluates it
to a constant in the program's data.

The operation is free to duplicate, so the inliner puts a global holding such a literal wherever the
global is read, and the reader sees the length and the elements of the array as constants.
*/

use crate::ast::{
    expr::{expr_builtin, Expr, ExprNode},
    program::Program,
    traverse::{EndVisitResult, ExprVisitor, VisitState},
};
use crate::fixstd::builtin::{ConstantArrayLitOp, NumberLiteral};
use std::sync::Arc;

/// Replaces each array literal of number literals in the program's symbols with a `ConstantArrayLitOp`.
pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        let res = ConstantArrayLiteralMaker {}.traverse(sym.expr.as_ref().unwrap());
        if res.changed {
            sym.expr = Some(res.expr);
        }
    }
}

/// Replaces an array literal of number literals with a `ConstantArrayLitOp`, and leaves every other
/// expression as it is.
struct ConstantArrayLiteralMaker {}

impl ExprVisitor for ConstantArrayLiteralMaker {
    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        // An empty literal allocates an empty array, which is what `Array::empty` does too.
        let elements = expr
            .get_array_lit_elements()
            .iter()
            .map(number_literal_of)
            .collect::<Option<Vec<_>>>()
            .filter(|elements| !elements.is_empty());
        let Some(elements) = elements else {
            return EndVisitResult::unchanged(expr);
        };
        let ty = expr
            .type_
            .clone()
            .expect("an array literal is typed by the time the program is optimized");
        EndVisitResult::changed(
            expr_builtin(
                Box::new(ConstantArrayLitOp { elements }),
                ty.clone(),
                expr.source.clone(),
            )
            .set_type(ty),
        )
    }
}

/// The number literal `expr` is, looking through the type annotations around it, and `None` where
/// it is anything else.
///
/// # Examples
/// `1`, `(1 : I64)` and `((1 : I64) : I64)` are the literal `1`; `1 + 2` is `None`.
fn number_literal_of(expr: &Arc<ExprNode>) -> Option<NumberLiteral> {
    match &*expr.expr {
        Expr::Builtin(builtin) => NumberLiteral::of_op(builtin.op.as_ref()),
        Expr::TyAnno(annotated, _) => number_literal_of(annotated),
        _ => None,
    }
}
