//! The RC IR data types.

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::name::{FullName, Name};
use crate::ast::types::TypeNode;
use crate::misc::{grow_stack, Map, Set};
use crate::parse::sourcefile::Span;
use serde::{Serialize, Serializer};
use std::sync::Arc;

/// A variable of the RC IR. Because a fresh name is minted at every binding, a name resolves its
/// binding uniquely, without scope tracking.
// PROOF: P1, P2, P7, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub struct RcVar {
    /// The name this variable is bound under, unique across the program.
    pub name: FullName,
    /// The type of the value bound here, always concrete (monomorphic).
    #[serde(serialize_with = "serialize_type_identity")]
    pub ty: Arc<TypeNode>,
    /// The source the value bound here is written at. `None` where no source spells it out. Left
    /// out of the serialized form; see `divide_program::debug_positions`.
    #[serde(skip)]
    pub source: Option<Span>,
    /// The source-level name this variable denotes, when it is the binding of a `let`-pattern
    /// variable, a match-arm payload, or a projected capture. Code generation emits a debug local
    /// variable under this name so a debugger can inspect it by its source name. `None` for the
    /// compiler-introduced intermediates that have no source name.
    pub debug_name: Option<Name>,
    /// Whether a reference-count operation on this value may skip the null check. Set for a non-empty
    /// capture object, the one value whose null check is worth removing: a possibly-empty capture is
    /// the null pointer, so every other capture object is checked. `false` elsewhere — an ordinary
    /// boxed value is non-null too, but it is never null-checked, so saying so here buys nothing.
    pub skip_null_check: bool,
}

/// A reference to a top-level RC IR function: a lifted lambda body, a global function, or an
/// uncurried function-pointer version.
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct FuncRef {
    /// The name the function is defined under. It is the whole of the reference: a function is
    /// identified by its name across the program.
    pub name: FullName,
}

/// A whole program: the top-level functions, the global-value initializers, and the names reached
/// from outside them. The default is the empty program, which defines nothing and is reached
/// nowhere.
#[derive(Default)]
pub struct RcProgram {
    /// The top-level functions, keyed by the name each is defined under.
    pub funcs: Map<FuncRef, RcFunc>,
    /// The initializer of each global value the program defines.
    pub globals: Vec<RcGlobalInit>,
    /// The functions and globals code generation reaches from outside this program: the entry
    /// point, the values exported as C functions, and — in a compilation unit's slice of the
    /// program — the names the unit publishes for the others (`divide_among_units`).
    /// `dead_code_elim::eliminate_unreachable` keeps what they reach and drops the rest.
    pub roots: Set<FullName>,
}

/// A top-level function. One shape uniformly represents lifted lambda bodies, global functions, and
/// uncurried funptr versions.
// PROOF: P27 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub struct RcFunc {
    /// The name this function is defined and called under, unique across the program: lowering mints
    /// a fresh one for each lambda it lifts, and a pass that copies a function appends a segment of
    /// its own.
    pub name: FuncRef,
    /// The lambda's arrow type (funptr or closure). It determines the LLVM function signature and
    /// distinguishes the funptr and closure ABIs.
    #[serde(serialize_with = "serialize_type_identity")]
    pub fn_ty: Arc<TypeNode>,
    /// The parameters. A closure-ABI function takes its single arrow argument; a funptr-ABI
    /// function takes the uncurried arguments (at least one).
    pub params: Vec<RcVar>,
    /// `Some` for the closure ABI: the trailing capture-pointer parameter, from which the body
    /// projects the captured values. `None` for the funptr ABI, which has no captures.
    pub capture: Option<RcVar>,
    /// The type of the value the body returns. For a funptr-ABI function it is the result after all
    /// of its parameters.
    #[serde(serialize_with = "serialize_type_identity")]
    pub ret_ty: Arc<TypeNode>,
    /// The body, evaluated with the parameters and the capture in scope. Its `Ret` returns the
    /// function's value.
    pub body: RcExprNode,
    /// The source the lambda this function came from was written at, which code generation records
    /// as the function's debug location. `None` where no source spells the function out. Left out
    /// of the serialized form; see `divide_program::debug_positions`.
    #[serde(skip)]
    pub source: Option<Span>,
    /// The reference-counting units this version borrows among its parameters and capture — the units
    /// it does not own, one `(parameter-name, unit-path)` each. Everything not listed is owned, so the
    /// empty set is the all-owning default. Borrow-ification writes it: an original version borrows
    /// nothing, a borrow version borrows its inferred read-only units.
    ///
    /// The empty default is correct at every stage — before borrow-ification every parameter is owned
    /// (the discipline `insert_rc` establishes), and a version that owns everything borrows nothing.
    /// `cancel` and the RC IR dump read the owned complement (via `all_owned_units`) for each call's
    /// consume sites and each parameter's ownership shape.
    #[serde(serialize_with = "serialize_sorted_var_paths")]
    pub borrowed_units: Set<VarPath>,
    /// Whether the back end is asked to inline every call of this function. Code generation writes
    /// it out as the `alwaysinline` attribute; a version cloned from a function carries it.
    pub inline_into_callers: bool,
}

