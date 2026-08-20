// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use crate::ast::name::FullName;
use crate::ast::name::Name;
use crate::ast::program::TypeEnv;
use crate::ast::types::type_tycon;
use crate::ast::types::TyCon;
use crate::ast::types::TypeNode;
use crate::configuration::Configuration;
use crate::constants::RefcntState;
use crate::constants::TraverserWorkType;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::constants::CLOSURE_FUNPTR_IDX;
use crate::constants::CTRL_BLK_REFCNT_IDX;
use crate::constants::CTRL_BLK_REFCNT_STATE_IDX;
use crate::constants::DESTRUCTOR_OBJECT_DTOR_FIELD_IDX;
use crate::constants::DESTRUCTOR_OBJECT_VALUE_FIELD_IDX;
use crate::constants::DYNAMIC_OBJ_CAP_IDX;
use crate::constants::DYNAMIC_OBJ_TRAVARSER_IDX;
use crate::constants::SYMBOL_VERSION_SEPARATOR;
use crate::constants::SYMBOL_VERSION_SEPARATOR_SUBSTITUTE;
use crate::error::panic_with_msg;
use crate::ffi::CSignature;
use crate::fixstd::builtin::make_dynamic_object_ty;
use crate::fixstd::builtin::run_io_or_ios_runner;
use crate::fixstd::runtime::RUNTIME_ABORT;
use crate::fixstd::runtime::RUNTIME_EPRINTLN;
use crate::misc::flatten_opt;
use crate::misc::Map;
use crate::object::build_free_boxed;
use crate::object::control_block_type;
use crate::object::create_traverser;
use crate::object::lambda_function_type;
use crate::object::lambda_return_part_types;
use crate::object::refcnt_state_type;
use crate::object::refcnt_type;
use crate::object::traverser_type;
use crate::object::traverser_work_type;
use crate::object::ty_to_debug_embedded_ty;
use crate::object::ty_to_object_ty;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::SourceFile;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::RcState;
use crate::return_abi::{
    lambda_calling_convention_of_target, return_registers_of_target, returns_through_out_pointer,
    ReturnRegisters,
};
use either::Either;
use either::Either::Left;
use either::Either::Right;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::llvm_sys::debuginfo::LLVMMetadataReplaceAllUsesWith;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::types::StructType;
use inkwell::values::BasicValue;
use inkwell::values::BasicValueEnum;
use inkwell::values::FunctionValue;
use inkwell::values::GlobalValue;
use inkwell::values::IntValue;
use inkwell::values::PointerValue;
use inkwell::AddressSpace;
use inkwell::AtomicOrdering;
use inkwell::AtomicRMWBinOp;
use inkwell::IntPredicate;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    basic_block::BasicBlock,
    debug_info::{
        AsDIScope, DICompileUnit, DIDerivedType, DIFile, DIScope, DISubprogram, DIType,
        DWARFEmissionKind, DWARFSourceLanguage, DebugInfoBuilder,
    },
    intrinsics::Intrinsic,
    module::{FlagBehavior, Linkage},
    targets::{TargetData, TargetMachine},
    types::{AnyType, BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, CallSiteValue},
};
use std::{cell::RefCell, iter::successors, sync::Arc};

// A value bound to a name in the current scope.
#[derive(Clone)]
pub struct ScopedValue<'c> {
    accessor: ValueAccessor<'c>,
    /// Whether `get_scoped_obj` retains the value's boxed subobjects when reading it. True only for
    /// unboxed globals, which keep their own reference and so must hand out a retained copy; a boxed
    /// global is moved out on read, and local values are reference-counted by explicit RC-IR nodes.
    retain_on_read: bool,
}

// How a scoped value's `Object` is obtained: an in-register local object, or a global read through
// its getter function.
#[derive(Clone)]
pub enum ValueAccessor<'c> {
    Local(Object<'c>),
    Global(FunctionValue<'c>, Arc<TypeNode>),
}

impl<'c> ValueAccessor<'c> {
    // Get the object.
    pub fn get<'m>(&self, gc: &mut Generator<'c, 'm>) -> Object<'c> {
        match self {
            ValueAccessor::Local(ptr) => ptr.clone(),
            ValueAccessor::Global(fun, ty) => {
                let val = if ty.is_funptr() {
                    fun.as_global_value().as_basic_value_enum()
                } else {
                    let call = gc
                        .builder()
                        .build_call(fun.clone(), &[], "get_global_obj")
                        .unwrap()
                        .try_as_basic_value();
                    match call {
                        Left(val) => val,
                        Right(_) => {
                            let ty = ty.get_embedded_type(gc);
                            Generator::get_undef(&ty)
                        }
                    }
                };
                Object::new(val, ty.clone(), gc)
            }
        }
    }
}

// A Fix value being generated, living in LLVM registers, together with the Fix type that gives it
// its layout and reference-counting behavior.
#[derive(Clone)]
pub struct Object<'c> {
    // The object's value, held as the list of parts it splits into, in `type_parts` order. A boxed
    // object is the single heap pointer, a funcptr the single function pointer, an unboxed scalar
    // the value itself; an unbox struct is its fields spread out here rather than kept as one
    // aggregate, so a loop-carried field (an `Array`'s `@size`) stays visible to LLVM instead of
    // hiding inside an aggregate phi. A struct too wide to split is one part holding the whole
    // aggregate. The aggregate is reassembled on demand by `value`, only at memory and ABI
    // boundaries.
    data: Vec<BasicValueEnum<'c>>,
    pub ty: Arc<TypeNode>,
}

impl<'c> Object<'c> {
    // Construct an object from its assembled value: the heap pointer of a boxed object, the
    // function pointer of a funptr, or the embedded value of an unboxed one. The value is split
    // into parts on the way in.
    pub fn new<'m>(
        value: BasicValueEnum<'c>,
        ty: Arc<TypeNode>,
        gc: &mut Generator<'c, 'm>,
    ) -> Self {
        assert!(ty.is_ground());
        if gc.config.develop_mode && ty.is_unbox(gc.type_env()) && !ty.is_funptr() {
            let embed_ty = ty.get_embedded_type(gc);
            assert_eq!(embed_ty, value.get_type());
        }
        let data = gc.value_parts(value);
        Object { data, ty }
    }

    // Construct an object directly from its parts, in `type_parts` order. This is the fast path at
    // ABI and phi boundaries, where the parts are already in hand and reforming the aggregate only
    // to split it again in `new` would be wasted work.
    pub fn from_parts<'m>(
        data: Vec<BasicValueEnum<'c>>,
        ty: Arc<TypeNode>,
        gc: &mut Generator<'c, 'm>,
    ) -> Self {
        assert!(ty.is_ground());
        if gc.config.develop_mode {
            let embed_ty = ty.get_embedded_type(gc);
            let part_tys = gc.type_parts(embed_ty);
            assert_eq!(
                data.len(),
                part_tys.len(),
                "Object::from_parts part count disagrees with type_parts"
            );
            for (part, part_ty) in data.iter().zip(part_tys.iter()) {
                assert_eq!(part.get_type(), *part_ty);
            }
        }
        Object { data, ty }
    }

    // The parts of this object, in `type_parts` order.
    pub fn parts(&self) -> &[BasicValueEnum<'c>] {
        &self.data
    }

    // The object's parts as call arguments, for a callee that takes the object split into them.
    pub fn part_call_args(&self) -> Vec<BasicMetadataValueEnum<'c>> {
        self.data.iter().map(|v| (*v).into()).collect()
    }

    // Reassemble the object's value from its parts. Free for a boxed object, a funcptr, an unboxed
    // scalar, or a struct too wide to split (the single part is returned as is); a split unbox
    // struct is rebuilt with one `insertvalue` per field, which SROA folds away wherever the
    // aggregate is not truly needed.
    pub fn value<'m>(&self, gc: &mut Generator<'c, 'm>) -> BasicValueEnum<'c> {
        if self.ty.is_box(gc.type_env()) || self.ty.is_funptr() {
            return self.data[0];
        }
        let embedded = self.ty.get_embedded_type(gc);
        let mut parts = self.data.iter().copied();
        gc.assemble_from_parts(embedded, &mut parts)
    }

    // An object of type `ty` whose value is `undef`, for an unreachable point that still has to
    // produce a value of the type.
    pub fn undef<'m>(ty: Arc<TypeNode>, gc: &mut Generator<'c, 'm>) -> Self {
        let val = if ty.is_unbox(gc.type_env()) {
            ty.get_struct_type(gc).get_undef().as_basic_value_enum()
        } else {
            gc.context
                .ptr_type(AddressSpace::from(0))
                .get_undef()
                .as_basic_value_enum()
        };
        Object::new(val, ty.clone(), gc)
    }

    pub fn is_unbox(&self, type_env: &TypeEnv) -> bool {
        self.ty.is_unbox(type_env)
    }

    pub fn is_box(&self, type_env: &TypeEnv) -> bool {
        self.ty.is_box(type_env)
    }

    /// Whether this object is a function pointer, carried as the bare pointer: it captures nothing,
    /// so it has no struct to extract fields from and nothing to reference-count.
    pub fn is_funptr(&self) -> bool {
        self.ty.is_funptr()
    }

    /// Whether this object is a `#DynamicObject`, the boxed object a closure keeps its captured
    /// values in. Its fields vary with the closure, so its layout follows from the capture types
    /// passed to `ty_to_object_ty` rather than from its type alone, and it carries its own traverse
    /// function to drive those captures' lifetimes.
    pub fn is_dynamic_object(&self) -> bool {
        self.ty.is_dynamic()
    }

    // Whether this object is carried as one aggregate rather than split into its fields' parts, so
    // that its single part holds the whole struct (see `Generator::is_carried_whole`).
    pub fn is_carried_whole<'m>(&self, gc: &mut Generator<'c, 'm>) -> bool {
        let embedded = self.ty.get_embedded_type(gc);
        gc.is_carried_whole(embedded)
    }

    /// Whether this object is a `Std::FFI::Destructor`, which runs the destructor function it holds
    /// over its value as it is destroyed.
    pub fn is_destructor_object(&self) -> bool {
        self.ty.is_destructor_object()
    }

    /// The debug-info type describing this object where it is embedded in another value.
    pub fn debug_embedded_ty<'m>(&self, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
        ty_to_debug_embedded_ty(self.ty.clone(), gc)
    }

    /// The LLVM struct this object is laid out as: the struct held inline for an unboxed object, the
    /// struct pointed to for a boxed one.
    pub fn struct_ty<'m>(&self, gc: &mut Generator<'c, 'm>) -> StructType<'c> {
        assert!(!self.is_funptr());
        self.ty.get_struct_type(gc)
    }

    // Get the pointer to the field of an boxed object.
    pub fn gep_boxed<'m>(&self, gc: &mut Generator<'c, 'm>, field_idx: u32) -> PointerValue<'c> {
        assert!(self.ty.is_box(gc.type_env()));
        let struct_ty = self.struct_ty(gc);
        let ptr = self.value(gc).into_pointer_value();
        gc.builder()
            .build_struct_gep(struct_ty, ptr, field_idx, "ptr_to_field_nocap")
            .unwrap()
    }

    // Extract a field value of an object.
    // This cannot be used to get field of dynamic objects. Use `load_field_dynamic` instead.
    // This function does not support funptr type since in that case the `value` is not a struct.
    pub fn extract_field<'m>(
        &self,
        gc: &mut Generator<'c, 'm>,
        field_idx: u32,
    ) -> BasicValueEnum<'c> {
        assert!(!self.is_funptr());
        if self.is_unbox(&gc.type_env) {
            if self.is_carried_whole(gc) {
                return gc
                    .builder()
                    .build_extract_value(self.data[0].into_struct_value(), field_idx, "field")
                    .unwrap();
            }
            // The object's parts already hold the field, spread across a contiguous range; slice
            // that range and reassemble the field's value. The field lives directly in the parts,
            // so they stay independent for LLVM.
            let struct_ty = self.ty.get_embedded_type(gc).into_struct_type();
            let (off, cnt) = gc.field_part_range(struct_ty, field_idx);
            let field_ty = struct_ty.get_field_type_at_index(field_idx).unwrap();
            let mut parts = self.data[off..off + cnt].iter().copied();
            gc.assemble_from_parts(field_ty, &mut parts)
        } else {
            // When the object is boxed,
            let struct_ty = self.struct_ty(gc);
            self.extract_field_as(gc, struct_ty, field_idx)
        }
    }

    // Extract a field value of an object.
    // You can specify the struct type of the boxed object, ignoring the `ty` field of the object.
    // Can be used only for boxed objects, because currently there is no use case of this function for unboxed objects.
    pub fn extract_field_as<'m>(
        &self,
        gc: &mut Generator<'c, 'm>,
        ty: StructType<'c>,
        field_idx: u32,
    ) -> BasicValueEnum<'c> {
        assert!(self.is_box(&gc.type_env));
        let ptr_to_field = self.ptr_to_field_as(gc, ty, field_idx);
        let field_ty = ty.get_field_type_at_index(field_idx).unwrap();
        gc.builder()
            .build_load(field_ty, ptr_to_field, "field")
            .unwrap()
    }

    // Extract a field of an object as an `Object`, keeping its value in the part domain: for an
    // unbox object the field's parts are sliced straight out with no aggregate formed, so a struct
    // field never round-trips through an `insertvalue`/`extractvalue` that a later pass could sink
    // into an aggregate phi. `field_ty` is the field's Fix type; for a boxed object the field is
    // loaded from the heap and its (materialized) value split back into parts. The field is
    // moved out, not retained.
    pub fn extract_field_object<'m>(
        &self,
        gc: &mut Generator<'c, 'm>,
        field_idx: u32,
        field_ty: Arc<TypeNode>,
    ) -> Object<'c> {
        assert!(!self.is_funptr());
        if self.is_unbox(&gc.type_env) {
            if self.is_carried_whole(gc) {
                let field_val = self.extract_field(gc, field_idx);
                return Object::new(field_val, field_ty, gc);
            }
            let struct_ty = self.ty.get_embedded_type(gc).into_struct_type();
            let (off, cnt) = gc.field_part_range(struct_ty, field_idx);
            Object::from_parts(self.data[off..off + cnt].to_vec(), field_ty, gc)
        } else {
            let struct_ty = self.struct_ty(gc);
            let field_val = self.extract_field_as(gc, struct_ty, field_idx);
            Object::new(field_val, field_ty, gc)
        }
    }

    // Insert a field value into an object.
    // This cannot be used to set field of dynamic objects. Use `store_field_dynamic` instead.
    // This function does not support funptr type since in that case the `value` is not a struct.
    pub fn insert_field<'m, V>(
        mut self,
        gc: &mut Generator<'c, 'm>,
        field_idx: u32,
        val: V,
    ) -> Object<'c>
    where
        V: BasicValue<'c>,
    {
        assert!(!self.is_funptr());
        if self.is_unbox(&gc.type_env) {
            if self.is_carried_whole(gc) {
                self.data[0] = gc
                    .builder()
                    .build_insert_value(
                        self.data[0].into_struct_value(),
                        val,
                        field_idx,
                        "set_field",
                    )
                    .unwrap()
                    .as_basic_value_enum();
                return self;
            }
            // Swap the field's parts in place: split the new field value into its own parts and
            // splice them over the range this field occupies, leaving every other field's parts
            // untouched and never materializing an aggregate.
            let struct_ty = self.ty.get_embedded_type(gc).into_struct_type();
            let (off, cnt) = gc.field_part_range(struct_ty, field_idx);
            let new_parts = gc.value_parts(val.as_basic_value_enum());
            assert_eq!(new_parts.len(), cnt);
            self.data.splice(off..off + cnt, new_parts);
        } else {
            // When the object is boxed,
            let struct_ty = self.struct_ty(gc);
            self.insert_field_as(gc, struct_ty, field_idx, val);
        }
        self
    }

    // Insert an `Object` into a field, keeping the value in the part domain: for an unbox object the
    // source object's parts are spliced straight into the field's range with no aggregate formed on
    // either side. For a boxed object the field is stored to the heap, where the value must be
    // materialized. The counterpart of `extract_field_object`.
    pub fn insert_field_object<'m>(
        mut self,
        gc: &mut Generator<'c, 'm>,
        field_idx: u32,
        field: &Object<'c>,
    ) -> Object<'c> {
        assert!(!self.is_funptr());
        if self.is_unbox(&gc.type_env) {
            if self.is_carried_whole(gc) {
                let val = field.value(gc);
                return self.insert_field(gc, field_idx, val);
            }
            let struct_ty = self.ty.get_embedded_type(gc).into_struct_type();
            let (off, cnt) = gc.field_part_range(struct_ty, field_idx);
            assert_eq!(field.parts().len(), cnt);
            self.data
                .splice(off..off + cnt, field.parts().iter().copied());
        } else {
            let val = field.value(gc);
            let struct_ty = self.struct_ty(gc);
            self.insert_field_as(gc, struct_ty, field_idx, val);
        }
        self
    }

    // Insert a field value into an object.
    // You can specify the struct type of the boxed object, ignoring the `ty` field of the object.
    // Can be used only for boxed objects, because currently there is no use case of this function for unboxed objects.
    pub fn insert_field_as<'m, V>(
        &self,
        gc: &mut Generator<'c, 'm>,
        ty: StructType<'c>,
        field_idx: u32,
        value: V,
    ) where
        V: BasicValue<'c>,
    {
        assert!(self.is_box(&gc.type_env));
        let ptr_to_field = self.ptr_to_field_as(gc, ty, field_idx);
        gc.builder().build_store(ptr_to_field, value).unwrap();
    }

    // Get the pointer to traverser function from a dynamic object.
    pub fn extract_trav_from_dynamic<'m>(&self, gc: &mut Generator<'c, 'm>) -> PointerValue<'c> {
        assert!(self.ty.is_dynamic());
        self.extract_field(gc, DYNAMIC_OBJ_TRAVARSER_IDX)
            .into_pointer_value()
    }

    // Check if the pointer is null.
    // Can be used for boxed objects.
    pub fn is_null<'m>(&self, gc: &mut Generator<'c, 'm>) -> IntValue<'c> {
        assert!(self.is_box(gc.type_env()));
        let ptr = self.value(gc).into_pointer_value();
        gc.builder().build_is_null(ptr, "is_null").unwrap()
    }

    // Get the pointer to the field of an boxed object.
    // Can be used only for boxed objects.
    pub fn ptr_to_field<'m>(&self, gc: &mut Generator<'c, 'm>, field_idx: u32) -> PointerValue<'c> {
        assert!(self.is_box(&gc.type_env));
        let ty = self.struct_ty(gc);
        self.ptr_to_field_as(gc, ty, field_idx)
    }

    // Get the pointer to the field of an boxed object.
    // You can specify the struct type of the boxed object, ignoring the `ty` field of the object.
    // Can be used only for boxed objects.
    pub fn ptr_to_field_as<'m>(
        &self,
        gc: &mut Generator<'c, 'm>,
        ty: StructType<'c>,
        field_idx: u32,
    ) -> PointerValue<'c> {
        assert!(self.is_box(&gc.type_env));
        let ptr = self.value(gc).into_pointer_value();
        gc.builder()
            .build_struct_gep(ty, ptr, field_idx, "gep2field")
            .unwrap()
    }
}

