# Reference-count state: drop `Global` where it earns nothing, infer `Local` where it pays

Issue #122. `RcState` has four values and lowering emits only `Unknown`, so every `Retain`,
`Release` and `is_unique` loads the object's `refcnt_state` byte and branches on it. This document
records what that branch costs, what each of the three known states is worth, and the design the
measurements point to — which is not the state inference the issue first proposed.

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

So: **stop marking objects `GLOBAL`, and give a global value a large reference count instead.**

`mark_global` walks the graph a global initializer's value reaches. Instead of writing
`REFCNT_STATE_GLOBAL` into each object's state byte, it writes a large count — `i32::MAX / 2`, since
`refcnt_type` is `i32` — into each object's refcount. A global then behaves as a permanently shared
object:

- it is never unique, because `refcnt == 1` is false, which is what `is_unique`'s `global_bb` arm
  already forces by jumping straight to `shared_bb`;
- it is never freed, because a decrement never reaches zero;
- it is retained and released like anything else, with no state to consult.

In a `threaded = false` build the state byte then has one value, `LOCAL`, and nothing reads it:
`build_branch_by_refcnt_state` becomes an unconditional jump and disappears, along with the load and
the compare. That is the "assume local" compiler measured above — **-2.4% to -13.9%** — obtained by
construction rather than by an analysis that has to be proved sound.

In a `threaded = true` build `GLOBAL` keeps earning its keep, and the state inference the issue asks
for is what matters there: proving `Local` replaces an atomic operation with a non-atomic one, which
is a far larger saving than removing a predictable branch.

### Headroom

`refcnt_type` is `i32`, so a mark of `i32::MAX / 2` leaves 2^30 in each direction. Overflowing needs
2^30 references to one global live at once, each of which occupies a machine word somewhere;
underflowing needs 2^30 more releases than retains. Both are out of reach of a program that fits in
memory. The constant belongs beside `refcnt_type`, so that widening or narrowing the counter moves
it too.

### What it costs

A retain or release of a global now writes the control block where today it skips a branch. At
`-O max` that is the three operations per program measured above. At `-O none` and `-O basic` it is
millions — but those levels also pay the branch on every other operation, and there are far more of
those. **Measure both levels before committing to the change**: the corpus counts above give the
number of operations on each side, and the ceiling measurement gives the saving per operation.

## Consequences for the static-memory plan

Putting the empty-array and string-literal storages in static memory (issue #122's addendum, and PR
#129, which is closed) interacts with this directly.

Today a `GLOBAL` object is never written, which is what would let such a storage live in a read-only
LLVM `constant`. Wave 10 of the bug hunt found that a stray write to one is IR-level undefined
behaviour that the optimizer *deletes* rather than faults on, so read-only is a weak guarantee
already. Under this design the interaction is sharper: **a retain or release does write the control
block, so a statically allocated storage has to be a mutable global with its refcount initialized to
the large value, not a constant.** That is a straightforward initializer to emit, and it removes the
need for the storage to be special-cased at every RC site — but it has to be decided before the
static-memory work starts, not after.

## Stages

### Stage 1 — drop `GLOBAL` from non-threaded builds

1. Add the large-count constant beside `refcnt_type`.
2. `mark_global_one` writes that count into the refcount instead of `REFCNT_STATE_GLOBAL` into the
   state byte. It keeps its traversal and its already-marked check — an object whose count is
   already large needs no second visit, which is what stops a cycle.
3. In a `threaded = false` build, `build_branch_by_refcnt_state` emits an unconditional branch to
   the local arm, and `is_unique` reaches `shared_bb` through the refcount compare it already does.
4. `RcState::Global` stops being produced. It can stay in the enum for threaded builds.

Verification: the full suite at three optimization levels; development-mode valgrind memcheck, where
a global freed by mistake shows up as an invalid free; `benchmark/speedtest`, against the ceiling
above; and the `-O none` / `-O basic` measurement named under *What it costs*.

A test worth writing first, because it is the one thing that changes behaviour rather than speed: a
global holding a boxed value, released more times than it is retained in a loop, and still readable
afterwards. It fails today by construction (the state byte makes the releases no-ops), so it pins
the large count rather than the old mechanism.

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
- LOCAL to GLOBAL: `mark_global`, from `implement_rc_global` on a global initializer's value; it
  marks the whole graph the value reaches. **Stage 1 replaces this with the large count.**
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

Most of the corpus creates no marked object at all — which is also why the state byte earns so little
in a single-threaded build.

## Not in this work

- Moving the empty-array and string-literal storages to static memory, beyond recording above what
  stage 1 requires of them.
- A changelog entry. The observable behaviour does not change, except that a global value now
  survives an unbalanced release rather than being immune to release, which no program can rely on.
