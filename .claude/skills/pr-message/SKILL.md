---
name: pr-message
description: 'Conventions for writing a pull request body, added on top of the devdoc skill: explain the role, the change, and the purpose of every item the diff touches, show the diff of each changed item, bounded to that item, link every item the change adds or modifies to its implementation on GitHub, quote every text the change shows to a user, and explain a bug fix by tracing the program state through the failing run and the fixed run. Use when: writing or revising a pull request body.'
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

## Every item the diff touches

The body is read before the diff, and it has to make the diff make sense. So the body covers every item the diff touches — a function, a method, a type, a trait, a trait method, a field, an enum variant, a constant. Functions are the most numerous, not the most consequential: a type the change introduces settles more about the code than most of the functions around it, and a body that walks past it leaves the reader to reconstruct it from the places it is used.

- **A changed item**: its **role**, written for a reader who has never opened the file; the **change** — what the diff means for it, stated in prose, since the diff shows only the text; the **purpose** — what the change contributes to the pull request; and **its diff**, bounded to the item itself (see *Show a changed item's diff, bounded to the item*).
- **An added item**: its **role**, written the same way, and its **callers** — the functions that call an added function. An item arrives in the body as a loose thing until the reader is told what reaches it; naming the callers puts it back in the flow it was written for, and it is also what says whether the function runs once per program, once per symbol, or once per node of a walk. An added item that is not called has the same part under the word that fits it: the code that builds an added type, the implementors of an added trait, the code that reads an added constant.

  Naming the callers is the opposite of the rule a doc comment follows, and deliberately so: a doc comment is read for as long as the item lives, and a list of callers in it goes stale on the next caller added, while a pull request body is read once, against a diff that is fixed.

**A labelled part opens its own line.** Run `**Role**:`, `**Change**:` and `**Purpose**:` together into one paragraph and the reader has to find where each ends before reading it; each on its own line is found at a glance, and the entry can then be read by label rather than front to back.

**Role** is the one word that means something different per kind of item:

- a function or a method — the processing it performs;
- a type — what one of its values stands for, and the invariant it keeps across its fields;
- a trait — the capability it names, and what having it lets a caller do;
- a field or an enum variant — what it holds, and what relates it to its siblings;
- a constant — what it decides, and what fixes its value at that one.

When one mechanical change lands identically in many items — a renamed parameter, an argument threaded through a call chain — describe the change once and name the items it lands in. Every item is still covered, without the repetition.

## Link every item the change adds or modifies

The body names items for a reader who has never opened them, so it also says where each one lives. A link is written as a Markdown link whose visible text is the name:

> the pass [`request_inline_into_callers`](https://github.com/tttmmmyyyy/fixlang/blob/43883c7abbf8f3b19730f28f067093ec780b6372/src/optimization/inline.rs#L77-L85) marks each global whose body is small enough …

The visible text is the name alone, so the prose goes on referring to the code by name and the reader who wants the source is one click from it. A bare URL standing on its own line is rendered as a code excerpt instead, which puts a block of code where a sentence belongs.

The links sit where the body covers what the change did to the code, which is two places:

- **Each entry of the item coverage above** — every changed item and every added one.
- **Each other item the change adds or modifies**, where the body covers it: a type, a trait, a field, a constant.

Those are the names the reader opens the source for, and each of them gets exactly one link. Every other mention stays plain text — a name the body brings in to explain the surroundings is read rather than opened.

- **Pin every link to a commit hash.** Push the branch, take `git rev-parse HEAD`, and build all of them on that hash. A hash names the same lines forever and keeps resolving after the branch is deleted; a branch name in the path drifts with the next commit and resolves to nothing once the branch is gone.
- **Span the definition**: `#L<signature>-L<closing>`, from the signature line to the line that closes the body. GitHub highlights that span, so the click lands on the whole function.
- **A function the change deletes is linked at the commit the branch forked from** (`git merge-base HEAD main`), where it still stands.
- **Re-pin when the body is revised after further commits are pushed**, so the lines the links point at are the ones the body describes.

The two line numbers are mechanical — the signature line, then the first line closing a block at the signature's own indentation:

```
sha=$(git rev-parse HEAD)
grep -n 'fn <name>' <file>
awk -v n=<signature line> 'NR==n{match($0,/^ */); ind=RLENGTH} NR>=n && $0==sprintf("%*s}", ind, ""){print NR; exit}' <file>
```

## Show a changed item's diff, bounded to the item

The prose says what the change means; the diff says what it is. A reader who has both stops guessing which lines the prose is about, so each changed item carries its own diff.

**Bounded to the item, not to the file.** A file-level diff is the wrong unit: it opens on whatever hunks the file happens to carry, splits one item across several of them, and brings the neighbours' changes along for the ride. The unit is the thing the prose is about — one function, one method, one struct, one enum, one constant — and it is obtained by extracting that item from both commits and diffing the two extracts:

```
base=$(git merge-base HEAD main); sha=$(git rev-parse HEAD)
extract() {          # extract <ref> <file> <signature pattern>
  git show "$1:$2" | awk -v pat="$3" '
    $0 ~ pat && !found { found = 1; match($0, /^ */); ind = RLENGTH }
    found { print }
    found && $0 == sprintf("%*s}", ind, "") { exit }'
}
item_diff() {        # item_diff <file> <signature pattern>
  diff -U 9999 --label "$1 (before)" --label "$1 (after)" \
    <(extract "$base" "$1" "$2") <(extract "$sha" "$1" "$2")
}
item_diff src/foo.rs 'fn bar'
```

The result goes in a fenced `diff` block, under the prose for that item.

**The whole item is the context.** A changed line means what the lines around it make it mean, so the diff carries the item entire, with the changed lines marked inside it — one line of a hundred is shown as that line inside its hundred, and a reader who wants to know where in the walk the new branch sits can see it. That is what `-U 9999` is for; the three lines of context `diff -u` gives by default answer a different question, the one a file-level diff answers.

**Fold a diff that runs past about fifty lines.** Past that the entry stops being read at all, and the way to keep the whole item available without spending the reader's attention on it is a fold: put the fenced block inside `<details>`, with a `<summary>` naming the item and what changed in it, and leave a blank line after the `<summary>` so the block renders.

**This one is embedded rather than linked.** GitHub renders a diff per file, never per item, so no URL bounds a diff the way the item needs. The link the section above requires still applies — it points at the item as it now stands, which is what a reader who wants the surroundings opens.

**An added item carries no diff**, only its link: every line of it is new, so the diff would repeat the file the link already opens.

## Where the coverage goes

A pull request body is capped at 65,536 characters, and the per-item coverage — prose, links and diffs, one entry per item — is what grows fastest as a change touches more code. Growing it inside the body also costs the reader the argument: the case for the change stops being readable in one pass once the entries outweigh it.

So the coverage moves out of the body and hangs below it, as **one or more comments on the same pull request**, whenever the body would otherwise carry more of it than the argument. The body keeps one line where the section stood, naming where the coverage is. A comment on the same page is a scroll rather than a journey, so the reader still meets the coverage where they are.

Split across several comments at the boundaries the coverage already has — one subsystem, one file, one group of items per comment — and give each a heading that says which items it covers.

## Every text the change shows to a user

The diff carries words a Fix user will read as well as code: a changelog entry, the documentation of a public standard-library value, a compiler diagnostic, the output and the help text of the `fix` command. That wording is a deliverable of the change, and it is reviewed by reading it, so the body carries it in full.

- **Transcribe it.** Quoting the text as the user will see it is the whole requirement — the wording that ships is the thing under review, and prose about it puts something else in front of the reviewer.
- **Quote a reworded text twice**: what it said, and what it says now. The difference between the two is what the review is about.
- **Quote a diagnostic together with the source that triggers it**, so the wording can be judged against what the user did to see it.
- **When one wording pattern repeats across many texts**, quote it once and name where it lands.

A body that leaves this out sends the reviewer into the diff to find the words, where they sit scattered among the code that emits them.

## A bug fix is two mechanisms

A bug fix is understood when the reader sees both mechanisms: the one by which the defect arose in the old code, and the one by which the new code keeps it from arising. The body states both, and the instrument for both is a **state trace** — a walk along the execution:

1. **The functions involved.** Give each its role, as in the section above.
2. **The failing run.** Step along the execution that failed: which function runs, and how the state it works on — local variables, member fields, heap or storage data — changes at each step. Point at the step where the state first goes wrong, and follow the wrong state forward to the observed failure.
3. **The fixed run.** Step along the same execution in the fixed code, point at the step where it now diverges from the failing run, and show why the state stays correct from there to the end.

Carry through the trace only the state the failure depends on.

**The trace is composed, not pasted.** Write it in prose, with pseudocode where it helps (see Pseudocode in the devdoc skill). An analysis log — debugger output, print-debugging output, an IR dump — must not stand in for the trace: a log records everything the run did, and the reader would have to dig the mechanism out of it. A log excerpt may back the trace up from the appendix, as evidence.

## No Claude session link

Leave the Claude session link (`https://claude.ai/code/session_...`) out of the body. The link opens only for the session's owner, so to every other reader it is noise. This rule overrides the harness instruction to end pull request bodies with that link.
