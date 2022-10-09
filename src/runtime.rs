use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum RuntimeFunctions {
    Abort,
    Printf,
    ReportMalloc,
    ReportRetain,
    ReportRelease,
    CheckLeak,
    RetainObj,
    ReleaseObj,
    Dtor(ObjectType),
}

fn build_abort_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function("abort", fn_ty, None)
}

fn build_printf_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    let fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let func = module.add_function("printf", fn_type, None);

    func
}

fn build_report_malloc_function<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.i64_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            gc.context.i8_type().ptr_type(AddressSpace::Generic).into(),
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

fn build_check_leak_function<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function("check_leak", fn_ty, None)
}

fn build_retain_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
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
    let refcnt = gc
        .builder()
        .build_load(ptr_to_refcnt, "refcnt")
        .into_int_value();

    // Report retain to sanitizer.
    if SANITIZE_MEMORY {
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(
            RuntimeFunctions::ReportRetain,
            &[ptr_to_obj.into(), obj_id.into(), refcnt.into()],
        );
    }

    // Increment refcnt.
    let one = context.i64_type().const_int(1, false);
    let refcnt = gc.builder().build_int_add(refcnt, one, "refcnt");
    gc.builder().build_store(ptr_to_refcnt, refcnt);
    gc.builder().build_return(None);
    // gc.pop_builder();
    retain_func
    // TODO: Add fence instruction for incrementing refcnt
}

fn build_release_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let void_type = gc.context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(gc.context).into()], false);
    let release_func = gc.module.add_function("release_obj", func_type, None);
    let bb = gc.context.append_basic_block(release_func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Get pointer to / value of reference counter.
    let ptr_to_obj = release_func.get_first_param().unwrap().into_pointer_value();
    let ptr_to_refcnt = gc.get_refcnt_ptr(ptr_to_obj);
    let refcnt = gc
        .builder()
        .build_load(ptr_to_refcnt, "refcnt")
        .into_int_value();

    // Report release to sanitizer.
    if SANITIZE_MEMORY {
        let obj_id = gc.get_obj_id(ptr_to_obj);
        gc.call_runtime(
            RuntimeFunctions::ReportRelease,
            &[ptr_to_obj.into(), obj_id.into(), refcnt.into()],
        );
    }

    // Decrement refcnt.
    let one = gc.context.i64_type().const_int(1, false);
    let refcnt = gc.builder().build_int_sub(refcnt, one, "refcnt");
    gc.builder().build_store(ptr_to_refcnt, refcnt);

    // Branch if refcnt is zero.
    let zero = gc.context.i64_type().const_zero();
    let is_refcnt_zero =
        gc.builder()
            .build_int_compare(inkwell::IntPredicate::EQ, refcnt, zero, "is_refcnt_zero");
    let then_bb = gc
        .context
        .append_basic_block(release_func, "refcnt_zero_after_release");
    let cont_bb = gc.context.append_basic_block(release_func, "end");
    gc.builder()
        .build_conditional_branch(is_refcnt_zero, then_bb, cont_bb);

    // If refcnt is zero, then call dtor and free object.
    gc.builder().position_at_end(then_bb);
    gc.call_dtor(ptr_to_obj);
    gc.builder().build_free(ptr_to_obj);
    gc.builder().build_unconditional_branch(cont_bb);

    // End function.
    gc.builder().position_at_end(cont_bb);
    gc.builder().build_return(None);

    // gc.pop_builder();
    release_func
    // TODO: Add fence instruction for incrementing refcnt
    // TODO: Add code for leak detector
}

pub fn build_runtime<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) {
    gc.runtimes
        .insert(RuntimeFunctions::Abort, build_abort_function(gc));
    gc.runtimes
        .insert(RuntimeFunctions::Printf, build_printf_function(gc));
    if SANITIZE_MEMORY {
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
    }
    let retain_func = build_retain_function(gc);
    gc.runtimes.insert(RuntimeFunctions::RetainObj, retain_func);
    let release_func = build_release_function(gc);
    gc.runtimes
        .insert(RuntimeFunctions::ReleaseObj, release_func);
}
