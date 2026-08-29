# T -- パイプラインが保存するもの

対象コミット: `deb3b0eac2b0f53d05f39c3560c685758e9a6e81`

このファイルは主定理 T を証明する。T は合成であり、P14・P23・P24・P26・P27 の**言明**と、
`optimize_rc_program` が 2 つのパスを呼ぶ順序だけを使う。引用する命題の証明は読まない。

第 3 節が、T が立つ 20 個の仮定を果たす者とともに 1 か所に並べる。第 4 節が、T が届かないところを
述べる。**この定理が読み手に何を与え、何を与えないかを決めるのは第 4 節である。**

## 1. T の言明

以下、`p0` を `borrow_ify` の入力とし、

    p1 == borrow_ify(p0, type_env, develop_mode)
    p2 == cancel(p1, type_env)

と書く。この 3 つの名前はこのファイルの中でだけ使う。

**T** (パイプラインが保存するもの)

    ASSUME NEW p0 : RcProgram, NEW type_env : TypeEnv, NEW develop_mode : bool,
      H0. `p0` は、`optimize_rc_program` が `split_rc_units` を呼んだ後の `prog` の値である。
      H1. `p0` について A1 が成り立つ。すなわち `p0` は D12 の意味で RC 規律を満たし、
          `p0` のすべての関数の `borrowed_units` は空である。
      H2. `p0` について A2 が成り立つ。すなわち `p0` のすべての `Retain`/`Release` 節点の path は、
          その変数の型の `rc_units` の要素である。
      H3. A3 から A20 までの 18 個の仮定が成り立つ (第 3 節が 20 個すべてを並べる)。
    PROVE
      (T1) `p2` は D12 の意味で RC 規律を満たす。
      (T2) `p2` のどの実行 (D24) においても、解放されたオブジェクトの読み (D7) は起きず、
           どのオブジェクトも高々 1 回しか解放されない。`p2` の正常終了する実行 (D31) で
           最後まで解放されずに残る計数下のオブジェクト (D26) は、環境が持つ参照 (D25) が
           指すオブジェクトから到達できるものに限る。
      (T3) `p2` の各一意性の観測点 (D18) `q2` について、D19 の対応が `q2` に与える `p0` の
           観測点を `q0` とする。`q0` の観測値が真であれば、`q2` の観測値も真である。
           (実行についての量化は P26 の言明のものである。第 4 節がその範囲を述べる。)
      (T4) `p0` から `p2` を作る 2 つのパスの合成は、`roots` を変えず、各関数の `fn_ty` /
           `ret_ty` / `params` の型 / `inline_into_callers` を変えず、各グローバル初期化子の
           `symbol` と `ty` を変えず、`owns_initializer` と `owns_storage` に `true` を書く。
           この書き込みは正しい値を書く。

(T1) と (T4) は D12 の見る部分と見ない部分を分けて述べる。(T2) は (T1) から実行の水準へ渡る。
(T3) は D12 の軸の上に無い性質であり、(T1) からは出ない。

## 2. 証明

### <1>1. `cancel` が受け取るプログラムは `p1` である

`optimize_rc_program` は、`split_rc_units` の後に `p0` を `borrow_ify` に渡し、その返り値を
`cancel` に渡す。この 2 つの呼び出しの間に走るのは `validate` の呼び出しだけであり、`validate` は
プログラムを変えない。

  **<2>1.** `config.enable_borrow_optimization()` が真のとき、`optimize_rc_program` は
  `split_rc_units(&mut prog, type_env)` とその後の `validate` の後、次の 4 行をこの順で実行する。

  ```rust
  prog = borrow_ify(&prog, type_env, config.develop_mode);
  validate(&prog, "after borrow_ify");
  prog = cancel(&prog, type_env);
  validate(&prog, "after cancel");
  ```

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`

  **<2>2.** この 4 行の `validate` は、同じ関数の中で束縛された次の閉包である。

  ```rust
  let validate = |prog: &RcProgram, stage: &str| {
      if config.develop_mode {
          validate::validate(prog, &symbol_names, type_env, stage);
      }
  };
  ```

  この閉包は `prog` を共有参照で受け取り、値を返さない。呼ぶ先の
  `pub fn validate(prog: &RcProgram, symbol_names: &Set<FullName>, type_env: &TypeEnv, stage: &str)`
  も同じである。よってこの 2 つの呼び出しは束縛 `prog` の値を変えない。

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`,
       `CODE src/rc_ir/validate.rs: validate`

  **<2>3. QED**

  <2>1 の 1 行目で `prog` に入る値は、`p0` を第 1 引数とする `borrow_ify` の返り値、すなわち `p1` で
  ある (H0 と第 1 節の `p1` の定義)。<2>2 より 2 行目は `prog` を変えないので、3 行目の
  `cancel(&prog, type_env)` が受け取るのは `p1` であり、その返り値は `p2` である
  (第 1 節の `p2` の定義)。

    BY <2>1, <2>2, H0

