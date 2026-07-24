# Handoff from `unique-check-elim` to `bce` (2026-07-21)

`unique-check-elim` carries three commits past the merge base `e0943d00` that `bce` last merged
(`c252f404`). This document says what they are, how to merge them, what they mean for the
Array/Storage redesign, and what work they leave open.

## 1. The three commits

### `f4d9d30f` Read a fully unboxed field as a borrow of its container

`InlineLLVMStructGetBody` took ownership of the container: `ObjectFieldType::get_struct_fields`
retains the field out of a boxed container and releases it, and out of an unboxed one moves the field
out and releases the siblings. Reading one scalar field of a loop-state struct therefore forced a
retain of the whole container whenever it was read again, which demoted every leaf of that container
to `Unknown` and left the runtime uniqueness check on a later `array_set` of its `Array` field.

A fully unboxed field holds no reference, so the value read out of it takes nothing from the
container. The read is now declared a borrow and extracts with `move_out_struct_field`; reference-count
insertion releases the container at its last use instead. Measured on a loop-state struct with an
`Array` field: the `retain` of the array leaf disappears, provenance rises from `unknown` to `arg1.1`,
and the specialized `array_set` becomes `array_set[unique]`.

Two declarations changed shape:

- `LLVMGen::borrows_operand` now takes `(&[Arc<TypeNode>], &TypeEnv)`, since whether the read borrows
  depends on the field's type. This follows `unique_check_operand`, which took the same turn earlier.
- The invariant an op that declares a borrow must keep is written on `borrows_operand`: **read that
  operand with `get_scoped_obj_noretain`**. A plain `get_scoped_obj` retains an unboxed global's boxed
  subobjects, and a borrow has no matching release, so a plain read leaks. Every op that borrowed
  already did this; the reason was not recorded anywhere.

### `26dca513` Track the object a match binding carries

A match binding was `Binding::Producer`, so `root` answered that it denotes an object of its own. It
denotes **one of the arms' results**: an arm whose payload aliases the scrutinee — an unboxed union's
variant slot, or a catch-all arm's whole scrutinee, on a boxed union too — carries the scrutinee's
reference into the binding.

Reference-count cancellation mis-paired because of it. `test_match_option` failed as a use-after-free:

```
    retain v#106                     // the union is read again after the match
    let match#111 = match v#106 { some(v) => v, none(_) => fresh }
    let v#113 = struct_get_0(match#111)
    release match#111                // drops the reference the retain made
    let v#120 = union_as_1(v#106)
    release v#120                    // drops the union's own reference
```

`cancel` read `release match#111` as acting on an unrelated object, so it cancelled `retain v#106`
against `release v#120` and freed the value while `v#106` still had to be read.

`root` is replaced by `origin`, which answers with the object a leaf **is**, or the objects it **may
be** together with the name the join gives it:

```rust
enum Origin {
    Exactly(VarPath),
    Join { identity: VarPath, candidates: Set<VarPath> },
}
```

