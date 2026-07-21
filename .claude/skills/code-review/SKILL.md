---
name: code-review
description: "Run review aspects sequentially against a chosen scope of code via subagents. Each subagent applies one aspect's conventions (fix-test-main-reference, design-fit, refactor-scope, test-sufficiency, code-quality, naming, shorten-qualifiers, comment-style, no-personal-info), all defined in this same file. Each editing aspect runs twice — once inside the diff, once over the rest of the touched files for behavior-preserving cleanups — and the orchestrator commits every pass on its own, then applies `cargo fmt` as a final standalone commit, and records in memory how far this branch has now been reviewed. Use when: reviewing code just written by AI (uncommitted changes), reviewing whatever has accumulated since the last review ('review the unreviewed code' — resumes from the recorded checkpoint), or doing a pre-merge review of an entire branch."
argument-hint: "Scope: 'unreviewed' for everything since this branch's last recorded review, 'uncommitted' for staged+unstaged changes, 'last N' for the last N commits, 'branch' for everything since the branch forked from main, or any git ref. If omitted, the skill asks."
---

# Code Review

Run a fixed sequence of review aspects against a chosen scope, **one after another** in subagents. The orchestrator section below resolves scope and dispatches subagents. The aspects (`## Aspect: ...` sections, further down) define the conventions each subagent applies — they are not separate skills, they are sections of this file that subagents read directly. As the editing aspects run, the orchestrator commits each one's changes as its own commit — the fixes inside the diff and the cleanup that aspect made around it as two — and finishes with `cargo fmt` in a standalone commit, so every pass is a separate, reviewable commit and formatting churn never mixes with the substantive changes.

## Scope

This orchestrator owns scope selection. It resolves the argument into a single **base ref** and passes that base to each subagent. Subagents run their own `git diff <base>` to find changes — they do **not** decide scope themselves.

| Argument            | Base ref                                  | Use when                                                         |
| ------------------- | ----------------------------------------- | ---------------------------------------------------------------- |
| (none)              | *ask the user* — see Procedure step 1     | The invoker did not specify a scope                              |
| `unreviewed`        | the branch's recorded checkpoint — see *Review Checkpoints* | Reviewing whatever has accumulated since the last review, working tree included |
| `uncommitted`       | `HEAD`                                    | Reviewing code just written / staged but not yet committed       |
| `last N` (e.g. `last 3`) | `HEAD~N`                             | Reviewing the last N commits on the current branch               |
| `branch`            | `$(git merge-base HEAD main)`             | Pre-merge review of the whole branch since it forked from `main` |
| `<git ref>`         | `<git ref>`                               | Arbitrary base (a commit hash, another branch, etc.)             |

Every scope reviews the working tree as it stands, so uncommitted changes are always included — the base ref decides only how far back the review reaches.

## Review Checkpoints

A completed review records how far it got, so the next one can pick up from there. The record lives in the session memory directory (the path is in the memory instructions the orchestrator already carries), as a single memory file named `code-review-checkpoints`, type `project`, holding one line per branch:

```
- <branch>: reviewed through <short hash> (<subject>) — <YYYY-MM-DD>
```

The recorded hash is `HEAD` **after** the review's own commits land, so the next review starts past the cleanup commits this one made. Branch is the key: worktrees of this repository share one file, and a branch's line is updated in place rather than appended to.

The checkpoint is a record of work done, so it is written only by a review that ran to completion. A review halted by the PII gate or by a subagent failure leaves the previous checkpoint standing.

## Review Radius

A review reaches past the hunks it was handed, because code improves only where someone is already working: the stretches between hunks are the stretches nobody ever cleans. Three rings, and the ring decides what an aspect may do:

- **Ring 1 — the diff hunks.** Every convention of the aspect applies in full, editing and flagging alike.
- **Ring 2 — the rest of each touched file.** Behavior-preserving cleanups apply, under the budget below; whatever else the aspect notices here becomes a finding.
- **Ring 3 — the rest of the project.** Findings only. Aspects read ring 3 freely — `code-quality`'s search for an existing helper and `refactor-scope`'s hunt for near-duplicates both need it — and they edit nothing there.

### What may be edited in ring 2

Only edits that **preserve behavior by construction**, so that untouched code stays correct without leaning on tests the change under review never exercised:

- Comments and doc comments, including one added to a pre-existing undocumented item.
- Imports and qualified paths.
- Renames of local bindings — `let`, loop and `match` binders, closure and function parameters.
- Deletion of commented-out code.
- Extraction of a block duplicated within the file into one function, with both call sites moved onto it.

Everything else stays a finding in ring 2: item renames, moves, signature changes, splitting a function, narrowing mutable state, rewriting an algorithm, dropping a defensive branch. Each changes an interface or a behavior, and the change under review carries no test that would catch a mistake there.

An unused private item outside the hunks is a finding as well, even though deleting it would compile: CLAUDE.md keeps such an item — and the `dead_code` warning it carries — as the reminder that a staged rollout still has a step to go, so the author decides whether it has served its purpose.

### Budget

Ring-2 work is opportunistic, so it is capped: **at most five edits per file per aspect**, taken nearest-first outward from the hunks, each one justifiable on its own. When candidates remain past the cap, report how many and let the author decide whether to widen. Three aspects sit outside the cap:

- `shorten-qualifiers` applies its whole-file convention uncapped — a half-converted import block reads worse than either end state.
- `code-quality`'s fail-loud fallback scan already covers the whole touched file as a correctness check, and runs as written in ring 1.
- `no-personal-info` works in ring 1 alone: it gates what this diff would add to the history.

### Modes

Each editing aspect runs twice: once in **`in-diff` mode** (ring 1; ring-2 candidates are listed and left alone), then once in **`neighborhood` mode** (ring 2, budget applied). Each mode is committed on its own, so the cleanup near the change is a separate commit the author can weigh — or revert — as one unit.

## Aspect Sequence

Run these aspects in this order, each in its own subagent. The **flag-only** aspects (they only report findings, never edit) run first; then the **editing** aspects (they modify files, and each is committed on its own — see the Procedure):

1. **no-personal-info** — scan every changed file for the user's personal data (real name, personal email, phone, address, secrets) embedded in checked-in files, and flag it. Runs first as a gate: a finding stops the review before anything is committed.
2. **design-fit** — with the implementation now visible, re-evaluate whether the chosen design is the best fit for the change's goal; flag mismatches (this aspect never redesigns).
3. **refactor-scope** — check whether the change bent itself out of shape to leave existing code untouched; flag the scars a declined refactor left behind (this aspect never refactors).
4. **test-sufficiency** — check whether the tests cover what the implementation actually does, including cases only visible once the code exists; flag coverage gaps (this aspect never writes tests).
5. **fix-test-main-reference** — for changed Fix-source compile tests, ensure every top-level declaration introduced by the test is referenced from `main` (directly, transitively, or via `eval`); otherwise the Fix compiler can silently skip a broken definition.
6. **code-quality** — apply general programming-maxim review (DRY, single responsibility, dead-code removal, defensive-code trimming, shotgun-surgery annotation, root-cause vs symptom check, etc.).
7. **naming** — judge the names the diff introduces; rename local bindings inline, flag item names (modules, types, functions, fields) for the author.
8. **shorten-qualifiers** — replace verbose `crate::module::Type` paths with imports (also covers any new imports the `code-quality` pass introduced).
9. **comment-style** — apply the project comment/doc conventions (Rust comments and hand-written Markdown docs) to whatever survived the earlier editing passes.

