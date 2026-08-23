use crate::configuration::OutputFileType;
use crate::constants::{C_ENTRY_POINT_NAME, GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV};
use crate::generator::Generator;
use inkwell::attributes::AttributeLoc;
use inkwell::module::Linkage;
use inkwell::types::{BasicMetadataTypeEnum, FunctionType};
use inkwell::values::{BasicValue, FunctionValue};
use inkwell::AddressSpace;

/// The runtime function that ends the program where a check has failed. It returns to no one.
pub const RUNTIME_ABORT: &str = "fixruntime_abort";
/// The runtime function that reports an array index outside the array and ends the program. It
/// takes the index and the size, and returns to no one.
pub const RUNTIME_INDEX_OUT_OF_RANGE: &str = "fixruntime_index_out_of_range";
/// The runtime function that reports a negative array size or capacity and ends the program. It
/// takes the size, and returns to no one.
pub const RUNTIME_NEGATIVE_ARRAY_SIZE: &str = "fixruntime_negative_array_size";
/// The runtime function that reports an array capacity beyond what an element buffer can hold and
/// ends the program. It takes the capacity, and returns to no one.
pub const RUNTIME_ARRAY_SIZE_OVERFLOW: &str = "fixruntime_array_size_overflow";
/// The runtime function that writes a C string to standard error, followed by a newline.
pub const RUNTIME_EPRINTLN: &str = "fixruntime_eprintln";
/// libc `sprintf`, which writes a formatted value into a buffer the caller provides.
pub const RUNTIME_SPRINTF: &str = "sprintf";
/// The runtime function giving the distance in bytes from its second pointer to its first.
pub const RUNTIME_SUBTRACT_PTR: &str = "fixruntime_subtract_ptr";
/// The runtime function giving the address a signed number of bytes past the pointer it is given.
pub const RUNTIME_PTR_ADD_OFFSET: &str = "fixruntime_ptr_add_offset";
/// libc `pthread_once`, which runs an initializer at the first thread to reach it and makes every
/// other thread wait for that run to finish.
pub const RUNTIME_PTHREAD_ONCE: &str = "pthread_once";
/// The runtime function giving the number of command line arguments the program was started with.
pub const RUNTIME_GET_ARGC: &str = "fixruntime_get_argc";
/// The runtime function giving the command line argument at an index, as a C string.
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

/// The prefix under which the compiler names the runtime's own functions, and the globals holding
/// `argc` and `argv`.
pub const RUNTIME_NAME_PREFIX: &str = "fixruntime_";

/// Why the C function name `name` is one the compiler writes the body of when it builds an `output`
/// artifact, phrased to follow "cannot be the name of ...: "; `None` where the program is free to
/// define the function itself.
///
/// A module holds one function under a name, and LLVM renames whichever of two definitions arrives
/// second rather than reporting them. So where the compiler writes a body, a program that writes
/// one too gets a silently renamed function and a program that does not do what it says.
///
/// A C function the compiler only *calls* is not here. Supplying that definition is what linking an
/// object file of one's own does, and it reaches the same place: the compiler emits a call to an
/// undefined symbol either way, and the linker binds it to whatever definition the program brings.
/// Interposing on the C library is therefore the program's to do, and `Document.md` says what it
/// costs.
///
/// # Arguments
/// * `output` — what is being built. The entry point is written into an executable alone, so a
///   dynamic library is free to carry a `main` of its own.
pub fn compiler_defined_c_function_reason(name: &str, output: OutputFileType) -> Option<String> {
    if name == C_ENTRY_POINT_NAME && output == OutputFileType::Executable {
        return Some(
            "it is the entry point of the program, which the compiler defines".to_string(),
        );
    }
    if name.starts_with(RUNTIME_NAME_PREFIX) {
        return Some(format!(
            "a name beginning with `{}` belongs to the Fix runtime",
            RUNTIME_NAME_PREFIX
        ));
    }
    None
}

