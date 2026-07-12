---
name: code-review
description: "Run review aspects sequentially against a chosen scope of code via subagents. Each subagent applies one aspect's conventions (fix-test-main-reference, design-fit, test-sufficiency, code-quality, shorten-qualifiers, comment-style, no-personal-info), all defined in this same file. After the aspects, the orchestrator commits the review edits and then applies `cargo fmt` as a separate standalone commit. Use when: reviewing code just written by AI (uncommitted changes), or doing a pre-merge review of an entire branch."
argument-hint: "Scope: 'uncommitted' for staged+unstaged changes, 'last N' for the last N commits, 'branch' for everything since the branch forked from main, or any git ref. If omitted, the skill asks."
---

# Code Review

Run a fixed sequence of review aspects against a chosen scope, **one after another** in subagents. The orchestrator section below resolves scope and dispatches subagents. The seven aspects (`## Aspect: ...` sections, further down) define the conventions each subagent applies — they are not separate skills, they are sections of this file that subagents read directly. Once every aspect has finished, the orchestrator commits the accumulated review edits and then runs `cargo fmt` as a final, standalone commit so formatting churn never mixes with the substantive changes.

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
2. **design-fit** — with the implementation now visible, re-evaluate whether the chosen design is the best fit for the change's goal; flag mismatches (this aspect never redesigns).
3. **test-sufficiency** — check whether the tests cover what the implementation actually does, including cases only visible once the code exists; flag coverage gaps (this aspect never writes tests).
4. **code-quality** — apply general programming-maxim review (DRY, single responsibility, dead-code removal, defensive-code trimming, shotgun-surgery annotation, root-cause vs symptom check, etc.).
5. **shorten-qualifiers** — replace verbose `crate::module::Type` paths with imports (also covers any new imports introduced by step 4).
6. **comment-style** — apply the project comment/doc conventions (Rust comments and hand-written Markdown docs) to whatever survived steps 4–5.
7. **no-personal-info** — scan every changed file for the user's personal data (real name, personal email, phone, address, secrets) embedded in checked-in files, and flag it. This aspect never edits; a finding blocks the commit step in the Procedure.

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
3. **Format and commit.** After the aspect chain completes, isolate the formatting churn into its own commit so it never mixes with the substantive edits.
   - **First, honor a personal-info block.** If the `no-personal-info` aspect flagged any finding, do **not** commit anything. Surface those findings prominently and stop here — personal data must be removed before it can land in a commit (and be pushed). Leave the review edits uncommitted in the tree for the user to handle. Proceed to the steps below only when `no-personal-info` came back clean.
   a. **Commit the review edits first.** If `git status --porcelain` reports pending changes, stage and commit them all as one commit — `git add -A && git commit -m "Apply code review fixes"` — so the tree is clean before formatting. This sweeps *every* pending change in the tree, the aspect edits and any other uncommitted work alike. If the tree is already clean, skip this commit.
   b. **Run `cargo fmt`.**
   c. **Commit the formatting as a standalone commit.** If `git status --porcelain` now reports changes (i.e. `cargo fmt` reformatted something), commit them on their own — `git commit -am "Apply cargo fmt"`. If `cargo fmt` changed nothing, make no commit and note that the code was already formatted.
