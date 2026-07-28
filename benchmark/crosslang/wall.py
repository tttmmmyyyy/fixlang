#!/usr/bin/env python3
"""Wall-clock time for the binaries `build.sh` produced, as the minimum of N runs.

The public benchmark sites report wall time, and neither instruction counts nor cycles
predict it reliably: `fannkuch` runs 2.20x C's instructions and 1.08x its time, `fib` the
other way round. The minimum is the run least disturbed by everything else on the machine,
but it is still wall time, so this refuses to measure a busy one.

The programs take no arguments -- they carry the input the case fixes -- so process startup
is included. Every comparable case runs for tens of milliseconds against a startup of about
0.3, which is where the cases that carry counterparts are chosen from.

    RUNS=15 python3 wall.py [case ...]
"""

import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
CASES = HERE.parent / "speedtest" / "cases"
LANGS = ["fix", "c", "rust"]
RUNS = int(os.environ.get("RUNS", "15"))
LOAD_LIMIT = float(os.environ.get("LOAD_LIMIT", str(os.cpu_count() / 2)))


def comparable_cases():
    """Cases carrying a counterpart in every language."""
    return sorted(d.name for d in CASES.iterdir()
                  if (d / "ref.c").exists() and (d / "ref.rs").exists())


def best_ms(command, runs):
    times = []
    for _ in range(runs):
        start = time.perf_counter()
        # The programs check their own answer, so a mismatch fails here.
        subprocess.run(command, stdout=subprocess.DEVNULL, check=True)
        times.append((time.perf_counter() - start) * 1000.0)
    return min(times)


def main():
    cases = comparable_cases()
    selected = sys.argv[1:] or cases
    unknown = [c for c in selected if c not in cases]
    if unknown:
        sys.exit(f"no case with ref.c and ref.rs named: {', '.join(unknown)}")
    load = os.getloadavg()[0]
    if load > LOAD_LIMIT:
        sys.exit(f"the machine is busy (load {load:.2f} > {LOAD_LIMIT:.1f}); "
                 f"wall-clock numbers taken now are not worth having. "
                 f"Raise LOAD_LIMIT to measure anyway.")

    print(f"load1={load:.2f}  runs={RUNS}")
    print(f"  {'case':<14} {'lang':<5} {'time':>10} {'vs C':>7} {'vs Rust':>8}")
    for name in selected:
        taken = {}
        for lang in LANGS:
            binary = HERE / "bin" / f"{name}_{lang}"
            if not binary.exists():
                sys.exit(f"no {binary} -- run build.sh first")
            taken[lang] = best_ms([str(binary)], RUNS)
        for lang in LANGS:
            print(f"  {name:<14} {lang:<5} {taken[lang]:9.2f}ms "
                  f"{taken[lang] / taken['c']:6.2f}x {taken[lang] / taken['rust']:7.2f}x")


main()