/// Emits the runtime support functions into the module: their declarations when
/// `mode` is `Declare`, the bodies of the ones implemented here when it is `Implement`.
pub fn build_runtime<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let i64_ty = gc.context.i64_type();
    declare_noreturn_runtime_function(gc, mode, RUNTIME_ABORT, &[]);
    declare_noreturn_runtime_function(
        gc,
        mode,
        RUNTIME_INDEX_OUT_OF_RANGE,
        &[i64_ty.into(), i64_ty.into()],
    );
    declare_noreturn_runtime_function(gc, mode, RUNTIME_NEGATIVE_ARRAY_SIZE, &[i64_ty.into()]);
    declare_noreturn_runtime_function(gc, mode, RUNTIME_ARRAY_SIZE_OVERFLOW, &[i64_ty.into()]);
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

/// Which part of a runtime function a call in `build_runtime` emits.
///
/// The runtime functions split into two groups: those provided externally (by
/// the C runtime, e.g. `malloc`), which need only a declaration, and those
/// implemented in this module (e.g. `fixruntime_ptr_add_offset`), which also
/// need a body. Each build pass runs once in `Declare` mode and once in
/// `Implement` mode.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BuildMode {
    /// Add the function's declaration (signature) to the module.
    Declare,
    /// Emit the body of a function that this module implements itself.
    Implement,
}

/// Declare a runtime function that ends the program: it takes `param_types`, returns nothing, and
/// is marked `noreturn` so LLVM knows control never continues past a call to it.
///
/// Without `noreturn`, a bounds-check failure path (which calls the function and then flows to a
/// merge) keeps contributing an `undef` value to the merge, forcing an aggregate phi that hides the
/// array size and defeats bounds-check elimination.
fn declare_noreturn_runtime_function<'c, 'm>(
    gc: &Generator<'c, 'm>,
    mode: BuildMode,
    name: &str,
    param_types: &[BasicMetadataTypeEnum<'c>],
) {
    if mode != BuildMode::Declare {
        return;
    }
    if gc.module.get_function(name).is_some() {
        return;
    }

    let fn_ty = gc.context.void_type().fn_type(param_types, false);
    let func = gc.module.add_function(name, fn_ty, None);
    gc.add_enum_attribute(func, "noreturn", AttributeLoc::Function);
}

/// Prepare the runtime function `name` of type `fn_ty`, which this module implements itself: in
/// `Declare` mode add its declaration, in `Implement` mode look the declaration up.
///
/// Returns the function whose body the caller is to emit, and `None` in `Declare` mode, where there
/// is no body to emit yet.
fn declare_or_lookup_runtime_function<'c, 'm>(
    gc: &Generator<'c, 'm>,
    mode: BuildMode,
    name: &str,
    fn_ty: FunctionType<'c>,
) -> Option<FunctionValue<'c>> {
    match mode {
        BuildMode::Declare => {
            if gc.module.get_function(name).is_none() {
                gc.module.add_function(name, fn_ty, Some(Linkage::External));
            }
            None
        }
        BuildMode::Implement => Some(
            gc.module
                .get_function(name)
                .unwrap_or_else(|| panic!("Runtime function {} is not declared", name)),
        ),
    }
}

/// Declare `fixruntime_eprintln`, which writes a C string to stderr followed by a newline and
/// flushes it.
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

/// Declare `sprintf`, which takes the output buffer and the format string and goes on to take the
/// values the format names.
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

