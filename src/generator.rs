// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use std::{cell::RefCell, env, sync::Arc};

use crate::error::panic_with_err;
use crate::error::panic_with_err_src;
use ast::name::FullName;
use ast::name::Name;
use either::Either;
use either::Either::Left;
use either::Either::Right;
use inkwell::values::GlobalValue;
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
use misc::flatten_opt;
use misc::Map;
use misc::Set;

use super::*;

#[derive(Clone)]
pub struct ScopedValue<'c> {
    accessor: ValueAccessor<'c>,
    used_later: u32,
}

#[derive(Clone)]
pub enum ValueAccessor<'c> {
    Local(Object<'c>),
    Global(FunctionValue<'c>, Arc<TypeNode>),
}

impl<'c> ValueAccessor<'c> {
    // Get the object.
    pub fn get<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> Object<'c> {
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
                            GenerationContext::get_undef(&ty)
                        }
                    }
                };
                Object::new(val, ty.clone(), gc)
            }
        }
    }

    // Get global object's function value.
    pub fn get_global_fun(&self) -> FunctionValue<'c> {
        match self {
            ValueAccessor::Local(_) => panic!("`\"get_global_fun\"` called for local variable."),
            ValueAccessor::Global(fun, _) => *fun,
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
        gc: &mut GenerationContext<'c, 'm>,
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

    pub fn debug_embedded_ty<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> DIType<'c> {
        ty_to_debug_embedded_ty(self.ty.clone(), gc)
    }

    pub fn struct_ty<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructType<'c> {
        assert!(!self.is_funptr());
        ty_to_object_ty(&self.ty, &vec![], gc.type_env()).to_struct_type(gc, vec![])
    }

    // Get the pointer to the field of an boxed object.
    pub fn gep_boxed<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> PointerValue<'c> {
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
        gc: &mut GenerationContext<'c, 'm>,
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
        gc: &mut GenerationContext<'c, 'm>,
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
        gc: &mut GenerationContext<'c, 'm>,
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
        gc: &mut GenerationContext<'c, 'm>,
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
    pub fn extract_trav_from_dynamic<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> PointerValue<'c> {
        assert!(self.ty.is_dynamic());
        self.extract_field(gc, DYNAMIC_OBJ_TRAVARSER_IDX)
            .into_pointer_value()
    }

    // Check if the pointer is null.
    // Can be used for boxed objects.
    pub fn is_null<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> IntValue<'c> {
        assert!(self.is_box(gc.type_env()));
        gc.builder()
            .build_is_null(self.value.into_pointer_value(), "is_null")
            .unwrap()
    }

    // Get the pointer to the field of an boxed object.
    // Can be used only for boxed objects.
    pub fn ptr_to_field<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> PointerValue<'c> {
        assert!(self.is_box(&gc.type_env));
        let ty = self.struct_ty(gc);
        self.ptr_to_field_as(gc, ty, field_idx)
    }

    // Get the pointer to the field of an boxed object.
    // You can specify the struct type of the boxed object, ignoring the `ty` field of the object.
    // Can be used only for boxed objects.
    pub fn ptr_to_field_as<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
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
            used_later: 0,
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

    fn modify_used_later(&mut self, vars: &Set<FullName>, by: i32) {
        for var in vars {
            if !var.is_local() {
                continue;
            }
            let used_later = &mut self
                .data
                .get_mut(var)
                .unwrap()
                .last_mut()
                .unwrap()
                .used_later;
            *used_later = add_i32_to_u32(*used_later, by);
        }
    }
    fn increment_used_later(&mut self, names: &Set<FullName>) {
        self.modify_used_later(names, 1);
    }
    fn decrement_used_later(&mut self, names: &Set<FullName>) {
        self.modify_used_later(names, -1);
    }
    fn is_used_later(&self, name: &FullName) -> bool {
        self.data.get(name).unwrap().last().unwrap().used_later > 0
    }
}

fn add_i32_to_u32(u: u32, i: i32) -> u32 {
    if i.is_negative() {
        u - i.wrapping_abs() as u32
    } else {
        u + i as u32
    }
}

pub struct GenerationContext<'c, 'm> {
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

impl<'c, 'm> GenerationContext<'c, 'm> {
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

    pub fn alignment(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_preferred_alignment(ty) as u64
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
            panic_with_err(&format!("Duplicate symbol: {}", name.to_string()));
        } else {
            let used_later = if ty.is_box(self.type_env()) {
                // We do not need to retain global objects. Always move out it.
                0
            } else {
                u32::MAX / 2
            };
            self.global.insert(
                name.clone(),
                ScopedValue {
                    accessor: ValueAccessor::Global(function, ty),
                    used_later,
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
            self.global.get(var).unwrap().clone()
        }
    }

    // Get an object on the scope (or global).
    // This function does not retain the object.
    pub fn get_scoped_obj_noretain(&mut self, name: &FullName) -> Object<'c> {
        self.get_scoped_value(name).accessor.get(self)
    }

