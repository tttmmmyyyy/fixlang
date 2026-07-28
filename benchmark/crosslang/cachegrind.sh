#!/usr/bin/env bash
# Instruction counts for the binaries `build.sh` produced, from cachegrind.
#
# Cachegrind counts the whole process, so each binary is run twice -- with the real
# input and with a trivial one -- and the difference is reported: the marginal cost of
# the work, comparable across languages. The counts are deterministic whatever else the
# machine is doing, which is what makes this the measurement to run first.
#
#   TAG=<name>   which bin_<tag>/ to measure; default `cachegrind`
#
# Naming programs measures only those.
set -euo pipefail
cd "$(dirname "$0")"

SELECTED=("$@")
wanted() {
    [ ${#SELECTED[@]} -eq 0 ] && return 0
    printf '%s\n' "${SELECTED[@]}" | grep -qx "$1"
}

TAG=${TAG:-cachegrind}
BIN="bin_$TAG"
OUT="results_$TAG.csv"
CGPY=../speedtest/cachegrind-benchmarking/cachegrind.py

cg() { python3 "$CGPY" "$@" 2>/dev/null | tail -1; }

: > "$OUT"
while read -r name full base; do
    [ -z "$name" ] && continue
    case "$name" in \#*) continue ;; esac
    wanted "$name" || continue
    for lang in fix c rust; do
        bin="$BIN/${name}_${lang}"
        IFS=, read -r fi fm <<<"$(cg "$bin" "$full")"
        IFS=, read -r bi bm <<<"$(cg "$bin" "$base")"
        echo "$name,$lang,$((fi - bi)),$((fm - bm))" | tee -a "$OUT"
    done
done < programs.txt

echo
echo "== work-only cost, startup subtracted ($TAG) =="
awk -F, '{ inst[$1","$2]=$3; mem[$1","$2]=$4; if (!seen[$1]++) order[++n]=$1 }
END {
    printf "  %-13s %-5s %14s %8s %16s %8s\n", "prog", "lang", "inst(Ir)", "vs C", "mem(est)", "vs C"
    for (i = 1; i <= n; i++) {
        p = order[i]
        for (j = 1; j <= 3; j++) {
            l = (j == 1 ? "fix" : (j == 2 ? "c" : "rust"))
            printf "  %-13s %-5s %14d %7.2fx %16d %7.2fx\n", p, l, inst[p","l],
                   inst[p","l] / inst[p",c"], mem[p","l], mem[p","l] / mem[p",c"]
        }
    }
}' "$OUT"
