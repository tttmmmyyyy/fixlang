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

Where that language is English, write it plainly: a reader need not be a native speaker, and a long sentence or a rare word costs them more than it costs a native reader.

## Wording

The conventions of this section hold for every piece of prose in this repository: a dev doc, a pull request or issue body, a comment in the code, a hand-written Markdown document. The `code-review` skill's `comment-style` aspect applies them to comments and documents.

### Write in the affirmative

State what *is*, not what *isn't*. Two forms to catch:

**(a) Negating a rejected alternative in a definition or explanation.** Forms like "not A but B", "B rather than A", "A is wrong; B" — in a Japanese text 「A ではなく B」「A というより B」「A は誤りで B」 — name a rejected option, an alternative term, or a passing misunderstanding (A) only to knock it down. A reader who never had A in mind gains nothing from "it's not A"; it only makes them wonder what A was. Describe B directly.
- *Before*: `/// Not a deep copy — shares the backing buffer.`
- *After*: `/// Shares the backing buffer.`

**(b) Negating an unnecessary action in a procedure or guide.** Forms like "you don't need to do X" describe a non-action. State only what the reader must do.
- *Before*: `The caller does not need to lock the mutex first.`
- *After*: drop it, or if the locking discipline is load-bearing, `Acquires the mutex internally.`

**Keep** genuine prohibitions and deprecations — "must not be called after `close()`", "callers should not rely on the ordering here". These regulate future behavior rather than negating a phantom alternative, so they belong.

**Rewrite** (a) and (b) into the affirmative equivalent. When the negation carries no residual information once affirmed, drop the sentence.

### Name a thing the way the reader's own code names it

An identifier the reader's code carries is vocabulary they already share with the text, and a word they can search for. A text addressed to the people working on the compiler shares the compiler's names: write `FullName`, not "the full name"; `CTypeShape`, not "the shape a C declaration carries". A paraphrase makes a second name for one thing and cuts the way back to the code, and a translated paraphrase cuts it twice.

**Which code that is follows from who reads the text.** A Fix programmer reading the manual or `CHANGELOG.md` shares Fix's names — `FFI_EXPORT`, `I8`, `Std::Array::get_size` — and shares nothing with the compiler's source, so a compiler identifier there names something they cannot reach. Say what they can see instead.

Where the reader's code names nothing, say what the thing is rather than inventing a word for it. A word coined to carry a concept through a few paragraphs — a "position" for a parameter or a result, a "slot", a "unit" — means nothing to a reader who meets it once and has nowhere to look it up. The test: can the sentence be written without the coined word? It usually can, and is shorter for it.

**Rewrite**: put the reader's identifier where a paraphrase of it stands, and what it stands for where a coined word stands.

### Reference by name, not by line or section number

Point at things by a name the reader can search for — a function, type, module, or a section's title — not by a line number or a numbered position ("line 214", "section 4.2", "the third bullet", "rule 5 above"). Line and section numbers drift the moment anything above them is edited, so the reference silently goes stale and points at the wrong place.

- *Before*: `// see the check on line 214` / `// as described in section 4.2`
- *After*: `// see resolve_symbol` / `// as described under "Scope Discipline"`

**Rewrite**: replace the numeric locator with the name of the thing it points to.

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

A declaration says what a function accepts and returns; what it computes is left to the prose. Add one input and the output it produces when the pair would carry the reader to the behavior faster than that prose does. The prose does not have to be incomplete for that to hold: a concrete pair is read at a glance, where a rule has to be applied before it answers anything. So the question is which one the reader gets there on, not whether the prose is sufficient. Head the line with **Examples**, so the reader sees at a glance that what follows is a case rather than the rule:

**Examples**

```
split(",", "a,b")   ->  ["a", "b"]
split(",", "a,,b")  ->  ["a", "", "b"]
```

The gap an example closes is usually one of these:

- **A transformation** leaves open what the output is made of: which parts of the input survive, in what order, and what an empty or repeated part becomes.
- **A predicate** leaves open where the line falls: the reflexive case, the empty case, and — when two arguments share a type — which of the two the question is about.
- **An encoding, a generated name, a formatted string** leaves open the shape of the result, which is the whole content of the function.
- **Several rules the reader has to add up**: each is stated on its own, and the answer is what they come to together.

Choose the input a reader would meet first — the ordinary call rather than a boundary. The example is what the reader carries away from the paragraph, so it has to show the function doing its usual work. Add a second only where the ordinary one leaves a boundary open: what an empty or repeated part becomes, which side of the line an endpoint falls on.

Keep it to what a glance holds — a line or two. An example needing a paragraph of setup says the prose should carry the behavior instead.

Where the name and the type already carry the behavior — reading a field, replacing a field, constructing a value, performing an effect — state the meaning they leave unsaid and stop there.
