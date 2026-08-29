# P26 -- 一意性は悪化しない

この文書は `README.md` の P26 を扱う。立つのは `README.md` の定義と仮定、および命題 P1-P25 の**言明**で
ある。実行の水準の定義 -- 環境 D22、活性化 D23、実行 D24、参照の持ち手 D25、対応する活性化 D29、実行の
終わり方 D31、実行の読み D32 -- は README の第 3.6 節にある。

## 0. 結論

| 半分 | 結果 |
|---|---|
| P26 (`cancel` について) | **証明済み** (第 4 節)。共通接頭 (D-CP) の上で |
| P26 (`borrow_ify` について) | **偽。第 4 の反例** (L10、第 6 節) |
| P21 と P26 の関係 | **相互依存。P26 の言明だけでは閉じない** (第 10 節) |

`borrow_ify` の側について、この文書が閉じたものと閉じないものを分けて書く。

- **閉じた (L8a、L8b、L8、L9)**。`borrow_ify` が参照カウントを変えうる場所は 3 つしかなく、その 3 つは
  どれも「`route` が借用版へ回した呼び出しの窓」の中にある。よって**関数ごとに借用版を作らない門は、
  正しい梃子である** -- 借用版を作らなければ、別名の解析を一切せずに、その関数の実行の間の計数は入力と
  一致する。L9 は門の広さを読まないので、門をどう広げてもこの段は動かない。
- **閉じない (L10)**。門が数える到達可能性が、窓の中で走りうる本体を尽くしていない。制御が 1 つの本体の
  新しい活性化に届く仕組みは 5 つあり (L10 の `<1>3`)、`funcs_observing_uniqueness` が辺を張るのは
  そのうち 2 つ -- callee の名前が `prog.funcs` の鍵である `App` と、鍵でない `App` -- だけである。
  残る 3 つのうち、**`Llvm` の op がその生成コードで関数の型のオペランドを適用する**場合が門を素通り
  する。`Std::Option` の `mod_some` を 1 つ書けば、`-O none` と `-O basic` で `true`、`-O max` で
  `false` を印字するプログラムになる。第 6 節がその反例である。あとの 2 つのうち、林の根を作る段は
  1 つの制御の流れでは起きず、グローバルのアクセサが初期化子の活性化を作る段 ((E7)) は、この文書が
  決めていない (L10 の `<1>7`、第 11 節)。

すなわち**広げた門はまだ必要条件どまりであり、十分ではない**。

第 7 節・第 8 節・第 9 節は、直った 3 つの反例を記録する。3 つは 1 つの列をなす -- 門が無かったとき、
門が直接呼び出しだけを閉じていたとき、門がクロージャを `prog.funcs` の本体からだけ集めていたとき。
第 6 節の反例はその 4 つ目であり、直っていない。README 第 6 節の較正表が第 7 節の反例を引く。

## 1. 記法

1 つの関数の本体を固定し、その本体から作られる `VarTable` を `vars`、プログラムの `TypeEnv` を `type_env` と
書く。`origin(x, π)` は `origin(vars, type_env, x, π)` の略である。

- `H(o)` はオブジェクト `o` の参照カウント (D7)。入力側の実行の量を `H`、出力側の実行の量を `H'` と書く。
- 型 `B` は `box struct { v : I64 }` とする。`B` は `is_box` が真なので `boxed_leaf_paths(B) = [[]]` であり
  `rc_units(B) = [[]]` である (D4 の規則 3、D5 の `unit_step` の `is_box` の行)。以下、`B` 型の値の path は
  すべて空列 `[]` である。

## 2. 何と何を比べるか

P26 の言明は次である。

> 入力の一意性の観測点 (D18) で観測値が真であるすべての**実際の**実行について、出力の対応する観測点 (D19)
> の観測値も真である。「実際の実行」とは、観測点が返す値がその時点の参照カウントと一致する活性化の木の
> ことである (D21)。

観測値は参照カウントであり、参照カウントは 1 つの本体の量ではなくヒープの量なので、この言明は**実行**
(D24) の水準にある。「対応する観測点」は D19 が構文の上で定め、「対応する活性化」は D29 が定める。この節は
その対応を**段の列**の上で読み直し、どこまで伸びるかを言う量 -- 共通接頭 -- を置く。

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
ある** -- 第 4 節の L7a と第 10 節がそれを述べる。

**D-CP と、README の D29・P25 の関係。** README の D29 は 1 つの本体の活性化どうしの対応を定め、P25 は
出力の実行の活性化の木と入力の活性化の木が同型であって、対応する `App` が同じ値を返し各オブジェクトの
参照カウントを同じだけ動かすことを述べる。D-CP は同じ対応を段の列の上で定め、対応が伸びる限りの部分を
共通接頭と呼ぶ。第 4 節は D-CP の上に立っている。第 11 節の項目 5 がこれを報告する。

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
向きではこの差は効かない。第 11 節が D18 の穴として書き出す。

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
  BY D10, D24 (E2)(E3)(E4)

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
    BY D24 (E5), A8
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

### 5.1 3 つの条件と、門がどれを断つか

P26 が破れる形は、次の 3 つが揃うことである。第 6 節から第 9 節の 4 つの反例は、どれもこの形である。

1. ある関数 `f` のパラメータ leaf `x` が、`f` の本体のどこでも D9 の意味で消費されない。`infer_ownership` は
   これを借用に倒す (`Release` は消費ではないので、`x` を捨てるだけの本体は借用に倒れる)。
2. `f` の本体の、入力での `Release(x)` より後ろの位置から到達される一意性の観測点があり、その観測点の
   オペランドが実行時に `x` と同じオブジェクトを指す。
3. `f` を呼ぶ側で `routing_saves_retain` が真になる。これは引数についての `any` なので、条件 1 と 2 に
   関わらない別の引数が単独で満たしてよい。

`funcs_observing_uniqueness` の門が断つのは**条件 2 だけ**である。条件 1 の推論 (`infer_ownership`、
`level_ownership`) にも、条件 3 の判定 (`routing_is_safe`、`routing_saves_retain`) にも、この関数は触れない
(`CODE src/rc_ir/borrow.rs: borrow_ify` -- `observing.contains(&func.name)` を見るのは `borrow_versions` を
作る `for` だけである)。

**門の広さが動かすのは L10 であって L9 ではない。** L9 は「差が出るのは借用版の呼び出しの窓の中だけである」
と述べ、その証明は `rewrite_rc` と `call_rc` だけを読み、どの関数に借用版を作るかを読まない。L10 は「その窓の
中に観測点は現れない」と述べ、これが門の広さを読む。L9 が**関数ごとの門が正しい梃子であること**を与え、
L10 が**その梃子が実際に届いているか**を問う。

**関数ごとの門は十分になりうる。** L9 より、借用版を作らなければ、どの `Release` が落ちたか、どの引数が
どのオブジェクトと別名になっているかを問わず、その関数の実行の間の計数は入力と一致する。条件 2 を
「観測点がその本体の中にあるか」で判定すると別名の解析が要るのに対し、「その版を作らない」は解析を要らない。

**広げた門はまだ十分でない。** 門は `App` の callee だけを辺にする。`Llvm` の op については
`observes_uniqueness()` だけを問い、その op が生成コードでオペランドの関数を適用するかどうかを問わない。
`Std::Option::mod_some` のような `mod_{変位}` はその形の op であり、門はその先にある観測点へ届かない。
第 6 節がその反例である。

## L8a (原本の版とグローバル初期化子では `owns_unit` はつねに真である)

**言明**。`f_own` の `RewriteCtx` とグローバル初期化子の `RewriteCtx` では、どの `(arg, unit)` についても
`owns_unit(arg, unit)` は真である。

