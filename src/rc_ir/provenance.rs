//! Provenance analysis of the RC IR.
//!
//! The analysis abstractly interprets a function and tracks, for every boxed leaf of every variable,
//! its *provenance*: where the value came from. A leaf is `Fresh` (a newly produced value, uniquely
//! owned), `Dyn` (of unknown sharing — read out of a boxed container, a global, or the result of a
//! `Retain` that made a second reference), or `Arg(i, path)` (it flows unchanged from input `i`'s
//! leaf at `path`). Uniqueness is recovered later by resolving a function's provenance against the
//! uniqueness of its actual inputs (`Fresh` resolves to unique, `Dyn` to dynamic); that resolution
//! and the elimination it drives are a later pass, so this pass only computes provenance.
//!
//! `Retain` is the only operation that demotes a leaf (`Fresh`/`Arg` to `Dyn`): duplicating a
//! reference makes the value shared. Everything else preserves provenance. A boxed container's
//! contents are not tracked (reading an element yields `Dyn`); an unboxed aggregate's children are
//! tracked, so a boxed value threaded through a tuple or an unboxed union (a loop state) keeps its
//! provenance.
//!
//! A function's parameters are seeded symbolically (`Arg`), primitive results are declared by
//! `result_prov` and composed with the operands, and branches join by set union. Each function's
//! effect — its result provenance, symbolic in its parameters — is computed to a fixed point (so
//! recursion converges) and substituted at a direct call site; an indirect call is conservatively
//! `Dyn`. A `Retain` demotes the retained variable's own leaves. Demoting the *other* variables that
//! alias the same object — one reached by projecting the same unboxed-aggregate leaf — needs the
//! shared object-identity (`root`) analysis, which the borrow-inference pass introduces; the demotion
//! becomes root-based then.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::misc::{Map, Set};
use crate::rc_ir::ast::{
    FuncRef, MatchArm, Path, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar,
};
use std::sync::Arc;

/// The origin of one boxed leaf.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BaseSource {
    /// A newly produced value: an allocation, or a force-unique op's result. Resolves to `Unique`.
    Fresh,
    /// Of unknown sharing: read out of a boxed container, a global, or duplicated by a `Retain`.
    /// Resolves to `Dynamic`. An absorbing state (`Fresh`/`Arg` demote to it, never back).
    Dyn,
    /// Carried unchanged from input `i`'s leaf at the given path (identity, projection, passthrough).
    Arg(usize, Path),
}

/// The origin of one boxed leaf as a set of `BaseSource`s: usually a singleton, several after a
/// branch join, empty for an absent union variant (the bottom of the lattice).
pub type LeafSource = Set<BaseSource>;

/// The provenance of a whole value, shaped like the value's type: `Unboxed` for a value with no
/// boxed leaf, `UnboxedAgg` for an unboxed aggregate whose children are tracked, `Boxed` for a boxed
/// leaf (whose contents are not tracked).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Provenance {
    Unboxed,
    UnboxedAgg(Vec<Provenance>),
    Boxed(LeafSource),
}

impl Provenance {
    /// The singleton leaf-source `{src}`.
    pub fn leaf(src: BaseSource) -> LeafSource {
        let mut s = Set::default();
        s.insert(src);
        s
    }

    /// Build the provenance shape of a value of type `ty`. `leaf` is called once for each boxed leaf,
    /// receiving the path to that leaf — the sequence of field indices from the root of `ty` down to
    /// it — so `leaf` can tell which leaf it is describing (e.g. to record `Arg(i, path)`). The shape
    /// mirrors the type: a scalar (no fields) becomes `Unboxed`, an unboxed aggregate expands into an
    /// `UnboxedAgg` over its fields (a union's variants' payloads) even when it holds no boxed leaf,
    /// and a closure becomes `{funptr, capture}` with the capture a single boxed leaf.
    pub fn build_shape(
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        leaf: &dyn Fn(&Path) -> LeafSource,
    ) -> Provenance {
        // The recursion accumulates the field indices from the root into `path`, which is what `leaf`
        // is handed at each boxed leaf. It always starts empty, so it stays internal here rather than
        // being a caller-facing parameter.
        fn go(
            ty: &Arc<TypeNode>,
            type_env: &TypeEnv,
            leaf: &dyn Fn(&Path) -> LeafSource,
            path: &mut Path,
        ) -> Provenance {
            if ty.is_closure() {
                // A closure lowers to `{funptr, capture-pointer}`; only the capture is a boxed leaf.
                path.push(CLOSURE_CAPTURE_IDX as usize);
                let cap = Provenance::Boxed(leaf(path));
                path.pop();
                return Provenance::UnboxedAgg(vec![Provenance::Unboxed, cap]);
            }
            if ty.is_box(type_env) {
                return Provenance::Boxed(leaf(path));
            }
            // Expand every unboxed aggregate (struct, tuple, or unboxed union) into its fields (for a
            // union, its variants' payloads), even when it holds no boxed leaf, so a value's shape
            // always mirrors its type. Only a scalar (an unboxed leaf with no fields) becomes
            // `Unboxed`. Normalizing to this expanded shape keeps `join` total without a fallback.
            let fields = ty.field_types(type_env);
            if fields.is_empty() {
                return Provenance::Unboxed;
            }
            let mut children = Vec::with_capacity(fields.len());
            for (i, fty) in fields.iter().enumerate() {
                path.push(i);
                children.push(go(fty, type_env, leaf, path));
                path.pop();
            }
            Provenance::UnboxedAgg(children)
        }
        go(ty, type_env, leaf, &mut Vec::new())
    }

