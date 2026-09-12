"""Run a program under `perf stat` and print what the hardware counters say about it.

Prints one line, `<instructions>,<ram>,<splits>,<cycles>,<contention>`: instructions retired in
user space, accesses that missed the last level cache and so went to main memory, loads and
stores that crossed a cache-line boundary, user-space core cycles, and the CPU that work other
than this measurement took while it ran, in cores.

The instruction and split counts are decided by the program and its input, which is what makes
them the columns a change to the compiler is read on. The runs here take a fixed environment with
address-space randomization off, and under those eight runs of `nbody` read one split count and six
of `iter_flatten` read one. A handful of the split count belongs to where the path of the program
put its stack rather than to the program: the same binary measured as `./ref_r`, `./ref_rust` and
`./ref_rustxxxxx` read 461,525, 461,527 and 461,523, each of those three times over. Two languages
whose split counts differ by a few are not two programs that differ. A line-crossing access costs real time and an instruction count has no
notion of one -- an array whose elements start 8 bytes into a 16-byte-aligned allocation splits
half of its 32-byte accesses. The RAM count moves with what else was in the cache. The cycle
count is the one figure here that says how fast the machine gets through the work, which is where
a change to code layout or branch density shows up.

Every figure is the lowest reading over `--windows` windows of runs, since other work only ever
raises one.

The cycle count is reported only where nothing else could have moved it, and the other columns
whatever the machine was doing. Other work reaches a cycle count two ways. It runs on the other
thread of the same core: the measurement is pinned to one CPU, and how busy that thread was is
read for every window. And it takes the cache every core shares, which costs the program that
goes to main memory often enough for the loss to add up -- the RAM count above is what says
whether this program is one of those. Where either of them could have moved a run, the cycle
field comes back empty, since a figure logged there would say more about that competition than
about the program.

Exits 2 when the measured program itself exits non-zero. A case checks its own answer, so that
is a case whose answer moved, and a caller that treated it as a missing measurement would leave
the same empty columns behind as it does for a machine with no counters to read.

Exits 1 when the counters are unavailable (no hardware PMU, or `kernel.perf_event_paranoid` above
2) or when the PMU had to time-slice them, so a caller can leave the columns empty instead of
logging an estimate.

    python3 perf_counters.py [--windows N] ./a.out [args...]
    python3 perf_counters.py --cpu
"""

import os
import re
import resource
import subprocess
import sys
import time
from typing import NamedTuple

# Keeping the list short matters -- asking for more events than the PMU has counters makes perf
# time-slice them and report scaled estimates, which `read_counters` refuses. Cycles and
# instructions sit on fixed counters, so three of these five compete for the general-purpose ones.
SPLIT_EVENTS = ["mem_inst_retired.split_loads:u", "mem_inst_retired.split_stores:u"]
CYCLE_EVENT = "cycles:u"
INSTRUCTION_EVENT = "instructions:u"
# Accesses that missed the last level cache, so their data came from main memory. This counts what
# a prefetcher fetched as well as what a load waited for, which together are what the program takes
# of the cache and the memory bus that every core shares.
RAM_EVENT = "cache-misses:u"
ALL_EVENTS = SPLIT_EVENTS + [CYCLE_EVENT, INSTRUCTION_EVENT, RAM_EVENT]
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
# change its cycle count by taking the cache from it. The rate was read on an 11th Gen Intel Core
# i5-11400 (6 cores, 12 MiB of last level cache), with four processes each walking twice that cache
# on other cores. A case was read disturbed and undisturbed in turn inside one round, and a round
# the sibling thread was busy through was thrown away. Every case of the suite below this rate came
# within 3.5% of its undisturbed cycles, and every case above 0.000045 took between 1.08 and 7.8
# times them. The limit sits at the low end of the gap between the two, because a case dropped
# costs a row one figure where a case kept wrongly puts a figure in the log that reads like every
# other one. What `cache-misses` counts belongs to the processor that counted it, so a machine
# with a cache of another size is applying a number measured elsewhere.
#
# This numerator is the least steady figure the program reads: over the suite, six runs of a case
# moved its instruction count by at most 0.0010% and its split count not at all, while its
# `cache-misses` moved by as much as 4.6 times. The case that sits nearest the limit, `fib`, read
# 7.44e-06 to 1.906e-05 over eight measurements at the five windows the harness uses -- on one
# side of the limit, with 5% to spare. Read at one window it straddles, since fewer windows raise
# the minimum: `--windows` moves this rate as well as the cycle count it gates. Narrowing that
# would mean reading the rate from the window whose cycles are kept, which is a different
# numerator and wants its own calibration on a machine with nothing else running.
RAM_RATE_LIMIT = 0.00002

