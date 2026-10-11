# fast_float (ffc.h)

The functions of `src/fixstd/std.fix` that read a floating point number from decimal text are
translated from ffc.h, Koleman Nix's C99 port of fast_float, Daniel Lemire's reader of decimal
floating point text. `Std::F64::from_string` and `Std::F32::from_string` are built on them: they
round the number a text names to the nearest number of the type, ties to even, and take `.` for the
point whatever locale the program runs in.

- Source: https://github.com/kolemannix/ffc.h, a port of https://github.com/fastfloat/fast_float
- Revision: `965d4db45d03365b4c852ab34f4d5b08b53bc7b3`
- License: MIT (`LICENSE-MIT`), or the Apache License 2.0 or the Boost Software License 1.0 at the
  user's choice. The translation is taken under MIT.
- Papers: Daniel Lemire, "Number Parsing at a Gigabyte per Second", Software: Practice and
  Experience 51 (8), 2021; and Noble Mushtak and Daniel Lemire, "Fast Number Parsing Without
  Fallback", Software: Practice and Experience 53 (7), 2023.

The license is also named on the compiler's third-party licenses page, since every program `fix`
builds may contain the translation.

## What the translation is

The translated functions are in the namespace `Std::String`, and each names the function of ffc.h
it translates in its comment. One function reads both types: the trait `Std::_BinaryFloat` gives
it the constants of a type, which ffc.h names `FFC_DOUBLE_*` and `FFC_FLOAT_*`, as a
`Std::_BinaryFormat`. The table `FFC_POWERS_OF_FIVE` is `String::_powers_of_five_128`, written as an
array literal of `Std::U64`, where it is a constant of the program.

| ffc.h | Fix |
| --- | --- |
| `ffc_from_chars` | `String::_read_float_text` |
| `ffc_parse_number_string` | `String::_parse_decimal_text`, `String::_parse_explicit_exponent`, `String::_reread_first_19_digits` |
| `ffc_parse_infnan` | `String::_read_infinity_or_nan` |
| `ffc_from_chars_advanced`, `ffc_clinger_fast_path_impl` | `String::_round_decimal_text` |
| `ffc_compute_float`, `ffc_compute_error` | `String::_compute_float`, `String::_compute_float_truncated` |
| `ffc_digit_comp`, `ffc_parse_mantissa` | `String::_round_by_digit_comparison`, `String::_parse_significant_digits` |
| `ffc_bigint` | the functions of `String` whose names contain `bigint` |

The translation changes ffc.h in these ways.

- The text is the whole of what is read, so that a number followed by anything else is malformed,
  where ffc.h answers where the number ends.
- The options are fixed to those `from_string` reads with: `FFC_PRESET_GENERAL` with
  `FFC_FORMAT_FLAG_ALLOW_LEADING_PLUS`, and `.` for the point. The parts of ffc.h that only other
  options reach are left out: the JSON and Fortran formats, the skipping of white space and another
  character for the point. So are its readers of integers, `ffc_format_double_fixed`, and its
  readers of characters wider than a byte.
- Clinger's fast path relies on the rounding mode being to nearest, as ffc.h does with
  `FFC_ROUNDS_TO_NEAREST` defined. Fix computes with floating point numbers in that mode, and
  `Document.md` leaves the results unspecified in a program that changes it.
- A big integer is an `Array U64` allocated with room for 64 limbs, where ffc.h holds 4000 bits on
  the stack, and its limbs are 64 bits on every target. Multiplying by a big integer accumulates
  the product in one array, where ffc.h's `ffc_bigint_long_mul` adds the product by each limb into
  a copy.

To take a newer ffc.h, diff upstream's file against the revision recorded above, and carry the
differences into the translation.
