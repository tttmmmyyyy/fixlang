//! Locality inference on the RC IR: which reference-counting operations act on an object that is
//! certainly in the `REFCNT_STATE_LOCAL` state, so that the operation can drop the runtime state
//! dispatch and increment or decrement the count directly.
//!
//! An object leaves the local state through exactly three doors: reading a global (whose initializer
//! marks its whole result graph global), `Std::mark_threaded`, and `Std::boxed_from_retained_ptr`.
//! Everything else — allocating, updating in place, cloning a shared container — leaves the state
//! byte alone. A forward may-analysis over the value flow therefore decides the question, provided
//! it distinguishes two facts about a value, because reference counting is *shallow*: a retain
//! touches only the root object, a release recurses into children only at zero and through a
//! dispatching traverser, and `is_unique` reads only the root. So `DeepLocal ⊑ RootLocal ⊑ MayExt`:
//! the root fact is what an annotation needs, and the deep fact is what reading out of a container
//! needs.
//!
//! Two layers carry those facts, mirroring the provenance / uniqueness pair. The symbolic layer
//! (`ExtCond`, `LeafCond`, `ExtShape`) states, for a function or an op, the condition on its inputs
//! under which a result leaf is non-local; it is what a summary and an op's `locality_flow` hold. The
//! resolved layer (`Locality`, `LocalityKey`) is what a condition becomes once the inputs are
//! concrete — inside a clone specialized on them.

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::misc::{grow_stack, Map, Set};
use crate::rc_ir::ast::{
    FieldPath, FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs,
    RcState, RcVar,
};
use crate::rc_ir::provenance::boxed_leaf_paths;
use crate::rc_ir::specialize::{specializable_callee, CloneRegistry};
use std::collections::BTreeMap;
use std::sync::Arc;

/// What is proved about one boxed leaf, once the enclosing function's inputs are concrete. A
/// three-point chain `DeepLocal ⊑ RootLocal ⊑ MayExt`, joined towards `MayExt`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Locality {
    /// The object this leaf points to, and every object reachable from it, are `REFCNT_STATE_LOCAL`.
    DeepLocal,
    /// The object this leaf points to is `REFCNT_STATE_LOCAL`. Nothing is claimed about what it
    /// reaches.
    RootLocal,
    /// Nothing is proved.
    MayExt,
}

impl Locality {
    /// Join: what is proved of a value that is either of two, which is what both prove.
    pub fn join(self, other: Locality) -> Locality {
        match (self, other) {
            (Locality::MayExt, _) | (_, Locality::MayExt) => Locality::MayExt,
            (Locality::RootLocal, _) | (_, Locality::RootLocal) => Locality::RootLocal,
            (Locality::DeepLocal, Locality::DeepLocal) => Locality::DeepLocal,
        }
    }

    /// The annotation a reference-counting node covering leaves of this locality carries.
    pub fn annotation(self) -> RcState {
        match self {
            Locality::DeepLocal => RcState::DeepLocal,
            Locality::RootLocal => RcState::Local,
            Locality::MayExt => RcState::Unknown,
        }
    }
}

/// Which of an input leaf's two facts an atom tests. The atom carries it, because the take-out rule
/// moves a container's *deep* condition into the result's *root* field: an atom's meaning has to
/// survive that move, so it cannot be read off the field the condition sits in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Aspect {
    /// The input leaf is `MayExt` — its own object may be non-local.
    Root,
    /// The input leaf is not `DeepLocal` — something it reaches may be non-local.
    Deep,
}

/// One test on an input: input `input`'s boxed leaf at `path`, read through `aspect`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Atom {
    /// The index of the input: a parameter, then the capture past them, in a summary; an operand
    /// slot in an op's `locality_flow`.
    pub input: usize,
    pub path: FieldPath,
    pub aspect: Aspect,
}

/// A condition under which something is non-local. The lattice of conditions is ordered by
/// implication: the bottom `IfAny(∅)` never holds (local unconditionally), the top `Always` always
/// does.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExtCond {
    /// Holds whatever the inputs are, and absorbs: `Always ⊔ c = Always`. It is a variant of its own
    /// so that the absorption holds by the shape of the value — the specialization gate asks whether
    /// a condition mentions any input at all, and a top expressed as an atom set would answer with
    /// the inputs it happened to list beside it.
    Always,
    /// Holds when any listed atom holds. The empty set is the bottom.
    IfAny(Set<Atom>),
}

impl ExtCond {
    /// The bottom: never holds, whatever the inputs.
    pub fn bottom() -> ExtCond {
        ExtCond::IfAny(Set::default())
    }

