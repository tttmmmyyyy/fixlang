# Benchmark History

Newer is above.

**Rows measured before `cachegrind.py` fixed the environment are not comparable with rows after
it.** Cachegrind counts the dynamic loader and libc start-up along with the program, and both walk
the environment, so a row carried about 600 instructions per variable the shell that ran the
harness happened to export — a background run and an interactive one differed by tens of thousands
of instructions on every case. The measured command now runs with a fixed minimal environment, and
the `startup` case records what a program that does nothing costs, so a row says how much of each
figure was there before any of the work.

## 87649b1914a230ade36083b5c693b2f531313578

Runs LLVM's `default<O3>` pipeline to a fixpoint — once at `-O basic`, three times at `-O max` and
`-O experimental` — in place of the twelve hand-picked passes that used to follow a single run. The
whole suite comes to **-2.49%** on the geometric mean of the instruction counts, 22 of the 46 cases
falling and 3 rising.

| case | change | | case | change |
|---|--:|---|---|--:|
| nbody | -21.44% | | sort | -2.15% |
| fannkuch_scratch | -14.47% | | cp_lib_scc | -1.90% |
| fill, fill_from_map | -12.31% | | cp_lib_dijkstra | -1.43% |
| fannkuch | -8.63% | | cp_lib_segtree | -1.31% |
| mandelbrot, mandelbrot_fold | -6.47% | | cp_lib_lsegtree | -1.13% |
| nbody_fold | -5.29% | | arrayrw, arrayrw_fn, struct_field_mod | -0.66% |
| cp_lib_unionfind | -4.08% | | levenshtein | -0.29% |
| random_state | -3.32% | | cp_lib_bipartite | +0.92% |
| option_plumbing | -3.09% | | cp_lib_conv_zp | +0.26% |
| gen_random_array | -2.64% | | cp_lib_prime_list | +0.21% |

A fourth run of the pipeline changes no case by a single instruction, which is what fixes the count
at three.

**These figures are an upper bound on what a machine with avx512 gets.** The suite disables avx512
so that cachegrind can simulate the program, and the extra runs earn much more without it: `nbody`
falls 21.4% here and 6.6% when the same change is measured with avx512 enabled.

**Cycles move far less than instructions.** Over the nine cases of `benchmark/crosslang`, measured
by interleaving the two builds within one run and taking the minimum of thirty rounds, instructions
come to 0.956x and cycles to 0.987x. `nbody` loses 6.6% of its instructions there and no cycles at
all — those instructions were issuing in slots that were going empty. Compile time rises about 14%
at `-O experimental` and stays level at `-O basic`.

This row is also the first that another pipeline can be measured against in the same directory: the
pass pipeline now takes part in the object-file cache key, where before a second build returned the
objects the first had cached, whatever pipeline it was given.

## 423e50e1538e9f4f75708dac436869be871539c7

Evaluates a call's arguments in the order they are written (PR #140), where a prefix call used to
evaluate them backwards at `-O max`. The percentages are against the `a9a1b1a2` row, measured with
the same compiler sources on both sides.

| case | instructions | memory accesses |
|---|--:|--:|
| fib | -6.35% | -4.80% |
| levenshtein | -2.87% | -2.57% |
| binary_trees | -1.07% | +0.35% |
| cp_lib_unionfind | -0.37% | -0.31% |
| sort | +0.05% | 0.00% |
| cp_lib_bipartite | +0.35% | +0.15% |
| cp_lib_conv_zp | +0.74% | +1.00% |

Every other case moves less than 0.05%; the 46 together retire 0.26% fewer instructions. `fib` is the
case the order decides: LLVM's tail-recursion elimination folds the *last* call into a loop, so which
of `fib(n - 1)` and `fib(n - 2)` goes last picks the decomposition, and the written order picks the
one with fewer leaf calls.

**The splits column of the `a9a1b1a2` row is not comparable with this one.** It reads as a 90% fall on 41
of the 46 cases, including `startup`, whose instruction count is identical in the two rows — a program
that does nothing cannot have lost 152 split accesses to an evaluation-order change. Measured
back-to-back at one path, the two compilers give the same count: `startup` 16 and 16, `arrayrw` 17 and
17, `sum_by_fold` 23 and 23. The counter is repeatable within a run (five runs, one value) and perf
reports it 100% enabled. `perf_counters.py` reads the count and ignores the enabled percentage perf
prints beside it, so a run whose events the PMU time-sliced enters the log as a scaled estimate that
looks like any other measurement -- which is the condition that produces a tenfold column.

