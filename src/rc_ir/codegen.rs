//! Code generation from the RC IR to LLVM.
//!
//! The parallel LLVM back end that consumes the RC IR (with its explicit `Retain`/`Release` nodes)
//! instead of the AST. Reference counting is driven entirely by
//! the RC nodes; variable reads are plain (no `used_later` retain decision). The generator runs with
//! `rc_ir_mode = true`, which makes `get_scoped_obj` read plain and the borrow getters skip their
//! conditional container release. Non-reference-counting work (closure layout, FFI, struct/array
//! construction, the inline-LLVM builtins) reuses the existing `Generator` helpers unchanged.

use crate::ast::name::FullName;
use crate::ast::types::TypeNode;
use crate::constants::{
    pthread_once_init_flag_type, pthread_once_init_flag_value, CLOSURE_CAPTURE_IDX,
    CLOSURE_FUNPTR_IDX, DYNAMIC_OBJ_CAP_IDX,
};
use crate::fixstd::builtin::make_dynamic_object_ty;
use crate::fixstd::runtime::RUNTIME_PTHREAD_ONCE;
use crate::generator::{Generator, Object};
use crate::misc::Map;
use crate::object::{create_obj, lambda_function_type, ObjectFieldType};
use crate::rc_ir::ast::{
    FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs, RcVar,
};
use inkwell::basic_block::BasicBlock;
use inkwell::debug_info::AsDIScope;
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue, IntValue};
use inkwell::{AddressSpace, IntPredicate};
use std::sync::Arc;

impl<'c, 'm> Generator<'c, 'm> {
    /// Generate LLVM code for the functions and global initializers of `prog` — one compilation
    /// unit's worth. Every top-level symbol has already been declared (by `declare_symbol`, run for
    /// all symbols in every unit), so those declarations are reused; only this unit's lifted lambdas
    /// are declared here, and only `prog`'s functions and globals are implemented.
    pub fn implement_rc_program(&mut self, prog: &RcProgram) {
        let saved_mode = self.rc_ir_mode;
        self.rc_ir_mode = true;

        let mut fn_map: Map<FuncRef, FunctionValue<'c>> = Map::default();
        for (fref, func) in prog.funcs.iter() {
            let fn_val = self
                .module
                .get_function(&func.name.name.to_string())
                .unwrap_or_else(|| self.declare_rc_function(func));
            fn_map.insert(fref.clone(), fn_val);
        }

        for (fref, func) in prog.funcs.iter() {
            self.implement_rc_function(func, fn_map[fref], &fn_map);
        }

        for glob in prog.globals.iter() {
            self.implement_rc_global(glob, &fn_map);
        }

        self.rc_ir_mode = saved_mode;
    }