    /// The condition holding exactly when input `input`'s leaf at `path`, read through `aspect`, is
    /// non-local.
    pub fn atom(input: usize, path: FieldPath, aspect: Aspect) -> ExtCond {
        let mut s = Set::default();
        s.insert(Atom {
            input,
            path,
            aspect,
        });
        ExtCond::IfAny(s)
    }

    /// Join (a branch merge, or a value built from several others): the condition holding when
    /// either does. `Always` absorbs.
    pub fn join(&self, other: &ExtCond) -> ExtCond {
        match (self, other) {
            (ExtCond::Always, _) | (_, ExtCond::Always) => ExtCond::Always,
            (ExtCond::IfAny(a), ExtCond::IfAny(b)) => ExtCond::IfAny(a.union(b).cloned().collect()),
        }
    }

    /// Whether this condition's answer depends on the inputs — the specialization gate's question.
    /// `Always` does not: it holds under every key.
    pub fn depends_on_inputs(&self) -> bool {
        match self {
            ExtCond::Always => false,
            ExtCond::IfAny(atoms) => !atoms.is_empty(),
        }
    }

    /// Substitute the operands' conditions for the atoms, moving a declared condition from the atom
    /// space of its declaration (an op's operand slots, or a callee's inputs) into that of the
    /// caller. Used both to compose an op's `locality_flow` with its operands and to compose a
    /// callee's summary with a call's arguments.
    pub fn substitute(&self, operands: &[ExtShape]) -> ExtCond {
        let ExtCond::IfAny(atoms) = self else {
            return ExtCond::Always;
        };
        let mut out = ExtCond::bottom();
        for atom in atoms {
            // Every atom names an operand: an op declares only its own argument slots, and a call
            // supplies one operand per input of the callee's summary (the capture included).
            let operand = operands.get(atom.input).unwrap_or_else(|| {
                unreachable!(
                    "a declaration names input {} but {} operands were supplied",
                    atom.input,
                    operands.len()
                )
            });
            out = out.join(operand.leaf_at(&atom.path).aspect(atom.aspect));
        }
        out
    }

    /// Whether this condition holds for the given concrete inputs.
    pub fn holds(&self, inputs: &[LocalityKey]) -> bool {
        let ExtCond::IfAny(atoms) = self else {
            return true;
        };
        atoms.iter().any(|atom| {
            // As in `substitute`, every atom names a supplied input.
            let input = inputs.get(atom.input).unwrap_or_else(|| {
                unreachable!(
                    "an atom names input {} but {} inputs were supplied",
                    atom.input,
                    inputs.len()
                )
            });
            let locality = input.at(&atom.path);
            match atom.aspect {
                Aspect::Root => locality == Locality::MayExt,
                Aspect::Deep => locality != Locality::DeepLocal,
            }
        })
    }
}

/// The symbolic value of one boxed leaf: the condition under which its own object is non-local, and
/// the condition under which something it reaches is.
///
/// The invariant `root ⊑ deep` — a non-local root is something the leaf reaches — is established by
/// the constructor, which joins `root` into `deep`. It leaves the lattice with a single order
/// (subset), and makes `root ⊔ deep` equal `deep`, so a reader asking only "is anything down there
/// non-local" reads `deep` alone.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LeafCond {
    pub root: ExtCond,
    pub deep: ExtCond,
}

impl LeafCond {
    /// The leaf condition with the given root condition and a deep condition covering it.
    pub fn new(root: ExtCond, deep: ExtCond) -> LeafCond {
        let deep = deep.join(&root);
        LeafCond { root, deep }
    }

    /// Proved local, root and all, whatever the inputs.
    pub fn bottom() -> LeafCond {
        LeafCond::new(ExtCond::bottom(), ExtCond::bottom())
    }

    /// Nothing proved, whatever the inputs.
    pub fn always() -> LeafCond {
        LeafCond::new(ExtCond::Always, ExtCond::Always)
    }

    /// Input `input`'s own leaf at `path`, carried through unchanged.
    pub fn input_leaf(input: usize, path: FieldPath) -> LeafCond {
        LeafCond::new(
            ExtCond::atom(input, path.clone(), Aspect::Root),
            ExtCond::atom(input, path, Aspect::Deep),
        )
    }

    /// The value read out of a boxed container whose leaf is `container`. Reading out of a container
    /// promotes the container's deep fact to the result's root fact: a `RootLocal` container says
    /// nothing about its contents, and a `DeepLocal` one hands out a `DeepLocal` value.
    pub fn take_out_of(container: &LeafCond) -> LeafCond {
        LeafCond::new(container.deep.clone(), container.deep.clone())
    }

    /// The condition this leaf offers for the given aspect.
    pub fn aspect(&self, aspect: Aspect) -> &ExtCond {
        match aspect {
            Aspect::Root => &self.root,
            Aspect::Deep => &self.deep,
        }
    }