## Why Sequential, Not Parallel

1. **Avoid conflicting edits.** The editing aspects modify files. Parallel runs would fight each other.
2. **Each aspect should see prior changes.** E.g., `shorten-qualifiers` should see imports added by `code-quality`; `comment-style` shouldn't waste effort polishing comments that `code-quality` just deleted.
3. **Per-aspect commits need it.** Each editing aspect is committed on its own, and each of its two modes separately again (see the Procedure), which means running and committing them one at a time.

## Procedure

**Running an aspect** (used in the steps below): extract its section from this file — everything from `## Aspect: <aspect-name>` up to (but not including) the next `## Aspect:` heading or end of file, all `### ...` sub-sections included — then launch one subagent via `Agent` (subagent_type: `general-purpose`), brief it with the prompt template below (substitute the aspect name, the mode, the base ref, the *Review Radius* section, and the extracted aspect text inline; do **not** tell it to open `SKILL.md`), and **wait** for it to finish before starting the next. Never use `run_in_background`. The flag-only aspects run in `in-diff` mode, where the mode makes no difference to a pass that edits nothing.

1. **Resolve the base ref** from the argument:
   - empty → ask the user which scope they want using `AskUserQuestion`. Offer four options; the free-text option the tool adds covers any git ref the user wants to name:
     1. **Since the last review** — everything after this branch's recorded checkpoint. Offer this first when a checkpoint exists for the current branch, naming the commit it would start from.
     2. **Uncommitted** — staged + unstaged changes (`HEAD`).
     3. **Branch (vs main)** — everything since this branch forked from `main` (`$(git merge-base HEAD main)`).
     4. **Last N commits** — review the last N commits on the current branch (resolve to `HEAD~N`). When the user picks this, ask a follow-up question for `N`.

     After the user picks, resolve to the corresponding base ref using the rules below.
   - `unreviewed` (also: "review the code that hasn't been reviewed yet", or any phrasing asking for what has accumulated since last time) → read the `code-review-checkpoints` memory and take the line for the current branch (`git branch --show-current`). Then:
     - Verify the recorded hash is still an ancestor of `HEAD`: `git rev-parse --verify <hash>^{commit}` and `git merge-base --is-ancestor <hash> HEAD`. A hash that no longer resolves, or that sits outside this branch's history, means the branch was rebased or reset since the checkpoint was written — say so and ask the user for a base instead of guessing.
     - When the hash equals `HEAD` and `git status --porcelain` is empty, there is nothing to review: report that and stop.
     - When no line exists for this branch, say so and ask the user for a base, offering `$(git merge-base HEAD main)` as the natural starting point for a branch that has never been reviewed.
     - When a merge brought in commits already reviewed on another branch, the diff will include them. Note it in the summary so the user can narrow the scope if the noise is not worth it.
   - `uncommitted` → `HEAD`.
   - `last N` (where N is a positive integer) → `HEAD~N`. Verify it resolves with `git rev-parse --verify HEAD~N`.
   - `branch` → `$(git merge-base HEAD main)`. Verify `main` exists; if the project uses a different default branch, abort and ask.
   - anything else → treat as a git ref. Verify it resolves with `git rev-parse --verify <ref>`.
2. **Run the flag-only reviews first**, in order: `no-personal-info`, `design-fit`, `refactor-scope`, `test-sufficiency`. They only report findings; they make no edits.
   - **PII gate.** If `no-personal-info` flagged any finding, **stop the review here**: commit nothing, and surface that finding together with any `design-fit` / `refactor-scope` / `test-sufficiency` findings so the user can remove the personal data before re-running. Because these aspects make no edits, the working tree is untouched.
3. **Commit the code under review.** If `git status --porcelain` reports pending changes, they are part of what was just reviewed: commit them now as their own commit, with a message describing the change (you have the context of what was written; if it is genuinely unclear, use a concise placeholder and say so in the summary). This keeps the reviewed code separate from the cleanup commits that follow. On a clean tree, skip this step.
4. **Run the editing aspects, committing each mode separately**, in order: `fix-test-main-reference`, `code-quality`, `naming`, `shorten-qualifiers`, `comment-style`. For each aspect, in turn:
   - Run it in **`in-diff` mode**. If it changed any files, commit exactly those changes — `git add -A && git commit -m "code-review: <what this aspect did>"` (e.g. `code-review: shorten qualified paths`).
   - If it reported ring-2 candidates, run it again in **`neighborhood` mode**, handing it that list as its starting point, and commit what it changed as `code-review: <what this aspect did> — cleanup near the change`. An aspect that reported no candidates skips this second run.

   A mode that changed nothing produces no commit. **Per-aspect, per-mode, fine-grained commits are the goal — never bundle several into one commit.**
