use super::{
    capture_struct::{fresh_global_name, CaptureStruct},
    find_usage_of_name::{self, UsageType},
    uncurry::internalize_let_to_var_at_head,
    unique_local_names,
};
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
    graph::Graph,
    misc::{Map, Set},
    optimization::{pull_let, rename::rename_free_names},
    tool::stopwatch::StopWatch,
};
use std::{
    cell::RefCell,
    collections::VecDeque,
    hash::{Hash, Hasher},
    mem,
    rc::Rc,
    sync::Arc,
};

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
    // The way in through the argument itself, at the given position.
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

// The ways into each function a copy is worth making for, keyed by the function.
type SpecializableSlots = Map<FullName, Set<Slot>>;

// A closure value whose identity is known: which lambda it is, and which of the fields of the
// capture list it carries are themselves known.
//
// This is what a copy is keyed on, and what the type constructor of a capture list is named after,
// so that a value of that type says what to call it with.
//
// The value is shared, and carries a digest of what it says. Two fields holding the same value hold
// one copy of it, and comparing, hashing or naming a tree reads the digest instead of walking the
// tree. A relay chain that narrows two capture fields per link would otherwise build, and repeatedly
// walk, a structure that doubles per link while the program describing it grows by one function.
#[derive(Clone, Debug)]
struct ClosureTree(Rc<ClosureTreeData>);

// The contents of a tree, held behind an `Rc` so that every occurrence of one value shares them.
#[derive(Debug)]
struct ClosureTreeData {
    // The lambda decapturing lifted, which is what a call through this value reaches.
    lambda: FullName,
    // The capture fields whose own identity is known, by position, in ascending order.
    fields: Vec<(usize, ClosureTree)>,
    // What the two above say. Two trees are the same value exactly when their digests agree.
    digest: md5::Digest,
    // How many lambdas this value names, itself included. The count is stored because the trees a
    // value is built from are shared, so computing it on demand costs one step per path through the
    // value.
    lambdas: usize,
}

impl ClosureTree {
    // The value of `lambda` with the given capture fields narrowed to the values they hold.
    fn new(lambda: FullName, fields: Vec<(usize, ClosureTree)>) -> Self {
        // Both the digest below and the substitution `receiving_copy` reads off the fields are
        // functions of their order, so the same value given in two orders would name two copies of
        // one function.
        assert!(
            fields.windows(2).all(|pair| pair[0].0 < pair[1].0),
            "the narrowed capture fields of {} are given as {:?}, which is not ascending",
            lambda.to_string(),
            fields.iter().map(|(field, _)| *field).collect::<Vec<_>>()
        );
        // A field contributes its child's digest, which is of fixed width. That keeps the rendering
        // linear in this tree's own fields, and it closes each child off, so that a value nested one
        // level down reads differently from two values side by side.
        let mut hash_data = lambda.to_string();
        for (field, tree) in &fields {
            hash_data += &format!("|{}:{}", field, tree.digest_hex());
        }
        let digest = md5::compute(hash_data);
        let lambdas = 1 + fields.iter().map(|(_, tree)| tree.lambdas()).sum::<usize>();
        ClosureTree(Rc::new(ClosureTreeData {
            lambda,
            fields,
            digest,
            lambdas,
        }))
    }

    // The value of a lambda whose capture fields are all still closures.
    fn leaf(lambda: FullName) -> Self {
        ClosureTree::new(lambda, Vec::new())
    }

    // The lambda a call through this value reaches.
    fn lambda(&self) -> &FullName {
        &self.0.lambda
    }

    // The capture fields whose own identity is known, by position, in ascending order.
    fn fields(&self) -> &[(usize, ClosureTree)] {
        &self.0.fields
    }

    // How the tree reads in a name, and so in the hash that name carries.
    fn digest_hex(&self) -> String {
        format!("{:x}", self.0.digest)
    }

    // How many lambdas this value names, itself included.
    fn lambdas(&self) -> usize {
        self.0.lambdas
    }

    // The copy that receives a value of this tree: the lambda, with its known capture fields
    // substituted.
    fn receiving_copy(&self) -> FuncCopy {
        FuncCopy {
            origin: self.0.lambda.clone(),
            subst: self
                .0
                .fields
                .iter()
                .map(|(field, tree)| (Slot::capture_field(*field), tree.clone()))
                .collect(),
        }
    }
}

impl PartialEq for ClosureTree {
    // Compares the digests, each of which stands for a whole tree, so the cost stays the same
    // however deep the two values are.
    fn eq(&self, other: &Self) -> bool {
        self.0.digest == other.0.digest
    }
}
impl Eq for ClosureTree {}
impl Hash for ClosureTree {
    // Hashes the digest, so that values equal under `eq` hash alike.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.digest.hash(state);
    }
}

// One copy of a function: the function it copies, and what each of its ways in is known to receive.
// An empty substitution names the function itself.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct FuncCopy {
    // The function the copy is made from.
    origin: FullName,
    // Slots in ascending order, so that `name` is a function of the copy alone.
    subst: Vec<(Slot, ClosureTree)>,
}

impl FuncCopy {
    // The copy of `origin` whose ways in receive the given values.
    fn new(origin: FullName, subst: Map<Slot, ClosureTree>) -> Self {
        let mut subst = subst.into_iter().collect::<Vec<_>>();
        subst.sort_by_key(|(slot, _)| *slot);
        FuncCopy { origin, subst }
    }

    // The name the copy carries: the origin's, with a hash of the substitution appended. The copy
    // that substitutes nothing is the function itself and keeps the origin's name.
    fn name(&self) -> FullName {
        if self.subst.is_empty() {
            return self.origin.clone();
        }
        let mut full_name = self.origin.clone();
        let name = full_name.name_as_mut();
        *name += CLOSURE_SPEC_SUFFIX;
        let mut hash_data = String::new();
        for (slot, tree) in &self.subst {
            hash_data += &format!(",{},{}", slot.to_string(), tree.digest_hex());
        }
        *name += &format!("_{:x}", md5::compute(hash_data));
        full_name
    }

    // How many lambdas this copy names across everything it substitutes.
    fn lambdas(&self) -> usize {
        self.subst.iter().map(|(_, tree)| tree.lambdas()).sum()
    }