    pub fn join(&self, other: &LeafCond) -> LeafCond {
        LeafCond::new(self.root.join(&other.root), self.deep.join(&other.deep))
    }

    pub fn substitute(&self, operands: &[ExtShape]) -> LeafCond {
        LeafCond::new(
            self.root.substitute(operands),
            self.deep.substitute(operands),
        )
    }

    /// What this leaf resolves to for concrete inputs.
    pub fn resolve(&self, inputs: &[LocalityKey]) -> Locality {
        if self.root.holds(inputs) {
            Locality::MayExt
        } else if self.deep.holds(inputs) {
            Locality::RootLocal
        } else {
            Locality::DeepLocal
        }
    }

    /// Whether either condition's answer depends on the inputs.
    pub fn depends_on_inputs(&self) -> bool {
        self.root.depends_on_inputs() || self.deep.depends_on_inputs()
    }
}

/// The symbolic value of a whole value: the condition of each of its boxed leaves, keyed by path.
/// A value with no boxed leaf is the empty map. A function's summary and an op's `locality_flow`
/// share this type, differing only in what the atoms' input indices name.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ExtShape(Map<FieldPath, LeafCond>);

impl ExtShape {
    /// The condition of each boxed leaf of a value of type `ty`, keyed by its path. `leaf` is called
    /// once per boxed leaf, with that path, so no leaf of the type can be left out.
    pub fn build_shape(
        ty: &Arc<TypeNode>,
        type_env: &TypeEnv,
        leaf: &dyn Fn(&FieldPath) -> LeafCond,
    ) -> ExtShape {
        ExtShape(
            boxed_leaf_paths(ty, type_env)
                .into_iter()
                .map(|path| {
                    let cond = leaf(&path);
                    (path, cond)
                })
                .collect(),
        )
    }

    /// The shape whose every boxed leaf is `cond`.
    pub fn uniform(ty: &Arc<TypeNode>, type_env: &TypeEnv, cond: LeafCond) -> ExtShape {
        ExtShape::build_shape(ty, type_env, &|_| cond.clone())
    }

