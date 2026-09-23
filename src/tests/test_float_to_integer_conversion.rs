//! What a conversion from a floating-point type to an integer type answers for a value the target
//! type does not hold: a NaN, an infinity, and a finite value beyond an end of the range.

use crate::configuration::Configuration;
use crate::tests::test_util::{
    integer_operations_checked_config, source_with_a_runtime_zero, test_source, test_source_fail,
};

/// Builds `source_with_a_runtime_zero(body)` under `config`, runs it, and fails the test unless the
/// program exits with code 0.
fn test_with_a_runtime_zero(body: &str, config: Configuration) {
    test_source(&source_with_a_runtime_zero(body), config);
}

/// Builds `source_with_a_runtime_zero(body)` under a configuration that stops at a value the target
/// type does not hold, runs it, and asserts that it stops with a report containing `report`.
fn assert_the_check_stops_running(body: &str, report: &str) {
    test_source_fail(
        &source_with_a_runtime_zero(body),
        integer_operations_checked_config(),
        report,
    );
}

/// The Fix source that binds `one`, `nan` and `infinite` to values the compiler cannot fold, so
/// that the conversions below are performed at run time.
const BINDINGS_BUILT_AT_RUN_TIME: &str = r#"
    let one = zero.f64 + 1.0;
    let nan = (one - one) / (one - one);
    let infinite = one / (one - one);
"#;

