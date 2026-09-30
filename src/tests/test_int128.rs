//! The 128-bit integer types `I128` and `U128`: their literals, their arithmetic and conversions,
//! their text and bytes, and the C signatures that do not take them.

use crate::configuration::Configuration;
use crate::tests::test_util::{test_source, test_source_fail, test_with_a_runtime_zero};

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
        "`Std::U128` cannot be used as an argument of an exported function.",
    );
}
