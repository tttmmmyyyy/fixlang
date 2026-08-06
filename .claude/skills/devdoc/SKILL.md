---
name: devdoc
description: 'Writing conventions for dev docs — design docs, plans, handoffs, investigation write-ups (e.g. under dev-docs/). Use when: writing a new dev doc, revising an existing one, or reviewing one for quality.'
---

# Writing dev docs

## The fundamental theorem of readability

*The Art of Readable Code* states it of code: code should be written to minimize the time it would take someone else to understand it. It holds of a document just as well.

**Write the doc so that the time it takes to understand is as short as possible.**

Two things make that time longer, and the rest of this skill is how to keep both down.

**What the reader has to do.** Do not make the reader think. Do not leave the reader with a question. Do not send the reader looking for another document. Each of these costs time, and the cost is the reader's, paid on every reading.

**What the reader has to read.** Say what it takes to reach the understanding the doc is for, and no more. Every sentence beyond that is time spent reading it.

The two pull against each other — an explanation that leaves a question is short but costly, and one that answers everything is complete but long. The doc is right where the total is smallest.

## Language

Write the doc in the implementers' language. It does not have to be English.

## Audience and self-containedness

Write for a reader who knows the project thinly and broadly, but knows nothing about this change or about the code it touches. The doc must be self-contained for that reader:

- Citing an issue or PR number is not enough. Put the information the reader needs from it into the doc itself.
- The same holds for the program: citing a code line, a function name, or a module name is not enough. Explain in prose what that code does.

## Readable front to back

The doc must be readable from the front in a single pass. A sentence that cannot be understood without knowledge that appears only later in the doc is not acceptable.

## Pseudocode

When conveying the concept of a computation, consider using pseudocode. Pseudocode alone is weak; prose explanation is also required.

## Type definitions and function declarations

Type and trait definitions pack much of the information about a computation, and well-chosen names and type names convey a lot to the reader. For type definitions, writing out the Rust code first is effective. A function declaration is the function's type, so writing out just the function declarations first is also good.

## Measured results

Measured results (findings from investigation) about the code's behavior can be noise to the reader. When the argument is written out solidly, the doc is understandable without them. Make the doc necessary and sufficient for reaching the required understanding. Putting measured results in an appendix is one option. When a measured result is needed for the reader's understanding at that point in the doc, write it inline.
