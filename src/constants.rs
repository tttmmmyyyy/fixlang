use inkwell::{context::Context, types::IntType, values::IntValue};

pub const NAMESPACE_SEPARATOR: &str = "::";
pub const MODULE_SEPARATOR: &str = ".";

pub const STD_NAME: &str = "Std";
pub const FFI_NAME: &str = "FFI";
pub const IO_NAME: &str = "IO";
pub const IO_DATA_NAME: &str = "runner";
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
pub const ARROW_NAME: &str = "Arrow";

pub const C_CHAR_NAME: &str = "CChar";
pub const C_UNSIGNED_CHAR_NAME: &str = "CUnsignedChar";
pub const C_SHORT_NAME: &str = "CShort";
pub const C_UNSIGNED_SHORT_NAME: &str = "CUnsignedShort";
pub const C_INT_NAME: &str = "CInt";
pub const C_UNSIGNED_INT_NAME: &str = "CUnsignedInt";
pub const C_LONG_NAME: &str = "CLong";
pub const C_UNSIGNED_LONG_NAME: &str = "CUnsignedLong";
pub const C_LONG_LONG_NAME: &str = "CLongLong";
pub const C_UNSIGNED_LONG_LONG_NAME: &str = "CUnsignedLongLong";
pub const C_SIZE_T_NAME: &str = "CSizeT";
pub const C_FLOAT_NAME: &str = "CFloat";
pub const C_DOUBLE_NAME: &str = "CDouble";

pub const IOSTATE_NAME: &str = "IOState";
pub const BOOL_NAME: &str = "Bool";
pub const ARRAY_NAME: &str = "Array";
pub const LAZY_NAME: &str = "Lazy";
pub const FUNCTOR_NAME: &str = "Functor";
pub const TUPLE_NAME: &str = "Tuple";
pub const DESTRUCTOR_OBJECT_NAME: &str = "Destructor";
pub const DESTRUCTOR_OBJECT_VALUE_FIELD_IDX: u32 = 0;
pub const DESTRUCTOR_OBJECT_DTOR_FIELD_IDX: u32 = 1;
pub const STRING_NAME: &str = "String";
pub const MONAD_NAME: &str = "Monad";
pub const MONAD_BIND_NAME: &str = "bind";
pub const COMPOSE_FUNCTION_NAME: &str = "compose";
pub const MAIN_FUNCTION_NAME: &str = "main";
pub const MAIN_MODULE_NAME: &str = "Main";
pub const TEST_FUNCTION_NAME: &str = "test";
pub const TEST_MODULE_NAME: &str = "Test";
pub const BOXED_TRAIT_NAME: &str = "Boxed";
pub const WITH_RETAINED_NAME: &str = "with_retained";

// Array methods.
pub const ARRAY_GETTER_FUNCTION_NAME: &str = "@";

// Structure methods.
pub const STRUCT_GETTER_SYMBOL: &str = "@";
pub const STRUCT_SETTER_SYMBOL: &str = "set_";
pub const STRUCT_MODIFIER_SYMBOL: &str = "mod_";
pub const STRUCT_ACT_SYMBOL: &str = "act_";

// Names used by compiler.
pub const FUNPTR_NAME: &str = "#FunPtr";
pub const DYNAMIC_OBJECT_NAME: &str = "#DynamicObject";
pub const PARAM_NAME: &str = "#param";
pub const EVAL_VAR_NAME: &str = "#eval_var";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "#";
pub const STRUCT_PUNCH_SYMBOL: &str = "#punch_";
pub const STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL: &str = "#punch_fu_";
pub const STRUCT_PLUG_IN_SYMBOL: &str = "#plug_in_";
pub const STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL: &str = "#plug_in_fu_";
pub const PUNCHED_TYPE_SYMBOL: &str = "#PunchedAt";
pub const CAP_NAME: &str = "#CAP";
pub const DECAP_NAME: &str = "#decap";

// pub const LOOP_RESULT_CONTINUE_IDX: usize = 0;

// Struct layout constants.
pub const CONTROL_BLOCK_IDX: u32 = 0;
pub const BOXED_TYPE_DATA_IDX: u32 = CONTROL_BLOCK_IDX + 1;
pub const UNION_TAG_IDX: u32 = 0; // Should be added to `BOXED_TYPE_DATA_IDX` if the union is boxed.
pub const UNION_DATA_IDX: u32 = UNION_TAG_IDX + 1;
pub const CLOSURE_FUNPTR_IDX: u32 = 0;
pub const CLOSURE_CAPTURE_IDX: u32 = CLOSURE_FUNPTR_IDX + 1;
pub const ARRAY_LEN_IDX: u32 = CONTROL_BLOCK_IDX + 1;
pub const ARRAY_CAP_IDX: u32 = ARRAY_LEN_IDX + 1;
pub const ARRAY_BUF_IDX: u32 = ARRAY_CAP_IDX + 1;
pub const DYNAMIC_OBJ_TRAVARSER_IDX: u32 = CONTROL_BLOCK_IDX + 1;
pub const DYNAMIC_OBJ_CAP_IDX: u32 = DYNAMIC_OBJ_TRAVARSER_IDX + 1;

