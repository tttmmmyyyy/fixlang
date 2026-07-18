# Bounds-Check Elimination (BCE) — Design

Status: design only, not implemented. Targets the RC IR back end on branch
`unique-check-elim`, alongside the existing unique-check elimination.

## 1. Goal and motivation

Remove the runtime array bounds check when the compiler can prove the index is
in range. The check is a first-class RC IR op `_check_range(idx, size)`
(`InlineLLVMArrayCheckRange`) that returns `idx` and, under
`config.runtime_check()`, panics unless `0 <= idx < size`. It is emitted by the
safe accessors `Array::@`, `set`, `mod`, `act`, `get` (each inlines
`_check_range` at Max). A smaller secondary target is `_check_size(size)`
(`InlineLLVMArrayCheckSize`, which asserts `size >= 0`).

Three motivations, in order of weight:

1. **Unblock LLVM O3 loop vectorization.** A bounds check is a conditional
   branch to a panic in the middle of the loop body; it blocks the vectorizer.
   Read loops whose checks are absent vectorize at O3 and run several times
   faster (a sum-over-array case measured ~8.8x fewer instructions when its
   read-loop check was gone). This is the largest potential win.
2. **Close the direct check cost.** On this branch the safe checked `arrayrw`
   runs ~2.8% over the check-free unsafe version; that residual is the bounds
   check unique-check elimination cannot touch.
3. **Enable deleting the surviving unsafe primitives.** Removing
   `_unsafe_get_linear_bounds_unchecked_unretained` and
   `_unsafe_set_bounds_uniqueness_unchecked_unreleased` requires that safe
   `@`/`set` reach parity on the provable cases; without BCE, migrating the Std
   builders and cp-library to the safe ops reintroduces the check. See the
   delete-unsafe-primitives plan.

## 2. The elimination target

`_check_range(idx, size)` lowers to `RcRhs::Llvm(InlineLLVMArrayCheckRange, [idx, size])`.
Its `generate` emits the panic guard only when `config.runtime_check()` and then
returns `idx` unchanged. So the op is a conditional identity: eliminating the
check means keeping the identity and dropping the guard.

Elimination mechanism is byte-for-byte the unique-check pattern already in the
tree:

- Give `InlineLLVMArrayCheckRange` a `checked: bool` field (default `true`),
  exactly like `force_unique` on `InlineLLVMArraySetBody`.
- Add two `LLVMGen` trait methods with defaults in `inline_llvm.rs`:
  `range_check_operands(&self) -> Option<RangeCheckOperand>` (which operand slot
  is the index and which is the size — the analogue of `unique_check_operand`)
  and `assuming_in_range(&self) -> Box<dyn LLVMGen>` (the check-free variant,
  returning a clone with `checked = false`; the default is `unreachable!`, as
  with `assuming_unique`).
- BCE swaps a proven-safe op for `gen.assuming_in_range()` in place. Codegen is
  unchanged otherwise: the op still returns `idx`, it just omits the guard.

`_check_size` gets the analogous `assuming_nonneg`.

## 3. Why this is hard in Fix

Fix has no structured loops. Every `loop` / `fold` / `loop_iter` / `range`
iteration lowers to a **tail-self-recursive `RcFunc`**. The loop index is a
*parameter*, carried inside the loop-state aggregate — a tuple for `loop`, an
`ArrayIterator { arr, idx }` or `RangeIterator { next, end }` for the iterators —
and destructured at the top of the body. The only loop structure surviving in RC
IR is the tail self-`App`; there is no loop node and no induction-variable
metadata.

Two further facts, confirmed from real RC IR dumps:

- The bound `arr.@size` is **recomputed every iteration** (`Array::get_size`),
  not hoisted.
- The array is **threaded through the loop state**: `arr.set(i, ..)` returns a
  fresh array that is repacked into the next state and destructured out on the
  next turn.

So proving `0 <= idx < arr.@size` at a check is inherently a **loop-carried
(inductive) invariant** problem, and it depends on knowing that
`set` / `mod` / `punch` / `plug` **preserve size**, so the `get_size` of the
threaded array equals the original bound.

