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
    pub ret_ty: Arc<TypeNode>,
    pub body: RcExprNode,
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
    Destructure(RcVar, Vec<(usize, RcVar)>, RcExprNode),
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
    pub container_index: usize,
    pub path: FieldPath,
}

/// One arm of a `Match`: the variant it matches, the variable its payload is bound to, and the arm
/// body, whose value is its final `Ret`. `tag` is `Some` for a variant arm, whose payload is that
/// variant's value; it is `None` for a catch-all arm, whose payload is the whole scrutinee.
/// Code generation treats the last arm as the default case (mirroring the tag switch), so a
/// catch-all is always the final arm.
#[derive(Clone)]
pub struct MatchArm {
    pub tag: Option<usize>,
    pub payload: RcVar,
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
/// which is always sound; later state inference can specialize it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RcState {
    /// Read the object's refcount state at run time and dispatch three ways.
    Unknown,
    /// Known local: non-atomic increment/decrement, no state check.
    Local,
    /// Known threaded: atomic increment/decrement, no state check.
    Threaded,
    /// Known global: a no-op, emitting no code.
    Global,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::types::type_tyvar_star;

    // Cloning a node is O(1) because the expression is shared through an `Arc` rather than deep-copied
    // (the simplifier clones whole bodies, which can be thousands of nodes deep). Build a chain far
    // deeper than a deep-copying clone could hold on any thread stack, clone it, and confirm the clone
    // returns — a guard against a regression to a recursive clone.
    #[test]
    fn rc_expr_node_clone_is_stack_safe() {
        let var = RcVar {
            name: FullName::local("a"),
            ty: type_tyvar_star("a"),
            source: None,
            debug_name: None,
            skip_null_check: false,
        };
        let mut node = RcExprNode {
            expr: Arc::new(RcExpr::Ret(var.clone())),
            source: None,
        };
        for _ in 0..100_000 {
            node = RcExprNode {
                expr: Arc::new(RcExpr::Eval(var.clone(), node)),
                source: None,
            };
        }

        let clone = node.clone();

        // The recursive `Drop` of a chain this deep would itself overflow the stack; that is a
        // separate case this test does not exercise, so leak both chains and let the process reclaim
        // them flat at exit.
        std::mem::forget(node);
        std::mem::forget(clone);
    }
}
