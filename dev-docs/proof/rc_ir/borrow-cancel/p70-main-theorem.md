# T -- パイプラインが保存するもの

対象コミット: `30ef78ce`

このファイルは主定理 T を証明する。T は合成であり、引用する命題の**言明**だけを使う (証明は
読まない)。第 2 節の `BY` の行が引くのは次のものである。

- **命題**: P9・P12・P14・P14a・P14b・P22・P23・P24・P26・P27。
- **定義**: D1・D11・D12・D14・D18・D19・D23・D24・D30。
- **仮定**: A1・A2 (前提 H1・H2 として)、A13・A17・A18・A19・A20 (前提 H3 が束ねる第 4 節の仮定の
  中から名指す)。
- **コード**: `optimize_rc_program` が 2 つのパスを呼ぶ順序、`validate`、`borrow_ify` と `cancel` が
  `RcProgram` を組み立てる 3 つの欄、`cancel` の本体の書き換えが `drop_nodes` を呼ぶこと、
  `clone_func`、`borrow_funcref`、`RcProgram` の型。

T の言明はこのほかに、D7・D21・D25・D26・D31 を名前で引く。

第 3 節が、T が立つ 25 個の仮定を果たす者とともに 1 か所に並べる。第 4 節が、T が届かないところを
述べる。**この定理が読み手に何を与え、何を与えないかを決めるのは第 4 節である。**第 4 節はこのほかに、
`cancel` の後に走る 3 種のパスと、最適化水準の述語と、開発ビルドでだけ走る検査のコードを引く。

(T3) は D30 (対応する 2 つの実行と、その共通接頭) の上に立つ。`<1>9` が P26 の 2 つの実例を 1 つの
`p1` の実行の上で連結する。

## 1. T の言明

以下、`p0` を `borrow_ify` の入力とし、

    p1 == borrow_ify(p0, type_env, develop_mode)
    p2 == cancel(p1, type_env)

と書く。この 3 つの名前はこのファイルの中でだけ使う。

**T** (パイプラインが保存するもの)

    ASSUME NEW p0 : RcProgram, NEW type_env : TypeEnv, NEW develop_mode : bool,
      H0. `p0` は、`optimize_rc_program` の 1 回の呼び出しにおいて `split_rc_units` を呼んだ後の
          `prog` の値であり、`type_env` はその呼び出しの引数 `type_env`、`develop_mode` は
          その呼び出しの `config.develop_mode` である。
      H1. `p0` について A1 が成り立つ。すなわち `p0` は D12 の意味で RC 規律を満たし、
          `p0` のすべての関数の `borrowed_units` は空である。
      H2. `p0` について A2 が成り立つ。すなわち `p0` のすべての `Retain`/`Release` 節点の path は、
          その変数の型の `rc_units` の要素である。
      H3. 第 4 節の仮定がすべて成り立つ。第 4 節が挙げるのは A1 から A25 までの 25 個であり、
          そのうち A1 と A2 を `p0` について述べたものが H1 と H2 である (第 3 節が 25 個すべてを
          並べる)。
    PROVE
      (T1) `p2` は D12 の意味で RC 規律を満たす。
      (T2) `p2` のどの実行 (D24) においても、解放されたオブジェクトの読み (D7) は起きず、どの
           オブジェクトも高々 1 回しか解放されない。さらに `p2` の正常終了する (D31) 実行では、
           計数下のオブジェクト (D26) はちょうど 1 回解放される -- 環境が持つ参照から到達できる
           (D25) ものを除く。
      (T3) `X0`・`X1`・`X2` をそれぞれ `p0`・`p1`・`p2` の**実際の**実行 -- 観測点が返す値が
           その時点の参照カウントと一致する実行 (P26、D21) -- とし、D30 が `borrow_ify` について
           `(X0, X1)` を対応させ、D30 が `cancel` について `(X1, X2)` を対応させるとする。
           `(X0, X1)` の共通接頭を `C01`、`(X1, X2)` の共通接頭を `C12` と書く。`X1` の段のうち、
           `C01` の中で `X0` の段と対になっており、かつ `C12` の中で `X2` の段と対にもなっている
           ものの全体を `C` と書く。

           **`C` は `X1` の段の列の接頭とは限らない。** D30 の `π` は `borrow_ify` か `cancel` の
           どちらか 1 つであり、合成 `cancel(borrow_ify(・))` は `π` ではないので、`X0` と `X2` を
           直接対応させる共通接頭は存在しない。`C01` と `C12` は `X1` の段の異なる集合であって、
           一方が他方を含むとは限らない。

           `C` の段が一意性の観測点 (D18) を訪れるとき、`C01` がその段と対にした `X0` の段の
           観測値が真ならば、`C12` がその段と対にした `X2` の段の観測値も真である。この 3 つの
           段が訪れる 3 つの観測点は、D19 の 2 つの対応を合成したもので結ばれている。

           **`C` が終わった後については何も主張しない。**とくに `C` が終わる出口が (X1) か (X2) か
           (X3) かを述べない。理由は第 4 節が述べる。
      (T4) `p0` から `p2` を作る 2 つのパスの合成は、D12 が見ない部分について次を保つ。

           - **`roots`**: `p2.roots` は `p0.roots` に等しい。
           - **関数**: `p2.funcs` の各エントリには、P24 の「出力の各関数は入力のちょうど 1 つの
             関数から作られる」という対応を 2 度たどって `p0.funcs` のちょうど 1 つの関数が定まり、
             その `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` はその関数のものに
             等しい。また `p0.funcs` の各関数の `name` は `p2.funcs` の鍵である。

             **`p1` と `p2` は `p0` に無い関数を持つ。**`borrow_ify` は借用版を足すからである
             (P24)。借用版の名前は、`p0.funcs` のどの鍵とも、`p0.funcs` のどの関数の `name` とも
             異なる。この言明が借用版について述べるのは、上の 4 つの欄を複製元の関数と共有する
             ことだけである。
           - **本体**: `p2` の各本体が、上の対応でたどった `p0` の本体と違うのは、`Retain`/`Release`
             の節点と、`App` の callee の名前だけである。節点の種類・その順序・`Let` の束縛変数・
             `Match` のアームの構成・`Llvm` の op とオペランド・`Destructure` のフィールドは、
             いずれも `p0` の本体のものに等しい (複製の名前替えを P9 で戻したうえで)。
           - **`App` の callee**: `p2` の各本体の各 `App` 節点の callee の名前は、上の対応で
             たどった `p0` の本体の同じ位置の `App` 節点の callee と**同じ関数の版**である --
             元の呼び出し先そのものか、その借用版のどちらかである。呼び出し先が `p0` の関数を
             名指すとき、その名前は `p2.funcs` の鍵である。局所変数を経由する間接呼び出しでは、
             名前は元のものである (複製の名前替えを P9 で戻したうえで)。
           - **グローバル初期化子**: `p2.globals` は `p0.globals` と同じ長さの列であり、第 `i`
             要素の `symbol` と `ty` は `p0.globals` の第 `i` 要素のものに等しく、
             `owns_initializer` と `owns_storage` は `true` である。D1 が述べる呼び出し順に
             より、この `true` は正しい値である。

(T1) と (T4) は D12 の見る部分と見ない部分を分けて述べる。(T2) は (T1) から実行の水準へ渡る。
(T3) は D12 の軸の上に無い性質であり、(T1) からは出ない。

**(T4) の `App` の callee の節が無いと、T は呼び出し先の差し替えを許す。**`fn_ty`・`params` の型・
`ret_ty`・`borrowed_units` が一致する 2 つの関数 `g` と `h` を取り、ある `App(g, args)` の callee を
`h` に書き替えるパスを考える。`rhs_consumes` が読むのは呼び出し先の `params` と `borrowed_units`
だけなので `Obl` の推移は 1 段も変わらず、(T1) と (T2) は成り立つ。(T3) の共通接頭はその段で終わるので
(T3) の結論も成り立つ。(T4) の残る 4 つの節は「変えてよいもの」を数え上げるだけなので破れない。
**足し算の代わりに引き算をするパスが、それ以外の全項を満たす。**この節がそれを閉じる。