<1>1. `owns_unit(arg, unit)` は `origin(arg, unit)` の候補すべてについて `owns_object(root, path)` を問う。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit

<1>2. `owns_object(root, path)` は `self.vars.param_tys` に `root` が無いとき真を返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>3. グローバル初期化子の `RewriteCtx` の `vars` は `VarTable::body_only(&g.init)` であり、`param_tys` は
      空である。よって `<1>2` の腕がつねに取られる。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/ownership.rs: VarTable::body_only

<1>4. `f_own` の `RewriteCtx` の `vars` は `VarTable::of(&f_own)` であり、`param_tys` は `f_own` の
      パラメータと capture だけを持つ。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::new, CODE src/rc_ir/ownership.rs: VarTable::of

<1>5. `borrow_ify` は入力の各関数について `owned_units.extend(param_capture_units(func, type_env))` を
      行う。`f_own` は入力の関数のパラメータ名と型をそのまま持つので、`f_own` のパラメータ・capture の
      各 `p` と各 `u ∈ rc_units(p.ty)` について `(p.name, u) ∈ owned_units` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

<1>6. `root` が `f_own` のパラメータ・capture のとき、`owns_object(root, path)` は
      `units_under(root_ty, path, type_env)` の各 `unit` について
      `owned_units.contains(&(root, truncate_to_unit(root_ty, unit, type_env)))` を問う。D5 より
      `truncate_to_unit(root_ty, ・)` の値は `rc_units(root_ty)` の元であり、`<1>5` よりそれらはすべて
      `owned_units` にある。
  BY D5, <1>5, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>6

## L8b (原本の版とグローバル初期化子は借用しない)

**言明**。出力の各 `f_own` の `borrowed_units` は空である。出力の各グローバル初期化子はパラメータも capture も
持たない。よって `f_own` の活性化とグローバル初期化子の活性化の D10 の初期値は、入力の対応する本体の活性化の
初期値に一致する。

<1>1. `borrow_ify` は最後に各出力関数について
      `func.borrowed_units = param_capture_units(func, type_env).filter(|u| !owned_units.contains(u))` を
      置く。`f_own` については L8a の `<1>5` が挙げる unit がすべて `owned_units` にあるので、この集合は
      空である。
  BY L8a, CODE src/rc_ir/borrow.rs: borrow_ify, param_capture_units

<1>2. グローバル初期化子 `RcGlobalInit` はパラメータも capture も持たない。
  BY D1

<1>3. QED
  D10 の初期値は、所有する (D14) パラメータ・capture の unit の下の inhabited な各 leaf につき 1 つで
  ある。`<1>1` と `<1>2` より出力の側の所有は全 unit であり、A1 より入力の側の所有も全 unit である。
  BY A1, D10, D14, <1>1, <1>2

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

<1>5. `call_rc` の `before` に要素が入るのは `callee_owns && !arg_owned` のときであり、`arg_owned` は
      `owns_unit(arg, unit)` である。L8a より、`before` に要素が入るのは借用版の本体だけである。
  BY L8a, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc

<1>6. `call_rc` の `after` に要素が入るのは `!callee_owns && arg_owned` のときである。`callee_owns` は、
      `callee_params` に呼び出し先が無いとき真であり、あるときは
      `owned_units.contains(&(params[arg_idx].0, unit))` である。`borrow_ify` は入力の各関数について
      `owned_units.extend(param_capture_units(func, type_env))` を行い、`callee_params` には原本について
      `param_names_and_types(func)` すなわち入力の関数のパラメータ名をそのまま入れるので、呼び出し先が
      原本 `g_own` のとき `callee_owns` は真である。よって `callee_owns` が偽になるのは呼び出し先が
      借用版のときだけである。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc, borrow_ify, param_capture_units,
     param_names_and_types

<1>7. QED
  `<1>2` の (iii) の節点は `H` を同じだけ動かす。(i) のうち呼び出し先の付け替えは `<1>3` より `H` を
  動かさず、入れた節点は `<1>5` の (b) と `<1>6` の (c) である。(ii) は `<1>4` より借用版の中でだけ
  節点を落とし、それが (a) である。
  BY <1>2, <1>3, <1>4, <1>5, <1>6

## L9 (差が出るのは借用版の呼び出しの窓の中だけである)

**DEF 窓**。出力の実行における**窓**とは、`route` が借用版へ回した呼び出しの (E3) の段から、その呼び出し元が
その呼び出しの `call_rc` の `after` の `Release` 節点をすべて実行し終える段までである。その間その窓は
**開いている**。

**言明**。D-CP の共通接頭の時点 `p` で `H'(p, O) ≠ H(p, O)` となる計数下オブジェクト `O` があるならば、`p` で
生きている出力側の活性化に借用版の本体を持つものがあるか、`p` で走っている活性化の位置が `call_rc` の入れた
`after` の `Release` 節点である。

<1>1. 参照の持ち手 (D25) のうち、生きているオブジェクトが持つ分と環境が持つ分は、2 つの実行で
      一致する。
  <2>1. オブジェクトが持つ分。A5 より、生きているオブジェクトの inhabited な各 boxed leaf がちょうど 1 つの
        参照を持つ。D-CP より対応するオブジェクトは対応する値を持つ。
    BY A5, D-CP
  <2>2. 環境が持つ分。環境が活性化を作るのは `roots` が名指す関数についてである (A17 (i))。P24 より `roots`
        は変わらず、`borrow_funcref` が付ける `#borrow` の名前は入力に無いので `roots` に無い。よって環境が
        作る活性化の本体は `f_own` かグローバル初期化子であり、L8b よりその D10 の初期値は入力の対応する
        活性化の初期値に一致する。(E4) が `E` へ渡す参照は D9 の終端の `Ret` の行が決め、返る値は D-CP より
        一致する。
    BY A17, D9, D-CP, L8b, P24, CODE src/rc_ir/borrow.rs: borrow_funcref
  <2>3. QED
    BY <2>1, <2>2

<1>2. `Δ(p, O) := H'(p, O) − H(p, O)` は、対応する生きている活性化の対 `(a, a_in)` を渡る和
      `Σ_a (Obl'(a)(O) − Obl(a_in)(O))` に等しい。
  BY <1>1, D25, D8

