# Static reference-count state: what to build, and in what order

Issue #122. `RcState` has four values and lowering emits only `Unknown`, so every `Retain`,
`Release` and `is_unique` loads the object's `refcnt_state` byte and branches on it. This document
records what the branch actually costs, which of the three known states is worth inferring, and the
staging that follows from those two answers.

Measured on `18578264` (main), `-O experimental`, the 46 `benchmark/speedtest` cases.

## What the dispatch costs

A compiler with a counter on each arm of `Generator::build_branch_by_refcnt_state`, at all three
sites that call it (`build_branch_by_is_unique`, `retain_nonnull_boxed`, `build_release_boxed_with`).

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

The largest global count in the corpus is 3, in `cp_lib_dijkstra`. So **`Global` is worth nothing to
infer** — it removes single-digit branches from a program — and the whole prize is proving `Local`,
which removes the state load, the compare and the branch from tens of millions of operations.
`Threaded` cannot arise at all in a build with `threaded = false`, since `Std::mark_threaded` is
rejected there.

## What proving `Local` is worth

A compiler that emits only the local arm, unconditionally — unsound, but it bounds the payoff.
Cachegrind instruction counts, same build path for both sides.

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

So the ceiling on a reference-counting program is a few per cent, reaching 14% where the operations
are dense. That is the budget the analysis has to earn back.

## Where the states come from

- `create_obj` initializes every boxed object to refcount 1, `REFCNT_STATE_LOCAL`.
- LOCAL to GLOBAL: `mark_global`, called from `implement_rc_global` on a global initializer's value.
  It marks the whole graph the value reaches.
- LOCAL to THREADED: `Std::mark_threaded`, which a `threaded = false` build rejects.
- THREADED to LOCAL: `mark_local_one`, on `build_branch_by_is_unique`'s unique-threaded path.
- Nothing leaves GLOBAL.

In a build with `threaded = false`, therefore: **an object is GLOBAL exactly when it is reachable
from a global initializer's value, and LOCAL otherwise.**

`mark_global` goes through `emit_rc_helper_call`, which emits nothing for a value with no boxed
leaf, so a global of a fully unboxed type produces no GLOBAL object at all. Counting the global
initializers that do:

| case | global initializers with a boxed leaf |
| --- | --- |
| fannkuch, binary_trees, levenshtein, sort, index_syntax, fib | 0 |
| nbody_fold | 1 (`Main::init`) |
| cp_lib_lsegtree, cp_lib_unionfind, cp_lib_bipartite | 2 (`Random::_mag01`, `Std::IO::stdout`) |

Programs that create no GLOBAL object at all are common, and they are most of the corpus.

## The direction that has to be right

The two mistakes are not symmetric.

- Widening to `Unknown` is always safe: it is what lowering emits today.
- Narrowing to `Local` wrongly is a heap corruption. A release specialized to `Local` decrements
  without reading the state byte and frees at zero, so a GLOBAL object reaching it frees a global
  initializer's block — and, once the empty-array and string-literal storages move to static memory,
  a block that `free` has no business seeing.

The state byte is the **only** thing guarding `free` today: `build_release_boxed_with` dispatches on
it and the GLOBAL arm returns without touching the refcount. Specializing a release removes that
guard, so the design has to say what replaces it. This plan's answer is the development-mode
assertion in stage 0 — the analysis is the guarantee, and the assertion is what tests the guarantee.

## Stages

### Stage 0 — the assertion that makes the rest testable

Add, under `config.develop_mode`, a check at every operation the later stages specialize: read the
state byte and abort unless it is what the specialization claimed. Costs a load and a compare in
development builds and nothing in a user's build, and it turns a wrong `Local` from a silent heap
corruption into a stop at the operation that was mis-specialized.

Build this first, with a deliberate mis-specialization to show it fires. Every later stage is
verified by running the suite with it armed.

### Stage 1 — the whole-program rule

If the program emits no `mark_global` — no global initializer has a boxed leaf — then no GLOBAL
object exists, and in a `threaded = false` build no THREADED object can, so **every** `Retain`,
`Release` and `is_unique` in the program is `Local`.