## 2. 証明

### <1>1. `optimize_rc_program` が `borrow_ify` に渡すプログラムは `p0`、`cancel` に渡すプログラムは `p1` であり、`cancel` の返り値は `p2` である

  **<2>1.** `optimize_rc_program` の本体には、次の 6 行がこの順で連続して現れる。

  ```rust
  split_rc_units(&mut prog, type_env);
  validate(&prog, "after split_rc_units");
  if config.enable_borrow_optimization() {
      prog = borrow_ify(&prog, type_env, config.develop_mode);
      validate(&prog, "after borrow_ify");
      prog = cancel(&prog, type_env);
  ```

  この 6 行の間に、束縛 `prog` に書く文はほかに無い。4 行目から 6 行目は 3 行目の `if` の本体に
  あり、その条件式は `config` のメソッド呼び出しであって `prog` を読まない。`optimize_rc_program`
  の本体で `borrow_ify` を呼ぶのは 4 行目だけ、`cancel` を呼ぶのは 6 行目だけである。

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`

  **<2>2.** `optimize_rc_program` の中の `validate` の呼び出しは、束縛 `prog` の値を変えない。
  `validate` は同じ関数の中で束縛された次の閉包である。

  ```rust
  let validate = |prog: &RcProgram, stage: &str| {
      if config.develop_mode {
          validate::validate(prog, &symbol_names, type_env, stage);
      }
  };
  ```

  この閉包は `RcProgram` を共有参照で受け取り、値を返さない。呼ぶ先の
  `pub fn validate(prog: &RcProgram, symbol_names: &Set<FullName>, type_env: &TypeEnv, stage: &str)`
  も同じである。よって `validate` の呼び出しは、それがどこに置かれていても、束縛 `prog` の値を
  変えない。とくに `<2>1` が引用した 2 つの呼び出し -- `"after split_rc_units"` と
  `"after borrow_ify"` -- がこれに当たる。

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`,
       `CODE src/rc_ir/validate.rs: validate`

  **<2>3. QED**

  H0 が固定するのは `optimize_rc_program` の 1 回の呼び出しである。その呼び出しにおいて、
  `split_rc_units(&mut prog, type_env)` の直後の束縛 `prog` の値が `p0` であり、その呼び出しの
  引数 `type_env` が T の `type_env`、その呼び出しの `config.develop_mode` が T の `develop_mode`
  である。次の文 `validate(&prog, "after split_rc_units")` は `prog` を変えない (`<2>2`)。`if` の
  条件式も `prog` を読まない (`<2>1`)。よって `if` の本体の 1 行目に着く時点の `prog` は `p0` で
  ある。

  その 1 行目 `prog = borrow_ify(&prog, type_env, config.develop_mode)` は、第 1 引数に `p0` を、
  第 2 引数にその呼び出しの `type_env` を、第 3 引数にその呼び出しの `config.develop_mode` を
  渡す。H0 よりこの 3 つは順に `p0`・T の `type_env`・T の `develop_mode` であるから、この行の後の
  `prog` は第 1 節の `p1` である。

  2 行目 `validate(&prog, "after borrow_ify")` は `prog` を変えない (`<2>2`)。よって 3 行目
  `prog = cancel(&prog, type_env)` が受け取るのは `p1` であり、その第 2 引数は `p1` を作った行と
  同じ `type_env`、すなわち T の `type_env` なので、その返り値は第 1 節の `p2` である。`<2>1` より、
  この 6 行の間に `prog` に書く文はほかに無く、`borrow_ify` と `cancel` を呼ぶ位置もそれぞれ
  この 1 つである。

    BY <2>1, <2>2, H0

### <1>2. `p0` は D12 の意味で RC 規律を満たす

    BY H1, A1

(H1 は A1 の本文をそのまま述べており、その第 1 文がこれである。)

### <1>3. `p1` は D12 の意味で RC 規律を満たす

P14 の言明の前提は 3 つである -- 入力が D12 の意味で RC 規律を満たすこと、入力が A1 を満たすこと、
入力が A2 を満たすこと。この 3 つを `p0` について与えるのが `<1>2`、H1、H2 である。P14 の結論は
`borrow_ify` の出力が D12 の意味で RC 規律を満たすことであり、その出力は `p1` である
(第 1 節の `p1` の定義)。

    BY <1>2, H1, H2, P14

### <1>4. `p1` は P23 の言明が入力に置く条件を満たし、A19 (ii-b) と P14a は `p1` の各本体について読める

P23 の言明は入力を **`borrow_ify` の出力**に限る。第 1 節の `p1` の定義より `p1` は `borrow_ify` の
返り値であり、`<1>1` より `cancel` が受け取るプログラムでもある。よって `p1` は P23 の言明が入力に
置く条件を満たす。

P23 がその限定の理由に挙げる 2 つも、同じ理由で `p1` について読める。A19 の量化範囲は
「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の入力) の
両方」であり、`p1` は後者である。P14a の量化範囲は「`borrow_ify` の出力の各本体」であり、`p1` は
それである。

    BY <1>1, P23, H3, A19, P14a

### <1>5. (T1) `p2` は D12 の意味で RC 規律を満たす

P23 の言明が入力に置く条件は「入力が `borrow_ify` の出力であること」であり、それを `p1` に
ついて与えるのが `<1>4` である。P23 の結論は「`cancel` の出力**も** D12 の意味で RC 規律を
満たす」であり、その「も」が前提する「入力が D12 を満たす」を `p1` について与えるのが `<1>3`
である。`cancel` が受け取るのが `p1` であり、その返り値が `p2` であることを与えるのが `<1>1`
である。

    BY <1>1, <1>3, <1>4, P23

### <1>6. `p2` のすべての本体は、`p2` の `borrowed_units` が定める所有と借用の割り当て (D14) の下で D11 を満たす

D12 は、プログラム `P` が RC 規律を満たすことを、`P` のすべての関数の本体と、すべてのグローバル
初期化子の `init` が、`P` の `borrowed_units` が定める所有と借用の割り当て (D14) の下で D11 を
満たすことと定める。D11 はその割り当てを引数に取るので、割り当てを言わずに「D11 を満たす」とは
書けない。D23 より、本体とは関数の `body` かグローバル初期化子の `init` であり、この 2 種で
尽きる。

    BY <1>5, D14, D23 DEF D12

### <1>7. (T2)

  **<2>1.** P27 の言明が無修飾で書く「プログラムのすべての本体が D11 を満たし」は、そのプログラム
  自身の `borrowed_units` が定める所有と借用の割り当て (D14) の下で読む。

  D11 は割り当てを引数に取るが、D14 はその割り当てを 1 つのプログラムについて一意に定める --
  関数の各パラメータ・capture の各 unit は所有か借用かのどちらかであり、借用するものの集合が
  その関数の `RcFunc::borrowed_units` の欄、残りが所有するものである。よって 1 つのプログラムの
  本体について「D14 の割り当ての下で D11 を満たす」と言うとき、割り当ては `borrowed_units` の欄が
  決めるもの 1 つである。D12 が「`P` の `borrowed_units` が定める所有と借用の割り当て (D14) の
  下で」と書くのはこの割り当てである。

  P27 の結論が読む割り当ても同じである。P27 が語るのはそのプログラムの実行 (D24) であり、D23 は
  「D9 の `App` の行と D10 の生成の `App` の行が『呼び出し先』と言うのは実行時の関数である」
  「D9 の `App` の行が読む所有は D14 が `RcFunc::borrowed_units` から定めるものなので、その
  呼び出し先はプログラムの `funcs` の関数である」と定める。すなわち実行の段が読む割り当ては、
  実行されるプログラムの `funcs` の各関数の `borrowed_units` が定めるものである。

    BY D12, D14, D23, D24, P27

  **<2>2. QED**

  P27 の言明の前提は、プログラムのすべての本体が D11 を満たすこと、および A17 と A18 が成り立つ
  ことである。`<2>1` より第 1 の前提は、そのプログラム自身の `borrowed_units` が定める割り当ての
  下で読む。`<1>6` は `p2` についてまさにその形 -- 「`p2` の `borrowed_units` が定める所有と借用の
  割り当て (D14) の下で」-- でそれを与える。H3 が A17 と A18 を与える。

  (T2) は 3 つを主張し、その 3 つは P27 の結論の 3 つである。第 1 は (R1) (解放されたオブジェクトの
  読みは起きない)、第 2 は (R2) (どのオブジェクトも高々 1 回しか解放されない)、第 3 は P27 の系
  「正常終了する実行では、計数下のオブジェクトはちょうど 1 回解放される -- ただし環境が持つ参照から
  到達できるものを除く」である。

    BY <1>6, <2>1, H3, A17, A18, P27

