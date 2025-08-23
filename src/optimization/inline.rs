/*
Inlining optimization.
*/

use std::{mem, sync::Arc};

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
    stopwatch::StopWatch,
    ExprNode, Program, Symbol,
};

use super::{beta_reduction, remove_renaming};

pub const INLINE_COST_THRESHOLD: i32 = 30;

pub fn run(prg: &mut Program, show_build_times: bool) {
    // Calculate free variables of all symbols.
    for (_name, sym) in &mut prg.symbols {
        sym.expr = Some(sym.expr.as_ref().unwrap().calculate_free_vars());
    }

    let mut skip_symbols = Set::default();
    while run_one(prg, &mut skip_symbols) {}
    let _sw = StopWatch::new("inline::run remove_renaming", show_build_times);
    remove_renaming::run(prg);
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
            // If inlining was done, run beta reduction.
            changed = true;
            sym.expr = Some(res.expr.calculate_free_vars());
            beta_reduction::run_on_symbol(&mut sym);
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
    for (name, sym) in &prg.symbols {
        let mut cost_calculator = InlineCostCalculator::new(name.clone());
        cost_calculator.traverse(&sym.expr.as_ref().unwrap());
        costs.add_cost_calculation_result(name, cost_calculator);

        // If the expression is of the form `|x, y, ...| {llvm}`, then set as `is_llvm_lam`.
        let expr = sym.expr.as_ref().unwrap();
        let (_params, body) = expr.destructure_lam_sequence();
        let is_llvm_lam = body.is_llvm();
        costs.costs.get_mut(name).unwrap().is_llvm_lam = is_llvm_lam;

        // If the expression is a primitive literal, set as `is_primitive_literal`.
        if expr.is_llvm() {
            let is_primitive_literal = expr.get_llvm().generator.is_primitve_literal();
            costs.costs.get_mut(name).unwrap().is_primitive_literal = is_primitive_literal;
        }
    }
    costs
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
}

impl InlineCost {
    // Returns true if the symbol can be inlined even at a non-call site.
    fn inline_at_non_call_site(&self) -> bool {
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
        return false;
        // NOTE
        // * Even values with simple types should not be inlined if the computation is complex.
        // * Values created using FFI_CALL are heavy.
        // * Boxed types and Strings also increase memory allocation when inlined, such as string literals.
    }

    // Returns true if the symbol can be inlined at a call site.
    pub fn inline_at_call_site(&self) -> bool {
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        if !self.is_lambda {
            return false;
        }
        (self.call_count as i32 - 1) * self.complexity as i32 <= INLINE_COST_THRESHOLD
        // If call count is 1, then inline always regardless of complexity.
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

    pub fn get_call_count(&self, name: &FullName) -> usize {
        self.costs.get(name).map_or(0, |c| c.call_count)
    }

    fn add_cost_calculation_result(&mut self, name: &FullName, cost: InlineCostCalculator) {
        // Add call counts.
        for (sym, count) in cost.call_count {
            if let Some(c) = self.costs.get_mut(&sym) {
                c.call_count += count;
            } else {
                self.costs.insert(
                    sym,
                    InlineCost {
                        call_count: count,
                        complexity: 0,
                        is_self_recursive: false,
                        is_lambda: false,
                        is_llvm_lam: false,
                        is_primitive_literal: false,
                    },
                );
            }
        }

        // Set other fields.
        if let Some(c) = self.costs.get_mut(name) {
            c.complexity = cost.complexity;
            c.is_self_recursive = cost.is_call_self;
            c.is_lambda = cost.is_lambda;
        } else {
            self.costs.insert(
                name.clone(),
                InlineCost {
                    call_count: 0,
                    complexity: cost.complexity,
                    is_self_recursive: cost.is_call_self,
                    is_lambda: cost.is_lambda,
                    is_primitive_literal: false,
                    is_llvm_lam: false,
                },
            );
        }
    }
}

struct InlineCostCalculator {
    // The name of the symbol.
    name: FullName,
    // For each global symbol, the count of calls.
    call_count: Map<FullName, usize>,
    // The cost of the symbol.
    complexity: usize,
    // Is the symbol calling itself?
    is_call_self: bool,
    // Is the top-level construct a lambda expression?
    is_lambda: bool,
}

impl InlineCostCalculator {
    fn new(name: FullName) -> Self {
        InlineCostCalculator {
            name,
            call_count: Map::default(),
            complexity: 0,
            is_call_self: false,
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
            self.is_call_self = true;
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
}
