use inkwell::{context::Context, types::IntType, values::IntValue};

use crate::{ast::program::Program, configuration::Configuration, runtime::RuntimeFunctions};

pub const NAMESPACE_SEPARATOR: &str = "::";

pub const STD_NAME: &str = "Std";
pub const IO_NAME: &str = "IO";
pub const PTR_NAME: &str = "Ptr";
pub const U8_NAME: &str = "U8";
pub const I8_NAME: &str = "I8";
pub const U16_NAME: &str = "U16";
pub const I16_NAME: &str = "I16";
pub const I32_NAME: &str = "I32";
pub const U32_NAME: &str = "U32";
pub const I64_NAME: &str = "I64";
pub const U64_NAME: &str = "U64";
pub const F32_NAME: &str = "F32";
pub const F64_NAME: &str = "F64";
pub const BOOL_NAME: &str = "Bool";
pub const ARRAY_NAME: &str = "Array";
pub const LAZY_NAME: &str = "Lazy";
pub const DESTRUCTOR_OBJECT_NAME: &str = "Destructor";
pub const DESTRUCTOR_OBJECT_VALUE_FIELD_IDX: u32 = 0;
pub const DESTRUCTOR_OBJECT_DTOR_FIELD_IDX: u32 = 1;
pub const STRING_NAME: &str = "String";
pub const MONAD_NAME: &str = "Monad";
pub const MONAD_BIND_NAME: &str = "bind";
pub const COMPOSE_FUNCTION_NAME: &str = "compose";
pub const MAIN_FUNCTION_NAME: &str = "main";
pub const MAIN_MODULE_NAME: &str = "Main";

pub const ARG_NAME: &str = "#arg";
pub const EVAL_VAR_NAME: &str = "#eval_var";
pub const FUNPTR_NAME: &str = "#FunPtr";
pub const DYNAMIC_OBJECT_NAME: &str = "#DynamicObject";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "#";
pub const ARRAY_GETTER_FUNCTION_NAME: &str = "@";
pub const STRUCT_GETTER_SYMBOL: &str = "@";
pub const STRUCT_SETTER_SYMBOL: &str = "set_";

pub const CAP_NAME: &str = "#CAP";

pub const LOOP_RESULT_CONTINUE_IDX: usize = 0;

pub const CLOSURE_FUNPTR_IDX: u32 = 0;
pub const CLOSURE_CAPTURE_IDX: u32 = CLOSURE_FUNPTR_IDX + 1;
pub const ARRAY_LEN_IDX: u32 = 1/* ControlBlock */;
pub const ARRAY_CAP_IDX: u32 = ARRAY_LEN_IDX + 1;
pub const ARRAY_BUF_IDX: u32 = ARRAY_CAP_IDX + 1;
pub const DYNAMIC_OBJ_TRAVARSER_IDX: u32 = 1/* Next of ControlBlock */;
pub const DYNAMIC_OBJ_CAP_IDX: u32 = DYNAMIC_OBJ_TRAVARSER_IDX + 1;

// REFCNT_STATE_* values are stored to a field of the control block of each boxed object.
pub const REFCNT_STATE_LOCAL: u8 = 0; // This is local object in the sense that it is not shared with other threads but should be released since it is not global.
pub const REFCNT_STATE_THREADED: u8 = 1; // This object is shared between multiple threads and should be released or retained atomically.
pub const REFCNT_STATE_GLOBAL: u8 = 2; // This is global object and should not be released or retained.

pub const CTRL_BLK_REFCNT_IDX: u32 = 0;
pub const CTRL_BLK_REFCNT_STATE_IDX: u32 = 1;
pub const CTRL_BLK_OBJ_ID_IDX: u32 = 2;

pub const TYPE_CHECK_CACHE_PATH: &str = ".fixlang/type_check_cache";
pub const DOT_FIXLANG: &str = ".fixlang";
pub const INTERMEDIATE_PATH: &str = ".fixlang/intermediate";

