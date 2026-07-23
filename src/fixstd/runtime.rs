use crate::constants::{GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV};
use crate::generator::Generator;
use inkwell::attributes::{Attribute, AttributeLoc};
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, FunctionValue};
use inkwell::AddressSpace;

pub const RUNTIME_ABORT: &str = "fixruntime_abort";
pub const RUNTIME_INDEX_OUT_OF_RANGE: &str = "fixruntime_index_out_of_range";
pub const RUNTIME_NEGATIVE_ARRAY_SIZE: &str = "fixruntime_negative_array_size";
pub const RUNTIME_EPRINTLN: &str = "fixruntime_eprintln";
pub const RUNTIME_SPRINTF: &str = "sprintf";
pub const RUNTIME_SUBTRACT_PTR: &str = "fixruntime_subtract_ptr";
pub const RUNTIME_PTR_ADD_OFFSET: &str = "fixruntime_ptr_add_offset";
pub const RUNTIME_PTHREAD_ONCE: &str = "pthread_once";
pub const RUNTIME_GET_ARGC: &str = "fixruntime_get_argc";
pub const RUNTIME_GET_ARGV: &str = "fixruntime_get_argv";
/// libc `malloc`, declared with a 64-bit size parameter.
///
/// We declare it ourselves rather than using inkwell's `build_malloc` /
/// `build_array_malloc`, because those wrap LLVM's `CallInst::CreateMalloc`
/// which declares `malloc` with an i32 size parameter and truncates the size
/// before the call, breaking allocations >= 4 GiB.
pub const RUNTIME_MALLOC: &str = "malloc";

/// `realloc`, declared with an i64 size parameter for the same reason as
/// `RUNTIME_MALLOC`: it resizes a single malloc block in place when it can, so
/// growing a uniquely owned array's capacity avoids copying its elements.
pub const RUNTIME_REALLOC: &str = "realloc";

pub fn build_runtime<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    build_abort_function(gc, mode);
    build_index_out_of_range_function(gc, mode);
    build_negative_array_size_function(gc, mode);
    build_eprintf_function(gc, mode);
    build_sprintf_function(gc, mode);
    build_subtract_ptr_function(gc, mode);
    build_ptr_add_offset_function(gc, mode);
    if gc.config.threaded {
        build_pthread_once_function(gc, mode);
    }
    build_get_argc_function(gc, mode);
    build_get_argv_function(gc, mode);
    build_malloc_function(gc, mode);
    build_realloc_function(gc, mode);
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BuildMode {
    Declare,
    Implement,
}

// Attach the valueless LLVM enum attribute `name` to `func` at `loc`.
fn set_enum_attribute<'c, 'm>(
    gc: &Generator<'c, 'm>,
    func: FunctionValue<'c>,
    name: &str,
    loc: AttributeLoc,
) {
    let kind = Attribute::get_named_enum_kind_id(name);
    let attribute = gc.context.create_enum_attribute(kind, 0);
    func.add_attribute(loc, attribute);
}

// Mark a runtime function as `noreturn` so LLVM knows control never continues past a call to it.
// Without this, a bounds-check failure path (which calls the function and then flows to a merge)
// keeps contributing an `undef` value to the merge, forcing an aggregate phi that hides the array
// size and defeats bounds-check elimination.
fn set_noreturn<'c, 'm>(gc: &Generator<'c, 'm>, func: FunctionValue<'c>) {
    set_enum_attribute(gc, func, "noreturn", AttributeLoc::Function);
}

fn build_abort_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_ABORT) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(&[], false);
    let func = gc.module.add_function(RUNTIME_ABORT, fn_ty, None);
    set_noreturn(gc, func);
    return;
}

fn build_index_out_of_range_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_INDEX_OUT_OF_RANGE) {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(
        &[gc.context.i64_type().into(), gc.context.i64_type().into()],
        false,
    );
    let func = gc
        .module
        .add_function(RUNTIME_INDEX_OUT_OF_RANGE, fn_ty, None);
    set_noreturn(gc, func);
    return;
}

fn build_negative_array_size_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_NEGATIVE_ARRAY_SIZE) {
        return;
    }

    let fn_ty = gc
        .context
        .void_type()
        .fn_type(&[gc.context.i64_type().into()], false);
    let func = gc
        .module
        .add_function(RUNTIME_NEGATIVE_ARRAY_SIZE, fn_ty, None);
    set_noreturn(gc, func);
    return;
}

fn build_eprintf_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_EPRINTLN) {
        return;
    }

    let context = gc.context;
    let module = gc.module;

    let ptr_ty = context.ptr_type(AddressSpace::from(0));

    let fn_ty = context.void_type().fn_type(&[ptr_ty.into()], true);
    module.add_function(RUNTIME_EPRINTLN, fn_ty, None);

    return;
}

fn build_sprintf_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_SPRINTF) {
        return;
    }

    let context = gc.context;
    let module = gc.module;

    let i32_ty = context.i32_type();
    let ptr_ty = context.ptr_type(AddressSpace::from(0));

    let fn_ty = i32_ty.fn_type(
        &[
            ptr_ty.into(), /* output buffer */
            ptr_ty.into(), /* format */
        ],
        true,
    );
    module.add_function(RUNTIME_SPRINTF, fn_ty, None);

    return;
}

