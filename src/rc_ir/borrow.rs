//! Borrow-ification over the RC IR: an `Own` parameter that a function only reads becomes a
//! `Borrow` parameter, so the caller keeps ownership across the call and needs no retain before a
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
//! closure capture, or an unboxed union (a union is one unit, since a physical refcount operation on
//! it must dispatch on the tag).
//!
//! Borrow-ification leaves the caller with a retain before a borrow call and a release after it,
//! bracketing the call with no consume between. `cancel` removes those net-zero brackets: a retain is
//! cancellable when, on every forward path, releases un-bump every reference it bumped before the
//! value is consumed. That keeps the value `Unique` for the uniqueness analysis, the reason
//! borrow-ification exists.
//!
//! Which construct consumes which reference, and which object a reference belongs to, is the shared
//! model in `ownership`, so every part of this pass gives the same answer.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{grow_stack, Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    for_each_node, for_each_var, FieldPath, FuncRef, Ownership, OwnershipShape, RcExpr, RcExprNode,
    RcFunc, RcGlobalInit, RcProgram, RcRhs, RcState, RcVar, VarPath,
};
use crate::rc_ir::leaf_map::boxed_leaf_paths;
use crate::rc_ir::ownership::{
    acted_references, all_owned_units, collect_consumes, destructure_consumes, origin, rc_units,
    rhs_consumes, truncate_to_unit, unit_step, units_under, References, UnitStep, VarTable,
};
use crate::rc_ir::rename::fresh_rename_function;
use std::sync::Arc;

/// The parameter leaves borrow inference found `Own`, keyed by the parameter variable's name and the
/// leaf path; a leaf absent from the set is `Borrow`.
///
/// These are **leaves**, one per reference a parameter holds. The `owned_units` this pass carries
/// beside them are **units**, one per reference-count operation, and `truncate_to_unit` turns a leaf
/// into the unit it keys to. Both are sets of `VarPath`, so the name is what tells one from the
/// other.
// PROOF: P5, P6, P7, P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
struct OwnedLeaves(Set<VarPath>);

impl OwnedLeaves {
    /// Whether the leaf `path` of the parameter `var` is owned.
    fn owns(&self, var: &FullName, path: &FieldPath) -> bool {
        self.0.contains(&(var.clone(), path.clone()))
    }
}

/// Infer parameter ownership for every function of `prog` by a fixed point: start every parameter
/// leaf `Borrow`, then repeatedly demote to `Own` any leaf that a consume site traces back to, until
/// nothing changes. Demotion is monotone (`Borrow` to `Own` only), so it terminates.
// PROOF: P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
fn infer_ownership(prog: &RcProgram, type_env: &TypeEnv) -> OwnedLeaves {
    let var_tables: Map<FuncRef, VarTable> = prog
        .funcs
        .values()
        .map(|f| (f.name.clone(), VarTable::of(f)))
        .collect();

    let sites: Map<FuncRef, Vec<(RcVar, FieldPath)>> = prog
        .funcs
        .values()
        .map(|f| (f.name.clone(), levelled_sites(f, type_env)))
        .collect();

    let mut owned_leaves: Set<VarPath> = Set::default();
    loop {
        let mut changed = false;
        for func in prog.funcs.values() {
            let vars = &var_tables[&func.name];
            let mut consumed = vec![];
            collect_consumes(
                &func.body,
                vars,
                prog,
                &owned_leaves,
                type_env,
                &mut consumed,
            );
            for (var, path) in consumed {
                // Attribute the consume to the parameters it may originate from, and own them. A
                // consumed leaf that is one of several objects is consumed whichever it is, so every
                // parameter it may be has to be owned.
                for (root_var, root_path) in origin(vars, type_env, &var, &path).candidates() {
                    if vars.param_tys.contains_key(root_var)
                        && owned_leaves.insert((root_var.clone(), root_path.clone()))
                    {
                        changed = true;
                    }
                }
            }
            for site in &sites[&func.name] {
                changed |= level_ownership(vars, type_env, site, &mut owned_leaves);
            }
        }
        if !changed {
            break;
        }
    }

    OwnedLeaves(owned_leaves)
}

/// The `(value, unit)` pairs whose ownership this pass reads as a single truth value: the target of
/// every reference-count node, and every argument unit of every call.
///
/// `owns_unit` answers once for such a pair, while the node it decides acts on each boxed leaf under
/// the unit. Where those leaves come from roots the version owns differently, no single answer is
/// right, so the leaves are levelled (`level_ownership`) before any of them is read.
fn levelled_sites(func: &RcFunc, type_env: &TypeEnv) -> Vec<(RcVar, FieldPath)> {
    let mut sites = vec![];
    for_each_node(&func.body, &mut |node| match node.expr.as_ref() {
        RcExpr::Retain(v, path, _, _) | RcExpr::Release(v, path, _, _) => {
            sites.push((v.clone(), path.clone()))
        }
        RcExpr::Let(_, RcRhs::App(_, args), _) => {
            for arg in args {
                for unit in rc_units(&arg.ty, type_env) {
                    sites.push((arg.clone(), unit));
                }
            }
        }
        RcExpr::Let(..) | RcExpr::Destructure(..) | RcExpr::Eval(..) | RcExpr::Ret(..) => {}
    });
    sites
}

