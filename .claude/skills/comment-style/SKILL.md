---
name: comment-style
description: "Review and rewrite Rust doc/inline comments to follow project comment conventions: don't enumerate callers, don't list every None/error case, describe inputs/outputs cleanly. Use when: reviewing comments before commit, or as a stage of code-review."
argument-hint: "Base git ref to diff against; defaults to HEAD"
---

# Comment Style Review

Scan the doc/inline comments touched by the diff against a given base ref, and rewrite ones that violate the project conventions below.

The base ref is provided by the caller (e.g., the `code-review` orchestrator). When run directly without an argument, default to `HEAD` (i.e., review uncommitted changes only).

## When to Use

- Reviewing comments added or changed in some range of work.
- After writing code with new comments, before commit.
- As one stage of `code-review`.

## Conventions

### 1. Don't enumerate callers

A doc comment should describe *what the thing is and how it behaves*, not *who calls it*. Lists like "used by handlers X, Y, Z", "the three callers share this result", or "called from foo() and bar()" rot fast — every new caller is a missed comment edit, and stale lists are worse than none.

**Rewrite**: drop the caller list. Let callers be discoverable by `grep` / IDE.

### 2. Don't enumerate None / error cases

For functions returning `Option<T>` or `Result<T, E>`, omit a trailing "Returns `None` when X, Y, Z" paragraph that lists routine failure cases. The return type already advertises that the call can fail; routine cases (input not found, position out of range, name not bound) aren't surprising.

**Keep** such notes only when a particular failure mode is *genuinely surprising* — a fail-fast condition the caller must guard against, an invariant the type signature can't express, or behavior diverging from what a reader would reasonably guess.

### 3. Describe inputs/outputs, not internal helpers

A comment should be readable without prior knowledge of internal helpers or sibling functions. If the comment says "projects X's `internal_helper` into Y" or "wraps `private_thing`", rewrite it to describe the inputs and outputs directly.

### 4. Don't narrate history / how-we-got-here

Comments are read by people coming to the code fresh. They want to know *what the code does and why it works the way it does*, not the path that led the author here. AI-written comments often slip in narratives like "originally we did X, but switched to Y", "now uses the new helper", "refactored from the old approach", "previously this returned a Map" — these belong in commit messages and PR descriptions, not in the source.

**Rewrite**: drop the historical narrative. State the current behavior on its own terms.

**Keep** a "why" reference to history *only* when the implementation departs from what a reader would naturally expect, and the past explains the departure — e.g., "we don't use approach X here because of bug #1234" or "the obvious recursion is unrolled to avoid stack overflow on deeply nested input." The test: would a fresh reader, seeing the code, be surprised by the choice and benefit from knowing why?

### 5. Comments must be in English

This project's source comments are written in English (`Document.md`, `std.fix`, prior code, and PRs all assume English). A comment in any other language — Japanese, Chinese, etc. — is a style violation.

**Rewrite**: translate the comment into clear English while preserving its meaning.

## Procedure

1. Run `git diff <base>` to find changed Rust files and the touched line ranges (`<base>` = the argument, defaulting to `HEAD`).
2. For each changed `.rs` file, examine the comments that appear in the diff hunks (added or modified lines).
3. For each comment violating a convention:
   - Identify which convention (1 / 2 / 3 / 4 / 5).
   - Propose a rewrite that preserves the intent but removes the anti-pattern.
   - Apply the edit with the `Edit` tool.
4. After all edits, run `cargo check` to confirm nothing broke (comment edits shouldn't affect builds, but verify in case of doctest changes).
5. Report what was changed: file, conventions violated, brief rationale.

## Scope

- **Touch only comments inside the current diff hunks.** Don't comb through the entire codebase for violations — that is out of scope and would create a giant unrelated change.
- **Do not rewrite comments just because they're long.** Length is not the issue; the listed anti-patterns are.
- **Do not enforce conventions beyond the five above.** Other style judgments (tone, capitalization, line wrapping) are not in scope.