/// A variable together with a path into its value. Where the path is truncated to a reference-
/// counting unit, the pair names one unit of that variable — the form the ownership tables hold.
// PROOF: P5, P6, P7, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub type VarPath = (FullName, FieldPath);

/// An RC IR expression together with its source span. An expression's value type is that of the
/// variable its final `Ret` returns, so it is read from that variable.
// PROOF: P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub struct RcExprNode {
    /// The expression this node stands for. It is shared through an `Arc`, so cloning a node is
    /// O(1). The continuation chain is thousands of nodes deep for a large body, and the simplifier
    /// clones whole bodies, so a deep-copying clone would overflow the stack.
    pub expr: Arc<RcExpr>,
    /// The source the expression is written at. `None` where no source spells it out. Left out of
    /// the serialized form; see `divide_program::debug_positions`.
    #[serde(skip)]
    pub source: Option<Span>,
}

/// The statement-nested form: `Let`, `Retain`, and `Release` each carry a continuation, and `Ret`
/// is the only terminator.
// PROOF: P1, P2, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub enum RcExpr {
    /// `let x = rhs; k`: bind the result of a compound expression to a single variable (ANF).
    Let(RcVar, RcRhs, RcExprNode),
    /// Duplicate (root refcount +1) every boxed leaf of the subtree of the variable named by the
    /// path, then continue. The empty path names the whole value.
    Retain(RcVar, FieldPath, RcState, RcExprNode),
    /// Drop (refcount -1, freeing and traversing owned children at zero) every boxed leaf of the
    /// subtree of the variable named by the path, then continue.
    Release(RcVar, FieldPath, RcState, RcExprNode),
    /// Destructure a struct/tuple container into its fields at once, then continue. Each `(index,
    /// var)` binds field `index` to `var`. The container is consumed: an unboxed container's leaves
    /// are moved into the field variables (no per-field retain) and its fields not named here are
    /// dropped; a boxed container retains each named field and releases the container. Reference-count
    /// insertion retains the container before this node iff it is used afterward — together this
    /// mirrors code generation's `get_scoped_obj` retain-if-used-later plus `get_struct_fields`
    /// whole-container extraction. One node for the whole destructure lets that retain be decided
    /// once, from the container's liveness after the destructure, and placed before the extraction.
    Destructure(RcVar, Vec<(usize, RcVar)>, RcState, RcExprNode),
    /// Force the variable's value for its effect and discard it, then continue — the RC IR form of the
    /// source `eval e0; e1`. Forcing a local is a no-op (it is already computed); forcing a global
    /// runs its call-once initializer, whose evaluation may have an effect (e.g. an `undefined`-valued
    /// global). It performs no reference-count operation itself: the variable is only observed, so a
    /// following `Release` disposes it when it is dead. A node of its own keeps a value forced for
    /// effect distinguishable from dead code.
    Eval(RcVar, RcExprNode),
    /// The sole terminator: the value of this expression (a function body or a match arm) is this
    /// variable.
    Ret(RcVar),
}

