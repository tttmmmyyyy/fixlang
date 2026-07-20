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
//! A branch's condition is read in one case: the arm a `match` on an `is_unique` flag takes for `true`
//! runs where the tested leaf's reference count was one, so that arm reads the leaf as `Fresh` however
//! little is known about where the value came from. Everywhere else the interpretation ignores what a
//! branch decided.
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
use crate::constants::{
    BOOL_TRUE_TAG, CLOSURE_CAPTURE_IDX, IS_UNIQUE_FLAG_FIELD, IS_UNIQUE_VALUE_FIELD,
};
use crate::fixstd::builtin::InlineLLVMIsUniqueFunctionBody;
use crate::misc::{Map, Set};
use crate::rc_ir::ast::{
    FuncRef, MatchArm, Path, RcExpr, RcExprNode, RcFunc, RcProgram, RcRhs, RcVar,
};
use std::collections::BTreeMap;
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

/// The provenance of a whole value: the source of each of its boxed leaves, keyed by the leaf's path
/// (the field indices from the value's root down to the leaf). A value with no boxed leaf — a scalar
/// or a fieldless aggregate — is the empty map. The value's type is the sole authority on the shape
/// (which paths are boxed leaves), so a provenance stores no aggregate structure that could disagree
/// with the type; a fieldless value therefore has exactly one representation.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Provenance(Map<Path, LeafSource>);