# How much of a run the other thread of the measurement's core may be busy for. The two threads
# share the core's front end and execution units, and the cycle counter runs while the program's
# instructions wait for a slot: a case read with the sibling busy for most of the run came out
# 6.6% above its idle-machine figure, and 9.2% above with the sibling saturated, while the same
# case read below this limit came within 0.22% of it.
SIBLING_BUSY_LIMIT = 0.5

# The shortest run the sibling reading can speak for. `/proc/stat` counts in ticks of a
# hundredth of a second, so over a run of a few ticks the reading rounds by more than
# `SIBLING_BUSY_LIMIT` itself. A program that returns sooner is run again until the window is this
# long, and the cycle count taken is the lowest of those runs.
MINIMUM_WINDOW_SECONDS = 0.2

# The environment the measured command gets. The initial stack is laid out above the environment
# block, so every address on the stack moves with how much the caller happened to export, and a
# stack object that lands 8 bytes below a cache-line boundary splits every wide access to it. Left
# to the caller's environment, one unchanged binary reported 70,765 splits from one shell and
# 170,766 from another.
MEASUREMENT_ENV = {"PATH": "/usr/bin:/bin", "LC_ALL": "C"}

# What this program exits with when the program it measures exits non-zero, which a caller reads
# apart from the 1 that says the counters could not be read. Every caller spells this number
# again, since importing this module would set the affinity of the importing process.
PROGRAM_FAILED = 2

CLOCK_TICK = os.sysconf("SC_CLK_TCK")

ARCH = subprocess.check_output(["uname", "-m"], text=True).strip()


def cpu_list(text):
    """The CPU numbers a sysfs list like `5,11` or `2-5,8` names."""
    numbers = []
    for part in text.strip().split(","):
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
    highest = max(int(name[3:]) for name in os.listdir("/sys/devices/system/cpu")
                  if re.fullmatch(r"cpu\d+", name))
    siblings_path = f"/sys/devices/system/cpu/cpu{highest}/topology/thread_siblings_list"
    with open(siblings_path, encoding="utf-8") as siblings:
        threads = sorted(cpu_list(siblings.read()))
    return threads[0], threads[-1] if len(threads) > 1 else None


MEASUREMENT_CPU, SIBLING_CPU = measurement_core()

# The programs inherit this, so they run where the sibling is watched. Setting it in this process
# leaves the chain of programs that leads to the measured one as short as it can be: the initial
# stack is laid out above that chain's arguments, so a `taskset` in front of the command would move
# which accesses straddle a cache line, which is the `-splits` column.
os.sched_setaffinity(0, {MEASUREMENT_CPU})


def cpu_seconds(cpu_name):
    """CPU seconds spent off idle since boot, from the `/proc/stat` line of that name.

    # Arguments
    * `cpu_name` - what the line begins with: `cpu` for the machine as a whole, `cpu5` for one
      logical CPU.
    """
    with open("/proc/stat", encoding="utf-8") as stat:
        for line in stat:
            if line.startswith(cpu_name + " "):
                # user, nice, system, idle, iowait, irq, softirq, steal, guest, guest_nice
                fields = [int(f) for f in line.split()[1:]]
                return (sum(fields) - fields[3] - fields[4]) / CLOCK_TICK
    # Reading a busy CPU as an idle one would let every window through, so say so instead.
    sys.exit(f"/proc/stat carries no {cpu_name} line")


