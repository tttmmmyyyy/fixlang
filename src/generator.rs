// generator module
// --
// GenerationContext struct, code generation and convenient functions.

use std::{cell::RefCell, rc::Rc};

use inkwell::values::{BasicMetadataValueEnum, CallSiteValue};

use super::*;

#[derive(Clone)]
pub struct Variable<'c> {
    pub ptr: PointerValue<'c>,
    used_later: u32,
    // TODO: add type for removing destructor pointers.
}

#[derive(Default)]
pub struct Scope<'c> {
    data: HashMap<NameSpacedName, Vec<Variable<'c>>>,
}

impl<'c> Scope<'c> {
    fn push(self: &mut Self, var: &NameSpacedName, code: &PointerValue<'c>) {
        if !self.data.contains_key(var) {
            self.data.insert(var.clone(), Default::default());
        }
        self.data.get_mut(var).unwrap().push(Variable {
            ptr: code.clone(),
            used_later: 0,
        });
    }

    fn pop(self: &mut Self, var: &NameSpacedName) {
        self.data.get_mut(var).unwrap().pop();
        if self.data.get(var).unwrap().is_empty() {
            self.data.remove(var);
        }
    }

    pub fn get(self: &Self, var: &NameSpacedName) -> Variable<'c> {
        self.data.get(var).unwrap().last().unwrap().clone()
    }

    pub fn get_field<'m, 'b>(
        self: &Self,
        var: &NameSpacedName,
        field_idx: u32,
        ty: StructType<'c>,
        gc: &GenerationContext<'c, 'm>,
    ) -> BasicValueEnum<'c> {
        let expr = self.get(var);
        gc.load_obj_field(expr.ptr, ty, field_idx)
    }

    fn modify_used_later(self: &mut Self, vars: &HashSet<NameSpacedName>, by: i32) {
        for var in vars {
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
    fn increment_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.modify_used_later(names, 1);
    }
    fn decrement_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
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
    pub runtimes: HashMap<RuntimeFunctions, FunctionValue<'c>>,
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
    // Create new gc.
    pub fn new(ctx: &'c Context, module: &'m Module<'c>) -> Self {
        let ret = Self {
            context: ctx,
            module,
            builders: Rc::new(RefCell::new(vec![Rc::new(ctx.create_builder())])),
            scope: Rc::new(RefCell::new(vec![Default::default()])),
            runtimes: Default::default(),
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

    // Push a new scope.
    pub fn push_scope(&mut self) -> PopScopeGuard<'c> {
        self.scope.borrow_mut().push(Default::default());
        PopScopeGuard {
            scope: self.scope.clone(),
        }
    }

    // Get a variable from scope.
    pub fn scope_get(&self, var: &NameSpacedName) -> Variable<'c> {
        self.scope.borrow().last().unwrap().get(var)
    }

    // Lock variables in scope from moved out.
    fn scope_lock_as_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .increment_used_later(names);
    }

    // Release lock variables in scope from moved out.
    fn scope_unlock_as_used_later(self: &mut Self, names: &HashSet<NameSpacedName>) {
        self.scope
            .borrow_mut()
            .last_mut()
            .unwrap()
            .decrement_used_later(names);
    }

    // Get field of object in the scope.
    pub fn scope_get_field(
        self: &Self,
        var: &NameSpacedName,
        field_idx: u32,
        ty: StructType<'c>,
    ) -> BasicValueEnum<'c> {
        self.scope
            .borrow_mut()
            .last()
            .unwrap()
            .get_field(var, field_idx, ty, self)
    }

    // Push scope.
    fn scope_push(self: &mut Self, var: &NameSpacedName, code: &PointerValue<'c>) {
        self.scope.borrow_mut().last_mut().unwrap().push(var, code)
    }

    // Pop scope.
    fn scope_pop(self: &mut Self, var: &NameSpacedName) {
        self.scope.borrow_mut().last_mut().unwrap().pop(var);
    }

    pub fn get_var_retained_if_used_later(&mut self, var: &NameSpacedName) -> PointerValue<'c> {
        let var = self.scope_get(var);
        let code = var.ptr;
        if var.used_later > 0 {
            self.retain(code);
        }
        code
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
        self.cast_pointer(obj, ptr_to_control_block_type(self.context))
    }

    // Get pointer to reference counter of a given object.
    pub fn get_refcnt_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        let ptr_control_block = self.get_control_block_ptr(obj);
        self.builder()
            .build_struct_gep(ptr_control_block, 0, "ptr_to_refcnt")
            .unwrap()
    }

    // Call dtor of object.
    pub fn call_dtor(&self, obj: PointerValue<'c>) {
        let ptr_to_dtor = self
            .load_obj_field(obj, control_block_type(self.context), 1)
            .into_pointer_value();
        let dtor_func = CallableValue::try_from(ptr_to_dtor).unwrap();
        self.builder()
            .build_call(dtor_func, &[obj.into()], "call_dtor");
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

    // Take a closure object and return function pointer.
    fn get_lambda_func_ptr(&self, obj: PointerValue<'c>) -> PointerValue<'c> {
        let lam_ty = lambda_type(self.context);
        self.load_obj_field(obj, lam_ty, 1).into_pointer_value()
    }

    // Apply a object to a closure.
    pub fn apply_lambda(
        &self,
        ptr_to_lambda: PointerValue<'c>,
        ptr_to_arg: PointerValue<'c>,
    ) -> PointerValue<'c> {
        let ptr_to_func = self.get_lambda_func_ptr(ptr_to_lambda);
        let lambda_func = CallableValue::try_from(ptr_to_func).unwrap();
        let ret = self.builder().build_call(
            lambda_func,
            &[ptr_to_arg.into(), ptr_to_lambda.into()],
            "call_lambda",
        );
        ret.set_tail_call(true);
        ret.try_as_basic_value().unwrap_left().into_pointer_value()
    }

    // Retain object.
    pub fn retain(&self, ptr_to_obj: PointerValue<'c>) {
        let ptr_to_obj = self.cast_pointer(ptr_to_obj, ptr_to_object_type(self.context));
        self.call_runtime(RuntimeFunctions::RetainObj, &[ptr_to_obj.clone().into()]);
    }

    // Release object.
    pub fn release(&self, ptr_to_obj: PointerValue<'c>) {
        let ptr_to_obj = self.cast_pointer(ptr_to_obj, ptr_to_object_type(self.context));
        self.call_runtime(RuntimeFunctions::ReleaseObj, &[ptr_to_obj.clone().into()]);
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
        assert!(SANITIZE_MEMORY);
        self.load_obj_field(ptr_to_obj, control_block_type(self.context), 2)
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

    // Evaluate expression.
    pub fn eval_expr(&mut self, expr: Arc<ExprNode>) -> PointerValue<'c> {
        let ret = match &*expr.expr {
            Expr::Var(var) => self.eval_var(var.clone()),
            Expr::Lit(lit) => self.eval_lit(lit.clone()),
            Expr::App(lambda, arg) => self.eval_app(lambda.clone(), arg.clone()),
            Expr::Lam(arg, val) => self.eval_lam(arg.clone(), val.clone()),
            Expr::Let(var, bound, expr) => self.eval_let(var.clone(), bound.clone(), expr.clone()),
            Expr::If(cond_expr, then_expr, else_expr) => {
                self.eval_if(cond_expr.clone(), then_expr.clone(), else_expr.clone())
            }
            Expr::TyAnno(e, _) => self.eval_expr(e.clone()),
        };
        self.cast_pointer(ret, ptr_to_object_type(self.context))
    }

    // Evaluate variable.
    fn eval_var(&mut self, var: Arc<Var>) -> PointerValue<'c> {
        self.get_var_retained_if_used_later(&var.namespaced_name())
    }

    // Evaluate application
    fn eval_app(&mut self, lambda: Arc<ExprNode>, arg: Arc<ExprNode>) -> PointerValue<'c> {
        self.scope_lock_as_used_later(arg.free_vars());
        let lambda_code = self.eval_expr(lambda);
        self.scope_unlock_as_used_later(arg.free_vars());
        let arg_code = self.eval_expr(arg);
        self.apply_lambda(lambda_code, arg_code)
    }

    // Evaluate literal
    fn eval_lit(&mut self, lit: Arc<Literal>) -> PointerValue<'c> {
        (lit.generator)(self)
    }

    // Evaluate lambda abstraction.
    fn eval_lam(&mut self, arg: Arc<Var>, val: Arc<ExprNode>) -> PointerValue<'c> {
        let context = self.context;
        let module = self.module;
        // Fix ordering of captured names
        let mut captured_names = val.free_vars().clone();
        captured_names.remove(&arg.namespaced_name());
        captured_names.remove(&NameSpacedName::local(SELF_NAME));
        let captured_names: Vec<NameSpacedName> = captured_names.into_iter().collect();
        // Determine the type of closure
        let mut field_types = vec![
            ObjectFieldType::ControlBlock,
            ObjectFieldType::LambdaFunction,
        ];
        for _ in captured_names.iter() {
            field_types.push(ObjectFieldType::SubObject);
        }
        let obj_type = ObjectType { field_types };
        let closure_ty = obj_type.to_struct_type(context);
        // Declare lambda function
        let lam_fn_ty = lambda_function_type(context);
        let lam_fn = module.add_function("lambda", lam_fn_ty, None);
        // Implement lambda function
        {
            // Create new builder and set up
            let _builder_guard = self.push_builder();
            let bb = context.append_basic_block(lam_fn, "entry");
            self.builder().position_at_end(bb);

            // Create new scope
            let _scope_guard = self.push_scope();

            // Set up new scope
            let arg_ptr = lam_fn.get_first_param().unwrap().into_pointer_value();
            self.scope_push(&arg.namespaced_name(), &arg_ptr);
            let closure_obj = lam_fn.get_nth_param(1).unwrap().into_pointer_value();
            self.scope_push(&NameSpacedName::local(SELF_NAME), &closure_obj);
            for (i, cap_name) in captured_names.iter().enumerate() {
                let cap_obj = self
                    .load_obj_field(closure_obj, closure_ty, i as u32 + 2)
                    .into_pointer_value();
                self.scope_push(cap_name, &cap_obj);
            }
            // Retain captured objects
            for cap_name in &captured_names {
                let ptr = self.scope_get(cap_name).ptr;
                self.retain(ptr);
            }
            // Release SELF and arg if unused
            if !val.free_vars().contains(&NameSpacedName::local(SELF_NAME)) {
                self.release(closure_obj);
            }
            if !val.free_vars().contains(&arg.namespaced_name()) {
                self.release(arg_ptr);
            }
            // Generate value
            let val = self.eval_expr(val.clone());
            // Return result
            let ptr = self.cast_pointer(val, ptr_to_object_type(self.context));
            self.builder().build_return(Some(&ptr));
        }
        // Allocate and set up closure
        let name = expr_abs(arg, val, None).expr.to_string();
        let obj = obj_type.create_obj(self, Some(name.as_str()));
        self.store_obj_field(
            obj,
            closure_ty,
            1,
            lam_fn.as_global_value().as_pointer_value(),
        );
        for (i, cap) in captured_names.iter().enumerate() {
            let ptr = self.get_var_retained_if_used_later(cap);
            self.store_obj_field(obj, closure_ty, i as u32 + 2, ptr);
        }
        // Return closure object
        obj
    }

    // Evaluate let
    fn eval_let(
        &mut self,
        var: Arc<Var>,
        bound: Arc<ExprNode>,
        val: Arc<ExprNode>,
    ) -> PointerValue<'c> {
        let var_name = &var.namespaced_name();
        let mut used_in_val_except_var = val.free_vars().clone();
        used_in_val_except_var.remove(var_name);
        self.scope_lock_as_used_later(&used_in_val_except_var);
        let bound_code = self.eval_expr(bound.clone());
        self.scope_unlock_as_used_later(&used_in_val_except_var);
        self.scope_push(&var_name, &bound_code);
        if !val.free_vars().contains(var_name) {
            self.release(bound_code);
        }
        let val_code = self.eval_expr(val.clone());
        self.scope_pop(&var_name);
        val_code
    }

    // Evaluate if
    fn eval_if(
        &mut self,
        cond_expr: Arc<ExprNode>,
        then_expr: Arc<ExprNode>,
        else_expr: Arc<ExprNode>,
    ) -> PointerValue<'c> {
        let mut used_then_or_else = then_expr.free_vars().clone();
        used_then_or_else.extend(else_expr.free_vars().clone());
        self.scope_lock_as_used_later(&used_then_or_else);
        let ptr_to_cond_obj = self.eval_expr(cond_expr);
        self.scope_unlock_as_used_later(&used_then_or_else);
        let bool_ty = ObjectType::bool_obj_type().to_struct_type(self.context);
        let cond_val = self
            .load_obj_field(ptr_to_cond_obj, bool_ty, 1)
            .into_int_value();
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
            if !then_expr.free_vars().contains(var_name) && self.scope_get(var_name).used_later == 0
            {
                self.release(self.scope_get(var_name).ptr);
            }
        }
        let then_code = self.eval_expr(then_expr.clone());
        let then_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(else_bb);
        // Release variables used only in the then block.
        for var_name in then_expr.free_vars() {
            if !else_expr.free_vars().contains(var_name) && self.scope_get(var_name).used_later == 0
            {
                self.release(self.scope_get(var_name).ptr);
            }
        }
        let else_code = self.eval_expr(else_expr);
        let else_bb = self.builder().get_insert_block().unwrap();
        self.builder().build_unconditional_branch(cont_bb);

        self.builder().position_at_end(cont_bb);
        let phi = self
            .builder()
            .build_phi(ptr_to_object_type(self.context), "phi");
        phi.add_incoming(&[(&then_code, then_bb), (&else_code, else_bb)]);
        phi.as_basic_value().into_pointer_value()
    }
}

pub fn ptr_type<'c>(ty: StructType<'c>) -> PointerType<'c> {
    ty.ptr_type(AddressSpace::Generic)
}

pub static SELF_NAME: &str = "%SELF%";
