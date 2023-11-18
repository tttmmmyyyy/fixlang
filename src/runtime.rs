use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum RuntimeFunctions {
    Abort,
    Eprint,
    Sprintf,
    ReportMalloc,
    ReportRetain,
    ReportRelease,
    ReportMarkGlobal,
    CheckLeak,
    RetainBoxedObject,
    ReleaseBoxedObject,
    MarkGlobalBoxedObject,
    SubtractPtr,
    PtrAddOffset,
    PthreadOnce,
}

fn build_abort_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function("abort", fn_ty, None)
}

fn build_eprintf_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;

    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::from(0));

    let fn_type = context.void_type().fn_type(&[i8_ptr_type.into()], true);
    let func = module.add_function("fixruntime_eprint", fn_type, None);

    func
}

fn build_sprintf_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
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
    let func = module.add_function("sprintf", fn_type, None);

    func
}

fn build_report_malloc_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.i64_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            gc.context.i8_type().ptr_type(AddressSpace::from(0)).into(),
        ],
        false,
    );
    gc.module.add_function("report_malloc", fn_ty, None)
}

fn build_report_retain_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            obj_id_type(gc.context).into(),
            refcnt_type(gc.context).into(),
        ],
        false,
    );
    gc.module.add_function("report_retain", fn_ty, None)
}

fn build_report_release_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            obj_id_type(gc.context).into(),
            refcnt_type(gc.context).into(),
        ],
        false,
    );
    gc.module.add_function("report_release", fn_ty, None)
}

fn build_report_mark_global_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc
        .context
        .void_type()
        .fn_type(&[obj_id_type(gc.context).into()], false);
    gc.module.add_function("report_mark_global", fn_ty, None)
}

fn build_check_leak_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function("check_leak", fn_ty, None)
}

fn build_retain_boxed_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(context).into()], false);
    let retain_func = module.add_function("fixruntime_retain_obj", func_type, None);
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
        old_refcnt: IntValue<'c>,
    ) {
        // Report retain to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(
                RuntimeFunctions::ReportRetain,
                &[obj_ptr.into(), obj_id.into(), old_refcnt.into()],
            );
        }
    }

    // Implement `local_bb`.
    gc.builder().position_at_end(local_bb);
    // Increment refcnt and return.
    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
    let old_refcnt_local = gc.builder().build_load(ptr_to_refcnt, "").into_int_value();
    let new_refcnt = gc.builder().build_int_nsw_add(
        old_refcnt_local,
        refcnt_type(gc.context).const_int(1, false).into(),
        "",
    );
    gc.builder().build_store(ptr_to_refcnt, new_refcnt);
    report_retain_to_sanitizer(gc, obj_ptr, old_refcnt_local);
    gc.builder().build_return(None);

    // Implement threaded_bb.
    gc.builder().position_at_end(threaded_bb);
    // Increment refcnt atomically and jump to `end_bb`.
    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
    let old_refcnt_threaded = gc
        .builder()
        .build_atomicrmw(
            inkwell::AtomicRMWBinOp::Add,
            ptr_to_refcnt,
            refcnt_type(gc.context).const_int(1, false),
            inkwell::AtomicOrdering::Monotonic,
        )
        .unwrap();
    report_retain_to_sanitizer(gc, obj_ptr, old_refcnt_threaded);
    gc.builder().build_return(None);

    // Implement global_bb.
    gc.builder().position_at_end(global_bb);
    // In this case, nothing to do.
    gc.builder().build_return(None);

    retain_func
}

fn build_release_boxed_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
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
    let release_func = gc
        .module
        .add_function("fixruntime_release_obj", func_type, None);
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
        old_refcnt: IntValue<'c>,
    ) {
        // Report release to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(
                RuntimeFunctions::ReportRelease,
                &[obj_ptr.into(), obj_id.into(), old_refcnt.into()],
            );
        }
    }

    // Implement local_bb.
    gc.builder().position_at_end(local_bb);
    let ptr_to_refcnt = gc.get_refcnt_ptr(obj_ptr);
    // Decrement refcnt.
    let old_refcnt = gc.builder().build_load(ptr_to_refcnt, "").into_int_value();
    let new_refcnt = gc.builder().build_int_nsw_sub(
        old_refcnt,
        refcnt_type(gc.context).const_int(1, false).into(),
        "",
    );
    gc.builder().build_store(ptr_to_refcnt, new_refcnt);
    report_release_to_sanitizer(gc, obj_ptr, old_refcnt);

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
    report_release_to_sanitizer(gc, obj_ptr, old_refcnt);

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

    // Implement `destruction_bb`
    gc.builder().position_at_end(destruction_bb);

    // Get dtor.
    let ptr_to_dtor = release_func.get_nth_param(1).unwrap().into_pointer_value();

    // If dtor is null, then skip calling dtor and jump to free_bb.
    let free_bb = gc.context.append_basic_block(release_func, "free");
    let call_dtor_bb = gc.context.append_basic_block(release_func, "call_dtor");
    let ptr_int_ty = gc.context.ptr_sized_int_type(gc.target_data(), None);
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
    let null_ptr = ptr_to_object_type(gc.context).const_null();
    gc.builder()
        .build_call(dtor_func, &[obj_ptr.into(), null_ptr.into()], "call_dtor");
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

    release_func
}

