use super::{
    capture_struct::{fresh_global_name, CaptureStruct},
    find_usage_of_name::{self, UsageType},
    uncurry::internalize_let_to_var_at_head,
    unique_local_names,
};
use crate::graph::Graph;
use crate::{
    ast::{
        expr::{
            expr_abs_typed, expr_app_typed, expr_let_typed, expr_make_struct, expr_var, var_local,
            var_var, ExprNode,
        },
        name::FullName,
        pattern::{Pattern, PatternNode},
        program::{Program, Symbol},
        traverse::{EndVisitResult, ExprVisitor, StartVisitResult, VisitState},
        types::{type_fun, TyCon, TyConInfo, TypeNode},
    },
    constants::{
        CAP_NAME, CLOSURE_CALL_LAM_SUFFIX, CLOSURE_CAP_NAME, CLOSURE_LAM_SUFFIX,
        CLOSURE_SPEC_SUFFIX,
    },
    misc::{Map, Set},
    optimization::{pull_let, rename::rename_free_names},
    tool::stopwatch::StopWatch,
};
use std::{cell::RefCell, collections::VecDeque, mem, rc::Rc, sync::Arc};

/*
# Closure specialization

## Overview

This pass makes a lambda cheaper to call, by two techniques that feed each other. **Decapturing**
lifts a lambda to a global function taking its captured environment as an argument, which leaves a
plain value where a closure was. **Specialization** then copies a global function that receives such
a value, one copy per lifted lambda it is given, so the copy calls that lambda by name.

### Decapturing

For each lambda expression, a structure is defined that summarizes the values captured by that lambda expression.
And the lambda expression is defined as a global function.

For example, consider the following lambda expression:
```
let f = |x| x + n;
```

Then, the following structure and global function are defined. The names below are written short
for reading; see "The names this pass mints" for the shape they really take.
```
type #Cap = unbox struct { n: I64 };

#lam : #Cap -> I64 -> I64;
#lam = |{ n : n }, x| x + n;
```

The creation of the lambda value is replaced with `#Cap { n : n }`.
```
let f = #Cap { n : n };
```

### Rewriting the usage of the lambda

The call of the decaptured lambda expression `f(x)` is transformed into the following code.
```
#lam(f, x)
```

In principle, `f` is replaced with `#lam(f)` in places where `f` appears alone (i.e., not in a call expression).
However, in cases where specialization can be applied, `f` is left as is.

### Specializing a function on the lambda it is given

Consider the case where a lambda is given as an argument to a global function.
As an example, consider the case where `f` is passed as the second argument to `fold`.

```
fold : S -> (A -> S -> S) -> Iter -> S;
fold = |s, op, iter| (
    match iter.advance {
        none() => s,
        some((iter, a)) => iter.fold(op(a, s), op)
    }
);
```

```
it.fold(s0, f)
```

In this case, the following code is generated.

```
fold#spec : S -> #Cap -> Iter -> S;
fold#spec = |s, op, iter| (
    match iter.advance {
        none() => s,
        some((iter, a)) => iter.fold#spec(#lam(op, a, s), op)
    }
);
```

```
it.fold#spec(s0, f)
```

### Narrowing a capture list

A lambda that captures another lambda holds it as a closure. Where the captured value's identity is
known, the field holding it takes the capture list of that value instead of the closure, which is
called **narrowing** below. A narrowed capture list is a different type, so the lambda that consumes
it is copied to receive it: that copy is what the field's own uses reach directly.

Narrowing is what lets a chain of copies continue past a lambda that only relays the closure it was
given.

## Applicable range and limitations

### The path from defining a lambda to using it

If a lambda is defined and used as is, decapturing is applied.
Example: `iter.fold(s0, |acm, i| acm + i)`

If a lambda is defined in the right-hand side of a let statement and its name is used, decapturing is applied.
Example: `let f = |acm, i| acm + i; iter.fold(s0, f)`

However, if the path from defining a lambda to using it is more complex than this, decapturing is not applied.
Example: `let (_, f) = (0, |acm, i| acm + i); iter.fold(s0, f)`

## The names this pass mints

The names carry one `#closure` stem, so that a dump says which pass produced them:

* `<symbol>#closure_lam<n>` — the global function a decaptured lambda becomes.
* `<symbol>#closure_spec_<hash>` — the copy of a global function specialized on the lambdas passed
  to it, where the hash stands for which way in received which value.
* `<local>#closure_call_lam` — a local binding. Where an inline-LLVM expression reads a variable
  holding a decaptured lambda's capture list, the call of that lambda is bound to a local of this
  name and the expression reads that instead.
* `CLOSURE_CAP_NAME` — the parameter a decaptured lambda receives its capture list through.

The capture struct's type constructor is named by `CaptureStruct`, which this pass gives the prefix
`#CapList` and `defunctionalize_fix` gives `#FixCap`: those two are chosen together at that
constructor and read against each other, so they sit outside this stem.

## Relations to other optimizations

* We perform `pull-let` transformation before decapturing.
* Inline expansion should be performed before this optimization. For example, the expression `f >> g` is replaced with a lambda expression by inline expansion, making it a target of this optimization.
* It may be worth performing inline expansion after this optimization. This is because global functions are generated by this optimization, and inline expansion may be performed on them.
* It may be worth performing inline expansion before this optimization. This is because the number of arguments that can be specialized increases due to eta expansion.
* To improve the performance of global functions generated by this optimization, uncurrying should be performed after this optimization.
*/

// A way into a function, which is what a copy of it is keyed on. Decapturing leaves a value two ways
// of reaching a function's body: as an argument, or as a field of the capture list a lifted lambda
// receives. A path one field deep therefore covers both, and covering no more is what keeps the set
// of copies a program can ask for finite.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Slot {
    // Which argument the value arrives through.
    arg: usize,
    // `None` for the argument itself; `Some(j)` for the j-th field of the capture list it carries.
    field: Option<usize>,
}

impl Slot {
    fn arg(arg: usize) -> Self {
        Slot { arg, field: None }
    }

    // The way in through the j-th field of the capture list, which a lifted lambda receives as its
    // first argument.
    fn capture_field(field: usize) -> Self {
        Slot {
            arg: 0,
            field: Some(field),
        }
    }

    // How the slot reads in a copy's name, and so in the hash that name carries.
    fn to_string(&self) -> String {
        match self.field {
            None => format!("{}", self.arg),
            Some(field) => format!("{}.{}", self.arg, field),
        }
    }
}

// A closure value whose identity is known: which lambda it is, and which of the fields of the
// capture list it carries are themselves known.
//
// This is what a copy is keyed on, and what the type constructor of a capture list is named after,
// so that a value of that type says what to call it with.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct Tree {
    // The lambda decapturing lifted, which is what a call through this value reaches.
    lambda: FullName,
    // The capture fields whose own identity is known, by position, in ascending order.
    fields: Vec<(usize, Tree)>,
}

impl Tree {
    // The value of a lambda whose capture fields are all still closures.
    fn leaf(lambda: FullName) -> Self {
        Tree {
            lambda,
            fields: Vec::new(),
        }
    }

    // How the tree reads in a name, and so in the hash that name carries.
    fn to_string(&self) -> String {
        let mut text = self.lambda.to_string();
        for (field, tree) in &self.fields {
            text += &format!("|{}:{}", field, tree.to_string());
        }
        text
    }