    /// The result of an op that produces a container the analysis knows the root state of: freshly
    /// allocated, force-uniqued by this op, or updated in place under a uniqueness the caller
    /// guarantees. Its root is local; what it reaches is whatever any operand reaches.
    ///
    /// The middle case is where the contract bites: a global object never passes a uniqueness check
    /// (`build_branch_by_is_unique` sends its global arm to the shared arm), so a force-uniqued
    /// container is local. An op that hands back an operand's object *without* that backing must
    /// not declare `merge`.
    pub fn merge(
        result_ty: &Arc<TypeNode>,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> ExtShape {
        let mut deep = ExtCond::bottom();
        for (i, arg_ty) in arg_tys.iter().enumerate() {
            for path in boxed_leaf_paths(arg_ty, type_env) {
                deep = deep.join(&ExtCond::atom(i, path, Aspect::Deep));
            }
        }
        ExtShape::uniform(result_ty, type_env, LeafCond::new(ExtCond::bottom(), deep))
    }

    /// The result of an op whose value comes from outside the analysis.
    pub fn always(result_ty: &Arc<TypeNode>, type_env: &TypeEnv) -> ExtShape {
        ExtShape::uniform(result_ty, type_env, LeafCond::always())
    }

    /// The bottom: every leaf proved local, root and all. It reads no operand, which is what an op
    /// that diverges needs — it returns no value, so any claim about one is vacuously true, and the
    /// bottom is the most precise such claim. An op that merely allocates declares `merge` instead.
    pub fn bottom(result_ty: &Arc<TypeNode>, type_env: &TypeEnv) -> ExtShape {
        ExtShape::uniform(result_ty, type_env, LeafCond::bottom())
    }

    /// The identity summary of input `input`: every boxed leaf is that input's own leaf at the same
    /// path. It is what a parameter is seeded with.
    pub fn identity(ty: &Arc<TypeNode>, type_env: &TypeEnv, input: usize) -> ExtShape {
        ExtShape::build_shape(ty, type_env, &|path| {
            LeafCond::input_leaf(input, path.clone())
        })
    }

    /// The condition of the boxed leaf at `path`. The path is always a boxed leaf of the value's
    /// type, which is the sole authority on the shape.
    pub fn leaf_at(&self, path: &[usize]) -> &LeafCond {
        self.0
            .get(path)
            .unwrap_or_else(|| unreachable!("{:?} is not a boxed leaf of this value's type", path))
    }

    /// The conditions of the boxed leaves under `path` — the leaves one reference-counting operation
    /// on that subtree touches. The empty path covers the whole value.
    pub fn leaves_under<'a>(&'a self, path: &'a [usize]) -> impl Iterator<Item = &'a LeafCond> {
        self.0
            .iter()
            .filter(move |(leaf_path, _)| leaf_path.starts_with(path))
            .map(|(_, cond)| cond)
    }

    /// Every boxed leaf's condition, in no particular order.
    pub fn leaves(&self) -> impl Iterator<Item = &LeafCond> {
        self.0.values()
    }

    /// Pointwise join. Both operands are shapes of the same type, hence of the same leaf paths.
    pub fn join(&self, other: &ExtShape) -> ExtShape {
        // Differing paths would leave the result shaped like neither operand's type, which every
        // reader of a leaf takes for granted.
        assert!(
            self.0.len() == other.0.len() && self.0.keys().all(|path| other.0.contains_key(path)),
            "joining the locality of differently shaped values"
        );
        ExtShape(
            self.0
                .iter()
                .map(|(path, cond)| (path.clone(), cond.join(&other.0[path])))
                .collect(),
        )
    }

    /// The shape of field `i` of an unboxed aggregate: the leaves whose path descends through field
    /// `i`, with that head index stripped.
    pub fn project(&self, i: usize) -> ExtShape {
        ExtShape(
            self.0
                .iter()
                .filter_map(|(path, cond)| match path.split_first() {
                    Some((head, rest)) if *head == i => Some((rest.to_vec(), cond.clone())),
                    _ => None,
                })
                .collect(),
        )
    }

    /// Substitute the operands' shapes for the atoms of every leaf.
    pub fn substitute(&self, operands: &[ExtShape]) -> ExtShape {
        ExtShape(
            self.0
                .iter()
                .map(|(path, cond)| (path.clone(), cond.substitute(operands)))
                .collect(),
        )
    }

    /// What this shape resolves to for concrete inputs.
    pub fn resolve(&self, inputs: &[LocalityKey]) -> LocalityKey {
        LocalityKey(
            self.0
                .iter()
                .map(|(path, cond)| (path.clone(), cond.resolve(inputs)))
                .collect(),
        )
    }
}

/// The resolved locality of a whole value: the `Locality` of each of its boxed leaves, keyed by path.
/// A specialization key is a function's parameters' locality, so it is `Hash`; the `BTreeMap` orders
/// the paths canonically, giving equal shapes an identical hash and comparison.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LocalityKey(BTreeMap<FieldPath, Locality>);

impl LocalityKey {
    /// The locality of the boxed leaf at `path`. As in `ExtShape::leaf_at`, the path always names a
    /// boxed leaf of the value's type.
    pub fn at(&self, path: &[usize]) -> Locality {
        self.0
            .get(path)
            .copied()
            .unwrap_or_else(|| unreachable!("{:?} is not a boxed leaf of this locality key", path))
    }

    /// The key of a value of type `ty` about which nothing is proved — what an input reached from
    /// outside the analysis (an entry point, an indirect call, a call from outside this compilation
    /// unit) carries, and the key of every function's canonical version.
    pub fn all_may_ext(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> LocalityKey {
        LocalityKey(
            boxed_leaf_paths(ty, type_env)
                .into_iter()
                .map(|path| (path, Locality::MayExt))
                .collect(),
        )
    }
}

// --- the transfer rules -------------------------------------------------------------------------

/// One forward walk of a body under the transfer rules, threading an environment that maps each
/// local binding to the symbolic locality of its value. A binding's value is decided where it is
/// bound and never changes afterwards (nothing moves a live object out of the local state in a
/// non-threaded build), so the environment only grows and a branch needs no join of exit
/// environments.
struct Walk<'a> {
    prog: &'a RcProgram,
    type_env: &'a TypeEnv,
    /// Each function's result, symbolic in its inputs, from the phase-1 fixed point.
    summaries: &'a Map<FuncRef, ExtShape>,
    env: Map<FullName, ExtShape>,
    mode: Mode<'a>,
}

/// What a walk does besides computing shapes.
enum Mode<'a> {
    /// Compute shapes, leaving the body as it is, and collect what the clone gate asks: whether some
    /// reference-counting site's answer depends on the inputs, and which direct callees this body
    /// hands an input-dependent leaf to.
    Survey {
        has_dependent_site: bool,
        dependent_calls: Vec<FuncRef>,
    },
    /// Annotate the reference-counting nodes and route the direct calls, under inputs the enclosing
    /// clone's key makes concrete.
    Clone {
        inputs: &'a [LocalityKey],
        gate: &'a Set<FuncRef>,
        clones: &'a mut CloneRegistry<Vec<LocalityKey>>,
    },
}

