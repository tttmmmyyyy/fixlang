//! What `shift_left` and `shift_right` answer where the shift amount is outside the range a shift
//! is defined on, which is from zero up to the number of bits of the type shifted.

use crate::configuration::Configuration;
use crate::tests::test_util::{test_source, test_source_fail};

/// A program whose `main` binds `zero` to a `Std::I64` that is 0 at run time and that no
/// optimization level can fold, and runs `body` after it.
///
/// A shift amount built from `zero` reaches the code generator as a value, so the amount a case
/// names is the amount the shift instruction receives. An amount written as a literal is folded
/// long before that, and the shift then answers at compile time whatever the folding chose.
fn source_with_a_runtime_zero(body: &str) -> String {
    format!(
        r#"
        module Main;
        main : IO ();
        main = (
            let args = *get_args;
            assert_eq(|_|"The test program is run with its own path alone", args.@size, 1);;
            let zero = args.@size - 1;
            {}
            pure()
        );
    "#,
        body
    )
}

/// Builds `source_with_a_runtime_zero(body)` under `config`, runs it, and fails the test unless the
/// program exits with code 0.
fn test_with_a_runtime_zero(body: &str, config: Configuration) {
    test_source(&source_with_a_runtime_zero(body), config);
}

/// A configuration that stops the program where the amount of a shift is outside the range the
/// shift is defined on, as `--check-shift-amount` asks for.
fn shift_amount_checked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.check_shift_amount = true;
    config
}

/// A configuration that asks for the shift amount check and then leaves out every check that ends
/// the program, as `--check-shift-amount --no-runtime-check` does.
fn shift_amount_unchecked_config() -> Configuration {
    let mut config = shift_amount_checked_config();
    config.no_runtime_check = true;
    config
}

/// Builds `source_with_a_runtime_zero(body)` under a configuration that stops at a shift amount
/// outside the width, runs it, and asserts that it stops with a report containing `report`.
fn assert_the_check_stops(body: &str, report: &str) {
    test_source_fail(
        &source_with_a_runtime_zero(body),
        shift_amount_checked_config(),
        report,
    );
}

/// A shift by an amount the type has no room for answers one value: what the result compares as
/// and what it prints as agree.
///
/// `shl` answers `poison` where the amount reaches the width of the value, and a `poison` is a
/// permission to take any value rather than a value, so two readers of one shift may take
/// different answers from it — a value that compares as positive and prints with a leading `-`.
#[test]
pub fn test_a_shift_past_the_width_answers_one_value() {
    test_with_a_runtime_zero(
        r#"
            let x = 1.shift_left(zero - 1);
            assert_eq(
                |_|"What the value compares as and what it prints as",
                x < 0,
                x.to_string.get_bytes.@(0) == '-'
            );;
        "#,
        Configuration::develop_mode(),
    );
}

/// The shift amount is taken modulo the width of the type, so an amount of the width itself leaves
/// the value where it is and an amount one past it moves the value one bit.
///
/// The documentation of `shift_left` and `shift_right` leaves the answer to an amount outside that
/// range unspecified, so this pins the answer the code generator gives rather than one a program
/// may rely on.
#[test]
pub fn test_a_shift_amount_is_taken_modulo_the_width() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(|_|"I64 left by its width", 1.shift_left(zero + 64), 1);;
            assert_eq(|_|"I64 left by one past its width", 1.shift_left(zero + 65), 2);;
            assert_eq(
                |_|"I64 right by its width",
                I64::minimum.shift_right(zero + 64),
                I64::minimum
            );;
        "#,
        Configuration::develop_mode(),
    );
}

/// A negative shift amount is taken modulo the width like any other, so -1 moves the value by one
/// less than the width.
///
/// The amount of a shift carries the type of the value shifted, so every signed integer type
/// accepts one.
#[test]
pub fn test_a_negative_shift_amount_is_taken_modulo_the_width() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(|_|"I64 left by -1", 1.shift_left(zero - 1), I64::minimum);;
            assert_eq(|_|"I32 left by -1", 1_I32.shift_left((zero - 1).to_I32), I32::minimum);;
        "#,
        Configuration::develop_mode(),
    );
}

