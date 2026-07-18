# Speedtest baseline for the RC-IR simplifier / BCE work

Baseline measurements recorded **before** implementing the simplifier, so the gain
of each phase is measurable as a before/after difference on the same cases.

## Method

- Compiler: this worktree at commit `dd53ecc7` (`Add BCE / RC-IR simplifier
  implementation plan`), built `--release`.
- Build per case: `fix build -O experimental --disable-cpu-feature 'avx512.*'`
  (the flags the speedtest harness uses).
- Metric: cachegrind `Ir` (instructions retired) via
  `benchmark/speedtest/cachegrind-benchmarking/cachegrind.py`, the same number the
  harness records into `log.csv`.
- Ceiling column `noCheck_Ir`: the same build with `--no-runtime-check`, which
  deletes the `_check_range` call (but leaves every union / iterator / boxed-pointer
  plumbing in place). `check_share = (baseline - noCheck) / baseline` is therefore the
  *direct* cost of the bounds check alone. It is a floor on the simplifier's target,
  not a hard ceiling: the simplifier also removes the union, which for some shapes
  unblocks vectorization the check-only removal does not reach (see the reduction note
  below).

## Target cases (array-iteration idioms)

Each row: the iteration idiom, the array access, whether the loop-carried state holds
a boxed pointer (the design's key axis), the baseline, the check-removed ceiling, the
bounds-check share, and the design phase expected to move it.

| case | idiom | access | boxed ptr in state | baseline Ir | noCheck Ir | check share | expected mover |
| --- | --- | --- | --- | ---: | ---: | ---: | --- |
| **sum_by_range_fold** (new) | `range.fold` | read | no (arr captured) | 818,988 | 618,871 | 24.4% | Phase 1 (union removal) |
| **write_by_range_fold** (new) | `range.fold` | write | yes (threaded) | 1,660,636 | 1,360,161 | 18.1% | Phase 3 SROA + Phase 4 size-norm |
| **option_plumbing** (new) | `range.fold` | none (Option) | no | 856,221 | 856,109 | 0.0% | Phase 1 (case-of-case) |
| sum_by_fold | `to_iter.fold` | read | yes (ArrayIterator) | 1,319,007 | 1,118,888 | 15.2% | Phase 1 + Phase 3 SROA |
| sum_by_loop_iter | `to_iter.loop_iter` | read (index) | yes | 1,819,062 | 1,618,950 | 11.0% | Phase 1 + Phase 3 SROA |
| sum_by_loop_arr | `loop` | read | no (arr captured) | 262,798 | 262,683 | 0.0% | already optimized |
| arrayrw | `loop` | read+write | yes (threaded `(i,arr)`) | 2,401,768,613 | 120,568,136 | 95.0% | Phase 3 SROA + Phase 4 |
| array_mod | `range.fold` | write + read (dep) | yes (threaded) | 1,319,564 | 918,901 | 30.4% | Phase 3 + Phase 4 (no vec: dep) |
| prime_table | `loop` (nested) | write | yes (threaded) | 14,556,233 | 8,469,246 | 41.8% | Phase 3 + Phase 4 |

## What the baseline already tells us

- **The `loop` idiom with a captured array is already fully optimized.**
  `sum_by_loop_arr` shows 0% check share and ~262k Ir summing 100000 elements
  (~1.3 instructions per element counting the `from_map` construction) — the current
  back end (borrow inference + unique-check elimination) already elides the check and
  vectorizes it. This is the level the other read idioms should reach.

- **The iterator / fold idioms are the gap.** The *same* array read costs 3.1x more via
  `range.fold` (`sum_by_range_fold`, 818,988) than via `loop` (`sum_by_loop_arr`,
  262,798), and still carries a 24.4% bounds check. `to_iter.fold` (`sum_by_fold`,
  15.2%) and `to_iter.loop_iter` (`sum_by_loop_iter`, 11.0%) are likewise unoptimized.
  These are the primary simplifier targets: removing the `Option`/`RangeIterator`/
  `ArrayIterator` union (and, where the state threads a boxed pointer, scalar-replacing
  it) should bring `range.fold` down toward the `loop` level.

- **A boxed pointer in the loop-carried state is the decisive blocker.** Compare
  `sum_by_loop_arr` (array captured, all-scalar state, 0% check, already vectorized)
  against `arrayrw` (array threaded in the state tuple, 95% check, the 19.9x ceiling).
  Same `loop` idiom; the only difference is whether the array is threaded. This is
  direct evidence for the design's SROA thesis: hoist the boxed pointer out of the
  threaded state so the RC passes can borrow it read-only and stop churning its
  reference count each iteration.

- **`option_plumbing` isolates the non-array axis.** 0% check share confirms it has no
  bounds check; any win must come from case-of-case + case-of-known-constructor removing
  the `Option` union, not from BCE. Its `--no-runtime-check` ceiling equals its baseline,
  so the check is genuinely absent; whether the current compiler already collapses the
  small unboxed `Option I64` union is the open question this case measures after
  implementation.

- **Reduction vs write, on the ceiling.** For write loops with independent elements
  (`arrayrw`) removing the check alone already unblocks vectorization (95% / 19.9x),
  because the check's panic branch was the blocker. For a reduction
  (`sum_by_range_fold`) removing the check alone yields only 24.4% and no vectorization,
  because the accumulator's cross-iteration dependency plus the surviving union still
  block it. The simplifier removes the union too, so its achievable win on reductions may
  *exceed* the check-only ceiling — the `noCheck` figure is a floor there, not a cap.
  `array_mod` has a genuine element-to-element data dependency (`arr[i] += arr[i-1]`) and
  cannot vectorize at all; its win is limited to the 30.4% check removal.

## Coverage map

The suite now exercises every idiom x access x state combination the design predicts a
win for:

- read, all-scalar state: `sum_by_loop_arr` (`loop`, already optimized reference),
  `sum_by_range_fold` (`range.fold`, the Phase-1 flagship).
- read, boxed-ptr state: `sum_by_fold` (`to_iter.fold`), `sum_by_loop_iter`
  (`to_iter.loop_iter`).
- write, boxed-ptr state: `write_by_range_fold` (pure, vectorizable), `array_mod`
  (`range.fold`, data-dependent), `arrayrw` / `arrayrw_fn` (`loop`, read+write),
  `prime_table` (nested `loop`).
- non-array plumbing: `option_plumbing` (`Option` via case-of-case), `sum_by_loop`
  (`LoopState` + tuple, pure scalar).
- indexed containers: `bounds_check_indexable`, `index_syntax` (2D `Indexable`).

Candidate additions if a phase needs finer isolation (not yet added): a nested pure-read
matrix sum (does the win compose under nesting), and a `loop`-idiom pure write to pair
with `write_by_range_fold` across idioms.
