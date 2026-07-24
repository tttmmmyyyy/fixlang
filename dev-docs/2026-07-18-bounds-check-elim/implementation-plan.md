# RC-IR simplifier + BCE: implementation plan

Concrete, staged plan derived from `simplifier-design.md` and the pre-experiment. Each phase
is a self-contained, committable, measurable increment. Not started.

## Ground rules

- **Home**: a new `src/rc_ir/simplify.rs`, run between `lower_program` and `insert_rc` inside
  `lower_and_insert_rc` (build_object_files.rs). Reuses `rc_ir/rename.rs` for alpha-renaming.
- **Gate**: a new `Configuration::enable_simplify()` predicate (sibling of
  `enable_borrow_optimization`), initially `= Max`, so the whole simplifier can be toggled for
  A/B measurement.
- **Per-phase verification** (every phase): `cargo test --release` at all opt levels
  (max/basic/none); program outputs unchanged on the speedtest suite; memcheck on the RC-IR
  tests. Byte-identical output is **not** expected past Phase 0. Plus, per phase, a **clone /
  speedtest regression check** (a spurious clone shows up as both a slowdown and extra allocation
  under memcheck), and the **effect measurement** (cachegrind Ir + LLVM-IR inspection for the
  panic branch disappearing and vector ops appearing) against the `--no-runtime-check` ceiling in
  `design.md` §0.
- **Adversarial tests** added alongside the passes: a multiply-used constructor (must NOT fire
  case-of-known-constructor), a boxed value live across a rewritten region, an `eval`-forced side
  (must be preserved).

## Phase 0 — the `Eval` node (foundational, behavior-neutral)

Prerequisite for safe DCE and correct single-use counting.

- ast.rs: add `RcExpr::Eval(RcVar, RcExprNode)` (peer of `Retain`/`Release`/`Destructure`).
- lower.rs `lower_eval`: always emit `Eval(side_var, cont)` (today it only materializes the
  global-reference case).
- Add the arm in every `RcExpr` consumer, mirroring `Release` (recurse into the continuation,
  treat the var as a use): rc_insert.rs, borrow.rs (all walks), provenance.rs, unique_elim.rs,
  rename.rs, print.rs.
- codegen.rs: `Eval(x, k)` = materialize/observe `x` (forces a global's initializer; a local is
  already computed), then eval `k`. Emits no value; `insert_rc` places any release.
- **Verify byte-identical / behavior-equivalent** (Eval is a no-op force marker; with no DCE yet,
  nothing is dropped). Test at all opt levels + a test that uses `eval`.

## Phase 1 — union removal: case-of-case + case-of-known-constructor

The essential core. Delivers `range.fold` (all-scalar state) read-loop vectorization.

- New `simplify.rs` with a fixpoint driver (worklist or iterate-to-no-change over each function
  body) and two rewrites:
  - **case-of-case**: `Let(r, Match(a, arms), k)` -> `Match(a, arms')` where each arm's body is
    threaded into `k` with the arm's result substituted for `r`. (Commuting conversion; the main
    ANF-manipulation care point — splice the continuation into each arm's tail.)
  - **case-of-known-constructor**: `Let(r, Llvm(MakeUnion(v, tag), _), Match(r, arms))` with `r`
    used exactly once -> the `tag` arm with its payload substituted by `v`, dropping both the
    construction and the match. Same for `Destructure` of a just-built `MakeStruct`.
- **Single-use guard** (RC/clone safety): fire only when the constructed value is consumed
  exactly once. Add a use-count helper over `RcExpr` (counts App args, Match scrutinee, Eval,
  etc.).
- Substitute payload variables directly via `rename.rs` (no leftover `let p = v`), so no dead
  construction remains and no boxed value is duplicated.
- Wire into `lower_and_insert_rc` behind `enable_simplify()`.
- **Measure**: `range.fold` / `to_iter` read cases — expect the Option union gone, the check
  folded, and vectorization on the all-scalar (`range.fold`) case. `loop` and `to_iter` do not
  fully land yet (need Phase 2 inline and Phase 3 SROA respectively).

## Phase 2 — inline-single-use

Completes the `loop` idiom by merging a `loop` body into its driver.

- **inline-single-use**: inline a funptr function called exactly once into its call site
  (substitute args for params, alpha-rename via `rename.rs`). This merges a `loop` body into its
  driver so Phase 1's passes then remove the `LoopState` union.
- **copy-prop / DCE — not needed at Max (measured), so omitted.** The AST-level `let_elimination`
  copy-propagates before lowering: a rename-heavy program has 100 `let x = y` copies in the RC IR at
  `-O basic` but 0 at `-O max`, and the post-simplify IR carries no dead bindings (a
  `range.fold`/`to_iter.fold` program: 199 let-bindings, 0 single-use). The Phase-1 rewrites already
  see clean, copy-free ANF. Add these only if a future lowering change lets copies reach the RC IR
  at Max. (`Let(x, Var(y), k)` -> substitute `y`; drop any unused `Let`/`Destructure` — with no
  per-op effect predicate, since Fix is pure except the dedicated `Eval` node.)
- **Measure**: the `loop` idiom (e.g. an explicit index loop) now removes its union; all-scalar
  loop state vectorizes.

## Phase 3 — SROA of aggregate parameters

Delivers `to_iter.fold` and arrayrw's `(i, arr)` state — the big (arrayrw-class) wins.

- When a funptr function's parameter is an unboxed aggregate the body only `Destructure`s, and
  every call passes a `MakeStruct`, split it into scalar-leaf parameters and rewrite the call
  sites. Handle the trailing `cap` capture parameter explicitly (leave it intact).
- The point (per the pre-experiment): hoisting a **boxed pointer** out of the threaded state lets
  `borrow_ify` (which runs later) borrow it read-only, killing the per-iteration RC churn that
  blocks LLVM. Verify SROA output feeds `borrow_ify` well; consider whether teaching `borrow_ify`
  to borrow a boxed leaf *through* the threaded struct achieves the same with less machinery
  (decide here, with measurement).
- **Measure**: arrayrw / `to_iter.fold` — expect the arrayrw-class vectorization win to appear.

## Phase 4 — size normalization (write loops)

- Rewrite `get_size(op(a, ..)) => get_size(a)` for size-preserving `op` (`set`/`mod`/`punch`/
  `plug`), tracing to the root with `borrow.rs::root`. Makes a write loop's bound loop-invariant
  so the scalar-IV check folds.
- **Measure**: `arr.set`-threading write loops now BCE.

## Sequencing rationale

Phase 0 is foundational and behavior-neutral. Phase 1 alone yields a measurable win
(`range.fold`) and validates the whole approach end-to-end in production. Phase 2 and 3 extend
coverage to the idioms that need inlining and boxed-pointer hoisting. Phase 4 covers write loops.
Ship and measure after each; a phase that regresses is reverted as its own commit.

## Deferred / open

- Ordering vs `specialize`/`borrow_ify` interleaving (currently: simplify once before
  `insert_rc`); revisit only if measurement shows lost cross-opportunities.
- The bespoke range analysis (`design.md` approach A) remains the fallback for checks the
  simplifier does not reach (indices unrelated to a size, non-loop checks).
- Definitive validation of a *specific* simplifier output shape, if ever needed, via an
  in-memory `RcProgram` harness run through `insert_rc`->optimize->codegen (heavier than the
  Fix-source pre-experiments, which already exercise `insert_rc`).
