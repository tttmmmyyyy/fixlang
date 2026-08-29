# P3 / P4 -- `origin` の健全性

対象は、作業ブランチ `proof-critic-round1` の `69506b7c65351425531225214bacc031893f200d` である。
定義・仮定・命題の番号は同ディレクトリの `README.md` による。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P3 (`origin` の健全性 -- `Exactly`) | 証明した (第 6 節の系 1)。ただし言明は計数下のオブジェクト (D26) に限る |
| P4 (`origin` の健全性 -- `Join`) | 証明した (第 6 節の系 2)。同じ限定が付く |

P3 と P4 は、1 つの補題 Q (第 6 節) の 2 通りの読みである。Q は `origin` が辿る別名の辺を 1 本ずつ D9 の
移動の表と A3 の宣言に突き合わせる帰納法で示す。

- 第 2 節が L6 (`origin_inner` の別名の辺と D9 の移動の対応)、第 2.1 節が L7 (複数元の宣言はこのプログラムに
  無いこと)、第 2.2 節が L8 (`Llvm` の腕が答えるもの)。
- 第 3 節が L9 (`origin_inner` が `Exactly((var, path))` を答える道と D10 の生成の対応)。
- 第 4 節が L1 から L5、L10 (変数に値を与える構文と、値が束縛の後 変わらないこと)、L11 (別名の辺の
  行き先の変数も値を得ていること)、L12 (スロットが参照を持つのは計数下のオブジェクトを指すときで
  あること)。
- 第 5 節が DEF-1 -- D17 の「対応するスロット」を、Q の帰納法が辿る鎖の形に書き直したもの。
- 第 6 節が補題 Q と、その 2 つの系。

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
- `α` は 1 つの活性化 (D21)、`ρ` は `α` が辿る実行路 (D21) とする。D21 の約束により、実行路について述べる
  言明は、その路を辿るすべての活性化についての言明として読む。

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

**L6 (`origin_inner` の別名の辺)**: 別名の辺は次の第 1 の表の E1 から E7 で尽きている。E4 を除く 6 つは
D9 の移動の表の 6 行と 1 対 1 に対応し、その対応は第 2 の表のとおりである。D9 の移動の表に E4 に対応する
行は無い。

| 辺 | 腕 | 行き先 |
|---|---|---|
| E1 | `Move(y)` | `origin(y, π)` |
| E2 | `Join(rs)` | 各 `r` in `rs` について `origin(r, π)` |
| E3 | `Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)` | `origin(args[j], σ)` |
| E4 | `Llvm` かつ E3 でない | `π` の下の各 leaf の宣言の各 `Arg(j, σ')` について `origin(args[j], t_{ty(args[j])}(σ'))` |
| E5 | `Field(c, i)` かつ `c` が unbox | `origin(c, [i] ++ π)` |
| E6 | `Payload(s, None)` | `origin(s, π)` |
| E7 | `Payload(s, Some(t))` かつ `s` が unbox | `origin(s, [t] ++ π)` |

| D9 の移動 | 辺 |
|---|---|
| `Let(x, Var(y), k)` | E1 |
| `Match` のアーム本体の `Ret(x)` | E2 |
| unbox 容器の `Destructure` の名前付きフィールド | E5 |
| unbox union の変位アームの payload 束縛 | E7 |
| catch-all アームの payload 束縛 | E6 |
| `Llvm` の素通し leaf (`result_prov` が単一の `Arg(i, σ)`) | E3 |

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

<1>2a. D9 の移動の 6 行は、第 2 の表のとおり E1、E2、E5、E7、E6、E3 に対応する。
  BY D9 の移動の表, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `RcRhs::Var(y)` に `Binding::Move(y)` を、
     `RcRhs::Match` の腕に `Binding::Join(arm_results)` を作り、`arm_results` は各アーム本体の
     `returned_var` である (`CODE src/rc_ir/ownership.rs: returned_var` -- 本体の終端の `Ret` が名指す
     変数)。`RcExpr::Destructure` の腕は名前付きフィールドにだけ `Binding::Field` を作り、`Match` の
     各アームの payload に `Binding::Payload(scrut, arm.tag)` を作る。`tag` が `None` のアームが
     catch-all であり、`Some(t)` のアームが変位アームである (D2 の `MatchArm` の `tag`)。
     素通し leaf の行が E3 に当たるのは、`as_arg_projection` が集合の要素数 1 と `Arg` を要求するから
     である (`CODE src/rc_ir/ownership.rs: as_arg_projection`)。

<1>2b. D9 の移動の表に E4 に対応する行は無い。
  BY <1>2a -- D9 の移動の表は 6 行であり、その 6 行が E1、E2、E5、E7、E6、E3 に 1 対 1 に対応する。
     この 6 つはいずれも E4 ではない。

<1>3. QED
  BY <1>1, <1>2, <1>2a, <1>2b

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

<1>4. 第 3 の場合、答えは `origin(args[j], σ)` である (辺 E3)。これは D9 の移動の表の `Llvm` の行と
      A3 の「単一の `Arg(j, σ)`」の行に一致する。
  BY <1>3, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `Some((j, σ))` の枝,
     D9 の移動の表, A3

<1>5. 残る 4 つの場合は `origin_from_leaves_under(vars, type_env, &decl, args, π, &(x, π))` に入り、
      それが `None` を返すときの答えは `Exactly((x, π))` である。
  BY <1>3, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕の `None =>` の枝
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
        `operand_units` の各要素 `(j, unit)` について `origin(args[j], unit)` を並べたものである。
    BY <2>1, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
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
| H7 `Llvm` かつ `π` の下の leaf の宣言がすべて空集合 | `origin_from_leaves_under` が `None` を返し `unwrap_or_else(here)` | A3 より、その leaf は inhabited にならない。D10 の生成は起きない |

