"""Measure what a Fix project costs to compile, over a corpus of real projects.

The figure logged is user-space instructions retired by the `fix` process and every process it
starts, which includes the C compiler and the linker. It is what the compiler did, so two runs of
one command return the same number whatever else the machine is doing: over five runs of a warm
build while eleven other jobs ran, the count spanned 0.04% and the wall clock spanned 75%.

Each project is measured under several scenarios, which differ in what the compiler is allowed to
find already done:

    check_cold   type-check everything, with no cache entry to read
    check_warm   type-check again, with every entry there to read
    build_cold   build, with neither type-check entries nor object files
    build_warm   build again, with both
    build_edit   build after one source file changed, which is what an edit costs

    python3 bench.py prepare [--corpus DIR]      copy the projects and install their dependencies
    python3 bench.py run [--corpus DIR] [--fix PATH] [--reps N] [--out CSV]
"""

import argparse
import csv
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
import time

# Where the projects are copied to. A project is measured in a copy of its own, because measuring
# it in place would leave the caches and the built binary of a benchmark run behind in it.
DEFAULT_CORPUS = os.path.expanduser("~/fix-compile-bench/corpus")

# The directory a Fix project keeps everything the compiler produced in.
DOT_FIXLANG = ".fixlang"

# What a cold run removes: the type-check entries, the object files, and the record of what was
# built. What stays is `deps`, which holds the source of the projects this one depends on and
# costs a network fetch to restore.
COLD_PATHS = [
    os.path.join(DOT_FIXLANG, "cache"),
    os.path.join(DOT_FIXLANG, "intermediate"),
]

# The optimization levels a build is measured at.
OPT_LEVELS = ["none", "basic", "max"]

# How long one measured command may take before it is abandoned, in seconds. A cold build of the
# largest project in the corpus takes about three minutes at `-O max`.
TIMEOUT_SECONDS = 1800

# The line `perf stat -x,` writes for the event, e.g. `25704171187,,instructions:u,...`.
PERF_LINE = re.compile(r"^(\d+),,instructions:u")


def run_perf(command, cwd):
    """Run `command` in `cwd` under `perf stat` and return (instructions, wall_seconds, ok)."""
    perf = ["perf", "stat", "-e", "instructions:u", "-x,"] + command
    started = time.monotonic()
    try:
        proc = subprocess.run(
            perf, cwd=cwd, capture_output=True, text=True, timeout=TIMEOUT_SECONDS
        )
    except subprocess.TimeoutExpired:
        return None, None, False
    wall = time.monotonic() - started
    instructions = None
    for line in proc.stderr.splitlines():
        matched = PERF_LINE.match(line)
        if matched:
            instructions = int(matched.group(1))
    return instructions, wall, proc.returncode == 0


def fix_command(fix, subcommand, opt_level=None):
    """The command line that runs `subcommand`. `fix check` takes no options at all."""
    if subcommand == "check":
        return [fix, subcommand]
    command = [fix, subcommand, "--allow-preliminary-commands", "--allow-deprecated"]
    if opt_level is not None:
        command += ["-O", opt_level]
    return command


def make_cold(project_dir):
    for path in COLD_PATHS:
        shutil.rmtree(os.path.join(project_dir, path), ignore_errors=True)


def edit_source(project_dir, source):
    """Append a comment to `source`, so that the module it holds has to be compiled again."""
    path = os.path.join(project_dir, source)
    with open(path, "a") as handle:
        handle.write("// A line the compile-time benchmark added.\n")


def revert_source(project_dir, source, original):
    with open(os.path.join(project_dir, source), "w") as handle:
        handle.write(original)


def measure(project_dir, command, reps, setup=None):
    """Run `command` `reps` times and return the lowest instruction count of the runs.

    The lowest is the run that did the least work other than the compile itself. The counts sit
    within a tenth of a percent of each other, so which of them is taken hardly matters; taking
    the lowest keeps a run that happened to page in the compiler binary from being the figure.
    """
    best = None
    best_wall = None
    for _ in range(reps):
        if setup is not None:
            setup()
        instructions, wall, ok = run_perf(command, project_dir)
        if not ok or instructions is None:
            return None, None
        if best is None or instructions < best:
            best, best_wall = instructions, wall
    return best, best_wall


def project_sources(project_dir):
    """The Fix source files the project file lists, in the order it lists them."""
    project_file = os.path.join(project_dir, "fixproj.toml")
    with open(project_file, "rb") as handle:
        return tomllib.load(handle).get("build", {}).get("files", [])


def source_line_count(project_dir, sources):
    total = 0
    for source in sources:
        path = os.path.join(project_dir, source)
        if os.path.exists(path):
            with open(path, errors="replace") as handle:
                total += sum(1 for _ in handle)
    return total


def corpus_name(origin, taken):
    """A directory name for `origin` that no other project of the corpus has.

    Projects are named after their directory, and a name two of them share -- `engine`, `sudoku` --
    grows leftwards along the path until it is the name of one project alone.
    """
    parts = origin.rstrip("/").split("/")
    for depth in range(1, len(parts) + 1):
        name = "_".join(parts[-depth:])
        if name not in taken:
            taken.add(name)
            return name
    raise ValueError("two projects share the whole of their path: " + origin)


