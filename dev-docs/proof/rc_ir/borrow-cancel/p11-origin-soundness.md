# P3 / P4 -- `origin` の健全性

対象は、作業ブランチ `proof-critic-round1` の `6af3eb3b4d9b38c9fe75890cce0b499ff6753498` である。
定義・仮定・命題の番号は同ディレクトリの `README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P3 (`origin` の健全性 -- `Exactly`) | 証明した (第 6 節の系 1) |
| P4 (`origin` の健全性 -- `Join`) | 証明した (第 6 節の系 2) |

P3 と P4 は、1 つの補題 Q (第 6 節) の 2 通りの読みである。Q は `origin` が辿る別名の辺を 1 本ずつ D9 の
移動の表と A3 の宣言に突き合わせる帰納法で示す。第 2 節と第 3 節がその突き合わせ、第 4 節が補題、
第 5 節が D17 の「対応するスロット」をこの文書がどう読むか (DEF-1) である。

第 7 節に、この 2 つの命題の外にある観察を 1 つ置く -- `origin` は 1 つの値の unit の path と leaf の
path に別の答えを与え、leaf の側の `identity` が unit の側の答えに現れないことがある。第 8 節は README
への要望、第 9 節は `level_ownership` がこの 2 つの命題の真偽を動かさないことである。

## 1. 記法

`origin(x, π)` は `origin(vars, type_env, &x, &π)` の略記とする。`VarPath` を `(x, π)` と書く。
`ty(x)` は `x` に束縛された値の型 (D6) である。`x` がこの関数の束縛変数であるとき、`vars.var_tys` が
それを記録する -- パラメータと capture、および `Let`、`Destructure`、`Match` のアーム payload が束縛する
変数のすべてについて `var_tys` に型が入る (`CODE src/rc_ir/ownership.rs: VarTable::of`, `collect_bindings`,
`CODE src/rc_ir/ownership.rs: VarTable` の `var_tys` フィールド)。

- `leaves(τ)` は `boxed_leaf_paths(τ)`、`leaves(τ, π)` は `π` で始まる `leaves(τ)` の要素とする
  (`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`, `LeafMap::leaves_under`)。
- `t_τ(p)` は `truncate_to_unit(τ, p)` とする (`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。
- `id(v, π)` は `origin(v, π).identity()`、`cand(v, π)` は `origin(v, π).candidates()` の集合、
  `act(v, π)` は `origin(v, π).acted_on()` の集合とする。
- `p ⊒ q` は「`p` が `q` を接頭辞として持つ」とする。

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

### 2.1 複数元の宣言は現在のプログラムに存在しない

この事実は第 5 節の DEF-1 と第 6 節の補題 Q が使う。leaf ごとの宣言の要素数が 1 以下でなければ、leaf の
辿る先が 1 つに決まらず、DEF-1 の鎖が定義できない。README の A3 の本文が同じ数え上げを持つ。

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
     `origin_inner` が読むのは `llvm_gen.result_prov(..)` の返り値そのもの、すなわち宣言である。

