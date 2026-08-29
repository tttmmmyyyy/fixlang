# T -- パイプラインが保存するもの

対象コミット: `a8dab6e04bdc82916b6a1296e4ab186027347ef3`

このファイルは主定理 T を証明する。T は合成であり、P14・P23・P24・P26・P27 の**言明**と、
D19・D30 の定義と、`optimize_rc_program` が 2 つのパスを呼ぶ順序だけを使う。引用する命題の証明は
読まない。

第 3 節が、T が立つ 20 個の仮定を果たす者とともに 1 か所に並べる。第 4 節が、T が届かないところを
述べる。**この定理が読み手に何を与え、何を与えないかを決めるのは第 4 節である。**

(T3) は D30 (対応する 2 つの実行と、その共通接頭) の上に立つ。`<1>9` が P26 の 2 つの実例を 1 つの
`p1` の実行の上で連結する。

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
      (T3) `X1` を `p1` の 1 つの**実際の**実行とし、D30 が `borrow_ify` について `X1` と組にする
           `p0` の実際の実行を `X0`、D30 が `cancel` について `X1` と組にする `p2` の実際の実行を
           `X2` とする。`(X0, X1)` の共通接頭と `(X1, X2)` の共通接頭 (D30) がともに覆う部分の
           上で、`p0` の各一意性の観測点 (D18) `q0` の `X0` における観測値が真ならば、D19 の
           2 つの対応を合成して得る `p2` の観測点 `q2` (`<1>8`) の `X2` における観測値も真で
           ある。またその部分が終わるのは 2 つの共通接頭のいずれかの出口であり、それは (X1) か
           (X2) -- どちらも観測値が改善する向き -- である。
      (T4) `p0` から `p2` を作る 2 つのパスの合成は、D12 が見ない部分について次を保つ。

           - **`roots`**: `p2.roots` は `p0.roots` に等しい。
           - **関数**: `p2.funcs` の各エントリは `p0.funcs` のちょうど 1 つの関数から作られ
             (この「作られる」を定めるのは `<1>10` の `<2>1` と `<2>3` である)、その `fn_ty` /
             `ret_ty` / `params` の型 / `inline_into_callers` は、その元の関数のものに等しい。
             また `p0.funcs` の各鍵は `p2.funcs` の鍵である。

             **`p1` と `p2` は `p0` に無い関数を持つ。**`borrow_ify` は借用版を持つ関数について
             借用版を 1 つ足すので (`<2>1`)、その版は `p0.funcs` のどの鍵のものでもない。この
             言明が借用版について述べるのは、上の 4 つの欄を複製元の関数と共有することだけで
             ある。
           - **グローバル初期化子**: `p2.globals` は `p0.globals` と同じ長さの列であり、第 `i`
             要素の `symbol` と `ty` は `p0.globals` の第 `i` 要素のものに等しく、
             `owns_initializer` と `owns_storage` は `true` である。D1 が述べる呼び出し順に
             より、この `true` は正しい値である。

(T1) と (T4) は D12 の見る部分と見ない部分を分けて述べる。(T2) は (T1) から実行の水準へ渡る。
(T3) は D12 の軸の上に無い性質であり、(T1) からは出ない。

## 2. 証明

