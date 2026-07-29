# Cross-language benchmarks

How far Fix is from C and Rust today. `../speedtest` tracks Fix against its own history and
runs on every commit; this directory takes the same programs and measures what the other two
do with them.

The programs are not kept here. A case under `../speedtest/cases/` becomes comparable by
carrying `ref.c` and `ref.rs` beside its `main.fix`: the same algorithm, on the same input,
checking the same answer. One Fix source, so the three cannot drift apart, and the
instruction counts land in `log.csv` next to the case's own — which is where the chart draws
them as reference lines.

## Running

```
bash build.sh          # all three languages, tuned for this host
python3 wall.py        # wall time, on an idle machine
bash perf.sh           # split accesses and cycles
```

Every script takes case names, so a before-and-after on one case costs one case's time:
`bash build.sh arrayrw && bash perf.sh arrayrw`. `build.sh` takes `FIX=<path>` so two
compilers can be compared.

Instruction counts come from the speedtest run, not from here.

## Adding a case to the comparison

Write `ref.c` and `ref.rs` next to a case's `main.fix`, on the input the case fixes, ending
in a check of the same expected value — the Fix case asserts, so the counterparts must too,
or a reference that computes something else becomes a number on the chart instead of a
failure.

Only add counterparts to a case whose work dwarfs process startup. Startup is around 0.3 ms
and is included in the wall-clock figure, and cachegrind counts it in the instruction figure
as well; the cases carrying counterparts today all run for hundreds of millions of
instructions, where that is under a percent.

## Counting allocations

`allocations.c` interposes on the allocator and reports the counts at exit, which is the
question a reference-counted language raises constantly and neither of the measurements above
answers.

```
gcc -shared -fPIC -O2 allocations.c -o allocations.so -ldl
LD_PRELOAD=$PWD/allocations.so ./bin/fannkuch_fix
```

## Two measurements, two questions

**Instruction counts** (from `../speedtest`) are deterministic: the same program and input
give the same number whatever else the machine is doing. Fix is built without avx512 there,
because cachegrind cannot simulate it, and the counterparts are built the same way.

**Wall time** is what the public sites report, and the instruction count does not predict it.
`fannkuch` executes 2.20x C's instructions and takes 1.08x its time. Both numbers are worth
quoting, and neither substitutes for the other.

**Split accesses** fill a hole the instruction count leaves: a load or store crossing a
64-byte cache line costs the load/store unit twice while counting as one instruction. The
count is deterministic, so `perf.sh` runs on a busy machine; the cycles beside it do not.