/// A path into the unboxed structure of a value: a sequence of indices, each a struct/tuple field
/// number or an unboxed-union variant number. It names a boxed leaf or a subtree; the empty path is
/// the whole value. A `Retain`/`Release` path stops at the root of an unboxed-union subtree (a
/// physical refcount operation must be tag-safe), whereas an analysis path may descend past a known
/// tag.
// PROOF: P1, P2, P7, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub type FieldPath = Vec<usize>;

/// The boxed leaf whose runtime uniqueness an inline-LLVM op branches on: which operand carries the
/// container, and the path to the leaf within that operand's value.
pub struct UniqueCheckOperand {
    /// The position, among the operation's arguments, of the operand holding the container.
    pub container_index: usize,
    /// The path from the root of that operand's value down to the checked boxed leaf.
    pub path: FieldPath,
}

/// A value an inline-LLVM operation reference-counts inside its own `generate`, named the way the
/// operation sees it. Locality inference resolves each against the operation's operands and result,
/// and annotates the operation only where all of them are local.
pub enum RcTarget {
    /// A boxed leaf of the result, under this path — an element, field or payload the operation
    /// retained on its way out of a container.
    Result(FieldPath),
    /// A boxed leaf of operand `.0`, under the path — a container the operation released.
    Operand(usize, FieldPath),
    /// What operand `.0`'s leaf at the path reaches — an element the operation overwrote or dropped,
    /// or one it retain-copied while cloning a shared container. Such a value is no variable of the
    /// IR, so the judgement is the operand leaf's deep fact.
    Contents(usize, FieldPath),
}

/// One arm of a `Match`: the variant it matches, the variable its payload is bound to, the state of
/// the payload it retains out of a boxed union, and the arm body, whose value is its final `Ret`.
/// `tag` is `Some` for a variant arm, whose payload is that variant's value; it is `None` for a
/// catch-all arm, whose payload is the whole scrutinee.
/// Code generation treats the last arm as the default case (mirroring the tag switch), so a
/// catch-all is always the final arm.
// PROOF: P1, P2, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub struct MatchArm {
    /// The variant number this arm matches, or `None` for a catch-all arm.
    pub tag: Option<usize>,
    /// The variable `body` reads the matched value through.
    pub payload: RcVar,
    /// What is known about the payload a variant arm of a boxed union retains out of the container.
    /// A catch-all arm binds the scrutinee itself and retains nothing, so its state is `Unknown`.
    pub payload_state: RcState,
    /// The expression this arm evaluates to, in the scope extended with `payload`.
    pub body: RcExprNode,
}

impl MatchArm {
    /// This arm with `body` in place of its own: it matches the same variant and binds the same
    /// payload, and evaluates to what `body` gives.
    // PROOF: P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn with_body(&self, body: RcExprNode) -> MatchArm {
        MatchArm {
            body,
            ..self.clone()
        }
    }
}

/// A compound expression. It appears only as the right-hand side of a `Let`; the arguments of `App`
/// and `Llvm` are atoms (variables).
// PROOF: P1, P2, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16, P17, P18, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Serialize)]
pub enum RcRhs {
    /// Move / rename `y := x`, consuming `x`.
    Var(RcVar),
    /// A closure call or a direct funptr call, with the callee as the first variable. Code
    /// generation dispatches on the callee type.
    App(RcVar, Vec<RcVar>),
    /// A closure value: a top-level function together with its captured variables. It lowers to an
    /// unboxed `{funptr, capture-object pointer}` pair; only the capture object is boxed (a null
    /// pointer for an empty capture).
    Closure(FuncRef, Vec<RcVar>),
    /// A built-in operation (arithmetic, projection getters, set/mod, construction, fill, literals,
    /// FFI, and so on), reusing the existing inline-LLVM generators.
    Llvm(Box<dyn LLVMGen>, Vec<RcVar>),
    /// The sole branching construct (booleans included). It always appears as the right-hand side
    /// of a `Let`.
    Match(RcVar, Vec<MatchArm>),
}

