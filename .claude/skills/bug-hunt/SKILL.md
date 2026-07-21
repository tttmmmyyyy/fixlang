---
name: bug-hunt
description: "Hunt for latent bugs in a chosen target through a fan-out of finder subagents, kill the false positives with adversarial verification, and report each surviving bug with a reproduction test, a fix proposal, and a recurrence barrier. Report-only: it never fixes code and never commits. Use when: sweeping a subsystem for defects, auditing before a merge or release, or running the periodic hunt."
argument-hint: "Target (a subsystem path, a branch's diff, the standard library, or the whole compiler) and optionally a lens (miscompilation, memory/RC, boundary, invariant, error path, tool behavior). If omitted, the skill asks."
---

# Bug Hunt

Find bugs that are already in the code and that nobody is looking for. The deliverable is a report: for each bug, what it is, a test that reproduces it, a fix the author can weigh, and a barrier that stops the class from coming back.

This skill **never edits the code under test and never commits**. A compiler fix needs the author's judgment and a test, and the hunt runs unattended often enough that silent fixes would be dangerous. Its only writes are its own *Techniques That Found Bugs* section and the hunt log in memory.

`code-review` is the complement: it applies conventions to a diff, in one pass, and it edits. A hunt is shaped differently — bugs run out only when repeated search stops finding new ones, and most of what a search turns up is wrong and has to be killed before it reaches the user.

## Target and Lens

The invoker names the target, and optionally the lens. When either is missing, ask with `AskUserQuestion` before starting.

**Targets** — anything with a boundary the hunt can enumerate:

- A subsystem path (`src/rc_ir/`, `src/typecheck.rs`, the LSP server, the package manager, the documentation generator).
- The diff of a branch or a range of commits.
- The Fix standard library (`std.fix`) and the behavior `Document.md` promises for it — a documented behavior the compiler does not deliver is a bug in one of the two.
- The whole compiler, which the scout pass then splits into areas.

**Lenses** — each finder gets one, and a hunt runs several so that the search angles stay independent:

- **Miscompilation** — the compiler accepts a program and emits code that computes the wrong thing. The highest-severity class here, and the hardest to see by reading, because the wrong answer surfaces far from its cause.
- **Memory and reference counting** — leaks, double frees, use-after-free, a retain/release asymmetry on one path, an ownership declaration that disagrees with what the code does.
- **Boundary and edge cases** — empty input, one element, the first and last index, the recursion base case, integer overflow, an empty struct or a zero-field union.
- **Invariants** — something the code relies on without checking, and a path that can violate it: a stale cache, an index into a container that has since changed shape, a two-field pair that must stay in sync.
- **Error and failure paths** — the arm nobody runs: a malformed source file, a missing dependency, a broken manifest, an I/O failure, a user program that fails to type-check in an unusual way.
- **Tool behavior** — `fix` as a command: the build cache, dependency resolution, the LSP responses, the generated documentation.

The lens the last hunt used is in the hunt log. Start elsewhere: rotating the lens finds more than deepening the same one.

## Severity

Report most severe first, and use this ordering when the budget forces a choice about what to chase:

1. **Miscompilation** — valid program, wrong behavior, no diagnostic.
2. **Crash on valid input** — the compiler panics or aborts on a program it should accept, or on one it should reject with a diagnostic.
3. **Memory error** — leak, double free, use-after-free in the emitted program or in the compiler.
4. **Wrong diagnostic** — an error reported at the wrong place, with the wrong cause, or missing entirely for an invalid program.
5. **Tool misbehavior** — a stale build, a wrong dependency resolution, an LSP answer that points at the wrong symbol.

## Evidence Bar

A candidate becomes a finding when it carries a **concrete failing scenario**: named inputs or program state, the path they take, and the wrong output or crash they produce. Everything else is speculation and stays out of the report.

Rank the evidence honestly in each finding:

