---
name: soundness-proof
description: "Prove that a piece of code is sound — that inputs satisfying a stated precondition produce outputs satisfying a stated postcondition — and write the proof as a dev doc: definitions, then a sequence of numbered propositions whose every leaf step cites exactly the facts, definitions and code it rests on, in Lamport's structured-proof style. The orchestrator fixes the target commit, states the property and calibrates it against a bug the code actually had, then alternates prover subagents that write the proof with verifier subagents that check one step at a time and report every step that is false, non-obvious, hedged, or built on an undefined word. Use when: asked to prove that a pass, a function, or a subsystem is sound or correct; or to re-verify an existing proof after the code changed."
argument-hint: "The processing to prove sound (a pass, a function, a module), and optionally the property to prove. If omitted, the skill asks."
---

# Soundness proof

The deliverable is a document: **definitions, then a sequence of propositions, each with a proof**, ending in the theorem that the target is sound. It is written for a reader who checks it rather than one who is persuaded by it, so every inference step must be obvious from the items it cites alone.

The work is done by two kinds of subagent under this orchestrator: **provers**, who write proofs, and **verifiers**, who check one step at a time and are forbidden to think. The document is finished when a fresh round of verifiers returns nothing.

This skill **never modifies the code under proof**. A step that cannot be proved because the code does not do what it should is a bug, and it is reported the way `bug-hunt` reports one — a failing scenario, a reproduction, a fix proposal for the author to weigh. Fixing it is a separate change.

## What this is for, and what it is not

`bug-hunt` searches: it finds what it happens to look at, and its silence means only that this wave looked elsewhere. A proof covers every input by construction, so it turns "no bug found" into "no bug of this kind exists". That is the whole reason to pay for it — and it is also why a proof of a property too weak to be violated by real bugs is worse than no proof, since it buys confidence it did not earn. Hence the calibration step below.

## The proof language

Write the proof in **Lamport's structured-proof style** (Leslie Lamport, *How to Write a 21st Century Proof*, 2012), in prose, using the notation below. It is not a formal language and nothing machine-checks it; what it buys is that **each step names the facts it uses**, so a checker can verify one step by reading that step, the items it cites, and nothing else. That property is what makes the verifier's job mechanical, and it is the reason for choosing this style over ordinary mathematical prose.

Nested equational chains (see *Calculational steps*) come from the Dijkstra calculational style as extended by Back and von Wright's *structured derivations*; they are the one place where the argument may be laid out as a chain rather than as steps.

### Steps

A proof is a numbered list of steps. The last step of every list is `QED`, whose statement is the goal of the proof it closes.

```
<1>1. STATEMENT
  <2>1. STATEMENT
    BY D3, <1>0
  <2>2. QED
    BY <2>1, P4
<1>2. QED
  BY <1>1, A1
```

`<k>n` is step `n` of a level-`k` proof. A step's proof is either a nested list of `<k+1>` steps ending in `QED`, or a single `BY` line when it needs no decomposition. Every step has one or the other: a step with neither is an unproved assertion, and the verifier reports it.

### `BY` — the citation

```
BY <2>1, <1>3, P4, A1, D2 DEF unit_key CODE `src/rc_ir/borrow.rs: CancelAnalysis::un_bump`
```

A `BY` line lists **everything** the step rests on, in these four groups (omit an empty group):

- **Facts**: steps in scope (`<k>n`), propositions (`P<n>`), assumptions (`A<n>`), definitions used as facts (`D<n>`).
- **`DEF`**: the definitions whose bodies are unfolded in this step. Citing `D2` as a fact means using what it states; citing it under `DEF` means substituting its text.
- **`CODE`**: the source this step reads, as *file*: *symbol path* — a function, a method, a type, a match arm identified by its pattern. Never a line number: line numbers move, and a citation the reader cannot resolve after one edit is a citation nobody checks.
- **External results**: a named theorem from outside the document, given its full statement in the definitions section.

The rule behind the format: **a reader who is handed only the cited items must reach the step's conclusion**. Anything they would additionally need is missing from the `BY`.

### `ASSUME` / `PROVE`

A proposition about "any input satisfying …" is stated as an assume/prove, which is also the natural shape of a function's contract:

```
P3. ASSUME  NEW p: RcProgram,
            p is well-formed (D1),
            every retain in p is balanced (D4)
    PROVE   cancel(p) is well-formed (D1)
            and every retain in cancel(p) is balanced (D4)
```

`NEW x` introduces a variable and asserts the conclusion for every value of it. The assumptions of an assume/prove hold **inside its proof only**.

A step may itself be an assume/prove; its assumptions then hold only inside that step's proof, and this is what the scoping rule below is protecting.

### `SUFFICES`, `CASE`, `PICK`, `DEFINE`

