use inkwell::{context::Context, types::IntType, values::IntValue};
use std::env;

pub const NAMESPACE_SEPARATOR: &str = "::";
pub const MODULE_SEPARATOR: &str = ".";

// PROOF: P1, P2, P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
pub const STD_NAME: &str = "Std";
pub const FFI_NAME: &str = "FFI";
pub const IO_NAME: &str = "IO";
/// The field of `Std::IO` holding the action, a function taking an `IOState` to the state after the
/// action and the action's result.
pub const IO_DATA_NAME: &str = "runner";
/// The `Std` value that puts the values reachable from a value into multi-threaded mode.
pub const MARK_THREADED_NAME: &str = "mark_threaded";
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
// PROOF: P3, P4, P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
pub const BOOL_NAME: &str = "Bool";
// PROOF: P3, P4, P5, P6, P7, P7a, P7d, P7e (dev-docs/proof/rc_ir/borrow-cancel)
pub const ARRAY_NAME: &str = "Array";
pub const PUNCHED_ARRAY_NAME: &str = "PunchedArray";
pub const LAZY_NAME: &str = "Lazy";
pub const FUNCTOR_NAME: &str = "Functor";
// PROOF: P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
pub const TUPLE_NAME: &str = "Tuple";
pub const DESTRUCTOR_NAME: &str = "Destructor";
pub const DESTRUCTOR_OBJECT_VALUE_FIELD_IDX: u32 = 0;
pub const DESTRUCTOR_OBJECT_DTOR_FIELD_IDX: u32 = 1;
pub const STRING_NAME: &str = "String";
pub const MONAD_NAME: &str = "Monad";
pub const IDENTITY_NAME: &str = "Identity";
pub const CONST_NAME: &str = "Const";
pub const MONAD_BIND_NAME: &str = "bind";
pub const COMPOSE_FUNCTION_NAME: &str = "compose";
/// The name of the Fix value a program starts from: `Main::main`, of type `IO ()`. It holds the
/// same string as `C_ENTRY_POINT_NAME`, which names the C function of the object file.
pub const MAIN_FUNCTION_NAME: &str = "main";
/// The name of the module whose namespace holds the entry point `Main::main`.
pub const MAIN_MODULE_NAME: &str = "Main";
/// The name of the entry point in the object file: the function the C runtime calls once it has set
/// the process up, which the compiler generates to run the program's `IO` action.
pub const C_ENTRY_POINT_NAME: &str = "main";
/// The name of the Fix value `fix test` starts from: `Test::test`, of type `IO ()`.
pub const TEST_FUNCTION_NAME: &str = "test";
/// The name of the module whose namespace holds the test entry point `Test::test`.
pub const TEST_MODULE_NAME: &str = "Test";
pub const BOXED_TRAIT_NAME: &str = "Boxed";
pub const WITH_RETAINED_NAME: &str = "with_retained";
#[allow(unused)]
pub const ARRAY_ACT_NAME: &str = "act";
pub const BUILTIN_ACT_NAME: &str = "_unsafe_act_bounds_unchecked";
pub const INDEXABLE_TRAIT_NAME: &str = "Indexable";
pub const INDEXABLE_TRAIT_ACT_NAME: &str = "act_at_index";

// Array methods.
pub const ARRAY_UNSAFE_GET_BOUNDS_UNCHECKED: &str = "_unsafe_get_bounds_unchecked";
pub const ARRAY_CHECK_RANGE: &str = "_check_range";
pub const ARRAY_CHECK_SIZE: &str = "_check_size";
pub const ARRAY_UNSAFE_EMPTY_NAME: &str = "_unsafe_empty_capacity_unchecked";

// Structure methods.
/// The head of the name of a struct field's getter, which reads the field out of a value: the
/// getter of the field `x` is named `@x`, in the namespace of the struct the field belongs to.
pub const STRUCT_GETTER_SYMBOL: &str = "@";
pub const STRUCT_SETTER_SYMBOL: &str = "set_";
pub const STRUCT_MODIFIER_SYMBOL: &str = "mod_";
pub const STRUCT_ACT_SYMBOL: &str = "act_";

// The spelling of a Fix name in an object file's symbol table.
/// The character an object file's symbol table cannot hold. ELF reads it in a symbol name as the
/// separator of `symbol@version`, so GNU ld refuses a symbol carrying one while it builds the
/// dynamic symbol table of a shared library. A Fix name carries one wherever it names a field
/// getter (`STRUCT_GETTER_SYMBOL`).
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub const SYMBOL_VERSION_SEPARATOR: &str = "@";

