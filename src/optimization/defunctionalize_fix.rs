// Defunctionalize `Std::fix` into a directly self-recursive global function.
//
// The `fix` combinator builds a self-referential closure whose recursive self-call dispatches
// through a function pointer. LLVM's tail-call elimination folds only *direct* self-recursion into a
// loop, so an indirect self-call whose return value is passed via `sret` (a value of four or more
// scalar leaves) keeps a real `call` and grows the stack by one frame per iteration.
//
// This pass rewrites `fix(|self| body)` into a call to a fresh global function `G`:
//
//     G : FixCap -> a -> b
//     G = |cap| let FixCap { .. } = cap; body[self := G(cap)]
//
// After uncurrying, each `self(args)` in the body is a saturated application of the global `G`, i.e.
// a direct self-call, which LLVM turns into a loop regardless of the return-value ABI. The captured
// environment is threaded as one struct value `cap`, so a self-call never re-mentions an individual
// captured name and the rewrite is insensitive to shadowing of those names. `self` used as a bare
// value becomes the partial application `G(cap)`, i.e. a closure, preserving non-tail uses.
//
// The `fix` argument is resolved to the lambda it denotes: written inline, bound to a local
// `let name = |..| ..`, or a global function whose definition is a lambda. Only a bare lambda
// qualifies — a `let name = let x = ..; |..| ..` is left as a closure fix, so a heavy initializer is
// never duplicated across `fix` sites, matching `let_elimination`'s reason for inlining only bare
// lambdas. The rewrite replaces only the `fix(..)` occurrence, leaving the argument's own binding for
// its other uses. A local lambda is lifted per `fix` site — the duplication is cheap next to the loop
// it enables — while a global is lifted once and shared, which also makes a global that fixes itself
// terminate.

use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_var, var_local, Expr, ExprNode,
        },
        name::FullName,
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{type_fun, TyCon, TyConInfo, TypeNode},
    },
    misc::{Map, Set},
    optimization::{
        capture_struct::{fresh_global_name, CaptureStruct},
        let_elimination,
        rename::substitute_free_name,
        uncurry::{internalize_let_to_var_at_head, is_std_fix},
        unique_local_names,
    },
    tool::stopwatch::StopWatch,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// A global function's name paired with its defining lambda, for resolving `fix(GlobalName)`.
type GlobalLambdas = Rc<Map<FullName, Arc<ExprNode>>>;

// The shared `fix` reference built for each global fixed via `fix`: `GlobalName -> G(cap)` of type
// `a -> b`. One entry per global (deduplicated), so a global that fixes itself defunctionalizes to a
// single recursive `G` instead of lifting without end.
type GlobalFixRefs = Rc<RefCell<Map<FullName, Arc<ExprNode>>>>;

// Run the pass to a fixpoint, so a `fix` nested inside a lifted function is defunctionalized too.
pub fn run(prg: &mut Program, show_build_times: bool) {
    // Global functions whose definition is a bare lambda, so `fix(GlobalName)` can resolve to it. The
    // definition is preprocessed only when a `fix` actually resolves to it (see `normalize_for_lift`),
    // so a program that never fixes a global pays nothing here.
    let global_lambdas: GlobalLambdas = Rc::new(
        prg.symbols
            .iter()
            .filter_map(|(name, sym)| {
                let expr = sym.expr.as_ref()?;
                expr.is_lam().then(|| (name.clone(), expr.clone()))
            })
            .collect(),
    );
    // Shared across the whole run, so a global's fixpoint is created once and its self-`fix` closes
    // onto that one `G`.
    let global_fix_refs: GlobalFixRefs = Rc::new(RefCell::new(Map::default()));

    let mut stable = Set::default();
    while run_one(
        prg,
        &mut stable,
        &global_lambdas,
        &global_fix_refs,
        show_build_times,
    ) {}
}

