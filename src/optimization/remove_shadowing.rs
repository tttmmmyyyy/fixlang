// This module provides a function to rename local names in an expression so that shadowing does not occur in it.
use std::sync::Arc;

use crate::{
    ast::{
        expr::ExprNode,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    optimization::utils::rename_lam_param_avoiding,
};

use super::utils::{rename_let_pattern_avoiding, rename_match_pattern_avoiding};

pub fn run_on_expr(expr: &Arc<ExprNode>) -> Arc<ExprNode> {
    let mut renamer = Renamer {};
    renamer.traverse(expr).expr
}

struct Renamer {}

impl ExprVisitor for Renamer {
    fn start_visit_var(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Get the parameter name
        let params = expr.get_lam_params();
        assert_eq!(params.len(), 1);
        let param = params[0].clone();
        let param = &param.name;

        // If there is no conflict with the current scope, do nothing
        if !state.scope.has_value(&param.name) {
            return StartVisitResult::VisitChildren;
        }

        // Rename the parameter name
        let local_names = state.scope.local_names_as_fullname();
        let expr = rename_lam_param_avoiding(&local_names, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_lam(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Get the local names introduced
        let local_names = expr.get_let_pat().pattern.vars();

        // Do nothing if there are no conflicting names
        if local_names
            .iter()
            .all(|local_name| !state.scope.has_value(&local_name.name))
        {
            return StartVisitResult::VisitChildren;
        }

        // If there are conflicting names, rename the local names
        let local_names = state.scope.local_names_as_fullname();
        let expr = rename_let_pattern_avoiding(&local_names, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_let(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // If there are no local names introduced that cause shadowing, do nothing
        let mut shadowing = false;
        for (pat, _val) in expr.get_match_pat_vals() {
            for local_name in pat.pattern.vars() {
                if state.scope.has_value(&local_name.name) {
                    shadowing = true;
                    break;
                }
            }
        }
        if !shadowing {
            return StartVisitResult::VisitChildren;
        }

        // If there are conflicting names, rename the local names
        let local_names = state.scope.local_names_as_fullname();
        let expr = rename_match_pattern_avoiding(&local_names, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_match(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