- `SUFFICES <statement>` — proving the statement proves the current goal. The step's own proof shows that implication; the rest of the proof then works on the new goal. Use it to strip a quantifier or set up a contradiction without adding a level.
- `CASE <condition>` — abbreviates "assume the condition, prove the current goal". A case split is a run of `CASE` steps followed by `QED`, and **the `QED` must cite the cases and show they are exhaustive**. A case split over a Rust `match` cites the enum's definition for exhaustiveness; one over a condition cites the excluded middle. A split that silently omits an arm is the most common way a proof of a compiler pass goes wrong, so the verifier checks exhaustiveness by name against the type.
- `PICK x SUCH THAT P(x)` — introduces `x` and asserts `P(x)`; the step's proof shows such an `x` exists.
- `DEFINE name == expression` — names an expression for the rest of the current level. A definition local to a proof, as opposed to a `D<n>` in the definitions section, which is global.

### Hierarchical numbering, and what a step may cite

From inside the proof of step `<3>4`, the citable facts are exactly:

- every `D<n>`, `A<n>`, and every `P<n>` that precedes this proposition in the dependency order;
- the strictly preceding siblings at each enclosing level: `<1>1` .. `<1>(i-1)`, `<2>1` .. `<2>(j-1)`, `<3>1` .. `<3>3`, where `<1>i` and `<2>j` are the ancestors of this step.

Nothing else. Not a step inside a sibling's proof, not a later step, not a step of another proposition. The reason is not tidiness: a step inside a sibling's proof may have been proved under that sibling's assume/prove assumptions, which do not hold here, so citing it is unsound. Because the rule is syntactic, the verifier applies it without understanding the argument.

When two proofs need the same fact, it is lifted out — into a preceding sibling step at the level where both can see it, or into its own `P<n>`.

### Inserting a step

The verifier's findings are answered by inserting steps, and renumbering would break every citation in the document. So **insert with a letter suffix**: a step between `<1>1` and `<1>2` is `<1>1a`, then `<1>1b`. Lamport's own worked proof does this. Numbers and suffixed numbers are never reused within one proof, even after a deletion.

### Calculational steps

A chain of relations may replace a run of steps when each link is short:

```
<2>3. refs_after(v) covers refs_before(v)
  refs_after(v)
    = refs_before(v) + {o}        [BY <2>1 DEF apply_retain]
    covers refs_before(v)         [BY D7: `covers` is monotone under insertion]
```

State which transitivity closes the chain when the relations differ (`=` then `covers` closes to `covers`; `covers` then `=` likewise; mixing a relation with its converse does not close, and a chain that does not close is a `FALSE` finding). Each link carries its own bracketed justification, in `BY` format.

### How fine a step must be

The bar is a university entrance exam answer: not a research paper, where the reader is expected to reconstruct; an exam answer, where anything the grader has to supply costs marks. Three concrete tests, which the verifier applies:

- **Interpolation.** Does checking the step require the reader to invent an intermediate fact that is written nowhere? Then it is too coarse.
- **Case.** Does the step's truth depend on which branch of the code was taken, while the step names no branch? Then the branches belong in `CASE` steps.
- **Code distance.** If the step asserts what the code does, does the cited symbol show it plainly, or must the reader step into a function it calls? Each function stepped into is another step, with its own `CODE` citation — or, if it is used more than once, its own `P<n>`.

There is no penalty for length. A proof that takes three hundred steps and is checkable beats one that takes thirty and is not.

### Words that are not allowed

The failure this guards against is a proof that reads as though it understands the code while carrying its weight in words that mean nothing. Each of these is a finding on its own, before the step's logic is even considered:

- **An undefined word.** Every term a statement leans on is either (a) an identifier of the code under proof, (b) a `D<n>`, or (c) introduced by an earlier step in scope. A word like "生きている", "正しい位置", "整合している", "対応する" carries the argument and is not defined by being used confidently. Name it and define it, or use the code's own identifier.
- **A word that shifts.** One word, one definition, throughout the document. When two notions need two words, they get two words — and when the code has two names for one thing, the document says so in the definitions, once.
- **明らかに / obviously / 容易に分かる**, with no citation. Every such claim is either a `BY` or a substep.
- **同様に / similarly / 同じ議論で**. Either the other case is written out, or the shared part is lifted into a `P<n>` that both cases cite. The case skipped as "similar" is where the asymmetry hides.
- **Hedges**: 適切に, うまく, 基本的に, 通常, ほとんどの場合, essentially, in practice, should. A hedge marks the place where the writer does not know. The sentence is deleted or proved.
- **An appeal to intent**: "この関数は … するつもりで書かれている", "設計上 … のはず". The proof is about what the code does. Intent belongs to the definitions section, as the property being proved.

