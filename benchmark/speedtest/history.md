# Benchmark History

Newer is above.

**Rows measured before `cachegrind.py` fixed the environment are not comparable with rows after
it.** Cachegrind counts the dynamic loader and libc start-up along with the program, and both walk
the environment, so a row carried about 600 instructions per variable the shell that ran the
harness happened to export — a background run and an interactive one differed by tens of thousands
of instructions on every case. The measured command now runs with a fixed minimal environment, and
the `startup` case records what a program that does nothing costs, so a row says how much of each
figure was there before any of the work.

**A cycle count no longer waits for an idle machine, and the rows read the new way stay
comparable with the rows below them.** The count is taken from the windows of runs that the other
thread of the measurement's core stayed out of, and it is kept for a case whose data does not come
from main memory often enough for another program to take the cache from it. The same C
counterpart, byte for byte, read 203,745,780 cycles under 11 cores of other work and 203,299,270
with the machine to itself, a difference of 0.22%. The `-ram` columns carry the main-memory
accesses the cache condition reads.

**The split columns are comparable from `b2de6116d89ff9d43449c2a12fe5c29dd1304bb4` down, and not
across it.** The counters were read with whatever environment the harness inherited until that row,
and a split count moves with the environment for the reason given there.

## 30176a1ecd13001de31148dde1573928bcb172d9

The first row carrying the five combinator-chain cases — `iter_map`, `iter_map_map`, `iter_filter`,
`iter_filter_map` and `iter_flatten`, a million elements each. The corpus had no case where the
function an iterator carries sits in a field, which is what #450 is about, so nothing here moved
with the work on it.

Built with the merge base instead, the same five cases read 17,083,788 / 25,084,087 / 19,091,760 /
19,593,750 / 24,346,695 instructions, so this row stands at -75.00%, -81.98%, -39.30%, -38.29% and
-28.73%.

