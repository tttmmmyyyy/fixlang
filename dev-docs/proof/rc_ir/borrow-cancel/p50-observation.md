# P26 -- 一意性は悪化しない

この文書は `README.md` の P26 を扱う。立つのは `README.md` の定義 D1-D27、仮定 A1-A19、および命題 P1-P24 の
**言明**である。実行 (プログラム全体の 1 回の計算) の水準の定義は README に無いので、`p51-runs.md` の第 1 節が
提案する D22-D26 を使う。第 9 節がこれを穴として書き出す。

## 0. 結論

| 半分 | 結果 |
|---|---|
| P26 (`cancel` について) | **証明済み** (第 4 節)。共通接頭 (D-CP) の上で |
| P26 (`borrow_ify` について) | **偽。反例あり** (第 5 節・第 6 節) |
| P21 と P26 の関係 | **相互依存**。P26 の言明は P21 の前提 (路の対応) を果たさない (第 8 節) |

`borrow_ify` の側が偽である理由を 1 行で書く。`funcs_observing_uniqueness` は「観測点に到達する関数」を
**直接呼び出しのグラフ**の上の最小不動点で取る。借用版の本体にある間接呼び出し -- 局所変数への
`App` -- はそのグラフに辺を持たないので、借用版が上げた参照カウントを、間接に呼ばれたクロージャの本体の
観測点が読む。`-O none` と `-O basic` で走り `-O max` で `Debug::assert_unique` が止まるプログラムを第 6 節に
挙げる。

第 7 節は、`funcs_observing_uniqueness` が入る前のコードに対する反証を記録する。README 第 6 節の較正表が
その反証を引く。

## 1. 記法

1 つの関数の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの `TypeEnv` を `type_env` と
書く。`origin(x, π)` は `origin(vars, type_env, x, π)` の略である。

- `H(o)` はオブジェクト `o` の参照カウント (D7)。入力側の実行の量を `H`、出力側の実行の量を `H'` と書く。
- 型 `B` は `box struct { v : I64 }` とする。`B` は `is_box` が真なので `boxed_leaf_paths(B) = [[]]` であり
  `rc_units(B) = [[]]` である (D4 の規則 3、D5 の `unit_step` の `is_box` の行)。以下、`B` 型の値の path は
  すべて空列 `[]` である。

## 2. 何と何を比べるか

P26 の言明は次である。

> 入力の一意性の観測点 (D18) で観測値が真であるすべての実行について、出力の対応する観測点 (D19) の観測値も
> 真である。

観測値は参照カウントであり、参照カウントは 1 つの本体の量ではなくヒープの量なので、この言明は**実行**
(`p51-runs.md` の D24) の水準にある。「対応する観測点」は D19 が構文の上で定めるが、「対応する実行」は
README が定めていない。この節がそれを定める。

**D-CP (対応する 2 つの実行と、その共通接頭)**
`T` を `borrow_ify` か `cancel` とし、`P` をその入力、`P' = T(P)` をその出力とする。`P` の実行 `X` と `P'` の
実行 `X'` を、環境 (D22) が与える入力 -- C のエントリ点の `argc` と `argv`、`FFI_EXPORT` の引数、FFI の
呼び出しが返す値 -- を同じにし、複数の制御の流れがある場合は段の並び方を同じにして取る。

`X` の段と `X'` の段を、`T` が消した節点と入れた節点の段を除いて、節点の対応 (D19 と P22) で 1 対 1 に
並べる。並んだ 2 つの段が同じ位置の節点を実行し、その段が作る値が等しく、その段が名指すオブジェクトが
同じであるとき、対応は次の段へ伸びる。伸びる限りの部分を**共通接頭**と呼ぶ。

**共通接頭が終わる点は 2 種類しかない。**

- **(X1)** 一意性の観測点 (D18) で観測値が違う。
- **(X2)** 参照カウントを内部で読む op -- `LLVMGen::unique_check_operand` が一意性検査を宣言する op -- の
  分岐が違う。カウントが 1 のときこの op は複製を作らず、そうでないとき複製を作るので、分岐が違えば以後の
  段が名指すオブジェクトが違う
  (`CODE src/ast/inline_llvm.rs: LLVMGen::unique_check_operand`,
  `CODE src/fixstd/builtin.rs: make_array_unique_with_hole`,
  `CODE src/generator.rs: Generator::build_branch_by_is_unique`)。

ほかの節点は、同じ値と同じオブジェクトを与えられれば同じ値を作り、同じオブジェクトを名指す。2 つの実行の
オブジェクトは物として同じではないので、「同じオブジェクト」は、それを割り当てた段の対応で読む -- 対応する
2 つの段が割り当てた 2 つのオブジェクトが対応し、対応するオブジェクトを名指す leaf が対応する。生成コードが
`RcExpr` と `LLVMGen` の宣言のとおりに動くことは A3 と A4 であり、環境と FFI が同じ値を返すことは D-CP の
取り方である。

P26 は共通接頭の上の観測点についての主張として読む。共通接頭が終わった後の 2 つの実行は違う節点列を辿るので、
D19 の観測点の対応が実行の上の対応を与えない。**共通接頭がどちら向きに終わりうるかは、P26 が答えるべき事柄で
ある** -- 第 4 節の L7a と第 8 節がそれを述べる。

## 3. 観測点が読む量

## L0 (観測値は `H(o) = 1` である)

**言明**。`Llvm(gen, [a])` が `unsafe_is_unique` の演算であり、`a` の leaf `[]` が指すオブジェクトが `o` で
あるとき、その位置での観測値は、`o` の参照カウント状態が local か threaded ならば `H(o) = 1` と同値であり、
global ならば偽である。

<1>1. `Std::unsafe_is_unique` の本体は、`InlineLLVMIsUniqueFunctionBody` を 1 つ持つ `Llvm` 式である。
      `make_std_mod` は `FullName::from_strs(&[STD_NAME], "unsafe_is_unique")` に `is_unique_function()` を
      登録し、`is_unique_function` は `expr_abs(vec![var_local("x")], expr_llvm(Box::new(
      InlineLLVMIsUniqueFunctionBody { assume_local: false, var_name: FullName::local("x"),
      assume_unique: false }), ret_type, None), None)` を返す。
  BY CODE src/fixstd/stdlib.rs: make_std_mod, CODE src/fixstd/builtin.rs: is_unique_function

<1>2. `assume_unique` が偽のとき、`InlineLLVMIsUniqueFunctionBody::generate` が返す組の第 0 成分は、
      `gc.build_branch_by_is_unique(obj_ptr, assumed_state(self.assume_local))` が返す `unique_bb` から
      来る辺で `1`、`shared_bb` から来る辺で `0` となる phi である。ここで `obj_ptr` はオペランドの値である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::generate

<1>3. `build_branch_by_is_unique(obj_ptr, state)` は、`build_branch_by_refcnt_state` が返す 3 つの
      基本ブロックのそれぞれについて次のように分岐する。local のとき
      `build_is_refcnt_one(obj_ptr, false, "")` の真偽で `unique_bb` と `shared_bb` へ、threaded のとき
      `build_is_refcnt_one(obj_ptr, true, "")` の真偽で `unique_threaded_bb` (そこから `unique_bb`) と
      `shared_bb` へ、global のとき `shared_bb` へ無条件に。
  BY CODE src/generator.rs: Generator::build_branch_by_is_unique

<1>4. QED
  `<1>1` より観測点の演算は `InlineLLVMIsUniqueFunctionBody` であり、`<1>2` よりその `Bool` は
  `build_branch_by_is_unique` の 2 つの出口のどちらを通ったかで決まる。`<1>3` の 3 つの状態が
  `build_branch_by_refcnt_state` が返す全体であり、local と threaded では
  `build_is_refcnt_one` すなわちカウントが 1 であるかが選択を決め、global では `shared_bb` に決まる。
  D7 より `H(o)` はその参照カウントである。
  BY <1>1, <1>2, <1>3, D7

**観測値が振る舞いを変えること。** `Debug::assert_unique` は `let (unique, x) = x.unsafe_is_unique;
if !unique { undefined(...) }; x` であり、観測値が偽のときプログラムを止める。
`Destructor::mutate_unique_io` は `let (unique, dtor) = dtor.unsafe_is_unique;` の後、真のときは資源を
そのまま使い、偽のときは `ctor` を走らせて資源を複製する
(`CODE src/fixstd/std.fix: assert_unique`, `mutate_unique_io`)。

