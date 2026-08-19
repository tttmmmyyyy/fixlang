"""Build and measure a case's C or Rust counterpart the way the case itself is measured.

A case may carry `ref.c` and `ref.rs`: the same program on the same input, checking the
same answer. Measured under the same cachegrind and the same hardware counters, they give
the Fix line a reference to be read against -- how far the language is from C on that
program, tracked over time rather than sampled once.

Building and measuring are separate commands so that the harness can get every build out
of the way before it reads a counter: the cycle count is dropped on a machine that is busy,
and a compiler running between two measurements is what makes it busy.

Run from inside a case directory. Exits 2 when the case carries no counterpart for the
language asked for.

    python3 reference.py build <c|rust>
    python3 reference.py measure <c|rust> [--repeat N]

`measure` prints `<inst>,<mem>,<ram>,<splits>,<cycles>,<contention>`. The last three come back
empty, empty and `0.00` where the hardware counters are out of reach, as they do for the case
itself.
"""

import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHEGRIND = HERE / "cachegrind-benchmarking" / "cachegrind.py"
PERF_COUNTERS = HERE / "perf_counters.py"

# The Fix case is built for this host with avx512 left out, since cachegrind cannot
# simulate it. The counterparts get the same deal, so the comparison is between the
# languages rather than between the instruction sets they were allowed to use.
BUILD = {
    "c": (["gcc", "-O3", "-march=native", "-mno-avx512f", "ref.c", "-o", "ref_c", "-lm"], "ref.c", "ref_c"),
    "rust": (["rustc", "-O", "-C", "target-cpu=native", "-C", "target-feature=-avx512f",
              "ref.rs", "-o", "ref_rust"], "ref.rs", "ref_rust"),
}


def source_and_binary(language):
    """The counterpart's source file and program file for `language`, exiting 2 where this
    case carries no counterpart in it."""
    _command, source, binary = BUILD[language]
    if not Path(source).exists():
        sys.exit(2)
    return source, binary


def build(language):
    """Compile the counterpart for `language`."""
    command, source, _binary = BUILD[language]
    source_and_binary(language)
    built = subprocess.run(command, capture_output=True, text=True)
    if built.returncode != 0:
        sys.exit(f"building {source} failed:\n{built.stderr.strip()}")


def measure(language, repeat):
    """The counters for the counterpart of `language`, as one
    `<inst>,<mem>,<ram>,<splits>,<cycles>,<contention>` line.

    # Arguments
    * `repeat` - how many times the hardware counters are read; the cycle count reported is
      the lowest of them.
    """
    _source, binary = source_and_binary(language)
    # The counterpart checks its own answer, so a reference that drifted away from the case
    # fails here instead of quietly becoming a number on the chart.
    simulated = subprocess.run(["python3", str(CACHEGRIND), f"./{binary}"],
                               capture_output=True, text=True)
    if simulated.returncode != 0:
        sys.exit(f"measuring {binary} failed:\n{simulated.stderr.strip()}")
    cachegrind = simulated.stdout.strip().splitlines()[-1]
    simulated_counts = cachegrind.split(",")
    if len(simulated_counts) != 3:
        sys.exit(f"measuring {binary} produced \"{cachegrind}\"")
    instructions, _memory_accesses, dram_accesses = simulated_counts
    # A machine without the counters leaves these three fields the way the case's own
    # measurement leaves them, so a row is short of the same columns on both lines.
    counted = subprocess.run(
        ["python3", str(PERF_COUNTERS), "--repeat", str(repeat),
         # What the counterpart asks of main memory decides whether its cycle count survives
         # a busy machine, the same way it does for the case.
         "--dram-accesses", dram_accesses, "--instructions", instructions,
         f"./{binary}"],
        capture_output=True, text=True)
    hardware = counted.stdout.strip() if counted.returncode == 0 else ",,0.00"
    return f"{cachegrind},{hardware}"


def main():
    argv = sys.argv[1:]
    repeat = 1
    if len(argv) >= 2 and argv[-2] == "--repeat":
        repeat = int(argv[-1])
        argv = argv[:-2]
    if len(argv) != 2 or argv[0] not in ("build", "measure") or argv[1] not in BUILD:
        sys.exit("usage: reference.py build <c|rust>\n"
                 "       reference.py measure <c|rust> [--repeat N]")
    if argv[0] == "build":
        build(argv[1])
    else:
        print(measure(argv[1], repeat))


main()