/// Own every parameter leaf a site's candidate objects reach, where the version owns any of them.
///
/// A unit whose leaves come from different roots gets one ownership answer for all of them. Where
/// the roots disagree, neither answer is right: `Borrow` drops the release the owned leaf needed,
/// and `Own` disposes a reference the borrowed leaf was only lent. Owning all of them is the answer
/// the reference counting can express, and a value owned where it could have been borrowed costs a
/// count rather than correctness. Ownership only grows here, so the fixed point still terminates.
fn level_ownership(
    vars: &VarTable,
    type_env: &TypeEnv,
    (v, unit): &(RcVar, FieldPath),
    owned_leaves: &mut Set<VarPath>,
) -> bool {
    let candidates: Vec<VarPath> = origin(vars, type_env, &v.name, unit)
        .candidates()
        .into_iter()
        .cloned()
        .collect();
    let owns_a_candidate = candidates
        .iter()
        .any(|(root, path)| owns_object_yet(vars, type_env, root, path, owned_leaves));
    if !owns_a_candidate {
        return false;
    }
    let mut changed = false;
    for (root, path) in &candidates {
        let Some(ty) = vars.param_tys.get(root) else {
            continue;
        };
        for leaf in covered_leaves(ty, path, type_env) {
            changed |= owned_leaves.insert((root.clone(), leaf));
        }
    }
    changed
}

/// Whether the ownership decided so far already gives this version the object at `(root, path)`.
///
/// This is `RewriteCtx::owns_object` asked of the inference's own state. The two have to answer
/// alike, because `level_ownership` fires on this answer and the rewrite acts on that one. A unit is
/// owned once **any** leaf truncating to it is owned, which is how `borrow_ify` turns the inferred
/// leaves into `owned_units`, so a unit's other leaves are owned by that step whether or not the
/// inference ever named them. Reading the leaves directly here would miss exactly those, and the
/// unit would be rewritten as owned while the levelling never fired.
fn owns_object_yet(
    vars: &VarTable,
    type_env: &TypeEnv,
    root: &FullName,
    path: &FieldPath,
    owned_leaves: &Set<VarPath>,
) -> bool {
    // A root this version takes no parameter for is a producer or a global, and the version owns
    // what it produced.
    let Some(ty) = vars.param_tys.get(root) else {
        return true;
    };
    let leaves = boxed_leaf_paths(ty, type_env);
    units_under(ty, path, type_env).iter().all(|unit| {
        // `units_under` answers with the path itself where the path runs below a unit -- into a
        // variant of an unboxed union, say -- so the answer is truncated before it is a key, as
        // `owns_object` truncates it.
        let key = truncate_to_unit(ty, unit, type_env);
        leaves.iter().any(|leaf| {
            truncate_to_unit(ty, leaf, type_env) == key
                && owned_leaves.contains(&(root.clone(), leaf.clone()))
        })
    })
}

/// The boxed leaves of `ty` that `path` covers: the ones beneath it, and the one it lies beneath.
///
/// A path reaches a leaf from either side. A unit path stops above the leaves of an unboxed union's
/// variants, and a path into a variant runs below the leaf a punched array holds.
fn covered_leaves(ty: &Arc<TypeNode>, path: &FieldPath, type_env: &TypeEnv) -> Vec<FieldPath> {
    boxed_leaf_paths(ty, type_env)
        .into_iter()
        .filter(|leaf| leaf.starts_with(path) || path.starts_with(leaf))
        .collect()
}

// --- borrow-ification ---

