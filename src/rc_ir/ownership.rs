//! The ownership and consume model of the RC IR: which construct consumes which reference, which
//! binding merely aliases one, and how a value's references are grouped into reference-counting
//! units. Every pass that reasons about reference counting reads its answers here, so they all work
//! from one model; `dev-docs/2026-06-28-unique-check-elim/rc-ownership-model.md` is its written
//! specification.
//!
//! Three vocabularies meet here. A **boxed leaf** is one reference a value holds
//! (`boxed_leaf_paths`); consumption and provenance are stated over leaves. A **reference-counting
//! unit** is one refcount operation's target (`rc_units`): a boxed value, a closure capture, an
//! unboxed union (whose operation dispatches on the tag rather than naming a variant), or a punched
//! array. `truncate_to_unit` and `units_under` bridge the two. An **origin** is the object a leaf's
//! reference belongs to (`origin`), reached by following the alias edges — move-binds, unboxed-
//! aggregate projections, unboxed-union payloads, and pure `Llvm` projections — back to the binding
//! that produced it.

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::fixstd::builtin::InlineLLVMMakeUnionBody;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{
    FieldPath, FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar, VarPath,
};
use crate::rc_ir::leaf_map::boxed_leaf_paths;
use crate::rc_ir::provenance::LeafOrigin;
use std::sync::Arc;

/// What binds a variable, enough to trace a leaf back to the object that produced it (its `origin`).
enum Binding {
    /// A parameter or capture — the origin of a leaf.
    Param,
    /// `let x = y`: a move-bind, transparent to `origin`.
    Move(RcVar),
    /// `let x = op(args)`: an alias when the result leaf is a pure projection of one argument,
    /// otherwise a producer. Carries the result type to consult `result_prov`.
    Llvm(Box<dyn LLVMGen>, Vec<RcVar>, Arc<TypeNode>),
    /// `let x = f(args)` or a closure — an opaque producer.
    Producer,
    /// A `destructure` field: field `idx` of the container.
    Field(RcVar, usize),
    /// A `match`-arm payload: the variant tag (`None` for a catch-all), and the scrutinee.
    Payload(RcVar, Option<usize>),
    /// `let x = match ...`: the value each arm returns. A match binding produces nothing of its own —
    /// it receives one of these, chosen by the path taken — so its object identity is path-dependent.
    Join(Vec<RcVar>),
}

/// The variables of one function, as `origin` and the consume walk read them.
pub(crate) struct VarTable {
    /// What binds each variable, which `origin` follows back to the object a leaf belongs to.
    bindings: Map<FullName, Binding>,
    /// The function each closure value targets, so a call of one resolves to that function's
    /// parameters and their ownership.
    closure_targets: Map<FullName, FuncRef>,
    /// The type of each parameter and capture. A variable recorded here roots at an input of the
    /// function, which is where inferred ownership is stated.
    pub(crate) param_tys: Map<FullName, Arc<TypeNode>>,
    /// The type of every variable, parameters included, so a leaf that roots at any of them can be
    /// truncated to its reference-counting unit.
    pub(crate) var_tys: Map<FullName, Arc<TypeNode>>,
}
impl VarTable {
    /// The variable table of a function: its parameters and capture as `Param` bindings, plus the `Binding` and
    /// type of every variable bound in its body.
    pub(crate) fn of(func: &RcFunc) -> VarTable {
        let mut vars = VarTable::empty();
        for p in func.params.iter().chain(func.capture.iter()) {
            vars.bindings.insert(p.name.clone(), Binding::Param);
            vars.param_tys.insert(p.name.clone(), p.ty.clone());
            vars.var_tys.insert(p.name.clone(), p.ty.clone());
        }
        collect_bindings(&func.body, &mut vars);
        vars
    }

    /// The vars of a param-less body (a global initializer).
    pub(crate) fn body_only(body: &RcExprNode) -> VarTable {
        let mut vars = VarTable::empty();
        collect_bindings(body, &mut vars);
        vars
    }

    /// A table with no variable in it, to be filled by the constructor that knows what to put there.
    fn empty() -> VarTable {
        VarTable {
            bindings: Map::default(),
            closure_targets: Map::default(),
            param_tys: Map::default(),
            var_tys: Map::default(),
        }
    }
}