### <1>2. `p0` は D12 の意味で RC 規律を満たす

    BY H1, A1

(H1 は A1 の本文をそのまま述べており、その第 1 文がこれである。)

### <1>3. `p1` は D12 の意味で RC 規律を満たす

P14 の言明の前提は 3 つである -- 入力が D12 の意味で RC 規律を満たすこと、入力が A1 を満たすこと、
入力が A2 を満たすこと。この 3 つを `p0` について与えるのが <1>2、H1、H2 である。P14 の結論は
`borrow_ify` の出力が D12 の意味で RC 規律を満たすことであり、その出力は `p1` である
(第 1 節の `p1` の定義)。

    BY <1>2, H1, H2, P14

### <1>4. A19 (ii-b) と P14a は `p1` の各本体について読める

A19 の範囲は「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の
入力) の両方」であり、P14a の範囲は「`borrow_ify` の出力の各本体」である。<1>1 より `p1` は
`borrow_ify` の出力であり、かつ `cancel` が受け取るプログラムである。よって `p1` の各本体は両方の
範囲に入る。

    BY <1>1, A19, P14a

P23 の言明が前提に置くのは入力が D12 を満たすことだけなので、次のステップはこのステップを要しない。
ここに置くのは、A19 (ii-b) を読む P18a・P19・P21 と、P14a を読む命題が、`cancel` の入力について
書かれた仮定・命題であり、その「`cancel` の入力」が `borrow_ify` の出力であることは <1>1 が言うまで
与えられていないからである。第 4 節の「引用した命題自身の状態」がこの結び目に触れる。

### <1>5. (T1) `p2` は D12 の意味で RC 規律を満たす

P23 の言明の前提は、入力が D12 の意味で RC 規律を満たすことである。それを `p1` について与えるのが
<1>3 であり、`p1` が `cancel` の入力であることを与えるのが <1>1 である。P23 の結論は `cancel` の
出力が D12 の意味で RC 規律を満たすことであり、その出力は `p2` である。

    BY <1>1, <1>3, <1>4, P23

### <1>6. `p2` のすべての本体は D11 を満たす

D12 は、プログラム `P` が RC 規律を満たすことを、`P` のすべての関数の本体と、すべてのグローバル
初期化子の `init` が、`P` の `borrowed_units` が定める所有と借用の割り当て (D14) の下で D11 を
満たすことと定める。D23 より、本体とは関数の `body` かグローバル初期化子の `init` であり、この 2 種で
尽きる。

    BY <1>5 DEF D12, D23

### <1>7. (T2)

P27 の言明の前提は、プログラムのすべての本体が D11 を満たすこと、および A17 と A18 が成り立つこと
である。<1>6 が第 1 の前提を `p2` について与え、H3 が A17 と A18 を与える。P27 の結論が (R1)(R2)(R3)
であり、(T2) の 3 つの文はその 3 つである。

    BY <1>6, H3 (A17, A18), P27

### <1>8. D19 の対応は 2 つのパスを跨いで合成する

D19 は、`borrow_ify` と `cancel` のそれぞれについて、出力の各一意性の観測点から入力のちょうど 1 つの
観測点への対応を与える。<1>1 より `p1` は `borrow_ify` の出力かつ `cancel` の入力なので、`p2` の各
観測点は `p1` のちょうど 1 つの観測点へ写り、`p1` の各観測点は `p0` のちょうど 1 つの観測点へ写る。
写像の合成は写像なので、`p2` の各観測点は `p0` のちょうど 1 つの観測点へ写る。(T3) の `q0` はこの
合成の像である。

    BY <1>1, D19

### <1>9. (T3)