/// The paths of the boxed leaves of a value of type `ty` — the field indices from the root of `ty`
/// down to each boxed leaf. A closure lowers to `{funptr, capture-pointer}`, so its one boxed leaf is
/// the capture; a boxed value is a single leaf at the current path; an unboxed aggregate (struct,
/// tuple, or union) recurses into its fields (a union's variants' payloads); a fully unboxed value
/// has none. It is the single source of truth for which of a type's paths are boxed leaves.
pub fn boxed_leaf_paths(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Vec<Path> {
    fn go(ty: &Arc<TypeNode>, type_env: &TypeEnv, path: &mut Path, out: &mut Vec<Path>) {
        if ty.is_fully_unboxed(type_env) {
            return;
        }
        if ty.is_closure() {
            path.push(CLOSURE_CAPTURE_IDX as usize);
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
            go(fty, type_env, path, out);
            path.pop();
        }
    }
    let mut out = Vec::new();
    go(ty, type_env, &mut Vec::new(), &mut out);
    out
}

impl Provenance {
    /// A value with no boxed leaf (a scalar or a fieldless aggregate).
    pub fn empty() -> Provenance {
        Provenance(Map::default())
    }

    /// The singleton leaf-source `{src}`.
    pub fn leaf(src: BaseSource) -> LeafSource {
        let mut s = Set::default();
        s.insert(src);
        s
    }

    /// The source of each boxed leaf of a value of type `ty`, keyed by its path. `leaf` is called once
    /// per boxed leaf with that path, so it can describe the leaf (e.g. record `Arg(i, path)`).
    pub fn build_shape(
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        leaf: &dyn Fn(&Path) -> LeafSource,
    ) -> Provenance {
        Provenance(
            boxed_leaf_paths(ty, type_env)
                .into_iter()
                .map(|path| {
                    let ls = leaf(&path);
                    (path, ls)
                })
                .collect(),
        )
    }

    /// The provenance whose every boxed leaf is `src`.
    pub fn uniform(ty: &Arc<TypeNode>, type_env: &TypeEnv, src: BaseSource) -> Provenance {
        Provenance::build_shape(ty, type_env, &|_| Provenance::leaf(src.clone()))
    }

    /// The provenance of the result of an operation that produces one uniquely owned value among
    /// values of unknown sharing: every boxed leaf under `path` is `Fresh`, every other leaf `Dyn`.
    pub fn fresh_under(ty: &Arc<TypeNode>, type_env: &TypeEnv, path: &[usize]) -> Provenance {
        Provenance::uniform(ty, type_env, BaseSource::Dyn).set_leaves_under(path, BaseSource::Fresh)
    }

    /// The provenance whose every boxed leaf at path `π` is `Arg(arg_index, π)` — the whole value of
    /// input `arg_index` carried through unchanged.
    pub fn arg_passthrough(ty: &Arc<TypeNode>, type_env: &TypeEnv, arg_index: usize) -> Provenance {
        Provenance::build_shape(ty, type_env, &|path: &Path| {
            Provenance::leaf(BaseSource::Arg(arg_index, path.clone()))
        })
    }

    /// Pointwise join (branch merge): the leaf sources are unioned per path. Both operands have the
    /// same type, hence the same boxed-leaf paths, so every path is present on both sides; a path on
    /// only one side (not expected for same-typed operands) is carried through unchanged.
    fn join(&self, other: &Provenance) -> Provenance {
        let mut m = self.0.clone();
        for (path, ls) in &other.0 {
            m.entry(path.clone())
                .and_modify(|acc| *acc = acc.union(ls).cloned().collect())
                .or_insert_with(|| ls.clone());
        }
        Provenance(m)
    }

    /// The leaf source recorded for the boxed leaf at path `π`, or the empty set when `π` is not a
    /// boxed leaf of this value — a scalar, or an aggregate queried at a non-leaf path such as its
    /// root `[]` (which `root` does to test whether the whole value is a single boxed leaf). The empty
    /// set is the bottom of the lattice, so an absent leaf resolves to `Unique`, matching a recorded
    /// `⊥`.
    pub fn leaf_at(&self, path: &[usize]) -> LeafSource {
        self.0.get(path).cloned().unwrap_or_default()
    }

    /// The source of every boxed leaf, in no particular order (the paths are not reported).
    pub fn leaves(&self) -> impl Iterator<Item = &LeafSource> {
        self.0.values()
    }

    /// Substitute the `Arg(j, σ)` symbols of a declared provenance with the operands' provenance:
    /// `Arg(j, σ)` becomes operand `j`'s leaf source at `σ`. `Fresh`/`Dyn` stay. Used to compose a
    /// primitive's declared `result_prov` (and, later, a callee's effect) with its actual operands.
    fn compose(&self, operand_provs: &[Provenance]) -> Provenance {
        let mut m = Map::default();
        for (path, ls) in &self.0 {
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
            m.insert(path.clone(), out);
        }
        Provenance(m)
    }

    /// The provenance of field `i`: the leaves whose path descends through field `i`, with that head
    /// index stripped. A boxed value or a scalar has no such leaf, so it projects to the empty value.
    fn project(&self, i: usize) -> Provenance {
        let mut m = Map::default();
        for (path, ls) in &self.0 {
            if let Some((head, rest)) = path.split_first() {
                if *head == i {
                    m.insert(rest.to_vec(), ls.clone());
                }
            }
        }
        Provenance(m)
    }

    /// Give every boxed leaf under `path` the source `src`. An empty path covers the whole value.
    fn set_leaves_under(&self, path: &[usize], src: BaseSource) -> Provenance {
        let mut m = self.0.clone();
        for (leaf_path, ls) in m.iter_mut() {
            if leaf_path.starts_with(path) {
                *ls = Provenance::leaf(src.clone());
            }
        }
        Provenance(m)
    }

    /// Demote every boxed leaf under `path` to `Dyn` (the effect of duplicating the reference with a
    /// `Retain`). An empty path demotes the whole value.
    fn demote(&self, path: &[usize]) -> Provenance {
        self.set_leaves_under(path, BaseSource::Dyn)
    }

    /// A readable one-line rendering, for the RC IR dump: a value with no boxed leaf as `unboxed`, a
    /// boxed value (one leaf at the root) as its source, and anything else as its leaves rendered
    /// `π=source`, sorted by path, inside braces.
    pub fn to_string(&self) -> String {
        if self.0.is_empty() {
            return "unboxed".to_string();
        }
        if self.0.len() == 1 {
            let (path, ls) = self.0.iter().next().unwrap();
            if path.is_empty() {
                return leaf_source_to_string(ls);
            }
        }
        let mut entries: Vec<(&Path, &LeafSource)> = self.0.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        let inner = entries
            .iter()
            .map(|(path, ls)| {
                let p = path.iter().map(|i| format!(".{}", i)).collect::<String>();
                format!("{}={}", p, leaf_source_to_string(ls))
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", inner)
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

/// The resolved uniqueness of a whole value: the `CTRefCnt` verdict of each of its boxed leaves,
/// keyed by path (mirroring `Provenance`, with a `CTRefCnt` in place of each leaf source). A
/// specialization key is a function's parameters' uniqueness, so it is `Hash`; the `BTreeMap` orders
/// the paths canonically, giving equal shapes an identical hash and comparison.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Uniqueness(BTreeMap<Path, CTRefCnt>);

impl Uniqueness {
    /// The `CTRefCnt` of the boxed leaf at path `π`. `π` is always a boxed leaf of this shape: the only
    /// caller (`resolve_leaf`) resolves an `Arg`'s path, which addresses a boxed leaf of the input's
    /// type — so a miss is a malformed provenance, not a case to default away.
    fn leaf_at(&self, path: &[usize]) -> CTRefCnt {
        self.0.get(path).copied().unwrap_or_else(|| {
            unreachable!(
                "path {:?} is not a boxed leaf of the uniqueness shape",
                path
            )
        })
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
            // `Arg(i, path)` names input `i`, which `resolve` always supplies (a parameter per key,
            // plus the capture), so the index is in range; an out-of-range index is malformed.
            BaseSource::Arg(i, path) => inputs
                .get(*i)
                .unwrap_or_else(|| {
                    unreachable!(
                        "Arg names input {} but resolve was given {} inputs",
                        i,
                        inputs.len()
                    )
                })
                .leaf_at(path),
        };
        if rc == CTRefCnt::Dynamic {
            return CTRefCnt::Dynamic;
        }
    }
    CTRefCnt::Unique
}

/// Resolve a provenance against the uniqueness of its function's inputs, mapping each boxed leaf to
/// its `CTRefCnt` verdict. `inputs` must give the uniqueness of every input the provenance's `Arg`
/// leaves name — a parameter per index, plus the capture past them (an unspecialized function's are
/// all `Dynamic`). A provenance with no `Arg` leaf (e.g. an all-`Dyn` one) references none, so it
/// needs no inputs.
///
/// A value with no boxed leaf resolves to the empty map, so a specialization key built from the
/// resolved uniqueness reflects only the boxed leaves and ignores how the unboxed structure nests.
pub fn resolve(prov: &Provenance, inputs: &[Uniqueness]) -> Uniqueness {
    Uniqueness(
        prov.0
            .iter()
            .map(|(path, ls)| (path.clone(), resolve_leaf(ls, inputs)))
            .collect(),
    )
}

/// Whether the boxed leaf at path `π` of a value with this provenance is statically `Unique`, given
/// its function's input uniqueness.
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
    /// The result of each `is_unique`, mapped to the path of the leaf whose reference count it tested,
    /// so that destructuring the result can pair the flag it returns with the value it returns.
    is_unique_results: Map<FullName, Path>,
    /// A variable holding an `is_unique` flag, mapped to the value the same operation returned and the
    /// path of the tested leaf. The arm a `match` on that flag takes for `true` runs where the leaf's
    /// reference count was one, so that arm may read the leaf as uniquely owned.
    unique_flags: Map<FullName, (RcVar, Path)>,
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
            is_unique_results: Map::default(),
            unique_flags: Map::default(),
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
                // A retain is how a second reference to a value comes into existence, so it ends what
                // an `is_unique` established: the count it read is no longer the count that holds. The
                // retain may name an alias of the tested value rather than the value itself, and
                // telling that apart needs the object identity the borrow pass computes, so every
                // pending flag goes — the refinement `refine_by_condition` performs would otherwise
                // restore a value the retain had just shared.
                self.is_unique_results.clear();
                self.unique_flags.clear();
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
                self.note_unique_flag(container, fields);
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
                    // `is_unique` is the one operation that answers a uniqueness question rather than
                    // acting on it, so its result is what makes a branch's condition readable as a
                    // fact about a value (`interp_match` refines the `true` arm with it).
                    if gen.as_any().is::<InlineLLVMIsUniqueFunctionBody>() {
                        self.is_unique_results
                            .insert(result.name.clone(), uc.path.clone());
                    }
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

    /// Record that a `Destructure` takes apart an `is_unique` result, pairing the flag it returns with
    /// the value it returns so that a `match` on that flag can refine the value in its `true` arm.
    ///
    /// The pairing holds only where the tested leaf is a boxed leaf of the value: `is_unique` reports
    /// an unboxed value as unique unconditionally, which says nothing about the boxed leaves inside it.
    /// A result reached any other way (projected field by field, say) is simply not paired, and the
    /// refinement is skipped.
    fn note_unique_flag(&mut self, container: &RcVar, fields: &[(usize, RcVar)]) {
        let Some(path) = self.is_unique_results.get(&container.name).cloned() else {
            return;
        };
        let field = |i: usize| fields.iter().find(|(idx, _)| *idx == i).map(|(_, v)| v);
        let (Some(flag), Some(value)) = (field(IS_UNIQUE_FLAG_FIELD), field(IS_UNIQUE_VALUE_FIELD))
        else {
            return;
        };
        if !boxed_leaf_paths(&value.ty, self.type_env).contains(&path) {
            return;
        }
        self.unique_flags
            .insert(flag.name.clone(), (value.clone(), path));
    }

    /// The environment an arm runs under: the one before the branch, refined where the branch's
    /// condition is an `is_unique` flag and the arm is its `true` one. Reaching that arm means the
    /// tested leaf's reference count was one — no other holder exists — so the arm may read the leaf as
    /// uniquely owned however little the analysis knows about where the value came from. A `Retain`
    /// inside the arm demotes it again, as it does any other leaf.
    fn refine_by_condition(
        &self,
        scrut: &RcVar,
        arm: &MatchArm,
        env: &Map<FullName, Provenance>,
    ) -> Map<FullName, Provenance> {
        let mut env = env.clone();
        if arm.variant != Some(BOOL_TRUE_TAG) {
            return env;
        }
        let Some((value, path)) = self.unique_flags.get(&scrut.name) else {
            return env;
        };
        // The flag and the value are bound by one `Destructure` upstream of this `match`, so the value
        // is in scope here.
        let refined = env
            .get(&value.name)
            .unwrap_or_else(|| {
                unreachable!(
                    "the value paired with an is_unique flag is unbound at the match on that flag: {}",
                    value.name.to_string()
                )
            })
            .set_leaves_under(path, BaseSource::Fresh);
        env.insert(value.name.clone(), refined);
        env
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
            let mut arm_env = self.refine_by_condition(scrut, arm, env);
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
        // A match has at least one arm (an `if` lowers to two, a union match to one per variant), so
        // the fold ran; an empty match would leave the result shape-less, which code generation also
        // rejects.
        (
            joined_result.unwrap_or_else(|| unreachable!("a match has at least one arm")),
            joined_env.unwrap_or_else(|| unreachable!("a match has at least one arm")),
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

    // A single boxed value's provenance: one leaf at the root path.
    fn boxed(src: BaseSource) -> Provenance {
        let mut m = Map::default();
        m.insert(vec![], Provenance::leaf(src));
        Provenance(m)
    }
    fn fresh() -> Provenance {
        boxed(BaseSource::Fresh)
    }
    fn dyn_() -> Provenance {
        boxed(BaseSource::Dyn)
    }
    fn arg(i: usize, path: Vec<usize>) -> Provenance {
        boxed(BaseSource::Arg(i, path))
    }
    // An unboxed aggregate: each child's leaves keyed under the child's field index.
    fn agg(children: Vec<Provenance>) -> Provenance {
        let mut m = Map::default();
        for (i, child) in children.into_iter().enumerate() {
            for (path, ls) in child.0 {
                let mut p = vec![i];
                p.extend(path);
                m.insert(p, ls);
            }
        }
        Provenance(m)
    }
    // A resolved uniqueness from its `(path, verdict)` leaves.
    fn uniq(leaves: Vec<(Vec<usize>, CTRefCnt)>) -> Uniqueness {
        Uniqueness(leaves.into_iter().collect())
    }

    #[test]
    fn join_unions_leaf_sources() {
        let ls = fresh().join(&dyn_()).leaf_at(&[]);
        assert!(ls.contains(&BaseSource::Fresh));
        assert!(ls.contains(&BaseSource::Dyn));
        assert_eq!(ls.len(), 2);
    }

    #[test]
    fn demote_only_the_named_leaf() {
        // `(fresh, fresh)`, demoting child 0, leaves child 1 fresh.
        let a = agg(vec![fresh(), fresh()]);
        assert_eq!(a.demote(&[0]), agg(vec![dyn_(), fresh()]));
    }

    #[test]
    fn demote_empty_path_demotes_whole_value() {
        let a = agg(vec![fresh(), arg(0, vec![])]);
        assert_eq!(a.demote(&[]), agg(vec![dyn_(), dyn_()]));
    }

    #[test]
    fn compose_substitutes_arg_with_operand_leaf() {
        // A declaration `arg0` composed with operand 0 = `fresh` yields `fresh`.
        assert_eq!(arg(0, vec![]).compose(&[fresh()]), fresh());
    }

    #[test]
    fn compose_substitutes_arg_through_a_subpath() {
        // A declaration `arg0.1` composed with operand 0 = `(unboxed, fresh)` yields `fresh`.
        let operand = agg(vec![Provenance::empty(), fresh()]);
        assert_eq!(arg(0, vec![1]).compose(&[operand]), fresh());
    }

    #[test]
    fn compose_keeps_fresh_and_dyn_and_unions_with_arg() {
        // `{fresh | arg1}` (a union-mod-style phi) composed with operand 1 = `dyn` yields
        // `{fresh | dyn}`.
        let mut ls = Set::default();
        ls.insert(BaseSource::Fresh);
        ls.insert(BaseSource::Arg(1, vec![]));
        let mut dm = Map::default();
        dm.insert(vec![], ls);
        let composed = Provenance(dm).compose(&[Provenance::empty(), dyn_()]);
        let out = composed.leaf_at(&[]);
        assert!(out.contains(&BaseSource::Fresh));
        assert!(out.contains(&BaseSource::Dyn));
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn leaf_at_navigates_to_the_boxed_leaf() {
        let a = agg(vec![Provenance::empty(), arg(1, vec![0])]);
        assert!(a.leaf_at(&[1]).contains(&BaseSource::Arg(1, vec![0])));
    }

    #[test]
    fn project_reads_an_aggregate_child() {
        let a = agg(vec![fresh(), dyn_()]);
        assert_eq!(a.project(0), fresh());
        assert_eq!(a.project(1), dyn_());
    }

    #[test]
    fn a_fieldless_value_is_the_empty_map() {
        // A scalar, an empty aggregate, and an all-unboxed aggregate share one representation — the
        // empty map — so there is no aggregate structure that could disagree with the type.
        assert_eq!(agg(vec![]), Provenance::empty());
        assert_eq!(
            agg(vec![Provenance::empty(), Provenance::empty()]),
            Provenance::empty()
        );
        assert_eq!(
            Provenance::empty().join(&Provenance::empty()),
            Provenance::empty()
        );
    }

    #[test]
    fn to_string_renders_leaves() {
        // A fieldless value is `unboxed`; a boxed value shows its source (a bottom leaf as `_`);
        // an aggregate's leaves are keyed by path inside braces.
        assert_eq!(Provenance::empty().to_string(), "unboxed");
        let mut bottom = Map::default();
        bottom.insert(vec![], Set::default());
        assert_eq!(Provenance(bottom).to_string(), "_");
        assert_eq!(fresh().to_string(), "fresh");
        assert_eq!(agg(vec![fresh(), dyn_()]).to_string(), "{.0=fresh, .1=dyn}");
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
    fn resolve_keys_only_the_boxed_leaves() {
        // A boxed-leaf-free value resolves to the empty map, so a uniqueness key reflects only the
        // boxed leaves and same-typed values key alike whatever their unboxed structure.
        assert_eq!(resolve(&agg(vec![]), &[]), uniq(vec![]));
        assert_eq!(
            resolve(&agg(vec![Provenance::empty(), Provenance::empty()]), &[]),
            uniq(vec![])
        );
        // A value with a boxed leaf keeps that leaf's verdict at its path.
        assert_eq!(
            resolve(&agg(vec![Provenance::empty(), fresh()]), &[]),
            uniq(vec![(vec![1], CTRefCnt::Unique)])
        );
    }
}
