# Benchmark History

Newer is above.

## a77ad9dd29282fb48a29763115d27aedefd59a4b

Scalarizing loop-carried unbox structs, measured against the flip row `4537cc17`. A loop-carried
`Array` (or an iterator holding one) was threaded through a `fold` / `loop` as one LLVM aggregate,
so `@size` hid inside an aggregate phi, the per-element bounds check survived, and the read loop
did not vectorize. Passing unbox-struct function arguments as flat leaf scalars, and building
codegen's value-merge phis one scalar phi per leaf, exposes `@size` again and the loops vectorize —
with tail-call optimization intact (unlike the `reg2mem` alternative).

Read / fold regressions are erased, most now below the pre-unboxing baseline: sum_by_loop_iter_cap
-77.6%, sum_by_fold / sum_by_fold_cap / sum_by_range_fold -75.4%, array_mod -59.2%,
fill_from_map -57.5%, sum_by_loop_arr -43.9%, sum_by_loop_iter -39.2%. Write wins are kept
(arrayrw -94.0%, arrayrw_shared -95.3%) and other loops improve as their state goes scalar
(option_plumbing -60.4%, nbody -31.3%, random_state -29.9%, nbody_fold -26.7%, push_back -8.0%).
Three cases regress, all carrying a large aggregate re-formed past the change's reach:
bounds_check_indexable +10.9% (the value is also returned, and returns stay aggregate),
cp_lib_bipartite +6.9%, cp_lib_dijkstra +1.4%.

## 4537cc177baee6a72256f5c96a14f643795c9afc

The Array value-layout flip to unboxed `{ storage, size, capacity }`, measured against the
step-1-end row `69d9257b`. Write-heavy cases improve as intended, because `@size` / `@capacity`
become register reads and the bounds / capacity checks fold: struct_field_mod -95.0%,
prime_table -45.0%, write_by_range_fold -38.5%, array_mod -25.4%, arrayrw -16.7%,
push_back -13.6%, cp_lib_prime_list -13.4%.

Read / fold cases regress, the risk the design's §10 anticipated: the fatter 3-word `Array`
value swells the iterator loop state (`Option (ArrayIterator a, a)`), which then spills to
memory instead of staying scalar. sum_by_loop_iter_cap +165%, sum_by_fold / sum_by_fold_cap /
sum_by_range_fold +141%, fill_from_map +136%, sum_by_loop_iter +40%. cp_lib_unionfind +30%
(this row also carries the cp-library 0.7.3 -> 0.7.4 migration diff on the eight cp_lib cases,
and the subprocess migration on the driver).

## 7afe8e174d0a785106d7c0e4961bce88e2d3beb0

Reverted the temporary no-runtime-check enablement.

## 0bec40c5d5765799987c474f93c6f2bb50369cf9

Temporarily enabled no-runtime-check. (Will be reverted in the next commit)

## ba06b2f2ced3ce16719038b71bdf790dccfdeb2c

Performance degradation due to adding checks for non-negative capacity and size in Array::fill and Array::empty.

## 7bd496c3cd6245f5604df0f2fb1ca96b657fe05e

Due to changes in the implementation of the check_range function.
In the previous commit e4e3a33dd436b06bd8126d4e273ab17957c483e2, check_range was already introduced, but it only displayed an error message and aborted.
Between that commit and 7bd496c3cd6245f5604df0f2fb1ca96b657fe05e, fixruntime_index_out_of_range was defined in runtime.c and changed to be called from check_range.
This caused performance degradation.
Note that we forgot to run the benchmark immediately after changing the check_range function implementation, so the impact appeared in the benchmark of a slightly later commit.