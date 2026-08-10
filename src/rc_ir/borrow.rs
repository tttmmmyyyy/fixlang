//! Borrow-ification over the RC IR: rewriting `Own` parameters that a function only
//! reads to `Borrow`, so the caller keeps ownership across the call and no retain is needed before a
//! non-last use — which is what keeps a value `Unique` for the uniqueness analysis.
//!
//! Lowering makes every parameter `Own` (the callee releases it). Borrow-ification has three parts:
//!
//! 1. *Inference*: decide, for each source function, which parameter leaves can be borrowed. A leaf
//!    is borrowable unless it reaches a *consume site* — an owning argument position, a capture, or
//!    a return — traced back through aliases (move-binds and unboxed-aggregate projections) to the
//!    parameter it originates from. Ownership is a fixed point: whether an argument position consumes
//!    depends on the callee's ownership, which is itself being decided.
//!
//! 2. *Version routing*: a function with a borrowable parameter is materialized in two versions, the
//!    all-`Own` baseline (`f_own`, the original) and a borrowing clone (`f_borrow`). Each direct call
//!    is routed to one version. A call is routed to the borrow version only when it is *safe* — the
//!    call is not in tail position, or it passes no owned argument — so a tail call is never turned
//!    into a non-tail one by an after-call release. Indirect calls keep the all-`Own` original.
//!
//! 3. *Reference-count rewrite*: the borrow clone drops the reference counting on its borrowed
//!    parameter leaves, and each call site takes over the counting the callee no longer does — a
//!    release after the call for an owned value passed to a borrowed position, and a retain before it
//!    for a borrowed value passed to an owning position.
//!
//! Borrow-ification and cancellation both work one reference-counting unit at a time, so
//! `split_rc_units` first normalizes the lowered reference counting to that granularity: it
//! decomposes a whole-value or subtree `Retain`/`Release` into one node per unit — a boxed leaf, a
//! closure capture, or an unboxed-union root (a union is one unit, since a physical refcount
//! operation on it must dispatch on the tag rather than name a variant).
//!
//! Borrow-ification leaves the caller with a retain before a borrow call and a release after it,
//! bracketing the call with no consume between. `cancel` removes those net-zero brackets: a retain is
//! cancellable when, on every forward path, a release un-bumps it before the value is consumed. That
//! keeps the value `Unique` for the uniqueness analysis, the reason borrow-ification exists.
//!
//! Which construct consumes which reference, and which object a reference belongs to, is the shared
//! model in `ownership`: inference, rewrite, and cancellation all read it, so they agree.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::misc::{grow_stack, Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    FieldPath, FuncRef, MatchArm, Ownership, OwnershipShape, RcExpr, RcExprNode, RcFunc,
    RcGlobalInit, RcProgram, RcRhs, RcState, RcVar, VarPath,
};
use crate::rc_ir::ownership::{
    acted_unit_keys, all_owned_units, boxed_leaves, collect_consumes, destructure_consumes, origin,
    rc_units, rhs_consumes, truncate_to_unit, unit_key, units_under, VarTable,
};
use crate::rc_ir::rename::fresh_rename_function;
use std::sync::Arc;

/// The result of borrow inference: which parameter leaves are `Own` (all others are `Borrow`), keyed
/// by the parameter variable's name and the leaf path.
struct Ownerships {
    own: Set<VarPath>,
}

