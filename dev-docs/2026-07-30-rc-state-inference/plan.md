# Reference-count state: keep the runtime flag, prove `Local` where it can be proved

Issue #122. `RcState` has four values and lowering emits only `Unknown`, so every `Retain`,
`Release` and `is_unique` loads the object's `refcnt_state` byte and branches on it. This document
records what that branch costs, what removing it is worth, and the design that reaches for it.

Measured on the 46 `benchmark/speedtest` cases.

## What the dispatch costs

A compiler with a counter on each arm of `Generator::build_branch_by_refcnt_state`, at all three
sites that call it (`build_branch_by_is_unique`, `retain_nonnull_boxed`, `build_release_boxed_with`),
run at `-O experimental`.

**The global arm is taken 0 to 3 times per program. The threaded arm is never taken. Everything
else is the local arm.**

| case | is_unique local | retain local | release local | local total | global total |
| --- | --: | --: | --: | --: | --: |
| cp_lib_lsegtree | 22,333,891 | 63,443,948 | 63,443,967 | 149,221,806 | 5 |
| fannkuch | 40,083,233 | 13,492,900 | 16,758,822 | 70,334,955 | 0 |
| fannkuch_scratch | 17,913,799 | 13,492,900 | 13,492,903 | 44,899,602 | 0 |
| levenshtein | 0 | 8,882,189 | 9,882,190 | 18,764,379 | 0 |
| binary_trees | 0 | 4,194,300 | 6,291,451 | 10,485,751 | 0 |
| nbody, nbody_fold | 10,000,000 | 1 | 7 | 10,000,008 | 2, 3 |
| index_syntax | 2,000,000 | 2,999,002 | 4,002,001 | 9,001,003 | 0 |
| cp_lib_unionfind | 285,233 | 2,499,647 | 2,515,772 | 5,300,652 | 5 |
| sort | 0 | 2,559,622 | 2,569,817 | 5,129,439 | 0 |
| cp_lib_bipartite | 363,330 | 1,844,249 | 1,901,546 | 4,109,125 | 5 |

The largest global count in the corpus is 3, in `cp_lib_dijkstra`. So the state byte is read tens of
millions of times to answer a question whose answer is "local" every time but a handful.

### What removing the dispatch is worth

A compiler that emits only the local arm, unconditionally — unsound as it stands, but it bounds the
payoff. Cachegrind instruction counts, same build path for both sides.

| case | baseline | assume local | change |
| --- | --: | --: | --: |
| sort | 63,406,190 | 54,610,586 | -13.87% |
| levenshtein | 971,924,296 | 911,160,401 | -6.25% |
| nbody_fold | 966,122,785 | 918,122,754 | -4.97% |
| fannkuch | 2,456,217,165 | 2,345,744,356 | -4.50% |
| index_syntax | 413,534,070 | 395,284,306 | -4.41% |
| fannkuch_scratch | 1,347,372,907 | 1,293,818,798 | -3.97% |
| binary_trees | 776,125,189 | 757,250,817 | -2.43% |

`cp_lib_lsegtree` (-14.02%), `cp_lib_unionfind` (-5.34%) and `cp_lib_bipartite` (-2.85%) print
different output under the unsound compiler — they are exactly the cases whose global arm is
reachable — so their figures bound the payoff only loosely.

## The design

**Keep the runtime state byte. Prove `Local` where it can be proved, and fall back to the runtime
dispatch everywhere else.** A proof that is missed costs the dispatch that is paid today, so the
analysis is free to be partial, and no program can be made wrong by an analysis that gives up.

The alternative — dropping the `GLOBAL` state and keeping a global's object alive by a permanent
reference count — was implemented and measured, and is recorded below under *Dropping `GLOBAL`* as
the road not taken.

### What has to be proved

In a `threaded = false` build an object is `GLOBAL` if and only if `mark_global` reached it, which
happens once per global value, over the graph its initializer's value reaches. Every other object is
`LOCAL`. So proving `Local` is proving *this object is not reachable from a non-`LOCAL` source*.

**The proof is a value-level taint analysis** — the sources of non-`LOCAL` objects (reading a
global, `mark_threaded`, `boxed_from_retained_ptr`) taint the values read from them, the taint
propagates along value flow, and an operation on an untainted value emits `Local`. The full design
is `design.md` beside this file. Two earlier shapes of the proof were considered and rejected:

- **A whole-program type closure** ("no global's graph contains this type") proves everything for a
  program with no boxed global, but a single boxed global de-proves its whole type — placing one
  `Array I64` global would slow every `Array I64` operation in the program, a cliff a language
  should not have.
- **Reusing `provenance`'s `Fresh`** conflates two questions: provenance tracks where a value came
  from for *uniqueness*, deliberately dropping to `Unknown` at every boxed-container read, which is
  the wrong default for locality (a value read out of a local-only container is local). Measured
  coverage under that reuse: 0% on `index_syntax`, 56% on `cp_lib_lsegtree` — the misses were
  exactly the container reads.

