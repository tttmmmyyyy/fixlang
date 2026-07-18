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
   the body's constructor build and the driver's `match` into one function.
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
   any `let`/`MakeUnion`/`MakeStruct` whose result is now unused. Cleanup that both tidies
   the output and unblocks further constructor cancellation.
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
- **Phase 3 — SROA of aggregate parameters.** Exposes the scalar induction variable;
  expect the arrayrw-class vectorization win to appear here.
- **Phase 4 — size normalization.** Extends the win to write loops.

Ship and measure after each phase against the `--no-runtime-check` ceiling.

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
