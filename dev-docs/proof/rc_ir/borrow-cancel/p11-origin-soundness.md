# P3 / P4 -- `origin` の健全性

対象は `b34b594d002240cbe027b017c55a34175213efdf` である。
定義・仮定・命題の番号は同ディレクトリの `README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P3 (`origin` の健全性 -- `Exactly`) | 証明した (第 6 節の系 1)。参照の同一は README の P3 と同じくスロット `(x, λ)` が D8 の意味の参照を持つ場合に限り、オブジェクトの同一はその条件なしに立つ |
| P4 (`origin` の健全性 -- `Join`) | 証明した (第 6 節の系 2)。同じ 2 つの節が付く |

P3 と P4 は、1 つの補題 Q (第 6 節) の 2 通りの読みである。Q は `origin` が辿る別名の辺を 1 本ずつ D9 の
移動の表と A3 の宣言に突き合わせる帰納法で示す。

- 第 1 節が DEF-0 -- `RcVar` が実行路の 1 つの位置で持つ値を 3 つの場合に分けたもの。D6 のスロットが
  在るのはそのうち 2 つの場合である。節点が束縛する変数について、その束縛の D2 のスコープの根の節点を
  **授与位置**と呼び、束縛の 4 つの形ごとに名指す。
- 第 2 節が L6 (`origin_inner` の別名の辺と、各辺が読む D9 の移動の行)、第 2.1 節が L7 (複数元の宣言は
  このプログラムに無いこと)、第 2.2 節が L8 (`Llvm` の腕が答えるもの)。
- 第 3 節が L9 (`origin_inner` が `Exactly((var, path))` を答える道と、その道が D10 の生成の表のどこに
  当たるか)。
- 第 4 節が L1 から L5、L10 (変数に値を与える構文と、値が束縛の後 変わらないこと)、L11 (別名の辺の
  行き先の `RcVar` も値を持つこと)、L12 (値の leaf が参照を持つのは計数下のオブジェクトを指すときで
  あること)、L13 (束縛を持たない名前の値はグローバル状態のオブジェクトだけを指すこと)、L14 (`origin` の
  再帰呼び出しの鍵の関係が整礎であること)、L15 (`origin` の答えは鍵ごとに 1 つに決まり、それは
  `origin_inner` が答えた値であること)。
- 第 5 節が DEF-1 -- D17 の「対応するスロット」を、Q の帰納法が辿る鎖の形に書き直したもの。
- 第 6 節が補題 Q と、その 2 つの系。

第 7 節に、この 2 つの命題の外にある観察を 1 つ置く -- `origin` は 1 つの値の unit の path と leaf の
path に別の答えを与え、leaf の側の `identity` が unit の側の答えに現れないことがある。第 8 節は
`level_ownership` がこの 2 つの命題の真偽を動かさないことである。

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`VarPath` を `(x, π)` と書く。
`ty(x)` は `x` に束縛された値の型 (D6) である。`x` がこの関数の束縛変数であるとき、`vars.var_tys` が
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

**鍵の範囲。** この文書が `origin` に問う鍵はすべて P2 の範囲にある。問う相手は本体に現れる `RcVar` で
あり、その名前は `vars.bindings` に束縛を持つ (そのときプログラムの束縛変数である) か、持たないかの
どちらかである (L10 (a'))。P2 はその 2 種の `x` について、`π` を問わず `origin(x, π)` が panic せずに答えを
返して停止することを述べる。**よって、この文書が扱う `origin` の呼び出しの中で走る `assert!` はどれも
発火しない。** L2 (c) と L9 の `<2>3` の `<3>3` がこれを読む。

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
  辿り、その後 `k` へ進むからである。**アームの中の位置は `k` の部分木に無いので、そこでは `v` はまだ
  値を得ておらず、D6 の「`P` における `v` の値」も定まらない。**

  **`P` 自身がスコープに在ることは要求しない。** D6 が `x` を「その時点で束縛されている変数」に
  限らないと述べ、`Let(m, Match(s, [Let(a, App(f, []), Ret(a))]), k)` の `a` を `k` の位置で名指す例を
  挙げているのがこの広さである。条件は、授与位置を `ρ` が `P` までに通ったかどうかだけである。
- **(v-2)** `v` がその本体の関数のパラメータか capture であること。その値は活性化の入力の束縛 (D23) が
  与える値である。D2 よりパラメータと capture のスコープは本体の全体なので、`ρ` のどの位置でもこれに
  当たる。よってこの場合も D6 の「その時点までに値を得た変数」である。本体がグローバル初期化子の
  `init` であるときこの場合は空である (D1 より `init` はパラメータも capture も持たない)。
- **(v-3)** `v` の名前が `vars.bindings` に束縛を持たず、かつ `ρ` が `v` を名指す節点を `P` までに
  (`P` 自身を含めて) 通っていること。その名前は最上位の記号の名前であり
  (D6 の「**束縛を持たない名前は、必ず最上位の記号の名前である。**」)、その値はその記号の値である。
  A12 の「束縛を持たない `RcVar` の型が、その名前の記号の型であること」がその型を与える。

  **節点を通ったことを条件に置くのは、記憶域を通る記号の値が在るのがそれ以後だからである。** その節点の
  実行がその記号を読み、まだ初期化されていなければ (E7) の段がその初期化子の活性化を作る (D24 の (E7))。
  初期化子が返る前にその記憶域が持つものは、その記号の値ではない。L13 の `<1>4` がこの条件を読み、
  L11 の `<1>1a` がこの条件を満たす形で行き先の `RcVar` を扱う。

`v` が (v-1) か (v-2) であり `λ` がその値の `P` で inhabited な boxed leaf であるとき、`(v, λ)` は
D6 のスロットであり、`obj(v, λ)` はその leaf が指すオブジェクトである。**(v-3) の名前に D6 はスロットを
与えない** -- 節点も入力の束縛もその名前に値を与えないからである。D6 はその対を**記号の位置**と呼ぶ。
L13 が、(v-3) の値の leaf は D8 の意味の参照を持たないことを述べ、補題 Q の (iii) はその形で (v-3) を
排除する。

この 3 つは尽きており、互いに排他である。`VarTable::of` は関数の各パラメータと capture に
`Binding::Param` を入れ、`VarTable::body_only` はそれを持たず、どちらも `collect_bindings` を呼んで
節点が束縛する変数にだけ `Binding` を入れる (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`,
`collect_bindings`)。L10 (a') がこれを述べる。

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

## 2. L6 -- D9 の「移動」と `origin_inner` の別名の辺

**別名の辺**とは、`origin_inner` が `origin` を再帰呼び出しする先をいう
(`CODE src/rc_ir/ownership.rs: origin_inner`)。

**L6 (`origin_inner` の別名の辺)**: 別名の辺は次の第 1 の表の E1 から E7 で尽きている。各辺が辿る先へ
参照と値を渡す構文を述べる D9 の移動の表の行は、第 2 の表のとおりである。**E3 と E4 はどちらも
D9 の `Llvm` の素通し leaf の行の下にある** -- E3 は問うた path 自身が単一の `Arg` を宣言された
leaf である場合、E4 はその path の下の leaf が単一の `Arg` を宣言されている場合であり、どちらもその行が
述べる素通しである。**E4 の側で宣言が単一であることは L7 が与える** -- コードは宣言の集合の各元を辿るので、
要素数 2 以上の宣言を持つ leaf も辿りうる形をしている。

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
| E4 | `Llvm` の素通し leaf (同じ行を leaf ごとに読む。宣言が単一であることは L7 が与える) |
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

<1>2. 別名の辺は第 1 の表の E1 から E7 で尽きている。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner -- `origin` を再帰呼び出しするのは `Move` の腕、
     `Join` の腕、`Llvm` の腕の 2 つの枝、`Field` の `else` の枝、`Payload` の `None` の枝と
     `Some(tag) if !scrut.ty.is_box(type_env)` の枝である。残る枝 (`None | Param | Producer` の腕、
     `Field` の `is_box` の枝、`Payload` の `Some(_)` の枝) は `here()` を返して再帰呼び出しをしない。
     `Llvm` の腕の 2 つの枝が E3 と E4 であり、E4 の中身は `origin_from_leaves_under` である
     (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

<1>2c. 第 2 の表の 7 行はそれぞれ正しい。
  BY D9 の移動の表, D10 の生成の表, L7, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `RcRhs::Var(y)` に `Binding::Move(y)` を作るので、
     `Binding::Move(y)` を持つ変数を束縛する構文は `Let(x, Var(y), k)` であり、E1 が辿る先はその `y` で
     ある。`RcRhs::Match` の腕に `Binding::Join(arm_results)` を作り、`arm_results` は各アーム
     本体の `returned_var` である (`CODE src/rc_ir/ownership.rs: returned_var` -- 本体の終端の `Ret` が
     名指す変数) ので、E2 が辿る先はアーム本体の `Ret` が名指す変数である。
     `RcExpr::Destructure` の腕は名前付きフィールドにだけ `Binding::Field` を作るので、E5 が辿る先は
     unbox 容器のその名前付きフィールドである。`Match` の各アームの payload に
     `Binding::Payload(scrut, arm.tag)` を作り、`tag` が `None` のアームが catch-all、`Some(t)` の
     アームが変位アームである (D2 の `MatchArm` の `tag`) ので、E6 が辿る先は catch-all アームの
     scrutinee、E7 が辿る先は変位アームの scrutinee の変位 `t` である。
     E3 と E4 は `Binding::Llvm` の腕の 2 つの枝である。E3 は `as_arg_projection` が問うた path 自身の
     宣言について集合の要素数 1 と `Arg` を要求する枝であり
     (`CODE src/rc_ir/ownership.rs: as_arg_projection`)、その leaf は D9 の行が言う「単一の
     `Arg(i, σ)`」を宣言された素通し leaf である。E4 は `origin_from_leaves_under` が path の下の各 leaf の
     宣言の**各**元を辿る枝であり (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under` --
     `for sources in decl.leaf_origins_under(path)` の内側の `for src in sources` が、集合の元 1 つずつに
     ついて `operand_units` を積む)、単一でない宣言も辿りうる形をしている。L7 より、このプログラムに
     現れるどの宣言も要素数は 0 か 1 であり、`Arg(j, σ')` を含む leaf の宣言はその 1 元だけからなる。
     よって E4 が辿る leaf も D9 の行が言う「単一の `Arg(i, σ)`」を宣言された素通し leaf である。
     **L7 が要るのは、要素数 2 以上の宣言を持つ leaf が D9 の移動の行ではなく D10 の生成の `Llvm` の行
     (「`result_prov` の宣言が単一の `Arg(j, σ)` **でない**もの」) に当たるからである** -- L7 が無いと
     E4 の辺は D9 の移動の表に行を持たない。**L7 の証明は L6 に依らず、A3 とコードだけに立つ**
     (第 2.1 節) ので、ここで引いても循環しない。

<1>3. QED
  BY <1>1, <1>2, <1>2c

E4 を leaf ごとに分解した形が、第 5 節の DEF-1 の段 E4a と停止条件 S2 である。その各段が D9 と A3 の
どの行に当たるかは第 6 節が述べる。E4 が答えを作る規則そのものの性質は L3 と L4 に置く。

### 2.1 L7 -- 複数元の宣言は現在のプログラムに存在しない

**L7 (宣言の要素数は 1 以下である)**: このコミットのプログラムに現れるどの `LLVMGen` についても、
`result_prov` が結果の各 leaf に置く `LeafOrigins` の要素数は 0 か 1 である。

この事実は第 5 節の DEF-1 と第 6 節の補題 Q が使う。leaf ごとの宣言の要素数が 1 以下でなければ、leaf の
辿る先が 1 つに決まらず、DEF-1 の鎖が定義できない。README の A3 の本文が同じ数え上げを持つ。

<1>1. `LLVMGen` の実装は 78 個あり、そのうち 29 個が `result_prov` を override し、49 個は既定を使う。
      どの呼び出しも abort せず `Provenance` を返す (A3)。
  BY A3, CODE src/fixstd/builtin.rs の `impl LLVMGen for` (78 個、すべてこのファイルにある),
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov (既定、このファイルにある 1 個)

<1>2. 既定は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` であり、各 leaf に単一の
      `Unknown` を置く。
  BY CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
     CODE src/rc_ir/provenance.rs: Provenance::uniform, sole_origin

<1>3. 29 個の override が leaf に置く集合は、`sole_origin(..)` (単一)、`Set::default()` (空)、
      `Provenance::uniform` (単一)、`Provenance::uniform_bottom` (空)、`Provenance::fresh_under` (単一) の
      いずれかで作られる。
  BY CODE src/fixstd/builtin.rs の 29 個の `result_prov` の本体、および
     CODE src/fixstd/builtin.rs: replaced_field_prov (2 個の op が共有する),
     CODE src/rc_ir/provenance.rs: Provenance::uniform, uniform_bottom, fresh_under, sole_origin

<1>4. QED
  BY <1>1, <1>2, <1>3 -- どの宣言も要素数 0 か 1 の集合しか持たない。複数元の集合を作るのは
     `Provenance::join` (アームの合流) と `Provenance::compose` (呼び出し先の効果の代入) だけであり
     (`CODE src/rc_ir/provenance.rs: Provenance::join`, `Provenance::compose`)、どちらも解析の側である。
     `origin_inner` が読むのは `llvm_gen.result_prov(..)` の返り値そのもの、すなわち宣言である。

### 2.2 L8 -- `Llvm` の腕が答えるもの

A3 は `result_prov` が leaf ごとに `LeafOrigins` (`Set<LeafOrigin>`) を返すとし、空集合・単一の
`Arg`・単一の `Fresh`・単一の `Unknown`・複数元の 5 行を持つ。`origin_inner` の `Llvm` の腕がその 5 つを
どう扱うかを書き下す。

**L8 (`Llvm` の腕が答えるもの)**: `x` の `Binding` が `Llvm(gen, args, ty(x))` であるとし、
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

<1>1. `decl` が記録する leaf の集合は `leaves(ty(x))` そのものであり、`decl.leaf_origins_at(p)` は
      `p ∈ leaves(ty(x))` のときその leaf の `LeafOrigins` を `Some` で返し、そうでないとき `None` を
      返す。
  <2>1. `Provenance` は `LeafMap<LeafOrigins>` の newtype であり、`LeafMap::build_shape(τ, type_env, f)`
        が作る `LeafMap` の鍵の集合は `boxed_leaf_paths(τ, type_env)` そのものである。
    BY CODE src/rc_ir/provenance.rs: Provenance (`Provenance(LeafMap<LeafOrigins>)`),
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape -- `boxed_leaf_paths(ty, type_env)` の各要素を
       鍵にして `collect` する
  <2>2. `result_prov` の呼び出しは値を返し (A3)、返す `Provenance` は、既定と 29 個の override の
        いずれについても、`Provenance::uniform`、`Provenance::build_shape`、
        `Provenance::uniform_bottom`、`Provenance::fresh_under`、`replaced_field_prov` のいずれかを
        `result_ty` に対して呼んだ値である。
    BY A3 (`result_prov` の呼び出しは abort せず `Provenance` を返す),
       CODE src/ast/inline_llvm.rs: LLVMGen::result_prov,
       CODE src/fixstd/builtin.rs の 29 個の `result_prov` の本体,
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
    BY <1>1, <2>1 -- `None` と `Some` を分け、`Some` の中身の集合を要素数 0、1、2 以上で分け、
       要素数 1 を構成子で分けた。

<1>3. `as_arg_projection(sources)` が `Some` を返すのは L8 (b) の第 3 の場合だけである。
  BY CODE src/rc_ir/ownership.rs: as_arg_projection -- `sources.len() != 1` で `None`、要素が `Fresh` か
     `Unknown` でも `None`。

<1>4. 第 3 の場合、鍵 `(x, π)` の答えは鍵 `(args[j], σ)` の答えである (辺 E3)。これは D9 の移動の表の
      `Llvm` の行と A3 の「単一の `Arg(j, σ)`」の行に一致する。
  BY <1>3, L15 (鍵の答えは 1 つに決まり、それは `origin_inner` がその鍵について答えた値である),
     CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `Some((j, σ))` の枝
     -- この枝は `origin(vars, type_env, &args[j].name, &p)` の返り値をそのまま返すので、
     `origin_inner(x, π)` が答える値は鍵 `(args[j], σ)` の答えである,
     D9 の移動の表, A3

<1>5. 残る 4 つの場合は `origin_from_leaves_under(vars, type_env, &decl, args, π, &(x, π))` に入り、
      それが `None` を返すとき、鍵 `(x, π)` の答えは `Exactly((x, π))` である。
  BY <1>3, L15 (鍵の答えは 1 つに決まり、それは `origin_inner` がその鍵について答えた値である),
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
        `operand_units` の各要素 `(j, unit)` について鍵 `(args[j], unit)` の答えを並べたものである。
    BY <2>1, L15 (鍵の答えは 1 つに決まり、それは `origin_inner` がその鍵について答えた値である),
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
する。要素数 2 以上の宣言は L7 よりこのプログラムに無い。

## 3. L9 -- D10 の「生成」と `here()` の腕

`here()` は `Origin::Exactly((var, path))` を返す閉包である
(`CODE src/rc_ir/ownership.rs: origin_inner` の先頭)。

**L9 (`here()` の答えに着く道)**: `origin_inner(vars, type_env, var, path)` が
`Origin::Exactly((var, path))` に着く道は次の表の H1 から H7 である。H1 から H5 では `match` の腕が
`here()` をそのまま返し、この 5 つは `here()` をそのまま返す枝の全体である。H6 と H7 では `Llvm` の腕が
`origin_from_leaves_under` を通ってその値に着く。

| 道 | 着き方 | D10 での位置 |
|---|---|---|
| H1 `None` (表に無い名前) | 直接 | A8 (グローバルは線形規律の外) |
| H2 `Param` | 直接 | D10 の初期値 |
| H3 `Producer` | 直接 | 生成の表の `App` の行と `Closure` の行 |
| H4 `Field(c, i)` かつ `c` が boxed | 直接 | 生成の表の boxed 容器の `Destructure` の行 |
| H5 `Payload(s, Some(t))` かつ `s` が boxed | 直接 | 生成の表の boxed union の変位アームの行 |
| H6 `Llvm` かつ `π` の下のある leaf の宣言が `Fresh` か `Unknown` を含み、`reached` の全要素が等しい | `origin_from_leaves_under` が `Exactly(here)` を `reached` に積み、それが答えになる | 生成の表の `Llvm` の行 |
| H7 `Llvm` かつ `π` の下の leaf の宣言がすべて空集合 | `origin_from_leaves_under` が `None` を返し `unwrap_or_else(here)` | 生成の表の `Llvm` の行が覆う。ただし A3 と D16 よりその leaf は inhabited にならないので参照は生じない |

<1>1. `origin_inner` の `match` の腕のうち `here()` をそのまま返す枝は、H1 から H5 の 5 つで尽きている。
  BY L6 (別名の辺は E1 から E7 で尽きている), CODE src/rc_ir/ownership.rs: origin_inner -- 6 本の腕の
     うち `here()` をそのまま返すのは、
     `None | Some(Binding::Param) | Some(Binding::Producer)` の腕 (H1、H2、H3)、
     `Some(Binding::Field(container, idx))` の `container.ty.is_box(type_env)` の枝 (H4)、
     `Some(Binding::Payload(..))` の `Some(_)` の枝 (H5)。ほかの枝は `origin` を再帰呼び出しするか
     `origin_from_leaves_under` に入る。

<1>2. `Llvm` の腕が `Exactly((var, path))` に着くのは H6 と H7 の 2 つである。
  <2>1. `Llvm` の腕が答えを作る道は 4 つある -- `as_arg_projection` が `Some((j, p))` を返す枝の
        `origin(args[j], p)`、`origin_from_leaves_under` の `reached` の全要素が等しい枝が返す
        `first.clone()`、その `else` に当たる末尾の `of_candidates(candidates, here)`、そして
        `origin_from_leaves_under` が `None` を返したときの `unwrap_or_else(here)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>2. 第 4 の道が `Exactly((var, path))` を返すのが H7 である。
    BY <2>1, L8 (d2), CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- `reached` が空のとき
       `reached.first()?` が `None` を返し、`unwrap_or_else(here)` が `Exactly((var, path))` を答える。
       `reached` が空であることと `π` の下の leaf の宣言がすべて空集合であることは同値である
       (L8 (d1) と L8 (d3) がその 2 つの向きを与える)。
  <2>3. 末尾の `of_candidates(candidates, here)` は `Exactly` を返さない。
    <3>1. この道に入るのは `reached` の全要素が等しくないときであり、そのとき `reached` は相異なる
          2 つの `Origin` `o_1 ≠ o_2` を含む。
      BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`if reached.iter().all(..)` が偽である
         枝)
    <3>2. `candidates ⊇ act(o_1) ∪ act(o_2)` である。
      BY <3>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`flat_map(|reached_origin|
         reached_origin.acted_on())`)
    <3>3. `|candidates| ≥ 2` である。
      BY <3>1, <3>2, L2 (b), L2 (c), 第 1 節の「鍵の範囲」 -- `o_1` と `o_2` は
         `origin_from_leaves_under` が `reached` に積んだ値であり、この `origin_inner` の実行の中で
         作られたものである。第 1 節の「鍵の範囲」より、それを含む `origin` の呼び出しは panic せずに
         返るので、L2 (c) の前提が満たされる。`|candidates| = 1` とすると、`act(o_1)` と `act(o_2)` は
         どちらもその 1 元集合である。L2 (c) より `Join` の `act` は 2 元以上なので `o_1` と `o_2` は
         どちらも `Exactly` であり、L2 (b) よりその `act` は自分の `VarPath` の 1 元集合なので
         `o_1 = o_2` となって <3>1 に反する。
    <3>4. QED
      BY <3>3, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- 要素数が 1 でない集合には
         `Join` を返す。
  <2>3a. `origin(v, q)` が返す `Origin` に現れる `VarPath` は、鍵 `(v, q)` から**再帰の辺**を 0 回以上
         辿って着く鍵である。ここで再帰の辺とは、L6 の E1 から E7 が名指す `origin` の再帰呼び出しの
         鍵への辺をいう。鍵についての言明にできるのは、`origin` の答えが鍵ごとに 1 つに決まり、それが
         その鍵について `origin_inner` が答えた値だからである (L15)。
    <3>1. `Origin` の値を作る式は 3 つある -- `origin_inner` の `here()`、`origin_from_leaves_under` の
          `Origin::Exactly(here.clone())`、そして `Origin::of_candidates` である。
      BY L1 (`Origin::Join { .. }` を作る式は `of_candidates` の中の 1 か所だけであり、どの `Origin` の
         値も `Exactly` か `of_candidates` が作った `Join` かその複製である),
         CODE src/rc_ir/ownership.rs: origin_inner (`here` は
         `Origin::Exactly((var.clone(), path.to_vec()))` を返す閉包である),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
         (`reached.push(Origin::Exactly(here.clone()))`),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates (`1 =>` の腕が `Origin::Exactly` を、
         `_ =>` の腕が `Origin::Join` を作る)
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
    <3>1c. `of_candidates` を呼ぶのは `ownership.rs` の 2 か所だけであり、どちらでも `h` はその
           呼び出し自身の `(var, path)`
           であり、`C` の各元は、その呼び出しが畳み込む `Origin` のいずれかに現れる `VarPath` である。
      BY L2 (a) (`acted_on()` の元は `identity` か `candidates` の元である), L4,
         L1 (`Origin` という識別子は `ownership.rs` の外に現れない),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates (`fn` に `pub` が付かないので、この関数を
         呼べるのは `ownership.rs` の中だけである),
         CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕
         (`Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` であり、`candidates` は
         各アーム結果の `origin(..).acted_on()` の元を集めたものである),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under (末尾の
         `Origin::of_candidates(candidates, here)` であり、`candidates` は `reached` の各元の
         `acted_on()` を集めたものである), CODE src/rc_ir/ownership.rs: Origin::acted_on
    <3>2. 1 つの鍵の答え -- その鍵について `origin_inner` が返した値 (L15) -- は、次の 4 つのいずれかで
          ある。(r1) `here()` -- <1>1 の 5 つの枝と、
          `origin_from_leaves_under` が `None` を返したときの `unwrap_or_else(here)`。(r2) 子の呼び出し
          `origin(..)` が返した値そのもの -- `Move` の腕、`Field` の `else` の枝、`Payload` の 2 つの枝、
          `Llvm` の腕の `as_arg_projection` が `Some` を返す枝。(r3) `origin_from_leaves_under` が
          `reached` の全要素が等しいときに返す `first.clone()`。`reached` の各元は、子の呼び出しが
          返した値か `Origin::Exactly(here.clone())` である。(r4) `of_candidates(C, h)` の値 --
          `Binding::Join` の腕と `origin_from_leaves_under` の末尾。
      BY <1>1, <2>1, L6, L15, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
    <3>3. QED
      BY <3>1, <3>1a, <3>1b, <3>1c, <3>2, L14, L15 -- 鍵の再帰の辺の関係は整礎である (L14)。その関係の
         上の整礎帰納で示す。子の呼び出しが返した値はその子の鍵の答えである (L15)。
         (r1) が返す値の `VarPath` はその呼び出しの鍵そのもの (<3>1a)。
         (r2) が返す値には帰納法の仮定が当たり、その子の鍵はこの鍵から辺 1 本で着く (L6)。(r3) が
         返すのは、子の呼び出しが返した値の複製か `Origin::Exactly(here.clone())` の複製であり、前者には
         帰納法の仮定が当たり、後者の `VarPath` はこの呼び出しの鍵である (<3>1a)。複製は `VarPath` を
         変えない (L1)。(r4) が返す値の `VarPath` は `h` か `C` の元であり (<3>1b)、`h` はこの呼び出しの
         鍵、`C` の元は畳み込む `Origin` -- すなわち子の呼び出しが返した値か
         `Origin::Exactly(here.clone())` -- に現れる `VarPath` である (<3>1c)。
  <2>4. `Llvm` の腕がオペランドについて行う再帰呼び出し -- `as_arg_projection` が `Some((j, p))` を
        返す枝の `origin(args[j], p)` と、`origin_from_leaves_under` が `reached` を作るときの
        `origin(args[j], unit)` -- は、どれも `Exactly((var, path))` を返さない。
    <3>1. そのような呼び出しが `Exactly((var, path))` を返すならば、その呼び出しの鍵から再帰の辺を
          0 回以上辿って鍵 `(var, path)` に着く。
      BY <2>3a
    <3>2. 鍵 `(var, path)` からその呼び出しの鍵へは、再帰の辺が 1 本ある。
      BY L6 (E3 と E4 は `Binding::Llvm` の腕の再帰呼び出しである),
         CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. QED
      BY <3>1, <3>2, L14 -- 2 つを繋ぐと、鍵 `(var, path)` から再帰の辺を 1 回以上辿って
         `(var, path)` 自身に着く。すなわち鍵の辺の上に閉路がある。その閉路を繰り返せば無限に降りる
         鍵の列ができるので、L14 に反する。
  <2>4a. 第 2 の道が `Exactly((var, path))` を返すのが H6 である。
    BY <2>1, <2>4, L8 (d1), CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- この道が返すのは
       `reached` の全要素が等しいときの `first.clone()` であり、`reached` の元は
       `origin(args[j], unit)` の値と、`produced_here` が真のときに積まれる `Exactly(here)` である。
       <2>4 より前者は `Exactly((var, path))` ではないので、`first` が `Exactly((var, path))` である
       ためには `produced_here` が真、すなわち `π` の下のある leaf の宣言が `Fresh` か `Unknown` を
       含むことが要る。逆にそのとき L8 (d1) より `Exactly((var, path))` は `reached` の元であり、
       全要素が等しければそれが答えである。
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>4, <2>4a

<1>3. D10 の生成の 5 行はすべて `here()` の道を持つ -- H3 が `App` の行と `Closure` の行、H4 が boxed
      容器の `Destructure` の行、H5 が boxed union の変位アームの行、H6 が `Llvm` の行である。
  BY D10 の生成の表, <1>1, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Binding::Producer` を作るのは `RcRhs::App` と
     `RcRhs::Closure` の 2 つだけであり、`Binding::Field` は `RcExpr::Destructure` の名前付き
     フィールド、`Binding::Payload(scrut, Some(t))` は変位アームの payload である。boxed か unbox かは
     `is_box` の枝が分ける。

<1>4. H1、H2、H7 のどれも新しい参照を作らない。
  <2>1. H1 と H2 は D10 の生成の表に行を持たない。
    BY D10 の生成の表 (5 行はいずれも `Llvm`、`App`、`Closure`、boxed 容器の `Destructure`、boxed union の
       変位アームであり、束縛を持たない名前についての行もパラメータ・capture についての行も無い),
       A8 (H1: グローバル値が到達するオブジェクトは線形規律の外にある),
       D10 の初期値 (H2: パラメータと capture の参照はそこに置かれるのであって、生成されるのではない)
  <2>2. H7 の leaf は D10 の生成の `Llvm` の行が覆うが、その leaf は inhabited にならないので参照は
        生じない。
    BY D10 の生成の表の `Llvm` の行 (「宣言が空集合 (bottom) のとき、`Fresh` や `Unknown` を含むとき、
       複数の元を持つときのすべてを含む。空集合と宣言された leaf は inhabited にならないので、参照は
       生じない (A3)」), A3 (空集合の行), D16
  <2>3. QED
    BY <2>1, <2>2

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

## 4. 補題

以下の補題は、この文書のすべての証明が使う。**この節の補題はどれも L6 から L9 に依らない** -- 依拠するのは
README の定義・仮定とコードだけである。よって第 2 節と第 3 節の証明もこれらを引く。

**L1 (`Origin::Join` は `of_candidates` だけが作る)**: `Origin::Join { .. }` を値として作る式は
`Origin::of_candidates` の中の 1 か所だけである。よって、どの `Origin` の値も、`Exactly` であるか、
`of_candidates` が作った `Join` (あるいはその複製) である。

<1>1. 識別子 `Origin` は、`src/` の中で `src/rc_ir/ownership.rs` の外に現れない。すなわち
      `Origin::Join { .. }` という式を書きうるのはこのファイルの中だけである。`ProjectOrigin` と
      `LeafOrigin` は別の識別子である。
  BY CODE src/rc_ir/ownership.rs: Origin -- `src/` 全体を `Origin` で検索し、`ProjectOrigin` と
     `LeafOrigin` を除くと、当たるのは `src/rc_ir/ownership.rs` の行だけである。
<1>2. 完全修飾の道 (`crate::rc_ir::ownership::Origin::Join { .. }`) も現れない。
  BY <1>1 -- その道も識別子 `Origin` を含むので、<1>1 の検索に当たる。
<1>3. `ownership.rs` の中で `Origin::Join` と書かれているのは 3 か所であり、`Origin::identity` と
      `Origin::candidates` の 2 つはパターン、`Origin::of_candidates` の 1 つが構成である。
  BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates, Origin::of_candidates
<1>4. `Origin` は `Clone` を導出するので、`Join` の値は複製によっても現れる。複製は `identity` と
      `candidates` をそのまま運ぶ。
  BY CODE src/rc_ir/ownership.rs: Origin (`#[derive(Clone, Debug, PartialEq, Eq)]`)
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

**L2 (`identity`、`candidates`、`acted_on` の関係)**: 任意の `Origin` の値 `o` について次が成り立つ。

- **(a)** `act(o) = {id(o)} ∪ cand(o)`。とくに `act(o) ⊇ cand(o)` であり `id(o) ∈ act(o)` である。
- **(b)** `o = Exactly(p)` ならば `id(o) = p` かつ `cand(o) = act(o) = {p}` である。
- **(c)** `o` が `Join` であり、かつ `o` が、panic せずに返る `origin` の呼び出しの中で作られた値で
  あるならば、`|cand(o)| ≥ 2` であり、よって `|act(o)| ≥ 2` である。この前提を第 1 節の「鍵の範囲」が
  この文書のすべての `Origin` について与える。

<1>1. (a)。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on -- `identity` を先頭に置き、`candidates` から
     `identity` に等しいものを除いたものを続ける。

<1>2. (b)。
  BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates, Origin::acted_on

<1>3. (c)。
  <2>1. `Join` の値を作る式は `Origin::of_candidates` の中の 1 か所だけであり、その枝は
        `candidates.len()` が 1 でないときに走る。複製は `candidates` をそのまま運ぶ。
    BY L1, CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>2. `of_candidates` はその枝へ進む前に `assert!(!candidates.is_empty(), ..)` を評価する。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>3. その `assert!` は発火せず、`candidates` は空でない。
    BY <2>2, (c) の前提 (`o` は panic せずに返る `origin` の呼び出しの中で作られた値である) -- 表明が
       発火すればその呼び出しは panic して返らないので、`o` が値として在ることと相容れない
  <2>4. QED
    BY <2>1, <2>3, <1>1 -- `|cand(o)| ≠ 1` かつ `cand(o)` が空でないので `|cand(o)| ≥ 2` であり、
       <1>1 の `act(o) ⊇ cand(o)` より `|act(o)| ≥ 2` である。

<1>4. QED
  BY <1>1, <1>2, <1>3

**L3 (`of_candidates` の `acted_on` は与えた集合を含む)**: 空でない集合 `C` と `h` について
`act(of_candidates(C, h)) ⊇ C`。

<1>1. `|C| = 1` のとき `of_candidates(C, h) = Exactly(c)` (`C = {c}`) であり、`act = {c} = C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, L2 (b)
<1>2. `|C| ≥ 2` のとき `of_candidates(C, h) = Join { identity: h, candidates: C }` であり、
      `act = {h} ∪ C ⊇ C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, L2 (a)
<1>3. QED
  BY <1>1, <1>2

**L4 (畳み込みの答えの形)**: `origin_inner` の `Binding::Join` の腕と `origin_from_leaves_under` が
畳み込む先の `Origin` を `o_1, ..., o_k` (`k ≥ 1`) とし、答えを `o` とする。

- **(a)** `Binding::Join` の腕では `o = of_candidates(∪_i act(o_i), (var, path))` である。
- **(b)** `origin_from_leaves_under` では、`reached` の全要素が等しいときは `o` はその要素そのもので
  あり、そうでないときは `o = of_candidates(∪_i act(o_i), here)` である。
- **(c)** どちらの場合も `act(o) ⊇ act(o_1) ∪ ... ∪ act(o_k)` である。

<1>1. (a)。
  BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕 -- ループは
     `origin(..).acted_on()` の各要素を `candidates` に入れる。
<1>2. (b)。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `if reached.iter().all(..)` の枝と
     その後の `flat_map(|reached_origin| reached_origin.acted_on())`
<1>3. (a) と (b) の後者の場合、`act(o) ⊇ ∪_i act(o_i)`。
  BY <1>1, <1>2, L3
<1>4. (b) の前者の場合、`o = o_1 = ... = o_k` なので `act(o) = ∪_i act(o_i)`。
  BY <1>2
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4 -- (c) はどちらの場合も成り立つ。

**L5 (leaf は互いに比較不能である)**: 型 `τ` の相異なる 2 つの boxed leaf の一方が他方の接頭辞になることは
無い。

<1>1. `boxed_leaf_paths` の走査は、leaf を積んだ位置の下へ降りない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- 走査 `go` が `out.push` を行う 3 つの枝
     (`is_closure`、`is_box`、`is_array`) は、いずれも `unpunched_field_types` のループへ進まずに
     `return` する。
<1>2. QED
  BY <1>1 -- leaf が積まれる位置の下は走査されないので、leaf の真の延長が leaf になることは無い。

**L10 (変数に値を与える構文と、値が束縛の後 変わらないこと)**

- **(a)** 本体の節点のうち、変数に値を与えるのは 3 つの構文である -- `Let(x, rhs, k)` は `x` に、
  `Destructure(c, fs, s, k)` は各 `(i, x)` の `x` に、`Match` の各アームはその `payload` に値を与える。
  残る 4 種 (`Retain`、`Release`、`Eval`、`Ret`) はどの変数にも値を与えない。**パラメータと capture に
  値を与えるのは節点ではなく、活性化の入力の束縛 (D23) である。**
- **(a')** `vars.bindings` に入る名前は、その関数のパラメータと capture (いずれも `Binding::Param`)、
  および (a) の 3 構文が束縛する変数だけである。すなわち DEF-0 の (v-1) と (v-2) は `vars.bindings` が
  束縛を持つ名前、(v-3) はそれ以外の名前である。
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

<1>1. (a)。
  BY D2 (節点の 6 種の表 -- `Let` は `rhs` の値を `x` に束縛し、`Destructure` は各 `(i, x)` の `x` に
     第 `i` フィールドを束縛し、`Retain` は参照を作り、`Release` は参照を処分し、`Eval` は評価して
     捨て、`Ret` はその式の値を述べる), D9 の移動の表の値の水準の 6 行 (「unbox union の変位アームの
     payload 束縛: payload 変数の値は scrutinee の値の活性変位の payload である」と「catch-all アームの
     payload 束縛: payload 変数の値は scrutinee の値そのものである」の 2 行が、アームの `payload` が
     値を得る束縛であることを述べる), D2 の束縛の及ぶ範囲の段落 (パラメータと capture のスコープは
     本体の全体である), D23 (活性化の入力の束縛が各パラメータと capture に 1 つずつの値を与える)
<1>1a. (a')。
  BY CODE src/rc_ir/ownership.rs: VarTable::of (関数の本体について、`func.params` と `func.capture` の
     各 `p` に `Binding::Param` を入れ、続けて `collect_bindings` を呼ぶ),
     CODE src/rc_ir/ownership.rs: VarTable::body_only (グローバル初期化子の本体について、`collect_bindings` だけを
     呼ぶ。D1 より `init` はパラメータも capture も持たない),
     CODE src/rc_ir/ownership.rs: collect_bindings -- 変数に `Binding` と型を入れるのは `RcExpr::Let` の
     腕、`RcExpr::Destructure` の腕、`RcRhs::Match` の腕の `arm.payload` の 3 か所だけであり、
     `RcExpr::Retain`、`RcExpr::Release`、`RcExpr::Eval`、`RcExpr::Ret` の腕はどの変数も入れない。
     この 3 か所は (a) の 3 構文である。
<1>1b. (c)。
  <2>1. (c1)。(a) の 3 構文が作る束縛は、`Let(v, rhs, k)` の `v`、`Destructure(c, fs, s, k)` の `fs` の
        各変数、`Match` の各アームの `payload` である。DEF-0 の (v-1) の表は、1 つ目を `rhs` が `Match`
        であるかどうかで 2 行に分け、2 つ目と 3 つ目に 1 行ずつを当てている。
    BY <1>1, DEF-0 の (v-1) の表, D2 (`RcRhs` の 5 種 -- `rhs` が `Match` であるかそうでないかは
       この 5 種を 2 つに分ける)
  <2>2. (c2)。`Let(v, rhs, k)` で `rhs` が `Match` でないとき、`ρ` は `k` の根に進む前にその `Let` の
        節点を通り、`v` は `rhs` の値を持つ。`Destructure(c, fs, s, k)` でも同じく、`ρ` は `k` の根に
        進む前にその `Destructure` の節点を通り、各 `(i, x)` の `x` は容器の第 `i` フィールドの値を持つ。
        `Let(v, Match(s, arms), k)` では、`ρ` は `k` の根に進む前に `α` が選んだアーム本体の実行路を
        辿り終える -- すなわちそのアーム本体の終端の `Ret` を通る -- ので、`v` はその `Ret` が名指す
        変数の値を持つ。`Match` のアーム `A` の `payload` については、`ρ` が `A` の `body` の根の節点を
        通るのは `α` が `A` を選んだときに限り、そのとき `A` の payload 束縛が `v` に値を与える。
    BY D2 (節点の 6 種の表 -- `Let` は `rhs` の値を `x` に束縛し、`Destructure` は各 `(i, x)` の `x` に
       第 `i` フィールドを束縛する。および `MatchArm` の `payload` の欄), D3 (`Let(x, Match(v, arms), k)`
       ではアームを 1 つ選び、そのアーム本体の実行路を辿り、その後 `k` へ進む。アーム本体の `Ret` は
       そのアーム本体の実行路を終える), D21 (`α` が選ぶアームは決まっている), <1>1,
       D9 の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の束縛変数の値は `x` の値で
       ある。」)
  <2>3. (c3)。授与位置はそのスコープの根の節点であり (DEF-0 の (v-1))、`ρ` が部分木の節点を通るには
        その部分木の根の節点を先に通る。
    BY DEF-0 の (v-1) (授与位置はスコープの根の節点である),
       D2 (束縛の及ぶ範囲の段落 -- `Let` と `Destructure` が束縛する変数のスコープは `k` の部分木、
       `Match` のアームの `payload` のスコープはそのアームの `body` の部分木である),
       D3 (実行路は根から辿るので、部分木の節点を通る前にその根の節点を通る)
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>1c. (d)。
  <2>1. `v` の名前は最上位の記号の名前であり、局所名ではない。
    BY DEF-0 の (v-3) (`v` の名前は `vars.bindings` に束縛を持たない),
       D6 (「**束縛を持たない名前は、必ず最上位の記号の名前である。**」),
       A13 (「**最上位の記号の名前は局所名ではない。**`FullName::is_local` が偽であり、`prog.funcs` の
       鍵と `global_types` の鍵はどちらもそのような名前である」)
  <2>2. 局所でない名前の値は、`declare_program_global` が用意する 2 つのうちの一方である -- 型が
        `is_funptr` なら `declare_lambda_function` が返す LLVM 関数の番地、そうでなければ
        `add_global_object` が登録するグローバルのアクセサが返す値である。
    BY <2>1, CODE src/generator.rs: Generator::get_scoped_value (`var.is_local()` が偽なら
       `get_or_declare_global` へ行く),
       CODE src/generator.rs: Generator::get_or_declare_global (`declare_program_global` を呼ぶ),
       CODE src/generator.rs: Generator::declare_program_global (`ty.is_funptr()` なら
       `declare_lambda_function` を返し、そうでなければアクセサ関数を作って `add_global_object` に
       登録する), CODE src/generator.rs: Generator::add_global_object,
       CODE src/generator.rs: ValueAccessor::get (`is_funptr` の枝は `fun.as_global_value()` を、
       そうでない枝はアクセサの呼び出しの結果を、その名前の値とする)
  <2>3. QED
    BY <2>1, <2>2
<1>2. `ρ` は本体の木の各位置を高々 1 度しか通る。
  BY D2 (分岐は `Match` のアームだけであり、節点が自分自身を含むことはないので、本体は有限の木である),
     D3 (実行路は根から継続へ、アームでは アーム本体を辿ってから `k` へ進む)
<1>3. (v-1) の場合。`v` に値を与える束縛は本体に 1 つであり (A6、<1>1)、`ρ` はその束縛の節点も `v` の
      授与位置も高々 1 度しか通らない (<1>2) ので、その束縛が `v` に与える値は 1 つに定まる。`ρ` 上で
      `v` が値を持つのは授与位置以後の位置に限り (DEF-0 の (v-1))、そのどの位置でも `v` の値はその 1 つで
      ある -- 授与位置で `v` はその値を持ち (<1>1b (c2))、`v` に値を与える構文はほかに無い (<1>1)。
      `N` もその中の 1 つなので、`N` 以後 `v` の値は変わらない。
  BY A6 (束縛変数の名前は相異なる), <1>1, <1>1b, <1>2, DEF-0 の (v-1),
     D6 (「変数の値は、それを束縛する節点の後は変わらない。」)
<1>3a. (v-2) の場合。`v` の値は活性化が始まった時点の入力の束縛が与える 1 つであり、`ρ` のどの節点も
       それを変えない。
  BY D23 (入力の束縛は各パラメータと capture に 1 つずつの値を与える), <1>1 (節点が値を与えるのは
     (a) の 3 構文が束縛する変数だけであり、A6 よりその名前はパラメータ・capture の名前と異なる), A6
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
         CODE src/generator.rs: Generator::get_or_declare_global (`declared_globals` に在ればその
         `ScopedValue` を返し、無ければ `declare_program_global` で用意してからその欄を返す),
         CODE src/generator.rs: Generator::declare_program_global (`ty.is_funptr()` の枝が用意するのは
         その名前の 1 つの関数である),
         CODE src/generator.rs: Generator::add_global_object (`declared_globals` へ入れるのはここだけで
         あり、同じ名前を 2 度入れようとすると `panic_with_msg` で止まる。**表明が発火するならその
         プログラムは走らず、その本体の活性化は存在しない**ので、走る本体では 1 つの名前の欄は 1 つで
         ある)
    <3>3. QED
      BY <3>1, <3>2, D24 -- `v` の値は `v` の名前と、読み込まれた像がその関数に与える番地だけで決まる
         (<3>1、<3>2)。D24 の各段が書くのは参照カウント、状態バイト、オブジェクトの記憶域、および
         グローバルの記憶域 ((E5)、(E7)) であり、関数の番地はそのどれでもない。**この段は「番地は
         リンクが決める定数である」を仮定しない** -- 値を作る式が記憶域を読まないこと (<3>1) と、
         `fun` が名前だけで決まること (<3>2) だけを読む。
  <2>2. そうでないとき、`v` の値は `v` が名指す記号の記憶域が持つ値であり、`v` が値を持つ `ρ` 上の
        どの位置でも同じである。
    <3>1. `v` が `P` で値を持つならば、`ρ` は `v` を名指す節点を `P` までに通っており、その節点の
          実行がその記号を読む。まだ初期化されていなければ (E7) の段が初期化子の活性化を作り、
          その活性化が終わってから読む者が再開する。よって `P` では初期化が済んでいる。
      BY DEF-0 の (v-3), D24 の (E7) (`g` を読む節点の位置にある活性化は、初期化子の活性化 `b` が
         終わるまで中断中であり、`b` が終端の `Ret` に着き (E5) が走ってから再開する)
    <3>2. 初期化の後、その記憶域は書き替えられない。
      BY D22 のグローバルのアクセサの行 (「初期化済みの旗を見て、まだならグローバル初期化子の本体を
         持つ関数 `InitValue#<symbol>` を呼び、返った値を記憶域へ格納する」「以後の読みは記憶域を
         読むだけである」)
    <3>3. QED
      BY <1>1c ((d) のアクセサの枝), <3>1, <3>2
  <2>3. QED
    BY <1>1c, <2>1, <2>2, DEF-0 の (v-3) -- (d) より場合はこの 2 つで尽きている。`v` が `N` で値を
       持つならば、`ρ` は `v` を名指す節点を `N` までに通っているので `N` 以後のどの位置でも `v` は
       値を持ち、その値は `v` が値を持つどの位置でも同じである。
<1>4. QED
  BY <1>1, <1>1a, <1>1b, <1>1c, <1>3, <1>3a, <1>3b -- (a) は <1>1、(a') は <1>1a、(c) は <1>1b、
     (d) は <1>1c が与える。(b) は、DEF-0 の 3 つの場合が尽きており (<1>1a)、どの場合も `N` 以後
     `v` の値が変わらないこと (<1>3、<1>3a、<1>3b) から出る。

**L11 (別名の辺の行き先の `RcVar` も値を持つ)**: 活性化 `α`、それが辿る実行路 `ρ`、`ρ` 上の位置 `P`、
`P` で値を持つ (DEF-0) `RcVar` `x` を取る。`x` の名前が `vars.bindings` に持つ束縛が名指す `RcVar` --
`Move(y)` の `y`、`Payload(s, ・)` の `s`、`Field(c, i)` の `c`、`Llvm(gen, args, ・)` の各 `args[j]`、
`Join(rs)` のうち `α` が選んだアームの本体の終端の `Ret` が名指す変数 `r_0` -- は、いずれも `P` で値を
持つ (DEF-0)。

**この補題は行き先が D6 のスロットを持つとは言わない。** 行き先が (v-3) の名前 -- グローバル値を読む
`RcVar` -- でありうるからである。`Let(x, Var(g), k)` (`g` はグローバル値) の `g` がその形であり、D26 が
「この形は実プログラムにいくらでもある」と述べる。行き先を (v-1) と (v-2) に絞るのは L13 の仕事であり、
補題 Q の (iii) はそこで仮定 (H) を使う。

<1>1. `x` は DEF-0 の (v-1) か (v-2) である。(v-2) のとき `x` の束縛は `Binding::Param` であり、
      どの `RcVar` も名指さないので主張は空虚である。以下 `x` は (v-1) であるとし、`x` の束縛を作る
      節点を `N` とする。`ρ` は `N` を `P` までに通る。
  BY DEF-0 (`x` は束縛を持つので (v-3) ではない。(v-1) より `ρ` は `x` の授与位置を `P` までに通る。
     (v-1) の表の 4 行のいずれでも、授与位置は `N` の部分木の中に在る -- 第 1 行から第 3 行の `k` も、
     第 4 行のアーム `A` の `body` も、`N` が持つ部分木である), D3 (実行路は木を根から辿るので、
     部分木の節点を通る前にその根の節点を通る), L10 (a'), L10 (c1),
     A6 (束縛変数の名前は相異なる),
     CODE src/rc_ir/ownership.rs: Binding (`Param` は `RcVar` の欄を持たない),
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Let` は束縛する変数に `Move` / `Llvm` /
     `Producer` / `Join` を、`Destructure` は各名前付きフィールド変数に `Field` を、`Match` の各アームは
     その `payload` に `Payload` を作る。名前は相異なる (A6) ので、変数と節点の対応は 1 対 1 である。
<1>1a. `ρ` が `P` までに通る節点 `M` に書かれたオペランド `v` は、`P` で値を持つ (DEF-0)。
  <2>1. CASE: `v` の名前が `vars.bindings` に束縛を持たない。
    BY DEF-0 の (v-3) -- `v` は節点 `M` に書かれたオペランドであり、`ρ` は `M` を `P` までに通るので、
       (v-3) の条件 (`ρ` が `v` を名指す節点を `P` までに通っていること) が満たされる。
  <2>2. CASE: `v` の名前が `Binding::Param` を持つ。
    BY DEF-0 の (v-2) -- パラメータと capture は `ρ` のどの位置でも値を持つ。
  <2>3. CASE: `v` の名前が L10 (a) の 3 構文が作る束縛を持つ。
    BY A11 (`M` の位置での `v` の使用は、その位置でスコープに入っている束縛に解決する),
       L10 (a') (`vars.bindings` の束縛のうち `Binding::Param` でないものは (a) の 3 構文が作る),
       L10 (c3) (`M` はその束縛の D2 の意味のスコープに在り、`ρ` は `M` を通るので、`ρ` は `v` の
       授与位置を `M` までに通る),
       DEF-0 の (v-1) (`ρ` は `v` の授与位置を `M` までに、したがって `P` までに通るので、`v` は `P` で
       値を持つ)
  <2>4. QED
    BY <2>1, <2>2, <2>3, L10 (a') -- `vars.bindings` に入る名前は `Binding::Param` を持つものと
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
     `P` までに通る), L10 (c2) (`ρ` はその位置へ進む前に `α` が選んだアーム本体の終端の `Ret` を通る),
     D21 (活性化が選ぶアームは決まっている),
     CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
     CODE src/rc_ir/ownership.rs: returned_var (本体の終端の `Ret` が名指す変数)
<1>4. QED
  BY <1>1, <1>1a, <1>2, <1>3

**L12 (値の leaf が参照を持つのは、計数下のオブジェクトを指すときである)**: `v` を `P` で値を持つ
(DEF-0) `RcVar`、`λ` を `ty(v)` の boxed leaf であって `P` で inhabited (D16) であるものとする。`v` の値の
leaf `λ` が D8 の意味の参照を持つことと、その leaf が指すオブジェクトが計数下 (D26) であることは同値で
あり、持つときその個数はちょうど 1 である。

`v` が DEF-0 の (v-1) か (v-2) であるとき、`(v, λ)` は D6 のスロットであり、この補題はそのスロットが
持つ参照について述べる。(v-3) のときスロットは無いが、値の leaf は在るので、言明はそのまま読める。

<1>1. `λ` が計数下のオブジェクトを指すとき、`v` の値の leaf `λ` にはちょうど 1 つの参照がある。
  BY A5 -- その本文は「値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf のうち、
     inhabited (D16) であって計数下のオブジェクト (D26) を指すものにちょうど 1 つずつある」である。
<1>2. `λ` がグローバル状態のオブジェクトを指すとき、`v` の値の leaf `λ` は D8 の意味の参照を持たない。
  BY A5 (「グローバル状態のオブジェクトを指す leaf も参照を持たない (D26)」), D26 (「グローバル状態の
     オブジェクトを指す leaf は、D8 の意味の参照を持たない」)
<1>3. オブジェクトは計数下かグローバル状態かのどちらかである。
  BY D26 (「オブジェクトは**計数下**かグローバル状態かのどちらかである」)
<1>4. QED
  BY <1>1, <1>2, <1>3, D16 (「**null ポインタの leaf は inhabited でない。**」) -- `λ` は inhabited
     なので null ポインタの leaf ではなく、A5 が例外に挙げる capture が空のクロージャの capture の
     leaf には当たらない。

**L13 (束縛を持たない名前の値はグローバル状態のオブジェクトだけを指す)**: `v` を、`P` で DEF-0 の
(v-3) として値を持つ `RcVar` -- その名前が `vars.bindings` に束縛を持たず、`ρ` が `v` を名指す節点を
`P` までに通っているもの -- とする。`P` における `v` の値の inhabited な各 boxed leaf が指す
オブジェクトは、グローバル状態 (D26) である。よって L12 より、その leaf は D8 の意味の参照を持たない。

**この補題が D6 に足すもの。** D6 の「値を得る形は 3 つあり」の段落は、束縛を持たない名前について
「そこが指すのは funptr かグローバル状態のオブジェクト」と、2 つを並べたまま述べる。L13 はその 2 つを
分け、funptr の側には boxed leaf が無いこと (`<1>3`) を示して、**inhabited な各 boxed leaf について**
グローバル状態であると言う。L12 が要求するのはこの形である。

**「束縛を持たない名前は最上位の記号の名前である」を与えるのは D6 である。** `Lowerer::lower_var` の
`resolve` が `None` を返す枝に立つ `assert!(!v.name.is_local(), ..)` は、この条件が成り立つ理由では
ない -- README の第 4 節が「**表明は不変条件の出どころであって、仮定を果たす者ではない。**」と述べる
とおりである。

<1>1. `v` の名前は、そのプログラムの最上位の記号の名前である。
  BY DEF-0 の (v-3) (`v` の名前は `vars.bindings` に束縛を持たない),
     D6 (「**束縛を持たない名前は、必ず最上位の記号の名前である。**」), L10 (d)
<1>1a. `v` の名前は局所名ではない。すなわち `FullName::is_local` が偽である。
  BY <1>1, L10 (d), A13 (「**最上位の記号の名前は局所名ではない。**`FullName::is_local` が偽であり、
     `prog.funcs` の鍵と `global_types` の鍵はどちらもそのような名前である」)
<1>2. `v` の値は、`declare_program_global` が用意する 2 つのうちの一方から来る -- 型が `is_funptr` なら
      `declare_lambda_function` が返す関数の番地、そうでなければ `add_global_object` が登録する
      グローバルのアクセサが返す値である。
  BY <1>1a, L10 (d)
<1>3. 型が `is_funptr` のとき、`ty(v)` は boxed leaf を持たない。よって主張は空虚である。
  BY A12 (束縛を持たない `RcVar` の型は、その名前の記号の型である), D4 の第 1 の規則
     (`is_fully_unboxed` が真の型は leaf を持たない),
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed (`is_funptr` の型に真を返す),
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths (`is_fully_unboxed` の型で走査が `return` する)
<1>4. そうでないとき `v` はグローバル値であり、`P` における `v` の値の inhabited な各 boxed leaf が
      指すオブジェクトはグローバル状態である。**言明が要るのは深さ 1 -- 値自身の各 boxed leaf -- だけ
      なので、この段はそこまでを述べる。**
  <2>1. `v` の値は、`v` が名指す記号の初期化子の活性化が返し、アクセサが記憶域へ格納した 1 つの値で
        ある。`P` はその活性化の (E5) の段より後にある。**DEF-0 の (v-3) が「その記号の値」と言うのは
        この格納された値であり、初期化子がそれを返す前にその値は無い。**
    BY <1>2 (アクセサの枝), DEF-0 の (v-3) (`v` の値はその記号の値であり、`ρ` は `v` を名指す節点を
       `P` までに通っている),
       D22 のグローバルのアクセサの行 (「まだならグローバル初期化子の本体を持つ関数 `InitValue#<symbol>`
       を呼び、返った値を記憶域へ格納する」「以後の読みは記憶域を読むだけである」),
       D24 の (E7) (`g` を読む節点の位置にある活性化は初期化子の活性化 `b` が終わるまで中断中であり、
       `b` が終端の `Ret` に着き、返す前に (E5) の段が走ってから再開する) -- `ρ` が通ったその節点が
       `v` を読むので、その時点までに初期化が済んでおり、(E5) の段はそれより前にある
  <2>2. その (E5) の段の時点で、`v` の値の inhabited な各 boxed leaf が指すオブジェクトには
        `mark_global` が印を付ける。
    BY <2>1, D24 の (E5) (「返す前に、環境が `mark_global` でその値が到達するオブジェクトのグラフ全体に
       印を付ける」), A8 (「グローバル値が到達するオブジェクトは、記憶域に「グローバル」を表す状態を
       持ち」), D25 (「オブジェクト `o` から `o''` へ到達できる」の起点を値に取ったものが A8 と (E5) の
       「値が到達するオブジェクト」であり、値の inhabited な各 boxed leaf が指すオブジェクトはその
       第 1 段である)
  <2>3. `P` においてその leaf が指すオブジェクトは、(E5) の段で印が付いたオブジェクトそのものである。
    BY <2>1, L10 (b) (`v` が値を持った位置以後、`v` の値は同じである), D16 (leaf が inhabited である
       かどうかはその値が通る各 unbox union の節のタグで決まるので、値が同じなら (E5) の時点と `P` で
       一致する),
       A5 (値が保持する参照の在りかは、その型の boxed leaf のうち inhabited なものである)
  <2>4. QED
    BY <2>2, <2>3, D26 (「割り当てられたオブジェクトは計数下であり、グローバル値が到達するグラフに
       `mark_global` が印を付けた時点でグローバル状態になる。逆向きの遷移は無い」) -- 印が付いた時点
       以後そのオブジェクトはグローバル状態であり、`P` はその時点より後である (<2>1)。
<1>5. QED
  BY <1>1, <1>1a, <1>2, <1>3, <1>4, L12 -- <1>2 の 2 つの場合のうち funptr の側は <1>3 が空虚にし、
     残る側は <1>4 が与える。L12 が、グローバル状態のオブジェクトを指す leaf は D8 の意味の参照を
     持たないことを与える。

**L14 (`origin` の再帰呼び出しの鍵の関係は整礎である)**: 鍵 `(v, q)` の間の**再帰の辺**
`(v, q) -> (v', q')` を、「`origin_inner(vars, type_env, v, q)` の実行が
`origin(vars, type_env, v', q')` を呼ぶこと」と定める。`origin` が呼ばれる鍵 `K_0` を取ると、`K_0` から
再帰の辺を辿って無限に降りる鍵の列は無い。すなわちこの関係は `K_0` から到達する鍵の上で整礎であり、
その上の整礎帰納が使える。

**閉路の不在だけでは足りない。** 鍵の第 2 成分は path であり、E5 と E7 の辺はその先頭に添字を足すので
path は伸びる。鍵の到達集合が有限であることはどこにも述べられていないので、閉路が無いことから無限降下列が
無いことは出ない。この補題は有限性を経由せず、P2 の停止性と memo の規律から無限降下列を直接排除する。

<1>1. `origin(K)` は、`vars.origins` に鍵 `K` が在ればその値の複製を返し、無ければ `origin_inner(K)` を
      走らせ、その返り値を鍵 `K` で `origins` に入れてから返す。`origins` から要素が取り除かれることは
      無い。
  BY CODE src/rc_ir/ownership.rs: origin (`vars.origins.borrow().get(&key)` が当たれば `known.clone()` を
     返し、そうでなければ `grow_stack(|| origin_inner(..))` の値を
     `vars.origins.borrow_mut().insert(key, answer.clone())` で入れてから返す),
     CODE src/rc_ir/ownership.rs: VarTable (`origins` は `RefCell<Map<VarPath, Origin>>` であり、
     `VarTable::of` と `VarTable::body_only` が空で作る。読み書きするのは `origin` のこの 2 行だけで
     あり、取り除く操作はどこにも無い),
     A15 (`grow_stack` は閉包をちょうど 1 回呼び、その返り値を返す)
<1>1a. 1 つの `VarTable` に対する `origin` の呼び出しは 1 つの制御の流れの上にあり、`origins` への
       `insert` は時間で全順序に並ぶ。
  BY <1>1, CODE src/rc_ir/ownership.rs: VarTable -- `origins` は `RefCell` の欄であり、Rust の規則に
     より `RefCell<T>` は `Sync` ではなく、それを欄に持つ `VarTable` も `Sync` ではない。よって
     `&VarTable` は別のスレッドへ渡らず、この表を読み書きする呼び出しは 1 つの流れの上に並ぶ。
<1>2. どの鍵についても `origin` の呼び出しは停止する。
  BY L10 (a') (`vars.bindings` に束縛を持つ名前は、その関数のパラメータ・capture か、値を与える 3 構文が
     束縛する変数であり、どれもプログラムの束縛変数である。束縛を持たない名前は P2 が第 2 の場合として
     挙げるものである。よってどの名前もこの 2 つのどちらかであり、鍵はすべて P2 の範囲に在る),
     P2 (`origin(x, π)` はその 2 種の `x` について `π` を問わず panic せずに答えを返し、停止する)
<1>3. `origin(K)` が呼ばれるならば、その呼び出しが返った時点で `origins` は `K` を含む。`origins` へ
      要素を入れる `insert` は <1>1 の 1 か所だけであり、その呼び出しは時間で全順序に並ぶ。`K` を入れる
      最初の `insert` が何番目かを `t(K)` と書く。
  BY <1>1, <1>1a, <1>2
<1>4. `origin(K)` が呼ばれ、再帰の辺 `K -> K'` が在るならば、`origin(K')` も呼ばれ、`t(K') < t(K)` で
      ある。
  <2>1. `K` を `origins` に入れた呼び出しが在り、それは `origin_inner(K)` を走らせ、それが返った後に
        `K` を入れている。
    BY <1>1, <1>3 -- `origin(K)` が呼ばれるので `K` は `origins` に入る (<1>3)。`insert` は <1>1 が
       挙げる 1 か所にしかなく、入れるのは `origin_inner` の返り値である。
  <2>2. その `origin_inner(K)` の実行は `origin(K')` を呼び、その呼び出しは `origin_inner(K)` が返る
        前に返る。
    BY <2>1, 再帰の辺の定義, <1>2 (その呼び出しは停止する),
       CODE src/rc_ir/ownership.rs: origin_inner (`origin_inner` が行う再帰呼び出しの鍵は `vars`、
       `type_env`、`var`、`path` だけで決まる。memo は `origin` の側に在り、`origin_inner` がどの鍵を
       呼ぶかを変えない。よって `origin_inner(K)` のどの実行も同じ辺を辿る)
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
  BY <1>3, <1>4 -- 無限に降りる列 `K_0 -> K_1 -> K_2 -> ...` が在るとすると、`origin(K_0)` は呼ばれる
     ので `t(K_0)` が定まり (<1>3)、<1>4 を各辺に順に当てると `origin(K_i)` はどれも呼ばれ、
     `t(K_0) > t(K_1) > t(K_2) > ...` となる。自然数の無限の狭義減少列は無い。

**L15 (`origin` の答えは鍵ごとに 1 つに決まり、それは `origin_inner` が答えた値である)**: 1 つの
`VarTable` と 1 つの `TypeEnv` を固定する。鍵 `K` について `origin(K)` の呼び出しが 1 つでも在るならば、
次の 2 つが成り立つ。

- **(a)** そのうち `origin_inner(K)` を走らせる呼び出し (**cold な呼び出し**) はちょうど 1 つである。
- **(b)** `origin(K)` のどの呼び出しも、その cold な呼び出しの `origin_inner(K)` が返した値か、その
  複製を返す。複製は `identity` と `candidates` をそのまま運ぶ (L1)。

よって **`origin(K)` の答えは `K` について 1 つに決まる。** 以下これを**鍵 `K` の答え**と呼び、
`origin(v, q)` という記法はその答えを指す。`origin_inner` の 1 つの実行について立てた言明は、そのまま
その鍵の答えについての言明として読める。

**この補題が要る場所。** `origin_inner` の腕の多くは「再帰呼び出しが返した値をそのまま返す」形をして
おり、その値を**鍵の答え**と読み替える段が L8 と補題 Q の各 CASE に在る。`origin` は memo が当たると
`origin_inner` を走らせないので、その読み替えはこの補題を経る。

<1>1. `origin(K)` の呼び出しは 2 種である。`origins` に `K` が在るときはその値の複製を返し
      (**warm な呼び出し**)、無いときは `origin_inner(K)` を走らせ、その返り値を鍵 `K` で `origins` に
      入れてから返す (**cold な呼び出し**)。`origins` へ `insert` するのはこの 1 か所であり、要素が
      取り除かれることは無い。`origins` は空で作られる。
  BY CODE src/rc_ir/ownership.rs: origin (`vars.origins.borrow().get(&key)` が当たれば `known.clone()` を
     返し、そうでなければ `grow_stack(|| origin_inner(..))` の値を
     `vars.origins.borrow_mut().insert(key, answer.clone())` で入れてから返す),
     CODE src/rc_ir/ownership.rs: VarTable (`origins` は `RefCell<Map<VarPath, Origin>>` であり、
     `VarTable::of` と `VarTable::body_only` は `VarTable::empty` を通って空の表を作る。読み書きするのは
     `origin` のこの 2 行だけであり、取り除く操作はどこにも無い),
     A15 (`grow_stack` は閉包をちょうど 1 回呼び、その返り値を返す)
<1>1a. 1 つの `VarTable` に対する `origin` の呼び出しは 1 つの制御の流れの上にある。よって 2 つの
       呼び出しの時間の区間が重なるならば、一方は他方の内側にある。
  BY <1>1, CODE src/rc_ir/ownership.rs: VarTable -- `origins` は `RefCell` の欄であり、Rust の規則に
     より `RefCell<T>` は `Sync` ではなく、それを欄に持つ `VarTable` も `Sync` ではない。よって
     `&VarTable` は別のスレッドへ渡らず、この表を読み書きする呼び出しは 1 つの流れの上に並ぶ。
<1>2. (a)。
  <2>1. 鍵 `K` について `origin(K)` の呼び出しが在るならば、そのうち最初のものは cold である。
    BY <1>1, <1>1a -- `origins` は空で作られ、`K` が入るのは `K` についての cold な呼び出しの
       `insert` だけである。
  <2>2. 同じ鍵 `K` について相異なる cold な呼び出し `c_1` と `c_2` が在るとすると、2 つの区間は重なる。
    BY <1>1 -- `c_2` が memo を見る時点で `origins` に `K` は無く、`c_1` は返る時点で `K` を入れて以後
       取り除かれないので、`c_2` が memo を見る時点は `c_1` が返る時点より前である。添字を入れ替えると
       同じ議論が `c_1` が memo を見る時点は `c_2` が返る時点より前であることを与える。
  <2>3. よって一方は他方の内側にあり、内側の呼び出しは外側の `origin_inner(K)` の実行の中で起きる。
    BY <2>2, <1>1a, <1>1,
       CODE src/rc_ir/ownership.rs: origin -- 外側の呼び出しがほかのコードを走らせるのは
       `grow_stack(|| origin_inner(..))` の 1 か所だけであり、`insert` はその後に来る
  <2>4. QED
    BY <2>1, <2>3, L14 -- 内側の呼び出しは `origin_inner(K)` の実行の中の `origin(K)` の呼び出しで
       ある。その入れ子は `origin_inner` と `origin` の枠が交互に並ぶ列であり、何かを内側に持つ
       `origin` の枠は `origin_inner` を走らせているので cold である。よって鍵 `K` から再帰の辺を
       1 回以上辿って `K` に着く道が在る。その道を繰り返せば `K` から無限に降りる鍵の列ができ、
       L14 に反する。したがって cold な呼び出しは高々 1 つであり、<2>1 と合わせてちょうど 1 つである。
<1>3. (b)。
  <2>1. cold な呼び出しは `origin_inner(K)` が返した値そのものを返し、同じ値を `origins` に入れる。
    BY <1>1
  <2>2. warm な呼び出しは `origins` の `K` の値の複製を返す。`origins` の `K` の値を入れたのは
        `K` についての cold な呼び出しであり (<1>1)、それは 1 つである (<1>2)。
    BY <1>1, <1>2
  <2>3. QED
    BY <2>1, <2>2, L1 (複製は `identity` と `candidates` をそのまま運ぶ)
<1>4. QED
  BY <1>2, <1>3

## 5. DEF-1 -- D17 の「対応するスロット」を鎖の形に書き直したもの

D17 は「`origin` が `(x, π)` から `(u, σ)` へ辿った別名の辺の列を、`π` の下の leaf `λ` について辿ったときに
着く leaf のスロット」を、`λ` に**対応するスロット**と呼ぶ。辺ごとの `λ` の写り方と、辺の行き先は D17 が
2 つの箇条書きで与えている。行き先の側の 3 行は次のとおりである。

> - `Binding::Join` の辺は、その活性化が選んだアームの結果へ辿る。`origin_inner` はアームを静的に列挙して
>   候補を集めるが (D3)、1 つの活性化では 1 つのアームが選ばれ (D21)、対応するスロットはその結果の側にある。
> - `Binding::Llvm` の leaf の宣言が単一の `Fresh` または単一の `Unknown` であるとき、鎖はそこで止まり、
>   対応するスロットはその位置の `(u, λ)` である。
> - `origin_from_leaves_under` が辿る辺の行き先の path は、宣言の `σ'` ではなく
>   `truncate_to_unit(ty(args[j]), σ')` である。

DEF-1 は、この 2 つの箇条書きを、Q の帰納法が 1 段ずつ辿れる 3 つ組の列として書き直したものである。

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

`λ_cur` の宣言が空集合である場合は、A3 よりその leaf は inhabited でないので、補題 Q の量化から外れる。
`λ_cur` の宣言が 2 元以上である場合は、L7 より現在のプログラムには存在しない。`λ_cur` に宣言が無い場合は
L8 (a) より起きない。よってこの表と停止条件は尽きている。

止まった位置の 3 つ組を `(u, σ_end, μ)` とし、`(u, μ)` を `λ` の**対応する位置**と呼ぶ。補題 Q の (ii)
が `(u, μ)` は `(x, λ)` と同じオブジェクトを指すことを、(iii) が、`x` の値の leaf `λ` が計数下の
オブジェクト (D26) を指すとき `(u, μ)` は D6 のスロットであることを示す。D17 の「対応するスロット」が
指すのは、その場合のこれである。そうでないとき `(u, μ)` は D6 の**記号の位置**でありうる。

E4a の行き先の path が `σ'` ではなく `t_{ty(args[j])}(σ')` であることは、コードでは
`operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)))` である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。`truncate_to_unit` は `out` を `path` の
接頭辞として作るので `σ' ⊒ t_{ty(args[j])}(σ')` であり、leaf は行き先の path の下に留まる
(`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。

## 6. 補題 Q、および P3 と P4

**補題 Q**。ASSUME:

- NEW `α`: 活性化、NEW `ρ`: `α` が辿る実行路、NEW `P`: `ρ` 上の位置、
- NEW `x`: `P` で値を持つ (DEF-0) `RcVar`、NEW `π`: path、
- NEW `λ`: `ty(x)` の boxed leaf であって `λ ⊒ π` かつ `P` で inhabited (D16) であるもの

PROVE: DEF-1 の鎖は有限で止まり、その停止点 `(u, σ_end, μ)` は次を満たす。

- (i) `(u, σ_end) ∈ cand(x, π)`。
- (ii) `u` は `P` で値を持ち (DEF-0)、`μ ⊒ σ_end` であり、`μ` は `ty(u)` の boxed leaf であって `P` で
  inhabited であり、**`u` の値の leaf `μ` は `x` の値の leaf `λ` と同じオブジェクトを指す**。
- (iii) さらに **(H)** -- `P` における `x` の値の leaf `λ` が指すオブジェクトは計数下 (D26) である --
  を仮定すると、`u` は DEF-0 の (v-1) か (v-2) であり、`(x, λ)` と `(u, μ)` はどちらも `P` のスロット
  (D6) であって、それぞれちょうど 1 つの D8 の参照を持ち、その 2 つは同一である。

**(H) を読むのは (iii) だけである。** D26 がグローバル状態のオブジェクトを D8 の勘定の外に置くので、
(H) が無いと参照について言えることは無い。L12 より (H) は「`(x, λ)` が D8 の意味の参照を持つ」と同値で
あり、README の P3 と P4 が「**スロット `(x, λ)` が D8 の意味の参照を持つとき**」と書いているのと同じ
条件である。グローバル値 `g` を `Let(x, Var(g), k)` で受ける本体では、鎖は `vars.bindings` が束縛を
持たない名前 `g` で止まり (L9 の H1)、`g` は DEF-0 の (v-3) なので `(g, λ)` は D6 のスロットではなく
**記号の位置**であって、両端の leaf はどちらも D8 の意味の参照を持たない (L13)。D26 自身が「この形は
実プログラムにいくらでもある」と述べている。

**(ii) のオブジェクトの同一は (H) を読まない。** 各段でそれを与えるのは D9 の移動の表の**値の水準の
6 行**であり、その 6 行はどれも参照を持ち出さない。README の P3 が「**条件を外した形では、2 つのスロットは
同じオブジェクトを指す。**」と述べ、その根拠を「両端がグローバル状態を指す場合は D9 の値の水準の行が
与える」と書くのがこの節である。

証明は、`origin` が `(x, π)` から行う再帰呼び出しの関係の上の整礎帰納による。この関係が整礎であることは
L14 が与える。DEF-1 の各段は `origin_inner` の再帰呼び出しの 1 つに一致する (L6) ので、鎖の各段で
帰納法の仮定が使える。

**各 CASE の第 1 の段は L15 を経る。** `origin_inner` の腕が返すのは再帰呼び出しの返り値であり、それを
`origin(v, π')` という**鍵の答え**の等式として読むには、答えが鍵ごとに 1 つに決まることが要る。
`origin` は memo が当たると `origin_inner` を走らせないので、L15 がその一段を与える。

**各段の形は共通である。** 段が `(x, π, λ)` から `(v, π', λ')` へ進むとき、次の 5 つをこの順に置く。

1. `v` は `P` で値を持つ (L11)。
2. `x` の値の leaf `λ` と `v` の値の leaf `λ'` は、同じオブジェクトを指す。**どの段でもこれを与えるのは
   D9 の移動の表の値の水準の行である** -- 2 つの leaf の値が等しければ、2 つは同じオブジェクトを指す。
   `Llvm` の 2 つの段 (E3、E4a) もその表の「`Llvm` の素通し leaf」の行を読む。この一歩は (H) を読まない。
3. (H) を仮定すると、2 と D26 より `v` の値の leaf `λ'` が指すオブジェクトも計数下である。L13 の対偶より
   `v` は DEF-0 の (v-3) ではなく、(v-1) か (v-2) である。すなわち `(v, λ')` は `P` のスロットである
   (D6)。
4. (H) の下で、L12 より `(x, λ)` と `(v, λ')` はそれぞれちょうど 1 つの参照を持ち、D9 の移動の表の
   参照の水準の行よりその 2 つは同一である。`Llvm` の 2 つの段では、その行が述べる素通しを A3 の
   「単一の `Arg(j, σ)`」の行が生成コードの水準で言い直している。
5. 帰納法の仮定を `(v, π')` に適用する。その前提 -- `v` が `P` で値を持つこと (1) と、`λ'` が
   `ty(v)` の `P` で inhabited な boxed leaf であること (各 CASE の型についての段) -- は (H) を
   読まない。(i) を `cand(x, π)` へ読み替え、(ii) のオブジェクトの同一を 2 と繋ぐ。(H) の下では 3 が
   帰納法の仮定の (iii) の前提を与えるので、(iii) を 4 と繋ぐ。

**この段落は読みの見取り図である。**以下の各 CASE は、この 5 つを、その段が読む D9 の行と A3 の行を
名指して書き下す。

<1>0. (H) を仮定すると、`x` は DEF-0 の (v-1) か (v-2) であり、`(x, λ)` は `P` のスロット (D6) であって、
      ちょうど 1 つの D8 の参照を持つ。
  BY (H), L13 (対偶 -- (v-3) の名前の値の inhabited な leaf はグローバル状態のオブジェクトを指す),
     D26 (計数下とグローバル状態は排他である), 前提 (`x` は `P` で値を持ち、`λ` は `ty(x)` の `P` で
     inhabited な boxed leaf である), D6, L12

<1>1. CASE: 停止条件 S1 (`origin_inner` が `here()` を答える)。
  <2>1. `origin(x, π) = Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `None | Some(Binding::Param) | Some(Binding::Producer)`
       の腕、`Some(Binding::Field(..))` の `container.ty.is_box` の枝、`Some(Binding::Payload(..))` の
       `Some(_)` の枝, L9, L2 (b), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>2. 停止点は `(x, π, λ)` である。
    BY DEF-1 の S1
  <2>3. QED
    BY <2>1, <2>2, <1>0 -- (i) は `(x, π) ∈ {(x, π)}`。(ii) は前提の `λ ⊒ π`、`λ` が `ty(x)` の
       `P` で inhabited な boxed leaf であること、`x` が `P` で値を持つこと、および停止点が `(x, λ)`
       自身なのでオブジェクトの同一が等号であること。(iii) は (H) の下で `(u, μ) = (x, λ)` すなわち
       同じスロットどうしであり、<1>0 がそれがちょうど 1 つの参照を持つことを与える。

<1>2. CASE: 停止条件 S2 (`Llvm` で `λ` の宣言が単一の `Fresh` または単一の `Unknown`)。
  <2>1. `Exactly((x, π))` は `reached` の要素である。
    BY L8 (d1)
  <2>2. `(x, π) ∈ cand(x, π)`。
    <3>1. `reached` の全要素が等しいとき、鍵 `(x, π)` の答えは `Exactly((x, π))` であり
          `cand(x, π) = {(x, π)}`。
      BY <2>1, L4 (b), L2 (b), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. そうでないとき、鍵 `(x, π)` の答えは `of_candidates(C, (x, π))` であり
          `C ⊇ act(Exactly((x, π))) = {(x, π)}` である。`of_candidates` の `candidates()` は `C` そのもの
          である。
      BY <2>1, L4 (b), L3, L2 (b), L15 (`origin(・, ・)` の記法は鍵の答えを指す),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. QED
    BY <2>2, <1>0, DEF-1 の S2 -- 停止点は `(x, π, λ)` なので (i) は <2>2、(ii) は前提 (`x` は
       `P` で値を持ち、`λ` は `P` で inhabited である) と、オブジェクトの同一が等号であることから出る。
       (iii) は (H) の下で同じスロットどうしであり、<1>0 がそれがちょうど 1 つの参照を持つことを
       与える。

<1>3. CASE: 段 E1 (`Move(y)`)。
  <2>1. `origin(x, π) = origin(y, π)` であり `cand(x, π) = cand(y, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕 (この腕は
       `origin(vars, type_env, &y.name, path)` の返り値をそのまま返す), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `y` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E1 (`x` の束縛は `Move(y)` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>2. `P` における `x` の値は `y` の値であり、`ty(y) = ty(x)` であり、`λ` は `ty(y)` の boxed leaf で
        `P` で inhabited である。
    BY D9 の移動の表の値の水準の行 (「`Let(x, Var(y), k)`: `x` の値は `y` の値である」),
       A12 (move-bind の両辺の型が一致する), L10 (b) (`x` の値も `y` の値も、値を持った位置の後は
       変わらない), <2>1a, D16 -- `x` と `y` の値は同じなので、`λ` が通る各 unbox union の節のタグも
       同じである。
  <2>2a. `y` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>2, <2>1a, D9 の移動の表の値の水準の行 (「`Let(x, Var(y), k)`: `x` の値は `y` の値である」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `y` は DEF-0 の (v-1) か (v-2) で
         あり、`(y, λ)` は `P` のスロットである。
    BY <2>2a, <2>2, <2>1a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>3. (H) の下で、`(x, λ)` と `(y, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, L12, D9 の移動の表の `Let(x, Var(y), k)` の行 (`y` の参照が `x` へ), D8
  <2>4. 帰納法の仮定を `(y, π)` に適用する。前提は <2>1a (`y` は `P` で値を持つ)、<2>2 (`λ` は `ty(y)` の
        `P` で inhabited な boxed leaf であり `λ ⊒ π`) が与える。結論は (i) `(u, σ_end) ∈ cand(y, π)`、
        (ii) `(u, μ)` が `(y, λ)` と同じオブジェクトを指すこと、(iii) (H) の下で `(y, λ)` の参照と
        同一であること、である。(iii) の前提となる `(y, λ)` についての (H) は <2>2b が与える。
    BY <2>1a, <2>2, <2>2b, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>2a, <2>3, <2>4, <1>0 -- (i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は <2>4 の (ii) と
       <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と <2>4 の (iii) の推移であり、`(x, λ)` が
       スロットであることは <1>0 が与える。

<1>4. CASE: 段 E6 (`Payload(s, None)`、catch-all)。
  <2>1. `origin(x, π) = origin(s, π)` であり `cand(x, π) = cand(s, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の `None =>` の枝
       (この枝は `origin(vars, type_env, &scrut.name, path)` の返り値をそのまま返す), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `s` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E6 (`x` の束縛は `Payload(s, None)` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>2. `P` における `x` の値は `s` の値であり、`ty(s) = ty(x)` であり、`λ` は `ty(s)` の boxed leaf で
        `P` で inhabited である。
    BY D9 の移動の表の値の水準の行 (「catch-all アームの payload 束縛: payload 変数の値は
       scrutinee の値そのものである。」), A12 (catch-all アームの payload と scrutinee の型が一致する),
       L10 (b) (`x` の値も `s` の値も、値を持った位置の後は変わらない), <2>1a, D16 -- 2 つの値は同じ
       なので、`λ` が通る各 unbox union の節のタグも同じである。
  <2>2a. `s` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>2, <2>1a, D9 の移動の表の値の水準の行 (「catch-all アームの payload 束縛: payload 変数の値は
       scrutinee の値そのものである。」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `s` は DEF-0 の (v-1) か (v-2) で
         あり、`(s, λ)` は `P` のスロットである。
    BY <2>2a, <2>2, <2>1a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>3. (H) の下で、`(x, λ)` と `(s, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, L12, D9 の移動の表の catch-all アームの payload 束縛の行 (scrutinee の参照が payload
       変数へ), D20 (catch-all アームの scrutinee から payload 変数への別名の辺), D8
  <2>4. 帰納法の仮定を `(s, π)` に適用する。前提は <2>1a、<2>2 が与える。結論は
        (i) `(u, σ_end) ∈ cand(s, π)`、(ii) `(u, μ)` が `(s, λ)` と同じオブジェクトを指すこと、
        (iii) (H) の下で `(s, λ)` の参照と同一であること、である。(iii) の前提となる `(s, λ)` に
        ついての (H) は <2>2b が与える。
    BY <2>1a, <2>2, <2>2b, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>2a, <2>3, <2>4, <1>0 -- (i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は <2>4 の (ii) と
       <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と <2>4 の (iii) の推移であり、`(x, λ)` が
       スロットであることは <1>0 が与える。

<1>5. CASE: 段 E5 (`Field(c, i)`、`c` が unbox)。
  <2>1. `origin(x, π) = origin(c, [i] ++ π)` であり `cand(x, π) = cand(c, [i] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(..))` の `else` の枝 (この枝は
       `origin(vars, type_env, &container.name, &container_path)` の返り値をそのまま返す),
       L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `c` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E5 (`x` の束縛は `Field(c, i)` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>2. `[i] ++ λ` は `ty(c)` の boxed leaf であり、`[i] ++ λ ⊒ [i] ++ π` である。
    BY A12 (`Destructure` のフィールド変数とフィールドの型が合っていること、容器が構造体であること、
       **`Destructure` が名指すフィールドがその型が実際に持つ (punched でない) ものであること**),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- unbox 集約の枝は `unpunched_field_types` の
       各フィールドへ添字を積んで降りるので、punched でないフィールド `i` の leaf は
       `[i] ++ (そのフィールドの型の leaf)` である,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types (punched なフィールドを落とす)
  <2>3. `P` における `x` の値は `c` の値の第 `i` フィールドであり、`[i] ++ λ` は `P` で inhabited で
        ある。
    BY <2>2, <2>1a, D16, D9 の移動の表の値の水準の行 (「unbox 容器の `Destructure` の名前付き
       フィールド: フィールド変数の値は容器の値のそのフィールドである。」), L10 (b) (`x` の値も `c` の
       値も、値を持った位置の後は変わらない) -- `[i]` は unbox 構造体のフィールド添字なので unbox union
       の節を通らず、`[i] ++ λ` が通る union の節は `λ` が通る節と同じである。
  <2>3a. `c` の値の leaf `[i] ++ λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY <2>3, <2>1a, D9 の移動の表の値の水準の行 (「unbox 容器の `Destructure` の名前付きフィールド:
       フィールド変数の値は容器の値のそのフィールドである。」)
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `c` は DEF-0 の (v-1) か (v-2) で
         あり、`(c, [i] ++ λ)` は `P` のスロットである。
    BY <2>3a, <2>3, <2>1a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>4. (H) の下で、`(x, λ)` と `(c, [i] ++ λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, L12, D9 の移動の表の unbox 容器の `Destructure` の名前付きフィールドの行 (`c` のその
       フィールドの参照がフィールド変数へ), D8
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>1a, <1>0, 帰納法の仮定を `(c, [i] ++ π)` に適用 --
       帰納法の仮定の前提は <2>1a、<2>2、<2>3 が与え、その (iii) の前提となる `(c, [i] ++ λ)` に
       ついての (H) は <2>3b が与える。(i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は帰納法の
       仮定の (ii) と <2>3a を繋いだものである。(iii) は (H) の下で <2>4 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。

<1>6. CASE: 段 E7 (`Payload(s, Some(t))`、`s` が unbox)。
  <2>1. `origin(x, π) = origin(s, [t] ++ π)` であり `cand(x, π) = cand(s, [t] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の
       `Some(tag) if !scrut.ty.is_box(type_env)` の枝 (この枝は
       `origin(vars, type_env, &scrut.name, &scrut_path)` の返り値をそのまま返す),
       L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>1a. `s` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E7 (`x` の束縛は `Payload(s, Some(t))` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf であり、`[t] ++ λ ⊒ [t] ++ π` である。
    BY A12 (payload と変位の型が合っていること、scrutinee が union であること、**`Match` が名指す変位が
       その型が実際に持つ (punched でない) ものであること**),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types -- union の `unpunched_field_types` は
       punched でない各変位の payload の型を返すので、変位 `t` の leaf は
       `[t] ++ (その payload の型の leaf)` である
  <2>3. `P` において `s` のタグは `t` である。
    <3>1. `x` は `tag = Some(t)` のアーム `A` の `payload` である。`α` は `A` を選んでおり、`ρ` は
          `A` の `body` の根の節点を `P` までに通っている。
      BY 補題 Q の前提 (`x` は `P` で値を持つ), DEF-0 の (v-1) (`x` は `Binding` を持つので (v-3) では
         なく、`Binding::Payload` は `Binding::Param` ではないので (v-2) でもない。よって `ρ` は `x` の
         授与位置を `P` までに通る。(v-1) の表の第 4 行より、その授与位置は `A` の `body` の根の節点で
         ある), L10 (c2) (`ρ` が `A` の `body` の根の節点を通るのは `α` が `A` を選んだときに限る),
         L10 (a), L10 (c1), A6,
         CODE src/rc_ir/ownership.rs: collect_bindings -- `x` に `Binding::Payload(s, Some(t))` を
         与えるのは `tag = Some(t)` のアームの payload 束縛だけである
    <3>1a. この `Match` が catch-all アーム (`tag` が `None`) を持つならば、それは最後のアームである。
      BY 補題 Q の前提 (活性化 `α` が在る -- すなわちこの本体はコード生成を経て走っている),
         CODE src/rc_ir/codegen.rs: Generator::eval_rc_match -- 最後のアーム以外の各アームについて
         `arm.tag.expect("a non-final match arm must be a variant arm")` を評価するので、最後でない
         位置に catch-all アームが在ればコード生成が止まる。**この向き -- 表明が発火するならその
         プログラムは走らず、その本体の活性化は存在しない -- で読む。**
    <3>2. そのアームに入った時点で、`s` の値の実行時のタグは `t` である。
      <4>1. CASE: `s` の値の実行時のタグが、いずれかのアームの `tag` に等しい。D21 より `α` はそのような
            アームを選ぶ。`α` が選んだのは `A` であり (<3>1)、`A` の `tag` は `Some(t)` なので、その
            タグは `t` である。
        BY <3>1, D21
      <4>2. CASE: `s` の値の実行時のタグが、どのアームの `tag` とも等しくない。この場合は起きない。
        BY <3>1, <3>1a, A16 (`Match` のアームは scrutinee のタグを尽くす -- この場合 `arms` は
           catch-all アームを持つ), D21 (タグが等しいアームが無ければコード生成の振る舞いに従う),
           README の A16 の本文 (「コード生成は最後のアームのブロックを switch の default とする」),
           CODE src/rc_ir/codegen.rs: Generator::eval_rc_match (`else_bb` は最後のアームのブロックで
           あり、`build_switch` の default であって、`cases` が空のときの無条件の分岐先でもある)
           -- catch-all アームは最後のアームなので (<3>1a) `α` はそこへ進む。その `tag` は `None` で
           あり `A` の `tag` は `Some(t)` なので、`α` が `A` を選んだこと (<3>1) に反する。
      <4>3. QED
        BY <4>1, <4>2 -- 2 つの場合は尽きている。
    <3>3. `s` の値は、`s` が値を得た後の `ρ` 上のすべての位置で同じである。
      BY L10 (b)
    <3>4. QED
      BY <3>1, <3>2, <3>3, <2>1a -- `P` はアームに入った時点以後にあり、その時点のタグは `t` であり、
         その間 `s` の値は変わらない。
  <2>4. `P` における `x` の値は `s` の値の変位 `t` の payload であり、`[t] ++ λ` は `P` で inhabited で
        ある。
    BY <2>2, <2>3, <2>1a, D16, D20 (unbox union の変位アームの scrutinee から payload 変数への別名の辺),
       D9 の移動の表の値の水準の行 (「unbox union の変位アームの payload 束縛: payload 変数の値は
       scrutinee の値の活性変位の payload である。」), L10 (b) (`x` の値も `s` の値も、値を持った位置の
       後は変わらない) -- `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (タグ `t` で <2>3 に
       より一致する) と、`λ` が通る節である。後者が一致するのは、`x` の値が `s` の値の変位 `t` の
       payload だからである。
  <2>4a. `s` の値の leaf `[t] ++ λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY <2>4, <2>3, <2>1a, D9 の移動の表の値の水準の行 (「unbox union の変位アームの payload 束縛:
       payload 変数の値は scrutinee の値の活性変位の payload である。」-- <2>3 よりその活性変位は `t`
       である)
  <2>4b. (H) を仮定すると、そのオブジェクトは計数下である。よって `s` は DEF-0 の (v-1) か (v-2) で
         あり、`(s, [t] ++ λ)` は `P` のスロットである。
    BY <2>4a, <2>4, <2>1a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>5. (H) の下で、`(x, λ)` と `(s, [t] ++ λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>4b, L12, D9 の移動の表の unbox union の変位アームの payload 束縛の行 (scrutinee の活性変位の
       参照が payload 変数へ), D8, <2>3 (この行が名指す活性変位が `t` であること)
  <2>6. QED
    BY <2>1, <2>2, <2>4, <2>4a, <2>4b, <2>5, <2>1a, <1>0, 帰納法の仮定を `(s, [t] ++ π)` に適用 --
       帰納法の仮定の前提は <2>1a、<2>2、<2>4 が与え、その (iii) の前提となる `(s, [t] ++ λ)` に
       ついての (H) は <2>4b が与える。(i) は <2>1 で `cand(x, π)` に読み替わる。(ii) は帰納法の
       仮定の (ii) と <2>4a を繋いだものである。(iii) は (H) の下で <2>5 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。

<1>7. CASE: 段 E3 (`Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)`)。
  <2>1. `π` は `ty(x)` の boxed leaf であり、よって `λ = π` である。
    BY L8 (a) (`leaf_origins_at` が `Some` を返すのは `π ∈ leaves(ty(x))` のときである), L5, 前提の
       `λ ⊒ π`
  <2>2. `origin(x, π) = origin(args[j], σ)` であり `cand(x, π) = cand(args[j], σ)`。
    BY L8 (c), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
  <2>2a. `args[j]` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E3 (`x` の束縛は `Llvm(gen, args, ・)` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>3. `σ` は `ty(args[j])` の boxed leaf であり、`P` で inhabited である。
    BY A3 (「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字であり、`σ` は
       その型の boxed leaf である」),
       A3 の「単一の `Arg(j, σ)`」の行 -- 結果のその leaf が inhabited であることと第 `j` オペランドの
       leaf `σ` が inhabited であることは同値である。<2>1 より
       `λ = π` は `P` で inhabited である。A3 のこの同値は `Let(x, Llvm(gen, args), k)` の節点の時点に
       ついてのものであり、`P` へ運ぶのは L10 (b) と <2>2a である -- `x` の値も `args[j]` の値も、値を
       持った位置の後は変わらないので、両者の leaf が通る unbox union の節のタグは `P` でもその時点と
       同じである。この一歩に L10 (b)、<2>2a、D16 を読む。
  <2>3a. `args[j]` の値の leaf `σ` が指すオブジェクトは、`x` の値の leaf `λ = π` が指すオブジェクトと
         同じである。
    BY D9 の移動の表の値の水準の行 (「`Llvm` の素通し leaf: 結果のその leaf の値はオペランド `i` の
       その leaf の値である。」-- 2 つの leaf の値が等しいので、2 つは同じオブジェクトを指す),
       <2>1, <2>3, <2>2a, L10 (b) (`x` の値も `args[j]` の値も、値を持った位置の後は変わらない)
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `args[j]` は DEF-0 の (v-1) か
         (v-2) であり、`(args[j], σ)` は `P` のスロットである。
    BY <2>3a, <2>3, <2>2a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>4. (H) の下で、`(x, π)` と `(args[j], σ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, L12, D9 の移動の表の `Llvm` の素通し leaf の行 (オペランド `i` の参照が結果へ),
       A3 の「単一の `Arg(j, σ)`」の行 (「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を
       作らない」), D8
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>2a, <1>0, 帰納法の仮定を `(args[j], σ)` に適用 --
       DEF-1 の E3 の次の 3 つ組は `(args[j], σ, σ)` であり、`σ ⊒ σ` である。帰納法の仮定の前提は
       <2>2a、<2>3 が与え、その (iii) の前提となる `(args[j], σ)` についての (H) は <2>3b が与える。
       (i) は <2>2 で読み替わる。(ii) は帰納法の仮定の (ii) と <2>3a を繋いだものである。(iii) は
       (H) の下で <2>4 と帰納法の仮定の (iii) の推移であり、`(x, λ)` がスロットであることは <1>0 が
       与える。

<1>8. CASE: 段 E4a (`Llvm` かつ E3 でなく、`λ` の宣言が単一の `Arg(j, σ')`)。
  <2>1. `u_j := t_{ty(args[j])}(σ')` とおくと、`origin(args[j], u_j)` は `reached` の要素である。
    BY L8 (a) (`λ` は `ty(x)` の boxed leaf なので `decl` に宣言を持つ), L8 (d3)
  <2>2. `cand(x, π) ⊇ cand(args[j], u_j)`。
    <3>1. `reached` の全要素が等しいとき、鍵 `(x, π)` の答えは `origin(args[j], u_j)` そのものである。
      BY <2>1, L4 (b), L15 (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. そうでないとき、鍵 `(x, π)` の答えは `of_candidates(C, (x, π))` であり、
          `C ⊇ act(origin(args[j], u_j))` である。`of_candidates` の `candidates()` は `C` そのもので
          あり、`act ⊇ cand` (L2 (a)) である。
      BY <2>1, L4 (b), L3, L2 (a), L15 (`origin(・, ・)` の記法は鍵の答えを指す),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>2a. `args[j]` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E4a (`x` の束縛は `Llvm(gen, args, ・)` である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>3. `σ'` は `ty(args[j])` の boxed leaf であり、`σ' ⊒ u_j` であり、`P` で inhabited である。
    BY A3 (「**単一の `Arg(j, σ)` の宣言は well-formed である。** `j` は `args` の添字であり、`σ` は
       その型の boxed leaf である」),
       A3 の「単一の `Arg(j, σ)`」の行 (inhabited の同値),
       CODE src/rc_ir/ownership.rs: truncate_to_unit (`out` は `path` の接頭辞である),
       L10 (b), <2>2a, D16 -- A3 の同値は `Let(x, Llvm(gen, args), k)` の節点の時点についてのもので
       あり、`x` の値も `args[j]` の値も値を持った位置の後は変わらないので、`P` でも同じことが言える。
  <2>3a. `args[j]` の値の leaf `σ'` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと
         同じである。
    BY D9 の移動の表の値の水準の行 (「`Llvm` の素通し leaf: 結果のその leaf の値はオペランド `i` の
       その leaf の値である。」-- 2 つの leaf の値が等しいので、2 つは同じオブジェクトを指す),
       <2>3, <2>2a, L10 (b) (`x` の値も `args[j]` の値も、値を持った位置の後は変わらない)
  <2>3b. (H) を仮定すると、そのオブジェクトは計数下である。よって `args[j]` は DEF-0 の (v-1) か
         (v-2) であり、`(args[j], σ')` は `P` のスロットである。
    BY <2>3a, <2>3, <2>2a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>4. (H) の下で、`(x, λ)` と `(args[j], σ')` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の
        参照である。
    BY <2>3b, L12, D9 の移動の表の `Llvm` の素通し leaf の行 (オペランド `i` の参照が結果へ),
       A3 の「単一の `Arg(j, σ)`」の行 (「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を
       作らない」), D8
  <2>5. QED
    BY <2>2, <2>3, <2>3a, <2>3b, <2>4, <2>2a, <1>0, 帰納法の仮定を `(args[j], u_j)` に適用 --
       DEF-1 の E4a の次の 3 つ組は `(args[j], u_j, σ')` である。帰納法の仮定の前提は <2>2a、<2>3 が
       与え、その (iii) の前提となる `(args[j], σ')` についての (H) は <2>3b が与える。帰納法の仮定の
       (i) は `cand(args[j], u_j)` の元を与え、<2>2 がそれを `cand(x, π)` の元にする。(ii) は帰納法の
       仮定の (ii) と <2>3a を繋いだものである。(iii) は (H) の下で <2>4 と帰納法の仮定の (iii) の
       推移であり、`(x, λ)` がスロットであることは <1>0 が与える。

<1>9. CASE: 段 E2 (`Join(rs)`)。
  <2>1. `α` はこの `Match` のちょうど 1 つのアームを選び、`ρ` はそのアーム本体の終端の `Ret` を `P` までに
        通っている。`P` における `x` の値はその `Ret` が名指す変数 `r_0` の値である。
    BY 補題 Q の前提 (`x` は `P` で値を持つ),
       DEF-0 の (v-1) の表の第 2 行 (`x` の束縛は `Let(x, Match(scrut, arms), k)` が作るので、`x` の
       授与位置は `k` の根の節点であり、(v-1) より `ρ` はそれを `P` までに通る),
       L10 (c2) (`ρ` はその位置へ進む前に `α` が選んだアーム本体の終端の `Ret` を通り、`x` はその `Ret` が
       名指す変数の値を持つ),
       D3 (`Let(x, Match(v, arms), k)` ではアームを 1 つ選ぶ), D21 (活性化が選ぶアームは決まっている),
       D2 (`Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、`Ret(v)` はその式の値が `v` であることを
       述べる), L10 (b) (`x` の値は値を得た後は変わらない),
       D9 の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の束縛変数の値は `x` の
       値である。」),
       CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
       CODE src/rc_ir/ownership.rs: returned_var
  <2>1a. `r_0` は `P` で値を持つ (DEF-0)。
    BY L11, DEF-1 の段 E2 (`x` の束縛は `Join(rs)` であり、`r_0` は `α` が選んだアームの結果である), 補題 Q の前提 (`x` は `P` で値を持つ)
  <2>2. `ty(r_0) = ty(x)` であり、`λ` は `ty(r_0)` の boxed leaf で `P` で inhabited である。
    BY A12 (アームの結果と `Match` の束縛変数の型が一致する), <2>1, <2>1a, L10 (b), D16 -- <2>1 より
       `P` における `x` の値は `r_0` の値なので、`λ` が通る各 unbox union の節のタグも同じである。
  <2>2a. `r_0` の値の leaf `λ` が指すオブジェクトは、`x` の値の leaf `λ` が指すオブジェクトと同じで
         ある。
    BY <2>1, <2>2, <2>1a, D9 の移動の表の値の水準の行 (「`Match` のアーム本体の `Ret(x)`: `Match` の
       束縛変数の値は `x` の値である。」)
  <2>2b. (H) を仮定すると、そのオブジェクトは計数下である。よって `r_0` は DEF-0 の (v-1) か (v-2) で
         あり、`(r_0, λ)` は `P` のスロットである。
    BY <2>2a, <2>1, <2>2, <2>1a, (H), L13 (対偶), D26 (計数下とグローバル状態は排他である), D6
  <2>3. (H) の下で、`(x, λ)` と `(r_0, λ)` はそれぞれちょうど 1 つの参照を持ち、2 つは同一の参照である。
    BY <2>2b, L12, <2>1,
       D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行 (`x` の参照が `Match` の束縛変数へ), D8
  <2>4. `C_π := ∪_{r ∈ rs} act(r, π)` とおくと、`origin(x, π) = of_candidates(C_π, (x, π))` であり、
        `cand(x, π) ⊇ cand(r_0, π)`。
    <3>1. `origin(x, π) = of_candidates(C_π, (x, π))`。
      BY L4 (a), CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕,
         L15 (`origin(・, ・)` の記法は鍵の答えを指す)
    <3>2. `C_π` は空でない。
      BY A9 (`Match` は 1 つ以上のアームを持つ), L2 (a) (`act` は `identity` を含むので空でない)
    <3>3. `|C_π| ≥ 2` のとき `cand(x, π) = C_π ⊇ act(r_0, π) ⊇ cand(r_0, π)`。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, L2 (a)
    <3>4. `|C_π| = 1` のとき、`C_π = {z}` とおくと `cand(x, π) = {z}` であり、
          `cand(r_0, π) ⊆ act(r_0, π) ⊆ C_π = {z}` である。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, L2 (a),
         L2 (b)
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4 -- `C_π` は空でない (<3>2) ので、`|C_π| ≥ 2` と `|C_π| = 1` の 2 つで
         尽きている。
  <2>5. QED
    BY <2>2, <2>2a, <2>2b, <2>3, <2>4, <2>1a, <1>0, 帰納法の仮定を `(r_0, π)` に適用 -- DEF-1 の E2 の
       次の 3 つ組は `(r_0, π, λ)` である。帰納法の仮定の前提は <2>1a、<2>2 が与え、その (iii) の
       前提となる `(r_0, λ)` についての (H) は <2>2b が与える。(i) は <2>4 が `cand(x, π)` の元にする。
       (ii) は帰納法の仮定の (ii) と <2>2a を繋いだものである。(iii) は (H) の下で <2>3 と帰納法の
       仮定の (iii) の推移であり、`(x, λ)` がスロットであることは <1>0 が与える。

<1>10. QED
  <2>1. `x` の名前が `vars.bindings` に持つ束縛による場合分けは、DEF-1 の段 E1、E2、E3、E4a、E5、E6、E7
        と停止条件 S1、S2 で尽きている。
    <3>1. `origin_inner` の `match` の腕は 6 本で、`None` と `Binding` の 7 構成子の 8 つの場合を
          尽くしている。
      BY L6
    <3>2. `None`、`Param`、`Producer` の腕は `here()` を返す (S1 の H1、H2、H3)。`Move(y)` は E1、
          `Join(rs)` は E2 である。`Field(c, i)` は `c` が boxed なら `here()` (S1 の H4)、そうでなければ
          E5 である。`Payload(s, None)` は E6、`Payload(s, Some(t))` は `s` が boxed なら `here()`
          (S1 の H5)、そうでなければ E7 である。
      BY L9, L6, CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `Llvm` の腕は、`leaf_origins_at(π)` が単一の `Arg(j, σ)` であれば E3 である。そうでないとき、
          `λ` は `ty(x)` の boxed leaf なので `decl` に宣言を持ち、その宣言は単一の `Arg` (E4a)、単一の
          `Fresh` または単一の `Unknown` (S2)、空集合、要素数 2 以上のいずれかである。
      BY L8 (a), L8 (b), L8 (c), L8 (d), 補題 Q の前提 (`λ` は `ty(x)` の boxed leaf である)
    <3>4. 空集合と要素数 2 以上は起きない。
      BY A3 (空集合と宣言された leaf は inhabited にならない), 補題 Q の前提 (`λ` は `P` で inhabited で
         ある), L7 (要素数 2 以上の宣言はこのプログラムに無い)
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4
  <2>2. 鎖は有限で止まる。
    BY <1>1, <1>2 (停止条件では鎖の長さは 0 である), <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9 (段では
       帰納法の仮定が次の 3 つ組からの鎖の有限性を与える), L14 (再帰呼び出しの関係は整礎である)
  <2>3. QED
    BY <2>1, <2>2, <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9 -- <2>1 の 9 つの場合が
       上の 9 つの CASE であり、どの場合も (i)、(ii)、(iii) が成り立つ。<1>0 が、(iii) のうち
       `(x, λ)` が `P` のスロットであるという部分をすべての場合について与える。

**系 1 (P3)**。`origin(x, π) = Exactly(u, σ)` とする。すべての活性化、それが辿る実行路、およびその上の
すべての位置 `P` において、`x` が `P` で値を持ち (DEF-0)、`λ` が `π` の下の `P` で inhabited な leaf で
あるとき、次の 2 つが成り立つ。

- **(a)** `λ` に対応する位置 (DEF-1) の変数は `u`、その path は `σ` の下の leaf であり、その位置は
  `(x, λ)` と**同じオブジェクトを指す**。
- **(b)** さらに**スロット `(x, λ)` が `P` で D8 の意味の参照を持つとき**、対応する位置は `P` の
  スロットである。これが D17 の「対応するスロット」であり、それは `(x, λ)` と同一の参照を持つ。

(b) の限定の形は README の P3 の言明と同じであり、(a) が README の P3 の「**条件を外した形では、
2 つのスロットは同じオブジェクトを指す。**」に当たる。**(a) の対応する位置はスロットとは限らない** --
鎖は `vars.bindings` に束縛を持たない名前で止まりうるので、そのとき対応する位置は D6 の**記号の位置**で
ある。D6 が「スロットと記号の位置を合わせて**位置**と呼ぶ」と定め、「位置を主語にする定義は記号の位置を
含み、参照を主語にする定義は含まない」と述べるのがこの区別である。

<1>1. `cand(x, π) = {(u, σ)}`。
  BY 前提, L2 (b)
<1>2. 補題 Q の前提が満たされる。
  BY 系 1 の前提 (`x` は `P` で値を持ち、`λ` は `P` で inhabited な `ty(x)` の boxed leaf である)
<1>3. (a)。
  BY <1>1, <1>2, 補題 Q -- Q の (i) より停止点の `VarPath` は `(u, σ)` であり、(ii) よりその位置は
     `P` で値を持つ `u` と `μ ⊒ σ` の leaf からなり、`(x, λ)` と同じオブジェクトを指す。
<1>4. QED
  BY <1>1, <1>2, <1>3, L12 (`(x, λ)` が D8 の意味の参照を持つとき `obj(x, λ)` は計数下であり、これが
     Q の (H) である), 補題 Q -- (a) は <1>3、(b) は Q の (iii) が与える。

**系 2 (P4)**。`origin(x, π) = Join { identity, candidates }` とする。すべての活性化、それが辿る実行路、
およびその上のすべての位置 `P` において、`x` が `P` で値を持ち (DEF-0)、`λ` が `π` の下の `P` で
inhabited な leaf であるとき、次の 2 つが成り立つ。

- **(a)** `λ` に対応する位置 (DEF-1) の `VarPath` は `candidates` のいずれかの下にあり、その位置は
  `(x, λ)` と**同じオブジェクトを指す**。
- **(b)** さらに**スロット `(x, λ)` が `P` で D8 の意味の参照を持つとき**、対応する位置は `P` の
  スロットである。これが D17 の「対応するスロット」であり、それは `(x, λ)` と同一の参照を持つ。

(b) の限定の形は README の P4 の言明と同じであり、(a) の位置がスロットとは限らないことは系 1 と同じ
理由による。

<1>1. `cand(x, π) = candidates`。
  BY 前提, CODE src/rc_ir/ownership.rs: Origin::candidates
<1>2. 補題 Q の前提が満たされる。
  BY 系 2 の前提 (`x` は `P` で値を持ち、`λ` は `P` で inhabited な `ty(x)` の boxed leaf である)
<1>3. (a)。
  BY <1>1, <1>2, 補題 Q -- Q の (i) より停止点の `VarPath` は `candidates` の元であり、(ii) の
     `μ ⊒ σ_end` より対応する位置はその元の下にあって、`(x, λ)` と同じオブジェクトを指す。
<1>4. QED
  BY <1>1, <1>2, <1>3, L12 (`(x, λ)` が D8 の意味の参照を持つとき `obj(x, λ)` は計数下であり、これが
     Q の (H) である), 補題 Q -- (a) は <1>3、(b) は Q の (iii) が与える。

**候補集合が広いことは Q を弱めない。** 補題 Q の証明が候補集合について使うのは「`cand(x, π)` が内側の
候補を**含む**」という向きだけであり、使う位置は `<1>2` の `<2>2`、`<1>8` の `<2>2`、`<1>9` の `<2>4` の
3 か所である。`of_candidates` に渡る集合は畳み込む各 `Origin` の `acted_on()` の和であり、`act ⊇ cand`
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
`VarPath` が呼び出し自身の `(var, path)` とは限らないことの実例である (L9 の `<2>3a` の `<3>1b`)。

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

**この食い違いに依拠する読み手は無い。** `borrow_ify` と `cancel` が `origin` を呼ぶ位置のうち、leaf で
ない path を渡しうるのは次の 4 つであり、どれも答えの候補が名指す**根**について `owns_object` /
`owns_object_yet` か `used_later` を引くだけで、leaf の `identity` を unit の答えから引かない。

- `RewriteCtx::owns_unit` は候補すべてに `owns_object` を要求する
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::owns_unit`)。
- `RewriteCtx::check_ownership_is_levelled` は候補の `owns_object` が揃うことを表明する
  (`CODE src/rc_ir/borrow.rs: RewriteCtx::check_ownership_is_levelled`)。
- `routing_saves_retain` は `comes_from_a_value_used_later` を通じて候補の根に `used_later` を引く
  (`CODE src/rc_ir/borrow.rs: routing_saves_retain`)。
- `level_ownership` は候補の根の所有を読み、所有の側へ倒す
  (`CODE src/rc_ir/borrow.rs: level_ownership`)。

`cancel` の走査は unit の path で `origin` を問わない。`Retain`/`Release` が触れる先は
`acted_references` と `CancelAnalysis::other_objects` が leaf ごとに `origin` を問うて作り
(`CODE src/rc_ir/ownership.rs: acted_references`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)、
消費の側は `rhs_consumes` と `destructure_consumes` が報告する leaf ごとに `CancelAnalysis::consume` が
`origin` を問う (`CODE src/rc_ir/ownership.rs: rhs_consumes`, `destructure_consumes`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`)。P5 (c) が結ぶ 2 つの量はどちらも leaf ごとの量で
あり、unit の側の答えを読まない。unit の答えと leaf の答えの食い違いを扱うのは P7a であり、その言明は
`levelled_sites` の site と `infer_ownership` の不動点に限り、かつ inhabited (D16) な leaf に限る形で
それを述べる。

## 8. `level_ownership` が P3 と P4 に及ぼすもの

`level_ownership` は `infer_ownership` の不動点の中で走る段である
(`CODE src/rc_ir/borrow.rs: infer_ownership`, `levelled_sites`, `level_ownership`)。

<1>1. P3 と P4 の言明が読む関数は `origin` であり、D17 の対応するスロットを決めるのは `origin_inner` と
      `origin_from_leaves_under` である。
  BY README の P3 と P4 の言明, D13, D17
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
<1>3. `level_ownership` が書くのは 2 つである -- `infer_ownership` の局所変数 `owned_leaves` と、
      それが呼ぶ `origin` を通じて `VarTable` の `origins` の memo である。`VarTable` の残りの欄
      (`bindings`、`closure_targets`、`param_tys`、`var_tys`) にも `TypeEnv` にも書かない。
  BY CODE src/rc_ir/borrow.rs: level_ownership -- 引数は `&VarTable`、`&TypeEnv`、site、
     `&mut Set<VarPath>` であり、自分が行う書き込みは `owned_leaves.insert` だけである。呼ぶのは
     `origin`、`owns_object_yet`、`covered_leaves` の 3 つである,
     CODE src/rc_ir/borrow.rs: owns_object_yet (`&Set<VarPath>` を読むだけで、どこにも書かない),
     CODE src/rc_ir/ownership.rs: origin -- `vars.origins.borrow_mut().insert(key, answer.clone())` が
     memo を書く。`origins` は `RefCell` なので `&VarTable` からでも書ける,
     CODE src/rc_ir/ownership.rs: VarTable -- 5 つの欄のうち `RefCell` を持つのは `origins` だけである
<1>3a. memo への書き込みは鍵の答えを変えない。
  <2>1. 鍵の答えは、その鍵についての cold な呼び出しの `origin_inner` が返した値であり、memo が当たる
        呼び出しはその複製を返す。すなわち memo は答えを決めるのではなく、決まった答えを配る。
    BY L15, L1 (複製は `identity` と `candidates` をそのまま運ぶ)
  <2>2. `origin_inner` が返す値は、`vars.bindings`、`type_env`、`bindings` が持つ `LLVMGen` の
        `result_prov` の返り値、および `origin` の再帰呼び出しが返す鍵の答えだけで決まる。
    BY <1>2, A3 (`result_prov` の呼び出しは abort せず `Provenance` を返す),
       CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under, as_arg_projection,
       truncate_to_unit
  <2>3. QED
    BY <2>1, <2>2, <1>3 (`level_ownership` は <2>2 が挙げるもののどれにも書かない), L14 -- 鍵の再帰の辺の
       関係は整礎である (L14)。その関係の上の整礎帰納で、どの鍵の答えも `level_ownership` が memo を
       埋めたかどうかで変わらない。
<1>4. QED
  BY <1>1, <1>2, <1>3, <1>3a -- `owned_leaves` は <1>2 が挙げる入力に入らず、memo への書き込みは答えを
     変えない (<1>3a) ので、P3 と P4 の真偽は `level_ownership` の有無で変わらない。

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