- **Executed** — the failure was reproduced by running something. This is what the hunt is for; prefer it wherever the target allows.
- **Traced** — the failing path was followed line by line through the code, and every step is cited by file and symbol. Say what stops it from being executed.

A finding with neither is dropped.

## Procedure

1. **Resolve target and lens.** Ask when the invocation left either open. Confirm the working tree is clean (`git status --porcelain`) and note the current commit — the hunt reports against that state.
2. **Read the hunt log** from memory: which targets and lenses previous hunts covered, which candidates were dismissed and why. A dismissed candidate is re-raised only with new evidence.
3. **Scout, inline.** Walk the target and build the work-list: the areas a finder can own (a file, a pass, a module, a phase), and for each, what it is responsible for. Keep the round to about a dozen finders — areas times lenses — and widen only when the invoker asked for an exhaustive sweep. Report the work-list before launching, so a mis-scoped hunt is caught early.
4. **Run the workflow** below via the `Workflow` tool, passing the target, the areas, and the lenses as `args`.
5. **Report** as described under *Report*, then **append** any technique that earns it, then **update the hunt log**.
6. **Leave the tree as you found it.** Verify `git status --porcelain` is empty at the end, and that every temporary probe is reverted.

### Workflow

```js
export const meta = {
  name: 'bug-hunt',
  description: 'Fan out finders over a target, adversarially verify each candidate, deepen the survivors',
  phases: [
    { title: 'Find', detail: 'one finder per area x lens, repeated until dry' },
    { title: 'Verify', detail: 'three independent refuters per candidate' },
    { title: 'Deepen', detail: 'repro test, fix, and barrier for each survivor' },
  ],
}

const CANDIDATE = {
  type: 'object',
  properties: {
    candidates: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          file: { type: 'string' },
          symbol: { type: 'string' },
          claim: { type: 'string' },
          scenario: { type: 'string' },
          evidence: { type: 'string', enum: ['executed', 'traced'] },
          severity: { type: 'integer' },
        },
        required: ['file', 'symbol', 'claim', 'scenario', 'evidence', 'severity'],
      },
    },
  },
  required: ['candidates'],
}

const VERDICT = {
  type: 'object',
  properties: {
    refuted: { type: 'boolean' },
    reason: { type: 'string' },
  },
  required: ['refuted', 'reason'],
}

const REPORT = {
  type: 'object',
  properties: {
    title: { type: 'string' },
    severity: { type: 'integer' },
    description: { type: 'string' },
    repro_test: { type: 'string' },
    fix: { type: 'string' },
    barrier: { type: 'string' },
    technique: { type: 'string' },
  },
  required: ['title', 'severity', 'description', 'repro_test', 'fix', 'barrier'],
}

const { target, areas, lenses } = args
const REFUTE_LENSES = [
  'Reachability: construct the input that reaches this line, or show no input does.',
  'Semantics: assume the path is taken and decide whether the result is actually wrong.',
  'Reproduction: build and run something that exhibits the failure, and report what happened.',
]

const key = (c) => `${c.file}::${c.symbol}::${c.claim.slice(0, 80)}`
const seen = new Set()
const confirmed = []
let dryRounds = 0

while (dryRounds < 2) {
  const round = await parallel(
    areas.flatMap((area) =>
      lenses.map((lens) => () =>
        agent(
          `Hunt for bugs in ${target}, area: ${area}, through this lens: ${lens}.\n` +
            `Read the code. Build and run whatever demonstrates a failure — the compiler is here and works.\n` +
            `Report a candidate only with a concrete failing scenario: named inputs or state, the path they take, and the wrong output or crash.\n` +
            `Already reported, do not repeat:\n${[...seen].join('\n') || '(nothing yet)'}\n` +
            `Revert every temporary probe you make before you finish.`,
          { label: `find:${area}`, phase: 'Find', schema: CANDIDATE },
        ),
      ),
    ),
  )
  const fresh = round.filter(Boolean).flatMap((r) => r.candidates).filter((c) => !seen.has(key(c)))
  if (fresh.length === 0) {
    dryRounds++
    continue
  }
  dryRounds = 0
  fresh.forEach((c) => seen.add(key(c)))

  const survivors = await pipeline(
    fresh,
    (c) =>
      parallel(
        REFUTE_LENSES.map((lens) => () =>
          agent(
            `Try to refute this bug claim. Default to refuted=true when the evidence does not hold up.\n` +
              `Claim: ${c.claim}\nWhere: ${c.file} / ${c.symbol}\nScenario: ${c.scenario}\n\nYour angle: ${lens}`,
            { label: `verify:${c.symbol}`, phase: 'Verify', schema: VERDICT },
          ),
        ),
      ).then((votes) => ({ c, votes: votes.filter(Boolean) })),
    ({ c, votes }) => {
      if (votes.filter((v) => v.refuted).length >= 2) return null
      return agent(
        `This bug survived adversarial verification:\n${c.claim}\nWhere: ${c.file} / ${c.symbol}\nScenario: ${c.scenario}\n\n` +
          `Produce: (1) a description a maintainer can act on, (2) a reproduction test written to this project's conventions ` +
          `— a Fix compile-and-run test in src/tests/, an integration test driving the fix binary, or a Rust unit test, whichever fits ` +
          `— (3) a fix proposal naming the root cause and what to change, and (4) a barrier that would have caught this class: ` +
          `a code-review convention when a reader of the diff could have spotted it, a regression test when a specific input pins it, ` +
          `or a runtime assertion when an unstated invariant is the real gap. ` +
          `Add a technique only when the way you found this generalizes past this bug.`,
        { label: `deepen:${c.symbol}`, phase: 'Deepen', schema: REPORT },
      )
    },
  )
  confirmed.push(...survivors.filter(Boolean))
}

