/*
Inlining optimization.
*/

use std::{mem, sync::Arc};

use crate::{
    ast::{
        name::FullName,
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::Map,
    ExprNode, InstantiatedSymbol, Program,
};

use super::beta_reduction;

pub const INLINE_ITERATION: usize = 1;
pub const INLINE_COST_THRESHOLD: usize = 30;

pub fn run(prg: &mut Program) {
    for _ in 0..INLINE_ITERATION {
        run_one(prg);
    }
}

pub fn run_one(prg: &mut Program) {
    let costs = calculate_inline_costs(prg);
    let symbols = mem::take(&mut prg.instantiated_symbols);
    let mut inliner = Inliner {
        costs: &costs,
        symbols: symbols.clone(),
    };
    let mut new_symbols: Map<FullName, InstantiatedSymbol> = Map::default();
    let root_value_names = prg.root_value_names();
    for (name, mut sym) in symbols {
        // If call count of the symbol is 0, and neither of entry point or exported, discard the symbol.
        if costs.get_call_count(&name) == 0 && !root_value_names.contains(&name) {
            continue;
        }

        // Traverse the expression and inline the symbol.
        let res = inliner.traverse(&sym.expr.as_ref().unwrap());

        // If some name is inlined, run beta reduction.
        if res.changed {
            sym.expr = Some(res.expr.calculate_free_vars());
            beta_reduction::run_on_symbol(&mut sym);
        }

        new_symbols.insert(name, sym);
    }

    prg.instantiated_symbols = new_symbols;
}

fn calculate_inline_costs(prg: &Program) -> InlineCosts {
    let mut costs = InlineCosts::new();
    for (name, sym) in &prg.instantiated_symbols {
        let mut cost_calculator = InlineCostCalculator::new(name.clone());
        cost_calculator.traverse(&sym.expr.as_ref().unwrap());
        costs.add_cost_calculation_result(name, cost_calculator);

        // If the expression is of the form `|x, y, ...| {llvm}`, then set as `is_llvm_lam`.
        let expr = sym.expr.as_ref().unwrap();
        let (_params, body) = expr.destructure_lam_sequence();
        let is_llvm_lam = body.is_llvm();
        costs.costs.get_mut(name).unwrap().is_llvm_lam = is_llvm_lam;
    }
    costs
}

// A struct to store information about the cost of inlining a symbol.
struct InlineCost {
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
}

impl InlineCost {
    // Returns true if the symbol can be inlined even at a non-call site.
    fn inline_at_non_call_site(&self) -> bool {
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        self.call_count == 1
        // TODO: we should allow constants to be inlined even if they are called more than once.
    }

    // Returns true if the symbol can be inlined at a call site.
    fn inline_at_call_site(&self) -> bool {
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        if !self.is_lambda {
            return false;
        }
        self.call_count * self.complexity <= INLINE_COST_THRESHOLD
    }
}

// The map from each symbol to the cost of inlining it.
struct InlineCosts {
    costs: Map<FullName, InlineCost>,
}

impl InlineCosts {
    fn new() -> Self {
        InlineCosts {
            costs: Map::default(),
        }
    }

    fn get_call_count(&self, name: &FullName) -> usize {
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
            // If calling a global symbol, increase the call count.
            if let Some(count) = self.call_count.get_mut(var_name) {
                *count += 1;
            } else {
                self.call_count.insert(var_name.clone(), 1);
            }

            // Add the complexity of the symbol.
            self.complexity += 1;

            // If it calls itself, set `is_call_self`.
            if var_name == &self.name {
                self.is_call_self = true;
            }
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
    symbols: Map<FullName, InstantiatedSymbol>,
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
