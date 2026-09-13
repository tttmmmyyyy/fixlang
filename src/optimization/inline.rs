/*
Inlining optimization.
*/

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
    },
    misc::{Map, Set},
    optimization::{application_inlining, uncurry::is_std_fix},
};
use std::{mem, sync::Arc};

/// The `complexity` a body may reach and still be put where it is called.
///
/// What `complexity` weighs is what a copy of the body costs at a call site against the call it
/// saves.
/// It measures the expression the optimizer holds; the instructions the body finally generates
/// exceed it, as reference counting and the bounds checks still to be inserted feed them too.
const INLINE_COST_THRESHOLD: i32 = 30;

/// The nodes one round of substitution may add to a symbol's expression.
///
/// `application_inlining` runs after a round of substitution and rewrites what it leaves, so the
/// symbol ends the round at what that pass makes of the nodes this bound let in.
///
/// The bound is counted in `node_count`, where `INLINE_COST_THRESHOLD` is counted in `complexity`,
/// and the two counts answer different questions. `complexity` weighs what a copy of a body costs
/// the program that runs, so a `let` binding one name to another counts nothing. `node_count`
/// weighs what a copy costs the compiler that holds it, so every node counts, however little code
/// it generates.
///
/// Globals that name each other in a cycle are why the bound is counted in `node_count`. None of
/// them calls itself, so `is_self_recursive` never stops the substitution, and each round puts the
/// cycle into each member again. What accumulates is the renaming a substitution leaves behind —
/// `let x = p; let p = x;` per turn — which `complexity` counts as nothing, so no ceiling expressed
/// in it can ever be reached. A ring of three globals doubles each of them per round — 6, 10, 18,
/// 34, 66 nodes and on — while `complexity` reads 3 throughout, and a ring whose members carry two
/// thousand renamings apiece exhausts the compiler's stack and aborts the build.
///
/// The bound is on the growth of one round, so a symbol that has already grown large still takes
/// the substitutions of the rounds that follow, a one-node alias among them, which
/// `split_struct_args` and `closure_specialization` then read. A cycle therefore grows by at most
/// this many nodes in each of `MAX_ROUNDS` rounds.
///
/// It is set where no symbol in a corpus program comes near it: across LangArena's fifty
/// programs and the standard library the largest holds 4,130 nodes in total, and the 99th
/// percentile 1,164, so no program whose inlining stops on its own reaches it.
///
/// A symbol that names more globals than `MAX_ROUNDS` rounds of this bound can pay for keeps the
/// rest as calls. An arithmetic operator is three nodes, so that is around thirty thousand
/// operators written into one symbol.
const MAX_NODES_SUBSTITUTED_PER_ROUND: usize = 10000;

/// How many times `run` rewrites the program before it stops asking for more.
///
/// Inlining reaches its result by rewriting until nothing changes, and a program whose global
/// definitions name each other in a cycle never gets there: the rewriting goes around the cycle
/// instead. A round doubles how far each name has been followed, so a chain of definitions L long
/// settles in about log2(L) rounds — the standard library and every program measured alongside it
/// settle within five, and a chain 500 long within eleven.
///
/// Ten covers what a program of any ordinary depth needs. `MAX_NODES_SUBSTITUTED_PER_ROUND` bounds
/// what a cycle adds in each of those rounds.
const MAX_ROUNDS: usize = 10;

