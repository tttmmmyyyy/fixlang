use inkwell::module::Linkage;

use super::*;

pub const RUNTIME_ABORT: &str = "abort";
pub const RUNTIME_EPRINT: &str = "fixruntime_eprint";
pub const RUNTIME_SPRINTF: &str = "sprintf";
// pub const RUNTIME_RETAIN_BOXED_OBJECT: &str = "fixruntime_retain_obj";
// pub const RUNTIME_RELEASE_BOXED_OBJECT: &str = "fixruntime_release_obj";
// pub const RUNTIME_MARK_GLOBAL_BOXED_OBJECT: &str = "fixruntime_mark_global_obj";
// pub const RUNTIME_MARK_THREADED_BOXED_OBJECT: &str = "fixruntime_mark_threaded_obj";
pub const RUNTIME_SUBTRACT_PTR: &str = "fixruntime_subtract_ptr";
pub const RUNTIME_PTR_ADD_OFFSET: &str = "fixruntime_ptr_add_offset";
pub const RUNTIME_PTHREAD_ONCE: &str = "pthread_once";
// pub const RUNTIME_RUN_FUNCTION: &str = "fixruntime_run_function_llvm";
pub const RUNTIME_GET_ARGC: &str = "fixruntime_get_argc";
pub const RUNTIME_GET_ARGV: &str = "fixruntime_get_argv";

pub fn build_runtime<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    build_abort_function(gc, mode);
    build_eprintf_function(gc, mode);
    build_sprintf_function(gc, mode);
    // build_retain_boxed_function(gc, mode);
    // build_release_boxed_function(gc, mode);
    // build_mark_global_boxed_object_function(gc, mode);
    build_subtract_ptr_function(gc, mode);
    build_ptr_add_offset_function(gc, mode);
    if gc.config.threaded {
        build_pthread_once_function(gc, mode);
        // build_mark_threaded_boxed_object_function(gc, mode);
    }
    // build_run_function(gc, mode); // This should be built after `build_mark_threaded_boxed_object_function`.
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
            gc.module.add_function(
                RUNTIME_SUBTRACT_PTR,
                fn_ty,
                Some(gc.config.external_if_separated()),
            );
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
            gc.module.add_function(
                RUNTIME_PTR_ADD_OFFSET,
                fn_ty,
                Some(gc.config.external_if_separated()),
            );
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

fn build_get_argc_function<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm>, mode: BuildMode) {
    let argc_gv_ty = gc.context.i32_type();
    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_GET_ARGC) {
                return;
            }
            let fn_ty = argc_gv_ty.fn_type(&[], false);
            gc.module.add_function(
                RUNTIME_GET_ARGC,
                fn_ty,
                Some(gc.config.external_if_separated()),
            );
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
    argc_gv.set_linkage(Linkage::Internal);

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
            gc.module.add_function(
                RUNTIME_GET_ARGV,
                fn_ty,
                Some(gc.config.external_if_separated()),
            );
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
    argv_gv.set_linkage(Linkage::Internal);

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
