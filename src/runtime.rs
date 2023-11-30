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
    MarkThreadedBoxedObject,
    SubtractPtr,
    PtrAddOffset,
    PthreadOnce,
    ThreadPoolInitialize,
    ThreadPoolTerminate,
    ThreadPoolRunTask,
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
    ) {
        // Report retain to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(
                RuntimeFunctions::ReportRetain,
                &[obj_ptr.into(), obj_id.into()],
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
    report_retain_to_sanitizer(gc, obj_ptr);
    gc.builder().build_return(None);

    // Implement threaded_bb.
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
    ) {
        // Report release to sanitizer.
        if gc.config.sanitize_memory {
            let obj_id = gc.get_obj_id(obj_ptr);
            gc.call_runtime(
                RuntimeFunctions::ReportRelease,
                &[obj_ptr.into(), obj_id.into()],
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

    release_func
}

fn build_mark_global_or_threaded_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mark_global: bool,
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
    let func_name = if mark_global {
        "fixruntime_mark_global_obj"
    } else {
        "fixruntime_mark_threaded_obj"
    };
    let mark_func = gc.module.add_function(func_name, func_type, None);
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
    let ptr_int_ty = gc.context.ptr_sized_int_type(gc.target_data(), None);
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
        gc.call_runtime(RuntimeFunctions::ReportMarkGlobal, &[obj_id.into()]);
    }

    gc.builder().build_return(None);

    mark_func
}

fn build_mark_global_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    build_mark_global_or_threaded_boxed_object_function(gc, true)
}

fn build_mark_threaded_boxed_object_function<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    build_mark_global_or_threaded_boxed_object_function(gc, false)
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

// Build `fixruntime_threadpool_run_task` function, which is called from runtime.c.
pub fn build_threadpool_run_task<'c, 'm>(gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;

    let fn_type = context
        .void_type()
        .fn_type(&[ptr_to_object_type(context).into()], false);
    let func = module.add_function("fixruntime_threadpool_run_task", fn_type, None);

    let bb = context.append_basic_block(func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Create type `TaskData ()` instead of `TaskData a`:
    // We don't have information of the type of task result, but it is not necessary to know it in the following code.
    let task_data_ty = type_tyapp(
        type_tycon(&tycon(FullName::from_strs(&["AsyncTask"], "TaskData"))),
        make_unit_ty(),
    );

    // Create instance of `TaskData`.
    let task_data_ptr = func.get_first_param().unwrap().into_pointer_value();
    let task_data = Object::new(task_data_ptr, task_data_ty);

    // Extract task function from task data.
    let task_func = ObjectFieldType::get_struct_field_noclone(gc, &task_data, 0);
    gc.retain(task_func.clone());

    // Call task function.
    let unit_val: Object<'_> = allocate_obj(make_unit_ty(), &vec![], None, gc, Some("unit_value"));
    let task_result_array = gc.apply_lambda(task_func, vec![unit_val], None);

    // Mark `task_result_array` as threaded.
    // This is necessary because `AsyncTask::Task` object may be threaded upto here.
    gc.mark_threaded(task_result_array.clone());

    // Store the result to task_data.
    let old_array = ObjectFieldType::get_struct_field_noclone(gc, &task_data, 1);
    gc.release(old_array);
    ObjectFieldType::set_struct_field_norelease(gc, &task_data, 1, &task_result_array);

    gc.builder().build_return(None);

    func
}

fn build_get_argc_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    // Add GLOBAL_VAR_NAME_ARGC global variable here.
    let argc_gv_ty = gc.context.i32_type();
    let argc_gv = gc.module.add_global(argc_gv_ty, None, GLOBAL_VAR_NAME_ARGC);
    argc_gv.set_initializer(&argc_gv_ty.const_zero());

    let fn_ty = argc_gv_ty.fn_type(&[], false);
    let func = gc.module.add_function("fixruntime_get_argc", fn_ty, None);
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

    func
}

fn build_get_argv_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
    // Add GLOBAL_VAR_NAME_ARGV global variable here.
    let argv_gv_ty = gc
        .context
        .i8_type()
        .ptr_type(AddressSpace::from(0))
        .ptr_type(AddressSpace::from(0));
    let argv_gv = gc.module.add_global(argv_gv_ty, None, GLOBAL_VAR_NAME_ARGV);
    argv_gv.set_initializer(&argv_gv_ty.const_zero());

    let fn_ty = gc
        .context
        .i8_type()
        .ptr_type(AddressSpace::from(0))
        .fn_type(&[gc.context.i64_type().into()], false);
    let func = gc.module.add_function("fixruntime_get_argv", fn_ty, None);
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
    let ptr_int_ty = gc.context.ptr_sized_int_type(gc.target_data(), None);
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

    func
}

fn build_threadpool_initialize_function<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module
        .add_function("fixruntime_threadpool_initialize", fn_ty, None)
}

fn build_threadpool_terminate_function<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm>,
) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module
        .add_function("fixruntime_threadpool_terminate", fn_ty, None)
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
        let mark_threaded_func = build_mark_threaded_boxed_object_function(gc);
        gc.runtimes.insert(
            RuntimeFunctions::MarkThreadedBoxedObject,
            mark_threaded_func,
        );
    }
    if gc.config.async_task {
        let run_task_func = build_threadpool_run_task(gc);
        gc.runtimes
            .insert(RuntimeFunctions::ThreadPoolRunTask, run_task_func);
        let threadpool_initialize = build_threadpool_initialize_function(gc);
        gc.runtimes.insert(
            RuntimeFunctions::ThreadPoolInitialize,
            threadpool_initialize,
        );
        if gc.config.sanitize_memory {
            // If AsyncTask is used and memory sanitizer is enabled, then we need to terminate thread pool before leak checking.
            let threadpool_terminate = build_threadpool_terminate_function(gc);
            gc.runtimes
                .insert(RuntimeFunctions::ThreadPoolTerminate, threadpool_terminate);
        }
    }
    build_get_argc_function(gc);
    build_get_argv_function(gc);
}
