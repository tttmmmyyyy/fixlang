---
name: comment-style
description: "Review Rust doc/inline comments to follow project comment conventions: don't enumerate callers, don't list every None/error case, describe inputs/outputs cleanly, English only, no historical narration. Also require doc comments on every Rust item (type, field, variant, trait, function, method) touched by the diff. Use when: reviewing comments before commit, or as a stage of code-review."
argument-hint: "Base git ref to diff against; defaults to HEAD"
---

# Comment Style Review

Scan the doc/inline comments touched by the diff against a given base ref. Rewrite ones that violate conventions 1–5, and add missing doc comments per convention 6.

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

### 6. Every Rust item must have a doc comment

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
- Fix sample programs under `src/tests/test_*/cases/` (these are `.fix` files; comment-style only walks `.rs` anyway).
- Items generated by `derive` macros or build scripts.

**Apply**: when a Rust item appears in the diff hunks (newly added, or its signature line was modified) and has no `///` directly above it, add a meaningful one-liner.
**Flag instead of writing boilerplate.** If you cannot articulate the item's purpose without restating its name, surface it as a flagged item rather than writing filler — the missing description may signal the item itself is redundant or poorly named.

## Procedure

1. Run `git diff <base>` to find changed Rust files and the touched line ranges (`<base>` = the argument, defaulting to `HEAD`).
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

## Scope

- **Touch only comments and items inside the current diff hunks.** Don't comb through the entire codebase for violations — that is out of scope and would create a giant unrelated change. In particular, convention 6 does *not* require backfilling doc comments on pre-existing undocumented items in the same file.
- **Do not rewrite comments just because they're long.** Length is not the issue; the listed anti-patterns are.
- **Do not enforce conventions beyond the six above.** Other style judgments (tone, capitalization, line wrapping) are not in scope.