The condition is a scan of `prog.globals` for a type with a boxed leaf (`boxed_leaf_paths` already
answers this), computed once. There is no lattice, no fixpoint, and no specialization: the pass
rewrites every `RcState::Unknown` in the program to `RcState::Local`.

This captures the full ceiling on 6 of the 10 measured cases, including `sort` at -13.87%. It also
delivers the codegen for `RcState::Local`, which stages 2 and 3 then reuse.

Its weakness is that it is all-or-nothing: one boxed global anywhere — `Std::IO::stdout` is enough —
turns it off for the whole program. The cp_lib cases, which hold the densest reference counting in
the corpus, are exactly the ones it misses.

### Stage 2 — per-value, for the programs stage 1 rejects

A lattice over boxed leaves, **separate from `provenance`'s**:

- `Local` — the value's leaf cannot be reachable from a global initializer.
- `MaybeGlobal` — top; it may be.

`create_obj`'s result is `Local`. A reference to a global symbol is `MaybeGlobal`. A read out of a
container is `MaybeGlobal`, because a LOCAL container can hold a GLOBAL object — the containment
implication runs one way only, from `mark_global` marking a whole reachable graph. A join takes the
top.

Across function boundaries, extend `unique_check_elim::specialize`: its `SpecializationKey` is
`Vec<Uniqueness>` today, and becoming `Vec<(Uniqueness, State)>` reuses the clone naming, the
worklist, the caching and the call rerouting whole, instead of standing up a second specializer.
Watch the clone count — the key's cardinality doubles per parameter, and that pass already governs
how many functions reach codegen.

Do not fold this into `provenance`. `LeafOrigin::Fresh` answers a different question, and issue #122
records what happens to code that conflates them: once the empty-array storage moves to a module
constant, a `Fresh` value can be GLOBAL, and a release specialized off `Fresh` would free it. Today
`Fresh` does imply `Local`, so the two agree — which is precisely why keeping them apart has to be
deliberate rather than discovered later.

The container read is what decides whether stage 2 is worth its cost. Before building it, measure:
instrument the analysis to report, per case, how many of the dispatches stage 1 misses would be
proved `Local` by "`Fresh`-derived is `Local`, everything else `MaybeGlobal`". If that number is
small on the cp_lib cases, the container read has to be refined — for instance by a whole-program
check that no value derived from a boxed global is ever stored into a container — and that
refinement should be priced separately.

### Stage 3 — `Threaded`, later

Needs a build with `threaded = true`, where the lattice gains a third element and the THREADED to
LOCAL edge in `build_branch_by_is_unique` has to be modelled. A wrong `Local` on a threaded object
is a data race, which a single-threaded test cannot see, so this stage waits for the race-detection
work in #96.

`Global` stays unimplemented. The measurement says it is worth single-digit branches per program.

## Verification

For each stage:

- The full suite at `-O none`, `-O basic` and `-O max`, with the stage-0 assertion armed.
- Development-mode valgrind memcheck, which `test_source` runs automatically — a wrong `Local` on a
  release shows up there as an invalid free rather than as a wrong answer.
- `benchmark/speedtest`, against the ceiling above. A stage that does not move a case whose ceiling
  says it should has not fired, and the reason is a finding.
- `--emit-rc-ir` reads back the inferred state directly: `print.rs` already tags `@local`,
  `@threaded` and `@global`, so a spot check needs no new tooling.

The RC IR validator's balance checking (#105) does not depend on the state, so it cannot catch a
mis-specialization; the stage-0 assertion is what covers this class.

## Not in this work

- Moving the empty-array and string-literal storages to static memory. Today a string literal's
  bytes are a private LLVM constant but its `Array U8` storage is allocated and copied at run time,
  and PR #129 (one shared storage for empty arrays) is closed. When that changes, `Fresh` stops
  implying `Local` and the release path loses the state byte that guards `free` — which is why
  stage 0 comes first.
- A changelog entry. The observable behaviour does not change.
