# Speedtest: old codegen vs RC-IR back end (captured at the flip)

This is a one-time record taken **just before the old AST-walking code generator was
deleted**. After that deletion the two paths can no longer be compared on the same
compiler, so the numbers below are the definitive comparison of the RC-IR back end
against the code generator it replaced.

## Method

- Every case under `benchmark/speedtest/cases/` (the 22 tracked cases), built twice from
  the same compiler binary: once with the old code generator and once with the RC-IR back
  end (then selected by the `USE_RC_IR` env var, which the flip removed).
- Build flags: `-O experimental --emit-symbols --disable-cpu-feature avx512.* --allow-preliminary-commands`.
- Metric: retired instructions (`Ir`) from `cachegrind.py` — deterministic, so the numbers
  are exact and independent of machine load. `.fixlang` was removed between the two builds
  (the RC-IR selection was not part of the object cache key).
- `delta = (new - old) / old`. A negative delta means the RC-IR back end is faster.

## Results

| case                   |       old Ir |       new Ir |   delta |
| ---------------------- | -----------: | -----------: | ------: |
| random_state           |  720,443,438 |  670,283,248 | -6.962% |
| gen_random_array       |    8,908,363 |    8,406,758 | -5.631% |
| cp_lib_prime_list      | 11,547,216,920 | 11,050,637,284 | -4.300% |
| cp_lib_unionfind       |  118,104,645 |  116,182,113 | -1.628% |
| cp_lib_segtree         |  156,732,946 |  154,826,539 | -1.216% |
| cp_lib_scc             |  181,062,265 |  179,859,005 | -0.665% |
| cp_lib_bipartite       |  247,684,984 |  246,578,132 | -0.447% |
| cp_lib_dijkstra        |  223,270,104 |  222,369,516 | -0.403% |
| array_mod              |    2,119,533 |    2,119,533 | +0.000% |
| bounds_check_indexable |  116,404,261 |  116,404,261 | +0.000% |
| index_syntax           |  468,575,146 |  468,575,146 | +0.000% |
| prime_table            |   22,497,119 |   22,497,119 | +0.000% |
| sort                   |  103,098,502 |  103,098,502 | +0.000% |
| sum_by_fix             |  655,161,462 |  655,161,464 | +0.000% |
| sum_by_fold            |    1,319,057 |    1,319,057 | +0.000% |
| sum_by_fold_cap        |    2,119,037 |    2,119,037 | +0.000% |
| sum_by_loop            |      856,217 |      856,217 | +0.000% |
| sum_by_loop_arr        |    2,319,042 |    2,319,042 | +0.000% |
| sum_by_loop_iter_cap   |    2,519,022 |    2,519,022 | +0.000% |
| cp_lib_lsegtree        | 2,218,854,064 | 2,222,335,641 | +0.157% |
| cp_lib_conv_zp         |  315,029,472 |  319,075,241 | +1.284% |
| sum_by_loop_iter       |    2,219,066 |    2,319,067 | +4.506% |

## Summary

The RC-IR back end is a net win: eight cases are meaningfully faster (up to -6.96%, and
`cp_lib_prime_list` saves ~0.5 billion instructions), eleven are at exact parity, and three
are slightly slower.

## Analysis — what drives the differences

The important invariant behind these numbers: **at the reference-counting level the RC-IR
back end never does more work than the old code generator.** In every hot function measured,
the RC-IR path emits the *same* number of retain/release operations as the old path, or
*fewer*. (Examples from `cp_lib_conv_zp`: in `Main::main#10` retains dropped from 16 to 11
with releases unchanged at 81; in `CPLib.ZP::fft#11` retains dropped from 2 to 1 and releases
from 24 to 22.) So neither the wins nor the losses come from the RC-IR path inserting extra
reference counting.

**Why most cases win or match.** Two effects:

- *Fewer reference-counting operations.* The RC-IR path's backward last-use insertion, plus
  the nullability optimization below, leaves equal-or-fewer retains/releases, which is where
  the -4% to -7% cases come from.
- *The non-null-capture release skip.* A closure whose capture is non-empty holds a real
  (non-null) allocation, so releasing that capture can skip the dynamic-object null check the
  general release path emits (`release_nonnull_boxed`). This is what moved the `sort`
  comparator — whose whole cost was that per-call null check — from +1.45% to parity, and it
  helps the closure-heavy cases generally. (The symmetric `retain_nonnull_boxed` exists too,
  but is inert in practice: a capture object flows linearly — projected, released, moved — so
  it is almost never *retained* on a hot path.)

**Why three cases lose despite equal-or-fewer RC operations.** The losses are entirely
downstream of reference counting: the RC-IR back end lowers to structurally different (but
reference-count-equivalent) LLVM IR, and LLVM's register allocator and instruction scheduler
react to that shape. These are the "the RC got better or stayed the same, but LLVM made the
final code slower" cases:

- **`sum_by_loop_iter` (+4.5%)** — exactly one extra instruction per loop iteration
  (+100,001 instructions over 100,000 iterations). The loop-carried accumulator is a
  single-field `{ i64 }`. The old code generator routes the loop's continue edge through a
  separate block and keeps the accumulator in one register (`add` in place); the RC-IR path
  branches straight from the closure-exit block back to the loop header with the accumulator
  `insertvalue` there, and LLVM ends up unable to coalesce it — so each iteration adds one
  `mov` to copy the accumulator back into its loop-carried register.
- **`cp_lib_conv_zp` (+1.28%)** — diffuse register-allocation and scheduling noise in the NTT,
  with no systematic per-iteration extra operation. `CPLib.ZP::fft#11` is only +0.38% (the old
  path rematerializes the modulus constant repeatedly where the RC-IR path loads it at the use
  site — roughly a wash); `Main::main#10` is +3.13% from register-allocation and stack-spill
  layout differences (the RC-IR path there even drops some null checks).
- **`cp_lib_lsegtree` (+0.16%)** — the same class of scheduling noise, near the measurement
  floor.

These three were investigated and accepted rather than "fixed": there is no extra reference
counting to remove, and reshaping the RC-IR lowering to coax LLVM's register allocator on one
microbenchmark's loop is speculative and would not generalize.
