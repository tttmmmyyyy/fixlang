use crate::{
    configuration::Configuration,
    constants::COMPILER_TEST_WORKING_PATH,
    misc::function_name,
    tests::test_util::{fix_command, test_source, test_source_fail, test_source_with_c},
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

#[test]
pub fn test_export_scalar_types() {
    // Every scalar type an exported function may exchange, in both directions, checked against the
    // C type it is documented to correspond to.
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

#[test]
pub fn test_narrow_integers_carry_the_c_extension_attribute() {
    // An integer narrower than 32 bits travels in the low bits of a register, and `signext` /
    // `zeroext` is how a signature says which side of the call extends it. Leaving it off costs
    // nothing on x86-64, where whoever reads the value narrows it anyway, and yields the wrong
    // number on AArch64, where the reader takes the whole register on the promise that the other
    // side extended it. On an x86-64 host the attribute is therefore visible only in the emitted
    // IR, which is what this test reads.
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
    let ir = fs::read_dir(&work_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.ends_with(".ll") && !name.ends_with("_optimized.ll")
        })
        .map(|p| fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    for expected in [
        // An exported function extends its narrow arguments and result, by sign or by zero
        // according to the Fix type.
        "define signext i8 @c_add_i8(i8 signext %0, i8 signext %1)",
        "define zeroext i16 @c_add_u16(i16 zeroext %0, i16 zeroext %1)",
        // A type that fills the register carries no attribute, and neither does a pointer.
        "define i64 @c_add_i64(i64 %0, i64 %1)",
        "define void @c_write_byte(ptr %0, i8 zeroext %1)",
        // The same holds for the C functions Fix calls.
        "declare void @fixruntime_u8_to_bytes(ptr, i8 zeroext)",
    ] {
        assert!(
            ir.contains(expected),
            "emitted IR lacks `{}`:\n{}",
            expected,
            ir
        );
    }
}

#[test]
pub fn test_export_boxed_value() {
    // A boxed value returned to the foreign language arrives as an opaque pointer carrying one
    // responsibility to release, and an exported function taking a boxed argument takes that
    // responsibility over. The two together are balanced, which memcheck checks.
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

#[test]
pub fn test_export_non_ascii_c_function_name_fails() {
    // A C identifier is written in ASCII, so a letter outside it is refused even though it is a
    // letter. The grammar takes any character up to the `]`, so the name reaches this check.
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

#[test]
pub fn test_ffi_call_unit_parameter_fails() {
    // `()` stands for `void`, which a C function takes as a return type alone.
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

#[test]
pub fn test_export_aggregate_argument_fails() {
    // A tuple is passed by value element by element, which the C ABI does not do for any target.
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

#[test]
pub fn test_export_unbox_struct_result_fails() {
    // A named unbox struct is refused as the result of an exported function, as any aggregate is.
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

#[test]
pub fn test_export_union_argument_fails() {
    // A union is an aggregate of a tag and a payload buffer.
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

#[test]
pub fn test_export_string_result_fails() {
    // `String` is an unbox struct holding an array, so it is an aggregate like any other.
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

#[test]
pub fn test_export_bool_argument_fails() {
    // `Bool` is one byte in Fix, but the width C gives `_Bool` is implementation-defined, and a
    // caller is free to declare the function with `int` instead.
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

#[test]
pub fn test_export_bool_result_fails() {
    // The reason `Bool` is refused does not depend on which side of the arrow it sits.
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

#[test]
pub fn test_export_newtype_argument_fails() {
    // A struct with one field is a struct. `unwrap_newtype` would later replace it with the field
    // type, but only at `-O max` and above, so what C receives would depend on the optimization
    // level. Unwrap it in the exported function's own signature instead.
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

#[test]
pub fn test_export_unit_argument_fails() {
    // `()` is a struct with no field. It is allowed as the result, where it becomes `void`.
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

#[test]
pub fn test_export_closure_argument_fails() {
    // A closure is a function pointer paired with its captured values.
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

#[test]
pub fn test_export_aggregate_through_pointer() {
    // The way to exchange an aggregate with C: the C side owns the memory and passes a pointer to
    // it, and Fix copies in or out through a boxed value. `borrow_boxed` and `mutate_boxed` hand
    // out a pointer to the payload of a boxed value, whose fields are laid out like the fields of
    // the corresponding C structure.
    //
    // The Fix source is the "Returning more than one value" example of the FFI section of
    // `Document.md` and `Document-ja.md`.
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

#[test]
pub fn test_export_boxed_value_and_reference_counting_funptrs() {
    // A boxed value crosses the FFI boundary as an opaque pointer, and the C side keeps it alive
    // through the `void (*)(void*)` functions that `get_funptr_retain` and `get_funptr_release`
    // return. This test follows the accounting the documentation prescribes: the pointer that
    // `boxed_to_retained_ptr` returns carries one responsibility to release, each retain adds one,
    // and passing the pointer to an exported function takes one over.
    //
    // The destructor of the exchanged value calls back into C, so the C side sees exactly when the
    // value is destroyed rather than inferring it.
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

#[test]
pub fn test_document_example_boxed_array_across_ffi() {
    // Handing an array to the foreign language as a retained pointer, and reading an element back
    // through it. `get_fix_array_element` takes over the responsibility to release, so C calls it
    // once for the one pointer it holds.
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

#[test]
pub fn test_document_example_borrow_boxed_struct_from_c() {
    // Reading the fields of a boxed Fix struct from C through the pointer `borrow_boxed` lends.
    // The C side records what it saw so that Fix can check the fields arrived at the offsets a C
    // structure with the same fields would put them at.
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

#[test]
pub fn test_document_example_array_elements_from_c() {
    // Passing an array's element buffer to C, for writing through `mutate_elements` and for
    // reading through `borrow_elements`.
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

#[test]
pub fn test_export_signature_shapes() {
    // Each shape an exported value's type may take — no argument, one argument, two arguments, and
    // an `IO` action with and without a result — reaches C as a function of the matching arity and
    // result type.
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

#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_1() {
    // `get_funptr_release` and `get_funptr_retain` take the type to act on from a `Lazy` whose body
    // never runs, so a boxed type that has no value at hand can still be named.
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

#[test]
pub fn test_unsafe_get_release_retain_function_of_boxed_value_decltype_technique_2() {
    // The `Lazy` that names the type may fix it indirectly: through the domain or the codomain of a
    // function in scope, or through an annotation mentioning a `Boxed` type variable.
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

#[test]
pub fn test_get_funptr_release_error() {
    // Only a boxed type has a reference counter, so `get_funptr_release` on an unboxed type such as
    // `I64` is a compilation error.
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

#[test]
pub fn test_get_funptr_retain_error() {
    // `get_funptr_retain` requires a boxed type, and rejects an unboxed one such as `I64`.
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
