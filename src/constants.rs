pub const STD_NAME: &str = "Std";
pub const IO_NAME: &str = "IO";
pub const PTR_NAME: &str = "Ptr";
pub const U8_NAME: &str = "U8";
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
pub const GETTER_SYMBOL: &str = "@";
pub const SETTER_SYMBOL: &str = "set_";

pub const CAP_NAME: &str = "#CAP";

pub const LOOP_RESULT_CONTINUE_IDX: usize = 0;

pub const CLOSURE_FUNPTR_IDX: u32 = 0;
pub const CLOSURE_CAPTURE_IDX: u32 = CLOSURE_FUNPTR_IDX + 1;
pub const ARRAY_LEN_IDX: u32 = 1/* ControlBlock */;
pub const ARRAY_CAP_IDX: u32 = ARRAY_LEN_IDX + 1;
pub const ARRAY_BUF_IDX: u32 = ARRAY_CAP_IDX + 1;
pub const DYNAMIC_OBJ_DTOR_IDX: u32 = 1/* ControlBlock */;
pub const DYNAMIC_OBJ_CAP_IDX: u32 = DYNAMIC_OBJ_DTOR_IDX + 1;

pub const TYPE_CHECK_CACHE_PATH: &str = ".fixlang/type_check_cache";
pub const DOT_FIXLANG: &str = ".fixlang";
pub const INTERMEDIATE_PATH: &str = ".fixlang/intermediate";
pub const SEARCH_DYNAMIC_LIBRARY_TEMP_FILE: &str = ".fixlang/search_dl_temp";

pub const STANDARD_LIBRARIES: &[(&str, &str, &str, Option<&str>)] = &[
    ("Debug", include_str!("./fix/debug.fix"), "debug", None),
    ("Hash", include_str!("./fix/hash.fix"), "hash", None),
    (
        "HashMap",
        include_str!("./fix/hashmap.fix"),
        "hashmap",
        None,
    ),
    (
        "HashSet",
        include_str!("./fix/hashset.fix"),
        "hashset",
        None,
    ),
    ("Math", include_str!("./fix/math.fix"), "math", Some("m")),
    ("Time", include_str!("./fix/time.fix"), "time.fix", None),
    (
        "Character",
        include_str!("./fix/character.fix"),
        "character",
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