    /// Declare the LLVM function for an `RcFunc` (signature from its arrow type). Funptr functions
    /// get external linkage under separated compilation; closure functions are always internal.
    fn declare_rc_function(&mut self, func: &RcFunc) -> FunctionValue<'c> {
        let fn_ty = lambda_function_type(&func.fn_ty, self);
        let name = func.name.name.to_string();
        let linkage = if func.fn_ty.is_funptr() && self.config.enable_separated_compilation() {
            Linkage::External
        } else {
            Linkage::Internal
        };
        let fn_val = self.module.add_function(&name, fn_ty, Some(linkage));
        if self.has_di() {
            let fn_name = fn_val.get_name().to_str().unwrap().to_string();
            fn_val.set_subprogram(self.create_debug_subprogram(&fn_name, func.source.clone()));
        }
        fn_val
    }

    /// Implement an `RcFunc` body: bind the parameters (and the capture pointer, for a closure) onto
    /// the scope as plain objects, then evaluate the body in tail position. Capture read-back and the
    /// release of unused parameters/captures are already explicit in the body, so nothing extra is
    /// done here.
    fn implement_rc_function(
        &mut self,
        func: &RcFunc,
        fn_val: FunctionValue<'c>,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) {
        let _builder_guard = self.push_builder();
        let bb = self.context.append_basic_block(fn_val, "entry");
        self.builder().position_at_end(bb);

        let _di_scope_guard = if self.has_di() {
            let subprogram = fn_val.get_subprogram();
            Some(self.push_debug_scope(subprogram.map(|sub| sub.as_debug_info_scope())))
        } else {
            None
        };

        let _scope_guard = self.push_scope();

        for (i, param) in func.params.iter().enumerate() {
            let val = fn_val.get_nth_param(i as u32).unwrap();
            let obj = Object::new(val, param.ty.clone(), self);
            self.scope_push(&param.name, &obj);
        }
        if let Some(cap) = &func.cap {
            let val = fn_val.get_nth_param(func.params.len() as u32).unwrap();
            let obj = Object::new(val, cap.ty.clone(), self);
            self.scope_push(&cap.name, &obj);
        }

        self.eval_rc_expr(&func.body, true, fn_map);
    }

    /// Evaluate an RC IR expression. Returns the produced object when `tail` is false; when `tail` is
    /// true the return has been built and `None` is returned.
    fn eval_rc_expr(
        &mut self,
        node: &RcExprNode,
        tail: bool,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        self.push_debug_location(node.source.clone());
        // A deeply nested continuation recurses deeply here (as lowering and RC insertion do); grow
        // the stack on demand so a large program does not overflow it.
        let result = stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.eval_rc_expr_body(node, tail, fn_map)
        });
        self.pop_debug_location();
        result
    }

    fn eval_rc_expr_body(
        &mut self,
        node: &RcExprNode,
        tail: bool,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        match node.expr.as_ref() {
            RcExpr::Ret(x) => {
                let obj = self.get_scoped_obj(&x.name);
                self.build_tail(obj, tail)
            }
            RcExpr::Retain(x, _path, _state, k) => {
                let obj = self.get_scoped_obj_noretain(&x.name);
                if x.nonnull {
                    // A statically non-null boxed value (a non-empty capture object): retain
                    // without the null check that a possibly-null capture object needs.
                    //
                    // The `nonnull` bit is set only on capture objects, and a capture object
                    // flows linearly — projected (a borrow), released at its last use, moved when
                    // threaded onward — so it is seldom duplicated and hence seldom retained. This
                    // skip therefore almost never lands on a hot path, unlike the symmetric
                    // release-side skip, which fires wherever a non-empty capture is released. It
                    // is kept for that symmetry and for the rare code that does retain a capture.
                    self.retain_nonnull_boxed(&obj);
                } else {
                    self.build_retain(obj);
                }
                self.eval_rc_expr(k, tail, fn_map)
            }
            RcExpr::Release(x, _path, _state, k) => {
                let obj = self.get_scoped_obj_noretain(&x.name);
                if x.nonnull {
                    // A statically non-null boxed value (a non-empty capture object): release
                    // without the null check that a possibly-null capture object needs.
                    self.release_nonnull_boxed(&obj);
                } else {
                    self.release(obj);
                }
                self.eval_rc_expr(k, tail, fn_map)
            }
            RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
                let match_tail = self.tail_fuses(x, k, tail);
                let obj = self.eval_rc_match(x, scrut, arms, match_tail, fn_map);
                if match_tail {
                    // Each arm returned directly; the continuation is a pure rename to `Ret`.
                    None
                } else {
                    self.bind_and_continue(x, obj.unwrap(), k, tail, fn_map)
                }
            }
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let app_tail = self.tail_fuses(x, k, tail);
                let callee_obj = self.get_scoped_obj(&callee.name);
                let arg_objs: Vec<Object<'c>> =
                    args.iter().map(|a| self.get_scoped_obj(&a.name)).collect();
                let obj = self.apply_lambda(callee_obj, arg_objs, app_tail);
                if app_tail {
                    None
                } else {
                    self.bind_and_continue(x, obj.unwrap(), k, tail, fn_map)
                }
            }
            RcExpr::Let(x, RcRhs::Llvm(gen, _args), k) => {
                // An inline-LLVM op may itself be in tail position (e.g. `FixBody`) and may diverge
                // (e.g. the panic/undefined ops); in both cases `generate` returns `None`.
                let llvm_tail = self.tail_fuses(x, k, tail);
                match gen.generate(self, &x.ty, llvm_tail) {
                    None => None,
                    Some(obj) => self.bind_and_continue(x, obj, k, tail, fn_map),
                }
            }
            RcExpr::Let(x, rhs, k) => {
                let obj = self.eval_rc_rhs(rhs, &x.ty, fn_map);
                self.bind_and_continue(x, obj, k, tail, fn_map)
            }
            RcExpr::Destructure(container, fields, k) => {
                // Extract all fields at once (the container was retained beforehand by a `Retain`
                // node iff it is used afterward). `get_struct_fields` performs the whole-container
                // reference counting: a boxed container retains the fields and releases itself, an
                // unboxed container moves the fields out and releases the fields not named here.
                let cont_obj = self.get_scoped_obj_noretain(&container.name);
                let field_indices: Vec<u32> = fields.iter().map(|(idx, _)| *idx as u32).collect();
                let subobjs = ObjectFieldType::get_struct_fields(self, &cont_obj, &field_indices);
                for ((_, fv), obj) in fields.iter().zip(subobjs.iter()) {
                    self.scope_push(&fv.name, obj);
                    self.emit_rc_debug_local(fv, obj);
                }
                let res = self.eval_rc_expr(k, tail, fn_map);
                for (_, fv) in fields {
                    self.scope_pop(&fv.name);
                }
                res
            }
        }
    }

    /// Bind `obj` to `x` on the scope, emit its debug local variable, evaluate the continuation `k`,
    /// then pop the binding.
    fn bind_and_continue(
        &mut self,
        x: &RcVar,
        obj: Object<'c>,
        k: &RcExprNode,
        tail: bool,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        self.scope_push(&x.name, &obj);
        self.emit_rc_debug_local(x, &obj);
        let res = self.eval_rc_expr(k, tail, fn_map);
        self.scope_pop(&x.name);
        res
    }

    /// Whether the binding of `x` in tail position may fuse into the tail return, skipping a scoped
    /// binding. It fuses for the compiler-introduced ANF temporaries, which carry no source name; in
    /// a debug build it does NOT fuse a source-named binding, so a debugger can inspect that value —
    /// matching the current back end, which materializes every source `let` binding as a scoped
    /// value. Genuine tail calls and tail recursion go through unnamed temporaries, so they still
    /// fuse in every build.
    fn tail_fuses(&self, x: &RcVar, k: &RcExprNode, tail: bool) -> bool {
        tail && is_tail_cont(k, &x.name) && !(self.has_di() && x.debug_name.is_some())
    }

    /// Emit a debug local variable for the binding of `var` to `obj`, when debug info is enabled and
    /// `var` carries a source-level name. A debugger can then inspect the value under that name.
    fn emit_rc_debug_local(&mut self, var: &RcVar, obj: &Object<'c>) {
        if self.has_di() {
            if let Some(name) = &var.debug_name {
                self.create_debug_local_variable(name, obj);
            }
        }
    }

    /// Evaluate a `Var` or `Closure` right-hand side to an object. `App`, `Match`, and `Llvm` are
    /// handled directly in `eval_rc_expr_body`.
    fn eval_rc_rhs(
        &mut self,
        rhs: &RcRhs,
        result_ty: &Arc<TypeNode>,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Object<'c> {
        match rhs {
            RcRhs::Var(v) => self.get_scoped_obj(&v.name),
            RcRhs::Closure(func, caps) => self.build_rc_closure(func, caps, result_ty, fn_map),
            RcRhs::App(..) | RcRhs::Match(..) | RcRhs::Llvm(..) => {
                unreachable!("App, Match, and Llvm are handled in eval_rc_expr_body")
            }
        }
    }

    /// Build a closure value `{funptr, capture-object pointer}` for `Closure(func, caps)`.
    fn build_rc_closure(
        &mut self,
        func: &FuncRef,
        caps: &[RcVar],
        result_ty: &Arc<TypeNode>,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Object<'c> {
        let fn_val = fn_map[func];
        let mut lam = create_obj(result_ty.clone(), &vec![], None, self, Some("closure"));
        let fn_ptr = fn_val.as_global_value().as_pointer_value();
        lam = lam.insert_field(self, CLOSURE_FUNPTR_IDX, fn_ptr);

        let cap_ptr: BasicValueEnum<'c> = if caps.is_empty() {
            self.context
                .ptr_type(AddressSpace::from(0))
                .const_null()
                .as_basic_value_enum()
        } else {
            let cap_tys: Vec<Arc<TypeNode>> = caps.iter().map(|c| c.ty.clone()).collect();
            let dyn_ty = make_dynamic_object_ty();
            let cap_obj = create_obj(
                dyn_ty.clone(),
                &cap_tys,
                None,
                self,
                Some("captured_objects"),
            );
            let cap_obj_str_ty = dyn_ty
                .get_object_type(&cap_tys, self.type_env())
                .to_struct_type(self, vec![]);
            for (i, cap) in caps.iter().enumerate() {
                let val = self.get_scoped_obj(&cap.name).value;
                cap_obj.insert_field_as(self, cap_obj_str_ty, i as u32 + DYNAMIC_OBJ_CAP_IDX, val);
            }
            cap_obj.value
        };
        lam = lam.insert_field(self, CLOSURE_CAPTURE_IDX, cap_ptr);
        lam
    }

    /// Generate a `Match`. `tail` selects direct returns in each arm (no merge) versus a phi that
    /// yields the match value. The scrutinee's per-arm container release and dead-branch releases are
    /// already explicit `Release` nodes in the arm bodies; here only the payload retain-getter is
    /// baked in (mirroring `get_union_value`).
    fn eval_rc_match(
        &mut self,
        result: &RcVar,
        scrut: &RcVar,
        arms: &[MatchArm],
        tail: bool,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        let scrut_obj = self.get_scoped_obj_noretain(&scrut.name);
        let is_box = scrut_obj.ty.is_box(self.type_env());

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

        // A basic block per arm, and the (tag, block) cases for the switch (all arms but the last).
        let mut arm_bbs = vec![];
        for (i, _arm) in arms.iter().enumerate() {
            arm_bbs.push(
                self.context
                    .append_basic_block(current_func, &format!("case_{}", i)),
            );
        }
        let else_bb = *arm_bbs.last().unwrap();
        let mut cases: Vec<(IntValue<'c>, BasicBlock<'c>)> = vec![];
        for (i, arm) in arms.iter().enumerate().take(arms.len() - 1) {
            let tag = arm
                .variant
                .expect("a non-final match arm must be a variant arm");
            let tag_val = ObjectFieldType::UnionTag
                .to_basic_type(self, vec![])
                .into_int_type()
                .const_int(tag as u64, false);
            cases.push((tag_val, arm_bbs[i]));
        }
        if cases.is_empty() {
            self.builder().build_unconditional_branch(else_bb).unwrap();
        } else {
            let tag_val = ObjectFieldType::get_union_tag(self, &scrut_obj);
            self.builder()
                .build_switch(tag_val, else_bb, &cases)
                .unwrap();
        }

        // Implement each arm.
        let mut incomings: Vec<(BasicValueEnum<'c>, BasicBlock<'c>)> = vec![];
        for (i, arm) in arms.iter().enumerate() {
            self.builder().position_at_end(arm_bbs[i]);

            // Bind the arm payload: a variant arm extracts (and, for a boxed union, retains) the
            // variant value; a catch-all arm binds the whole scrutinee.
            let payload_obj = match arm.variant {
                Some(_) => {
                    let scrut_obj = self.get_scoped_obj_noretain(&scrut.name);
                    let value = ObjectFieldType::get_union_value_noretain_norelease(
                        self,
                        scrut_obj,
                        &arm.payload.ty,
                    );
                    if is_box {
                        self.build_retain(value.clone());
                    }
                    value
                }
                None => self.get_scoped_obj_noretain(&scrut.name),
            };
            self.scope_push(&arm.payload.name, &payload_obj);
            self.emit_rc_debug_local(&arm.payload, &payload_obj);

            let arm_val = self.eval_rc_expr(&arm.body, tail, fn_map);
            self.scope_pop(&arm.payload.name);

            // A non-tail arm that produced a value branches to the merge block and feeds the phi. An
            // arm that returned (tail) or diverged yields `None` and contributes nothing.
            if let Some(arm_val) = arm_val {
                let end_bb = self.builder().get_insert_block().unwrap();
                incomings.push((arm_val.value, end_bb));
                self.builder()
                    .build_unconditional_branch(cont_bb.unwrap())
                    .unwrap();
            }
        }

        if tail {
            return None;
        }
        let cont_bb = cont_bb.unwrap();
        self.builder().position_at_end(cont_bb);
        if incomings.len() == 1 {
            return Some(Object::new(incomings[0].0, result.ty.clone(), self));
        }
        let phi_ty = incomings[0].0.get_type();
        let phi = self.builder().build_phi(phi_ty, "match_phi").unwrap();
        for (val, bb) in &incomings {
            phi.add_incoming(&[(val, *bb)]);
        }
        Some(Object::new(phi.as_basic_value(), result.ty.clone(), self))
    }

    /// Implement a global initializer: a lazily-initialized accessor that computes the value once
    /// (call-once), marks it and its reachable graph global, and stores it. Mirrors
    /// `implement_symbol`'s global branch, with the initializer evaluated from the RC IR.
    fn implement_rc_global(
        &mut self,
        glob: &RcGlobalInit,
        fn_map: &Map<FuncRef, FunctionValue<'c>>,
    ) {
        let acc_fn = self
            .module
            .get_function(&format!("Get#{}", glob.symbol.to_string()))
            .unwrap();
        if self.has_di() {
            let fn_name = acc_fn.get_name().to_str().unwrap().to_string();
            acc_fn.set_subprogram(self.create_debug_subprogram(&fn_name, glob.init.source.clone()));
        }

        let obj_embed_ty = glob.ty.get_embedded_type(self, &vec![]);

        // The storage for the initialized value, and the call-once flag.
        let global_var = self.module.add_global(
            obj_embed_ty,
            None,
            &format!("GlobalVar#{}", glob.symbol.to_string()),
        );
        global_var.set_initializer(&obj_embed_ty.const_zero());
        global_var.set_linkage(Linkage::Internal);
        let global_var_ptr = global_var.as_basic_value_enum().into_pointer_value();

        let (flag_ty, flag_init_val) = if self.config.threaded {
            (
                pthread_once_init_flag_type(self.context),
                pthread_once_init_flag_value(self.context),
            )
        } else {
            let ty = self.context.i8_type();
            (ty, ty.const_zero())
        };
        let init_flag = self.module.add_global(
            flag_ty,
            None,
            &format!("InitFlag#{}", glob.symbol.to_string()),
        );
        init_flag.set_initializer(&flag_init_val);
        init_flag.set_linkage(Linkage::Internal);
        let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

        let _builder_guard = self.push_builder();
        let entry_bb = self.context.append_basic_block(acc_fn, "entry");
        self.builder().position_at_end(entry_bb);
        let _di_scope_guard = if self.has_di() {
            let subprogram = acc_fn.get_subprogram();
            Some(self.push_debug_scope(subprogram.map(|sp| sp.as_debug_info_scope())))
        } else {
            None
        };

        // Branch to the initialization code only on the first access.
        let (init_bb, end_bb, mut init_fn_di_guard) = if !self.config.threaded {
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
            let init_fn_name = format!("InitOnce#{}", glob.symbol.to_string());
            let init_fn = self.module.add_function(
                &init_fn_name,
                self.context.void_type().fn_type(&[], false),
                Some(Linkage::Internal),
            );
            if self.has_di() {
                init_fn.set_subprogram(
                    self.create_debug_subprogram(&init_fn_name, glob.init.source.clone()),
                );
            }
            self.call_runtime(
                RUNTIME_PTHREAD_ONCE,
                &[
                    init_flag.into(),
                    init_fn.as_global_value().as_pointer_value().into(),
                ],
            );
            let end_bb = self.context.append_basic_block(acc_fn, "end_bb");
            self.builder().build_unconditional_branch(end_bb).unwrap();
            let init_bb = self.context.append_basic_block(init_fn, "init_bb");
            let guard = if self.has_di() {
                let subprogram = init_fn.get_subprogram();
                Some(self.push_debug_scope(subprogram.map(|sp| sp.as_debug_info_scope())))
            } else {
                None
            };
            (init_bb, end_bb, guard)
        };

        // Evaluate the initializer, mark it global, and store it.
        {
            self.builder().position_at_end(init_bb);
            let _scope_guard = self.push_scope();
            let obj = self.eval_rc_expr(&glob.init, false, fn_map).unwrap();
            self.mark_global(obj.clone());
            self.builder()
                .build_store(global_var_ptr, obj.value)
                .unwrap();
        }

        if !self.config.threaded {
            self.builder()
                .build_store(init_flag, self.context.i8_type().const_int(1, false))
                .unwrap();
            self.builder().build_unconditional_branch(end_bb).unwrap();
        } else {
            self.builder().build_return(None).unwrap();
            init_fn_di_guard.take();
            self.set_debug_location(None);
        }

        // Return the stored value.
        self.builder().position_at_end(end_bb);
        let value = self
            .builder()
            .build_load(obj_embed_ty, global_var_ptr, "load_global_var")
            .unwrap();
        if self.sizeof(&value.get_type()) == 0 {
            self.builder().build_return(None).unwrap();
        } else {
            self.builder().build_return(Some(&value)).unwrap();
        }
    }
}

/// Whether the continuation `k` carries `x` to the terminator only by move-renames — i.e. the
/// binding of `x` is in tail position.
fn is_tail_cont(k: &RcExprNode, x: &FullName) -> bool {
    match k.expr.as_ref() {
        RcExpr::Ret(r) => r.name == *x,
        RcExpr::Let(y, RcRhs::Var(x2), k2) => x2.name == *x && is_tail_cont(k2, &y.name),
        _ => false,
    }
}