impl<'a> Walk<'a> {
    /// Walk a body, seeding the parameters (and the capture past them) as the identity summary of
    /// the input they are. Returns the shape of the body's value and the body, rewritten in clone
    /// mode and unchanged in survey mode.
    fn run_func(&mut self, func: &RcFunc) -> (ExtShape, RcExprNode) {
        for (i, p) in func.params.iter().enumerate() {
            let shape = ExtShape::identity(&p.ty, self.type_env, i);
            self.env.insert(p.name.clone(), shape);
        }
        if let Some(cap) = &func.capture {
            let shape = ExtShape::identity(&cap.ty, self.type_env, func.params.len());
            self.env.insert(cap.name.clone(), shape);
        }
        self.walk(&func.body)
    }

    fn walk(&mut self, node: &RcExprNode) -> (ExtShape, RcExprNode) {
        grow_stack(|| self.walk_inner(node))
    }

    fn walk_inner(&mut self, node: &RcExprNode) -> (ExtShape, RcExprNode) {
        match node.expr.as_ref() {
            RcExpr::Ret(x) => (self.shape_of(x), node.clone()),
            RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
                let (result, arms) = self.walk_match(scrut, arms);
                self.env.insert(x.name.clone(), result);
                let (shape, k) = self.walk(k);
                let rebuilt = || RcExpr::Let(x.clone(), RcRhs::Match(scrut.clone(), arms), k);
                (shape, self.rebuild(node, rebuilt))
            }
            RcExpr::Let(x, rhs, k) => {
                let (value, routed) = self.walk_rhs(x, rhs);
                self.env.insert(x.name.clone(), value);
                let (shape, k) = self.walk(k);
                let rebuilt = || {
                    let rhs = match routed {
                        Some(callee) => match rhs {
                            RcRhs::App(_, args) => RcRhs::App(callee, args.clone()),
                            _ => unreachable!("only a call is routed to another function"),
                        },
                        None => rhs.clone(),
                    };
                    RcExpr::Let(x.clone(), rhs, k)
                };
                (shape, self.rebuild(node, rebuilt))
            }
            RcExpr::Retain(v, path, _, k) => {
                let state = self.annotate_rc_site(v, path);
                let (shape, k) = self.walk(k);
                let rebuilt = || RcExpr::Retain(v.clone(), path.clone(), state, k);
                (shape, self.rebuild(node, rebuilt))
            }
            RcExpr::Release(v, path, _, k) => {
                let state = self.annotate_rc_site(v, path);
                let (shape, k) = self.walk(k);
                let rebuilt = || RcExpr::Release(v.clone(), path.clone(), state, k);
                (shape, self.rebuild(node, rebuilt))
            }
            RcExpr::Destructure(container, fields, k) => {
                self.bind_destructured_fields(container, fields);
                let (shape, k) = self.walk(k);
                let rebuilt = || RcExpr::Destructure(container.clone(), fields.clone(), k);
                (shape, self.rebuild(node, rebuilt))
            }
            RcExpr::Eval(v, k) => {
                let (shape, k) = self.walk(k);
                (shape, self.rebuild(node, || RcExpr::Eval(v.clone(), k)))
            }
        }
    }

    /// Rebuild a node around its rewritten parts in clone mode; return it untouched in survey mode,
    /// where nothing is rewritten and the parts are discarded.
    fn rebuild(&self, node: &RcExprNode, build: impl FnOnce() -> RcExpr) -> RcExprNode {
        match self.mode {
            Mode::Survey { .. } => node.clone(),
            Mode::Clone { .. } => RcExprNode {
                expr: Arc::new(build()),
                source: node.source.clone(),
            },
        }
    }

    /// The symbolic locality of an operand. A name the environment does not hold is a global symbol,
    /// whose graph its initializer marked global — the first of the three doors, and the only rule
    /// that reads whether a name is local. It has to sit here, at the one place an operand is
    /// resolved, because a global reaches every operand position: the right-hand side of a `let`, an
    /// argument, a scrutinee, a destructured container, and — after borrow-ification introduces a
    /// release for a value the callee borrows — the target of a `Release`.
    fn shape_of(&self, var: &RcVar) -> ExtShape {
        match self.env.get(&var.name) {
            Some(shape) => shape.clone(),
            None => ExtShape::always(&var.ty, self.type_env),
        }
    }

    /// The shape of a `let`'s right-hand side (`Match` excepted, which the caller handles for the
    /// payload bindings its arms make), and the callee a call is routed to. Only a call is ever
    /// rewritten, so every other right-hand side answers `None` and the caller carries it over.
    fn walk_rhs(&mut self, result: &RcVar, rhs: &RcRhs) -> (ExtShape, Option<RcVar>) {
        match rhs {
            RcRhs::Var(y) => (self.shape_of(y), None),
            RcRhs::Llvm(llvm_gen, args) => {
                let arg_shapes: Vec<ExtShape> = args.iter().map(|a| self.shape_of(a)).collect();
                let arg_tys: Vec<Arc<TypeNode>> = args.iter().map(|a| a.ty.clone()).collect();
                let declared = llvm_gen.locality_flow(&result.ty, &arg_tys, self.type_env);
                (declared.substitute(&arg_shapes), None)
            }
            RcRhs::Closure(_, caps) => {
                // `{funptr, capture}`: the capture object is freshly allocated, so its root is
                // local, and it holds the captured values.
                let mut deep = ExtCond::bottom();
                for cap in caps {
                    for leaf in self.shape_of(cap).leaves() {
                        deep = deep.join(&leaf.deep);
                    }
                }
                let shape = ExtShape::uniform(
                    &result.ty,
                    self.type_env,
                    LeafCond::new(ExtCond::bottom(), deep),
                );
                (shape, None)
            }
            RcRhs::App(callee, args) => self.walk_app(result, callee, args),
            RcRhs::Match(..) => {
                unreachable!("a Match rhs is handled by walk_inner for the bindings its arms make")
            }
        }
    }

    /// The shape of a call's result, and the callee it is routed to. A call whose callee is not a
    /// function of this program — a closure value, or a function of another compilation unit — tells
    /// nothing about its result and is routed nowhere.
    fn walk_app(
        &mut self,
        result: &RcVar,
        callee: &RcVar,
        args: &[RcVar],
    ) -> (ExtShape, Option<RcVar>) {
        let arg_shapes: Vec<ExtShape> = args.iter().map(|a| self.shape_of(a)).collect();
        let cref = FuncRef {
            name: callee.name.clone(),
        };
        let Some(g) = self.prog.funcs.get(&cref) else {
            return (ExtShape::always(&result.ty, self.type_env), None);
        };
        // Code generation requires every call to supply one argument per parameter.
        assert_eq!(
            arg_shapes.len(),
            g.params.len(),
            "call to `{}` supplies {} arguments to {} parameters",
            g.name.name.to_string(),
            arg_shapes.len(),
            g.params.len()
        );
        // A summary is symbolic in the capture as well as the parameters, and a direct call passes
        // no capture, so an atom naming it resolves to `Always`.
        let mut operands = arg_shapes.clone();
        if let Some(cap) = &g.capture {
            operands.push(ExtShape::always(&cap.ty, self.type_env));
        }
        let shape = self.summaries[&cref].substitute(&operands);
        (shape, self.route_call(callee, &cref, &arg_shapes))
    }

    /// Route a direct call to the clone for the argument locality it passes, requesting that clone.
    /// In survey mode nothing is routed; the call is recorded instead when it hands the callee a
    /// leaf whose locality depends on this body's own inputs, which is what makes cloning this body
    /// worthwhile even where it counts no reference itself.
    fn route_call(
        &mut self,
        callee: &RcVar,
        cref: &FuncRef,
        arg_shapes: &[ExtShape],
    ) -> Option<RcVar> {
        let depends = arg_shapes
            .iter()
            .flat_map(|s| s.leaves())
            .any(|leaf| leaf.depends_on_inputs());
        match &mut self.mode {
            Mode::Survey {
                dependent_calls, ..
            } => {
                // A callee taking a capture is never routed anywhere (see `specializable_callee`),
                // so passing it an input-dependent leaf is no reason to clone this body.
                if depends && self.prog.funcs[cref].capture.is_none() {
                    dependent_calls.push(cref.clone());
                }
                None
            }
            Mode::Clone {
                inputs,
                gate,
                clones,
            } => {
                specializable_callee(self.prog, callee, gate)?;
                let g = &self.prog.funcs[cref];
                let key: Vec<LocalityKey> = arg_shapes.iter().map(|s| s.resolve(inputs)).collect();
                let is_canonical = key == canonical_key(g, self.type_env);
                let name = clones.request(cref, key, is_canonical);
                let mut c = callee.clone();
                c.name = name.name;
                Some(c)
            }
        }
    }

    /// The annotation of a reference-counting node on the subtree of `v` at `path`, and, in survey
    /// mode, the record that the node's answer depends on the inputs. The node covers every boxed
    /// leaf under the path — several of them where the path stops at an unboxed union — and may drop
    /// the state dispatch only where all of them are proved local, so the leaves' localities join.
    fn annotate_rc_site(&mut self, v: &RcVar, path: &FieldPath) -> RcState {
        let shape = self.shape_of(v);
        match &mut self.mode {
            Mode::Survey {
                has_dependent_site, ..
            } => {
                if shape
                    .leaves_under(path)
                    .any(|leaf| leaf.depends_on_inputs())
                {
                    *has_dependent_site = true;
                }
                RcState::Unknown
            }
            Mode::Clone { inputs, .. } => shape
                .leaves_under(path)
                .map(|leaf| leaf.resolve(inputs))
                .fold(Locality::DeepLocal, Locality::join)
                .annotation(),
        }
    }

    /// Bind the fields a `Destructure` names. Out of a boxed container each field is read by the
    /// take-out rule; out of an unboxed one it is the projection of the container's own leaves.
    fn bind_destructured_fields(&mut self, container: &RcVar, fields: &[(usize, RcVar)]) {
        let container_shape = self.shape_of(container);
        let boxed = container.ty.is_box(self.type_env);
        for (idx, fv) in fields {
            let field_shape = if boxed {
                let cond = LeafCond::take_out_of(container_shape.leaf_at(&[]));
                ExtShape::uniform(&fv.ty, self.type_env, cond)
            } else {
                container_shape.project(*idx)
            };
            self.env.insert(fv.name.clone(), field_shape);
        }
    }

    /// Walk a `match`: bind each arm's payload, walk the arm, and join the arms' values. A variant
    /// arm of a boxed union reads its payload out of the container by the take-out rule; of an
    /// unboxed union it projects the scrutinee's variant. A catch-all arm binds the scrutinee itself.
    fn walk_match(&mut self, scrut: &RcVar, arms: &[MatchArm]) -> (ExtShape, Vec<MatchArm>) {
        let scrut_shape = self.shape_of(scrut);
        let boxed = scrut.ty.is_box(self.type_env);
        let mut joined: Option<ExtShape> = None;
        let mut out = Vec::with_capacity(arms.len());
        for arm in arms {
            let payload_shape = match arm.tag {
                Some(tag) => {
                    if boxed {
                        let cond = LeafCond::take_out_of(scrut_shape.leaf_at(&[]));
                        ExtShape::uniform(&arm.payload.ty, self.type_env, cond)
                    } else {
                        scrut_shape.project(tag)
                    }
                }
                None => scrut_shape.clone(),
            };
            self.env.insert(arm.payload.name.clone(), payload_shape);
            let (arm_shape, body) = self.walk(&arm.body);
            joined = Some(match joined {
                None => arm_shape,
                Some(acc) => acc.join(&arm_shape),
            });
            out.push(MatchArm {
                tag: arm.tag,
                payload: arm.payload.clone(),
                body,
            });
        }
        // A match has at least one arm (an `if` lowers to two, a union match to one per variant).
        let joined = joined.unwrap_or_else(|| unreachable!("a match has at least one arm"));
        (joined, out)
    }
}