This is also precisely why LLVM O3 leaves the check in place: after `arr.set`,
LLVM sees a new SSA array value and cannot prove `get_size(arr') == get_size(arr)`
(it does not model the set intrinsic's size preservation), nor the index
invariant across the (pre-TCO) self-recursion. The Fix compiler knows both. That
is the case for doing BCE here rather than leaning on the back end.

## 4. What BCE can and cannot prove

**Provable — the dominant real idioms.** The index is a monotone counter
produced by a standard iterator or loop and checked against the *same array's*
size (or a range end equal to it):

- `arr.to_iter. ...` — `ArrayIterator { arr, idx }`, advance guards
  `idx == arr.@size`, accesses `arr.@(idx)` in the has-element arm.
- `Iterator::range(0, arr.@size). ...` — `RangeIterator { next, end }` with
  `end` = `arr.@size`.
- `loop` counting `i` from 0, body accessing `arr.@(i)` under a guard
  `i == arr.@size` or `i < arr.@size`.

Safety rests on `0 <= i` (starts at 0, steps by +1) and `i < bound` (guard),
with `bound` size-linked to the accessed array. All three reduce to the same
inductive invariant `0 <= i <= S` maintained across the recursion.

**Not provable, and correctly so.** A loop whose bound is unrelated to the
accessed array's size — for example `for i in 0..1000 { arr.set(i, ..) }` where
`arr.@size` is not known to be `>= 1000` — may be genuinely out of range, so BCE
must leave the check. (A synthetic `arrayrw` with a hardcoded `1000` is this case
unless `arr.@size` is a known constant `>= 1000`.) The win comes from the
"iterate over the array's own indices" pattern, not from arbitrary bounds; the
design states this plainly rather than over-claiming.

## 5. Analysis design — interprocedural inductive range analysis

Mirror the two existing RC IR facilities: the abstract interpreter in
`provenance.rs` (forward walk + two-phase call-graph fixpoint) and the transform
in `unique_elim.rs` (snapshot-at-op, then specialize).

### 5.1 Abstract domain

For each `I64` variable, a **relational range**: a lower bound `lo` in
`{0, known-constant}` and an upper relation `v < S` or `v <= S`, where `S` is a
**size token**.

- A **size token** is the identity of a `get_size(arr)` value, keyed by the
  *root array* of `arr`. Root tracing reuses and extends `borrow.rs::root`
  (follow move-binds, projections, union payloads) with the size-preserving ops
  `set` / `mod` / `punch` / `plug` and loop-state threading. Two `get_size`
  calls with the same root yield the same token and are therefore comparable. A
  constant size (from `fill(n, ..)` / `empty(n)` / an integer literal) is a token
  bound to that constant.
- Track constant integer values (`InlineLLVMIntLit`) and simple affine facts
  (`v = w + 1`, `v = w - 1`) — enough to carry the `+1` / `-1` step through the
  induction.

The domain is a finite-height lattice with a **widening** (widen ranges toward
the guard bound) so the loop fixpoint terminates.

### 5.2 Interpretation

A forward abstract interpreter over `RcExpr`, reusing the
`interp` / `interp_inner` / `interp_match` structure (environment threaded by
value, Match arms joined at their merge point).

- Integer ops: `IntLit` gives a constant; `IntAdd` / `IntSub` with a constant
  operand gives an affine shift; `IntEq` / `IntLessThan` / `IntLessThanOrEq`
  produce a `Bool` that feeds a `Match`.
- **Guard refinement.** In the arm where `i < S` is true — or where `i == S` is
  false combined with a known `i <= S` — refine `i`'s upper bound to `< S`. This
  is where the `ArrayIterator`/`RangeIterator` guard pays off.
- `get_size(arr)` yields the size token for `root(arr)`.
- Size preservation: when `arr' = set/mod/punch/plug(arr, ..)`, record
  `root(arr') = root(arr)` so the two arrays share a size token.
- **Loop-carried fixpoint.** Seed parameters at top, interpret the body, and at
  the tail self-`App` feed the argument ranges back as the next parameter
  approximation; iterate with widening until stable. This is what establishes
  `0 <= i <= S` as inductive: the body increments `i` by 1 only under the guard
  `i < S` / `i != S`.
- Interprocedural: as `provenance.rs` threads `call_args` / `effects`, a callee's
  parameter ranges are the meet of its call-site argument ranges, over a
  two-phase call-graph fixpoint. A loop is the self-call special case.

### 5.3 Snapshot at the check op

At each `_check_range(idx, size)` op, snapshot (keyed by the op's result var, the
analogue of `provenance.rs::op_containers`) the pair `(range(idx), token(size))`
at that program point. The verdict `provably_in_range` holds when
`range(idx).lo >= 0` and `range(idx)`'s upper relation is `< token(size)`.

## 6. Transform design

New file `src/rc_ir/bce.rs`, entry `eliminate_bounds_checks(prog, type_env) -> RcProgram`:

1. Run the range analysis (Section 5) to get per-op snapshots.
2. Walk each body; at a `_check_range` op whose snapshot proves the index in
   range, swap in `gen.assuming_in_range()`. This is the `maybe_elide` shape from
   `unique_elim.rs`.

**Specialization.** Safety can depend on the caller's argument ranges — a helper
`get(i, arr)` is safe only when called with `i < arr.@size`. Reuse the
`unique_elim.rs` machinery: a `(FuncRef, Key)` worklist, `beneficial` gating,
keep-the-canonical-version-and-add-clones (so codegen stays total), and the
renamers in `rename.rs` (with a fresh marker, e.g. `"b"`). The Key is a small,
finite quantization of per-parameter range facts, for example
`{ Unknown, NonNeg, InRangeOf(param j) }`. This is the heaviest component and is
deferred past the MVP (Section 8).

## 7. Pipeline placement

Add to `optimize_rc_program` (`build_object_files.rs`), Max-gated, **after**
`specialize`, so BCE sees the specialized clones unique-check elimination
produced:

```
split_rc_units -> borrow_ify -> cancel -> specialize -> eliminate_bounds_checks
```

An independent `enable_bce_optimization()` predicate (sibling of
`enable_borrow_optimization`) lets BCE be toggled separately for measurement.
BCE removing the check branch then feeds LLVM `default<O3>`, which performs the
loop-formation (via tail-call elimination) and vectorization.

## 8. Phasing

- **Phase A (MVP, highest value).** Self-recursive loop functions only. Prove
  `0 <= idx < size` for the monotone-counter-vs-same-array-size pattern via the
  loop-carried fixpoint plus size-token(root). Covers `arr.to_iter`,
  `range(0, arr.@size)`, and `loop` over an array's own indices. No cross-function
  specialization. Ship and measure.
- **Phase B.** Interprocedural specialization for helpers that receive an
  in-range index, mirroring `unique_elim.rs`'s worklist.
- **Phase C.** `_check_size` elimination, more affine relations, `get_sub` /
  slice patterns, and user-written `if i < n` loops.
- **Follow-up.** Revisit deleting the surviving unsafe RMW primitives once the
  safe builders reach parity on the provable cases.

## 9. Alternatives considered

- **Rely on LLVM O3 alone.** Insufficient (leaves the ~2.8% and does not
  vectorize the threaded-array read loop): LLVM cannot prove
  `get_size(arr') == get_size(arr)` across the size-preserving `set`, nor the
  induction invariant across the pre-TCO recursion. The Fix compiler has both
  facts. Rejected as the sole mechanism; retained as the downstream vectorizer.
- **Syntactic combinator pattern-matching** (recognize the `ArrayIterator` /
  `RangeIterator` shapes and hard-code safe access). Fragile — it breaks under
  inlining and specialization variation and does not cover user-written loops.
  The range analysis subsumes it robustly. Rejected.
- **Full ABCD (Bodik, Gupta, Sarkar).** The classic algorithm assumes SSA with
  phi nodes and a structured intra-function CFG. Fix's loop is interprocedural
  (self-recursion, index as a parameter). We adapt its inequality-graph idea into
  the interprocedural fixpoint instead of reconstructing a CFG.
- **Encode the invariant in the iterator types** (`ArrayIterator` carries
  `idx <= size`). Not expressible in the type system. Rejected.

## 10. Safety and verification

A wrongly eliminated check is memory unsafety (out-of-bounds read or write), so
verification is adversarial:

- memcheck tests over: fresh and shared arrays; off-by-one bounds; a loop bound
  reversed or unrelated to the array; an array that shrinks inside the loop; a
  helper called with both in-range and unknown indices. Assert (via the RC IR
  pre/post dump, as `test_provenance` does) that the check op is elided exactly
  where it is safe and retained everywhere else.
- speedtest cases: `arrayrw` over the array's own size, and a sum over
  `arr.to_iter`; compare cachegrind Ir against the pre-BCE baseline and inspect
  the emitted LLVM IR / asm for vector ops on the read loop.
- full `cargo test --release` at all opt levels, plus minilib and project_euler
  memcheck, as in the P1/P3 verifications.

Byte-identical output is not a criterion here (BCE changes emitted code by
design). Correctness = identical program outputs, no new memcheck errors, and
measured speedups.

## 11. Open questions and dependencies

- **Loop formation at O3.** Fix emits tail calls (`build_tail` returns the
  callee's result); the recursion becomes a loop only inside LLVM O3. Confirm O3
  reliably loop-ifies these self-recursions so vectorization actually fires once
  the check is gone; if it does not, Fix-level loop formation (the recursion-TCO
  line of work) would compound BCE's value.
- **Widening.** Pick a simple fixed-threshold widening for the range lattice and
  confirm termination on nested loops.
- **Root tracing across loop-state threading.** Verify the extended
  `borrow.rs::root` follows the index-in-tuple and array-in-tuple projections and
  reconstructions that the loop state introduces.