P26 は 2 つの半分からなる。`borrow_ify` の半分は「`p0` の観測点の観測値が真ならば、`p1` の対応する
観測点の観測値も真である」を、`cancel` の半分は「`p1` の観測点の観測値が真ならば、`p2` の対応する
観測点の観測値も真である」を与える。<1>8 が 2 つの対応をつなぐので、含意の推移により
「`q0` が真ならば `q2` が真」を得る。

    BY <1>1, <1>8, P26 (`borrow_ify` の半分), P26 (`cancel` の半分)

**この段は P26 の 2 つの半分の言明に立つ。`cancel` の半分は証明されているが、`borrow_ify` の半分は
まだ証明されていない (README 第 7 節の表)。よって鎖が閉じるのはその半分が閉じたときである。**
実行についての量化を P26 がどう置いているかは第 4 節が述べる。

### <1>10. (T4)

  **<2>1.** 2 つのパスがどちらもある欄を変えないならば、その合成もその欄を変えない。

  合成 `cancel(borrow_ify(p0))` の下でその欄の値は、`p0` での値、`p1` での値、`p2` での値と辿り、
  前の 2 つの等式を等号の推移でつなぐ。

    BY 等号の推移 (外部の初等的な事実)

  **<2>2.** P24 は、`borrow_ify` と `cancel` のそれぞれについて、`roots` を変えないこと、各関数の
  `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` を変えないこと、各グローバル初期化子の
  `symbol` と `ty` を変えないことを述べる。

    BY P24

  **<2>3.** 「同じ関数」を定める鍵は `cancel` の前後で変わらない。`cancel` は `prog.funcs` の各 `f`
  について `let mut clone = f.clone();` を作り `clone.body` だけを差し替えて `(f.name.clone(), clone)`
  を収める。よって出力の `funcs` の鍵の集合は入力のそれに等しく、各鍵の関数は `body` 以外のすべての
  欄を入力の同名の関数と共有する。

    BY `CODE src/rc_ir/borrow.rs: cancel`

  **<2>4.** P24 は、`borrow_ify` と `cancel` のそれぞれについて、各グローバル初期化子の
  `owns_initializer` と `owns_storage` に `true` を書くこと、および D1 が述べる呼び出し順により
  この書き込みが正しい値を書くことを述べる。2 つのパスを続けて適用すると、この 2 つの欄には 2 回とも
  `true` が書かれ、最後に書かれた値も `true` である。

    BY P24, D1

  **<2>5. QED**

  (T4) の前半 3 つは <2>1 と <2>2 (鍵については <2>3) から、後半は <2>4 から得る。

    BY <2>1, <2>2, <2>3, <2>4

### <1>11. QED

(T1) は <1>5、(T2) は <1>7、(T3) は <1>9、(T4) は <1>10 である。

    BY <1>5, <1>7, <1>9, <1>10

## 3. T が立つ仮定と、それを果たす者

T は、引用する命題が証明されている仮定の集合の上に立つ。README 第 4 節の仮定は 20 個あり、その全部が
ここに載る。「果たす者」は README 第 4 節の言葉である。右の欄は、README がその仮定を名指して結び
付けているものを書き、README がどこでも名指していない仮定には `--` を置く。

