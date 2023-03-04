// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use std::{cell::RefCell, rc::Rc};

use either::Either;
use inkwell::{
    execution_engine::ExecutionEngine,
    intrinsics::Intrinsic,
    module::Linkage,
    targets::{TargetData, TargetMachine},
    types::AnyType,
    values::{BasicMetadataValueEnum, CallSiteValue, StructValue},
};

use super::*;

#[derive(Clone)]
pub struct Variable<'c> {
    pub ptr: VarValue<'c>,
    used_later: u32,
}

#[derive(Clone)]
pub enum VarValue<'c> {
    Local(Object<'c>),
    Global(FunctionValue<'c>, Rc<TypeNode>),
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
}

#[derive(Clone)]
pub struct Object<'c> {
    ptr: PointerValue<'c>,
    pub ty: Rc<TypeNode>,
}

impl<'c> Object<'c> {
    pub fn new(ptr: PointerValue<'c>, ty: Rc<TypeNode>) -> Self {
        assert!(ty.free_vars().is_empty());
        Object { ptr, ty }
    }

    // If boxed type, then create Object from pointer.
    // If unboxed type, then store the value to stack and create Object.
    pub fn create_from_value<'m>(
        val: BasicValueEnum<'c>,
        ty: Rc<TypeNode>,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> Object<'c> {
        let ptr = if ty.is_box(gc.type_env()) || ty.is_funptr() {
            val.into_pointer_value()
        } else {
            let str = ty.get_struct_type(gc, &vec![]);
            let ptr = gc.builder().build_alloca(str, "alloca_for_unboxed_obj");
            gc.builder().build_store(ptr, val);
            ptr
        };
        Object::new(ptr, ty)
    }

    pub fn value<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> BasicValueEnum<'c> {
        if self.ty.is_box(gc.type_env()) || self.is_funptr() {
            self.ptr(gc).as_basic_value_enum()
        } else {
            self.load_nocap(gc).as_basic_value_enum()
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

    pub fn ptr<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
        if self.is_box(gc.type_env()) {
            gc.cast_pointer(self.ptr, ptr_to_object_type(gc.context))
        } else if self.is_funptr() {
            let fun_ty = lambda_function_type(&self.ty, gc);
            gc.cast_pointer(self.ptr, fun_ty.ptr_type(AddressSpace::from(0)))
        } else {
            let str_ty = self.struct_ty(gc);
            gc.cast_pointer(self.ptr, ptr_type(str_ty))
        }
    }

    pub fn struct_ty<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructType<'c> {
        assert!(!self.is_funptr());
        get_object_type(&self.ty, &vec![], gc.type_env()).to_struct_type(gc)
    }

    pub fn load_nocap<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructValue<'c> {
        assert!(!self.is_funptr());
        let struct_ty = self.struct_ty(gc);
        let ptr = gc.cast_pointer(self.ptr, ptr_type(struct_ty));
        gc.builder()
            .build_load(ptr, "load_unbox")
            .into_struct_value()
    }

    pub fn store_unbox<'m, V>(&self, gc: &mut GenerationContext<'c, 'm>, value: V)
    where
        V: BasicValue<'c>,
    {
        assert!(self.is_unbox(gc.type_env()));
        assert!(!self.is_funptr());
        let ptr = self.ptr(gc);
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

    // Get function pointer to destructor.
    pub fn get_dtor_ptr_boxed<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> PointerValue<'c> {
        assert!(self.is_box(gc.type_env()));
        assert!(!self.is_funptr());
        if self.ty.is_dynamic() {
            self.load_field_nocap(gc, DYNAMIC_OBJ_DTOR_IDX).into_pointer_value()
        } else {
            get_dtor_ptr(&self.ty, &vec![], gc)
        }
    }

    // Get dtor function.
    pub fn get_dtor_unboxed<'m>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> Option<FunctionValue<'c>> {
        assert!(self.is_unbox(gc.type_env()));
        assert!(!self.is_funptr());
        create_dtor(&self.ty, &vec![], gc)
    }

    // Check if the pointer is null.
    pub fn is_null<'m>(&self, gc: &mut GenerationContext<'c, 'm>) -> IntValue<'c> {
        gc.builder().build_is_null(self.ptr, "is_null")
    }
}

