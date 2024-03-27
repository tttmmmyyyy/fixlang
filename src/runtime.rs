use inkwell::attributes::AttributeLoc;

use super::*;

pub const RUNTIME_ABORT: &str = "abort";
pub const RUNTIME_EPRINT: &str = "fixruntime_eprint";
pub const RUNTIME_SPRINTF: &str = "sprintf";
pub const RUNTIME_REPORT_MALLOC: &str = "report_malloc";
pub const RUNTIME_REPORT_RETAIN: &str = "report_retain";
pub const RUNTIME_REPORT_RELEASE: &str = "report_release";
pub const RUNTIME_REPORT_MARK_GLOBAL: &str = "report_mark_global";
pub const RUNTIME_CHECK_LEAK: &str = "check_leak";
pub const RUNTIME_RETAIN_BOXED_OBJECT: &str = "fixruntime_retain_obj";
pub const RUNTIME_RELEASE_BOXED_OBJECT: &str = "fixruntime_release_obj";
pub const RUNTIME_MARK_GLOBAL_BOXED_OBJECT: &str = "fixruntime_mark_global_obj";
pub const RUNTIME_MARK_THREADED_BOXED_OBJECT: &str = "fixruntime_mark_threaded_obj";
pub const RUNTIME_SUBTRACT_PTR: &str = "fixruntime_subtract_ptr";
pub const RUNTIME_PTR_ADD_OFFSET: &str = "fixruntime_ptr_add_offset";
pub const RUNTIME_PTHREAD_ONCE: &str = "pthread_once";
pub const RUNTIME_THREAD_PREPARE_TERMINATION: &str = "fixruntime_thread_prepare_termination";
pub const RUNTIME_THREAD_TERMINATE: &str = "fixruntime_thread_terminate";
pub const RUNTIME_RUN_FUNCTION: &str = "fixruntime_run_function_llvm";
pub const RUNTIME_GET_ARGC: &str = "fixruntime_get_argc";
pub const RUNTIME_GET_ARGV: &str = "fixruntime_get_argv";

pub fn build_runtime<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    build_abort_function(gc, mode);
    build_eprintf_function(gc, mode);
    build_sprintf_function(gc, mode);
    if gc.config.sanitize_memory {
        build_report_malloc_function(gc, mode);
        build_report_retain_function(gc, mode);
        build_report_release_function(gc, mode);
        build_check_leak_function(gc, mode);
        build_report_mark_global_function(gc, mode);
    }
    build_retain_boxed_function(gc, mode);
    build_release_boxed_function(gc, mode);
    build_mark_global_boxed_object_function(gc, mode);
    build_subtract_ptr_function(gc, mode);
    build_ptr_add_offset_function(gc, mode);
    if gc.config.threaded {
        build_pthread_once_function(gc, mode);
        build_mark_threaded_boxed_object_function(gc, mode);
    }
    if gc.config.async_task && gc.config.sanitize_memory {
        build_thread_prepare_termination_function(gc, mode);
        build_thread_terminate_function(gc, mode);
    }
    build_run_function(gc, mode); // This should be built after `build_mark_threaded_boxed_object_function`.
    build_get_argc_function(gc, mode);
    build_get_argv_function(gc, mode);
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BuildMode {
    Declare,
    Implement,
}

fn build_abort_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_ABORT) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function(RUNTIME_ABORT, fn_ty, None);
    return;
}

fn build_eprintf_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_EPRINT) {
        return;
    }

    let context = gc.context;
    let module = gc.module;

    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::from(0));

    let fn_type = context.void_type().fn_type(&[i8_ptr_type.into()], true);
    module.add_function(RUNTIME_EPRINT, fn_type, None);

    return;
}

fn build_sprintf_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_SPRINTF) {
        return;
    }

    let context = gc.context;
    let module = gc.module;

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::from(0));

    let fn_type = i32_type.fn_type(
        &[
            i8_ptr_type.into(), /* output buffer */
            i8_ptr_type.into(), /* format */
        ],
        true,
    );
    module.add_function(RUNTIME_SPRINTF, fn_type, None);

    return;
}

fn build_report_malloc_function<'c, 'm>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_REPORT_MALLOC) {
        return;
    }

    let fn_ty = gc.context.i64_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            gc.context.i8_type().ptr_type(AddressSpace::from(0)).into(),
        ],
        false,
    );
    gc.module.add_function(RUNTIME_REPORT_MALLOC, fn_ty, None);
    return;
}

