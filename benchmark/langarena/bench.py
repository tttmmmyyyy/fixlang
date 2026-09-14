"""Measure what a Fix program costs to run, over the fifty programs of LangArena.

`../speedtest` measures programs written to isolate one thing the compiler does. These are whole
programs — a JSON parser, a Huffman coder, a maze solver, a raytracer — each checking its own
answer against a checksum, and each with counterparts in the other languages LangArena carries. A
change that the speedtest cases call neutral can still move these, and a change that moves these is
one a user feels.

The programs are not kept here. They live in `LangArena`, cloned at the revision `PINNED_REVISION`
names: a benchmark whose source moves takes its numbers with it, and a row measured before that
move would then be compared against a row measured after it. `prepare` is what puts them where
`run` can find them.

Every figure comes from `../speedtest/perf_counters.py`, so a row here and a row there mean the
same thing: instructions retired in user space, accesses that reached main memory, loads and stores
that crossed a cache-line boundary, and user-space core cycles, each the lowest reading over
`--windows` windows. The cycle field is empty where other work could have moved it, and the
`contention` column says how much of the machine the run had.

One program is measured per process, which the counters need, and the whole process is counted.
Starting the process, reading the configuration and writing the answer come to 954 thousand
instructions and 630 thousand cycles, against 1.06 billion and 1.48 billion for the shortest of the
fifty, so what a row holds is the program.

    python3 bench.py prepare [--work-dir DIR]
    python3 bench.py run --label NAME [--fix PATH] [--windows N] [--only NAME ...]
    python3 bench.py compare BEFORE AFTER

A run takes about ten minutes at five windows. The cycle column is worth having only on a quiet
machine, so ask for the machine before starting one.
"""

import argparse
import csv
import json
import os
import shutil
import subprocess
import sys

# The repository the programs come from, and the revision every run measures. Pinning is what makes
# two rows comparable: a change to a benchmark's source moves its numbers with no compiler change
# behind it. Raise the revision deliberately, and say in `history.md` which rows sit on which side.
REPO_URL = "https://github.com/tttmmmyyyy/LangArena.git"
PINNED_REVISION = "69a97c9374f1dccabdba4e577846c4dcb8793da7"

# Where the clone lives. Outside the fixlang tree, so that a benchmark run leaves nothing behind in
# it and so that the 330 MB of build output a measured project accumulates sits somewhere a user
# expects to find it.
DEFAULT_WORK_DIR = os.path.expanduser("~/fix-langarena-bench")

# Where the Fix implementation and its configuration sit inside that clone. The configuration names
# the fifty programs and fixes the input each one runs on.
PROJECT_SUBDIR = "fix"
CONFIG = os.path.join("..", "run.js")

# Where the built program goes. The directory is not in the repository, so a fresh clone has to be
# given it before the linker is asked to write there.
BIN_SUBDIR = "target"
BIN_NAME = "bin_bench"

# What a build removes before it runs, so that the compiler under test does the work rather than
# reading what another compiler left. `deps` stays: it holds the source of the projects this one
# depends on, and restoring it costs a network fetch.
COLD_PATHS = [
    os.path.join(".fixlang", "cache"),
    os.path.join(".fixlang", "intermediate"),
]

# The optimization level the programs are built at. LangArena's own runner uses this one, so a row
# here and a figure from that runner are about the same program.
DEFAULT_OPT_LEVEL = "max"

# How many windows of runs the hardware counters are read over. Every figure is the lowest of them,
# and other work only ever raises one.
DEFAULT_WINDOWS = 5

# The counter script, shared with `../speedtest` so that the two logs hold the same measurement.
PERF_COUNTERS = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "speedtest", "perf_counters.py"
)

# What `perf_counters.py` exits with when the counters could not be read, and when the program it
# measured exited non-zero. A LangArena program checks its own answer against a checksum, so the
# second is a program whose answer moved.
COUNTERS_UNAVAILABLE = 1
PROGRAM_FAILED = 2

# The columns a row carries. `label` names the compiler the row was measured with and `cpu` the
# processor it ran on, since neither a row from another compiler nor one from another machine means
# anything beside its neighbours without them.
COLUMNS = [
    "label",
    "cpu",
    "benchmark",
    "instructions",
    "ram",
    "splits",
    "cycles",
    "contention",
]

DEFAULT_OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "log.csv")