    /// The provenance whose every boxed leaf is `src`.
    pub fn uniform(ty: &Arc<TypeNode>, type_env: &TypeEnv, src: BaseSource) -> Provenance {
        Provenance::build_shape(ty, type_env, &|_| Provenance::leaf(src.clone()))
    }

    /// The provenance whose every boxed leaf at path `π` is `Arg(arg_index, π)` — the whole value of
    /// input `arg_index` carried through unchanged.
    pub fn arg_passthrough(ty: &Arc<TypeNode>, type_env: &TypeEnv, arg_index: usize) -> Provenance {
        Provenance::build_shape(ty, type_env, &|path: &Path| {
            Provenance::leaf(BaseSource::Arg(arg_index, path.clone()))
        })
    }

    /// Pointwise join (branch merge): union the leaf sources, recurse into aggregates.
    fn join(&self, other: &Provenance) -> Provenance {
        match (self, other) {
            (Provenance::Unboxed, Provenance::Unboxed) => Provenance::Unboxed,
            (Provenance::UnboxedAgg(a), Provenance::UnboxedAgg(b)) if a.len() == b.len() => {
                Provenance::UnboxedAgg(a.iter().zip(b).map(|(x, y)| x.join(y)).collect())
            }
            (Provenance::Boxed(a), Provenance::Boxed(b)) => {
                Provenance::Boxed(a.union(b).cloned().collect())
            }
            // Both sides of a branch merge have the same type, and a provenance's shape now mirrors
            // its type exactly (`build_shape` expands every aggregate; a value read from a boxed
            // container is a `Boxed` leaf), so the arms above are exhaustive. A mismatch would be a
            // bug in the analysis, not valid input; fail loudly rather than silently returning a wrong
            // provenance, which feeds unique-check elimination and could mask a miscompile.
            _ => unreachable!(
                "Provenance::join on mismatched shapes: {:?} vs {:?}",
                self, other
            ),
        }
    }

    /// The leaf source at path `π`, navigating through aggregates to the boxed leaf. An empty set when
    /// `π` ends at a position that is not a single boxed leaf — an aggregate root or a scalar,
    /// including an `Arg(j, [])` source ("the whole of operand `j`") composed against an aggregate
    /// operand. A non-empty `π` that runs off the end of an aggregate or continues past a boxed or
    /// scalar leaf is inconsistent with the value's type — since a provenance's shape mirrors its
    /// type, that cannot happen for a well-formed query.
    pub fn leaf_at(&self, path: &[usize]) -> LeafSource {
        match (self, path.split_first()) {
            (Provenance::UnboxedAgg(children), Some((i, rest))) if *i < children.len() => {
                children[*i].leaf_at(rest)
            }
            (Provenance::Boxed(ls), None) => ls.clone(),
            (Provenance::UnboxedAgg(_) | Provenance::Unboxed, None) => Set::default(),
            (_, Some(_)) => unreachable!(
                "Provenance::leaf_at: path {:?} inconsistent with shape {:?}",
                path, self
            ),
        }
    }

    /// Substitute the `Arg(j, σ)` symbols of a declared provenance with the operands' provenance:
    /// `Arg(j, σ)` becomes operand `j`'s leaf source at `σ`. `Fresh`/`Dyn` stay. Used to compose a
    /// primitive's declared `result_prov` (and, later, a callee's effect) with its actual operands.
    fn compose(&self, operand_provs: &[Provenance]) -> Provenance {
        match self {
            Provenance::Unboxed => Provenance::Unboxed,
            Provenance::UnboxedAgg(children) => {
                Provenance::UnboxedAgg(children.iter().map(|c| c.compose(operand_provs)).collect())
            }
            Provenance::Boxed(ls) => {
                let mut out = Set::default();
                for src in ls {
                    match src {
                        BaseSource::Fresh => {
                            out.insert(BaseSource::Fresh);
                        }
                        BaseSource::Dyn => {
                            out.insert(BaseSource::Dyn);
                        }
                        BaseSource::Arg(j, sigma) => match operand_provs.get(*j) {
                            Some(op) => {
                                for s in op.leaf_at(sigma) {
                                    out.insert(s);
                                }
                            }
                            // An argument index with no operand (a partial application, or a stray
                            // `Arg` in a callee's effect) is resolved conservatively to `Dyn` rather
                            // than dropped, which would wrongly leave the leaf `⊥` (unique).
                            None => {
                                out.insert(BaseSource::Dyn);
                            }
                        },
                    }
                }
                Provenance::Boxed(out)
            }
        }
    }