    // The capture list this copy receives, where it is a copy of a lifted lambda whose capture list
    // is narrowed. This is the inverse of `ClosureTree::receiving_copy`.
    fn capture_list_tree(&self) -> Option<ClosureTree> {
        let fields = self
            .subst
            .iter()
            .filter_map(|(slot, tree)| slot.field.map(|field| (field, tree.clone())))
            .collect::<Vec<_>>();
        if fields.is_empty() {
            return None;
        }
        Some(ClosureTree::new(self.origin.clone(), fields))
    }
}

// What a chain of copies has committed to, keyed by `(function, slot, lambda)`. The value is the one
// value that key may be specialized on: meeting the same key with a different value is what stops a
// recursion that wraps its closure argument on every round from asking for a copy per round. Every
// copy carries the table of the request that created it, so the walk over that copy continues the
// same chain rather than starting a fresh one.
type Pinned = Map<(FullName, Slot, FullName), ClosureTree>;

// Commit the chain reaching here to specializing `slot` of `func` on `tree`, and report whether it
// may be specialized on at all.
//
// It may not where the chain has already committed that key to a different value: that is a
// recursion handing the next round a closure built from the one it was given, and following it would
// ask for one copy per round.
fn commit(pinned: &mut Pinned, func: &FullName, slot: Slot, tree: &ClosureTree) -> bool {
    let key = (func.clone(), slot, tree.lambda().clone());
    match pinned.get(&key) {
        Some(committed) => committed == tree,
        None => {
            pinned.insert(key, tree.clone());
            true
        }
    }
}

// How many combining copies a function is allowed before any other function has asked for one, and
// how many more each function that asks brings with it.
//
// The base is twice the largest number of copies any function has in the corpus, which is four.
// None of those four is a combining copy, so the budget counts nothing there; the base is the room
// left for a program whose copies do combine before any other function has asked.
const BASE_COMBINING_COPIES: usize = 8;
const COMBINING_COPIES_PER_ASKING_FUNCTION: usize = 1;

// The copies each function has been committed to, over the whole program.
//
// The pinning table bounds the depth of a chain, and this bounds its width. The two are needed
// separately because the pinning table is a rule about one slot, and a rule about one slot leaves
// the product: a function whose closure slots are decided independently gets a copy per combination
// of them, and each combination meets a key of the pinning table it has not met before, so the
// table agrees to every one of them.
#[derive(Default)]
struct CopyBudget {
    // The copies committed to, keyed by the function they copy. Every one of them names two lambdas
    // or more.
    copies: Map<FullName, Set<FuncCopy>>,
    // Which functions asked for those copies, keyed by the function they copy. A function asked for
    // copies from more places is allowed more of them, so that a program which asks once from each
    // of many places is bounded by how much it asks for.
    askers: Map<FullName, Set<FullName>>,
}

