use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::generator::{Generator, Object};
use crate::rc_ir::ast::{FieldPath, RcTarget, UniqueCheckOperand};
use crate::rc_ir::locality::ExtShape;
use crate::rc_ir::provenance::{boxed_leaf_paths, LeafOrigin, Provenance};
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

    /// A display name for dumps and pretty-printing: the op's name, the attributes that select it
    /// (in the name, or in brackets), then every operand in parentheses. An op with no operand shows
    /// its literal there instead.
    fn name(&self) -> String;

    /// Whether this op is a primitive literal.
    fn is_primitve_literal(&self) -> bool {
        false
    }

    /// Whether operand `i` is only borrowed (read without taking ownership), for the operand types
    /// the op is instantiated at. Default: every operand is owned.
    ///
    /// An op that declares a borrow reads that operand with `get_scoped_obj_noretain`: a plain read
    /// retains an unboxed global's boxed subobjects, and a borrow has no matching release.
    ///
    /// The default is the conservative answer; see `result_prov` for what an op that keeps it records.
    fn borrows_operand(&self, _i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        false
    }

    /// The container operand and boxed-leaf path whose runtime uniqueness this op branches on, for
    /// the operand types the op is instantiated at. Default: the op carries no such branch.
    ///
    /// Whether the branch exists depends on those types, so an op that emits one declares it through
    /// `unique_check_on_boxed_leaf`, which is where that dependence is stated. Readers may then take
    /// a declared check to be one the op really emits.
    fn unique_check_operand(
        &self,
        _arg_tys: &[Arc<TypeNode>],
        _type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        None
    }

    /// This op with the object its declared uniqueness check tests taken to be local, so that the
    /// check reads the reference count without first reading the state. Only an op that reports a
    /// check through `unique_check_operand` is asked for this, and every such op overrides it.
    ///
    /// A `generate` that emits a check it does not declare leaves that one reading the state, so the
    /// pair of methods stays honest about which check the annotation covers.
    fn assuming_local(&self) -> Box<dyn LLVMGen> {
        unreachable!("assuming_local called on an op that declares no uniqueness check")
    }

    /// Whether `assuming_local` was applied. The RC IR dump renders it.
    fn assumes_local(&self) -> bool {
        false
    }

    /// This op with its runtime uniqueness branch dropped. Only an op that reports a branch through
    /// `unique_check_operand` is asked to drop it, and every such op overrides this method; an op with
    /// no branch is never routed here.
    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        unreachable!("assuming_unique called on an op that carries no uniqueness branch")
    }

    /// The provenance of this op's result. Default: conservatively `Unknown` on every boxed leaf.
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
        Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)
    }

    /// The values this op reference-counts inside `generate`, for the operand types it is
    /// instantiated at. An op that counts none answers the empty list and is never annotated.
    ///
    /// **The declaration has to cover every retain and release `generate` emits.** The annotation
    /// says of all of them at once that the objects they touch are local, so one left out is the
    /// direction of error that corrupts memory. It is the same kind of hand-maintained claim as
    /// `unique_check_operand`, and `assuming_local` is what carries the answer back.
    ///
    /// The default covers the clone path: an op that force-uniques a shared container retain-copies
    /// its contents and releases the old container, so every op declaring a uniqueness check gets
    /// those two targets without writing them. An op that declares a check and only reads the count
    /// has no clone path, and says so by overriding with the empty list — the default errs towards
    /// over-declaring, which costs precision and never soundness.
    fn internal_rc_targets(&self, arg_tys: &[Arc<TypeNode>], type_env: &TypeEnv) -> Vec<RcTarget> {
        clone_path_rc_targets(self.unique_check_operand(arg_tys, type_env))
    }

    /// The locality of this op's result: for each of its boxed leaves, the condition on the operands
    /// under which that leaf's own object is non-local, and the condition under which something it
    /// reaches is. `ExtShape::merge` / `always` / `bottom` build the recurring answers, and
    /// `ExtShape::build_shape` writes a leaf-by-leaf one.
    ///
    /// Every op states its own; the choice among them is the author's to make, because both ways of
    /// defaulting fail silently. `merge` would pass an op that produces a boxed object its operands
    /// do not reach, which is the direction of error that corrupts memory; `always` would cost
    /// precision with no symptom to notice it by.
    fn locality_flow(
        &self,
        result_ty: &Arc<TypeNode>,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> ExtShape;

    /// Downcast hook, for the few passes that special-case a concrete op.
    fn as_any(&self) -> &dyn Any;
}
dyn_clone::clone_trait_object!(LLVMGen);

/// The uniqueness check an op emits on the value at `path` of operand `container_index`, if that
/// value is reference-counted. `None` where it is not: an unboxed value is taken to be unique
/// without a runtime test — `make_struct_union_unique` returns it unchanged and `is_unique` answers
/// the constant `true` — so there is no branch to report and none to drop.
pub fn unique_check_on_boxed_leaf(
    container_index: usize,
    path: FieldPath,
    arg_tys: &[Arc<TypeNode>],
    type_env: &TypeEnv,
) -> Option<UniqueCheckOperand> {
    let container_ty = &arg_tys[container_index];
    if !boxed_leaf_paths(container_ty, type_env).contains(&path) {
        return None;
    }
    Some(UniqueCheckOperand {
        container_index,
        path,
    })
}

/// The clone path a force-unique op takes when its container is shared: it retain-copies the
/// contents into a new container and releases the old one. Every op declaring a uniqueness check
/// takes it, so its targets are composed here — by `LLVMGen::internal_rc_targets`'s default, and by
/// the ops that override that default to add targets of their own.
pub fn clone_path_rc_targets(check: Option<UniqueCheckOperand>) -> Vec<RcTarget> {
    match check {
        Some(check) => vec![
            RcTarget::Operand(check.container_index, check.path.clone()),
            RcTarget::Contents(check.container_index, check.path),
        ],
        None => vec![],
    }
}

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
