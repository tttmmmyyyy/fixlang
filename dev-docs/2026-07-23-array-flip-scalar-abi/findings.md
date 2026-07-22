# Array flip: scalarizing loop-carried unbox structs to restore vectorization

The unboxed-`Array` flip (`Array` = `{ storage, size, capacity }`) sped up write loops but
regressed `fold` / iterator read loops, because the flipped `Array` is threaded through a loop as a
loop-carried **aggregate** value whose `@size` LLVM cannot see through, so the per-element bounds
check is not proven redundant and the loop does not vectorize (see
`dev-docs/2026-07-22-array-flip-read-regression/findings.md` for the root cause, and why demoting the
aggregate phi to memory with `reg2mem` is not viable — it breaks tail-call optimization).

This note records the codegen change that fixes it and its measured outcome.

## The change

A loop-carried unbox struct becomes an aggregate phi in one of two ways, so both are addressed.

1. **Function arguments** (`b82b35d9`). `fold` / `loop` lower to a recursive tail call; LLVM's
   `tailcallelim` turns that into the loop, making one loop-carried phi per recursion argument. An
   unbox-struct argument is one LLVM aggregate, so the phi is an aggregate and `@size` is buried in
   it. The fix passes each unbox-struct argument as its flat leaf scalars instead of one aggregate:
   `lambda_function_type` builds the flattened signature, `apply_lambda` explodes the argument at the
   call, and `implement_rc_function` reassembles it at the entry. `tailcallelim` then builds per-field
   scalar phis, `@size` is a value LLVM correlates with the trip count, the bounds check folds, and
   the loop vectorizes. Helpers `flatten_to_scalar_leaves` / `explode_to_scalar_leaves` /
   `assemble_from_scalar_leaves` (in `generator.rs`) are the shared flatten/rebuild primitives.

2. **Codegen-emitted value merges** (`7abd23c9`). Some merges are phis Fix codegen emits directly —
   the unique/shared merge in `Array::set` and `set_capacity`, the match-arm merge, the union-modify
   merge. These are aggregate phis the argument change does not reach, so a merge inside a hot loop
   (a `set` on a shared array, cloned once then written in place) keeps `@size` opaque.
   `scalar_build_phi` merges each leaf field with its own scalar phi and reassembles the aggregate
   afterward, and replaces `build_phi` at those sites.

In both cases the reassembly `insertvalue`/`extractvalue` is folded away by SROA/instcombine, leaving
only the scalar phis. Unlike `reg2mem`, nothing is demoted to memory, so tail-call optimization is
untouched and deep tail recursions (e.g. `test22`) still run in constant stack.

## Measured outcome

Per-case cachegrind instruction counts at `-O experimental`, against the flip baseline (`4537cc17`)
and the pre-unboxing baseline (`69d9257b`): **24 improved, 14 flat, 3 regressed** (the flip alone
regressed 16). The read/fold regressions are erased, most now below the pre-unboxing baseline, and
the write wins are kept:

| case | vs flip | vs pre-flip | note |
| --- | --- | --- | --- |
| sum_by_fold | -75% | **-40.7%** | read loop; below pre-unboxing |
| sum_by_loop_iter_cap | -78% | -41% | |
| array_mod | -59% | -70% | read+write |
| fill_from_map | -58% | -0% | back to pre-unboxing |
| arrayrw / arrayrw_fn | -94% | -95% | write win kept |
| arrayrw_shared | -95% | -95% | fixed by `scalar_build_phi` |
| nbody / nbody_fold | -31% / -27% | | |
| option_plumbing | -60% | -60% | |
| struct_field_mod | -0% | -95% | write win kept |

Correctness: the full suite is **980 passed / 0 failed at `-O basic` and `-O max`**. At `-O none` two
external-project tests (`random`, `hashset`) stack-overflow; this is the pre-existing `-O none`
weakness — that level has no tail-call optimization, so deep recursion overflows the default stack —
and it reproduces identically on the flip baseline and passes with a larger stack, so it is not a
miscompile. A project_euler output diff of this build against the flip baseline agrees byte-for-byte
on every solution that builds and runs, and breaks none that the baseline builds.

## Residual regressions

Three cases regress, all sharing one cause: a loop-carried aggregate that the change does not turn
into scalars because it is re-formed after the point the change controls. Each has a standalone
speedtest case, so the pattern stays visible.

- **bounds_check_indexable** (+11% vs flip, at the pre-unboxing level). The loop carries a nested
  `Array2 { data: Array, size: (I64, I64) }`. The argument is scalarized, but the value is also
  *returned* from the fold body, and LLVM merges the returns into an aggregate `common.ret.op` phi —
  the return is still one aggregate. Only arguments are scalarized here, not returns.
- **cp_lib_bipartite** (+7% vs flip). A large record (several arrays plus a union) is carried through
  augmenting-path control flow with many exits. The entry reassembles it, and SimplifyCFG re-forms
  the aggregate as sink (`.pn`) phis faster than SROA removes it, so the scalarization overhead
  exceeds its benefit for this large state.
- **cp_lib_dijkstra** (+1% vs flip). Small churn from `scalar_build_phi` on a merge that was not the
  bottleneck.

Erasing these needs a deeper change than this one: carrying loop-state fields as scalars through the
whole function body (so the `Object` never re-materializes the aggregate), or extending the scalar
ABI to return values as well as arguments.
