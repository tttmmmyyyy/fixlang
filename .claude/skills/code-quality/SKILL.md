---
name: code-quality
description: "Review changed Rust code for general programming-maxim violations: code duplication, missed reuse of existing utilities, fuzzy responsibility, premature abstraction, dead code, defensive code at non-boundaries, obvious inefficiency, shotgun-surgery coupling, and struct fields that are redundant or off-role. Use when: reviewing AI-written code before commit, or as a stage of code-review."
argument-hint: "Base git ref to diff against; defaults to HEAD"
---

# Code Quality Review

Walk the Rust code changed against a base ref, check it against the nine conventions below, apply the safe fixes inline, and surface the riskier ones for the user to decide.

The base ref is provided by the caller (e.g., the `code-review` orchestrator). When run directly without an argument, default to `HEAD`.

## When to Use

- Reviewing AI-generated code before commit.
- Pre-merge cleanup of a branch.
- As one stage of `code-review` (between `shorten-qualifiers` and `comment-style`).

## Out of Scope

- Comments → handled by `comment-style`.
- Import paths and wildcard imports → handled by `shorten-qualifiers`.
- Formatting → `cargo fmt`.
- Major redesigns / architectural changes → flag for the user, do not perform.

## Conventions

### 1. DRY: don't repeat logic; reuse existing utilities

If the diff introduces logic that already exists elsewhere — in `crate::misc`, in a sibling module, or as a method on the type being passed in — replace the duplicate with a call to the existing helper.

Project specifics:
- Use `Set` / `Map` from `crate::misc`, not `std::collections::HashSet` / `HashMap`.
- Before writing a new helper, grep `src/` (especially `src/misc.rs` and the module the new code lives in) for similar patterns.

**Apply** when an existing helper is a near-identical match.
**Report only** when an existing version diverges enough that merging would change behavior.

### 2. Extract a function on the second copy (no speculation)

This convention is specifically about **function extraction** (関数化), not about introducing traits, generics, or other heavier abstractions.

When the diff contains two or more blocks of code with the same intent — same shape, same purpose, differing only in variable names or values — extract a shared function. The threshold is **two, not three**: don't wait for a third copy to appear, because two copies will already start drifting apart.

The two users must exist in the code *today*. Don't introduce a function for a single caller "in case it's useful later" — that's speculation, not extraction.

**Apply**:
- Two near-identical blocks with the same intent → extract a function, replace both call sites.
- A function or method introduced by the diff with only one caller in the entire codebase (`grep` to confirm) → inline it back.

For **traits, generics, or other forms of abstraction** introduced with only one impl / one instantiation today: the same "concrete user must already exist" test applies, but these introduce richer machinery — flag them for the user to decide rather than auto-inlining.

### 3. Single responsibility per function

A function should do one thing at one level of abstraction. Watch for functions that mix:
- I/O and pure computation.
- Parsing and validation.
- Building data and rendering it.

**Apply** when the split is mechanical (a long function with a clear seam, no shared mutable state across the seam).
**Report only** when the split would require redesigning callers.

### 4. Don't add defensive code inside the trust boundary

CLAUDE.md is explicit: validate at boundaries (user input, external APIs, file I/O), trust internal calls. Reject:
- `unwrap_or_default()` / `if let Some` guards on values the caller statically guarantees.
- `Result` propagation on paths that cannot fail.
- Re-validation of arguments already checked upstream.

**Apply**: remove the defensive branch.
**Keep** the guard only when it documents a real precondition the type system can't express — and in that case leave a one-line comment naming the invariant.

### 5. Remove dead and half-finished code

Delete:
- Functions / structs / enums introduced by the diff with no callers anywhere (`grep` to confirm).
- Commented-out code.
- `_unused` parameters left as a "will use later" placeholder.
- `todo!()` / `unimplemented!()` branches the diff doesn't actually exercise.

After deletion, run `cargo check` — the build will surface anything you misjudged as dead.

### 6. Avoid obvious quadratic / repeated-allocation patterns

Catch *only* clear, unbounded cases — not micro-optimizations:
- `Vec::contains` inside a loop over the same `Vec` → use a `Set`.
- `format!("{}{}", a, b)` inside a tight loop → `push_str` / `write!` against one buffer.
- Repeated `clone()` of large data when a borrow would do.

**Apply** when the inputs are clearly unbounded (collections of unknown size, iterators over user data).
**Skip** when the upper bound is small and fixed (e.g., iterating over a 4-variant enum).

