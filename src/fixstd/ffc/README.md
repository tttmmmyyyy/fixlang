# fast_float (ffc.h)

Koleman Nix's C99 port of fast_float, Daniel Lemire's reader of decimal floating point text.
`Std::F64::from_string` and `Std::F32::from_string` are built on it: it rounds the number a text
names to the nearest `double` or `float`, ties to even, and takes `.` for the point whatever locale
the program runs in.

- Source: https://github.com/kolemannix/ffc.h, a port of https://github.com/fastfloat/fast_float
- Revision: `965d4db45d03365b4c852ab34f4d5b08b53bc7b3`
- License: MIT (`LICENSE-MIT`), or the Apache License 2.0 or the Boost Software License 1.0 at the
  user's choice. This copy is taken under MIT.
- Papers: Daniel Lemire, "Number Parsing at a Gigabyte per Second", Software: Practice and
  Experience 51 (8), 2021; and Noble Mushtak and Daniel Lemire, "Fast Number Parsing Without
  Fallback", Software: Practice and Experience 53 (7), 2023.

The license of this copy is also named on the compiler's third-party licenses page, since every
program `fix` builds links the object it is compiled into.

## The files

`ffc.h` is the amalgamated header upstream publishes, with the content upstream gives it, except
that each of the 27 functions its public section declares is declared `static`. To take a newer
ffc.h, replace it with the upstream file of the same name, put `static` before each function its
public section declares, and record the revision it came from. The build carries this directory to
the C compiler through `RUNTIME_INCLUDED_FILES` in `src/build/build.rs`, which
`test_vendored_files_are_all_carried` holds to the files that are here.

`float_text.c` includes it with `FFC_IMPL` defined, which compiles its implementation into that
translation unit. The definitions take internal linkage from the `static` declarations, so none of
fast_float's names reaches the link of a program, which may carry ffc.h on its own.
`test_runtime_defines_only_fixruntime_names` fails on a function a newer ffc.h declares without
`static`.

Two of its macros are left as upstream sets them:

- `FFC_ROUNDS_TO_NEAREST` stays undefined. Defining it drops the check of the rounding mode that
  keeps the fastest path correct when a program has changed the mode with `fesetround`.
- `FFC_ASSERT` keeps its own definition. Its arguments carry computations the slow path needs, so
  a definition that drops them under `NDEBUG`, as `assert` does, reads wrong numbers.