### 2.2 `Llvm` の腕を宣言の形で場合分けする

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
        なり、`reached.first()?` が `None` を返して、`origin_inner` の `unwrap_or_else(here)` が
        `Exactly((v, π))` を答える。A3 の空集合の行よりこの leaf は inhabited にならないので、この答えが
        名付ける参照は存在しない。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for sources` と `reached.first()?`,
       CODE src/rc_ir/ownership.rs: origin_inner の `None =>` の枝, A3, D16
  <2>3. 単一の `Fresh` または単一の `Unknown` の場合。`produced_here` が真になり、`Exactly(here)` が
        `reached` に積まれる。A3 の対応する 2 行はどちらも新しい参照なので、D10 の生成の `Llvm` の行に
        一致する。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `LeafOrigin::Fresh | LeafOrigin::Unknown`
       の腕と `produced_here`, A3, D10 の生成の表
  <2>4. 要素数 2 以上の場合。ループは要素ごとに回り、`Arg(j, σ')` は `operand_units` に入って別名として
        辿られ、`Fresh` と `Unknown` は `produced_here` を立てる。A3 の複数元の行は「いずれの路でも
        新しい参照」なので、`Arg` を別名として辿るのはこの行と食い違う。ただしこの場合は 2.1 より
        現在のプログラムには存在しない。
    BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under の `for src in sources` のループ, A3, 2.1
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4

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

<1>1. `boxed_leaf_paths` の走査は、leaf を積んだ位置の下へ降りない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- 走査 `go` が `out.push` を行う 3 つの枝
     (`is_closure`、`is_box`、`is_array`) は、いずれも `unpunched_field_types` のループへ進まずに
     `return` する。
<1>2. QED
  BY <1>1 -- leaf が積まれる位置の下は走査されないので、leaf の真の延長が leaf になることは無い。

## 5. DEF-1 -- この文書が使う D17 の読み方

D17 は「`origin` が `(x, π)` から `(u, σ)` へ辿った別名の辺の列を、`π` の下の leaf `λ` について辿ったときに
着く leaf のスロット」を、`λ` に**対応するスロット**と呼ぶ。辺ごとの `λ` の写り方は D17 が表で与えている。
D17 が決めていないものが 3 つあるので、この文書はそれを次のように読む。第 8 節に、README へ足すべき文面
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
`λ_cur` の宣言が 2 元以上である場合は、2.1 より現在のプログラムには存在しない。よってこの表と停止条件は
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
       boxed leaf であること。(iii) は同じスロットどうしなので同一である。

<1>2. CASE: 停止条件 S2 (`Llvm` で `λ` の宣言が単一の `Fresh` または単一の `Unknown`)。
  <2>1. `produced_here` が真になり、`Exactly((x, π))` が `reached` に入る。
    BY 2.2 の <1>4 の <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under の
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
  <2>1. `origin(x, π) = origin(s, π)` であり `cand(x, π) = cand(s, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の `None =>` の枝
  <2>2. `ty(s) = ty(x)` であり、`λ` は `ty(s)` の boxed leaf で `P` で inhabited である。
    BY A12 (catch-all アームの payload と scrutinee の型が一致する), D16
  <2>3. スロット `(x, λ)` とスロット `(s, λ)` は同じ参照を持つ。
    BY D9 の移動の表の catch-all アームの payload 束縛の行, D8
  <2>4. 帰納法の仮定を `(s, π)` に適用すると、停止点 `(u, σ_end, μ)` は (i) `(u, σ_end) ∈ cand(s, π)`、
        (ii)、(iii) スロット `(s, λ)` と同じ参照、を満たす。
    BY <2>2, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>3, <2>4 -- (i) は <2>1 で `cand(x, π)` に読み替わり、(iii) は <2>3 と <2>4 の推移。

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
    BY A16 (`Match` のアームは scrutinee のタグを尽くす), D21 (活性化は実行時のタグに `tag` が等しい
       アームを選ぶ) -- `P` は `tag = Some(t)` のアームの中の位置なので、その活性化における `s` の
       タグは `t` である。
  <2>4. `[t] ++ λ` は `P` で inhabited である。
    BY <2>2, <2>3, D16 -- `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (タグ `t` で <2>3 に
       より一致する) と、`λ` が通る節 (前提より一致する) である。
  <2>5. スロット `(x, λ)` とスロット `(s, [t] ++ λ)` は同じ参照を持つ。
    BY D9 の移動の表の unbox union の変位アームの payload 束縛の行, D8, <2>3 (この行が名指す活性変位が
       `t` であること)
  <2>6. QED
    BY <2>1, <2>2, <2>4, <2>5, 帰納法の仮定を `(s, [t] ++ π)` に適用

<1>7. CASE: 段 E3 (`Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)`)。
  <2>1. `π` は `ty(x)` の boxed leaf であり、よって `λ = π` である。
    BY 2.2 の <1>1 の <2>1 (`leaf_origins_at` が `Some` を返すのは `π` が leaf のとき), L5
  <2>2. `origin(x, π) = origin(args[j], σ)` であり `cand(x, π) = cand(args[j], σ)`。
    BY 2.2 の <1>3
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
     停止条件 S1、S2 で尽きており (2.1 と A3 が残る 2 つの場合を消す)、それぞれが上のいずれかの CASE で
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

**候補集合が広いことは Q を弱めない。** 補題 Q の証明が候補集合について使うのは「`cand(x, π)` が内側の
候補を**含む**」という向きだけであり、使う位置は <1>2 の <2>2、<1>8 の <2>2、<1>9 の <2>4 の 3 か所である。
`of_candidates` に渡る集合は畳み込む各 `Origin` の `acted_on()` の和であり、`act ⊇ cand` (L2) なので
`candidates()` の和を含む。含む向きに広いことは、この 3 か所のどれも壊さない。

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
(2.2 の <1>4 の <2>3)。`union_make` の `result_prov` は、作った変位の leaf に単一の `Arg(0, ..)`、他の
変位の leaf に空集合を宣言する
(`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeUnionBody` の `result_prov`)。変位 0 を作る
`x` について、`origin(x, [])` は `origin_from_leaves_under` を通り、空集合を宣言された leaf `[1]` は
ループを 1 度も回さないので `reached` は `origin(node, [])` の 1 要素になり、答えは
`Exactly((node, []))` である。
`origin(x, [1])` は `leaf_origins_under([1])` が空集合だけを返して `reached` が空になり、`origin_inner`
の `unwrap_or_else(here)` が `Exactly((x, [1]))` を答える。変位 1 を作る `y` については、leaf `[1]` の
宣言が単一の `Arg(0, [])`、leaf `[0]` の宣言が空集合なので、`origin(y, []) = Exactly((node, []))` かつ
`origin(y, [0]) = Exactly((y, [0]))` である。よって `m` では
`act(x, []) ∪ act(y, []) = {(node, [])}` が 1 要素で `origin(m, []) = Exactly((node, []))` となり、
`act(x, [0]) ∪ act(y, [0]) = {(node, []), (y, [0])}` が 2 要素で
`origin(m, [0]) = Join { identity: (m, [0]), candidates: {(node, []), (y, [0])} }` となる。`(m, [0])` は
`origin(m, [])` の答えのどこにも現れない。

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

## 8. README への要望

- **D17 に 3 つの行を足す。** 第 5 節の DEF-1 が使った読みである。D17 が現在この点について書いているのは
  次の 3 行だけである。

  > - `Binding::Move`、catch-all アームの payload、`Binding::Join`: `λ` を変えない。
  > - unbox 容器の `Destructure` のフィールド、unbox union の変位アームの payload: `λ` の先頭に添字を足す。
  > - `Binding::Llvm` の 2 つの道 (`leaf_origins_at(π)` が単一の `Arg` の場合と `origin_from_leaves_under` の
  >   場合): `λ` を、`λ` 自身の宣言 `Arg(j, σ')` の `σ'` へ置き換える。

  足すべきものは次の 3 つである。(a) `Binding::Join` の辺は行き先を複数持つので、どれを辿るかを決める --
  実行路が選んだアームの結果である。(b) `Llvm` の leaf の宣言が単一の `Fresh` または単一の `Unknown` の
  とき、鎖はそこで止まり、対応するスロットはその位置の `(u, λ)` である。上の 3 行はこの場合を挙げて
  いない。(c) `Llvm` の `origin_from_leaves_under` の道では、行き先の path は宣言の `σ'` ではなく
  `truncate_to_unit(ty(args[j]), σ')` であり、leaf が `σ'` である。上の 3 行は `λ` の写り方だけを述べ、
  行き先の path を述べていない。

- **P3 の言明で、対応するスロットが `(u, σ)` の下にあることを明示する。** P4 は
  「`candidates` のいずれかの**下の**対応するスロット (D17)」と書き、P3 は「`λ` に対応するスロット (D17)」
  と書く。DEF-1 の停止条件 S1 と S2 では、対応するスロットの path は答えの path `σ` そのものではなく
  `σ` の下の leaf `μ` (`μ ⊒ σ`) である。同じものを指す 2 つの書き方が並んでいる。

## 9. `level_ownership` が P3 と P4 に及ぼすもの

`level_ownership` は `infer_ownership` の不動点の中で走る段である
(`CODE src/rc_ir/borrow.rs: infer_ownership`, `levelled_sites`, `level_ownership`)。

<1>1. P3 と P4 の言明が読む関数は `origin` であり、D17 の対応するスロットを決めるのは `origin_inner` と
      `origin_from_leaves_under` である。
  BY README の P3 と P4 の言明, D13, D17
<1>2. この 3 つが読むのは `VarTable` (`bindings`、`var_tys`、`param_tys`、`origins` の memo)、`TypeEnv`、
      および `bindings` が持つ `LLVMGen` の `result_prov` の返り値だけである。
  BY CODE src/rc_ir/ownership.rs: origin, origin_inner, origin_from_leaves_under, as_arg_projection,
     truncate_to_unit
<1>3. `level_ownership` が書くのは `infer_ownership` の局所変数 `owned_leaves` だけである。
  BY CODE src/rc_ir/borrow.rs: level_ownership -- 引数は `&VarTable`、`&TypeEnv`、site、
     `&mut Set<VarPath>` であり、書き込みは `owned_leaves.insert` だけである。
<1>4. QED
  BY <1>1, <1>2, <1>3 -- `owned_leaves` は <1>2 の入力に入らないので、P3 と P4 の真偽は
     `level_ownership` の有無で変わらない。`level_ownership` は `origin` を呼ぶので `VarTable::origins` の
     memo が埋まるが、memo は同じ答えを返す (`CODE src/rc_ir/ownership.rs: origin`)。

**観察 (この文書の命題の外)。** `level_ownership` の発火判定は、site の `origin` の候補のうち 1 つでも
`owns_object_yet` が真であれば真になる (`CODE src/rc_ir/borrow.rs: level_ownership`, `owns_object_yet`)。
site の答えが `of_candidates(C, ・)` で作られるとき、`candidates()` は `C` そのものであり
(`CODE src/rc_ir/ownership.rs: Origin::of_candidates`, `Origin::candidates`)、`C` は畳み込まれた各
`Origin` の `acted_on()` の和である。よってそのうち 1 つでも `Join` であれば、その `identity` が
`candidates()` に入る (L2 の <1>1)。`Join` の `identity` の根は `Let` が束縛する局所変数である --
パラメータと capture の binding は `Binding::Param` であり、その腕は `here()` を返して `Join` を作らない
(`CODE src/rc_ir/ownership.rs: origin_inner`)。局所変数は `vars.param_tys` に無いので `owns_object_yet` は
それを真と答える。すなわち、site の候補集合が `Join` の `identity` を含むとき、`level_ownership` は必ず
発火し、その site の候補が名指すパラメータ leaf をすべて所有へ倒す。所有が増える向きなので、この段の doc が
述べるとおり「costs a count rather than correctness」であり、P8 と P14 の側で見るべき事柄である。
