# Verification of the `unique-check-elim` tip (`f8ffebe5`)

What was run against the three commits described in `handoff.md`, and what came out.

## Test suite

`cargo test --release` at each optimization level, since the borrow of a fully unboxed field changes
code generation at every level while the `Origin` work only runs at Max.

| `FIX_MAX_OPT_LEVEL` | result |
| --- | --- |
| `none` | 968 passed, 0 failed |
| `basic` | 968 passed, 0 failed |
| default (Max) | 968 passed, 0 failed |

The four new tests in `src/tests/test_match_result_alias.rs` are in that count. Each was checked
against a deliberately reverted compiler to confirm it fails there:

- reverting the `candidates` half of `acted_on` (the objects a leaf may be) fails
  `test_match_result_alias_correctness` and `..._memory_safety` with `Invalid read of size 8`.
- reverting the `identity` half fails them as well, and independently fails `test_struct_act` and
  `test_struct_act2`.

## fixlang_minilib

Every sub-project, `fix test -O max` with `memcheck = true` injected into `[build.test]` (the field is
restored afterwards), so the whole test binary runs under valgrind.

14 of 16 clean. The two that report are the known compiler-independent ones:

- `fixlang-minilib-thread`: `definitely lost: 0`, `indirectly lost: 0`; all seven contexts are
  `pthread_create` -> `_dl_allocate_tls` -> `calloc`, glibc's per-thread TLS block, reported as
  *possibly* lost.
- `fixlang-minilib-media`: every context is inside `libpng16.so` (`png_write_sig`,
  `png_write_chunk_data`, `png_write_chunk_end`), reached through the project's own PNG FFI wrapper.

## project_euler

99 solutions. Each was built and run at `-O max` by the pre-session baseline `e0943d00` and by the
branch tip, and the outputs compared; the branch binary was then run under valgrind. `main` cannot be
used as the baseline here — these projects are pinned to the migrated cp-library revision.

| outcome | count |
| --- | --- |
| built and ran on both, outputs identical | 50 |
| built on neither | 49 |

**No miscompilation.** The one solution the comparison first flagged, `345-matrix-sum`, prints its own
elapsed time; with that line removed the two outputs are identical (`13938`).

The 49 that build on neither compiler are the pre-existing old-API/WIP set: they fail on the baseline
too, so they are not a regression from this session.

Valgrind over the 50 that ran:

| outcome | count |
| --- | --- |
| clean | 30 |
| skipped, GMP/MPFR-linked | 19 |
| not measured | 1 |

Nine solutions first died under valgrind with SIGILL — VEX cannot decode the EVEX (AVX-512)
instructions LLVM emits (`unhandled instruction bytes: 0x62 ...`). Rebuilt with
`--disable-cpu-feature 'avx512.*'` they are clean; this is a valgrind limitation, not a property of
the generated code. The GMP-linked solutions are skipped because libgmp's hand-written assembly
reports uninitialised-value reads of its own. The one not measured, `719-number-splitting`, did not
finish in 30 minutes under valgrind; the "136 bytes definitely lost" in its partial log is live data
held by `main` at the moment the timeout's SIGTERM arrived.

## Speedtest

See `speedtest.md` next to this file.