/// The reference-counting state dispatch of a `Retain` or `Release`. Lowering emits `Unknown`,
/// which is always sound; locality inference specializes it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum RcState {
    /// Read the object's refcount state at run time and dispatch three ways.
    Unknown,
    /// Known local: non-atomic increment/decrement on the object itself, no state check. What it
    /// reaches is unknown, so the traverser a release runs at zero still dispatches per child.
    Local,
    /// Known local, and so is everything reachable from it.
    DeepLocal,
    /// Known threaded: atomic increment/decrement, no state check.
    Threaded,
    /// Known global: a no-op, emitting no code.
    Global,
}

impl RcState {
    /// Whether code generation must read the object's state byte to decide how to count it.
    pub fn dispatches(self) -> bool {
        match self {
            RcState::Unknown => true,
            RcState::Local | RcState::DeepLocal => false,
            RcState::Threaded | RcState::Global => unreachable!(
                "no pass produces {:?}; code generation for it is not implemented",
                self
            ),
        }
    }

    /// The suffix a reference-counting helper generated under this state carries in its name. The
    /// helpers and traversers are memoized by name, so this is what keys one per (type, state) and
    /// gives the states that generate the same code a single definition.
    pub fn name_suffix(self) -> &'static str {
        if self.dispatches() {
            ""
        } else {
            // A `DeepLocal` release could also drop the dispatch on the objects it reaches; until
            // the stateless traverser exists it emits the `Local` code, so the two share a helper.
            "_local"
        }
    }
}

/// The ownership of a single reference-counting unit.
// PROOF: P27 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ownership {
    /// The callee receives ownership: it consumes the unit, by releasing it or by moving it into
    /// the result, and the caller retains it before the call at a non-last use.
    Own,
    /// The callee only borrows the unit: neither side performs a refcount operation.
    Borrow,
}

/// The ownership of one argument, shaped like the value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OwnershipShape {
    /// A part of the value holding no reference-counting unit.
    NoUnit,
    /// An unboxed aggregate, with the shape of each of its fields. A field is at its own field
    /// index, so a field holding no unit keeps its place as `NoUnit`.
    Fields(Vec<OwnershipShape>),
    /// A single reference-counting unit, owned or borrowed.
    Unit(Ownership),
}

/// The initializer of a global value, run once when a reader first asks for the value. The whole
/// graph the value reaches is marked global (refcount-exempt) before it is stored.
#[derive(Clone, Serialize)]
pub struct RcGlobalInit {
    /// The name the global value is defined and read under.
    pub symbol: FullName,
    /// The type of the value, always concrete (monomorphic).
    #[serde(serialize_with = "serialize_type_identity")]
    pub ty: Arc<TypeNode>,
    /// The expression computing the value.
    pub init: RcExprNode,
    /// Whether this program generates the function computing the value. A compilation unit reading
    /// a global it does not compute calls the function the unit computing it publishes, so one
    /// function computes the value however many units read it.
    pub owns_initializer: bool,
    /// Whether this program defines the storage the value is kept in and the flag saying it has
    /// been computed. A compilation unit reading a global it does not keep reads the storage and
    /// the flag the unit keeping it publishes, so one storage holds the value however many units
    /// read it.
    ///
    /// The two are apart where a unit keeps a value another computes, which is what a global one
    /// unit reads becomes: that unit keeps the value, and nothing about the storage is published,
    /// so LLVM optimizes the reads knowing every write. The initializer follows the storage only
    /// where moving it adds little code to that unit
    /// (`divide_program::MOVED_INITIALIZER_NODE_LIMIT`).
    pub owns_storage: bool,
}

