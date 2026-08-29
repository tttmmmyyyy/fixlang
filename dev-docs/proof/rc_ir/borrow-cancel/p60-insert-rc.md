# (P-insert): `insert_rc` の出力と A19 (ii)

この文書は、`p13-disposals-and-pending.md` が `(P-insert)` として書き出した言明を扱う。README の定義
D1-D32、仮定 A1-A20、命題 P1-P27 の**言明**の上に立つ。加えて `p13-disposals-and-pending.md` の第 7 節の
局所の定義 (DEF 実行時の作用、DEF bump の帰属、DEF ρ-歩みと ρ-終端、DEF 別名類、DEF 類ごとの参照) と
補題 `L9`・`L12`・`L13`・`L16` の**言明**を使う。証明対象のコードは `src/rc_ir/rc_insert.rs` の全体である。
対象コミットは `a4ff3c44` である。

**結論を先に書く。**

- **`(P-insert)` は偽である。** `insert_rc` が実際に出力する 5 節点の本体が反例である (第 4 節)。破れは
  1 か所の数え落としから来る -- 言明が数える「残りの消費と `Release`」には、まだ実行されていない
  `Retain` が作る参照の分が入っており、その分を引いていない。**これはコードの欠陥ではない。** 反例の本体は
  D12 を満たし、A19 (ii) も満たす。
- **数え落としを直した形だけでは A19 (ii) は出ない。** 直した形は `p13-disposals-and-pending.md` の
  反例 `C2` が満たし、`C2` は A19 (ii) を破る (第 5 節)。よって A19 (ii) へ渡るには、直した形に加えて
  `insert_rc` の出力を `C2` から区別する事実が要る。すなわち `(P-insert)` は、強すぎる読みでは
  `insert_rc` の出力について偽であり、弱めた読みでは `C2` を弾かない。
- **A19 (ii) が `insert_rc` に要求しているものは、走査の名前に触れずには書けない。** 第 6 節が A19 (ii) を
  台帳の形 `U + X ≥ D` に書き直す。`D` は別名類の処分の個数、`U` と `X` は走査がその類の名前について
  落とす bump の量である。`held` だけを主語にする言明はこの不等式を決めない。
- **`insert_rc` が与える形は、構文の形として書ける。** 第 7 節の `L9` -- `insert_rc` が置く `Retain` は、
  その変数を名指す構文の直前 (間に `Retain` 以外の節点を挟まない位置) にしか現れない -- を証明する。
  `C1` と `C2` の `main` はどちらもこの形を破るので、**どちらも `insert_rc` の出力ではない**
  (第 7 節の `L10`)。この判定は節点 1 つを見れば済む。
- **A19 を読む 2 つの命題は、別のものを要求している。** P18a・P19・P21 が読む形 (走査の帳簿) と、
  P14 が読む形 (由来ごとの非負性と、名指す由来に参照が 1 つ以上残っていること) は、どちらも他方を
  導かない。`C1` が前者だけを破り、第 9 節の `B_1` が後者だけを破る (第 9 節)。**A19 は 2 つに割れる。**
- **(O1) は証明されている。** `insert_rc` の liveness は使用だけから作られる集合であり、`Retain` と
  `Release` の位置はそれだけで決まる。第 10 節が、**各検査点において 1 つのスロットに割り当たる参照の
  個数はその変数が live かどうかで決まる** (`L18`) ことを示し、そこから別名類の粒度の RC 規律 (`L19`)
  と、`L4`・`L7` が前提に持つ「生成事象は高々 1 つ」(`L20`) を出す。
- **(O2) は前提 (N) の下で証明されている。** 第 11 節が、`origin` の `identity` が別名類を区画へ切り、
  区画が木をなすこと (`L21`) を示し、**その木の各部分木について「付いている bump の総和はその部分木が
  保持している参照の個数より真に小さい」**(`L24`) を示す。木の根で読むと `held ≥ 1 + bumps` になる
  (`L25`)。前提 (N) -- 名前は別名類を決める -- はこの文書が証明しない (6.2 節)。
- **A19 (ii-b) は活性化の終わりの 1 点先へは延びない。** 終端の `Ret` の消費の後、`held` は 0 になるが
  (`L19` (c))、走査の `RcExpr::Ret` の腕は pending の要素を取り除かないので `bumps ≥ 1` が残りうる。
  第 12 節がその形の `insert_rc` の出力を挙げる。(ii-a) はこの点でも成り立つ。

## 1. 記法

`insert_rc(prog, type_env)` は `prog` の各関数の `body` と各グローバル初期化子の `init` を書き換える
(`CODE src/rc_ir/rc_insert.rs: insert_rc`)。

- **骨格**とは `insert_rc` の入力の本体である。`RcExpr::Retain` と `RcExpr::Release` を含まない --
  含むと `insert_into_expr_inner` が panic する (`CODE src/rc_ir/rc_insert.rs:
  RcInserter::insert_into_expr_inner` の `RcExpr::Retain(..) | RcExpr::Release(..)` の腕)。
- `ty(x)`、`origin(x, π)`、`acted_on(x, π)`、`L(v, π)`、`ActRefs(v, π)` は
  `p13-disposals-and-pending.md` の第 1 節と同じ意味で使う。
- `needs_rc(v)` は `RcInserter::needs_rc(v)`、すなわち `!v.ty.is_fully_unboxed(type_env)`
  (`CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc`)。

**時点。** この文書では**時点**とは節点の訪問の入口を指す。`held_ρ(n, C)`、`bumps_ρ(n, C)` は、
`ρ` の上の節点 `n` の入口における値である。

**A19 (ii) の読み。** README の現在の形で読む。**(ii-a)** は、各時点と各**計数下の**別名類 `C` に
ついて `held_ρ(τ, C) ≥ 0` であり、読む構文と `Retain`/`Release` がその類を名指す時点では
`held_ρ(τ, C) ≥ 1` であることである。**(ii-b)** は、`bumps_ρ(τ, C) ≥ 1` である時点では
`held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であることである。この文書が単に「A19 (ii)」と書くときは (ii-b)
を指す。計数下の類に限るのは、グローバル値を ρ-終端とする類に `DEF 類ごとの参照` の表が開始値を
与えないからである (D26)。

**この文書の補題は `L1` から `L25` と番号を付ける。** 他のファイルの補題を引くときは
`p13 の L16` のように書く。

## 2. 言明

`p13-disposals-and-pending.md` の第 7.5.6 節が書き出した言明はこれである。

> **(P-insert)** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各時点の各別名類
> `C` は、`ρ` の上でその類のスロットを名指す残りの消費と `Release` の個数以上の参照を持つ。

「残りの消費と `Release` の個数」を次の量として読む。

**DEF `Fut`**。`ρ` の上の節点 `n` と別名類 `C` について、`Fut_ρ(n, C)` を、`ρ` の上で `n` より後にある
次の事象の個数の和とする。

- `(w, μ) ∈ C` である leaf の D9 の意味の消費。1 つの構文が `C` のスロットを `k` 個消費するときは `k` を
  数える。
- `Release(v, π)` 節点であって `(v, λ) ∈ C` である `λ` を `π` の下に持つもの。そのような `λ` 1 つにつき
  1 を数える。

時点は節点の訪問の入口なので、`n` 自身の実行が行う事象は「`n` より後」に数える。

これは `DEF 類ごとの参照` (`p13-disposals-and-pending.md` の第 7.5.3 節) の表で `held_ρ(·, C)` を
**減らす** 2 種類の事象の、`n` より後の分の個数である。「参照を持つ」は同じ表の `held_ρ(n, C)` である。
すなわち `(P-insert)` は次を主張する。

> `ρ` の上の各節点 `n` と各別名類 `C` について、`held_ρ(n, C) ≥ Fut_ρ(n, C)`。

## 3. この文書が示すこと

- **R1** (第 4 節)。`(P-insert)` を破る `insert_rc` の出力が在る。
- **R2** (第 5 節)。`Fut` から未実行の `Retain` の分を引いた形 `(P-insert-net)` は、
  `p13-disposals-and-pending.md` の反例 `C2` が満たす。`C2` は A19 (ii) を破るので、
  `(P-insert-net)` だけから A19 (ii) は出ない。
- **L7** (第 6 節)。A19 (ii) は、別名類ごとの台帳の不等式 `U + X ≥ D` と同値である (前提つき)。
- **L9**、**L10** (第 7 節)。`insert_rc` の出力の `Retain` の位置の形と、それが `C1` と `C2` を弾くこと。
- **L12**、**L13** (第 9 節)。A19 を読む 2 つの形が、どちらも他方を導かないこと。
- **L14** - **L20** (第 10 節)。(O1)。`insert_rc` の liveness と実行時の参照の分布が一致すること
  (`L18`)、そこから出る別名類の粒度の RC 規律 (`L19`)、および生成事象の一意性 (`L20`)。
- **L21** - **L25** (第 11 節)。(O2)。名前の鎖が作る区画の木 (`L21`)、その部分木についての不等式
  (`L24`)、そこから出る `held ≥ 1 + bumps` (`L25`)。
- 第 12 節。A19 (ii-b) が活性化の終わりの 1 点先では偽であること。

## 4. R1: `(P-insert)` は `insert_rc` の出力で偽である

### 4.1 反例のプログラム

`p13-disposals-and-pending.md` の第 7.5.7 節と同じ道具立てを使う。`Arr` を boxed な型、`I` を
`is_fully_unboxed` が真の型とする。`Llvm` 演算 `alloc : () -> Arr` は結果の leaf を単一の `Fresh` と
宣言し、`zero : () -> I` は boxed leaf を持たない結果を返す。どちらもオペランドを持たない。

骨格 `S_f` (関数 `f`、パラメータ `b : Arr`、`borrowed_units` は空、返り値の型 `I`):

```
Let(z, Llvm(zero, []), Ret(z))
```

骨格 `S_main` (関数 `main`、パラメータ無し、capture 無し、返り値の型 `I`):

```
Let(p, Llvm(alloc, []),
Let(u, App(f, [p]),
Let(w, App(f, [p]),
Ret(w))))
```

どちらも `Retain`/`Release` を含まないので骨格である。

### 4.2 `L1` (`insert_rc` が出力する本体)

**言明**。`insert_rc` は `S_main` を次の本体 `B_0` に書き換える。

```
Let(p, Llvm(alloc, []),
Retain(p, [], RcState::Unknown,
Let(u, App(f, [p]),
Let(w, App(f, [p]),
Ret(w)))))
```

また `S_f` を `Release(b, [], RcState::Unknown, Let(z, Llvm(zero, []), Ret(z)))` に書き換える。

**証明**

<1>1. `needs_rc(p)` は真であり、`needs_rc(u)`、`needs_rc(w)`、`needs_rc(z)` は偽である。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` である。`ty(p) = Arr` は boxed なので
  `is_fully_unboxed` は偽であり、`ty(u) = ty(w) = ty(z) = I` は 4.1 の仮定より `is_fully_unboxed` が真で
  ある。

<1>2. `p`、`u`、`w`、`z`、`b` の名前は `is_local` が真であり、`f` の名前は偽である。
  BY CODE src/ast/name.rs: FullName::is_local
  `is_local` は名前空間が空かを答える。局所変数の名前は名前空間を持たず、トップレベルの関数 `f` の名前は
  名前空間の下にある。

<1>3. `insert_rc` は `main` について `RcInserter::insert_into_func(main)` を呼び、それは
      `self.insert_into_expr(func.body, &Set::default())` を呼ぶ。
  BY CODE src/rc_ir/rc_insert.rs: insert_rc, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func

<1>4. `insert_into_expr(Ret(w), ∅)` は `(Ret(w), {w})` を返す。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live, <1>1, <1>2, A15
  この腕は `live = live_after ∪ {x}` を作り (`insert_if_local`、<1>2 より `w` は入る)、
  `retain_if_live(&x, live_after, ret)` を呼ぶ。`live_after = ∅` は `w` を含まないので、`retain_if_live`
  の条件 `live.contains(&var.name)` が偽であり、節点はそのまま返る。A15 より `insert_into_expr` は
  `insert_into_expr_inner` をちょうど 1 回呼ぶ。