fn build_report_retain_function<'c, 'm>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_REPORT_RETAIN) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            obj_id_type(gc.context).into(),
        ],
        false,
    );
    gc.module.add_function(RUNTIME_REPORT_RETAIN, fn_ty, None);
    return;
}

fn build_report_release_function<'c, 'm>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_REPORT_RELEASE) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            obj_id_type(gc.context).into(),
        ],
        false,
    );
    gc.module.add_function(RUNTIME_REPORT_RELEASE, fn_ty, None);
    return;
}

fn build_report_mark_global_function<'c, 'm>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_REPORT_MARK_GLOBAL) {
        return;
    }

    let fn_ty = gc
        .context
        .void_type()
        .fn_type(&[obj_id_type(gc.context).into()], false);
    gc.module
        .add_function(RUNTIME_REPORT_MARK_GLOBAL, fn_ty, None);
    return;
}

fn build_check_leak_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_CHECK_LEAK) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function(RUNTIME_CHECK_LEAK, fn_ty, None);
    return;
}

fn build_retain_boxed_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let context = gc.context;
    let module = gc.module;

    let retain_func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_RETAIN_BOXED_OBJECT) {
                return;
            }
            let void_type = context.void_type();
            let func_type = void_type.fn_type(&[ptr_to_object_type(context).into()], false);
            module.add_function(RUNTIME_RETAIN_BOXED_OBJECT, func_type, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_RETAIN_BOXED_OBJECT) {
            Some(func) => func,
            None => panic!(
                "Runtime function {} is not declared",
                RUNTIME_RETAIN_BOXED_OBJECT
            ),
        },
    };

    let bb = context.append_basic_block(retain_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to object.
    let obj_ptr = retain_func.get_first_param().unwrap().into_pointer_value();

    // Branch by refcnt_state.
    let (local_bb, threaded_bb, global_bb) = gc.build_branch_by_refcnt_state(obj_ptr);

    // A function to genearte code to report retain to sanitizer.
    fn report_retain_to_sanitizer<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        obj_ptr: PointerValue<'c>,
    ) {
        // Report retain to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(RUNTIME_REPORT_RETAIN, &[obj_ptr.into(), obj_id.into()]);
        }
    }

    // Implement `local_bb`.
    gc.builder().position_at_end(local_bb);
    // Increment refcnt and return.
    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
    let old_refcnt_local = gc
        .builder()
        .build_load(refcnt_type(gc.context), ptr_to_refcnt, "")
        .into_int_value();
    let new_refcnt = gc.builder().build_int_nsw_add(
        old_refcnt_local,
        refcnt_type(gc.context).const_int(1, false).into(),
        "",
    );
    gc.builder().build_store(ptr_to_refcnt, new_refcnt);
    report_retain_to_sanitizer(gc, obj_ptr);
    gc.builder().build_return(None);

    // Implement threaded_bb.
    if threaded_bb.is_some() {
        let threaded_bb = threaded_bb.unwrap();

        gc.builder().position_at_end(threaded_bb);
        // Increment refcnt atomically and jump to `end_bb`.
        let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
        let _old_refcnt_threaded = gc
            .builder()
            .build_atomicrmw(
                inkwell::AtomicRMWBinOp::Add,
                ptr_to_refcnt,
                refcnt_type(gc.context).const_int(1, false),
                inkwell::AtomicOrdering::Monotonic,
            )
            .unwrap();
        report_retain_to_sanitizer(gc, obj_ptr);
        gc.builder().build_return(None);
    }

    // Implement global_bb.
    gc.builder().position_at_end(global_bb);
    // In this case, nothing to do.
    gc.builder().build_return(None);

    return;
}

