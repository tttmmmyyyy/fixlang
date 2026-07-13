# Post-flip verification and the `Array Bool` layout fix

A comprehensive soundness + performance verification of the RC-IR back end after the
flip and after merging `main` (which brought the `_` type wildcard and LSP work). It
records the speedtest method and numbers, and the root cause + fix of the one regression
the verification surfaced.

## CI-level tests

`cargo test --release` at every `FIX_MAX_OPT_LEVEL` (the CI matrix: `max`, `basic`,
`none`). The only failure at any level is `test_external_project_cp_library`: cp-library
`main` still imports the deleted builtin `_unsafe_swap_bounds_uniqueness_unchecked`, so it
fails to compile. This is the known by-design failure, identical at every opt level, and
`none` additionally skips the test for speed. No opt-dependent miscompilation.

## Speedtest method

Metric is retired instructions (`Ir`) from `cachegrind.py` — deterministic and
load-independent. Two comparisons, because `main` and the branch differ in Fix version:

- **`main` is Fix 1.4.0; the branch is 1.5.0.** The speedtest's `cp_lib_*` cases pin
  cp-library, whose `fixproj.toml` requires `^1.5.0`, so `main` cannot build them. A
  direct "vs main" comparison therefore covers only the 14 cases that use no such
  dependency.
- **Codegen-isolated comparison.** To compare the two code generators at a single version
  (and to cover the `cp_lib_*` cases), the old AST-walking generator was built from the
  last pre-flip commit `c4469704` (still 1.5.0, still selects the generator by the
  `USE_RC_IR` env var: unset = old, set = RC-IR). `USE_RC_IR` unset on that binary
  reproduces the old generator exactly (random_state 720,443,392 vs the flip record's
  720,443,438). New = the current HEAD binary.

Build flags for every case: `build -O experimental --emit-symbols --disable-cpu-feature
avx512.* --allow-preliminary-commands`. Object caches (`.fixlang/cache`,
`.fixlang/intermediate`) are cleared between the old and new build of each case;
`.fixlang/deps` is kept.

## Codegen-isolated results (old AST-walk vs HEAD RC-IR, both Fix 1.5.0)

Reproduces the flip record on the post-merge HEAD — the RC-IR generator has not regressed
from the merges or the code-review passes.

| case                   |       old Ir |       new Ir |   delta |
| ---------------------- | -----------: | -----------: | ------: |
| random_state           |  720,443,438 |  670,283,248 | -6.962% |
| gen_random_array       |    8,908,363 |    8,406,758 | -5.631% |
| cp_lib_prime_list      | 11,547,216,896 | 11,050,637,257 | -4.300% |
| cp_lib_unionfind       |  118,104,655 |  116,182,116 | -1.628% |
| cp_lib_segtree         |  156,732,956 |  154,826,546 | -1.216% |
| cp_lib_scc             |  181,062,440 |  179,858,998 | -0.665% |
| cp_lib_bipartite       |  247,684,981 |  246,578,132 | -0.447% |
| cp_lib_dijkstra        |  223,270,110 |  222,369,516 | -0.403% |
| eight cases            |            = |            = | +0.000% |
| cp_lib_lsegtree        | 2,218,854,064 | 2,222,335,634 | +0.157% |
| cp_lib_conv_zp         |  315,029,469 |  319,075,251 | +1.284% |
| sum_by_loop_iter       |    2,219,066 |    2,319,067 | +4.506% |

`prime_table` and `sort` are +0.000% here (codegen parity) — so their movement in the
"vs main" numbers below is not the code generator.

## vs main (Fix 1.4.0 old-codegen vs HEAD 1.5.0 RC-IR), non-`cp_lib` cases

Total upgrade effect, which also includes standard-library changes between 1.4.0 and
1.5.0:

| case              |    main Ir |  branch Ir |    delta | note |
| ----------------- | ---------: | ---------: | -------: | ---- |
| sort              | 118,937,220 | 103,098,502 | -13.317% | std: sort migrated to `swap_bounds_unchecked` |
| random_state      | 720,443,437 | 670,283,248 |  -6.962% | RC-IR |
| gen_random_array  |   8,908,363 |   8,406,758 |  -5.631% | RC-IR |
| sum_by_loop_iter  |   2,219,066 |   2,319,067 |  +4.506% | known codegen regression |
| prime_table       |  17,597,132 |  22,497,119 | +27.845% | **std/layout regression — fixed below** |
| others            |          = |          = |  +0.000% | |

## The `prime_table` regression: `Array Bool` lost its compact layout

`prime_table` is a `Bool` sieve. Its +27.8% is not the code generator (codegen-isolated
parity above) and not the opt level (identical at `-O max` and `-O experimental`). It is
a standard-library/lowering change:

- Isolated, `-O max`: a `Bool` set-only loop is +58.1% (36,160,620 → 57,160,625); the
  same loop on `Array I64` is +0.0%; reads (`@`) are +0.0%. So the regression is specific
  to writing small-element arrays.
- On `main`, LLVM packs the `Bool` element stores into `store <32 x i8>`; on the branch
  there are no packed i8 stores. Memory accesses more than doubled (53M → 119M).

Root cause: commit `dfe77970` made `Bool` an `unbox union { _false : (), _true : () }`
(sound in itself). A union's buffer takes its alignment from the payload's *preferred*
alignment, and the preferred alignment of an empty aggregate is 8. So the empty buffer
became `[0 x i64]`, padding `Bool` to `{ i8, [0 x i64] }` = 8 bytes. `Array Bool` then
used 8 bytes per element and its i8 stores no longer vectorized.

Fix (`ObjectFieldType::to_basic_type`, `UnionBuf` arm): when the payload is empty
(`max_size == 0`), pin the buffer alignment to 1. `Bool` lowers to `{ i8, [0 x i8] }` =
1 byte (tag only), `Array Bool` regains packed i8 stores. After the fix, both the sieve
and the isolated `Bool` set loop match `main` exactly:

| case             | main (1.4.0) | branch, fixed |  delta |
| ---------------- | -----------: | ------------: | -----: |
| prime_table      |   17,597,118 |    17,597,118 | +0.000% |
| Bool set loop    |   36,160,666 |    36,160,666 | +0.000% |

`Bool` stays a union; only the wasted alignment of its empty buffer is removed. The fix
also restores `Bool`'s size to 1 byte, matching `main`.
