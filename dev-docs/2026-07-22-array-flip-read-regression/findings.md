# Array flip: the read-loop regression, its cause, and why the pass-level fix is not viable

The unboxed-`Array` flip (`Array` = `{ storage, size, capacity }`, a 3-word value) makes write loops
much faster but regresses `fold` / iterator read loops badly (sum_by_fold +141%,
sum_by_loop_iter_cap +165%, fill_from_map +136% vs the pre-flip row; real code cp_lib_unionfind
+30%, arrayrw_shared +8.4%). This note records the confirmed root cause, a pass-level fix that
recovers the reads but breaks tail-call optimization (so it cannot ship), and the real fix.

## Confirmed root cause

Measured on `sum_by_fold` (`arr.to_iter.fold(0, |acc,x| acc+x)`, 100000 elements) at `-O
experimental`, comparing the pre-flip compiler (`69d9257b`) against the post-flip tip, by emitting
LLVM IR with `--emit-llvm` and inspecting the optimized module:

| stage | pre-flip | post-flip |
| --- | --- | --- |
| bounds-check calls in codegen output (pre-pass IR) | 3 | 3 |
| bounds-check calls after optimization | **0** | **2** |
| vector operations after optimization | **60** | **0** |

Both compilers emit the same three `fixruntime_index_out_of_range` bounds checks. The difference is
what LLVM can then remove. Pre-flip it proves all three redundant and vectorizes; post-flip two
survive, and `loop-vectorize` then refuses the loop — its own remark says **"call instruction cannot
be vectorized"**: the surviving bounds-check call (and its abort branch) cannot be if-converted, so
the read loop stays scalar. Whole-program instruction cost goes from ~4.2/element (vectorized) to
~10/element (scalar), the ~2.4x regression. There is no other cause — no per-iteration reference
counting (release happens once, after the loop), no aggregate spill, no per-element layout overhead;
the loop-carried value sits in registers.

Why can't LLVM remove the checks post-flip? The per-element check is `i < arr.@size`, and the loop
runs exactly `@size` times, so it is always true. Proving that needs LLVM to see `@size` as a scalar
it can correlate with the loop trip count. Post-flip the `Array` value is threaded through the loop
as a **loop-carried aggregate `phi { ptr, i64, i64 }`**, reconstructed with `insertvalue` each
iteration. `@size` is `extractvalue(phi, 1)` — buried inside the aggregate — and LLVM's value
analyses (SCEV, CVP, SCCP) do not see through an aggregate phi. So the bound is opaque, the check is
kept, and the loop does not vectorize. Pre-flip the array was a single pointer; its `@size` came from
a plain load LLVM tracked, and the loop carried only a scalar index, so the checks folded and it
vectorized.

The aggregate phi comes from Fix lowering `loop` / `fold` as tail recursion and relying on LLVM's
`tailcallelim` to build the loop: `tailcallelim` makes one loop-carried phi per recursion argument,
of that argument's type, and an unbox struct is passed as an LLVM aggregate. The user's intuition is
right — `{ptr,size,cap}` is the C++ `std::vector` / Rust `Vec` layout and is not inherently slow.
C++/Rust vectorize array sums because their loop-carried state is scalar in the IR; Fix's is an
aggregate. The layout is fine; the loop-carried *shape* is the problem.

## A pass-level fix recovers the reads — but breaks tail-call optimization

`sroa` scalarizes memory allocas, not SSA phis, so running it on the aggregate phi does nothing (and
no other single pass helps: O3×2, O2-after-O3, correlated-propagation, sccp, indvars, loop-rotate,
aggressive-instcombine were all tried). The way to reach the phi is a memory round-trip: `reg2mem`
demotes SSA (including the aggregate phi) to allocas, `sroa` splits the aggregate alloca per field,
`mem2reg` promotes the fields back to individual scalar phis, and a second `default<O3>` then sees
the scalar size, folds the check, and vectorizes. Ground-truth cachegrind with
`default<O3>, reg2mem, sroa, mem2reg, default<O3>`:

| case | flip (current) | with the pass fix | pre-flip |
| --- | --- | --- | --- |
| sum_by_fold (read) | 1,011,790 | **401,787** | 419,292 |
| write_by_range_fold (write) | 1,021,669 | 1,020,469 | 1,661,039 |
| array_mod | 1,111,798 | **606,402** | 1,490,522 |