fn build_release_boxed_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let release_func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_RELEASE_BOXED_OBJECT) {
                return;
            }
            let void_type = gc.context.void_type();
            let func_type = void_type.fn_type(
                &[
                    ptr_to_object_type(gc.context).into(),
                    ObjectFieldType::TraverseFunction
                        .to_basic_type(gc, vec![])
                        .into(),
                ],
                false,
            );
            gc.module
                .add_function(RUNTIME_RELEASE_BOXED_OBJECT, func_type, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_RELEASE_BOXED_OBJECT) {
            Some(func) => func,
            None => panic!(
                "Runtime function {} is not declared",
                RUNTIME_RELEASE_BOXED_OBJECT
            ),
        },
    };

    let entry_bb = gc.context.append_basic_block(release_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(entry_bb);

    // Get pointer to the object.
    let obj_ptr = release_func.get_first_param().unwrap().into_pointer_value();

    // Branch by refcnt_state.
    let (local_bb, threaded_bb, global_bb) = gc.build_branch_by_refcnt_state(obj_ptr);
    let destruction_bb = gc
        .context
        .append_basic_block(release_func, "destruction_bb");
    let end_bb = gc.context.append_basic_block(release_func, "end_bb");

    // A function to genearte code to report retain to sanitizer.
    fn report_release_to_sanitizer<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        obj_ptr: PointerValue<'c>,
    ) {
        // Report release to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(RUNTIME_REPORT_RELEASE, &[obj_ptr.into(), obj_id.into()]);
        }
    }

    // Implement local_bb.
    gc.builder().position_at_end(local_bb);
    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
    // Decrement refcnt.
    let old_refcnt = gc
        .builder()
        .build_load(refcnt_type(gc.context), ptr_to_refcnt, "")
        .into_int_value();
    let new_refcnt = gc.builder().build_int_nsw_sub(
        old_refcnt,
        refcnt_type(gc.context).const_int(1, false).into(),
        "",
    );
    gc.builder().build_store(ptr_to_refcnt, new_refcnt);
    report_release_to_sanitizer(gc, obj_ptr);

    // Branch to `destruction_bb` if old_refcnt is one.
    let is_refcnt_one = gc.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        old_refcnt,
        refcnt_type(gc.context).const_int(1, false),
        "is_refcnt_zero",
    );
    gc.builder()
        .build_conditional_branch(is_refcnt_one, destruction_bb, end_bb);

    // Implement threaded_bb.
    if threaded_bb.is_some() {
        let threaded_bb = threaded_bb.unwrap();

        gc.builder().position_at_end(threaded_bb);
        let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
        // Decrement refcnt atomically.
        let old_refcnt = gc
            .builder()
            .build_atomicrmw(
                inkwell::AtomicRMWBinOp::Sub,
                ptr_to_refcnt,
                refcnt_type(gc.context).const_int(1, false),
                inkwell::AtomicOrdering::Release,
            )
            .unwrap();
        report_release_to_sanitizer(gc, obj_ptr);

        // Branch to `threaded_destruction_bb` if old_refcnt is one.
        let threaded_destruction_bb = gc
            .context
            .append_basic_block(release_func, "threaded_destruction_bb");
        let is_refcnt_one = gc.builder().build_int_compare(
            inkwell::IntPredicate::EQ,
            old_refcnt,
            refcnt_type(gc.context).const_int(1, false),
            "is_refcnt_one",
        );
        gc.builder()
            .build_conditional_branch(is_refcnt_one, threaded_destruction_bb, end_bb);

        // Implement `threaded_destruction_bb`.
        gc.builder().position_at_end(threaded_destruction_bb);
        gc.builder()
            .build_fence(inkwell::AtomicOrdering::Acquire, 0, "");
        gc.builder().build_unconditional_branch(destruction_bb);
    }

    // Implement `destruction_bb`
    gc.builder().position_at_end(destruction_bb);

    // Get dtor.
    let ptr_to_dtor = release_func.get_nth_param(1).unwrap().into_pointer_value();

    // If dtor is null, then skip calling dtor and jump to free_bb.
    let free_bb = gc.context.append_basic_block(release_func, "free");
    let call_dtor_bb = gc.context.append_basic_block(release_func, "call_dtor");
    let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
    let is_dtor_null = gc.builder().build_int_compare(
        IntPredicate::EQ,
        gc.builder()
            .build_ptr_to_int(ptr_to_dtor, ptr_int_ty, "ptr_to_dtor"),
        ptr_int_ty.const_zero(),
        "is_dtor_null",
    );
    gc.builder()
        .build_conditional_branch(is_dtor_null, free_bb, call_dtor_bb);

    // Implement `call_dtor_bb`.
    gc.builder().position_at_end(call_dtor_bb);
    // Call dtor and jump to free_bb.
    let dtor_func = CallableValue::try_from(ptr_to_dtor).unwrap();
    gc.builder().build_call(
        dtor_func,
        &[
            obj_ptr.into(),
            traverser_work_type(gc.context)
                .const_int(TRAVERSER_WORK_RELEASE as u64, false)
                .into(),
        ],
        "call_dtor",
    );
    gc.builder().build_unconditional_branch(free_bb);

    // free.
    gc.builder().position_at_end(free_bb);
    gc.builder().build_free(obj_ptr);
    gc.builder().build_unconditional_branch(end_bb);

    // Implement end_bb.
    gc.builder().position_at_end(end_bb);
    gc.builder().build_return(None);

    // Implement global_bb.
    gc.builder().position_at_end(global_bb);
    // In this case, nothing to do.
    gc.builder().build_return(None);

    // Workaround for #14.
    if gc.config.threaded {
        release_func.add_attribute(
            AttributeLoc::Function,
            gc.context.create_enum_attribute(25 /* noinline */, 0),
        );
    }
    return;
}

