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
//!
//! What one reference-count operation bumps is a count of references per object
//! (`acted_references`). A retain of an unboxed union and a release of one field of the payload it
//! holds key to the same unit, and the counts are what tell the two apart: the release un-bumps
//! part of what the retain bumped.

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::{CLOSURE_CAPTURE_IDX, CLOSURE_FIELD_COUNT};
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{
    FieldPath, FuncRef, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar, VarPath,
};
use crate::rc_ir::leaf_map::boxed_leaf_paths;
use crate::rc_ir::provenance::{LeafOrigin, Provenance};
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

/// The variables of one function, enough to trace a leaf back to the object it belongs to and to
/// resolve a call to its callee's parameters.
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

    /// An empty table, which a constructor fills.
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
            let binding = match rhs {
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
            vars.bindings.insert(x.name.clone(), binding);
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
    /// The leaf denotes one of several objects, chosen by the path taken.
    Join {
        /// The match binding that joins the candidates. Every alias chain through that binding
        /// agrees on this name, so it is the name to use where one name for the value is required.
        identity: VarPath,
        /// Every object the leaf may denote, one per path the match can take.
        candidates: Set<VarPath>,
    },
}

impl Origin {
    /// The one name for the value, for a reader that pairs two operations on it — a retain with the
    /// release that un-bumps it — which only a single name can decide. Two leaves with the same
    /// identity hold the same reference.
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

    /// The origin a set of candidate objects amounts to: exactly the object they agree on, and a
    /// join under `identity` where they name several. `identity` is the one name every alias chain
    /// through the value agrees on.
    fn of_candidates(candidates: Set<VarPath>, identity: &VarPath) -> Origin {
        assert!(
            !candidates.is_empty(),
            "the origin of `{}` reaches no object",
            identity.0.to_string()
        );
        match candidates.len() {
            1 => Origin::Exactly(
                candidates
                    .into_iter()
                    .next()
                    .expect("a one-element set has an element"),
            ),
            _ => Origin::Join {
                identity: identity.clone(),
                candidates,
            },
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
        // value read as an atom. No variable of this function aliases one, so it is its own origin,
        // as a parameter and a producer are.
        None | Some(Binding::Param) | Some(Binding::Producer) => here(),
        Some(Binding::Move(y)) => origin(vars, type_env, &y.name, path),
        Some(Binding::Join(arm_results)) => {
            // The arms bind their values to one variable, so a path into it is a path into each of
            // them. Arms that all reach the same object leave the value exact.
            let mut candidates = Set::default();
            for arm_result in arm_results {
                for p in origin(vars, type_env, &arm_result.name, path).candidates() {
                    candidates.insert(p.clone());
                }
            }
            Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))
        }
        Some(Binding::Llvm(llvm_gen, args, result_ty)) => {
            let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
            let decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env);
            // A result leaf that is a single `Arg(j, p)` is a pure projection of argument `j`'s leaf
            // `p` — an alias; anything else (a fresh allocation, a boxed-container read, a join of
            // several sources) is a producer, stopping here. An `Llvm` op is never partially applied,
            // so a well-formed `result_prov` names only real argument indices (`args[j]` else panics).
            // A path whose own record does not name an object is handled by
            // `origin_from_leaves_under`: a reference-counting unit path may name an unboxed union
            // itself, whose provenance is declared on the leaves of its variants.
            match decl.leaf_origins_at(path).and_then(as_arg_projection) {
                Some((j, p)) => origin(vars, type_env, &args[j].name, &p),
                None => {
                    // The leaves beneath name no object only for a value that holds no reference:
                    // either the path covers no boxed leaf, or every leaf beneath it is `⊥`, which
                    // an operation declares for the variants a union does not have and for the
                    // result of one that aborts. Taking the value for its own object is safe there,
                    // since a release is keyed to a reference and none is keyed to this answer.
                    let here_identity = (var.clone(), path.to_vec());
                    origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)
                        .unwrap_or_else(here)
                }
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

/// The object a value denotes at a path whose own record does not name one: the objects the leaves
/// beneath that path reach.
///
/// A reference-counting unit path may name an unboxed union itself, whose provenance is declared one
/// level down, on the leaves of its variants, so a reader that stops at the union finds nothing
/// recorded and would read the value as one produced here — its own to release, when it may be an
/// operand's. The leaves beneath decide instead. A leaf recorded as `⊥` belongs to a
/// variant the value does not have and holds no reference, so it names no object and is passed
/// over; a leaf produced here rather than projected names this value itself. A leaf that records
/// several sources holds one per path it can be reached by, and each of them names an object.
///
/// The answer is exact where the leaves agree on one unit. Where they reach several, or where one
/// of them is a value produced here rather than a projection, every object the value may denote is
/// reported together, so that a reader whose answer has to hold on all paths — whether this
/// version owns the value — sees them all. `None` where no leaf names an object.
///
/// # Arguments
/// * `here` - the value's own identity, which is what it denotes on any path the leaves leave open.
///
/// # Examples
/// Asked for `unbox union { wait : Guard, pair : (Guard, Guard), mark : I64 }` itself, read out of one
/// borrowed node, the `wait` leaf is `⊥` and both `pair` leaves project out of that node, so the
/// answer is `Exactly` the node's object. Give one of those two leaves a second operand and the
/// answer becomes a `Join` naming both.
fn origin_from_leaves_under(
    vars: &VarTable,
    type_env: &TypeEnv,
    decl: &Provenance,
    args: &[RcVar],
    path: &[usize],
    here: &VarPath,
) -> Option<Origin> {
    // The leaves of one unit usually project out of one operand unit, so the operand units are
    // gathered before any of them is followed back: resolving each distinct one once keeps a chain
    // of aliases from being walked once per leaf.
    let mut operand_units: Set<(usize, FieldPath)> = Set::default();
    let mut produced_here = false;
    for sources in decl.leaf_origins_under(path) {
        // A leaf holds one source per path it can be reached by, so every one of them is an object
        // the value may denote. A leaf recorded as `⊥` holds none and contributes nothing.
        for src in sources {
            match src {
                LeafOrigin::Arg(j, leaf) => {
                    operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)));
                }
                LeafOrigin::Fresh | LeafOrigin::Unknown => produced_here = true,
            }
        }
    }
    let mut candidates: Set<VarPath> = Set::default();
    if produced_here {
        candidates.insert(here.clone());
    }
    for (j, unit) in operand_units {
        for p in origin(vars, type_env, &args[j].name, &unit).candidates() {
            candidates.insert(p.clone());
        }
    }
    if candidates.is_empty() {
        return None;
    }
    Some(Origin::of_candidates(candidates, here))
}