4. **Summarize.** Per aspect, list which files were touched and a one-line description of each change, and surface every finding the flag-only aspects (design-fit, test-sufficiency, no-personal-info) raised. Then report the commits created in step 3 — the review-edits commit and the `cargo fmt` commit, with their short hashes — and remind the user they can reword or split the review-edits commit, since it gathered all pending changes under one message.
5. **Stop on failure.** If any subagent reports an error (aspect couldn't run, build broke, etc.), stop the chain and surface the failure. Do not continue. If `cargo fmt` itself fails, surface that and skip the formatting commit.

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
- Don't fold formatting into the substantive commit — `cargo fmt` always gets its own commit.
- Don't commit anything when `no-personal-info` flagged a finding — stop and surface it so the user can remove the personal data first.

---

# Review Aspects

The seven sections below are referenced by name from the orchestrator. Each is self-contained: a subagent should be able to apply an aspect by reading only its section.

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

## Aspect: design-fit

Every other aspect works *within* the design the author chose — tidying the code, the imports, the comments. This one steps outside it and asks the question the author could not fully answer before writing the code: **now that the implementation exists, is this design the best fit for the goal?** Some design flaws are invisible on the whiteboard and only become legible once the code is on the page — the shape of the implementation is itself evidence about the design.

This aspect **only flags**; it never edits or redesigns. Redesign is the author's call and out of scope for the review, so the deliverable is a clear, evidence-backed report.

### First, reconstruct the goal

You cannot judge fit without the goal. Before reading for design, recover what the change is *for*:

- Read the commit messages between the base ref and `HEAD` (`git log <base>..HEAD`), plus any branch or PR context available.
- Read the diff itself — the tests it adds, the public surface it changes, the names it introduces.
- State the goal in one or two sentences. If the diff plainly serves a narrower or different goal than the commit message claims, that mismatch is itself a finding.

### What the implementation reveals about the design

Look for concrete evidence, in the code as written, that the chosen design fights the goal:

- **Complexity out of proportion to the goal.** A conceptually simple goal that needed a lot of code, many special cases, or deep nesting — a sign the decomposition is wrong.
- **Code fighting its own data structures.** Repeated conversions, adapters, or "reshape it first" steps because a type chosen upstream is awkward for what happens downstream.
- **Information threaded through many layers.** A value passed down through several functions that don't use it, or a context object growing to carry it — a misplaced responsibility boundary.
- **Abstraction that earns nothing.** A trait / generic / builder with exactly one real instantiation (over-abstraction); or, conversely, two paths the implementation reveals to be identical and that should be unified (missed unification).
- **Edge cases accreting around the core model.** A steady pile of `if special { ... }` around the main path suggests the model doesn't match the problem.
- **A more direct route via existing machinery.** The goal is reachable by leaning on a subsystem the project already has, rather than the parallel mechanism the diff built (the DRY instinct, at the design level).

### Discipline

- **Flag only, never redesign.** No edits.
- **Evidence, not taste.** Raise a finding only when the *implementation* gives concrete evidence of a mismatch. "You could also structure it as X," with no evidence the current shape hurts, is noise — skip it.
- **Name the alternative and its cost.** A finding must state: the goal, the current design, the implementation evidence, the better-fitting design you see, and — honestly — what the rework would cost. A design flag with no proposed direction is not actionable.

### Report

- **Flagged for review**: the goal as you understood it; then, per finding, the design concern, the concrete evidence in the code, and the alternative design with its rough cost.
- If the design fits the goal well, say so in one line and flag nothing. A clean result here is the common case, not a failure to find something.

---

## Aspect: test-sufficiency

Tests written alongside a design tend to cover what the author *expected* to matter. Once the implementation exists, it exposes cases the author could not have known to test up front — the branch that turned out reachable, the boundary the algorithm actually has, the invariant the code now leans on. This aspect reads the finished implementation and asks: **do the tests cover what this code actually does, or only what the author first imagined?**

It **only flags** missing or weak coverage; it does not write tests here (that is follow-up work the author scopes). It is distinct from `fix-test-main-reference`, which checks that a test's declarations are reachable from `main`; this one checks whether *enough* of the right tests exist at all.

### First, reconstruct the goal and the behavior

- Read the commit messages between the base ref and `HEAD` (`git log <base>..HEAD`) and the diff, and state what behavior the change introduces or alters.
- Enumerate that behavior's observable cases from the *implementation*: the branches, the boundary values, the error paths, the invariants the new code relies on.

### Project testing conventions

Ground the sufficiency judgment in how this project tests (per CLAUDE.md):

- **Fix grammar or standard-library changes** need tests that **compile and execute Fix code**, with the thing under test reached from `main`. (The reachability itself is `fix-test-main-reference`'s job; here, check that such a test *exists* and exercises the new behavior.)
- **`fix` command behavior changes** want **integration tests** that run the real `fix` binary against a sample project under `tests/` (the `setup_test_env()` pattern), rather than unit tests bolted onto internals.

### Coverage gaps to flag

- **A new branch or case nothing reaches.** The implementation added a path; no test exercises it.
- **A boundary the implementation clearly has, left untested** — empty input, an off-by-one edge, the first/last element, the recursion base case, overflow.
- **An error / failure path with only the success path tested.**
- **An invariant the new code depends on, asserted nowhere.**
- **A bug fix with no regression test** — the fix could silently revert and nothing would catch it.
- **A behavior change whose only "test" is that the existing tests still pass** — nothing pins the *new* behavior.

### Discipline

- **Flag only; write no tests here.** No edits.
- **Tie each gap to a concrete case.** Name the specific input / branch / boundary left uncovered and where in the diff it lives — not "add more tests."
- **Weigh against what's already there.** Read the tests the diff adds or touches before flagging; don't flag a case an existing test already covers.

### Report

- **Flagged for review**: per gap, the untested case (a concrete input or branch), where the behavior lives in the diff, and the kind of test the project convention calls for (a Fix compile-and-run test, a `fix` integration test, or a Rust unit test).
- If coverage is adequate, say so in one line.

---

## Aspect: code-quality

Walk the Rust code changed against the base ref, check it against the thirteen conventions below, apply the safe fixes inline, and surface the riskier ones for the user to decide.

### Out of Scope

- Comments → handled by the `comment-style` aspect.
- Import paths and wildcard imports → handled by the `shorten-qualifiers` aspect.
- Formatting → the orchestrator runs `cargo fmt` in a final step; don't reformat here.
- Major redesigns / architectural changes → flag for the user, do not perform.

### Conventions

#### 1. DRY: don't repeat logic; reuse existing utilities

If the diff introduces logic that already exists elsewhere in the project, replace the duplicate with a call to the existing helper.

This project is a compiler bundled with a sizable toolchain — LSP server, package manager, documentation generator, build runner. That breadth alone makes it overwhelmingly likely that "generic" supporting logic — file/path handling, name and span arithmetic, source-text scanning, dependency graph traversal, version comparison, manifest parsing, AST/type walks, identifier formatting, and so on — is already implemented somewhere in `src/`. Before writing such logic, **reason from the project's feature set first**: ask "is this the kind of helper a compiler / LSP / package manager / doc generator would plausibly already have?" When the answer is yes (it usually is), search the entire `src/` tree to confirm before adding a new one. Do not confine the search to the new code's neighborhood — helpers cross subsystem boundaries freely, and a routine written for one feature is often exactly what another wants.

If your reasoning suggested the helper should exist but the search comes up empty, **add it where it ought to live, not where you happen to need it.** For example, if the diff inlines `s.start <= pos && pos <= s.end` for a `Span`, and you were expecting `Span` to expose an `includes` (or similarly named) method, add the method on `Span` rather than leaving the arithmetic at the call site. The very reasoning that made you go looking — "this is the kind of thing `Span` would expose" — applies just as strongly to *placing* the new helper as to *finding* an existing one. Inlining at the use site forfeits the benefit and guarantees the next caller will re-derive the same expression.

**Apply** when an existing helper is a near-identical match, or when no helper exists but its proper home is unambiguous (a method on the obvious receiver type, a constructor on the obvious factory, etc.).
**Report only** when an existing version diverges enough that merging would change behavior, or when no helper exists and the right home is a judgment call.

#### 2. Extract a function on the second copy

This convention is specifically about **function extraction** (関数化), not about introducing traits, generics, or other heavier abstractions.

When the diff contains two or more blocks of code with the same intent — same shape, same purpose, differing only in variable names or values — extract a shared function. The threshold is **two, not three**: don't wait for a third copy to appear, because two copies will already start drifting apart.

**Apply**: Two near-identical blocks with the same intent → extract a function, replace both call sites.

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

#### 11. Use the project's canonical types over their standard-library counterparts

Where the project provides its own version of a common type, use it instead of the standard-library one. These are mechanical substitutions, not stylistic preferences — they exist so that the rest of the codebase can rely on a single consistent type.

- Use `Set` / `Map` from `crate::misc`, not `std::collections::HashSet` / `std::collections::HashMap`.

**Apply** unconditionally.

#### 12. Extract named steps for readability

Conventions 1 and 2 catch *duplication* (same code in two places). This one catches *cohesion*: a block of code that is conceptually one named step embedded inside a larger function, even though it only appears once. Extracting it into a helper lets the caller read as a sequence of named steps and keeps a single abstraction level per function.

The risk this rule has to actively avoid is **function fragmentation** — chopping every 3-line block into its own helper so the reader has to chase a chain of one-line functions to follow the flow. The mechanical gates below exist to ban that outcome.

**Apply** when *all* of the following hold:

- The block is at least **5 statements** long, and is self-contained — no shared mutable state with the surrounding code aside from clearly enumerable inputs/outputs (no taking arbitrary local borrows out of a partly-built data structure mid-construction).
- You can name what the block *produces* in 2–4 words, describing the WHAT (e.g. `parse_constraints`, `inject_abs_path_implicit_imports`), not the WHERE or HOW (`step_one`, `do_the_loop`, `helper_for_main`).
- The block has **≤3 inputs** (parameters / closed-over values) and **≤1 conceptual output** (return value, or mutation of one named receiver). A long parameter list signals the block isn't actually cohesive.
- The block doesn't have to interleave with surrounding code via `?` propagation in a way that the extracted version can't preserve. (A pure-Rust extraction has to keep error behavior identical.)

**Skip** when:

- The block is short enough that naming it is more overhead than reading it.
- The only name you can come up with restates the function name plus a number (`process_step_1`, `helper_part_2`, `do_remaining_work`). The missing intuitive name usually means the surrounding function's seam is in the wrong place — flag for redesign rather than carve out filler.
- The block reads naturally as part of its surrounding flow (e.g., the loop body of a top-level `for` whose entire purpose is that loop).
- Extraction would force passing 4+ parameters or returning a tuple of more than 2 unrelated values.

**Flag instead of applying** when extraction looks beneficial but you can't satisfy all the Apply conditions — especially when a good name doesn't come to mind. Surface the location and your proposed name in the report; let the author decide whether the seam belongs there.

#### 13. No ad-hoc / hacky mechanisms

Convention 10 catches the *symptom-patch* flavor of ad-hoc code, but only in a diff that fixes a bug. This one is broader and un-gated: it asks of **any** changed code — feature work included — whether the mechanism it uses is sound, or whether it *works by coincidence* and will break the moment an unstated assumption shifts.

Smells to look for:

- **Bypassing a safe API with an unsafe or lower-level trick** where the safe path exists — e.g. a `transmute` or raw pointer cast to reinterpret a value that a proper conversion, enum, or trait method already handles.
- **Parsing a tool's human-readable output instead of using its API** — scraping `--emit` text, log lines, or `Debug` output to recover data the producer can hand over structurally (an accessor, a typed return, a callback).
- **Depending on a coincidence or on another component's bug** — relying on an incidental iteration order, a hash-map layout, a whitespace quirk, a filename sort, or a known-wrong behavior of a library/tool that *happens to* give the right answer today.
- **Manipulating structured data as a string** — regex / `replace` / `split` surgery on source text, JSON, or an identifier where the structured representation (an AST node, a parsed value, a builder) is available.
- **A hardcoded value or branch encoding an assumption the code never checks** — a fixed array size, a magic offset, or a `match` arm that silently assumes the only cases seen so far.

The litmus test: *would this still be correct if the thing it silently assumes changed — a reordering, a reformat, a new variant, an upstream fix?* If the honest answer is "no, but that won't happen," the mechanism is a hack.

**Report only — never auto-fix.** Replacing a hack with the sound mechanism is a design change that usually needs judgment the reviewer lacks, and the user wants to weigh in on hacky mechanisms directly. In the report, name the hack, name the sound alternative you'd expect (the safe API, the structural accessor, the invariant to enforce), and state the assumption it rides on.

**Exception**: a deliberate workaround for a confirmed external limitation (an upstream bug, a missing API, a platform constraint) is legitimate when a comment names the limitation and the condition for removing the workaround.

### Procedure

1. Run `git diff <base>` to find changed files and hunks.
2. For each changed `.rs` file, walk the diff hunks and check each of the thirteen conventions against the added/modified code.
3. For each violation:
   - Identify which convention (1–13).
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

Scan the doc/inline comments touched by the diff — in Rust source, and the prose in hand-written Markdown docs. Rewrite ones that violate conventions 1–5 and 7, and add missing doc comments per convention 6. Conventions 1–3 and 6 apply to Rust only; conventions 4, 5, and 7 apply to Markdown prose as well.

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

Comments are read by people coming to the code fresh. They want to know *what the code does and why it works the way it does* — not the path that led the author here, and not a justification for the choice against alternatives the reader never saw. AI-written comments often slip in narratives like "originally we did X, but switched to Y", "now uses the new helper", "refactored from the old approach", "previously this returned a Map", or defensive asides excusing why this was done rather than that — these belong in commit messages and PR descriptions, not in the source.

**The litmus test** — apply it to every touched comment: *would this sentence be written if the prior conversation, the deliberation, and the previous version of the code did not exist?* A sentence that only makes sense as a reaction to that history fails; cut it.

**Rewrite**: drop the historical narrative. State the current behavior on its own terms.

**Keep** a "why" reference to history *only* when the implementation departs from what a reader would naturally expect, and the past explains the departure — e.g., "we don't use approach X here because of bug #1234" or "the obvious recursion is unrolled to avoid stack overflow on deeply nested input." The test: would a fresh reader, seeing the code, be surprised by the choice and benefit from knowing why?

#### 5. Comments must be in English

This project's source comments are written in English (`Document.md`, `std.fix`, prior code, and PRs all assume English). A comment in any other language — Japanese, Chinese, etc. — is a style violation.

**Rewrite**: translate the comment into clear English while preserving its meaning.

#### 6. Every Rust item must have a doc comment

Conventions 1–5 and 7 are about *rewriting* existing comments (and prose). This one is about *adding* missing ones.

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
- Pre-existing undocumented items in the same file — the convention covers only items the diff introduces or whose signature it modifies.
- Fix sample programs under `src/tests/test_*/cases/` (these are `.fix` files; this aspect only walks `.rs` anyway).
- Items generated by `derive` macros or build scripts.

**Apply**: when a Rust item appears in the diff hunks (newly added, or its signature line was modified) and has no `///` directly above it, add a meaningful one-liner.
**Flag instead of writing boilerplate.** If you cannot articulate the item's purpose without restating its name, surface it as a flagged item rather than writing filler — the missing description may signal the item itself is redundant or poorly named.

#### 7. Write in the affirmative

State what *is*, not what *isn't*. Two anti-patterns to catch, both applying to Rust comments and Markdown prose alike:

**(a) Negating a rejected alternative in a definition or explanation.** Forms like "not A but B", "B rather than A", "A is wrong; B" name a rejected option, an alternative term, or a passing misunderstanding (A) only to knock it down. A reader who never had A in mind gains nothing from "it's not A" — it just makes them wonder what A was. Describe B directly.
- *Before*: `/// Not a deep copy — shares the backing buffer.`
- *After*: `/// Shares the backing buffer.`

**(b) Negating an unnecessary action in a procedure or guide.** Forms like "you don't need to do X" describe a non-action. State only what the reader must do.
- *Before*: `The caller does not need to lock the mutex first.`
- *After*: drop it, or if the locking discipline is load-bearing, `Acquires the mutex internally.`

**Keep** genuine prohibitions and deprecations — "must not be called after `close()`", "callers should not rely on the ordering here". These regulate future behavior rather than negating a phantom alternative, so they belong.

**Rewrite** (a) and (b) into the affirmative equivalent. When the negation carries no residual information once affirmed, drop the sentence.

### Procedure

1. Run `git diff <base>` to find changed files and the touched line ranges. Two file kinds are in scope:
   - **Rust source** (`.rs`): all conventions apply.
   - **Hand-written Markdown docs** (`.md`) — e.g. `Document.md`, `README.md`, `CHANGELOG.md`, docs under `docs/`: only the prose conventions 4, 5, and 7 apply. **Exclude generated docs** under `std_doc/` (regenerated from source, so a hand edit would be overwritten).
2. For each changed `.rs` file, examine:
   - (a) comments that appear in the diff hunks (added or modified lines), for conventions 1–5 and 7;
   - (b) Rust items defined or whose signature was modified in the diff hunks, for convention 6.
3. For each changed hand-written `.md` file, examine the prose added or modified in the diff hunks for conventions 4, 5, and 7.
4. For each violation:
   - Identify which convention (1–7).
   - For 1–5 and 7: propose a rewrite that preserves the intent but removes the anti-pattern, then apply with `Edit`.
   - For 6: write a meaningful one-liner above the item, or flag it for review if you cannot articulate the purpose without restating the name.
5. After all edits, run `cargo check` to confirm nothing broke (comment edits shouldn't affect builds, but verify in case of doctest changes). Markdown edits don't affect the build.
6. Report:
   - **Applied edits**: file, convention, brief rationale.
   - **Flagged for review** (convention 6 only): file, item name, why no comment was written.

### Scope

- **Do not rewrite comments just because they're long.** Length is not the issue; the listed anti-patterns are.
- **Do not enforce conventions beyond the seven above.** Other style judgments (tone, capitalization, line wrapping) are not in scope.

---

## Aspect: no-personal-info

Checked-in files — source, config, and documentation — become public the moment they are committed and pushed. This aspect scans the diff for the user's *personal* data embedded in those files and flags it before it can land in history. It **never edits or deletes**: removing personal data is the user's call, and an automatic deletion could just as easily strip a legitimate value or a test fixture.

### What Counts as Personal Data

Flag, in the added/modified lines of any changed file:

- **Personal email addresses** — a Gmail / corporate / ISP address, or any real mailbox tied to an individual.
- **Real personal names** presented as author, owner, or contact — in a byline, an `authors = [...]` field, a comment signature, or a copyright line naming an individual.
- **Phone numbers, postal addresses, and other direct contact details.**
- **Secrets that authenticate a person** — API tokens, private keys, session cookies, passwords.

### Exceptions — Do Not Flag

- **Published identifiers the user uses in the open**: a public handle / username, a GitHub *noreply* address (`*@users.noreply.github.com`), or an address the project already publishes in its committed metadata. These are how the user chooses to be identified publicly.
- **Placeholder / example values**: `user@example.com`, anything under the reserved `example.{com,org,net}` domains, `127.0.0.1`, obviously-fake keys.
- **Data that is the point of the code**: a test fixture whose subject *is* a name/email string, a parser test exercising address formats, documentation *about* this very rule. When the value is the thing under test, it is not a leak.

### Procedure

1. Run `git diff <base>` to find every changed file — all kinds, not only `.rs`.
2. For each file, scan the **added/modified lines** for the categories above.
3. For each candidate, check the exceptions. When unsure whether a value is personal or published/placeholder, flag it — the user makes the final call.
4. **Make no edits.** Collect every surviving candidate as a flagged item.
5. Report:
   - **Flagged for review**: file, line, enough of the value to locate it (do not reproduce a full secret), and its category.
   - If nothing was found, **say so explicitly** — the orchestrator treats a clean result as permission to run its commit step.

### Scope Discipline

- **Flag only; touch no files.** This is the one aspect that never edits.
- **Only added/modified lines are in scope.** Pre-existing personal data in untouched parts of a changed file is out of scope; this aspect guards against *new* leaks in the diff.
- **A single flag blocks the commit.** The orchestrator will not run its commit step while any finding stands — that interlock is the point, so don't downgrade a genuine hit to a passing note.