**コードは D18 より広い集合を観測点として扱う。** `LLVMGen::observes_uniqueness` を override するのは
`InlineLLVMIsUniqueFunctionBody` と `InlineLLVMArrayIsStorageUniqueBody` の 2 つであり、どちらも
`!assume_unique` を返す (`CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness`,
`CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::observes_uniqueness`,
`InlineLLVMArrayIsStorageUniqueBody::observes_uniqueness`)。後者は
`Std::Array::_unsafe_is_storage_unique` の本体であり (`CODE src/fixstd/stdlib.rs: make_std_mod`)、D18 は
これを名指していない。この文書は D18 の集合について証明する。コードが押さえるのは上位集合なので、押さえる
向きではこの差は効かない。第 9 節が D18 の穴として書き出す。

**`assume_unique` が真の op について P26 は自明である。** `assume_unique` を真にするのは
`unique_check_elim::specialize` であり、それが走るのは `borrow_ify` と `cancel` の後である
(`CODE src/build/build_object_files.rs: optimize_rc_program`)。`assume_unique` が真のとき
`InlineLLVMIsUniqueFunctionBody::generate` は分岐を作らず定数 `1` を返すので、観測値は入力でも出力でも
真である (`CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::generate`)。

## 4. `cancel` の半分

## L6 (`cancel` は参照カウントを上げない)

**言明**。`cancel` の入力 `P` が D12 を満たすとする。D-CP の共通接頭の各時点 `p` と各計数下オブジェクト
(D26) `O` について `H'(p, O) ≤ H(p, O)` である。

<1>1. `P'` の各本体は、`P` の対応する本体から `cancelled()` が返す `Retain`/`Release` 節点だけを取り除いた
      ものであり、他の節点の種類・変数・path・並びを変えない。
  BY P22, CODE src/rc_ir/borrow.rs: cancel

<1>2. `cancel` は各関数の `borrowed_units` を変えない。関数については `f.clone()` を作って `body` だけを
      差し替え、グローバル初期化子については `symbol` と `ty` をそのまま運ぶ。
  BY CODE src/rc_ir/borrow.rs: cancel

<1>3. よって D14 の所有と借用の割り当ては `P` と `P'` で同じであり、対応する節点は D9 の同じ消費を行い、
      D10 の同じ生成を行い、活性化の初期 `Obl` は同じである。
  BY <1>1, <1>2, D9, D10, D14

<1>4. `H` を動かす段は、`Retain`、`Release`、D10 の生成の表の各行、および D10 の消費のうち捨てる 2 行
      (boxed 容器の `Destructure`、unbox 容器の名前の付いていないフィールド) である。呼び出しの段 (E3) と
      返りの段 (E4) は `H` を変えない。
  BY D10, p51 の D24 (E2)(E3)(E4)

<1>5. 共通接頭の各時点 `p` と各計数下オブジェクト `O` について
      `H'(p, O) − H(p, O) = Σ_r d_r(O) − Σ_t b_t(O)` である。ここで `t` は `p` までに `X` が実行した
      削除される `Retain`、`r` は `p` までに `X` が実行した削除される `Release` を渡り、`b_t(O)` と
      `d_r(O)` はその節点が `X` で実際に作る / 処分する `O` への参照の個数である。
  `<1>3` と `<1>4` より、削除された節点以外のすべての段は 2 つの実行で `H` を同じだけ動かす。削除された
  節点は `X` だけが実行する。実際に作る / 処分する参照の個数が `inhabited` な leaf の分であることは P6 で
  ある。
  BY <1>1, <1>3, <1>4, P6

<1>6. 削除される `Release` は、削除される `Retain` によって分割される。
  <2>1. `walk_inner` の `RcExpr::Release` の腕は、`un_bump` が `InBracket(t)` を返したときにだけ、その
        `Release` の `NodeId` を `un_bump_releases` の `t` の項目に push する。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の RcExpr::Release の腕
  <2>2. 走査は本体の各位置をちょうど 1 回訪れる。
    BY P15
  <2>3. `<2>1` と `<2>2` より、1 つの `Release` 節点は高々 1 つの項目に入る。
    BY <2>1, <2>2
  <2>4. `cancelled()` が返す集合は、`needed_retains` に入らずかつ項目が空でない `Retain` と、その `Retain`
        の項目に入っている `Release` だけからなる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled
  <2>5. QED
    BY <2>3, <2>4

<1>6a. 以下、`ρ` を `X` の活性化が辿る実行路 (D21) とし、`t` を削除される `Retain` の 1 つとし、
      「`t` の群」を `<1>6` の分割で `t` に属する削除される `Release` の集合とする。`<1>7` から `<1>9` は
      この `ρ` と、すべての `t` についての主張である。

<1>7. `t` を含まない実行路の上には、`t` の群の `Release` は 1 つも無い。
  BY P19, <1>6a

<1>8. `t` が `ρ` の上にあるとき、`t` の群の `Release` のうち `ρ` の上にあるものが `ρ` で処分する参照の
      多重集合の総和は、`t` が `ρ` で作る参照の多重集合 `b_t` に等しい。
  P19 より、`ρ` の上で群は `t` の `outstanding` を空にする。D27 の `B(t, ρ)` は `b_t` から群の処分を引いた
  ものであり、P18b より `outstanding` が空ならば `B(t, ρ)` も空である。
  BY P19, P18b, D27, <1>6a

<1>9. `t` が `p` までに実行されたとき、`p` までに `X` が実行した `t` の群の `Release` の処分の総和は、
      各オブジェクト `O` について `b_t(O)` 以下である。
  `p` までに実行された群の `Release` は `ρ` の上の群の `Release` の部分集合であり、各 `d_r(O)` は 0 以上
  なので、部分集合についての和は `<1>8` の総和以下である。
  BY <1>8, <1>6a

<1>10. QED
  `<1>5` の右辺を `<1>6` の分割で群ごとにまとめると、群 `t` の項は
  `Σ_{p までの r ∈ 群(t)} d_r(O) − (t が p までに実行されたならば b_t(O)、さもなくば 0)` である。
  `t` が実行されていない群では `<1>7` より第 1 項が 0 なので項は 0 である。`t` が実行された群では `<1>9`
  より項は 0 以下である。総和も 0 以下である。
  BY <1>5, <1>6, <1>7, <1>9

## L7 (P26 は `cancel` について成り立つ)

**言明**。`cancel` の入力 `P` が D12 を満たすとする。D-CP の共通接頭の上の一意性の観測点 (D18) `q` に
ついて、`X` の `q` での観測値が真ならば、`X'` の `q` での観測値も真である。

<1>1. `q` のオペランドの leaf が指すオブジェクトは `X` と `X'` で同じ `O` である。
  D-CP より、共通接頭の対応する段は同じオブジェクトを名指す。
  BY D-CP

<1>2. `O` の参照カウント状態は `X` と `X'` で同じである。
  <2>1. 状態を global にするのは (E5) の段だけであり、印を付けるのはグローバル初期化子の活性化が返した値が
        到達するグラフである。
    BY p51 の D24 (E5), A8
  <2>2. `cancel` はグローバル初期化子の `init` から `Retain`/`Release` 節点を取り除くだけで、他の節点の
        種類・変数・path・並びを変えない。よって `init` の活性化が返す値は `X` と `X'` で同じであり、
        その値が到達するオブジェクトのグラフも同じである。
    BY P22, D-CP, CODE src/rc_ir/borrow.rs: cancel
  <2>3. QED
    BY <2>1, <2>2

<1>3. `X` の `q` での観測値が真なので、`O` の状態は local か threaded であり、`H(q, O) = 1` である。
  BY L0, <1>1

<1>4. `H'(q, O) ≤ 1` である。
  BY L6, <1>3

<1>5. `H'(q, O) ≥ 1` である。
  `q` は D7 の読む構文の表の `Llvm` の行であり、`O` は `q` のオペランドの inhabited な leaf が指す
  オブジェクトである。P23 より `P'` は D12 を満たすので、(S-c) より `O` は `q` の時点で解放されていない。
  D7 より、解放されるのはカウントが 0 になったオブジェクトである。
  BY P23, D7, D11, <1>1