/// Infer parameter ownership for every function of `prog` by a fixed point: start every parameter
/// leaf `Borrow`, then repeatedly demote to `Own` any leaf that a consume site traces back to, until
/// nothing changes. Demotion is monotone (`Borrow` to `Own` only), so it terminates.
fn infer_ownership(prog: &RcProgram, type_env: &TypeEnv) -> Ownerships {
    let vars: Map<FuncRef, VarTable> = prog
        .funcs
        .values()
        .map(|f| (f.name.clone(), VarTable::of(f)))
        .collect();

    let mut own: Set<VarPath> = Set::default();
    loop {
        let mut changed = false;
        for func in prog.funcs.values() {
            let vars = &vars[&func.name];
            let mut consumed = vec![];
            collect_consumes(&func.body, vars, prog, &own, type_env, &mut consumed);
            for (var, path) in consumed {
                // Attribute the consume to the parameters it may originate from, and own them. A
                // consumed leaf that is one of several objects is consumed whichever it is, so every
                // parameter it may be has to be owned.
                for (root_var, root_path) in origin(vars, type_env, &var, &path).candidates() {
                    if vars.param_tys.contains_key(root_var)
                        && own.insert((root_var.clone(), root_path.clone()))
                    {
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    Ownerships { own }
}

// --- borrow-ification ---

/// Borrow-ify a program: materialize a borrowing version of every function with a borrowable
/// parameter, route each direct call to a version, rewrite the reference counting accordingly, and
/// annotate every output version with the parameter/capture units it borrows (`RcFunc::borrowed_units`,
/// whose owned complement `cancel` reads to find each call's consume sites and the RC IR dump reads
/// for its shapes).
pub fn borrow_ify(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let ownerships = infer_ownership(prog, type_env);

    // The funptr functions that get a borrow version, and the name of that version. Only funptr
    // functions are considered: a closure is reached only by an indirect call, which keeps the
    // all-`Own` original, so a borrow clone of it would never be routed to.
    let mut borrow_versions: Map<FuncRef, FuncRef> = Map::default();
    for func in prog.funcs.values() {
        if func.capture.is_none() && func_has_borrowable_param(func, &ownerships, type_env) {
            borrow_versions.insert(func.name.clone(), borrow_funcref(&func.name));
        }
    }

    // The owned parameter units of every output version, keyed by the version's own parameter names:
    // an original (`f_own`) owns all of them, a borrow clone (`f_borrow`) owns the inferred subset.
    let mut owned_units: Set<VarPath> = Set::default();
    let mut rename_counter: u64 = 0;
    let mut clones: Vec<(FuncRef, RcFunc, Map<FullName, FullName>)> = vec![];
    for func in prog.funcs.values() {
        // `f_own`: every parameter and capture unit is owned.
        for p in func.params.iter().chain(func.capture.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                owned_units.insert((p.name.clone(), unit));
            }
        }
        // `f_borrow`: a fresh clone whose owned units are the inferred ones, clamped to units.
        if let Some(bref) = borrow_versions.get(&func.name) {
            let (clone, rename) = clone_func(func, bref.clone(), &mut rename_counter);
            for p in &func.params {
                for leaf in boxed_leaves(&p.ty, type_env) {
                    if ownerships.own.contains(&(p.name.clone(), leaf.clone())) {
                        let unit = truncate_to_unit(&p.ty, &leaf, type_env);
                        owned_units.insert((rename[&p.name].clone(), unit));
                    }
                }
            }
            clones.push((bref.clone(), clone, rename));
        }
    }

    // The parameter names and types of every output version, so a call site can read the ownership
    // of the routed callee's positions.
    let mut callee_params: Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>> = Map::default();
    for func in prog.funcs.values() {
        callee_params.insert(func.name.clone(), param_names_and_types(func));
    }
    for (bref, clone, _) in &clones {
        callee_params.insert(bref.clone(), param_names_and_types(clone));
    }

    // Rewrite every version's body: route its calls and adjust the reference counting.
    let mut funcs: Map<FuncRef, RcFunc> = Map::default();
    for func in prog.funcs.values() {
        let mut f_own = func.clone();
        let ctx = RewriteCtx::new(
            &f_own,
            false,
            &owned_units,
            &borrow_versions,
            &callee_params,
            type_env,
        );
        f_own.body = ctx.rewrite(&f_own.body);
        funcs.insert(f_own.name.clone(), f_own);
    }
    for (bref, mut clone, _) in clones {
        let ctx = RewriteCtx::new(
            &clone,
            true,
            &owned_units,
            &borrow_versions,
            &callee_params,
            type_env,
        );
        clone.body = ctx.rewrite(&clone.body);
        funcs.insert(bref, clone);
    }

    // Globals are param-less function bodies: route and rewrite them the same way (as `f_own`).
    let globals = prog
        .globals
        .iter()
        .map(|g| {
            let vars = VarTable::body_only(&g.init);
            let ctx = RewriteCtx {
                type_env,
                is_borrow_version: false,
                owned_units: &owned_units,
                borrow_versions: &borrow_versions,
                callee_params: &callee_params,
                tail: tail_result_vars(&g.init),
                vars,
            };
            RcGlobalInit {
                symbol: g.symbol.clone(),
                ty: g.ty.clone(),
                init: ctx.rewrite(&g.init),
            }
        })
        .collect();

    // Annotate every version with the parameter/capture units it borrows (those not in `owned_units`).
    for func in funcs.values_mut() {
        let mut borrowed = Set::default();
        for p in func.params.iter().chain(func.capture.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                let unit_path = (p.name.clone(), unit);
                if !owned_units.contains(&unit_path) {
                    borrowed.insert(unit_path);
                }
            }
        }
        func.borrowed_units = borrowed;
    }

    RcProgram {
        funcs,
        globals,
        entry: prog.entry.clone(),
    }
}

/// The ownership shape of every parameter and capture variable of the program, keyed by the
/// variable's name: which of the variable's reference-counting units its function owns and which it
/// borrows.
pub fn param_ownership_shapes(
    prog: &RcProgram,
    type_env: &TypeEnv,
) -> Map<FullName, OwnershipShape> {
    let owned_units = all_owned_units(prog, type_env);
    let mut shapes = Map::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.capture.iter()) {
            shapes.insert(
                p.name.clone(),
                param_ownership_shape(&p.name, &p.ty, &owned_units, type_env),
            );
        }
    }
    shapes
}

/// The name of a function's borrow version: its name with a `#borrow` suffix. No lowered name ends in
/// `#borrow`, so this stays globally unique.
fn borrow_funcref(name: &FuncRef) -> FuncRef {
    let mut n = name.name.clone();
    n.name.push_str("#borrow");
    FuncRef { name: n }
}

/// Whether any of a function's parameter leaves is borrowable (not in the inferred owned set).
fn func_has_borrowable_param(func: &RcFunc, ownerships: &Ownerships, type_env: &TypeEnv) -> bool {
    func.params.iter().any(|p| {
        boxed_leaves(&p.ty, type_env)
            .iter()
            .any(|leaf| !ownerships.own.contains(&(p.name.clone(), leaf.clone())))
    })
}

/// The name/type of each parameter and capture, in order.
fn param_names_and_types(func: &RcFunc) -> Vec<(FullName, Arc<TypeNode>)> {
    func.params
        .iter()
        .chain(func.capture.iter())
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect()
}

/// The ownership shape of one parameter, read from the owned-unit set: `Own` at a reference-counting
/// unit that is owned, else `Borrow`.
fn param_ownership_shape(
    var: &FullName,
    ty: &Arc<TypeNode>,
    owned_units: &Set<VarPath>,
    type_env: &TypeEnv,
) -> OwnershipShape {
    /// The shape of the subtree of type `ty` that `path` names within `var`, `path` being the field
    /// path from the parameter root down to that subtree.
    fn go(
        var: &FullName,
        ty: &Arc<TypeNode>,
        owned_units: &Set<VarPath>,
        type_env: &TypeEnv,
        path: &mut FieldPath,
    ) -> OwnershipShape {
        let ownership_at = |path: &FieldPath| {
            if owned_units.contains(&(var.clone(), path.clone())) {
                Ownership::Own
            } else {
                Ownership::Borrow
            }
        };
        if ty.is_fully_unboxed(type_env) {
            return OwnershipShape::NoUnit;
        }
        if ty.is_closure() {
            path.push(CLOSURE_CAPTURE_IDX as usize);
            let capture_ownership = ownership_at(path);
            path.pop();
            return OwnershipShape::Fields(vec![
                OwnershipShape::NoUnit,
                OwnershipShape::Unit(capture_ownership),
            ]);
        }
        if ty.is_rc_unit_root(type_env) {
            return OwnershipShape::Unit(ownership_at(path));
        }
        // A field the value holds nothing at keeps its place in the shape, so that a shape index is a
        // field index.
        let mut children = vec![OwnershipShape::NoUnit; ty.field_types(type_env).len()];
        for (i, fty) in ty.unpunched_field_types(type_env) {
            path.push(i);
            children[i] = go(var, &fty, owned_units, type_env, path);
            path.pop();
        }
        OwnershipShape::Fields(children)
    }
    go(var, ty, owned_units, type_env, &mut vec![])
}

// --- tail-call recognition ---

/// The variables bound to an `App` or `Match` in tail position: a call in tail position must not be
/// turned into a non-tail one by an after-call release, so routing consults this set.
fn tail_result_vars(body: &RcExprNode) -> Set<FullName> {
    let mut out = Set::default();
    mark_tail(body, true, &mut out);
    out
}

/// Collect the tail-position `App` and `Match` bindings of a subtree into `out`. `in_tail` says
/// whether the subtree itself sits in tail position: a binding is in tail position when its
/// continuation does nothing but return it (`trivially_returns`) and the subtree holding it is too,
/// and a match arm inherits the tail position of the match.
fn mark_tail(node: &RcExprNode, in_tail: bool, out: &mut Set<FullName>) {
    match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => {
            let is_tail = in_tail && trivially_returns(k, &x.name);
            match rhs {
                RcRhs::App(..) if is_tail => {
                    out.insert(x.name.clone());
                }
                RcRhs::Match(_, arms) => {
                    if is_tail {
                        out.insert(x.name.clone());
                    }
                    for arm in arms {
                        mark_tail(&arm.body, is_tail, out);
                    }
                }
                _ => {}
            }
            mark_tail(k, in_tail, out);
        }
        RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k)
        | RcExpr::Destructure(_, _, _, k)
        | RcExpr::Eval(_, k) => mark_tail(k, in_tail, out),
        RcExpr::Ret(_) => {}
    }
}

