//! Debug-only well-formedness checks for the RC IR.
//!
//! [`validate`] runs after each RC-IR-rewriting pass during compiler development (gated on
//! `Configuration::develop_mode`; it is never run in a normal `fix` build). It catches a malformed
//! rewrite — one that leaves a dangling variable use or duplicates a binding name — at the pass that
//! produced it, on any input. That closes a gap the runtime checks leave: valgrind and a uniqueness
//! assertion need a triggering input and reachable code, whereas this is static and total.
//!
//! It checks the structural invariants of the RC IR: within each function every bound name is
//! unique (no shadowing), every variable use resolves to a binding in scope, or to a global — a
//! function or a global value, both referenceable by name (a direct call's callee is a function
//! name, not a local binding) — every `Retain`/`Release` names one reference-counting unit of its
//! variable; a function carries a capture parameter exactly for the closure ABI; every match has at
//! least one arm, with any catch-all arm last; an `Llvm` operation's embedded operand names match
//! its argument list; and a closure value stores the capture layout its target function projects.
//!
//! It then checks the reference counting itself against the shared consume model (`ownership`):
//! along every path each object unit is disposed of exactly as often as it is held, the arms of a
//! match agree on what they leave counted, and no value is read after its last reference is
//! consumed. That the model this reads is the one the passes write is the point: a rewrite whose
//! reference counting contradicts what code generation will do fails here, at the pass that made it.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::fixstd::builtin::InlineLLVMCaptureProjectBody;
use crate::misc::{grow_stack, Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    FieldPath, FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar, VarPath,
};
use crate::rc_ir::ownership::{
    all_owned_units, boxed_leaves, destructure_consumes, is_unboxed_union_unit, rc_units,
    rhs_consumes, truncate_to_unit, unit_key, VarTable,
};
use colored::Color;
use std::sync::Arc;

/// Check the well-formedness of every function and global, panicking on the first violation. A
/// violation is an internal compiler error — the RC IR is malformed — so it aborts rather than
/// returns. `stage` names the pass just run, so a failure points at the culprit.
///
/// `symbol_names` is every symbol name in the whole program. A use naming one refers to a global
/// function or value (a direct call's callee, a funptr atom, or a global operand) — one this
/// compilation unit may not define, since separated compilation splits the program across units — and
/// code generation materializes it, so it is always in scope. Local names are globally-unique fresh
/// names, so admitting the symbol names never masks a dangling local.
pub fn validate(prog: &RcProgram, symbol_names: &Set<FullName>, type_env: &TypeEnv, stage: &str) {
    // The globally-referenceable names: every program symbol, plus this program's own functions and
    // globals — which include the clones borrow-ification and specialization mint (not program
    // symbols) and any unit-local function.
    let mut globals = symbol_names.clone();
    for f in prog.funcs.keys() {
        globals.insert(f.name.clone());
    }
    for g in &prog.globals {
        globals.insert(g.symbol.clone());
    }
    let capture_layouts = capture_layouts(prog, stage);

    for func in prog.funcs.values() {
        // A capture parameter is present exactly for the closure ABI: it is the trailing capture-
        // pointer parameter a closure projects its captures from, and the funptr ABI has none. A clone
        // that copies the arrow type but sets the wrong capture would mis-lower the ABI.
        if func.capture.is_some() != func.fn_ty.is_closure() {
            panic!(
                "[RC IR validate] {}: `{}` capture-present={} disagrees with closure-ABI={}",
                stage,
                func.name.name.to_string(),
                func.capture.is_some(),
                func.fn_ty.is_closure(),
            );
        }
        let mut v = Validator::new(
            stage,
            &globals,
            prog,
            &capture_layouts,
            type_env,
            func.name.name.to_string(),
        );
        for p in func.params.iter().chain(func.capture.iter()) {
            v.bind(&p.name);
        }
        v.check_expr(&func.body);
    }
    for g in &prog.globals {
        let mut v = Validator::new(
            stage,
            &globals,
            prog,
            &capture_layouts,
            type_env,
            g.symbol.to_string(),
        );
        v.check_expr(&g.init);
    }

    check_reference_counting(prog, type_env, stage);
}

/// The capture-object layout each function's capture projections read: the field types every
/// projection of that function records, which is the layout a closure value targeting it must store.
/// A function that projects no capture has no entry — it reads nothing, so any layout suits it.
///
/// Checking the projections of a function against each other, and against its capture parameter,
/// happens here, where they are gathered.
fn capture_layouts(prog: &RcProgram, stage: &str) -> Map<FuncRef, Vec<Arc<TypeNode>>> {
    let mut out = Map::default();
    for func in prog.funcs.values() {
        let mut layout: Option<Vec<Arc<TypeNode>>> = None;
        for_each_rhs(&func.body, &mut |rhs| {
            let RcRhs::Llvm(llvm_gen, _) = rhs else {
                return;
            };
            let Some(proj) = llvm_gen
                .as_any()
                .downcast_ref::<InlineLLVMCaptureProjectBody>()
            else {
                return;
            };
            check_capture_projection(func, proj, layout.as_ref(), stage);
            layout = Some(proj.cap_tys.clone());
        });
        if let Some(layout) = layout {
            out.insert(func.name.clone(), layout);
        }
    }
    out
}