    // The unit that receives a value of this tree: the lambda, with its known capture fields
    // substituted.
    fn unit(&self) -> UnitKey {
        UnitKey {
            origin: self.lambda.clone(),
            subst: self
                .fields
                .iter()
                .map(|(field, tree)| (Slot::capture_field(*field), tree.clone()))
                .collect(),
        }
    }
}

// One copy of a function: the function it copies, and what each of its ways in is known to receive.
// An empty substitution names the function itself.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct UnitKey {
    origin: FullName,
    // Slots in ascending order, so that the name below is a function of the key alone.
    subst: Vec<(Slot, Tree)>,
}

impl UnitKey {
    fn new(origin: FullName, subst: Map<Slot, Tree>) -> Self {
        let mut subst = subst.into_iter().collect::<Vec<_>>();
        subst.sort_by_key(|(slot, _)| *slot);
        UnitKey { origin, subst }
    }

    fn name(&self) -> FullName {
        if self.subst.is_empty() {
            return self.origin.clone();
        }
        let mut full_name = self.origin.clone();
        let name = full_name.name_as_mut();
        *name += CLOSURE_SPEC_SUFFIX;
        let mut hash_data = String::new();
        for (slot, tree) in &self.subst {
            hash_data += &format!(",{},{}", slot.to_string(), tree.to_string());
        }
        *name += &format!("_{:x}", md5::compute(hash_data));
        full_name
    }

    // The capture list this unit receives, where it copies a lifted lambda whose capture list is
    // narrowed. This is the inverse of `Tree::unit`.
    fn capture_list_tree(&self) -> Option<Tree> {
        let fields = self
            .subst
            .iter()
            .filter_map(|(slot, tree)| slot.field.map(|field| (field, tree.clone())))
            .collect::<Vec<_>>();
        if fields.is_empty() {
            return None;
        }
        Some(Tree {
            lambda: self.origin.clone(),
            fields,
        })
    }
}

// What a chain of copies has committed to, keyed by `(function, slot, lambda)`. The value is the one
// value that key may be specialized on: meeting the same key with a different value is what stops a
// recursion that wraps its closure argument on every round from asking for a copy per round. Every
// copy carries the table of the request that created it, so the walk over that copy continues the
// same chain rather than starting a fresh one.
type Pinned = Map<(FullName, Slot, FullName), Tree>;

// Commit the chain reaching here to specializing `slot` of `func` on `tree`, and report whether it
// may be specialized on at all.
//
// It may not where the chain has already committed that key to a different value: that is a
// recursion handing the next round a closure built from the one it was given, and following it would
// ask for one copy per round.
fn commit(pinned: &mut Pinned, func: &FullName, slot: Slot, tree: &Tree) -> bool {
    let key = (func.clone(), slot, tree.lambda.clone());
    match pinned.get(&key) {
        Some(committed) => committed == tree,
        None => {
            pinned.insert(key, tree.clone());
            true
        }
    }
}

// What a lambda expression became once decaptured.
struct LiftedLambda {
    // The capture list threading its captured environment into the global function.
    cap: CaptureStruct,
    // The type of that global function.
    func_ty: Arc<TypeNode>,
}

// The lambdas decapturing has lifted, and the capture lists this pass has minted for them.
//
// A lambda is recorded here the moment it is lifted, so a value of its capture list can be read at
// once. The body of the global function it became lives in the program's symbol table, where the
// walks that follow keep it current.
#[derive(Default)]
struct LiftedLambdas {
    // Each lifted lambda by the name of the global function it became.
    lambdas: Map<FullName, LiftedLambda>,
    // The value a capture list of each type constructor this pass minted carries. This is what makes
    // the type of a capture list say what to call it with.
    trees: Map<FullName, Tree>,
    // The type constructors minted so far, which the caller registers into the program's type
    // environment.
    new_tycons: Map<TyCon, TyConInfo>,
}

impl LiftedLambdas {
    fn insert(&mut self, name: FullName, cap: CaptureStruct, func_ty: Arc<TypeNode>) {
        self.record_capture_list(&cap, &Tree::leaf(name.clone()));
        self.lambdas.insert(name, LiftedLambda { cap, func_ty });
    }

    // The capture list a lifted lambda was built with.
    fn capture_struct(&self, lambda: &FullName) -> Option<&CaptureStruct> {
        self.lambdas.get(lambda).map(|lifted| &lifted.cap)
    }

    // The type of the global function a lifted lambda became.
    fn func_ty(&self, lambda: &FullName) -> Arc<TypeNode> {
        self.lambdas[lambda].func_ty.clone()
    }

    // The value a capture list of `tycon` carries.
    fn tree_of_capture_list(&self, tycon: &FullName) -> Option<Tree> {
        self.trees.get(tycon).cloned()
    }

    // The capture struct a value of `tree` is: the lifted lambda's, with each known field narrowed
    // to the capture struct of what it holds. The type constructor is named after the unit that
    // receives it, so the type and the tree determine each other.
    fn capture_struct_of(&mut self, tree: &Tree) -> CaptureStruct {
        let mut cap = self.lambdas[&tree.lambda].cap.clone();
        let owner = tree.unit().name();
        for (field, inner) in &tree.fields {
            let inner_ty = self.capture_struct_of(inner).ty;
            cap = cap.with_field_type(CAP_LIST_PREFIX, &owner, *field, inner_ty);
        }
        self.record_capture_list(&cap, tree);
        cap
    }

    fn record_capture_list(&mut self, cap: &CaptureStruct, tree: &Tree) {
        self.trees.insert(cap.tycon.name.clone(), tree.clone());
        self.new_tycons
            .insert(cap.tycon.as_ref().clone(), cap.tycon_info.clone());
    }

    fn take_new_tycons(&mut self) -> Map<TyCon, TyConInfo> {
        mem::take(&mut self.new_tycons)
    }
}

// Run the optimization over `prg`, in three phases.
//
// Lifting runs to completion first, so that the set of functions a copy can be keyed on is settled
// before anything is keyed on it. The table of what is worth copying is then solved once over the
// program lifting left behind. Only then are copies made, from that same program, so that a copy's
// body names the functions the table answers for.
pub fn run(prg: &mut Program, show_build_times: bool) {
    let _sw = StopWatch::new("closure_specialization::run", show_build_times);

    let lifted = Rc::new(RefCell::new(LiftedLambdas::default()));
    lift_all(prg, &lifted, show_build_times);

    let specializable_funcs = Rc::new(specializable_functions(&prg.symbols, &lifted.borrow()));
    realize_all(prg, &lifted, specializable_funcs, show_build_times);
}