5. **Apply `cargo fmt` as a standalone commit.** Run `cargo fmt`; if `git status --porcelain` then reports changes, commit them on their own — `git commit -am "Apply cargo fmt"`. If nothing changed, make no commit and note the code was already formatted.
6. **Record the checkpoint.** Take `git rev-parse --short HEAD` and write it to the `code-review-checkpoints` memory under the current branch, in the format given in *Review Checkpoints* — replacing that branch's existing line, and adding the `MEMORY.md` pointer when the memory file is new. A branch whose review found nothing to change still gets its line updated: the point of the record is how far the review reached, and that advanced regardless.
7. **Summarize.** For each editing aspect, give a one-line description of what it changed in each mode (or note it changed nothing), keeping the cleanup near the change visible as its own line so the author can judge it separately; surface every flagged finding, both from the flag-only reviews (`design-fit`, `refactor-scope`, `test-sufficiency`) and from the editing aspects' report-only items (e.g. `code-quality` hacks, `naming` item renames); list every commit created, with its short hash; and state the base ref the review covered and the checkpoint now recorded.
8. **Stop on failure.** If any subagent reports an error (aspect couldn't run, build broke, etc.), stop and surface the failure; do not continue, and leave the checkpoint at its previous value. If `cargo fmt` itself fails, surface that and skip the formatting commit.

## Subagent Prompt Template

```
You are running one aspect of a code review.

Aspect: <aspect-name>
Mode: <in-diff | neighborhood>
Base ref: <base>

The base ref is the comparison point: review the diff between <base>
and the current working tree, i.e. run your own `git diff <base>` to
find the files and hunks to operate on.

How far your edits may reach is set by the mode and by the radius rules
below.

In `in-diff` mode, edit inside the diff hunks (ring 1). Wherever the
aspect would also fix something in the rest of a touched file, collect
it as a **ring-2 candidate** — file, location, the one-line fix, and
which convention it comes from — and leave that code alone.

In `neighborhood` mode, work ring 2 of the touched files: start from
the candidate list below, add anything it missed, keep the edits the
radius rules allow, and turn the rest into findings.

Ring-2 candidates carried over from the in-diff pass:
<the list the in-diff run reported, or "none — this is the in-diff pass">

----- BEGIN RADIUS RULES -----
<paste the full text of the `## Review Radius` section here, including
all its `### ...` sub-sections>
----- END RADIUS RULES -----

The full instructions for this aspect follow between the BEGIN/END
markers. Treat them, together with the radius rules, as your sole
instructions. Do not look elsewhere for additional conventions.

----- BEGIN ASPECT INSTRUCTIONS -----
<paste the full text of `## Aspect: <aspect-name>` here, including all
its `### ...` sub-sections, up to but not including the next
`## Aspect:` heading or the end of file>
----- END ASPECT INSTRUCTIONS -----

Apply the edits the aspect prescribes within your mode's ring. If you
modified code, run `cargo check` afterwards to confirm the project
still builds.

Report back in under 100 words: which files you touched and a one-line
summary of the change in each. Report separately, and in full, every
finding the aspect asks you to flag, and — in the in-diff pass — the
ring-2 candidate list. The word limit governs the summary of your
edits; findings and candidates are added on top of it.
```

## What NOT to do

- Don't run aspects in parallel.
- Don't let subagents decide their own scope — always pass the resolved base and the mode.
- Don't let a `neighborhood` pass edit past the radius rules: an interface change or a behavior change outside the hunks is a finding, whatever the mode.
- Don't let neighborhood edits ride along in an `in-diff` commit — the split is what lets the author revert the cleanup on its own.
- Don't continue the chain if a step fails.
- Commit each editing aspect separately — don't bundle several aspects' edits into one commit, and always give `cargo fmt` its own commit.
- Don't commit anything when `no-personal-info` flagged a finding — stop and surface it so the user can remove the personal data first.
- Don't record a checkpoint for a review that stopped early — the record must mean "everything up to here was reviewed".
- Don't fall back to a guessed base when a recorded checkpoint fails to resolve — ask the user.

---

## Adding an Aspect or a Convention

Every convention below is read in full by its aspect's subagent on every review, so the set is a shared budget: one that rarely fires spends the attention that the ones firing often need. What earns a place is a **class of mistake**, described so that it can be recognized in code written by another author, in another subsystem, months from now. A concrete case is welcome as the illustration; the rule is what the entry is made of.

Before adding one, strip the case that prompted it away and read what remains. An entry that no longer says what to look for was a report about one incident — the durable form of that is a regression test, or an assertion at the invariant it violated. An entry that names a specific function, type, module, or pass fails the same way: only a reader who already knows the case can apply it.

Prefer extending the convention whose subject already covers the class over adding a sibling beside it, and prefer a new convention inside an existing aspect over a new aspect. A new aspect earns its own section when it asks a question none of the existing ones ask — a different lens on the code, rather than another rule under a lens already here.

---

# Review Aspects

The sections below are referenced by name from the orchestrator. Each is self-contained: a subagent should be able to apply an aspect by reading only its section.

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

- **Touch only test files the diff changed.** Within those files, the mode decides which tests you may edit: the ones the diff introduced or modified in `in-diff` mode, the file's other tests in `neighborhood` mode. A test file the diff leaves alone is out of scope in both.
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

## Aspect: refactor-scope

A change is judged by the codebase it leaves behind, so reshaping existing code is part of a change's legitimate scope — the project's standing instruction is to prefer the cleanliness of the end state over the smallness of the diff. This aspect reads the seam between the new code and the code it was grafted onto, and asks: **did the change bend itself out of shape so that the existing code could stay untouched?**

Every other editing aspect confines itself to the diff hunks by design. That discipline keeps them safe, and it also makes them blind to this: a duplicate created to avoid editing an existing function looks perfectly clean from inside its own hunk, because the defect lives in the relationship between the new item and the old one.

This aspect **only flags**; it never refactors. Widening a change to reshape existing code is the author's call, and the ripple through call sites, tests, and downstream Fix programs needs judgment this review does not have.

### Distinguish from design-fit

`design-fit` judges the design of the new code against the goal, taking the existing code as a fixed backdrop. This aspect questions the backdrop: the shape that fits the goal may have required changing an existing function, type, or module, and the finding is the mark that leaving it alone left on the new code.

### Scars to look for

Each of these is concrete evidence in the diff that existing code was routed around:

- **A near-duplicate of an existing item.** A new function, type, or pass that is an existing one plus a tweak — `resolve_symbol_with_span` beside `resolve_symbol`, a second visitor differing in one arm, a parallel enum carrying the same cases. The end state wants one generalized item with both call sites on it. This is the highest-yield check, and it takes a search across `src/`: the twin is usually in another module, which is exactly why the author wrote a fresh one.
- **A `bool` or `Option` parameter added to an existing function to serve the new caller.** A flag argument is the record of a seam the author declined to move: the two behaviors want either two functions, or one function with the varying part lifted into the caller.
- **A conversion or adapter that exists only because an upstream type stayed as it was.** The new code reshapes a value at every call because changing the type — or adding the constructor it wanted — would have touched more files.
- **A constant, table, or invariant re-stated in the new code** because the existing copy lives in a module the change avoided. Beyond the duplication, this plants the shotgun-surgery trap that `code-quality` documents.
- **A forwarding wrapper preserving a superseded signature.** The old entry point kept alive as a one-line delegation so its call sites need no edit. When every caller is inside this repository, updating them and deleting the forwarder is the cleaner end state.
- **A special case grafted onto an existing function** where the general rule the new caller needs would have subsumed the old behavior as well.
- **A superseded path left standing.** The change introduced the replacement, and the old code is still reachable because deleting it and its callers would have widened the diff. `code-quality` removes only dead items the diff itself introduced, so a path that *became* dead falls to this aspect.

The litmus test: **explain the resulting code to someone who never saw the diff.** If the explanation of why there are two of something, why a flag exists, or why a conversion happens is "so that the change would touch fewer files", then the reason describes the diff — and the diff is gone the moment it lands, leaving behind only the thing it justified.

### Discipline

- **Flag only, never refactor.** No edits.
- **Anchor every finding on a scar in the new code** — the duplicate, the flag, the adapter, the wrapper, the stranded path. Pre-existing mess that the change neither created nor bent around belongs to the editing aspects' neighborhood pass; raising it here buries the findings that the diff is actually responsible for.
- **Price the ripple.** A finding must say what the refactor would touch: which files, roughly how many call sites, which tests. "Generalize this" with no estimate leaves the author no basis to decide.
- **Name the compatibility cost.** Some existing items have consumers outside this repository — Fix standard-library signatures, the LSP protocol surface, APIs that external Fix projects call. Reshaping those is a compatibility decision; flag it all the same, and state that cost as part of the finding.
- **An author who priced it already has answered.** When a commit message or a comment states why the existing code was left as it is, treat that as the decision and skip the finding.

### Procedure

1. Read `git log <base>..HEAD` and `git diff <base>`, and state in one or two sentences what the change is for.
2. List the items the diff adds — functions, types, traits, passes, constants. For each, search `src/` for an existing counterpart it resembles in name, signature, or shape, and read any candidate in full before judging.
3. For each existing item the diff **modifies**, read its pre-diff version (`git show <base>:<file>`) and ask whether the modification is a graft — a flag, an extra case, a widened type — where reshaping would have served the old and new callers together.
4. For each existing item the diff **calls but leaves alone**, ask whether the new code bends around it: a conversion at every call site, a re-stated constant, a value threaded through only to satisfy its signature.
5. Confirm each candidate against the code, check it against *Discipline*, and collect the survivors. Make no edits.

### Report

- **Flagged for review**: per finding — the scar in the new code (file, and what it is), the existing code that would have to change, the end state you would aim for, and the cost of getting there.
- If the change reshaped existing code wherever it needed to, say so in one line and flag nothing.

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

Walk the Rust code changed against the base ref, check it against the conventions below, apply the safe fixes inline, and surface the riskier ones for the user to decide.

### Out of Scope

- Comments → handled by the `comment-style` aspect.
- Import paths and wildcard imports → handled by the `shorten-qualifiers` aspect.
- Formatting → the orchestrator runs `cargo fmt` in a final step; don't reformat here.
- Major redesigns / architectural changes → flag for the user, do not perform.

### Conventions

#### DRY: don't repeat logic; reuse existing utilities

If the diff introduces logic that already exists elsewhere in the project, replace the duplicate with a call to the existing helper.

This project is a compiler bundled with a sizable toolchain — LSP server, package manager, documentation generator, build runner. That breadth alone makes it overwhelmingly likely that "generic" supporting logic — file/path handling, name and span arithmetic, source-text scanning, dependency graph traversal, version comparison, manifest parsing, AST/type walks, identifier formatting, and so on — is already implemented somewhere in `src/`. Before writing such logic, **reason from the project's feature set first**: ask "is this the kind of helper a compiler / LSP / package manager / doc generator would plausibly already have?" When the answer is yes (it usually is), search the entire `src/` tree to confirm before adding a new one. Search the whole tree rather than the files around the new code — helpers cross subsystem boundaries freely, and a routine written for one feature is often exactly what another wants.

If your reasoning suggested the helper should exist but the search comes up empty, **add it where it ought to live, not where you happen to need it.** For example, if the diff inlines `s.start <= pos && pos <= s.end` for a `Span`, and you were expecting `Span` to expose an `includes` (or similarly named) method, add the method on `Span` rather than leaving the arithmetic at the call site. The very reasoning that made you go looking — "this is the kind of thing `Span` would expose" — applies just as strongly to *placing* the new helper as to *finding* an existing one. Inlining at the use site forfeits the benefit and guarantees the next caller will re-derive the same expression.

**Apply** when an existing helper is a near-identical match, or when no helper exists but its proper home is unambiguous (a method on the obvious receiver type, a constructor on the obvious factory, etc.).
**Report only** when an existing version diverges enough that merging would change behavior, or when no helper exists and the right home is a judgment call.

#### Extract a function on the second copy

This convention is specifically about **function extraction** (関数化), not about introducing traits, generics, or other heavier abstractions.

When the diff contains two or more blocks of code with the same intent — same shape, same purpose, differing only in variable names or values — extract a shared function. The threshold is **two, not three**: don't wait for a third copy to appear, because two copies will already start drifting apart.

**Apply**: Two near-identical blocks with the same intent → extract a function, replace both call sites.

#### Single responsibility per function

A function should do one thing at one level of abstraction. Watch for functions that mix:
- I/O and pure computation.
- Parsing and validation.
- Building data and rendering it.

**Apply** when the split is mechanical (a long function with a clear seam, no shared mutable state across the seam).
**Report only** when the split would require redesigning callers.

#### Don't add defensive code inside the trust boundary

CLAUDE.md is explicit: validate at boundaries (user input, external APIs, file I/O), trust internal calls. Reject:
- `unwrap_or_default()` / `if let Some` guards on values the caller statically guarantees.
- `Result` propagation on paths that cannot fail.
- Re-validation of arguments already checked upstream.

**Apply**: remove the defensive branch.
**Keep** the guard only when it documents a real precondition the type system can't express — and in that case leave a one-line comment naming the invariant.

#### Don't let a fallback silently handle a case the author calls impossible — fail loud

A catch-all or fallback that absorbs a case the author believes cannot occur — `_ => self.clone()`, `unwrap_or(default)`, a silent clamp, "return the left side" at a merge, an early `return` on a "shouldn't happen" shape — trades a loud failure for a silent wrong answer. If the case truly is impossible, the fallback is dead code lending false confidence. If a bug elsewhere ever makes it reachable, the fallback *swallows that bug*: execution continues with a plausible-but-wrong value instead of stopping where the invariant broke. In correctness-critical code — an analysis feeding codegen, a type checker, an optimizer — that wrong value becomes a miscompile surfacing far from its cause.

This comes in two shapes. The **easy-to-spot** one is a comment asserting the invariant right next to a fallback that contradicts it — "both sides have the same type, so this can't arise — fall back to the left", "shapes never mismatch — keep the function total": the comment says the branch is unreachable, the code quietly handles it anyway, so one of the two is wrong. The **more common and more dangerous** one carries *no* comment at all — a bare `_ => default`, an `unwrap_or(0)`, a `.get(i).copied().unwrap_or(...)` — where the author never flagged the assumption. A missing comment is not reassurance; it usually means the reachability was never even considered. So do not scan only for contradicting comments: treat **every** catch-all and silent default as a question — *which concrete case does this absorb, and can it actually occur?*

```rust
// Both are suspect. The first at least announces its assumption; the second hides it entirely.
match (self, other) {
    (Foo(a), Foo(b)) => Foo(a.merge(b)),
    // Mismatched shapes never arise from a well-typed program.
    _ => self.clone(),
}
let elem = arr.get(i).copied().unwrap_or(0);   // is `i` out of range ever a real case, or a bug?
```

The default in the **conservative / safe direction** is the one that slips through most often, because it reads as obviously harmless — an analysis defaulting to "assume shared" (`Dynamic`), a predicate to `false`, an ownership to `Borrow`, a lookup to an empty set. It is *not* harmless when the absorbed case is unreachable: the safe default still hides a logic bug at the point the invariant broke, and "safe today" is one refactor away from "unsafe tomorrow". **Reachability, not the safeness of the default, decides.** An unreachable case must fail loud however conservative its fallback looks; "it errs on the safe side" is not a reason to keep it. Do not let a `map_or(SAFE, …)` / `unwrap_or(SAFE)` pass just because `SAFE` could not itself miscompile.

**Apply**: replace the fallback with `unreachable!("… {:?} vs {:?}", a, b)` (or `panic!` / `debug_assert!`) so a violated invariant fails at its origin with the offending values in the message. Crucially, **verify that the absorbed case really is impossible rather than trusting it** — exercise the code, or temporarily make the arm panic and run the suite. The case is often actually reachable; when the arm fires, you have found a real bug (this is one of the higher-yield checks — a swallowed case is exactly where latent bugs hide). Fix the upstream cause, and *then* the arm is genuinely unreachable.
**Keep** the fallback only when the case turns out to be a **supported input** the caller legitimately reaches — and then document the concrete scenario, not a vague "just in case".
**Report only** when converting to a hard failure could crash on inputs you cannot prove absent — flag it for the author to confirm the invariant.

**Catch the refactor regression.** When a hunk *rewrites* an existing function, compare it against the version it replaced — not only against the review base. A refactor that relaxes a prior `assert!` / `unreachable!` / `panic!` into a silent default (a representation change that "simplifies" a fail-loud arm back into a `map_or(default)`) reintroduces a swallowed case, and that is as much a defect as writing one fresh. Read the pre-refactor body (`git show <base>:<file>` for the symbol) whenever a rewritten function now returns a default where it used to abort.

**Scope for this convention: the whole touched file, not just the diff hunks.** Fallbacks accrete over time and refactors relocate them, so a hunk-only view routinely hides them (a swallowed case three functions away from the change is still the bug that bites). This is the one code-quality convention that reaches past the mode's ring: scan every function of each file the diff touches, in either mode. For a correctness-critical subsystem (an analysis feeding codegen, a type checker, an optimizer), a periodic dedicated sweep of the whole subsystem — every `_ => …`, `unwrap_or`, `map_or`, `.get(…).unwrap_or…` — catches still more than any diff-triggered pass.

#### Remove dead and half-finished code

Delete:
- Functions / structs / enums introduced by the diff with no callers anywhere (`grep` to confirm).
- Commented-out code.
- `_unused` parameters left as a "will use later" placeholder.
- `todo!()` / `unimplemented!()` branches the diff doesn't actually exercise.

After deletion, run `cargo check` — the build will surface anything you misjudged as dead.

#### Avoid obvious quadratic / repeated-allocation patterns

Catch *only* clear, unbounded cases — not micro-optimizations:
- `Vec::contains` inside a loop over the same `Vec` → use a `Set`.
- `format!("{}{}", a, b)` inside a tight loop → `push_str` / `write!` against one buffer.
- Repeated `clone()` of large data when a borrow would do.

**Apply** when the inputs are clearly unbounded (collections of unknown size, iterators over user data).
**Skip** when the upper bound is small and fixed (e.g., iterating over a 4-variant enum).

#### Narrow the scope of mutable state

Every `let mut` and every public field expands the surface a reader must hold in their head. Prefer:
- Pushing the mutation into a `let x = { let mut tmp = ...; ...; tmp };` block so the binding is immutable outside.
- Returning a built value rather than taking an `&mut` out-parameter.
- Marking fields private when the diff doesn't need them public.

**Apply** for trivial scope tightening.
**Report only** when narrowing scope would touch multiple call sites.

#### Avoid shotgun-surgery coupling; annotate it when unavoidable

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

#### Struct fields must be non-redundant and on-role

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

#### Fix root causes, not symptoms

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

#### Use the project's canonical types over their standard-library counterparts

Where the project provides its own version of a common type, use it instead of the standard-library one. These are mechanical substitutions, not stylistic preferences — they exist so that the rest of the codebase can rely on a single consistent type.

- Use `Set` / `Map` from `crate::misc`, not `std::collections::HashSet` / `std::collections::HashMap`.

**Apply** unconditionally.

#### Extract named steps for readability

The DRY and function-extraction conventions catch *duplication* (same code in two places). This one catches *cohesion*: a block of code that is conceptually one named step embedded inside a larger function, even though it only appears once. Extracting it into a helper lets the caller read as a sequence of named steps and keeps a single abstraction level per function.

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

#### Place each item in the namespace that fits its role

Where a value, function, or method *lives* is part of its interface. An item filed under the wrong module — or left as a free function when it is really an operation of one type — sends the reader looking in the wrong place and invites the next duplicate, because the natural home looked empty. This is the code analogue of *Struct fields must be non-redundant and on-role*: that convention keeps a value in the struct whose role it matches; this one keeps an item in the module or on the type whose role it matches.

Placements to correct:

- **A free function that is really an operation of one type.** If a function takes some `T` as its primary argument and reads as something `T` *does* (or produces a `T` from nothing else), it belongs as a method or associated function on `T`, not as a loose function in an unrelated module — the same reasoning the DRY convention applies when *adding* a helper, here applied to one the diff has already placed.
- **An item filed in a module about a different concern.** A function whose name and behavior are about subsystem B, sitting in subsystem A's file because that is where it was first needed.

The litmus test: reading only the item's name and what it does, would someone new to the project look for it where it currently lives? If they would look on a type or in a module elsewhere, that elsewhere is its home.

This convention is distinct from `shorten-qualifiers`, which fixes how an item is *imported*, not where it is *defined*. It triggers on items the diff **adds or moves**; a pre-existing item the diff merely references is out of scope. Relocating an item necessarily edits its definition, its call sites, and the destination module's imports — those edits are the mechanical consequence of the move and are in scope even where they fall outside the original diff hunk.

**Apply** when the right home is clear and the move is mechanical: relocate the definition, update the call sites and imports, run `cargo check`, and prefer performing the move over deferring it.
**Report only** when choosing the home is a genuine design call, when the move would cross a crate or visibility boundary that needs judgment, or when the call sites ripple widely enough that the author should decide. Name the item, its current home, the home you would give it, and why.

#### Split an overgrown file at a natural seam

When the diff has grown a file to the point that it now spans several distinct concerns — different groups of types, or unrelated passes / utilities that merely share a file — and it has become large enough to be hard to navigate, flag it for splitting. **Report only**: moving code into new modules changes module paths, imports, and visibility across call sites, so it is a redesign the author should choose, not a hunk-local edit.

Split at a **natural seam**, never at an arbitrary line count: a cohesive group of related types and their methods, a self-contained submodule (a parser, a formatter, one compiler pass), or a cluster that shares a concern. Aim for files that each carry one responsibility — not two halves of one responsibility sawn apart at the midpoint.

In the report, name the file, say why it has become unwieldy, and describe the natural seam(s) to split along (e.g. "move the `Span` / `SourcePos` types and their impls into a `span` submodule"). Don't perform the split.

**Skip** when the file is long but genuinely one cohesive unit, or when the diff only touched a small part of an already-long file — flagging a split on a file the diff barely changed is out of scope, since that length is pre-existing.

#### No ad-hoc / hacky mechanisms

The root-cause-vs-symptom convention catches the *symptom-patch* flavor of ad-hoc code, but only in a diff that fixes a bug. This one is broader and un-gated: it asks of **any** changed code — feature work included — whether the mechanism it uses is sound, or whether it *works by coincidence* and will break the moment an unstated assumption shifts.

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
2. For each changed `.rs` file, walk the diff hunks and check each convention against the added/modified code.
3. For each violation:
   - Identify which convention it violates.
   - If it falls in "Apply": make the edit with `Edit`.
   - If it falls in "Report only": collect it for the final report; do not edit.
4. Run `cargo check` after edits. On failure, revert the offending edit and reclassify it as a flagged item.
5. Report:
   - **Applied edits**: file, convention (by title), one-line summary per change.
   - **Flagged for review**: file, convention (by title), what you saw and why you didn't auto-fix.

### Scope Discipline

- **Let the mode set the reach.** In `in-diff` mode, edit inside the diff hunks and collect what the rest of each touched file needs as ring-2 candidates; in `neighborhood` mode, work those candidates under the radius rules. One convention stands apart: *Don't let a fallback silently handle a case the author calls impossible* covers the whole of each touched file in `in-diff` mode already, per its own scope note — a swallowed case is a bug rather than opportunistic cleanup.
- **The conventions that travel to ring 2** are the ones whose edit preserves behavior by construction: *DRY* and *Extract a function on the second copy* within a single file, and *Remove dead and half-finished code* for commented-out code. The rest become findings in ring 2, since nothing in this change's tests would catch a mistake there — splitting a function, narrowing mutable state, rewriting a quadratic pattern, dropping a defensive branch, relocating an item, and also *Use the project's canonical types*, because `Set` / `Map` are `fxhash` maps whose iteration order differs from the standard library's and a compiler can let that order reach its output.
- **Do not redesign.** If the right fix is "extract a new module" or "rewrite this pipeline," report it; don't do it.
- **One convention at a time per hunk.** If a hunk hits multiple conventions, apply the smallest fix that satisfies one, then re-check before moving on.

---

## Aspect: naming

A name is the one piece of documentation every reader is guaranteed to read. A misleading name costs more than a missing comment: the reader trusts it, builds a wrong model of the code, and whatever gets written on top of that model inherits the mistake. This aspect reads the names the diff introduces and asks whether each lets the reader predict what the thing is.

It works in two modes, split by how far a rename reaches:

- **Item names** — modules, types, traits, trait methods, functions, methods, struct fields, enum variants, constants, and top-level Fix declarations. **Report only**: a rename here changes an interface and ripples across call sites, other crates, and Fix programs, so the author decides.
- **Local names** — `let` bindings, `for` / `while let` binders, `match` arm binders, closure parameters, and function parameters. **Apply**: their scope is one function body, so the rename is mechanical and self-contained.

### What to check

One question: **can the reader predict the contents from the name?** Read the name, fix the expectation it creates, then read the body and check whether the expectation held. `result`, `tmp`, `step2_output` create no expectation — they name the slot the value sits in. `timeout` on a bare integer creates one with the unit guessed. A `check_ty` that now also mutates the environment creates a confident and wrong one, which is the costliest of the three.

The rest is calibration that question alone does not supply:

- **Read the pre-diff version of a rewritten item** (`git show <base>:<file>`) before judging its name. A name that survived a behavior change is the most common wrong name in a diff.
- **The project's word wins.** A name can be clear on its own and still be wrong: `origin_info` beside an established `provenance`, or `prev` / `next` in a file that says `old` / `new`, makes the reader learn two words for one thing. `grep` the concept before choosing.
- **Abbreviations are calibrated per project.** `ty`, `expr`, `idx`, `arg`, `ptr` are established here and read fine; a fresh `cfgm`, `rslv`, `ntc` costs the reader a decoding step.
- **Grammar follows the role.** Functions and methods read as verb phrases (`resolve_symbol`); `bool`-returning predicates start with `is_` / `has_` / `can_`; conversions follow the Rust convention — `as_` for a cheap borrowing view, `to_` for a non-consuming conversion, `into_` for a consuming one. Types, modules, and fields read as noun phrases; a trait names a capability or a role.
- **A negated boolean stacks into double negatives at the use site.** `not_ready`, `disable_check` become `ready`, `check_enabled`.
- **Length scales with scope.** In a three-line scope the surrounding lines supply the picture, so `i`, `ty`, `n` carry it; a binding live across eighty lines, or an exported item, has to supply it alone. A long descriptive name inside a tight loop is noise.
- **No good name is a design signal.** For an item it usually means two jobs, or a seam in the wrong place. Report it as a finding naming the two jobs, instead of settling for `process_step_2`.

### Discipline

- **Item names stay report-only in every mode.** A rename there is a project-vocabulary decision that ripples across the repository, so it belongs to the author whether the item sits in a hunk or elsewhere in the file.
- **Local names follow the mode.** `in-diff` mode renames the local bindings the diff introduces or gives new meaning, and collects the rest of the touched file's misleading locals as ring-2 candidates; `neighborhood` mode renames those, under the radius rules. A local rename is contained in one function body, which is what makes it safe to carry into untouched code.
- **A synonym is not an improvement.** Rename when the current name misleads, hides the meaning, or breaks the project's vocabulary. Renaming for taste costs review attention and muddies `git blame`.
- **Check a proposed term against the codebase.** `grep` the candidate and the concept first — the right name is usually the one the project already uses.
- **Parameter renames carry two obligations**: a parameter of a trait `impl` method keeps the name its trait declaration uses, and any `# Arguments` entry or doc-comment mention of the parameter is updated with it.
- **Run `cargo check` after renames.**

### Procedure

1. Run `git diff <base>` to find changed files and hunks. Rust sources (`.rs`) and Fix sources (`.fix`, including Fix source strings embedded in tests) are in scope.
2. Collect the names the diff introduces: item declarations and local bindings inside the hunks. For an item the diff rewrites, read its pre-diff version as well.
3. Judge each name against *What to check*.
4. For a local name that fails: rename it with `Edit` at every occurrence in its scope. For an item name that fails: collect it as a finding and leave the code alone.
5. Run `cargo check`.
6. Report:
   - **Applied renames**: file, `old` -> `new`, one line on what the old name hid.
   - **Flagged for review**: file, item, current name, proposed name, and what the current name misleads the reader about. When no good name is available, say what the item does that resists naming.
   - If the names read well, say so in one line.

---

## Aspect: shorten-qualifiers

Three related cleanups for Rust import style:

1. **Shorten qualified paths** — replace `crate::foo::bar::Baz` with `Baz` plus a `use` import, as far as the short name still identifies what it names (see *Keep the qualifier that carries the meaning*).
2. **Eliminate wildcard imports** — replace `use foo::*;` with an explicit list of the names actually used.
3. **Collapse the use block** — remove blank lines between `use` statements at the top of the file. The project convention is one contiguous block; sectioning (std vs external vs crate, as `rust-analyzer` likes to do) is not meaningful here.

The project convention is *explicit imports, no wildcards, no section breaks*. All three cleanups serve that convention.

### Procedure

1. **Collect changed files**: Run `git diff --name-only <base>` to find affected files.

2. **Identify cleanup targets**: For each affected file, search the **entire file**. The diff is used only to determine *which files* to process. In `in-diff` mode, fix the violations that sit in the diff hunks and list the file's remaining ones as ring-2 candidates; in `neighborhood` mode, fix those. This convention runs uncapped in ring 2 — a file whose import block is half converted reads worse than one at either end state. Look for:
   - **Wildcard imports**: `use module::*;` (and grouped variants like `use module::{*}`).
   - **Qualified paths**: `crate::module::Ident`, `crate::module::{A, B}`, `module::submodule::Ident`. Paths used as types, function calls, trait bounds, or in expressions.
   - **Blank lines inside the top-of-file `use` block**: any empty line between two `use` statements at the start of the file. These are typically `rust-analyzer`-inserted section breaks (std / external crates / `crate` / `super`) that the project does not want.

3. **Read each file's existing imports**: For each affected file, read the `use` block at the top to know what is already imported.

4. **Plan replacements**:
   - **For each wildcard import** `use foo::*;`: list every identifier from `foo` actually referenced in the file, and rewrite the import as `use foo::{A, B, C};` (or merge into an existing line if one already imports from `foo`).
   - **For each qualified path**:
     - Determine the short name (last segment, e.g., `Baz` from `crate::foo::bar::Baz`).
     - Apply the litmus test in *Keep the qualifier that carries the meaning*. When the module segment is what identifies the item, import that module and leave the call written as `module::item`.
     - Check if the short name conflicts with another import or a different qualified path in the same file.
       - **No conflict**: Add a `use` statement and replace all occurrences with the short name.
       - **Conflict**: Keep the minimal qualification needed to disambiguate (e.g., `bar::Baz` instead of full `crate::foo::bar::Baz`).
     - If an existing `use` already imports from the same module, extend it (e.g., `use crate::foo::{A};` → `use crate::foo::{A, B};`).
   - **For each blank line inside the top-of-file `use` block**: delete it. The block should run from the first `use` line to the last with no empty lines between. Do not reorder the imports — only remove the blanks. The single blank line that separates the entire `use` block from the code below it stays.

5. **Apply edits**: Add/update `use` statements in the import block, following the file's existing style. Replace qualified paths with short names.

6. **Verify**: Build the project (`cargo check`) to confirm no compilation errors. Wildcard removal is the most error-prone step — if some identifier was implicitly pulled in via the wildcard, the build will fail and reveal it; add it to the explicit list.

### Keep the qualifier that carries the meaning

Shortening serves readability, so it stops where readability does. Some last segments are generic — `run`, `new`, `build`, `check`, `parse`, `visit`, `Config`, `Error`, `Builder` — and say only what *kind* of thing the item is, leaving *which* one to the module name. For those, the module segment is the meaning: `foo_opt_path::run()` tells the reader which pass runs, while a bare `run()` at the call site identifies nothing.

The litmus test: reading the call site alone, does the short name identify what is being called? If it does, import it. If the module name is what identifies it, keep one module segment — the one that supplies the meaning — and import the module itself: `use crate::optimize::foo_opt_path;`, then call `foo_opt_path::run()`.

This exception applies to the qualifier that identifies the item. Any remaining leading segments (`crate::optimize::` above) still shorten away.

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

Scan the doc/inline comments touched by the diff — in Rust source, and the prose in hand-written Markdown docs. Rewrite whatever violates a convention below; the *Every Rust item must have a doc comment* convention is the one that *adds* a missing comment rather than rewriting. Each convention is tagged with where it applies — **[Rust]** for Rust comments only, **[Rust + Markdown]** for prose in both.

### Conventions

#### Don't enumerate callers — [Rust]

A doc comment should describe *what the thing is and how it behaves*, not *who calls it*. Lists like "used by handlers X, Y, Z", "the three callers share this result", or "called from foo() and bar()" rot fast — every new caller is a missed comment edit, and stale lists are worse than none.

**Rewrite**: drop the caller list. Let callers be discoverable by `grep` / IDE.

#### Don't enumerate None / error cases — [Rust]

For functions returning `Option<T>` or `Result<T, E>`, omit a trailing "Returns `None` when X, Y, Z" paragraph that lists routine failure cases. The return type already advertises that the call can fail; routine cases (input not found, position out of range, name not bound) aren't surprising.

**Keep** such notes only when a particular failure mode is *genuinely surprising* — a fail-fast condition the caller must guard against, an invariant the type signature can't express, or behavior diverging from what a reader would reasonably guess.

#### Describe inputs/outputs, not internal helpers — [Rust]

A comment should be readable without prior knowledge of internal helpers or sibling functions. If the comment says "projects X's `internal_helper` into Y" or "wraps `private_thing`", rewrite it to describe the inputs and outputs directly.

#### Don't narrate history / how-we-got-here — [Rust + Markdown]

Comments are read by people coming to the code fresh. They want to know *what the code does and why it works the way it does* — not the path that led the author here, and not a justification for the choice against alternatives the reader never saw. AI-written comments often slip in narratives like "originally we did X, but switched to Y", "now uses the new helper", "refactored from the old approach", "previously this returned a Map", or defensive asides excusing why this was done rather than that — these belong in commit messages and PR descriptions, not in the source.

**The litmus test** — apply it to every touched comment: *would this sentence be written if the prior conversation, the deliberation, and the previous version of the code did not exist?* A sentence that only makes sense as a reaction to that history fails; cut it.

**Rewrite**: drop the historical narrative. State the current behavior on its own terms.

**Keep** a "why" reference to history *only* when the implementation departs from what a reader would naturally expect, and the past explains the departure — e.g., "we don't use approach X here because of bug #1234" or "the obvious recursion is unrolled to avoid stack overflow on deeply nested input." The test: would a fresh reader, seeing the code, be surprised by the choice and benefit from knowing why?

#### Comments and prose must be in English — [Rust + Markdown]

This project's source comments are written in English (`Document.md`, `std.fix`, prior code, and PRs all assume English). A comment in any other language — Japanese, Chinese, etc. — is a style violation.

**Rewrite**: translate the comment into clear English while preserving its meaning.

#### Every Rust item must have a doc comment — [Rust]

The other conventions here mostly *rewrite* existing comments; this one *adds* the missing ones and governs what each should contain.

Every Rust item — `struct`, `enum`, `union`, their fields and variants, `trait`, trait method, free function, and `impl`-block method — must carry a `///` doc comment. **This applies to both `pub` and private items.**

The test for what a comment should contain, in one line: **leave out what the name and signature already make obvious, and put in the meaning they leave unsaid that a reader of this item would want to know.** Each rule below is that one test applied to a specific part of the comment.

Function comment shape:
- The first line describes what the function does. Don't restate the name (e.g., don't write `/// Adds two numbers.` for `fn add`).
- Add a `# Arguments` section *only* for arguments whose role isn't self-evident from the function's purpose — those that prompt the reader to ask "why is this argument needed?" Skip arguments whose role is obvious from name and type. When you do add an entry, state the argument's *meaning* — the part its name and type leave unsaid, such as units, indexing base and bounds, frame of reference, or which of two same-typed values (`from` / `to`) this is — not a restatement of the name. Format:
  ```rust
  /// Resolves the symbol at the given position.
  ///
  /// # Arguments
  /// * `prefer_definition` — when true, jump through re-exports to the original definition. Used by goto-def; references search wants this off.
  ```
  A bare restatement adds nothing: `the position` for `pos` says only what the name already says, where `0-indexed byte offset into the source string` states the meaning.