`Binding::Producer` keeps App results and closures; a match binding is `Binding::Join(Vec<RcVar>)`,
holding the variable each arm returns (`returned_var` walks the arm's continuation chain to its `Ret`).
Arms that all reach one object collapse to `Exactly`, which is a pairing the single-valued answer lost.

The three readers pick their projection:

| reader | projection | why |
| --- | --- | --- |
| `CancelAnalysis::unit_key` (retain push, release pairing) | `identity` | pairing a retain with the release that un-bumps it needs one name; deciding it on a may-answer would delete a real reference operation |
| `CancelAnalysis::consume`, and a `Release` on the objects other than its identity | `acted_on` = identity + candidates | the operation drops the reference the leaf holds, **and** that reference belongs to one of the candidates; a pending retain on any of them is load-bearing |
| `infer_ownership`, `owns_unit` | `candidates` | their answers have to hold on every path |

**Both halves of `acted_on` are needed.** Dropping the identity from the consume set was a second
use-after-free, caught by `test_struct_act`: `retain m; <consume m>; ...; release m` cancelled the
retain because the consume looked only at the arms' objects and never at `m` itself.

Two claims made along the way turned out to be wrong and are recorded so they are not re-derived:

- **`infer_ownership` and `owns_unit` were not broken.** `collect_consumes_go` counts *every* `Ret` as
  a consume, including an arm's, so an arm returning a payload already attributed the consume to the
  scrutinee. Their `candidates` form is the correct reading of the enum, not a fix.
- **"A match consumes its scrutinee" is false for an unboxed union.** `eval_rc_match` extracts a
  variant payload with `get_union_value_noretain_norelease` and retains only when the scrutinee is
  boxed; `insert_into_match` emits the container release only for a boxed union's *variant* arm. So an
  unboxed union's payload, and a catch-all arm's payload on either kind, are pure aliases.

### The rename commit

`Origin::Join`'s fields were first `at` / `may`; they are `identity` / `candidates`.

## 2. Merging into `bce`

There is no textual conflict — `git merge-tree --write-tree bce unique-check-elim` produces a tree
with none. `bce`'s changes to `borrow.rs` and `rc_insert.rs` are the `RcExpr::Eval` arms it added, in
different hunks from this session's.

**One thing will not compile after the merge**: `returned_var` in `borrow.rs` matches every `RcExpr`
variant and `bce` has one more. Add

```rust
RcExpr::Eval(_, k) => returned_var(k),
```

alongside the other continuation-carrying arms. `Eval` observes its variable and continues, so the
returned variable is its continuation's.

Nothing else in the merged code needs to know about `Eval`: `origin` reads `VarTable::bindings`, which
`collect_bindings` already fills through `Eval` on `bce`.

After merging, the checks worth re-running are the full suite at Max (the borrow-optimization passes
only run there) and `test_simplify` — the simplifier rewrites match arms, which is exactly what
`Binding::Join` reads.

## 3. What this means for the Array/Storage redesign

The redesign makes `Array a` an unboxed struct `{ _storage, _size, _cap }` with the storage as the
only reference-counted leaf. `f4d9d30f` is the mechanism that makes reading `@size` and `@capacity`
free: they are fully unboxed fields of an unboxed container, so the read borrows and the storage leaf
keeps its provenance. Without it, every `arr.@size` in a write loop would retain the storage and
demote it, and the loop's uniqueness checks would stand. The design's claim that size and capacity
become register reads depends on this commit being in.

`26dca513` matters wherever the redesign's Fix-source builders return a value out of a `match` — the
`push_back` builder and the `LoopState` plumbing both do. A binding that carries the array's storage
out of a match is now tracked, so cancellation will not free it early.

## 4. Open work this session leaves

### The RC IR validator (the largest item)

`dev-docs/2026-06-28-unique-check-elim/rc-ir-validator.md` plans checks (ii) use-after-consume and
(iii) reference balance. **Check (ii) would have caught this session's miscompile statically**: on the
`some` path the match binding is the union's payload, `release match#111` drops that key to zero, and
the following `union_as_1(v#106)` is a read of a dead key. That is a strong reason to implement it.

Two corrections the plan needs before it is implemented:

- The plan says "`Match`: ... その後結果 `x` を producer として +1". That is wrong for the same reason
  `Binding::Producer` was: the match binding receives the arm's reference rather than creating one.
  The arm's `Ret` transfers ownership of the returned leaf to `x`; counting a fresh `+1` on top of it
  would hide a missing reference (or invent one for a boxed union's variant arm, where the payload
  really is a new reference the arm's `Ret` already accounts for).
- The plan keys on `root`. Use `origin` — but note the validator does **not** need `candidates`. It
  walks each arm from a copy of the pre-branch state, so inside an arm the match binding's object is
  known exactly. Path-sensitivity buys it the precision `cancel` cannot have, since `cancel` merges
  the arms. Key the match binding to the arm's own value while walking that arm.

### Smaller items

- **`union_as` has the same shape as `struct_get`.** Reading a fully unboxed payload out of a union
  takes nothing from it, so it could borrow too. `InlineLLVMUnionAsBody::generate` calls
  `get_union_value`, which consumes; the change mirrors `f4d9d30f`. Not attempted here.
- **Unbox-getter stage 2.** A *reference-counted* field read out of an unboxed container still takes
  the consuming path, because as a borrow the result would alias the container's leaf and
  reference-count insertion releases a variable at its last use without following aliases. Making it a
  borrow needs `insert_rc`'s liveness to work on objects rather than names.
- **`InlineLLVMUnionModBody` can declare `{Fresh | Arg(union)}`.** Exact — a tag match builds a new
  union, a mismatch returns the argument — and expressible since `63454e4b`. Parked for want of a
  benchmark that threads a `mod_<variant>` result through an ownership chain.
- **`LeafOrigin::SharingOf(i, path)`** (sharing without aliasing) stays deferred. Revisit if the
  pipeline re-analyzes provenance after elision, or if the redesign keeps several storage primitives
  that do not force uniqueness.
- **`EndNode` carrying its source span**, to drop the `(node, pos)` pair in the LSP helpers. Unrelated
  to RC, still open.

## 5. Verification

Recorded in `verification.md` next to this file.
