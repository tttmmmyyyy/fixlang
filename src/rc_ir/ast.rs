//! The RC IR data types.

use crate::ast::inline_llvm::LLVMGen;
use crate::ast::name::{FullName, Name};
use crate::ast::types::TypeNode;
use crate::misc::{Map, Set};
use crate::parse::sourcefile::Span;
use std::sync::Arc;

/// A variable of the RC IR: a globally unique name together with its concrete (monomorphic) type
/// and the source span it comes from. Because a fresh name is minted at every binding, a name
/// resolves its binding uniquely, without scope tracking.
#[derive(Clone)]
pub struct RcVar {
    pub name: FullName,
    pub ty: Arc<TypeNode>,
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
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FuncRef {
    pub name: FullName,
}

/// A whole program: the top-level functions, the global-value initializers, and the entry point.
pub struct RcProgram {
    pub funcs: Map<FuncRef, RcFunc>,
    pub globals: Vec<RcGlobalInit>,
    pub entry: FuncRef,
}

/// A top-level function. One shape uniformly represents lifted lambda bodies, global functions, and
/// uncurried funptr versions.
#[derive(Clone)]
pub struct RcFunc {
    /// The name this function is defined and called under, unique across the program: lowering mints
    /// a fresh one for each lambda it lifts, and a pass that copies a function appends a segment of
    /// its own.
    pub name: FuncRef,
    /// The lambda's arrow type (funptr or closure). It determines the LLVM function signature and
    /// distinguishes the funptr and closure ABIs.
    pub fn_ty: Arc<TypeNode>,
    /// The parameters. A closure-ABI function takes its single arrow argument; a funptr-ABI
    /// function takes the uncurried arguments (at least one).
    pub params: Vec<RcVar>,
    /// `Some` for the closure ABI: the trailing capture-pointer parameter, from which the body
    /// projects the captured values. `None` for the funptr ABI, which has no captures.
    pub capture: Option<RcVar>,
    /// The type of the value the body returns, which for a funptr-ABI function is the result after
    /// all of its parameters rather than the arrow taking the rest of them.
    pub ret_ty: Arc<TypeNode>,
    /// The body, evaluated with the parameters and the capture in scope. Its `Ret` returns the
    /// function's value.
    pub body: RcExprNode,
    /// The source the lambda this function came from was written at, which code generation records
    /// as the function's debug location. `None` where no source spells the function out.
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
    pub borrowed_units: Set<VarPath>,
    /// Whether the back end is asked to inline every call of this function. Code generation writes
    /// it out as the `alwaysinline` attribute; a version cloned from a function carries it.
    pub inline_into_callers: bool,
}

/// A variable together with a path into its value. Where the path is truncated to a reference-
/// counting unit, the pair names one unit of that variable — the form the ownership tables hold.
pub type VarPath = (FullName, FieldPath);

/// An RC IR expression together with its source span. An expression's value type is that of the
/// variable its final `Ret` returns, so it is read from that variable rather than stored here.
///
/// The expression is shared through an `Arc`, so cloning a node is O(1). The continuation chain is
/// thousands of nodes deep for a large body, and the simplifier clones whole bodies, so a
/// deep-copying clone would overflow the stack.
#[derive(Clone)]
pub struct RcExprNode {
    pub expr: Arc<RcExpr>,
    pub source: Option<Span>,
}

/// The statement-nested form: `Let`, `Retain`, and `Release` each carry a continuation, and `Ret`
/// is the only terminator.
#[derive(Clone)]
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
    /// mirrors the current back end's `get_scoped_obj` retain-if-used-later plus `get_struct_fields`
    /// whole-container extraction. Representing the whole destructure as one node (rather than
    /// per-field getters) lets that retain be decided once, from the container's liveness after the
    /// destructure, and placed before the extraction.
    Destructure(RcVar, Vec<(usize, RcVar)>, RcState, RcExprNode),
    /// Force the variable's value for its effect and discard it, then continue — the RC IR form of the
    /// source `eval e0; e1`. Forcing a local is a no-op (it is already computed); forcing a global
    /// runs its call-once initializer, whose evaluation may have an effect (e.g. an `undefined`-valued
    /// global). It performs no reference-count operation itself: the variable is only observed, so a
    /// following `Release` disposes it when it is dead. A distinct node — rather than a binding whose
    /// result is unused — keeps a value forced for effect from being indistinguishable from dead code.
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
pub type FieldPath = Vec<usize>;

/// The boxed leaf whose runtime uniqueness an inline-LLVM op branches on: which operand carries the
/// container, and the path to the leaf within that operand's value. Unlike `VarPath`, `container_index`
/// is an operand slot (resolved against the op's arguments), not a bound variable name.
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
#[derive(Clone)]
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

/// A compound expression. It appears only as the right-hand side of a `Let`; the arguments of `App`
/// and `Llvm` are atoms (variables).
#[derive(Clone)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// The ownership of a single reference-counting unit. `Own` receives ownership: the callee consumes it (by
/// releasing it or moving it into the result), and the caller retains it before the call at a
/// non-last use. `Borrow` only borrows it: neither side performs a refcount operation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ownership {
    Own,
    Borrow,
}

/// The ownership of one argument, shaped like the value: each reference-counting unit is `Own` or
/// `Borrow`, and a part of the value holding no unit is `NoUnit`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OwnershipShape {
    NoUnit,
    Fields(Vec<OwnershipShape>),
    Unit(Ownership),
}

/// The initializer of a global value: the symbol, its type, and the expression that computes it,
/// with the whole reachable graph marked global (refcount-exempt) before it is stored.
#[derive(Clone)]
pub struct RcGlobalInit {
    pub symbol: FullName,
    pub ty: Arc<TypeNode>,
    pub init: RcExprNode,
}