def sibling_cpu_seconds():
    """CPU seconds the other thread of the measurement's core has spent off idle since boot, or
    zero where that core has one thread and so nothing shares it."""
    return cpu_seconds(f"cpu{SIBLING_CPU}") if SIBLING_CPU is not None else 0.0


def own_cpu_seconds():
    """CPU seconds this process and the programs it has waited for have spent."""
    mine = resource.getrusage(resource.RUSAGE_SELF)
    theirs = resource.getrusage(resource.RUSAGE_CHILDREN)
    return mine.ru_utime + mine.ru_stime + theirs.ru_utime + theirs.ru_stime


def event_name(event):
    """`event` without the `:u` that asks perf to count user space alone.

    perf prints an event under the name it was asked for, `:u` included, so the counts and the
    names they are looked up under are both put through this.
    """
    return event.removesuffix(":u")


def lower_of(best, reading):
    """The lower of the two, taking `reading` where there is no `best` yet."""
    return reading if best is None or reading < best else best


class Counts(NamedTuple):
    """What the counters read, in the order this program prints them.

    One of these describes a run, a window of runs, or a whole measurement, since the four are
    folded together the same way at every level.
    """

    instructions: int
    ram_accesses: int
    splits: int
    cycles: int

    def lowest_with(self, other):
        """Each count of the two, whichever of the pair is lower."""
        return Counts(*(lower_of(a, b) for a, b in zip(self, other)))


def read_counters(argv):
    """Event name -> count for every event of `ALL_EVENTS`.

    Exits 1 where perf could not read one of them, and `PROGRAM_FAILED` where it read them all
    and the measured program still exited non-zero.
    """
    proc = subprocess.run(
        # ASLR off: the split count depends on where the allocator puts the data, so a moving
        # heap would move the number.
        ["setarch", ARCH, "-R", "perf", "stat", "-x,",
         "-e", ",".join(ALL_EVENTS), "--"] + argv,
        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
        env=MEASUREMENT_ENV,
    )
    found = {}
    for line in proc.stderr.splitlines():
        fields = line.split(",")
        if len(fields) < 3 or not fields[0].strip().isdigit():
            continue
        name = event_name(fields[2].strip())
        # Field 5 is the percentage of the run the event was actually on a counter. Below
        # 100 the PMU time-sliced the events and perf scaled the count up to compensate, so
        # what it prints is an estimate that looks like any other measurement.
        if len(fields) >= 5 and fields[4].strip():
            try:
                on_a_counter = float(fields[4])
            except ValueError:
                sys.exit(f"perf reported \"{fields[4].strip()}\" for how much of the run it kept "
                         f"{name} on a counter, which is what says whether the count is an "
                         f"estimate")
            if on_a_counter < 100.0:
                sys.exit(f"perf could keep {name} on a counter for only "
                         f"{fields[4].strip()}% of the run, so its count is an estimate")
        found[name] = int(fields[0])
    # Which of the two went wrong is read from the counts rather than from the status: perf exits
    # with the measured program's status where that program ran, and an event this processor does
    # not carry -- `SPLIT_EVENTS` names two that only Intel does -- leaves it exiting 129 with no
    # count at all.
    missing = [event_name(e) for e in ALL_EVENTS if event_name(e) not in found]
    if missing:
        sys.exit(f"perf reported none of {', '.join(missing)}. perf said:\n"
                 + proc.stderr.strip())
    # perf reports whatever the program managed to execute before it died, and counting a partial
    # run as a measurement would put a plausible number in the log.
    if proc.returncode != 0:
        print(f"{argv[0]} exited with {proc.returncode}", file=sys.stderr)
        sys.exit(PROGRAM_FAILED)
    return found