- Add a `# Returns` section only when the return value needs explanation beyond the type — and then say what the value *means*, such as what a `None`, an empty collection, or a particular variant signifies here. Keep this to the surprising cases; the *Don't enumerate None / error cases* convention still suppresses routine failures.

Test comment shape (for `#[test]` functions): the comment must state *what perspective the test exercises* — which behavior, edge case, or invariant it validates — not just "tests `foo`." Example: `/// Verifies that rename across an import boundary updates both the definition and the qualified callsite.`

**Excluded:**
- Pre-existing undocumented items in the same file, in `in-diff` mode — that mode covers the items the diff introduces or whose signature it modifies, and collects the file's other undocumented items as ring-2 candidates for the `neighborhood` pass to document.
- Fix sample programs under `src/tests/test_*/cases/` (these are `.fix` files; this aspect only walks `.rs` anyway).
- Items generated by `derive` macros or build scripts.

**Apply**: when a Rust item appears in the diff hunks (newly added, or its signature line was modified) and has no `///` directly above it, add a meaningful one-liner.
**Flag instead of writing boilerplate.** If you cannot articulate the item's purpose without restating its name, surface it as a flagged item rather than writing filler — the missing description may signal the item itself is redundant or poorly named.

#### Write in the affirmative — [Rust + Markdown]

