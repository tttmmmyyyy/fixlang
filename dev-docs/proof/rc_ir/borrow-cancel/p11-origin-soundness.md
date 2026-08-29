# P3 / P4 / P5 (c) -- `origin` の健全性と不変条件 (N)

対象は、README の対象コミット `b81cc2c8e859a00cbf007e4f43483a514c813c73` に `be26b396` (PR #531、#529 の
修正) を加えたものである。読んだのは作業ブランチ `proof-critic-round1` の `dff7d934` の版で、そこには
`be26b396` が入っている。定義・仮定・命題の番号は同ディレクトリの `README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P3 (`origin` の健全性 -- `Exactly`) | 証明した (第 6 節の系 1) |
| P4 (`origin` の健全性 -- `Join`) | 証明した (第 6 節の系 2) |
| P5 (c) = 不変条件 (N) | **閉じない。命題は偽であり、反例を第 8 節に置く** |

P3 と P4 は、1 つの補題 Q (第 6 節) の 2 通りの読みである。Q は `origin` が辿る別名の辺を 1 本ずつ D9 の
移動の表と A3 の宣言に突き合わせる帰納法で示す。#529 の修正はこの 2 つを弱めない -- `acted_on` は
`candidates` を含む (第 4 節の L2) ので、候補集合が増えることは P4 の存在主張を易しくするだけである。

P5 (c) は閉じない。帰納法は 1 つの場合を除いて回り、残った場合には反例がある。止まる場所は
`origin_inner` の `Binding::Join` の腕で、**unit の path では候補が 1 つに畳まれて `Exactly` になるのに、
その下の leaf の path では候補が 2 つ以上あって `Join` になる**ときである。このとき leaf の側の答えは
`(v, λ)` という新しい名前を identity に持ち、その名前は unit の側の答えのどこにも現れない。第 8 節の反例は
`origin` の静的な計算だけで閉じており、第 9 節に、その形が実在の Fix プログラムから出ることを、出力させた
RC IR で示す。

第 10 節に、cancel の側で何がこの取りこぼしを埋めているかを書く。**この反例から miscompile を作ることは
できていない**。埋めているのは `References` の多重集合 (`covers` が成り立たないので対にならない) と、
leaf の path での `acted_unit_keys` (そちらは取りこぼさない) の 2 つである。よって「(N) は偽」までが
この文書の主張であり、「コードが誤っている」はまだ主張しない。第 11 節に、(N) を真にするために取りうる
2 つの向きを書く。

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`VarPath` を `(x, π)` と書く。
`ty(x)` は `vars.var_tys` が `x` に記録する型である (`CODE src/rc_ir/ownership.rs: VarTable`)。

- `leaves(τ)` は `boxed_leaf_paths(τ)`、`leaves(τ, π)` は `π` で始まる `leaves(τ)` の要素とする
  (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `LeafMap::leaves_under`)。
- `t_τ(p)` は `truncate_to_unit(τ, p)` とする (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。
- `id(v, π)` は `origin(v, π).identity()`、`cand(v, π)` は `origin(v, π).candidates()` の集合、
  `act(v, π)` は `origin(v, π).acted_on()` の集合とする。
- `K(v, π)` は `acted_unit_keys(v, π)` の要素の集合とする
  (`CODE src/rc_ir/ownership.rs: acted_unit_keys`)。
- `p ⊒ q` は「`p` が `q` を接頭辞として持つ」とする。

`Origin` の構成子と読み出しは次のとおりである (`CODE src/rc_ir/ownership.rs: Origin`, `Origin::identity`,
`Origin::candidates`, `Origin::acted_on`, `Origin::of_candidates`)。

- `Exactly(p)`: `identity() = p`、`candidates() = [p]`、`acted_on() = [p]`。
- `Join { identity, candidates }`: `identity() = identity`、`candidates() = candidates`、
  `acted_on() = [identity] ++ (candidates` から `identity` を除いたもの`)`。集合として
  `acted_on() = {identity} ∪ candidates` である。
- `of_candidates(C, h)`: `C` が 1 要素ならその要素の `Exactly`、2 要素以上なら
  `Join { identity: h, candidates: C }`。`C` が空なら panic する。

`be26b396` が変えたのは 2 行だけである。`origin_inner` の `Binding::Join` の腕と
`origin_from_leaves_under` の末尾が、内側の `Origin` を畳むときに `candidates()` ではなく `acted_on()` を
使う (`CODE src/rc_ir/ownership.rs: origin_inner` の `Some(Binding::Join(..))` の腕、
`origin_from_leaves_under` の `let candidates = reached.iter().flat_map(..)`)。

## 2. 突き合わせ 1 -- D9 の「移動」と `origin_inner` の別名の辺

**別名の辺**とは、`origin_inner` が `origin` を再帰呼び出しする先をいう
(`CODE src/rc_ir/ownership.rs: origin_inner`)。

<1>1. `origin_inner` の `match` の腕は 7 つで尽きている。
  <2>1. `vars.bindings.get(var)` は `Option<&Binding>` を返す。
    BY CODE src/rc_ir/ownership.rs: VarTable の `bindings` フィールド
  <2>2. `Binding` の構成子は `Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join` の 7 つで
        ある。
    BY CODE src/rc_ir/ownership.rs: Binding
  <2>3. QED
    BY <2>1, <2>2 -- 腕は `None | Some(Param) | Some(Producer)` を束ねた 1 本と、残り 6 本。

<1>2. 別名の辺は次の 7 つで尽きている。
  BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner

| 辺 | 腕 | 行き先 |
|---|---|---|
| E1 | `Move(y)` | `origin(y, π)` |
| E2 | `Join(rs)` | 各 `r` in `rs` について `origin(r, π)` |
| E3 | `Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)` | `origin(args[j], σ)` |
| E4 | `Llvm` かつ E3 でない | `π` の下の各 leaf の宣言の各 `Arg(j, σ')` について `origin(args[j], t_{ty(args[j])}(σ'))` |
| E5 | `Field(c, i)` かつ `c` が unbox | `origin(c, [i] ++ π)` |
| E6 | `Payload(s, None)` | `origin(s, π)` |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `origin(s, [t] ++ π)` |

E4 の中身は `origin_from_leaves_under` である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。

D9 の移動の 6 行との対応:

| D9 の移動 | 辺 |
|---|---|
| `Let(x, Var(y), k)` | E1 |
| `Match` のアーム本体の `Ret(x)` | E2 (`Binding::Join` はアーム本体の `Ret` の変数を集める。`CODE src/rc_ir/ownership.rs: collect_bindings` の `RcRhs::Match` の腕、`returned_var`) |
| unbox 容器の `Destructure` の名前付きフィールド | E5 (`collect_bindings` の `RcExpr::Destructure` の腕は名前付きフィールドにだけ `Binding::Field` を作る) |
| unbox union の変位アームの payload 束縛 | E7 |
| catch-all アームの payload 束縛 | E6 |
| `Llvm` の素通し leaf (`result_prov` が単一の `Arg(i, σ)`) | E3 (`as_arg_projection` が集合の要素数 1 を要求する。`CODE src/rc_ir/ownership.rs: as_arg_projection`) |

**E4 は D9 の移動の表に対応する行を持たない。** E4 は `π` の下の leaf 群をまとめて 1 つの答えにする段であり、
その答えは leaf ごとの移動ではない。E4 を leaf ごとに分解すると D9 と A3 に合う -- それが第 5 節の DEF-1 で
ある。E4 が答えを作る規則そのものの性質は第 4 節の L3 と L4 に置く。

### 2.1 `Llvm` の腕を宣言の形で場合分けする

A3 は `result_prov` が leaf ごとに `LeafOrigins` (`Set<LeafOrigin>`) を返すとし、空集合・単一の
`Arg`・単一の `Fresh`・単一の `Unknown`・複数元の 5 行を持つ。`origin_inner` の `Llvm` の腕がその 5 つを
どう扱うかを書き下す。以下 `decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env)` とする。

<1>1. `decl.leaf_origins_at(π)` の値は次の 5 つで尽きている。`None`、`Some` の空集合、`Some` の単一の
      `Arg(j, σ)`、`Some` の単一の `Fresh` または単一の `Unknown`、`Some` の要素数 2 以上。
  <2>1. `leaf_origins_at(π)` は、`π` が `decl` の記録する leaf でなければ `None`、そうでなければその leaf の
        `LeafOrigins` を `Some` で返す。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>2. `LeafOrigins` は `Set<LeafOrigin>` であり、`LeafOrigin` の構成子は `Fresh`、`Unknown`、`Arg` の
        3 つである。
    BY CODE src/rc_ir/provenance.rs: LeafOrigin, LeafOrigins
  <2>3. QED
    BY <2>1, <2>2 -- 集合を要素数 0、1、2 以上で分け、要素数 1 を構成子で分けた。

<1>2. `as_arg_projection(sources)` が `Some` を返すのは <1>1 の第 3 の場合だけである。
  BY CODE src/rc_ir/ownership.rs: as_arg_projection -- `sources.len() != 1` で `None`、要素が `Fresh` か
     `Unknown` でも `None`。

<1>3. 第 3 の場合、答えは `origin(args[j], σ)` である (辺 E3)。これは D9 の移動の表の `Llvm` の行と
      A3 の「単一の `Arg(j, σ)`」の行に一致する。
  BY <1>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `Some((j, σ))` の枝

<1>4. 残る 4 つの場合は `origin_from_leaves_under(vars, type_env, &decl, args, π, &here_identity)` に入り、
      それぞれ次の答えになる。
  <2>1. `None` の場合。`π` は `ty(v)` の boxed leaf ではない。`leaf_origins_under(π)` は `π` で始まる各 leaf
        の宣言を返し、以下の 3 つの場合がその各 leaf について適用される。
    BY <1>1 の <2>1, CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under
  <2>2. 空集合の場合。`for src in sources` のループは 1 度も回らないので、その leaf は `operand_units` にも
        `produced_here` にも寄与しない。`π` の下の leaf の宣言がすべて空集合であるときは `reached` が空に
        なり、`reached.first()?` が `None` を返して、`origin_inner` の `unwrap_or_else(here)` が `here()` を
        答える (第 3 節の H7)。A3 の空集合の行よりこの leaf は inhabited にならないので、この答えが名付ける
        参照は存在しない。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for sources` と `reached.first()?`,
       CODE src/rc_ir/ownership.rs: origin_inner の `None =>` の枝, A3, D16
  <2>3. 単一の `Fresh` または単一の `Unknown` の場合。`produced_here` が真になり、`Exactly(here)` が
        `reached` に積まれる。A3 の対応する 2 行はどちらも新しい参照なので、D10 の生成の `Llvm` の行に
        一致する。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `LeafOrigin::Fresh | LeafOrigin::Unknown`
       の腕と `produced_here`, A3, D10 の生成の表
  <2>4. 要素数 2 以上の場合。ループは要素ごとに回り、`Arg(j, σ')` は `operand_units` に入って別名として
        辿られ、`Fresh` と `Unknown` は `produced_here` を立てる。A3 の複数元の行は「いずれの路でも
        新しい参照」なので、`Arg` を別名として辿るのはこの行と食い違う。ただしこの場合は 2.2 より
        現在のプログラムには存在しない。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for src in sources` のループ, A3, 2.2
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

### 2.2 複数元の宣言は現在のプログラムに存在しない

この事実は第 5 節の DEF-1 と第 6 節の補題 Q が使う。leaf ごとの宣言の要素数が 1 以下でなければ、leaf の
辿る先が 1 つに決まらず、DEF-1 の鎖が定義できない。

<1>1. `LLVMGen` の実装は 78 個あり、そのうち 29 個が `result_prov` を override し、49 個は既定を使う。
  BY CODE src/fixstd/builtin.rs の `impl LLVMGen for` (78 個、すべてこのファイルにある),
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
     `origin_inner` が読むのは `llvm_gen.result_prov(..)` の返り値そのもの、すなわち宣言なので、2.1 の
     <1>4 の <2>4 の場合には到達しない。

## 3. 突き合わせ 2 -- D10 の「生成」と `here()` の腕

`here()` は `Origin::Exactly((var, path))` を返す閉包である
(`CODE src/rc_ir/ownership.rs: origin_inner` の先頭)。`here()` と同じ答えに着く道は 7 つある。

| 道 | 着き方 | D10 での位置 |
|---|---|---|
| H1 `None` (表に無い名前) | 直接 | A8 (グローバルは線形規律の外) |
| H2 `Param` | 直接 | D10 の初期値 |
| H3 `Producer` | 直接 | 生成の表の `App` の行と `Closure` の行 (`collect_bindings` は `RcRhs::App` と `RcRhs::Closure` にだけ `Binding::Producer` を作る) |
| H4 `Field(c, i)` かつ `c` が boxed | 直接 | 生成の表の boxed 容器の `Destructure` の行 |
| H5 `Payload(s, Some(t))` かつ `s` が boxed | 直接 | 生成の表の boxed union の変位アームの行 |
| H6 `Llvm` かつ `π` の下のある leaf の宣言が `Fresh` か `Unknown` を含む | `origin_from_leaves_under` が `produced_here` を立てて `Exactly(here)` を `reached` に積む。`reached` が 1 要素ならそれが答え | 生成の表の `Llvm` の行 |
| H7 `Llvm` かつ `π` の下の leaf の宣言がすべて空集合 | `origin_from_leaves_under` が `None` を返し `unwrap_or_else(here)` | A3 より、その leaf は inhabited にならない。D10 の生成は起きない |

D10 の生成の 5 行はすべて `here()` の道を持つ (H3 が 2 行、H4、H5、H6)。逆向きには `here()` の方が 3 つ多く、
H1 は A8 が規律の外に置くもの、H2 は D10 の初期値、H7 は inhabited な leaf を持たない位置である。どれも
「新しい参照を作る」とは主張していないので、生成の表に無いことと矛盾しない。

## 4. 補題

以下の補題は、この文書のすべての証明が使う。

**L1 (`Origin::Join` は `of_candidates` だけが作る)**

<1>1. `Origin::Join { .. }` を値として作る式は `of_candidates` の中の 1 か所だけである。
  BY CODE src/rc_ir/ownership.rs: Origin (宣言), Origin::identity (パターン), Origin::candidates
     (パターン), Origin::of_candidates (唯一の構成)

<1>2. QED
  BY <1>1

**L2 (`Join` の候補は 2 つ以上、`acted_on` は `candidates` を含む)**

<1>1. 任意の `Origin` の値 `o` について `act(o) = {id(o)} ∪ cand(o)` である。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on -- `identity` を先頭に置き、`candidates` から
     `identity` に等しいものを除いたものを続ける。

<1>2. `o` が `Join` ならば `|cand(o)| ≥ 2` であり、よって `|act(o)| ≥ 2` である。
  BY L1, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- `Join` を作る枝は `candidates.len()` が
     1 でない枝であり、`candidates` が空なら手前の `assert!` が panic する。<1>1 より
     `act(o) ⊇ cand(o)`。

<1>3. `o` が `Exactly(p)` ならば `act(o) = cand(o) = {p}` である。
  BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates, Origin::acted_on

<1>4. QED
  BY <1>1, <1>2, <1>3 -- どちらの構成子でも `act(o) ⊇ cand(o)`。

**L3 (`of_candidates` の `acted_on` は与えた集合を含む)**: 空でない集合 `C` と `h` について
`act(of_candidates(C, h)) ⊇ C`。

<1>1. `|C| = 1` のとき `of_candidates(C, h) = Exactly(c)` (`C = {c}`) であり、`act = {c} = C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, L2 の <1>3
<1>2. `|C| ≥ 2` のとき `of_candidates(C, h) = Join { identity: h, candidates: C }` であり、
      `act = {h} ∪ C ⊇ C`。
  BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, L2 の <1>1