## The document

Under `dev-docs/YYYY-MM-DD-<target>-soundness/`, following the `devdoc` skill for everything the conventions below leave open — in particular, written in the implementers' language, self-contained for a reader who knows the project thinly and has never opened the code under proof, and readable front to back.

- `README.md` — target and commit, definitions, assumptions, the proposition list in dependency order, the calibration, the main theorem, and the verification status table.
- `p<NN>-<slug>.md` — one file per proposition, or per tightly coupled group. One file has one owner, so two provers never edit one file.

`README.md` holds, in this order:

1. **Target.** The commit hash the proof is about, in full, and the files and top-level symbols covered. A proof is about one state of the code and says which.
2. **Definitions** `D1`, `D2`, … Every notion the propositions use, including the soundness property itself. A definition is precise enough that two readers cannot disagree about whether a given program satisfies it. Where the notion is already a Rust type or function, define it by naming that item and stating what it means, so the document stays self-contained.
3. **Assumptions** `A1`, `A2`, … Facts the proof uses and does not prove. Each carries **who discharges it**: the caller (name the call site), an earlier pass (name it), a language rule (name it), or *nobody*. The soundness of the input is normally `A1` — a pass is proved to *preserve* soundness, and the composition of the passes is a separate proposition at the end that chains the preservations. An assumption discharged by nobody is a hole in the guarantee; it stays in the list, it appears in the main theorem's hypotheses, and the closing report names it.
4. **Propositions.** Every `P<n>` **statement** (not its proof), in an order where each depends only on earlier ones, with the dependency edges written down. Typically one proposition per function or per loop body, stated as its contract.
5. **Calibration** — see below.
6. **Main theorem.** The last proposition, and the shape of the chain that reaches it.
7. **Verification status.** One row per proposition: the file, the last verifier round that examined it, its verdict, and the commit the proof was written against. This is what a re-run reads to know what to re-verify.

## Calibrating the property

Before any prover starts, check that the property is worth proving: **take a bug the code actually had, and check that the stated property is violated by the old code.** The most recent fixed bug in the target is the natural case; `git show` on its fix gives the old code, and the changelog and the issue tracker give the symptom.

A property that the buggy code also satisfies is too weak, and a proof of it proves nothing while looking like it proves everything. Widen the property until the old bug violates it, and record in `README.md` which bug was used and which clause of the definition it breaks.

This is the `bug-hunt` rule "show the detector fires before trusting its silence", applied to a specification: a soundness property is a detector, and its silence is worth exactly as much as any other detector's.

Re-run the calibration whenever the property changes — above all when it changes because a proof would not close otherwise.

## Procedure

1. **Resolve the target and the property.** Ask with `AskUserQuestion` when either was left open. Confirm `git status --porcelain` is empty and record `git rev-parse HEAD`; that hash goes in the document. If the code changes while the work runs, re-verify every proposition whose `CODE` citations name a changed symbol.
2. **Write the skeleton, inline.** Read the target end to end — not by grep — and write `README.md`: definitions, assumptions, the decomposition into propositions with their statements and dependency order, the calibration, and the main theorem. This is the design of the proof and it needs the whole target in one head, so the orchestrator does it rather than a subagent.

   **Show the skeleton to the user before dispatching provers.** A wrong definition wastes every prover that runs under it, and the decomposition is where a proof is won or lost.
3. **Run the provers.** One subagent per proposition file, dispatched in dependency layers: a prover may cite an earlier proposition's *statement*, so a whole layer of independent propositions goes out in one parallel block. They write documents rather than code, so they share the working tree; file ownership is what keeps them apart. Brief each with the *Briefing a prover* section below.
4. **Run the verifiers.** One subagent per proposition, all in parallel, each given only what the *Briefing a verifier* section allows. Wait for all.
5. **Iterate.** Hand each verifier's findings to that file's prover. `NOT-OBVIOUS` is answered by inserting substeps, never by rewording the step to sound more certain. `FALSE`, `UNDEFINED`, `BAD-CITATION` and `HEDGE` are answered as the section below prescribes. Then verify again — with **fresh** verifier subagents, which have not seen the previous round's findings, so the check is never anchored to what it already accepted.
6. **Stop at the fixed point.** The document is finished when one full round over every proposition returns no finding of any kind. Record the round in the status table.
7. **Report.** What was proved; under which assumptions; which assumptions nobody discharges; how many rounds it took; and every code bug the attempt turned up, in `bug-hunt` shape. Then update the hunt log's neighbours in memory if the proof changed what is known about the subsystem.

## Briefing a prover

Give it: the target commit; `README.md` in full; the proposition it owns and the file to write; the statements (not the proofs) of the propositions it may cite; the *proof language* and *words that are not allowed* sections of this file, inline; and, on a later round, the verifier findings against its file.

