//! Put the body of a global where the program names it.
//!
//! Putting a body where the name of its global stood takes away a call the program would make, and
//! leaves the body among the expressions around that name, where the passes below read it.
//!
//! What a copy of a global costs, and where it may be put, is what `InlineCost` holds:
//! `may_be_inlined_at_non_call_site` answers for the places a global is named, and
//! `may_be_inlined_at_call_site` for the places it is called. A round of substitution leaves a
//! lambda applied where it stands, so a symbol the round rewrote is run to the fixpoint of
//! `let_elimination` and `application_inlining` (`inline_local`), which reduces that shape.
//!
//! Globals that name each other in a cycle never reach a program that stops changing, which is why
//! the rewriting is bounded: `MAX_ROUNDS` bounds the rounds, and `MAX_NODES_SUBSTITUTED_PER_ROUND`
//! bounds what one round adds to a symbol.

use crate::{
    ast::{
        expr::ExprNode,
        name::FullName,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, VisitState},
        type_env::TypeEnv,
    },
    misc::{map_on_compiler_threads, Map, Set},
    optimization::{
        inline_local, let_elimination::create_global_lambda_to_arity_map, uncurry::is_std_fix,
    },
    rc_ir::locality::ExtShape,
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
/// `inline_local` runs after a round of substitution and reduces what it leaves, so the symbol ends
/// the round at what that pass makes of the nodes this bound let in.
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
/// It is set above what a round of a corpus program asks for. Measured over LangArena's fifty
/// programs and the standard library, with the bound lifted so that every substitution is taken,
/// the largest round asks for 5,166 nodes and the 99th percentile for 657, over 3,516
/// symbol-rounds. This leaves the largest of them a tenth of the bound.
///
/// A program that asks for more than this in one round takes the rest in the rounds that follow,
/// and reaches the same program where `MAX_ROUNDS` rounds are enough for it. A function of one
/// hundred and sixty branches, each building a string the way LangArena's
/// `Calculator::_right_hand_side` does, compiles to the same symbols as an unbounded compiler; so
/// does one of eighty, which at a bound of ten thousand kept fifteen closures
/// `closure_specialization` folds away here.
const MAX_NODES_SUBSTITUTED_PER_ROUND: usize = 50000;

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
/// name occurs; a lambda small enough (`INLINE_COST_THRESHOLD`) and one wrapping a builtin
/// operation go into the calls of it. A body that calls itself stays where it is.
pub fn run(prg: &mut Program, num_threads: usize) {
    let mut stable_symbols = Set::default();
    for _ in 0..MAX_ROUNDS {
        if !run_one(prg, &mut stable_symbols, num_threads) {
            break;
        }
    }
}

/// One round of `run`: substitute into each symbol once and discard the symbols nothing names.
/// Returns whether the program changed.
///
/// Every symbol reads the bodies of the others as they were when the round began, so the symbols
/// are rewritten independently of one another, on up to `num_threads` threads.
///
/// # Arguments
/// * `stable_symbols` — the symbols with nothing left to substitute into them, carried from round to
///   round: a symbol listed here is passed through untouched, and a symbol this round leaves
///   unchanged joins it.
fn run_one(prg: &mut Program, stable_symbols: &mut Set<FullName>, num_threads: usize) -> bool {
    let costs = calculate_inline_costs(prg, num_threads);
    let symbols = mem::take(&mut prg.symbols);
    let global_lambda_to_arity = create_global_lambda_to_arity_map(&symbols);
    let root_value_names = prg.root_value_names();
    let round = InlineRound {
        costs: &costs,
        symbols: &symbols,
        global_lambda_to_arity: &global_lambda_to_arity,
        root_value_names: &root_value_names,
        stable_symbols,
    };
    let outcomes = map_on_compiler_threads(symbols.iter().collect(), num_threads, |(name, sym)| {
        (name, round.run_on_symbol(name, sym))
    });

    let mut changed = false;
    let mut new_symbols: Map<FullName, Symbol> = Map::default();
    for (name, outcome) in outcomes {
        match outcome {
            SymbolOutcome::Discarded => changed = true,
            SymbolOutcome::Kept {
                sym,
                changed: sym_changed,
                stable,
            } => {
                changed |= sym_changed;
                if stable {
                    stable_symbols.insert(name.clone());
                }
                new_symbols.insert(name.clone(), sym);
            }
        }
    }
    prg.symbols = new_symbols;

    changed
}

/// What one round of `run` reads while it rewrites each symbol: the program as the round found it.
struct InlineRound<'r> {
    /// What each global of the program costs to inline, and where it may be inlined.
    costs: &'r InlineCosts,
    /// The symbols of the program as the round found them, holding the bodies to put at the names.
    symbols: &'r Map<FullName, Symbol>,
    /// How many parameters each global lambda takes, for `inline_local`.
    global_lambda_to_arity: &'r Map<FullName, usize>,
    /// The entry point and the exported values, which are kept although nothing names them.
    root_value_names: &'r [FullName],
    /// The symbols the rounds before found nothing left to substitute into.
    stable_symbols: &'r Set<FullName>,
}

