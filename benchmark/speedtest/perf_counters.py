"""Run a program under `perf stat` and print the counters cachegrind cannot express.

Prints one line, `<splits>,<cycles>,<contention>`: loads and stores that crossed a cache-line
boundary, user-space core cycles, and the CPU that work other than this measurement took while
it ran, in cores. Cachegrind's cache model counts references and misses and has no notion of a
line-crossing access, yet those cost real time -- an array whose elements start 8 bytes into a
16-byte-aligned allocation splits half of its 32-byte accesses. And an instruction count says
nothing about how fast the machine gets through those instructions, which is where a change to
code layout or branch density shows up.

The split count is decided by the program and the environment it is given; the cycle count is
not, so it is the minimum over `--repeat` windows, and it is reported when nothing else could
have moved it. Other work reaches it two ways: over the core it shares with the thread beside it,
which the run is pinned and the sibling watched for, and over the cache every core shares, which
the caller answers for by passing what cachegrind counted for this program. A run either could
have moved leaves the cycle field empty rather than logging a figure that says more about that
competition than about the program.

Exits non-zero when the counters are unavailable (no hardware PMU, or
`kernel.perf_event_paranoid` above 2) or when the PMU had to time-slice them, so a caller
can leave the columns empty instead of logging an estimate.

    python3 perf_counters.py [--repeat N] [--dram-accesses N --instructions N] ./a.out [args...]
    python3 perf_counters.py --cpu
"""

import os
import re
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

# How often a program's data comes from main memory, per instruction, before another process can
# change its cycle count by taking the cache from it. A program above this rate loses between 1.5
# and 2.1 times its cycles when four processes walk twice the last level cache beside it; one
# below it stays within the spread of an undisturbed reading.
CONTENDED_DRAM_RATE = 0.002

# How much of a run the other thread of the measurement's core may be busy for. The two threads
# share the core's front end and execution units, and the cycle counter runs while the program's
# instructions wait for a slot: a case read with the sibling busy for most of the run came out
# 6.6% above its idle-machine figure, and 9.2% above with the sibling saturated, while the same
# case read below this limit came within 0.22% of it.
SIBLING_BUSY_LIMIT = 0.5

# The shortest run the sibling reading can speak for. `/proc/stat` counts in ticks of a
# hundredth of a second, so a run of a few of them puts the reading's own quantisation above the
# limit it is compared with. A program that returns sooner is run again until the window holds
# enough of them, and the cycle count taken is the lowest of those runs.
MINIMUM_WINDOW = 0.2

# The environment the measured command gets, fixed the way `cachegrind.py` fixes it. The
# initial stack is laid out above the environment block, so every address on the stack moves
# with how much the caller happened to export, and a stack object that lands 8 bytes below a
# cache-line boundary splits every wide access to it. Left to the caller's environment, one
# unchanged binary reported 70,765 splits from one shell and 170,766 from another.
MEASUREMENT_ENV = {"PATH": "/usr/bin:/bin", "LC_ALL": "C"}

CLOCK_TICK = os.sysconf("SC_CLK_TCK")

ARCH = subprocess.check_output(["uname", "-m"], text=True).strip()


def cpu_list(text):
    """The CPU numbers a sysfs list like `5,11` or `2-5,8` names."""
    numbers = []
    for part in text.strip().split(","):
        if not part:
            continue
        ends = part.split("-")
        numbers.extend(range(int(ends[0]), int(ends[-1]) + 1))
    return numbers


def measurement_core():
    """The CPU the programs are run on, and the other thread of its core where that core has
    two.

    Pinning is what makes the sibling knowable: a program the scheduler is free to move shares
    its core with a different thread from one moment to the next, and which one it was is gone
    by the time the run ends.

    Of the core's two threads the run takes the lower, leaving the higher to be watched. Linux
    numbers a core's second thread into the upper half of the CPUs and fills the lower half
    first, so the thread watched here is the one the rest of the machine reaches for last, and a
    measurement finds its core to itself that much more often.
    """
    present = sorted(int(name[3:]) for name in os.listdir("/sys/devices/system/cpu")
                     if re.fullmatch(r"cpu\d+", name))
    topology = f"/sys/devices/system/cpu/cpu{present[-1]}/topology/thread_siblings_list"
    try:
        with open(topology, encoding="utf-8") as siblings:
            threads = sorted(cpu_list(siblings.read()))
    except OSError:
        threads = [present[-1]]
    return threads[0], threads[-1] if len(threads) > 1 else None


CPU, SIBLING = measurement_core()

