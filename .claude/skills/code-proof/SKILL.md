---
name: code-proof
description: "Prove that a piece of code has a stated property — that inputs satisfying a precondition produce outputs satisfying a postcondition — and write the proof as a dev doc: definitions, then a sequence of numbered propositions whose every leaf step cites exactly the facts, definitions and code it rests on, in Lamport's structured-proof style. The orchestrator fixes the target commit, states the property and calibrates it against a bug the code actually had, has a critic subagent judge whether the property is the right one and strong enough, then alternates prover subagents that write the proof with verifier subagents that check one step at a time and report every step that is false, non-obvious, hedged, or built on an undefined word. Use when: asked to prove that a pass, a function, or a subsystem is correct, sound, safe, or otherwise satisfies a property; or to re-verify an existing proof after the code changed."
argument-hint: "The processing to prove something about (a pass, a function, a module), and optionally the property to prove. If omitted, the skill asks."
---

# Proving a property of code

The deliverable is a document: **definitions, then a sequence of propositions, each with a proof**, ending in the theorem that the target has the property. It is written for a reader who checks it rather than one who is persuaded by it, so every inference step must be obvious from the items it cites alone.

The work is done by three kinds of subagent under this orchestrator: a **critic**, who judges whether the property and the decomposition are the right ones; **provers**, who write proofs; and **verifiers**, who check one step at a time and are forbidden to think. The document is finished when a fresh round of verifiers returns nothing.

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

### Never write a range, a count, or any other derived figure

A range of names -- `P8 - P14b`, `D1`-`D34`, "assumptions A1 through A26" -- names its endpoints and
leaves its members to be reconstructed. Nothing can resolve it: not a reader who does not know which
numbers exist, not a tool, and not the author a year later. Measured, a lettered number placed
between two others (`A26a`) dropped out of a range that read "A1 through A26", and two steps that
claimed to check every assumption silently skipped it.

The same holds for a count. "The 78 implementations", "the six arms of the match", "the three places
that write the field" -- each is a figure derived from the code or from the frame, and each goes
stale the moment either moves, silently, because a wrong number reads exactly like a right one.
Measured across six proof documents, half of the counted claims sat next to the enumeration they
counted, so the figure carried nothing the list did not already carry.

**Write what the figure was derived from.** Name the members, or give the predicate that selects
them -- "every `impl LLVMGen` whose `result_prov` returns a `Fresh` leaf" resolves forever, while
"the 29 such implementations" resolves until someone adds one. Where a count is the claim itself --
zero occurrences, or exactly one call site when uniqueness is the point -- it stays, because there
the figure is the property rather than a description of it.

### Identity is not a number

A step cites `P28` and a document is named `p20-borrow-ify.md`, so numbers and names look like
identities. They are not. A number is a display: it orders items for a reader, and ordering is the
one thing that changes when an item is inserted. Making it the identity is what forces the letter
suffixes of the preceding section, and those suffixes are what ranges then lose.

Give each item an identity that carries no meaning -- a short random string -- and let the number
stay a display that may be reassigned freely. A tool then answers "what cites this" and "what moved
under this" from the identities, and renumbering costs nothing. Keep the number in the prose: an
identity is for machines to follow, and a reader who meets `a3f9c21` in a `BY` line learns nothing.

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

## What the proof talks about

Before the definitions can be written, one thing has to be settled: **what the proof takes as given about the layer below.** Code sits on other code — a compiler pass on the semantics of the program it rewrites, a data structure on its allocator, a protocol on its transport. A proof that tries to reach all the way down never gets written; one that never says where it stops is proving nothing about anything.

So the skeleton names the boundary, as assumptions with no discharger. One pattern recurs: the code under proof reads a **declared model** of the layer below — a table of what each primitive does, an interface's contract, an invariant a type promises — and reasons from that. The proof is then about the declared model, and one assumption says the declaration is faithful to what the layer actually does. That assumption is the proof's largest hole, and writing it down is what makes it a known hole rather than an assumed truth. Say beside it what does check it — a test, an audit, a runtime detector — so a reader can see what the proof stands on.

The definitions also have to say what an **execution** is and what **state** it moves, because the property is a statement about executions and is undefined without them. For a transformation, that means saying what a run of the input looks like, what a run of the output looks like, and what has to correspond between the two.

## The document

