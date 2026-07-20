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
//! keeps the value `Unique` for the uniqueness analysis, the reason borrow-ification exists. The
//! cancellation shares the object-identity (`root`) and consume-site machinery with the inference and
//! rewrite above, so all three read the same aliasing facts.

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::fixstd::builtin::InlineLLVMMakeUnionBody;
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    FuncRef, MatchArm, Ownership, OwnershipShape, Path, RcExpr, RcExprNode, RcFunc, RcGlobalInit,
    RcProgram, RcRhs, RcState, RcUnit, RcVar,
};
use crate::rc_ir::provenance::{boxed_leaf_paths, BaseSource, Provenance};
use crate::rc_ir::rename::fresh_rename_function;
use std::sync::Arc;

/// What binds a variable, enough to trace a leaf back to the object that produced it (its `root`).
enum Def {
    /// A parameter or capture — the origin of a leaf.
    Param,
    /// `let x = y`: a move-bind, transparent to `root`.
    Move(RcVar),
    /// `let x = op(args)`: an alias when the result leaf is a pure projection of one argument,
    /// otherwise a producer. Carries the result type to consult `result_prov`.
    Llvm(Box<dyn LLVMGen>, Vec<RcVar>, Arc<TypeNode>),
    /// `let x = f(args)` or a closure or a match — an opaque producer.
    Producer,
    /// A `destructure` field: field `idx` of the container.
    Field(RcVar, usize),
    /// A `match`-arm payload: the variant tag (`None` for a catch-all), and the scrutinee.
    Payload(RcVar, Option<usize>),
}

/// The per-function facts `root` and the consume walk need: how each local variable is bound, which
/// closure value targets which function, the function's own parameters, and every variable's type
/// (so a leaf that roots at any variable can be clamped to its reference-counting unit).
struct FuncFacts {
    defs: Map<FullName, Def>,
    closure_targets: Map<FullName, FuncRef>,
    params: Map<FullName, Arc<TypeNode>>,
    types: Map<FullName, Arc<TypeNode>>,
}

/// The result of borrow inference: which parameter leaves are `Own` (all others are `Borrow`), keyed
/// by the parameter variable's name and the leaf path.
struct Ownerships {
    own: Set<RcUnit>,
}

