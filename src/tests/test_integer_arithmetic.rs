//! What `--check-signed-overflow` stops the program at: an arithmetic operation on a signed integer
//! type whose mathematical result leaves the range of that type.

use crate::configuration::Configuration;
use crate::tests::test_util::{test_source, test_source_fail};

/// A configuration that stops the program where the result of a signed arithmetic operation leaves
/// the range of its type, as `--check-signed-overflow` leaves it.
fn overflow_checked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.check_signed_overflow = true;
    config
}

/// Builds a program that evaluates `expression` under a configuration that stops at a signed
/// overflow, runs it, and asserts that it stops with a report containing `report`.
fn assert_the_check_stops(expression: &str, report: &str) {
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
    test_source_fail(&source, overflow_checked_config(), report);
}

/// A sum past the greatest value of a signed integer type stops the program, and the message
/// reports the operation and both operands.
#[test]
pub fn test_signed_overflow_check_stops_an_addition_past_the_greatest() {
    assert_the_check_stops(
        "I64::maximum + 1",
        "Signed integer overflow: I64 addition, with 9223372036854775807 and 1",
    );
}

/// A product past the greatest value stops the program. Multiplication reaches a different LLVM
/// intrinsic from addition, so it is read on its own.
#[test]
pub fn test_signed_overflow_check_stops_a_product_past_the_greatest() {
    assert_the_check_stops(
        "I32::maximum * 2_I32",
        "Signed integer overflow: I32 multiplication, with 2147483647 and 2",
    );
}

/// Negating the least value stops the program: its magnitude is one past the greatest. Negation is
/// emitted as a subtraction from zero, which is the shape the report names.
#[test]
pub fn test_signed_overflow_check_stops_the_negation_of_the_least() {
    assert_the_check_stops(
        "-(I64::minimum)",
        "Signed integer overflow: I64 negation, with 0 and -9223372036854775808",
    );
}

/// Dividing the least value by -1 stops the program: the quotient is one past the greatest. A
/// division carries no LLVM intrinsic reporting the overflow, so the check compares the operands
/// against the one pair that overflows.
#[test]
pub fn test_signed_overflow_check_stops_dividing_the_least_by_minus_one() {
    assert_the_check_stops(
        "I64::minimum / -1",
        "Signed integer overflow: I64 division, with -9223372036854775808 and -1",
    );
}

/// The check leaves unsigned arithmetic alone: an unsigned operation is taken modulo two to the
/// width of its type, so a build that stops at a signed overflow computes it and carries on.
///
/// Every operation here leaves the range of the signed type of its width, and none leaves the range
/// of the unsigned one, so a check that read these as signed would stop the program at each.
#[test]
pub fn test_signed_overflow_check_leaves_unsigned_arithmetic_alone() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            assert_eq(|_|"U8 add", 127_U8 + 1_U8, 128_U8);;
            assert_eq(|_|"U8 mul", 64_U8 * 2_U8, 128_U8);;
            assert_eq(|_|"U8 neg", -(128_U8), 128_U8);;
            assert_eq(|_|"U16 add", 32767_U16 + 1_U16, 32768_U16);;
            assert_eq(|_|"U32 add", 2147483647_U32 + 1_U32, 2147483648_U32);;
            assert_eq(|_|"U64 add", 9223372036854775807_U64 + 1_U64, 9223372036854775808_U64);;
            pure()
        );
    "#;
    test_source(&source, overflow_checked_config());
}
