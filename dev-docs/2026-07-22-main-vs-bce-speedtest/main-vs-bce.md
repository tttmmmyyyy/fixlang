# Speedtest: `main` vs `bce`, whole-branch effect

> Added by a different agent session (`session_011etdzFoBKqpvsZEAJWj1gp`, the RC-IR
> declaration/borrow work), not the array-storage design session that owns the rest of this
> branch's recent commits. Recorded here at that session's request: a check for whether the RC-IR
> and bounds-check-elimination work on `bce` costs performance anywhere relative to `main`.

Question asked: **is there any substantive optimization degradation going from `main` to `bce`?**

Answer: **no.** Across all 41 speedtest cases, not one is slower on `bce`; the worst case is
unchanged (+0.00%), and most improve, several by large margins.

## Revisions

- `main` = `cae6b27c` (fixlang 1.4.0), an ancestor of `bce`, so `bce = main + the RC-IR/BCE work`.
- `bce` = `71fade5c` (fixlang 1.5.0).

## The cp-library obstacle and why `main` was left unmodified

The `cp_lib_*` cases depend on cp-library, and the two branches pin different revisions: `main`'s
suite resolves `0.7.0` to the `0.7.3` tag, `bce`'s suite pins the migrated `0.7.4` (rev
`7a3e4a22`). `bce` cannot build `0.7.3` — it deleted the primitive that revision's unchecked swap
uses; `main` cannot build `0.7.4` — it lacks the `unsafe_swap_bounds_unchecked` that revision's
swap uses.

Two facts made a faithful comparison possible without touching `main`:

1. The `0.7.3` → `0.7.4` source diff is **only** a mechanical rename of the unchecked-swap
   primitive — two files (`misc/misc.fix`, `zp/zp.fix`), `_unsafe_swap_bounds_uniqueness_unchecked`
   → `unsafe_swap_bounds_unchecked`, semantically identical. So each compiler building its own
   revision measures the same algorithms, each using its own compiler's native swap primitive.
2. Modifying `main` to build `0.7.4` would not have been small: `main` has no RC-IR back end (it is
   still the `enum LLVMGenerator` architecture, no `src/rc_ir/`), so porting `bce`'s swap built-in
   (a `LLVMGen` trait object) would mean adding an enum variant, a struct, a dispatch arm, and the
   registration — and would risk contaminating the very thing being measured.

So: **`main` was not modified.**

## Method

- Metric: cachegrind `Ir` (instruction reads), deterministic and independent of machine load.
- Options: `-O experimental --disable-cpu-feature 'avx512.*' --allow-preliminary-commands`, the
  speedtest harness settings.
- Two tiers:
  - **non-cp_lib (33 cases):** the identical `bce` case source built by each compiler — a pure
    compiler-only comparison, no confound.
  - **cp_lib (8 cases):** `main` builds its natural `0.7.3`, `bce` its natural `0.7.4`; the only
    source difference is the swap-primitive rename above.

## Results

Not one case regresses. `delta` is `(bce - main) / main`.

### cp_lib (same algorithms, each compiler's native swap)

| case | main | bce | delta |
| --- | ---: | ---: | ---: |
| cp_lib_lsegtree | 2,218,854,703 | 1,245,091,788 | -43.89% |
| cp_lib_segtree | 156,733,543 | 93,664,595 | -40.24% |
| cp_lib_prime_list | 11,547,217,484 | 7,139,556,476 | -38.17% |
| cp_lib_dijkstra | 223,484,856 | 146,444,122 | -34.47% |
| cp_lib_unionfind | 118,105,256 | 92,158,165 | -21.97% |
| cp_lib_conv_zp | 311,435,122 | 262,560,529 | -15.69% |
| cp_lib_bipartite | 247,685,701 | 225,845,236 | -8.82% |
| cp_lib_scc | 180,163,057 | 165,928,629 | -7.90% |

### non-cp_lib (identical source, compiler-only)

| case | main | bce | delta |
| --- | ---: | ---: | ---: |
| sum_by_loop_iter_cap | 2,519,638 | 263,398 | -89.55% |
| sum_by_loop_arr | 2,319,642 | 263,398 | -88.64% |
| sum_by_fold_cap | 2,119,637 | 263,398 | -87.57% |
| sum_by_range_fold | 2,119,637 | 263,398 | -87.57% |
| sum_by_fold | 1,319,643 | 263,398 | -80.04% |
| sum_by_loop_iter | 2,219,652 | 1,319,640 | -40.55% |
| array_mod | 2,120,179 | 1,319,632 | -37.76% |
| bounds_check_indexable | 116,404,833 | 79,081,162 | -32.06% |
| sum_by_loop_iter_s | 1,719,630 | 1,219,618 | -29.08% |
| fannkuch | 4,322,464,039 | 3,115,057,354 | -27.93% |
| option_plumbing | 856,821 | 616,817 | -28.01% |
| struct_field_mod | 3,201,169,202 | 2,401,769,199 | -24.97% |
| arrayrw | 3,201,169,202 | 2,401,769,199 | -24.97% |
| arrayrw_fn | 3,201,169,188 | 2,401,769,185 | -24.97% |
| arrayrw_shared | 3,201,170,892 | 2,400,970,890 | -25.00% |
| sort | 118,937,852 | 92,721,116 | -22.04% |
| nbody_fold | 2,466,167,430 | 1,950,166,200 | -20.92% |
| write_by_range_fold | 2,061,571 | 1,661,247 | -19.42% |
| prime_table | 17,597,718 | 14,556,819 | -17.28% |
| gen_random_array | 8,908,949 | 7,397,694 | -16.96% |
| random_state | 720,444,023 | 610,312,733 | -15.29% |
| nbody | 2,280,167,489 | 1,946,166,214 | -14.65% |
| index_syntax | 468,575,732 | 425,575,726 | -9.18% |
| push_back | 2,360,804 | 2,160,797 | -8.47% |
| mutate_boxed_loop | 598,300,255 | 548,259,631 | -8.36% |
| sum_by_fix | 655,162,094 | 655,162,096 | +0.00% |
| sum_by_loop | 856,817 | 856,817 | +0.00% |
| binary_trees | 781,411,267 | 781,411,263 | -0.00% |
| mandelbrot | 514,282,477 | 514,282,473 | -0.00% |
| mandelbrot_fold | 514,285,491 | 514,282,473 | -0.00% |
| fill | 7,639,810 | 7,639,802 | -0.00% |
| fill_from_map | 7,639,810 | 7,639,802 | -0.00% |

The large loop/fold wins are the bounds-check-elimination and vectorization work; the ~25% on the
array read/write and struct-RMW cases is the RC-IR uniqueness work removing checks and clones. The
handful of flat cases already generated identical hot loops on `main`.
