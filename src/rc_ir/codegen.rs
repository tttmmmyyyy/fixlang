//! Code generation from the RC IR to LLVM.
//!
//! The LLVM back end consumes the RC IR (with its explicit `Retain`/`Release` nodes). Reference
//! counting is driven entirely by the RC nodes: variable reads are plain and the read-getters do not
//! release their container — the explicit `Release` nodes dispose it. Non-reference-counting work
//! (closure layout, FFI, struct/array construction, the inline-LLVM builtins) reuses the existing
//! `Generator` helpers unchanged.

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
    FuncRef, MatchArm, RcExpr, RcExprNode, RcFunc, RcGlobalInit, RcProgram, RcRhs, RcState, RcVar,
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
    /// all symbols in every unit), so those declarations are reused; only the functions synthesized
    /// after that — lifted lambdas, and at higher optimization levels the borrow and specialization
    /// versions — are declared here, and only `prog`'s functions and globals are implemented.
    pub fn implement_rc_program(&mut self, prog: &RcProgram) {
        let mut func_vals: Map<FuncRef, FunctionValue<'c>> = Map::default();
        for (fref, func) in prog.funcs.iter() {
            let fn_val = self
                .module
                .get_function(&func.name.name.to_string())
                .unwrap_or_else(|| self.declare_rc_function(func));
            // A function is implemented once. A name minted here that collides with one already
            // implemented would take a second body, appended after the first `entry` block and never
            // reached, dropping one of the two.
            assert_eq!(
                fn_val.count_basic_blocks(),
                0,
                "function `{}` is implemented twice",
                func.name.name.to_string()
            );
            func_vals.insert(fref.clone(), fn_val);
        }

        for (fref, func) in prog.funcs.iter() {
            self.implement_rc_function(func, func_vals[fref], &func_vals);
        }

        for global_init in prog.globals.iter() {
            self.implement_rc_global(global_init, &func_vals);
        }
    }

    /// Declare the LLVM function for an `RcFunc` (signature from its arrow type) and, for a funptr
    /// function, register its global accessor so a direct call by name resolves it. This is the
    /// funptr analogue of `declare_symbol` for the functions born after declaration — lifted lambdas,
    /// and the borrow and specialization versions synthesized while optimizing — which `declare_symbol`
    /// never saw. Funptr functions get external linkage under separated compilation; closure functions
    /// are always internal.
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
        if func.fn_ty.is_funptr() {
            self.add_global_object(func.name.name.clone(), fn_val, func.fn_ty.clone());
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
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) {
        let _builder_guard = self.push_builder();
        let bb = self.context.append_basic_block(fn_val, "entry");
        self.builder().position_at_end(bb);

        let _di_scope_guard = if self.has_di() {
            let subprogram = fn_val
                .get_subprogram()
                .expect("a function implemented with debug info has a subprogram");
            Some(self.push_debug_scope(Some(subprogram.as_debug_info_scope())))
        } else {
            None
        };

        let _scope_guard = self.push_scope();

        // Each parameter arrives as its flat leaf scalars (see `lambda_function_type`); reassemble
        // the (possibly aggregate) parameter value from them. The CAP pointer follows all of them.
        let mut next_param = 0u32;
        for param in func.params.iter() {
            let embedded = param.ty.get_embedded_type(self, &vec![]);
            let leaf_count = self.flatten_to_scalar_leaves(embedded).len() as u32;
            let leaf_vals: Vec<_> = (0..leaf_count)
                .map(|k| fn_val.get_nth_param(next_param + k).unwrap())
                .collect();
            next_param += leaf_count;
            let mut leaves = leaf_vals.into_iter();
            let val = self.assemble_from_scalar_leaves(embedded, &mut leaves);
            let obj = Object::new(val, param.ty.clone(), self);
            self.scope_push(&param.name, &obj);
        }
        if let Some(cap) = &func.capture {
            let val = fn_val.get_nth_param(next_param).unwrap();
            let obj = Object::new(val, cap.ty.clone(), self);
            self.scope_push(&cap.name, &obj);
        }

        self.eval_rc_expr(&func.body, true, func_vals);
    }

    /// Evaluate an RC IR expression. Returns the produced object when `tail` is false; when `tail` is
    /// true the return has been built and `None` is returned.
    fn eval_rc_expr(
        &mut self,
        node: &RcExprNode,
        tail: bool,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        self.push_debug_location(node.source.clone());
        // A deeply nested continuation recurses deeply here (as lowering and RC insertion do); grow
        // the stack on demand so a large program does not overflow it.
        let result = stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.eval_rc_expr_inner(node, tail, func_vals)
        });
        self.pop_debug_location();
        result
    }

    fn eval_rc_expr_inner(
        &mut self,
        node: &RcExprNode,
        tail: bool,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        match node.expr.as_ref() {
            RcExpr::Ret(x) => {
                let obj = self.get_scoped_obj(&x.name);
                self.build_tail(obj, tail)
            }
            RcExpr::Retain(x, path, state, k) => {
                // Only the runtime three-way dispatch is implemented; whoever adds the state
                // inference must implement the states it produces here.
                assert_eq!(
                    *state,
                    RcState::Unknown,
                    "reference-count state dispatch is not implemented"
                );
                let obj = self.get_scoped_obj_noretain(&x.name);
                let obj = self.project_rc_unit(obj, path);
                if x.skip_null_check {
                    // A statically non-null boxed value (a non-empty capture object): retain
                    // without the null check that a possibly-null capture object needs.
                    //
                    // The bit describes the whole variable, so it says nothing about a sub-object.
                    assert!(
                        path.is_empty(),
                        "`skip_null_check` describes the whole variable, not a projection of it"
                    );
                    //
                    // The `skip_null_check` bit is set only on capture objects, and a capture object
                    // flows linearly — projected (a borrow), released at its last use, moved when
                    // threaded onward — so it is seldom duplicated and hence seldom retained. This
                    // skip therefore almost never lands on a hot path, unlike the symmetric
                    // release-side skip, which fires wherever a non-empty capture is released. It
                    // is kept for that symmetry and for the rare code that does retain a capture.
                    let one = self.context.i64_type().const_int(1, false);
                    self.retain_nonnull_boxed(&obj, one);
                } else {
                    let one = self.context.i64_type().const_int(1, false);
                    self.build_retain(obj, one);
                }
                self.eval_rc_expr(k, tail, func_vals)
            }
            RcExpr::Release(x, path, state, k) => {
                // Only the runtime three-way dispatch is implemented; whoever adds the state
                // inference must implement the states it produces here.
                assert_eq!(
                    *state,
                    RcState::Unknown,
                    "reference-count state dispatch is not implemented"
                );
                let obj = self.get_scoped_obj_noretain(&x.name);
                let obj = self.project_rc_unit(obj, path);
                if x.skip_null_check {
                    // A statically non-null boxed value (a non-empty capture object): release
                    // without the null check that a possibly-null capture object needs.
                    //
                    // The bit describes the whole variable, so it says nothing about a sub-object.
                    assert!(
                        path.is_empty(),
                        "`skip_null_check` describes the whole variable, not a projection of it"
                    );
                    self.release_nonnull_boxed(&obj);
                } else {
                    self.release(obj);
                }
                self.eval_rc_expr(k, tail, func_vals)
            }
            RcExpr::Eval(x, k) => {
                // Observe `x` to force its evaluation for effect, then discard it: reading a global
                // runs its call-once initializer; a local is already computed. No reference-count
                // operation and no value are produced here — a preceding `Retain` or following
                // `Release` carries any reference counting.
                let _ = self.get_scoped_obj_noretain(&x.name);
                self.eval_rc_expr(k, tail, func_vals)
            }
            RcExpr::Let(x, RcRhs::Match(scrut, arms), k) => {
                let match_tail = self.binding_fuses_into_return(x, k, tail);
                let obj = self.eval_rc_match(x, scrut, arms, match_tail, func_vals);
                if match_tail {
                    // Each arm returned directly; the continuation is a pure rename to `Ret`.
                    None
                } else {
                    self.bind_and_continue(x, obj.unwrap(), k, tail, func_vals)
                }
            }
            RcExpr::Let(x, RcRhs::App(callee, args), k) => {
                let app_tail = self.binding_fuses_into_return(x, k, tail);
                let callee_obj = self.get_scoped_obj(&callee.name);
                let arg_objs: Vec<Object<'c>> =
                    args.iter().map(|a| self.get_scoped_obj(&a.name)).collect();
                let obj = self.apply_lambda(callee_obj, arg_objs, app_tail);
                if app_tail {
                    None
                } else {
                    self.bind_and_continue(x, obj.unwrap(), k, tail, func_vals)
                }
            }
            RcExpr::Let(x, RcRhs::Llvm(llvm_gen, _args), k) => {
                // An inline-LLVM op may build the tail return itself (`FixBody`), in which case it
                // yields no value. A diverging op (`undefined`) does not: it emits `unreachable` and
                // yields an undef value, so the continuation is generated as dead code.
                let llvm_tail = self.binding_fuses_into_return(x, k, tail);
                match llvm_gen.generate_tail(self, &x.ty, llvm_tail) {
                    None => {
                        // Yielding no value says the op built the return, which it may only do in
                        // tail position; elsewhere the continuation below would be dropped and the
                        // block left without a terminator.
                        assert!(
                            llvm_tail,
                            "inline-LLVM op `{}` yielded no value outside tail position",
                            llvm_gen.name()
                        );
                        None
                    }
                    Some(obj) => self.bind_and_continue(x, obj, k, tail, func_vals),
                }
            }
            RcExpr::Let(x, rhs, k) => {
                let obj = self.eval_rc_rhs(rhs, &x.ty, func_vals);
                self.bind_and_continue(x, obj, k, tail, func_vals)
            }
            RcExpr::Destructure(container, fields, k) => {
                // Extract all fields at once (the container was retained beforehand by a `Retain`
                // node iff it is used afterward). `get_struct_fields` performs the whole-container
                // reference counting: a boxed container retains the fields and releases itself, an
                // unboxed container moves the fields out and releases the fields not named here.
                let cont_obj = self.get_scoped_obj_noretain(&container.name);
                let field_indices: Vec<u32> = fields.iter().map(|(idx, _)| *idx as u32).collect();
                let field_objs =
                    ObjectFieldType::get_struct_fields(self, &cont_obj, &field_indices);
                // One object per requested index; the pop below walks `fields`, so a shorter list
                // would leave it popping names this loop never pushed.
                assert_eq!(
                    fields.len(),
                    field_objs.len(),
                    "a destructure extracts one object per field"
                );
                for ((_, fv), obj) in fields.iter().zip(field_objs.iter()) {
                    self.scope_push(&fv.name, obj);
                    self.emit_debug_local_variable(fv, obj);
                }
                let res = self.eval_rc_expr(k, tail, func_vals);
                for (_, fv) in fields {
                    self.scope_pop(&fv.name);
                }
                res
            }
        }
    }

    /// Project the whole object `obj` down `path` to the sub-object naming one reference-counting
    /// unit. Each index descends one unboxed struct/tuple field — or a closure's capture field — so
    /// the path stops at the unit itself: a boxed leaf, an unboxed union, or a closure capture. The
    /// empty path names the whole value, returning `obj` unchanged. The caller retains or releases
    /// the returned sub-object as a whole, which reference-counts exactly that unit (a boxed leaf
    /// directly, a union by tag dispatch).
    fn project_rc_unit(&mut self, obj: Object<'c>, path: &[usize]) -> Object<'c> {
        let mut cur = obj;
        for &idx in path {
            let field_ty = if cur.ty.is_closure() {
                // The only unit path into a closure names its capture object, its second field. The
                // other field is the function pointer, which a reference-count operation must never
                // reach.
                assert_eq!(
                    idx as u32, CLOSURE_CAPTURE_IDX,
                    "a reference-counting unit path into a closure names its capture"
                );
                make_dynamic_object_ty()
            } else {
                // Descending is what a path into an aggregate does; a unit root is where it stops,
                // so a path going on past one would reference-count a part of that unit rather than
                // the unit. (A closure is not a unit root: its unit is the capture, reached above.)
                assert!(
                    !cur.ty.is_rc_unit_root(self.type_env()),
                    "a reference-counting unit path descends past the unit it names"
                );
                cur.ty.field_types(self.type_env())[idx].clone()
            };
            let val = cur.extract_field(self, idx as u32);
            cur = Object::new(val, field_ty, self);
        }
        cur
    }

    /// Bind `obj` to `x` on the scope, emit its debug local variable, evaluate the continuation `k`,
    /// then pop the binding.
    fn bind_and_continue(
        &mut self,
        x: &RcVar,
        obj: Object<'c>,
        k: &RcExprNode,
        tail: bool,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        self.scope_push(&x.name, &obj);
        self.emit_debug_local_variable(x, &obj);
        let res = self.eval_rc_expr(k, tail, func_vals);
        self.scope_pop(&x.name);
        res
    }

    /// Whether the binding of `x` in tail position may fuse into the tail return, skipping a scoped
    /// binding. It fuses for the compiler-introduced ANF temporaries, which carry no source name; in
    /// a debug build it does NOT fuse a source-named binding, so a debugger can inspect that value —
    /// matching the current back end, which materializes every source `let` binding as a scoped
    /// value. Genuine tail calls and tail recursion go through unnamed temporaries, so they still
    /// fuse in every build.
    fn binding_fuses_into_return(&self, x: &RcVar, k: &RcExprNode, tail: bool) -> bool {
        tail && carries_var_to_return(k, &x.name) && !(self.has_di() && x.debug_name.is_some())
    }

    /// Emit a debug local variable for the binding of `var` to `obj`, when debug info is enabled and
    /// `var` carries a source-level name. A debugger can then inspect the value under that name.
    fn emit_debug_local_variable(&mut self, var: &RcVar, obj: &Object<'c>) {
        if self.has_di() {
            if let Some(name) = &var.debug_name {
                self.create_debug_local_variable(name, obj);
            }
        }
    }

    /// Evaluate a `Var` or `Closure` right-hand side to an object. `App`, `Match`, and `Llvm` are
    /// handled directly in `eval_rc_expr_inner`.
    fn eval_rc_rhs(
        &mut self,
        rhs: &RcRhs,
        result_ty: &Arc<TypeNode>,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Object<'c> {
        match rhs {
            RcRhs::Var(v) => self.get_scoped_obj(&v.name),
            RcRhs::Closure(func, captures) => {
                self.build_rc_closure(func, captures, result_ty, func_vals)
            }
            RcRhs::App(..) | RcRhs::Match(..) | RcRhs::Llvm(..) => {
                unreachable!("App, Match, and Llvm are handled in eval_rc_expr_inner")
            }
        }
    }

    /// Build a closure value `{funptr, capture-object pointer}` for `Closure(func, captures)`.
    fn build_rc_closure(
        &mut self,
        func: &FuncRef,
        captures: &[RcVar],
        result_ty: &Arc<TypeNode>,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Object<'c> {
        let fn_val = func_vals[func];
        let mut closure = create_obj(result_ty.clone(), &vec![], None, self, Some("closure"));
        let fn_ptr = fn_val.as_global_value().as_pointer_value();
        closure = closure.insert_field(self, CLOSURE_FUNPTR_IDX, fn_ptr);

        let capture_ptr: BasicValueEnum<'c> = if captures.is_empty() {
            self.context
                .ptr_type(AddressSpace::from(0))
                .const_null()
                .as_basic_value_enum()
        } else {
            let capture_tys: Vec<Arc<TypeNode>> = captures.iter().map(|c| c.ty.clone()).collect();
            let dyn_ty = make_dynamic_object_ty();
            let capture_obj = create_obj(
                dyn_ty.clone(),
                &capture_tys,
                None,
                self,
                Some("captured_objects"),
            );
            let capture_struct_ty = dyn_ty
                .get_object_type(&capture_tys, self.type_env())
                .to_struct_type(self, vec![]);
            for (i, cap) in captures.iter().enumerate() {
                let val = self.get_scoped_obj(&cap.name).value;
                capture_obj.insert_field_as(
                    self,
                    capture_struct_ty,
                    i as u32 + DYNAMIC_OBJ_CAP_IDX,
                    val,
                );
            }
            capture_obj.value
        };
        closure = closure.insert_field(self, CLOSURE_CAPTURE_IDX, capture_ptr);
        closure
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
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) -> Option<Object<'c>> {
        let scrut_obj = self.get_scoped_obj_noretain(&scrut.name);
        let scrut_is_boxed = scrut_obj.ty.is_box(self.type_env());

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
        let else_bb = *arm_bbs.last().expect("a match has at least one arm");
        let mut cases: Vec<(IntValue<'c>, BasicBlock<'c>)> = vec![];
        for (i, arm) in arms.iter().enumerate().take(arms.len() - 1) {
            let tag = arm
                .tag
                .expect("a non-final match arm must be a variant arm");
            let tag_val = ObjectFieldType::UnionTag
                .to_basic_type(self, vec![])
                .into_int_type()
                .const_int(tag as u64, false);
            cases.push((tag_val, arm_bbs[i]));
        }
        if cases.is_empty() {
            // The only arm takes every value of the scrutinee: it is either a catch-all, or the one
            // variant of its union. A variant arm standing alone over a multi-variant union would
            // bind the payload of whichever variant is actually there.
            assert!(
                arms[0].tag.is_none() || scrut_obj.ty.field_types(self.type_env()).len() == 1,
                "a match with one variant arm must be over a single-variant union"
            );
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
            let payload_obj = match arm.tag {
                Some(_) => {
                    let scrut_obj = self.get_scoped_obj_noretain(&scrut.name);
                    let value = ObjectFieldType::get_union_value_noretain_norelease(
                        self,
                        scrut_obj,
                        &arm.payload.ty,
                    );
                    if scrut_is_boxed {
                        let one = self.context.i64_type().const_int(1, false);
                        self.build_retain(value.clone(), one);
                    }
                    value
                }
                None => self.get_scoped_obj_noretain(&scrut.name),
            };
            self.scope_push(&arm.payload.name, &payload_obj);
            self.emit_debug_local_variable(&arm.payload, &payload_obj);

            let arm_val = self.eval_rc_expr(&arm.body, tail, func_vals);
            self.scope_pop(&arm.payload.name);

            // A non-tail arm that produced a value branches to the merge block and feeds the phi. An
            // arm that returned (tail) yields `None` and contributes nothing; a diverging arm feeds
            // the phi an undef value from its unreachable block.
            if let Some(arm_val) = arm_val {
                let arm_end_bb = self.builder().get_insert_block().unwrap();
                incomings.push((arm_val.value, arm_end_bb));
                self.builder()
                    .build_unconditional_branch(
                        cont_bb.expect("a non-tail match has a merge block"),
                    )
                    .unwrap();
            }
        }

        if tail {
            return None;
        }
        let cont_bb = cont_bb.expect("a non-tail match has a merge block");
        self.builder().position_at_end(cont_bb);
        // Every arm of a non-tail match yields a value, a diverging one included.
        assert!(
            !incomings.is_empty(),
            "a non-tail match has no arm that reaches its merge block"
        );
        if incomings.len() == 1 {
            return Some(Object::new(incomings[0].0, result.ty.clone(), self));
        }
        let phi = self.build_scalar_phi(&incomings, "match_phi");
        Some(Object::new(phi, result.ty.clone(), self))
    }

    /// Implement a global initializer: a lazily-initialized accessor that computes the value once
    /// (call-once), marks it and its reachable graph global, and stores it. Mirrors
    /// `implement_symbol`'s global branch, with the initializer evaluated from the RC IR.
    fn implement_rc_global(
        &mut self,
        global_init: &RcGlobalInit,
        func_vals: &Map<FuncRef, FunctionValue<'c>>,
    ) {
        let acc_fn = self
            .module
            .get_function(&format!("Get#{}", global_init.symbol.to_string()))
            .expect("a global has an accessor, declared with its symbol");
        if self.has_di() {
            let fn_name = acc_fn.get_name().to_str().unwrap().to_string();
            acc_fn.set_subprogram(
                self.create_debug_subprogram(&fn_name, global_init.init.source.clone()),
            );
        }

        let obj_embed_ty = global_init.ty.get_embedded_type(self, &vec![]);

        // The storage for the initialized value, and the call-once flag.
        let global_var = self.module.add_global(
            obj_embed_ty,
            None,
            &format!("GlobalVar#{}", global_init.symbol.to_string()),
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
            &format!("InitFlag#{}", global_init.symbol.to_string()),
        );
        init_flag.set_initializer(&flag_init_val);
        init_flag.set_linkage(Linkage::Internal);
        let init_flag = init_flag.as_basic_value_enum().into_pointer_value();

        let _builder_guard = self.push_builder();
        let entry_bb = self.context.append_basic_block(acc_fn, "entry");
        self.builder().position_at_end(entry_bb);
        let _di_scope_guard = if self.has_di() {
            let subprogram = acc_fn
                .get_subprogram()
                .expect("a function implemented with debug info has a subprogram");
            Some(self.push_debug_scope(Some(subprogram.as_debug_info_scope())))
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
            let init_fn_name = format!("InitOnce#{}", global_init.symbol.to_string());
            let init_fn = self.module.add_function(
                &init_fn_name,
                self.context.void_type().fn_type(&[], false),
                Some(Linkage::Internal),
            );
            if self.has_di() {
                init_fn.set_subprogram(
                    self.create_debug_subprogram(&init_fn_name, global_init.init.source.clone()),
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
                let subprogram = init_fn
                    .get_subprogram()
                    .expect("a function implemented with debug info has a subprogram");
                Some(self.push_debug_scope(Some(subprogram.as_debug_info_scope())))
            } else {
                None
            };
            (init_bb, end_bb, guard)
        };

        // Evaluate the initializer, mark it global, and store it.
        {
            self.builder().position_at_end(init_bb);
            let _scope_guard = self.push_scope();
            let obj = self
                .eval_rc_expr(&global_init.init, false, func_vals)
                .expect("an expression evaluated outside tail position yields a value");
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
fn carries_var_to_return(k: &RcExprNode, x: &FullName) -> bool {
    match k.expr.as_ref() {
        RcExpr::Ret(r) => r.name == *x,
        RcExpr::Let(y, RcRhs::Var(x2), k2) => x2.name == *x && carries_var_to_return(k2, &y.name),
        _ => false,
    }
}