<1>6. QED
  `<1>4` と `<1>5` より `H'(q, O) = 1` である。`<1>2` と `<1>3` より `O` の状態は `X'` でも local か
  threaded なので、L0 より `X'` の `q` での観測値は真である。
  BY L0, <1>2, <1>3, <1>4, <1>5

## L7a (共通接頭の 2 つの出口は、どちらも許された向きである)

**言明**。`cancel` について、D-CP の共通接頭が終わるとき、(X1) では `X` の観測値が偽で `X'` の観測値が真で
ある。(X2) では `X` が複製を作り `X'` が作らない。

<1>1. (X1) について。観測値が違い、かつ `X` の観測値が真であるとすると、L7 より `X'` の観測値も真になり、
      違うことに反する。よって `X` の観測値は偽であり、違うのだから `X'` の観測値は真である。
  BY L7

<1>2. (X2) について。分岐が違う点を `p`、その op が参照カウントを読むオブジェクトを `O` とする。この op の
      分岐はカウントが 1 であるかで決まり、1 のとき複製を作らない
      (`CODE src/generator.rs: Generator::build_branch_by_is_unique`,
      `CODE src/fixstd/builtin.rs: make_array_unique_with_hole`)。L6 より
      `H'(p, O) ≤ H(p, O)` なので、分岐が違うのは `H(p, O) > 1` かつ `H'(p, O) = 1` のときだけである。
  BY L6, CODE src/generator.rs: Generator::build_branch_by_is_unique,
     CODE src/fixstd/builtin.rs: make_array_unique_with_hole

<1>3. QED
  BY <1>1, <1>2

`unsafe_is_unique` の doc が認めているのは共有から一意への向きである
(`CODE src/fixstd/std.fix: unsafe_is_unique` の doc)。`<1>1` と `<1>2` の出口はどちらもその向きである。

## 5. `borrow_ify` の半分

### 5.1 3 つの条件と、修正がどれを断つか

P26 が破れる形は、次の 3 つが揃うことである。第 6 節の反例と第 7 節の反例はどちらもこの形である。

1. ある関数 `f` のパラメータ leaf `x` が、`f` の本体のどこでも D9 の意味で消費されない。`infer_ownership` は
   これを借用に倒す (`Release` は消費ではないので、`x` を捨てるだけの本体は借用に倒れる)。
2. `f` の本体の、入力での `Release(x)` より後ろの位置から到達される一意性の観測点があり、その観測点の
   オペランドが実行時に `x` と同じオブジェクトを指す。第 7 節の反例では観測点が `f` の本体そのものにあり、
   第 6 節の反例では `f` が間接に呼ぶクロージャの本体にある。
3. `f` を呼ぶ側で `routing_saves_retain` が真になる。これは引数についての `any` なので、条件 1 と 2 に
   関わらない別の引数が単独で満たしてよい。

`funcs_observing_uniqueness` が断つのは**条件 2 だけ**である。条件 1 の推論 (`infer_ownership`、
`level_ownership`) にも、条件 3 の判定 (`routing_is_safe`、`routing_saves_retain`) にも、この関数は触れない
(`CODE src/rc_ir/borrow.rs: borrow_ify` -- `observing.contains(&func.name)` を見るのは
`borrow_versions` を作る `for` だけである)。

**版を止めるのが十分条件になりうる理由**は、差の出どころが条件 2 の細部に依らないことである。L8 が示すとおり
`borrow_ify` が参照カウントを変えうる場所は 3 つしかなく、その 3 つはどれも借用版に結び付いている。よって
借用版を作らなければ、どの `Release` が落ちたか、どの引数がどのオブジェクトと別名になっているかを問わずに、
その関数の実行の間の計数は入力と一致する。条件 2 を「観測点がその本体の中にあるか」で判定しようとすると
別名の解析が要るのに対し、「その版を作らない」は解析を要らない。

**十分でないのは、断ち方が狭いからである。** 差が観測されうる範囲は借用版の活性化の**動的な広がり**
(その活性化から呼び出しの列で到達される活性化の全体) であり、`funcs_observing_uniqueness` が閉じるのは
**直接呼び出しのグラフ**だけである。L10 がその差を示す。

## L8 (`borrow_ify` が参照カウントを変えうる場所は 3 つである)

**言明**。`borrow_ify` の入力の実行と出力の実行を D-CP で比べるとき、`H` の動きが違う節点は次の 3 つだけで
ある。

- **(a)** 借用版の `rewrite_rc` が落とした `Retain` / `Release`。
- **(b)** 借用版の本体に `call_rc` が入れた `before` の `Retain`。
- **(c)** 呼び出し先が借用版である呼び出しの直後に `call_rc` が入れた `after` の `Release`。

<1>1. 出力の関数は、入力の各関数 `f` について `f_own` (`f.clone()` の `body` を `is_borrow_version` が偽の
      `RewriteCtx::rewrite` に掛けたもの) と、`borrow_versions` が像を持つ `f` について `f#borrow`
      (`clone_func` の複製の `body` を `is_borrow_version` が真の `RewriteCtx::rewrite` に掛けたもの) で
      ある。グローバル初期化子は `is_borrow_version` が偽の `RewriteCtx` で書き換えられる。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>2. `rewrite_inner` が節点に行うのは次の 3 つだけである。(i) `Let(x, App(callee, args), k)` で callee を
      `route` の答えに替え、`call_rc` の `before` を呼び出しの前に、`after` を継続の先頭に置く。
      (ii) `Retain` / `Release` を `rewrite_rc` に掛ける。(iii) 残る節点は種類・変数・path・並びを変えずに
      継続を書き換える。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner

<1>3. (i) の付け替えと `clone_func` の名前替えは、本体が計算する値と作るオブジェクトを変えない。`route` が
      返す呼び出し先は元の呼び出し先と同じ入力関数の版であり (P12)、借用版の本体は原本の本体の束縛変数を
      一斉に付け替えたものである (P9)。
  BY P9, P12

<1>4. `rewrite_rc` が節点を落とすのは `is_borrow_version` が真のときだけである。偽のときは
      `rc_node(is_release, v.clone(), path.clone(), state, k, source)` をそのまま返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>5. `f_own` とグローバル初期化子の `RewriteCtx` では、`owns_unit(arg, unit)` はつねに真である。
  <2>1. `owns_unit(arg, unit)` は `origin(arg, unit)` の候補すべてについて `owns_object(root, path)` を
        問う。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit
  <2>2. `owns_object(root, path)` は `self.vars.param_tys` に `root` が無いとき真を返す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
  <2>3. グローバル初期化子の `RewriteCtx` の `vars` は `VarTable::body_only(&g.init)` であり、`param_tys`
        は空である。よって `<2>2` の腕がつねに取られる。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/ownership.rs: VarTable::body_only
  <2>4. `f_own` の `RewriteCtx` の `vars` は `VarTable::of(&f_own)` であり、`param_tys` は `f_own` の
        パラメータと capture だけを持つ。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, CODE src/rc_ir/ownership.rs: VarTable::of
  <2>5. `borrow_ify` は入力の各関数について `owned_units.extend(param_capture_units(func, type_env))` を
        行う。`f_own` は入力の関数のパラメータ名と型をそのまま持つので、`f_own` のパラメータ・capture の
        各 `p` と各 `u ∈ rc_units(p.ty)` について `(p.name, u) ∈ owned_units` である。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units
  <2>6. `root` が `f_own` のパラメータ・capture のとき、`owns_object(root, path)` は
        `units_under(root_ty, path, type_env)` の各 `unit` について
        `owned_units.contains(&(root, truncate_to_unit(root_ty, unit, type_env)))` を問う。D5 より
        `truncate_to_unit(root_ty, ・)` の値は `rc_units(root_ty)` の元であり、`<2>5` よりそれらはすべて
        `owned_units` にある。
    BY D5, <2>5, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>6