fn build_mark_global_or_threaded_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mark_global: bool,
    mode: BuildMode,
) {
    let func_name = if mark_global {
        RUNTIME_MARK_GLOBAL_BOXED_OBJECT
    } else {
        RUNTIME_MARK_THREADED_BOXED_OBJECT
    };

    let mark_func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(func_name) {
                return;
            }
            let void_type = gc.context.void_type();
            let func_type = void_type.fn_type(
                &[
                    ptr_to_object_type(gc.context).into(),
                    ObjectFieldType::TraverseFunction
                        .to_basic_type(gc, vec![])
                        .into(),
                ],
                false,
            );
            gc.module.add_function(func_name, func_type, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(func_name) {
            Some(func) => func,
            None => panic!("Runtime function {} is not declared", func_name),
        },
    };

    let bb = gc.context.append_basic_block(mark_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to the object.
    let ptr_to_obj = mark_func.get_first_param().unwrap().into_pointer_value();

    // Get pointer to traverser function.
    let ptr_to_traverser = mark_func.get_nth_param(1).unwrap().into_pointer_value();

    // If traverser is null, then skip calling traverser.
    let mark_self_bb = gc.context.append_basic_block(mark_func, "mark_self");
    let call_traverser_bb = gc.context.append_basic_block(mark_func, "call_traverser");
    let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
    let is_traverser_null = gc.builder().build_int_compare(
        IntPredicate::EQ,
        gc.builder()
            .build_ptr_to_int(ptr_to_traverser, ptr_int_ty, "ptr_to_traverser"),
        ptr_int_ty.const_zero(),
        "is_traverser_null",
    );
    gc.builder()
        .build_conditional_branch(is_traverser_null, mark_self_bb, call_traverser_bb);

    // Call traverser to mark all subobjects as global.
    gc.builder().position_at_end(call_traverser_bb);
    let traverser = CallableValue::try_from(ptr_to_traverser).unwrap();
    let work = if mark_global {
        TRAVERSER_WORK_MARK_GLOBAL
    } else {
        TRAVERSER_WORK_MARK_THREADED
    };
    gc.builder().build_call(
        traverser,
        &[
            ptr_to_obj.into(),
            traverser_work_type(gc.context)
                .const_int(work as u64, false)
                .into(),
        ],
        "call_traverser_for_mark",
    );
    gc.builder().build_unconditional_branch(mark_self_bb);

    // Mark the object itself.
    gc.builder().position_at_end(mark_self_bb);
    if mark_global {
        gc.mark_global_one(ptr_to_obj);
    } else {
        gc.mark_threaded_one(ptr_to_obj);
    }

    if mark_global && gc.config.sanitize_memory {
        // Report mark global to sanitizer.
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(RUNTIME_REPORT_MARK_GLOBAL, &[obj_id.into()]);
    }

    gc.builder().build_return(None);

    return;
}

fn build_mark_global_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mode: BuildMode,
) {
    build_mark_global_or_threaded_boxed_object_function(gc, true, mode);
}

fn build_mark_threaded_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mode: BuildMode,
) {
    build_mark_global_or_threaded_boxed_object_function(gc, false, mode);
}