/// What `SYMBOL_VERSION_SEPARATOR` is written as in a symbol table. `$` is legal there, and no Fix
/// name contains it, so the two spellings stand for the same set of names.
pub const SYMBOL_VERSION_SEPARATOR_SUBSTITUTE: &str = "$";

// Union methods.
pub const UNION_AS_SYMBOL: &str = "as_";
pub const UNION_IS_SYMBOL: &str = "is_";
pub const UNION_MOD_SYMBOL: &str = "mod_";

// Names used by compiler.
// PROOF: P1, P2, P3, P4, P5, P6, P7, P7a, P7d, P7e, P26 (dev-docs/proof/rc_ir/borrow-cancel)
pub const FUNPTR_NAME: &str = "#FunPtr";
pub const DYNAMIC_OBJECT_NAME: &str = "#DynamicObject";
// The internal boxed type holding an array's refcount and raw element buffer. Like `#DynamicObject`,
// its `#` prefix makes it un-nameable in source, so it cannot leak out of `Array`'s interface.
pub const ARRAY_STORAGE_NAME: &str = "#ArrayStorage";
pub const PARAM_NAME: &str = "#param";
pub const INSTANCIATED_NAME_SEPARATOR: &str = "#";
pub const STRUCT_PUNCH_SYMBOL: &str = "#punch_";
pub const STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL: &str = "#punch_fu_";
pub const STRUCT_PLUG_IN_SYMBOL: &str = "#plug_in_";
pub const STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL: &str = "#plug_in_fu_";
pub const PUNCHED_TYPE_SYMBOL: &str = "#PunchedAt";
/// The name standing for the captured environment of a lambda. Every lambda binds it implicitly, so
/// it is the one local name that the free variables of an expression leave out.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub const CAP_NAME: &str = "#CAP";
/// The name of the parameter through which a decaptured lambda receives its capture list.
pub const CLOSURE_CAP_NAME: &str = "#closure_cap";
/// The suffix, followed by a counter, of the global function a decaptured lambda becomes.
pub const CLOSURE_LAM_SUFFIX: &str = "#closure_lam";
/// The suffix, followed by a hash of which argument received which decaptured lambda, of a
/// function specialized on the lambdas passed to it.
pub const CLOSURE_SPEC_SUFFIX: &str = "#closure_spec";
/// The suffix of the local binding holding the call of a decaptured lambda, which an inline-LLVM
/// expression reads in place of the variable that held the lambda's capture list.
pub const CLOSURE_CALL_LAM_SUFFIX: &str = "#closure_call_lam";
/// The prefix of the type constructor naming a capture list that closure specialization builds. A
/// parameter of this type is a function whose identity the receiving body knows.
pub const CAP_LIST_PREFIX: &str = "#CapList";
/// The prefix of the name `collapse_constructions` binds a field value to, so that a reader of the
/// struct is given a name rather than the expression that produced the value.
pub const BOUND_FIELD_PREFIX: &str = "#field";
/// The suffix, followed by a counter, of the global function taking one argument per field of a
/// struct argument of the function it is named after.
pub const SPLIT_ARG_SUFFIX: &str = "#split_arg";
/// The prefix of the type variable standing for the concrete type behind an opaque type. The rest of
/// the name is the name of the opaque type's TyCon, which the type checker reads back off it.
pub const WRAP_OPAQUE_TYVAR_PREFIX: &str = "#wrap_opaque_tyvar_";
/// The name of the global generated in the namespace of each value whose signature has an opaque
/// type. It wraps the value's definition so that type inference records the concrete type behind the
/// opaque type; instantiation then removes the applications of it.
pub const WRAP_OPAQUE_FUNC_NAME: &str = "#wrap_opaque";