### <1>8. `p2` の各一意性の観測点に、`p1` の観測点と `p0` の観測点がちょうど 1 つずつ定まる

D19 は、`borrow_ify` と `cancel` のそれぞれについて、**出力の**各一意性の観測点が入力のちょうど
1 つの観測点から来ることを言う。`<1>1` より `p1` は `borrow_ify` の出力であり、かつ `cancel` の
入力である。

D19 を `cancel` (入力 `p1`、出力 `p2`) に当てると、`p2` の各観測点 `q2` に `p1` の観測点 `q1` が
ちょうど 1 つ定まる。その `q1` に D19 を `borrow_ify` (入力 `p0`、出力 `p1`) に当てると、`p0` の
観測点 `q0` がちょうど 1 つ定まる。この 2 段を続けて適用すれば、`q2` から `q1` と `q0` が 1 つずつ
定まる。

**量化は出力の側で行う。**D19 が「ちょうど 1 つ」と言うのは出力の観測点から入力の観測点への向き
だけであり、逆向きではない。`borrow_ify` は関数の借用版を作り、D19 は「借用版の本体の観測点は、
複製元の本体の同じ位置の観測点に対応する」と述べるので、`p0` の 1 つの観測点に `p1` の観測点が
複数対応しうる -- 原本の中の観測点と、借用版の中の同じ位置の観測点である。

    BY <1>1, D19

### <1>9. (T3)

    ASSUME NEW X0, NEW X1, NEW X2,
      H4. `X0`・`X1`・`X2` はそれぞれ `p0`・`p1`・`p2` の実際の実行である。
      H5. D30 は `borrow_ify` について `(X0, X1)` を対応させる。その共通接頭を `C01` と書く。
      H6. D30 は `cancel` について `(X1, X2)` を対応させる。その共通接頭を `C12` と書く。
    PROVE
      (T3) の結論。

(H4-H6 は `<1>9` の証明の中でだけ立つ。)

  **<2>1.** H5 と H6 の 2 つの読み方は、どちらも D30 が要求する形である。

  D30 は 1 つのパス `π` と、その入力 `P` と出力 `P' = π(P)` について書かれている。H5 は `π` を
  `borrow_ify`、`P` を `p0`、`P'` を `p1` と読む。H6 は `π` を `cancel`、`P` を `p1`、`P'` を
  `p2` と読む。`<1>1` より `p1` は `borrow_ify` の出力であり、かつ `cancel` の入力であるから、
  どちらの読み方も D30 の `P` と `P'` の要求を満たす。**`X1` は 2 つの読み方で同じ実行である。**

  **`X1` から `X0` と `X2` が定まるとは言わない。** D30 が定めるのは 2 つの実行の**組の取り方** --
  環境が与える入力を同じにし、複数の制御の流れがある場合は段の並び方を同じにすること -- であって、
  片方から相手が存在することも一意であることも述べない。実行が環境の入力と段の並び方だけで決まる
  ことは D24 も D30 も述べない。よって (T3) は組を与件として量化する。

    BY <1>1, D30

  **<2>2.** P26 を `borrow_ify` の実例 (H4、H5) に当てると、次の 2 つが出る。

  - `C01` の上の各一意性の観測点 (D18) で、`X0` の観測値と `X1` の観測値は**等しい**。とくに
    `X0` の観測値が真ならば、D19 が対応させる `p1` の観測点の `X1` における観測値も真である。
  - `C01` が終わる出口は (X1) ではない。

  **P26 が言明で量化するのは「D30 が対応させる 2 つの実際の実行」である。** `X0` と `X1` が
  実際の実行 -- 観測点が返す値がその時点の参照カウントと一致する実行 -- であることを与えるのは
  H4 であり、H5 が与えるのは D30 の対応だけである。

  P26 は第 2 文 -- 共通接頭が終わる出口が許された向きであること -- を `borrow_ify` については
  主張しない。P26 の言明がそう書いている。

  > **出口の向きを言うのは `π` が `cancel` のときである。** `borrow_ify` については (X3) が開くので
  > ((D30))、出口が (X1) か (X2) であることを言う者が居ない。代わりに `borrow_ify` の側は第 1 文が
  > 等号で立つ -- 共通接頭の上で 2 つの観測値は**等しい**ので (X1) の出口が起きない。(X2) と (X3) の
  > 出口がどちらの向きであるかは、この文書が示さないものである。

    BY <2>1, H4, H5, P26, D30

  **<2>3.** P26 を `cancel` の実例 (H4、H6) に当てると、次の 2 つが出る。

  - `C12` の上の各一意性の観測点で、`X1` の観測値が真ならば、D19 が対応させる `p2` の観測点の
    `X2` における観測値も真である。これは P26 の第 1 文である。
  - `C12` が終わる出口は (X1) か (X2) であり、(X1) では `X1` の観測値が偽で `X2` の観測値が真、
    (X2) では `X1` が複製を作り `X2` が作らない。

  `X1` と `X2` が P26 の量化する実際の実行であることを与えるのは H4 である。

  第 2 項は 2 つを合わせたものである。**出口が (X3) でないことを言うのは D30 である。**

  > **(X3) は `π` が `borrow_ify` のときに開く。** `cancel` については、削除される `Retain` と
  > その群の `Release` が同じ実行路の上で対になるので (P19、P20)、解放の段は動かない。

  D30 は共通接頭が終わる点を (X1)・(X2)・(X3) の 3 種に尽くすので、`π` が `cancel` である
  この実例では出口は (X1) か (X2) である。その 2 つの向き -- (X1) では入力の観測値が偽で出力の
  観測値が真、(X2) では入力が複製を作り出力が作らない -- を言うのが P26 の第 2 文である。

    BY <2>1, H4, H6, P26, D30

  **<2>4.** 一意性の観測点を訪れる `X1` の段は、`C01` の中にあれば `X0` の段と対になり、`C12` の
  中にあれば `X2` の段と対になる。

  D30 は 2 つの実行の段を先頭から順に 1 対 1 に並べ、その並びから除くのは「`π` が消した節点と入れた
  節点の段」である。並びは共通接頭までで定まり、共通接頭の外の段はどの段とも対にならない。すなわち
  `X1` の 1 つの段が対を持たないのは、それが除かれる段であるか、共通接頭の外にあるかのどちらかの
  ときである。

  一意性の観測点は `Llvm` の演算 1 つが現れる位置である (D18)。D19 は `borrow_ify` と `cancel` に
  ついて「値を計算する構文を作らず、消さず、並べ替えない」と述べ、P24 は「本体について書き換えが
  変えるのは、`Retain`/`Release` の節点と、`App` の callee の名前だけである」と述べる。よって
  どちらのパスも観測点の節点を消さず、入れないので、観測点を訪れる段は除かれる段ではない。よって
  その段が対を持たない理由は共通接頭の外にあることだけであり、`C01` の中にあれば `X0` の段と、
  `C12` の中にあれば `X2` の段と対になる。

  **除かれる段は 2 つの実例で別である。** `C01` の側で除かれるのは `borrow_ify` が消した節点と
  入れた節点 (`call_rc` が置く `Retain`/`Release`) の段であり、`C12` の側で除かれるのは `cancel` が
  消した節点 -- そのうちには `p0` から来て `borrow_ify` が残した節点もある -- の段である。2 つの対の
  定義域は、どちらも `X1` の段の列を先頭から取ったものから除かれる段を落としたものであり、落とす先が
  この 2 つで別である。よって一方が他方を含むとは限らず、`C01` と `C12` を `X1` の段の列の接頭として
  比べることはできない。

    BY D18, D19, D30, P24

  **<2>5. QED**

  `C` を第 1 節の (T3) が定めたものとする。`C` の 1 つの段 `s` を取り、`s` が一意性の観測点を
  訪れるとする。`C` の定め方より `s` は `C01` の中で `X0` の段 `s0` と対になっており、`C12` の中で
  `X2` の段 `s2` と対になっている。`<2>4` より、観測点を訪れる `X1` の段は `C01` と `C12` の両方の
  中にあれば両方で対を持つ。すなわち `C` は、2 つの共通接頭がともに及ぶ範囲にある観測点の段を
  落とさない。

  `s` が訪れる `p1` の観測点を `q1` とする。D30 は段を「節点の対応 (D19 と P22)」で並べるので、
  `s2` が訪れる `p2` の観測点を `q2` とすると、`q2` に D19 が対応させる `p1` の観測点は `q1` で
  あり、`s0` が訪れる `p0` の観測点は `q1` に D19 が対応させるものである。`<1>8` より、`q2` から
  `q1` が、`q1` から `p0` の観測点が 1 つずつ定まるので、この 3 つは `<1>8` の合成で結ばれた 3 つ
  である。

  `s0` の観測値が真であるとする。`s` は `C01` に在るので、`<2>2` の第 1 項より `s` の観測値も真で
  ある。`s` は `C12` にも在るので、`<2>3` の第 1 項より `s2` の観測値も真である。含意を 2 つ
  つないだこれが (T3) の結論である。

  **出口について述べることは無い。** (T3) は `C` が終わった後について何も主張しないので、`C` が
  終わる出口がどれであるかを示す義務がない。示せない理由は第 4 節が述べる。

    BY <1>8, <2>1, <2>2, <2>3, <2>4, D30

