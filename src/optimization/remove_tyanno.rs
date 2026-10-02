/*
Remove type annotations from the AST.

This simplifies the AST and makes it easier to implement optimizations.
*/

use std::sync::Arc;

use crate::ast::{
    expr::ExprNode,
    program::{Program, Symbol},
    traverse::{EndVisitResult, ExprVisitor, VisitState},
};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(sym: &mut Symbol) {
    let mut remover = TyAnnoRemover {};
    let res = remover.traverse(&sym.expr.as_ref().unwrap());
    if res.changed {
        sym.expr = Some(res.expr);
    }
}

struct TyAnnoRemover {}

impl ExprVisitor for TyAnnoRemover {
    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        let expr = expr.get_tyanno_expr();
        EndVisitResult::changed(expr)
    }
}