/// Whether a continuation does nothing but rename `x` and return it — the tail chain a real operation
/// (a retain, release, or any non-rename binding) would break.
fn trivially_returns(k: &RcExprNode, x: &FullName) -> bool {
    match k.expr.as_ref() {
        RcExpr::Ret(v) => v.name == *x,
        RcExpr::Let(s, RcRhs::Var(y), k2) if y.name == *x => trivially_returns(k2, &s.name),
        _ => false,
    }
}

// --- cloning a function with fresh names ---

/// Clone a function as its borrow version: mint a fresh name for every bound variable (parameters,
/// capture, `let` bindings, destructure fields, match-arm payloads) and rewrite all occurrences,
/// keeping global name uniqueness. The recursive references to top-level functions are not bound
/// here, so they are left for routing to retarget. Returns the clone and the binder renaming.
fn clone_func(
    func: &RcFunc,
    new_ref: FuncRef,
    rename_counter: &mut u64,
) -> (RcFunc, Map<FullName, FullName>) {
    let (params, capture, body, rename) =
        fresh_rename_function(&func.params, &func.capture, &func.body, "b", rename_counter);
    (
        RcFunc {
            name: new_ref,
            fn_ty: func.fn_ty.clone(),
            params,
            capture,
            ret_ty: func.ret_ty.clone(),
            body,
            source: func.source.clone(),
            borrowed_units: Set::default(),
        },
        rename,
    )
}

// --- routing and reference-count rewrite ---

/// The per-version state the body rewrite reads: this version's aliasing vars and tail calls,
/// whether it is the borrow clone, and the whole-program ownership and version tables.
struct RewriteCtx<'a> {
    /// The type definitions, for resolving a value's type to its reference-counting units.
    type_env: &'a TypeEnv,
    /// Whether this version is the borrow clone, whose reference counting on its borrowed parameter
    /// leaves is dropped. The all-owning original keeps every node it was given.
    is_borrow_version: bool,
    /// The inferred `Own` parameter leaves of the whole program, one `(parameter-name, unit-path)`
    /// each. A parameter leaf absent from it is borrowed.
    owned_units: &'a Set<VarPath>,
    /// The borrow clone of each function that got one, which a call is routed to where routing is
    /// safe and saves a reference count.
    borrow_versions: &'a Map<FuncRef, FuncRef>,
    /// The parameter names and types of every version, original and borrow clone alike, so a call
    /// can look its callee's parameters up in `owned_units`.
    callee_params: &'a Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>>,
    /// The bindings of this version's tail-position calls and matches, whose calls stay on the
    /// owning version so that no after-call release lands on a tail call.
    tail: Set<FullName>,
    /// This version's variables: what binds each one and its type, which decide the object a leaf
    /// belongs to and whether this version owns it.
    vars: VarTable,
}