/// A capture projection reads slot `cap_idx` of the function's capture object, whose layout it
/// carries as `cap_tys`. It must read the function's own capture parameter, name a slot that layout
/// has, and agree on the layout with the function's other projections — they are copies of one list,
/// so a rewrite that retyped or reordered the captures of one projection alone would leave the rest
/// reading the old layout.
fn check_capture_projection(
    func: &RcFunc,
    proj: &InlineLLVMCaptureProjectBody,
    layout: Option<&Vec<Arc<TypeNode>>>,
    stage: &str,
) {
    let location = func.name.name.to_string();
    let capture = func.capture.as_ref().map(|c| &c.name);
    if capture != Some(&proj.cap_name) {
        panic!(
            "[RC IR validate] {}: capture projection reads `{}`, which is not the capture parameter of `{}`",
            stage,
            proj.cap_name.to_string(),
            location,
        );
    }
    if proj.cap_idx >= proj.cap_tys.len() {
        panic!(
            "[RC IR validate] {}: capture projection reads slot {} of a {}-slot capture in `{}`",
            stage,
            proj.cap_idx,
            proj.cap_tys.len(),
            location,
        );
    }
    if let Some(layout) = layout {
        if *layout != proj.cap_tys {
            panic!(
                "[RC IR validate] {}: capture projections of `{}` disagree on the capture layout: {:?} and {:?}",
                stage, location, layout, proj.cap_tys,
            );
        }
    }
}

/// Apply `f` to every right-hand side of an expression, arms of a match included.
fn for_each_rhs(node: &RcExprNode, f: &mut impl FnMut(&RcRhs)) {
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Let(_, rhs, k) => {
            f(rhs);
            if let RcRhs::Match(_, arms) = rhs {
                for arm in arms {
                    for_each_rhs(&arm.body, f);
                }
            }
            for_each_rhs(k, f);
        }
        RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k)
        | RcExpr::Eval(_, k)
        | RcExpr::Destructure(_, _, k) => for_each_rhs(k, f),
        RcExpr::Ret(_) => {}
    })
}

/// The per-function state: the names bound anywhere in the function (`seen`, for uniqueness) and the
/// names currently in scope (`scope`, for use resolution).
struct Validator<'a> {
    stage: &'a str,
    globals: &'a Set<FullName>,
    prog: &'a RcProgram,
    capture_layouts: &'a Map<FuncRef, Vec<Arc<TypeNode>>>,
    type_env: &'a TypeEnv,
    location: String,
    seen: Set<FullName>,
    scope: Set<FullName>,
}

impl<'a> Validator<'a> {
    fn new(
        stage: &'a str,
        globals: &'a Set<FullName>,
        prog: &'a RcProgram,
        capture_layouts: &'a Map<FuncRef, Vec<Arc<TypeNode>>>,
        type_env: &'a TypeEnv,
        location: String,
    ) -> Self {
        Validator {
            stage,
            globals,
            prog,
            capture_layouts,
            type_env,
            location,
            seen: Set::default(),
            scope: Set::default(),
        }
    }

    /// Introduce a binding: it must be unique within the function, and it enters scope.
    fn bind(&mut self, name: &FullName) {
        if !self.seen.insert(name.clone()) {
            panic!(
                "[RC IR validate] {}: duplicate binding `{}` in `{}`",
                self.stage,
                name.to_string(),
                self.location
            );
        }
        self.scope.insert(name.clone());
    }

    /// A variable use must resolve to a binding in scope or to a global (a function or global value).
    /// A `Retain`/`Release` path stops at or above a reference-counting unit of its variable — at one
    /// exactly once `split_rc_units` has run, and above one (a whole value, or a subtree holding
    /// several units) before then. Descending past a unit is what must not happen: code generation
    /// projects the path without checking it, so such a path would reference-count a part of the unit
    /// instead of the unit, or a closure's function pointer instead of its capture.
    fn check_rc_unit(&self, var: &RcVar, path: &FieldPath) {
        let units = rc_units(&var.ty, self.type_env);
        if !units.iter().any(|unit| unit.starts_with(path)) {
            panic!(
                "[RC IR validate] {}: reference counting `{}` at {:?} in `{}`, which reaches none of its units {:?}",
                self.stage,
                var.name.to_string(),
                path,
                self.location,
                units
            );
        }
    }

    fn use_var(&self, name: &FullName) {
        if !self.scope.contains(name) && !self.globals.contains(name) {
            panic!(
                "[RC IR validate] {}: use of unbound variable `{}` in `{}`",
                self.stage,
                name.to_string(),
                self.location
            );
        }
    }

    fn check_expr(&mut self, node: &RcExprNode) {
        grow_stack(|| self.check_expr_inner(node));
    }

    fn check_expr_inner(&mut self, node: &RcExprNode) {
        match node.expr.as_ref() {
            RcExpr::Let(x, rhs, k) => {
                self.check_rhs(rhs);
                self.bind(&x.name);
                self.check_expr(k);
                self.scope.remove(&x.name);
            }
            RcExpr::Retain(v, path, _, k) | RcExpr::Release(v, path, _, k) => {
                self.use_var(&v.name);
                self.check_rc_unit(v, path);
                self.check_expr(k);
            }
            // `Eval` names no RC unit — it only observes its variable — so there is no path to check.
            RcExpr::Eval(v, k) => {
                self.use_var(&v.name);
                self.check_expr(k);
            }
            RcExpr::Destructure(container, fields, k) => {
                self.use_var(&container.name);
                for (_, field) in fields {
                    self.bind(&field.name);
                }
                self.check_expr(k);
                for (_, field) in fields {
                    self.scope.remove(&field.name);
                }
            }
            RcExpr::Ret(v) => self.use_var(&v.name),
        }
    }

