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

呼び出し先を差し替えるパスの直後ごとである。差し替えるのは `cancel` (借用版へ振り替える)、
`unique_check_elim::specialize`、`locality::specialize` の 3 つで、ある版への最後の呼び出しが
差し替わった時点で、その版と、その版だけが呼んでいた関数が、まとめて呼び手を失う。パスとパスの
あいだで刈ると、下のパスが「誰も実行しない関数」を複製し、解析し、コードにする手間が消える。

差し替えるパスが 1 つも走らない水準では、下ろしたプログラムを 1 回通せば済む。

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

## 付録: 実測

### 掃除の効き

speedtest のケースを `-O max` でビルドし、コンパイラが渡す最適化前の LLVM module を数えた。
`gdce` 欄は、その module にさらに `opt -passes=globaldce` をかけたときに残る関数の数である。

| ケース | 関数 (前) | 関数 (後) | gdce | 命令 (前) | 命令 (後) | 差 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| cp_lib_bipartite | 391 | 185 | 181 | 58,211 | 14,237 | -75.5% |
| cp_lib_scc | 259 | 118 | 114 | 28,360 | 6,513 | -77.0% |
| cp_lib_lsegtree | 198 | 110 | 106 | 25,791 | 7,607 | -70.5% |
| cp_lib_segtree | 142 | 86 | 82 | 15,499 | 4,127 | -73.4% |
| levenshtein | 81 | 34 | 30 | 4,711 | 1,209 | -74.3% |
| sort_stable | 56 | 26 | 22 | 3,600 | 1,320 | -63.3% |
| fannkuch | 54 | 30 | 26 | 5,177 | 2,398 | -53.7% |
| nbody | 54 | 43 | 39 | 2,080 | 1,561 | -25.0% |
| binary_trees | 20 | 16 | 12 | 509 | 407 | -20.0% |
| mandelbrot | 14 | 14 | 10 | 303 | 303 | 0.0% |
| fib | 44 | 44 | 42 | 1,193 | 1,193 | 0.0% |

どのケースでも、掃除の後に `globaldce` がさらに落とすのはちょうど 4 つ — `fixruntime_get_argc`、
`fixruntime_get_argv`、`fixruntime_ptr_add_offset`、`fixruntime_subtract_ptr` — で、いずれも RC IR が
持たないランタイムの C ヘルパである。**Fix 側の到達不可能な関数は残らない。**

生き残った関数の本体は、掃除の前後で一致する。cp_lib_bipartite で 391 個中 206 個が消え、
新たに現れた関数は 0、本体の異なる関数は 4 つで、その差は `@global_string.5` が
`@global_string.7` になったという**無名文字列定数の自動採番のずれだけ**である
(消えた関数がリテラルを登録しなくなったため番号が詰まる)。指す文字列の内容は同じである。

### 置き場所を決めた計測 (`fix` 自身の cold `-O max`)

刈る場所を増やしながら測った。どの版も**出す IR は 1 行も変わらない**。

| 版 | 命令 | 掃除なし比 |
| --- | ---: | ---: |
| 掃除なし | 312.607 G | — |
| 最後に 1 回だけ | 306.416 G | -1.98% |
| `cancel` の後にも | 302.785 G | -3.14% |
| 差し替える 3 パスすべての直後 | **297.033 G** | **-4.98%** |

同じプログラムを出しているかは、出力の行の多重集合で確かめた。複製パスが振る連番 (`#u1230` など) と
無名文字列定数の番号を消し、行を数え上げて比べると、speedtest の 6 ケースすべてで**完全に一致する**。
関数の数も命令の行数も 1 つ違わない。ずれるのは定義の並び順 (関数表の反復順) と連番 (作る複製が減った分)
だけである。

### コンパイル時間 (cold `-O max`、`perf stat -e instructions:u`)

- speedtest 51 本 合計 277.820 G -> 273.051 G (**-1.72%**)。最大は cp_lib_bipartite の -5.77%
- 実プロジェクトのコーパス 17 本 合計 452.416 G -> 436.507 G (**-3.52%**)、退行 0 本。
  最大は `fix` 自身の 312.691 G -> 297.548 G (**-4.84%**)

見込んだ 1-2 割よりは小さい。LLVM の `-O3` パイプラインは早い段階で自前の大域デッドコード除去を
通すので、死んだ関数は高い最適化に届く前に消えていた。減ったのは **Fix が LLVM IR を組み立てる分と、
LLVM がそれを読んで検証する分**、そして**下の RC IR のパスが死んだ関数に費やしていた分**である。

### 実行時の中立性

speedtest 51 本の `-O max` バイナリを `perf stat -e instructions:u` で測り、**全 51 本が ±0.000%**、
0.05% を超えて動いたケースはゼロ。

### 低い水準

フルスイート (1,572 本) が `FIX_MAX_OPT_LEVEL` の `max` / `basic` / `none` すべてで通る。加えて、
コーパスの 17 本が `-O basic` で 17/17 リンクする — 単位をまたいで呼ばれるシンボルを落とせば、
未定義シンボルでリンクが失敗する。

### `FFI_EXPORT` した値がルートであること

Fix からは誰も参照しない値だけを持つ dylib をビルドし、`nm -D` に `c_triple` が出ること、
C から呼んで正しい値が返ることを確かめた。単位が 1 つのビルドではこの値の到達経路はルート集合
だけなので、エクスポートをルートから落とすとリンクが通らない。
