// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::name::Name;
use crate::ast::program::Symbol;
use crate::ast::program::TypeEnv;
use crate::ast::types::type_tycon;
use crate::ast::types::TyCon;
use crate::ast::types::TypeNode;
use crate::configuration::Configuration;
use crate::constants::TraverserWorkType;
use crate::constants::CLOSURE_CAPTURE_IDX;
use crate::constants::CLOSURE_FUNPTR_IDX;
use crate::constants::CTRL_BLK_REFCNT_IDX;
use crate::constants::CTRL_BLK_REFCNT_STATE_IDX;
use crate::constants::DESTRUCTOR_OBJECT_DTOR_FIELD_IDX;
use crate::constants::DESTRUCTOR_OBJECT_VALUE_FIELD_IDX;
use crate::constants::DYNAMIC_OBJ_CAP_IDX;
use crate::constants::DYNAMIC_OBJ_TRAVARSER_IDX;
use crate::constants::REFCNT_STATE_GLOBAL;
use crate::constants::REFCNT_STATE_LOCAL;
use crate::constants::REFCNT_STATE_THREADED;
use crate::error::panic_with_msg;
use crate::error::panic_with_msg_src;
use crate::fixstd::builtin::make_dynamic_object_ty;
use crate::fixstd::builtin::run_io_or_ios_runner;
use crate::fixstd::runtime::RUNTIME_ABORT;
use crate::fixstd::runtime::RUNTIME_EPRINTLN;
use crate::misc::flatten_opt;
use crate::misc::Map;
use crate::object;
use crate::object::control_block_type;
use crate::object::create_traverser;
use crate::object::lambda_function_type;
use crate::object::refcnt_state_type;
use crate::object::refcnt_type;
use crate::object::traverser_type;
use crate::object::traverser_work_type;
use crate::object::ty_to_debug_embedded_ty;
use crate::object::ty_to_object_ty;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::SourceFile;
use crate::parse::sourcefile::Span;
use either::Either;
use either::Either::Left;
use either::Either::Right;
use inkwell::builder::Builder;
use inkwell::context::Context;
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
use inkwell::IntPredicate;
use inkwell::{
    basic_block::BasicBlock,
    debug_info::{
        AsDIScope, DICompileUnit, DIFile, DIScope, DISubprogram, DIType, DebugInfoBuilder,
    },
    intrinsics::Intrinsic,
    module::Linkage,
    targets::{TargetData, TargetMachine},
    types::{AnyType, BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, CallSiteValue},
};
use std::{cell::RefCell, env, sync::Arc};

#[derive(Clone)]
pub struct ScopedValue<'c> {
    accessor: ValueAccessor<'c>,
    /// Whether `get_scoped_obj` retains the value's boxed subobjects when reading it. True only for
    /// unboxed globals, which keep their own reference and so must hand out a retained copy; a boxed
    /// global is moved out on read, and local values are reference-counted by explicit RC-IR nodes.
    retain_on_read: bool,
}

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
                            let ty = ty.get_embedded_type(gc, &vec![]);
                            Generator::get_undef(&ty)
                        }
                    }
                };
                Object::new(val, ty.clone(), gc)
            }
        }
    }
}

#[derive(Clone)]
pub struct Object<'c> {
    // The value of the object.
    // For boxed type, this is a pointer to the object allocated on the heap.
    // For funcptr type, this is the function pointer.
    // For unboxed type, this is a value on the (virtual) register.
    pub value: BasicValueEnum<'c>,
    pub ty: Arc<TypeNode>,
}

impl<'c> Object<'c> {
    pub fn new<'m>(
        value: BasicValueEnum<'c>,
        ty: Arc<TypeNode>,
        gc: &mut Generator<'c, 'm>,
    ) -> Self {
        assert!(ty.free_vars().is_empty());
        let value = if ty.is_box(gc.type_env()) {
            value
        } else if ty.is_funptr() {
            value
        } else {
            // Unboxed case
            let embed_ty = ty.get_embedded_type(gc, &vec![]);
            assert_eq!(embed_ty, value.get_type());
            value
        };
        Object { value, ty }
    }

    pub fn undef<'m>(ty: Arc<TypeNode>, gc: &mut Generator<'c, 'm>) -> Self {
        let val = if ty.is_unbox(gc.type_env()) {
            ty.get_struct_type(gc, &vec![])
                .get_undef()
                .as_basic_value_enum()
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

    pub fn is_funptr(&self) -> bool {
        self.ty.is_funptr()
    }

    pub fn is_dynamic_object(&self) -> bool {
        self.ty.is_dynamic()
    }

    pub fn is_destructor_object(&self) -> bool {
        self.ty.is_destructor_object()
    }

    // pub fn opaque_boxed_ptr<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
    //     assert!(self.is_box(gc.type_env()));
    //     gc.builder().build_pointer_cast(
    //         self.value.into_pointer_value(),
    //         ptr_to_object_type(gc.context),
    //         "cast_boxed_to_opaq_ptr",
    //     )
    // }

    // pub fn opaque_funptr<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
    //     assert!(self.is_funptr());
    //     gc.builder().build_pointer_cast(
    //         self.value.into_pointer_value(),
    //         opaque_lambda_function_ptr_type(&gc.context),
    //         "cast_funcptr_to_opaq_ptr",
    //     )
    // }

    // pub fn unboxed_struct_value<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructValue<'c> {
    //     assert!(self.is_unbox(gc.type_env()));
    //     self.value.into_struct_value()
    // }

    pub fn debug_embedded_ty<'m>(&self, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
        ty_to_debug_embedded_ty(self.ty.clone(), gc)
    }

    pub fn struct_ty<'m>(&self, gc: &mut Generator<'c, 'm>) -> StructType<'c> {
        assert!(!self.is_funptr());
        ty_to_object_ty(&self.ty, &vec![], gc.type_env()).to_struct_type(gc, vec![])
    }

    // Get the pointer to the field of an boxed object.
    pub fn gep_boxed<'m>(&self, gc: &mut Generator<'c, 'm>, field_idx: u32) -> PointerValue<'c> {
        assert!(self.ty.is_box(gc.type_env()));
        let struct_ty = self.struct_ty(gc);
        let ptr = self.value.into_pointer_value();
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
            // When the object is unboxed,
            gc.builder()
                .build_extract_value(
                    self.value.into_struct_value(),
                    field_idx,
                    format!("field_{}", field_idx).as_str(),
                )
                .unwrap()
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
            // When the object is unboxed,
            let struct_val = self.value.into_struct_value();
            let struct_val = gc
                .builder()
                .build_insert_value(struct_val, val, field_idx, "insert_field")
                .unwrap()
                .as_basic_value_enum();
            self.value = struct_val;
        } else {
            // When the object is boxed,
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
        gc.builder()
            .build_is_null(self.value.into_pointer_value(), "is_null")
            .unwrap()
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
        let ptr = self.value.into_pointer_value();
        gc.builder()
            .build_struct_gep(ty, ptr, field_idx, "gep2field")
            .unwrap()
    }
}

