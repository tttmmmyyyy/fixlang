// This module provides a transformation that makes all local names defined in an expression unique.
// Not only avoids shadowing but also avoids collisions with names that are already out of scope.
use std::sync::Arc;

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    misc::Set,
    optimization::utils::rename_lam_param_avoiding,
};

use super::utils::{rename_let_pattern_avoiding, rename_match_pattern_avoiding};

pub fn run_on_expr(expr: &Arc<ExprNode>, mut occupied: Set<FullName>) -> Arc<ExprNode> {
    let expr = &expr.calculate_free_vars();
    let free_vars = expr.free_vars();
    occupied.extend(free_vars.iter().cloned());
    let mut renamer = Renamer { occupied: occupied };
    renamer.traverse(expr).expr.calculate_free_vars()
}

struct Renamer {
    // The set of names that are already used.
    occupied: Set<FullName>,
}

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
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Get the parameter name
        let params = expr.get_lam_params();
        assert_eq!(params.len(), 1); // Currently, we only support single-parameter lambdas, but this can be extended
        let param = params[0].clone();
        let param = &param.name;
        let conflict = self.occupied.contains(param);

        // If there is no conflict with the occupied names, do nothing
        if !conflict {
            self.occupied.insert(param.clone());
            return StartVisitResult::VisitChildren;
        }

        // Rename the parameter name
        let expr = rename_lam_param_avoiding(&self.occupied, expr.clone());
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
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Get the local names introduced
        let new_names = expr.get_let_pat().pattern.vars();
        let conflict = new_names
            .iter()
            .any(|new_name| self.occupied.contains(&new_name));

        // Do nothing if there are no conflicting names
        if !conflict {
            self.occupied.extend(new_names.iter().cloned());
            return StartVisitResult::VisitChildren;
        }

        // If there are conflicting names, rename the local names
        let expr = rename_let_pattern_avoiding(&self.occupied, expr.clone());
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
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // If there are no local names introduced that cause shadowing, do nothing
        let mut new_names = vec![];
        for (pat, _val) in expr.get_match_pat_vals() {
            for new_name in pat.pattern.vars() {
                new_names.push(new_name.clone());
            }
        }
        let conflict = new_names
            .iter()
            .any(|new_name| self.occupied.contains(&new_name));
        if !conflict {
            self.occupied.extend(new_names.iter().cloned());
            return StartVisitResult::VisitChildren;
        }

        // If there are conflicting names, rename the local names
        let expr = rename_match_pattern_avoiding(&self.occupied, expr.clone());
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