    fn check_rhs(&mut self, rhs: &RcRhs) {
        match rhs {
            RcRhs::Var(y) => self.use_var(&y.name),
            RcRhs::App(callee, args) => {
                self.use_var(&callee.name);
                for a in args {
                    self.use_var(&a.name);
                }
            }
            RcRhs::Closure(fref, caps) => {
                // A closure names a function of the program. A rewrite that mints a clone name and
                // forgets to add its body leaves this reference dangling, which code generation only
                // meets much later.
                if !self.prog.funcs.contains_key(fref) {
                    panic!(
                        "[RC IR validate] {}: closure targets `{}`, which is not a function of the program, in `{}`",
                        self.stage,
                        fref.name.to_string(),
                        self.location
                    );
                }
                // The closure stores its captures in this order, and the target reads them out by
                // slot index against its own copy of the layout. The two are redundant stores of one
                // layout, so a rewrite that reordered, retyped, added, or dropped the captures at one
                // end alone would leave every projection reading the wrong slot.
                if let Some(layout) = self.capture_layouts.get(fref) {
                    let stored: Vec<Arc<TypeNode>> = caps.iter().map(|c| c.ty.clone()).collect();
                    if *layout != stored {
                        panic!(
                            "[RC IR validate] {}: closure stores captures {:?} where `{}` projects {:?}, in `{}`",
                            self.stage,
                            stored,
                            fref.name.to_string(),
                            layout,
                            self.location,
                        );
                    }
                }
                for c in caps {
                    self.use_var(&c.name);
                }
            }
            RcRhs::Llvm(llvm_gen, args) => {
                // The generator embeds its operand names — code generation resolves the operands from
                // them — while the `args` list carries the same names, in the same order, for the
                // reference-counting analyses. Lowering builds one from the other and renaming rewrites
                // both, so the two stay identical; a rewrite that updated one and not the other would
                // desync what code generation reads from what the analyses track.
                let embedded_names = llvm_gen.free_vars();
                let arg_names: Vec<FullName> = args.iter().map(|a| a.name.clone()).collect();
                if embedded_names != arg_names {
                    panic!(
                        "[RC IR validate] {}: LLVM operand names {:?} disagree with argument names {:?} in `{}`",
                        self.stage,
                        embedded_names.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
                        arg_names.iter().map(|n| n.to_string()).collect::<Vec<_>>(),
                        self.location,
                    );
                }
                for a in args {
                    self.use_var(&a.name);
                }
            }
            RcRhs::Match(scrutinee, arms) => {
                self.use_var(&scrutinee.name);
                // A match has at least one arm, and a catch-all arm (`tag == None`) — which code
                // generation compiles as the tag switch's default case — is the last arm, so every
                // earlier arm names a variant. A rewrite that moved a catch-all before another arm
                // would shadow the arms after it.
                if arms.is_empty() {
                    panic!(
                        "[RC IR validate] {}: match with no arms in `{}`",
                        self.stage, self.location,
                    );
                }
                for arm in &arms[..arms.len() - 1] {
                    if arm.tag.is_none() {
                        panic!(
                            "[RC IR validate] {}: a catch-all match arm precedes a later arm in `{}`",
                            self.stage, self.location,
                        );
                    }
                }
                // Each arm's payload is in scope only within that arm's body, so bind it, check the
                // body, and unbind it before the next sibling arm.
                for arm in arms {
                    self.bind(&arm.payload.name);
                    self.check_expr(&arm.body);
                    self.scope.remove(&arm.payload.name);
                }
            }
        }
    }
}

// --- reference counting ---

/// Check the reference counting of every function and global initializer: on every path through a
/// body each object unit's count returns to where it started, and no value is read after its last
/// reference is consumed.
fn check_reference_counting(prog: &RcProgram, type_env: &TypeEnv, stage: &str) {
    let owned_units = all_owned_units(prog, type_env);
    for func in prog.funcs.values() {
        let vars = VarTable::of(func);
        let name = func.name.name.to_string();
        let mut walk = BalanceWalk::new(stage, name, &vars, prog, &owned_units, type_env);
        let entry = walk.entry_balance(func);
        walk.walk(&func.body, entry, Terminal::Return);
    }
    // A global initializer takes no input, so it starts holding nothing.
    for g in &prog.globals {
        let vars = VarTable::body_only(&g.init);
        let name = g.symbol.to_string();
        let mut walk = BalanceWalk::new(stage, name, &vars, prog, &owned_units, type_env);
        walk.walk(&g.init, Balance::default(), Terminal::Return);
    }
}

/// The unit keys an unboxed union's reference is counted under, which the balance leaves out. Such a
/// reference is whichever variant is live, so which object it belongs to is decided by the tag, while
/// `origin` — defined over boxed leaves — answers with the object the construction site laid in. On
/// the path that takes another variant, counting it would charge an object the code never holds.
fn union_keys(vars: &VarTable, type_env: &TypeEnv) -> Set<VarPath> {
    let mut out = Set::default();
    for (name, ty) in &vars.var_tys {
        for unit in rc_units(ty, type_env) {
            if !is_unboxed_union_unit(ty, &unit, type_env) {
                continue;
            }
            out.insert(unit_key(vars, type_env, name, &unit));
            // The union's leaves are its variants' payloads, and `origin` answers for each of them
            // separately: the one the construction laid in aliases the value stored, and every other
            // one — a payload of a variant this value does not hold — roots at the union itself. One
            // reference, one key per variant, so all of them are left out together.
            for leaf in boxed_leaves(ty, type_env) {
                if truncate_to_unit(ty, &leaf, type_env) == unit {
                    out.insert(unit_key(vars, type_env, name, &leaf));
                }
            }
        }
    }
    out
}