#[derive(Default)]
pub struct Scope<'c> {
    data: Map<FullName, Vec<ScopedValue<'c>>>,
}

impl<'c> Scope<'c> {
    fn push_local(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        // TODO: add assertion that var is local (or change var to Name).
        if !self.data.contains_key(var) {
            self.data.insert(var.clone(), Default::default());
        }
        self.data.get_mut(var).unwrap().push(ScopedValue {
            accessor: ValueAccessor::Local(obj.clone()),
            retain_on_read: false,
        });
    }

    fn pop_local(&mut self, var: &FullName) {
        // TODO: add assertion that var is local (or change var to Name).
        self.data.get_mut(var).unwrap().pop();
        if self.data.get(var).unwrap().is_empty() {
            self.data.remove(var);
        }
    }

    pub fn get(&self, var: &FullName) -> ScopedValue<'c> {
        self.data.get(var).unwrap().last().unwrap().clone()
    }
}

pub struct Generator<'c, 'm> {
    pub context: &'c Context,
    pub module: &'m Module<'c>,
    builders: Arc<RefCell<Vec<Arc<Builder<'c>>>>>,
    scope: Arc<RefCell<Vec<Scope<'c>>>>,
    debug_info: Option<(DebugInfoBuilder<'c>, DICompileUnit<'c>)>,
    debug_scope: Arc<RefCell<Vec<Option<DIScope<'c>>>>>, // None implies that currently generating codes for function whose source is unknown.
    debug_location: Vec<Option<Span>>, // None implies that currently generating codes for function whose source is unknown.
    pub global: Map<FullName, ScopedValue<'c>>,
    type_env: TypeEnv,
    pub target_data: TargetData,
    pub config: Configuration,
    global_strings: Map<String, GlobalValue<'c>>,
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
    // Add a global string.
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
        let current_func = current_bb.get_parent().unwrap();
        let first_bb = current_func.get_first_basic_block().unwrap();
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

    pub fn type_env(&self) -> &TypeEnv {
        &self.type_env
    }

    pub fn sizeof(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_bit_size(ty) / 8
    }

    // The minimum alignment required to store/load a value of this type. Unlike the preferred
    // alignment, this does not over-align: an empty aggregate is 1, not 8.
    pub fn abi_alignment(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_abi_alignment(ty) as u64
    }

    pub fn ptr_size(&mut self) -> u64 {
        let ptr_ty = self.context.ptr_type(AddressSpace::from(0));
        let ptr_size = self.target_data.get_bit_size(&ptr_ty) / 8;
        assert_eq!(ptr_size, 8);
        ptr_size
    }

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

    // Create new gc.
    pub fn new(
        ctx: &'c Context,
        module: &'m Module<'c>,
        target_data: TargetData,
        config: Configuration,
        type_env: TypeEnv,
    ) -> Self {
        let ret = Self {
            context: ctx,
            module,
            builders: Arc::new(RefCell::new(vec![Arc::new(ctx.create_builder())])),
            scope: Arc::new(RefCell::new(vec![Default::default()])),
            debug_scope: Arc::new(RefCell::new(vec![])),
            debug_info: Default::default(),
            debug_location: vec![],
            global: Default::default(),
            type_env,
            target_data: target_data,
            config,
            global_strings: Map::default(),
        };
        ret
    }