/// Record every local variable's `Binding` and type (and any closure value's target function) in a
/// function body.
fn collect_bindings(node: &RcExprNode, vars: &mut VarTable) {
    match node.expr.as_ref() {
        RcExpr::Ret(_) => {}
        RcExpr::Let(x, rhs, k) => {
            let def = match rhs {
                RcRhs::Var(y) => Binding::Move(y.clone()),
                RcRhs::Llvm(llvm_gen, args) => {
                    Binding::Llvm(llvm_gen.clone(), args.clone(), x.ty.clone())
                }
                RcRhs::Closure(fref, _) => {
                    vars.closure_targets.insert(x.name.clone(), fref.clone());
                    Binding::Producer
                }
                RcRhs::App(..) => Binding::Producer,
                RcRhs::Match(scrut, arms) => {
                    let mut arm_results = vec![];
                    for arm in arms {
                        vars.bindings.insert(
                            arm.payload.name.clone(),
                            Binding::Payload(scrut.clone(), arm.tag),
                        );
                        vars.var_tys
                            .insert(arm.payload.name.clone(), arm.payload.ty.clone());
                        collect_bindings(&arm.body, vars);
                        arm_results.push(returned_var(&arm.body).clone());
                    }
                    Binding::Join(arm_results)
                }
            };
            vars.bindings.insert(x.name.clone(), def);
            vars.var_tys.insert(x.name.clone(), x.ty.clone());
            collect_bindings(k, vars);
        }
        RcExpr::Destructure(container, fields, _state, k) => {
            for (idx, fv) in fields {
                vars.bindings
                    .insert(fv.name.clone(), Binding::Field(container.clone(), *idx));
                vars.var_tys.insert(fv.name.clone(), fv.ty.clone());
            }
            collect_bindings(k, vars);
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            collect_bindings(k, vars)
        }
    }
}

/// The variable an expression returns: the one that its final `Ret` names. Every construct of the RC
/// IR has a single continuation, so a body has exactly one such `Ret` (a match's arms each have their
/// own, which return the arm's value to the match binding).
fn returned_var(node: &RcExprNode) -> &RcVar {
    grow_stack(|| match node.expr.as_ref() {
        RcExpr::Ret(v) => v,
        RcExpr::Let(_, _, k)
        | RcExpr::Destructure(_, _, _, k)
        | RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k)
        | RcExpr::Eval(_, k) => returned_var(k),
    })
}

/// Where the object at a leaf comes from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    /// The leaf denotes exactly this object.
    Exactly(VarPath),
    /// The leaf denotes one of `candidates`, chosen by the path taken. `identity` names the match
    /// binding that joins them: every alias chain through that binding agrees on the name, so it is
    /// the name to use where one name for the value is required.
    Join {
        identity: VarPath,
        candidates: Set<VarPath>,
    },
}

impl Origin {
    /// The one name for the value, for a reader that pairs operations on it — reference-count
    /// cancellation pairs a retain with the release that un-bumps it, and only a single identity can
    /// decide that. Two leaves with the same identity hold the same reference.
    pub(crate) fn identity(&self) -> &VarPath {
        match self {
            Origin::Exactly(p) => p,
            Origin::Join { identity, .. } => identity,
        }
    }

    /// Every object the leaf may denote, for a reader whose answer has to hold on all paths.
    pub(crate) fn candidates(&self) -> Vec<&VarPath> {
        match self {
            Origin::Exactly(p) => vec![p],
            Origin::Join { candidates, .. } => candidates.iter().collect(),
        }
    }

    /// Every object an operation on the leaf acts on: the reference the leaf holds, which `identity`
    /// names, and the object that reference belongs to, which is any of `candidates`.
    pub(crate) fn acted_on(&self) -> Vec<&VarPath> {
        let mut out = vec![self.identity()];
        out.extend(
            self.candidates()
                .into_iter()
                .filter(|p| *p != self.identity()),
        );
        out
    }
}

/// Where a leaf's object comes from: follow alias edges (move-binds, pure projections, unboxed-union
/// payloads, catch-all payloads) back to the variable that produced it. The variable is a parameter
/// when the leaf ultimately comes from an input, and a `Join` when a match forwards several arms'
/// values to one binding.
pub(crate) fn origin(
    vars: &VarTable,
    type_env: &TypeEnv,
    var: &FullName,
    path: &[usize],
) -> Origin {
    grow_stack(|| origin_inner(vars, type_env, var, path))
}

