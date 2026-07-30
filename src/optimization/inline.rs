/*
Inlining optimization.
*/

use std::{mem, sync::Arc};

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
    optimization::uncurry::is_std_fix,
};

use super::application_inlining;

pub const INLINE_COST_THRESHOLD: i32 = 30;

pub fn run(prg: &mut Program) {
    // Calculate free variables of all symbols.
    for (_name, sym) in &mut prg.symbols {
        sym.expr = Some(sym.expr.as_ref().unwrap().clone());
    }

    let mut skip_symbols = Set::default();
    while run_one(prg, &mut skip_symbols) {}
}

// Run inlining optimization once.
pub fn run_one(prg: &mut Program, stable_symbols: &mut Set<FullName>) -> bool {
    let mut changed = false;

    let costs = calculate_inline_costs(prg);
    let symbols = mem::take(&mut prg.symbols);
    let mut inliner = Inliner {
        costs: &costs,
        symbols: symbols.clone(),
    };
    let mut new_symbols: Map<FullName, Symbol> = Map::default();
    let root_value_names = prg.root_value_names();

    for (name, mut sym) in symbols {
        // If call count of the symbol is 0, and it is neither of entry point nor exported value, discard it.
        if costs.get_call_count(&name) == 0 && !root_value_names.contains(&name) {
            changed = true;
            continue;
        }

        // If the symbol is known to be stable, skip it.
        if stable_symbols.contains(&name) {
            new_symbols.insert(name, sym);
            continue;
        }

        // If the new symbol has no free variables, it cannot be inlined furthermore.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            stable_symbols.insert(name.clone());
            new_symbols.insert(name.clone(), sym);
            continue;
        }

        // Traverse the expression and inline the symbol.
        let res = inliner.traverse(&sym.expr.as_ref().unwrap());

        if res.changed {
            // If inlining was done, inline application.
            changed = true;
            sym.expr = Some(res.expr);
            application_inlining::run_on_symbol(&mut sym);
        } else {
            // If inlining was not done, it cannot be inlined furthermore.
            stable_symbols.insert(name.clone());
        }

        // If the new symbol has no free variables, it cannot be inlined furthermore.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            stable_symbols.insert(name.clone());
        }

        new_symbols.insert(name, sym);
    }

    prg.symbols = new_symbols;

    changed
}

pub fn calculate_inline_costs(prg: &Program) -> InlineCosts {
    let mut costs = InlineCosts::new();
    // The global each alias refers to, from which `aliases_with_an_end` decides which of them can be
    // expanded.
    let mut alias_targets: Map<FullName, FullName> = Map::default();
    for (name, sym) in &prg.symbols {
        let mut cost_calculator = InlineCostCalculator::new(name.clone());
        cost_calculator.traverse(&sym.expr.as_ref().unwrap());
        costs.add_cost_calculation_result(cost_calculator);

        let expr = sym.expr.as_ref().unwrap();
        // If the expression is of the form `|x, y, ...| {llvm}`, then set as `is_llvm_lam`.
        let (_params, body) = expr.destructure_lam_sequence();
        let is_llvm_lam = body.is_llvm();
        costs.costs.get_mut(name).unwrap().is_llvm_lam = is_llvm_lam;

        // If the expression is a primitive literal, set as `is_primitive_literal`.
        if expr.is_llvm() {
            let is_primitive_literal = expr.get_llvm().generator.is_primitve_literal();
            costs.costs.get_mut(name).unwrap().is_primitive_literal = is_primitive_literal;
        }

        // If the expression is instantiated by `Std::fix`, set as `is_std_fix`.
        costs.costs.get_mut(name).unwrap().is_std_fix = is_std_fix(name);

        // If the expression is an alias to another global value, record what it aliases.
        if expr.is_var() {
            let var_name = &expr.get_var().name;
            assert!(var_name.is_global());
            alias_targets.insert(name.clone(), var_name.clone());
        }
    }
    for name in aliases_with_an_end(&alias_targets) {
        costs.costs.get_mut(&name).unwrap().is_expandable_alias = true;
    }
    costs
}

/// The aliases whose chain of aliases ends: following it from one of these reaches a definition that
/// is not an alias, which is the definition expanding the alias arrives at. Following it from any
/// other alias comes back to a name it has already passed.
/// The aliases whose chain of aliases ends: following it from one of these reaches a definition that
/// is not an alias, which is the definition expanding the alias arrives at. Following it from any
/// other alias comes back to a name it has already passed.
fn aliases_with_an_end(alias_targets: &Map<FullName, FullName>) -> Set<FullName> {
    let mut with_an_end = Set::default();
    let mut settled: Set<FullName> = Set::default();
    for start in alias_targets.keys() {
        if settled.contains(start) {
            continue;
        }
        // Walk the chain until it reaches a definition that is not an alias, a name whose answer is
        // already known, or a name this walk has already passed.
        let mut path = vec![];
        let mut on_path: Set<FullName> = Set::default();
        let mut name = start.clone();
        let ends = loop {
            if on_path.contains(&name) {
                break false;
            }
            if settled.contains(&name) {
                break with_an_end.contains(&name);
            }
            let Some(target) = alias_targets.get(&name) else {
                break true;
            };
            on_path.insert(name.clone());
            path.push(name);
            name = target.clone();
        };
        for name in path {
            settled.insert(name.clone());
            if ends {
                with_an_end.insert(name);
            }
        }
    }
    with_an_end
}

