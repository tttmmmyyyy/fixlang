# rc-fuzz

Random testing of the reference-counting optimizations (the RC IR passes under `src/rc_ir/`).

A wrong optimization of reference counting shows itself as a use after free, a leak, a double
free, or an update made in place on an object that another reference still reads. The test suite
reaches the program shapes someone wrote a test for. This tool generates many small programs that
read values out of boxed containers, update the containers in place or after a copy, and then use
the values read, and checks each under Valgrind MemCheck and against the output of a reference
compiler.

## Scripts

- `gen.py SEED` prints one Fix program, determined by the seed. With `FUZZ_DENSE=1` in the
  environment, the statements lean toward reads followed by updates, branches and calls.
- `run.py` generates the programs of a range of seeds, builds each with the compiler under test and
  with the reference compiler, runs the first under MemCheck and the second natively, and records
  the result of each seed.
- `reduce.py` shrinks a failing program, dropping top-level definitions and then lines, as long as
  it keeps failing in the same way.
- `common.py` holds the building and running that `run.py` and `reduce.py` share.

Both compilers build with `-O max` and with AVX-512 and GFNI disabled, since Valgrind does not
implement those instructions.

## Running

A campaign over seeds 0 to 999, with the compiler of this repository under test and a released
`fix` as the reference:

```
python3 tools/rc-fuzz/run.py target/release/fix 0 1000 --reference fix --out ../rc-fuzz-out --jobs 8
```

The compilers are paths or commands on `PATH`. The output directory defaults to `rc-fuzz-out` in
the current directory, and must be given with `--out` when the current directory is inside the
repository. `--stop-on-finding` ends the campaign at its first finding. The Valgrind suppression
file defaults to the repository's `valgrind.supp`; `--suppressions` names another.

Each program is run with three argument lists — none, one, and three arguments — since `main`
computes with their count.

To reduce the program of a finding, give the kind of failure and the arguments it failed with:

```
python3 tools/rc-fuzz/reduce.py target/release/fix ../rc-fuzz-out/123/main.fix min.fix \
    --reference fix --kind memcheck --args x
```

`min.fix` holds the smallest failing program found so far, so it can be read while the reducer
runs. `--kind output` reduces a difference in output instead of a MemCheck error.

## Findings

A seed is a finding when, for one of the argument lists,

- MemCheck reports an error in the program of the compiler under test (`memcheck`),
- the two programs exit with different codes (`exit`),
- the two programs print different output (`output`), or
- either program dies of SIGILL (`SIGILL_setup`). This one is a fault of the setup: the program
  uses an instruction Valgrind or the machine lacks.

A seed is also a finding when the compiler under test fails to build a program that the reference
compiler builds (`FINDING_build_fail`).

## Results

`run.py` appends one JSON object per seed to `results.jsonl` in the output directory, and keeps
each program as `<seed>/main.fix`. The field `status` is one of:

| `status` | Meaning |
| --- | --- |
| `pass` | Both programs ran clean and printed the same output. |
| `FINDING` | `problems` lists each failure, with its `kind`, its `args`, and the stderr, exit codes or outputs behind it. |
| `FINDING_build_fail` | `log` is the tail of the compiler's output. |
| `ref_build_fail` | The reference compiler failed to build the program, so it was not checked. `log` is its output. |
| `timeout` | A program ran for more than two minutes. |

`outs` lists, per argument list, the output of the reference's program and that of the program
under test. For a seed that passed, the build products are deleted; for any other seed,
`<seed>/t/` holds the build of the compiler under test, with its RC IR in
`t/.fixlang/rc_ir.post.txt`, and `<seed>/r/` that of the reference.

At the end, `run.py` prints the number of seeds of each status:

```
counts {'pass': 6, 'FINDING': 4}
```

## Requirements

- Python 3.
- Valgrind.
- Two `fix` binaries: the compiler under test and a reference compiler whose output is trusted.
