# Ryu

The functions of `src/fixstd/std.fix` that write a floating point number as decimal text are
translated from Ulf Adams' implementation of Ryu, the algorithm that finds the shortest decimal
digits which read back as the same binary floating point number, and of Ryu printf, which writes a
number with a given number of places. `Std::F64::to_string` and `Std::F32::to_string` are built on
the first; `to_string_precision`, `to_string_exp` and `to_string_exp_precision` of both types on the
second.

- Source: https://github.com/ulfjack/ryu
- Revision: `4c0618b0e44f7ef027ebae05d2cc7812048f7c8f`
- License: Apache-2.0 (`LICENSE-Apache2`), or the Boost Software License 1.0 at the user's choice.
  The translation is taken under Apache-2.0.
- Papers: Ulf Adams, "Ryū: fast float-to-string conversion", PLDI 2018; and "Ryū revisited: printf
  floating point conversion", OOPSLA 2019, which `d2fixed.c` implements.

The license is also named on the compiler's third-party licenses page, since every program `fix`
builds may contain the translation.

## What the translation is

The translated functions are in the namespaces `Std::F64` and `Std::F32`, and each names the Ryu
function it translates in its comment. The tables are Ryu's `d2s_full_table.h` and
`d2fixed_full_table.h`, written as array literals of `Std::F64`, where they are constants of the
program.

| Ryu | Fix |
| --- | --- |
| `d2d`, `d2s_shortest` (`d2s.c`) | `F64::_find_shortest_decimal_digits`, `F64::_find_shortest_decimal` |
| `f2d`, `f2s_shortest` (`f2s.c`) | `F32::_find_shortest_decimal` |
| `d2fixed_buffered_n` (`d2fixed.c`) | `F64::_write_fixed_text` |
| `d2exp_buffered_n` (`d2fixed.c`) | `F64::_write_exp_text` |
| the helpers of `common.h`, `d2s_intrinsics.h`, `f2s_intrinsics.h` | the functions of `F64` and `F32` that name them |

The translation changes Ryu in these ways.

- The shortest digits are answered as a number, its power of ten and its digit count, and
  `String::_from_float_decimal` writes Fix's text from them: `0.25`, `1.0e300`. Upstream's
  `to_chars` writes `2.5E-1` and `1E300`.
- `F64::_write_exp_text` writes the power of ten as Fix writes it, `1.50e2` where upstream writes
  `1.50e+02`.
- The functions take a finite number. `to_string` and the precision functions write an infinity, a
  NaN and a zero themselves, as `inf`, `-inf`, `nan`, `0.0` and `-0.0`.
- Ryu's `RYU_OPTIMIZE_SIZE` and `RYU_FLOAT_FULL_TABLE` configurations are left out: the tables are
  the full ones, and `F32` reads those of `F64`, as Ryu does without `RYU_FLOAT_FULL_TABLE`.

To take a newer Ryu, diff upstream's files against the revision recorded above, and carry the
differences into the translation.