def read_window(argv):
    """The lowest `Counts` over a window of runs, and how busy the sibling thread was through it.

    A window holds as many runs as `MINIMUM_WINDOW_SECONDS` needs, so that the sibling reading covers
    enough ticks of `/proc/stat` to mean something.

    Each count is the window's lowest on its own: whatever the rest of the machine does to a run
    raises the counts of that run and lowers none of them.
    """
    sibling_before = sibling_cpu_seconds()
    started = time.monotonic()
    lowest = None
    while True:
        found = read_counters(argv)
        run = Counts(instructions=found[event_name(INSTRUCTION_EVENT)],
                     ram_accesses=found[event_name(RAM_EVENT)],
                     splits=sum(found[event_name(e)] for e in SPLIT_EVENTS),
                     cycles=found[event_name(CYCLE_EVENT)])
        lowest = run if lowest is None else lowest.lowest_with(run)
        elapsed = time.monotonic() - started
        if elapsed >= MINIMUM_WINDOW_SECONDS:
            break
    sibling_busy = (sibling_cpu_seconds() - sibling_before) / elapsed
    return lowest, sibling_busy


def measure(argv, windows):
    """The `Counts` of the measurement, and the CPU other work took over the whole of it, in cores.

    Three of the four counts are the lowest readings of every window, busy or not: other work
    raises them and cannot lower them. The cycle count is the lowest of the windows the sibling
    thread left alone, and `None` where it left none, since a core the program had only half of
    gives a figure about the sharing rather than about the program.
    """
    assert windows >= 1, windows
    machine_before = cpu_seconds("cpu")
    own_before = own_cpu_seconds()
    started = time.monotonic()
    lowest = None
    cycles = None
    for _ in range(windows):
        window, sibling_busy = read_window(argv)
        lowest = window if lowest is None else lowest.lowest_with(window)
        if sibling_busy <= SIBLING_BUSY_LIMIT:
            cycles = lower_of(cycles, window.cycles)
    elapsed = time.monotonic() - started
    others = (cpu_seconds("cpu") - machine_before) - (own_cpu_seconds() - own_before)
    # `/proc/stat` counts in whole ticks and the rusage clocks round, so a short measurement
    # can put the difference slightly below zero. The span itself is positive for a measurement
    # of a program, whose every window runs for at least `MINIMUM_WINDOW_SECONDS`; it is `0.0`
    # only for the canned windows of `self_check`, which return without a clock running.
    contention = max(0.0, others) / elapsed if elapsed > 0 else 0.0
    return lowest._replace(cycles=cycles), contention


def cpu_model():
    """The processor these counters would be read on.

    A split count belongs to a microarchitecture the way an instruction count does not,
    so counts read on different machines cannot be compared with each other.
    """
    for line in open("/proc/cpuinfo", encoding="utf-8"):
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip().replace(",", " ")
    return "unknown"


def cycles_are_comparable(counts, contention):
    """Whether a cycle count read under this much competition says more about the program than
    about the competition.

    Other work reaches a cycle count two ways. It shares the core through the thread beside the
    measurement: `measure` keeps only the windows that thread stayed out of, and hands back `None`
    where it stayed out of none. And it takes the cache that every core shares, which costs the
    program that goes to main memory often enough for the loss to add up.

    A program the scheduler takes the CPU from resumes with the same count of cycles ahead of
    it, and one the machine clocks down spends the same count getting there, so neither of those
    reaches the count at all.

    # Arguments
    * `counts` - what `measure` returned, whose `cycles` is `None` where no window was the
      measurement's own. Its other counts are each the lowest reading of the windows, and other
      work raises them and lowers none, so the rate they give is the closest to the program's own
      that a busy machine offers.
    """
    if counts.cycles is None:
        return False
    if contention <= QUIET_CONTENTION:
        return True
    assert counts.instructions > 0, counts
    return counts.ram_accesses / counts.instructions <= RAM_RATE_LIMIT


