# Phase 1 grounding: the union to remove in `sum_by_range_fold`

The concrete lowered RC IR (pre-`insert_rc`, `--emit-rc-ir all`) that Phase 1
(case-of-case + case-of-known-constructor) must collapse. This is the specialized
`range.fold` driver; `arr` is a borrowed capture (`#v1`), the accumulator (`#v0`) is a
scalar parameter, and the iterator (`#v2`) is a `RangeIterator {next, end}`.

## Before (the `Option` is built by `advance`, then matched by `fold`)

```
fn fold_specialized(#v0 : I64 [acc], #v1 : CapList<arr>, #v2 : RangeIterator) -> I64:
    destructure #v2 { .0 -> next, .1 -> end }
    let cond = int_eq(end, next)
    let opt : Option (RangeIterator, I64) = match cond {        // ADVANCE builds the Option
        case 1:  let u = make_struct(); let n = union_0(u); ret n          // none  (tag 0)
        case 0:  let next' = next+1;
                 let iter' = make_struct(next', end);                       // RangeIterator
                 let s = make_struct(iter', next);
                 let sm = union_1(s); ret sm                                // some  (tag 1)
    }
    let res : I64 = match opt {                                 // FOLD matches the Option
        case 0(p0 : ()):                     release #v1; destructure p0 {}; ret #v0
        case 1(p1 : (RangeIterator, I64)):   destructure p1 { .0 -> iter, .1 -> a };
                                             retain #v1;
                                             let cb  = decap_lam1(#v1, a, #v0);   // acc + arr[a]
                                             let rec = fold_specialized(cb, #v1, iter);
                                             ret rec
    }
    ret res
```

Note the RC nodes (`release`/`retain #v1`) are shown because this dump is post-`insert_rc`;
the simplifier runs **before** `insert_rc`, so it sees the same shape without them.

## The two rewrites

`res` is bound by `let res = match(opt, armsFold)`, and its scrutinee `opt` is itself bound by
`let opt = match(cond, armsAdvance)`. This is a **match whose scrutinee is a let-bound match**.

1. **case-of-case** floats the outer `match res` into each arm of the inner `match opt`. In ANF
   the inner arm ends `... ret n` (its result becomes `opt`); replace that tail with the outer
   match performed on the arm's result value. Concretely, for each inner arm, drop its `ret rv`
   and splice `let res = match(rv, armsFold); <continuation after res>` in its place. `rv` in each
   arm is a freshly built union (`union_0` / `union_1`).

2. **case-of-known-constructor** then fires in each spliced arm, because the scrutinee is now a
   `let rv = union_k(payload)` immediately matched: replace `match(rv, armsFold)` with `armsFold`'s
   `case k` body, binding that arm's payload variable to `payload`. The union construction and the
   match both vanish. The dual on structs: `destructure x {..}` where `x = make_struct(f0, f1, ..)`
   binds each field variable directly to `fi`.

## After (Option gone; loop-carried state is the plain `RangeIterator {next,end}`)

```
fn fold_specialized(acc, cap, iter):
    destructure iter { next, end }
    let cond = int_eq(end, next)
    let res = match cond {
        case 1:  release cap; ret acc                              // (empty-struct plumbing DCE'd)
        case 0:  let next' = next+1;
                 let iter' = make_struct(next', end);              // RangeIterator{next+1, end}
                 retain cap;
                 let cb  = decap_lam1(cap, next, acc);
                 let rec = fold_specialized(cb, cap, iter');
                 ret rec
    }
    ret res
```

The `Option` union is removed. The loop-carried state threaded through the recursion is now the
plain two-scalar `RangeIterator {next, end}` struct (`iter'`), with `arr` a borrowed capture and
the accumulator a scalar parameter — no boxed pointer in the state. Per the pre-experiment
(`simplifier-design.md` §8) this is exactly the shape LLVM's own SROA scalarizes into a scalar
induction variable, letting SCEV fold `_check_range(a, get_size(arr))` and vectorize. So Phase 1
alone should land `sum_by_range_fold`; no Fix-side SROA is needed for the all-scalar state.

## Implementation notes (ANF splicing)

- `Match` is an RHS, so a match is `Let(m, Match(scrut, arms), k)`. "Match of a known constructor"
  is: `scrut` is bound by `Let(scrut, Llvm(MakeUnion tag, [payload]), _)` (detect via
  `gen.as_any().downcast_ref::<InlineLLVMMakeUnionBody>()`, tag = `variant_index()`), used once.
- "Match of a match" is: `scrut` is bound by `Let(scrut, Match(inner_scrut, inner_arms), _)`, used
  once. case-of-case splices the outer `Let(m, Match(scrut,arms), k)` into each inner arm's tail,
  substituting the arm's `Ret` value for `scrut`.
- The single-use guard (RC safety) is mandatory: fire only when the constructed value / inner-match
  result is consumed exactly once, so no boxed payload gains a second reference. Loop state is
  always linear, so it fires.
- Substitution renames only variables (payloads to the constructor's operands, an arm result to the
  outer match binding), which is `let_elimination`'s safe case 1 — no computation is moved, so no
  boxed value's lifetime is extended (see `simplifier-design.md` §10b). Reuse the private renaming
  in `rename.rs` (may need to expose a variable-substitution helper).
```