# Where this measurement's own spread ends, as a percentage. Two runs of one compiler over all
# fifty programs on an idle machine moved the cycle count by at most 1.98% and the median by 0.31%,
# and the instruction count by less than 0.0001%. So a cycle difference under the first figure is
# this measurement rather than the compiler, while any instruction difference at all is real.
CYCLE_NOISE = 2.0
INSTRUCTION_NOISE = 0.001


def project_dir(work_dir):
    """The Fix project inside the clone, which is what `run` builds and measures."""
    return os.path.join(work_dir, "LangArena", PROJECT_SUBDIR)


def run_or_die(command, cwd=None, what=None):
    """Run `command`, and stop with its output where it fails."""
    proc = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.exit(
            "%s failed:\n%s%s" % (what or " ".join(command), proc.stdout, proc.stderr)
        )
    return proc.stdout


def prepare(args):
    """Put the programs where `run` can find them, at the pinned revision."""
    os.makedirs(args.work_dir, exist_ok=True)
    repo = os.path.join(args.work_dir, "LangArena")
    if not os.path.isdir(os.path.join(repo, ".git")):
        run_or_die(["git", "clone", REPO_URL], cwd=args.work_dir, what="git clone")
    run_or_die(["git", "fetch", "--quiet", "origin"], cwd=repo, what="git fetch")
    run_or_die(
        ["git", "checkout", "--quiet", PINNED_REVISION], cwd=repo, what="git checkout"
    )
    print("%s at %s" % (repo, PINNED_REVISION))


def benchmark_names(work_dir):
    """The programs the configuration names, in the order it lists them.

    A name is handed to the runner as a filter, which selects the programs whose name holds it. No
    name here is part of another, so each one selects itself alone.
    """
    config = os.path.join(project_dir(work_dir), CONFIG)
    with open(config) as handle:
        return [entry["name"] for entry in json.load(handle)]


def check_revision(work_dir):
    """Stop where the clone stands at a revision other than the pinned one.

    A clone prepared before the pin was raised holds programs whose numbers belong to that older
    revision, and a run over them would answer for the compiler under test and the benchmark
    sources at once.
    """
    repo = os.path.join(work_dir, "LangArena")
    proc = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True
    )
    if proc.returncode != 0 or proc.stdout.strip() != PINNED_REVISION:
        sys.exit(
            "%s stands at %s, not the pinned %s — run `python3 bench.py prepare`"
            % (repo, proc.stdout.strip() or "no revision", PINNED_REVISION)
        )


def build(fix, work_dir, opt_level):
    """Build the programs with the compiler under test, and answer where the binary went."""
    directory = project_dir(work_dir)
    if not os.path.isdir(directory):
        sys.exit(
            "no programs at %s — run `python3 bench.py prepare` first" % directory
        )
    check_revision(work_dir)
    for path in COLD_PATHS:
        shutil.rmtree(os.path.join(directory, path), ignore_errors=True)
    binary = os.path.join(BIN_SUBDIR, BIN_NAME)
    os.makedirs(os.path.join(directory, BIN_SUBDIR), exist_ok=True)
    run_or_die(
        [fix, "build", "--allow-preliminary-commands", "-O", opt_level, "-o", binary],
        cwd=directory,
        what="fix build",
    )
    return binary


def measure(binary, directory, name, windows):
    """Read the counters over one program, and answer its four counts and the contention.

    Answers `None` for the counts where the counters could not be read, which is a machine without
    them rather than a program without an answer.
    """
    proc = subprocess.run(
        [
            "python3",
            PERF_COUNTERS,
            "--windows",
            str(windows),
            binary,
            CONFIG,
            name,
        ],
        cwd=directory,
        capture_output=True,
        text=True,
    )
    if proc.returncode == PROGRAM_FAILED:
        sys.exit("%s failed its own check:\n%s%s" % (name, proc.stdout, proc.stderr))
    if proc.returncode == COUNTERS_UNAVAILABLE:
        return None, 0.0
    if proc.returncode != 0:
        sys.exit(
            "perf_counters.py failed on %s:\n%s%s" % (name, proc.stdout, proc.stderr)
        )
    fields = proc.stdout.strip().split(",")
    if len(fields) != 5:
        sys.exit("perf_counters.py reported %r for %s" % (proc.stdout, name))
    contention = float(fields[4]) if fields[4] else 0.0
    return fields[:4], contention


