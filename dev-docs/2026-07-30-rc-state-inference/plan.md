# Reference-count state: drop `Global` where it earns nothing, infer `Local` where it pays

Issue #122. `RcState` had four values and lowering emits only `Unknown`, so every `Retain`,
`Release` and `is_unique` loaded the object's `refcnt_state` byte and branched on it. This document
records what that branch costs, what each known state is worth, and the design the measurements
point to — which is not the state inference the issue first proposed.

Measured on `18578264` (main), the 46 `benchmark/speedtest` cases.

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

The largest global count in the corpus is 3, in `cp_lib_dijkstra`.

### The shape that would need `Global`

A table held in a global and read from a hot loop — the Project Euler shape — is the one the issue
expects to dominate the global arm. It does not, at the level that matters. A million-iteration loop
over a global `Array I64`, counting retain plus release on the global arm, in the three ways the
table can be reached:

| -O | referenced directly | passed to a function | bound to a local first |
| --- | --: | --: | --: |
| none | 4,000,012 | 8,000,012 | — |
| basic | 4,000,012 | 6,000,012 | — |
| max | 3 | 3 | 3 |

At `-O max` all three are the same three dispatches for the whole program: borrow inference and
cancellation remove the operations themselves. The global arm is only hot at `-O none` and
`-O basic`, the levels the project keeps deliberately weak.

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

## The design the measurements point to

`GLOBAL` exists to keep reference counting off objects a whole program shares: with threads, an
atomic increment on one hot global would serialize every thread on one cache line. That reason is a
threading reason. In a build with `threaded = false` it buys the numbers above: single-digit skipped
operations, against a branch paid by every operation in the program.

So: **stop marking objects `GLOBAL`, and give a global value a permanent reference count instead.**

`mark_permanent` walks the graph a global initializer's value reaches and writes `PERMANENT_REFCNT`
into each object's count. A global then behaves as a permanently shared object:

- it is never unique, because `refcnt == 1` is false;
- it is never freed, because the count a release reads is never 1;
- it is retained and released like anything else, with no state to consult.

In a `threaded = false` build the state byte then has one value, `LOCAL`, and nothing reads it:
`build_branch_by_refcnt_state` returns the block it was called in, and the load and the compare are
gone. That is the "assume local" compiler measured above — **-2.4% to -13.9%** — obtained by
construction rather than by an analysis that has to be proved sound.

### The exemption was covering an unbalanced count

Reference-count insertion skipped a global operand, on the reasoning that a global needs no retain;
a function it was passed to released the argument it received, like any other. So a boxed global
**lost one count per read** that reached a consuming callee, and what kept it alive was `GLOBAL`
turning that release into a no-op. Measured on a `box struct` global read a million times, counting
the global arm: 6 retains against 1,000,006 releases at `-O none` and `-O basic` (at `-O max` borrow
inference removes the operations, as everywhere else).

A permanent count does not survive that. The drain is not bounded by how many references are live at
once — the reasoning that makes `2^31` look unreachable — but by how many times the program reads
the global, cumulatively, which a loop reaches in seconds.

Retaining on read is the wrong repair, because it puts the retain below the passes that would have
to pair it: `retain_on_read` fired in `Generator::get_scoped_obj`, during code generation, which runs
after `borrow_ify` and `cancel`. The retain could then never be cancelled — and where borrow
inference had already removed the release, it would be a leak.

The repair that fits is that **a global is a variable that is live everywhere**. Reference-count
insertion's three placement rules then give it a retain before an owned use and no release after a
borrowed one, which is exactly what a consuming callee expects; nothing about reading a value
performs a reference-count operation, so `retain_on_read` and the second accessor it distinguished
are gone. Where the callee borrows, no retain is inserted in the first place, which is why the
`-O max` counts above are what they are.

### Headroom

**One count is dangerous, and it is 1.** `build_release_boxed_with` destructs when the count it read
*before* decrementing is 1, and `build_branch_by_is_unique` calls a count of 1 unique; zero is never
tested. Every comparison the compiler makes on a refcount is an equality against 1 — there is no
ordering comparison on one anywhere in the tree — so the counter is used as a bit pattern and the
signedness of `refcnt_type` never enters.

`PERMANENT_REFCNT` therefore sits as far from 1 as the 32 bits allow in both directions, at `2^31`:
reaching 1 from there takes `2^31 - 1` more releases than retains, or `2^31 + 1` more retains than
releases before the count wraps around to it. As an `i32` that bit pattern is negative, which nothing
observes. With reads balanced, the count drifts only by the number of references live at once, each
of which occupies a machine word somewhere, so neither distance is reachable.

The constant belongs beside `refcnt_type`, so that widening or narrowing the counter moves it too.

A permanent count hides an unbalanced one, which is how the drain above went unnoticed: a global
that loses a count per read still works for its first two billion reads. So the guarantee wants a
test that reads the count rather than the program's output — `test_global_refcount` reads it through
FFI and asserts that a loop of consuming reads leaves it where it was.

### Does `GLOBAL` earn anything in a threaded build?