/// The local variables in scope at the point being generated. Globals are held separately, in
/// `Generator::declared_globals`.
#[derive(Default)]
pub struct Scope<'c> {
    /// Bindings of each name, innermost last: a lookup sees the last one pushed, so a binding
    /// shadows the outer bindings of the same name for as long as it lives.
    data: Map<FullName, Vec<ScopedValue<'c>>>,
}

impl<'c> Scope<'c> {
    /// Bind `var` to `obj`, shadowing whatever the name is bound to until the binding is popped.
    fn push_local(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        // TODO: add assertion that var is local (or change var to Name).
        self.data.entry(var.clone()).or_default().push(ScopedValue {
            accessor: ValueAccessor::Local(obj.clone()),
            retain_on_read: false,
        });
    }

    /// Drop the innermost binding of `var`, revealing the binding it shadowed.
    fn pop_local(&mut self, var: &FullName) {
        // TODO: add assertion that var is local (or change var to Name).
        let bindings = self.data.get_mut(var).unwrap();
        bindings.pop();
        if bindings.is_empty() {
            self.data.remove(var);
        }
    }

    /// The value `var` is currently bound to, which is the innermost of its bindings: a shadowed
    /// binding is seen again once the binding shadowing it is popped.
    pub fn get(&self, var: &FullName) -> ScopedValue<'c> {
        self.data.get(var).unwrap().last().unwrap().clone()
    }
}

/// The state of code generation for one LLVM module: the module being written, where in it the next
/// instruction goes, what is in scope there, and the caches shared across the whole module.
pub struct Generator<'c, 'm> {
    /// The LLVM context every type and value built here belongs to.
    pub context: &'c Context,
    /// The LLVM module being written.
    pub module: &'m Module<'c>,
    /// Stack of builders; the innermost is where instructions are appended. Generating a nested
    /// function pushes a builder of its own and pops it on the way out.
    builders: Arc<RefCell<Vec<Arc<Builder<'c>>>>>,
    /// Stack of local scopes, one per function body being generated. A local name is looked up in
    /// the innermost scope alone.
    scope: Arc<RefCell<Vec<Scope<'c>>>>,
    /// The debug info builder and compile unit, present where the module is built with debug info.
    debug_info: Option<(DebugInfoBuilder<'c>, DICompileUnit<'c>)>,
    /// Stack of debug scopes matching the function bodies being generated. `None` where the code
    /// being generated has no known source, in which case no debug location is emitted.
    debug_scope: Arc<RefCell<Vec<Option<DIScope<'c>>>>>,
    /// Stack of source spans; the innermost is the location instructions are attributed to. `None`
    /// where the code being generated has no known source.
    debug_location: Vec<Option<Span>>,
    /// The value of each global symbol the module has reached so far, by name. A global enters this
    /// map when code generation first asks for it (`get_or_declare_global`), which is also where it
    /// is declared, so the module declares the globals it uses and no others.
    declared_globals: Map<FullName, ScopedValue<'c>>,
    /// The functions this module emitted for each of its globals.
    /// `keep_initializers_out_of_shared_accessors` reads it once every reader of every global is
    /// in the module, and decides there which accessors keep their initializer.
    pub(crate) emitted_globals: Vec<EmittedGlobal<'c>>,
    /// The type of every global symbol of the program, by name — every compilation unit's, since a
    /// unit's code calls into the others. It is what a global is declared from on first use.
    global_types: Arc<Map<FullName, Arc<TypeNode>>>,
    /// Type definitions of the program, used to resolve a Fix type to its layout.
    type_env: TypeEnv,
    /// Layout of the target the module is compiled for: sizes, alignments and struct offsets.
    pub target_data: TargetData,
    /// How many registers of each class the target returns a value in, read once from the module's
    /// triple. `returns_through_out_pointer` needs it for every function type built.
    return_registers: ReturnRegisters,
    /// The convention every Fix lambda in this module is defined and called with, read once from the
    /// module's triple.
    lambda_calling_convention: u32,
    /// The configuration the program is being built under.
    pub config: Configuration,
    /// The global constant emitted for each Rust string embedded in the module, keyed by the string,
    /// so that one string is emitted once.
    global_strings: Map<String, GlobalValue<'c>>,
    /// Debug type built for each Fix type, keyed by the type's canonical string, so a type is
    /// described once and shared across every reference to it.
    di_type_cache: Map<String, DIType<'c>>,
    /// Placeholder node for each Fix type whose debug type is mid-construction. A reference to the
    /// type reached while building it (as a recursive type refers to itself) resolves to the
    /// placeholder, which is replaced by the finished type once construction completes. Without it,
    /// describing a recursive type would recurse forever.
    di_type_placeholders: Map<String, DIDerivedType<'c>>,
    /// The LLVM struct each Fix type is laid out as. Laying a type out walks every type it is built
    /// from, so a type reached from many places -- a field type repeated across a struct, a struct
    /// nested several levels deep -- would otherwise be laid out once per path that reaches it.
    struct_types: Map<Arc<TypeNode>, StructType<'c>>,
    /// The LLVM type each Fix type takes where it is embedded in another value, kept for the reason
    /// given at `struct_types`.
    embedded_types: Map<Arc<TypeNode>, BasicTypeEnum<'c>>,
    /// The out-pointer buffer of each Fix type returned through one.
    out_pointer_buffers: Map<Arc<TypeNode>, StructType<'c>>,
}

pub struct PopBuilderGuard<'c> {
    builders: Arc<RefCell<Vec<Arc<Builder<'c>>>>>,
}

impl<'c> Drop for PopBuilderGuard<'c> {
    fn drop(&mut self) {
        self.builders.borrow_mut().pop().unwrap();
    }
}

pub struct PopScopeGuard<'c> {
    scope: Arc<RefCell<Vec<Scope<'c>>>>,
}

impl<'c> Drop for PopScopeGuard<'c> {
    fn drop(&mut self) {
        self.scope.borrow_mut().pop();
    }
}

pub struct PopDebugScopeGuard<'c> {
    scope: Arc<RefCell<Vec<Option<DIScope<'c>>>>>,
}

impl<'c> Drop for PopDebugScopeGuard<'c> {
    fn drop(&mut self) {
        self.scope.borrow_mut().pop();
    }
}