    /// The provenance of field `i` of an unboxed aggregate. Only an unboxed aggregate is projected: a
    /// field read out of a boxed container is handled by the caller (its leaves become `Dyn`), and a
    /// scalar has no field.
    fn project(&self, i: usize) -> Provenance {
        match self {
            Provenance::UnboxedAgg(children) => {
                assert!(
                    i < children.len(),
                    "project: field index {} out of range ({} children)",
                    i,
                    children.len()
                );
                children[i].clone()
            }
            _ => unreachable!("project: receiver {:?} is not an unboxed aggregate", self),
        }
    }

    /// Demote every boxed leaf under `path` to `Dyn` (the effect of duplicating the reference with a
    /// `Retain`). An empty path demotes the whole value.
    fn demote(&self, path: &[usize]) -> Provenance {
        match path.split_first() {
            None => self.map_leaves(&|_| Provenance::leaf(BaseSource::Dyn)),
            Some((i, rest)) => match self {
                Provenance::UnboxedAgg(children) if *i < children.len() => {
                    let mut cs = children.clone();
                    cs[*i] = cs[*i].demote(rest);
                    Provenance::UnboxedAgg(cs)
                }
                // A non-empty path descends only through unboxed aggregates; reaching a boxed or
                // scalar leaf, or an out-of-range index, means the path is inconsistent with the
                // value's type — a bug. (Swallowing it would leave a leaf that should demote to `Dyn`
                // still `Fresh`/`Arg`, i.e. wrongly unique.)
                _ => unreachable!("demote: path {:?} inconsistent with shape {:?}", path, self),
            },
        }
    }

    /// Rewrite every boxed leaf's source through `f`.
    fn map_leaves(&self, f: &dyn Fn(&LeafSource) -> LeafSource) -> Provenance {
        match self {
            Provenance::Unboxed => Provenance::Unboxed,
            Provenance::UnboxedAgg(children) => {
                Provenance::UnboxedAgg(children.iter().map(|c| c.map_leaves(f)).collect())
            }
            Provenance::Boxed(ls) => Provenance::Boxed(f(ls)),
        }
    }

    /// A readable one-line rendering, for the RC IR dump.
    pub fn to_string(&self) -> String {
        match self {
            Provenance::Unboxed => "unboxed".to_string(),
            Provenance::UnboxedAgg(children) => {
                let inner = children
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }
            Provenance::Boxed(ls) => leaf_source_to_string(ls),
        }
    }
}

/// Render a leaf source as `fresh` / `dyn` / `arg{i}{.path}`, joining several with `|`, and the
/// empty set (an absent union variant) as `_`.
fn leaf_source_to_string(ls: &LeafSource) -> String {
    if ls.is_empty() {
        return "_".to_string();
    }
    // Sort for a deterministic dump (the set has no inherent order).
    let mut parts: Vec<String> = ls
        .iter()
        .map(|s| match s {
            BaseSource::Fresh => "fresh".to_string(),
            BaseSource::Dyn => "dyn".to_string(),
            BaseSource::Arg(i, path) => {
                let p = path.iter().map(|i| format!(".{}", i)).collect::<String>();
                format!("arg{}{}", i, p)
            }
        })
        .collect();
    parts.sort();
    parts.join(" | ")
}

impl Provenance {
    /// The provenance whose every boxed leaf is bottom (the empty set) — an absent union variant.
    pub fn uniform_bottom(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Provenance {
        Provenance::build_shape(ty, type_env, &|_| Set::default())
    }
}

/// The compile-time reference-count verdict for one boxed leaf, obtained by resolving its provenance
/// against the uniqueness of a function's inputs: `Fresh` resolves to `Unique`, `Dyn` to `Dynamic`,
/// and `Arg(i, π)` to input `i`'s verdict at `π`. A two-point lattice with `Unique < Dynamic`, so a
/// leaf sourced from several places is `Unique` only when every source is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum CTRefCnt {
    Unique,
    Dynamic,
}