/// Visit every node of `node`: the continuation chain it heads, and the body of every arm of every
/// `Match` along it.
// PROOF: P7, P8, P9, P10, P11, P12, P13, P14 (dev-docs/proof/rc_ir/borrow-cancel)
pub(crate) fn for_each_node(node: &RcExprNode, visit: &mut impl FnMut(&RcExprNode)) {
    // A deep continuation chain recurses to its full depth here; grow the stack on demand.
    grow_stack(|| for_each_node_inner(node, visit))
}

/// Call `visit` on one node, then descend into its continuation and the body of each of its arms.
// PROOF: P7, P18 (dev-docs/proof/rc_ir/borrow-cancel)
fn for_each_node_inner(node: &RcExprNode, visit: &mut impl FnMut(&RcExprNode)) {
    visit(node);
    match node.expr.as_ref() {
        RcExpr::Let(_, rhs, k) => {
            if let RcRhs::Match(_, arms) = rhs {
                for arm in arms {
                    for_each_node(&arm.body, visit);
                }
            }
            for_each_node(k, visit);
        }
        RcExpr::Retain(_, _, _, k)
        | RcExpr::Release(_, _, _, k)
        | RcExpr::Eval(_, k)
        | RcExpr::Destructure(_, _, _, k) => for_each_node(k, visit),
        RcExpr::Ret(_) => {}
    }
}

/// Visit every variable `node` binds or reads, the payload of every arm of every `Match` included.
///
/// A variable carries the type of the value bound to it, so this is also the walk over the types a
/// body is generated from.
pub(crate) fn for_each_var(node: &RcExprNode, visit: &mut impl FnMut(&RcVar)) {
    for_each_node(node, &mut |node| for_each_var_of_node(node, visit))
}

/// Visit the variables the node itself binds or reads, without following its continuation or the
/// bodies of the arms of a `Match`.
fn for_each_var_of_node(node: &RcExprNode, visit: &mut impl FnMut(&RcVar)) {
    match node.expr.as_ref() {
        RcExpr::Let(var, rhs, _) => {
            visit(var);
            for_each_var_of_rhs(rhs, visit);
        }
        RcExpr::Retain(var, _, _, _) | RcExpr::Release(var, _, _, _) | RcExpr::Eval(var, _) => {
            visit(var)
        }
        RcExpr::Destructure(var, fields, _, _) => {
            visit(var);
            for (_, field) in fields {
                visit(field);
            }
        }
        RcExpr::Ret(var) => visit(var),
    }
}

/// Visit the variables of a right-hand side: the payload variable of each arm of a `Match` among
/// them, and not the arm bodies.
fn for_each_var_of_rhs(rhs: &RcRhs, visit: &mut impl FnMut(&RcVar)) {
    match rhs {
        RcRhs::Var(var) => visit(var),
        RcRhs::App(callee, args) => {
            visit(callee);
            for arg in args {
                visit(arg);
            }
        }
        RcRhs::Closure(_, captured) => {
            for var in captured {
                visit(var);
            }
        }
        RcRhs::Llvm(_, operands) => {
            for var in operands {
                visit(var);
            }
        }
        RcRhs::Match(scrutinee, arms) => {
            visit(scrutinee);
            for arm in arms {
                visit(&arm.payload);
            }
        }
    }
}

/// Serialize a set of variable paths in the order of the paths themselves, so that a digest taken
/// over a function is the same whenever the function is.
fn serialize_sorted_var_paths<S: Serializer>(
    paths: &Set<VarPath>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut paths: Vec<&VarPath> = paths.iter().collect();
    paths.sort();
    paths.serialize(serializer)
}

/// Serialize a type by its identity — the type expression — rather than by everything the node
/// carries. Where a type was written decides no code, and it moves whenever a byte is inserted
/// before it in its file, so a digest reading it would follow an edit that changes nothing.
pub(crate) fn serialize_type_identity<S: Serializer>(
    ty: &Arc<TypeNode>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    ty.type_hash().serialize(serializer)
}