Under `dev-docs/YYYY-MM-DD-<target>-<property>/`, following the `devdoc` skill for everything the conventions below leave open — in particular, written in the implementers' language, self-contained for a reader who knows the project thinly and has never opened the code under proof, and readable front to back.

- `README.md` — target and commit, definitions, assumptions, the proposition list in dependency order, the calibration, the main theorem, and the verification status table.
- `p<NN>-<slug>.md` — one file per proposition, or per tightly coupled group. One file has one owner, so two provers never edit one file.

`README.md` holds, in this order:

1. **Target.** The commit hash the proof is about, in full, and the files and top-level symbols covered. A proof is about one state of the code and says which.
2. **Definitions** `D1`, `D2`, … Every notion the propositions use, including the property itself. A definition is precise enough that two readers cannot disagree about whether a given program satisfies it. Where the notion is already a Rust type or function, define it by naming that item and stating what it means, so the document stays self-contained.
3. **Assumptions** `A1`, `A2`, … Facts the proof uses and does not prove. Each carries **who discharges it**: the caller (name the call site), an earlier pass (name it), a language rule (name it), or *nobody*. For a transformation, the input having the property is normally `A1` — the transformation is proved to *preserve* it, and the composition of the transformations is a separate proposition at the end that chains the preservations. An assumption discharged by nobody is a hole in the guarantee; it stays in the list, it appears in the main theorem's hypotheses, and the closing report names it.
4. **Propositions.** Every `P<n>` **statement** (not its proof), in an order where each depends only on earlier ones, with the dependency edges written down. Typically one proposition per function or per loop body, stated as its contract.
5. **Calibration** — see below.
6. **Main theorem.** The last proposition, and the shape of the chain that reaches it.
7. **Verification status.** One row per proposition: the file, the last verifier round that examined it, its verdict, and the commit the proof was written against. This is what a re-run reads to know what to re-verify.

### Check the skeleton before dispatching

The definitions carry every proof in the document, so an undefined word in one costs as many rewrites as there are provers working under it. Run the verifier's checks over the skeleton itself first — `UNDEFINED` and `HEDGE` against the definitions, the assumptions, and the proposition statements — before any prover starts. One verifier subagent on the skeleton alone is cheap beside a layer of provers building on a definition that then has to change under them.

Two rules keep definitions through that check:

- **A definition that quantifies over constructs enumerates them.** "the constructs that read a value", "the ones that create a reference", "the ones that consume" — each is a closed list in the language under proof, and writing the list is what makes the definition checkable. Given as a property instead, every prover derives the list itself, and they derive different ones.
- **A definition that names a function of the code says whether that function *is* the definition or merely implements it.** Where the two can differ — the function reports a superset, or answers for a case the definition leaves out — say so in the definition, and name the reader that depends on the difference. A prover told the function is the definition cannot see that gap, and the gap is where a bug sits.
- **A definition drawn from an enumerating function says whether the enumeration is exact or an over-approximation, and of what.** A function that enumerates the positions a value *could* hold something enumerates, at run time, a superset of the positions it *does*. Writing "exactly one per enumerated position" turns a static possibility into a dynamic fact, and every proposition built on it then proves something that is not true. The passes themselves usually work on the over-approximation quite deliberately, so the definition has to carry both the static set and the dynamic subset, and each proposition has to say which one it means.

Expect this first check to return findings on most items. That is the normal yield, not evidence the skeleton was written badly: the orchestrator is no better at writing definitions than a prover is at writing proofs, and the definitions are read by everyone.

**The check is a gate, not a parallel task.** Running it beside the first provers saves nothing: a definition that changes underneath a prover costs that prover a full rewrite, and it costs one per prover. Wait for it.

### Link the proof and the code both ways

A citation runs one way: the proof names the code it rests on, and the code says nothing back. Someone editing a cited item has no way to learn a proof stands on it, and nobody can answer "what changed under this proof since it was written" without reading the whole diff.

So each cited item carries a comment naming the propositions that rest on it, and the proof carries the digest of each cited item's source text. **Generate both from the citations** rather than writing either by hand: two hand-kept sides drift, and a digest that stops matching is what turns "did anything change" into "re-verify exactly these propositions".

`dev-docs/proof/proof_links.py` does this for this repository — `--write` to regenerate, no argument to check. Show the check fires before trusting its silence: rename a cited item, and mutate one's body, and see it report each.