<1>3. QED
  BY <1>1, <1>2

**L4 (畳み込みは推移的である -- 「入れ子が 2 段以上」の答え)**: `origin_inner` の `Binding::Join` の腕と
`origin_from_leaves_under` が畳み込む先の `Origin` を `o_1, ..., o_k` (`k ≥ 1`) とし、答えを `o` とすると、
`act(o) ⊇ act(o_1) ∪ ... ∪ act(o_k)` である。よって `act(o)` は、畳み込みの木のどの深さに現れる `Join` の
`identity` も含む。

<1>1. `Binding::Join` の腕の答えは `of_candidates(∪_i act(o_i), (var, path))` である。
  BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕 -- ループは
     `origin(..).acted_on()` の各要素を `candidates` に入れる。
<1>2. `origin_from_leaves_under` の答えは、`reached` の全要素が等しいときはその要素そのもの、
      そうでないときは `of_candidates(∪_i act(o_i), here)` である。
  BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `if reached.iter().all(..)` の枝と
     その後の `flat_map(|reached_origin| reached_origin.acted_on())`
<1>3. <1>1 と <1>2 の後者の場合、`act(o) ⊇ ∪_i act(o_i)`。
  BY <1>1, <1>2, L3
<1>4. <1>2 の前者の場合、`o = o_1 = ... = o_k` なので `act(o) = ∪_i act(o_i)`。
  BY <1>2
<1>5. QED
  BY <1>3, <1>4 -- どちらの場合も `act(o) ⊇ ∪_i act(o_i)`。`act(o_i) ∋ id(o_i)` (L2 の <1>1) なので、
     `o_i` が `Join` ならその `identity` は `act(o)` に入る。これを畳み込みの木の深さについて繰り返せば、
     どの深さの `Join` の `identity` も `act(o)` に入る。

**L5 (leaf は互いに比較不能である)**: 型 `τ` の相異なる 2 つの boxed leaf の一方が他方の接頭辞になることは
無い。

<1>1. `boxed_leaf_paths` の走査は、`is_box` / `is_array` が真の型と closure の capture の位置で path を
      積んで戻り、その下へは降りない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- `is_closure`、`is_box`、`is_array` の 3 つの枝は
     いずれも `out.push` の直後に `return` する。
<1>2. QED
  BY <1>1 -- leaf が積まれる位置の下は走査されないので、leaf の真の延長が leaf になることは無い。

**L6 (unit-closed な path)**: 型 `τ` の path `π` が **unit-closed** であるとは、`t_τ(π) ∈ rc_units(τ)` で
あり、かつ `leaves(τ, π)` のすべての要素 `λ` について `t_τ(λ) = t_τ(π)` であることをいう。