| 仮定 | 内容 | 果たす者 (README 第 4 節) | README がこの仮定に結び付けているもの |
|---|---|---|---|
| A1 | 入力が RC 規律を満たす | 前段のパス (`insert_rc`) | P14 の言明 |
| A2 | 単位への正規化 | `insert_rc` と `split_rc_units` | P14 の言明 |
| **A3** | 宣言されたモデルの忠実さ | **誰も** | D10、D21、D24、D29、P5 (a) |
| **A4** | コード生成の忠実さ | **誰も** | `--` |
| A5 | 型が leaf の上位近似 | `leaf_map.rs` の設計 | D25、P7a |
| A6 | 名前の一意性 | lowering | P9 (出力側は P9 が示す) |
| A7 | 呼び出し先の解決 | `resolve_callee_params` の設計 | `--` |
| A8 | グローバルは線形規律の外 | `mark_global` | D26、(E7) |
| A9 | `Match` はアームを持つ | lowering (検査は develop mode の `validate`) | P16 |
| A10 | 型の well-formedness | `validate_layouts` | `boxed_leaf_paths` と `rc_units` の停止性 |
| A11 | スコープの規律 | lowering (検査は develop mode の `validate`) | `origin` の停止性 |
| **A12** | 束縛の形と型が合っている | **誰も** (検査するコードは無い) | `held_field_type`、`rhs_consumes` の停止性 |
| A13 | 名前の形 | `Lowerer::fresh_var` と `clone_fresh` (検査は develop mode の `check_clone_names_are_fresh`) | P9 の後半 |
| A14 | 過適用が無い | 型検査と lowering | `call_rc`、`rhs_consumes` |
| A15 | `grow_stack` は閉包をちょうど 1 回呼ぶ | `stacker` crate | `origin`、`CancelAnalysis::walk`、`RewriteCtx::rewrite`、`drop_nodes`、`rename_expr` |
| A16 | `Match` のアームは scrutinee のタグを尽くす | lowering (**検査: 無し**) | P5 (a)、P6 |
| A17 | 環境の契約 | 環境のコード (**検査: 無し**) | P27 の言明 |
| **A18** | 生きているオブジェクトのグラフの非巡回性と、グローバル状態のオブジェクトが計数下の参照を持たないこと | **誰も** | P27 の (R3) |
| A19 | bump の下に余りが在る | `insert_rc` と `borrow_ify` (**検査: 無し**。**`insert_rc` の側は開いている**) | (ii-a) は P14、(ii-b) は P18a・P19・P21 |
| A20 | 借りた参照は活性化の間 生きている | 呼び出し元 (**検査: 無し**) | P14 の (S-c)、P14a |

**誰も果たさない仮定は A3・A4・A12・A18 の 4 つである。** この 4 つが T の値段であり、どれか 1 つが
偽であれば T の結論は成り立たない。README 第 4 節が、A3 については人手の照合の記録
(`dev-docs/2026-06-28-unique-check-elim/audit-2026-07-20-op-declarations.md`) を、A18 (a) については
valgrind の下で走るテストを、それぞれ検査として挙げている。A4 と A12 には検査が無い。

**果たす者が居て検査の無い仮定は A16・A17・A20 の 3 つである。** このうち A20 は呼び出し元の側の
契約であり、この文書のどの命題も、出力の呼び出し元がそれを満たすことを示していない。

**A19 は半分だけ果たされている。** README 第 4 節が、`borrow_ify` の側は示されており、`insert_rc` の
側には (O1) と (O2) の 2 つの義務が残ると述べる。A19 は T の前提 H3 に入っているので、**(T1) から (T4)
までのすべては、A19 の `insert_rc` の側が閉じるまで、そこを仮定として持つ。** (ii-a) を読むのは P14、
(ii-b) を読むのは P18a・P19・P21 である (README 第 4 節の A19)。

## 4. T が届かないところ

### 証明していない 3 つ

README 第 1 節が名前で挙げるとおり、**評価順の保存**、**FFI の副作用の列の保存**、**返り値の一致**は
証明しない。返り値の一致は `cancel` についても主張しない -- カウントが下がると
`unique_check_operand` を宣言する op が別の腕を取り、割り当てるオブジェクトの個数が変わる。この向きの
変化は言語が認めている (P26)。

### T が言うのは `optimize_rc_program` の返り値についてではない

`cancel` の後に、同じ関数の中で 3 種のパスが走る
(`CODE src/build/build_object_files.rs: optimize_rc_program`)。

- `dead_code_elim::eliminate_unreachable` -- `cancel` の直後、`unique_check_elim::specialize` の
  直後、`locality::specialize` の直後の 3 か所で、`prog.roots` が到達しない関数とグローバルを落とす
  (`CODE src/rc_ir/dead_code_elim.rs: eliminate_unreachable`)。
- `unique_check_elim::specialize` -- 入力の一意性で関数を複製し、それによって示せるようになった
  unique check を取り除く (`CODE src/rc_ir/unique_check_elim.rs: specialize`)。
- `locality::specialize` -- `config.threaded` が偽のときだけ走り、対象が局所であると示せる参照カウント
  操作に注釈を付け、呼び出し元が到達する入力の局所性ごとに関数を複製する
  (`CODE src/rc_ir/locality.rs: specialize`)。

**T はこの 3 つを覆わない。** T の主語 `p2` は `optimize_rc_program` の途中の値であり、コード生成へ
渡るのはこの 3 つを通した後の値である。よって読み手が T から得るのは「`borrow_ify` と `cancel` は
RC 規律を壊さない」であって、「コード生成に届くプログラムが RC 規律を満たす」ではない。後者を言うには、
この 3 つのパスについて P14・P23 に当たる命題が要る。