/// The reference-count walk of one function body or global initializer.
struct BalanceWalk<'a> {
    stage: &'a str,
    location: String,
    vars: &'a VarTable,
    prog: &'a RcProgram,
    owned_units: &'a Set<VarPath>,
    type_env: &'a TypeEnv,
    /// The parameter/capture units this version borrows: their references belong to the caller, so
    /// they are not counted here and driving one to zero does not end a value's life.
    borrowed: Set<VarPath>,
    /// The keys an unboxed union of this body counts its reference under (`union_keys`).
    union_keys: Set<VarPath>,
    /// The source span of the node being walked, so a failure points at the code it came from.
    source: Option<Span>,
}

/// The reference count of each object unit at a point of the walk. The key is the unit of the object
/// a reference belongs to (`unit_key`), not the binding that names it: `cancel` pairs a retain with a
/// release across bindings, so per-binding counting breaks after it.
#[derive(Clone, Default)]
struct Balance {
    /// The live count of each unit. A unit at zero is absent, so two states compare equal exactly
    /// when they hold the same counts.
    counts: Map<VarPath, i64>,
    /// The units a consume has driven to zero: reading one is a use-after-consume.
    dead: Set<VarPath>,
}

impl Balance {
    /// The live counts, rendered for a failure message.
    fn render(&self) -> String {
        let mut units: Vec<String> = self
            .counts
            .iter()
            .map(|(key, count)| format!("{} x{}", render_key(key), count))
            .collect();
        units.sort();
        format!("{{{}}}", units.join(", "))
    }
}

/// What the `Ret` closing an expression does with its value.
enum Terminal<'a> {
    /// Returns it from the function, consuming it.
    Return,
    /// Hands it to a match binding, which holds its reference from there on.
    Arm(&'a RcVar),
}

impl<'a> BalanceWalk<'a> {
    fn new(
        stage: &'a str,
        location: String,
        vars: &'a VarTable,
        prog: &'a RcProgram,
        owned_units: &'a Set<VarPath>,
        type_env: &'a TypeEnv,
    ) -> Self {
        BalanceWalk {
            stage,
            location,
            vars,
            prog,
            owned_units,
            type_env,
            borrowed: Set::default(),
            union_keys: union_keys(vars, type_env),
            source: None,
        }
    }

    /// The counts a function body starts from: an owned parameter or capture unit arrives holding one
    /// reference the body must dispose of, and a borrowed one arrives holding none — the body may
    /// only read it, or retain it first and dispose of that reference like any other.
    fn entry_balance(&mut self, func: &RcFunc) -> Balance {
        let mut entry = Balance::default();
        for p in func.params.iter().chain(func.capture.iter()) {
            for unit in rc_units(&p.ty, self.type_env) {
                let key = (p.name.clone(), unit);
                if self.owned_units.contains(&key) {
                    self.inc(&mut entry, key);
                } else {
                    self.borrowed.insert(key);
                }
            }
        }
        entry
    }

    fn walk(&mut self, node: &RcExprNode, bal: Balance, terminal: Terminal) -> Balance {
        grow_stack(|| self.walk_inner(node, bal, terminal))
    }