/// Substitute the definitions of globals into the places that name them, round after round until the
/// program stops changing or `MAX_ROUNDS` rounds have passed. A global that nothing names, and that
/// is neither the entry point nor exported, is dropped along the way.
///
/// A global whose body is an operation a copy of which costs no more than the operation itself
/// (`is_free_to_duplicate`), and a global that is one name standing for another, go wherever the
/// name occurs; a lambda small enough (`INLINE_COST_THRESHOLD`) and one wrapping an inline-LLVM
/// operation go into the calls of it. A body that calls itself stays where it is.
pub fn run(prg: &mut Program) {
    let mut stable_symbols = Set::default();
    for _ in 0..MAX_ROUNDS {
        if !run_one(prg, &mut stable_symbols) {
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
        budget: 0,
        refused_for_budget: false,
    };
    let mut new_symbols: Map<FullName, Symbol> = Map::default();
    let root_value_names = prg.root_value_names();

    for (name, mut sym) in symbols {
        // If nothing in the program names the symbol, and it is neither the entry point nor an
        // exported value, discard it.
        if costs.get_use_count(&name) == 0 && !root_value_names.contains(&name) {
            changed = true;
            continue;
        }

        // If the symbol is known to be stable, skip it.
        if stable_symbols.contains(&name) {
            new_symbols.insert(name, sym);
            continue;
        }

        // A symbol whose expression names nothing has nothing to substitute into it.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            stable_symbols.insert(name.clone());
            new_symbols.insert(name.clone(), sym);
            continue;
        }

        let res = inliner.substitute_into(&sym.expr.as_ref().unwrap());

        if res.changed {
            changed = true;
            sym.expr = Some(res.expr);
            application_inlining::run_on_symbol(&mut sym);
        } else if !inliner.refused_for_budget {
            // A round that substituted nothing and refused nothing has reached the symbol's end
            // state. A symbol whose round refused a body it could not afford is asked again next
            // round: the body may come back smaller, since `application_inlining` rewrites what the
            // substitution left in every symbol the round changed.
            stable_symbols.insert(name.clone());
        }

        // The round may have left an expression that names nothing, which is the same end state.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            stable_symbols.insert(name.clone());
        }

        new_symbols.insert(name, sym);
    }

    prg.symbols = new_symbols;

    changed
}

/// Measure every symbol of the program: how large its expression is, how many times the program
/// names it, and the shapes of that expression which decide where it may be inlined.
fn calculate_inline_costs(prg: &Program) -> InlineCosts {
    let mut costs = InlineCosts::new();
    let type_env = prg.type_env();
    for (name, sym) in &prg.symbols {
        let expr = sym.expr.as_ref().unwrap();

        let mut cost_calculator = InlineCostCalculator::new(name.clone());
        cost_calculator.traverse(expr);
        costs.add_cost_calculation_result(cost_calculator);

        let cost = costs.get_mut(name);

        // Count what a copy of the expression costs the compiler, which is every node of it,
        // including the kinds `complexity` counts as nothing.
        cost.node_count = expr.node_count();

        // An expression that takes no parameter is the operation itself, and a copy of it costs what
        // the operation costs, which `is_free_to_duplicate` answers.
        let (params, body) = expr.destructure_lam_sequence();
        cost.is_llvm_lam = !params.is_empty() && body.is_llvm();

        if expr.is_llvm() {
            let generator = &expr.get_llvm().generator;
            let is_free_to_duplicate = generator.is_free_to_duplicate();
            // An operation whose result holds a boxed part allocates that part, so a copy of it
            // allocates once more. The declaration and the operation agree only where the type of
            // what it answers with is unboxed throughout.
            assert!(
                !is_free_to_duplicate || sym.ty.is_fully_unboxed(&type_env),
                "the inline-LLVM operation `{}` declares a copy of itself free while it answers \
                 with `{}`, which holds a boxed part",
                generator.name(),
                sym.ty.to_string()
            );
            cost.is_free_to_duplicate = is_free_to_duplicate;
        }

        cost.is_std_fix = is_std_fix(name);

        if expr.is_var() {
            assert!(expr.get_var().name.is_global());
            cost.is_alias = true;
        }
    }
    costs
}

/// What one symbol costs to inline, and the shapes of its expression that decide where it may
/// be inlined at all.
struct InlineCost {
    /// The number of times the program names the symbol.
    use_count: usize,
    /// What a copy of the symbol's expression costs the program that runs: one for each node that
    /// generates code. A local variable, a type annotation, an `eval`, and a `let` or a `match`
    /// that only renames a local count nothing.
    complexity: usize,
    /// The number of nodes the expression holds, which is what a copy of it costs the compiler.
    /// `complexity` answers what a copy costs the program that runs, where a node that generates no
    /// code counts as nothing.
    node_count: usize,
    /// Does the symbol's expression name the symbol itself?
    is_self_recursive: bool,
    /// Is the top-level construct a lambda expression?
    is_lambda: bool,
    /// Is the expression of the form `|x, y, ...| {llvm}`?
    is_llvm_lam: bool,
    /// Does a copy of the expression cost no more than the expression itself?
    is_free_to_duplicate: bool,
    /// Is this expression an alias to another value, as in `x = y;`?
    is_alias: bool,
    /// Is the expression instantiated by `Std::fix`?
    is_std_fix: bool,
}

