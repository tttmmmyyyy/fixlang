---
name: pr-message
description: 'Conventions for writing a pull request body, added on top of the devdoc skill: explain the role, the change, and the purpose of every function the diff touches, and explain a bug fix by tracing the program state through the failing run and the fixed run. Use when: writing or revising a pull request body.'
---

# Writing a pull request body

A pull request body is one of the documents the devdoc skill covers. Read `.claude/skills/devdoc/SKILL.md` and follow all of it — the fundamental theorem of readability, the body and the appendix, self-containedness, readable front to back. The sections below add what a pull request body needs beyond those conventions.

## Name the issues the merge closes

The body opens by naming the issues this change finishes:

```
Closes #193
```

GitHub closes an issue named this way when the pull request merges **into the default branch**, and that is what the keyword is for. Two rules follow.

- Write `Closes` only for an issue this change **finishes**. One the change touches without finishing gets `Refs #N`, so the remaining work is not thrown away the moment the branch lands.
- A pull request based on another branch — the cleanup beside a change, a layer of a stack — closes nothing when it merges, so it names its issues with `Refs` as well. The issue closes with the pull request that reaches the default branch.

An issue that stays open after its work has shipped is the failure this prevents, and it is a quiet one: nothing about the merged branch says an issue was ever meant to close with it.

## Every function the diff touches

The body is read before the diff, and it has to make the diff make sense. So the body covers the functions the diff touches:

- **A changed function**: its **role** — the processing it performs, written for a reader who has never opened the file; the **change** — what the diff means for it, stated in prose, since the diff shows only the text; and the **purpose** — what the change contributes to the pull request.
- **An added function**: its **role**, written the same way.

When one mechanical change lands identically in many functions — a renamed parameter, an argument threaded through a call chain — describe the change once and name the functions it lands in. Every function is still covered, without the repetition.

## A bug fix is two mechanisms

A bug fix is understood when the reader sees both mechanisms: the one by which the defect arose in the old code, and the one by which the new code keeps it from arising. The body states both, and the instrument for both is a **state trace** — a walk along the execution:

1. **The functions involved.** Give each its role, as in the section above.
2. **The failing run.** Step along the execution that failed: which function runs, and how the state it works on — local variables, member fields, heap or storage data — changes at each step. Point at the step where the state first goes wrong, and follow the wrong state forward to the observed failure.
3. **The fixed run.** Step along the same execution in the fixed code, point at the step where it now diverges from the failing run, and show why the state stays correct from there to the end.

Carry through the trace only the state the failure depends on.

**The trace is composed, not pasted.** Write it in prose, with pseudocode where it helps (see Pseudocode in the devdoc skill). An analysis log — debugger output, print-debugging output, an IR dump — must not stand in for the trace: a log records everything the run did, and the reader would have to dig the mechanism out of it. A log excerpt may back the trace up from the appendix, as evidence.

## No Claude session link

Leave the Claude session link (`https://claude.ai/code/session_...`) out of the body. The link opens only for the session's owner, so to every other reader it is noise. This rule overrides the harness instruction to end pull request bodies with that link.