fn build_subtract_ptr_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_SUBTRACT_PTR) {
                return;
            }
            let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
            let fn_ty = gc
                .context
                .i64_type()
                .fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
            gc.module.add_function(RUNTIME_SUBTRACT_PTR, fn_ty, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_SUBTRACT_PTR) {
            Some(func) => func,
            None => panic!("Runtime function {} is not declared", RUNTIME_SUBTRACT_PTR),
        },
    };

    let bb = gc.context.append_basic_block(func, "entry");
    let _builder_guard = gc.push_builder();

    gc.builder().position_at_end(bb);
    let lhs = func.get_first_param().unwrap().into_pointer_value();
    let rhs = func.get_nth_param(1).unwrap().into_pointer_value();
    let res = gc
        .builder()
        .build_ptr_diff(lhs, rhs, "ptr_diff@fixruntime_subtract_ptr");
    gc.builder().build_return(Some(&res));
    return;
}

fn build_ptr_add_offset_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let i64_ty = gc.context.i64_type();
    let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));

    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_PTR_ADD_OFFSET) {
                return;
            }
            let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), i64_ty.into()], false);
            gc.module.add_function(RUNTIME_PTR_ADD_OFFSET, fn_ty, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_PTR_ADD_OFFSET) {
            Some(func) => func,
            None => panic!(
                "Runtime function {} is not declared",
                RUNTIME_PTR_ADD_OFFSET
            ),
        },
    };

    let bb = gc.context.append_basic_block(func, "entry");
    let _builder_guard = gc.push_builder();

    gc.builder().position_at_end(bb);
    let ptr = func.get_first_param().unwrap().into_pointer_value();
    let off = func.get_nth_param(1).unwrap().into_int_value();
    let ptr = gc
        .builder()
        .build_ptr_to_int(ptr, i64_ty, "ptr_to_int@fixruntime_ptr_add_offset");
    let ptr = gc
        .builder()
        .build_int_add(ptr, off, "add@fixruntime_ptr_add_offset");
    let ptr = gc
        .builder()
        .build_int_to_ptr(ptr, ptr_ty, "int_to_ptr@fixruntime_ptr_add_offset");
    gc.builder().build_return(Some(&ptr));
    return;
}

pub fn build_pthread_once_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
    mode: BuildMode,
) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_PTHREAD_ONCE) {
        return;
    }

    let init_flag_ty = pthread_once_init_flag_type(gc.context);
    let init_fn_ty = gc.context.void_type().fn_type(&[], false);
    let pthread_once_ty = gc.context.void_type().fn_type(
        &[
            init_flag_ty.ptr_type(AddressSpace::from(0)).into(),
            init_fn_ty.ptr_type(AddressSpace::from(0)).into(),
        ],
        false,
    );
    gc.module
        .add_function(RUNTIME_PTHREAD_ONCE, pthread_once_ty, None);
    return;
}

// Build `fixruntime_run_task` function, which is called from runtime.c.
pub fn build_run_function<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let context = gc.context;
    let module = gc.module;

    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_RUN_FUNCTION) {
                return;
            }
            let fn_type =
                ptr_to_object_type(context).fn_type(&[ptr_to_object_type(context).into()], false);
            module.add_function(RUNTIME_RUN_FUNCTION, fn_type, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_RUN_FUNCTION) {
            Some(func) => func,
            None => panic!("Runtime function {} is not declared", RUNTIME_RUN_FUNCTION),
        },
    };

    let bb = context.append_basic_block(func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Create type `Boxed (() -> a)` where `a = Boxed ()`.
    // Since we don't have information of which the type `a` is, and use `Boxed ()` instead.
    // This is not a problem because the following code only depends on the fact that `a` is a boxed type.
    let boxed_ty = type_tycon(&tycon(FullName::from_strs(&[STD_NAME], BOXED_NAME)));
    let unit_ty = make_unit_ty();
    let task_func_ty = type_fun(
        make_unit_ty(),
        type_tyapp(boxed_ty.clone(), unit_ty.clone()),
    );
    let boxed_task_func_ty = type_tyapp(boxed_ty, task_func_ty);

    // Create an boxed task function object from the function parameter.
    let task_func_ptr = func.get_first_param().unwrap().into_pointer_value();
    let boxed_task_func = Object::new(task_func_ptr, boxed_task_func_ty);

    // Extract task function from `boxed_task_func`.
    let task_func =
        ObjectFieldType::get_struct_fields(gc, &boxed_task_func, vec![(0, None)])[0].clone();

    // Call the task function.
    let unit_val: Object<'_> = allocate_obj(unit_ty, &vec![], None, gc, Some("unit_value"));
    let task_result = gc.apply_lambda(task_func, vec![unit_val], None);
    let task_result_ptr = task_result.ptr(gc);

    gc.builder().build_return(Some(&task_result_ptr));

    return;
}

