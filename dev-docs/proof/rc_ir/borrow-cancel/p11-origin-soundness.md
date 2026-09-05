# P3 / P4 -- `origin` の健全性

この文書が読んだコードのコミットは `9f5599bb19325ccce1226a1aae5986823c526fa4` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で、この文書が `CODE` で引く
ファイル (`src/rc_ir/ownership.rs`、`src/rc_ir/leaf_map.rs`、`src/rc_ir/provenance.rs`、
`src/rc_ir/borrow.rs`、`src/rc_ir/codegen.rs`、`src/generator.rs`、`src/ast/types.rs`、
`src/ast/inline_llvm.rs`、`src/fixstd/builtin.rs`、`src/fixstd/runtime.rs`、
`src/parse/sourcefile.rs`、`src/misc.rs`、`src/constants.rs`、`src/error.rs`) に変わったのは
`// PROOF:` コメントだけである。
**この一覧は本文の `CODE` の行を数え上げて作る** -- 手で並べた一覧は、証明が新しいファイルを引くたびに
落ちる。
定義・仮定・命題の番号は同ディレクトリの `README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P3 (`origin` の健全性 -- `Exactly`) | 証明した (第 6 節の系 1)。参照の同一は README の P3 と同じくスロット `(x, λ)` が D8 の意味の参照を持つ場合に限り、オブジェクトの同一はその条件なしに、位置 (D6) の上で立つ。前提の「`origin(x, π)` が呼ばれる」は、README の P3 が言明の先頭に置く「**解析がその鍵で `origin` を呼び**」である (系 1 の脇) |
| P4 (`origin` の健全性 -- `Join`) | 証明した (第 6 節の系 2)。同じ 3 つの節が付く。さらに系 2 は、README の P4 が「いずれか」と述べる候補を 1 つに決める (系 2 の脇) |

P3 と P4 は、1 つのL17 (第 6 節) の 2 通りの読みである。L17 は `origin` が辿る再帰の辺を 1 本ずつ D9 の
移動の表と A3 の宣言に突き合わせる帰納法で示す。

- 第 1 節が、この文書が固定する本体と `VarTable`、DEF-0、および 10 の外部の結果 (`EXT`) -- `RcVar` が
  実行路の 1 つの位置で持つ値を 3 つの場合に分けたのが DEF-0 であり、D6 のスロットが
  在るのはそのうち 2 つの場合である。節点が束縛する変数について、その束縛の D2 のスコープの根の節点を
  **授与位置**と呼び、束縛の 4 つの形ごとに名指す。
- 第 2 節が `DEF 再帰の辺` と L6 (`origin_inner` の再帰の辺と、各辺が読む D9 の移動の行)、
  第 2.1 節が L8 (`Llvm` の腕が答えるもの)。
- 第 3 節が L9 (`origin_inner` が `Exactly((var, path))` を答える道と、その道が D10 の生成の表のどこに
  当たるか)。
- 第 4 節が L1 から L5、L10 (変数に値を与える構文と、値が束縛の後 変わらないこと)、L11 (再帰の辺の
  行き先の `RcVar` も値を持つこと)、L12 (値の leaf が参照を持つのは計数下のオブジェクトを指すときで
  あること)、`DEF 値からの到達` と L13 (束縛を持たない名前の値はグローバル状態のオブジェクトだけを
  指すこと)、L15 (この文書が `origin` に問う鍵はすべて P2 の範囲にあること)、L16 (鍵ごとの答えが
  1 つに決まること)、L14 (`origin` の
  再帰呼び出しの鍵の関係が (a) 整礎であることと、(b) 辺の先の鍵についても `origin` が呼ばれること)。
- 第 5 節が DEF-1 -- D17 の「対応するスロット」を、L17 の帰納法が辿る鎖の形に書き直したもの。
- 第 6 節がL17 と、その 3 つの系。系 1 が P3、系 2 が P4、系 3 が「DEF-1 の鎖は D33 の `ρ` 歩みで
  ある」である。

第 7 節に、この 2 つの命題の外にある観察を 1 つ置く -- `origin` は 1 つの値の unit の path と leaf の
path に別の答えを与え、leaf の側の `identity` が unit の側の答えに現れないことがある。第 8 節は
`level_ownership` がこの 2 つの命題の真偽を動かさないことである。

## 1. 記法

**この文書は 1 つの本体とその `VarTable` を固定する。** `B` を、`borrow_ify` の入力プログラムの 1 つの
本体 (D23) -- ある関数 `f` の `body` か、あるグローバル初期化子 `g` の `init` -- とする。`type_env` を
そのプログラムの `TypeEnv`、`vars` を `B` について作られた `VarTable` -- `B` が `f.body` なら
`VarTable::of(f)`、`B` が `g.init` なら `VarTable::body_only(&g.init)` -- とする。
**この 2 つが `VarTable` を作る形の全体である** -- `VarTable` の値を組み立てる式の在りかと
`VarTable::empty` を呼ぶ式の在りかは、この節の前提が走査で挙げる
(`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`, `VarTable::empty`)。

**`borrow_ify` の入力に固定するのは、この文書が引く仮定と命題の範囲がそこだからである。** A6 と A13 は
`borrow_ify` の入力の名前についての仮定であり、A11 のスコープの規律も同じ入力にかかる。P2a の `vars` は
「**`vars` は、A6 と A11 を満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が
作った表である。**」に限られており、`B` を入力の本体に取ると、A6 と A11 がその制限を満たし、上の 2 つの
構成子がこの表を作る。**`borrow_ify` の出力について P3 と P4 を読む者は、README の A6 の脇が定めるとおり
P9 と合わせて読む** -- 「層 1 の証明は依存の順で P9 を引けないので、A6 を読む段は入力について読み、
出力について読む者は P9 と合わせて読む。」P3 と P4 は層 1 の命題であり、P9 は番号が大きいのでこの文書は
引けない。`cancel` が扱うのは `borrow_ify` の出力なので、その側の読み手はこの 2 段を経る。

**以下、「本体」と書けば `B` を指す。**活性化 `α` (D21)、それが辿る実行路 `ρ` (D3)、その上の位置 `P` は
いずれも `B` のものであり、DEF-0 の 3 つの場合も、L10 から L16 も、L17 も、この 1 つの `B` と
`vars` の上で読む。**この固定が無いと `vars.bindings` と `vars.var_tys` の主語が定まらず、L10 (a')、
L11、L13 が当たらない。**

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`VarPath` を `(x, π)` と書く。
`ty(x)` は `x` に束縛された値の型 (D6) である。`x` が `B` の束縛変数であるとき、`vars.var_tys` が
それを記録する -- パラメータと capture、および `Let`、`Destructure`、`Match` のアーム payload が束縛する
変数のすべてについて `var_tys` に型が入る (`CODE src/rc_ir/ownership.rs: VarTable::of`, `collect_bindings`,
`CODE src/rc_ir/ownership.rs: VarTable` の `var_tys` フィールド)。`x` が束縛を持たない名前のとき、
`ty(x)` はその `RcVar` の `ty` の欄であり、A12 よりそれはその名前の記号の型である。

- `leaves(τ)` は `boxed_leaf_paths(τ)`、`leaves(τ, π)` は `π` で始まる `leaves(τ)` の要素とする
  (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `LeafMap::leaves_under`)。
- `t_τ(p)` は `truncate_to_unit(τ, p)` とする (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。
- `id(v, π)` は `origin(v, π).identity()`、`cand(v, π)` は `origin(v, π).candidates()` の集合、
  `act(v, π)` は `origin(v, π).acted_on()` の集合とする。
- `p ⊒ q` は「`p` が `q` を接頭辞として持つ」とする。
- `α` は 1 つの活性化 (D21)、`ρ` は `α` が辿る実行路 (D21) とする。D21 の約束により、実行路について述べる
  言明は、その路を辿るすべての活性化についての言明として読む。
- `origin(v, q)` という記法は、鍵 `(v, q)` の**答え** -- L16 が 1 つに定める値 -- を指す。

**`origin` の呼び出しが panic せずに 1 つの値を返すことは、第 4 節の L15 と L16 が示す。** L15 が
`assert!`・`assert_eq!`・`panic!` の発火しないことを、L16 が答えが鍵ごとに 1 つであることを述べる。
`origin_inner` の腕が「再帰呼び出しの返り値をそのまま返す」形をしているとき、L16 の読み替えで鍵の答え
どうしの等式が得られる。読み替えを行う段は `BY` に L16 を挙げる。

**DEF-0 (`RcVar` が位置 `P` で持つ値)**。活性化 `α`、それが辿る実行路 `ρ`、`ρ` 上の位置 `P` を固定する。
本体に現れる `RcVar` `v` が `P` で**値を持つ**とは、次の 3 つのいずれかである。

- **(v-1)** `v` が本体の節点が束縛する変数であり、`ρ` が `v` の**授与位置**を `P` までに (`P` 自身を
  含めて) 通っていること。その値はその束縛が与えた値である。**授与位置**とは、その束縛の D2 の意味の
  スコープの**根の節点**であり、束縛の形ごとに次の 4 行になる。

  | 束縛 | D2 のスコープ | `v` の授与位置 |
  |---|---|---|
  | `Let(v, rhs, k)` で `rhs` が `Match` でない | `k` の部分木 | `k` の根の節点 |
  | `Let(v, Match(s, arms), k)` | `k` の部分木 | `k` の根の節点。`ρ` はそこへ進む前に `α` が選んだアーム本体の実行路を辿り終える (D3) |
  | `Destructure(c, fs, s, k)` が束縛する `fs` の変数 | `k` の部分木 | `k` の根の節点 |
  | `Match` のアーム `A` の `payload` | `A` の `body` の部分木 | `A` の `body` の根の節点。`α` が `A` を選ばなければ `ρ` はそこを通らない |

  **この 4 行は D6 の「その実行路の上でその時点までに値を得た変数」を節点の水準に書き下したものである。**
  4 行が L10 (a) の 3 構文を尽くすこと、および授与位置で `v` がその束縛の値を持つことは L10 (c) が
  述べる。第 2 行の括弧書きが要るのは、D3 が `Let(x, Match(v, arms), k)` でアーム本体の実行路を先に
  辿り、その後 `k` へ進むからである。**アームの中の位置は授与位置より前にあるので、この定義はそこで
  `v` を値を持つものに数えない。**

  **`P` 自身がスコープに在ることは要求しない。** D6 が `x` を「その時点で束縛されている変数」に
  限らないと述べ、`Let(m, Match(s, [Let(a, App(f, []), Ret(a))]), k)` の `a` を `k` の位置で名指す例を
  挙げているのがこの広さである。条件は、授与位置を `ρ` が `P` までに通ったかどうかだけである。
- **(v-2)** `v` がその本体の関数のパラメータか capture であること。その値は活性化の入力の束縛 (D23) が
  与える値である。**この場合は `P` に条件を課さない。** その置き方が D6 の「その時点までに値を得た
  変数」に当たることと、この場合が `B` がグローバル初期化子の `init` であるとき空であることは、
  L10 (e) が述べる。
- **(v-3)** `v` の名前が `vars.bindings` に束縛を持たず、かつ `ρ` が `v` を名指す節点を `P` までに
  (`P` 自身を含めて) 通っていること。その名前は最上位の記号の名前であり
  (D6 の「**束縛を持たない名前は、必ず最上位の記号の名前である。**」)、その値はその記号の値である。
  A12 の「束縛を持たない `RcVar` の型が、その名前の記号の型であること」がその型を与える。

  **節点を通ったことを条件に置くのは、D6 が「記号の位置が値を持つのは、その記号のグローバル化の段 (E5)
  より後の時点である」と定めるからである。** この条件から `P` がその記号の (E5) の段より後にあることは
  L10 (e) が述べる。関数の名前については記号の位置は funptr を指し、初期化の段を持たない (D6) が、
  DEF-0 は 2 つを分けずに同じ条件を課す -- その場合を扱うのは L10 (d) の funptr の枝である。
  L13 と L11 がこの条件を読む。

`v` が (v-1) か (v-2) であり `λ` がその値の `P` で inhabited な boxed leaf であるとき、`(v, λ)` を
`P` のスロットと呼び、`obj(v, λ)` をその leaf が指すオブジェクトと書く。(v-3) の名前について D6 は
「**スロットではない**」と述べ、その対を**記号の位置**と呼ぶ。(v-1) と (v-2) が D6 のスロットを、
(v-3) が D6 の記号の位置を与えることは L10 (e) が述べる。
L13 が、(v-3) の値の leaf は D8 の意味の参照を持たないことを述べ、L17 の (iii) はその形で (v-3) を
排除する。

この 3 つが尽きており互いに排他であることは、L10 (e3) が述べる。その排他性が、(v-1) と (v-2) が
名指す値を 1 つに決める。

`Origin` の構成子と読み出しは次のとおりである (`CODE src/rc_ir/ownership.rs: Origin`, `Origin::identity`,
`Origin::candidates`, `Origin::acted_on`, `Origin::of_candidates`)。

- `Exactly(p)`: `identity() = p`、`candidates() = [p]`、`acted_on() = [p]`。
- `Join { identity, candidates }`: `identity() = identity`、`candidates() = candidates`、
  `acted_on() = [identity] ++ (candidates` から `identity` を除いたもの`)`。集合として
  `acted_on() = {identity} ∪ candidates` である。
- `of_candidates(C, h)`: `C` が 1 要素ならその要素の `Exactly`、2 要素以上なら
  `Join { identity: h, candidates: C }`。`C` が空なら panic する。

`origin_inner` の `Binding::Join` の腕と `origin_from_leaves_under` の末尾は、内側の `Origin` を畳んで
候補集合を作るとき、その `acted_on()` を集める (`CODE src/rc_ir/ownership.rs: origin_inner` の
`Some(Binding::Join(..))` の腕、`origin_from_leaves_under` の
`let candidates = reached.iter().flat_map(..)`)。よって内側が `Join` のとき、その `identity` は外側の
候補集合に入る。

**外部の結果。** README の第 2 節は、文書の外の名前つき結果を `EXT <名前>` の名札で第 1 節に据え、
`BY` からその名前で引くことを求める。この文書が引くのは次の 11 である。

**EXT auto trait と共有** (Rust の言語規則)。

1. `RefCell<T>` は `Sync` を実装しない。
2. `Sync` は auto trait であり、構造体がそれを実装するのは各欄の型がそれを実装するときに限る。
   手で `unsafe impl Sync` を書いた型はこの限りでない。
3. `&T` が `Send` を実装することと `T` が `Sync` を実装することは同値である。
4. 1 つの値は各時点でちょうど 1 つの所有者を持つ。値がスレッドの間を渡るとき、渡す動作は、渡す前の
   アクセスと渡した後のアクセスを順序づける。
5. `&T` が `Send` でない型 `T` については、2 つのスレッドが 1 つの `T` の値への共有参照を同時に持つ
   ことはない。したがって、その値に対して共有参照を通じて行われる動作は 2 つのスレッドで重ならず、
   時間で全順序に並ぶ。

**EXT 呼び出しの入れ子**。関数の 1 つの実行が別の関数を呼ぶとき、その呼び出しは呼び出し元の実行が
返るより前に返る。すなわち入れ子の呼び出しは後入れ先出しの順に返る。

**EXT `pthread_once`** (POSIX)。1 つの `pthread_once_t` の値について、`pthread_once` に渡された
初期化ルーチンは、そのプロセスの実行の中でちょうど 1 度だけ呼ばれる。`pthread_once` の呼び出しは、
その初期化ルーチンが終わってから返る。

**EXT 導出した Clone** (Rust の言語規則)。`#[derive(Clone)]` が与える `clone` は、列挙型については
同じ構成子の値を返し、各欄にその型の `clone` が返す複製を置く。

**EXT 標準ライブラリのハッシュ** (Rust)。(1) `impl<T: Hash + ?Sized> Hash for Arc<T>` の `hash` は、
指す先の `T` の `hash` を呼ぶ。(2) `HashMap::get(k)` は鍵 `k` の `Hash` の実装を走らせて索く。

**EXT 内部可変性** (Rust)。(1) `RefCell<T>` の中身は、共有参照 `&RefCell<T>` から `borrow_mut` で
書き替えられる。(2) `OnceLock<T>::get_or_init(&self, f)` は、欄がまだ空なら共有参照 `&self` を通じて
`f()` の値でそれを埋め、以後の呼び出しは同じ値を返す。欄は一度だけ書かれる。

**EXT 整礎性**。(a) 自然数の狭義減少する無限列は無い。(b) ある集合の上の関係が、その関係を辿って
無限に降りる列を 1 つも持たないとき、その関係は整礎であり、その上の整礎帰納が使える。

**EXT 動作の番号づけ**。1 つのプロセスの実行は開始の時点を持ち、その実行の各動作の前に在る動作は
有限個である。よって時間で全順序に並ぶ動作の族は、その順序を保ったまま自然数で番号づけられる。

**EXT 借用規則** (Rust の言語規則)。共有参照 `&T` を通じて `T` の値へ書き込むことはできない。内部
可変性を持つ欄 (`Cell`、`RefCell`、`OnceLock` など) だけがその例外であり、それ以外の欄の値は共有参照を
受け取った計算のあいだ動かない。

**EXT 可視性** (Rust の言語規則)。`pub` の付かない項目 (関数・欄) を名指せるのは、それを定義する
モジュールとその子孫のモジュールだけである。よってそのような項目の呼び出しと欄への書き込みは、その
モジュールの中を数え上げれば尽きる。

**EXT panic** (Rust の言語規則)。`panic!` は panic を起こす。`Option::expect` は値が `None` のとき、
`unreachable!` は制御がそこへ届いたとき、いずれも同じく panic を起こす。panic を起こした式は値を
返さず、その先の動作を行わない。

### 在りかの前提

**コードのどこに何が在るかの数え上げは、段の中で行わない。** 段が自分で在りかを数え上げると、その
数え上げには果たす者が居らず、検査するものも無い。**記号を名指す `CODE` の引用はその記号の本体しか
与えないので、「ほかの記号はそれをしない」の側はそこから出ない。** 在りかは名前つきの前提として置き、
`BY` の行ではその名前で引く。**個数は書かない** -- 一覧が在れば個数は一覧の長さである。

**果たすのは走査である。** 在りかを走らせられる字面で書き、`dev-docs/proof/proof_links.py` がその字面を
`src/` の全体に走らせて、下の一覧と突き合わせる。挙がった各項目が何であるかは `--` の後に書く。走査は
字面の上位近似なので、一覧には構成でなくパターンとしてその字面を持つ項目も入る。`#[cfg(test)]` の下の
項目は走査が除く。項目の名前は走査が呼ぶ名前である -- 自由関数がその直前の `impl` の名前を冠して
挙がる形を含む。

**前提 `result_prov` の本体の在りか** --- `LLVMGen::result_prov` の本体が在る項目は次で尽きる。
**そのどれもが、返す `Provenance` を `Provenance::uniform`・`Provenance::build_shape`・
`Provenance::uniform_bottom`・`Provenance::fresh_under`・`replaced_field_prov` のいずれかを第 1 引数
`result_ty` に対して呼んで作る。** どれを呼ぶかは `--` の後に書く。