impl<'a> RewriteCtx<'a> {
    /// The rewrite state of one output version of `func`. `is_borrow_version` marks the borrow
    /// clone, the version whose reference counting on its borrowed parameter leaves is dropped.
    fn new(
        func: &RcFunc,
        is_borrow_version: bool,
        owned_units: &'a Set<VarPath>,
        borrow_versions: &'a Map<FuncRef, FuncRef>,
        callee_params: &'a Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>>,
        type_env: &'a TypeEnv,
    ) -> RewriteCtx<'a> {
        RewriteCtx {
            type_env,
            is_borrow_version,
            owned_units,
            borrow_versions,
            callee_params,
            tail: tail_result_vars(&func.body),
            vars: VarTable::of(func),
        }
    }

    /// Rewrite a body for this version: route each direct call to a callee version, bracket a call
    /// with the reference counting the routed callee no longer does, and drop the counting this
    /// version's borrowed parameters no longer need.
    fn rewrite(&self, node: &RcExprNode) -> RcExprNode {
        grow_stack(|| self.rewrite_inner(node))
    }

    /// One node of the rewrite, rebuilt over its rewritten continuation.
    fn rewrite_inner(&self, node: &RcExprNode) -> RcExprNode {
        match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let callee = self.route(x, callee, args, k);
                let (before, after) = self.call_rc(&callee, args);
                let k = prepend_rc(after, true, self.rewrite(k));
                let app = expr_node(
                    RcExpr::Let(x.clone(), RcRhs::App(callee, args.clone()), k),
                    &node.source,
                );
                prepend_rc(before, false, app)
            }
            RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
                let arms = arms
                    .iter()
                    .map(|arm| MatchArm {
                        payload_state: arm.payload_state,
                        tag: arm.tag,
                        payload: arm.payload.clone(),
                        body: self.rewrite(&arm.body),
                    })
                    .collect();
                expr_node(
                    RcExpr::Let(
                        x.clone(),
                        RcRhs::Match(scrut.clone(), arms),
                        self.rewrite(k),
                    ),
                    &node.source,
                )
            }
            RcExpr::Let(x, rhs, k) => expr_node(
                RcExpr::Let(x.clone(), rhs.clone(), self.rewrite(k)),
                &node.source,
            ),
            RcExpr::Retain(v, path, state, k) => {
                self.rewrite_rc(v, path, *state, false, k, &node.source)
            }
            RcExpr::Release(v, path, state, k) => {
                self.rewrite_rc(v, path, *state, true, k, &node.source)
            }
            RcExpr::Destructure(container, fields, state, k) => expr_node(
                RcExpr::Destructure(container.clone(), fields.clone(), *state, self.rewrite(k)),
                &node.source,
            ),
            RcExpr::Eval(v, k) => expr_node(RcExpr::Eval(v.clone(), self.rewrite(k)), &node.source),
            RcExpr::Ret(v) => expr_node(RcExpr::Ret(v.clone()), &node.source),
        }
    }

    /// Route a direct call: retarget the callee to its borrow version when that has a version and
    /// routing to it is both safe and beneficial; otherwise keep the original (the all-`Own` version,
    /// or an indirect callee this leaves untouched). `k` is the call's continuation, which the
    /// benefit test reads to tell an argument's last use from a use that outlives the call.
    fn route(&self, x: &RcVar, callee: &RcVar, args: &[RcVar], k: &RcExprNode) -> RcVar {
        let orig = FuncRef {
            name: callee.name.clone(),
        };
        if let Some(bref) = self.borrow_versions.get(&orig) {
            if self.routing_is_safe(x, args) && self.routing_saves_retain(bref, args, k) {
                let mut c = callee.clone();
                c.name = bref.name.clone();
                return c;
            }
        }
        callee.clone()
    }

    /// A call is safe to route to the borrow version when it is not in tail position, or it passes no
    /// owned argument — so the after-call release the borrow version needs never lands on a tail call.
    fn routing_is_safe(&self, x: &RcVar, args: &[RcVar]) -> bool {
        !self.tail.contains(&x.name) || !args.iter().any(|a| self.any_owned_unit(a))
    }

    /// Whether routing this call to the borrow version removes a reference count it would otherwise
    /// need, for at least one argument unit. Routing helps a unit that the borrow version borrows and
    /// that would otherwise be retained: a borrowed value (which an owning callee makes the caller
    /// retain before the call) or an owned value used again after the call (whose retain-before the
    /// borrow cancels). An owned value at its last use is moved either way, so borrowing it removes no
    /// retain and only delays its release; it is not a benefit.
    fn routing_saves_retain(&self, bref: &FuncRef, args: &[RcVar], k: &RcExprNode) -> bool {
        // `bref` is a borrow version, and `borrow_ify` registers every version's parameters, so it is a
        // key here.
        let bparams = &self.callee_params[bref];
        args.iter().enumerate().any(|(arg_idx, arg)| {
            let last_use = !used_later(&arg.name, k);
            rc_units(&arg.ty, self.type_env).iter().any(|unit| {
                // `arg_idx` is in range since `args.len() <= params.len()`.
                let callee_borrows = !self
                    .owned_units
                    .contains(&(bparams[arg_idx].0.clone(), unit.clone()));
                callee_borrows && !(self.owns_unit(arg, unit) && last_use)
            })
        })
    }

    /// Whether this version owns the value at any of `arg`'s reference-counting units.
    fn any_owned_unit(&self, arg: &RcVar) -> bool {
        rc_units(&arg.ty, self.type_env)
            .iter()
            .any(|unit| self.owns_unit(arg, unit))
    }

    /// Whether this version owns the value at `arg@unit`: a leaf that comes from an owned parameter,
    /// or from a producer (a fresh value, a call result, a boxed-container read), is owned; a leaf
    /// that comes from a borrowed parameter is not. A leaf that may be one of several objects is
    /// owned only when it is owned whichever it is.
    fn owns_unit(&self, arg: &RcVar, unit: &FieldPath) -> bool {
        origin(&self.vars, self.type_env, &arg.name, unit)
            .candidates()
            .iter()
            .all(|(root, path)| self.owns_object(root, path))
    }

    /// Whether this version owns the object a leaf comes from.
    fn owns_object(&self, root: &FullName, path: &FieldPath) -> bool {
        match self.vars.param_tys.get(root) {
            // The path may name a subtree that spans several reference-counting units rather than
            // one — a union built from an unboxed tuple roots to the tuple at the empty path, whose
            // units are its fields. The value is owned only when every unit it covers is owned. Each
            // covered path is clamped to its unit key, so a path that descends into a union variant
            // keys to the union root the owned set records.
            Some(root_ty) => units_under(root_ty, path, self.type_env)
                .iter()
                .all(|unit| {
                    self.owned_units
                        .contains(&(root.clone(), truncate_to_unit(root_ty, unit, self.type_env)))
                }),
            // A root this version takes no parameter for is a producer, or a global — whose reachable
            // graph is refcount-exempt. Either way the caller lent no reference of it, so the value
            // here is this version's.
            None => true,
        }
    }

    /// The reference-count operations a call site takes over: for each argument unit, a release after
    /// the call when an owned value is passed to a borrowed position, and a retain before the call
    /// when a borrowed value is passed to an owning position.
    fn call_rc(
        &self,
        callee: &RcVar,
        args: &[RcVar],
    ) -> (Vec<(RcVar, FieldPath)>, Vec<(RcVar, FieldPath)>) {
        let cparams = self.callee_params.get(&FuncRef {
            name: callee.name.clone(),
        });
        let mut before = vec![];
        let mut after = vec![];
        for (arg_idx, arg) in args.iter().enumerate() {
            for unit in rc_units(&arg.ty, self.type_env) {
                // An unresolved (indirect) callee owns every position (the all-`Own` ABI); a resolved
                // one is indexed by `arg_idx`, which is in range since `args.len() <= params.len()`.
                let callee_owns = match cparams {
                    None => true,
                    Some(ps) => self
                        .owned_units
                        .contains(&(ps[arg_idx].0.clone(), unit.clone())),
                };
                let arg_owned = self.owns_unit(arg, &unit);
                if !callee_owns && arg_owned {
                    after.push((arg.clone(), unit));
                } else if callee_owns && !arg_owned {
                    before.push((arg.clone(), unit));
                }
            }
        }
        (before, after)
    }

    /// Rewrite a `Retain`/`Release`: in the borrow clone, drop the units that root at a borrowed
    /// parameter (the callee no longer counts them); otherwise keep the node unchanged.
    fn rewrite_rc(
        &self,
        v: &RcVar,
        path: &FieldPath,
        state: RcState,
        is_release: bool,
        k: &RcExprNode,
        source: &Option<Span>,
    ) -> RcExprNode {
        let k = self.rewrite(k);
        if !self.is_borrow_version {
            return rc_node(is_release, v.clone(), path.clone(), state, k, source);
        }
        let kept: Vec<FieldPath> = units_under(&v.ty, path, self.type_env)
            .into_iter()
            .filter(|unit| self.owns_unit(v, unit))
            .collect();
        kept.into_iter().rev().fold(k, |cont, unit| {
            rc_node(is_release, v.clone(), unit, state, cont, source)
        })
    }
}