fn build_thread_prepare_termination_function<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm>,
    mode: BuildMode,
) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_THREAD_PREPARE_TERMINATION) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module
        .add_function(RUNTIME_THREAD_PREPARE_TERMINATION, fn_ty, None);
    return;
}

fn build_thread_terminate_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_THREAD_TERMINATE) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module
        .add_function(RUNTIME_THREAD_TERMINATE, fn_ty, None);
    return;
}

fn build_get_argc_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let argc_gv_ty = gc.context.i32_type();
    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_GET_ARGC) {
                return;
            }
            let fn_ty = argc_gv_ty.fn_type(&[], false);
            gc.module.add_function(RUNTIME_GET_ARGC, fn_ty, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_GET_ARGC) {
            Some(func) => func,
            None => panic!("Runtime function {} is not declared", RUNTIME_GET_ARGC),
        },
    };
    // Add GLOBAL_VAR_NAME_ARGC global variable.
    let argc_gv = gc.module.add_global(argc_gv_ty, None, GLOBAL_VAR_NAME_ARGC);
    argc_gv.set_initializer(&argc_gv_ty.const_zero());

    let bb = gc.context.append_basic_block(func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);
    let argc_ptr = gc
        .module
        .get_global(GLOBAL_VAR_NAME_ARGC)
        .unwrap()
        .as_basic_value_enum()
        .into_pointer_value();
    let argc = gc.builder().build_load(argc_ptr, "argc").into_int_value();
    gc.builder().build_return(Some(&argc));

    return;
}

fn build_get_argv_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let argv_gv_ty = gc
        .context
        .i8_type()
        .ptr_type(AddressSpace::from(0))
        .ptr_type(AddressSpace::from(0));

    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_GET_ARGV) {
                return;
            }

            let fn_ty = gc
                .context
                .i8_type()
                .ptr_type(AddressSpace::from(0))
                .fn_type(&[gc.context.i64_type().into()], false);
            gc.module.add_function(RUNTIME_GET_ARGV, fn_ty, None);
            return;
        }
        BuildMode::Implement => match gc.module.get_function(RUNTIME_GET_ARGV) {
            Some(func) => func,
            None => panic!("Runtime function {} is not declared", RUNTIME_GET_ARGV),
        },
    };

    // Add GLOBAL_VAR_NAME_ARGV global variable.
    let argv_gv = gc.module.add_global(argv_gv_ty, None, GLOBAL_VAR_NAME_ARGV);
    argv_gv.set_initializer(&argv_gv_ty.const_zero());

    let bb = gc.context.append_basic_block(func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);
    let idx = func.get_first_param().unwrap().into_int_value();
    let argv = gc
        .module
        .get_global(GLOBAL_VAR_NAME_ARGV)
        .unwrap()
        .as_basic_value_enum()
        .into_pointer_value();
    let argv = gc.builder().build_load(argv, "argv").into_pointer_value();

    // Get argv[idx].
    // First, offset argv by idx * size_of_pointer.
    let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
    let argv = gc.builder().build_ptr_to_int(argv, ptr_int_ty, "argv");
    let idx = gc.builder().build_int_z_extend(idx, ptr_int_ty, "idx");
    let ptr_size = gc.ptr_size();
    let offset = gc
        .builder()
        .build_int_mul(idx, ptr_int_ty.const_int(ptr_size, false), "offset");
    let argv = gc.builder().build_int_add(argv, offset, "argv");
    let argv = gc.builder().build_int_to_ptr(argv, argv_gv_ty, "argv");
    // Then, load argv[idx].
    let argv = gc.builder().build_load(argv, "argv").into_pointer_value();
    gc.builder().build_return(Some(&argv));

    return;
}