Do not transcribe the code into the proof instead. **An unchecked copy is worse than a citation**: it looks authoritative, it drifts silently, and a verifier checking a step against the copy will pass a step that is false against the real code — which is the one thing the whole procedure exists to prevent. Quote the few lines a step's inference turns on, inside that step; cite the symbol for everything else. Quote small, cite large.

## Calibrating the property

Before any prover starts, check that the property is worth proving: **take a bug the code actually had, and check that the stated property is violated by the old code.** The most recent fixed bug in the target is the natural case; `git show` on its fix gives the old code, and the changelog and the issue tracker give the symptom.

A property that the buggy code also satisfies is too weak, and a proof of it proves nothing while looking like it proves everything. Widen the property until the old bug violates it, and record in `README.md` which bug was used and which clause of the definition it breaks.

This is the `bug-hunt` rule "show the detector fires before trusting its silence", applied to a specification: a stated property is a detector, and its silence is worth exactly as much as any other detector's.

**A property with several clauses needs a bug per clause.** One bug that breaks two clauses at once leaves the rest uncalibrated, and an uncalibrated clause is where a missing clause hides: nothing ever tested whether the property notices that kind of wrongness, because nothing of that kind was ever tried against it. Where no past bug breaks a clause on its own, break it by hand — mutate the code so that only that clause fails, and check the property catches it.

**Then write down what the property does not cover.** Beside the definition, name the class of wrong behaviour it permits. A transformation proved to preserve an invariant may still compute the wrong answer; a transformation proved to compute the right answer may still leak. A document that states only what it proves reads as though it proved more, and the reader who most needs the limit — the one deciding whether the proof lets them stop worrying — is the one who will not find it.

Re-run the calibration whenever the property changes — above all when it changes because a proof would not close otherwise.

## Briefing a critic

A verifier works inside the frame: does this step follow from what it cites? A critic asks whether the frame is the right one. The two disciplines are opposite — a verifier is told not to think, and a critic has to — so they are separate passes with separate subagents, and neither is asked to do the other's job.

The orchestrator wrote the skeleton, which makes it the worst judge of whether the skeleton is right. That is why this is a subagent and not a self-review.

Give it: `README.md` in full; the target and what the requester asked for, in their own words; and the calibration. Not the proofs — it is judging the statements.

Have it answer these, in order, and rank what it finds:

- **Adequacy.** If every proposition were proved, what would the reader be entitled to conclude? State it in the requester's words rather than the document's. Is that what was asked for?
- **Strength.** **Name a wrong program that satisfies the property.** If one comes easily, the property is too weak, and every proof built on it will prove less than it appears to. This is the sharpest of the questions and the cheapest to answer, and it catches the failure the others miss: a property that is internally consistent, precisely stated, well-calibrated against one past bug, and still permits the thing the reader most fears.
- **Assumption load.** Which assumption, dropped, would make the theorem false? Is one of them doing so much work that the theorem is nearly vacuous? An assumption nobody discharges that carries the whole result is a hole wearing a proof's clothes.
- **Decomposition.** Does any proposition exist to make its own proof easy rather than to say something about the code? Does the chain from the propositions to the main theorem actually close, or does it close only under a step nobody has written?
- **Terminology.** Does the property's name mean, in the field this code belongs to, what the document uses it to mean? A pass is called *correct* when it preserves semantics; *sound* is what an analysis or a logic is. A name borrowed wrongly invites every reader to assume a result the document does not prove.

Run it twice, and only twice. Running it continuously produces churn against a frame that is fine.

- **After the skeleton, before any prover.** The highest-leverage moment: everything downstream is built on the frame, and a frame corrected here costs nothing while one corrected later costs every proof above it.
- **After the main theorem closes.** The moment the reader will over-read the result. What the theorem lets them stop worrying about, and what it does not, belongs in the document in the critic's words.

## Procedure