/// Borrow-ify a program: materialize a borrowing version of every function with a borrowable
/// parameter, route each direct call to a version, rewrite the reference counting accordingly, and
/// annotate every output version with the parameter/capture units it borrows
/// (`RcFunc::borrowed_units`).
// PROOF: P3, P4, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn borrow_ify(prog: &RcProgram, type_env: &TypeEnv, develop_mode: bool) -> RcProgram {
    let owned_leaves = infer_ownership(prog, type_env);

    // The funptr functions that get a borrow version, and the name of that version. Only funptr
    // functions are considered: a closure is reached only by an indirect call, which keeps the
    // all-`Own` original, so a borrow clone of it would never be routed to.
    let mut borrow_versions: Map<FuncRef, FuncRef> = Map::default();
    let observing = funcs_observing_uniqueness(prog, type_env);
    for func in prog.funcs.values() {
        if observing.contains(&func.name) {
            continue;
        }
        if func.capture.is_none() && func_has_borrowable_param(func, &owned_leaves, type_env) {
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
        owned_units.extend(param_capture_units(func, type_env));
        // `f_borrow`: a fresh clone whose owned units are the inferred owned leaves, each truncated
        // to its unit.
        if let Some(borrow_version) = borrow_versions.get(&func.name) {
            let (clone, rename) = clone_func(func, borrow_version.clone(), &mut rename_counter);
            for p in &func.params {
                for leaf in boxed_leaf_paths(&p.ty, type_env) {
                    if owned_leaves.owns(&p.name, &leaf) {
                        let unit = truncate_to_unit(&p.ty, &leaf, type_env);
                        owned_units.insert((rename[&p.name].clone(), unit));
                    }
                }
            }
            clones.push((borrow_version.clone(), clone, rename));
        }
    }

    if develop_mode {
        check_clone_names_are_fresh(prog, clones.iter().map(|(_, _, rename)| rename));
    }

    // The parameter names and types of every output version, so a call site can read the ownership
    // of the routed callee's positions.
    let mut callee_params: Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>> = Map::default();
    for func in prog.funcs.values() {
        callee_params.insert(func.name.clone(), param_names_and_types(func));
    }
    for (borrow_version, clone, _) in &clones {
        callee_params.insert(borrow_version.clone(), param_names_and_types(clone));
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
    for (borrow_version, mut clone, _) in clones {
        let ctx = RewriteCtx::new(
            &clone,
            true,
            &owned_units,
            &borrow_versions,
            &callee_params,
            type_env,
        );
        if develop_mode {
            ctx.check_ownership_is_levelled(&clone);
        }
        clone.body = ctx.rewrite(&clone.body);
        funcs.insert(borrow_version, clone);
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
                owns_initializer: true,
                owns_storage: true,
            }
        })
        .collect();

    // Annotate every version with the parameter/capture units it borrows (those not in `owned_units`).
    for func in funcs.values_mut() {
        func.borrowed_units = param_capture_units(func, type_env)
            .into_iter()
            .filter(|unit_path| !owned_units.contains(unit_path))
            .collect();
    }

    RcProgram {
        funcs,
        globals,
        roots: prog.roots.clone(),
    }
}

/// Check that the names a clone mints are names the program does not already bind.
///
/// `fresh_rename_function` makes a name by appending its pass tag and a counter, reading none of the
/// program's names, so the name it makes is new only because no binder the earlier passes mint ends
/// in that shape. Nothing establishes that, and a collision would give two bindings one name, which
/// is what `origin` follows a value back to: cancellation would then pair a retain of one with a
/// release of the other and delete both.
///
/// Walking every binder costs a pass over the program, so this runs where the test suite runs it.
fn check_clone_names_are_fresh<'a>(
    prog: &RcProgram,
    renames: impl Iterator<Item = &'a Map<FullName, FullName>>,
) {
    let mut bound: Set<FullName> = Set::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.capture.iter()) {
            bound.insert(p.name.clone());
        }
        for_each_var(&func.body, &mut |v| {
            bound.insert(v.name.clone());
        });
    }
    for global in &prog.globals {
        for_each_var(&global.init, &mut |v| {
            bound.insert(v.name.clone());
        });
    }
    for rename in renames {
        for fresh in rename.values() {
            assert!(
                !bound.contains(fresh),
                "the clone's fresh name `{}` is already bound in the program",
                fresh.to_string()
            );
        }
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
    let mut borrow_name = name.name.clone();
    borrow_name.name.push_str("#borrow");
    FuncRef { name: borrow_name }
}

/// Whether borrow inference left any of a function's parameter leaves `Borrow`.
// PROOF: P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
fn func_has_borrowable_param(
    func: &RcFunc,
    owned_leaves: &OwnedLeaves,
    type_env: &TypeEnv,
) -> bool {
    func.params.iter().any(|p| {
        boxed_leaf_paths(&p.ty, type_env)
            .iter()
            .any(|leaf| !owned_leaves.owns(&p.name, &leaf))
    })
}