`iter_flatten` carries one split store per element (999,017 against the merge base's 15), and its
cycles fall by 12.1% while its instructions fall by 28.7% — the straddling store is eating part of
the win. The other four chains stay at 24 splits.

`cp_lib_bipartite` and `push_back` came away without a cycle count under 0.65 cores of other work.

## dde0bf3c1af23804a56e5277e5fb5395c200b978

Splitting a struct argument into one argument per field, and reading a construction where the code
taking it apart can see it (#450, #452, PR #473), measured against `c445cc15`, the fork point, whose
row is recorded beside it. Both rows were measured on this machine, one after the other.

**Five cases move, and every one of them gets faster.** `bounds_check_indexable` reads and writes a
two-dimensional array through `Indexable`, and the indirect call it made per element is gone, so LLVM
folds the inner loop into a copy of the row: 71,268,374 instructions to 2,271,451, **-96.81%**.
`sum_by_fix` -3.01%, `cp_lib_segtree` -0.40%, `cp_lib_unionfind` -0.37%, `cp_lib_lsegtree` -0.08%.
The other forty-six are identical to the instruction.

Six cases read one instruction column apart by exactly fourteen instructions in each direction,
`startup` among them. Fourteen is what `startup` itself moved by, so it is the harness environment
and not the corpus.

**Both rows came away short of cycle counts** — twenty-four of fifty-one on the new row and
twenty-five on the old, with other work taking 1.2 and 1.8 cores while they ran. The instruction and
memory columns are read under cachegrind and do not move with the machine, so the comparison above
stands on those.
## 8440ee5f070f0f8f2abd7118711d5ab9f12f4575

「共有だと観測してから行動する」が原子的でない件 (#461) の修正を、fork point の `fffebab4` と並べて
測った。両方の行がこの下に在る。

**命令数の動いた 22 ケースのうち、ゆらぎでないのは 1 つだけ。** `cp_lib_scc` が **-299,830 命令
(-0.20%)**、メモリ参照も -299,146。`Std::FFI::Destructor` の解放から、減算の前に置かれていた一意性の検査
(状態の load と比較と分岐) が消えた分である。残る 21 ケースの差は最大 233 命令で、`startup` を含む
±14 の帯は #291 / #342 が記録している実行ごとのゆらぎ。

**配列側の修正はコーパスを動かさない。** 共有された記憶域の解放が最後になり得るのは他のスレッドが
参照を落とせるときだけなので、要素の走査は threaded なビルドで、かつ記憶域が local と証明されていない
ときにだけ出す。コーパスは全部が単一スレッドなので、生成されるコードは fork point と同じである。
**参照を持たない要素型で走査と retain を出さなくした分も、この水準では中立である** — `-O experimental`
では LLVM が同じものを消すので、命令数が動いたのは上の 1 ケースだけだった (`-O none` では効く:
100,000 要素の `Array I64` を 100 本、解放が 220,630,800 -> 130,629,002 命令、複製が
161,899,537 -> 141,899,523 命令)。

**条件を付けない版は `cp_lib_bipartite` を +1,433,506 命令 (+0.61%) 動かした。** 差は全部
`CPLib.MaxFlow::Dinic::dfs` の中にあり (173,041,661 -> 174,475,184)、他の関数は 1 命令も動いていない。
足した走査は**一度も実行されない** — 走査の先頭で abort するようにしたプログラムが最後まで走る — ので、
これは冷たい枝が 230 バイト太った (`dfs` が 5,235 -> 5,465 バイト) ことで hot なループのレジスタ割り当てが
変わった分である。**同じケースは、走査の代わりに abort を置いた 3 つ目の版では -0.32% 動いた**:
この関数の命令数は、この領域のコードの大きさに対して ±0.6% の幅で動く。

**2 つの行のサイクル列は互いに比較できない。** fork point の行は他の仕事が 2.99 コアを使っている間に、
この行は 0.72 コアの間に測った。両方に残った 11 ケースは -4.11% から +0.16% までばらつき、11 件中 8 件が
負で、命令数が 1 命令も動いていない `nbody` が -2.36% を示している。これは変更ではなく機械の状態である。

## 1334dac79d3b0cac55a4f0c622a23b9d793137e3

値のオブジェクトへ印を付けるとき、経路ごとではなくオブジェクトごとに 1 度だけ訪れるようにした変更 (#433) を、
fork point の `0343f84b` と並べて測った。両方の行がこの下に在る。

**コーパスは動かない。** 命令数の動いた 38 ケースで、最大の絶対差は **18 命令** (`cp_lib_bipartite`、
2 億 3644 万分の 18)、最大の相対差は **`startup` の 0.0123%** (113,647 -> 113,633、14 命令) である。
`startup` は `pure()` を返すだけで、印付けも解放も持たない — **この変更が触れようのないケースが、相対では
最も大きく動いている**。これは #291 / #342 が記録している実行ごとのゆらぎで、ランタイムの C ソースが
乱数の名前を持ち、その名前がシンボルを通って動的ローダの仕事を変えることによる。メモリ参照の最大の差は
`index_syntax` の -8,208 (0.0014%)。

**足した検査は最適化で消える。** `gen_random_array` (グローバルの配列に印を付ける) を両方のコンパイラで
`--emit-llvm` してビルドすると、**最適化後の LLVM IR がバイト単位で一致する**。確保直後のオブジェクトは
状態が分かっているので、印付けが読む load と比較と分岐は定数畳み込みで消える。生成直後の IR には差が出る。
捕捉を持つクロージャのプログラムでも、トラバーサの `switch` の下がり方は 2 つのコンパイラで同一だった
(既定に `unreachable` を置いたので、LLVM は既定に来ないと分かる)。

**この 2 行は、サイクルを取る規則が変わる前のハーネスで測っている** (上の前書きが述べる、忙しい機械でも
サイクルを残す読み方は、この 2 行には掛かっていない)。**したがって 2 行のサイクル列は互いに比較できない。** `0343f84b` の行は contention 0.38 で 51 ケース全部の
サイクルを持つが、`1334dac7` の行は contention 11.47 で 34 ケース分しか残っていない。命令数とメモリ参照は
cachegrind のもので機械の負荷に依存しないので、上の判断はそちらだけに載っている。

**ベースラインは測り直した。** この 2 行の前の最新は `b08926a5` で、fork point の 51 コミット前に在る。

## b08926a54a66d5cb1eb4bb0f4708196bdddb5ab4

Making the RC-IR simplifier's case-of-case rewrite cancel in one step and take a move only when the
result is smaller (#404), measured against `663cfac3`, whose row is recorded beside it. Both rows were
measured on this machine at one path, so the path effect recorded under `342d67aa` is out of the way.

**The corpus does not move.** Forty-three of the fifty-one cases are identical to the instruction and
to the memory reference. The eight that moved are exactly the eight carrying an external dependency
(`cp_lib_*`), and they moved in both directions by at most 168 instructions out of 249 million
(`cp_lib_conv_zp`, 0.0001%) — the per-run wobble #291 and #342 describe, which reaches the dynamic
loader through the names the emitted symbols carry. The `-splits` columns are identical for all fifty
cases that have one.

The RC IR agrees with the rows: built with both compilers, the forty cases with no external dependency
give byte-identical `rc_ir.pre` and `rc_ir.post` dumps once the suffixes of the names case-of-case
mints are stripped, which they must be — the rewrite mints a different number of them.

**The cycle columns of these two rows cannot be compared with each other.** The `663cfac3` row came
away with none of them: other work reached 11.57 cores while the counters were read, and the harness
drops a cycle count it cannot trust. The `b08926a5` row has them for forty-two of the fifty-one cases
(0.99 cores). The instruction and memory columns are cachegrind's and do not depend on the machine,
which is what the paragraph above rests on.

**A baseline was measured rather than taken from the log.** The newest row before these two,
`3d128e6f`, sits eighty-one commits behind the fork point, so reading the corpus against it would have
charged this change with everything in between.

## 941fe0af8c4c72e86472b736724159352e2a954f

Asking the back end to inline every global whose body is small enough to stand where it is called
(#221), measured against `b90ec410`. `fannkuch` -40.1% and `fannkuch_scratch` -38.0%, where a
`Std::loop` copy is called once per permutation and runs 124 instructions per call, so what the
inline removes is the work around the call rather than the loop; `cp_lib_bipartite` -8.8%,
`cp_lib_lsegtree` -8.7%, `cp_lib_scc` -2.4%, `cp_lib_unionfind` -0.7%, all in the library code
these cases call. No case moved the other way.

## b90ec410b0466aa3463fdcbcebeeb1b76e863d56

Unwrapping a newtype where a field type is read instead of specializing the declaration in advance
(#245), measured against `342d67aa`.

**The corpus does not move.** The two compilers emit the same machine code: all 51 cases were built
with both and their `--emit-llvm` and `--emit-rc-ir` output is byte-identical, with only the module
hash normalized, so the symbol hashes were compared as they stand. The rows agree with that. Every
instruction count that moved did so by about fourteen instructions, in both directions, which is the
build path reaching the dynamic loader through the module hash — the same effect recorded under
`342d67aa`, and here it lands on more cases because both rows were measured in different worktrees.
The largest movement is 0.0068% (`sum_by_loop_iter_cap`, 206,383 to 206,397).

**A row could not be written between `9b002b1b` and this one.** Row 107 of `log.csv` held two rows
joined at a missing newline, so it carried 549 of the 275 columns the header names, and the harness
checks that before writing: a run reached the check after measuring every case and aborted there.
The join was undone by keeping the first of the two, the second being the row that already follows.

## 342d67aa7f2360bfdd9ab22125b0d2d11cd33e85

Deciding once which one-field unboxed structs are replaced by their field (#231, #234), measured
against `8a883f8d`, whose row is recorded beside it.

**The corpus does not move.** The only type constructor whose classification changes is the form of
`Std::DynIterator` with its one field punched out — the type `mod_` and `act_` hold the rest of the
struct in while the field is out — and no case reaches it. `cp_lib_scc` compiled by the two
compilers is byte-identical in `.text` and in `.rodata`, and cachegrind gives it the same
150,092,862 instructions and 234,694,884 memory references.

**Two rows measured in two worktrees differ on the cases that carry an external dependency, by
about a ten-thousandth of a percent, and that difference is the paths they were built at.** Here the
eight `cp_lib_*` cases moved by up to 0.0005% while the other forty-three were identical to the
instruction. A build path feeds the module hash that names the emitted symbols, and cachegrind
counts the dynamic loader resolving them. Measuring both commits at one path removes it.

**Two executables built by the same compiler are not byte-identical**, so a whole-file hash is not
the differential to run. The runtime's C source is written to a temporary file whose name carries a
random number, and that name reaches `.strtab`; two builds of one case differ there by a few bytes
and nowhere else. Compare `.text`.

## 598a391a50af0a37273e9c7bdbd3426d5dfdf277

Closure specialization carried along a chain of copies, measured against `4052838928`. A lambda
handed to a function is lifted into a global function and a capture list, and the function receiving
it is copied so that the call goes by name. The chain used to stop at the first step that stored the
value into a capture list instead of passing it on, which is where a sort's comparison goes: the
lambda handed to `fold` captures it. Narrowing that capture field's type to the capture list of what
it holds carries the identity across the step, and the comparison reaches the leaf as a direct call.

Thirteen cases move, twelve of them down: sort_ordered -63.6%, sort -62.0%, sort_few_values -52.1%,
sort_organ_pipe -49.1%, sort_stable_ordered -30.8%, sort_stable -14.2%, cp_lib_conv_zp -5.0%,
cp_lib_bipartite -0.3%, cp_lib_scc -0.1%, and four more by under a thousandth of a percent, of which
cp_lib_unionfind is the one that rises, by 179 instructions. The memory column follows the
instruction column on every one of them.

**`sort` saves what it saved before the sort was rewritten, against a smaller total.** Against the
implementation that #227 and #229 replaced it read 64,037,014 -> 35,921,169, a saving of 28,115,845;
here it reads 49,703,021 -> 18,895,019, a saving of 30,808,002. The five other sort cases entered the
corpus with those two changes, so this is the first row that reads them with the chain followed.

## 4052838995f52c3d8a2ba2ac82fc0e6cb3c02b8a

`Array::sort` spends its recursion budget only on a split that leaves under an eighth of the range
on one side, instead of on every split, and a split that found nothing less than the pivot gathers
everything equal to it beside it and drops all of that from the recursion (#227). The partition's
read of the element it compares no longer checks its bounds, the way the swap beside it already did
not.

**`sort` costs 49,703,021 instructions against 64,037,014, -22.4%, and 70,499,931 memory accesses
against 87,856,297.** `sort_stable` reports the same 35,516,421 as the row below, and every other
case holds to within a thousandth of a percent. Three cases join the corpus, one per shape the
change turns on: `sort_few_values` sorts an input of 16 distinct values at 15,087,278,
`sort_ordered` one already in order at 31,217,869, and `sort_organ_pipe` one that rises to the
middle and falls back at 127,510,119.

Off the corpus, on 1,000,000 elements with `perf stat -e instructions:u` and the generator
subtracted: pseudo-random 663,737,123 -> **511,899,164** (-22.9%), 16 distinct values 483,798,807
-> 110,773,672 (-77.1%), already ordered 398,916,794 -> 372,735,723 (-6.6%), reversed 431,928,778
-> 405,531,368 (-6.1%), boxed elements at 200,000 elements 379,362,768 -> 369,528,805 (-2.6%).

**The organ pipe is what the budget rule is for, and it is where a budget spent wrongly shows up.**
Doubling the budget outright reads pseudo-random at -17.9% and the organ pipe at **+30.0%**
(1,452,057,887 -> 1,887,233,235), because the middle element is the greatest of every sub-range
there and twice the budget is twice as long before heap sort takes over. Spending the budget only
on the one-sided splits keeps the logarithm it always was, and the organ pipe lands at
1,486,423,058, **+2.4%**.

## 6e3c9d0534027c05d806c3a2e42eb8e4c397d7fa

`Array::sort_stable` merges between the array and a copy of it, the two exchanging roles at every
level of the recursion, instead of filling one working buffer and copying it back over the array
(#222). A range of 12 elements or fewer is sorted by insertion; a merge whose two runs are already
in order copies them instead of comparing them; and the merge and the range copies are
tail-recursive functions rather than `Std::loop` bodies.

**Every case of the corpus holds: the largest move is 0.0002%, and `sort` reports the same
64,037,014 instructions as the row below.** No case sorts stably, which is why two join here:
`sort_stable` sorts a pseudo-random 100,000-element `Array I64` at 35,516,421 instructions, and
`sort_stable_ordered` sorts one already in order at 10,090,283, that being the only case reaching
the copy the ordered-run check takes. The `startup` case says 113,633 of each was spent before
`main`.

Off the corpus, on 1,000,000 elements with `perf stat -e instructions:u` and the generator
subtracted: pseudo-random 2,263,395,952 -> **373,925,715** (-83.5%), already ordered 1,571,133,487
-> 90,713,298 (-94.2%), reversed 1,579,854,830 -> 252,954,574 (-84.0%), 16 distinct values
2,229,446,252 -> 367,524,544 (-83.5%), and boxed elements at 200,000 elements 680,478,640 ->
242,621,975 (-64.3%). Sorting stably now costs **0.56 times** what `Array::sort` costs on the same
input.

Halving the moves accounts for a factor of 2.2 of it; the rest is the price of one move. Written as
a `Std::loop` body the merge loop is a closure LLVM leaves out of line, and every element pays a
call that spills six callee-saved registers: the same changes measure 1,450,225,596 with the loop
and 426,179,761 without it. That is the cliff of #221, which this row steps off rather than removes.

## b2c580cd758ba315d530f61c278540fb9e401d36

The primitive that copies a range of one array onto the end of another was split into an owning one
that takes the whole source and a borrowing one that takes a range (#204). `Array::append` calls the
owning one; `Array::get_sub` and `Array::sort_stable` call the borrowing one, which leaves the array
it reads to its caller. Without the reference duplication the old primitive forced on them, a write
after such a copy can have its uniqueness check removed.

`get_sub` **-3.48%** instructions and -3.66% memory accesses, the case the change is for.
`cp_lib_scc` +0.15%: none of the changed call sites is in its hot path, so the movement is code
layout. The other 44 cases hold to within a hundredth of a percent.

**The split column of this row is not comparable with the row below: the two were measured from
different filesystem paths.** A program's initial stack is laid out above its argument and
environment block, so the path the harness is invoked from moves every address on the stack, and a
hot stack object a few bytes from a line boundary crosses it or does not. The `cp_lib_lsegtree`
binary of this row, run unchanged from two directories differing only in the length of their names,
reports **200,024 splits and 23**. Sixteen cases move by more than a twentieth of a percent here for
that reason, `cp_lib_lsegtree` 23 -> 400,026 and `sum_by_fold_cap` 23 -> 1,588 among them; the row
below records the same binaries flipping between the same pairs. **Measure both sides at one path
before reading anything into this column.**

**Not in the corpus: `Array::sort_stable` costs 17.6% more instructions.** On a 2,000,000-element
`Array I64` it goes from 4,049,614,425 to 4,763,914,157, and the rate holds at 100,000 and 500,000
elements. Its RC IR improves — two fewer branches in the operation, one fewer release, and 478 lines
of LLVM-IR down to 409 — but LLVM stops inlining the merge loop's body into `Std::loop`, and 48.7% of
the run is then spent in the function that stays behind. The corpus has no `sort_stable` case.

## b2de6116d89ff9d43449c2a12fe5c29dd1304bb4

The compiler emits the same code as at the row below. Each of the 46 cases was built with both
compilers and the two binaries hold the same machine code: 43 of them byte for byte in `.text`, and
`sort`, `cp_lib_scc` and `cp_lib_bipartite` under a renaming of the numbers a specialization carries
in its symbol. Every instruction count agrees to within a fiftieth of a percent, which is what the
loader and libc cost before `main`.

**Read the split column against this row, not against the ones above it.** `perf_counters.py` gave
the measured command whatever environment the harness inherited, and a program's initial stack is
laid out above the environment block, so every address on the stack moved with how much the caller
happened to export. One unchanged binary reported 70,765 splits from one shell and 170,766 from
another, and `array_mod` reported 23 and 3,153. The counters are now read with the fixed environment
`cachegrind.py` reads its simulation with, which is why `cp_lib_unionfind` (7,374 above, 207,242
here) and `cp_lib_lsegtree` (400,155 above, 23 here) move by a factor with the code unchanged.

**A split count belongs to the way the harness invokes the program.** A case whose hot stack object
sits a few bytes from a line boundary flips by a large factor whenever a frame moves — measured
under one environment but from a longer path, `cp_lib_lsegtree` reports 400,026 and `cp_lib_scc`
70,765 — so a jump of that shape is worth attributing before it is read as a change in what the
program touches.

**What the cycle column reads when nothing changed at all.** The two compilers' binaries, alternated
within one run under one environment, differ by up to 5% on the cases below a few million cycles and
by about 1% on the large ones, with the code identical on both sides. Seven cases carry a cycle count
here; the machine took 11.52 cores for other work while the rest were read.

## 0fa42cadaaac5aa40e2b9bbf1e2cff3ab34502ff

The capacity check that #178 put in front of every array write which clones a shared array is now
emitted only where the program chooses a capacity (#191). The two extra basic blocks per write were
costing the enclosing loop its unrolling.

`nbody` -21.0%, `fannkuch` -10.4%, `cp_lib_lsegtree` -3.5%, `nbody_fold` -2.7%, `cp_lib_conv_zp`
-2.2%, `cp_lib_dijkstra` -0.5%, `cp_lib_unionfind` -0.4%, `fannkuch_scratch` -0.1%;
`cp_lib_bipartite` +0.1%. The other 37 cases hold to within a twentieth of a percent. `nbody` also
stops splitting 6,000,000 of its accesses across a cache line.

**Read the cycle column here for the large cases only.** `fannkuch` -5.9%, `nbody` -5.8% and
`cp_lib_lsegtree` -3.8% follow their instruction counts, and `mutate_boxed_loop` takes 11.3% fewer
cycles for an instruction count that did not move at all -- code that changed size around it landed
differently. Below a few million instructions the column is dominated by what start-up costs and by
run-to-run variation, so the percent figures on the `sum_by_*` cases say nothing.

**`fannkuch_scratch` costs 2.7% of its cycles**, which three back-to-back pairs of builds put
outside the run-to-run range on both sides. Its instruction count moves -0.1% and its split count is
23 either way, so what changed is code layout.

## 61d9cc8f1d540f778a48d369152a5d4c2ead7f67

The first row whose cycle counts were judged by the CPU the measurement had rather than by the
one-minute load average, and the first to carry the hardware counters for the C and Rust
counterparts. Every one of the 46 cases has a cycle count; the run before this change, on the same
idle machine, came away with 18.

The compiler is unchanged from the row below, so the cachegrind columns repeat it. What is new is
what the counters say:

**`nbody` splits 18,000,171 accesses across a cache line where its Rust counterpart splits 390**,
and its C counterpart 26,000,202. `fannkuch` splits 13,063,842 against Rust's 1,814,692 and C's
159 -- while `fannkuch_scratch`, the same problem written against a scratch buffer, splits 152 and
is the one case here that takes fewer cycles than Rust (0.84x).

On cycles the Fix line sits at 1.00x Rust on `arrayrw`, `levenshtein` and `modulo_loop`, 0.96x on
`mandelbrot`, and behind on `binary_trees` (1.12x), `fannkuch` (1.14x), `fib` (1.19x) and `nbody`
(1.45x).

## d55854afbbe68601fe24e05a1a911d92482f8d61

Reference counting reads a byte on the object and branches three ways, because the object may be
local to this thread, shared between threads, or a global that is not counted at all. This row is
the first with the branch gone wherever the compiler can prove the object local (#122, PR #190).

Fourteen cases gain and one loses. `cp_lib_lsegtree` -31.2%, `levenshtein` -5.2%,
`cp_lib_unionfind` -4.4%, `fannkuch` -3.8%, `fannkuch_scratch` -3.7%, `index_syntax` -3.7%,
`cp_lib_bipartite` -3.2%, `cp_lib_scc` -2.4%, `binary_trees` -1.9%, `cp_lib_dijkstra` -1.6%,
`get_sub` -1.0%; `cp_lib_conv_zp` +0.4%. The rest move by under a tenth of a percent.

**Read `nbody` +26.6% and `fannkuch` +6.3% against the row below as belonging to the interval, not
to this change.** No row was taken across the eleven pull requests between them, and the capacity
check #178 put in front of every array write that clones a shared array is what costs those two:
measured on its own, this change moves `nbody` -0.0% and `fannkuch` -3.8%.

**`sort` is the case to read for what is left.** It moves 0.06% against a ceiling of 13%, and that
ceiling is one thing: the comparator closure's capture object, released once per comparison. A
closure keeps a single version, since it is reached indirectly and no call site names it, so no
proof about the caller's data can attach to it. Removing that release is closure specialization's
job, not this pass's — issue #166.

## 383a7b5e18fb9332f9a49cb40be8f85f8cbcc4b8

Follows the `default<O3>` rounds with `speculative-execution`, `loop-vectorize` and `pseudo-probe`
(PR #153), which a search by cycle count kept out of the twelve passes the compiler shipped until
#147.

**Read this row in cycles, not in instructions.** The instruction counts come to +0.05% over the 46
cases, which would have rejected the change: `fib` -4.02% against `sort` +2.50%, `cp_lib_lsegtree`
+1.06%, `cp_lib_scc` +0.77%, `levenshtein` +0.76%, `cp_lib_dijkstra` +0.63%, `fannkuch` +0.52%. What
these three passes change is where the branches fall and how the front end fetches the code, so the
work stays the same and the machine gets through it faster.

Measured by building the same fifteen cases both ways and alternating between the two binaries
within one run, the three are worth **0.80%** of the cycles, and **1.22%** over the five cases held
out of the search: `get_sub` -4.4%, `fib` -4.4%, `cp_lib_dijkstra` -2.8%, `levenshtein` -2.2%,
`fannkuch_scratch` -1.9%, against `cp_lib_lsegtree` +2.0% and `cp_lib_segtree` +1.5%. The three are
one unit — `pseudo-probe` alone costs 0.48%, and 1.21% on the held-out cases.

Against C and Rust, measured on an idle machine over the nine cases that carry counterparts, Fix now
comes to **1.21x C and 1.08x Rust** in cycles and the same in wall clock, the two agreeing case by
case. `fannkuch_scratch` 0.93x C, `mandelbrot` 0.93x, `arrayrw` 0.95x; `fib` 1.74x C and 1.19x Rust,
`binary_trees` 1.59x and 1.15x, `nbody` 1.32x and 1.48x.

**This row's own cycle columns were taken while the machine got busy** — the `load` column reads
13.49, so they are a starting point for later rows rather than something to compare against the row
above. From here `perf_counters.py` leaves the cycle field empty above a load of 2, so a count that
reaches the log is one taken on a quiet machine and the series is sparse by construction.

## daebd8de1544fe7ae2f50abc578980955edb98b7

Bounds how many times inlining rewrites the program (PR #145), so that globals naming each other in
a cycle stop the pass instead of holding it forever.

**Nothing in this row is that change.** The bound is reached only by a program that would not have
finished, and an assertion that it is ever reached was run over all 46 cases here: none reached it,
so the pass performed exactly the rounds it performed before and emitted the same code.

The row is not comparable with the one above it either. Thirty commits landed in between, among them
the LLVM pass pipeline run to a fixpoint (#147) and the change that evaluates a call's arguments in
the order they are written (#140). That is where `levenshtein` (-4.0%), `fib` (-2.4%) and
`binary_trees` (-1.1%) come from.

## 87649b1914a230ade36083b5c693b2f531313578

Runs LLVM's `default<O3>` pipeline to a fixpoint — once at `-O basic`, three times at `-O max` and
`-O experimental` — in place of the twelve hand-picked passes that used to follow a single run. The
whole suite comes to **-2.49%** on the geometric mean of the instruction counts, 22 of the 46 cases
falling and 3 rising.

| case | change | | case | change |
|---|--:|---|---|--:|
| nbody | -21.44% | | sort | -2.15% |
| fannkuch_scratch | -14.47% | | cp_lib_scc | -1.90% |
| fill, fill_from_map | -12.31% | | cp_lib_dijkstra | -1.43% |
| fannkuch | -8.63% | | cp_lib_segtree | -1.31% |
| mandelbrot, mandelbrot_fold | -6.47% | | cp_lib_lsegtree | -1.13% |
| nbody_fold | -5.29% | | arrayrw, arrayrw_fn, struct_field_mod | -0.66% |
| cp_lib_unionfind | -4.08% | | levenshtein | -0.29% |
| random_state | -3.32% | | cp_lib_bipartite | +0.92% |
| option_plumbing | -3.09% | | cp_lib_conv_zp | +0.26% |
| gen_random_array | -2.64% | | cp_lib_prime_list | +0.21% |

A fourth run of the pipeline changes no case by a single instruction, which is what fixes the count
at three.

**These figures are an upper bound on what a machine with avx512 gets.** The suite disables avx512
so that cachegrind can simulate the program, and the extra runs earn much more without it: `nbody`
falls 21.4% here and 6.6% when the same change is measured with avx512 enabled.

**Cycles move far less than instructions.** Over the nine cases of `benchmark/crosslang`, measured
by interleaving the two builds within one run and taking the minimum of thirty rounds, instructions
come to 0.956x and cycles to 0.987x. `nbody` loses 6.6% of its instructions there and no cycles at
all — those instructions were issuing in slots that were going empty. Compile time rises about 14%
at `-O experimental` and stays level at `-O basic`.

This row is also the first that another pipeline can be measured against in the same directory: the
pass pipeline now takes part in the object-file cache key, where before a second build returned the
objects the first had cached, whatever pipeline it was given.

## 423e50e1538e9f4f75708dac436869be871539c7

Evaluates a call's arguments in the order they are written (PR #140), where a prefix call used to
evaluate them backwards at `-O max`. The percentages are against the `a9a1b1a2` row, measured with
the same compiler sources on both sides.

| case | instructions | memory accesses |
|---|--:|--:|
| fib | -6.35% | -4.80% |
| levenshtein | -2.87% | -2.57% |
| binary_trees | -1.07% | +0.35% |
| cp_lib_unionfind | -0.37% | -0.31% |
| sort | +0.05% | 0.00% |
| cp_lib_bipartite | +0.35% | +0.15% |
| cp_lib_conv_zp | +0.74% | +1.00% |

Every other case moves less than 0.05%; the 46 together retire 0.26% fewer instructions. `fib` is the
case the order decides: LLVM's tail-recursion elimination folds the *last* call into a loop, so which
of `fib(n - 1)` and `fib(n - 2)` goes last picks the decomposition, and the written order picks the
one with fewer leaf calls.

**The splits column of the `a9a1b1a2` row is not comparable with this one.** It reads as a 90% fall on 41
of the 46 cases, including `startup`, whose instruction count is identical in the two rows — a program
that does nothing cannot have lost 152 split accesses to an evaluation-order change. Measured
back-to-back at one path, the two compilers give the same count: `startup` 16 and 16, `arrayrw` 17 and
17, `sum_by_fold` 23 and 23. The counter is repeatable within a run (five runs, one value) and perf
reports it 100% enabled. `perf_counters.py` reads the count and ignores the enabled percentage perf
prints beside it, so a run whose events the PMU time-sliced enters the log as a scaled estimate that
looks like any other measurement -- which is the condition that produces a tenfold column.

## d51e4a2eeaf179d01e5a918974b3a28e40dfbb3f

Removes two latent defects from the substitutor that rewrites free names (PR #127): a rewrite the
substitutor reported as unchanged, which the enclosing `let` or `match` then discarded, and a `let`
the inline-LLVM substitution introduces capturing a name that another replacement reads.

Neither is reachable through the compiler's own passes, so this row is here to show that the code
generated for these cases is the same. It is: no case moves by more than 0.05% in instructions or
in memory accesses, and the totals over the 46 cases move by -0.0000% and -0.0001%. The largest
movements are around fourteen instructions, on the cases small enough for that to register
(`startup`, `sum_by_fix`).

The split-access column drops by roughly 230 on nearly every case, `startup` included. A case that
runs no code of its own cannot have gained that from a compiler change, so read it as process
start-up rather than as anything this row measures.

## a9a1b1a2bd93952205e127f3cbe603d2e6a6c2c0

Starts a large array's elements on a 32-byte boundary (PR #128), so that a vectorized loop over them
stops straddling cache lines.

**Read this row in the split columns, not the instruction counts.** The instruction count cannot see
either the straddle or its removal: `arrayrw` retires the same instructions before and after, to
within eleven out of 120 million, and runs 1.71 times faster. What the splits say:

| case | before | after |
|---|--:|--:|
| arrayrw | 49,600,017 | 171 |
| arrayrw_fn | 49,600,017 | 171 |
| struct_field_mod | 49,600,019 | 171 |
| cp_lib_bipartite | 54,743,346 | 76,832 |
| fill, fill_from_map | 1,250,023 | 249 |
| cp_lib_prime_list | 195,356 | 175 |
| get_sub | 1,083,622 | 378,414 |
| levenshtein | 577,126 | 344,955 |
| nbody | 32,000,029 | 18,000,182 |
| nbody_fold | 30,000,029 | 16,000,183 |

A case whose elements are wider than a vector access is aligned only at its first element, which is
why `nbody` and `levenshtein` halve rather than clear: their elements are 24 bytes, and 24 does not
divide 32. `fannkuch` and `arrayrw_shared` do not move at all, the first because its arrays stay
under the size from which elements are aligned. Two cases gain splits — `cp_lib_lsegtree` 25,034 to
400,179 and `cp_lib_scc` 45,542 to 70,919 — from the up-to-31 bytes a large array now asks for
moving every allocation after it; neither moves in wall clock.

In wall clock, measured on an idle machine with twenty runs of each case: `fill_from_map` 2.8 times
faster and `fill` 2.3, `arrayrw`, `arrayrw_fn` and `struct_field_mod` 1.6, `cp_lib_bipartite` 1.2,
`nbody_fold` and `index_syntax` 1.04, and 32 of the 46 cases within three percent either way and
steady. The suite comes to 0.926 on the geometric mean of the ratios, or 0.934 taking each case's
fastest run. `cp_lib_scc` is 4 percent slower, the one steady regression: its arrays are 8 and 24
bytes, only 8 of its 226,000 allocations clear the threshold, so it pays the three instructions and
the byte store an array allocation now costs and wins nothing back. `prime_table` reads 5 percent
slower and `bounds_check_indexable`, `sort` and `cp_lib_segtree` within one percent, all four with a
spread wide enough that the figure moves between runs.

**`fill` and `fill_from_map` are bimodal, and that is the shape of the problem this change is
about.** Each allocates a thousand-element array ten thousand times, so one recycled block decides
the whole run, and where that block lands decides whether its accesses straddle. Five independent
measurements of `fill` give the unaligned build a mean of 3.7 to 4.3 ms with a standard deviation of
2.3, against 1.1 to 2.3 ms for the aligned one: the same program, the same input, and a factor of
four between runs of the binary that leaves its elements where the allocator put them. A single
timing of a case like this says more about the addresses it drew than about the code, which is why
the split counters are the column to read.

**This is the first row measured in the fixed environment, so the instruction counts fall against
the row above by a constant that belongs to the instrument.** The micro-benchmarks all move by
-43,546 give or take twenty; add that back to read what the change did. The large cases carry the
constant too, where it is lost in the total: `nbody_fold` -9.1%, `fannkuch` -1.6% and
`cp_lib_conv_zp` -1.0% fall, `cp_lib_scc` +2.0%, `index_syntax` +1.7%, `get_sub` +1.5% and
`cp_lib_dijkstra` +1.2% rise, from inlining decisions moving in both directions around the
allocation. Measured against the fork point in one environment, the whole suite comes to +0.33%.

`push_back` is the largest riser and the least interesting: two register-to-register moves left in
its inner loop by register allocation, on a program that retires ten instructions per iteration.

The row also carries the corrected element size (an array of a boxed element type reserved the size
of the element's own object where it stores a reference), which shows up nowhere here: no case in
the suite holds an array of a boxed type.

## fd0a7ee93588a9bd19e7ec67dcbd9b7ed26586c6

Opens three kinds of column: the split accesses read from the hardware counters, the processor the
row was measured on, and — for the seven cases that now carry `ref.c` and `ref.rs` — the same
program in C and in Rust, measured the same way.

**No case moves.** Every `-inst` figure is identical to the row above it, which is what the interval
should give: the only change to `src/` between the two rows is the narrow-integer extension at the
FFI boundary (PR #114), and no case here exports a function.

The comparison the reference columns open, in instructions:

| case | Fix | C | Rust | Fix/C | Fix/Rust |
|---|--:|--:|--:|--:|--:|
| modulo_loop | 112,658,350 | 140,161,307 | 112,835,735 | 0.80x | 1.00x |
| arrayrw | 120,570,247 | 150,175,966 | 119,944,182 | 0.80x | 1.01x |
| mandelbrot | 236,876,291 | 249,642,758 | 237,050,646 | 0.95x | 1.00x |
| binary_trees | 784,558,542 | 705,427,716 | 739,079,768 | 1.11x | 1.06x |
| nbody | 1,112,167,494 | 706,162,512 | 602,334,325 | 1.57x | 1.85x |
| levenshtein | 1,007,853,029 | 572,081,751 | 902,130,778 | 1.76x | 1.12x |
| fannkuch | 2,731,406,969 | 1,256,317,448 | 954,912,486 | 2.17x | 2.86x |

The counterparts are built for this host with avx512 left out, as the Fix case is, so the three are
allowed the same instruction set. Fix meets or beats Rust on four of the seven and beats C outright
on three. The two that stand out are `fannkuch` at 2.86x Rust — one array clone per permutation,
which is fixlang#123 — and `nbody` at 1.85x.

`splits` opens across every case. `arrayrw` reads 49,600,017 against 16 for its C counterpart, which
is fixlang#120: the element buffer starts 8 bytes into a 16-byte-aligned allocation, so half of
every 32-byte access crosses a cache line. The instruction count cannot express that, which is why
the case looks like the best in the suite there and is the slowest in wall-clock time.

`modulo_loop` is a new case. A running sum has a closed form the optimizer reaches; the carried
modulo denies it that, and vectorization with it, so what is left is the cost of an iteration.

## 4161bc12449319e678c03ab42eacd25a2142f53c

Adds the `fib` and `levenshtein` cases, so their columns open here at 200,990,240 and 1,007,853,029
instructions. Both tasks are carried by the public cross-language benchmark suites: `fib` is naive
recursion, where the whole cost is the call sequence, and `levenshtein` runs a two-row
dynamic-programming table over every pair of a thousand generated words.

**The other cases move because of what the two rows straddle, not because of anything added here.**
The row above was measured on the `fix-many-args-compile-blowup` branch, which forked before the
wide-return tail call reached main, so `mandelbrot` and `mandelbrot_fold` fall 53.94%, `index_syntax`
3.33% and `cp_lib_conv_zp` 1.99% — the same three cases and the same percentages `12165c4494bf`
records for that work.

Of what is left, every micro-benchmark moves by a constant 1,253 to 1,281 instructions. That is
start-up, and the constant is the difference between the environments the two runs were measured
from — about two variables' worth, at the 600 instructions each cost before `cachegrind.py` fixed
the environment. Four cases move by more: `fannkuch` +1.51%,
`cp_lib_bipartite` +1.10%, `cp_lib_lsegtree` +0.57% and `binary_trees` +0.40%, from the rest of the
work merged between the two rows.

## 6591c2396f24380a346a09577850db263b506225

The `fix-many-args-compile-blowup` branch (PR #106), which stops application inlining from binding a
variable argument to a fresh name each time it pushes an application into a `let`, an `if`, a `match`
or an `eval`. The rewrite is what uncurrying's eta expansion runs per parameter, so the binding per
level made the intermediate expression grow as `2^arity`: compiling a 15-parameter function took 314
seconds, and a 13-parameter one aborted the compiler on the stacks v1.4.0 shipped with.

**The emitted programs are unchanged.** Measured back-to-back at one path against the branch's fork
point `9ed0e65a` — the row before this one in `log.csv`, taken minutes earlier in the same
environment — the
executed-instruction total moves from 16,384,259,453 to 16,384,259,427, or -0.0000%. No case moves by
more than 0.05%; the largest single movement is +0.037% on `sum_by_fix`'s memory accesses, a
300-thousand-access micro-benchmark. Dropping the intermediate binding makes the argument variable
occur once per branch, which could have cost `let_elimination` its "used exactly once" condition and
with it an inlining opportunity. It does not — every path that runs this pass runs let-elimination
afterwards, and the binding the pass used to add is a `let` whose bound expression is a variable,
which is exactly what let-elimination removes, so the two shapes converge before code generation.

What the change buys is compile time: at `-O basic` a 15-parameter function goes from 314 seconds to
2.8, and 25 and 40 parameters, previously out of reach, compile in 2.8 and 3.4 seconds.

## b8d298a0550fc15b9369694b53f9483a57f079d2

The same branch with the x86-64 return-register budget corrected: `tailcc`, the convention Fix
lambdas use there, returns five floating-point values in registers where the C convention returns
four, so a result of exactly five floating-point leaves now comes back in registers instead of
through the out-pointer.

**Nothing moves**: the 41 cases together go from 15,866,111,410 to 15,866,111,433 instructions, 34 of
them byte-identical and the rest within ten instructions of program startup. No case in the suite
returns exactly five floating-point leaves, so the corrected entry changes no code here. It changes
what a reader has to re-measure when the convention or the LLVM version changes.

## 12165c4494bf4cc806f72ec6475cc146b2b36532

The `wide-return-tail-call` branch (PR #109), which keeps a Fix tail call compiled as a jump in the
two cases where the backend used to give up on one: a return value wider than the return registers
(the value now travels through an out-pointer parameter) and a tail call carrying more arguments than
the argument registers (every Fix lambda on x86-64 is `tailcc`). Measured against the previous row
`476f40aa`.

**mandelbrot and mandelbrot_fold drop 53.94%**, index_syntax 3.33% and cp_lib_conv_zp 1.99%. The 41
cases together go from 16,386,181,734 to 15,866,111,410 instructions, **-3.17%**. Every win comes from
the out-pointer half: passing the pointer as an ordinary parameter from the start of the IR lets SROA
delete the buffer wherever the callee inlines, where LLVM's own return demotion used to introduce the
pointer at instruction selection, after every IR pass had already run.

The out-pointer half regresses one case on its own, sort by 0.06%, and that is instruction-selection
churn rather than a cost of the buffer: the strength-reduced remainder in the input-generation loop
comes out two instructions longer per iteration, against which the case's insertion-sort phase gets
cheaper.

The rest of the regressions come from `tailcc`: fannkuch +1.51%, cp_lib_bipartite +1.10%, sort +0.73%,
cp_lib_unionfind +0.68%, cp_lib_lsegtree +0.56%, binary_trees +0.40%, cp_lib_scc +0.39%, get_sub
+0.09%. The cases that compile to one inlined loop are unchanged. Three properties of a
guarantees-tail-calls convention account for all of it on x86-64:

- The callee pops the argument area, and `GetAlignedArgumentStackSize` rounds that area up so that it
  plus the return address is 16-byte aligned. A function with no stack arguments at all therefore
  ends in `ret $8`, and every **non-tail** call site pays one `sub $0x8, %rsp` to restore its own
  frame. get_sub calls `slice_bench` 100,000 times and grows by 100,000 instructions.
- Incoming stack arguments become mutable frame objects, since a tail call may overwrite them
  (`X86TargetLowering::LowerMemArgument` marks them so whenever the convention guarantees tail calls).
  A callee can no longer reload one from the caller's slot on demand, so it copies them into its own
  frame in the prologue. fannkuch's `Std::loop#2` grows its frame from 0x28 to 0x48 bytes and its
  prologue by about ten instructions per call; over its 3,628,800 calls that is 38.9M of its
  45.1M-instruction increase, the remainder being the `sub` at its two inner call sites.
- A tail call under such a convention never takes the sibcall path (`IsSibcall` is set only when the
  callee's convention does not guarantee tail calls), so it rewrites the outgoing argument area even
  when the values are unchanged, where a sibcall recognizes matching stack offsets and leaves them
  alone. This is also what buys the fix: a sibcall may not grow the argument area, and a guaranteed
  tail call may.

Restricting `tailcc` to the functions whose arguments exceed the argument registers would spare
everything else the callee-popped argument area and the prologue copies of incoming stack arguments,
but a tail call between two different conventions becomes an ordinary call in **both** directions
(measured with `llc -O2`; `IsEligibleForTailCallOptimization` requires the conventions to match once
either side guarantees tail calls). A narrow function tail-calling a wide one, which is what a
monadic chain is made of, would stop being a jump. The convention has to be uniform across
everything reachable by a tail call.

## 476f40aa1ef55bf5f0880495bd2000860ad13e13

The `defunctionalize-fix-tco` branch (PR #95), which rewrites `Std::fix` into a directly
self-recursive global so LLVM's tail-call elimination can fold it into a loop. Measured against the
previous row `eec295f8` on the same speedtest path.

The one benchmark that uses the `fix` combinator, **sum_by_fix, drops from 655.2M to 0.21M
instructions — -99.97%, a 3175x reduction**. Defunctionalization turns the indirect `fix` self-call
into a direct one; LLVM loop-ifies it and SCEV then closes the accumulation into a constant-time
form. This is the win the branch exists for.

The branch changes nothing else. Its passes only touch `fix`-using symbols, and the standard library
uses no `fix`, so every other program's code is untouched. Confirmed directly: the emitted LLVM IR of
a representative non-`fix` case (sum_by_loop), both before and after LLVM's own optimization passes,
is byte-identical between this branch and its fork point `6dd8c629`, and the two produce the same
executed-instruction count when built at the same path.

The remaining movement the graph shows on the small cases — roughly +44,000 instructions, up to +18%
on the ~250K-instruction micro-benchmarks but +0.00% on every case above a few million — is not a
code change. Built head-to-head today at a fixed path, the previous row's compiler (`eec295f8`) and
this branch's compiler produce the same instruction count within noise (within +/-60 on the ~250K
micro-benchmarks; identical on sum_by_loop), so neither the intervening `main` commits (#88-#91) nor
this branch regressed anything. The two rows were measured ~18 hours apart (`eec295f8` on 2026-07-25
16:01, this row 2026-07-26 10:05) across overnight system-package activity — a kernel and `libc-bin`
update landed at 16:51, minutes after the `eec295f8` run. The +44K is a shift in the emitted
program's fixed per-program startup, an environment effect on the harness's real-project build, not
the compiler. Read a pure code delta by measuring two commits back-to-back in one environment, not
against a historical row.

## eec295f846d6110826a74e823fde8a6ae02859d4

The object-scalarization branch merged with `main`, measured against the previous row `96f68049` (the
cp-library 0.13.0 bump). The branch makes the codegen `Object` hold leaf scalars and materialize an
aggregate only at memory and foreign-ABI boundaries: the body, the return ABI, and the per-type RC
helpers (retain / release / mark / traverser) all pass leaf scalars. The array-loop win it targets was
already banked by the shipped scalar-argument ABI and `build_scalar_phi`, so what remains is code
unification, and the measurement bears that out: most cases are byte-identical (binary_trees, arrayrw,
nbody, mandelbrot, struct_field_mod all to 0.00%), with sub-1% movement each way on the rest (sort
-3.7%, cp_lib_lsegtree -0.8%, fannkuch -0.8%; get_sub +0.5%).

This baseline predates two `main` commits the merge also brings in — the per-signature FFI typing
(#85) and the zero-sized-phi-to-undef change (#86) — so the delta folds those in as well. The only
movement above 1% is confined to the two heaviest cp_lib cases, cp_lib_conv_zp +2.1% and
cp_lib_prime_list +1.9%; with the scalarization confirmed byte-neutral on every non-cp_lib case, that
residue tracks the folded-in #85/#86 codegen changes rather than the scalarization.

## 96f680496768b92145e8d577c26356091e0104d9

Moving the eight `cp_lib_*` cases from cp-library 0.7.4 to 0.13.0, measured against the previous
baseline row `d29b6c3c` on the same compiler. The six cases that generate input with `Random` gained a
direct random 1.1.2 dependency (0.13.0 dropped random from its build deps), the same version 0.7.4
supplied transitively, so the workloads are unchanged. The 32 non-cp_lib cases confirm this: every one
is identical to the baseline to within 0.0000%.

Seven of the eight cp_lib cases are likewise unchanged — their algorithms compile identically across
the two cp-library versions. The exception is **unionfind, which regresses +29.8% in instructions
(111.2M -> 144.4M) and +31.2% in memory accesses (179.1M -> 235.0M)**: cp-library 0.13.0's UnionFind
is meaningfully heavier than 0.7.4's. Compiler and input are held fixed, so this is a cp-library-side
change to weigh, not a compiler regression.

## d29b6c3ccfdd8c92f3999aaec0c7c78778b238c2

Baseline of `main` after the bce merge (#80) and the external-test change (#83), taken before bumping
the `cp_lib_*` cases from cp-library 0.7.4 to 0.13.0. It matches the previous bce row `f0a60009` to
within noise — the intervening commits (the `Arc<RcExpr>` change, the `grow_stack` helper, and the
test-only #83) do not touch code generation — so the merged main reproduces the last bce measurement
and gives a clean reference for the cp-library version change measured next.

## f0a600092158e34fccbe3ac6c44d64b6db8782d5

Removing the traverser `alwaysinline` attribute, measured against the row that added it as an enum
attribute (`0adf6eba`). The two are identical to within noise (total +0.000%, per-case median
diff 0), so the attribute did nothing in any form — an enum `alwaysinline` on the traversers is as
inert as the string one was. The small fixed offset seen against the older `9e6c6f64` row is present
with the attribute and without it alike, so it belongs to other commits or run-to-run startup, not
to this attribute.

## 0adf6ebaa6a8eb33360e6d7044ebcd54389e198d

Attaching `alwaysinline` to the object traversers as a real enum attribute (it had been a string
attribute, which LLVM ignores), measured against the previous row `9e6c6f64`. The effect is nil:
every case is within +0.5%, the whole suite totals +0.00%, and the tiny non-zero deltas are a fixed
per-program increment (a little more inlined traverser code on the startup path), not a per-workload
change. Forcing the traversers to inline buys nothing — most RC traversal goes through a function
pointer stored in the control block, where the hint cannot apply, and the direct calls LLVM already
inlines on its own.

## 9e6c6f64eb4fdb73c48e46a2d766ee332d5eaec4

Marking the runtime panic functions (`fixruntime_abort`, `fixruntime_index_out_of_range`,
`fixruntime_negative_array_size`) as `noreturn`, measured against the previous row `a77ad9dd`. These
functions never return, but their LLVM declarations lacked the attribute, so a bounds-check failure
path called one and then flowed to a merge, feeding an `undef` into it. That forced an aggregate phi
for the loop-carried value, which hid the array size and left the per-element bounds check standing.
This had regressed when the direct libc `abort` (which LLVM recognizes as `noreturn`) was replaced by
these custom functions to print richer messages and backtraces. The attribute lets LLVM prune the
failure path, the aggregate collapses to scalars, and the check folds.

Sixteen cases improve and none regress: get_sub -80.5%, cp_lib_segtree -20.4%, nbody_fold -16.5%,
cp_lib_lsegtree -15.5%, fannkuch -14.3%, bounds_check_indexable -9.8%, gen_random_array -9.5%,
nbody -9.2%, cp_lib_prime_list -8.6%, random_state -8.3%, sort -7.7%, cp_lib_unionfind -4.1%,
cp_lib_scc -4.0%, cp_lib_dijkstra -3.6%, cp_lib_bipartite -3.2%, index_syntax -1.6%. Against the flip
row `4537cc17`, every read/fold regression is now erased and the write wins are kept; the sole case
still above that baseline is cp_lib_bipartite (+3.4%), which carries a genuine multi-exit
control-flow aggregate that this change does not reach.

## a77ad9dd29282fb48a29763115d27aedefd59a4b

Scalarizing loop-carried unbox structs, measured against the flip row `4537cc17`. A loop-carried
`Array` (or an iterator holding one) was threaded through a `fold` / `loop` as one LLVM aggregate,
so `@size` hid inside an aggregate phi, the per-element bounds check survived, and the read loop
did not vectorize. Passing unbox-struct function arguments as flat leaf scalars, and building
codegen's value-merge phis one scalar phi per leaf, exposes `@size` again and the loops vectorize —
with tail-call optimization intact (unlike the `reg2mem` alternative).

Read / fold regressions are erased, most now below the pre-unboxing baseline: sum_by_loop_iter_cap
-77.6%, sum_by_fold / sum_by_fold_cap / sum_by_range_fold -75.4%, array_mod -59.2%,
fill_from_map -57.5%, sum_by_loop_arr -43.9%, sum_by_loop_iter -39.2%. Write wins are kept
(arrayrw -94.0%, arrayrw_shared -95.3%) and other loops improve as their state goes scalar
(option_plumbing -60.4%, nbody -31.3%, random_state -29.9%, nbody_fold -26.7%, push_back -8.0%).
Three cases regress, all carrying a large aggregate re-formed past the change's reach:
bounds_check_indexable +10.9% (the value is also returned, and returns stay aggregate),
cp_lib_bipartite +6.9%, cp_lib_dijkstra +1.4%.

## 4537cc177baee6a72256f5c96a14f643795c9afc

The Array value-layout flip to unboxed `{ storage, size, capacity }`, measured against the
step-1-end row `69d9257b`. Write-heavy cases improve as intended, because `@size` / `@capacity`
become register reads and the bounds / capacity checks fold: struct_field_mod -95.0%,
prime_table -45.0%, write_by_range_fold -38.5%, array_mod -25.4%, arrayrw -16.7%,
push_back -13.6%, cp_lib_prime_list -13.4%.

Read / fold cases regress, the risk the design's §10 anticipated: the fatter 3-word `Array`
value swells the iterator loop state (`Option (ArrayIterator a, a)`), which then spills to
memory instead of staying scalar. sum_by_loop_iter_cap +165%, sum_by_fold / sum_by_fold_cap /
sum_by_range_fold +141%, fill_from_map +136%, sum_by_loop_iter +40%. cp_lib_unionfind +30%
(this row also carries the cp-library 0.7.3 -> 0.7.4 migration diff on the eight cp_lib cases,
and the subprocess migration on the driver).

## 7afe8e174d0a785106d7c0e4961bce88e2d3beb0

Reverted the temporary no-runtime-check enablement.

## 0bec40c5d5765799987c474f93c6f2bb50369cf9

Temporarily enabled no-runtime-check. (Will be reverted in the next commit)

## ba06b2f2ced3ce16719038b71bdf790dccfdeb2c

Performance degradation due to adding checks for non-negative capacity and size in Array::fill and Array::empty.

## 7bd496c3cd6245f5604df0f2fb1ca96b657fe05e

Due to changes in the implementation of the check_range function.
In the previous commit e4e3a33dd436b06bd8126d4e273ab17957c483e2, check_range was already introduced, but it only displayed an error message and aborted.
Between that commit and 7bd496c3cd6245f5604df0f2fb1ca96b657fe05e, fixruntime_index_out_of_range was defined in runtime.c and changed to be called from check_range.
This caused performance degradation.
Note that we forgot to run the benchmark immediately after changing the check_range function implementation, so the impact appeared in the benchmark of a slightly later commit.