impl InlineCost {
    /// A cost with nothing counted and every flag false, for the walks to fill in.
    fn new() -> Self {
        InlineCost {
            use_count: 0,
            complexity: 0,
            node_count: 0,
            is_self_recursive: false,
            is_lambda: false,
            is_llvm_lam: false,
            is_free_to_duplicate: false,
            is_std_fix: false,
            is_alias: false,
        }
    }

    /// Whether the symbol's expression may be substituted wherever the symbol is named, and not
    /// only where it is called.
    ///
    /// What qualifies is what costs nothing to hold in several places: an operation a copy of
    /// which costs no more than itself, a lambda whose body is one inline-LLVM operation, and a
    /// name that stands for another name.
    fn may_be_inlined_at_non_call_site(&self) -> bool {
        if self.is_std_fix {
            return false;
        }
        if self.is_free_to_duplicate {
            // TODO: Let an expression of primitive type whose value is constant qualify here too.
            // What a type is says nothing about what the expression computing it costs: a value an
            // `FFI_CALL` produces has a primitive type and is heavy.
            return true;
        }
        if self.is_self_recursive {
            return false;
        }
        if self.is_llvm_lam {
            return true;
        }
        if self.is_alias {
            return true;
        }
        return false;
    }

    /// Whether the symbol's expression may be substituted where the symbol is called.
    ///
    /// A body that calls itself is left alone, since substituting it leaves the call it makes to
    /// itself; so is `Std::fix`, whose defunctionalization matches the shape it is written in. What
    /// is left is judged by size, against `INLINE_COST_THRESHOLD`.
    fn may_be_inlined_at_call_site(&self) -> bool {
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
    /// A table holding no cost yet.
    fn new() -> Self {
        InlineCosts {
            costs: Map::default(),
        }
    }

    /// The cost recorded for the symbol named `name`. `calculate_inline_costs` records one for
    /// every symbol of the program it walks, so every name of that program has one.
    fn get(&self, name: &FullName) -> &InlineCost {
        self.costs
            .get(name)
            .unwrap_or_else(|| Self::panic_no_cost_recorded(name))
    }

    /// The cost recorded for the symbol named `name`, to write to. Every name the program defines
    /// has one, as `get` says.
    fn get_mut(&mut self, name: &FullName) -> &mut InlineCost {
        self.costs
            .get_mut(name)
            .unwrap_or_else(|| Self::panic_no_cost_recorded(name))
    }

    /// Fail, naming the symbol whose cost was asked for and not found.
    fn panic_no_cost_recorded(name: &FullName) -> ! {
        panic!("no inline cost is recorded for `{}`", name.to_string())
    }

    /// The cost recorded for the symbol named `name`, given an entry of its own with nothing
    /// counted and every flag false if it has none yet, for the walks to fill in as they meet the
    /// name.
    fn get_or_insert(&mut self, name: FullName) -> &mut InlineCost {
        self.costs.entry(name).or_insert_with(InlineCost::new)
    }

    /// How many times the program names the symbol, counted over every expression the walk covered.
    fn get_use_count(&self, name: &FullName) -> usize {
        self.get(name).use_count
    }

    /// Take in what the walk of one symbol found: every global name that symbol uses has its use
    /// count raised, and the symbol itself gets the size, the self-reference and the lambda shape the
    /// walk measured.
    fn add_cost_calculation_result(&mut self, calculator: InlineCostCalculator) {
        // Raise the use count of each global name the walked symbol uses.
        for (sym, count) in calculator.use_count {
            self.get_or_insert(sym).use_count += count;
        }

        // Set other fields for the symbol itself that `InlineCostCalculator` has traversed.
        let inline_cost = self.get_or_insert(calculator.name);
        inline_cost.complexity = calculator.complexity;
        inline_cost.is_self_recursive = calculator.is_self_recursive;
        inline_cost.is_lambda = calculator.is_lambda;
    }
}

/// Walks the expression of one symbol and measures it: how large it is, which global names it
/// uses and how often, whether it names the symbol itself, and whether its top-level construct is
/// a lambda.
struct InlineCostCalculator {
    /// The name of the symbol whose expression is walked.
    name: FullName,
    /// For each global name, how many times the symbol names it.
    use_count: Map<FullName, usize>,
    /// The size of the part of the expression walked so far: one for each node that generates
    /// code.
    complexity: usize,
    /// Does the symbol name itself?
    is_self_recursive: bool,
    /// Is the construct visited last a lambda expression? The walk ends at the top-level
    /// construct, so this answers for that one.
    is_lambda: bool,
}

impl InlineCostCalculator {
    /// A calculator that has measured nothing, for the symbol named `name`.
    fn new(name: FullName) -> Self {
        InlineCostCalculator {
            name,
            use_count: Map::default(),
            complexity: 0,
            is_self_recursive: false,
            is_lambda: false,
        }
    }

