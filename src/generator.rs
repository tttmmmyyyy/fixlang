// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use std::{cell::RefCell, env, sync::Arc};

use crate::error::error_exit;
use crate::error::error_exit_with_src;
use ast::name::FullName;
use ast::name::Name;
use either::Either;
use inkwell::{
    basic_block::BasicBlock,
    debug_info::{
        AsDIScope, DICompileUnit, DIFile, DIScope, DISubprogram, DIType, DebugInfoBuilder,
    },
    intrinsics::Intrinsic,
    module::Linkage,
    targets::{TargetData, TargetMachine},
    types::{AnyType, BasicMetadataTypeEnum, BasicType},
    values::{BasicMetadataValueEnum, CallSiteValue, StructValue},
};
use misc::flatten_opt;
use misc::Map;
use misc::Set;

use super::*;

#[derive(Clone)]
pub struct Variable<'c> {
    pub ptr: VarValue<'c>,
    used_later: u32,
}

#[derive(Clone)]
pub enum VarValue<'c> {
    Local(Object<'c>),
    Global(FunctionValue<'c>, Arc<TypeNode>),
}

impl<'c> VarValue<'c> {
    // Get pointer.
    pub fn get<'m>(&self, gc: &GenerationContext<'c, 'm>) -> Object<'c> {
        match self {
            VarValue::Local(ptr) => ptr.clone(),
            VarValue::Global(fun, ty) => {
                let ptr = if ty.is_funptr() {
                    fun.as_global_value().as_pointer_value()
                } else {
                    gc.builder()
                        .build_call(fun.clone(), &[], "get_ptr")
                        .try_as_basic_value()
                        .left()
                        .unwrap()
                        .into_pointer_value()
                };
                Object::new(ptr, ty.clone())
            }
        }
    }

    // Get global object's function value.
    pub fn get_global_fun(&self) -> FunctionValue<'c> {
        match self {
            VarValue::Local(_) => panic!("`\"get_global_fun\"` called for local variable."),
            VarValue::Global(fun, _) => *fun,
        }
    }
}

#[derive(Clone)]
pub struct Object<'c> {
    ptr: PointerValue<'c>,
    pub ty: Arc<TypeNode>,
}

impl<'c> Object<'c> {
    pub fn new(ptr: PointerValue<'c>, ty: Arc<TypeNode>) -> Self {
        assert!(ty.free_vars().is_empty());
        Object { ptr, ty }
    }

    // If boxed type, then create Object from pointer.
    // If unboxed type, then store the value to stack and create Object.
    pub fn create_from_value<'m>(
        val: BasicValueEnum<'c>,
        ty: Arc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> Object<'c> {
        let ptr = if ty.is_box(gc.type_env()) || ty.is_funptr() {
            val.into_pointer_value()
        } else {
            let str = ty.get_struct_type(gc, &vec![]);
            let ptr = gc.build_alloca_at_entry(str, "alloca_for_unboxed_obj");
            gc.builder().build_store(ptr, val);
            ptr
        };
        Object::new(ptr, ty)
    }

    pub fn value<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> BasicValueEnum<'c> {
        if self.ty.is_box(gc.type_env()) || self.is_funptr() {
            self.ptr(gc).as_basic_value_enum()
        } else {
            self.load_value_unbox(gc).as_basic_value_enum()
        }
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

    pub fn ptr<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
        if self.is_box(gc.type_env()) {
            gc.cast_pointer(self.ptr, ptr_to_object_type(gc.context))
        } else if self.is_funptr() {
            gc.cast_pointer(self.ptr, opaque_lambda_function_ptr_type(&gc.context))
        } else {
            let str_ty = self.struct_ty(gc);
            gc.cast_pointer(self.ptr, ptr_type(str_ty))
        }
    }

    pub fn debug_embedded_ty<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> DIType<'c> {
        ty_to_debug_embedded_ty(self.ty.clone(), gc)
    }

    pub fn struct_ty<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructType<'c> {
        assert!(!self.is_funptr());
        ty_to_object_ty(&self.ty, &vec![], gc.type_env()).to_struct_type(gc, vec![])
    }

    pub fn load_value_unbox<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructValue<'c> {
        assert!(!self.is_funptr());
        // If applied to boxed value, the following code loads not only the value of the object but also the reference counter. So this function should not be applied to boxed value.
        assert!(self.is_unbox(gc.type_env()));
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder()
            .build_load(ptr, "load_value")
            .into_struct_value()
    }

    pub fn store_value_unbox<'m, V>(&self, gc: &mut GenerationContext<'c, 'm>, value: V)
    where
        V: BasicValue<'c>,
    {
        assert!(!self.is_funptr());
        // If applied to boxed value, the following code stores not only the value of the object but also the reference counter. So this function should not be applied to boxed value.
        assert!(self.is_unbox(gc.type_env()));
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder().build_store(ptr, value);
    }

    pub fn ptr_to_field_nocap<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> PointerValue<'c> {
        assert!(!self.is_funptr());
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder()
            .build_struct_gep(ptr, field_idx, "ptr_to_field_nocap")
            .unwrap()
    }

    pub fn load_field_nocap<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        field_idx: u32,
    ) -> BasicValueEnum<'c> {
        assert!(!self.is_funptr());
        let struct_ty = self.struct_ty(gc);
        gc.load_obj_field(self.ptr, struct_ty, field_idx)
    }

    pub fn store_field_nocap<'m, V>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        field_idx: u32,
        val: V,
    ) where
        V: BasicValue<'c>,
    {
        assert!(!self.is_funptr());
        let struct_ty = self.struct_ty(gc);
        gc.store_obj_field(self.ptr, struct_ty, field_idx, val)
    }

    // Get function pointer to traverser.
    pub fn get_traverser_ptr_boxed<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> PointerValue<'c> {
        assert!(self.is_box(gc.type_env()));
        assert!(!self.is_funptr());
        if self.ty.is_dynamic() {
            self.load_field_nocap(gc, DYNAMIC_OBJ_TRAVARSER_IDX)
                .into_pointer_value()
        } else {
            get_traverser_ptr(&self.ty, &vec![], gc)
        }
    }

    // Get traverser function.
    pub fn get_traverser_unboxed<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> Option<FunctionValue<'c>> {
        assert!(self.is_unbox(gc.type_env()));
        assert!(!self.is_funptr());
        create_traverser(&self.ty, &vec![], gc)
    }

    // Check if the pointer is null.
    pub fn is_null<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> IntValue<'c> {
        gc.builder().build_is_null(self.ptr, "is_null")
    }
}

