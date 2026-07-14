//! Borrow-ification over the RC IR (plan §2.1): rewriting `Own` parameters that a function only
//! reads to `Borrow`, so the caller keeps ownership across the call and no retain is needed before a
//! non-last use — which is what keeps a value `Unique` for the uniqueness analysis (§3).
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

use crate::ast::inline_llvm::LLVMGenerator;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{
    FuncRef, MatchArm, Ownership, OwnershipShape, Path, RcExpr, RcExprNode, RcFunc, RcGlobalInit,
    RcProgram, RcRhs, RcState, RcVar,
};
use crate::rc_ir::provenance::{result_prov, BaseSource};
use std::sync::Arc;

/// A boxed leaf: a variable together with the path to one of its boxed leaves. Because RC IR names
/// are globally unique, it identifies the leaf across a whole program.
type Leaf = (FullName, Path);

/// What binds a variable, enough to trace a leaf back to the object that produced it (its `root`).
enum Def {
    /// A parameter or capture — the origin of a leaf.
    Param,
    /// `let x = y`: a move-bind, transparent to `root`.
    Move(RcVar),
    /// `let x = op(args)`: an alias when the result leaf is a pure projection of one argument,
    /// otherwise a producer. Carries the result type to consult `result_prov`.
    Llvm(LLVMGenerator, Vec<RcVar>, Arc<TypeNode>),
    /// `let x = f(args)` or a closure or a match — an opaque producer.
    Producer,
    /// A `destructure` field: field `idx` of the container.
    Field(RcVar, usize),
    /// A `match`-arm payload: the variant tag (`None` for a catch-all), and the scrutinee.
    Payload(RcVar, Option<usize>),
}

/// The per-function facts `root` and the consume walk need: how each local variable is bound, which
/// closure value targets which function, and the function's own parameters (with their types, so a
/// leaf that roots at a parameter can be clamped to its reference-counting unit).
struct FuncFacts {
    defs: Map<FullName, Def>,
    closure_targets: Map<FullName, FuncRef>,
    params: Map<FullName, Arc<TypeNode>>,
}

/// The result of borrow inference: which parameter leaves are `Own` (all others are `Borrow`), keyed
/// by the parameter variable's name and the leaf path.
struct Ownerships {
    own: Set<Leaf>,
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

    let mut own: Set<Leaf> = Set::default();
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
    fn of(func: &RcFunc) -> FuncFacts {
        let mut defs = Map::default();
        let mut closure_targets = Map::default();
        let mut params = Map::default();
        for p in &func.params {
            defs.insert(p.name.clone(), Def::Param);
            params.insert(p.name.clone(), p.ty.clone());
        }
        if let Some(cap) = &func.cap {
            defs.insert(cap.name.clone(), Def::Param);
            params.insert(cap.name.clone(), cap.ty.clone());
        }
        collect_defs(&func.body, &mut defs, &mut closure_targets);
        FuncFacts {
            defs,
            closure_targets,
            params,
        }
    }

    /// The facts of a param-less body (a global initializer).
    fn body_only(body: &RcExprNode) -> FuncFacts {
        let mut defs = Map::default();
        let mut closure_targets = Map::default();
        collect_defs(body, &mut defs, &mut closure_targets);
        FuncFacts {
            defs,
            closure_targets,
            params: Map::default(),
        }
    }
}

/// Record every local variable's `Def` (and any closure value's target function) in a function body.
fn collect_defs(
    node: &RcExprNode,
    defs: &mut Map<FullName, Def>,
    closure_targets: &mut Map<FullName, FuncRef>,
) {
    match node.expr.as_ref() {
        RcExpr::Ret(_) => {}
        RcExpr::Let(x, rhs, k) => {
            let def = match rhs {
                RcRhs::Var(y) => Def::Move(y.clone()),
                RcRhs::Llvm(gen, args) => Def::Llvm(gen.clone(), args.clone(), x.ty.clone()),
                RcRhs::Closure(fref, _) => {
                    closure_targets.insert(x.name.clone(), fref.clone());
                    Def::Producer
                }
                RcRhs::App(..) => Def::Producer,
                RcRhs::Match(scrut, arms) => {
                    for arm in arms {
                        defs.insert(
                            arm.payload.name.clone(),
                            Def::Payload(scrut.clone(), arm.variant),
                        );
                        collect_defs(&arm.body, defs, closure_targets);
                    }
                    Def::Producer
                }
            };
            defs.insert(x.name.clone(), def);
            collect_defs(k, defs, closure_targets);
        }
        RcExpr::Destructure(container, fields, k) => {
            for (idx, fv) in fields {
                defs.insert(fv.name.clone(), Def::Field(container.clone(), *idx));
            }
            collect_defs(k, defs, closure_targets);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => {
            collect_defs(k, defs, closure_targets)
        }
    }
}

/// The object a leaf originates from: follow alias edges (move-binds, pure projections, unboxed-union
/// payloads) back to the producing variable and path. The returned variable is a parameter when the
/// leaf ultimately comes from an input.
fn root(facts: &FuncFacts, type_env: &TypeEnv, var: &FullName, path: &[usize]) -> Leaf {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
        root_inner(facts, type_env, var, path)
    })
}