fn build_mark_global_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
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
    let mark_func = gc
        .module
        .add_function("fixruntime_mark_global", func_type, None);
    let bb = gc.context.append_basic_block(mark_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to / value of reference counter.
    let ptr_to_obj = mark_func.get_first_param().unwrap().into_pointer_value();

    // Get pointer to traverser function.
    let ptr_to_traverser = mark_func.get_nth_param(1).unwrap().into_pointer_value();

    // If traverser is null, then skip calling traverser.
    let mark_self_global_bb = gc.context.append_basic_block(mark_func, "mark_self_global");
    let call_traverser_bb = gc.context.append_basic_block(mark_func, "call_traverser");
    let ptr_int_ty = gc.context.ptr_sized_int_type(gc.target_data(), None);
    let is_traverser_null = gc.builder().build_int_compare(
        IntPredicate::EQ,
        gc.builder()
            .build_ptr_to_int(ptr_to_traverser, ptr_int_ty, "ptr_to_traverser"),
        ptr_int_ty.const_zero(),
        "is_traverser_null",
    );
    gc.builder().build_conditional_branch(
        is_traverser_null,
        mark_self_global_bb,
        call_traverser_bb,
    );

    // Call traverser to mark all subobjects as global.
    gc.builder().position_at_end(call_traverser_bb);
    let dtor_func = CallableValue::try_from(ptr_to_traverser).unwrap();
    let one_ptr = gc.object_ptr_one();
    gc.builder().build_call(
        dtor_func,
        &[ptr_to_obj.into(), one_ptr.into()],
        "call_traverser_for_mark",
    );
    gc.builder().build_unconditional_branch(mark_self_global_bb);

    // Mark the object itself as global.
    gc.builder().position_at_end(mark_self_global_bb);
    gc.mark_global_one(ptr_to_obj);

    // Report mark global to sanitizer.
    if gc.config.sanitize_memory {
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(RuntimeFunctions::ReportMarkGlobal, &[obj_id.into()]);
    }

    gc.builder().build_return(None);

    mark_func
}

fn build_subtract_ptr_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
    let fn_ty = gc
        .context
        .i64_type()
        .fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
    let func = gc
        .module
        .add_function("fixruntime_subtract_ptr", fn_ty, None);
    let bb = gc.context.append_basic_block(func, "entry");
    let _builder_guard = gc.push_builder();

    gc.builder().position_at_end(bb);
    let lhs = func.get_first_param().unwrap().into_pointer_value();
    let rhs = func.get_nth_param(1).unwrap().into_pointer_value();
    let res = gc
        .builder()
        .build_ptr_diff(lhs, rhs, "ptr_diff@fixruntime_subtract_ptr");
    gc.builder().build_return(Some(&res));
    func
}

fn build_ptr_add_offset_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
    let i64_ty = gc.context.i64_type();
    let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), i64_ty.into()], false);
    let func = gc
        .module
        .add_function("fixruntime_ptr_add_offset", fn_ty, None);
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
    func
}

pub fn build_pthread_once_function<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
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
        .add_function("pthread_once", pthread_once_ty, None)
}

pub fn build_runtime<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) {
    gc.runtimes
        .insert(RuntimeFunctions::Abort, build_abort_function(gc));
    gc.runtimes
        .insert(RuntimeFunctions::Eprint, build_eprintf_function(gc));
    gc.runtimes
        .insert(RuntimeFunctions::Sprintf, build_sprintf_function(gc));
    if gc.config.sanitize_memory {
        gc.runtimes.insert(
            RuntimeFunctions::ReportMalloc,
            build_report_malloc_function(gc),
        );
        gc.runtimes.insert(
            RuntimeFunctions::ReportRetain,
            build_report_retain_function(gc),
        );
        gc.runtimes.insert(
            RuntimeFunctions::ReportRelease,
            build_report_release_function(gc),
        );
        gc.runtimes
            .insert(RuntimeFunctions::CheckLeak, build_check_leak_function(gc));
        gc.runtimes.insert(
            RuntimeFunctions::ReportMarkGlobal,
            build_report_mark_global_function(gc),
        );
    }
    let retain_func = build_retain_boxed_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::RetainBoxedObject, retain_func);
    let release_func = build_release_boxed_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::ReleaseBoxedObject, release_func);
    let mark_func = build_mark_global_boxed_object_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::MarkGlobalBoxedObject, mark_func);
    let subtract_ptr_func = build_subtract_ptr_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::SubtractPtr, subtract_ptr_func);
    let ptr_add_offset_func = build_ptr_add_offset_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::PtrAddOffset, ptr_add_offset_func);
    if gc.config.threaded {
        let pthread_call_once_func = build_pthread_once_function(gc);
        gc.runtimes
            .insert(RuntimeFunctions::PthreadOnce, pthread_call_once_func);
    }
}
