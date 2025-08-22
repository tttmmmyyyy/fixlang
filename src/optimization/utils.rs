use std::sync::Arc;

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
    ExprNode, PatternNode,
};

// Replace free variables of an expression to other names.
pub fn replace_free_var_of_expr(
    expr: &Arc<ExprNode>,
    mut map: Map<FullName, FullName>,
) -> Arc<ExprNode> {
    // If `map` includes a redundant mapping, we can skip the replacement.
    map.retain(|from, to| from != to);
    if map.is_empty() {
        return expr.clone();
    }
    let mut replacer = FreeVarReplacer::new(map);
    let res = replacer.traverse(expr);
    res.expr.calculate_free_vars()
}

// Replace free variables of an expression to other names.
pub fn replace_free_var_of_expr_one(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &FullName,
) -> Arc<ExprNode> {
    let mut map = Map::default();
    map.insert(from.clone(), to.clone());
    let expr = replace_free_var_of_expr(expr, map);
    expr
}

pub struct FreeVarReplacer {
    // The mapping from old names to new names.
    map: Map<FullName, FullName>,
    // Local names available at this scope.
    shadowed: Set<FullName>,
}

impl FreeVarReplacer {
    fn new(map: Map<FullName, FullName>) -> Self {
        Self {
            map,
            shadowed: Set::default(),
        }
    }

    // When visiting an expression where a new local name is introduced, determine whether to rename that local name and compute the new name.
    fn create_rename_of_local_names(
        &self,
        introduced_names: &Vec<FullName>,
        expr: &Arc<ExprNode>,
    ) -> Map<FullName, FullName> {
        // If the local name being introduced belongs to `self.to_names`, we need to change the local name to something else.
        // The conditions that the new name must satisfy are:
        // - It must not conflict with `self.to_names`.
        // - It must not conflict with the free names of this expression.
        // - Additionally, the local names should not conflict with each other.

        let introduced_names_set = introduced_names.iter().cloned().collect::<Set<FullName>>();
        assert!(
            introduced_names_set.len() == introduced_names.len(),
            "Introduced local names are not unique: {}",
            introduced_names
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let to_names = self.map.values().cloned().collect::<Set<FullName>>();

        let mut renamed_names = vec![];
        for introduced_name in introduced_names {
            if to_names.contains(&introduced_name) {
                renamed_names.push(introduced_name.clone());
            }
        }

        let expr = expr.calculate_free_vars();
        let fvs = expr.free_vars();
        let ng_as_new_name = |name: &FullName| {
            to_names.contains(&name) || fvs.contains(&name) || introduced_names.contains(name)
        };
        let new_names = generate_new_names_pred(ng_as_new_name, renamed_names.len());

        let mut map = Map::default();
        for (old_name, new_name) in renamed_names.into_iter().zip(new_names) {
            map.insert(old_name, new_name);
        }

        map
    }
}

impl ExprVisitor for FreeVarReplacer {
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var().clone();

        // If the visited variable is not in the map, do nothing.
        if self.map.get(&var.name).is_none() {
            return EndVisitResult::unchanged(expr);
        }

        let to = self.map.get(&var.name).unwrap();
        let expr = expr.set_var_var(var.set_name(to.clone()));
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
        let mut changed = false;
        let mut llvm = expr.get_llvm().as_ref().clone();

        let generator = &mut llvm.generator;
        for llvm_fv in generator.free_vars_mut() {
            // Replace
            if let Some(to) = self.map.get(llvm_fv) {
                if llvm_fv == to {
                    continue; // No need to replace if they are the same.
                }

                *llvm_fv = to.clone();
                changed = true;
            }
        }