// Struct layout constants.
/// The index of the control block, the field a boxed object's layout begins with.
pub const CONTROL_BLOCK_IDX: u32 = 0;
/// The index at which a boxed object's own fields begin, after its control block.
pub const BOXED_TYPE_DATA_IDX: u32 = CONTROL_BLOCK_IDX + 1;
/// The index of a union's tag among the union's own fields. The fields of a boxed union begin at
/// `BOXED_TYPE_DATA_IDX`, which `struct_field_idx` adds.
pub const UNION_TAG_IDX: u32 = 0;
/// The index of a union's payload buffer among the union's own fields, after the tag.
pub const UNION_DATA_IDX: u32 = UNION_TAG_IDX + 1;
/// The width of a union's tag, which holds the index of the variant the value was created as.
pub const UNION_TAG_BITS: u32 = 8;
/// How many variants a union may declare: the indices a tag of `UNION_TAG_BITS` bits tells apart.
pub const MAX_UNION_VARIANTS: usize = 1 << UNION_TAG_BITS;
/// The index of the function pointer among a closure's fields.
pub const CLOSURE_FUNPTR_IDX: u32 = 0;
/// The index, among a closure's fields, of the pointer to the captured values the function is
/// called with.
// PROOF: P1, P2, P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub const CLOSURE_CAPTURE_IDX: u32 = CLOSURE_FUNPTR_IDX + 1;
/// How many fields a closure has: the function pointer and the capture.
// PROOF: P1, P2, P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
pub const CLOSURE_FIELD_COUNT: usize = 2;
// Field layout of the unbox `Array` value: a `SubObject` pointer to the `#ArrayStorage`, then the
// register-resident size and capacity.
pub const ARRAY_STORAGE_IDX: u32 = 0;
pub const ARRAY_SIZE_IDX: u32 = ARRAY_STORAGE_IDX + 1;
pub const ARRAY_CAP_IDX: u32 = ARRAY_SIZE_IDX + 1;

/// The index of the field of the unbox `Std::PunchedArray` value that holds the array.
pub const PUNCHED_ARRAY_ARRAY_IDX: u32 = 0;
/// The index of the field of the unbox `Std::PunchedArray` value that holds the index of the slot
/// whose element was moved out of the array.
pub const PUNCHED_ARRAY_HOLE_IDX: u32 = PUNCHED_ARRAY_ARRAY_IDX + 1;

// Field layout of the internal `#ArrayStorage` object: a control block and the raw element buffer,
// with no length or capacity (those live in the owning `Array` value).
pub const STORAGE_CTRL_IDX: u32 = CONTROL_BLOCK_IDX;
pub const STORAGE_BUF_IDX: u32 = STORAGE_CTRL_IDX + 1;

// The boundary a large array's element buffer starts on. A vectorized loop moves 32 bytes per
// iteration, and an access that crosses a 64-byte cache line costs about 1.75 times one that does
// not, so a buffer off this boundary makes every second vector access straddle a line.
pub const ARRAY_BUF_ALIGNMENT: u64 = 32;

// The bytes an `#ArrayStorage` allocation carries on top of the object, so that the object can be
// placed off the base of its block and land with its element buffer on `ARRAY_BUF_ALIGNMENT`. The
// object is placed by less than the alignment, so this is the widest that distance can be.
pub const ARRAY_STORAGE_ALLOC_SLACK: u64 = ARRAY_BUF_ALIGNMENT - 1;

// The `#ArrayStorage` allocation size, in bytes, from which the element buffer is worth aligning.
// Below it the loop over the elements is too short for the alignment to pay for the bytes the
// alignment costs, and such arrays are numerous enough that those bytes show up on their own.
pub const ARRAY_ALIGNED_ALLOC_THRESHOLD: u64 = 256;

// The default for `Configuration::max_split_scalars`: the most scalars an unboxed value is split
// into and carried as separate LLVM values, above which it stays one aggregate wherever it is
// carried.
//
// A scalar here is one LLVM value: a struct contributes the scalars of its fields, and everything
// else is one, an array included however many elements it holds. That is the quantity this limit
// exists to bound -- the LLVM values a Fix value occupies, which is what a union's payload buffer
// costs whatever its width. `return_abi.rs`'s `demand_of` counts the same array element by element,
// because it answers the other question: how many registers the return lowering asks for, and that
// lowering flattens an array into its elements.
//
// Splitting is what keeps a loop-carried field visible to LLVM (see `Generator::type_parts`), and
// the widest type in the benchmark suite holds 21 scalars, the widest across the minilib libraries
// 37, so this is well above what real code splits. Above it the count is what matters: a value of
// 4096 scalars costs one LLVM value per scalar at every function boundary it crosses, and the
// backend's per-block work grows faster than the count.
pub const MAX_SPLIT_SCALARS: usize = 128;

// The variant tags of `Std::Bool = unbox union { _false : (), _true : () }`.
pub const BOOL_FALSE_TAG: usize = 0;
pub const BOOL_TRUE_TAG: usize = 1;
// The fields of the result of `Std::unsafe_is_unique : a -> (Bool, a)`.
pub const IS_UNIQUE_FLAG_FIELD: usize = 0;
pub const IS_UNIQUE_VALUE_FIELD: usize = 1;

