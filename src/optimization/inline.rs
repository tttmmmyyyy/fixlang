/*
Inlining optimization.
*/

use super::application_inlining;
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
use std::{mem, sync::Arc};

/// The size a body may reach and still be put where it is called, counted over the Fix expression
/// as `InlineCosts` counts it.
///
/// What the count weighs is what a copy of the body costs at a call site against the call it saves.
/// It measures the expression the optimizer holds; the instructions the body finally generates
/// exceed it, as reference counting and the bounds checks still to be inserted feed them too.
const INLINE_COST_THRESHOLD: i32 = 30;

/// How many times `run` rewrites the program before it stops asking for more.
///
/// Inlining reaches its result by rewriting until nothing changes, and a program whose global
/// definitions name each other in a cycle never gets there: the rewriting goes around the cycle
/// instead. A round doubles how far each name has been followed, so a chain of definitions L long
/// settles in about log2(L) rounds — the standard library and every program measured alongside it
/// settle within five, and a chain 500 long within eleven.
///
/// Ten covers what a program of any ordinary depth needs. It is also the reason to keep the number
/// small: the bodies of globals that call each other in a cycle double every round, so each round
/// left here is a term twice as large to finish with.
const MAX_ROUNDS: usize = 10;

/// Substitute the definitions of globals into the places that name them, round after round until the
/// program stops changing or `MAX_ROUNDS` rounds have passed. A global that nothing names, and that
/// is neither the entry point nor exported, is dropped along the way.
///
/// A primitive literal, and a global that is one name standing for another, go wherever the name
/// occurs; a lambda small enough (`INLINE_COST_THRESHOLD`) and one wrapping an inline-LLVM operation
/// go into the calls of it. A body that calls itself stays where it is.
pub fn run(prg: &mut Program) {
    let mut skip_symbols = Set::default();
    for _ in 0..MAX_ROUNDS {
        if !run_one(prg, &mut skip_symbols) {
            break;
        }
    }
}

/// One round of `run`: substitute into each symbol once and discard the symbols nothing names.
/// Returns whether the program changed.
///
/// # Arguments
/// * `stable_symbols` — the symbols with nothing left to substitute into them, carried from round to
///   round: a symbol listed here is passed through untouched, and a symbol this round leaves
///   unchanged joins it.
fn run_one(prg: &mut Program, stable_symbols: &mut Set<FullName>) -> bool {
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

fn calculate_inline_costs(prg: &Program) -> InlineCosts {
    let mut costs = InlineCosts::new();
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

        // If the expression is an alias to another global value, set as `is_alias`.
        if expr.is_var() {
            let var_name = &expr.get_var().name;
            assert!(var_name.is_global());
            costs.costs.get_mut(name).unwrap().is_alias = true;
        }
    }
    costs
}

/// What one symbol costs to inline, and the shapes of its expression that decide where it may
/// be inlined at all.
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
    // Is the expression primitive literal?
    is_primitive_literal: bool,
    // Is this expression an alias to another value?
    //
    // Example:
    // ```
    // x = y;
    // ```
    is_alias: bool,
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
            is_alias: false,
        }
    }

    /// Whether the symbol's expression may be substituted wherever the symbol is named, and not
    /// only where it is called.
    ///
    /// What qualifies is what costs nothing to hold in several places: a literal, a body that is
    /// one inline-LLVM operation, and a name that stands for another name.
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
        if !self.is_self_recursive && self.is_alias {
            return true;
        }
        return false;
        // NOTE
        // * Even values with simple types should not be inlined if the computation is complex.
        // * Values created using FFI_CALL are heavy.
        // * Boxed types and Strings also increase memory allocation when inlined, such as string literals.
    }

    /// Whether the symbol's expression may be substituted where the symbol is called.
    ///
    /// A body that calls itself is left alone, since substituting it leaves the call it makes to
    /// itself; so is `Std::fix`, whose defunctionalization matches the shape it is written in. What
    /// is left is judged by size, against `INLINE_COST_THRESHOLD`.
    fn inline_at_call_site(&self) -> bool {
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

/// What each symbol of a program costs to inline, and how often the program names it.
struct InlineCosts {
    /// One entry per symbol of the program walked, and one per global name those symbols use.
    costs: Map<FullName, InlineCost>,
}

impl InlineCosts {
    fn new() -> Self {
        InlineCosts {
            costs: Map::default(),
        }
    }

    /// Give `name` an entry of its own if it has none yet, with nothing counted and every flag
    /// false, for the walks to fill in as they meet the name.
    fn insert_cost_if_absent(&mut self, name: &FullName) {
        if !self.costs.contains_key(name) {
            self.costs.insert(name.clone(), InlineCost::new());
        }
    }

    /// The cost recorded for the symbol named `name`. `calculate_inline_costs` records one for
    /// every symbol of the program it walks, so every name of that program has one.
    fn get(&self, name: &FullName) -> &InlineCost {
        self.costs
            .get(name)
            .unwrap_or_else(|| panic!("no inline cost is recorded for `{}`", name.to_string()))
    }

    /// How many times the program names the symbol, counted over every expression the walk covered.
    fn get_call_count(&self, name: &FullName) -> usize {
        self.get(name).call_count
    }

    /// Take in what the walk of one symbol found: every global name that symbol uses has its call
    /// count raised, and the symbol itself gets the size, the self-reference and the lambda shape the
    /// walk measured.
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
