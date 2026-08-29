# (P-insert): `insert_rc` の出力と A19 (ii)

この文書は、`p13-disposals-and-pending.md` が `(P-insert)` として書き出した言明を扱う。README の定義
D1-D27、仮定 A1-A19、命題 P1-P27 の**言明**の上に立つ。加えて `p13-disposals-and-pending.md` の第 7 節の
局所の定義 (DEF 実行時の作用、DEF bump の帰属、DEF ρ-歩みと ρ-終端、DEF 別名類、DEF 類ごとの参照) と
補題 `L9`・`L12`・`L13`・`L16` の**言明**を使う。証明対象のコードは `src/rc_ir/rc_insert.rs` の全体である。
対象コミットは `59dff7b5` である。

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
  (第 7 節の `L10`)。この判定は節点 1 つを見れば済み、`p13-disposals-and-pending.md` が `C1` と `C2` に
  ついて挙げている理由より狭い (第 9 節の差し戻し D)。
- 残る義務は第 8 節が書く。**この文書は A19 (ii) を証明しない。**

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

**時点。** `p13-disposals-and-pending.md` の第 7.6 節の差し戻し 6 と同じく、**時点**とは節点の訪問の
入口とする。`held_ρ(n, C)`、`bumps_ρ(n, C)` は、`ρ` の上の節点 `n` の入口における値である。

**この文書の補題は `L1` から `L11` と番号を付ける。** 他のファイルの補題を引くときは
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

`B_0` は A19 (ii) の弱い形 -- 「`held ≥ bumps` であり、`bumps ≥ 1` のときは `held ≥ 1 + bumps`」、
第 9 節の差し戻し B -- を満たす。`Retain(p, [])` の要素の `outstanding` は
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
    BY D9, A1, CODE src/rc_ir/borrow.rs: split_rc_units, D14
    消費の表の `App` の行の後半は「呼び出し先がその位置の unit を所有する」引数の leaf を挙げる。
    `L3` の <1>1 と同じ理由で、`insert_rc` の出力ではすべての関数の `borrowed_units` が空であり、
    D14 よりすべての位置が所有される。
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

## 8. 残る義務

`L7` より、A19 (ii) は各別名類 `C` と各時点について `U + X ≥ D` である。`L9` と `L11` は、`R` を増やす
節点 -- `Retain` -- が、その変数を名指す構文の直前にしか立たないことを示す。残るのは次の 2 つである。

**(O1) 類ごとの規律。** `L4` の前提 (各別名類の生成事象が `ρ` の上に 1 つだけ在ること) と、
`held_ρ(n, C) ≥ 0` が `insert_rc` の出力で成り立つこと。これは D11 を別名類の粒度へ絞った主張であり、
A1 (入力が RC 規律を満たす) を `insert_rc` について示す作業の一部である。README は A1 の果たす者を
`insert_rc` と書いており、その証明は無い。