/// An expression node with the given source span.
fn expr_node(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Arc::new(expr),
        source: source.clone(),
    }
}

/// A `Release` (when `is_release`) or `Retain` of `var` at `path` wrapping continuation `k`.
fn rc_node(
    is_release: bool,
    var: RcVar,
    path: FieldPath,
    state: RcState,
    k: RcExprNode,
    source: &Option<Span>,
) -> RcExprNode {
    let expr = if is_release {
        RcExpr::Release(var, path, state, k)
    } else {
        RcExpr::Retain(var, path, state, k)
    };
    expr_node(expr, source)
}

/// Wrap a continuation in a `Retain` (or `Release`) of each given unit.
fn prepend_rc(items: Vec<(RcVar, FieldPath)>, is_release: bool, k: RcExprNode) -> RcExprNode {
    items.into_iter().rev().fold(k, |cont, (var, path)| {
        rc_node(is_release, var, path, RcState::Unknown, cont, &None)
    })
}

/// Whether the variable named `name` is used again in an expression subtree — any occurrence as a
/// value: a move, a call callee or argument, an inline-LLVM operand, a closure capture, a match
/// scrutinee, a destructured container, or the returned variable. A `Retain`/`Release` names its
/// variable only for reference counting, not as a use, so those are transparent — which lets a call
/// be recognized as an argument's last use even when the lowering brackets it with reference counts.
fn used_later(name: &FullName, node: &RcExprNode) -> bool {
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Ret(v) => v.name == *name,
        RcExpr::Let(_, rhs, k) => rhs_uses(name, rhs) || used_later(name, k),
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => used_later(name, k),
        RcExpr::Destructure(container, _, _state, k) => {
            container.name == *name || used_later(name, k)
        }
        // `Eval` observes its variable, so — unlike the transparent reference-count nodes — it counts
        // as a use.
        RcExpr::Eval(v, k) => v.name == *name || used_later(name, k),
    })
}

