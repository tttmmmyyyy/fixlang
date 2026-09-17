//! What `Std::I64::shift_left` and `Std::I64::shift_right` answer where the shift amount is
//! outside `0 <= bits < the width of the type`.

use crate::configuration::Configuration;
use crate::tests::test_util::test_source;

/// Builds a program whose `main` binds `zero` to a `Std::I64` that is 0 at run time and that no
/// optimization level can fold, runs `body` after it, and fails the test unless the program exits
/// with code 0.
///
/// A shift amount built from `zero` reaches the code generator as a value, so the amount a case
/// names is the amount the shift instruction receives. An amount written as a literal is folded
/// long before that, and the shift then answers at compile time whatever the folding chose.
fn test_with_a_runtime_zero(body: &str) {
    let source = format!(
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
    );
    test_source(&source, Configuration::develop_mode());
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
    );
}

/// The shift amount is taken modulo the width of the type, so an amount of the width itself leaves
/// the value where it is and an amount one past it moves the value one bit.
///
/// `Document.md` leaves the answer to an amount outside `0 <= bits < the width` unspecified, and
/// this pins the answer the code generator gives rather than one a program may rely on.
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
    );
}