// Number of array elements claimed by array debug info. The element count of an array
// is only known at run time, which debug info cannot express here, so array debug types
// claim this fixed number of elements, and their byte sizes cover the claimed elements
// so that debuggers display them with values read from the target. See the array branch
// of `ObjectFieldType::to_debug_type` in object.rs and the debugging section of
// Document.md.
pub const DEBUG_ARRAY_ASSUMED_LEN: u64 = 100;

// Field layout of the `#DynamicObject` a closure keeps its captured values in: a control block, the
// traverse function that drives the captures' lifetimes, then the captures themselves. The captures
// vary with the closure, which is why the object carries its own traverse function.
pub const DYNAMIC_OBJ_TRAVARSER_IDX: u32 = CONTROL_BLOCK_IDX + 1;
pub const DYNAMIC_OBJ_CAP_IDX: u32 = DYNAMIC_OBJ_TRAVARSER_IDX + 1;

/// How the reference count of a boxed object is maintained, stored in a byte of its control block.
///
/// The values ascend with how far a state exempts an object from reference counting, and code
/// generation reads that order: `LOCAL` is counted, `THREADED` is counted atomically, and `GLOBAL`
/// is not counted at all, so a state covers every state below it. Marking asks exactly this
/// question — an object whose state already reaches the mark being made has nothing left to
/// receive, and neither has anything it owns.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RefcntState(u8);

// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
impl RefcntState {
    /// Reachable from one thread: the count is updated without atomics, and the object is freed
    /// when it reaches zero.
    pub const LOCAL: RefcntState = RefcntState(0);
    /// Reachable from several threads at once: the count is updated atomically.
    pub const THREADED: RefcntState = RefcntState(1);
    /// Exempt from counting: the object is neither retained, released nor freed, and lives for as
    /// long as the program does.
    pub const GLOBAL: RefcntState = RefcntState(2);

    /// The byte stored in the control block.
    pub fn value(self) -> u8 {
        self.0
    }
}

/// The order the states are compared in, which is what lets a comparison against one state answer
/// for the states beyond it.
const _: () = assert!(
    RefcntState::LOCAL.0 < RefcntState::THREADED.0
        && RefcntState::THREADED.0 < RefcntState::GLOBAL.0
);

// Field layout of the control block every boxed object begins with: the reference count, then the
// `RefcntState` saying how that count is to be maintained.
pub const CTRL_BLK_REFCNT_IDX: u32 = 0;
pub const CTRL_BLK_REFCNT_STATE_IDX: u32 = 1;
// How far the object sits above the base of its allocation. Nonzero where the object was placed
// off the base to put a buffer following it on a boundary, which `#ArrayStorage` does for its
// elements; freeing or reallocating the object steps back by it to recover the block. It occupies
// a byte of the control block's tail padding, so the control block keeps its size.
pub const CTRL_BLK_ALLOC_OFFSET_IDX: u32 = 2;

/// The name of the LLVM module a compilation unit's code is generated into, before the unit's hash.
/// `--emit-llvm` names the file it writes after the module.
pub const UNIT_MODULE_NAME_PREFIX: &str = "Module-";

// Paths
pub const DOT_FIXLANG: &str = ".fixlang";
pub const RUN_PATH: &str = ".fixlang/run";
pub const TYPE_CHECK_CACHE_PATH: &str = ".fixlang/cache/typecheck";
pub const UNITS_CACHE_PATH: &str = ".fixlang/cache/units";
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
pub const LOCK_FILE_TEST_PATH: &str = "fixdeps.test.lock";
pub const LOCK_FILE_LSP_PATH: &str = ".fixlang/fixdeps.lsp.lock";
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
pub const TRY_FIX_DEPS_UPDATE: &str = "Try `fix deps update` to update the lock file.";
pub const TRY_FIX_DEPS_UPDATE_TEST: &str =
    "Try `fix deps update --test` to update the test dependencies lock file.";