    fn walk_inner(&mut self, node: &RcExprNode, mut bal: Balance, terminal: Terminal) -> Balance {
        if node.source.is_some() {
            self.source = node.source.clone();
        }
        match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::Match(scrutinee, arms), k) => {
                self.read(&bal, scrutinee);
                // Each arm runs from the state before the branch and hands its value to `x`, so the
                // arms' exits differ only in which values each disposed of. They must not: a unit one
                // arm leaves counted and another does not is leaked on one path or freed twice on the
                // other, and the code after the match cannot be right for both.
                let exits: Vec<Balance> = arms
                    .iter()
                    .map(|arm| {
                        // Reading a variant out of a boxed union retains it, so the payload holds a
                        // reference of its own; the payload of an unboxed union, and of a catch-all,
                        // is the scrutinee's own value and holds none.
                        let mut entry = bal.clone();
                        self.produce(&mut entry, &arm.payload);
                        self.walk(&arm.body, entry, Terminal::Arm(x))
                    })
                    .collect();
                let mut merged = exits.first().expect("a match has at least one arm").clone();
                for exit in &exits[1..] {
                    if exit.counts != merged.counts {
                        panic!(
                            "[RC IR validate] {}: match arms leave different reference counts in `{}`: {} and {}{}",
                            self.stage,
                            self.location,
                            merged.render(),
                            exit.render(),
                            self.at(),
                        );
                    }
                    // A value one arm consumed is dead after the match: reading it is a use after
                    // consume on that path, whichever arm ran.
                    merged.dead.extend(exit.dead.iter().cloned());
                }
                self.walk(k, merged, terminal)
            }
            RcExpr::Let(x, rhs, k) => {
                for v in rhs_reads(rhs) {
                    self.read(&bal, v);
                }
                self.consume_rhs(&mut bal, rhs, &x.ty);
                self.produce(&mut bal, x);
                self.walk(k, bal, terminal)
            }
            // A reference-count node names a subtree, which holds the units at or below its path.
            RcExpr::Retain(v, path, _, k) => {
                for key in self.units_at(v, path) {
                    self.inc(&mut bal, key);
                }
                self.walk(k, bal, terminal)
            }
            RcExpr::Release(v, path, _, k) => {
                for key in self.units_at(v, path) {
                    self.dec(&mut bal, key);
                }
                self.walk(k, bal, terminal)
            }
            RcExpr::Eval(v, k) => {
                self.read(&bal, v);
                self.walk(k, bal, terminal)
            }
            RcExpr::Destructure(container, fields, k) => {
                self.read(&bal, container);
                self.consume_leaves(
                    &mut bal,
                    container,
                    destructure_consumes(container, fields, self.type_env),
                );
                for (_, field) in fields {
                    self.produce(&mut bal, field);
                }
                self.walk(k, bal, terminal)
            }
            RcExpr::Ret(v) => {
                self.read(&bal, v);
                match terminal {
                    Terminal::Return => {
                        self.consume_leaves(&mut bal, v, boxed_leaves(&v.ty, self.type_env));
                        // Every reference the body held is now disposed of: one still counted is
                        // leaked, since nothing downstream names it.
                        if !bal.counts.is_empty() {
                            panic!(
                                "[RC IR validate] {}: `{}` returns holding references it never disposes of: {}{}",
                                self.stage,
                                self.location,
                                bal.render(),
                                self.at(),
                            );
                        }
                    }
                    // The arm's value moves into the match binding: the reference it holds is counted
                    // under the binding from here on. An arm whose value already keys to the binding
                    // (every arm reaches the same object, so the binding aliases it) moves nothing.
                    Terminal::Arm(x) => {
                        // The arm's value has the binding's type, so their units correspond.
                        let from_keys = self.value_keys(&v.name, &x.ty);
                        let to_keys = self.value_keys(&x.name, &x.ty);
                        for ((_, from), (_, to)) in from_keys.into_iter().zip(to_keys) {
                            if from != to {
                                self.dec(&mut bal, from);
                                self.inc(&mut bal, to);
                            }
                        }
                    }
                }
                bal
            }
        }
    }

    /// Consume what a right-hand side consumes, by the shared consume model.
    fn consume_rhs(&self, bal: &mut Balance, rhs: &RcRhs, result_ty: &Arc<TypeNode>) {
        let owns = |p: &RcVar, pi: &FieldPath| {
            self.owned_units
                .contains(&(p.name.clone(), truncate_to_unit(&p.ty, pi, self.type_env)))
        };
        let mut consumed = vec![];
        rhs_consumes(
            rhs,
            result_ty,
            self.vars,
            self.prog,
            self.type_env,
            &owns,
            &mut consumed,
        );
        for (var, leaf) in consumed {
            let key = self.unit_key(&var, &leaf);
            self.dec(bal, key);
        }
    }

    /// Consume leaves of one value, one reference per unit they reach: the leaves of a single unit —
    /// the variants of an unboxed union — are one reference. Two units that key to the same object are
    /// two references of it, so this counts units rather than keys: a value laid into two fields of an
    /// aggregate is held twice by it.
    fn consume_leaves(&self, bal: &mut Balance, var: &RcVar, leaves: Vec<FieldPath>) {
        for (_, key) in self.unit_keys(&var.name, &var.ty, leaves) {
            self.dec(bal, key);
        }
    }

    /// Count the references a binding produces: those of its units whose object is the binding
    /// itself. A unit that keys elsewhere is an alias — a move-bind, a projection out of an unboxed
    /// aggregate, a pure `Llvm` projection — and holds a reference already counted there.
    fn produce(&self, bal: &mut Balance, x: &RcVar) {
        for (unit, key) in self.value_keys(&x.name, &x.ty) {
            if key == (x.name.clone(), unit) {
                self.inc(bal, key);
            }
        }
    }

    /// A value read: every unit it names must still hold a reference.
    fn read(&self, bal: &Balance, v: &RcVar) {
        for (_, key) in self.value_keys(&v.name, &v.ty) {
            if bal.dead.contains(&key) {
                panic!(
                    "[RC IR validate] {}: `{}` reads `{}`, whose object {} was already consumed{}",
                    self.stage,
                    self.location,
                    v.name.to_string(),
                    render_key(&key),
                    self.at(),
                );
            }
        }
    }

    fn inc(&self, bal: &mut Balance, key: VarPath) {
        if !self.is_counted(&key) {
            return;
        }
        *bal.counts.entry(key).or_default() += 1;
    }

    fn dec(&self, bal: &mut Balance, key: VarPath) {
        if !self.is_counted(&key) {
            return;
        }
        let count = bal.counts.entry(key.clone()).or_default();
        *count -= 1;
        if *count < 0 {
            panic!(
                "[RC IR validate] {}: `{}` disposes of {} more often than it holds it{}",
                self.stage,
                self.location,
                render_key(&key),
                self.at(),
            );
        }
        if *count == 0 {
            bal.counts.remove(&key);
            // The caller's reference outlives a borrowed unit, so consuming what the body retained of
            // it leaves the value readable.
            if !self.borrowed.contains(&key) {
                bal.dead.insert(key);
            }
        }
    }

    fn unit_key(&self, var: &FullName, path: &FieldPath) -> VarPath {
        unit_key(self.vars, self.type_env, var, path)
    }

    /// The keys of the units a path of a value names: those at or below it.
    fn units_at(&self, v: &RcVar, path: &FieldPath) -> Vec<VarPath> {
        self.value_keys(&v.name, &v.ty)
            .into_iter()
            .filter(|(unit, _)| unit.starts_with(path))
            .map(|(_, key)| key)
            .collect()
    }

    /// Each unit of a value of type `ty` held by `var`, as the unit path and the key its reference is
    /// counted under. A unit is reached through one of its boxed leaves, where `origin` is defined:
    /// the unit path itself names no leaf when the unit is a punched value or an unboxed union, and
    /// `origin` would answer for it as if the value were produced there.
    fn value_keys(&self, var: &FullName, ty: &Arc<TypeNode>) -> Vec<(FieldPath, VarPath)> {
        self.unit_keys(var, ty, boxed_leaves(ty, self.type_env))
    }

    /// The units a value's leaves reach, each named once: the unit path and the key its reference is
    /// counted under, taken from the first leaf that reaches the unit.
    fn unit_keys(
        &self,
        var: &FullName,
        ty: &Arc<TypeNode>,
        leaves: Vec<FieldPath>,
    ) -> Vec<(FieldPath, VarPath)> {
        let mut units: Set<FieldPath> = Set::default();
        let mut out = vec![];
        for leaf in leaves {
            let unit = truncate_to_unit(ty, &leaf, self.type_env);
            if units.insert(unit.clone()) {
                out.push((unit, self.unit_key(var, &leaf)));
            }
        }
        out
    }

    /// Whether a unit takes part in the balance. Left out are a unit rooted at a global (a global's
    /// reachable graph is refcount-exempt, so reading one mints a reference no one disposes of), a
    /// path that names no unit of its root (the hole of a punched value), and a unit an unboxed union
    /// counts its reference under (see `union_keys`).
    fn is_counted(&self, key: &VarPath) -> bool {
        let (root, unit) = key;
        let Some(ty) = self.vars.var_tys.get(root) else {
            return false;
        };
        rc_units(ty, self.type_env).contains(unit) && !self.union_keys.contains(key)
    }

    /// The source excerpt the walk is at, to place a failure in the program being compiled.
    fn at(&self) -> String {
        match &self.source {
            Some(span) => format!("\n{}", span.to_string(Color::Red)),
            None => String::new(),
        }
    }
}

