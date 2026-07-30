"""Searches for an LLVM pass pipeline that runs the benchmark cases in fewer cycles.

The objective is `cycles:u`, the core cycles a program spends in user mode, rather than the
instruction count `benchmark/speedtest` tracks. The two disagree often enough to send a search the
wrong way: appending the twelve passes the compiler used to ship costs `levenshtein` 2.4% of its
instructions while saving 0.4% of its cycles, and saves `fannkuch_scratch` 2.2% of its cycles while
leaving its instruction count where it was. What a pass changes is usually the shape of the code —
where the branches fall, how the front end fetches it — which the instruction count cannot see.

Cycles are not deterministic, so the measurement is built to survive that:

- A candidate is always measured **against the incumbent in the same run**, alternating between the
  two binaries, so that whatever else the machine is doing lands on both.
- Each figure is the **minimum** over several rounds — the round least disturbed by the rest of the
  machine.
- A candidate has to beat the incumbent by more than `IMPROVEMENT`, which sits above the noise the
  `--noise` mode measures, so that noise alone cannot promote a pipeline.

Perf costs about as much as running the program, where cachegrind costs fifty times that, so the
search covers far more candidates per hour than the instruction-count version did.

    python3 passes_optimizer.py            # search until a line is typed on stdin
    python3 passes_optimizer.py --noise    # measure the noise floor and exit

The pipeline the compiler ships lives in `Configuration::llvm_passes`; adopting a result means
editing that. The file this hands to `--llvm-passes-file` is the complete pipeline — it replaces the
passes the optimization level implies, so a candidate lists every pass it wants run.
"""

import math
import os
import random
import re
import select
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent
CASES_DIR = REPO / "benchmark/speedtest/cases"
FIX = REPO / "target/release/fix"
LOG_FILE = REPO / "passes_optimizer.log"

# Where the best sequence found so far is recorded.
LLVM_PASSES_BEST_FILE = REPO / "llvm_passes_best.txt"

# The pipeline the compiler ships, which the search starts from. Must stay in sync with
# `LLVM_O3_PIPELINE` x `LLVM_O3_RUNS_FOR_SPEED` in `src/configuration.rs`.
INITIAL_PASSES = ["default<O3>"] * 3

# The cases the search optimizes. Each runs long enough that process start-up is lost in it, and
# together they cover array loops, allocation, recursion, floating point and string work.
SEARCH_CASES = [
    "arrayrw", "binary_trees", "fannkuch", "fannkuch_scratch", "fib", "levenshtein",
    "mandelbrot", "nbody", "sort", "cp_lib_lsegtree",
]

# Measured only when a candidate becomes the new optimum, and never used to choose one. A pipeline
# that wins on the search cases and loses on these was fitted to the search cases.
HOLDOUT_CASES = ["cp_lib_scc", "cp_lib_dijkstra", "cp_lib_segtree", "nbody_fold", "get_sub"]

# Rounds of the alternating measurement. The minimum over this many is what a figure reports.
ROUNDS = 12

# How much a candidate has to win by, as a ratio of the geometric mean of its per-case cycle counts
# to the incumbent's, and it has to win by that twice over independent rounds before it is promoted.
# Three runs of `--noise` — two builds of one pipeline, so every difference is noise — put the
# geometric mean within 0.07% of 1, with single cases reaching 0.6%. One confirmation at 0.3% is
# already four times that spread, and requiring a second is what keeps a long search from promoting
# the one candidate in a hundred that drew a good pair of rounds.
IMPROVEMENT = 0.997

# Up to this many passes are appended in one add phase.
ADDED_PASSES_NUM = 10

# The measured command gets a fixed environment, so that start-up costs the same whatever shell the
# search was launched from.
MEASURE_ENV = {"PATH": "/usr/bin:/bin", "LC_ALL": "C"}