pub const ASYNCTASK_NAME: &str = "AsyncTask";

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TraverserWorkType(pub u32);
impl TraverserWorkType {
    pub fn release() -> Self {
        Self(TRAVERSER_WORK_RELEASE)
    }
    pub fn mark_global() -> Self {
        Self(TRAVERSER_WORK_MARK_GLOBAL)
    }
    pub fn mark_threaded() -> Self {
        Self(TRAVERSER_WORK_MARK_THREADED)
    }
    pub fn runtime_function(&self) -> RuntimeFunctions {
        match self.0 {
            TRAVERSER_WORK_RELEASE => RuntimeFunctions::ReleaseBoxedObject,
            TRAVERSER_WORK_MARK_GLOBAL => RuntimeFunctions::MarkGlobalBoxedObject,
            TRAVERSER_WORK_MARK_THREADED => RuntimeFunctions::MarkThreadedBoxedObject,
            _ => unreachable!(),
        }
    }
}
pub const TRAVERSER_WORK_RELEASE: u32 = 0;
pub const TRAVERSER_WORK_MARK_GLOBAL: u32 = 1;
pub const TRAVERSER_WORK_MARK_THREADED: u32 = 2;

pub const STANDARD_LIBRARIES: &[(
    &str,                           /* mod_name */
    &str,                           /* source_content */
    &str,                           /* file_name */
    Option<fn(&mut Configuration)>, /* config_modifier */
    Option<fn(&mut Program)>,       /* module_modifier */
)] = &[
    (
        "Debug",
        include_str!("./fix/debug.fix"),
        "debug",
        None,
        None,
    ),
    ("Hash", include_str!("./fix/hash.fix"), "hash", None, None),
    (
        "HashMap",
        include_str!("./fix/hashmap.fix"),
        "hashmap",
        None,
        None,
    ),
    (
        "HashSet",
        include_str!("./fix/hashset.fix"),
        "hashset",
        None,
        None,
    ),
    (
        "Math",
        include_str!("./fix/math.fix"),
        "math",
        Some(Configuration::add_libm),
        None,
    ),
    (
        "Time",
        include_str!("./fix/time.fix"),
        "time.fix",
        None,
        None,
    ),
    (
        "Character",
        include_str!("./fix/character.fix"),
        "character",
        None,
        None,
    ),
    (
        "Subprocess",
        include_str!("./fix/subprocess.fix"),
        "subprocess",
        None,
        None,
    ),
    (
        ASYNCTASK_NAME,
        include_str!("./fix/asynctask.fix"),
        "asynctask",
        Some(Configuration::set_async_task),
        None,
    ),
];

#[allow(unused)]
pub const DW_ATE_ADDRESS: u32 = 1;
#[allow(unused)]
pub const DW_ATE_BOOLEAN: u32 = 2;
#[allow(unused)]
pub const DW_ATE_FLOAT: u32 = 4;
#[allow(unused)]
pub const DW_ATE_SIGNED: u32 = 5;
#[allow(unused)]
pub const DW_ATE_SIGNED_CHAR: u32 = 6;
#[allow(unused)]
pub const DW_ATE_UNSIGNED: u32 = 7;
#[allow(unused)]
pub const DW_ATE_UNSINGED_CHAR: u32 = 8;

// Max number of arguments of function pointer lambda.
pub const FUNPTR_ARGS_MAX: u32 = 100;
// Max tuple size.
// This affects on compilation time heavily. We should make tuple generation on-demand in a future.
pub const TUPLE_SIZE_MAX: u32 = 4;
// Is tuple unboxed?
pub const TUPLE_UNBOX: bool = true;

// The type in LLVM corresponding to `pthread_once_t` of this system.
pub fn pthread_once_init_flag_type<'c>(ctx: &'c Context) -> IntType<'c> {
    ctx.i32_type()
}

// The value of `PTHREAD_ONCE_INIT` of this system.
pub fn pthread_once_init_flag_value<'c>(ctx: &'c Context) -> IntValue<'c> {
    pthread_once_init_flag_type(ctx).const_zero()
}

pub const GLOBAL_VAR_NAME_ARGC: &str = "fixruntime_argc";
pub const GLOBAL_VAR_NAME_ARGV: &str = "fixruntime_argv";