<1>3. 対応する生きている活性化の対 `(a, a_in)` について、`B(a)` が借用版でないならば次が成り立つ。
      `a` が借用版へ回した呼び出し `c` で中断中でも、`c` の `after` の `Release` 節点の位置にいるのでも
      ないとき、`Obl'(a) = Obl(a_in)` である。そのどちらかであるときは、`Obl'(a) − Obl(a_in)` は `c` の
      `after` のうちまだ実行されていない `Release` が処分する参照の多重集合である。
  `a` の段についての帰納で示す。
  <2>1. `a` が作られた段。`B(a)` が借用版でないので、L8b より D10 の初期値が一致する。`a` はまだどの
        呼び出しでも中断していない。
    BY L8b
  <2>2. L8 の (a) と (b) の節点は借用版の本体にしかないので、`B(a)` が借用版でない `a` は実行しない。
    BY L8
  <2>3. L8 の (a)(b)(c) 以外の節点の段。L8 よりこの段は 2 つの実行で同じ節点であり、D-CP より同じ値を
        与えられるので、D10 の生成と消費は同じ参照を作り同じ参照を処分する。`Obl'(a) − Obl(a_in)` は
        変わらない。
    BY L8, D-CP, D10
  <2>4. 呼び出しの段 (E3)。D9 の `App` の行が消費する leaf は呼び出し先の所有 (D14) で決まる。呼び出し先が
        借用版でないとき、L8b より出力の呼び出し先は全 unit を所有し、A1 より入力の呼び出し先も全 unit を
        所有するので、消費は一致し差は変わらない。呼び出し先が借用版のとき、入力は借用される unit の参照を
        渡し出力は渡さないので、差はその unit の参照ちょうど 1 つずつ増える。`call_rc` はその unit を
        `after` に入れる -- `callee_owns` は偽であり、`arg_owned` は `B(a)` が借用版でないので L8a より
        真である。よって差は `after` が処分する参照に一致し、`a` は中断中になる。
    BY A1, D9, D10, D14, L8a, L8b, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
  <2>5. 返りの段 (E4)。呼び出し先が返す値は 2 つの実行で同じであり (D-CP)、D9 の終端の `Ret` の行が渡す
        参照の個数も同じなので、差は変わらない。`a` は `c` の `after` の `Release` 節点の位置へ進む。
        `prepend_rc` が `after` を継続の先頭に置くので、`a` は `after` を実行し終えるまで他の節点を
        実行しない。
    BY D9, D-CP, CODE src/rc_ir/borrow.rs: prepend_rc, RewriteCtx::rewrite_inner
  <2>6. `after` の `Release` の段 (L8 の (c))。出力だけがこれを実行し、`c` の `after` の参照を 1 つずつ
        処分する。最後の 1 つを実行し終えた時点で差は 0 に戻り、`a` は中断中でも `after` の位置でも
        なくなる。
    BY <2>4, <2>5, D10
  <2>6a. (E7) の段。`a` がまだ初期化されていないグローバルを読む節点の位置にあるとき、アクセサが初期化子の
        活性化を作り、`a` はそれが終わるまで中断中である。初期化子の活性化が終わって渡す参照の行き先は
        `E` であって `a` ではなく、`a` はその段で参照を得ない (D24 の (E7) の末尾、A8、D26)。よって
        `Obl'(a) − Obl(a_in)` は変わらない。
    BY A8, D24, D26
  <2>7. QED
    `<2>1` が基底、`<2>2` から `<2>6a` が段の全種である。段の種は D24 の (E1)-(E7) と、L10 の `<1>3` の
    (iii) -- `Llvm` の op が関数の型のオペランドを適用する段 -- である。(iii) は `a` の節点の段なので
    `<2>3` が扱う。(E5) は `Obl` を動かさず、(E6) の後に段は無い。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>6a, D24

<1>4. 借用版の活性化が作られるのは、`route` が借用版へ回した呼び出しの (E3) の段だけである。
  <2>1. 借用版の名前を `App` の callee に置くのは `route` だけである。`rewrite_inner` の他の腕は rhs を
        そのまま複製し、`RcRhs::Closure(target, caps)` の `target` も書き換えない。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, route
  <2>2. よってクロージャの値が指す関数が借用版であることは無く、D23 の意味の呼び出し先が借用版になるのは、
        `route` が置いた名前を callee に持つ呼び出しのときだけである。
    BY <2>1, D23, P12
  <2>3. QED
    BY <2>1, <2>2

<1>5. QED
  `<1>2` より `Δ(p, O) ≠ 0` ならば、ある対 `(a, a_in)` で `Obl'(a) ≠ Obl(a_in)` である。`<1>3` より、
  `B(a)` が借用版であるか、`a` が借用版へ回した呼び出しで中断中であるか、`a` がその呼び出しの `after` の
  `Release` 節点の位置にいるかのいずれかである。第 1 の場合は `a` 自身が生きている借用版の活性化である。
  第 2 の場合は `<1>4` よりその呼び出しが作った活性化が生きている借用版の活性化である。第 3 の場合が
  言明の後半である。
  BY <1>2, <1>3, <1>4

**この命題は 1 つの制御の流れを前提にする。** `p` で走っている活性化と、生きている借用版の活性化が別の
スレッドに属するとき、この命題の結論は「観測点を実行している活性化がその借用版の子孫である」を与えない。
第 11 節に前提として書き出す。

## L10 (借用版の動的な広がりに一意性の観測点は現れない) -- **偽**

**言明** (`funcs_observing_uniqueness` が果たそうとしているもの)。`f` が `borrow_versions` の鍵であるとき、
`f#borrow` の活性化から、活性化を作る段 (`<1>3` が 5 つ挙げる) の列で到達される活性化の本体に、一意性の
観測点は無い。以下この列を**到達の列**と呼ぶ。

L9 が与えるのは「差が出ているならば生きている借用版の活性化がある」までであり、観測している活性化がその
借用版からの到達の列の上にあることは、生きている活性化が 1 本の道をなすときにだけ従う。D24 の「活性化の林」
の段落がその形を述べる -- 「子は親が中断中の間だけ段を持ち、親は子が終わってから再開するので、1 つの制御の
流れの中で生きている活性化は根から下への 1 本の道をなす」。複数の制御の流れがある実行については第 11 節が
前提として書き出す。

<1>1. `funcs_observing_uniqueness` が返す集合 `observing` は、次の 4 つの規則が作る辺の上の最小不動点で
      ある。走査は 1 つの閉包 `scan(body, owner)` であり、`prog.funcs` の各関数の `body` に
      `owner = Some(その関数)` で、`prog.globals` の各初期化子の `init` に `owner = None` で適用される。
      (i) `owner` を持つ本体に `observes_uniqueness()` が真の `Llvm` の op があれば、`owner` が種として
      `observing` に入る。(ii) `owner` を持つ本体の `Let(_, App(callee, _), _)` の
      `FuncRef { name: callee.name }` が `prog.funcs` の鍵であれば、`owner` からその鍵へ辺が張られる。
      (iii) `owner` を持つ本体に `prog.funcs` の鍵でない callee への `App` があれば `owner` が
      `calls_indirectly` に入り、`calls_indirectly` の各元から `closure_targets` の各元へ辺が張られる。
      (iv) `closure_targets` は、`owner` の有無によらず、走査したどの本体の `Let(_, Closure(target, _), _)`
      が名指す `target` も含む。
  BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness

<1>2. `borrow_ify` は `observing` の元に借用版を作らない。`for func in prog.funcs.values()` の先頭で
      `if observing.contains(&func.name) { continue; }` が回る。
  BY CODE src/rc_ir/borrow.rs: borrow_ify

