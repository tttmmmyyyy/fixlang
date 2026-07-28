# Cross-language benchmarks

The same eight programs written in Fix, C and Rust, to place Fix against the languages the
public benchmark suites compare it with. `../speedtest` tracks Fix against its own history and
runs on every change; this directory answers a different question -- how far Fix is from C and
Rust today -- and runs when someone asks.

Each program takes its size as the last argument and prints a result the three languages must
agree on. Check that they do before believing any number:

```
for l in fix c rust; do ./bin_native/fannkuch_$l 10; done
```

`fib` and `loop` print `<language>,<program>,<nanoseconds>,<result>`, so compare the last field
alone there; the other six print the result by itself.

## Running

```
MODE=cachegrind bash build.sh && bash cachegrind.sh              # instruction counts
MODE=native     bash build.sh && bash perf.sh                    # split accesses
MODE=native     bash build.sh && BIN=bin_native python3 wall.py  # wall time, idle machine only
```

Every script takes program names, so a before-and-after on one program costs one program's
time: `MODE=native bash build.sh arrayrw && bash perf.sh arrayrw`.

`build.sh` takes `FIX=<path>` (default: this repository's `target/release/fix`) so two compilers
can be compared by building each into its own `TAG`.

## The three measurements answer different questions

**Instruction counts** come from cachegrind and are deterministic: the same program and input
give the same number whatever else the machine is doing. Fix is built without avx512 because
cachegrind cannot simulate it, and C at `-O2`.

**Split accesses** come from the hardware counters and are deterministic too, so `perf.sh` also
runs on a busy machine. They fill a hole the instruction count leaves: a load or store crossing a
64-byte cache line costs the load/store unit twice while counting as one instruction, so a layout
that splits every other access is invisible to cachegrind and plain in the time. Reading the same
program in Fix and in Rust shows it -- identical instruction sequences, and the counts differ by
four orders of magnitude when one of them starts its data 8 bytes into the allocation.

**Wall time** is what the public sites report, and the instruction count does not predict it.
`fannkuch` executes 2.20x C's instructions and takes 1.08x its time; `fib` executes 1.14x and
takes 1.92x. Both numbers are worth quoting, and neither substitutes for the other.

For wall time every language is tuned for the host CPU (`-march=native`, `-C target-cpu=native`).
Fix enables every feature the host has by default, so comparing it against a plain `gcc -O3`
would credit Fix with vectorized loops for reasons that have nothing to do with the language.
`wall.py` refuses to measure when the machine is busy.

## The programs

| program | what it exercises |
|---|---|
| `fib` | naive recursion: the call sequence and nothing else |
| `loop` | a tight integer loop with a carried dependency and a modulo |
| `binary_trees` | allocation and reference counting |
| `mandelbrot` | float arithmetic in nested loops, no arrays |
| `nbody` | float arithmetic over an array of structs, with `sqrt` through libm |
| `fannkuch` | in-place mutation of an integer array |
| `arrayrw` | in-place mutation at its narrowest: `arr.set(i, arr.@(i) + 1)` over 1000 elements |
| `levenshtein` | a two-row dynamic-programming table over every pair of a thousand words |

`fib` and `loop` time themselves in-process through `bench_clock.c`, with opaque barriers around
the work so the compiler cannot hoist it out of the timed region. The rest are timed from
outside.