The read regression is erased (below pre-flip) and the write wins are kept — perfect, except it does
not survive the test suite. **`test_basic::test22` stack-overflows** (`SIGSEGV`) under this pipeline.
test22 is a two-accumulator tail recursion of 1,000,000 iterations; Fix compiles `fix $ |loop,a,x|
... loop(a2,x2)` to an **indirect** `tail call` through the closure's function pointer, which LLVM's
`tailcallelim` cannot turn into a loop — it relies on the backend performing the tail call (reusing
the frame). `reg2mem` demotes values to allocas, and a closure passes its captured state to the
recursive call **by pointer**, so an alloca's address escapes into the indirect call; `mem2reg`
cannot promote an address-taken alloca, it stays live across the call, and the backend can no longer
do the tail call. The frame grows per recursion → stack overflow. This was confirmed with the
scalarization both before and after the first `O3` (`O3,O3,reg2mem,...` and `O3,reg2mem,...,O3,O3`):
both vectorize sum_by_fold and both still overflow test22.

`reg2mem` is the only pass-level way to scalarize the aggregate phi, and it is fundamentally
incompatible with Fix's closure-based tail recursion. **The pass-pipeline route is a dead end.**

## The real fix (codegen)

Emit loop-carried unbox-struct values as scalar SSA, not one aggregate — so there is no aggregate phi
to scalarize and no `reg2mem` is needed (TCO is untouched). Concretely, pass unbox-struct function
arguments as exploded scalar fields instead of one LLVM aggregate (`Object::to_embedded_type` puts
the aggregate type straight into signatures today); then `tailcallelim` builds per-field scalar phis,
`@size` is a scalar LLVM correlates with the trip count, the checks fold, and the loop vectorizes —
exactly the pre-flip behaviour. This ripples through the ABI (retain/release/traverser signatures and
the closure ABI all pass unbox structs), so it is correctness-critical and must clear the full suite
+ memcheck + project_euler. Narrower variants: a Fix-level TCO for `loop` that emits scalar loop-state
phis instead of leaning on `tailcallelim`, or an RC-IR-level elimination of the `@(i)` bounds check
when the index provably ranges over `[0, @size)` (kills the check symptom before LLVM, sidestepping
the aggregate-phi opacity, but does not restore the from_map store-loop vectorization).

## Recommendation

The read regression is fixable and the win is large (sum_by_fold below pre-flip, array_mod a further
-45%), but only through the codegen change above — the pass-level shortcut breaks deep tail recursion
and cannot ship. This is a real compiler change, not an overnight tweak, so it was not attempted
unsupervised; a miscompile on `bce` would be worse than the perf regression. Until it is done, the
flip stands as a net trade-off: large write wins, and read/fold regressions that are large in ratio
but small in absolute terms on the micro-benchmarks and single-to-low-double-digit percent on real
cp_lib code.

## Reproduction

- Post-flip codegen IR of a case: in `benchmark/speedtest/cases/<case>`, `rm -rf .fixlang *.ll` then
  `fix build -O experimental --emit-symbols --disable-cpu-feature 'avx512.*' --emit-llvm
  --allow-preliminary-commands`; the `Module-*.ll` is pre-pass, `Module-*_optimized.ll` is post-pass.
  (A cached build emits no IR — clear `.fixlang` first.)
- Pre-flip compiler worktree: `/home/maruyama/fixlang/.claude/worktrees/preflip-ir` (`69d9257b`, built).
- Try pass pipelines fast without rebuilding Fix: `/home/maruyama/llvm-17.0.6/bin/opt -passes='...'
  <pre-pass>.ll -S -o out.ll`, then grep `fixruntime_index_out_of_range` (checks) and `<N x i64>` /
  `vector.body` (vectorization). Vectorization-failure remarks: add
  `-pass-remarks-missed=loop-vectorize -pass-remarks-analysis=loop-vectorize`.
- The pass pipeline lives in `src/build/build_object_files.rs` `optimize_and_verify`; `--llvm-passes-file`
  replaces the post-`default<O3>` passes, so a file of `reg2mem,sroa,mem2reg,default<O3>` reproduces
  the read fix (and the test22 overflow).