impl CopyBudget {
    // Commit to making `copy` for the function `asked_by`, and report whether it may be made.
    //
    // A copy already committed to is always allowed. A fresh one is allowed while its origin is
    // under the allowance the functions asking for it have earned. Refusing one costs optimization
    // and can never cost correctness, whatever the allowance is: the values it would have received
    // are wrapped back into closures instead.
    //
    // **What is counted** is the copies that name two lambdas or more, which is what grows as a
    // product — across the slots of one function, and down the capture fields of one value. A copy
    // naming one lambda is one per value the program hands to that way in, so a function called with
    // a different lambda at each of a hundred sites gets a hundred of them and wants every one.
    //
    // **What the allowance scales on** is the functions that have asked, named by what they were
    // before this pass copied them. A program with many call sites spreads them over many functions
    // and pays for each; a chain that multiplies asks over and over from copies of one function, so
    // its allowance stops growing while its demands do not.
    //
    // # Arguments
    // * `asked_by` — the function whose body is being walked, before this pass copied it. Every copy
    //   of one function answers to that one name, which is what makes a chain's repeated asking
    //   count once.
    fn admit(&mut self, copy: &FuncCopy, asked_by: &FullName) -> bool {
        if copy.lambdas() < 2 {
            return true;
        }
        let committed = self.copies.entry(copy.origin.clone()).or_default();
        if committed.contains(copy) {
            return true;
        }
        let askers = self.askers.entry(copy.origin.clone()).or_default();
        askers.insert(asked_by.clone());
        let allowance = BASE_COMBINING_COPIES + COMBINING_COPIES_PER_ASKING_FUNCTION * askers.len();
        let committed = self.copies.entry(copy.origin.clone()).or_default();
        if committed.len() >= allowance {
            return false;
        }
        committed.insert(copy.clone());
        true
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
    trees: Map<FullName, ClosureTree>,
    // The capture struct each value is, which is asked for once per wrap and once per call through a
    // narrowed capture list. Deriving it walks the whole tree, so it is derived once per value.
    caps: Map<ClosureTree, CaptureStruct>,
    // The type constructors minted so far, which the caller registers into the program's type
    // environment.
    new_tycons: Map<TyCon, TyConInfo>,
}

impl LiftedLambdas {
    // Record a lambda just lifted, under the name of the global function it became.
    fn insert(&mut self, name: FullName, cap: CaptureStruct, func_ty: Arc<TypeNode>) {
        self.record_capture_list(&cap, &ClosureTree::leaf(name.clone()));
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
    fn tree_of_capture_list(&self, tycon: &FullName) -> Option<ClosureTree> {
        self.trees.get(tycon).cloned()
    }

    // The value a capture list of type `ty` carries, where `ty` is one.
    fn tree_of_capture_list_type(&self, ty: &Arc<TypeNode>) -> Option<ClosureTree> {
        self.tree_of_capture_list(&ty.toplevel_tycon()?.name)
    }

    // Whether `name` is the global function a lambda was lifted to.
    fn is_lifted(&self, name: &FullName) -> bool {
        self.lambdas.contains_key(name)
    }

    // The capture struct a value of `tree` is: the lifted lambda's, with each known field narrowed
    // to the capture struct of what it holds. The type constructor is named after the copy that
    // receives it, so the type and the tree determine each other.
    fn capture_struct_of(&mut self, tree: &ClosureTree) -> CaptureStruct {
        if let Some(cap) = self.caps.get(tree) {
            return cap.clone();
        }
        let base = &self.lambdas[tree.lambda()].cap;
        if tree.fields().is_empty() {
            return base.clone();
        }
        let mut fields = base.fields().to_vec();
        for (field, inner) in tree.fields() {
            fields[*field].1 = self.capture_struct_of(inner).ty;
        }
        let cap = CaptureStruct::new(CAP_LIST_PREFIX, &tree.receiving_copy().name(), &fields);
        self.record_capture_list(&cap, tree);
        self.caps.insert(tree.clone(), cap.clone());
        cap
    }

    // Remember the value a capture list of `cap`'s type constructor carries, and hold that type
    // constructor until it is registered.
    fn record_capture_list(&mut self, cap: &CaptureStruct, tree: &ClosureTree) {
        self.trees.insert(cap.tycon.name.clone(), tree.clone());
        self.new_tycons
            .insert(cap.tycon.as_ref().clone(), cap.tycon_info.clone());
    }

    // Take the type constructors minted so far, for the caller to register into the program's type
    // environment.
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

    let specializable_slots = Rc::new(find_specializable_slots(&prg.symbols, &lifted.borrow()));
    realize_all(prg, &lifted, specializable_slots, show_build_times);
}

// Lift every lambda in the program to a global function, until lifting one leaves nothing more to
// lift. A lambda lifted here is a global function of its own, which the next pass over the symbols
// walks in turn.
fn lift_all(prg: &mut Program, lifted: &Rc<RefCell<LiftedLambdas>>, show_build_times: bool) {
    let _sw = StopWatch::new("closure_specialization::lift_all", show_build_times);

    // Nothing is specializable during this phase, so the walk only lifts: a value whose identity is
    // known is wrapped back into a closure wherever it is used, and no request is raised.
    let nothing_specializable = Rc::new(Map::default());
    // This phase makes no copy, so no walk below draws on this.
    let budget = Rc::new(RefCell::new(CopyBudget::default()));
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
                budget.clone(),
                name.clone(),
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

    prg.type_env
        .add_tycons(lifted.borrow_mut().take_new_tycons());
}

// Make every copy the program asks for, starting from the functions themselves.
//
// The bodies every copy is made from are the ones lifting left behind, so a copy names the same
// functions its original does and the table answers for all of them.
fn realize_all(
    prg: &mut Program,
    lifted: &Rc<RefCell<LiftedLambdas>>,
    specializable_slots: Rc<SpecializableSlots>,
    show_build_times: bool,
) {
    let _sw = StopWatch::new("closure_specialization::realize_all", show_build_times);

    let originals = mem::take(&mut prg.symbols);
    let mut global_names = originals.keys().cloned().collect::<Set<_>>();
    let mut symbols: Map<FullName, Symbol> = Map::default();
    let budget = Rc::new(RefCell::new(CopyBudget::default()));

    // Every function stands for the copy of itself that substitutes nothing.
    let mut queue = originals
        .keys()
        .map(|origin| SpecializationRequest {
            func_copy: FuncCopy::new(origin.clone(), Map::default()),
            org_func_ty: originals[origin].ty.clone(),
            pinned: Pinned::default(),
        })
        .collect::<VecDeque<_>>();

    while let Some(request) = queue.pop_front() {
        let name = request.func_copy.name();
        if symbols.contains_key(&name) {
            continue;
        }
        let original = &originals[&request.func_copy.origin];
        let expr = unique_local_names::run_on_expr(original.expr.as_ref().unwrap(), Set::default());

        // A copy of a lifted lambda whose capture list is narrowed receives it through the same
        // parameter, at the narrowed type. The walk retypes the pattern destructuring it when it
        // meets that pattern.
        let narrowed = narrowed_capture_list(&request.func_copy, lifted);
        let expr = match &narrowed {
            Some(narrowed) => {
                let codom = expr.type_.as_ref().unwrap().get_lambda_dst();
                expr.set_type(type_fun(narrowed.cap.ty.clone(), codom))
            }
            None => expr,
        };
        let local_decap_lambdas = known_arguments(&request.func_copy, &expr, lifted);

        let mut visitor = ClosureSpecializationVisitor::new(
            name.clone(),
            specializable_slots.clone(),
            lifted.clone(),
            global_names.clone(),
            request.pinned.clone(),
            budget.clone(),
            request.func_copy.origin.clone(),
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
                // A copy is one more instantiation of what its origin instantiates, so it says the
                // same thing about where it came from before any instantiation.
                generic_name: original.generic_name.clone(),
                ty,
                expr: Some(trav_res.expr),
            },
        );
        global_names.insert(name);
        queue.extend(visitor.required_specializations);
    }