**(O2) 帳簿の遅れが無いこと。** 各別名類の各処分について、`U + X` が同時に増えるか、増えない分が
それより前の `X` の余剰で埋まっていること。`L9` と `L11` はこのうち 1 つの場合を閉じる -- `Retain(v, π)` の
直後の構文が `v` の leaf `μ ∈ L(v, π)` を**消費**するとき、その消費が `consume` に渡す `acted_on(v, μ)` は
`identity(v, μ)` を含み、その `Retain` の要素の `outstanding = ActRefs(v, π)` はその名前を鍵に持つ
(`ActRefs` は `π` の下の各 boxed leaf を `origin` の `identity` で数える) ので、`consume_objects` が
その要素を落とす (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/ownership.rs:
acted_references`)。消費される leaf は、`split_rc_units` が作った鎖のちょうど 1 つの節点の `π` の下に
ある (P1)。閉じないのは、直後の構文が `v` の leaf を**移動**させる場合 -- `L9` の (b)、
(c) の名前付きフィールド、(d) の payload 束縛、(a) の `Var` と `Llvm` の素通し -- であり、移動の先の
スロットの `identity` が `v` の側と異なりうるのは `Binding::Join` の腕が候補を 2 つ以上持つとき
だけである (`CODE src/rc_ir/ownership.rs: origin_inner` -- `Binding::Move`、`Binding::Field` の unbox の
腕、`Binding::Payload` の catch-all と unbox 変位の腕、`Binding::Llvm` の単一 `Arg` の腕は、いずれも
呼んだ `origin` の値をそのまま返す)。

**(O2) の残る場合の形。** `Match` の束縛変数 `m` の `origin` が 2 つ以上の候補を持つとき、`m` の
`identity` は `(m, λ)` であり、アームが返した変数 `p` の `identity` とは異なる。この形で `D` が増えて
`U + X` が増えないのは、`p` の側の処分が `m` の側の要素に届かないときである。`insert_rc` はこの位置に
`retain_if_live(&x, live_after, ret)` で `Retain(p)` を置き (`L9` の (b))、その要素はアームの中で
作られるので `merge` が `pending` から落とす (`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge` --
`entered_with.contains(&retain)` が偽の要素は `needed_retains` に入り、返り値には入らない)。すなわち
`X` がそこで増える。**この文書はこの埋め合わせが常に足りることを証明しない。**

## 9. README と `p13-disposals-and-pending.md` へ差し戻す点

### 差し戻し A (`(P-insert)` の言明を差し替える)

`p13-disposals-and-pending.md` の第 7.5.6 節の `(P-insert)` は、第 4 節の `B_0` -- `insert_rc` が
実際に出力する 5 節点の本体 -- で偽である。数え落としを直した `(P-insert-net)` は `C2` が満たすので
A19 (ii) を導かない (第 5 節)。第 6 節の `L7` の形 -- 各別名類、各時点について `U + X ≥ D` -- が、
A19 (ii) が要求しているものである。言明をこの形に差し替えることを提案する。

### 差し戻し B (A19 (ii) に `bumps ≥ 1` の限定を足す)

A19 (ii) は「各時点 `τ` と各別名類 `C` について `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)`」と書かれている。
この形は、類の最後の参照が処分された後の時点で偽になる。第 4 節の `B_0` の終端の `Ret(w)` の入口では
`held_ρ0(·, C_p) = 0` かつ `bumps_ρ0(·, C_p) = 0` であり、`0 ≥ 1` は成り立たない。

`p13-disposals-and-pending.md` の第 7.5.4 節が使っているのは弱い形である -- `<1>3` は
「各 `C` について `held ≥ bumps`」と「`bumps ≥ 1` である `C0` について `held ≥ bumps + 1`」の 2 つだけを
足し合わせる。A19 (ii) を次の形に書き直すことを提案する。

> **(ii)** 各時点 `τ` と各別名類 `C` について `held_ρ(τ, C) ≥ bumps_ρ(τ, C)` であり、
> `bumps_ρ(τ, C) ≥ 1` のときは `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` である。

### 差し戻し C (別名類を計数下のものに限る)

`DEF 類ごとの参照` の表は、類の ρ-終端が生成で作られるか、パラメータ・capture の leaf であるときに
`held` の開始値を与える。グローバル値の leaf を ρ-終端とする類にはどの行も当てはまらず、`held` が
定まらない。D26 はグローバル状態のオブジェクトを D8 の参照の勘定の外に置くので、A19 (ii) と
`(P-insert)` の量化を「`obj(C)` が計数下である別名類」に限ることを提案する。第 6 節はその限定の下で
書いている。

### 差し戻し D (`C1` と `C2` が現れない理由を `L9` に差し替える)

`p13-disposals-and-pending.md` は、`C1` が現れない理由を「`App(f, [p])` は `Match` の後の `p` の使用なので、
`insert_rc` はアーム 0 の `Ret(p)` の前に `Retain(p)` を置く」と書き、`C2` が現れない理由を
「`insert_rc` は変数が live な位置にしか `Retain` を置かず、消費された変数はその消費より後では live で
ない」と書いている。

後者は `insert_rc` の liveness と合わない。`insert_rc` の liveness は使用だけから作られ、消費を読まない
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let` の `live_before` の作り方 --
オペランドの名前を無条件に足す)。`C2` の `o` は `App(id, [o])` の後に `Eval(o)` で使われるので、
`App(id, [o])` の後の位置で live である。`C2` が `insert_rc` の出力でない理由は、`Retain(o, [])` の
直後の構文が `o` を名指さないことである (第 7.4 節の `L10`)。

`L9` は `C1` と `C2` の両方を、節点 1 つを見るだけで弾く。両者の理由をこれに差し替えることを提案する。
