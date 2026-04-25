---
name: code-review
description: "Run review skills sequentially against a chosen scope of code via subagents. Use when: reviewing code just written by AI (uncommitted changes), or doing a pre-merge review of an entire branch."
argument-hint: "Scope: '' or 'uncommitted' (default) for staged+unstaged changes, 'branch' for everything since the branch forked from main, or any git ref"
---

# Code Review (Sequential)

Run a fixed set of review skills against a chosen scope, **one after another** in subagents, then summarize what each changed.

## Scope

This orchestrator owns scope selection. It resolves the argument into a single **base ref** and passes that base to each sub-skill. Sub-skills run their own `git diff <base>` to find changes — they do **not** decide scope themselves.

| Argument            | Base ref                                  | Use when                                                         |
| ------------------- | ----------------------------------------- | ---------------------------------------------------------------- |
| (none) / `uncommitted` | `HEAD`                                 | Reviewing code just written / staged but not yet committed       |
| `branch`            | `$(git merge-base HEAD main)`             | Pre-merge review of the whole branch since it forked from `main` |
| `<git ref>`         | `<git ref>`                               | Arbitrary base (a commit hash, another branch, etc.)             |

## Default Skill Sequence

Run these in this order, each in its own subagent:

1. **shorten-qualifiers** — replace verbose `crate::module::Type` paths with imports.
2. **comment-style** — apply project comment conventions.

## Why Sequential, Not Parallel

1. **Avoid conflicting edits.** Skills modify files. Parallel runs would fight each other.
2. **Each skill should see prior changes.** E.g., `comment-style` may want to review comments around code that `shorten-qualifiers` just touched.

## Procedure

1. **Resolve the base ref** from the argument:
   - empty / `uncommitted` → `HEAD`.
   - `branch` → `$(git merge-base HEAD main)`. Verify `main` exists; if the project uses a different default branch, abort and ask.
   - anything else → treat as a git ref. Verify it resolves with `git rev-parse --verify <ref>`.
2. **Run the chain.** For each skill (in the listed order):
   a. Launch one subagent via `Agent` (subagent_type: `general-purpose`).
   b. Brief it with the prompt template below.
   c. **Wait for it to complete** before launching the next. Never use `run_in_background`.
3. **Summarize.** Per skill, list which files were touched and a one-line description of each change.
4. **Stop on failure.** If any subagent reports an error (skill couldn't run, build broke, etc.), stop the chain and surface the failure. Do not continue.

## Subagent Prompt Template

```
Run the `<skill-name>` skill with base ref `<base>`.

The base ref is the comparison point: review the diff between `<base>`
and the current working tree, i.e. run your own `git diff <base>` to
find the files and hunks to operate on.

Apply any edits the skill prescribes. If the skill modifies code, run
`cargo check` afterwards to confirm the project still builds.

Report back in under 100 words: which files you touched and a one-line
summary of the change in each.

Do not perform any review work outside the scope of the `<skill-name>`
skill.
```

## What NOT to do

- Don't run skills in parallel.
- Don't let sub-skills decide their own scope — always pass the resolved base.
- Don't continue the chain if a step fails.