return { confirmed, examined: seen.size }
```

The loop stops after two consecutive rounds that surface nothing new, so the tail of rare bugs gets the same attention as the first easy one. Dedup runs against everything seen, including what the refuters killed — otherwise a rejected candidate returns every round and the loop never converges.

## Report

Per bug, most severe first:

- **What it is** — the defect in one or two sentences, with file and symbol, plus the failing scenario and whether the evidence is executed or traced.
- **Reproduction test** — a test in the shape this project uses: a Fix compile-and-run test for language and standard-library behavior (reaching the thing under test from `main`), an integration test running the real `fix` binary for tool behavior, a Rust unit test for compiler internals. Give the test body, and say what it does today versus what it should do.
- **Fix proposal** — the root cause, then what to change. When the root cause is a design gap rather than a line, say so and name the options.
- **Barrier** — see *Recurrence Barriers*.

Close with what the hunt covered: the areas and lenses, the number of candidates examined, and the number the refuters killed. A hunt that found nothing reports that plainly along with its coverage — a clean sweep of a well-worn subsystem is information.

## Recurrence Barriers

Every bug gets one barrier, chosen for what would actually have caught it:

- **A `code-review` convention** — when a reader of the diff could have recognized the mistake from the code alone, **and the mistake is a class rather than an incident**. State the class: the convention has to fire on code a different author writes in a different subsystem, with the bug you found serving as one illustration of it. A convention that names the function, type, or pass where you found it can only be applied by someone who already knows the bug, so it protects nothing. When the class already has a home in an existing convention, extend that one. This is the strongest barrier and also the most expensive: every convention is read in full by every future review, so the set is a shared budget and a rarely-firing entry dilutes the ones that fire often.
- **A regression test** — when a specific input pins the behavior. The default choice, and the right one whenever the bug is about *this* code rather than a class of code.
- **A runtime assertion** — when the real gap is an invariant nobody stated. Assert it where it is established, so a violation stops at its origin instead of surfacing as a wrong answer downstream.

Propose exactly one, and say why the other two are weaker for this bug. Adding a convention for a one-off input, or a test for a mistake that will recur in the next twenty diffs, spends the barrier in the wrong place.

## Techniques That Found Bugs

How to hunt, learned from hunts that worked. A finder reads this section before starting.

**The bar for adding one.** Write an entry when the technique would help a future hunt on **a different subsystem and a different bug**. The test to apply before writing: strip out the bug you just found — does the entry still tell a finder what to do? A technique tied to one function, one type, or one pass fails it and belongs in the bug report instead; it teaches a future finder nothing and costs it attention. Name the mechanism and the class of bug it exposes, and let the bug you found be an illustration rather than the content. When an existing entry already covers the ground, extend that entry rather than adding a neighbor. When an entry stops paying, delete it. This section is worth reading only while it stays short.

### Make the impossible case fail loud, then run the suite

Where the code absorbs a case the author believes cannot happen — a catch-all `_ =>`, an `unwrap_or`, a `map_or`, a silent clamp — replace it with a panic and run the tests. When the arm fires, the swallowed case was reachable and something upstream is already broken. Swallowed cases are where latent bugs live, because the wrong value keeps flowing and the failure appears somewhere else entirely. Revert the probe afterwards.

### Run the same program at every optimization level

`fix run -O none`, `-O basic`, `-O max`, `-O experimental` must agree. A difference is a miscompilation by definition, with no judgment call about intent, and it points straight at the pass that differs. This is the cheapest miscompile detector available, and it needs no expected output — the levels check each other.

### Compare against a baseline binary on a large real corpus

Build the compiler at a known-good commit, build it again at the commit under test, and run both over a body of real Fix code — a multi-project library, a bank of solved problems, an external project. Output differences and new build failures surface what small tests miss, because real code combines features in ways a test author never writes down.

### Reproduce on the baseline before blaming the change

Every failure a hunt finds gets run against the merge base or the unmodified upstream first. Pre-existing failures, environment noise, and third-party library defects all look exactly like a fresh bug until this step. This is the single highest-yield false-positive filter, and it costs one run.

### Run the emitted programs under valgrind memcheck

Leaks, double frees, and use-after-free produce correct output on a good day, so comparing outputs finds none of them. Memcheck does. Interpret the report against the same baseline: a glibc thread-local pattern or a third-party library's internal allocation will show up identically on unmodified code.

## Hygiene

- **The tree stays clean.** Probes — a panicking arm, a temporary definition added to `std.fix`, a debug print — are reverted by the agent that made them, and the orchestrator checks `git status --porcelain` before reporting.
- **Builds run in release.** `cargo test --release`, and only the optimization levels the target can affect.
- **The machine is shared.** A sweep that builds a corpus at several optimization levels saturates the machine; run it when the machine is idle, and say in the report that the timing matters if any measurement is part of the evidence.

## The Hunt Log

Kept in the session memory directory (the path is in the memory instructions the orchestrator already carries) as a memory file named `bug-hunt-log`, type `project`:

- One line per hunt: date, target, lenses, commit, how many bugs were confirmed.
- One line per dismissed candidate: what was claimed, and why the refuters killed it. This is what keeps a periodic hunt from re-reporting the same non-bug every time. A dismissed candidate returns only with evidence the refuters did not have.
- One line per confirmed bug that the author left unfixed, so the next hunt reports it as known rather than new.

## What NOT to do

- Don't edit the code under test, and don't commit. The report is the deliverable.
- Don't report a candidate without a concrete failing scenario, however plausible the reasoning reads.
- Don't leave a probe in the tree.
- Don't re-raise a dismissed candidate without new evidence.
- Don't add a `code-review` convention for a bug that a regression test pins better — the review skill is read in full by every aspect subagent, so its conventions are a shared budget.
- Don't write a convention or a technique whose subject is one incident. Both sections are instructions to future agents that will meet different code; anything that only makes sense next to the bug at hand degrades them.