### <1>10. (T4)

  **<2>1.** `borrow_ify` が返す `RcProgram` の 3 つの欄を、コードの上で読む。

  - **`funcs`**: 型は `Map<FuncRef, RcFunc>` であり、空の `Map` から始まる。これに触れるのは
    3 つのループだけである。

    第 1 のループは `prog.funcs.values()` の各 `func` について `f_own = func.clone()` を作り、
    `f_own.body` だけを `ctx.rewrite(&f_own.body)` に差し替えて `funcs.insert(f_own.name.clone(),
    f_own)` を行う。第 2 のループは、借用版を持つ `func` について `clone_func` が作った `clone` の
    `body` を差し替えて `funcs.insert(borrow_version, clone)` を行う。`clone_func` は
    `name: new_ref` を置き、その `new_ref` は `borrow_version` そのものなので、`clone.name` は
    鍵 `borrow_version` に等しい。**第 3 のループは `funcs.values_mut()` を走り、各エントリの
    `borrowed_units` の欄だけを書き替える。**

    ```rust
    for func in funcs.values_mut() {
        func.borrowed_units = param_capture_units(func, type_env)
            .into_iter()
            .filter(|unit_path| !owned_units.contains(unit_path))
            .collect();
    }
    ```

    この 3 つのループが `funcs` に対して呼ぶのは `insert` と `values_mut` の 2 つだけである。写像の
    `insert` は鍵を足すか既にある鍵の値を差し替えるかのどちらかであり、`values_mut` は値だけを可変に
    走査する。どちらも鍵を取り除かない。ここから 3 つが出る。

    1. `p1.funcs` の各エントリは、2 つの挿入のループのどちらかが入れた値の `borrowed_units` を
       第 3 のループが書き替えたものである。**(T4) が名指す 4 つの欄 -- `fn_ty` / `ret_ty` /
       `params` の型 / `inline_into_callers` -- は、第 3 のループが書き替える欄ではない。**
    2. **`p1.funcs` のどのエントリも、鍵はその関数の `name` に等しい。**第 1 のループは
       `f_own.name` を鍵に取り、第 2 のループは `clone.name` に等しい `borrow_version` を鍵に
       取る。第 3 のループは `name` も鍵も変えない。
    3. **`p0.funcs` の各関数の `name` は `p1.funcs` の鍵である。**第 1 のループは
       `prog.funcs.values()` を走り、各 `func` について `func.name` を鍵に挿入する。その後の挿入も
       `values_mut` も鍵を取り除かない。
  - **`globals`**: `prog.globals.iter().map(|g| RcGlobalInit { symbol: g.symbol.clone(),
    ty: g.ty.clone(), init: ctx.rewrite(&g.init), owns_initializer: true, owns_storage: true })`
    を `collect` した列である。よって `p1.globals` は `p0.globals` と同じ長さであり、第 `i`
    要素の `symbol` と `ty` は `p0.globals` の第 `i` 要素のものに等しい。
  - **`roots`**: `roots: prog.roots.clone()`。よって `p1.roots` は `p0.roots` に等しい。

    BY `CODE src/rc_ir/borrow.rs: borrow_ify`, `CODE src/rc_ir/borrow.rs: clone_func`,
       `CODE src/rc_ir/ast.rs: RcProgram`

  **<2>1a.** 第 2 のループが入れる各借用版の名前は、`p0.funcs` のどの鍵とも、`p0.funcs` のどの
  関数の `name` とも異なる。

  第 2 のループの鍵 `borrow_version` は、`borrow_ify` が `borrow_versions` に入れた値である。
  その値を作るのは `borrow_versions.insert(func.name.clone(), borrow_funcref(&func.name))` の 1 行
  だけであり、`borrow_funcref` は元の名前の `name` フィールドの文字列に `"#borrow"` を継ぎ足す
  (`borrow_name.name.push_str("#borrow")`)。よって借用版の名前を `#` で区切った最後の断片は
  `borrow` である。

  A13 は、`borrow_ify` の**入力に現れるすべての名前**について、`name` フィールドを `#` で区切った
  最後の断片が `borrow` ではないと述べ、その理由として `borrow_funcref` が借用版の名前を
  `<元の名前>#borrow` として作ることを挙げる。A13 が挙げる名前の範囲には **`prog.funcs` の鍵**と
  **各 `RcFunc` の `name`** が入っている。`p0` は `borrow_ify` の入力であるから、`p0.funcs` の鍵にも、
  そこに収まる各関数の `name` にも A13 が当たる。

    BY <2>1, H3, A13, `CODE src/rc_ir/borrow.rs: borrow_ify`,
       `CODE src/rc_ir/borrow.rs: borrow_funcref`

  **<2>2.** P24 は、`borrow_ify` と `cancel` のそれぞれについて、次の**4 つ**を述べる。

  1. `roots` を変えない。
  2. **出力の各関数は入力のちょうど 1 つの関数から作られ**、その `fn_ty` / `ret_ty` /
     `params` の型 / `inline_into_callers` は元の関数のものに等しい。
  3. 出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の
     第 `i` 要素のものに等しい。`owns_initializer` と `owns_storage` には `true` を書き、D1 が
     述べる呼び出し順によりこの書き込みは正しい値を書く。
  4. **本体について書き換えが変えるのは、`Retain`/`Release` の節点と、`App` の callee の名前だけ
     である。**節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の op と
     オペランド・`Destructure` のフィールドは、いずれも元の本体のものに等しい (複製の名前替えを
     P9 で戻したうえで)。

  **P24 は関数の対応を鍵の一致では述べない。**「出力の各関数は入力のちょうど 1 つの関数から
  作られ」という形であり、その理由も P24 自身が挙げている -- `borrow_ify` は入力に無い関数
  (借用版) を足すからである。以下、この 2 番目の項が与える関係を **P24 の対応**と呼ぶ。

    BY P24, P9

  **<2>2a.** `borrow_ify` の書き換えが `App` の callee に入れる名前は `route` の返り値であり、
  それは元の呼び出し先と同じ関数の版である。

  `<2>2` の 4 より、`borrow_ify` の書き換えは節点の種類もその順序も変えない。よって出力の各 `App`
  節点は、入力の同じ位置の `App` 節点を書き換えたものである。

  `RewriteCtx::rewrite_inner` で `App` を rhs とする節点を受けるのは、第 1 の腕
  `RcExpr::Let(x, RcRhs::App(callee, args), k)` である。Rust の `match` は腕を上から順に試すので、
  `rhs` を clone する第 3 の腕 `RcExpr::Let(x, rhs, k)` へは落ちない。第 1 の腕は
  `let callee = self.route(x, callee, args, k);` を作り、それをそのまま
  `RcExpr::Let(x.clone(), RcRhs::App(callee, args.clone()), k)` に据える。よって出力の各 `App`
  節点の callee は `route` の返り値である。

  P12 は「`route` が返す呼び出し先は、元の呼び出し先と同じ関数の版である (元の版そのものか、その
  `borrow_versions` の像)」「呼び出し先が入力の関数を名指すとき、返る名前は出力の `funcs` の鍵で
  ある」「局所変数を経由する間接呼び出しでは `route` は呼び出し先をそのまま返し、その名前は
  どちらの `funcs` の鍵でもない」と述べる。

    BY <2>2, P12, `CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner`

  **<2>3.** `cancel` が返す `funcs` の鍵を、コードの上で読む。`cancel` は `prog.funcs.values()` の
  各 `f` について `let mut clone = f.clone();` を作り、`clone.body` だけを `cancel_body` の結果に
  差し替えて `(f.name.clone(), clone)` を収める。`funcs` を作るのはこの 1 つの `map` であり、
  ほかに `funcs` へ書くところは無い。

  よって `p2.funcs` の鍵の集合は `{ f.name : f ∈ p1.funcs.values() }` である。`<2>1` の 2 より
  `p1.funcs` のどのエントリも鍵はその `name` に等しいので、この集合は `p1.funcs` の鍵の集合に
  等しい。`<2>1` の 3 と合わせて、**`p0.funcs` の各関数の `name` は `p2.funcs` の鍵である。**

    BY <2>1, `CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/ast.rs: RcProgram`

  **<2>3a.** `cancel` が返す `globals` と `roots` を、コードの上で読む。`cancel` は
  `prog.globals.iter().map(|g| RcGlobalInit { symbol: g.symbol.clone(), ty: g.ty.clone(),
  init: cancel_body(&vars, &g.init), owns_initializer: true, owns_storage: true })` を `collect`
  した列を返す。よって `p2.globals` は `p1.globals` と同じ長さであり、第 `i` 要素の `symbol` と
  `ty` は `p1.globals` の第 `i` 要素のものに等しい。`roots` は `roots: prog.roots.clone()` なので
  `p2.roots` は `p1.roots` に等しい。

    BY `CODE src/rc_ir/borrow.rs: cancel`

  **<2>3b.** `cancel` は `App` の callee の名前を変えない。

  `cancel` が本体に対して行う書き換えは、同じ関数の中で束縛された閉包 `cancel_body` であり、その
  返り値は `drop_nodes(body, &analysis.cancelled())` である。`cancel` が `funcs` に収める本体も
  (`<2>3`)、`globals` に収める本体も (`<2>3a`)、どちらもこの閉包の返り値である。

  P22 は「`drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release` 節点だけを
  取り除いた木を返し、他の節点の種類・変数・path・並びを変えない」と述べる。`App` の callee は
  その節点の変数である。

    BY <2>3, <2>3a, P22, `CODE src/rc_ir/borrow.rs: cancel`

  **<2>4.** `owns_initializer` と `owns_storage` について。`<2>2` の 3 を 2 つのパスに続けて
  当てると、この 2 つの欄には 2 回とも `true` が書かれ、最後に書かれた値も `true` である。その
  `true` が正しい値であることも `<2>2` の 3 が述べる。

    BY <2>2, D1

  **<2>5. QED**

  **`roots` について。**`roots` はプログラム自身の欄なので、関数の対応は要らない。

      p2.roots
        = p1.roots        [BY <2>3a]
        = p0.roots        [BY <2>1]

  `=` と `=` は等号の推移で `=` に閉じる。

  **関数について。**`g2` を `p2.funcs` の任意のエントリとする。`<2>2` の 2 を `cancel` (入力
  `p1`、出力 `p2`) に当てると、`p1.funcs` の関数 `g1` がちょうど 1 つ定まり、4 つの欄が等しい。
  同じく `<2>2` の 2 を `borrow_ify` (入力 `p0`、出力 `p1`) に当てると、`g1` に `p0.funcs` の
  関数 `f` がちょうど 1 つ定まり、4 つの欄が等しい。`φ` を `fn_ty` / `ret_ty` / `params` の型 /
  `inline_into_callers` のいずれかとすると

      φ(g2)
        = φ(g1)           [BY <2>2 の 2 (`cancel` について)]
        = φ(f)            [BY <2>2 の 2 (`borrow_ify` について)]

  であり、等号の推移で `φ(g2) = φ(f)` に閉じる。`g1` も `f` もちょうど 1 つなので、この 2 段の
  合成も `g2` に `f` をちょうど 1 つ与える。これが (T4) の関数についての第 1 の主張である。
  第 2 の主張 -- `p0.funcs` の各関数の `name` が `p2.funcs` の鍵であること -- は `<2>3` が述べ、
  借用版の名前が `p0.funcs` のどの鍵とも どの関数の `name` とも異なることは `<2>1a` が述べる。

  **本体について。**`<2>2` の 4 を 2 つのパスに続けて当てる。`borrow_ify` が変えるのは
  `Retain`/`Release` の節点と `App` の callee の名前だけであり、`cancel` が変えるのも同じ 2 つ
  だけなので、合成が変えるのもこの 2 つだけである。列挙された欄 -- 節点の種類・その順序・`Let` の
  束縛変数・`Match` のアームの構成・`Llvm` の op とオペランド・`Destructure` のフィールド -- は
  各段で等しいので、等号の推移で `p0` の本体のものに等しい。名前替えを P9 で戻す必要があるのは
  `borrow_ify` の複製の段だけである -- `cancel` が作る各エントリは元の `f` の clone であって、
  関数の名前替えを行わない (`<2>3`)。

  **`App` の callee について。**`g2` の本体の `App` 節点 `n2` を取り、いま述べた本体の対応で
  `p1` の本体の `App` 節点 `n1` と `p0` の本体の `App` 節点 `n0` をたどる。`<2>3b` より `n2` の
  callee の名前は `n1` のものに等しい。`<2>2a` より `n1` の callee の名前は `route` の返り値で
  あり、同じ段より、それは `n0` の callee と同じ関数の版 -- `n0` の callee そのものか、その借用版 --
  である。よって `n2` の callee は `n0` の callee と同じ関数の版である。

  `n0` の callee が `p0` の関数を名指すとき、`<2>2a` よりその名前は `p1.funcs` の鍵であり、
  `<2>3` より `p2.funcs` の鍵の集合は `p1.funcs` の鍵の集合に等しいので、`p2.funcs` の鍵でもある。
  局所変数を経由する間接呼び出しでは `route` は callee をそのまま返すので (`<2>2a`)、名前は複製の
  名前替えを P9 で戻せば `n0` のものである。

  **グローバル初期化子について。**`<2>1` と `<2>3a` より、3 つの列は同じ長さである。`ψ` を
  `symbol` か `ty` とすると

      ψ(p2.globals[i])
        = ψ(p1.globals[i])   [BY <2>3a]
        = ψ(p0.globals[i])   [BY <2>1]

  であり、等号の推移で閉じる。`owns_initializer` と `owns_storage` が `true` であること、および
  その `true` が正しい値であることは `<2>4` が述べる。

    BY <2>1, <2>1a, <2>2, <2>2a, <2>3, <2>3a, <2>3b, <2>4