/// Infer parameter ownership for every function of `prog` by a fixed point: start every parameter
/// leaf `Borrow`, then repeatedly demote to `Own` any leaf that a consume site traces back to, until
/// nothing changes. Demotion is monotone (`Borrow` to `Own` only), so it terminates.
fn infer_ownership(prog: &RcProgram, type_env: &TypeEnv) -> Ownerships {
    let facts: Map<FuncRef, FuncFacts> = prog
        .funcs
        .values()
        .map(|f| (f.name.clone(), FuncFacts::of(f)))
        .collect();

    let mut own: Set<RcUnit> = Set::default();
    loop {
        let mut changed = false;
        for func in prog.funcs.values() {
            let facts = &facts[&func.name];
            let mut consumed = vec![];
            collect_consumes(&func.body, facts, prog, &own, type_env, &mut consumed);
            for (var, path) in consumed {
                let (root_var, root_path) = root(facts, type_env, &var, &path);
                // Attribute the consume to the parameter it originates from, if any, and own it.
                if facts.params.contains_key(&root_var) && own.insert((root_var, root_path)) {
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }

    Ownerships { own }
}

impl FuncFacts {
    /// The facts of a function: its parameters and capture as `Param` origins, plus the `Def` and
    /// type of every variable bound in its body.
    fn of(func: &RcFunc) -> FuncFacts {
        let mut facts = FuncFacts::empty();
        for p in func.params.iter().chain(func.cap.iter()) {
            facts.defs.insert(p.name.clone(), Def::Param);
            facts.params.insert(p.name.clone(), p.ty.clone());
            facts.types.insert(p.name.clone(), p.ty.clone());
        }
        collect_defs(&func.body, &mut facts);
        facts
    }

    /// The facts of a param-less body (a global initializer).
    fn body_only(body: &RcExprNode) -> FuncFacts {
        let mut facts = FuncFacts::empty();
        collect_defs(body, &mut facts);
        facts
    }

    fn empty() -> FuncFacts {
        FuncFacts {
            defs: Map::default(),
            closure_targets: Map::default(),
            params: Map::default(),
            types: Map::default(),
        }
    }
}

/// Record every local variable's `Def` and type (and any closure value's target function) in a
/// function body.
fn collect_defs(node: &RcExprNode, facts: &mut FuncFacts) {
    match node.expr.as_ref() {
        RcExpr::Ret(_) => {}
        RcExpr::Let(x, rhs, k) => {
            let def = match rhs {
                RcRhs::Var(y) => Def::Move(y.clone()),
                RcRhs::Llvm(gen, args) => Def::Llvm(gen.clone(), args.clone(), x.ty.clone()),
                RcRhs::Closure(fref, _) => {
                    facts.closure_targets.insert(x.name.clone(), fref.clone());
                    Def::Producer
                }
                RcRhs::App(..) => Def::Producer,
                RcRhs::Match(scrut, arms) => {
                    for arm in arms {
                        facts.defs.insert(
                            arm.payload.name.clone(),
                            Def::Payload(scrut.clone(), arm.variant),
                        );
                        facts
                            .types
                            .insert(arm.payload.name.clone(), arm.payload.ty.clone());
                        collect_defs(&arm.body, facts);
                    }
                    Def::Producer
                }
            };
            facts.defs.insert(x.name.clone(), def);
            facts.types.insert(x.name.clone(), x.ty.clone());
            collect_defs(k, facts);
        }
        RcExpr::Destructure(container, fields, k) => {
            for (idx, fv) in fields {
                facts
                    .defs
                    .insert(fv.name.clone(), Def::Field(container.clone(), *idx));
                facts.types.insert(fv.name.clone(), fv.ty.clone());
            }
            collect_defs(k, facts);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            collect_defs(k, facts)
        }
    }
}

/// The object a leaf originates from: follow alias edges (move-binds, pure projections, unboxed-union
/// payloads) back to the producing variable and path. The returned variable is a parameter when the
/// leaf ultimately comes from an input.
fn root(facts: &FuncFacts, type_env: &TypeEnv, var: &FullName, path: &[usize]) -> RcUnit {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        root_inner(facts, type_env, var, path)
    })
}

fn root_inner(facts: &FuncFacts, type_env: &TypeEnv, var: &FullName, path: &[usize]) -> RcUnit {
    let here = || (var.clone(), path.to_vec());
    match facts.defs.get(var) {
        None | Some(Def::Param) | Some(Def::Producer) => here(),
        Some(Def::Move(y)) => root(facts, type_env, &y.name, path),
        Some(Def::Llvm(gen, args, result_ty)) => {
            // Constructing an unboxed union lays its payload in place, so the whole union's root is
            // the payload's root — the construction alias edge, dual to reading a payload out with
            // `match`. The whole-union path is where this matters: a leaf path descends into the
            // active variant, which the projection rule below already aliases through `result_prov`.
            if path.is_empty()
                && !args.is_empty()
                && gen.as_any().is::<InlineLLVMMakeUnionBody>()
                && !result_ty.is_box(type_env)
            {
                return root(facts, type_env, &args[0].name, &[]);
            }
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            let decl = gen.result_prov(result_ty, &arg_tys, type_env);
            // A result leaf that is a single `Arg(j, p)` is a pure projection of argument `j`'s leaf
            // `p` — an alias; anything else (a fresh allocation, a boxed-container read, a join of
            // several sources) is a producer, stopping here. An `Llvm` op is never partially applied,
            // so a well-formed `result_prov` names only real argument indices (`args[j]` else panics).
            match single_arg(&decl.leaf_at(path)) {
                Some((j, p)) => root(facts, type_env, &args[j].name, &p),
                None => here(),
            }
        }
        Some(Def::Field(container, idx)) => {
            if container.ty.is_box(type_env) {
                // Reading a field of a boxed struct retains it: a producer.
                here()
            } else {
                let mut p = vec![*idx];
                p.extend_from_slice(path);
                root(facts, type_env, &container.name, &p)
            }
        }
        Some(Def::Payload(scrut, variant)) => match variant {
            // A catch-all binds the whole scrutinee: the same object.
            None => root(facts, type_env, &scrut.name, path),
            // An unboxed union's payload is the scrutinee's variant slot — an alias; a boxed union's
            // payload is read out (retained) — a producer.
            Some(k) if !scrut.ty.is_box(type_env) => {
                let mut p = vec![*k];
                p.extend_from_slice(path);
                root(facts, type_env, &scrut.name, &p)
            }
            Some(_) => here(),
        },
    }
}

/// The single `Arg(j, p)` a leaf source consists of, if it is exactly that.
fn single_arg(ls: &Set<BaseSource>) -> Option<(usize, Path)> {
    if ls.len() != 1 {
        return None;
    }
    match ls.iter().next() {
        Some(BaseSource::Arg(j, p)) => Some((*j, p.clone())),
        _ => None,
    }
}

/// Collect the leaves consumed in a function body, given `own` as the owned parameter leaves that
/// decide which argument positions consume: an owning argument position, a captured value, or a
/// returned value. Alias edges are not consumes here — the consume of an alias is attributed to its
/// `root`. Explicit `Release` nodes are own-then-release drops, not consumes.
fn collect_consumes(
    node: &RcExprNode,
    facts: &FuncFacts,
    prog: &RcProgram,
    own: &Set<RcUnit>,
    type_env: &TypeEnv,
    out: &mut Vec<RcUnit>,
) {
    let owns = |p: &RcVar, pi: &Path| own.contains(&(p.name.clone(), pi.clone()));
    collect_consumes_go(node, facts, prog, type_env, &owns, out);
}

fn collect_consumes_go<F: Fn(&RcVar, &Path) -> bool>(
    node: &RcExprNode,
    facts: &FuncFacts,
    prog: &RcProgram,
    type_env: &TypeEnv,
    owns: &F,
    out: &mut Vec<RcUnit>,
) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => push_leaves(&x.name, &x.ty, type_env, out),
        RcExpr::Let(x, rhs, k) => {
            match rhs {
                RcRhs::Match(_, arms) => {
                    for arm in arms {
                        collect_consumes_go(&arm.body, facts, prog, type_env, owns, out);
                    }
                }
                _ => rhs_consumes(rhs, &x.ty, facts, prog, type_env, owns, out),
            }
            collect_consumes_go(k, facts, prog, type_env, owns, out);
        }
        RcExpr::Destructure(container, fields, k) => {
            for pi in destructure_consumes(container, fields, type_env) {
                out.push((container.name.clone(), pi));
            }
            collect_consumes_go(k, facts, prog, type_env, owns, out)
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            collect_consumes_go(k, facts, prog, type_env, owns, out)
        }
    }
}

/// The container leaves a `Destructure` consumes. A boxed container is released whole, so every boxed
/// leaf of it goes; an unboxed container moves each named field's leaves into that field's variable,
/// an alias whose consume is attributed to the field variable, so only a dropped (unnamed) field's
/// leaves go. This is the model code generation implements (`ObjectFieldType::get_struct_fields`), and
/// every reader of the consume model shares it.
fn destructure_consumes(
    container: &RcVar,
    fields: &[(usize, RcVar)],
    type_env: &TypeEnv,
) -> Vec<Path> {
    let leaves = boxed_leaves(&container.ty, type_env);
    if container.ty.is_box(type_env) {
        return leaves;
    }
    let named: Set<usize> = fields.iter().map(|(i, _)| *i).collect();
    leaves
        .into_iter()
        .filter(|pi| {
            // A boxed leaf of an unboxed container starts with a field index, so its path is non-empty.
            let field = pi
                .first()
                .expect("a boxed leaf of an unboxed container has a non-empty path");
            !named.contains(field)
        })
        .collect()
}

