---
name: bug-hunt
description: "Hunt for latent bugs in a chosen target with three finder subagents, kill the false positives by refuting each candidate, and report every survivor with a reproduction test, a fix proposal, and a recurrence barrier. It never fixes the code under test and never commits to its working branch; a hypothesis test that passes is kept as a regression test on a dedicated branch. Use when: sweeping a subsystem for defects, auditing before a merge or release, or running the periodic hunt."
argument-hint: "Target (a subsystem path, a branch's diff, the standard library, or the whole compiler) and optionally the angles to search from. If omitted, the skill asks."
---

# Bug Hunt

Find bugs that are already in the code and that nobody is looking for. The deliverable is a report: for each bug, what it is, a test that reproduces it, a fix the author can weigh, and a barrier that stops the class from coming back.

This skill **never fixes the code under test and never commits to its working branch**. A compiler fix needs the author's judgment, and the hunt runs unattended often enough that silent fixes would be dangerous. A throwaway probe — a panicking arm, a temporary definition, a debug print — is reverted by whoever made it. A *test* is different: when the hunt suspects a bug, writes a test in the project's idiom to trigger it, and the test **passes** — the suspected bug is absent — that green test pins an invariant the wave just confirmed, so it is worth keeping rather than discarding. Those the orchestrator commits to a **dedicated branch** in its own worktree, so the working branch under test stays clean, and drops any that a test already in the suite covers. Its writes, then, are: passing hypothesis tests on that dedicated branch, its own *Techniques That Found Bugs* section, and the hunt log in memory.

`code-review` is the complement: it applies conventions to a diff, in one pass, and it edits. A hunt is shaped differently — most of what a search turns up is wrong and has to be killed before it reaches the user, and bugs run out only when repeated search stops finding new ones. One hunt is one wave of that search, small enough to run often; the hunt log is what makes the waves add up.

## Target and Lens

The invoker names the target, and optionally the lens. When either is missing, ask with `AskUserQuestion` before starting.

**Targets** — anything with a boundary the hunt can enumerate:

- A subsystem path (`src/rc_ir/`, `src/typecheck.rs`, the LSP server, the package manager, the documentation generator).
- The diff of a branch or a range of commits.
- The Fix standard library (`std.fix`) and the behavior `Document.md` promises for it — a documented behavior the compiler does not deliver is a bug in one of the two.
- The whole compiler, which the scout pass then splits into areas.

**Lenses** — a lens is a search angle handed to one finder, so that the three look for different things instead of converging on the same shallow three bugs. A lens is powerful for the same reason it is dangerous: a finder told to look for boundary bugs will find boundary bugs, and will walk past everything else. So a hunt **derives its two lenses from the target** rather than picking them off a list, and gives the third finder no lens at all:

- In the scout pass, read the target and write down what it is responsible for: the invariants it maintains, the inputs it accepts, the guarantees its callers rely on, the ways it can fail silently. Each of those is a candidate lens, phrased as a question about *this* target.
- Take two that are blind to each other. Two lenses that would read the same code the same way are one lens.
- **The third finder runs unlensed**, told to ignore the angles the hunt chose and report whatever is actually wrong. It is the check on the lens set itself, and comparing its yield against the other two says whether the lenses are helping or narrowing.

Recurring angles look roughly like the following. Treat them as calibration for how wide a lens should be, and let the target dictate the actual set — a hunt that only ever asks the questions on this list will only ever find the classes of bug already known to this project:

- Miscompilation — a valid program compiled into code that computes the wrong thing.
- Memory and reference counting — leaks, double frees, use-after-free, an ownership declaration that disagrees with what the code does.
- Boundary and edge cases — empty input, one element, first and last, the recursion base case, overflow, the degenerate shape.
- Invariants — something the code relies on without checking, and a path that violates it.
- Error and failure paths — the arm nobody runs on input nobody sends.
- Tool behavior — the build cache, dependency resolution, the LSP answers, the generated documentation.

The hunt log holds the angles previous hunts used and what each returned. Start somewhere else: a lens that has been run twice with nothing to show is spent, and the classes named by the last hunt's completeness critic are the strongest candidates for this one.

## Severity