`borrow_ify` と `cancel` が走るのは `config.enable_borrow_optimization()` が真のとき、すなわち
`-O max` 以上のときに限る (`CODE src/configuration.rs: Configuration::enable_borrow_optimization`)。
それより下の水準ではこの 2 つは走らないので、T の言うことは無い。

### (T2) が依頼者の言葉で何を言うか

依頼者の言葉は「この 2 つのパスが原因の segv と二重解放は起きない」である。(T2) が届くのは次のとおり
である。

- **届く**: `p2` のどの実行でも、解放されたオブジェクトの読み (D7 の意味の読み) は 1 度も起きず、
  どのオブジェクトも 2 回目の解放を受けない。T の前提 H1 が入力について D12 を要求し、結論 (T1) が
  出力について D12 を与えるので、**この 2 つの誤りをこの 2 つのパスが持ち込むことは無い。**
- **届かない**: segv 一般。D7 の読みとは、オブジェクトの記憶域のうち参照カウントと状態バイトを除いた
  部分を読むことである。null ポインタの参照、配列の範囲外、`FFI_CALL` の先で起きる誤りは D11 の軸の
  上に無く、T は何も言わない。
- **届かない**: 「ちょうど 1 回解放される」。P27 の言明が反例を 3 つ挙げる -- グローバル状態の
  オブジェクトは 1 度も解放されない、発散する実行と中断する実行では終端の `Ret` に着かない活性化が
  残る、正常終了する実行でも C のエントリ点は `run_ios_runner` が作る `IOState` を処分しない。
  (T2) が言えるのは**高々 1 回**までである。
- **漏れについて言えるのはここまで**: (R3) は「漏れるとすれば、環境が持つ参照から到達できるものだけ
  である」という形の主張であり、「漏れない」ではない。

### (T3) が閉じるために残っているもの

2 つある。

1. **P26 の `borrow_ify` の半分がまだ証明されていない** (README 第 7 節の表)。<1>9 はその言明を引用
   しているので、鎖が閉じるのはその半分が閉じたときである。
2. **P26 の言明は、入力の実行と出力の実行の対応を定めていない。** P26 は「入力の一意性の観測点で
   観測値が真であるすべての**実際の**実行について、出力の対応する観測点の観測値も真である」と述べる。
   D29 (対応する活性化) は `cancel` についてこの対応を作るが、`borrow_ify` については置かれていない
   (D29 の「**`borrow_ify` にはこの対応を置かない。**」)。<1>9 の含意の推移は、P26 の 2 つの半分が
   同じ読み方で読まれることの上に立つ。

### 開発ビルドでだけ走る検査

`optimize_rc_program` の `validate`、`borrow_ify` の `check_clone_names_are_fresh` と
`check_ownership_is_levelled` は `config.develop_mode` が真のときだけ走る
(`CODE src/build/build_object_files.rs: optimize_rc_program`, `CODE src/rc_ir/borrow.rs: borrow_ify`)。
第 3 節の表で A9 と A11 の検査として挙がっているのが `validate` の中の検査であり、A13 の検査として
挙がっているのが `check_clone_names_are_fresh` である。**出荷ビルドではこの 3 つの仮定は果たす者だけに
立つ。**

`check_clone_names_are_fresh(prog: &RcProgram, renames: impl Iterator<Item = &'a Map<FullName, FullName>>)`
も `check_ownership_is_levelled(&self, func: &RcFunc)` も、共有参照を取って値を返さない
(`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`,
`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`)。よって `borrow_ify` が組み立てる
`RcProgram` は `develop_mode` によらず、T の主語は 2 つのビルドで同じプログラムである。

### 引用した命題自身の状態

T は P14・P23・P24・P26・P27 の言明を引用する。それらが対象コミットに対してどこまで証明され、どこまで
検証されているかは README 第 7 節の表が述べる。**T が閉じることは、それらが閉じることを意味しない。**

P23 の言明は「D12 の意味で RC 規律を満たすプログラム」一般について述べるが、その下にある A19 (ii-b)
と P14a は `borrow_ify` の出力についてだけ書かれている (<1>4)。T が P23 に渡すのは `borrow_ify` の
出力そのものなので、T はこの差に触れずに済む。
