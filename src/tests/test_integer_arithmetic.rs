//! What `--check-integer-operations` stops the program at: arithmetic on a signed integer type
//! whose result does not fit that type.

use crate::tests::test_util::{
    integer_operations_checked_config, source_with_a_runtime_zero, test_source, test_source_fail,
};

/// Builds a program that evaluates `expression` under a configuration that stops at a signed
/// overflow, runs it, and asserts that it stops with a report containing `report`.
fn assert_the_check_stops_evaluating(expression: &str, report: &str) {
    let source = format!(
        r#"
        module Main;
        main : IO ();
        main = (
            eval {};
            pure()
        );
    "#,
        expression
    );
    test_source_fail(&source, integer_operations_checked_config(), report);
}

/// A sum past the greatest value of a signed integer type stops the program, and the message
/// reports the operation and both operands.
#[test]
pub fn test_signed_overflow_check_stops_an_addition_past_the_greatest() {
    assert_the_check_stops_evaluating(
        "I64::maximum + 1",
        "Signed integer overflow: I64 addition, with 9223372036854775807 and 1",
    );
}

/// A product past the greatest value stops the program. Multiplication reaches a different LLVM
/// intrinsic from addition, so it is read on its own.
#[test]
pub fn test_signed_overflow_check_stops_a_product_past_the_greatest() {
    assert_the_check_stops_evaluating(
        "I32::maximum * 2_I32",
        "Signed integer overflow: I32 multiplication, with 2147483647 and 2",
    );
}

/// A difference past the least value stops the program. A subtraction carries a left operand of
/// its own, where a negation subtracts from zero, and the report names the operation
/// `subtraction`.
#[test]
pub fn test_signed_overflow_check_stops_a_difference_past_the_least() {
    assert_the_check_stops_evaluating(
        "I32::minimum - 1_I32",
        "Signed integer overflow: I32 subtraction, with -2147483648 and 1",
    );
}

/// Negating the least value stops the program: its magnitude is one past the greatest. Negation is
/// emitted as a subtraction from zero, which is the shape the report names.
#[test]
pub fn test_signed_overflow_check_stops_the_negation_of_the_least() {
    assert_the_check_stops_evaluating(
        "-(I64::minimum)",
        "Signed integer overflow: I64 negation, with 0 and -9223372036854775808",
    );
}

/// Dividing the least value by -1 stops the program: the quotient is one past the greatest. A
/// division carries no LLVM intrinsic reporting the overflow, so the check compares the operands
/// against the one pair that overflows.
#[test]
pub fn test_signed_overflow_check_stops_dividing_the_least_by_minus_one() {
    assert_the_check_stops_evaluating(
        "I64::minimum / -1",
        "Signed integer overflow: I64 division, with -9223372036854775808 and -1",
    );
}

/// The check leaves unsigned arithmetic alone: an unsigned operation wraps, so a build that stops
/// at a signed overflow computes it and carries on.
///
/// Every operation here gives a result outside the range of the signed type of its width and
/// inside the range of the unsigned one, so a check reading these as signed would stop the program
/// at each. The operands are built from the run-time zero, which the compiler cannot fold the
/// arithmetic away through.
#[test]
pub fn test_signed_overflow_check_leaves_unsigned_arithmetic_alone() {
    let source = source_with_a_runtime_zero(
        r#"
            let one = (zero + 1).to_U8;
            let half8 = 127_U8 * one;
            assert_eq(|_|"U8 add", half8 + half8, 254_U8);;
            assert_eq(|_|"U8 mul", half8 * 2_U8, 254_U8);;
            assert_eq(|_|"U8 neg", -(128_U8 * one), 128_U8);;

            let half16 = 32767_U16 * one.to_U16;
            assert_eq(|_|"U16 add", half16 + half16, 65534_U16);;

            let half32 = 2147483647_U32 * one.to_U32;
            assert_eq(|_|"U32 add", half32 + half32, 4294967294_U32);;

            let half64 = 9223372036854775807_U64 * one.to_U64;
            assert_eq(|_|"U64 add", half64 + half64, 18446744073709551614_U64);;
        "#,
    );
    test_source(&source, integer_operations_checked_config());
}
