use crate::{
    ast::{
        expr::{expr_let_typed, expr_var, var_var, ExprNode},
        name::FullName,
        pattern::PatternNode,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
};
use std::sync::Arc;

/// Replaces the free occurrences of the names of `map` with the names they map to. A local name of
/// `expr` that would capture one of the new names is renamed apart first.
pub fn rename_free_names(expr: &Arc<ExprNode>, mut map: Map<FullName, FullName>) -> Arc<ExprNode> {
    // If `map` includes a redundant mapping, we can skip the replacement.
    map.retain(|from, to| from != to);
    if map.is_empty() {
        return expr.clone();
    }
    let map = map
        .into_iter()
        .map(|(from, to)| (from, expr_var(to, None)))
        .collect::<Map<FullName, Arc<ExprNode>>>();
    let mut substitutor = Substitutor::new(map);
    let res = substitutor.traverse(expr);
    res.expr
}

/// Replaces the free occurrences of `from` in `expr` with `to`.
pub fn rename_free_name(expr: &Arc<ExprNode>, from: &FullName, to: &FullName) -> Arc<ExprNode> {
    let mut map = Map::default();
    map.insert(from.clone(), to.clone());
    let expr = rename_free_names(expr, map);
    expr
}

/// Replaces the free occurrences of `from` in `expr` with the expression `to`, i.e. computes
/// `{expr}[{from} := {to}]`.
pub fn substitute_free_name(
    expr: &Arc<ExprNode>,
    from: &FullName,
    to: &Arc<ExprNode>,
) -> Arc<ExprNode> {
    let mut map = Map::default();
    map.insert(from.clone(), to.clone());
    let mut substitutor = Substitutor::new(map);
    let res = substitutor.traverse(expr);
    res.expr
}

/// An ExprVisitor that performs substitution of free names in an expression, i.e. `{expr0}[x:={expr1}]`
struct Substitutor {
    /// The mapping from names to the expressions they are replaced by.
    map: Map<FullName, Arc<ExprNode>>,
}

/// The substitution state of the scope a binder was entered from, which `Substitutor::leave_scope`
/// puts back.
struct ScopeBackup {
    /// The mapping from names to expressions in force outside the binder.
    map: Map<FullName, Arc<ExprNode>>,
}

impl Substitutor {
    /// Creates a substitutor that replaces each name of `map` with the expression it maps to.
    fn new(map: Map<FullName, Arc<ExprNode>>) -> Self {
        Self { map }
    }

    /// Enter the scope of a binder that introduces `introduced_names` in `expr`: the names it binds
    /// stop being substituted, and each local name that would capture a substituted value is given a
    /// new name. Returns that renaming together with the state that `leave_scope` puts back.
    fn enter_scope(
        &mut self,
        introduced_names: &Vec<FullName>,
        expr: &Arc<ExprNode>,
    ) -> (ScopeBackup, Map<FullName, FullName>) {
        let backup = ScopeBackup {
            map: self.map.clone(),
        };

        for name in introduced_names {
            self.map.remove(name);
        }

        let renaming = self.create_rename_of_local_names(introduced_names, expr);
        for (old, renamed) in renaming.iter() {
            self.map
                .insert(old.clone(), expr_var(renamed.clone(), None));
        }

        (backup, renaming)
    }

    /// Leave the scope entered by `enter_scope`, putting the enclosing scope's substitution back.
    fn leave_scope(&mut self, backup: ScopeBackup) {
        self.map = backup.map;
    }

    /// Decides which of the local names `introduced_names` that `expr` introduces have to be
    /// renamed for the substitution to enter their scope without capturing them, and computes a new
    /// name for each of those. A name that can stay as it is has no entry in the returned map.
    fn create_rename_of_local_names(
        &self,
        introduced_names: &Vec<FullName>,
        expr: &Arc<ExprNode>,
    ) -> Map<FullName, FullName> {
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

        // The names the replacement expressions read, which an introduced name would capture.
        let mut to_names = Set::default();
        for to in self.map.values() {
            to_names.extend(to.free_vars());
        }

        calculate_renaming_bound_vars_avoiding(&to_names, introduced_names.clone(), expr.clone())
    }
}

impl ExprVisitor for Substitutor {
    /// Replaces a variable occurrence with the expression its name maps to, giving the replacement
    /// the occurrence's type when it carries none of its own.
    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let var = expr.get_var().clone();

        // If the visited variable is not in the map, do nothing.
        let Some(to) = self.map.get(&var.name) else {
            return EndVisitResult::unchanged(expr);
        };

        let mut new_expr = to.clone();
        if new_expr.type_.is_none() && expr.type_.is_some() {
            new_expr = new_expr.set_type(expr.type_.clone().unwrap());
        }
        EndVisitResult::changed(new_expr)
    }

    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    /// Substitutes the free names an inline-LLVM node reads: a name mapped to another name is
    /// renamed in place, and a name mapped to a general expression is renamed to a fresh name,
    /// which a `let` wrapped around the node binds to that expression.
    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // A parent expression keeps the subexpression it rebuilt only when the subexpression's
        // traversal reports a change, so every rewrite below has to be recorded in `changed`.
        let mut changed = false;
        let mut llvm = expr.get_llvm().as_ref().clone();

        let generator = &mut llvm.generator;

        // Substitute free names in LLVM.
        // (1) A free name `x` of the node replaced by another name `y` is renamed in the node's list
        //     of free names.
        // (2) A free name `x` replaced by a Fix expression `e` turns the node into
        //     `let {v} = e in {llvm}`, where `v` is a fresh name that (1) renames `x` onto.

        // The names the node reads on entry. Both rewrites answer from this list, so that the
        // substitution is simultaneous. Sorted, so that the result does not depend on the order the
        // generator holds them in.
        let mut llvm_fvs = generator.free_vars();
        llvm_fvs.sort();
        llvm_fvs.dedup();

        // The names a fresh binder must avoid: what the node reads, and what each replacement it
        // uses reads. Those are the names in the scope of the binders (2) wraps around the node.
        let mut ng_names = llvm_fvs.iter().cloned().collect::<Set<FullName>>();
        for llvm_fv in &llvm_fvs {
            if let Some(to) = self.map.get(llvm_fv) {
                ng_names.extend(to.free_vars());
            }
        }
        let bound_count = llvm_fvs
            .iter()
            .filter(|llvm_fv| matches!(self.map.get(*llvm_fv), Some(to) if !to.is_var()))
            .count();
        let mut fresh_names = generate_new_names(&ng_names, bound_count).into_iter();

        // What each of the node's free names is renamed to, and the expressions to bind around the
        // node. A binder named after the name it replaces would capture that name wherever another
        // bound expression reads it, or wherever (1) has just renamed an occurrence onto it, since
        // every bound expression sits outside every one of these binders. A fresh name cannot.
        let mut renaming = Map::default();
        let mut bindings = vec![];
        for llvm_fv in llvm_fvs {
            let Some(to) = self.map.get(&llvm_fv) else {
                continue;
            };
            if to.is_var() {
                renaming.insert(llvm_fv, to.get_var().name.clone());
            } else {
                let fresh = fresh_names.next().unwrap();
                renaming.insert(llvm_fv, fresh.clone());
                bindings.push((fresh, to.clone()));
            }
        }

        // (1)
        for llvm_fv in generator.free_vars_mut() {
            let Some(to_name) = renaming.get(llvm_fv) else {
                continue;
            };
            changed |= llvm_fv != to_name;
            *llvm_fv = to_name.clone();
        }
        let mut expr = expr.set_llvm(llvm);

        // (2)
        for (fresh, bound) in bindings {
            changed = true;
            let pat = PatternNode::make_var(var_var(fresh), None)
                .set_type(bound.type_.as_ref().unwrap().clone());
            expr = expr_let_typed(pat, bound, expr);
        }

        if !changed {
            return EndVisitResult::unchanged(&expr);
        }

        EndVisitResult::changed(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// Substitutes the body of a lambda under the scope its parameter opens, renaming the parameter
    /// when it would capture a name a replacement expression reads. Panics on a lambda taking
    /// several parameters.
    fn start_visit_lam(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let mut params = expr.get_lam_params();
        assert_eq!(
            params.len(),
            1,
            "This function does not support multi-parameter lambdas."
        );
        let introduced_names: Vec<FullName> = params.iter().map(|p| p.name.clone()).collect();

        let (backup, renaming) = self.enter_scope(&introduced_names, expr);

        if self.map.is_empty() {
            self.leave_scope(backup);
            return StartVisitResult::Return;
        }

        // Rename the parameters.
        for param in &mut params {
            if let Some(new_name) = renaming.get(&param.name) {
                *param = param.set_name(new_name.clone());
            }
        }
        let body = expr.get_lam_body().clone();
        let body = self.traverse(&body).expr;
        let expr = expr.set_lam_params(params).set_lam_body(body);

        self.leave_scope(backup);

        StartVisitResult::ReplaceAndReturn(expr)
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// Substitutes the bound expression of a `let` in the enclosing scope and its body under the
    /// scope its pattern opens, renaming the names the pattern binds when they would capture a name
    /// a replacement expression reads.
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
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

        let (backup, renaming) = self.enter_scope(&introduced_names, &expr);
        if self.map.is_empty() {
            self.leave_scope(backup);
            if changed {
                return StartVisitResult::ReplaceAndReturn(expr);
            } else {
                return StartVisitResult::Return;
            }
        }

        // Rename the local names.
        let pattern = expr.get_let_pat();
        let pattern = pattern.rename_by_map(&renaming);
        let value = expr.get_let_value();
        let value = self.traverse(&value).expr;
        let expr = expr.set_let_pat(pattern).set_let_value(value);

        self.leave_scope(backup);

        StartVisitResult::ReplaceAndReturn(expr)
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_if(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    /// Substitutes the condition of a `match` in the enclosing scope and each arm's body under the
    /// scope that arm's pattern opens, renaming the names the pattern binds when they would capture
    /// a name a replacement expression reads.
    fn start_visit_match(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let mut changed;

        let cond = expr.get_match_cond();
        let cond_res = self.traverse(&cond);
        changed = cond_res.changed;
        let cond = cond_res.expr;
        let expr = expr.set_match_cond(cond);

        let mut pat_vals = expr.get_match_pat_vals();

        for (pat, val) in pat_vals.iter_mut() {
            let introduced_names = pat.pattern.vars().into_iter().collect::<Vec<_>>();

            let (backup, renaming) = self.enter_scope(&introduced_names, &expr);
            if self.map.is_empty() {
                self.leave_scope(backup);
                continue;
            }
            changed = true;

            *pat = pat.rename_by_map(&renaming);
            *val = self.traverse(&val).expr;

            self.leave_scope(backup);
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
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
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_eval(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_eval(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }
}

/// Generates `n` fresh names, each of which is absent from `ng_list`.
pub fn generate_new_names(ng_list: &Set<FullName>, n: usize) -> Vec<FullName> {
    generate_new_names_pred(|name| ng_list.contains(name), n)
}

/// Generates `n` new names, each satisfying `!is_ng_name(name)`.
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

/// Renames the names `pattern` binds so that they are disjoint from `ng_list`, and applies the same
/// renaming to `value`, the expression evaluated under the pattern.
pub fn rename_pattern_value_avoiding(
    ng_list: &Set<FullName>,
    mut pattern: Arc<PatternNode>,
    mut value: Arc<ExprNode>,
) -> (Arc<PatternNode>, Arc<ExprNode>) {
    let renaming = calculate_renaming_bound_vars_avoiding(
        ng_list,
        pattern.pattern.vars().into_iter().collect(),
        value.clone(),
    );
    pattern = pattern.rename_by_map(&renaming);
    value = rename_free_names(&value, renaming);

    (pattern, value)
}

/// Renames the names the pattern of the `let` expression `let_expr` binds so that they are disjoint
/// from `ng_list`, and applies the same renaming to its body.
pub fn rename_let_pattern_avoiding(
    ng_list: &Set<FullName>,
    let_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let pattern = let_expr.get_let_pat().clone();
    let value = let_expr.get_let_value().clone();
    let (pattern, value) = rename_pattern_value_avoiding(ng_list, pattern, value);
    let_expr.set_let_pat(pattern).set_let_value(value)
}

/// Renames the names each arm of the `match` expression `match_expr` binds so that they are
/// disjoint from `ng_list`, and applies the same renaming to that arm's body.
pub fn rename_match_pattern_avoiding(
    ng_list: &Set<FullName>,
    match_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    let match_expr = match_expr.clone();
    let mut pat_vals = match_expr.get_match_pat_vals();
    for (pat, val) in pat_vals.iter_mut() {
        let (new_pat, new_val) = rename_pattern_value_avoiding(ng_list, pat.clone(), val.clone());
        *pat = new_pat;
        *val = new_val;
    }
    match_expr.set_match_pat_vals(pat_vals)
}

/// Renames the parameter of the lambda `lam_expr` so that it is disjoint from `ng_list`, and
/// applies the same renaming to the body. Panics on a lambda taking several parameters.
pub fn rename_lam_param_avoiding(
    ng_list: &Set<FullName>,
    lam_expr: Arc<ExprNode>,
) -> Arc<ExprNode> {
    if lam_expr.get_lam_params().len() > 1 {
        panic!("This function does not support multi-parameter lambdas.");
    }
    let old_params = lam_expr.get_lam_params();
    let old_param = old_params[0].clone();
    let old_body = lam_expr.get_lam_body().clone();
    let renaming = calculate_renaming_bound_vars_avoiding(
        ng_list,
        vec![old_param.name.clone()],
        old_body.clone(),
    );

    let new_param = if let Some(new_name) = renaming.get(&old_param.name) {
        old_param.set_name(new_name.clone())
    } else {
        old_param.clone()
    };
    let new_body = rename_free_names(&old_body, renaming);
    lam_expr
        .set_lam_params(vec![new_param])
        .set_lam_body(new_body)
}

/// Computes how to rename the variables `bound_vars` that a `let`, `match` or lambda binds over the
/// expression `value`, so that the renamed variables are disjoint from `ng_list`. A variable that
/// can keep its name has no entry in the returned map.
fn calculate_renaming_bound_vars_avoiding(
    ng_list: &Set<FullName>,
    bound_vars: Vec<FullName>,
    value: Arc<ExprNode>,
) -> Map<FullName, FullName> {
    // Calculate the set of names that should be renamed.
    let mut names_to_rename: Vec<FullName> = vec![];
    for name in bound_vars.iter() {
        if ng_list.contains(name) {
            names_to_rename.push(name.clone());
        }
    }

    // Calculate the set of names that should be avoided when we decide new names.
    let mut ng_list = ng_list.clone();
    for var in value.free_vars() {
        ng_list.insert(var.clone()); // Avoid shadowing free variables by bound variables.
    }
    for var in bound_vars.iter() {
        ng_list.insert(var.clone()); // Avoid conflicts with other bound variables.
    }

    // Decide new names.
    let new_names = generate_new_names(&ng_list, names_to_rename.len());

    // Create the renaming map.
    let mut renaming: Map<FullName, FullName> = Map::default();
    for (old, new) in names_to_rename.into_iter().zip(new_names.into_iter()) {
        renaming.insert(old, new);
    }
    renaming
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::{expr_let, expr_llvm, expr_match};
    use crate::ast::types::tycon;
    use crate::fixstd::builtin::{
        make_i64_ty, make_ptr_ty, make_tuple_name_abs, make_tuple_ty, InlineLLVMMakeStructBody,
        InlineLLVMNullPtrLit,
    };

    /// `name` in the empty namespace, where the bindings local to an expression live.
    fn local(name: &str) -> FullName {
        FullName::local(name)
    }

    /// An inline-LLVM expression building a tuple out of the local names `names`, in that order, so
    /// that it reads exactly those names.
    fn llvm_with_free_names(names: &[&str]) -> Arc<ExprNode> {
        let generator = InlineLLVMMakeStructBody {
            field_names: names.iter().map(|name| local(name)).collect(),
        };
        let ty = make_tuple_ty(vec![make_i64_ty(); names.len()]);
        expr_llvm(Box::new(generator), ty.clone(), None).set_type(ty)
    }

    /// A null pointer, as an expression a name can be mapped to that is not a variable.
    fn null_ptr() -> Arc<ExprNode> {
        expr_llvm(Box::new(InlineLLVMNullPtrLit {}), make_ptr_ty(), None).set_type(make_ptr_ty())
    }

    /// The names the inline-LLVM expression `expr` reads from its enclosing scope, in the order it
    /// holds them.
    fn llvm_free_names(expr: &Arc<ExprNode>) -> Vec<FullName> {
        expr.get_llvm().generator.free_vars()
    }

    /// The names bound by the `let`s wrapped around an inline-LLVM expression, outermost first,
    /// together with the expression itself.
    fn peel_lets(expr: &Arc<ExprNode>) -> (Vec<FullName>, Arc<ExprNode>) {
        let mut binders = vec![];
        let mut expr = expr.clone();
        while expr.is_let() {
            binders.push(expr.get_let_pat().get_var().name.clone());
            expr = expr.get_let_value();
        }
        (binders, expr)
    }

    /// The pattern `(x, y)`, which binds both of the names `x_to_a_and_y_to_y` substitutes.
    fn binds_x_and_y() -> Arc<PatternNode> {
        PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                (
                    "0".to_string(),
                    PatternNode::make_var(var_var(local("x")), None),
                ),
                (
                    "1".to_string(),
                    PatternNode::make_var(var_var(local("y")), None),
                ),
            ],
        )
    }

    /// A substitutor renaming `x` to `a` and leaving `y` where it is. An inline-LLVM node reading
    /// both applies the identity mapping second, after it has already renamed a name.
    fn x_to_a_and_y_to_y() -> Substitutor {
        let mut map = Map::default();
        map.insert(local("x"), expr_var(local("a"), None));
        map.insert(local("y"), expr_var(local("y"), None));
        Substitutor::new(map)
    }

    /// Verifies that an inline-LLVM node reports a change once any of its free names is renamed,
    /// even when a name mapped to itself is substituted afterwards.
    #[test]
    fn renaming_one_llvm_free_name_reports_a_change() {
        let res = x_to_a_and_y_to_y().traverse(&llvm_with_free_names(&["x", "y"]));
        assert_eq!(llvm_free_names(&res.expr), vec![local("a"), local("y")]);
        assert!(res.changed);
    }

    /// Verifies that an inline-LLVM node whose only substituted name maps to itself reports no
    /// change, which is what lets a caller take the flag as an answer about the node.
    #[test]
    fn an_identity_mapping_alone_reports_no_change() {
        let res = x_to_a_and_y_to_y().traverse(&llvm_with_free_names(&["y", "w"]));
        assert_eq!(llvm_free_names(&res.expr), vec![local("y"), local("w")]);
        assert!(!res.changed);
    }

    /// Verifies that a `let` whose pattern binds every substituted name still carries the renaming
    /// applied inside its bound expression.
    #[test]
    fn let_keeps_the_rename_of_its_bound_expression() {
        // `let (x, y) = LLVM(x, y) in z`. The pattern binds every name being substituted, so the
        // substitution stops at the binder and the let is rebuilt from its bound expression alone.
        let expr = expr_let(
            binds_x_and_y(),
            llvm_with_free_names(&["x", "y"]),
            expr_var(local("z"), None),
            None,
        );
        let res = x_to_a_and_y_to_y().traverse(&expr);
        assert_eq!(
            llvm_free_names(&res.expr.get_let_bound()),
            vec![local("a"), local("y")]
        );
    }

    /// Verifies that a `match` whose every arm binds all the substituted names still carries the
    /// renaming applied inside its condition.
    #[test]
    fn match_keeps_the_rename_of_its_condition() {
        // `match LLVM(x, y) { (x, y) => z }`, the condition's counterpart of
        // `let_keeps_the_rename_of_its_bound_expression`.
        let expr = expr_match(
            llvm_with_free_names(&["x", "y"]),
            vec![(binds_x_and_y(), expr_var(local("z"), None))],
            None,
        );
        let res = x_to_a_and_y_to_y().traverse(&expr);
        assert_eq!(
            llvm_free_names(&res.expr.get_match_cond()),
            vec![local("a"), local("y")]
        );
    }

    /// Verifies that a name mapped to a general expression is bound around the node and read from
    /// there, leaving the node's other names alone.
    #[test]
    fn a_llvm_replacement_expression_is_bound_around_the_node() {
        let mut map = Map::default();
        map.insert(local("x"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "w"]));
        let (binders, node) = peel_lets(&res.expr);
        assert_eq!(binders.len(), 1);
        assert_eq!(
            llvm_free_names(&node),
            vec![binders[0].clone(), local("w")],
            "the node did not read the name its replacement was bound to"
        );
        assert!(res.changed);
    }

    /// Verifies that a name read twice is bound once, and both occurrences read that one binding.
    #[test]
    fn a_llvm_name_read_twice_is_bound_once() {
        let mut map = Map::default();
        map.insert(local("x"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "x"]));
        let (binders, node) = peel_lets(&res.expr);
        assert_eq!(binders.len(), 1);
        assert_eq!(
            llvm_free_names(&node),
            vec![binders[0].clone(), binders[0].clone()]
        );
    }

    /// Verifies that the name a replacement is bound to avoids what the node already reads.
    #[test]
    fn a_llvm_binder_avoids_a_name_the_node_already_reads() {
        // The node reads `#v0`, the first name the generator hands out.
        let mut map = Map::default();
        map.insert(local("x"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "#v0"]));
        assert!(
            res.expr.has_free_var(&local("#v0")),
            "`#v0` of the enclosing scope was captured"
        );
    }

    /// Verifies that the `let`s an inline-LLVM node's substitution introduces leave the names its
    /// replacements read denoting what they denoted outside.
    #[test]
    fn a_llvm_let_does_not_capture_a_name_another_replacement_reads() {
        // `LLVM(x, z)` under `x := LLVM(z)`, `z := null`. Both replacements are bound around the
        // node, and the one for `x` reads the `z` of the enclosing scope, so `z` stays free.
        let mut map = Map::default();
        map.insert(local("x"), llvm_with_free_names(&["z"]));
        map.insert(local("z"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "z"]));
        assert!(
            res.expr.has_free_var(&local("z")),
            "`z` of the enclosing scope was captured"
        );
    }

    /// Verifies that a `let` the substitution introduces leaves the names it renamed the node onto
    /// denoting what they denoted outside.
    #[test]
    fn a_llvm_let_does_not_capture_a_name_the_node_was_renamed_onto() {
        // `LLVM(x, z)` under `x := z`, `z := null`. The occurrence renamed from `x` to `z` reads
        // the `z` of the enclosing scope, while the occurrence of `z` reads the null pointer.
        let mut map = Map::default();
        map.insert(local("x"), expr_var(local("z"), None));
        map.insert(local("z"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "z"]));
        assert!(
            res.expr.has_free_var(&local("z")),
            "`z` of the enclosing scope was captured"
        );
    }

    /// Verifies that the substitution of an inline-LLVM node's free names is simultaneous: it
    /// applies to the names the node read on entry, so a name a rename introduced is left alone.
    #[test]
    fn substituting_llvm_free_names_is_simultaneous() {
        let mut map = Map::default();
        map.insert(local("x"), expr_var(local("a"), None));
        map.insert(local("a"), null_ptr());
        let res = Substitutor::new(map).traverse(&llvm_with_free_names(&["x", "w"]));
        assert!(
            res.expr.is_llvm(),
            "`a` was substituted again and wrapped the node in a let"
        );
        assert_eq!(llvm_free_names(&res.expr), vec![local("a"), local("w")]);
    }
}