// Lift every lambda in the program to a global function, until lifting one leaves nothing more to
// lift. A lambda lifted here is a global function of its own, which the next pass over the symbols
// walks in turn.
fn lift_all(prg: &mut Program, lifted: &Rc<RefCell<LiftedLambdas>>, show_build_times: bool) {
    let _sw = StopWatch::new("closure_specialization::lift_all", show_build_times);

    // Nothing is specializable during this phase, so the walk only lifts: a value whose identity is
    // known is wrapped back into a closure wherever it is used, and no request is raised.
    let nothing_specializable = Rc::new(Map::default());
    let mut stable = Set::default();

    loop {
        let mut changed = false;
        let symbols = mem::take(&mut prg.symbols);
        let mut new_symbols: Map<FullName, Symbol> = Map::default();
        let mut global_names = symbols.keys().cloned().collect::<Set<_>>();

        for (name, mut sym) in symbols {
            if stable.contains(&name) {
                new_symbols.insert(name, sym);
                continue;
            }
            let mut visitor = ClosureSpecializationVisitor::new(
                name.clone(),
                nothing_specializable.clone(),
                lifted.clone(),
                global_names.clone(),
                Pinned::default(),
            );
            let expr = pull_let::run_on_expr(sym.expr.as_ref().unwrap()); // Increase the number of places decapturing applies to.
            let expr = unique_local_names::run_on_expr(&expr, Set::default()); // Preconditions for decapturing.
            let trav_res = visitor.traverse(&expr);
            if !trav_res.changed {
                stable.insert(name.clone());
                new_symbols.insert(name, sym);
                continue;
            }
            changed = true;
            sym.expr = Some(trav_res.expr);
            register_lifted_lambdas(visitor.new_symbols, &mut new_symbols, &mut global_names);
            new_symbols.insert(name, sym);
        }

        prg.symbols = new_symbols;
        if !changed {
            break;
        }
    }

    prg.type_env.add_tycons(lifted.borrow_mut().take_new_tycons());
}

// Make every copy the program asks for, starting from the functions themselves.
//
// The bodies every copy is made from are the ones lifting left behind, so a copy names the same
// functions its original does and the table answers for all of them.
fn realize_all(
    prg: &mut Program,
    lifted: &Rc<RefCell<LiftedLambdas>>,
    specializable_funcs: Rc<Map<FullName, SpecializableFunctionInfo>>,
    show_build_times: bool,
) {
    let _sw = StopWatch::new("closure_specialization::realize_all", show_build_times);

    let originals = mem::take(&mut prg.symbols);
    let mut global_names = originals.keys().cloned().collect::<Set<_>>();
    let mut symbols: Map<FullName, Symbol> = Map::default();

    // Every function stands for the copy of itself that substitutes nothing.
    let mut queue = originals
        .keys()
        .map(|origin| SpecializationRequest {
            unit: UnitKey::new(origin.clone(), Map::default()),
            org_func_ty: originals[origin].ty.clone(),
            pinned: Pinned::default(),
        })
        .collect::<VecDeque<_>>();

    while let Some(request) = queue.pop_front() {
        let name = request.unit.name();
        if symbols.contains_key(&name) {
            continue;
        }
        let original = &originals[&request.unit.origin];
        let expr = unique_local_names::run_on_expr(original.expr.as_ref().unwrap(), Set::default());

        // A copy of a lifted lambda whose capture list is narrowed receives it through the same
        // parameter, at the narrowed type. The walk retypes the pattern destructuring it when it
        // meets that pattern.
        let narrowed = narrowed_capture_list(&request.unit, lifted);
        let expr = match &narrowed {
            Some(narrowed) => {
                let codom = expr.type_.as_ref().unwrap().get_lambda_dst();
                expr.set_type(type_fun(narrowed.cap.ty.clone(), codom))
            }
            None => expr,
        };
        let local_decap_lambdas = known_arguments(&request.unit, &expr, lifted);

        let mut visitor = ClosureSpecializationVisitor::new(
            name.clone(),
            specializable_funcs.clone(),
            lifted.clone(),
            global_names.clone(),
            request.pinned.clone(),
        );
        visitor.local_decap_lambdas = local_decap_lambdas;
        visitor.narrowed_capture_list = narrowed;
        let trav_res = visitor.traverse(&expr);
        assert!(
            visitor.narrowed_capture_list.is_none(),
            "the capture list of {} is narrowed, but its body does not destructure one",
            name.to_string()
        );
        assert!(
            visitor.new_symbols.is_empty(),
            "{} still has a lambda to lift after lifting ran to completion",
            name.to_string()
        );

        let ty = request.specialized_func_ty(&mut lifted.borrow_mut());
        symbols.insert(
            name.clone(),
            Symbol {
                name: name.clone(),
                generic_name: request.unit.origin.clone(),
                ty,
                expr: Some(trav_res.expr),
            },
        );
        global_names.insert(name);
        queue.extend(visitor.required_specializations);
    }

    prg.type_env.add_tycons(lifted.borrow_mut().take_new_tycons());
    prg.symbols = symbols;
}

// The capture list a unit receives in place of the one its origin was built with, where the unit
// copies a lifted lambda and narrows one of its capture fields.
fn narrowed_capture_list(
    unit: &UnitKey,
    lifted: &RefCell<LiftedLambdas>,
) -> Option<NarrowedCaptureList> {
    let tree = unit.capture_list_tree()?;
    let original = lifted
        .borrow()
        .capture_struct(&unit.origin)
        .unwrap()
        .tycon
        .clone();
    let cap = lifted.borrow_mut().capture_struct_of(&tree);
    Some(NarrowedCaptureList {
        original,
        cap,
        fields: tree.fields,
    })
}

// What each substituted argument of `unit` holds, by the local name it arrives under, which is what
// the walk over `body` is told. A substituted capture field is left out: the walk learns that one
// from the pattern destructuring the capture list.
fn known_arguments(
    unit: &UnitKey,
    body: &Arc<ExprNode>,
    lifted: &RefCell<LiftedLambdas>,
) -> Map<FullName, Known> {
    let mut known_args = Map::default();
    let (args, _) = body.destructure_lam_sequence();
    for (slot, tree) in &unit.subst {
        if slot.field.is_some() {
            continue;
        }
        assert!(
            slot.arg < args.len(),
            "{} is substituted at argument {}, but takes {} of them",
            unit.origin.to_string(),
            slot.arg,
            args.len()
        );
        assert_eq!(args[slot.arg].len(), 1);
        let arg_name = args[slot.arg][0].name.clone();
        let cap_list_ty = lifted.borrow_mut().capture_struct_of(tree).ty;
        known_args.insert(
            arg_name.clone(),
            Known::bare(tree.clone(), expr_var(arg_name, None).set_type(cap_list_ty)),
        );
    }
    known_args
}

// Register the global function each lambda lifted by a walk became.
fn register_lifted_lambdas(
    new_symbols: Vec<Symbol>,
    symbols: &mut Map<FullName, Symbol>,
    global_names: &mut Set<FullName>,
) {
    for sym in new_symbols {
        global_names.insert(sym.name.clone());
        symbols.insert(sym.name.clone(), sym);
    }
}