# Candidate passes, from `opt --print-passes`. See also:
# https://gist.github.com/gingerBill/d889ae03d429653a4a9081ad6dc2a6c3
# Excluded because they may break the program: attributor, attributor-cgscc, unify-loop-exits.
# `reg2mem` is excluded as well: the allocas it introduces escape into indirect tail calls, which
# costs the program its guaranteed tail calls and overflows the stack.
PASSES = '''
default<O3>
aa-eval
adce
add-discriminators
aggressive-instcombine
alignment-from-assumptions
always-inline
annotation-remarks
annotation2metadata
argpromotion
assume-builder
assume-simplify
bdce
break-crit-edges
called-value-propagation
callsite-splitting
canon-freeze
canonicalize-aliases
chr
consthoist
constmerge
constraint-elimination
correlated-propagation
dce
deadargelim
dfa-jump-threading
div-rem-pairs
dse
early-cse
elim-avail-extern
flattencfg
float2int
forceattrs
function-attrs
globaldce
globalopt
globalsplit
guard-widening
gvn
gvn-hoist
gvn-sink
hotcoldsplit
indvars
infer-address-spaces
inferattrs
inject-tli-mappings
inline
inliner-wrapper
inliner-wrapper-no-mandatory-first
instcombine
instsimplify
internalize
irce
iroutliner
jump-threading
lcssa
libcalls-shrinkwrap
licm
lnicm
load-store-vectorizer
loop-bound-split
loop-data-prefetch
loop-deletion
loop-distribute
loop-flatten
loop-fusion
loop-idiom
loop-instsimplify
loop-interchange
loop-load-elim
loop-predication
loop-reduce
loop-reroll
loop-rotate
loop-simplify
loop-simplifycfg
loop-sink
loop-unroll
loop-unroll-and-jam
loop-unroll-full
loop-vectorize
loop-versioning
loop-versioning-licm
lower-constant-intrinsics
lower-expect
lower-guard-intrinsic
lower-matrix-intrinsics
lower-widenable-condition
mem2reg
memcpyopt
mergefunc
mergeicmps
mergereturn
mldst-motion
move-auto-init
nary-reassociate
newgvn
partial-inliner
partially-inline-libcalls
reassociate
recompute-globalsaa
redundant-dbg-inst-elim
rel-lookup-table-converter
rpo-function-attrs
scalarize-masked-mem-intrin
scalarizer
scc-oz-module-inliner
sccp
separate-const-offset-from-gep
simple-loop-unswitch
simplifycfg
sink
slp-vectorizer
slsr
speculative-execution
sroa
strip-dead-prototypes
tailcallelim
tlshoist
typepromotion
vector-combine
'''


def all_passes():
    """Every pass the search may add, one per line of `PASSES`."""
    return [p for p in (line.strip() for line in PASSES.split("\n")) if p]


def build(passes, cases, out_dir):
    """Builds each case with `passes` as its whole pipeline, into `out_dir`.

    Returns the case-to-binary map, or `None` when any case fails to build — a pipeline that cannot
    compile the suite is out of the running, and reporting which case broke is what tells the
    searcher whether the pass or the program is at fault.
    """
    out_dir.mkdir(parents=True, exist_ok=True)
    passes_file = out_dir / "passes.txt"
    passes_file.write_text("\n".join(passes) + "\n")
    binaries = {}
    for case in cases:
        binary = out_dir / case
        result = subprocess.run(
            [str(FIX), "build", "-f", "main.fix", "-O", "experimental",
             "--allow-preliminary-commands", "--llvm-passes-file", str(passes_file),
             "-o", str(binary)],
            cwd=CASES_DIR / case, capture_output=True)
        if result.returncode != 0 or not binary.exists():
            print(f"  {case} failed to build: "
                  f"{result.stderr.decode()[-200:].strip()}", flush=True)
            return None
        binaries[case] = binary
    return binaries


def cycles(binary):
    """User-mode core cycles for one run of `binary`, with ASLR off."""
    arch = subprocess.check_output(["uname", "-m"]).decode().strip()
    result = subprocess.run(
        ["setarch", arch, "-R", "perf", "stat", "-x,", "-e", "cycles:u", "--", str(binary)],
        env=MEASURE_ENV, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, check=True)
    for line in result.stderr.decode().splitlines():
        field = line.split(",")[0]
        if re.fullmatch(r"\d+", field):
            return int(field)
    raise RuntimeError(f"no cycle count for {binary}: {result.stderr.decode()!r}")


def compare(candidate, incumbent, rounds=ROUNDS):
    """Cycles of `candidate` against `incumbent`, per case and as a geometric mean.

    The two are run alternately within each round, so a change in what else the machine is doing
    reaches both; each case's figure is its minimum over the rounds.
    """
    cases = sorted(candidate)
    best = {case: {"candidate": math.inf, "incumbent": math.inf} for case in cases}
    for _ in range(rounds):
        for case in cases:
            for label, binaries in (("candidate", candidate), ("incumbent", incumbent)):
                best[case][label] = min(best[case][label], cycles(binaries[case]))
    ratios = {case: best[case]["candidate"] / best[case]["incumbent"] for case in cases}
    geomean = math.exp(sum(math.log(r) for r in ratios.values()) / len(ratios))
    return geomean, ratios


def report(geomean, ratios, threshold=0.005):
    """Prints the per-case ratios that moved, then the geometric mean."""
    for case, ratio in sorted(ratios.items(), key=lambda kv: kv[1]):
        if abs(ratio - 1) >= threshold:
            print(f"    {case:<20}{(ratio - 1) * 100:+7.2f}%")
    print(f"    {'geometric mean':<20}{(geomean - 1) * 100:+7.2f}%", flush=True)


