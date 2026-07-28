#!/usr/bin/env python3
"""Wall-clock time for the binaries `build.sh` produced, reported as work-only time.

The minimum of N runs on the real input, minus the minimum of N runs on a trivial one.
The public benchmark sites report wall time, and neither instruction counts nor cycles
predict it reliably: `fannkuch` runs 2.20x C's instructions but 1.08x its time, and
`fib` the other way round. Subtracting the trivial run removes process startup, which
the languages do not spend equally and which would otherwise pull the ratios of the
short programs toward 1. The minimum is the run least disturbed by everything else on
the machine -- but it is still wall time, so measure on an idle machine.

    BIN=bin_native RUNS=15 python3 wall.py [program ...]
"""

import os
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
LANGS = ["fix", "c", "rust"]
RUNS = int(os.environ.get("RUNS", "15"))
BIN = os.environ.get("BIN", "bin_native")
LOAD_LIMIT = float(os.environ.get("LOAD_LIMIT", str(os.cpu_count() / 2)))


def programs():
    out = {}
    for line in (HERE / "programs.txt").read_text().splitlines():
        line = line.split("#")[0].strip()
        if line:
            name, full, base = line.split()
            out[name] = (full, base)
    return out


def best_ms(cmd, runs):
    times = []
    for _ in range(runs):
        t0 = time.perf_counter()
        subprocess.run(cmd, stdout=subprocess.DEVNULL, check=True)
        times.append((time.perf_counter() - t0) * 1000.0)
    return min(times)


def main():
    progs = programs()
    selected = sys.argv[1:] or list(progs)
    unknown = [p for p in selected if p not in progs]
    if unknown:
        sys.exit(f"not in programs.txt: {', '.join(unknown)}")
    load = os.getloadavg()[0]
    if load > LOAD_LIMIT:
        sys.exit(f"the machine is busy (load {load:.2f} > {LOAD_LIMIT:.1f}); "
                 f"wall-clock numbers taken now are not worth having. "
                 f"Raise LOAD_LIMIT to measure anyway.")
    print(f"load1={load:.2f}  runs={RUNS}  bin={BIN}")
    print(f"  {'prog':<13} {'lang':<5} {'work-only':>10} {'startup':>9} {'vs C':>7} {'vs Rust':>8}")
    for name in selected:
        full, base = progs[name]
        work = {}
        for lang in LANGS:
            cmd = str(HERE / BIN / f"{name}_{lang}")
            hi = best_ms([cmd, full], RUNS)
            lo = best_ms([cmd, base], RUNS)
            if hi - lo <= 0:
                sys.exit(f"{name}/{lang}: the full input cost no more than the trivial one "
                         f"({hi:.2f}ms vs {lo:.2f}ms). Raise the input in programs.txt.")
            work[lang] = (hi - lo, lo)
        for lang in LANGS:
            w, start = work[lang]
            print(f"  {name:<13} {lang:<5} {w:9.2f}ms {start:8.2f}ms "
                  f"{w / work['c'][0]:6.2f}x {w / work['rust'][0]:7.2f}x")


main()
