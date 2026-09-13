//! What an arithmetic operation on an integer type assumes about its result: a signed operation
//! assumes that the result falls within the range of its type, while an unsigned one is taken
//! modulo two to the width of the type. A build made with `--check-signed-overflow` stops the
//! program where a signed result leaves the range instead.

use crate::configuration::Configuration;
use crate::tests::test_util::{generated_llvm_ir, test_source, test_source_fail};

/// The operations whose generated instruction carries the assumption, each as the name the code
/// generator gives the result and the LLVM instruction it emits. Negation is emitted as a
/// subtraction from zero.
const ARITHMETIC_OPERATIONS: &[(&str, &str)] =
    &[("add", "add"), ("sub", "sub"), ("mul", "mul"), ("neg", "sub")];

/// The widths, in bits, of the integer types of `Std`.
const INTEGER_WIDTHS: &[u32] = &[8, 16, 32, 64];

/// A program that adds, subtracts, multiplies and negates at each of the four signed integer types.
///
/// `main` forces each result with `eval`, which keeps the four operations of every type in the
/// program while leaving out the arithmetic that printing or asserting a value carries with it, so
/// that the arithmetic instructions the build emits are exactly these.
const ARITHMETIC_AT_EVERY_SIGNED_TYPE: &str = r#"
    module Main;

    arith_i8 : I8 -> I8 -> I8;
    arith_i8 = |x, y| (x + y) - (x * y) - (-x);

    arith_i16 : I16 -> I16 -> I16;
    arith_i16 = |x, y| (x + y) - (x * y) - (-x);

    arith_i32 : I32 -> I32 -> I32;
    arith_i32 = |x, y| (x + y) - (x * y) - (-x);

    arith_i64 : I64 -> I64 -> I64;
    arith_i64 = |x, y| (x + y) - (x * y) - (-x);

    main : IO ();
    main = (
        eval arith_i8(1_I8, 2_I8);
        eval arith_i16(1_I16, 2_I16);
        eval arith_i32(1_I32, 2_I32);
        eval arith_i64(1_I64, 2_I64);
        pure()
    );
"#;

/// The program of `ARITHMETIC_AT_EVERY_SIGNED_TYPE` at the four unsigned integer types.
const ARITHMETIC_AT_EVERY_UNSIGNED_TYPE: &str = r#"
    module Main;

    arith_u8 : U8 -> U8 -> U8;
    arith_u8 = |x, y| (x + y) - (x * y) - (-x);

    arith_u16 : U16 -> U16 -> U16;
    arith_u16 = |x, y| (x + y) - (x * y) - (-x);

    arith_u32 : U32 -> U32 -> U32;
    arith_u32 = |x, y| (x + y) - (x * y) - (-x);

    arith_u64 : U64 -> U64 -> U64;
    arith_u64 = |x, y| (x + y) - (x * y) - (-x);

    main : IO ();
    main = (
        eval arith_u8(1_U8, 2_U8);
        eval arith_u16(1_U16, 2_U16);
        eval arith_u32(1_U32, 2_U32);
        eval arith_u64(1_U64, 2_U64);
        pure()
    );
"#;

/// A configuration that stops the program where the result of a signed arithmetic operation leaves
/// the range of its type, as `--check-signed-overflow` leaves it.
fn overflow_checked_config() -> Configuration {
    let mut config = Configuration::develop_mode();
    config.check_signed_overflow = true;
    config
}

/// The instructions of `ir` that the code generator emitted for the operation whose result it names
/// `result_name`, as `instruction` at an integer type `bits` wide, split into those that carry the
/// assumption that the result fits the type and those that carry nothing.
///
/// The code generator names the result of each of these after the trait method it implements, which
/// is what tells them apart from the address arithmetic and the reference-count updates the runtime
/// emits around them.
fn arithmetic_instructions<'a>(
    ir: &'a str,
    result_name: &str,
    instruction: &str,
    bits: u32,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let assuming = format!("{} nsw i{} ", instruction, bits);
    let wrapping = format!("{} i{} ", instruction, bits);
    let mut with_assumption = vec![];
    let mut without_assumption = vec![];
    for line in ir.lines().map(str::trim) {
        let Some((register, operation)) = line.split_once(" = ") else {
            continue;
        };
        // LLVM appends digits to a name it has already given out, so the digits come off before the
        // name is read.
        let named = register
            .strip_prefix('%')
            .map(|name| name.trim_end_matches(|c: char| c.is_ascii_digit()));
        if named != Some(result_name) {
            continue;
        }
        if operation.starts_with(&assuming) {
            with_assumption.push(line);
        } else if operation.starts_with(&wrapping) {
            without_assumption.push(line);
        }
    }
    (with_assumption, without_assumption)
}