/// The resolved uniqueness of a whole value, shaped like the value's type (mirroring `Provenance`,
/// with each boxed leaf a `CTRefCnt` instead of a leaf source). Specialization keys a function clone
/// on its parameters' shapes, so it is `Hash`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Uniqueness {
    Unboxed,
    UnboxedAgg(Vec<Uniqueness>),
    Boxed(CTRefCnt),
}

impl Uniqueness {
    /// The verdict at path `π`. `Dynamic` (the conservative default) when `π` ends at a position that
    /// is not a single boxed leaf — an aggregate root or a scalar. A non-empty `π` that runs off the
    /// end of an aggregate or past a boxed/scalar leaf is inconsistent with the value's type, which
    /// cannot happen for a well-formed query. (Mirrors `Provenance::leaf_at`.)
    fn leaf_at(&self, path: &[usize]) -> CTRefCnt {
        match (self, path.split_first()) {
            (Uniqueness::UnboxedAgg(children), Some((i, rest))) if *i < children.len() => {
                children[*i].leaf_at(rest)
            }
            (Uniqueness::Boxed(rc), None) => *rc,
            (Uniqueness::UnboxedAgg(_) | Uniqueness::Unboxed, None) => CTRefCnt::Dynamic,
            (_, Some(_)) => unreachable!(
                "Uniqueness::leaf_at: path {:?} inconsistent with shape {:?}",
                path, self
            ),
        }
    }

    /// The uniqueness shape of a value of type `ty` whose every boxed leaf is `Dynamic` — the input
    /// uniqueness of a function whose caller supplies no static information (an entry point, an
    /// indirectly reached function, or the baseline version of any function).
    pub fn all_dynamic(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Uniqueness {
        resolve(&Provenance::uniform(ty, type_env, BaseSource::Dyn), &[])
    }
}

/// Resolve one leaf source against the input uniqueness: `Unique` unless some source is `Dynamic` (a
/// `Dyn`, or an `Arg` reaching a `Dynamic` input leaf). An empty set (an absent union variant, the
/// bottom of the lattice) resolves to `Unique`.
fn resolve_leaf(ls: &LeafSource, inputs: &[Uniqueness]) -> CTRefCnt {
    for src in ls {
        let rc = match src {
            BaseSource::Fresh => CTRefCnt::Unique,
            BaseSource::Dyn => CTRefCnt::Dynamic,
            BaseSource::Arg(i, path) => inputs
                .get(*i)
                .map_or(CTRefCnt::Dynamic, |u| u.leaf_at(path)),
        };
        if rc == CTRefCnt::Dynamic {
            return CTRefCnt::Dynamic;
        }
    }
    CTRefCnt::Unique
}

/// Resolve a provenance against the uniqueness of its function's inputs, mapping each boxed leaf to
/// its `CTRefCnt` verdict. `inputs[i]` is the uniqueness of parameter `i`; a parameter beyond the end
/// (an unspecialized function, whose inputs are unknown) leaves its `Arg` leaves `Dynamic`.
///
/// An aggregate with no boxed leaf collapses to `Unboxed`: it carries no uniqueness, so the shape
/// (which a provenance renders inconsistently — a constructed empty struct is an empty aggregate, the
/// same struct as a parameter is `Unboxed`) must not make two otherwise equal specialization keys
/// differ.
pub fn resolve(prov: &Provenance, inputs: &[Uniqueness]) -> Uniqueness {
    match prov {
        Provenance::Unboxed => Uniqueness::Unboxed,
        Provenance::UnboxedAgg(children) => {
            let resolved: Vec<Uniqueness> = children.iter().map(|c| resolve(c, inputs)).collect();
            if resolved.iter().all(|u| *u == Uniqueness::Unboxed) {
                Uniqueness::Unboxed
            } else {
                Uniqueness::UnboxedAgg(resolved)
            }
        }
        Provenance::Boxed(ls) => Uniqueness::Boxed(resolve_leaf(ls, inputs)),
    }
}

/// Whether the boxed leaf at path `π` of a value with this provenance is statically `Unique`, given
/// its function's input uniqueness. Passing no inputs treats every `Arg` leaf as `Dynamic`, the
/// sound verdict for a function whose inputs are unknown (only its locally produced `Fresh` values
/// are then unique).
pub fn leaf_is_unique(prov: &Provenance, path: &[usize], inputs: &[Uniqueness]) -> bool {
    resolve_leaf(&prov.leaf_at(path), inputs) == CTRefCnt::Unique
}

/// The provenance analysis of one function or global initializer: a forward abstract interpretation
/// that records each variable's provenance at its binding point.
struct Interp<'a> {
    type_env: &'a TypeEnv,
    /// Each function's result provenance, symbolic in its parameters (`Arg`), used to compose a
    /// direct call at its call site. Built to a fixed point before the recording pass.
    effects: &'a Map<FuncRef, Provenance>,
    /// The function each local closure value targets, so a call through it resolves to a direct
    /// callee. Populated as `Closure` bindings are interpreted (a forward pass sees them first).
    closure_targets: Map<FullName, FuncRef>,
    /// The provenance recorded at each variable's binding, for the dump. Names are globally unique.
    bindings: Map<FullName, Provenance>,
    /// For each operation that branches on a container's uniqueness (a force-unique op or `is_unique`),
    /// keyed by its result variable, the provenance of that container operand at the operation's
    /// program point. Unlike `bindings` (a value's provenance where it is bound), this is read from the
    /// live environment at the operation, so a container demoted to `Dyn` by an intervening `Retain` is
    /// seen as `Dyn` here — the flow-sensitive fact unique-check elimination needs to decide, per
    /// operation, whether the container is still unique.
    op_containers: Map<FullName, Provenance>,
    /// For each call, keyed by its result variable, the provenance of each argument at the call's
    /// program point. Specialization keys the callee's clone on these (resolved against the caller's
    /// own input uniqueness), again taking the call-point value rather than the arguments' bindings.
    call_args: Map<FullName, Vec<Provenance>>,
}