// Compute the set of specializable functions.
//
// More precisely, it calculates whether a certain way into a function is specializable or not.
//
// Specialization can cause infinite loops.
// For example, suppose a function `f` takes a closure `p` as its first parameter. `f` creates a new closure `q` that captures `p` and calls `f(q)`.
// In this case, if the parameter `p` of `f` is specialized, it will require an infinite number of specializations for different types.
//
// The table says only which ways in are worth copying for; what stops such a chain is the commitment
// `commit` records, which refuses a key already met with a different value.
fn specializable_functions(
    symbols: &Map<FullName, Symbol>,
    lifted: &LiftedLambdas,
) -> Map<FullName, SpecializableFunctionInfo> {
    let (call_graph, name_to_idx) = call_graph_of(symbols);
    let call_graph_scc = call_graph.compute_sccs();

    // Callers of each function, which is who has to be judged again once it enters the table. A
    // callee absent from `symbols` is a copy whose body is not made yet, and nothing reads its slots
    // until it is.
    let mut callers = vec![Vec::<usize>::new(); call_graph.len()];
    for (caller, sym) in symbols {
        let caller_idx = name_to_idx[caller];
        for callee in sym.expr.as_ref().unwrap().free_vars() {
            if let Some(callee_idx) = name_to_idx.get(&callee) {
                callers[*callee_idx].push(caller_idx);
            }
        }
    }

    // Whether a way in is specializable is defined in terms of the table being built: a value
    // forwarded to a specializable way into another function is specializable in turn. Growing the
    // table can only make more ways in qualify, so starting from the empty table and adding what
    // qualifies reaches the least fixed point.
    //
    // Seeding the queue callees-first means most functions are judged once, since what they forward
    // to has settled by the time they come up.
    let mut order = (0..call_graph.len()).collect::<Vec<_>>();
    order.sort_by(|a, b| call_graph_scc[*b].cmp(&call_graph_scc[*a]));
    let mut queue = order.into_iter().collect::<VecDeque<_>>();
    let mut queued = vec![true; call_graph.len()];

    let mut specializable_funcs: Map<FullName, SpecializableFunctionInfo> = Map::default();
    while let Some(idx) = queue.pop_front() {
        queued[idx] = false;
        let sym_name = call_graph.get(idx);
        let sym = symbols.get(sym_name).unwrap();
        let specializable_slots = specializable_slots_of(sym, &specializable_funcs, lifted);
        if specializable_slots.is_empty() {
            continue;
        }
        let settled = specializable_funcs
            .get(sym_name)
            .is_some_and(|info| info.specializable_slots == specializable_slots);
        if settled {
            continue;
        }
        specializable_funcs.insert(
            sym_name.clone(),
            SpecializableFunctionInfo {
                specializable_slots,
            },
        );
        for caller_idx in &callers[idx] {
            if !queued[*caller_idx] {
                queued[*caller_idx] = true;
                queue.push_back(*caller_idx);
            }
        }
    }
    specializable_funcs
}

// Whether the table says a copy of `func` is worth making for `slot`.
fn is_specializable(
    specializable_funcs: &Map<FullName, SpecializableFunctionInfo>,
    func: &FullName,
    slot: Slot,
) -> bool {
    specializable_funcs
        .get(func)
        .is_some_and(|info| info.specializable_slots.contains(&slot))
}

// Whether a value arriving under `name` inside `body` is reached without an indirect call: it is
// either called there, or handed to a way into another function that is itself specializable. That
// is what a copy gains — the call becomes direct, or the function downstream gets a known lambda and
// is copied in turn — so the size of the function holding it does not enter the judgement.
fn reaches_a_direct_call(
    name: &FullName,
    body: &Arc<ExprNode>,
    specializable_funcs: &Map<FullName, SpecializableFunctionInfo>,
    lifted: &LiftedLambdas,
) -> bool {
    find_usage_of_name::run(body, name)
        .into_iter()
        .any(|usage| match usage {
            UsageType::CalledAsFunction => true,
            UsageType::FunctionArgument(func, idx) => {
                is_specializable(specializable_funcs, &func, Slot::arg(idx))
            }
            // A value captured into a lifted lambda's capture list arrives in that lambda's body
            // through the field it was stored in, which is a way in like an argument. A struct this
            // pass did not mint — one the program declares, or the capture list
            // `defunctionalize_fix` builds — carries no such way in, so the value is reached there
            // only by an indirect call.
            UsageType::CapturedInto(tycon, position) => lifted
                .tree_of_capture_list(&tycon)
                .is_some_and(|tree| {
                    is_specializable(
                        specializable_funcs,
                        &tree.lambda,
                        Slot::capture_field(position),
                    )
                }),
        })
}

// The ways into `sym` a copy is worth making for, judged against the table of specializable
// functions as it stands. Adding entries to `specializable_funcs` can only add slots here, never
// remove one.
fn specializable_slots_of(
    sym: &Symbol,
    specializable_funcs: &Map<FullName, SpecializableFunctionInfo>,
    lifted: &LiftedLambdas,
) -> Set<Slot> {
    let expr = sym.expr.as_ref().unwrap();

    // Check if each parameter of `sym` is specializable.
    let (params, body) = expr.destructure_lam_sequence();
    let params = params
        .iter()
        .map(|may_multi_param| {
            assert_eq!(may_multi_param.len(), 1);
            may_multi_param[0].name.clone()
        })
        .collect::<Vec<_>>();
    let param_tys = sym.ty.collect_app_src(usize::MAX).0;
    let mut specializable_slots = Set::default();
    for param_idx in 0..params.len() {
        // A parameter shadowed by a later one is never the one in scope where the body uses that
        // name, so nothing arrives through it.
        let param_name = &params[param_idx];
        if params[param_idx + 1..]
            .iter()
            .any(|name| name == param_name)
        {
            continue;
        }

        // A specializable argument must have a type of function.
        if param_tys[param_idx].is_closure()
            && reaches_a_direct_call(param_name, &body, specializable_funcs, lifted)
        {
            specializable_slots.insert(Slot::arg(param_idx));
        }
    }

    // The capture fields of a lifted lambda are ways into its body too, and narrowing one is what
    // lets a chain of copies continue past a lambda that only relays the closure it captured.
    if let Some(cap) = lifted.capture_struct(&sym.name) {
        if let Some((field_names, cap_body)) = capture_list_destructuring(&body, &cap.tycon) {
            for (position, (_, field_ty)) in cap.fields().iter().enumerate() {
                if field_ty.is_closure()
                    && reaches_a_direct_call(
                        &field_names[position],
                        &cap_body,
                        specializable_funcs,
                        lifted,
                    )
                {
                    specializable_slots.insert(Slot::capture_field(position));
                }
            }
        }
    }

    specializable_slots
}

// The `let` destructuring the capture list of a lifted lambda, as the names it binds the capture
// fields to and the body it stands in front of. `body` is what remains of the lambda's global
// function once its parameters are stripped; the destructuring stands at the head of it, behind any
// binding that does not depend on it.
fn capture_list_destructuring(
    body: &Arc<ExprNode>,
    tycon: &Arc<TyCon>,
) -> Option<(Vec<FullName>, Arc<ExprNode>)> {
    let mut expr = body.clone();
    while expr.is_let() {
        let pat = expr.get_let_pat();
        if let Pattern::Struct(pat_tycon, field_to_pat) = &pat.pattern {
            if pat_tycon.as_ref() == tycon.as_ref() {
                assert!(
                    field_to_pat.iter().all(|(_, _, pat)| pat.is_var()),
                    "the capture list {} is destructured by a pattern that binds a field to \
                     something other than a name",
                    tycon.name.to_string()
                );
                let names = field_to_pat
                    .iter()
                    .map(|(_, _, pat)| pat.get_var().name.clone())
                    .collect();
                return Some((names, expr.get_let_value()));
            }
        }
        expr = expr.get_let_value();
    }
    None
}

