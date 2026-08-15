use crate::constants::{C_ENTRY_POINT_NAME, GLOBAL_VAR_NAME_ARGC, GLOBAL_VAR_NAME_ARGV};
use crate::generator::{module_functions, Generator};
use inkwell::attributes::AttributeLoc;
use inkwell::module::Linkage;
use inkwell::types::{BasicMetadataTypeEnum, FunctionType};
use inkwell::values::{BasicValue, FunctionValue};
use inkwell::AddressSpace;

pub const RUNTIME_ABORT: &str = "fixruntime_abort";
pub const RUNTIME_INDEX_OUT_OF_RANGE: &str = "fixruntime_index_out_of_range";
pub const RUNTIME_NEGATIVE_ARRAY_SIZE: &str = "fixruntime_negative_array_size";
pub const RUNTIME_ARRAY_SIZE_OVERFLOW: &str = "fixruntime_array_size_overflow";
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

/// `free`, which releases the allocation a boxed object lives in.
///
/// We declare it ourselves rather than using inkwell's `build_free`, so that the name enters the
/// module through `build_runtime` like every other C library function the compiler calls, where
/// `RUNTIME_C_LIBRARY_FUNCTIONS` reserves it and the check at the end of `build_runtime` sees it.
pub const RUNTIME_FREE: &str = "free";

/// The prefix under which the compiler names the runtime's own functions, and the globals holding
/// `argc` and `argv`.
pub const RUNTIME_NAME_PREFIX: &str = "fixruntime_";

/// The C library functions the compiler's output calls, which a program therefore cannot define
/// over the top of.
///
/// `build_runtime` declares most of them, and the code generator emits calls to the rest: the array
/// primitives copy element buffers through LLVM's memcpy and memmove intrinsics, which the back end
/// lowers to the C library functions of those names.
const RUNTIME_C_LIBRARY_FUNCTIONS: &[&str] = &[
    RUNTIME_SPRINTF,
    RUNTIME_PTHREAD_ONCE,
    RUNTIME_MALLOC,
    RUNTIME_REALLOC,
    RUNTIME_FREE,
    "memcpy",
    "memmove",
];

/// What the compiler does with the C function name `name`, and `None` where the name is free for a
/// program to use as it likes.
///
/// A module holds one function under a name, so what the compiler does with a name decides what a
/// program may do with it: LLVM renames whichever of two definitions of one symbol arrives second,
/// and the program that comes out calls something other than what it names.
///
/// The answer is the same however the program is built. `pthread_once` reaches the module only in a
/// multi-threaded program and the entry point only in an executable, and the compiler's claim on
/// both holds everywhere, so turning multi-threading on or building the same source as a dynamic
/// library leaves the set of programs that compile as it was.
pub fn compiler_use_of_c_function_name(name: &str) -> Option<CompilerNameUse> {
    if name == C_ENTRY_POINT_NAME {
        return Some(CompilerNameUse::Defines(
            "it is the entry point of the program, which the compiler defines".to_string(),
        ));
    }
    if name.starts_with(RUNTIME_NAME_PREFIX) {
        return Some(CompilerNameUse::Calls(format!(
            "a name beginning with `{}` belongs to the Fix runtime",
            RUNTIME_NAME_PREFIX
        )));
    }
    if RUNTIME_C_LIBRARY_FUNCTIONS.contains(&name) {
        return Some(CompilerNameUse::Calls(
            "it is a C library function the Fix runtime calls".to_string(),
        ));
    }
    None
}

/// What the compiler does with a C function name, which is what decides what a program may do with
/// it. Each carries the reason, phrased to follow "cannot be the name of ...: ".
pub enum CompilerNameUse {
    /// The compiler writes this function's body, so the name is its own: a program that names it at
    /// all takes it away, whether to define it or to call it.
    Defines(String),
    /// The compiler calls this function, which something outside the program defines. A program may
    /// call it too — that reaches the same function — and may not define it over the top.
    Calls(String),
}

impl CompilerNameUse {
    /// Why the compiler holds the name.
    pub fn reason(&self) -> &str {
        match self {
            CompilerNameUse::Defines(reason) => reason,
            CompilerNameUse::Calls(reason) => reason,
        }
    }
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
    build_free_function(gc, mode);

    // Every name the calls above put in the module is one an `FFI_EXPORT` of it would take away, so
    // each has to be one `compiler_use_of_c_function_name` answers for. It answers by prefix for the
    // runtime's own functions and by name for the C library ones, so `RUNTIME_C_LIBRARY_FUNCTIONS`
    // is what a runtime function calling a new C library function would leave behind.
    if mode == BuildMode::Declare {
        for function in module_functions(gc.module) {
            let name = function.get_name().to_str().unwrap();
            assert!(
                compiler_use_of_c_function_name(name).is_some(),
                "the runtime declares `{}`, which a program is free to export",
                name
            );
        }
    }
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
                gc.module
                    .add_function(name, fn_ty, Some(gc.config.external_if_separated()));
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

/// Declares `free` in the module with signature `void (ptr)`, plus the `nobuiltin` attribute the
/// other allocator functions carry.
fn build_free_function<'c, 'm, 'b>(gc: &Generator<'c, 'm>, mode: BuildMode) {
    if mode != BuildMode::Declare {
        return;
    }
    if let Some(_func) = gc.module.get_function(RUNTIME_FREE) {
        return;
    }
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let fn_ty = gc.context.void_type().fn_type(&[ptr_ty.into()], false);
    let func = gc.module.add_function(RUNTIME_FREE, fn_ty, None);
    // As for `malloc`, keep LLVM from inferring the full allocator attribute set
    // (see `build_malloc_function`).
    gc.add_enum_attribute(func, "nobuiltin", AttributeLoc::Function);
}