State what *is*, not what *isn't*. Two anti-patterns to catch, both applying to Rust comments and Markdown prose alike:

**(a) Negating a rejected alternative in a definition or explanation.** Forms like "not A but B", "B rather than A", "A is wrong; B" name a rejected option, an alternative term, or a passing misunderstanding (A) only to knock it down. A reader who never had A in mind gains nothing from "it's not A" — it just makes them wonder what A was. Describe B directly.
- *Before*: `/// Not a deep copy — shares the backing buffer.`
- *After*: `/// Shares the backing buffer.`

**(b) Negating an unnecessary action in a procedure or guide.** Forms like "you don't need to do X" describe a non-action. State only what the reader must do.
- *Before*: `The caller does not need to lock the mutex first.`
- *After*: drop it, or if the locking discipline is load-bearing, `Acquires the mutex internally.`

**Keep** genuine prohibitions and deprecations — "must not be called after `close()`", "callers should not rely on the ordering here". These regulate future behavior rather than negating a phantom alternative, so they belong.

**Rewrite** (a) and (b) into the affirmative equivalent. When the negation carries no residual information once affirmed, drop the sentence.

#### Reference by name, not by line or section number — [Rust + Markdown]

Point at things by a name the reader can search for — a function, type, module, or a section's title — not by a line number or a numbered position ("line 214", "section 4.2", "the third bullet", "rule 5 above"). Line and section numbers drift the moment anything above them is edited, so the reference silently goes stale and points at the wrong place.