1. **Resolve the target and the property.** Ask with `AskUserQuestion` when either was left open. Confirm `git status --porcelain` is empty and record `git rev-parse HEAD`; that hash goes in the document. If the code changes while the work runs, re-verify every proposition whose `CODE` citations name a changed symbol.
2. **Write the skeleton, inline.** Read the target end to end — not by grep — and write `README.md`: definitions, assumptions, the decomposition into propositions with their statements and dependency order, the calibration, and the main theorem. This is the design of the proof and it needs the whole target in one head, so the orchestrator does it rather than a subagent.

   **Show the skeleton to the user before dispatching provers.** A wrong definition wastes every prover that runs under it, and the decomposition is where a proof is won or lost.
3. **Run the critic on the skeleton, and wait.** One subagent, briefed as *Briefing a critic* says. Act on what it returns before dispatching anything: a property the critic can name a wrong program for is a property to widen now, not after the proofs are written against it.
4. **Run the provers.** One subagent per proposition file, dispatched in dependency layers: a prover may cite an earlier proposition's *statement*, so a whole layer of independent propositions goes out in one parallel block. **Each one gets its own git worktree** — see *Give every agent its own worktree*. Brief each with the *Briefing a prover* section below.

   A prover on a real subsystem is a long-running, expensive agent, so send a layer out a couple at a time — see *Two agents at a time*. When one is interrupted, its file is on disk: commit what is there and **resume that agent** rather than launching a fresh one, since the reading it has already done is most of what it spent.
5. **Run the verifiers.** One subagent per proposition, each given only what the *Briefing a verifier* section allows, dispatched a couple at a time — see *Two agents at a time*. Wait for all.
6. **Iterate.** Hand each verifier's findings to that file's prover. `NOT-OBVIOUS` is answered by inserting substeps, never by rewording the step to sound more certain. `FALSE`, `UNDEFINED`, `BAD-CITATION` and `HEDGE` are answered as the section below prescribes. Then verify again — with **fresh** verifier subagents, which have not seen the previous round's findings, so the check is never anchored to what it already accepted.
7. **Stop at the fixed point.** The document is finished when one full round over every proposition returns no finding of any kind. Record the round in the status table.
8. **Report.** Run the critic a second time before this, on the closed document, and carry what it says about the result's limits into the report. What was proved; under which assumptions; which assumptions nobody discharges; how many rounds it took; and every code bug the attempt turned up, in `bug-hunt` shape. Then update the hunt log's neighbours in memory if the proof changed what is known about the subsystem.

### Give every agent its own worktree

Dispatch provers and verifiers with `isolation: "worktree"`. A worktree of a source tree is small — measure it, but it is the tracked files and not the build directory, so a proof agent that never compiles costs almost nothing.

**File ownership does not keep agents apart in a shared tree.** Each brief already says which one file the agent owns, and that is not what breaks. What breaks is a git command that writes the whole working tree: `git stash` to set work aside, `git checkout -- <path>` to undo a probe, `git reset --hard` to drop a commit. Each of those silently discards every other agent's uncommitted work, and the agent that ran it had no idea anyone else was there. Forbidding them by name does not hold either — the list has a next member, and the orchestrator is as likely to run one as any subagent.

So the brief carries both: the worktree, and the line that no agent runs a git command which writes the working tree. Reads — `git rev-parse HEAD`, `git log`, `git diff` of its own file — stay open.

**Name the commit in the brief.** A worktree is created from the repository's default branch, which during a proof effort is far behind the frame. An agent that reads that README grades against definitions that were replaced hours ago. So the brief opens with the commit to sync to and the command to do it, and the agent reports which commit it actually read — that report is what lets the orchestrator tell a stale finding from a live one.

**The agent commits its own file** on its own branch, naming the path (never `git add -A`), and reports the branch. The orchestrator merges. Since the agents own disjoint files and only the orchestrator writes `README.md`, the merges are clean.

**Hold frame changes until the round returns.** Each agent's `README.md` is frozen where it started, so a definition the orchestrator edits mid-round reaches nobody and turns every report that touches it into a claim about a document that no longer exists. Collect the frame edits a round asks for, apply them between rounds, and dispatch the next round from the new commit. This is the structural half of the rule under *Briefing a prover* that a report quoting the document must re-read it first: the rule catches a stale claim, and the batching stops most of them from being written.

### Two agents at a time

Keep about two of these agents in flight. They are the expensive kind — each reads a whole proof file, the frame, and every symbol its citations name — and the ceiling that matters is not the machine's but the session's.

