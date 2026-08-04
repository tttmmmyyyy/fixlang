"""Run a program under `perf stat` and print the counters cachegrind cannot express.

Prints one line, `<splits>,<cycles>,<contention>`: loads and stores that crossed a cache-line
boundary, user-space core cycles, and the CPU that work other than this measurement took while
it ran, in cores. Cachegrind's cache model counts references and misses and has no notion of a
line-crossing access, yet those cost real time -- an array whose elements start 8 bytes into a
16-byte-aligned allocation splits half of its 32-byte accesses. And an instruction count says
nothing about how fast the machine gets through those instructions, which is where a change to
code layout or branch density shows up.

The split count is deterministic; the cycle count is not, so it is the minimum over `--repeat`
runs, and it is reported only when the machine had CPU to spare for the program throughout. A
run the rest of the machine competed with leaves the cycle field empty rather than logging a
figure that says more about that competition than about the program.

Exits non-zero when the counters are unavailable (no hardware PMU, or
`kernel.perf_event_paranoid` above 2) or when the PMU had to time-slice them, so a caller
can leave the columns empty instead of logging an estimate.

    python3 perf_counters.py [--repeat N] ./a.out [args...]
    python3 perf_counters.py --cpu
"""

import os
import resource
import subprocess
import sys
import time

# Three counters per run. Keeping the list short matters -- asking for more events than the
# PMU has counters makes perf time-slice them and report scaled estimates.
SPLIT_EVENTS = ["mem_inst_retired.split_loads", "mem_inst_retired.split_stores"]
CYCLE_EVENT = "cycles:u"

# The CPU that work other than this measurement may take while it runs, in cores. Above this
# the cycle count says as much about that work as about the program, so the field comes back
# empty and every count that reaches the log is one worth comparing.
#
# This is what the one-minute load average cannot say. That average counts the program being
# measured, and everything the caller ran in the minute before, alongside whatever else the
# machine is doing -- so on a machine with nothing to do but this it still reads above one, and
# a threshold on it rejects measurements that were never disturbed.
QUIET_CONTENTION = 0.5

CLOCK_TICK = os.sysconf("SC_CLK_TCK")

ARCH = subprocess.check_output(["uname", "-m"], text=True).strip()


def machine_cpu_seconds():
    """CPU seconds every process on this machine has spent off idle since boot."""
    # user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice
    fields = [int(f) for f in open("/proc/stat", encoding="utf-8").readline().split()[1:]]
    return (sum(fields) - fields[3] - fields[4]) / CLOCK_TICK


def own_cpu_seconds():
    """CPU seconds this process and the programs it has waited for have spent."""
    mine = resource.getrusage(resource.RUSAGE_SELF)
    theirs = resource.getrusage(resource.RUSAGE_CHILDREN)
    return mine.ru_utime + mine.ru_stime + theirs.ru_utime + theirs.ru_stime


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
    """The split count, the lowest cycle count over `repeat` runs, and the CPU other work
    took over the whole of it, in cores.

    The run with the fewest cycles is the one the rest of the machine disturbed least. The
    split count comes from the same runs and has to agree across them, since it counts
    retired instructions of a kind and nothing about the machine's state can change it.
    """
    machine_before = machine_cpu_seconds()
    own_before = own_cpu_seconds()
    started = time.monotonic()
    splits = None
    cycles = None
    for _ in range(repeat):
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
    elapsed = time.monotonic() - started
    others = (machine_cpu_seconds() - machine_before) - (own_cpu_seconds() - own_before)
    # `/proc/stat` counts in whole ticks and the rusage clocks round, so a short measurement
    # can put the difference slightly below zero.
    contention = max(0.0, others) / elapsed if elapsed > 0 else 0.0
    return splits, cycles, contention


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
    splits, cycles, contention = measure(argv, repeat)
    # The split count is deterministic, so it is reported whatever the machine was doing.
    reported_cycles = "" if contention > QUIET_CONTENTION else str(cycles)
    print(f"{splits},{reported_cycles},{contention:.2f}")


main()