### <1>1. `config.enable_borrow_optimization()` が真のとき、`optimize_rc_program` が `cancel` に渡すプログラムは `p1` であり、その返り値は `p2` である

  **<2>1.** `optimize_rc_program` は、`symbol_names` と閉包 `validate`・`prune` を束縛した後、
  次をこの順で実行する。

  ```rust
  validate(&prog, "after insert_rc");
  split_rc_units(&mut prog, type_env);
  validate(&prog, "after split_rc_units");
  if config.enable_borrow_optimization() {
      prog = borrow_ify(&prog, type_env, config.develop_mode);
      validate(&prog, "after borrow_ify");
      prog = cancel(&prog, type_env);
      validate(&prog, "after cancel");
      // -- この後に走る 3 種のパスは第 4 節が述べる
  }
  prog
  ```

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`

  **<2>2.** `optimize_rc_program` の中の `validate` の呼び出しは、**どれも**束縛 `prog` の値を
  変えない。`validate` は同じ関数の中で束縛された次の閉包である。

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
  変えない。この言明は `<2>1` が引用した呼び出しをすべて覆う --
  `"after insert_rc"`、`"after split_rc_units"`、`"after borrow_ify"`、`"after cancel"` の
  4 つである。

    BY `CODE src/build/build_object_files.rs: optimize_rc_program`,
       `CODE src/rc_ir/validate.rs: validate`

  **<2>3. QED**

  H0 より、`split_rc_units(&mut prog, type_env)` の直後の束縛 `prog` の値が `p0` である。
  次の文 `validate(&prog, "after split_rc_units")` は `prog` を変えない (`<2>2`)。よって
  `if` の本体に入る時点の `prog` は `p0` である。

  本体の 1 行目 `prog = borrow_ify(&prog, type_env, config.develop_mode)` は、第 1 引数に `p0` を
  渡す。T の `develop_mode` は自由変数 (`NEW develop_mode : bool`) なので、これを
  `config.develop_mode` に取れば、この行の後の `prog` は第 1 節の `p1` である。

  2 行目 `validate(&prog, "after borrow_ify")` は `prog` を変えない (`<2>2`)。よって 3 行目
  `prog = cancel(&prog, type_env)` が受け取るのは `p1` であり、その返り値は第 1 節の `p2` で
  ある。

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

P23 の言明は入力を **`borrow_ify` の出力**に限る。`<1>1` より `p1` は `borrow_ify` の返り値で
あり、かつ `cancel` が受け取るプログラムである。よって `p1` はこの条件を満たす。

P23 がその限定の理由に挙げる 2 つも、同じ理由で `p1` について読める。A19 の範囲は
「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の入力) の
両方」であり、`p1` は後者である。P14a の範囲は「`borrow_ify` の出力の各本体」であり、`p1` は
それである。

    BY <1>1, A19, P14a

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

P27 の言明の前提は、プログラムのすべての本体が D11 を満たすこと、および A17 と A18 が成り立つこと
である。`<1>6` が第 1 の前提を `p2` について与え、H3 が A17 と A18 を与える。P27 の結論が
(R1)(R2)(R3) であり、(T2) の 3 つの主張 -- 解放されたオブジェクトの読みが起きないこと、どの
オブジェクトも高々 1 回しか解放されないこと、正常終了する実行で残る計数下のオブジェクトが環境の
持つ参照から到達できるものに限ること -- はその 3 つである。

    BY <1>6, H3 (A17, A18), P27

### <1>8. `p2` の各一意性の観測点に、`p0` の観測点がちょうど 1 つ定まる

D19 は、`borrow_ify` と `cancel` のそれぞれについて、出力の各一意性の観測点が入力のちょうど 1 つの
観測点から来ることを言う。`<1>1` より `p1` は `borrow_ify` の出力であり、かつ `cancel` の入力で
ある。

D19 を `cancel` (入力 `p1`、出力 `p2`) に当てると、`p2` の各観測点 `q2` に `p1` の観測点 `q1` が
ちょうど 1 つ定まる。その `q1` に D19 を `borrow_ify` (入力 `p0`、出力 `p1`) に当てると、`p0` の
観測点 `q0` がちょうど 1 つ定まる。この 2 段を続けて適用すれば、`q2` から `q0` が 1 つ定まる。
(T3) の `q0` はこれである。

    BY <1>1, D19

### <1>9. (T3)

D30 は 1 つのパスについての定義であり (入力 1 つと出力 1 つを持つ)、P26 はその上に立つ。この鎖は
2 つを 2 度使う -- 1 度目は `borrow_ify` について (入力 `p0`、出力 `p1`)、2 度目は `cancel` に
ついて (入力 `p1`、出力 `p2`) である。README 第 7 節の表はこの 2 つの実例を別の行に記録している。

```
| P26 (`cancel` の半分) | `p50-observation.md` | 有 | 証明済み (D29 の上で) | 未着手 |
| P26 (`borrow_ify` の半分) | `p50-observation.md` | 有 | 証明済み。場合分けは D24 の段の種と `apply_lambda` の呼び出し位置の 2 通りから導いた | 未着手 |
```

どちらの `証明` の欄も `証明済み` であり、`未着手` なのは `検証` の欄である。

  **<2>1.** `X1` を `p1` の 1 つの実際の実行とすると、D30 の 2 つの実例が `X1` と組になる `X0` と
  `X2` を与える。**2 つの実例が名指す `p1` の実行は同じ `X1` である。**

  D30 が 2 つの実行を取る仕方は、環境 (D22) が与える入力 -- C のエントリ点の `argc` と `argv`、
  `FFI_EXPORT` の引数、FFI の呼び出しが返す値 -- を同じにし、複数の制御の流れがある場合は段の
  並び方を同じにすることである。`X1` の側を先に固定し、その入力と並び方を `X0` と `X2` の側に
  課せば、2 つの組はどちらも `X1` を含む。

  - D30 を `borrow_ify` (D30 の `T` を `borrow_ify`、`P` を `p0`、`P'` を `p1` と読む) に当てて、
    `X1` と組になる `p0` の実際の実行を `X0` とする。この組の共通接頭を `C01` と書く。
  - D30 を `cancel` (D30 の `T` を `cancel`、`P` を `p1`、`P'` を `p2` と読む) に当てて、`X1` と
    組になる `p2` の実際の実行を `X2` とする。この組の共通接頭を `C12` と書く。

  `<1>1` より `p1` は `borrow_ify` の出力であり、かつ `cancel` の入力であるから、この 2 通りの
  読み方はどちらも D30 の要求する形である。

    BY <1>1, D30

  **<2>2.** P26 を `borrow_ify` の実例に当てる。`C01` の上の各一意性の観測点 (D18) について、
  `X0` の観測値が真ならば、D19 が対応させる `p1` の観測点の `X1` における観測値も真である。
  さらに `C01` が終わる出口は (X1) か (X2) であり、(X1) では `X0` の観測値が偽で `X1` の観測値が
  真、(X2) では `X0` が複製を作り `X1` が作らない。

    BY <2>1, P26

  **<2>3.** P26 を `cancel` の実例に当てる。`C12` の上の各一意性の観測点について、`X1` の観測値が
  真ならば、D19 が対応させる `p2` の観測点の `X2` における観測値も真である。さらに `C12` が
  終わる出口は (X1) か (X2) であり、(X1) では `X1` の観測値が偽で `X2` の観測値が真、(X2) では
  `X1` が複製を作り `X2` が作らない。

    BY <2>1, P26

  **<2>4.** `C01` と `C12` はどちらも `X1` の段の列の接頭である。よって一方が他方を含み、短い方を
  `C` と書けば、`C` の各段は `C01` の段でも `C12` の段でもある。`C` は `X1` の接頭であり、`C` が
  終わるのは `C01` が終わるか `C12` が終わるかのどちらか (または両方) である。

  D30 は共通接頭を「対応が伸びる限りの部分」と定めるので、`C01` も `C12` も `X1` の先頭から
  途切れずに続く部分である。1 つの列の 2 つの接頭は、短い方が長い方に含まれる。

    BY <2>1, D30

  **<2>5. QED**

  `q0` を `p0` の一意性の観測点、`q1` を D19 が `q0` に対応させる `p1` の観測点、`q2` を D19 が
  `q1` に対応させる `p2` の観測点とする。`<1>8` より `q2` から `q0` はこの 2 段で定まり、これが
  (T3) の `q0` と `q2` である。

  `C` は `X1` の接頭なので、`C` の 1 つの段を取り、それが `q1` の訪問であるとする。`<2>4` より
  その段は `C01` にも `C12` にも在るので、`C01` の対応が `X0` の側の段を、`C12` の対応が `X2` の
  側の段を、それぞれこの段に与える。D19 よりその 2 つは `q0` の訪問と `q2` の訪問である。

  `X0` のその段で `q0` の観測値が真であるとすると、その段は `C01` に在るので `<2>2` より `X1` の
  その段で `q1` の観測値が真である。同じ段は `C12` にも在るので、`<2>3` より `X2` のその段で
  `q2` の観測値が真である。含意を 2 つつないだこの導出が、(T3) の前半である。

  (T3) の後半 -- `C` が終わるのは 2 つの共通接頭のいずれかの出口であり、それが (X1) か (X2) で
  ある -- は、`<2>4` (どちらかが終わる) と `<2>2`・`<2>3` (その出口が (X1) か (X2) である) の
  合わせである。

  **この段が言うのは 2 つの実例のそれぞれの出口についてであり、`X0` と `X2` の対についての
  出口を 1 つの言明にまとめることではない。** T の項目 3 も「P26 を `borrow_ify` と `cancel` に
  順に適用し、D19 の 2 つの対応を合成する」とその形を指定している。

    BY <1>8, <2>2, <2>3, <2>4