impl<'c, 'm> Generator<'c, 'm> {
    /// The module-level constant holding `s` as a null-terminated string. One constant is created
    /// per distinct string, and every later call for that string returns it again.
    pub fn add_global_string(&mut self, s: &str) -> GlobalValue<'c> {
        if let Some(val) = self.global_strings.get(s) {
            return val.clone();
        }
        let gv = self
            .builder()
            .build_global_string_ptr(s, "global_string")
            .unwrap();
        self.global_strings.insert(s.to_string(), gv);
        gv
    }

    // Build alloca at current function's entry bb.
    pub fn build_alloca_at_entry<T: BasicType<'c>>(
        &mut self,
        ty: T,
        name: &str,
    ) -> PointerValue<'c> {
        let current_bb = self.builder().get_insert_block().unwrap();
        let first_bb = self.current_function().get_first_basic_block().unwrap();
        match first_bb.get_first_instruction() {
            Some(first_inst) => self.builder().position_before(&first_inst),
            None => self.builder().position_at_end(first_bb),
        }
        let ptr = self.builder().build_alloca(ty, name).unwrap();
        self.builder().position_at_end(current_bb);
        self.reset_debug_location();
        ptr
    }

    // Store stack pointer.
    #[allow(dead_code)]
    pub fn save_stack(&mut self) -> PointerValue<'c> {
        let intrinsic = Intrinsic::find("llvm.stacksave").unwrap();
        let func = intrinsic.get_declaration(&self.module, &[]).unwrap();
        self.builder()
            .build_call(func, &[], "save_stack")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap()
            .into_pointer_value()
    }

    // Restore stack pointer.
    #[allow(dead_code)]
    pub fn restore_stack(&mut self, pos: PointerValue<'c>) {
        let intrinsic = Intrinsic::find("llvm.stackrestore").unwrap();
        assert!(!intrinsic.is_overloaded()); // So we don't need to specify type parameters in the next line.
        let func = intrinsic.get_declaration(&self.module, &[]).unwrap();
        self.builder()
            .build_call(func, &[pos.into()], "restore_stack")
            .unwrap();
    }

    /// The type definitions of the program, which a Fix type is resolved to its layout through.
    pub fn type_env(&self) -> &TypeEnv {
        &self.type_env
    }

    /// The LLVM struct a value of `ty` is laid out as.
    pub fn struct_type_of(&mut self, ty: &Arc<TypeNode>) -> StructType<'c> {
        if let Some(struct_ty) = self.struct_types.get(ty) {
            return *struct_ty;
        }
        let object_ty = ty_to_object_ty(ty, &vec![], self.type_env());
        let struct_ty = object_ty.to_struct_type(self);
        self.struct_types.insert(ty.clone(), struct_ty);
        struct_ty
    }

    /// The LLVM type a value of `ty` takes where it is embedded in another value: the struct it is
    /// laid out as when it is unboxed, a pointer when it is boxed.
    pub fn embedded_type_of(&mut self, ty: &Arc<TypeNode>) -> BasicTypeEnum<'c> {
        if let Some(embedded_ty) = self.embedded_types.get(ty) {
            return *embedded_ty;
        }
        let object_ty = ty_to_object_ty(ty, &vec![], self.type_env());
        let embedded_ty = object_ty.to_embedded_type(self);
        self.embedded_types.insert(ty.clone(), embedded_ty);
        embedded_ty
    }

    /// The buffer a lambda returning `ret_ty` writes its result through, as the flat struct of the
    /// parts the result would otherwise have been returned in.
    ///
    /// The struct is named after the type, so the module writes the parts out once, at the name's
    /// definition. An anonymous struct is written out in full at every instruction that names it,
    /// and the buffer is named by one instruction per part at both ends of a call, which puts the
    /// module's text in the square of the part count.
    pub fn out_pointer_buffer_type(
        &mut self,
        ret_ty: &Arc<TypeNode>,
        part_tys: &[BasicTypeEnum<'c>],
    ) -> StructType<'c> {
        if let Some(buf_ty) = self.out_pointer_buffers.get(ret_ty) {
            let buf_ty = *buf_ty;
            // One buffer serves every writer and reader of a type's result, so they must all name
            // the same parts; disagreeing lists would put the two ends of a call at different
            // offsets. Checked under develop mode (the unit tests).
            if self.config.develop_mode {
                assert_eq!(
                    buf_ty.get_field_types().as_slice(),
                    part_tys,
                    "`{}` reached its out-pointer buffer with two different part lists",
                    ret_ty.to_string()
                );
            }
            return buf_ty;
        }
        let buf_ty = self
            .context
            .opaque_struct_type(&format!("out.{}", ret_ty.to_string()));
        buf_ty.set_body(part_tys, false);
        self.out_pointer_buffers.insert(ret_ty.clone(), buf_ty);
        buf_ty
    }

    /// The number of bytes a value of `ty` occupies on the target, padding included.
    pub fn sizeof(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_bit_size(ty) / 8
    }

    // The minimum alignment required to store or load a value of this type; an empty aggregate is 1.
    pub fn abi_alignment(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_abi_alignment(ty) as u64
    }

    /// The number of bytes a pointer occupies on the target. Fix supports 64-bit targets, so this
    /// asserts the size is 8.
    pub fn ptr_size(&mut self) -> u64 {
        let ptr_ty = self.context.ptr_type(AddressSpace::from(0));
        let ptr_size = self.target_data.get_bit_size(&ptr_ty) / 8;
        assert_eq!(ptr_size, 8);
        ptr_size
    }

    // An empty LLVM module called `name`, carrying the triple and data layout of `target_machine`
    // so that the types built in it get that target's sizes, alignments and offsets.
    pub fn create_module(
        name: &str,
        ctx: &'c Context,
        target_machine: &TargetMachine,
    ) -> Module<'c> {
        let module = ctx.create_module(name);
        module.set_triple(&target_machine.get_triple());
        module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        module
    }

    // Create new gc. `global_types` gives the type of every global symbol of the program, from which
    // a global is declared the first time this module reaches it.
    pub fn new(
        ctx: &'c Context,
        module: &'m Module<'c>,
        target_data: TargetData,
        config: Configuration,
        type_env: TypeEnv,
        global_types: Arc<Map<FullName, Arc<TypeNode>>>,
    ) -> Self {
        let triple = module.get_triple().as_str().to_string_lossy().to_string();
        let gc = Self {
            context: ctx,
            module,
            builders: Arc::new(RefCell::new(vec![Arc::new(ctx.create_builder())])),
            scope: Arc::new(RefCell::new(vec![Default::default()])),
            debug_scope: Arc::new(RefCell::new(vec![])),
            debug_info: Default::default(),
            debug_location: vec![],
            declared_globals: Default::default(),
            emitted_globals: Vec::new(),
            global_types,
            type_env,
            target_data: target_data,
            return_registers: return_registers_of_target(&triple),
            lambda_calling_convention: lambda_calling_convention_of_target(&triple),
            config,
            global_strings: Map::default(),
            di_type_cache: Map::default(),
            di_type_placeholders: Map::default(),
            struct_types: Map::default(),
            embedded_types: Map::default(),
            out_pointer_buffers: Map::default(),
        };
        gc
    }

    /// Opens the debug information of this module: the builder every debug entity is emitted
    /// through, and the compilation unit they all belong to, whose directory is the one the build
    /// runs in.
    pub fn create_debug_info(&mut self) {
        let debug_metadata_version = self.context.i32_type().const_int(3, false);
        self.module.add_basic_value_flag(
            "Debug Info Version",
            FlagBehavior::Warning,
            debug_metadata_version,
        );
        // The compilation directory reaches the generated code here alone, which
        // `Configuration::object_generation_hash` rests on: it covers the directory for a build
        // with debug information and leaves it out of the key of any other build.
        let compilation_directory = self.config.compilation_directory.clone();
        let (dib, dicu) = self.module.create_debug_info_builder(
            true,
            DWARFSourceLanguage::C,
            "NA",
            compilation_directory.to_str().unwrap(),
            "fix",
            false,
            "",
            0,
            "",
            DWARFEmissionKind::Full,
            0,
            false,
            false,
            "",
            "",
        );
        self.debug_info = Some((dib, dicu));
    }

    // Get builder.
    pub fn builder(&self) -> Arc<Builder<'c>> {
        self.builders.borrow().last().unwrap().clone()
    }

    // Push a new builder.
    pub fn push_builder(&mut self) -> PopBuilderGuard<'c> {
        self.builders
            .borrow_mut()
            .push(Arc::new(self.context.create_builder()));
        PopBuilderGuard {
            builders: self.builders.clone(),
        }
    }

    // Add a global object.
    pub fn add_global_object(
        &mut self,
        name: FullName,
        function: FunctionValue<'c>,
        ty: Arc<TypeNode>,
    ) {
        if self.declared_globals.contains_key(&name) {
            panic_with_msg(&format!("Duplicate symbol: {}", name.to_string()));
        } else {
            // A boxed global is moved out when read, so it needs no retain; an unboxed global keeps
            // its own reference, so reading it must retain its boxed subobjects.
            let retain_on_read = !ty.is_box(self.type_env());
            self.declared_globals.insert(
                name.clone(),
                ScopedValue {
                    accessor: ValueAccessor::Global(function, ty),
                    retain_on_read,
                },
            );
        }
    }

    // Push a new scope.
    pub fn push_scope(&mut self) -> PopScopeGuard<'c> {
        self.scope.borrow_mut().push(Default::default());
        PopScopeGuard {
            scope: self.scope.clone(),
        }
    }

    /// Enters `scope` as the debug scope the code generated from here on belongs to, carrying no
    /// source location inside it yet. The returned guard leaves the scope when it is dropped.
    pub fn push_debug_scope(&mut self, scope: Option<DIScope<'c>>) -> PopDebugScopeGuard<'c> {
        self.debug_scope.borrow_mut().push(scope);
        self.push_debug_location(None);
        PopDebugScopeGuard {
            scope: self.debug_scope.clone(),
        }
    }

    /// The debug scope the code being generated belongs to. It is `None` where that code has no
    /// known source, and instructions generated there carry no debug location.
    pub fn debug_scope(&self) -> Option<DIScope<'c>> {
        flatten_opt(self.debug_scope.borrow().last().cloned())
    }

    // Get a variable.
    pub fn get_scoped_value(&mut self, var: &FullName) -> ScopedValue<'c> {
        if var.is_local() {
            self.scope.borrow().last().unwrap().get(var)
        } else {
            self.get_or_declare_global(var)
        }
    }

    // The value the global `var` is reached through, declared here on the module's first use of it.
    // Declaring on use is what keeps a module's declarations to the globals its code reaches: the
    // program's globals number in the hundreds and a module calls a handful of them.
    fn get_or_declare_global(&mut self, var: &FullName) -> ScopedValue<'c> {
        if let Some(value) = self.declared_globals.get(var).cloned() {
            return value;
        }
        self.declare_program_global(var)
            .unwrap_or_else(|| panic!("global not found in codegen: `{}`", var.to_string()));
        self.declared_globals[var].clone()
    }

    // Get an object on the scope (or global).
    // This function does not retain the object.
    pub fn get_scoped_obj_noretain(&mut self, name: &FullName) -> Object<'c> {
        self.get_scoped_value(name).accessor.get(self)
    }

    // Get an object on the scope (or global).
    // Retains the object's boxed subobjects when the value's `retain_on_read` is set, i.e. when
    // reading an unboxed global (which keeps its own reference); other reads are plain.
    pub fn get_scoped_obj(&mut self, var_name: &FullName) -> Object<'c> {
        let val = self.get_scoped_value(var_name);
        let obj = val.accessor.get(self);
        if val.retain_on_read {
            let one = self.context.i64_type().const_int(1, false);
            // The subobjects of an unboxed global are marked global, so this retain is already a
            // no-op at run time; it keeps the runtime dispatch that makes it one.
            self.build_retain(obj.clone(), one, RcState::Unknown);
        }
        obj
    }

    // Get field of object on the scope.
    // This function retains the object if it will be used later.
    pub fn get_scoped_obj_field(
        self: &mut Self,
        var: &FullName,
        field_idx: u32,
    ) -> BasicValueEnum<'c> {
        let obj = self.get_scoped_obj(var);
        obj.extract_field(self, field_idx)
    }

    /// Bind `var` to `obj` in the innermost scope, shadowing any binding `var` already has there.
    pub fn scope_push(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .push_local(var, obj)
    }

    /// Drop the innermost binding of `var`, revealing the binding it shadowed.
    pub fn scope_pop(self: &mut Self, var: &FullName) {
        self.scope.borrow_mut().last_mut().unwrap().pop_local(var);
    }

    /// The pointer to the reference count in the control block of the boxed object at `obj`.
    pub fn get_refcnt_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        self.builder()
            .build_struct_gep(
                control_block_type(self),
                obj,
                CTRL_BLK_REFCNT_IDX,
                "ptr_to_refcnt",
            )
            .unwrap()
    }

    /// Whether the object's reference count is one, read in the current block.
    ///
    /// `acquire` makes the load an acquire one, which is what an object in the threaded state needs:
    /// the writes a unique answer licences must be ordered after the reads the other holders did
    /// before releasing the object. Keep the acquire on the load itself: ThreadSanitizer draws no
    /// happens-before edge from a standalone `fence acquire`, so an acquire moved into one leaves
    /// the code correct while making the race detector report the writes that follow as racing.
    ///
    /// `name_suffix` distinguishes the emitted values from those of the other counts a function
    /// reads.
    fn build_is_refcnt_one(
        &mut self,
        obj_ptr: PointerValue<'c>,
        acquire: bool,
        name_suffix: &str,
    ) -> IntValue<'c> {
        let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
        let refcnt = self
            .builder()
            .build_load(
                refcnt_type(self.context),
                ptr_to_refcnt,
                &format!("refcnt{}", name_suffix),
            )
            .unwrap()
            .into_int_value();
        if acquire {
            refcnt
                .as_instruction_value()
                .unwrap()
                .set_atomic_ordering(AtomicOrdering::Acquire)
                .expect("Set atomic ordering failed");
        }
        let one = refcnt_type(self.context).const_int(1, false);
        self.builder()
            .build_int_compare(
                IntPredicate::EQ,
                refcnt,
                one,
                &format!("is_unique{}", name_suffix),
            )
            .unwrap()
    }

    /// Branch on whether the object's reference count is one, returning the block for each answer.
    ///
    /// A global object is never unique, so the count is read only after the state says the object
    /// is local. Where `state` says so already, the state is not read and the global case does not
    /// exist — the check becomes the comparison against one alone.
    pub fn build_branch_by_is_unique(
        self: &mut Generator<'c, 'm>,
        obj_ptr: PointerValue<'c>,
        state: RcState,
    ) -> (BasicBlock<'c>, BasicBlock<'c>) {
        let current_func = self.current_function();

        let unique_bb = self.context.append_basic_block(current_func, "unique_bb");
        let shared_bb = self.context.append_basic_block(current_func, "shared_bb");

        // Branch by refcnt_state.
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr, state);

        // Implement local_bb.
        self.builder().position_at_end(local_bb);
        // Jump to shared_bb if refcnt > 1.
        let is_unique = self.build_is_refcnt_one(obj_ptr, false, "");
        self.builder()
            .build_conditional_branch(is_unique, unique_bb, shared_bb)
            .unwrap();

        // Implement threaded_bb.
        if let Some(threaded_bb) = threaded_bb {
            let unique_threaded_bb = self
                .context
                .append_basic_block(current_func, "unique_threaded_bb");

            self.builder().position_at_end(threaded_bb);
            // Jump to shared_bb if refcnt > 1.
            let is_unique = self.build_is_refcnt_one(obj_ptr, true, "");
            self.builder()
                .build_conditional_branch(is_unique, unique_threaded_bb, shared_bb)
                .unwrap();

            // Implement unique_threaded_bb.
            self.builder().position_at_end(unique_threaded_bb);
            // A unique object has one holder, so its count is updated without atomics. Marking
            // rests on this too: `build_mark_boxed_with` ends its traversal at an object carrying
            // the mark, and a threaded object is returned to the local state here before a write
            // in place gives it a child of its own.
            self.set_refcnt_state(obj_ptr, RefcntState::LOCAL);
            // And jump to unique_bb.
            self.builder()
                .build_unconditional_branch(unique_bb)
                .unwrap();
        }

        // Implement global_bb.
        if let Some(global_bb) = global_bb {
            self.builder().position_at_end(global_bb);
            // Jump to shared_bb.
            self.builder()
                .build_unconditional_branch(shared_bb)
                .unwrap();
        }

        (unique_bb, shared_bb)
    }

    /// Load the object's reference-count state and branch on it, returning the block to emit each
    /// case in: the local one, the threaded one (only in a threaded build), and the global one.
    ///
    /// Where `state` says the object is known local, no state is loaded and no branch is built: the
    /// local case is emitted in the current block and the other two blocks do not exist.
    pub fn build_branch_by_refcnt_state(
        self: &mut Generator<'c, 'm>,
        obj_ptr: PointerValue<'c>,
        state: RcState,
    ) -> (
        BasicBlock<'c>,
        Option<BasicBlock<'c>>,
        Option<BasicBlock<'c>>,
    ) {
        if !state.dispatches() {
            self.build_assert_refcnt_state_local(obj_ptr);
            // The caller is positioned where the operation goes, which is where its local case goes.
            return (self.builder().get_insert_block().unwrap(), None, None);
        }
        // Load refcnt_state.
        let current_func = self.current_function();
        let refcnt_state = self.build_load_refcnt_state(obj_ptr, "refcnt_state");

        // Add three basic blocks.
        let local_bb = self.context.append_basic_block(current_func, "local_bb");
        let mut threaded_bb: Option<BasicBlock<'_>> = None;
        let global_bb = self.context.append_basic_block(current_func, "global_bb");

        if !self.config.threaded {
            // In single-threaded program,

            // Check refcnt_state and jump to local_bb if the object is local.
            let is_refcnt_state_local = self.build_compare_refcnt_state(
                refcnt_state,
                IntPredicate::EQ,
                RefcntState::LOCAL,
                "is_refcnt_state_local",
            );
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, global_bb)
                .unwrap();
        } else {
            // In multi-threaded program,
            threaded_bb = Some(self.context.append_basic_block(current_func, "threaded_bb"));
            let threaded_bb = threaded_bb.unwrap();

            let nonlocal_bb = self.context.append_basic_block(current_func, "nonlocal_bb");

            let is_refcnt_state_local = self.build_compare_refcnt_state(
                refcnt_state,
                IntPredicate::EQ,
                RefcntState::LOCAL,
                "is_refcnt_state_local",
            );
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, nonlocal_bb)
                .unwrap();

            // Implement nonlocal_bb.
            self.builder().position_at_end(nonlocal_bb);
            let is_refcnt_state_threaded = self.build_compare_refcnt_state(
                refcnt_state,
                IntPredicate::EQ,
                RefcntState::THREADED,
                "is_refcnt_state_threaded",
            );
            self.builder()
                .build_conditional_branch(is_refcnt_state_threaded, threaded_bb, global_bb)
                .unwrap();
        }
        (local_bb, threaded_bb, Some(global_bb))
    }

    /// Abort when the boxed object at `obj_ptr` is not in the local reference-counting state.
    ///
    /// It stands where the state dispatch a `RcState::Local` annotation removed used to be, so a
    /// wrong annotation stops at the operation that made it instead of corrupting a reference count
    /// somewhere else. Locality inference rests on a hand-written declaration per inline-LLVM
    /// operation, and this is the only check on those: the whole test suite is built in develop
    /// mode, so every annotated site is verified dynamically on every test program.
    ///
    /// The check belongs in every state dispatch, so that no annotated site goes unchecked.
    fn build_assert_refcnt_state_local(&mut self, obj_ptr: PointerValue<'c>) {
        if !self.config.develop_mode {
            return;
        }
        let refcnt_state = self.build_load_refcnt_state(obj_ptr, "refcnt_state@assert_local");
        let is_local = self.build_compare_refcnt_state(
            refcnt_state,
            IntPredicate::EQ,
            RefcntState::LOCAL,
            "is_refcnt_state_local@assert",
        );
        let current_func = self.current_function();
        let nonlocal_bb = self
            .context
            .append_basic_block(current_func, "nonlocal_bb@assert_local");
        let local_bb = self
            .context
            .append_basic_block(current_func, "local_bb@assert_local");
        self.builder()
            .build_conditional_branch(is_local, local_bb, nonlocal_bb)
            .unwrap();

        self.builder().position_at_end(nonlocal_bb);
        self.panic("A reference-counting operation inferred local reached a non-local object.\n");
        self.builder().build_unconditional_branch(local_bb).unwrap();

        self.builder().position_at_end(local_bb);
    }

    /// Abort, in compiler development mode, when the object at `obj_ptr` is shared where the
    /// uniqueness analysis proved it unique.
    ///
    /// Dropping the check that clones a shared container is what makes an in-place write legal, so
    /// a wrong proof turns a value another holder can see into one this code overwrites. The check
    /// the proof removed is the observation that would have caught it, so it is made again here,
    /// where a violated proof stops at the write rather than at whatever reads the value later.
    ///
    /// It reads the count the way `build_branch_by_is_unique` does, so that it answers as the check
    /// it stands in for would: a global object is shared whatever its count says, and a threaded
    /// one is read by an acquire load, which is also what keeps this check from being a race of its
    /// own. It leaves the state as it found it, the operation it guards having none of its own to
    /// mark.
    ///
    /// The object's state is read at run time. An op declares the uniqueness check it emits through
    /// `LLVMGen::unique_check_operand`, and it withdraws that declaration exactly where the proof
    /// was accepted, which is where this check stands; a locality annotation resting on the
    /// withdrawn declaration therefore says nothing about the object here.
    ///
    /// Development mode only: this restores the cost the proof exists to remove.
    pub fn build_assert_unique(&mut self, obj_ptr: PointerValue<'c>) {
        if !self.config.develop_mode {
            return;
        }
        let current_func = self.current_function();
        let unique_bb = self
            .context
            .append_basic_block(current_func, "unique_bb@assert_unique");
        let shared_bb = self
            .context
            .append_basic_block(current_func, "shared_bb@assert_unique");

        let (local_bb, threaded_bb, global_bb) =
            self.build_branch_by_refcnt_state(obj_ptr, RcState::Unknown);

        // Implement local_bb: read the count and compare it against one.
        self.builder().position_at_end(local_bb);
        let is_unique = self.build_is_refcnt_one(obj_ptr, false, "@assert_unique");
        self.builder()
            .build_conditional_branch(is_unique, unique_bb, shared_bb)
            .unwrap();

        // Implement threaded_bb: the same, reading the count atomically.
        if let Some(threaded_bb) = threaded_bb {
            self.builder().position_at_end(threaded_bb);
            let is_unique = self.build_is_refcnt_one(obj_ptr, true, "@assert_unique");
            self.builder()
                .build_conditional_branch(is_unique, unique_bb, shared_bb)
                .unwrap();
        }

        // Implement global_bb: a global object is shared, so the proof is wrong wherever it names
        // one.
        let global_bb =
            global_bb.expect("the state is read under `RcState::Unknown`, so a global arm exists.");
        self.builder().position_at_end(global_bb);
        self.builder()
            .build_unconditional_branch(shared_bb)
            .unwrap();

        self.builder().position_at_end(shared_bb);
        self.panic("A value proven uniquely owned was reached while shared.\n");
        self.builder()
            .build_unconditional_branch(unique_bb)
            .unwrap();

        self.builder().position_at_end(unique_bb);
    }

    /// The pointer to the reference-count state in the control block of the boxed object at `obj`.
    pub fn get_refcnt_state_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        self.builder()
            .build_struct_gep(
                control_block_type(self),
                obj,
                CTRL_BLK_REFCNT_STATE_IDX,
                "ptr_to_refcnt_state",
            )
            .unwrap()
    }

    /// The code pointer to call a lambda through: the funcptr field of a closure, or the value
    /// itself when the lambda is a bare function pointer.
    fn get_lambda_func_ptr(&mut self, obj: Object<'c>) -> PointerValue<'c> {
        // Get the pointer value.
        if obj.ty.is_closure() {
            obj.extract_field(self, CLOSURE_FUNPTR_IDX)
                .into_pointer_value()
        } else if obj.ty.is_funptr() {
            obj.value(self).into_pointer_value()
        } else {
            panic!()
        }
    }

    // Apply objects to a lambda.
    pub fn apply_lambda(
        &mut self,
        fun: Object<'c>,
        args: Vec<Object<'c>>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let src_tys = fun.ty.get_lambda_srcs();
        let ret_ty = fun.ty.get_lambda_dst();

        // Validate arguments.
        assert!(fun.ty.is_closure() || fun.ty.is_funptr());
        assert_eq!(args.len(), src_tys.len());
        for i in 0..args.len() {
            assert_eq!(args[i].ty, src_tys[i])
        }

        // Get function.
        let func_ptr = self.get_lambda_func_ptr(fun.clone());
        let func_ty = lambda_function_type(&fun.ty, self);

        // A tail call returns what this function returns, so the two signatures agree on how the
        // result travels: both return it by value, or both take an out-pointer and return `void`.
        // Everything below leans on that — the forwarded out-pointer names this function's own
        // buffer, and the callee's result is returned verbatim. Checked under develop mode (the unit
        // tests).
        if tail && self.config.develop_mode {
            let func = self.current_function();
            assert_eq!(
                func.get_type().get_return_type(),
                func_ty.get_return_type(),
                "the tail call in `{}` returns something other than what it returns",
                func.get_name().to_str().unwrap()
            );
        }

        // Call function pointer with the out-pointer if the result is too wide for the return
        // registers, then the arguments, then CAP if closure. Each unbox-struct argument is
        // split into its parts to match the signature (see
        // `lambda_function_type`).
        let ret_part_tys = lambda_return_part_types(&fun.ty, self);
        let out_ptr = if self.returns_through_out_pointer(&ret_part_tys) {
            Some(self.build_out_pointer_argument(&ret_ty, &ret_part_tys, tail))
        } else {
            None
        };
        let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
        if let Some(out_ptr) = out_ptr {
            call_args.push(out_ptr.into());
        }
        for arg in args {
            for part in arg.parts() {
                call_args.push((*part).into());
            }
        }
        if fun.ty.is_closure() {
            call_args.push(fun.extract_field(self, CLOSURE_CAPTURE_IDX).into());
        }

        let call_site = self
            .builder()
            .build_indirect_call(func_ty, func_ptr, &call_args, "call_lambda")
            .unwrap();
        call_site.set_call_convention(self.lambda_calling_convention());
        // `tail` asserts that the callee reaches no alloca of this function, which a call handed a
        // buffer allocated here does. In tail position the pointer is this function's own parameter,
        // naming an ancestor's buffer, so the assertion holds there.
        let passes_local_buffer = out_ptr.is_some() && !tail;
        call_site.set_tail_call(!passes_local_buffer);
        let call_result = call_site.try_as_basic_value().left();
        if tail {
            // The callee's flat return value already has this function's return type (a tail call
            // returns what its caller returns), so forward it verbatim without unpacking and repacking.
            // A callee writing through the out-pointer returned `void` and has already filled this
            // function's own buffer, which the forwarded pointer named.
            match call_result {
                Some(v) => self.builder().build_return(Some(&v)).unwrap(),
                None => self.builder().build_return(None).unwrap(),
            };
            return None;
        }
        if let Some(out_ptr) = out_ptr {
            return Some(self.load_out_pointer_buffer(out_ptr, &ret_part_tys, ret_ty));
        }
        Some(self.unpack_return(call_result, ret_ty))
    }

    // The pointer to pass as a call's out-pointer argument. In tail position it is this function's
    // own out-pointer: a tail call returns what its caller returns, so the two share a return type
    // and hence this ABI, and the buffer belongs to an ancestor frame that outlives the frame being
    // replaced. Elsewhere it is a fresh buffer in this function's entry block, which the caller
    // reads back with `load_out_pointer_buffer`.
    fn build_out_pointer_argument(
        &mut self,
        ret_ty: &Arc<TypeNode>,
        ret_part_tys: &[BasicTypeEnum<'c>],
        tail: bool,
    ) -> PointerValue<'c> {
        if tail {
            return self.own_out_pointer();
        }
        let buf_ty = self.out_pointer_buffer_type(ret_ty, ret_part_tys);
        self.build_alloca_at_entry(buf_ty, "out@call_lambda")
    }

    // The out-pointer parameter of the function being generated, the buffer its result is written
    // through. A function whose result goes through an out-pointer returns `void` and takes the
    // pointer before every other parameter (see `lambda_function_type`). Checked under develop mode
    // (the unit tests).
    fn own_out_pointer(&self) -> PointerValue<'c> {
        let func = self.current_function();
        if self.config.develop_mode {
            assert!(
                func.get_type().get_return_type().is_none() && func.count_params() >= 1,
                "`{}` returns through an out-pointer: it must return `void` and take it first",
                func.get_name().to_str().unwrap()
            );
        }
        func.get_nth_param(0).unwrap().into_pointer_value()
    }

    // Read back the parts a callee wrote through the out-pointer, as the object of type `ret_ty`
    // it returned.
    fn load_out_pointer_buffer(
        &mut self,
        out_ptr: PointerValue<'c>,
        ret_part_tys: &[BasicTypeEnum<'c>],
        ret_ty: Arc<TypeNode>,
    ) -> Object<'c> {
        let buf_ty = self.out_pointer_buffer_type(&ret_ty, ret_part_tys);
        let parts: Vec<BasicValueEnum<'c>> = ret_part_tys
            .iter()
            .enumerate()
            .map(|(i, part_ty)| {
                let part_ptr = self
                    .builder()
                    .build_struct_gep(buf_ty, out_ptr, i as u32, "out_part_ptr")
                    .unwrap();
                self.builder()
                    .build_load(*part_ty, part_ptr, "load_out_part")
                    .unwrap()
            })
            .collect();
        Object::from_parts(parts, ret_ty, self)
    }

    // Whether a function returning `part_tys` takes an out-pointer for its result on this module's
    // target (see `return_abi`).
    pub fn returns_through_out_pointer(&self, part_tys: &[BasicTypeEnum<'c>]) -> bool {
        returns_through_out_pointer(part_tys, self.return_registers)
    }

    // The convention every Fix lambda in this module is defined and called with (see `return_abi`).
    pub fn lambda_calling_convention(&self) -> u32 {
        self.lambda_calling_convention
    }

    /// The function the builder is positioned in, which is the one being generated.
    pub fn current_function(&self) -> FunctionValue<'c> {
        self.builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
    }

    // Build an `undef` constant of the given basic type.
    pub fn get_undef(ty: &BasicTypeEnum<'c>) -> BasicValueEnum<'c> {
        match ty {
            BasicTypeEnum::IntType(ty) => ty.get_undef().as_basic_value_enum(),
            BasicTypeEnum::FloatType(ty) => ty.get_undef().as_basic_value_enum(),
            BasicTypeEnum::PointerType(ty) => ty.get_undef().as_basic_value_enum(),
            BasicTypeEnum::VectorType(ty) => ty.get_undef().as_basic_value_enum(),
            BasicTypeEnum::StructType(ty) => ty.get_undef().as_basic_value_enum(),
            BasicTypeEnum::ArrayType(ty) => ty.get_undef().as_basic_value_enum(),
        }
    }

    // Whether `ty` occupies no storage, such as an empty union's `[0 x i8]` payload. A zero-sized
    // value carries no information, so the part helpers drop it: it yields no part (no phi, no ABI
    // slot) and is rebuilt as `undef`. A phi of a zero-sized aggregate also crashes LLVM's
    // AArch64 GlobalISel, so dropping it keeps `-O none` codegen valid there.
    fn is_zero_sized(&self, ty: BasicTypeEnum<'c>) -> bool {
        self.target_data.get_bit_size(&ty) == 0
    }

    // Whether `ty` holds more than `limit` scalars, counting through nested structs. A non-struct
    // type is one scalar -- an array included, however many elements it holds -- and a zero-sized
    // type is none; see `MAX_SPLIT_SCALARS` for what that count is of.
    //
    // A type whose fields nest holds a number of scalars exponential in the nesting depth, so the
    // count stops as soon as it settles the answer: counting the rest would cost what the limit is
    // there to bound.
    fn holds_more_scalars_than(&self, ty: BasicTypeEnum<'c>, limit: usize) -> bool {
        self.scalar_count_until_over(ty, limit) > limit
    }

    // The number of scalars `ty` holds, or some number above `limit` once the count passes it: the
    // descent stops as soon as the limit is settled.
    fn scalar_count_until_over(&self, ty: BasicTypeEnum<'c>, limit: usize) -> usize {
        if self.is_zero_sized(ty) {
            return 0;
        }
        match ty {
            BasicTypeEnum::StructType(st) => {
                let mut count = 0;
                for i in 0..st.count_fields() {
                    if count > limit {
                        break;
                    }
                    count +=
                        self.scalar_count_until_over(st.get_field_type_at_index(i).unwrap(), limit);
                }
                count
            }
            _ => 1,
        }
    }

    // Whether a value of `ty` is carried as one aggregate rather than split into the parts of its
    // fields: it holds more scalars than `Configuration::max_split_scalars`.
    //
    // A value of one scalar is carried as that scalar however low the limit is set: there is no
    // aggregate to keep together, and a funptr is carried as its bare function pointer rather than
    // as the one-field struct it is laid out as.
    pub fn is_carried_whole(&self, ty: BasicTypeEnum<'c>) -> bool {
        let limit = self.config.max_split_scalars.max(1);
        matches!(ty, BasicTypeEnum::StructType(_))
            && !self.is_zero_sized(ty)
            && self.holds_more_scalars_than(ty, limit)
    }

    // Split an embedded type into the parts a value of it is carried as: the scalars of its nested
    // structs, except that a struct wide enough for `is_carried_whole` is one part of its own. A
    // non-struct type is one part, and a zero-sized type is none.
    //
    // Splitting an unbox struct across a function boundary, rather than passing one aggregate, keeps
    // a loop-carried field (such as an `Array`'s `@size`) visible to LLVM's value analyses: the
    // recursive `fold`/`loop` tail call then carries scalar phis instead of an opaque aggregate phi,
    // so the per-element bounds check folds away and the loop vectorizes. The limit is what stops a
    // deeply nested type from paying one LLVM value per scalar; see `Configuration::max_split_scalars`.
    pub fn type_parts(&self, ty: BasicTypeEnum<'c>) -> Vec<BasicTypeEnum<'c>> {
        if self.is_carried_whole(ty) {
            return vec![ty];
        }
        self.split_type_parts(ty)
    }

    // The parts of a type the caller has found to be split. Its fields are split as well, so the
    // descent asks no further: a struct holds the scalars of its fields and no fewer, so a field of
    // a type within the limit is within it too.
    fn split_type_parts(&self, ty: BasicTypeEnum<'c>) -> Vec<BasicTypeEnum<'c>> {
        if self.is_zero_sized(ty) {
            return vec![];
        }
        match ty {
            BasicTypeEnum::StructType(st) => (0..st.count_fields())
                .flat_map(|i| self.split_type_parts(st.get_field_type_at_index(i).unwrap()))
                .collect(),
            _ => vec![ty],
        }
    }

    // The number of parts `ty` splits into under `type_parts`, without allocating the list of their
    // types.
    pub fn part_count(&self, ty: BasicTypeEnum<'c>) -> usize {
        if self.is_carried_whole(ty) {
            return 1;
        }
        self.split_part_count(ty)
    }

    // The part count of a type the caller has found to be split; see `split_type_parts`.
    fn split_part_count(&self, ty: BasicTypeEnum<'c>) -> usize {
        if self.is_zero_sized(ty) {
            return 0;
        }
        match ty {
            BasicTypeEnum::StructType(st) => (0..st.count_fields())
                .map(|i| self.split_part_count(st.get_field_type_at_index(i).unwrap()))
                .sum(),
            _ => 1,
        }
    }

    // The half-open range `[offset, offset + count)` of parts that field `field_idx` of `struct_ty`
    // occupies within the struct's part list, so a split value can address one field without
    // materializing the aggregate. `offset` is the part count of the preceding fields. A struct
    // carried whole has no such range -- its fields live inside its one part -- so this is for a
    // struct `is_carried_whole` rejects.
    pub fn field_part_range(&self, struct_ty: StructType<'c>, field_idx: u32) -> (usize, usize) {
        let offset: usize = (0..field_idx)
            .map(|i| self.part_count(struct_ty.get_field_type_at_index(i).unwrap()))
            .sum();
        let count = self.part_count(struct_ty.get_field_type_at_index(field_idx).unwrap());
        (offset, count)
    }

    // The parts a value is carried as, in the order of `type_parts` on its type, emitting an
    // `extractvalue` per struct field at the current insert position. A zero-sized value yields no
    // part, and a value carried whole is one part already.
    pub fn value_parts(&self, val: BasicValueEnum<'c>) -> Vec<BasicValueEnum<'c>> {
        if self.is_carried_whole(val.get_type()) {
            return vec![val];
        }
        self.split_value_parts(val)
    }

    // The parts of a value whose type the caller has found to be split; see `split_type_parts`.
    fn split_value_parts(&self, val: BasicValueEnum<'c>) -> Vec<BasicValueEnum<'c>> {
        if self.is_zero_sized(val.get_type()) {
            return vec![];
        }
        match val {
            BasicValueEnum::StructValue(sv) => (0..sv.get_type().count_fields())
                .flat_map(|i| {
                    let field = self
                        .builder()
                        .build_extract_value(sv, i, "split_part")
                        .unwrap();
                    self.split_value_parts(field)
                })
                .collect(),
            _ => vec![val],
        }
    }

    // Reassemble a value of `ty` from a part iterator produced in `type_parts` order, emitting an
    // `insertvalue` per struct field. The inverse of `value_parts`. A zero-sized type consumes
    // no part and is rebuilt as `undef`; a type carried whole consumes the one part that is its
    // value.
    pub fn assemble_from_parts(
        &self,
        ty: BasicTypeEnum<'c>,
        parts: &mut impl Iterator<Item = BasicValueEnum<'c>>,
    ) -> BasicValueEnum<'c> {
        if self.is_carried_whole(ty) {
            return parts.next().expect("too few parts to assemble the value");
        }
        self.assemble_split_parts(ty, parts)
    }

    // Reassemble a value whose type the caller has found to be split; see `split_type_parts`.
    fn assemble_split_parts(
        &self,
        ty: BasicTypeEnum<'c>,
        parts: &mut impl Iterator<Item = BasicValueEnum<'c>>,
    ) -> BasicValueEnum<'c> {
        if self.is_zero_sized(ty) {
            return Self::get_undef(&ty);
        }
        match ty {
            BasicTypeEnum::StructType(st) => {
                let mut val = st.get_undef();
                for i in 0..st.count_fields() {
                    let field_ty = st.get_field_type_at_index(i).unwrap();
                    let field = self.assemble_split_parts(field_ty, parts);
                    val = self
                        .builder()
                        .build_insert_value(val, field, i, "assemble_part")
                        .unwrap()
                        .into_struct_value();
                }
                val.as_basic_value_enum()
            }
            _ => parts.next().expect("too few parts to assemble the value"),
        }
    }

    // Merge objects of one type across predecessor edges as one phi per part, rather than as a
    // single aggregate phi. LLVM's value analyses see through the per-part phis where they cannot see
    // through an aggregate one, so a loop-carried field (an `Array`'s `@size`) exposed this way lets
    // the bounds check fold and the loop vectorize. Each incoming object holds its parts directly, so
    // no aggregate is ever formed here. Every incoming's parts must be available on its edge (defined
    // in, or dominating, its predecessor block), and every predecessor must already have its
    // terminator; the phis are placed at the current insert block.
    //
    // A zero-sized part (an empty union's `[0 x i8]` payload) never reaches here: `parts()` excludes
    // it, since `value_parts` drops it. So no per-part phi is ever a zero-sized one
    // (which would crash AArch64 GlobalISel), and a wholly zero-sized value has no part and merges
    // to an empty object with no phi at all — the zero-sized part is rebuilt on materialization.
    pub fn build_object_phi(
        &mut self,
        incomings: &[(Object<'c>, BasicBlock<'c>)],
        name: &str,
    ) -> Object<'c> {
        let ty = incomings[0].0.ty.clone();
        let part_count = incomings[0].0.parts().len();
        // Every incoming merges an object of the same type, so their parts line up one-to-one.
        // Checked under develop mode (the unit tests).
        if self.config.develop_mode {
            for (obj, _) in incomings {
                assert_eq!(obj.ty, ty, "build_object_phi incomings must share one type");
                assert_eq!(
                    obj.parts().len(),
                    part_count,
                    "build_object_phi incomings must share one part count"
                );
            }
        }
        // One phi per part, built consecutively so the block's phis stay contiguous at its top.
        let part_phis: Vec<BasicValueEnum<'c>> = (0..part_count)
            .map(|j| {
                let part_ty = incomings[0].0.parts()[j].get_type();
                let phi = self.builder().build_phi(part_ty, name).unwrap();
                for (obj, bb) in incomings {
                    phi.add_incoming(&[(&obj.parts()[j], *bb)]);
                }
                phi.as_basic_value()
            })
            .collect();
        Object::from_parts(part_phis, ty, self)
    }

    /// Define (once per module) and call the per-type RC helper `<prefix>_<hash>` for `obj`. The
    /// object is passed as its parts rather than as one aggregate (see `lambda_function_type`), so
    /// no aggregate is materialized across the call; `build_body` emits the retain / release / mark
    /// work on the object reassembled from those parts inside the helper.
    fn emit_rc_helper_call(
        &mut self,
        obj: Object<'c>,
        prefix: &str,
        call_name: &str,
        build_body: impl FnOnce(&mut Self, Object<'c>),
    ) {
        let func_name = format!("{}_{}", prefix, obj.ty.hash());
        let func = if let Some(func) = self.module.get_function(&func_name) {
            func
        } else {
            let embedded = obj.ty.get_embedded_type(self);
            let part_tys = self.type_parts(embedded);
            let param_tys = part_tys
                .iter()
                .map(|t| (*t).into())
                .collect::<Vec<BasicMetadataTypeEnum>>();
            let func = self.module.add_function(
                &func_name,
                self.context.void_type().fn_type(&param_tys, false),
                Some(Linkage::Internal),
            );
            let bb = self.context.append_basic_block(func, "entry");
            let _builder_guard = self.push_builder();
            self.builder().position_at_end(bb);
            let parts = (0..part_tys.len())
                .map(|i| func.get_nth_param(i as u32).unwrap())
                .collect::<Vec<_>>();
            let obj = Object::from_parts(parts, obj.ty.clone(), self);
            build_body(self, obj);
            self.builder().build_return(None).unwrap();
            func
        };

        let args = obj.part_call_args();
        self.builder().build_call(func, &args, call_name).unwrap();
    }

    /// Retain `obj`: increment the reference count of every boxed object it owns, once.
    pub fn retain(&mut self, obj: Object<'c>, state: RcState) {
        let one = self.context.i64_type().const_int(1, false);
        let prefix = format!("retain{}", state.name_suffix());
        self.emit_rc_helper_call(obj, &prefix, "call_retain", move |gc, obj| {
            gc.build_retain(obj, one, state);
        });
    }

    /// Emit `body` where the boxed object is known to be non-null. A dynamic object can be null, so
    /// the body goes on the non-null side of a null check and control rejoins after it; any other
    /// boxed object is never null, so the body is emitted where the caller stands.
    ///
    /// # Arguments
    /// * `tag` — suffix distinguishing the two basic blocks the null check adds from those of
    ///   another null check in the same function.
    fn build_if_nonnull(&mut self, obj: &Object<'c>, tag: &str, body: impl FnOnce(&mut Self)) {
        if !obj.is_dynamic_object() {
            body(self);
            return;
        }
        let current_func = self.current_function();
        let nonnull_bb = self
            .context
            .append_basic_block(current_func, &format!("nonnull_bb@{}", tag));
        let cont_bb = self
            .context
            .append_basic_block(current_func, &format!("cont_bb@{}", tag));

        // Branch to `nonnull_bb` if the object is not null.
        let is_null = obj.is_null(self);
        self.builder()
            .build_conditional_branch(is_null, cont_bb, nonnull_bb)
            .unwrap();

        self.builder().position_at_end(nonnull_bb);
        body(self);
        self.builder().build_unconditional_branch(cont_bb).unwrap();
        self.builder().position_at_end(cont_bb);
    }

    /// Retain an object `amount` times: every boxed leaf reached has its reference count increased
    /// by `amount`, an i64 count.
    pub fn build_retain(&mut self, obj: Object<'c>, amount: IntValue<'c>, state: RcState) {
        if obj.is_box(self.type_env()) {
            self.build_if_nonnull(&obj, "retain", |gc| {
                // Increment the reference count of the (now known non-null) boxed object.
                gc.retain_nonnull_boxed(&obj, amount, state);
            });
        } else {
            // When the object is unboxed,
            let obj_type = ty_to_object_ty(&obj.ty, &vec![], self.type_env());
            for (i, ft) in obj_type.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::ControlBlock => unreachable!(),
                    ObjectFieldType::TraverseFunction => unreachable!(),
                    ObjectFieldType::LambdaFunction(_) => {}
                    ObjectFieldType::Ptr => {}
                    ObjectFieldType::I8 => {}
                    ObjectFieldType::U8 => {}
                    ObjectFieldType::I16 => {}
                    ObjectFieldType::U16 => {}
                    ObjectFieldType::I32 => {}
                    ObjectFieldType::U32 => {}
                    ObjectFieldType::I64 => {}
                    ObjectFieldType::U64 => {}
                    ObjectFieldType::F32 => {}
                    ObjectFieldType::F64 => {}
                    ObjectFieldType::SubObject(subty, is_punched) => {
                        if *is_punched {
                            continue;
                        }
                        let subval = obj.extract_field(self, i as u32);
                        let subobj = Object::new(subval, subty.clone(), self);
                        if is_const_one(amount) {
                            self.retain(subobj, state);
                        } else {
                            self.build_retain(subobj, amount, state);
                        }
                    }
                    // The storage buffer appears only inside the boxed `#ArrayStorage`, whose retain
                    // bumps its control block rather than descending into fields, so it is never
                    // reached here (like `Array`).
                    ObjectFieldType::ArrayStorageBuf(_) => unreachable!(),
                    ObjectFieldType::UnionBuf(_) => {
                        ObjectFieldType::retain_union(self, obj.clone(), amount, state);
                    }
                    ObjectFieldType::UnionTag => {}
                    ObjectFieldType::Array(_) => unreachable!(),
                }
            }
        }
    }

    /// Increment by `amount` the reference count of a boxed object, in the way its refcount state
    /// calls for. The caller guarantees the object is a non-null boxed pointer (e.g. a non-empty
    /// capture object).
    pub(crate) fn retain_nonnull_boxed(
        &mut self,
        obj: &Object<'c>,
        amount: IntValue<'c>,
        state: RcState,
    ) {
        let obj_ptr = obj.value(self).into_pointer_value();
        // The refcount is narrower than the i64 count, so bring the amount to its width. A constant
        // 1 folds to a constant, leaving the single-retain code unchanged.
        let amount = self
            .builder()
            .build_int_truncate(amount, refcnt_type(self.context), "retain_amount")
            .unwrap();
        if !state.dispatches() {
            // A local object is counted in place: no state load, no branch, no continuation block.
            self.build_assert_refcnt_state_local(obj_ptr);
            let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
            let old_refcnt = self
                .builder()
                .build_load(refcnt_type(self.context), ptr_to_refcnt, "")
                .unwrap()
                .into_int_value();
            let new_refcnt = self
                .builder()
                .build_int_nsw_add(old_refcnt, amount, "")
                .unwrap();
            self.builder()
                .build_store(ptr_to_refcnt, new_refcnt)
                .unwrap();
            return;
        }
        let current_func = self.current_function();
        let cont_bb = self
            .context
            .append_basic_block(current_func, "cont_bb@retain_nonnull");

        // Branch by refcnt_state.
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr, state);

        // In `local_bb`, increment refcnt and jump to `cont_bb`.
        self.builder().position_at_end(local_bb);
        let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
        let old_refcnt_local = self
            .builder()
            .build_load(refcnt_type(self.context), ptr_to_refcnt, "")
            .unwrap()
            .into_int_value();
        let new_refcnt = self
            .builder()
            .build_int_nsw_add(old_refcnt_local, amount, "")
            .unwrap();
        self.builder()
            .build_store(ptr_to_refcnt, new_refcnt)
            .unwrap();
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        // In `threaded_bb`, increment refcnt atomically and jump to `cont_bb`. An increment hands
        // nothing over to another thread and reads nothing another thread wrote, so it carries no
        // ordering of its own.
        if let Some(threaded_bb) = threaded_bb {
            self.builder().position_at_end(threaded_bb);
            let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
            let _old_refcnt_threaded = self
                .builder()
                .build_atomicrmw(
                    AtomicRMWBinOp::Add,
                    ptr_to_refcnt,
                    amount,
                    AtomicOrdering::Monotonic,
                )
                .unwrap();
            self.builder().build_unconditional_branch(cont_bb).unwrap();
        }

        // In `global_bb`, there is no refcount to update; jump to `cont_bb`.
        self.builder()
            .position_at_end(global_bb.expect("the runtime dispatch always has a global case"));
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        self.builder().position_at_end(cont_bb);
    }

    /// Release or mark a non-null boxed object, processing the references it owns with the
    /// traverser generated for its type.
    fn build_release_mark_nonnull_boxed(
        &mut self,
        obj: &Object<'c>,
        work: TraverserWorkType,
        state: RcState,
    ) {
        let obj_for_refs = obj.clone();
        self.build_release_mark_nonnull_boxed_with(obj, work, state, move |gc| {
            gc.traverse_boxed_refs(&obj_for_refs, work)
        });
    }

    /// Release or mark a non-null boxed object, calling `traverse_refs` to process the references
    /// it owns. `traverse_refs` stands where the traverser generated for the object's type would:
    /// on the release path once the count reaches zero, and on a mark path once the object itself
    /// is marked.
    ///
    /// # Arguments
    /// * `state` — what is known of the object's refcount state, which the release path dispatches
    ///   on. A mark reads the state from the object itself, whatever the caller knows of it.
    pub(crate) fn build_release_mark_nonnull_boxed_with(
        &mut self,
        obj: &Object<'c>,
        work: TraverserWorkType,
        state: RcState,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        // If the work is release, and the object's type is Std::Destructor, then call destructor when the refcnt is one.
        if work == TraverserWorkType::release() && obj.is_destructor_object() {
            // Branch by whether or not the reference counter is one.
            let obj_ptr = obj.value(self).into_pointer_value();
            // Whether the object is uniquely owned is read from its refcount state, whatever the
            // caller knows of it.
            let (unique_bb, shared_bb) = self.build_branch_by_is_unique(obj_ptr, RcState::Unknown);

            // If reference counter is one, call destructor.
            self.builder().position_at_end(unique_bb);
            let value = ObjectFieldType::move_out_struct_field(
                self,
                obj,
                DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
            );
            let dtor =
                ObjectFieldType::move_out_struct_field(self, obj, DESTRUCTOR_OBJECT_DTOR_FIELD_IDX);
            let one = self.context.i64_type().const_int(1, false);
            self.build_retain(dtor.clone(), one, RcState::Unknown);
            let io_act = self.apply_lambda(dtor, vec![value], false).unwrap();
            let res = run_io_or_ios_runner(self, &io_act);
            ObjectFieldType::move_into_struct_field(
                self,
                obj.clone(), // Since `obj` is boxed, it is ok to clone it and discard the result of `move_into_struct_field`.
                DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
                &res,
            );
            self.builder()
                .build_unconditional_branch(shared_bb)
                .unwrap();

            self.builder().position_at_end(shared_bb);
        }

        if work == TraverserWorkType::release() {
            self.build_release_boxed_with(obj, state, traverse_refs);
        } else {
            self.build_mark_boxed_with(obj, work, traverse_refs);
        }
    }

    /// Perform `work` — release, mark-global or mark-threaded — on every boxed object `obj` owns.
    pub fn build_release_mark(&mut self, obj: Object<'c>, work: TraverserWorkType, state: RcState) {
        if obj.is_box(self.type_env()) {
            self.build_if_nonnull(&obj, "release_mark", |gc| {
                gc.build_release_mark_nonnull_boxed(&obj, work, state);
            });
        } else if obj.is_funptr() {
            // Nothing to do for function pointers.
        } else {
            // Unboxed case (inlude lambda object).
            match create_traverser(&obj.ty, &vec![], self, Some(work), state) {
                Some(trav) => {
                    // Pass the object as its parts, matching `traverser_type`.
                    let args = obj.part_call_args();
                    self.builder()
                        .build_call(trav, &args, "call_traverser_of_unboxed")
                        .unwrap();
                }
                None => {}
            }
        }
    }

    /// Traverse a non-null boxed object's owned references (its elements / fields) for `work`
    /// (release / mark). A dynamic object carries its traverser and is called through it; any other
    /// object is traversed by the function generated for its type.
    fn traverse_boxed_refs(&mut self, obj: &Object<'c>, work: TraverserWorkType) {
        let obj_ptr = obj.value(self).into_pointer_value();
        if obj.is_dynamic_object() {
            let trav = obj.extract_trav_from_dynamic(self);
            let trav_ty = traverser_type(self, &obj.ty, true);
            self.builder()
                .build_indirect_call(
                    trav_ty,
                    trav,
                    &[
                        obj_ptr.into(),
                        traverser_work_type(self.context)
                            .const_int(work.0 as u64, false)
                            .into(),
                    ],
                    "call_trav",
                )
                .unwrap();
        } else {
            // A boxed object always has a traverser: `create_traverser` declines only a dynamic
            // object without a capture and a fully unboxed type.
            // The children of a boxed object are traversed when its count reaches zero, and what
            // is known about the object itself says nothing about them, so they keep the dispatch.
            let trav = create_traverser(&obj.ty, &vec![], self, Some(work), RcState::Unknown)
                .unwrap_or_else(|| {
                    panic!("No traverser for the boxed type {}.", obj.ty.to_string())
                });
            self.builder()
                .build_call(trav, &[obj_ptr.into()], "call_trav")
                .unwrap();
        }
    }

    /// Release a non-null boxed object, emitting `traverse_refs` to release the references it owns
    /// once the refcount reaches zero, before the object is freed.
    fn build_release_boxed_with(
        &mut self,
        obj: &Object<'c>,
        state: RcState,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        // Get pointer to the object.
        let obj_ptr = obj.value(self).into_pointer_value();

        // Branch by refcnt_state.
        let current_func = self.current_function();
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr, state);
        let destruction_bb = self
            .context
            .append_basic_block(current_func, "destruction_bb");
        let end_bb = self
            .context
            .append_basic_block(current_func, "end_release_bb");

        // Implement local_bb.
        self.builder().position_at_end(local_bb);
        let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);

        // Decrement refcnt.
        let old_refcnt = self
            .builder()
            .build_load(refcnt_type(self.context), ptr_to_refcnt, "")
            .unwrap()
            .into_int_value();
        let new_refcnt = self
            .builder()
            .build_int_nsw_sub(
                old_refcnt,
                refcnt_type(self.context).const_int(1, false).into(),
                "",
            )
            .unwrap();
        self.builder()
            .build_store(ptr_to_refcnt, new_refcnt)
            .unwrap();

        // Branch to `destruction_bb` if old_refcnt is one.
        let is_refcnt_one = self
            .builder()
            .build_int_compare(
                IntPredicate::EQ,
                old_refcnt,
                refcnt_type(self.context).const_int(1, false),
                "is_refcnt_one",
            )
            .unwrap();
        self.builder()
            .build_conditional_branch(is_refcnt_one, destruction_bb, end_bb)
            .unwrap();

        // Implement threaded_bb.
        if threaded_bb.is_some() {
            let threaded_bb = threaded_bb.unwrap();

            self.builder().position_at_end(threaded_bb);
            let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
            // Decrement refcnt atomically. The decrement acquires as well as releases, so that the
            // thread that brings the count to zero sees every write the other holders made. Keep
            // the acquire in the read-modify-write: ThreadSanitizer draws no happens-before edge
            // from a standalone `fence acquire`, so an acquire moved into one on the way to
            // destruction leaves the code correct while making the race detector report the
            // destructor's reads as racing with those writes. The acquire is free on x86-64, where
            // a `lock`-prefixed read-modify-write already orders both ways; on AArch64 it costs an
            // acquire on every decrement and saves a `dmb` on the destruction path.
            let old_refcnt = self
                .builder()
                .build_atomicrmw(
                    AtomicRMWBinOp::Sub,
                    ptr_to_refcnt,
                    refcnt_type(self.context).const_int(1, false),
                    AtomicOrdering::AcquireRelease,
                )
                .unwrap();

            // Destroy the object if old_refcnt is one. The decrement carries the ordering the
            // destruction needs, so this path branches into `destruction_bb` directly, as the
            // other modes do.
            let is_refcnt_one = self
                .builder()
                .build_int_compare(
                    IntPredicate::EQ,
                    old_refcnt,
                    refcnt_type(self.context).const_int(1, false),
                    "is_refcnt_one",
                )
                .unwrap();
            self.builder()
                .build_conditional_branch(is_refcnt_one, destruction_bb, end_bb)
                .unwrap();
        }

        // Implement `destruction_bb`
        self.builder().position_at_end(destruction_bb);

        // Release the object's owned references, then free it.
        traverse_refs(self);
        build_free_boxed(self, obj_ptr, &obj.ty);
        self.builder().build_unconditional_branch(end_bb).unwrap();

        // Implement global_bb.
        if let Some(global_bb) = global_bb {
            self.builder().position_at_end(global_bb);
            self.builder().build_unconditional_branch(end_bb).unwrap();
        }

        self.builder().position_at_end(end_bb);
    }

    /// Mark a boxed object, and through `traverse_refs` every object it owns.
    ///
    /// An object that already carries the mark ends the traversal there, since it owns only
    /// objects carrying the mark or a stronger one. Each object is therefore marked once: the work
    /// is proportional to the objects a value holds, even where one of them is reached along many
    /// paths, and the traversal terminates on a cyclic graph.
    ///
    /// A marked object owns only marked objects because a write in place is the only way it gains
    /// a child, and such a write reaches a marked object through `build_branch_by_is_unique`: a
    /// global object leaves that check as shared and is cloned, and a threaded one is returned to
    /// the local state there. That check stays in place for a value made threaded, since
    /// `Std::mark_threaded` hands the value back with an `Unknown` provenance and unique-check
    /// elimination drops a check only on a value it knows to be uniquely owned (see
    /// `InlineLLVMMarkThreadedFunctionBody::result_prov`).
    fn build_mark_boxed_with(
        &mut self,
        obj: &Object<'c>,
        work: TraverserWorkType,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        assert!(
            work == TraverserWorkType::mark_global() || work == TraverserWorkType::mark_threaded()
        );
        let marks_global = work == TraverserWorkType::mark_global();

        // Get pointer to the object.
        let obj_ptr = obj.value(self).into_pointer_value();

        let current_func = self.current_function();
        let mark_bb = self
            .context
            .append_basic_block(current_func, "mark_bb@mark_boxed");
        let cont_bb = self
            .context
            .append_basic_block(current_func, "cont_bb@mark_boxed");

        // Load refcnt state.
        let refcnt_state = self.build_load_refcnt_state(obj_ptr, "refcnt_state");

        // Branch by whether or not the object carries the mark. A state covers every state below
        // it, so the object carries the mark where its state reaches the marked one.
        let mark_state = if marks_global {
            RefcntState::GLOBAL
        } else {
            RefcntState::THREADED
        };
        let is_marked = self.build_compare_refcnt_state(
            refcnt_state,
            IntPredicate::UGE,
            mark_state,
            "is_marked",
        );
        self.builder()
            .build_conditional_branch(is_marked, cont_bb, mark_bb)
            .unwrap();

        // Implement mark_bb: mark the object itself, then the objects it owns.
        self.builder().position_at_end(mark_bb);
        self.set_refcnt_state(obj_ptr, mark_state);
        traverse_refs(self);
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        // Set builder's position as preparation for following implementation.
        self.builder().position_at_end(cont_bb);
    }

    /// Release `obj`: decrement the reference count of every boxed object it owns, destroying the
    /// ones whose count reaches zero.
    pub fn release(&mut self, obj: Object<'c>, state: RcState) {
        let prefix = format!("release{}", state.name_suffix());
        self.emit_rc_helper_call(obj, &prefix, "call_release", move |gc, obj| {
            gc.build_release_mark(obj, TraverserWorkType::release(), state);
        });
    }

    /// Decrement the reference count of a boxed object, releasing what it owns and freeing it
    /// where the count reaches zero. The caller guarantees the object is a non-null boxed pointer.
    pub(crate) fn release_nonnull_boxed(&mut self, obj: &Object<'c>, state: RcState) {
        self.build_release_mark_nonnull_boxed(obj, TraverserWorkType::release(), state)
    }

    /// Put every boxed object `obj` owns into the global refcount state, in which an object is
    /// neither retained, released nor freed, so that it lives for the rest of the program.
    pub fn mark_global(&mut self, obj: Object<'c>) {
        self.emit_rc_helper_call(obj, "mark_global", "call_mark_global", |gc, obj| {
            gc.build_release_mark(obj, TraverserWorkType::mark_global(), RcState::Unknown);
        });
    }

    /// Put every boxed object `obj` owns into the threaded refcount state, where a reference count
    /// is updated atomically, so that an object can be held by several threads at once. An object
    /// already in the global state keeps it.
    pub fn mark_threaded(&mut self, obj: Object<'c>) {
        self.emit_rc_helper_call(obj, "mark_threaded", "call_mark_threaded", |gc, obj| {
            gc.build_release_mark(obj, TraverserWorkType::mark_threaded(), RcState::Unknown);
        });
    }

    /// Load the reference-count state of the boxed object at `obj_ptr`, naming the loaded value
    /// `name` in the emitted code.
    fn build_load_refcnt_state(&mut self, obj_ptr: PointerValue<'c>, name: &str) -> IntValue<'c> {
        let refcnt_state_ptr = self.get_refcnt_state_ptr(obj_ptr);
        self.builder()
            .build_load(refcnt_state_type(self.context), refcnt_state_ptr, name)
            .unwrap()
            .into_int_value()
    }

    /// Compare a loaded reference-count state against `state` under `predicate`, naming the result
    /// `name` in the emitted code.
    fn build_compare_refcnt_state(
        &self,
        refcnt_state: IntValue<'c>,
        predicate: IntPredicate,
        state: RefcntState,
        name: &str,
    ) -> IntValue<'c> {
        self.builder()
            .build_int_compare(
                predicate,
                refcnt_state,
                refcnt_state_type(self.context).const_int(state.value() as u64, false),
                name,
            )
            .unwrap()
    }

    /// Put the boxed object at `ptr` alone into `state`, leaving the objects it owns as they are.
    pub(crate) fn set_refcnt_state(&mut self, ptr: PointerValue<'c>, state: RefcntState) {
        let ptr_refcnt_state: PointerValue<'_> = self.get_refcnt_state_ptr(ptr);
        self.builder()
            .build_store(
                ptr_refcnt_state,
                refcnt_state_type(self.context).const_int(state.value() as u64, false),
            )
            .unwrap();
    }

    /// Emit code writing `string` to stderr, followed by a newline.
    fn eprint(&mut self, string: &str) {
        let string_ptr = self.add_global_string(string);
        let string_ptr = string_ptr.as_pointer_value();
        self.call_runtime(RUNTIME_EPRINTLN, &[string_ptr.into()]);
    }

    /// Emit code writing `string` to stderr and aborting the program.
    pub fn panic(&mut self, string: &str) {
        self.eprint(string);
        self.call_runtime(RUNTIME_ABORT, &[]);
    }

    /// Emit a call to the runtime function named `func_name`, which the module must already
    /// declare.
    pub fn call_runtime(
        &self,
        func_name: &str,
        args: &[BasicMetadataValueEnum<'c>],
    ) -> CallSiteValue<'c> {
        let func = self
            .module
            .get_function(func_name)
            .unwrap_or_else(|| panic!("Runtime function not found: {}", func_name));
        self.builder()
            .build_call(func, args, "call_runtime")
            .unwrap()
    }

    // Build return instruction if `tail` is true.
    pub fn build_tail(&mut self, obj: Object<'c>, tail: bool) -> Option<Object<'c>> {
        if tail {
            self.build_return_object(obj);
            None
        } else {
            Some(obj)
        }
    }

    // Return `obj` from the current function under the split return ABI (see
    // `lambda_function_type`): its parts are packed into the flat return value — `void` for none, the
    // bare part for one, a flat struct built with one `insertvalue` per part for several. Parts too
    // wide for the return registers are stored through the function's out-pointer instead.
    pub fn build_return_object(&mut self, obj: Object<'c>) {
        let parts: Vec<BasicValueEnum<'c>> = obj.parts().to_vec();
        let part_tys: Vec<BasicTypeEnum<'c>> = parts.iter().map(|p| p.get_type()).collect();
        if self.returns_through_out_pointer(&part_tys) {
            let out_ptr = self.own_out_pointer();
            let buf_ty = self.out_pointer_buffer_type(&obj.ty, &part_tys);
            for (i, part) in parts.iter().enumerate() {
                let part_ptr = self
                    .builder()
                    .build_struct_gep(buf_ty, out_ptr, i as u32, "out_part_ptr")
                    .unwrap();
                self.builder().build_store(part_ptr, *part).unwrap();
            }
            self.builder().build_return(None).unwrap();
            return;
        }
        match parts.len() {
            0 => {
                self.builder().build_return(None).unwrap();
            }
            1 => {
                self.builder().build_return(Some(&parts[0])).unwrap();
            }
            _ => {
                let struct_ty = self.context.struct_type(&part_tys, false);
                let mut val = struct_ty.get_undef();
                for (i, part) in parts.iter().enumerate() {
                    val = self
                        .builder()
                        .build_insert_value(val, *part, i as u32, "pack_return")
                        .unwrap()
                        .into_struct_value();
                }
                self.builder().build_return(Some(&val)).unwrap();
            }
        }
    }

    // Reconstruct the object a call returns under the split return ABI: a `void` result carries no
    // part, a single part is the result itself, and several are the fields of the flat struct it
    // returned. Which of the three applies follows from the return type's part count, which the
    // shape of the returned value would answer wrongly for a type carried whole. The inverse of
    // `build_return_object`.
    pub fn unpack_return(
        &mut self,
        call_result: Option<BasicValueEnum<'c>>,
        ret_ty: Arc<TypeNode>,
    ) -> Object<'c> {
        let embedded = ret_ty.get_embedded_type(self);
        let parts: Vec<BasicValueEnum<'c>> = match call_result {
            None => vec![],
            Some(single_part) if self.part_count(embedded) == 1 => vec![single_part],
            Some(packed) => {
                let packed = packed.into_struct_value();
                (0..packed.get_type().count_fields())
                    .map(|i| {
                        self.builder()
                            .build_extract_value(packed, i, "unpack_return")
                            .unwrap()
                    })
                    .collect()
            }
        };
        Object::from_parts(parts, ret_ty, self)
    }

    // Add the LLVM function a Fix lambda of type `fn_ty` compiles into, under `name`, and return it.
    // A funptr function is reachable from another compilation unit when compilation is separated;
    // a closure function is internal, so LLVM resolves a collision between two such names by
    // renaming one of them.
    //
    // A funptr function is also registered as the value of `name`, because the bodies that call it
    // read it by name. Registering here is what leaves no way to declare one and reach it through a
    // name that resolves to nothing.
    pub fn declare_lambda_function(
        &mut self,
        fn_ty: &Arc<TypeNode>,
        name: &FullName,
    ) -> FunctionValue<'c> {
        let llvm_fn_ty = lambda_function_type(fn_ty, self);
        let linkage = if fn_ty.is_funptr() && self.config.enable_separated_compilation() {
            Linkage::External
        } else {
            Linkage::Internal
        };
        let func =
            self.module
                .add_function(&object_file_symbol_name(name), llvm_fn_ty, Some(linkage));
        func.set_call_conventions(self.lambda_calling_convention());
        if fn_ty.is_funptr() {
            self.add_global_object(name.clone(), func, fn_ty.clone());
        }
        func
    }

    // Declare the function the program's global `name` is obtained through, register it as that
    // global's value, and return it — or `None` where the program has no global of that name. A
    // global of funptr type is the lambda's own function; any other global is reached through an
    // accessor function taking no argument and returning its value.
    //
    // The signature is built from the program's global types, which is what makes this the only way
    // to declare a global: the module that defines one and every module that calls into it read the
    // same entry, so a global has one signature everywhere it is declared. That agreement is load
    // bearing and unchecked — an accessor is reached by a direct call, so a module that declared it
    // to return a value while the defining module returns none reads an undefined value, and neither
    // the LLVM verifier nor the linker looks at it.
    pub fn declare_program_global(&mut self, name: &FullName) -> Option<FunctionValue<'c>> {
        let ty = self.global_types.get(name).cloned()?;
        if ty.is_funptr() {
            return Some(self.declare_lambda_function(&ty, name));
        }
        let acc_fn_name = global_accessor_name(name);
        let embedded_ty = ty.get_embedded_type(self);
        let acc_fn_ty = if self.sizeof(&embedded_ty) == 0 {
            self.context.void_type().fn_type(&[], false)
        } else {
            embedded_ty.fn_type(&[], false)
        };
        let acc_fn = self.module.add_function(
            &acc_fn_name,
            acc_fn_ty,
            Some(self.config.external_if_separated()),
        );
        self.add_global_object(name.clone(), acc_fn, ty);
        Some(acc_fn)
    }

    /// Give `func`, whose body is about to be emitted, its debug-info subprogram and open that
    /// subprogram as the debug scope the body is generated under. The returned guard closes the
    /// scope when it is dropped; it is `None` when the build carries no debug info.
    ///
    /// Attaching the subprogram here, where the body is created, is what keeps it off a function
    /// that has none: LLVM's verifier rejects a `!dbg` attachment on a bodyless declaration, and a
    /// module declares every global it refers to but defines only its own.
    pub fn push_debug_subprogram(
        &mut self,
        func: FunctionValue<'c>,
        span: Option<Span>,
    ) -> Option<PopDebugScopeGuard<'c>> {
        if !self.has_di() {
            return None;
        }
        let fn_name = func.get_name().to_str().unwrap().to_string();
        let subprogram = self.create_debug_subprogram(&fn_name, span);
        func.set_subprogram(subprogram);
        Some(self.push_debug_scope(Some(subprogram.as_debug_info_scope())))
    }

    /// The debug-info subprogram describing the function that carries the symbol name `fn_name`
    /// and is defined at `span`. A function whose source is unknown is recorded at line 0 of the
    /// file that stands for an unknown source.
    fn create_debug_subprogram(&self, fn_name: &str, span: Option<Span>) -> DISubprogram<'c> {
        let (di_builder, di_compile_unit) = self.debug_info.as_ref().unwrap();
        let line_no = if let Some(span) = span.as_ref() {
            span.start_line_no()
        } else {
            0
        };
        let file = self.create_di_file(span.map(|s| s.input));
        let subroutine_type = di_builder.create_subroutine_type(file, None, &[], 0);
        di_builder.create_function(
            di_compile_unit.as_debug_info_scope(),
            fn_name,
            None,
            file,
            line_no as u32,
            subroutine_type,
            true,
            true,
            line_no as u32,
            0,
            false,
        )
    }

    /// Attributes the instructions generated from here on to `span`, until `pop_debug_location`
    /// brings back the location that was current before. `None` leaves them without a line of
    /// their own.
    pub fn push_debug_location(&mut self, span: Option<Span>) {
        self.debug_location.push(span.clone());
        self.set_debug_location(span);
    }

    /// Drops the location pushed last, so the instructions generated from here on carry the one
    /// that was current before it.
    pub fn pop_debug_location(&mut self) {
        self.debug_location.pop();
        self.reset_debug_location();
    }

    /// Attributes the instructions the builder appends from now on to `span`, within the debug
    /// scope the code being generated belongs to. Where it belongs to no debug scope, those
    /// instructions carry no location at all.
    pub fn set_debug_location(&mut self, span: Option<Span>) {
        if let Some(debug_scope) = self.debug_scope() {
            let (line, col) = if let Some(span) = span.as_ref() {
                span.start_line_col()
            } else {
                (0, 0)
            };
            let loc = self.get_di_builder().create_debug_location(
                self.context,
                line as u32,
                col as u32,
                debug_scope,
                None,
            );
            self.builder().set_current_debug_location(loc);
        } else {
            self.builder().unset_current_debug_location();
        }
    }

    /// Attributes the instructions the builder appends from now on to the innermost span still
    /// pushed, which is the location moving the builder within a function has to restore.
    pub fn reset_debug_location(&mut self) {
        self.set_debug_location(flatten_opt(self.debug_location.last().cloned()));
    }

    /// Emit a call to a C function from already-evaluated argument objects and a pre-allocated
    /// return object. Each argument is marshalled to its C scalar (field 0), the function is called,
    /// and the result is written back into the return object (field 1 of the `(IOState, ret)` tuple
    /// when `is_io`, else field 0). A void return writes nothing.
    pub fn build_ffi_call_core(
        &mut self,
        mut obj: Object<'c>,
        fun_name: &Name,
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        is_var_args: bool,
        arg_objs: Vec<Object<'c>>,
        is_io: bool,
    ) -> Object<'c> {
        // Get c function
        let c_fun = CSignature::of_ffi_call(ret_tycon, param_tys, is_var_args)
            .get_or_declare_in_module(fun_name, self);

        // Get argment values
        let args_vals = arg_objs
            .iter()
            .map(|obj| obj.extract_field(self, 0).into())
            .collect::<Vec<_>>();

        // Call c function
        let call_site = self
            .builder()
            .build_call(c_fun, &args_vals, &format!("FFI_CALL({})", fun_name))
            .unwrap();
        match call_site.try_as_basic_value() {
            Either::Left(ret_c_val) => {
                if is_io {
                    let ret_struct_ty = type_tycon(ret_tycon).get_struct_type(self);
                    let ret_struct_val = ret_struct_ty.get_undef();
                    let ret_struct_val = self
                        .builder()
                        .build_insert_value(ret_struct_val, ret_c_val, 0, "")
                        .unwrap();
                    obj = obj.insert_field(self, 1, ret_struct_val);
                } else {
                    obj = obj.insert_field(self, 0, ret_c_val);
                }
            }
            Either::Right(_) => {}
        }

        obj
    }

    /// Project the captured value at `cap_idx` out of a closure's capture object `cap_name`,
    /// retaining it (a retain-getter).
    ///
    /// # Arguments
    /// * `cap_tys` — the types of all the captured values, which give the capture object its struct
    ///   layout.
    /// * `result_ty` — the type of the projected value.
    pub fn build_capture_project(
        &mut self,
        cap_name: &FullName,
        cap_idx: usize,
        cap_tys: &Vec<Arc<TypeNode>>,
        result_ty: &Arc<TypeNode>,
        state: RcState,
    ) -> Object<'c> {
        let cap_obj = self.get_scoped_obj_noretain(cap_name);
        let cap_obj_ty = make_dynamic_object_ty().get_object_type(cap_tys, self.type_env());
        let cap_obj_struct_ty = cap_obj_ty.to_struct_type(self);
        let cap_val = cap_obj.extract_field_as(
            self,
            cap_obj_struct_ty,
            cap_idx as u32 + DYNAMIC_OBJ_CAP_IDX,
        );
        let obj = Object::new(cap_val, result_ty.clone(), self);
        let one = self.context.i64_type().const_int(1, false);
        self.build_retain(obj.clone(), one, state);
        obj
    }

    /// Whether this module is being built with debug information.
    pub fn has_di(&self) -> bool {
        self.debug_info.is_some()
    }

    /// The builder every debug entity of this module is emitted through. The module is expected to
    /// be built with debug information.
    pub fn get_di_builder(&self) -> &DebugInfoBuilder<'c> {
        &self.debug_info.as_ref().unwrap().0
    }

    /// The compilation unit every debug entity of this module belongs to. The module is expected
    /// to be built with debug information.
    pub fn get_di_compile_unit(&self) -> &DICompileUnit<'c> {
        &self.debug_info.as_ref().unwrap().1
    }

    /// Closes the debug information of this module, resolving what was left open while it was
    /// written, once every subprogram is checked to sit on a function the module defines. A module
    /// built without debug information passes through.
    pub fn finalize_di(&self) {
        if self.has_di() {
            self.assert_no_subprogram_on_declaration();
            self.get_di_builder().finalize();
        }
    }

    /// A debug-info subprogram belongs to a function this module defines. LLVM's verifier rejects
    /// one attached to a function this module only declares, reporting every offending function at
    /// once and naming no cause; this says which function took a subprogram it should not have, at
    /// the point the attachment is still attributable to the code that made it. See
    /// `push_debug_subprogram`, which is where a subprogram is attached.
    fn assert_no_subprogram_on_declaration(&self) {
        for func in self.module.get_functions() {
            if func.count_basic_blocks() == 0 && func.get_subprogram().is_some() {
                panic_with_msg(&format!(
                    "the declaration of `{}` carries a debug info subprogram",
                    func.get_name().to_str().unwrap()
                ));
            }
        }
    }

    /// A symbol this module defines is one an object file's symbol table can hold, which is the
    /// spelling `object_file_symbol_name` gives a Fix name. A symbol carrying
    /// `SYMBOL_VERSION_SEPARATOR` is read by the linker as `symbol@version`, and stops it from
    /// building the dynamic symbol table of a shared library, so this names the offending symbol at
    /// the module that minted it, in every build whatever its output type.
    ///
    /// What the module only declares is left out: the C function `FFI_CALL` names is spelled as the
    /// library spells it, a version specifier included.
    pub fn assert_defined_symbols_fit_a_symbol_table(&self) {
        let defined_function_symbols = self
            .module
            .get_functions()
            .filter(|func| func.count_basic_blocks() > 0)
            .map(|func| func.get_name().to_str().unwrap().to_string());
        let defined_global_symbols = self
            .module
            .get_globals()
            .filter(|global| global.get_initializer().is_some())
            .map(|global| global.get_name().to_str().unwrap().to_string());
        for symbol in defined_function_symbols.chain(defined_global_symbols) {
            if symbol.contains(SYMBOL_VERSION_SEPARATOR) {
                panic_with_msg(&format!(
                    "the symbol `{}` carries `{}`, which a symbol table cannot hold",
                    symbol, SYMBOL_VERSION_SEPARATOR
                ));
            }
        }
    }

    /// The debug info record of the file `src` lives in. A source that is unknown is recorded as
    /// the file `<unknown source>`, so every debug entity has a file to point at.
    pub fn create_di_file(&self, src: Option<SourceFile>) -> DIFile<'c> {
        if let Some(src) = src {
            self.get_di_builder()
                .create_file(&src.get_file_name(), &src.get_file_dir())
        } else {
            self.get_di_builder().create_file("<unknown source>", "")
        }
    }

    /// Return the debug type identified by `key`, building it with `build` only on the first
    /// request and caching the result. A recursive type refers to itself, so `build` may ask for
    /// the same `key` again before it returns; that inner request resolves to a placeholder node
    /// which this method replaces with the finished type once `build` completes, breaking what
    /// would otherwise be unbounded recursion.
    pub fn get_or_build_di_type(
        &mut self,
        key: String,
        build: impl FnOnce(&mut Self) -> DIType<'c>,
    ) -> DIType<'c> {
        if let Some(cached) = self.di_type_cache.get(&key) {
            return *cached;
        }
        if let Some(placeholder) = self.di_type_placeholders.get(&key) {
            return placeholder.as_type();
        }
        let placeholder = unsafe {
            self.get_di_builder()
                .create_placeholder_derived_type(self.context)
        };
        self.di_type_placeholders.insert(key.clone(), placeholder);
        let real = build(self);
        // inkwell's safe `replace_placeholder_derived_type` only accepts a derived type as the
        // replacement, but a struct's debug type is a composite type, so replace through the C API
        // that inkwell itself wraps.
        unsafe {
            LLVMMetadataReplaceAllUsesWith(placeholder.as_mut_ptr(), real.as_mut_ptr());
        }
        self.di_type_placeholders.remove(&key);
        self.di_type_cache.insert(key, real);
        real
    }

    /// Records the local named `name`, holding `obj`, as a variable a debugger can inspect by that
    /// name: the value is stored into a stack slot of its own, and the debug information declares
    /// that slot to be the variable.
    pub fn create_debug_local_variable(&mut self, name: &Name, obj: &Object<'c>) {
        // Push the value on the stack.
        let obj_val = obj.value(self);
        let storage =
            self.build_alloca_at_entry(obj_val.get_type(), "alloca@create_debug_local_variable");
        self.builder().build_store(storage, obj_val).unwrap();

        let embed_ty = obj.debug_embedded_ty(self);
        let loc_var = self.get_di_builder().create_auto_variable(
            self.debug_scope().unwrap(),
            &name.to_string(),
            self.create_di_file(None), // TODO: give more good source location.
            0, // TODO: give more good source location. Should show defined location?
            embed_ty,
            true,
            0,
            0, // TODO: What is this?
        );
        self.get_di_builder().insert_declare_at_end(
            storage,
            Some(loc_var),
            None,
            self.builder().get_current_debug_location().unwrap(),
            self.builder().get_insert_block().unwrap(),
        );
    }

    /// The bits of `val` read as a value of `to_ty`. The two types may differ in size, in which case
    /// the value travels through a stack slot wide enough for both.
    pub fn bit_cast(
        &mut self,
        val: BasicValueEnum<'c>,
        to_ty: BasicTypeEnum<'c>,
    ) -> BasicValueEnum<'c> {
        let (from_ty, to_ty) = (val.get_type(), to_ty);
        if from_ty == to_ty {
            return val;
        }
        // If the types are not equal, we need to use alloca to bit cast.
        let (from_size, to_size) = (self.sizeof(&from_ty), self.sizeof(&to_ty));
        let larger_ty = if from_size > to_size { from_ty } else { to_ty };
        let ptr = self.build_alloca_at_entry(larger_ty, "alloca@bit_cast");
        self.builder().build_store(ptr, val).unwrap();
        self.builder().build_load(to_ty, ptr, "bit_cast").unwrap()
    }

    /// Add a named enum attribute (e.g. `noreturn`, `noalias`) to a function. Enum attributes
    /// must be created through their kind id; a string attribute of the same name is silently
    /// ignored by LLVM.
    pub fn add_enum_attribute(&self, func: FunctionValue<'c>, name: &str, loc: AttributeLoc) {
        let kind = enum_attribute_kind_id(name);
        func.add_attribute(loc, self.context.create_enum_attribute(kind, 0));
    }

    /// Mark a value crossing the C boundary as one the ABI extends to the unit it travels in.
    ///
    /// A C compiler puts the extension on every such parameter and result, and a Fix function
    /// reaching C carries it for the same reason: without it the reader of a promise-based ABI sees
    /// whatever the bits happen to hold. `CIntegerExtension` holds which values need one and why.
    ///
    /// Two descriptions a program writes of one C function agree on the extension at each position —
    /// that is what `Program::validate_c_function_calls` decides — so a position written twice in
    /// Fix source is written with the same attribute both times.
    pub fn add_c_integer_extension_attribute(
        &self,
        func: FunctionValue<'c>,
        loc: AttributeLoc,
        tycon: &TyCon,
    ) {
        let Some(extension) = tycon.c_integer_extension() else {
            return;
        };
        self.add_enum_attribute(func, extension.attribute_name(), loc);
    }

    /// Have every function in the module keep its frame pointer. macOS `backtrace()` walks the
    /// chain of frame pointers, so a frame that drops its own is a frame the backtrace stops at.
    pub fn add_frame_pointer_attribute_to_all_functions(&self) {
        for function in module_functions(self.module) {
            // Add "frame-pointer"="all" attribute to ensure frame pointers are always kept
            function.add_attribute(
                AttributeLoc::Function,
                self.context.create_string_attribute("frame-pointer", "all"),
            );
        }
    }
}