**A rate limit is session-wide, and it kills every agent in flight at the same moment.** Ten agents dispatched together do not fail one at a time as capacity tightens; they all stop mid-sentence on the same request. What survives is only what each had already committed, so a wide fan-out converts a transient limit into the loss of a whole round's reading — the reading being most of what the round cost. Two at a time turns the same limit into a pause.

So the brief tells the agent to **commit as it goes** rather than once at the end: a finished section is worth committing even though the file is not done. Then an interrupted agent has something to resume onto, and *resume* it — the transcript holds the citations it has already opened.

The orchestrator's own work fills the gaps: while two agents run, edit the frame, read the reports that have landed, and prepare the next brief. Waiting is not the cost; re-reading is.

## Briefing a prover

Give it: the target commit; `README.md` in full; the proposition it owns and the file to write; the statements (not the proofs) of the propositions it may cite; the *proof language*, *how fine a step must be* and *words that are not allowed* sections of this file, **inline**; and, on a later round, the verifier findings against its file.

Inline, because the subagent may be working on a branch where this skill file does not exist — a briefing that sends it off to read the conventions is a briefing whose conventions it may not find.

The orchestrator may point a prover at the parts it expects to be hard — the alias chain whose identity has to survive, the case split that is easy to leave incomplete. A prover is proving a fixed statement rather than searching, so a hint changes where it spends its time and not what counts as an answer. It must not point at **how the question came out before**: naming the commit that fixed a bug in the very property under proof invites the prover to re-derive that fix's own reasoning and hand it back as a proof. Describe the shape that is hard, and leave out what was concluded about it.

Require of it:

- It reads the code it cites. A `CODE` citation is a claim about the source, and a prover that cites a symbol it did not open is the failure mode this whole procedure exists to catch.
- It writes only its own file.
- It does not modify the code under proof, and it does not modify `README.md` — a definition or assumption it finds it needs is **reported to the orchestrator**, not added. Definitions are global, and a prover that adds one silently breaks the propositions proved under the old set.
- **A report that the document is missing something quotes what the document says there instead.** Name the section looked in and quote the nearest text, or state that the section has nothing on the subject. Without that, the report is a claim about a document the prover read at some earlier point, and the orchestrator cannot tell a real gap from one already closed.

  This is the single most common way a round is wasted. `README.md` moves between rounds — often *because* of the prover's own earlier report — and a prover that drafts its findings from memory of the version it started with will hand back items that were answered before it wrote them. Re-read the definitions and assumptions **immediately before** writing the report, not at the start of the work.
- **It stops at a bug.** The moment a step will not close because the code does not do what the proposition needs, the prover returns — right then, with the step it got stuck at, the input that breaks it, and which of the three cases below it believes it is in. It does not prove the rest of the file first, it does not prove the proposition "modulo that case", and it does not add an assumption to route around it. A bug is the most valuable thing a prover can return, and it is worth much less buried under a proof of everything else, written after the writer already knew the subject was broken.
- A prover that cannot prove something and writes a plausible paragraph instead has done the worst available thing.

## Briefing a verifier

Give it, and only it:

- The definitions and assumptions from `README.md`.
- The **statements** of every proposition — never their proofs, except the one it is checking. A verifier that has read the whole argument starts reconstructing it, and a reconstructed argument is exactly what it is supposed to fail to do.
- The one proposition file it checks.
- The *proof language* and *words that are not allowed* sections of this file, inline.
- Read access to the repository, for one purpose only: opening every symbol named in a `CODE` citation to check that the code says what the step claims.

**Every `CODE` citation is opened, not only the suspicious ones.** A proof of a program is a claim about source text, and the reader who never opens the source is checking the argument's grammar rather than its subject — which is the failure the whole procedure exists to catch. So the verifier's report opens with a **citation ledger**: one row per distinct `CODE` citation, naming the file and symbol and quoting the lines the citing steps rely on. A symbol that cannot be found is a `BAD-CITATION` against every step that cites it. **A step whose `CODE` citations are not all in the ledger may not be given `OK`.**

The ledger costs the verifier one read per symbol and it buys the one thing the orchestrator otherwise cannot recover: the difference between a citation that was checked and a citation that was assumed. Both look like `OK`.

Instruct it in these terms:

> You are grading an exam answer. **Do not think.** Your job is not to decide whether the proposition is true — it is to decide, for each step, whether that step follows **obviously** from the items its `BY` line cites, and nothing else.
>
> You will be tempted to fill in gaps, because you can. Do not. **Anything you supply was missing from the proof.** If you find yourself thinking "presumably this means…", or "this is true because of that other thing I know about the code", or pausing even for a moment over what a word means — that step is a finding, not an OK.
>
> Do not edit the document. Do not propose fixes, do not rewrite steps, do not suggest wording. Report.
>
> Give a verdict for **every** step, in document order, including the ones that pass. Silence is not OK; a step you did not mention is a step you did not check, and the orchestrator has no way to tell the two apart. Then detail every non-OK verdict.
>
> Build the citation ledger first, by opening every symbol any `CODE` line names. Quote what you found. Then grade, and grade a step's `CODE` citations against your ledger rather than against what you expect the code to say.

The verdicts:

- **`OK`** — follows obviously from the cited items.
- **`FALSE`** — does not follow, or is false. Give the counterexample or name the broken inference.
- **`NOT-OBVIOUS`** — may well be true, but reaching it from the cited items required you to supply something. Say exactly what you had to supply.
- **`UNDEFINED`** — the step leans on a word or symbol that is not in the definitions, not an identifier of the cited code, and not introduced by an earlier step in scope. Name the word.
- **`BAD-CITATION`** — a cited item does not exist, is out of scope by the numbering rule, or does not say what the step claims. For a `CODE` citation, compare the step against the ledger row and report the mismatch in the code's own words. A citation that names a symbol the repository does not have is this verdict even when the step's claim is true of some other symbol.
- **`HEDGE`** — the step's argument rests on a word from *Words that are not allowed*. Quote it.
- **`INCOMPLETE`** — a `CASE` split whose cases are not exhaustive (name the missing case, by the arm of the type or the condition it corresponds to), a `QED` that does not cite everything its conclusion needs, a step with neither a `BY` nor a subproof, or a calculational chain whose relations do not compose.

A verifier that returns `OK` for everything on its first pass over a proof of any size has almost certainly been persuaded rather than convinced; the orchestrator treats that as a signal to re-run with a fresh verifier before believing it.

A verifier edits nothing, so it makes no commit, but it still reports the commit its worktree ended up on. A finding of the form "the document claims the frame lacks X, and the frame has X" is only worth acting on when the verifier read the frame the orchestrator meant.

## When the proof will not close

Three cases, and they are told apart before anything is written:

1. **The proof is wrong.** The decomposition, or a step. Fix it and continue. This is the common case and needs no ceremony.
2. **The statement is wrong** — the proposition needs a precondition nobody wrote down. The precondition is promoted to an `A<n>` **only with a named discharger**: the caller that establishes it, the earlier pass that guarantees it, the language rule that makes it impossible to violate. Adding an assumption nobody discharges does not close the proof; it moves the hole, and it must be reported as such rather than buried in the list.
3. **The code is wrong.** The property genuinely fails for some input. Stop and report it as `bug-hunt` does: the input, the path it takes, the wrong output, and a fix proposal. Do not fix the code, and do not prove the property "modulo that case".

   The orchestrator stops with it. It refutes the claim first, the way `bug-hunt` verifies a candidate — can any input reach that path, is the result genuinely wrong, does it reproduce on the commit under proof — and then holds the layer: the propositions that depend on the stuck one are not dispatched, because their statements rest on a contract the code does not meet, and proofs written against a contract that is about to change are proofs written twice. Work that depends on nothing stuck carries on.

The rule that binds all three: **never weaken the property to make the proof close.** A property is weakened only deliberately, and when it is, the calibration is re-run; if the old bug now satisfies the weakened property, the weakening is rejected and the case is (2) or (3) instead.

## Re-verifying after the code changes

A proof is about one commit. When the code moves, the document is not thrown away:

1. Diff the new commit against the one in `README.md`, and take the set of changed symbols.
2. Every proposition whose file cites a changed symbol in a `CODE` line goes back to a prover, then to a verifier.
3. Every proposition that cites one of those propositions is re-verified too, transitively — the statement may have moved even when the file did not.
4. Update the target commit and the status table.

A change that touches no cited symbol changes only the recorded commit. That is worth saying out loud: it means the `CODE` citations are also the index from a code change to the proof obligations it disturbs, which is most of what makes the document worth maintaining rather than rewriting.