### <1>10. (T4)

  **<2>1.** `borrow_ify` の下で `p0` と `p1` の実体を次で同定する。`borrow_ify` が返す
  `RcProgram` の 3 つの欄は、それぞれ次のように組み立てられる。

  - **`funcs`**: 空の `Map` から始め、2 つのループだけが要素を入れる。第 1 のループは
    `prog.funcs.values()` の各 `func` について `f_own = func.clone()` を作り、`f_own.body` だけを
    `ctx.rewrite(&f_own.body)` に差し替えて `funcs.insert(f_own.name.clone(), f_own)` を行う。
    第 2 のループは、借用版を持つ `func` について `clone_func(func, borrow_funcref(&func.name), ..)`
    が作る `clone` の `body` を差し替えて `funcs.insert(borrow_version, clone)` を行う。
    `clone_func` は `name: new_ref` を置くので、`clone.name` は鍵 `borrow_version` に等しい。

    `p1.funcs` の各エントリは、この 2 種の挿入のうち最後にその鍵へ書いたものである。第 1 の
    ループの挿入が残っているならそのエントリは `func.clone()` から作られており、第 2 のループの
    挿入が残っているなら `clone_func(func, ..)` から作られている。どちらの場合も `p0.funcs` の
    ちょうど 1 つの関数 `func` から作られており、これをそのエントリの**元**と呼ぶ。また `Map` への
    挿入は鍵を取り除かないので、`p0.funcs` の各鍵は `p1.funcs` の鍵である。さらに 2 つのループの
    どちらも、
    エントリの `name` に等しい鍵で挿入する。すなわち **`p1.funcs` のどのエントリも、鍵はその
    関数の `name` に等しい。**これは `RcProgram` が `funcs` について述べる不変条件
    (「The top-level functions, keyed by the name each is defined under」) を `p1` について
    確かめたものである。
  - **`globals`**: `prog.globals.iter().map(|g| RcGlobalInit { symbol: g.symbol.clone(),
    ty: g.ty.clone(), init: ctx.rewrite(&g.init), owns_initializer: true, owns_storage: true })`
    を `collect` した列である。よって `p1.globals` は `p0.globals` と同じ長さであり、第 `i`
    要素の元は `p0.globals` の第 `i` 要素である。`symbol` と `ty` はそのまま運ばれる。
  - **`roots`**: `roots: prog.roots.clone()`。よって `p1.roots` は `p0.roots` に等しい。

    BY `CODE src/rc_ir/borrow.rs: borrow_ify`, `CODE src/rc_ir/borrow.rs: clone_func`,
       `CODE src/rc_ir/borrow.rs: borrow_funcref`, `CODE src/rc_ir/ast.rs: RcProgram`

  **<2>2.** P24 は、`borrow_ify` と `cancel` のそれぞれについて、次の 3 つを述べる。`roots` を
  変えない。**出力の各関数は入力のちょうど 1 つの関数から作られ**、その `fn_ty` / `ret_ty` /
  `params` の型 / `inline_into_callers` は元の関数のものに等しい。出力のグローバル初期化子の列は
  入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の第 `i` 要素のものに等しい。

  **P24 は関数の対応を鍵の一致では述べない。**「出力の各関数は入力のちょうど 1 つの関数から
  作られ」という形であり、その理由も P24 自身が挙げている -- `borrow_ify` は入力に無い関数
  (借用版) を足すからである。`<2>1` と `<2>3` は、この「作られる」を 2 つのパスのそれぞれに
  ついてコードの上で定める。

    BY P24

  **<2>3.** `cancel` の下で `p1` と `p2` の関数を同定する。`cancel` は `prog.funcs.values()` の
  各 `f` について `let mut clone = f.clone();` を作り、`clone.body` だけを `cancel_body` の結果に
  差し替えて `(f.name.clone(), clone)` を収める。

  よって `p2.funcs` の各エントリの**元**は `p1.funcs` のちょうど 1 つのエントリであり、元と
  `body` 以外のすべての欄を共有する。鍵の集合については、`p2.funcs` の鍵は
  `{ f.name : f ∈ p1.funcs.values() }` である。`<2>1` より `p1.funcs` のどのエントリも鍵は
  その `name` に等しいので、この集合は `p1.funcs` の鍵の集合に等しい。`<2>1` はさらに
  `p0.funcs` の各鍵が `p1.funcs` の鍵であると言うので、`p0.funcs` の各鍵は `p2.funcs` の鍵で
  ある。

    BY <2>1, `CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/ast.rs: RcProgram`

  **<2>3a.** `cancel` の下で `p1` と `p2` のグローバル初期化子を同定し、`roots` を突き合わせる。
  `cancel` は `prog.globals.iter().map(|g| RcGlobalInit { symbol: g.symbol.clone(),
  ty: g.ty.clone(), init: cancel_body(&vars, &g.init), owns_initializer: true,
  owns_storage: true })` を `collect` した列を返す。よって `p2.globals` は `p1.globals` と同じ
  長さであり、第 `i` 要素の元は `p1.globals` の第 `i` 要素である。`symbol` と `ty` はそのまま
  運ばれる。`roots` は `roots: prog.roots.clone()` なので `p2.roots` は `p1.roots` に等しい。

    BY `CODE src/rc_ir/borrow.rs: cancel`

  **<2>4.** P24 は、`borrow_ify` と `cancel` のそれぞれについて、各グローバル初期化子の
  `owns_initializer` と `owns_storage` に `true` を書くこと、および D1 が述べる呼び出し順により
  この書き込みが正しい値を書くことを述べる。2 つのパスを続けて適用すると、この 2 つの欄には
  2 回とも `true` が書かれ、最後に書かれた値も `true` である。

    BY P24, D1

  **<2>5. QED**

  **P24 の「元」と `<2>1`・`<2>3` の「元」は同じものである。**P24 が「出力の各関数は入力のちょうど
  1 つの関数から作られ」と言う関係を、`borrow_ify` については `<2>1` が、`cancel` については
  `<2>3` が、コードの上で名指す。以下の 3 つの計算はその関係でつなぐ。

  **`roots` について。**`roots` はプログラム自身の欄なので、実体の同定は要らない。

      p2.roots
        = p1.roots        [BY <2>3a]
        = p0.roots        [BY <2>1]

  `=` と `=` は等号の推移で `=` に閉じる。

  **関数について。**`g2` を `p2.funcs` の任意のエントリとする。`<2>3` より `g2` の元 `g1` が
  `p1.funcs` にちょうど 1 つあり、`<2>1` より `g1` の元 `f` が `p0.funcs` にちょうど 1 つある。
  `φ` を `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` のいずれかとすると

      φ(g2)
        = φ(g1)           [BY <2>3, <2>2 (`cancel` について)]
        = φ(f)            [BY <2>1, <2>2 (`borrow_ify` について)]

  であり、等号の推移で `φ(g2) = φ(f)` に閉じる。これが (T4) の関数についての第 1 の主張である。
  第 2 の主張 -- `p0.funcs` の各鍵が `p2.funcs` の鍵であること -- は `<2>3` が述べる。借用版に
  ついて言えるのが上の等式だけであることは、`<2>1` が借用版の元を複製元の `func` と定めることから
  従う。

  **グローバル初期化子について。**`<2>1` と `<2>3a` より、3 つの列は同じ長さであり、`p2.globals`
  の第 `i` 要素の元は `p1.globals` の第 `i` 要素、その元は `p0.globals` の第 `i` 要素である。
  `ψ` を `symbol` か `ty` とすると

      ψ(p2.globals[i])
        = ψ(p1.globals[i])   [BY <2>3a]
        = ψ(p0.globals[i])   [BY <2>1]

  であり、等号の推移で閉じる。`owns_initializer` と `owns_storage` が `true` であること、および
  その `true` が正しい値であることは `<2>4` が述べる。

    BY <2>1, <2>2, <2>3, <2>3a, <2>4

