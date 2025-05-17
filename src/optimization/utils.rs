use std::sync::Arc;

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
    ExprNode, PatternNode,
};

// Replace a free variable of an expression to another name.
pub fn replace_free_var_of_expr(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &FullName,
) -> Arc<ExprNode> {
    if from == to {
        return expr.clone();
    }
    let mut replacer = FreeVarReplacer {
        from: from.clone(),
        to: to.clone(),
    };
    let res = replacer.traverse(expr);
    res.expr.calculate_free_vars()
}

pub struct FreeVarReplacer {
    from: FullName,
    to: FullName,
}

impl FreeVarReplacer {
    // Should we replace `from` to `to` at this scope?
    //
    // If `from` is shadowed, then we should not replace it. so return false.
    // If `from` is not shadowed but `to` is shadowed, then we should not replace it to avoid name conflict, so return false and set `fail` to true.
    fn can_replace_at_state(&mut self, state: &VisitState) -> bool {
        let local_names = state.scope.local_names();
        // If `from` is shadowed, do nothing.
        if local_names.contains(&self.from.to_string()) {
            return false;
        }

        // `to` should be free from shadowing.
        assert!(!state.scope.local_names().contains(&self.to.name));
        true
    }
}

impl ExprVisitor for FreeVarReplacer {
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var().clone();
        // If the visited variable is not equal to `from`, do nothing.
        if var.name != self.from {
            return EndVisitResult::unchanged(expr);
        }

