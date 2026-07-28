"""Run a program under `perf stat` and print the counters cachegrind cannot express.

Prints one line, `<splits>,<cycles>`: loads and stores that crossed a cache-line
boundary, and user-space cycles. Cachegrind's cache model counts references and
misses and has no notion of a line-crossing access, yet those cost real time -- an
array whose elements start 8 bytes into a 16-byte-aligned allocation splits half of
its 32-byte accesses.

Exits non-zero when the counters are unavailable (no hardware PMU, or
`kernel.perf_event_paranoid` above 2), so a caller can leave the columns empty
instead of failing.

    python3 perf_counters.py ./a.out [args...]
"""

import subprocess
import sys

# Two counters per run: the split accesses, and cycles as a diagnostic. Keeping the
# list short matters -- asking for more events than the PMU has counters makes perf
# time-slice them and report scaled estimates.
SPLIT_EVENTS = ["mem_inst_retired.split_loads", "mem_inst_retired.split_stores"]
CYCLE_EVENT = "cycles:u"

ARCH = subprocess.check_output(["uname", "-m"], text=True).strip()


def counters(argv):
    """Event name -> count, for the events perf managed to read."""
    proc = subprocess.run(
        # ASLR off, as cachegrind.py runs it: the split count depends on where the
        # allocator puts the data, so a moving heap would move the number.
        ["setarch", ARCH, "-R", "perf", "stat", "-x,",
         "-e", ",".join(SPLIT_EVENTS + [CYCLE_EVENT]), "--"] + argv,
        stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
    )
    found = {}
    for line in proc.stderr.splitlines():
        fields = line.split(",")
        if len(fields) < 3 or not fields[0].strip().isdigit():
            continue
        # perf appends `:u` to the event name when it may only count user space.
        name = fields[2].strip().removesuffix(":u")
        found[name] = int(fields[0])
    return found, proc.stderr


def cpu_model():
    """The processor these counters would be read on.

    A split count belongs to a microarchitecture the way an instruction count does not,
    so a row measured on another machine cannot be compared with its neighbours. The log
    records this next to the counts.
    """
    for line in open("/proc/cpuinfo", encoding="utf-8"):
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip().replace(",", " ")
    return "unknown"


def main():
    if len(sys.argv) == 2 and sys.argv[1] == "--cpu":
        print(cpu_model())
        return
    if len(sys.argv) < 2:
        sys.exit("usage: perf_counters.py <program> [args...]\n"
                 "       perf_counters.py --cpu")
    found, report = counters(sys.argv[1:])
    missing = [e for e in SPLIT_EVENTS + [CYCLE_EVENT.removesuffix(":u")] if e not in found]
    if missing:
        # Say which of the two it was: the program never ran, or the counters are out of reach.
        sys.exit(f"perf reported none of {', '.join(missing)}. perf said:\n" + report.strip())
    splits = sum(found[e] for e in SPLIT_EVENTS)
    print(f"{splits},{found[CYCLE_EVENT.removesuffix(':u')]}")


main()