def read_cpu(directory):
    """The processor the rows are measured on, or the empty string where it cannot be read."""
    proc = subprocess.run(
        ["python3", PERF_COUNTERS, "--cpu"],
        cwd=directory,
        capture_output=True,
        text=True,
    )
    return proc.stdout.strip() if proc.returncode == 0 else ""


def count(field):
    """A counter field as it is printed: grouped by thousands, or a dash where it is empty."""
    return f"{int(field):,}" if field else "-"


def run(args):
    """Build the programs and write one row per program."""
    fix = args.fix or shutil.which("fix")
    if fix is None:
        sys.exit("no `fix` on the path — pass --fix")
    directory = project_dir(args.work_dir)
    names = benchmark_names(args.work_dir)
    if args.only:
        unknown = [name for name in args.only if name not in names]
        if unknown:
            sys.exit("no such benchmark: %s" % ", ".join(unknown))
        names = [name for name in names if name in args.only]

    binary = build(fix, args.work_dir, args.opt_level)
    cpu = read_cpu(directory)
    print("%s, %s, %s, %d windows" % (args.label, fix, cpu, args.windows))

    new_file = not os.path.exists(args.out)
    max_contention = 0.0
    with open(args.out, "a", newline="") as handle:
        writer = csv.writer(handle)
        if new_file:
            writer.writerow(COLUMNS)
        for name in names:
            counts, contention = measure(binary, directory, name, args.windows)
            max_contention = max(max_contention, contention)
            row = counts if counts is not None else ["", "", "", ""]
            writer.writerow([args.label, cpu, name] + row + [contention])
            handle.flush()
            print("%-28s %16s inst %16s cycles" % (name, count(row[0]), count(row[3])))
    print("wrote %s; the busiest window gave away %.2f cores" % (args.out, max_contention))


def rows_of(out, label):
    """The rows of one label, keyed by benchmark."""
    with open(out) as handle:
        rows = [row for row in csv.DictReader(handle) if row["label"] == label]
    if not rows:
        sys.exit("no rows labelled %r in %s" % (label, out))
    return {row["benchmark"]: row for row in rows}


def compare(args):
    """Print what one label costs against another, program by program."""
    before = rows_of(args.out, args.before)
    after = rows_of(args.out, args.after)
    cpus = {row["cpu"] for row in list(before.values()) + list(after.values())}
    if len(cpus) > 1:
        sys.exit(
            "the two labels were measured on different processors (%s), so their cycle "
            "counts say more about the machines than about the compiler" % ", ".join(cpus)
        )

    print("%-28s %10s %10s" % ("", "inst", "cycles"))
    beyond = {"instructions": 0, "cycles": 0}
    limit = {"instructions": INSTRUCTION_NOISE, "cycles": CYCLE_NOISE}
    compared = 0
    for name, old in before.items():
        new = after.get(name)
        if new is None:
            continue
        compared += 1
        shown = []
        for column in ("instructions", "cycles"):
            if not (old[column] and new[column]):
                shown.append("%10s" % "-")
                continue
            percent = (int(new[column]) / int(old[column]) - 1.0) * 100.0
            shown.append("%+9.2f%%" % percent)
            if abs(percent) > limit[column]:
                beyond[column] += 1
        print("%-28s %s %s" % (name, shown[0], shown[1]))
    print(
        "of %d programs, %d moved the instruction count and %d moved the cycle count past "
        "what two runs of one compiler move it (%.3f%% and %.2f%%)"
        % (compared, beyond["instructions"], beyond["cycles"], INSTRUCTION_NOISE, CYCLE_NOISE)
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", default=DEFAULT_WORK_DIR)
    parser.add_argument("--out", default=DEFAULT_OUT)
    subparsers = parser.add_subparsers(dest="mode", required=True)

    prepared = subparsers.add_parser("prepare")
    prepared.set_defaults(func=prepare)

    ran = subparsers.add_parser("run")
    ran.add_argument("--label", required=True)
    ran.add_argument("--fix", default=None)
    ran.add_argument("--windows", type=int, default=DEFAULT_WINDOWS)
    ran.add_argument("--opt-level", default=DEFAULT_OPT_LEVEL)
    ran.add_argument("--only", nargs="+")
    ran.set_defaults(func=run)

    compared = subparsers.add_parser("compare")
    compared.add_argument("before")
    compared.add_argument("after")
    compared.set_defaults(func=compare)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