SCAN src/ `fn result_prov`
  = src/ast/inline_llvm.rs: result_prov -- trait の既定の本体。`Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMStringBuf::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayUnsafeEmpty::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayTruncateBoundsUnchecked::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayAppendValueCapacityUnchecked::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArraySetCapacityBoundsUnchecked::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayAppendCapacityUnchecked::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayGrowSizeBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArraySetBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArraySwapBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayPunchBody::result_prov -- `Provenance::fresh_under`
  = src/fixstd/builtin.rs: InlineLLVMPunchedArrayPlugBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov -- 容器が boxed なら `Provenance::uniform`、unbox なら `Provenance::build_shape`
  = src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov -- `Provenance::build_shape`
  = src/fixstd/builtin.rs: InlineLLVMArrayLitBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMStructPunchBody::result_prov -- 穴の開いた成分が boxed なら `Provenance::fresh_under`、unbox なら `Provenance::build_shape`
  = src/fixstd/builtin.rs: InlineLLVMStructPlugInBody::result_prov -- `replaced_field_prov`
  = src/fixstd/builtin.rs: InlineLLVMStructSetBody::result_prov -- `replaced_field_prov`
  = src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov -- `Provenance::build_shape`
  = src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov -- scrutinee が boxed なら `Provenance::uniform`、unbox なら `Provenance::build_shape`
  = src/fixstd/builtin.rs: InlineLLVMUndefinedInternalBody::result_prov -- `Provenance::uniform_bottom`
  = src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMArrayIsStorageUniqueBody::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedInternalFunctionBody::result_prov -- `Provenance::fresh_under`
  = src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedIOSInternalBody::result_prov -- `Provenance::fresh_under`
  = src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsInternalBody::result_prov -- `Provenance::fresh_under`
  = src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsIosInternalBody::result_prov -- `Provenance::fresh_under`
  = src/fixstd/builtin.rs: InlineLLVMDestructorMake::result_prov -- `Provenance::uniform`
  = src/fixstd/builtin.rs: InlineLLVMMarkThreadedFunctionBody::result_prov -- `Provenance::uniform`

**前提 `Origin::Exactly` を作る式の在りか** --- `Origin::Exactly` の字面が在る項目は次で尽きる。

SCAN src/ `Origin::Exactly(`
  = src/rc_ir/ownership.rs: Origin::identity -- `match` のパターンであって構成ではない
  = src/rc_ir/ownership.rs: Origin::candidates -- `match` のパターンであって構成ではない
  = src/rc_ir/ownership.rs: Origin::of_candidates -- `1 =>` の腕が構成する
  = src/rc_ir/ownership.rs: origin_inner -- 閉包 `here` が構成する
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- `reached.push(Origin::Exactly(here.clone()))`

**前提 `Origin` の `Join` 変位を作る式の在りか** --- `Origin` の `Join` 変位を値として作る式は、
どの綴りで書いてもその字面に変位の名前 `Join` を含む。**変位の名前で走査するのは、型の名前を
別名にした import や、変位を直に取り込んだ import が、型の名前で走査すると落ちるからである。**
`Join` の字面が在る項目は次で尽きる。

SCAN src/ `Join`
  = src/ast/program.rs: Program::linked_mods -- doc の英語 (「Joins `other` into this program」)
  = src/elaboration/name_resolution.rs: NameResolutionContext::create_ambiguous_message -- コメントの英語
  = src/fixstd/std.fix: Iterator::Item -- Fix のソースのコメントの英語
  = src/misc.rs: spawn_compiler_thread -- `std::thread::JoinHandle`
  = src/misc.rs: join_compiler_threads -- `std::thread::JoinHandle`
  = src/printer.rs: Text::add_indent -- コメントの英語
  = src/rc_ir/locality.rs: Locality -- doc の英語
  = src/rc_ir/locality.rs: ExtCond::atom -- doc の英語
  = src/rc_ir/ownership.rs: Binding -- `Binding` の `Join` 変位の宣言
  = src/rc_ir/ownership.rs: collect_bindings -- `Binding::Join` の構成
  = src/rc_ir/ownership.rs: Origin -- `Origin` の `Join` 変位の宣言
  = src/rc_ir/ownership.rs: Origin::identity -- `Origin::Join` は `match` のパターンであって構成ではない
  = src/rc_ir/ownership.rs: Origin::candidates -- `Origin::Join` は `match` のパターンであって構成ではない
  = src/rc_ir/ownership.rs: Origin::of_candidates -- `_ =>` の腕が `Origin::Join { .. }` を構成する
  = src/rc_ir/ownership.rs: Origin::acted_on -- 自由関数 `origin` の doc の英語
  = src/rc_ir/ownership.rs: origin_inner -- `Binding::Join` のパターンと、自由関数 `origin_from_leaves_under` の doc の英語
  = src/tests/test_lsp/lsp_client.rs: LspClient::new -- コメントの英語

**前提 `of_candidates` を呼ぶ式の在りか** --- `of_candidates` の字面が在る項目は次で尽きる。

SCAN src/ `of_candidates(`
  = src/rc_ir/ownership.rs: Origin::of_candidates -- 定義
  = src/rc_ir/ownership.rs: origin_inner -- `Some(Binding::Join(..))` の腕の呼び出し
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- 末尾の呼び出し

**前提 `origin_inner` を呼ぶ式の在りか** --- `origin_inner` の字面が在る項目は次で尽きる。

SCAN src/ `origin_inner(`
  = src/rc_ir/ownership.rs: origin -- `grow_stack(|| origin_inner(..))` の呼び出し
  = src/rc_ir/ownership.rs: origin_inner -- 定義

**前提 `VarTable` を組み立てる式の在りか** --- `VarTable` の値を組み立てる式はその字面に
`VarTable {` を含む。その字面が在る項目は次で尽きる。

SCAN src/ `VarTable {`
  = src/rc_ir/ownership.rs: VarTable -- 型の宣言
  = src/rc_ir/ownership.rs: VarTable::of -- 返り値の型の綴り
  = src/rc_ir/ownership.rs: VarTable::body_only -- 返り値の型の綴り
  = src/rc_ir/ownership.rs: VarTable::empty -- 5 欄をすべて空で置く構成

**前提 `VarTable::empty` を呼ぶ式の在りか** --- `VarTable::empty` の字面が在る項目は次で尽きる
(走査は `#[cfg(test)]` の下の項目を除く)。

SCAN src/ `VarTable::empty(`
  = src/rc_ir/ownership.rs: VarTable::of -- 呼び出し
  = src/rc_ir/ownership.rs: VarTable::body_only -- 呼び出し

**前提 `VarTable` の `bindings` の欄に触れる式の在りか** --- その欄は `pub` を持たないので、それを
名指せるのは `src/rc_ir/ownership.rs` とその子孫のモジュールだけである (`EXT 可視性`)。欄アクセスの
字面が在る項目は次で尽き、その欄を持つ `VarTable` の値を組み立てるのは `VarTable::empty` である
(前提 `VarTable` を組み立てる式の在りか)。走査は字面の上位近似なので、一覧には
`src/rc_ir/provenance.rs` の `Interpreter` が持つ同じ名前の別の欄も入る。

SCAN src/ `.bindings`
  = src/build/build_object_files.rs: dump_rc_ir -- `analyze_program(..).bindings`、`Provenance` の解析の欄
  = src/rc_ir/ownership.rs: VarTable::of -- パラメータと capture に `Binding::Param` を入れる
  = src/rc_ir/ownership.rs: collect_bindings -- 節点が束縛する変数に `Binding` を入れる
  = src/rc_ir/ownership.rs: origin_inner -- `vars.bindings.get(var)` の読み
  = src/rc_ir/provenance.rs: Interpreter::record -- `Interpreter` の欄
  = src/rc_ir/provenance.rs: Interpreter::refine_by_unique_flag -- `Interpreter` の欄
  = src/rc_ir/provenance.rs: analyze_program -- `Interpreter` の欄

**前提 `declared_globals` の欄に触れる式の在りか** --- その欄は `pub` を持たないので、それを
名指せるのは `src/generator.rs` とその子孫のモジュールだけである (`EXT 可視性`)。その字面が在る
項目は次で尽きる。

SCAN src/ `declared_globals`
  = src/generator.rs: Generator -- 欄の宣言
  = src/generator.rs: Generator::new -- 空の表を置く
  = src/generator.rs: Generator::add_global_object -- 表へ項目を入れる
  = src/generator.rs: Generator::get_or_declare_global -- 表を名前で引く
  = src/generator.rs: Object::ptr_to_field_as -- 直後の項目の doc が `Generator::declared_globals` を名指す

**前提 `add_global_object` を呼ぶ式の在りか** --- `add_global_object` の字面が在る項目は次で尽きる。

SCAN src/ `add_global_object(`
  = src/generator.rs: Generator::add_global_object -- 定義
  = src/generator.rs: Generator::declare_program_global -- アクセサの枝の呼び出し
  = src/generator.rs: Generator::declare_lambda_function -- funptr の枝の呼び出し

**前提 記号の記憶域の番地が現れる式の在りか** --- 記号の記憶域 `GlobalVar#<symbol>` の番地を持つ
局所変数は `global_var_ptr` であり、その字面が在る項目は次で尽きる。

SCAN src/ `global_var_ptr`
  = src/rc_ir/codegen.rs: Generator::implement_rc_global -- 番地を作り、`store_init_value` と末尾の `build_load` に渡す
  = src/rc_ir/codegen.rs: Generator::store_init_value -- 受け取った番地へ `build_store` する
  = src/rc_ir/codegen.rs: Generator::ACCESSES_PER_INITIALIZATION -- 直後の項目の doc が名指す

**前提 初期化済みの旗の番地が現れる式の在りか** --- 旗 `InitFlag#<symbol>` の番地を持つ局所変数は
`init_flag_ptr` であり、その字面が在る項目は次で尽きる。

SCAN src/ `init_flag_ptr`
  = src/rc_ir/codegen.rs: Generator::implement_rc_global -- 旗のロード、旗への store、`pthread_once` へ渡す引数

**前提 `origin` を呼ぶ式の在りか** --- `origin` の呼び出しの字面が在る項目は次で尽きる。
`borrow_ify` と `cancel` が住む `src/rc_ir/borrow.rs` の側を読むのは第 7 節である。

SCAN src/rc_ir/ `origin(`
  = src/rc_ir/borrow.rs: infer_ownership -- 呼び出し
  = src/rc_ir/borrow.rs: level_ownership -- 呼び出し
  = src/rc_ir/borrow.rs: RewriteCtx::owns_unit -- 呼び出し
  = src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled -- 呼び出し
  = src/rc_ir/borrow.rs: RewriteCtx::comes_from_a_value_used_later -- 呼び出し
  = src/rc_ir/borrow.rs: CancelAnalysis::consume -- 呼び出し
  = src/rc_ir/borrow.rs: CancelAnalysis::other_objects -- 呼び出し
  = src/rc_ir/ownership.rs: origin -- 定義
  = src/rc_ir/ownership.rs: origin_inner -- 再帰呼び出し
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- 再帰呼び出し
  = src/rc_ir/ownership.rs: acted_references -- 呼び出し
  = src/rc_ir/provenance.rs: Provenance::uniform -- `sole_origin(` の字面
  = src/rc_ir/provenance.rs: Provenance::set_leaves_under -- `sole_origin(` の字面
  = src/rc_ir/provenance.rs: Provenance::arg_passthrough -- `sole_origin(` の字面
  = src/rc_ir/provenance.rs: sole_origin -- `sole_origin(` の字面 (定義)

**前提 `get_scoped_value` を呼ぶ式の在りか** --- `get_scoped_value` の字面が在る項目は次で尽きる。

SCAN src/ `get_scoped_value(`
  = src/generator.rs: Generator::get_scoped_value -- 定義
  = src/generator.rs: Generator::get_scoped_obj -- 呼び出し
  = src/generator.rs: Generator::get_scoped_obj_noretain -- 呼び出し

**前提 `build_capture_project` を呼ぶ式の在りか** --- `build_capture_project` の字面が在る項目は
次で尽きる。

SCAN src/ `build_capture_project(`
  = src/generator.rs: Generator::build_capture_project -- 定義
  = src/fixstd/builtin.rs: InlineLLVMCaptureProjectBody::generate -- 呼び出し

**前提 `unsafe impl` の在りか** --- `unsafe impl` の字面が在る項目は無い。よって `Send` と `Sync` を
手で実装した型はこのクレートに無く、`EXT auto trait と共有` の 2 の但し書きに当たる型も無い。

SCAN src/ `unsafe impl`

**前提 `VarTable` の `origins` の欄に触れる式の在りか** --- その欄は `pub` を持たないので、それを
名指せるのは `src/rc_ir/ownership.rs` とその子孫のモジュールだけである (`EXT 可視性`)。欄アクセスの
字面が在る項目は次で尽き、その欄を持つ `VarTable` の値を組み立てるのは `VarTable::empty` である。

SCAN src/ `.origins`
  = src/rc_ir/ownership.rs: origin -- 自由関数 `origin` が直前の `impl Origin` の名前を冠して挙がる。`borrow()` の読みと `borrow_mut().insert(..)` の書きの 2 行

## 2. D9 の「移動」と `origin_inner` の再帰の辺 <!--#747d82d-->

**DEF 再帰の辺**。鍵 `(v, q)` から鍵 `(v', q')` への**再帰の辺**とは、
`origin_inner(vars, type_env, v, q)` の実行が `origin(vars, type_env, v', q')` を呼ぶことをいう
(`CODE src/rc_ir/ownership.rs: origin_inner`)。これは `origin` の再帰呼び出しの上の関係である。
**D20 の別名の辺は位置 (D6) の間の関係であり、その存在はその辺を定める節点が実行路の上に在ることと、
両端がその路の位置であることに条件づけられている** (D20)。2 つの関係の対応は L17 の (iv) と 系 3 が
述べる。

**L6 (`origin_inner` の再帰の辺)**: 再帰の辺は次の第 1 の表の E1 から E7 で尽きている。各辺が辿る先へ <!--#9357e31-->
参照と値を渡す構文を述べる D9 の移動の表の行は、第 2 の表のとおりである。**E3 と E4 はどちらも
D9 の `Llvm` の素通し leaf の行の下にある** -- E3 は問うた path 自身が単一の `Arg` を宣言された
leaf である場合、E4 はその path の下の leaf が単一の `Arg` を宣言されている場合であり、どちらもその行が
述べる素通しである。**E4 の側で宣言が単一であることは A3 が与える** -- コードは宣言の集合の各元を辿るので
要素数 2 以上の宣言を持つ leaf も辿りうる形をしているが、A3 の「**複数の元を宣言する op は存在しない。**」が
それを排除する。

| 辺 | 腕 | 行き先 |
|---|---|---|
| E1 | `Move(y)` | `origin(y, π)` |
| E2 | `Join(rs)` | 各 `r` in `rs` について `origin(r, π)` |
| E3 | `Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)` | `origin(args[j], σ)` |
| E4 | `Llvm` かつ E3 でない | `π` の下の各 leaf の宣言の各 `Arg(j, σ')` について `origin(args[j], t_{ty(args[j])}(σ'))` |
| E5 | `Field(c, i)` かつ `c` が unbox | `origin(c, [i] ++ π)` |
| E6 | `Payload(s, None)` | `origin(s, π)` |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `origin(s, [t] ++ π)` |

| 辺 | その辺が辿る先へ参照と値を渡す D9 の移動の表の行 |
|---|---|
| E1 | `Let(x, Var(y), k)` |
| E2 | `Match` のアーム本体の `Ret(x)` |
| E3 | `Llvm` の素通し leaf (`result_prov` が単一の `Arg(i, σ)`) |
| E4 | `Llvm` の素通し leaf (同じ行を leaf ごとに読む。宣言が単一であることは A3 が与える) |
| E5 | unbox 容器の `Destructure` の名前付きフィールド |
| E6 | catch-all アームの payload 束縛 |
| E7 | unbox union の変位アームの payload 束縛 |

<1>1. `origin_inner` の `match` の腕は 6 本であり、`vars.bindings.get(var)` が返しうる 8 つの場合
      (`None` と `Binding` の 7 構成子) を尽くしている。
  <2>1. `vars.bindings.get(var)` は `Option<&Binding>` を返す。
    BY CODE src/rc_ir/ownership.rs: VarTable の `bindings` フィールド
  <2>2. `Binding` の構成子は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つで
        ある。
    BY CODE src/rc_ir/ownership.rs: Binding
  <2>3. `match` の腕は 6 本である -- `None | Some(Param) | Some(Producer)` を束ねた 1 本、
        `Some(Move(y))`、`Some(Join(arm_results))`、`Some(Llvm(..))`、`Some(Field(container, idx))`、
        `Some(Payload(scrut, variant))`。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. QED
    BY <2>1, <2>2, <2>3 -- 8 つの場合が 6 本の腕に分かれ、`None`、`Param`、`Producer` の 3 つが 1 本目に
       束ねられている。

<1>2. 再帰の辺は第 1 の表の E1 から E7 で尽きている。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner -- `origin` を再帰呼び出しするのは `Move` の腕、
     `Join` の腕、`Llvm` の腕の 2 つの枝、`Field` の `else` の枝、`Payload` の `None` の枝と
     `Some(tag) if !scrut.ty.is_box(type_env)` の枝である。残る枝 (`None | Param | Producer` の腕、
     `Field` の `is_box` の枝、`Payload` の `Some(_)` の枝) は `here()` を返して再帰呼び出しをしない。
     `Llvm` の腕の 2 つの枝が E3 と E4 であり、E4 の中身は `origin_from_leaves_under` である
     (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

<1>2c. 第 2 の表の各行は、その辺が辿る先へ参照と値を渡す構文を述べる D9 の移動の表の行を名指して
       いる。
  BY <ref id=9d74736/> の移動の表, <ref id=f06144e/> の生成の表, <ref id=e11772a/>, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `RcRhs::Var(y)` に `Binding::Move(y)` を作るので、
     `Binding::Move(y)` を持つ変数を束縛する構文は `Let(x, Var(y), k)` であり、E1 が辿る先はその `y` で
     ある。`RcRhs::Match` の腕に `Binding::Join(arm_results)` を作り、`arm_results` は各アーム
     本体の `returned_var` である (`CODE src/rc_ir/ownership.rs: returned_var` -- 本体の終端の `Ret` が
     名指す変数) ので、E2 が辿る先はアーム本体の `Ret` が名指す変数である。
     `RcExpr::Destructure` の腕は名前付きフィールドにだけ `Binding::Field` を作るので、E5 が辿る先は
     unbox 容器のその名前付きフィールドである。`Match` の各アームの payload に
     `Binding::Payload(scrut, arm.tag)` を作り、`tag` が `None` のアームが catch-all、`Some(t)` の
     アームが変位アームである (<ref id=b3dfa37/> の `MatchArm` の `tag`) ので、E6 が辿る先は catch-all アームの
     scrutinee、E7 が辿る先は変位アームの scrutinee の変位 `t` である。
     E3 と E4 は `Binding::Llvm` の腕の 2 つの枝である。E3 は `as_arg_projection` が問うた path 自身の
     宣言について集合の要素数 1 と `Arg` を要求する枝であり
     (`CODE src/rc_ir/ownership.rs: as_arg_projection`)、その leaf は <ref id=9d74736/> の行が言う「単一の
     `Arg(i, σ)`」を宣言された素通し leaf である。E4 は `origin_from_leaves_under` が path の下の各 leaf の
     宣言の**各**元を辿る枝であり (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under` --
     `for sources in decl.leaf_origins_under(path)` の内側の `for src in sources` が、集合の元 1 つずつに
     ついて `operand_units` を積む)、単一でない宣言も辿りうる形をしている。<ref id=e11772a/> の「**複数の元を宣言する
     op は存在しない。**」より、このプログラムに現れるどの宣言も要素数は 0 か 1 であり、`Arg(j, σ')` を
     含む leaf の宣言はその 1 元だけからなる。よって E4 が辿る leaf も <ref id=9d74736/> の行が言う「単一の
     `Arg(i, σ)`」を宣言された素通し leaf である。
     **この節が要るのは、要素数 2 以上の宣言を持つ leaf が <ref id=9d74736/> の移動の行ではなく <ref id=f06144e/> の生成の `Llvm` の行
     (「`result_prov` の宣言が単一の `Arg(j, σ)` **でない**もの」) に当たるからである** -- 無いと
     E4 の辺は <ref id=9d74736/> の移動の表に行を持たない。

<1>3. QED
  BY <1>1, <1>2, <1>2c

E4 を leaf ごとに分解した形が、第 5 節の DEF-1 の段 E4a と停止条件 S2 である。その各段が D9 と A3 の
どの行に当たるかは第 6 節が述べる。E4 が答えを作る規則そのものの性質は L3 と L4 に置く。

### 2.1 `Llvm` の腕が答えるもの <!--#bc3628d-->

A3 は `result_prov` が leaf ごとに `LeafOrigins` (`Set<LeafOrigin>`) を返すとし、空集合・単一の
`Arg`・単一の `Fresh`・単一の `Unknown`・複数元の 5 行を持つ。`origin_inner` の `Llvm` の腕がその 5 つを
どう扱うかを書き下す。

**L8 (`Llvm` の腕が答えるもの)**: `x` の `Binding` が `Llvm(gen, args, ty(x))` であるとし、 <!--#9a6b1cd-->
`decl = gen.result_prov(ty(x), arg_tys, type_env)` とする。次が成り立つ。

- **(a)** `decl` が記録する leaf の集合は `leaves(ty(x))` そのものである。よって
  `decl.leaf_origins_at(p)` は、`p ∈ leaves(ty(x))` のときその leaf の `LeafOrigins` を `Some` で返し、
  そうでないとき `None` を返す。
- **(b)** `decl.leaf_origins_at(π)` の値は次の 5 つで尽きている。`None`、`Some` の空集合、`Some` の単一の
  `Arg(j, σ)`、`Some` の単一の `Fresh` または単一の `Unknown`、`Some` の要素数 2 以上。
- **(c)** 第 3 の場合 (`Some` の単一の `Arg(j, σ)`) には `origin(x, π) = origin(args[j], σ)` である
  (辺 E3)。
- **(d)** 残る 4 つの場合には、`origin(x, π)` は
  `origin_from_leaves_under(vars, type_env, &decl, args, π, &(x, π))` が返す値であり、それが `None` の
  ときは `Exactly((x, π))` である。この関数について次が成り立つ。
  - **(d1)** `π` の下のある leaf の宣言が `Fresh` または `Unknown` を含むとき、`produced_here` が真に
    なり、`Exactly((x, π))` は `reached` の要素である。
  - **(d2)** `π` の下の leaf の宣言がすべて空集合であるとき (`π` の下に leaf が 1 つも無いときを含む)、
    `reached` は空であり、`origin(x, π) = Exactly((x, π))` である。
  - **(d3)** `π` の下のある leaf の宣言が `Arg(j, σ')` を含むとき、
    `origin(args[j], t_{ty(args[j])}(σ'))` は `reached` の要素である (辺 E4)。

<1>0. `x` は `B` に現れる `RcVar` の名前であり、`args` の各要素も `B` に現れる `RcVar` である。
      よってこの証明が引く L16 の前提 -- 鍵の第 1 成分が `B` に現れる `RcVar` の名前であること --
      は、鍵 `(x, π)` についても鍵 `(args[j], ・)` についても満たされる。
  この命題の前提より `x` の `Binding` は `Llvm(..)` であって `Binding::Param` ではないので、
  L10 (a') より `x` は L10 (a) の 3 構文のいずれかが束縛する変数である。`Binding::Llvm` を作る
  のは `collect_bindings` の `RcExpr::Let` の腕の `RcRhs::Llvm(llvm_gen, args)` の枝であり、
  その `args` は `B` のその `Let` 節点の欄である。
  BY <ref id=9a6b1cd/> の前提 (`x` の `Binding` は `Llvm(gen, args, ty(x))` である), <ref id=49da857/> (a), <ref id=49da857/> (a'),
     CODE src/rc_ir/ownership.rs: Binding (`Param` と `Llvm` は相異なる構成子である),
     CODE src/rc_ir/ownership.rs: collect_bindings (`RcRhs::Llvm(llvm_gen, args)` の枝が
     `Binding::Llvm(llvm_gen.clone(), args.clone(), x.ty.clone())` を作る。その `args` は
     走査中の `Let` 節点の欄である)

<1>1. `decl` が記録する leaf の集合は `leaves(ty(x))` そのものであり、`decl.leaf_origins_at(p)` は
      `p ∈ leaves(ty(x))` のときその leaf の `LeafOrigins` を `Some` で返し、そうでないとき `None` を
      返す。
  <2>1. `Provenance` は `LeafMap<LeafOrigins>` の newtype であり、`LeafMap::build_shape(τ, type_env, f)`
        が作る `LeafMap` の鍵の集合は `boxed_leaf_paths(τ, type_env)` そのものである。
    BY CODE src/rc_ir/provenance.rs: Provenance (`Provenance(LeafMap<LeafOrigins>)`),
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape -- `boxed_leaf_paths(ty, type_env)` の各要素を
       鍵にして `collect` する
  <2>2. `result_prov` の呼び出しは値を返し (A3)、返す `Provenance` は、`Provenance::uniform`、
        `Provenance::build_shape`、`Provenance::uniform_bottom`、`Provenance::fresh_under`、
        `replaced_field_prov` のいずれかを `result_ty` に対して呼んだ値である。
        **在りかを与えるのは走査である** -- `LLVMGen::result_prov` の本体が在る項目を第 1 節の前提が
        挙げ、その各項目が 5 つのどれを呼ぶかもそこに書いてある。
    BY <ref id=e11772a/> (`result_prov` の呼び出しは abort せず `Provenance` を返す),
       前提 `result_prov` の本体の在りか,
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov (既定の本体は `Provenance::uniform` を
       `result_ty` に対して呼ぶ),
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody (`result_prov` は
       `Provenance::build_shape` を `result_ty` に対して呼ぶ),
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMUndefinedInternalBody (`result_prov` は
       `Provenance::uniform_bottom` を `result_ty` に対して呼ぶ),
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMArrayPunchBody (`result_prov` は
       `Provenance::fresh_under` を `result_ty` に対して呼ぶ),
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMStructSetBody (`result_prov` は
       `replaced_field_prov` に `result_ty` を渡す),
       CODE src/fixstd/builtin.rs: replaced_field_prov (`result_ty` が boxed なら
       `Provenance::uniform`、そうでなければ `Provenance::build_shape`)
  <2>3. <2>2 の 5 つはいずれも `LeafMap::build_shape(result_ty, ..)` を通り、鍵の集合を変えない。
    BY CODE src/rc_ir/provenance.rs: Provenance::build_shape, uniform, uniform_bottom, fresh_under,
       set_leaves_under -- `Provenance::build_shape` は `LeafMap::build_shape` をそのまま呼び、
       `uniform` は `LeafMap::uniform` を、`uniform_bottom` は `Provenance::build_shape` を呼ぶ。
       `fresh_under` は `Provenance::uniform` の結果に `set_leaves_under` を掛けたものであり、
       `replaced_field_prov` は `Provenance::uniform` か `Provenance::build_shape` を返す,
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, uniform, map_leaves_under --
       `LeafMap::uniform` は `LeafMap::build_shape` を呼ぶ。`set_leaves_under` が通る
       `map_leaves_under` は、各 `(leaf_path, fact)` の `fact` だけを写し、鍵をそのまま運ぶ
  <2>4. QED
    BY <2>1, <2>2, <2>3, CODE src/rc_ir/ownership.rs: collect_bindings (`Binding::Llvm` の第 3 成分は
       束縛される変数の型 `x.ty` である), CODE src/rc_ir/ownership.rs: origin_inner (`decl` はその型を
       `result_ty` として `result_prov` を呼んだ値である),
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at (`LeafMap::get` をそのまま呼ぶ),
       CODE src/rc_ir/leaf_map.rs: LeafMap::get (鍵に無い path には `None`)

<1>2. `decl.leaf_origins_at(π)` の値は L8 (b) の 5 つで尽きている。
  <2>1. `LeafOrigins` は `Set<LeafOrigin>` であり、`LeafOrigin` の構成子は `Fresh`、`Unknown`、`Arg` の
        3 つである。
    BY CODE src/rc_ir/provenance.rs: LeafOrigin, LeafOrigins
  <2>2. QED
    場合分けは、`None` と `Some` を分け、`Some` の中身の集合を要素数 0、1、2 以上で分け、
    要素数 1 を構成子で分けたものである。
    BY <1>1, <2>1

<1>3. `as_arg_projection(sources)` が `Some` を返すのは L8 (b) の第 3 の場合だけである。
  BY CODE src/rc_ir/ownership.rs: as_arg_projection -- `sources.len() != 1` で `None`、要素が `Fresh` か
     `Unknown` でも `None`。

<1>4. 第 3 の場合、鍵 `(x, π)` の答えは鍵 `(args[j], σ)` の答えである (辺 E3)。これは D9 の移動の表の
      `Llvm` の行と A3 の「単一の `Arg(j, σ)`」の行に一致する。
  BY <1>0 (`x` も `args[j]` も `B` に現れる `RcVar` の名前なので、<ref id=3c6aa4c/> の前提を満たす),
     <1>3, <ref id=3c6aa4c/> (鍵の答え),
     CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `Some((j, σ))` の枝
     -- この枝は `origin(vars, type_env, &args[j].name, &p)` の返り値をそのまま返すので、
     `origin_inner(x, π)` が答える値は鍵 `(args[j], σ)` の答えである,
     <ref id=9d74736/> の移動の表, <ref id=e11772a/>

<1>5. 残る 4 つの場合は `origin_from_leaves_under(vars, type_env, &decl, args, π, &(x, π))` に入り、
      それが `None` を返すとき、鍵 `(x, π)` の答えは `Exactly((x, π))` である。
  BY <1>0 (`x` は `B` に現れる `RcVar` の名前なので、<ref id=3c6aa4c/> の前提を満たす),
     <1>3, <ref id=3c6aa4c/> (鍵の答え),
     CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `None =>` の枝
     -- `here_identity` は `(var.clone(), path.to_vec())` であり、`unwrap_or_else(here)` の `here` は
     `Origin::Exactly((var.clone(), path.to_vec()))` を返す閉包である

<1>6. (d1)、(d2)、(d3) が成り立つ。
  <2>1. `leaf_origins_under(π)` は `π` で始まる各 leaf の宣言を返す。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>2. (d1)。`for src in sources` のループは `Fresh` と `Unknown` について `produced_here` を立て、
        ループの後に `if produced_here { reached.push(Origin::Exactly(here.clone())) }` が走る。
    BY <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>3. (d3)。ループは `Arg(j, leaf)` について
        `operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)))` を行い、`reached` は
        `operand_units` の各要素 `(j, unit)` について鍵 `(args[j], unit)` の答えを並べた列で始まる。
        **`produced_here` が真のときはその列の後ろに `Exactly(here)` が 1 つ積まれる** (`<2>2`) ので、
        この記述が `reached` の全体を述べるのは `produced_here` が偽のときだけである。(d3) が言うのは
        `origin(args[j], u_j)` が `reached` の要素であることなので、どちらの場合でも成り立つ。
    BY <1>0 (`args[j]` は `B` に現れる `RcVar` の名前なので、<ref id=3c6aa4c/> の前提を満たす),
       <2>1, <2>2, <ref id=3c6aa4c/> (鍵の答え),
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. (d2)。宣言がすべて空集合ならループは 1 度も回らず、`operand_units` は空、`produced_here` は
        偽であり、`reached` は空である。`reached.first()?` が `None` を返すので、<1>5 より答えは
        `Exactly((x, π))` である。
    BY <2>1, <1>5, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>5. QED
    BY <2>2, <2>3, <2>4

<1>7. QED
  BY <1>1, <1>2, <1>4, <1>5, <1>6

A3 の 5 行との突き合わせは次のとおりである。空集合と宣言された leaf は inhabited にならないので、L8 (d2)
が答える `Exactly((x, π))` が名付ける参照は存在しない。単一の `Fresh` と単一の `Unknown` はどちらも
新しい参照であり、L8 (d1) が `Exactly((x, π))` を `reached` に積むことは D10 の生成の `Llvm` の行に一致
する。要素数 2 以上の宣言は A3 よりこのプログラムに無い。

## 3. L9 -- D10 の「生成」と `here()` の腕 <!--#0212823-->

`here()` は `Origin::Exactly((var, path))` を返す閉包である
(`CODE src/rc_ir/ownership.rs: origin_inner` の先頭)。

**L9 (`here()` の答えに着く道)**: **`var` を `B` に現れる `RcVar` の名前とする。** <!--#64230aa-->
`origin_inner(vars, type_env, var, path)` が
`Origin::Exactly((var, path))` に着く道は次の表の H1 から H7 である。H1 から H5 では `match` の腕が
`here()` をそのまま返し、この 5 つは `here()` をそのまま返す枝の全体である。H6 と H7 では `Llvm` の腕が
`origin_from_leaves_under` を通ってその値に着く。

**前提を置くのは、この証明が L15 と L16 を読むからである。** その 2 つはどちらも、主語の名前が
`B` に現れる `RcVar` の名前であることを前提に置く命題であり、L9 の前提と `<1>0` がそれを満たす。
**この文書が L9 を引くのは、どれも L17 の ASSUME を満たす変数についてである** -- DEF-0 が
「値を持つ」を定めるのは本体に現れる `RcVar` についてなので、その変数はこの前提を満たす。

| 道 | 着き方 | D10 での位置 |
|---|---|---|
| H1 `None` (表に無い名前) | 直接 | A8 (グローバルは線形規律の外) |
| H2 `Param` | 直接 | D10 の初期値 |
| H3 `Producer` | 直接 | 生成の表の `App` の行と `Closure` の行 |
| H4 `Field(c, i)` かつ `c` が boxed | 直接 | 生成の表の boxed 容器の `Destructure` の行 |
| H5 `Payload(s, Some(t))` かつ `s` が boxed | 直接 | 生成の表の boxed union の変位アームの行 |
| H6 `Llvm` かつ `π` の下のある leaf の宣言が `Fresh` か `Unknown` を含み、`reached` の全要素が等しい | `origin_from_leaves_under` が `Exactly(here)` を `reached` に積み、それが答えになる | 生成の表の `Llvm` の行 |
| H7 `Llvm` かつ `π` の下の leaf の宣言がすべて空集合 | `origin_from_leaves_under` が `None` を返し `unwrap_or_else(here)` | 生成の表の `Llvm` の行が覆う。ただし A3 と D16 よりその leaf は inhabited にならないので参照は生じない |

<1>0. 第 1 成分が `B` に現れる `RcVar` の名前である鍵から再帰の辺を 0 回以上辿って着く鍵は、その
      第 1 成分も `B` に現れる `RcVar` の名前である。とくに `(var, path)` から着く鍵がそうである
      (L9 の前提)。
  1 本の辺については、`origin_inner` が再帰呼び出しに
  渡す名前が `vars.bindings` の `Binding` が持つ `RcVar` -- `Move(y)` の `y`、`Payload(s, ・)` の
  `s`、`Field(c, i)` の `c`、`Llvm(・, args, ・)` の `args[j]`、`Join(rs)` の各要素 -- の `name` で
  あり、`collect_bindings` はその `RcVar` を `B` の節点の欄から入れるからである。辿った辺の本数に
  ついての帰納で、0 回以上の場合に届く。
  BY <ref id=0212823/> の前提, <ref id=9357e31/> (再帰の辺 E1 から E7 の行き先の `RcVar` は、`B` の節点の欄から来る),
     CODE src/rc_ir/ownership.rs: collect_bindings (`Binding::Move`・`Binding::Payload`・
     `Binding::Field`・`Binding::Llvm`・`Binding::Join` に入る `RcVar` は、その `Binding` を作る
     節点の欄である),
     CODE src/rc_ir/ownership.rs: origin_inner (各腕が再帰呼び出しに渡すのは、その腕が受け取った
     `Binding` の `RcVar` の `name` である),
     EXT 整礎性 ((a) 自然数の狭義減少する無限列は無く、(b) その上の帰納が使える -- 辿った辺の
     本数は自然数である)

<1>1. `origin_inner` の `match` の腕のうち `here()` をそのまま返す枝は、H1 から H5 の 5 つで尽きている。
  BY <ref id=9357e31/> (再帰の辺は E1 から E7 で尽きている), CODE src/rc_ir/ownership.rs: origin_inner -- 6 本の腕の
     うち `here()` をそのまま返すのは、
     `None | Some(Binding::Param) | Some(Binding::Producer)` の腕 (H1、H2、H3)、
     `Some(Binding::Field(container, idx))` の `container.ty.is_box(type_env)` の枝 (H4)、
     `Some(Binding::Payload(..))` の `Some(_)` の枝 (H5)。ほかの枝は `origin` を再帰呼び出しするか
     `origin_from_leaves_under` に入る。

<1>2. `here()` をそのまま返さない道 -- `Llvm` の腕と、`Move` の腕・`Field` の `else` の枝・
      `Payload` の 2 つの枝・`Join` の腕 -- のうち `Exactly((var, path))` に着くのは、`Llvm` の腕の
      H6 と H7 の 2 つだけである。
  <2>1. `Llvm` の腕が答えを作る道は 4 つある -- `as_arg_projection` が `Some((j, p))` を返す枝の
        `origin(args[j], p)`、`origin_from_leaves_under` の `reached` の全要素が等しい枝が返す
        `first.clone()`、その `else` に当たる末尾の `of_candidates(candidates, here)`、そして
        `origin_from_leaves_under` が `None` を返したときの `unwrap_or_else(here)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>2. 第 4 の道が `Exactly((var, path))` を返すのが H7 である。
    BY <2>1, <ref id=9a6b1cd/> (d2), CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- `reached` が空のとき
       `reached.first()?` が `None` を返し、`unwrap_or_else(here)` が `Exactly((var, path))` を答える。
       `reached` が空であることと `π` の下の leaf の宣言がすべて空集合であることは同値である
       (<ref id=9a6b1cd/> (d1) と <ref id=9a6b1cd/> (d3) がその 2 つの向きを与える)。
  <2>3. 末尾の `of_candidates(candidates, here)` は `Exactly` を返さない。
    <3>1. この道に入るのは `reached` の全要素が等しくないときであり、そのとき `reached` は相異なる
          2 つの `Origin` `o_1 ≠ o_2` を含む。
      BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`if reached.iter().all(..)` が偽である
         枝)
    <3>2. `candidates ⊇ act(o_1) ∪ act(o_2)` である。
      BY <3>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`flat_map(|reached_origin|
         reached_origin.acted_on())`)
    <3>3. `|candidates| ≥ 2` である。
      BY <3>1, <3>2, <ref id=0212823/> の前提 (`var` は `B` に現れる `RcVar` の名前である), <ref id=e05fb56/> (b), <ref id=e05fb56/> (c), <ref id=0376e8d/>,
         前提 `origin_inner` を呼ぶ式の在りか (走らせる式は `origin` の中の 1 つである),
         CODE src/rc_ir/ownership.rs: origin (`grow_stack(|| origin_inner(..))` がその式で
         ある) -- `o_1` と `o_2` は
         `origin_from_leaves_under` が `reached` に積んだ値であり、この `origin_inner` の実行の中で
         作られたものである。その実行を走らせるのは `origin(var, path)` の呼び出しであり、
         <ref id=0212823/> の前提が <ref id=0376e8d/> の前提を満たすので、<ref id=0376e8d/> より その呼び出しは panic せずに
         返る。よって <ref id=e05fb56/> (c) の前提が満たされる。`|candidates| = 1` とすると、`act(o_1)` と `act(o_2)` は
         どちらもその 1 元集合である。<ref id=e05fb56/> (c) より `Join` の `act` は 2 元以上なので `o_1` と `o_2` は
         どちらも `Exactly` であり、<ref id=e05fb56/> (b) よりその `act` は自分の `VarPath` の 1 元集合なので
         `o_1 = o_2` となって <3>1 に反する。
    <3>4. QED
      BY <3>3, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- 要素数が 1 でない集合には
         `Join` を返す。
  <2>3a. **`v` を `B` に現れる `RcVar` の名前とする。** `origin(v, q)` が呼ばれるとき、それが返す
         `Origin` に現れる `VarPath` は、鍵 `(v, q)` から
         **再帰の辺** (`DEF 再帰の辺`) を 0 回以上辿って着く鍵である。**再帰の辺は `DEF 再帰の辺` が
         定めるものであり、L6 よりそれは E1 から E7 で尽きる。** 鍵についての言明にできるのは、
         `origin` の答えが鍵ごとに 1 つに決まり、それが
         その鍵について `origin_inner` が答えた値だからである (L16)。
         **前提を 2 つ置くのは L14 (a) と L16 のためである** -- L14 (a) の整礎性は、`origin` が
         呼ばれる鍵から到達する鍵の上でしか言えず、L16 は鍵の第 1 成分が `B` に現れる `RcVar` の
         名前であることを前提に置く。`<1>0` より、その鍵から到達する鍵の第 1 成分も `B` に現れる
         `RcVar` の名前である。
    <3>1. `#[cfg(test)]` のモジュールを除くと、`Origin` の値を作る式は 3 つである --
          `origin_inner` の `here()`、`origin_from_leaves_under` の `Origin::Exactly(here.clone())`、
          そして `Origin::of_candidates` である。**除くのは、そのモジュールの項目が製品のコードの
          実行路に無いからである** -- 第 1 節が `VarTable::empty` の呼び出し元について同じ形で
          除いている。
      BY 前提 `Origin::Exactly` を作る式の在りか (走査は `#[cfg(test)]` の下の項目を除く。
         挙がった項目のうち `Origin::identity` と `Origin::candidates` はパターンであって
         構成ではない),
         <ref id=d6c2508/> (`Origin::Join { .. }` を作るのは `Origin::of_candidates` だけであり、どの `Origin` の
         値も `Exactly` か `of_candidates` が作った `Join` かその複製である),
         CODE src/rc_ir/ownership.rs: origin_inner (`here` は
         `Origin::Exactly((var.clone(), path.to_vec()))` を返す閉包である),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
         (`reached.push(Origin::Exactly(here.clone()))`),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates (`1 =>` の腕が `Origin::Exactly` を、
         `_ =>` の腕が `Origin::Join` を作る),
         CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates (`Origin::Exactly` を
         `match` のパターンとして持つ)
    <3>1a. 前の 2 つが作る `Exactly` の `VarPath` は、その呼び出し自身の `(var, path)` である。
      BY CODE src/rc_ir/ownership.rs: origin_inner (`here` の本体、および
         `origin_from_leaves_under` に渡す `here_identity` が `(var.clone(), path.to_vec())` であること),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`here` は呼び出し元が渡す引数である)
    <3>1b. `of_candidates(C, h)` が作る `Origin` に現れる `VarPath` は、`C` の元か `h` である。
           **`1 =>` の腕は `C` のただ 1 つの元を `Exactly` に運ぶので、その `VarPath` は `h` とは限ら
           ない。**
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates -- `1 =>` の腕は `candidates` の唯一の元を
         `Origin::Exactly` に置き、`_ =>` の腕は `identity` に `h` を、`candidates` に `C` を置く,
         CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates
    <3>1c. `of_candidates` を呼ぶのは `origin_inner` の `Some(Binding::Join(..))` の腕と
           `origin_from_leaves_under` の末尾であり、どちらでも `h` はその
           呼び出し自身の `(var, path)`
           であり、`C` の各元は、その呼び出しが畳み込む `Origin` のいずれかに現れる `VarPath` である。
      BY <ref id=e05fb56/> (a) (`acted_on()` の元は `identity` か `candidates` の元である), <ref id=d2c1f1f/>,
         前提 `of_candidates` を呼ぶ式の在りか,
         EXT 可視性 (`pub` の付かない項目を名指せるのは、それを定義するモジュールとその子孫だけで
         ある),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates (`fn` に `pub` が付かないので、この関数を
         呼べるのは `ownership.rs` の中だけである),
         CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕
         (`Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` であり、`candidates` は
         各アーム結果の `origin(..).acted_on()` の元を集めたものである),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under (末尾の
         `Origin::of_candidates(candidates, here)` であり、`candidates` は `reached` の各元の
         `acted_on()` を集めたものである), CODE src/rc_ir/ownership.rs: Origin::acted_on
    <3>2. 1 つの鍵の答え -- その鍵について `origin_inner` が返した値 (L16) -- は、次の 4 つのいずれかで
          ある。(r1) `here()` -- <1>1 の 5 つの枝と、
          `origin_from_leaves_under` が `None` を返したときの `unwrap_or_else(here)`。(r2) 子の呼び出し
          `origin(..)` が返した値そのもの -- `Move` の腕、`Field` の `else` の枝、`Payload` の 2 つの枝、
          `Llvm` の腕の `as_arg_projection` が `Some` を返す枝。(r3) `origin_from_leaves_under` が
          `reached` の全要素が等しいときに返す `first.clone()`。`reached` の各元は、子の呼び出しが
          返した値か `Origin::Exactly(here.clone())` である。(r4) `of_candidates(C, h)` の値 --
          `Binding::Join` の腕と `origin_from_leaves_under` の末尾。
      BY <1>1, <2>1, <1>0 (この帰納が扱う鍵の第 1 成分は `B` に現れる `RcVar` の名前であり、<ref id=3c6aa4c/> の
         前提を満たす), <ref id=9357e31/>, <ref id=3c6aa4c/> (鍵の答え),
         CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
    <3>3. QED
      BY <3>1, <3>1a, <3>1b, <3>1c, <3>2, <1>0, <ref id=3c6aa4c/>, <ref id=9357e31/> (再帰の辺は E1 から E7 で尽きる),
         <ref id=391d1ce/> (a), <ref id=391d1ce/> (b) (`origin(K)` が呼ばれ再帰の辺 `K -> K'` が在れば `origin(K')` も呼ばれる
         ので、帰納法の仮定の前提が各段で満たされる),
         EXT 整礎性 ((b) 整礎な関係の上では整礎帰納が
         使える), <ref id=b1f6e13/> -- 鍵の再帰の辺の関係は、`origin(v, q)` が呼ばれる鍵から到達する鍵の上で
         整礎である (<ref id=391d1ce/> (a))。その関係の
         上の整礎帰納で示す。子の呼び出しが返した値はその子の鍵の答えである (<ref id=3c6aa4c/>)。
         (r1) が返す値の `VarPath` はその呼び出しの鍵そのもの (<3>1a)。
         (r2) が返す値には帰納法の仮定が当たり、その子の鍵はこの鍵から辺 1 本で着く (<ref id=9357e31/>)。(r3) が
         返すのは、子の呼び出しが返した値の複製か `Origin::Exactly(here.clone())` の複製であり、前者には
         帰納法の仮定が当たり、後者の `VarPath` はこの呼び出しの鍵である (<3>1a)。複製は `VarPath` を
         変えない (<ref id=d6c2508/>)。(r4) が返す値の `VarPath` は `h` か `C` の元であり (<3>1b)、`h` はこの呼び出しの
         鍵、`C` の元は畳み込む `Origin` -- すなわち子の呼び出しが返した値か
         `Origin::Exactly(here.clone())` -- に現れる `VarPath` である (<3>1c)。
  <2>4. `Llvm` の腕がオペランドについて行う再帰呼び出し -- `as_arg_projection` が `Some((j, p))` を
        返す枝の `origin(args[j], p)` と、`origin_from_leaves_under` が `reached` を作るときの
        `origin(args[j], unit)` -- は、どれも `Exactly((var, path))` を返さない。
    <3>1. そのような呼び出しが `Exactly((var, path))` を返すならば、その呼び出しの鍵から再帰の辺を
          0 回以上辿って鍵 `(var, path)` に着く。
      BY <2>3a, <1>0 -- その呼び出しは `origin_inner(var, path)` の実行が行う `origin` の呼び出しな
         ので `<2>3a` の第 2 の前提 (その鍵で `origin` が呼ばれる) を満たし、その鍵は `(var, path)`
         から再帰の辺 1 本で着くので `<1>0` が第 1 の前提 (第 1 成分が `B` に現れる `RcVar` の名前で
         ある) を与える,
         CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>2. 鍵 `(var, path)` からその呼び出しの鍵へは、再帰の辺が 1 本ある。
      BY <ref id=9357e31/> (E3 と E4 は `Binding::Llvm` の腕の再帰呼び出しである),
         CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. QED
      `<3>1` と `<3>2` を繋ぐと、鍵 `(var, path)` から再帰の辺を 1 回以上辿って `(var, path)` 自身に
      着く。すなわち鍵の辺の上に閉路がある。その閉路を繰り返せば無限に降りる鍵の列ができるので、
      L14 (a) に反する。
      BY <3>1, <3>2, <ref id=391d1ce/> (a)
  <2>4a. 第 2 の道が `Exactly((var, path))` を返すのが H6 である。
    BY <2>1, <2>4, <ref id=9a6b1cd/> (d1), CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- この道が返すのは
       `reached` の全要素が等しいときの `first.clone()` であり、`reached` の元は
       `origin(args[j], unit)` の値と、`produced_here` が真のときに積まれる `Exactly(here)` である。
       <2>4 より前者は `Exactly((var, path))` ではないので、`first` が `Exactly((var, path))` である
       ためには `produced_here` が真、すなわち `π` の下のある leaf の宣言が `Fresh` か `Unknown` を
       含むことが要る。逆にそのとき <ref id=9a6b1cd/> (d1) より `Exactly((var, path))` は `reached` の元であり、
       全要素が等しければそれが答えである。
  <2>4b. `here()` をそのまま返さない残る 5 つの道 -- `Move(y)` の腕、`Field(c, i)` の `c` が unbox の
        枝、`Payload(s, v)` の `None` の枝、`Payload(s, Some(t))` かつ `s` が unbox の枝、`Join(rs)` の
        腕 -- は、どれも `Exactly((var, path))` を返さない。
    <3>1. 前の 4 つが返すのは、子の呼び出し `origin(..)` が返した値そのものである。`Join` の腕が
          返すのは `of_candidates(C, (var, path))` であり、`C` は各子の呼び出しが返した値の
          `acted_on()` の和である。
      BY CODE src/rc_ir/ownership.rs: origin_inner, Origin::of_candidates, Origin::acted_on
    <3>2. どの子の鍵へも、鍵 `(var, path)` から再帰の辺が 1 本ある。
      BY <ref id=9357e31/> (E1 が `Move`、E5 が `Field` の unbox の枝、E6 と E7 が `Payload` の 2 つの枝、
         E2 が `Join` の腕の再帰呼び出しである),
         CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. どの子の呼び出しも `Exactly((var, path))` を返さない。
      返すとすると、`<2>3a` よりその子の鍵から再帰の辺を 0 回以上辿って `(var, path)` に着く -- 子の
      呼び出しは `origin_inner(var, path)` の実行が行う `origin` の呼び出しなので `<2>3a` の第 2 の
      前提を満たし、その鍵は `(var, path)` から辺 1 本で着くので (`<3>2`) `<1>0` が第 1 の前提を
      与える。`<3>2` と
      繋ぐと、鍵 `(var, path)` から再帰の辺を 1 回以上辿って `(var, path)` 自身に着く。その閉路を
      繰り返せば無限に降りる鍵の列ができるので、L14 (a) に反する。
      BY <2>3a, <1>0, <3>2, <ref id=391d1ce/> (a), CODE src/rc_ir/ownership.rs: origin_inner
    <3>4. QED
      前の 4 つの道については `<3>1` と `<3>3` から出る。`Join` の腕については、`of_candidates(C, h)` が
      `Exactly` を返すのは `|C| = 1` のときであり、そのとき返るのは `C` の唯一の元を持つ
      `Exactly` である。その元は子の呼び出しが返した値に現れる `VarPath` なので (`<3>1`、L2 (a) と
      L2 (b))、`<2>3a` よりその子の鍵から再帰の辺を 0 回以上辿って着く鍵である -- その子の鍵は
      `(var, path)` から辺 1 本で着くので (`<3>2`)、`<1>0` が `<2>3a` の第 1 の前提を与える。それが
      `(var, path)` であれば `<3>3` と同じ閉路ができ、L14 (a) に反する。
      BY <2>3a, <1>0, <3>1, <3>2, <3>3, <ref id=e05fb56/>, <ref id=391d1ce/> (a), CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>4, <2>4a, <2>4b

<1>3. D10 の生成の 5 行はすべて `here()` の道を持つ -- H3 が `App` の行と `Closure` の行、H4 が boxed
      容器の `Destructure` の行、H5 が boxed union の変位アームの行、H6 が `Llvm` の行である。
  BY <ref id=f06144e/> の生成の表, <1>1, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Binding::Producer` を作るのは `RcRhs::App` と
     `RcRhs::Closure` の 2 つだけであり、`Binding::Field` は `RcExpr::Destructure` の名前付き
     フィールド、`Binding::Payload(scrut, Some(t))` は変位アームの payload である。boxed か unbox かは
     `is_box` の枝が分ける。

<1>4. H1、H2、H7 のどれも新しい参照を作らない。
  <2>1. H1 と H2 は D10 の生成の表に行を持たない。
    BY <ref id=f06144e/> の生成の表 (5 行はいずれも `Llvm`、`App`、`Closure`、boxed 容器の `Destructure`、boxed union の
       変位アームであり、束縛を持たない名前についての行もパラメータ・capture についての行も無い),
       <ref id=b6673ca/> (H1: グローバル値が到達するオブジェクトは線形規律の外にある),
       <ref id=f06144e/> の初期値 (H2: パラメータと capture の参照はそこに置かれるのであって、生成されるのではない)
  <2>2. H7 の leaf は D10 の生成の `Llvm` の行が覆うが、その leaf は inhabited にならないので参照は
        生じない。
    BY <ref id=f06144e/> の生成の表の `Llvm` の行 (「宣言が空集合 (bottom) のとき、`Fresh` や `Unknown` を含むとき、
       複数の元を持つときのすべてを含む。空集合と宣言された leaf は inhabited にならないので、参照は
       生じない (A3)」), <ref id=e11772a/> (空集合の行), <ref id=66c9670/>
  <2>3. QED
    BY <2>1, <2>2

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

## 4. 命題

以下の命題は、この文書のすべての証明が使う。よって第 2 節と第 3 節の証明もこれらを引く。

**この節の命題のうち第 2 節・第 3 節のものへ届くのは L14 だけであり、その `BY` が引くのは第 2 節の
`DEF 再帰の辺` である。** L14 の脇は L6 の表も読む -- E5 と E7 の辺が path の先頭に添字を足すことが
そこから来る。L14 はこの節の L10 (a') も引く。L15 の脇も L6 を読む。
残る L1 から L5、L10 から L13、L15、L16 が引くのは、README の定義・仮定・命題 (P2、P2a)、コード、
`EXT` の外部の結果、およびこの節の先行する命題である -- L2 は L1 を、L3 は L2 を、L4 は L2 と L3 を、
L11 は L10 を、L13 は L10 と L12 を、L15 は L10 を、L16 は L15 を引く。**循環は生じない** -- L6 の
証明は第 4 節のどの命題も引かず、L8 の証明が引くのは L16 と L10 だけなので、順序は
「L1 から L5、L10 から L13 → L6 → L15 → L16 → L14 → L8 → L9」に並ぶ。

**L1 (`Origin::Join` は `of_candidates` だけが作る)**: `Origin::Join { .. }` を値として作るのは <!--#d6c2508-->
`Origin::of_candidates` だけである。よって、どの `Origin` の値も、`Exactly` であるか、
`of_candidates` が作った `Join` (あるいはその複製) である。

<1>1. `Origin` の `Join` 変位を値として作る式は、その字面に変位の名前 `Join` を含む。よってその式が
      在りうるのは、第 1 節の前提が挙げる項目の中だけである。**在りかを与えるのは走査である** --
      挙がった各項目が何であるかは、その前提の `--` の後に書いてある。
  BY 前提 `Origin` の `Join` 変位を作る式の在りか,
     CODE src/rc_ir/ownership.rs: Origin (`Join` は `Origin` の変位の名前である)
<1>2. 挙がった項目のうち `src/rc_ir/ownership.rs` の外にあるものは、どれも `Origin` の `Join` 変位を
      作らない -- 前提の `--` が述べるとおり、英語の doc かコメント、または
      `std::thread::JoinHandle` の綴りだからである。
  BY <1>1, 前提 `Origin` の `Join` 変位を作る式の在りか
<1>3. `src/rc_ir/ownership.rs` の中で挙がった項目のうち、`Origin::Join { .. }` を値として作るのは
      `Origin::of_candidates` だけである。`Origin::identity` と `Origin::candidates` の
      `Origin::Join` は `match` のパターン、`Origin` に挙がったのは変位の宣言であり、
      `Binding`・`collect_bindings` と `origin_inner` の `Binding::Join` は別の型 `Binding` の
      変位である。`Origin::acted_on` と `origin_inner` に挙がった残りは英語の doc である。
  BY <1>1, <1>2, CODE src/rc_ir/ownership.rs: Origin, Origin::identity, Origin::candidates,
     Origin::of_candidates, Origin::acted_on, Binding, collect_bindings, origin_inner
<1>4. `Origin` は `Clone` を導出するので、`Join` の値は複製によっても現れる。複製は `identity` と
      `candidates` をそのまま運ぶ。
  BY EXT 導出した Clone (`#[derive(Clone)]` の `clone` は同じ構成子の値を返し、各欄にその型の
     `clone` が返す複製を置く),
     CODE src/rc_ir/ownership.rs: Origin (`#[derive(Clone, Debug, PartialEq, Eq)]`)
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

**L2 (`identity`、`candidates`、`acted_on` の関係)**: 任意の `Origin` の値 `o` について次が成り立つ。 <!--#e05fb56-->

- **(a)** `act(o) = {id(o)} ∪ cand(o)`。とくに `act(o) ⊇ cand(o)` であり `id(o) ∈ act(o)` である。
- **(b)** `o = Exactly(p)` ならば `id(o) = p` かつ `cand(o) = act(o) = {p}` である。
- **(c)** `o` が `Join` であり、かつ `o` が、panic せずに返る `origin` の呼び出しの中で作られた値で
  あるならば、`|cand(o)| ≥ 2` であり、よって `|act(o)| ≥ 2` である。この前提を、L15 が
  P2 から、この文書のすべての `Origin` について与える。**支えるのは表明ではなく P2 である** -- `len == 0`
  で `Join` を作る枝はコードの上に在り、それを塞ぐ表明が発火しないことの根拠が P2 である。

<1>1. (a)。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on -- `identity` を先頭に置き、`candidates` から
     `identity` に等しいものを除いたものを続ける。

<1>2. (b)。
  BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates, Origin::acted_on

<1>3. (c)。
  <2>1. `Join` の値を作る式は `Origin::of_candidates` の中の 1 か所だけであり、その枝は
        `candidates.len()` が 1 でないときに走る。複製は `candidates` をそのまま運ぶ。
    BY <ref id=d6c2508/>, EXT 導出した Clone, CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>2. `of_candidates` はその枝へ進む前に `assert!(!candidates.is_empty(), ..)` を評価する。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>3. その `assert!` は発火せず、`candidates` は空でない。表明が発火すればその呼び出しは panic して
        返らないので、`o` が値として在ることと相容れない。
    BY <2>2, 言明の (c) の前提 (`o` は panic せずに返る `origin` の呼び出しの中で作られた値である)
  <2>4. QED
    BY <2>1, <2>3, <1>1 -- `|cand(o)| ≠ 1` かつ `cand(o)` が空でないので `|cand(o)| ≥ 2` であり、
       <1>1 の `act(o) ⊇ cand(o)` より `|act(o)| ≥ 2` である。

<1>4. QED
  BY <1>1, <1>2, <1>3

**L3 (`of_candidates` の `acted_on` は与えた集合を含む)**: 空でない集合 `C` と `h` について <!--#3de9373-->
`act(of_candidates(C, h)) ⊇ C`。

<1>1. `|C| = 1` のとき `of_candidates(C, h) = Exactly(c)` (`C = {c}`) であり、`act = {c} = C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, <ref id=e05fb56/> (b)
<1>2. `|C| ≥ 2` のとき `of_candidates(C, h) = Join { identity: h, candidates: C }` であり、
      `act = {h} ∪ C ⊇ C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, <ref id=e05fb56/> (a)
<1>3. QED
  BY <1>1, <1>2

**L4 (畳み込みの答えの形)**: `origin_inner` の `Binding::Join` の腕と `origin_from_leaves_under` が <!--#d2c1f1f-->
畳み込む先の `Origin` を `o_1, ..., o_k` とし、答えを `o` とする。**`k ≥ 1` である** -- `Binding::Join`
の腕では `k` はアームの個数であって A9 がそれを 1 以上にし、`origin_from_leaves_under` では答えを作る
道で `reached` が空でない。

- **(a)** `Binding::Join` の腕では `o = of_candidates(∪_i act(o_i), (var, path))` である。
- **(b)** `origin_from_leaves_under` では、`reached` の全要素が等しいときは `o` はその要素そのもので
  あり、そうでないときは `o = of_candidates(∪_i act(o_i), here)` である。
- **(c)** どちらの場合も `act(o) ⊇ act(o_1) ∪ ... ∪ act(o_k)` である。

<1>0. `k ≥ 1` である。
  BY <ref id=1172c08/> (`Match` は 1 つ以上のアームを持つ),
     CODE src/rc_ir/ownership.rs: origin_inner (`Some(Binding::Join(arm_results))` の腕が畳み込む先は
     `arm_results` の各要素である),
     CODE src/rc_ir/ownership.rs: collect_bindings (`RcRhs::Match` の腕は各アームの `returned_var` を
     `arm_results` に 1 つずつ入れるので、`arm_results` の長さはアームの個数である),
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`reached.first()?` は `reached` が空の
     とき `None` を返してそこで抜けるので、畳み込みの答えを作る道では `reached` は空でない)
<1>1. (a)。
  BY <1>0, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕 -- ループは
     `origin(..).acted_on()` の各要素を `candidates` に入れる。
<1>2. (b)。
  BY <1>0, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `if reached.iter().all(..)` の枝と
     その後の `flat_map(|reached_origin| reached_origin.acted_on())`
<1>3. (a) と (b) の後者の場合、`act(o) ⊇ ∪_i act(o_i)`。
  BY <1>0, <1>1, <1>2, <ref id=e05fb56/> (a) (`act(o_i)` は `id(o_i)` を含むので空でない -- よって `k ≥ 1` と
     合わせて `∪_i act(o_i)` は空でなく、<ref id=3de9373/> の前提が満たされる), <ref id=3de9373/>
<1>4. (b) の前者の場合、`o = o_1 = ... = o_k` なので `act(o) = ∪_i act(o_i)`。
  BY <1>0, <1>2
<1>5. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>4 -- (c) はどちらの場合も成り立つ。

**L5 (leaf は互いに比較不能である)**: 型 `τ` の相異なる 2 つの boxed leaf の一方が他方の接頭辞になることは <!--#c2174d1-->
無い。

<1>1. `boxed_leaf_paths` の走査は、leaf を積んだ位置の下へ降りない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- 走査 `go` が `out.push` を行う 3 つの枝
     (`is_closure`、`is_box`、`is_array`) は、いずれも `unpunched_field_types` のループへ進まずに
     `return` する。
<1>2. QED
  BY <1>1 (leaf が積まれる位置の下は走査されないので、leaf の真の延長が leaf になることは無い)

**L10 (変数に値を与える構文と、値が束縛の後 変わらないこと)** <!--#49da857-->

- **(a)** 本体の節点のうち、変数に値を与えるのは 3 つの構文である -- `Let(x, rhs, k)` は `x` に、
  `Destructure(c, fs, s, k)` は各 `(i, x)` の `x` に、`Match` の各アームはその `payload` に値を与える。
  残る 4 種 (`Retain`、`Release`、`Eval`、`Ret`) はどの変数にも値を与えない。**パラメータと capture に
  値を与えるのは節点ではなく、活性化の入力の束縛 (D23) である。**
- **(a')** `vars.bindings` に入る名前は、`B` が関数 `f` の `body` であるときの `f` のパラメータと
  capture (いずれも `Binding::Param`)、および (a) の 3 構文が束縛する変数だけである。すなわち DEF-0 の
  (v-1) と (v-2) は `vars.bindings` が束縛を持つ名前、(v-3) はそれ以外の名前である。
- **(b)** 活性化 `α` とそれが辿る実行路 `ρ` について、`RcVar` `v` が `ρ` 上の位置 `N` で値を持つ
  (DEF-0) ならば、`ρ` 上の `N` 以後のすべての位置で `v` の値は同じである。
- **(c)** DEF-0 の (v-1) の授与位置について次の 3 つが成り立つ。
  - **(c1)** DEF-0 の (v-1) の表の 4 行は、(a) の 3 構文が作る束縛を尽くしている。
  - **(c2)** `ρ` がある束縛の授与位置を通るとき、その位置でその変数はその束縛が与える値を持つ。値が
    何であるかまで名指せるのは次の 3 つである。`Let(v, rhs, k)` (`rhs` が `Match` でない) と
    `Destructure(c, fs, s, k)` については D2 の節点の表が名指す。`Let(v, Match(s, arms), k)` については、
    `ρ` は授与位置へ進む前に `α` が選んだアーム本体の終端の `Ret` を通っており、`v` の値はその `Ret` が
    名指す変数の値である。`Match` のアーム `A` の `payload` については、`ρ` がその授与位置を通るのは
    `α` が `A` を選んだときに限る、というところまでを言う。
  - **(c3)** 節点 `M` が、ある束縛の D2 の意味のスコープに在り、`ρ` が `M` を通るならば、`ρ` はその束縛の
    授与位置を `M` までに通る。
- **(d)** DEF-0 の (v-3) の名前 -- `vars.bindings` に束縛を持たない名前 -- は、そのプログラムの最上位の
  記号の名前であり、`FullName::is_local` が偽である。その値は `declare_program_global` が用意する 2 つの
  うちの一方である -- 型が `is_funptr` なら `declare_lambda_function` が返す LLVM 関数の番地、そうで
  なければ `add_global_object` が登録するグローバルのアクセサが返す値である。
- **(e)** DEF-0 の (v-1) と (v-2) の `v` は、D6 の「その実行路の上でその時点までに値を得た変数」で
  あり、`ty(v)` の inhabited な boxed leaf との対は `P` のスロット (D6) である。(v-3) の `v` との対は
  D6 の記号の位置である。詳しくは次の 3 つである。
  - **(e1)** (v-2) が `P` に条件を課さないのは、D2 よりパラメータと capture のスコープが本体の全体だ
    からである。`B` がグローバル初期化子の `init` であるとき、この場合は空である。
  - **(e2)** `ρ` が `v` を名指す節点を `P` **より前に**通っているとき、`P` はその記号の (E5) の段より
    後にある。`P` がその節点自身の位置であるときは、その段が `v` を読む前に (E7) と (E5) を走らせるので、
    **その読みの時点**が (E5) の段より後にある。DEF-0 の (v-3) が「その記号の値」と言うのは、どちらでも
    その時点にその記号の記憶域が持つ値である。
  - **(e3)** DEF-0 の 3 つの場合は尽きており、互いに排他である。

<1>1. (a)。**「`Match` の各アームはその `payload` に値を与える」は、boxed union の変位アームについても
      言う。** D9 の値の水準の 6 行が持つアームの行は 2 つ -- unbox union の変位アームと catch-all --
      であり、boxed union の変位アームを覆うのは D2 の `MatchArm` の `payload` 欄と束縛の及ぶ範囲の
      段落である。
  BY <ref id=b3dfa37/> (節点の 6 種の表 -- `Let` は `rhs` の値を `x` に束縛し、`Destructure` は各 `(i, x)` の `x` に
     第 `i` フィールドを束縛し、`Retain` は参照を作り、`Release` は参照を処分し、`Eval` は評価して
     捨て、`Ret` はその式の値を述べる),
     <ref id=b3dfa37/> の `MatchArm` の `payload` 欄 (「`Match` の各アーム `MatchArm` は 4 個のフィールドを持つ」--
     その 1 つが「`payload` (payload 変数)」である) と束縛の及ぶ範囲の段落 (「`Match` のアームの
     `payload` のスコープはそのアームの `body` の部分木である」-- すなわち、変位が boxed か unbox かに
     依らず、どのアームもその `payload` を束縛する), <ref id=9d74736/> の移動の表の値の水準の 6 行 (「unbox union の変位アームの
     payload 束縛: payload 変数の値は scrutinee の値の活性変位の payload である」と「catch-all アームの
     payload 束縛: payload 変数の値は scrutinee の値そのものである」の 2 行が、その 2 つの場合について
     値が何であるかを名指す), <ref id=b3dfa37/> の束縛の及ぶ範囲の段落 (パラメータと capture のスコープは
     本体の全体である), <ref id=ff5985d/> (活性化の入力の束縛が各パラメータと capture に 1 つずつの値を与える)
<1>1a. (a')。
  BY CODE src/rc_ir/ownership.rs: VarTable::of (関数の本体について、`func.params` と `func.capture` の
     各 `p` に `Binding::Param` を入れ、続けて `collect_bindings` を呼ぶ),
     CODE src/rc_ir/ownership.rs: VarTable::body_only (グローバル初期化子の本体について、`collect_bindings` だけを
     呼ぶ。<ref id=a502f3e/> より `init` はパラメータも capture も持たない),
     CODE src/rc_ir/ownership.rs: collect_bindings -- 変数に `Binding` と型を入れるのは `RcExpr::Let` の
     腕、`RcExpr::Destructure` の腕、`RcRhs::Match` の腕の `arm.payload` の 3 か所だけであり、
     `RcExpr::Retain`、`RcExpr::Release`、`RcExpr::Eval`、`RcExpr::Ret` の腕はどの変数も入れない。
     この 3 か所は (a) の 3 構文である。
<1>1b. (c)。
  <2>1. (c1)。(a) の 3 構文が作る束縛は、`Let(v, rhs, k)` の `v`、`Destructure(c, fs, s, k)` の `fs` の
        各変数、`Match` の各アームの `payload` である。DEF-0 の (v-1) の表は、1 つ目を `rhs` が `Match`
        であるかどうかで 2 行に分け、2 つ目と 3 つ目に 1 行ずつを当てている。
    BY <1>1, DEF-0 の (v-1) の表, <ref id=b3dfa37/> (`RcRhs` の 5 種 -- `rhs` が `Match` であるかそうでないかは
       この 5 種を 2 つに分ける)
  <2>2. (c2)。`Let(v, rhs, k)` で `rhs` が `Match` でないとき、`ρ` は `k` の根に進む前にその `Let` の
        節点を通り、`v` は `rhs` の値を持つ。`Destructure(c, fs, s, k)` でも同じく、`ρ` は `k` の根に
        進む前にその `Destructure` の節点を通り、各 `(i, x)` の `x` は容器の第 `i` フィールドの値を持つ。
        `Let(v, Match(s, arms), k)` では、`ρ` は `k` の根に進む前に `α` が選んだアーム本体の実行路を
        辿り終える -- すなわちそのアーム本体の終端の `Ret` を通る -- ので、`v` はその `Ret` が名指す
        変数の値を持つ。`Match` のアーム `A` の `payload` については、`ρ` が `A` の `body` の根の節点を
        通るのは `α` が `A` を選んだときに限り、そのとき `A` の payload 束縛が `v` に値を与える。
    BY <ref id=b3dfa37/> (節点の 6 種の表 -- `Let` は `rhs` の値を `x` に束縛し、`Destructure` は各 `(i, x)` の `x` に
       第 `i` フィールドを束縛する。および `MatchArm` の `payload` の欄), <ref id=ca36627/> (`Let(x, Match(v, arms), k)`
       ではアームを 1 つ選び、そのアーム本体の実行路を辿り、その後 `k` へ進む。アーム本体の `Ret` は
       そのアーム本体の実行路を終える), <ref id=c232680/> (`α` が選ぶアームは決まっている), <1>1,
       <ref id=9d74736/> の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の束縛変数の値は `x` の値で
       ある。」)
  <2>3. (c3)。授与位置はそのスコープの根の節点であり (DEF-0 の (v-1))、`ρ` が部分木の節点を通るには
        その部分木の根の節点を先に通る。
    BY DEF-0 の (v-1) (授与位置はスコープの根の節点である),
       <ref id=b3dfa37/> (束縛の及ぶ範囲の段落 -- `Let` と `Destructure` が束縛する変数のスコープは `k` の部分木、
       `Match` のアームの `payload` のスコープはそのアームの `body` の部分木である),
       <ref id=ca36627/> (実行路は根から辿るので、部分木の節点を通る前にその根の節点を通る)
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>1c. (d)。
  <2>1. `v` の名前は最上位の記号の名前であり、局所名ではない。
    BY DEF-0 の (v-3) (`v` の名前は `vars.bindings` に束縛を持たない),
       <ref id=596a46d/> (「**束縛を持たない名前は、必ず最上位の記号の名前である。**」),
       <ref id=cb35ab1/> (「**最上位の記号の名前は局所名ではない。**`FullName::is_local` が偽であり、`prog.funcs` の
       鍵と `global_types` の鍵はどちらもそのような名前である」)
  <2>2. 局所でない名前の値は、`declare_program_global` が用意する 2 つのうちの一方である -- 型が
        `is_funptr` なら `declare_lambda_function` が返す LLVM 関数の番地、そうでなければ
        `add_global_object` が登録するグローバルのアクセサが返す値である。
    **数え上げるのは名前から値を引く関数であって、それを呼ぶファイルではない。** 名前から
    `ScopedValue` を引くのは `get_scoped_value` だけであり、それを呼ぶのは `get_scoped_obj` と
    `get_scoped_obj_noretain` の 2 つだけである -- 第 1 節の前提がその走査である
    (`get_scoped_obj_field` は前者を呼ぶ)。**`Llvm` 節点の
    オペランドはこの 3 つを `builtin.rs` の側から通る** -- `codegen.rs` の `RcRhs::Llvm` の腕は
    `llvm_gen.generate_tail(..)` を呼ぶだけで、オペランドを読むのは op の生成コードだからである。
    D6 より (v-3) の名前は `Llvm` のオペランドとしても現れうるので、この道も数える。どちらの側でも
    値は `get_scoped_value` を通り、局所でない名前は `get_or_declare_global` へ行く。
    **`declare_program_global` はこの 2 つのどちらも用意しない場合を持つ** -- `global_types` に無い
    名前には `None` を返し、そのとき `get_or_declare_global` は `panic!` で止まる。**その `panic!` は
    `develop_mode` の門を持たない。** README の第 4 節はその形を「**検査して診断を出す。**」の段に
    入れ、そのプログラムは走らないのでその本体の活性化は存在しない。よって走る本体ではその名前は
    `global_types` の鍵であり、`declare_program_global` は 2 つのうちの一方を用意する。
    BY <2>1, README の第 4 節 (「**コード生成が `expect` や `unreachable!` で止まる形も、
       `develop_mode` の門を持たない限りこの段に入る** -- そのプログラムは走らないので、その本体の
       活性化は存在しない。」),
       前提 `get_scoped_value` を呼ぶ式の在りか,
       前提 `build_capture_project` を呼ぶ式の在りか,
       CODE src/generator.rs: Generator::get_scoped_obj, Generator::get_scoped_obj_noretain,
       Generator::get_scoped_obj_field (`get_scoped_obj_field` は `get_scoped_obj` を呼ぶ。この 3 つに
       名前を渡す呼び出しを `src/` 全体で数えると、`src/rc_ir/codegen.rs` に 12 か所、`Llvm` 節点の
       オペランドを読む `src/fixstd/builtin.rs` の op の生成コードに 127 か所、`src/generator.rs` に
       2 か所、`src/ast/export_statement.rs` と `src/build/build_object_files.rs` に 1 か所ずつある。
       `src/generator.rs` の 2 か所は、`get_scoped_obj_field` の中の `get_scoped_obj` と、
       `Generator::build_capture_project` の `self.get_scoped_obj_noretain(cap_name)` である。
       後者を呼ぶのは `src/fixstd/builtin.rs` の `InlineLLVMCaptureProjectBody` の生成コードだけなので、
       これも `Llvm` 節点のオペランドを読む道である。残る 2 か所 --
       `src/ast/export_statement.rs` と `src/build/build_object_files.rs` -- は環境 (<ref id=243ae2c/>) の側で
       あって、本体の節点ではない),
       CODE src/generator.rs: Generator::build_capture_project,
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMCaptureProjectBody (`generate` が
       `gc.build_capture_project(..)` を呼ぶ),
       CODE src/rc_ir/codegen.rs: Generator::eval_rc_expr_inner, Generator::eval_rc_rhs,
       Generator::eval_rc_match (`RcExpr::Let` の `RcRhs::Llvm` の腕は `llvm_gen.generate_tail` を
       呼び、オペランドを自分では読まない),
       CODE src/ast/inline_llvm.rs: LLVMGen::generate_tail (`self.generate(gc, ty)` を呼ぶので、
       オペランドを読むのは各 op の `generate` である),
       CODE src/generator.rs: Generator::get_scoped_value (`var.is_local()` が偽なら
       `get_or_declare_global` へ行く。名前から `ScopedValue` を引く式はこの 1 つであり、これを呼ぶのは
       `get_scoped_obj` と `get_scoped_obj_noretain` の 2 つだけである),
       CODE src/generator.rs: Generator::get_or_declare_global (`declare_program_global` が `None` を
       返すと `panic!` で止まり、そうでなければ `declared_globals` の欄を返す),
       CODE src/generator.rs: Generator::declare_program_global (`global_types` に無い名前には
       `None` を返す。在るとき、`ty.is_funptr()` なら
       `declare_lambda_function` を返し、そうでなければアクセサ関数を作って `add_global_object` に
       登録する),
       CODE src/generator.rs: Generator::declare_lambda_function (`fn_ty.is_funptr()` のとき、
       作った関数を `self.add_global_object(name.clone(), func, fn_ty.clone())` で登録してから返す。
       これが funptr の枝で `declared_globals` の欄を書く者である),
       CODE src/generator.rs: Generator::add_global_object,
       CODE src/generator.rs: ValueAccessor::get (`is_funptr` の枝は `fun.as_global_value()` を、
       そうでない枝はアクセサの呼び出しの結果を、その名前の値とする)
  <2>3. QED
    BY <2>1, <2>2
<1>1d. (e)。
  <2>0. (v-1) と (v-2) の `v` は D6 の「その実行路の上でその時点までに値を得た変数」であり、
        `ty(v)` の inhabited な boxed leaf との対は `P` のスロットである。(v-3) の `v` との対は
        D6 の記号の位置である。
    BY DEF-0 (3 つの場合), <1>1a ((a') -- (v-1) と (v-2) は `vars.bindings` が束縛を持つ名前、
       (v-3) はそれ以外の名前である),
       <ref id=596a46d/> (「**値を得る形は 3 つあり、スロットが在るのはそのうち 2 つである。** 節点が束縛する変数と、
       パラメータ・capture (D23 の入力の束縛、D2 のスコープ) はスロットを持つ。」、束縛を持たない名前
       について「**スロットではない**」、および「`g` を束縛を持たない名前、`λ` を `ty(g)` の
       inhabited な boxed leaf とするとき、対 `(g, λ)` を**記号の位置**と呼び」)
  <2>1. (e1)。
    BY <ref id=b3dfa37/> (束縛の及ぶ範囲の段落 -- パラメータと capture のスコープは本体の全体である),
       <ref id=a502f3e/> (グローバル初期化子の `init` はパラメータも capture も持たない),
       DEF-0 の (v-2), <ref id=596a46d/> (「その実行路の上でその時点までに値を得た変数」)
  <2>2. (e2)。
    BY DEF-0 の (v-3), <ref id=596a46d/> (「**記号の位置が値を持つのは、その記号のグローバル化の段 (E5) より後の
       時点である。**」「**それでも `g` を読む節点は必ず値を読む** -- まだ初期化されていなければ、
       その節点の段が先に (E7) と (E5) を走らせるからである ((E7))」),
       <ref id=e3436e8/> の (E7) (まだ初期化されていないグローバルを読む者が居るとき、そのアクセサが初期化子の
       活性化を作り、返る前に (E5) の段が走り、アクセサはその値を記憶域へ格納する),
       <ref id=e3436e8/> の (E5) (グローバル初期化子の活性化が終端の `Ret` に着き、返す前に印が付く),
       <ref id=ff5985d/> (活性化は節点を 1 つ実行するごとに位置を 1 つ進めるので、`P` より前に通った節点の段は
       `P` までに終わっている)
  <2>3. (e3)。
    BY <1>1a ((a') -- DEF-0 の (v-1) と (v-2) は `vars.bindings` が束縛を持つ名前、(v-3) はそれ以外の
       名前であり、(v-1) と (v-2) は `Binding::Param` を持つかどうかで分かれる),
       CODE src/rc_ir/ownership.rs: VarTable::of (関数の各パラメータと capture に `Binding::Param` を
       入れる), CODE src/rc_ir/ownership.rs: VarTable::body_only (`Binding::Param` を入れない),
       CODE src/rc_ir/ownership.rs: collect_bindings (節点が束縛する変数に入れる `Binding` は
       `Move`・`Llvm`・`Producer`・`Join`・`Field`・`Payload` のいずれかであり、`Param` ではない)
  <2>4. QED
    BY <2>0, <2>1, <2>2, <2>3
<1>2. `ρ` は本体の木の各位置を高々 1 度しか通らない。
  <2>1. 本体は有限の木である。
    BY <ref id=b3dfa37/> (分岐は `Match` のアームだけであり、節点が自分自身を含むことはないので、本体は有限の木で
       ある)
  <2>2. 節点 `n` の部分木の大きさについての帰納で、`n` から始まる実行路は `n` の部分木の各位置を
        高々 1 度しか通らず、その外の位置を通らない。**README の D3 は「`n` から始まる実行路は
        `ret(n)` で終わる」を同じ帰納で示している。**
    <3>1. `n` が `Ret` のとき、`n` から始まる実行路は `n` だけからなる。
      BY <ref id=ca36627/> (関数本体の根から辿ってきて `Ret` に着いたら、そこで終わる),
         <ref id=b3dfa37/> (`Ret` は継続を持たず、唯一の終端子である)
    <3>2. `n` が `Let(x, Match(s, arms), k)` のとき、実行路は `n` の後、アームを 1 つ選んで
          そのアーム本体の実行路を辿り、その後 `k` へ進む。選んだアーム本体と `k` はどちらも `n` の
          部分木であり、`n` より小さく、互いに素であり、どちらも `n` を含まない。
      BY <ref id=ca36627/> (`Let(x, Match(v, arms), k)` では、アームを 1 つ選び、そのアーム本体の実行路を辿り、
         その後 `k` へ進む), <2>1,
         <ref id=b3dfa37/> (本体は式の節点の木であり、位置が相異なれば節点も相異なる。アームの `body` と継続 `k` は
         `n` の相異なる部分木なので、木の性質により互いに素である)
    <3>3. `n` がそれ以外のとき、実行路は `n` の後、継続 `k` へ進む。`k` は `n` の部分木であり、
          `n` より小さく、`n` を含まない。
      BY <ref id=ca36627/> (`Ret` を除く 5 種の節点では、その継続へ進む),
         <ref id=b3dfa37/> (`Ret` を除く 5 種はちょうど 1 つの継続を持つ)
    <3>4. QED
      `n` から始まる実行路は `n` を 1 度通り、その後は `n` より小さい部分木から始まる実行路の連結で
      ある -- `<3>1` では 0 個、`<3>3` では `k` の 1 つ、`<3>2` ではアーム本体と `k` の 2 つである。
      帰納法の仮定をその各々に当てると、どれもその部分木の中に留まって各位置を高々 1 度しか通らず、
      `<3>2` の 2 つの部分木は互いに素なので、連結も `n` の部分木の各位置を高々 1 度しか通らない。
      `n` 自身はどの部分木にも入らない。
      BY <2>1, <3>1, <3>2, <3>3
  <2>3. QED
    BY <2>1, <2>2, <ref id=ca36627/> (実行路は本体の根から辿って得られる) -- 本体の根に <2>2 を当てる。
<1>3. (v-1) の場合。`v` に値を与える束縛は本体に 1 つであり (A6、<1>1)、`ρ` はその束縛の節点も `v` の
      授与位置も高々 1 度しか通らない (<1>2) ので、その束縛が `v` に与える値は 1 つに定まる。`ρ` 上で
      `v` が値を持つのは授与位置以後の位置に限り (DEF-0 の (v-1))、そのどの位置でも `v` の値はその 1 つで
      ある -- 授与位置で `v` はその値を持ち (<1>1b (c2))、`v` に値を与える構文はほかに無い (<1>1)。
      `N` もその中の 1 つなので、`N` 以後 `v` の値は変わらない。
  BY <ref id=33c54dc/> (束縛変数の名前は相異なる), <1>1, <1>1b, <1>2, DEF-0 の (v-1),
     <ref id=596a46d/> (「変数の値は、それを束縛する節点の後は変わらない。」)
<1>3a. (v-2) の場合。`v` の値は活性化が始まった時点の入力の束縛が与える 1 つであり、`ρ` のどの節点も
       それを変えない。
  BY <ref id=ff5985d/> (入力の束縛は各パラメータと capture に 1 つずつの値を与える), <1>1 (節点が値を与えるのは
     (a) の 3 構文が束縛する変数だけであり、<ref id=33c54dc/> よりその名前はパラメータ・capture の名前と異なる), <ref id=33c54dc/>
<1>3b. (v-3) の場合。`v` が値を持つ `ρ` 上のどの位置でも、`v` の値は同じである。
  <2>1. `ty(v)` が `is_funptr` のとき、`v` の値は `v` の名前について宣言された 1 つの LLVM 関数の
        グローバル値の番地であり、`ρ` 上の位置に依らない。
    <3>1. `v` を読むコードは `ValueAccessor::Global(fun, ty)` の `is_funptr` の枝であり、`fun` の
          グローバル値をそのまま値とする。この枝は記憶域を読まず、関数を呼ばない。
      BY <1>1c ((d) の funptr の枝),
         CODE src/generator.rs: Generator::get_scoped_value (局所でない名前は
         `get_or_declare_global` が返す `ScopedValue` で読む),
         CODE src/generator.rs: ValueAccessor::get (`is_funptr` の枝は `fun.as_global_value()` を
         返すだけで、`build_call` の枝へ行かない)
    <3>2. `fun` は `v` の名前が `declared_globals` に持つ 1 つの欄から来るので、`v` の名前だけで
          決まる。
      BY <1>1c ((d) の funptr の枝),
         README の第 4 節 (「**コード生成が `expect` や `unreachable!` で止まる形も、`develop_mode` の
         門を持たない限りこの段に入る** -- そのプログラムは走らないので、その本体の活性化は存在
         しない。」),
         CODE src/generator.rs: Generator::get_or_declare_global (`declared_globals` に在ればその
         `ScopedValue` を返し、無ければ `declare_program_global` で用意してからその欄を返す),
         CODE src/generator.rs: Generator::declare_program_global (`ty.is_funptr()` の枝が用意するのは
         その名前の 1 つの関数である),
         CODE src/generator.rs: Generator::declare_lambda_function (`fn_ty.is_funptr()` のとき、
         作った関数を `add_global_object` で `declared_globals` へ登録してから返す。
         `declare_program_global` の funptr の枝はこの関数を呼んで戻るので、欄を書くのはここである),
         CODE src/generator.rs: Generator::add_global_object (`declared_globals` へ入れるのはここだけで
         あり、同じ名前を 2 度入れようとすると `panic_with_msg` で止まる。その `panic_with_msg` は
         `develop_mode` の門を持たないので、README の第 4 節が「**検査して診断を出す。**」と呼ぶ段に
         当たり、走る本体では
         1 つの名前の欄は 1 つである)
    <3>3. QED
      `v` の値を作る式は記憶域を読まず (`<3>1`)、その式が返すグローバル値 `fun` は `v` の名前だけで
      決まる (`<3>2`)。よって `ρ` のどの位置でも `v` の値は同じ 1 つの LLVM グローバル値である。
      **この段は D24 の段の一覧も、「番地はリンクが決める定数である」も読まない** -- 値が記憶域から
      来ないので、どの段が何を書くかは答えに入らない。
      BY <3>1, <3>2
  <2>2. そうでないとき、`v` の値は `v` が名指す記号の記憶域が持つ値であり、`v` が値を持つ `ρ` 上の
        どの位置でも同じである。
    <3>1. `v` が `P` で値を持つならば、`P` における `v` の読みはその記号の (E5) の段より後にあり、
          その記号の記憶域はその読みの時点で初期化されている。
      BY DEF-0 の (v-3), <1>1d ((e2)),
         <ref id=596a46d/> (「**記号の位置が値を持つのは、その記号のグローバル化の段 (E5) より後の
         時点である。**」「**それでも `g` を読む節点は必ず値を読む**」)
    <3>1a. `ρ` が `v` を名指して**最初に**通る節点の段は、その記号のアクセサの**1 回の実行** `A` を
           含み、`v` の値は `A` が返す値である。**`A` は活性化ではない** -- D23 の活性化は関数の `body`
           かグローバル初期化子の `init` の 1 回の実行であり、アクセサはそのどちらでもない。D22 は
           アクセサを環境の 4 つの箇条の 1 つに挙げ、D24 の (E7) はアクセサと、それが作る初期化子の
           活性化 `b` とを別のものとして書く。**「その読み」とは、`A` の末尾の `build_load` が記憶域を
           読む瞬間をいう。** `α` の位置のうち `v` が値を持つものにおける `v` の読みは、どれもこの読み
           以後にある。
      BY DEF-0 の (v-3) (`ρ` は `v` を名指す節点を `P` までに (`P` 自身を含めて) 通っている), <1>1d ((e2)),
         <ref id=ff5985d/> (活性化は 1 つの本体の 1 回の実行である),
         <ref id=243ae2c/> (環境の 4 つの箇条 -- グローバルのアクセサはその 1 つである),
         <ref id=e3436e8/> の (E7) (`g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る),
         <ref id=243ae2c/> のグローバルのアクセサの行 (「初期化済みの旗を見て、まだならグローバル初期化子の本体を
         持つ関数 `InitValue#<symbol>` を呼び、返った値を記憶域へ格納する」),
         CODE src/rc_ir/codegen.rs: Generator::implement_rc_global (アクセサは末尾の `build_load` の
         値を返す),
         CODE src/generator.rs: ValueAccessor::get (`is_funptr` でない枝はアクセサを `build_call` で
         呼び、その結果をその名前の値とする), <1>1c ((d) のアクセサの枝 -- 型が `is_funptr` でない
         局所でない名前の値は、`add_global_object` が登録するグローバルのアクセサが返す値である)
    <3>2. `<3>1a` の読みの後、`α` が終わるまでのあいだ、その記憶域は書き替えられない。
      **数え上げるのは D24 の段ではなく、その記憶域へ書き込む生成コードである。** 記号の記憶域は
      オブジェクトの記憶域ではないので、オブジェクトの中身を書く段 -- (E2) の `struct_set` や、(F) が
      デストラクタの結果を `_value` の欄へ戻す動作 -- はそこへ届かない。届く道は、その番地へ store を
      出す生成コードと、番地を渡された環境の 2 つだけである。
      <4>1. `v` が名指す記号の記憶域は、LLVM のグローバル変数 `GlobalVar#<symbol>` である。生成コードが
            そこへ store を出すのは `store_init_value` の `build_store` ただ 1 か所であり、その番地
            (`global_var_ptr`) が現れるのは `implement_rc_global` の中の 3 つ -- `store_init_value` へ
            渡す 2 つの引数 (非 threaded の枝と threaded の枝であり、1 つのビルドではその一方だけが
            生成される) と、アクセサの末尾の `build_load` -- だけである。どれもその番地を値として外へ
            渡さない。
        BY CODE src/rc_ir/codegen.rs: Generator::implement_rc_global (`global_var` を作り、その
           `global_var_ptr` を `store_init_value` と末尾の `build_load` にだけ渡す),
           CODE src/rc_ir/codegen.rs: Generator::store_init_value (`InitValue#<symbol>` を呼び、
           返った値を `global_var_ptr` へ `build_store` する)
      <4>2. `config.threaded` が真のビルドでは、1 つの実行でその store が走るのは高々 1 度である。
        BY <4>1, EXT `pthread_once`,
           CODE src/rc_ir/codegen.rs: Generator::implement_rc_global (`config.threaded` が真のとき、
           `store_init_value` の呼び出しは `InitOnce#<symbol>` の本体の中の 1 つだけであり、アクセサは
           その関数を旗 `InitFlag#<symbol>` とともに `pthread_once` へ渡す。旗の型と初期値は
           `pthread_once_init_flag_type` と `pthread_once_init_flag_value` である。旗は記号ごとに
           1 つである -- 記憶域を持たない単位はそれを `External` で宣言し、持つ単位が定義する),
           CODE src/generator.rs: Generator::call_runtime (名前で `module` から関数を引き、その関数
           への `build_call` を出す。よって `call_runtime(RUNTIME_PTHREAD_ONCE, ..)` はその名前の
           関数の呼び出しである),
           CODE src/fixstd/runtime.rs: RUNTIME_PTHREAD_ONCE (定数の値は `"pthread_once"` であり、
           その doc が libc の `pthread_once` を名指す),
           CODE src/fixstd/runtime.rs: build_pthread_once_function (その名前の関数を
           `module.add_function` で宣言する。本体を持たないので、呼び出しに着くのは libc の側の
           定義である)
      <4>2a. `config.threaded` が偽のビルドでは、その store が走るのは旗 `InitFlag#<symbol>` を 0 と
             読んだアクセサの実行の中だけであり、旗は一度 0 でなくなれば以後 0 に戻らない。
             **アクセサは入口の基本ブロックで真っ先に旗をロードする** -- その読みより前にアクセサは
             どの関数も呼ばない。
        BY <4>1, CODE src/rc_ir/codegen.rs: Generator::implement_rc_global -- `config.threaded` が
           偽のとき、アクセサは入口の基本ブロックで旗をロードして 0 のときだけ `flag_is_zero` の
           ブロックへ分岐し、その
           ブロックが `store_init_value` を呼んでから旗へ 1 を書く。入口の基本ブロックはその
           ロードと分岐だけからなり、呼び出しを持たない。旗へ store を出す生成コードは
           この 1 か所だけであり、書く値は 1 である。旗の静的な初期値は 0 である。旗の番地
           (`init_flag_ptr`) が現れるのは `implement_rc_global` の中の 3 つ -- 旗のロード、この
           store、`pthread_once` へ渡す引数 (threaded の枝だけ) -- であり、値として外へ渡らない,
           <ref id=243ae2c/> (環境の `FFI_CALL` の行 -- 環境は「Fix の側から番地を渡され、その番地の指すものを
           読み書きする」ので、渡されていない番地を書かない)
      <4>2b. `config.threaded` が偽のビルドでは、`<3>1a` の読みの後に走るその store は、`α` が
             終わった後に走る。
        <5>0. 以下は 1 つの制御の流れについて読む。
          BY <ref id=e3436e8/> (「**`config.threaded` が偽のビルドで制御の流れが 2 つある実行は、このモデルの外に
             ある。** そのビルドのグローバルのアクセサは旗 `InitFlag#<symbol>` を同期なしに読み書き
             するので、2 つの流れがそれを同時に触る実行の振る舞いは LLVM の定めるところで未定義で
             ある。(E6) が実行時検査を切ったビルドの `undefined` について置くのと同じ形の節である。
             読む者は `p11-origin-soundness.md` の `L10` である。」)
        <5>1. `<3>1a` の読みを行うアクセサの実行 `A` が返る時点で、旗は 0 でない。
          BY <5>0, <3>1a (その節点の段はアクセサの実行 `A` を含み、その読みは `A` の末尾の
             `build_load` である), <4>2a -- `A` が旗を 0 と読んだ
             ならば `A` は `flag_is_zero` のブロックを通り、そこで旗へ 1 を書いてから返る。0 でないと
             読んだならば旗はその時点で既に 0 でなく、以後 0 に戻らない (<4>2a)。
        <5>2. その読みの後に走る store を行うアクセサの実行 `A'` は、`A` が返るより前に旗を読んで
              おり、その store は `A'` が呼ぶ `InitValue#<symbol>` の活性化 `b'` が終わった後に走る。
          BY <5>0, <4>2a (store を行うのは旗を 0 と読んだ実行だけである), <5>1 (旗は `A` が返る時点で
             0 でなく、以後 0 に戻らない),
             CODE src/rc_ir/codegen.rs: Generator::store_init_value (`InitValue#<symbol>` を
             `build_call` で呼び、返った値を store する。store はその呼び出しが返った後にある),
             <ref id=e3436e8/> の (E7) (`g` のアクセサが `g` の初期化子の `init` の活性化を作る),
             EXT 呼び出しの入れ子
        <5>2a. `A` は `b'` の中に在る。すなわち `b'` は `A` が始まるより前に始まり、`A` が返った後に
               終わる。
          1 つの制御の流れでは呼び出しは後入れ先出しに入れ子になる (EXT 呼び出しの入れ子) ので、
          `A'` と `A` の関係は 3 つのいずれかである -- `A'` が `A` の始まる前に終わっている、`A'` が
          `A` の中に在る、`A` が `A'` の中に在る。第 1 の場合、`A'` の store は `A` が始まるより前に
          走るので、この読みより後ではない。第 2 の場合、`A'` は `A` が返るより前に終わるので、その
          store も `A` が返るより前、すなわちこの読みより前に走る。どちらもこの段の前提に反する。
          よって第 3 の場合であり、`A` は `A'` の中に在る。**`A'` が旗を読むのは入口の基本ブロックで
          あり、その読みより前に `A'` はどの関数も呼ばないので** (<4>2a)、`A` は `A'` が旗を 0 と
          読んだ後に始まる。**`A'` の store はこの段の前提よりこの読みより後にある**ので、`A` は
          `A'` の store より前に終わる。`A'` が旗を 0 と読んでからその store までの
          あいだに `A'` が行う呼び出しは `InitValue#<symbol>` の 1 つだけなので、`A` はその活性化
          `b'` の中に在る。
          BY <5>0, <5>2, <4>2a, EXT 呼び出しの入れ子,
             CODE src/rc_ir/codegen.rs: Generator::implement_rc_global (旗を 0 と読んだ枝が行うのは
             `store_init_value` の呼び出しと旗への store だけである),
             CODE src/rc_ir/codegen.rs: Generator::store_init_value (`InitValue#<symbol>` の
             `build_call` と `build_store` だけからなる)
        <5>3. その読みの時点で `b'` は生きており (D23)、`b'` は `α` の祖先か `α` 自身である。
          <6>1. その読みの時点で `b'` は生きている。
            BY <5>2a (`b'` は `A` が始まるより前に始まり、`A` が返った後に終わるので、その読みの
               時点で始まっていて終わっていない),
               <ref id=ff5985d/> (生きている活性化とは、始まって終わっていない活性化である)
          <6>2. その読みの時点で、`α` の子孫の活性化は生きていない。
            **1 つの段は子の活性化を 2 つ以上作りうるので、`A` が作る `b` だけを見ては足りない。**
            `α` の位置の節点の段は、`A` より前に別のグローバルのアクセサを走らせてその初期化子の
            活性化を作りうるし、オペランドを適用する `Llvm` の段はそれ自身が活性化を作る。以下の
            `<7>2` がその全体を扱う。
            <7>1. 活性化 `a` の子の活性化はどれも、`a` の制御の流れが行う 1 つの呼び出しの中で作られ、
                  その呼び出しが返るまでに終わる。よって `a` の子が終わった時点で、その子の子孫の
                  活性化もすべて終わっている。
              BY <5>0, <ref id=e3436e8/> の活性化の林 (子を作る段は (E3) と (E7)、(E2) のうち
                 オペランドを適用する `Llvm` の段、および (F) の解放が `Destructor` について作る段で
                 あり、どれもその子の本体を呼び出しとして走らせる。「子は親が中断中の間だけ段を持ち、
                 親は子が終わってから再開する」),
                 <ref id=e3436e8/> の (E3) (「`a` は `b` が終わるまで**中断中**であり、その間 `a` は
                 段を持たない」),
                 EXT 呼び出しの入れ子, <ref id=ff5985d/> (活性化は 1 つの本体の 1 回の実行である)
            <7>2. `A` が始まるより前に `α` が作った子の活性化は、その読みの時点でどれも終わっている。
                  その読みは `A` の末尾の `build_load` の瞬間であり (`<3>1a`)、`A` は `α` の位置の
                  節点の段の中で走る 1 つの呼び出しである。`A` より前に `α` が作った子は、それを
                  作った呼び出しが `A` の呼び出しより前に返っているので、`A` が始まる時点で既に
                  終わっている (`<7>1`)。**同じ段が `A` より前に作った子もここに入る** -- その段が
                  先に読んだ別のグローバルのアクセサが作る初期化子の活性化も、オペランドを適用する
                  `Llvm` の段が作る活性化も、`A` の呼び出しより前に返った呼び出しの中に在る。
              BY <5>0, <7>1, <3>1a (その読みは `A` の末尾の `build_load` である),
                 EXT 呼び出しの入れ子, <ref id=ff5985d/> (終わった活性化はそれ以後の段を持たない)
            <7>3. `A` が始まってからその読みまでに `α` が作る子の活性化は、その読みの時点で終わって
                  いる。その間 `α` の制御の流れは `A` の中に在るので、作られる活性化は `A` が行う
                  呼び出しの中に在る。非 threaded の枝で `A` が行う呼び出しは `store_init_value` の
                  1 つだけであり、それが作る活性化は `InitValue#<symbol>` の `b` である。その呼び出しは
                  末尾の `build_load` より前に返るので、`b` はその時点で既に終わっている (`<7>1`)。
              BY <5>0, <7>1, <3>1a (その読みは `A` の末尾の `build_load` である),
                 <ref id=e3436e8/> の (E7) (`g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。読む者は
                 `g` を読む節点の位置にある生きている活性化 `a` である),
                 <ref id=e3436e8/> の活性化の林 ((E7) が作る活性化は、それを作った活性化の子である),
                 CODE src/rc_ir/codegen.rs: Generator::implement_rc_global (非 threaded の枝では
                 アクセサが行う呼び出しは `flag_is_zero` のブロックの `store_init_value` の 1 つだけで
                 あり、その呼び出しは末尾の `build_load` より前に返る),
                 CODE src/rc_ir/codegen.rs: Generator::store_init_value (`InitValue#<symbol>` の
                 `build_call` と `build_store` だけからなる),
                 EXT 呼び出しの入れ子
            <7>4. QED
              その読みより前に `α` が作った子の活性化は `<7>2` と `<7>3` で尽き、どちらもその時点で
              終わっている。その読みより後に作られる子はその時点でまだ始まっていない。終わった子の
              子孫もその時点で終わっているので (`<7>1`)、`α` の子孫の活性化は 1 つも生きていない。
              BY <5>0, <7>1, <7>2, <7>3,
                 <ref id=ff5985d/> (生きている活性化とは、始まって終わっていない活性化である)
          <6>3. QED
            BY <6>1, <6>2, <5>2a, EXT 呼び出しの入れ子,
               <ref id=e3436e8/> の活性化の林 (「子は親が中断中の間だけ段を持ち、親は子が終わってから再開するので、
               1 つの制御の流れの中で生きている活性化は根から下への 1 本の道をなす」),
               <ref id=e3436e8/> の (E3) (「`a` は `b` が終わるまで**中断中**であり、その間 `a` は段を持たない」)
               -- その読みを行う節点を実行しているのは `α` であり、その時点で `α` の子孫は生きて
               いない (<6>2) ので、`α` はその道の末端に在る。`b'` はその時点で生きている (<6>1) ので、
               `α` の祖先か `α` 自身である。
        <5>4. QED
          `α` は `b'` の子孫か `b'` 自身なので、`α` は `b'` が終わるより後まで生きていない。
          `A'` の store は `b'` が終わった後に走る (`<5>2`) ので、`α` が終わった後に走る。
          BY <5>2, <5>2a, <5>3, EXT 呼び出しの入れ子
      <4>3. 環境はその記憶域を書かない。
        BY <4>1 (その番地は生成コードの外へ渡らない),
           <ref id=243ae2c/> (環境の `FFI_CALL` の行 -- 環境は「Fix の側から番地を渡され、その番地の指すものを
           読み書きする」),
           <ref id=c9e4cca/> の (ii-b) (「**記号の記憶域 (`GlobalVar#<symbol>`) も書かない。**それは計数下の
           オブジェクトではないので、前半の節はそこも覆わない。読む者は
           `p11-origin-soundness.md` の `L10` である」)
      <4>4. QED
        BY <3>1, <3>1a, <4>1, <4>2, <4>2a, <4>2b, <4>3,
           <ref id=e3436e8/> の (E7) (「アクセサはその値を `g` の記憶域へ格納する」-- `<3>1` の言う初期化がこの
           格納である) -- その記憶域へ書く道は `<4>1` の store と環境の 2 つで尽き (<4>1)、環境は
           書かない (<4>3)。store の側は `config.threaded` の真偽で分かれる。真のビルドでは store は
           1 つの実行で高々 1 度しか走らない (<4>2)。その 1 度は `A` が返るまでに済んでいる --
           アクセサは旗がまだなら格納してから値を返し (<ref id=243ae2c/> のグローバルのアクセサの行)、`v` は `P` で
           値を持つので記憶域はその読みの時点で初期化されている (<3>1、<3>1a)。偽のビルドでは、その
           読みの後に走る store は `α` が終わった後である (<4>2b)。どちらでも、その読みから `α` が
           終わるまでのあいだ、記憶域へ書く動作は 1 つも無い。
    <3>3. QED
      BY <1>1c ((d) のアクセサの枝), <3>1, <3>1a, <3>2 -- `v` が値を持つ `ρ` 上の各位置における
         `v` の読みは、`<3>1a` の読み以後にあり (<3>1a)、その読みから `α` が終わるまで記憶域は
         書き替えられない (<3>2)。よって `v` の値はそのどの位置でも同じである。
  <2>3. QED
    BY <1>1c, <2>1, <2>2, DEF-0 の (v-3) -- (d) より場合はこの 2 つで尽きている。`v` が `N` で値を
       持つならば、`ρ` は `v` を名指す節点を `N` までに通っているので `N` 以後のどの位置でも `v` は
       値を持ち、その値は `v` が値を持つどの位置でも同じである。
<1>4. QED
  (a) は `<1>1`、(a') は `<1>1a`、(c) は `<1>1b`、(d) は `<1>1c`、(e) は `<1>1d` が与える。
  (b) は、DEF-0 の 3 つの場合が尽きており (`<1>1d` の (e3))、どの場合も `N` 以後 `v` の値が
  変わらないこと (`<1>3`、`<1>3a`、`<1>3b`) から出る。
  BY <1>1, <1>1a, <1>1b, <1>1c, <1>1d, <1>3, <1>3a, <1>3b

**L11 (再帰の辺の行き先の `RcVar` も値を持つ)**: 活性化 `α`、それが辿る実行路 `ρ`、`ρ` 上の位置 `P`、 <!--#9b70f28-->
`P` で値を持つ (DEF-0) `RcVar` `x` を取る。`x` の名前が `vars.bindings` に持つ束縛が名指す `RcVar` --
`Move(y)` の `y`、`Payload(s, ・)` の `s`、`Field(c, i)` の `c`、`Llvm(gen, args, ・)` の各 `args[j]`、
`Join(rs)` のうち `α` が選んだアームの本体の終端の `Ret` が名指す変数 `r_0` -- は、いずれも `P` で値を
持つ (DEF-0)。**さらに、`x` が DEF-0 の (v-1) であるとき、`ρ` は `x` の束縛を作る節点を `P` までに
通る。** 後半を言明に置くのは、D20 が別名の辺の存在に「**辺が在るのは、その辺を定める節点が実行路の
上に在り、かつその節点と leaf `λ` が作る 2 つの対がどちらもその路の位置であるときであり、そのときに
限る。**」と条件を課すからである。

**この命題は行き先が D6 のスロットを持つとは言わない。** 行き先が (v-3) の名前 -- グローバル値を読む
`RcVar` -- でありうるからである。`Let(x, Var(g), k)` (`g` はグローバル値) の `g` がその形であり、
README の P3 が「条件を落とすと `Let(x, Var(g), k)` (`g` はグローバル値) で両端が参照を持たず、言明が
意味を失う」と、まさにこの形を挙げる。行き先を (v-1) と (v-2) に絞るのは L13 の仕事であり、
L17 の (iii) はそこで仮定 (H) を使う。

<1>1. `x` は DEF-0 の (v-1) か (v-2) である。(v-2) のとき `x` の束縛は `Binding::Param` であり、
      どの `RcVar` も名指さないので主張は空虚である。以下 `x` は (v-1) であるとし、`x` の束縛を作る
      節点を `N` とする。`ρ` は `N` を `P` までに通る。
  BY DEF-0 (`x` は束縛を持つので (v-3) ではない。(v-1) より `ρ` は `x` の授与位置を `P` までに通る。
     (v-1) の表の 4 行のいずれでも、授与位置は `N` の部分木の中に在る -- 第 1 行から第 3 行の `k` も、
     第 4 行のアーム `A` の `body` も、`N` が持つ部分木である), <ref id=ca36627/> (実行路は木を根から辿るので、
     部分木の節点を通る前にその根の節点を通る), <ref id=49da857/> (a'), <ref id=49da857/> (c1),
     <ref id=33c54dc/> (束縛変数の名前は相異なる),
     CODE src/rc_ir/ownership.rs: Binding (`Param` は `RcVar` の欄を持たない),
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Let` は束縛する変数に `Move` / `Llvm` /
     `Producer` / `Join` を、`Destructure` は各名前付きフィールド変数に `Field` を、`Match` の各アームは
     その `payload` に `Payload` を作る。名前は相異なる (<ref id=33c54dc/>) ので、変数と節点の対応は 1 対 1 である。
<1>1a. `ρ` が `P` までに通る節点 `M` に書かれたオペランド `v` は、`P` で値を持つ (DEF-0)。
  <2>1. CASE: `v` の名前が `vars.bindings` に束縛を持たない。
    `v` は節点 `M` に書かれたオペランドであり、`ρ` は `M` を `P` までに通るので、DEF-0 の (v-3) の
    条件 (`ρ` が `v` を名指す節点を `P` までに通っていること) が満たされる。
    BY DEF-0 の (v-3),
       <ref id=596a46d/> (「**それでも `g` を読む節点は必ず値を読む**」-- `M` の段は、まだ初期化されていなければ
       先に (E7) と (E5) を走らせる)
  <2>2. CASE: `v` の名前が `Binding::Param` を持つ。
    BY DEF-0 の (v-2) -- パラメータと capture は `ρ` のどの位置でも値を持つ。
  <2>3. CASE: `v` の名前が L10 (a) の 3 構文が作る束縛を持つ。
    A11 が言うのは「変数の使用は、その位置でスコープに入っている束縛に解決する」までであり、その束縛が
    `vars.bindings` の記録と同じものであることは言わない。それを与えるのは A6 である -- 束縛変数の
    名前は相異なるので、`v` の名前を束縛する節点は本体に 1 つであり、`collect_bindings` が `v` の
    名前に記録する `Binding` はその節点が作るものである。
    BY <ref id=3905b4e/> (`M` の位置での `v` の使用は、その位置でスコープに入っている束縛に解決する),
       <ref id=33c54dc/> (束縛変数の名前は相異なる。よって名前は束縛を一意に決める),
       <ref id=49da857/> (a') (`vars.bindings` の束縛のうち `Binding::Param` でないものは (a) の 3 構文が作る),
       <ref id=49da857/> (c1) (DEF-0 の (v-1) の表の 4 行は (a) の 3 構文が作る束縛を尽くす),
       <ref id=49da857/> (c3) (`M` はその束縛の <ref id=b3dfa37/> の意味のスコープに在り、`ρ` は `M` を通るので、`ρ` は `v` の
       授与位置を `M` までに通る),
       CODE src/rc_ir/ownership.rs: collect_bindings (`v` の名前に `Binding` と型を入れるのは、
       `v` を束縛する節点を訪れた 1 か所である),
       DEF-0 の (v-1) (`ρ` は `v` の授与位置を `M` までに、したがって `P` までに通るので、`v` は `P` で
       値を持つ)
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=49da857/> (a') -- `vars.bindings` に入る名前は `Binding::Param` を持つものと
       (a) の 3 構文が束縛するものだけなので、この 3 つの場合は尽きている。
<1>2. `Move(y)`、`Payload(s, ・)`、`Field(c, i)`、`Llvm(・, args, ・)` の場合、名指される `RcVar` は
      節点 `N` に書かれたオペランドである。`ρ` は `N` を `P` までに通るので、<1>1a よりそれは `P` で値を
      持つ。
  BY <1>1, <1>1a, CODE src/rc_ir/ownership.rs: collect_bindings --
     `Binding::Move(y)` は `Let(x, Var(y), k)` の `y`、`Binding::Payload(scrut, ・)` は
     `Let(x, Match(scrut, arms), k)` の `scrut`、`Binding::Field(container, i)` は
     `Destructure(container, fs, s, k)` の `container`、`Binding::Llvm(・, args, ・)` は
     `Let(x, Llvm(gen, args), k)` の `args` である。
<1>3. `Join(rs)` の場合、`N` は `Let(x, Match(scrut, arms), k)` である。`ρ` は `α` が選んだアーム本体の
      終端の `Ret` を `P` までに通る。`r_0` はその `Ret` に書かれたオペランドであり、<1>1a より `r_0` は
      `P` で値を持つ。
  BY <1>1 (`x` は (v-1) である), <1>1a,
     DEF-0 の (v-1) の表の第 2 行 (`x` の授与位置は `k` の根の節点であり、(v-1) より `ρ` はそれを
     `P` までに通る), <ref id=49da857/> (c2) (`ρ` はその位置へ進む前に `α` が選んだアーム本体の終端の `Ret` を通る),
     <ref id=c232680/> (活性化が選ぶアームは決まっている),
     CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
     CODE src/rc_ir/ownership.rs: returned_var (本体の終端の `Ret` が名指す変数)
<1>4. QED
  BY <1>1, <1>1a, <1>2, <1>3 -- 前半は <1>2 と <1>3 が場合ごとに与える。後半 (`ρ` が `x` の束縛を作る
     節点を `P` までに通ること) は <1>1 が与える。

**L12 (値の leaf が参照を持つのは、計数下のオブジェクトを指すときである)**: `v` を `P` で値を持つ <!--#eb90864-->
(DEF-0) `RcVar`、`λ` を `ty(v)` の boxed leaf であって `P` で inhabited (D16) であるものとする。`v` の値の
leaf `λ` が D8 の意味の参照を持つことと、その leaf が指すオブジェクトが計数下 (D26) であることは同値で
あり、持つときその個数はちょうど 1 である。

`v` が DEF-0 の (v-1) か (v-2) であるとき、`(v, λ)` は D6 のスロットであり、この命題はそのスロットが
持つ参照について述べる。(v-3) のときスロットは無いが、値の leaf は在るので、言明はそのまま読める。

**A5 の配列の記憶域の例外は、この命題に当たらない。** その例外が読み替えるのは `#ArrayStorage` の
オブジェクトが**保持する**参照の単位 -- leaf ではなくその記憶域の各スロット -- であって、
D25 の 2 つ目の持ち手の数え方である。この命題が数えるのは変数の値の leaf が持つ参照であり、`Array` にも
`#ArrayStorage` にも `boxed_leaf_paths` が返す leaf は 1 つである (A5)。

<1>1. `λ` が計数下のオブジェクトを指すとき、`v` の値の leaf `λ` にはちょうど 1 つの参照がある。
  BY <ref id=4f63121/> (「値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf のうち、
     inhabited (D16) であって計数下のオブジェクト (D26) を指すものにちょうど 1 つずつある」)
<1>2. `λ` がグローバル状態のオブジェクトを指すとき、`v` の値の leaf `λ` は D8 の意味の参照を持たない。
  BY <ref id=4f63121/> (「グローバル状態のオブジェクトを指す leaf も参照を持たない (D26)」), <ref id=88a06de/> (「グローバル状態の
     オブジェクトを指す leaf は、D8 の意味の参照を持たない」)
<1>3. オブジェクトは計数下かグローバル状態かのどちらかである。
  BY <ref id=88a06de/> (「オブジェクトは**計数下**か**グローバル状態**かのどちらかである」)
<1>4. QED
  BY <1>1, <1>2, <1>3, <ref id=66c9670/> (「**null ポインタの leaf は inhabited でない。**」) -- `λ` は inhabited
     なので null ポインタの leaf ではなく、<ref id=4f63121/> が例外に挙げる capture が空のクロージャの capture の
     leaf には当たらない。

**DEF 値からの到達**。値 `w` から**到達できる**オブジェクトとは、`w` の inhabited (D16) な各 boxed
leaf が指すオブジェクトと、そこから D25 の意味で到達できるオブジェクトの全体である。**この定義が要るのは、
D25 が定めるのがオブジェクトからオブジェクトへの到達だからである** -- A8 の「グローバル値が到達する
オブジェクト」と D24 の (E5) の「その値が到達するオブジェクトのグラフ全体」は値を起点に取るので、
その起点の一歩をここで定める。その一歩を値の inhabited な boxed leaf に置く根拠は A5 であり、それを
引くのは L13 である。

**L13 (束縛を持たない名前の値はグローバル状態のオブジェクトだけを指す)**: `v` を、`P` で DEF-0 の <!--#596d4c9-->
(v-3) として値を持つ `RcVar` -- その名前が `vars.bindings` に束縛を持たず、`ρ` が `v` を名指す節点を
`P` までに通っているもの -- とする。`P` における `v` の値の inhabited な各 boxed leaf が指す
オブジェクトは、グローバル状態 (D26) である。よって L12 より、その leaf は D8 の意味の参照を持たない。

**この命題が D6 に足すもの。** D6 の「**束縛を持たない名前は、必ず最上位の記号の名前である。**」の
段落は、その名前について
「そこが指すのは funptr かグローバル状態のオブジェクト」と、2 つを並べたまま述べる。L13 はその 2 つを
分け、funptr の側には boxed leaf が無いこと (`<1>3`) を示して、**inhabited な各 boxed leaf について**
グローバル状態であると言う。L12 が要求するのはこの形である。

**この条件を与えるのは D6 である。** D6 の「**束縛を持たない名前は、必ず最上位の記号の名前である。**」が
それであり、`Lowerer::lower_var` の
`resolve` が `None` を返す枝に立つ `assert!(!v.name.is_local(), ..)` は、この条件が成り立つ理由では
ない -- README の第 4 節が「**表明は不変条件の出どころであって、仮定を果たす者ではない。**」と述べる
とおりである。

<1>1. `v` の名前は、そのプログラムの最上位の記号の名前である。
  BY DEF-0 の (v-3) (`v` の名前は `vars.bindings` に束縛を持たない),
     <ref id=596a46d/> (「**束縛を持たない名前は、必ず最上位の記号の名前である。**」), <ref id=49da857/> (d)
<1>1a. `v` の名前は局所名ではない。すなわち `FullName::is_local` が偽である。
  BY <1>1, <ref id=49da857/> (d), <ref id=cb35ab1/> (「**最上位の記号の名前は局所名ではない。**`FullName::is_local` が偽であり、
     `prog.funcs` の鍵と `global_types` の鍵はどちらもそのような名前である」)
<1>2. `v` の値は、`declare_program_global` が用意する 2 つのうちの一方から来る -- 型が `is_funptr` なら
      `declare_lambda_function` が返す関数の番地、そうでなければ `add_global_object` が登録する
      グローバルのアクセサが返す値である。
  BY <1>1a, <ref id=49da857/> (d)
<1>3. 型が `is_funptr` のとき、`ty(v)` は boxed leaf を持たない。よって主張は空虚である。
  <2>1. `ty(v)` の最上位の tycon `tc` は、名前空間が `Std` の 1 段であって名前が `#FunPtr` で始まる。
        よってその tycon は `bulitin_tycons()` が置く鍵の 1 つであり、`type_env.tycons()` がその鍵の
        下に持つ項目は `make_funptr_tycon(n)` の項目 -- `is_unbox` が真、`variant` が
        `TyConVariant::Primitive` であるもの -- である。
    BY <ref id=8412761/> (プログラムに現れる型は ground であり、その tycon は `type_env` にある),
       <ref id=3d4be43/> (`E.tycons()` の項目のうち鍵が `bulitin_tycons()` の置く鍵のいずれかであるものは
       `bulitin_tycons()` がその鍵の下に置いた項目であり、`tc.name.namespace` が `Std` の 1 段で
       `tc.name.name` が `FUNPTR_NAME` で始まる鍵の項目がそうである),
       CODE src/ast/types.rs: TypeNode::is_funptr, TypeNode::toplevel_tycon_satisfies
       (`is_funptr` が真であるのは、最上位の tycon が在って `is_funptr_tycon` がそれに `Some` を
       返すときである),
       CODE src/fixstd/builtin.rs: is_funptr_tycon (`Some` を返すのは、名前空間が `Std` の 1 段で
       あって名前が `FUNPTR_NAME` で始まるときである),
       CODE src/fixstd/builtin.rs: bulitin_tycons (`make_funptr_tycon(arity)` の項目は
       `is_unbox: true`、`variant: TyConVariant::Primitive` である),
       CODE src/constants.rs: FUNPTR_NAME (`"#FunPtr"`)
  <2>2. `is_box(ty(v))` は偽である。
    BY <2>1, CODE src/ast/types.rs: TypeNode::is_unbox (`is_closure() || toplevel_tycon_info(type_env).is_unbox`
       -- 前者が真なら `is_unbox` は真であり、偽なら後者を読む。後者は `type_env.tycons()` を最上位の
       tycon で引いた項目の `is_unbox` の欄であり、<2>1 よりそれは真である),
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info (型の `TyConInfo` は、その最上位の
       tycon で `type_env.tycons()` を引いた 1 つである),
       CODE src/ast/types.rs: TypeNode::is_box (`!self.is_unbox(type_env)`)
  <2>3. `is_closure(ty(v))` と `is_array(ty(v))` はどちらも偽である。
    BY <2>1, CODE src/ast/types.rs: TypeNode::is_closure (最上位の tycon の名前が
       `make_arrow_name_abs()` に等しいかどうか),
       CODE src/ast/types.rs: TypeNode::is_array, TypeNode::toplevel_tycon_satisfies (`is_array` は
       最上位の tycon が `is_array_tycon` を満たすかどうかである),
       CODE src/fixstd/builtin.rs: is_array_tycon, make_array_name (`is_array_tycon` は tycon が
       `make_array_tycon()` に等しいことであり、その名前は `Std` の下の `ARRAY_NAME` である),
       CODE src/fixstd/builtin.rs: make_arrow_name_abs (`Std` の下の `ARROW_NAME` である),
       CODE src/constants.rs: ARRAY_NAME, ARROW_NAME, FUNPTR_NAME -- `"Array"` も `"Arrow"` も
       `"#FunPtr"` で始まらないので、<2>1 の `tc` の名前はそのどちらとも異なる
  <2>4. QED
    BY <2>2, <2>3, <ref id=83d98e9/> (束縛を持たない `RcVar` の型は、その名前の記号の型である), <ref id=0594f24/> の第 1 の規則
       (`is_fully_unboxed` が真の型は leaf を持たない),
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed (`is_box`・`is_closure`・`is_array` が
       いずれも偽で `is_funptr` が真の型に真を返す),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (`is_fully_unboxed` の型で走査が `return` する)
<1>4. そうでないとき `v` はグローバル値であり、`P` における `v` の値の inhabited な各 boxed leaf が
      指すオブジェクトはグローバル状態である。**言明が要るのは深さ 1 -- 値自身の各 boxed leaf -- だけ
      なので、この段はそこまでを述べる。**
  <2>1. `v` の値は、`v` が名指す記号の初期化子の活性化が返し、アクセサが記憶域へ格納した 1 つの値で
        ある。`P` における `v` の読みはその活性化の (E5) の段より後にある。**DEF-0 の (v-3) が
        「その記号の値」と言うのはこの格納された値であり、初期化子がそれを返す前にその値は無い。**
    BY <1>2 (アクセサの枝), DEF-0 の (v-3) (`v` の値はその記号の値であり、`ρ` は `v` を名指す節点を
       `P` までに通っている), <ref id=49da857/> (e) ((e2)),
       <ref id=596a46d/> (「**記号の位置が値を持つのは、その記号のグローバル化の段 (E5) より後の時点である。**」),
       <ref id=243ae2c/> のグローバルのアクセサの行 (「まだならグローバル初期化子の本体を持つ関数 `InitValue#<symbol>`
       を呼び、返った値を記憶域へ格納する」「以後の読みは記憶域を読むだけである」)
  <2>2. その (E5) の段の時点で、`v` の値の inhabited な各 boxed leaf が指すオブジェクトには
        `mark_global` が印を付ける。
    BY <2>1, <ref id=e3436e8/> の (E5) (「返す前に、環境が `mark_global` でその値が到達するオブジェクトのグラフ全体に
       印を付ける」), <ref id=b6673ca/> (「グローバル値が到達するオブジェクトは、記憶域に「グローバル」を表す状態を
       持ち」), DEF 値からの到達 (<ref id=b6673ca/> と (E5) が言う「値が到達するオブジェクト」はこれであり、値の
       inhabited な各 boxed leaf が指すオブジェクトはその第 1 段である), <ref id=0b850c9/>, <ref id=4f63121/>
  <2>3. `P` においてその leaf が指すオブジェクトは、(E5) の段で印が付いたオブジェクトそのものである。
    BY <2>1, <ref id=49da857/> (b) (`v` が値を持った位置以後、`v` の値は同じである), <ref id=66c9670/> (leaf が inhabited である
       かどうかはその値が通る各 unbox union の節のタグで決まるので、値が同じなら (E5) の時点と `P` で
       一致する),
       <ref id=4f63121/> (値が保持する参照の在りかは、その型の boxed leaf のうち inhabited なものである)
  <2>4. QED
    BY <2>2, <2>3, <ref id=88a06de/> (「割り当てられたオブジェクトは計数下であり、グローバル値が到達するグラフに
       `mark_global` が印を付けた時点でグローバル状態になる。逆向きの遷移は無い」) -- 印が付いた時点
       以後そのオブジェクトはグローバル状態であり、`P` における `v` の読みはその時点より後である
       (<2>1)。
<1>5. QED
  BY <1>1, <1>1a, <1>2, <1>3, <1>4, <ref id=eb90864/> -- <1>2 の 2 つの場合のうち funptr の側は <1>3 が空虚にする。
     残る側は <1>4 が与える。グローバル状態のオブジェクトを指す leaf が <ref id=ec8d1a0/> の意味の参照を
     持たないことは <ref id=eb90864/> が与える。

**L15 (鍵の範囲)**: `x` を `B` に現れる `RcVar` の名前とする。`x` は P2 の範囲にある -- すなわち <!--#0376e8d-->
プログラムの束縛変数であるか、`vars.bindings` に束縛を持たない名前 (D6 の第 3 の形) である。よって
どの path `π` についても `origin(x, π)` の呼び出しは panic せずに答えを返して停止し、**その呼び出しの
中で走る `assert!`・`assert_eq!`・`panic!` はどれも発火しない**。

**この文書が `origin` に問う鍵の第 1 成分はどれもこの形である。** 問う相手は `B` の節点が名指す
`RcVar` であり、再帰の辺が進む先の `RcVar` も `B` の節点の欄から来る (L6)。L2 (c)、L9、L16、
L17 がこれを読む。

<1>1. `x` の名前が `vars.bindings` に束縛を持つとき、`x` はプログラムの束縛変数であり、P2 の第 1 の
      場合に当たる。
  BY <ref id=49da857/> (a') (`vars.bindings` に入る名前は、`B` が関数 `f` の `body` であるときの `f` のパラメータと
     capture、および値を与える 3 構文が束縛する変数だけである),
     <ref id=0edb0ba/> (「**「プログラムの束縛変数」は、節点が束縛する変数と、その本体のパラメータ・capture の
     両方である。**」)
<1>2. 束縛を持たないとき、`x` は最上位の記号の名前であり、P2 の第 2 の場合に当たる。
  BY <ref id=596a46d/> (「**束縛を持たない名前は、必ず最上位の記号の名前である。**」),
     <ref id=0edb0ba/> (第 2 の場合として「`vars.bindings` に束縛を持たない名前 (D6 の第 3 の形)」を挙げる)
<1>3. QED
  BY <1>1, <1>2, <ref id=49da857/> (a') (名前は束縛を持つか持たないかのどちらかである),
     <ref id=0edb0ba/> (その 2 種の `x` について、`π` を問わず `origin(x, π)` は panic せずに答えを返し、停止する)
     -- 表明が発火すればその呼び出しは panic して返らないので、<ref id=0edb0ba/> の言う「panic せずに答えを返し、
     停止する」と
     相容れない。

**L16 (鍵の答え)**: `x` を `B` に現れる `RcVar` の名前、`π` を path とし、`K = (x, π)` とする。 <!--#3c6aa4c-->
`origin(K)` の呼び出しはどれも値を返し、その値は 1 つである。これを**鍵 `K` の答え**と呼ぶ。
さらに、`origin_inner(K)` の 1 つの実行が返す値は鍵 `K` の答えである。よって `origin_inner` の 1 つの
実行について立てた言明は、そのまま鍵の答えについての言明として読める。

<1>1. `origin(K)` の呼び出しは panic せずに値を返す。
  BY <ref id=0376e8d/>
<1>2. 2 つの呼び出しが返す値は等しい。
  この文書は 1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定し (第 1 節)、`B` は
  `borrow_ify` の入力プログラムの本体なので A6 と A11 を満たし、`vars` は `VarTable::of` か
  `VarTable::body_only` が作った表である。P2a の言明が置く制限はこれである。
  BY <ref id=b1f6e13/> (「**1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定する。**」「**`vars` は、
     A6 と A11 を満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が作った表で
     ある。**」), <ref id=33c54dc/>, <ref id=3905b4e/>, <1>1,
     CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only
<1>3. `origin_inner(K)` を走らせる式は `origin` の中の 1 つだけであり、memo に `K` が無い状態の
      `origin(K)` の呼び出しはその返り値をそのまま返す。
  BY 前提 `origin_inner` を呼ぶ式の在りか (挙がった項目は定義と `origin` の 2 つであり、
     `origin_inner` を走らせる式は `origin` の中の 1 つである),
     CODE src/rc_ir/ownership.rs: origin (`vars.origins` に `K` が在ればその値の複製を返し、無ければ
     `grow_stack(|| origin_inner(..))` の値を `origins` に入れてから返す),
     <ref id=3e6b0e0/> (`grow_stack` は閉包をちょうど 1 回呼び、その返り値を返す)
<1>4. QED
  BY <1>1, <1>2, <1>3 -- <1>1 と <1>2 より鍵 `K` について返る値は 1 つであり、<1>3 より
     `origin_inner(K)` の 1 つの実行が返す値はその値である。

**L14 (`origin` の再帰呼び出しの鍵の関係は整礎である)**: 鍵の間の再帰の辺 (`DEF 再帰の辺`) について <!--#391d1ce-->
次の 2 つが成り立つ。

- **(a)** `origin` が呼ばれる鍵 `K_0` を取ると、`K_0` から再帰の辺を辿って無限に降りる鍵の列は無い。
  すなわちこの関係は `K_0` から到達する鍵の上で整礎であり、その上の整礎帰納が使える。
- **(b)** `origin(K)` が呼ばれ、再帰の辺 `K -> K'` が在るならば、`origin(K')` も呼ばれる。

(b) を言明に置くのは、L17 と 系 3 が「`origin(x, π)` が呼ばれる」を前提に取り、鎖の次の段の鍵に
ついてその前提を要るからである。

**閉路の不在だけでは足りない。** 鍵の第 2 成分は path であり、E5 と E7 の辺はその先頭に添字を足すので
path は伸びる。鍵の到達集合が有限であることはどこにも述べられていないので、閉路が無いことから無限降下列が
無いことは出ない。この命題は有限性を経由せず、P2 の停止性と memo の規律から無限降下列を直接排除する。

<1>1. `origin(K)` は、`vars.origins` に鍵 `K` が在ればその値の複製を返し、無ければ `origin_inner(K)` を
      走らせ、その返り値を鍵 `K` で `origins` に入れてから返す。`origins` から要素が取り除かれることは
      無い。
  **読み書きの在りかを与えるのは走査である。** `origins` の欄は `pub` を持たないので、それを名指せる
  のは `src/rc_ir/ownership.rs` とその子孫のモジュールだけであり (`EXT 可視性`)、欄アクセスの字面が
  在る項目は第 1 節の前提が挙げる 1 つ -- `origin` -- だけである。欄を持つ値を組み立てるのは
  `VarTable::empty` であり、そこが置くのは空の表である。よって `insert` はこの 1 か所、取り除く操作は
  どこにも無い。
  BY CODE src/rc_ir/ownership.rs: origin (`vars.origins.borrow().get(&key)` が当たれば `known.clone()` を
     返し、そうでなければ `grow_stack(|| origin_inner(..))` の値を
     `vars.origins.borrow_mut().insert(key, answer.clone())` で入れてから返す。この関数が
     `origins` を名指すのはこの 2 行であり、除去を呼ぶ式は持たない),
     前提 `VarTable` の `origins` の欄に触れる式の在りか, EXT 可視性,
     CODE src/rc_ir/ownership.rs: VarTable (`origins` は `pub` の付かない
     `RefCell<Map<VarPath, Origin>>` の欄である),
     CODE src/rc_ir/ownership.rs: VarTable::empty (`origins: RefCell::default()` -- 空の表を置く),
     CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only (どちらも `VarTable::empty` から
     始める),
     <ref id=3e6b0e0/> (`grow_stack` は閉包をちょうど 1 回呼び、その返り値を返す)
<1>1a. 1 つの `VarTable` の `origins` への `insert` は、時間で全順序に並び、その順序を保ったまま
       自然数で番号づけられる。
  `insert` は `origin(vars: &VarTable, ..)` の中の 1 行であり、共有参照 `&VarTable` を通じた動作で
  ある。`VarTable` は `origins: RefCell<Map<VarPath, Origin>>` を欄に持つので `Sync` ではなく
  (EXT auto trait と共有 の 1 と 2)、したがって `&VarTable` は `Send` ではない
  (EXT auto trait と共有 の 3)。よってその表への動作が 2 つのスレッドで重なることはなく、時間で
  全順序に並ぶ (EXT auto trait と共有 の 5)。
  **結論に要るのは全順序であって「1 つの制御の流れ」ではない。** `&VarTable` が `Send` でないことは、
  `VarTable` の**値そのもの**が別のスレッドへ move されることを排除しない。それでも順序は付く --
  値の所有者は各時点で 1 つであり、渡す動作が前後のアクセスを順序づけるからである
  (EXT auto trait と共有 の 4)。
  **番号づけには全順序だけでは足りない。** 全順序の元に自然数の番号を振るには、各元の前に在る元が
  有限個であることが要る。`insert` はコンパイラのプロセスの実行の動作なので、EXT 動作の番号づけが
  それを与える。
  BY <1>1, EXT auto trait と共有 (1 から 5), EXT 動作の番号づけ,
     前提 `unsafe impl` の在りか (EXT auto trait と共有 の 2 の但し書きに当たる型はこのクレートに
     無い),
     CODE src/rc_ir/ownership.rs: VarTable (`origins` は `RefCell<Map<VarPath, Origin>>` の欄である),
     CODE src/rc_ir/ownership.rs: origin (`vars.origins.borrow_mut().insert(..)` はこの関数の中の
     1 行であり、`vars` は共有参照である)
<1>2. どの鍵についても `origin` の呼び出しは panic せずに答えを返し、停止する。
  鍵の第 1 成分 `v` は、`vars.bindings` に束縛を持つか持たないかのどちらかであり、**この 2 つが
  P2 の 2 つの場合である。** 持つとき、L10 (a') より `v` は `B` が関数 `f` の `body` であるときの
  `f` のパラメータか capture、または値を与える 3 構文が束縛する変数であり、P2 の脇が
  「**「プログラムの束縛変数」は、節点が束縛する変数と、その本体のパラメータ・capture の
  両方である。**」と述べるとおり P2 の第 1 の場合に当たる。持たないとき、P2 が第 2 の場合として
  挙げる「`vars.bindings` に束縛を持たない名前 (D6 の第 3 の形)」そのものである。**場合分けが
  `vars.bindings` の membership で尽きるので、この段は鍵の第 1 成分に他の条件を置かない。**
  BY <ref id=0edb0ba/> (範囲は「`x` がプログラムの束縛変数であるか、`vars.bindings` に束縛を持たない名前
     (D6 の第 3 の形) であるようなすべての `(x, π)`」であり、脇が
     「**「プログラムの束縛変数」は、節点が束縛する変数と、その本体のパラメータ・capture の
     両方である。**」と定める。その範囲で `origin` は `π` を問わず panic せずに答えを返し、停止する。
     第 2 の場合について脇は「その場合 `origin_inner` は `None` の腕に入り、即座に `here()` を
     返す。」と述べる),
     <ref id=49da857/> (a') (`vars.bindings` に入る名前は、`B` が関数 `f` の `body` であるときの `f` のパラメータと
     capture、および値を与える 3 構文が束縛する変数だけである)
<1>3. `origin(K)` が呼ばれるならば、その呼び出しが返った時点で `origins` は `K` を含む。`origins` へ
      要素を入れる `insert` は <1>1 の 1 か所だけであり、その呼び出しは時間で全順序に並んで自然数で
      番号づけられる (<1>1a)。`K` を入れる最初の `insert` が何番目かを `t(K)` と書く。**`t(K)` は
      自然数である。**
  BY <1>1, <1>1a, <1>2
<1>4. `origin(K)` が呼ばれ、再帰の辺 `K -> K'` が在るならば、`origin(K')` も呼ばれ、`t(K') < t(K)` で
      ある。
  <2>1. `K` を `origins` に入れた呼び出しが在り、それは `origin_inner(K)` を走らせ、それが返った後に
        `K` を入れている。
    BY <1>1, <1>3 -- `origin(K)` が呼ばれるので `K` は `origins` に入る (<1>3)。`insert` は <1>1 が
       挙げる 1 か所にしかなく、入れるのは `origin_inner` の返り値である。
  <2>2. その `origin_inner(K)` の実行は `origin(K')` を呼び、その呼び出しは `origin_inner(K)` が返る
        前に返る。
    `origin_inner` が行う再帰呼び出しの鍵は `vars`、`type_env`、`var`、`path` だけで決まる。memo は
    `origin` の側に在り、`origin_inner` がどの鍵を呼ぶかを変えない。**`Llvm` の腕が
    `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の返り値から鍵を決めることは、A3 の決定性の
    節が片付ける** -- `result_prov` は `&self` を取るので、同じ引数に同じ値を返すことを言う者が無ければ
    2 回の評価が違う鍵の集合を作りうる。よって `origin_inner(K)` のどの実行も同じ辺を辿る。
    BY <2>1, <ref id=e11772a/> (`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である), DEF 再帰の辺,
       <1>2 (その呼び出しは停止する),
       EXT 呼び出しの入れ子 (`origin_inner(K)` が呼ぶ `origin(K')` は、`origin_inner(K)` が返るより
       前に返る),
       CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `origin(K')` が返った時点で `origins` は `K'` を含む。
    BY <2>2, <1>3
  <2>4. QED
    BY <2>1, <2>2, <2>3, <1>1 (`origins` から要素は取り除かれない), <1>3 -- `t(K)` を与える `insert` を
       行う呼び出しを `c_0` とする。`insert` を行うのは `origin_inner` を走らせた呼び出しだけなので
       (<1>1)、`c_0` は `origin_inner(K)` を走らせ、それが返った後に `K` を入れている。`origin_inner(K)`
       のどの実行も同じ辺を辿る (<2>2) ので、`c_0` の実行も `origin(K')` を呼び、その呼び出しは
       `c_0` の `insert` より前に返る。返った時点で `K'` は `origins` に在る (<2>3) ので、`K'` を入れる
       最初の `insert` は `c_0` の `insert` より前である。すなわち `t(K') < t(K)`。
<1>5. QED
  BY <1>3, <1>4, EXT 整礎性 ((a) 自然数の狭義減少する無限列は無い) -- (b) は <1>4 の前半そのもので
     ある。(a) は次のとおり: 無限に降りる列
     `K_0 -> K_1 -> K_2 -> ...` が在るとすると、`origin(K_0)` は呼ばれるので `t(K_0)` が定まり
     (<1>3)、<1>4 を各辺に順に当てると `origin(K_i)` はどれも呼ばれ、
     `t(K_0) > t(K_1) > t(K_2) > ...` となる。`t` は自然数の値なので、EXT 整礎性 (a) に反する。

## 5. DEF-1 -- D17 の「対応するスロット」を鎖の形に書き直したもの

D17 は「`origin` が `(x, π)` から `(u, σ)` へ辿った別名の辺の列を、`π` の下の leaf `λ` について辿ったときに
着く leaf のスロット」を、`λ` に**対応するスロット**と呼ぶ。辺ごとの `λ` の写り方と、辺の行き先は D17 が
2 つの箇条書きで与えている。行き先の側の 3 行は次のとおりである (`……` は途中打ち切りの印であり、
第 2 と第 3 の箇条はコードを名指す残りを落としている)。

> - `Binding::Join` の辺は、その活性化が選んだアームの結果へ辿る。`origin_inner` はアームを静的に列挙して
>   候補を集めるが (D3)、1 つの活性化では 1 つのアームが選ばれ (D21)、対応するスロットはその結果の側にある。
> - `Binding::Llvm` の leaf の宣言が単一の `Fresh` または単一の `Unknown` であるとき、鎖はそこで止まり、
>   対応するスロットはその位置の `(u, λ)` である。……
> - `origin_from_leaves_under` が辿る辺の行き先の path は、宣言の `σ'` ではなく
>   `truncate_to_unit(ty(args[j]), σ')` である。……

DEF-1 は、この 2 つの箇条書きを、L17 の帰納法が 1 段ずつ辿れる 3 つ組の列として書き直したものである。

**DEF-1 (対応の鎖)**。活性化 `α`、それが辿る実行路 `ρ`、`ρ` 上の位置 `P` を固定する。3 つ組
`(現在の変数, 現在の path, 現在の leaf)` の列を、`(x, π, λ)` から次の規則で作る。各段の「現在の変数」の
`Binding` が、どの規則を使うかを決める。

| 段 | 条件 | 次の 3 つ組 | D17 の行 |
|---|---|---|---|
| E1 | `Move(y)` | `(y, π_cur, λ_cur)` | 写り方の第 1 行 (`λ` を変えない) |
| E2 | `Join(rs)` | `(r_0, π_cur, λ_cur)`。`r_0` は `α` が選んだアームの結果である | 写り方の第 1 行、行き先の第 1 行 |
| E3 | `Llvm` かつ `leaf_origins_at(π_cur)` が単一の `Arg(j, σ)` | `(args[j], σ, σ)` | 写り方の第 3 行 |
| E4a | `Llvm` かつ E3 でなく、`λ_cur` の宣言が単一の `Arg(j, σ')` | `(args[j], t_{ty(args[j])}(σ'), σ')` | 写り方の第 3 行、行き先の第 3 行 |
| E5 | `Field(c, i)` かつ `c` が unbox | `(c, [i] ++ π_cur, [i] ++ λ_cur)` | 写り方の第 2 行 |
| E6 | `Payload(s, None)` | `(s, π_cur, λ_cur)` | 写り方の第 1 行 |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `(s, [t] ++ π_cur, [t] ++ λ_cur)` | 写り方の第 2 行 |

次の 2 つの場合、列はそこで止まる。

| 停 | 条件 | D17 の行 |
|---|---|---|
| S1 | `origin_inner` が `here()` をそのまま返す 5 つの枝 (L9 の H1 から H5) | 辿る辺が無い |
| S2 | `Llvm` かつ E3 でなく、`λ_cur` の宣言が単一の `Fresh` または単一の `Unknown` | 行き先の第 2 行 |

E3、E4a、S2 の 3 つの条件が読む `decl` は
`gen.result_prov(ty(x_cur), arg_tys, type_env)` の返り値である (L8)。**L17 の ASSUME を満たす
3 つ組では、この 9 行のうちちょうど 1 行が当たる。** そのことと、条件が読む `decl` が 1 つに決まる
ことは、L17 の証明が場合分けとして示す。よってその範囲でこの規則は 1 つの鎖を定める。

止まった位置の 3 つ組を `(u, σ_end, μ)` とし、`(u, μ)` を `λ` の**対応する位置**と呼ぶ。L17 の (ii)
が `(u, μ)` は `(x, λ)` と同じオブジェクトを指すことを、(iii) が、`x` の値の leaf `λ` が計数下の
オブジェクト (D26) を指すとき `(u, μ)` は D6 のスロットであることを示す。D17 の「対応するスロット」が
指すのは、その場合のこれである。そうでないとき `(u, μ)` は D6 の**記号の位置**でありうる。

**この鎖と D33 の `ρ` 歩みの関係は 系 3 (第 6 節) が述べる。** L6 の第 2 の表だけでは足りない。その表が
言うのは、各段が D9 の移動の表のどの行の下にあるかまでであり、D20 は**辺の存在**に条件を課すからで
ある -- 「その辺を定める節点が実行路の上に在り、かつその節点と leaf `λ` が作る 2 つの対がどちらもその
路の位置であるとき」であり、アームの中の行についてはさらに「路がそのアームを選ぶこと」が要る。その
条件を各段について与えるのはL17 の (iv) であり、系 3 はそれを鎖の全体へ延ばす。

E4a の行き先の path が `σ'` ではなく `t_{ty(args[j])}(σ')` であることは、コードでは
`operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)))` である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。leaf が行き先の path の下に留まること
(`σ' ⊒ t_{ty(args[j])}(σ')`) は、L17 の CASE E4a が示す。

## 6. 主命題、および P3 と P4 <!--#b5c57dc-->

**L17**。ASSUME: <!--#95f1cbf-->

- NEW `α`: 活性化、NEW `ρ`: `α` が辿る実行路、NEW `P`: `ρ` 上の位置、
- NEW `x`: `P` で値を持つ (DEF-0) `RcVar`、NEW `π`: path であって、この `vars` と `type_env` について
  `origin(x, π)` が呼ばれるもの、
- NEW `λ`: `ty(x)` の boxed leaf であって `λ ⊒ π` かつ `P` で inhabited (D16) であるもの

PROVE: DEF-1 の鎖は有限で止まり、その停止点 `(u, σ_end, μ)` は次を満たす。

- (i) `(u, σ_end) ∈ cand(x, π)`。
- (ii) `u` は `P` で値を持ち (DEF-0)、`μ ⊒ σ_end` であり、`μ` は `ty(u)` の boxed leaf であって `P` で
  inhabited であり、**`u` の値の leaf `μ` は `x` の値の leaf `λ` と同じオブジェクトを指す**。
- (iii) さらに **(H)** -- `P` における `x` の値の leaf `λ` が指すオブジェクトは計数下 (D26) である --
  を仮定すると、`u` は DEF-0 の (v-1) か (v-2) であり、`(x, λ)` と `(u, μ)` はどちらも `P` のスロット
  (D6) であって、それぞれちょうど 1 つの D8 の参照を持ち、その 2 つは同一である。
- (iv) 鎖が停止条件 (S1、S2) で始まらないとき、第 1 段が進む先を `(v, π', λ')` とすると、`v` は `P` で
  値を持ち (DEF-0)、`λ'` は `ty(v)` の `P` で inhabited な boxed leaf であって `λ' ⊒ π'` であり、
  `origin(v, π')` は呼ばれる。よって `(x, λ)` と `(v, λ')` はどちらも `P` の位置 (D6) であり、
  **この第 1 段は D20 の別名の辺である**。

**(iv) が第 1 段だけを述べるのは、鎖の全体へ延ばすのが 系 3 の仕事だからである。** (iv) は次の段の鍵に
ついて L17 の ASSUME をそのまま与えるので、系 3 はそれを鎖の上の整礎帰納で繋ぐ。

**(H) を読むのは (iii) だけである。** D26 がグローバル状態のオブジェクトを D8 の勘定の外に置くので、
(H) が無いと参照について言えることは無い。L12 より (H) は「`(x, λ)` が D8 の意味の参照を持つ」と同値で
あり、README の P3 と P4 が「**スロット `(x, λ)` が D8 の意味の参照を持つとき**」と書いているのと同じ
条件である。グローバル値 `g` を `Let(x, Var(g), k)` で受ける本体では、鎖は `vars.bindings` が束縛を
持たない名前 `g` で止まり (L9 の H1)、`g` は DEF-0 の (v-3) なので `(g, λ)` は D6 のスロットではなく
**記号の位置**であって、両端の leaf はどちらも D8 の意味の参照を持たない (L13)。README の P3 が
条件の要る理由としてこの形を挙げている。

**(ii) のオブジェクトの同一は (H) を読まない。** 各段でそれを与えるのは D9 の移動の表の**値の水準の
6 行**であり、その 6 行はどれも参照を持ち出さない。README の P3 が「**条件を外した形では、2 つの位置 (D6)
は同じオブジェクトを指す。**」、P4 が「**条件を外した形では、その位置 (D6) と対応する位置のいずれかは同じ
オブジェクトを指す。**」と述べるのがこの節である。P3 はその根拠を 2 つに分け、参照を持つ場合は D8 の
参照の同一から、両端がグローバル状態を指す場合は D9 の値の水準の行から出すと書く。**L17 は 2 つを分けず、
どちらの場合も D9 の値の水準の行から出す。**主語がスロットでなく位置であるのは、鎖が記号の位置で
止まりうるからである (P3 の同じ節)。

証明は、`origin` が `(x, π)` から行う再帰呼び出しの関係の上の整礎帰納による。この関係が整礎であることは
L14 (a) が与える。**`π` に「`origin(x, π)` が呼ばれる」を課すのは L14 (a) のためである** -- L14 (a) は
無条件ではなく、「`origin` が呼ばれる鍵 `K_0` を取る」ことを前提に、`K_0` から到達する鍵の上での
整礎性を言う。
その証明が使う `t(K)` は `origins` への実際の `insert` の順番なので、呼ばれていない鍵には定まらない。
**この条件は L17 を使う側が果たす** -- 系 1 と系 2 はどちらも `origin(x, π)` の値を主語に取るので、その
呼び出しが在る。DEF-1 の各段は `origin_inner` の再帰呼び出しの 1 つに一致する (L6) ので、鎖の各段で
帰納法の仮定が使える。

**各 CASE の第 1 の段は P2a を経る。** `origin_inner` の腕が返すのは再帰呼び出しの返り値であり、それを
`origin(v, π')` という**鍵の答え** (L16) の等式として読むには、答えが鍵ごとに 1 つに決まり、memo の
状態に依らないことが要る。`origin` は memo が当たると `origin_inner` を走らせないので、P2a がその一段を
与える。

**各段の形は共通である。** 段が `(x, π, λ)` から `(v, π', λ')` へ進むとき、次の 6 つをこの順に置く。

1. `v` は `P` で値を持つ (L11)。
2. `x` の値の leaf `λ` と `v` の値の leaf `λ'` は、同じオブジェクトを指す。**どの段でもこれを与えるのは
   D9 の移動の表の値の水準の行である** -- 2 つの leaf の値が等しければ、2 つは同じオブジェクトを指す。
   `Llvm` の 2 つの段 (E3、E4a) もその表の「`Llvm` の素通し leaf」の行を読む。この一歩は (H) を読まない。
   **D20 の節はこの一歩を両端がスロットである場合に限った形である** -- D20 の
   「**別名の辺の両端のスロットは、同じオブジェクトを指す。**」がそれであり、鎖は記号の位置で
   終わりうるので、L17 が読むのはスロットに限らない D9 の行の側である。
3. (H) を仮定すると、2 と D26 より `v` の値の leaf `λ'` が指すオブジェクトも計数下である。L13 の対偶より
   `v` は DEF-0 の (v-3) ではなく、(v-1) か (v-2) である。すなわち `(v, λ')` は `P` のスロットである
   (D6)。
4. (H) の下で、L12 より `(x, λ)` と `(v, λ')` はそれぞれちょうど 1 つの参照を持ち、D9 の移動の表の
   参照の水準の行よりその 2 つは同一である。`Llvm` の 2 つの段では、その行が述べる素通しを A3 の
   「単一の `Arg(j, σ)`」の行が生成コードの水準で言い直している。
5. その段が D20 の別名の辺であることを述べる ((iv))。要るのは 3 つである -- 両端 `(x, λ)` と
   `(v, λ')` がどちらも `P` の位置であること (1 と、その CASE の型についての段)、その辺を定める節点が
   `ρ` の上に在ること (L11 の後半)、そしてアームの中の行 (E2、E6、E7) については `α` がそのアームを
   選んでいることである。この一歩も (H) を読まない。
6. 帰納法の仮定を `(v, π')` に適用する。その前提 -- `v` が `P` で値を持つこと (1) と、`λ'` が
   `ty(v)` の `P` で inhabited な boxed leaf であること (各 CASE の型についての段) -- は (H) を
   読まない。(i) を `cand(x, π)` へ読み替え、(ii) のオブジェクトの同一を 2 と繋ぐ。(H) の下では 3 が
   帰納法の仮定の (iii) の前提を与えるので、(iii) を 4 と繋ぐ。

**この段落は読みの見取り図である。**以下の各 CASE は、この 6 つを、その段が読む D9 の行と A3 の行を
名指して書き下す。

<1>0. (H) を仮定すると、`x` は DEF-0 の (v-1) か (v-2) であり、`(x, λ)` は `P` のスロット (D6) であって、
      ちょうど 1 つの D8 の参照を持つ。
  BY (H), <ref id=596d4c9/> (対偶 -- (v-3) の名前の値の inhabited な leaf はグローバル状態のオブジェクトを指す),
     <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、
     `λ` は `ty(x)` の `P` で inhabited な boxed leaf である), <ref id=596a46d/>, <ref id=eb90864/>

<1>1. CASE: 停止条件 S1 (`origin_inner` が `here()` を答える)。
  <2>1. `origin(x, π) = Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `None | Some(Binding::Param) | Some(Binding::Producer)`
       の腕、`Some(Binding::Field(..))` の `container.ty.is_box` の枝、`Some(Binding::Payload(..))` の
       `Some(_)` の枝, <ref id=0212823/>, <ref id=e05fb56/> (b), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す),
       <ref id=95f1cbf/> の ASSUME と DEF-0 (`x` は `P` で値を持つ `RcVar` なので `B` に現れる。これが <ref id=0212823/> と
       <ref id=3c6aa4c/> の前提である)
  <2>2. 停止点は `(x, π, λ)` である。
    BY DEF-1 の S1
  <2>3. QED
    (i) は `(x, π) ∈ {(x, π)}`。(ii) は ASSUME の `λ ⊒ π`、`λ` が `ty(x)` の `P` で inhabited な
    boxed leaf であること、`x` が `P` で値を持つこと、および停止点が `(x, λ)` 自身なのでオブジェクトの
    同一が等号であること。(iii) は (H) の下で `(u, μ) = (x, λ)` すなわち同じスロットどうしであり、
    `<1>0` がそれがちょうど 1 つの参照を持つことを与える。(iv) は、鎖が停止条件で始まるので空虚で
    ある。
    BY <2>1, <2>2, <1>0, ASSUME

<1>2. CASE: 停止条件 S2 (`Llvm` で `λ` の宣言が単一の `Fresh` または単一の `Unknown`)。
  <2>1. `Exactly((x, π))` は `reached` の要素である。
    BY <ref id=9a6b1cd/> (d1)
  <2>2. `(x, π) ∈ cand(x, π)`。
    <3>1. `reached` の全要素が等しいとき、鍵 `(x, π)` の答えは `Exactly((x, π))` であり
          `cand(x, π) = {(x, π)}`。
      BY <2>1, <ref id=d2c1f1f/> (b), <ref id=e05fb56/> (b), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. そうでないとき、鍵 `(x, π)` の答えは `of_candidates(C, (x, π))` であり
          `C ⊇ act(Exactly((x, π))) = {(x, π)}` である。`of_candidates` の `candidates()` は `C` そのもの
          である。
      BY <2>1, <ref id=d2c1f1f/> (b), <ref id=3de9373/>, <ref id=e05fb56/> (b), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. QED
    停止点は `(x, π, λ)` なので、(i) は `<2>2`、(ii) は ASSUME の `λ ⊒ π`、`λ` が `ty(x)` の `P` で
    inhabited な boxed leaf であること、`x` が `P` で値を持つこと、および停止点が `(x, λ)` 自身なので
    オブジェクトの同一が等号であることから出る。(iii) は (H) の下で同じスロットどうしであり、
    `<1>0` がそれがちょうど 1 つの参照を持つことを与える。(iv) は、鎖が停止条件で始まるので空虚で
    ある。
    BY <2>2, <1>0, DEF-1 の S2, ASSUME

<1>3. CASE: 段 E1 (`Move(y)`)。
  <2>1. `origin(x, π) = origin(y, π)` であり `cand(x, π) = cand(y, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕 (この腕は
       `origin(vars, type_env, &y.name, path)` の返り値をそのまま返す),
       <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `y` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E1 (`x` の束縛は `Move(y)` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>2. `P` における `x` の値は `y` の値であり、`ty(y) = ty(x)` であり、`λ` は `ty(y)` の boxed leaf で
        `P` で inhabited である。
    BY <ref id=9d74736/> の移動の表の値の水準の行 (「`Let(x, Var(y), k)`: `x` の値は `y` の値である」),
       <ref id=83d98e9/> (move-bind の両辺の型が一致する), <ref id=49da857/> (b) (`x` の値も `y` の値も、値を持った位置の後は
       変わらない), <2>1a, <ref id=66c9670/> -- `x` と `y` の値は同じなので、`λ` が通る各 unbox union の節のタグも
       同じである。
  <2>2a. `y` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>2, <2>1a, <ref id=9d74736/> の移動の表の値の水準の行 (「`Let(x, Var(y), k)`: `x` の値は `y` の値である」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `y` は DEF-0 の (v-1) か (v-2) で
         あり、`(y, λ)` は `P` のスロットである。
    BY <2>2a, <2>2, <2>1a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>3. (H) の下で、`(x, λ)` と `(y, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の `Let(x, Var(y), k)` の行 (`y` の参照が `x` へ), <ref id=ec8d1a0/>
  <2>3a. `origin(y, π)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と `(y, λ)` は
         どちらも `P` の位置 (D6) である。
    BY <2>1a, <2>2, <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited な
       boxed leaf であり、`origin(x, π)` は呼ばれる), <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed
       leaf の対である), <ref id=9357e31/> (E1 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b) (`origin(K)` が呼ばれ
       再帰の辺 `K -> K'` が在れば `origin(K')` も呼ばれる),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Var(y), k)` を `P` までに通る),
       <ref id=9c7c27a/> (「`Let(x, Var(y), k)` の `y` から `x` へ」の辺。「辺が在るのは、その辺を定める節点が
       実行路の上に在り、かつその節点と leaf `λ` が作る 2 つの対がどちらもその路の位置であるときで
       あり、そのときに限る」。この行はアームの中の行ではないので、アームの選択は要らない),
       DEF-1 の段 E1
  <2>4. 帰納法の仮定を `(y, π)` に適用する。前提は <2>1a (`y` は `P` で値を持つ)、<2>2 (`λ` は `ty(y)` の
        `P` で inhabited な boxed leaf であり `λ ⊒ π`)、<2>3a (`origin(y, π)` は呼ばれる) が与える。
        結論は (i) `(u, σ_end) ∈ cand(y, π)`、
        (ii) `(u, μ)` が `(y, λ)` と同じオブジェクトを指すこと、(iii) (H) の下で `(y, λ)` の参照と
        同一であること、である。(iii) の前提となる `(y, λ)` についての (H) は <2>2b が与える。
    BY <2>1a, <2>2, <2>2b, <2>3a, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>2a, <2>3, <2>3a, <2>4, <1>0 -- (i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は
       <2>4 の (ii) と
       <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と <2>4 の (iii) の推移であり、`(x, λ)` が
       スロットであることは <1>0 が与える。(iv) は <2>3a である。

<1>4. CASE: 段 E6 (`Payload(s, None)`、catch-all)。
  <2>1. `origin(x, π) = origin(s, π)` であり `cand(x, π) = cand(s, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の `None =>` の枝
       (この枝は `origin(vars, type_env, &scrut.name, path)` の返り値をそのまま返す),
       <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `s` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E6 (`x` の束縛は `Payload(s, None)` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>2. `P` における `x` の値は `s` の値であり、`ty(s) = ty(x)` であり、`λ` は `ty(s)` の boxed leaf で
        `P` で inhabited である。
    BY <ref id=9d74736/> の移動の表の値の水準の行 (「catch-all アームの payload 束縛: payload 変数の値は
       scrutinee の値そのものである。」), <ref id=83d98e9/> (catch-all アームの payload と scrutinee の型が一致する),
       <ref id=49da857/> (b) (`x` の値も `s` の値も、値を持った位置の後は変わらない), <2>1a, <ref id=66c9670/> -- 2 つの値は同じ
       なので、`λ` が通る各 unbox union の節のタグも同じである。
  <2>2a. `s` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>2, <2>1a, <ref id=9d74736/> の移動の表の値の水準の行 (「catch-all アームの payload 束縛: payload 変数の値は
       scrutinee の値そのものである。」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `s` は DEF-0 の (v-1) か (v-2) で
         あり、`(s, λ)` は `P` のスロットである。
    BY <2>2a, <2>2, <2>1a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>3. (H) の下で、`(x, λ)` と `(s, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の catch-all アームの payload 束縛の行 (scrutinee の参照が payload
       変数へ), <ref id=9c7c27a/> (catch-all アームの scrutinee から payload 変数への別名の辺), <ref id=ec8d1a0/>
  <2>3a. `origin(s, π)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と `(s, λ)` は
         どちらも `P` の位置 (D6) である。
    <3>1. `α` は `x` を payload とする catch-all アーム `A` を選んでおり、`ρ` は `A` の `body` の根の
          節点を `P` までに通っている。
      BY <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持つ),
         DEF-0 の (v-1) (`x` は `Binding::Payload` を持つので (v-3) でも (v-2) でもない。よって `ρ` は
         `x` の授与位置を `P` までに通る。(v-1) の表の第 4 行より、その授与位置は `A` の `body` の根の
         節点である), <ref id=49da857/> (c2) (`ρ` が `A` の `body` の根の節点を通るのは `α` が `A` を選んだときに
         限る), <ref id=49da857/> (a), <ref id=49da857/> (c1), <ref id=33c54dc/>,
         CODE src/rc_ir/ownership.rs: collect_bindings -- `x` に `Binding::Payload(s, None)` を
         与えるのは `tag` が `None` のアームの payload 束縛だけである
    <3>2. QED
      BY <3>1, <2>1a, <2>2, <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で
         inhabited な boxed leaf であり、`origin(x, π)` は呼ばれる),
         <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
         <ref id=9357e31/> (E6 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
         <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Match(s, arms), k)` を `P` までに通る),
         <ref id=9c7c27a/> (「catch-all アームの scrutinee から payload 変数へ」の辺。両端が路の位置であることに
         加え、アームの中の行なので「路がそのアームを選ぶこと」も要り、<3>1 がそれを与える),
         DEF-1 の段 E6
  <2>4. 帰納法の仮定を `(s, π)` に適用する。前提は <2>1a、<2>2、<2>3a (`origin(s, π)` は呼ばれる) が
        与える。結論は
        (i) `(u, σ_end) ∈ cand(s, π)`、(ii) `(u, μ)` が `(s, λ)` と同じオブジェクトを指すこと、
        (iii) (H) の下で `(s, λ)` の参照と同一であること、である。(iii) の前提となる `(s, λ)` に
        ついての (H) は <2>2b が与える。
    BY <2>1a, <2>2, <2>2b, <2>3a, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>2a, <2>3, <2>3a, <2>4, <1>0 -- (i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は
       <2>4 の (ii) と
       <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と <2>4 の (iii) の推移であり、`(x, λ)` が
       スロットであることは <1>0 が与える。(iv) は <2>3a である。

<1>5. CASE: 段 E5 (`Field(c, i)`、`c` が unbox)。
  <2>1. `origin(x, π) = origin(c, [i] ++ π)` であり `cand(x, π) = cand(c, [i] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(..))` の `else` の枝 (この枝は
       `origin(vars, type_env, &container.name, &container_path)` の返り値をそのまま返す),
       <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `c` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E5 (`x` の束縛は `Field(c, i)` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>2. `[i] ++ λ` は `ty(c)` の boxed leaf であり、`[i] ++ λ ⊒ [i] ++ π` である。
    **`boxed_leaf_paths` の走査が `unpunched_field_types` のループへ降りるのは、`is_fully_unboxed`、
    `is_closure`、`is_box`、`is_array` の 4 つがいずれも偽のときである。** `is_box` は DEF-1 の段 E5 の
    条件 (`c` が unbox) が偽にする。`is_closure` を偽にするのは A12 の「**この仮定が型の `variant` を
    述べる各節では、その型の `is_closure()` は偽である。**」の節であり、「`Destructure` の容器が構造体
    である」がその節の 1 つである。同じ節が `is_array` と `is_funptr` も偽にする。`is_array()` が真に
    なるのは最上位の tycon が `Std::Array` であるときに限り、`is_funptr()` が真になるのは最上位の
    tycon がいずれかの `Std::#FunPtr{n}` であるときに限る。型の `TyConInfo` はその最上位の tycon で
    `type_env.tycons()` を引いた 1 つであり、`Std::Array` と `Std::#FunPtr{n}` はどちらも
    `bulitin_tycons()` が置く鍵なので、A28 よりその鍵の下の項目は `bulitin_tycons()` が置いた項目で
    ある -- 前者の `variant` は `Array`、後者のそれは `Primitive` である。ところが A12 のこの節より
    `ty(c)` の `variant` は `Struct` なので、`ty(c)` の最上位の tycon は `Std::Array` でも
    いずれかの `Std::#FunPtr{n}` でもない。
    残る `is_fully_unboxed` は、この 4 つが偽なので unpunched な各フィールドの型が
    すべて fully unboxed であることに帰着するが、フィールド `i` の型 `ty(x)` は boxed leaf `λ` を
    持つので fully unboxed ではない -- fully unboxed な型に `boxed_leaf_paths` は leaf を返さない。
    BY <ref id=83d98e9/> (`Destructure` のフィールド変数とフィールドの型が合っていること、容器が構造体であること、
       **`Destructure` が名指すフィールドがその型が実際に持つ (punched でない) ものであること**、
       および「**この仮定が型の `variant` を述べる各節では、その型の `is_closure()` は偽である。**」),
       <ref id=95f1cbf/> の ASSUME (`λ` は `ty(x)` の boxed leaf である), DEF-1 の段 E5 (`c` は unbox である),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- 走査は `is_fully_unboxed`、`is_closure`、
       `is_box`、`is_array` の 4 つを順に見て `return` し、どれも偽のときだけ
       `unpunched_field_types` のループへ降りる。降りた枝は各フィールドへ添字を積むので、punched で
       ないフィールド `i` の leaf は `[i] ++ (そのフィールドの型の leaf)` である,
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed (`is_box`・`is_closure`・`is_array` に偽を
       返した後、`is_funptr` なら真、そうでなければ unpunched な各フィールドの型についての `all`),
       CODE src/ast/types.rs: TypeNode::is_struct (`toplevel_tycon_info` の `variant` が `Struct` か),
       <ref id=3d4be43/> (`E.tycons()` の項目のうち鍵が `bulitin_tycons()` の置く鍵のいずれかであるものは、
       `bulitin_tycons()` がその鍵の下に置いた項目である。とくに `make_array_tycon()` の項目と、
       `tc.name.namespace` が `Std` の 1 段であって `tc.name.name` が `FUNPTR_NAME` で始まる鍵の
       項目がそうである),
       CODE src/fixstd/builtin.rs: bulitin_tycons (`make_array_tycon()` の項目の `variant` は
       `TyConVariant::Array`、`make_funptr_tycon(arity)` の項目のそれは `TyConVariant::Primitive`
       である),
       CODE src/ast/types.rs: TypeNode::is_array, TypeNode::is_funptr,
       TypeNode::toplevel_tycon_satisfies (`is_array` は最上位の tycon が `is_array_tycon` を満たすか
       どうか、`is_funptr` は `is_funptr_tycon` がその tycon に `Some` を返すかどうかであり、
       最上位の tycon を持たない型についてはどちらも偽である),
       CODE src/fixstd/builtin.rs: is_array_tycon, is_funptr_tycon (`is_array_tycon` は tycon が
       `make_array_tycon()` に等しいことであり、`is_funptr_tycon` が `Some` を返すのは、名前空間が
       `Std` で名前が `#FunPtr` に続く 10 進の数であるときである),
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info (型の `TyConInfo` は、その最上位の
       tycon で `type_env.tycons()` を引いた 1 つである),
       CODE src/ast/types.rs: TypeNode::unpunched_field_types (punched なフィールドを落とす)
  <2>3. `P` における `x` の値は `c` の値の第 `i` フィールドであり、`[i] ++ λ` は `P` で inhabited で
        ある。
    BY <2>2, <2>1a, <ref id=66c9670/>, <ref id=9d74736/> の移動の表の値の水準の行 (「unbox 容器の `Destructure` の名前付き
       フィールド: フィールド変数の値は容器の値のそのフィールドである。」), <ref id=49da857/> (b) (`x` の値も `c` の
       値も、値を持った位置の後は変わらない) -- `[i]` は unbox 構造体のフィールド添字なので unbox union
       の節を通らず、`[i] ++ λ` が通る union の節は `λ` が通る節と同じである。
  <2>3a. `c` の値の leaf `[i] ++ λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY <2>3, <2>1a, <ref id=9d74736/> の移動の表の値の水準の行 (「unbox 容器の `Destructure` の名前付きフィールド:
       フィールド変数の値は容器の値のそのフィールドである。」)
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `c` は DEF-0 の (v-1) か (v-2) で
         あり、`(c, [i] ++ λ)` は `P` のスロットである。
    BY <2>3a, <2>3, <2>1a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>4. (H) の下で、`(x, λ)` と `(c, [i] ++ λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の unbox 容器の `Destructure` の名前付きフィールドの行 (`c` のその
       フィールドの参照がフィールド変数へ), <ref id=ec8d1a0/>
  <2>4a. `origin(c, [i] ++ π)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と
         `(c, [i] ++ λ)` はどちらも `P` の位置 (D6) である。
    BY <2>1a, <2>2, <2>3, <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited
       な boxed leaf であり、`origin(x, π)` は呼ばれる),
       <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
       <ref id=9357e31/> (E5 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Destructure(c, fs, s, k)` を `P` までに通る),
       <ref id=9c7c27a/> (「unbox 容器の `Destructure` の名前付きフィールドの容器からフィールド変数へ」の辺。
       この行はアームの中の行ではないので、アームの選択は要らない), DEF-1 の段 E5
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>4a, <2>1a, <1>0, 帰納法の仮定を `(c, [i] ++ π)` に適用 --
       帰納法の仮定の前提は <2>1a、<2>2、<2>3、<2>4a が与え、その (iii) の前提となる `(c, [i] ++ λ)` に
       ついての (H) は <2>3b が与える。(i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は帰納法の
       仮定の (ii) と <2>3a を繋いだものである。(iii) は (H) の下で <2>4 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。(iv) は <2>4a である。

<1>6. CASE: 段 E7 (`Payload(s, Some(t))`、`s` が unbox)。
  <2>1. `origin(x, π) = origin(s, [t] ++ π)` であり `cand(x, π) = cand(s, [t] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の
       `Some(tag) if !scrut.ty.is_box(type_env)` の枝 (この枝は
       `origin(vars, type_env, &scrut.name, &scrut_path)` の返り値をそのまま返す),
       <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `s` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E7 (`x` の束縛は `Payload(s, Some(t))` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf であり、`[t] ++ λ ⊒ [t] ++ π` である。
    **`boxed_leaf_paths` の走査が `unpunched_field_types` のループへ降りるのは、`is_fully_unboxed`、
    `is_closure`、`is_box`、`is_array` の 4 つがいずれも偽のときである。** `is_box` は DEF-1 の段 E7 の
    条件 (`s` が unbox) が偽にする。`is_closure` を偽にするのは A12 の「**この仮定が型の `variant` を
    述べる各節では、その型の `is_closure()` は偽である。**」の節であり、「`Match` の scrutinee が
    union である」がその節の 1 つである。同じ節が `is_array` と `is_funptr` も偽にする。`is_array()` が
    真になるのは最上位の tycon が `Std::Array` であるときに限り、`is_funptr()` が真になるのは最上位の
    tycon がいずれかの `Std::#FunPtr{n}` であるときに限る。型の `TyConInfo` はその最上位の tycon で
    `type_env.tycons()` を引いた 1 つであり、`Std::Array` と `Std::#FunPtr{n}` はどちらも
    `bulitin_tycons()` が置く鍵なので、A28 よりその鍵の下の項目は `bulitin_tycons()` が置いた項目で
    ある -- 前者の `variant` は `Array`、後者のそれは `Primitive` である。ところが A12 のこの節より
    `ty(s)` の `variant` は `Union` なので、`ty(s)` の最上位の tycon は `Std::Array` でも
    いずれかの `Std::#FunPtr{n}` でもない。
    残る `is_fully_unboxed` は、この 4 つが偽なので unpunched な
    各変位の payload の型がすべて fully unboxed であることに帰着するが、変位 `t` の payload の型
    `ty(x)` は boxed leaf `λ` を持つので fully unboxed ではない。
    BY <ref id=83d98e9/> (payload と変位の型が合っていること、scrutinee が union であること、**`Match` が名指す変位が
       その型が実際に持つ (punched でない) ものであること**、および「**この仮定が型の `variant` を
       述べる各節では、その型の `is_closure()` は偽である。**」),
       <ref id=95f1cbf/> の ASSUME (`λ` は `ty(x)` の boxed leaf である), DEF-1 の段 E7 (`s` は unbox である),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- 走査は `is_fully_unboxed`、`is_closure`、
       `is_box`、`is_array` の 4 つを順に見て `return` し、どれも偽のときだけ
       `unpunched_field_types` のループへ降りる,
       CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/ast/types.rs: TypeNode::is_union (`toplevel_tycon_info` の `variant` が `Union` か),
       <ref id=3d4be43/> (`E.tycons()` の項目のうち鍵が `bulitin_tycons()` の置く鍵のいずれかであるものは、
       `bulitin_tycons()` がその鍵の下に置いた項目である。とくに `make_array_tycon()` の項目と、
       `tc.name.namespace` が `Std` の 1 段であって `tc.name.name` が `FUNPTR_NAME` で始まる鍵の
       項目がそうである),
       CODE src/fixstd/builtin.rs: bulitin_tycons (`make_array_tycon()` の項目の `variant` は
       `TyConVariant::Array`、`make_funptr_tycon(arity)` の項目のそれは `TyConVariant::Primitive`
       である),
       CODE src/ast/types.rs: TypeNode::is_array, TypeNode::is_funptr,
       TypeNode::toplevel_tycon_satisfies (`is_array` は最上位の tycon が `is_array_tycon` を満たすか
       どうか、`is_funptr` は `is_funptr_tycon` がその tycon に `Some` を返すかどうかであり、
       最上位の tycon を持たない型についてはどちらも偽である),
       CODE src/fixstd/builtin.rs: is_array_tycon, is_funptr_tycon (`is_array_tycon` は tycon が
       `make_array_tycon()` に等しいことであり、`is_funptr_tycon` が `Some` を返すのは、名前空間が
       `Std` で名前が `#FunPtr` に続く 10 進の数であるときである),
       CODE src/ast/types.rs: TypeNode::toplevel_tycon_info (型の `TyConInfo` は、その最上位の
       tycon で `type_env.tycons()` を引いた 1 つである),
       CODE src/ast/types.rs: TypeNode::unpunched_field_types -- union の `unpunched_field_types` は
       punched でない各変位の payload の型を返すので、変位 `t` の leaf は
       `[t] ++ (その payload の型の leaf)` である
  <2>3. `x` は `tag = Some(t)` のアーム `A` の `payload` であり、`α` は `A` を選んでいる。また
        `P` において `s` のタグは `t` である。
    <3>1. `x` は `tag = Some(t)` のアーム `A` の `payload` である。`α` は `A` を選んでおり、`ρ` は
          `A` の `body` の根の節点を `P` までに通っている。
      BY <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ), DEF-0 の (v-1) (`x` は `Binding` を持つので (v-3) では
         なく、`Binding::Payload` は `Binding::Param` ではないので (v-2) でもない。よって `ρ` は `x` の
         授与位置を `P` までに通る。(v-1) の表の第 4 行より、その授与位置は `A` の `body` の根の節点で
         ある), <ref id=49da857/> (c2) (`ρ` が `A` の `body` の根の節点を通るのは `α` が `A` を選んだときに限る),
         <ref id=49da857/> (a), <ref id=49da857/> (c1), <ref id=33c54dc/>,
         CODE src/rc_ir/ownership.rs: collect_bindings -- `x` に `Binding::Payload(s, Some(t))` を
         与えるのは `tag = Some(t)` のアームの payload 束縛だけである
    <3>2. `α` が `A` に入った時点で、`s` の値の実行時のタグは `t` である。
      BY <3>1, <ref id=f769887/> (「**活性化が `tag = Some(t)` のアームに入るのは、`s` の実行時のタグが `t` である
         ときに限る。**」)
    <3>3. `s` の値は、`s` が値を得た後の `ρ` 上のすべての位置で同じである。
      BY <ref id=49da857/> (b)
    <3>4. QED
      BY <3>1, <3>2, <3>3, <2>1a -- 前半は <3>1 である。後半は次の 3 つから出る。`P` はアームに
         入った時点以後にある。その時点で `s` のタグは `t` である。その間 `s` の値は変わらない。
  <2>4. `P` における `x` の値は `s` の値の変位 `t` の payload であり、`[t] ++ λ` は `P` で inhabited で
        ある。
    BY <2>2, <2>3, <2>1a, <ref id=66c9670/>, <ref id=9c7c27a/> (unbox union の変位アームの scrutinee から payload 変数への別名の辺),
       <ref id=9d74736/> の移動の表の値の水準の行 (「unbox union の変位アームの payload 束縛: payload 変数の値は
       scrutinee の値の活性変位の payload である。」), <ref id=49da857/> (b) (`x` の値も `s` の値も、値を持った位置の
       後は変わらない) -- `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (タグ `t` で <2>3 に
       より一致する) と、`λ` が通る節である。後者が一致するのは、`x` の値が `s` の値の変位 `t` の
       payload だからである。
  <2>4a. `s` の値の leaf `[t] ++ λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY <2>4, <2>3, <2>1a, <ref id=9d74736/> の移動の表の値の水準の行 (「unbox union の変位アームの payload 束縛:
       payload 変数の値は scrutinee の値の活性変位の payload である。」-- <2>3 よりその活性変位は `t`
       である)
  <2>4b. (H) を仮定すると、そのオブジェクトは計数下である。よって `s` は DEF-0 の (v-1) か (v-2) で
         あり、`(s, [t] ++ λ)` は `P` のスロットである。
    BY <2>4a, <2>4, <2>1a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>5. (H) の下で、`(x, λ)` と `(s, [t] ++ λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>4b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の unbox union の変位アームの payload 束縛の行 (scrutinee の活性変位の
       参照が payload 変数へ), <ref id=ec8d1a0/>, <2>3 (この行が名指す活性変位が `t` であること)
  <2>5a. `origin(s, [t] ++ π)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と
         `(s, [t] ++ λ)` はどちらも `P` の位置 (D6) である。
    BY <2>1a, <2>2, <2>3 (`α` は `tag = Some(t)` のアーム `A` を選んでいる), <2>4,
       <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited な boxed leaf で
       あり、`origin(x, π)` は呼ばれる),
       <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
       <ref id=9357e31/> (E7 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Match(s, arms), k)` を `P` までに通る),
       <ref id=9c7c27a/> (「unbox union の変位アームの scrutinee から payload 変数へ」の辺。両端が路の位置である
       ことに加え、アームの中の行なので「路がそのアームを選ぶこと」も要り、<2>3 がそれを与える),
       DEF-1 の段 E7
  <2>6. QED
    BY <2>1, <2>2, <2>4, <2>4a, <2>4b, <2>5, <2>5a, <2>1a, <1>0, 帰納法の仮定を `(s, [t] ++ π)` に適用 --
       帰納法の仮定の前提は <2>1a、<2>2、<2>4、<2>5a が与え、その (iii) の前提となる `(s, [t] ++ λ)` に
       ついての (H) は <2>4b が与える。(i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は帰納法の
       仮定の (ii) と <2>4a を繋いだものである。(iii) は (H) の下で <2>5 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。(iv) は <2>5a である。

<1>7. CASE: 段 E3 (`Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)`)。
  <2>1. `π` は `ty(x)` の boxed leaf であり、よって `λ = π` である。
    BY <ref id=9a6b1cd/> (a) (`leaf_origins_at` が `Some` を返すのは `π ∈ leaves(ty(x))` のときである), <ref id=c2174d1/>,
       ASSUME (`λ ⊒ π` であり `λ` は `ty(x)` の boxed leaf である)
  <2>2. `origin(x, π) = origin(args[j], σ)` であり `cand(x, π) = cand(args[j], σ)`。
    BY <ref id=9a6b1cd/> (c), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>2a. `args[j]` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E3 (`x` の束縛は `Llvm(gen, args, ・)` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>3. `σ` は `ty(args[j])` の boxed leaf であり、`P` で inhabited である。
    BY <ref id=e11772a/> (「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字であり、`σ` は
       その型の boxed leaf である」),
       <ref id=e11772a/> の「単一の `Arg(j, σ)`」の行 -- 結果のその leaf が inhabited であることと第 `j` オペランドの
       leaf `σ` が inhabited であることは同値である。<2>1 より
       `λ = π` は `P` で inhabited である。<ref id=e11772a/> のこの同値は `Let(x, Llvm(gen, args), k)` の節点の時点に
       ついてのものであり、`P` へ運ぶのは <ref id=49da857/> (b) と <2>2a である -- `x` の値も `args[j]` の値も、値を
       持った位置の後は変わらないので、両者の leaf が通る unbox union の節のタグは `P` でもその時点と
       同じである。この一歩に <ref id=49da857/> (b)、<2>2a、<ref id=66c9670/> を読む。
  <2>3a. `args[j]` の値の leaf `σ` が指すオブジェクトは、`x` の値の leaf `λ = π` が指すオブジェクトと
         同じである。
    BY <ref id=9d74736/> の移動の表の値の水準の行 (「`Llvm` の素通し leaf: 結果の leaf `λ` の宣言が単一の
       `Arg(i, σ)` であるとき、その leaf の値は**オペランド `i` の leaf `σ` の値**である。」-- この
       CASE の宣言は単一の `Arg(j, σ)` なので、`x` の値の leaf `π` の値は `args[j]` の値の leaf `σ` の
       値である。2 つの leaf の値が等しいので、2 つは同じオブジェクトを指す),
       <ref id=49da857/> (b) (`x` の値も `args[j]` の値も、値を持った位置の後は変わらない), <2>1, <2>3, <2>2a
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `args[j]` は DEF-0 の (v-1) か
         (v-2) であり、`(args[j], σ)` は `P` のスロットである。
    BY <2>3a, <2>3, <2>2a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>4. (H) の下で、`(x, π)` と `(args[j], σ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の `Llvm` の素通し leaf の行 (オペランド `i` の参照が結果へ),
       <ref id=e11772a/> の「単一の `Arg(j, σ)`」の行 (「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を
       作らない」), <ref id=ec8d1a0/>
  <2>4a. `origin(args[j], σ)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ) = (x, π)` と
         `(args[j], σ)` はどちらも `P` の位置 (D6) である。
    BY <2>1, <2>2a, <2>3, <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited
       な boxed leaf であり、`origin(x, π)` は呼ばれる),
       <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
       <ref id=9357e31/> (E3 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Llvm(gen, args), k)` を `P` までに通る),
       <ref id=9c7c27a/> (「`Llvm` の素通し leaf のオペランドから結果へ」の辺。この行はアームの中の行ではないので、
       アームの選択は要らない), DEF-1 の段 E3
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>4a, <2>2a, <1>0, 帰納法の仮定を `(args[j], σ)` に適用 --
       DEF-1 の E3 の次の 3 つ組は `(args[j], σ, σ)` であり、`σ ⊒ σ` である。帰納法の仮定の前提は
       <2>2a、<2>3、<2>4a が与え、その (iii) の前提となる `(args[j], σ)` についての (H) は <2>3b が
       与える。
       (i) は <2>2 で読み替わる。(ii) は帰納法の仮定の (ii) と <2>3a を繋いだものである。(iii) は
       (H) の下で <2>4 と帰納法の仮定の (iii) の推移であり、`(x, λ)` がスロットであることは <1>0 が
       与える。(iv) は <2>4a である。

<1>8. CASE: 段 E4a (`Llvm` かつ E3 でなく、`λ` の宣言が単一の `Arg(j, σ')`)。
  <2>1. `u_j := t_{ty(args[j])}(σ')` とおくと、この `truncate_to_unit` の呼び出しは値を返し、
        `origin(args[j], u_j)` は `reached` の要素である。
    `truncate_to_unit` は `UnitStep::NoUnit` の腕で `panic!`、`UnitStep::Capture` の腕で
    `assert_eq!`、`held_field_type` で panic しうる。この呼び出しは `origin_from_leaves_under` の中に
    在り、それは `origin(x, π)` の中で走る。L17 の ASSUME より `origin(x, π)` は呼ばれるので、
    L15 より、その呼び出しは panic せずに答えを返す。よってその中で走る
    `truncate_to_unit` も値を返す。
    BY <ref id=9a6b1cd/> (a) (`λ` は `ty(x)` の boxed leaf なので `decl` に宣言を持つ), <ref id=9a6b1cd/> (d3),
       <ref id=95f1cbf/> の ASSUME (`origin(x, π)` は呼ばれる), <ref id=0376e8d/>,
       CODE src/rc_ir/ownership.rs: truncate_to_unit (`panic!` と `assert_eq!` と
       `held_field_type` を持つ),
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`truncate_to_unit` をここで呼ぶ)
  <2>2. `cand(x, π) ⊇ cand(args[j], u_j)`。
    <3>1. `reached` の全要素が等しいとき、鍵 `(x, π)` の答えは `origin(args[j], u_j)` そのものである。
      BY <2>1, <ref id=d2c1f1f/> (b), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. そうでないとき、鍵 `(x, π)` の答えは `of_candidates(C, (x, π))` であり、
          `C ⊇ act(origin(args[j], u_j))` である。`of_candidates` の `candidates()` は `C` そのもので
          あり、`act ⊇ cand` (L2 (a)) である。
      BY <2>1, <ref id=d2c1f1f/> (b), <ref id=3de9373/>, <ref id=e05fb56/> (a), <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>2a. `args[j]` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E4a (`x` の束縛は `Llvm(gen, args, ・)` である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>3. `σ'` は `ty(args[j])` の boxed leaf であり、`σ' ⊒ u_j` であり、`P` で inhabited である。
    BY <ref id=e11772a/> (「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字であり、`σ` は
       その型の boxed leaf である」),
       <ref id=e11772a/> の「単一の `Arg(j, σ)`」の行 (inhabited の同値),
       CODE src/rc_ir/ownership.rs: truncate_to_unit (`out` は `path` の接頭辞である),
       <ref id=49da857/> (b), <2>2a, <ref id=66c9670/> -- <ref id=e11772a/> の同値は `Let(x, Llvm(gen, args), k)` の節点の時点についてのもので
       あり、`x` の値も `args[j]` の値も値を持った位置の後は変わらないので、`P` でも同じことが言える。
  <2>3a. `args[j]` の値の leaf `σ'` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY <ref id=9d74736/> の移動の表の値の水準の行 (「`Llvm` の素通し leaf: 結果の leaf `λ` の宣言が単一の
       `Arg(i, σ)` であるとき、その leaf の値は**オペランド `i` の leaf `σ` の値**である。」-- この
       CASE の宣言は単一の `Arg(j, σ')` なので、`x` の値の leaf `λ` の値は `args[j]` の値の leaf `σ'`
       の値である。その行は「**`λ` と `σ` は一般に別の path である**」と続けるので、行き先の leaf が
       `λ` と別の path であることはこの行の中にある。2 つの leaf の値が等しいので、2 つは同じ
       オブジェクトを指す),
       <ref id=49da857/> (b) (`x` の値も `args[j]` の値も、値を持った位置の後は変わらない), <2>3, <2>2a
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `args[j]` は DEF-0 の (v-1) か
         (v-2) であり、`(args[j], σ')` は `P` のスロットである。
    BY <2>3a, <2>3, <2>2a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>4. (H) の下で、`(x, λ)` と `(args[j], σ')` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, <ref id=eb90864/>, <ref id=9d74736/> の移動の表の `Llvm` の素通し leaf の行 (オペランド `i` の参照が結果へ),
       <ref id=e11772a/> の「単一の `Arg(j, σ)`」の行 (「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を
       作らない」), <ref id=ec8d1a0/>
  <2>4a. `origin(args[j], u_j)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と
         `(args[j], σ')` はどちらも `P` の位置 (D6) である。
    BY <2>2a, <2>3, <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited な
       boxed leaf であり、`origin(x, π)` は呼ばれる),
       <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
       <ref id=9357e31/> (E4 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Llvm(gen, args), k)` を `P` までに通る),
       <ref id=9c7c27a/> (「`Llvm` の素通し leaf のオペランドから結果へ」の辺。この行はアームの中の行ではないので、
       アームの選択は要らない), DEF-1 の段 E4a
  <2>5. QED
    BY <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>4a, <2>2a, <1>0, 帰納法の仮定を `(args[j], u_j)` に適用 --
       DEF-1 の E4a の次の 3 つ組は `(args[j], u_j, σ')` である。帰納法の仮定の前提は <2>2a、<2>3、
       <2>4a が
       与え、その (iii) の前提となる `(args[j], σ')` についての (H) は <2>3b が与える。帰納法の仮定の
       (i) は `cand(args[j], u_j)` の元を与え、<2>2 がそれを `cand(x, π)` の元にする。(ii) は帰納法の
       仮定の (ii) と <2>3a を繋いだものである。(iii) は (H) の下で <2>4 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。(iv) は <2>4a である。

<1>9. CASE: 段 E2 (`Join(rs)`)。
  <2>1. `α` はこの `Match` のちょうど 1 つのアームを選び、`ρ` はそのアーム本体の終端の `Ret` を `P` までに
        通っている。`P` における `x` の値はその `Ret` が名指す変数 `r_0` の値である。
    BY <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ),
       DEF-0 の (v-1) の表の第 2 行 (`x` の束縛は `Let(x, Match(scrut, arms), k)` が作るので、`x` の
       授与位置は `k` の根の節点であり、(v-1) より `ρ` はそれを `P` までに通る),
       <ref id=49da857/> (c2) (`ρ` はその位置へ進む前に `α` が選んだアーム本体の終端の `Ret` を通り、`x` はその `Ret` が
       名指す変数の値を持つ),
       <ref id=ca36627/> (`Let(x, Match(v, arms), k)` ではアームを 1 つ選ぶ), <ref id=c232680/> (活性化が選ぶアームは決まっている),
       <ref id=b3dfa37/> (`Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、`Ret(v)` はその式の値が `v` であることを
       述べる), <ref id=49da857/> (b) (`x` の値は値を得た後は変わらない),
       <ref id=9d74736/> の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の束縛変数の値は `x` の
       値である。」),
       CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
       CODE src/rc_ir/ownership.rs: returned_var
  <2>1a. `r_0` は `P` で値を持つ (DEF-0)。
    BY <ref id=9b70f28/>, DEF-1 の段 E2 (`x` の束縛は `Join(rs)` であり、`r_0` は `α` が選んだアームの結果である), <ref id=95f1cbf/> の前提 (`x` は `P` で値を持つ)
  <2>2. `ty(r_0) = ty(x)` であり、`λ` は `ty(r_0)` の boxed leaf で `P` で inhabited である。
    BY <ref id=83d98e9/> (アームの結果と `Match` の束縛変数の型が一致する), <2>1, <2>1a, <ref id=49da857/> (b), <ref id=66c9670/> -- <2>1 より
       `P` における `x` の値は `r_0` の値なので、`λ` が通る各 unbox union の節のタグも同じである。
  <2>2a. `r_0` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>1, <2>2, <2>1a, <ref id=9d74736/> の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の
       束縛変数の値は `x` の値である。」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `r_0` は DEF-0 の (v-1) か (v-2) で
         あり、`(r_0, λ)` は `P` のスロットである。
    BY <2>2a, <2>1, <2>2, <2>1a, (H), <ref id=596d4c9/> (対偶), <ref id=88a06de/> (計数下とグローバル状態は排他である), <ref id=596a46d/>
  <2>3. (H) の下で、`(x, λ)` と `(r_0, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, <ref id=eb90864/>, <2>1,
       <ref id=9d74736/> の移動の表の `Match` のアーム本体の `Ret(x)` の行 (`x` の参照が `Match` の束縛変数へ), <ref id=ec8d1a0/>
  <2>4. `C_π := ∪_{r ∈ rs} act(r, π)` とおくと、`origin(x, π) = of_candidates(C_π, (x, π))` であり、
        `cand(x, π) ⊇ cand(r_0, π)`。
    <3>1. `origin(x, π) = of_candidates(C_π, (x, π))`。
      BY <ref id=d2c1f1f/> (a), CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕,
         <ref id=3c6aa4c/> (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. `C_π` は空でない。
      BY <ref id=1172c08/> (`Match` は 1 つ以上のアームを持つ), <ref id=e05fb56/> (a) (`act` は `identity` を含むので空でない)
    <3>3. `|C_π| ≥ 2` のとき `cand(x, π) = C_π ⊇ act(r_0, π) ⊇ cand(r_0, π)`。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, <ref id=e05fb56/> (a)
    <3>4. `|C_π| = 1` のとき、`C_π = {z}` とおくと `cand(x, π) = {z}` であり、
          `cand(r_0, π) ⊆ act(r_0, π) ⊆ C_π = {z}` である。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, <ref id=e05fb56/> (a),
         <ref id=e05fb56/> (b)
    <3>5. QED
      `C_π` は空でない (`<3>2`) ので、`|C_π| ≥ 2` と `|C_π| = 1` の 2 つで尽きている。
      BY <3>1, <3>2, <3>3, <3>4
  <2>4a. `origin(r_0, π)` は呼ばれる。この段は D20 の別名の辺であり、両端 `(x, λ)` と `(r_0, λ)` は
         どちらも `P` の位置 (D6) である。
    BY <2>1 (`α` はこの `Match` のちょうど 1 つのアームを選び、`ρ` はそのアーム本体の終端の `Ret` を
       `P` までに通っている), <2>1a, <2>2,
       <ref id=95f1cbf/> の ASSUME (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で inhabited な boxed leaf で
       あり、`origin(x, π)` は呼ばれる),
       <ref id=596a46d/> (位置は、値を得た名前と inhabited な boxed leaf の対である),
       <ref id=9357e31/> (E2 は `origin_inner` の再帰の辺である), <ref id=391d1ce/> (b),
       <ref id=9b70f28/> (`ρ` は `x` の束縛を作る節点 `Let(x, Match(scrut, arms), k)` を `P` までに通る),
       <ref id=9c7c27a/> (「アーム本体の `Ret(x)` の `x` から `Match` の束縛変数へ」の辺。両端が路の位置であること
       に加え、アームの中の行なので「路がそのアームを選ぶこと」も要り、<2>1 がそれを与える),
       DEF-1 の段 E2
  <2>5. QED
    BY <2>2, <2>2a, <2>2b, <2>3, <2>4, <2>4a, <2>1a, <1>0, 帰納法の仮定を `(r_0, π)` に適用 --
       DEF-1 の E2 の
       次の 3 つ組は `(r_0, π, λ)` である。帰納法の仮定の前提は <2>1a、<2>2、<2>4a が与え、その (iii) の
       前提となる `(r_0, λ)` についての (H) は <2>2b が与える。(i) は <2>4 が `cand(x, π)` の元にする。
       (ii) は帰納法の仮定の (ii) と <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と帰納法の
       仮定の (iii) の推移であり、`(x, λ)` がスロットであることは <1>0 が与える。(iv) は <2>4a で
       ある。

<1>10. QED
  <2>1. `x` の名前が `vars.bindings` に持つ束縛による場合分けは、DEF-1 の段 E1、E2、E3、E4a、E5、E6、E7
        と停止条件 S1、S2 で尽きている。
    <3>1. `origin_inner` の `match` の腕は 6 本で、`None` と `Binding` の 7 構成子の 8 つの場合を
          尽くしている。
      BY <ref id=9357e31/>
    <3>2. `None`、`Param`、`Producer` の腕は `here()` を返す (S1 の H1、H2、H3)。`Move(y)` は E1、
          `Join(rs)` は E2 である。`Field(c, i)` は `c` が boxed なら `here()` (S1 の H4)、そうでなければ
          E5 である。`Payload(s, None)` は E6、`Payload(s, Some(t))` は `s` が boxed なら `here()`
          (S1 の H5)、そうでなければ E7 である。
      BY <ref id=0212823/>, <ref id=9357e31/>, CODE src/rc_ir/ownership.rs: origin_inner,
         <ref id=95f1cbf/> の ASSUME と DEF-0 (`x` は `P` で値を持つ `RcVar` なので `B` に現れる。これが
         <ref id=0212823/> の前提である)
    <3>3. `Llvm` の腕は、`leaf_origins_at(π)` が単一の `Arg(j, σ)` であれば E3 である。そうでないとき、
          `λ` は `ty(x)` の boxed leaf なので `decl` に宣言を持ち、その宣言は単一の `Arg` (E4a)、単一の
          `Fresh` または単一の `Unknown` (S2)、空集合、要素数 2 以上のいずれかである。
          **この場合分けが読む `decl` は 1 つに決まる** -- 3 つの条件はどれも
          `gen.result_prov(ty(x), arg_tys, type_env)` の返り値を読むので、その呼び出しが同じ引数に同じ
          値を返すことが要る。
      BY <ref id=9a6b1cd/> (a), <ref id=9a6b1cd/> (b), <ref id=9a6b1cd/> (c), <ref id=9a6b1cd/> (d), <ref id=95f1cbf/> の前提 (`λ` は `ty(x)` の boxed leaf である),
         <ref id=e11772a/> (「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である**」-- 同じ引数に対して常に同じ値を
         返す)
    <3>4. 空集合と要素数 2 以上は起きない。
      BY <ref id=e11772a/> (空集合と宣言された leaf は inhabited にならない。「**複数の元を宣言する op は存在しない。**」),
         <ref id=95f1cbf/> の前提 (`λ` は `P` で inhabited である)
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4
  <2>2. 鎖は有限で止まる。
    ASSUME より `origin(x, π)` は呼ばれるので、L14 (a) を `K_0 = (x, π)` に当てられる。L14 (a) より
    `(x, π)` から到達する鍵の上でこの関係は整礎であり、DEF-1 の各段は `origin_inner` の再帰呼び出しの
    1 つに一致する (L6) ので、鎖の各段はその関係の辺 1 本を進む。
    BY ASSUME, <1>1, <1>2 (停止条件では鎖の長さは 0 である), <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9
       (段では帰納法の仮定が次の 3 つ組からの鎖の有限性を与える), <ref id=9357e31/>,
       <ref id=391d1ce/> (a) (再帰呼び出しの関係は、呼ばれる鍵から到達する鍵の上で整礎である),
       EXT 整礎性 ((b) 整礎な関係の上では整礎帰納が使える)
  <2>3. QED
    BY <2>1, <2>2, <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9 -- <2>1 の 9 つの場合が
       上の 9 つの CASE であり、どの場合も (i)、(ii)、(iii)、(iv) が成り立つ。<1>0 が、(iii) のうち
       `(x, λ)` が `P` のスロットであるという部分をすべての場合について与える。(iv) は、停止条件の
       2 つの CASE では空虚であり、7 つの段の CASE ではその CASE の対応する段が与える。

**系 1 (P3)**。**この `vars` と `type_env` について `origin(x, π)` が呼ばれる**とし、
`origin(x, π) = Exactly(u, σ)` とする。すべての活性化、それが辿る実行路、およびその上の
すべての位置 `P` において、`x` が `P` で値を持ち (DEF-0)、`λ` が `π` の下の `P` で inhabited な leaf で
あるとき、次の 2 つが成り立つ。

- **(a)** `λ` に対応する位置 (DEF-1) の変数は `u`、その path は `σ` の下の leaf (`σ` 自身を含む) で
  あり、その位置は `(x, λ)` と**同じオブジェクトを指す**。
- **(b)** さらに**スロット `(x, λ)` が `P` で D8 の意味の参照を持つとき**、対応する位置は `P` の
  スロットである。これが D17 の「対応するスロット」であり、それは `(x, λ)` と同一の参照を持つ。

**「`origin(x, π)` が呼ばれる」を前提に置く。** 証明はL17 を経て `L14` (a) の整礎性に立ち、
`L14` (a) が言うのは「`origin` が呼ばれる鍵 `K_0` を取ると」から先だけである -- その証明が使う `t(K)` は
`vars.origins` への実際の `insert` の順番なので、呼ばれていない鍵には定まらない。**これは README の
P3 が言明の先頭に置く「解析がその鍵で `origin` を呼び」と同じ条件である。** README は P4 の脇で、その
節が要る理由としてこの文書の `L14` (a) を名指し、「**この節は下流を狭めない** -- P3・P4 を読む段は
`origin` を呼んでその答えを見るので、節はそこで満たされる」と述べる。**よって系 1 が示すのは
README の P3 そのものである。**

(b) の限定の形は README の P3 の言明と同じであり、(a) が README の P3 の「**条件を外した形では、
2 つの位置 (D6) は同じオブジェクトを指す。**」に当たる。**「`x` が `P` で値を持つ」を前提に書くのは、
README の P3 が「スロット `(x, λ)`」と「2 つの位置 (D6)」を主語に取るからである** -- D6 のスロットも
記号の位置も、値を得た名前と inhabited な boxed leaf の対なので、値を持たない `x` については
README の P3 も何も述べない。**(a) の対応する位置はスロットとは限らない** --
鎖は `vars.bindings` に束縛を持たない名前で止まりうるので、そのとき対応する位置は D6 の**記号の位置**で
ある。D6 が「スロットと記号の位置を合わせて**位置**と呼ぶ」と定め、「位置を主語にする定義は記号の位置を
含み、参照を主語にする定義は含まない」と述べるのがこの区別である。

<1>1. `cand(x, π) = {(u, σ)}`。
  BY 系 1 の前提 (`origin(x, π) = Exactly(u, σ)` である), <ref id=e05fb56/> (b)
<1>2. L17 の前提が満たされる。
  BY 系 1 の前提 (`x` は `P` で値を持ち、`λ` は `P` で inhabited な `ty(x)` の boxed leaf であり、
     `origin(x, π)` は呼ばれ、その値は `Exactly(u, σ)` である)
<1>3. (a)。
  L17 の (i) より停止点の `VarPath` は `(u, σ)` であり、(ii) よりその位置は `P` で値を持つ `u` と
  `μ ⊒ σ` の leaf からなり、`(x, λ)` と同じオブジェクトを指す。
  BY <1>1, <1>2, <ref id=95f1cbf/>
<1>4. QED
  BY <1>1, <1>2, <1>3, <ref id=eb90864/> (`(x, λ)` が <ref id=ec8d1a0/> の意味の参照を持つとき `obj(x, λ)` は計数下であり、これが
     <ref id=95f1cbf/> の (H) である), <ref id=95f1cbf/> -- (a) は <1>3、(b) は <ref id=95f1cbf/> の (iii) が与える。

**系 2 (P4)**。**この `vars` と `type_env` について `origin(x, π)` が呼ばれる**とし、
`origin(x, π) = Join { identity, candidates }` とする。すべての活性化、それが辿る実行路、
およびその上のすべての位置 `P` において、`x` が `P` で値を持ち (DEF-0)、`λ` が `π` の下の `P` で
inhabited な leaf であるとき、次の 2 つが成り立つ。

- **(a)** `λ` に対応する位置 (DEF-1) の `VarPath` は `candidates` のいずれかの下にあり、その位置は
  `(x, λ)` と**同じオブジェクトを指す**。
- **(b)** さらに**スロット `(x, λ)` が `P` で D8 の意味の参照を持つとき**、対応する位置は `P` の
  スロットである。これが D17 の「対応するスロット」であり、それは `(x, λ)` と同一の参照を持つ。

**「`origin(x, π)` が呼ばれる」を前提に置く理由は系 1 と同じであり、それは README の P4 が言明の
先頭に置く「解析がその鍵で `origin` を呼び」と同じ条件である。**

(b) の限定の形は README の P4 の言明と同じであり、(a) が README の P4 の「**条件を外した形では、その
位置 (D6) と対応する位置のいずれかは同じオブジェクトを指す。**」に当たる。**よって系 2 は README の
P4 を示し、さらにそれより強い** -- L17 はその「いずれか」がどれであるかを 1 つに決めるからである。
対応する位置は DEF-1 の鎖が止まった位置である。(a) の位置が
スロットとは限らないことと、「`x` が `P` で値を持つ」を前提に書く理由は、系 1 と同じである。

<1>1. `cand(x, π) = candidates`。
  BY 系 2 の前提 (`origin(x, π) = Join { identity, candidates }` である),
     CODE src/rc_ir/ownership.rs: Origin::candidates
<1>2. L17 の前提が満たされる。
  BY 系 2 の前提 (`x` は `P` で値を持ち、`λ` は `P` で inhabited な `ty(x)` の boxed leaf であり、
     `origin(x, π)` は呼ばれ、その値は `Join { identity, candidates }` である)
<1>3. (a)。
  L17 の (i) より停止点の `VarPath` は `candidates` の元であり、(ii) の `μ ⊒ σ_end` より対応する位置は
  その元の下にあって、`(x, λ)` と同じオブジェクトを指す。
  BY <1>1, <1>2, <ref id=95f1cbf/>
<1>4. QED
  BY <1>1, <1>2, <1>3, <ref id=eb90864/> (`(x, λ)` が <ref id=ec8d1a0/> の意味の参照を持つとき `obj(x, λ)` は計数下であり、これが
     <ref id=95f1cbf/> の (H) である), <ref id=95f1cbf/> -- (a) は <1>3、(b) は <ref id=95f1cbf/> の (iii) が与える。

**系 3 (DEF-1 の鎖は D33 の `ρ` 歩みである)**。L17 の ASSUME を満たす `α`、`ρ`、`P`、`x`、`π`、`λ` を
取る。DEF-1 の 3 つ組の列を (現在の変数, 現在の leaf) へ写した列は、位置 `(x, λ)` から始まる D33 の
`ρ` 歩みであり、その停止点 `(u, μ)` はその `ρ` 終端である。

**D33 が歩みを定めるのは位置についてであり、`x` が (v-3) の名前であるときは始点が記号の位置になる**
-- そのとき鎖は L9 の H1 で直ちに止まる。

<1>0. DEF-1 の各段が進む先の leaf は、D17 の写り方に一致する。**段 E3 はこの一致に `λ_cur = π_cur` を
      要る** -- その条件が読む宣言は `leaf_origins_at(π_cur)` であって `λ_cur` の宣言ではないのに、
      D17 の `Binding::Llvm` の行は「`λ` を、`λ` 自身の宣言 `Arg(j, σ')` の `σ'` へ置き換える」と
      書くからである。E3 の条件が成り立つとき `π_cur` は `ty(x_cur)` の boxed leaf であり、
      L17 の ASSUME の `λ_cur ⊒ π_cur` と L5 より `λ_cur = π_cur` である。残る 6 つの段は `λ_cur` を
      変えないか (E1、E2、E6)、先頭に添字を足すか (E5、E7)、`λ_cur` 自身の宣言を読む (E4a) ので、
      表の第 4 列が名指す D17 の行がそのまま当たる。
  BY DEF-1 (表と第 4 列), <ref id=d59f90b/> (辺ごとの `λ` の写り方の 3 行), <ref id=9a6b1cd/> (a) (`leaf_origins_at` が `Some` を
     返すのは `π_cur ∈ leaves(ty(x_cur))` のときである), <ref id=c2174d1/> (相異なる 2 つの boxed leaf の一方が
     他方の接頭辞になることは無い), <ref id=95f1cbf/> の ASSUME (`λ ⊒ π` であり `λ` は `ty(x)` の boxed leaf
     である), <ref id=95f1cbf/> ((iv) -- 各段の行き先の 3 つ組も <ref id=95f1cbf/> の ASSUME を満たすので、鎖のどの 3 つ組でも
     `λ_cur ⊒ π_cur` と「`λ_cur` は `ty(x_cur)` の boxed leaf である」が成り立つ)
<1>1. 鎖の各段は D20 の別名の辺であり、写した列はその辺を 1 本ずつ進む。**鎖のどの 3 つ組も
      L17 の ASSUME を満たす。**
  L14 (a) が与える整礎関係 -- `(x, π)` から到達する鍵の上の再帰の辺の関係 -- の上の整礎帰納による。
  鎖が停止条件で始まるときは段が無いので空虚である。そうでないとき、L17 の (iv) より第 1 段は D20 の
  別名の辺であり、その行き先 `(v, π', λ')` について `v` は `P` で値を持ち、`λ'` は `ty(v)` の `P` で
  inhabited な boxed leaf であって `λ' ⊒ π'` であり、`origin(v, π')` は呼ばれる。すなわち
  `(v, π', λ')` は L17 の ASSUME を満たすので、帰納法の仮定がそこから先の段について同じことを与える。
  鎖の第 2 段以降は `(v, π', λ')` から始まる鎖の段である (DEF-1 -- 規則は現在の 3 つ組だけで決まる)。
  各段が進む先の leaf が D17 の写り方と一致することは <1>0 が与える。
  BY <1>0, <ref id=95f1cbf/> ((iv)), <ref id=9357e31/> (DEF-1 の各段は `origin_inner` の再帰の辺 1 本である), <ref id=391d1ce/> (a),
     EXT 整礎性 ((b) 整礎な関係の上では整礎帰納が使える), DEF-1 (表の第 4 列),
     <ref id=d59f90b/> (辺ごとの `λ` の写り方と、辺の行き先の 3 行), <ref id=30d6238/> (歩みは <ref id=9c7c27a/> の別名の辺を辿り、その行き先は
     <ref id=d59f90b/> が定める `λ` に対応する位置である)
<1>2. 鎖が止まる位置は、D33 が歩みを止める位置と一致する。
  D33 が歩みを止めるのは、辺を持たない束縛 (`Binding::Param` が L9 の H2、`Binding::Producer` が H3、
  束縛を持たない名前が H1)、`Binding::Llvm` であって `λ_cur` の宣言が単一の `Fresh` または単一の
  `Unknown` である位置 (DEF-1 の S2)、boxed 容器の `Destructure` の名前付きフィールド (H4)、
  boxed union の変位アームの payload (H5) である。DEF-1 の S1 は H1 から H5 の 5 つ、S2 は残る 1 つで
  あり、この 2 つは D33 の一覧を尽くす。
  BY <ref id=30d6238/> (歩みを止める 3 つの箇条), DEF-1 の S1 と S2, <ref id=0212823/> (H1 から H5),
     <1>1 (鎖のどの 3 つ組もL17 の ASSUME を満たすので、その現在の変数は `P` で値を持つ `RcVar`
     である), DEF-0 (値を持つのは本体に現れる `RcVar` なので、その変数は <ref id=0212823/> の前提を満たす)
<1>3. QED
  BY <1>0, <1>1, <1>2, <ref id=30d6238/> (`ρ` 歩みは、<ref id=9c7c27a/> の別名の辺を辿り、止まる位置で終わる列である), <ref id=95f1cbf/> ((iv) が
     鎖の各段の両端を `P` の位置にするので、写した列は <ref id=30d6238/> が歩む位置の列である)

**この系が要るのは、D20 が辺の存在を条件つきに定めるからである。** L6 の第 2 の表は、各段が D9 の
移動の表のどの行の下にあるかを述べるだけで、その辺が**この**活性化に在ることを述べない。読む者は、
`ρ` 歩みと `ρ` 終端を主語にする README の定義 -- D33 の別名類と D34 の `held` -- を、この文書の鎖の
上で読む段である。

**候補集合が広いことは L17 を弱めない。** L17 の証明が候補集合について使うのは「`cand(x, π)` が内側の
候補を**含む**」という向きだけであり、使うのは停止条件 S2 の CASE、段 E4a の CASE、段 E2 の CASE の
3 つである。`of_candidates` に渡る集合は畳み込む各 `Origin` の `acted_on()` の和であり、`act ⊇ cand`
(L2 (a)) なので `candidates()` の和を含む。含む向きに広いことは、この 3 か所のどれも壊さない。

## 7. unit の path と leaf の path が別の答えになること

**この節は P3 と P4 の証明の外にある観察である。**

`origin(v, π)` と `origin(v, λ)` (`λ ⊒ π`) は別々の問いであり、後者の `identity` が前者の答えのどこにも
現れないことがある。`Binding::Join(rs)` の腕は 2 つの問いをそれぞれ各アームへ降ろし、集めた候補の個数が
答えの形を決める (`CODE src/rc_ir/ownership.rs: origin_inner`, `Origin::of_candidates`)。unit の path で
候補が 1 つに畳まれ、その下の leaf の path で 2 つ以上残ると、unit の側の答えは `Exactly` で `v` の名前を
持たず、leaf の側の答えは `Join` で `identity` が `(v, λ)` になる。

形は次である。`Node` を boxed 構造体、`Choice` を `unbox union { a : Node, b : Node }` とし、1 つの `Node`
の値 `node` から 2 つのアームがそれぞれ別の変位を作って `Match` の束縛変数 `m` に集める。`leaves(Choice)`
は `[0]` と `[1]`、`rc_units(Choice)` は `[]` である (D4、D5)。`node` は `struct_make` の結果であり、その
`result_prov` は boxed 構造体の唯一の leaf `[]` に単一の `Fresh` を置くので
(`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody` の `result_prov` の
`None => sole_origin(LeafOrigin::Fresh)` の枝)、`origin(node, []) = Exactly((node, []))` である
(L8 (d1)、L4 (b))。`union_make` の `result_prov` は、作った変位の leaf に単一の `Arg(0, ..)`、他の
変位の leaf に空集合を宣言する
(`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeUnionBody` の `result_prov`)。変位 0 を作る
`x` について、`origin(x, [])` は `origin_from_leaves_under` を通り、空集合を宣言された leaf `[1]` は
ループを 1 度も回さないので `reached` は `origin(node, [])` の 1 要素になり、答えは
`Exactly((node, []))` である。
`origin(x, [1])` は `leaf_origins_under([1])` が空集合だけを返して `reached` が空になり、`origin_inner`
の `unwrap_or_else(here)` が `Exactly((x, [1]))` を答える (L8 (d2))。変位 1 を作る `y` については、
leaf `[1]` の宣言が単一の `Arg(0, [])`、leaf `[0]` の宣言が空集合なので、
`origin(y, []) = Exactly((node, []))` かつ `origin(y, [0]) = Exactly((y, [0]))` である。よって `m` では
`act(x, []) ∪ act(y, []) = {(node, [])}` が 1 要素で `origin(m, []) = Exactly((node, []))` となり、
`act(x, [0]) ∪ act(y, [0]) = {(node, []), (y, [0])}` が 2 要素で
`origin(m, [0]) = Join { identity: (m, [0]), candidates: {(node, []), (y, [0])} }` となる。`(m, [0])` は
`origin(m, [])` の答えのどこにも現れない。

`origin(m, []) = Exactly((node, []))` は、`Origin::of_candidates` の `1 =>` の腕が返す `Exactly` の
`VarPath` が呼び出し自身の `(var, path)` とは限らないことの実例である (L9 の証明がその形を扱う)。

この形は普通の Fix のソースから出る。boxed 構造体を 1 つ作り、`if` の 2 つの枝でそれぞれ別の変位の union
を作って、その union を関数に渡すプログラムを `-O max --emit-rc-ir all` でコンパイルすると、
`.fixlang/rc_ir.pre.txt` に次が現れる (名前を短くし、無関係な行を落とした)。

```
let node : Main::Node = struct_make(k)
...
let m : Main::Choice = match cond {
    case 1(unit): let x : Main::Choice = union_make_0(node)
                  ret x
    case 0(unit): let y : Main::Choice = union_make_1(node)
                  ret y
}
let seen : Std::I64 = Main::peek(m, two)
```

**この食い違いに依拠する読み手は無い。** `borrow_ify` と `cancel` が住む `src/rc_ir/borrow.rs` に
`origin` の呼び出しは 7 か所ある。**この一覧はそのファイルの `origin(` の呼び出しを数え上げて作る。**
そのうち 3 つは leaf の path しか渡さない。

- `infer_ownership` は `collect_consumes` が報告した `(var, path)` を渡す
  (`CODE src/rc_ir/borrow.rs: infer_ownership`)。`collect_consumes` が `out` に積むのは
  `push_boxed_leaves`、`destructure_consumes`、`rhs_consumes` が挙げる path であり、3 つとも
  `boxed_leaf_paths` の要素である
  (`CODE src/rc_ir/ownership.rs: collect_consumes`, `collect_consumes_go`, `destructure_consumes`,
  `rhs_consumes`, `push_boxed_leaves`)。
- `CancelAnalysis::consume` は `rhs_consumes` と `destructure_consumes` が報告した leaf を渡す
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`, `CancelAnalysis::consume_rhs`)。
- `CancelAnalysis::other_objects` は `boxed_leaf_paths` の各要素を渡す
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。

残る 4 つが leaf でない path を渡しうる。どれも答えの候補が名指す**根**について `owns_object` /
`owns_object_yet` か `used_later` を引くだけで、leaf の `identity` を unit の答えから引かない。

- `RewriteCtx::owns_unit` は候補すべてに `owns_object` を要求する
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit`)。
- `RewriteCtx::check_ownership_is_levelled` は候補の `owns_object` が揃うことを表明する
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`)。
- `routing_saves_retain` は `comes_from_a_value_used_later` を通じて候補の根に `used_later` を引く
  (`CODE src/rc_ir/borrow.rs: routing_saves_retain`,
  `CODE src/rc_ir/borrow.rs: RewriteCtx::comes_from_a_value_used_later` -- `origin` の呼び出しは
  この関数の中に在る)。
- `level_ownership` は候補の根の所有を読み、所有の側へ倒す
  (`CODE src/rc_ir/borrow.rs: level_ownership`)。

`cancel` の走査は unit の path で `origin` を問わない。`Retain`/`Release` が触れる先は
`acted_references` と `CancelAnalysis::other_objects` が leaf ごとに `origin` を問うて作り
(`CODE src/rc_ir/ownership.rs: acted_references`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)、
消費の側は `rhs_consumes` と `destructure_consumes` が報告する leaf ごとに `CancelAnalysis::consume` が
`origin` を問う (`CODE src/rc_ir/ownership.rs: rhs_consumes`, `destructure_consumes`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`)。P5 (c) が結ぶ 2 つの量はどちらも leaf ごとの量で
あり、unit の側の答えを読まない。unit の答えと leaf の答えの食い違いを扱うのは P7a であり、その言明は
P7a の意味の site -- その版が書き換える本体を `for_each_node` で歩いて挙げた `Retain`/`Release` 節点の
`(v, path)` と、`App` の各引数と各 unit の対 -- と `infer_ownership` の不動点に限り、かつ
inhabited (D16) な leaf に限る形でそれを述べる。

## 8. `level_ownership` が P3 と P4 に及ぼすもの

`level_ownership` は `infer_ownership` の不動点の中で走る段である
(`CODE src/rc_ir/borrow.rs: infer_ownership`, `levelled_sites`, `level_ownership`)。

<1>1. P3 と P4 の言明が読む関数は `origin` であり、D17 の対応するスロットを決めるのは `origin_inner` と
      `origin_from_leaves_under` である。
  BY <ref id=eb26ddb/> (言明が読む関数は `origin` である), <ref id=e19246e/> (同じ), <ref id=3f68b95/>, <ref id=d59f90b/>
<1>1a. P3 と P4 の言明の先頭の節「**解析がその鍵で `origin` を呼び**」が覆う鍵の集合は、
       `level_ownership` の有無で動く -- `level_ownership` は `origin` の呼び出し元の 1 つだからで
       ある。それでも 2 つの言明の真偽は動かない。系 1 と系 2 はこの `vars` と `type_env` について
       `origin(x, π)` が呼ばれる**任意の**鍵について立つので、その集合が増えても言明は各鍵で成り立ち、
       減っても残る鍵で成り立つ。
  BY 系 1, 系 2, <ref id=95f1cbf/> (ASSUME が鍵に課すのは「`origin(x, π)` が呼ばれる」ことだけである),
     <ref id=eb26ddb/> (言明の先頭の節「**解析がその鍵で `origin` を呼び**」), <ref id=e19246e/> (同じ節を持つ),
     CODE src/rc_ir/borrow.rs: level_ownership (`origin` を呼ぶ)
<1>2. この 3 つが読むのは `VarTable` の `bindings` と `origins` の memo、`TypeEnv`、および `bindings` が
      持つ `LLVMGen` の `result_prov` の返り値だけである。**`var_tys` と `param_tys` は読まない** --
      `Llvm` の腕が使うオペランドの型は、`Binding::Llvm` が持つ `Vec<RcVar>` の要素の `ty` の欄から来る。
  BY CODE src/rc_ir/ownership.rs: origin (`vars` について読み書きするのは `origins` だけである),
     CODE src/rc_ir/ownership.rs: origin_inner (`vars.bindings.get(var)` のほかに `vars` の欄を読まない。
     `Llvm` の腕の `arg_tys` は `args.iter().map(|a| a.ty.clone())` であり、`args` は `Binding::Llvm` の
     欄である),
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under (型を取るのは `args[*j].ty` からである),
     CODE src/rc_ir/ownership.rs: as_arg_projection (`VarTable` を引数に取らない),
     CODE src/rc_ir/ownership.rs: truncate_to_unit (引数は型・path・`TypeEnv` であり `VarTable` を
     取らない)
<1>3. `level_ownership` の実行が書き込むのは 3 種である -- `infer_ownership` の局所変数
      `owned_leaves`、それが呼ぶ `origin` を通じた `VarTable` の `origins` の memo、そして
      到達する `TypeNode` から届く、一度だけ書かれる memo である。第 3 の種の在りかは
      `TypeNode` 自身の `hash_cache`・`ground_cache`・`depth_cache` と、`info.source` から
      `Span.input` を経て届く `SourceFile` の `string` と `hash` である。**`VarTable` の残りの欄
      (`bindings`、`closure_targets`、`param_tys`、`var_tys`) と `TypeEnv` の値は変わらない。**
  <2>1. `level_ownership` の実行が `infer_ownership` の局所変数へ行う書き込みは `owned_leaves.insert`
        だけである。
        `level_ownership` が受け取る可変参照は `owned_leaves: &mut Set<VarPath>` の 1 つであり、
        本体がそれに行う動作は `owned_leaves.insert` である。本体が呼ぶ残りは `origin`、
        `Origin::candidates`、`owns_object_yet`、`vars.param_tys.get`、`covered_leaves` である。
        `Origin::candidates` と `vars.param_tys.get` は共有参照から読んで新しい値を作るだけ、
        `covered_leaves` は型・path・`TypeEnv` を取って `Vec<FieldPath>` を返すだけであり、
        `owns_object_yet` は `owned_leaves` を共有参照で受け取るのでそれを値として変えない。
        `origin` は `VarTable` の `origins` の memo を書く。
    BY CODE src/rc_ir/borrow.rs: level_ownership (引数は `&VarTable`、`&TypeEnv`、site、
       `&mut Set<VarPath>` であり、本体が呼ぶのは `origin`、`Origin::candidates`、
       `owns_object_yet`、`vars.param_tys.get`、`covered_leaves` であって、`owned_leaves` へ行う
       動作は `owned_leaves.insert` である),
       CODE src/rc_ir/ownership.rs: Origin::candidates (`&self` を取り `Vec<&VarPath>` を返す),
       CODE src/rc_ir/borrow.rs: covered_leaves (`&Arc<TypeNode>`、`&FieldPath`、`&TypeEnv` を取り
       `Vec<FieldPath>` を返す),
       CODE src/misc.rs: Map (`FxHashMap` の別名であり、`get` は共有参照から読む),
       CODE src/rc_ir/borrow.rs: owns_object_yet (`owned_leaves` を `&Set<VarPath>` で受け取る),
       EXT 借用規則 (共有参照を通じて書き込めるのは内部可変性を持つ欄だけである。`Set<VarPath>` は
       その欄を持たない),
       EXT 内部可変性 ((1) `RefCell` の中身は共有参照から書き替えられるので、`&VarTable` からでも
       `origins` に書ける),
       CODE src/rc_ir/ownership.rs: origin (`vars.origins.borrow_mut().insert(key, answer.clone())` が
       memo を書く)
  <2>2. `level_ownership` の実行が書きうる memo は、この段の親の言明が第 3 の種として挙げる欄の
        いずれかである。
        **「`VarTable` の 5 つの欄のうち `RefCell` を持つのは `origins` だけである」では、内部可変性の
        数え上げは尽きない** -- `param_tys`・`var_tys`・`bindings` は `Arc<TypeNode>` を持ち、
        `TypeNode` からは内部可変性を持つ欄が届くからである。**道を数えるのではなく、`TypeNode` から
        届く欄を数える。** 在りかを型で決めるのは A3 であり、その走査が `TypeNode` から届くものとして
        挙げるのは、`TypeNode` 自身の `OnceLock` の 3 欄と、`info.source` から `Span.input` を経て届く
        `SourceFile` の `string` と `hash` である。**`OnceLock` の 3 欄だけでは足りないことも、A3 が
        その節で述べる。**
        **道の数え上げでは閉じない。** `level_ownership` は `vars.param_tys.get(root)` が返す型を
        `covered_leaves` へ渡し、`owns_object_yet` も同じ型を取って `boxed_leaf_paths` を呼び、
        `origin` は `origin_inner` の `Binding::Llvm` の腕で
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を呼ぶ。`origin` の道はそこで尽きず、
        `origin_from_leaves_under` は `truncate_to_unit` を通って `unit_step` から
        `unpunched_field_types` へ降り、`origin_inner` は `container.ty.is_box` と `scrut.ty.is_box`
        から `toplevel_tycon_info` へ降りる。A3 は `result_prov` の道を名指しているが
        (「**`Box<dyn LLVMGen>` の op が内部可変性に届くのもこの道である**」)、書かれる欄がどれで
        あるかを決めるのは道ではなく型である。
        1 本の道を書き下すと次である。`result_prov` が返す `Provenance` は、既定と override の
        いずれについても `Provenance::uniform`・`build_shape`・`uniform_bottom`・`fresh_under`・
        `replaced_field_prov` のいずれかを通り、その 5 つはどれも `LeafMap::build_shape` を経て
        `boxed_leaf_paths` を呼ぶ。`boxed_leaf_paths` の走査は `unpunched_field_types` を呼ぶ。
        `unpunched_field_types` は `instance_field_types` を経て、tycon が kind `*` でない型変数を
        持つとき `unwrap_newtypes_memoized` を呼び、そこで `Map<Arc<TypeNode>, Arc<TypeNode>>` を
        `Arc<TypeNode>` の鍵で引く。`Map` は `FxHashMap` なので、`get` はその鍵を hash する
        (EXT 標準ライブラリのハッシュ (2))。鍵の型は `Arc<TypeNode>` であり、その `hash` は指す先の
        `TypeNode` の `hash` を呼ぶ (EXT 標準ライブラリのハッシュ (1))。
        `impl Hash for TypeNode` は `type_hash` を呼び、`type_hash` は `hash_cache.get_or_init` を
        走らせる -- 共有参照から `hash_cache` を埋める (EXT 内部可変性 (2))。この道で書かれるのは
        `hash_cache` であり、これは A3 の走査が挙げる欄の 1 つである。
    BY <ref id=e11772a/> (「**在りかは型で決める。** `RefCell`・`Cell`・`OnceCell`・`OnceLock`・
       `Mutex`・`RwLock`・`UnsafeCell`・`Atomic*` のいずれかを含む欄の宣言を走査し、その値から
       到達できるものを取る。」「**特定のメソッド名や特定の欄で数え上げると、別の型を経て届く道が
       落ちる。**」「**`OnceLock` の 3 欄だけを数えると足りない。** `TypeNode` は `info.source` から
       `Span.input` を経て `SourceFile` に届き、その `string` と `hash` は
       `Arc<Mutex<Option<String>>>` である」「**`Box<dyn LLVMGen>` の op が内部可変性に届くのも
       この道である**」),
       EXT 標準ライブラリのハッシュ, EXT 内部可変性,
       CODE src/rc_ir/borrow.rs: level_ownership, covered_leaves,
       CODE src/rc_ir/borrow.rs: owns_object_yet (`vars.param_tys.get(root)` が返す型に
       `boxed_leaf_paths` を掛ける),
       CODE src/rc_ir/ownership.rs: origin_inner (`Binding::Llvm` の腕が `result_prov` を呼び、
       `Field` と `Payload` の腕が `container.ty.is_box` と `scrut.ty.is_box` を呼ぶ),
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under, truncate_to_unit, unit_step
       (`truncate_to_unit` は `unit_step` を通って `unpunched_field_types` へ降りる),
       CODE src/ast/types.rs: TypeNode::is_box, TypeNode::is_unbox, TypeNode::toplevel_tycon_info,
       CODE src/rc_ir/provenance.rs: Provenance::build_shape, Provenance::uniform,
       Provenance::uniform_bottom, Provenance::fresh_under,
       CODE src/fixstd/builtin.rs: replaced_field_prov,
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape (`boxed_leaf_paths` を呼ぶ),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types, TypeNode::instance_field_types,
       TypeNode::unwrap_newtypes_memoized, TypeNode::type_hash, TypeNode,
       CODE src/ast/types.rs: impl Hash for TypeNode,
       CODE src/parse/sourcefile.rs: SourceFile (`string` と `hash` は
       `Arc<Mutex<Option<String>>>` である),
       CODE src/misc.rs: Map (`FxHashMap` の別名である)
  <2>3. その書き込みは、`VarTable` の残りの欄と `TypeEnv` が持つ値の等しさを変えない。
    BY <2>2, <ref id=e11772a/> (「**`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変え
       ない。** 到達できる型が内部可変性を持つ欄を持つときは、その欄は**一度だけ書かれる memo で
       あって、その値はその型の `PartialEq` が読む成分の関数である**」、および
       「**「内部可変性を持たない」と書くと偽になる。**」の節 -- `TypeNode` の `hash_cache`・
       `ground_cache`・`depth_cache` を名指し、「**その 3 つは一度だけ書かれる memo であり、
       `impl PartialEq for TypeNode` は `ty` だけを読み、3 つの memo の値はどれも `ty` の関数で
       ある**」「**`impl Hash for TypeNode` は `type_hash` を呼ぶので `hash_cache` を読み、かつ
       書く。**反映されるのは `ty` だけなので、等しい 2 つの値は等しくハッシュされる」と述べる。
       `SourceFile` の 2 つについては「**この 2 つも一度だけ書かれる memo であり、
       `impl PartialEq for TypeNode` が読むのは `ty` だけなので、埋まっても値の等しさは
       動かない。**」と述べる)
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>3a. `origins` の memo への書き込みは鍵の答えを変えない。
  P2a は 1 つの `VarTable` の値と 1 つの `TypeEnv` の値を固定したうえで、鍵 `(x, π)` が等しい 2 つの
  `origin` の呼び出しの返り値が等しいこと、すなわち「答えは `vars.origins` が保持する memo の状態に
  依らない」ことを述べる。`<1>3` より `level_ownership` は同じ 1 つの `VarTable` の `origins` を書き、
  `bindings` と `TypeEnv` の値は動かさないので、P2a の固定した範囲がそのまま当たる。
  **表を跨ぐ形は引かない** -- P2a は「`bindings` が等しい相異なる 2 つの `VarTable` について答えが
  等しい」ことを主張しないので、この段はその形を使わない。
  BY <ref id=b1f6e13/>, <1>3
<1>4. QED
  `<1>2` が挙げる入力 -- `VarTable` の `bindings`、`TypeEnv`、`bindings` が持つ `LLVMGen` の
  `result_prov` の返り値、および `origins` の memo -- のうち、`level_ownership` の実行が**値として**
  変えるのは `origins` の memo だけである (`<1>3`) -- `TypeNode` の memo への書き込みは `bindings` と
  `TypeEnv` の値を動かさない。`owned_leaves` はその並びに入らない。`bindings` と `TypeEnv` の値が
  動かないので (`<1>3`)、`origin_inner` がそれらの型について呼ぶ `is_box`・`truncate_to_unit`・
  `result_prov` も同じ答えを返す -- 前の 2 つは型の値の関数であり、`result_prov` は決定的である
  (A3)。`<1>3a` より `origins` の memo への書き込みは答えを変えない。
  **比較は 1 つの `VarTable` の値の上で行う** -- `level_ownership` は `bindings` を書かないので
  (`<1>3`)、その有無で変わるのは同じ表の `origins` だけであり、P2a の固定した範囲を出ない。
  鍵ごとの答えが動かないことと、言明の前件が覆う鍵の集合が動いても真偽が動かないこと (`<1>1a`) を
  合わせて、P3 と P4 の真偽は `level_ownership` の有無で変わらない。
  BY <ref id=b1f6e13/>, <ref id=e11772a/> (値の等しさの節と、「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である**」の節),
     <1>1, <1>1a, <1>2, <1>3, <1>3a

**観察 (この文書の命題の外)。** `level_ownership` の発火判定は、site の `origin` の候補のうち 1 つでも
`owns_object_yet` が真であれば真になる (`CODE src/rc_ir/borrow.rs: level_ownership`, `owns_object_yet`)。
site の答えが `of_candidates(C, ・)` で作られるとき、`candidates()` は `C` そのものであり
(`CODE src/rc_ir/ownership.rs: Origin::of_candidates`, `Origin::candidates`)、`C` は畳み込まれた各
`Origin` の `acted_on()` の和である。よってそのうち 1 つでも `Join` であれば、その `identity` が
`candidates()` に入る (L2 (a))。`Join` の `identity` の根は `Let` が束縛する局所変数である --
パラメータと capture の binding は `Binding::Param` であり、その腕は `here()` を返して `Join` を作らない
(`CODE src/rc_ir/ownership.rs: origin_inner`)。局所変数は `vars.param_tys` に無いので `owns_object_yet` は
それを真と答える。すなわち、site の候補集合が `Join` の `identity` を含むとき、`level_ownership` は必ず
発火し、その site の候補が名指すパラメータ leaf をすべて所有へ倒す。所有が増える向きなので、この段の doc が
述べるとおり「costs a count rather than correctness」であり、P8 と P14 の側で見るべき事柄である。
