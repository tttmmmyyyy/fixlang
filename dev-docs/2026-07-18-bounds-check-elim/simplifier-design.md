# RC-IR Simplifier: broad optimization + BCE via loop-state scalarization

Status: design only, not implemented. Companion to `design.md` (which analyses the
bounds check directly). This document takes the broader route the measurements point
to: build a general-purpose term simplifier whose composition **scalarizes loop-carried
state**, so LLVM's mature machinery does bounds-check elimination and vectorization —
and, as a side effect, unwraps the pervasive `Option`/`Result`/tuple/union plumbing of
Fix's functional style everywhere else too.

## 1. Rationale

The `--no-runtime-check` experiment (see `design.md` §0) established two facts:

- Removing the bounds check unblocks LLVM vectorization; across the speedtest suite the
  checks are ~37% of instructions, up to 19.9x on arrayrw.
- LLVM eliminates the check on its own **once the loop induction variable is a scalar
  phi** (proven on hand-written IR). What hides it today is that Fix threads the loop
  state (the iterator plus the `LoopState`/`Option` continuation) as a tagged union
  `{i8,[N x i64]}`, reconstructed each iteration.

So the leverage is not a bespoke bounds-check analysis but **exposing the scalar
induction variable**. The transformations that do this — cancel a `match` on a
just-built constructor, cancel a `Destructure` of a just-built struct, split an
aggregate parameter that is only ever destructured into scalar parameters — are
classic, broadly useful simplifications. The user's suggested centerpiece,
**case-of-known-constructor**, is exactly the missing piece. Building these as a small
simplifier improves all Fix code and makes BCE fall out of LLVM.

## 2. Current state (inventory)

Two IR levels exist: the monomorphic typed `Expr` AST (where every `src/optimization/`
pass runs) and the RC IR `RcProgram`. Relevant existing passes:

- **Inlining / beta**: `inline.rs` + `inline_local.rs` + `application_inlining.rs` (AST).
- **Copy propagation + dead-let**: `let_elimination.rs` (AST).
- **Dead global elimination**: `dead_symbol_elimination.rs` (AST).
- **Newtype unwrap**: `unwrap_newtype.rs` (AST, type-driven, single-field unbox struct).
- **Higher-order specialization**: `decapturing.rs` specializes functions — including
  **self-recursive** ones — on their lambda arguments (`worth_specialized =
  self_recursive || inline_at_call_site`), minting `#specialized` names. **This is the
  enabling fact**: after decapturing, a `fold`/`loop`/`loop_iter` call is a specialized
  function whose closure body is a *known* function, so the body-into-driver inline
  below has a concrete target.
- RC IR has **no general term simplifier** — only the RC-specific `borrow_ify` /
  `cancel` / `split_rc_units` and the uniqueness `specialize` (unique-check elimination),
  plus the `provenance` analysis. `rc_ir/rename.rs` provides alpha-renaming/cloning.

Missing everywhere: **case-of-known-constructor**, **case-of-case**, **CSE**,
**scalar-replacement-of-aggregate-parameters**, and any RC-IR term rewriter.

## 3. Where the simplifier runs

Recommended: a new simplifier on the **lowered RC IR before RC insertion** — the
`RcProgram` produced by `lower_program`, in `lower_and_insert_rc`, inserted between
`lower_program` and `insert_rc`.

Why this stage:

- **A-normal form.** Every compound value is `let`-bound to a uniquely-named variable, so
  "is this `match` scrutinee a known constructor?" is answered by following the
  scrutinee's binding to a `MakeUnion` — no look-through of nested expressions as the
  non-ANF AST would need. Case-of-known-constructor and SROA are dramatically simpler and
  more complete here.
- **No RC nodes to preserve.** `lower_program` emits `Let`/`Destructure`/`Match`/`Ret`
  only; `Retain`/`Release` are added later by `insert_rc`. Rewriting before insertion
  frees the simplifier from RC bookkeeping, and `insert_rc` then produces optimal RC over
  the already-simplified, already-scalarized code.