fn build_subtract_ptr_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_SUBTRACT_PTR) {
                return;
            }
            let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
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
        .build_ptr_diff(
            gc.context.i8_type(),
            lhs,
            rhs,
            "ptr_diff@fixruntime_subtract_ptr",
        )
        .unwrap();
    gc.builder().build_return(Some(&res)).unwrap();
    return;
}

fn build_ptr_add_offset_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let i64_ty = gc.context.i64_type();
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));

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
    let offset = func.get_nth_param(1).unwrap().into_int_value();
    let ptr_int = gc
        .builder()
        .build_ptr_to_int(ptr, i64_ty, "ptr_to_int@fixruntime_ptr_add_offset")
        .unwrap();
    let sum_int = gc
        .builder()
        .build_int_add(ptr_int, offset, "add@fixruntime_ptr_add_offset")
        .unwrap();
    let sum_ptr = gc
        .builder()
        .build_int_to_ptr(sum_int, ptr_ty, "int_to_ptr@fixruntime_ptr_add_offset")
        .unwrap();
    gc.builder().build_return(Some(&sum_ptr)).unwrap();
    return;
}

pub fn build_pthread_once_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_PTHREAD_ONCE) {
        return;
    }

    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let pthread_once_ty = gc
        .context
        .void_type()
        .fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
    gc.module
        .add_function(RUNTIME_PTHREAD_ONCE, pthread_once_ty, None);
    return;
}

fn build_get_argc_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
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
    let argc = gc
        .builder()
        .build_load(argc_gv_ty, argc_ptr, "argc")
        .unwrap()
        .into_int_value();
    gc.builder().build_return(Some(&argc)).unwrap();

    return;
}

fn build_get_argv_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let func = match mode {
        BuildMode::Declare => {
            if let Some(_func) = gc.module.get_function(RUNTIME_GET_ARGV) {
                return;
            }

            let fn_ty = gc
                .context
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
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let argv_gv = gc.module.add_global(ptr_ty, None, GLOBAL_VAR_NAME_ARGV);
    argv_gv.set_initializer(&ptr_ty.const_zero());
    argv_gv.set_linkage(Linkage::Internal);

    let bb = gc.context.append_basic_block(func, "entry");

    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);
    let idx = func.get_first_param().unwrap().into_int_value();
    let argv_gv_ptr = gc
        .module
        .get_global(GLOBAL_VAR_NAME_ARGV)
        .unwrap()
        .as_basic_value_enum()
        .into_pointer_value();
    let argv_ptr = gc
        .builder()
        .build_load(ptr_ty, argv_gv_ptr, "argv")
        .unwrap()
        .into_pointer_value();

    // Get argv[idx].
    // First, offset argv by idx * size_of_pointer.
    let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
    let argv_int = gc
        .builder()
        .build_ptr_to_int(argv_ptr, ptr_int_ty, "argv_int")
        .unwrap();
    let idx = gc
        .builder()
        .build_int_z_extend(idx, ptr_int_ty, "idx")
        .unwrap();
    let ptr_size = gc.ptr_size();
    let offset = gc
        .builder()
        .build_int_mul(idx, ptr_int_ty.const_int(ptr_size, false), "offset")
        .unwrap();
    let elem_int = gc
        .builder()
        .build_int_add(argv_int, offset, "elem_int")
        .unwrap();
    let elem_ptr = gc
        .builder()
        .build_int_to_ptr(elem_int, ptr_ty, "elem_ptr")
        .unwrap();

    // Then, load argv[idx] to get the pointer to the argument string.
    let arg_ptr = gc
        .builder()
        .build_load(ptr_ty, elem_ptr, "arg_ptr")
        .unwrap()
        .into_pointer_value();
    gc.builder().build_return(Some(&arg_ptr)).unwrap();

    return;
}

/// Declares `malloc` in the module with signature `ptr (i64)`, plus the
/// LLVM attributes needed for correct codegen around allocator calls.
fn build_malloc_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_MALLOC) {
        return;
    }
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let i64_ty = gc.context.i64_type();
    let fn_ty = ptr_ty.fn_type(&[i64_ty.into()], false);
    let func = gc.module.add_function(RUNTIME_MALLOC, fn_ty, None);
    // The returned pointer does not alias any other pointer visible to the
    // caller, so mark it `noalias`.
    set_enum_attribute(gc, func, "noalias", AttributeLoc::Return);
    // Mark the function as `nobuiltin` so LLVM does NOT auto-infer the full
    // set of allocator attributes (`allockind`, `allocsize`,
    // `memory(inaccessiblemem: readwrite)`, ...) via TargetLibraryInfo. Those
    // attributes enable an aggressive CSE on loads around the malloc call
    // that, in refcount-state-checking inner loops, ends up spilling a
    // working register. Measured impact: removing this attribute regresses
    // cp_lib_prime_list by +5.9% and cp_lib_lsegtree by +3.0% in wall clock
    // (hyperfine, 30 runs each), with no benchmark in the speedtest suite
    // measurably benefiting from builtin recognition.
    set_enum_attribute(gc, func, "nobuiltin", AttributeLoc::Function);
}

fn build_realloc_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_REALLOC) {
        return;
    }
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let i64_ty = gc.context.i64_type();
    let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), i64_ty.into()], false);
    let func = gc.module.add_function(RUNTIME_REALLOC, fn_ty, None);
    // As for `malloc`, keep LLVM from inferring the full allocator attribute set
    // (see `build_malloc_function`).
    set_enum_attribute(gc, func, "nobuiltin", AttributeLoc::Function);
}