<1>6. `call_rc` の `before` に要素が入るのは `callee_owns && !arg_owned` のときであり、`arg_owned` は
      `owns_unit(arg, unit)` である。`<1>5` より、`before` に要素が入るのは借用版の本体だけである。
  BY <1>5, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>7. `call_rc` の `after` に要素が入るのは `!callee_owns && arg_owned` のときである。`callee_owns` は、
      `callee_params` に呼び出し先が無いとき真であり、あるときは
      `owned_units.contains(&(params[arg_idx].0, unit))` である。`borrow_ify` は入力の各関数について
      `owned_units.extend(param_capture_units(func, type_env))` を行い、`callee_params` には原本について
      `param_names_and_types(func)` すなわち入力の関数のパラメータ名をそのまま入れるので、呼び出し先が
      原本 `g_own` のとき `callee_owns` は真である。よって `callee_owns` が偽になるのは呼び出し先が
      借用版のときだけである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc, borrow_ify, param_capture_units,
     param_names_and_types

<1>8. QED
  `<1>2` の (iii) の節点は `H` を同じだけ動かす。(i) のうち呼び出し先の付け替えは `<1>3` より `H` を
  動かさず、入れた節点は `<1>6` の (b) と `<1>7` の (c) である。(ii) は `<1>4` より借用版の中でだけ
  節点を落とし、それが (a) である。
  BY <1>2, <1>3, <1>4, <1>6, <1>7

## L9 (差の出る観測点は借用版の動的な広がりの中にある) -- 認める段

**言明**。D-CP の共通接頭の観測点 `q` と計数下オブジェクト `O` について `H'(q, O) > H(q, O)` ならば、`q` を
実行する活性化は、ある `f#borrow` (`borrow_versions` の像) の本体の活性化であるか、その活性化から呼び出しの
段 (E3) の列で到達される活性化である。

**この段はこの文書では証明しない。** L8 の 3 か所はどれも借用版に結び付いており、(a) の窓は落とされた
`Release` の位置から呼び出し元の `after` の `Release` まで、(b) の窓は `before` の `Retain` から呼び出し先が
その参照を処分するまでであって、いずれも呼び出しの入れ子の中にある -- という形の帰納がこの段の内容である。

次の L10 が偽なので、L9 を認めても `borrow_ify` の側は閉じない。L9 を認めることは主張を強めるので、L10 の
反証はいっそう強い。

## L10 (借用版の動的な広がりに一意性の観測点は現れない) -- **偽**

**言明** (`funcs_observing_uniqueness` が果たそうとしているもの)。`f` が `borrow_versions` の鍵であるとき、
`f#borrow` の活性化から呼び出しの段 (E3) の列で到達される活性化の本体に、一意性の観測点は無い。

<1>1. `funcs_observing_uniqueness` が返す集合 `observing` は、次の 2 つの規則の最小不動点である。
      (i) `prog.funcs` の関数 `g` の本体に `observes_uniqueness()` が真の `Llvm` の op があれば
      `g ∈ observing`。(ii) `g` の本体の `Let(_, App(callee, _), _)` について
      `FuncRef { name: callee.name }` が `observing` の元であれば `g ∈ observing`。
  BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness

<1>2. `borrow_ify` は `observing` の元に借用版を作らない。`for func in prog.funcs.values()` の先頭で
      `if observing.contains(&func.name) { continue; }` が回る。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>3. CASE 呼び出しの列がすべて直接呼び出し (呼び出し先の名前が `prog.funcs` の鍵) である場合。この場合
      L10 は成り立つ。
  <2>1. `f#borrow` の本体は `f` の本体の名前替えであり (P9)、`route` が返す呼び出し先は元の呼び出し先と
        同じ入力関数の版である (P12)。よって出力の直接呼び出しの辺は、入力の直接呼び出しの辺と同じ入力関数
        の対を結ぶ。
    BY P9, P12
  <2>2. 到達される活性化の本体に観測点があるとすると、その本体の入力関数 `g` は `<1>1` の (i) により
        `observing` の元である。`<2>1` より `f` から `g` へ入力の直接呼び出しの辺の列があるので、`<1>1` の
        (ii) を辺の個数について繰り返して `f ∈ observing` である。
    BY <1>1, <2>1
  <2>3. QED
    `<2>2` は `<1>2` に反するので、到達される活性化の本体に観測点は無い。
    BY <1>2, <2>2

<1>4. CASE 呼び出しの列に間接呼び出し (呼び出し先が局所変数) がある場合。この場合 L10 は**偽**である。
  <2>1. `funcs_observing_uniqueness` は `RcRhs::App(callee, _)` について
        `FuncRef { name: callee.name.clone() }` を辺に積む。呼び出し先が局所変数 -- パラメータ、`Let` の
        束縛、`Match` の束縛 -- であるとき、その名前は `prog.funcs` の鍵ではないので、`observing` の元でも
        `callees` の鍵でもない。よってこの辺は `<1>1` の (ii) を発火させない。
    BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness
  <2>2. `RcRhs::Closure(..)` の腕は辺を積まない。よって、クロージャを作る関数は、そのクロージャの本体が
        観測点を持っても `observing` に入らない。
    BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness
  <2>3. QED (反証)
    第 6 節のプログラムがこの形である。`relay` の本体は `observes_uniqueness()` が真の op を持たず、その
    `App` の呼び出し先は自分自身と、パラメータ `cl` である。`<2>1` より `relay ∉ observing` であり、
    `<1>2` の門を通って `relay#borrow` が作られる。`relay#borrow` の `arm0` の `App(cl, ...)` が呼ぶ
    クロージャの本体に観測点があり、その位置で `H' = 2`、`H = 1` である。
    BY <2>1, <2>2, 第 6 節

<1>5. QED
  `<1>3` と `<1>4` は呼び出しの列に間接呼び出しがあるかどうかで尽きている。`<1>4` が反例を与えるので
  L10 は偽である。
  BY <1>3, <1>4

## 6. 欠陥の報告 -- 借用版が間接呼び出しを通して観測を倒す

### 6.1 入力

```fix
module Main;

type B = box struct { v : I64 };

// 再帰があるので呼び出し元へインライン化されない。
// `x` と `w` は本体で消費されないので、借用推論はこの 2 つを借用に倒す。
// 観測点へは間接呼び出し `cl()` だけが届く。
relay : I64 -> B -> B -> (() -> Bool) -> Bool;
relay = |n, x, w, cl| (
    if n > 0 { relay(n - 1, x, w, cl) };
    cl()
);

main : IO ();
main = (
    let args = *get_args;
    let o = B { v : 1 };
    let w = B { v : 2 };
    let f1 = |_| (
        let (u, o) = o.unsafe_is_unique;      // <- 観測点。`o` は capture から読む
        eval o;
        u
    );
    let f2 : () -> Bool = |_| false;
    // 実行時の値で選ぶので、クロージャ特殊化が `cl()` を直接呼び出しに変えられない。
    let cl = if args.@size > 100 { f2 } else { f1 };
    let u = relay(0, o, w, cl);
    eval w.@v;                                 // `w` を呼び出しの後で使う
    println("unique = " + u.to_string)
);
```

3 つの成分がそれぞれ役目を持つ。

- **`x`**: `relay` の本体で消費されないので借用に倒れ、その `Release` が借用版で落ちる。実行時には
  `f1` の capture が指すオブジェクトと同じ `o` である。
- **`w`**: `main` が呼び出しの後で使うので `routing_saves_retain` が真になる。`w` を落とすと振り分けが
  起きない。
- **`if args.@size > 100`**: 呼び出し先のクロージャを実行時に決めるので、間接呼び出しが RC IR まで残る。
  この分岐を落として `f1` を直に渡すと、クロージャ特殊化が `relay` を `f1` について特殊化し、観測点が
  `relay` の本体に入るので `relay ∈ observing` となり、欠陥は現れない。

### 6.2 経路

`--emit-rc-ir Main` が書く `rc_ir.Main.pre.txt` (`insert_rc` の直後) は次を含む。名前は短くしてある。

```
fn relay(n, x, w, cl) -> Bool:
  ...
  case 0(n <= 0):
      release x
      release w
      let s : () = struct_make()
      let a : Bool = cl(s)          // <- 間接呼び出し
      ret a

fn main(...):
  let o : B = struct_make(1)
  let w : B = struct_make(2)
  retain o
  let f1 : CapList = struct_make(o)        // capture が o を持つ
  ...
  retain w
  let u : Bool = relay(0, o, w, cl)
  let t : I64 = struct_get_0(w)
  release w
  ...

fn main::closure_lam0::closure#0(param), cap cap -> Bool:
  let c : CapList = capture_project_0(cap)
  release_nonnull cap
  destructure c { .0 -> o : B }
  let v : (Bool, B) = is_unique(o)          // <- 観測点
  destructure v { .0 -> u : Bool, .1 -> o2 : B }
  eval o2
  release o2
  ret u
```

