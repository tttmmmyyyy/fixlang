---
name: code-review
description: "Run review aspects sequentially against a chosen scope of code via subagents. Each subagent applies one aspect's conventions (fix-test-main-reference, code-quality, shorten-qualifiers, comment-style), all defined in this same file. Use when: reviewing code just written by AI (uncommitted changes), or doing a pre-merge review of an entire branch."
argument-hint: "Scope: 'uncommitted' for staged+unstaged changes, 'last N' for the last N commits, 'branch' for everything since the branch forked from main, or any git ref. If omitted, the skill asks."
---

# Code Review

Run a fixed sequence of review aspects against a chosen scope, **one after another** in subagents. The orchestrator section below resolves scope and dispatches subagents. The four aspects (`## Aspect: ...` sections, further down) define the conventions each subagent applies — they are not separate skills, they are sections of this file that subagents read directly.

## Scope

This orchestrator owns scope selection. It resolves the argument into a single **base ref** and passes that base to each subagent. Subagents run their own `git diff <base>` to find changes — they do **not** decide scope themselves.

| Argument            | Base ref                                  | Use when                                                         |
| ------------------- | ----------------------------------------- | ---------------------------------------------------------------- |
| (none)              | *ask the user* — see Procedure step 1     | The invoker did not specify a scope                              |
| `uncommitted`       | `HEAD`                                    | Reviewing code just written / staged but not yet committed       |
| `last N` (e.g. `last 3`) | `HEAD~N`                             | Reviewing the last N commits on the current branch               |
| `branch`            | `$(git merge-base HEAD main)`             | Pre-merge review of the whole branch since it forked from `main` |
| `<git ref>`         | `<git ref>`                               | Arbitrary base (a commit hash, another branch, etc.)             |

## Aspect Sequence

Run these aspects in this order, each in its own subagent:

1. **fix-test-main-reference** — for changed Fix-source compile tests, ensure every top-level declaration introduced by the test is referenced from `main` (directly, transitively, or via `eval`); otherwise the Fix compiler can silently skip a broken definition.
2. **code-quality** — apply general programming-maxim review (DRY, single responsibility, dead-code removal, defensive-code trimming, shotgun-surgery annotation, root-cause vs symptom check, etc.).
3. **shorten-qualifiers** — replace verbose `crate::module::Type` paths with imports (also covers any new imports introduced by step 2).
4. **comment-style** — apply project comment conventions to whatever survived steps 2–3.

## Why Sequential, Not Parallel

1. **Avoid conflicting edits.** Aspects modify files. Parallel runs would fight each other.
2. **Each aspect should see prior changes.** E.g., `shorten-qualifiers` should see imports added by `code-quality`; `comment-style` shouldn't waste effort polishing comments that `code-quality` just deleted.

## Procedure

1. **Resolve the base ref** from the argument:
   - empty → ask the user which scope they want using `AskUserQuestion`. Offer four options:
     1. **Uncommitted** — staged + unstaged changes (`HEAD`).
     2. **Last N commits** — review the last N commits on the current branch (resolve to `HEAD~N`). When the user picks this, ask a follow-up question for `N`.
     3. **Branch (vs main)** — everything since this branch forked from `main` (`$(git merge-base HEAD main)`).
     4. **Custom git ref** — any commit hash / branch / tag the user names. Ask for the ref as a follow-up.

     After the user picks, resolve to the corresponding base ref using the rules below.
   - `uncommitted` → `HEAD`.
   - `last N` (where N is a positive integer) → `HEAD~N`. Verify it resolves with `git rev-parse --verify HEAD~N`.
   - `branch` → `$(git merge-base HEAD main)`. Verify `main` exists; if the project uses a different default branch, abort and ask.
   - anything else → treat as a git ref. Verify it resolves with `git rev-parse --verify <ref>`.
2. **Run the chain.** For each aspect (in the listed order):
   a. Extract the aspect's section from this file: take everything from the line `## Aspect: <aspect-name>` up to (but not including) the next `## Aspect:` heading, or end of file. Drop nothing — include all sub-sections (`### ...`).
   b. Launch one subagent via `Agent` (subagent_type: `general-purpose`).
   c. Brief it with the prompt template below, substituting the aspect name, the base ref, and the extracted aspect text inline. Do **not** ask the subagent to open `SKILL.md` — the section text is the instruction.
   d. **Wait for it to complete** before launching the next. Never use `run_in_background`.
