//! `count_leading_zeros`, `count_trailing_zeros` and `count_ones`: the counts of an integer's bits, at every
//! integer type.

use crate::configuration::Configuration;
use crate::tests::test_util::test_with_a_runtime_zero;

/// Each count answers at every integer type, as a value of that type, and a zero operand has as
/// many leading and trailing zeros as its type has bits. The operands are built from the run-time
/// zero, so the counts are computed by the program rather than folded.
#[test]
pub fn test_the_bit_counts_of_every_integer_type() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(|_|"U8 count_leading_zeros", (1_U8 + zero.u8).count_leading_zeros, 7_U8);;
            assert_eq(|_|"U8 count_trailing_zeros", (12_U8 + zero.u8).count_trailing_zeros, 2_U8);;
            assert_eq(|_|"U8 count_ones", (11_U8 + zero.u8).count_ones, 3_U8);;
            assert_eq(|_|"I8 of zero", (zero.i8.count_leading_zeros, zero.i8.count_trailing_zeros, zero.i8.count_ones), (8_I8, 8_I8, 0_I8));;
            assert_eq(|_|"I8 of -1", ((zero.i8 - 1_I8).count_leading_zeros, (zero.i8 - 1_I8).count_ones), (0_I8, 8_I8));;
            assert_eq(|_|"I8::minimum count_trailing_zeros", (I8::minimum + zero.i8).count_trailing_zeros, 7_I8);;
            assert_eq(|_|"U16 count_leading_zeros", (256_U16 + zero.u16).count_leading_zeros, 7_U16);;
            assert_eq(|_|"I16 count_ones", (I16::maximum + zero.i16).count_ones, 15_I16);;
            assert_eq(|_|"U32 count_trailing_zeros", zero.u32.count_trailing_zeros, 32_U32);;
            assert_eq(|_|"I32 count_leading_zeros", (1_I32 + zero.i32).count_leading_zeros, 31_I32);;
            assert_eq(|_|"I64 count_leading_zeros of zero", zero.count_leading_zeros, 64);;
            assert_eq(|_|"I64 count_trailing_zeros", (1024 + zero).count_trailing_zeros, 10);;
            assert_eq(|_|"U64 count_ones", (U64::maximum + zero.u64).count_ones, 64_U64);;
            assert_eq(|_|"I128 count_leading_zeros of zero", zero.i128.count_leading_zeros, 128_I128);;
            assert_eq(|_|"U128 count_leading_zeros", (1_U128 + zero.u128).count_leading_zeros, 127_U128);;
            assert_eq(|_|"U128 count_trailing_zeros past 64 bits", (1_U128 + zero.u128).shift_left(100_U128).count_trailing_zeros, 100_U128);;
            assert_eq(|_|"U128 count_ones", (U128::maximum + zero.u128).count_ones, 128_U128);;
        "#,
        Configuration::develop_mode(),
    );
}

/// A count of leading zeros shifts an integer so that its highest one bit becomes the highest bit
/// of its type, without a conversion, since the count and the shift amount share the type.
#[test]
pub fn test_count_leading_zeros_normalizes_a_shift() {
    test_with_a_runtime_zero(
        r#"
            let x = 12345_U64 + zero.u64;
            assert_eq(|_|"Normalized", x.shift_left(x.count_leading_zeros), 13899234349972193280_U64);;
        "#,
        Configuration::develop_mode(),
    );
}
