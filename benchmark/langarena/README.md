# LangArena

What fifty whole programs cost to run, read with the counters `../speedtest` reads.

`../speedtest` measures programs written to isolate one thing the compiler does. These are whole
programs — a JSON parser, a Huffman coder, a maze solver, a raytracer — each checking its own
answer against a checksum. A change the speedtest cases call neutral can still move these, and a
change that moves these is one a user feels.

The programs are not kept here. They live in [LangArena](https://github.com/tttmmmyyyy/LangArena),
cloned at the revision `PINNED_REVISION` names in `bench.py`. `prepare` is what puts them where
`run` can find them, under `~/fix-langarena-bench` by default.

## Running

```
python3 bench.py prepare                                  # clone the programs, at the pinned revision
python3 bench.py run --label main                         # measure the `fix` on the path
python3 bench.py run --label mine --fix ../../target/release/fix
python3 bench.py compare main mine
```

Rows go to `log.csv`, one per program, labelled with the name the run was given. `compare` reads
two labels out of it and prints what the second costs against the first. Both take `--only` to
name the programs to measure, which is what a before-and-after on one of them costs.

A whole run is about ten minutes: the fifty programs take two and a quarter minutes between them,
and every figure is the lowest reading over five windows.

## The cycle column needs a quiet machine

The instruction and split counts are decided by the program and its input, so they come back the
same whatever else the machine is doing. The cycle count does not: the last level cache is shared
by every core, and a program that reaches main memory pays for what another program put there.
`perf_counters.py` leaves the cycle field empty where other work could have moved it, and the
`contention` column says how much of the machine the run had. Ask for the machine before starting a
run whose cycle column you mean to read.

Two runs of one compiler over all fifty programs, on an idle machine, moved the cycle count by at
most 1.98% and the median by 0.31%, while the instruction count moved by less than 0.0001%. So a
cycle difference under about 2% is this measurement rather than the compiler, and `compare` counts
the programs that moved past that.

**The later of two runs comes out faster.** Of those fifty programs, 41 read fewer cycles the
second time, by 0.35% on the average — so a comparison whose two labels were measured back to back
is biased toward the second by about that much. Measure the first compiler again after the second
(`--label before1`, `--label after`, `--label before2`) and read both comparisons: where they
disagree about a program, that program's difference is the ordering.

## What a figure means

Each row carries four counts, all from `../speedtest/perf_counters.py`:

- **instructions** — retired in user space. The program's own figure, and the one to reach for
  while narrowing a search, since it survives a busy machine.
- **ram** — accesses that missed the last level cache. It says whether this program is one whose
  cycle count another program can move.
- **splits** — loads and stores that crossed a cache-line boundary. These cost real time and an
  instruction count has no notion of them.
- **cycles** — what the machine actually took. The final judgement is this column: on a
  memory-bound core the other three mislead by an order of magnitude.

One program is measured per process, and the whole process is counted. Starting the process,
reading the configuration and writing the answer come to 954 thousand instructions and 630 thousand
cycles, against 1.06 billion and 1.48 billion for the shortest of the fifty, so what a row holds is
the program.

## Raising the pin

A benchmark whose source moves takes its numbers with it, so a row measured before that move
cannot be compared against a row measured after it. Raise `PINNED_REVISION` deliberately, and
re-measure both sides of whatever comparison the old rows were standing in.