// REFCNT_STATE_* values are stored to a field of the control block of each boxed object.
pub const REFCNT_STATE_LOCAL: u8 = 0; // This is local object in the sense that it is not shared with other threads but should be released since it is not global.
pub const REFCNT_STATE_THREADED: u8 = 1; // This object is shared between multiple threads and should be released or retained atomically.
pub const REFCNT_STATE_GLOBAL: u8 = 2; // This is global object and should not be released or retained.

pub const CTRL_BLK_REFCNT_IDX: u32 = 0;
pub const CTRL_BLK_REFCNT_STATE_IDX: u32 = 1;

// Paths
pub const DOT_FIXLANG: &str = ".fixlang";
pub const RUN_PATH: &str = ".fixlang/run";
pub const TYPE_CHECK_CACHE_PATH: &str = ".fixlang/cache/typecheck";
pub const INTERMEDIATE_PATH: &str = ".fixlang/intermediate";
pub const COMPILATION_UNITS_PATH: &str = ".fixlang/intermediate/units";
pub const TEMPORARY_SRC_PATH: &str = ".fixlang/tmp/src";
pub const CHECK_C_TYPES_PATH: &str = ".fixlang/check_c_types";
pub const C_TYPES_JSON_PATH: &str = ".fixlang/c_types.json";
#[allow(unused)]
pub const COMPILER_TEST_WORKING_PATH: &str = ".fixlang/compiler_test";
pub const LOG_FILE_PATH: &str = ".fixlang/fix.log";
pub const PROJECT_FILE_PATH: &str = "fixproj.toml";
pub const SAMPLE_MAIN_FILE_PATH: &str = "main.fix";
pub const SAMPLE_TEST_FILE_PATH: &str = "test.fix";
pub const LOCK_FILE_PATH: &str = "fixdeps.lock";
pub const EXTERNAL_PROJ_INSTALL_PATH: &str = ".fixlang/deps";
pub const FIX_CONFIG_FILE_NAME: &str = ".fixconfig.toml";

// Urls
pub const DEFAULT_REGISTRY: &str =
    "https://raw.githubusercontent.com/tttmmmyyyy/fixlang-registry/refs/heads/main/registry.toml";

// Optimization levels
pub const OPTIMIZATION_LEVEL_NONE: &str = "none";
pub const OPTIMIZATION_LEVEL_BASIC: &str = "basic";
pub const OPTIMIZATION_LEVEL_MAX: &str = "max";
pub const OPTIMIZATION_LEVEL_EXPERIMENTAL: &str = "experimental";

// Format of stdout of preliminary build commands.
pub const PRELIMINARY_BUILD_LD_FLAGS: &str = "fix.ld_flags=";

// Messages
pub const TRY_FIX_RESOLVE: &str = "Try `fix deps update` to update the lock file.";

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
    // pub fn runtime_function(&self) -> &str {
    //     match self.0 {
    //         TRAVERSER_WORK_RELEASE => RUNTIME_RELEASE_BOXED_OBJECT,
    //         TRAVERSER_WORK_MARK_GLOBAL => RUNTIME_MARK_GLOBAL_BOXED_OBJECT,
    //         TRAVERSER_WORK_MARK_THREADED => RUNTIME_MARK_THREADED_BOXED_OBJECT,
    //         _ => unreachable!(),
    //     }
    // }
}
pub const TRAVERSER_WORK_RELEASE: u32 = 0;
pub const TRAVERSER_WORK_MARK_GLOBAL: u32 = 1;
pub const TRAVERSER_WORK_MARK_THREADED: u32 = 2;

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
// The max size of tuples which are defined in any program.
// Any bigger tuples are defined on demand.
pub const TUPLE_SIZE_BASE: u32 = 3;
// Is tuple unboxed?
pub const TUPLE_UNBOX: bool = true;

// The type in LLVM corresponding to `pthread_once_t` of this system.
pub fn pthread_once_init_flag_type<'c>(ctx: &'c Context) -> IntType<'c> {
    // TODO: we should compile C program including "sizeof(pthread_once_t)" and run it to get the correct size.
    if std::env::consts::OS == "macos" {
        ctx.i128_type()
    } else {
        ctx.i32_type()
    }
}

// The value of `PTHREAD_ONCE_INIT` of this system.
pub fn pthread_once_init_flag_value<'c>(ctx: &'c Context) -> IntValue<'c> {
    pthread_once_init_flag_type(ctx).const_zero()
}

pub const GLOBAL_VAR_NAME_ARGC: &str = "fixruntime_argc";
pub const GLOBAL_VAR_NAME_ARGV: &str = "fixruntime_argv";

pub const DEFAULT_COMPILATION_UNIT_MAX_SIZE: usize = 128;
pub const DEFAULT_COMPILATION_UNIT_MAX_SIZE_STR: &str = "128";

pub fn chars_allowed_in_identifiers() -> String {
    // If you add a new character, please also update `name_char` in `grammar.pest`.
    let mut chars = String::new();
    chars.push_str("abcdefghijklmnopqrstuvwxyz");
    chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    chars.push_str("0123456789");
    chars.push_str("_@");
    chars
}

// Error codes.
pub const ERR_UNKNOWN_NAME: &str = "unknown-name";
pub const ERR_NO_VALUE_MATCH: &str = "no-value-match";
