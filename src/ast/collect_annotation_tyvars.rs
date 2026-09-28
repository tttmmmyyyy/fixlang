use crate::{
    ast::{
        expr::{Expr, ExprNode},
        pattern::{Pattern, PatternNode},
        traverse::{ExprVisitor, StartVisitResult, VisitState},
        types::TyVar,
    },
    parse::sourcefile::Span,
};
use std::sync::Arc;

// Collect type variables used in type annotations within the expr.
pub fn collect_annotation_tyvars(expr: &Arc<ExprNode>) -> Vec<(Arc<TyVar>, Option<Span>)> {
    let mut collector = Collector { tyvars: Vec::new() };
    collector.traverse(expr);
    collector.tyvars
}

struct Collector {
    pub tyvars: Vec<(Arc<TyVar>, Option<Span>)>,
}

impl Collector {
    fn collect_from_pattern(&mut self, pattern: &Arc<PatternNode>) {
        match &pattern.pattern {
            Pattern::Var(_, Some(ty)) => {
                ty.free_vars_to_vec_with_span(&mut self.tyvars);
            }
            Pattern::Struct(_, fields, _) => {
                for (_, _, field_pat) in fields {
                    self.collect_from_pattern(field_pat);
                }
            }
            Pattern::Union(_, _, inner_pat) => {
                self.collect_from_pattern(inner_pat);
            }
            Pattern::Var(_, None) => {}
        }
    }
}

impl ExprVisitor for Collector {
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Expr::Let(pattern, _, _) = expr.expr.as_ref() {
            self.collect_from_pattern(pattern);
        }
        StartVisitResult::VisitChildren
    }

    fn start_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Expr::Match(_, arms) = expr.expr.as_ref() {
            for (pattern, _) in arms {
                self.collect_from_pattern(pattern);
            }
        }
        StartVisitResult::VisitChildren
    }

    fn start_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        if let Expr::TyAnno(_, ty) = expr.expr.as_ref() {
            ty.free_vars_to_vec_with_span(&mut self.tyvars);
        }
        StartVisitResult::VisitChildren
    }
}