<1>1. `origin_inner` の `match` の腕のうち `here()` をそのまま返す枝は、H1 から H5 の 5 つで尽きている。
  BY L6 (別名の辺は E1 から E7 で尽きている), CODE src/rc_ir/ownership.rs: origin_inner -- 6 本の腕の
     うち `here()` をそのまま返すのは、
     `None | Some(Binding::Param) | Some(Binding::Producer)` の腕 (H1、H2、H3)、
     `Some(Binding::Field(container, idx))` の `container.ty.is_box(type_env)` の枝 (H4)、
     `Some(Binding::Payload(..))` の `Some(_)` の枝 (H5)。ほかの枝は `origin` を再帰呼び出しするか
     `origin_from_leaves_under` に入る。

<1>2. `Llvm` の腕が `Exactly((var, path))` に着くのは H6 と H7 の 2 つである。
  BY L8 (d1), L8 (d2), CODE src/rc_ir/ownership.rs: origin_from_leaves_under -- H6 では
     `Exactly((var, path))` が `reached` の要素であり、`if reached.iter().all(..)` の枝が `reached` の
     全要素が等しいときその要素を返す。H7 では `reached` が空で、答えは `Exactly((var, path))` である。

<1>3. D10 の生成の 5 行はすべて `here()` の道を持つ -- H3 が `App` の行と `Closure` の行、H4 が boxed
      容器の `Destructure` の行、H5 が boxed union の変位アームの行、H6 が `Llvm` の行である。
  BY D10 の生成の表, <1>1, <1>2,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Binding::Producer` を作るのは `RcRhs::App` と
     `RcRhs::Closure` の 2 つだけであり、`Binding::Field` は `RcExpr::Destructure` の名前付き
     フィールド、`Binding::Payload(scrut, Some(t))` は変位アームの payload である。boxed か unbox かは
     `is_box` の枝が分ける。

<1>4. H1、H2、H7 は D10 の生成の表に行を持たず、それでよい。
  BY A8 (H1: グローバル値が到達するオブジェクトは線形規律の外にある), D10 の初期値 (H2),
     A3 (H7: 空集合と宣言された leaf は inhabited にならない), D16 -- どれも「新しい参照を作る」とは
     主張していない。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

## 4. 補題

以下の補題は、この文書のすべての証明が使う。

**L1 (`Origin::Join` は `of_candidates` だけが作る)**: `Origin::Join { .. }` を値として作る式は
`Origin::of_candidates` の中の 1 か所だけである。よって、どの `Origin` の値も、`Exactly` であるか、
`of_candidates` が作った `Join` (あるいはその複製) である。

<1>1. `Origin` の宣言は `src/rc_ir/ownership.rs` にあり、可視性は `pub(crate)` である。よって
      `Origin::Join { .. }` という式を書けるのは、名前 `Origin` がスコープに入っているモジュール --
      `ownership.rs` 自身と、`Origin` を import するモジュール -- だけである。
  BY CODE src/rc_ir/ownership.rs: Origin
<1>2. `crate::rc_ir::ownership` から import するモジュールは `src/rc_ir/borrow.rs`、
      `src/rc_ir/validate.rs`、`src/rc_ir/codegen.rs` の 3 つであり、どの import 並びにも `Origin` は
      無い。
  BY CODE src/rc_ir/borrow.rs の `use crate::rc_ir::ownership::{..}` (`acted_references`,
     `all_owned_units`, `collect_consumes`, `destructure_consumes`, `origin`, `rc_units`,
     `rhs_consumes`, `truncate_to_unit`, `unit_step`, `units_under`, `References`, `UnitStep`,
     `VarTable`), CODE src/rc_ir/validate.rs の `use crate::rc_ir::ownership::rc_units`,
     CODE src/rc_ir/codegen.rs の `use crate::rc_ir::ownership::{held_field_type, unit_step, UnitStep}`
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
- **(c)** `o` が `Join` ならば `|cand(o)| ≥ 2` であり、よって `|act(o)| ≥ 2` である。

<1>1. (a)。
  BY CODE src/rc_ir/ownership.rs: Origin::acted_on -- `identity` を先頭に置き、`candidates` から
     `identity` に等しいものを除いたものを続ける。

<1>2. (b)。
  BY CODE src/rc_ir/ownership.rs: Origin::identity, Origin::candidates, Origin::acted_on

<1>3. (c)。
  BY L1, CODE src/rc_ir/ownership.rs: Origin::of_candidates -- `Join` を作る枝は `candidates.len()` が
     1 でない枝であり、`candidates` が空なら手前の `assert!` が panic する。複製は `candidates` を
     そのまま運ぶ (L1)。<1>1 より `act(o) ⊇ cand(o)`。

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

- **(a)** 変数に値を与える構文は 3 つである -- `Let(x, rhs, k)` は `x` に、`Destructure(c, fs, s, k)` は
  各 `(i, x)` の `x` に、`Match` の各アームはその `payload` に値を与える。残る 4 種 (`Retain`、`Release`、
  `Eval`、`Ret`) はどの変数にも値を与えない。
- **(b)** 活性化 `α` とそれが辿る実行路 `ρ` について、変数 `v` が `ρ` 上の位置 `N` で値を得たならば、
  `ρ` 上の `N` 以後のすべての位置で `v` の値は同じである。

<1>1. (a)。
  BY D2 (節点の 6 種の表 -- `Let` は `rhs` の値を `x` に束縛し、`Destructure` は各 `(i, x)` の `x` に
     第 `i` フィールドを束縛し、`Retain` は参照を作り、`Release` は参照を処分し、`Eval` は評価して
     捨て、`Ret` はその式の値を述べる), D9 の移動の表 (「unbox union の変位アームの payload 束縛」と
     「catch-all アームの payload 束縛」の 2 行が、アームの `payload` が束縛であることを述べる),
     CODE src/rc_ir/ownership.rs: collect_bindings -- 変数に `Binding` と型を入れるのは `RcExpr::Let` の
     腕、`RcExpr::Destructure` の腕、`RcRhs::Match` の腕の `arm.payload` の 3 か所だけであり、
     `RcExpr::Retain`、`RcExpr::Release`、`RcExpr::Eval`、`RcExpr::Ret` の腕はどの変数も入れない。
<1>2. `ρ` は本体の木の各位置を高々 1 度しか通る。
  BY D2 (分岐は `Match` のアームだけであり、節点が自分自身を含むことはないので、本体は有限の木である),
     D3 (実行路は根から継続へ、アームでは アーム本体を辿ってから `k` へ進む)
<1>3. `v` を束縛する節点は本体に高々 1 つであり、`ρ` 上で `v` に値を与える位置は `N` だけである。
  BY A6 (束縛変数の名前は相異なる), <1>1, <1>2
<1>4. QED
  BY <1>1, <1>3 -- `N` より後のどの位置も `v` に値を与えないので、`v` の値は変わらない。

**L11 (別名の辺の行き先の変数も値を得ている)**: 活性化 `α`、それが辿る実行路 `ρ`、`ρ` 上の位置 `P`、
`P` までに値を得た (D6) 変数 `x` を取る。`x` の `Binding` が名指す変数 -- `Move(y)` の `y`、
`Payload(s, ・)` の `s`、`Field(c, i)` の `c`、`Llvm(gen, args, ・)` の各 `args[j]`、`Join(rs)` のうち
`α` が選んだアームの本体の終端の `Ret` が名指す変数 `r_0` -- は、いずれも `P` までに値を得ている。

<1>1. `ρ` は `P` までに、`x` に値を与える節点 `N` を通る。`N` は `x` の `Binding` を作る節点である。
  BY D6 (`P` までに値を得た変数), L10 (a), A6,
     CODE src/rc_ir/ownership.rs: collect_bindings -- `Let` は束縛する変数に `Move` / `Llvm` /
     `Producer` / `Join` を、`Destructure` は各名前付きフィールド変数に `Field` を、`Match` の各アームは
     その `payload` に `Payload` を作る。名前は相異なる (A6) ので、変数と節点の対応は 1 対 1 である。
<1>2. `Move(y)`、`Payload(s, ・)`、`Field(c, i)`、`Llvm(・, args, ・)` の場合、名指される変数は `N` の
      位置で使用され、その使用はその位置でスコープに入っている束縛に解決する。よってその束縛の節点は
      `ρ` の上で `N` より前にあり、名指される変数は `N` までに値を得ている。
  BY A11 (スコープの規律), D3 (実行路は木を根から下へ辿るので、`N` の位置でスコープに入っている束縛の
     節点は `ρ` 上で `N` より前にある), CODE src/rc_ir/ownership.rs: collect_bindings --
     `Binding::Move(y)` は `Let(x, Var(y), k)` の `y`、`Binding::Payload(scrut, ・)` は
     `Let(x, Match(scrut, arms), k)` の `scrut`、`Binding::Field(container, i)` は
     `Destructure(container, fs, s, k)` の `container`、`Binding::Llvm(・, args, ・)` は
     `Let(x, Llvm(gen, args), k)` の `args` である。
<1>3. `Join(rs)` の場合、`N` は `Let(x, Match(scrut, arms), k)` であり、`x` が値を得るのは `α` が選んだ
      アームの本体の終端の `Ret` に着いた時点である。`r_0` はその `Ret` が名指す変数なので、その位置で
      スコープに入っており、そこまでに値を得ている。
  BY D3 (アームを 1 つ選び、そのアーム本体の実行路を辿り、その後 `k` へ進む), D21 (活性化が選ぶアームは
     決まっている), A11, CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
     CODE src/rc_ir/ownership.rs: returned_var (本体の終端の `Ret` が名指す変数)
<1>4. QED
  BY <1>1, <1>2, <1>3 -- いずれの場合も、名指される変数は `N` までに値を得ており、`N` は `ρ` 上で `P`
     までにある。

**L12 (スロットが参照を持つのは、計数下のオブジェクトを指すときである)**: 実行のある時点のスロット
`(v, λ)` (D6) について、`(v, λ)` が D8 の意味の参照を持つことと `obj(v, λ)` が計数下 (D26) である
ことは同値であり、持つときその個数はちょうど 1 である。

<1>1. スロットの `λ` はその時点で inhabited な boxed leaf であり (D6)、`(v, λ)` にはちょうど 1 つの
      参照がある。
  BY D6 (スロットは `(x, λ)` であり `λ` は inhabited な boxed leaf である),
     A5 (値が保持する参照は inhabited な各 boxed leaf にちょうど 1 つずつある)
<1>2. `obj(v, λ)` がグローバル状態のオブジェクトであるとき、`(v, λ)` は D8 の意味の参照を持たない。
  BY D26
<1>3. QED
  BY <1>1, <1>2, D8, D26 -- D26 は「D8 の参照は計数下のオブジェクトへの参照だけを対象とする」と定める
     ので、<1>1 の 1 つは `obj(v, λ)` が計数下のときの参照であり、そうでないときは無い。

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

止まった位置の 3 つ組を `(u, σ_end, μ)` とし、スロット `(u, μ)` を `λ` に**対応するスロット**と呼ぶ。

E4a の行き先の path が `σ'` ではなく `t_{ty(args[j])}(σ')` であることは、コードでは
`operand_units.insert((*j, truncate_to_unit(&args[*j].ty, leaf, type_env)))` である
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。`truncate_to_unit` は `out` を `path` の
接頭辞として作るので `σ' ⊒ t_{ty(args[j])}(σ')` であり、leaf は行き先の path の下に留まる
(`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。