### <1>11. QED

(T1) は `<1>5`、(T2) は `<1>7`、(T3) は `<1>9`、(T4) は `<1>10` である。

    BY <1>5, <1>7, <1>9, <1>10

## 3. T が立つ仮定と、それを果たす者

T は、引用する命題が証明されている仮定の集合の上に立つ。README 第 4 節の仮定は 20 個あり、その全部が
ここに載る。「果たす者」は README 第 4 節の言葉である。右の欄は、README がその仮定を名指して結び
付けているものを書き、README がどこでも名指していない仮定には `--` を置く。

| 仮定 | 内容 | 果たす者 (README 第 4 節) | README がこの仮定に結び付けているもの |
|---|---|---|---|
| A1 | 入力が RC 規律を満たす | 前段のパス (`insert_rc`) | P14 の言明 |
| A2 | 単位への正規化 | `insert_rc` と `split_rc_units` | P14 の言明 |
| **A3** | 宣言されたモデルの忠実さ | **誰も**。ただし `applies_a_function_operand` については `Generator::apply_lambda` の develop mode の検査が果たす | D10、D21、D24、D29、D30、P5 (a) |
| **A4** | コード生成の忠実さ | **誰も** | D30 |
| A5 | 型が leaf の上位近似 | `leaf_map.rs` の設計 | D25、P7a |
| A6 | 名前の一意性 | lowering | P9 (出力側は P9 が示す) |
| A7 | 呼び出し先の解決 | `resolve_callee_params` の設計 | `--` |
| A8 | グローバルは線形規律の外 | `mark_global` | D26、(E7) |
| A9 | `Match` はアームを持つ | lowering (検査は develop mode の `validate`) | P16 |
| A10 | 型の well-formedness | `validate_layouts` (最適化が作る型の再検査は develop build だけ) | `boxed_leaf_paths` と `rc_units` の停止性 |
| A11 | スコープの規律 | lowering (検査は develop mode の `validate`) | `origin` の停止性 |
| **A12** | 束縛の形と型が合っている | **誰も** (検査するコードは無い) | `held_field_type`、`rhs_consumes` の停止性 |
| A13 | 名前の形 | `Lowerer::fresh_var` と `clone_fresh` (検査は develop mode の `check_clone_names_are_fresh`) | P9 の後半 |
| A14 | 過適用が無い | 型検査と lowering | `call_rc`、`rhs_consumes` |
| A15 | `grow_stack` は閉包をちょうど 1 回呼ぶ | `stacker` crate | `origin`、`CancelAnalysis::walk`、`RewriteCtx::rewrite`、`drop_nodes`、`rename_expr` |
| A16 | `Match` のアームは scrutinee のタグを尽くす | lowering と、アームの列を保つ後段のパス (**検査: 無し**) | P5 (a)、P6 |
| A17 | 環境の契約 | 環境のコード (**検査: 無し**) | P27 の言明 |
| **A18** | 生きているオブジェクトのグラフの非巡回性と、グローバル状態のオブジェクトが計数下の参照を持たないこと | **誰も** | P27 の (R3) |
| A19 | bump の下に余りが在る | `insert_rc` と `borrow_ify` (**検査: 無し**。**`insert_rc` の側は開いている**) | (ii-a) は P14、(ii-b) は P18a・P19・P21 |
| A20 | 借りた参照は活性化の間 生きている | 呼び出し元 (**検査: 無し**) | P14 の (S-c)、P14a |