## d51e4a2eeaf179d01e5a918974b3a28e40dfbb3f

Removes two latent defects from the substitutor that rewrites free names (PR #127): a rewrite the
substitutor reported as unchanged, which the enclosing `let` or `match` then discarded, and a `let`
the inline-LLVM substitution introduces capturing a name that another replacement reads.

Neither is reachable through the compiler's own passes, so this row is here to show that the code
generated for these cases is the same. It is: no case moves by more than 0.05% in instructions or
in memory accesses, and the totals over the 46 cases move by -0.0000% and -0.0001%. The largest
movements are around fourteen instructions, on the cases small enough for that to register
(`startup`, `sum_by_fix`).

The split-access column drops by roughly 230 on nearly every case, `startup` included. A case that
runs no code of its own cannot have gained that from a compiler change, so read it as process
start-up rather than as anything this row measures.

## a9a1b1a2bd93952205e127f3cbe603d2e6a6c2c0

Starts a large array's elements on a 32-byte boundary (PR #128), so that a vectorized loop over them
stops straddling cache lines.

**Read this row in the split columns, not the instruction counts.** The instruction count cannot see
either the straddle or its removal: `arrayrw` retires the same instructions before and after, to
within eleven out of 120 million, and runs 1.71 times faster. What the splits say:

| case | before | after |
|---|--:|--:|
| arrayrw | 49,600,017 | 171 |
| arrayrw_fn | 49,600,017 | 171 |
| struct_field_mod | 49,600,019 | 171 |
| cp_lib_bipartite | 54,743,346 | 76,832 |
| fill, fill_from_map | 1,250,023 | 249 |
| cp_lib_prime_list | 195,356 | 175 |
| get_sub | 1,083,622 | 378,414 |
| levenshtein | 577,126 | 344,955 |
| nbody | 32,000,029 | 18,000,182 |
| nbody_fold | 30,000,029 | 16,000,183 |

A case whose elements are wider than a vector access is aligned only at its first element, which is
why `nbody` and `levenshtein` halve rather than clear: their elements are 24 bytes, and 24 does not
divide 32. `fannkuch` and `arrayrw_shared` do not move at all, the first because its arrays stay
under the size from which elements are aligned. Two cases gain splits — `cp_lib_lsegtree` 25,034 to
400,179 and `cp_lib_scc` 45,542 to 70,919 — from the up-to-31 bytes a large array now asks for
moving every allocation after it; neither moves in wall clock.

In wall clock, measured on an idle machine with twenty runs of each case: `fill_from_map` 2.8 times
faster and `fill` 2.3, `arrayrw`, `arrayrw_fn` and `struct_field_mod` 1.6, `cp_lib_bipartite` 1.2,
`nbody_fold` and `index_syntax` 1.04, and 32 of the 46 cases within three percent either way and
steady. The suite comes to 0.926 on the geometric mean of the ratios, or 0.934 taking each case's
fastest run. `cp_lib_scc` is 4 percent slower, the one steady regression: its arrays are 8 and 24
bytes, only 8 of its 226,000 allocations clear the threshold, so it pays the three instructions and
the byte store an array allocation now costs and wins nothing back. `prime_table` reads 5 percent
slower and `bounds_check_indexable`, `sort` and `cp_lib_segtree` within one percent, all four with a
spread wide enough that the figure moves between runs.

**`fill` and `fill_from_map` are bimodal, and that is the shape of the problem this change is
about.** Each allocates a thousand-element array ten thousand times, so one recycled block decides
the whole run, and where that block lands decides whether its accesses straddle. Five independent
measurements of `fill` give the unaligned build a mean of 3.7 to 4.3 ms with a standard deviation of
2.3, against 1.1 to 2.3 ms for the aligned one: the same program, the same input, and a factor of
four between runs of the binary that leaves its elements where the allocator put them. A single
timing of a case like this says more about the addresses it drew than about the code, which is why
the split counters are the column to read.

**This is the first row measured in the fixed environment, so the instruction counts fall against
the row above by a constant that belongs to the instrument.** The micro-benchmarks all move by
-43,546 give or take twenty; add that back to read what the change did. The large cases carry the
constant too, where it is lost in the total: `nbody_fold` -9.1%, `fannkuch` -1.6% and
`cp_lib_conv_zp` -1.0% fall, `cp_lib_scc` +2.0%, `index_syntax` +1.7%, `get_sub` +1.5% and
`cp_lib_dijkstra` +1.2% rise, from inlining decisions moving in both directions around the
allocation. Measured against the fork point in one environment, the whole suite comes to +0.33%.

`push_back` is the largest riser and the least interesting: two register-to-register moves left in
its inner loop by register allocation, on a program that retires ten instructions per iteration.

The row also carries the corrected element size (an array of a boxed element type reserved the size
of the element's own object where it stores a reference), which shows up nowhere here: no case in
the suite holds an array of a boxed type.

## fd0a7ee93588a9bd19e7ec67dcbd9b7ed26586c6

Opens three kinds of column: the split accesses read from the hardware counters, the processor the
row was measured on, and — for the seven cases that now carry `ref.c` and `ref.rs` — the same
program in C and in Rust, measured the same way.

**No case moves.** Every `-inst` figure is identical to the row above it, which is what the interval
should give: the only change to `src/` between the two rows is the narrow-integer extension at the
FFI boundary (PR #114), and no case here exports a function.

The comparison the reference columns open, in instructions:

| case | Fix | C | Rust | Fix/C | Fix/Rust |
|---|--:|--:|--:|--:|--:|
| modulo_loop | 112,658,350 | 140,161,307 | 112,835,735 | 0.80x | 1.00x |
| arrayrw | 120,570,247 | 150,175,966 | 119,944,182 | 0.80x | 1.01x |
| mandelbrot | 236,876,291 | 249,642,758 | 237,050,646 | 0.95x | 1.00x |
| binary_trees | 784,558,542 | 705,427,716 | 739,079,768 | 1.11x | 1.06x |
| nbody | 1,112,167,494 | 706,162,512 | 602,334,325 | 1.57x | 1.85x |
| levenshtein | 1,007,853,029 | 572,081,751 | 902,130,778 | 1.76x | 1.12x |
| fannkuch | 2,731,406,969 | 1,256,317,448 | 954,912,486 | 2.17x | 2.86x |

The counterparts are built for this host with avx512 left out, as the Fix case is, so the three are
allowed the same instruction set. Fix meets or beats Rust on four of the seven and beats C outright
on three. The two that stand out are `fannkuch` at 2.86x Rust — one array clone per permutation,
which is fixlang#123 — and `nbody` at 1.85x.

`splits` opens across every case. `arrayrw` reads 49,600,017 against 16 for its C counterpart, which
is fixlang#120: the element buffer starts 8 bytes into a 16-byte-aligned allocation, so half of
every 32-byte access crosses a cache line. The instruction count cannot express that, which is why
the case looks like the best in the suite there and is the slowest in wall-clock time.

`modulo_loop` is a new case. A running sum has a closed form the optimizer reaches; the carried
modulo denies it that, and vectorization with it, so what is left is the cost of an iteration.

## 4161bc12449319e678c03ab42eacd25a2142f53c

Adds the `fib` and `levenshtein` cases, so their columns open here at 200,990,240 and 1,007,853,029
instructions. Both tasks are carried by the public cross-language benchmark suites: `fib` is naive
recursion, where the whole cost is the call sequence, and `levenshtein` runs a two-row
dynamic-programming table over every pair of a thousand generated words.

**The other cases move because of what the two rows straddle, not because of anything added here.**
The row above was measured on the `fix-many-args-compile-blowup` branch, which forked before the
wide-return tail call reached main, so `mandelbrot` and `mandelbrot_fold` fall 53.94%, `index_syntax`
3.33% and `cp_lib_conv_zp` 1.99% — the same three cases and the same percentages `12165c4494bf`
records for that work.

Of what is left, every micro-benchmark moves by a constant 1,253 to 1,281 instructions. That is
start-up, and the constant is the difference between the environments the two runs were measured
from — about two variables' worth, at the 600 instructions each cost before `cachegrind.py` fixed
the environment. Four cases move by more: `fannkuch` +1.51%,
`cp_lib_bipartite` +1.10%, `cp_lib_lsegtree` +0.57% and `binary_trees` +0.40%, from the rest of the
work merged between the two rows.

## 6591c2396f24380a346a09577850db263b506225

The `fix-many-args-compile-blowup` branch (PR #106), which stops application inlining from binding a
variable argument to a fresh name each time it pushes an application into a `let`, an `if`, a `match`
or an `eval`. The rewrite is what uncurrying's eta expansion runs per parameter, so the binding per
level made the intermediate expression grow as `2^arity`: compiling a 15-parameter function took 314
seconds, and a 13-parameter one aborted the compiler on the stacks v1.4.0 shipped with.

**The emitted programs are unchanged.** Measured back-to-back at one path against the branch's fork
point `9ed0e65a` — the row before this one in `log.csv`, taken minutes earlier in the same
environment — the
executed-instruction total moves from 16,384,259,453 to 16,384,259,427, or -0.0000%. No case moves by
more than 0.05%; the largest single movement is +0.037% on `sum_by_fix`'s memory accesses, a
300-thousand-access micro-benchmark. Dropping the intermediate binding makes the argument variable
occur once per branch, which could have cost `let_elimination` its "used exactly once" condition and
with it an inlining opportunity. It does not — every path that runs this pass runs let-elimination
afterwards, and the binding the pass used to add is a `let` whose bound expression is a variable,
which is exactly what let-elimination removes, so the two shapes converge before code generation.

What the change buys is compile time: at `-O basic` a 15-parameter function goes from 314 seconds to
2.8, and 25 and 40 parameters, previously out of reach, compile in 2.8 and 3.4 seconds.

## b8d298a0550fc15b9369694b53f9483a57f079d2

The same branch with the x86-64 return-register budget corrected: `tailcc`, the convention Fix
lambdas use there, returns five floating-point values in registers where the C convention returns
four, so a result of exactly five floating-point leaves now comes back in registers instead of
through the out-pointer.

**Nothing moves**: the 41 cases together go from 15,866,111,410 to 15,866,111,433 instructions, 34 of
them byte-identical and the rest within ten instructions of program startup. No case in the suite
returns exactly five floating-point leaves, so the corrected entry changes no code here. It changes
what a reader has to re-measure when the convention or the LLVM version changes.

## 12165c4494bf4cc806f72ec6475cc146b2b36532

The `wide-return-tail-call` branch (PR #109), which keeps a Fix tail call compiled as a jump in the
two cases where the backend used to give up on one: a return value wider than the return registers
(the value now travels through an out-pointer parameter) and a tail call carrying more arguments than
the argument registers (every Fix lambda on x86-64 is `tailcc`). Measured against the previous row
`476f40aa`.

**mandelbrot and mandelbrot_fold drop 53.94%**, index_syntax 3.33% and cp_lib_conv_zp 1.99%. The 41
cases together go from 16,386,181,734 to 15,866,111,410 instructions, **-3.17%**. Every win comes from
the out-pointer half: passing the pointer as an ordinary parameter from the start of the IR lets SROA
delete the buffer wherever the callee inlines, where LLVM's own return demotion used to introduce the
pointer at instruction selection, after every IR pass had already run.

The out-pointer half regresses one case on its own, sort by 0.06%, and that is instruction-selection
churn rather than a cost of the buffer: the strength-reduced remainder in the input-generation loop
comes out two instructions longer per iteration, against which the case's insertion-sort phase gets
cheaper.

The rest of the regressions come from `tailcc`: fannkuch +1.51%, cp_lib_bipartite +1.10%, sort +0.73%,
cp_lib_unionfind +0.68%, cp_lib_lsegtree +0.56%, binary_trees +0.40%, cp_lib_scc +0.39%, get_sub
+0.09%. The cases that compile to one inlined loop are unchanged. Three properties of a
guarantees-tail-calls convention account for all of it on x86-64:

- The callee pops the argument area, and `GetAlignedArgumentStackSize` rounds that area up so that it
  plus the return address is 16-byte aligned. A function with no stack arguments at all therefore
  ends in `ret $8`, and every **non-tail** call site pays one `sub $0x8, %rsp` to restore its own
  frame. get_sub calls `slice_bench` 100,000 times and grows by 100,000 instructions.
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
everything else the callee-popped argument area and the prologue copies of incoming stack arguments,
but a tail call between two different conventions becomes an ordinary call in **both** directions
(measured with `llc -O2`; `IsEligibleForTailCallOptimization` requires the conventions to match once
either side guarantees tail calls). A narrow function tail-calling a wide one, which is what a
monadic chain is made of, would stop being a jump. The convention has to be uniform across
everything reachable by a tail call.

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