`rc_ir.Main.post.txt` は `relay#borrow` を含み、そのパラメータは
`(#v0 : I64 {u}, #v1 : B {borrow}, #v2 : B {borrow}, #v3 : () -> Bool {(u, own)})` と注釈され、その
`case 0` は `release` を 1 つも持たずに `cl(s)` を呼ぶ。`main` は `relay#borrow(...)` を呼び、その直後に
`release o` を持つ。

入力の実行での `o` の参照カウントは、観測点で 1 である。`main` は `retain o` で 2 にしてから 1 つを `f1` の
capture へ渡し、もう 1 つを `relay` の `x` へ渡す (`relay` は入力では全パラメータを所有する)。`relay` の
`release x` が 1 つを処分し、capture の 1 つだけが残る。

出力の実行では 2 である。`relay#borrow` は `x` を借用するので `release x` を持たず、`main` はその参照を
呼び出しの後まで持つ。`main` の `release o` は呼び出しの直後にあるが、観測点はそれより前にある。

### 6.3 誤った出力

| 最適化水準 | `unique = ` |
|---|---|
| `-O none` | `true` |
| `-O basic` | `true` |
| `-O max` | `false` |

`unsafe_is_unique` を `Debug::assert_unique` に置き換えた同じ形のプログラム -- `f1` の本体を
`let o = o.assert_unique(|_| "o is shared"); eval o.@v; true` にしたもの -- は、`-O none` と `-O basic` で
`done = true` を印字し、`-O max` では次を出して止まる。

```
Value is not unique: o is shared
error: Program terminated by signal 6
```

`unsafe_is_unique` の doc が認めているのは共有から一意への向きだけなので、この向きは認められていない。

### 6.4 この入力は D11 を破らない

出力の実行で `o` の参照は、`main` が 1 つ (呼び出しの後の `release o` が処分する)、`f1` の capture が
1 つ (クロージャの本体の `release o2` が処分する) である。過剰処分 (S-a) も漏れ (S-b) も解放後の読み (S-c) も
起きない。すなわちこれは、D11 を満たしたまま P26 だけを破る入力である。

### 6.5 修正の候補

どれを採るかは言語の契約に関わる判断なので、候補と代償を並べる。

**候補 A -- 到達可能性に間接呼び出しの辺を入れる。** `funcs_observing_uniqueness` が `RcRhs::App` で
呼び出し先が `prog.funcs` の鍵でないとき、その呼び出し元を `observing` に入れる。すなわち「間接呼び出しを
持つ関数はすべて観測に到達しうる」とみなす。代償は、間接呼び出しを持つ関数がすべて借用版を失うことであり、
反復子とクロージャを使う経路の広い範囲がこれに当たる。

**候補 B -- クロージャの本体を辺で結ぶ。** `RcRhs::Closure(f, caps)` の腕で `f` への辺を積み、さらに
`VarTable::closure_targets` が追える範囲で間接呼び出しの呼び出し先を解決する。追えない呼び出しは候補 A の
扱いにする。代償は、`funcs_observing_uniqueness` が `VarTable` を作る必要が出ること
(現在この関数は `type_env` すら使っていない) と、追えない呼び出しが残る限り候補 A の代償を部分的に払うこと。

**候補 C -- 観測点を持つ関数を、呼び出しグラフでなくオブジェクトの側で押さえる。** 借用版が落とす
`Release` の対象になりうるオブジェクトを、観測点のオペランドが指しうるオブジェクトから遠ざける。これは
別名の解析であり、`origin` は関数の内側しか見ないので、関数をまたぐ解析が要る。

**候補 D -- 契約を広げる。** `unsafe_is_unique` の doc を、一意から共有への変化も認めるように書き換え、
P26 を落とす。代償は `Debug::assert_unique` が最適化水準によって止まることと、
`Destructor::mutate_unique_io` が一意な資源を複製することを言語が認める点である。

## 7. 記録 -- `funcs_observing_uniqueness` が入る前のコードに対する反証