<1>5. `insert_into_expr(Let(w, App(f, [p]), Ret(w)), ∅)` は
      `(Let(w, App(f, [p]), Ret(w)), {p})` を返す。
  <2>1. この呼び出しは `insert_into_operation_let(w, App(f, [p]), Ret(w), source, ∅)` に入る。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Let(x, rhs, cont)` の腕
    右辺は `Match` ではない。
  <2>2. `live_cont = {w}` である。
    BY <1>4, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    この関数はまず `self.insert_into_expr(cont, live_after)` を呼ぶ。
  <2>3. `retains_before` と `releases_after` はどちらも空である。
    BY CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
       RcInserter::insert_into_operation_let, <1>2, <2>2
    `rhs_operands(App(f, [p]))` は `[(f, Own), (p, Own)]` である。逆順の走査はまず `p` を見る。
    `live_after_operand` は `live_cont = {w}` の写しなので `p` を含まず、`used_later` は偽である。
    所有は `Own` なので `releases_after` には入らず、`used_later` が偽なので `retains_before` にも
    入らない。次に `f` を見るが、<1>2 より `f.name.is_local()` が偽なので何も起きない。
  <2>4. `after` は空である。
    BY <2>2, <2>3, <1>1, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `after` は `releases_after` (<2>3 より空) に、`!live_cont.contains(&x.name) && self.needs_rc(&x)` の
    ときだけ `x = w` を足したものである。<2>2 より `live_cont = {w}` は `w` を含むので、この条件は偽で
    ある。
  <2>5. 返る節点は `Let(w, App(f, [p]), Ret(w))` である。
    BY <2>3, <2>4, CODE src/rc_ir/rc_insert.rs: build_releases, CODE src/rc_ir/rc_insert.rs: build_retains
    `build_releases(∅, cont)` と `build_retains(∅, node)` はどちらも引数の節点をそのまま返す
    (空の `Vec` の `fold` は初期値を返す)。
  <2>6. QED
    BY <2>2, <2>3, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `live_before` は `live_cont` から `x = w` を除き、各オペランドの局所名を足したものである。
    `{w} \ {w} = ∅` に `p` が入り、`f` は <1>2 より入らない。

<1>6. `insert_into_expr(Let(u, App(f, [p]), <5 の骨格>), ∅)` は
      `(Retain(p, [], Unknown, Let(u, App(f, [p]), Let(w, App(f, [p]), Ret(w)))), {p})` を返す。
  <2>1. この呼び出しは `insert_into_operation_let(u, App(f, [p]), ·, source, ∅)` に入り、
        `live_cont = {p}` である。
    BY <1>5, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の
       `RcExpr::Let(x, rhs, cont)` の腕
  <2>2. `retains_before = [p]` であり、`releases_after` は空である。
    BY <2>1, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
       RcInserter::insert_into_operation_let
    逆順の走査はまず `p` を見る。`live_after_operand` は `live_cont = {p}` の写しなので `p` を含み、
    `used_later` は真である。所有は `Own` で `needs_rc(p)` が真 (<1>1) なので `retains_before` に `p` が
    入る。`f` は <1>2 より飛ばされる。
  <2>3. `after` は空である。
    BY <2>1, <2>2, <1>1, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `releases_after` は空であり、`x = u` については `needs_rc(u)` が偽 (<1>1) なので足されない。
  <2>4. 返る節点は `Retain(p, [], RcState::Unknown, Let(u, App(f, [p]), <5 の本体>))` である。
    BY <2>2, <2>3, CODE src/rc_ir/rc_insert.rs: build_retains
    `build_retains([p], node)` は `RcExpr::Retain(p, vec![], RcState::Unknown, node)` を作る。
  <2>5. QED
    BY <2>1, <2>2, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `live_before` は `{p} \ {u} = {p}` に `p` を足したもので `{p}` である。

<1>7. `insert_into_expr(S_main, ∅)` は `(B_0, ∅)` を返す。
  BY <1>6, <1>1, CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
     RcInserter::insert_into_operation_let
  `rhs_operands(Llvm(alloc, []))` は空の列である (`alloc` はオペランドを持たない)。よって
  `retains_before` と `releases_after` は空である。`x = p` については `live_cont = {p}` が `p` を含むので
  `after` に入らない。`live_before` は `{p} \ {p} = ∅` である。

<1>8. `main` について `insert_into_func` はこの節点をそのまま `func.body` にする。
  BY <1>7, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func
  `live` は空なので `for name in &live` の表明は走らず、`main` はパラメータも capture も持たないので
  `unused` は空であり、`build_releases(∅, body)` は `body` を返す。

<1>9. `f` について `insert_rc` は `Release(b, [], RcState::Unknown, Let(z, Llvm(zero, []), Ret(z)))` を
      作る。
  BY <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: build_releases
  `Ret(z)` の腕は `live = {z}` を返し、`retain_if_live` は `live_after = ∅` の下で発火しない。
  `Let(z, Llvm(zero, []), Ret(z))` はオペランドを持たず、`live_cont = {z}` が `x = z` を含むので
  `after` は空であり、`live_before = {z} \ {z} = ∅` である。`insert_into_func` はこの `live` が `b` を
  含まないことと `needs_rc(b)` が真であること (<1>1、`ty(b) = Arr`) から `unused = [b]` を作り、
  `build_releases([b], body)` が `Release(b, [], RcState::Unknown, ·)` を被せる。

<1>10. QED
  BY <1>3, <1>8, <1>9

### 4.3 `L2` (`B_0` の実行路と別名類)

**言明**。`B_0` は実行路 `ρ_0` をちょうど 1 本持つ。`ρ_0` を辿る活性化について、`(p, [])` は
別名類 `C_p = {(p, [])}` の唯一のスロットであり、`C_p` の ρ-終端は `(p, [])` 自身、`obj(C_p)` は
`alloc` が割り当てたオブジェクトで、これは D26 の意味で計数下である。

**証明**

<1>1. `B_0` は `Match` を含まないので、実行路は 1 本である。
  BY D3
  D3 の規則で分岐が生じるのは `Let(x, Match(v, arms), k)` の行だけである。

<1>2. `ρ_0` の上のスロットのうち、変数が `p`、`u`、`w` のいずれかであるものは `(p, [])` だけである。
  BY D4, D6
  D4 より `boxed_leaf_paths(Arr) = {[]}` (`Arr` は `is_box` が真なので自分自身の位置 1 つ) であり、
  `boxed_leaf_paths(I) = ∅` (`is_fully_unboxed` が真なので leaf を持たない) である。`ty(u) = ty(w) = I`
  なので、この 2 つは leaf を持たない。

<1>2a. `B_0` に現れる残りの `RcVar` は `App` の callee `f` であり、その各 boxed leaf `λ` について
       `(f, λ)` は ρ-終端である。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin_inner,
     p13 の DEF ρ-歩みと ρ-終端
  `collect_bindings` は `f` に束縛を入れない (`f` は `B_0` のどの `Let`、`Destructure`、`Match` の
  束縛変数でもない)。`origin_inner` の `None` の腕は `here()` を返し、`origin` を呼ばない。

<1>3. `origin(p, []) = Origin::Exactly((p, []))` である。
  BY CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under, A3
  `collect_bindings` は `Let(p, Llvm(alloc, []), ·)` に `Binding::Llvm(alloc, [], Arr)` を入れる。
  `origin_inner` の `Binding::Llvm` の腕は、まず `decl.leaf_origins_at([])` を `as_arg_projection` に
  掛ける。A3 と 4.1 より宣言は単一の `Fresh` なので `as_arg_projection` は `None` を返す
  (`CODE src/rc_ir/ownership.rs: as_arg_projection` は `LeafOrigin::Fresh` に `None` を返す)。次に
  `origin_from_leaves_under` を呼ぶ。オペランドが無いので `operand_units` は空、`Fresh` により
  `produced_here` が真になり、`reached = [Origin::Exactly((p, []))]` の 1 元である。`reached` の元が
  すべて等しいので、その値がそのまま返る。

<1>4. `(p, [])` は ρ-終端であり、`C_p = {(p, [])}` である。
  BY <1>3, <1>2, <1>2a, p13 の DEF ρ-歩みと ρ-終端, p13 の DEF 別名類
  <1>3 の計算で `origin_inner` は `origin` を 1 度も呼ばない (`operand_units` が空なので
  `origin_from_leaves_under` の中の `origin` の呼び出しも起きない)。よって `(p, [])` は ρ-終端である。
  別名類は ρ-終端が等しいスロットの集まりなので、`(p, [])` を ρ-終端とするスロットを数えればよい。
  <1>2 より `p`、`u`、`w` の側にはそれは `(p, [])` だけであり、<1>2a より `f` の側のスロットの
  ρ-終端は `(f, λ)` であって `(p, [])` ではない。

<1>5. `obj(C_p)` は計数下である。
  BY A3, D26
  A3 の表より、単一の `Fresh` を宣言する leaf には新しく割り当てたオブジェクトへの参照が置かれる。D26 より
  割り当てられたオブジェクトは計数下である。

<1>6. QED
  BY <1>1, <1>4, <1>5

### 4.4 `L3` (`B_0` の上の `held` と `Fut`)

**言明**。`n_R` を `B_0` の `Retain(p, [])` 節点とする。`held_ρ0(n_R, C_p) = 1` であり、
`Fut_ρ0(n_R, C_p) = 2` である。

**証明**

<1>1. `f` のパラメータ `b` の unit は `f` が所有する。
  BY A1, CODE src/rc_ir/borrow.rs: split_rc_units, D14
  A1 より `borrow_ify` の入力のすべての関数の `borrowed_units` は空である。`borrow_ify` の入力は
  `split_rc_units` の出力であり、`split_rc_units` は各関数の `body` と各グローバル初期化子の `init` しか
  書き換えない (`CODE src/rc_ir/borrow.rs: split_rc_units`)。よって `insert_rc` の出力でも
  `f.borrowed_units` は空であり、D14 より `b` のすべての unit を `f` が所有する。

<1>2. `ρ_0` の上で `C_p` のスロットを名指す D9 の消費は、2 つの `App(f, [p])` の引数の位置の 2 つである。
  BY D9, <1>1, L2, CODE src/rc_ir/ownership.rs: rhs_consumes
  D9 の `App` の行より、呼び出し先が所有する位置の引数の leaf が消費される。<1>1 よりどちらの
  `App(f, [p])` も `(p, [])` を消費する。同じ行の callee の leaf は `(f, ·)` であって `C_p` のスロット
  ではない -- L2 より `C_p` のスロットは `(p, [])` だけである。`B_0` に `Closure`、`Destructure`、
  `Llvm` の消費する形は無く、終端の `Ret(w)` は `ty(w) = I` に boxed leaf を持たないので何も消費しない
  (D4)。

<1>3. `ρ_0` の上に `Release` 節点は無い。
  BY L1
  `B_0` の 5 つの節点は `Let`、`Retain`、`Let`、`Let`、`Ret` である。

<1>4. `Fut_ρ0(n_R, C_p) = 2` である。
  BY <1>2, <1>3, DEF `Fut`
  2 つの消費はどちらも `n_R` より後にある。

<1>5. `held_ρ0(n_R, C_p) = 1` である。
  BY L2, D10, p13 の DEF 類ごとの参照
  `C_p` の ρ-終端 `(p, [])` は D10 の生成の表の `Llvm` の行で作られる (宣言が単一の `Arg` でないため)。
  表の第 1 行より `held` は 1 から始まる。`n_R` の入口までに起きた事象はこの生成だけである -- `n_R` は
  `Let(p, Llvm(alloc, []), ·)` の直後の節点であり、その間に `Retain`、`Release`、消費は無い。

<1>6. QED
  BY <1>4, <1>5

### 4.5 R1 の結論

**R1**。`(P-insert)` は偽である。

**証明**
`L1` より `B_0` は `insert_rc` の出力である。`L2` より `B_0` は実行路 `ρ_0` を持ち、`C_p` はその上の
別名類である。`L3` より、`ρ_0` の上の節点 `n_R` の入口という時点において
`held_ρ0(n_R, C_p) = 1 < 2 = Fut_ρ0(n_R, C_p)` である。第 2 節の読みより、これは `(P-insert)` の否定で
ある。∎

### 4.6 破れの出どころ

`B_0` は D12 を満たす。`main` の `Obl` と `H(O_p)` は、割り当ての後、`Retain(p, [])` の後、最初の
`App(f, [p])` が返った後、2 番目の `App(f, [p])` が返った後の順に
`{O_p}, 1` → `{O_p, O_p}, 2` → `{O_p}, 1` → `{}, 0` と動く。`App` の消費は `Obl` から参照を 1 つ
取り除いて呼び出し先に渡し (D10)、`f` の `Release(b, [])` がその参照を処分して `H` を 1 下げる。
(S-a) は各除去が `Obl` に入っているので、(S-b) は終端の `Ret(w)` の時点で `Obl` が空で `ty(w) = I` に
boxed leaf が無いので、(S-c) は最初の `App` が `H(O_p) = 2`、2 番目の `App` が `H(O_p) = 1`、
`Retain(p, [])` が `H(O_p) = 1` の下で読み・触れるので、それぞれ成り立つ。

`B_0` は A19 (ii) (第 1 節の読み) を満たす。`Retain(p, [])` の要素の `outstanding` は
`ActRefs(p, []) = {(p, []): 1}` であり、最初の `App(f, [p])` の消費が渡す
`acted_on(p, []) = {(p, [])}` はその鍵なので、`consume_objects` がその要素を取り除く
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)。`Retain(p, [])` の入口から順に、
`(held, bumps)` は `(1, 0)`、`(2, 1)`、`(1, 0)`、`(0, 0)` と動く。

**破れているのは言明の側である。** `Fut_ρ(n, C)` は `n` より後の処分だけを数え、`n` より後の `Retain` が
作る参照を数えない。`held_ρ(n, C)` はその時点の参照の個数なので、後で `Retain` が増やす分を先取りして
持ってはいない。第 5 節がこの数え落としを直した形を扱う。

## 5. R2: 数え落としを直した形は A19 (ii) を果たさない

**DEF `Ret#`**。`ρ` の上の節点 `n` と別名類 `C` について、`Ret#_ρ(n, C)` を、`ρ` の上で `n` より後にある
`Retain(v, π)` 節点であって `(v, λ) ∈ C` である `λ` を `π` の下に持つものについて、そのような `λ` の
個数の総和とする。`DEF Fut` と同じく、`n` 自身が `Retain` であるときその `Retain` は「`n` より後」に
数える。

**DEF `(P-insert-net)`**。`ρ` の上の各節点 `n` と各別名類 `C` について、
`held_ρ(n, C) ≥ Fut_ρ(n, C) - Ret#_ρ(n, C)`。

### 5.1 `L4` (`held` は生成と増減で決まる)

**言明**。`ρ` の上の各節点 `n` と、`ρ` の上で生成事象を 1 つだけ持つ各別名類 `C` について、
`held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)` である。ここで `R_ρ(n, C)` は `n` より前に実行された
`Retain(v, π)` 節点の `(v, λ) ∈ C` である `λ` の個数の総和、`D_ρ(n, C)` は `n` より前に実行された
`Release(v, π)` 節点の同じ数え上げと `C` のスロットの D9 の消費の個数の和である。

**証明**
`p13-disposals-and-pending.md` の `DEF 類ごとの参照` の表が `held_ρ(·, C)` の変化のすべてである。表は
6 行を持つ。最初の 3 行は開始値 1 を与え、仮定よりそのうち 1 つだけが `ρ` の上で起きる。残りの 3 行は
`Retain` につき `+1`、`Release` につき `-1`、消費につき `-1` である。`R` と `D` はこの 3 行の個数の
定義そのものである。∎

### 5.2 `L5` (`C2` は `(P-insert-net)` を満たす)

**言明**。`p13-disposals-and-pending.md` の第 7.5.8 節の反例 `C2` の `main` の本体、その唯一の実行路
`ρ`、`o` の別名類 `C_o` について、`(P-insert-net)` の不等式は `ρ` の上のすべての節点で成り立つ。

**証明**

<1>1. `C_o` の上の事象は、`ρ` の順に、生成 (`Llvm(alloc, [])`)、消費 (`App(id, [o])` の引数)、
      `Retain(o, [])`、`Release(o, [])` の 4 つである。
  BY p13 の第 7.5.8 節 (`C2` の本体と、`o` と `y` が別の別名類であること), D9, D10
  `C2` の `main` は `Let(o, Llvm(alloc, []), Let(y, App(id, [o]), Retain(o, [], s, Let(u, App(f, [y]),
  Eval(o, Release(o, [], s, Ret(u)))))))` である。`Eval` は D9 の消費でも移動でもない。`App(f, [y])` が
  消費するのは `(y, [])` であり、第 7.5.8 節より `(y, [])` は `C_o` のスロットではない。

<1>2. 各節点の入口で `(held, Fut, Ret#)` は次のとおりである。
  BY <1>1, L4, DEF `Fut`, DEF `Ret#`

  | 入口 | `held` | `Fut` | `Ret#` |
  |---|---|---|---|
  | `Let(y, App(id, [o]), ·)` | 1 | 2 | 1 |
  | `Retain(o, [])` | 0 | 1 | 1 |
  | `Let(u, App(f, [y]), ·)` | 1 | 1 | 0 |
  | `Eval(o, ·)` | 1 | 1 | 0 |
  | `Release(o, [])` | 1 | 1 | 0 |
  | `Ret(u)` | 0 | 0 | 0 |

<1>3. QED
  BY <1>2
  各行で `held ≥ Fut - Ret#` である (`1 ≥ 1`、`0 ≥ 0`、`1 ≥ 1`、`1 ≥ 1`、`1 ≥ 1`、`0 ≥ 0`)。

### 5.3 R2 の結論

**R2**。`(P-insert-net)` を満たしながら A19 (ii) を破る本体が在る。

**証明**
`p13-disposals-and-pending.md` の第 7.5.8 節は、`C2` が A19 (ii) を破ることを示している --
`Retain(o, [])` の要素が `pending` に在る間 `bumps_ρ(·, C_o) = 1` であり `held_ρ(·, C_o) = 1 < 2` で
ある。`L5` よりその同じ `C2` が `(P-insert-net)` を満たす。∎

**この 2 つが挟んでいるもの。** `(P-insert)` は `insert_rc` の出力について偽であり (R1)、数え落としを
直した `(P-insert-net)` は `C2` を弾かない (R2)。`C2` は `insert_rc` の出力ではない (第 7.4 節の `L10`)
ので、R2 は `insert_rc` の出力について A19 (ii) が破れることを言うのではない。R2 が言うのは、
`(P-insert-net)` から A19 (ii) へ渡る段が、`insert_rc` の出力を `C2` から区別する事実を要ることである。
第 6 節がその段の形を書き、第 7 節がその区別を与える。

## 6. A19 (ii) の台帳形

### 6.1 局所の定義

`ρ` と活性化を固定し、`obj(C)` が計数下である別名類 `C` を固定する。`ρ` の上の節点 `n` について:

- `R_ρ(n, C)`、`D_ρ(n, C)` は `L4` のもの。
- `U_ρ(n, C)` を、`n` より前の `Release` の訪問で `un_bump` が `InBracket` を返し、選ばれた要素の
  `B_ρ` から引かれた量のうち、`C` のスロットの `identity` に付いていた分の総和とする
  (`CODE src/rc_ir/borrow.rs: un_bump`, p13 の DEF bump の帰属)。
- `X_ρ(n, C)` を、`n` より前に `pending` から取り除かれた要素の、取り除かれた時点の `B_ρ` のうち
  `C` のスロットの `identity` に付いていた分の総和とする。取り除く操作は `consume_objects` と `merge` で
  ある (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/borrow.rs:
  CancelAnalysis::merge`)。

### 6.2 `L6` (bump の残高)

**言明**。次の 3 つの前提の下で、`bumps_ρ(n, C) = R_ρ(n, C) - U_ρ(n, C) - X_ρ(n, C)` である。

- **(N)** 名前は類を決める。すなわち `origin(v, λ).identity()` が `C` のスロットの `identity` に等しい
  leaf `(v, λ)` は、`C` のスロットである。
- **(I)** `C` のスロットは `ρ` の上で inhabited であり、`obj(C)` は計数下である。
- **(A)** `ρ` の上の各活性化について、`Retain(v, π)` が `B_ρ` に足す量のうち `C` の名前に付く分は、
  `π` の下の `(v, λ) ∈ C` である `λ` の個数に等しい。

**証明**
p13 の `DEF bump の帰属` は `B_ρ` の変化のすべてを表にしている。表の 6 行のうち、`B_ρ` を増やすのは
`Retain` の行だけであり、その量は (A) より `R` の増分に等しい。`B_ρ` を減らすのは `Release` で
`un_bump` が `InBracket` を返す行だけであり、その量が `U` の定義である。要素が `pending` を離れると
その `B_ρ` は和から落ちる。それが `X` の定義である。アームへの複製と `merge` の運搬、および残る行は
`B_ρ` を変えない。`bumps_ρ(n, C)` は `pending` の要素の `B_ρ` のうち `C` の名前に付く分の総和なので
(A19 (ii) の言明)、その値は足した分から引いた分と落ちた分を除いたものである。(N) は、`C` の名前に付く
量が `C` のスロット由来の量だけであることを言う。∎

**(N) について。** p13 の `L9` (identity は inhabited を決める) と `L12`・`L13` (ρ-歩みと `acted_on`)、
および P5 (a) がこれを与える形をしている。**この文書は (N) を証明しない。** `L6` は (N) を前提に持つ
補題である。

### 6.3 `L7` (A19 (ii) の台帳形)

**言明**。`L6` の 3 つの前提と `L4` の前提 (生成事象が 1 つ) の下で、`ρ` の上の節点 `n` について
次が成り立つ。

```
held_ρ(n, C) - (1 + bumps_ρ(n, C)) = U_ρ(n, C) + X_ρ(n, C) - D_ρ(n, C)
```

とくに `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)` と `U_ρ(n, C) + X_ρ(n, C) ≥ D_ρ(n, C)` は同値である。

**証明**

```
held - (1 + bumps)
  = (1 + R - D) - (1 + bumps)          [BY L4]
  = (1 + R - D) - (1 + R - U - X)      [BY L6]
  = U + X - D                          [算術]