/// The work a traverser function performs on the boxed objects an object owns. The wrapped value is
/// one of the `TRAVERSER_WORK_*` codes, and is what the generated traverser receives as its work
/// argument when the work is chosen at run time.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TraverserWorkType(pub u32);
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
impl TraverserWorkType {
    /// Drop one reference to each object reached, freeing an object whose count falls to zero.
    pub fn release() -> Self {
        Self(TRAVERSER_WORK_RELEASE)
    }
    /// Put each object reached into the global reference counting state, in which retains and
    /// releases leave it alone.
    pub fn mark_global() -> Self {
        Self(TRAVERSER_WORK_MARK_GLOBAL)
    }
    /// Put each object reached into the threaded reference counting state, in which retains and
    /// releases update its counter atomically.
    pub fn mark_threaded() -> Self {
        Self(TRAVERSER_WORK_MARK_THREADED)
    }
}
pub const TRAVERSER_WORK_RELEASE: u32 = 0;
pub const TRAVERSER_WORK_MARK_GLOBAL: u32 = 1;
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
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
// PROOF: P1, P2, P5, P6, P7, P7a, P7d, P7e (dev-docs/proof/rc_ir/borrow-cancel)
pub const FUNPTR_ARGS_MAX: u32 = 100;
// The max size of tuples which are defined in any program.
// Any bigger tuples are defined on demand.
pub const TUPLE_SIZE_BASE: u32 = 3;
// Is tuple unboxed?
// PROOF: D/A, P5, P6, P7 (dev-docs/proof/rc_ir/borrow-cancel)
pub const TUPLE_UNBOX: bool = true;

// The type in LLVM corresponding to `pthread_once_t` of this system.
pub fn pthread_once_init_flag_type<'c>(ctx: &'c Context) -> IntType<'c> {
    // TODO: we should compile C program including "sizeof(pthread_once_t)" and run it to get the correct size.
    if env::consts::OS == "macos" {
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

pub const DEFAULT_COMPILATION_UNIT_SIZE: usize = 128;
pub const DEFAULT_COMPILATION_UNIT_SIZE_STR: &str = "128";

/// The `cu_size` that puts the whole program in one compilation unit, which `--cu-size inf` asks
/// for.
///
/// A boundary falls where the hash of an entry's name lands in a band one `cu_size` wide
/// (`misc::split_at_name_boundaries`), so a size this large leaves one unit for all but two of the
/// 2^64 hashes a name can take. `divide_program::divide_into_units` reads the value itself rather
/// than the band, so a program divided this way is one unit whatever its names hash to.
pub const WHOLE_PROGRAM_IN_ONE_UNIT: usize = usize::MAX;

/// What `--cu-size` is given to ask for `WHOLE_PROGRAM_IN_ONE_UNIT`.
pub const WHOLE_PROGRAM_IN_ONE_UNIT_STR: &str = "inf";

/// Stack size, in bytes, of each compiler worker thread. Parallel type checking and per-unit code
/// generation recurse over the user program's expression tree, whose nesting depth (deeply nested
/// `let` or `;;` chains) is unbounded, so a worker needs far more stack than the default thread
/// stack gives. The stack is reserved as virtual address space and backed by physical memory only
/// for the pages a thread actually touches, so this large reservation costs real memory only in
/// proportion to the recursion depth reached.
pub const COMPILER_THREAD_STACK_SIZE: usize = 256 * 1024 * 1024;

/// The characters an identifier in a Fix source file may be built from, as one string.
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
pub const ERR_AMBIGUOUS_NAME: &str = "ambiguous-name";
pub const ERR_NO_VALUE_MATCH: &str = "no-value-match";
pub const ERR_MISSING_TRAIT_IMPL: &str = "missing-trait-impl";
pub const ERR_MISSING_STRUCT_FIELD: &str = "missing-struct-field";
/// Diagnostic code emitted for each `Std::#hole` reference left in the
/// program after elaboration.
pub const ERR_HOLE: &str = "missing-expression";

/// Internal placeholder value generated by the parser when an expression
/// position is left empty (e.g. `let x = 10; ` with no body). Defined as
/// `Std::#hole : a`; the leading `#` keeps it disjoint from any name a
/// user can write (the `name` grammar rule does not accept `#`).
pub const HOLE_NAME: &str = "#hole";

/// Prefix of the local names the parser generates for `_` wildcard
/// patterns (e.g. `#wildcard0`). Each `_` binds a distinct name so that
/// multiple `_`s in one pattern do not collide; the leading `#` keeps
/// these names disjoint from any name a user can write (the `name`
/// grammar rule does not accept `#`). Pattern display renders such a
/// binder back as `_`.
pub const PATTERN_WILDCARD_VAR_PREFIX: &str = "#wildcard";

/// Prefix of the type-variable names the parser generates for `_` type
/// wildcards (e.g. `#typewildcard0`). Each `_` in a type annotation gets a
/// distinct name so that, for example, the two wildcards in `(_, _)` stay
/// independent; the leading `#` keeps these names disjoint from any type
/// variable a user can write. `validate_type_annotation` replaces each
/// such name with a fresh inference variable, so it never surfaces to the
/// user.
pub const TYPE_WILDCARD_VAR_PREFIX: &str = "#typewildcard";

// Formatting
pub const FORMAT_LINE_LIMIT: usize = 100;