<1>3. 制御が 1 つの本体の新しい活性化に届く仕組みは、次の 5 つで尽きる。
      **(i)** callee の名前が `prog.funcs` の鍵である `App` の段 ((E3))。**(ii)** callee の名前が
      `prog.funcs` の鍵でない `App` の段 ((E3))。**(iii)** `Llvm` の op の段 ((E2)) のうち、その op の
      生成コードが関数の型のオペランドを適用するもの。**(iv)** C のエントリ点または `FFI_EXPORT` の
      エントリ点が林の根を作る段 ((E1))。**(v)** グローバルのアクセサが初期化子の活性化を作る段 ((E7))。
  <2>1. D24 が挙げる 7 種の段のうち、新しい活性化を作るのは (E1)、(E3)、(E7) の 3 つである。D24 の
        「活性化の林」の段落が「(E1) が作る活性化を**根**、(E3) と (E7) が作る活性化をそれを作った活性化の
        **子**と呼ぶ」と書く。(E2) は 1 つの活性化が自分の位置の節点を実行する段、(E4) は活性化が終わる段、
        (E5) は `mark_global` の段、(E6) はプロセスが終わる段である。
    BY D24
  <2>2. (E3) の呼び出し先は、callee の値がクロージャならその funptr が指す関数、funptr ならそれ自身で
        ある (D23)。`apply_lambda` はこの 2 つ以外の callee を受け取らない --
        `assert!(fun.ty.is_closure() || fun.ty.is_funptr())` が冒頭にある。`RcRhs::App` の callee は
        `RcVar` 1 つなので、その名前が `prog.funcs` の鍵であるか否かで (i) と (ii) に割れる。
    BY D23, CODE src/generator.rs: Generator::apply_lambda, CODE src/rc_ir/ast.rs: RcRhs
  <2>3. (E1) が (iv)、(E7) が (v) である。D22 は環境を 3 つに分け、そのうちグローバルのアクセサについて
        「アクセサはグローバルを読む活性化から呼ばれる。その活性化は初期化子の活性化が終わるまで中断中で
        あり ((E7))、C のエントリ点や `FFI_EXPORT` のエントリ点のように活性化の根を作るのではない」と
        書く。残る 2 つが (E1) である。
    BY D22, D24
  <2>4. **D24 の 3 つでは尽きない。** (E2) の段のうち `Let(x, Llvm(gen, args), k)` の段では、その op の
        生成コードが `Generator::apply_lambda` を呼んで関数の型のオペランドを適用することがある。
        `impl LLVMGen for` の中で `apply_lambda` に届く呼び出しを持つのは次の 8 つである。
        `InlineLLVMFixBody` (`Std::fix`)、`InlineLLVMUnionModBody` (どの union の `mod_{変位}` も
        これである)、`InlineLLVMWithRetainedFunctionBody`、`InlineLLVMArrayBorrowElementsBody`、
        および `apply_io_act_to_data_ptr` と `run_ios_runner` を呼ぶ 4 つ --
        `InlineLLVMUnsafeMutateBoxedInternalFunctionBody`、`InlineLLVMUnsafeMutateBoxedIOSInternalBody`、
        `InlineLLVMArrayMutateElementsInternalBody`、`InlineLLVMArrayMutateElementsIosInternalBody`。
        これが (iii) である。
    BY CODE src/fixstd/builtin.rs: InlineLLVMFixBody::generate_tail,
       InlineLLVMUnionModBody::generate, InlineLLVMWithRetainedFunctionBody::generate,
       InlineLLVMArrayBorrowElementsBody::generate,
       InlineLLVMUnsafeMutateBoxedInternalFunctionBody::generate,
       InlineLLVMUnsafeMutateBoxedIOSInternalBody::generate,
       InlineLLVMArrayMutateElementsInternalBody::generate,
       InlineLLVMArrayMutateElementsIosInternalBody::generate,
       apply_io_act_to_data_ptr, run_ios_runner
  <2>5. QED
    `<2>1` が段の種を尽くし、`<2>2` が (E3) を (i) と (ii) に割り、`<2>3` が (E1) を (iv)、(E7) を (v) と
    し、`<2>4` が (E2) の中に (iii) を見つける。(E4)(E5)(E6) は活性化を作らない。
    BY <2>1, <2>2, <2>3, <2>4

  **この列挙には宣言された裏付けが無い。** `borrows_operand`、`result_prov`、`observes_uniqueness`、
  `unique_check_operand` はいずれも `LLVMGen` のメソッドとして op が自分について宣言するものであり、
  A3 がその忠実さを仮定として引き受けている。「この op は生成コードでオペランドの関数を適用する」を
  述べるメソッドは無く、`<2>4` の 8 つは `apply_lambda` の呼び出し位置を数えて得たものである。
  第 11 節がこれを穴として報告する。

<1>4. CASE 到達の列が (i) と (ii) の段だけからなる場合。この場合 L10 は成り立つ。
  <2>1. 到達の列の各活性化の本体は、入力のある関数 `g` について `g_own` か `g#borrow` のどちらかであり、
        どちらも `g` の本体の束縛変数を付け替えたものである。列の最初は `f#borrow` であって `f` の本体の
        名前替えである (P9)。グローバル初期化子の本体は列に現れない -- その活性化を作るのは (v) の段
        だけであり、この場合の列は (v) の段を持たない。
    BY P9
  <2>2. (i) の段について。`route` が返す呼び出し先は元の呼び出し先と同じ入力関数の版であり、呼び出し先が
        入力の関数を名指すとき返る名前は出力の `funcs` の鍵である (P12)。よって `g` の本体の位置にある
        (i) の段は、`<1>1` の (ii) の辺 `g -> h` を与える。ここで `h` は到達される活性化の本体の入力関数で
        ある。
    BY P12, <2>1
  <2>3. (ii) の段について。その呼び出し先は、その時点で callee の値が持つクロージャの funptr が指す関数で
        ある。
    BY D23
  <2>4. クロージャの値を作る節点は `Let(x, Closure(target, caps), k)` だけである。`Llvm` の op や容器から
        読み出したクロージャの値も、どこかの `Closure` 節点が作ったオブジェクトである -- A3 の宣言の表で
        `Unknown` は「既存のオブジェクト」であり、`Fresh` はその op が割り当てるオブジェクトである。
        `Closure` 以外に capture object を割り当てる節点は D10 の生成の表に無い。
    BY A3, D10, CODE src/rc_ir/codegen.rs: Generator::build_rc_closure
  <2>5. `Closure` 節点の `target` は、その節点がどの本体にあっても `closure_targets` の元である。`<1>1` の
        (iv) より、走査は `prog.funcs` の各 `body` と `prog.globals` の各 `init` の両方を通り、`Closure` の
        腕は `owner` を見ずに `target` を入れる。`route` は `Closure` の `target` を書き換えない
        (`rewrite_inner` は `RcRhs::Closure` をそのまま複製する) ので、出力のクロージャが指す関数は入力の
        関数である。
    BY <1>1, CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness, RewriteCtx::rewrite_inner
  <2>6. よって `g` の本体の位置にある (ii) の段は、`g` を `calls_indirectly` に入れ、`<1>1` の (iii) と
        `<2>5` により `g` から到達される活性化の本体の入力関数への辺を与える。
    BY <1>1, <2>3, <2>4, <2>5
  <2>7. QED
    到達される活性化の本体に観測点があるとすると、その本体の入力関数は `<1>1` の (i) により `observing` の
    種である。`<2>2` と `<2>6` より、到達の列の各段は入力の関数の対の間に `<1>1` の辺を与えるので、辺の
    個数について繰り返して `f ∈ observing` である。これは `<1>2` に反する。
    BY <1>1, <1>2, <2>1, <2>2, <2>6

<1>5. CASE 到達の列に (iii) の段がある場合。この場合 L10 は**偽**である。
  <2>1. `funcs_observing_uniqueness` が `RcRhs::Llvm(llvm_gen, _)` の腕で問うのは
        `llvm_gen.observes_uniqueness()` だけである。オペランドの型も、その op が生成コードでオペランドの
        関数を適用するかどうかも問わない。よって (iii) の段を持つ本体の `owner` は、その段だけを理由に
        `calls_indirectly` に入ることは無く、`closure_targets` への辺を得ない。
    BY CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness
  <2>2. QED (反証)
    第 6 節のプログラムがこの形である。`Main::relay` の本体は `union_mod_1(modifier, opt)` を持ち、この op
    は `InlineLLVMUnionModBody` であって `modifier` を適用する。`modifier` が持つクロージャの target
    `closure#0` は観測点を持つ。`relay` の `App` の callee はどれも `prog.funcs` の鍵なので `<2>1` より
    `relay ∉ calls_indirectly` であり、`relay ∉ observing` である。`<1>2` の門を通って `relay#borrow` が
    作られ、観測点の位置で `H' = 2`、`H = 1` である。
    BY <2>1, 第 6 節

