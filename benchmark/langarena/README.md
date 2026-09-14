# LangArena

What fifty whole programs cost to run, in the columns `../speedtest` writes.

`../speedtest` measures programs written to isolate one thing the compiler does. These are whole
programs — a JSON parser, a Huffman coder, a maze solver, a raytracer — each checking its own
answer against a checksum. A change the speedtest cases call neutral can still move these, and a
change that moves these is one a user feels.

The programs are not kept here. They live in [LangArena](https://github.com/tttmmmyyyy/LangArena),
cloned at the revision `PINNED_REVISION` names in `bench.py`, under `~/fix-langarena-bench` by
default.

## Running

They are measured by the speedtest driver, which asks for them when it is given `--langarena`:

```
cd ../speedtest && ./a.out --langarena
```

Their counts go into the same row of `../speedtest/log.csv` as the cases, under columns named after
each program (`Hash::SHA256-inst`, `Hash::SHA256-cycles`, ...), and `graph_html.py` draws them
beside the cases.

A run with `--langarena` takes about fifteen minutes, against the two a run without it takes: two
minutes to build the programs and ten to measure them, since each of the fifty is measured in a
process of its own over five windows. That is why they are asked for rather than measured every
time.

`bench.py` can also be run on its own, which is what a before-and-after on one program costs:

```
python3 bench.py prepare
python3 bench.py build --fix ../../target/release/fix
python3 bench.py measure --only "Hash::SHA256"
```

`measure` prints one line per program, `<name>,<instructions>,<ram>,<splits>,<cycles>,<contention>`,
and writes no log of its own.

## The cycle column needs a quiet machine

The instruction and split counts are decided by the program and its input, so they come back the
same whatever else the machine is doing. The cycle count does not: the last level cache is shared
by every core, and a program that reaches main memory pays for what another program put there.
`perf_counters.py` leaves the cycle field empty where other work could have moved it, and the
`contention` column says how much of the machine the run had. Ask for the machine before starting a
run whose cycle column you mean to read.

Two runs of one compiler over all fifty programs, on an idle machine, moved the cycle count by at
most 1.98% and the median by 0.31%, while the instruction count moved by less than 0.0001%. So a
cycle difference under about 2% is this measurement rather than the compiler.

**The later of two runs comes out faster.** Of those fifty programs, 41 read fewer cycles the
second time, by 0.35% on the average — so a comparison between two rows measured back to back is
biased toward the second by about that much. Measure the first compiler again after the second and
read both comparisons: where they disagree about a program, that program's difference is the
ordering.

## What a figure means

Each program fills four columns, all from `../speedtest/perf_counters.py`:

- **`-inst`** — instructions retired in user space. The program's own figure, and the one to reach
  for while narrowing a search, since it survives a busy machine.
- **`-ram`** — accesses that missed the last level cache. It says whether this program is one whose
  cycle count another program can move.
- **`-splits`** — loads and stores that crossed a cache-line boundary. These cost real time and an
  instruction count has no notion of them.
- **`-cycles`** — what the machine actually took. The final judgement is this column: on a
  memory-bound core the other three mislead by an order of magnitude, and a change can move this
  one while leaving the instruction count where it was.

One program is measured per process, and the whole process is counted. Starting the process,
reading the configuration and writing the answer come to 954 thousand instructions and 630 thousand
cycles, against 1.06 billion and 1.48 billion for the shortest of the fifty, so what a column holds
is the program.

## Raising the pin

A benchmark whose source moves takes its numbers with it, so a row measured before that move cannot
be compared against a row measured after it. Raise `PINNED_REVISION` deliberately, say in
`../speedtest/history.md` which rows sit on which side, and re-measure both sides of whatever
comparison the old rows were standing in.