/// A conversion of a value the target type does not hold answers one value: what the result
/// compares as and what it prints as agree.
///
/// `fptosi` answers `poison` for such a value, and a `poison` is a permission to take any value, so
/// two readers of one conversion may take different answers from it — a value that compares as zero
/// and prints as the least value of the type.
#[test]
pub fn test_a_conversion_out_of_range_answers_one_value() {
    test_with_a_runtime_zero(
        &format!(
            r#"
            {}
            let x = nan.i64;
            assert_eq(
                |_|"What the value compares as and what it prints as",
                x < 0,
                x.to_string.get_bytes.@(0) == '-'
            );;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        Configuration::develop_mode(),
    );
}

/// A value beyond an end of the target type's range converts to that end, and a NaN converts to
/// zero.
///
/// The documentation of the conversion leaves these answers unspecified, so this pins the answer
/// the code generator gives rather than one a program may rely on.
#[test]
pub fn test_what_a_conversion_out_of_range_answers() {
    test_with_a_runtime_zero(
        &format!(
            r#"
            {}
            assert_eq(|_|"A NaN converts to zero", nan.i64, 0);;
            assert_eq(|_|"A NaN converts to zero in an unsigned type", nan.u64, 0_U64);;
            assert_eq(|_|"A NaN converts to zero in a narrow type", nan.i32, 0_I32);;
            assert_eq(
                |_|"A value above the range converts to the greatest value of the type",
                infinite.i64,
                I64::maximum
            );;
            assert_eq(
                |_|"A value below the range converts to the least value of the type",
                (-infinite).i64,
                I64::minimum
            );;
            assert_eq(
                |_|"A negative value converts to zero in an unsigned type",
                (-one).u8,
                0_U8
            );;
            assert_eq(
                |_|"A finite value above the range converts to the greatest value of the type",
                (one * 1.0e30).i64,
                I64::maximum
            );;
            assert_eq(
                |_|"A value above the range converts to the greatest value of an unsigned type",
                infinite.u64,
                U64::maximum
            );;
            assert_eq(
                |_|"A finite value above the range converts to the greatest value of a narrow unsigned type",
                (one * 1000.0).u8,
                U8::maximum
            );;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        Configuration::develop_mode(),
    );
}

/// The greatest value of an integer type, sent through a floating-point type and back, returns as
/// the greatest value.
///
/// `F64` has no exact form for `I64::maximum` and rounds it to `2^63`, which `I64` does not hold,
/// so the conversion back is one of the out-of-range cases; the round trip used to return the least
/// value of the type.
#[test]
pub fn test_the_round_trip_of_the_greatest_value() {
    test_with_a_runtime_zero(
        r#"
            assert_eq(
                |_|"I64::maximum through F64 and back",
                (I64::maximum + zero).f64.i64,
                I64::maximum
            );;
            assert_eq(
                |_|"I32::maximum through F32 and back",
                (I32::maximum + zero.i32).f32.i32,
                I32::maximum
            );;
        "#,
        Configuration::develop_mode(),
    );
}

/// A value whose rounded form lies inside the range of the target type converts to that rounded
/// value, and the check lets it through.
///
/// The values here sit against the ends of their ranges, where rounding decides the answer:
/// `-128.5` rounds towards zero to `-128`, which `I8` holds. A check reading the value before
/// rounding would stop every one of them.
#[test]
pub fn test_a_value_rounding_into_range_is_converted_and_passes_the_check() {
    let body = r#"
        let z = zero.f64;
        assert_eq(|_|"-128.5 into I8", (-128.5 + z).i8, -128_I8);;
        assert_eq(|_|"127.9 into I8", (127.9 + z).i8, 127_I8);;
        assert_eq(|_|"255.9 into U8", (255.9 + z).u8, 255_U8);;
        assert_eq(|_|"-0.9 into U8", (-0.9 + z).u8, 0_U8);;
        assert_eq(
            |_|"-2147483648.5 into I32",
            (-2147483648.5 + z).i32,
            I32::minimum
        );;
        assert_eq(|_|"3.7 into I64", (3.7 + z).i64, 3);;
        assert_eq(|_|"-3.7 into I64", (-3.7 + z).i64, -3);;
    "#;
    test_with_a_runtime_zero(body, Configuration::develop_mode());
    test_with_a_runtime_zero(body, integer_operations_checked_config());
}

/// The check stops the program at a NaN, and the report names the conversion and the value.
#[test]
pub fn test_the_check_stops_a_conversion_of_a_nan() {
    assert_the_check_stops_running(
        &format!(
            r#"
            {}
            eval nan.i64;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        "Floating-point value outside the range of the integer type: F64 to I64, with nan",
    );
}

/// The check stops the program at a value above the range of the target type.
#[test]
pub fn test_the_check_stops_a_conversion_above_the_range() {
    assert_the_check_stops_running(
        &format!(
            r#"
            {}
            eval infinite.i64;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        "Floating-point value outside the range of the integer type: F64 to I64, with inf",
    );
}

/// The check stops the program where an unsigned type receives a negative value, which is outside
/// its range although it is inside the range of the signed type of the same width.
#[test]
pub fn test_the_check_stops_a_negative_value_into_an_unsigned_type() {
    assert_the_check_stops_running(
        &format!(
            r#"
            {}
            eval (-one).u8;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        "Floating-point value outside the range of the integer type: F64 to U8, with -1",
    );
}

/// The check names the floating-point type the value came from, so a conversion from `F32` reports
/// `F32` rather than the type the report widens the value to.
#[test]
pub fn test_the_check_names_the_source_type() {
    assert_the_check_stops_running(
        r#"
            let one = zero.f32 + 1.0_F32;
            let nan = (one - one) / (one - one);
            eval nan.i32;
        "#,
        "Floating-point value outside the range of the integer type: F32 to I32, with nan",
    );
}

/// The check stops the program at a value above the range of an unsigned type, whose greatest
/// value lies below the power of two of its whole width rather than of one less.
#[test]
pub fn test_the_check_stops_a_value_above_an_unsigned_range() {
    assert_the_check_stops_running(
        &format!(
            r#"
            {}
            eval (one * 1000.0).u8;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        "Floating-point value outside the range of the integer type: F64 to U8, with 1000",
    );
}

/// The check stops the program at the power of two the range ends below, which is the value
/// `I64::maximum` takes in `F64`.
///
/// A bound built from `I64::maximum` would be rounded to that same power of two and let the value
/// through, leaving saturation to answer a conversion the type has no answer for.
#[test]
pub fn test_the_check_stops_the_power_of_two_the_range_ends_below() {
    assert_the_check_stops_running(
        "eval (I64::maximum + zero).f64.i64;",
        "Floating-point value outside the range of the integer type: F64 to I64",
    );
}

/// The values at the ends of each type's range pass the check and convert to themselves.
///
/// A bound placed one power of two too low — `2^(w-1)` for an unsigned type of `w` bits — or a
/// comparison that excluded the end of the range would stop one of these. `9223372036854774784`
/// and `18446744073709549568` are the greatest values of `F64` below `2^63` and `2^64`.
#[test]
pub fn test_the_check_lets_the_ends_of_the_range_through() {
    test_with_a_runtime_zero(
        r#"
            let z = zero.f64;
            assert_eq(
                |_|"The greatest F64 below 2^63 into I64",
                (9223372036854774784.0 + z).i64,
                9223372036854774784
            );;
            assert_eq(
                |_|"The greatest F64 below 2^64 into U64",
                (18446744073709549568.0 + z).u64,
                18446744073709549568_U64
            );;
            assert_eq(|_|"The greatest value of U32", (4294967295.0 + z).u32, U32::maximum);;
            assert_eq(|_|"The least value of I16", (-32768.0 + z).i16, I16::minimum);;
            assert_eq(|_|"The greatest value of U16", (65535.9 + z).u16, U16::maximum);;
        "#,
        integer_operations_checked_config(),
    );
}

/// A configuration that asks for the conversion check and then leaves out every check that ends
/// the program, as `--check-integer-operations --no-runtime-check` does.
fn integer_operations_checked_runtime_unchecked_config() -> Configuration {
    let mut config = integer_operations_checked_config();
    config.no_runtime_check = true;
    config
}

/// `--no-runtime-check` takes the conversion check out with the rest of the checks that end the
/// program, so a build given both it and `--check-integer-operations` runs on at a value the
/// target type does not hold.
#[test]
pub fn test_the_check_respects_no_runtime_check() {
    test_with_a_runtime_zero(
        &format!(
            r#"
            {}
            assert_eq(|_|"A NaN converts to zero", nan.i64, 0);;
        "#,
            BINDINGS_BUILT_AT_RUN_TIME
        ),
        integer_operations_checked_runtime_unchecked_config(),
    );
}