/// The leaves an `App`, `Llvm`, or `Closure` right-hand side consumes: an owning argument position
/// (`owns` decides, for the callee's parameter leaf), a captured value, and the closure callee. A
/// `Var` move and a `Match` consume nothing here — a move is an alias, and a match's consumes live in
/// its arms. `result_ty` is the type the right-hand side binds, needed to read an op's passthrough.
fn rhs_consumes<F: Fn(&RcVar, &Path) -> bool>(
    rhs: &RcRhs,
    result_ty: &Arc<TypeNode>,
    facts: &FuncFacts,
    prog: &RcProgram,
    type_env: &TypeEnv,
    owns: &F,
    out: &mut Vec<RcUnit>,
) {
    match rhs {
        RcRhs::Var(_) | RcRhs::Match(..) => {}
        RcRhs::Closure(_, caps) => {
            for c in caps {
                push_leaves(&c.name, &c.ty, type_env, out);
            }
        }
        RcRhs::App(callee, args) => {
            // Calling a closure consumes it (the callee releases its capture).
            push_leaves(&callee.name, &callee.ty, type_env, out);
            // Each argument at an owning position of the callee is consumed. An unresolved (indirect)
            // callee owns every position.
            let callee_params = resolve_callee_params(callee, facts, prog);
            for (i, a) in args.iter().enumerate() {
                for pi in boxed_leaves(&a.ty, type_env) {
                    // `i` ranges over the arguments and `args.len() <= params.len()` (no over-
                    // application), so `params[i]` is in range.
                    let owns_pos = match &callee_params {
                        Some(params) => owns(&params[i], &pi),
                        None => true,
                    };
                    if owns_pos {
                        out.push((a.name.clone(), pi));
                    }
                }
            }
        }
        RcRhs::Llvm(gen, args) => {
            let passthrough = passthrough_arg_leaves(&**gen, result_ty, args, type_env);
            for (i, a) in args.iter().enumerate() {
                if gen.borrows_operand(i) {
                    continue;
                }
                for pi in boxed_leaves(&a.ty, type_env) {
                    // An argument leaf that the op passes through to its result is not consumed;
                    // anything else at an owning position is moved into the op.
                    if !passthrough.contains(&(i, pi.clone())) {
                        out.push((a.name.clone(), pi));
                    }
                }
            }
        }
    }
}

/// The parameters of a directly-called callee: a closure value built in this function, or a
/// top-level function referenced by name. `None` for an indirect call (an owning, all-`Own` ABI).
fn resolve_callee_params<'a>(
    callee: &RcVar,
    facts: &FuncFacts,
    prog: &'a RcProgram,
) -> Option<&'a [RcVar]> {
    let fref = facts
        .closure_targets
        .get(&callee.name)
        .cloned()
        .or_else(|| {
            let fref = FuncRef {
                name: callee.name.clone(),
            };
            prog.funcs.contains_key(&fref).then_some(fref)
        })?;
    prog.funcs.get(&fref).map(|f| f.params.as_slice())
}

/// The `(arg index, leaf path)` pairs an LLVM op passes through unchanged to its result — the pure
/// projections declared by `result_prov` as `Arg(i, path)`.
fn passthrough_arg_leaves(
    gen: &dyn LLVMGen,
    result_ty: &Arc<TypeNode>,
    args: &[RcVar],
    type_env: &TypeEnv,
) -> Set<(usize, Path)> {
    let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
    let decl = gen.result_prov(result_ty, &arg_tys, type_env);
    let mut out = Set::default();
    collect_arg_leaves(&decl, &mut out);
    out
}

/// Collect every `Arg(i, path)` symbol appearing in a declared provenance.
fn collect_arg_leaves(prov: &Provenance, out: &mut Set<(usize, Path)>) {
    for ls in prov.leaves() {
        for s in ls {
            if let BaseSource::Arg(i, p) = s {
                out.insert((*i, p.clone()));
            }
        }
    }
}

/// Push every boxed leaf of a value onto `out`.
fn push_leaves(var: &FullName, ty: &Arc<TypeNode>, type_env: &TypeEnv, out: &mut Vec<RcUnit>) {
    for p in boxed_leaves(ty, type_env) {
        out.push((var.clone(), p));
    }
}