#[derive(Default)]
pub struct Scope<'c> {
    data: Map<FullName, Vec<Variable<'c>>>,
}

impl<'c> Scope<'c> {
    fn push_local(self: &mut Self, var: &FullName, obj: &Object<'c>) {
        // TODO: add assertion that var is local (or change var to Name).
        if !self.data.contains_key(var) {
            self.data.insert(var.clone(), Default::default());
        }
        self.data.get_mut(var).unwrap().push(Variable {
            ptr: VarValue::Local(obj.clone()),
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

    pub fn get(&self, var: &FullName) -> Variable<'c> {
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
    pub global: Map<FullName, Variable<'c>>,
    type_env: TypeEnv,
    pub target_data: TargetData,
    pub config: Configuration,
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
        let ptr = self.builder().build_alloca(ty, name);
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
            .build_call(func, &[pos.into()], "restore_stack");
    }

    pub fn type_env(&self) -> &TypeEnv {
        &self.type_env
    }

    pub fn sizeof(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data.get_bit_size(ty) / 8
    }

    pub fn ptr_size(&mut self) -> u64 {
        let ptr_ty = self.context.i8_type().ptr_type(AddressSpace::from(0));
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
            error_exit(&format!("Duplicate symbol: {}", name.to_string()));
        } else {
            let used_later = if ty.is_box(self.type_env()) {
                // We do not need to retain global objects. Always move out it.
                0
            } else {
                u32::MAX / 2
            };
            self.global.insert(
                name.clone(),
                Variable {
                    ptr: VarValue::Global(function, ty),
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

    // Get the value of a variable.
    pub fn get_var(&self, var: &FullName) -> Variable<'c> {
        if var.is_local() {
            self.scope.borrow().last().unwrap().get(var)
        } else {
            self.global.get(var).unwrap().clone()
        }
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
        self.scope.borrow().last().unwrap().is_used_later(var)
    }

    // Get field of object in the scope.
    pub fn get_var_field(self: &mut Self, var: &FullName, field_idx: u32) -> BasicValueEnum<'c> {
        let obj = self.get_var(var).ptr.get(self);
        obj.load_field_nocap(self, field_idx)
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

    pub fn get_var_retained_if_used_later(
        &mut self,
        var_name: &FullName,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let var = self.get_var(var_name);
        let obj = var.ptr.get(self);
        if var.used_later > 0 {
            // If used later, clone object.
            self.retain(obj.clone());
            if obj.is_unbox(self.type_env()) {
                // if unboxed, in addition to retain, we also need to store the value to another memory region other than obj,
                // since all unboxed values are treated as like unique object (i.e., the object of refcnt = 1)
                // in the sense that it will be modified by functions such as `set` function of struct.
                if rvo.is_some() {
                    let rvo = rvo.unwrap();
                    let obj_val = obj.value(self);
                    rvo.store_value_unbox(self, obj_val);
                    rvo
                } else {
                    Object::create_from_value(obj.value(self), obj.ty, self)
                }
            } else {
                assert!(rvo.is_none());
                obj
            }
        } else {
            // If this variable is not used later,
            if rvo.is_some() {
                // and if rvo is required, then "move" the ownership of the value to the rvo memory region.
                assert!(obj.is_unbox(self.type_env()));
                let rvo = rvo.unwrap();
                let obj_val = obj.value(self);
                rvo.store_value_unbox(self, obj_val);
                rvo
            } else {
                obj
            }
        }
    }

    pub fn cast_pointer(&self, from: PointerValue<'c>, to: PointerType<'c>) -> PointerValue<'c> {
        if from.get_type() == to {
            from
        } else {
            self.builder().build_pointer_cast(from, to, "pointer_cast")
        }
    }

    // Get pointer to control block of a given object.
    pub fn get_control_block_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        self.cast_pointer(obj, ptr_to_control_block_type(self))
    }

    // Get pointer to reference counter of a given object.
    pub fn get_refcnt_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        let ptr_control_block = self.get_control_block_ptr(obj);
        self.builder()
            .build_struct_gep(ptr_control_block, CTRL_BLK_REFCNT_IDX, "ptr_to_refcnt")
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
            .build_load(ptr_to_refcnt, "refcnt")
            .into_int_value();
        // Jump to shared_bb if refcnt > 1.
        let one = refcnt_type(self.context).const_int(1, false);
        let is_unique: IntValue<'_> =
            self.builder()
                .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique");
        self.builder()
            .build_conditional_branch(is_unique, unique_bb, shared_bb);

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
                .build_load(ptr_to_refcnt, "refcnt")
                .into_int_value();
            refcnt
                .as_instruction_value()
                .unwrap()
                .set_atomic_ordering(inkwell::AtomicOrdering::Monotonic)
                .expect("Set atomic ordering failed");
            // Jump to shared_bb if refcnt > 1.
            let is_unique =
                self.builder()
                    .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique");
            self.builder()
                .build_conditional_branch(is_unique, unique_threaded_bb, shared_bb);

            // Implement unique_threaded_bb.
            self.builder().position_at_end(unique_threaded_bb);
            // We need to build acquire fence to avoid data race between
            // - write / modify operations which will follow in this thread and
            // - read operations done before another thread releases this object.
            self.builder()
                .build_fence(inkwell::AtomicOrdering::Acquire, 0, "");
            // Mark the object as non_threaded.
            self.mark_as_local_one(obj_ptr);
            // And jump to unique_bb.
            self.builder().build_unconditional_branch(unique_bb);
        }

        // Implement global_bb.
        self.builder().position_at_end(global_bb);
        // Jump to shared_bb.
        self.builder().build_unconditional_branch(shared_bb);

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
            .build_load(refcnt_state_ptr, "refcnt_state")
            .into_int_value();

        // Add three basic blocks.
        let local_bb = self.context.append_basic_block(current_func, "local_bb");
        let mut threaded_bb: Option<BasicBlock<'_>> = None;
        let global_bb = self.context.append_basic_block(current_func, "global_bb");

        if !self.config.threaded {
            // In single-threaded program,

            // Check refcnt_state and jump to local_bb if it is equal to `REFCNT_STATE_LOCAL`.
            let is_refcnt_state_local = self.builder().build_int_compare(
                inkwell::IntPredicate::EQ,
                refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
                "is_refcnt_state_local",
            );
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, global_bb);
        } else {
            // In multi-threaded program,
            let th_bb = self.context.append_basic_block(current_func, "threaded_bb");
            threaded_bb = Some(th_bb);
            let threaded_bb = threaded_bb.clone().unwrap();

            let nonlocal_bb = self.context.append_basic_block(current_func, "nonlocal_bb");

            let is_refcnt_state_local = self.builder().build_int_compare(
                inkwell::IntPredicate::EQ,
                refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
                "is_refcnt_state_local",
            );
            self.builder()
                .build_conditional_branch(is_refcnt_state_local, local_bb, nonlocal_bb);

            // Implement nonlocal_bb.
            self.builder().position_at_end(nonlocal_bb);
            let is_refcnt_state_threaded = self.builder().build_int_compare(
                inkwell::IntPredicate::EQ,
                refcnt_state,
                refcnt_state_type(self.context).const_int(REFCNT_STATE_THREADED as u64, false),
                "is_refcnt_state_threaded",
            );
            self.builder().build_conditional_branch(
                is_refcnt_state_threaded,
                threaded_bb,
                global_bb,
            );
        }
        (local_bb, threaded_bb, global_bb)
    }

    // Get pointer to state of reference counter of a given object.
    pub fn get_refcnt_state_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        let ptr_control_block = self.get_control_block_ptr(obj);
        self.builder()
            .build_struct_gep(
                ptr_control_block,
                CTRL_BLK_REFCNT_STATE_IDX,
                "ptr_to_refcnt_state",
            )
            .unwrap()
    }

    // Take an pointer of struct and return the loaded value of a pointer field.
    pub fn load_obj_field(
        &self,
        obj: PointerValue<'c>,
        ty: StructType<'c>,
        index: u32,
    ) -> BasicValueEnum<'c> {
        let ptr = self.cast_pointer(obj, ptr_type(ty));
        let ptr_to_field = self
            .builder()
            .build_struct_gep(ptr, index, "ptr_to_field")
            .unwrap();
        self.builder().build_load(ptr_to_field, "field_value")
    }

    // Take an pointer of struct and store a value into a pointer field.
    pub fn store_obj_field<V>(
        &self,
        obj: PointerValue<'c>,
        ty: StructType<'c>,
        index: u32,
        value: V,
    ) where
        V: BasicValue<'c>,
    {
        let ptr = self.cast_pointer(obj, ptr_type(ty));
        let ptr_to_field = self
            .builder()
            .build_struct_gep(ptr, index, "ptr_to_field")
            .unwrap();
        self.builder().build_store(ptr_to_field, value);
    }

    // Take a lambda object and return function pointer.
    fn get_lambda_func_ptr(&mut self, obj: Object<'c>) -> PointerValue<'c> {
        // Get the pointer value.
        let ptr = if obj.ty.is_closure() {
            obj.load_field_nocap(self, CLOSURE_FUNPTR_IDX)
                .into_pointer_value()
        } else if obj.ty.is_funptr() {
            obj.ptr(self)
        } else {
            panic!()
        };

        // Cast to function pointer type.
        let func_ptr_ty = lambda_function_type(&obj.ty, self).ptr_type(AddressSpace::from(0));
        self.cast_pointer(ptr, func_ptr_ty)
    }

    // Apply objects to a lambda.
    pub fn apply_lambda(
        &mut self,
        fun: Object<'c>,
        args: Vec<Object<'c>>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let src_tys = fun.ty.get_lambda_srcs();
        let ret_ty = fun.ty.get_lambda_dst();

        // Validate arguments.
        assert!(fun.ty.is_closure() || fun.ty.is_funptr());
        assert_eq!(args.len(), src_tys.len());
        for i in 0..args.len() {
            assert_eq!(args[i].ty, src_tys[i])
        }
        if rvo.is_some() {
            assert_eq!(rvo.clone().unwrap().ty, ret_ty);
        }

        // If argument is unboxed, load it.
        let args = args.iter().map(|arg| arg.value(self)).collect::<Vec<_>>();

        // Get function.
        let ptr_to_func = self.get_lambda_func_ptr(fun.clone());
        let func = CallableValue::try_from(ptr_to_func).unwrap();

        // Call function.
        if ret_ty.is_unbox(self.type_env()) {
            // If return type is unboxed, perform return value optimization.
            let rvo = if rvo.is_none() {
                // Allocate memory region for rvo here.
                allocate_obj(
                    ret_ty.clone(),
                    &vec![],
                    None,
                    self,
                    Some(&format!("alloca_rvo_{}", ret_ty.to_string())),
                )
            } else {
                rvo.unwrap()
            };
            let rvo_ptr = rvo.ptr(self);
            let rvo_ptr = self.cast_pointer(rvo_ptr, ptr_to_object_type(self.context));

            // Call function pointer with arguments, CAP if closure, and rvo.
            let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
            for arg in args {
                call_args.push(arg.into())
            }
            if fun.ty.is_closure() {
                call_args.push(fun.load_field_nocap(self, CLOSURE_CAPTURE_IDX).into());
            }
            call_args.push(rvo_ptr.into());

            let ret = self.builder().build_call(func, &call_args, "call_lambda");
            ret.set_tail_call(!self.has_di());
            rvo
        } else {
            // If return type is boxed,
            assert!(rvo.is_none());

            // Call function pointer with arguments, CAP if closure.
            let mut call_args: Vec<BasicMetadataValueEnum> = vec![];
            for arg in args {
                call_args.push(arg.into())
            }
            if fun.ty.is_closure() {
                call_args.push(fun.load_field_nocap(self, CLOSURE_CAPTURE_IDX).into());
            }

            let ret = self.builder().build_call(func, &call_args, "call_lambda");
            ret.set_tail_call(!self.has_di());
            let ret = ret.try_as_basic_value().unwrap_left();
            Object::create_from_value(ret, ret_ty, self)
        }
    }

    // Retain an object.
    pub fn retain(&mut self, obj: Object<'c>) {
        if obj.is_box(self.type_env()) {
            let cont_bb = if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.

                // Dynamic object can be null.
                let current_bb = self.builder().get_insert_block().unwrap();
                let current_func = current_bb.get_parent().unwrap();
                let nonnull_bb = self
                    .context
                    .append_basic_block(current_func, "nonnull_in_retain_dynamic");
                let cont_bb = self
                    .context
                    .append_basic_block(current_func, "cont_in_retain_dynamic");

                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder()
                    .build_conditional_branch(is_null, cont_bb, nonnull_bb);

                // Implement nonnull_bb.
                self.builder().position_at_end(nonnull_bb);

                Some(cont_bb)
            } else {
                None
            };

            let obj_ptr = obj.ptr(self);
            self.call_runtime(RUNTIME_RETAIN_BOXED_OBJECT, &[obj_ptr.into()]);

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                self.builder().build_unconditional_branch(cont_bb.unwrap());
                self.builder().position_at_end(cont_bb.unwrap());
            }
        } else {
            // When the object is unboxed,
            let obj_type = ty_to_object_ty(&obj.ty, &vec![], self.type_env());
            let struct_type = obj_type.to_struct_type(self, vec![]);
            let ptr = obj.ptr(self);
            let ptr = self.cast_pointer(ptr, ptr_type(struct_type));
            let mut union_tag: Option<IntValue<'c>> = None;
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
                    ObjectFieldType::SubObject(ty, is_punched) => {
                        if *is_punched {
                            continue;
                        }
                        let ptr = if ty.is_box(self.type_env()) {
                            self.load_obj_field(ptr, struct_type, i as u32)
                                .into_pointer_value()
                        } else {
                            self.builder()
                                .build_struct_gep(ptr, i as u32, &format!("ptr_to_{}th_field", i))
                                .unwrap()
                        };
                        self.retain(Object::new(ptr, ty.clone()));
                    }
                    ObjectFieldType::UnionBuf(_) => {
                        let buf = obj.ptr_to_field_nocap(self, i as u32);
                        ObjectFieldType::retain_union_buf(
                            self,
                            buf,
                            union_tag.unwrap(),
                            &obj.ty.field_types(self.type_env()),
                        );
                    }
                    ObjectFieldType::UnionTag => {
                        union_tag = Some(
                            self.load_obj_field(ptr, struct_type, i as u32)
                                .into_int_value(),
                        );
                    }
                    ObjectFieldType::Array(_) => unreachable!(),
                }
            }
        }
    }

    // Release or mark global or mark threaded nonnull boxed object.
    pub fn release_or_mark_nonnull_boxed(
        &mut self,
        obj: &Object<'c>,
        work_type: TraverserWorkType,
    ) {
        // If the work is release, and the object's type is Std::Destructor, then call destructor when the refcnt is one.
        if work_type == TraverserWorkType::release() && obj.is_destructor_object() {
            // Branch by whether or not the reference counter is one.
            let obj_ptr = obj.ptr(self);
            let (unique_bb, shared_bb) = self.build_branch_by_is_unique(obj_ptr);

            // If reference counter is one, call destructor.
            self.builder().position_at_end(unique_bb);
            let value = ObjectFieldType::get_struct_field_noclone(
                self,
                obj,
                DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
            );
            let dtor = ObjectFieldType::get_struct_field_noclone(
                self,
                obj,
                DESTRUCTOR_OBJECT_DTOR_FIELD_IDX,
            );
            self.retain(dtor.clone());
            let io_act = self.apply_lambda(dtor, vec![value], None);
            let res = run_io_value(self, &io_act, None);
            ObjectFieldType::set_struct_field_norelease(
                self,
                obj,
                DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
                &res,
            );
            self.builder().build_unconditional_branch(shared_bb);

            self.builder().position_at_end(shared_bb);
        }

        let ptr = obj.ptr(self);
        let ptr = self.cast_pointer(ptr, ptr_to_object_type(self.context));
        let traverser = obj.get_traverser_ptr_boxed(self);
        self.call_runtime(
            work_type.runtime_function(),
            &[ptr.into(), traverser.into()],
        );
    }

    // Release or mark global or mark threaded an object.
    pub fn release_or_mark(&mut self, obj: Object<'c>, work_type: TraverserWorkType) {
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
                    .build_conditional_branch(is_null, cont_bb, nonnull_bb);

                // Implement nonnull_bb.
                self.builder().position_at_end(nonnull_bb);

                Some(cont_bb)
            } else {
                None
            };

            // If the object is boxed and not dynamic,
            self.release_or_mark_nonnull_boxed(&obj, work_type);

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                self.builder().build_unconditional_branch(cont_bb.unwrap());
                self.builder().position_at_end(cont_bb.unwrap());
            }
        } else if obj.is_funptr() {
            // Nothing to do.
        } else {
            match obj.get_traverser_unboxed(self) {
                Some(traverser) => {
                    // Argument of dtor function is i8*, even when the object is unboxed.
                    let ptr = obj.ptr(self);
                    let ptr: PointerValue<'_> =
                        self.cast_pointer(ptr, ptr_to_object_type(self.context));
                    self.builder().build_call(
                        traverser,
                        &[
                            ptr.into(),
                            traverser_work_type(self.context)
                                .const_int(work_type.0 as u64, false)
                                .into(),
                        ],
                        "call_traverser_of_unboxed",
                    );
                }
                None => {}
            }
        }
    }

    // Release object.
    pub fn release(&mut self, obj: Object<'c>) {
        self.release_or_mark(obj, TraverserWorkType::release())
    }

    // Release nonnull boxed object.
    pub fn release_nonnull_boxed(&mut self, obj: &Object<'c>) {
        self.release_or_mark_nonnull_boxed(obj, TraverserWorkType::release())
    }

    // Mark all objects reachable from `obj` as global.
    pub fn mark_global(&mut self, obj: Object<'c>) {
        self.release_or_mark(obj, TraverserWorkType::mark_global())
    }

    pub fn mark_threaded(&mut self, obj: Object<'c>) {
        self.release_or_mark(obj, TraverserWorkType::mark_threaded())
    }

    pub fn mark_as_local_one(&mut self, ptr: PointerValue<'c>) {
        let ptr_refcnt_state: PointerValue<'_> = self.get_refcnt_state_ptr(ptr);
        // Store `REFCNT_STATE_LOCAL` to `ptr_refcnt_state`.
        self.builder().build_store(
            ptr_refcnt_state,
            refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
        );
    }

    pub fn mark_threaded_one(&mut self, obj_ptr: PointerValue<'c>) {
        let current_bb = self.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let cont_bb = self
            .context
            .append_basic_block(current_func, "cont_bb@mark_threaded");

        // Load refcnt state.
        let ptr_refcnt_state = self.get_refcnt_state_ptr(obj_ptr);
        let refcnt_state = self
            .builder()
            .build_load(ptr_refcnt_state, "refcnt_state")
            .into_int_value();

        // Branch by whether or not the refcnt state is `REFCNT_STATE_LOCAL`.
        let local_bb = self
            .context
            .append_basic_block(current_func, "local_bb@mark_threaded");
        let is_refcnt_state_local = self.builder().build_int_compare(
            inkwell::IntPredicate::EQ,
            refcnt_state,
            refcnt_state_type(self.context).const_int(REFCNT_STATE_LOCAL as u64, false),
            "is_refcnt_state_local",
        );
        self.builder()
            .build_conditional_branch(is_refcnt_state_local, local_bb, cont_bb);

        // Implement local_bb.
        self.builder().position_at_end(local_bb);
        // Store `REFCNT_STATE_THREADED` to `ptr_refcnt_state`.
        self.builder().build_store(
            ptr_refcnt_state,
            refcnt_state_type(self.context).const_int(REFCNT_STATE_THREADED as u64, false),
        );
        self.builder().build_unconditional_branch(cont_bb);

        // Set builder's position as preparation for following implementation.
        self.builder().position_at_end(cont_bb);
    }

    // Mark object as global so that it will not be retained or released.
    pub fn mark_global_one(&mut self, ptr: PointerValue<'c>) {
        let ptr_refcnt_state: PointerValue<'_> = self.get_refcnt_state_ptr(ptr);
        // Store `REFCNT_STATE_GLOBAL` to `ptr_refcnt_state`.
        self.builder().build_store(
            ptr_refcnt_state,
            refcnt_state_type(self.context).const_int(REFCNT_STATE_GLOBAL as u64, false),
        );
    }

    // Print Rust's &str to stderr.
    fn eprint(&self, string: &str) {
        let string_ptr = self.builder().build_global_string_ptr(string, "rust_str");
        let string_ptr = string_ptr.as_pointer_value();
        self.call_runtime(RUNTIME_EPRINT, &[string_ptr.into()]);
    }

    // Panic with Rust's &str (i.e, print string and abort.)
    pub fn panic(&self, string: &str) {
        self.eprint(string);
        self.call_runtime(RUNTIME_ABORT, &[]);
    }

    // Get object id of a object
    pub fn get_obj_id(&self, ptr_to_obj: PointerValue<'c>) -> IntValue<'c> {
        assert!(self.config.sanitize_memory);
        self.load_obj_field(ptr_to_obj, control_block_type(self), CTRL_BLK_OBJ_ID_IDX)
            .into_int_value()
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
        self.builder().build_call(func, args, "call_runtime")
    }

    // Evaluate expression.
    pub fn eval_expr(&mut self, expr: Arc<ExprNode>, rvo: Option<Object<'c>>) -> Object<'c> {
        assert!(expr.ty.as_ref().unwrap().free_vars().is_empty());

        if self.has_di() {
            self.push_debug_location(expr.source.clone())
        };

        let mut ret = match &*expr.expr {
            Expr::Var(var) => self.eval_var(var.clone(), rvo),
            Expr::LLVM(lit) => self.eval_llvm(lit.clone(), expr.ty.clone().unwrap().clone(), rvo),
            Expr::App(lambda, args) => self.eval_app(lambda.clone(), args.clone(), rvo),
            Expr::Lam(_, _) => self.eval_lam(expr.clone(), rvo),
            Expr::Let(pat, bound, expr) => self.eval_let(pat, bound.clone(), expr.clone(), rvo),
            Expr::If(cond_expr, then_expr, else_expr) => {
                self.eval_if(cond_expr.clone(), then_expr.clone(), else_expr.clone(), rvo)
            }
            Expr::TyAnno(e, _) => self.eval_expr(e.clone(), rvo),
            Expr::MakeStruct(_, fields) => {
                let struct_ty = expr.ty.clone().unwrap();
                self.eval_make_struct(fields.clone(), struct_ty, rvo)
            }
            Expr::ArrayLit(elems) => self.eval_array_lit(elems, expr.ty.clone().unwrap(), rvo),
            Expr::FFICall(fun_name, ret_ty, param_tys, args, is_io) => {
                self.eval_ffi_call(&expr, fun_name, ret_ty, param_tys, args, *is_io, rvo)
            }
        };

        if self.has_di() {
            self.pop_debug_location();
        }

        ret.ty = expr.ty.clone().unwrap();
        ret
    }

    // Evaluate variable.
    fn eval_var(&mut self, var: Arc<Var>, rvo: Option<Object<'c>>) -> Object<'c> {
        self.get_var_retained_if_used_later(&var.name, rvo)
    }

    // Evaluate application
    fn eval_app(
        &mut self,
        mut fun: Arc<ExprNode>,
        args: Vec<Arc<ExprNode>>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        // Prepare for borrowing optimization.
        let borrowing_optimization_data = if self.config.perform_borrowing_optimization() {
            borrowing_optimization_evaluating_application(self, fun.clone(), &args)
        } else {
            None
        };

        // Replace fun to the borrowing one.
        if borrowing_optimization_data.is_some() {
            fun = borrowing_optimization_data.as_ref().unwrap().0.clone();
        }

        // Before evaluating `fun`, we lock all variables in arguments as used later.
        for arg in &args {
            self.scope_lock_as_used_later(arg.free_vars());
        }

        // Evaluate the function object.
        let fun_obj = self.eval_expr(fun, None);

        // Evaluate arguments.
        let mut arg_objs = vec![];
        for (i, arg) in args.iter().enumerate() {
            self.scope_unlock_as_used_later(arg.free_vars());

            // Check whether or not the argument should be borrowed.
            let borrow_arg_var = borrowing_optimization_data.is_some()
                && borrowing_optimization_data.as_ref().unwrap().1.contains(&i);
            let arg_obj = if borrow_arg_var {
                // Borrow the argument (which is also a variable expression).
                let var_name = &arg.get_var().name;
                self.get_var(var_name).ptr.get(self)
            } else {
                // Evaluate the argument expression.
                self.eval_expr(arg.clone(), None)
            };
            arg_objs.push(arg_obj)
        }

        // Call the function.
        self.apply_lambda(fun_obj, arg_objs, rvo)
    }

    // Evaluate llvm
    fn eval_llvm(
        &mut self,
        llvm: Arc<InlineLLVM>,
        ty: Arc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        llvm.generator.generate(self, &ty, rvo, &llvm.borrowed_vars)
    }

    // Calculate captured variables and their types of lambda expression.
    // Normalize its orderings.
    pub fn calculate_captured_vars_of_lambda(
        &mut self,
        lam: Arc<ExprNode>,
    ) -> Vec<(FullName, Arc<TypeNode>)> {
        let (args, body) = lam.destructure_lam();

        let mut cap_names = body.free_vars().clone();
        for arg in args {
            cap_names.remove(&arg.name);
        }
        cap_names.remove(&FullName::local(CAP_NAME));

        // We need not and should not capture global variable
        // If we capture global variable, then global recursive function such as
        // "main = |x| if x == 0 then 0 else x + main(x-1)" results in infinite recursion at its initialization.
        let mut cap_vars = cap_names
            .into_iter()
            .filter(|name| name.is_local())
            .map(|name| (name.clone(), self.get_var(&name).ptr.get(self).ty))
            .collect::<Vec<_>>();
        cap_vars.sort_by_key(|(name, _)| name.to_string());

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
        let linkage = if lam_ty.is_funptr() && self.config.separate_compilation() {
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
            let arg_obj = Object::create_from_value(arg_val, arg_ty.clone(), self);
            self.scope_push(&arg.name, &arg_obj);
            arg_objs.push(arg_obj);
        }

        // Get rvo field if return value is unboxed.
        let ret_ty = lam_ty.get_lambda_dst();
        let rvo = if ret_ty.is_unbox(self.type_env()) {
            let rvo_idx = args.len() + if lam_ty.is_closure() { 1 } else { 0 };
            let ptr = lam_fn
                .get_nth_param(rvo_idx as u32)
                .unwrap()
                .into_pointer_value();
            Some(Object::new(ptr, ret_ty))
        } else {
            None
        };

        // Push CAP on scope if lambda is closure.
        let (cap_obj_ptr, cap_obj) = if lam_ty.is_closure() {
            let self_idx = args.len();
            let cap_obj_ptr = lam_fn
                .get_nth_param(self_idx as u32)
                .unwrap()
                .into_pointer_value();
            let cap_obj = Object::new(cap_obj_ptr, make_dynamic_object_ty());
            self.scope_push(&FullName::local(CAP_NAME), &cap_obj);
            (Some(cap_obj_ptr), Some(cap_obj))
        } else {
            (None, None)
        };

        // Push captured objects on scope.
        if lam_ty.is_closure() {
            let cap_obj_ty = make_dynamic_object_ty().get_object_type(&cap_tys, self.type_env());
            let cap_obj_str_ty = cap_obj_ty.to_struct_type(self, vec![]);

            for (i, (cap_name, cap_ty)) in cap_vars.iter().enumerate() {
                let cap_val = self.load_obj_field(
                    cap_obj_ptr.unwrap(),
                    cap_obj_str_ty,
                    i as u32 + DYNAMIC_OBJ_CAP_IDX,
                );
                let cap_obj = Object::create_from_value(cap_val, cap_ty.clone(), self);
                self.retain(cap_obj.clone());
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
        let val = self.eval_expr(body.clone(), rvo.clone());

        // Return lambda function.
        if rvo.is_some() {
            self.builder().build_return(None);
        } else {
            self.builder().build_return(Some(&val.value(self)));
        }
    }

    // Evaluate lambda abstraction.
    fn eval_lam(&mut self, lam: Arc<ExprNode>, rvo: Option<Object<'c>>) -> Object<'c> {
        let (args, body) = lam.destructure_lam();
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
        let name = expr_abs(args.clone(), body.clone(), None).expr.to_string();
        let lam = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(lam_ty.clone(), &vec![], None, self, Some(name.as_str()))
        };

        // Set function pointer to lambda.
        let funptr_idx = if lam_ty.is_closure() {
            CLOSURE_FUNPTR_IDX
        } else {
            0
        };
        let lam_fn_ptr = lam_fn.as_global_value().as_pointer_value();
        let lam_fn_ptr =
            self.cast_pointer(lam_fn_ptr, opaque_lambda_function_ptr_type(&self.context));
        lam.store_field_nocap(self, funptr_idx, lam_fn_ptr);

        if lam_ty.is_closure() {
            // Set captured objects.

            let cap_obj_ptr = if cap_vars.len() > 0 {
                // If some objects are captured,

                // Allocate dynamic object to store captured objects.
                let dynamic_obj_ty = make_dynamic_object_ty();
                let cap_obj = allocate_obj(
                    dynamic_obj_ty.clone(),
                    &cap_tys,
                    None,
                    self,
                    Some(&format!("captured_objects_of_{}", name)),
                );
                let cap_obj_ptr = cap_obj.ptr(self);

                // Get struct type of cap_obj.
                let cap_obj_str_ty = dynamic_obj_ty
                    .get_object_type(&cap_tys, self.type_env())
                    .to_struct_type(self, vec![]);

                // Set captured objects to cap_obj.
                for (i, (cap_name, _cap_ty)) in cap_vars.iter().enumerate() {
                    let cap_obj = self.get_var_retained_if_used_later(cap_name, None);
                    let cap_val = cap_obj.value(self);
                    self.store_obj_field(
                        cap_obj_ptr,
                        cap_obj_str_ty,
                        i as u32 + DYNAMIC_OBJ_CAP_IDX,
                        cap_val,
                    );
                }

                cap_obj.ptr(self)
            } else {
                ptr_to_object_type(self.context).const_null()
            };

            // Store cap_obj to lambda
            lam.store_field_nocap(self, CLOSURE_CAPTURE_IDX, cap_obj_ptr);
        }

        // Return lambda object
        lam
    }

    // Evaluate let
    fn eval_let(
        &mut self,
        pat: &Arc<PatternNode>,
        bound: Arc<ExprNode>,
        val: Arc<ExprNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let vars = pat.pattern.vars();
        let mut used_in_val_except_pat = val.free_vars().clone();
        for v in vars {
            used_in_val_except_pat.remove(&v);
        }
        self.scope_lock_as_used_later(&used_in_val_except_pat);
        let bound = self.eval_expr(bound.clone(), None);
        self.scope_unlock_as_used_later(&used_in_val_except_pat);
        let suboobjs = self.destructure_object_by_pattern(pat, &bound);
        for (var_name, obj) in &suboobjs {
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
        let val_code = self.eval_expr(val.clone(), rvo);
        for (var_name, _) in &suboobjs {
            if val.free_vars().contains(&var_name) {
                self.scope_pop(var_name);
            }
        }
        val_code
    }

    // Destructure object by pattern
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
                let field_indices_rvo = field_to_pat
                    .iter()
                    .map(|(name, _)| (field_to_idx[name], None))
                    .collect::<Vec<_>>();
                let fields = ObjectFieldType::get_struct_fields(self, obj, field_indices_rvo);

                // Match to subpatterns.
                for (i, (_, pat)) in field_to_pat.iter().enumerate() {
                    ret.append(&mut self.destructure_object_by_pattern(&pat, &fields[i]));
                }
            }
            Pattern::Union(tc, field_name, pat) => {
                let union_fields = self
                    .type_env()
                    .tycons
                    .get(tc.as_ref())
                    .unwrap()
                    .fields
                    .iter();
                let field_idx = union_fields
                    .enumerate()
                    .find_map(|(i, f)| if &f.name == field_name { Some(i) } else { None })
                    .unwrap();
                let field_ty = obj.ty.field_types(self.type_env())[field_idx].clone();
                let expect_tag_value = ObjectFieldType::UnionTag
                    .to_basic_type(self, vec![])
                    .into_int_type()
                    .const_int(field_idx as u64, false);
                ObjectFieldType::panic_if_union_tag_unmatch(self, obj.clone(), expect_tag_value);
                let field = ObjectFieldType::get_union_field(self, obj.clone(), &field_ty, None);
                ret.append(&mut self.destructure_object_by_pattern(pat, &field));
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
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let mut used_then_or_else = then_expr.free_vars().clone();
        used_then_or_else.extend(else_expr.free_vars().clone());
        self.scope_lock_as_used_later(&used_then_or_else);
        let ptr_to_cond_obj = self.eval_expr(cond_expr, None);
        self.scope_unlock_as_used_later(&used_then_or_else);
        let cond_val = ptr_to_cond_obj.load_field_nocap(self, 0).into_int_value();
        self.release(ptr_to_cond_obj);
        let cond_val =
            self.builder()
                .build_int_cast(cond_val, self.context.bool_type(), "cond_val_i1");
        let bb = self.builder().get_insert_block().unwrap();
        let func = bb.get_parent().unwrap();
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let cont_bb = self.context.append_basic_block(func, "cont");
        self.builder()
            .build_conditional_branch(cond_val, then_bb, else_bb);

        self.builder().position_at_end(then_bb);
        // Release variables used only in the else block.
        for var_name in &else_expr.free_vars_sorted() {
            // Here we use sorted free variables to fix the binary code.
            if !then_expr.free_vars().contains(var_name) && self.get_var(var_name).used_later == 0 {
                self.release(self.get_var(var_name).ptr.get(self));
            }
        }
        let then_val = self.eval_expr(then_expr.clone(), rvo.clone());
        let then_val_ptr = then_val.ptr(self);
        let then_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(else_bb);
        // Release variables used only in the then block.
        for var_name in &then_expr.free_vars_sorted() {
            // Here we use sorted free variables to fix the binary code.
            if !else_expr.free_vars().contains(var_name) && self.get_var(var_name).used_later == 0 {
                self.release(self.get_var(var_name).ptr.get(self));
            }
        }
        let else_val = self.eval_expr(else_expr, rvo.clone());
        let else_val_ptr = else_val.ptr(self);
        let else_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(cont_bb);
        if rvo.is_none() {
            // If don't perform rvo, then return phi value.
            let phi_ty = if then_val.is_box(self.type_env()) {
                ptr_to_object_type(self.context)
            } else {
                ptr_type(then_val.struct_ty(self))
            };
            let phi = self.builder().build_phi(phi_ty, "phi");
            phi.add_incoming(&[(&then_val_ptr, then_bb), (&else_val_ptr, else_bb)]);
            Object::new(
                phi.as_basic_value().into_pointer_value(),
                then_val.ty.clone(),
            )
        } else {
            // if perform rvo then return rvo
            rvo.unwrap()
        }
    }

    // Evaluate make pair
    fn eval_make_struct(
        &mut self,
        fields: Vec<(Name, Arc<ExprNode>)>,
        struct_ty: Arc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let pair = if rvo.is_some() {
            assert!(struct_ty.is_unbox(self.type_env()));
            rvo.unwrap()
        } else {
            allocate_obj(
                struct_ty.clone(),
                &vec![],
                None,
                self,
                Some("allocate_MakeStruct"),
            )
        };
        let field_types = struct_ty.field_types(self.type_env());
        assert_eq!(field_types.len(), fields.len());

        for i in 0..fields.len() {
            self.scope_lock_as_used_later(fields[i].1.free_vars());
        }
        for i in 0..fields.len() {
            self.scope_unlock_as_used_later(fields[i].1.free_vars());

            let field_expr = fields[i].1.clone();
            let field_ty = field_types[i].clone();
            if field_ty.is_unbox(self.type_env()) {
                let rvo = ObjectFieldType::get_struct_field_noclone(self, &pair, i as u32);
                self.eval_expr(field_expr, Some(rvo));
            } else {
                let field_obj = self.eval_expr(field_expr, None);
                let field_val = field_obj.value(self);
                let offset = if struct_ty.is_box(self.type_env()) {
                    1
                } else {
                    0
                };
                pair.store_field_nocap(self, i as u32 + offset, field_val);
            }
        }
        pair
    }

    fn eval_ffi_call(
        &mut self,
        expr: &Arc<ExprNode>,
        fun_name: &Name,
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        args: &Vec<Arc<ExprNode>>,
        is_io: bool,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        // Prepare return object.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            let ret_ty = type_tycon(ret_tycon);
            let ret_ty = if is_io {
                make_tuple_ty(vec![make_iostate_ty(), ret_ty])
            } else {
                ret_ty
            };
            allocate_obj(ret_ty.clone(), &vec![], None, self, Some("allocate_CallC"))
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
                            error_exit_with_src(
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
            arg_objs.push(self.eval_expr(args[i].clone(), None));
        }

        // Get argment values
        let args_vals = arg_objs
            .iter()
            .map(|obj| obj.load_field_nocap(self, 0).into())
            .collect::<Vec<_>>();

        // Call c function
        let ret_c_val =
            self.builder()
                .build_call(c_fun, &args_vals, &format!("FFI_CALL({})", fun_name));
        match ret_c_val.try_as_basic_value() {
            Either::Left(ret_c_val) => {
                if is_io {
                    let ret_str = type_tycon(ret_tycon).get_struct_type(self, &vec![]);
                    let ret_str_val = ret_str.get_undef();
                    let ret_str_val = self
                        .builder()
                        .build_insert_value(ret_str_val, ret_c_val, 0, "")
                        .unwrap();
                    obj.store_field_nocap(self, 1, ret_str_val)
                } else {
                    obj.store_field_nocap(self, 0, ret_c_val)
                }
            }
            Either::Right(_) => {}
        }

        obj
    }

    fn eval_array_lit(
        &mut self,
        elems: &Vec<Arc<ExprNode>>,
        array_ty: Arc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        assert!(rvo.is_none());

        // Make length value
        let len = self.context.i64_type().const_int(elems.len() as u64, false);

        // Allocate
        let array = allocate_obj(
            array_ty,
            &vec![],
            Some(len),
            self,
            Some(&format!(
                "array_literal[{}]",
                elems
                    .iter()
                    .map(|e| e.expr.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
        let buffer = array.ptr_to_field_nocap(self, ARRAY_BUF_IDX);

        // Set length.
        array.store_field_nocap(self, ARRAY_LEN_IDX, len);

        // Evaluate each element and store to the array
        for i in 0..elems.len() {
            self.scope_lock_as_used_later(elems[i].free_vars());
        }
        for i in 0..elems.len() {
            self.scope_unlock_as_used_later(elems[i].free_vars());

            // Evaluate element
            let value = self.eval_expr(elems[i].clone(), None);

            // Store into the array.
            let idx = self.context.i64_type().const_int(i as u64, false);
            ObjectFieldType::write_to_array_buf(self, None, buffer, idx, value, false);
        }

        array
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
        let storage = if obj.is_unbox(self.type_env()) {
            obj.ptr(self)
        } else {
            // In boxed case, push ptr onto stack.
            let ptr = obj.ptr(self);
            let storage =
                self.build_alloca_at_entry(ptr.get_type(), "alloca@create_debug_local_variable");
            self.builder().build_store(storage, ptr);
            storage
        };

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

    pub fn check_leak(&mut self) {
        if !self.config.sanitize_memory {
            return;
        }
        self.call_runtime(RUNTIME_CHECK_LEAK, &[]);
    }

    pub fn declare_symbol(&mut self, sym: &InstantiatedSymbol) -> FunctionValue<'c> {
        let name = &sym.instantiated_name;
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
            let acc_fn_type = ptr_to_object_type(self.context).fn_type(&[], false);
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

    pub fn implement_symbol(&mut self, sym: &InstantiatedSymbol) {
        let name = &sym.instantiated_name;
        // Get the function to implement.
        let global_obj = self.global.get(name);
        let sym_fn = match global_obj {
            Some(var) => var.ptr.get_global_fun(),
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
            let global_var = global_var.as_basic_value_enum().into_pointer_value();

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
                    .build_load(init_flag, "load_init_flag")
                    .into_int_value();
                let is_zero = self.builder().build_int_compare(
                    IntPredicate::EQ,
                    flag,
                    flag.get_type().const_zero(),
                    "flag_is_zero",
                );
                let init_bb = self.context.append_basic_block(acc_fn, "flag_is_zero");
                let end_bb = self.context.append_basic_block(acc_fn, "flag_is_nonzero");
                self.builder()
                    .build_conditional_branch(is_zero, init_bb, end_bb);

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
                self.builder().build_unconditional_branch(end_bb);

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

                // Prepare memory space for rvo.
                let rvo = if obj_ty.is_unbox(self.type_env()) {
                    Some(Object::new(global_var, obj_ty.clone()))
                } else {
                    None
                };
                // Execute expression.
                let obj = self.eval_expr(sym.expr.as_ref().unwrap().clone(), rvo.clone());

                // Mark the object and all object reachable from it as global.
                self.mark_global(obj.clone());

                // If we didn't rvo, then store the result to global_ptr.
                if rvo.is_none() {
                    let obj_val = obj.value(self);
                    self.builder().build_store(global_var, obj_val);
                }
            }

            // After initialization,
            if !self.config.threaded {
                // In unthreaded mode, set the initialized flag 1 by hand.
                self.builder()
                    .build_store(init_flag, self.context.i8_type().const_int(1, false));

                // And jump to the end of accessor function.
                self.builder().build_unconditional_branch(end_bb);
            } else {
                // In threaded mode, simply return from the initialization function.
                self.builder().build_return(None);

                // Drop di_scope_guard for initialization function.
                init_fun_di_scope_guard.take();
                self.set_debug_location(None);
            }

            // In the end of the accessor function, return the object.
            self.builder().position_at_end(end_bb);
            let ret = if obj_ty.is_box(self.type_env()) {
                self.builder()
                    .build_load(global_var, "ptr_to_obj")
                    .into_pointer_value()
            } else {
                global_var
            };
            let ret = self.cast_pointer(ret, ptr_to_object_type(self.context));
            self.builder().build_return(Some(&ret));
        }
    }
}

pub fn ptr_type<'c>(ty: StructType<'c>) -> PointerType<'c> {
    ty.ptr_type(AddressSpace::from(0))
}