この節は、`LLVMGen::observes_uniqueness` と `funcs_observing_uniqueness` が入る前のコード (issue #551) に
対する反証である。**その欠陥は直っている** -- L2 の `<1>1` (`borrow_versions` が `observe` を含むこと) が、
今のコードでは `borrow_ify` の `if observing.contains(&func.name) { continue; }` によって偽になる。
README 第 6 節の較正表が、P26 の節を較正するバグとしてこの反証を引く。

### 7.1 入力

```fix
module Main;

type B = box struct { v : I64 };

observe : I64 -> B -> B -> B -> Bool;
observe = |n, x, y, w| (
    if n > 0 { observe(n - 1, x, y, w) };
    let (u, y) = y.unsafe_is_unique;
    eval y;
    u
);

main : IO ();
main = (
    let o = B { v : 1 };
    let w = B { v : 2 };
    let u = observe(0, o, o, w);   // 同じオブジェクトを第 1・第 2 引数に渡す
    eval w.@v;                     // `w` を呼び出しの後で使う
    println("unique = " + u.to_string)
);
```

観測点は `observe` の本体にあり、`observe` から直接呼び出しで到達される。よって今のコードでは
`observe ∈ observing` となり、借用版が作られない。`-O none`、`-O basic`、`-O max` のいずれでも
`unique = true` を印字する。修正前のコードでは `-O max` だけが `unique = false` を印字した。

### 7.2 入力の RC IR

`split_rc_units` の出力、すなわち `borrow_ify` の入力にあたる 2 つの本体を書く。名前は短くしてある。

```
fn main(...):
  Let(o, Llvm(struct_make, [c1]),            // o : B
  Let(w, Llvm(struct_make, [c2]),            // w : B
  Let(n, Llvm(int, []),                      // n : I64 = 0
  Retain(w, [])
  Retain(o, [])
  Let(u, App(observe, [n, o, o, w]),
  Let(t, Llvm(struct_get_0, [w]),            // struct_get_0 は容器を借用する
  Release(w, [])
  Eval(t, ... println ...))))))

fn observe(n, x, y, w) -> Bool:
  Let(c, Llvm(int_lt, [zero, n]),
  Let(r, Match(c, [arm1, arm0]),
  Ret(r)))

  arm1 (n > 0):
    Let(n2, Llvm(int_sub, [n, one]),
    Let(a, App(observe, [n2, x, y, w]),
    Ret(a)))

  arm0 (n <= 0):
    Release(x, [])
    Release(w, [])
    Let(p, Llvm(is_unique, [y]),             // <- 観測点
    Destructure(p, [(0, u), (1, y2)],
    Eval(y2,
    Release(y2, [])
    Ret(u))))
```

`Llvm(struct_get_0, [w])` の `borrows_operand(0)` は、取り出すフィールドの型が fully unboxed のとき真で
ある。`B.@v : I64` は fully unboxed なので真であり、この演算は `w` を消費しない
(`CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand`)。

### 7.3 修正前のコードがこの入力に何を出力したか

以下、`observe` の第 1・第 2・第 3・第 4 パラメータをそれぞれ `n`、`x`、`y`、`w` と書く。

## L1 (推論の不動点は `y` だけを所有する)

**言明**。`infer_ownership` が返す `OwnedLeaves` は、`observe` のパラメータについては `(y, [])` だけを含み、
`(x, [])` と `(w, [])` を含まない。

<1>1. `observe` の本体が D9 の意味で消費する leaf を `collect_consumes` は次のように報告する。
  <2>1. `arm0` の `Llvm(is_unique, [y])` について、`rhs_consumes` の `RcRhs::Llvm` の腕は、
        `borrows_operand(i)` が偽で `passthrough_arg_leaves` に入らない各オペランド leaf を報告する。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>2. `InlineLLVMIsUniqueFunctionBody` は `borrows_operand` を override しないので、既定の `false` を
        返す。
    BY CODE src/fixstd/builtin.rs: `impl LLVMGen for InlineLLVMIsUniqueFunctionBody`,
       CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
  <2>3. `InlineLLVMIsUniqueFunctionBody::result_prov` は
        `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` を返す。`passthrough_arg_leaves` は
        結果の leaf の宣言が単一の `Arg(j, σ)` であるものだけを集めるので、この op については空である。
    BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov,
       CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection
  <2>4. よって `Llvm(is_unique, [y])` は `(y, [])` を報告する。
    BY <2>1, <2>2, <2>3
  <2>5. `arm0` の `Release(x, [])` と `Release(w, [])` は何も報告しない。`collect_consumes_go` の
        `RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は継続へ降りるだけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Retain(..) | RcExpr::Release(..) |
       RcExpr::Eval(..)` の腕
  <2>6. `arm0` の `Destructure(p, [(0, u), (1, y2)], ...)` は何も報告しない。`p` の型は unbox のタプル
        `(Bool, B)` であり、`destructure_consumes` は unbox 容器について、名前が付いていないフィールドの
        leaf だけを返す。両フィールドに名前が付いているので空である。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes
  <2>7. `arm0` の `Ret(u)`、`arm1` の `Ret(a)`、本体の `Ret(r)` は何も報告しない。
        `collect_consumes_go` の `RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, ...)` を呼び、
        `u`、`a`、`r` の型はいずれも `Std::Bool` で、`boxed_leaf_paths` は空である。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret` の腕, D4
  <2>7a. `Llvm(int_lt, [zero, n])` と `Llvm(int_sub, [n, one])` は何も報告しない。`rhs_consumes` の
        `RcRhs::Llvm` の腕はオペランドの `boxed_leaf_paths` を渡るが、`Std::I64` は
        `is_fully_unboxed` が真なので leaf を持たない。
    BY D4, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm` の腕
  <2>7b. `Let(r, Match(c, [arm1, arm0]), k)` の `Match` 自身は何も報告しない。`collect_consumes_go` の
        `RcRhs::Match(_, arms)` の場合は各アームの本体へ降りるだけである。
    BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcRhs::Match` の場合
  <2>8. `arm1` の `App(observe, [n2, x, y, w])` が報告するもののうち、`observe` のパラメータ leaf を
        `owned_leaves` に入れうるのは、`resolve_callee_params` で解決した各位置について
        `owns(&params[i], &leaf)` が真である引数の leaf だけである。`rhs_consumes` の `RcRhs::App` の腕は
        これに先立って `push_boxed_leaves(&callee.name, &callee.ty, ...)` も報告するが、その主語は名前
        `observe` であり、`origin_inner` の `vars.bindings.get(var)` が `None` を返す腕により
        `Exactly((observe, ・))` になる。`observe` は `vars.param_tys` に無いので `infer_ownership` は
        これについて何も挿入しない。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App` の腕, resolve_callee_params,
       origin_inner の `None` の腕, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>9. QED
    `<2>4`、`<2>5`、`<2>6`、`<2>7`、`<2>7a`、`<2>7b`、`<2>8` が `observe` の本体のすべての節点を尽くす。
    `main` の本体については `<1>2` で扱う。
    BY <2>4, <2>5, <2>6, <2>7, <2>7a, <2>7b, <2>8

<1>2. `infer_ownership` の不動点で `owned_leaves` に入る `observe` のパラメータ leaf は `(y, [])` だけで
      ある。
  <2>1. `owned_leaves` の初期値は空である。
    BY CODE src/rc_ir/borrow.rs: infer_ownership
  <2>2. 1 周目、`<1>1` の `<2>4` が `(y, [])` を報告し、`origin(y, [])` は `Exactly((y, []))` である
        (`y` は `Binding::Param`)。`y` は `vars.param_tys` にあるので `(y, [])` が挿入される。
    BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: origin_inner の
       `Binding::Param` の腕
  <2>3. `<1>1` の `<2>8` の `App(observe, [n2, x, y, w])` は、`owns` が真である位置の引数だけを報告する。
        `owns` は `owned_leaves` を読むので、`(x, [])` と `(w, [])` が `owned_leaves` に入っていない限り
        位置 1 と位置 3 は報告されない。位置 2 は `(y, [])` が入っているので `(y, [])` を報告し、これは
        すでに入っている。
    BY <1>1, <2>2, CODE src/rc_ir/borrow.rs: infer_ownership
  <2>4. `main` の本体を歩いた分は `observe` のパラメータ leaf を `owned_leaves` に入れない。
        `infer_ownership` が `owned_leaves` に入れるのは `(root_var, root_path)` のうち
        `vars.param_tys.contains_key(root_var)` を満たすものだけであり、ここでの `vars` は `main` の
        `VarTable` である。`main` のパラメータは `Std::IO::IOState` 型の 1 つだけなので、`observe` の
        パラメータ名はその `param_tys` に無い。
    BY CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: VarTable::of
  <2>5. `level_ownership` は `(x, [])` と `(w, [])` を挿入しない。`levelled_sites` が挙げる site のうち
        `x` を主語にするのは `arm0` の `Release(x, [])` と `arm1` の `App` の位置 1 であり、どちらも
        `origin(x, [])` の候補は `{(x, [])}` である。`owns_object_yet(vars, type_env, x, [], owned_leaves)`
        は、`x` が `param_tys` にあるので `units_under(B, [], type_env) = [[]]` の各 unit について
        `truncate_to_unit(B, [], type_env) = []` を鍵として `(x, []) ∈ owned_leaves` を問い、`<2>2` と
        `<2>3` よりこれは偽である。よって `owns_a_candidate` が偽になり、`level_ownership` は `false` を
        返して何も挿入しない。`w` についても同じ形の site で同じ答えになる。
    BY <2>2, <2>3, CODE src/rc_ir/borrow.rs: levelled_sites, level_ownership, owns_object_yet
  <2>6. QED
    `<2>1` から `<2>5` より、不動点に達したときの `owned_leaves` は `observe` のパラメータについて
    `(y, [])` だけを持つ。
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>3. QED
  BY <1>2

## L2 (`main` の呼び出しは借用版へ振り分けられる)

**言明**。修正前のコードでは、`borrow_ify` は `observe#borrow` を作り、`main` の
`App(observe, [n, o, o, w])` をその借用版へ振り分け、呼び出しの後に `Release(o, [])` と `Release(w, [])` を
置く。

<1>1. `borrow_versions` は `observe` を含む。`observe` は `capture` を持たず、
      `func_has_borrowable_param(observe, owned_leaves, type_env)` は、`x` の leaf `[]` が
      `owned_leaves.owns(x, [])` を偽にするので真である。
  **今のコードではこの段が偽である。** `observe` の本体は `observes_uniqueness()` が真の op を持つので
  `observe ∈ observing` であり、`borrow_ify` は `continue` でこの関数を飛ばす。
  BY L1, CODE src/rc_ir/borrow.rs: borrow_ify, func_has_borrowable_param, funcs_observing_uniqueness

<1>2. `owned_units` は、`observe` の原本について `(n, ・)` を除く全 unit すなわち `(x, [])`、`(y, [])`、
      `(w, [])` を含み、借用版 `observe#borrow` については `(y', [])` だけを含む。ここで `y'` は
      `clone_func` が `y` に付けた新しい名前である。
  `borrow_ify` は各関数について `owned_units.extend(param_capture_units(func, type_env))` を行い、
  借用版については `owned_leaves.owns(&p.name, &leaf)` が真である leaf だけを `truncate_to_unit` で
  unit にして入れる。`I64` は `rc_units` が空なので `n` の unit は無い。
  BY L1, <1>1, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

<1>3. `routing_is_safe(u, [n, o, o, w])` は真である。`main` の本体は呼び出しの後に
      `Llvm(struct_get_0, [w])` を持つので、`trivially_returns(k, u)` は `RcExpr::Let(..)` の腕で偽になり、
      `u` は `tail_result_vars(main.body)` に入らない。
  BY CODE src/rc_ir/borrow.rs: routing_is_safe, tail_result_vars, mark_tail, trivially_returns

<1>4. `routing_saves_retain(observe#borrow, [n, o, o, w], k)` は真である。
  <2>1. 引数 3 (`w`) の unit `[]` について `callee_borrows` は真である。
        `!self.owned_units.contains(&(borrow_params[3].0.clone(), []))` であり、`<1>2` より借用版の
        `owned_units` は `w` の複製名を含まない。
    BY <1>2, CODE src/rc_ir/borrow.rs: routing_saves_retain
  <2>2. `arg_used_later = used_later(&w.name, k)` は真である。`k` は呼び出しの継続であり、その中の
        `Let(t, Llvm(struct_get_0, [w]), ...)` について `used_later` の `RcExpr::Let(_, rhs, k)` の腕が
        `rhs_uses(name, rhs)` を真にする。
    BY CODE src/rc_ir/borrow.rs: used_later
  <2>3. QED
        `<2>2` より `!arg_used_later` は偽であり、`self.owns_unit(arg, unit) && !arg_used_later && ...` は
        偽になる。その否定は真なので、`<2>1` と合わせて引数 3 の unit `[]` が
        `callee_borrows && !(...)` を真にし、`any` が真を返す。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: routing_saves_retain

<1>5. `route(u, observe, [n, o, o, w], k)` は `observe#borrow` を返す。
  BY <1>1, <1>3, <1>4, CODE src/rc_ir/borrow.rs: route

<1>6. `call_rc(observe#borrow, [n, o, o, w])` は `before = []`、`after = [(o, []), (w, [])]` を返す。
  <2>1. 引数 0 (`n : I64`) は `rc_units(I64, type_env)` が空なので、どちらにも入らない。
    BY D5, CODE src/rc_ir/borrow.rs: call_rc
  <2>2. `main` はパラメータとして `o` も `w` も取らないので、`owns_object` の
        `self.vars.param_tys.get(root)` は `None` を返し、`owns_unit(o, [])` と `owns_unit(w, [])` は
        ともに真である。
    BY CODE src/rc_ir/borrow.rs: owns_unit, owns_object
  <2>3. 引数 1 (`o`、位置は `x`) について `callee_owns` は偽である (`<1>2`)。`<2>2` より `arg_owned` は
        真なので、`!callee_owns && arg_owned` の枝が取られ `after` に `(o, [])` が入る。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>4. 引数 2 (`o`、位置は `y`) について `callee_owns` は真である (`<1>2`)。`<2>2` より `arg_owned` も
        真なので、どちらの枝も取られない。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>5. 引数 3 (`w`、位置は `w`) について `callee_owns` は偽 (`<1>2`)、`arg_owned` は真 (`<2>2`) なので
        `after` に `(w, [])` が入る。
    BY <1>2, <2>2, CODE src/rc_ir/borrow.rs: call_rc
  <2>6. QED
    BY <2>1, <2>3, <2>4, <2>5

<1>7. QED
  `rewrite_inner` の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は、`route` の答えを callee にし、
  `call_rc` の `after` を `prepend_rc(after, true, ...)` で継続の先頭に置き、`before` を
  `prepend_rc(before, false, ...)` で呼び出しの前に置く。`<1>5` と `<1>6` より、`main` の出力は
  `Retain(w, []) ; Retain(o, []) ; Let(u, App(observe#borrow, [n, o, o, w]), Release(o, []) ;
  Release(w, []) ; k')` である。
  BY <1>5, <1>6, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, prepend_rc

## L3 (借用版は `Release(x, [])` を落とす)

**言明**。`observe#borrow` の本体の `arm0` には `Release(x', [])` も `Release(w', [])` も無く、
`Release(y2', [])` はある。ここで `'` は `clone_func` の付けた名前を表す。

<1>1. `observe#borrow` の本体は `observe` の本体の束縛変数を一斉に付け替えたものであり、それ以外の違いを
      持たない。
  BY P9

<1>2. `RewriteCtx::rewrite_rc` は、`is_borrow_version` が真のとき、`units_under(&v.ty, path, type_env)` の
      うち `self.owns_unit(v, unit)` が真である unit だけを節点として残す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc

<1>3. `owns_unit(x', [])` は偽である。
  `origin(x', [])` は `x'` が `Binding::Param` なので `Exactly((x', []))` を返し、候補は `{(x', [])}` で
  ある。`owns_object(x', [])` は `param_tys` に `x'` があるので
  `units_under(B, [], type_env) = [[]]` の各 unit について
  `owned_units.contains(&(x', truncate_to_unit(B, [], type_env)))` すなわち
  `owned_units.contains(&(x', []))` を問い、L2 の `<1>2` よりこれは偽である。
  BY L2, CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Param` の腕,
     CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>4. `owns_unit(w', [])` は偽である。理由は `<1>3` と同じ形で、`owned_units.contains(&(w', []))` が
      L2 の `<1>2` より偽である。
  BY L2, CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Param` の腕,
     CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>5. `owns_unit(y2', [])` は真である。
  <2>1. `y2'` は `Destructure(p', [(0, u'), (1, y2')], ...)` が束縛するので `Binding::Field(p', 1)` を
        持つ。`p'` の型 `(Std::Bool, B)` は unbox のタプルなので `is_box` は偽であり、`origin_inner` の
        `Binding::Field` の腕は `origin(p', [1])` へ帰着する。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, origin_inner の `Binding::Field` の腕
  <2>2. `origin(p', [1])` は `Exactly((p', [1]))` である。`p'` は `Binding::Llvm(is_unique, [y'],
        (Std::Bool, B))` を持つ。`InlineLLVMIsUniqueFunctionBody::result_prov` は
        `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` を返すので、
        `decl.leaf_origins_at([1])` は `{Unknown}` であり `as_arg_projection` は `None` を返す。よって
        `origin_from_leaves_under(..., path = [1], here = (p', [1]))` に入り、そこでは `[1]` の下の唯一の
        leaf の源が `Unknown` なので `operand_units` は空、`produced_here` は真、`reached` は
        `[Exactly((p', [1]))]` の 1 元になり、その 1 元がそのまま返る。
    BY CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov,
       CODE src/rc_ir/ownership.rs: origin_inner の `Binding::Llvm` の腕, as_arg_projection,
       origin_from_leaves_under
  <2>3. QED
    `<2>1` と `<2>2` より `origin(y2', [])` の候補は `{(p', [1])}` である。`p'` は `param_tys` に無いので
    `owns_object(p', [1])` は `None` の枝で真を返す。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: owns_unit, owns_object