**README 第 7 節が「誰も果たさない仮定」に挙げるのは A3・A4・A12・A18 の 4 つである。**このうち
A3 は、`applies_a_function_operand` の宣言についてだけ果たす者を持つ -- `Generator::apply_lambda`
の develop mode の検査である。**T の証明はこの 4 つの上に立っている。**どれか 1 つが偽であれば、
T の結論はこの文書では支えられていない。4 つのどれかが偽になったときに T の結論が実際に破れるか
どうかは別の問いであり、この文書はそれに答えない。README 第 4 節が、A3 については人手の照合の記録
(`dev-docs/2026-06-28-unique-check-elim/audit-2026-07-20-op-declarations.md`) を、A18 (a) については
valgrind の下で走るテストを、それぞれ検査として挙げている。A4 と A12 には検査が無い。

**果たす者が居て検査の無い仮定は A16・A17・A19・A20 の 4 つである。**このうち A19 は果たす者の
半分が開いており、次の段落が述べる。A20 は呼び出し元の側の契約であり、この文書のどの命題も、出力の
呼び出し元がそれを満たすことを示していない。

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
(`<1>1` の `<2>1`)。この述語は「`-O max` 以上」と同値であり、それは 3 つの関数を辿って出る。
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
- **届かない**: segv 一般。D7 の読みとは、オブジェクトの記憶域のうち参照カウントと状態バイトを除いた
  部分を読むことである。null ポインタの参照、配列の範囲外、`FFI_CALL` の先で起きる誤りは D11 の軸の
  上に無く、T は何も言わない。