/// One step of `origin`: the alias edge the variable's binding offers, followed to the object at the
/// far end, or the variable itself where the chain stops.
fn origin_inner(vars: &VarTable, type_env: &TypeEnv, var: &FullName, path: &[usize]) -> Origin {
    let here = || Origin::Exactly((var.clone(), path.to_vec()));
    match vars.bindings.get(var) {
        // A name the table does not bind is a global — a function a direct call names, or a global
        // value read as an atom — which this function's variables alias nothing of. So it is its own
        // origin, as a parameter and a producer are.
        None | Some(Binding::Param) | Some(Binding::Producer) => here(),
        Some(Binding::Move(y)) => origin(vars, type_env, &y.name, path),
        Some(Binding::Join(arm_results)) => {
            // The arms bind their values to one variable, so a path into it is a path into each of
            // them. Arms that all reach the same object leave the value exact.
            let mut candidates = Set::default();
            for r in arm_results {
                for p in origin(vars, type_env, &r.name, path).candidates() {
                    candidates.insert(p.clone());
                }
            }
            match candidates.len() {
                1 => Origin::Exactly(
                    candidates
                        .into_iter()
                        .next()
                        .expect("a one-element set has an element"),
                ),
                _ => Origin::Join {
                    identity: (var.clone(), path.to_vec()),
                    candidates,
                },
            }
        }
        Some(Binding::Llvm(llvm_gen, args, result_ty)) => {
            // Constructing an unboxed union lays its payload in place, so the whole union's root is
            // the payload's root — the construction alias edge, dual to reading a payload out with
            // `match`. The whole-union path is where this matters: a leaf path descends into the
            // active variant, which the projection rule below already aliases through `result_prov`.
            if path.is_empty()
                && !args.is_empty()
                && llvm_gen.as_any().is::<InlineLLVMMakeUnionBody>()
                && !result_ty.is_box(type_env)
            {
                return origin(vars, type_env, &args[0].name, &[]);
            }
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            let decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env);
            // A result leaf that is a single `Arg(j, p)` is a pure projection of argument `j`'s leaf
            // `p` — an alias; anything else (a fresh allocation, a boxed-container read, a join of
            // several sources) is a producer, stopping here. An `Llvm` op is never partially applied,
            // so a well-formed `result_prov` names only real argument indices (`args[j]` else panics).
            // A path with no declared leaf is not a projection either: a reference-counting unit
            // path may name the root of an unboxed union, which is a subtree rather than a leaf.
            match decl.leaf_origins_at(path).and_then(as_arg_projection) {
                Some((j, p)) => origin(vars, type_env, &args[j].name, &p),
                None => here(),
            }
        }
        Some(Binding::Field(container, idx)) => {
            if container.ty.is_box(type_env) {
                // Reading a field of a boxed struct retains it: a producer.
                here()
            } else {
                let mut p = vec![*idx];
                p.extend_from_slice(path);
                origin(vars, type_env, &container.name, &p)
            }
        }
        Some(Binding::Payload(scrut, variant)) => match variant {
            // A catch-all binds the whole scrutinee: the same object.
            None => origin(vars, type_env, &scrut.name, path),
            // An unboxed union's payload is the scrutinee's variant slot — an alias; a boxed union's
            // payload is read out (retained) — a producer.
            Some(tag) if !scrut.ty.is_box(type_env) => {
                let mut p = vec![*tag];
                p.extend_from_slice(path);
                origin(vars, type_env, &scrut.name, &p)
            }
            Some(_) => here(),
        },
    }
}

/// The single `Arg(j, p)` a leaf source consists of, if it is exactly that.
fn as_arg_projection(sources: &Set<LeafOrigin>) -> Option<(usize, FieldPath)> {
    if sources.len() != 1 {
        return None;
    }
    match sources.iter().next() {
        Some(LeafOrigin::Arg(j, p)) => Some((*j, p.clone())),
        _ => None,
    }
}

/// Collect the leaves consumed in a function body, given `own` as the owned parameter leaves that
/// decide which argument positions consume: an owning argument position, a captured value, or a
/// returned value. Alias edges are not consumes here — the consume of an alias is attributed to its
/// `origin`. Explicit `Release` nodes are own-then-release drops, not consumes.
pub(crate) fn collect_consumes(
    node: &RcExprNode,
    vars: &VarTable,
    prog: &RcProgram,
    own: &Set<VarPath>,
    type_env: &TypeEnv,
    out: &mut Vec<VarPath>,
) {
    let owns = |p: &RcVar, leaf: &FieldPath| own.contains(&(p.name.clone(), leaf.clone()));
    collect_consumes_go(node, vars, prog, type_env, &owns, out);
}

