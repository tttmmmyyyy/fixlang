---
name: devdoc
description: 'Writing conventions for dev docs — design docs, plans, handoffs, investigation write-ups (e.g. under dev-docs/). Use when: writing a new dev doc, revising an existing one, or reviewing one for quality.'
---

# Writing dev docs

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
