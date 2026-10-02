# Ryu

Ulf Adams' implementation of Ryu, the algorithm that finds the shortest decimal digits which read
back as the same binary floating point number, and of Ryu printf, which writes a number with a given
number of places. `Std::F64::to_string` and `Std::F32::to_string` are built on the first;
`to_string_precision`, `to_string_exp` and `to_string_exp_precision` of both types on the second.

- Source: https://github.com/ulfjack/ryu
- Revision: `4c0618b0e44f7ef027ebae05d2cc7812048f7c8f`
- License: Apache-2.0 (`LICENSE-Apache2`), or the Boost Software License 1.0 at the user's choice.
  This copy is taken under Apache-2.0.
- Papers: Ulf Adams, "Ryū: fast float-to-string conversion", PLDI 2018; and "Ryū revisited: printf
  floating point conversion", OOPSLA 2019, which `d2fixed.c` implements.

The license of this copy is also named on the compiler's third-party licenses page, since every
program `fix` builds links these objects.

## The files

The files are upstream's, changed in three ways. Each change other than a deletion is marked in the
source with a comment that opens with `Modified from upstream Ryu by the Fix project`.

- `ryu.h` declares only the four functions `float_text.c` calls, and opens with a `#define` for
  each, which gives the function a name beginning with `fixruntime_ryu_`.
- `d2s.c` and `f2s.c` give the shortest digits as a number, through `d2s_shortest` and
  `f2s_shortest`, and `float_text.c` writes Fix's text from them. Upstream's `to_chars`, the
  `*_buffered*` functions and `copy_special_str` in `common.h` are deleted.
- `d2fixed.c` writes the power of ten as Fix writes it, `1.50e2` where upstream writes `1.50e+02`.
  The power of ten is written by `append_exponent`, which `digit_table.h` gains together with
  `exponent_length`, and which `float_text.c` calls as well. A number given to `d2fixed.c` is
  finite, which it asserts, where upstream writes `Infinity` and `nan`. Upstream's
  `copy_special_str_printf`, `d2fixed`, `d2fixed_buffered`, `d2exp` and `d2exp_buffered` are
  deleted.

To take a newer Ryu, diff upstream's files against the revision recorded above, and carry the
differences into these files. The build carries this directory to the C compiler through
`RUNTIME_HEADERS` and `RUNTIME_SOURCES` in `src/build/build.rs`, which
`test_vendored_headers_are_all_carried` and `test_vendored_sources_are_all_compiled` hold to the
files that are here.

The renaming keeps Ryu's names out of the link of a program, which may carry Ryu on its own and
define the same names. `test_runtime_defines_only_fixruntime_names` fails on a function `ryu.h`
declares without a `#define`.

| File | Role |
| --- | --- |
| `ryu.h` | The declarations `float_text.c` uses: `d2s_shortest`, `f2s_shortest`, `d2fixed_buffered_n` and `d2exp_buffered_n`. |
| `d2s.c` | `double` to its shortest decimal digits. |
| `f2s.c` | `float` to its shortest decimal digits. |
| `d2fixed.c` | `double` to its text with a given number of places, positional or with a power of ten. |
| `common.h`, `digit_table.h` | Bit and digit helpers the sources above use. |
| `d2s_intrinsics.h`, `f2s_intrinsics.h` | The 64 x 64 and 32 x 32 multiplications the algorithm rests on. |
| `d2s_full_table.h`, `f2s_full_table.h`, `d2fixed_full_table.h` | The powers of ten the algorithm looks up. |
| `d2s_small_table.h` | The powers of ten computed rather than tabulated, which `RYU_OPTIMIZE_SIZE` selects. |

## What these produce

`d2s_shortest` and `f2s_shortest` give the shortest digits that read back as a number and the power
of ten they are multiplied by: `{ 25, -2, 2 }` for `0.25`, which has 2 digits. From these,
`fixruntime_write_float_text` in `float_text.c` writes Fix's spelling: `0.25`, `1.0e300`.

`d2fixed_buffered_n` and `d2exp_buffered_n` write a number with a given number of places,
positionally or with a power of ten: `3.140`, `3.140e0`, and at a precision of 0, `3`, `3e0`.

`float_text.c` writes an infinity, a NaN and a zero itself, as `inf`, `-inf`, `nan`, `0.0` and
`-0.0`.