/// Collect the leaves an expression and its continuation consume. `owns` answers whether a callee's
/// parameter leaf is owned, which decides whether the argument at that position is consumed.
fn collect_consumes_go<F: Fn(&RcVar, &FieldPath) -> bool>(
    node: &RcExprNode,
    vars: &VarTable,
    prog: &RcProgram,
    type_env: &TypeEnv,
    owns: &F,
    out: &mut Vec<VarPath>,
) {
    match node.expr.as_ref() {
        RcExpr::Ret(x) => push_boxed_leaves(&x.name, &x.ty, type_env, out),
        RcExpr::Let(x, rhs, k) => {
            match rhs {
                RcRhs::Match(_, arms) => {
                    for arm in arms {
                        collect_consumes_go(&arm.body, vars, prog, type_env, owns, out);
                    }
                }
                _ => rhs_consumes(rhs, &x.ty, vars, prog, type_env, owns, out),
            }
            collect_consumes_go(k, vars, prog, type_env, owns, out);
        }
        RcExpr::Destructure(container, fields, _state, k) => {
            for leaf in destructure_consumes(container, fields, type_env) {
                out.push((container.name.clone(), leaf));
            }
            collect_consumes_go(k, vars, prog, type_env, owns, out)
        }
        RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k) => {
            collect_consumes_go(k, vars, prog, type_env, owns, out)
        }
    }
}

