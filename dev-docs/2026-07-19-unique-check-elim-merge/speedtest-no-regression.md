# Merge `unique-check-elim` into `bce`: speedtest shows no regression

Merge commit `c97d6a4e` brings the `unique-check-elim` RC-IR fixes into `bce`
(empty-struct provenance normalization, the destructured-boxed-struct unique-check
miscompile fix, and the provenance-as-map-of-boxed-leaves representation). These touch
the RC-IR provenance / borrow / unique-elimination machinery, which is active at Max
(`-O experimental`), so they can in principle change generated code. This records the
isolated before/after measurement.

## Method

- Metric: cachegrind `Ir` (instruction reads) — deterministic, independent of machine load.
- Build: `fix build -O experimental --emit-symbols --disable-cpu-feature avx512.*` (the
  speedtest harness settings), full 37-case suite in `benchmark/speedtest/cases/`.
- Isolation: the pre-merge tip `cb10127c` was measured in its own git worktree so each row
  is produced by the `fix` compiler built from that exact revision; the post-merge tip
  `c97d6a4e` was measured in the `bce` worktree. The only difference between the two rows is
  the merged commits.

Both rows carry the read-loop BCE work already on `bce` (iterator `>=` termination + the
RC-IR simplifier), confirmed by e.g. `sum_by_range_fold` sitting at its vectorized ~265k in
both — so this comparison isolates the merge, not the BCE work.

## Result

Every case is within a few instructions (max |delta| ≈ 0.005%, i.e. measurement-level jitter
on cases of 10^5–10^9 instructions); the suite total moves by `-0.000%`. No regression.

| case | pre `cb10127c` | post `c97d6a4e` | delta |
| --- | ---: | ---: | ---: |
| array_mod | 1,422,155 | 1,422,169 | +0.00% |
| arrayrw | 2,401,771,185 | 2,401,771,185 | +0.00% |
| arrayrw_fn | 2,401,771,185 | 2,401,771,185 | +0.00% |
| binary_trees | 781,413,249 | 781,413,235 | -0.00% |
| bounds_check_indexable | 104,129,742 | 104,129,742 | +0.00% |
| cp_lib_bipartite | 234,786,701 | 234,786,680 | -0.00% |
| cp_lib_conv_zp | 262,562,620 | 262,562,616 | -0.00% |
| cp_lib_dijkstra | 162,325,895 | 162,325,883 | -0.00% |
| cp_lib_lsegtree | 1,296,184,288 | 1,296,184,284 | -0.00% |
| cp_lib_prime_list | 7,139,558,578 | 7,139,558,534 | -0.00% |
| cp_lib_scc | 167,181,437 | 167,181,441 | +0.00% |
| cp_lib_segtree | 109,810,614 | 109,810,621 | +0.00% |
| cp_lib_unionfind | 98,489,618 | 98,489,600 | -0.00% |
| fannkuch | 3,684,918,184 | 3,684,918,170 | -0.00% |
| fill | 7,641,788 | 7,641,774 | -0.00% |
| fill_from_map | 7,641,774 | 7,641,788 | +0.00% |
| gen_random_array | 7,399,680 | 7,399,666 | -0.00% |
| index_syntax | 425,577,712 | 425,577,698 | -0.00% |
| mandelbrot | 514,284,473 | 514,284,473 | +0.00% |
| mandelbrot_fold | 514,284,459 | 514,284,459 | +0.00% |
| nbody | 1,946,168,585 | 1,946,168,599 | +0.00% |
| nbody_fold | 1,950,168,599 | 1,950,168,599 | +0.00% |
| option_plumbing | 618,803 | 618,803 | +0.00% |
| prime_table | 14,558,805 | 14,558,805 | +0.00% |
| push_back | 2,162,769 | 2,162,783 | +0.00% |
| random_state | 640,250,657 | 640,250,643 | -0.00% |
| sort | 92,723,102 | 92,723,088 | -0.00% |
| sum_by_fix | 655,164,096 | 655,164,096 | +0.00% |
| sum_by_fold | 265,384 | 265,384 | +0.00% |
| sum_by_fold_cap | 265,384 | 265,384 | +0.00% |
| sum_by_loop | 858,803 | 858,803 | +0.00% |
| sum_by_loop_arr | 265,384 | 265,384 | +0.00% |
| sum_by_loop_iter | 1,321,626 | 1,321,612 | -0.00% |
| sum_by_loop_iter_cap | 265,384 | 265,370 | -0.01% |
| sum_by_loop_iter_s | 1,221,618 | 1,221,618 | +0.00% |
| sum_by_range_fold | 265,370 | 265,384 | +0.01% |
| write_by_range_fold | 1,663,233 | 1,663,233 | +0.00% |
| **suite total** | **25,631,362,939** | **25,631,362,791** | **-0.000%** |

## Why this is expected

The merged commits fix correctness in provenance/unique-check handling for shapes
(empty-struct payloads surfaced by union removal, destructured boxed-struct fields) that the
benchmark cases do not exercise in their hot loops, so the emitted code for these cases is
unchanged. The sub-instruction differences come from incidental symbol ordering, not from a
codegen change on any measured path.