// Run one pass over all symbols. Returns whether any symbol changed.
//
// `stable` holds symbols already known to contain no further `fix` to defunctionalize; they are
// carried over untouched.
fn run_one(
    prg: &mut Program,
    stable: &mut Set<FullName>,
    global_lambdas: &GlobalLambdas,
    global_fix_refs: &GlobalFixRefs,
    show_build_times: bool,
) -> bool {
    let _sw = StopWatch::new("defunctionalize_fix::run_one", show_build_times);

    let mut changed = false;
    // Elaboration aliases the self-parameter of `fix` behind a `let` (`let go = #self`); collapsing
    // trivial variable-aliasing lets makes every self-reference name the parameter directly, so the
    // single substitution below reaches them all.
    let arity_map = let_elimination::create_global_lambda_to_arity_map(prg);
    let symbols = std::mem::take(&mut prg.symbols);
    let mut global_names: Set<FullName> = symbols.keys().cloned().collect();
    let mut new_symbols: Map<FullName, Symbol> = Map::default();
    let mut new_tycons: Map<TyCon, TyConInfo> = Map::default();

    for (name, mut sym) in symbols {
        if stable.contains(&name) {
            new_symbols.insert(name, sym);
            continue;
        }

        // Only a symbol that applies `fix` needs any work.
        if !sym
            .expr
            .as_ref()
            .unwrap()
            .free_vars()
            .iter()
            .any(is_std_fix)
        {
            stable.insert(name.clone());
            new_symbols.insert(name, sym);
            continue;
        }

        // Normalize before lifting: uniquify locals for a collision-free capture destructure and
        // collapse self-aliases; the arity map also inlines saturated global-lambda applications.
        let expr = normalize_for_lift(sym.expr.as_ref().unwrap(), &arity_map);

        let mut visitor = FixDefunctionalizer::new(
            name.clone(),
            global_names.clone(),
            global_lambdas.clone(),
            global_fix_refs.clone(),
        );
        let res = visitor.traverse(&expr);

        if !res.changed {
            // No `fix` was applied to a literal lambda here; keep the original symbol and drop the
            // preprocessing, matching the untouched symbols above.
            stable.insert(name.clone());
            new_symbols.insert(name, sym);
            continue;
        }

        changed = true;
        sym.expr = Some(res.expr);
        for lifted in visitor.lifted {
            global_names.insert(lifted.func_name.clone());
            new_tycons.insert(
                lifted.cap.tycon.as_ref().clone(),
                lifted.cap.tycon_info.clone(),
            );
            new_symbols.insert(lifted.func_name.clone(), lifted.into_symbol());
        }
        new_symbols.insert(name, sym);
    }

    prg.type_env.add_tycons(new_tycons);
    prg.symbols = new_symbols;
    changed
}

// A global function lifted out of one `fix` application.
struct LiftedFix {
    func_name: FullName,
    // `G = |cap| let FixCap { .. } = cap; body[self := G(cap)]`, typed `FixCap -> a -> b`.
    func_expr: Arc<ExprNode>,
    cap: CaptureStruct,
}

impl LiftedFix {
    fn into_symbol(self) -> Symbol {
        Symbol {
            name: self.func_name.clone(),
            generic_name: self.func_name,
            ty: self.func_expr.type_.as_ref().unwrap().clone(),
            expr: Some(self.func_expr),
            inline_into_callers: false,
        }
    }
}

struct FixDefunctionalizer {
    // The symbol being processed; lifted-function names are derived from it.
    current_symbol: FullName,
    // Global names in use, to keep generated names collision-free.
    global_names: Set<FullName>,
    // Counter feeding the generated names.
    counter: u32,
    // Functions lifted out during this traversal.
    lifted: Vec<LiftedFix>,
    // Local lambda bindings `let name = |..| ..`, so `fix(name)` — a `fix` whose argument is a
    // let-bound lambda that could not be inlined into the call (e.g. it is used more than once) —
    // resolves to the lambda. Names are unique across the symbol (`unique_local_names` ran), so one
    // flat map needs no scoping.
    local_lambdas: Map<FullName, Arc<ExprNode>>,
    // Global lambda definitions and the deduplicated fixpoints built for them, shared across symbols.
    global_lambdas: GlobalLambdas,
    global_fix_refs: GlobalFixRefs,
}