## 6. 補題 Q、および P3 と P4

**補題 Q**。`α` を活性化、`ρ` を `α` が辿る実行路、`P` を `ρ` 上の位置で `x` が `P` までに値を得ている
(D6) もの、`λ` を `ty(x)` の boxed leaf で `λ ⊒ π` かつ `P` で inhabited (D16) であるものとする。この
とき DEF-1 の鎖は有限で止まり、その停止点 `(u, σ_end, μ)` は次を満たす。

- (i) `(u, σ_end) ∈ cand(x, π)`。
- (ii) `μ ⊒ σ_end` であり、`μ` は `ty(u)` の boxed leaf であって `P` で inhabited である。また `u` は
  `P` までに値を得ている。すなわち `(u, μ)` は `P` のスロットである (D6)。
- (iii) スロット `(x, λ)` が `P` で D8 の意味の参照を持つとき、スロット `(u, μ)` も持ち、2 つは同一の
  参照である。

**(iii) が参照を持つ場合に限るのは、D26 がそう定めているからである。** L12 より、スロットが D8 の意味の
参照を持つことと、それが指すオブジェクトが計数下 (D26) であることは同値である。グローバル値 `g` を
`Let(x, Var(g), k)` で受ける本体では、鎖は `vars.bindings` が持たない名前 `g` で止まり (L9 の H1)、両端の
スロットはどちらも D8 の意味の参照を持たない。限定を外すと (iii) はその本体で意味を失う。D26 自身が
「この形は実プログラムにいくらでもある」と述べている。