impl<'a> Interp<'a> {
    fn new(type_env: &'a TypeEnv, effects: &'a Map<FuncRef, Provenance>) -> Self {
        Interp {
            type_env,
            effects,
            closure_targets: Map::default(),
            bindings: Map::default(),
            op_containers: Map::default(),
            call_args: Map::default(),
        }
    }

    /// Seed a parameter (or capture) as the symbolic input `Arg(arg_index, π)`.
    fn seed_param(&mut self, var: &RcVar, arg_index: usize, env: &mut Map<FullName, Provenance>) {
        let prov = Provenance::arg_passthrough(&var.ty, self.type_env, arg_index);
        self.record(var, &prov, env);
    }

    /// Record a variable's provenance both in the live environment and in the binding table.
    fn record(&mut self, var: &RcVar, prov: &Provenance, env: &mut Map<FullName, Provenance>) {
        env.insert(var.name.clone(), prov.clone());
        self.bindings.insert(var.name.clone(), prov.clone());
    }

    /// The provenance of an operand from the environment; a value not in scope (a global reference)
    /// has `Dyn` boxed leaves.
    fn operand(&self, var: &RcVar, env: &Map<FullName, Provenance>) -> Provenance {
        match env.get(&var.name) {
            Some(p) => p.clone(),
            None => Provenance::uniform(&var.ty, self.type_env, BaseSource::Dyn),
        }
    }

