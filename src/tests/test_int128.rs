//! The 128-bit integer types `I128` and `U128`: their literals, their arithmetic and conversions,
//! their text and bytes, and the C signatures that do not take them.

use crate::configuration::{Configuration, FixOptimizationLevel};
use crate::tests::test_util::{
    generated_llvm_ir, test_source, test_source_fail, test_with_a_runtime_zero,
};

/// A literal of a 128-bit type holds every value of its type, the bits past 64 included, and a
/// hexadecimal literal may fill the width of a signed type.
#[test]
pub fn test_a_128_bit_literal_holds_every_value_of_its_type() {
    test_source(
        r#"
        module Main;
        main : IO ();
        main = (
            assert_eq(|_|"U128::maximum", U128::maximum, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_U128);;
            assert_eq(|_|"I128::minimum", I128::minimum, -0x80000000000000000000000000000000_I128);;
            assert_eq(|_|"A bit pattern filling I128", 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_I128, -1_I128);;
            assert_eq(|_|"A literal past 2^64", 18446744073709551616_U128 - 1_U128, U64::maximum.u128);;
            assert_eq(|_|"A literal with an exponent", 1e38_U128, 100000000000000000000000000000000000000_U128);;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// A literal outside the range of a 128-bit type is reported, and so is a bit pattern wider than
/// the type.
#[test]
pub fn test_a_128_bit_literal_outside_its_type_is_reported() {
    let source = |literal: &str| {
        format!(
            r#"
            module Main;
            main : IO ();
            main = (
                eval {};
                pure()
            );
        "#,
            literal
        )
    };
    test_source_fail(
        &source("340282366920938463463374607431768211456_U128"),
        Configuration::develop_mode(),
        "The value of an integer literal `340282366920938463463374607431768211456` is out of range of `U128`.",
    );
    test_source_fail(
        &source("-170141183460469231731687303715884105729_I128"),
        Configuration::develop_mode(),
        "The value of an integer literal `-170141183460469231731687303715884105729` is out of range of `I128`.",
    );
    test_source_fail(
        &source("0x100000000000000000000000000000000_I128"),
        Configuration::develop_mode(),
        "The value of an integer literal `0x100000000000000000000000000000000` does not fit in the width of `I128`.",
    );
}

/// The arithmetic, the comparisons and the bit operations of the 128-bit types compute on all 128
/// bits. The operands are built from the run-time zero, so a division and a remainder reach the
/// helper functions the compiled program links rather than being folded.
#[test]
pub fn test_the_128_bit_types_compute_on_every_bit() {
    test_with_a_runtime_zero(
        r#"
            let u = zero.u128;
            let i = zero.i128;
            assert_eq(|_|"U128 division", (U128::maximum + u) / 3_U128, 113427455640312821154458202477256070485_U128);;
            assert_eq(|_|"U128 remainder", (U128::maximum + u) % 10_U128, 5_U128);;
            assert_eq(|_|"I128 division", (I128::minimum + i) / 7_I128, -24305883351495604533098186245126300818_I128);;
            assert_eq(|_|"I128 remainder", (I128::minimum + i) % 7_I128, -2_I128);;
            assert_eq(|_|"U128 wraps", U128::maximum + u + 1_U128, 0_U128);;
            assert_eq(|_|"I128 comparison", I128::minimum + i < I128::maximum + i, true);;
            assert_eq(|_|"U128 comparison past 2^64", 18446744073709551616_U128 + u > U64::maximum.u128, true);;

            let x = 12345678901234567_U64 + zero.u64;
            let y = 98765432109876543_U64 + zero.u64;
            let product = x.u128 * y.u128;
            assert_eq(|_|"The high half of a 64-bit product", product.shift_right(64_U128).u64, 66099811787816_U64);;
            assert_eq(|_|"The low half of a 64-bit product", product.u64, 6301857727962151225_U64);;

            assert_eq(|_|"I128 arithmetic right shift", (I128::minimum + i).shift_right(127_I128), -1_I128);;
            assert_eq(|_|"U128 left shift", (1_U128 + u).shift_left(127_U128), 170141183460469231731687303715884105728_U128);;
            assert_eq(|_|"U128 bit_and", (U128::maximum + u).bit_and(18446744073709551616_U128), 18446744073709551616_U128);;
            assert_eq(|_|"U128 bit_or", (18446744073709551616_U128 + u).bit_or(1_U128), 18446744073709551617_U128);;
            assert_eq(|_|"U128 bit_xor", (U128::maximum + u).bit_xor(U128::maximum), 0_U128);;
            assert_eq(|_|"I128 bit_not", (0_I128 + i).bit_not, -1_I128);;
            assert_eq(|_|"I128::abs", (-5_I128 + i).abs, 5_I128);;
        "#,
        Configuration::develop_mode(),
    );
}

/// A 128-bit value converts to and from the other numeric types: a narrower integer is extended by
/// its own sign, a narrowing keeps the low bits, and a floating-point value rounds.
#[test]
pub fn test_the_128_bit_types_convert_to_and_from_the_other_numeric_types() {
    test_with_a_runtime_zero(
        r#"
            let u = zero.u128;
            let i = zero.i128;
            assert_eq(|_|"I64 to I128", (zero - 1).i128, -1_I128);;
            assert_eq(|_|"I64 to U128", (zero - 1).u128, U128::maximum);;
            assert_eq(|_|"U64 to I128", (zero.u64 - 1_U64).i128, 18446744073709551615_I128);;
            assert_eq(|_|"U128 to I64", (U128::maximum + u).i64, -1);;
            assert_eq(|_|"U128 to I128", (U128::maximum + u).i128, -1_I128);;
            assert_eq(|_|"I128 to F64", (I128::minimum + i).f64, -1.7014118346046923e38);;
            assert_eq(|_|"F64 to U128", (1.0e30 + zero.f64).u128, 1000000000000000019884624838656_U128);;
            assert_eq(|_|"U128 to F32", (1_U128 + u).shift_left(100_U128).f32, 1267650600228229401496703205376.0_F32);;
        "#,
        Configuration::develop_mode(),
    );
}

/// A 128-bit value writes as its decimal text and reads back from it, at the numbers where the
/// writer cuts the digits into chunks of 19 and where a chunk begins with zeros.
#[test]
pub fn test_the_128_bit_types_convert_to_and_from_text() {
    test_source(
        r#"
        module Main;

        round_trip : [a : ToString, a : FromString, a : Eq] String -> a -> IO ();
        round_trip = |text, v| (
            assert_eq(|_|"to_string of " + text, v.to_string, text);;
            assert_eq(|_|"from_string of " + text, from_string(text), Result::ok(v) : Result ErrMsg a)
        );

        main : IO ();
        main = (
            round_trip("0", 0_U128);;
            round_trip("18446744073709551615", 18446744073709551615_U128);;
            round_trip("18446744073709551616", 18446744073709551616_U128);;
            round_trip("10000000000000000000000000000000000000", 10000000000000000000000000000000000000_U128);;
            round_trip("184467440737095516160000000000000000000", 184467440737095516160000000000000000000_U128);;
            round_trip("340282366920938463463374607431768211455", U128::maximum);;
            round_trip("-1", -1_I128);;
            round_trip("-18446744073709551616", -18446744073709551616_I128);;
            round_trip("170141183460469231731687303715884105727", I128::maximum);;
            round_trip("-170141183460469231731687303715884105728", I128::minimum);;
            assert_eq(|_|"+007", from_string("+007"), Result::ok(7_I128) : Result ErrMsg I128);;
            assert_eq(|_|"-0", from_string("-0"), Result::ok(0_U128) : Result ErrMsg U128);;
            assert_eq(
                |_|"One past U128::maximum",
                from_string("340282366920938463463374607431768211456"),
                Result::err("Failed to convert string to integer (out of range): 340282366920938463463374607431768211456") : Result ErrMsg U128
            );;
            assert_eq(
                |_|"One past I128::minimum",
                from_string("-170141183460469231731687303715884105729"),
                Result::err("Failed to convert string to integer (out of range): -170141183460469231731687303715884105729") : Result ErrMsg I128
            );;
            assert_eq(
                |_|"A negative U128",
                from_string("-1"),
                Result::err("Failed to convert string to integer (out of range): -1") : Result ErrMsg U128
            );;
            assert_eq(
                |_|"Not a number",
                from_string("12a"),
                Result::err("Failed to convert string to integer (invalid format): 12a") : Result ErrMsg I128
            );;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// A 128-bit value writes as 16 bytes and reads back from them, and 15 bytes are an error.
#[test]
pub fn test_the_128_bit_types_convert_to_and_from_bytes() {
    test_source(
        r#"
        module Main;
        main : IO ();
        main = (
            let v = 0x0123456789ABCDEFFEDCBA9876543210_U128;
            assert_eq(|_|"U128 bytes", v.to_bytes.@size, 16);;
            assert_eq(|_|"U128 round trip", from_bytes(v.to_bytes), Result::ok(v) : Result ErrMsg U128);;
            assert_eq(|_|"I128 round trip", from_bytes(I128::minimum.to_bytes), Result::ok(I128::minimum) : Result ErrMsg I128);;
            assert_eq(|_|"I128 and U128 share their bytes", (-1_I128).to_bytes, U128::maximum.to_bytes);;
            assert_eq(
                |_|"15 bytes",
                from_bytes(Array::fill(15, 0_U8)),
                Result::err("Byte array of length 15 cannot be interpreted as U128.") : Result ErrMsg U128
            );;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// A C function signature written in `FFI_CALL` does not take a 128-bit type, since C has no
/// standard type for it, and the report says how to pass such a value.
#[test]
pub fn test_ffi_call_rejects_a_128_bit_type() {
    test_source_fail(
        r#"
        module Main;
        main : IO ();
        main = (
            eval FFI_CALL[I64 labs(I128), 1_I128];
            pure()
        );
    "#,
        Configuration::develop_mode(),
        "`I128` has no counterpart among the C types, so a C function cannot take or return it.\nHINT: pass the value as two `U64`s, its low and its high 64 bits.",
    );
    test_source_fail(
        r#"
        module Main;
        main : IO ();
        main = (
            eval FFI_CALL[U128 labs(I64), 1];
            pure()
        );
    "#,
        Configuration::develop_mode(),
        "`U128` has no counterpart among the C types",
    );
}

/// An exported function does not exchange a 128-bit type.
#[test]
pub fn test_ffi_export_rejects_a_128_bit_type() {
    test_source_fail(
        r#"
        module Main;
        twice : U128 -> U128;
        twice = |x| x * 2_U128;
        FFI_EXPORT[twice, fix_twice];
        main : IO ();
        main = pure();
    "#,
        Configuration::develop_mode(),
        "`Std::U128` cannot be used as an argument of an exported function. C has no standard 128-bit integer type, so pass the value as two `U64`s, its low and its high 64 bits.",
    );
}

/// `from_string` into a 128-bit type reads and rejects the texts the readers of the other integer
/// types read and reject: a text that is not an optional sign followed by decimal digits is
/// malformed, even where its digits name a number outside the type, and digits naming a number far
/// beyond the type are out of range.
#[test]
pub fn test_the_128_bit_types_read_the_texts_the_other_integer_types_read() {
    test_source(
        r#"
        module Main;

        // Reads `text` as the type of the first argument, and asserts that the answer, written as
        // text, or the error message is `expected`.
        expect : [a : FromString, a : ToString] a -> String -> String -> IO ();
        expect = |_, text, expected| (
            let read : Result ErrMsg a = text.from_string;
            let actual = if read.is_ok { read.as_ok.to_string } else { read.as_err };
            assert_eq(|_|"reading \"" + text + "\"", actual, expected)
        );

        out_of_range : String -> String;
        out_of_range = |text| "Failed to convert string to integer (out of range): " + text;

        malformed : String -> String;
        malformed = |text| "Failed to convert string to integer (invalid format): " + text;

        main : IO ();
        main = (
            expect(0_U128, "99999999999999999999999999999999999999999999", out_of_range("99999999999999999999999999999999999999999999"));;
            expect(0_I128, "-99999999999999999999999999999999999999999999", out_of_range("-99999999999999999999999999999999999999999999"));;
            expect(0_U128, "0000000000340282366920938463463374607431768211455", "340282366920938463463374607431768211455");;
            expect(0_I128, "", malformed(""));;
            expect(0_U128, "-", malformed("-"));;
            expect(0_I128, "+-1", malformed("+-1"));;
            expect(0_U128, "1 ", malformed("1 "));;
            expect(0_I128, "99999999999999999999999999999999999999999999x", malformed("99999999999999999999999999999999999999999999x"));;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// A value of a 128-bit type keeps all its bits wherever a value can be held: a field of a boxed
/// struct placed after a narrower field, a field of an unbox struct, a variant of a union, an
/// element of an array both below and above the size at which an array's buffer is aligned, and a
/// value a closure captures.
#[test]
pub fn test_a_128_bit_value_keeps_its_bits_in_every_container() {
    test_source(
        r#"
        module Main;

        type Boxed = box struct { tag : U8, value : I128 };
        type Unboxed = unbox struct { tag : U8, value : U128 };
        type Wide = union { narrow : U8, wide : U128 };

        main : IO ();
        main = (
            let args = *get_args;
            let zero = args.@size - 1;
            let w = 18446744073709551616_I128 + zero.i128;

            let boxed = Boxed { tag : 1_U8, value : -w };
            assert_eq(|_|"A field of a boxed struct", boxed.@value, -18446744073709551616_I128);;
            let unboxed = Unboxed { tag : 1_U8, value : U128::maximum - w.u128 };
            assert_eq(|_|"A field of an unbox struct", unboxed.@value, 340282366920938463444927863358058659839_U128);;
            let wide = Wide::wide(U128::maximum - w.u128);
            assert_eq(|_|"A variant of a union", wide.as_wide, 340282366920938463444927863358058659839_U128);;
            let some = Option::some(-w);
            assert_eq(|_|"A variant of Option", some.as_some, -18446744073709551616_I128);;

            let small = [w, -w, w + 1_I128];
            assert_eq(|_|"An element of a small array", small.@(1), -18446744073709551616_I128);;
            let large = Array::from_map(1000, |k| k.i128 * w - 1_I128);
            assert_eq(|_|"An element of a large array", large.@(999), 18428297329635842064383_I128);;
            let pushed = large.push_back(-w);
            assert_eq(|_|"An element pushed onto an array", pushed.@(1000), -18446744073709551616_I128);;

            let times = |k| k.i128 * w;
            assert_eq(|_|"A captured value", times(3), 55340232221128654848_I128);;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// An argument passed through the `...` of an `FFI_CALL` has no 128-bit type, as a declared
/// parameter has none.
#[test]
pub fn test_ffi_call_rejects_a_128_bit_variadic_argument() {
    test_source_fail(
        r#"
        module Main;
        main : IO ();
        main = println(FFI_CALL[CInt printf(Ptr, ...), nullptr, 1_U128].to_string);
    "#,
        Configuration::develop_mode(),
        "`Std::U128` cannot be passed through the `...` of an `FFI_CALL`. C has no standard 128-bit integer type, so pass the value as two `U64`s, its low and its high 64 bits.",
    );
}

/// `from_bytes` into a 128-bit type takes 16 bytes and nothing else: a byte array longer than that
/// is an error, as a shorter one is.
#[test]
pub fn test_the_128_bit_types_read_only_16_bytes() {
    test_source(
        r#"
        module Main;
        main : IO ();
        main = (
            let read_u128 : Array U8 -> Result ErrMsg U128 = from_bytes;
            let read_i128 : Array U8 -> Result ErrMsg I128 = from_bytes;
            assert(|_|"17 bytes into U128", read_u128(Array::fill(17, 0_U8)).is_err);;
            assert(|_|"15 bytes into I128", read_i128(Array::fill(15, 0_U8)).is_err);;
            assert(|_|"17 bytes into I128", read_i128(Array::fill(17, 0_U8)).is_err);;
            pure()
        );
    "#,
        Configuration::develop_mode(),
    );
}

/// A 128-bit value passed from one function to another reaches it intact at every optimization
/// level. An iterator over an array of structs holding one hands the value to the next closure in a
/// tail call with more arguments than fit in registers, which at `-O none` put the 128-bit value on
/// the stack.
#[test]
pub fn test_a_128_bit_value_crosses_a_tail_call_at_every_level() {
    let source = r#"
        module Main;

        type Pair = unbox struct { a : U8, b : I128 };

        main : IO ();
        main = (
            let args = *get_args;
            let zero = args.@size - 1;
            let pairs = Array::from_map(3 + zero, |i| Pair { a : i.u8, b : i.i128 });
            let text = pairs.to_iter.map(|p| p.@a.to_string + ":" + p.@b.to_string).join(",");
            assert_eq(|_|"The pairs as text", text, "0:0,1:1,2:2");;
            let tuples = Array::from_map(3 + zero, |k| (k.u8, k.i128));
            let sum = tuples.to_iter.fold(0_I128, |(_, x), acc| acc + x);
            assert_eq(|_|"The sum of the 128-bit values", sum, 3_I128);;
            pure()
        );
    "#;
    for opt_level in [
        FixOptimizationLevel::None,
        FixOptimizationLevel::Basic,
        FixOptimizationLevel::Max,
    ] {
        let mut config = Configuration::develop_mode();
        config.set_fix_opt_level(opt_level);
        test_source(source, config);
    }
}

/// The Fix source of a program that passes 128-bit integers to functions in every shape a value is
/// carried in: alone, beside narrower fields, in a union's payload, and in a struct of more
/// scalars than a value is split into, which is carried whole.
fn source_passing_128_bit_integers_in_every_shape() -> String {
    // 65 fields of two words each hold 130 scalars, past the 128 a value is split into.
    let field_count = 65;
    let fields = (0..field_count)
        .map(|i| format!("f{} : I128", i))
        .collect::<Vec<_>>()
        .join(", ");
    let field_values = (0..field_count)
        .map(|i| format!("f{} : zero.i128 + {}_I128", i, i))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
        module Main;

        type Wide = unbox struct {{ {} }};

        sum_pair : (U8, I128) -> I128;
        sum_pair = |(a, b)| a.i128 + b;

        sum_option : Option (I128, I128) -> I128;
        sum_option = |o| o.as_some.@0 + o.as_some.@1;

        sum_result : Result ErrMsg U128 -> I128;
        sum_result = |r| r.as_ok.i128;

        sum_wide : Wide -> I128;
        sum_wide = |w| w.@f0 + w.@f64;

        main : IO ();
        main = (
            let args = *get_args;
            let zero = args.@size - 1;
            let wide = Wide {{ {} }};
            let total = sum_pair((1_U8, zero.i128 + 2_I128))
                + sum_option(Option::some((zero.i128 + 3_I128, 4_I128)))
                + sum_result(Result::ok(zero.u128 + 5_U128))
                + sum_wide(wide.set_f0(wide.@f0 + 6_I128));
            assert_eq(|_|"The sum", total, 85_I128);;
            pure()
        );
    "#,
        fields, field_values
    )
}

/// A 128-bit value reaches a function intact in every shape a value is carried in, at every
/// optimization level.
#[test]
pub fn test_a_128_bit_value_reaches_a_function_in_every_shape() {
    let source = source_passing_128_bit_integers_in_every_shape();
    for opt_level in [
        FixOptimizationLevel::None,
        FixOptimizationLevel::Basic,
        FixOptimizationLevel::Max,
    ] {
        let mut config = Configuration::develop_mode();
        config.set_fix_opt_level(opt_level);
        test_source(&source, config);
    }
}

/// No function the compiler defines takes a bare `i128` argument, whatever shape the 128-bit value
/// is carried in.
///
/// LLVM 22's x86-64 backend miscompiles a call to a function that pops its own arguments, as a
/// Fix function does there, when its stack arguments hold a bare `i128` and do not add up to a
/// multiple of 16 bytes: the caller's stack pointer comes back 16 bytes lower than before the call.
/// An `i128` inside an array or a struct argument is passed correctly.
#[test]
pub fn test_no_function_takes_a_bare_128_bit_argument() {
    let ir = generated_llvm_ir(&source_passing_128_bit_integers_in_every_shape(), "none");
    let mut checked = 0;
    for line in ir.lines().filter(|line| line.starts_with("define ")) {
        // The parameter list follows the function's name, which is quoted where it holds
        // punctuation.
        let after_name = match line.split_once("@\"") {
            Some((_, rest)) => rest.split_once('"').unwrap().1,
            None => line.split_once('@').unwrap().1,
        };
        let params = &after_name[after_name.find('(').unwrap() + 1..];
        // Split the list at the commas outside the brackets of an aggregate type, and read each
        // parameter's type as its first word.
        let mut depth = 0;
        let mut param_start = 0;
        for (i, c) in params.char_indices() {
            match c {
                '(' | '{' | '[' | '<' => depth += 1,
                ')' | '}' | ']' | '>' if depth > 0 => depth -= 1,
                _ => {}
            }
            let ends_param = (c == ',' && depth == 0) || (c == ')' && depth == 0);
            if ends_param {
                let param = params[param_start..i].trim();
                assert!(
                    param.split_whitespace().next() != Some("i128"),
                    "a function takes a bare `i128` argument: {}",
                    line
                );
                param_start = i + 1;
            }
            if c == ')' && depth == 0 {
                break;
            }
        }
        checked += 1;
    }
    assert!(checked > 0, "the IR defines no function");
}