/// Whether the variable named `name` occurs as a value in a right-hand side.
fn rhs_uses(name: &FullName, rhs: &RcRhs) -> bool {
    match rhs {
        RcRhs::Var(v) => v.name == *name,
        RcRhs::App(callee, args) => callee.name == *name || args.iter().any(|a| a.name == *name),
        RcRhs::Closure(_, caps) => caps.iter().any(|c| c.name == *name),
        RcRhs::Llvm(llvm_gen, args) => {
            args.iter().any(|a| a.name == *name) || llvm_gen.free_vars().iter().any(|v| v == name)
        }
        RcRhs::Match(scrut, arms) => {
            scrut.name == *name || arms.iter().any(|arm| used_later(name, &arm.body))
        }
    }
}

// --- unit normalization ---

/// Decompose every `Retain`/`Release` into one node per reference-counting unit its path covers, so
/// borrow-ification and cancellation both see reference counting at unit granularity. A path that
/// already names a single unit is unchanged; a whole-value retain on a fully-unboxed value (a no-op)
/// disappears.
pub fn split_rc_units(prog: &mut RcProgram, type_env: &TypeEnv) {
    for func in prog.funcs.values_mut() {
        func.body = split_body(&func.body, type_env);
    }
    for g in &mut prog.globals {
        g.init = split_body(&g.init, type_env);
    }
}

/// Rebuild a body with every `Retain`/`Release` in it replaced by one node per reference-counting
/// unit its path covers.
fn split_body(node: &RcExprNode, type_env: &TypeEnv) -> RcExprNode {
    grow_stack(|| split_body_inner(node, type_env))
}

/// One node of `split_body`'s rebuild, over its rebuilt continuation.
fn split_body_inner(node: &RcExprNode, type_env: &TypeEnv) -> RcExprNode {
    match node.expr.as_ref() {
        RcExpr::Retain(v, path, state, k) => {
            let k = split_body(k, type_env);
            split_rc(v, path, *state, false, k, &node.source, type_env)
        }
        RcExpr::Release(v, path, state, k) => {
            let k = split_body(k, type_env);
            split_rc(v, path, *state, true, k, &node.source, type_env)
        }
        RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    payload_state: arm.payload_state,
                    tag: arm.tag,
                    payload: arm.payload.clone(),
                    body: split_body(&arm.body, type_env),
                })
                .collect();
            expr_node(
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrut.clone(), arms),
                    split_body(k, type_env),
                ),
                &node.source,
            )
        }
        RcExpr::Let(x, rhs, k) => expr_node(
            RcExpr::Let(x.clone(), rhs.clone(), split_body(k, type_env)),
            &node.source,
        ),
        RcExpr::Destructure(container, fields, state, k) => expr_node(
            RcExpr::Destructure(
                container.clone(),
                fields.clone(),
                *state,
                split_body(k, type_env),
            ),
            &node.source,
        ),
        RcExpr::Eval(v, k) => expr_node(
            RcExpr::Eval(v.clone(), split_body(k, type_env)),
            &node.source,
        ),
        RcExpr::Ret(v) => expr_node(RcExpr::Ret(v.clone()), &node.source),
    }
}

/// Rebuild a `Retain`/`Release` as one node per unit under its path, preserving the state and span.
fn split_rc(
    v: &RcVar,
    path: &FieldPath,
    state: RcState,
    is_release: bool,
    k: RcExprNode,
    source: &Option<Span>,
    type_env: &TypeEnv,
) -> RcExprNode {
    units_under(&v.ty, path, type_env)
        .into_iter()
        .rev()
        .fold(k, |cont, unit| {
            rc_node(is_release, v.clone(), unit, state, cont, source)
        })
}

// --- retain/release cancellation ---

/// The pending retains at a program point: for each object (a reference-counting unit, keyed by
/// `unit_key`), the stack of retains that have bumped it and not yet been un-bumped. A release
/// un-bumps the most recent — the innermost bracket, which keeps the un-bump non-zeroing.
type PendingRetains = Map<VarPath, Vec<NodeId>>;

/// A node's identity within one tree: the address of its expression, stable while the tree is
/// borrowed. The analysis records which nodes to drop by identity, and the deletion pass, walking the
/// same borrowed tree, recognizes them by the same identity.
type NodeId = usize;

/// The `NodeId` of a node: the address of its boxed `RcExpr`.
fn node_id(node: &RcExprNode) -> NodeId {
    node.expr.as_ref() as *const RcExpr as NodeId
}

