//! Borrow inference over the RC IR (plan §2.1), step 1: deciding, for each source function, which
//! of its parameters can be *borrowed* rather than *owned*.
//!
//! Lowering makes every parameter `Own` (the callee is responsible for releasing it). A parameter
//! that a function only reads — never passing it to an owning position, capturing it, or returning
//! it — can instead be `Borrow`ed: the caller keeps ownership and releases it after the call, and
//! the callee performs no reference counting on it. Borrowing removes the caller's retain before a
//! non-last use, which is what keeps a value `Unique` for the uniqueness analysis (§3).
//!
//! A parameter is borrowable unless one of its boxed leaves reaches a *consume site* — an owning
//! argument position, a capture, or a return — traced back through aliases (move-binds and
//! unboxed-aggregate projections) to the parameter it originates from. Ownership is a fixed point:
//! whether an argument position consumes depends on the callee's ownership, which is itself being
//! decided. This module computes that fixed point; the version routing and reference-count rewrite
//! it enables are later steps.

use crate::ast::inline_llvm::LLVMGenerator;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{Map, Set};
use crate::rc_ir::ast::{
    FuncRef, Ownership, OwnershipShape, Path, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar,
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
/// closure value targets which function, and the function's own parameters.
struct FuncFacts {
    defs: Map<FullName, Def>,
    closure_targets: Map<FullName, FuncRef>,
    params: Set<FullName>,
}

/// The result of borrow inference: which parameter leaves are `Own` (all others are `Borrow`), keyed
/// by the parameter variable's name and the leaf path.
pub struct Ownerships {
    own: Set<Leaf>,
}

impl Ownerships {
    /// The ownership shape of one parameter (or capture): each boxed leaf is `Own` if inference put
    /// it in the owned set, else `Borrow`.
    pub fn shape_of(&self, param: &RcVar, type_env: &TypeEnv) -> OwnershipShape {
        self.shape(&param.name, &param.ty, type_env, &mut vec![])
    }

    fn shape(
        &self,
        var: &FullName,
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        path: &mut Path,
    ) -> OwnershipShape {
        if ty.is_fully_unboxed(type_env) {
            return OwnershipShape::Unboxed;
        }
        if ty.is_closure() {
            path.push(1);
            let cap = self.leaf_ownership(var, path);
            path.pop();
            return OwnershipShape::UnboxedAgg(vec![
                OwnershipShape::Unboxed,
                OwnershipShape::Boxed(cap),
            ]);
        }
        if ty.is_box(type_env) {
            return OwnershipShape::Boxed(self.leaf_ownership(var, path));
        }
        let fields = ty.field_types(type_env);
        let mut children = Vec::with_capacity(fields.len());
        for (i, fty) in fields.iter().enumerate() {
            path.push(i);
            children.push(self.shape(var, fty, type_env, path));
            path.pop();
        }
        OwnershipShape::UnboxedAgg(children)
    }

    fn leaf_ownership(&self, var: &FullName, path: &Path) -> Ownership {
        if self.own.contains(&(var.clone(), path.clone())) {
            Ownership::Own
        } else {
            Ownership::Borrow
        }
    }
}

/// Infer parameter ownership and return each parameter and capture's ownership shape, keyed by the
/// parameter variable's (globally unique) name — the borrow-inference annotations for the RC IR dump.
pub fn param_ownership_map(prog: &RcProgram, type_env: &TypeEnv) -> Map<FullName, OwnershipShape> {
    let ownerships = infer_ownership(prog, type_env);
    let mut out = Map::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.cap.iter()) {
            out.insert(p.name.clone(), ownerships.shape_of(p, type_env));
        }
    }
    out
}

/// Infer parameter ownership for every function of `prog` by a fixed point: start every parameter
/// leaf `Borrow`, then repeatedly demote to `Own` any leaf that a consume site traces back to, until
/// nothing changes. Demotion is monotone (`Borrow` to `Own` only), so it terminates.
pub fn infer_ownership(prog: &RcProgram, type_env: &TypeEnv) -> Ownerships {
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
                if facts.params.contains(&root_var) && own.insert((root_var, root_path)) {
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
        let mut params = Set::default();
        for p in &func.params {
            defs.insert(p.name.clone(), Def::Param);
            params.insert(p.name.clone());
        }
        if let Some(cap) = &func.cap {
            defs.insert(cap.name.clone(), Def::Param);
            params.insert(cap.name.clone());
        }
        collect_defs(&func.body, &mut defs, &mut closure_targets);
        FuncFacts {
            defs,
            closure_targets,
            params,
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