fn render_key((root, unit): &VarPath) -> String {
    format!("`{}`{:?}", root.to_string(), unit)
}

/// The variables a right-hand side reads as values. A `Match` reads its scrutinee, which the walk
/// handles together with the arms.
fn rhs_reads(rhs: &RcRhs) -> Vec<&RcVar> {
    match rhs {
        RcRhs::Var(y) => vec![y],
        RcRhs::App(callee, args) => std::iter::once(callee).chain(args).collect(),
        RcRhs::Closure(_, caps) => caps.iter().collect(),
        RcRhs::Llvm(_, args) => args.iter().collect(),
        RcRhs::Match(scrutinee, _) => vec![scrutinee],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::types::{type_fun, type_funptr};
    use crate::fixstd::builtin::{
        bulitin_tycons, make_dynamic_object_ty, make_i64_ty, make_ptr_ty, InlineLLVMNullPtrLit,
    };
    use crate::rc_ir::ast::{MatchArm, RcState};

    /// A type environment holding the built-in type constructors, which a check consults to tell a
    /// boxed type from an unboxed one.
    fn type_env() -> TypeEnv {
        TypeEnv::new(bulitin_tycons(), Map::default())
    }

    fn var(name: &str) -> RcVar {
        var_of(name, make_i64_ty())
    }

    fn var_of(name: &str, ty: Arc<TypeNode>) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty,
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    fn node(expr: RcExpr) -> RcExprNode {
        RcExprNode {
            expr: Arc::new(expr),
            source: None,
        }
    }

    /// A program with no functions and no globals, for a check that names none of its own.
    fn empty_prog() -> RcProgram {
        RcProgram {
            funcs: Map::default(),
            globals: vec![],
            entry: FuncRef {
                name: FullName::local("main"),
            },
        }
    }

    /// A function returning `I64`: `fn_ty` fixes the ABI, and `capture` is the capture parameter the
    /// closure ABI takes.
    fn func(
        name: &str,
        fn_ty: Arc<TypeNode>,
        params: Vec<RcVar>,
        capture: Option<RcVar>,
        body: RcExprNode,
    ) -> RcFunc {
        RcFunc {
            name: FuncRef {
                name: FullName::local(name),
            },
            fn_ty,
            params,
            capture,
            ret_ty: make_i64_ty(),
            body,
            source: None,
            borrowed_units: Set::default(),
        }
    }

    /// Validate a program made of `funcs`, the first of which is its entry point.
    fn validate_prog(funcs: Vec<RcFunc>) {
        let entry = funcs
            .first()
            .expect("a program has at least one function")
            .name
            .clone();
        let funcs = funcs.into_iter().map(|f| (f.name.clone(), f)).collect();
        let prog = RcProgram {
            funcs,
            globals: vec![],
            entry,
        };
        validate(&prog, &Set::default(), &type_env(), "test");
    }

    /// Check `body` as a function whose only bindings in scope on entry are `params`.
    fn check(body: &RcExprNode, params: &[&str]) {
        check_with_globals(body, params, &[]);
    }

    /// Check `body` as `check` does, with `globals` as the program's global names.
    fn check_with_globals(body: &RcExprNode, params: &[&str], globals: &[&str]) {
        let globals: Set<FullName> = globals.iter().map(|g| FullName::local(g)).collect();
        let type_env = type_env();
        let prog = empty_prog();
        let capture_layouts = Map::default();
        let mut v = Validator::new(
            "test",
            &globals,
            &prog,
            &capture_layouts,
            &type_env,
            "f".to_string(),
        );
        for p in params {
            v.bind(&FullName::local(p));
        }
        v.check_expr(body);
    }

    #[test]
    fn accepts_well_formed() {
        // let x = p; ret x   (p is a parameter)
        let body = node(RcExpr::Let(
            var("x"),
            RcRhs::Var(var("p")),
            node(RcExpr::Ret(var("x"))),
        ));
        check(&body, &["p"]);
    }

    #[test]
    #[should_panic(expected = "use of unbound variable")]
    fn rejects_unbound_use() {
        // ret y   (y is never bound)
        check(&node(RcExpr::Ret(var("y"))), &[]);
    }

    #[test]
    #[should_panic(expected = "duplicate binding")]
    fn rejects_duplicate_binding() {
        // let x = p; let x = p; ret x   (x bound twice)
        let inner = node(RcExpr::Let(
            var("x"),
            RcRhs::Var(var("p")),
            node(RcExpr::Ret(var("x"))),
        ));
        let body = node(RcExpr::Let(var("x"), RcRhs::Var(var("p")), inner));
        check(&body, &["p"]);
    }

    #[test]
    fn accepts_use_of_a_global_name() {
        // let r = call g(); ret r   where g is a global (not a local binding)
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::App(var("g"), vec![]),
            node(RcExpr::Ret(var("r"))),
        ));
        check_with_globals(&body, &[], &["g"]);
    }

    #[test]
    #[should_panic(expected = "capture-present=false disagrees with closure-ABI=true")]
    fn rejects_capture_missing_for_closure_abi() {
        // A closure-typed function with no capture parameter: the closure ABI has a capture pointer.
        validate_prog(vec![func(
            "f",
            type_fun(make_i64_ty(), make_i64_ty()),
            vec![var("p")],
            None,
            node(RcExpr::Ret(var("p"))),
        )]);
    }

    #[test]
    #[should_panic(expected = "match with no arms")]
    fn rejects_empty_match_arms() {
        // let m = match s {}; ret m   (s is a parameter; the match has no arms)
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("s"), vec![]),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["s"]);
    }

    #[test]
    #[should_panic(expected = "catch-all match arm precedes a later arm")]
    fn rejects_catch_all_before_a_later_arm() {
        // let m = match s { _ -> c; 1 -> p }; ret m   (a catch-all arm before a tagged arm)
        let arms = vec![
            MatchArm {
                tag: None,
                payload: var("c"),
                body: node(RcExpr::Ret(var("c"))),
            },
            MatchArm {
                tag: Some(1),
                payload: var("p"),
                body: node(RcExpr::Ret(var("p"))),
            },
        ];
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("s"), arms),
            node(RcExpr::Ret(var("m"))),
        ));
        check(&body, &["s"]);
    }

    #[test]
    #[should_panic(expected = "capture-present=true disagrees with closure-ABI=false")]
    fn rejects_capture_present_for_funptr_abi() {
        // A funptr-typed function with a capture parameter: the funptr ABI has no capture pointer.
        validate_prog(vec![func(
            "f",
            type_funptr(vec![make_i64_ty()], make_i64_ty()),
            vec![var("p")],
            Some(var("cap")),
            node(RcExpr::Ret(var("p"))),
        )]);
    }

    #[test]
    #[should_panic(expected = "disagree with argument names")]
    fn rejects_llvm_operand_name_mismatch() {
        // let r = <nullptr op with no embedded operands>(x); ret r
        // The op's embedded operand names () disagree with the argument list (x).
        let body = node(RcExpr::Let(
            var("r"),
            RcRhs::Llvm(Box::new(InlineLLVMNullPtrLit {}), vec![var("x")]),
            node(RcExpr::Ret(var("r"))),
        ));
        check(&body, &["x"]);
    }

    /// A lifted closure function `f` reading captured value `cap_idx` of a capture object laid out as
    /// `cap_tys`, out of `reads` — its capture parameter, in a well-formed function. It disposes of
    /// the capture it owns, so its reference counting balances.
    fn projecting_func(reads: &RcVar, cap_idx: usize, cap_tys: Vec<Arc<TypeNode>>) -> RcFunc {
        let capture = var_of("cap", make_dynamic_object_ty());
        let proj = Box::new(InlineLLVMCaptureProjectBody {
            cap_name: reads.name.clone(),
            cap_idx,
            cap_tys,
        });
        let body = node(RcExpr::Let(
            var("c"),
            RcRhs::Llvm(proj, vec![reads.clone()]),
            node(RcExpr::Release(
                capture.clone(),
                vec![],
                RcState::Unknown,
                node(RcExpr::Ret(var("c"))),
            )),
        ));
        func(
            "f",
            type_fun(make_i64_ty(), make_i64_ty()),
            vec![var("p")],
            Some(capture),
            body,
        )
    }

    /// A function `g` whose body builds a closure value targeting `f`, storing `stored`.
    fn closure_building_func(stored: RcVar) -> RcFunc {
        let body = node(RcExpr::Let(
            var("cl"),
            RcRhs::Closure(
                FuncRef {
                    name: FullName::local("f"),
                },
                vec![stored.clone()],
            ),
            node(RcExpr::Ret(var("cl"))),
        ));
        func(
            "g",
            type_funptr(vec![stored.ty.clone()], make_i64_ty()),
            vec![stored],
            None,
            body,
        )
    }

    #[test]
    fn accepts_a_closure_whose_captures_match_the_projected_layout() {
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![
            closure_building_func(var_of("v", make_ptr_ty())),
            projecting_func(&capture, 0, vec![make_ptr_ty()]),
        ]);
    }

    #[test]
    #[should_panic(expected = "closure stores captures")]
    fn rejects_a_closure_whose_captures_differ_from_the_projected_layout() {
        // `f` projects one captured `I64`, while the closure value stores one `Ptr`.
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![
            closure_building_func(var_of("v", make_ptr_ty())),
            projecting_func(&capture, 0, vec![make_i64_ty()]),
        ]);
    }

    #[test]
    #[should_panic(expected = "is not the capture parameter")]
    fn rejects_a_capture_projection_of_another_variable() {
        // The projection reads the parameter `p` rather than the capture parameter.
        validate_prog(vec![projecting_func(&var("p"), 0, vec![make_i64_ty()])]);
    }

    #[test]
    #[should_panic(expected = "of a 1-slot capture")]
    fn rejects_a_capture_projection_past_the_layout() {
        // The projection reads slot 1 of a capture object laid out with one slot.
        let capture = var_of("cap", make_dynamic_object_ty());
        validate_prog(vec![projecting_func(&capture, 1, vec![make_i64_ty()])]);
    }

    /// A boxed parameter, which holds one reference-counting unit.
    fn boxed_var(name: &str) -> RcVar {
        var_of(name, make_dynamic_object_ty())
    }

    /// A function taking a boxed parameter `b` and an unboxed `p`, with `borrowed` as the units it
    /// borrows rather than owns.
    fn boxed_param_func(body: RcExprNode, borrowed: Set<VarPath>) -> RcFunc {
        let mut f = func(
            "f",
            type_funptr(vec![make_dynamic_object_ty(), make_i64_ty()], make_i64_ty()),
            vec![boxed_var("b"), var("p")],
            None,
            body,
        );
        f.borrowed_units = borrowed;
        f
    }

    /// `release b; k` — dispose of the boxed parameter.
    fn release_b(k: RcExprNode) -> RcExprNode {
        node(RcExpr::Release(boxed_var("b"), vec![], RcState::Unknown, k))
    }

    #[test]
    fn accepts_a_disposed_parameter() {
        // release b; ret p
        let body = release_b(node(RcExpr::Ret(var("p"))));
        validate_prog(vec![boxed_param_func(body, Set::default())]);
    }

    #[test]
    #[should_panic(expected = "more often than it holds it")]
    fn rejects_a_reference_disposed_of_twice() {
        // release b; release b; ret p   (the parameter arrives holding one reference)
        let body = release_b(release_b(node(RcExpr::Ret(var("p")))));
        validate_prog(vec![boxed_param_func(body, Set::default())]);
    }

    #[test]
    #[should_panic(expected = "returns holding references it never disposes of")]
    fn rejects_a_return_leaving_a_reference_undisposed() {
        // ret p   (the owned parameter `b` is never disposed of)
        validate_prog(vec![boxed_param_func(
            node(RcExpr::Ret(var("p"))),
            Set::default(),
        )]);
    }

    #[test]
    fn accepts_a_borrowed_parameter_left_undisposed() {
        // let x = b; ret p   (a borrowed parameter's reference belongs to the caller)
        let body = node(RcExpr::Let(
            boxed_var("x"),
            RcRhs::Var(boxed_var("b")),
            node(RcExpr::Ret(var("p"))),
        ));
        let borrowed = [(FullName::local("b"), vec![])].into_iter().collect();
        validate_prog(vec![boxed_param_func(body, borrowed)]);
    }

    #[test]
    #[should_panic(expected = "was already consumed")]
    fn rejects_a_read_after_the_last_reference_is_disposed() {
        // release b; let x = b; ret p
        let body = release_b(node(RcExpr::Let(
            boxed_var("x"),
            RcRhs::Var(boxed_var("b")),
            node(RcExpr::Ret(var("p"))),
        )));
        validate_prog(vec![boxed_param_func(body, Set::default())]);
    }

    #[test]
    #[should_panic(expected = "match arms leave different reference counts")]
    fn rejects_match_arms_leaving_different_reference_counts() {
        // let m = match p { 1 -> (release b; ret u1); 0 -> ret u2 }; ret m
        let arms = vec![
            MatchArm {
                tag: Some(1),
                payload: var("u1"),
                body: release_b(node(RcExpr::Ret(var("u1")))),
            },
            MatchArm {
                tag: Some(0),
                payload: var("u2"),
                body: node(RcExpr::Ret(var("u2"))),
            },
        ];
        let body = node(RcExpr::Let(
            var("m"),
            RcRhs::Match(var("p"), arms),
            node(RcExpr::Ret(var("m"))),
        ));
        validate_prog(vec![boxed_param_func(body, Set::default())]);
    }
}
