#!/usr/bin/env bash
# Build each comparable case in all three languages, tuned for this host.
#
# `../speedtest` measures the same programs under cachegrind on every commit, which is
# deterministic and hardware-independent. This directory answers the other question --
# how long they actually take -- and for that every language has to be allowed the
# instruction set the machine has: Fix enables every host feature by default, so a plain
# `gcc -O3` would be losing vectorized loops for reasons that have nothing to do with the
# language.
#
#   FIX=<path to the fix binary>   default: the release build in this repository
#
# Naming cases builds only those, which is what a before-and-after on one case wants.
set -euo pipefail
cd "$(dirname "$0")"

source ./common.sh
select_cases "$@"

FIX=${FIX:-../../target/release/fix}
if [ ! -x "$FIX" ]; then
    echo "no fix binary at $FIX -- build one with \`cargo build --release\`, or set FIX=<path>" >&2
    exit 1
fi
FIX=$(realpath "$FIX")
BIN=$PWD/bin
mkdir -p "$BIN"
echo "== building with $FIX"

for name in $(comparable_cases); do
    is_wanted "$name" || continue
    (cd "$CASES/$name" \
        && "$FIX" build -f main.fix -O experimental -o "$BIN/${name}_fix" >/dev/null \
        && gcc -O3 -march=native ref.c -o "$BIN/${name}_c" -lm \
        && rustc -O -C target-cpu=native ref.rs -o "$BIN/${name}_rust" 2>/dev/null)
    echo "   $name"
done

echo "built into bin/"