def counter_line(counts, contention):
    """The line this program prints for a measurement.

    Every count reaches it whatever the machine was doing, and the cycle count only where nothing
    else could have moved it; the field left empty there is what says so.
    """
    reported_cycles = str(counts.cycles) if cycles_are_comparable(counts, contention) else ""
    return (f"{counts.instructions},{counts.ram_accesses},{counts.splits},{reported_cycles},"
            f"{contention:.2f}")


def take_options(argv):
    """The options standing in front of the program, and the command left after them."""
    options = {"--windows": 5}
    while argv and argv[0] in options:
        if len(argv) < 2:
            sys.exit(f"{argv[0]} takes a value")
        options[argv[0]] = int(argv[1])
        argv = argv[2:]
    return options["--windows"], argv


def self_check():
    """Check the rules that need no machine to read, so one that stopped holding shows up here
    rather than in a column of the log.

    The counters themselves need a PMU and a program to run; these are the parts that decide what
    is done with them, and they cost microseconds.
    """
    # A sysfs CPU list names single CPUs, ranges, and both together, and ends in a newline.
    assert cpu_list("5\n") == [5], cpu_list("5\n")
    assert cpu_list("5,11\n") == [5, 11], cpu_list("5,11\n")
    assert cpu_list("2-5\n") == [2, 3, 4, 5], cpu_list("2-5\n")
    assert cpu_list("2-5,8\n") == [2, 3, 4, 5, 8], cpu_list("2-5,8\n")

    # A count is looked up under the name its event asks for with `:u` taken off, and no other
    # part of a name moves. Two events left sharing a name would have `read_counters` report one
    # of them twice.
    assert event_name(CYCLE_EVENT) == "cycles", event_name(CYCLE_EVENT)
    assert event_name("mem_inst_retired.split_loads") == "mem_inst_retired.split_loads"
    assert len({event_name(e) for e in ALL_EVENTS}) == len(ALL_EVENTS), ALL_EVENTS
    # Every event asks for user space alone. A machine whose `kernel.perf_event_paranoid` lets a
    # program read the kernel's counts would otherwise answer one of them with a count of both,
    # and nothing in the row would say which machine it came from.
    assert all(e.endswith(":u") for e in ALL_EVENTS), ALL_EVENTS

    assert lower_of(None, 3) == 3
    assert lower_of(5, 3) == 3 and lower_of(3, 5) == 3

    # A quiet machine's count is kept whatever the program asks of main memory. Above the rate
    # another process reaches the count through the cache; below the rate it cannot.
    busy = QUIET_CONTENTION * 2
    below = int(RAM_RATE_LIMIT * 10 ** 9 / 2)
    above = int(RAM_RATE_LIMIT * 10 ** 9 * 2)
    def counted(cycles, ram_accesses):
        return Counts(instructions=10 ** 9, ram_accesses=ram_accesses, splits=0, cycles=cycles)

    assert cycles_are_comparable(counted(1, above), QUIET_CONTENTION / 2)
    assert cycles_are_comparable(counted(1, below), busy)
    assert not cycles_are_comparable(counted(1, above), busy)
    # Each limit is the most a reading may carry and still be kept.
    assert cycles_are_comparable(counted(1, above), QUIET_CONTENTION)
    assert cycles_are_comparable(counted(1, int(RAM_RATE_LIMIT * 10 ** 9)), busy)
    # A count no window gave is no count, however little the program asks of main memory.
    assert not cycles_are_comparable(counted(None, below), QUIET_CONTENTION / 2)

    # The line carries every count whatever the machine was doing, and withholds the cycle count
    # where other work could have moved it.
    assert counter_line(counted(7, below), busy) == f"1000000000,{below},0,7,{busy:.2f}"
    assert counter_line(counted(7, above), busy) == f"1000000000,{above},0,,{busy:.2f}"

    # The options are the ones standing in front of the program; what follows the program is the
    # program's own, however it is spelled.
    assert take_options(["./a.out"]) == (5, ["./a.out"])
    assert take_options(["--windows", "3", "./a.out", "--windows", "9"]) == (
        3, ["./a.out", "--windows", "9"])
    try:
        take_options(["--windows"])
    except SystemExit:
        pass
    else:
        raise AssertionError("--windows without a value was taken as an option")

    global read_counters, read_window

    # Every count a window reports is the lowest of its runs, taken on its own. The run below with
    # the lowest cycles has the highest of every other count, so a count taken from that run
    # instead would show up here as that run's figure. The counts are keyed the way
    # `read_counters` keys them, so one looked up under a name perf does not print would show up
    # as an event the run never reported.
    def canned_run(cycles, split_loads, split_stores, instructions, ram_accesses):
        counts = dict(zip((event_name(e) for e in SPLIT_EVENTS), (split_loads, split_stores)))
        counts[event_name(CYCLE_EVENT)] = cycles
        counts[event_name(INSTRUCTION_EVENT)] = instructions
        counts[event_name(RAM_EVENT)] = ram_accesses
        return counts

    canned_runs = []
    real_counters, read_counters = read_counters, lambda argv: canned_runs.pop(0)
    # A window runs until `MINIMUM_WINDOW_SECONDS` has passed, so the clock is what says how many
    # runs it holds, and the last reading is what the sibling figure is divided by.
    clock_readings = iter([0.0, MINIMUM_WINDOW_SECONDS / 2, MINIMUM_WINDOW_SECONDS])
    real_monotonic, time.monotonic = time.monotonic, lambda: next(clock_readings)
    try:
        canned_runs[:] = [canned_run(70, 4, 5, 11, 8), canned_run(90, 3, 4, 10, 5)]
        window, _sibling_busy = read_window(None)
        assert window == Counts(instructions=10, ram_accesses=5, splits=7, cycles=70), window
    finally:
        read_counters, time.monotonic = real_counters, real_monotonic

    # Only the windows the sibling stayed out of reach the cycle count, and where none did there is
    # no count to report. A window the sibling was busy for exactly the limit is one it stayed out
    # of. The window left out below carries the lowest count of the three, so `measure` letting it
    # through would show up here as that count. The other three counts are the lowest of every
    # window, including the ones the sibling was busy through.
    canned_windows = []
    real_window, read_window = read_window, lambda argv: canned_windows.pop(0)
    try:
        canned_windows[:] = [
            (Counts(instructions=11, ram_accesses=5, splits=9, cycles=90), 0.0),
            (Counts(instructions=10, ram_accesses=3, splits=7, cycles=70), 1.0),
            (Counts(instructions=12, ram_accesses=6, splits=4, cycles=80), SIBLING_BUSY_LIMIT),
        ]
        counts, _contention = measure(None, 3)
        assert counts == Counts(instructions=10, ram_accesses=3, splits=4, cycles=80), counts
        canned_windows[:] = [
            (Counts(instructions=11, ram_accesses=5, splits=9, cycles=90), 1.0),
            (Counts(instructions=10, ram_accesses=3, splits=7, cycles=70), 1.0),
        ]
        counts, _contention = measure(None, 2)
        assert counts == Counts(instructions=10, ram_accesses=3, splits=7, cycles=None), counts
    finally:
        read_window = real_window


def main():
    self_check()
    argv = sys.argv[1:]
    if argv == ["--cpu"]:
        print(cpu_model())
        return
    windows, argv = take_options(argv)
    if not argv:
        sys.exit("usage: perf_counters.py [--windows N] <program> [args...]\n"
                 "       perf_counters.py --cpu")
    counts, contention = measure(argv, windows)
    print(counter_line(counts, contention))


main()