<1>6. CASE 到達の列に (iii) の段が無く、(iv) の段がある場合。**1 つの制御の流れではこの場合は起きない。**
  (iv) の段は林の根を作る。D24 の「活性化の林」の段落より、1 つの制御の流れの中で生きている活性化は根から
  下への 1 本の道をなすので、`f#borrow` の活性化が生きている間に別の根が作られるのは、別の制御の流れが入る
  ときだけである。D24 はその形を「`FFI_EXPORT` を通じて外から入るスレッド」と書く。複数の制御の流れがある
  実行はこの言明の前提の外にあり、第 11 節がそれを書き出す。
  BY D24

<1>7. CASE 到達の列に (iii) の段も (iv) の段も無く、(v) の段がある場合。**この文書はこの場合を決めて
      いない。**
  窓が開いている間にグローバルの最初の読みが起きると、アクセサが初期化子の活性化を作り ((E7))、その本体が
  窓の中で走る。`funcs_observing_uniqueness` は `owner` を持つ本体からしか種を取らないので、初期化子の
  本体に直接在る観測 op はどの関数も `observing` に入れない。決めるのに要る事実と、それを支持する測定は
  第 11 節が述べる。

<1>8. QED
  `<1>3` が段の種を尽くし、`<1>4` から `<1>7` はその上で尽くされている -- 到達の列は (iii) の段を持つか
  持たないかであり、持たないならば (iv) の段を持つか持たないかであり、どちらも持たないならば (v) の段を
  持つか持たないかである。`<1>5` が反例を与えるので L10 は偽である。`<1>7` は決まっていないので、`<1>5` の
  欠陥を直しても L10 が成り立つとは、この文書は言えない。
  BY <1>3, <1>4, <1>5, <1>6, <1>7

## 6. 欠陥の報告 -- `Llvm` の op が適用するクロージャに門が届かない

### 6.1 入力

```fix
module Main;

type B = box struct { v : I64 };

// `x` と `w` は消費されないので借用に倒れる。`y` は `union_make` が消費するので所有のまま残る。
// 観測点へ届くのは `mod_some` が適用するクロージャだけであり、`mod_some` の本体は
// `InlineLLVMUnionModBody` の 1 つの `Llvm` の op である。
relay : I64 -> B -> B -> B -> Bool;
relay = |n, x, y, w| (
    if n > 0 { relay(n - 1, x, y, w) };
    let opt : Option B = Option::some(y);
    let opt = opt.mod_some(|b| (
        let (u, b) = b.unsafe_is_unique;
        eval b;
        B { v : if u { 1 } else { 0 } }
    ));
    opt.as_some.@v == 1
);

main : IO ();
main = (
    let o = B { v : 1 };
    let w = B { v : 2 };
    let u = relay(0, o, o, w);   // 同じオブジェクトを借用位置 `x` と所有位置 `y` に渡す
    eval w.@v;                   // `w` を呼び出しの後で使う
    println("unique = " + u.to_string)
);
```

各成分の役目は次のとおりである。

- **`x`**: `relay` の本体で消費されないので借用に倒れ、その `Release` が借用版で落ちる。
- **`y`**: `union_make_1` が消費するので所有のまま残る。実行時には `x` と同じオブジェクトを指し、`mod_some`
  が適用するクロージャの引数になる。
- **`w`**: `main` が呼び出しの後で使うので `routing_saves_retain` が真になる。
- **`mod_some`**: 観測点をクロージャの中に置き、そのクロージャを `Llvm` の op に適用させる。どの union の
  `mod_{変位}` も `InlineLLVMUnionModBody` である
  (`CODE src/ast/program.rs: Program::add_methods` -- union の各フィールドについて `union_mod_function` を
  登録する、`CODE src/fixstd/builtin.rs: union_mod_function`)。

### 6.2 経路

`--emit-rc-ir Main` が書く `rc_ir.Main.pre.txt` (`insert_rc` の直後) は次を含む。名前は短くしてある。

```
fn relay(n, x, y, w) -> Bool:
  ...
  case 0(n <= 0):
      release w
      release x
      let opt : Option B = union_make_1(y)
      let cap : #CapList = struct_make()
      let m : B -> B = Main::relay::closure_lam0#funptr1(cap)   // 直接呼び出し
      let opt2 : Option B = union_mod_1(m, opt)                 // <- op が `m` を適用する
      let b : B = union_as_1(opt2)
      let v : I64 = struct_get_0(b)
      release b
      let r : Bool = int_eq(1, v)
      ret r

fn Main::relay::closure_lam0#funptr1(cap0 : #CapList) -> B -> B:
    let cl : B -> B = closure closure#0[cap0]                   // <- 唯一の Closure 節点
    ret cl

fn Main::relay::closure_lam0#funptr1::closure#0(p : B), cap cap : #DynamicObject -> B:
    ...
    let v : (Bool, B) = is_unique(p)                            // <- 観測点
    ...

fn main(...):
  let o : B = struct_make(1)
  let w : B = struct_make(2)
  retain w
  retain o
  let u : Bool = relay(0, o, o, w)
  let t : I64 = struct_get_0(w)
  release w
  ...
```

`funcs_observing_uniqueness` が `relay` の本体から取るものは次のとおりである。`observes_uniqueness()` が真の
op は無いので種にならない。`App` の callee は `relay` 自身と `Main::relay::closure_lam0#funptr1` の 2 つで、
どちらも `prog.funcs` の鍵なので直接呼び出しの辺になり、`calls_indirectly` には入らない。`union_mod_1` は
`RcRhs::Llvm` なので `observes_uniqueness()` だけを問われ、`m` を適用することは見られない。`closure#0` は
`closure_targets` に入るが、`relay` は `calls_indirectly` に居ないのでそこへの辺を持たない。
`Main::relay::closure_lam0#funptr1` の本体は `Closure` 節点と `Ret` だけなので、そこからも辺は伸びない。
よって `relay ∉ observing` である。

`rc_ir.Main.post.txt` は次を含む。

```
fn relay#borrow(n : I64 {u}, x : B {borrow}, y : B {own}, w : B {borrow}) -> Bool:
  ...
  case 0(n <= 0):
      let opt : Option B = union_make_1(y)          // `release w` と `release x` が消えている
      ...

fn main(...):
  let o : B = struct_make(1)
  let w : B = struct_make(2)
  retain o
  let u : Bool = relay#borrow(0, o, o, w)
  release o                                          // call_rc の after
  let t : I64 = struct_get_0(w)
  release w
  ...
```

入力の実行での `o` の参照カウントは、観測点で 1 である。`main` は `o` を 2 つの位置に渡すために retain して
2 にし、両方を `relay` へ渡す (`relay` は入力では全パラメータを所有する)。`relay` の `release x` が 1 つを
処分し、`union_make_1(y)` が残る 1 つを union の payload へ移し、`union_mod_1` がその payload をクロージャへ
渡す。

出力の実行では 2 である。`relay#borrow` は `x` を借用するので `release x` を持たず、`main` はその参照を
呼び出しの後まで持つ。`main` の `release o` は呼び出しの直後にあるが、観測点はそれより前にある。

**差がクロージャの適用の仕組みだけであることを測った。** 上のプログラムの `opt.mod_some(f)` を、同じ
クロージャの `App` による適用

```fix
    let f = |b| ( let (u, b) = b.unsafe_is_unique; eval b; B { v : if u { 1 } else { 0 } } );
    let opt : Option B = Option::some(y);
    let opt = if opt.is_some { Option::some(f(opt.as_some)) } else { opt };
```