<1>1. `π ∈ rc_units(τ)` ならば `π` は unit-closed である。
  <2>1. `rc_units_go` が `π` を積むのは `unit_step` が `Unit` を返した位置か `Capture` を返した位置で
        あり、`π` の真の接頭辞の各位置では `unit_step` は `Fields` を返している。
    BY CODE src/rc_ir/ownership.rs: rc_units, rc_units_go
  <2>2. `t_τ(π) = π`。
    BY <2>1, CODE src/rc_ir/ownership.rs: truncate_to_unit -- `Fields` の枝は添字を積んで降り、`Unit` の
       枝は break し、`Capture` の枝は添字を積んで break する。`π` の添字を順に処理すると、最後の位置まで
       `Fields` で降りて `π` を積み切るか (`Unit` の場合)、最後の添字が capture で break する
       (`Capture` の場合) かのどちらかである。
  <2>3. `λ ∈ leaves(τ, π)` ならば `t_τ(λ) = π`。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: truncate_to_unit -- `λ ⊒ π` なので走査は `π` の添字を
       同じ順に処理し、`π` の位置で `Unit` に当たれば break して `π` を返す。`Capture` の場合、
       `boxed_leaf_paths` は closure の下に capture の位置 1 つしか leaf を作らないので `λ = π` である
       (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths` の `is_closure` の枝)。
  <2>4. QED
    BY <2>2, <2>3
<1>2. `π ∈ leaves(τ)` ならば `π` は unit-closed である。
  BY L5 (`leaves(τ, π) = {π}`), P1 (`t_τ(π) ∈ rc_units(τ)`)
<1>3. QED
  BY <1>1, <1>2

**L7 (自分の名前の `unit_of` は path によらない)**: `π` が `ty(v)` で unit-closed であり
`λ ∈ leaves(ty(v), π)` であるとき、`unit_of((v, λ)) = unit_of((v, π)) = (v, t_{ty(v)}(π))`。

<1>1. `vars.var_tys` は `v` の型を記録している。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, collect_bindings -- パラメータと capture、および
     `Let`、`Destructure`、`Match` のアーム payload が束縛する変数のすべてについて `var_tys` に型を
     入れる。
<1>2. `unit_of((v, p)) = (v, t_{ty(v)}(p))` が任意の `p` について成り立つ。
  BY <1>1, CODE src/rc_ir/ownership.rs: unit_of -- `var_tys` に型があるので `Some(ty)` の枝を通り、
     根を変えずに `truncate_to_unit(ty, path, type_env)` を返す。
<1>3. QED
  BY <1>2, L6 -- unit-closed より `t_{ty(v)}(λ) = t_{ty(v)}(π)`。

## 5. DEF-1 -- この文書が使う D17 の読み方

D17 は「`origin` が `(x, π)` から `(u, σ)` へ辿った別名の辺の列を、`π` の下の leaf `λ` について辿ったときに
着く leaf のスロット」を、`λ` に**対応するスロット**と呼ぶ。辺ごとの `λ` の写り方は D17 が表で与えている。
D17 が決めていないものが 3 つあるので、この文書はそれを次のように読む。第 12 節に、README へ足すべき文面
として書き出す。

**DEF-1 (対応の鎖)**。実行路 `ρ` と、`ρ` 上の `x` が束縛されている位置を固定する。3 つ組
`(現在の変数, 現在の path, 現在の leaf)` の列を、`(x, π, λ)` から次の規則で作る。各段の「現在の変数」の
`Binding` が、どの規則を使うかを決める。

| 段 | 条件 | 次の 3 つ組 |
|---|---|---|
| E1 | `Move(y)` | `(y, π_cur, λ_cur)` |
| E2 | `Join(rs)` | `(r_0, π_cur, λ_cur)`。`r_0` は、`ρ` の上でこの `Match` が選んだアームの結果である |
| E3 | `Llvm` かつ `leaf_origins_at(π_cur)` が単一の `Arg(j, σ)` | `(args[j], σ, σ)` |
| E4a | `Llvm` かつ E3 でなく、`λ_cur` の宣言が単一の `Arg(j, σ')` | `(args[j], t_{ty(args[j])}(σ'), σ')` |
| E5 | `Field(c, i)` かつ `c` が unbox | `(c, [i] ++ π_cur, [i] ++ λ_cur)` |
| E6 | `Payload(s, None)` | `(s, π_cur, λ_cur)` |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `(s, [t] ++ π_cur, [t] ++ λ_cur)` |

次の 2 つの場合、列はそこで止まる。

| 停 | 条件 |
|---|---|
| S1 | `origin_inner` が `here()` を答える 5 つの腕 (第 3 節の H1 から H5) |
| S2 | `Llvm` かつ E3 でなく、`λ_cur` の宣言が単一の `Fresh` または単一の `Unknown` |

`λ_cur` の宣言が空集合である場合は、A3 よりその leaf は inhabited でないので、補題 Q の量化から外れる。
`λ_cur` の宣言が 2 元以上である場合は、2.2 より現在のプログラムには存在しない。よってこの表と停止条件は
尽きている。

止まった位置の 3 つ組を `(u, σ_end, μ)` とし、スロット `(u, μ)` を `λ` に**対応するスロット**と呼ぶ。

D17 に対してこの読みが足しているのは次の 3 つである。

1. **E2 の行き先**。`Binding::Join` の辺は行き先を複数持つ。`ρ` が選んだアームの結果を辿ると読む。
2. **S2**。`Llvm` の leaf の宣言が `Fresh` か `Unknown` のとき、その leaf には辿る先が無い。鎖はそこで
   止まり、対応するスロットはその位置の `(u, λ_cur)` である。
3. **E4a の path**。D17 は「`λ` を宣言の `σ'` へ置き換える」と書く。行き先の path はその `σ'` ではなく
   `t_{ty(args[j])}(σ')` であり (`CODE src/rc_ir/ownership.rs: origin_from_leaves_under` の
   `operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)))`)、leaf は `σ'` である。
   `t` は path を降りるだけなので `σ' ⊒ t(σ')` であり、leaf は行き先の path の下に留まる
   (`CODE src/rc_ir/ownership.rs: truncate_to_unit` -- `out` は `path` の接頭辞である)。

## 6. 補題 Q、および P3 と P4

**補題 Q**。`ρ` を実行路、`P` を `ρ` 上の `x` が束縛されている位置、`λ` を `ty(x)` の boxed leaf で
`λ ⊒ π` かつ `P` で inhabited (D16) であるものとする。このとき DEF-1 の鎖は有限で止まり、その停止点
`(u, σ_end, μ)` は次を満たす。

- (i) `(u, σ_end) ∈ cand(x, π)`。
- (ii) `μ ⊒ σ_end` であり、`μ` は `ty(u)` の boxed leaf であって `P` で inhabited である。すなわち
  `(u, μ)` は `P` のスロットである (D6)。
- (iii) スロット `(x, λ)` が持つ参照とスロット `(u, μ)` が持つ参照は同一である (D8)。

証明は、`origin` が `(x, π)` から行う再帰呼び出しの関係の上の帰納法による。P2 よりこの関係は整礎である
(`origin` は停止するので、無限に降りる呼び出しの列は無い)。DEF-1 の各段は `origin_inner` の再帰呼び出しの
1 つに一致するので、鎖の各段で帰納法の仮定が使える。

<1>1. CASE: 停止条件 S1 (`origin_inner` が `here()` を答える)。
  <2>1. `origin(x, π) = Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `None | Some(Binding::Param) | Some(Binding::Producer)`
       の腕、`Some(Binding::Field(..))` の `container.ty.is_box` の枝、`Some(Binding::Payload(..))` の
       `Some(_)` の枝, L2 の <1>3
  <2>2. 停止点は `(x, π, λ)` である。
    BY DEF-1 の S1
  <2>3. QED
    BY <2>1, <2>2 -- (i) は `(x, π) ∈ {(x, π)}`。(ii) は前提の `λ ⊒ π` と、`λ` が `ty(x)` の inhabited な
       boxed leaf であること。(iii) は同じスロットどうしなので自明に同一。

<1>2. CASE: 停止条件 S2 (`Llvm` で `λ` の宣言が単一の `Fresh` または単一の `Unknown`)。
  <2>1. `produced_here` が真になり、`Exactly((x, π))` が `reached` に入る。
    BY 2.1 の <1>4 の <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の
       `if produced_here { reached.push(Origin::Exactly(here.clone())) }` と、`here` に
       `(var.clone(), path.to_vec())` が渡されること (`CODE src/rc_ir/ownership.rs: origin_inner` の
       `let here_identity = (var.clone(), path.to_vec());`)
  <2>2. `(x, π) ∈ cand(x, π)`。
    <3>1. `reached` の全要素が等しいとき、答えは `Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
      BY <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `if reached.iter().all(..)` の枝,
         L2 の <1>3
    <3>2. そうでないとき、答えは `of_candidates(C, (x, π))` であり `C ⊇ act(Exactly((x, π))) = {(x, π)}`。
          `of_candidates` の `candidates()` は `C` そのものである。
      BY <2>1, L4 の <1>2, L3, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. QED
    BY <2>2, DEF-1 の S2 -- 停止点は `(x, π, λ)` なので (i) は <2>2、(ii) は前提、(iii) は同じスロット。

<1>3. CASE: 段 E1 (`Move(y)`)。
  <2>1. `origin(x, π) = origin(y, π)` であり `cand(x, π) = cand(y, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. `ty(y) = ty(x)` であり、`λ` は `ty(y)` の boxed leaf で `P` で inhabited である。
    BY A12 (move-bind の両辺の型が一致する), D16
  <2>3. スロット `(x, λ)` とスロット `(y, λ)` は同じ参照を持つ。
    BY D9 の移動の表の `Let(x, Var(y), k)` の行, D8
  <2>4. 帰納法の仮定を `(y, π)` に適用すると、停止点 `(u, σ_end, μ)` は (i) `(u, σ_end) ∈ cand(y, π)`、
        (ii)、(iii) スロット `(y, λ)` と同じ参照、を満たす。
    BY <2>2, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>3, <2>4 -- (i) は <2>1 で `cand(x, π)` に読み替わり、(iii) は <2>3 と <2>4 の推移。

<1>4. CASE: 段 E6 (`Payload(s, None)`、catch-all)。
  BY <1>3 と同じ形 -- `origin(x, π) = origin(s, π)` (`CODE src/rc_ir/ownership.rs: origin_inner` の
     `Some(Binding::Payload(..))` の `None =>` の枝)、`ty(s) = ty(x)` (A12)、参照の同一は D9 の移動の表の
     catch-all の行、あとは帰納法の仮定を `(s, π)` に適用する。

<1>5. CASE: 段 E5 (`Field(c, i)`、`c` が unbox)。
  <2>1. `origin(x, π) = origin(c, [i] ++ π)` であり `cand(x, π) = cand(c, [i] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(..))` の `else` の枝
  <2>2. `[i] ++ λ` は `ty(c)` の boxed leaf であり、`[i] ++ λ ⊒ [i] ++ π` である。
    BY A12 (`Destructure` のフィールド変数とフィールドの型が合っている、容器が構造体である),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- unbox 集約の枝は
       `unpunched_field_types` の各フィールドへ添字を積んで降りるので、フィールド `i` の leaf は
       `[i] ++ (そのフィールドの型の leaf)` である。
  <2>3. `[i] ++ λ` は `P` で inhabited である。
    BY <2>2, D16 -- `[i]` は unbox 構造体のフィールド添字なので unbox union の節を通らず、`[i] ++ λ` が
       通る union の節は `λ` が通る節と同じである。
  <2>4. スロット `(x, λ)` とスロット `(c, [i] ++ λ)` は同じ参照を持つ。
    BY D9 の移動の表の unbox 容器の `Destructure` の名前付きフィールドの行, D8
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, 帰納法の仮定を `(c, [i] ++ π)` に適用

<1>6. CASE: 段 E7 (`Payload(s, Some(t))`、`s` が unbox)。
  <2>1. `origin(x, π) = origin(s, [t] ++ π)` であり `cand(x, π) = cand(s, [t] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の
       `Some(tag) if !scrut.ty.is_box(type_env)` の枝
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf であり、`[t] ++ λ ⊒ [t] ++ π` である。
    BY A12 (payload と変位の型が合っている、scrutinee が union である),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths, CODE src/ast/types.rs: TypeNode::unpunched_field_types
       -- union の `unpunched_field_types` は各変位の payload の型を返すので、変位 `t` の leaf は
       `[t] ++ (その payload の型の leaf)` である。
  <2>3. `P` において `s` のタグは `t` である。
    BY D9 の移動の表の「unbox union の変位アームの payload 束縛 -- scrutinee の活性変位の参照が payload
       変数へ」の行 -- `P` は変位 `t` のアームの中の位置であり、この行はそのアームで動く参照を
       scrutinee の活性変位のものと述べる。
  <2>4. `[t] ++ λ` は `P` で inhabited である。
    BY <2>2, <2>3, D16 -- `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (タグ `t` で <2>3 に
       より一致する) と、`λ` が通る節 (前提より一致する) である。
  <2>5. スロット `(x, λ)` とスロット `(s, [t] ++ λ)` は同じ参照を持つ。
    BY D9 の移動の表の unbox union の変位アームの payload 束縛の行, D8
  <2>6. QED
    BY <2>1, <2>2, <2>4, <2>5, 帰納法の仮定を `(s, [t] ++ π)` に適用

<1>7. CASE: 段 E3 (`Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)`)。
  <2>1. `π` は `ty(x)` の boxed leaf であり、よって `λ = π` である。
    BY 2.1 の <1>1 の <2>1 (`leaf_origins_at` が `Some` を返すのは `π` が leaf のとき), L5
  <2>2. `origin(x, π) = origin(args[j], σ)` であり `cand(x, π) = cand(args[j], σ)`。
    BY 2.1 の <1>3
  <2>3. `σ` は `ty(args[j])` の boxed leaf であり、`P` で inhabited である。
    BY A3 の「単一の `Arg(j, σ)`」の行 -- 宣言は第 `j` オペランドの leaf `σ` を名指し、結果のその leaf が
       inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることは同値である。前提より
       `λ = π` は `P` で inhabited である。
  <2>4. スロット `(x, π)` とスロット `(args[j], σ)` は同じ参照を持つ。
    BY A3 の同じ行 (「第 `j` オペランドの leaf `σ` と同じ参照。新しい参照を作らない」), D8
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, 帰納法の仮定を `(args[j], σ)` に適用 -- DEF-1 の E3 の次の 3 つ組は
       `(args[j], σ, σ)` であり、`σ ⊒ σ` である。

<1>8. CASE: 段 E4a (`Llvm` かつ E3 でなく、`λ` の宣言が単一の `Arg(j, σ')`)。
  <2>1. `u_j := t_{ty(args[j])}(σ')` とおくと、`origin(args[j], u_j)` は `reached` の要素である。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- `leaf_origins_under(π)` は `λ` の宣言を
       含み、そのループが `operand_units` に `(j, truncate_to_unit(&args[j].ty, σ', type_env))` を入れ、
       `reached` はその各要素の `origin(args[j], unit)` である。
  <2>2. `cand(x, π) ⊇ cand(args[j], u_j)`。
    <3>1. `reached` の全要素が等しいとき、答えは `origin(args[j], u_j)` そのものである。
      BY <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の
         `if reached.iter().all(|reached_origin| reached_origin == first) { return Some(first.clone()) }`
    <3>2. そうでないとき、答えは `of_candidates(C, (x, π))` であり、`C ⊇ act(origin(args[j], u_j))`
          である。`of_candidates` の `candidates()` は `C` そのものであり、`act ⊇ cand` (L2) である。
      BY <2>1, L4 の <1>2, L3, L2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. `σ'` は `ty(args[j])` の boxed leaf であり、`σ' ⊒ u_j` であり、`P` で inhabited である。
    BY A3 の「単一の `Arg(j, σ)`」の行 (leaf であることと inhabited の同値),
       CODE src/rc_ir/ownership.rs: truncate_to_unit (`out` は `path` の接頭辞である)
  <2>4. スロット `(x, λ)` とスロット `(args[j], σ')` は同じ参照を持つ。
    BY A3 の同じ行, D8
  <2>5. QED
    BY <2>2, <2>3, <2>4, 帰納法の仮定を `(args[j], u_j)` に適用 -- DEF-1 の E4a の次の 3 つ組は
       `(args[j], u_j, σ')` である。帰納法の仮定の (i) は `cand(args[j], u_j)` の元を与え、<2>2 が
       それを `cand(x, π)` の元にする。

<1>9. CASE: 段 E2 (`Join(rs)`)。
  <2>1. `ρ` はこの `Match` のちょうど 1 つのアームを通り、`P` における `x` の値はそのアーム本体の `Ret` が
        返した変数 `r_0` の値である。
    BY D3 (`Let(x, Match(v, arms), k)` ではアームを 1 つ選ぶ), D9 の移動の表の
       「`Match` のアーム本体の `Ret(x)`」の行, CODE src/rc_ir/ownership.rs: collect_bindings の
       `RcRhs::Match` の腕と `returned_var` (`Binding::Join` はアーム本体の `Ret` の変数を集める)
  <2>2. `ty(r_0) = ty(x)` であり、`λ` は `ty(r_0)` の boxed leaf で `P` で inhabited である。
    BY A12 (アームの結果と `Match` の束縛変数の型が一致する), <2>1, D16
  <2>3. スロット `(x, λ)` とスロット `(r_0, λ)` は同じ参照を持つ。
    BY <2>1, D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行, D8
  <2>4. `C_π := ∪_{r ∈ rs} act(r, π)` とおくと、`origin(x, π) = of_candidates(C_π, (x, π))` であり、
        `cand(x, π) ⊇ cand(r_0, π)`。
    <3>1. `origin(x, π) = of_candidates(C_π, (x, π))`。
      BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
    <3>2. `C_π` は空でない。
      BY A9 (`Match` は 1 つ以上のアームを持つ), L2 の <1>1 (`act` は `identity` を含むので空でない)
    <3>3. `|C_π| ≥ 2` のとき `cand(x, π) = C_π ⊇ act(r_0, π) ⊇ cand(r_0, π)`。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, L2
    <3>4. `|C_π| = 1` のとき、`C_π = {z}` とおくと `cand(x, π) = {z}` であり、
          `cand(r_0, π) ⊆ act(r_0, π) ⊆ C_π = {z}` である。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates, L2
    <3>5. QED
      BY <3>1, <3>3, <3>4
  <2>5. QED
    BY <2>2, <2>3, <2>4, 帰納法の仮定を `(r_0, π)` に適用 -- DEF-1 の E2 の次の 3 つ組は
       `(r_0, π, λ)` である。

<1>10. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9, DEF-1 -- DEF-1 の段は E1 から E7 の 7 つと
     停止条件 S1、S2 で尽きており (2.2 と A3 が残る 2 つの場合を消す)、それぞれが上のいずれかの CASE で
     ある (E4 は `λ` の宣言の形で E4a と S2 に分かれる)。鎖が有限であることは、各段が `origin` の再帰
     呼び出しの 1 つであり、その関係が整礎である (P2) ことによる。

**系 1 (P3)**。`origin(x, π) = Exactly(u, σ)` のとき、すべての実行路のすべての位置において、`π` の下の
inhabited な各 leaf `λ` について、`obj(x, λ)` を指す参照は `λ` に対応するスロット (D17、DEF-1) が持つ参照と
同一である。

<1>1. `cand(x, π) = {(u, σ)}`。
  BY 前提, L2 の <1>3
<1>2. QED
  BY <1>1, 補題 Q -- Q の (i) より停止点の `VarPath` は `(u, σ)` であり、(ii) よりその位置のスロットで
     あり、(iii) より `(x, λ)` と同じ参照を持つ。

**系 2 (P4)**。`origin(x, π) = Join { identity, candidates }` のとき、各実行路の各位置において、`π` の下の
inhabited な各 leaf のスロットが持つ参照は、`candidates` のいずれかの下の対応するスロット (D17、DEF-1) が
持つ参照と同一である。

<1>1. `cand(x, π) = candidates`。
  BY 前提, CODE src/rc_ir/ownership.rs: Origin::candidates
<1>2. QED
  BY <1>1, 補題 Q -- Q の (i) より停止点の `VarPath` は `candidates` の元であり、(ii) の `μ ⊒ σ_end` より
     対応するスロットはその元の下にあり、(iii) が参照の同一を与える。

**#529 の修正が P3 と P4 に及ぼすもの**。`be26b396` は畳み込みを `candidates()` から `acted_on()` に変えた。
L2 より `act ⊇ cand` なので、`of_candidates` に渡る集合は広くなる。補題 Q の証明で候補集合を使うのは
<1>2 の <2>2、<1>8 の <2>2、<1>9 の <2>4 の 3 か所で、いずれも「`cand(x, π)` が内側の候補を**含む**」と
いう向きにしか使わないので、集合が広くなることは証明を弱めない。よって P3 と P4 は修正の前後どちらでも
成り立つ。

## 7. P5 (c) -- 帰納法と、それが止まる場所

### 7.1 言明の形

P5 (c) は次である。

> **(N)** `acted_unit_keys(v, π)` は、`acted_references(v, π)` が名指すオブジェクトのうち inhabited な
> leaf に由来するものを、`unit_of` で写した上ですべて含む。

`acted_references(v, π)` は `π` で始まる各 boxed leaf `λ` について `id(v, λ)` を数える
(`CODE src/rc_ir/ownership.rs: acted_references`)。`acted_unit_keys(v, π)` は `act(v, π)` の各元を
`unit_of` で写す (`CODE src/rc_ir/ownership.rs: acted_unit_keys`)。よって (N) は次と同じである。

> **(N')** `π` の下の inhabited な各 boxed leaf `λ` について、`unit_of(id(v, λ)) ∈ K(v, π)`。

**`π` に制限が要る。** `unit_of` は `truncate_to_unit` の結果がその型の unit であることを表明する
(`CODE src/rc_ir/ownership.rs: unit_of` の `assert!(units.contains(&truncated), ..)`)。`π` が `ty(v)` の
unit でも leaf でもないとき、この表明は破れうる。たとえば `ty(v)` が boxed な値を 2 つ持つ unbox 構造体で
`π = []` のとき、`rc_units(ty(v)) = [[0], [1]]` であって `t(π) = []` はその要素ではない。このとき (N) は
偽ではなく、**未定義**である。

`acted_unit_keys` の呼び出し元は 2 つで、どちらも `π` を unit か leaf に限っている。

- `CancelAnalysis::walk_inner` の `RcExpr::Release` の腕は、`Release` 節点の path を渡す。A2 と P10 より
  これは `ty(v)` の RC unit である。
- `CancelAnalysis::consume` は `rhs_consumes` と `destructure_consumes` が報告する leaf を渡す
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`, `walk_inner` の `RcExpr::Destructure` の腕)。

以下、`π` は `ty(v)` の RC unit または boxed leaf であるとする。L6 よりこれは unit-closed である。

### 7.2 帰納法

**主張 N(v, π)**。`π` が `ty(v)` で unit-closed (L6) であるとき、実行路 `ρ` の `v` が束縛されている位置
`P` の、`π` の下の `P` で inhabited な各 boxed leaf `λ` について、`unit_of(id(v, λ)) ∈ K(v, π)`。

補題 Q と同じく、`origin` の再帰呼び出しの関係 (P2 より整礎) の上の帰納法で示す。

<1>1. `π` が `ty(v)` の boxed leaf であるときは成り立つ。
  <2>1. `λ = π` である。
    BY L5
  <2>2. QED
    BY <2>1, L2 の <1>1 -- `id(v, π) ∈ act(v, π)` なので `unit_of(id(v, π)) ∈ K(v, π)`。

<1>1a. SUFFICES ASSUME `π` は `ty(v)` の boxed leaf ではない PROVE 主張 N(v, π)。
  BY <1>1

<1>2. CASE: `origin_inner` が `here()` を答える腕 (H1 から H5)。
  <2>1. これらの腕は `path` を読まずに `here()` を答えるので、`origin(v, π) = Exactly((v, π))` かつ
        `origin(v, λ) = Exactly((v, λ))` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `None | Some(Binding::Param) | Some(Binding::Producer)`
       の腕、`Some(Binding::Field(..))` の `container.ty.is_box` の枝、`Some(Binding::Payload(..))` の
       `Some(_)` の枝 -- どの枝も `here()` を返し、`here` は `(var, path)` の `Exactly` である。
  <2>2. `unit_of((v, λ)) = unit_of((v, π))`。
    BY L7
  <2>3. QED
    BY <2>1, <2>2, L2 の <1>3 -- `K(v, π) = {unit_of((v, π))}` であり、
       `unit_of(id(v, λ)) = unit_of((v, λ))` がそれに等しい。

<1>3. CASE: 段 E1 (`Move(y)`) または 段 E6 (`Payload(s, None)`)。
  BY CODE src/rc_ir/ownership.rs: origin_inner の該当する 2 つの枝 (どちらも同じ path で再帰する),
     A12 (両辺の型が一致する), 帰納法の仮定を `(y, π)` あるいは `(s, π)` に適用 -- `origin` の答えが
     `π` でも `λ` でも一致するので `K` と `id` がそのまま移り、`π` の unit-closed も型が同じなので移る。

<1>4. CASE: 段 E5 (`Field(c, i)`、`c` が unbox)。
  <2>1. `origin(v, π) = origin(c, [i] ++ π)` かつ `origin(v, λ) = origin(c, [i] ++ λ)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(..))` の `else` の枝
  <2>2. `leaves(ty(c), [i] ++ π) = { [i] ++ λ' : λ' ∈ leaves(ty(v), π) }` であり、`[i] ++ λ` はその要素で
        `P` で inhabited である。
    BY 第 6 節の <1>5 の <2>2 と <2>3 と同じ計算
  <2>3. `[i] ++ π` は `ty(c)` で unit-closed である。
    <3>1. `t_{ty(c)}([i] ++ p) = [i] ++ t_{ty(v)}(p)` が任意の `p` について成り立つ。
      BY CODE src/rc_ir/ownership.rs: truncate_to_unit, unit_step -- `ty(c)` は unbox 集約なので
         `unit_step` は `Fields` を返し、走査は添字 `i` を積んでフィールドの型 `ty(v)` へ降りる (A12)。
    <3>2. `t_{ty(c)}([i] ++ π) ∈ rc_units(ty(c))`。
      BY <3>1, L6 (`t_{ty(v)}(π) ∈ rc_units(ty(v))`),
         CODE src/rc_ir/ownership.rs: rc_units_go の `UnitStep::Fields` の枝 -- フィールド `i` の unit は
         `[i] ++ (そのフィールドの型の unit)` である。
    <3>3. QED
      BY <3>1, <3>2, <2>2, L6 -- `[i] ++ λ'` の truncate は `[i] ++ t_{ty(v)}(λ') = [i] ++ t_{ty(v)}(π)`。
  <2>4. QED
    BY <2>1, <2>2, <2>3, 帰納法の仮定を `(c, [i] ++ π)` に適用

<1>5. CASE: 段 E7 (`Payload(s, Some(t))`、`s` が unbox)。
  <2>1. `origin(v, π) = origin(s, [t] ++ π)` かつ `origin(v, λ) = origin(s, [t] ++ λ)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の
       `Some(tag) if !scrut.ty.is_box(type_env)` の枝
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf であり `P` で inhabited である。
    BY 第 6 節の <1>6 の <2>2 と <2>4 と同じ計算
  <2>3. `[t] ++ π` は `ty(s)` で unit-closed である。
    BY CODE src/rc_ir/ownership.rs: unit_step (`ty(s)` は unbox union なので `is_union` の枝で `Unit`),
       truncate_to_unit (`Unit` の枝は最初の添字で break する) -- 空でないどの path も `[]` に truncate し、
       `rc_units(ty(s)) = [[]]` (`CODE src/rc_ir/ownership.rs: rc_units_go` の `UnitStep::Unit` の枝) で
       ある。
  <2>4. QED
    BY <2>1, <2>2, <2>3, 帰納法の仮定を `(s, [t] ++ π)` に適用

<1>6. CASE: 段 E4 (`Llvm` かつ E3 でない。`π` は leaf ではないので E3 は起きない)。
  <2>1. `λ` の宣言は単一の `Arg(j, σ')` か、単一の `Fresh` か、単一の `Unknown` である。
    BY 2.2 (要素数 2 以上の宣言は存在しない), A3 (空集合の宣言の leaf は inhabited にならない), 前提
       (`λ` は `P` で inhabited)
  <2>2. `reached` は空でなく、`act(v, π) ⊇ act(o)` が `reached` の各要素 `o` について成り立つ。
    BY <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under (`λ` の宣言が `reached` に 1 つ元を
       入れる -- `Arg` なら `origin(args[j], u_j)`、`Fresh`/`Unknown` なら `Exactly((v, π))`), L4
  <2>3. CASE: `λ` の宣言が単一の `Fresh` または単一の `Unknown`。
    <3>1. `origin(v, λ) = Exactly((v, λ))`。
      BY 2.1 の <1>4 の <2>3, L5 (`λ` は leaf なので `leaf_origins_under(λ)` はその 1 つだけを返す),
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- `operand_units` は空、
         `produced_here` は真、`reached = [Exactly((v, λ))]` で全要素が等しいのでそれが答え。
    <3>2. `Exactly((v, π)) ∈ reached` であり、よって `(v, π) ∈ act(v, π)`。
      BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `produced_here` の枝, L2
    <3>3. QED
      BY <3>1, <3>2, L7 -- `unit_of(id(v, λ)) = unit_of((v, λ)) = unit_of((v, π))` であり、これは
         `(v, π) ∈ act(v, π)` より `K(v, π)` の元である。
  <2>4. CASE: `λ` の宣言が単一の `Arg(j, σ')`。
    <3>1. `u_j := t_{ty(args[j])}(σ')` とおくと、`origin(args[j], u_j) ∈ reached` であり
          `act(v, π) ⊇ act(args[j], u_j)`。
      BY <2>2, 第 6 節の <1>8 の <2>1
    <3>2. `origin(v, λ) = origin(args[j], σ')`。
      BY 2.1 の <1>3 -- `λ` は leaf であり、その宣言が単一の `Arg(j, σ')` なので `origin_inner` は
         E3 の枝を通る。
    <3>3. `u_j` は `ty(args[j])` で unit-closed であり、`σ'` は `ty(args[j])` の boxed leaf で
          `σ' ⊒ u_j` であり、`P` で inhabited である。
      BY P1 (leaf の truncate は unit である), L6 の <1>1, A3 の「単一の `Arg(j, σ)`」の行 (leaf で
         あることと inhabited の同値), CODE src/rc_ir/ownership.rs: truncate_to_unit
    <3>4. QED
      BY <3>1, <3>2, <3>3, 帰納法の仮定を `(args[j], u_j)` に適用 -- 仮定より
         `unit_of(id(args[j], σ')) ∈ K(args[j], u_j) = unit_of[act(args[j], u_j)]`、これが <3>1 より
         `unit_of[act(v, π)] = K(v, π)` に含まれる。
  <2>5. QED
    BY <2>1, <2>3, <2>4

<1>7. CASE: 段 E2 (`Join(rs)`)。
  <2>1. `C_π := ∪_{r ∈ rs} act(r, π)`、`C_λ := ∪_{r ∈ rs} act(r, λ)` とおくと、
        `origin(v, π) = of_candidates(C_π, (v, π))` かつ `origin(v, λ) = of_candidates(C_λ, (v, λ))`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
  <2>2. `act(v, π) ⊇ C_π`。
    BY <2>1, L3
  <2>3. `ρ` が選んだアームの結果を `r_0` とすると、`λ` は `ty(r_0)` の boxed leaf で `P` で inhabited で
        あり、`π` は `ty(r_0)` で unit-closed である。
    BY 第 6 節の <1>9 の <2>1 と <2>2, A12 (アームの結果と `Match` の束縛変数の型が一致する)
  <2>4. CASE: `|C_λ| = 1`。
    <3>1. `C_λ = {p}` とおくと `id(v, λ) = p` であり、`act(r_0, λ) = {p}`。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates (1 要素なら `Exactly`),
         Origin::identity -- `act(r_0, λ) ⊆ C_λ = {p}` であり、`act` は空でない (L2 の <1>1)。
    <3>2. `id(r_0, λ) = p`。
      BY <3>1, L2 の <1>1 -- `id(r_0, λ) ∈ act(r_0, λ) = {p}`。
    <3>3. `unit_of(p) ∈ K(r_0, π) = unit_of[act(r_0, π)]`。
      BY <3>2, <2>3, 帰納法の仮定を `(r_0, π)` に適用
    <3>4. QED
      BY <3>1, <3>3, <2>2 -- `act(r_0, π) ⊆ C_π ⊆ act(v, π)` なので
         `unit_of(p) ∈ unit_of[act(v, π)] = K(v, π)`。
  <2>5. CASE: `|C_λ| ≥ 2` かつ `|C_π| ≥ 2`。
    <3>1. `id(v, λ) = (v, λ)` かつ `(v, π) ∈ act(v, π)`。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates (2 要素以上なら `identity` を
         `Join` に据える), Origin::identity, L2 の <1>1
    <3>2. QED
      BY <3>1, L7 -- `unit_of(id(v, λ)) = unit_of((v, λ)) = unit_of((v, π))` であり、これは
         `(v, π) ∈ act(v, π)` より `K(v, π)` の元である。
  <2>6. CASE: `|C_λ| ≥ 2` かつ `|C_π| = 1`。**この場合は偽である。**
    <3>1. `id(v, λ) = (v, λ)` であり、`unit_of((v, λ)) = (v, t_{ty(v)}(π))`。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity, L7
    <3>2. `C_π = {z}` とおくと `K(v, π) = {unit_of(z)}`。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates (1 要素なら `Exactly`),
         Origin::acted_on, acted_unit_keys
    <3>3. `(v, t_{ty(v)}(π)) = unit_of(z)` は成り立たない。
      BY 第 8 節 -- `v` が `m`、`π` が `[]`、`λ` が `[0]`、`z` が `(node, [])` である `origin` の計算が
         あり、そこでは `(m, []) != (node, [])` である。
    <3>4. QED
      BY <3>1, <3>2, <3>3 -- `unit_of(id(v, λ))` は `K(v, π)` の唯一の元と異なるので、主張 N は
         この場合に偽である。
  <2>7. QED
    BY <2>4, <2>5, <2>6 -- `|C_λ|` と `|C_π|` はどちらも 1 以上 (L2 の <1>1 と A9 より `C_π` と `C_λ` は
       空でない) なので、3 つの場合で尽きている。ただし <2>6 が主張 N を偽にする。

<1>8. QED
  BY <1>1, <1>1a, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, CODE src/rc_ir/ownership.rs: Binding,
     CODE src/rc_ir/ownership.rs: origin_inner -- `Binding` の 7 つの構成子と `None` に対する
     `origin_inner` の腕は、`here()` を答える 5 つ (<1>2)、E1 と E6 (<1>3)、E5 (<1>4)、E7 (<1>5)、
     `Llvm` (<1>1a より `π` は leaf でないので E3 は起きず E4 だけ、<1>6)、E2 (<1>7) で尽きている。
     <1>7 の <2>6 が主張 N を偽にするので、**主張 N は成り立たず、P5 (c) は閉じない**。

### 7.3 止まった場所の性質

止まったのは `Binding::Join` の腕の、次の形の場合である。

- unit の path `π` では、すべてのアームの答えが同じ `Exactly(z)` に畳まれ、答えは `Exactly(z)` になる。
  identity は `z` であり、`v` の名前はどこにも残らない。
- その下の leaf `λ` では、アームの答えが食い違い、答えは `Join { identity: (v, λ), .. }` になる。
  identity は `(v, λ)` という**新しい名前**である。

`acted_references(v, π)` は leaf ごとの identity を数えるので `(v, λ)` を名指す。`acted_unit_keys(v, π)` は
unit の path の答えだけから作られるので `(v, ・)` を 1 つも含まない。これが取りこぼしである。

#529 の修正 (`acted_on` の畳み込み) がこの場合を埋めないのは、畳み込みが**同じ問い**の中でしか働かない
からである。L4 が示すのは「`(v, π)` の答えを作るときに畳み込まれた `Origin` の identity は `(v, π)` の
`acted_on` に残る」であって、「`(v, λ)` の答えを作るときに現れる identity が `(v, π)` の `acted_on` に
残る」ではない。`(v, π)` の問いと `(v, λ)` の問いは、`Binding::Join` の腕では別々にアームへ降りるので、
互いを見ない。

**unit の問いと leaf の問いを別々の行き先へ送る辺は E4 だけである。** E1、E2、E6 は `(v, π)` と `(v, λ)` を
同じ変数の同じ path 対へ送り、E5 と E7 は両方に同じ添字を積む。E3 は `π` が leaf のときだけ働き、そのとき
`λ = π` である (L5)。E4 だけが 2 つの問いを分ける。分け方は 2 つある。

- **空集合の宣言を持つ leaf。** `origin_from_leaves_under` の `for src in sources` のループは空集合の
  leaf を素通りするので、unit の問いの答えはその leaf を勘定しない。同じ leaf を直に問うと、`reached` が
  空になって `here()` が答えになる (第 3 節の H7)。第 8 節の反例が使うのはこちらで、`origin(y, [])` が
  `(node, [])` を答えるのに `origin(y, [0])` が `(y, [0])` を答える。
- **`truncate_to_unit`。** unit の問いは `(args[j], t_{ty(args[j])}(σ'))` へ行き、leaf の問いは E3 経由で
  `(args[j], σ')` へ行く。`σ'` がその unit の下にあるとき、この 2 つは別の問いである。

## 8. 反例

型は次の 2 つである。

```
type Node   = box   struct { n : I64 };
type Choice = unbox union  { a : Node, b : Node };
```

RC IR の断片 (`c` は `Bool`、`node` は `Node`):

```
let node = struct_make(k);                 // Binding::Llvm(struct_make, [k], Node)
let m = match c {
          1 => { let x = union_make_0(node); ret x },
          0 => { let y = union_make_1(node); ret y }
        };
```

<1>1. 型の walk は次を与える。`leaves(Choice) = [[0], [1]]`、`rc_units(Choice) = [[]]`、
      `leaves(Node) = [[]]`、`rc_units(Node) = [[]]`。
  <2>1. `Choice` は `is_fully_unboxed` でも `is_closure` でも `is_box` でも `is_array` でもないので、
        `boxed_leaf_paths` は `unpunched_field_types` の各要素へ降りる。union の
        `unpunched_field_types` は各変位の payload の型を返すので、変位 0 と変位 1 の `Node` へ降り、
        `Node` は `is_box` なのでそこで leaf を積む。
    BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types, TypeNode::is_union
  <2>2. `unit_step(Choice)` は `is_union` の枝で `Unit` を返すので `rc_units(Choice) = [[]]`。
        `unit_step(Node)` は `is_box` の枝で `Unit` を返すので `rc_units(Node) = [[]]`。
    BY CODE src/rc_ir/ownership.rs: unit_step, rc_units_go
  <2>3. QED
    BY <2>1, <2>2

<1>2. `origin(node, []) = Exactly((node, []))`。
  <2>1. `InlineLLVMMakeStructBody::result_prov` は、boxed 構造体の唯一の leaf `[]` に
        `sole_origin(LeafOrigin::Fresh)` を宣言する。
    BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody の `result_prov` の
       `None => sole_origin(LeafOrigin::Fresh)` の枝, <1>1 (`leaves(Node) = [[]]`)
  <2>2. QED
    BY <2>1, 2.1 の <1>4 の <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under --
       `produced_here` が真、`operand_units` は空なので `reached = [Exactly((node, []))]`、全要素が
       等しいのでそれが答え。

<1>3. `x` の宣言は leaf `[0]` に `{Arg(0, [])}`、leaf `[1]` に空集合。`y` の宣言は leaf `[1]` に
      `{Arg(0, [])}`、leaf `[0]` に空集合。
  BY CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeUnionBody の `result_prov` --
     `Some((k, rest)) if *k == variant_idx => sole_origin(LeafOrigin::Arg(0, rest.to_vec()))`、
     `Some(_) => Set::default()`。<1>1 より leaf は `[0]` と `[1]` であり、変位 0 の leaf `[0]` の
     `rest` は `[]` である。

<1>4. `origin(x, []) = origin(y, []) = Exactly((node, []))`。
  <2>1. `[]` は `Choice` の boxed leaf ではないので `leaf_origins_at([])` は `None` を返し、
        `origin_from_leaves_under` に入る。
    BY <1>1, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>2. `x` について、`operand_units = {(0, t_{Node}([]))} = {(0, [])}` であり `produced_here` は偽。
    BY <1>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for src in sources` のループ --
       leaf `[0]` の `Arg(0, [])` が `(0, truncate_to_unit(Node, [], env)) = (0, [])` を入れ、leaf `[1]`
       の空集合はループを 1 度も回さない。`Fresh` も `Unknown` も現れない。
  <2>3. `reached = [origin(node, [])]` は 1 要素なので、答えはその要素そのものである。
    BY <2>2, <1>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の
       `if reached.iter().all(..) { return Some(first.clone()) }`
  <2>4. QED
    BY <2>1, <2>2, <2>3 -- `y` についても、寄与する leaf が `[1]` に変わるだけで同じ計算になる。

<1>5. `origin(x, [0]) = Exactly((node, []))` かつ `origin(x, [1]) = Exactly((x, [1]))`。
      `origin(y, [1]) = Exactly((node, []))` かつ `origin(y, [0]) = Exactly((y, [0]))`。
  <2>1. `x` の leaf `[0]` は宣言が単一の `Arg(0, [])` なので E3 を通り、答えは
        `origin(node, []) = Exactly((node, []))`。
    BY <1>3, 2.1 の <1>3, <1>2
  <2>2. `x` の leaf `[1]` は宣言が空集合なので `as_arg_projection` が `None` を返し、
        `origin_from_leaves_under(path = [1])` に入る。`leaf_origins_under([1])` は空集合 1 つを返すので
        `reached` は空、`reached.first()?` が `None`、`origin_inner` の `unwrap_or_else(here)` が
        `Exactly((x, [1]))` を答える。
    BY <1>3, 2.1 の <1>4 の <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: origin_inner の `None =>` の枝
  <2>3. QED
    BY <2>1, <2>2 -- `y` については変位が入れ替わるだけで同じ計算になる。

<1>6. `origin(m, []) = Exactly((node, []))`。
  <2>1. `m` の binding は `Binding::Join([x, y])` である。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕, returned_var
  <2>2. `C_[] = act(x, []) ∪ act(y, []) = {(node, [])}`。
    BY <1>4, L2 の <1>3
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕,
       Origin::of_candidates -- 1 要素なので `Exactly`。

<1>7. `origin(m, [0]) = Join { identity: (m, [0]), candidates: {(node, []), (y, [0])} }`。
  <2>1. `C_[0] = act(x, [0]) ∪ act(y, [0]) = {(node, [])} ∪ {(y, [0])}` は 2 要素である。
    BY <1>5, L2 の <1>3
  <2>2. QED
    BY <2>1, <1>6 の <2>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(..))` の腕,
       Origin::of_candidates -- 2 要素なので `Join`、`identity` は `(m, [0])`。

<1>8. `acted_references(m, []) = { (m, [0]): 1, (m, [1]): 1 }`。
  BY <1>1 (leaf は `[0]` と `[1]`), <1>7, <1>7 と同じ計算による
     `origin(m, [1]) = Join { identity: (m, [1]), candidates: {(node, []), (x, [1])} }`,
     CODE src/rc_ir/ownership.rs: acted_references

<1>9. `acted_unit_keys(m, []) = [(node, [])]`。
  <2>1. `act(m, []) = {(node, [])}`。
    BY <1>6, L2 の <1>3
  <2>2. `unit_of((node, [])) = (node, [])`。
    BY <1>1 (`rc_units(Node) = [[]]`), CODE src/rc_ir/ownership.rs: unit_of, truncate_to_unit
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: acted_unit_keys

<1>10. 変位 0 のアームを通る実行路では、`m` の leaf `[0]` は inhabited であり、
       `unit_of((m, [0])) = (m, [])` である。
  <2>1. その実行路では `m` の値は `x` の値であり、`x` は `union_make_0` の結果なのでタグは 0 である。
    BY <1>6 の <2>1, D3, D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行, A4 (コード生成の忠実さ),
       CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeUnionBody の `generate`
       (`set_union_tag` に `field_idx` を書く)
  <2>2. QED
    BY <2>1, D16 (leaf `[0]` が通る唯一の union の節はタグ 0 を選ぶ),
       <1>1 (`rc_units(Choice) = [[]]`), CODE src/rc_ir/ownership.rs: unit_of, truncate_to_unit,
       unit_step -- `Choice` は `is_union` なので `unit_step` は `Unit` を返し、走査は最初の添字で
       break して `[]` を返す。

<1>11. QED
  BY <1>8, <1>9, <1>10 -- `acted_references(m, [])` は inhabited な leaf `[0]` に由来する
     オブジェクト `(m, [0])` を名指し、その `unit_of` は `(m, [])` である。
     `acted_unit_keys(m, []) = [(node, [])]` はそれを含まない。(N) の反例である。

## 9. 反例の形は実在の Fix プログラムから出る

第 8 節の RC IR は手で書いたものだが、同じ形は普通の Fix プログラムから出る。次のプログラムを
`-O max --emit-rc-ir all` でコンパイルした。

```
module Main;

type Node = box struct { n : I64 };
type Choice = union { a : Node, b : Node };

// 再帰なので呼び出しの境界が残り、union は境界を越える。
peek : Choice -> I64 -> I64;
peek = |m, k| (
    if k <= 0 {
        if m.is_a { m.as_a.@n } else { m.as_b.@n + 100 }
    } else {
        peek(m, k - 1)
    }
);

probe : I64 -> I64;
probe = |k| (
    let node = Node { n : k };
    let m = if k % 2 == 0 { Choice::a(node) } else { Choice::b(node) };
    peek(m, 2)
);

main : IO ();
main = (
    let sum = Iterator::range(0, 6).fold(0, |acc, i| acc + probe(i));
    println $ sum.to_string
);
```

出力された `.fixlang/rc_ir.pre.txt` の該当箇所は次である (名前を短くし、無関係な行を落とした)。

```
let node : Main::Node = struct_make(k)
...
let m : Main::Choice = match tag {
    case 1(unit):
        let x : Main::Choice = union_make_0(node)
        ret x
    case 0(unit):
        let y : Main::Choice = union_make_1(node)
        ret y
}
let seen : Std::I64 = Main::peek(m, two)
```

`m` の binding は `Binding::Join([x, y])`、`x` と `y` の binding は
`Binding::Llvm(InlineLLVMMakeUnionBody, [node], Choice)`、`node` の binding は
`Binding::Llvm(InlineLLVMMakeStructBody, [k], Node)` であり、第 8 節の計算がそのまま当てはまる。

## 10. cancel の側で何が埋めているか

第 8 節の反例から miscompile は作れていない。`Release(m, [])` が `consume_unit((m, []))` を呼ばないことが
害になるのは、鍵 `(m, [])` の下に pending な `Retain` があり、その `Retain` が作った参照をこの `Release` が
処分し、しかも `Retain` が別の `Release` と対になって両方消えるときである。この 3 つを同時に満たす本体を
作ろうとすると、次の 2 つに当たる。

1. **`References` の多重集合。** `Retain(m, [])` の `outstanding` は `{(m, [0]): 1, (m, [1]): 1}`、
   `Release(node, [])` の `un_bumped` は `{(node, []): 1}` である。`covers` は名前の一致の上に立つので
   (`CODE src/rc_ir/ownership.rs: References::covers`)、この 2 つは対にならず、`un_bump` は
   `OutsideBracket` を返し、`walk_inner` の `RcExpr::Release` の腕がその鍵を `consume_unit` する
   (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`, `un_bump`)。名前が食い違うことが、対消滅を
   起こさない向きに働く。
2. **leaf の path での `acted_unit_keys`。** 鍵 `(m, [])` の下に pending な `Retain` を作るには、
   `m` の変位 payload を別名で持つ値 `p` が要る (`Binding::Payload(m, Some(0))` か
   `union_as_0(m)`)。`origin(p, []) = origin(m, [0])` であり、これは `Join` で候補に `(node, [])` を
   持つ (第 8 節の <1>7) ので、`p` を名指す `Release` や消費は `acted_unit_keys(p, [])` を通じて
   `consume_unit((node, []))` を呼ぶ。7.2 の <1>1 が示すとおり、leaf の path では (N) は取りこぼさない。

よってこの文書が主張するのは「(N) は偽であり P5 (c) は閉じない」までであって、「コードが誤っている」では
ない。P19 から P21 が (N) を引用するなら、(N) を直すか、cancel が実際に依拠している弱い形を立てて
そちらを証明するかのどちらかが要る。

## 11. (N) を真にする 2 つの向き

どちらを採るかは、証明の側ではなくコードの設計の判断である。

**向き 1: 言明を、pending になりうる鍵に絞る。** 取りこぼされる鍵は必ず `(v', t_{ty(v')}(π'))` の形で
ある (7.3)。ここで `v'` は、`Release` の変数から別名の鎖で辿り着く `Binding::Join` の変数であり、`π'` は
そこでの path である。この鍵の下に pending な `Retain` があるのは、その `Retain` の `origin` の identity が
`(v', λ')` であるとき、すなわち `v'` の leaf の別名を持つ値を retain したときに限る。(N) をその場合に限って述べれば、第 8 節の反例は言明の外に出る。ただしそのとき (N) は
`origin` だけでは書けなくなり、本体の他の節点を量化する形になる。

**向き 2: `acted_on` が鎖の通った名前をすべて持つようにする。** 取りこぼしは `origin(v, λ)` の
identity が `origin(v, π)` の答えのどこにも現れないことである。`origin_inner` が `(u, σ)` について答えを
返すたびにその `(u, σ)` を `acted_on` に積めば、7.2 の <1>7 の <2>6 は閉じる。cancel は `consume_unit` を
余分に呼ぶことになり、pending な `Retain` が無い鍵への呼び出しは何もしない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit`) ので、失われるのは対消滅の機会だけである。
どれだけ失うかは測っていない。

## 12. README への要望

- **D17 に 3 つの行を足す。** 第 5 節の DEF-1 が使った読みである。(a) `Binding::Join` の辺は、実行路が
  選んだアームの結果へ辿る。(b) `Llvm` の leaf の宣言が単一の `Fresh` または単一の `Unknown` のとき、
  鎖はそこで止まり、対応するスロットはその位置の `(u, λ)` である。(c) `Llvm` の E4 の行き先の path は
  宣言の `σ'` ではなく `truncate_to_unit(ty(args[j]), σ')` であり、leaf が `σ'` である。
  `truncate_to_unit` は path を降りるだけなので、leaf は行き先の path の下に留まる。
- **P3 と P4 の言明で、対応するスロットが候補の「下」にあることを明示する。** P4 は既に
  「`candidates` のいずれかの**下の**対応するスロット」と書いている。P3 も同じ形で読む必要がある --
  第 5 節の停止条件 S1 と S2 では、対応するスロットの path は答えの path `σ` そのものではなく `σ` の下の
  leaf である。
- **P5 (c) の `π` に定義域を書く。** `π` は `ty(v)` の RC unit または boxed leaf であるとする。そうでない
  `π` では `unit_of` の表明が破れるので、言明は偽ではなく未定義である (7.1)。
- **変位アームに入る条件を書く。** 補題 Q の <1>6 の <2>3 が使った「変位 `t` のアームの中では scrutinee の
  タグは `t` である」は、D3 にも A4 にも書かれていない。D9 の移動の表の「scrutinee の活性変位の参照が
  payload 変数へ」という行から読み取れるが、D16 (inhabited) と結び付けて使うので、D3 か A4 に 1 行として
  置くのが安全である。
- **2.2 の事実を仮定か命題として立てる。** 「`result_prov` が leaf ごとに返す集合の要素数は 1 以下で
  ある」は、第 5 節の DEF-1 と補題 Q が使う。これが破れると leaf の辿る先が 1 つに決まらず、D17 の対応
  そのものが定義できない。現在は 2.2 が数え上げで示しているだけで、新しい op が 1 つ加われば黙って破れる。
- **第 7 節の表と第 8 節の発見を更新する。** P3 と P4 は証明済みになった。P5 (c) は「#529 の修正の後も
  閉じない」であり、原因は #529 とは別のところ (`Binding::Join` の腕が unit の問いと leaf の問いを
  別々にアームへ降ろすこと) である。

