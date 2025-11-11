// This module provides a transformation that makes all local names defined in an expression unique.
// Not only avoids shadowing but also avoids collisions with names that are already out of scope.
use std::sync::Arc;

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult},
    },
    misc::{Map, Set},
};

pub fn run_on_expr(expr: &Arc<ExprNode>, mut occupied: Set<FullName>) -> Arc<ExprNode> {
    let free_vars = expr.free_vars();
    occupied.extend(free_vars.iter().cloned());
    let mut renamer = Renamer {
        map: Map::default(),
        occupied: occupied,
        name_no: 0,
    };
    renamer.traverse(expr).expr
}

struct Renamer {
    // The renaming map
    map: Map<FullName, FullName>,
    // The set of names that are already used.
    occupied: Set<FullName>,
    // The counter to determine the next name.
    name_no: usize,
}

impl Renamer {
    fn create_rename_of_local_names(
        &mut self,
        local_names: &Vec<FullName>,
    ) -> Map<FullName, FullName> {
        let mut rename = Map::default();
        for local_name in local_names {
            if !self.occupied.contains(local_name) {
                self.occupied.insert(local_name.clone());
                continue;
            }
            let new_name = loop {
                let new_name = FullName::local(&format!("#v{}", self.name_no));
                self.name_no += 1;
                if !self.occupied.contains(&new_name) {
                    break new_name;
                }
            };
            rename.insert(local_name.clone(), new_name.clone());
            self.occupied.insert(new_name);
        }
        rename
    }
}

impl ExprVisitor for Renamer {
    fn start_visit_var(
        &mut self,
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let mut var = expr.get_var().clone();

        if let Some(to) = self.map.get(&var.name) {
            var = var.set_name(to.clone());
        }
        let expr = expr.set_var_var(var);

        StartVisitResult::ReplaceAndReturn(expr)
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
        expr: &std::sync::Arc<crate::ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let mut llvm = expr.get_llvm().as_ref().clone();

        let generator = &mut llvm.generator;
        for llvm_fv in generator.free_vars_mut() {
            // Replace
            if let Some(to) = self.map.get(llvm_fv) {
                *llvm_fv = to.clone();
            }
        }

        let expr = expr.set_llvm(llvm);
        StartVisitResult::ReplaceAndReturn(expr)
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
        let mut params = expr.get_lam_params();
        assert_eq!(
            params.len(),
            1,
            "This function does not support multi-parameter lambdas."
        );
        let introduced_names: Vec<FullName> = params.iter().map(|p| p.name.clone()).collect();

        let bak_map = self.map.clone();

        for name in &introduced_names {
            self.map.remove(name);
        }

        let rename = self.create_rename_of_local_names(&introduced_names);
        for (org, renamed) in rename.iter() {
            self.map.insert(org.clone(), renamed.clone());
        }

        // Rename the parameters.
        for param in &mut params {
            if let Some(new_name) = rename.get(&param.name) {
                *param = param.set_name(new_name.clone());
            }
        }
        let body = expr.get_lam_body().clone();
        let body = self.traverse(&body).expr;
        let expr = expr.set_lam_params(params).set_lam_body(body);

        self.map = bak_map;

        StartVisitResult::ReplaceAndReturn(expr)
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
        let bound = expr.get_let_bound();
        let bound_res = self.traverse(&bound);
        let bound = bound_res.expr;
        let expr = expr.set_let_bound(bound);

        let introduced_names = expr
            .get_let_pat()
            .pattern
            .vars()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let bak_map = self.map.clone();

        for name in &introduced_names {
            self.map.remove(name);
        }

        let rename = self.create_rename_of_local_names(&introduced_names);
        for (org, renamed) in rename.iter() {
            self.map.insert(org.clone(), renamed.clone());
        }

        // Rename the local names.
        let pattern = expr.get_let_pat();
        let pattern = pattern.rename_by_map(&rename);
        let value = expr.get_let_value();
        let value = self.traverse(&value).expr;
        let expr = expr.set_let_pat(pattern).set_let_value(value);

        self.map = bak_map;

        StartVisitResult::ReplaceAndReturn(expr)
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
        let cond = expr.get_match_cond();
        let cond_res = self.traverse(&cond);
        let cond = cond_res.expr;
        let expr = expr.set_match_cond(cond);

        let mut pat_vals = expr.get_match_pat_vals();

        for (pat, val) in pat_vals.iter_mut() {
            let introduced_names = pat.pattern.vars().into_iter().collect::<Vec<_>>();

            let bak_map = self.map.clone();

            for name in &introduced_names {
                self.map.remove(name);
            }

            let rename = self.create_rename_of_local_names(&introduced_names);
            for (org, renamed) in rename.iter() {
                self.map.insert(org.clone(), renamed.clone());
            }

            *pat = pat.rename_by_map(&rename);
            *val = self.traverse(&val).expr;

            self.map = bak_map;
        }
        let expr = expr.set_match_pat_vals(pat_vals);

        StartVisitResult::ReplaceAndReturn(expr)
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

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut crate::ast::traverse::VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}