#[derive(Default)]
pub struct Scope<'c> {
    data: HashMap<FullName, Vec<Variable<'c>>>,
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

    pub fn get(self: &Self, var: &FullName) -> Variable<'c> {
        self.data.get(var).unwrap().last().unwrap().clone()
    }

    fn modify_used_later(self: &mut Self, vars: &HashSet<FullName>, by: i32) {
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
    fn increment_used_later(self: &mut Self, names: &HashSet<FullName>) {
        self.modify_used_later(names, 1);
    }
    fn decrement_used_later(self: &mut Self, names: &HashSet<FullName>) {
        self.modify_used_later(names, -1);
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
    builders: Rc<RefCell<Vec<Rc<Builder<'c>>>>>,
    scope: Rc<RefCell<Vec<Scope<'c>>>>,
    pub global: HashMap<FullName, Variable<'c>>,
    pub runtimes: HashMap<RuntimeFunctions, FunctionValue<'c>>,
    pub typechecker: Option<TypeCheckContext>,
    pub target: Either<TargetMachine, ExecutionEngine<'c>>,
    target_data_cache: Option<TargetData>,
    pub config: Configuration,
}

pub struct PopBuilderGuard<'c> {
    builders: Rc<RefCell<Vec<Rc<Builder<'c>>>>>,
}

impl<'c> Drop for PopBuilderGuard<'c> {
    fn drop(&mut self) {
        self.builders.borrow_mut().pop().unwrap();
    }
}

pub struct PopScopeGuard<'c> {
    scope: Rc<RefCell<Vec<Scope<'c>>>>,
}

impl<'c> Drop for PopScopeGuard<'c> {
    fn drop(&mut self) {
        self.scope.borrow_mut().pop();
    }
}

impl<'c, 'm> GenerationContext<'c, 'm> {
    // Store stack pointer.
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
    pub fn restore_stack(&mut self, pos: PointerValue<'c>) {
        let intrinsic = Intrinsic::find("llvm.stackrestore").unwrap();
        assert!(!intrinsic.is_overloaded()); // So we don't need to specify type parameters in the next line.
        let func = intrinsic.get_declaration(&self.module, &[]).unwrap();
        self.builder()
            .build_call(func, &[pos.into()], "restore_stack");
    }

    pub fn type_env(&self) -> &TypeEnv {
        &self.typechecker.as_ref().unwrap().type_env
    }

    pub fn target_data(&mut self) -> &TargetData {
        if self.target_data_cache.is_some() {
            return self.target_data_cache.as_ref().unwrap();
        }
        match &self.target {
            Either::Left(tm) => {
                self.target_data_cache = Some(tm.get_target_data());
                self.target_data_cache.as_ref().unwrap()
            }
            Either::Right(ee) => ee.get_target_data(),
        }
    }

    pub fn sizeof(&mut self, ty: &dyn AnyType<'c>) -> u64 {
        self.target_data().get_bit_size(ty) / 8
    }

    // Create new gc.
    pub fn new(
        ctx: &'c Context,
        module: &'m Module<'c>,
        target: Either<TargetMachine, ExecutionEngine<'c>>,
        config: Configuration,
    ) -> Self {
        let ret = Self {
            context: ctx,
            module,
            builders: Rc::new(RefCell::new(vec![Rc::new(ctx.create_builder())])),
            scope: Rc::new(RefCell::new(vec![Default::default()])),
            global: Default::default(),
            runtimes: Default::default(),
            typechecker: None,
            target,
            target_data_cache: None,
            config,
        };
        ret
    }

