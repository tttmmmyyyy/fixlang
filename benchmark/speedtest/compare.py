#!/usr/bin/env python3
"""Compares the instruction counts of rows of `log.csv`, the record the speedtest harness appends a
row to per run.

Usage: compare.py LOG_CSV BASE OTHER... [--threshold PERCENT]

Each row is named `LABEL:COMMIT:INDEX`: `COMMIT` is the first column of the row (the commit the
measured compiler reports), and `INDEX` picks one of the rows of that commit, counted as a Python
index (`-1` the last, `-2` the one before it), since one compiler is often measured more than once.
The first row named is the base. For each other row, the script prints the geometric mean of the
ratios of its instruction counts to the base's, and how many cases moved beyond the threshold
(0.1% by default), over the cases both rows measured. Then it lists every case that moved in some
row, with its change in each row.

An A-B-A measurement names the two runs of the base compiler and the run of the changed one:

    compare.py log.csv A1:4dc4684:-2 A2:4dc4684:-1 B:337bf4b:-1

The line of A2 against A1 says how much the counts move between two runs of one compiler; a change
of B smaller than that is not a change.
"""

import argparse
import csv
import math
import sys


def instruction_counts(header, row):
    """The instruction count of each case a row measured, by case name."""
    counts = {}
    for i, name in enumerate(header):
        if name.endswith("-inst") and i < len(row) and row[i]:
            counts[name[: -len("-inst")]] = float(row[i])
    return counts


def pick(rows, spec):
    """The label and the instruction counts of the row `spec` (`LABEL:COMMIT:INDEX`) names."""
    try:
        label, commit, index = spec.split(":")
        index = int(index)
    except ValueError:
        sys.exit(f"a row is named LABEL:COMMIT:INDEX, and `{spec}` is not")
    header, body = rows[0], rows[1:]
    of_commit = [row for row in body if row[0] == commit]
    try:
        row = of_commit[index]
    except IndexError:
        sys.exit(f"`{commit}` has {len(of_commit)} rows, so index {index} names none")
    return label, instruction_counts(header, row)


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("log_csv")
    parser.add_argument("rows", nargs="+", metavar="LABEL:COMMIT:INDEX")
    parser.add_argument("--threshold", type=float, default=0.1, metavar="PERCENT")
    args = parser.parse_args()
    if len(args.rows) < 2:
        sys.exit("name a base row and at least one other")

    with open(args.log_csv, newline="") as f:
        rows = list(csv.reader(f))
    picked = [pick(rows, spec) for spec in args.rows]
    (base_label, base), others = picked[0], picked[1:]
    cases = [c for c in base if all(c in counts for _, counts in others)]
    limit = args.threshold / 100

    def change(counts, case):
        return counts[case] / base[case] - 1

    print(f"{len(cases)} cases")
    for label, counts in others:
        geomean = math.exp(sum(math.log(1 + change(counts, c)) for c in cases) / len(cases)) - 1
        down = sum(1 for c in cases if change(counts, c) < -limit)
        up = sum(1 for c in cases if change(counts, c) > limit)
        print(
            f"{label} vs {base_label}: geomean {100 * geomean:+.2f}%, "
            f"moved beyond {args.threshold}%: {down + up} (down {down}, up {up})"
        )
    moved = sorted(
        {c for _, counts in others for c in cases if abs(change(counts, c)) > limit},
        key=lambda c: change(others[-1][1], c),
    )
    for case in moved:
        cells = " ".join(f"{label} {100 * change(counts, case):+7.2f}%" for label, counts in others)
        print(f"{case:42s} {cells}")


if __name__ == "__main__":
    main()