/// The key of a function's canonical clone: nothing proved about any input.
fn canonical_key(func: &RcFunc, type_env: &TypeEnv) -> Vec<LocalityKey> {
    func.params
        .iter()
        .map(|p| LocalityKey::all_may_ext(&p.ty, type_env))
        .collect()
}

/// The locality of every input of the clone `(func, key)`: the key gives the parameters, and a
/// closure capture (the input past them) proves nothing, since closures are not specialized.
fn input_localities(func: &RcFunc, key: &[LocalityKey], type_env: &TypeEnv) -> Vec<LocalityKey> {
    let mut inputs = key.to_vec();
    if let Some(cap) = &func.capture {
        inputs.push(LocalityKey::all_may_ext(&cap.ty, type_env));
    }
    inputs
}

/// A walk that computes shapes and collects the clone gate's material, leaving the body as it is.
fn survey<'a>(
    prog: &'a RcProgram,
    type_env: &'a TypeEnv,
    summaries: &'a Map<FuncRef, ExtShape>,
) -> Walk<'a> {
    Walk {
        prog,
        type_env,
        summaries,
        env: Map::default(),
        mode: Mode::Survey {
            has_dependent_site: false,
            dependent_calls: vec![],
        },
    }
}

// --- phase 1: the symbolic summaries ------------------------------------------------------------