に置き換えると、`-O max` でも `unique = true` になり、出力に `relay#borrow` は現れない。観測点も、借用に
倒れるパラメータも、振り分けの条件も同じで、違うのはクロージャを適用するのが `Llvm` の op か `App` かだけで
ある。

### 6.3 誤った出力

| 最適化水準 | `unique = ` |
|---|---|
| `-O none` | `true` |
| `-O basic` | `true` |
| `-O max` | `false` |

`unsafe_is_unique` を `Debug::assert_unique` に置き換えた同じ形のプログラム -- クロージャの本体を
`let b = b.assert_unique(|_| "b is shared"); B { v : b.@v }` にしたもの -- は、`-O none` と `-O basic` で
`done = true` を印字し、`-O max` では次を出して止まる。

```
Value is not unique: b is shared
error: Program terminated by signal 6
```

各水準の前に `.fixlang` を消して測った。

### 6.4 この入力は D11 を破らない

出力の実行で `o` の参照は、`main` が 1 つ (呼び出しの後の `release o` が処分する)、`relay#borrow` の `y` が
1 つ (union の payload へ移り、クロージャの中で処分される) である。過剰処分 (S-a) も漏れ (S-b) も
解放後の読み (S-c) も起きない。`-O max` でビルドした二値を valgrind の下で走らせると、エラー 0、
`definitely lost` 0 バイトである。すなわちこれは、D11 を満たしたまま P26 だけを破る入力である。

### 6.5 修正の候補

門が閉じそこねているのは「その `Llvm` の op が、関数の型のオペランドを適用しうるか」である。L10 の `<1>3`
の `<2>4` が挙げる 8 つの op がそれを行う。候補は 3 つあり、どれを採るかは設計の判断である。

**候補 A -- op に宣言させる。** `LLVMGen` に「第 `i` オペランドを適用しうる」を答えるメソッドを足し、
8 つの op がそれを override する。`funcs_observing_uniqueness` はそれが真の op を持つ本体の `owner` を
`calls_indirectly` に入れる。`borrows_operand`・`result_prov`・`observes_uniqueness`・
`unique_check_operand` と同じ形であり、A3 が引き受けている「宣言は生成コードを正しく述べている」の傘の下に
入る。代償は、宣言を落とした op が門を素通りすることであり、それは A3 の他の宣言と同じ性質の穴である。

**候補 B -- オペランドの型から取る。** `Llvm` の op のオペランドのうち、型が
`is_closure() || is_funptr()` であるものが 1 つでもあれば、`owner` を `calls_indirectly` に入れる。
宣言を手で並べずに済むが、関数を運ぶだけで適用しない op -- `struct_make`、`union_make`、`array_set` --
でも発火するので、門は広くなる。広げた分の代償は測っていない。

**候補 C -- `Llvm` の op をすべて間接呼び出しとみなす。** 最も広く、最も述べやすい。関数の型のオペランドを
1 つも持たない op でも発火するので、`observing` はほとんどの関数を含むことになりうる。借用版がいくつ止まるかは
測っていない。

**この 3 件目の落ち方は、第 7 節から第 9 節と同じ族である。** 門が数える「制御が本体へ届く仕組み」の列挙を
手で並べており、落とした分だけ穴が開く。L10 の `<1>3` が、その列挙を段の種 (D24) から作り直したもの
である。

## 7. 記録 (1) -- 門が無かったときの反例 (#551)

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

<1>1. `observe` の本体について `collect_consumes` が報告するのは、`arm0` の `Llvm(is_unique, [y])` に
      ついての `(y, [])` と、`arm1` の `App(observe, [n2, x, y, w])` について `owns` が真である位置の
      引数の leaf だけである。ほかのどの節点も何も報告しない。
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
  <2>2. 1 周目、`<1>1` より `Llvm(is_unique, [y])` が `(y, [])` を報告し、`origin(y, [])` は
        `Exactly((y, []))` である (`y` は `Binding::Param`)。`y` は `vars.param_tys` にあるので
        `(y, [])` が挿入される。
    BY <1>1, CODE src/rc_ir/borrow.rs: infer_ownership, CODE src/rc_ir/ownership.rs: origin_inner の
       `Binding::Param` の腕
  <2>3. `<1>1` より `App(observe, [n2, x, y, w])` は、`owns` が真である位置の引数だけを報告する。
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

## 8. 記録 (2) -- 門が直接呼び出しだけを閉じていたときの反例

この節は、`funcs_observing_uniqueness` が到達可能性を直接呼び出しのグラフだけで取っていたときのコードに
対する反証である。**その欠陥は直っている** -- 今のコードは `calls_indirectly` と `closure_targets` を
集め、間接呼び出しを持つ関数からクロージャの target すべてへ辺を張るので、下の `relay` は `observing` に
入り借用版を持たない。下のプログラムは 3 つの水準すべてで `unique = true` を印字する。


### 8.1 入力

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

### 8.2 経路

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

### 8.3 誤った出力

門が直接呼び出しだけを閉じていたときの二値での結果である。

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

### 8.4 この入力は D11 を破らない

出力の実行で `o` の参照は、`main` が 1 つ (呼び出しの後の `release o` が処分する)、`f1` の capture が
1 つ (クロージャの本体の `release o2` が処分する) である。過剰処分 (S-a) も漏れ (S-b) も解放後の読み (S-c) も
起きない。すなわちこれは、D11 を満たしたまま P26 だけを破る入力である。

## 9. 記録 (3) -- 門がクロージャを `prog.funcs` の本体からだけ集めていたときの反例

この節は、`funcs_observing_uniqueness` が `closure_targets` を `prog.funcs` の関数の本体からだけ
集めていたときのコードに対する反証である。**その欠陥は直っている** -- 今のコードは走査を 1 つの閉包
`scan(body, owner)` にまとめ、`prog.funcs` の各 `body` と `prog.globals` の各 `init` の両方をそれに
通す。`RcRhs::Closure(target, _)` の腕は `owner` を見ずに `closure_targets.insert(target.clone())` を
行うので、グローバル初期化子の本体が作るクロージャの target も集まる
(`CODE src/rc_ir/borrow.rs: funcs_observing_uniqueness`)。下のプログラムは 3 つの水準すべてで
`unique = true` を印字する。

### 9.1 入力

```fix
module Main;

type B = box struct { v : I64 };

// 観測点を持つ関数のクロージャを、グローバル初期化子の本体が作る。
// 当時の `closure_targets` は `prog.funcs` の本体だけから集めたので、この target を含まない。
checkers : Array (B -> Bool);
checkers = [ |b| ( let (u, b) = b.unsafe_is_unique; eval b; u ) ];

// `x` と `w` は消費されないので借用に倒れる。`y` は `cl(y)` が消費するので所有のまま残る。
// 観測点へは間接呼び出し `cl(y)` だけが届く。
relay : I64 -> B -> B -> B -> (B -> Bool) -> Bool;
relay = |n, x, y, w, cl| (
    if n > 0 { relay(n - 1, x, y, w, cl) };
    cl(y)
);

main : IO ();
main = (
    let o = B { v : 1 };
    let w = B { v : 2 };
    let cl = checkers.@(0);
    let u = relay(0, o, o, w, cl);   // 同じオブジェクトを借用位置 `x` と所有位置 `y` に渡す
    eval w.@v;                       // `w` を呼び出しの後で使う
    println("unique = " + u.to_string)
);
```

各成分の役目は次のとおりである。

- **`x`**: `relay` の本体で消費されないので借用に倒れ、その `Release` が借用版で落ちる。
- **`y`**: `cl(y)` が消費するので所有のまま残る。実行時には `x` と同じオブジェクトを指し、観測点の
  オペランドになる。