def prepare(args):
    """Copy each project into the corpus directory and find out what can be measured on it."""
    with open(args.projects_from) as handle:
        origins = [line.strip() for line in handle if line.strip() and not line.startswith("#")]

    os.makedirs(args.corpus, exist_ok=True)
    entries = []
    taken = set()
    for origin in origins:
        name = corpus_name(origin, taken)
        destination = os.path.join(args.corpus, name)
        if os.path.exists(destination):
            shutil.rmtree(destination)
        shutil.copytree(
            origin,
            destination,
            ignore=shutil.ignore_patterns(DOT_FIXLANG, "target", ".git", "*.o", "*.out"),
            symlinks=True,
        )

        sources = project_sources(destination)
        entry = {
            "name": name,
            "origin": origin,
            "lines": source_line_count(destination, sources),
            "edit_source": sources[-1] if sources else None,
        }

        # Installing the dependencies and building once tells us which of the two commands this
        # project supports: a library has no `Main::main` and cannot be built, and one whose C
        # dependencies are missing from this machine compiles neither way.
        subprocess.run(
            [args.fix, "deps", "install"], cwd=destination, capture_output=True, timeout=600
        )
        checked = subprocess.run(
            fix_command(args.fix, "check"), cwd=destination, capture_output=True, timeout=1800
        )
        entry["check"] = checked.returncode == 0
        built = subprocess.run(
            fix_command(args.fix, "build", "basic"),
            cwd=destination,
            capture_output=True,
            timeout=1800,
        )
        entry["build"] = built.returncode == 0
        if not entry["check"]:
            entry["error"] = checked.stderr.decode(errors="replace")[-400:]
        elif not entry["build"]:
            entry["error"] = built.stderr.decode(errors="replace")[-400:]
        entries.append(entry)
        print(
            "%-44s lines=%-7d check=%-5s build=%s"
            % (name, entry["lines"], entry["check"], entry["build"]),
            flush=True,
        )

    with open(os.path.join(args.corpus, "corpus.json"), "w") as handle:
        json.dump(entries, handle, indent=2)


def scenarios_for(entry, opt_levels):
    """The (scenario, subcommand, opt_level) triples that can be measured on `entry`."""
    scenarios = []
    if entry["check"]:
        scenarios += [("check_cold", "check", None), ("check_warm", "check", None)]
    if entry["build"]:
        for opt in opt_levels:
            scenarios += [
                ("build_cold", "build", opt),
                ("build_warm", "build", opt),
                ("build_edit", "build", opt),
            ]
    return scenarios


def run(args):
    with open(os.path.join(args.corpus, "corpus.json")) as handle:
        entries = json.load(handle)
    if args.only:
        entries = [e for e in entries if e["name"] in args.only]

    with open(args.out, "w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["project", "lines", "scenario", "opt", "instructions", "wall"])
        for entry in entries:
            project_dir = os.path.join(args.corpus, entry["name"])
            source = entry["edit_source"]
            original = None
            if source is not None and os.path.exists(os.path.join(project_dir, source)):
                with open(os.path.join(project_dir, source)) as source_handle:
                    original = source_handle.read()

            for scenario, subcommand, opt in scenarios_for(entry, args.opt_levels):
                command = fix_command(args.fix, subcommand, opt)
                if scenario.endswith("_cold"):
                    setup = lambda: make_cold(project_dir)
                elif scenario.endswith("_edit"):
                    setup = lambda: edit_source(project_dir, source)
                else:
                    setup = None

                # A warm run is warm because something ran before it, and an edited run has to
                # start from a tree the compiler has already seen.
                if scenario.endswith("_warm") or scenario.endswith("_edit"):
                    run_perf(command, project_dir)

                instructions, wall = measure(project_dir, command, args.reps, setup)
                if original is not None:
                    revert_source(project_dir, source, original)
                writer.writerow(
                    [entry["name"], entry["lines"], scenario, opt or "", instructions, wall]
                )
                handle.flush()
                print(
                    "%-40s %-11s %-6s %s"
                    % (entry["name"], scenario, opt or "-", f"{instructions:,}" if instructions else "FAILED"),
                    flush=True,
                )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", default=DEFAULT_CORPUS)
    parser.add_argument("--fix", default=shutil.which("fix"))
    subparsers = parser.add_subparsers(dest="mode", required=True)

    prepared = subparsers.add_parser("prepare")
    prepared.add_argument("--projects-from", required=True)
    prepared.set_defaults(func=prepare)

    ran = subparsers.add_parser("run")
    ran.add_argument("--reps", type=int, default=3)
    ran.add_argument("--out", default="compiletime.csv")
    ran.add_argument("--opt-levels", nargs="+", default=OPT_LEVELS)
    ran.add_argument("--only", nargs="+")
    ran.set_defaults(func=run)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