// The call graph of `symbols` — an edge from A to B means A calls B — together with the node each
// name is held at.
//
// A call site is rewritten to name a copy before that copy's body is made, so a callee absent from
// `symbols` is one about to be created and carries no edge yet.
fn call_graph_of(symbols: &Map<FullName, Symbol>) -> (Graph<FullName>, Map<FullName, usize>) {
    let names = symbols.keys().cloned().collect::<Vec<_>>();
    let name_to_idx = names
        .iter()
        .enumerate()
        .map(|(idx, name)| (name.clone(), idx))
        .collect::<Map<FullName, usize>>();
    let mut graph = Graph::new(names);
    for (caller, sym) in symbols {
        for callee in sym.expr.as_ref().unwrap().free_vars() {
            if let Some(callee_idx) = name_to_idx.get(&callee) {
                graph.connect_idx(name_to_idx[caller], *callee_idx);
            }
        }
    }
    (graph, name_to_idx)
}

// The capture list a copy of a lifted lambda receives in place of the one the lambda was built with.
struct NarrowedCaptureList {
    // The type constructor of the capture list the lambda was built with, which the pattern
    // destructuring it names.
    original: Arc<TyCon>,
    // The capture list this copy receives instead.
    cap: CaptureStruct,
    // What each narrowed field holds, by position.
    fields: Vec<(usize, Tree)>,
}

// The visitor that lifts a symbol's lambdas and records the copies they enable.
struct ClosureSpecializationVisitor {
    /* Decapturing */
    // The global function each lambda lifted by this walk became.
    new_symbols: Vec<Symbol>,
    // When a value whose identity is known is given a local name, it is stored here.
    local_decap_lambdas: Map<FullName, Known>,
    // Every lambda decapturing has lifted, which is what a tree is read against. A lambda lifted by
    // this walk is in here the moment it is made, so a tree naming it can be read at once.
    lifted: Rc<RefCell<LiftedLambdas>>,
    // Where this walk copies a lifted lambda whose capture list is narrowed: what the copy receives.
    // The `let` destructuring the capture list takes it.
    narrowed_capture_list: Option<NarrowedCaptureList>,

    /* Specialization */
    // Specializable functions
    specializable_funcs: Rc<Map<FullName, SpecializableFunctionInfo>>,
    // Copies this walk asks for
    required_specializations: Vec<SpecializationRequest>,

    /* Fields related to name generation of lambda function */
    // Counter used to generate lambda function names
    lam_func_counter: u32,
    // Name of the symbol currently being optimized
    // Used to generate the names of lambda functions.
    current_symbol: FullName,
    // Set of global names
    // Used to avoid name collisions when generating new global names.
    global_names: Set<FullName>,

    /* Termination */
    // What the chain reaching the symbol being walked has committed to.
    pinned: Pinned,
}

// A value whose identity is known: which lambda it is with its narrowed capture fields, and where
// the bare capture list carrying it can be read from.
//
// Reading the identity of a binding never rewrites it: a rewrite would leave the wrapping rules
// something to do again, and the two would take turns undoing each other.
#[derive(Clone)]
struct Known {
    tree: Tree,
    // An expression yielding the bare capture list, evaluable wherever the value itself is.
    cap_list: Arc<ExprNode>,
    // Whether the expression this was read off is the bare capture list rather than a closure
    // wrapped around one. Only a bare capture list has to be wrapped where a closure is called for.
    is_bare: bool,
}

impl Known {
    fn bare(tree: Tree, cap_list: Arc<ExprNode>) -> Self {
        Known {
            tree,
            cap_list,
            is_bare: true,
        }
    }
}

impl ClosureSpecializationVisitor {
    // Create a new visitor
    fn new(
        current_symbol: FullName,
        specializable_funcs: Rc<Map<FullName, SpecializableFunctionInfo>>,
        lifted: Rc<RefCell<LiftedLambdas>>,
        global_names: Set<FullName>,
        pinned: Pinned,
    ) -> Self {
        ClosureSpecializationVisitor {
            new_symbols: Vec::new(),
            local_decap_lambdas: Map::default(),
            lifted,
            narrowed_capture_list: None,
            specializable_funcs,
            required_specializations: Vec::new(),
            lam_func_counter: 0,
            current_symbol,
            global_names,
            pinned,
        }
    }

    // The capture struct a value of `tree` is.
    fn cap_of(&self, tree: &Tree) -> CaptureStruct {
        self.lifted.borrow_mut().capture_struct_of(tree)
    }

    // The function a value of `tree` is called through, as an expression, together with its type.
    // For a tree whose fields are all still closures this is the lifted lambda itself; otherwise it
    // is the copy that receives the narrowed capture list.
    fn lambda_func_of(&self, tree: &Tree) -> Arc<ExprNode> {
        let base = self.lifted.borrow().func_ty(&tree.lambda);
        let (mut doms, codom) = base.collect_app_src(usize::MAX);
        doms[0] = self.cap_of(tree).ty;
        let mut ty = codom;
        for dom in doms.iter().rev() {
            ty = type_fun(dom.clone(), ty);
        }
        expr_var(tree.unit().name(), None).set_type(ty)
    }

    // The expression a known value is carried by: the bare capture list, or the closure wrapping it.
    // A wrap names the copy of the lambda that receives the capture list as it now is (R3).
    fn value_expr(&self, known: &Known) -> Arc<ExprNode> {
        if known.is_bare {
            return known.cap_list.clone();
        }
        expr_app_typed(
            self.lambda_func_of(&known.tree),
            vec![known.cap_list.clone()],
        )
    }

    // Whether `expr` already is the expression `known` calls for. A narrowed capture list has a type
    // of its own, and the closure wrapping one keeps the type it had but changes the function it
    // names, so the two shapes are told apart differently.
    fn is_up_to_date(&self, expr: &Arc<ExprNode>, known: &Known) -> bool {
        let value = self.value_expr(known);
        let type_of = |expr: &Arc<ExprNode>| expr.type_.as_ref().unwrap().to_string();
        if known.is_bare {
            return type_of(expr) == type_of(&value);
        }
        let (func, args) = expr.destructure_app();
        let (value_func, value_args) = value.destructure_app();
        func.is_var()
            && func.get_var().name == value_func.get_var().name
            && type_of(&args[0]) == type_of(&value_args[0])
    }

    // The value `expr` carries, where its identity is known.
    //
    // Identity arrives in three shapes: a local this walk was told about, a capture list built here
    // — whose type constructor says which lambda consumes it — and a capture list wrapped back into
    // a closure, which is what lifting leaves wherever it could not say more.
    fn known_value(&self, expr: &Arc<ExprNode>) -> Option<Known> {
        if expr.is_var() {
            return self.local_decap_lambdas.get(&expr.get_var().name).cloned();
        }
        if let Some((tycon, _)) = expr.destructure_make_struct() {
            let tree = self.lifted.borrow().tree_of_capture_list(&tycon.name)?;
            return Some(Known::bare(tree, expr.clone()));
        }
        if !expr.is_app() {
            return None;
        }
        let (func, args) = expr.destructure_app();
        if args.len() != 1 || !func.is_var() {
            return None;
        }
        let known = self.known_value(&args[0])?;
        let name = &func.get_var().name;
        if *name != known.tree.lambda && *name != known.tree.unit().name() {
            return None;
        }
        Some(Known {
            is_bare: false,
            ..known
        })
    }