### 7. Narrow the scope of mutable state

Every `let mut` and every public field expands the surface a reader must hold in their head. Prefer:
- Pushing the mutation into a `let x = { let mut tmp = ...; ...; tmp };` block so the binding is immutable outside.
- Returning a built value rather than taking an `&mut` out-parameter.
- Marking fields private when the diff doesn't need them public.

**Apply** for trivial scope tightening.
**Report only** when narrowing scope would touch multiple call sites.

### 8. Avoid shotgun-surgery coupling; annotate it when unavoidable

If a future change to one location will silently force a change at a distant location — different file, different module, different crate — that is a maintenance trap. The next editor will fix one side and ship the bug.

Examples:
- A constant duplicated in two modules.
- A struct's field order matched by a hand-written serializer elsewhere.
- An enum variant assumed by a **non-exhaustive** `match` (one with `_ =>`) or a `matches!` / `if let` listing only some variants, in another file. An *exhaustive* `match` is *not* a coupling problem — the compiler forces the consumer to handle every new variant. The trap is specifically the non-exhaustive case, where adding a variant compiles fine but silently falls through to the catch-all or the `false` branch. Example:
  ```rust
  // producer: ast/types.rs
  enum TyKind { Var, App, Fun }   // adding `Lit` here…

  // consumer: typecheck/unify.rs
  fn complexity(t: &TyKind) -> usize {
      match t {
          TyKind::Var => 1, TyKind::App => 3, TyKind::Fun => 4,
          _ => 0,                  // …silently makes Lit's complexity 0
      }
  }
  ```
- A `pub` function name referenced as a string in tests / config / generated code.

**Apply**: where possible, collapse to one source of truth — a single `const`, a `From`/`Into`, a derive macro, an exhaustive `match` that the compiler will force you to update.
**If unavoidable**: add a one-line comment at *both* sites pointing to the other, e.g. `// must stay in sync with foo::BAR` and `// must stay in sync with bar::FOO`. The pair of comments is the load-bearing artifact, not the choice of words.

### 9. Struct fields must be non-redundant and on-role

Two failure modes to catch:

**(a) Redundant fields.** A field whose value is a pure function of other fields in the same struct should not be stored. Example:

```rust
// Wrong — keys is just data.keys().
struct Index {
    keys: Set<Symbol>,
    data: Map<Symbol, Entry>,
}
```

The second copy can drift out of sync with the first, and the type system cannot enforce the invariant. Replace with a method:

```rust
struct Index {
    data: Map<Symbol, Entry>,
}
impl Index {
    fn keys(&self) -> impl Iterator<Item = &Symbol> { self.data.keys() }
}
```

**(b) Off-role fields.** Only fields that match the struct's stated role belong in it. Don't graft unrelated bookkeeping onto a struct just because it happens to be in scope at the call site (e.g., a "parser state" struct gaining a `last_error_message_for_ui: Option<String>` field). Either pass the value through as a function argument, return it from the call, or put it in a struct whose role it actually matches.

**Apply** for (a): delete the redundant field, add a method that derives it, update reads.
**Apply** for (b) only when the field has one or two readers and the right home is obvious. Otherwise flag — picking the new home may require user judgment.

**Exception** for (a): if the derivation is genuinely hot and profiling justifies caching, the cached field is allowed — but add a comment naming the invariant (e.g., `// invariant: keys == data.keys(); cached because the lookup is on the hot path`).

## Procedure

1. Run `git diff <base>` to find changed files and hunks.
2. For each changed `.rs` file, walk the diff hunks and check each of the nine conventions against the added/modified code.
3. For each violation:
   - Identify which convention (1–9).
   - If it falls in "Apply": make the edit with `Edit`.
   - If it falls in "Report only": collect it for the final report; do not edit.
4. Run `cargo check` after edits. On failure, revert the offending edit and reclassify it as a flagged item.
5. Report:
   - **Applied edits**: file, convention number, one-line summary per change.
   - **Flagged for review**: file, convention number, what you saw and why you didn't auto-fix.

## Scope Discipline

- **Touch only code inside diff hunks.** Pre-existing violations in untouched parts of the file are out of scope.
- **Do not redesign.** If the right fix is "extract a new module" or "rewrite this pipeline," report it; don't do it.
- **One convention at a time per hunk.** If a hunk hits multiple conventions, apply the smallest fix that satisfies one, then re-check before moving on.