/// What one round of `run` did with one symbol.
enum SymbolOutcome {
    /// Nothing in the program names the symbol, and it is neither the entry point nor an exported
    /// value, so it is dropped.
    Discarded,
    /// The symbol stays in the program.
    Kept {
        /// The symbol as the round left it.
        sym: Symbol,
        /// Whether the round substituted anything into it.
        changed: bool,
        /// Whether it has reached its end state, so that the rounds after pass it through.
        stable: bool,
    },
}

impl<'r> InlineRound<'r> {
    /// Substitute into the symbol `name` once, or discard it if nothing names it.
    fn run_on_symbol(&self, name: &FullName, sym: &Symbol) -> SymbolOutcome {
        // If nothing in the program names the symbol, and it is neither the entry point nor an
        // exported value, discard it.
        if self.costs.get_use_count(name) == 0 && !self.root_value_names.contains(name) {
            return SymbolOutcome::Discarded;
        }

        let mut sym = sym.clone();

        // If the symbol is known to be stable, skip it.
        if self.stable_symbols.contains(name) {
            return SymbolOutcome::Kept {
                sym,
                changed: false,
                stable: true,
            };
        }

        // A symbol whose expression names nothing has nothing to substitute into it.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            return SymbolOutcome::Kept {
                sym,
                changed: false,
                stable: true,
            };
        }

        let mut inliner = Inliner {
            costs: self.costs,
            symbols: self.symbols,
            budget: 0,
            refused_for_budget: false,
        };
        let res = inliner.substitute_into(&sym.expr.as_ref().unwrap());

        let mut stable = false;
        if res.changed {
            sym.expr = Some(res.expr);
            inline_local::run_on_symbol(&mut sym, self.global_lambda_to_arity);
        } else if !inliner.refused_for_budget {
            // A round that substituted nothing and refused nothing has reached the symbol's end
            // state. A symbol whose round refused a body it could not afford is asked again next
            // round: the body may come back smaller, since `inline_local` reduces what the
            // substitution left in every symbol the round changed.
            stable = true;
        }

        // The round may have left an expression that names nothing, which is the same end state.
        if sym.expr.as_ref().unwrap().free_vars().is_empty() {
            stable = true;
        }

        SymbolOutcome::Kept {
            sym,
            changed: res.changed,
            stable,
        }
    }
}

/// Measure every symbol of the program: how large its expression is, how many times the program
/// names it, and the shapes of that expression which decide where it may be inlined. The symbols
/// are measured on up to `num_threads` threads.
fn calculate_inline_costs(prg: &Program, num_threads: usize) -> InlineCosts {
    let type_env = prg.type_env();
    let measures =
        map_on_compiler_threads(prg.symbols.iter().collect(), num_threads, |(name, sym)| {
            measure_symbol(name, sym, &type_env)
        });
    let mut costs = InlineCosts::new();
    for (name, measure) in measures {
        costs.add_symbol_measure(name, measure);
    }
    costs
}

/// What the walk of one symbol found.
struct SymbolMeasure {
    /// What the symbol itself costs, with `use_count` left at 0: how often the symbol is named is
    /// counted over the other symbols.
    cost: InlineCost,
    /// For each global name, how many times the symbol names it.
    use_count: Map<FullName, usize>,
}

/// Measure the symbol `name`, whose definition is `sym`: what a copy of its expression costs, the
/// shapes of that expression, and the global names it uses.
fn measure_symbol(name: &FullName, sym: &Symbol, type_env: &TypeEnv) -> (FullName, SymbolMeasure) {
    let expr = sym.expr.as_ref().unwrap();

    let mut cost_calculator = InlineCostCalculator::new(name.clone());
    cost_calculator.traverse(expr);

    let mut cost = InlineCost::new();
    cost.complexity = cost_calculator.complexity;
    cost.is_self_recursive = cost_calculator.is_self_recursive;
    cost.is_lambda = cost_calculator.is_lambda;

    // Count what a copy of the expression costs the compiler, which is every node of it,
    // including the kinds `complexity` counts as nothing.
    cost.node_count = expr.node_count();

    // An expression that takes no parameter is the operation itself, and a copy of it costs what
    // the operation costs, which `is_free_to_duplicate` answers.
    let (params, body) = expr.destructure_lam_sequence();
    cost.is_builtin_lam = !params.is_empty() && body.is_builtin();

    if expr.is_builtin() {
        let op = &expr.get_builtin().op;
        let is_free_to_duplicate = op.is_free_to_duplicate();
        // An operation whose result holds a boxed part allocates that part, so a copy of it
        // allocates once more, unless every boxed part is a constant in the program's data,
        // which the operation declares non-local. So an operation declared free to duplicate
        // answers with a type unboxed throughout, or declares its whole result non-local.
        assert!(
            !is_free_to_duplicate
                || sym.ty.is_fully_unboxed(type_env)
                || op.result_locality(&sym.ty, &[], type_env)
                    == ExtShape::always(&sym.ty, type_env),
            "the builtin operation `{}` declares a copy of itself free while it answers \
             with `{}`, which holds a boxed part it may allocate",
            op.name(),
            sym.ty.to_string()
        );
        cost.is_free_to_duplicate = is_free_to_duplicate;
    }

    cost.is_std_fix = is_std_fix(name);

    if expr.is_var() {
        assert!(expr.get_var().name.is_global());
        cost.is_alias = true;
    }

    (
        name.clone(),
        SymbolMeasure {
            cost,
            use_count: cost_calculator.use_count,
        },
    )
}

