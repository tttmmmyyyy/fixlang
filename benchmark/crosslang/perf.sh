#!/usr/bin/env bash
# Split accesses and cycles for the binaries `build.sh` produced.
#
# A load or store that crosses a 64-byte cache line costs the load/store unit twice, and an
# instruction count has no notion of it: the count is the same either way. The
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
        if ! out=$(python3 "$COUNTERS" "$binary"); then
            status=$?
            # 2 is the counters answering and the program failing the check it makes of its own
            # answer; anything else is the counters themselves being out of reach.
            if [ "$status" -eq 2 ]; then
                echo "  $name $lang: failed its own check" >&2
                exit 1
            fi
            echo "  $name $lang: unavailable"
            continue
        fi
        IFS=, read -r _instructions _ram splits cycles _contention <<<"$out"
        printf "  %-14s %-5s %14s %14s\n" "$name" "$lang" "$splits" "$cycles"
    done
done