### <1>11. QED

(T1) は `<1>5`、(T2) は `<1>7`、(T3) は `<1>9`、(T4) は `<1>10` である。

    BY <1>5, <1>7, <1>9, <1>10

## 3. T が立つ仮定と、それを果たす者

T は、引用する命題が証明されている仮定の集合の上に立つ。README 第 4 節の仮定は A1 から A25 までの
25 個あり、その全部がここに載る。「果たす者」は README 第 4 節の言葉である。

右の欄は、README がその仮定を**読む者**として名指すものを挙げる。読む者として定義も命題も
ほかの仮定も名指されていない仮定については、README がその仮定を名指す節、またはその仮定自身の項が
挙げる読み手を書く。

| 仮定 | 内容 | 果たす者 (README 第 4 節) | README が挙げる読み手 |
|---|---|---|---|
| A1 | 入力が RC 規律を満たす | 前段のパス (`insert_rc`) | P14 と T の言明の前提。A19 (i) の量化範囲に D12 を与える (P28 の項) |
| A2 | 単位への正規化 | `insert_rc` と `split_rc_units` | P14 と T の言明の前提。P9 の項 |
| **A3** | 宣言されたモデルの忠実さ | **誰も**。ただし `applies_a_function_operand` については `Generator::apply_lambda` の develop mode の検査が果たす | D10、D21、D24、D29、D30、P5 (a) |
| **A4** | コード生成の忠実さ | **誰も** | D29、D30 |
| A5 | 型が leaf の上位近似 | `leaf_map.rs` の設計 | D25、P7a、A19 (i) |
| A6 | 名前の一意性 | lowering | D6、P9、A11 |
| A7 | 呼び出し先の解決 | `resolve_callee_params` の設計 | 第 8 節 (#551 の 1 件目の直しが健全な近似である根拠) |
| A8 | グローバルは線形規律の外 | `mark_global` | D26、D24 の (E7)、P27 の系 |
| A9 | `Match` はアームを持つ | lowering (検査は develop mode の `validate`) | P16 |
| A10 | 型の well-formedness | `validate_layouts` (最適化が作る型の再検査は develop build だけ) | P1 の言明、A12。A10 自身の項が `boxed_leaf_paths` と `rc_units` の停止性を挙げる |
| A11 | スコープの規律 | lowering (検査は develop mode の `validate`) | D2、P9。A11 自身の項が `origin` の停止性を挙げる |
| **A12** | 束縛の形と型が合っている | **誰も** (項の見出し)。ただし箇条ごとには果たす者が居る -- `Llvm` 節点の `args` の名前の列は演算を作る側 (検査: develop mode の `validate` の `check_rhs`)、`Llvm` 節点の型についての残る 3 つは `struct_punch`・`struct_set` と `struct_plug_in`・`struct_get` と `union_as` が結果の型に取る形。punched でないことを検査するコードは無い | A3。A12 自身の項が P2 と `held_field_type`、`rhs_consumes` の停止性を挙げる |
| A13 | 名前の形 | `Lowerer::fresh_var` と `clone_fresh` (検査は develop mode の `check_clone_names_are_fresh`) | P9 の後半。このファイルの `<1>10` の `<2>1a` |
| A14 | 適用は飽和している (`App` の `args` の個数は呼び出し先のパラメータの個数に**等しい**) | 型検査と lowering (検査は `Generator::apply_lambda` の `assert_eq!`) | A14 自身の項が両向きの読み手を挙げる -- 以下は `call_rc` と `rhs_consumes` の `params[arg_idx]`、以上は D10 の初期値 |
| A15 | `grow_stack` は閉包をちょうど 1 回呼ぶ | `stacker` crate | A15 自身の項が `origin`、`CancelAnalysis::walk`、`RewriteCtx::rewrite`、`drop_nodes`、`rename_expr` を挙げる |
| A16 | `Match` のアームは scrutinee のタグを尽くす | lowering と、アームの列を保つ後段のパス (**検査: 無し**) | P5 (a)、P6 |
| A17 | 環境の契約 | 環境のコード (**検査: 無し**) | D21、D24 の (i-c)、P27 の言明 |
| **A18** | 生きているオブジェクトのグラフの非巡回性と、グローバル状態のオブジェクトが計数下の参照を持たないこと | **誰も** | P27 の言明と (R3) |
| A19 | bump の下に余りが在る | (ii-a) と (ii-b) は `insert_rc`・`split_rc_units`・`borrow_ify` の 3 人、(i) は同じ 3 人が P28 と A20 の上で (**検査: 無し**) | (ii-a) は P14・P18c、(ii-b) は P18a・P18c・P19・P21、(i) は P28 の項 |
| A20 | 借りた参照は活性化の間 生きている | 呼び出し元 (**検査: 無し**) | A19 (i)、P14 の (S-c)、P14a |
| A21 | 関数の値を作る演算 | `builtin.rs` の op の集合 (**検査: 無し**) | P27 (第 7 節: 「`L0` の数え上げは A21 が片付けた」)。A21 自身の項が「実行時の呼び出し先を静的な名前から決める議論」を挙げる |
| A22 | `funcs` の鍵は関数の名前 | lowering (**検査: 無し**) | A22 自身の項が `resolve_callee_params` と `call_rc` を挙げる |
| A23 | 持ち上げた lambda は closure 型である | 型検査と `uncurry` (**検査: 無し**) | `p20-borrow-ify.md` の `L18` |
| **A24** | `fix` の op は capture を持つ本体にだけ在る | **誰も** (検査は `Lowerer::lower_llvm` の panic) | `p20-borrow-ify.md` の `L18a` |
| A25 | 骨格は `Retain`/`Release` を持たない | lowering と `simplify` (検査は `RcInserter::insert_into_expr_inner` の `panic!`) | `p60-insert-rc.md` の `L8`・`L14`・`L28` |

**README 第 7 節が「誰も果たさない仮定」に挙げるのは A3・A4・A12・A18・A24 の 5 つである。**この
うち A3 は、`applies_a_function_operand` の宣言についてだけ果たす者を持つ --
`Generator::apply_lambda` の develop mode の検査である。**T の証明はこの 5 つの上に立っている。**
どれか 1 つが偽であれば、T の結論はこの文書では支えられていない。5 つのどれかが偽になったときに
T の結論が実際に破れるかどうかは別の問いであり、この文書はそれに答えない。README 第 4 節が、
A3 については人手の照合の記録
(`dev-docs/2026-06-28-unique-check-elim/audit-2026-07-20-op-declarations.md`) を、A18 (a) については
valgrind の下で走るテストを、A24 については `Lowerer::lower_llvm` の panic を、それぞれ検査として
挙げている。A12 の検査は最初の箇条 -- `Llvm` 節点の `args` の名前の列が `gen.free_vars()` に
等しいこと -- についてだけであり、develop mode の `validate` の `check_rhs` である。A4 には検査が
無い。

**README が「検査: 無し」と書き、かつ果たす者が居るのは A16・A17・A19・A20・A21・A22・A23 の
7 つである。** A18 (b) にも「検査: 無し」と書いてあるが、A18 は果たす者が居ない側に入る。A20 は
呼び出し元の側の契約であり、この文書のどの命題も、出力の呼び出し元がそれを満たすことを示していない。
A21 を支えているのは `impl LLVMGen for` の 78 個の通読 (`llvmgen-function-values.md`) だけであり、
`Generator::apply_lambda` の develop mode の表明が検査するのは `applies_a_function_operand` --
op が関数を**適用**するか -- であって、A21 が述べる「関数の値を**作る**」ではない。

**果たす者が居ると書かれていて、なお残る点が 2 つある。** README 第 7 節が挙げるものである。A23 は
果たす者を 2 人持つが、その 2 人の間を funptr 型の `Expr::Lam` が式の内側へ移らずに通ることは、まだ
誰も確かめていない。A10 は newtype を剥がす節を持ち、その節は誰も示していない。

**A19 の 3 つの節はどれも果たす者を持つ。** README 第 8 節が「その 3 節とも果たす者を持つ --
(ii-a) と (ii-b) は `insert_rc`・`split_rc_units`・`borrow_ify` の 3 人、(i) は P28 と A20 で
ある」と述べる。README 第 7 節の「A19 の果たす者」の項は、そこに 1 つ足す。

> (ii-a) は `insert_rc`・`split_rc_units`・`borrow_ify` の 3 人が果たす。(i) は同じ 3 人が P28 の
> 上で果たす。**(ii-b) は前提 (N) -- 名前は別名類を決める -- の下でしか閉じていない。**

README 第 4 節が、果たす作業の状態として次の 4 つを述べる。

- 「**(O1) と (O2) はどちらも閉じた**」。(O1) は無条件である。
- (O2) は「名前は別名類を決める」という前提 (N) の下で閉じており、その (N) について README は
  「`p60-insert-rc.md` はそれを証明していない」と書く。
- 「**`split_rc_units` の段も閉じた**」。
- 「**(i) も閉じた**」。

**残っているのは前提 (N) である。** A19 は T の前提 H3 に入っているので、**(T1) から (T4) までの
すべては、(N) をそこに持つ。**

## 4. T が届かないところ

### 証明していない 3 つ

README 第 1 節が名前で挙げるとおり、**評価順の保存**、**FFI の副作用の列の保存**、**返り値の一致**は
証明しない。返り値の一致は `cancel` についても主張しない -- カウントが下がると
`unique_check_operand` を宣言する op が別の腕を取り、割り当てるオブジェクトの個数が変わる。この向きの
変化は言語が認めている (P26)。

### T が言うのは `optimize_rc_program` の返り値についてではない

`cancel` の後に、同じ関数の中で 3 種のパスが走る
(`CODE src/build/build_object_files.rs: optimize_rc_program`)。

- `dead_code_elim::eliminate_unreachable` -- `prog.roots` が到達しない関数とグローバルを落とす
  (`CODE src/rc_ir/dead_code_elim.rs: eliminate_unreachable`)。呼ぶのは同じ関数の中で束縛された
  閉包 `prune` であり、`cancel` の直後、`unique_check_elim::specialize` の直後、
  `locality::specialize` の直後の 3 か所に置かれている。3 か所目は `locality::specialize` と同じ
  `if !config.threaded` の中にあるので、`config.threaded` が真のビルドでは 2 か所である。
- `unique_check_elim::specialize` -- 入力の一意性で関数を複製し、それによって示せるようになった
  unique check を取り除く (`CODE src/rc_ir/unique_check_elim.rs: specialize`)。
- `locality::specialize` -- `config.threaded` が偽のときだけ走り、対象が局所であると示せる参照カウント
  操作に注釈を付け、呼び出し元が到達する入力の局所性ごとに関数を複製する
  (`CODE src/rc_ir/locality.rs: specialize`)。

**T はこの 3 つを覆わない。** T の主語 `p2` は `optimize_rc_program` の途中の値であり、コード生成へ
渡るのはこの 3 つを通した後の値である。よって読み手が T から得るのは「`borrow_ify` と `cancel` は
RC 規律を壊さない」であって、「コード生成に届くプログラムが RC 規律を満たす」ではない。後者を言うには、
この 3 つのパスについて P14・P23 に当たる命題が要る。

`borrow_ify` と `cancel` が走るのは `config.enable_borrow_optimization()` が真のときである
(`<1>1` の `<2>1`)。**T の言明はこの述語を前提に置かない。**T の主語 `p2` は第 1 節の 2 つの等式が
定める値であり、`<1>1` が読むのはこの 2 つを呼ぶ位置とそこを流れる `prog` の値である。走るかどうかを
決めるのがこの述語であり、走らないビルドでは `p2` を計算する段がそもそも無い。
この述語は「`-O max` 以上」と同値であり、それは 3 つの関数を辿って出る。
`enable_borrow_optimization` は `self.runs_from(FixOptimizationLevel::Max)` を返す
(`CODE src/configuration.rs: Configuration::enable_borrow_optimization`)。`runs_from(level)` は
`self.force_all_optimizations() || self.fix_opt_level >= level` を返す
(`CODE src/configuration.rs: Configuration::runs_from`)。`force_all_optimizations` は `false` を
返す (`CODE src/configuration.rs: Configuration::force_all_optimizations`)。よって
`enable_borrow_optimization()` は `self.fix_opt_level >= FixOptimizationLevel::Max` に等しい。
それより下の水準ではこの 2 つは走らないので、T の言うことは無い。

### (T2) が依頼者の言葉で何を言うか

依頼者の言葉は「この 2 つのパスが原因の segv と二重解放は起きない」である。(T2) が届くのは次のとおり
である。

- **届く**: `p2` のどの実行でも、解放されたオブジェクトの読み (D7 の意味の読み) は 1 度も起きず、
  どのオブジェクトも 2 回目の解放を受けない。T の前提 H1 が入力について D12 を要求し、結論 (T1) が
  出力について D12 を与えるので、**この 2 つの誤りをこの 2 つのパスが持ち込むことは無い。**
- **届く (限定つき)**: 「ちょうど 1 回解放される」。P27 の系が「正常終了する実行では、計数下の
  オブジェクトはちょうど 1 回解放される -- ただし環境が持つ参照から到達できるものを除く」を与え、
  (T2) の第 3 文はその系である。P27 は、無条件に書けない場合を 2 つ挙げ、どちらもこの限定が
  すでに除いていると述べる -- グローバル状態のオブジェクト (D26) は 1 度も解放されないがそもそも
  計数下ではなく、発散する実行と中断する実行 ((E6)) では終端の `Ret` に着かない活性化があるが、
  正常終了の限定がそれを除く。
- **届かない**: segv 一般。D7 の読みとは、オブジェクトの記憶域のうち参照カウントと状態バイトを除いた
  部分を読むことである。null ポインタの参照、配列の範囲外、`FFI_CALL` の先で起きる誤りは D11 の軸の
  上に無く、T は何も言わない。
- **漏れについて言えるのはここまで**: (R3) は「漏れるとすれば、環境が持つ参照から到達できるものだけ
  である」という形の主張であり、「漏れない」ではない。上の「ちょうど 1 回」に付く限定はこれと同じ
  ものである。

### (T3) が言うこと、言わないこと

**(T3) の量化は D30 のものである。** D30 は 2 つの実行を、環境が与える入力を同じにし、複数の制御の
流れがある場合は段の並び方を同じにして取る。共通接頭は、並んだ 2 つの段が同じ位置の節点を実行し、
同じ値を作り、同じオブジェクトを名指す限り伸びる部分である。**(T3) が述べるのは 2 つの共通接頭が
ともに覆う部分の上のことだけである。**

**(T3) は組を与件として量化する。** 「`X1` を 1 つ取れば `X0` と `X2` が定まる」とは書かない。
D30 が定めるのは組の取り方であって、片方から相手が存在することも一意であることも述べない
(`<1>9` の `<2>1`)。

**(T3) が言わないのは、共通接頭が終わる出口の向きである。** `C` が終わるのは `C01` が終わるか
`C12` が終わるかである。`C12` の側については 2 つが言える -- 出口が (X1) か (X2) であることは
D30 が言い ((X3) は `π` が `borrow_ify` のときに開く)、その 2 つがどちらも観測値の改善する向きで
あることは P26 の第 2 文が言う (`<1>9` の `<2>3`)。`C01` の側については向きを言う者が居ない。P26 が
`borrow_ify` について与えるのは、共通接頭の上で観測値が**等しい**こと (よって (X1) の出口は
起きないこと) までであり、「(X2) と (X3) の出口がどちらの向きであるかは、この文書が示さない
ものである」と書く。(X3) が開くのは、`borrow_ify` が `Retain`/`Release` の位置を動かして解放の段を
動かしうるからである (D30)。よって 2 つを合成した `C` の出口の向きは言えない。README の T の項目 3 も
「`borrow_ify` の側は (X3) を開くので出口の向きを言う者が居ない」と書き、第 9 節の「測って外した
設計」が、その半分を閉じる判定とその代償を記録している。

**接頭の中では「一意だったものが共有になる」ことは起きない。** これは `<1>9` の `<2>2` と `<2>3` の
第 1 項が与えるもので、出口の向きとは別の主張である。

**残っているのは検証である。** README 第 7 節の表は P26 を 2 行に分けて記録する。

```
| P26 (`cancel` の半分) | `p50-observation.md` | 有 | 証明済み (D29 の上で) | 未着手 |
| P26 (`borrow_ify` の半分) | `p50-observation.md` | 有 | 証明済み。第 1 文は**等号**で立つので (X1) の出口が起きない。第 2 文は `cancel` の側だけの主張になった -- `borrow_ify` は (X3) を開くため | 検証済み (指摘 37 件を反映)。**2 周目が要る** |
```

どちらの `証明` の欄も `証明済み` である。`検証` の欄は、`cancel` の半分については `未着手`、
`borrow_ify` の半分については `検証済み` で 2 周目を待っている。

### 借用版の名前が新しいことを、T は A13 から取る

`borrow_funcref` は元の名前に `#borrow` を足す (`CODE src/rc_ir/borrow.rs: borrow_funcref`)。
その名前が入力のどの名前とも異なることを述べるのは **A13 の後半**である。

> `borrow_ify` の入力に現れる**すべての名前**について -- 束縛名、直接呼び出しが名指す関数の名前、
> グローバル値を読む `RcVar` の名前、**`prog.funcs` の鍵**、**各 `RcFunc` の `name`** を含む --
> その `name` フィールドを `#` で区切った最後の断片は、文字 `b` の後に 10 進数字だけが続く形では
> なく、`borrow` でもない。後者は `borrow_funcref` が借用版の名前を `<元の名前>#borrow` として
> 作るからで ... これが無いと借用版の名前が入力の名前と衝突しうる。

(T4) がこれを読むのは 1 か所である -- 「借用版の名前は `p0.funcs` のどの鍵とも どの関数の `name`
とも異なる」を示す `<1>10` の `<2>1a` である。(T4) の残りは A13 に依らない。関数の対応は鍵ではなく
P24 の対応でつなぎ、「`p0.funcs` の各関数の `name` が `p2.funcs` の鍵である」は、挿入も
`values_mut` も鍵を取り除かないことから出る (`<1>10` の `<2>1` と `<2>3`)。

### 開発ビルドでだけ走る検査

README 第 4 節は、`develop_mode` のときだけ走る表明を「3 段目より弱い」とし、そこに **A9・A11・
A13** を挙げる。**A12** の最初の箇条 (`Llvm` 節点の `args` の名前の列) の検査、**A3** の
`applies_a_function_operand` の検査、**A10** の「最適化が作る型の再検査」も同じである。すなわち
出荷ビルドでは、この **6 つ**の仮定は果たす者だけに立つ。

このうち A3 の検査は `Generator::apply_lambda` の develop mode の表明であり、`borrow_ify` と
`cancel` の外 -- コード生成の側 -- に在る。A10 の再検査は `validate_layouts` の側にある。残る
A9・A11・A12・A13 の 4 つがこの 2 つのパスの側に在る。

`optimize_rc_program` の `validate`、`borrow_ify` の `check_clone_names_are_fresh` と
`check_ownership_is_levelled` は、`config.develop_mode` / `develop_mode` が真のときだけ走る
(`CODE src/build/build_object_files.rs: optimize_rc_program`, `CODE src/rc_ir/borrow.rs: borrow_ify`)。
第 3 節の表で A9・A11・A12 の検査として挙がっているのが `validate` の中の検査 -- A9 と A12 は
`check_rhs`、A11 は `check_expr_inner` と `check_rhs` -- であり、A13 の検査として挙がっているのが
`check_clone_names_are_fresh` である。

この 3 つの検査は、表明が成り立つときは何もせず、成り立たないときはプロセスを止める。
`check_clone_names_are_fresh` は集めた名前の集合に対する `assert!` 1 つからなり、
`check_ownership_is_levelled` は `expect` 1 つと `assert!` 1 つからなり、`validate` の中の検査は
`panic!` を呼ぶ (`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`,
`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`,
`CODE src/rc_ir/validate.rs: validate`)。**どれも自分が調べる対象を共有参照で受け取り、値を
返さない** -- `check_clone_names_are_fresh` と `validate` は `&RcProgram` を、
`check_ownership_is_levelled` は `&self` と `&RcFunc` を取る。よってプログラムを書き換えることは
しない。

よって次の 2 つが言える。表明がすべて成り立つ入力については、`borrow_ify` が組み立てる `RcProgram` は
`develop_mode` によらず同じであり、T の主語は 2 つのビルドで同じプログラムである --
`borrow_ify` が `develop_mode` を読むのは `check_clone_names_are_fresh` を呼ぶ `if` と
`check_ownership_is_levelled` を呼ぶ `if` の 2 か所だけで、ほかにこの引数を読むところは無い
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。表明が破れる入力については、開発ビルドは `borrow_ify` の
返り値を持たない -- プロセスが止まるので、T の主語がそもそも存在しない。

### 引用した命題自身の状態

T は P9・P12・P14・P14a・P22・P23・P24・P26・P27 の言明を引用する。それらが対象コミットに対してどこまで
証明され、どこまで検証されているかは README 第 7 節の表が述べる。**T が閉じることは、それらが
閉じることを意味しない。**

P23 の言明は、入力を `borrow_ify` の出力に限る。T が P23 に渡すのはまさに `borrow_ify` の出力なので
(`<1>1`)、この限定は主定理の鎖を切らない。限定の理由は P23 自身が述べており、A19 (ii-b) の量化範囲と
P14a の量化範囲がどちらも `borrow_ify` の側に付いていることである。`<1>4` がそれを `p1` について
確かめる。

P24 の言明は、出力の各関数が入力のちょうど 1 つの関数から作られる形で関数の対応を述べる。(T4) は
その対応を 2 度たどってつなぐ (`<1>10` の `<2>5`)。コードの上の読み (`<1>10` の `<2>1`・`<2>1a`・
`<2>3`・`<2>3a`) が足すのは鍵と名前についての事実だけであり、4 つの欄の等式は P24 の対応の側から
出る。

**`App` の callee についての節は P12 が持つ。** P24 の 4 番目の項は「書き換えが変えるのは
`Retain`/`Release` の節点と `App` の callee の名前だけである」と、変えてよいものを数え上げる形で
述べるので、そこへどの名前が入るかは言わない。それを言うのは P12 であり、(T4) が引くのはその 2 つの
節 -- 返る呼び出し先が元の呼び出し先と同じ関数の版であること、および入力の関数を名指す呼び出しでは
返る名前が出力の `funcs` の鍵であること -- である (`<1>10` の `<2>2a`)。`cancel` の側で名前が動かない
ことは P22 が与える (`<1>10` の `<2>3b`)。

README の D30 は `borrow_ify` か `cancel` を表す記号として `π` を使う。このファイルは D30 を引くとき、
`π` の代わりにパスの名前 (`borrow_ify` / `cancel`) を書く。
