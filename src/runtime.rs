use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum RuntimeFunctions {
    Printf,
    ReportMalloc,
    ReportRetain,
    ReportRelease,
    CheckLeak,
    PrintIntObj,
    RetainObj,
    ReleaseObj,
    Dtor(ObjectType),
}

fn generate_func_printf<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    let fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let func = module.add_function("printf", fn_type, None);

    func
}

fn generate_func_report_malloc<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let fn_ty = gc.context.i64_type().fn_type(
        &[
            ptr_to_object_type(gc.context).into(),
            gc.context.i8_type().ptr_type(AddressSpace::Generic).into(),
        ],
        false,
    );
    gc.module.add_function("report_malloc", fn_ty, None)
}

fn generate_func_report_retain<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
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

fn generate_func_report_release<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
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

fn generate_check_leak<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
    let fn_ty = gc.context.void_type().fn_type(&[], false);
    gc.module.add_function("check_leak", fn_ty, None)
}

fn generate_func_print_int_obj<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let system_functions = &gc.runtimes;
    let void_type = context.void_type();
    let int_obj_type = ObjectType::int_obj_type().to_struct_type(context);
    let int_obj_ptr_type = int_obj_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[int_obj_ptr_type.into()], false);
    let func = module.add_function("print_int_obj", fn_type, None);

    let entry_bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry_bb);
    let int_obj_ptr = func.get_first_param().unwrap().into_pointer_value();
    let int_field_ptr = builder
        .build_struct_gep(int_obj_ptr, 1, "int_field_ptr")
        .unwrap();
    let int_val = builder
        .build_load(int_field_ptr, "int_val")
        .into_int_value();
    let string_ptr = builder.build_global_string_ptr("%lld\n", "int_placefolder");
    let printf_func = *system_functions.get(&RuntimeFunctions::Printf).unwrap();
    builder.build_call(
        printf_func,
        &[string_ptr.as_pointer_value().into(), int_val.into()],
        "call_print_int",
    );
    builder.build_return(None);

    func
}

fn generate_func_retain_obj<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(context).into()], false);
    let retain_func = module.add_function("retain_obj", func_type, None);
    let bb = context.append_basic_block(retain_func, "entry");

    let builder = context.create_builder();
    let (mut new_gc, pop_gc) = gc.push_builder(&builder);
    {
        let gc = &mut new_gc;
        builder.position_at_end(bb);

        // Get pointer to / value of reference counter.
        let ptr_to_obj = retain_func.get_first_param().unwrap().into_pointer_value();
        let ptr_to_refcnt = gc.build_ptr_to_refcnt(ptr_to_obj);
        let refcnt = builder.build_load(ptr_to_refcnt, "refcnt").into_int_value();

        // Report retain to sanitizer.
        if SANITIZE_MEMORY {
            let objid = build_get_objid(ptr_to_obj, gc);
            builder.build_call(
                *gc.runtimes.get(&RuntimeFunctions::ReportRetain).unwrap(),
                &[ptr_to_obj.into(), objid.into(), refcnt.into()],
                "call_report_retain",
            );
        }

        // Increment refcnt.
        let one = context.i64_type().const_int(1, false);
        let refcnt = builder.build_int_add(refcnt, one, "refcnt");
        builder.build_store(ptr_to_refcnt, refcnt);
        builder.build_return(None);
    }
    pop_gc(new_gc);
    retain_func
    // TODO: Add fence instruction for incrementing refcnt
}