/// The functions `module` holds, defined and declared alike, in the order LLVM keeps them.
pub(crate) fn module_functions<'c>(module: &Module<'c>) -> impl Iterator<Item = FunctionValue<'c>> {
    successors(module.get_first_function(), |function| {
        function.get_next_function()
    })
}

/// The kind id LLVM knows the enum attribute `name` under.
///
/// An enum attribute is created from its kind id, and a name LLVM does not know yields kind id 0,
/// whose attribute every consumer ignores. Asking for an attribute that does nothing is a mistake
/// in the caller, so the lookup panics on it.
pub(crate) fn enum_attribute_kind_id(name: &str) -> u32 {
    let kind_id = Attribute::get_named_enum_kind_id(name);
    assert!(
        kind_id != 0,
        "LLVM does not know the enum attribute `{}`.",
        name
    );
    kind_id
}

/// Whether `v` is the integer constant 1. An integer the program computes at run time answers
/// `false`, whatever it turns out to hold.
pub(crate) fn is_const_one(v: IntValue) -> bool {
    v.get_zero_extended_constant() == Some(1)
}

/// The name under which the value `name` enters the symbol table of an object file.
///
/// A Fix name carries `SYMBOL_VERSION_SEPARATOR` wherever it names a field getter, and a symbol
/// table cannot hold that character, so it is written as `SYMBOL_VERSION_SEPARATOR_SUBSTITUTE`
/// here. Every symbol a Fix name reaches an object file under is written through this function,
/// which is what makes the module defining a value and the modules calling into it name it
/// identically.
///
/// # Examples
///
/// The getter of the field `x` of `Main::Point` is the Fix name `Main::Point::@x`, and enters a
/// symbol table as `Main::Point::$x`.
pub(crate) fn object_file_symbol_name(name: &FullName) -> String {
    let name = name.to_string();
    assert!(
        !name.contains(SYMBOL_VERSION_SEPARATOR_SUBSTITUTE),
        "the name `{}` carries `{}`, which stands for `{}` in a symbol table",
        name,
        SYMBOL_VERSION_SEPARATOR_SUBSTITUTE,
        SYMBOL_VERSION_SEPARATOR
    );
    name.replace(
        SYMBOL_VERSION_SEPARATOR,
        SYMBOL_VERSION_SEPARATOR_SUBSTITUTE,
    )
}

/// The two functions a module emits for one of its globals.
#[derive(Clone, Copy)]
pub(crate) struct EmittedGlobal<'c> {
    /// Reads the global: tests the initialization flag, calls `init_value` and stores what it
    /// returns on the first access, and loads the storage.
    pub accessor: FunctionValue<'c>,
    /// Computes the value of the global, and is called once in the program's life.
    pub init_value: FunctionValue<'c>,
}

/// The name of the LLVM function through which the global `name`, of a type other than funptr, is
/// obtained. It is the name every module — the one defining the global and the ones calling into
/// it — declares and looks the accessor up under.
pub(crate) fn global_accessor_name(name: &FullName) -> String {
    format!("Get#{}", object_file_symbol_name(name))
}