    // Get builder.
    pub fn builder(&self) -> Rc<Builder<'c>> {
        self.builders.borrow().last().unwrap().clone()
    }

    // Push a new builder.
    pub fn push_builder(&mut self) -> PopBuilderGuard<'c> {
        self.builders
            .borrow_mut()
            .push(Rc::new(self.context.create_builder()));
        PopBuilderGuard {
            builders: self.builders.clone(),
        }
    }

    // Add a global object.
    pub fn add_global_object(
        &mut self,
        name: FullName,
        function: FunctionValue<'c>,
        ty: Rc<TypeNode>,
    ) {
        if self.global.contains_key(&name) {
            error_exit(&format!("duplicate symbol: {}", name.to_string()));
        } else {
            let used_later = if self.config.preretain_global && ty.is_box(self.type_env()) {
                // Global boxed objects are pre-retained, so we do not need to retain. Always move out it.
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

    // Get the value of a variable.
    pub fn get_var(&self, var: &FullName) -> Variable<'c> {
        if var.is_local() {
            self.scope.borrow().last().unwrap().get(var)
        } else {
            self.global.get(var).unwrap().clone()
        }
    }

    // Lock variables in scope to avoid being moved out.
    fn scope_lock_as_used_later(self: &mut Self, names: &HashSet<FullName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .increment_used_later(names);
    }

    // Unlock variables in scope.
    fn scope_unlock_as_used_later(self: &mut Self, names: &HashSet<FullName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .decrement_used_later(names);
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
                // in the sense that it will be modified by functions such as `Struct.set`.
                if rvo.is_some() {
                    let rvo = rvo.unwrap();
                    let obj_val = obj.value(self);
                    rvo.store_unbox(self, obj_val);
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
                rvo.store_unbox(self, obj_val);
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
            .build_struct_gep(ptr_control_block, 0, "ptr_to_refcnt")
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

    // Take an pointer of struct and store a value value into a pointer field.
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
        if obj.ty.is_closure() {
            obj.load_field_nocap(self, CLOSURE_FUNPTR_IDX)
                .into_pointer_value()
        } else if obj.ty.is_funptr() {
            obj.ptr(self)
        } else {
            panic!()
        }
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
            // If return type is unboxed, preform return value optimization.
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
            ret.set_tail_call(true);
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
            ret.set_tail_call(true);
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
                let nonnull_bb = self.context.append_basic_block(current_func, "nonnull_in_retain_dynamic");
                let cont_bb = self.context.append_basic_block(current_func, "cont_in_retain_dynamic");
                
                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder().build_conditional_branch(is_null, cont_bb, nonnull_bb);

                // Implement nonnull_bb.
                self.builder().position_at_end(nonnull_bb);

                Some(cont_bb)
            } else {
                None
            };

            let obj_ptr = obj.ptr(self);
            self.call_runtime(RuntimeFunctions::RetainBoxedObject, &[obj_ptr.into()]);

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                self.builder().build_unconditional_branch(cont_bb.unwrap());
                self.builder().position_at_end(cont_bb.unwrap());
            }
        } else {
            let obj_type = get_object_type(&obj.ty, &vec![], self.type_env());
            let struct_type = obj_type.to_struct_type(self);
            let ptr = obj.ptr(self);
            let ptr = self.cast_pointer(ptr, ptr_type(struct_type));
            let mut union_tag: Option<IntValue<'c>> = None;
            for (i, ft) in obj_type.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::ControlBlock => unreachable!(),
                    ObjectFieldType::DtorFunction => unreachable!(),
                    ObjectFieldType::LambdaFunction(_) => {}
                    ObjectFieldType::I64 => {}
                    ObjectFieldType::I8 => {}
                    ObjectFieldType::SubObject(ty) => {
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
                    ObjectFieldType::ArraySize(_) => unreachable!(),
                }
            }
        }
    }

    // Release nonnull boxed object.
    pub fn release_nonnull_boxed(&mut self, obj: &Object<'c>) {
        let ptr = obj.ptr(self);
        let ptr = self.cast_pointer(ptr, ptr_to_object_type(self.context));
        let dtor = obj.get_dtor_ptr_boxed(self);
        self.call_runtime(
            RuntimeFunctions::ReleaseBoxedObject,
            &[ptr.into(), dtor.into()],
        );
    }

    // Release object.
    pub fn release(&mut self, obj: Object<'c>) {
        if obj.is_box(self.type_env()) {
            let cont_bb = if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.

                // Append basic blocks.
                let current_bb = self.builder().get_insert_block().unwrap();
                let current_func = current_bb.get_parent().unwrap();
                let nonnull_bb = self.context.append_basic_block(current_func, "nonnull_in_release_dynamic");
                let cont_bb = self.context.append_basic_block(current_func, "cont_in_release_dynamic");
                
                // Branch to nonnull_bb if object is not null.
                let is_null = obj.is_null(self);
                self.builder().build_conditional_branch(is_null, cont_bb, nonnull_bb);

                // Implement nonnull_bb.
                self.builder().position_at_end(nonnull_bb);

                Some(cont_bb)
            } else {
                None
            };

            // If the object is boxed and not dynamic, 
            self.release_nonnull_boxed(&obj);

            if obj.is_dynamic_object() {
                // Dynamic object can be null, so build null checking.
                self.builder().build_unconditional_branch(cont_bb.unwrap());
                self.builder().position_at_end(cont_bb.unwrap());
            }
        } else if obj.is_funptr() {
            // Nothing to do.
        } else {
            match obj.get_dtor_unboxed(self) {
                Some(dtor) => {
                    // Argument of dtor function is i8*, even when the object is unboxed.
                    let ptr = obj.ptr(self);
                    let ptr = self.cast_pointer(ptr, ptr_to_object_type(self.context));
                    self.builder()
                        .build_call(dtor, &[ptr.into()], "dtor_of_unboxed");
                }
                None => {}
            }
        }
    }

    // Printf Rust's &str.
    pub fn printf(&self, string: &str) {
        let string_ptr = self.builder().build_global_string_ptr(string, "rust_str");
        let string_ptr = string_ptr.as_pointer_value();
        self.call_runtime(RuntimeFunctions::Printf, &[string_ptr.into()]);
    }

    // Panic with Rust's &str (i.e, print string and abort.)
    pub fn panic(&self, string: &str) {
        self.printf(string);
        self.call_runtime(RuntimeFunctions::Abort, &[]);
    }

    // Get object id of a object
    pub fn get_obj_id(&self, ptr_to_obj: PointerValue<'c>) -> IntValue<'c> {
        assert!(self.config.sanitize_memory);
        self.load_obj_field(ptr_to_obj, control_block_type(self), 1)
            .into_int_value()
    }

    // Call a runtime function.
    pub fn call_runtime(
        &self,
        func: RuntimeFunctions,
        args: &[BasicMetadataValueEnum<'c>],
    ) -> CallSiteValue<'c> {
        self.builder()
            .build_call(*self.runtimes.get(&func).unwrap(), args, "call_runtime")
    }

    // Panics if the given object is shared.
    pub fn panic_if_shared(&mut self, obj: &Object<'c>, msg: &str) {
        // If object is unboxed, it is always unique; do nothing.
        if obj.is_unbox(self.type_env()) {
            return;
        }

        // Get refcnt.
        let refcnt = {
            let control_block = obj.ptr_to_field_nocap(self, 0);
            self.load_obj_field(control_block, control_block_type(self), 0)
                .into_int_value()
        };

        // Jump by refcnt.
        let current_fn = self
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let unique_bb = self
            .context
            .append_basic_block(current_fn, "unique_bb_panic_if_shared");
        let shared_bb = self
            .context
            .append_basic_block(current_fn, "shared_bb_panic_if_shared");
        // Jump to shared_bb if refcnt > 1.
        let one = refcnt_type(self.context).const_int(1, false);
        let is_unique = self.builder().build_int_compare(
            IntPredicate::EQ,
            refcnt,
            one,
            "is_unique_panic_if_shared",
        );
        self.builder()
            .build_conditional_branch(is_unique, unique_bb, shared_bb);

        // In shared_bb, panic.
        self.builder().position_at_end(shared_bb);
        self.panic(msg);
        self.builder().build_unconditional_branch(unique_bb);

        self.builder().position_at_end(unique_bb);
    }

    // Evaluate expression.
    pub fn eval_expr(&mut self, expr: Rc<ExprNode>, rvo: Option<Object<'c>>) -> Object<'c> {
        let expr = expr.set_inferred_type(
            self.typechecker
                .as_ref()
                .unwrap()
                .substitute_type(&expr.inferred_ty.clone().unwrap()),
        );
        assert!(expr.inferred_ty.as_ref().unwrap().free_vars().is_empty());
        let mut ret = match &*expr.expr {
            Expr::Var(var) => self.eval_var(var.clone(), rvo),
            Expr::Lit(lit) => {
                self.eval_lit(lit.clone(), expr.inferred_ty.clone().unwrap().clone(), rvo)
            }
            Expr::App(lambda, args) => self.eval_app(lambda.clone(), args.clone(), rvo),
            Expr::Lam(_, _) => self.eval_lam(expr.clone(), rvo),
            Expr::Let(pat, bound, expr) => self.eval_let(pat, bound.clone(), expr.clone(), rvo),
            Expr::If(cond_expr, then_expr, else_expr) => {
                self.eval_if(cond_expr.clone(), then_expr.clone(), else_expr.clone(), rvo)
            }
            Expr::TyAnno(e, _) => self.eval_expr(e.clone(), rvo),
            Expr::MakeStruct(_, fields) => {
                let struct_ty = expr.inferred_ty.clone().unwrap();
                self.eval_make_struct(fields.clone(), struct_ty, rvo)
            }
            Expr::ArrayLit(elems) => {
                self.eval_array_lit(elems, expr.inferred_ty.clone().unwrap(), rvo)
            }
        };
        ret.ty = expr.inferred_ty.clone().unwrap();
        ret
    }

    // Evaluate variable.
    fn eval_var(&mut self, var: Rc<Var>, rvo: Option<Object<'c>>) -> Object<'c> {
        self.get_var_retained_if_used_later(&var.name, rvo)
    }

    // Evaluate application
    fn eval_app(
        &mut self,
        fun: Rc<ExprNode>,
        args: Vec<Rc<ExprNode>>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        for arg in &args {
            self.scope_lock_as_used_later(arg.free_vars());
        }
        let fun_obj = self.eval_expr(fun, None);
        let mut arg_objs = vec![];
        for arg in &args {
            self.scope_unlock_as_used_later(arg.free_vars());
            arg_objs.push(self.eval_expr(arg.clone(), None))
        }
        self.apply_lambda(fun_obj, arg_objs, rvo)
    }

    // Evaluate literal
    fn eval_lit(
        &mut self,
        lit: Rc<Literal>,
        ty: Rc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        (lit.generator)(self, &ty, rvo)
    }

    // Calculate captured variables and their types of lambda expression.
    // Normalize it's orderings.
    pub fn calculate_captured_vars_of_lambda(
        &mut self,
        lam: Rc<ExprNode>,
    ) -> Vec<(FullName, Rc<TypeNode>)> {
        let (args, body) = lam.destructure_lam();

        let mut cap_names = body.free_vars().clone();
        for arg in args {
            cap_names.remove(&arg.name);
        }
        cap_names.remove(&FullName::local(CAP_NAME));

        // We need not and should not capture global variable
        // If we capture global variable, then global recursive function such as
        // "main = |x| if x == 0 then 0 else x + main(x-1)" results in infinite recursion at it's initialization.
        let mut cap_vars = cap_names
            .into_iter()
            .filter(|name| name.is_local())
            .map(|name| (name.clone(), self.get_var(&name).ptr.get(self).ty))
            .collect::<Vec<_>>();
        cap_vars.sort_by_key(|(name, _)| name.to_string());

        // Validation
        let lam_ty = lam.inferred_ty.clone().unwrap();
        assert!(!lam_ty.is_funptr() || cap_vars.len() == 0); // Function poitners cannot capture objects.

        cap_vars
    }

    // Declare function of lambda expression
    pub fn declare_lambda_function(&mut self, lam: Rc<ExprNode>) -> FunctionValue<'c> {
        let lam_ty = lam.inferred_ty.clone().unwrap();
        let lam_fn_ty = lambda_function_type(&lam_ty, self);
        let lam_fn = self.module.add_function(
            &format!("lambda_{}", lam_ty.to_string_normalize()),
            lam_fn_ty,
            Some(Linkage::Internal),
        );
        lam_fn
    }

    // Implement function of lambda expression
    pub fn implement_lambda_function(
        &mut self,
        lam: Rc<ExprNode>,
        lam_fn: FunctionValue<'c>,
        cap_vars: Option<Vec<(FullName, Rc<TypeNode>)>>,
    ) {
        let lam_ty = lam.inferred_ty.clone().unwrap();
        let (args, body) = lam.destructure_lam();
        let cap_vars = if cap_vars.is_some() {
            cap_vars.unwrap()
        } else {
            self.calculate_captured_vars_of_lambda(lam)
        };
        let cap_tys = cap_vars
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect::<Vec<_>>();

        // Create new builder and set up
        let _builder_guard = self.push_builder();
        let bb = self.context.append_basic_block(lam_fn, "entry");
        self.builder().position_at_end(bb);

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
            let cap_obj_str_ty = cap_obj_ty.to_struct_type(self);

            for (i, (cap_name, cap_ty)) in cap_vars.iter().enumerate() {
                let cap_val = self.load_obj_field(
                    cap_obj_ptr.unwrap(),
                    cap_obj_str_ty,
                    i as u32 + DYNAMIC_OBJ_CAP_IDX,
                );
                let cap_obj = Object::create_from_value(cap_val, cap_ty.clone(), self);
                self.retain(cap_obj.clone());
                self.scope_push(cap_name, &cap_obj);
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
    fn eval_lam(&mut self, lam: Rc<ExprNode>, rvo: Option<Object<'c>>) -> Object<'c> {
        let (args, body) = lam.destructure_lam();
        let lam_ty = lam.inferred_ty.clone().unwrap();

        // Calculate captured variables.
        let cap_vars = self.calculate_captured_vars_of_lambda(lam.clone());
        let cap_tys = cap_vars
            .iter()
            .map(|(_name, ty)| ty.clone())
            .collect::<Vec<_>>();

        // Define lambda function
        let lam_fn = self.declare_lambda_function(lam.clone());
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
        lam.store_field_nocap(self, funptr_idx, lam_fn.as_global_value().as_pointer_value());

        if lam_ty.is_closure() {
            // Set captured objects. 

            let cap_obj_ptr = if cap_vars.len() > 0 {
                // If some objects are captured, 

                // Allocate dynamic object to store captured objects.
                let dynamic_obj_ty = make_dynamic_object_ty();
                let cap_obj = allocate_obj(dynamic_obj_ty.clone(), &cap_tys, None, self, Some(&format!("captured_objects_of_{}", name)));
                let cap_obj_ptr = cap_obj.ptr(self);

                // Get struct type of cap_obj.
                let cap_obj_str_ty = dynamic_obj_ty.get_object_type(&cap_tys, self.type_env()).to_struct_type(self);

                // Set captured objects to cap_obj.
                for (i, (cap_name, _cap_ty)) in cap_vars.iter().enumerate() {
                    let cap_obj = self.get_var_retained_if_used_later(cap_name, None);
                    let cap_val = cap_obj.value(self);
                    self.store_obj_field(cap_obj_ptr, cap_obj_str_ty, i as u32 + DYNAMIC_OBJ_CAP_IDX, cap_val);
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
        pat: &Rc<Pattern>,
        bound: Rc<ExprNode>,
        val: Rc<ExprNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let vars = pat.vars();
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
        pat: &Rc<Pattern>,
        obj: &Object<'c>,
    ) -> Vec<(FullName, Object<'c>)> {
        let mut ret = vec![];
        match pat.as_ref() {
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
                    .collect::<HashMap<_, _>>();

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
                    .to_basic_type(self)
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
        cond_expr: Rc<ExprNode>,
        then_expr: Rc<ExprNode>,
        else_expr: Rc<ExprNode>,
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
        for var_name in else_expr.free_vars() {
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
        for var_name in then_expr.free_vars() {
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
        fields: Vec<(Name, Rc<ExprNode>)>,
        struct_ty: Rc<TypeNode>,
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

    fn eval_array_lit(
        &mut self,
        elems: &Vec<Rc<ExprNode>>,
        array_ty: Rc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        assert!(rvo.is_none());

        // Make size value
        let size = self.context.i64_type().const_int(elems.len() as u64, false);

        // Allocate
        let array = allocate_obj(
            array_ty,
            &vec![],
            Some(size),
            self,
            Some(&format!("array_literal[{}]", elems.len())),
        );
        let buffer = array.ptr_to_field_nocap(self, ARRAY_BUF_IDX);

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
            ObjectFieldType::write_to_array_buf(self, size, buffer, idx, value, false);
        }

        array
    }
}

pub fn ptr_type<'c>(ty: StructType<'c>) -> PointerType<'c> {
    ty.ptr_type(AddressSpace::from(0))
}

pub static CAP_NAME: &str = "%CAP";