- *Before*: `// see the check on line 214` / `// as described in section 4.2`
- *After*: `// see resolve_symbol` / `// as described under "Scope Discipline"`

**Rewrite**: replace the numeric locator with the name of the thing it points to.

### Procedure

1. Run `git diff <base>` to find changed files and the touched line ranges. Two file kinds are in scope:
   - **Rust source** (`.rs`): all conventions apply.
   - **Hand-written Markdown docs** (`.md`) — e.g. `Document.md`, `README.md`, `CHANGELOG.md`, docs under `docs/`: only the **[Rust + Markdown]** conventions apply. **Exclude generated docs** under `std_doc/` (regenerated from source, so a hand edit would be overwritten).
2. For each changed `.rs` file, examine:
   - (a) comments that appear in the diff hunks (added or modified lines), for the rewriting conventions;
   - (b) Rust items defined or whose signature was modified in the diff hunks, for the *Every Rust item must have a doc comment* convention.

   In `neighborhood` mode, apply (a) and (b) to the rest of the file instead — its other comments and its undocumented items — under the radius rules. Every convention here edits prose alone, so all of them travel to ring 2.
3. For each changed hand-written `.md` file, examine the prose added or modified in the diff hunks for the **[Rust + Markdown]** conventions; in `neighborhood` mode, the document's other prose.
4. For each violation:
   - Identify which convention it is.
   - For a rewriting convention: propose a rewrite that preserves the intent but removes the anti-pattern, then apply with `Edit`.
   - For the doc-comment convention: write a meaningful one-liner above the item, or flag it for review if you cannot articulate the purpose without restating the name.
5. After all edits, run `cargo check` to confirm nothing broke (comment edits shouldn't affect builds, but verify in case of doctest changes). Markdown edits don't affect the build.
6. Report:
   - **Applied edits**: file, convention, brief rationale.
   - **Flagged for review** (the doc-comment convention only): file, item name, why no comment was written.

### Scope

- **Do not rewrite comments just because they're long.** Length is not the issue; the listed anti-patterns are.
- **Do not enforce conventions beyond the ones listed here.** Other style judgments (tone, capitalization) are not in scope.

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
   - If nothing was found, **say so explicitly** — the orchestrator treats a clean result as permission to proceed to the commit steps.

### Scope Discipline

- **Flag only; touch no files.** This is the one aspect that never edits.
- **Only added/modified lines are in scope.** Pre-existing personal data in untouched parts of a changed file is out of scope; this aspect guards against *new* leaks in the diff.
- **A single flag stops the review before any commit.** This aspect runs first, as a gate; the orchestrator commits nothing while a finding stands — that interlock is the point, so don't downgrade a genuine hit to a passing note.
