"""Measure a case's C or Rust counterpart the way the case itself is measured.

A case may carry `ref.c` and `ref.rs`: the same program on the same input, checking the
same answer. Measured under the same cachegrind, they give the Fix line a reference to be
read against -- how far the language is from C on that program, tracked over time rather
than sampled once.

Run from inside a case directory. Prints `<inst>,<mem>`, or exits 2 when the case carries
no counterpart for the language asked for.

    python3 reference.py <c|rust>
"""

import subprocess
import sys
from pathlib import Path

CACHEGRIND = Path(__file__).resolve().parent / "cachegrind-benchmarking" / "cachegrind.py"

# The Fix case is built for this host with avx512 left out, since cachegrind cannot
# simulate it. The counterparts get the same deal, so the comparison is between the
# languages rather than between the instruction sets they were allowed to use.
BUILD = {
    "c": (["gcc", "-O3", "-march=native", "-mno-avx512f", "ref.c", "-o", "ref_c", "-lm"], "ref.c", "ref_c"),
    "rust": (["rustc", "-O", "-C", "target-cpu=native", "-C", "target-feature=-avx512f",
              "ref.rs", "-o", "ref_rust"], "ref.rs", "ref_rust"),
}


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in BUILD:
        sys.exit("usage: reference.py <c|rust>")
    command, source, binary = BUILD[sys.argv[1]]
    if not Path(source).exists():
        sys.exit(2)

    build = subprocess.run(command, capture_output=True, text=True)
    if build.returncode != 0:
        sys.exit(f"building {source} failed:\n{build.stderr.strip()}")

    # The counterpart checks its own answer, so a reference that drifted away from the case
    # fails here instead of quietly becoming a number on the chart.
    measured = subprocess.run(["python3", str(CACHEGRIND), f"./{binary}"],
                              capture_output=True, text=True)
    if measured.returncode != 0:
        sys.exit(f"measuring {binary} failed:\n{measured.stderr.strip()}")
    last = measured.stdout.strip().splitlines()[-1]
    if len(last.split(",")) != 2:
        sys.exit(f"measuring {binary} produced \"{last}\"")
    print(last)


main()