    /// Record one use of the global name `used_name` by the symbol being walked, and note a
    /// symbol that names itself.
    fn on_find_usage_of_global_name(&mut self, used_name: &FullName) {
        // Count one use of the global symbol.
        assert!(used_name.is_global());
        *self.use_count.entry(used_name.clone()).or_insert(0) += 1;

        // If the symbol names itself, set `is_self_recursive`.
        if used_name == &self.name {
            self.is_self_recursive = true;
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
        // A `let` of the form `let {local_var0} = {local_var1} in (...)` counts nothing.
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

        // A `match` of the form `match {local_var0} { {local_var1} -> (...) }` counts nothing.
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

        // A type annotation counts nothing.
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

/// Walks an expression and puts the body of a global at each place the expression names it, where
/// the cost of that global allows.
struct Inliner<'c> {
    /// What each global of the program costs to inline, and where it may be inlined.
    costs: &'c InlineCosts,
    /// The symbols of the program, holding the bodies to put at the names.
    symbols: Map<FullName, Symbol>,
    /// How many nodes the symbol being traversed may still gain this round. `substitute_into` sets
    /// it to `MAX_NODES_SUBSTITUTED_PER_ROUND` before each symbol, every substitution spends the
    /// nodes of the body it copies, and a body larger than what is left is refused.
    budget: usize,
    /// Whether the traversal of the symbol refused a substitution that `budget` did not cover.
    refused_for_budget: bool,
}

impl<'c> Inliner<'c> {
    /// Substitute into `expr` the body of each global it names, where the cost of that global
    /// allows, letting `expr` gain at most `MAX_NODES_SUBSTITUTED_PER_ROUND` nodes.
    fn substitute_into(&mut self, expr: &Arc<ExprNode>) -> EndVisitResult {
        self.budget = MAX_NODES_SUBSTITUTED_PER_ROUND;
        self.refused_for_budget = false;
        self.traverse(expr)
    }

    /// Whether a copy of `name`'s expression fits in the `budget` left for the symbol being
    /// traversed this round, taking its nodes out of `budget` where it does.
    fn take_budget_for(&mut self, name: &FullName) -> bool {
        let node_count = self.costs.get(name).node_count;
        assert!(
            node_count > 0,
            "the body of `{}` is about to be copied while its node count reads 0; \
             `calculate_inline_costs` counts the nodes of every symbol the program defines",
            name.to_string()
        );
        let Some(left) = self.budget.checked_sub(node_count) else {
            self.refused_for_budget = true;
            return false;
        };
        self.budget = left;
        true
    }
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

        if !self.costs.get(var_name).may_be_inlined_at_non_call_site() {
            return EndVisitResult::unchanged(expr);
        }
        if !self.take_budget_for(var_name) {
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
        if !self.costs.get(func_name).may_be_inlined_at_call_site() {
            return EndVisitResult::unchanged(expr);
        }
        if !self.take_budget_for(func_name) {
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