/// Each function's result, symbolic in its inputs, computed to a fixed point. A direct call
/// substitutes the callee's summary, so recursion needs iteration; the lattice is finite and the
/// join is monotone, so it converges.
///
/// Starting from the bottom is what makes it precise. Soundness comes from the result being a
/// post-fixed point, which the top would also give, but a recursive function started there would
/// feed its own unproven answer back into itself and settle on `Always`.
fn summarize(prog: &RcProgram, type_env: &TypeEnv) -> Map<FuncRef, ExtShape> {
    let mut summaries: Map<FuncRef, ExtShape> = prog
        .funcs
        .values()
        .map(|f| (f.name.clone(), ExtShape::bottom(&f.ret_ty, type_env)))
        .collect();
    loop {
        let mut changed = false;
        let mut next = summaries.clone();
        for func in prog.funcs.values() {
            let (result, _) = survey(prog, type_env, &summaries).run_func(func);
            let merged = summaries[&func.name].join(&result);
            if merged != summaries[&func.name] {
                next.insert(func.name.clone(), merged);
                changed = true;
            }
        }
        summaries = next;
        if !changed {
            return summaries;
        }
    }
}

/// A walk that annotates the reference-counting nodes and routes the direct calls, under inputs the
/// enclosing clone's key made concrete.
fn annotating<'a>(
    prog: &'a RcProgram,
    type_env: &'a TypeEnv,
    summaries: &'a Map<FuncRef, ExtShape>,
    gate: &'a Set<FuncRef>,
    inputs: &'a [LocalityKey],
    clones: &'a mut CloneRegistry<Vec<LocalityKey>>,
) -> Walk<'a> {
    Walk {
        prog,
        type_env,
        summaries,
        env: Map::default(),
        mode: Mode::Clone {
            inputs,
            gate,
            clones,
        },
    }
}