fn root_inner(facts: &FuncFacts, type_env: &TypeEnv, var: &FullName, path: &[usize]) -> Leaf {
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
                && matches!(gen, LLVMGenerator::MakeUnionBody(_))
                && !result_ty.is_box(type_env)
            {
                return root(facts, type_env, &args[0].name, &[]);
            }
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            let decl = result_prov(gen, result_ty, &arg_tys, type_env);
            // A result leaf that is a single `Arg(j, p)` is a pure projection of argument `j`'s leaf
            // `p` — an alias; anything else (a fresh allocation, a boxed-container read, a join of
            // several sources) is a producer, stopping here.
            match single_arg(&decl.leaf_at(path)) {
                Some((j, p)) if j < args.len() => root(facts, type_env, &args[j].name, &p),
                _ => here(),
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

/// Collect the leaves consumed in a function body: an owning argument position, a captured value, or
/// a returned value. Alias edges are not consumes here — the consume of an alias is attributed to its
/// `root`. Explicit `Release` nodes are own-then-release drops, not consumes.
fn collect_consumes(
    node: &RcExprNode,
    facts: &FuncFacts,
    prog: &RcProgram,
    own: &Set<Leaf>,
    type_env: &TypeEnv,
    out: &mut Vec<Leaf>,
) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => push_leaves(&x.name, &x.ty, type_env, out),
        RcExpr::Let(_, rhs, k) => {
            match rhs {
                RcRhs::Var(_) => {}
                RcRhs::Closure(_, caps) => {
                    for c in caps {
                        push_leaves(&c.name, &c.ty, type_env, out);
                    }
                }
                RcRhs::App(callee, args) => {
                    // Calling a closure consumes it (the callee releases its capture).
                    push_leaves(&callee.name, &callee.ty, type_env, out);
                    // Each argument at an owning position of the callee is consumed. An unresolved
                    // (indirect) callee owns every position.
                    let callee_params = resolve_callee_params(callee, facts, prog);
                    for (i, a) in args.iter().enumerate() {
                        for pi in boxed_leaves(&a.ty, type_env) {
                            let owns_pos = match &callee_params {
                                Some(params) => params
                                    .get(i)
                                    .map_or(true, |p| own.contains(&(p.name.clone(), pi.clone()))),
                                None => true,
                            };
                            if owns_pos {
                                out.push((a.name.clone(), pi));
                            }
                        }
                    }
                }
                RcRhs::Llvm(gen, args) => {
                    let passthrough = passthrough_arg_leaves(gen, node, args, type_env);
                    for (i, a) in args.iter().enumerate() {
                        if gen.borrows_operand(i) {
                            continue;
                        }
                        for pi in boxed_leaves(&a.ty, type_env) {
                            // An argument leaf that the op passes through to its result is not
                            // consumed; anything else at an owning position is moved into the op.
                            if !passthrough.contains(&(i, pi.clone())) {
                                out.push((a.name.clone(), pi));
                            }
                        }
                    }
                }
                RcRhs::Match(_, arms) => {
                    for arm in arms {
                        collect_consumes(&arm.body, facts, prog, own, type_env, out);
                    }
                }
            }
            collect_consumes(k, facts, prog, own, type_env, out);
        }
        RcExpr::Destructure(_, _, k) => collect_consumes(k, facts, prog, own, type_env, out),
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => {
            collect_consumes(k, facts, prog, own, type_env, out)
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
    gen: &LLVMGenerator,
    node: &RcExprNode,
    args: &[RcVar],
    type_env: &TypeEnv,
) -> Set<(usize, Path)> {
    let result_ty = match node.expr.as_ref() {
        RcExpr::Let(x, _, _) => x.ty.clone(),
        _ => return Set::default(),
    };
    let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
    let decl = result_prov(gen, &result_ty, &arg_tys, type_env);
    let mut out = Set::default();
    collect_arg_leaves(&decl, &mut out);
    out
}

/// Collect every `Arg(i, path)` symbol appearing in a declared provenance.
fn collect_arg_leaves(prov: &crate::rc_ir::provenance::Provenance, out: &mut Set<(usize, Path)>) {
    use crate::rc_ir::provenance::Provenance;
    match prov {
        Provenance::Unboxed => {}
        Provenance::UnboxedAgg(children) => {
            for c in children {
                collect_arg_leaves(c, out);
            }
        }
        Provenance::Boxed(ls) => {
            for s in ls {
                if let BaseSource::Arg(i, p) = s {
                    out.insert((*i, p.clone()));
                }
            }
        }
    }
}

/// Push every boxed leaf of a value onto `out`.
fn push_leaves(var: &FullName, ty: &Arc<TypeNode>, type_env: &TypeEnv, out: &mut Vec<Leaf>) {
    for p in boxed_leaves(ty, type_env) {
        out.push((var.clone(), p));
    }
}

/// The paths of every boxed leaf of a type: the whole value if boxed, the capture of a closure, or
/// each boxed leaf of an unboxed aggregate.
fn boxed_leaves(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Path> {
    let mut out = vec![];
    boxed_leaves_go(ty, type_env, &mut vec![], &mut out);
    out
}

fn boxed_leaves_go(ty: &Arc<TypeNode>, type_env: &TypeEnv, path: &mut Path, out: &mut Vec<Path>) {
    if ty.is_fully_unboxed(type_env) {
        return;
    }
    if ty.is_closure() {
        path.push(1);
        out.push(path.clone());
        path.pop();
        return;
    }
    if ty.is_box(type_env) {
        out.push(path.clone());
        return;
    }
    for (i, fty) in ty.field_types(type_env).iter().enumerate() {
        path.push(i);
        boxed_leaves_go(fty, type_env, path, out);
        path.pop();
    }
}

// --- reference-counting units ---

/// The reference-counting units of a value's type: the boxed value, the capture of a closure, or
/// each unit of an unboxed struct/tuple. Unlike `boxed_leaves`, it stops at the root of an unboxed
/// union — a union is one unit, since only its active variant is live and a refcount operation on it
/// must dispatch on the tag rather than name a variant's leaf unconditionally.
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
        path.push(1);
        out.push(path.clone());
        path.pop();
        return;
    }
    if ty.is_box(type_env) || ty.is_union(type_env) {
        out.push(path.clone());
        return;
    }
    for (i, fty) in ty.field_types(type_env).iter().enumerate() {
        path.push(i);
        rc_units_go(fty, type_env, path, out);
        path.pop();
    }
}

/// Truncate a leaf path to its reference-counting unit: the path up to the first unboxed union it
/// enters (a union subtree is one unit). Paths that stay within unboxed structs are unchanged.
fn clamp_unit(ty: &Arc<TypeNode>, path: &[usize], type_env: &TypeEnv) -> Path {
    let mut out = vec![];
    let mut cur = ty.clone();
    for &idx in path {
        if cur.is_closure() {
            // The only path into a closure names its capture, which is a single unit.
            out.push(idx);
            break;
        }
        if cur.is_box(type_env) || cur.is_union(type_env) {
            // A boxed value (including a boxed union) and an unboxed union are each one unit.
            break;
        }
        let fields = cur.field_types(type_env);
        if idx >= fields.len() {
            break;
        }
        out.push(idx);
        cur = fields[idx].clone();
    }
    out
}

// --- borrow-ification ---

/// The result of borrow-ification: the rewritten program, and — for the RC IR dump — each version's
/// parameter ownership shape, keyed by the (globally unique) parameter variable name.
pub struct BorrowIfied {
    pub program: RcProgram,
    pub param_owns: Map<FullName, OwnershipShape>,
}

/// Borrow-ify a program: materialize a borrowing version of every function with a borrowable
/// parameter, route each direct call to a version, and rewrite the reference counting accordingly.
pub fn borrow_ify(prog: &RcProgram, type_env: &TypeEnv) -> BorrowIfied {
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
    let mut own_out: Set<Leaf> = Set::default();
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

    // The dump annotation: each output version's parameter shapes, from `own_out`.
    let mut param_owns: Map<FullName, OwnershipShape> = Map::default();
    for func in funcs.values() {
        for p in func.params.iter().chain(func.cap.iter()) {
            param_owns.insert(
                p.name.clone(),
                shape_from_own(&p.name, &p.ty, &own_out, type_env),
            );
        }
    }

    BorrowIfied {
        program: RcProgram {
            funcs,
            globals,
            entry: prog.entry.clone(),
        },
        param_owns,
    }
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
    own_out: &Set<Leaf>,
    type_env: &TypeEnv,
) -> OwnershipShape {
    fn go(
        var: &FullName,
        ty: &Arc<TypeNode>,
        own_out: &Set<Leaf>,
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
            path.push(1);
            let cap = owned(path);
            path.pop();
            return OwnershipShape::UnboxedAgg(vec![
                OwnershipShape::Unboxed,
                OwnershipShape::Boxed(cap),
            ]);
        }
        if ty.is_box(type_env) || ty.is_union(type_env) {
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
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Destructure(_, _, k) => {
            mark_tail(k, in_tail, out)
        }
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
    let mut rename: Map<FullName, FullName> = Map::default();
    for p in func.params.iter().chain(func.cap.iter()) {
        fresh_rename(&p.name, &mut rename, counter);
    }
    collect_binders(&func.body, &mut rename, counter);

    let params = func.params.iter().map(|p| rename_var(p, &rename)).collect();
    let cap = func.cap.as_ref().map(|c| rename_var(c, &rename));
    let body = rename_expr(&func.body, &rename);
    (
        RcFunc {
            name: new_ref,
            fn_ty: func.fn_ty.clone(),
            params,
            cap,
            ret_ty: func.ret_ty.clone(),
            body,
            source: func.source.clone(),
        },
        rename,
    )
}

/// Assign `name` a fresh globally-unique name (unless it already has one).
fn fresh_rename(name: &FullName, rename: &mut Map<FullName, FullName>, counter: &mut u64) {
    if rename.contains_key(name) {
        return;
    }
    *counter += 1;
    let mut fresh = name.clone();
    fresh.name = format!("{}#b{}", fresh.name, counter);
    rename.insert(name.clone(), fresh);
}

/// Record a fresh name for every variable bound in a function body.
fn collect_binders(node: &RcExprNode, rename: &mut Map<FullName, FullName>, counter: &mut u64) {
    match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => {
            fresh_rename(&x.name, rename, counter);
            if let RcRhs::Match(_, arms) = rhs {
                for arm in arms {
                    fresh_rename(&arm.payload.name, rename, counter);
                    collect_binders(&arm.body, rename, counter);
                }
            }
            collect_binders(k, rename, counter);
        }
        RcExpr::Destructure(_, fields, k) => {
            for (_, fv) in fields {
                fresh_rename(&fv.name, rename, counter);
            }
            collect_binders(k, rename, counter);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) => {
            collect_binders(k, rename, counter)
        }
        RcExpr::Ret(_) => {}
    }
}

/// A variable with its name rewritten through `rename` (unchanged if it names a global rather than a
/// local binder).
fn rename_var(var: &RcVar, rename: &Map<FullName, FullName>) -> RcVar {
    let mut v = var.clone();
    if let Some(n) = rename.get(&var.name) {
        v.name = n.clone();
    }
    v
}

/// A deep clone of an expression with every variable occurrence rewritten through `rename`. The
/// operand names embedded in an `Llvm` generator are rewritten too, since they name the same locals.
fn rename_expr(node: &RcExprNode, rename: &Map<FullName, FullName>) -> RcExprNode {
    let expr = match node.expr.as_ref() {
        RcExpr::Let(x, rhs, k) => RcExpr::Let(
            rename_var(x, rename),
            rename_rhs(rhs, rename),
            rename_expr(k, rename),
        ),
        RcExpr::Retain(v, path, state, k) => RcExpr::Retain(
            rename_var(v, rename),
            path.clone(),
            *state,
            rename_expr(k, rename),
        ),
        RcExpr::Release(v, path, state, k) => RcExpr::Release(
            rename_var(v, rename),
            path.clone(),
            *state,
            rename_expr(k, rename),
        ),
        RcExpr::Destructure(container, fields, k) => RcExpr::Destructure(
            rename_var(container, rename),
            fields
                .iter()
                .map(|(i, v)| (*i, rename_var(v, rename)))
                .collect(),
            rename_expr(k, rename),
        ),
        RcExpr::Ret(v) => RcExpr::Ret(rename_var(v, rename)),
    };
    RcExprNode {
        expr: Box::new(expr),
        source: node.source.clone(),
    }
}

fn rename_rhs(rhs: &RcRhs, rename: &Map<FullName, FullName>) -> RcRhs {
    match rhs {
        RcRhs::Var(v) => RcRhs::Var(rename_var(v, rename)),
        RcRhs::App(callee, args) => RcRhs::App(
            rename_var(callee, rename),
            args.iter().map(|a| rename_var(a, rename)).collect(),
        ),
        RcRhs::Closure(fref, caps) => RcRhs::Closure(
            fref.clone(),
            caps.iter().map(|c| rename_var(c, rename)).collect(),
        ),
        RcRhs::Llvm(gen, args) => {
            let mut gen = gen.clone();
            for slot in gen.free_vars_mut() {
                if let Some(n) = rename.get(slot) {
                    *slot = n.clone();
                }
            }
            RcRhs::Llvm(gen, args.iter().map(|a| rename_var(a, rename)).collect())
        }
        RcRhs::Match(scrut, arms) => RcRhs::Match(
            rename_var(scrut, rename),
            arms.iter()
                .map(|arm| MatchArm {
                    variant: arm.variant,
                    payload: rename_var(&arm.payload, rename),
                    body: rename_expr(&arm.body, rename),
                })
                .collect(),
        ),
    }
}

// --- routing and reference-count rewrite ---

/// The per-version state the body rewrite reads: this version's aliasing facts and tail calls,
/// whether it is the borrow clone, and the whole-program ownership and version tables.
struct RewriteCtx<'a> {
    type_env: &'a TypeEnv,
    is_borrow: bool,
    own_out: &'a Set<Leaf>,
    borrow_versions: &'a Map<FuncRef, FuncRef>,
    callee_params: &'a Map<FuncRef, Vec<(FullName, Arc<TypeNode>)>>,
    tail: Set<FullName>,
    facts: FuncFacts,
}

