"""Read the hardware counters over the fifty whole programs of LangArena.

`../speedtest` measures programs written to isolate one thing the compiler does. These are whole
programs — a JSON parser, a Huffman coder, a maze solver, a raytracer — each checking its own
answer against a checksum. A change the speedtest cases call neutral can still move these, and a
change that moves these is one a user feels.

`../speedtest/main.fix` is what drives this and what writes the log, the way it drives
`perf_counters.py` and `reference.py` over its own cases. This file is what that driver cannot do
itself: put the programs on the disk, build them with the compiler under test, and read the
counters over each one. It writes no log of its own.

The programs are not kept here. They live in `LangArena`, cloned at the revision `PINNED_REVISION`
names: a benchmark whose source moves takes its numbers with it, and a row measured before that
move would then be compared against a row measured after it.

Every figure comes from `../speedtest/perf_counters.py`, so a LangArena column and a speedtest
column of one row mean the same thing: instructions retired in user space, accesses that reached
main memory, loads and stores that crossed a cache-line boundary, and user-space core cycles, each
the lowest reading over `--windows` windows. The cycle field is empty where other work could have
moved it.

One program is measured per process, which the counters need, and the whole process is counted.
Starting the process, reading the configuration and writing the answer come to 954 thousand
instructions and 630 thousand cycles, against 1.06 billion and 1.48 billion for the shortest of the
fifty, so what a column holds is the program.

    python3 bench.py prepare [--work-dir DIR]
    python3 bench.py build --fix PATH [--work-dir DIR] [--opt-level LEVEL]
    python3 bench.py measure [--work-dir DIR] [--windows N] [--only NAME ...]

`measure` prints one line per program, `<name>,<instructions>,<ram>,<splits>,<cycles>,<contention>`,
which is `perf_counters.py`'s own line with the program's name in front of it. Measuring all fifty
takes about ten minutes at five windows, against the two minutes the speedtest cases take, so the
driver asks for them only when it is given `--langarena`.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys

# The repository the programs come from, and the revision every run measures. Pinning is what makes
# two rows comparable: a change to a benchmark's source moves its numbers with no compiler change
# behind it. Raise the revision deliberately, and say in `../speedtest/history.md` which rows sit on
# which side.
REPO_URL = "https://github.com/tttmmmyyyy/LangArena.git"
PINNED_REVISION = "53067d51df3061c7cebf6b22b1d5c63c4ffdab69"

# Where the clone lives. Outside the fixlang tree, so that a benchmark run leaves nothing behind in
# it and so that the build output a measured project accumulates sits somewhere a user expects to
# find it.
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

# The optimization level the programs are built at, which is the one LangArena's own runner uses.
DEFAULT_OPT_LEVEL = "max"

# How many windows of runs the hardware counters are read over. Every figure is the lowest of them,
# and other work only ever raises one.
DEFAULT_WINDOWS = 5

# The counter script, shared with `../speedtest` so that a LangArena column and a speedtest column
# hold the same measurement.
PERF_COUNTERS = os.path.join(
    os.path.dirname(os.path.abspath(__file__)), "..", "speedtest", "perf_counters.py"
)

# What `perf_counters.py` exits with when the counters could not be read, and when the program it
# measured exited non-zero. A LangArena program checks its own answer against a checksum, so the
# second is a program whose answer moved.
COUNTERS_UNAVAILABLE = 1
PROGRAM_FAILED = 2

# How many fields `perf_counters.py` prints: the four counts and the contention beside them.
COUNTER_FIELDS = 5


def project_dir(work_dir):
    """The Fix project inside the clone, which is what `build` builds and `measure` runs."""
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
    """Put the programs where `build` can find them, at the pinned revision."""
    os.makedirs(args.work_dir, exist_ok=True)
    repo = os.path.join(args.work_dir, "LangArena")
    if not os.path.isdir(os.path.join(repo, ".git")):
        run_or_die(["git", "clone", REPO_URL], cwd=args.work_dir, what="git clone")
    run_or_die(["git", "fetch", "--quiet", "origin"], cwd=repo, what="git fetch")
    run_or_die(
        ["git", "checkout", "--quiet", PINNED_REVISION], cwd=repo, what="git checkout"
    )
    print("%s at %s" % (repo, PINNED_REVISION))


def check_clone(work_dir):
    """Stop where the clone is missing or stands at a revision other than the pinned one.

    A clone prepared before the pin was raised holds programs whose numbers belong to that older
    revision, and a run over them would answer for the compiler under test and the benchmark
    sources at once.
    """
    directory = project_dir(work_dir)
    if not os.path.isdir(directory):
        sys.exit("no programs at %s — run `python3 bench.py prepare` first" % directory)
    repo = os.path.join(work_dir, "LangArena")
    proc = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True
    )
    if proc.returncode != 0 or proc.stdout.strip() != PINNED_REVISION:
        sys.exit(
            "%s stands at %s, not the pinned %s — run `python3 bench.py prepare`"
            % (repo, proc.stdout.strip() or "no revision", PINNED_REVISION)
        )
    return directory


def binary_path(work_dir):
    """Where `build` puts the program and `measure` looks for it."""
    return os.path.join(project_dir(work_dir), BIN_SUBDIR, BIN_NAME)


def build(args):
    """Build the programs with the compiler under test, and print where the program went."""
    directory = check_clone(args.work_dir)
    fix = os.path.abspath(args.fix)
    if not os.path.exists(fix):
        sys.exit("no compiler at %s" % fix)
    for path in COLD_PATHS:
        shutil.rmtree(os.path.join(directory, path), ignore_errors=True)
    os.makedirs(os.path.join(directory, BIN_SUBDIR), exist_ok=True)
    run_or_die(
        [
            fix,
            "build",
            "--allow-preliminary-commands",
            "-O",
            args.opt_level,
            "-o",
            os.path.join(BIN_SUBDIR, BIN_NAME),
        ],
        cwd=directory,
        what="fix build",
    )
    print(binary_path(args.work_dir))


def benchmark_names(work_dir):
    """The programs the configuration names, in the order it lists them.

    A name is handed to the runner as a filter, which selects the programs whose name holds it. No
    name here is part of another, so each one selects itself alone.
    """
    with open(os.path.join(project_dir(work_dir), CONFIG)) as handle:
        return [entry["name"] for entry in json.load(handle)]


def measure(args):
    """Print what the counters say about each program, one line per program."""
    directory = check_clone(args.work_dir)
    binary = binary_path(args.work_dir)
    if not os.path.exists(binary):
        sys.exit("no program at %s — run `python3 bench.py build` first" % binary)

    names = benchmark_names(args.work_dir)
    if args.only:
        unknown = [name for name in args.only if name not in names]
        if unknown:
            sys.exit("no such benchmark: %s" % ", ".join(unknown))
        names = [name for name in names if name in args.only]

    for name in names:
        proc = subprocess.run(
            [
                "python3",
                PERF_COUNTERS,
                "--windows",
                str(args.windows),
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
            # A machine with no counters to read leaves every column empty, the way it does for a
            # speedtest case, and the run carries on.
            print("%s,,,,," % name, flush=True)
            continue
        if proc.returncode != 0:
            sys.exit(
                "perf_counters.py failed on %s:\n%s%s" % (name, proc.stdout, proc.stderr)
            )
        fields = proc.stdout.strip().split(",")
        if len(fields) != COUNTER_FIELDS:
            sys.exit("perf_counters.py reported %r for %s" % (proc.stdout, name))
        print("%s,%s" % (name, ",".join(fields)), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--work-dir", default=DEFAULT_WORK_DIR)
    subparsers = parser.add_subparsers(dest="mode", required=True)

    prepared = subparsers.add_parser("prepare")
    prepared.set_defaults(func=prepare)

    built = subparsers.add_parser("build")
    built.add_argument("--fix", required=True)
    built.add_argument("--opt-level", default=DEFAULT_OPT_LEVEL)
    built.set_defaults(func=build)

    measured = subparsers.add_parser("measure")
    measured.add_argument("--windows", type=int, default=DEFAULT_WINDOWS)
    measured.add_argument("--only", nargs="+")
    measured.set_defaults(func=measure)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