// A struct to store information about the cost of inlining a symbol.
pub struct InlineCost {
    // The number of times the symbol is called.
    call_count: usize,
    // The complexity of the expression.
    complexity: usize,
    // Is the function calling itself?
    is_self_recursive: bool,
    // Is the top-level construct a lambda expression?
    is_lambda: bool,
    // Is the expression of the form `|x, y, ...| {llvm}`?
    is_llvm_lam: bool,
    // Is the expression primitive literal?
    is_primitive_literal: bool,
    // Is this expression an alias to another value whose own chain of aliases ends at a definition
    // that is not an alias?
    //
    // Example:
    // ```
    // x = y;
    // ```
    //
    // An alias whose chain runs into a cycle is excluded. Expanding one replaces its body with the
    // next name around the cycle, which is a change every round, so the inlining fixpoint would
    // never converge.
    is_expandable_alias: bool,
    // Is the expression instantiated by Std::fix?
    is_std_fix: bool,
}

impl InlineCost {
    fn new() -> Self {
        InlineCost {
            call_count: 0,
            complexity: 0,
            is_self_recursive: false,
            is_lambda: false,
            is_llvm_lam: false,
            is_primitive_literal: false,
            is_std_fix: false,
            is_expandable_alias: false,
        }
    }

    // Returns true if the symbol can be inlined even at a non-call site.
    fn inline_at_non_call_site(&self) -> bool {
        if self.is_std_fix {
            return false;
        }
        if self.is_primitive_literal {
            // TODO: Allow (not only literals but) constant primitives to be inlined too.
            return true;
        }
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        if self.is_expandable_alias {
            return true;
        }
        return false;
        // NOTE
        // * Even values with simple types should not be inlined if the computation is complex.
        // * Values created using FFI_CALL are heavy.
        // * Boxed types and Strings also increase memory allocation when inlined, such as string literals.
    }

    // Returns true if the symbol can be inlined at a call site.
    pub fn inline_at_call_site(&self) -> bool {
        if self.is_std_fix {
            return false;
        }
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        if !self.is_lambda {
            return false;
        }
        self.complexity as i32 <= INLINE_COST_THRESHOLD
    }
}

// The map from each symbol to the cost of inlining it.
pub struct InlineCosts {
    pub costs: Map<FullName, InlineCost>,
}

impl InlineCosts {
    fn new() -> Self {
        InlineCosts {
            costs: Map::default(),
        }
    }

    fn insert_cost_if_absent(&mut self, name: &FullName) {
        if !self.costs.contains_key(name) {
            self.costs.insert(name.clone(), InlineCost::new());
        }
    }

    pub fn get_call_count(&self, name: &FullName) -> usize {
        self.costs.get(name).map_or(0, |c| c.call_count)
    }

    // After `InlineCostCalculator` has been executed, add its result to `InlineCosts`.
    fn add_cost_calculation_result(&mut self, cost: InlineCostCalculator) {
        // For each global symbol called from the symbol where `InlineCostCalculator` has been executed, add the call count.
        for (sym, count) in cost.call_count {
            self.insert_cost_if_absent(&sym);
            self.costs.get_mut(&sym).unwrap().call_count += count;
        }

        // Set other fields for the symbol itself that `InlineCostCalculator` has traversed.
        self.insert_cost_if_absent(&cost.name);
        let inline_cost = self.costs.get_mut(&cost.name).unwrap();
        inline_cost.complexity = cost.complexity;
        inline_cost.is_self_recursive = cost.is_refer_self;
        inline_cost.is_lambda = cost.is_lambda;
    }
}

struct InlineCostCalculator {
    // The name of the symbol.
    name: FullName,
    // For each global symbol, the count of calls.
    call_count: Map<FullName, usize>,
    // The cost of the symbol.
    complexity: usize,
    // Is the symbol referring itself?
    is_refer_self: bool,
    // Is the top-level construct a lambda expression?
    is_lambda: bool,
}

impl InlineCostCalculator {
    fn new(name: FullName) -> Self {
        InlineCostCalculator {
            name,
            call_count: Map::default(),
            complexity: 0,
            is_refer_self: false,
            is_lambda: false,
        }
    }

    fn on_find_usage_of_global_name(&mut self, used_name: &FullName) {
        // If calling a global symbol, increase the call count.
        assert!(used_name.is_global());
        if let Some(count) = self.call_count.get_mut(used_name) {
            *count += 1;
        } else {
            self.call_count.insert(used_name.clone(), 1);
        }

        // If it calls itself, set `is_call_self`.
        if used_name == &self.name {
            self.is_refer_self = true;
        }
    }
}