/// The container leaves a `Destructure` consumes. A boxed container is released whole, so every boxed
/// leaf of it goes; an unboxed container moves each named field's leaves into that field's variable,
/// an alias whose consume is attributed to the field variable, so only a dropped (unnamed) field's
/// leaves go. This is the model code generation implements (`ObjectFieldType::get_struct_fields`), and
/// every reader of the consume model shares it.
pub(crate) fn destructure_consumes(
    container: &RcVar,
    fields: &[(usize, RcVar)],
    type_env: &TypeEnv,
) -> Vec<FieldPath> {
    let leaves = boxed_leaves(&container.ty, type_env);
    if container.ty.is_box(type_env) {
        return leaves;
    }
    let named: Set<usize> = fields.iter().map(|(i, _)| *i).collect();
    leaves
        .into_iter()
        .filter(|leaf| {
            // A boxed leaf of an unboxed container starts with a field index, so its path is non-empty.
            let field = leaf
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
pub(crate) fn rhs_consumes<F: Fn(&RcVar, &FieldPath) -> bool>(
    rhs: &RcRhs,
    result_ty: &Arc<TypeNode>,
    vars: &VarTable,
    prog: &RcProgram,
    type_env: &TypeEnv,
    owns: &F,
    out: &mut Vec<VarPath>,
) {
    match rhs {
        RcRhs::Var(_) | RcRhs::Match(..) => {}
        RcRhs::Closure(_, caps) => {
            for c in caps {
                push_boxed_leaves(&c.name, &c.ty, type_env, out);
            }
        }
        RcRhs::App(callee, args) => {
            // Calling a closure consumes it (the callee releases its capture).
            push_boxed_leaves(&callee.name, &callee.ty, type_env, out);
            // Each argument at an owning position of the callee is consumed. An unresolved (indirect)
            // callee owns every position.
            let callee_params = resolve_callee_params(callee, vars, prog);
            for (i, a) in args.iter().enumerate() {
                for leaf in boxed_leaves(&a.ty, type_env) {
                    // `i` ranges over the arguments and `args.len() <= params.len()` (no over-
                    // application), so `params[i]` is in range.
                    let owns_pos = match &callee_params {
                        Some(params) => owns(&params[i], &leaf),
                        None => true,
                    };
                    if owns_pos {
                        out.push((a.name.clone(), leaf));
                    }
                }
            }
        }
        RcRhs::Llvm(llvm_gen, args) => {
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            let passthrough = passthrough_arg_leaves(&**llvm_gen, result_ty, args, type_env);
            for (i, a) in args.iter().enumerate() {
                if llvm_gen.borrows_operand(i, &arg_tys, type_env) {
                    continue;
                }
                for leaf in boxed_leaves(&a.ty, type_env) {
                    // An argument leaf that the op passes through to its result is not consumed;
                    // anything else at an owning position is moved into the op.
                    if !passthrough.contains(&(i, leaf.clone())) {
                        out.push((a.name.clone(), leaf));
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
    vars: &VarTable,
    prog: &'a RcProgram,
) -> Option<&'a [RcVar]> {
    let fref = vars
        .closure_targets
        .get(&callee.name)
        .cloned()
        .or_else(|| {
            let fref = FuncRef {
                name: callee.name.clone(),
            };
            prog.funcs.contains_key(&fref).then_some(fref)
        })?;
    // A closure target is registered as a function when it is lifted, and the other branch resolved
    // the name against the program already, so the callee is there either way. Reporting it absent
    // here would read as an indirect call and quietly give up the borrow optimization.
    let func = prog.funcs.get(&fref).unwrap_or_else(|| {
        unreachable!(
            "callee `{}` is not a function of the program",
            fref.name.to_string()
        )
    });
    Some(func.params.as_slice())
}

/// The `(arg index, leaf path)` pairs an LLVM op passes through unchanged to its result — the pure
/// projections `as_arg_projection` reads out of `result_prov`.
///
/// Dropping an argument leaf's consume is sound exactly when the result aliases it, so this shares
/// `as_arg_projection` with `origin`: a leaf that joins an argument with another source aliases nothing and
/// keeps its consume, and one whose sole source is `Arg` does both.
fn passthrough_arg_leaves(
    llvm_gen: &dyn LLVMGen,
    result_ty: &Arc<TypeNode>,
    args: &[RcVar],
    type_env: &TypeEnv,
) -> Set<(usize, FieldPath)> {
    let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
    let decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env);
    decl.leaves().filter_map(as_arg_projection).collect()
}

/// Push every boxed leaf of a value onto `out`.
fn push_boxed_leaves(
    var: &FullName,
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    out: &mut Vec<VarPath>,
) {
    for p in boxed_leaves(ty, type_env) {
        out.push((var.clone(), p));
    }
}

/// The paths of every boxed leaf of a type: the whole value if boxed, the capture of a closure, or
/// each boxed leaf of an unboxed aggregate.
pub(crate) fn boxed_leaves(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<FieldPath> {
    boxed_leaf_paths(ty, type_env)
}

// --- reference-counting units ---

/// The reference-counting units of a value's type: the capture of a closure, or each unit root
/// (`is_rc_unit_root`) — a boxed value, an unboxed union, or a punched array — reached by descending
/// its unboxed structs/tuples. Unlike `boxed_leaves`, it stops at a unit root rather than expanding it
/// into the inner boxed leaves (e.g. an unboxed union is one unit, since only its active variant is
/// live and a refcount operation must dispatch on the tag rather than name a variant's leaf).
pub(crate) fn rc_units(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<FieldPath> {
    let mut out = vec![];
    rc_units_go(ty, type_env, &mut vec![], &mut out);
    out
}

/// Descend a type, pushing onto `out` the path of each unit root reached. `path` is the field path
/// from the value's root down to `ty`, which each pushed unit is named relative to.
fn rc_units_go(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    path: &mut FieldPath,
    out: &mut Vec<FieldPath>,
) {
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
pub(crate) fn truncate_to_unit(
    ty: &Arc<TypeNode>,
    path: &[usize],
    type_env: &TypeEnv,
) -> FieldPath {
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
            "truncate_to_unit: path index {} out of range ({} fields)",
            idx,
            fields.len()
        );
        out.push(idx);
        cur = fields[idx].clone();
    }
    out
}

/// The reference-counting unit a leaf belongs to, as an object identity: its `origin`'s identity,
/// clamped to the unit. A leaf below an unboxed union keys to the union root, so a whole-union
/// retain and a payload consume land in the same bucket (without which a payload consume could not
/// keep the union retain needed, and a later union release would wrongly cancel it).
///
/// This is the key a retain and a release are paired on, and the key a reference count is kept
/// under, so it must name one object: a leaf whose object is path-dependent keys to the match
/// binding that joins the paths, which every alias chain through it agrees on. The units an
/// operation on it really touches are `acted_unit_keys`.
pub(crate) fn unit_key(
    vars: &VarTable,
    type_env: &TypeEnv,
    var: &FullName,
    path: &[usize],
) -> VarPath {
    unit_of(vars, type_env, origin(vars, type_env, var, path).identity())
}

/// Every reference-counting unit an operation on a leaf acts on: the one its reference is counted
/// under, and the ones the object it belongs to may be counted under.
pub(crate) fn acted_unit_keys(
    vars: &VarTable,
    type_env: &TypeEnv,
    var: &FullName,
    path: &[usize],
) -> Vec<VarPath> {
    origin(vars, type_env, var, path)
        .acted_on()
        .into_iter()
        .map(|p| unit_of(vars, type_env, p))
        .collect()
}

/// The unit key of an object identity: the root it names, with its path truncated to the
/// reference-counting unit that holds it.
fn unit_of(vars: &VarTable, type_env: &TypeEnv, (root, path): &VarPath) -> VarPath {
    let Some(ty) = vars.var_tys.get(root) else {
        // A root with no type here is a global: the table holds the function's own variables.
        // Reference counting is inserted for locals only and a global's reachable graph is
        // refcount-exempt, so no retain or release keys to it and there is nothing to line up.
        assert!(
            !root.is_local(),
            "local `{}` has no recorded type",
            root.to_string()
        );
        return (root.clone(), path.clone());
    };
    (root.clone(), truncate_to_unit(ty, path, type_env))
}

/// The owned parameter/capture units of every function: each version's units minus the ones it
/// borrows (`RcFunc::borrowed_units`, the annotation borrow-ification writes).
pub(crate) fn all_owned_units(prog: &RcProgram, type_env: &TypeEnv) -> Set<VarPath> {
    let mut owned = Set::default();
    for func in prog.funcs.values() {
        for p in func.params.iter().chain(func.capture.iter()) {
            for unit in rc_units(&p.ty, type_env) {
                let unit_path = (p.name.clone(), unit);
                if !func.borrowed_units.contains(&unit_path) {
                    owned.insert(unit_path);
                }
            }
        }
    }
    owned
}

/// The reference-counting units under a path of a value's type: the units of the subtree the path
/// names, or the path itself when it already names a unit (a boxed value, a union, or a leaf).
pub(crate) fn units_under(
    ty: &Arc<TypeNode>,
    path: &FieldPath,
    type_env: &TypeEnv,
) -> Vec<FieldPath> {
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
fn subtree_type(ty: &Arc<TypeNode>, path: &FieldPath, type_env: &TypeEnv) -> Option<Arc<TypeNode>> {
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

#[cfg(test)]
mod tests {
    use super::{as_arg_projection, origin, Binding, Origin, VarTable};
    use crate::ast::name::FullName;
    use crate::ast::program::TypeEnv;
    use crate::fixstd::builtin::make_i64_ty;
    use crate::misc::Set;
    use crate::rc_ir::ast::{RcVar, VarPath};
    use crate::rc_ir::provenance::{LeafOrigin, Provenance};

    /// The sources of one result leaf, as `result_prov` declares them.
    fn sources(srcs: Vec<LeafOrigin>) -> Set<LeafOrigin> {
        srcs.into_iter().collect()
    }

    /// A result leaf whose only source is one argument leaf aliases that leaf, and is reported with
    /// the argument's index and path.
    #[test]
    fn a_lone_arg_is_a_projection() {
        let ls = Provenance::leaf(LeafOrigin::Arg(1, vec![0]));
        assert_eq!(as_arg_projection(&ls), Some((1, vec![0])));
    }

    /// A result leaf that is the argument on one path and a new value on another aliases neither:
    /// the op consumes the argument, and `origin` stops at the op. Reading such a leaf as a
    /// projection would drop the consume without the alias, releasing one object twice.
    #[test]
    fn an_arg_joined_with_another_source_is_not_a_projection() {
        let ls = sources(vec![LeafOrigin::Fresh, LeafOrigin::Arg(0, vec![])]);
        assert_eq!(as_arg_projection(&ls), None);
    }

    /// A result leaf that may come from either of two arguments aliases neither: a projection names
    /// one argument, and here the choice would fall to whichever of the two the set yields first.
    #[test]
    fn one_of_two_args_is_not_a_projection() {
        let ls = sources(vec![LeafOrigin::Arg(0, vec![]), LeafOrigin::Arg(1, vec![])]);
        assert_eq!(as_arg_projection(&ls), None);
    }

    /// A leaf the op itself produced — a fresh object, or one of unknown origin — aliases no
    /// argument.
    #[test]
    fn a_produced_leaf_is_not_a_projection() {
        assert_eq!(
            as_arg_projection(&Provenance::leaf(LeafOrigin::Fresh)),
            None
        );
        assert_eq!(
            as_arg_projection(&Provenance::leaf(LeafOrigin::Unknown)),
            None
        );
    }

    /// A leaf with no source at all — the result of `_undefined_internal`, which aborts — aliases no
    /// argument.
    #[test]
    fn a_bottom_leaf_is_not_a_projection() {
        assert_eq!(as_arg_projection(&sources(vec![])), None);
    }

    /// A local variable of type `I64`.
    fn var(name: &str) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty: make_i64_ty(),
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    /// A table of the given bindings, with every named variable also a known local.
    fn table(bindings: Vec<(&str, Binding)>) -> VarTable {
        let mut vars = VarTable::empty();
        for (name, b) in bindings {
            let v = var(name);
            vars.bindings.insert(v.name.clone(), b);
            vars.var_tys.insert(v.name, v.ty);
        }
        vars
    }

    /// The whole value of a local variable: its name at the empty path.
    fn at(name: &str) -> VarPath {
        (FullName::local(name), vec![])
    }

    /// The origin of a variable's whole value.
    fn origin_of(vars: &VarTable, name: &str) -> Origin {
        origin(vars, &TypeEnv::default(), &FullName::local(name), &[])
    }

    /// A variable bound by the op that produced the value is the origin of that value.
    #[test]
    fn a_producer_is_exactly_itself() {
        let vars = table(vec![("p", Binding::Producer)]);
        assert_eq!(origin_of(&vars, "p"), Origin::Exactly(at("p")));
    }

    /// A move-bind reaches through to the variable it moved, so both names key to one object.
    #[test]
    fn a_move_bind_is_the_moved_variable() {
        let vars = table(vec![
            ("p", Binding::Producer),
            ("m", Binding::Move(var("p"))),
        ]);
        assert_eq!(origin_of(&vars, "m"), Origin::Exactly(at("p")));
    }

    /// A match binding whose arms produce different objects is one of them: its candidates are the
    /// arms' results, and the join itself is the name every alias chain through it agrees on.
    #[test]
    fn a_match_binding_may_be_any_arm_result() {
        let vars = table(vec![
            ("p", Binding::Producer),
            ("q", Binding::Producer),
            ("m", Binding::Join(vec![var("p"), var("q")])),
        ]);
        let o = origin_of(&vars, "m");
        assert_eq!(o.identity(), &at("m"));
        assert_eq!(
            o.candidates()
                .into_iter()
                .cloned()
                .collect::<Set<VarPath>>(),
            vec![at("p"), at("q")].into_iter().collect::<Set<VarPath>>()
        );
    }

    /// A match binding whose arms all reach one variable, here with one arm reaching it through a
    /// move-bind, is exactly that variable.
    #[test]
    fn a_match_binding_whose_arms_agree_is_exact() {
        let vars = table(vec![
            ("p", Binding::Producer),
            ("m1", Binding::Move(var("p"))),
            ("m", Binding::Join(vec![var("p"), var("m1")])),
        ]);
        assert_eq!(origin_of(&vars, "m"), Origin::Exactly(at("p")));
    }

    /// A move of a match binding keeps the join's identity: the identity has to survive an alias
    /// chain, or a retain of the binding and a release of the moved-to variable would key
    /// differently and never pair.
    #[test]
    fn a_move_of_a_match_binding_keeps_the_joins_name() {
        let vars = table(vec![
            ("p", Binding::Producer),
            ("q", Binding::Producer),
            ("m", Binding::Join(vec![var("p"), var("q")])),
            ("n", Binding::Move(var("m"))),
        ]);
        assert_eq!(origin_of(&vars, "n").identity(), &at("m"));
    }

    /// A join over another join flattens: every result the inner join could yield is among the outer
    /// join's candidates.
    #[test]
    fn a_join_of_joins_may_be_any_of_their_results() {
        let vars = table(vec![
            ("p", Binding::Producer),
            ("q", Binding::Producer),
            ("r", Binding::Producer),
            ("inner", Binding::Join(vec![var("p"), var("q")])),
            ("m", Binding::Join(vec![var("inner"), var("r")])),
        ]);
        assert_eq!(
            origin_of(&vars, "m")
                .candidates()
                .into_iter()
                .cloned()
                .collect::<Set<VarPath>>(),
            vec![at("p"), at("q"), at("r")]
                .into_iter()
                .collect::<Set<VarPath>>()
        );
    }
}