# The programs inherit this, so they run where the sibling is watched. Setting it here rather
# than putting `taskset` in front of the command keeps the chain of programs that leads to the
# measured one exactly as long as it was: the initial stack is laid out above that chain's
# arguments, and moving it moves which accesses straddle a cache line, which is the `-splits`
# column.
os.sched_setaffinity(0, {CPU})


def sibling_cpu_seconds():
    """CPU seconds the other thread of the measurement's core has spent off idle since boot."""
    if SIBLING is None:
        return 0.0
    with open("/proc/stat", encoding="utf-8") as stat:
        for line in stat:
            if line.startswith(f"cpu{SIBLING} "):
                fields = [int(f) for f in line.split()[1:]]
                return (sum(fields) - fields[3] - fields[4]) / CLOCK_TICK
    return 0.0


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
        env=MEASUREMENT_ENV,
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


def read_window(argv, splits):
    """The lowest cycle count over a window of runs, how busy the sibling thread was through
    it, and the split count the runs agreed on.

    A window holds as many runs as `MINIMUM_WINDOW` needs, so that the sibling reading covers
    enough ticks of `/proc/stat` to mean something. The split count comes from the same runs
    and has to agree across them: the runs are given one environment, and from there nothing
    about the machine's state reaches the addresses the program touches.
    """
    sibling_before = sibling_cpu_seconds()
    started = time.monotonic()
    cycles = None
    while True:
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
        if elapsed >= MINIMUM_WINDOW:
            break
    sibling_busy = (sibling_cpu_seconds() - sibling_before) / elapsed
    return cycles, sibling_busy, splits


def measure(argv, repeat):
    """The split count, the lowest cycle count over the windows the sibling thread left alone,
    and the CPU other work took over the whole of it, in cores.

    The cycle count comes back as `None` where the sibling was busy through every window, since
    a core the program had only half of gives a figure about the sharing rather than about the
    program.
    """
    machine_before = machine_cpu_seconds()
    own_before = own_cpu_seconds()
    started = time.monotonic()
    splits = None
    cycles = None
    for _ in range(repeat):
        window_cycles, sibling_busy, splits = read_window(argv, splits)
        if sibling_busy <= SIBLING_BUSY_LIMIT:
            cycles = window_cycles if cycles is None else min(cycles, window_cycles)
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


def cycles_are_comparable(contention, dram_accesses, instructions):
    """Whether a cycle count read under this much competition says more about the program than
    about the competition.

    Two channels carry other work into a cycle count, and this answers for the second of them.
    The core the program runs on is shared with the thread beside it, which `measure` answers
    for by keeping only the windows that thread stayed out of. What is left is the cache, which
    every core on the machine shares: another process takes a line the program was going to
    find there, and the program that loses time to it is the one that goes to main memory often
    enough for the loss to add up.

    A program the scheduler takes the CPU from resumes with the same count of cycles ahead of
    it, and one the machine clocks down spends the same count getting there, so neither of those
    reaches the count at all.

    # Arguments
    * `dram_accesses`, `instructions` - what cachegrind counted for this program, or `None`
      where the caller has no figure for it. Both are decided by the program rather than by the
      machine it ran on, so the rate they give is the same on a busy machine as on an idle one.
    """
    if contention <= QUIET_CONTENTION:
        return True
    if dram_accesses is None or not instructions:
        return False
    return dram_accesses / instructions <= CONTENDED_DRAM_RATE


def take_options(argv):
    """The options standing in front of the program, and the command left after them."""
    options = {"--repeat": 5, "--dram-accesses": None, "--instructions": None}
    while argv and argv[0] in options:
        if len(argv) < 2:
            sys.exit(f"{argv[0]} takes a value")
        options[argv[0]] = int(argv[1])
        argv = argv[2:]
    return options["--repeat"], options["--dram-accesses"], options["--instructions"], argv


def main():
    argv = sys.argv[1:]
    if argv == ["--cpu"]:
        print(cpu_model())
        return
    repeat, dram_accesses, instructions, argv = take_options(argv)
    if not argv:
        sys.exit("usage: perf_counters.py [--repeat N] [--dram-accesses N --instructions N]"
                 " <program> [args...]\n"
                 "       perf_counters.py --cpu")
    splits, cycles, contention = measure(argv, repeat)
    # The split count is deterministic, so it is reported whatever the machine was doing.
    reported_cycles = (str(cycles) if cycles is not None
                       and cycles_are_comparable(contention, dram_accesses, instructions) else "")
    print(f"{splits},{reported_cycles},{contention:.2f}")


main()