    prg.type_env
        .add_tycons(lifted.borrow_mut().take_new_tycons());
    prg.symbols = symbols;
}

// The capture list a copy receives in place of the one its origin was built with, where the copy is
// of a lifted lambda and narrows one of its capture fields.
fn narrowed_capture_list(
    func_copy: &FuncCopy,
    lifted: &RefCell<LiftedLambdas>,
) -> Option<NarrowedCaptureList> {
    let tree = func_copy.capture_list_tree()?;
    let original = lifted
        .borrow()
        .capture_struct(&func_copy.origin)
        .unwrap()
        .tycon
        .clone();
    let cap = lifted.borrow_mut().capture_struct_of(&tree);
    Some(NarrowedCaptureList { original, cap })
}

// What each substituted argument of `func_copy` holds, by the local name it arrives under, which is
// what the walk over `body` is told. A substituted capture field is left out: the walk learns that
// one from the pattern destructuring the capture list.
fn known_arguments(
    func_copy: &FuncCopy,
    body: &Arc<ExprNode>,
    lifted: &RefCell<LiftedLambdas>,
) -> Map<FullName, Known> {
    let mut known_args = Map::default();
    let (param_lists, _) = body.destructure_lam_sequence();
    for (slot, tree) in &func_copy.subst {
        if slot.field.is_some() {
            continue;
        }
        assert!(
            slot.arg < param_lists.len(),
            "{} is substituted at argument {}, but takes {} of them",
            func_copy.origin.to_string(),
            slot.arg,
            param_lists.len()
        );
        assert_eq!(param_lists[slot.arg].len(), 1);
        let arg_name = param_lists[slot.arg][0].name.clone();
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

// The ways into each function a copy is worth making for, over the whole program.
//
// Specialization runs forever on its own: a function `f` taking a closure `p` can build a closure
// `q` that captures `p` and call `f(q)`, so specializing `p` asks for a copy at a new type on every
// round. The table says only which ways in are worth copying for; what stops such a chain is the
// commitment `commit` records, which refuses a key already met with a different value.
fn find_specializable_slots(
    symbols: &Map<FullName, Symbol>,
    lifted: &LiftedLambdas,
) -> SpecializableSlots {
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

    let mut specializable_slots = SpecializableSlots::default();
    while let Some(idx) = queue.pop_front() {
        queued[idx] = false;
        let sym_name = call_graph.get(idx);
        let sym = symbols.get(sym_name).unwrap();
        let slots = specializable_slots_of(sym, &specializable_slots, lifted);
        if slots.is_empty() {
            continue;
        }
        let settled = specializable_slots
            .get(sym_name)
            .is_some_and(|settled| *settled == slots);
        if settled {
            continue;
        }
        specializable_slots.insert(sym_name.clone(), slots);
        for caller_idx in &callers[idx] {
            if !queued[*caller_idx] {
                queued[*caller_idx] = true;
                queue.push_back(*caller_idx);
            }
        }
    }
    specializable_slots
}

// Whether the table says a copy of `func` is worth making for `slot`.
fn is_specializable(specializable_slots: &SpecializableSlots, func: &FullName, slot: Slot) -> bool {
    specializable_slots
        .get(func)
        .is_some_and(|slots| slots.contains(&slot))
}

// Whether a value arriving under `name` inside `body` is reached without an indirect call: it is
// either called there, or handed to a way into another function that is itself specializable. That
// is what a copy gains — the call becomes direct, or the function downstream gets a known lambda and
// is copied in turn — so the size of the function holding it does not enter the judgement.
fn reaches_a_direct_call(
    name: &FullName,
    body: &Arc<ExprNode>,
    specializable_slots: &SpecializableSlots,
    lifted: &LiftedLambdas,
) -> bool {
    find_usage_of_name::run(body, name)
        .into_iter()
        .any(|usage| match usage {
            UsageType::CalledAsFunction => true,
            // A call whose callee is an expression rather than a name is one no copy can be made
            // of, so nothing arrives at a way in through it.
            UsageType::FunctionArgument(func, idx) => func
                .is_some_and(|func| is_specializable(specializable_slots, &func, Slot::arg(idx))),
            // A value captured into a lifted lambda's capture list arrives in that lambda's body
            // through the field it was stored in, which is a way in like an argument. A struct this
            // pass did not mint — one the program declares, or the capture list
            // `defunctionalize_fix` builds — carries no such way in, so the value is reached there
            // only by an indirect call.
            UsageType::CapturedInto(tycon, position) => {
                lifted.tree_of_capture_list(&tycon).is_some_and(|tree| {
                    is_specializable(
                        specializable_slots,
                        tree.lambda(),
                        Slot::capture_field(position),
                    )
                })
            }
        })
}

// The ways into `sym` a copy is worth making for, judged against the table of specializable
// functions as it stands. Adding entries to `specializable_slots` can only add slots here, never
// remove one.
fn specializable_slots_of(
    sym: &Symbol,
    specializable_slots: &SpecializableSlots,
    lifted: &LiftedLambdas,
) -> Set<Slot> {
    let expr = sym.expr.as_ref().unwrap();

    // Check if each parameter of `sym` is specializable.
    let (param_lists, body) = expr.destructure_lam_sequence();
    let params = param_lists
        .iter()
        .map(|param_list| {
            assert_eq!(param_list.len(), 1);
            param_list[0].name.clone()
        })
        .collect::<Vec<_>>();
    let param_tys = sym.ty.collect_app_src(usize::MAX).0;
    let mut slots = Set::default();
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
            && reaches_a_direct_call(param_name, &body, specializable_slots, lifted)
        {
            slots.insert(Slot::arg(param_idx));
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
                        specializable_slots,
                        lifted,
                    )
                {
                    slots.insert(Slot::capture_field(position));
                }
            }
        }
    }

    slots
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
                let field_names = field_to_pat
                    .iter()
                    .map(|(_, _, pat)| pat.get_var().name.clone())
                    .collect();
                return Some((field_names, expr.get_let_value()));
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
    // Which ways into which functions a copy is worth making for, solved once over the whole
    // program.
    specializable_slots: Rc<SpecializableSlots>,
    // Copies this walk asks for
    required_specializations: Vec<SpecializationRequest>,

    /* Fields related to name generation of lambda function */
    // Advances at each lambda this walk lifts, so that the names minted for one symbol stay
    // distinct.
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
    // How many copies of each function the program has been committed to, shared by every walk.
    budget: Rc<RefCell<CopyBudget>>,
    // The function this walk is over, before this pass copied it. Every copy of one function carries
    // the same name here.
    walking_origin: FullName,
}

// A value whose identity is known: which lambda it is with its narrowed capture fields, and where
// the bare capture list carrying it can be read from.
//
// Reading the identity of a binding never rewrites it: a rewrite would leave the wrapping rules
// something to do again, and the two would take turns undoing each other.
#[derive(Clone)]
struct Known {
    // Which lambda the value is, and which of its capture fields are themselves known.
    tree: ClosureTree,
    // An expression yielding the bare capture list, evaluable wherever the value itself is.
    cap_list: Arc<ExprNode>,
    // Whether the expression this was read off is the bare capture list rather than a closure
    // wrapped around one. Only a bare capture list has to be wrapped where a closure is called for.
    is_bare: bool,
}

impl Known {
    // A known value carried by the bare capture list `cap_list`.
    fn bare(tree: ClosureTree, cap_list: Arc<ExprNode>) -> Self {
        Known {
            tree,
            cap_list,
            is_bare: true,
        }
    }
}