impl<'a> RewriteCtx<'a> {
    fn new(
        func: &RcFunc,
        is_borrow: bool,
        own_out: &'a Set<Leaf>,
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
                let callee = self.route(x, callee, args);
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
            RcExpr::Ret(v) => node_of(RcExpr::Ret(v.clone()), &node.source),
        }
    }

    /// Route a direct call: if the callee has a borrow version and routing to it is safe, retarget
    /// the callee to that version; otherwise keep the original (the all-`Own` version, or an
    /// indirect callee this leaves untouched).
    fn route(&self, x: &RcVar, callee: &RcVar, args: &[RcVar]) -> RcVar {
        let orig = FuncRef {
            name: callee.name.clone(),
        };
        if let Some(bref) = self.borrow_versions.get(&orig) {
            if self.safe(x, args) {
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
            Some(rty) => {
                let ru = clamp_unit(rty, &rp, self.type_env);
                self.own_out.contains(&(r, ru))
            }
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
                // An unresolved (indirect) callee owns every position (the all-`Own` ABI).
                let callee_owns = match cparams.and_then(|ps| ps.get(q)) {
                    Some((pn, _)) => self.own_out.contains(&(pn.clone(), unit.clone())),
                    None => true,
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
            let expr = if is_release {
                RcExpr::Release(v.clone(), path.clone(), state, k)
            } else {
                RcExpr::Retain(v.clone(), path.clone(), state, k)
            };
            return node_of(expr, source);
        }
        let kept: Vec<Path> = units_under(&v.ty, path, self.type_env)
            .into_iter()
            .filter(|unit| self.owns_unit(v, unit))
            .collect();
        kept.into_iter().rev().fold(k, |cont, unit| {
            let expr = if is_release {
                RcExpr::Release(v.clone(), unit, state, cont)
            } else {
                RcExpr::Retain(v.clone(), unit, state, cont)
            };
            node_of(expr, source)
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

/// Wrap a continuation in a `Retain` (or `Release`) of each given unit.
fn prepend_rc(items: Vec<(RcVar, Path)>, is_release: bool, k: RcExprNode) -> RcExprNode {
    items.into_iter().rev().fold(k, |cont, (var, path)| {
        let expr = if is_release {
            RcExpr::Release(var, path, RcState::Unknown, cont)
        } else {
            RcExpr::Retain(var, path, RcState::Unknown, cont)
        };
        node_of(expr, &None)
    })
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
/// reaches a unit boundary (a boxed value, a union, a closure, or a fully-unboxed leaf).
fn subtree_type(ty: &Arc<TypeNode>, path: &Path, type_env: &TypeEnv) -> Option<Arc<TypeNode>> {
    let mut cur = ty.clone();
    for &idx in path {
        if cur.is_closure()
            || cur.is_box(type_env)
            || cur.is_union(type_env)
            || cur.is_fully_unboxed(type_env)
        {
            return None;
        }
        let fields = cur.field_types(type_env);
        if idx >= fields.len() {
            return None;
        }
        cur = fields[idx].clone();
    }
    Some(cur)
}