- **`w`**: `main` が呼び出しの後で使うので `routing_saves_retain` が真になる。
- **`checkers`**: クロージャをグローバル初期化子の本体で作らせる。`|b| ...` を `relay` の呼び出しに直接
  書くと、その `Closure` 節点は `main` の本体に入るので `closure_targets` に載り、門が閉じる。

### 9.2 経路

`--emit-rc-ir Main` が書く `rc_ir.Main.pre.txt` (`insert_rc` の直後) は次を含む。名前は短くしてある。

```
fn Main::checkers::closure#0(#param : Main::B), cap cap : Std::#DynamicObject -> Std::Bool:
    ...
    let v : (Std::Bool, Main::B) = is_unique(#param)     // <- 観測点
    ...

global Main::checkers:
    let closure : Main::B -> Std::Bool = closure closure#0[]     // <- ここが唯一の Closure 節点
    ret closure

fn relay(n, x, y, w, cl) -> Bool:
  ...
  case 0(n <= 0):
      release x
      release w
      let a : Bool = cl(y)          // <- 間接呼び出し
      ret a
```

当時の `funcs_observing_uniqueness` は `for (fref, func) in &prog.funcs` の中でだけ `RcRhs::Closure` を見た。
`global Main::checkers` の本体はこのループを通らないので、`Main::checkers::closure#0` は
`closure_targets` に入らない。`relay` は `cl` への `App` で `calls_indirectly` に入るが、そこから伸びる辺は
`closure_targets` の元だけなので、観測点を持つ関数へ届かない。よって `relay ∉ observing` である。

`rc_ir.Main.post.txt` は次を含む。

```
fn Main::relay#...#borrow#l1(#v0 : I64 {u}, #v1 : B {borrow}, #v2 : B {own}, #v3 : B {borrow},
                             #v4 : B -> Bool {(u, own)}) -> Bool:
```

`main` は `relay#...#borrow#l1(...)` を呼び、その直後に `release o` を持つ。

入力の実行での `o` の参照カウントは、観測点で 1 である。`main` は `o` を 2 つの位置に渡すために retain して
2 にし、両方を `relay` へ渡す (`relay` は入力では全パラメータを所有する)。`relay` の `release x` が 1 つを
処分し、`cl(y)` が残る 1 つをクロージャの本体へ渡す。

出力の実行では 2 である。`relay#borrow` は `x` を借用するので `release x` を持たず、`main` はその参照を
呼び出しの後まで持つ。`main` の `release o` は呼び出しの直後にあるが、観測点はそれより前にある。

**差が `Closure` 節点の居場所だけであることを測った。** 上のプログラムの `checkers.@(0)` を、`main` の本体で
作った 2 つのクロージャの実行時の選択

```fix
    let f1 = |b| ( let (u, b) = b.unsafe_is_unique; eval b; u );
    let f2 : B -> Bool = |b| ( eval b; false );
    let cl = if args.@size > 100 { f2 } else { f1 };
```

に置き換えると、`-O max` でも `unique = true` になる。間接呼び出しであることも、観測点も、借用に倒れる
パラメータも、振り分けの条件も同じで、違うのは `Closure` 節点が `main` の本体にあるか
`global Main::checkers` の本体にあるかだけである。

### 9.3 誤った出力

門がクロージャを `prog.funcs` の本体からだけ集めていたときの二値での結果である。

| 最適化水準 | `unique = ` |
|---|---|
| `-O none` | `true` |
| `-O basic` | `true` |
| `-O max` | `false` |

各水準の前に `.fixlang` を消して測った。

### 9.4 この入力は D11 を破らない

出力の実行で `o` の参照は、`main` が 1 つ (呼び出しの後の `release o` が処分する)、`relay#borrow` の `y` が
1 つ (クロージャの本体へ渡り、そこで処分される) である。過剰処分 (S-a) も漏れ (S-b) も解放後の読み (S-c) も
起きない。すなわちこれは、D11 を満たしたまま P26 だけを破る入力である。

## 10. P21 と P26 の関係 -- D-CP は P26 の側だけを閉じる

**答えを先に書く。D-CP は P26 の循環を閉じるが、P21 の側は閉じない。** 2 つは相互依存であり、その依存は
P26 を強めても閉じない。これは 2 つの言明についての発見であって、証明の書き方の問題ではない。

### 10.1 循環が何であったか

P21 は「対応する 2 つの活性化」を「同じ実行路を辿る 2 つの活性化」と定め、それを**前提**に置く。README は
倒れうる経路として D18 の観測を挙げ、「P26 がその観測が悪化しないことを述べるが、P26 は改善の向き (偽から
真へ) を許すので、路の対応そのものは別に前提される」と書く。循環はここにある。

1. 路の対応が保たれるには、2 つの実行の観測値が**等しい**ことが要る。観測値が違えば、その値を scrutinee に
   する `Match` で選ばれるアームが違い、以後の節点列が違う。D18 の観測値は `Bool` であり、
   `Debug::assert_unique` はそれを `if` に掛け、`if` は RC IR では `Match` になる
   (`CODE src/fixstd/std.fix: assert_unique`, `CODE src/rc_ir/lower.rs: Lowerer::lower_if`)。
2. P26 が言うのは「入力が真ならば出力も真」であって、等しいことではない。L7a の `<1>1` が示すとおり、
   `cancel` では「入力が偽、出力が真」が実際に起きうる。`unsafe_is_unique` の doc が認めているのがその
   向きなので、P26 を等号に強めることはできない。
3. よって P21 の前提は P26 から出ない。

### 10.2 P26 の側は閉じた

D-CP は路の対応を**前提に置くのでなく定義する**。共通接頭は、対応する段が同じ節点を実行し同じ値を作る
限りの部分であり、そこまでは対応が定義から成り立つ。P26 を共通接頭の上の主張として述べれば、循環は無い --
L6 と L7 は共通接頭の各時点についての帰納であり、路の対応を前提にしない。

さらに L7a が、`cancel` について共通接頭の終わり方を尽くす。出口は (X1) と (X2) の 2 つであり、どちらも
`unsafe_is_unique` の doc が認めている向きに倒れる。**すなわち `cancel` について、P26 は共通接頭の上で
成り立ち、共通接頭が終わる点でも認められた向きにしか終わらない。** P26 の側に残るものは無い。

`borrow_ify` についてはこれが言えない。第 6 節の入力では、共通接頭は (X1) で終わり、その向きは
「入力が真、出力が偽」である。門が正しくなるまで、L7a に対応する言明は `borrow_ify` について偽である。

### 10.3 P21 の側は閉じない

P21 の結論が要るのは P23 -- `cancel` の出力が D12 を満たすこと -- である。D12 は D11 を全関数の本体に
課し、D11 は本体の**すべての静的な実行路** (D3) についての述語であって、D21 はそれを「その路を辿るすべての
活性化についての主張」と読む。よって P23 を示すには、出力の本体の各静的な実行路 `ρ'` と、`ρ'` を辿る各
活性化について (S-a)(S-b)(S-c) を示す必要がある。

共通接頭が終わった後の出力の活性化は、`ρ'` を辿りながら、入力の対応する静的な実行路 `ρ` を辿る活性化を
相手に持たない。D21 は「1 つの実行路を辿る活性化が存在しないこともある」と書いており、存在しない路に
ついて D12 は空虚に成り立つので、そこから出力についての事実は出ない。P21 はその活性化について何も言わず、
P23 はその活性化について (S-c) を持たない。

**この穴は P26 を強めても塞がらない。** 塞ぐには P26 を等号にする必要があり、10.1 の 2 がそれを禁じる。

### 10.4 出口の候補 (判断はオーケストレータに残す)