impl ClosureSpecializationVisitor {
    // A walk over `current_symbol`, starting with nothing lifted and no copy asked for.
    fn new(
        current_symbol: FullName,
        specializable_slots: Rc<SpecializableSlots>,
        lifted: Rc<RefCell<LiftedLambdas>>,
        global_names: Set<FullName>,
        pinned: Pinned,
        budget: Rc<RefCell<CopyBudget>>,
        walking_origin: FullName,
    ) -> Self {
        ClosureSpecializationVisitor {
            new_symbols: Vec::new(),
            local_decap_lambdas: Map::default(),
            lifted,
            narrowed_capture_list: None,
            specializable_slots,
            required_specializations: Vec::new(),
            lam_func_counter: 0,
            current_symbol,
            global_names,
            pinned,
            budget,
            walking_origin,
        }
    }

    // The capture struct a value of `tree` is.
    fn cap_of(&self, tree: &ClosureTree) -> CaptureStruct {
        self.lifted.borrow_mut().capture_struct_of(tree)
    }

    // The function a value of `tree` is called through, as an expression, together with its type.
    // For a tree whose fields are all still closures this is the lifted lambda itself; otherwise it
    // is the copy that receives the narrowed capture list.
    fn lambda_func_of(&self, tree: &ClosureTree) -> Arc<ExprNode> {
        let base = self.lifted.borrow().func_ty(tree.lambda());
        let (mut doms, codom) = base.collect_app_src(usize::MAX);
        doms[0] = self.cap_of(tree).ty;
        expr_var(tree.receiving_copy().name(), None).set_type(fun_ty(&doms, codom))
    }

    // The expression a known value is carried by: the bare capture list, or the closure wrapping it.
    // A wrap names the copy of the lambda that receives the capture list as it now is.
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
        if known.is_bare {
            return is_same_type(expr, &value);
        }
        let (func, args) = expr.destructure_app();
        let (value_func, value_args) = value.destructure_app();
        func.is_var()
            && func.get_var().name == value_func.get_var().name
            && is_same_type(&args[0], &value_args[0])
    }

    // The value `name` holds, where its identity is known and it is carried by a bare capture list.
    fn known_bare_value(&self, name: &FullName) -> Option<Known> {
        let known = self.local_decap_lambdas.get(name)?;
        if !known.is_bare {
            return None;
        }
        Some(known.clone())
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
        let name = func.get_var().name.clone();
        if let Some(known) = self.known_value(&args[0]) {
            if name != *known.tree.lambda() && name != known.tree.receiving_copy().name() {
                return None;
            }
            return Some(Known {
                is_bare: false,
                ..known
            });
        }
        // The capture list behind the wrap carries no identity of its own, so the lambda the wrap
        // names is all there is to say.
        if !self.lifted.borrow().is_lifted(&name) {
            return None;
        }
        Some(Known {
            tree: ClosureTree::leaf(name),
            cap_list: args[0].clone(),
            is_bare: false,
        })
    }

    // Narrow the capture list a known value is carried by, where it is built here: a field the table
    // is willing to specialize on takes the capture list of the value it holds, in place of the
    // closure.
    //
    // Creating a narrowed value asks for the copy of the lambda that receives it, so that the value
    // can be wrapped back into a closure wherever one is called for.
    //
    // The commitments the narrowing makes are recorded in `pinned` only once the copy it calls for
    // is one the budget allows, so that a narrowing the budget refuses leaves no trace of a
    // commitment the chain never made.
    fn narrow(&mut self, known: Known, pinned: &mut Pinned) -> Known {
        let mut fields = match known.cap_list.destructure_make_struct() {
            Some((_, fields)) => fields.clone(),
            None => return known,
        };
        // The table is held through the loop below, which takes `self` mutably.
        let specializable_slots = self.specializable_slots.clone();
        let Some(slots) = specializable_slots.get(known.tree.lambda()) else {
            return known;
        };
        let mut pinned_with_narrowing = pinned.clone();
        let mut narrowed_fields = Vec::new();
        for (position, (_, _, value)) in fields.iter_mut().enumerate() {
            let slot = Slot::capture_field(position);
            if !slots.contains(&slot) {
                continue;
            }
            let known_field = match self.known_value(value) {
                Some(known_field) => known_field,
                None => continue,
            };
            if !commit(
                &mut pinned_with_narrowing,
                known.tree.lambda(),
                slot,
                &known_field.tree,
            ) {
                continue;
            }
            *value = known_field.cap_list;
            narrowed_fields.push((position, known_field.tree));
        }
        // A value arrives already narrowed where a copy passes it on, and the narrowing here is
        // rebuilt from the fields. A field the value already narrowed has to survive that rebuild:
        // the type it is declared at says it is narrowed, and dropping it from the tree would leave
        // the two disagreeing.
        assert!(
            known.tree.fields().iter().all(|(position, tree)| narrowed_fields
                .iter()
                .any(|(narrowed_position, narrowed_tree)| narrowed_position == position
                    && narrowed_tree == tree)),
            "field {:?} of {} arrived narrowed and is dropped by the narrowing in {}, which keeps {:?}",
            known.tree.fields().iter().map(|(position, _)| *position).collect::<Vec<_>>(),
            known.tree.lambda().to_string(),
            self.current_symbol.to_string(),
            narrowed_fields.iter().map(|(position, _)| *position).collect::<Vec<_>>()
        );
        if narrowed_fields == known.tree.fields() {
            return known;
        }

        let tree = ClosureTree::new(known.tree.lambda().clone(), narrowed_fields);
        if !self
            .budget
            .borrow_mut()
            .admit(&tree.receiving_copy(), &self.walking_origin)
        {
            return known;
        }
        *pinned = pinned_with_narrowing;
        self.request_lambda_copy(&tree, pinned);
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

    // Narrow `known` along the chain this walk carries, so that what the narrowing commits to is
    // what the requests raised later in this walk are judged against.
    fn narrow_and_pin(&mut self, known: Known) -> Known {
        let mut pinned = self.pinned.clone();
        let known = self.narrow(known, &mut pinned);
        self.pinned = pinned;
        known
    }

    // Ask for the copy of the lambda that receives a capture list of `tree`. Asking where the tree is
    // created is what makes wrapping a value of it back into a closure legal everywhere.
    fn request_lambda_copy(&mut self, tree: &ClosureTree, pinned: &Pinned) {
        let func_copy = tree.receiving_copy();
        if func_copy.subst.is_empty() {
            return;
        }
        let org_func_ty = self.lifted.borrow().func_ty(tree.lambda());
        self.required_specializations.push(SpecializationRequest {
            func_copy,
            org_func_ty,
            pinned: pinned.clone(),
        });
    }

    // Whether `expr` is a lambda whose captured environment can be read off it, so that it can be
    // lifted to a global function taking that environment as an argument. The free variables of an
    // expression leave `CAP_NAME` out, so a body that reads it captures more than this can see.
    fn decapturable(expr: &Arc<ExprNode>) -> bool {
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
        lam: Arc<ExprNode>,
        state: &mut VisitState,
    ) -> (ClosureTree, Arc<ExprNode>) {
        // Get the capture list.
        let cap_names = lam.lambda_cap_names();

        // For each captured name, get the type the field holding it is declared at. A name holding a
        // bare capture list is captured at the closure type it wraps back into, so that every
        // capture field is declared at a type narrowing leaves alone — which is what makes wrapping
        // available at a field whose value will not be narrowed. The construction below reads such a
        // name at the closure type, and the wrapping rule repairs it when the rewritten expression is
        // visited again.
        let cap_names_types = cap_names
            .iter()
            .map(|name| {
                let ty = match self.local_decap_lambdas.get(name).cloned() {
                    Some(known) if known.is_bare => self
                        .lambda_func_of(&known.tree)
                        .type_
                        .as_ref()
                        .unwrap()
                        .get_lambda_dst(),
                    _ => state.scope.get_local(&name.name).unwrap().unwrap().clone(),
                };
                (name.clone(), ty)
            })
            .collect::<Vec<_>>();

        // A capture field declared at a capture list type has no wrap to fall back on, so a value
        // stored in one has to follow wherever that value's type goes. Keeping every field at a type
        // narrowing leaves alone is what lets each field be decided on its own.
        for (name, ty) in &cap_names_types {
            assert!(
                self.lifted.borrow().tree_of_capture_list_type(ty).is_none(),
                "the capture field `{}` of a lambda lifted in {} is declared at the capture list \
                 type {}",
                name.to_string(),
                self.current_symbol.to_string(),
                ty.to_string()
            );
        }

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
        (ClosureTree::leaf(lambda_func_name), cap_list_expr)
    }
}