## 13. `level_ownership` が P3 / P4 / P5 (c) に及ぼすもの

`level_ownership` は `infer_ownership` の不動点の中で走る新しい段である
(`CODE src/rc_ir/borrow.rs: infer_ownership`, `levelled_sites`, `level_ownership`)。

<1>1. P3、P4、P5 (c) の言明が読む関数は `origin`、`acted_references`、`acted_unit_keys`、`unit_of`、
      `unit_key` である。
  BY README の P3、P4、P5 (c) の言明, D13, D15
<1>2. これらが読むのは `VarTable` (`bindings`、`var_tys`、`param_tys`、`origins` の memo) と `TypeEnv` だけ
      である。
  BY CODE src/rc_ir/ownership.rs: origin, origin_inner, origin_from_leaves_under, acted_references,
     acted_unit_keys, unit_of, unit_key, truncate_to_unit
<1>3. `level_ownership` が書くのは `infer_ownership` の局所変数 `owned_leaves` だけである。
  BY CODE src/rc_ir/borrow.rs: level_ownership -- 引数は `&VarTable`、`&TypeEnv`、site、
     `&mut Set<VarPath>` であり、書き込みは `owned_leaves.insert` だけである。
<1>4. QED
  BY <1>1, <1>2, <1>3 -- `owned_leaves` は <1>2 の入力に入らないので、P3、P4、P5 (c) の真偽は
     `level_ownership` の有無で変わらない。`level_ownership` は `origin` を呼ぶので `VarTable::origins` の
     memo が埋まるが、memo は同じ答えを返す (`CODE src/rc_ir/ownership.rs: origin`)。

**観察 (この文書の命題の外)。** `level_ownership` は `origin(..).candidates()` を読む。#529 の修正で
`candidates` は内側の `Join` の identity を含むようになった (L4)。identity は `Match` の束縛変数、すなわち
局所変数なので `vars.param_tys.get(root)` は `None` を返し、`level_ownership` の `owns_a_candidate` の
`None => true` の枝に入る。よって修正の後、`level_ownership` は修正の前より多くの site で発火し、
より多くのパラメータ leaf を `Own` にしうる。所有が増える向きなので、この段の doc が述べるとおり
「costs a count rather than correctness」であり、P8 と P14 の側で見るべき事柄である。
