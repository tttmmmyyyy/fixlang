# Ryu

Ulf Adams' implementation of Ryu, the algorithm that finds the shortest decimal text which reads
back as the same binary floating point number, and of Ryu printf, which writes a number with a given
number of places the way C's `printf` does. `Std::F64::to_string` and `Std::F32::to_string` are
built on the first; `to_string_precision`, `to_string_exp` and `to_string_exp_precision` of both
types on the second.

- Source: https://github.com/ulfjack/ryu
- Revision: `4c0618b0e44f7ef027ebae05d2cc7812048f7c8f`
- License: Apache-2.0 (`LICENSE-Apache2`), or the Boost Software License 1.0 at the user's choice.
  This copy is taken under Apache-2.0.
- Papers: Ulf Adams, "Ryū: fast float-to-string conversion", PLDI 2018; and "Ryū revisited: printf
  floating point conversion", OOPSLA 2019, which `d2fixed.c` implements.

The license of this copy is also named on the compiler's third-party licenses page, since every
program `fix` builds links these objects.

## The files

The files hold the content upstream gives them. To take a newer Ryu, replace them with the
upstream files of the same names and record the revision they came from. The build carries this
directory to the C compiler through `RUNTIME_HEADERS` and `RUNTIME_SOURCES` in
`src/build/build.rs`, which `test_vendored_ryu_headers_are_all_carried` and
`test_vendored_ryu_sources_are_all_compiled` hold to the files that are here.

| File | Role |
| --- | --- |
| `ryu.h` | The declarations a caller uses. `float_text.c` calls `d2s_buffered_n`, `f2s_buffered_n`, `d2fixed_buffered_n` and `d2exp_buffered_n`. |
| `d2s.c` | `double` to its shortest decimal text. |
| `f2s.c` | `float` to its shortest decimal text. |
| `d2fixed.c` | `double` to its text with a given number of places, positional (`%.*f`) or with a power of ten (`%.*e`). |
| `common.h`, `digit_table.h` | Bit and digit helpers both of the above use. |
| `d2s_intrinsics.h`, `f2s_intrinsics.h` | The 64 x 64 and 32 x 32 multiplications the algorithm rests on. |
| `d2s_full_table.h`, `f2s_full_table.h`, `d2fixed_full_table.h` | The powers of ten the algorithm looks up. |
| `d2s_small_table.h` | The powers of ten computed rather than tabulated, which `RYU_OPTIMIZE_SIZE` selects. |

`d2s.c` and `f2s.c` each define a `to_chars` of their own, so each source is compiled as a
translation unit of its own.

## The text these produce

`d2s_buffered_n` and `f2s_buffered_n` write scientific text: `1E300`, `3.333333333333333E-1`,
`-0E0`, `Infinity`, `NaN`. Fix spells a number differently — `1e300`, `0.3333333333333333`,
`-0.0`, `inf`, `nan` — so `fixruntime_f64_to_str_shortest` in `float_text.c` takes the digits and
the exponent from that text and writes Fix's spelling.

`d2fixed_buffered_n` and `d2exp_buffered_n` write the text `printf` writes for `%.*f` and `%.*e`
under the `C` locale — `3.140`, `3.140e+00` — except where the number is not finite: they write
`Infinity` and `nan`, and `fixruntime_copy_precision_text` in `float_text.c` spells the infinity
`inf`.