    // Narrow the capture list a known value is carried by, where it is built here: a field the table
    // is willing to specialize on takes the capture list of the value it holds, in place of the
    // closure. R1.
    //
    // Creating a narrowed value asks for the copy of the lambda that receives it, so that the value
    // can be wrapped back into a closure wherever one is called for.
    fn narrow(&mut self, known: Known, pinned: &mut Pinned) -> Known {
        let mut fields = match known.cap_list.destructure_make_struct() {
            Some((_, fields)) => fields.clone(),
            None => return known,
        };
        // The table is held through the loop below, which takes `self` mutably.
        let specializable_funcs = self.specializable_funcs.clone();
        let Some(info) = specializable_funcs.get(&known.tree.lambda) else {
            return known;
        };
        let mut narrowed_fields = Vec::new();
        for (position, (_, _, value)) in fields.iter_mut().enumerate() {
            let slot = Slot::capture_field(position);
            if !info.specializable_slots.contains(&slot) {
                continue;
            }
            let known_field = match self.known_value(value) {
                Some(known_field) => known_field,
                None => continue,
            };
            if !commit(pinned, &known.tree.lambda, slot, &known_field.tree) {
                continue;
            }
            *value = known_field.cap_list;
            narrowed_fields.push((position, known_field.tree));
        }
        if narrowed_fields == known.tree.fields {
            return known;
        }

        let tree = Tree {
            lambda: known.tree.lambda,
            fields: narrowed_fields,
        };
        self.request_lambda_unit(&tree, pinned);
        let cap = self.cap_of(&tree);
        let cap_list = expr_make_struct(
            cap.tycon.clone(),
            fields.into_iter().map(|(name, _, e)| (name, e)).collect(),
        )
        .set_type(cap.ty);
        Known {
            tree,
            cap_list,
            is_bare: known.is_bare,
        }
    }

    // Ask for the copy of the lambda that receives a capture list of `tree`. Asking where the tree is
    // created is what makes wrapping a value of it back into a closure legal everywhere.
    fn request_lambda_unit(&mut self, tree: &Tree, pinned: &Pinned) {
        let unit = tree.unit();
        if unit.subst.is_empty() {
            return;
        }
        let org_func_ty = self.lifted.borrow().func_ty(&tree.lambda);
        self.required_specializations.push(SpecializationRequest {
            unit,
            org_func_ty,
            pinned: pinned.clone(),
        });
    }

    // Whether `expr` is a lambda whose captured environment can be read off it, so that it can be
    // lifted to a global function taking that environment as an argument. The free variables of an
    // expression leave `CAP_NAME` out, so a body that reads it captures more than this can see.
    fn decapturable(expr: &Arc<ExprNode>) -> bool {
        // If the expression is a not lambda expression, it is not decapturable.
        if !expr.is_lam() {
            return false;
        }

        let body = expr.get_lam_body();
        if body.has_free_var(&FullName::local(CAP_NAME)) {
            return false;
        }

        true
    }

    // Decapture a lambda expression.
    //
    // Returns the value the lambda is, and the expression that generates its capture list.
    fn decapture_lambda(
        &mut self,
        mut lam: Arc<ExprNode>,
        state: &mut VisitState,
    ) -> (Tree, Arc<ExprNode>) {
        // Get the capture list.
        let cap_names = lam.lambda_cap_names();

        // If the lambda captures a decaptured lambda, visit `lam` in advance to ensure that the decaptured lambda in `lam` is processed.
        for cap_name in &cap_names {
            if self.local_decap_lambdas.contains_key(cap_name) {
                let lam_visit_res = self.visit_expr(&lam, state);
                lam = self.revisit_if_changed(lam_visit_res, state).expr;
                break;
            }
        }

        // For each captured name, get its type.
        let cap_names_types = cap_names
            .iter()
            .map(|name| {
                let ty = state.scope.get_local(&name.name).unwrap().unwrap();
                (name.clone(), ty.clone())
            })
            .collect::<Vec<_>>();

        // Name the lifted function first: the capture list is named after it, so that a value of
        // that capture list says which function consumes it.
        let lambda_func_name = fresh_global_name(
            &self.current_symbol,
            CLOSURE_LAM_SUFFIX,
            &mut self.lam_func_counter,
            &mut self.global_names,
        );

        // Build the capture list struct that threads the captured environment into the lifted
        // function.
        let cap = CaptureStruct::new(CAP_LIST_PREFIX, &lambda_func_name, &cap_names_types);
        let cap_list_expr = cap.struct_expr();

        let func = lifted_lambda_func(&cap, &lam);
        let func_ty = func.type_.as_ref().unwrap().clone();
        self.new_symbols.push(Symbol {
            name: lambda_func_name.clone(),
            generic_name: lambda_func_name.clone(),
            ty: func_ty.clone(),
            expr: Some(func),
        });
        self.lifted
            .borrow_mut()
            .insert(lambda_func_name.clone(), cap, func_ty);
        (Tree::leaf(lambda_func_name), cap_list_expr)
    }
}

// Information of specializable functions
#[derive(Clone)]
struct SpecializableFunctionInfo {
    // The ways into this function a copy is worth making for.
    specializable_slots: Set<Slot>,
}

// A copy the program asks for.
struct SpecializationRequest {
    // The copy asked for.
    unit: UnitKey,
    // The type of the function the copy is made from.
    org_func_ty: Arc<TypeNode>,
    // What the chain producing this request has committed to. The copy carries it on, so the
    // requests raised while walking its body are judged against the same chain.
    pinned: Pinned,
}

impl SpecializationRequest {
    // Create the type of the specialized function
    fn specialized_func_ty(&self, lifted: &mut LiftedLambdas) -> Arc<TypeNode> {
        // Decompose the function type `A1 -> A2 -> ... -> An -> B` into `([A1, A2, ..., An], B)`,
        // and replace the type of each substituted argument with the type of the capture list.
        let (mut doms, codom) = self.org_func_ty.collect_app_src(usize::MAX);
        for (slot, tree) in &self.unit.subst {
            if slot.field.is_some() {
                continue;
            }
            doms[slot.arg] = lifted.capture_struct_of(tree).ty;
        }
        // A copy of a lifted lambda receives its capture list through the first argument.
        if let Some(tree) = self.unit.capture_list_tree() {
            doms[0] = lifted.capture_struct_of(&tree).ty;
        }

        // Convert back to a function type
        let mut func_ty = codom;
        for dom in doms.iter().rev() {
            func_ty = type_fun(dom.clone(), func_ty);
        }

        func_ty
    }

    // Create an expression to refer to the specialized function.
    fn specialized_func_expr(&self, lifted: &mut LiftedLambdas) -> Arc<ExprNode> {
        expr_var(self.unit.name(), None).set_type(self.specialized_func_ty(lifted))
    }
}

// The prefix of the type constructor naming a capture list this pass builds.
const CAP_LIST_PREFIX: &str = "#CapList";

// The global function a lifted lambda becomes: it receives the captured environment as an argument
// and destructures it at the head of the body.
fn lifted_lambda_func(cap: &CaptureStruct, lam: &Arc<ExprNode>) -> Arc<ExprNode> {
    let body = expr_let_typed(
        cap.pattern(),
        expr_var(FullName::local(CLOSURE_CAP_NAME), None).set_type(cap.ty.clone()),
        lam.clone(),
    );
    let func = expr_abs_typed(var_local(CLOSURE_CAP_NAME), cap.ty.clone(), body);
    internalize_let_to_var_at_head(&func)
}

