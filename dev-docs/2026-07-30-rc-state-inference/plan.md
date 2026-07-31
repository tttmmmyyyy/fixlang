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
`LOCAL`. So proving `Local` is proving *this object is not reachable from a global*.

**By type.** `mark_global` traverses by type, so an object whose type does not occur in the type
closure of any global's value is never marked. Collect the globals whose initializers carry a
reference-counting unit, close their types under the traverser's reachability (fields, array
elements, union variants, a closure's capture object), and every reference-count operation on a type
outside that closure emits `Local`.

This is the whole proof for a program with no such global — the closure is empty — but it does not
turn into a cliff when one appears: a global of type `Array U64` puts `Array U64` and what it reaches
into the closure and leaves every other type proved. What it does not survive is a global holding a
**closure**, whose capture is a `#DynamicObject` — one such global puts every capture object in the
closure. How much that costs is for the measurement to say.

**By provenance.** For the types the closure does contain, `provenance` already computes what is
needed: a leaf whose origin is `Fresh` was allocated by the code being compiled and so is not
reachable from a global. `Fresh` implies `Local` today; it stops implying it if a `Fresh` value can
ever be statically allocated (issue #122's addendum), which is a reason for the two pieces of work to
know about each other. A leaf that is `Arg(i, path)` resolves against the caller, which
`unique_check_elim::specialize` already has the machinery for — its `SpecializationKey` is
`Vec<Uniqueness>`, and widening it carries the clone naming, the worklist, the caching and the call
rerouting along.

### Stages

1. **Prove by type.** Compute the closure, thread it to the three dispatch sites, emit `Local` where
   the type is outside it. Measure what fraction of the corpus's operations it proves — in
   particular whether `cp_lib_lsegtree`'s 149 million are inside or outside.
2. **Prove by provenance.** `Fresh` leaves, then argument leaves through specialization. Only worth
   starting once stage 1's measurement says what is left to win.
3. **Prove `Local` in threaded builds.** The same analysis, against `Threaded` rather than `Global`,
   where it replaces an atomic operation with a non-atomic one rather than removing a predictable
   branch. A wrong answer here is a data race, which a single-threaded test cannot see, so it waits
   for the race detection in #96, and wants a `develop_mode` check that reads the state byte at every
   specialized operation and aborts unless it is what the specialization claimed.

`RcState::Local` has to be implemented in code generation before any of this: today the `Retain` and
`Release` arms of `implement_rc_program` assert that the state is `Unknown`.

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