// A copy the program asks for.
struct SpecializationRequest {
    // The copy asked for.
    func_copy: FuncCopy,
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
        for (slot, tree) in &self.func_copy.subst {
            if slot.field.is_some() {
                continue;
            }
            doms[slot.arg] = lifted.capture_struct_of(tree).ty;
        }
        // A copy of a lifted lambda receives its capture list through the first argument.
        if let Some(tree) = self.func_copy.capture_list_tree() {
            doms[0] = lifted.capture_struct_of(&tree).ty;
        }

        fun_ty(&doms, codom)
    }

    // Create an expression to refer to the specialized function.
    fn specialized_func_expr(&self, lifted: &mut LiftedLambdas) -> Arc<ExprNode> {
        expr_var(self.func_copy.name(), None).set_type(self.specialized_func_ty(lifted))
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

// The function type taking `doms` in order and returning `codom`, which is what `collect_app_src`
// takes apart.
fn fun_ty(doms: &[Arc<TypeNode>], codom: Arc<TypeNode>) -> Arc<TypeNode> {
    let mut ty = codom;
    for dom in doms.iter().rev() {
        ty = type_fun(dom.clone(), ty);
    }
    ty
}

// `func` applied to `args`, one argument at a time.
fn apply(func: Arc<ExprNode>, args: Vec<Arc<ExprNode>>) -> Arc<ExprNode> {
    let mut expr = func;
    for arg in args {
        expr = expr_app_typed(expr, vec![arg]);
    }
    expr
}

// Whether `left` and `right` carry the same type, compared as the two types render.
fn is_same_type(left: &Arc<ExprNode>, right: &Arc<ExprNode>) -> bool {
    let type_string = |expr: &Arc<ExprNode>| expr.type_.as_ref().unwrap().to_string();
    type_string(left) == type_string(right)
}

impl ExprVisitor for ClosureSpecializationVisitor {
    // Wrap a name holding a bare capture list back into the closure its use here calls for: where
    // the use wants a value of type `T` and the lifted lambda has type `C -> T` for the capture list
    // type `C`, the name is replaced by that global function applied to it.
    fn start_visit_var(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // Get the name
        let name = &expr.get_var().name;

        // Check that the variable name is local.
        if !name.is_local() {
            return StartVisitResult::VisitChildren;
        }

        // Check if this name holds the capture list of a lambda this walk knows. A name bound to
        // the closure wrapped around one already has the type its uses call for.
        let Some(known) = self.known_bare_value(name) else {
            return StartVisitResult::VisitChildren;
        };
        let tree = known.tree;

        // If the required type for this expression is already the capture list type, do nothing.
        let expr_ty = expr.type_.as_ref().unwrap().clone();
        let cap_list_ty = self.cap_of(&tree).ty;
        if expr_ty.to_string() == cap_list_ty.to_string() {
            return StartVisitResult::VisitChildren;
        }

        // Check that the required type for this expression matches the codomain of the lambda function.
        let lam_func = self.lambda_func_of(&tree);
        let lambda_codom_ty = lam_func.type_.as_ref().unwrap().get_lambda_dst();
        assert_eq!(expr_ty.to_string(), lambda_codom_ty.to_string());

        // Replace with an expression that applies the lambda function to the capture list.
        let expr = expr_app_typed(lam_func, vec![expr.set_type(cap_list_ty)]);
        StartVisitResult::ReplaceAndRevisit(expr)
    }

    fn end_visit_var(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        EndVisitResult::unchanged(expr)
    }

    // Give an inline-LLVM expression the closures it is written against: each free variable holding
    // a bare capture list is bound, ahead of the expression, to the lifted lambda applied to that
    // capture list, and the expression reads the binding under the name `CLOSURE_CALL_LAM_SUFFIX`
    // gives it.
    fn start_visit_llvm(
        &mut self,
        llvm_expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        // The expression each free variable holding a capture list is replaced with.
        let mut replacements = Map::default();
        for free_name in llvm_expr.free_vars() {
            let Some(known) = self.known_bare_value(&free_name) else {
                continue;
            };
            let tree = known.tree;

            // Create an expression that applies the lambda function to the capture list.
            let lam_func = self.lambda_func_of(&tree);
            let name_expr = expr_var(free_name.clone(), None).set_type(self.cap_of(&tree).ty);
            let expr = expr_app_typed(lam_func, vec![name_expr]);

            replacements.insert(free_name.clone(), expr);
        }

        // If none of the free variables in the LLVM expression refer to a decaptured lambda, do nothing.
        if replacements.is_empty() {
            return StartVisitResult::VisitChildren;
        }

        let make_new_name = |name: &FullName| {
            let mut new_name = name.clone();
            new_name.name_as_mut().push_str(CLOSURE_CALL_LAM_SUFFIX);
            new_name
        };

        // Rename free variables in the LLVM expression
        let mut llvm_expr = llvm_expr.clone();
        let mut renames: Map<FullName, FullName> = Default::default();
        for (name, _) in replacements.iter() {
            renames.insert(name.clone(), make_new_name(name));
        }
        llvm_expr = rename_free_names(&llvm_expr, renames);

        // Insert `let (new name) = (lambda function call);` before the LLVM expression
        let mut expr = llvm_expr.clone();
        for (name, call_lam_expr) in replacements.iter() {
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

    // Lift a lambda written among the arguments, name the copy of a lambda a call through a known
    // value reaches, and specialize the called function on the arguments whose identity is known,
    // where it is a specializable global.
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

        // A capture list reached through the lambda that consumes it names the copy of that lambda
        // which receives the capture list as it now is. This covers the closure a capture list is
        // wrapped into and a call made through such a closure alike.
        if func.is_var() && !args.is_empty() {
            if let Some(known) = self.known_value(&args[0]) {
                let called = func.get_var().name.clone();
                if called == *known.tree.lambda() || called == known.tree.receiving_copy().name() {
                    let known = self.narrow_and_pin(known);
                    let head = self.lambda_func_of(&known.tree);
                    if head.get_var().name != called || !is_same_type(&args[0], &known.cap_list) {
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
        let specializable_slots = self.specializable_slots.clone();
        let Some(slots) = specializable_slots.get(&func_name) else {
            return StartVisitResult::VisitChildren;
        };

        // An argument whose identity is known is handed over as its bare capture list, where the
        // table is willing to copy the function for that way in and the stopping rule allows it.
        let mut pinned = self.pinned.clone();
        let mut known_args = Vec::new();
        let mut subst = Map::default();
        let mut specialized_args = Set::default();
        for (i, arg) in args.iter().enumerate() {
            let known = match self.known_value(arg) {
                Some(known) => known,
                None => continue,
            };
            let known = self.narrow(known, &mut pinned);
            let slot = Slot::arg(i);
            if slots.contains(&slot) && commit(&mut pinned, &func_name, slot, &known.tree) {
                subst.insert(slot, known.tree.clone());
                specialized_args.insert(i);
            }
            known_args.push((i, known));
        }

        // The copy those arguments call for is made only where the budget has room for one more copy
        // of this function. Past that the call hands over every argument as a closure, which is the
        // shape it would have had were none of their identities known.
        let func_copy = FuncCopy::new(func_name, subst);
        if !self
            .budget
            .borrow_mut()
            .admit(&func_copy, &self.walking_origin)
        {
            specialized_args.clear();
        }

        let mut new_args = args.clone();
        let mut changed = false;
        for (i, known) in known_args {
            if specialized_args.contains(&i) {
                new_args[i] = known.cap_list;
                changed = true;
                continue;
            }
            // The value stays a closure here, but a narrowed capture list needs the copy that
            // receives it named.
            if !known.is_bare && !self.is_up_to_date(&args[i], &known) {
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
            func_copy,
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

    // Retype the domain of a lambda whose parameter holds a bare capture list, so that the body is
    // walked against the type that parameter now has.
    fn start_visit_lam(
        &mut self,
        expr: &Arc<ExprNode>,
        _state: &mut VisitState,
    ) -> StartVisitResult {
        let params = expr.get_lam_params();
        assert_eq!(params.len(), 1);
        let param = &params[0];
        let param_name = &param.name;
        let cap_list_ty = match self.local_decap_lambdas.get(param_name).cloned() {
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

    // Retype the codomain of a lambda whose body changed type while it was walked.
    //
    // In `|x| |y| (...)` where `y` holds a bare capture list, walking `|y| (...)` retypes its
    // domain, and the codomain of the outer lambda follows it.
    fn end_visit_lam(&mut self, expr: &Arc<ExprNode>, _state: &mut VisitState) -> EndVisitResult {
        let lam_ty = expr.type_.as_ref().unwrap();
        let dom_ty = lam_ty.get_lambda_srcs()[0].clone();
        let codom_ty = lam_ty.get_lambda_dst().clone();
        let lam_body = expr.get_lam_body();
        let body_ty = lam_body.type_.as_ref().unwrap();
        if codom_ty.to_string() == body_ty.to_string() {
            return EndVisitResult::unchanged(expr);
        }
        let new_lambda_ty = type_fun(dom_ty, body_ty.clone());
        let expr = expr.set_type(new_lambda_ty);
        EndVisitResult::changed(expr)
    }

    // Lift a lambda bound by this `let`, and record the bound name in `local_decap_lambdas` so that
    // later uses of it are read as the value the binding now holds. A binding whose value already
    // has a known identity passes that reading on to the new name.
    fn start_visit_let(
        &mut self,
        expr: &Arc<ExprNode>,
        state: &mut VisitState,
    ) -> StartVisitResult {
        let pat = expr.get_let_pat();
        let bound = expr.get_let_bound();
        let value = expr.get_let_value();

        // The capture list this copy receives is narrowed, so the pattern destructuring it takes the
        // narrowed types.
        if self.destructures_narrowed_capture_list(&pat) {
            let narrowed = self.narrowed_capture_list.take().unwrap();
            return self.destructure_narrowed_capture_list(&narrowed, &pat, &bound, &value);
        }
        // A field of a capture list that holds another capture list hands that value's identity to
        // the name it binds. The field's type is what says so: a capture list's type constructor is
        // one-to-one with the value it carries.
        self.record_capture_list_fields(&pat);

        if Self::decapturable(&bound) {
            // If the bound expression is a lambda, perform decapturing.
            assert!(pat.is_var());
            let var_name = pat.get_var().name.clone();
            let (tree, cap_list) = self.decapture_lambda(bound, state); // visit `bound` inside this call
            let cap_list_ty = cap_list.type_.as_ref().unwrap().clone();
            self.local_decap_lambdas.insert(
                var_name.clone(),
                Known::bare(tree, expr_var(var_name, None).set_type(cap_list_ty.clone())),
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
        let known = self.narrow_and_pin(known);

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

    // Hand the identity of every field that holds a capture list to the name that field binds, where
    // `pat` destructures a capture list this pass built.
    fn record_capture_list_fields(&mut self, pat: &Arc<PatternNode>) {
        let Pattern::Struct(tycon, field_to_pat) = &pat.pattern else {
            return;
        };
        if self
            .lifted
            .borrow()
            .tree_of_capture_list(&tycon.name)
            .is_none()
        {
            return;
        }
        for (_, _, field_pat) in field_to_pat {
            let field_ty = field_pat.info.type_.as_ref().unwrap();
            let Some(tree) = self.lifted.borrow().tree_of_capture_list_type(field_ty) else {
                continue;
            };
            let field_name = field_pat.get_var().name.clone();
            let cap_list = expr_var(field_name.clone(), None).set_type(field_ty.clone());
            self.local_decap_lambdas
                .insert(field_name, Known::bare(tree, cap_list));
        }
    }

    // Retype the pattern destructuring the capture list to the narrowed one.
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

// Values a copy is keyed on that stand for different things, and the distinct names they mint. Two
// values a key or a name ran together would hand one copy to both.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::INSTANCIATED_NAME_SEPARATOR;

    /// The name of the global function the `index`-th lambda of `Main::main` was lifted to.
    fn lifted(index: u32) -> FullName {
        FullName::from_strs(
            &["Main"],
            &format!(
                "main{}0123abcd{}{}",
                INSTANCIATED_NAME_SEPARATOR, CLOSURE_LAM_SUFFIX, index
            ),
        )
    }

    /// The values two capture fields of one lambda hold read differently from the one value the
    /// first field holds when it is itself narrowed: `M{0:P, 1:Q}` against `M{0:P{1:Q}}`. Where a
    /// rendering runs one field into the next these read alike, and one copy then carries both
    /// names.
    #[test]
    fn a_value_nested_one_level_down_differs_from_two_side_by_side() {
        let (m, p, q) = (lifted(0), lifted(1), lifted(2));
        let side_by_side = ClosureTree::new(
            m.clone(),
            vec![
                (0, ClosureTree::leaf(p.clone())),
                (1, ClosureTree::leaf(q.clone())),
            ],
        );
        let nested = ClosureTree::new(
            m,
            vec![(0, ClosureTree::new(p, vec![(1, ClosureTree::leaf(q))]))],
        );
        assert_ne!(side_by_side, nested);
        assert_ne!(
            side_by_side.receiving_copy().name(),
            nested.receiving_copy().name()
        );
    }

    /// Which field a value arrives through is part of what the copy receiving it is told, since
    /// that copy reads the capture list by position.
    #[test]
    fn the_position_of_a_narrowed_field_is_part_of_the_value() {
        let (m, p) = (lifted(0), lifted(1));
        let at_first = ClosureTree::new(m.clone(), vec![(0, ClosureTree::leaf(p.clone()))]);
        let at_second = ClosureTree::new(m, vec![(1, ClosureTree::leaf(p))]);
        assert_ne!(at_first, at_second);
        assert_ne!(
            at_first.receiving_copy().name(),
            at_second.receiving_copy().name()
        );
    }

    /// A relay chain narrows one link per step, so a value differs from the value one link shorter
    /// and from the one holding a different lambda at the far end.
    #[test]
    fn a_chain_of_narrowed_fields_says_its_own_depth() {
        let (m, p, q) = (lifted(0), lifted(1), lifted(2));
        let one = ClosureTree::new(m.clone(), vec![(0, ClosureTree::leaf(p.clone()))]);
        let two = ClosureTree::new(m.clone(), vec![(0, one.clone())]);
        let three = ClosureTree::new(m.clone(), vec![(0, two.clone())]);
        let other_end = ClosureTree::new(
            m,
            vec![(0, ClosureTree::new(p, vec![(0, ClosureTree::leaf(q))]))],
        );
        let names = [&one, &two, &three, &other_end]
            .iter()
            .map(|tree| tree.receiving_copy().name().to_string())
            .collect::<Set<_>>();
        assert_eq!(names.len(), 4);
    }

    /// The copy a value is received by and the value itself determine each other, which is what
    /// lets the type of a capture list say what to call it with.
    #[test]
    fn a_copy_and_the_capture_list_it_receives_determine_each_other() {
        let (m, p, q) = (lifted(0), lifted(1), lifted(2));
        let tree = ClosureTree::new(
            m,
            vec![
                (
                    0,
                    ClosureTree::new(p, vec![(2, ClosureTree::leaf(q.clone()))]),
                ),
                (1, ClosureTree::leaf(q)),
            ],
        );
        assert_eq!(
            tree.receiving_copy().capture_list_tree(),
            Some(tree.clone())
        );
    }

    /// An argument and a capture field of the same index are two ways into a function, and the copy
    /// made for one is not the copy made for the other.
    #[test]
    fn an_argument_and_a_capture_field_of_the_same_index_name_two_copies() {
        let func = FullName::from_strs(&["Main"], "f#0123abcd");
        let tree = ClosureTree::leaf(lifted(0));
        let on_argument = FuncCopy::new(
            func.clone(),
            [(Slot::arg(0), tree.clone())].into_iter().collect(),
        );
        let on_capture_field =
            FuncCopy::new(func, [(Slot::capture_field(0), tree)].into_iter().collect());
        assert_ne!(on_argument.name(), on_capture_field.name());
    }
}