<1>6. QED
  `<1>2` と `<1>3` より `Release(x', [])` の `kept` は空になり、節点は継続に置き換わる。`<1>4` より
  `Release(w', [])` も同じである。`<1>5` より `Release(y2', [])` は残る。`<1>1` より本体の他の部分は
  原本と同じ形である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### 7.4 2 つの実行の参照カウント

以下、`main` の 1 回の活性化を `M`、`observe` (または `observe#borrow`) の 1 回の活性化を `F` と書く。
`n = 0` なので `Match` は `arm0` を選ぶ。`o` が指すオブジェクトを `o*` と書く。

## L4 (入力の実行では観測値が真である)

**言明**。入力プログラムの実行において、観測点 `Llvm(is_unique, [y])` の位置で `H(o*) = 1` である。

<1>1. `Llvm(struct_make, [c1])` の結果の leaf `[]` は新しい参照を 1 つ作り、`H(o*)` は 1 から始まる。
      `M` の `Obl` に参照が 1 つ入る。`InlineLLVMMakeStructBody::result_prov` は、boxed な結果の根の path に
      `sole_origin(LeafOrigin::Fresh)` を置く。`B` は `is_box` が真なのでその leaf は根の path `[]` だけで
      あり (D4 の規則 3)、宣言は単一の `Arg(j, σ)` ではない。D10 の生成の表の `Llvm` の行がこれに当たり、
      A3 の表の `Fresh` の行が「新しく割り当てたオブジェクトへの新しい参照」と述べる。
  BY A3, D4, D10, CODE src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov

<1>2. `Retain(o, [])` の後、`H(o*) = 2` であり、`M` の `Obl` は `o*` への参照を 2 つ持つ。
  BY D10, <1>1

<1>3. `App(observe, [n, o, o, w])` は、A1 より `observe` が全パラメータを所有するので、位置 1 と位置 2 の
      引数 `o` の leaf `[]` を消費する。消費は `H` を変えないので `H(o*) = 2` のままであり、`M` の `Obl` は
      `o*` について空になり、`F` の初期 `Obl` は `x` の leaf と `y` の leaf の分で `o*` への参照を 2 つ持つ。
  BY A1, D9, D10, <1>2

<1>4. `arm0` の `Release(x, [])` の後、`H(o*) = 1` である。
  BY D10, <1>3