/// The functions whose body can reach an op that reports a reference count to the program
/// (`LLVMGen::observes_uniqueness`) — directly, or through a direct call to another such function.
///
/// Reaching one is over-approximated where the callee is not named: a call through a local holding a
/// closure is given an edge to every function a closure can carry, since which one it holds is
/// decided at run time (A7 declines to resolve it).
///
/// Borrowing changes what such an op reports. A borrowed parameter's reference is disposed of by the
/// caller after the call rather than by this function before the op runs, so the count the op reads
/// is one higher than it was without borrowing, and a value that was unique reads as shared.
/// `Debug::assert_unique` then halts a program that ran, and `Destructor::mutate_unique_io` copies a
/// resource it did not need to. `Std::unsafe_is_unique` documents the opposite change — a value
/// going from shared to unique as optimization removes a use — and not this one.
///
/// Only these functions are held back. A function that never reaches such an op computes the same
/// result at either count, so borrowing is free to change it.
fn funcs_observing_uniqueness(prog: &RcProgram, type_env: &TypeEnv) -> Set<FuncRef> {
    let _ = type_env;
    let mut observing: Set<FuncRef> = Set::default();
    let mut callees: Map<FuncRef, Vec<FuncRef>> = Map::default();
    // The functions a closure can carry, and the functions that call one without naming it.
    let mut closure_targets: Set<FuncRef> = Set::default();
    let mut calls_indirectly: Set<FuncRef> = Set::default();
    for (fref, func) in &prog.funcs {
        let mut cs = vec![];
        for_each_node(&func.body, &mut |node| {
            let RcExpr::Let(_, rhs, _) = node.expr.as_ref() else {
                return;
            };
            match rhs {
                RcRhs::Llvm(llvm_gen, _) => {
                    if llvm_gen.observes_uniqueness() {
                        observing.insert(fref.clone());
                    }
                }
                RcRhs::App(callee, _) => {
                    let target = FuncRef {
                        name: callee.name.clone(),
                    };
                    // A callee the program does not define is a local holding a closure, and which
                    // function it holds is decided at run time.
                    if prog.funcs.contains_key(&target) {
                        cs.push(target);
                    } else {
                        calls_indirectly.insert(fref.clone());
                    }
                }
                RcRhs::Closure(target, _) => {
                    closure_targets.insert(target.clone());
                }
                RcRhs::Var(..) | RcRhs::Match(..) => {}
            }
        });
        callees.insert(fref.clone(), cs);
    }
    // An indirect call reaches whichever function the closure holds, so give it an edge to every
    // function a closure can carry. Resolving which one is what `A7` declines to do.
    for fref in &calls_indirectly {
        callees
            .entry(fref.clone())
            .or_default()
            .extend(closure_targets.iter().cloned());
    }
    // Least fixed point over that graph: a caller of an observing function reaches the op.
    loop {
        let mut grew = false;
        for (fref, cs) in &callees {
            if !observing.contains(fref) && cs.iter().any(|c| observing.contains(c)) {
                observing.insert(fref.clone());
                grew = true;
            }
        }
        if !grew {
            return observing;
        }
    }
}

/// The name/type of each parameter and capture, in order.
fn param_names_and_types(func: &RcFunc) -> Vec<(FullName, Arc<TypeNode>)> {
    func.params
        .iter()
        .chain(func.capture.iter())
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect()
}

/// Every reference-counting unit of a function's parameters and capture, each as the
/// `(variable, unit path)` pair the owned and borrowed unit sets are keyed by.
// PROOF: P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
fn param_capture_units(func: &RcFunc, type_env: &TypeEnv) -> Vec<VarPath> {
    func.params
        .iter()
        .chain(func.capture.iter())
        .flat_map(|p| {
            rc_units(&p.ty, type_env)
                .into_iter()
                .map(|unit| (p.name.clone(), unit))
        })
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
        match unit_step(ty, type_env) {
            UnitStep::NoUnit => OwnershipShape::NoUnit,
            UnitStep::Capture {
                capture_idx,
                field_count,
            } => {
                path.push(capture_idx);
                let capture_ownership = ownership_at(path);
                path.pop();
                let mut children = vec![OwnershipShape::NoUnit; field_count];
                children[capture_idx] = OwnershipShape::Unit(capture_ownership);
                OwnershipShape::Fields(children)
            }
            UnitStep::Unit => OwnershipShape::Unit(ownership_at(path)),
            UnitStep::Fields {
                field_count,
                held_fields,
            } => {
                // A field the value holds nothing at keeps its place in the shape, so that a shape
                // index is a field index.
                let mut children = vec![OwnershipShape::NoUnit; field_count];
                for (i, fty) in held_fields {
                    path.push(i);
                    children[i] = go(var, &fty, owned_units, type_env, path);
                    path.pop();
                }
                OwnershipShape::Fields(children)
            }
        }
    }
    go(var, ty, owned_units, type_env, &mut vec![])
}

// --- tail-call recognition ---

/// The variables bound to an `App` or `Match` in tail position. Such a call must not be turned into
/// a non-tail one by an after-call release.
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
                // A tail-position result is bound by a call or a match, so the remaining shapes —
                // a call out of tail position included — leave the set alone.
                RcRhs::App(..) | RcRhs::Var(..) | RcRhs::Closure(..) | RcRhs::Llvm(..) => {}
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
        // A binding of anything but a rename of `x`, and every other construct, is a real operation
        // that breaks the chain.
        RcExpr::Let(..)
        | RcExpr::Retain(..)
        | RcExpr::Release(..)
        | RcExpr::Destructure(..)
        | RcExpr::Eval(..) => false,
    }
}

// --- cloning a function with fresh names ---