/// The single `Arg(j, p)` a leaf source consists of, if it is exactly that.
fn as_arg_projection(sources: &Set<LeafOrigin>) -> Option<(usize, FieldPath)> {
    if sources.len() != 1 {
        return None;
    }
    match sources
        .iter()
        .next()
        .expect("a one-element set has an element")
    {
        LeafOrigin::Arg(j, p) => Some((*j, p.clone())),
        LeafOrigin::Fresh | LeafOrigin::Unknown => None,
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
                // A match holds the only sub-expressions a right-hand side can carry, so every
                // other shape consumes within itself.
                RcRhs::Var(..) | RcRhs::App(..) | RcRhs::Closure(..) | RcRhs::Llvm(..) => {
                    rhs_consumes(rhs, &x.ty, vars, prog, type_env, owns, out)
                }
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
/// leaves go. This is the model code generation implements (`ObjectFieldType::get_struct_fields`).
pub(crate) fn destructure_consumes(
    container: &RcVar,
    fields: &[(usize, RcVar)],
    type_env: &TypeEnv,
) -> Vec<FieldPath> {
    let leaves = boxed_leaf_paths(&container.ty, type_env);
    if container.ty.is_box(type_env) {
        return leaves;
    }
    let named_fields: Set<usize> = fields.iter().map(|(i, _)| *i).collect();
    leaves
        .into_iter()
        .filter(|leaf| {
            // A boxed leaf of an unboxed container starts with a field index, so its path is non-empty.
            let field = leaf
                .first()
                .expect("a boxed leaf of an unboxed container has a non-empty path");
            !named_fields.contains(field)
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
                for leaf in boxed_leaf_paths(&a.ty, type_env) {
                    // `i` ranges over the arguments and `args.len() <= params.len()` (no over-
                    // application), so `params[i]` is in range.
                    let is_owning_position = match &callee_params {
                        Some(params) => owns(&params[i], &leaf),
                        None => true,
                    };
                    if is_owning_position {
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
                for leaf in boxed_leaf_paths(&a.ty, type_env) {
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
    for p in boxed_leaf_paths(ty, type_env) {
        out.push((var.clone(), p));
    }
}

// --- reference-counting units ---

/// What a walk over reference-counting units does at one type: whether a unit sits here, whether the
/// walk descends, and where it goes next.
///
/// Which types carry a unit is one rule, and every walk over units takes its step from here, so a
/// new kind of unit is stated once and every walk follows it. A walk that a new kind left behind
/// would produce a path that names no unit; reference counting would key an operation to that path
/// and free the object while it is still held.
pub(crate) enum UnitStep {
    /// The value holds no reference, so no unit sits here or below it.
    NoUnit,
    /// A closure, whose capture is the one unit it holds.
    Capture {
        /// The field the capture sits at, the index a path into the closure names.
        capture_idx: usize,
        /// How many fields the closure has, so that a table a walk builds over them is indexed by
        /// field index. The other field is the function pointer, which holds no reference.
        field_count: usize,
    },
    /// One unit sits here and the walk stops: a boxed value, an unboxed union (only its active
    /// variant is live, so a refcount operation dispatches on the tag rather than naming a variant's
    /// leaf), an array (its own traverser drives its elements' lifetime through the storage), or a
    /// punched array (whose traversal skips the moved-out hole at a run-time index).
    Unit,
    /// An unboxed struct or tuple, whose units are those of the fields it holds.
    Fields {
        /// How many fields the type declares, so that a table a walk builds over them is indexed by
        /// field index.
        field_count: usize,
        /// The fields a value of the type holds, each with its index. A punched field holds nothing,
        /// so it is left out.
        held_fields: Vec<(usize, Arc<TypeNode>)>,
    },
}

/// The step a unit walk takes at `ty`.
///
/// The order the cases come in carries part of the answer: a closure is not a declared type, so
/// `is_box` and `is_union` abort when they are asked about one, and the closure case has to come
/// before them.
///
/// # Examples
/// `unit_step` of `Array I64` is `Unit`, of `I64` is `NoUnit`, and of `(Array I64, I64)` is `Fields`
/// whose two fields are both held.
pub(crate) fn unit_step(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> UnitStep {
    if ty.is_fully_unboxed(type_env) {
        return UnitStep::NoUnit;
    }
    if ty.is_closure() {
        return UnitStep::Capture {
            capture_idx: CLOSURE_CAPTURE_IDX as usize,
            field_count: CLOSURE_FIELD_COUNT,
        };
    }
    if ty.is_box(type_env) || ty.is_union(type_env) || ty.is_array() || ty.is_punched_array() {
        return UnitStep::Unit;
    }
    UnitStep::Fields {
        field_count: ty.toplevel_tycon_info(type_env).fields.len(),
        held_fields: ty.unpunched_field_types(type_env),
    }
}

/// The type of the field `idx` names among the fields a value holds.
///
/// Unit and leaf enumerations both leave a punched field out, so every path that reference counting
/// works with names a held field. An index naming a punched field, or one out of range, aborts the
/// walk `walk_name` names.
pub(crate) fn held_field_type(
    held_fields: &[(usize, Arc<TypeNode>)],
    idx: usize,
    walk_name: &str,
) -> Arc<TypeNode> {
    held_fields
        .iter()
        .find(|(i, _)| *i == idx)
        .unwrap_or_else(|| {
            panic!(
                "{}: path index {} names no field the value holds (it holds {:?})",
                walk_name,
                idx,
                held_fields.iter().map(|(i, _)| *i).collect::<Vec<_>>()
            )
        })
        .1
        .clone()
}

/// The reference-counting units of a value's type: each unit `unit_step` reaches by descending the
/// unboxed structs and tuples of the type. Unlike `boxed_leaf_paths`, it stops at a unit rather than
/// expanding it into the boxed leaves inside (an unboxed union is one unit, since only its active
/// variant is live and a refcount operation must dispatch on the tag rather than name a variant's
/// leaf).
pub(crate) fn rc_units(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<FieldPath> {
    let mut out = vec![];
    rc_units_go(ty, type_env, &mut vec![], &mut out);
    out
}

/// Descend a type, pushing onto `out` the path of each unit reached. `path` is the field path from
/// the whole value down to `ty`, which each pushed unit is named relative to.
fn rc_units_go(
    ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    path: &mut FieldPath,
    out: &mut Vec<FieldPath>,
) {
    match unit_step(ty, type_env) {
        UnitStep::NoUnit => {}
        UnitStep::Capture { capture_idx, .. } => {
            path.push(capture_idx);
            out.push(path.clone());
            path.pop();
        }
        UnitStep::Unit => out.push(path.clone()),
        UnitStep::Fields { held_fields, .. } => {
            for (i, fty) in held_fields {
                path.push(i);
                rc_units_go(&fty, type_env, path, out);
                path.pop();
            }
        }
    }
}

/// Truncate a leaf path to its reference-counting unit: the path down to the first unit `unit_step`
/// reaches, whose whole subtree is that one unit. A path that stays within unboxed structs is
/// unchanged.
pub(crate) fn truncate_to_unit(
    ty: &Arc<TypeNode>,
    path: &[usize],
    type_env: &TypeEnv,
) -> FieldPath {
    let mut out = vec![];
    let mut cur = ty.clone();
    for &idx in path {
        match unit_step(&cur, type_env) {
            // A value holding no reference has no unit below it for the rest of the path to name.
            UnitStep::NoUnit => panic!(
                "truncate_to_unit: the path {:?} enters `{}`, which holds no reference",
                path,
                cur.to_string()
            ),
            UnitStep::Capture { capture_idx, .. } => {
                // The only path into a closure names its capture, which is a single unit.
                assert_eq!(
                    idx, capture_idx,
                    "truncate_to_unit: a path into a closure names its capture"
                );
                out.push(idx);
                break;
            }
            // A leaf below a unit — a boxed leaf under a union variant, or the punched array's inner
            // array — keys to that unit.
            UnitStep::Unit => break,
            UnitStep::Fields { held_fields, .. } => {
                out.push(idx);
                cur = held_field_type(&held_fields, idx, "truncate_to_unit");
            }
        }
    }
    out
}

/// The reference-counting unit a leaf belongs to, as an object identity: its `origin`'s identity,
/// truncated to the unit. A leaf below an unboxed union keys to the union itself, so a whole-union
/// retain and a consume of the payload get the same key: the consume marks that retain as needed,
/// and cancellation keeps it together with the union release that un-bumps it.
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

/// The references a reference-count operation acts on: how many references of each object it bumps
/// or un-bumps. A value holding one object's reference twice contributes two of it.
///
/// Two operations that key to one `unit_key` need not act on the same references, so the key alone
/// does not say whether a release un-bumps a retain. An unboxed union is one unit, counted on the
/// union itself, and a retain of it bumps every reference its payload holds; a projection of that
/// payload names those references one by one, so a release of the projection un-bumps only part of
/// what the retain bumped.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct References(Map<VarPath, usize>);

impl References {
    /// Whether every reference of `other` is among these, counting multiplicity.
    ///
    /// # Examples
    /// References holding one reference of `a` and two of `b` cover one holding one of each, and do
    /// not cover one holding three of `b`. Every `References` covers itself and covers an empty one.
    pub(crate) fn covers(&self, other: &References) -> bool {
        other
            .0
            .iter()
            .all(|(object, count)| self.0.get(object).is_some_and(|held| held >= count))
    }

    /// Drop `other`'s references from these, where `covers` holds of the two.
    pub(crate) fn subtract(&mut self, other: &References) {
        for (object, count) in &other.0 {
            let held = self
                .0
                .get_mut(object)
                .expect("the removed references are covered by these");
            *held -= count;
            if *held == 0 {
                self.0.remove(object);
            }
        }
    }

    /// Whether the operation acts on no reference at all.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The references an operation on the subtree of `v` that `path` names acts on: the reference each
/// boxed leaf under `path` holds, named by the object its `origin` identifies.
pub(crate) fn acted_references(
    vars: &VarTable,
    type_env: &TypeEnv,
    v: &RcVar,
    path: &FieldPath,
) -> References {
    let mut references: Map<VarPath, usize> = Map::default();
    for leaf in boxed_leaf_paths(&v.ty, type_env) {
        if !leaf.starts_with(path) {
            continue;
        }
        let object = origin(vars, type_env, &v.name, &leaf).identity().clone();
        *references.entry(object).or_default() += 1;
    }
    References(references)
}

/// The unit key of an object identity: the root it names, with its path truncated to the
/// reference-counting unit that holds it. The returned path is always one of that root's units.
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
    let truncated = truncate_to_unit(ty, path, type_env);
    // Truncation only descends, so an identity whose path stops above every unit of its type
    // comes out naming no unit at all. A retain under such a key pairs with no release of the
    // object, so cancellation drops it and the object is freed while it is still held. The check
    // sits here, where every key is made.
    let units = rc_units(ty, type_env);
    assert!(
        units.contains(&truncated),
        "the key `{}{:?}` names no reference-counting unit of `{}`, whose units are {:?}",
        root.to_string(),
        truncated,
        ty.to_string(),
        units,
    );
    (root.clone(), truncated)
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
/// reaches a value the walk stops at — a closure, a unit, or a value holding no reference.
fn subtree_type(ty: &Arc<TypeNode>, path: &FieldPath, type_env: &TypeEnv) -> Option<Arc<TypeNode>> {
    let mut cur = ty.clone();
    for &idx in path {
        match unit_step(&cur, type_env) {
            UnitStep::Fields { held_fields, .. } => {
                cur = held_field_type(&held_fields, idx, "subtree_type")
            }
            UnitStep::NoUnit | UnitStep::Capture { .. } | UnitStep::Unit => return None,
        }
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::{
        acted_references, as_arg_projection, held_field_type, origin, rc_units, truncate_to_unit,
        unit_step, Binding, Origin, UnitStep, VarTable,
    };
    use crate::ast::name::FullName;
    use crate::ast::program::TypeEnv;
    use crate::ast::typedecl::Field;
    use crate::ast::types::{
        kind_arrow, kind_star, make_tyvar, tycon, type_fun, type_tyapp, type_tycon,
        type_tyvar_star, TyCon, TyConInfo, TyConVariant, TypeNode,
    };
    use crate::constants::{CLOSURE_CAPTURE_IDX, CLOSURE_FIELD_COUNT, CLOSURE_FUNPTR_IDX};
    use crate::fixstd::builtin::{
        bulitin_tycons, make_array_ty, make_dynamic_object_ty, make_i64_ty, make_punched_array_ty,
        make_punched_array_tycon,
    };
    use crate::misc::{Map, Set};
    use crate::object::{ty_to_object_ty, ObjectFieldType};
    use crate::rc_ir::ast::{FieldPath, RcVar, VarPath};
    use crate::rc_ir::leaf_map::boxed_leaf_paths;
    use crate::rc_ir::provenance::{sole_origin, LeafOrigin};
    use std::sync::Arc;

    /// The type `Test::<name>`, of a declaration these tests make themselves. It takes no type
    /// argument.
    fn test_ty(name: &str) -> Arc<TypeNode> {
        type_tycon(&tycon(FullName::from_strs(&["Test"], name)))
    }

    /// A declaration taking no type argument, holding the named fields in the order given.
    fn test_tycon_info(
        variant: TyConVariant,
        is_unbox: bool,
        fields: Vec<(&str, Arc<TypeNode>)>,
    ) -> TyConInfo {
        TyConInfo {
            kind: kind_star(),
            variant,
            is_unbox,
            tyvars: vec![],
            fields: fields
                .into_iter()
                .map(|(name, ty)| Field::make(name.to_string(), ty, None))
                .collect(),
            source: None,
            document: None,
            punched_from: None,
        }
    }

    /// `Array I64`.
    fn array_of_i64() -> Arc<TypeNode> {
        type_tyapp(make_array_ty(), make_i64_ty())
    }

    /// A type environment holding the built-in type constructors, `Std::PunchedArray` as `std.fix`
    /// declares it, and one declaration of each kind a unit walk distinguishes.
    fn type_env() -> TypeEnv {
        let mut tycons = bulitin_tycons();
        tycons.insert(
            make_punched_array_tycon(),
            TyConInfo {
                kind: kind_arrow(kind_star(), kind_star()),
                variant: TyConVariant::Struct,
                is_unbox: true,
                tyvars: vec![make_tyvar("a", &kind_star())],
                fields: vec![
                    Field::make(
                        "_arr".to_string(),
                        type_tyapp(make_array_ty(), type_tyvar_star("a")),
                        None,
                    ),
                    Field::make("_idx".to_string(), make_i64_ty(), None),
                ],
                source: None,
                document: None,
                punched_from: None,
            },
        );
        // An unboxed struct holding one reference, an unboxed union of a variant that holds one and
        // one that holds none, a boxed struct, a struct nesting the union beside a closure, and an
        // unboxed union whose payload holds two references.
        for (name, tycon_info) in [
            (
                "Pair",
                test_tycon_info(
                    TyConVariant::Struct,
                    true,
                    vec![("fst", array_of_i64()), ("snd", make_i64_ty())],
                ),
            ),
            (
                "Choice",
                test_tycon_info(
                    TyConVariant::Union,
                    true,
                    vec![("l", array_of_i64()), ("r", make_i64_ty())],
                ),
            ),
            (
                "BoxedPair",
                test_tycon_info(TyConVariant::Struct, false, vec![("a", array_of_i64())]),
            ),
            (
                "Nested",
                test_tycon_info(
                    TyConVariant::Struct,
                    true,
                    vec![
                        ("c", test_ty("Choice")),
                        ("f", type_fun(make_i64_ty(), make_i64_ty())),
                    ],
                ),
            ),
            (
                "Twins",
                test_tycon_info(
                    TyConVariant::Struct,
                    true,
                    vec![("fst", array_of_i64()), ("snd", array_of_i64())],
                ),
            ),
            (
                "TwinChoice",
                test_tycon_info(
                    TyConVariant::Union,
                    true,
                    vec![("twins", test_ty("Twins")), ("none", make_i64_ty())],
                ),
            ),
        ] {
            tycons.insert(TyCon::new(FullName::from_strs(&["Test"], name)), tycon_info);
        }
        TypeEnv::new(tycons, Map::default())
    }

    /// Every boxed leaf of a type truncates to a reference-counting unit of that type, and every
    /// unit is the truncation of a leaf.
    ///
    /// The leaf enumeration (`boxed_leaf_paths`) and the unit enumeration (`rc_units`) are two walks
    /// over one type, and `truncate_to_unit` is the bridge between them: a consume is recorded at a
    /// leaf while the retain that pays for it is keyed at a unit, so a leaf whose truncation names
    /// no unit leaves a retain that no release pairs with, and the object is freed while it is
    /// still held. `unit_of` asserts this of each key it makes; here it is asserted of the
    /// classification itself, over one type of every kind the walks distinguish.
    #[test]
    fn the_leaves_of_a_type_truncate_onto_its_units() {
        let type_env = type_env();
        let cases: Vec<Arc<TypeNode>> = vec![
            make_i64_ty(),                                      // holds no reference
            array_of_i64(),                                     // an array
            make_dynamic_object_ty(),                           // a boxed value
            type_fun(make_i64_ty(), make_i64_ty()),             // a closure
            type_tyapp(make_punched_array_ty(), make_i64_ty()), // a punched array
            test_ty("Pair"),                                    // an unboxed struct
            test_ty("Choice"),                                  // an unboxed union
            test_ty("BoxedPair"),                               // a boxed struct
            test_ty("Nested"),                                  // a struct of a union and a closure
            test_ty("Twins"),                                   // a struct of two references
            test_ty("TwinChoice"), // a union whose payload holds two references
        ];
        for ty in cases {
            let truncated: Set<FieldPath> = boxed_leaf_paths(&ty, &type_env)
                .into_iter()
                .map(|leaf| truncate_to_unit(&ty, &leaf, &type_env))
                .collect();
            let units: Set<FieldPath> = rc_units(&ty, &type_env).into_iter().collect();
            assert_eq!(
                truncated,
                units,
                "the leaves of `{}` do not truncate onto its units",
                ty.to_string()
            );
        }
    }

    /// A field is found by the index it sits at in the layout, so a field after a punched one keeps
    /// its own index.
    ///
    /// A punched field holds nothing and is left out of the fields a walk descends, while every
    /// other field keeps the index it is addressed by. Reading the list by position instead would
    /// answer an index with a neighbour's type, and the walk would carry on down a type the value
    /// does not have there.
    #[test]
    fn a_held_field_is_found_by_its_layout_index() {
        // A three-field value whose middle field is punched.
        let held = vec![(0, make_i64_ty()), (2, array_of_i64())];
        assert_eq!(held_field_type(&held, 0, "test"), make_i64_ty());
        assert_eq!(held_field_type(&held, 2, "test"), array_of_i64());
    }

    /// A path index naming a punched field aborts the walk, and the message names the walk that
    /// aborted.
    ///
    /// Every path that reference counting works with comes from a unit or a leaf enumeration, and
    /// both leave a punched field out, so such an index means the path and the type disagree.
    /// Answering it with a type would key a reference-count operation to a slot that holds nothing.
    #[test]
    #[should_panic(expected = "truncate_to_unit")]
    fn a_punched_field_index_aborts_the_walk() {
        let held = vec![(0, make_i64_ty()), (2, array_of_i64())];
        held_field_type(&held, 1, "truncate_to_unit");
    }

    /// A closure's step is its capture, and the fields it reports are the fields a closure is laid
    /// out with.
    ///
    /// `unit_step` states the capture's index and the closure's field count apart from the layout
    /// `ty_to_object_ty` gives, and `param_ownership_shape` builds a table of that width indexed by
    /// field index. A closure that grew a field would leave the count behind, and the capture would
    /// be recorded at a field index that is no longer the capture's.
    #[test]
    fn a_closures_step_is_the_capture_its_layout_holds() {
        let type_env = type_env();
        let closure_ty = type_fun(make_i64_ty(), make_i64_ty());
        match unit_step(&closure_ty, &type_env) {
            UnitStep::Capture {
                capture_idx,
                field_count,
            } => {
                assert_eq!(capture_idx, CLOSURE_CAPTURE_IDX as usize);
                assert_eq!(field_count, CLOSURE_FIELD_COUNT);
            }
            _ => panic!("a closure's unit step is its capture"),
        }
        let object_ty = ty_to_object_ty(&closure_ty, &vec![], &type_env);
        assert_eq!(object_ty.field_types.len(), CLOSURE_FIELD_COUNT);
        assert!(matches!(
            object_ty.field_types[CLOSURE_FUNPTR_IDX as usize],
            ObjectFieldType::LambdaFunction(_)
        ));
        assert!(matches!(
            object_ty.field_types[CLOSURE_CAPTURE_IDX as usize],
            ObjectFieldType::SubObject(_, false)
        ));
    }

    /// The sources of one result leaf, as `result_prov` declares them.
    fn sources(srcs: Vec<LeafOrigin>) -> Set<LeafOrigin> {
        srcs.into_iter().collect()
    }

    /// A result leaf whose only source is one argument leaf aliases that leaf, and is reported with
    /// the argument's index and path.
    #[test]
    fn a_lone_arg_is_a_projection() {
        let leaf_srcs = sole_origin(LeafOrigin::Arg(1, vec![0]));
        assert_eq!(as_arg_projection(&leaf_srcs), Some((1, vec![0])));
    }

    /// A result leaf that is the argument on one path and a new value on another aliases neither:
    /// the op consumes the argument, and `origin` stops at the op. Reading such a leaf as a
    /// projection would drop the consume without the alias, releasing one object twice.
    #[test]
    fn an_arg_joined_with_another_source_is_not_a_projection() {
        let leaf_srcs = sources(vec![LeafOrigin::Fresh, LeafOrigin::Arg(0, vec![])]);
        assert_eq!(as_arg_projection(&leaf_srcs), None);
    }

    /// A result leaf that may come from either of two arguments aliases neither: a projection names
    /// one argument, and here the choice would fall to whichever of the two the set yields first.
    #[test]
    fn one_of_two_args_is_not_a_projection() {
        let leaf_srcs = sources(vec![LeafOrigin::Arg(0, vec![]), LeafOrigin::Arg(1, vec![])]);
        assert_eq!(as_arg_projection(&leaf_srcs), None);
    }

    /// A leaf the op itself produced — a fresh object, or one of unknown origin — aliases no
    /// argument.
    #[test]
    fn a_produced_leaf_is_not_a_projection() {
        assert_eq!(as_arg_projection(&sole_origin(LeafOrigin::Fresh)), None);
        assert_eq!(as_arg_projection(&sole_origin(LeafOrigin::Unknown)), None);
    }

    /// A leaf with no source at all — the result of `_undefined_internal`, which aborts — aliases no
    /// argument.
    #[test]
    fn a_bottom_leaf_is_not_a_projection() {
        assert_eq!(as_arg_projection(&sources(vec![])), None);
    }

    /// A local variable of the given type.
    fn typed_var(name: &str, ty: Arc<TypeNode>) -> RcVar {
        RcVar {
            name: FullName::local(name),
            ty,
            source: None,
            debug_name: None,
            skip_null_check: false,
        }
    }

    /// A local variable of type `I64`.
    fn var(name: &str) -> RcVar {
        typed_var(name, make_i64_ty())
    }

    /// A table of the given bindings, recording each variable's type as `VarTable::of` does. A
    /// parameter's type is recorded among the parameter types as well.
    fn table(bindings: Vec<(RcVar, Binding)>) -> VarTable {
        let mut vars = VarTable::empty();
        for (v, binding) in bindings {
            if matches!(binding, Binding::Param) {
                vars.param_tys.insert(v.name.clone(), v.ty.clone());
            }
            vars.bindings.insert(v.name.clone(), binding);
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
        let vars = table(vec![(var("p"), Binding::Producer)]);
        assert_eq!(origin_of(&vars, "p"), Origin::Exactly(at("p")));
    }

    /// A move-bind reaches through to the variable it moved, so both names key to one object.
    #[test]
    fn a_move_bind_is_the_moved_variable() {
        let vars = table(vec![
            (var("p"), Binding::Producer),
            (var("m"), Binding::Move(var("p"))),
        ]);
        assert_eq!(origin_of(&vars, "m"), Origin::Exactly(at("p")));
    }

    /// A match binding whose arms produce different objects is one of them: its candidates are the
    /// arms' results, and the join itself is the name every alias chain through it agrees on.
    #[test]
    fn a_match_binding_may_be_any_arm_result() {
        let vars = table(vec![
            (var("p"), Binding::Producer),
            (var("q"), Binding::Producer),
            (var("m"), Binding::Join(vec![var("p"), var("q")])),
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
            (var("p"), Binding::Producer),
            (var("m1"), Binding::Move(var("p"))),
            (var("m"), Binding::Join(vec![var("p"), var("m1")])),
        ]);
        assert_eq!(origin_of(&vars, "m"), Origin::Exactly(at("p")));
    }

    /// A move of a match binding keeps the join's identity: the identity has to survive an alias
    /// chain, or a retain of the binding and a release of the moved-to variable would key
    /// differently and never pair.
    #[test]
    fn a_move_of_a_match_binding_keeps_the_joins_name() {
        let vars = table(vec![
            (var("p"), Binding::Producer),
            (var("q"), Binding::Producer),
            (var("m"), Binding::Join(vec![var("p"), var("q")])),
            (var("n"), Binding::Move(var("m"))),
        ]);
        assert_eq!(origin_of(&vars, "n").identity(), &at("m"));
    }

    /// A retain of an unboxed union acts on every reference its payload holds, while a release of
    /// one field of that payload acts on one of them.
    ///
    /// Both key to one reference-counting unit — the union — so the key alone would pair them, and
    /// cancelling the retain against that one release would leave the payload's other reference
    /// released without ever having been retained. The references each acts on are what tells the
    /// two apart: the field's are covered by the union's without exhausting them, and it takes the
    /// releases of both fields to un-bump the retain.
    #[test]
    fn a_union_holds_the_references_of_every_field_of_its_payload() {
        let type_env = type_env();
        let scrutinee = typed_var("u", test_ty("TwinChoice"));
        let payload = typed_var("p", test_ty("Twins"));
        let vars = table(vec![
            (scrutinee.clone(), Binding::Param),
            (
                payload.clone(),
                Binding::Payload(scrutinee.clone(), Some(0)),
            ),
        ]);

        let whole_union = acted_references(&vars, &type_env, &scrutinee, &vec![]);
        let first_field = acted_references(&vars, &type_env, &payload, &vec![0]);
        let second_field = acted_references(&vars, &type_env, &payload, &vec![1]);
        assert!(whole_union.covers(&first_field));
        assert_ne!(whole_union, first_field);
        assert!(
            !first_field.covers(&second_field),
            "the two fields of the payload hold references of different objects"
        );

        let mut outstanding = whole_union;
        outstanding.subtract(&first_field);
        assert!(
            !outstanding.is_empty(),
            "the release of one field leaves the payload's other reference bumped"
        );
        assert!(outstanding.covers(&second_field));
        outstanding.subtract(&second_field);
        assert!(
            outstanding.is_empty(),
            "the releases of both fields un-bump the retain of the union"
        );
    }

    /// A join over another join flattens: every result the inner join could yield is among the outer
    /// join's candidates.
    #[test]
    fn a_join_of_joins_may_be_any_of_their_results() {
        let vars = table(vec![
            (var("p"), Binding::Producer),
            (var("q"), Binding::Producer),
            (var("r"), Binding::Producer),
            (var("inner"), Binding::Join(vec![var("p"), var("q")])),
            (var("m"), Binding::Join(vec![var("inner"), var("r")])),
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
