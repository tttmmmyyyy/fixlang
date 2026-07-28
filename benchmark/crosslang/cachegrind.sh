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

source ./common.sh
select_programs "$@"

TAG=${TAG:-cachegrind}
BIN="bin_$TAG"
OUT="results_$TAG.csv"
CGPY=../speedtest/cachegrind-benchmarking/cachegrind.py

cg() {
    local out
    out=$(python3 "$CGPY" "$@" | tail -1)
    # An empty or non-numeric line means the run never happened -- a missing binary, a
    # program that aborted on this input, no valgrind. Subtracting it would report zero
    # instructions, which reads as the best result in the table instead of as a failure.
    case "$out" in
        [0-9]*,[0-9]*) echo "$out" ;;
        *) echo "measuring $* produced \"$out\"" >&2; return 1 ;;
    esac
}

: > "$OUT"
while read -r name full base; do
    [ -z "$name" ] && continue
    case "$name" in \#*) continue ;; esac
    wanted "$name" || continue
    for lang in fix c rust; do
        bin="$BIN/${name}_${lang}"
        [ -x "$bin" ] || { echo "no $bin -- run build.sh first" >&2; exit 1; }
        out_full=$(cg "$bin" "$full") || exit 1
        out_base=$(cg "$bin" "$base") || exit 1
        IFS=, read -r full_inst full_mem <<<"$out_full"
        IFS=, read -r base_inst base_mem <<<"$out_base"
        [ "$((full_inst - base_inst))" -gt 0 ] || { echo "$name/$lang: the full run cost no more than the trivial one" >&2; exit 1; }
        echo "$name,$lang,$((full_inst - base_inst)),$((full_mem - base_mem))" | tee -a "$OUT"
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