impl ExprVisitor for InlineCostCalculator {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let var_name = &expr.get_var().name;
        if var_name.is_global() {
            self.on_find_usage_of_global_name(var_name);
            // Add the complexity of the symbol.
            self.complexity += 1;
        }
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = false;
        for free_name in expr.free_vars() {
            if free_name.is_global() {
                self.on_find_usage_of_global_name(&free_name);
            }
        }
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = true;
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // If the let binding is of the form `let {local_var0} = {local_var1} in (...)`, does not increase the complexity.
        self.complexity += 1;
        let pat = expr.get_let_pat();
        if pat.is_var() && pat.get_var().name.is_local() {
            self.complexity -= 1;
        }
        self.is_lambda = false;
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
        self.complexity += 1;
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_match(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.is_lambda = false;

        // If the match is of the form `match {local_var0} { {local_var1} -> (...) }`, does not increase the complexity.
        self.complexity += 1;
        let match_cond = expr.get_match_cond();
        if match_cond.is_var() && match_cond.get_var().name.is_local() {
            let pat_vals = expr.get_match_pat_vals();
            if pat_vals.len() == 1
                && pat_vals[0].1.is_var()
                && pat_vals[0].1.get_var().name.is_local()
            {
                self.complexity -= 1;
            }
        }

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
        self.is_lambda = false;

        // Does not increase the complexity.
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
        self.is_lambda = false;
        self.complexity += 1;
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
        self.is_lambda = false;
        self.complexity += 1;
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
        self.is_lambda = false;
        self.complexity += 1;
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
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }
}

struct Inliner<'c> {
    // The cost of inlining.
    costs: &'c InlineCosts,
    // All symbols.
    symbols: Map<FullName, Symbol>,
}

impl<'c> ExprVisitor for Inliner<'c> {
    fn start_visit_var(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // If the variable is global, then try to inline the variable.
        let var_name = &expr.get_var().name;
        if var_name.is_local() {
            return EndVisitResult::unchanged(expr);
        }

        let cost = self.costs.costs.get(var_name).unwrap();
        if !cost.inline_at_non_call_site() {
            return EndVisitResult::unchanged(expr);
        }

        let sym = self.symbols.get(var_name).unwrap();
        let expr = sym.expr.as_ref().unwrap();
        EndVisitResult::changed(expr.clone())
    }

    fn start_visit_llvm(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // Judge whether to inline the function at the call site.
        let func = expr.get_app_func();
        if !func.is_var() {
            return EndVisitResult::unchanged(expr);
        }
        let func_name = &func.get_var().name;
        if func_name.is_local() {
            return EndVisitResult::unchanged(expr);
        }
        if !self
            .costs
            .costs
            .get(func_name)
            .unwrap()
            .inline_at_call_site()
        {
            return EndVisitResult::unchanged(expr);
        }
        let func_expr = self.symbols.get(func_name).unwrap().expr.as_ref().unwrap();
        let expr = expr.set_app_func(func_expr.clone());
        EndVisitResult::changed(expr)
    }

    fn start_visit_lam(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_let(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

    fn start_visit_match(
        &mut self,
        _expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
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

#[cfg(test)]
mod tests {
    use super::*;

    fn global(name: &str) -> FullName {
        FullName::from_strs(&["Main"], name)
    }

    /// An alias graph, given as the pairs `(alias, the name it refers to)`.
    fn alias_targets(pairs: &[(&str, &str)]) -> Map<FullName, FullName> {
        pairs
            .iter()
            .map(|(alias, target)| (global(alias), global(target)))
            .collect()
    }

    /// The names `aliases_with_an_end` finds in that graph, in alphabetical order.
    fn ends(pairs: &[(&str, &str)]) -> Vec<String> {
        let mut names = aliases_with_an_end(&alias_targets(pairs))
            .iter()
            .map(|name| name.name.clone())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn a_chain_of_aliases_ends_at_the_definition_it_names() {
        // `a` refers to `b`, `b` to `real`, and `real` is a definition of its own.
        assert_eq!(ends(&[("a", "b"), ("b", "real")]), vec!["a", "b"]);
    }

    #[test]
    fn a_cycle_of_aliases_has_no_end() {
        assert!(ends(&[("a", "b"), ("b", "c"), ("c", "a")]).is_empty());
        assert!(ends(&[("a", "b"), ("b", "a")]).is_empty());
    }

    #[test]
    fn a_chain_leading_into_a_cycle_has_no_end() {
        // `x` and `y` are outside the cycle and still arrive nowhere.
        assert!(ends(&[("x", "y"), ("y", "a"), ("a", "b"), ("b", "a")]).is_empty());
    }

    #[test]
    fn a_chain_that_ends_and_a_cycle_are_told_apart_in_one_pass() {
        // Whichever of the two the walk reaches first, both answers have to come out right.
        assert_eq!(
            ends(&[
                ("a", "b"),
                ("b", "real"),
                ("p", "q"),
                ("q", "r"),
                ("r", "p")
            ]),
            vec!["a", "b"]
        );
    }
}