<1>4a. `o*` の参照カウント状態は global ではない。A8 より global の状態を持つのはグローバル値が到達する
      オブジェクトであり、`o*` は `main` の `Llvm(struct_make, [c1])` が実行時に作ったオブジェクトで、
      どのグローバル値からも到達されない。
  BY A8

<1>5. QED
  `<1>4` の直後の節点が観測点である。`<1>4a` と L0 より観測値は `H(o*) = 1` と同値であり、`<1>4` より
  それは真である。
  BY L0, <1>4, <1>4a

## L5 (修正前の出力の実行では観測値が偽である)

**言明**。修正前のコードの `borrow_ify` の出力の実行において、対応する観測点の位置で `H(o*) = 2` である。

<1>1. 観測点の対応。L3 の `<1>1` より `observe#borrow` の本体は `observe` の本体の名前替えであり、D19 より
      借用版の本体の観測点は複製元の本体の同じ位置の観測点に対応する。`main` の本体の他の節点は L2 の
      `<1>7` が述べる 2 つの `Release` の挿入と callee の付け替えだけを受ける。
  BY D19, L2, L3

<1>2. `Llvm(struct_make, [c1])` から `Retain(o, [])` までは入力と同じ節点であり、`H(o*) = 2` であって
      `M` の `Obl` は `o*` への参照を 2 つ持つ。
  BY L4, <1>1

<1>3. `App(observe#borrow, [n, o, o, w])` は、位置 2 の引数 `o` の leaf を消費し、位置 1 の引数 `o` の
      leaf を消費しない。L2 の `<1>2` より `owned_units` は `(y', [])` を含み `(x', [])` を含まないので、
      P13 より `observe#borrow` の `borrowed_units` は `(x', [])` を含み `(y', [])` を含まない。D14 が
      その集合で所有と借用を定め、D9 の `App` の行が消費される leaf をそれで決める。消費は `H` を変えない
      ので `H(o*) = 2` のままであり、`M` の `Obl` は `o*` への参照を 1 つ残し、`F` の初期 `Obl` は `y'` の
      分の 1 つだけを持つ (D10 の初期値は借用する unit の下の leaf を入れない)。
  BY D9, D10, D14, P13, L2, <1>2

<1>4. `arm0` に `Release(x', [])` は無いので、観測点の位置で `H(o*) = 2` である。
  BY L3, <1>3

<1>5. QED
  L4 の `<1>4a` と同じ理由で `o*` の状態は global ではない。L0 より観測値は `H(o*) = 1` と同値であり、
  `<1>4` よりそれは偽である。
  BY L0, L4, <1>4

### 7.5 この記録が P26 の節を較正すること

L4 と L5 は、修正前のコードが P26 を破ることを示す。同じ出力は D11 を満たす -- 出力の実行で `o*` の参照は、
`M` が 1 つ (呼び出しの後の `Release(o, [])` が処分する)、`F` の `y'` が 1 つ (`Release(y2', [])` が処分
する) であり、過剰処分 (S-a) も漏れ (S-b) も解放後の読み (S-c) も起きない。すなわちこれは、D11 を満たした
まま P26 だけを破る入力であり、README 第 6 節が求めた「この節だけを破る例」である。

## 8. P21 と P26 の関係

**P26 の言明は P21 の前提を果たさない。2 つは相互依存であり、その依存は閉じない。**

P21 は「対応する 2 つの活性化」を「同じ実行路を辿る 2 つの活性化」と定め、それを前提に置く。README は倒れうる
経路として D18 の観測を挙げ、「P26 がその観測が悪化しないことを述べるが、P26 は改善の向き (偽から真へ) を
許すので、路の対応そのものは別に前提される」と書く。この節はその「別に前提される」ものが何かを言う。

1. 路の対応が保たれるには、2 つの実行の観測値が**等しい**ことが要る。観測値が違えば、その値を scrutinee に
   する `Match` で選ばれるアームが違い、以後の節点列が違う。D18 の観測値は `Bool` であり、
   `Debug::assert_unique` はそれを `if` に掛け、`if` は RC IR では `Match` になる
   (`CODE src/fixstd/std.fix: assert_unique`, `CODE src/rc_ir/lower.rs: Lowerer::lower_if`)。

2. P26 が言うのは「入力が真ならば出力も真」であって、等しいことではない。L7a の `<1>1` が示すとおり、
   `cancel` では「入力が偽、出力が真」が実際に起きうる。これは `unsafe_is_unique` の doc が認めている
   向きなので、P26 を強めて等号にすることはできない。

3. よって P21 の前提は P26 から出ない。P21 の前提が破れる点は、L7a が示すとおり (X1) と (X2) の 2 つで
   あり、(X2) は P21 も P26 も名指していない -- 内部の一意性検査は D18 の観測点ではないが、参照カウントを
   読み、分岐によって作るオブジェクトを変える。

4. 依存は循環する。P21 の前提 (路の対応) は P26 の結論 (観測値の関係) を要し、P26 の証明は
   「対応する観測点」を言うために路の対応を要する。この文書は D-CP でその循環を切った -- 路の対応を
   前提に置くのでなく、**共通接頭**として定義し、P26 を共通接頭の上の主張として述べ、共通接頭が終わる
   点でどちら向きに終わるかを L7a で述べた。

**この切り方で何が残るか。** 共通接頭が終わった後について、D-CP の下でも P21 も P26 も何も言わない。それで
足りるのは、D11 が静的な実行路 (D3) すべてについての述語だからである -- (S-a)(S-b)(S-c) は 1 つの実行の
比較ではなく、出力の本体の全実行路についての条件である。**足りないのは P21 の証明の側である。** P21 は
「入力が D12 を満たす」から出力の解放を導くのに 2 つの活性化の比較を使っており、共通接頭が終わった後の
出力の実行路については、対応する入力の実行路を辿る活性化が存在するとは限らない。D21 は「1 つの実行路を辿る
活性化が存在しないこともある」と書いており、存在しない実行路について D12 は空虚に成り立つので、そこから
出力についての事実は出ない。

**オーケストレータへの報告**。P21 の言明は、前提を「同じ実行路を辿る」でなく「共通接頭の上で」と読み替えるか、
(S-c) を出力の本体について直接示す形に組み替えるかのどちらかが要る。この文書は README を書き換えないので、
判断は残す。

## 9. オーケストレータへ報告する定義と仮定の穴

1. **実行の水準の定義が README に無い。** P26 は参照カウントについての主張であり、参照カウントはヒープの量
   なので、活性化 (D21) の水準では述べられない。この文書は `p51-runs.md` の第 1 節が提案する D22-D26
   (環境・活性化・実行・参照の持ち手) を使った。これらを README へ移すかどうかは判断が要る。

2. **「対応する実行」が定義されていない。** D19 は観測点の構文の対応を定めるだけである。この文書は D-CP を
   置いた。D-CP は定義であり、README の定義ではない。

3. **D18 が `Std::Array::_unsafe_is_storage_unique` を覆っていない。** D18 は観測点を `unsafe_is_unique` の
   演算が現れる位置と定めるが、コードが観測として扱うのは `LLVMGen::observes_uniqueness` が真を返す op で
   あり、それは `InlineLLVMIsUniqueFunctionBody` と `InlineLLVMArrayIsStorageUniqueBody` の 2 つである。
   後者は `Std::Array::_unsafe_is_storage_unique` の本体である。コードが押さえる集合は D18 の集合の上位
   集合なので、押さえる向きではこの差は効かない。破る向き -- 第 6 節の反例 -- を
   `_unsafe_is_storage_unique` で書けるかどうかは調べていない。

4. **内部の一意性検査 ((X2)) をどの命題も名指していない。** `LLVMGen::unique_check_operand` が宣言する
   一意性検査は、参照カウントを読んで複製を作るかどうかを決める。カウントを変える変換はこの分岐を倒し、
   2 つの実行のオブジェクトを違える。P21 の但し書きは D18 の観測だけを数えている。

## 10. この文書が読んだコードの版

対象コミット `8e47c1b5` の作業ツリーを読んだ。第 6 節の実行に使った二値は、`src/` の内容が対象コミットと
`// PROOF:` の注釈行を除いて一致するコミット `b6170522` からビルドされたものである
(`.claude/worktrees/rc-stats/target/release/fix`、`fix 1.5.0 (b617052)`)。第 7 節の実行 (今のコードで
`unique = true` になること) にも同じ二値を使った。