    /// Interpret an expression, threading the environment forward. Returns the provenance of the
    /// expression's value (the value its final `Ret` returns) and the environment at its exit.
    fn interp(
        &mut self,
        node: &RcExprNode,
        env: Map<FullName, Provenance>,
    ) -> (Provenance, Map<FullName, Provenance>) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.interp_inner(node, env))
    }

    fn interp_inner(
        &mut self,
        node: &RcExprNode,
        mut env: Map<FullName, Provenance>,
    ) -> (Provenance, Map<FullName, Provenance>) {
        match node.expr.as_ref() {
            RcExpr::Ret(x) => (self.operand(x, &env), env),
            // A `match` needs its arms' exit environments joined for the continuation (a variable
            // demoted on one arm but live after the match must stay demoted downstream), so it is
            // handled separately from the other right-hand sides.
            RcExpr::Let(x, RcRhs::Match(scrut, arms), cont) => {
                let (result, exit_env) = self.interp_match(scrut, arms, &env);
                env = exit_env;
                self.record(x, &result, &mut env);
                self.interp(cont, env)
            }
            RcExpr::Let(x, rhs, cont) => {
                let prov = self.interp_rhs(x, rhs, &env);
                self.record(x, &prov, &mut env);
                self.interp(cont, env)
            }
            RcExpr::Retain(var, path, _, cont) => {
                if let Some(p) = env.get(&var.name) {
                    let demoted = p.demote(path);
                    env.insert(var.name.clone(), demoted);
                }
                self.interp(cont, env)
            }
            RcExpr::Release(_, _, _, cont) => self.interp(cont, env),
            RcExpr::Destructure(container, fields, cont) => {
                // Destructuring a boxed container retains each field out of the shared allocation, so
                // every field's boxed leaf is `Dyn` (the same read-out-of-a-shared-box rule as a boxed
                // union's payload in `interp_match`). An unboxed container's fields carry the tracked
                // provenance projected from the container.
                let boxed = container.ty.is_box(self.type_env);
                let cprov = self.operand(container, &env);
                for (idx, fv) in fields {
                    let fprov = if boxed {
                        Provenance::uniform(&fv.ty, self.type_env, BaseSource::Dyn)
                    } else {
                        cprov.project(*idx)
                    };
                    self.record(fv, &fprov, &mut env);
                }
                self.interp(cont, env)
            }
        }
    }

    /// The provenance produced by a `let`'s right-hand side (excluding `Match`, handled by the
    /// caller for its environment join).
    fn interp_rhs(
        &mut self,
        result: &RcVar,
        rhs: &RcRhs,
        env: &Map<FullName, Provenance>,
    ) -> Provenance {
        match rhs {
            RcRhs::Var(y) => self.operand(y, env),
            RcRhs::Llvm(gen, args) => {
                let arg_provs: Vec<Provenance> =
                    args.iter().map(|a| self.operand(a, env)).collect();
                // Snapshot the checked container operand of a uniqueness-branching operation at this
                // program point, for unique-check elimination to resolve later.
                if let Some(uc) = gen.unique_check_operand() {
                    self.op_containers
                        .insert(result.name.clone(), arg_provs[uc.container_index].clone());
                }
                let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
                let decl = gen.result_prov(&result.ty, &arg_tys, self.type_env);
                decl.compose(&arg_provs)
            }
            RcRhs::Closure(fref, _) => {
                // `{funptr, capture}`: the capture is a freshly allocated (or null) object. Remember
                // which function this closure targets so a later call through it resolves directly.
                self.closure_targets
                    .insert(result.name.clone(), fref.clone());
                Provenance::uniform(&result.ty, self.type_env, BaseSource::Fresh)
            }
            RcRhs::App(callee, args) => self.interp_app(result, callee, args, env),
            RcRhs::Match(..) => {
                unreachable!("a Match rhs is handled by interp_inner for its environment join")
            }
        }
    }

    /// The provenance of a call. A direct call — one whose callee resolves to a known function (a
    /// closure value built here, or a top-level function referenced by name) — composes that
    /// function's effect with the actual arguments. An indirect call, or a partial application whose
    /// argument count does not match the callee's parameters, is conservatively `Dyn`.
    fn interp_app(
        &mut self,
        result: &RcVar,
        callee: &RcVar,
        args: &[RcVar],
        env: &Map<FullName, Provenance>,
    ) -> Provenance {
        // Snapshot the arguments' provenance at this call's program point, for specialization to key
        // the callee's clone on (again, the call-point value rather than the arguments' bindings).
        let arg_provs: Vec<Provenance> = args.iter().map(|a| self.operand(a, env)).collect();
        self.call_args
            .insert(result.name.clone(), arg_provs.clone());

        let target = self.closure_targets.get(&callee.name).cloned().or_else(|| {
            let fref = FuncRef {
                name: callee.name.clone(),
            };
            self.effects.contains_key(&fref).then_some(fref)
        });
        if let Some(fref) = target {
            if let Some(effect) = self.effects.get(&fref) {
                // The effect is symbolic in the callee's parameters; substitute the actual
                // arguments (`compose` resolves any unmatched parameter to `Dyn`, so an arity
                // mismatch stays sound).
                return effect.compose(&arg_provs);
            }
        }
        Provenance::uniform(&result.ty, self.type_env, BaseSource::Dyn)
    }

    /// Interpret a function body from its parameters (seeded symbolically as `Arg`) and return its
    /// result provenance, recording each binding into `self.bindings`.
    fn run_func(&mut self, func: &RcFunc) -> Provenance {
        let mut env = Map::default();
        for (i, p) in func.params.iter().enumerate() {
            self.seed_param(p, i, &mut env);
        }
        if let Some(cap) = &func.cap {
            // The capture sits just past the parameters in the argument numbering.
            self.seed_param(cap, func.params.len(), &mut env);
        }
        let (result, _exit) = self.interp(&func.body, env);
        result
    }

    /// Interpret a `match`: run each arm from the pre-branch environment, then join the arms' result
    /// provenances and their exit environments pointwise. The joined exit environment carries a
    /// demotion that happened on only one arm forward to the continuation (a variable made `Dyn` in
    /// one branch must stay `Dyn` where the branches merge).
    fn interp_match(
        &mut self,
        scrut: &RcVar,
        arms: &[MatchArm],
        env: &Map<FullName, Provenance>,
    ) -> (Provenance, Map<FullName, Provenance>) {
        let sprov = self.operand(scrut, env);
        let mut joined_result: Option<Provenance> = None;
        let mut joined_env: Option<Map<FullName, Provenance>> = None;
        for arm in arms {
            let mut arm_env = env.clone();
            // The payload is the variant's value (a tagged arm) or the whole scrutinee (catch-all). A
            // variant payload of an unboxed union is projected from the scrutinee's expanded shape; of
            // a boxed union it is read out of the shared container, so its every boxed leaf is `Dyn`.
            let payload_prov = match arm.variant {
                Some(tag) => {
                    if scrut.ty.is_box(self.type_env) {
                        Provenance::uniform(&arm.payload.ty, self.type_env, BaseSource::Dyn)
                    } else {
                        sprov.project(tag)
                    }
                }
                None => sprov.clone(),
            };
            self.record(&arm.payload, &payload_prov, &mut arm_env);
            let (arm_prov, arm_exit) = self.interp(&arm.body, arm_env);
            joined_result = Some(match joined_result {
                None => arm_prov,
                Some(acc) => acc.join(&arm_prov),
            });
            joined_env = Some(match joined_env {
                None => arm_exit,
                Some(acc) => join_envs(&acc, &arm_exit),
            });
        }
        (
            joined_result.unwrap_or(Provenance::Unboxed),
            joined_env.unwrap_or_else(|| env.clone()),
        )
    }
}