```

∎

### 6.4 台帳形が言うもの

`D` は別名類の処分の個数であり、`U + X` は走査がその類の名前について落とす bump の量である。すなわち
A19 (ii) は「**走査の帳簿は、その類の処分に遅れない**」という主張である。

`p13-disposals-and-pending.md` の 2 つの反例は、この形で次のように読める。

- `C1`: `Retain(m, [])` の要素の `outstanding` は `{(m, []): 1}` である。`App(f, [p])` の消費が渡す
  `acted_on(p, []) = {(p, [])}` はその鍵ではないので `consume_objects` は要素を落とさない。よって
  `D` が 1 増えて `U + X` は増えない。
- `C2`: `App(id, [o])` の消費の時点で `pending` に要素が無いので `X` は増えず、`D` が 1 増える。その後の
  `Retain(o, [])` は `R` を増やすが `U + X` は増やさない。

**この不等式を決めるのは名前である。** `U` と `X` は、走査が持つ `outstanding` の鍵 -- `origin` の
`identity` -- と、処分の構文が渡す `acted_on` が名前を共有するかで決まる。`C2` は `(P-insert-net)` を
満たしながら (`L5`) この不等式を破るので、`held` の推移についての言明からこの不等式へ渡る段には、
名前についての事実が要る。第 7 節がその事実を与える。

## 7. `insert_rc` が置く `Retain` の位置

台帳形が名前を要求する以上、`insert_rc` の側で書けるのは「どの `Retain` がどの構文の直前に立つか」で
ある。この節はそれを証明する。

### 7.1 `L8` (`Retain` を作る位置は 4 つである)

**言明**。`insert_rc` が `RcExpr::Retain` を作る位置は `build_retains` の呼び出し 2 か所だけであり、
その呼び出し元は次の 4 か所である。

1. `RcInserter::insert_into_operation_let` の `build_retains(retains_before, node)`。`node` は
   その呼び出しが作った `Let(x, rhs, cont)` である。
2. `RcInserter::insert_into_expr_inner` の `RcExpr::Ret(x)` の腕の `retain_if_live(&x, live_after, ret)`。
   `ret` はその腕が作った `Ret(x)` である。
3. `RcInserter::insert_into_destructure` の `retain_if_live(&container, &live_cont, node)`。`node` は
   その呼び出しが作った `Destructure(container, fields, Unknown, cont)` である。
4. `RcInserter::insert_into_match` の `retain_if_live(&scrut, &live_at_arm_head, node)`。`node` は
   その呼び出しが作った `Let(x, Match(scrut, new_arms), cont)` である。

**証明**
`RcExpr::Retain` を作るのは `build_retains` だけである (`CODE src/rc_ir/rc_insert.rs: build_retains`
以外に `RcExpr::Retain` を構成する式は `src/rc_ir/rc_insert.rs` に無い)。`build_retains` を呼ぶのは
`insert_into_operation_let` と `retain_if_live` の 2 か所であり (`CODE src/rc_ir/rc_insert.rs:
RcInserter::retain_if_live`)、`retain_if_live` を呼ぶのは上の 2・3・4 の 3 か所である。∎

### 7.2 `L9` (`Retain` はその変数を名指す構文の直前に立つ)

**言明**。`insert_rc` の出力の各本体の各 `Retain` 節点 `t` について、`t` は `Retain(v, [],
RcState::Unknown, k)` の形であり、`k` から継続を辿って最初に現れる `RcExpr::Retain` でない節点 `n_t` は
次のいずれかである。いずれの場合も `n_t` は `v` を名指す。

- **(a)** `Let(x, rhs, k')` であって、`rhs_operands(rhs)` が `(v, Ownership::Own)` を含む。
- **(b)** `Ret(v)` であって、その `Ret` は `Match` のアーム本体の終端である。
- **(c)** `Destructure(v, fs, s, k')`。
- **(d)** `Let(x, Match(v, arms), k')`。

**証明**

<1>1. `build_retains(vars, cont)` が作る各 `Retain` 節点は `Retain(v, vec![], RcState::Unknown, ·)` の
      形であり、その継続は同じ呼び出しが作る次の `Retain` 節点か `cont` である。
  BY CODE src/rc_ir/rc_insert.rs: build_retains
  `vars.into_iter().rev().fold(cont, ...)` は `cont` から始めて外へ向かって節点を積む。

<1>2. `insert_rc` は、一度作った節点の継続を書き換えない。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func
  これらの関数は `RcExprNode` を作って返すだけで、返した節点の継続を差し替える式を持たない。呼び出し元が
  するのは、返された節点を別の構成子の継続 (`cont` または `node`) として渡すことだけであり、
  `build_releases` と `build_retains` も渡された節点を継続として**包む**。よって出力の `Retain` 節点の
  継続は、それが作られた時点の継続である。

<1>3. CASE `t` が `L8` の 1 で作られた。
  BY L8, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
  `build_retains(retains_before, node)` の `node` は `Let(x, rhs, cont)` であり、`rhs` は `Match` では
  ない (この関数の冒頭の表明)。`n_t` はこの `Let` である。`retains_before` に入るのは、`rhs_operands(rhs)`
  が `Ownership::Own` を与えるオペランド `v` だけである (同関数のループの `else if` の枝)。これが (a) で
  ある。

<1>4. CASE `t` が `L8` の 2 で作られた。
  <2>1. `n_t` は `Ret(v)` である。
    BY L8, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の
       `RcExpr::Ret(x)` の腕
    `retain_if_live(&x, live_after, ret)` に渡る `ret` はこの腕が作った `Ret(x)` であり、`v = x` で
    ある。
  <2>2. 本体の根を書き換える呼び出しの `live_after` は空集合である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func, CODE src/rc_ir/rc_insert.rs: insert_rc
    関数については `insert_into_func` が `self.insert_into_expr(func.body, &Set::default())` を呼び、
    グローバル初期化子については `insert_rc` が `inserter.insert_into_expr(glob.init, &Set::default())` を
    呼ぶ。
  <2>3. `Let`(`Match` でない右辺)、`Destructure`、`Eval`、`Let(x, Match(..), k)` のいずれについても、
        継続を書き換える呼び出しの `live_after` は、その節点を書き換える呼び出しの `live_after` に
        等しい。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    4 つとも `self.insert_into_expr(cont, live_after)` を呼ぶ。
  <2>4. 本体の終端の `Ret` を書き換える呼び出しの `live_after` は空集合である。
    BY <2>2, <2>3
    本体の終端の `Ret` は、根から継続を辿って着く節点である (D2 -- `Ret` を除く 5 種はちょうど 1 つの
    継続を持ち、アーム本体はそのアーム本体の終端の `Ret` で終わる)。継続の鎖の長さについての帰納で、
    鎖の各節点を書き換える呼び出しの `live_after` は根のもの、すなわち空集合である。
  <2>5. 本体の終端の `Ret` では `retain_if_live` は節点をそのまま返す。
    BY <2>4, CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live
    `live.contains(&var.name)` は空集合について偽である。
  <2>6. QED
    BY <2>1, <2>5, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `retain_if_live` がこの位置で発火する `Ret` は、本体の終端の `Ret` ではない。`Ret` を書き換える
    呼び出しに空でない `live_after` が渡るのは、`insert_into_match` が
    `self.insert_into_expr(arm.body, &live_after_match)` でアーム本体を書き換える枝を経由したときだけで
    あり (<2>2 と <2>3 より、他の枝は根の空集合を運ぶ)、その枝の下で終端になる `Ret` はアーム本体の
    終端の `Ret` である。これが (b) である。

<1>5. CASE `t` が `L8` の 3 で作られた。
  BY L8, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure
  `node` は `Destructure(container, fields, RcState::Unknown, cont)` であり、`v = container` である。
  これが (c) である。

<1>6. CASE `t` が `L8` の 4 で作られた。
  BY L8, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `node` は `Let(x, RcRhs::Match(scrut, new_arms), cont)` であり、`v = scrut` である。これが (d) で
  ある。

<1>7. QED
  BY L8, <1>3, <1>4, <1>5, <1>6
  `L8` が場合を尽くす。

### 7.3 `L9` の系: `n_t` は `v` の参照を処分するか移す

**言明**。`L9` の各場合について、`n_t` は `v` の inhabited な各 boxed leaf の参照を、D9 の意味で消費する
か移動させる。

**証明**

<1>1. CASE (a)。`rhs_operands` が `Ownership::Own` を与えるオペランドは、`Var` の被移動変数、`App` の
      callee と各引数、`Closure` の各 capture、`Llvm` の `borrows_operand(i)` が偽であるオペランドの
      いずれかである。
  BY CODE src/rc_ir/rc_insert.rs: rhs_operands
  `Var` の腕は唯一のオペランドに `Own` を与え、`App` の腕は callee と全引数に `Own` を与え、`Closure` の
  腕は全 capture に `Own` を与え、`Llvm` の腕は `borrows_operand(i)` が真のものにだけ `Borrow` を与える。
  `Match` の腕はこの関数に来ない。
  <2>1. `Var` の被移動変数の各 leaf は D9 の移動である。
    BY D9
    移動の表の `Let(x, Var(y), k)` の行。
  <2>2. `App` の callee の各 leaf は D9 の消費である。
    BY D9
    消費の表の `App` の行の前半。
  <2>3. `App` の引数の各 leaf は D9 の消費である。
    BY D9, A1, D14, CODE src/rc_ir/borrow.rs: split_rc_units, CODE src/rc_ir/rc_insert.rs: insert_rc
    消費の表の `App` の行の後半は「呼び出し先がその位置の unit を所有する」引数の leaf を挙げる。
    A1 の後半より `borrow_ify` の入力のすべての関数の `borrowed_units` は空であり、`borrow_ify` の入力は
    `split_rc_units` の出力である。`split_rc_units` も `insert_rc` も各関数の `body` と各グローバル
    初期化子の `init` しか書き換えないので、`insert_rc` の出力でも `borrowed_units` は空であり、D14 より
    すべての位置が所有される。
  <2>4. `Closure` の capture の各 leaf は D9 の消費である。
    BY D9
    消費の表の `Closure` の行。
  <2>5. `Llvm` の `borrows_operand(i)` が偽であるオペランドの各 leaf は、D9 の消費か移動である。
    BY D9
    消費の表の `Llvm` の行は、`result_prov` が単一の `Arg(i, σ)` として素通しを宣言していない leaf を
    挙げる。素通しを宣言している leaf は移動の表の `Llvm` の行にある。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. CASE (b)。アーム本体の終端の `Ret(v)` は D9 の移動である。
  BY D9
  移動の表の「`Match` のアーム本体の `Ret(x)`」の行。

<1>3. CASE (c)。`Destructure(v, fs, s, k')` は `v` の各 leaf を消費するか移動させる。
  BY D9
  `v` が boxed なら消費の表の「`Destructure`、`c` が boxed」の行が全 leaf を挙げる。`v` が unbox なら、
  名前の付いていないフィールドの leaf は消費の表の行に、名前の付いたフィールドの leaf は移動の表の
  「unbox 容器の `Destructure` の名前付きフィールド」の行にある。

<1>4. CASE (d)。`Let(x, Match(v, arms), k')` の選ばれたアームの入口で、`v` の各 leaf の参照は移動するか、
      `insert_into_match` が置いた `Release(v, [])` が処分する。
  BY D9, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `v` が unbox union のとき、変位アームの payload 束縛と catch-all アームの payload 束縛は移動の表の
  行である。`v` が boxed union のとき、`insert_into_match` は `release_container` が真になり、
  各変位アームの先頭に `Release(v, [])` を置く (`head.push(scrut.clone())` と `build_releases(head, body)`)。
  catch-all アームでは payload 束縛が移動の表の行である。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, L9
  `L9` の 4 つの場合が尽くす。

### 7.4 `L10` (`C1` と `C2` の `main` は `insert_rc` の出力ではない)

**言明**。`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` の `main` の本体と、第 7.5.8 節の `C2` の
`main` の本体は、どちらも `insert_rc` の出力ではない。

**証明**

<1>1. `C1` の `main` の `Retain(m, [], s, k)` について、`k` は `Let(u, App(f, [p]), ·)` である。
  BY p13 の第 7.5.7 節 (`C1` の `main` の本体)

<1>2. `C1` の `main` は `L9` を破る。
  BY <1>1, L9, CODE src/rc_ir/rc_insert.rs: rhs_operands
  `n_t = Let(u, App(f, [p]), ·)` は `L9` の (a) の形だが、`rhs_operands(App(f, [p]))` は
  `[(f, Own), (p, Own)]` であって `m` を含まない。(b)(c)(d) の形でもない。

<1>3. `C2` の `main` の `Retain(o, [], s, k)` について、`k` は `Let(u, App(f, [y]), ·)` である。
  BY p13 の第 7.5.8 節 (`C2` の `main` の本体)

<1>4. `C2` の `main` は `L9` を破る。
  BY <1>3, L9, CODE src/rc_ir/rc_insert.rs: rhs_operands
  `rhs_operands(App(f, [y]))` は `[(f, Own), (y, Own)]` であって `o` を含まない。

<1>5. QED
  BY <1>2, <1>4, L9
  `L9` は `insert_rc` の出力のすべての `Retain` について成り立つ。

### 7.5 `L11` (`L9` の形は `cancel` の入力まで残る)

**言明**。`cancel` の入力の各本体の各 `Retain` 節点 `t = Retain(v, π, s, k)` について、`k` から継続を
辿って最初に現れる `RcExpr::Retain` でない節点 `n_t` は `v` を名指し、`L9` の (a)-(d) のいずれかの形で
ある。

**証明**

<1>1. `split_rc_units` は `Retain(v, π, s, k)` を `units_under(ty(v), π)` の各 unit の `Retain` 節点の
      鎖に置き換え、他の節点は継続を書き換えた上でその場に作り直す。
  BY CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/borrow.rs: split_rc
  `split_rc` は `fold` で `k` の外へ節点を積む。他の腕は同じ種類の節点を `split_body` した継続の上に
  作り直す。よって `Retain` の鎖の後に来る節点の種類・変数・並びは変わらない。

<1>2. `borrow_ify` の `rewrite_rc` は `Retain(v, π, s, k)` を `units_under(ty(v), π)` の部分列の
      `Retain` 節点の鎖に置き換える。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  借用版でないときは同じ節点を作り直し、借用版のときは `owns_unit` が真の unit だけを残して鎖にする。
  鎖が空になることはあるが、後に来る節点は変わらない。

<1>3. `borrow_ify` の `rewrite_inner` は、`Let(x, App(callee, args), k)` について `call_rc` の `before` の
      `Retain` 節点をその `Let` 節点の直前に積み、`after` の `Release` 節点をその `Let` 節点と継続の間に
      置く。他の節点は継続を書き換えた上でその場に作り直す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: prepend_rc
  `prepend_rc(before, false, app)` は `app` の外へ `Retain` を積み、`prepend_rc(after, true,
  self.rewrite(k))` は継続の外へ `Release` を積む。

<1>4. `before` の各 `Retain(a, u)` について、その直後の非 `Retain` 節点はその `App` の `Let` であり、
      `a` はその `args` の元である。
  BY <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
  `call_rc` は `args` の元だけを `before` に入れる。`rhs_operands(App(callee, args))` は `args` の各元に
  `Ownership::Own` を与えるので、これは `L9` の (a) の形である。

<1>5. `clone_func` が作る借用版は元の本体の束縛変数を一斉に付け替えたものである。
  BY P9
  付け替えは節点の種類・並び・どの変数を名指すかを変えないので、`L9` の形は保たれる。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, L9
  `insert_rc` の出力が持つ形 (`L9`) は `split_rc_units` (<1>1) と `borrow_ify` (<1>2、<1>3、<1>5) を
  通って残り、`borrow_ify` が足す `Retain` も同じ形を持つ (<1>4)。

## 8. (O1) と (O2) の言明

`L7` より、A19 (ii-b) は各別名類 `C` と `bumps ≥ 1` である各時点について `U + X ≥ D` である。`L9` と
`L11` は、`R` を増やす節点 -- `Retain` -- が、その変数を名指す構文の直前にしか立たないことを示す。
README の第 4 節が (O1) と (O2) と呼ぶ 2 つを、この節が言明として書く。

**(O1) 由来の形。** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、(a) 各時点の
各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` であり、(b) D7 の読む構文が読む値の各スロット、および
`Retain(v, π)`・`Release(v, π)` が触れる各スロットについて、そのスロットが属する計数下の別名類は、その
節点の入口で `held ≥ 1` である。これは D11 を別名類の粒度へ絞った主張であり、A19 (ii-a) が要求する
ものである。**第 10 節の `L19` が示す。** 併せて `L20` が、`L4` と `L7` が前提に持つ「各別名類の生成
事象は `ρ` の上に高々 1 つ」を示す。

**(O2) 帳簿の遅れが無いこと。** 各時点の各計数下の別名類について、`bumps ≥ 1` ならば
`held ≥ 1 + bumps` である。`L7` より `U + X ≥ D` と同値であり、A19 (ii-b) が要求するものである。
**第 11 節の `L25` が、前提 (N) の下で示す。**

### 8.1 (O2) の証明が立つ 3 つの事実

台帳が処分に遅れうるのは、処分されるスロットの名前を、pending な要素の `outstanding` が名指さないとき
だけである (`L23`)。第 11 節が使う事実は次の 3 つである。

1. **上流の名前に bump が付くのは、`insert_rc` がその変数の後の使用のために `Retain` を置いたときで
   あり、その `Retain` は同じ変数を名指す構文の直前に立つ (`L9`)。** その構文が消費するときは消費が
   要素を落とし、移動するときは移動先のスロットが増える。どちらでも、その名前を `Anc` に持つスロットの
   個数と、その名前に付く bump の個数が同じだけ動く。
2. **`Join` で名前が切り替わるとき、上流のスロットは消えるか、アームの中の `Retain` を伴う。**
   アームの結果が `Match` の後でも live であるとき `insert_rc` は `retain_if_live(&x, live_after, ret)` で
   `Retain` を置き (`L9` の (b))、その要素はアームの中で作られるので `merge` の `entered_with` の
   ゲートが `pending` から落とす。live でないときは上流のスロットが消える。
3. **下流のスロットの処分は上流の名前にも届く。** `origin_inner` の `Binding::Join` の腕は各アームの
   結果の `acted_on()` を候補に積むので、下流のスロットの `acted_on` は上流の名前をすべて含む
   (`L21` (a))。よって遅れが生じうるのは上流のスロットの処分だけである。

第 11 節はこの 3 つを、区画の木の各部分木についての 1 つの不等式 (`L24`) に束ねる。

### 8.2 `split_rc_units` の段は塞がっていない

A19 (ii-a) と (ii-b) の範囲は「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体」で
あり、`borrow_ify` の入力は `split_rc_units` の出力である。第 10 節と第 11 節が示すのは `insert_rc` の
出力についてであり、`split_rc_units` が `Retain(v, [])` を unit ごとの鎖へ割る段は扱っていない。
`L11` は `L9` の形がその段を通ることを示すが、(O1) と (O2) そのものについては示していない。A19 の
果たす者に `split_rc_units` は挙がっていない。

## 9. A19 を読む 2 つの形

README の A19 は 2 種の読み手を持つ。P18a・P18c・P19・P21 が読むのは (ii-b) -- 走査の `pending` を
主語にする形 -- であり、P14 と P18c が読むのは次の形 ((ii-a)) である。

**DEF 由来の形**。`ρ` の上の各時点 `τ` について次の 2 つが成り立つこと。

- **(a)** 各計数下の別名類 `C` について `held_ρ(τ, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その時点で `held_ρ(τ, C) ≥ 1` である。

「由来」-- `origin` の再帰を実行が辿った枝に沿って追い切った先 -- は `p13-disposals-and-pending.md` の
ρ-終端であり、別名類は ρ-終端が等しいスロットの集まりなので (p13 の DEF 別名類)、「由来ごとの参照の
個数」は `held_ρ(·, C)` である。

**この 2 つの形はどちらも他方を導かない。** 9.1 と 9.2 がそれぞれの向きの反例を挙げる。

### 9.1 `L12` (`C1` は由来の形を満たし、A19 (ii) を破る)

**言明**。`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` の `main` は、変位 0 を選ぶ実行路で
由来の形を満たす。同じ実行路で A19 (ii) を破る。

**証明**

<1>1. その実行路の計数下の別名類は `C_1 = {(p, []), (m, [])}` と `C_2 = {(q, [])}` である。
  BY p13 の第 7.5.7 節 (`m` と `p` が 1 つの別名類に属すること), D4, D26
  `ty(c)` と payload の型 `Bl` の変位は boxed leaf を持たないので、`c`、`y0`、`y1` はスロットを持たない。
  `ty(u)` は `I` で leaf を持たない。`App` の callee の leaf が指すオブジェクトはグローバル状態であり
  (p13 の第 7.5.7 節)、D26 より計数下ではない。

<1>2. `held(C_1)` は、割り当ての後 1、`Retain(m, [])` の後 2、`App(f, [p])` の消費の後 1、
      `Release(m, [])` の後 0 である。`held(C_2)` は、割り当ての後 1、`App(f, [q])` の消費の後 0 で
      ある。
  BY <1>1, L4, p13 の DEF 類ごとの参照
  `p` と `q` の割り当ては D10 の生成であり、開始値 1 を与える。`Retain(m, [])` は `(m, []) ∈ C_1` を
  名指し、`Release(m, [])` も同じである。`App(f, [p])` は `(p, []) ∈ C_1` を、`App(f, [q])` は
  `(q, []) ∈ C_2` を消費する (D9、`f` は `b` を所有する)。

<1>3. 由来の形の (a) が成り立つ。
  BY <1>2
  <1>2 の値はすべて 0 以上である。

<1>4. 由来の形の (b) が成り立つ。
  BY <1>1, <1>2, D7
  読む構文と `Retain`/`Release` を順に見る。`Let(m, Match(c, ...))` が読むのは `c` であり、<1>1 より
  `c` はスロットを持たない。`Retain(m, [])` は `(m, [])` に触れ、その時点の `held(C_1) = 1` である。
  `Let(u, App(f, [p]), ·)` は callee と `p` を読み、`p` の側は `held(C_1) = 2` である
  (callee の側は <1>1 より計数下でない)。`Let(w, App(f, [q]), ·)` は `q` を読み `held(C_2) = 1` で
  ある。`Eval(m)` は `m` を読み `held(C_1) = 1` である。`Release(m, [])` は `(m, [])` に触れ
  `held(C_1) = 1` である。終端の `Ret(u)` は D7 の読む構文ではない。

<1>5. `C1` は A19 (ii) を破る。
  BY p13 の第 7.5.7 節
  `App(f, [p])` の消費の後、`Retain(m, [])` の要素は `pending` に残り `bumps(C_1) = 1` であるのに
  `held(C_1) = 1` である。第 1 節の読みでは `bumps ≥ 1` のとき `held ≥ 1 + bumps` が要る。

<1>6. QED
  BY <1>3, <1>4, <1>5

### 9.2 `L13` (A19 (ii) を満たし、由来の形を破る本体 `B_1`)

4.1 の道具立てに、関数 `id` (パラメータ `a : Arr`、`borrowed_units` は空、返り値の型 `Arr`、本体
`Ret(a)`) を足す。本体 `B_1` (`main`、パラメータ無し、返り値の型 `I`) を次で定める。

```
Let(o, Llvm(alloc, []),
Let(y, App(id, [o]),
Eval(o,
Let(u, App(f, [y]),
Ret(u)))))
```

**言明**。`B_1` は D12 と A19 (ii) を満たし、由来の形の (b) を破る。`B_1` は `insert_rc` の出力では
ない。

**証明**

<1>1. `B_1` の実行路は 1 本であり、その上の計数下の別名類は `C_o = {(o, [])}` と `C_y = {(y, [])}` で
      ある。
  BY D3, D4, D26, A3, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     CODE src/rc_ir/ownership.rs: as_arg_projection
  `Match` が無いので実行路は 1 本である。`collect_bindings` は `Let(o, Llvm(alloc, []), ・)` に
  `Binding::Llvm(alloc, [], Arr)` を入れる。A3 と 4.1 より `alloc` の宣言は単一の `Fresh` なので
  `as_arg_projection` は `None` を返し、`origin_from_leaves_under` はオペランドを持たないこの op に
  ついて `Exactly((o, []))` を返す。よって `origin(o, []) = Exactly((o, []))` である。`y` は `RcRhs::App` に束縛されるので `collect_bindings` は `Binding::Producer` を入れ、
  `origin_inner` の `Producer` の腕は `here()` を返して `origin` を呼ばない。よって `(o, [])` と
  `(y, [])` はどちらも ρ-終端であり、別々の類である。`ty(u) = I` は leaf を持たない。

<1>2. `Obl` と `H(O)` は、割り当ての後 `{O}, 1`、`App(id, [o])` が返った後 `{O}, 1`、
      `App(f, [y])` が返った後 `{}, 0` である。
  BY D9, D10, A1
  `App(id, [o])` は `(o, [])` を消費し (`id` は `a` を所有する)、`H` を動かさない。同じ節点の
  D10 の生成の表の `App` の行が結果の leaf `(y, [])` の参照を作り、`H` はここでも動かない。
  `App(f, [y])` は `(y, [])` を消費し、`f` の `Release(b, [])` が `H` を 1 下げる。

<1>3. `B_1` は D12 を満たす。
  BY <1>2
  (S-a): 取り除かれる参照はどちらの消費でもその時点の `Obl` に入っている。(S-b): 終端の `Ret(u)` の
  時点で `Obl` は空であり、`ty(u) = I` は leaf を持たない。(S-c): `App(id, [o])` は `H(O) = 1` の下で
  `o` を、`Eval(o)` は `H(O) = 1` の下で `o` を、`App(f, [y])` は `H(O) = 1` の下で `y` を読む。

<1>4. `B_1` は A19 (ii) を満たす。
  BY <1>1, L4, 第 1 節の読み
  `B_1` に `Retain` 節点は無いので、走査の `pending` は空のままであり `bumps ≡ 0` である
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` -- `pending` に要素を足すのは
  `RcExpr::Retain` の腕だけである)。`held(C_o)` は 1 の後 `App(id, [o])` の消費で 0、`held(C_y)` は
  生成で 1 の後 `App(f, [y])` の消費で 0 であり、どちらも 0 以上である。`bumps ≥ 1` の時点は無い。

<1>5. `B_1` は由来の形の (b) を破る。
  BY <1>1, <1>4, D7
  `Eval(o, ·)` は D7 の読む構文であり、`o` の leaf `[]` を読む。そのスロットの類は `C_o` であり、
  その時点の `held(C_o) = 0` である。

<1>6. `B_1` は `insert_rc` の出力ではない。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, CODE src/ast/name.rs: FullName::is_local,
     CODE src/rc_ir/rc_insert.rs: build_retains
  `Let(y, App(id, [o]), cont)` を書き換える呼び出しの `live_cont` は `o` を含む -- `insert_into_eval` は
  返す `live_before` に `o` を入れる。`o` は局所名であり (`is_local` は名前空間が空かを答え、局所変数の
  名前は名前空間を持たない)、`rhs_operands(App(id, [o]))` は `o` に `Ownership::Own` を与え、
  `needs_rc(o)` は真である (`ty(o) = Arr` は boxed なので `is_fully_unboxed` は偽)。よって
  `retains_before` に `o` が入り、`build_retains` が `Retain(o, [])` をこの `Let` の直前に置く。
  `B_1` はその節点を持たない。

<1>7. QED
  BY <1>3, <1>4, <1>5, <1>6

### 9.3 2 つの形の関係

`L12` は「由来の形 ⇒ A19 (ii-b)」の反例であり、`L13` は「A19 (ii-b) ⇒ 由来の形」の反例である。よって
**2 つの形はどちらも他方を導かない。** 1 つの仮定として置くと、P14 が読む形は文面に無く、
P18a・P19・P21 が読む形は P14 には強すぎる。

`insert_rc` の側の義務は、この 2 つに分かれる。

- **由来の形 ((ii-a))** = 第 8 節の **(O1)**。走査を読まないので、`insert_rc` の liveness の規律だけで
  書ける。`C2` はこれを破り (`Retain(o, [])` の時点で `held(C_o) = 0`)、`L10` がその形を弾く。
- **帳簿の形 ((ii-b))** = **(O1) + (O2)**。`L7` の恒等式より、(O1) の下でこの形は `U + X ≥ D` に
  等しい。`C1` はこれだけを破る (`L12`)。

**第 10 節が (O1) を、第 11 節が (O2) を示す。** よって `insert_rc` の出力について A19 の 2 つの形は
どちらも成り立つ -- (ii-a) は無条件に、(ii-b) は前提 (N) の下で。

## 10. (O1) の証明 -- 別名類の粒度の RC 規律

この節は `insert_rc` の出力について (O1) を示す。支えるのは `insert_rc` が持つ 1 つの等式である --
**各時点において、1 つのスロットに割り当たる参照の個数は、その変数が `insert_rc` の liveness で live
かどうかで決まる。** `insert_rc` の liveness は使用だけから作られる集合であり、参照カウントも義務集合も
見ない。その集合と実行時の参照の分布が一致する、というのがこの等式である。(O1) の 2 つの節はここから
出る。

### 10.1 塊、検査点、live 集合、割り当て

`insert_rc` は骨格 (第 1 節) の各節点を、次の 3 つを並べた列へ写す
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner`)。

- **前置 `Retain` 鎖**: `build_retains` が積む `Retain` 節点の列 (空のこともある)。
- **核節点**: 骨格節点と同じ種類の節点 (`Let`、`Destructure`、`Eval`、`Ret`)。
- **後置 `Release` 鎖**: `build_releases` が積む `Release` 節点の列 (空のこともある)。

骨格節点 `m` の**検査点**とは、`m` の前置 `Retain` 鎖の最初の節点 (鎖が空なら核節点) の入口をいう。

**DEF `Λ(m)`、`A(m)`**。骨格節点 `m` を書き換える `insert_into_expr(m, live_after)` の呼び出しが返す
第 2 成分 (`live_before`) を `Λ(m)`、渡された `live_after` を `A(m)` と書く
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr`)。

**DEF 割り当て `μ`**。活性化を固定する。`ρ` の上の各時点 (節点の訪問の入口) `τ` と各スロット `(v, λ)`
(D6) について、整数 `μ_τ(v, λ)` を次の規則で定める。値は活性化の開始時にはすべて 0 であり、`ρ` の上の
次の 6 種の事象がそれを動かす。

- **D10 の初期値**: 所有する各パラメータ・capture の inhabited な各 leaf `(p, λ)` について `+1`。
- **D10 の生成**: 生じた inhabited な各 leaf `(v, λ)` について `+1`。
- **`Retain(v, π)`**: `π` の下の inhabited な各 leaf `λ` について `μ(v, λ)` を `+1`。
- **`Release(v, π)`**: 同じ leaf について `-1`。
- **D9 の消費**: 消費される inhabited な各 leaf `(v, λ)` について `-1`。
- **D9 の移動**: 移動元の leaf `(v, λ)` について `-1`、移動先の leaf `(x, λ')` について `+1`。

D9 の末尾の段落が「上の 2 つの表と D10 の生成の表で、参照を作る・移す・手放す構文はすべてである」と
述べるとおり、活性化が保持する参照を動かす事象はこの 6 種で尽きる。移動の辺は別名の辺 (D20) であり
移動元と移動先は同じ類に属するので、移動は 1 つの類の中の持ち手を替えるだけである。よって各計数下の
別名類 `C` と各時点 `τ` について
`held_ρ(τ, C) = Σ_{(v, λ) ∈ C} μ_τ(v, λ)`
である -- 右辺を動かす事象と、`p13-disposals-and-pending.md` の `DEF 類ごとの参照` が
`held_ρ(・, C)` を動かす事象は、移動を除いて同じであり、移動は右辺を変えない。

**DEF `N_ρ(m, C)`**。`κ_C(v) := #{λ : λ は ty(v) の inhabited (D16) な boxed leaf で (v, λ) ∈ C}` と
置き、骨格節点 `m` について `N_ρ(m, C) := Σ_{v ∈ Λ(m)} κ_C(v)` と置く。

### 10.2 `L14` (`live_before` は自由変数と `live_after` の和である)

**言明**。`insert_rc` が骨格節点 `m` を `live_after = A(m)` の下で書き換えるとき、
`Λ(m) = free_locals(m) ∪ A(m)` である (`CODE src/rc_ir/rc_insert.rs: free_locals`)。

**証明**

<1>1. `free_locals(m)` は、`m` を根とする部分木が参照する局所名から、その部分木が束縛する局所名を
      除いたものである。
  BY CODE src/rc_ir/rc_insert.rs: free_locals,
     CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound
  `free_locals` は `collect_referenced_and_bound` で `refs` と `bound` を集め、`refs` から `bound` の
  元を落とす。`collect_referenced_and_bound` は `Ret` の変数、`Let` の右辺の各変数 (`Match` の
  scrutinee を含む)、`Destructure` の容器、`Retain`/`Release`/`Eval` の変数を `refs` に入れ、`Let` の
  束縛変数、`Match` の各アームの payload、`Destructure` の各フィールド変数を `bound` に入れ、継続と
  アーム本体へ降りる。局所名の判定は `insert_if_local` が行う
  (`CODE src/rc_ir/rc_insert.rs: insert_if_local`)。

<1>2. `m` が本体の根であるとき `A(m) = ∅` である。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func, CODE src/rc_ir/rc_insert.rs: insert_rc
  関数は `insert_into_expr(func.body, &Set::default())`、グローバル初期化子は
  `inserter.insert_into_expr(glob.init, &Set::default())` で呼ばれる。

<1>3. `m` の部分木の中で束縛される名前は `A(m)` に入らない。
  BY <1>2, A6, A11, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `A(m)` は、根では空であり (<1>2)、継続については囲む節点の `A` そのもの
  (4 つの関数はいずれも `self.insert_into_expr(cont, live_after)` を呼ぶ)、アーム本体については
  `live_after_match = live_cont \ {x}` である。どの場合も `A(m)` の名前は `m` より後ろの位置で使われる
  名前だけからなる。A11 より名前の使用はその位置でスコープに入っている束縛に解決するので、`m` の
  部分木の中で束縛される名前は `m` より後ろではスコープに無く、A6 より同名の別の束縛も無い。

<1>4. 部分木の節点数についての帰納で言明が成り立つ。
  <2>1. CASE `m = Ret(x)`。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この腕は `live = live_after.clone()` に `insert_if_local(&mut live, &x.name)` で `x` を足したものを
    返す。<1>1 より `free_locals(Ret(x))` は `x` が局所名なら `{x}`、そうでなければ空である。
  <2>2. CASE `m = Let(x, rhs, cont)` で `rhs` が `Match` でない。
    BY <2>7, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: insert_if_local,
       <1>1, <1>3
    この関数は `insert_into_expr(cont, live_after)` を呼び、返った `live_cont` から `x` を除き、
    `rhs_operands(rhs)` の各オペランドの局所名を足したものを返す。帰納法の仮定より
    `live_cont = free_locals(cont) ∪ A(m)`。`rhs_operands` が挙げるオペランドは `rhs` が参照する変数の
    すべてである -- `Var` は被移動変数、`App` は callee と全引数、`Closure` は全 capture、`Llvm` は
    全オペランドを挙げ、`Match` はこの関数に来ない。よって `Λ(m)` は
    `((free_locals(cont) ∪ A(m)) \ {x}) ∪ ops` であり、<1>3 より `x ∉ A(m)` なので
    `(free_locals(cont) \ {x}) ∪ ops ∪ A(m)` に等しい。<1>1 よりこの前 2 項は `free_locals(m)` である。
  <2>3. CASE `m = Destructure(container, fields, _, cont)`。
    BY <2>7, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>3
    この関数は `live_cont` から各フィールド変数を除き、`container` の局所名を足したものを返す。
    フィールド変数は <1>3 より `A(m)` に入らない。
  <2>4. CASE `m = Eval(x, cont)`。
    BY <2>7, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この関数は `live_cont` に `x` の局所名を足したものを返す。
  <2>5. CASE `m = Let(x, Match(scrut, arms), cont)`。
    BY <2>7, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>3, A6, A11
    この関数は `live_cont = insert_into_expr(cont, live_after)` を取り、
    `live_after_match = live_cont \ {x}` の下で各アーム本体を書き換え、返った `body_live` をすべて
    `live_before_arms` に集めて各アームの payload を除き、最後に `x` を除いて `scrut` を足す。
    帰納法の仮定より `live_cont = free_locals(cont) ∪ A(m)` であり、
    `body_live_j = free_locals(arm_j.body) ∪ live_after_match` である。payload は <1>3 より
    `live_after_match` に入らないので、`live_before_arms` は
    `(∪_j (free_locals(arm_j.body) \ {payload_j})) ∪ live_after_match` である。A6 と A11 より、ある
    アームが束縛する名前を別のアームや `cont` が参照することはないので、この和は
    `collect_referenced_and_bound` がアームについて集める `refs \ bound` と一致する。よって `Λ(m)` は
    `(∪_j (free_locals(arm_j.body) \ {payload_j})) ∪ (free_locals(cont) \ {x}) ∪ {scrut} ∪ A(m)` で
    あり、<1>1 よりこれは `free_locals(m) ∪ A(m)` である。
  <2>6. CASE `m` が `Retain` または `Release` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の
       `RcExpr::Retain(..) | RcExpr::Release(..)` の腕
    骨格はこの 2 種を含まない (第 1 節)。この腕は panic するので、この場合は起きない。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, D2, A15
    帰納は `m` の部分木の節点数について行う。<2>2 から <2>5 の各場合で帰納法の仮定を使うのは `cont` と
    アーム本体 -- どれも真に小さい部分木 -- についてだけであり、<2>1 が基底である。場合が尽きることは
    `RcExpr` が 6 個の構成子を持つこと (D2) と、`insert_into_expr_inner` の `match` がその 6 個を
    この 6 つの腕で覆うことによる。A15 より `insert_into_expr` は `insert_into_expr_inner` を
    ちょうど 1 回呼ぶ。

<1>5. QED
  BY <1>4

### 10.3 `L15` (`Λ` と `insert_rc` の出力についての 5 つの性質)

**言明**。骨格節点 `m` について次の 5 つが成り立つ。

- **(a)** `Λ(m)` の各名前は、`ρ` の上で `m` の検査点に至るまでに値を得ている。
- **(b)** `m` の核節点が名指す局所変数は、すべて `Λ(m)` に入る。「名指す」とは、`Ret(x)` の `x`、
  `Let` の右辺の各変数 (`Match` の scrutinee を含む)、`Destructure` の容器、`Eval` の変数を指す。
- **(c)** `needs_rc(v)` が偽の変数 `v` はスロットを持たない。
- **(d)** 局所でない名前のスロットは、計数下の別名類に属さない。
- **(e)** `insert_rc` の出力のすべての関数の `borrowed_units` は空であり、D14 よりそのすべての
  パラメータ・capture の unit はその関数が所有する。

**証明**

<1>1. (a)。
  BY L14, A11
  `L14` より `Λ(m) = free_locals(m) ∪ A(m)`。`free_locals(m)` の各名前は `m` の部分木が参照して束縛
  しない名前なので、A11 よりその束縛は `m` を囲むスコープに在り、`m` の検査点までに値を得ている。
  `A(m)` の各名前は `m` より後ろで使われる名前であり、同じ理由でその束縛は `m` を囲むスコープに在る。

<1>2. (b)。
  BY L14, CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound, A6
  核節点が名指す変数は、`collect_referenced_and_bound` が `refs` に入れる名前のうち、`m` 自身の
  節点が挙げるものである。これらは `m` 自身が参照する名前であって `m` の部分木が束縛する名前では
  ない -- `m` 自身の束縛変数 (`Let` の `x`、`Destructure` のフィールド変数、`Match` の payload) とは
  A6 より一致しない。よって `free_locals(m)` に入り、`L14` より `Λ(m)` に入る。

<1>3. (c)。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, D4, D6
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` であり、D4 の第 1 規則より `is_fully_unboxed` が
  真の型は boxed leaf を持たない。D6 よりスロットは boxed leaf についてのみ在る。

<1>4. (d)。
  BY D26, A8, CODE src/rc_ir/lower.rs: Lowerer::lower_var, CODE src/ast/name.rs: FullName::is_local
  `is_local` は名前空間が空かを答える。局所でない名前は、直接呼び出しが名指す関数か、グローバル値を
  読む `RcVar` である。D26 と A8 より、グローバル値が到達するオブジェクトはグローバル状態であり、
  D8 の意味の参照を持たない。

<1>5. (e)。
  BY A1, D14, CODE src/rc_ir/borrow.rs: split_rc_units, CODE src/rc_ir/rc_insert.rs: insert_rc
  A1 の後半より `borrow_ify` の入力のすべての関数の `borrowed_units` は空である。`borrow_ify` の入力は
  `split_rc_units` の出力であり、`split_rc_units` は各関数の `body` と各グローバル初期化子の `init` しか
  書き換えない。`insert_rc` も `body` と `init` しか書き換えず、`borrowed_units` を読み書きしない。
  よって `insert_rc` の出力でもすべての関数の `borrowed_units` は空であり、D14 よりすべての unit が
  所有される。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### 10.4 `L16` (借用するオペランドの leaf は素通しを宣言されない)

**言明**。`Llvm(gen, args)` について、`gen.borrows_operand(i, ・, ・)` が真であるとき、結果のどの leaf も
単一の `Arg(i, σ)` を宣言しない。

**証明**

<1>1. `borrows_operand` を override する `impl LLVMGen for` は 13 個であり、既定は偽を返す。
  BY CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
  既定の実装は `false` を返す。override するのは `src/fixstd/builtin.rs` の
  `InlineLLVMArrayUnsafeGetBoundsUnchecked`、`InlineLLVMArrayCopyCapacityBoundsUnchecked`、
  `InlineLLVMArrayGetPtrBody`、`InlineLLVMArrayGetSizeBody`、`InlineLLVMArrayGetCapacityBody`、
  `InlineLLVMStructGetBody`、`InlineLLVMCaptureProjectBody`、`InlineLLVMUnionAsBody`、
  `InlineLLVMUnionIsBody`、`InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody`、
  `InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody`、`InlineLLVMGetBoxedDataPtrFunctionBody`、
  `InlineLLVMArrayBorrowElementsBody` の 13 個である。

<1>2. `result_prov` の既定の実装は、結果のどの leaf にも `Unknown` だけを置く。
  BY CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, CODE src/rc_ir/provenance.rs: Provenance,
     CODE src/rc_ir/ownership.rs: as_arg_projection
  既定は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` であり、`uniform` は各 boxed
  leaf に `sole_origin(Unknown)` を置く。`as_arg_projection` は `LeafOrigin::Unknown` に `None` を
  返す。

<1>3. <1>1 の 13 個のうち `result_prov` を override するのは `InlineLLVMStructGetBody` と
      `InlineLLVMUnionAsBody` の 2 個である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov
  残る 11 個は `result_locality` を override するが `result_prov` は override しない。

<1>4. この 2 個は、`borrows_operand(i)` が真であるとき結果の型が `is_fully_unboxed` である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_operand
  前者の `borrows_operand` は
  `i == 0 && Self::borrows_container(&arg_tys[0].field_types(type_env)[self.field_idx], type_env)` で
  あり、`borrows_container(field_ty, ・)` は `field_ty.is_fully_unboxed(type_env)` である。この
  `field_ty` はこの op の結果の型 (読み出すフィールドの型) である。後者も同じ形で、`borrows_union` は
  payload の型 -- この op の結果の型 -- について `is_fully_unboxed` を問う。どちらも `i == 0` 以外の
  `i` には偽を返す。

<1>5. 結果の型が `is_fully_unboxed` であるとき、宣言はどの leaf にも何も置かない。
  BY D4, CODE src/rc_ir/provenance.rs: Provenance
  D4 の第 1 規則より `is_fully_unboxed` が真の型は boxed leaf を持たない。`Provenance::build_shape` と
  `Provenance::uniform` はどちらも型の boxed leaf の上に `LeafMap` を作るので、leaf を持たない型では
  空である。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5
  `borrows_operand(i)` が真になるのは <1>1 の 13 個のいずれかであり、そのうち 11 個は既定の
  `result_prov` を持ち `Arg` を宣言しない (<1>2、<1>3)。残る 2 個は、`borrows_operand(i)` が真である
  とき結果に leaf が無い (<1>4、<1>5) ので、やはり `Arg(i, σ)` を宣言する leaf を持たない。

**この補題が要る理由。** D9 の消費の表の `Llvm` の行は `borrows_operand(i)` が偽のオペランドだけを
挙げるが、移動の表の `Llvm` の行 (素通し leaf) はその条件を持たない。両方が同時に成り立つ op が在ると、
1 つの参照が結果へ移りながら呼び出し元にも残ることになり、`insert_rc` が置く「借用オペランドの最後の
使用の後の `Release`」がその参照を二重に処分する。`L16` はその形が無いことを言う。

### 10.5 `L17` (遷移は割り当てを liveness の指示関数へ運ぶ)

`ρ` の上で連続する 2 つの検査点の間に在る節点の列を**遷移**と呼ぶ。

**言明**。`insert_rc` の出力の 1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。

- **(a)** `ρ` の上の各節点の入口は、本体の根の検査点より前の `Release` 鎖 (`insert_into_func` の
  `unused` の鎖) の中の点か、連続する 2 つの検査点の間の遷移の中の点 (先の検査点を含む) か、
  関数本体・初期化子の終端の `Ret` の検査点のいずれかである。遷移は下の (T1)・(T2)・(T3) の 3 種で
  尽きる。
- **(b)** 検査点 `m` から検査点 `m'` への遷移について、`m` の時点で「各スロット `(v, λ)` の `μ` は、
  `v ∈ Λ(m)` のとき 1、そうでないとき 0 である」が成り立つならば、`m'` の時点で同じことが `Λ(m')`
  について成り立つ。さらに遷移の中の各節点の入口で、すべてのスロットについて `μ ≥ 0` であり、その
  節点が読む値の各スロットと `Retain`/`Release` が触れる各スロットについて `μ ≥ 1` である。

**証明**

<1>1. (a)。遷移は次の 3 種で尽きる。
  <2>1. **(T1)** `m` が `Let(x, rhs, cont)` (`rhs` は `Match` でない)、`Destructure(c, fs, _, cont)`、
        `Eval(x, cont)` のいずれかであるとき。遷移は `m` の前置 `Retain` 鎖、核節点、後置 `Release` 鎖
        であり、`m'` は `cont` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval
    3 つとも、`insert_into_expr(cont, ・)` が返した節点の外へ後置 `Release` 鎖を積み、その外に核節点を
    作り、その外に前置 `Retain` 鎖を積む。
  <2>2. **(T2)** `m` が `Let(x, Match(scrut, arms), cont)` であるとき。`ρ` が選ぶアームを `j` とすると、
        遷移は `m` の前置 `Retain` 鎖 (`retain_if_live(&scrut, &live_at_arm_head, ・)` が置く高々 1 つの
        `Retain`)、核節点 `Let(x, Match(scrut, arms'), ・)`、アーム `j` の頭の `Release` 鎖であり、
        `m'` はアーム `j` の本体の根である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, D3
    `insert_into_match` は各アームの本体を `build_releases(head, body)` で包む。D3 より
    `Let(x, Match(v, arms), k)` の実行路はアームを 1 つ選んでその本体へ進む。
  <2>3. **(T3)** `m` がアーム本体の終端の `Ret(r)` であるとき。遷移は `m` の前置 `Retain` 鎖
        (`retain_if_live(&r, live_after, ret)` が置く高々 1 つの `Retain`)、核節点 `Ret(r)`、および
        その `Match` の核節点と `cont` の間に置かれた `Release` 鎖 (`x` が `live_cont` に入らないときの
        `Release(x, [])`) であり、`m'` はその `Match` の `cont` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, D3
    D3 より、アーム本体の実行路を辿り終えると `k` へ進む。`insert_into_match` は
    `!live_cont.contains(&x.name) && self.needs_rc(&x)` のとき `build_releases(vec![x], cont)` を置く。
  <2>4. QED
    BY D2, D3, <2>1, <2>2, <2>3, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func
    出力の各節点は、`insert_into_func` が積む `unused` の解放鎖に属するか、いずれかの骨格節点の塊に
    属する (10.1 節)。塊は検査点で始まるので、各塊は 1 つの検査点を持ち、その検査点から次の検査点
    までが遷移である。
    D2 より骨格節点は 6 種であり、`Retain` と `Release` は骨格に無い (第 1 節)。残る 4 種のうち
    `Let`(非 `Match`)、`Destructure`、`Eval` は (T1)、`Let`(`Match`) は (T2) である。`Ret` は木の中の
    位置で 2 つに分かれる -- アーム本体の終端は (T3) であり (D3 より、アーム本体の実行路を辿り終えると
    `Match` の継続へ進む)、関数本体・初期化子の終端は `ρ` の最後の節点なので (D3) 後続の検査点を持たず、
    遷移を成さない。

<1>2. **CASE (T1)** で `m = Let(x, rhs, cont)`、`rhs` は `Match` でない。
  <2>1. `Λ(m) = (Λ(m') \ {x}) ∪ ops` である。ここで `ops` は `rhs_operands(rhs)` が挙げるオペランドの
        局所名の集合であり、`x ∉ ops` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let, A6
  <2>2. `ops` の各名前 `v` について、`n_v` を `v` の `Own` の出現回数とすると、前置 `Retain` 鎖が
        `v` を名指す回数は `n_v - [v ∉ Λ(m') かつ v の最後の出現が Own]` であり、後置 `Release` 鎖が
        `v` を名指す回数は `[v ∉ Λ(m') かつ v の最後の出現が Borrow]` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: build_retains,
       CODE src/rc_ir/rc_insert.rs: build_releases, L15
    ループは `operands.iter().rev()` を走り、`live_after_operand` を `live_cont`(= `Λ(m')`) の写しから
    始めて各局所オペランドの名前を足していく。よって `v` の出現 `i` における `used_later` は
    「`v ∈ Λ(m')` または `v` の出現が `i` より後ろにある」である。`Own` の出現は `used_later` が真で
    `needs_rc(v)` のとき `retains_before` に入り、`Borrow` の出現は `used_later` が偽で `needs_rc(v)`
    のとき `releases_after` に入る。`used_later` が偽になるのは `v ∉ Λ(m')` のときの最後の出現だけで
    ある。
  <2>3. 後置 `Release` 鎖は、さらに `x ∉ Λ(m')` のとき `x` を名指す `Release` を 1 つ置く。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let, L15
    `after` は `releases_after` に、`!live_cont.contains(&x.name) && self.needs_rc(&x)` のとき `x` を
    足したものである。
  <2>4. 核節点は、`ops` の各 `Own` の出現ごとに、そのオペランドの inhabited な各 boxed leaf について
        `μ` を 1 下げる。`Borrow` の出現は `μ` を変えない。
    BY D9, L15, L16
    `rhs = Var(y)`: D9 の移動の表の `Let(x, Var(y), k)` の行が `y` の全 leaf を移す。
    `rhs = App(callee, args)`: D9 の消費の表の `App` の行が callee の全 boxed leaf と、呼び出し先が
    所有する位置の引数の leaf を消費する。`L15` (e) よりすべての位置が所有される。
    `rhs = Closure(f, caps)`: 消費の表の `Closure` の行が全 capture の全 leaf を消費する。
    `rhs = Llvm(gen, args)`: `borrows_operand(i)` が偽のオペランドの各 leaf は、素通しを宣言されて
    いれば移動の表の `Llvm` の行で結果へ移り、されていなければ消費の表の `Llvm` の行で消費される。
    どちらでも `μ` は 1 下がる。`borrows_operand(i)` が真のオペランドは、消費の表の行に入らず、`L16`
    より移動の表の行にも入らない。
  <2>5. 核節点は、結果の inhabited な各 boxed leaf `(x, μ')` について `μ` を 1 上げる。
    BY D9, D10, A3, A5
    `rhs = Var(y)`: 移動の表の行が `x` の各 leaf に移す。
    `rhs = App` / `Closure`: D10 の生成の表の対応する行が結果の各 leaf に参照を作る。
    `rhs = Llvm`: 素通しの leaf は移動の表の行で結果へ入り、素通しでない leaf は D10 の生成の表の
    `Llvm` の行で参照を得る。A3 と A5 より、素通しを宣言する結果の leaf とそれが名指すオペランドの
    leaf は 1 対 1 に対応する -- 2 つの結果 leaf が同じ `Arg(i, σ)` を宣言してどちらも inhabited で
    あると、A3 の「同じ参照」と A5 の「inhabited な各 leaf にちょうど 1 つ」が両立しない。よって
    結果の各 inhabited な leaf は、素通しか生成かのちょうど一方で `μ` を 1 得る。
  <2>6. 遷移の後、`ops` の各名前 `v` について `μ(v, λ) = [v ∈ Λ(m')]`、`x` について
        `μ(x, μ') = [x ∈ Λ(m')]` であり、他の名前の `μ` は変わらない。
    BY <2>2, <2>3, <2>4, <2>5
    仮定より遷移の前は `μ(v, λ) = 1` (`v ∈ ops ⊆ Λ(m)`) であり `μ(x, μ') = 0` (`x` はまだ値を得て
    いない)。`v` について、前置 `Retain` が `n_v - [v ∉ Λ(m') ∧ 最後が Own]` 回上げ、核節点が `n_v`
    回下げ、後置 `Release` が `[v ∉ Λ(m') ∧ 最後が Borrow]` 回下げるので、
    `1 - [v ∉ Λ(m') ∧ 最後が Own] - [v ∉ Λ(m') ∧ 最後が Borrow] = 1 - [v ∉ Λ(m')] = [v ∈ Λ(m')]`。
    `x` について、核節点が 1 上げ、後置 `Release` が `[x ∉ Λ(m')]` 回下げるので `[x ∈ Λ(m')]`。
    <2>1 より `Λ(m)` と `Λ(m')` の差は `ops` と `{x}` の上にしか無いので、他の名前は両方に入るか
    両方に入らないかであり、`μ` も変わらない。
  <2>7. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, A6
    前置 `Retain` 鎖の中では `μ` は上がるだけなので、`μ(v, λ) ≥ 1` (`v ∈ ops`) が保たれ、鎖の各
    `Retain` が触れるスロットは `μ ≥ 1` である。核節点の入口では `μ(v, λ) = 1 + (前置の回数) ≥ n_v`
    であり、核節点が読む値 (D7 の表: `Llvm` の各オペランド、`App` の callee と各引数、`Closure` の
    各 capture、`Destructure` の容器、`Eval` の変数) はいずれも `ops` の名前か局所でない名前
    (`L15` (d)) なので `μ ≥ 1` である。核節点の後、後置 `Release` 鎖の各名前は相異なる -- `releases_after`
    に同じ名前が 2 度入ることはなく (`used_later` が偽になるのは最後の出現だけ)、`x` はオペランドとは
    別の名前である (A6) -- ので、鎖の `i` 番目の `Release` の入口での `μ` は、その名前について
    <2>6 の最終値 `[・ ∈ Λ(m')]` に、まだ実行していない自分自身の分 1 を足したもの以上であり、1 以上で
    ある。それらの `Release` の後の値は <2>6 の最終値であり非負である。
  <2>8. QED
    BY <2>6, <2>7

<1>3. **CASE (T1)** で `m = Destructure(container, fields, _, cont)`。
  <2>1. `Λ(m) = (Λ(m') \ {フィールド変数}) ∪ {container}` である。前置 `Retain` 鎖は
        `container ∈ Λ(m')` のとき `Retain(container, [])` を 1 つ置き、後置 `Release` 鎖は `Λ(m')` に
        入らない各フィールド変数の `Release` を 1 つずつ置く。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live, L15
    `retain_if_live(&container, &live_cont, node)` は `live_cont` (= `Λ(m')`) を見る。`dead` は
    `!live_cont.contains(&fv.name) && self.needs_rc(fv)` を満たすフィールド変数である。
  <2>2. CASE `container` が boxed。
    BY <2>1, D9, D10
    D9 の消費の表の「`Destructure`、`c` が boxed」の行より容器の全 boxed leaf が消費され (`μ` が各
    leaf で 1 下がる)、D10 の生成の表の「boxed 容器の `Destructure` の各名前付きフィールドの各 leaf」の
    行より各フィールドの leaf の `μ` が 1 上がる。よって遷移の後
    `μ(container, λ) = 1 + [container ∈ Λ(m')] - 1 = [container ∈ Λ(m')]`、
    `μ(fv, λ') = 1 - [fv ∉ Λ(m')] = [fv ∈ Λ(m')]`。
  <2>3. CASE `container` が unbox。
    BY <2>1, D9, CODE src/rc_ir/ownership.rs: destructure_consumes
    D9 の消費の表の「`Destructure`、`c` が unbox」の行より名前の付いていないフィールドの leaf が
    消費され、移動の表の「unbox 容器の `Destructure` の名前付きフィールド」の行より名前の付いた
    フィールドの leaf は容器からフィールド変数へ移る。どちらでも容器の各 leaf の `μ` は 1 下がり、
    名前の付いたフィールドの leaf では対応するフィールド変数の `μ` が 1 上がる。よって <2>2 と同じ
    最終値になる。
  <2>4. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <2>1, <2>2, <2>3, A6
    前置 `Retain` は `μ(container, ・)` を上げるだけである。核節点が読む値は容器であり、その入口で
    `μ(container, λ) = 1 + [container ∈ Λ(m')] ≥ 1`。後置 `Release` 鎖の名前は相異なるフィールド変数
    (A6) であり、各 `Release` の入口でその変数の `μ` は 1 である。
  <2>5. QED
    BY <2>2, <2>3, <2>4, D5
    容器は boxed か unbox かのいずれかである。

<1>4. **CASE (T1)** で `m = Eval(x, cont)`。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval, D9, D7, L15
  `Λ(m) = Λ(m') ∪ ({x} ∩ 局所名)` である。前置 `Retain` 鎖は空であり、後置 `Release` 鎖は
  `x` が局所名で `x ∉ Λ(m')` かつ `needs_rc(x)` のとき `Release(x, [])` を 1 つ置く。D9 の 2 つの表に `Eval` の行は無いので核節点は
  `μ` を変えない。よって遷移の後 `μ(x, λ) = 1 - [x ∉ Λ(m')] = [x ∈ Λ(m')]` であり、他の名前は変わら
  ない。核節点は `x` を読み (D7)、その入口で `μ(x, λ) = 1` である。`Release` の入口でも 1 である。

<1>5. **CASE (T2)**。`m = Let(x, Match(scrut, arms), cont)`、`ρ` が選ぶアームを `j` とする。
  <2>1. `H := live_at_arm_head`、`M := live_after_match`、`U_j := arm_free_locals(arm_j)`、
        `P_j := {payload_j} ∩ free_locals(arm_j.body)` と置くと、`Λ(m) = H ∪ {scrut}`、
        `Λ(m') = U_j ∪ P_j ∪ M` であり、`H = M ∪ (∪_i U_i)` である。
    BY L14, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals
    `live_at_arm_head` は `live_after_match` に各アームの `arm_free_locals` を足したものである。
    `L14` より `Λ(m) = free_locals(m) ∪ A(m)` であり、`free_locals(m)` は
    `{scrut} ∪ (∪_i U_i) ∪ (free_locals(cont) \ {x})`、`M = live_cont \ {x}` は
    `(free_locals(cont) ∪ A(m)) \ {x}` なので `Λ(m) = {scrut} ∪ (∪_i U_i) ∪ M = {scrut} ∪ H`。
    アーム本体は `insert_into_expr(arm.body, &live_after_match)` で書き換えられるので `L14` より
    `Λ(m') = free_locals(arm_j.body) ∪ M = U_j ∪ P_j ∪ M`。
  <2>2. 前置 `Retain` 鎖は `scrut ∈ H` のとき `Retain(scrut, [])` を 1 つ置く。アーム `j` の頭の
        `Release` 鎖は、`DB_j := H \ (U_j ∪ M)` の各名前の `Release`、`scrut.ty.is_box` かつ
        `arm_j.tag` が `Some` のときの `Release(scrut, [])`、`payload_j ∉ Λ(m')` のときの
        `Release(payload_j, [])` をこの順に置く。コードはどれも `needs_rc` で絞るが、`needs_rc` が
        偽の名前はスロットを持たない (`L15` (c)) ので、以下の勘定では区別しない。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: build_releases, <2>1, L15
    `retain_if_live(&scrut, &live_at_arm_head, node)` は `H` を見る。`head` は dead-branch
    (`!used.contains(n) && !live_after_match.contains(n)` を満たす `live_at_arm_head` の名前)、
    `release_container && arm.tag.is_some() && needs_rc(&scrut)`、
    `!body_live.contains(&payload.name) && needs_rc(&payload)` の順に積まれる。`body_live` は
    `Λ(m')` であり、`release_container` は `scrut.ty.is_box(self.type_env)` である。
  <2>3. `DB_j` の各名前 `n` (`scrut` を含みうる) について `n ∉ Λ(m')` であり、`Λ(m) \ Λ(m')` は
        `DB_j ∪ ({scrut} \ H)`、`Λ(m') \ Λ(m)` は `P_j` である。
    BY <2>1, A6
    `DB_j = H \ (U_j ∪ M)` であり `Λ(m') = U_j ∪ P_j ∪ M`。`payload_j` は A6 より `H` に入らないので
    `DB_j ∩ Λ(m') = ∅`。`Λ(m) \ Λ(m') = (H ∪ {scrut}) \ (U_j ∪ P_j ∪ M)` であり、`payload_j` は
    `H ∪ {scrut}` に入らないので `= (H \ (U_j ∪ M)) ∪ ({scrut} \ (U_j ∪ M))`。`scrut ∈ H` のとき
    第 2 項は第 1 項に含まれ、`scrut ∉ H` のとき `scrut ∉ M ∪ (∪_i U_i)` なので第 2 項は `{scrut}` で
    ある。`Λ(m') \ Λ(m) = (U_j ∪ P_j ∪ M) \ (H ∪ {scrut}) = P_j` (`U_j ∪ M ⊆ H`)。
  <2>4. CASE `scrut.ty.is_box` かつ `arm_j.tag = Some(t)`。
    BY <2>2, <2>3, D9, D10
    D10 の生成の表の「boxed union の変位アームの payload の各 leaf」の行が `μ(payload_j, λ')` を
    1 上げる。scrutinee の側は D9 の 2 つの表のどの行にも当たらない -- 参照を処分するのは
    `insert_into_match` が置いた `Release(scrut, [])` である。よって遷移の後
    `μ(scrut, λ) = 1 + [scrut ∈ H] - [scrut ∈ DB_j] - 1`。`scrut ∈ H` のとき
    `[scrut ∈ DB_j] = [scrut ∉ U_j ∪ M]` なのでこれは `[scrut ∈ U_j ∪ M] = [scrut ∈ Λ(m')]`
    (`scrut ∉ P_j`)。`scrut ∉ H` のとき `scrut ∉ U_j ∪ M` すなわち `scrut ∉ Λ(m')` であり、
    `[scrut ∈ DB_j] = 0` なので値は 0 である。どちらでも `[scrut ∈ Λ(m')]` に等しい。
    `μ(payload_j, λ') = 1 - [payload_j ∉ Λ(m')] = [payload_j ∈ Λ(m')]`。
    `DB_j` の他の名前 `n` は `μ(n, ・) = 1 - 1 = 0 = [n ∈ Λ(m')]` (<2>3)。
  <2>5. CASE `arm_j.tag = None`、または (`arm_j.tag = Some(t)` かつ `scrut.ty.is_box` が偽)。
    BY <2>2, <2>3, D9, D16, D21, A16, A12
    どちらの場合も `release_container && arm.tag.is_some()` が偽なので容器の `Release` は置かれない。
    catch-all アームでは、D9 の移動の表の「catch-all アームの payload 束縛」の行より scrutinee の参照が
    payload へ移り、A12 より payload と scrutinee の型は等しいので、scrutinee の inhabited な各 leaf が
    payload の同じ path の leaf へ移る。unbox union の変位アームでは、D9 の移動の表の「unbox union の
    変位アームの payload 束縛」の行より活性変位の参照が payload へ移る。D21 と A16 より選ばれたアームの
    `tag` は scrutinee の実行時のタグに等しく、D16 より scrutinee の inhabited な leaf はその変位の下に
    あるものだけなので、どちらの場合も移るのは scrutinee のすべての inhabited な leaf である。
    よって遷移の後 `μ(scrut, λ) = 1 + [scrut ∈ H] - 1 - [scrut ∈ DB_j]` であり、<2>4 と同じ計算で
    `[scrut ∈ Λ(m')]` に等しい。`μ(payload_j, λ'') = 1 - [payload_j ∉ Λ(m')] = [payload_j ∈ Λ(m')]`。
    `DB_j` の他の名前は <2>4 と同じである。
  <2>6. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <2>2, <2>4, <2>5, A6, D7
    前置 `Retain` が置かれるのは `scrut ∈ H ⊆ Λ(m)` のときであり、そのときその入口で
    `μ(scrut, λ) = 1` である。核節点は scrutinee を読み (D7)、その入口で
    `μ(scrut, λ) = 1 + [scrut ∈ H] ≥ 1`。頭の `Release` 鎖の名前のうち `scrut` は最大 2 回
    現れる -- `DB_j` に入るときと容器解放のときである。両方が起きるのは
    `scrut ∈ DB_j` すなわち `scrut ∈ H` のときであり、そのとき核節点の直後の
    `μ(scrut, λ)` は 2 なので、2 つの `Release` の入口の値は 2 と 1 である。片方だけのときは
    入口の値は 1 以上である。`payload_j` と `DB_j` の他の名前は互いに、また `scrut` とも異なり (A6)、
    それぞれ 1 度しか現れないので、その入口の値は 1 である。<2>4 と <2>5 の最終値はすべて非負である。
  <2>7. QED
    BY <2>4, <2>5, <2>6
    `arm_j.tag` は `Some` か `None` のいずれかであり、`Some` の場合は `scrut.ty.is_box` の真偽で
    分かれる。3 つの場合を <2>4 と <2>5 が覆う。

<1>6. **CASE (T3)**。`m` はアーム本体の終端の `Ret(r)`、その `Match` を `Let(x, Match(s, arms), cont)`
      とする。
  <2>1. `Λ(m) = M ∪ ({r} ∩ 局所名)`、`Λ(m') = live_cont` であり `M = live_cont \ {x}` である。
    BY L14, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    アーム本体は `live_after = M` で書き換えられるので `L14` より `Λ(m) = free_locals(Ret(r)) ∪ M`。
  <2>2. 前置 `Retain` 鎖は `r ∈ M` のとき `Retain(r, [])` を 1 つ置き、`Match` の核節点と `cont` の
        間の `Release` 鎖は `x ∉ live_cont` のとき `Release(x, [])` を 1 つ置く。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, L15
  <2>3. 核節点 `Ret(r)` は `r` の inhabited な各 boxed leaf について `μ(r, λ)` を 1 下げ、
        `μ(x, λ)` を 1 上げる。
    BY D9, A12
    D9 の移動の表の「`Match` のアーム本体の `Ret(x)`」の行。A12 よりアームの結果と `Match` の束縛変数の
    型は等しいので leaf は対応する。
  <2>4. QED
    BY <2>1, <2>2, <2>3, A6
    A6 より `x ≠ r` であり、`r ∈ M ⟺ r ∈ live_cont` である。遷移の後
    `μ(r, λ) = 1 + [r ∈ M] - 1 = [r ∈ Λ(m')]`、
    `μ(x, λ) = 1 - [x ∉ live_cont] = [x ∈ Λ(m')]`。他の名前は `Λ(m)` と `Λ(m')` の両方に入るか
    両方に入らないかである。遷移の中の各節点については、前置 `Retain` の入口で `μ(r, λ) = 1`、
    核節点は D7 の読む構文ではないが `r` を名指し `μ(r, λ) = 1 + [r ∈ M] ≥ 1`、`Release(x, [])` の
    入口で `μ(x, λ) = 1` である。値はどこでも非負である。

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### 10.6 `L18` (検査点では `held` は live なスロットの個数に等しい)

**言明**。`insert_rc` の出力の 1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化について、`ρ` の
上の各検査点 `m` において、各スロット `(v, λ)` の `μ` は `[v ∈ Λ(m)]` に等しい。したがって各計数下の
別名類 `C` について `held_ρ(m, C) = N_ρ(m, C)` である。また本体の根の検査点より前にある `Release` 節点
(`insert_into_func` の `unused` の鎖) の各入口で、すべてのスロットの `μ` は非負であり、その `Release`
が触れるスロットの `μ` は 1 である。

**証明**

<1>1. 最初の検査点 -- 本体の根の骨格節点の検査点 -- で言明が成り立つ。
  <2>1. CASE 本体がグローバル初期化子の `init` である。
    BY CODE src/rc_ir/rc_insert.rs: insert_rc, D10, D1
    D1 より `init` はパラメータも capture も持たないので D10 の初期値は空であり、活性化の開始時の
    `μ` はすべて 0 である。`insert_rc` はこの本体について `live.is_empty()` を表明するので
    `Λ(根) = ∅` であり、根の前に節点は無い。
  <2>2. CASE 本体が関数の `body` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: build_releases, D10, L15
    `L15` (e) より、すべてのパラメータ・capture の unit は所有される。D10 の初期値は、所有する
    各パラメータ・capture の inhabited な各 leaf の `μ` を 1 にする。`insert_into_func` は
    `func.body = build_releases(unused, body)` を作り、`unused` は
    `self.needs_rc(p) && !live.contains(&p.name)` を満たすパラメータと capture である。ここで
    `live = Λ(根)` である。`L15` (c) より `needs_rc(p)` が偽のパラメータはスロットを持たない。よって `unused` の解放鎖の後、パラメータ・capture の各スロットの `μ` は
    `[p ∈ Λ(根)]` である。`unused` の名前は相異なるので各 `Release` の入口で `μ = 1` であり、
    `insert_into_func` の表明より `Λ(根)` はパラメータと capture の名前だけからなるので、他の名前の
    スロットは値を得ておらず `μ = 0 = [・ ∈ Λ(根)]` である。
  <2>3. QED
    BY <2>1, <2>2, D23, A6
    D23 より本体は関数の `body` かグローバル初期化子の `init` かのいずれかである。A6 より
    `unused` の名前は相異なる。

<1>2. 各検査点で `μ` は `Λ` の指示関数である。
  BY <1>1, L17
  `ρ` の上の検査点の列についての帰納。基底は <1>1、段は `L17` である。

<1>3. QED
  BY <1>2 DEF 割り当て
  `held_ρ(m, C) = Σ_{(v, λ) ∈ C} μ_m(v, λ) = Σ_{(v, λ) ∈ C, v ∈ Λ(m)} 1 = Σ_{v ∈ Λ(m)} κ_C(v)
  = N_ρ(m, C)`。

### 10.7 `L19` ((O1))

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、次の 2 つが成り立つ。

- **(a)** `ρ` の上の各時点において、各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その節点の入口で `held_ρ(・, C) ≥ 1` である。
- **(c)** 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った後、各計数下の別名類 `C` について
  `held_ρ(・, C) = 0` である。

**証明**

<1>1. (a)。
  BY L18, L17 DEF 割り当て
  `L17` (a) より各節点の入口は 3 通りである。`unused` の解放鎖の中では `L18` が `μ ≥ 0` を与え、
  遷移の中では `L17` (b) が与え、終端の `Ret` の検査点では `L18` が `μ` を `Λ` の指示関数と定める
  ので `μ ≥ 0` である。`held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ)` は非負の項の和である。

<1>2. (b)。
  BY L18, L17 DEF 割り当て
  `L17` (b) より、遷移の中の各節点の入口で、その節点が読む値のスロットと `Retain`/`Release` が触れる
  スロットは `μ ≥ 1` である。検査点はその遷移の最初の節点の入口なので同じ主張に含まれる。
  `unused` の解放鎖の中の `Release` については `L18` が `μ = 1` を与える。終端の `Ret` の検査点に
  ついては、`L18` よりその節点が名指す `x` のスロットの `μ` は `[x ∈ Λ]` であり、`L15` (b) より
  `x ∈ Λ` なので 1 である (終端の `Ret` は D7 の読む構文ではないので、この場合は言明の対象外である)。
  そのスロット `(v, λ)` が計数下の類 `C` に属するとき、
  `held_ρ(・, C) = Σ_{(w, μ') ∈ C} μ(w, μ') ≥ μ(v, λ) ≥ 1`。

<1>3. (c)。
  <2>1. 関数本体・初期化子の終端の `Ret(x)` を書き換える呼び出しの `live_after` は空集合である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, D2
    根を書き換える呼び出しの `live_after` は空集合である (`insert_into_func` は
    `insert_into_expr(func.body, &Set::default())`、`insert_rc` は
    `inserter.insert_into_expr(glob.init, &Set::default())` を呼ぶ)。継続を書き換える 4 つの関数は
    いずれも `self.insert_into_expr(cont, live_after)` を呼び、`live_after` をそのまま渡す。D2 より
    `Ret` 以外の 5 種はちょうど 1 つの継続を持つので、関数本体・初期化子の終端の `Ret` は根から継続
    だけを辿って着く節点である。継続の鎖の長さについての帰納で `live_after` は空集合である。
  <2>2. その検査点の前置 `Retain` 鎖は空である。
    BY <2>1, CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live
    `retain_if_live` の条件 `live.contains(&var.name)` は空集合について偽である。
  <2>3. QED
    BY <2>1, <2>2, L18, L14, D9, DEF 割り当て
    <2>1 と `L14` より `Λ(終端の Ret) = free_locals(Ret(x))` であり、これは `x` が局所名なら `{x}`、
    そうでなければ空である。`L18` よりその検査点で `μ` は `Λ` の指示関数なので、`x` 以外のスロットの
    `μ` は 0 である。D9 の消費の表の「関数本体の終端の `Ret(x)`」の行より、核節点は `x` の inhabited な
    全 boxed leaf の参照を消費するので、その後 `μ(x, λ) = 0` である。よってすべてのスロットで
    `μ = 0` であり `held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ) = 0`。

<1>4. QED
  BY <1>1, <1>2, <1>3

**(c) が言っているもの。** (c) は D11 の (S-b) を別名類の粒度へ絞ったものである。A19 (ii-a) を活性化の
終わりの 1 点先まで読む読み手は、この点で `held ≥ 0` を要る -- (c) はそれを等式の形で与える。
**A19 (ii-b) はこの点へは延びない。** `held = 0` である一方、走査の `RcExpr::Ret` の腕は
`returns_from_func` が真のとき pending の要素を `needed_retains` に入れるだけで `pending` から
取り除かないので (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)、`bumps ≥ 1` のまま
`held = 0` になる時点が在る。第 12 節がその形の `insert_rc` の出力を挙げる。

### 10.8 `L20` (各別名類の生成事象は `ρ` の上に高々 1 つ)

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各計数下の別名類 `C` に
`held_ρ(・, C)` の開始値 1 を与える事象は `ρ` の上に高々 1 つである。

**証明**

<1>1. 開始値を与える事象は、パラメータ・capture の leaf についての D10 の初期値と、D10 の生成の表の
      各行である。
  BY D10
  D10 は義務集合の初期値と生成の表を持ち、それ以外に参照を作る行を持たない。

<1>2. これらの事象が作る参照が属する類の ρ-終端は、その事象の位置のスロット自身である。
  BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings,
     p13 の DEF ρ-歩みと ρ-終端, A3, D17
  パラメータ・capture は `Binding::Param` で `here()`、`App` と `Closure` の結果は
  `Binding::Producer` で `here()`、boxed 容器の `Destructure` のフィールドは `Binding::Field` の
  boxed の枝で `here()`、boxed union の変位アームの payload は `Binding::Payload` の `Some(_)` かつ
  boxed の枝で `here()` を返す。`Llvm` の結果の素通しでない leaf は `Binding::Llvm` の腕で
  `as_arg_projection` が `None` を返す枝に入り、A3 よりこのコミットの宣言は単一の `Fresh`・単一の
  `Unknown`・空集合のいずれかなので、D17 の第 2 項よりそこで止まる。いずれも `origin` を呼ばずに
  自分自身を答えるので、ρ-歩みはその位置で終わる。

<1>3. QED
  BY <1>1, <1>2, D6, A6, A11
  <1>2 より、相異なる生成事象は相異なるスロットを ρ-終端とする -- 事象の位置が相異なれば、そこで値を
  得る変数が相異なる (A6) か、同じ変数の相異なる leaf である。よって相異なる生成事象は相異なる類に
  開始値を与える。A11 と D6 より 1 つのスロットが 2 度値を得ることはない。

**`L4` の前提が満たされること。** `L4` は「`ρ` の上で生成事象を 1 つだけ持つ各別名類」について述べる。
`L20` より生成事象は高々 1 つである。生成事象を 1 つも持たない類は `ρ` の上にスロットを持たない
(D6 -- スロットの変数はその実行路の上で値を得ている) ので、勘定の対象にならない。

## 11. (O2) の証明 -- 帳簿は処分に遅れない

`L7` より、(O2) は「`bumps ≥ 1` である各時点で `held ≥ 1 + bumps`」と同値である。この節はそれを示す。
支えるのは、**別名類を名前で切った区画の木**についての不等式である。1 つの別名類の中で、`origin` の
`identity` は `Binding::Join` が候補を 2 つ以上持つ位置で切り替わる。切り替わりで区切られた各部分を
区画と呼ぶと、区画は木をなし、走査の帳簿は各区画の名前ごとに付く。示す不等式は
**「1 つの区画とその下の区画に付いている bump の総和は、その区画とその下が保持している参照の個数より
真に小さい」**であり、木の根で読むとそれがちょうど `held ≥ 1 + bumps` になる。

この節は第 10 節の `L15`・`L17`・`L18`・`L20` と、`DEF 割り当て` の `μ` を使う。

### 11.1 名前の鎖、区画の木、帳簿

活性化と実行路 `ρ` と計数下の別名類 `C` を固定する。以下、`C` のスロットを単に**スロット**と書く。

**DEF `id(s)`**。スロット `s = (v, λ)` について `id(s) := origin(v, λ).identity()` と置く
(`CODE src/rc_ir/ownership.rs: origin`, `CODE src/rc_ir/ownership.rs: Origin`)。

**DEF `Anc(s)`**。`p13-disposals-and-pending.md` の `DEF ρ-歩みと ρ-終端` の ρ-歩みは、`s` から
ρ-終端まで辿るスロットの列 `s = s_0, s_1, …, s_n` である。`Anc(s) := {id(s_0), id(s_1), …, id(s_n)}`
と置き、**`s` の名前の鎖**と呼ぶ。

**DEF `Ids(C)`、`Sub(id)`**。`Ids(C) := {id(s) : s はスロット}`。`id ∈ Ids(C)` について
`Sub(id) := {id(s) : s はスロットで id ∈ Anc(s)}`。

**DEF `Down`、`Bmp`、`Bsub`**。`ρ` の上の時点 `τ` と `id ∈ Ids(C)` について
`Down_τ(id) := Σ_{s : id ∈ Anc(s)} μ_τ(s)`、
`Bmp_τ(id) := Σ_{p ∈ pending} B(p, ρ)[id]` (D27)、
`Bsub_τ(id) := Σ_{id' ∈ Sub(id)} Bmp_τ(id')` と置く。

**前提 (N)**。`L6` と同じ前提を置く -- **名前は類を決める**。すなわち `origin(w, μ).identity()` が
`Ids(C)` の元であるスロット `(w, μ)` は `C` のスロットである。この文書は (N) を証明しない
(6.2 節)。

### 11.2 `L21` (名前の鎖の形)

**言明**。各スロット `s` について次の 5 つが成り立つ。

- **(a)** `Anc(s) ⊆ acted_on(s)`。ここで `acted_on(s) := origin(v, λ).acted_on()` の元の集合である
  (D15)。
- **(b)** `Anc(s)` は `id(s)` だけで決まる。以下 `Anc(id(s))` とも書く。
- **(c)** `id(s) ∈ Anc(s)` であり、ρ-終端 `t` の名前 `id_0 := id(t)` は `Anc(s)` に入る。
- **(d)** `id ∈ Anc(s)` ならば `Anc(id) ⊆ Anc(s)` である。したがって `Anc(id)` の元は
  「`Anc` に含まれる」の関係で線形順序をなし、`Ids(C)` は `id_0` を根とする木をなす。
- **(e)** D9 の移動の表の辺の移動元 `s` と移動先 `s'` について、`Anc(s') = Anc(s)` (したがって
  `id(s') = id(s)`) であるか、`Anc(s') = {id(s')} ∪ Anc(s)` かつ `id(s') ∉ Anc(s)` であるかの
  いずれかである。後者になるのはアーム本体の `Ret` の辺で、`Match` の束縛変数の `origin` が候補を
  2 つ以上持つときに限る。

**証明**

<1>1. ρ-歩みの 1 歩で `origin` の値がどう変わるかは、`origin_inner` の 6 つの腕で尽きる。
  BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding, D17
  `Binding` は 7 個の構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `origin_inner` の `match` は、束縛の無い場合 (`None`) と `Param` と `Producer` を 1 つの腕に
  まとめた 6 つの腕で、この 8 通りを覆う。

<1>2. 腕は 2 群に分かれる。**止まる腕**では `origin` は `here()` を返し、ρ-歩みはそこで終わる。
      **辿る腕**では `origin` は次のスロットの `origin` を返し、`identity` はそのまま受け継がれる。
      **`Join` の腕**だけが第 3 の形であり、候補が 2 つ以上のとき `identity` を `here()` に取り替えた
      うえで、その活性化が選んだアームの結果へ辿る。
  BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Origin::of_candidates,
     CODE src/rc_ir/ownership.rs: as_arg_projection, D17, A3
  止まる腕: `None`、`Binding::Param`、`Binding::Producer`、`Binding::Field` の容器が boxed の枝、
  `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝、`Binding::Llvm` の
  `as_arg_projection` が `None` を返す枝 (スロットの path は boxed leaf なので `leaf_origins_at` は
  その leaf 自身の宣言を返し、A3 よりこのコミットの宣言は単一の `Fresh`・単一の `Unknown`・空集合の
  いずれかである)。
  辿る腕: `Binding::Move`、`Binding::Field` の容器が unbox の枝、`Binding::Payload` の `None` の枝と
  `Some(tag)` かつ scrutinee が unbox の枝、`Binding::Llvm` の単一 `Arg` の枝。いずれも
  `origin(次のスロット)` をそのまま返す。
  `Binding::Join` の腕は `of_candidates(candidates, here)` を返す。候補が 1 つのときそれは
  `Exactly(その候補)` であり、その候補は選ばれたアームの結果の `origin` の値そのものなので辿る腕と同じ
  形になる。候補が 2 つ以上のとき `Join { identity: here, candidates }` であり、`identity` は
  そのスロット自身である。D17 より ρ-歩みは選ばれたアームの結果へ進む。

<1>3. (c)。
  BY <1>2
  ρ-歩みの各歩で `identity` は変わらないか (辿る腕、候補 1 つの `Join`)、そのスロット自身に
  取り替わるか (候補 2 つ以上の `Join`)、そこで止まる (止まる腕) かのいずれかである。よって `id(s)` は
  歩みの上のどれかのスロットの位置であり、`Anc(s)` の元である。ρ-終端 `t` は歩みの最後の元なので
  `id_0 = id(t) ∈ Anc(s)`。

<1>4. (a)。
  BY <1>2, D15, CODE src/rc_ir/ownership.rs: origin_inner
  歩みの長さについての帰納。長さ 0 (止まる腕) では `Anc(s) = {id(s)}` であり
  `acted_on(s) ∋ identity()` なので成り立つ。辿る腕と候補 1 つの `Join` では `origin(s)` が次の
  スロットの `origin` に等しいので `Anc(s) = Anc(次)` かつ `acted_on(s) = acted_on(次)` であり、
  帰納法の仮定がそのまま渡る。候補 2 つ以上の `Join` では `Anc(s) = {id(s)} ∪ Anc(次)` であり、
  `acted_on(s) = {id(s)} ∪ candidates` で `candidates` は各アームの結果の `acted_on()` の和なので
  (`origin_inner` の `Binding::Join` の腕)、選ばれたアームの結果について
  `Anc(次) ⊆ acted_on(次) ⊆ candidates` が帰納法の仮定から出る。

<1>5. (b)。
  BY <1>2, <1>3
  <1>3 の議論より、`id(s)` は歩みの上のスロット `s_k` の位置であり、`s_k` は「`origin(s_k)` の
  `identity` が `s_k` 自身である」最初のスロットである。歩みの `s_0` から `s_k` までの区間では
  `identity` は変わらないので `Anc(s)` のうち `id(s)` より前の元は無く、`Anc(s) = {id(s)} ∪ Anc(s_{k+1})`
  (`s_k` が候補 2 つ以上の `Join` のとき) または `Anc(s) = {id(s)}` (`s_k` が止まる腕のとき) である。
  どちらも `s_k`、すなわち `id(s)` だけで決まる。

<1>6. (d)。
  BY <1>3, <1>5
  `id ∈ Anc(s)` のとき、`Anc(s)` の定義より `id = id(s_j)` であるスロット `s_j` が `s` の歩みの上に
  ある。`s` の歩みの `s_j` 以降は `s_j` の歩みそのものなので `Anc(s_j) ⊆ Anc(s)` であり、(b) より
  `Anc(s_j) = Anc(id)` である。
  よって `Anc(s)` の元は `s` の歩みに現れる順に並び、その順で後ろの元 `id'` ほど `Anc(id')` が小さい
  -- すなわち `Anc(s)` は「`Anc` の包含」で線形順序をなす。すべての `Anc` は `id_0` を含む (<1>3) ので、
  `Ids(C)` の上の関係「`id' ∈ Anc(id)`」は `id_0` を最大元とする半順序であり、各 `id` の上側は鎖で
  ある。すなわち `Ids(C)` は `id_0` を根とする木をなす。

<1>7. (e)。
  BY <1>2, <1>5, D9, D20, A6, CODE src/rc_ir/ownership.rs: collect_bindings
  D9 の移動の表は 6 行を持つ。`Let(x, Var(y), k)` の移動先は `Binding::Move`、unbox 容器の
  `Destructure` の名前付きフィールドは `Binding::Field` の unbox の枝、unbox union の変位アームの
  payload 束縛は `Binding::Payload` の `Some(tag)` かつ scrutinee が unbox の枝、catch-all アームの
  payload 束縛は `Binding::Payload` の `None` の枝、`Llvm` の素通し leaf は `Binding::Llvm` の
  単一 `Arg` の枝に束縛される (`collect_bindings`)。この 5 つは <1>2 の辿る腕であり、`origin(s')` は
  `origin(s)` に等しいので `Anc(s') = Anc(s)` である。
  残る 1 行 -- `Match` のアーム本体の `Ret(x)` -- の移動先は `Binding::Join` である。候補が 1 つの
  ときは <1>2 より辿る腕と同じ形で `Anc(s') = Anc(s)`、候補が 2 つ以上のときは `id(s')` が `s'` 自身
  であり、<1>5 の展開より `Anc(s') = {id(s')} ∪ Anc(s)` である。`Anc(s)` の元は `s` の歩みの上の
  スロットの位置であり、それらは `s'` の束縛変数より前に値を得ている変数の leaf なので、A6 より
  `id(s') ∉ Anc(s)` である。

<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

**`Sub` と `Down` の言い換え。** (b) より `id ∈ Anc(s)` と `id(s) ∈ Sub(id)` は同値である
(`id(s) ∈ Sub(id)` はあるスロット `s''` について `id(s'') = id(s)` かつ `id ∈ Anc(s'')` であること
であり、(b) より `Anc(s'') = Anc(s)`)。よって `Down_τ(id) = Σ_{s : id(s) ∈ Sub(id)} μ_τ(s)` である。

### 11.3 `L22` (木の根で読むと `held` と `bumps` になる)

**言明**。(N) の下で、`ρ` の上の各時点 `τ` について `Down_τ(id_0) = held_ρ(τ, C)` かつ
`Bsub_τ(id_0) = bumps_ρ(τ, C)` である。

**証明**

<1>1. `Down_τ(id_0) = held_ρ(τ, C)`。
  BY L21, DEF 割り当て
  `L21` (c) よりすべてのスロット `s` について `id_0 ∈ Anc(s)` なので、`Down_τ(id_0)` は `C` の全
  スロットにわたる `μ` の和であり、10.1 節の関係よりそれは `held_ρ(τ, C)` である。

<1>2. `Sub(id_0) = Ids(C)`。
  BY L21
  `L21` (c) より各スロット `s` について `id_0 ∈ Anc(s)` であり、`Sub(id_0)` の定義よりその `id(s)` は
  `Sub(id_0)` に入る。

<1>3. QED
  BY <1>2, 前提 (N), D27, 第 1 節
  `bumps_ρ(τ, C)` は「走査がその類について `pending` に数えている bump の個数」であり、D27 の
  `B(p, ρ)` は `origin` の `identity` で名付けられている。(N) より、`Ids(C)` の名前が付く bump は
  `C` のスロットの bump であり、その逆も成り立つ。よって
  `bumps_ρ(τ, C) = Σ_{id ∈ Ids(C)} Bmp_τ(id) = Bsub_τ(id_0)`。

### 11.4 `L23` (処分の事象に対する走査の応答)

**DEF 処分の事象**。次の 2 種を**処分の事象**と呼ぶ。

- `CancelAnalysis` の `consume` の 1 回の呼び出し (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`)
  と、それに対応する D9 の消費 1 つ。
- `RcExpr::Release` の腕の 1 回の訪問 (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`) と、
  それに対応する `Release(v, π)` の実行時の処分。

**D9 の消費と `consume` の呼び出しの対応。** `walk_inner` の `Let(x, rhs, k)` の腕は `consume_rhs` を
通じて `rhs_consumes` が挙げる各 leaf について `consume` を呼び、`Destructure` の腕は
`destructure_consumes` が挙げる各 leaf について呼ぶ
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`,
`CODE src/rc_ir/ownership.rs: rhs_consumes`, `CODE src/rc_ir/ownership.rs: destructure_consumes`)。
`L15` (e) よりすべての unit が所有されるので `owns` は常に真であり、`L16` より借用オペランドの素通しは
無いので、この 2 つが挙げる leaf は D9 の消費の表の対応する行が挙げる leaf に一致する。
**関数本体・初期化子の終端の `Ret` の消費には、対応する呼び出しが無い。** `walk_inner` の
`RcExpr::Ret` の腕は `returns_from_func` が真のとき `needed_retains` に入れるだけで `pending` を
変えない。この消費の後には `ρ` の上に時点が無いので、`L24` と `L25` の言明はそこを扱わない
(`L19` (c) と第 12 節)。

**言明**。(N) の下で、1 つの処分の事象と各 `id ∈ Ids(C)` について、その事象が `Down(id)` を減らす量を
`d` とすると、次のどちらかが成り立つ。

- **(i)** その事象で `Bsub(id)` は `d` 以上減る。
- **(ii)** 事象の後、その事象が処分したスロットのうち `id ∈ Anc(s)` であるもの `s` について、
  `Anc(s) ∩ Sub(id)` の各名前 `id'` は `Bmp(id') ≤ 0` である。

**証明**

<1>1. スロット `s` について、`Bmp(id') ≥ 1` である名前 `id'` は、ある pending の要素の `outstanding`
      が名指す。
  BY P18b
  P18b より各要素の `outstanding` は `B(p, ρ)` を `covers` するので、`B(p, ρ)[id'] ≥ 1` ならば
  `outstanding[id'] ≥ 1`、すなわち `outstanding.names(id')` が真である
  (`CODE src/rc_ir/ownership.rs: References`)。

<1>2. `consume_objects(pending, objects)` は、`outstanding` が `objects` のいずれかを名指す要素を
      すべて `pending` から取り除く。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects
  `pending.retain` の閉包は、`objects` のいずれかを `retain.outstanding.names` が真とするとき
  `false` を返す。

<1>3. CASE 事象が `consume(var, path)` の呼び出しである。
  BY <1>1, <1>2, L21, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, D15
  `consume` は `origin(vars, type_env, var, path).acted_on()` を `consume_objects` に渡す。`path` は
  boxed leaf の path であり、それが inhabited でなければ D9 の消費は参照を処分せず `d = 0` で (i) が
  成り立つ (`consume_objects` は要素を減らすだけなので `Bsub` は増えない)。inhabited なとき処分される
  スロットは `s = (var, path)` の 1 つであり、`d = [id ∈ Anc(s)]` である。`id ∉ Anc(s)` なら `d = 0`
  で同じく (i) が成り立つ。`id ∈ Anc(s)` なら、
  `L21` (a) より `Anc(s) ⊆ acted_on(s)` であり、<1>1 と <1>2 より `Anc(s)` のうち `Bmp ≥ 1` である
  名前を持つ要素はすべて取り除かれる。よって事象の後 `Anc(s)` の各名前について `Bmp ≤ 0` であり、
  (ii) が成り立つ。

<1>4. CASE 事象が `Release(v, π)` の訪問である。
  <2>1. 訪問はまず `consume_objects(pending, other_objects(v, π))` を行う。
        `other_objects(v, π)` は `π` の下の各 boxed leaf の `candidates() \ {identity()}` を集める。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects
  <2>2. この段の後、処分される各スロット `s = (v, λ)` について、`Anc(s) \ {id(s)}` の各名前
        `id'` は `Bmp(id') ≤ 0` である。
    BY <2>1, <1>1, <1>2, L21, D15
    `L21` (a) より `Anc(s) ⊆ acted_on(s) = {id(s)} ∪ candidates(s)` (D15) なので
    `Anc(s) \ {id(s)} ⊆ candidates(s) \ {id(s)}` であり、それは `other_objects(v, π)` に含まれる。
  <2>3. 続けて `un_bump(pending, acted_references(v, π))` が呼ばれる。返り値が `InBracket` のとき、
        選ばれた要素の `B(p, ρ)` からこの `Release` が `ρ` で実際に処分する参照の多重集合が引かれる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: un_bump, D27
  <2>4. CASE `un_bump` が `InBracket` を返す。(i) が成り立つ。
    BY <2>3, D27, 前提 (N), L21
    この `Release` が処分するのは `π` の下の inhabited な各 leaf の参照であり (D10)、`Down(id)` が
    減る量 `d` はそのうち `id ∈ Anc(s)` であるスロットの個数である。`B(p, ρ)` は `origin` の
    `identity` を鍵とする多重集合なので (D27)、そこから引かれる「実際に処分する参照の多重集合」も
    同じ鍵で数えたものであり、`Sub(id)` に落ちる分はちょうど
    `id(s) ∈ Sub(id)` すなわち `id ∈ Anc(s)` であるスロットの分、すなわち `d` である。(N) より
    `Sub(id) ⊆ Ids(C)` の名前が付く分はほかに無い。よって `Bsub(id)` はちょうど `d` 減る。
  <2>5. CASE `un_bump` が `OutsideBracket` を返す。(ii) が成り立つ。
    BY <2>1, <2>2, <1>1, <1>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
       `RcExpr::Release` の腕, CODE src/rc_ir/ownership.rs: acted_references
    この腕は `consume_objects(pending, un_bumped.objects())` を呼ぶ。`acted_references(v, π)` は
    `π` の下の各 boxed leaf を `origin` の `identity` で数えるので、処分される各スロット `s` の
    `id(s)` はその `objects()` に入る。よって <1>1 と <1>2 より、事象の後 `Bmp(id(s)) ≤ 0` である。
    <2>2 と合わせて `Anc(s)` の各名前について `Bmp ≤ 0` であり、その部分集合
    `Anc(s) ∩ Sub(id)` についても成り立つ。
  <2>6. CASE `un_bump` が `NoBracket` を返す。(ii) が成り立つ。
    BY <2>2, <1>1, CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: acted_references
    `NoBracket` は、`outstanding` が `un_bumped` とオブジェクトを共有する要素が `pending` に無いこと
    である。処分される各スロット `s` の `id(s)` は `un_bumped` が名指すので、どの要素の `outstanding`
    も `id(s)` を名指さない。<1>1 より `Bmp(id(s)) ≤ 0` である。<2>2 と合わせて `Anc(s)` の各名前に
    ついて成り立つ。
  <2>7. QED
    BY <2>4, <2>5, <2>6, CODE src/rc_ir/borrow.rs: un_bump
    `un_bump` は `UnBump::NoBracket`、`UnBump::OutsideBracket`、`UnBump::InBracket` のいずれかを
    返す。

<1>5. QED
  BY <1>3, <1>4
  処分の事象は `DEF 処分の事象` の 2 種で尽きる。

### 11.5 `L24` (区画の木の不等式)

**言明**。(N) の下で、`ρ` の上の各時点 `τ` と各 `id ∈ Ids(C)` について、`Bsub_τ(id) ≥ 1` ならば
`Bsub_τ(id) ≤ Down_τ(id) - 1` である。

**証明**

<1>1. `Ids(C)` の木について、`id` を固定し、`Sub(id)` の部分集合 `S` が「`Anc` について上に閉じて
      いる」-- すなわち `id' ∈ S` かつ `id'' ∈ Anc(id') ∩ Sub(id)` ならば `id'' ∈ S` -- とする。
      このとき `Sub(id) \ S` は、互いに比較不能な名前 `r_1, …, r_k` について
      `Sub(r_1) ⊎ … ⊎ Sub(r_k)` に分かれる。さらに `Down(r_i)` が数えるスロットは互いに素であり、
      すべて `Down(id)` が数えるスロットであり、`S` の名前を `id(s)` とするスロットはどの
      `Down(r_i)` にも数えられない。
  BY L21
  `L21` (d) より `Ids(C)` は `id_0` を根とする木であり、`Sub(id)` はその部分木である。`id'' ∈ Sub(id)`
  で `id'' ∉ S` であるものについて、`Anc(id'') ∩ Sub(id)` は `id''` から `id` へ至る鎖であり、`S` が
  上に閉じているのでその鎖は「`S` に入らない先頭部分」と「`S` に入る後続部分」に分かれる。先頭部分の
  最後の元を `r(id'')` と置くと、`r(id'')` の親は `S` に在るか `id''` の鎖が `id` で終わるかである。
  相異なる `r` は比較不能である -- `r_i ∈ Anc(r_j)` とすると `r_j` の鎖は `r_i` を通り、`r_i` の親から
  上はすべて `S` に在るので `r_i` は `r(r_j)` の定義に反する。`Sub(r_i)` は互いに素であり (ある名前が
  2 つの比較不能な名前を `Anc` に持つことは、`Anc` が線形順序であること (`L21` (d)) に反する)、
  `Sub(r_i) ⊆ Sub(id) \ S` である -- `id'' ∈ Sub(r_i)` は `r_i ∈ Anc(id'')` を意味し、`id'' ∈ S` と
  すると `S` が上に閉じていることから `r_i ∈ S` となって `r_i` の取り方に反する。逆に
  `id'' ∈ Sub(id) \ S` は `Sub(r(id''))` に入るので、この和は `Sub(id) \ S` に等しい。`Down(r_i)` が数えるスロット `s` は `r_i ∈ Anc(s)` を満たし、
  `id ∈ Anc(r_i) ⊆ Anc(s)` (`L21` (d)) なので `Down(id)` にも数えられる。`S` の名前を `id(s)` と
  するスロット `s` は `Anc(s) ∋ id(s) ∈ S` であり、`r_i ∈ Anc(s)` とすると `r_i` は `s` の鎖の上で
  `id(s)` より上にあり、`S` が上に閉じているので `r_i ∈ S` -- 矛盾。

<1>2. `ρ` の上で `Down` と `Bsub` を動かす事象は次の 6 種で尽きる。
  BY DEF 割り当て, D27, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  `Down` を動かすのは `μ` を動かす事象、すなわち `DEF 割り当て` の 6 種である。`Bsub` を動かすのは
  D27 が `B(p, ρ)` を動かすと述べる事象と、要素が `pending` を離れる事象である。両者を並べると:
  **(E-生成)** D10 の初期値と生成、**(E-Retain)** `Retain` 節点の訪問、**(E-移動-同名)** `L21` (e) の
  前者の移動、**(E-移動-新名)** `L21` (e) の後者の移動、**(E-処分)** `DEF 処分の事象`、
  **(E-落とし)** `merge` と、処分に伴わない `consume_objects` による要素の除去。
  `Eval` の訪問と `Match` 節点の訪問自身はどちらも動かさない (D9 の 2 つの表に行が無く、
  `walk_inner` の `Eval` の腕は `pending` を素通しし、`Match` の腕は `consume_rhs` を呼ばない)。
  関数本体・初期化子の終端の `Ret` の消費は `Down` を減らすが、その後に `ρ` の上の時点が無いので
  言明の対象にならない (`L23` の `DEF 処分の事象` の最後の段落)。

<1>3. 事象の前の時点で言明が成り立つならば、その事象の後の時点でも成り立つ。
  <2>1. CASE (E-生成)。
    BY L20, D27
    `L20` より `C` の生成事象は `ρ` の上に高々 1 つであり、それが起きる前に `C` はスロットを持たない。
    生成の直後、`Ids(C) = {id_0}` であり、`id_0` を名前とする bump は無い -- 要素が `pending` に
    入るのは `Retain` 節点の訪問だけであり (D27)、その名前はその時点で値を得ている変数の leaf の
    `identity` なので、`id_0` の変数が値を得る前には作れない。よって `Bsub = 0` で言明は空虚に真で
    ある。
  <2>2. CASE (E-Retain)。
    BY D27, DEF 割り当て, L17, L21
    `Retain(v, π)` の訪問は、`π` の下の inhabited かつ計数下の各 leaf `λ` について `μ(v, λ)` を
    1 上げ (`DEF 割り当て`)、同時に押し込まれる要素の `B(p, ρ)[id(v, λ)]` を 1 上げる (D27)。
    `id` を固定し、`k := #{λ : λ は π の下の inhabited な leaf で (v, λ) は C のスロットであり
    id ∈ Anc(v, λ)}` と置くと、`Down(id)` も `Bsub(id)` もちょうど `k` 増える
    (`id(v, λ) ∈ Sub(id)` と `id ∈ Anc(v, λ)` は同値、11.2 節)。`k = 0` の `id` では何も変わらない。
    `k ≥ 1` のとき、差 `Down(id) - Bsub(id)` は変わらないので、事象の前に `Bsub(id) ≥ 1` であれば
    言明はそのまま保たれる。事象の前に `Bsub(id) ≤ 0` であったときは、`L17` よりその `Retain` が
    触れる `k` 個のスロットは訪問の入口で `μ ≥ 1` なので `Down(id) ≥ k` であり、事象の後
    `Down(id) ≥ 2k` かつ `Bsub(id) ≤ k` である。`k ≥ 1` より `2k ≥ k + 1` なので
    `Bsub(id) ≤ Down(id) - 1` である。
  <2>3. CASE (E-移動-同名)。
    BY DEF 割り当て, L21
    `L21` (e) の前者の場合であり、`Anc(s') = Anc(s)` である。`μ(s)` が 1 減り `μ(s')` が 1 増えるので、どの `id` についても `Down(id)` は変わらない。
    `B(p, ρ)` は動かない (D27) ので `Bsub` も変わらない。
  <2>4. CASE (E-移動-新名)。
    BY DEF 割り当て, L21, D27
    `L21` (e) の後者の場合であり、`Anc(s') = {id(s')} ∪ Anc(s)` かつ `id(s') ∉ Anc(s)` である。`id ∈ Anc(s)` については `Down(id)` は
    `-1` (移動元) `+1` (移動先) で変わらず、`Bsub(id)` も変わらない。`id = id(s')` については、
    移動の前に `id(s')` を `Anc` に持つスロットは無いので `Down(id(s')) = 0` から 1 になり、
    `Sub(id(s')) = {id(s')}` で、`id(s')` の変数はこの移動で初めて値を得るので `Bmp(id(s')) = 0`
    である (<2>1 と同じ理由)。よって `Bsub(id(s')) = 0` で言明は空虚に真である。
  <2>5. CASE (E-処分)。
    <3>1. `id ∉ Anc(s)` がすべての処分されるスロット `s` について成り立つ `id` では、`Down(id)` は
          変わらず `Bsub(id)` は増えない。
      BY L23, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
         CODE src/rc_ir/borrow.rs: un_bump, D27
      `consume_objects` は要素を取り除くだけ、`un_bump` は `InBracket` のとき 1 つの要素の `B` を
      引くだけなので、`Bsub` はどの `id` についても増えない。
    <3>2. 残る `id` について、`L23` の (i) が成り立つ場合は言明が保たれる。
      BY L23
      `Down(id)` が `d` 減り `Bsub(id)` が `d` 以上減るので、`Bsub(id) ≤ Down(id) - 1` は
      `Bsub` が 1 以上である限り保たれる。
    <3>3. 残る `id` について、`L23` の (ii) が成り立つ場合も言明が保たれる。
      BY L23, <1>1, L21, 帰納法の仮定
      `S := ∪_s (Anc(s) ∩ Sub(id))` と置く。ここで `s` はこの事象が処分したスロットのうち
      `id ∈ Anc(s)` であるものを走る。`S` は `Sub(id)` の中で `Anc` について上に閉じている --
      `id' ∈ Anc(s) ∩ Sub(id)` かつ `id'' ∈ Anc(id') ∩ Sub(id)` ならば `L21` (d) より
      `id'' ∈ Anc(s)` である。`L23` の (ii) より、事象の後 `S` の各名前 `id'` は `Bmp(id') ≤ 0` で
      ある。<1>1 より `Sub(id) \ S = Sub(r_1) ⊎ … ⊎ Sub(r_k)` と分かれる。よって事象の後
      `Bsub(id) ≤ Σ_i Bsub(r_i)` である。
      処分されたスロット `s` は `id(s) ∈ S` なので、<1>1 より どの `Down(r_i)` にも数えられない。
      したがって `Down(r_i)` はこの事象で変わらず、`Bsub(r_i)` は増えない (<3>1 と同じ理由) ので、
      帰納法の仮定 (事象の前の時点での言明) より `Bsub(r_i) ≥ 1` である `i` については事象の後も
      `Bsub(r_i) ≤ Down(r_i) - 1` である。
      `Bsub(id) ≥ 1` とすると、`Bsub(r_i) ≥ 1` である `i` が少なくとも 1 つあり、その個数を `k'` と
      すると
      `Bsub(id) ≤ Σ_{i : Bsub(r_i) ≥ 1} Bsub(r_i) ≤ Σ_{i : Bsub(r_i) ≥ 1} (Down(r_i) - 1)
       ≤ Down(id) - k' ≤ Down(id) - 1`
      である。最後から 2 つ目の不等号は、<1>1 より `Down(r_i)` が数えるスロットが互いに素で
      すべて `Down(id)` に数えられることによる。
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>6. CASE (E-落とし)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, D27
    `merge` が返す `pending` は `pending_in` を `uniform` で絞ったものであり、`uniform` に入るのは
    すべてのアームの出口に同じ `outstanding` で現れる要素だけなので、`ρ` が選んだアームの出口の
    `pending` の部分集合である。よって要素は減るだけであり、D27 より残る要素の `B(p, ρ)` は運ばれる。
    処分に伴わない `consume_objects` も要素を取り除くだけである。どちらも `Down` を変えず `Bsub` を
    増やさないので言明は保たれる。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <1>2

<1>4. QED
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: cancel
  `ρ` の上の事象の列についての帰納。`ρ` の上の各時点は、この列のある接頭のすべての事象が起きた後に
  来る。活性化の開始時、`pending` は空であり (`cancel` は
  `analysis.walk(body, PendingRetains::default(), true)` で走査を始める)、`C` はスロットを持たない
  ので `Bsub ≡ 0` かつ `Down ≡ 0` で言明は空虚に真である。段は <1>3 であり、<1>2 が事象を尽くす。

### 11.6 `L25` ((O2))

**言明**。(N) の下で、`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化、各時点、各計数下の
別名類 `C` について、`bumps_ρ(・, C) ≥ 1` ならば `held_ρ(・, C) ≥ 1 + bumps_ρ(・, C)` である。
`L7` より、これは `U + X ≥ D` と同値である。

**証明**

<1>1. `C` がスロットを持たない時点では `bumps_ρ(・, C) = 0` である。
  BY D27, D6, 前提 (N)
  D27 より要素が `pending` に入るのは `Retain` 節点の訪問だけであり、その `B(p, ρ)` の名前は、
  その時点で値を得ている変数の leaf の `origin` の `identity` である。`C` のスロットが 1 つも無い
  時点では `Ids(C)` は空なので、(N) の下で `C` について数えられる bump は無い。

<1>2. `C` がスロットを持つ時点では、`L24` を `id_0` に当てて `L22` で読み替えると言明が出る。
  BY L24, L22
  `L22` より `Down(id_0) = held_ρ(・, C)`、`Bsub(id_0) = bumps_ρ(・, C)`。`L24` を `id = id_0` に
  当てると、`bumps ≥ 1` のとき `bumps ≤ held - 1` である。

<1>3. QED
  BY <1>1, <1>2, L7
  `L7` の恒等式 `held - (1 + bumps) = U + X - D` より、`held ≥ 1 + bumps` と `U + X ≥ D` は同値で
  ある。`L7` の前提のうち `L4` の前提は `L20` が与え、(N) と (I) と (A) は 6.2 節の前提である。

## 12. A19 (ii-b) が延びない点

`L19` (c) は、関数本体・初期化子の終端の `Ret` の消費の後、各計数下の別名類の `held` が 0 であることを
示す。A19 (ii-a) はこの点でも成り立つ (`0 ≥ 0`)。**A19 (ii-b) はこの点では偽である。** この節はその形の
`insert_rc` の出力を挙げる。

`Arr` を boxed な型とし、`Pair` を `Arr` を 2 つ持つ unbox 構造体とする。`make_pair : (Arr, Arr) -> Pair`
は `InlineLLVMMakeStructBody` であり、その `result_prov` は unbox 構造体について、結果の leaf `[i] ++ σ`
を単一の `Arg(i, σ)` と宣言する (`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody`)。
`borrows_operand` は既定の偽である (`CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand`)。

骨格 `S_g` (関数 `g`、パラメータ `m : Arr`、`borrowed_units` は空、返り値の型 `Pair`):

```
Let(x, Llvm(make_pair, [m, m]), Ret(x))
```

**`insert_rc` の出力。** `Ret(x)` の腕は `live_after = ∅` の下で `live = {x}` を返し、`retain_if_live` は
発火しない。`insert_into_operation_let` は `live_cont = {x}` の下でオペランドを逆順に走る --
第 2 の `m` は `used_later` が偽 (`live_cont` に `m` が無い) なので何も置かず、第 1 の `m` は
`live_after_operand` に `m` が入っているので `used_later` が真で `retains_before` に入る。`x` は
`live_cont` に在るので `after` は空である。`live_before` は `({x} \ {x}) ∪ {m} = {m}` であり、
`insert_into_func` の `unused` は空である。よって出力は

```
Retain(m, [], RcState::Unknown,
Let(x, Llvm(make_pair, [m, m]),
Ret(x)))
```

である (`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let`,
`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func`)。

**別名類と割り当て。** `origin(x, [0])` と `origin(x, [1])` はどちらも `origin(m, [])` である
(`Binding::Llvm` の単一 `Arg` の枝)。`m` はパラメータなので `origin(m, []) = Exactly((m, []))` であり、
`(m, [])` が ρ-終端である。よって `C := {(m, []), (x, [0]), (x, [1])}` は 1 つの別名類であり、
`id` はどのスロットについても `(m, [])` である。実行路は 1 本で、`μ` は次のように動く。

| 時点 | `μ(m, [])` | `μ(x, [0])` | `μ(x, [1])` | `held` | `bumps` |
|---|---|---|---|---|---|
| `Retain(m, [])` の入口 | 1 | 0 | 0 | 1 | 0 |
| `Let(x, …)` の入口 | 2 | 0 | 0 | 2 | 1 |
| `Ret(x)` の入口 | 0 | 1 | 1 | 2 | 1 |
| 終端の `Ret` の消費の後 | 0 | 0 | 0 | 0 | 1 |

`Retain(m, [])` の要素の `outstanding` は `acted_references(m, []) = {(m, []): 1}` であり、`B` も
`{(m, []): 1}` である (D27)。`Let(x, Llvm(make_pair, [m, m]), ・)` の訪問は `consume_rhs` を呼ぶが、
`rhs_consumes` の `Llvm` の腕は素通しの leaf を消費として報告しない -- `passthrough_arg_leaves` が
`(0, [])` と `(1, [])` を返し、`m` の boxed leaf は `[]` だけだからである
(`CODE src/rc_ir/ownership.rs: rhs_consumes`, `CODE src/rc_ir/ownership.rs: passthrough_arg_leaves`)。
よってこの要素は落ちない。終端の `Ret` の腕は `returns_from_func` が真なので要素を `needed_retains` に
入れるが、`pending` からは取り除かない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Ret` の腕)。

最後の行で `held = 0` かつ `bumps = 1` なので `held ≥ 1 + bumps` は偽である。その 1 つ前の行までは
`held ≥ 1 + bumps` が成り立つ (`2 ≥ 2`) ので、`L25` の言明 -- 活性化が生きている間の各時点についての
主張 -- はこの本体でも成り立つ。

**この本体は D12 を満たす。** `Obl` と `H(O_m)` は、活性化の開始で `{O_m}, 1`、`Retain(m, [])` の後
`{O_m, O_m}, 2`、`make_pair` の後も `{O_m, O_m}, 2` (素通しは参照を作らず処分しない)、終端の `Ret` の
消費で `{}, 2` -- 2 つの参照は呼び出し元へ渡る。(S-a) は各除去がその時点の `Obl` に入っており、(S-b) は
終端の消費の後 `Obl` が空であり、(S-c) は `H(O_m) ≥ 1` の下で読み・触れるので、それぞれ成り立つ。