/// The paths of every boxed leaf of a type: the whole value if boxed, the capture of a closure, or
/// each boxed leaf of an unboxed aggregate.
fn boxed_leaves(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Path> {
    boxed_leaf_paths(ty, type_env)
}

// --- reference-counting units ---

/// The reference-counting units of a value's type: the capture of a closure, or each unit root
/// (`is_rc_unit_root`) — a boxed value, an unboxed union, or a punched array — reached by descending
/// its unboxed structs/tuples. Unlike `boxed_leaves`, it stops at a unit root rather than expanding it
/// into the inner boxed leaves (e.g. an unboxed union is one unit, since only its active variant is
/// live and a refcount operation must dispatch on the tag rather than name a variant's leaf).
fn rc_units(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Path> {
    let mut out = vec![];
    rc_units_go(ty, type_env, &mut vec![], &mut out);
    out
}

fn rc_units_go(ty: &Arc<TypeNode>, type_env: &TypeEnv, path: &mut Path, out: &mut Vec<Path>) {
    if ty.is_fully_unboxed(type_env) {
        return;
    }
    if ty.is_closure() {
        path.push(CLOSURE_CAPTURE_IDX as usize);
        out.push(path.clone());
        path.pop();
        return;
    }
    if ty.is_rc_unit_root(type_env) {
        out.push(path.clone());
        return;
    }
    let fields = ty.fields(type_env);
    for (i, fty) in ty.field_types(type_env).iter().enumerate() {
        // A punched struct field is a hole (its value has moved out): the whole-value traversal skips
        // it, so it names no unit.
        if fields[i].is_punched {
            continue;
        }
        path.push(i);
        rc_units_go(fty, type_env, path, out);
        path.pop();
    }
}

/// Truncate a leaf path to its reference-counting unit: the path down to the first unit root
/// (`is_rc_unit_root`) it reaches — an unboxed union or a punched array, whose subtree is one unit.
/// Paths that stay within unboxed structs are unchanged.
fn clamp_unit(ty: &Arc<TypeNode>, path: &[usize], type_env: &TypeEnv) -> Path {
    let mut out = vec![];
    let mut cur = ty.clone();
    for &idx in path {
        if cur.is_closure() {
            // The only path into a closure names its capture, which is a single unit.
            out.push(idx);
            break;
        }
        if cur.is_rc_unit_root(type_env) {
            // A boxed value, an unboxed union, or a punched array is one unit; a leaf below it (a
            // boxed leaf under a union variant, or the punched array's inner array) keys to its root.
            break;
        }
        // Here `cur` is an unboxed struct/tuple, so a well-formed unit/root path index is in range.
        let fields = cur.field_types(type_env);
        assert!(
            idx < fields.len(),
            "clamp_unit: path index {} out of range ({} fields)",
            idx,
            fields.len()
        );
        out.push(idx);
        cur = fields[idx].clone();
    }
    out
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
        if func.cap.is_none() && func_has_borrowable_param(func, &ownerships, type_env) {
            borrow_versions.insert(func.name.clone(), borrow_funcref(&func.name));
        }
    }

    // The owned parameter units of every output version, keyed by the version's own parameter names:
    // an original (`f_own`) owns all of them, a borrow clone (`f_borrow`) owns the inferred subset.
    let mut own_out: Set<RcUnit> = Set::default();
    let mut counter: u64 = 0;
    let mut clones: Vec<(FuncRef, RcFunc, Map<FullName, FullName>)> = vec![];
    for func in prog.funcs.values() {
        // `f_own`: every parameter and capture unit is owned.
        for p in func.params.iter().chain(func.cap.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                own_out.insert((p.name.clone(), unit));
            }
        }
        // `f_borrow`: a fresh clone whose owned units are the inferred ones, clamped to units.
        if let Some(bref) = borrow_versions.get(&func.name) {
            let (clone, rename) = clone_func(func, bref.clone(), &mut counter);
            for p in &func.params {
                for leaf in boxed_leaves(&p.ty, type_env) {
                    if ownerships.own.contains(&(p.name.clone(), leaf.clone())) {
                        let unit = clamp_unit(&p.ty, &leaf, type_env);
                        own_out.insert((rename[&p.name].clone(), unit));
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
        callee_params.insert(func.name.clone(), param_name_tys(func));
    }
    for (bref, clone, _) in &clones {
        callee_params.insert(bref.clone(), param_name_tys(clone));
    }

    // Rewrite every version's body: route its calls and adjust the reference counting.
    let mut funcs: Map<FuncRef, RcFunc> = Map::default();
    for func in prog.funcs.values() {
        let mut f_own = func.clone();
        let ctx = RewriteCtx::new(
            &f_own,
            false,
            &own_out,
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
            &own_out,
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
            let facts = FuncFacts::body_only(&g.init);
            let ctx = RewriteCtx {
                type_env,
                is_borrow: false,
                own_out: &own_out,
                borrow_versions: &borrow_versions,
                callee_params: &callee_params,
                tail: tail_apps(&g.init),
                facts,
            };
            RcGlobalInit {
                symbol: g.symbol.clone(),
                ty: g.ty.clone(),
                init: ctx.rewrite(&g.init),
            }
        })
        .collect();

    // Annotate every version with the parameter/capture units it borrows (those not in `own_out`).
    for func in funcs.values_mut() {
        let mut borrowed = Set::default();
        for p in func.params.iter().chain(func.cap.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                let leaf = (p.name.clone(), unit);
                if !own_out.contains(&leaf) {
                    borrowed.insert(leaf);
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

/// The owned parameter/capture units of every function: each version's units minus the ones it
/// borrows (`RcFunc::borrowed_units`, the annotation borrow-ification writes).
fn all_owned_units(prog: &RcProgram, type_env: &TypeEnv) -> Set<RcUnit> {
    let mut owned = Set::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.cap.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                let leaf = (p.name.clone(), unit);
                if !func.borrowed_units.contains(&leaf) {
                    owned.insert(leaf);
                }
            }
        }
    }
    owned
}

/// Each parameter/capture variable's ownership shape, derived from the functions' ownership
/// annotations. The RC IR dump reads it to annotate parameters; it is not needed for code generation.
pub fn param_ownership_shapes(
    prog: &RcProgram,
    type_env: &TypeEnv,
) -> Map<FullName, OwnershipShape> {
    let own_out = all_owned_units(prog, type_env);
    let mut shapes = Map::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.cap.iter()) {
            shapes.insert(
                p.name.clone(),
                shape_from_own(&p.name, &p.ty, &own_out, type_env),
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
fn param_name_tys(func: &RcFunc) -> Vec<(FullName, Arc<TypeNode>)> {
    func.params
        .iter()
        .chain(func.cap.iter())
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect()
}

/// The ownership shape of one parameter, read from the owned-unit set: `Own` at a reference-counting
/// unit that is owned, else `Borrow`.
fn shape_from_own(
    var: &FullName,
    ty: &Arc<TypeNode>,
    own_out: &Set<RcUnit>,
    type_env: &TypeEnv,
) -> OwnershipShape {
    fn go(
        var: &FullName,
        ty: &Arc<TypeNode>,
        own_out: &Set<RcUnit>,
        type_env: &TypeEnv,
        path: &mut Path,
    ) -> OwnershipShape {
        let owned = |path: &Path| {
            if own_out.contains(&(var.clone(), path.clone())) {
                Ownership::Own
            } else {
                Ownership::Borrow
            }
        };
        if ty.is_fully_unboxed(type_env) {
            return OwnershipShape::Unboxed;
        }
        if ty.is_closure() {
            path.push(CLOSURE_CAPTURE_IDX as usize);
            let cap = owned(path);
            path.pop();
            return OwnershipShape::UnboxedAgg(vec![
                OwnershipShape::Unboxed,
                OwnershipShape::Boxed(cap),
            ]);
        }
        if ty.is_rc_unit_root(type_env) {
            return OwnershipShape::Boxed(owned(path));
        }
        let fields = ty.field_types(type_env);
        let mut children = Vec::with_capacity(fields.len());
        for (i, fty) in fields.iter().enumerate() {
            path.push(i);
            children.push(go(var, fty, own_out, type_env, path));
            path.pop();
        }
        OwnershipShape::UnboxedAgg(children)
    }
    go(var, ty, own_out, type_env, &mut vec![])
}

// --- tail-call recognition ---

/// The variables bound to an `App` or `Match` in tail position: a call in tail position must not be
/// turned into a non-tail one by an after-call release, so routing consults this set.
fn tail_apps(body: &RcExprNode) -> Set<FullName> {
    let mut out = Set::default();
    mark_tail(body, true, &mut out);
    out
}

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
        | RcExpr::Destructure(_, _, k)
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
    counter: &mut u64,
) -> (RcFunc, Map<FullName, FullName>) {
    let (params, cap, body, rename) =
        fresh_rename_function(&func.params, &func.cap, &func.body, "b", counter);
    (
        RcFunc {
            name: new_ref,
            fn_ty: func.fn_ty.clone(),
            params,
            cap,
            ret_ty: func.ret_ty.clone(),
            body,
            source: func.source.clone(),
            borrowed_units: Set::default(),
        },
        rename,
    )
}

// --- routing and reference-count rewrite ---

/// The per-version state the body rewrite reads: this version's aliasing facts and tail calls,
/// whether it is the borrow clone, and the whole-program ownership and version tables.
struct RewriteCtx<'a> {
    type_env: &'a TypeEnv,
    is_borrow: bool,
    own_out: &'a Set<RcUnit>,
    borrow_versions: &'a Map<FuncRef, FuncRef>,
    callee_params: &'a Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>>,
    tail: Set<FullName>,
    facts: FuncFacts,
}

impl<'a> RewriteCtx<'a> {
    fn new(
        func: &RcFunc,
        is_borrow: bool,
        own_out: &'a Set<RcUnit>,
        borrow_versions: &'a Map<FuncRef, FuncRef>,
        callee_params: &'a Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>>,
        type_env: &'a TypeEnv,
    ) -> RewriteCtx<'a> {
        RewriteCtx {
            type_env,
            is_borrow,
            own_out,
            borrow_versions,
            callee_params,
            tail: tail_apps(&func.body),
            facts: FuncFacts::of(func),
        }
    }

    fn rewrite(&self, node: &RcExprNode) -> RcExprNode {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.rewrite_inner(node))
    }

    fn rewrite_inner(&self, node: &RcExprNode) -> RcExprNode {
        match node.expr.as_ref() {
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let callee = self.route(x, callee, args, k);
                let (before, after) = self.call_rc(&callee, args);
                let k = prepend_rc(after, true, self.rewrite(k));
                let app = node_of(
                    RcExpr::Let(x.clone(), RcRhs::App(callee, args.clone()), k),
                    &node.source,
                );
                prepend_rc(before, false, app)
            }
            RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
                let arms = arms
                    .iter()
                    .map(|arm| MatchArm {
                        variant: arm.variant,
                        payload: arm.payload.clone(),
                        body: self.rewrite(&arm.body),
                    })
                    .collect();
                node_of(
                    RcExpr::Let(
                        x.clone(),
                        RcRhs::Match(scrut.clone(), arms),
                        self.rewrite(k),
                    ),
                    &node.source,
                )
            }
            RcExpr::Let(x, rhs, k) => node_of(
                RcExpr::Let(x.clone(), rhs.clone(), self.rewrite(k)),
                &node.source,
            ),
            RcExpr::Retain(v, path, state, k) => {
                self.rewrite_rc(v, path, *state, false, k, &node.source)
            }
            RcExpr::Release(v, path, state, k) => {
                self.rewrite_rc(v, path, *state, true, k, &node.source)
            }
            RcExpr::Destructure(container, fields, k) => node_of(
                RcExpr::Destructure(container.clone(), fields.clone(), self.rewrite(k)),
                &node.source,
            ),
            RcExpr::Eval(v, k) => node_of(RcExpr::Eval(v.clone(), self.rewrite(k)), &node.source),
            RcExpr::Ret(v) => node_of(RcExpr::Ret(v.clone()), &node.source),
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
            if self.safe(x, args) && self.beneficial(bref, args, k) {
                let mut c = callee.clone();
                c.name = bref.name.clone();
                return c;
            }
        }
        callee.clone()
    }

    /// A call is safe to route to the borrow version when it is not in tail position, or it passes no
    /// owned argument — so the after-call release the borrow version needs never lands on a tail call.
    fn safe(&self, x: &RcVar, args: &[RcVar]) -> bool {
        !self.tail.contains(&x.name) || !args.iter().any(|a| self.any_owned_unit(a))
    }

    /// Whether routing this call to the borrow version removes a reference count it would otherwise
    /// need, for at least one argument unit. Routing helps a unit that the borrow version borrows and
    /// that would otherwise be retained: a borrowed value (which an owning callee makes the caller
    /// retain before the call) or an owned value used again after the call (whose retain-before the
    /// borrow cancels). An owned value at its last use is moved either way, so borrowing it removes no
    /// retain and only delays its release; it is not a benefit.
    fn beneficial(&self, bref: &FuncRef, args: &[RcVar], k: &RcExprNode) -> bool {
        // `bref` is a borrow version, and `borrow_ify` registers every version's parameters, so it is a
        // key here.
        let bparams = &self.callee_params[bref];
        args.iter().enumerate().any(|(q, arg)| {
            let last_use = !used_later(&arg.name, k);
            rc_units(&arg.ty, self.type_env).iter().any(|unit| {
                // `q` is in range since `args.len() <= params.len()`.
                let callee_borrows = !self.own_out.contains(&(bparams[q].0.clone(), unit.clone()));
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

    /// Whether this version owns the value at `arg@unit`: a leaf that roots at an owned parameter, or
    /// at a producer (a fresh value, a call result, a boxed-container read), is owned; a leaf that
    /// roots at a borrowed parameter is not.
    fn owns_unit(&self, arg: &RcVar, unit: &Path) -> bool {
        let (r, rp) = root(&self.facts, self.type_env, &arg.name, unit);
        match self.facts.params.get(&r) {
            // The root path may name a subtree that spans several reference-counting units rather than
            // one — a union built from an unboxed tuple roots to the tuple at the empty path, whose
            // units are its fields. The value is owned only when every unit it covers is owned. Each
            // covered path is clamped to its unit key, so a path that descends into a union variant
            // keys to the union root the owned set records.
            Some(rty) => units_under(rty, &rp, self.type_env).iter().all(|u| {
                self.own_out
                    .contains(&(r.clone(), clamp_unit(rty, u, self.type_env)))
            }),
            None => true,
        }
    }

    /// The reference-count operations a call site takes over: for each argument unit, a release after
    /// the call when an owned value is passed to a borrowed position, and a retain before the call
    /// when a borrowed value is passed to an owning position.
    fn call_rc(&self, callee: &RcVar, args: &[RcVar]) -> (Vec<(RcVar, Path)>, Vec<(RcVar, Path)>) {
        let cparams = self.callee_params.get(&FuncRef {
            name: callee.name.clone(),
        });
        let mut before = vec![];
        let mut after = vec![];
        for (q, arg) in args.iter().enumerate() {
            for unit in rc_units(&arg.ty, self.type_env) {
                // An unresolved (indirect) callee owns every position (the all-`Own` ABI); a resolved
                // one is indexed by `q`, which is in range since `args.len() <= params.len()`.
                let callee_owns = match cparams {
                    None => true,
                    Some(ps) => self.own_out.contains(&(ps[q].0.clone(), unit.clone())),
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
        path: &Path,
        state: RcState,
        is_release: bool,
        k: &RcExprNode,
        source: &Option<Span>,
    ) -> RcExprNode {
        let k = self.rewrite(k);
        if !self.is_borrow {
            return rc_node(is_release, v.clone(), path.clone(), state, k, source);
        }
        let kept: Vec<Path> = units_under(&v.ty, path, self.type_env)
            .into_iter()
            .filter(|unit| self.owns_unit(v, unit))
            .collect();
        kept.into_iter().rev().fold(k, |cont, unit| {
            rc_node(is_release, v.clone(), unit, state, cont, source)
        })
    }
}

/// An expression node with the given source span.
fn node_of(expr: RcExpr, source: &Option<Span>) -> RcExprNode {
    RcExprNode {
        expr: Box::new(expr),
        source: source.clone(),
    }
}

/// A `Release` (when `is_release`) or `Retain` of `var` at `path` wrapping continuation `k`.
fn rc_node(
    is_release: bool,
    var: RcVar,
    path: Path,
    state: RcState,
    k: RcExprNode,
    source: &Option<Span>,
) -> RcExprNode {
    let expr = if is_release {
        RcExpr::Release(var, path, state, k)
    } else {
        RcExpr::Retain(var, path, state, k)
    };
    node_of(expr, source)
}

/// Wrap a continuation in a `Retain` (or `Release`) of each given unit.
fn prepend_rc(items: Vec<(RcVar, Path)>, is_release: bool, k: RcExprNode) -> RcExprNode {
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
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || match node.expr.as_ref() {
        RcExpr::Ret(v) => v.name == *name,
        RcExpr::Let(_, rhs, k) => rhs_uses(name, rhs) || used_later(name, k),
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => used_later(name, k),
        RcExpr::Destructure(container, _, k) => container.name == *name || used_later(name, k),
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
        RcRhs::Llvm(gen, args) => {
            args.iter().any(|a| a.name == *name) || gen.free_vars().iter().any(|v| v == name)
        }
        RcRhs::Match(scrut, arms) => {
            scrut.name == *name || arms.iter().any(|arm| used_later(name, &arm.body))
        }
    }
}

/// The reference-counting units under a path of a value's type: the units of the subtree the path
/// names, or the path itself when it already names a unit (a boxed value, a union, or a leaf).
fn units_under(ty: &Arc<TypeNode>, path: &Path, type_env: &TypeEnv) -> Vec<Path> {
    match subtree_type(ty, path, type_env) {
        Some(sty) => rc_units(&sty, type_env)
            .into_iter()
            .map(|u| {
                let mut p = path.clone();
                p.extend(u);
                p
            })
            .collect(),
        None => vec![path.clone()],
    }
}

/// The type of the subtree a path names, descending only unboxed structs; `None` once the path
/// reaches a closure, a unit root (`is_rc_unit_root`), or a fully-unboxed leaf.
fn subtree_type(ty: &Arc<TypeNode>, path: &Path, type_env: &TypeEnv) -> Option<Arc<TypeNode>> {
    let mut cur = ty.clone();
    for &idx in path {
        if cur.is_closure() || cur.is_rc_unit_root(type_env) || cur.is_fully_unboxed(type_env) {
            return None;
        }
        // Here `cur` is an unboxed struct/tuple, so a well-formed unit/root path index is in range.
        let fields = cur.field_types(type_env);
        assert!(
            idx < fields.len(),
            "subtree_type: path index {} out of range ({} fields)",
            idx,
            fields.len()
        );
        cur = fields[idx].clone();
    }
    Some(cur)
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

fn split_body(node: &RcExprNode, type_env: &TypeEnv) -> RcExprNode {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || split_body_inner(node, type_env))
}

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
                    variant: arm.variant,
                    payload: arm.payload.clone(),
                    body: split_body(&arm.body, type_env),
                })
                .collect();
            node_of(
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrut.clone(), arms),
                    split_body(k, type_env),
                ),
                &node.source,
            )
        }
        RcExpr::Let(x, rhs, k) => node_of(
            RcExpr::Let(x.clone(), rhs.clone(), split_body(k, type_env)),
            &node.source,
        ),
        RcExpr::Destructure(container, fields, k) => node_of(
            RcExpr::Destructure(container.clone(), fields.clone(), split_body(k, type_env)),
            &node.source,
        ),
        RcExpr::Eval(v, k) => node_of(
            RcExpr::Eval(v.clone(), split_body(k, type_env)),
            &node.source,
        ),
        RcExpr::Ret(v) => node_of(RcExpr::Ret(v.clone()), &node.source),
    }
}

/// Rebuild a `Retain`/`Release` as one node per unit under its path, preserving the state and span.
fn split_rc(
    v: &RcVar,
    path: &Path,
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

/// The pending retains at a program point: for each object (a reference-counting unit, keyed by its
/// `root`), the stack of retains that have bumped it and not yet been un-bumped. A release un-bumps
/// the most recent — the innermost bracket, which keeps the un-bump non-zeroing.
type Pend = Map<RcUnit, Vec<NodeId>>;

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
    let own_out = all_owned_units(prog, type_env);
    let cancel_body = |facts: &FuncFacts, body: &RcExprNode| {
        let mut analysis = CancelAnalysis {
            facts,
            prog,
            own_out: &own_out,
            type_env,
            needed: Set::default(),
            pairs: Map::default(),
            all_retains: vec![],
        };
        analysis.walk(body, Pend::default(), true);
        drop_nodes(body, &analysis.cancelled())
    };

    let funcs = prog
        .funcs
        .values()
        .map(|f| {
            let facts = FuncFacts::of(f);
            let mut clone = f.clone();
            clone.body = cancel_body(&facts, &f.body);
            (f.name.clone(), clone)
        })
        .collect();
    let globals = prog
        .globals
        .iter()
        .map(|g| {
            let facts = FuncFacts::body_only(&g.init);
            RcGlobalInit {
                symbol: g.symbol.clone(),
                ty: g.ty.clone(),
                init: cancel_body(&facts, &g.init),
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
    facts: &'a FuncFacts,
    prog: &'a RcProgram,
    own_out: &'a Set<RcUnit>,
    type_env: &'a TypeEnv,
    /// Retains that are load-bearing on some path, so they cannot be cancelled.
    needed: Set<NodeId>,
    /// The releases each retain is un-bumped by; they are deleted together with the retain.
    pairs: Map<NodeId, Vec<NodeId>>,
    /// Every retain the walk saw, so the cancellable retains are those never made `needed`.
    all_retains: Vec<NodeId>,
}

impl<'a> CancelAnalysis<'a> {
    /// The reference-counting unit a leaf belongs to, as an object identity: its `root`, clamped to
    /// the unit. A leaf below an unboxed union keys to the union root, so a whole-union retain and a
    /// payload consume land in the same bucket (without which a payload consume could not keep the
    /// union retain needed, and a later union release would wrongly cancel it).
    fn key(&self, var: &FullName, path: &[usize]) -> RcUnit {
        let (r, rp) = root(self.facts, self.type_env, var, path);
        match self.facts.types.get(&r) {
            Some(ty) => (r, clamp_unit(ty, &rp, self.type_env)),
            None => (r, rp),
        }
    }

    /// Walk a node forward, threading the pending-retain state. `leaf_mode` marks that a terminal
    /// `Ret` here returns from the function — consuming its value and closing no bracket; inside a
    /// match arm it is false, since the arm's `Ret` flows its value to the match binding. Returns the
    /// pending state at the node's exit, so a match arm's exit can be merged into its continuation.
    fn walk(&mut self, node: &RcExprNode, pend: Pend, leaf_mode: bool) -> Pend {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.walk_inner(node, pend, leaf_mode)
        })
    }

    fn walk_inner(&mut self, node: &RcExprNode, mut pend: Pend, leaf_mode: bool) -> Pend {
        match node.expr.as_ref() {
            RcExpr::Retain(v, path, _, k) => {
                let r = node_id(node);
                self.all_retains.push(r);
                self.pairs.entry(r).or_default();
                pend.entry(self.key(&v.name, path)).or_default().push(r);
                self.walk(k, pend, leaf_mode)
            }
            RcExpr::Release(v, path, _, k) => {
                let o = self.key(&v.name, path);
                if let Some(stack) = pend.get_mut(&o) {
                    // A stack kept in `pend` is never empty (emptied stacks are removed below), so a
                    // pending retain to pair with is always present.
                    let r = stack.pop().expect("a stack kept in `pend` is non-empty");
                    self.pairs.entry(r).or_default().push(node_id(node));
                    if stack.is_empty() {
                        pend.remove(&o);
                    }
                }
                self.walk(k, pend, leaf_mode)
            }
            RcExpr::Let(_, RcRhs::Match(_, arms), k) => {
                let arm_exits: Vec<Pend> = arms
                    .iter()
                    .map(|arm| self.walk(&arm.body, pend.clone(), false))
                    .collect();
                let merged = self.merge(&pend, &arm_exits);
                self.walk(k, merged, leaf_mode)
            }
            RcExpr::Let(x, rhs, k) => {
                self.consume_rhs(&mut pend, rhs, &x.ty);
                self.walk(k, pend, leaf_mode)
            }
            RcExpr::Destructure(container, fields, k) => {
                for pi in destructure_consumes(container, fields, self.type_env) {
                    self.consume(&mut pend, &container.name, &pi);
                }
                self.walk(k, pend, leaf_mode)
            }
            // `Eval` neither consumes, retains, nor releases; it is transparent to the pending-retain
            // state (any release inserted after it is a separate `Release` node).
            RcExpr::Eval(_, k) => self.walk(k, pend, leaf_mode),
            RcExpr::Ret(_) => {
                if leaf_mode {
                    // A retain still pending at the function's return closes no bracket on this path.
                    for stack in pend.values() {
                        for &r in stack {
                            self.needed.insert(r);
                        }
                    }
                }
                pend
            }
        }
    }

    /// Mark every retain the right-hand side consumes as needed.
    fn consume_rhs(&mut self, pend: &mut Pend, rhs: &RcRhs, result_ty: &Arc<TypeNode>) {
        let owns = |p: &RcVar, pi: &Path| {
            self.own_out
                .contains(&(p.name.clone(), clamp_unit(&p.ty, pi, self.type_env)))
        };
        let mut consumed = vec![];
        rhs_consumes(
            rhs,
            result_ty,
            self.facts,
            self.prog,
            self.type_env,
            &owns,
            &mut consumed,
        );
        for (var, path) in consumed {
            self.consume(pend, &var, &path);
        }
    }

    /// A consume of a leaf: every retain pending for its unit is load-bearing here.
    fn consume(&mut self, pend: &mut Pend, var: &FullName, path: &[usize]) {
        let o = self.key(var, path);
        if let Some(stack) = pend.remove(&o) {
            for r in stack {
                self.needed.insert(r);
            }
        }
    }

    /// Merge match arms into their continuation: a retain pending in every arm's exit continues (a
    /// single downstream release un-bumps it on all paths); a retain pending in some but not all arms
    /// has a non-uniform fate and cannot be cleanly cancelled, so it is disqualified.
    fn merge(&mut self, pend_in: &Pend, arm_exits: &[Pend]) -> Pend {
        let n = arm_exits.len();
        let mut arms_pending: Map<NodeId, usize> = Map::default();
        for exit in arm_exits {
            let mut seen: Set<NodeId> = Set::default();
            for stack in exit.values() {
                for &r in stack {
                    if seen.insert(r) {
                        *arms_pending.entry(r).or_default() += 1;
                    }
                }
            }
        }
        for (&r, &count) in &arms_pending {
            if count != n {
                self.needed.insert(r);
            }
        }
        // Keep the retains pending in all arms, in the pre-match order so release pairing stays
        // innermost-first.
        let mut merged = Pend::default();
        for (o, stack) in pend_in {
            let kept: Vec<NodeId> = stack
                .iter()
                .copied()
                .filter(|r| arms_pending.get(r) == Some(&n))
                .collect();
            if !kept.is_empty() {
                merged.insert(o.clone(), kept);
            }
        }
        merged
    }

    /// The nodes to delete: every cancellable retain (one never made needed and paired by at least
    /// one release) together with the releases it pairs with.
    fn cancelled(&self) -> Set<NodeId> {
        let mut out = Set::default();
        for &r in &self.all_retains {
            if self.needed.contains(&r) {
                continue;
            }
            match self.pairs.get(&r) {
                Some(releases) if !releases.is_empty() => {
                    out.insert(r);
                    out.extend(releases.iter().copied());
                }
                // A retain with no un-bump release is left in place to keep the counting balanced.
                _ => {}
            }
        }
        out
    }
}

/// Rebuild a body with the analysis's cancelled retain and release nodes spliced out.
fn drop_nodes(node: &RcExprNode, to_delete: &Set<NodeId>) -> RcExprNode {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || drop_nodes_inner(node, to_delete))
}

fn drop_nodes_inner(node: &RcExprNode, to_delete: &Set<NodeId>) -> RcExprNode {
    match node.expr.as_ref() {
        RcExpr::Retain(v, path, state, k) => {
            let k = drop_nodes(k, to_delete);
            if to_delete.contains(&node_id(node)) {
                k
            } else {
                node_of(
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
                node_of(
                    RcExpr::Release(v.clone(), path.clone(), *state, k),
                    &node.source,
                )
            }
        }
        RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
            let arms = arms
                .iter()
                .map(|arm| MatchArm {
                    variant: arm.variant,
                    payload: arm.payload.clone(),
                    body: drop_nodes(&arm.body, to_delete),
                })
                .collect();
            node_of(
                RcExpr::Let(
                    x.clone(),
                    RcRhs::Match(scrut.clone(), arms),
                    drop_nodes(k, to_delete),
                ),
                &node.source,
            )
        }
        RcExpr::Let(x, rhs, k) => node_of(
            RcExpr::Let(x.clone(), rhs.clone(), drop_nodes(k, to_delete)),
            &node.source,
        ),
        RcExpr::Destructure(container, fields, k) => node_of(
            RcExpr::Destructure(container.clone(), fields.clone(), drop_nodes(k, to_delete)),
            &node.source,
        ),
        RcExpr::Eval(v, k) => node_of(
            RcExpr::Eval(v.clone(), drop_nodes(k, to_delete)),
            &node.source,
        ),
        RcExpr::Ret(v) => node_of(RcExpr::Ret(v.clone()), &node.source),
    }
}