- **届かない**: 「ちょうど 1 回解放される」。P27 の言明が反例を 3 つ挙げる -- グローバル状態の
  オブジェクトは 1 度も解放されない、発散する実行と中断する実行では終端の `Ret` に着かない活性化が
  残る、正常終了する実行でも C のエントリ点は `run_ios_runner` が作る `IOState` を処分しない。
  (T2) が言えるのは**高々 1 回**までである。
- **漏れについて言えるのはここまで**: (R3) は「漏れるとすれば、環境が持つ参照から到達できるものだけ
  である」という形の主張であり、「漏れない」ではない。

### (T3) が言うこと、言わないこと

**(T3) の量化は D30 のものである。** D30 は 2 つの実行を、環境が与える入力を同じにし、複数の制御の
流れがある場合は段の並び方を同じにして取る。共通接頭は、並んだ 2 つの段が同じ位置の節点を実行し、
同じ値を作り、同じオブジェクトを名指す限り伸びる部分である。**(T3) が述べるのはその共通接頭の上の
ことだけである。**

**共通接頭に限っても「一意だったものが共有になる」ことは起きない。** 接頭の中では観測値が悪化せず
(`<2>2`、`<2>3`)、接頭が終わるのは (X1) か (X2) -- どちらも観測値が改善する向き -- だからである
(P26)。この読み方は P26 の言明が自分で述べている。