Report most severe first, and use this ordering when the budget forces a choice about what to chase. It ranks the classes seen most often; a bug that fits none of them is ranked by the damage it does before it is noticed, which is what the ordering is really made of.

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

A hunt is **three finder subagents and the orchestrator**. Keeping the fan-out at three is what makes the hunt cheap enough to run on a schedule, and the schedule is where the depth comes from: one hunt is one wave, and the hunt log carries what has been searched from wave to wave.

1. **Resolve the target and the angles.** Ask with `AskUserQuestion` when the invocation left either open. Confirm the working tree is clean (`git status --porcelain`) and note the current commit — the hunt reports against that state.
2. **Read the hunt log** from memory: what previous hunts covered, which angles they used and what each returned, which candidates were dismissed and why, and which confirmed bugs are still unfixed. A dismissed candidate is re-raised only with evidence the refutation did not have.
3. **Scout, inline.** Read enough of the target to split it into areas and to derive the angles (see *Target and Lens*). Then fix the three assignments: **two derived angles, and one unlensed finder**. Each gets the areas it owns, so that together they cover the target. Report the assignment before launching, so a mis-scoped hunt is caught in seconds rather than after three subagents finish.
4. **Launch the three finders in parallel** with the `Agent` tool, in a single block, and wait for all three. Brief each with: the target and its areas, its angle (or, for the third, the instruction to ignore the hunt's angles and report whatever is actually wrong), the *Evidence Bar*, the *Techniques That Found Bugs* section, the dismissed candidates from the log, and the rule that a finder leaves its working tree clean — a throwaway probe is reverted, and a hypothesis test that passes is handed back in the finder's report (its full body, and where in the suite it belongs) before the finder reverts it, so the orchestrator can re-land it on the dedicated branch. Each returns candidates (file, symbol, claim, failing scenario, and whether the evidence is executed or traced) and, separately, the passing hypothesis tests it wrote.
5. **Verify adversarially, inline.** Take each candidate and try to refute it — this is the orchestrator's main job, and its independence from the finder that produced the candidate is what makes the check real. For each: can any input actually reach that path; assuming it is reached, is the result genuinely wrong; and does it reproduce when you build and run it — on the commit under test *and* on the merge-base or unmodified upstream, since a pre-existing failure, environment noise, or a third-party defect looks identical to a fresh bug until that one run? Default to dropping the candidate when the evidence does not hold. Deduplicate what survives against the log and against the other finders.
6. **Deepen each survivor, inline**: the four deliverables under *Report*.
7. **Critique the coverage.** With all three reports in hand, name the classes of bug that these angles could not have surfaced, whatever their yield. That answer goes in the report and becomes the strongest candidate angle for the next hunt.
8. **Commit the tests worth keeping.** Collect the passing hypothesis tests the finders handed back, drop any that a test already in the suite covers and any duplicated among the finders, and commit the survivors to a **dedicated branch** in its own worktree. That branch is a deliverable beside the report; name it there.
9. **Report**, then **append** any technique that earns it, then **update the hunt log**.
10. **Leave the working tree as you found it.** Verify `git status --porcelain` is empty on the branch under test, and that every probe is reverted. The kept tests live on the dedicated branch, not in the working tree.

## Report

Per bug, most severe first:

- **What it is** — the defect in one or two sentences, with file and symbol, plus the failing scenario and whether the evidence is executed or traced.
- **Reproduction test** — a test in the shape this project uses: a Fix compile-and-run test for language and standard-library behavior (reaching the thing under test from `main`), an integration test running the real `fix` binary for tool behavior, a Rust unit test for compiler internals. Give the test body, and say what it does today versus what it should do.
- **Fix proposal** — the root cause, then what to change. When the root cause is a design gap rather than a line, say so and name the options.
- **Barrier** — see *Recurrence Barriers*.

Close with what the hunt covered: the areas, the three angles and what each returned, how many candidates were examined and how many the refutation killed, and the classes of bug these angles could not have surfaced. A hunt that found nothing reports that plainly along with its coverage — a clean sweep of a well-worn subsystem is information, and so is an unlensed finder that out-yields both lenses.

Name the dedicated branch that carries the passing hypothesis tests, and say in one line what each pins. A green test for an invariant the hunt suspected and confirmed is as much a product of the wave as a red one that found a bug — a hunt that fixed nothing can still leave the suite stronger than it found it.

## Recurrence Barriers

Every bug gets one barrier, chosen for what would actually have caught it:

- **A `code-review` convention** — when a reader of the diff could have recognized the mistake from the code alone, **and the mistake is a class rather than an incident**. State the class: the convention has to fire on code a different author writes in a different subsystem, with the bug you found serving as one illustration of it. A convention that names the function, type, or pass where you found it can only be applied by someone who already knows the bug, so it protects nothing. When the class already has a home in an existing convention, extend that one. This is the strongest barrier and also the most expensive: every convention is read in full by every future review, so the set is a shared budget and a rarely-firing entry dilutes the ones that fire often.
- **A regression test** — when a specific input pins the behavior. The default choice, and the right one whenever the bug is about *this* code rather than a class of code.
- **A runtime assertion** — when the real gap is an invariant nobody stated. Assert it where it is established, so a violation stops at its origin instead of surfacing as a wrong answer downstream. A cheap check asserts unconditionally; one whose cost is comparable to the work it guards runs under `config.develop_mode`, which the unit tests enable. `debug_assert!` has no place here: the suite runs in release, where it compiles away. An assertion is for an internal invariant — a condition a user's source file can violate belongs in the diagnostic path as an error instead.

Propose exactly one, and say why the other two are weaker for this bug. Adding a convention for a one-off input, or a test for a mistake that will recur in the next twenty diffs, spends the barrier in the wrong place.

## Techniques That Found Bugs

How to hunt. A finder reads this section before starting. Two kinds of entry: a **smell** — a pattern you spot by reading the code, together with the probe that turns it into a reproduced bug — and a **detector** — the tool that shows whether a probe's result is wrong. A hunt reads for smells and confirms with detectors.

**The bar for adding one.** An entry earns its place when it would help a future hunt on **a different subsystem and a different bug**: it names a mechanism and the class of bug it exposes, with any concrete case serving as illustration rather than content. A smell need not have caught a bug yet — a well-grounded one, naming a real pattern and a concrete probe, is worth writing down before a hunt proves it. But an entry that stops paying is deleted: a smell repeated hunts never exploit, a detector that never fires, or one tied to a single function, type, or pass, which teaches a future finder nothing and costs it attention. When an existing entry already covers the ground, extend it. This section is worth reading only while it stays short.

### Smells

#### Make an unenforced invariant fail loud, then run the suite

The code leans on a condition it never checks — a catch-all `_ =>`, an `unwrap_or`, a `map_or`, or a silent clamp that absorbs a case the author calls impossible, or a comment asserting "same length" / "never empty" / "always last" with no assertion behind it. Turn the assumption into a loud failure — replace the fallback with a `panic!`, or add the `assert!` the comment implies — and run the suite. When it fires, the case was reachable and something upstream is already broken; the swallowed value was flowing on and would have surfaced somewhere else entirely. Revert the probe afterward.

#### Exploit a fact stored in two places

The same information lives in two representations kept in sync by convention rather than construction — a length beside the data it measures, an operand name embedded in an operation and repeated in its separate argument list, a cached field derivable from its source, a tag order that a switch's default silently assumes. Force or find a state where the two disagree: trace whether any pass updates one without the other, or add an assertion that the two are equal and run the suite. Then check whether a consumer reads the stale copy — a desync a consumer trusts is a use-after-free, a miscompile, or a wrong answer waiting for the first pass that breaks the convention.

#### Exploit a declaration divorced from the code that must honor it

A hand-written declaration binds a separate code path with no compiler link between them — an ownership or borrow annotation a pass must match, an arity or field count, the order a match's arms must keep. Derive both the declaration and the code from the same monomorphized source and assert they agree, or inject a deliberate mismatch and see whether anything catches it before code generation. Where the two can drift, the drift is the bug.

#### Exploit a path no test reaches

Untested code is where a latent bug survives, because a tested path carrying a bug would already have failed — so a gap in the suite is a map to where the bugs are. Enumerate the cases the target handles — the match arms, the error branches, the boundaries (empty, one element, the degenerate shape), the opt-level and config combinations — and cross off the ones a test exercises; a coverage tool (`cargo-llvm-cov`) mechanizes the same census. Craft the input that drives execution into what is left, run it, and read the result with a detector. This is the `test-sufficiency` review lens turned offensive: that aspect flags the gap for the author, a hunt shoots into it.

### Detectors

#### Run the same program at every optimization level

`fix run -O none`, `-O basic`, `-O max`, `-O experimental` must compute the same result. When two levels both complete and return different values, that is a miscompilation by definition — no judgment call about intent — and it points straight at the pass that differs; it needs no expected output, the levels check each other. Compare the *result*, not the *run*: `-O none` and `-O basic` are deliberately weak — they skip tail-call optimization, so a deeply tail-recursive program overflows the stack there while `-O max` runs it, and they can let an `O(n)` program degrade to `O(n²)`. A stack overflow or a hang at the lower levels is that known weakness, not a miscompile — take `-O max` / `-O experimental` as the reference, and read a divergence as a bug only when a completing run returns the wrong value.

#### Run the emitted programs under valgrind memcheck

Leaks, double frees, and use-after-free produce correct output on a good day, so comparing outputs finds none of them. Memcheck does. Interpret its report against a baseline: a glibc thread-local pattern or a third-party library's internal allocation shows up identically on unmodified code.

#### Run threaded programs under a data-race detector

Output comparison and memcheck both run single-threaded and miss data races — two threads racing on a refcount or on shared state produce the right answer on a good day. A data-race detector (ThreadSanitizer, or valgrind's helgrind / DRD) is the one sanitizer routine runs skip, because its false-positive rate is high; that cost is why the concurrency class stays unhunted, and paying it is how a hunt reaches races nothing else does. Triage against a baseline exactly as with memcheck: a glibc or library-internal race pattern shows identically on unmodified code.

## Hygiene

- **The working tree stays clean.** A probe — a panicking arm, a temporary definition added to `std.fix`, a debug print — is reverted by the agent that made it. A hypothesis test that passes is not a probe: the orchestrator lands it on the hunt's dedicated branch (deduped against the existing suite), never in the branch under test. The orchestrator checks `git status --porcelain` on the branch under test before reporting.
- **Builds run in release.** `cargo test --release`, and only the optimization levels the target can affect.
- **The machine is shared.** A sweep that builds a corpus at several optimization levels saturates the machine; run it when the machine is idle, and say in the report that the timing matters if any measurement is part of the evidence.

## The Hunt Log

One hunt is one wave, so the log is what turns a schedule of small hunts into a search that keeps going. Kept in the session memory directory (the path is in the memory instructions the orchestrator already carries) as a memory file named `bug-hunt-log`, type `project`:

- One line per hunt: date, target, commit, the three angles, and what each returned. A lens that has now returned nothing twice is spent — retire it, and say so on the line.
- One line per dismissed candidate: what was claimed, and why the refutation killed it. This is what keeps a periodic hunt from re-reporting the same non-bug every time. A dismissed candidate returns only with evidence the refutation did not have.
- One line per confirmed bug the author left unfixed, so the next hunt reports it as known rather than new.
- The classes the coverage critique named. They are the first place the next hunt looks for its angles.

## What NOT to do

- Don't fix the code under test, and don't commit to its working branch. The report — and the dedicated branch of passing tests — is the deliverable.
- Don't report a candidate without a concrete failing scenario, however plausible the reasoning reads.
- Don't leave a probe in the tree.
- Don't discard a passing hypothesis test that no existing test covers — it is a regression test the wave earned; land it on the dedicated branch.
- Don't re-raise a dismissed candidate without new evidence.
- Don't grow the hunt past three finders. The depth of this hunt comes from running it again, not from spending more on one wave.
- Don't let the recurring-angle list stand in for the scout pass. Angles derived from the target find what a fixed menu cannot.
- Don't add a `code-review` convention for a bug that a regression test pins better — the review skill is read in full by every aspect subagent, so its conventions are a shared budget.
- Don't write a convention or a technique whose subject is one incident. Both sections are instructions to future agents that will meet different code; anything that only makes sense next to the bug at hand degrades them.
