# Ryu

Ulf Adams' implementation of Ryu, the algorithm that finds the shortest decimal text which reads
back as the same binary floating point number. `Std::F64::to_string` and `Std::F32::to_string` are
built on it.

- Source: https://github.com/ulfjack/ryu
- Revision: `4c0618b0e44f7ef027ebae05d2cc7812048f7c8f`
- License: Apache-2.0 (`LICENSE-Apache2`), or the Boost Software License 1.0 at the user's choice.
  This copy is taken under Apache-2.0.
- Paper: Ulf Adams, "Ryū: fast float-to-string conversion", PLDI 2018.

## The files

The files hold the content upstream gives them. To take a newer Ryu, replace them with the
upstream files of the same names and record the revision above.

| File | Role |
| --- | --- |
| `ryu.h` | The declarations a caller uses. `runtime.c` calls `d2s_buffered_n` and `f2s_buffered_n`. |
| `d2s.c` | `double` to its shortest decimal text. |
| `f2s.c` | `float` to its shortest decimal text. |
| `common.h`, `digit_table.h` | Bit and digit helpers both of the above use. |
| `d2s_intrinsics.h`, `f2s_intrinsics.h` | The 64 x 64 and 32 x 32 multiplications the algorithm rests on. |
| `d2s_full_table.h`, `f2s_full_table.h` | The powers of ten the algorithm looks up. |
| `d2s_small_table.h` | The powers of ten computed rather than tabulated, which `RYU_OPTIMIZE_SIZE` selects. |

`d2s.c` and `f2s.c` each define a `to_chars` of their own, so they are compiled as separate
translation units.

## The text these produce

`d2s_buffered_n` and `f2s_buffered_n` write scientific text: `1E300`, `3.333333333333333E-1`,
`-0E0`, `Infinity`, `NaN`. Fix spells a number differently — `1e300`, `0.3333333333333333`,
`-0.0`, `inf`, `nan` — so `fixruntime_f64_to_str_shortest` in `runtime.c` takes the digits and the
exponent from that text and writes Fix's spelling.