/// Pointwise join of two environments: a variable present in both is joined; one present on only a
/// single side (an arm-local binding, out of scope past the match) is kept as is.
fn join_envs(
    a: &Map<FullName, Provenance>,
    b: &Map<FullName, Provenance>,
) -> Map<FullName, Provenance> {
    let mut out = a.clone();
    for (k, vb) in b {
        out.entry(k.clone())
            .and_modify(|va| *va = va.join(vb))
            .or_insert_with(|| vb.clone());
    }
    out
}

/// The result of analyzing a whole program. Names are globally unique, so each map is a single flat
/// table keyed by variable name.
pub struct Analysis {
    /// Each variable's provenance at its binding point (for the RC IR dump annotation).
    pub bindings: Map<FullName, Provenance>,
    /// For each operation that branches on a container's uniqueness (a force-unique op or `is_unique`,
    /// keyed by its result variable), that container operand's provenance at the operation's program
    /// point, which unique-check elimination resolves.
    pub op_containers: Map<FullName, Provenance>,
    /// For each call (keyed by its result variable), the arguments' provenance at the call's program
    /// point, which specialization resolves to key the callee's clone.
    pub call_args: Map<FullName, Vec<Provenance>>,
}

/// Analyze every function and global initializer of `prog`.
pub fn analyze_program(prog: &RcProgram, type_env: &TypeEnv) -> Analysis {
    // Phase 1: compute each function's effect (its result provenance, symbolic in its parameters) to
    // a fixed point. A direct call substitutes the callee's effect, so recursion needs iteration;
    // the lattice is finite and the join is monotone, so it converges. Start each effect at `⊥`.
    let mut effects: Map<FuncRef, Provenance> = prog
        .funcs
        .values()
        .map(|f| {
            (
                f.name.clone(),
                Provenance::uniform_bottom(&f.ret_ty, type_env),
            )
        })
        .collect();
    loop {
        let mut next = effects.clone();
        let mut changed = false;
        for func in prog.funcs.values() {
            let mut interp = Interp::new(type_env, &effects);
            let result = interp.run_func(func);
            let merged = effects[&func.name].join(&result);
            if merged != effects[&func.name] {
                next.insert(func.name.clone(), merged);
                changed = true;
            }
        }
        effects = next;
        if !changed {
            break;
        }
    }

    // Phase 2: record every variable's provenance using the converged effects.
    let mut bindings = Map::default();
    let mut op_containers = Map::default();
    let mut call_args = Map::default();
    for func in prog.funcs.values() {
        let mut interp = Interp::new(type_env, &effects);
        interp.run_func(func);
        bindings.extend(interp.bindings);
        op_containers.extend(interp.op_containers);
        call_args.extend(interp.call_args);
    }
    for glob in &prog.globals {
        let mut interp = Interp::new(type_env, &effects);
        let _ = interp.interp(&glob.init, Map::default());
        bindings.extend(interp.bindings);
        op_containers.extend(interp.op_containers);
        call_args.extend(interp.call_args);
    }
    Analysis {
        bindings,
        op_containers,
        call_args,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Provenance {
        Provenance::Boxed(Provenance::leaf(BaseSource::Fresh))
    }
    fn dyn_() -> Provenance {
        Provenance::Boxed(Provenance::leaf(BaseSource::Dyn))
    }
    fn arg(i: usize, path: Vec<usize>) -> Provenance {
        Provenance::Boxed(Provenance::leaf(BaseSource::Arg(i, path)))
    }

    #[test]
    fn join_unions_leaf_sources() {
        let j = fresh().join(&dyn_());
        match j {
            Provenance::Boxed(ls) => {
                assert!(ls.contains(&BaseSource::Fresh));
                assert!(ls.contains(&BaseSource::Dyn));
                assert_eq!(ls.len(), 2);
            }
            _ => panic!("expected Boxed"),
        }
    }

    #[test]
    fn demote_only_the_named_leaf() {
        // `(fresh, fresh)`, demoting child 0, leaves child 1 fresh.
        let agg = Provenance::UnboxedAgg(vec![fresh(), fresh()]);
        let demoted = agg.demote(&[0]);
        assert_eq!(demoted, Provenance::UnboxedAgg(vec![dyn_(), fresh()]));
    }

    #[test]
    fn demote_empty_path_demotes_whole_value() {
        let agg = Provenance::UnboxedAgg(vec![fresh(), arg(0, vec![])]);
        let demoted = agg.demote(&[]);
        assert_eq!(demoted, Provenance::UnboxedAgg(vec![dyn_(), dyn_()]));
    }

    #[test]
    fn compose_substitutes_arg_with_operand_leaf() {
        // A declaration `arg0` composed with operand 0 = `fresh` yields `fresh`.
        let decl = arg(0, vec![]);
        let composed = decl.compose(&[fresh()]);
        assert_eq!(composed, fresh());
    }

    #[test]
    fn compose_substitutes_arg_through_a_subpath() {
        // A declaration `arg0.1` composed with operand 0 = `(unboxed, fresh)` yields `fresh`.
        let decl = arg(0, vec![1]);
        let operand = Provenance::UnboxedAgg(vec![Provenance::Unboxed, fresh()]);
        let composed = decl.compose(&[operand]);
        assert_eq!(composed, fresh());
    }

    #[test]
    fn compose_keeps_fresh_and_dyn_and_unions_with_arg() {
        // `{fresh | arg1}` (a union-mod-style phi) composed with operand 1 = `dyn` yields
        // `{fresh | dyn}`.
        let mut ls = Set::default();
        ls.insert(BaseSource::Fresh);
        ls.insert(BaseSource::Arg(1, vec![]));
        let decl = Provenance::Boxed(ls);
        let composed = decl.compose(&[Provenance::Unboxed, dyn_()]);
        match composed {
            Provenance::Boxed(out) => {
                assert!(out.contains(&BaseSource::Fresh));
                assert!(out.contains(&BaseSource::Dyn));
                assert_eq!(out.len(), 2);
            }
            _ => panic!("expected Boxed"),
        }
    }

    #[test]
    fn leaf_at_navigates_to_the_boxed_leaf() {
        let agg = Provenance::UnboxedAgg(vec![Provenance::Unboxed, arg(1, vec![0])]);
        let ls = agg.leaf_at(&[1]);
        assert!(ls.contains(&BaseSource::Arg(1, vec![0])));
    }

    #[test]
    fn project_reads_an_aggregate_child() {
        let agg = Provenance::UnboxedAgg(vec![fresh(), dyn_()]);
        assert_eq!(agg.project(0), fresh());
        assert_eq!(agg.project(1), dyn_());
    }

    #[test]
    fn bottom_leaf_renders_as_underscore() {
        assert_eq!(Provenance::Boxed(Set::default()).to_string(), "_");
        assert_eq!(fresh().to_string(), "fresh");
    }

    #[test]
    fn join_envs_is_pointwise_and_keeps_one_sided_bindings() {
        let x = FullName::local("x");
        let y = FullName::local("y");
        let z = FullName::local("z");
        // `x` is fresh on one side, dyn on the other; `y` only on the left; `z` only on the right.
        let mut a = Map::default();
        a.insert(x.clone(), fresh());
        a.insert(y.clone(), fresh());
        let mut b = Map::default();
        b.insert(x.clone(), dyn_());
        b.insert(z.clone(), dyn_());

        let joined = join_envs(&a, &b);
        // A variable present on both sides is joined.
        assert_eq!(joined[&x], fresh().join(&dyn_()));
        // A variable present on only one side is kept as is.
        assert_eq!(joined[&y], fresh());
        assert_eq!(joined[&z], dyn_());
    }

    #[test]
    fn resolve_collapses_an_all_unboxed_aggregate() {
        // An aggregate with no boxed leaf resolves to `Unboxed`, so a constructed empty struct
        // (an empty aggregate) and the same struct as a parameter (`Unboxed`) key the same.
        assert_eq!(
            resolve(&Provenance::UnboxedAgg(vec![]), &[]),
            Uniqueness::Unboxed
        );
        assert_eq!(
            resolve(
                &Provenance::UnboxedAgg(vec![Provenance::Unboxed, Provenance::Unboxed]),
                &[]
            ),
            Uniqueness::Unboxed
        );
        // An aggregate with a boxed leaf keeps its shape.
        assert_eq!(
            resolve(
                &Provenance::UnboxedAgg(vec![Provenance::Unboxed, fresh()]),
                &[]
            ),
            Uniqueness::UnboxedAgg(vec![
                Uniqueness::Unboxed,
                Uniqueness::Boxed(CTRefCnt::Unique)
            ])
        );
    }
}