/// The types narrower than a machine register take their own width, not the register's.
///
/// A machine's shift instruction masks the amount by the width of the register holding the value,
/// which is wider than `I8` and `I16`, so an amount of 8 shifting an `I8` leaves the value where it
/// is only because the code generator masks it.
#[test]
pub fn test_a_type_narrower_than_a_register_takes_its_own_width() {
    test_with_a_runtime_zero(
        r#"
            let z8 = zero.to_I8;
            assert_eq(|_|"I8 left by its width", 1_I8.shift_left(z8 + 8_I8), 1_I8);;
            assert_eq(|_|"I8 left by one past its width", 1_I8.shift_left(z8 + 9_I8), 2_I8);;
            let zu8 = zero.to_U8;
            assert_eq(|_|"U8 right by its width", 128_U8.shift_right(zu8 + 8_U8), 128_U8);;
            let z16 = zero.to_I16;
            assert_eq(|_|"I16 left by its width", 1_I16.shift_left(z16 + 16_I16), 1_I16);;
        "#,
        Configuration::develop_mode(),
    );
}

/// The check stops the program where the amount reaches the width of the type, and the report names
/// the operation and the amount.
#[test]
pub fn test_the_check_stops_a_shift_by_the_width() {
    assert_the_check_stops(
        "eval 1.shift_left(zero + 64);",
        "Shift amount outside the width of the type: I64 shift_left, with 64",
    );
}

/// The check stops the program at a negative amount, and the report shows the amount the program
/// wrote rather than the bit pattern the comparison reads.
#[test]
pub fn test_the_check_stops_a_negative_shift_amount() {
    assert_the_check_stops(
        "eval 1.shift_left(zero - 1);",
        "Shift amount outside the width of the type: I64 shift_left, with -1",
    );
}

/// The check covers an unsigned type and a shift towards the least bit, and the report names which
/// shift it stopped.
///
/// `--check-signed-overflow` leaves an unsigned type alone, because arithmetic on one is taken
/// modulo two to its width and so has no result outside the type. A shift amount is outside the
/// width for either signedness.
#[test]
pub fn test_the_check_stops_a_shift_of_an_unsigned_type() {
    assert_the_check_stops(
        "eval 1_U8.shift_right(zero.to_U8 + 8_U8);",
        "Shift amount outside the width of the type: U8 shift_right, with 8",
    );
}

/// The report shows the amount the program wrote, so it widens the amount to 64 bits by the
/// signedness of its type: a negative amount of a signed type reads as the negative number, and an
/// amount of an unsigned type as the magnitude its bits hold.
///
/// A type as wide as the report's own 64 bits leaves the two widenings the same value, so the type
/// here is narrower than that.
#[test]
pub fn test_the_report_widens_the_amount_by_the_signedness_of_its_type() {
    assert_the_check_stops(
        "eval 1_I8.shift_left(zero.to_I8 - 1_I8);",
        "Shift amount outside the width of the type: I8 shift_left, with -1",
    );
    assert_the_check_stops(
        "eval 1_U8.shift_right(zero.to_U8 - 1_U8);",
        "Shift amount outside the width of the type: U8 shift_right, with 255",
    );
}

/// An amount inside the width runs on under the check.
#[test]
pub fn test_the_check_lets_an_amount_inside_the_width_run_on() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(|_|"I64 left by one less than its width", 1.shift_left(zero + 63), I64::minimum);;
            assert_eq(|_|"I64 left by zero", 1.shift_left(zero), 1);;
            assert_eq(|_|"U8 right by one less than its width", 128_U8.shift_right(zero.to_U8 + 7_U8), 1_U8);;
        "#,
        shift_amount_checked_config(),
    );
}

/// `--no-runtime-check` takes the shift amount check out with the rest of the checks that end the
/// program, so a build given both it and `--check-shift-amount` runs on at an amount outside the
/// width.
#[test]
pub fn test_the_check_respects_no_runtime_check() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(|_|"I64 left by its width", 1.shift_left(zero + 64), 1);;
        "#,
        shift_amount_unchecked_config(),
    );
}