    // Create debug info builders and compilation units.
    pub fn create_debug_info(&mut self) {
        let debug_metadata_version = self.context.i32_type().const_int(3, false);
        self.module.add_basic_value_flag(
            "Debug Info Version",
            inkwell::module::FlagBehavior::Warning,
            debug_metadata_version,
        );
        let cur_dir = match env::current_dir() {
            Err(why) => panic!("Failed to get current directory: {}", why),
            Ok(dir) => dir,
        };
        let (dib, dicu) = self.module.create_debug_info_builder(
            true,
            inkwell::debug_info::DWARFSourceLanguage::C,
            "NA",
            cur_dir.to_str().unwrap(),
            "fix",
            false,
            "",
            0,
            "",
            inkwell::debug_info::DWARFEmissionKind::Full,
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
        if self.global.contains_key(&name) {
            panic_with_msg(&format!("Duplicate symbol: {}", name.to_string()));
        } else {
            // A boxed global is moved out when read, so it needs no retain; an unboxed global keeps
            // its own reference, so reading it must retain its boxed subobjects.
            let retain_on_read = !ty.is_box(self.type_env());
            self.global.insert(
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

    // Push a new debug scope.
    pub fn push_debug_scope(&mut self, scope: Option<DIScope<'c>>) -> PopDebugScopeGuard<'c> {
        self.debug_scope.borrow_mut().push(scope);
        self.push_debug_location(None);
        PopDebugScopeGuard {
            scope: self.debug_scope.clone(),
        }
    }

    // Get a top debug scope.
    pub fn debug_scope(&self) -> Option<DIScope<'c>> {
        flatten_opt(self.debug_scope.borrow().last().cloned())
    }

    // Get a variable.
    pub fn get_scoped_value(&self, var: &FullName) -> ScopedValue<'c> {
        if var.is_local() {
            self.scope.borrow().last().unwrap().get(var)
        } else {
            self.global
                .get(var)
                .unwrap_or_else(|| panic!("global not found in codegen: `{}`", var.to_string()))
                .clone()
        }
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
            self.build_retain(obj.clone(), one);
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

    // Push scope.
    pub fn scope_push(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .push_local(var, obj)
    }

    // Pop scope.
    pub fn scope_pop(self: &mut Self, var: &FullName) {
        self.scope.borrow_mut().last_mut().unwrap().pop_local(var);
    }

    // Get pointer to reference counter of a given object.
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

    // Build branch by whether or not the reference counter is one.
    // Returns (unique_bb, shared_bb).
    pub fn build_branch_by_is_unique(
        self: &mut Generator<'c, 'm>,
        obj_ptr: PointerValue<'c>,
    ) -> (BasicBlock<'c>, BasicBlock<'c>) {
        let current_bb = self.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();

        let unique_bb = self.context.append_basic_block(current_func, "unique_bb");
        let shared_bb = self.context.append_basic_block(current_func, "shared_bb");

        // Branch by refcnt_state.
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr);

        // Implement local_bb.
        self.builder().position_at_end(local_bb);
        // Load refcnt.
        let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
        let refcnt = self
            .builder()
            .build_load(refcnt_type(self.context), ptr_to_refcnt, "refcnt")
            .unwrap()
            .into_int_value();
        // Jump to shared_bb if refcnt > 1.
        let one = refcnt_type(self.context).const_int(1, false);
        let is_unique: IntValue<'_> = self
            .builder()
            .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique")
            .unwrap();
        self.builder()
            .build_conditional_branch(is_unique, unique_bb, shared_bb)
            .unwrap();

        // Implement threaded_bb.
        if threaded_bb.is_some() {
            let threaded_bb = threaded_bb.clone().unwrap();
            let unique_threaded_bb = self
                .context
                .append_basic_block(current_func, "unique_threaded_bb");

            self.builder().position_at_end(threaded_bb);
            // Load refcnt atomically with monotonic ordering.
            let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
            let refcnt = self
                .builder()
                .build_load(refcnt_type(self.context), ptr_to_refcnt, "refcnt")
                .unwrap()
                .into_int_value();
            refcnt
                .as_instruction_value()
                .unwrap()
                .set_atomic_ordering(inkwell::AtomicOrdering::Monotonic)
                .expect("Set atomic ordering failed");
            // Jump to shared_bb if refcnt > 1.
            let is_unique = self
                .builder()
                .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique")
                .unwrap();
            self.builder()
                .build_conditional_branch(is_unique, unique_threaded_bb, shared_bb)
                .unwrap();

            // Implement unique_threaded_bb.
            self.builder().position_at_end(unique_threaded_bb);
            // We need to build acquire fence to avoid data race between
            // - write / modify operations which will follow in this thread and
            // - read operations done before another thread releases this object.
            self.builder()
                .build_fence(inkwell::AtomicOrdering::Acquire, 0, "")
                .unwrap();
            // Mark the object as non_threaded.
            self.mark_local_one(obj_ptr);
            // And jump to unique_bb.
            self.builder()
                .build_unconditional_branch(unique_bb)
                .unwrap();
        }

        // Implement global_bb.
        self.builder().position_at_end(global_bb);
        // Jump to shared_bb.
        self.builder()
            .build_unconditional_branch(shared_bb)
            .unwrap();

        (unique_bb, shared_bb)
    }

    // Load refcnt state and branch by the value.
    // Returns three building blocks (local_bb, threaded_bb, global_bb).
    pub fn build_branch_by_refcnt_state(
        self: &mut Generator<'c, 'm>,
        obj_ptr: PointerValue<'c>,
    ) -> (BasicBlock<'c>, Option<BasicBlock<'c>>, BasicBlock<'c>) {
        // Load refcnt_state.
        let current_bb = self.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let refcnt_state_ptr = self.get_refcnt_state_ptr(obj_ptr);
        let refcnt_state = self
            .builder()
            .build_load(
                refcnt_state_type(self.context),
                refcnt_state_ptr,
                "refcnt_state",
            )
            .unwrap()
            .into_int_value();

        // Add three basic blocks.
        let local_bb = self.context.append_basic_block(current_func, "local_bb");
        let mut threaded_bb: Option<BasicBlock<'_>> = None;
        let global_bb = self.context.append_basic_block(current_func, "global_bb");

        if !self.config.threaded {
            // In single-threaded program,

            // Check refcnt_state and jump to local_bb if it is equal to `REFCNT_STATE_LOCAL`.
            let is_refcnt_state_local = self
                .builder()
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    refcnt_state,
                    refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
                    "is_refcnt_state_local",
                )
                .unwrap();
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, global_bb)
                .unwrap();
        } else {
            // In multi-threaded program,
            let th_bb = self.context.append_basic_block(current_func, "threaded_bb");
            threaded_bb = Some(th_bb);
            let threaded_bb = threaded_bb.clone().unwrap();

            let nonlocal_bb = self.context.append_basic_block(current_func, "nonlocal_bb");

            let is_refcnt_state_local = self
                .builder()
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    refcnt_state,
                    refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
                    "is_refcnt_state_local",
                )
                .unwrap();
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, nonlocal_bb)
                .unwrap();

            // Implement nonlocal_bb.
            self.builder().position_at_end(nonlocal_bb);
            let is_refcnt_state_threaded = self
                .builder()
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    refcnt_state,
                    refcnt_state_type(self.context).const_int(REFCNT_STATE_THREADED as u64, false),
                    "is_refcnt_state_threaded",
                )
                .unwrap();
            self.builder()
                .build_conditional_branch(is_refcnt_state_threaded, threaded_bb, global_bb)
                .unwrap();
        }
        (local_bb, threaded_bb, global_bb)
    }

    // Get pointer to state of reference counter of a given object.
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

    // Take a lambda object and return function pointer.
    fn get_lambda_func_ptr(&mut self, obj: Object<'c>) -> PointerValue<'c> {
        // Get the pointer value.
        if obj.ty.is_closure() {
            obj.extract_field(self, CLOSURE_FUNPTR_IDX)
                .into_pointer_value()
        } else if obj.ty.is_funptr() {
            obj.value.into_pointer_value()
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

        // Call function pointer with arguments, CAP if closure. Each unbox-struct argument is
        // decomposed into its leaf scalars to match the flattened signature (see
        // `lambda_function_type`); the CAP pointer follows all argument scalars.
        let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
        for arg in args {
            for leaf in self.explode_to_scalar_leaves(arg.value) {
                call_args.push(leaf.into());
            }
        }
        if fun.ty.is_closure() {
            call_args.push(fun.extract_field(self, CLOSURE_CAPTURE_IDX).into());
        }

        let ret = self
            .builder()
            .build_indirect_call(func_ty, func_ptr, &call_args, "call_lambda")
            .unwrap();
        ret.set_tail_call(!self.has_di());
        let ret = match ret.try_as_basic_value() {
            Left(ret) => ret,
            Right(_) => {
                let ty = ret_ty.get_embedded_type(self, &vec![]);
                Generator::get_undef(&ty)
            }
        };
        let obj = Object::new(ret, ret_ty, self);
        self.build_tail(obj, tail)
    }

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

    // Expand an embedded type into the flat list of leaf (non-struct) scalar types it holds,
    // descending through nested structs; a non-struct type is a leaf on its own. Passing an unbox
    // struct across a function boundary as these scalars, rather than as one aggregate, keeps a
    // loop-carried field (such as an `Array`'s `@size`) visible to LLVM's value analyses: the
    // recursive `fold`/`loop` tail call then carries scalar phis instead of an opaque aggregate
    // phi, so the per-element bounds check folds away and the loop vectorizes.
    pub fn flatten_to_scalar_leaves(&self, ty: BasicTypeEnum<'c>) -> Vec<BasicTypeEnum<'c>> {
        match ty {
            BasicTypeEnum::StructType(st) => (0..st.count_fields())
                .flat_map(|i| self.flatten_to_scalar_leaves(st.get_field_type_at_index(i).unwrap()))
                .collect(),
            _ => vec![ty],
        }
    }

    // Decompose a value into its leaf scalars in the order of `flatten_to_scalar_leaves` on its
    // type, emitting an `extractvalue` per struct field at the current insert position.
    pub fn explode_to_scalar_leaves(&self, val: BasicValueEnum<'c>) -> Vec<BasicValueEnum<'c>> {
        match val {
            BasicValueEnum::StructValue(sv) => (0..sv.get_type().count_fields())
                .flat_map(|i| {
                    let field = self
                        .builder()
                        .build_extract_value(sv, i, "explode_leaf")
                        .unwrap();
                    self.explode_to_scalar_leaves(field)
                })
                .collect(),
            _ => vec![val],
        }
    }

    // Reassemble a value of `ty` from a leaf-scalar iterator produced in `flatten_to_scalar_leaves`
    // order, emitting an `insertvalue` per struct field. The inverse of `explode_to_scalar_leaves`.
    pub fn assemble_from_scalar_leaves(
        &self,
        ty: BasicTypeEnum<'c>,
        leaves: &mut impl Iterator<Item = BasicValueEnum<'c>>,
    ) -> BasicValueEnum<'c> {
        match ty {
            BasicTypeEnum::StructType(st) => {
                let mut val = st.get_undef();
                for i in 0..st.count_fields() {
                    let field_ty = st.get_field_type_at_index(i).unwrap();
                    let field = self.assemble_from_scalar_leaves(field_ty, leaves);
                    val = self
                        .builder()
                        .build_insert_value(val, field, i, "assemble_leaf")
                        .unwrap()
                        .into_struct_value();
                }
                val.as_basic_value_enum()
            }
            _ => leaves
                .next()
                .expect("too few leaf scalars to assemble the value"),
        }
    }

    // Build a phi that carries a possibly-aggregate value as one scalar phi per leaf field rather
    // than as a single aggregate phi. LLVM's value analyses see through the scalar phis where they
    // cannot see through an aggregate one, so a loop-carried field (an `Array`'s `@size`) exposed
    // this way lets the bounds check fold and the loop vectorize. The reassembly is folded away by
    // SROA/instcombine. For a non-aggregate value this is an ordinary phi. The current insert block
    // is where the phi is placed; every predecessor block must already have its terminator.
    pub fn scalar_build_phi(
        &self,
        incomings: &[(BasicValueEnum<'c>, BasicBlock<'c>)],
        name: &str,
    ) -> BasicValueEnum<'c> {
        let ty = incomings[0].0.get_type();
        if !matches!(ty, BasicTypeEnum::StructType(_)) {
            let phi = self.builder().build_phi(ty, name).unwrap();
            for (val, bb) in incomings {
                phi.add_incoming(&[(val, *bb)]);
            }
            return phi.as_basic_value();
        }
        let phi_bb = self.builder().get_insert_block().unwrap();
        // Explode each incoming value into its leaf scalars in its own predecessor block (before the
        // terminator), so each phi operand is a value available on that edge.
        let exploded: Vec<(Vec<BasicValueEnum<'c>>, BasicBlock<'c>)> = incomings
            .iter()
            .map(|(val, bb)| {
                self.builder().position_before(
                    &bb.get_terminator()
                        .expect("a predecessor feeding a phi has its terminator"),
                );
                (self.explode_to_scalar_leaves(*val), *bb)
            })
            .collect();
        // One scalar phi per leaf, all built before any reassembly so the block's phis stay
        // contiguous at its top.
        let leaf_tys = self.flatten_to_scalar_leaves(ty);
        self.builder().position_at_end(phi_bb);
        let leaf_phis: Vec<BasicValueEnum<'c>> = leaf_tys
            .iter()
            .enumerate()
            .map(|(j, lty)| {
                let phi = self.builder().build_phi(*lty, name).unwrap();
                for (leaves, bb) in &exploded {
                    phi.add_incoming(&[(&leaves[j], *bb)]);
                }
                phi.as_basic_value()
            })
            .collect();
        let mut leaves = leaf_phis.into_iter();
        self.assemble_from_scalar_leaves(ty, &mut leaves)
    }

    // Retain an object.
    pub fn retain(&mut self, obj: Object<'c>) {
        // Get retain function.
        let func_name = "retain_".to_string() + &obj.ty.hash();
        let func = if let Some(func) = self.module.get_function(&func_name) {
            func
        } else {
            let func = self.module.add_function(
                &func_name,
                self.context
                    .void_type()
                    .fn_type(&[obj.ty.get_embedded_type(self, &vec![]).into()], false),
                Some(Linkage::Internal),
            );
            let bb = self.context.append_basic_block(func, "entry");
            let _builder_guard = self.push_builder();
            self.builder().position_at_end(bb);
            let obj_val = func.get_first_param().unwrap();
            let obj = Object::new(obj_val, obj.ty.clone(), self);
            let one = self.context.i64_type().const_int(1, false);
            self.build_retain(obj, one);
            self.builder().build_return(None).unwrap();
            func
        };

        // Call retain function.
        self.builder()
            .build_call(func, &[obj.value.into()], "call_retain")
            .unwrap();
    }

    // Retain an object `amount` times: every boxed leaf reached has its reference count increased by
    // `amount` (an i64 count). Passing a constant 1 reproduces an ordinary single retain exactly, so
    // single-retain call sites stay byte-identical.
    pub fn build_retain(&mut self, obj: Object<'c>, amount: IntValue<'c>) {
        if obj.is_box(self.type_env()) {
            let cont_bb = if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                let current_bb = self.builder().get_insert_block().unwrap();
                let current_func = current_bb.get_parent().unwrap();
                let nonnull_bb = self
                    .context
                    .append_basic_block(current_func, "nonnull_bb@retain");
                let cont_bb = self
                    .context
                    .append_basic_block(current_func, "cont_bb@retain");

                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder()
                    .build_conditional_branch(is_null, cont_bb, nonnull_bb)
                    .unwrap();

                // Implement code to retain in nonnull_bb.
                self.builder().position_at_end(nonnull_bb);
                Some(cont_bb)
            } else {
                None
            };

            // Increment the reference count of the (now known non-null) boxed object.
            self.retain_nonnull_boxed(&obj, amount);

            if let Some(cont_bb) = cont_bb {
                self.builder().build_unconditional_branch(cont_bb).unwrap();
                self.builder().position_at_end(cont_bb);
            }
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
                            self.retain(subobj);
                        } else {
                            self.build_retain(subobj, amount);
                        }
                    }
                    // The storage buffer appears only inside the boxed `#ArrayStorage`, whose retain
                    // bumps its control block rather than descending into fields, so it is never
                    // reached here (like `Array`).
                    ObjectFieldType::ArrayStorageBuf(_) => unreachable!(),
                    ObjectFieldType::UnionBuf(_) => {
                        ObjectFieldType::retain_union(self, obj.clone(), amount);
                    }
                    ObjectFieldType::UnionTag => {}
                    ObjectFieldType::Array(_) => unreachable!(),
                }
            }
        }
    }

    // Increment the reference count of a non-null boxed object, according to its refcount state,
    // without the null check `build_retain` performs for a possibly-null dynamic object. The caller
    // guarantees the object is a non-null boxed pointer (e.g. a non-empty capture object).
    pub(crate) fn retain_nonnull_boxed(&mut self, obj: &Object<'c>, amount: IntValue<'c>) {
        let current_func = self
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let cont_bb = self
            .context
            .append_basic_block(current_func, "cont_bb@retain_nonnull");

        let obj_ptr = obj.value.into_pointer_value();
        // The refcount is narrower than the i64 count, so bring the amount to its width. A constant 1
        // folds to a constant, leaving the single-retain code unchanged.
        let amount = self
            .builder()
            .build_int_truncate(amount, refcnt_type(self.context), "retain_amount")
            .unwrap();
        // Branch by refcnt_state.
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr);

        // In `local_bb`, increment refcnt and jump to `cont_bb`.
        self.builder().position_at_end(local_bb);
        let old_refcnt_local = self
            .builder()
            .build_load(refcnt_type(self.context), obj_ptr, "")
            .unwrap()
            .into_int_value();
        let new_refcnt = self
            .builder()
            .build_int_nsw_add(old_refcnt_local, amount, "")
            .unwrap();
        self.builder().build_store(obj_ptr, new_refcnt).unwrap();
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        // In `threaded_bb`, increment refcnt atomically and jump to `cont_bb`.
        if let Some(threaded_bb) = threaded_bb {
            self.builder().position_at_end(threaded_bb);
            let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
            let _old_refcnt_threaded = self
                .builder()
                .build_atomicrmw(
                    inkwell::AtomicRMWBinOp::Add,
                    ptr_to_refcnt,
                    amount,
                    inkwell::AtomicOrdering::Monotonic,
                )
                .unwrap();
            self.builder().build_unconditional_branch(cont_bb).unwrap();
        }

        // In `global_bb`, there is no refcount to update; jump to `cont_bb`.
        self.builder().position_at_end(global_bb);
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        self.builder().position_at_end(cont_bb);
    }

    // Release or mark global or mark threaded nonnull boxed object.
    // Release or mark a non-null boxed object: process its owned references with the standard
    // traverser.
    fn build_release_mark_nonnull_boxed(&mut self, obj: &Object<'c>, work: TraverserWorkType) {
        let obj_for_refs = obj.clone();
        self.build_release_mark_nonnull_boxed_with(obj, work, move |gc| {
            gc.traverse_boxed_refs(&obj_for_refs, work)
        });
    }

    // Release or mark a non-null boxed object, using `traverse_refs` — in place of the type's
    // standard traverser — to process its owned references. A caller thus reuses the refcount
    // bookkeeping with a custom reference traversal.
    pub(crate) fn build_release_mark_nonnull_boxed_with(
        &mut self,
        obj: &Object<'c>,
        work: TraverserWorkType,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        // If the work is release, and the object's type is Std::Destructor, then call destructor when the refcnt is one.
        if work == TraverserWorkType::release() && obj.is_destructor_object() {
            // Branch by whether or not the reference counter is one.
            let obj_ptr = obj.value.into_pointer_value();
            let (unique_bb, shared_bb) = self.build_branch_by_is_unique(obj_ptr);

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
            self.build_retain(dtor.clone(), one);
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
            self.build_release_boxed_with(obj, traverse_refs);
        } else {
            self.build_mark_boxed_with(obj, work, traverse_refs);
        }
    }

    // Release or mark global or mark threaded an object.
    pub fn build_release_mark(&mut self, obj: Object<'c>, work: TraverserWorkType) {
        if obj.is_box(self.type_env()) {
            let cont_bb = if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.

                // Append basic blocks.
                let current_bb = self.builder().get_insert_block().unwrap();
                let current_func = current_bb.get_parent().unwrap();
                let nonnull_bb = self
                    .context
                    .append_basic_block(current_func, "nonnull_in_release_dynamic");
                let cont_bb = self
                    .context
                    .append_basic_block(current_func, "cont_in_release_dynamic");

                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder()
                    .build_conditional_branch(is_null, cont_bb, nonnull_bb)
                    .unwrap();

                // Implement nonnull_bb.
                self.builder().position_at_end(nonnull_bb);

                Some(cont_bb)
            } else {
                None
            };

            // If the object is boxed and not dynamic,
            self.build_release_mark_nonnull_boxed(&obj, work);

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                self.builder()
                    .build_unconditional_branch(cont_bb.unwrap())
                    .unwrap();
                self.builder().position_at_end(cont_bb.unwrap());
            }
        } else if obj.is_funptr() {
            // Nothing to do for function pointers.
        } else {
            // Unboxed case (inlude lambda object).
            match create_traverser(&obj.ty, &vec![], self, Some(work)) {
                Some(trav) => {
                    self.builder()
                        .build_call(trav, &[obj.value.into()], "call_traverser_of_unboxed")
                        .unwrap();
                }
                None => {}
            }
        }
    }

    // Traverse a non-null boxed object's owned references (its elements / fields) for `work`
    // (release / mark). Dynamic objects carry their traverser and are called indirectly;
    // others use the statically generated one.
    fn traverse_boxed_refs(&mut self, obj: &Object<'c>, work: TraverserWorkType) {
        let obj_ptr = obj.value.into_pointer_value();
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
            let trav = object::create_traverser(&obj.ty, &vec![], self, Some(work));
            if let Some(trav) = trav {
                self.builder()
                    .build_call(trav, &[obj_ptr.into()], "call_trav")
                    .unwrap();
            }
        }
    }

    // Release a non-null boxed object, emitting `traverse_refs` to release its owned references
    // once the refcount reaches zero, before the object is freed.
    fn build_release_boxed_with(
        &mut self,
        obj: &Object<'c>,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        // Get pointer to the object.
        let obj_ptr = obj.value.into_pointer_value();

        // Branch by refcnt_state.
        let current_func = self
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr);
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
                inkwell::IntPredicate::EQ,
                old_refcnt,
                refcnt_type(self.context).const_int(1, false),
                "is_refcnt_zero",
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
            // Decrement refcnt atomically.
            let old_refcnt = self
                .builder()
                .build_atomicrmw(
                    inkwell::AtomicRMWBinOp::Sub,
                    ptr_to_refcnt,
                    refcnt_type(self.context).const_int(1, false),
                    inkwell::AtomicOrdering::Release,
                )
                .unwrap();

            // Branch to `threaded_destruction_bb` if old_refcnt is one.
            let threaded_destruction_bb = self
                .context
                .append_basic_block(current_func, "threaded_destruction_bb");
            let is_refcnt_one = self
                .builder()
                .build_int_compare(
                    inkwell::IntPredicate::EQ,
                    old_refcnt,
                    refcnt_type(self.context).const_int(1, false),
                    "is_refcnt_one",
                )
                .unwrap();
            self.builder()
                .build_conditional_branch(is_refcnt_one, threaded_destruction_bb, end_bb)
                .unwrap();

            // Implement `threaded_destruction_bb`.
            self.builder().position_at_end(threaded_destruction_bb);
            self.builder()
                .build_fence(inkwell::AtomicOrdering::Acquire, 0, "")
                .unwrap();
            self.builder()
                .build_unconditional_branch(destruction_bb)
                .unwrap();
        }

        // Implement `destruction_bb`
        self.builder().position_at_end(destruction_bb);

        // Release the object's owned references, then free it.
        traverse_refs(self);
        self.builder().build_free(obj_ptr).unwrap();
        self.builder().build_unconditional_branch(end_bb).unwrap();

        // Implement global_bb.
        self.builder().position_at_end(global_bb);
        self.builder().build_unconditional_branch(end_bb).unwrap();

        self.builder().position_at_end(end_bb);
    }

    // Mark a boxed object, emitting `traverse_refs` to mark its owned references before the
    // object itself is marked.
    fn build_mark_boxed_with(
        &mut self,
        obj: &Object<'c>,
        work: TraverserWorkType,
        traverse_refs: impl FnOnce(&mut Self),
    ) {
        assert!(
            work == TraverserWorkType::mark_global() || work == TraverserWorkType::mark_threaded()
        );

        // Get pointer to the object.
        let obj_ptr = obj.value.into_pointer_value();

        // Mark the object's owned references.
        traverse_refs(self);

        // Mark the object itself.
        if work == TraverserWorkType::mark_global() {
            self.mark_global_one(obj_ptr);
        } else {
            self.mark_threaded_one(obj_ptr);
        }
    }

    // Release object.
    pub fn release(&mut self, obj: Object<'c>) {
        // Get release function.
        let func_name = "release_".to_string() + &obj.ty.hash();
        let func = if let Some(func) = self.module.get_function(&func_name) {
            func
        } else {
            let func = self.module.add_function(
                &func_name,
                self.context
                    .void_type()
                    .fn_type(&[obj.ty.get_embedded_type(self, &vec![]).into()], false),
                Some(Linkage::Internal),
            );
            let bb = self.context.append_basic_block(func, "entry");
            let _builder_guard = self.push_builder();
            self.builder().position_at_end(bb);
            let obj_val = func.get_first_param().unwrap();
            let obj = Object::new(obj_val, obj.ty.clone(), self);
            self.build_release_mark(obj, TraverserWorkType::release());
            self.builder().build_return(None).unwrap();
            func
        };

        // Call release function.
        self.builder()
            .build_call(func, &[obj.value.into()], "call_release")
            .unwrap();
    }

    // Release nonnull boxed object.
    pub(crate) fn release_nonnull_boxed(&mut self, obj: &Object<'c>) {
        self.build_release_mark_nonnull_boxed(obj, TraverserWorkType::release())
    }

    // Mark all objects reachable from `obj` as global.
    pub fn mark_global(&mut self, obj: Object<'c>) {
        // Get `mark_global` function.
        let func_name = "mark_global_".to_string() + &obj.ty.hash();
        let func = if let Some(func) = self.module.get_function(&func_name) {
            func
        } else {
            let func = self.module.add_function(
                &func_name,
                self.context
                    .void_type()
                    .fn_type(&[obj.ty.get_embedded_type(self, &vec![]).into()], false),
                Some(Linkage::Internal),
            );
            let bb = self.context.append_basic_block(func, "entry");
            let _builder_guard = self.push_builder();
            self.builder().position_at_end(bb);
            let obj_val = func.get_first_param().unwrap();
            let obj = Object::new(obj_val, obj.ty.clone(), self);
            self.build_release_mark(obj, TraverserWorkType::mark_global());
            self.builder().build_return(None).unwrap();
            func
        };

        // Call `mark_global` function.
        self.builder()
            .build_call(func, &[obj.value.into()], "call_mark_global")
            .unwrap();
    }

    pub fn mark_threaded(&mut self, obj: Object<'c>) {
        // Get `mark_threaded` function.
        let func_name = "mark_threaded_".to_string() + &obj.ty.hash();
        let func = if let Some(func) = self.module.get_function(&func_name) {
            func
        } else {
            let func = self.module.add_function(
                &func_name,
                self.context
                    .void_type()
                    .fn_type(&[obj.ty.get_embedded_type(self, &vec![]).into()], false),
                Some(Linkage::Internal),
            );
            let bb = self.context.append_basic_block(func, "entry");
            let _builder_guard = self.push_builder();
            self.builder().position_at_end(bb);
            let obj_val = func.get_first_param().unwrap();
            let obj = Object::new(obj_val, obj.ty.clone(), self);
            self.build_release_mark(obj, TraverserWorkType::mark_threaded());
            self.builder().build_return(None).unwrap();
            func
        };

        // Call `mark_threaded` function.
        self.builder()
            .build_call(func, &[obj.value.into()], "call_mark_threaded")
            .unwrap();
    }

    fn mark_local_one(&mut self, ptr: PointerValue<'c>) {
        let ptr_refcnt_state: PointerValue<'_> = self.get_refcnt_state_ptr(ptr);
        // Store `REFCNT_STATE_LOCAL` to `ptr_refcnt_state`.
        self.builder()
            .build_store(
                ptr_refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
            )
            .unwrap();
    }

    fn mark_threaded_one(&mut self, obj_ptr: PointerValue<'c>) {
        let current_bb = self.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let cont_bb = self
            .context
            .append_basic_block(current_func, "cont_bb@mark_threaded");

        // Load refcnt state.
        let ptr_refcnt_state = self.get_refcnt_state_ptr(obj_ptr);
        let refcnt_state = self
            .builder()
            .build_load(
                refcnt_state_type(self.context),
                ptr_refcnt_state,
                "refcnt_state",
            )
            .unwrap()
            .into_int_value();

        // Branch by whether or not the refcnt state is `REFCNT_STATE_LOCAL`.
        let local_bb = self
            .context
            .append_basic_block(current_func, "local_bb@mark_threaded");
        let is_refcnt_state_local = self
            .builder()
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
                "is_refcnt_state_local",
            )
            .unwrap();
        self.builder()
            .build_conditional_branch(is_refcnt_state_local, local_bb, cont_bb)
            .unwrap();

        // Implement local_bb.
        self.builder().position_at_end(local_bb);
        // Store `REFCNT_STATE_THREADED` to `ptr_refcnt_state`.
        self.builder()
            .build_store(
                ptr_refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_THREADED as u64, false),
            )
            .unwrap();
        self.builder().build_unconditional_branch(cont_bb).unwrap();

        // Set builder's position as preparation for following implementation.
        self.builder().position_at_end(cont_bb);
    }

    // Mark object as global so that it will not be retained or released.
    fn mark_global_one(&mut self, ptr: PointerValue<'c>) {
        let ptr_refcnt_state: PointerValue<'_> = self.get_refcnt_state_ptr(ptr);
        // Store `REFCNT_STATE_GLOBAL` to `ptr_refcnt_state`.
        self.builder()
            .build_store(
                ptr_refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_GLOBAL as u64, false),
            )
            .unwrap();
    }

    // Print Rust's &str to stderr.
    fn eprint(&mut self, string: &str) {
        let string_ptr = self.add_global_string(string);
        let string_ptr = string_ptr.as_pointer_value();
        self.call_runtime(RUNTIME_EPRINTLN, &[string_ptr.into()]);
    }

    // Panic with Rust's &str (i.e, print string and abort.)
    pub fn panic(&mut self, string: &str) {
        self.eprint(string);
        self.call_runtime(RUNTIME_ABORT, &[]);
    }

    // Call a runtime function.
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
            let ret = obj.value;
            if self.sizeof(&ret.get_type()) != 0 {
                self.builder().build_return(Some(&ret)).unwrap();
            } else {
                self.builder().build_return(None).unwrap();
            }
            None
        } else {
            Some(obj)
        }
    }

    // Declare function of lambda expression
    pub fn declare_lambda_function(
        &mut self,
        lam: Arc<ExprNode>,
        name: Option<&FullName>,
    ) -> FunctionValue<'c> {
        let lam_ty = lam.type_.clone().unwrap();
        let lam_fn_ty = lambda_function_type(&lam_ty, self);
        let name = if name.is_some() {
            name.unwrap().to_string()
        } else {
            format!("closure[{}]", lam_ty.to_string_normalize())
        };
        let linkage = if lam_ty.is_funptr() && self.config.enable_separated_compilation() {
            Linkage::External
        } else {
            Linkage::Internal // For closure function, we specify `Internal` so that LLVM avoids name collision automatically.
        };
        let lam_fn = self.module.add_function(&name, lam_fn_ty, Some(linkage));
        // Create and set debug info subprogram.
        if self.has_di() {
            let fn_name = lam_fn.get_name().to_str().unwrap();
            lam_fn.set_subprogram(self.create_debug_subprogram(fn_name, lam.source.clone()));
        }
        lam_fn
    }

    // Create debug info subprogram.
    pub fn create_debug_subprogram<'a>(
        &'a self,
        fn_name: &str,
        span: Option<Span>,
    ) -> DISubprogram<'a> {
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

    // Push debug location
    pub fn push_debug_location(&mut self, span: Option<Span>) {
        self.debug_location.push(span.clone());
        self.set_debug_location(span);
    }

    // Pop debug location.
    pub fn pop_debug_location(&mut self) {
        self.debug_location.pop();
        self.reset_debug_location();
    }

    // Set debug location
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

    pub fn reset_debug_location(&mut self) {
        self.set_debug_location(flatten_opt(self.debug_location.last().cloned()));
    }

    // Emit a call to a C function from already-evaluated argument objects and a pre-allocated return
    // object. Each argument is marshalled to its C scalar (field 0), the function is called, and the
    // result is written back into the return object (field 1 of the `(IOState, ret)` tuple when
    // `is_io`, else field 0). A void return writes nothing.
    pub fn build_ffi_call_core(
        &mut self,
        source: &Option<Span>,
        mut obj: Object<'c>,
        fun_name: &Name,
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        is_var_args: bool,
        arg_objs: Vec<Object<'c>>,
        is_io: bool,
    ) -> Object<'c> {
        // Get c function
        let c_fun = match self.module.get_function(&fun_name) {
            Some(fun) => fun,
            None => {
                let ret_c_ty = ret_tycon.get_c_type(self.context);
                let parm_c_tys: Vec<BasicMetadataTypeEnum> = param_tys
                    .iter()
                    .map(|param_ty| {
                        let c_type = param_ty.get_c_type(self.context);
                        if c_type.is_none() {
                            panic_with_msg_src(
                                "Cannot use `()` as a parameter type of C function.",
                                source,
                            )
                        }
                        c_type.unwrap().into()
                    })
                    .collect::<Vec<_>>();
                let fn_ty = match ret_c_ty {
                    None => {
                        // Void case.
                        self.context.void_type().fn_type(&parm_c_tys, is_var_args)
                    }
                    Some(ret_c_ty) => ret_c_ty.fn_type(&parm_c_tys, is_var_args),
                };
                self.module.add_function(&fun_name, fn_ty, None)
            }
        };

        // Get argment values
        let args_vals = arg_objs
            .iter()
            .map(|obj| obj.extract_field(self, 0).into())
            .collect::<Vec<_>>();

        // Call c function
        let ret_c_val = self
            .builder()
            .build_call(c_fun, &args_vals, &format!("FFI_CALL({})", fun_name))
            .unwrap();
        match ret_c_val.try_as_basic_value() {
            Either::Left(ret_c_val) => {
                if is_io {
                    let ret_str = type_tycon(ret_tycon).get_struct_type(self, &vec![]);
                    let ret_str_val = ret_str.get_undef();
                    let ret_str_val = self
                        .builder()
                        .build_insert_value(ret_str_val, ret_c_val, 0, "")
                        .unwrap();
                    obj = obj.insert_field(self, 1, ret_str_val);
                } else {
                    obj = obj.insert_field(self, 0, ret_c_val);
                }
            }
            Either::Right(_) => {}
        }

        obj
    }

    // Project the captured value at `cap_idx` out of a closure's capture object `cap_name`,
    // retaining it (a retain-getter). `cap_tys` are the types of all captured values, needed to
    // reconstruct the capture object's struct layout; `result_ty` is the projected value's type.
    pub fn build_capture_project(
        &mut self,
        cap_name: &FullName,
        cap_idx: usize,
        cap_tys: &Vec<Arc<TypeNode>>,
        result_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let cap_obj = self.get_scoped_obj_noretain(cap_name);
        let cap_obj_ty = make_dynamic_object_ty().get_object_type(cap_tys, self.type_env());
        let cap_obj_str_ty = cap_obj_ty.to_struct_type(self, vec![]);
        let cap_val =
            cap_obj.extract_field_as(self, cap_obj_str_ty, cap_idx as u32 + DYNAMIC_OBJ_CAP_IDX);
        let obj = Object::new(cap_val, result_ty.clone(), self);
        let one = self.context.i64_type().const_int(1, false);
        self.build_retain(obj.clone(), one);
        obj
    }

    pub fn has_di(&self) -> bool {
        self.debug_info.is_some()
    }

    // Get current debug info builder.
    pub fn get_di_builder(&self) -> &DebugInfoBuilder<'c> {
        &self.debug_info.as_ref().unwrap().0
    }

    // Get current debug info compilation unit.
    pub fn get_di_compile_unit(&self) -> &DICompileUnit<'c> {
        &self.debug_info.as_ref().unwrap().1
    }

    // Finalize all debug infos.
    pub fn finalize_di(&self) {
        if self.has_di() {
            self.get_di_builder().finalize();
        }
    }

    pub fn create_di_file(&self, src: Option<SourceFile>) -> DIFile<'c> {
        if let Some(src) = src {
            self.get_di_builder()
                .create_file(&src.get_file_name(), &src.get_file_dir())
        } else {
            self.get_di_builder().create_file("<unknown source>", "")
        }
    }

    pub fn create_debug_local_variable(&mut self, name: &Name, obj: &Object<'c>) {
        // Push the value on the stack.
        let obj_val = obj.value;
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

    pub fn declare_symbol(&mut self, sym: &Symbol) -> FunctionValue<'c> {
        let name = &sym.name;
        let obj_ty = &sym.ty;
        if obj_ty.is_funptr() {
            // Declare lambda function.
            let lam = sym.expr.as_ref().unwrap().clone();
            let lam = lam.set_type(obj_ty.clone());
            let lam_fn = self.declare_lambda_function(lam, Some(name));
            self.add_global_object(name.clone(), lam_fn, obj_ty.clone());
            lam_fn
        } else {
            // Declare accessor function.
            let acc_fn_name = format!("Get#{}", name.to_string());
            let ty = obj_ty.get_embedded_type(self, &vec![]);
            let acc_fn_type = if self.sizeof(&ty) == 0 {
                self.context.void_type().fn_type(&[], false)
            } else {
                ty.fn_type(&[], false)
            };
            let acc_fn = self.module.add_function(
                &acc_fn_name,
                acc_fn_type,
                Some(self.config.external_if_separated()),
            );

            // Register the accessor function to gc.
            self.add_global_object(name.clone(), acc_fn, obj_ty.clone());

            // Return the accessor function.
            acc_fn
        }
    }

    // Bit cast between two types.
    // Allows bit cast between types with different sizes.
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
        let (from_bits, to_bits) = (self.sizeof(&from_ty), self.sizeof(&to_ty));
        let larger_ty = if from_bits > to_bits { from_ty } else { to_ty };
        let ptr = self.build_alloca_at_entry(larger_ty, "alloca@bit_cast");
        self.builder().build_store(ptr, val).unwrap();
        self.builder().build_load(to_ty, ptr, "bit_cast").unwrap()
    }

    // Add frame-pointer attribute to all functions in the module
    // This is especially important on macOS where backtrace() relies on frame pointers
    pub fn add_frame_pointer_attribute_to_all_functions(&self) {
        let mut func = self.module.get_first_function();
        while let Some(function) = func {
            // Add "frame-pointer"="all" attribute to ensure frame pointers are always kept
            function.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                self.context.create_string_attribute("frame-pointer", "all"),
            );
            func = function.get_next_function();
        }
    }
}

// Whether `v` is the constant integer 1. Used where a retain-by-`amount` reproduces the ordinary
// single retain when the amount is exactly 1, so that path stays byte-identical.
pub(crate) fn is_const_one(v: IntValue) -> bool {
    v.get_zero_extended_constant() == Some(1)
}
