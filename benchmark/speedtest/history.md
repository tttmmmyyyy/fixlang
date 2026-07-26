# Benchmark History

Newer is above.

## 12165c4494bf4cc806f72ec6475cc146b2b36532

The `wide-return-tail-call` branch (PR #109), which keeps a Fix tail call compiled as a jump in the
two cases where the backend used to give up on one: a return value wider than the return registers
(the value now travels through an out-pointer parameter) and a tail call carrying more arguments than
the argument registers (every Fix lambda is now `tailcc`). Measured against the previous row
`476f40aa`.

**mandelbrot and mandelbrot_fold drop 53.94%**, index_syntax 3.33% and cp_lib_conv_zp 1.99%. The 41
cases together go from 16,386,181,734 to 15,866,111,410 instructions, **-3.17%**. Every win comes from
the out-pointer half: passing the pointer as an ordinary parameter from the start of the IR lets SROA
delete the buffer wherever the callee inlines, where LLVM's own return demotion used to introduce the
pointer at instruction selection, after every IR pass had already run.

Every regression comes from `tailcc`: fannkuch +1.51%, cp_lib_bipartite +1.10%, sort +0.73%,
cp_lib_unionfind +0.68%, cp_lib_lsegtree +0.56%, binary_trees +0.40%, cp_lib_scc +0.39%, get_sub
+0.09%. The cases that compile to one inlined loop are unchanged. Three properties of a
guarantees-tail-calls convention account for all of it on x86-64:

- The callee pops the argument area, and `GetAlignedArgumentStackSize` rounds that area up so that it
  plus the return address is 16-byte aligned. A function with no stack arguments at all therefore
  ends in `ret $8`, and every **non-tail** call site pays one `sub $0x8, %rsp` to restore its own
  frame. get_sub calls `slice_bench` 100,000 times and grows by exactly 100,000 instructions.
- Incoming stack arguments become mutable frame objects, since a tail call may overwrite them
  (`X86TargetLowering::LowerMemArgument` marks them so whenever the convention guarantees tail calls).
  A callee can no longer reload one from the caller's slot on demand, so it copies them into its own
  frame in the prologue. fannkuch's `Std::loop#2` grows its frame from 0x28 to 0x48 bytes and its
  prologue by about ten instructions per call; over its 3,628,800 calls that is 38.9M of its
  45.1M-instruction increase, the remainder being the `sub` at its two inner call sites.
- A tail call under such a convention never takes the sibcall path (`IsSibcall` is set only when the
  callee's convention does not guarantee tail calls), so it rewrites the outgoing argument area even
  when the values are unchanged, where a sibcall recognizes matching stack offsets and leaves them
  alone. This is also what buys the fix: a sibcall may not grow the argument area, and a guaranteed
  tail call may.

Restricting `tailcc` to the functions whose arguments exceed the argument registers would spare
everything else the first two costs, but LLVM refuses a tail call whose callee is `tailcc` and whose
caller is not — `IsEligibleForTailCallOptimization` returns false unless the conventions match — so a
narrow function tail-calling a wide one would stop being a jump. The convention has to be uniform
across everything reachable by a tail call.

## 476f40aa1ef55bf5f0880495bd2000860ad13e13

The `defunctionalize-fix-tco` branch (PR #95), which rewrites `Std::fix` into a directly
self-recursive global so LLVM's tail-call elimination can fold it into a loop. Measured against the
previous row `eec295f8` on the same speedtest path.

The one benchmark that uses the `fix` combinator, **sum_by_fix, drops from 655.2M to 0.21M
instructions — -99.97%, a 3175x reduction**. Defunctionalization turns the indirect `fix` self-call
into a direct one; LLVM loop-ifies it and SCEV then closes the accumulation into a constant-time
form. This is the win the branch exists for.

The branch changes nothing else. Its passes only touch `fix`-using symbols, and the standard library
uses no `fix`, so every other program's code is untouched. Confirmed directly: the emitted LLVM IR of
a representative non-`fix` case (sum_by_loop), both before and after LLVM's own optimization passes,
is byte-identical between this branch and its fork point `6dd8c629`, and the two produce the same
executed-instruction count when built at the same path.

The remaining movement the graph shows on the small cases — roughly +44,000 instructions, up to +18%
on the ~250K-instruction micro-benchmarks but +0.00% on every case above a few million — is not a
code change. Built head-to-head today at a fixed path, the previous row's compiler (`eec295f8`) and
this branch's compiler produce the same instruction count within noise (within +/-60 on the ~250K
micro-benchmarks; identical on sum_by_loop), so neither the intervening `main` commits (#88-#91) nor
this branch regressed anything. The two rows were measured ~18 hours apart (`eec295f8` on 2026-07-25
16:01, this row 2026-07-26 10:05) across overnight system-package activity — a kernel and `libc-bin`
update landed at 16:51, minutes after the `eec295f8` run. The +44K is a shift in the emitted
program's fixed per-program startup, an environment effect on the harness's real-project build, not
the compiler. Read a pure code delta by measuring two commits back-to-back in one environment, not
against a historical row.

## eec295f846d6110826a74e823fde8a6ae02859d4

The object-scalarization branch merged with `main`, measured against the previous row `96f68049` (the
cp-library 0.13.0 bump). The branch makes the codegen `Object` hold leaf scalars and materialize an
aggregate only at memory and foreign-ABI boundaries: the body, the return ABI, and the per-type RC
helpers (retain / release / mark / traverser) all pass leaf scalars. The array-loop win it targets was
already banked by the shipped scalar-argument ABI and `build_scalar_phi`, so what remains is code
unification, and the measurement bears that out: most cases are byte-identical (binary_trees, arrayrw,
nbody, mandelbrot, struct_field_mod all to 0.00%), with sub-1% movement each way on the rest (sort
-3.7%, cp_lib_lsegtree -0.8%, fannkuch -0.8%; get_sub +0.5%).

This baseline predates two `main` commits the merge also brings in — the per-signature FFI typing
(#85) and the zero-sized-phi-to-undef change (#86) — so the delta folds those in as well. The only
movement above 1% is confined to the two heaviest cp_lib cases, cp_lib_conv_zp +2.1% and
cp_lib_prime_list +1.9%; with the scalarization confirmed byte-neutral on every non-cp_lib case, that
residue tracks the folded-in #85/#86 codegen changes rather than the scalarization.

## 96f680496768b92145e8d577c26356091e0104d9

Moving the eight `cp_lib_*` cases from cp-library 0.7.4 to 0.13.0, measured against the previous
baseline row `d29b6c3c` on the same compiler. The six cases that generate input with `Random` gained a
direct random 1.1.2 dependency (0.13.0 dropped random from its build deps), the same version 0.7.4
supplied transitively, so the workloads are unchanged. The 32 non-cp_lib cases confirm this: every one
is identical to the baseline to within 0.0000%.

Seven of the eight cp_lib cases are likewise unchanged — their algorithms compile identically across
the two cp-library versions. The exception is **unionfind, which regresses +29.8% in instructions
(111.2M -> 144.4M) and +31.2% in memory accesses (179.1M -> 235.0M)**: cp-library 0.13.0's UnionFind
is meaningfully heavier than 0.7.4's. Compiler and input are held fixed, so this is a cp-library-side
change to weigh, not a compiler regression.

## d29b6c3ccfdd8c92f3999aaec0c7c78778b238c2

Baseline of `main` after the bce merge (#80) and the external-test change (#83), taken before bumping
the `cp_lib_*` cases from cp-library 0.7.4 to 0.13.0. It matches the previous bce row `f0a60009` to
within noise — the intervening commits (the `Arc<RcExpr>` change, the `grow_stack` helper, and the
test-only #83) do not touch code generation — so the merged main reproduces the last bce measurement
and gives a clean reference for the cp-library version change measured next.

## f0a600092158e34fccbe3ac6c44d64b6db8782d5

Removing the traverser `alwaysinline` attribute, measured against the row that added it as an enum
attribute (`0adf6eba`). The two are identical to within noise (total +0.000%, per-case median
diff 0), so the attribute did nothing in any form — an enum `alwaysinline` on the traversers is as
inert as the string one was. The small fixed offset seen against the older `9e6c6f64` row is present
with the attribute and without it alike, so it belongs to other commits or run-to-run startup, not
to this attribute.

## 0adf6ebaa6a8eb33360e6d7044ebcd54389e198d

Attaching `alwaysinline` to the object traversers as a real enum attribute (it had been a string
attribute, which LLVM ignores), measured against the previous row `9e6c6f64`. The effect is nil:
every case is within +0.5%, the whole suite totals +0.00%, and the tiny non-zero deltas are a fixed
per-program increment (a little more inlined traverser code on the startup path), not a per-workload
change. Forcing the traversers to inline buys nothing — most RC traversal goes through a function
pointer stored in the control block, where the hint cannot apply, and the direct calls LLVM already
inlines on its own.

## 9e6c6f64eb4fdb73c48e46a2d766ee332d5eaec4

Marking the runtime panic functions (`fixruntime_abort`, `fixruntime_index_out_of_range`,
`fixruntime_negative_array_size`) as `noreturn`, measured against the previous row `a77ad9dd`. These
functions never return, but their LLVM declarations lacked the attribute, so a bounds-check failure
path called one and then flowed to a merge, feeding an `undef` into it. That forced an aggregate phi
for the loop-carried value, which hid the array size and left the per-element bounds check standing.
This had regressed when the direct libc `abort` (which LLVM recognizes as `noreturn`) was replaced by
these custom functions to print richer messages and backtraces. The attribute lets LLVM prune the
failure path, the aggregate collapses to scalars, and the check folds.

Sixteen cases improve and none regress: get_sub -80.5%, cp_lib_segtree -20.4%, nbody_fold -16.5%,
cp_lib_lsegtree -15.5%, fannkuch -14.3%, bounds_check_indexable -9.8%, gen_random_array -9.5%,
nbody -9.2%, cp_lib_prime_list -8.6%, random_state -8.3%, sort -7.7%, cp_lib_unionfind -4.1%,
cp_lib_scc -4.0%, cp_lib_dijkstra -3.6%, cp_lib_bipartite -3.2%, index_syntax -1.6%. Against the flip
row `4537cc17`, every read/fold regression is now erased and the write wins are kept; the sole case
still above that baseline is cp_lib_bipartite (+3.4%), which carries a genuine multi-exit
control-flow aggregate that this change does not reach.

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