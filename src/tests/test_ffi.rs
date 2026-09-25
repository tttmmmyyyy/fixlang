use crate::{
    ast::{name::FullName, program::TypeEnv, types::tycon},
    build::build_object_files::get_target_machine,
    configuration::{Configuration, OutputFileType},
    constants::{COMPILER_TEST_WORKING_PATH, C_ENTRY_POINT_NAME, I8_NAME, STD_NAME, U16_NAME},
    elaboration::elaborate_via_config,
    error::panic_if_err,
    ffi::{c_abi_extends_narrow_integers, CSignature},
    fixstd::runtime::{
        build_runtime, compiler_defined_c_function_reason, BuildMode, RUNTIME_ABORT,
        RUNTIME_GET_ARGC,
    },
    generator::{enum_attribute_kind_id, Generator},
    misc::function_name,
    tests::test_util::{
        emitted_llvm_ir, fix_command, standalone_generator, test_source, test_source_fail,
        test_source_with_c, EmittedIr,
    },
};
use inkwell::{
    attributes::AttributeLoc,
    context::Context,
    targets::{TargetMachine, TargetTriple},
};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

// An exported function exchanges values with C through the C ABI, and the wrapper the compiler
// generates for it passes every argument and the result by value in the LLVM type Fix uses
// internally. That matches the C ABI for a scalar and for a pointer, and for nothing else: the C
// ABI classifies a structure by its size and by the class of each of its eightbytes
// (System V AMD64) or by whether it is a homogeneous floating-point aggregate (AAPCS64), while
// LLVM assigns a register to each element. The tests below pin both halves of that boundary — the
// types an export accepts, and the diagnostic the rest receive — and the way to exchange an
// aggregate anyway, which is a pointer to a region the C side owns.