/// Build `fixruntime_subtract_ptr`, which returns the distance in bytes from its second pointer
/// argument to its first.
fn build_subtract_ptr_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let fn_ty = gc
        .context
        .i64_type()
        .fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
    let Some(func) = declare_or_lookup_runtime_function(gc, mode, RUNTIME_SUBTRACT_PTR, fn_ty)
    else {
        return;
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

/// Build `fixruntime_ptr_add_offset`, which returns the address `offset` bytes past the pointer it
/// is given. The offset is applied to the integer address, so it may be negative and may land
/// outside the object the pointer points into.
fn build_ptr_add_offset_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let i64_ty = gc.context.i64_type();
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));

    let fn_ty = ptr_ty.fn_type(&[ptr_ty.into(), i64_ty.into()], false);
    let Some(func) = declare_or_lookup_runtime_function(gc, mode, RUNTIME_PTR_ADD_OFFSET, fn_ty)
    else {
        return;
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

/// Declare `pthread_once`, which takes the flag recording whether the initializer has run and the
/// initializer itself. A multi-threaded program initializes each global through it.
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

/// Build `fixruntime_get_argc`, which returns the number of command line arguments the program was
/// started with, together with the module-internal global variable holding that number, which the C
/// `main` function stores it into.
fn build_get_argc_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let argc_gv_ty = gc.context.i32_type();
    let fn_ty = argc_gv_ty.fn_type(&[], false);
    let Some(func) = declare_or_lookup_runtime_function(gc, mode, RUNTIME_GET_ARGC, fn_ty) else {
        return;
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

/// Build `fixruntime_get_argv`, which returns a pointer to the command line argument string at the
/// index it is given, together with the module-internal global variable holding the argument array,
/// which the C `main` function stores it into.
fn build_get_argv_function<'c, 'm, 'b>(gc: &mut Generator<'c, 'm>, mode: BuildMode) {
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let fn_ty = ptr_ty.fn_type(&[gc.context.i64_type().into()], false);
    let Some(func) = declare_or_lookup_runtime_function(gc, mode, RUNTIME_GET_ARGV, fn_ty) else {
        return;
    };

    // Add GLOBAL_VAR_NAME_ARGV global variable.
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
    gc.add_enum_attribute(func, "noalias", AttributeLoc::Return);
    // Mark the function as `nobuiltin` so LLVM does NOT auto-infer the full
    // set of allocator attributes (`allockind`, `allocsize`,
    // `memory(inaccessiblemem: readwrite)`, ...) via TargetLibraryInfo. Those
    // attributes enable an aggressive CSE on loads around the malloc call
    // that, in refcount-state-checking inner loops, ends up spilling a
    // working register. Measured impact: removing this attribute regresses
    // cp_lib_prime_list by +5.9% and cp_lib_lsegtree by +3.0% in wall clock
    // (hyperfine, 30 runs each), with no benchmark in the speedtest suite
    // measurably benefiting from builtin recognition.
    gc.add_enum_attribute(func, "nobuiltin", AttributeLoc::Function);
}

/// Declares `realloc` in the module with signature `ptr (ptr, i64)`, plus the LLVM attribute that
/// keeps code generation around allocator calls correct.
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
    gc.add_enum_attribute(func, "nobuiltin", AttributeLoc::Function);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The compiler writes the entry point into an executable alone, so a dynamic library is free to
    /// carry a `main` of its own, while the runtime's own names are the compiler's whatever is being
    /// built and the C library functions it merely calls are the program's either way.
    #[test]
    fn test_which_c_names_the_compiler_writes_the_body_of() {
        assert!(
            compiler_defined_c_function_reason(C_ENTRY_POINT_NAME, OutputFileType::Executable)
                .is_some()
        );
        assert!(compiler_defined_c_function_reason(
            C_ENTRY_POINT_NAME,
            OutputFileType::DynamicLibrary
        )
        .is_none());
        for output in [OutputFileType::Executable, OutputFileType::DynamicLibrary] {
            assert!(compiler_defined_c_function_reason(RUNTIME_ABORT, output).is_some());
            assert!(compiler_defined_c_function_reason(RUNTIME_GET_ARGC, output).is_some());
            assert!(compiler_defined_c_function_reason(RUNTIME_MALLOC, output).is_none());
            assert!(compiler_defined_c_function_reason("free", output).is_none());
            assert!(compiler_defined_c_function_reason("c_of_my_own", output).is_none());
        }
    }
}