/// What one symbol costs to inline, and the shapes of its expression that decide where it may
/// be inlined at all.
struct InlineCost {
    /// The number of times the program names the symbol.
    use_count: usize,
    /// What a copy of the symbol's expression costs the program that runs: one for each node that
    /// generates code. A local variable, a type annotation, an `eval`, a `let` whose pattern is one
    /// local name, and a `match` that only renames a local count nothing.
    complexity: usize,
    /// The number of nodes the expression holds, which is what a copy of it costs the compiler.
    /// `complexity` answers what a copy costs the program that runs, where a node that generates no
    /// code counts as nothing.
    node_count: usize,
    /// Does the symbol's expression name the symbol itself?
    is_self_recursive: bool,
    /// Is the top-level construct a lambda expression?
    is_lambda: bool,
    /// Is the expression of the form `|x, y, ...| {builtin}`?
    is_builtin_lam: bool,
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
            is_builtin_lam: false,
            is_free_to_duplicate: false,
            is_std_fix: false,
            is_alias: false,
        }
    }

    /// Whether the symbol's expression may be substituted wherever the symbol is named, and not
    /// only where it is called.
    ///
    /// What qualifies is what costs nothing to hold in several places: an operation a copy of
    /// which costs no more than itself, a lambda whose body is one builtin operation, and a name
    /// that stands for another name.
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
        if self.is_builtin_lam {
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
        if self.is_builtin_lam {
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

    /// Take in what the walk of the symbol `name` found: every global name that symbol uses has its
    /// use count raised, and the symbol itself gets the cost the walk measured.
    fn add_symbol_measure(&mut self, name: FullName, measure: SymbolMeasure) {
        for (used, count) in measure.use_count {
            self.get_or_insert(used).use_count += count;
        }
        let cost = self.get_or_insert(name);
        *cost = InlineCost {
            use_count: cost.use_count,
            ..measure.cost
        };
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
    // The measuring is done as the walk ends a node. Each `end_visit_*` also records in `is_lambda`
    // whether the node it ends is a lambda, which answers for the whole expression once the walk
    // has ended its top-level node.

    /// Counts a global name as one unit of `complexity` and as one use of the symbol it names. A
    /// local name counts nothing: it generates no code, and the uses counted are uses of globals.
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

    /// Counts the builtin operation as one unit of `complexity`, and each global name free in it as
    /// one use of the symbol it names.
    fn end_visit_builtin(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = false;
        for free_name in expr.free_vars() {
            if free_name.is_global() {
                self.on_find_usage_of_global_name(&free_name);
            }
        }
        EndVisitResult::unchanged(expr)
    }

    /// Counts the application as one unit of `complexity`.
    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the lambda as one unit of `complexity`.
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = true;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the `let` as one unit of `complexity`, except where its pattern is one local name:
    /// such a `let` binds the name to the value and generates no code of its own.
    fn end_visit_let(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // A `let` of the form `let {local_var0} = (...) in (...)` counts nothing.
        self.complexity += 1;
        let pat = expr.get_let_pat();
        if pat.is_var() && pat.get_var().name.is_local() {
            self.complexity -= 1;
        }
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the `if` as one unit of `complexity`.
    fn end_visit_if(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        self.complexity += 1;
        self.is_lambda = false;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the `match` as one unit of `complexity`, except where it only renames a local, which
    /// generates no code of its own.
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

    /// A type annotation generates no code, so it counts nothing.
    fn end_visit_tyanno(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        self.is_lambda = false;

        // A type annotation counts nothing.
        EndVisitResult::unchanged(expr)
    }

    /// Counts the struct construction as one unit of `complexity`.
    fn end_visit_make_struct(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        self.is_lambda = false;
        self.complexity += 1;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the array literal as one unit of `complexity`.
    fn end_visit_array_lit(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        self.is_lambda = false;
        self.complexity += 1;
        EndVisitResult::unchanged(expr)
    }

    /// Counts the FFI call as one unit of `complexity`.
    fn end_visit_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> EndVisitResult {
        self.is_lambda = false;
        self.complexity += 1;
        EndVisitResult::unchanged(expr)
    }

    /// An `eval` generates no code of its own, so it counts nothing. What it holds is counted as
    /// the walk reaches it.
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
    symbols: &'c Map<FullName, Symbol>,
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
    /// Replaces a global name with the expression of the symbol it names, where that symbol may be
    /// inlined wherever it is named and the round's budget covers a copy of it.
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

    /// Replaces the function a call names with the expression of that global, where it may be
    /// inlined at a call site and the round's budget covers a copy of it.
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
}