impl FixDefunctionalizer {
    fn new(
        current_symbol: FullName,
        global_names: Set<FullName>,
        global_lambdas: GlobalLambdas,
        global_fix_refs: GlobalFixRefs,
    ) -> Self {
        Self {
            current_symbol,
            global_names,
            counter: 0,
            lifted: vec![],
            local_lambdas: Map::default(),
            global_lambdas,
            global_fix_refs,
        }
    }

    // Build the lifted global `G` for `fix(f)`, and return a reference to `G` (typed
    // `FixCap -> a -> b`) together with the capture-struct expression built from the current scope.
    // `G(cap)` then reconstructs the recursive value `fix(f) : a -> b`.
    fn lift(&mut self, f: &Arc<ExprNode>, state: &VisitState) -> (Arc<ExprNode>, Arc<ExprNode>) {
        // `f = |self| f_body`, single-parameter before uncurrying.
        let (self_params, f_body) = f.destructure_lam();
        assert_eq!(
            self_params.len(),
            1,
            "the argument of `fix` is a single-parameter lambda before uncurrying"
        );
        let self_name = self_params[0].name.clone();
        let ab_ty = f_body.type_.as_ref().unwrap().clone(); // a -> b

        // Capture = free local variables of `f`, with their types read from the current scope.
        let cap_fields: Vec<(FullName, Arc<TypeNode>)> = f
            .lambda_cap_names()
            .iter()
            .map(|n| {
                let ty = state.scope.get_local(&n.name).unwrap().unwrap();
                (n.clone(), ty)
            })
            .collect();
        // Name the lifted function first: the capture struct is named after it, so that a value of
        // that capture struct says which function consumes it.
        let func_name = fresh_global_name(
            &self.current_symbol,
            "#fix_defunct",
            &mut self.counter,
            &mut self.global_names,
        );
        let cap = CaptureStruct::new("#FixCap", &func_name, &cap_fields);
        // The capture parameter must not clash with a captured field name. A captured field can
        // itself be an outer lift's `#fixcap..` parameter that this lambda closed over, so `cap`'s
        // destructure would bind a variable of that name and shadow this parameter — then `G(cap)`
        // would forward the captured struct in place of this one.
        let cap_param = loop {
            let name = format!("#fixcap{}", self.counter);
            self.counter += 1;
            if !cap_fields.iter().any(|(n, _)| n.to_string() == name) {
                break name;
            }
        };
        let cap_param_name = FullName::local(&cap_param);

        // The recursive value `G(cap)`, of type `a -> b`.
        let g_ref =
            expr_var(func_name.clone(), None).set_type(type_fun(cap.ty.clone(), ab_ty.clone()));
        let self_replacement = expr_app_typed(
            g_ref.clone(),
            vec![expr_var(cap_param_name.clone(), None).set_type(cap.ty.clone())],
        );

        // `body[self := G(cap)]`: capture-avoiding, and it leaves shadowed occurrences of `self`
        // untouched.
        let body = substitute_free_name(&f_body, &self_name, &self_replacement);

        // `G = |cap| let FixCap { .. } = cap; body`.
        let g_body = expr_let_typed(
            cap.pattern(),
            expr_var(cap_param_name, None).set_type(cap.ty.clone()),
            body,
        );
        let g_expr = internalize_let_to_var_at_head(&expr_abs_typed(
            var_local(&cap_param),
            cap.ty.clone(),
            g_body,
        ));

        let cap_expr = cap.struct_expr();
        self.lifted.push(LiftedFix {
            func_name,
            func_expr: g_expr,
            cap,
        });
        (g_ref, cap_expr)
    }
}