The argument for keeping it is that an atomic increment on one hot global would serialize every
thread on one cache line, which is the reason it was introduced. That argument needs the operations
to exist, and the measurement says they mostly do not: compiling the loops above **with
`--threaded`** gives counts identical to the single-threaded build, three global dispatches for the
whole program. Borrow inference removes the operations, and it does not consult the threading
setting.

That is not proof that `GLOBAL` earns nothing there. Fix's threading is FFI plus `mark_threaded`,
and no program in the corpus shares a global across threads and works it, so the case the argument is
about cannot be measured with what exists today. What can be said is that the argument is unsupported
by any measurement, and that a program would have to defeat borrow inference on a global before it
started paying.

A threaded build therefore marks a global `THREADED`, paying an atomic operation where it paid none.
The state byte survives there, holding two values, for the one decision it is good for — atomic or
not — and **the inference the issue asks for is what matters in such a build**: proving `Local`
replaces an atomic operation with a non-atomic one, which is a far larger saving than removing a
predictable branch.

## Consequences for the static-memory plan

Putting the empty-array and string-literal storages in static memory (issue #122's addendum, and PR
#129, which is closed) interacts with this directly.

A `GLOBAL` object was never written, which is what would let such a storage live in a read-only LLVM
`constant`. Wave 10 of the bug hunt found that a stray write to one is IR-level undefined behaviour
that the optimizer *deletes* rather than faults on, so read-only was a weak guarantee already. Under
this design the interaction is sharper: **a retain or release does write the control block, so a
statically allocated storage has to be a mutable global with its count initialized to
`PERMANENT_REFCNT`, not a constant.** That is a straightforward initializer to emit, and it removes
the need for the storage to be special-cased at every RC site — but it has to be decided before the
static-memory work starts, not after.

## Stages

### Stage 1 — drop `GLOBAL` (done)

1. Reference-count insertion treats a global as live everywhere, so an owned use of one retains it.
   `ScopedValue::retain_on_read` and `get_scoped_obj_noretain` go with it.
2. `mark_permanent_one` writes `PERMANENT_REFCNT` into the count and leaves the object in the state
   its threading calls for.
3. `build_branch_by_refcnt_state` has two outcomes instead of three, and none at all in a
   `threaded = false` build. `REFCNT_STATE_GLOBAL` and `RcState::Global` are gone.

Verified by the full suite, by `test_global_refcount` (which dies of a use-after-free without step
1), and by `benchmark/speedtest`.

### Stage 2 — infer `Local` in threaded builds

Only here does the issue's state inference pay, and only against `Threaded`. A lattice over boxed
leaves, kept **separate from `provenance`'s**:

- `Local` — the leaf cannot have been marked threaded.
- `MaybeThreaded` — top.

`create_obj`'s result is `Local`. `Std::mark_threaded`'s argument, and everything its traversal
reaches, is `MaybeThreaded`; so is anything read out of a container, since a local container can hold
a threaded object. A join takes the top.

Across function boundaries, extend `unique_check_elim::specialize`: its `SpecializationKey` is
`Vec<Uniqueness>` today, and `Vec<(Uniqueness, State)>` reuses the clone naming, the worklist, the
caching and the call rerouting whole. Watch the clone count — the key's cardinality doubles per
parameter, and that pass already governs how many functions reach code generation.

Do not fold this into `provenance`. `LeafOrigin::Fresh` answers a different question, and issue #122
records what happens to code that conflates them.

A wrong `Local` here is a data race, which a single-threaded test cannot see, so this stage waits for
the race-detection work in #96. Before it starts, add under `config.develop_mode` a check at every
specialized operation that reads the state byte and aborts unless it is what the specialization
claimed — and show it fires on a deliberate mis-specialization.

## Where the states come from

For reference, the transitions this design changes:

- `create_obj` initializes every boxed object to refcount 1, `REFCNT_STATE_LOCAL`.
- `mark_permanent`, from `implement_rc_global` on a global initializer's value, gives the whole graph
  the value reaches a permanent count. **This replaced the mark to `GLOBAL`.**
- LOCAL to THREADED: `Std::mark_threaded`, which a `threaded = false` build rejects.
- THREADED to LOCAL: `mark_local_one`, on `build_branch_by_is_unique`'s unique-threaded path.

`mark_permanent` goes through `emit_rc_helper_call`, which emits nothing for a value with no boxed
leaf, so a global of a fully unboxed type produces no marked object at all:

| case | global initializers with a boxed leaf |
| --- | --- |
| fannkuch, binary_trees, levenshtein, sort, index_syntax, fib | 0 |
| nbody_fold | 1 (`Main::init`) |
| cp_lib_lsegtree, cp_lib_unionfind, cp_lib_bipartite | 2 (`Random::_mag01`, `Std::IO::stdout`) |

Most of the corpus creates no marked object at all — which is also why the state byte earns so little
in a single-threaded build.

## Not in this work

- Moving the empty-array and string-literal storages to static memory, beyond recording above what
  stage 1 requires of them.
- A changelog entry. The observable behaviour does not change.