def measure_noise(work_dir):
    """Builds the shipped pipeline twice and compares the two, which have identical code.

    Whatever this reports is measurement noise, so `IMPROVEMENT` has to sit outside it.
    """
    print("Building the shipped pipeline twice to measure the noise floor.")
    first = build(INITIAL_PASSES, SEARCH_CASES, work_dir / "noise_a")
    second = build(INITIAL_PASSES, SEARCH_CASES, work_dir / "noise_b")
    if first is None or second is None:
        sys.exit("the shipped pipeline failed to build")
    geomean, ratios = compare(first, second)
    print(f"  noise floor over {ROUNDS} rounds (two builds of one pipeline):")
    report(geomean, ratios, threshold=0.0)
    print(f"  IMPROVEMENT is {IMPROVEMENT}, i.e. {(IMPROVEMENT - 1) * 100:+.2f}%")


def log(phase, geomean, passes):
    with open(LOG_FILE, "a") as f:
        f.write(f'{phase},{geomean:.5f},"{",".join(passes)}"\n')


def interrupted():
    """True once a line has been typed on stdin, which is how the search is stopped."""
    readable, _, _ = select.select([sys.stdin], [], [], 0)
    return bool(readable and sys.stdin.readline().strip())


def optimize(work_dir):
    pool = all_passes()
    optimum = list(INITIAL_PASSES)
    optimum_dir = work_dir / "optimum"
    optimum_binaries = build(optimum, SEARCH_CASES, optimum_dir)
    if optimum_binaries is None:
        sys.exit("the starting pipeline failed to build")

    with open(LOG_FILE, "a") as f:
        f.write(f"Start optimization at {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
    print("Starting from:", ", ".join(optimum))

    phase = 0
    while not interrupted():
        phase += 1
        # Alternate between adding passes and dropping one, so that the pipeline can grow toward a
        # win and then shed whatever turned out not to be carrying it.
        if phase % 2 == 1:
            added = [random.choice(pool) for _ in range(random.randint(1, ADDED_PASSES_NUM))]
            candidate = optimum + added
            print(f"\nPhase {phase}: adding {', '.join(added)}")
        else:
            if len(optimum) <= 1:
                continue
            dropped = random.randrange(len(optimum))
            candidate = optimum[:dropped] + optimum[dropped + 1:]
            print(f"\nPhase {phase}: dropping {optimum[dropped]}")

        candidate_dir = work_dir / f"candidate{phase}"
        candidate_binaries = build(candidate, SEARCH_CASES, candidate_dir)
        if candidate_binaries is None:
            shutil.rmtree(candidate_dir, ignore_errors=True)
            continue

        geomean, ratios = compare(candidate_binaries, optimum_binaries)
        report(geomean, ratios)
        # A dropped pass is kept out on a tie: a shorter pipeline that measures the same is the
        # better one, and it gives the next add phase room.
        accepted = geomean < IMPROVEMENT if phase % 2 == 1 else geomean <= 1.0
        if accepted and phase % 2 == 1:
            confirm_geomean, confirm_ratios = compare(candidate_binaries, optimum_binaries)
            print("  confirming:")
            report(confirm_geomean, confirm_ratios)
            accepted = confirm_geomean < IMPROVEMENT
        if not accepted:
            shutil.rmtree(candidate_dir, ignore_errors=True)
            continue

        print("  accepted", flush=True)
        holdout = build(candidate, HOLDOUT_CASES, candidate_dir / "holdout")
        holdout_incumbent = build(optimum, HOLDOUT_CASES, optimum_dir / "holdout")
        if holdout is not None and holdout_incumbent is not None:
            holdout_geomean, holdout_ratios = compare(holdout, holdout_incumbent)
            print("  held-out cases (not used to choose):")
            report(holdout_geomean, holdout_ratios)

        shutil.rmtree(optimum_dir, ignore_errors=True)
        optimum, optimum_dir, optimum_binaries = candidate, candidate_dir, candidate_binaries
        log(phase, geomean, optimum)
        LLVM_PASSES_BEST_FILE.write_text("\n".join(optimum) + "\n")
        print("  current optimum:", ", ".join(optimum), flush=True)


def main():
    if not FIX.exists():
        sys.exit(f"no fix binary at {FIX} -- build one with `cargo build --release`")
    with tempfile.TemporaryDirectory(prefix="passes_optimizer_") as tmp:
        work_dir = Path(tmp)
        if "--noise" in sys.argv:
            measure_noise(work_dir)
            return
        print(f"Type a line and press enter to stop. Load is {os.getloadavg()[0]:.2f}; "
              f"cycles are worth measuring on a quiet machine.")
        optimize(work_dir)


main()