- **The specialized-combinator structure survives lowering.** decapturing ran upstream at
  the AST level, so the lowered loop is a specialized self-recursive funptr function whose
  body is a direct `App` to a known function — ready for the body-into-driver inline.
- **Closest to codegen**, and reuses `rc_ir/rename.rs`.

Full pipeline ordering:

```
lower_program                    (RcProgram, no RC nodes)
[SIMPLIFIER: inline / case-of-case / case-of-known-constructor / copy-prop / DCE / SROA, to fixpoint]
insert_rc                        (adds Retain/Release)
split_rc_units
borrow_ify                       ┐
cancel                           ├ RC-specific (require RC nodes) — Max only
specialize (unique_elim)         ┘
```

The simplifier is a distinct stage *before* `insert_rc`, and thus before
`borrow_ify`/`cancel`/`specialize` (which must run after RC insertion because they read
and rewrite `Retain`/`Release`). This ordering is a benefit, not just a separation: the
simplifier hands the RC passes fewer, simpler, already-scalarized functions (SROA'd scalar
params make `specialize`'s per-parameter keys scalar, and `borrow_ify`'s read-only-leaf
inference simpler). Cross-opportunities in the other direction (a `specialize` clone
exposing new simplifications) are not captured by this run-once-before design; revisit only
if measurement shows it matters.

Alternative considered: extend the AST simplifier (add case-of-known-constructor and
SROA there, reusing the mature AST inliner). Rejected as the primary home because the
non-ANF AST makes constructor-tracing and SROA materially harder, though the AST inliner
remains valuable upstream. The cost of the recommended stage — reimplementing
inline/copy-prop/DCE on `RcProgram` — is small on ANF and yields the first general RC-IR
simplifier, reusable beyond this work.

## 4. The pass set

Each pass is individually a standard, broadly-effective simplification. They run
interleaved to a fixpoint (GHC-simplifier style), because each exposes opportunities for
the others.

1. **Inline single-use / small functions.** Inline a funptr function at a call site when
   it is called once (always beneficial) or is small. The load-bearing case: the loop
   **body** is called exactly once by the loop **driver**, so inline it in — this brings
   the body's constructor build and the driver's `match` into one function. This is a NEW
   RC-level pass, not the existing AST inliner: the AST inliner runs *before* decapturing
   (so it cannot see the specialized-loop structure decapturing creates) and operates on a
   different IR type. A minimal "inline a funptr called exactly once" pass suffices; it is
   not a re-implementation of the cost-based AST inliner. Essential for the `loop` idiom
   (body is a separate function); optional for `fold`/`to_iter` (see the idiom-coverage
   section: their union is already co-located, and LLVM inlines the small callback).
2. **Case-of-case.** When a `match` scrutinizes a variable bound to another `match` (or an
   `if`, already a `Bool` match), float the outer `match` into the inner arms. This is how
   the driver's `match` on the body's result meets the `continue`/`break` constructors the
   body builds in each of its arms.
3. **Case-of-known-constructor** (the centerpiece).
   - `Match(x, arms)` where `x` is bound by `let x = MakeUnion(v, tag)`: replace the match
     with the arm for `tag`, binding its payload to `v`. The union value and the match both
     disappear.
   - `Destructure(x, fields, k)` where `x` is bound by `let x = MakeStruct(f0, f1, ..)`:
     bind each field variable directly to the corresponding `fi`. The struct construction
     and destructure both disappear.
4. **Copy propagation + dead-binding elimination.** `let x = y` -> substitute `y`; drop
   any `let` whose bound variable is unused. Cleanup that both tidies the output and
   unblocks further constructor cancellation. **Effect handling is trivial in Fix**: the
   language has no side effects except `eval`, and aborts (out-of-range, etc.) are not
   guaranteed effects — so an unused `let` is dropped regardless of what its RHS is (no
   per-op purity/abort predicate). The one thing to preserve is `eval`-forcing, which the
   dedicated `Eval` node carries explicitly (see the effects section); DCE keeps `Eval`
   nodes and drops everything else that is unused, with no special case.