**(T3) が言わないのは、`X0` と `X2` の対についての 1 つの出口の言明である。** `<1>9` の `<2>5` が
述べるのは、`C` が終わるのが `C01` の出口か `C12` の出口であり、そのそれぞれが許された向きである
ことである。T の項目 3 が求めているのもこの形である。

**残っているのは検証である。** README 第 7 節の表の `検証` の欄は、`cancel` の半分の行も
`borrow_ify` の半分の行も `未着手` である (`<1>9` が両方の行を引用する)。どちらの `証明` の欄も
`証明済み` なので、これは検証の側の残りである。

### 借用版の名前が新しいことに、T は依らない

`borrow_funcref` は元の名前に `#borrow` を足すだけであり (`CODE src/rc_ir/borrow.rs: borrow_funcref`)、
その名前が入力のどの関数名とも異なることを述べるものは README に無い -- A6 は束縛名と関数名の
食い違いを、A13 は `#b` に 10 進数字が続く形を、P9 の後半は複製が導入する束縛名を述べる。

**T はそれを要らない。** (T4) は関数を鍵で対応させず、`<1>10` の `<2>1` と `<2>3` が定める「元」
-- そのエントリを作った入力の関数 -- で対応させる。`Map` への挿入が鍵を取り除かないことから
「`p0.funcs` の各鍵は `p2.funcs` の鍵である」も出るので、名前が衝突しないことを使う段が 1 つも
無い。読み手はこの仮定を探さなくてよい。