Measured ceiling for the taint approach (the probe classified every executed reference-count
operation by whether its operand's leaves resolve to allocations or arguments): with argument
resolution, `sort`/`levenshtein`/`fannkuch`/`nbody_fold`/`cp_lib_dijkstra`/`cp_lib_conv_zp` reach
~100%, `cp_lib_scc` 95%, `cp_lib_bipartite` 92%, `cp_lib_unionfind` 68%, `cp_lib_lsegtree` 56%
(container reads recover the rest under the taint rules), `index_syntax` 0% → recovered entirely by
container reads. Without argument resolution the coverage collapses (0–18%), so the interprocedural
part is not optional.

### Stages

1. **Value-level taint, non-threaded builds, `Retain`/`Release` sites** — `design.md`.
2. **The `is_unique` sites**, reached as a co-located op attribute (the `unique_check_elim`
   pattern); `fannkuch`'s dispatches are 57% `is_unique`.
3. **Prove `Local` in threaded builds.** The same lattice, but `mark_threaded` breaks the
   assign-once model through aliases (an object already bound can be marked through another
   reference), so this needs escape reasoning and the race detection in #96 before a wrong proof
   can even be observed. Deferred.

## Dropping `GLOBAL`: the road not taken

The first design dropped the `GLOBAL` state and wrote a permanent count (`2^31`) into a global's
objects instead, on the reasoning that the state buys single-digit skipped operations against a
branch paid by every operation. It was implemented, measured over the whole corpus, and reverted.
What it ran into is worth keeping.

**The exemption was covering an unbalanced count.** Reference-count insertion skips a global operand,
while a function it is passed to releases the argument it received. A boxed global therefore loses
one count per read that reaches a consuming callee — measured at 6 retains against 1,000,006
releases for a million reads — and what keeps it alive is `GLOBAL` turning that release into a no-op.
Drop `GLOBAL` and the read has to retain.

**The retain costs nothing to execute and 24% to emit.** In `nbody` the added retain is one node,
executed once, before a two-million-step loop; the program ran 234 million instructions more.
Uniqueness checks, allocations and copies were identical; what changed is that the array's length
stopped being the constant 5.

The accessor of a global reads a "was it initialized" flag and branches. LLVM's `GlobalOpt` splits
the global's value into pieces and encodes the length as `select(flag, 5, 0)`; `SimplifyCFG` then
turns that select into a **branch**, specializing the path where the length is 5, which is what lets
the loops over the five bodies unroll completely — nine complete unrollings become two without it.
The retain's store has to live at the block where the two paths merge, which pins the merge and
leaves the length a `phi`.

Two ways out were measured and both were worse:

- **Saturate the release at the mark** (so no retain is needed): geometric mean **+1.33%** against
  the retain design's -1.11%, with `random_state` +54%, `gen_random_array` +43% and
  `cp_lib_segtree` +10% — the `select` the saturation adds blocks more than the branch it removes.
- **Re-assert the mark inside the accessor's already-initialized path** (so the store is off the
  merge): `nbody` recovers fully, but every read of a global writes to it, which a global read in a
  hot loop pays for, and in a threaded build it is a non-atomic store to a shared object.

**The finding worth keeping** is about the accessor, not about reference counting: a global's value
is constant-folded only because LLVM converts that `select` into a branch, and one instruction at the
merge is enough to lose it. Emitting a global whose initializer is a compile-time constant as static
data — no flag, no accessor, no merge — would make the folding unconditional. That is issue #122's
addendum and PR #129 seen from a new angle, and it is worth doing on its own.

## Where the states come from

- `create_obj` initializes every boxed object to refcount 1, `REFCNT_STATE_LOCAL`.
- LOCAL to GLOBAL: `mark_global`, from `implement_rc_global` on a global initializer's value; it
  marks the whole graph the value reaches. This is the transition stage 1 proves the absence of.
- LOCAL to THREADED: `Std::mark_threaded`, which a `threaded = false` build rejects.
- THREADED to LOCAL: `mark_local_one`, on `build_branch_by_is_unique`'s unique-threaded path.
- Nothing leaves GLOBAL.

`mark_global` goes through `emit_rc_helper_call`, which emits nothing for a value with no boxed leaf,
so a global of a fully unboxed type produces no marked object at all:

| case | global initializers with a boxed leaf |
| --- | --- |
| fannkuch, binary_trees, levenshtein, sort, index_syntax, fib | 0 |
| nbody_fold | 1 (`Main::init`) |
| cp_lib_lsegtree, cp_lib_unionfind, cp_lib_bipartite | 2 (`Random::_mag01`, `Std::IO::stdout`) |

Most of the corpus creates no marked object at all, which is why the type closure is expected to
prove most of it.

## Not in this work

- Moving the empty-array and string-literal storages to static memory. Stage 1's proof reads
  `Fresh` as "not reachable from a global", which that work would break; the two have to be
  sequenced.
- A changelog entry. The observable behaviour does not change.
