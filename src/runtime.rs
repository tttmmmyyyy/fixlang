use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum RuntimeFunctions {
    Abort,
    Eprint,
    Sprintf,
    ReportMalloc,
    ReportRetain,
    ReportRelease,
    MarkGlobal,
    CheckLeak,
    RetainBoxedObject,
    ReleaseBoxedObject,
    SubtractPtr,
    PtrAddOffset,
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

fn build_mark_global_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc
        .context
        .void_type()
        .fn_type(&[obj_id_type(gc.context).into()], false);
    gc.module.add_function("mark_as_global", fn_ty, None)
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
    let retain_func = module.add_function("retain_obj", func_type, None);
    let bb = context.append_basic_block(retain_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to / value of reference counter.
    let ptr_to_obj = retain_func.get_first_param().unwrap().into_pointer_value();
    let ptr_to_refcnt = gc.get_refcnt_ptr(ptr_to_obj);

    // Increment refcnt.
    let old_refcnt = {
        if gc.config.atomic_refcnt {
            gc.builder()
                .build_atomicrmw(
                    inkwell::AtomicRMWBinOp::Add,
                    ptr_to_refcnt,
                    refcnt_type(gc.context).const_int(1, false),
                    inkwell::AtomicOrdering::Monotonic,
                )
                .unwrap()
        } else {
            let old_refcnt = gc.builder().build_load(ptr_to_refcnt, "").into_int_value();

            // If old_refcnt is negative as signed integer, the object is reachable from global, and should not be retained, so nothing to do here.
            let is_refcnt_negative = gc.builder().build_int_compare(
                inkwell::IntPredicate::SLT,
                old_refcnt,
                refcnt_type(gc.context).const_zero(),
                "is_refcnt_negative",
            );
            let then_bb = gc
                .context
                .append_basic_block(retain_func, "refcnt_negative@retain_obj");
            let cont_bb = gc
                .context
                .append_basic_block(retain_func, "refcnt_non_negative@retain_obj");
            gc.builder()
                .build_conditional_branch(is_refcnt_negative, then_bb, cont_bb);
            gc.builder().position_at_end(then_bb);
            gc.builder().build_return(None);
            gc.builder().position_at_end(cont_bb);

            // Increment refcnt.
            let refcnt = gc.builder().build_int_nsw_add(
                old_refcnt,
                refcnt_type(gc.context).const_int(1, false).into(),
                "",
            );
            gc.builder().build_store(ptr_to_refcnt, refcnt);
            old_refcnt
        }
    };

    // Report retain to sanitizer.
    if gc.config.sanitize_memory {
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(
            RuntimeFunctions::ReportRetain,
            &[ptr_to_obj.into(), obj_id.into(), old_refcnt.into()],
        );
    }

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
            ObjectFieldType::DtorFunction
                .to_basic_type(gc, vec![])
                .into(),
        ],
        false,
    );
    let release_func = gc.module.add_function("release_obj", func_type, None);
    let bb = gc.context.append_basic_block(release_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to / value of reference counter.
    let ptr_to_obj = release_func.get_first_param().unwrap().into_pointer_value();
    let ptr_to_refcnt = gc.get_refcnt_ptr(ptr_to_obj);

    // Decrement refcnt.
    let old_refcnt = {
        if gc.config.atomic_refcnt {
            gc.builder()
                .build_atomicrmw(
                    inkwell::AtomicRMWBinOp::Sub,
                    ptr_to_refcnt,
                    refcnt_type(gc.context).const_int(1, false),
                    inkwell::AtomicOrdering::Release,
                )
                .unwrap()
        } else {
            let old_refcnt = gc.builder().build_load(ptr_to_refcnt, "").into_int_value();

            // If old_refcnt is negative as signed integer, the object is reachable from global, and should not be released, so nothing to do here.
            let is_refcnt_negative = gc.builder().build_int_compare(
                inkwell::IntPredicate::SLT,
                old_refcnt,
                refcnt_type(gc.context).const_zero(),
                "is_refcnt_negative",
            );
            let then_bb = gc
                .context
                .append_basic_block(release_func, "refcnt_negative@release_obj");
            let cont_bb = gc
                .context
                .append_basic_block(release_func, "refcnt_non_negative@release_obj");
            gc.builder()
                .build_conditional_branch(is_refcnt_negative, then_bb, cont_bb);
            gc.builder().position_at_end(then_bb);
            gc.builder().build_return(None);
            gc.builder().position_at_end(cont_bb);

            // Decrement refcnt.
            let refcnt = gc.builder().build_int_nsw_sub(
                old_refcnt,
                refcnt_type(gc.context).const_int(1, false).into(),
                "",
            );
            gc.builder().build_store(ptr_to_refcnt, refcnt);
            old_refcnt
        }
    };

    // Report release to sanitizer.
    if gc.config.sanitize_memory {
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(
            RuntimeFunctions::ReportRelease,
            &[ptr_to_obj.into(), obj_id.into(), old_refcnt.into()],
        );
    }

    // Branch if old_refcnt is one.
    let is_refcnt_zero = gc.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        old_refcnt,
        refcnt_type(gc.context).const_int(1, false),
        "is_refcnt_zero",
    );
    let then_bb = gc
        .context
        .append_basic_block(release_func, "refcnt_zero_after_release");
    let cont_bb = gc.context.append_basic_block(release_func, "end");
    gc.builder()
        .build_conditional_branch(is_refcnt_zero, then_bb, cont_bb);

    // If refcnt is zero, try to call dtor and free object.
    gc.builder().position_at_end(then_bb);
    gc.builder()
        .build_fence(inkwell::AtomicOrdering::Acquire, 0, "");

    let ptr_to_dtor = release_func.get_nth_param(1).unwrap().into_pointer_value();

    // If dtor is null, then skip calling dtor.
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

    // Call dtor and jump to free_bb
    gc.builder().position_at_end(call_dtor_bb);
    let dtor_func = CallableValue::try_from(ptr_to_dtor).unwrap();
    gc.builder()
        .build_call(dtor_func, &[ptr_to_obj.into()], "call_dtor");
    gc.builder().build_unconditional_branch(free_bb);

    // free.
    gc.builder().position_at_end(free_bb);
    gc.builder().build_free(ptr_to_obj);
    gc.builder().build_unconditional_branch(cont_bb);

    // End function.
    gc.builder().position_at_end(cont_bb);
    gc.builder().build_return(None);

    release_func
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
            .insert(RuntimeFunctions::MarkGlobal, build_mark_global_function(gc));
        gc.runtimes
            .insert(RuntimeFunctions::CheckLeak, build_check_leak_function(gc));
    }
    let retain_func = build_retain_boxed_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::RetainBoxedObject, retain_func);
    let release_func = build_release_boxed_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::ReleaseBoxedObject, release_func);
    let subtract_ptr_func = build_subtract_ptr_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::SubtractPtr, subtract_ptr_func);
    let ptr_add_offset_func = build_ptr_add_offset_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::PtrAddOffset, ptr_add_offset_func);
}