5. **Scalar replacement of aggregate parameters (SROA of params).** When a funptr
   function's parameter is an unboxed aggregate that the body only ever `Destructure`s, and
   every call site passes a value built by a `MakeStruct` (or otherwise splittable),
   rewrite the function to take the scalar leaves as separate parameters and rewrite each
   call to pass the leaves. For the loop this turns `loop(state: (I64, Array a))` with
   `destructure state {i, arr}` and a self-call `loop(make_struct(i+1, arr'))` into
   `loop(i: I64, arr: Array a)` with self-call `loop(i+1, arr')` — the scalar induction
   variable LLVM needs. (Handle the `cap` capture parameter of the closure ABI explicitly.)
6. **Iterate to fixpoint.**

## 5. Worked example

The read idiom `arr.to_iter.fold(0, |x, acc| acc + x)` (or `range(0, arr.@size).fold(..)`)
lowers to a specialized self-recursive driver plus a once-called body, with the state a
tagged union carrying the iterator and the accumulator. The passes compose:

- **Inline** the body into the driver -> one self-recursive function whose body ends, in
  each branch, in `continue(state2)` or `break(result)`.
- **Case-of-case + case-of-known-constructor** cancel the `LoopState`/`Option` union: the
  driver's `match` over `continue`/`break` meets the freshly built constructors and
  collapses to a direct branch (`ret result` in the exit arm, the self-call in the other).
- **Case-of-known-constructor on the state struct + SROA of params** turn the iterator/
  accumulator aggregate into scalar parameters. The index becomes a scalar recursive
  parameter.
- After lowering to LLVM and O3 loop-formation, the index is a scalar phi `{0,+,1}`, the
  bound (`arr.@size`) is loop-invariant in the read loop, and SCEV folds
  `_check_range(idx, size)` to the identity — the panic branch vanishes and the loop
  vectorizes. This is the exact shape the hand-written scalar-IV IR proved O3 optimizes.

## 6. Size normalization for write loops

A write loop (`arr.set(i, ..)` threaded through the state) leaves the bound recomputed
on a changing array pointer, which LLVM cannot prove invariant. Add one small algebraic
rewrite, useful as general CSE of sizes:

- `get_size(op(a, ..)) => get_size(a)` for every size-preserving op (`set`, `mod`,
  `punch`, `plug`), tracing to the root array with the existing `borrow.rs::root`.

After this rewrite the size is a single value derived from the loop's original array, so
LLVM hoists it as loop-invariant and the scalar-IV check folds as in the read case. This
is the `get_size`-invariance idea, realized as a rewrite rather than an analysis.

## 7. Relationship to the existing passes and to `design.md`

- **decapturing** (higher-order specialization) is the upstream prerequisite; it already
  runs and needs no change.
- **unique_elim `specialize`** clones funptr functions per uniqueness key; the simplifier's
  inlining/SROA compose with it. Ordering: run the simplifier so that its scalarization
  precedes or follows specialization consistently; the natural choice is to simplify first
  (fewer, simpler functions to specialize), then `borrow_ify`/`cancel`/`specialize` on the
  scalarized IR. Revisit once measured.
- **design.md approach A** (a bespoke interprocedural range analysis that removes
  `_check_range` directly) remains the fallback for checks the simplifier does not reach
  (indices unrelated to a size, non-loop checks). The two are complementary: the simplifier
  captures the loop idioms broadly; the range analysis captures the residue.

## 8. Phasing

- **Phase 1 — case-of-known-constructor + copy-prop + DCE + case-of-case.** Immediately
  useful on all code (unwraps `Option`/`Result`/tuple plumbing), independent of BCE.
  Measure the suite-wide instruction change.
- **Phase 2 — inline single-use (body into driver).** Enables the union cancellation on
  loops.
- **Phase 3 — SROA of aggregate parameters.** Exposes the scalar induction variable.
  **May prove unnecessary** — see the pass-necessity note below: the confirmed hard
  blocker is the *union*, which phases 1-2 remove; LLVM's own SROA scalarizes the residual
  *plain* struct. Add SROA only for the shapes LLVM leaves aggregated.