fn generate_func_release_obj<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let void_type = gc.context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(gc.context).into()], false);
    let release_func = gc.module.add_function("release_obj", func_type, None);
    let bb = gc.context.append_basic_block(release_func, "entry");

    let builder = gc.context.create_builder();
    let (mut new_gc, pop_gc) = gc.push_builder(&builder);
    {
        let gc = &mut new_gc;
        builder.position_at_end(bb);

        // Get pointer to / value of reference counter.
        let ptr_to_obj = release_func.get_first_param().unwrap().into_pointer_value();
        let ptr_to_refcnt = gc.build_ptr_to_refcnt(ptr_to_obj);
        let refcnt = builder.build_load(ptr_to_refcnt, "refcnt").into_int_value();

        // Report release to sanitizer.
        if SANITIZE_MEMORY {
            let objid = build_get_objid(ptr_to_obj, gc);
            gc.builder.build_call(
                *gc.runtimes.get(&RuntimeFunctions::ReportRelease).unwrap(),
                &[ptr_to_obj.into(), objid.into(), refcnt.into()],
                "report_release_call",
            );
        }

        // Decrement refcnt.
        let one = gc.context.i64_type().const_int(1, false);
        let refcnt = builder.build_int_sub(refcnt, one, "refcnt");
        builder.build_store(ptr_to_refcnt, refcnt);

        // Branch if refcnt is zero.
        let zero = gc.context.i64_type().const_zero();
        let is_refcnt_zero =
            builder.build_int_compare(inkwell::IntPredicate::EQ, refcnt, zero, "is_refcnt_zero");
        let then_bb = gc
            .context
            .append_basic_block(release_func, "refcnt_zero_after_release");
        let cont_bb = gc.context.append_basic_block(release_func, "end");
        builder.build_conditional_branch(is_refcnt_zero, then_bb, cont_bb);

        // If refcnt is zero, then call dtor and free object.
        builder.position_at_end(then_bb);
        let ptr_to_control_block = gc.build_ptr_to_control_block(ptr_to_obj);
        let ptr_to_dtor_ptr = builder
            .build_struct_gep(ptr_to_control_block, 1, "ptr_to_dtor_ptr")
            .unwrap();
        let ptr_to_dtor = builder
            .build_load(ptr_to_dtor_ptr, "ptr_to_dtor")
            .into_pointer_value();
        let dtor_func = CallableValue::try_from(ptr_to_dtor).unwrap();
        builder.build_call(dtor_func, &[ptr_to_obj.into()], "call_dtor");
        builder.build_free(ptr_to_obj);
        builder.build_unconditional_branch(cont_bb);

        // End function.
        builder.position_at_end(cont_bb);
        builder.build_return(None);
    }
    pop_gc(new_gc);
    release_func
    // TODO: Add fence instruction for incrementing refcnt
    // TODO: Add code for leak detector
}

fn generate_func_empty_destructor<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let ptr_to_obj_type = context.i64_type().ptr_type(AddressSpace::Generic);
    let func_type = void_type.fn_type(&[ptr_to_obj_type.into()], false);
    let func = module.add_function("empty_destructor", func_type, None);
    let bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(bb);
    builder.build_return(None);

    func
}

fn generate_func_dtor<'c, 'm, 'b>(
    obj_type: StructType<'c>,
    subobj_indices: &[i32],
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let ptr_to_obj_type = obj_type.ptr_type(AddressSpace::Generic);
    let func_type = void_type.fn_type(&[ptr_to_obj_type.into()], false);
    let func = module.add_function("destructor", func_type, None); // TODO: give appropriate name
    let bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(bb);
    builder.build_return(None);
    func
}

pub fn generate_system_functions<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm, 'b>) {
    gc.runtimes
        .insert(RuntimeFunctions::Printf, generate_func_printf(gc));
    if SANITIZE_MEMORY {
        gc.runtimes.insert(
            RuntimeFunctions::ReportMalloc,
            generate_func_report_malloc(gc),
        );
        gc.runtimes.insert(
            RuntimeFunctions::ReportRetain,
            generate_func_report_retain(gc),
        );
        gc.runtimes.insert(
            RuntimeFunctions::ReportRelease,
            generate_func_report_release(gc),
        );
        gc.runtimes
            .insert(RuntimeFunctions::CheckLeak, generate_check_leak(gc));
    }
    gc.runtimes.insert(
        RuntimeFunctions::PrintIntObj,
        generate_func_print_int_obj(gc),
    );
    let retain_func = generate_func_retain_obj(gc);
    gc.runtimes.insert(RuntimeFunctions::RetainObj, retain_func);
    let release_func = generate_func_release_obj(gc);
    gc.runtimes
        .insert(RuntimeFunctions::ReleaseObj, release_func);
}