// `func` applied to `args`, one argument at a time.
fn apply(func: Arc<ExprNode>, args: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
    let mut expr = func;
    for arg in args {
        expr = expr_app_typed(expr, vec![arg]);
    }
    expr
}

impl ExprVisitor for ClosureSpecializationVisitor {
    fn start_visit_var(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // If `expr` refers to a decaptured lambda, and
        // the type of this expression is T, and the lambda function type is C->T (C is the capture list type),
        // replace it with an expression that applies the lambda function to the capture list.

        // Get the name
        let name = &expr.get_var().name;

        // Check that the variable name is local.
        if !name.is_local() {
            return StartVisitResult::VisitChildren;
        }

        // Check if this name holds the capture list of a lambda this walk knows. A name bound to
        // the closure wrapped around one already has the type its uses call for.
        let known = self.local_decap_lambdas.get(name).cloned();
        if known.is_none() {
            return StartVisitResult::VisitChildren;
        }
        let known = known.unwrap();
        if !known.is_bare {
            return StartVisitResult::VisitChildren;
        }
        let tree = known.tree;

        // If the required type for this expression is already the capture list type, do nothing.
        let expr_ty = expr.type_.as_ref().unwrap().clone();
        let cap_list_ty = self.cap_of(&tree).ty;
        if expr_ty.to_string() == cap_list_ty.to_string() {
            return StartVisitResult::VisitChildren;
        }

        // Check that the required type for this expression matches the codomain of the lambda function.
        let lam = self.lambda_func_of(&tree);
        let lambda_codom_ty = lam.type_.as_ref().unwrap().get_lambda_dst();
        assert_eq!(expr_ty.to_string(), lambda_codom_ty.to_string());

        // Replace with an expression that applies the lambda function to the capture list.
        let expr = expr_app_typed(lam, vec![expr.set_type(cap_list_ty)]);
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_llvm(
        &mut self,
        llvm_expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // If any free variable in the LLVM expression refers to a decaptured lambda,
        // replace it with an expression that applies the lambda function to the capture list.

        let mut replace = Map::default(); // Data for replacing free variables in the LLVM expression
        for free_name in llvm_expr.free_vars() {
            let known = self.local_decap_lambdas.get(&free_name).cloned();
            if known.is_none() {
                continue;
            }
            let known = known.unwrap();
            if !known.is_bare {
                continue;
            }
            let tree = known.tree;

            // Create an expression that applies the lambda function to the capture list.
            let lam = self.lambda_func_of(&tree);
            let name_expr = expr_var(free_name.clone(), None).set_type(self.cap_of(&tree).ty);
            let expr = expr_app_typed(lam, vec![name_expr]);

            replace.insert(free_name.clone(), expr);
        }

        // If none of the free variables in the LLVM expression refer to a decaptured lambda, do nothing.
        if replace.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        let make_new_name = |name: &FullName| {
            let mut new_name = name.clone();
            new_name.name_as_mut().push_str(CLOSURE_CALL_LAM_SUFFIX);
            new_name
        };

        // Rename free variables in the LLVM expression
        let mut llvm_expr = llvm_expr.clone();
        let mut rename: Map<FullName, FullName> = Default::default();
        for (name, _) in replace.iter() {
            rename.insert(name.clone(), make_new_name(name));
        }
        llvm_expr = rename_free_names(&llvm_expr, rename);

        // Insert `let (new name) = (lambda function call);` before the LLVM expression
        let mut expr = llvm_expr.clone();
        for (name, call_lam_expr) in replace.iter() {
            let new_name = make_new_name(name);
            expr = expr_let_typed(
                PatternNode::make_var(var_var(new_name.clone()), None)
                    .set_type(call_lam_expr.type_.as_ref().unwrap().clone()),
                call_lam_expr.clone(),
                expr.clone(),
            );
        }

        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_llvm(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_app(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        let (func, args) = expr.destructure_app();

        // A lambda written at the call site is lifted like one bound to a name, and what the call
        // receives is its capture list wrapped back into a closure. The rules below then read its
        // identity off that wrap.
        if args.iter().any(Self::decapturable) {
            let mut lifted_args = args.clone();
            for (i, arg) in args.iter().enumerate() {
                if !Self::decapturable(arg) {
                    continue;
                }
                let (tree, cap_list) = self.decapture_lambda(arg.clone(), state); // Visits `arg` inside this call
                lifted_args[i] = expr_app_typed(self.lambda_func_of(&tree), vec![cap_list]);
            }
            return StartVisitResult::ReplaceAndRevisit(apply(func, lifted_args));
        }

        // R3: a capture list reached through the lambda that consumes it names the copy of that
        // lambda which receives the capture list as it now is. This covers the closure a capture
        // list is wrapped into and a call made through such a closure alike.
        if func.is_var() && !args.is_empty() {
            if let Some(known) = self.known_value(&args[0]) {
                let called = func.get_var().name.clone();
                if called == known.tree.lambda || called == known.tree.unit().name() {
                    let mut pinned = self.pinned.clone();
                    let known = self.narrow(known, &mut pinned);
                    self.pinned = pinned;
                    let head = self.lambda_func_of(&known.tree);
                    let type_of = |expr: &Arc<ExprNode>| expr.type_.as_ref().unwrap().to_string();
                    if head.get_var().name != called
                        || type_of(&args[0]) != type_of(&known.cap_list)
                    {
                        let mut new_args = args.clone();
                        new_args[0] = known.cap_list;
                        return StartVisitResult::ReplaceAndRevisit(apply(head, new_args));
                    }
                }
            }
        }

        // Check that `func` is a specializable global function.
        if !func.is_var() {
            return StartVisitResult::VisitChildren;
        }
        let func_name = func.get_var().name.clone();
        if !func_name.is_global() {
            return StartVisitResult::VisitChildren;
        }
        // The table is held through the loop below, which takes `self` mutably.
        let specializable_funcs = self.specializable_funcs.clone();
        let Some(specialize_info) = specializable_funcs.get(&func_name) else {
            return StartVisitResult::VisitChildren;
        };

        // R2: an argument whose identity is known is handed over as its bare capture list, where the
        // table is willing to copy the function for that way in and the stopping rule allows it.
        let mut pinned = self.pinned.clone();
        let mut specialized_args = Map::default();
        let mut new_args = args.clone();
        let mut changed = false;
        for (i, arg) in args.iter().enumerate() {
            let known = match self.known_value(arg) {
                Some(known) => known,
                None => continue,
            };
            let known = self.narrow(known, &mut pinned);
            let slot = Slot::arg(i);
            if specialize_info.specializable_slots.contains(&slot)
                && commit(&mut pinned, &func_name, slot, &known.tree)
            {
                new_args[i] = known.cap_list;
                specialized_args.insert(slot, known.tree);
                changed = true;
                continue;
            }
            // The value stays a closure here, but a narrowed capture list needs the copy that
            // receives it named. R3.
            if !known.is_bare && !self.is_up_to_date(arg, &known) {
                new_args[i] = self.value_expr(&known);
                changed = true;
            }
        }
        if specialized_args.is_empty() {
            if !changed {
                return StartVisitResult::VisitChildren;
            }
            return StartVisitResult::ReplaceAndRevisit(apply(func, new_args));
        }

        // Request the copy and call it.
        let request = SpecializationRequest {
            unit: UnitKey::new(func_name, specialized_args),
            org_func_ty: func.type_.as_ref().unwrap().clone(),
            pinned,
        };
        let head = request.specialized_func_expr(&mut self.lifted.borrow_mut());
        self.required_specializations.push(request);
        StartVisitResult::ReplaceAndRevisit(apply(head, new_args))
    }

    fn end_visit_app(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    fn start_visit_lam(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // Before visiting children, if the argument refers to a decaptured lambda, fix the domain part of the lambda type since it is incorrect.
        let arg = expr.get_lam_params();
        assert_eq!(arg.len(), 1);
        let arg = &arg[0];
        let arg_name = &arg.name;
        let cap_list_ty = match self.local_decap_lambdas.get(arg_name).cloned() {
            Some(known) if known.is_bare => self.cap_of(&known.tree).ty,
            // If the argument does not hold a capture list, do nothing.
            _ => return StartVisitResult::VisitChildren,
        };
        let lam_ty = expr.type_.as_ref().unwrap();
        let arg_ty = lam_ty.get_lambda_srcs()[0].clone();
        // If the argument type is already correct, do nothing.
        if cap_list_ty.to_string() == arg_ty.to_string() {
            return StartVisitResult::VisitChildren;
        }
        // Fix the type of this lambda expression
        let new_lambda_ty = type_fun(cap_list_ty, lam_ty.get_lambda_dst());
        let expr = expr.set_type(new_lambda_ty);
        return StartVisitResult::ReplaceAndRevisit(expr);
    }

    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        // After visiting children, the codomain type of this expression may have changed, so fix the type if necessary.
        // Example: In `expr` is a lambda `|x| |y| (...)`, if `y` is a decaptured lambda, visiting `|y| (...)` may change its type, so the codomain of `|x| |y| (...)` may need to be fixed.
        let lam_ty = expr.type_.as_ref().unwrap();
        let dom_ty = lam_ty.get_lambda_srcs()[0].clone();
        let codom_ty = lam_ty.get_lambda_dst().clone();
        let lam_body = expr.get_lam_body();
        let impl_codom_ty = lam_body.type_.as_ref().unwrap();
        if codom_ty.to_string() == impl_codom_ty.to_string() {
            return EndVisitResult::unchanged(expr);
        }
        let new_lambda_ty = type_fun(dom_ty, impl_codom_ty.clone());
        let expr = expr.set_type(new_lambda_ty);
        EndVisitResult::changed(expr)
    }

    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();
        let value = expr.get_let_value();

        // The capture list this copy receives is narrowed, so the pattern destructuring it takes the
        // narrowed types, and each narrowed field hands its identity to the name it binds.
        if self.destructures_narrowed_capture_list(&pat) {
            let narrowed = self.narrowed_capture_list.take().unwrap();
            return self.destructure_narrowed_capture_list(&narrowed, &pat, &bound, &value);
        }

        if Self::decapturable(&bound) {
            // If the bound expression is a lambda, perform decapturing.
            assert!(pat.is_var());
            let var_name = pat.get_var().name.clone();
            let (tree, cap_list) = self.decapture_lambda(bound, state); // visit `bound` inside this call
            let cap_list_ty = cap_list.type_.as_ref().unwrap().clone();
            self.local_decap_lambdas.insert(
                var_name.clone(),
                Known::bare(
                    tree,
                    expr_var(var_name, None).set_type(cap_list_ty.clone()),
                ),
            );
            let pat = pat
                .set_var_tyanno(None) // Discard type annotation since it may become incorrect
                .set_type(cap_list_ty);
            let expr = expr_let_typed(pat, cap_list, value);
            return StartVisitResult::ReplaceAndRevisit(expr);
        }

        if !pat.is_var() {
            return StartVisitResult::VisitChildren;
        }
        let known = match self.known_value(&bound) {
            Some(known) => known,
            None => return StartVisitResult::VisitChildren,
        };
        let mut pinned = self.pinned.clone();
        let known = self.narrow(known, &mut pinned);
        self.pinned = pinned;

        // A binding whose value has a known identity passes that identity to the name it binds. A
        // binding of a bare capture list lends the name the capture list itself, which later uses
        // read off it.
        let var_name = pat.get_var().name.clone();
        let value_expr = self.value_expr(&known);
        let value_ty = value_expr.type_.as_ref().unwrap().clone();
        let record = if known.is_bare {
            Known::bare(
                known.tree.clone(),
                expr_var(var_name.clone(), None).set_type(value_ty.clone()),
            )
        } else {
            known.clone()
        };
        self.local_decap_lambdas.insert(var_name, record);

        if self.is_up_to_date(&bound, &known) {
            return StartVisitResult::VisitChildren;
        }
        let pat = pat
            .set_var_tyanno(None) // Discard type annotation since it may become incorrect
            .set_type(value_ty);
        StartVisitResult::ReplaceAndRevisit(expr_let_typed(pat, value_expr, value))
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

impl ClosureSpecializationVisitor {
    // Whether `pat` is the pattern destructuring the capture list this copy receives narrowed.
    fn destructures_narrowed_capture_list(&self, pat: &Arc<PatternNode>) -> bool {
        let Some(narrowed) = self.narrowed_capture_list.as_ref() else {
            return false;
        };
        match &pat.pattern {
            Pattern::Struct(tycon, _) => tycon.as_ref() == narrowed.original.as_ref(),
            _ => false,
        }
    }

    // Retype the pattern destructuring the capture list to the narrowed one, and record what each
    // narrowed field hands to the name it binds.
    fn destructure_narrowed_capture_list(
        &mut self,
        narrowed: &NarrowedCaptureList,
        pat: &Arc<PatternNode>,
        bound: &Arc<ExprNode>,
        value: &Arc<ExprNode>,
    ) -> StartVisitResult {
        let Pattern::Struct(_, field_to_pat) = &pat.pattern else {
            unreachable!()
        };
        let cap_fields = narrowed.cap.fields();
        assert_eq!(field_to_pat.len(), cap_fields.len());

        for (position, tree) in &narrowed.fields {
            let field_name = field_to_pat[*position].2.get_var().name.clone();
            let cap_list_ty = self.cap_of(tree).ty;
            self.local_decap_lambdas.insert(
                field_name.clone(),
                Known::bare(
                    tree.clone(),
                    expr_var(field_name, None).set_type(cap_list_ty),
                ),
            );
        }

        let field_to_pat = field_to_pat
            .iter()
            .enumerate()
            .map(|(position, (name, src, pat))| {
                (
                    name.clone(),
                    src.clone(),
                    pat.set_type(cap_fields[position].1.clone()),
                )
            })
            .collect();
        let pat = pat
            .set_struct_tycon(narrowed.cap.tycon.clone())
            .set_struct_field_to_pat(field_to_pat)
            .set_type(narrowed.cap.ty.clone());
        let bound = bound.set_type(narrowed.cap.ty.clone());
        StartVisitResult::ReplaceAndRevisit(expr_let_typed(pat, bound, value.clone()))
    }
}
