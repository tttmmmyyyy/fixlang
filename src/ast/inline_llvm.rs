use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::generator::{Generator, Object};
use crate::rc_ir::ast::UniqueCheckOperand;
use crate::rc_ir::provenance::{BaseSource, Provenance};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

/// One inline-LLVM builtin operation. Each builtin is a struct that implements this trait; an
/// `InlineLLVM` holds a `Box<dyn LLVMGen>`. `typetag` serializes the trait object (tagged by op) so
/// the typecheck cache round-trips it.
#[typetag::serde(tag = "op")]
pub trait LLVMGen: DynClone + Send + Sync {
    /// Emit the op's code and return its value.
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c>;

    /// Emit the op, threading `tail` for a possible tail return. The default computes `generate` and
    /// returns it, building the tail return when `tail`. `fix` overrides this to emit a real tail call.
    fn generate_tail<'c, 'm>(
        &self,
        gc: &mut Generator<'c, 'm>,
        ty: &Arc<TypeNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let obj = self.generate(gc, ty);
        if tail {
            gc.build_tail(obj, true);
            None
        } else {
            Some(obj)
        }
    }

    /// The mutable free-variable references (for renaming).
    fn free_vars_mut(&mut self) -> Vec<&mut FullName>;

    /// The free variables by value.
    fn free_vars(&self) -> Vec<FullName> {
        dyn_clone::clone_box(self)
            .free_vars_mut()
            .into_iter()
            .map(|n| (*n).clone())
            .collect()
    }

    /// A display name (used by dumps and pretty-printing).
    fn name(&self) -> String;

    /// Whether this op is a primitive literal.
    fn is_primitve_literal(&self) -> bool {
        false
    }

    /// Whether operand `i` is only borrowed (read without taking ownership). Default: every operand
    /// is owned.
    ///
    /// The default is the conservative answer; see `result_prov` for what an op that keeps it records.
    fn borrows_operand(&self, _i: usize) -> bool {
        false
    }

    /// The container operand and boxed-leaf path whose runtime uniqueness this op branches on.
    /// Default: the op carries no such branch.
    fn unique_check_operand(&self) -> Option<UniqueCheckOperand> {
        None
    }

    /// This op with its runtime uniqueness branch dropped. Only an op that reports a branch through
    /// `unique_check_operand` is asked to drop it, and every such op overrides this method; an op with
    /// no branch is never routed here.
    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        unreachable!("assuming_unique called on an op that carries no uniqueness branch")
    }

    /// The provenance of this op's result. Default: conservatively `Dyn` on every boxed leaf.
    ///
    /// The conservative default is always sound, so an op that leaves it (here or in
    /// `borrows_operand`) where a more precise declaration is possible says in a comment why it does
    /// and what it gives up.
    ///
    /// An `Arg(i, path)` leaf that is its leaf's only source declares that the result leaf *is*
    /// argument `i`'s leaf, which also declares that argument leaf unconsumed. It may therefore only
    /// name a leaf the op passes through without producing a new reference to it — an op that hands
    /// back a value whose reference count or sharing it also reports on, or that publishes the value,
    /// must not (see `InlineLLVMIsUniqueFunctionBody` and `InlineLLVMMarkThreadedFunctionBody`, which
    /// say why). A leaf that joins an argument with another source says only where the result's
    /// sharing comes from: the op consumes that argument like any other.
    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, BaseSource::Dyn)
    }

    /// Downcast hook, for the few passes that special-case a concrete op.
    fn as_any(&self) -> &dyn Any;
}
dyn_clone::clone_trait_object!(LLVMGen);

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVM {
    pub generator: Box<dyn LLVMGen>,
    // The type of this LLVM expression.
    //
    // For example, in `@ : I64 -> Array a -> a = |i, arr| LLVM<Array::@(i, arr)>;`, the `generic_ty` of the InlineLLVM `LLVM<arr.Array::@(i, arr)>` is `a`.
    // Note that `generic_ty` may contain type variables, and it is not changed in type instantiation.
    pub generic_ty: Arc<TypeNode>,
}

impl InlineLLVM {
    // Convert all global FullNames to absolute paths.
    pub fn global_to_absolute(&self) -> Arc<InlineLLVM> {
        Arc::new(InlineLLVM {
            generator: self.generator.clone(),
            generic_ty: self.generic_ty.global_to_absolute(),
        })
    }
}