Require of it:

- It reads the code it cites. A `CODE` citation is a claim about the source, and a prover that cites a symbol it did not open is the failure mode this whole procedure exists to catch.
- It writes only its own file.
- It does not modify the code under proof, and it does not modify `README.md` — a definition or assumption it finds it needs is **reported to the orchestrator**, not added. Definitions are global, and a prover that adds one silently breaks the propositions proved under the old set.
- When it cannot prove its proposition, it says so, with the step it got stuck at and which of the three cases below it believes it is in. A prover that cannot prove something and writes a plausible paragraph instead has done the worst available thing.

## Briefing a verifier

Give it, and only it:

- The definitions and assumptions from `README.md`.
- The **statements** of every proposition — never their proofs, except the one it is checking. A verifier that has read the whole argument starts reconstructing it, and a reconstructed argument is exactly what it is supposed to fail to do.
- The one proposition file it checks.
- The *proof language* and *words that are not allowed* sections of this file, inline.
- Read access to the repository, for one purpose only: opening a symbol named in a `CODE` citation to check that the code says what the step claims.

Instruct it in these terms:

> You are grading an exam answer. **Do not think.** Your job is not to decide whether the proposition is true — it is to decide, for each step, whether that step follows **obviously** from the items its `BY` line cites, and nothing else.
>
> You will be tempted to fill in gaps, because you can. Do not. **Anything you supply was missing from the proof.** If you find yourself thinking "presumably this means…", or "this is true because of that other thing I know about the code", or pausing even for a moment over what a word means — that step is a finding, not an OK.
>
> Do not edit the document. Do not propose fixes, do not rewrite steps, do not suggest wording. Report.
>
> Give a verdict for **every** step, in document order, including the ones that pass. Silence is not OK; a step you did not mention is a step you did not check, and the orchestrator has no way to tell the two apart. Then detail every non-OK verdict.

The verdicts:

- **`OK`** — follows obviously from the cited items.
- **`FALSE`** — does not follow, or is false. Give the counterexample or name the broken inference.
- **`NOT-OBVIOUS`** — may well be true, but reaching it from the cited items required you to supply something. Say exactly what you had to supply.
- **`UNDEFINED`** — the step leans on a word or symbol that is not in the definitions, not an identifier of the cited code, and not introduced by an earlier step in scope. Name the word.
- **`BAD-CITATION`** — a cited item does not exist, is out of scope by the numbering rule, or does not say what the step claims. For a `CODE` citation, open the file and compare; report the mismatch in the code's own words.
- **`HEDGE`** — the step's argument rests on a word from *Words that are not allowed*. Quote it.
- **`INCOMPLETE`** — a `CASE` split whose cases are not exhaustive (name the missing case, by the arm of the type or the condition it corresponds to), a `QED` that does not cite everything its conclusion needs, a step with neither a `BY` nor a subproof, or a calculational chain whose relations do not compose.

A verifier that returns `OK` for everything on its first pass over a proof of any size has almost certainly been persuaded rather than convinced; the orchestrator treats that as a signal to re-run with a fresh verifier before believing it.

## When the proof will not close

Three cases, and they are told apart before anything is written:

1. **The proof is wrong.** The decomposition, or a step. Fix it and continue. This is the common case and needs no ceremony.
2. **The statement is wrong** — the proposition needs a precondition nobody wrote down. The precondition is promoted to an `A<n>` **only with a named discharger**: the caller that establishes it, the earlier pass that guarantees it, the language rule that makes it impossible to violate. Adding an assumption nobody discharges does not close the proof; it moves the hole, and it must be reported as such rather than buried in the list.
3. **The code is wrong.** The property genuinely fails for some input. Stop and report it as `bug-hunt` does: the input, the path it takes, the wrong output, and a fix proposal. Do not fix the code, and do not prove the pass sound "modulo that case".

The rule that binds all three: **never weaken the property to make the proof close.** A property is weakened only deliberately, and when it is, the calibration is re-run; if the old bug now satisfies the weakened property, the weakening is rejected and the case is (2) or (3) instead.

## Re-verifying after the code changes

A proof is about one commit. When the code moves, the document is not thrown away:

1. Diff the new commit against the one in `README.md`, and take the set of changed symbols.
2. Every proposition whose file cites a changed symbol in a `CODE` line goes back to a prover, then to a verifier.
3. Every proposition that cites one of those propositions is re-verified too, transitively — the statement may have moved even when the file did not.
4. Update the target commit and the status table.

A change that touches no cited symbol changes only the recorded commit. That is worth saying out loud: it means the `CODE` citations are also the index from a code change to the proof obligations it disturbs, which is most of what makes the document worth maintaining rather than rewriting.