/// Remove the net-zero retain/release brackets borrow-ification leaves across borrow calls: a retain
/// is cancellable when, on every forward path, a release un-bumps it before the value is consumed.
/// Cancelling it (and the releases it pairs with) keeps the value `Unique` for the uniqueness
/// analysis. Each call's consume sites are decided by the parameter/capture units the functions own —
/// the complement of their `RcFunc::borrowed_units`, set by borrow-ification.
pub fn cancel(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let owned_units = all_owned_units(prog, type_env);
    let cancel_body = |vars: &VarTable, body: &RcExprNode| {
        let mut analysis = CancelAnalysis {
            vars,
            prog,
            owned_units: &owned_units,
            type_env,
            needed_retains: Set::default(),
            unbump_releases: Map::default(),
            all_retains: vec![],
        };
        analysis.walk(body, PendingRetains::default(), true);
        drop_nodes(body, &analysis.cancelled())
    };

    let funcs = prog
        .funcs
        .values()
        .map(|f| {
            let vars = VarTable::of(f);
            let mut clone = f.clone();
            clone.body = cancel_body(&vars, &f.body);
            (f.name.clone(), clone)
        })
        .collect();
    let globals = prog
        .globals
        .iter()
        .map(|g| {
            let vars = VarTable::body_only(&g.init);
            RcGlobalInit {
                symbol: g.symbol.clone(),
                ty: g.ty.clone(),
                init: cancel_body(&vars, &g.init),
            }
        })
        .collect();
    RcProgram {
        funcs,
        globals,
        entry: prog.entry.clone(),
    }
}

/// The forward must-analysis for one function: it decides which retain and release nodes to delete.
struct CancelAnalysis<'a> {
    vars: &'a VarTable,
    prog: &'a RcProgram,
    owned_units: &'a Set<VarPath>,
    type_env: &'a TypeEnv,
    /// Retains that are load-bearing on some path, so they cannot be cancelled.
    needed_retains: Set<NodeId>,
    /// The releases each retain is un-bumped by; they are deleted together with the retain.
    unbump_releases: Map<NodeId, Vec<NodeId>>,
    /// Every retain the walk saw, so the cancellable retains are those never marked needed.
    all_retains: Vec<NodeId>,
}

impl<'a> CancelAnalysis<'a> {
    /// The unit key a leaf of this function is counted under, which a retain and a release pair on
    /// (`ownership::unit_key`).
    fn unit_key(&self, var: &FullName, path: &[usize]) -> VarPath {
        unit_key(self.vars, self.type_env, var, path)
    }

    /// Every unit an operation on a leaf of this function acts on: a pending retain on any of them
    /// is load-bearing across the operation (`ownership::acted_unit_keys`).
    fn acted_unit_keys(&self, var: &FullName, path: &[usize]) -> Vec<VarPath> {
        acted_unit_keys(self.vars, self.type_env, var, path)
    }

    /// Walk a node forward, threading the pending-retain state. `returns_from_func` marks that a terminal
    /// `Ret` here returns from the function — consuming its value and closing no bracket; inside a
    /// match arm it is false, since the arm's `Ret` flows its value to the match binding. Returns the
    /// pending state at the node's exit, so a match arm's exit can be merged into its continuation.
    fn walk(
        &mut self,
        node: &RcExprNode,
        pending: PendingRetains,
        returns_from_func: bool,
    ) -> PendingRetains {
        grow_stack(|| self.walk_inner(node, pending, returns_from_func))
    }

    /// The body of `walk`, which owns the stack growth.
    fn walk_inner(
        &mut self,
        node: &RcExprNode,
        mut pending: PendingRetains,
        returns_from_func: bool,
    ) -> PendingRetains {
        match node.expr.as_ref() {
            RcExpr::Retain(v, path, _, k) => {
                let retain = node_id(node);
                self.all_retains.push(retain);
                self.unbump_releases.entry(retain).or_default();
                pending
                    .entry(self.unit_key(&v.name, path))
                    .or_default()
                    .push(retain);
                self.walk(k, pending, returns_from_func)
            }
            RcExpr::Release(v, path, _, k) => {
                let key = self.unit_key(&v.name, path);
                // A release of a value whose object is path-dependent un-bumps a retain of that same
                // value, so it pairs on the identity; on the other objects it may be, it is a drop
                // that no pending retain of theirs may be cancelled across.
                for other in self.acted_unit_keys(&v.name, path) {
                    if other != key {
                        self.consume_unit(&mut pending, other);
                    }
                }
                // A release with nothing pending for `key` disposes of a reference this walk did not
                // add — an owned parameter, or a value produced here — so it un-bumps no retain and
                // pairs with nothing.
                if let Some(stack) = pending.get_mut(&key) {
                    // A stack kept in `pending` is never empty (emptied stacks are removed below), so a
                    // pending retain to pair with is always present.
                    let retain = stack.pop().expect("a stack kept in `pending` is non-empty");
                    self.unbump_releases
                        .entry(retain)
                        .or_default()
                        .push(node_id(node));
                    if stack.is_empty() {
                        pending.remove(&key);
                    }
                }
                self.walk(k, pending, returns_from_func)
            }
            RcExpr::Let(_, RcRhs::Match(_, arms), k) => {
                let arm_exits: Vec<PendingRetains> = arms
                    .iter()
                    .map(|arm| self.walk(&arm.body, pending.clone(), false))
                    .collect();
                let merged = self.merge(&pending, &arm_exits);
                self.walk(k, merged, returns_from_func)
            }
            RcExpr::Let(x, rhs, k) => {
                self.consume_rhs(&mut pending, rhs, &x.ty);
                self.walk(k, pending, returns_from_func)
            }
            RcExpr::Destructure(container, fields, _state, k) => {
                for leaf in destructure_consumes(container, fields, self.type_env) {
                    self.consume(&mut pending, &container.name, &leaf);
                }
                self.walk(k, pending, returns_from_func)
            }
            // `Eval` neither consumes, retains, nor releases; it is transparent to the pending-retain
            // state (any release inserted after it is a separate `Release` node).
            RcExpr::Eval(_, k) => self.walk(k, pending, returns_from_func),
            RcExpr::Ret(_) => {
                if returns_from_func {
                    // A retain still pending at the function's return closes no bracket on this path.
                    for stack in pending.values() {
                        for &retain in stack {
                            self.needed_retains.insert(retain);
                        }
                    }
                }
                pending
            }
        }
    }