// --- phase 2: cloning by input locality ---------------------------------------------------------

/// The functions worth cloning: those whose own reference-counting sites are annotated differently
/// under different input localities, and those that hand an input-dependent leaf to such a function
/// through a direct call. A least fixed point over the direct-call graph, as the uniqueness gate is.
///
/// Closing it transitively is what keeps a forwarding function — one that counts no reference
/// itself — from staying canonical, which would make it derive its callee's key from inputs proving
/// nothing and lose the proof for every caller that goes through it.
fn clone_gate(
    prog: &RcProgram,
    type_env: &TypeEnv,
    summaries: &Map<FuncRef, ExtShape>,
) -> Set<FuncRef> {
    let mut gated: Set<FuncRef> = Set::default();
    let mut dependent_calls: Map<FuncRef, Vec<FuncRef>> = Map::default();
    for func in prog.funcs.values() {
        let mut walk = survey(prog, type_env, summaries);
        walk.run_func(func);
        let Mode::Survey {
            has_dependent_site,
            dependent_calls: calls,
        } = walk.mode
        else {
            unreachable!("the gate sweep runs in survey mode")
        };
        if has_dependent_site {
            gated.insert(func.name.clone());
        }
        dependent_calls.insert(func.name.clone(), calls);
    }
    loop {
        let mut changed = false;
        for (fref, calls) in &dependent_calls {
            if !gated.contains(fref) && calls.iter().any(|c| gated.contains(c)) {
                gated.insert(fref.clone());
                changed = true;
            }
        }
        if !changed {
            return gated;
        }
    }
}

/// Annotate the reference-counting operations of `prog` whose target is provably local, cloning a
/// function per input locality its callers reach it with where that proof depends on the inputs.
pub fn specialize(prog: &RcProgram, type_env: &TypeEnv) -> RcProgram {
    let summaries = summarize(prog, type_env);
    let gate = clone_gate(prog, type_env, &summaries);
    let mut clones: CloneRegistry<Vec<LocalityKey>> = CloneRegistry::new("l");

    // Keep every function's canonical version, which receives every path into this compilation unit.
    for (fref, func) in prog.funcs.iter() {
        clones.request(fref, canonical_key(func, type_env), true);
    }

    // A global initializer takes no argument, so its body resolves against no inputs.
    let mut globals: Vec<RcGlobalInit> = vec![];
    for g in prog.globals.iter() {
        let init = annotating(prog, type_env, &summaries, &gate, &[], &mut clones)
            .walk(&g.init)
            .1;
        globals.push(RcGlobalInit {
            symbol: g.symbol.clone(),
            ty: g.ty.clone(),
            init,
        });
    }

    // Materialize every requested clone; each materialization may request further clones.
    let mut output_funcs: Map<FuncRef, RcFunc> = Map::default();
    while let Some((fref, key)) = clones.pop() {
        let func = prog.funcs[&fref].clone();
        let inputs = input_localities(&func, &key, type_env);
        let body = annotating(prog, type_env, &summaries, &gate, &inputs, &mut clones)
            .run_func(&func)
            .1;
        let name = clones.request(&fref, key.clone(), key == canonical_key(&func, type_env));
        let clone = clones.finish_clone(&func, name, body);
        output_funcs.insert(clone.name.clone(), clone);
    }

    RcProgram {
        funcs: output_funcs,
        globals,
        entry: prog.entry.clone(),
    }
}
