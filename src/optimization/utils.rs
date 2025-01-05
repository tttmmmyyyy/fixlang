use std::sync::Arc;

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::Set,
    ExprNode, PatternNode,
};

// Replace a free variable of an expression to another name.
// If the name `to` is bound at the place `from` appears, returns Err.
pub fn replace_free_var_of_expr(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &FullName,
) -> Result<Arc<ExprNode>, ()> {
    let mut replacer = FreeVarReplacer {
        from: from.clone(),
        to: to.clone(),
        fail: false,
    };
    let res = replacer.traverse(expr);
    if replacer.fail {
        return Err(());
    }
    Ok(res.expr)
}

pub struct FreeVarReplacer {
    from: FullName,
    to: FullName,
    fail: bool,
}

impl ExprVisitor for FreeVarReplacer {
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var().clone();
        // If the visited variable is not equal to `from`, do nothing.
        if var.name != self.from {
            return EndVisitResult::unchanged(expr);
        }
        let local_names = state.scope.local_names();
        // If `from` is shadowed, do nothing.
        if local_names.contains(&self.from.to_string()) {
            return EndVisitResult::unchanged(expr);
        }
        // If the `to` is shadowed, raise an error.
        if state.scope.local_names().contains(&self.to.name) {
            self.fail = true;
            return EndVisitResult::unchanged(expr);
        }
        let expr = expr.set_var_var(var.set_name(self.to.clone()));
        EndVisitResult::changed(expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_tyanno(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_make_struct(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_array_lit(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_ffi_call(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

// Generate new names that is not in the set `names_set`.
pub fn generate_new_names(names_set: &Set<FullName>, n: usize) -> Vec<FullName> {
    let mut names = vec![];
    for _ in 0..n {
        let mut var_name_no = 0;
        let var_name = loop {
            let var_name = format!("#v{}", var_name_no);
            let var_name = FullName::local(&var_name);
            if !names_set.contains(&var_name) {
                break var_name;
            }
            var_name_no += 1;
        };
        names.push(var_name);
    }
    names
}

// Rename the names in the pattern to disjoint with the set `names_set`.
// Also, apply the same renaming to the given expression `value`.
pub fn rename_pattern_value_names(
    names_set: &Set<FullName>,
    mut pattern: Arc<PatternNode>,
    mut value: Arc<ExprNode>,
) -> (Arc<PatternNode>, Arc<ExprNode>) {
    let pattern_vars = pattern.pattern.vars();
    let all_names = pattern_vars.union(names_set).cloned().collect::<Set<_>>();
    let mut renamed: Vec<FullName> = vec![];
    for name in names_set.iter() {
        if pattern_vars.contains(name) {
            renamed.push(name.clone());
        }
    }
    let new_names = generate_new_names(&all_names, renamed.len());
    for (old, new) in renamed.into_iter().zip(new_names.into_iter()) {
        pattern = pattern.rename_var(&old, &new);
        value = replace_free_var_of_expr(&value, &old, &new).unwrap();
    }
    (pattern, value)
}
