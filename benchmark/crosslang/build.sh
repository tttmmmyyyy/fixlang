#!/usr/bin/env bash
# Build every program in Fix, C and Rust into `bin_$TAG/`.
#
#   MODE=cachegrind   flags for `cachegrind.sh`: Fix without avx512, which cachegrind
#                     cannot simulate, and C at -O2 against the baseline instruction set.
#   MODE=native       flags for `wall.py`: every language tuned for the host CPU, which
#                     is the only comparison that is fair on wall-clock time -- Fix enables
#                     every host feature by default, so a plain `gcc -O3` would be losing
#                     vectorized loops for reasons that have nothing to do with the language.
#
#   FIX=<path to the fix binary>   default: the release build in this repository
#   TAG=<name>                     default: the mode
set -euo pipefail
cd "$(dirname "$0")"

MODE=${MODE:-cachegrind}
FIX=${FIX:-../../target/release/fix}
TAG=${TAG:-$MODE}
BIN="bin_$TAG"

case "$MODE" in
  cachegrind)
    FIX_FLAGS=(-O experimental --disable-cpu-feature 'avx512.*')
    CC_FLAGS=(-O2)
    RUSTC_FLAGS=(-O)
    ;;
  native)
    FIX_FLAGS=(-O experimental)
    CC_FLAGS=(-O3 -march=native)
    RUSTC_FLAGS=(-O -C target-cpu=native)
    ;;
  *) echo "MODE must be cachegrind or native" >&2; exit 1 ;;
esac

mkdir -p "$BIN"
echo "== $MODE: $("$FIX" --version 2>/dev/null || echo 'fix (unknown version)')"
gcc -O2 -c bench_clock.c -o bench_clock.o

while read -r name _ _; do
    [ -z "$name" ] && continue
    case "$name" in \#*) continue ;; esac
    # `fib` and `loop` time themselves in-process through a C helper.
    fix_extra=()
    case "$name" in fib|loop) fix_extra=(-b "$PWD/bench_clock.o") ;; esac
    (cd programs && "$FIX" build -f "$name.fix" "${fix_extra[@]}" "${FIX_FLAGS[@]}" \
        -o "$OLDPWD/$BIN/${name}_fix" >/dev/null)
    gcc "${CC_FLAGS[@]}" "programs/$name.c" -o "$BIN/${name}_c" -lm
    rustc "${RUSTC_FLAGS[@]}" "programs/$name.rs" -o "$BIN/${name}_rust" 2>/dev/null
    echo "   $name"
done < programs.txt

echo "built into $BIN/"