/// Clone a function as its borrow version: mint a fresh name for every bound variable (parameters,
/// capture, `let` bindings, destructure fields, match-arm payloads) and rewrite all occurrences,
/// keeping global name uniqueness. References to top-level functions stay free here, for routing to
/// retarget. Returns the clone and the binder renaming.
// PROOF: P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
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
            inline_into_callers: func.inline_into_callers,
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
    // PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    fn rewrite(&self, node: &RcExprNode) -> RcExprNode {
        grow_stack(|| self.rewrite_inner(node))
    }

    /// One node of the rewrite, rebuilt over its rewritten continuation.
    // PROOF: P3, P4, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
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
                    .map(|arm| arm.with_body(self.rewrite(&arm.body)))
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
    // PROOF: P3, P4 (dev-docs/proof/rc_ir/borrow-cancel)
    fn route(&self, x: &RcVar, callee: &RcVar, args: &[RcVar], k: &RcExprNode) -> RcVar {
        let orig = FuncRef {
            name: callee.name.clone(),
        };
        if let Some(borrow_version) = self.borrow_versions.get(&orig) {
            if self.routing_is_safe(x, args) && self.routing_saves_retain(borrow_version, args, k) {
                let mut routed = callee.clone();
                routed.name = borrow_version.name.clone();
                return routed;
            }
        }
        callee.clone()
    }

    /// A call is safe to route to the borrow version when it is not in tail position, or it passes no
    /// owned argument — so the after-call release the borrow version needs never lands on a tail call.
    // PROOF: P3, P4 (dev-docs/proof/rc_ir/borrow-cancel)
    fn routing_is_safe(&self, x: &RcVar, args: &[RcVar]) -> bool {
        !self.tail.contains(&x.name) || !args.iter().any(|a| self.any_owned_unit(a))
    }

    /// Whether routing this call to the borrow version removes a reference count it would otherwise
    /// need, for at least one argument unit. Routing helps a unit that the borrow version borrows
    /// and that would otherwise be retained. Two kinds qualify: a borrowed value, which an owning
    /// callee makes the caller retain before the call, and an owned value whose object outlives the
    /// call, where the borrow cancels the retain made ahead of it. An owned value whose object ends
    /// at the call is moved either way, so borrowing it removes no retain and only delays its
    /// release.
    // PROOF: P3, P4 (dev-docs/proof/rc_ir/borrow-cancel)
    fn routing_saves_retain(
        &self,
        borrow_version: &FuncRef,
        args: &[RcVar],
        k: &RcExprNode,
    ) -> bool {
        // `borrow_ify` registers the parameters of every version, so a borrow version is a key here.
        let borrow_params = &self.callee_params[borrow_version];
        args.iter().enumerate().any(|(arg_idx, arg)| {
            let arg_used_later = used_later(&arg.name, k);
            rc_units(&arg.ty, self.type_env).iter().any(|unit| {
                // `arg_idx` is in range since `args.len() <= params.len()`.
                let callee_borrows = !self
                    .owned_units
                    .contains(&(borrow_params[arg_idx].0.clone(), unit.clone()));
                callee_borrows
                    && !(self.owns_unit(arg, unit)
                        && !arg_used_later
                        && !self.comes_from_a_value_used_later(arg, unit, k))
            })
        })
    }

    /// Whether `arg@unit` was read out of a value this function uses after the call. Such a leaf
    /// holds a reference this function made for the call, and routing to the borrow version removes
    /// that reference together with the retain that made it.
    fn comes_from_a_value_used_later(&self, arg: &RcVar, unit: &FieldPath, k: &RcExprNode) -> bool {
        origin(&self.vars, self.type_env, &arg.name, unit)
            .candidates()
            .iter()
            .any(|(root, _)| used_later(root, k))
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
    // PROOF: P3, P4, P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
    fn owns_unit(&self, arg: &RcVar, unit: &FieldPath) -> bool {
        origin(&self.vars, self.type_env, &arg.name, unit)
            .candidates()
            .iter()
            .all(|(root, path)| self.owns_object(root, path))
    }

    /// Check that every site the inference levelled has one ownership answer for all the objects the
    /// site may act on.
    ///
    /// `owns_unit` returns one boolean for a unit, while the node it decides acts on every boxed leaf
    /// under that unit. Where the leaves come from roots this version owns differently, neither
    /// answer is right: `Borrow` drops the release the owned leaf needed, and `Own` disposes a
    /// reference the borrowed leaf was only lent. `level_ownership` is what makes the answers agree,
    /// and this states that agreement where the rewrite reads it.
    ///
    /// Only the borrow version needs checking. The all-owning original holds every parameter and
    /// capture unit in `owned_units`, so `owns_object` is true of each of its objects.
    fn check_ownership_is_levelled(&self, func: &RcFunc) {
        for (v, unit) in levelled_sites(func, self.type_env) {
            let where_from = origin(&self.vars, self.type_env, &v.name, &unit);
            let mut answers = where_from
                .candidates()
                .into_iter()
                .map(|(root, path)| self.owns_object(root, path));
            let first = answers
                .next()
                .expect("an origin reaches at least one object");
            assert!(
                answers.all(|answer| answer == first),
                "in `{}`, the ownership of `{}`{:?} splits across the objects it may act on",
                func.name.name.to_string(),
                v.name.to_string(),
                unit
            );
        }
    }

    /// Whether this version owns the object a leaf comes from.
    // PROOF: P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
    fn owns_object(&self, root: &FullName, path: &FieldPath) -> bool {
        match self.vars.param_tys.get(root) {
            // The value is owned only when every reference-counting unit the path covers is owned.
            // Each covered path is truncated to its unit, so a path that descends into a union
            // variant keys to the union itself, which is what `owned_units` records.
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
    // PROOF: P3, P4 (dev-docs/proof/rc_ir/borrow-cancel)
    fn call_rc(
        &self,
        callee: &RcVar,
        args: &[RcVar],
    ) -> (Vec<(RcVar, FieldPath)>, Vec<(RcVar, FieldPath)>) {
        let params = self.callee_params.get(&FuncRef {
            name: callee.name.clone(),
        });
        let mut before = vec![];
        let mut after = vec![];
        for (arg_idx, arg) in args.iter().enumerate() {
            for unit in rc_units(&arg.ty, self.type_env) {
                // An unresolved (indirect) callee owns every position (the all-`Own` ABI); a resolved
                // one is indexed by `arg_idx`, which is in range since `args.len() <= params.len()`.
                let callee_owns = match params {
                    None => true,
                    Some(params) => self
                        .owned_units
                        .contains(&(params[arg_idx].0.clone(), unit.clone())),
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
    // PROOF: P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
fn expr_node(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Arc::new(expr),
        source: source.clone(),
    }
}

/// A `Release` (when `is_release`) or `Retain` of `var` at `path` wrapping continuation `k`.
// PROOF: P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
fn prepend_rc(units: Vec<(RcVar, FieldPath)>, is_release: bool, k: RcExprNode) -> RcExprNode {
    units.into_iter().rev().fold(k, |cont, (var, path)| {
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
        // `Eval` observes its variable, so it counts as a use.
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
/// that every later pass sees reference counting at unit granularity. A path that already names a
/// single unit is unchanged; a whole-value retain on a fully-unboxed value (a no-op) disappears.
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
                .map(|arm| arm.with_body(split_body(&arm.body, type_env)))
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

/// The retains that have bumped references nothing has un-bumped yet, in the order the walk met
/// them, so the innermost bracket is last.
///
/// A retain is not filed under a name. `origin` has no one name for a value whose boxed leaves reach
/// several objects — an `Option (a, b)` holds two — so filing one would mean making a name up, and a
/// release of one of those objects, named after the object, would be filed elsewhere and never meet
/// the retain. What decides whether a release closes a retain is the objects the two act on, which
/// `References` already carries.
type PendingRetains = Vec<PendingRetain>;

/// A retain whose bump is still outstanding, and the references of it that no release has un-bumped
/// yet.
///
/// A retain of an unboxed union bumps every reference its payload holds, while a release of a
/// projection of that payload un-bumps one of them, so a group of releases un-bumps one retain.
/// Cancelling it takes the whole group, and only once the group leaves nothing outstanding.
#[derive(Clone)]
struct PendingRetain {
    /// The retain node. The releases that un-bump it are recorded under this id.
    node: NodeId,
    /// The references the retain bumped that are still bumped here.
    outstanding: References,
}

/// What a release does to the retains pending where it happens.
enum UnBump {
    /// It un-bumps part or all of the innermost bracket that acts on what it acts on, whose retain
    /// this is.
    InBracket(NodeId),
    /// The innermost such bracket did not bump everything it disposes.
    OutsideBracket,
    /// No pending retain acts on any object it acts on.
    NoBracket,
}

/// Take a release's references off the innermost pending retain that acts on an object it acts on.
///
/// Innermost first, so the un-bump closes the tightest bracket around the release and leaves the
/// ones outside it bumped. A retain that shares an object with the release without carrying
/// everything it disposes is the release reaching past that bracket. A retain left with nothing
/// outstanding is un-bumped whole and stops being pending.
fn un_bump(pending: &mut PendingRetains, un_bumped: &References) -> UnBump {
    // A release no pending retain shares an object with disposes of a reference the walk did not
    // add — an owned parameter, or a value produced here — so it un-bumps no retain.
    let Some(index) = pending
        .iter()
        .rposition(|retain| retain.outstanding.shares_an_object(un_bumped))
    else {
        return UnBump::NoBracket;
    };
    let innermost = &mut pending[index];
    if !innermost.outstanding.covers(un_bumped) {
        return UnBump::OutsideBracket;
    }
    innermost.outstanding.subtract(un_bumped);
    let retain = innermost.node;
    if innermost.outstanding.is_empty() {
        pending.remove(index);
    }
    UnBump::InBracket(retain)
}

/// A node's identity within one tree: the address of its expression, stable while the tree is
/// borrowed. Nodes to drop are recorded under this identity and recognized by it again in a later
/// walk over the same borrowed tree.
type NodeId = usize;

/// The `NodeId` of a node: the address of its boxed `RcExpr`.
// PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
fn node_id(node: &RcExprNode) -> NodeId {
    node.expr.as_ref() as *const RcExpr as NodeId
}

/// Remove the net-zero retain/release brackets borrow-ification leaves across borrow calls: a retain
/// is cancellable when, on every forward path, releases un-bump every reference it bumped before the
/// value is consumed. Cancelling it (and the releases that un-bump it) keeps the value `Unique` for
/// the uniqueness analysis. Each call's consume sites are decided by the parameter/capture units the
/// functions own — the complement of their `RcFunc::borrowed_units`, set by borrow-ification.
// PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn cancel(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let owned_units = all_owned_units(prog, type_env);
    let cancel_body = |vars: &VarTable, body: &RcExprNode| {
        let mut analysis = CancelAnalysis {
            vars,
            prog,
            owned_units: &owned_units,
            type_env,
            needed_retains: Set::default(),
            un_bump_releases: Map::default(),
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
                owns_initializer: true,
                owns_storage: true,
            }
        })
        .collect();
    RcProgram {
        funcs,
        globals,
        roots: prog.roots.clone(),
    }
}

/// The forward must-analysis for one function: it decides which retain and release nodes to delete.
struct CancelAnalysis<'a> {
    /// This function's variables: what binds each one and its type, which decide the objects a
    /// retain and a release act on, and so which of them pair.
    vars: &'a VarTable,
    /// The whole program, so a call resolves to its callee's parameters.
    prog: &'a RcProgram,
    /// The parameter/capture units the program's functions own, which decide which argument
    /// positions of a call consume.
    owned_units: &'a Set<VarPath>,
    /// The type definitions, for resolving a value's type to its reference-counting units.
    type_env: &'a TypeEnv,
    /// Retains that are load-bearing on some path, so they cannot be cancelled.
    needed_retains: Set<NodeId>,
    /// The releases each retain is un-bumped by; they are deleted together with the retain.
    un_bump_releases: Map<NodeId, Vec<NodeId>>,
    /// Every retain the walk saw, so the cancellable retains are those never marked needed.
    all_retains: Vec<NodeId>,
}

impl<'a> CancelAnalysis<'a> {
    /// The references a reference-count node of this function acts on, which decide whether a
    /// release un-bumps a retain (`ownership::acted_references`).
    // PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    fn acted_references(&self, v: &RcVar, path: &FieldPath) -> References {
        let references = acted_references(self.vars, self.type_env, v, path);
        // Reference counting is inserted only for a value that holds a reference, and
        // `split_rc_units` leaves every node naming one unit of its value's type, so a node always
        // acts on at least one reference. A node acting on none would make both sides of the pairing
        // vacuous: such a retain would be un-bumped whole by the first release of its unit, whatever
        // that release reaches, and such a release would count as un-bumping a retain it leaves
        // fully bumped.
        assert!(
            !references.is_empty(),
            "the reference count of `{}`{:?} acts on no reference",
            v.name.to_string(),
            path
        );
        references
    }

    /// Walk a node forward, threading the pending-retain state. `returns_from_func` marks that a terminal
    /// `Ret` here returns from the function — consuming its value and closing no bracket; inside a
    /// match arm it is false, since the arm's `Ret` flows its value to the match binding. Returns the
    /// pending state at the node's exit, so a match arm's exit can be merged into its continuation.
    // PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    fn walk(
        &mut self,
        node: &RcExprNode,
        pending: PendingRetains,
        returns_from_func: bool,
    ) -> PendingRetains {
        grow_stack(|| self.walk_inner(node, pending, returns_from_func))
    }

    /// One node of the walk, threading the pending-retain state through its continuation and arms.
    // PROOF: P3, P4, P5, P6, P7, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
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
                self.un_bump_releases.entry(retain).or_default();
                let outstanding = self.acted_references(v, path);
                pending.push(PendingRetain {
                    node: retain,
                    outstanding,
                });
                self.walk(k, pending, returns_from_func)
            }
            RcExpr::Release(v, path, _, k) => {
                // A release of a value whose object is path-dependent un-bumps a retain of that same
                // value, so it pairs on the reference it names. On the other objects it may be, it
                // is a drop: a retain of one of them that is still pending cannot be cancelled
                // across it.
                let others = self.other_objects(v, path);
                self.consume_objects(&mut pending, &others);
                let un_bumped = self.acted_references(v, path);
                match un_bump(&mut pending, &un_bumped) {
                    UnBump::InBracket(retain) => self
                        .un_bump_releases
                        .entry(retain)
                        .or_default()
                        .push(node_id(node)),
                    // A release that reaches references the innermost bracket did not bump closes
                    // no bracket here. It un-bumps part of what retains outside that bracket
                    // bumped, so no retain acting on those objects can be cancelled as a whole.
                    UnBump::OutsideBracket => {
                        let objects = un_bumped.objects();
                        self.consume_objects(&mut pending, &objects)
                    }
                    UnBump::NoBracket => {}
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
            // `Eval` only observes its variable, so it is transparent to the pending-retain state
            // (any release inserted after it is a separate `Release` node).
            RcExpr::Eval(_, k) => self.walk(k, pending, returns_from_func),
            RcExpr::Ret(_) => {
                if returns_from_func {
                    // A retain still pending at the function's return closes no bracket on this path.
                    for retain in &pending {
                        self.needed_retains.insert(retain.node);
                    }
                }
                pending
            }
        }
    }

    /// Mark every retain the right-hand side consumes as needed.
    // PROOF: P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
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

    /// A consume of a leaf: every retain pending on an object the leaf may belong to is
    /// load-bearing here.
    fn consume(&mut self, pending: &mut PendingRetains, var: &FullName, path: &[usize]) {
        let objects: Vec<VarPath> = origin(self.vars, self.type_env, var, path)
            .acted_on()
            .into_iter()
            .cloned()
            .collect();
        self.consume_objects(pending, &objects);
    }

    /// A consume of some objects: every retain pending on one of them is load-bearing here, so it
    /// leaves the pending list without having been un-bumped.
    ///
    fn consume_objects(&mut self, pending: &mut PendingRetains, objects: &[VarPath]) {
        pending.retain(|retain| {
            if objects
                .iter()
                .any(|object| retain.outstanding.names(object))
            {
                self.needed_retains.insert(retain.node);
                return false;
            }
            true
        });
    }

    /// The objects a reference-count node on `(v, path)` may act on besides the ones it names: for
    /// each boxed leaf under the path, the candidates of its origin other than the one the leaf's
    /// reference is counted under.
    fn other_objects(&self, v: &RcVar, path: &FieldPath) -> Vec<VarPath> {
        let mut out = vec![];
        for leaf in boxed_leaf_paths(&v.ty, self.type_env) {
            if !leaf.starts_with(path) {
                continue;
            }
            let where_from = origin(self.vars, self.type_env, &v.name, &leaf);
            let identity = where_from.identity().clone();
            out.extend(
                where_from
                    .candidates()
                    .into_iter()
                    .filter(|candidate| **candidate != identity)
                    .cloned(),
            );
        }
        out
    }

    /// Merge match arms into their continuation. A retain the match was entered with continues when
    /// every arm exits with the same references of it still outstanding, since a single downstream
    /// release then un-bumps it on all paths. Two kinds are disqualified instead: a retain the arms
    /// leave in different states, whose fate is non-uniform, and a retain an arm created itself,
    /// which the merged state has no place for — that state is built over the retains the match was
    /// entered with.
    // PROOF: P3, P4, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    fn merge(
        &mut self,
        pending_in: &PendingRetains,
        arm_exits: &[PendingRetains],
    ) -> PendingRetains {
        // What each arm exits with for each retain. A retain absent from an arm's exit was fully
        // un-bumped on that path.
        let arm_states: Vec<Map<NodeId, &References>> = arm_exits
            .iter()
            .map(|exit| {
                exit.iter()
                    .map(|retain| (retain.node, &retain.outstanding))
                    .collect()
            })
            .collect();
        let entered_with: Set<NodeId> = pending_in.iter().map(|retain| retain.node).collect();
        let mut uniform: Map<NodeId, References> = Map::default();
        for states in &arm_states {
            for (&retain, &outstanding) in states {
                let is_uniform = entered_with.contains(&retain)
                    && arm_states
                        .iter()
                        .all(|other| other.get(&retain) == Some(&outstanding));
                if is_uniform {
                    uniform.insert(retain, outstanding.clone());
                } else {
                    self.needed_retains.insert(retain);
                }
            }
        }
        // Keep the retains the arms agree on, in the pre-match order so release pairing stays
        // innermost-first.
        pending_in
            .iter()
            .filter_map(|retain| {
                uniform.get(&retain.node).map(|outstanding| PendingRetain {
                    node: retain.node,
                    outstanding: outstanding.clone(),
                })
            })
            .collect()
    }

    /// The nodes to delete: every cancellable retain (one never marked needed and un-bumped by at
    /// least one release) together with the group of releases that un-bump it.
    // PROOF: P3, P4 (dev-docs/proof/rc_ir/borrow-cancel)
    fn cancelled(&self) -> Set<NodeId> {
        let mut out = Set::default();
        for &retain in &self.all_retains {
            if self.needed_retains.contains(&retain) {
                continue;
            }
            // The walk records an entry for every retain it meets, and only retains it met are here.
            let releases = self
                .un_bump_releases
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
                .map(|arm| arm.with_body(drop_nodes(&arm.body, to_delete)))
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