証明は、`origin` が `(x, π)` から行う再帰呼び出しの関係の上の帰納法による。P2 よりこの関係は整礎である
(`origin` は停止するので、無限に降りる呼び出しの列は無い)。DEF-1 の各段は `origin_inner` の再帰呼び出しの
1 つに一致するので、鎖の各段で帰納法の仮定が使える。

<1>1. CASE: 停止条件 S1 (`origin_inner` が `here()` を答える)。
  <2>1. `origin(x, π) = Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `None | Some(Binding::Param) | Some(Binding::Producer)`
       の腕、`Some(Binding::Field(..))` の `container.ty.is_box` の枝、`Some(Binding::Payload(..))` の
       `Some(_)` の枝, L9, L2 (b)
  <2>2. 停止点は `(x, π, λ)` である。
    BY DEF-1 の S1
  <2>3. QED
    BY <2>1, <2>2 -- (i) は `(x, π) ∈ {(x, π)}`。(ii) は前提の `λ ⊒ π`、`λ` が `ty(x)` の `P` で
       inhabited な boxed leaf であること、`x` が `P` までに値を得ていること。(iii) は `(u, μ) = (x, λ)`
       すなわち同じスロットどうしなので、持つ参照も同一である。

<1>2. CASE: 停止条件 S2 (`Llvm` で `λ` の宣言が単一の `Fresh` または単一の `Unknown`)。
  <2>1. `Exactly((x, π))` は `reached` の要素である。
    BY L8 (d1)
  <2>2. `(x, π) ∈ cand(x, π)`。
    <3>1. `reached` の全要素が等しいとき、答えは `Exactly((x, π))` であり `cand(x, π) = {(x, π)}`。
      BY <2>1, L4 (b), L2 (b)
    <3>2. そうでないとき、答えは `of_candidates(C, (x, π))` であり `C ⊇ act(Exactly((x, π))) = {(x, π)}`
          である。`of_candidates` の `candidates()` は `C` そのものである。
      BY <2>1, L4 (b), L3, L2 (b),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. QED
    BY <2>2, DEF-1 の S2 -- 停止点は `(x, π, λ)` なので (i) は <2>2、(ii) は前提 (`x` は `P` までに値を
       得ている)、(iii) は同じスロットどうしである。

<1>3. CASE: 段 E1 (`Move(y)`)。
  <2>1. `origin(x, π) = origin(y, π)` であり `cand(x, π) = cand(y, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. `ty(y) = ty(x)` であり、`λ` は `ty(y)` の boxed leaf で `P` で inhabited である。
    BY A12 (move-bind の両辺の型が一致する), D7 (`Var` と `Ret` は値を渡すだけである), D2
       (`Let(x, rhs, k)` は `rhs` の値を `x` に束縛する), L10 (b), D16 -- `x` と `y` の値は同じなので、
       `λ` が通る各 unbox union の節のタグも同じである
  <2>3. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(y, λ)` も持ち、2 つは同一の参照である。
    BY D9 の移動の表の `Let(x, Var(y), k)` の行 (`y` の参照が `x` へ), D8, L12 (スロットの参照は
       高々 1 つである)
  <2>3a. `y` は `P` までに値を得ている。
    BY L11
  <2>4. 帰納法の仮定を `(y, π)` に適用すると、停止点 `(u, σ_end, μ)` は (i) `(u, σ_end) ∈ cand(y, π)`、
        (ii)、(iii) スロット `(y, λ)` が参照を持つときそれと同一の参照、を満たす。
    BY <2>2, <2>3a, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>3, <2>4 -- (i) は <2>1 で `cand(x, π)` に読み替わる。(iii) は、`(x, λ)` が参照を持つ
       とき <2>3 が `(y, λ)` も同じ参照を持つことを与え、<2>4 がそれを `(u, μ)` へ運ぶ。

<1>4. CASE: 段 E6 (`Payload(s, None)`、catch-all)。
  <2>1. `origin(x, π) = origin(s, π)` であり `cand(x, π) = cand(s, π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の `None =>` の枝
  <2>2. `ty(s) = ty(x)` であり、`λ` は `ty(s)` の boxed leaf で `P` で inhabited である。
    BY A12 (catch-all アームの payload と scrutinee の型が一致する), D16, D20 (catch-all アームの
       scrutinee から payload 変数への別名の辺), D9 の移動の表の catch-all アームの payload 束縛の行
       (scrutinee の参照が payload 変数へ), L10 (b) -- catch-all の payload 変数は scrutinee 全体を
       束縛するので、`λ` が通る各 unbox union の節のタグは 2 つの値で同じである。
       **この最後の一歩が読む「catch-all アームの payload 変数の値は scrutinee の値である」を値の水準で
       述べる行は README に無い。第 8 節がそれを要望する。**
  <2>3. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(s, λ)` も持ち、2 つは同一の参照である。
    BY D9 の移動の表の catch-all アームの payload 束縛の行 (scrutinee の参照が payload 変数へ), D8, L12
  <2>3a. `s` は `P` までに値を得ている。
    BY L11
  <2>4. 帰納法の仮定を `(s, π)` に適用すると、停止点 `(u, σ_end, μ)` は (i) `(u, σ_end) ∈ cand(s, π)`、
        (ii)、(iii) スロット `(s, λ)` が参照を持つときそれと同一の参照、を満たす。
    BY <2>2, <2>3a, 帰納法の仮定
  <2>5. QED
    BY <2>1, <2>3, <2>4 -- (i) は <2>1 で `cand(x, π)` に読み替わり、(iii) は <2>3 と <2>4 の推移。

<1>5. CASE: 段 E5 (`Field(c, i)`、`c` が unbox)。
  <2>1. `origin(x, π) = origin(c, [i] ++ π)` であり `cand(x, π) = cand(c, [i] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(..))` の `else` の枝
  <2>2. `[i] ++ λ` は `ty(c)` の boxed leaf であり、`[i] ++ λ ⊒ [i] ++ π` である。
    BY A12 (`Destructure` のフィールド変数とフィールドの型が合っていること、容器が構造体であること、
       **`Destructure` が名指すフィールドがその型が実際に持つ (punched でない) ものであること**),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths -- unbox 集約の枝は `unpunched_field_types` の
       各フィールドへ添字を積んで降りるので、punched でないフィールド `i` の leaf は
       `[i] ++ (そのフィールドの型の leaf)` である,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types (punched なフィールドを落とす)
  <2>3. `[i] ++ λ` は `P` で inhabited である。
    BY <2>2, D16, D2 (`Destructure` は各 `(i, x)` の `x` に第 `i` フィールドを束縛する), L10 (b) -- `[i]` は
       unbox 構造体のフィールド添字なので unbox union の節を通らず、`[i] ++ λ` が通る union の節は
       `λ` が通る節と同じであり、`c` の値は `P` でも `Destructure` の時点と同じである
  <2>4. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(c, [i] ++ λ)` も持ち、2 つは同一の参照で
        ある。
    BY D9 の移動の表の unbox 容器の `Destructure` の名前付きフィールドの行 (`c` のそのフィールドの参照が
       フィールド変数へ), D8, L12
  <2>4a. `c` は `P` までに値を得ている。
    BY L11
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>4a, 帰納法の仮定を `(c, [i] ++ π)` に適用 -- 帰納法の仮定の (i) は
       <2>1 で `cand(x, π)` に読み替わり、(iii) は <2>4 と帰納法の仮定の (iii) の推移である。

<1>6. CASE: 段 E7 (`Payload(s, Some(t))`、`s` が unbox)。
  <2>1. `origin(x, π) = origin(s, [t] ++ π)` であり `cand(x, π) = cand(s, [t] ++ π)`。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(..))` の
       `Some(tag) if !scrut.ty.is_box(type_env)` の枝
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf であり、`[t] ++ λ ⊒ [t] ++ π` である。
    BY A12 (payload と変位の型が合っていること、scrutinee が union であること、**`Match` が名指す変位が
       その型が実際に持つ (punched でない) ものであること**),
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
       CODE src/ast/types.rs: TypeNode::unpunched_field_types -- union の `unpunched_field_types` は
       punched でない各変位の payload の型を返すので、変位 `t` の leaf は
       `[t] ++ (その payload の型の leaf)` である
  <2>3. `P` において `s` のタグは `t` である。
    <3>1. `ρ` はこの `Match` の `tag = Some(t)` のアームに入り、その payload 束縛が `x` に値を与える。
          `P` は `ρ` の上でその位置以後にある。
      BY 補題 Q の前提 (`x` は `P` までに値を得ている), D6, L10 (a), A6,
         CODE src/rc_ir/ownership.rs: collect_bindings -- `x` に `Binding::Payload(s, Some(t))` を
         与えるのは `tag = Some(t)` のアームの payload 束縛だけである
    <3>2. そのアームに入った時点で、`s` の値の実行時のタグは `t` である。
      BY <3>1, A16 (`Match` のアームは scrutinee のタグを尽くす), D21 (活性化は実行時のタグに `tag` が
         等しいアームを選ぶ)
    <3>3. `s` の値は、`s` が値を得た後の `ρ` 上のすべての位置で同じである。
      BY L10 (b)
    <3>4. QED
      BY <3>1, <3>2, <3>3, L11 (`s` は `x` が値を得るまでに値を得ている) -- `P` はアームに入った時点
         以後にあり、その時点のタグは `t` であり、その間 `s` の値は変わらない。
  <2>4. `[t] ++ λ` は `P` で inhabited である。
    BY <2>2, <2>3, D16, D20 (unbox union の変位アームの scrutinee から payload 変数への別名の辺),
       D9 の移動の表の unbox union の変位アームの payload 束縛の行 (scrutinee の活性変位の参照が
       payload 変数へ) -- `[t] ++ λ` が通る unbox union の節は、`ty(s)` の根の節 (タグ `t` で <2>3 に
       より一致する) と、`λ` が通る節である。後者が一致するのは、payload 変数が scrutinee の変位 `t` の
       payload を束縛するからである。**この一歩が読む「変位アームの payload 変数の値は scrutinee の
       活性変位の payload である」を値の水準で述べる行は README に無い。第 8 節がそれを要望する。**
  <2>5. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(s, [t] ++ λ)` も持ち、2 つは同一の参照で
        ある。
    BY D9 の移動の表の unbox union の変位アームの payload 束縛の行 (scrutinee の活性変位の参照が
       payload 変数へ), D8, L12, <2>3 (この行が名指す活性変位が `t` であること)
  <2>5a. `s` は `P` までに値を得ている。
    BY L11
  <2>6. QED
    BY <2>1, <2>2, <2>4, <2>5, <2>5a, 帰納法の仮定を `(s, [t] ++ π)` に適用 -- 帰納法の仮定の (i) は
       <2>1 で `cand(x, π)` に読み替わり、(iii) は <2>5 と帰納法の仮定の (iii) の推移である。

<1>7. CASE: 段 E3 (`Llvm` かつ `leaf_origins_at(π)` が単一の `Arg(j, σ)`)。
  <2>1. `π` は `ty(x)` の boxed leaf であり、よって `λ = π` である。
    BY L8 (a) (`leaf_origins_at` が `Some` を返すのは `π ∈ leaves(ty(x))` のときである), L5, 前提の
       `λ ⊒ π`
  <2>2. `origin(x, π) = origin(args[j], σ)` であり `cand(x, π) = cand(args[j], σ)`。
    BY L8 (c)
  <2>3. `σ` は `ty(args[j])` の boxed leaf であり、`P` で inhabited である。
    BY A3 の「単一の `Arg(j, σ)`」の行 -- 宣言は第 `j` オペランドの leaf `σ` を名指し、結果のその leaf が
       inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることは同値である。<2>1 より
       `λ = π` は `P` で inhabited である。
  <2>4. スロット `(x, π)` が `P` で参照を持つとき、スロット `(args[j], σ)` も持ち、2 つは同一の参照で
        ある。
    BY A3 の同じ行 (「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を作らない」), D8, L12
  <2>4a. `args[j]` は `P` までに値を得ている。
    BY L11
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>4a, 帰納法の仮定を `(args[j], σ)` に適用 -- DEF-1 の E3 の次の 3 つ組は
       `(args[j], σ, σ)` であり、`σ ⊒ σ` である。(i) は <2>2 で読み替わり、(iii) は <2>4 と帰納法の
       仮定の (iii) の推移である。

<1>8. CASE: 段 E4a (`Llvm` かつ E3 でなく、`λ` の宣言が単一の `Arg(j, σ')`)。
  <2>1. `u_j := t_{ty(args[j])}(σ')` とおくと、`origin(args[j], u_j)` は `reached` の要素である。
    BY L8 (a) (`λ` は `ty(x)` の boxed leaf なので `decl` に宣言を持つ), L8 (d3)
  <2>2. `cand(x, π) ⊇ cand(args[j], u_j)`。
    <3>1. `reached` の全要素が等しいとき、答えは `origin(args[j], u_j)` そのものである。
      BY <2>1, L4 (b)
    <3>2. そうでないとき、答えは `of_candidates(C, (x, π))` であり、`C ⊇ act(origin(args[j], u_j))`
          である。`of_candidates` の `candidates()` は `C` そのものであり、`act ⊇ cand` (L2 (a)) である。
      BY <2>1, L4 (b), L3, L2 (a),
         CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::candidates
    <3>3. QED
      BY <3>1, <3>2
  <2>3. `σ'` は `ty(args[j])` の boxed leaf であり、`σ' ⊒ u_j` であり、`P` で inhabited である。
    BY A3 の「単一の `Arg(j, σ)`」の行 (leaf であることと inhabited の同値),
       CODE src/rc_ir/ownership.rs: truncate_to_unit (`out` は `path` の接頭辞である)
  <2>4. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(args[j], σ')` も持ち、2 つは同一の参照で
        ある。
    BY A3 の同じ行, D8, L12
  <2>4a. `args[j]` は `P` までに値を得ている。
    BY L11
  <2>5. QED
    BY <2>2, <2>3, <2>4, <2>4a, 帰納法の仮定を `(args[j], u_j)` に適用 -- DEF-1 の E4a の次の 3 つ組は
       `(args[j], u_j, σ')` である。帰納法の仮定の (i) は `cand(args[j], u_j)` の元を与え、<2>2 が
       それを `cand(x, π)` の元にする。(iii) は <2>4 と帰納法の仮定の (iii) の推移である。

<1>9. CASE: 段 E2 (`Join(rs)`)。
  <2>1. `α` はこの `Match` のちょうど 1 つのアームを通り、`P` における `x` の値はそのアーム本体の `Ret` が
        返した変数 `r_0` の値である。
    BY D3 (`Let(x, Match(v, arms), k)` ではアームを 1 つ選ぶ), D21 (活性化が選ぶアームは決まっている),
       D2 (`Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、`Ret(v)` はその式の値が `v` であることを
       述べる), L10 (b) (`x` の値は束縛の後 変わらない),
       D9 の移動の表の「`Match` のアーム本体の `Ret(x)`」の行,
       CODE src/rc_ir/ownership.rs: collect_bindings の `RcRhs::Match` の腕,
       CODE src/rc_ir/ownership.rs: returned_var
  <2>2. `ty(r_0) = ty(x)` であり、`λ` は `ty(r_0)` の boxed leaf で `P` で inhabited である。
    BY A12 (アームの結果と `Match` の束縛変数の型が一致する), <2>1, D16
  <2>3. スロット `(x, λ)` が `P` で参照を持つとき、スロット `(r_0, λ)` も持ち、2 つは同一の参照である。
    BY <2>1, D9 の移動の表の `Match` のアーム本体の `Ret(x)` の行 (`x` の参照が `Match` の束縛変数へ),
       D8, L12
  <2>4. `C_π := ∪_{r ∈ rs} act(r, π)` とおくと、`origin(x, π) = of_candidates(C_π, (x, π))` であり、
        `cand(x, π) ⊇ cand(r_0, π)`。
    <3>1. `origin(x, π) = of_candidates(C_π, (x, π))`。
      BY L4 (a), CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
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
  <2>4a. `r_0` は `P` までに値を得ている。
    BY L11
  <2>5. QED
    BY <2>2, <2>3, <2>4, <2>4a, 帰納法の仮定を `(r_0, π)` に適用 -- DEF-1 の E2 の次の 3 つ組は
       `(r_0, π, λ)` である。(i) は <2>4 が `cand(x, π)` の元にし、(iii) は <2>3 と帰納法の仮定の
       (iii) の推移である。

<1>10. QED
  <2>1. `x` の `Binding` による場合分けは、DEF-1 の段 E1、E2、E3、E4a、E5、E6、E7 と停止条件 S1、S2 で
        尽きている。
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
       帰納法の仮定が次の 3 つ組からの鎖の有限性を与える), P2 (再帰呼び出しの関係は整礎である)
  <2>3. QED
    BY <2>1, <2>2, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9 -- <2>1 の 9 つの場合が上の
       9 つの CASE であり、どの場合も (i)、(ii)、(iii) が成り立つ。

**系 1 (P3)**。`origin(x, π) = Exactly(u, σ)` のとき、すべての活性化、それが辿る実行路、および `(x, λ)`
がスロットであるすべての位置において、`π` の下の inhabited な各 leaf `λ` について次が成り立つ。`λ` に
対応するスロット (D17、DEF-1) はその位置のスロットであり、その変数は `u`、その path は `σ` の下の leaf で
ある。スロット `(x, λ)` が D8 の意味の参照を持つとき -- L12 より、`obj(x, λ)` が計数下 (D26) であるとき
-- その参照は対応するスロットが持つ参照と同一である。

<1>1. `cand(x, π) = {(u, σ)}`。
  BY 前提, L2 (b)
<1>2. QED
  BY <1>1, 補題 Q -- Q の (i) より停止点の `VarPath` は `(u, σ)` であり、(ii) よりその位置のスロットで
     あって `μ ⊒ σ` であり、(iii) より `(x, λ)` と同じ参照を持つ。

**系 2 (P4)**。`origin(x, π) = Join { identity, candidates }` のとき、すべての活性化、それが辿る実行路、
および `(x, λ)` がスロットであるすべての位置において、`π` の下の inhabited な各 leaf `λ` について次が
成り立つ。`λ` に対応するスロット (D17、DEF-1) はその位置のスロットであり、その `VarPath` は `candidates`
のいずれかの下にある。スロット `(x, λ)` が D8 の意味の参照を持つとき、その参照は対応するスロットが持つ
参照と同一である。

<1>1. `cand(x, π) = candidates`。
  BY 前提, CODE src/rc_ir/ownership.rs: Origin::candidates
<1>2. QED
  BY <1>1, 補題 Q -- Q の (i) より停止点の `VarPath` は `candidates` の元であり、(ii) の `μ ⊒ σ_end` より
     対応するスロットはその元の下にあり、(iii) が参照の同一を与える。

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

- **P3 と P4 の言明を、参照を持つスロットに限る。** 補題 Q の (iii) がその形であり、系 1 と系 2 が届く
  のもその形である。P3 が現在この点について書いているのは次の文である。

  > **P3** (`origin` の健全性 -- `Exactly`)。`origin(x, π) = Exactly(u, σ)` のとき、すべての実行路のすべての
  >   位置において、`π` の下の inhabited な各 leaf `λ` について、`obj(x, λ)` を指す参照は、`(u, σ)` の下の
  >   `λ` に対応するスロット (D17) が持つ参照と同一である。対応するスロットの path は `σ` そのものではなく
  >   `σ` の下の leaf である。

  P4 も同じで、「`π` の下の inhabited な各 leaf のスロットが持つ参照は、`candidates` のいずれかの下の
  対応するスロット (D17) が持つ参照と同一である」と書く。どちらも「そのスロットが D8 の意味の参照を
  持つとき」を持たない。D26 は「グローバル状態のオブジェクトを指す leaf は、D8 の意味の参照を持たない」
  と定めているので、グローバル値を `Let(x, Var(g), k)` で受ける本体 -- D26 自身が「この形は実プログラムに
  いくらでもある」と述べる形 -- では、両端のスロットがどちらも D8 の意味の参照を持たず、現在の言明は
  意味を失う。足すべき文は「スロット `(x, λ)` が D8 の意味の参照を持つとき」、すなわち
  「`obj(x, λ)` が計数下 (D26) であるとき」である。

- **`Match` のアームの payload 束縛が値の水準で何を渡すかを、D2 か D9 に行として足す。これは補題 Q の
  (ii) が読むので、P3 と P4 そのものが要る。** 現在の D9 の移動の表は 6 行とも参照について述べる
  (「`y` の参照が `x` へ」「scrutinee の活性変位の参照が payload 変数へ」など)。値については、D2 の表が
  `Let` と `Destructure` について述べ (「`rhs` の値を `x` に束縛する」「容器 `c` をフィールドに分解し、
  各 `(i, x)` の `x` に第 `i` フィールドを束縛する」)、D2 の `Ret` の行が「この式の値は `v` である」と
  述べ、D7 が `Var` について述べる (「`Var` と `Ret` は値を渡すだけである」)。**catch-all アームの
  payload と、変位アームの payload の 2 つについては、値の水準で述べる行がどこにも無い。** D20 はこの
  2 つを別名の辺と呼ぶが、辺の両端の値が等しいとは述べない。D2 の `MatchArm` の欄も `payload`
  (payload 変数) と書くだけである。

  Q の (ii) は「`λ` は `ty(u)` の boxed leaf であって `P` で inhabited である」を運ぶ。inhabited は
  D16 より値の実行時のタグで決まるので、辺の両端の値が同じでなければ運べない。この一歩を使うのは Q の
  `<1>4` の `<2>2` (catch-all) と `<1>6` の `<2>4` (変位アーム) であり、どちらもその位置に印を付けて
  ある。足すべき 2 行は「catch-all アームの payload 変数は scrutinee の値を束縛する」と「unbox union の
  変位アームの payload 変数は scrutinee の活性変位の payload を束縛する」である。

- **オブジェクトの同一性を要る読み手が居るなら、P3 と P4 の言明にその節を足す。** 補題 Q が示すのは
  参照の同一だけであり、対応するスロットが**同じオブジェクトを指す**ことは、参照を持たない場合には
  出していない。P5 (a) は「1 つの実行路の 1 つの位置において `origin` の `identity` が等しい 2 つの
  leaf の」「スロットは、同じオブジェクトを指す」であり、オブジェクトの同一を結論に持つ。P5 (a) の証明が
  P3・P4 の言明だけを読むなら、その言明にオブジェクトの同一の節が要る。上の 2 行があれば、D2 と D7 の
  既存の行と合わせて、別名の辺の 6 つすべてが値の水準で閉じる。

- **変数の値が束縛の後 変わらないことを、D2 か D21 に 1 行として置く。** この文書では L10 として、D2 の
  節点の 6 種の表、D9 の移動の表、A6、D3 から出した。D6 が `x` を「その実行路の上でその時点までに」
  「値を得た変数」と書き、`obj(x, λ)` を位置に依らない記法で使うので、この事実は D6 の書き方が前提に
  している。補題 Q の `<1>6` の `<2>3` -- 変位アームの中で束縛された payload について、アームを出た後の
  位置でも scrutinee のタグが `t` であること -- がこれを読む。

- **A6 の範囲を、`borrow_ify` の出力にも届く形にする。** A6 は「`borrow_ify` の入力のすべての束縛変数の
  名前は相異なり」と書き、続けて「出力についての同じ」「性質は仮定ではなく P9 が示す」と述べる。L10 の `<1>3` (`v` を束縛する節点は本体に高々 1 つである) はこの性質を読む。P3 と P4 は層 1 の
  命題であり、依存の順で P9 より前にあるので、その証明から P9 を引けない。ところが `cancel` の側の
  命題は `borrow_ify` の出力について P3 と P4 を読む。この 2 つを繋ぐ文が要る。

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
`candidates()` に入る (L2 (a))。`Join` の `identity` の根は `Let` が束縛する局所変数である --
パラメータと capture の binding は `Binding::Param` であり、その腕は `here()` を返して `Join` を作らない
(`CODE src/rc_ir/ownership.rs: origin_inner`)。局所変数は `vars.param_tys` に無いので `owns_object_yet` は
それを真と答える。すなわち、site の候補集合が `Join` の `identity` を含むとき、`level_ownership` は必ず
発火し、その site の候補が名指すパラメータ leaf をすべて所有へ倒す。所有が増える向きなので、この段の doc が
述べるとおり「costs a count rather than correctness」であり、P8 と P14 の側で見るべき事柄である。