    /// Mark every retain the right-hand side consumes as needed.
    fn consume_rhs(
        &mut self,
        pending: &mut PendingRetains,
        rhs: &RcRhs,
        result_ty: &Arc<TypeNode>,
    ) {
        let owns = |p: &RcVar, leaf: &FieldPath| {
            self.owned_units
                .contains(&(p.name.clone(), truncate_to_unit(&p.ty, leaf, self.type_env)))
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
            self.consume(pending, &var, &leaf);
        }
    }

    /// A consume of a leaf: every retain pending for a unit it may belong to is load-bearing here.
    fn consume(&mut self, pending: &mut PendingRetains, var: &FullName, path: &[usize]) {
        for key in self.acted_unit_keys(var, path) {
            self.consume_unit(pending, key);
        }
    }

    /// A consume of one unit: every retain pending for it is load-bearing here.
    fn consume_unit(&mut self, pending: &mut PendingRetains, key: VarPath) {
        if let Some(stack) = pending.remove(&key) {
            for retain in stack {
                self.needed_retains.insert(retain);
            }
        }
    }

    /// Merge match arms into their continuation: a retain pending in every arm's exit continues (a
    /// single downstream release un-bumps it on all paths); a retain pending in some but not all arms
    /// has a non-uniform fate and cannot be cleanly cancelled, so it is disqualified.
    fn merge(
        &mut self,
        pending_in: &PendingRetains,
        arm_exits: &[PendingRetains],
    ) -> PendingRetains {
        let n = arm_exits.len();
        let mut arms_pending: Map<NodeId, usize> = Map::default();
        for exit in arm_exits {
            let mut seen: Set<NodeId> = Set::default();
            for stack in exit.values() {
                for &retain in stack {
                    if seen.insert(retain) {
                        *arms_pending.entry(retain).or_default() += 1;
                    }
                }
            }
        }
        for (&retain, &count) in &arms_pending {
            if count != n {
                self.needed_retains.insert(retain);
            }
        }
        // Keep the retains pending in all arms, in the pre-match order so release pairing stays
        // innermost-first.
        let mut merged = PendingRetains::default();
        for (key, stack) in pending_in {
            let kept: Vec<NodeId> = stack
                .iter()
                .copied()
                .filter(|retain| arms_pending.get(retain) == Some(&n))
                .collect();
            if !kept.is_empty() {
                merged.insert(key.clone(), kept);
            }
        }
        merged
    }

    /// The nodes to delete: every cancellable retain (one never marked needed and paired by at least
    /// one release) together with the releases it pairs with.
    fn cancelled(&self) -> Set<NodeId> {
        let mut out = Set::default();
        for &retain in &self.all_retains {
            if self.needed_retains.contains(&retain) {
                continue;
            }
            // The walk records an entry for every retain it meets, and only retains it met are here.
            let releases = self
                .unbump_releases
                .get(&retain)
                .unwrap_or_else(|| unreachable!("retain {:?} was never seen by the walk", retain));
            // A retain with no un-bump release is left in place to keep the counting balanced.
            if !releases.is_empty() {
                out.insert(retain);
                out.extend(releases.iter().copied());
            }
        }
        out
    }
}

/// Rebuild a body with the analysis's cancelled retain and release nodes spliced out.
fn drop_nodes(node: &RcExprNode, to_delete: &Set<NodeId>) -> RcExprNode {
    grow_stack(|| drop_nodes_inner(node, to_delete))
}

/// One node of `drop_nodes`'s rebuild, over its rebuilt continuation.
fn drop_nodes_inner(node: &RcExprNode, to_delete: &Set<NodeId>) -> RcExprNode {
    match node.expr.as_ref() {
        RcExpr::Retain(v, path, state, k) => {
            let k = drop_nodes(k, to_delete);
            if to_delete.contains(&node_id(node)) {
                k
            } else {
                expr_node(
                    RcExpr::Retain(v.clone(), path.clone(), *state, k),
                    &node.source,
                )
            }
        }
        RcExpr::Release(v, path, state, k) => {
            let k = drop_nodes(k, to_delete);
            if to_delete.contains(&node_id(node)) {
                k
            } else {
                expr_node(
                    RcExpr::Release(v.clone(), path.clone(), *state, k),
                    &node.source,
                )
            }
        }
        RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    payload_state: arm.payload_state,
                    tag: arm.tag,
                    payload: arm.payload.clone(),
                    body: drop_nodes(&arm.body, to_delete),
                })
                .collect();
            expr_node(
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrut.clone(), arms),
                    drop_nodes(k, to_delete),
                ),
                &node.source,
            )
        }
        RcExpr::Let(x, rhs, k) => expr_node(
            RcExpr::Let(x.clone(), rhs.clone(), drop_nodes(k, to_delete)),
            &node.source,
        ),
        RcExpr::Destructure(container, fields, state, k) => expr_node(
            RcExpr::Destructure(
                container.clone(),
                fields.clone(),
                *state,
                drop_nodes(k, to_delete),
            ),
            &node.source,
        ),
        RcExpr::Eval(v, k) => expr_node(
            RcExpr::Eval(v.clone(), drop_nodes(k, to_delete)),
            &node.source,
        ),
        RcExpr::Ret(v) => expr_node(RcExpr::Ret(v.clone()), &node.source),
    }
}
