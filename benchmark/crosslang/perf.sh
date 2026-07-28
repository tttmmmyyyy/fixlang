#!/usr/bin/env bash
# Split accesses and cycles for the binaries `build.sh` produced, from the hardware counters.
#
# A load or store that crosses a 64-byte cache line costs the load/store unit twice, and
# cachegrind's model cannot express it: the instruction count is the same either way. The
# split count is deterministic, so this runs on a busy machine; the cycle count beside it
# is not, and is only worth reading when the machine is idle.
#
#   TAG=<name>   which bin_<tag>/ to measure; default `native`, since the split behaviour
#                depends on the vector width and that is the build people run
#
# Naming programs measures only those.
set -euo pipefail
cd "$(dirname "$0")"

source ./common.sh
select_programs "$@"

TAG=${TAG:-native}
BIN="bin_$TAG"
COUNTERS=../speedtest/perf_counters.py

printf "  %-13s %-5s %14s %14s\n" "prog" "lang" "splits" "cycles"
while read -r name full _; do
    [ -z "$name" ] && continue
    case "$name" in \#*) continue ;; esac
    wanted "$name" || continue
    for lang in fix c rust; do
        out=$(python3 "$COUNTERS" "$BIN/${name}_${lang}" "$full") || { echo "  $name $lang: unavailable"; continue; }
        printf "  %-13s %-5s %14s %14s\n" "$name" "$lang" "${out%%,*}" "${out##*,}"
    done
done < programs.txt