3. **Summarize.** Per aspect, list which files were touched and a one-line description of each change.
4. **Stop on failure.** If any subagent reports an error (aspect couldn't run, build broke, etc.), stop the chain and surface the failure. Do not continue.

## Subagent Prompt Template

```
You are running one aspect of a code review.

Aspect: <aspect-name>
Base ref: <base>

The base ref is the comparison point: review the diff between <base>
and the current working tree, i.e. run your own `git diff <base>` to
find the files and hunks to operate on.

The full instructions for this aspect follow between the BEGIN/END
markers. Treat them as your sole instructions. Do not look elsewhere
for additional conventions.

----- BEGIN ASPECT INSTRUCTIONS -----
<paste the full text of `## Aspect: <aspect-name>` here, including all
its `### ...` sub-sections, up to but not including the next
`## Aspect:` heading or the end of file>
----- END ASPECT INSTRUCTIONS -----

Apply any edits the aspect prescribes. If the aspect modifies code, run
`cargo check` afterwards to confirm the project still builds.

Report back in under 100 words: which files you touched and a one-line
summary of the change in each.
```

## What NOT to do

- Don't run aspects in parallel.
- Don't let subagents decide their own scope — always pass the resolved base.
- Don't continue the chain if a step fails.

---

# Review Aspects

The four sections below are referenced by name from the orchestrator. Each is self-contained: a subagent should be able to apply an aspect by reading only its section.

## Aspect: fix-test-main-reference

Rust unit tests under `src/tests/test_*.rs` often embed Fix source as a string literal, then compile and run it. If the test only declares the value or trait member it is meant to verify, but never uses it from `main`, the Fix compiler may skip that symbol's elaboration — a broken definition can compile cleanly and the test passes despite the breakage.

This aspect scans the Fix sources in those tests and flags top-level declarations that are not reachable from `main`.

### Out of Scope

- Fix sample projects under `tests/` and `src/tests/test_*/cases/` (these are real `fix build` runs against `.fix` files; reachability there is the project's own concern, not this aspect's).
- Tests whose intent is to verify a *compile error* on the declaration itself — for those, the unreferenced symbol *is* the test.
- Rust files outside `src/tests/test_*.rs`.

If a test fits the second category, leave it as is and note it in the report.

### What Counts as a Top-Level Declaration

In a Fix source string, the things this aspect cares about:

- **Global values**: a top-level `name : Type;` paired with `name = expr;`, or a single `name = expr;` with inferred type.
- **Trait members**: each method declared in `trait a : TraitName { ... }`.
- **Trait impl members**: each method defined in `impl Type : TraitName { ... }`. An impl method by itself does **not** count as "referencing" the trait member — both the trait member and at least one *use* must be reachable from `main`.

Out of scope:
- Local bindings (`let x = ...` inside a function body).
- `main` itself.
- Type and trait declarations (these are reachable iff their inhabitants are; checking the inhabitants suffices).

### Reference Check

A declaration is "referenced from `main`" if its name appears as a token somewhere reachable from `main`:

1. Inside the body of `main`.
2. Inside the body of any other top-level value that is itself referenced from `main` (transitive).
3. Inside an `eval` expression on a path reachable from `main` — `eval some_value;` counts as a reference to `some_value`.

A practical heuristic that works well: a declared name is "referenced" if it appears as a **whole token** (word-boundary match) anywhere in the same Fix source string, **outside its own declaration lines**. This is conservative — it accepts more than strict transitive reachability — but a true compiler-skipped symbol almost never appears as a token outside its definition. False positives are rare; if a name happens to be a substring of an unrelated identifier, the word-boundary match will not collide.

For trait members, "outside its own declaration lines" means: outside both the `trait { ... }` block and any `impl { ... }` block that defines the same member. The member name must appear in a *call* position elsewhere.

### Procedure

1. Run `git diff <base> -- 'src/tests/test_*.rs'` to find changed Rust test files.
2. For each changed file, locate every Fix-source string literal that the diff added or modified:
   - Raw strings: `r#"..."#`, `r##"..."##`, etc.
   - Regular strings: `"..."` whose content looks like Fix source (e.g., starts with `module `, contains `main : IO ()`).
3. For each such Fix source string:
   a. Identify top-level declarations introduced or modified by the diff.
   b. For each, check whether the symbol's name has a token-level reference outside its declaration lines, in the same Fix source.
   c. Collect declarations with no such reference.
4. For each unreferenced declaration:
   - **Apply** when the fix is mechanical and unambiguous — for example, when `main` has a trivial body and adding `eval <name>;` (or `let _ = <name>;` for a value the test wants forced through type-checking) clearly preserves the test's intent.
   - **Flag** otherwise, with: file path, the Rust test name (the `#[test]` function), the unreferenced symbol(s), and a one-line suggestion (e.g., "add `eval my_value;` to `main`", or "call `myMethod` on a concrete instance from `main`").
5. Run `cargo check` after any edits.
6. Report:
   - **Applied edits**: file, test name, what was added.
   - **Flagged for review**: file, test name, symbol(s), and why no auto-fix was attempted.

### Edge Cases

- **No `main` in the Fix source.** Some tests compile a snippet that is not a complete program. Skip — the check assumes a `main` entry point.
- **Multiple Fix sources in one Rust test.** Each string is independent; check each on its own.
- **Symbol shadowing by a local binding.** If a top-level name is shadowed by a `let` of the same name elsewhere, the heuristic may count the shadowed use as a reference. Accept this false-negative direction — being lenient here is fine; the goal is to catch the *clearly* unreferenced case.
- **Fix source built up from format strings or concatenation.** If the source is not a single literal, fall back to flagging the test for manual review rather than guessing.

### Scope Discipline

- **Touch only test files in the diff.** Pre-existing tests with the same problem are out of scope unless the diff modifies them.
- **Do not change what the test verifies.** If the obvious fix would alter the property under test (e.g., switching from "does this declaration type-check" to "does this expression evaluate"), flag rather than apply.
- **One unreferenced symbol at a time.** Multiple unreferenced declarations in one hunk may each have a different right answer.

---

## Aspect: code-quality

Walk the Rust code changed against the base ref, check it against the nine conventions below, apply the safe fixes inline, and surface the riskier ones for the user to decide.

### Out of Scope

- Comments → handled by the `comment-style` aspect.
- Import paths and wildcard imports → handled by the `shorten-qualifiers` aspect.
- Formatting → `cargo fmt`.
- Major redesigns / architectural changes → flag for the user, do not perform.

### Conventions

#### 1. DRY: don't repeat logic; reuse existing utilities

If the diff introduces logic that already exists elsewhere — in `crate::misc`, in a sibling module, or as a method on the type being passed in — replace the duplicate with a call to the existing helper.

Project specifics:
- Use `Set` / `Map` from `crate::misc`, not `std::collections::HashSet` / `HashMap`.
- Before writing a new helper, grep `src/` (especially `src/misc.rs` and the module the new code lives in) for similar patterns.

**Apply** when an existing helper is a near-identical match.
**Report only** when an existing version diverges enough that merging would change behavior.

#### 2. Extract a function on the second copy (no speculation)

This convention is specifically about **function extraction** (関数化), not about introducing traits, generics, or other heavier abstractions.

When the diff contains two or more blocks of code with the same intent — same shape, same purpose, differing only in variable names or values — extract a shared function. The threshold is **two, not three**: don't wait for a third copy to appear, because two copies will already start drifting apart.

The two users must exist in the code *today*. Don't introduce a function for a single caller "in case it's useful later" — that's speculation, not extraction.

**Apply**:
- Two near-identical blocks with the same intent → extract a function, replace both call sites.
- A function or method introduced by the diff with only one caller in the entire codebase (`grep` to confirm) → inline it back.

For **traits, generics, or other forms of abstraction** introduced with only one impl / one instantiation today: the same "concrete user must already exist" test applies, but these introduce richer machinery — flag them for the user to decide rather than auto-inlining.

#### 3. Single responsibility per function

A function should do one thing at one level of abstraction. Watch for functions that mix:
- I/O and pure computation.
- Parsing and validation.
- Building data and rendering it.

**Apply** when the split is mechanical (a long function with a clear seam, no shared mutable state across the seam).
**Report only** when the split would require redesigning callers.

#### 4. Don't add defensive code inside the trust boundary

CLAUDE.md is explicit: validate at boundaries (user input, external APIs, file I/O), trust internal calls. Reject:
- `unwrap_or_default()` / `if let Some` guards on values the caller statically guarantees.
- `Result` propagation on paths that cannot fail.
- Re-validation of arguments already checked upstream.

**Apply**: remove the defensive branch.
**Keep** the guard only when it documents a real precondition the type system can't express — and in that case leave a one-line comment naming the invariant.

#### 5. Remove dead and half-finished code

Delete:
- Functions / structs / enums introduced by the diff with no callers anywhere (`grep` to confirm).
- Commented-out code.
- `_unused` parameters left as a "will use later" placeholder.
- `todo!()` / `unimplemented!()` branches the diff doesn't actually exercise.

After deletion, run `cargo check` — the build will surface anything you misjudged as dead.

#### 6. Avoid obvious quadratic / repeated-allocation patterns

Catch *only* clear, unbounded cases — not micro-optimizations:
- `Vec::contains` inside a loop over the same `Vec` → use a `Set`.
- `format!("{}{}", a, b)` inside a tight loop → `push_str` / `write!` against one buffer.
- Repeated `clone()` of large data when a borrow would do.

**Apply** when the inputs are clearly unbounded (collections of unknown size, iterators over user data).
**Skip** when the upper bound is small and fixed (e.g., iterating over a 4-variant enum).

#### 7. Narrow the scope of mutable state

Every `let mut` and every public field expands the surface a reader must hold in their head. Prefer:
- Pushing the mutation into a `let x = { let mut tmp = ...; ...; tmp };` block so the binding is immutable outside.
- Returning a built value rather than taking an `&mut` out-parameter.
- Marking fields private when the diff doesn't need them public.

**Apply** for trivial scope tightening.
**Report only** when narrowing scope would touch multiple call sites.

#### 8. Avoid shotgun-surgery coupling; annotate it when unavoidable

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

#### 9. Struct fields must be non-redundant and on-role

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

#### 10. Fix root causes, not symptoms

When the diff fixes a bug, a test failure, or a panic, check whether the change addresses the underlying cause or just suppresses the visible failure. Ad-hoc symptom patches tend to leave the broken invariant in place — the same root issue resurfaces elsewhere later, often in a harder-to-diagnose form.

Smells to look for:

- A `try` / `catch`, `if let Ok`, `unwrap_or`, or error swallow added around the line that was failing, without an explanation of *why* the error is benign here.
- A special-case branch for the exact input that was failing (`if name == "foo" { return ... }`), with no statement of what makes that input categorically different.
- A magic constant nudged until tests pass (timeout raised, threshold lowered, retry count increased) without a derivation of why the new value is correct.
- A guard that prevents the bad state from being *observed* but leaves the bad state itself constructible (e.g., a getter that returns a default when an invariant is violated, instead of preventing the violation upstream).
- An "early return on weird input" added at the top of a function whose real bug is that it built the weird input two frames up the stack.
- Data being post-processed to repair what an earlier step produced wrong (e.g., trimming a stray character that the parser shouldn't have emitted).
- Comments that admit the workaround nature of the change: `// HACK`, `// workaround for X`, `// TODO: figure out why this happens`, `// not sure why but this fixes it`.

The litmus test: can the diff author state, in one sentence, what invariant was being violated and where? If the answer is "the test was failing and this makes it pass," the fix is treating a symptom.

**Apply** when the root cause is visible inside the diff's surrounding code and the patch can be moved to the right place mechanically — e.g., the workaround sits at the call site but the bug is plainly in the callee a few lines above; or a swallowed error has an obvious upstream check that should have prevented it.

**Flag** when fixing the root cause would require changes outside the diff's scope, or design judgment the reviewer doesn't have. In the report, name the symptom, name the suspected root cause, and say why the current patch only treats the symptom — don't silently accept it.

**Exception**: a workaround for a confirmed external bug (upstream library, OS, hardware) is legitimate. Require a comment that names the external issue (link, version, ticket) and the condition for removing the workaround.

### Procedure

1. Run `git diff <base>` to find changed files and hunks.
2. For each changed `.rs` file, walk the diff hunks and check each of the ten conventions against the added/modified code.
3. For each violation:
   - Identify which convention (1–10).
   - If it falls in "Apply": make the edit with `Edit`.
   - If it falls in "Report only": collect it for the final report; do not edit.
4. Run `cargo check` after edits. On failure, revert the offending edit and reclassify it as a flagged item.
5. Report:
   - **Applied edits**: file, convention number, one-line summary per change.
   - **Flagged for review**: file, convention number, what you saw and why you didn't auto-fix.

### Scope Discipline

- **Touch only code inside diff hunks.** Pre-existing violations in untouched parts of the file are out of scope.
- **Do not redesign.** If the right fix is "extract a new module" or "rewrite this pipeline," report it; don't do it.
- **One convention at a time per hunk.** If a hunk hits multiple conventions, apply the smallest fix that satisfies one, then re-check before moving on.

---

## Aspect: shorten-qualifiers

Three related cleanups for Rust import style:

1. **Shorten qualified paths** — replace `crate::foo::bar::Baz` with `Baz` plus a `use` import.
2. **Eliminate wildcard imports** — replace `use foo::*;` with an explicit list of the names actually used.
3. **Collapse the use block** — remove blank lines between `use` statements at the top of the file. The project convention is one contiguous block; sectioning (std vs external vs crate, as `rust-analyzer` likes to do) is not meaningful here.

The project convention is *explicit imports, no wildcards, no section breaks*. All three cleanups serve that convention.

### Procedure

1. **Collect changed files**: Run `git diff --name-only <base>` to find affected files.

2. **Identify cleanup targets**: For each affected file, search the **entire file** (not just diff lines). The diff is used only to determine *which files* to process — pre-existing violations in the same file are also fixed. Look for:
   - **Wildcard imports**: `use module::*;` (and grouped variants like `use module::{*}`).
   - **Qualified paths**: `crate::module::Ident`, `crate::module::{A, B}`, `module::submodule::Ident`. Paths used as types, function calls, trait bounds, or in expressions.
   - **Blank lines inside the top-of-file `use` block**: any empty line between two `use` statements at the start of the file. These are typically `rust-analyzer`-inserted section breaks (std / external crates / `crate` / `super`) that the project does not want.

3. **Read each file's existing imports**: For each affected file, read the `use` block at the top to know what is already imported.

4. **Plan replacements**:
   - **For each wildcard import** `use foo::*;`: list every identifier from `foo` actually referenced in the file, and rewrite the import as `use foo::{A, B, C};` (or merge into an existing line if one already imports from `foo`).
   - **For each qualified path**:
     - Determine the short name (last segment, e.g., `Baz` from `crate::foo::bar::Baz`).
     - Check if the short name conflicts with another import or a different qualified path in the same file.
       - **No conflict**: Add a `use` statement and replace all occurrences with the short name.
       - **Conflict**: Keep the minimal qualification needed to disambiguate (e.g., `bar::Baz` instead of full `crate::foo::bar::Baz`).
     - If an existing `use` already imports from the same module, extend it (e.g., `use crate::foo::{A};` → `use crate::foo::{A, B};`).
   - **For each blank line inside the top-of-file `use` block**: delete it. The block should run from the first `use` line to the last with no empty lines between. Do not reorder the imports — only remove the blanks. The single blank line that separates the entire `use` block from the code below it stays.

5. **Apply edits**: Add/update `use` statements in the import block, following the file's existing style. Replace qualified paths with short names.

6. **Verify**: Build the project (`cargo check`) to confirm no compilation errors. Wildcard removal is the most error-prone step — if some identifier was implicitly pulled in via the wildcard, the build will fail and reveal it; add it to the explicit list.

### Collision Detection

A name collision exists when two different fully-qualified paths resolve to the same short name. For example:
- `crate::ast::types::TypeNode` and `crate::parse::types::TypeNode` both shorten to `TypeNode`.

In this case, keep one as a `use` import and qualify the other minimally, or qualify both if the file uses them equally.

### Edge Cases

- **Re-exports**: If `crate::module` re-exports a type, prefer the shorter re-export path.
- **Already imported**: If the identifier is already imported, just replace the qualified usage; don't add a duplicate import.
- **Inside macro invocations**: Be cautious with paths inside macros; they may require full qualification.
- **Non-`crate` paths**: Also handle `std::`, `serde::`, etc. external crate paths if they appear qualified in code.

---

## Aspect: comment-style

Scan the doc/inline comments touched by the diff. Rewrite ones that violate conventions 1–5, and add missing doc comments per convention 6.

### Conventions

#### 1. Don't enumerate callers

A doc comment should describe *what the thing is and how it behaves*, not *who calls it*. Lists like "used by handlers X, Y, Z", "the three callers share this result", or "called from foo() and bar()" rot fast — every new caller is a missed comment edit, and stale lists are worse than none.

**Rewrite**: drop the caller list. Let callers be discoverable by `grep` / IDE.

#### 2. Don't enumerate None / error cases

For functions returning `Option<T>` or `Result<T, E>`, omit a trailing "Returns `None` when X, Y, Z" paragraph that lists routine failure cases. The return type already advertises that the call can fail; routine cases (input not found, position out of range, name not bound) aren't surprising.

**Keep** such notes only when a particular failure mode is *genuinely surprising* — a fail-fast condition the caller must guard against, an invariant the type signature can't express, or behavior diverging from what a reader would reasonably guess.

#### 3. Describe inputs/outputs, not internal helpers

A comment should be readable without prior knowledge of internal helpers or sibling functions. If the comment says "projects X's `internal_helper` into Y" or "wraps `private_thing`", rewrite it to describe the inputs and outputs directly.

#### 4. Don't narrate history / how-we-got-here

Comments are read by people coming to the code fresh. They want to know *what the code does and why it works the way it does*, not the path that led the author here. AI-written comments often slip in narratives like "originally we did X, but switched to Y", "now uses the new helper", "refactored from the old approach", "previously this returned a Map" — these belong in commit messages and PR descriptions, not in the source.

**Rewrite**: drop the historical narrative. State the current behavior on its own terms.

**Keep** a "why" reference to history *only* when the implementation departs from what a reader would naturally expect, and the past explains the departure — e.g., "we don't use approach X here because of bug #1234" or "the obvious recursion is unrolled to avoid stack overflow on deeply nested input." The test: would a fresh reader, seeing the code, be surprised by the choice and benefit from knowing why?

#### 5. Comments must be in English

This project's source comments are written in English (`Document.md`, `std.fix`, prior code, and PRs all assume English). A comment in any other language — Japanese, Chinese, etc. — is a style violation.

**Rewrite**: translate the comment into clear English while preserving its meaning.

#### 6. Every Rust item must have a doc comment

Conventions 1–5 are about *rewriting* existing comments. This one is about *adding* missing ones.

Every Rust item — `struct`, `enum`, `union`, their fields and variants, `trait`, trait method, free function, and `impl`-block method — must carry a `///` doc comment. **This applies to both `pub` and private items.**

Function comment shape:
- The first line describes what the function does. Don't restate the name (e.g., don't write `/// Adds two numbers.` for `fn add`).
- Add a `# Arguments` section *only* for arguments whose role isn't self-evident from the function's purpose — those that prompt the reader to ask "why is this argument needed?" Skip arguments whose role is obvious from name and type. Format:
  ```rust
  /// Resolves the symbol at the given position.
  ///
  /// # Arguments
  /// * `prefer_definition` — when true, jump through re-exports to the original definition. Used by goto-def; references search wants this off.
  ```
- Add a `# Returns` section only when the return value needs explanation beyond the type.

Test comment shape (for `#[test]` functions): the comment must state *what perspective the test exercises* — which behavior, edge case, or invariant it validates — not just "tests `foo`." Example: `/// Verifies that rename across an import boundary updates both the definition and the qualified callsite.`

**Excluded:**
- Fix sample programs under `src/tests/test_*/cases/` (these are `.fix` files; this aspect only walks `.rs` anyway).
- Items generated by `derive` macros or build scripts.

**Apply**: when a Rust item appears in the diff hunks (newly added, or its signature line was modified) and has no `///` directly above it, add a meaningful one-liner.
**Flag instead of writing boilerplate.** If you cannot articulate the item's purpose without restating its name, surface it as a flagged item rather than writing filler — the missing description may signal the item itself is redundant or poorly named.

### Procedure

1. Run `git diff <base>` to find changed Rust files and the touched line ranges.
2. For each changed `.rs` file, examine:
   - (a) comments that appear in the diff hunks (added or modified lines), for conventions 1–5;
   - (b) Rust items defined or whose signature was modified in the diff hunks, for convention 6.
3. For each violation:
   - Identify which convention (1–6).
   - For 1–5: propose a rewrite that preserves the intent but removes the anti-pattern, then apply with `Edit`.
   - For 6: write a meaningful one-liner above the item, or flag it for review if you cannot articulate the purpose without restating the name.
4. After all edits, run `cargo check` to confirm nothing broke (comment edits shouldn't affect builds, but verify in case of doctest changes).
5. Report:
   - **Applied edits**: file, convention, brief rationale.
   - **Flagged for review** (convention 6 only): file, item name, why no comment was written.

### Scope

- **Touch only comments and items inside the current diff hunks.** Don't comb through the entire codebase for violations — that is out of scope and would create a giant unrelated change. In particular, convention 6 does *not* require backfilling doc comments on pre-existing undocumented items in the same file.
- **Do not rewrite comments just because they're long.** Length is not the issue; the listed anti-patterns are.
- **Do not enforce conventions beyond the six above.** Other style judgments (tone, capitalization, line wrapping) are not in scope.