    // Get an object on the scope (or global).
    // This function retains the object if it will be used later.
    pub fn get_scoped_obj(&mut self, var_name: &FullName) -> Object<'c> {
        let val = self.get_scoped_value(var_name);
        let obj = val.accessor.get(self);
        if val.used_later > 0 {
            // If used later, clone object.
            self.build_retain(obj.clone());
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

    // Lock variables in scope to avoid being moved out.
    pub fn scope_lock_as_used_later(self: &mut Self, names: &Set<FullName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .increment_used_later(names);
    }

    // Unlock variables in scope.
    pub fn scope_unlock_as_used_later(self: &mut Self, names: &Set<FullName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .decrement_used_later(names);
    }

    // Is a variable used later?
    pub fn is_var_used_later(&self, var: &FullName) -> bool {
        if var.is_global() {
            return true;
        }
        self.scope.borrow().last().unwrap().is_used_later(var)
    }

    // Push scope.
    fn scope_push(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .push_local(var, obj)
    }

    // Pop scope.
    fn scope_pop(self: &mut Self, var: &FullName) {
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
        self: &mut GenerationContext<'c, 'm>,
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
        self: &mut GenerationContext<'c, 'm>,
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

        // Call function pointer with arguments, CAP if closure.
        let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
        for arg in args {
            call_args.push(arg.value.into());
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
                GenerationContext::get_undef(&ty)
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
            self.build_retain(obj);
            self.builder().build_return(None).unwrap();
            func
        };

        // Call retain function.
        self.builder()
            .build_call(func, &[obj.value.into()], "call_retain")
            .unwrap();
    }

    // Retain an object.
    fn build_retain(&mut self, obj: Object<'c>) {
        if obj.is_box(self.type_env()) {
            let current_bb = self.builder().get_insert_block().unwrap();
            let current_func = current_bb.get_parent().unwrap();
            let cont_bb = self
                .context
                .append_basic_block(current_func, "cont_bb@retain");

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.

                // Dynamic object can be null.
                let nonnull_bb = self
                    .context
                    .append_basic_block(current_func, "nonnull_bb@retain");

                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder()
                    .build_conditional_branch(is_null, cont_bb, nonnull_bb)
                    .unwrap();

                // Implement code to retain in nonnull_bb.
                self.builder().position_at_end(nonnull_bb);
            }

            let obj_ptr = obj.value.into_pointer_value();
            // Branch by refcnt_state.
            let (local_bb, threaded_bb, global_bb) = self.build_branch_by_refcnt_state(obj_ptr);

            // Implement `local_bb`.
            self.builder().position_at_end(local_bb);

            // In `local_bb`, increment refcnt and jump to `cont_bb`.
            let old_refcnt_local = self
                .builder()
                .build_load(refcnt_type(self.context), obj_ptr, "")
                .unwrap()
                .into_int_value();
            let new_refcnt = self
                .builder()
                .build_int_nsw_add(
                    old_refcnt_local,
                    refcnt_type(self.context).const_int(1, false).into(),
                    "",
                )
                .unwrap();
            self.builder().build_store(obj_ptr, new_refcnt).unwrap();
            self.builder().build_unconditional_branch(cont_bb).unwrap();

            // Implement threaded_bb.
            if threaded_bb.is_some() {
                let threaded_bb = threaded_bb.unwrap();
                self.builder().position_at_end(threaded_bb);

                // In `threaded_bb`, increment refcnt atomically and jump to `cont_bb`.
                let ptr_to_refcnt = self.get_refcnt_ptr(obj_ptr);
                let _old_refcnt_threaded = self
                    .builder()
                    .build_atomicrmw(
                        inkwell::AtomicRMWBinOp::Add,
                        ptr_to_refcnt,
                        refcnt_type(self.context).const_int(1, false),
                        inkwell::AtomicOrdering::Monotonic,
                    )
                    .unwrap();
                self.builder().build_unconditional_branch(cont_bb).unwrap();
            }

            // Implement global_bb.
            self.builder().position_at_end(global_bb);
            self.builder().build_unconditional_branch(cont_bb).unwrap();

            self.builder().position_at_end(cont_bb);
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
                        self.retain(subobj);
                    }
                    ObjectFieldType::UnionBuf(_) => {
                        ObjectFieldType::retain_union(self, obj.clone());
                    }
                    ObjectFieldType::UnionTag => {}
                    ObjectFieldType::Array(_) => unreachable!(),
                }
            }
        }
    }

    // Release or mark global or mark threaded nonnull boxed object.
    fn build_release_mark_nonnull_boxed(&mut self, obj: &Object<'c>, work: TraverserWorkType) {
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
            self.build_retain(dtor.clone());
            let io_act = self.apply_lambda(dtor, vec![value], false).unwrap();
            let res = run_io_value(self, &io_act);
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
            self.build_release_boxed(obj);
        } else {
            self.build_mark_boxed(obj, work);
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

    // Release a boxed object.
    fn build_release_boxed(&mut self, obj: &Object<'c>) {
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

        // Call dtor.
        if obj.is_dynamic_object() {
            // If the object is dynamic, extract the traverser from the object and call it.
            let trav = obj.extract_trav_from_dynamic(self);
            let trav_ty = traverser_type(self, &obj.ty, true);
            self.builder()
                .build_indirect_call(
                    trav_ty,
                    trav,
                    &[
                        obj_ptr.into(),
                        traverser_work_type(self.context)
                            .const_int(TRAVERSER_WORK_RELEASE as u64, false)
                            .into(),
                    ],
                    "call_dtor",
                )
                .unwrap();
        } else {
            // If the object is not dynamic, call the dtor of the object.
            let dtor = object::create_traverser(
                &obj.ty,
                &vec![],
                self,
                Some(TraverserWorkType::release()),
            );
            if let Some(dtor) = dtor {
                self.builder()
                    .build_call(dtor, &[obj_ptr.into()], "call_dtor")
                    .unwrap();
            }
        }

        // free.
        self.builder().build_free(obj_ptr).unwrap();
        self.builder().build_unconditional_branch(end_bb).unwrap();

        // Implement global_bb.
        self.builder().position_at_end(global_bb);
        self.builder().build_unconditional_branch(end_bb).unwrap();

        self.builder().position_at_end(end_bb);
    }

    // Mark global or mark threaded a boxed object.
    fn build_mark_boxed(&mut self, obj: &Object<'c>, work: TraverserWorkType) {
        assert!(
            work == TraverserWorkType::mark_global() || work == TraverserWorkType::mark_threaded()
        );

        // Get pointer to the object.
        let obj_ptr = obj.value.into_pointer_value();

        // Get pointer to call the traverser function.
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
                    "call_trav_dynamic_for_mark",
                )
                .unwrap();
        } else {
            // If the object is not dynamic, call the dtor of the object.
            let trav = object::create_traverser(&obj.ty, &vec![], self, Some(work));
            if let Some(trav) = trav {
                self.builder()
                    .build_call(trav, &[obj_ptr.into()], "call_trav_for_mark")
                    .unwrap();
            }
        }

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
    fn release_nonnull_boxed(&mut self, obj: &Object<'c>) {
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
        self.call_runtime(RUNTIME_EPRINT, &[string_ptr.into()]);
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

    // Evaluate expression.
    // - tail: Whether or not the expression is in tail position, i.e., the result of the expression is the result of the function.
    //         If true, builds return instruction and returns `None`.
    pub fn eval_expr(&mut self, expr: Arc<ExprNode>, tail: bool) -> Option<Object<'c>> {
        assert!(expr.ty.as_ref().unwrap().free_vars().is_empty());

        if self.has_di() {
            self.push_debug_location(expr.source.clone())
        };

        let mut ret = match &*expr.expr {
            Expr::Var(var) => self.eval_var(var.clone(), tail),
            Expr::LLVM(lit) => self.eval_llvm(lit.clone(), expr.ty.clone().unwrap().clone(), tail),
            Expr::App(lambda, args) => self.eval_app(lambda.clone(), args.clone(), tail),
            Expr::Lam(_, _) => self.eval_lam(expr.clone(), tail),
            Expr::Let(pat, bound, expr) => self.eval_let(pat, bound.clone(), expr.clone(), tail),
            Expr::If(cond_expr, then_expr, else_expr) => self.eval_if(
                cond_expr.clone(),
                then_expr.clone(),
                else_expr.clone(),
                tail,
            ),
            Expr::Match(cond, pat_vals) => self.eval_match(cond.clone(), pat_vals, tail),
            Expr::TyAnno(e, _) => self.eval_expr(e.clone(), tail),
            Expr::MakeStruct(_, fields) => {
                let struct_ty = expr.ty.clone().unwrap();
                self.eval_make_struct(fields.clone(), struct_ty, tail)
            }
            Expr::ArrayLit(elems) => self.eval_array_lit(elems, expr.ty.clone().unwrap(), tail),
            Expr::FFICall(fun_name, ret_ty, param_tys, args, is_io) => {
                self.eval_ffi_call(&expr, fun_name, ret_ty, param_tys, args, *is_io, tail)
            }
        };

        if self.has_di() {
            self.pop_debug_location();
        }

        if let Some(ret) = ret.as_mut() {
            ret.ty = expr.ty.clone().unwrap();
        }
        ret
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

    // Evaluate variable.
    fn eval_var(&mut self, var: Arc<Var>, tail: bool) -> Option<Object<'c>> {
        let obj = self.get_scoped_obj(&var.name);
        self.build_tail(obj, tail)
    }

    // Evaluate application
    fn eval_app(
        &mut self,
        fun: Arc<ExprNode>,
        args: Vec<Arc<ExprNode>>,
        tail: bool,
    ) -> Option<Object<'c>> {
        // Before evaluating `fun`, we lock all variables in arguments as used later.
        for arg in &args {
            self.scope_lock_as_used_later(arg.free_vars());
        }

        // Evaluate the function object.
        let fun_obj = self.eval_expr(fun, false).unwrap();

        // Evaluate arguments.
        let mut arg_objs = vec![];
        for arg in args.iter() {
            self.scope_unlock_as_used_later(arg.free_vars());

            // Evaluate the argument expression.
            let arg_obj = self.eval_expr(arg.clone(), false).unwrap();
            arg_objs.push(arg_obj)
        }

        // Call the function.
        self.apply_lambda(fun_obj, arg_objs, tail)
    }

    // Evaluate llvm
    fn eval_llvm(
        &mut self,
        llvm: Arc<InlineLLVM>,
        ty: Arc<TypeNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        llvm.generator.generate(self, &ty, tail)
    }

    // Calculate captured variables and their types of lambda expression.
    // Normalize its orderings.
    pub fn calculate_captured_vars_of_lambda(
        &mut self,
        lam: Arc<ExprNode>,
    ) -> Vec<(FullName, Arc<TypeNode>)> {
        // let (args, body) = lam.destructure_lam();
        // let mut cap_names = body.free_vars().clone();
        // for arg in args {
        //     cap_names.remove(&arg.name);
        // }
        // cap_names.remove(&FullName::local(CAP_NAME));
        // let mut cap_vars = cap_names
        //     .into_iter()
        //     .filter(|name| name.is_local())
        //     .map(|name| (name.clone(), self.get_scoped_obj_noretain(&name).ty))
        //     .collect::<Vec<_>>();
        // cap_vars.sort_by_key(|(name, _)| name.to_string());

        let cap_vars = lam.lambda_cap_names();
        let cap_vars = cap_vars
            .into_iter()
            .map(|name| {
                let obj = self.get_scoped_obj_noretain(&name);
                (name, obj.ty)
            })
            .collect::<Vec<_>>();

        // Validation
        let lam_ty = lam.ty.clone().unwrap();
        assert!(!lam_ty.is_funptr() || cap_vars.len() == 0); // Function poitners cannot capture objects.

        cap_vars
    }

    // Declare function of lambda expression
    pub fn declare_lambda_function(
        &mut self,
        lam: Arc<ExprNode>,
        name: Option<&FullName>,
    ) -> FunctionValue<'c> {
        let lam_ty = lam.ty.clone().unwrap();
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
    pub fn create_debug_subprogram(&self, fn_name: &str, span: Option<Span>) -> DISubprogram {
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

    // Implement function of lambda expression
    pub fn implement_lambda_function(
        &mut self,
        lam: Arc<ExprNode>,
        lam_fn: FunctionValue<'c>,
        cap_vars: Option<Vec<(FullName, Arc<TypeNode>)>>,
    ) {
        let lam_ty = lam.ty.clone().unwrap();
        let (args, body) = lam.destructure_lam();
        let cap_vars = if cap_vars.is_some() {
            cap_vars.unwrap()
        } else {
            self.calculate_captured_vars_of_lambda(lam.clone())
        };
        let cap_tys = cap_vars
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect::<Vec<_>>();

        // Create new builder and set up
        let _builder_guard = self.push_builder();
        let bb = self.context.append_basic_block(lam_fn, "entry");
        self.builder().position_at_end(bb);

        // Push debug info scope.
        let _di_scope_guard = if self.has_di() {
            let subprogram = lam_fn.get_subprogram();
            Some(self.push_debug_scope(subprogram.map(|sub| sub.as_debug_info_scope())))
        } else {
            None
        };

        // Create new scope
        let _scope_guard = self.push_scope();

        // Push argments on scope.
        let mut arg_objs = vec![];
        for ((i, arg), arg_ty) in args.iter().enumerate().zip(lam_ty.get_lambda_srcs().iter()) {
            let arg_val = lam_fn.get_nth_param(i as u32).unwrap();
            let arg_obj = Object::new(arg_val, arg_ty.clone(), self);
            self.scope_push(&arg.name, &arg_obj);
            arg_objs.push(arg_obj);
        }

        // Push CAP on scope if lambda is closure.
        let cap_obj = if lam_ty.is_closure() {
            let self_idx = args.len();
            let cap_obj_val = lam_fn.get_nth_param(self_idx as u32).unwrap();
            let cap_obj = Object::new(cap_obj_val, make_dynamic_object_ty(), self);
            self.scope_push(&FullName::local(CAP_NAME), &cap_obj);
            Some(cap_obj)
        } else {
            None
        };

        // Push captured objects on scope.
        if lam_ty.is_closure() {
            let cap_obj_ty = make_dynamic_object_ty().get_object_type(&cap_tys, self.type_env());
            let cap_obj_str_ty = cap_obj_ty.to_struct_type(self, vec![]);

            for (i, (cap_name, cap_ty)) in cap_vars.iter().enumerate() {
                let cap_val = cap_obj.as_ref().unwrap().extract_field_as(
                    self,
                    cap_obj_str_ty,
                    i as u32 + DYNAMIC_OBJ_CAP_IDX,
                );
                let cap_obj = Object::new(cap_val, cap_ty.clone(), self);
                self.build_retain(cap_obj.clone());
                self.scope_push(cap_name, &cap_obj);
                // Create local variable for debug info.
                if self.has_di() {
                    self.create_debug_local_variable(&cap_name.to_string(), &cap_obj);
                }
            }
        }

        // Release CAP here if CAP is unused
        if lam_ty.is_closure() && cap_vars.len() > 0 {
            if !body.free_vars().contains(&FullName::local(CAP_NAME)) {
                // To avoid null checking, call release_nonnull_boxed directly.
                self.release_nonnull_boxed(&cap_obj.unwrap());
            }
        }

        // Release arguments if unused.
        for (i, arg) in args.iter().enumerate() {
            if !body.free_vars().contains(&arg.name) {
                self.release(arg_objs[i].clone());
            }
        }

        // Calculate body.
        self.eval_expr(body.clone(), true);
    }

    // Evaluate lambda abstraction.
    fn eval_lam(&mut self, lam: Arc<ExprNode>, tail: bool) -> Option<Object<'c>> {
        let lam_ty = lam.ty.clone().unwrap();

        // Calculate captured variables.
        let cap_vars = self.calculate_captured_vars_of_lambda(lam.clone());
        let cap_tys = cap_vars
            .iter()
            .map(|(_name, ty)| ty.clone())
            .collect::<Vec<_>>();

        // Define lambda function
        let lam_fn = self.declare_lambda_function(lam.clone(), None);
        self.implement_lambda_function(lam, lam_fn, Some(cap_vars.clone()));

        // Allocate lambda
        let name = format!("lamda[{}]", lam_ty.to_string());
        let mut lam = create_obj(lam_ty.clone(), &vec![], None, self, Some(name.as_str()));

        // Set function pointer to lambda.
        let funptr_idx = if lam_ty.is_closure() {
            CLOSURE_FUNPTR_IDX
        } else {
            0
        };
        let lam_fn_ptr = lam_fn.as_global_value().as_pointer_value();
        lam = lam.insert_field(self, funptr_idx, lam_fn_ptr);

        if lam_ty.is_closure() {
            // Set captured objects.
            let cap_obj_ptr = if cap_vars.len() > 0 {
                // If some objects are captured,

                // Allocate dynamic object to store captured objects.
                let dynamic_obj_ty = make_dynamic_object_ty();
                let cap_obj = create_obj(
                    dynamic_obj_ty.clone(),
                    &cap_tys,
                    None,
                    self,
                    Some(&format!("captured_objects_of_{}", name)),
                );

                // Get struct type of cap_obj.
                let cap_obj_str_ty = dynamic_obj_ty
                    .get_object_type(&cap_tys, self.type_env())
                    .to_struct_type(self, vec![]);

                // Set captured objects to cap_obj.
                for (i, (name, _)) in cap_vars.iter().enumerate() {
                    let obj = self.get_scoped_obj(name);
                    let val = obj.value;
                    cap_obj.insert_field_as(
                        self,
                        cap_obj_str_ty,
                        i as u32 + DYNAMIC_OBJ_CAP_IDX,
                        val,
                    );
                }

                cap_obj.value
            } else {
                // If no objects are captured, we set null pointer.
                self.context
                    .ptr_type(AddressSpace::from(0))
                    .const_null()
                    .as_basic_value_enum()
            };

            // Store cap_obj to lambda
            lam = lam.insert_field(self, CLOSURE_CAPTURE_IDX, cap_obj_ptr);
        }

        // Return lambda object
        self.build_tail(lam, tail)
    }

    // Evaluate let
    fn eval_let(
        &mut self,
        pat: &Arc<PatternNode>,
        bound: Arc<ExprNode>,
        val: Arc<ExprNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let vars = pat.pattern.vars();
        let mut used_in_val_except_pat = val.free_vars().clone();
        for v in vars {
            used_in_val_except_pat.remove(&v);
        }
        self.scope_lock_as_used_later(&used_in_val_except_pat);
        let bound = self.eval_expr(bound.clone(), false).unwrap();
        self.scope_unlock_as_used_later(&used_in_val_except_pat);
        let subobjs = self.destructure_object_by_pattern(pat, &bound);
        for (var_name, obj) in &subobjs {
            if val.free_vars().contains(&var_name) {
                self.scope_push(var_name, &obj);
            } else {
                self.release(obj.clone());
            }
            // Create local variable for debug info.
            if self.has_di() {
                self.create_debug_local_variable(&var_name.to_string(), &obj);
            }
        }
        let val_obj = self.eval_expr(val.clone(), tail);
        for (var_name, _) in &subobjs {
            if val.free_vars().contains(&var_name) {
                self.scope_pop(var_name);
            }
        }
        val_obj
    }

    // Destructure object by pattern.
    // For union pattern, the tag value is NOT checked.
    fn destructure_object_by_pattern(
        &mut self,
        pat: &Arc<PatternNode>,
        obj: &Object<'c>,
    ) -> Vec<(FullName, Object<'c>)> {
        let mut ret = vec![];
        match &pat.pattern {
            Pattern::Var(v, _) => {
                ret.push((v.name.clone(), obj.clone()));
            }
            Pattern::Struct(tc, field_to_pat) => {
                // Get field names of the struct.
                let str_fields = self
                    .type_env()
                    .tycons
                    .get(tc.as_ref())
                    .unwrap()
                    .fields
                    .iter()
                    .map(|field| field.name.clone());

                // Calculate a map that maps field name to its index.
                let field_to_idx = str_fields
                    .enumerate()
                    .map(|(i, name)| (name.clone(), i as u32))
                    .collect::<Map<_, _>>();

                // Extract fields.
                let field_indices = field_to_pat
                    .iter()
                    .map(|(name, _)| field_to_idx[name])
                    .collect::<Vec<_>>();
                let fields = ObjectFieldType::get_struct_fields(self, obj, &field_indices);

                // Match to subpatterns.
                for (i, (_, pat)) in field_to_pat.iter().enumerate() {
                    ret.append(&mut self.destructure_object_by_pattern(&pat, &fields[i]));
                }
            }
            Pattern::Union(variant_name, subpat) => {
                let (variant_idx, _union_tycon, _union_ti) =
                    Pattern::get_variant_info(variant_name, self.type_env());
                let variant_ty = obj.ty.field_types(self.type_env())[variant_idx].clone();
                let value = ObjectFieldType::get_union_value(self, obj.clone(), &variant_ty);
                ret.append(&mut self.destructure_object_by_pattern(subpat, &value));
            }
        }
        ret
    }

    // Evaluate if
    fn eval_if(
        &mut self,
        cond_expr: Arc<ExprNode>,
        then_expr: Arc<ExprNode>,
        else_expr: Arc<ExprNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let res_ty = then_expr.ty.clone().unwrap();

        let mut used_then_or_else = then_expr.free_vars().clone();
        used_then_or_else.extend(else_expr.free_vars().clone());
        self.scope_lock_as_used_later(&used_then_or_else);
        let cond_obj = self.eval_expr(cond_expr, false).unwrap();
        self.scope_unlock_as_used_later(&used_then_or_else);
        let cond_val = cond_obj.extract_field(self, 0).into_int_value();
        self.release(cond_obj);
        let cond_val = self
            .builder()
            .build_int_cast(cond_val, self.context.bool_type(), "cond_val_i1")
            .unwrap();
        let bb = self.builder().get_insert_block().unwrap();
        let func = bb.get_parent().unwrap();
        let mut then_bb = self.context.append_basic_block(func, "then");
        let mut else_bb = self.context.append_basic_block(func, "else");
        let cont_bb = if tail {
            None
        } else {
            Some(self.context.append_basic_block(func, "cont"))
        };
        self.builder()
            .build_conditional_branch(cond_val, then_bb, else_bb)
            .unwrap();

        // Implement then block.
        self.builder().position_at_end(then_bb);
        // Release variables used only in the else block.
        for var_name in &else_expr.free_vars_sorted() {
            // Here we use sorted free variables to fix the binary code.
            if !then_expr.free_vars().contains(var_name)
                && self.get_scoped_value(var_name).used_later == 0
            {
                let var = self.get_scoped_obj_noretain(var_name);
                self.release(var);
            }
        }
        let then_val = if tail {
            self.eval_expr(then_expr.clone(), true);
            None
        } else {
            let then_val = self.eval_expr(then_expr.clone(), false).unwrap();
            let then_val = then_val.value;
            then_bb = self.builder().get_insert_block().unwrap();
            self.builder()
                .build_unconditional_branch(cont_bb.unwrap())
                .unwrap();
            Some(then_val)
        };

        // Implement else block.
        self.builder().position_at_end(else_bb);
        // Release variables used only in the then block.
        for var_name in &then_expr.free_vars_sorted() {
            // Here we use sorted free variables to fix the binary code.
            if !else_expr.free_vars().contains(var_name)
                && self.get_scoped_value(var_name).used_later == 0
            {
                let var = self.get_scoped_obj_noretain(var_name);
                self.release(var);
            }
        }
        let else_val = if tail {
            self.eval_expr(else_expr.clone(), true);
            None
        } else {
            let else_val = self.eval_expr(else_expr.clone(), false).unwrap();
            let else_val = else_val.value;
            else_bb = self.builder().get_insert_block().unwrap();
            self.builder()
                .build_unconditional_branch(cont_bb.unwrap())
                .unwrap();
            Some(else_val)
        };

        // Return value.
        if tail {
            return None;
        };
        // Implement cont block.
        let cont_bb = cont_bb.unwrap();
        let then_val = then_val.unwrap();
        let else_val = else_val.unwrap();
        self.builder().position_at_end(cont_bb);
        // Return the phi value.
        let phi_ty = then_val.get_type();
        let phi = self.builder().build_phi(phi_ty, "phi").unwrap();
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);
        Some(Object::new(phi.as_basic_value(), res_ty.clone(), self))
    }

    fn eval_match(
        &mut self,
        cond: Arc<ExprNode>,
        pat_vals: &[(Arc<PatternNode>, Arc<ExprNode>)],
        tail: bool,
    ) -> Option<Object<'c>> {
        // Calculate the set of free variables used in values.
        let mut vars_used_in_any_case = Set::default();
        for (pat, val) in pat_vals {
            vars_used_in_any_case.extend(val.free_vars_shadowed_by(&pat.pattern.vars()));
        }

        // Evaluate the condition.
        self.scope_lock_as_used_later(&vars_used_in_any_case);
        let cond = self.eval_expr(cond, false).unwrap();
        self.scope_unlock_as_used_later(&vars_used_in_any_case);

        // Prepare basic blocks for each pattern.
        let current_func = self
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let cont_bb = if tail {
            None
        } else {
            Some(self.context.append_basic_block(current_func, "match_cont"))
        };
        let mut tag_pat_val_bbs: Vec<(
            Option<IntValue>,
            Arc<PatternNode>,
            Arc<ExprNode>,
            BasicBlock,
        )> = vec![];
        for (pat, val) in pat_vals {
            let pat_str = pat.pattern.to_string();
            let pat_bb = self
                .context
                .append_basic_block(current_func, &format!("case_`{}`", pat_str));
            let tag_val = if !pat.is_union() {
                // Non-variant pattern.
                None
            } else {
                // Variant pattern.
                let (variant_idx, _union_tycon, _union_ti) =
                    Pattern::get_variant_info(pat.get_union_variant(), self.type_env());
                let tag_val = ObjectFieldType::UnionTag
                    .to_basic_type(self, vec![])
                    .into_int_type()
                    .const_int(variant_idx as u64, false);
                Some(tag_val)
            };
            tag_pat_val_bbs.push((tag_val, pat.clone(), val.clone(), pat_bb));
        }

        // Build switch (if the match cases are variant patterns).
        // NOTE: It is already validated that:
        // - there are no empty `match`, and
        // - non-variant pattern is at the end of the cases.
        let else_bb = tag_pat_val_bbs.last().unwrap().3;
        let cases = tag_pat_val_bbs
            .iter()
            .take(tag_pat_val_bbs.len() - 1) // Skip the last one.
            .map(|(tag_val, _, _, bb)| (tag_val.unwrap(), *bb))
            .collect::<Vec<_>>();
        if cases.len() > 0 {
            let tag_val = ObjectFieldType::get_union_tag(self, &cond);
            self.builder()
                .build_switch(tag_val, else_bb, &cases)
                .unwrap();
        } else {
            self.builder().build_unconditional_branch(else_bb).unwrap();
        }

        // Implement each cases.
        let mut val_objs: Vec<(Object, BasicBlock)> = vec![]; // Data used to construct phi node. Empty if tail.
        for (_, pat, val, bb) in tag_pat_val_bbs {
            self.builder().position_at_end(bb);

            // Release variables used only in the other cases.
            let vars_used_in_later = val.free_vars_shadowed_by(&pat.pattern.vars());
            let vars_used_only_in_others = vars_used_in_any_case.difference(&vars_used_in_later);
            for var in vars_used_only_in_others {
                if self.get_scoped_value(var).used_later == 0 {
                    let var = self.get_scoped_obj_noretain(var);
                    self.release(var);
                }
            }

            // Destructure object by pattern.
            let subobjs = self.destructure_object_by_pattern(&pat, &cond);

            // Push subobjects on scope.
            for (var_name, obj) in &subobjs {
                if val.free_vars().contains(&var_name) {
                    self.scope_push(var_name, &obj);
                } else {
                    self.release(obj.clone());
                }
                // Create local variable for debug info.
                if self.has_di() {
                    self.create_debug_local_variable(&var_name.to_string(), &obj);
                }
            }

            // Evaluate the value.
            if tail {
                self.eval_expr(val.clone(), true);
            } else {
                let val_obj = self.eval_expr(val.clone(), false).unwrap();
                let bb = self.builder().get_insert_block().unwrap();
                val_objs.push((val_obj, bb));
                self.builder()
                    .build_unconditional_branch(cont_bb.unwrap())
                    .unwrap();
            }
            // Pop subobjects from scope.
            for (var_name, _) in &subobjs {
                if val.free_vars().contains(&var_name) {
                    self.scope_pop(var_name);
                }
            }
        }

        // Return the result.
        if tail {
            return None;
        }
        // Implement the cont_bb.
        let cont_bb = cont_bb.unwrap();
        self.builder().position_at_end(cont_bb);

        // Return value.
        if val_objs.len() == 1 {
            // If there is only one case, then return the value directly.
            Some(val_objs.pop().unwrap().0)
        } else {
            // In this case, build phi node.
            let phi_ty = val_objs[0].0.value.get_type();
            let phi = self.builder().build_phi(phi_ty, "match_phi").unwrap();
            for (val, bb) in &val_objs {
                phi.add_incoming(&[(&val.value, bb.clone())]);
            }
            Some(Object::new(
                phi.as_basic_value(),
                val_objs[0].0.ty.clone(),
                self,
            ))
        }
    }

    // Evaluate `MakeStruct` expression.
    fn eval_make_struct(
        &mut self,
        fields: Vec<(Name, Arc<ExprNode>)>,
        struct_ty: Arc<TypeNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        let mut str_obj = create_obj(
            struct_ty.clone(),
            &vec![],
            None,
            self,
            Some("allocate_MakeStruct"),
        );
        let field_types = struct_ty.field_types(self.type_env());
        assert_eq!(field_types.len(), fields.len());

        for i in 0..fields.len() {
            self.scope_lock_as_used_later(fields[i].1.free_vars());
        }
        for i in 0..fields.len() {
            self.scope_unlock_as_used_later(fields[i].1.free_vars());
            let field_expr = fields[i].1.clone();
            let field_obj = self.eval_expr(field_expr, false).unwrap();
            let field_val = field_obj.value;
            let offset = if struct_ty.is_box(self.type_env()) {
                1
            } else {
                0
            };
            str_obj = str_obj.insert_field(self, i as u32 + offset, field_val);
        }
        self.build_tail(str_obj, tail)
    }

    fn eval_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        fun_name: &Name,
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        args: &Vec<Arc<ExprNode>>,
        is_io: bool,
        tail: bool,
    ) -> Option<Object<'c>> {
        // Prepare return object.
        let mut obj = {
            let ret_ty = type_tycon(ret_tycon);
            let ret_ty = if is_io {
                make_tuple_ty(vec![make_iostate_ty(), ret_ty])
            } else {
                ret_ty
            };
            create_obj(ret_ty.clone(), &vec![], None, self, Some("allocate_CallC"))
        };

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
                            panic_with_err_src(
                                "Cannot use `()` as a parameter type of C function.",
                                &expr.source,
                            )
                        }
                        c_type.unwrap().into()
                    })
                    .collect::<Vec<_>>();
                let fn_ty = match ret_c_ty {
                    None => {
                        // Void case.
                        self.context.void_type().fn_type(&parm_c_tys, false)
                    }
                    Some(ret_c_ty) => ret_c_ty.fn_type(&parm_c_tys, false),
                };
                self.module.add_function(&fun_name, fn_ty, None)
            }
        };

        // Evaluate arguments
        let mut args = args.clone();
        if is_io {
            args.pop(); // Remove IOState
        }
        let mut arg_objs = vec![];
        for i in 0..args.len() {
            self.scope_lock_as_used_later(args[i].free_vars());
        }
        for i in 0..args.len() {
            self.scope_unlock_as_used_later(args[i].free_vars());
            arg_objs.push(self.eval_expr(args[i].clone(), false).unwrap());
        }

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

        self.build_tail(obj, tail)
    }

    fn eval_array_lit(
        &mut self,
        elems: &Vec<Arc<ExprNode>>,
        array_ty: Arc<TypeNode>,
        tail: bool,
    ) -> Option<Object<'c>> {
        // Make length value
        let len = self.context.i64_type().const_int(elems.len() as u64, false);

        // Allocate
        let array = create_obj(
            array_ty.clone(),
            &vec![],
            Some(len),
            self,
            Some(&format!("array_literal[{}]", array_ty.to_string())),
        );
        let buffer = array.gep_boxed(self, ARRAY_BUF_IDX);

        // Set length.
        let array = array.insert_field(self, ARRAY_LEN_IDX, len);

        // Evaluate each element and store to the array
        for i in 0..elems.len() {
            self.scope_lock_as_used_later(elems[i].free_vars());
        }
        for i in 0..elems.len() {
            self.scope_unlock_as_used_later(elems[i].free_vars());

            // Evaluate element
            let value = self.eval_expr(elems[i].clone(), false).unwrap();

            // Store into the array.
            let idx = self.context.i64_type().const_int(i as u64, false);
            ObjectFieldType::write_to_array_buf(self, None, buffer, idx, value, false);
        }

        self.build_tail(array, tail)
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
            let lam = lam.set_inferred_type(obj_ty.clone());
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

    pub fn implement_symbol(&mut self, sym: &Symbol) {
        let name = &sym.name;
        // Get the function to implement.
        let global_obj = self.global.get(name);
        let sym_fn = match global_obj {
            Some(var) => var.accessor.get_global_fun(),
            None => self.declare_symbol(sym),
        };

        // Create debug info subprogram
        if self.has_di() {
            sym_fn.set_subprogram(self.create_debug_subprogram(
                &sym_fn.get_name().to_str().unwrap(),
                sym.expr.as_ref().unwrap().source.clone(),
            ));
        }

        let obj_ty = &sym.ty;
        if obj_ty.is_funptr() {
            // Implement lambda function.
            let lam_fn = sym_fn;
            let lam = sym.expr.as_ref().unwrap().clone();
            let lam = lam.set_inferred_type(obj_ty.clone());
            self.implement_lambda_function(lam, lam_fn, None);
        } else {
            // Prepare global variable to store the initialized global value.
            let obj_embed_ty = obj_ty.get_embedded_type(self, &vec![]);
            let global_var_name = format!("GlobalVar#{}", name.to_string());
            let global_var = self.module.add_global(obj_embed_ty, None, &global_var_name);
            global_var.set_initializer(&obj_embed_ty.const_zero());
            global_var.set_linkage(Linkage::Internal);
            let global_var_ptr = global_var.as_basic_value_enum().into_pointer_value();

            // Prepare initialized flag.
            let flag_name = format!("InitFlag#{}", name.to_string());
            let (flag_ty, flag_init_val) = if self.config.threaded {
                (
                    pthread_once_init_flag_type(self.context),
                    pthread_once_init_flag_value(self.context),
                )
            } else {
                let ty = self.context.i8_type();
                (ty, ty.const_zero())
            };
            let init_flag = self.module.add_global(flag_ty, None, &flag_name);
            init_flag.set_initializer(&flag_init_val);
            init_flag.set_linkage(Linkage::Internal);
            let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

            // Start to implement accessor function.
            let acc_fn = sym_fn;
            let entry_bb = self.context.append_basic_block(acc_fn, "entry");
            self.builder().position_at_end(entry_bb);

            // Push debug info scope.
            let _di_scope_guard: Option<PopDebugScopeGuard<'_>> =
                if self.has_di() {
                    Some(self.push_debug_scope(
                        acc_fn.get_subprogram().map(|sp| sp.as_debug_info_scope()),
                    ))
                } else {
                    None
                };

            let (init_bb, end_bb, mut init_fun_di_scope_guard) = if !self.config.threaded {
                // In single-threaded mode, we implement `call_once` logic by hand.
                let flag = self
                    .builder()
                    .build_load(flag_ty, init_flag, "load_init_flag")
                    .unwrap()
                    .into_int_value();
                let is_zero = self
                    .builder()
                    .build_int_compare(
                        IntPredicate::EQ,
                        flag,
                        flag.get_type().const_zero(),
                        "flag_is_zero",
                    )
                    .unwrap();
                let init_bb = self.context.append_basic_block(acc_fn, "flag_is_zero");
                let end_bb = self.context.append_basic_block(acc_fn, "flag_is_nonzero");
                self.builder()
                    .build_conditional_branch(is_zero, init_bb, end_bb)
                    .unwrap();

                (init_bb, end_bb, None)
            } else {
                // In threaded mode, we add a function for initialization and call it by `pthread_once`.

                // Add initialization function.
                let init_fn_name = format!("InitOnce#{}", name.to_string());
                let init_fn_type = self.context.void_type().fn_type(&[], false);
                let init_fn =
                    self.module
                        .add_function(&init_fn_name, init_fn_type, Some(Linkage::Internal));

                // Create debug info subprgoram
                if self.has_di() {
                    init_fn.set_subprogram(self.create_debug_subprogram(
                        &init_fn_name,
                        sym.expr.as_ref().unwrap().source.clone(),
                    ));
                }

                // In the accessor function, call `init_fn` by `pthread_once`.
                self.call_runtime(
                    RUNTIME_PTHREAD_ONCE,
                    &[
                        init_flag.into(),
                        init_fn.as_global_value().as_pointer_value().into(),
                    ],
                );
                // The end block of the accessor function.
                let end_bb = self.context.append_basic_block(acc_fn, "end_bb");
                self.builder().build_unconditional_branch(end_bb).unwrap();

                // The entry block for the initialization function.
                let init_bb = self.context.append_basic_block(init_fn, "init_bb");

                // Push debug info scope for initialization function.
                let init_fn_di_scope_guard: Option<PopDebugScopeGuard<'_>> = if self.has_di() {
                    Some(self.push_debug_scope(
                        init_fn.get_subprogram().map(|sp| sp.as_debug_info_scope()),
                    ))
                } else {
                    None
                };
                (init_bb, end_bb, init_fn_di_scope_guard)
            };

            // Implement initialization code.
            {
                // Evaluate object value and store it to the global variable.
                self.builder().position_at_end(init_bb);

                // Execute expression.
                let obj = self
                    .eval_expr(sym.expr.as_ref().unwrap().clone(), false)
                    .unwrap();

                // Mark the object and all object reachable from it as global.
                self.mark_global(obj.clone());

                // Store the result to global_ptr.
                let obj_val = obj.value;
                self.builder().build_store(global_var_ptr, obj_val).unwrap();
            }

            // After initialization,
            if !self.config.threaded {
                // In unthreaded mode, set the initialized flag 1 by hand.
                self.builder()
                    .build_store(init_flag, self.context.i8_type().const_int(1, false))
                    .unwrap();

                // And jump to the end of accessor function.
                self.builder().build_unconditional_branch(end_bb).unwrap();
            } else {
                // In threaded mode, simply return from the initialization function.
                self.builder().build_return(None).unwrap();

                // Drop di_scope_guard for initialization function.
                init_fun_di_scope_guard.take();
                self.set_debug_location(None);
            }

            // In the end of the accessor function, return the object.
            self.builder().position_at_end(end_bb);
            let global_var = self
                .builder()
                .build_load(obj_embed_ty, global_var_ptr, "load_global_var")
                .unwrap();

            if self.sizeof(&global_var.get_type()) == 0 {
                self.builder().build_return(None).unwrap();
            } else {
                self.builder().build_return(Some(&global_var)).unwrap();
            }
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
}
