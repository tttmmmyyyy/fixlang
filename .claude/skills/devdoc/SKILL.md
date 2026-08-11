---
name: devdoc
description: 'Writing conventions for the prose that explains a change — design docs, plans, handoffs, investigation write-ups (e.g. under dev-docs/), pull request bodies, issue bodies. Use when: writing or revising any of these, or reviewing one for quality.'
---

# Writing dev docs, pull requests and issues

## What this covers

A dev doc under `dev-docs/`, a pull request body, and an issue body. All three are addressed to the people working on the change and are read the same way, so all three follow the conventions below. A pull request body and the dev doc of the change it carries are often the same text. A pull request body has further requirements on top of these conventions; the pr-message skill states them, so apply both when writing one.

## The fundamental theorem of readability

*The Art of Readable Code* states it of code: code should be written to minimize the time it would take someone else to understand it. It holds of a document just as well.

**Write the doc so that the time it takes to understand is as short as possible.**

Two things make that time longer, and the rest of this skill is how to keep both down.

**What the reader has to do.** Do not make the reader think. Do not leave the reader with a question. Do not send the reader looking for another document. Each of these costs time, and the cost is the reader's, paid on every reading.

**What the reader has to read.** Say what it takes to reach the understanding the doc is for, and no more. Every sentence beyond that is time spent reading it.

The two pull against each other — an explanation that leaves a question is short but costly, and one that answers everything is complete but long. The doc is right where the total is smallest.

## The body and the appendix

The body is what the reader must read to understand. The appendix comes after it and holds everything else the doc is worth carrying; the body is complete without it.

That split is what makes the appendix cheap. A sentence in the body costs every reader time, so the body has to be necessary and sufficient. A sentence in the appendix costs only the reader who goes looking for it, so the appendix can be as long as the material deserves. The question to ask of a piece of writing is therefore not "is this worth keeping?" but "does understanding require it?" — and what fails that question is moved rather than deleted.

Into the appendix:

- **Evidence.** Measurements, benchmark numbers, the output of a run, the experiment that confirmed a claim. The body states the claim and the argument for it; the appendix holds what was run and what came back.
- **Investigation results.** What was searched, what was found, which reading turned out to be wrong.
- **History and work records.** The alternatives that were tried, why one was dropped, what the earlier version did.

Write a measurement into the body instead when the argument at that point does not hold without it — and then only the part the argument uses.

## Language

Write the doc in the implementers' language. It does not have to be English.

## Audience and self-containedness

Write for a reader who knows the project thinly and broadly, but knows nothing about this change or about the code it touches. The doc must be self-contained for that reader:

- Citing an issue or PR number is not enough. Put the information the reader needs from it into the doc itself.
- The same holds for the program: citing a code line, a function name, or a module name is not enough. Explain in prose what that code does.

## Readable front to back

The body must be readable from the front in a single pass. A sentence that cannot be understood without knowledge that appears only later is not acceptable. The body may point into the appendix, since the reader who skips it still understands.

## Pseudocode

When conveying the concept of a computation, consider using pseudocode. Pseudocode alone is weak; prose explanation is also required.

## Type definitions and function declarations

Type and trait definitions pack much of the information about a computation, and well-chosen names and type names convey a lot to the reader. For type definitions, writing out the Rust code first is effective. A function declaration is the function's type, so writing out just the function declarations first is also good.

## An example of input and output

A declaration says what a function accepts and returns; what it computes is left to the prose. For a transformation — a function that takes a value apart and builds a different one — one input and the output it produces says it in a line:

```
split(",", "a,,b")  ->  ["a", "", "b"]
```

Choose the input that answers what the reader would otherwise have to ask: an empty collection, a boundary, an element the transformation drops.

Where the name and the type already carry the behavior — reading a field, replacing a field, constructing a value, performing an effect — state the meaning they leave unsaid and stop there.