/// Addition, subtraction, multiplication and negation at a signed integer type are emitted as
/// instructions that carry `nsw`, at each of the four signed types.
///
/// `Document.md` states the contract those instructions carry: arithmetic on a signed integer type
/// assumes that its mathematical result falls within the range of that type, and the behavior of a
/// program that performs one whose result falls outside that range is undefined. The flag is what
/// hands that assumption to LLVM, which then folds a comparison the assumption settles.
#[test]
pub fn test_signed_arithmetic_assumes_its_result_fits_the_type() {
    // The subject is what the compiler emits, so the IR is read before the LLVM pass pipeline has
    // run over it: an optimized module also holds arithmetic LLVM itself introduced.
    let ir = generated_llvm_ir(ARITHMETIC_AT_EVERY_SIGNED_TYPE, "none");
    for (result_name, instruction) in ARITHMETIC_OPERATIONS {
        for bits in INTEGER_WIDTHS {
            let (with_assumption, without_assumption) =
                arithmetic_instructions(&ir, result_name, instruction, *bits);
            assert!(
                !with_assumption.is_empty(),
                "`{}` at the signed integer type of {} bits should be emitted carrying `nsw`, but \
                 no such instruction was emitted",
                result_name,
                bits
            );
            assert!(
                without_assumption.is_empty(),
                "every `{}` at the signed integer type of {} bits should carry `nsw`, but {} of \
                 them were emitted without it:\n{}",
                result_name,
                bits,
                without_assumption.len(),
                without_assumption.join("\n")
            );
        }
    }
}

/// The same four operations at an unsigned integer type are emitted carrying nothing: an unsigned
/// operation is taken modulo two to the width of the type, so its result is defined wherever it
/// falls, and `Document.md` states that a program may rely on it.
#[test]
pub fn test_unsigned_arithmetic_assumes_nothing_about_its_result() {
    let ir = generated_llvm_ir(ARITHMETIC_AT_EVERY_UNSIGNED_TYPE, "none");
    for (result_name, instruction) in ARITHMETIC_OPERATIONS {
        for bits in INTEGER_WIDTHS {
            let (with_assumption, without_assumption) =
                arithmetic_instructions(&ir, result_name, instruction, *bits);
            assert!(
                !without_assumption.is_empty(),
                "`{}` at the unsigned integer type of {} bits should be emitted, but no such \
                 instruction was emitted",
                result_name,
                bits
            );
            assert!(
                with_assumption.is_empty(),
                "`{}` at the unsigned integer type of {} bits wraps, so it should carry no `nsw`, \
                 but {} of them were emitted with it:\n{}",
                result_name,
                bits,
                with_assumption.len(),
                with_assumption.join("\n")
            );
        }
    }
}

/// A sum past the greatest value of a signed integer type stops the program, and the message
/// reports the operation and both operands.
#[test]
pub fn test_signed_overflow_check_stops_an_addition_past_the_greatest() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            eval I64::maximum + 1;
            pure()
        );
    "#;
    test_source_fail(
        &source,
        overflow_checked_config(),
        "Signed integer overflow: I64 addition, with 9223372036854775807 and 1",
    );
}

/// A product past the greatest value stops the program. Multiplication reaches a different LLVM
/// intrinsic from addition, so it is read on its own.
#[test]
pub fn test_signed_overflow_check_stops_a_product_past_the_greatest() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            eval I32::maximum * 2_I32;
            pure()
        );
    "#;
    test_source_fail(
        &source,
        overflow_checked_config(),
        "Signed integer overflow: I32 multiplication, with 2147483647 and 2",
    );
}

/// Negating the least value stops the program: its magnitude is one past the greatest. Negation is
/// emitted as a subtraction from zero, which is the shape the report names.
#[test]
pub fn test_signed_overflow_check_stops_the_negation_of_the_least() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            eval -(I64::minimum);
            pure()
        );
    "#;
    test_source_fail(
        &source,
        overflow_checked_config(),
        "Signed integer overflow: I64 negation, with 0 and -9223372036854775808",
    );
}

/// Dividing the least value by -1 stops the program: the quotient is one past the greatest. A
/// division carries no LLVM intrinsic reporting the overflow, so the check compares the operands
/// against the one pair that overflows.
#[test]
pub fn test_signed_overflow_check_stops_dividing_the_least_by_minus_one() {
    let source = r#"
        module Main;
        main : IO ();
        main = (
            eval I64::minimum / -1;
            pure()
        );
    "#;
    test_source_fail(
        &source,
        overflow_checked_config(),
        "Signed integer overflow: I64 division, with -9223372036854775808 and -1",
    );
}

/// The check leaves unsigned arithmetic alone: wrapping is what an unsigned integer type promises,
/// so a build that stops at a signed overflow computes an unsigned result and carries on.
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