/// Every scalar type an exported function may exchange, in both directions, checked against the
/// C type it is documented to correspond to.
#[test]
pub fn test_export_scalar_types() {
    let source = r##"
        module Main;

        add_i8 : I8 -> I8 -> I8;
        add_i8 = |x, y| x + y;
        FFI_EXPORT[add_i8, c_add_i8];

        add_u8 : U8 -> U8 -> U8;
        add_u8 = |x, y| x + y;
        FFI_EXPORT[add_u8, c_add_u8];

        add_i16 : I16 -> I16 -> I16;
        add_i16 = |x, y| x + y;
        FFI_EXPORT[add_i16, c_add_i16];

        add_u16 : U16 -> U16 -> U16;
        add_u16 = |x, y| x + y;
        FFI_EXPORT[add_u16, c_add_u16];

        add_i32 : I32 -> I32 -> I32;
        add_i32 = |x, y| x + y;
        FFI_EXPORT[add_i32, c_add_i32];

        add_u32 : U32 -> U32 -> U32;
        add_u32 = |x, y| x + y;
        FFI_EXPORT[add_u32, c_add_u32];

        add_i64 : I64 -> I64 -> I64;
        add_i64 = |x, y| x + y;
        FFI_EXPORT[add_i64, c_add_i64];

        add_u64 : U64 -> U64 -> U64;
        add_u64 = |x, y| x + y;
        FFI_EXPORT[add_u64, c_add_u64];

        add_f32 : F32 -> F32 -> F32;
        add_f32 = |x, y| x + y;
        FFI_EXPORT[add_f32, c_add_f32];

        add_f64 : F64 -> F64 -> F64;
        add_f64 = |x, y| x + y;
        FFI_EXPORT[add_f64, c_add_f64];

        // `Ptr` travels through Fix unchanged.
        echo_ptr : Ptr -> Ptr;
        echo_ptr = |p| p;
        FFI_EXPORT[echo_ptr, c_echo_ptr];

        // A mixture wide enough that the arguments cross both register classes.
        combine : I64 -> F64 -> I64 -> F64 -> F64;
        combine = |a, b, c, d| a.to_F64 + b + c.to_F64 * 2.0 + d * 3.0;
        FFI_EXPORT[combine, c_combine];

        main : IO ();
        main = (
            let res = FFI_CALL[CInt run_c()];
            assert_eq(|_|"C reported failure", res, 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        int8_t c_add_i8(int8_t x, int8_t y);
        uint8_t c_add_u8(uint8_t x, uint8_t y);
        int16_t c_add_i16(int16_t x, int16_t y);
        uint16_t c_add_u16(uint16_t x, uint16_t y);
        int32_t c_add_i32(int32_t x, int32_t y);
        uint32_t c_add_u32(uint32_t x, uint32_t y);
        int64_t c_add_i64(int64_t x, int64_t y);
        uint64_t c_add_u64(uint64_t x, uint64_t y);
        float c_add_f32(float x, float y);
        double c_add_f64(double x, double y);
        void *c_echo_ptr(void *p);
        double c_combine(int64_t a, double b, int64_t c, double d);

        int run_c() {
            if (c_add_i8(-100, 30) != -70) { return 1; }
            if (c_add_u8(200, 55) != 255) { return 1; }
            if (c_add_i16(30000, -1000) != 29000) { return 1; }
            if (c_add_u16(60000, 5000) != (uint16_t)65000) { return 1; }
            if (c_add_i32(2000000000, -1000000000) != 1000000000) { return 1; }
            if (c_add_u32(4000000000u, 100u) != 4000000100u) { return 1; }
            if (c_add_i64(1LL << 40, 7) != ((1LL << 40) + 7)) { return 1; }
            if (c_add_u64(1ULL << 63, 7) != ((1ULL << 63) + 7)) { return 1; }
            if (c_add_f32(0.5f, 0.25f) != 0.75f) { return 1; }
            if (c_add_f64(0.5, 0.25) != 0.75) { return 1; }

            int marker = 0;
            if (c_echo_ptr(&marker) != &marker) { return 1; }

            if (c_combine(1, 0.5, 2, 0.25) != 1.0 + 0.5 + 4.0 + 0.75) { return 1; }

            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// An integer narrower than 32 bits travels in the low bits of a register, and on a target whose
/// C ABI extends it at a call, `signext` / `zeroext` is how a signature says which side of the call
/// extends it. The program's exported functions and the C functions it calls carry the attribute
/// on the host's target exactly where `c_abi_extends_narrow_integers` says that ABI extends, which
/// this test reads off the emitted IR.
#[test]
pub fn test_narrow_integer_extension_attribute_follows_the_host_abi() {
    let source = r#"
        module Main;

        add_i8 : I8 -> I8 -> I8;
        add_i8 = |x, y| x + y;
        FFI_EXPORT[add_i8, c_add_i8];

        add_u16 : U16 -> U16 -> U16;
        add_u16 = |x, y| x + y;
        FFI_EXPORT[add_u16, c_add_u16];

        add_i64 : I64 -> I64 -> I64;
        add_i64 = |x, y| x + y;
        FFI_EXPORT[add_i64, c_add_i64];

        write_byte : Ptr -> U8 -> IO ();
        write_byte = |p, v| FFI_CALL_IO[() fixruntime_u8_to_bytes(Ptr, U8), p, v];
        FFI_EXPORT[write_byte, c_write_byte];

        main : IO ();
        main = pure();
    "#;
    let work_dir = PathBuf::from(format!(
        "{}/{}",
        COMPILER_TEST_WORKING_PATH,
        function_name!()
    ));
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir).unwrap();
    File::create(work_dir.join("main.fix"))
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();

    let output = fix_command()
        .args([
            "build",
            "-O",
            "none",
            "--emit-llvm",
            "--file",
            "main.fix",
            "--output",
            "prog",
        ])
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "`fix build --emit-llvm` failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // `-O none` compiles the program as several modules, and the wrappers are spread over them.
    let ir = emitted_llvm_ir(&work_dir, EmittedIr::BeforeOptimization);

    let host_triple = TargetMachine::get_default_triple();
    let (signext, zeroext) =
        if c_abi_extends_narrow_integers(&host_triple.as_str().to_string_lossy()) {
            (" signext", " zeroext")
        } else {
            ("", "")
        };
    for expected in [
        // Where the ABI extends, an exported function extends its narrow arguments and result, by
        // sign or by zero according to the Fix type.
        format!("define{signext} i8 @c_add_i8(i8{signext} %0, i8{signext} %1)"),
        format!("define{zeroext} i16 @c_add_u16(i16{zeroext} %0, i16{zeroext} %1)"),
        // A type that fills the register carries no attribute, and neither does a pointer.
        format!("define i64 @c_add_i64(i64 %0, i64 %1)"),
        format!("define void @c_write_byte(ptr %0, i8{zeroext} %1)"),
        // The same holds for the C functions Fix calls.
        format!("declare void @fixruntime_u8_to_bytes(ptr, i8{zeroext})"),
    ] {
        assert!(
            ir.contains(&expected),
            "emitted IR lacks `{}`:\n{}",
            expected,
            ir
        );
    }
}

/// Whether a declaration of a C function carries `signext` / `zeroext` on a narrow integer follows
/// the C ABI of the target the module is compiled for: x86-64 and Apple's AArch64 extend, and
/// AAPCS64 on Linux leaves the bits to the reader. Each module here is given a target's triple, so
/// a host of either architecture checks all three answers.
#[test]
pub fn test_narrow_integer_extension_follows_the_target_abi() {
    let config = Configuration::develop_mode();
    let std_tycon = |name: &str| tycon(FullName::from_strs(&[STD_NAME], name));
    let signature = CSignature {
        param_tys: vec![std_tycon(U16_NAME)],
        ret_tycon: std_tycon(I8_NAME),
        is_var_args: false,
    };
    for (triple, extends) in [
        ("x86_64-unknown-linux-gnu", true),
        ("arm64-apple-darwin23.0.0", true),
        ("aarch64-unknown-linux-gnu", false),
    ] {
        let context = Context::create();
        let target_machine = get_target_machine(config.get_llvm_opt_level(), &config);
        let module = Generator::create_module("abi_test", &context, &target_machine);
        module.set_triple(&TargetTriple::create(triple));
        let gc = standalone_generator(
            &context,
            &module,
            &target_machine,
            &config,
            TypeEnv::default(),
        );
        let func = signature.get_or_declare_in_module(&"c_narrow".to_string(), &gc);
        let result_extended = func
            .get_enum_attribute(AttributeLoc::Return, enum_attribute_kind_id("signext"))
            .is_some();
        let argument_extended = func
            .get_enum_attribute(AttributeLoc::Param(0), enum_attribute_kind_id("zeroext"))
            .is_some();
        assert_eq!(
            (result_extended, argument_extended),
            (extends, extends),
            "the extension attributes of `int8_t c_narrow(uint16_t)` on {}",
            triple
        );
    }
}

/// A C function's result narrower than the unit the ABI carries an integer in arrives in the low
/// bits of a register, and the value is what those bits hold. Each C function here returns the low
/// bits of a wider product, so on an ABI that leaves the bits above to the reader they hold the rest
/// of that product, and the number the Fix side reads is the narrow one.
#[test]
pub fn test_ffi_call_reads_a_narrow_result() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let n = (*IO::get_args).@size.c_int;
            assert_eq(|_|"I8", FFI_CALL[I8 c_narrow_i8(CInt), n * 1000.c_int].i64, -72);;
            assert_eq(|_|"U8", FFI_CALL[U8 c_narrow_u8(CInt), n * 1000.c_int].i64, 184);;
            assert_eq(|_|"I16", FFI_CALL[I16 c_narrow_i16(CInt), n * 100000.c_int].i64, -27680);;
            assert_eq(|_|"U16", FFI_CALL[U16 c_narrow_u16(CInt), n * 100000.c_int].i64, 37856);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        int8_t   c_narrow_i8(int n)  { return (int8_t)(n * 3); }
        uint8_t  c_narrow_u8(int n)  { return (uint8_t)(n * 3); }
        int16_t  c_narrow_i16(int n) { return (int16_t)(n * 3); }
        uint16_t c_narrow_u16(int n) { return (uint16_t)(n * 3); }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// An exported function's parameter narrower than the unit the ABI carries an integer in arrives in
/// the low bits of a register, and the value is what those bits hold. The C caller here passes the
/// low bits of a wider product, so on an ABI that leaves the bits above to the reader they hold the
/// rest of that product, and the number the Fix side reads is the narrow one.
#[test]
pub fn test_ffi_export_reads_a_narrow_argument() {
    let source = r##"
        module Main;

        widen_i8 : I8 -> I64;
        widen_i8 = |x| x.i64;
        FFI_EXPORT[widen_i8, c_widen_i8];

        widen_u8 : U8 -> I64;
        widen_u8 = |x| x.i64;
        FFI_EXPORT[widen_u8, c_widen_u8];

        widen_i16 : I16 -> I64;
        widen_i16 = |x| x.i64;
        FFI_EXPORT[widen_i16, c_widen_i16];

        widen_u16 : U16 -> I64;
        widen_u16 = |x| x.i64;
        FFI_EXPORT[widen_u16, c_widen_u16];

        main : IO ();
        main = (
            let n = (*IO::get_args).@size.c_int;
            assert_eq(|_|"C reported a wrong argument", FFI_CALL[CInt run_c(CInt), n], 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        int64_t c_widen_i8(int8_t x);
        int64_t c_widen_u8(uint8_t x);
        int64_t c_widen_i16(int16_t x);
        int64_t c_widen_u16(uint16_t x);

        int run_c(int n) {
            if (c_widen_i8((int8_t)(n * 3000)) != -72) { return 1; }
            if (c_widen_u8((uint8_t)(n * 3000)) != 184) { return 2; }
            if (c_widen_i16((int16_t)(n * 300000)) != -27680) { return 3; }
            if (c_widen_u16((uint16_t)(n * 300000)) != 37856) { return 4; }
            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// Apple's AArch64 extends under either name LLVM gives the architecture, AArch64 Linux extends
/// under neither name, also when its triple leaves the vendor out, and x86-64 extends on every
/// operating system.
#[test]
pub fn test_c_abi_extends_narrow_integers_under_each_spelling() {
    for (triple, extends) in [
        ("aarch64-apple-darwin", true),
        ("arm64-apple-darwin23.0.0", true),
        ("aarch64-unknown-linux-gnu", false),
        ("aarch64-linux-gnu", false),
        ("arm64-unknown-linux-gnu", false),
        ("x86_64-apple-darwin", true),
        ("x86_64-unknown-linux-gnu", true),
        ("x86_64-linux-gnu", true),
    ] {
        assert_eq!(c_abi_extends_narrow_integers(triple), extends, "{}", triple);
    }
}

/// A boxed value returned to the foreign language arrives as an opaque pointer carrying one
/// responsibility to release, and an exported function taking a boxed argument takes that
/// responsibility over. The two together are balanced, which memcheck checks.
#[test]
pub fn test_export_boxed_value() {
    let source = r##"
        module Main;

        type Resource = box struct { tag : I64 };

        make_resource : I64 -> IO Resource;
        make_resource = |tag| pure $ Resource { tag : tag };
        FFI_EXPORT[make_resource, c_make_resource];

        make_resource_pure : I64 -> Resource;
        make_resource_pure = |tag| Resource { tag : tag };
        FFI_EXPORT[make_resource_pure, c_make_resource_pure];

        read_resource : Resource -> I64;
        read_resource = |res| res.@tag;
        FFI_EXPORT[read_resource, c_read_resource];

        main : IO ();
        main = (
            let res = FFI_CALL[CInt run_c()];
            assert_eq(|_|"C reported failure", res, 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        void *c_make_resource(int64_t tag);
        void *c_make_resource_pure(int64_t tag);
        int64_t c_read_resource(void *res);

        int run_c() {
            if (c_read_resource(c_make_resource(42)) != 42) { return 1; }
            if (c_read_resource(c_make_resource_pure(7)) != 7) { return 1; }
            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// A C identifier is written in ASCII, so a letter outside it is refused even though it is a
/// letter. The grammar takes any character up to the `]`, so the name reaches this check.
#[test]
pub fn test_export_non_ascii_c_function_name_fails() {
    let source = r##"
        module Main;

        value : CInt;
        value = 42.c_int;
        FFI_EXPORT[value, café];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "The rest of the characters should be an ASCII letter, a digit or an underscore",
    );
}

/// The first character of a C identifier is admitted by a rule of its own — a digit is excluded
/// there and allowed everywhere after it — so a name opening with a letter outside ASCII is refused
/// by that rule rather than by the one covering the rest of the characters.
#[test]
pub fn test_export_non_ascii_first_character_fails() {
    let source = r##"
        module Main;

        value : CInt;
        value = 42.c_int;
        FFI_EXPORT[value, Δelta];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "The first character should be an ASCII letter or an underscore",
    );
}

/// A module holds one function under a name, so an export of a name the compiler writes a body under
/// is a second definition of one symbol and one of the two loses the name. Both kinds: the entry
/// point, and a name of the Fix runtime, whether the compiler writes its body into the module or
/// `runtime.c` carries it.
#[test]
pub fn test_export_taking_a_name_the_compiler_owns_fails() {
    for (c_function_name, reason) in [
        (C_ENTRY_POINT_NAME, "it is the entry point of the program"),
        (RUNTIME_ABORT, "belongs to the Fix runtime"),
        (RUNTIME_GET_ARGC, "belongs to the Fix runtime"),
    ] {
        let source = format!(
            r##"
                module Main;

                value : CInt;
                value = 42.c_int;
                FFI_EXPORT[value, {}];

                main : IO ();
                main = println("the entry point ran");
            "##,
            c_function_name
        );
        test_source_fail(&source, Configuration::develop_mode(), reason);
    }
}

/// The names of the C functions the compiler writes a body for when it builds the runtime under
/// `config`.
///
/// The runtime is the only thing built into the module read back here, so every function it holds
/// is one of the runtime's own. A function the compiler leaves to the C runtime carries no basic
/// block; one whose body the compiler writes carries at least one.
fn names_of_runtime_functions_with_bodies(
    config: &Configuration,
    type_env: &TypeEnv,
) -> Vec<String> {
    let context = Context::create();
    let target_machine = get_target_machine(config.get_llvm_opt_level(), config);
    let module = Generator::create_module("runtime_test", &context, &target_machine);
    let mut gc = standalone_generator(&context, &module, &target_machine, config, type_env.clone());
    build_runtime(&mut gc, BuildMode::Declare);
    build_runtime(&mut gc, BuildMode::Implement);

    module
        .get_functions()
        .filter(|func| func.count_basic_blocks() > 0)
        .map(|func| func.get_name().to_string_lossy().to_string())
        .collect()
}

/// Every function the compiler writes a body for is one an export is refused.
///
/// The names are read off a module the runtime was built into, so the list follows the runtime as
/// it grows. A runtime function added under a name outside the reserved prefix fails this test;
/// without it, a program exporting that name would have its own definition silently renamed.
#[test]
pub fn test_every_function_the_compiler_writes_a_body_for_is_refused_as_an_export() {
    let base = panic_if_err(Configuration::check_mode());
    let program = panic_if_err(elaborate_via_config(&base));
    let type_env = program.type_env().clone();
    // Multi-threading decides which runtime functions a build emits, so both settings are read.
    for threaded in [false, true] {
        let mut config = base.clone();
        config.threaded = threaded;
        let names_with_bodies = names_of_runtime_functions_with_bodies(&config, &type_env);
        assert!(
            !names_with_bodies.is_empty(),
            "building the runtime writes the body of at least one function, so that this test has \
             a name to read",
        );
        for name in names_with_bodies {
            assert!(
                compiler_defined_c_function_reason(&name, OutputFileType::Executable).is_some(),
                "the compiler writes the body of `{}`, so an export of that name has to be refused",
                name,
            );
        }
    }
}

/// A program may call the C function it exports. The declaration the call writes and the definition
/// the export builds describe one function, and the module holds it once, so the call reaches the
/// exported value.
#[test]
pub fn test_export_and_ffi_call_of_one_c_name() {
    let source = r##"
        module Main;

        twice_it : CInt -> CInt;
        twice_it = |x| x + x;
        FFI_EXPORT[twice_it, c_twice_it];

        call_back : CInt -> CInt;
        call_back = |x| FFI_CALL[CInt c_twice_it(CInt), x];

        main : IO ();
        main = (
            assert_eq(|_|"call back", call_back(21.c_int).i64, 42);;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// A program may call its own entry point, which re-runs it: the body the compiler writes goes onto
/// the declaration the call left, so the name is not taken from the C runtime that starts the
/// program. The re-entered run is given no arguments, which is what ends the recursion here.
#[test]
pub fn test_ffi_call_of_the_entry_point_re_runs_the_program() {
    let source = r##"
        module Main;

        reenter : CInt -> Ptr -> IO CInt;
        reenter = |argc, argv| FFI_CALL_IO[CInt main(CInt, Ptr), argc, argv];

        main : IO ();
        main = (
            let args = *get_args;
            if args.get_size == 0 {
                println("re-entered")
            };
            let status = *reenter(0.c_int, nullptr);
            assert_eq(|_|"the re-entered run failed", status.i64, 0);;
            println("first run")
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The entry point has a signature, so a call that writes another one is reported like any other
/// disagreement over one C name.
#[test]
pub fn test_ffi_call_of_the_entry_point_at_another_signature_fails() {
    let source = r##"
        module Main;

        reenter : IO ();
        reenter = FFI_CALL_IO[() main()];

        main : IO ();
        main = (
            reenter;;
            println("done")
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// The export defines the C function and the call goes through that one definition, so a call
/// written at another arity asks the exported function for an argument it does not take.
#[test]
pub fn test_export_and_ffi_call_of_one_c_name_at_two_signatures_fails() {
    let source = r##"
        module Main;

        twice_it : CInt -> CInt;
        twice_it = |x| x + x;
        FFI_EXPORT[twice_it, c_twice_it];

        call_back : CInt -> CInt;
        call_back = |x| FFI_CALL[CInt c_twice_it(CInt, CInt), x, x];

        main : IO ();
        main = println(call_back(21.c_int).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// The two calls write the same result type and the same fixed parameter, and differ only in
/// whether the C function ends in `...`. A variadic function is called differently from a
/// fixed-arity one, so the one declaration the module holds cannot serve both.
#[test]
pub fn test_ffi_calls_of_one_c_name_with_and_without_var_args_fails() {
    let source = r##"
        module Main;

        variadic : Ptr -> CInt;
        variadic = |p| FFI_CALL[CInt c_report(Ptr, ...), p, 42.c_int];

        fixed : Ptr -> CInt;
        fixed = |p| FFI_CALL[CInt c_report(Ptr), p];

        main : IO ();
        main = println(variadic(nullptr).to_string + " " + fixed(nullptr).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// C's default argument promotions widen a `float` to a `double` and an integer narrower than `int`
/// to an `int` on the way through `...`, which is why the C function reads its variadic arguments
/// as `double` and `int`. A call writes the value the C function reads.
///
/// A value wide enough to travel unchanged is passed as it is: the `I64` and `F64` cases pin that
/// the promotion stops where C stops it.
#[test]
pub fn test_ffi_call_promotes_its_variadic_arguments() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            assert_eq(|_|"F32", FFI_CALL[CDouble c_va_double(CInt, ...), 1.c_int, 2.5_F32], 2.5);;
            assert_eq(|_|"F64", FFI_CALL[CDouble c_va_double(CInt, ...), 1.c_int, 2.5_F64], 2.5);;
            assert_eq(|_|"I8", FFI_CALL[CInt c_va_int(CInt, ...), 1.c_int, -1_I8].i64, -1);;
            assert_eq(|_|"I16", FFI_CALL[CInt c_va_int(CInt, ...), 1.c_int, -1_I16].i64, -1);;
            assert_eq(|_|"U8", FFI_CALL[CInt c_va_int(CInt, ...), 1.c_int, 255_U8].i64, 255);;
            assert_eq(|_|"U16", FFI_CALL[CInt c_va_int(CInt, ...), 1.c_int, 65535_U16].i64, 65535);;
            assert_eq(|_|"I32", FFI_CALL[CInt c_va_int(CInt, ...), 1.c_int, -1_I32].i64, -1);;
            assert_eq(|_|"I64", FFI_CALL[I64 c_va_i64(CInt, ...), 1.c_int, -4294967296_I64], -4294967296);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdarg.h>
        #include <stdint.h>
        double  c_va_double(int n, ...) { va_list ap; va_start(ap, n); double  d = va_arg(ap, double);  va_end(ap); return d; }
        int     c_va_int(int n, ...)    { va_list ap; va_start(ap, n); int     i = va_arg(ap, int);     va_end(ap); return i; }
        int64_t c_va_i64(int n, ...)    { va_list ap; va_start(ap, n); int64_t l = va_arg(ap, int64_t); va_end(ap); return l; }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// The declared parameters of an `FFI_CALL` are written as C types, and an argument past them has
/// to be a C type too: the call hands it to C as the one scalar the value is. The error for each
/// rejected type names the way to pass the value instead.
#[test]
pub fn test_ffi_call_variadic_argument_of_a_non_c_type_fails() {
    let source_calling_with = |argument: &str| {
        format!(
            r##"
        module Main;

        type Wrap = unbox struct {{ v : I8 }};

        main : IO ();
        main = println(FFI_CALL[CInt c_report(CInt, ...), 1.c_int, {}].to_string);
    "##,
            argument
        )
    };

    // `Bool` is one byte in Fix, and the width C gives `_Bool` is implementation-defined.
    test_source_fail(
        &source_calling_with("true"),
        Configuration::develop_mode(),
        "`Std::Bool` cannot be passed through the `...` of an `FFI_CALL`. Use `U8` or `CInt`",
    );

    // A `String` holds its bytes in an array, and C reads them through a pointer.
    test_source_fail(
        &source_calling_with(r#""hi""#),
        Configuration::develop_mode(),
        "`Std::String` cannot be passed through the `...` of an `FFI_CALL`. Use `Std::String::borrow_c_str`",
    );

    // A boxed value crosses to C as its address, which is not what the call would hand over.
    test_source_fail(
        &source_calling_with("[1, 2, 3]"),
        Configuration::develop_mode(),
        "`Std::Array Std::I64` cannot be passed through the `...` of an `FFI_CALL`",
    );

    // A struct of one field is a struct, whatever `unwrap_newtype` later does with it.
    test_source_fail(
        &source_calling_with("Wrap { v : -1_I8 }"),
        Configuration::develop_mode(),
        "`Main::Wrap` cannot be passed through the `...` of an `FFI_CALL`",
    );
}

/// A pointer is one of the types an argument going through the `...` may be, and the `scanf` example
/// in the FFI section of `Document.md` writes one there. A `Ptr` travels through as it stands, so the
/// C function reads the bytes at the address the call wrote. This is also the route the diagnostic
/// for a `String` names: a `Ptr` taken with `Std::String::borrow_c_str`.
#[test]
pub fn test_ffi_call_passes_a_variadic_pointer_unchanged() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let first_byte = "hi".borrow_c_str(|p| FFI_CALL[CInt c_va_first_byte(CInt, ...), 1.c_int, p]);
            assert_eq(|_|"the byte at the address the call wrote", first_byte.i64, 'h'.i64);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdarg.h>
        int c_va_first_byte(int n, ...) {
            va_list ap; va_start(ap, n);
            unsigned char *p = va_arg(ap, unsigned char *);
            va_end(ap);
            return p[0];
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// The `IO` forms of an `FFI_CALL` carry the `IOState` token as one further argument, past the
/// declared parameters, and C never receives it. The promotions land on the arguments before it: an
/// `F32` written in an `FFI_CALL_IO` reaches the function as a `double`, and an `I8` written in an
/// `FFI_CALL_IOS` reaches it as an `int`.
#[test]
pub fn test_ffi_call_io_promotes_its_variadic_arguments() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let promoted_f32 = *FFI_CALL_IO[CDouble c_va_double(CInt, ...), 1.c_int, 2.5_F32];
            assert_eq(|_|"F32 through the `...` of an `FFI_CALL_IO`", promoted_f32, 2.5);;
            let promoted_i8 = *IO::from_runner(|ios| FFI_CALL_IOS[CInt c_va_int(CInt, ...), 1.c_int, -1_I8, ios]);
            assert_eq(|_|"I8 through the `...` of an `FFI_CALL_IOS`", promoted_i8.i64, -1);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdarg.h>
        double c_va_double(int n, ...) { va_list ap; va_start(ap, n); double d = va_arg(ap, double); va_end(ap); return d; }
        int    c_va_int(int n, ...)    { va_list ap; va_start(ap, n); int    i = va_arg(ap, int);    va_end(ap); return i; }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// The `...` begins after the declared parameters: the last declared parameter travels as the C type
/// the signature gives it, and only what follows is widened. A variadic function may also be called
/// with nothing past the declared parameters.
#[test]
pub fn test_ffi_call_promotes_only_what_follows_the_declared_parameters() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            assert_eq(|_|"the declared parameter and the argument after it", FFI_CALL[CInt c_va_after_i8(I8, ...), -2_I8, -1_I8].i64, -3);;
            assert_eq(|_|"no argument past the declared parameters", FFI_CALL[CInt c_va_after_i8(I8, ...), 7_I8].i64, 7);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdarg.h>
        #include <stdint.h>
        // Reads one further argument only where `n` is negative, so the call that writes none reads none.
        int c_va_after_i8(int8_t n, ...) {
            if (n >= 0) { return n; }
            va_list ap; va_start(ap, n);
            int v = va_arg(ap, int);
            va_end(ap);
            return (int)n + v;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// A parameter is a position like the result: the ABI carries a narrow integer in the low bits of a
/// register and the sign says which side extends it, so the two calls ask the one declaration for
/// opposite promises about the bits above the value.
#[test]
pub fn test_ffi_calls_of_one_c_name_taking_a_narrow_argument_at_two_signs_fails() {
    let source = r##"
        module Main;

        send_signed : I8 -> IO ();
        send_signed = |v| FFI_CALL_IO[() c_take(I8), v];

        send_unsigned : U8 -> IO ();
        send_unsigned = |v| FFI_CALL_IO[() c_take(U8), v];

        main : IO ();
        main = (
            send_signed(-1_I8);;
            send_unsigned(255_U8);;
            pure()
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// Both calls go through the one function the module holds under the name, so the arity the second
/// one writes is an arity the C function does not have.
#[test]
pub fn test_ffi_calls_of_one_c_name_at_two_signatures_fails() {
    let source = r##"
        module Main;

        one_argument : CInt -> CInt;
        one_argument = |x| FFI_CALL[CInt c_pick(CInt), x];

        two_arguments : CInt -> CInt -> CInt;
        two_arguments = |x, y| FFI_CALL[CInt c_pick(CInt, CInt), x, y];

        main : IO ();
        main = println((one_argument(1.c_int).i64 + two_arguments(2.c_int, 3.c_int).i64).to_string);
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// A value that fills its register travels the same way whichever sign the reader gives the bits,
/// so a declaration writes `I64` and `U64` identically and a program may read one C function's
/// result as either.
#[test]
pub fn test_ffi_calls_of_one_c_name_reading_a_wide_result_as_both_signs() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let unsigned = FFI_CALL[U64 c_all_ones()];
            let signed = FFI_CALL[I64 c_all_ones()];
            assert_eq(|_|"read as unsigned", unsigned, 18446744073709551615_U64);;
            assert_eq(|_|"read as signed", signed, -1);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        uint64_t c_all_ones(void) {
            return UINT64_MAX;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// The ABI carries an integer narrower than 32 bits in the low bits of a register, and the sign is
/// what says which side extends it. So the two descriptions ask the one declaration for opposite
/// promises about the bits above the value.
#[test]
pub fn test_ffi_calls_of_one_c_name_reading_a_narrow_result_as_both_signs_fails() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            let unsigned = FFI_CALL[U8 c_all_ones()];
            let signed = FFI_CALL[I8 c_all_ones()];
            println(unsigned.to_string + " " + signed.to_string)
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "One C function has one signature",
    );
}

/// `()` stands for `void`, which a C function takes as a return type alone.
#[test]
pub fn test_ffi_call_unit_parameter_fails() {
    let source = r##"
        module Main;

        main : IO ();
        main = (
            FFI_CALL_IO[() puts(())];;
            pure()
        );
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "which a C function cannot take as a parameter",
    );
}

/// A tuple is passed by value element by element, which the C ABI does not do for any target.
#[test]
pub fn test_export_aggregate_argument_fails() {
    let source = r##"
        module Main;

        sum_pair : (I64, I64) -> I64;
        sum_pair = |p| p.@0 + p.@1;
        FFI_EXPORT[sum_pair, c_sum_pair];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as an argument of an exported function",
    );
}

/// A named unbox struct is refused as the result of an exported function, as any aggregate is.
#[test]
pub fn test_export_unbox_struct_result_fails() {
    let source = r##"
        module Main;

        type Pair = unbox struct { a : I64, b : I64 };

        make_pair : I64 -> Pair;
        make_pair = |a| Pair { a : a, b : a * 2 };
        FFI_EXPORT[make_pair, c_make_pair];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as the return value of an exported function",
    );
}

/// A union is an aggregate of a tag and a payload buffer.
#[test]
pub fn test_export_union_argument_fails() {
    let source = r##"
        module Main;

        or_zero : Option I64 -> I64;
        or_zero = |o| o.as_some_or(0);
        FFI_EXPORT[or_zero, c_or_zero];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as an argument of an exported function",
    );
}

/// `String` is an unbox struct holding an array, so it is an aggregate like any other.
#[test]
pub fn test_export_string_result_fails() {
    let source = r##"
        module Main;

        greeting : String;
        greeting = "hello";
        FFI_EXPORT[greeting, c_greeting];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as the return value of an exported function",
    );
}

/// `Bool` is one byte in Fix, but the width C gives `_Bool` is implementation-defined, and a
/// caller is free to declare the function with `int` instead.
#[test]
pub fn test_export_bool_argument_fails() {
    let source = r##"
        module Main;

        negate : Bool -> Bool;
        negate = |b| !b;
        FFI_EXPORT[negate, c_negate];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "Use `U8` or `CInt`, and convert it on the Fix side",
    );
}

/// `Bool` is refused as the result of an exported function for the reason it is refused as an
/// argument: the width C gives `_Bool` is implementation-defined.
#[test]
pub fn test_export_bool_result_fails() {
    let source = r##"
        module Main;

        is_positive : CInt -> Bool;
        is_positive = |x| x > 0.c_int;
        FFI_EXPORT[is_positive, c_is_positive];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as the return value of an exported function",
    );
}

/// A struct with one field is a struct. `unwrap_newtype` would later replace it with the field
/// type, but only at `-O max` and above, so what C receives would depend on the optimization
/// level. Unwrap it in the exported function's own signature instead.
#[test]
pub fn test_export_newtype_argument_fails() {
    let source = r##"
        module Main;

        type Meters = unbox struct { v : I64 };

        double_it : Meters -> I64;
        double_it = |m| m.@v * 2;
        FFI_EXPORT[double_it, c_double_it];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as an argument of an exported function",
    );
}

/// `()` is a struct with no field. It is allowed as the result, where it becomes `void`.
#[test]
pub fn test_export_unit_argument_fails() {
    let source = r##"
        module Main;

        ignore : () -> CInt;
        ignore = |_| 0.c_int;
        FFI_EXPORT[ignore, c_ignore];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as an argument of an exported function",
    );
}

/// A closure is a function pointer paired with its captured values.
#[test]
pub fn test_export_closure_argument_fails() {
    let source = r##"
        module Main;

        apply_to_zero : (I64 -> I64) -> I64;
        apply_to_zero = |f| f(0);
        FFI_EXPORT[apply_to_zero, c_apply_to_zero];

        main : IO ();
        main = pure();
    "##;
    test_source_fail(
        &source,
        Configuration::develop_mode(),
        "cannot be used as an argument of an exported function",
    );
}

/// The way to exchange an aggregate with C: the C side owns the memory and passes a pointer to
/// it, and Fix copies in or out through a boxed value. `borrow_boxed` and `mutate_boxed` hand
/// out a pointer to the payload of a boxed value, whose fields are laid out like the fields of
/// the corresponding C structure.
///
/// The Fix source is the "Returning more than one value" example of the FFI section of
/// `Document.md` and `Document-ja.md`.
#[test]
pub fn test_export_aggregate_through_pointer() {
    let source = r##"
        module Main;

        type Pair = box struct { a : I64, b : F64 };

        // Fills the `struct pair` that `dst` points to.
        write_pair : Ptr -> U64 -> IO ();
        write_pair = |dst, size| (
            let pair = Pair { a : 10, b : 0.5 };
            pair.borrow_boxed_io(|src| FFI_CALL_IO[Ptr memcpy(Ptr, Ptr, U64), dst, src, size]);;
            pure()
        );
        FFI_EXPORT[write_pair, write_pair]; // void write_pair(struct pair* dst, uint64_t size);

        // Reads the `struct pair` that `src` points to.
        sum_pair : Ptr -> U64 -> IO F64;
        sum_pair = |src, size| (
            let pair = Pair { a : 0, b : 0.0 };
            let (pair, _) = *pair.mutate_boxed_io(|dst|
                FFI_CALL_IO[Ptr memcpy(Ptr, Ptr, U64), dst, src, size]
            );
            pure $ pair.@a.to_F64 + pair.@b
        );
        FFI_EXPORT[sum_pair, sum_pair]; // double sum_pair(const struct pair* src, uint64_t size);

        main : IO ();
        main = (
            let res = FFI_CALL[CInt run_c()];
            assert_eq(|_|"C reported failure", res, 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        struct pair { int64_t a; double b; };

        void write_pair(struct pair *dst, uint64_t size);
        double sum_pair(const struct pair *src, uint64_t size);

        int run_c() {
            struct pair p = {0, 0.0};
            write_pair(&p, sizeof(p));
            if (p.a != 10 || p.b != 0.5) { return 1; }

            struct pair q = {3, 0.25};
            if (sum_pair(&q, sizeof(q)) != 3.25) { return 1; }

            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// A boxed value crosses the FFI boundary as an opaque pointer, and the C side keeps it alive
/// through the `void (*)(void*)` functions that `get_funptr_retain` and `get_funptr_release`
/// return. This test follows the accounting the documentation prescribes: the pointer that
/// `boxed_to_retained_ptr` returns carries one responsibility to release, each retain adds one,
/// and passing the pointer to an exported function takes one over.
///
/// The destructor of the exchanged value calls back into C, so the C side sees exactly when the
/// value is destroyed rather than inferring it.
#[test]
pub fn test_export_boxed_value_and_reference_counting_funptrs() {
    let source = r##"
        module Main;

        make_resource : I64 -> IO (Destructor I64);
        make_resource = |tag| Destructor::make(tag, |tag|
            FFI_CALL_IO[() c_note_release(I64), tag].map(|_| tag)
        );

        // Takes over one responsibility to release the resource it is given.
        resource_tag : Destructor I64 -> I64;
        resource_tag = |res| res.borrow(|tag| tag);
        FFI_EXPORT[resource_tag, c_resource_tag];

        main : IO ();
        main = (
            let retain = (|_| undefined("") : Destructor I64).get_funptr_retain;
            let release = (|_| undefined("") : Destructor I64).get_funptr_release;

            // C retains, calls the exported function, and releases. The value stays alive across
            // the call and is destroyed by the last release.
            let resource = *make_resource(7);
            let ptr = *resource.boxed_to_retained_ptr;
            let code = *FFI_CALL_IO[CInt c_use_resource(Ptr, Ptr, Ptr), ptr, retain, release];
            assert_eq(|_|"C reported failure", code, 0.c_int);;

            // A retain followed by a release leaves the value untouched, and `boxed_from_retained_ptr`
            // brings the remaining responsibility back to Fix.
            let resource = *make_resource(9);
            let ptr = *resource.boxed_to_retained_ptr;
            FFI_CALL_IO[() c_call_refcount_funptr(Ptr, Ptr), retain, ptr];;
            FFI_CALL_IO[() c_call_refcount_funptr(Ptr, Ptr), release, ptr];;
            let released = *FFI_CALL_IO[I64 c_was_released(I64), 9];
            assert_eq(|_|"the resource was destroyed while C still held it", released, 0);;
            let resource : Destructor I64 = *ptr.boxed_from_retained_ptr;
            assert_eq(|_|"the resource did not survive", resource.borrow(|tag| tag), 9);;

            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        #define TAG_COUNT 16

        static int64_t released[TAG_COUNT];

        void c_note_release(int64_t tag) {
            if (0 <= tag && tag < TAG_COUNT) { released[tag] = 1; }
        }

        int64_t c_was_released(int64_t tag) {
            return (0 <= tag && tag < TAG_COUNT) ? released[tag] : 0;
        }

        // Calls a function obtained by `get_funptr_retain` or `get_funptr_release`.
        void c_call_refcount_funptr(void *funptr, void *value) {
            ((void (*)(void *))funptr)(value);
        }

        int64_t c_resource_tag(void *resource);

        int c_use_resource(void *ptr, void *retain, void *release) {
            // One responsibility to release is held here. Retain once, so that `c_resource_tag`,
            // which takes one over, leaves the value alive.
            c_call_refcount_funptr(retain, ptr);
            if (c_resource_tag(ptr) != 7) { return 1; }
            if (c_was_released(7)) { return 1; }

            // The last responsibility. Fulfilling it destroys the value and runs its destructor.
            c_call_refcount_funptr(release, ptr);
            if (!c_was_released(7)) { return 1; }

            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

// The FFI section of `Document.md` and `Document-ja.md` prints the programs below for the reader
// to copy. Running them here keeps the manual honest: a change to the language or to the standard
// library that invalidates an example fails the suite instead of a reader's build.

/// Handing an array to the foreign language as a retained pointer, and reading an element back
/// through it. `get_fix_array_element` takes over the responsibility to release, so C calls it
/// once for the one pointer it holds.
#[test]
pub fn test_document_example_boxed_array_across_ffi() {
    let source = r##"
        module Main;

        create_fix_array : IO Ptr;
        create_fix_array = (
            let arr = Box::make([1,2,3,4,5]);
            arr.boxed_to_retained_ptr
        );
        FFI_EXPORT[create_fix_array, create_fix_array];

        get_fix_array_element : Ptr -> I64 -> IO I64;
        get_fix_array_element = |ptr, idx| (
            let arr : Box (Array I64) = *boxed_from_retained_ptr(ptr);
            pure $ arr.@value.@(idx)
        );
        FFI_EXPORT[get_fix_array_element, get_fix_array_element];

        main : IO ();
        main = (
            let res = FFI_CALL[CInt run_c()];
            assert_eq(|_|"C reported failure", res, 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdint.h>

        void *create_fix_array(void);
        int64_t get_fix_array_element(void *ptr, int64_t idx);

        int run_c() {
            void *arr = create_fix_array();
            if (get_fix_array_element(arr, 2) != 3) { return 1; }
            return 0;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// Reading the fields of a boxed Fix struct from C through the pointer `borrow_boxed` lends.
/// The C side records what it saw so that Fix can check the fields arrived at the offsets a C
/// structure with the same fields would put them at.
#[test]
pub fn test_document_example_borrow_boxed_struct_from_c() {
    let source = r##"
        module Main;

        type Vec = box struct { x : CDouble, y : CDouble };

        main : IO ();
        main = (
            let vec = Vec { x : 3.0, y : 4.0 };
            eval vec.borrow_boxed(|p| FFI_CALL[() access_vec(Ptr), p]);
            let sum = *FFI_CALL_IO[CDouble seen_sum()];
            assert_eq(|_|"C saw other fields", sum, 7.0);;
            pure()
        );
    "##;
    let c_source = r##"
        struct Vec {
            double x;
            double y;
        };

        static double sum;

        void access_vec(struct Vec *v) {
            sum = v->x + v->y;
        }

        double seen_sum() {
            return sum;
        }
    "##;
    test_source_with_c(&source, &c_source, function_name!());
}

/// Passing an array's element buffer to C, for writing through `mutate_elements` and for
/// reading through `borrow_elements`.
#[test]
pub fn test_document_example_array_elements_from_c() {
    let source = r##"
        module Main;

        fill_bytes : U8 -> Array U8 -> Array U8;
        fill_bytes = |c, arr| (
            let n = arr.@size.c_size_t;
            let (arr, _) = arr.mutate_elements(|p|
                pure $ FFI_CALL[Ptr memset(Ptr, CInt, CSizeT), p, c.c_int, n]
            );
            arr
        );

        contains_byte : U8 -> Array U8 -> Bool;
        contains_byte = |c, arr| (
            let n = arr.@size.c_size_t;
            let found = arr.borrow_elements(|p|
                FFI_CALL[Ptr memchr(Ptr, CInt, CSizeT), p, c.c_int, n]
            );
            found != nullptr
        );

        main : IO ();
        main = (
            let arr = Array::fill(4, 0_U8).fill_bytes(65_U8);
            assert_eq(|_|"fill_bytes wrote other bytes", arr, [65_U8, 65_U8, 65_U8, 65_U8]);;
            assert(|_|"contains_byte missed a byte that is there", arr.contains_byte(65_U8));;
            assert(|_|"contains_byte found a byte that is absent", !arr.contains_byte(66_U8));;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Each shape an exported value's type may take — no argument, one argument, two arguments, and
/// an `IO` action with and without a result — reaches C as a function of the matching arity and
/// result type.
#[test]
pub fn test_export_signature_shapes() {
    let source = r##"
        module Main;

        value : CInt;
        value = 42.c_int;
        FFI_EXPORT[value, c_value];

        increment : CInt -> CInt;
        increment = |x| x + 1.c_int;
        FFI_EXPORT[increment, c_increment];

        two_variable : CInt -> CInt -> CInt;
        two_variable = |x, y| 2.c_int * x + y;
        FFI_EXPORT[two_variable, c_two_variable];

        io_action : IO ();
        io_action = println("io_action");
        FFI_EXPORT[io_action, c_io_action];

        io_action2 : CInt -> IO ();
        io_action2 = |x| do {
            println("io_action2: " + x.to_string);;
            pure()
        };
        FFI_EXPORT[io_action2, c_io_action2];

        io_action3 : CInt -> IO CInt;
        io_action3 = |x| do {
            println("io_action3");;
            pure(x + 1.c_int)
        };
        FFI_EXPORT[io_action3, c_io_action3];

        main: IO ();
        main = (
            let res = FFI_CALL[CInt call_fix_values()];
            assert_eq(|_|"", res, 0.c_int);;
            pure()
        );
    "##;
    let c_source = r##"
        #include <stdio.h>

        int c_value();
        int c_increment(int x);
        int c_two_variable(int x, int y);
        void c_io_action();
        void c_io_action2(int x);
        int c_io_action3(int x);

        int call_fix_values() {
            int x = c_value();
            if (x != 42) {
                return 1;
            }

            int y = c_increment(42);
            if (y != 43) {
                return 1;
            }

            if (c_two_variable(3, 2) != 8) {
                return 1;
            }

            c_io_action();

            c_io_action2(42);

            int z = c_io_action3(42);
            if (z != 43) {
                return 1;
            }

            return 0;
        }
    "##;

    test_source_with_c(&source, &c_source, function_name!());
}

/// `get_funptr_release` and `get_funptr_retain` take the type to act on from a `Lazy` whose body
/// never runs, so a boxed type that has no value at hand can still be named.
#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_1() {
    let source = r##"
        module Main;

        type VoidType = box struct {};
        // No constructor for `VoidType` is provided.

        main: IO ();
        main = (
            let release = (|_| undefined("") : VoidType).get_funptr_release;
            let retain = (|_| undefined("") : VoidType).get_funptr_retain;
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// The `Lazy` that names the type may fix it indirectly: through the domain or the codomain of a
/// function in scope, or through an annotation mentioning a `Boxed` type variable.
#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_2() {
    let source = r##"
        module Main;

        get_release_func_of_codom : [b : Boxed] (a -> b) -> Ptr;
        get_release_func_of_codom = |f| (
            let lazy_b = |_| f(undefined(""));
            lazy_b.get_funptr_release
        );

        get_release_func_of_codom_2 : [b : Boxed] (a -> b) -> Ptr;
        get_release_func_of_codom_2 = |f| (
            let lazy_b = |_| undefined("") : b;
            lazy_b.get_funptr_release
        );

        get_retain_func_of_dom : [a : Boxed] (a -> b) -> Ptr;
        get_retain_func_of_dom = |f| (
            let lazy_a = |_| let x = undefined(""); let _ = f(x); x;
            lazy_a.get_funptr_release
        );

        get_retain_func_of_dom_2 : [a : Boxed] (a -> b) -> Ptr;
        get_retain_func_of_dom_2 = |f| (
            let lazy_a : Lazy a = |_| undefined("");
            lazy_a.get_funptr_release
        );

        main: IO ();
        main = (
            let release = get_release_func_of_codom(|_ : I64| Box::make(42));
            let retain = get_retain_func_of_dom(|_ : Box I64| 42);
            let release_2 = get_release_func_of_codom_2(|_ : I64| Box::make(42));
            let retain_2 = get_retain_func_of_dom_2(|_ : Box I64| 42);
            pure()
        );
    "##;
    test_source(&source, Configuration::develop_mode());
}

/// Only a boxed type has a reference counter, so `get_funptr_release` on an unboxed type such as
/// `I64` is a compilation error.
#[test]
pub fn test_get_funptr_release_error() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let release = (|_| undefined("") : I64).get_funptr_release;
            pure()
        );
    "##;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

/// `get_funptr_retain` requires a boxed type, and rejects an unboxed one such as `I64`.
#[test]
pub fn test_get_funptr_retain_error() {
    let source = r##"
        module Main;

        main: IO ();
        main = (
            let retain = (|_| undefined("") : I64).get_funptr_retain;
            pure()
        );
    "##;
    test_source_fail(&source, Configuration::develop_mode(), "");
}

/// A non-variadic `FFI_CALL` signature fixes the number of arguments the call takes, and both too
/// many and too few are reported as a source-level diagnostic.
#[test]
pub fn test_ffi_call_wrong_argument_count() {
    let too_many = r##"
        module Main;

        main : IO ();
        main = (
            eval *FFI_CALL_IO[CInt puts(Ptr), "hi".borrow_c_str(|p| p), 1];
            pure()
        );
    "##;
    test_source_fail(
        too_many,
        Configuration::develop_mode(),
        "Wrong number of arguments in FFI_CALL_IO expression.",
    );

    let too_few = r##"
        module Main;

        main : IO ();
        main = (
            eval *FFI_CALL_IO[CInt puts(Ptr)];
            pure()
        );
    "##;
    test_source_fail(
        too_few,
        Configuration::develop_mode(),
        "Wrong number of arguments in FFI_CALL_IO expression.",
    );
}