        if !changed {
            return EndVisitResult::unchanged(expr);
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
        let mut params = expr.get_lam_params();
        assert_eq!(
            params.len(),
            1,
            "This function does not support multi-parameter lambdas."
        );
        let introduced_names: Vec<FullName> = params.iter().map(|p| p.name.clone()).collect();

        let bak_map = self.map.clone();
        let bak_shadowed = self.shadowed.clone();

        for name in &introduced_names {
            self.map.remove(name);
            self.shadowed.insert(name.clone());
        }

        let rename = self.create_rename_of_local_names(&introduced_names, expr);
        for (org, renamed) in rename.iter() {
            self.map.insert(org.clone(), renamed.clone());
        }

        if self.map.is_empty() {
            self.map = bak_map;
            self.shadowed = bak_shadowed;
            return StartVisitResult::Return;
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
        self.shadowed = bak_shadowed;

        StartVisitResult::ReplaceAndReturn(expr)
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> crate::ast::traverse::StartVisitResult {
        let bound = expr.get_let_bound();
        let bound_res = self.traverse(&bound);
        let changed = bound_res.changed;
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
        let bak_shadowed = self.shadowed.clone();

        for name in &introduced_names {
            self.map.remove(name);
            self.shadowed.insert(name.clone());
        }

        let rename = self.create_rename_of_local_names(&introduced_names, &expr);
        for (org, renamed) in rename.iter() {
            self.map.insert(org.clone(), renamed.clone());
        }
        if self.map.is_empty() {
            self.map = bak_map;
            self.shadowed = bak_shadowed;
            if changed {
                return StartVisitResult::ReplaceAndReturn(expr);
            } else {
                return StartVisitResult::Return;
            }
        }

        // Rename the local names.
        let pattern = expr.get_let_pat();
        let pattern = pattern.rename_by_map(&rename);
        let value = expr.get_let_value();
        let value = self.traverse(&value).expr;
        let expr = expr.set_let_pat(pattern).set_let_value(value);

        self.map = bak_map;
        self.shadowed = bak_shadowed;

        StartVisitResult::ReplaceAndReturn(expr)
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
        let mut changed;

        let cond = expr.get_match_cond();
        let cond_res = self.traverse(&cond);
        changed = cond_res.changed;
        let cond = cond_res.expr;
        let expr = expr.set_match_cond(cond);

        let mut pat_vals = expr.get_match_pat_vals();

        for (pat, val) in pat_vals.iter_mut() {
            let introduced_names = pat.pattern.vars().into_iter().collect::<Vec<_>>();

            let bak_map = self.map.clone();
            let bak_shadowed = self.shadowed.clone();

            for name in &introduced_names {
                self.map.remove(name);
                self.shadowed.insert(name.clone());
            }

            let rename = self.create_rename_of_local_names(&introduced_names, &expr);
            for (org, renamed) in rename.iter() {
                self.map.insert(org.clone(), renamed.clone());
            }
            if self.map.is_empty() {
                self.map = bak_map;
                self.shadowed = bak_shadowed;
                continue;
            }
            changed = true;

            *pat = pat.rename_by_map(&rename);
            *val = self.traverse(&val).expr;

            self.map = bak_map;
            self.shadowed = bak_shadowed;
        }
        let expr = expr.set_match_pat_vals(pat_vals);

        if !changed {
            return StartVisitResult::Return;
        }
        StartVisitResult::ReplaceAndReturn(expr)
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

// Generate new names that is not in the set `ng_list`.
pub fn generate_new_names(ng_list: &Set<FullName>, n: usize) -> Vec<FullName> {
    generate_new_names_pred(|name| ng_list.contains(name), n)
}

// Generate `n` new names satisfies `!is_ng_name(x)`
pub fn generate_new_names_pred(is_ng_name: impl Fn(&FullName) -> bool, n: usize) -> Vec<FullName> {
    let mut names = vec![];
    let mut var_name_no = 0;
    for _ in 0..n {
        let var_name = loop {
            let var_name = format!("#v{}", var_name_no);
            var_name_no += 1;
            let var_name = FullName::local(&var_name);
            if !is_ng_name(&var_name) {
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
    pattern = pattern.rename_by_map(&renaming);
    value = replace_free_var_of_expr(&value, renaming);

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
    let new_value = replace_free_var_of_expr(&old_value, renaming);
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
