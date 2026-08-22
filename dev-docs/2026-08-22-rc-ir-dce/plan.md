# RC IR の到達不可能な関数とグローバルを落とす (#163)

## 何が問題か

コード生成は `RcProgram::funcs` の**全部**を LLVM module に書き出す (`Generator::implement_rc_program` が
2 度舐める — 1 度目で宣言、2 度目で本体)。到達できない関数もそこに含まれる。

`-O max` は単位が 1 つなので、書き出された関数はエントリ以外すべて `internal` になり、LLVM の
`globaldce` がいずれ落とす。落とすまでに、Fix 側が LLVM IR を構築する代金と、LLVM がそれを読んで
検証する代金を払っている。

**実測** — speedtest のケースを `-O max` でビルドし、Fix が渡した最適化前の module に `globaldce`
だけをかけた:

| ケース | 関数 | 残り | 死 | 命令 | 残り | 削減 |
| --- | --- | --- | --- | --- | --- | --- |
| cp_lib_bipartite | 391 | 181 | 54% | 57,634 | 14,062 | **76%** |
| cp_lib_scc | 259 | 114 | 56% | 27,852 | 6,368 | 77% |
| cp_lib_lsegtree | 198 | 106 | 46% | 25,400 | 7,473 | 71% |
| cp_lib_segtree | 142 | 82 | 42% | 15,156 | 4,014 | 74% |
| levenshtein | 81 | 30 | 63% | 4,616 | 1,166 | 75% |
| sort_stable | 56 | 22 | 61% | 3,520 | 1,266 | 64% |
| fannkuch | 54 | 26 | 52% | 5,122 | 2,351 | 54% |
| nbody | 54 | 39 | 28% | 2,047 | 1,520 | 26% |
| binary_trees | 20 | 12 | 40% | 499 | 381 | 24% |
| mandelbrot | 14 | 10 | 29% | 294 | 276 | 6% |
| fib | 44 | 42 | 5% | 1,174 | 1,165 | 1% |

cold `-O max` ビルドの内訳は、前半 (parse + 型検査 + Fix 最適化 + RC IR + LLVM IR 構築) が 39-41%、
LLVM の最適化が 35-41%、コード生成が 17-26% である。この掃除が減らすのは、前半のうち LLVM IR を
構築する分と、LLVM 最適化の入口の分である。

## 死骸を作っているのは誰か

AST 側の掃除は済んでいる。`dead_symbol_elimination` が到達しないシンボルを落とし、`let_elimination`
の条件 3 (「`x` が本体に現れない」) が未使用の let を落とす (`inline_local::run` が全シンボルを不動点まで
回す。どちらも `-O max` 以上)。

残るのは **RC IR 自身が増やした版**である。名前にそれが出ている:

- `lower_program` が持ち上げたラムダ (`...#closure_lam2#funptr1`)
- `borrow_ify` の借用版、`unique_check_elim::specialize` と `locality::specialize` の特殊化版
  (`...#closure_spec_<hash>#funptr3`)

呼び出し側が新しい版へ移ると、元の版と、その版だけが呼んでいたラムダが、まとめて宙に浮く。

## 設計

### 1. `RcProgram::entry` を本当の根にする

いまのフィールドはプレースホルダで、`lower_program` が `#entry` という実在しない名前を入れている。
そこに置かれたコメントが、この作業を予告している:

> `entry` labels the program in the dump only; ... It is a placeholder, NOT a reachability root:
> RC-IR dead-function elimination, when added, must take its roots from the real entry points
> (there can be several — every FFI-exported function is one), not from this field.

根は 1 つではなく、関数とは限らないので、フィールドは `entry: FuncRef` から `roots: Set<FullName>` へ
変える。doc には「コード生成がこのプログラムの外から到達させる名前」と、それが何かを書く。

現に `entry` を読んでいるのは 6 か所 — ダンプの見出し (`print.rs`)、それにプログラムを組み直す
`split_rc_units` / `borrow_ify` / `cancel` / `locality::specialize` が持ち回っている分である。

### 2. 根の集合

コード生成が外から到達させるものが根である。3 種類:

- **エントリ** — `Program::entry_io_value` が指す値。`build_main_function` が C の `main` から呼ぶ。
- **`FFI_EXPORT` した値** — `Program::export_statements` の各文。`build_exported_c_functions` が
  C のシンボルとして公開する。複数ある。

グローバル初期化子は根ではない。`implement_rc_program` が `prog.globals` を無条件に出すのは、いま掃除が
無いからで、**到達しないグローバルも関数と同じく落とす**。Fix の式はすべて純粋で、グローバルは呼ばれて
初めて走る call-once の初期化子なので、誰も参照しないものを落としても振る舞いは変わらない。`Eval` で
グローバルを強制する経路は参照そのものなので、そのグローバルは到達可能側に入る。

根はどちらも `lower_program` からは見えず、`build_object_files` が持っている。**根はビルドドライバから渡す。**

根は関数だけを指すとは限らない — エントリも `FFI_EXPORT` した値も、funptr 型でなければグローバルに
なる (`lower_symbol`: 「a funptr symbol becomes a top-level function under the symbol's own name, and a
symbol of any other type becomes the initializer of a global value」)。なので根は**名前の集合**として持ち、
関数とグローバルの両方を指せるようにする。

### 3. 分割コンパイルのとき

`-O basic` 以下は単位ごとに分けて `.o` にし、単位をまたぐ関数は外部リンケージになる
(`Configuration::external_if_separated`、`Generator::declare_lambda_function`)。他の単位から呼ばれ得る
関数を落としてはならないので、**根は「外から見える関数」と一致させる** — つまりリンケージを決める規則と
同じ規則で根を決める。この一致を doc に書く。

### 4. 辺

関数 `f` の本体が `g` を名指す経路は 2 つ:

- `RcRhs::Closure(fref, _)` — 持ち上げたラムダや funptr 関数への参照。
- **ローカルでない変数名** — `lower_program` は「A global: an atom naming the global」と書いており、
  グローバルや funptr シンボルへの参照は、その名前を持つ atom になる。ローカル名は必ず一意に鋳造される
  ので、`funcs` の鍵と一致する名前はその関数への参照である。`RcRhs::Llvm` の演算子が内部に持つ名前
  (`LLVMGen::free_vars`) も同じ扱い。

### 5. 置き場所

`optimize_rc_program` の最後、`locality::specialize` の後。特殊化が終わってからでないと、宙に浮いた版が
まだ浮いていない。

## やらないこと

**`Destructure` で消えたフィールド**のような、関数とグローバルの粒度より細かい死骸には手を付けない。
未使用の let は AST 側の `let_elimination` が既に落としている (`-O max` 以上)。

## 測り方

1. **正しさ** — フルスイート。加えて、掃除の前後で**生成される LLVM IR が、掃除した関数の分を除いて
   一致する**ことを 1 ケースで確かめる (`--emit-llvm` の差分)。
2. **効き** — speedtest 51 本の cold ビルド命令数と、コーパスの cold `-O max`。上の表から、
   cp-library を使うケースで LLVM が読む IR が 7 割減るので、cold の 1-2 割が動くと見込む。
3. **中立性** — speedtest 51 本の**実行**命令数。落とすのは到達しないものなので、動いてはならない。
   動いたら根の取り方が誤っている。
4. **`-O basic` が壊れていないこと** — 単位をまたぐ呼び出しが残ること。コーパスの `-O basic` ビルドと
   実行で確かめる。