        if !self.can_replace_at_state(state) {
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

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, state: &mut VisitState) -> EndVisitResult {
        if !self.can_replace_at_state(state) {
            return EndVisitResult::unchanged(expr);
        }

        let mut llvm = expr.get_llvm().as_ref().clone();

        let generator = &mut llvm.generator;
        for llvm_fv in generator.free_vars_mut() {
            // Replace `from` to `to`.
            if *llvm_fv == self.from {
                *llvm_fv = self.to.clone();
            }
        }
        let expr = expr.set_llvm(llvm);
        EndVisitResult::changed(expr)
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
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Check if `to` is going to be shadowed here.
        let lam_params = expr.get_lam_params();
        assert_eq!(
            lam_params.len(),
            1,
            "This function does not support multi-parameter lambdas."
        );
        let lam_param = &lam_params[0].clone();
        if lam_param.name != self.to {
            return StartVisitResult::VisitChildren;
        }

        // If `to` is going to be shadowed here, we need to avoid shadowing by renaming parameter of the lambda.
        let mut black_list = Set::default();
        black_list.insert(self.to.clone());
        let expr = rename_lam_param_avoiding(&black_list, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Check if `to` is going to be shadowed here.
        let pattern_vars = expr.get_let_pat().pattern.vars();
        if !pattern_vars.contains(&self.to) {
            return StartVisitResult::VisitChildren;
        }

        // If `to` is going to be shadowed here, we need to avoid shadowing by renaming the pattern.
        let mut black_list = Set::default();
        black_list.insert(self.to.clone());
        let expr = rename_let_pattern_avoiding(&black_list, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
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
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        // Check if `to` is going to be shadowed here.
        let shadowed = expr.get_match_pat_vals().iter().any(|(pat, _)| {
            let pattern_vars = pat.pattern.vars();
            pattern_vars.contains(&self.to)
        });
        if !shadowed {
            return StartVisitResult::VisitChildren;
        }

        // If `to` is going to be shadowed here, we need to avoid shadowing by renaming the pattern.
        let mut black_list = Set::default();
        black_list.insert(self.to.clone());
        let expr = rename_match_pattern_avoiding(&black_list, expr.clone());
        StartVisitResult::ReplaceAndRevisit(expr)
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

// Generate new names that is not in the set `black_list`.
pub fn generate_new_names(black_list: &Set<FullName>, n: usize) -> Vec<FullName> {
    let mut names = vec![];
    let mut var_name_no = 0;
    for _ in 0..n {
        let var_name = loop {
            let var_name = format!("#v{}", var_name_no);
            var_name_no += 1;
            let var_name = FullName::local(&var_name);
            if !black_list.contains(&var_name) {
                break var_name;
            }
        };
        names.push(var_name);
    }
    names
}

// Rename the names in the pattern so that they will be disjoint from the set `black_list`.
// Also, apply the same renaming to the value expression.
pub fn rename_pattern_value_avoiding(
    black_list: &Set<FullName>,
    mut pattern: Arc<PatternNode>,
    mut value: Arc<ExprNode>,
) -> (Arc<PatternNode>, Arc<ExprNode>) {
    let renaming = calculate_renaming_bound_vars_avoiding(
        black_list,
        pattern.pattern.vars().into_iter().collect(),
        value.clone(),
    );
    for (old, new) in renaming.iter() {
        pattern = pattern.rename_var(old, new);
        value = replace_free_var_of_expr(&value, old, new);
    }

    (pattern, value)
}

pub fn rename_let_pattern_avoiding(
    black_list: &Set<FullName>,
    let_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let pattern = let_expr.get_let_pat().clone();
    let value = let_expr.get_let_value().clone();
    let (pattern, value) = rename_pattern_value_avoiding(black_list, pattern, value);
    let_expr.set_let_pat(pattern).set_let_value(value)
}

pub fn rename_match_pattern_avoiding(
    black_list: &Set<FullName>,
    match_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let match_expr = match_expr.clone();
    let mut pat_vals = match_expr.get_match_pat_vals();
    for (pat, val) in pat_vals.iter_mut() {
        let (new_pat, new_val) =
            rename_pattern_value_avoiding(black_list, pat.clone(), val.clone());
        *pat = new_pat;
        *val = new_val;
    }
    match_expr.set_match_pat_vals(pat_vals)
}

pub fn rename_lam_param_avoiding(
    black_list: &Set<FullName>,
    lam_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    if lam_expr.get_lam_params().len() > 1 {
        panic!("This function does not support multi-parameter lambdas.");
    }
    let old_params = lam_expr.get_lam_params();
    let old_param = old_params[0].clone();
    let old_value = lam_expr.get_lam_body().clone();
    let renaming = calculate_renaming_bound_vars_avoiding(
        black_list,
        vec![old_param.name.clone()],
        old_value.clone(),
    );

    let new_param = if let Some(new_name) = renaming.get(&old_param.name) {
        old_param.set_name(new_name.clone())
    } else {
        old_param.clone()
    };
    let new_value = replace_free_var_of_expr(&old_value, &old_param.name, &new_param.name);
    lam_expr
        .set_lam_params(vec![new_param])
        .set_lam_body(new_value)
}

// Consider the situation that let, match or lam expression binds variables `bound_vars` and evaluates the expression `expr`.
// This function calculates how to rename bound variables so that they are disjoint from `black_list`.
fn calculate_renaming_bound_vars_avoiding(
    black_list: &Set<FullName>,
    bound_vars: Vec<FullName>,
    value: Arc<ExprNode>,
) -> Map<FullName, FullName> {
    // Calculate the set of names that should be renamed.
    let mut renamed: Vec<FullName> = vec![];
    for name in bound_vars.iter() {
        if black_list.contains(name) {
            renamed.push(name.clone());
        }
    }

    // Calculate the set of names that should be avoided when we decide new names.
    let mut black_list = black_list.clone();
    let value = value.calculate_free_vars();
    for var in value.free_vars() {
        black_list.insert(var.clone()); // Avoid shadowing free variables by bound variables.
    }
    for var in bound_vars.iter() {
        black_list.insert(var.clone()); // Avoid conflicts with other bound variables.
    }

    // Decide new names.
    let new_names = generate_new_names(&black_list, renamed.len());

    // Create the renaming map.
    let mut renaming: Map<FullName, FullName> = Map::default();
    for (old, new) in renamed.into_iter().zip(new_names.into_iter()) {
        renaming.insert(old, new);
    }
    renaming
}
