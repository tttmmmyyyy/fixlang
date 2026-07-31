"""Run a program under `perf stat` and print the counters cachegrind cannot express.

Prints one line, `<splits>,<cycles>,<load>`: loads and stores that crossed a cache-line
boundary, user-space core cycles, and the highest one-minute load average seen while
measuring. Cachegrind's cache model counts references and misses and has no notion of a
line-crossing access, yet those cost real time -- an array whose elements start 8 bytes
into a 16-byte-aligned allocation splits half of its 32-byte accesses. And an instruction
count says nothing about how fast the machine gets through those instructions, which is
where a change to code layout or branch density shows up.

The split count is deterministic; the cycle count is not, so it is the minimum over
`--repeat` runs, and it is reported only when the machine stayed quiet throughout. A run that
saw a higher load leaves the cycle field empty rather than logging a figure that says more
about the rest of the machine than about the program.

Exits non-zero when the counters are unavailable (no hardware PMU, or
`kernel.perf_event_paranoid` above 2) or when the PMU had to time-slice them, so a caller
can leave the columns empty instead of logging an estimate.

    python3 perf_counters.py [--repeat N] ./a.out [args...]
    python3 perf_counters.py --cpu
"""

import os
import subprocess
import sys

# Three counters per run. Keeping the list short matters -- asking for more events than the
# PMU has counters makes perf time-slice them and report scaled estimates.
SPLIT_EVENTS = ["mem_inst_retired.split_loads", "mem_inst_retired.split_stores"]
CYCLE_EVENT = "cycles:u"

# A one-minute load average above this means something else was running, and a cycle count taken
# then says more about that than about the program. The cycle field comes back empty instead, so
# every count that reaches the log is one worth comparing. One is the harness itself.
QUIET_LOAD = 2.0

ARCH = subprocess.check_output(["uname", "-m"], text=True).strip()


def read_counters(argv):
    """Event name -> count, for the events perf managed to read."""
    proc = subprocess.run(
        # ASLR off, as cachegrind.py runs it: the split count depends on where the
        # allocator puts the data, so a moving heap would move the number.
        ["setarch", ARCH, "-R", "perf", "stat", "-x,",
         "-e", ",".join(SPLIT_EVENTS + [CYCLE_EVENT]), "--"] + argv,
        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
    )
    # perf exits with the program's status, and it reports whatever the program managed to
    # execute before it died. Counting a partial run as a measurement would put a plausible
    # number in the log.
    if proc.returncode != 0:
        sys.exit(f"{argv[0]} exited with {proc.returncode}")
    found = {}
    for line in proc.stderr.splitlines():
        fields = line.split(",")
        if len(fields) < 3 or not fields[0].strip().isdigit():
            continue
        # perf appends `:u` to the event name when it may only count user space.
        name = fields[2].strip().removesuffix(":u")
        # Field 5 is the percentage of the run the event was actually on a counter. Below
        # 100 the PMU time-sliced the events and perf scaled the count up to compensate, so
        # what it prints is an estimate that looks like any other measurement.
        if len(fields) >= 5 and fields[4].strip():
            try:
                if float(fields[4]) < 100.0:
                    sys.exit(f"perf could keep {name} on a counter for only "
                             f"{fields[4].strip()}% of the run, so its count is an estimate")
            except ValueError:
                pass
        found[name] = int(fields[0])
    return found, proc.stderr


def measure(argv, repeat):
    """The split count, the lowest cycle count over `repeat` runs, and the highest load seen.

    The run with the fewest cycles is the one the rest of the machine disturbed least. The
    split count comes from the same runs and has to agree across them, since it counts
    retired instructions of a kind and nothing about the machine's state can change it.
    """
    splits = None
    cycles = None
    load = 0.0
    for _ in range(repeat):
        load = max(load, os.getloadavg()[0])
        found, report = read_counters(argv)
        missing = [e for e in SPLIT_EVENTS + [CYCLE_EVENT.removesuffix(":u")]
                   if e not in found]
        if missing:
            # Say which of the two it was: the program never ran, or the counters are out
            # of reach.
            sys.exit(f"perf reported none of {', '.join(missing)}. perf said:\n"
                     + report.strip())
        run_splits = sum(found[e] for e in SPLIT_EVENTS)
        if splits is None:
            splits = run_splits
        elif run_splits != splits:
            sys.exit(f"the split count changed between runs of the same program "
                     f"({splits} then {run_splits}), so one of them is not a measurement")
        run_cycles = found[CYCLE_EVENT.removesuffix(":u")]
        cycles = run_cycles if cycles is None else min(cycles, run_cycles)
    return splits, cycles, load


def cpu_model():
    """The processor these counters would be read on.

    A split count belongs to a microarchitecture the way an instruction count does not,
    so counts read on different machines cannot be compared with each other.
    """
    for line in open("/proc/cpuinfo", encoding="utf-8"):
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip().replace(",", " ")
    return "unknown"


def main():
    argv = sys.argv[1:]
    if argv == ["--cpu"]:
        print(cpu_model())
        return
    repeat = 5
    if len(argv) >= 2 and argv[0] == "--repeat":
        repeat = int(argv[1])
        argv = argv[2:]
    if not argv:
        sys.exit("usage: perf_counters.py [--repeat N] <program> [args...]\n"
                 "       perf_counters.py --cpu")
    splits, cycles, load = measure(argv, repeat)
    # The split count is deterministic, so it is reported whatever the machine was doing.
    reported_cycles = "" if load > QUIET_LOAD else str(cycles)
    print(f"{splits},{reported_cycles},{load:.2f}")


main()