- **Phase 4 — size normalization.** Extends the win to write loops.

Ship and measure after each phase against the `--no-runtime-check` ceiling.

**Pass necessity (evidence-based, measure-first).** Hand-written-IR `opt -O3` experiments
show the *union* representation (`{i8,[N x i64]}` + tag `select`) is what LLVM cannot see
through, while a *plain* aggregate phi (`{i64,i64}`) LLVM scalarizes on its own (it then
folds the check). So:
- **Essential core = case-of-case + case-of-known-constructor** (remove the union). Once the
  union is gone the residual loop state is a plain iterator/tuple struct that LLVM's SROA
  handles.
- **inline-single-use = essential for `loop`, optional for `fold`/`to_iter`** (their union
  is already co-located; LLVM inlines the small callback).
- **SROA-of-params = essential when the loop state holds a boxed pointer** (`to_iter.fold`'s
  `{arr,idx}`, arrayrw's `(i,arr)`), where it hoists that pointer out so the borrow pass can
  borrow it read-only and kill the per-iteration RC churn that otherwise blocks LLVM; **not
  needed for all-scalar loop state** (`range.fold`). Confirmed by the pre-experiment below.
- **copy-prop + DCE = glue**: needed for the Fix-side fixpoint to see through renamings and
  stay clean, but LLVM does the final cleanup, so they do not change the end result.
Implement the core first, measure against the ceiling, and add inline/SROA/glue only where a
case fails to vectorize.

**Agreed first experiment (no SROA).** Build only case-of-case + case-of-known-constructor
plus the copy-prop/DCE glue (no SROA-of-params, no Fix-side inline), and run it on a
`fold`/`to_iter` read loop. Expectation, already supported by the existing `opt -O3`
experiments: removing the union leaves a *single-level* plain struct as the loop-carried
state (`RangeIterator {next,end}` / `ArrayIterator {arr,idx}`; the accumulator is a separate
scalar parameter), and
- `agg_iv.ll` showed LLVM scalarizes a plain `{i64,i64}` phi and folds the check, while
- the `--no-runtime-check` arrayrw showed vectorization follows once the check is gone (there
  even with the union still present).
So union removal alone should let LLVM fold the check and vectorize, with no SROA. If a case
does not, that identifies the specific shape that needs SROA.

**Pre-experiment result (hand-written Fix through the real `-O max` pipeline).** Rather than
implement the simplifier, the target shapes were written directly as explicit tail-recursive
Fix functions (which lower to the union-free forms the simplifier would produce) and compiled,
inspecting the optimized LLVM IR. This exercises the real `insert_rc`, which turned out to be
decisive:

| shape (hand-written) | loop-carried state | boxed ptr in state? | check | vectorized |
| --- | --- | --- | --- | --- |
| scalar params | `arr` separate; `i`, `acc` scalar | no | folded | yes |
| all-scalar struct (= `range.fold`) | `arr` separate borrowed; `{next,end}` scalar struct | no | folded | yes |
| boxed-ptr struct (= `to_iter.fold`) | `{arr, idx}` threaded | **yes** | **kept** | **no** |

The determining factor is **whether the loop-carried aggregate holds a boxed pointer**. When it
does (`{arr, idx}`), the boxed `arr` is threaded through the rebuilt struct each iteration, and
`insert_rc` places per-iteration reference-count operations on it (load refcount, branch,
maybe free) — that RC churn is what blocks LLVM's SCEV/vectorizer, so the check survives. When
the array is instead a separate borrowed parameter and the threaded state is all-scalar
(`range.fold`'s `RangeIterator {next,end}`), there is no churn and LLVM folds the check and
vectorizes.

So the refined conclusion, correcting the "SROA = insurance for nested aggregates" note above:
**SROA is essential exactly when the loop state holds a boxed pointer** — `to_iter.fold`
(`ArrayIterator {arr,idx}`) and the `loop`/arrayrw `(i, arr)` state — where its job is to hoist
that pointer out into a separate parameter the borrow pass can then borrow read-only,
eliminating the churn. **SROA is not needed when the loop state is all-scalar** — `range.fold`,
and any `loop` whose state carries no boxed value. (For `to_iter`/arrayrw the equivalent effect
might also be reachable by teaching the borrow pass to borrow a boxed leaf through the threaded
struct, without a full SROA; to be decided when implementing.)

## 9. Verification

- Correctness: identical program outputs on the speedtest suite and the test suites at
  all opt levels; memcheck on minilib / project_euler as in the P1/P3 verifications. A
  mis-fired case-of-known-constructor or SROA is a miscompile, so include adversarial
  tests (a `match` whose scrutinee is only *sometimes* a known constructor across a join;
  an aggregate parameter passed an unknown value at one call site).
- Effect: cachegrind Ir vs the pre-simplifier baseline, and inspect the emitted LLVM IR
  for the disappearance of the panic branch and the appearance of vector instructions on
  the array loops. Target = the `--no-runtime-check` ceiling in `design.md` §0.
- Byte-identical output is not expected (the simplifier changes emitted code by design).

## 10. Open questions

- **SROA and the closure ABI.** A funptr function carries a trailing `cap` capture
  parameter; the param-scalarization must split the intended aggregate parameter while
  leaving `cap` intact, and update every call site (direct calls after decapturing).
- **Join points.** Case-of-known-constructor must not fire when the scrutinee is a `match`
  result that is a known constructor in only some arms; the fixpoint must handle partial
  knowledge soundly (fall back to no rewrite).
- **Fixpoint termination.** Inlining plus SROA can interact; bound the iteration and
  confirm termination on nested loops.
- **Ordering vs `specialize`/`borrow_ify`.** Decide by measurement whether the simplifier
  runs once before RC insertion, or also interleaves with the RC-level passes.

## 10b. Reference-counting / clone discipline

Fix uses copy-on-write reference counting: a boxed value is mutated in place while its RC
is 1, and cloned when a mutation meets RC >= 2. The AST `let_elimination` / `inline_local`
passes guard carefully against **extending a boxed value's lifetime**, because a longer live
range can push RC to 2 at a mutation and force a clone. The same discipline must hold here,
in two forms:

- **Delayed-evaluation lifetime extension does not arise in this simplifier.** That hazard —
  `let_elimination`'s reason for its `FreeOccurrenceProbe` guards — comes from substituting a
  *compound* `e0` into a later use, moving `e0`'s evaluation and extending the lifetime of the
  boxed values it references. In ANF every compound is already `let`-bound at its evaluation
  point (pinned), and this simplifier only ever renames *variables* (copy-prop, and the payload
  bindings that case-of-known-constructor introduces). Renaming a variable does not move a
  computation, so referenced boxed values do not shift. All substitutions here are
  `let_elimination`'s safe-and-improving case 1. (This is a direct benefit of ANF, and why the
  AST pass needs the guards and this one does not.)
- **Reference *duplication* is the form that does apply, and needs a single-use guard.** If
  case-of-known-constructor fired on a constructor whose result is used more than once, or
  inline-single-use fired on a function used more than once, a boxed payload/capture would gain
  a second reference (RC 2 -> clone). So both fire **only when the constructed value / callee is
  consumed exactly once (linear)** — which is also inherent to the rewrite (a construction still
  used elsewhere cannot be removed). Loop state is always linear (built, then destructured on the
  next turn), so the loop rewrites fire.

Because the simplifier runs **before `insert_rc`**, it never produces incorrect RC (insert_rc
computes it afterward); the worst case is a structure whose liveness forces an extra clone — a
performance regression, not a bug. The in-place-mutation invariant (e.g. `arr` at RC 1 for
`arr.set`) should be preserved or improved, since the passes only *remove* RC-neutral plumbing
(union/struct wrap and unwrap are moves) — but this must be measured, not assumed. Verification:
clone-count / speedtest regression checks (a spurious clone shows up as both), plus adversarial
tests (a multiply-used constructor; a boxed value live across the rewritten region; a `set` loop
staying in-place after scalarization).

## 11. Idiom coverage: `loop`, `range.fold`, `to_iter.fold`

All three iteration idioms share one shape — a self-recursive driver, the induction
variable buried in an aggregate parameter, and a union built and matched across the
recursion — so the same passes apply. The differences (confirmed from real lowered RC IR)
make `fold`/`to_iter` *easier* than `loop`:

| idiom | accumulator | aggregate holding the index | union to cancel | body separate? |
| --- | --- | --- | --- | --- |
| `loop` | inside the state tuple `(i, acc)` | state tuple `(i, acc)` | `LoopState` (continue/break) | **yes** — needs inline-single-use |
| `range.fold` | already a scalar parameter | `RangeIterator {next, end}` | `Option` (from `advance`) | no |
| `to_iter.fold` | already a scalar parameter | `ArrayIterator {arr, idx}` | `Option` (from `advance`) | no |

Key observation from the real `to_iter` fold driver: `advance` is **already inlined into
the fold driver**, so the `Option` is built (`union_0`/`union_1`) *and* matched in one
function. `case-of-case + case-of-known-constructor` therefore apply directly, with **no
inline-single-use needed** for the union. The accumulator is already a scalar parameter;
only the iterator struct (`RangeIterator`/`ArrayIterator`) needs splitting for the index to
become a scalar recursive parameter. For `range`, the `_check_range` lives in the user
callback (a separate function LLVM inlines); for `to_iter` it lives in the driver. The
`loop` idiom is the one that genuinely needs inline-single-use, because its body is a
separate function.

Net: the sequence covers all three; `loop` needs the full set, `fold`/`to_iter` need less
(union already co-located, accumulator already scalar).

## 12. Effects, `eval`, and the dedicated `Eval` node

**Fix has no side effects except `eval`.** Evaluation is not guaranteed: aborts
(out-of-range, `undefined`, division-by-zero) are not effects the optimizer must preserve,
and IO effects are captured by data dependency (the `IOState` is threaded, so an IO action's
result is used and never dead). The one construct that forces evaluation is `eval`, and even
its guarantee is weak: `eval x; y` forces `x` **at least once if `y` is used**, zero times if
`y` is discarded, and duplication (evaluating more than once) is allowed.

Consequence for the simplifier: **no per-op purity/abort predicate is needed.** DCE drops any
unused binding freely. The only thing to preserve is `eval`-forcing.

The hazard is that lowering dissolves `eval` into the binding stream: `lower_eval` emits the
side's bindings and discards the result, so `eval arr.@(i)` becomes an unused `_check_range` /
`_unsafe_get` chain — which a naive DCE would drop, losing the force. (The existing AST
`let_elimination` avoids this only because `eval` is a distinct `Expr::Eval` node at the AST
level; the hazard is *born* in lowering.)

**Decision: add a dedicated `RcExpr::Eval(RcVar, RcExprNode)` node** (rather than a flagged
`Let` or a naming convention). Rationale:
- It is consistent with `RcExpr`'s statement-node design — a peer of `Retain`/`Release`/
  `Destructure` (do something to a var, continue).
- DCE stays exception-free: `Eval` is not a `Let`, so "drop unused `let`s" needs no special
  case for "unused but must keep."
- Correctness by construction: `Let`/`Match`-rewriting passes (case-of-known-constructor,
  copy-prop, inline, SROA) forward through `Eval` and cannot mistreat it. A `Let`+marker
  would put the preservation burden on every such pass forever.
- Clear per-stage meaning: `insert_rc` treats `Eval(x)` as a use of `x` (a release point like
  `Release`); codegen emits nothing (the value is already materialized); the weak semantics
  make it a normal reachability root (droppable if its continuation is dead) and duplication-
  safe (case-of-case may copy it into branches).

Cost: one arm per `RcExpr` match across ~8 files (`lower`, `rc_insert`, `borrow`, `provenance`,
`unique_elim`, `rename`, `codegen`, `print`) plus the new simplifier — each a mechanical
"recurse into the continuation," mirroring `Release`. `lower_eval` changes to always emit
`Eval(side_var, cont)` (today it only materializes the global-reference case).
