#!/usr/bin/env bash
# Split accesses and cycles for the binaries `build.sh` produced.
#
# A load or store that crosses a 64-byte cache line costs the load/store unit twice, and
# cachegrind's model cannot express it: the instruction count is the same either way. The
# split count is deterministic, so this runs on a busy machine; the cycle count beside it
# is not, and is only worth reading when the machine is idle.
#
# Naming cases measures only those.
set -euo pipefail
cd "$(dirname "$0")"

source ./common.sh
select_cases "$@"

COUNTERS=../speedtest/perf_counters.py

printf "  %-14s %-5s %14s %14s\n" "case" "lang" "splits" "cycles"
for name in $(comparable_cases); do
    is_wanted "$name" || continue
    for lang in fix c rust; do
        binary="bin/${name}_${lang}"
        [ -x "$binary" ] || { echo "no $binary -- run build.sh first" >&2; exit 1; }
        out=$(python3 "$COUNTERS" "$binary") || { echo "  $name $lang: unavailable"; continue; }
        # splits,cycles,contention
        rest=${out#*,}
        printf "  %-14s %-5s %14s %14s\n" "$name" "$lang" "${out%%,*}" "${rest%%,*}"
    done
done