impl ExprVisitor for FixDefunctionalizer {
    fn start_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        let (func, args) = expr.destructure_app();
        if !func.is_var() || !is_std_fix(&func.get_var().name) || args.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        // Resolve `fix(f)` to `G(cap)` of type `a -> b`, where `f` is the recursion body. Only a bare
        // lambda body qualifies — inline, a local `let` binding, or a global definition. A `fix` whose
        // argument is anything else keeps the closure-based lowering.
        let base = if args[0].is_lam() {
            let (g_ref, cap) = self.lift(&args[0], state);
            expr_app_typed(g_ref, vec![cap])
        } else if args[0].is_var() {
            let name = args[0].get_var().name.clone();
            // Bind the lookups to owned values so no borrow of `self` outlives the `self.lift` calls.
            let local = self.local_lambdas.get(&name).cloned();
            let global = self.global_lambdas.get(&name).cloned();
            if let Some(lam) = local {
                // Lifted per site; the duplication is cheap and the binding stays for other uses.
                let (g_ref, cap) = self.lift(&lam, state);
                expr_app_typed(g_ref, vec![cap])
            } else if let Some(raw) = global {
                // Lifted once and shared, so a global that fixes itself converges.
                let shared = self.global_fix_refs.borrow().get(&name).cloned();
                match shared {
                    Some(shared) => shared,
                    None => {
                        // A global definition is raw here (unlike a local binding, recorded from the
                        // already-preprocessed enclosing symbol), so collapse its self-alias first.
                        let lam = normalize_for_lift(&raw, &Map::default());
                        let (g_ref, cap) = self.lift(&lam, state);
                        let shared = expr_app_typed(g_ref, vec![cap]);
                        self.global_fix_refs
                            .borrow_mut()
                            .insert(name, shared.clone());
                        shared
                    }
                }
            } else {
                return StartVisitResult::VisitChildren;
            }
        } else {
            return StartVisitResult::VisitChildren;
        };

        // `fix(f)(rest..)` becomes `G(cap)(rest..)`.
        let mut new_expr = base;
        for arg in &args[1..] {
            new_expr = expr_app_typed(new_expr, vec![arg.clone()]);
        }
        StartVisitResult::ReplaceAndRevisit(new_expr)
    }

    fn start_visit_var(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_var(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_llvm(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_llvm(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn end_visit_app(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_lam(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_lam(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_let(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        // Record `let name = |..| ..` so a later `fix(name)` in this let's body resolves to the
        // lambda. The `let` is an ancestor of any such use, so recording on the way down suffices;
        // the binding stays even after a use is rewritten, since the lambda may be used elsewhere too.
        if let Expr::Let(pat, bound, _) = &*e.expr {
            if bound.is_lam() {
                let vars = pat.var_infos();
                if vars.len() == 1 {
                    self.local_lambdas.insert(vars[0].0.clone(), bound.clone());
                }
            }
        }
        StartVisitResult::VisitChildren
    }
    fn end_visit_let(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_if(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_if(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_match(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_match(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_tyanno(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_tyanno(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_make_struct(
        &mut self,
        _e: &Arc<ExprNode>,
        _s: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_make_struct(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_array_lit(
        &mut self,
        _e: &Arc<ExprNode>,
        _s: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_array_lit(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_ffi_call(
        &mut self,
        _e: &Arc<ExprNode>,
        _s: &mut VisitState,
    ) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_ffi_call(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
    fn start_visit_eval(&mut self, _e: &Arc<ExprNode>, _s: &mut VisitState) -> StartVisitResult {
        StartVisitResult::VisitChildren
    }
    fn end_visit_eval(&mut self, e: &Arc<ExprNode>, _s: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(e)
    }
}

// Normalize an expression before its `fix` lambdas are lifted: make all local names unique, then run
// let-elimination to a fixpoint. Uniquifying keeps the capture destructuring `let FixCap { .. } = cap`
// collision-free; let-elimination collapses the `let user_name = #param` aliases the elaborator
// inserts so each self-reference names the `fix` parameter directly, letting one substitution reach
// them all. `arity_map` additionally lets let-elimination inline saturated global-lambda
// applications; pass an empty map to collapse aliases only.
fn normalize_for_lift(expr: &Arc<ExprNode>, arity_map: &Map<FullName, usize>) -> Arc<ExprNode> {
    let mut expr = unique_local_names::run_on_expr(expr, Set::default());
    while let_elimination::run_on_expr_once(&mut expr, arity_map) {}
    expr
}