- **(a) P21 から 2 つの実行の比較を外す。** 出力の本体について (S-c) を直接示す。走査が持つ量 --
  `pending`、`outstanding`、P18a と P18c の不等式 -- はどれも 1 つの本体の 1 つの実行路についての量で
  あって、入力の実行を参照しない。この道なら、共通接頭が終わった後の活性化も他と同じに扱える。
- **(b) P21 の前提を「共通接頭の上で」と読み替える。** そのとき P23 は共通接頭の上でしか言えないので、
  D12 の全称と食い違う。この道を採るなら D11 か D12 の側を書き換えることになる。

この文書は README を書き換えないので、どちらを採るかは残す。**同じ問いは、2 つの実行を比べる他の命題
すべてに掛かる。**

## 11. オーケストレータへ報告する定義と仮定の穴

1. **複数の制御の流れ (L9、L10)。** D24 の「活性化の林」の段落は、1 つの制御の流れの中で生きている活性化が
   根から下への 1 本の道をなすことを述べる。L9 と L10 はその形に立つ。D24 は複数の制御の流れ
   (`FFI_EXPORT` を通じて外から入るスレッド) がある実行も認めており、その形では借用版が別の流れで計数を
   持ち上げたまま、こちらの流れが観測しうる。そのとき入力の観測値そのものが段の並べ方で決まるので、
   性質の言明の側で扱う事柄である。果たす者: 無し。

2. **D24 が、活性化を作る段を 3 つしか挙げていない。** D24 の「活性化の林」の段落は「(E1) が作る活性化を
   **根**、(E3) と (E7) が作る活性化をそれを作った活性化の**子**と呼ぶ」と書く。L10 の `<1>3` の `<2>4` が
   示すとおり、(E2) の節点の段のうち `Let(x, Llvm(gen, args), k)` の段でも、op の生成コードが
   `Generator::apply_lambda` を呼んで新しい活性化を作る。第 6 節の欠陥は、
   `funcs_observing_uniqueness` がこの段を数えていないことである。**D24 にこの段を足す必要がある。**
   足す先は (E2) の表であり、その段の `Obl` の動きは D9 の `Llvm` の行と A3 の宣言が決める。

3. **「この op はオペランドの関数を適用する」を述べる宣言が無い。** `LLVMGen` は `borrows_operand`、
   `result_prov`、`observes_uniqueness`、`unique_check_operand` を持ち、A3 がその忠実さを引き受けている。
   オペランドを適用するかどうかを述べるメソッドは無いので、L10 の `<1>3` の 8 つは `apply_lambda` の
   呼び出し位置を数えて得たものであり、宣言の裏付けを持たない。第 6 節の修正候補 A はこの宣言を足す案で
   ある。果たす者: 無し。

4. **(E7) の段が作る初期化子の活性化 (L10 の `<1>7`)。** 窓が開いている間にグローバルの最初の読みが
   起きると、アクセサが初期化子の活性化を作り、その本体が窓の中で走る。
   `funcs_observing_uniqueness` は `owner` を持つ本体からしか種を取らないので、初期化子の本体に直接在る
   観測 op はどの関数も `observing` に入れない。**この文書はこの場合を決めていない。**

   決めるのに要るのは次の 3 つである。(a) 初期化子の活性化とその広がりが名指す計数下 (D26) のオブジェクトは
   すべてその広がりの中で割り当てられたものである、という補題。初期化子はパラメータも capture も持たず
   (D1)、`Obl` は空で始まる (D24 の (E7)) ので、出発点は空であり、A3 の `Unknown` の行 (「オペランドの leaf が
   指すオブジェクトから到達できるか、グローバル値が到達する」) と A18 (b) (「グローバル状態のオブジェクトは
   計数下のオブジェクトへの参照を持たない」) がこれを支える。(b) 窓を開けた側の活性化が、初期化子の活性化の
   間 段を持たないこと -- D24 の (E7) が「`a` は `b` が終わるまで**中断中**であり、その間 `a` は段を持たない」
   と書くので、これは D24 が与える。(c) その広がりの中で開く窓については L10 の (i)(ii)(iii) が要り、
   (iii) が偽なので今は閉じない。

   **(a) を支持する測定がある。** グローバル初期化子の本体に観測点を置き、その最初の読みを借用版の中で
   起こすプログラムを 3 水準で走らせた (`.fixlang` は各水準の前に消した)。`relay#borrow` は作られ、その
   3 つの `B` 型パラメータはすべて `{borrow}` と注釈され、両方のグローバルの最初の読みはその活性化の中で
   起きる。自分で割り当てたオブジェクトを観測する初期化子は 3 水準すべてで `true`、別のグローバルから
   読んだオブジェクトを観測する初期化子は 3 水準すべてで `false` を返した。後者は L0 の global の腕
   (グローバル状態のオブジェクトの観測値は偽) である。

5. **D-CP を D29 と P25 の上に置き直す作業が残っている。** README は D29 (対応する活性化) と P25 (活性化の
   木の対応) を持ち、P26 の言明も「実際の実行」-- 観測点が返す値がその時点の参照カウントと一致する活性化の
   木 (D21) -- の上に立つ形になった。この文書の第 2 節の D-CP は、同じ対応を段の列の上で定め、対応が伸びる
   限りの部分を共通接頭と呼ぶ。第 4 節 (`cancel` の半分) の証明は D-CP の上に立っている。D29 と P25 の上に
   置き直すかどうかはオーケストレータの判断であり、この文書は第 6 節の欠陥で止まったのでその置き直しを
   していない。

6. **D18 が `Std::Array::_unsafe_is_storage_unique` を覆っていない。** コードが観測として扱うのは
   `LLVMGen::observes_uniqueness` が真を返す op であり、それは `InlineLLVMIsUniqueFunctionBody` と
   `InlineLLVMArrayIsStorageUniqueBody` の 2 つである。後者は `Std::Array::_unsafe_is_storage_unique` の
   本体である。コードが押さえる集合は D18 の集合の上位集合なので、押さえる向きではこの差は効かない。
   破る向き -- 第 6 節から第 9 節の反例 -- を `_unsafe_is_storage_unique` で書けるかどうかは調べていない。

7. **内部の一意性検査 ((X2)) をどの命題も名指していない。** `LLVMGen::unique_check_operand` が宣言する
   一意性検査は、参照カウントを読んで複製を作るかどうかを決める。カウントを変える変換はこの分岐を倒し、
   2 つの実行のオブジェクトを違える。D21 が活性化の側のデータに挙げる 3 種 -- 一意性の観測点、外部の状態を
   読む `Llvm` の演算、`App` -- にこの分岐は入っていない。

## 12. この文書が読んだコードの版

対象コミット `131b1a21a7f865b88c320dd3d6a06bf6f5bbc97d` の `src/` を読んだ。

第 6 節と第 11 節の項目 4 の実行に使った二値は、そのコミットからビルドしたものである
(`target/release/fix`、`fix 1.5.0 (131b1a2-dirty)`。dirty が指すのは `dev-docs/` の下の markdown だけで、
`src/` はコミットと一致する)。同じ二値で第 7 節・第 8 節・第 9 節のプログラムも走らせ、3 つとも
`-O none`、`-O basic`、`-O max` のいずれでも `unique = true` を印字することを確かめた。

第 7 節の表の実行は門が無かった版の二値、第 8 節の表の実行は門が直接呼び出しだけを閉じていた版の二値
(`fix 1.5.0 (b617052)`)、第 9 節の表の実行は門がクロージャを `prog.funcs` の本体からだけ集めていた版の二値
(`fix 1.5.0 (2e5d2f0)`) による。各実行の前に `.fixlang` を消した。