### 開発ビルドでだけ走る検査

`optimize_rc_program` の `validate`、`borrow_ify` の `check_clone_names_are_fresh` と
`check_ownership_is_levelled` は、`config.develop_mode` / `develop_mode` が真のときだけ走る
(`CODE src/build/build_object_files.rs: optimize_rc_program`, `CODE src/rc_ir/borrow.rs: borrow_ify`)。
第 3 節の表で A9 と A11 の検査として挙がっているのが `validate` の中の検査であり、A13 の検査として
挙がっているのが `check_clone_names_are_fresh` である。A10 についても、最適化が作る型を再検査するのは
develop build だけである (README 第 4 節の A10)。**出荷ビルドでは、この 4 つの仮定は果たす者だけに
立つ。**

この 3 つの検査は、表明が成り立つときは何もせず、成り立たないときはプロセスを止める。
`check_clone_names_are_fresh` は集めた名前の集合に対する `assert!` 1 つからなり、
`check_ownership_is_levelled` は `expect` 1 つと `assert!` 1 つからなり、`validate` の中の検査は
`panic!` を呼ぶ (`CODE src/rc_ir/borrow.rs: check_clone_names_are_fresh`,
`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`,
`CODE src/rc_ir/validate.rs: validate`)。どれもプログラムを共有参照で受け取って値を返さないので、
プログラムを書き換えることはしない。

よって次の 2 つが言える。表明がすべて成り立つ入力については、`borrow_ify` が組み立てる `RcProgram` は
`develop_mode` によらず同じであり、T の主語は 2 つのビルドで同じプログラムである --
`borrow_ify` が `develop_mode` を読むのは `check_clone_names_are_fresh` を呼ぶ `if` と
`check_ownership_is_levelled` を呼ぶ `if` の 2 か所だけで、ほかにこの引数を読むところは無い
(`CODE src/rc_ir/borrow.rs: borrow_ify`)。表明が破れる入力については、開発ビルドは `borrow_ify` の
返り値を持たない -- プロセスが止まるので、T の主語がそもそも存在しない。

### 引用した命題自身の状態

T は P14・P23・P24・P26・P27 の言明を引用する。それらが対象コミットに対してどこまで証明され、どこまで
検証されているかは README 第 7 節の表が述べる。**T が閉じることは、それらが閉じることを意味しない。**

P23 の言明は、入力を `borrow_ify` の出力に限る。T が P23 に渡すのはまさに `borrow_ify` の出力なので
(`<1>1`)、この限定は主定理の鎖を切らない。限定の理由は P23 自身が述べており、A19 (ii-b) の範囲と
P14a の範囲がどちらも `borrow_ify` の側に付いていることである。`<1>4` がそれを `p1` について
確かめる。

P24 の言明は、出力の各関数が入力のちょうど 1 つの関数から作られる形で関数の対応を述べる。その
「作られる」を 2 つのパスのそれぞれについてコードの上で名指すのが `<1>10` の `<2>1` と `<2>3` で
あり、`<2>5` がその関係で 3 つのプログラムをつなぐ。

D30 の `T` と主定理の `T` は別のものである。README の D30 は `borrow_ify` か `cancel` を表す
記号として `T` を使い、主定理の名前も `T` である。このファイルは D30 を引くとき、`T` の代わりに
パスの名前 (`borrow_ify` / `cancel`) を書く。
