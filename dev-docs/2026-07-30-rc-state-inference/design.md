# Locality inference: value-level taint from the sources of non-LOCAL objects

The design for stage 1 of `plan.md`: keep the runtime reference-count state byte, and prove
`RcState::Local` per reference-count operation by a may-analysis over the RC IR. An operation the
analysis cannot prove keeps today's runtime dispatch, so a proof that gives up costs nothing and a
program cannot be made wrong by imprecision — only by unsoundness, which is what this document
argues away.

## The property

For a binding `x` and a boxed-leaf path `π` of its type:

> `shared(x.π)` — the object `x.π` points at, **or any object reachable from it**, may be in a
> non-`LOCAL` reference-count state at some point while the binding is live.

A `Retain`/`Release` on unit path `π` of `x` may emit `RcState::Local` iff `shared(x.σ)` is false
for every boxed leaf `σ` at or under `π`. The reachability closure is deliberate: it makes the
property compositional. Projecting out of a value, injecting into a fresh aggregate, and reading an
element out of a boxed container all become unions of operand taints, with no aliasing questions —
the closure of "what this value can reach" is exactly what survives every one of those operations.

The cost of the closure is that a fresh container holding a shared element is itself reported
shared (`let a = [g.@(0)]` — `a`'s storage is provably local, but the analysis taints it). That
imprecision is the price of never asking an aliasing question, and it errs on the sound side.

## The sources

An object leaves the `LOCAL` state through exactly three doors. The state byte has four writers —
`create_obj` (initializes to `LOCAL`), `mark_global_one`, `mark_threaded_one`, and `mark_local_one`
(a `THREADED`-to-`LOCAL` narrowing on the unique-threaded path, which removes sharedness rather
than adding it) — so enumerating the callers of the two marking writers enumerates the doors:

1. **Reading a global value.** `implement_rc_global` runs `mark_global` over the whole graph the
   initializer's value reaches, after evaluating it. Every use of a global symbol as a value is a
   read of that marked graph.
2. **`Std::mark_threaded`.** Marks its argument's graph `THREADED`. A `threaded = false` build
   rejects it at compile time, so in such builds this door does not exist.
3. **`Std::boxed_from_retained_ptr`.** Reconstructs a value from a raw pointer, about whose state
   nothing is known — the pointer may have crossed threads or come from a global's graph.

Checked and rejected as sources: `String::unsafe_from_c_str_ptr` copies into a fresh array;
`FFI_EXPORT` admits only non-aggregate scalars (#114), so no boxed value enters through an exported
function's arguments; the C runtime constructs no reference-counted objects; `argc`/`argv` are raw
scalars, and `Std::get_args` builds fresh strings; `boxed_to_retained_ptr` lends a pointer out
without changing any state (the value's return trip is door 3). A future fourth door is the
static-memory work (issue #122's addendum): a statically allocated storage never passes through
`create_obj`, so that work must declare its state and revisit this list.

**Timing makes the global door a read-side door.** `mark_global` runs after the initializer's value
is fully built, so every reference-count operation that executes *during* initialization — inside
the initializer's own body and inside every function it calls — operates on objects still `LOCAL`.
The taint therefore attaches to *reads of the global symbol*, not to the code that built the value,
and initializer bodies are analyzed and annotated like any other code. A global read inside another
global's initializer is already marked by then (its accessor completed first), and the ordinary
read rule taints it.

## Soundness depends on `threaded = false`

The analysis assigns a binding's taint once. That is sound only if no operation can transition an
*already-bound, aliased* object out of `LOCAL`:

- In a `threaded = false` build, the only marking transition is `mark_global`, and its subject is an
  initializer's result graph. An initializer takes no arguments and reads only globals, so no object
  a live local binding points at can be swept into the marking — the graph consists of objects the
  initializer built (which no one else holds; even a raw pointer stashed via FFI during
  initialization re-enters through door 3) and objects of other globals (already marked, already
  tainted at their read).
- With `mark_threaded` the argument breaks: `retain a; eval a.mark_threaded; ... use a` marks the
  object `a` still points at, while `a`'s taint was assigned before the call. Proving `Local` in a
  threaded build therefore needs escape reasoning about what may flow into a `mark_threaded`
  call — deferred with the rest of the threaded stage (plan stage 3, gated on #96).

**Stage 1 runs the annotation only when `config.threaded` is false.** Threaded builds keep every
dispatch, exactly as today.

## The lattice

Per boxed leaf: a set of origins, `Origin ∈ { Ext, Arg(input, σ) }`.

- The empty set proves the leaf local.
- `Ext` — tainted by one of the doors, or by anything the analysis does not track (an indirect
  call's result, a cross-unit call's result, a function callable from outside the unit).
- `Arg(i, σ)` — as tainted as leaf `σ` of input `i` (parameters, then the capture). Present only
  transiently: the interprocedural pass resolves it against concrete caller taints.

Join is set union. Leaf paths are bounded by the type and inputs are finite, so the lattice is
finite and the fixpoint terminates.

## Transfer

Per RC IR node, over an environment mapping each local binding to its per-leaf taint. A use of a
global symbol as a value ("global atom") taints every leaf of the read value `Ext` — this is door 1
and it is the *only* rule that consults whether a name is local.

- `let x = y` (move): copy.
- `let x = <global atom>`: all leaves `{Ext}`. (A funptr-typed global has no boxed leaf and taints
  nothing; a closure-typed global's capture leaf is `Ext`, which is correct — its capture object is
  marked.)
- `let x = Closure(f, caps)`: the capture leaf gets the union of the captures' full taints.
- `let x = App(callee, args)`:
  - callee names a function of this unit's `RcProgram` — a direct call; see interprocedural below.
  - anything else (a closure-valued variable, a function of another unit, a global closure value):
    all result leaves `{Ext}`.
- `let x = Llvm(op, args)`: by the op's *locality flow*, a new `LLVMGen` method:
  - **Default: every result leaf gets the union of every operand's every leaf-taint, and no `Ext`.**
    This is sound for any operation that can only allocate fresh objects and rearrange objects
    reachable from its operands — which is every builtin except the doors. Reads out of boxed
    containers (`array_get`, getters on boxed structs) are correct under the default by the
    reachability closure: the element was reachable from the container.
  - Overrides, co-located with each op in `builtin.rs` (the same pattern as the op-specific
    attributes):
    - `boxed_from_retained_ptr` (and `mark_threaded`, for completeness): all leaves `{Ext}`.
    - The unboxed-aggregate plumbing ops — struct/tuple make, get, set, mod, punch/plug-in, union
      make/as/mod, capture projection — route per leaf (result leaf `.i.σ` from operand leaf `σ`
      and so on), so that a tuple carrying a tainted and an untainted component keeps them apart.
      This is where loop states (`(tree, rng, sum)`) live, so this precision is what the hot loops
      see. The set is enumerated, small, and each override is a few lines.
  - The flow is a method of its own rather than a reading of `result_prov`: provenance answers a
    different question (`Unknown` there marks untracked sharing, not sharedness of state), and
    deriving one from the other would let a uniqueness-motivated edit silently change a soundness
    argument.
- `Destructure`: boxed container — every field gets the container leaf's taint; unboxed — project
  per field.
- `Match`: payload as in `Destructure` (per variant); the arm results join.
- `Retain`/`Release`/`Eval`: no change to the environment.
- `ret x`: the function's result taint joins `x`'s.

## Interprocedural: one concrete fixpoint, monovariant

Two maps, iterated together to a fixpoint over the unit's functions:

- `input_taint[f]` — per input leaf, the join of the argument taints over every *direct* call site
  of `f` seen so far, plus the conservative seeds below.
- `result_taint[f]` — `f`'s result taint under the current `input_taint[f]`.

Each round interprets every function body under its current `input_taint`, using `result_taint` at
direct call sites and joining the visited call sites' argument taints into the callees'
`input_taint`. Both maps grow monotonically in a finite lattice, so the iteration terminates. No
symbolic summaries: provenance needs them because uniqueness resolves per call site through
specialization; locality here is monovariant — one taint per function input, joined over callers.

Conservative seeds for `input_taint`:

- A function reachable by indirect call — one that some `Closure(f, …)` names — takes `Ext` on its
  *parameter* leaves (its capture leaf joins the taints of the closure sites, which are visible).
- A function callable from outside the unit takes `Ext` on every input leaf. In a single-unit build
  that set is the entry point and the FFI-exported functions, whose inputs carry no boxed leaf
  (`main` takes none; exports are scalar-only), so nothing is seeded. In a multi-unit build every
  program symbol is externally callable and gets seeded; the unit-local clones — the specialized,
  borrow, uncurried and decapturated versions, which are where the hot loops live — are not program
  symbols and stay precise. This is the honest cost of separated compilation; a cross-unit summary
  store is future work if measurement ever demands it.

The whole speedtest corpus compiles as one unit, and at `-O max` the hot path is direct calls all
the way down — decapturing bakes the loop body's identity into the specialized `fold` clone, whose
body calls the loop body *by name* (verified on the RC IR dump: `fold#…#specialized_…` calls
`main#…#decap_lam1#funptr3#borrow` directly). So the monovariant analysis reaches the sites that
matter without any indirect-call machinery. Where monovariance does lose (one helper called with
both tainted and clean arguments), the fallback is polyvariance by widening
`unique_check_elim::specialize`'s key — measured first, built only if needed.

## Annotation

After the fixpoint, one more interpretation of each function and each global-initializer body
records, at every `Retain(x, π, Unknown)` / `Release(x, π, Unknown)`, whether every leaf at or
under `π` is untainted; if so the state becomes `RcState::Local`. Nodes stay `Unknown` otherwise.

Placement in the pipeline: the last RC IR pass, after `specialize`, immediately before
`implement_rc_program` — the clones must exist and the reference-count operations must be final.
Gated like the other Max-and-above passes, and additionally on `!config.threaded`.

## Code generation

`implement_rc_program`'s `Retain`/`Release` arms currently assert `Unknown`. They gain the `Local`
arm:

- `Retain(Local)`: non-atomic increment, no state load, no branch (the body of today's `local_bb`).
- `Release(Local)`: non-atomic decrement, destruct when the count read was 1 — again today's local
  arm without the dispatch around it.

The null-check wrapping (`skip_null_check`, dynamic-object checks) is orthogonal and unchanged. The
type traverser functions that destruction calls keep their internal `Unknown` dispatches — a
per-state traverser family would double the emitted traverser code and is not stage 1.

The `is_unique` dispatch (the third reader of the state byte, inside the unique-check ops) is
**stage 2**: those reads happen inside `LLVMGen::generate` bodies, so the annotation has to reach
them as an op attribute — the same co-located-attribute pattern `unique_check_elim` already uses to
constant-fold checks on proven-unique operands. Worth doing: `fannkuch`'s dispatch count is 57%
`is_unique`, `cp_lib_lsegtree`'s 15%. Stage 1 ships without it and measures.

## Verifying the analysis, not just the code

- **A `develop_mode` runtime assertion**: at every operation annotated `Local`, load the state byte
  and abort unless it is `REFCNT_STATE_LOCAL`. The whole test suite runs under `develop_mode`, so
  every annotated site is dynamically checked on every test program — this is the plan's
  "specialized operation checks its claim" item, delivered with the stage instead of after it.
  Demonstrate once that it fires, by deliberately mis-annotating a site and watching the suite
  fail; then remove the sabotage.
- **Coverage measurement** (temporary probe, reverted after reading): count executed `Local` vs
  `Unknown` operations over the speedtest corpus and set it against the ceiling table in `plan.md`
  (`arg`+`local` row) — the fraction the monovariant fixpoint failed to resolve is the polyvariance
  case, with numbers.
- **The full suite** at all three levels, and **`benchmark/speedtest`** against the current `main`
  row, watching the knife-edge cases (`nbody`, `nbody_fold`) that flipped under the abandoned
  design.

## Files

| file | change |
| --- | --- |
| `src/rc_ir/locality.rs` (new) | lattice, transfer, fixpoint, annotation |
| `src/ast/inline_llvm.rs` | `LLVMGen::locality_flow` with the union default |
| `src/fixstd/builtin.rs` | the door overrides and the aggregate-plumbing overrides |
| `src/rc_ir/codegen.rs` | `Local` arms in `Retain`/`Release`; the `develop_mode` assertion |
| `src/generator.rs` | state-aware retain/release emission helpers |
| `src/build/build_object_files.rs` | run the annotation after `specialize` (Max+, non-threaded) |

`RcState::Local` and the `@local` dump form already exist; `validate` is state-agnostic.

## Out of scope

- Threaded builds (plan stage 3; the aliasing argument above is why).
- The `is_unique` sites (stage 2, attribute plumbing).
- Cross-unit summaries.
- Per-state traverser variants.
- A changelog entry: observable behaviour does not change.
