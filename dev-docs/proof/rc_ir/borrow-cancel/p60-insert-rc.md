# (P-insert): `insert_rc` と `split_rc_units` の出力と A19

この文書は、`p13-disposals-and-pending.md` が `(P-insert)` として書き出した言明を扱う。README の定義
D1-D34、仮定 A1-A25、命題 P1-P30 の**言明**の上に立つ。加えて `p13-disposals-and-pending.md` から次の
2 つを輸入する。

- 補題 `L9`・`L12a`・`L14` の**言明**。`L9` は `origin` の `identity` が
  inhabited (D16) を決めることを、`L12a` は 1 つの別名類に属する 2 つのスロットが同じオブジェクトを
  指すこと -- すなわち D33 の `obj(C)` が定まること -- を与える。`L14` の言明は 2 つの節からなる。
  - `ρ` の上のスロット `(x, λ)` について、`o = origin(x, λ).identity()` は `ρ` の上の位置 (D6) --
    スロットか記号の位置 -- であり、`obj(o) = obj(C_ρ(x, λ)) = obj(x, λ)` である。
  - **`obj(x, λ)` が D26 の意味で計数下であるとき、`o` は `ρ` の上のスロットであって
    `C_ρ(o) = C_ρ(x, λ)` であり、`o` は `ρ` で活性 (p13 の `DEF 名前の活性`) であって
    `obj_ρ(o) = obj(x, λ)` である。** 第 6.1b 節の `L5b` が読むのは、この後半のうち
    「`o` は `ρ` の上のスロットであって `C_ρ(o) = C_ρ(x, λ)`」の部分である。
- 第 7.5.7 節の反例 `C1` と第 7.5.8 節の反例 `C2` -- その実行路の上の別名類 (`C1` では `m` と `p` が
  1 つの類に属し、`C2` では `o` と `y` が別の類に属すること)、callee の leaf が指すオブジェクトが
  グローバル状態であること、この 2 つが D12 を満たすこと、そして A19 (ii) を破る時点と、その時点での
  `held` と `bumps` の値。**本体はこの文書が書き写す** -- `C1` は第 1 節に、`C2` は第 5 節に。

証明対象のコードは `src/rc_ir/rc_insert.rs` の全体と、
`src/rc_ir/borrow.rs` の `split_rc_units`・`split_body`・`split_body_inner`・`split_rc`、およびこれらが
呼ぶ `src/rc_ir/ownership.rs` の `units_under`・`subtree_type`・`rc_units`・`rc_units_go`・`unit_step` と
`src/rc_ir/leaf_map.rs` の `boxed_leaf_paths` である。
対象コミットは `f7d5469f1c7bef5b7eaade9cb8a4e9d56c9b0290` である。

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
  と、`L4`・`L7` が前提に持つ「開始事象は高々 1 つ」(`L20`) を出す。(ii-a) が節点の入口の外に置く
  1 点 -- 終端の `Ret` の消費を行った直後 -- は `L19` (c) が `held = 0` の等式で与える。
- **(O2) は証明されている。** 第 11 節が、1 つの別名類のスロットに `origin` の
  `identity` が付ける名前の全体 `Ids(C)` が木をなすこと (`L21`) を示し、**その木の各部分木について
  「付いている bump の総和はその部分木が保持している参照の個数より真に小さい」**(`L24`) を示す。
  木の根で読むと `held ≥ 1 + bumps` になる (`L25`)。**「名前は別名類を決める」は `L5b` が
  `p13` の `L14` の後半から出す** (6.1b 節)。第 11 節が本体について
  読むのは前提 (S) の 4 つだけであり、`insert_rc` の出力がそれを満たすことは `L20a` が与える。
- **A19 (i) はこの文書の義務ではない。** (i) は仮定ではなく、D21 が活性化に課す**制限**である。
  実行 (D24) が作る活性化がその制限を満たすことは P28 (b) が示す。この文書が (i) について読むのは、
  `held` と `μ` の差 -- 借用する (D14) unit の下の leaf を ρ-終端とする類の開始の 1 -- が (i) の
  `d(C)` が引く角括弧と同じものであることだけであり、それは `L13a` (b)(c) が与える。
- **`split_rc_units` の段も閉じている。** A19 の範囲は `borrow_ify` の入力、すなわち `split_rc_units` の
  出力である。第 13 節が、`Retain(v, π)` を `units_under(ty(v), π)` の鎖へ割る段について、(ii-a) が
  保存されること (`L31`) と、(ii-b) が出力について成り立つこと (`L32`) を示す。
  支えは、**unit が `π` の下の boxed leaf を分割する**こと (`L27`) と、**割る段が束縛を 1 つも作らない**
  ので `origin`・別名類・`held` が変わらないこと (`L29`) である。(i) はこの段に何も要求しない --
  D21 の制限であり、実行が作る活性化がそれを満たすことは P28 (b) が示す。
- **A19 (ii-b) は活性化の終わりの 1 点先へは延びない。** 終端の `Ret` の消費の後、`held` は 0 になるが
  (`L19` (c))、走査の `RcExpr::Ret` の腕は pending の要素を取り除かないので `bumps ≥ 1` が残りうる。
  第 12 節の `L38` がその形の `insert_rc` の出力を挙げる。(ii-a) はこの点でも成り立つ。

## 1. 記法

`insert_rc(prog, type_env)` は `prog` の各関数の `body` と各グローバル初期化子の `init` を書き換える
(`CODE src/rc_ir/rc_insert.rs: insert_rc`)。

- **骨格**とは `insert_rc` の入力の本体である。A25 より骨格は `RcExpr::Retain` と `RcExpr::Release` を
  含まない。
- `ty(x)`、`origin(x, π)`、`acted_on(x, π)`、`L(v, π)`、`ActRefs(v, π)` は
  `p13-disposals-and-pending.md` の第 1 節と同じ意味で使う。
- `needs_rc(v)` は `RcInserter::needs_rc(v)`、すなわち `!v.ty.is_fully_unboxed(type_env)`
  (`CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc`)。

**時点。** この文書では**時点**とは節点の訪問の入口を指す。`held_ρ(n, C)`、`bumps_ρ(n, C)` は、
`ρ` の上の節点 `n` の入口における値である。

**A19 (ii) の読み。** README の形で読む。**(ii-a)** は、各時点と各**計数下の**別名類 `C` に
ついて `held_ρ(τ, C) ≥ 0` であり、読む構文と `Retain`/`Release` がその類を名指す時点では
`held_ρ(τ, C) ≥ 1` であることである。**非負であることは、終端の `Ret` の消費を行った直後の時点に
ついても言う。** **(ii-b)** は、`bumps_ρ(τ, C) ≥ 1` である時点では
`held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であることである。この文書が単に「A19 (ii)」と書くときは (ii-b)
を指す。計数下の類に限るのは、グローバル値を ρ-終端とする類に D34 の表が開始値を
与えないからである (D26)。**`held` の帰属を定めるのは D34 の表、`bumps` の帰属を定めるのは
A19 (ii-b) の本文である** -- 「`bumps_ρ(τ, C)` とは、時点 `τ` の `pending` の各要素 `p` について
D27 が定める `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和である」。

**この文書の補題は `L1` から `L38` と番号を付ける。** 番号は固定された名札であり、間に挟むときは
`L13a` のように枝番を振る。`L30` と `L33` から `L37` までの番号は使わない。他のファイルの補題を引く
ときは `p13 の L14` のように書く。

**`C1` の道具立てと本体。** `Arr` を boxed な型、`I` を `is_fully_unboxed` が真の型、`Bl` を 2 つの変位が
どちらも payload を持たない unbox union とする。`Llvm` 演算 `alloc : () -> Arr` と `mkbl : () -> Bl` は
結果の leaf を単一の `Fresh` と宣言し、`zero : () -> I` は boxed leaf を持たない結果を返す。どれも
オペランドを持たない。関数 `f` (パラメータ `b : Arr`、`borrowed_units` は空、返り値の型 `I`) の本体は
`Let(z, Llvm(zero, []), Release(b, [], s, Ret(z)))` であり、関数 `main` (パラメータ無し、capture 無し、
返り値の型 `I`) の本体は次である。

```
Let(p, Llvm(alloc, []),
Let(q, Llvm(alloc, []),
Let(c, Llvm(mkbl, []),
Let(m, Match(c, [ MatchArm { tag: Some(0), payload: y0, body: Ret(p) },
                  MatchArm { tag: Some(1), payload: y1, body: Ret(q) } ]),
Retain(m, [], s,
Let(u, App(f, [p]),
Let(w, App(f, [q]),
Eval(m,
Release(m, [], s,
Ret(u))))))))))
```

`main` の束縛変数 `p`・`q`・`c`・`m`・`u`・`w`・`y0`・`y1` と `f` のパラメータ `b`・束縛変数 `z` は
いずれも名前空間を持たない名前であり、`f` と `main` は `prog.funcs` の鍵である。

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

これは D34 の表で `held_ρ(·, C)` を
**減らす** 2 種類の事象の、`n` より後の分の個数である。「参照を持つ」は同じ表の `held_ρ(n, C)` である。
すなわち `(P-insert)` は次を主張する。

> `ρ` の上の各節点 `n` と各別名類 `C` について、`held_ρ(n, C) ≥ Fut_ρ(n, C)`。

**量化の範囲。** D34 は「`held_ρ(τ, C)` はその時点以後の `τ` についてだけ定まり、`C` を渡る総和は
どれもその条件を付ける」と述べる。ここでいう「その時点」は `C` の ρ-終端の変数が値を得る時点であり、
この文書の不等式はどれも、`C` についてはその時点以後の `n` を走る。

## 3. この文書が示すこと

- **R1** (第 4 節)。`(P-insert)` を破る `insert_rc` の出力が在る。
- **R2** (第 5 節)。`Fut` から未実行の `Retain` の分を引いた形 `(P-insert-net)` は、
  `p13-disposals-and-pending.md` の反例 `C2` が満たす。`C2` は A19 (ii) を破るので、
  `(P-insert-net)` だけから A19 (ii) は出ない。
- **L5a**、**L5b**、**L7** (第 6 節)。走査の帳簿 `B(p, ρ)` が `outstanding` の一部と一致すること
  (`L5a`)、名前が別名類を決めること (`L5b`)、および A19 (ii) が別名類ごとの台帳の不等式 `U + X ≥ D` と
  同値であること (`L7`)。
- **L9**、**L10** (第 7 節)。`insert_rc` の出力の `Retain` の位置の形と、それが `C1` と `C2` を弾くこと。
- **L12**、**L13** (第 9 節)。A19 を読む 2 つの形が、どちらも他方を導かないこと。
- **L13a** - **L20** (第 10 節)。(O1)。移動の辺の両端が同じ別名類に属し `held` がスロットごとの割り当ての
  和であること (`L13a`)、終端の `Ret` を書き換える呼び出しの `live_after` が空であること (`L16a`)、
  `insert_rc` の liveness と実行時の参照の分布が一致すること
  (`L18`)、終端の `Ret` の前に `Retain` が立たないこと (`L18a`)、そこから出る別名類の粒度の RC 規律
  (`L19`)、および開始事象の一意性 (`L20`)。
- **L20a** - **L25** (第 11 節)。(O2)。この節が本体について読む 4 つを前提 (S) として括り出し、
  `insert_rc` の出力がそれを満たすこと (`L20a`)、名前の鎖が作る木 (`L21`)、その部分木についての
  不等式 (`L24`)、そこから出る `held ≥ 1 + bumps` (`L25`)。
- **L38** (第 12 節)。A19 (ii-b) が活性化の終わりの 1 点先では偽であること。
- **L26** - **L32** (第 13 節)。`split_rc_units` について。書き換えの形 (`L26`)、unit による leaf の分割
  (`L27`)、`insert_rc` の出す path が空列であること (`L28`)、束縛・`origin`・別名類・`held` が変わらない
  こと (`L29`)、A19 (ii-a) の保存 (`L31`)、出力が (S) を満たすので `L25` から
  A19 (ii-b) が出ること (`L32`)。

## 4. R1: `(P-insert)` は `insert_rc` の出力で偽である

### 4.1 反例のプログラム

第 1 節の `C1` の道具立てのうち `Bl` と `mkbl` を除いたものを使う。`Arr` を boxed な型、`I` を
`is_fully_unboxed` が真の型とする。`Llvm` 演算 `alloc : () -> Arr` は結果の leaf を単一の `Fresh` と
宣言し、`zero : () -> I` は boxed leaf を持たない結果を返す。どちらもオペランドを持たない。

**名前の取り方。** 以下の骨格のパラメータと束縛変数の名前は、いずれも名前空間を持たないものを取る。
関数 `f` と `main` は `prog.funcs` の鍵であり、A13 の最後の段落よりその名前は局所名ではない。

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

<1>1. `needs_rc(p)` と `needs_rc(b)` は真であり、`needs_rc(u)`、`needs_rc(w)`、`needs_rc(z)` は偽で
      ある。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, CODE src/ast/types.rs:
     TypeNode::is_fully_unboxed
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` である。`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので、boxed な型では偽である。`ty(p) = ty(b) = Arr`
  は boxed なので `needs_rc` は真であり、`ty(u) = ty(w) = ty(z) = I` は 4.1 の仮定より
  `is_fully_unboxed` が真なので `needs_rc` は偽である。

<1>2. `p`、`u`、`w`、`z`、`b` の名前は `is_local` が真であり、`f` の名前は偽である。
  BY A13, 4.1 節の名前の取り方, CODE src/ast/name.rs: FullName::is_local
  `is_local` は名前空間が空かを答える。4.1 節の名前の取り方より `p`、`u`、`w`、`z`、`b` の名前は
  名前空間を持たない。`f` は `prog.funcs` の鍵であり、A13 の最後の段落が「最上位の記号の名前は局所名
  ではない」-- `FullName::is_local` が偽であり、`prog.funcs` の鍵はそのような名前である -- と述べる。

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
  BY <1>1, <1>2, A15, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: insert_if_local,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: rhs_operands,
     CODE src/rc_ir/rc_insert.rs: build_releases
  A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。
  `insert_into_expr_inner` の `RcExpr::Ret(x)` の腕は `live = live_after ∪ {z}` を作り
  (`insert_if_local`、<1>2 より `z` は入る)、`retain_if_live(&z, live_after, ret)` を呼ぶ。
  `live_after = ∅` は `z` を含まないので `live.contains(&var.name)` が偽であり、節点はそのまま返る。
  `Let(z, Llvm(zero, []), Ret(z))` はオペランドを持たず、`live_cont = {z}` が `x = z` を含むので
  `after` は空であり、`live_before = {z} \ {z} = ∅` である。`insert_into_func` はこの `live` が `b` を
  含まないことと `needs_rc(b)` が真であること (<1>1) から `unused = [b]` を作り、
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
  BY D4, D6, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/ast/types.rs: TypeNode::is_unbox
  `is_fully_unboxed` は `if self.is_box(type_env) { return false; }` で始まるので、boxed な `Arr` では
  偽である。`is_unbox` は `self.is_closure() || toplevel_tycon_info(type_env).is_unbox` であり
  `is_box` はその否定なので、`is_box` が真の `Arr` では `is_closure` が偽である。よって D4 の判定は
  第 1 規則 (`is_fully_unboxed`) と第 2 規則 (クロージャ) を抜けて第 3 規則 (`is_box`) に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。
  `ty(u) = ty(w) = I` は 4.1 の仮定より `is_fully_unboxed` が真なので、D4 の第 1 規則より leaf を
  持たない。

<1>2a. `B_0` に現れる残りの `RcVar` は `App` の callee `f` であり、その各 boxed leaf `λ` について
       `(f, λ)` は記号の位置であってスロットではない。
  BY D6, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/lower.rs: Lowerer::lower_var
  `collect_bindings` は `f` に束縛を入れない (`f` は `B_0` のどの `Let`、`Destructure`、`Match` の
  束縛変数でもない)。`f` はパラメータでも capture でもないので `vars.bindings` に束縛を持たず、
  D6 の「値を得る 3 つの形」の 3 つ目 -- `Lowerer::lower_var` の `resolve` が `None` を返す腕が作る名前 --
  である。D6 よりその対は記号の位置であってスロットではない。

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
  BY <1>3, <1>2, <1>2a, D6, D33
  <1>3 の計算で `origin_inner` は `origin` を 1 度も呼ばない (`operand_units` が空なので
  `origin_from_leaves_under` の中の `origin` の呼び出しも起きない)。よって `(p, [])` は ρ-終端である。
  別名類は ρ-終端が等しいスロットの集まりなので、`(p, [])` を ρ-終端とするスロットを数えればよい。
  `B_0` に現れる `RcVar` は `p`、`u`、`w`、`f` であり、<1>2 より前 3 者のスロットは `(p, [])` だけ、
  <1>2a より `f` はスロットを持たない。

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
  BY L2, D10, D34
  `C_p` の ρ-終端 `(p, [])` は D10 の生成の表の `Llvm` の行で作られる (宣言が単一の `Arg` でないため)。
  表の第 1 行より `held` は 1 から始まる。`n_R` の入口までに起きた事象はこの生成だけである -- `n_R` は
  `Let(p, Llvm(alloc, []), ·)` の直後の節点であり、その間に `Retain`、`Release`、消費は無い。

<1>6. QED
  BY <1>4, <1>5

### 4.5 R1 の結論

**R1**。`(P-insert)` は偽である。

**証明**

<1>1. `B_0` は `insert_rc` の出力の本体である。
  BY L1

<1>2. `B_0` は実行路 `ρ_0` を持ち、`C_p` は `ρ_0` を辿る活性化の別名類である。
  BY L2

<1>3. `ρ_0` の上の節点 `n_R` の入口という時点において
      `held_ρ0(n_R, C_p) = 1 < 2 = Fut_ρ0(n_R, C_p)` である。
  BY L3

<1>4. QED
  BY <1>1, <1>2, <1>3, 第 2 節の読み
  第 2 節の読みより `(P-insert)` は「`ρ` の上の各節点 `n` と各別名類 `C` について
  `held_ρ(n, C) ≥ Fut_ρ(n, C)`」である。<1>1 から <1>3 が、`insert_rc` の出力の 1 つの本体と、
  その 1 本の実行路と、その上の 1 つの別名類と、1 つの節点でその不等式が破れる例を与える。

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

**`C2` の道具立てと本体。** 第 1 節の `C1` の道具立てに、関数 `id` (パラメータ `a : Arr`、
`borrowed_units` は空、返り値の型 `Arr`、本体 `Ret(a)`) を足す。`f` は第 1 節のものである。`C2` の
関数 `main` (パラメータ無し、capture 無し、返り値の型 `I`) の本体は次である。

```
Let(o, Llvm(alloc, []),
Let(y, App(id, [o]),
Retain(o, [], s,
Let(u, App(f, [y]),
Eval(o,
Release(o, [], s,
Ret(u)))))))
```

`main` の束縛変数 `o`・`y`・`u` と `id` のパラメータ `a` はいずれも名前空間を持たない名前であり、
`id`・`f`・`main` は `prog.funcs` の鍵である。

**DEF `Ret#`**。`ρ` の上の節点 `n` と別名類 `C` について、`Ret#_ρ(n, C)` を、`ρ` の上で `n` より後にある
`Retain(v, π)` 節点であって `(v, λ) ∈ C` である `λ` を `π` の下に持つものについて、そのような `λ` の
個数の総和とする。`DEF Fut` と同じく、`n` 自身が `Retain` であるときその `Retain` は「`n` より後」に
数える。

**DEF `(P-insert-net)`**。`ρ` の上の各節点 `n` と各**計数下の**別名類 `C` について、
`held_ρ(n, C) ≥ Fut_ρ(n, C) - Ret#_ρ(n, C)`。計数下の類に限るのは、D34 が `held` を計数下の類にしか
定めないからである。

**DEF 開始事象**。別名類 `C` の**開始事象**とは、
D34 の表で `held_ρ(・, C)` に開始値 1 を与える 3 行 -- `C` の終端が D10 の生成で
作られる行、`C` の終端が所有するパラメータ・capture の leaf である行、`C` の終端が借用する
パラメータ・capture の leaf である行 -- のいずれかに当たる `ρ` の上の事象をいう。D10 の生成の表は
5 行を持ち、パラメータ・capture が値を得ることはその表に無いので、開始事象は D10 の生成より広い。

### 5.1 `L4` (`held` は開始と増減で決まる)

**言明**。`ρ` の上の各節点 `n` と、`ρ` の上で開始事象を 1 つだけ持つ各別名類 `C` について、
`held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)` である。ここで `R_ρ(n, C)` は `n` より前に実行された
`Retain(v, π)` 節点の `(v, λ) ∈ C` である `λ` の個数の総和、`D_ρ(n, C)` は `n` より前に実行された
`Release(v, π)` 節点の同じ数え上げと `C` のスロットの D9 の消費の個数の和である。

**証明**

<1>1. `held_ρ(·, C)` を動かす事象は、D34 の表の 6 行で尽きる。
  BY D34
  その定義は `held_ρ(·, C)` を表で定めており、表に無い事象はこの量を動かさない。

<1>2. 表の最初の 3 行は `held_ρ(·, C)` に開始値 1 を与え、仮定よりそのうち 1 つだけが `ρ` の上で
      起きる。
  BY <1>1, DEF 開始事象, 言明の仮定 (`C` は `ρ` の上で開始事象を 1 つだけ持つ)
  `DEF 開始事象` はこの 3 行に当たる事象を開始事象と呼ぶ。

<1>3. 表の残る 3 行は、`(v, λ) ∈ C` である `λ` を `π` の下に持つ `Retain(v, π)` につき `+1`、
      同じ形の `Release(v, π)` につき `-1`、`C` のスロットの D9 の消費につき `-1` である。
  BY <1>1, D34

<1>4. QED
  BY <1>1, <1>2, <1>3
  `R_ρ(n, C)` は <1>3 の第 1 行の、`D_ρ(n, C)` は第 2 行と第 3 行の、`n` より前の個数の定義そのもので
  ある。<1>2 の開始値 1 に <1>3 の増減を足すと `held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)` である。

### 5.2 `L5` (`C2` は `(P-insert-net)` を満たす)

**言明**。第 5 節が書き写した `C2` の `main` の本体とその唯一の実行路 `ρ` について、`ρ` の上の
計数下の別名類は `C_o = {(o, [])}` と `C_y = {(y, [])}` の 2 つであり、`(P-insert-net)` の不等式は、
どちらの類についても、その類の開始の時点以後の `ρ` の上のすべての節点で成り立つ。

**証明**

<1>0. `ρ` の上の計数下の別名類は `C_o = {(o, [])}` と `C_y = {(y, [])}` の 2 つである。
  BY 第 5 節の `C2` の本体, p13 の第 7.5.8 節 (`o` と `y` が別の別名類であること), D4, D6, D26,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox
  `C2` の `main` が名指す変数は `o`、`y`、`u` と `App` の callee `id`・`f` である。`ty(o) = ty(y) = Arr`
  は boxed なので `is_fully_unboxed` は偽であり (`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まる)、`is_unbox` は
  `self.is_closure() || toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので `Arr` では
  `is_closure` も偽である。よって D4 の判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(u) = I` は `is_fully_unboxed` が真なので D4 の第 1 規則より
  leaf を持たない。callee は束縛を持たない名前なので、その boxed leaf との対は D6 の記号の位置であって
  スロットではなく、そこが指すのは funptr かグローバル状態のオブジェクトであって D26 より計数下では
  ない。よってスロットは `(o, [])` と `(y, [])` の 2 つであり、第 7.5.8 節よりこの 2 つは別の類に
  属する。

<1>1. `C_o` の上の事象は、`ρ` の順に、生成 (`Llvm(alloc, [])`)、消費 (`App(id, [o])` の引数)、
      `Retain(o, [])`、`Release(o, [])` の 4 つである。
  BY <1>0, 第 5 節の `C2` の本体, D9, D10
  第 5 節が書き写した `C2` の `main` の節点を `ρ` の順に見る。`Eval` は D9 の消費でも移動でもない。
  `App(f, [y])` が消費するのは `(y, [])` であり、<1>0 より `(y, [])` は `C_o` のスロットではない。

<1>1a. `C_o` の開始の時点は `Let(o, Llvm(alloc, []), ·)` の実行の直後であり、`ρ` の上でそれ以後の節点は
       `Let(y, App(id, [o]), ·)` から `Ret(u)` までの 6 つである。
  BY <1>1, D34, D6, 第 2 節の量化の範囲
  `C_o` の ρ-終端の変数は `o` であり、`o` が値を得るのは `Let(o, Llvm(alloc, []), ·)` を実行する段で
  ある (D6 -- 節点が束縛する変数はその節点の後に値を持つ)。D34 より `held_ρ(τ, C_o)` はその時点以後の
  `τ` についてだけ定まるので、`Let(o, Llvm(alloc, []), ·)` の入口は言明の走る範囲に入らない。

<1>2. 残る 6 つの節点の入口で `C_o` について `(held, Fut, Ret#)` は次のとおりである。
  BY <1>1, L4, DEF `Fut`, DEF `Ret#`

  | 入口 | `held` | `Fut` | `Ret#` |
  |---|---|---|---|
  | `Let(y, App(id, [o]), ·)` | 1 | 2 | 1 |
  | `Retain(o, [])` | 0 | 1 | 1 |
  | `Let(u, App(f, [y]), ·)` | 1 | 1 | 0 |
  | `Eval(o, ·)` | 1 | 1 | 0 |
  | `Release(o, [])` | 1 | 1 | 0 |
  | `Ret(u)` | 0 | 0 | 0 |

<1>2a. `C_y` の上の事象は、`ρ` の順に、生成 (`App(id, [o])` の結果)、消費 (`App(f, [y])` の引数) の
       2 つである。`C_y` の開始の時点は `Let(y, App(id, [o]), ·)` の実行の直後であり、`ρ` の上で
       それ以後の節点は `Retain(o, [])` から `Ret(u)` までの 5 つである。
  BY <1>0, D6, D9, D10, D34, 第 2 節の量化の範囲
  D10 の生成の表の `App` の行が `(y, [])` の参照を作り、D9 の `App` の行が `App(f, [y])` でそれを
  消費する。`Retain(o, [])` と `Release(o, [])` が名指す `(o, [])` は <1>0 より `C_y` のスロットでは
  なく、`Eval` は D9 の消費でも移動でもない。`C_y` の ρ-終端の変数は `y` であり、`y` が値を得るのは
  `Let(y, App(id, [o]), ·)` を実行する段である (D6)。D34 より `held_ρ(τ, C_y)` はその時点以後の
  `τ` についてだけ定まる。

<1>2b. 残る 5 つの節点の入口で `C_y` について `(held, Fut, Ret#)` は次のとおりである。
  BY <1>2a, L4, DEF `Fut`, DEF `Ret#`

  | 入口 | `held` | `Fut` | `Ret#` |
  |---|---|---|---|
  | `Retain(o, [])` | 1 | 1 | 0 |
  | `Let(u, App(f, [y]), ·)` | 1 | 1 | 0 |
  | `Eval(o, ·)` | 0 | 0 | 0 |
  | `Release(o, [])` | 0 | 0 | 0 |
  | `Ret(u)` | 0 | 0 | 0 |

<1>3. QED
  BY <1>0, <1>1a, <1>2, <1>2a, <1>2b
  <1>0 より量化する類は `C_o` と `C_y` の 2 つである。`C_o` について、<1>1a より走る節点は <1>2 の
  表の 6 行で尽き、各行で `held ≥ Fut - Ret#` である
  (`1 ≥ 1`、`0 ≥ 0`、`1 ≥ 1`、`1 ≥ 1`、`1 ≥ 1`、`0 ≥ 0`)。`C_y` について、<1>2a より走る節点は
  <1>2b の表の 5 行で尽き、各行で `held ≥ Fut - Ret#` である
  (`1 ≥ 1`、`1 ≥ 1`、`0 ≥ 0`、`0 ≥ 0`、`0 ≥ 0`)。

### 5.3 R2 の結論

**R2**。`(P-insert-net)` を満たしながら A19 (ii) を破る本体が在る。

**証明**

<1>1. `C2` の `main` は A19 (ii) を破る。
  BY p13 の第 7.5.8 節 (A19 (ii) を破る時点と、その時点での `held` と `bumps` の値)
  `Retain(o, [])` の要素が `pending` に在る間 `bumps_ρ(·, C_o) = 1` であり `held_ρ(·, C_o) = 1 < 2` で
  ある。第 1 節の読みでは `bumps ≥ 1` のとき `held ≥ 1 + bumps` が要る。

<1>2. その同じ `C2` の `main` は `(P-insert-net)` を満たす。
  BY L5
  `(P-insert-net)` は「`ρ` の上の各節点 `n` と各別名類 `C` について
  `held_ρ(n, C) ≥ Fut_ρ(n, C) - Ret#_ρ(n, C)`」である (5 節の `DEF (P-insert-net)`)。`L5` は
  `C2` の `main` の計数下の別名類が `C_o` と `C_y` の 2 つであることと、その両方について不等式が
  各節点で成り立つことを与える。`held` が定まるのは計数下の類だけである (D34)。

<1>3. QED
  BY <1>1, <1>2

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
  (`CODE src/rc_ir/borrow.rs: un_bump`, D27)。
- `X_ρ(n, C)` を、`n` より前に `pending` から取り除かれた要素の、取り除かれた時点の `B_ρ` のうち
  `C` のスロットの `identity` に付いていた分の総和とする。要素を `pending` から取り除く操作は 3 つで
  ある -- `consume_objects`、`merge`、そして `un_bump` が `InBracket` の subtract の後に置く
  `if innermost.outstanding.is_empty() { pending.remove(index); }` である
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/borrow.rs:
  CancelAnalysis::merge`, `CODE src/rc_ir/borrow.rs: un_bump`)。

### 6.1a `L5a` (`B` は `outstanding` の一部と一致する)

**言明**。1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。名前 `id` (`VarPath`) を、
`ρ` の上に `id(s) = id` であるスロット `s` -- D6 よりその leaf は inhabited (D16) -- が在り、かつ
その `s` が指すオブジェクトが計数下 (D26) であるとき**帳簿の名前**と呼ぶ。走査中の各時点と `pending` の
各要素 `p` について次が成り立つ。

- **(a)** `id` が帳簿の名前であるとき `B(p, ρ)[id] = p.outstanding[id]` であり、そうでないとき
  `B(p, ρ)[id] = 0` である。
- **(b)** したがって `B(p, ρ)[id] ≥ 0` である。
- **(c)** `p.outstanding` が空であるとき `B(p, ρ)` は空である。
- **(d)** `un_bump` が `InBracket` で `p` を選ぶとき、その `Release` が `ρ` で実際に処分する参照の
  多重集合は、各名前で `B(p, ρ)` を超えない。すなわち D27 の引き算は打ち切られない。

**証明**

<1>1. `ρ` の上の 2 つの boxed leaf の `identity` が等しいとき、2 つは同時に inhabited (D16) であり、
      inhabited であるならば同じオブジェクトを指す。したがって帳簿の名前 `id` について、`identity` が
      `id` である leaf はすべて inhabited であって計数下のオブジェクトを指し、帳簿の名前でない `id`
      について、`identity` が `id` である inhabited かつ計数下の leaf は無い。
  BY p13 の L9, P5 (a), D6, D26, 帳簿の名前の定義
  p13 の `L9` は identity が inhabited を決めることを与えるので、`identity` が等しい 2 つの leaf は
  同時に inhabited である。inhabited な leaf は D6 のスロットであり、P5 (a) より `identity` が等しい
  2 つのスロットは同じオブジェクトを指す。D26 より 1 つの活性化の間、そのオブジェクトが計数下で
  あるかどうかは変わらない。第 2 文の前半はこの 2 つを帳簿の名前の定義に当てたものである。後半は
  定義そのもの -- `identity` が `id` である inhabited かつ計数下の leaf はそれ自身がスロットであり
  計数下のオブジェクトを指すので、それが在れば `id` は帳簿の名前である。

<1>2. `Retain(v, π)` の訪問が `pending` に押し込む要素について、押し込んだ直後に (a) が成り立つ。
  BY <1>1, D15, D27, D16, D26, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/ownership.rs: acted_references
  `walk_inner` の `RcExpr::Retain` の腕は `outstanding` に `acted_references(v, path)` を置く。D15 より
  それは `π` の下の**すべての** boxed leaf を `origin` の `identity` で数えた多重集合である。D27 より
  `B(p, ρ)` は `π` の下の inhabited かつ計数下の各 leaf を同じ鍵で数えたものである。<1>1 より、`id` が
  帳簿の名前であれば `π` の下の `identity` が `id` である leaf はすべて inhabited かつ計数下なので
  2 つの数は等しく、そうでなければ `B(p, ρ)[id] = 0` である。

<1>3. `p` が `pending` に在る間、`B(p, ρ)` と `p.outstanding` を動かすのは `un_bump` が `InBracket` で
      `p` を選ぶ `Release` の訪問だけであり、その訪問は `p.outstanding` から
      `R := acted_references(v, π)` を引き、`B(p, ρ)` からその `Release` が `ρ` で実際に処分する参照の
      多重集合 `A` を引く。
  BY D27, CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
  `outstanding` を書き換える式は `un_bump` の `innermost.outstanding.subtract(un_bumped)` の 1 つだけで
  あり、`un_bumped` は `Release` の腕が渡す `acted_references(v, path)` である。D27 の 2 行目が
  `B(p, ρ)` の側を定める。残る 2 つの行では両者が揃って運ばれる -- アームへの複製は `pending` を
  `clone` するので `outstanding` をそのまま写し、D27 より `B(p, ρ)` も運ばれる。`merge` が返り値に
  据える要素の `outstanding` は `uniform` の値であり、`uniform` に入るのはすべてのアームの出口に
  同じ `outstanding` で現れる要素だけなので、それは `ρ` が選んだアームの出口での値に等しく、D27 より
  `B(p, ρ)` もその側から運ばれる。

<1>4. `id` が帳簿の名前であるとき `A[id] = R[id]` であり、そうでないとき `A[id] = 0` である。
  BY <1>1, D10, D15, D16, D26, D27, P6, P18b
  D10 の `Release` の行より `Release(v, π)` が処分するのは `π` の下の inhabited な各 leaf の参照で
  あり、D26 よりそのうち計数下のオブジェクトを指すものだけが D8 の意味の参照を持つ。P6 がこの
  数え上げと実行時に処分される参照の多重集合の一致を与える。`A` はそれを `origin` の `identity` で
  名付けたものであり (D27 と P18b の言明が `B` をその鍵で数えるのと同じ鍵である)、`R` は D15 より
  同じ `π` の下の**すべての** boxed leaf を同じ鍵で数えたものである。<1>1 より、帳簿の名前については
  両者が数える leaf が一致し、そうでない名前については `A` が数える leaf が無い。

<1>5. (a)。
  BY <1>2, <1>3, <1>4
  `p` が `pending` に在る間の事象の個数についての帰納。基底は <1>2、段は <1>3 と <1>4 -- 帳簿の名前で
  は両辺から同じ `R[id] = A[id]` が引かれ、そうでない名前では `B` の側が 0 のまま動かない。

<1>6. (b) と (c)。
  BY <1>5, CODE src/rc_ir/ownership.rs: References
  `outstanding` は `References`、すなわち `VarPath` から個数への写像であり、その値は 0 以上である。
  (a) より `B(p, ρ)[id]` はその値か 0 なので 0 以上である。`p.outstanding` が空 -- どの名前の個数も
  0 -- であれば、(a) より `B(p, ρ)` のどの名前の個数も 0 である。

<1>7. (d)。
  BY <1>4, <1>5, CODE src/rc_ir/borrow.rs: un_bump
  `un_bump` が `InBracket` を返すのは `innermost.outstanding.covers(un_bumped)` が真のときであり、
  そのとき各名前で `p.outstanding[id] ≥ R[id]` である。帳簿の名前では <1>5 より
  `B(p, ρ)[id] = p.outstanding[id] ≥ R[id] = A[id]` (<1>4) であり、そうでない名前では
  `A[id] = 0 = B(p, ρ)[id]` である。

<1>8. QED
  BY <1>5, <1>6, <1>7

### 6.1b `L5b` (名前は別名類を決める)

**言明**。1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。`C` を `obj(C)` が計数下
(D26) である別名類、`s_0 = (u, σ)` を `C` のスロットとする。`ρ` の上のスロット `(v, λ)` であって
`obj(v, λ)` が計数下であり `origin(v, λ).identity() = origin(u, σ).identity()` であるものは、`C` の
スロットである。

**この形で足りる。** この補題を読む段が主語にする leaf は、どれも D27 が数える leaf -- inhabited (D16)
かつ計数下 (D26) -- である。D6 より inhabited な leaf は `ρ` の上のスロットであり、それが指す
オブジェクトが計数下であることが第 2 の仮定である。

**証明**

<1>1. `o := origin(u, σ).identity()` は `ρ` の上のスロットであり `C_ρ(s_0) = C` かつ
      `C_ρ(o) = C` である。
  BY p13 の L14, p13 の L12a, D33, 言明の仮定
  `s_0` は `C` のスロットなので、D33 より `C_ρ(s_0) = C` であり、p13 の `L12a` より
  `obj(s_0) = obj(C)` である。言明の仮定よりそれは計数下 (D26) である。p13 の `L14` の後半を
  `s_0` に当てると、`o` は `ρ` の上のスロットであって `C_ρ(o) = C_ρ(s_0) = C` である。

<1>2. `o = origin(v, λ).identity()` であり、`o` は `ρ` の上のスロットであって
      `C_ρ(o) = C_ρ(v, λ)` である。
  BY p13 の L14, 言明の仮定
  言明の仮定より `(v, λ)` は `ρ` の上のスロットであって `obj(v, λ)` は計数下なので、p13 の `L14` の
  後半が `(v, λ)` に当たる。2 つの `identity` が等しいことも言明の仮定である。

<1>3. QED
  BY <1>1, <1>2, D33
  D33 より別名類は ρ-終端が等しいという関係でスロットを分けた同値類であり、ρ-終端はスロットごとに
  1 つに決まるので、1 つのスロットはちょうど 1 つの類に属する。よって `C_ρ(o)` は `o` がスロットで
  あることだけで定まる。<1>1 と <1>2 よりその 1 つの類は `C` にも `C_ρ(v, λ)` にも等しいので
  `C_ρ(v, λ) = C` であり、`(v, λ)` は `C` のスロットである。

**この補題は循環しない。** p13 の `L14` の証明が引くのは、同じ文書の `L8`・`L8a`・`L9`・`L12`・`L12a`
と D4・D6・D15・D26・D33・A8 と `origin_inner` のコードであって、A19 も P18a もこの文書の補題も
引かない。
p13 の第 7.1 節は本体を 1 つ固定するだけで `borrow_ify` の出力に限らないので、この補題は `insert_rc` の
出力と `split_rc_units` の出力にも当たる。

### 6.2 `L6` (bump の残高)

**言明**。次の前提の下で、`bumps_ρ(n, C) = R_ρ(n, C) - U_ρ(n, C) - X_ρ(n, C)` である。

- **(I)** `C` のスロットは `ρ` の上で inhabited であり、`obj(C)` は計数下である。

**(I) の前半は D6 から出る。** D6 のスロットの leaf は inhabited なので、この前提が本体について
言うのは後半 -- `obj(C)` が計数下であること -- だけであり、6.1 節はその条件を満たす `C` を固定して
いる。

**証明**

<1>1. `B_ρ` を動かす事象は、D27 の 3 つの箇条が挙げるもので尽きる。
  BY D27
  D27 は `B(p, ρ)` を 3 つの箇条で定め、最後の箇条が「ほかのどの操作も `B(p, ρ)` を変えない」と
  述べる。

<1>1a. `Retain(v, π)` の訪問が `B_ρ` に足す量のうち `C` の名前 -- `C` のスロットの `identity` -- に
       付く分は、`π` の下の `(v, λ) ∈ C` である `λ` の個数に等しい。
  BY L5b, D27, D6, D16, D26, D33, 前提 (I)
  D27 の第 1 箇条より、`Retain(v, π)` の訪問が押し込む要素の `B(p, ρ)` は、`π` の下の inhabited (D16)
  かつ計数下 (D26) の各 leaf を `origin` の `identity` で名付けて数えたものである。よって `C` の名前に
  付く分は、`π` の下の inhabited かつ計数下であって `identity` が `C` のスロットの `identity` に等しい
  leaf の個数である。D6 より inhabited な leaf は `ρ` の上のスロットなので、`L5b` よりそのような leaf は
  `C` のスロットである。逆に `(v, λ) ∈ C` で `λ` が `π` の下に在るならば、D6 より `λ` は inhabited で
  あり、D33 と前提 (I) より `obj(v, λ) = obj(C)` は計数下であり、その `identity` は `C` のスロットの
  `identity` である。よって 2 つの個数は等しい。

<1>2. `B_ρ` を増やすのは D27 の第 1 箇条 -- `Retain(v, π)` の訪問が要素を `pending` に押し込む段 --
      だけであり、`n` までにそれが `C` の名前に足す量の総和は `R_ρ(n, C)` である。
  BY <1>1, <1>1a, D27, L4
  D27 の第 2 箇条は引く側であり、第 3 箇条は運ぶか定めないかである。<1>1a より、`Retain(v, π)` が
  `B_ρ` に足す量のうち `C` の名前に付く分は、`π` の下の `(v, λ) ∈ C` である `λ` の個数に等しい。
  `R_ρ(n, C)` (`L4`) はその個数の `n` より前の総和である。

<1>3. `B_ρ` を減らすのは D27 の第 2 箇条 -- `un_bump` が `InBracket` で要素を選ぶ `Release` の訪問 --
      だけであり、`n` までにそれが `C` の名前から引く量の総和は `U_ρ(n, C)` である。
  BY <1>1, D27, 6.1 節の `U_ρ(n, C)` の定義

<1>4. 要素が `pending` を離れるとその `B_ρ` は `bumps` の和から落ち、`n` までに落ちた量のうち `C` の
      名前に付く分の総和は `X_ρ(n, C)` である。
  BY <1>1, L5a, 6.1 節の `X_ρ(n, C)` の定義
  `bumps_ρ(n, C)` は `pending` に在る要素についての総和なので、`pending` を離れた要素は数えられない。
  6.1 節の `X_ρ(n, C)` は 3 つの取り除く操作すべてについての総和であり、そのうち `un_bump` が
  `outstanding` の空になった要素を外す分は `L5a` (c) より 0 を寄せる。`un_bump` が同じ訪問で引いた分は
  `U_ρ(n, C)` が数えている (<1>3) ので、二重には数えられない。

<1>5. 残るもの -- アームへの複製と、`merge` が要素を返り値に据える段 -- は `B_ρ` を変えない。
  BY <1>1, D27
  D27 の第 3 箇条が「アームへの複製と、`merge` が `p` を返り値に据えるときは、`B(p, ρ)` をそのまま
  運ぶ」と述べる。

<1>6. QED
  BY <1>1, <1>1a, <1>2, <1>3, <1>4, <1>5, L5b, D27
  `bumps_ρ(n, C)` は `pending` の要素の `B_ρ` のうち `C` の名前に付く分の総和である (A19 (ii-b) が
  `bumps` の帰属をそう定める)。<1>2 から <1>5 より、その値は足した分 `R` から引いた分 `U` と落ちた分
  `X` を除いたものである。D27 が `B_ρ` に量を足すのは inhabited かつ計数下の leaf についてであり、
  `L5b` よりそのうち `C` の名前を持つものは `C` のスロットに限るので、`C` の外のスロットが `C` の
  名前へ量を足すことはない。

### 6.3 `L7` (A19 (ii) の台帳形)

**言明**。`L6` の前提 (I) と `L4` の前提 (開始事象が 1 つ) の下で、`ρ` の上の節点 `n` について
次が成り立つ。

```
held_ρ(n, C) - (1 + bumps_ρ(n, C)) = U_ρ(n, C) + X_ρ(n, C) - D_ρ(n, C)
```

とくに `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)` と `U_ρ(n, C) + X_ρ(n, C) ≥ D_ρ(n, C)` は同値である。

**証明**

<1>1. `held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)`。
  BY L4, 言明の前提 (`L4` の前提)

<1>2. `bumps_ρ(n, C) = R_ρ(n, C) - U_ρ(n, C) - X_ρ(n, C)`。
  BY L6, 言明の前提 (`L6` の前提 (I))

<1>3. `held_ρ(n, C) - (1 + bumps_ρ(n, C)) = U_ρ(n, C) + X_ρ(n, C) - D_ρ(n, C)`。
  BY <1>1, <1>2
  <1>1 の右辺から <1>2 の右辺に 1 を足したものを引くと
  `(1 + R - D) - (1 + R - U - X) = U + X - D` である。

<1>4. QED
  BY <1>3
  <1>3 より 2 つの式は同じ整数である。よって一方が 0 以上であることと他方が 0 以上であることは
  同値であり、すなわち `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)` と
  `U_ρ(n, C) + X_ρ(n, C) ≥ D_ρ(n, C)` は同値である。

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

<1>1. `insert_rc` の出力の `RcExpr::Retain` 節点は、すべて `build_retains` の中の 1 つの式が作った
      ものである。
  BY A25, CODE src/rc_ir/rc_insert.rs: build_retains
  `src/rc_ir/rc_insert.rs` で `RcExpr::Retain` を構成する式はこの 1 つである。A25 より骨格 (第 1 節) は
  `Retain` 節点を含まないので、出力の `Retain` 節点はこのパスが作ったものに限る。

<1>2. `build_retains` を呼ぶのは `insert_into_operation_let` と `retain_if_live` の 2 か所である。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live

<1>3. `retain_if_live` を呼ぶのは、言明の 2・3・4 に挙げた 3 か所である。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `insert_into_expr_inner` の `RcExpr::Ret(x)` の腕、`insert_into_destructure`、`insert_into_match` の
  3 か所である。

<1>4. QED
  BY <1>1, <1>2, <1>3
  <1>1 より `Retain` 節点を作る位置は `build_retains` の呼び出しに尽き、<1>2 と <1>3 よりその呼び出し元は
  言明の 4 か所である。

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
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: build_retains,
     CODE src/rc_ir/rc_insert.rs: build_releases
  これらの関数は `RcExprNode` を作って返すだけで、返した節点の継続を差し替える式を持たない。呼び出し元が
  するのは、返された節点を別の構成子の継続 (`cont` または `node`) として渡すことだけであり、
  `build_releases` と `build_retains` も渡された節点を継続として**包む**。よって出力の `Retain` 節点の
  継続は、それが作られた時点の継続である。

<1>3. CASE `t` が `L8` の 1 で作られた。
  BY L8, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner
  `build_retains(retains_before, node)` の `node` は `Let(x, rhs, cont)` であり、`rhs` は `Match` では
  ない -- `insert_into_expr_inner` は `RcExpr::Let(x, RcRhs::Match(..), cont)` を
  `insert_into_match` へ振り分けるので、`insert_into_operation_let` に来る右辺は `Match` でない。
  `n_t` はこの `Let` である。`retains_before` に入るのは、`rhs_operands(rhs)`
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
    BY <2>1, <2>2, <2>3, <2>5, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
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

### 7.3 `L9` の系: `n_t` に着くと `v` の参照は手を離れる

**言明**。`L9` の各場合について、`n_t` は `v` の inhabited な各 boxed leaf の参照を D9 の意味で消費するか
移動させる。ただし場合 (d) で scrutinee が boxed であって選ばれたアームが変位アーム (`tag` が `Some`) で
あるときは、`n_t` は移動も消費もせず、`insert_into_match` がそのアームの頭に置いた `Release(v, [])` が
その参照を処分する。D9 は「`Eval(v, k)` と `Let(x, Match(v, arms), k)` の `Match` 節点自身は、参照を
作らず、移さず、手放さない」と述べており、この場合の処分は `Match` 節点ではなくその `Release` が行う。

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
  BY D9, D16, D21, A16, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `v` が unbox union のとき、変位アームの payload 束縛と catch-all アームの payload 束縛は移動の表の
  行である。移動の表の行が名指すのは scrutinee の**活性**変位の参照なので、選ばれたアームの `tag` が
  実行時のタグに等しいことが要る -- D21 より活性化は実行時のタグに `tag` が等しいアームを選び、A16 より
  そのようなアームか catch-all アームが在る。D16 より scrutinee の inhabited な leaf はその変位の下に
  あるものだけである。`v` が boxed union のとき、`insert_into_match` は `release_container` が真になり、
  各変位アームの先頭に `Release(v, [])` を置く (`head.push(scrut.clone())` と `build_releases(head, body)`)。
  catch-all アームでは payload 束縛が移動の表の行である。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, L9, D9
  `L9` の 4 つの場合が尽くす。(a)(b)(c) と、(d) のうち scrutinee が unbox であるか catch-all アームが
  選ばれる場合は <1>1 から <1>4 が消費か移動を与える。(d) の残る場合 -- scrutinee が boxed で変位アームが
  選ばれる場合 -- は <1>4 が `Release(v, [])` による処分を与える。

### 7.4 `L10` (`C1` と `C2` の `main` は `insert_rc` の出力ではない)

**言明**。`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` の `main` の本体と、第 7.5.8 節の `C2` の
`main` の本体は、どちらも `insert_rc` の出力ではない。

**証明**

<1>1. `C1` の `main` の `Retain(m, [], s, k)` について、`k` は `Let(u, App(f, [p]), ·)` である。
  BY 第 1 節の `C1` の本体

<1>2. `C1` の `main` は `L9` を破る。
  BY <1>1, L9, CODE src/rc_ir/rc_insert.rs: rhs_operands
  `n_t = Let(u, App(f, [p]), ·)` は `L9` の (a) の形だが、`rhs_operands(App(f, [p]))` は
  `[(f, Own), (p, Own)]` であって `m` を含まない。(b)(c)(d) の形でもない。

<1>3. `C2` の `main` の `Retain(o, [], s, k)` について、`k` は `Let(u, App(f, [y]), ·)` である。
  BY 第 5 節の `C2` の本体

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

<1>3a. `rewrite_inner` の `App` の腕は callee を `route` の結果で差し替える。差し替わりうる名前は
       局所名ではないので、`insert_rc` が `Retain` を置いた変数ではありえない。
  BY P12, A13, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
     CODE src/rc_ir/borrow.rs: borrow_funcref,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/ast/name.rs: FullName::is_local
  P12 より `route` が返す呼び出し先は、元の呼び出し先と同じ関数の版 (元の版そのものか、その
  `borrow_versions` の像) であるか、局所変数を経由する間接呼び出しでは呼び出し先そのものである。
  後者では名前が変わらない。前者で名前が変わるとき、元の名前は `borrow_ify` の入力の `funcs` の鍵で
  あり、A13 の最後の段落よりそれは局所名ではない。返る名前は `borrow_funcref` が元の名前の最後の
  断片に `#borrow` を足したものであり、名前空間の欄を書き替えない。`FullName::is_local` は名前空間が
  空かを答えるので、返る名前も局所名ではない。一方 `insert_rc` が `Retain` を置くのは
  `insert_into_operation_let` の `if v.name.is_local()` の門の中と、`retain_if_live` の
  `var.name.is_local()` を要求する枝だけなので、`Retain(v, π)` の `v` は局所名である。よって
  差し替えが起きる callee の名前は `v` と異なり、`rhs_operands` はその `App` について `v` を
  `Ownership::Own` で挙げたままである。

<1>4. `before` の各 `Retain(a, u)` について、その直後の非 `Retain` 節点はその `App` の `Let` であり、
      `a` はその `args` の元である。
  BY <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
  `call_rc` は `args` の元だけを `before` に入れる。`rhs_operands(App(callee, args))` は `args` の各元に
  `Ownership::Own` を与えるので、これは `L9` の (a) の形である。

<1>5. `clone_func` が作る借用版は元の本体の束縛変数を一斉に付け替えたものである。
  BY P9
  付け替えは節点の種類・並び・どの変数を名指すかを変えないので、`L9` の形は保たれる。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5, L9
  `insert_rc` の出力が持つ形 (`L9`) は `split_rc_units` (<1>1) と `borrow_ify` (<1>2、<1>3、<1>3a、
  <1>5) を通って残り、`borrow_ify` が足す `Retain` も同じ形を持つ (<1>4)。

## 8. (O1) と (O2) の言明

`L7` より、A19 (ii-b) は各別名類 `C` と `bumps ≥ 1` である各時点について `U + X ≥ D` である。`L9` と
`L11` は、`R` を増やす節点 -- `Retain` -- が、その変数を名指す構文の直前にしか立たないことを示す。
README の第 4 節が (O1) と (O2) と呼ぶ 2 つを、この節が言明として書く。

**(O1) 由来の形。** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、(a) 各時点の
各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` であり、(b) D7 の読む構文が読む値の各スロット、および
`Retain(v, π)`・`Release(v, π)` が触れる各スロットについて、そのスロットが属する計数下の別名類は、その
節点の入口で `held ≥ 1` であり、(c) 関数本体・初期化子の終端の `Ret` の消費を行った直後の点でも
`held ≥ 0` である。これは D11 を別名類の粒度へ絞った主張であり、A19 (ii-a) が要求するものである
-- (c) は A19 (ii-a) の「非負であることは、終端の `Ret` の消費を行った直後の時点についても言う」の
分である。**第 10 節の `L19` が示す。** 併せて `L20` が、`L4` と `L7` が前提に持つ「各別名類の生成
事象は `ρ` の上に高々 1 つ」を示す。

**(O2) 帳簿の遅れが無いこと。** 各時点の各計数下の別名類について、`bumps ≥ 1` ならば
`held ≥ 1 + bumps` である。`L7` より `U + X ≥ D` と同値であり、A19 (ii-b) が要求するものである。
**第 11 節の `L25` が前提 (S) の下で示し、`L20a` が `insert_rc` の出力について (S) を与える。**

### 8.1 (O2) の証明が本体について読むもの

第 11 節が本体とそれが属するプログラムについて読むのは、`DEF 割り当て` の `μ` と、11.1 節の前提 (S) が
挙げる 4 つ -- (S0) 各時点で各スロットの `μ` が非負であること、(S1) すべての unit が所有されること、
(S2) 各 `Retain` 節点の入口でその節点が触れる各スロットの `μ` が 1 以上であること、(S3) 各計数下の
別名類に開始値 1 を与える事象が高々 1 つであること -- だけである。`insert_rc` の出力がその 4 つを
満たすことは `L20a` が第 10 節から出す。

不等式の形はこうである。台帳が処分に遅れうるのは、処分されるスロットの名前を、pending な要素の
`outstanding` が名指さないときだけであり、`L23` がその場合を切り分ける。名前どうしの関係 `Anc` は
`Ids(C)` を木にし (`L21`)、走査の帳簿はその木の各名前に付く。第 11 節はこれを木の各部分木についての
1 つの不等式 (`L24`) に束ね、木の根で読んで `held ≥ 1 + bumps` を得る (`L22`、`L25`)。

### 8.2 `split_rc_units` の段

A19 の範囲は「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体」であり、
`borrow_ify` の入力は `split_rc_units` の出力である。第 10 節と第 11 節が示すのは `insert_rc` の出力に
ついてなので、`Retain(v, [])` を unit ごとの鎖へ割る段が 1 つ残る。**第 13 節がそれを扱う。**

- **(ii-a)** は保存の形で示す (`L31`)。割る段は束縛を 1 つも作らないので `origin`・別名類・`held` は
  変わらず (`L29`)、鎖の途中に新しく生まれる時点だけが検査の対象になる。入力の側 -- `insert_rc` の
  出力 -- は `L19` が与える。
- **(i)** はこの段に何も要求しない。(i) は仮定ではなく D21 が活性化に課す制限であり、実行が作る
  活性化がそれを満たすことは P28 (b) が示す。この段の出力には借用する unit が無い
  (`L15` (e)、`L26` (a)) ので、その本体のどの別名類でも (i) の角括弧は 0 である (`L13a` (c))。
- **(ii-b)** は、割る段の出力が第 11 節の前提 (S) を満たすことを示して `L25` を当てる (`L32`)。保存の
  形では通らない -- 鎖に割ると `pending` の要素が細かくなり、粗い 1 要素なら落ちた場面で細かい要素の
  一部が残ることがあるので、`bumps` は割る前より大きくなりうる。

## 9. A19 を読む 2 つの形

README の A19 は 2 種の読み手を持つ。P18a・P18c・P19・P21 が読むのは (ii-b) -- 走査の `pending` を
主語にする形 -- であり、P14 と P18c が読むのは次の形 ((ii-a)) である。

**DEF 由来の形**。`ρ` の上の各時点 `τ` について次の 2 つが成り立つこと。

- **(a)** 各計数下の別名類 `C` について `held_ρ(τ, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その時点で `held_ρ(τ, C) ≥ 1` である。

「由来」-- `origin` の再帰を実行が辿った枝に沿って追い切った先 -- は `p13-disposals-and-pending.md` の
ρ-終端であり、別名類は ρ-終端が等しいスロットの集まりなので (D33)、「由来ごとの参照の
個数」は `held_ρ(·, C)` である。

**この 2 つの形はどちらも他方を導かない。** 9.1 と 9.2 がそれぞれの向きの反例を挙げる。

### 9.1 `L12` (`C1` は由来の形を満たし、A19 (ii) を破る)

**言明**。`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` の `main` は、変位 0 を選ぶ実行路で
由来の形を満たす。同じ実行路で A19 (ii) を破る。

**証明**

<1>1. その実行路の計数下の別名類は `C_1 = {(p, []), (m, [])}` と `C_2 = {(q, [])}` である。
  BY 第 1 節の `C1` の本体, p13 の第 7.5.7 節 (`m` と `p` が 1 つの別名類に属すること), D4, D6, D26,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox
  第 1 節が書き写した `C1` の `main` が名指す変数は `p`、`q`、`c`、`m`、`u`、`w` と、アームの payload
  `y0`・`y1` と、`App` の callee `f` である。`ty(p) = ty(q) = ty(m) = Arr` は boxed なので
  `is_fully_unboxed` は偽であり (`if self.is_box(type_env) { return false; }` で始まる)、`is_unbox` は
  `self.is_closure() || toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので
  `is_closure` も偽である。よって D4 の判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(c) = Bl` は 2 つの変位がどちらも payload を持たない
  unbox union なので boxed leaf を持たず、payload `y0`・`y1` も同じである。`ty(u) = ty(w) = I` は
  `is_fully_unboxed` が真なので D4 の第 1 規則より leaf を持たない。`App` の callee `f` は束縛を持たない
  名前なので、その boxed leaf との対は D6 の記号の位置であってスロットではない。それが指すオブジェクトは
  グローバル状態であり (p13 の第 7.5.7 節)、D26 より計数下ではない。
  よってスロットは `(p, [])`、`(q, [])`、`(m, [])` の 3 つであり、p13 の第 7.5.7 節より `m` と `p` は
  1 つの類に属する。

<1>2. `held(C_1)` は、割り当ての後 1、`Retain(m, [])` の後 2、`App(f, [p])` の消費の後 1、
      `Release(m, [])` の後 0 である。`held(C_2)` は、割り当ての後 1、`App(f, [q])` の消費の後 0 で
      ある。
  BY <1>1, 第 1 節の `C1` の本体, L4, D34, D9, D10
  第 1 節が書き写した `C1` の `main` の節点を `ρ` の順に見る。`p` と `q` の割り当ては D10 の生成で
  あり、開始値 1 を与える。`Let(m, Match(c, ...), ·)` は変位 0 のアームを選び、そのアーム本体の
  `Ret(p)` が D9 の移動を行うが、移動は `held` を変えない (D34 の表に移動の行は無い)。
  `Retain(m, [])` は `(m, []) ∈ C_1` を名指し、`Release(m, [])` も同じである。`App(f, [p])` は
  `(p, []) ∈ C_1` を、`App(f, [q])` は `(q, []) ∈ C_2` を消費する (D9、`f` は `b` を所有する)。
  `Eval(m, ·)` は D9 の消費でも移動でもない。終端の `Ret(u)` は `ty(u) = I` に boxed leaf が無いので
  何も消費しない。

<1>3. 由来の形の (a) が成り立つ。
  BY <1>2
  <1>2 の値はすべて 0 以上である。

<1>4. 由来の形の (b) が成り立つ。
  BY <1>1, <1>2, 第 1 節の `C1` の本体, D7
  第 1 節が書き写した `C1` の `main` の読む構文と `Retain`/`Release` を順に見る (D7 の表)。
  `Let(m, Match(c, ...))` が読むのは scrutinee `c` であり、<1>1 より `c` はスロットを持たない。
  `Retain(m, [])` は `(m, [])` に触れ、その時点の `held(C_1) = 1` である。
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

4.1 の道具立て (名前の取り方を含む) と、第 5 節の関数 `id` を使う。本体 `B_1` (`main`、パラメータ
無し、返り値の型 `I`) を次で定める。

```
Let(o, Llvm(alloc, []),
Let(y, App(id, [o]),
Eval(o,
Let(u, App(f, [y]),
Ret(u)))))
```

**言明**。`B_1` は D11 と A19 (ii) を満たし、由来の形の (b) を破る。`B_1` は `insert_rc` の出力では
ない。

**証明**

<1>1. `B_1` の実行路は 1 本であり、その上の計数下の別名類は `C_o = {(o, [])}` と `C_y = {(y, [])}` で
      ある。
  BY D3, D4, D6, D26, A3, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox
  `Match` が無いので実行路は 1 本である。`collect_bindings` は `Let(o, Llvm(alloc, []), ・)` に
  `Binding::Llvm(alloc, [], Arr)` を入れる。A3 と 4.1 より `alloc` の宣言は単一の `Fresh` なので
  `as_arg_projection` は `None` を返し、`origin_from_leaves_under` はオペランドを持たないこの op に
  ついて `Exactly((o, []))` を返す。よって `origin(o, []) = Exactly((o, []))` である。`y` は `RcRhs::App` に束縛されるので `collect_bindings` は `Binding::Producer` を入れ、
  `origin_inner` の `Producer` の腕は `here()` を返して `origin` を呼ばない。よって `(o, [])` と
  `(y, [])` はどちらも ρ-終端であり、別々の類である。
  `ty(o) = ty(y) = Arr` の leaf が `[]` の 1 つであることは D4 の判定から出る -- `is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `Arr` では偽であり、`is_unbox` は
  `self.is_closure() || toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので `Arr` では
  `is_closure` が偽である。よって判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(u) = I` は 4.1 の仮定より `is_fully_unboxed` が真なので
  D4 の第 1 規則より leaf を持たない。
  `B_1` に現れる残りの `RcVar` は `App` の callee `id` と `f` であり、どちらも `collect_bindings` が
  束縛を入れない名前なので、その boxed leaf との対は D6 の記号の位置であってスロットではない。
  D6 よりそこが指すのは funptr かグローバル状態のオブジェクトであり、D26 よりグローバル状態の
  オブジェクトは計数下ではない。

<1>2. `Obl` と `H(O)` は、割り当ての後 `{O}, 1`、`App(id, [o])` が返った後 `{O}, 1`、
      `App(f, [y])` が返った後 `{}, 0` である。
  BY <1>1, A1, D6, D8, D9, D10, D14, D26,
     CODE src/rc_ir/borrow.rs: split_rc_units, CODE src/rc_ir/rc_insert.rs: insert_rc
  A1 の後半より `borrow_ify` の入力のすべての関数の `borrowed_units` は空である。`borrow_ify` の入力は
  `split_rc_units` の出力であり、`split_rc_units` も `insert_rc` も各関数の `body` と各グローバル
  初期化子の `init` しか書き換えないので、`insert_rc` の出力でも `borrowed_units` は空であり、D14 より
  `id` の `a` も `f` の `b` も所有される。
  `App(id, [o])` は `(o, [])` を消費し、`H` を動かさない。同じ節点について D9 の `App` の行は callee の
  全 boxed leaf も消費として挙げるが、<1>1 より `(id, ・)` と `(f, ・)` は記号の位置であってスロットでは
  なく、D6 よりそこが指すのは funptr かグローバル状態のオブジェクトである。D26 よりそれらは D8 の意味の
  参照を持たないので、この消費は `Obl` から何も取り除かない。
  同じ節点の D10 の生成の表の `App` の行が結果の leaf `(y, [])` の参照を作り、`H` はここでも動かない。
  `App(f, [y])` は `(y, [])` を消費し、`f` の `Release(b, [])` が `H` を 1 下げる。

<1>3. `B_1` は D11 を満たす。
  BY <1>2, D11
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
  BY A13, 4.1 節の名前の取り方,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, CODE src/ast/name.rs: FullName::is_local,
     CODE src/rc_ir/rc_insert.rs: build_retains
  `Let(y, App(id, [o]), cont)` を書き換える呼び出しの `live_cont` は `o` を含む -- `insert_into_eval` は
  返す `live_before` に `o` を入れる。`o` は局所名であり (`is_local` は名前空間が空かを答え、4.1 節の
  名前の取り方より `o` の名前は名前空間を持たない)、`rhs_operands(App(id, [o]))` は `o` に
  `Ownership::Own` を与え、
  `needs_rc(o)` は真である (`is_fully_unboxed` は `if self.is_box(type_env) { return false; }` で
  始まるので boxed な `ty(o) = Arr` では偽である、`CODE src/ast/types.rs: TypeNode::is_fully_unboxed`)。
  よって
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
どちらも成り立つ。

## 10. (O1) の証明 -- 別名類の粒度の RC 規律

この節は `insert_rc` の出力について (O1) を示す。支えるのは `insert_rc` が持つ 1 つの等式である --
**各時点において、1 つのスロットに割り当たる参照の個数は、その変数が `insert_rc` の liveness で live
かどうかで決まる。** `insert_rc` の liveness は使用だけから作られる集合であり、参照カウントも義務集合も
見ない。その集合と実行時の参照の分布が一致する、というのがこの等式である。(O1) の 2 つの節はここから
出る。

### 10.1 塊、検査点、live 集合、割り当て

`insert_rc` は骨格 (第 1 節) の各節点 `m` を、次の 4 つの部分へ写す
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner`)。

- **前置 `Retain` 鎖**: `build_retains` が積む `Retain` 節点の列 (空のこともある)。核節点の外側に立つ。
- **核節点**: 骨格節点と同じ種類の節点 (`Let`、`Destructure`、`Eval`、`Ret`)。
- **アームの頭の `Release` 鎖**: `m` が `Let(x, Match(scrut, arms), cont)` であるとき、
  `insert_into_match` が各アームの本体の頭に `build_releases(head, body)` で積む `Release` 節点の列
  (空のこともある) (`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match`)。他の 5 種の骨格節点は
  この部分を持たない。
- **後置 `Release` 鎖**: `build_releases` が積む `Release` 節点の列 (空のこともある)。核節点と `m` の
  継続の写しの間に立つ。`Ret` はこの部分を持たない (`insert_into_expr_inner` の `RcExpr::Ret` の腕は
  `build_releases` を呼ばない)。

骨格節点 `m` の**塊**とは、この 4 つの部分の節点の全体をいう。骨格節点 `m` の**検査点**とは、`m` の前置
`Retain` 鎖の最初の節点 (鎖が空なら核節点) の入口をいう。

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
述べるとおり、活性化が保持する参照を動かす事象はこの 6 種で尽きる。**`μ` の類ごとの総和が
`held_ρ(・, C)` とどれだけ違うかは、`L13a` が示す** -- 違いは、借用する (D14) unit の下の
パラメータ・capture の leaf を ρ-終端とする類の開始の 1 だけである。

**DEF `N_ρ(m, C)`**。`κ_C(v) := #{λ : λ は ty(v) の inhabited (D16) な boxed leaf で (v, λ) ∈ C}` と
置き、骨格節点 `m` について `N_ρ(m, C) := Σ_{v ∈ Λ(m)} κ_C(v)` と置く。

### 10.1a `L13a` (移動の辺は類の中で閉じ、`held` は `μ` の類ごとの総和に借用の開始を足したものである)

**言明**。1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。別名類 `C` について、
`C` の ρ-終端がその本体の関数が**借用する** (D14) unit の下のパラメータ・capture の leaf であるとき
`β(C) := 1`、そうでないとき `β(C) := 0` と置く。

- **(a)** D9 の移動の各辺について、移動元が**スロット** `s` であるとき、`s` と移動先のスロット `s'` は
  同じ別名類に属する。移動元が**記号の位置** (D6) であるときは、`s'` が属する別名類の ρ-終端はその
  記号の位置であり、その類は計数下ではない。
- **(b)** `ρ` の上の各時点 `τ` と各**計数下の**別名類 `C` について
  `held_ρ(τ, C) = Σ_{s ∈ C} μ_τ(s) + β(C)` である。計数下の類に限るのは、D34 が `held` を計数下の類に
  しか定めないからである。
- **(c)** その本体の関数のパラメータ・capture のすべての unit が所有される (D14) とき、どの類でも
  `β(C) = 0` であり、(b) は `held_ρ(τ, C) = Σ_{s ∈ C} μ_τ(s)` になる。グローバル初期化子の `init` の
  本体もこれに当たる (D1 より `init` はパラメータも capture も持たない)。
- **(d)** パラメータ・capture の leaf `(p, λ)` の ρ-終端は `(p, λ)` 自身である。したがって 1 つの
  別名類はパラメータ・capture の leaf を高々 1 つ含み、`β(C) = 1` である類 `C` は、その借用する
  unit の下の leaf をスロットとして含む。
- **(e)** D10 の生成の表の各行が参照を作る位置のスロットは、それが属する別名類の ρ-終端である。
- **(f)** 計数下の別名類の ρ-終端は、パラメータ・capture の leaf か、D10 の生成の表の各行が参照を作る
  位置のスロットかのいずれかである。

**(c) の前提が成り立つ範囲。** この文書が扱うのは `insert_rc` の出力と
`split_rc_units` の出力であり、前者については `L15` (e) が、後者については `L15` (e) と `L26` (a) が
すべての unit が所有されることを与える。よってこの文書の中では (b) は (c) の形で読める。`β` つきの
(b) が要るのは、借用する unit を持つ本体 -- `borrow_ify` の出力 -- についてである。

**`held` の読みは A19 (i) の本文が固定している。** A19 (i) は、借用するパラメータを持つ借用版に
ついて「`g` の類も `f_borrow` の類も開始値 1 を持つ」と書き、`d(C) = held(C) - [C の ρ-終端が借用する
(D14) leaf ならば 1]` と置く。`β` はこの角括弧である。

**証明**

<1>1. D9 の移動の表の 6 行の移動先のスロットの変数は、`collect_bindings` が次の 6 通りの束縛を
      入れる変数である。
  BY D9, D20, CODE src/rc_ir/ownership.rs: collect_bindings
  `Let(x, Var(y), k)` の `x` は `Binding::Move(y)`、unbox 容器の `Destructure` の名前付きフィールドの
  変数は `Binding::Field(container, idx)`、unbox union の変位アームの payload は
  `Binding::Payload(scrut, Some(tag))`、catch-all アームの payload は `Binding::Payload(scrut, None)`、
  `Llvm` の素通し leaf の結果 `x` は `Binding::Llvm(gen, args, ty)`、`Match` のアーム本体の `Ret(x)` が
  値を渡す先の `Match` の束縛変数は `Binding::Join(arm_results)` である。

<1>2. これらの束縛のもとで、移動先のスロット `s'` の ρ-歩みの次の位置 (D6) は移動元の位置である。
      移動元の変数が局所名であるときそれはスロットであり、局所名でないときは記号の位置である。
  BY <1>1, D6, D9, D17, CODE src/rc_ir/ownership.rs: origin_inner
  `Binding::Move(y)` の腕は `origin(y, path)` を返し、D9 の値の水準の行より `x` の値は `y` の値な
  ので移動元は `(y, λ)` である。`Binding::Field` の unbox の枝は `origin(container, [idx] ++ path)` を
  返し、フィールド変数の値は容器のそのフィールドなので移動元は `(container, [idx] ++ λ)` である。
  `Binding::Payload` の `None` の枝は `origin(scrut, path)` を返し、catch-all の payload の値は
  scrutinee の値そのものなので移動元は `(scrut, λ)` である。`Binding::Payload` の `Some(tag)` かつ
  scrutinee が unbox の枝は `origin(scrut, [tag] ++ path)` を返し、payload の値は scrutinee の活性変位の
  payload なので移動元は `(scrut, [tag] ++ λ)` である。`Binding::Llvm` の単一 `Arg(j, σ')` の枝は
  `origin(args[j], σ')` を返し、D17 の `Binding::Llvm` の行より対応先は `(args[j], σ')` であって、
  結果のその leaf の値はオペランド `j` のその leaf の値である。`Binding::Join` の腕については D17 が
  「`Binding::Join` の辺は、その活性化が選んだアームの結果へ辿る」と定め、移動元はそのアーム本体の
  `Ret(x)` の `(x, λ)` である。

<1>3. (a)。
  BY <1>2, D6, D9, D26, D33
  移動元がスロット `s` であるとき、`s'` の ρ-歩みは 1 歩で `s` に着き、以後は `s` の ρ-歩みそのもの
  である。よって 2 つの ρ-終端は等しい。別名類は ρ-終端が等しいスロットの集まりなので (D33)、`s` と
  `s'` は同じ類に属する。
  移動元が記号の位置 `(g, λ)` であるとき、D6 より別名類の歩みは記号の位置で終わるので `s'` の ρ-終端は
  `(g, λ)` である。D9 の値の水準の行より移動先の値は移動元の値なので `s'` が指すオブジェクトは
  `obj(g, λ)` であり、D6 よりそれは funptr かグローバル状態のオブジェクトである。D26 より
  グローバル状態のオブジェクトは計数下ではなく、funptr の型は `is_fully_unboxed` が真で boxed leaf を
  持たない (D4 の第 1 規則) ので記号の位置を作らない。よって `s'` が属する類は計数下ではない。

<1>4. D34 の表は 6 行を持つ。最初の 3 行は `held_ρ(・, C)` に開始値 1 を
      与え、そのうちパラメータ・capture の inhabited (D16) な leaf については、その leaf の unit を
      関数が所有する (D14) か借用する (D14) かを問わず開始値 1 が与えられる。残る 3 行は
      `(v, λ) ∈ C` である `λ` を `π` の下に持つ `Retain(v, π)` につき `+1`、同じ形の `Release(v, π)`
      につき `-1`、`C` のスロットの D9 の消費につき `-1` である。移動の行は無い。
  BY D34

<1>4a. パラメータ・capture の leaf `(p, λ)` の ρ-終端は `(p, λ)` 自身である。したがって 1 つの別名類は
       パラメータ・capture の leaf を高々 1 つ含み、`β(C) = 1` であるのはその leaf の unit を関数が
       借用するときちょうどそのときである。
  BY D33, D13, D17,
     CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: origin_inner
  `VarTable::of` は `func.params` と `func.capture` の各名前に `Binding::Param` を入れる。
  `origin_inner` の `None | Some(Binding::Param) | Some(Binding::Producer)` の腕は
  `Origin::Exactly((var.clone(), path.to_vec()))` を返すので、`(p, λ)` の ρ-歩みは 1 歩も進まずに
  `(p, λ)` で止まる。別名類は ρ-終端が等しいスロットの集まりであり、ρ-終端はスロットごとに 1 つに
  決まるので、パラメータ・capture の 2 つの leaf は別々の類に属する。`β` の定義はその類の ρ-終端に
  ついての条件なので、この 1 つの leaf の unit が決める。

<1>4b. (e)。
  BY D10, D17, A3, D33,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin_inner
  D10 の生成の表は 5 行を持つ。`App(callee, args)` と `Closure(f, caps)` の結果は `collect_bindings` が
  `Binding::Producer` を入れ、`origin_inner` の `Producer` の腕は `here()` を返す。boxed 容器の
  `Destructure` の名前付きフィールドは `Binding::Field` の容器が boxed の枝、boxed union の変位アームの
  payload は `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝で、どちらも `here()` を返す。
  `Llvm` の結果のうち `result_prov` の宣言が単一の `Arg` でない leaf は `Binding::Llvm` の腕の
  `as_arg_projection` が `None` を返す枝に入り、A3 よりこのコミットの宣言は単一の `Fresh`・単一の
  `Unknown`・空集合のいずれかなので、D17 の第 2 項より鎖はそこで止まる。いずれの行でも `origin` を
  呼ばずに自分自身を答えるので、ρ-歩みはその位置で終わり、そのスロットは自分が属する類の ρ-終端で
  ある。

<1>4c. (f)。
  BY D6, D10, D17, A3, D33, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: as_arg_projection
  ρ-歩みが止まるのは `origin_inner` が `origin` を呼ばずに `here()` を答える腕であり、それは
  束縛の無い腕、`Binding::Param`、`Binding::Producer`、`Binding::Field` の容器が boxed の枝、
  `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝、`Binding::Llvm` の
  `as_arg_projection` が `None` を返す枝 (A3 よりこのコミットの宣言は単一の `Fresh`・単一の
  `Unknown`・空集合のいずれかなので、D17 の第 2 項よりそこで止まる) の 6 つである。束縛の無い腕が
  答える位置は D6 の記号の位置であってスロットではなく、D6 よりそれを終端とする類は計数下の類の
  範囲の外である。残る 5 つのうち `Binding::Param` はパラメータ・capture の leaf であり、
  `Producer` は `App` と `Closure` の結果、`Field` の boxed の枝は boxed 容器の `Destructure` の
  名前付きフィールド、`Payload` の `Some(_)` かつ boxed の枝は boxed union の変位アームの payload、
  `Llvm` の止まる枝は素通しでない `Llvm` の結果 leaf であって、この 4 つが D10 の生成の表の 5 行が
  参照を作る位置である。

<1>5. <1>4 の開始値を与える 3 行のうち、借用する (D14) unit の下のパラメータ・capture の leaf に
      ついての分を除いたものは、`DEF 割り当て` の第 1 行と第 2 行と同じ事象であり、両辺を同じだけ
      動かす。借用する unit の下のパラメータ・capture の leaf の開始値 1 には、`DEF 割り当て` に
      対応する行が無いので、左辺だけが 1 増える。
  BY <1>4, <1>4a, <1>4b, D8, D9, D10, D14, DEF 割り当て
  <1>4 の 3 行は `C` の ρ-終端についての条件で開始値を置く。<1>4a と <1>4b より、パラメータ・capture の
  leaf と D10 の生成が作るスロットはいずれも自分が属する類の ρ-終端なので、その位置に置かれる開始値は
  そのスロットを含む類のものである。よって `DEF 割り当て` の第 1 行・第 2 行が同じ位置で `μ` を
  `+1` するのと同じ事象である。
  `DEF 割り当て` の第 1 行は「**D10 の初期値**: 所有する各パラメータ・capture の inhabited な各 leaf
  `(p, λ)` について `+1`」であり、D10 の初期値も「所有する (D14) パラメータ・capture の unit の下の
  inhabited な各 leaf につき 1 つ。借用する unit の下の leaf は入れない」と述べる。よって所有する
  leaf については両辺が 1 ずつ増え、借用する leaf については左辺だけが 1 増える。D10 の生成の事象に
  ついては `DEF 割り当て` の第 2 行が同じ leaf について `+1` する。参照を作る事象が D10 の初期値と
  D10 の生成の表で尽きることは D9 の末尾の段落が述べる。

<1>5a. `Retain`・`Release`・D9 の消費の 3 種は、`DEF 割り当て` の第 3・第 4・第 5 行が <1>4 の残る
       3 行と同じ leaf について同じ向きに 1 だけ動かす。
  BY <1>4, D8, D10, DEF 割り当て
  どちらの量も D8 の意味の参照を `C` について数えたものである。`DEF 割り当て` の第 3 行は
  `Retain(v, π)` の `π` の下の inhabited な各 leaf について `+1`、第 4 行は `Release(v, π)` の同じ
  leaf について `-1`、第 5 行は D9 の消費される inhabited な各 leaf について `-1` であり、<1>4 の
  残る 3 行と同じ leaf を同じ向きに 1 だけ動かす。

<1>6. 計数下の別名類 `C` について、移動はどちらの辺も動かさない。
  BY <1>3, <1>4, D26, DEF 割り当て
  <1>4 より `held_ρ(・, C)` に移動の行は無い。右辺について、`DEF 割り当て` の第 6 行は移動元の `μ` を
  `-1`、移動先の `μ` を `+1` する。移動元がスロットであるとき、<1>3 より 2 つのスロットは同じ類に
  属するので、両方が `C` に入るか両方が入らないかであり、前者では `-1` と `+1` が相殺し、後者では
  和に現れない。移動元が記号の位置であるときは `-1` を受ける項が無いが、<1>3 より移動先のスロットが
  属する類は計数下ではないので、計数下の `C` の和には `+1` も現れない。

<1>7. QED
  BY <1>3, <1>4, <1>4a, <1>4b, <1>4c, <1>5, <1>5a, <1>6, D1, D9, D14, DEF 割り当て
  (a) は <1>3 である。(b) について、`C` を計数下の別名類とする。両辺は活性化の最初の事象が起きる前に
  0 である -- `DEF 割り当て` は
  `μ` の初期値を 0 と定め、`held_ρ(・, C)` の開始値 1 は <1>4 の 3 行の事象が入れるものだからである。
  D9 の末尾の段落より両辺を動かす事象は
  `DEF 割り当て` の 6 種と、<1>5 が挙げる「借用する unit の下のパラメータ・capture の leaf の
  開始値」で尽きる。<1>5・<1>5a・<1>6 が各事象について 2 つの増減の一致を与え、ずれるのは最後の
  1 種だけである。<1>4a よりその事象は `β(C) = 1` である類についてちょうど 1 回、`β(C) = 0` である
  類については 1 回も起きない。事象の個数についての帰納で (b) の等式が成り立つ。
  (c) は `β` の定義である -- すべての unit が所有されるとき、どの類の ρ-終端も借用する unit の下の
  leaf ではないので `β(C) = 0` である。グローバル初期化子の `init` は D1 よりパラメータも capture も
  持たないので、その本体の類の ρ-終端はパラメータ・capture の leaf ではない。(d) は <1>4a、(e) は
  <1>4b、(f) は <1>4c である。

### 10.2 `L14` (`live_before` は自由変数と `live_after` の和である)

**言明**。`insert_rc` が骨格節点 `m` を `live_after = A(m)` の下で書き換えるとき、次の 2 つが成り立つ。

- **(a)** `Λ(m) = free_locals(m) ∪ A(m)` である (`CODE src/rc_ir/rc_insert.rs: free_locals`)。
- **(b)** `A(m)` の各名前は、`m` の部分木の外にある位置で使われる名前である。したがって `m` の部分木の
  中で束縛される名前は `A(m)` に入らない。

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

<1>3. `A(m)` の各名前は、`m` の部分木の外にある位置で使われる名前である。したがって `m` の部分木の中で
      束縛される名前は `A(m)` に入らない。
  <2>1. 骨格節点を書き換える `insert_into_expr` の呼び出しは、本体の根についてのものか、ある骨格節点
        `m_0` の継続についてのものか、ある `Let(x, Match(scrut, arms), cont)` のアーム本体についての
        ものかの 3 通りである。
    BY D2, CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `insert_into_expr` を呼ぶのは `insert_rc` と `insert_into_func` (根) と、`insert_into_operation_let`・
    `insert_into_destructure`・`insert_into_eval`・`insert_into_match` の継続についての呼び出しと、
    `insert_into_match` のアーム本体についての呼び出しである。
  <2>2. `insert_into_expr(n, L)` が返す集合の各名前は、`n` の部分木の中で使われる名前か `L` の元で
        ある。
    BY A15, A25, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: insert_if_local
    `n` の部分木の節点数についての帰納。`RcExpr::Ret(x)` の腕は `L` に `x` を足したものを返す。残る
    4 種の腕はいずれも、継続 (と `Match` ではアーム本体) についての呼び出しが返した集合から名前を
    取り除き、`n` 自身が名指す名前 -- `rhs_operands` が挙げるオペランド、`Destructure` の容器、
    `Eval` の変数、`Match` の scrutinee と各アームの `arm_free_locals` -- を足したものを返す。
    足される名前はいずれも `n` の部分木の中で使われる。A25 より骨格は `Retain`/`Release` を含まないので
    その腕は通らない。
  <2>3. 根についての呼び出しでは前半が成り立つ。
    BY <1>2
    `A(根) = ∅` なので量化する名前が無い。
  <2>4. `m` が `m_0` の継続であるとき、`A(m_0)` について前半が成り立つならば `A(m)` についても
        成り立つ。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    4 つの関数はいずれも `self.insert_into_expr(cont, live_after)` を呼ぶので `A(m) = A(m_0)` である。
    `m` の部分木は `m_0` の部分木に含まれるので、`m_0` の部分木の外にある位置は `m` の部分木の外にも
    ある。
  <2>5. `m` が `m_0 = Let(x, Match(scrut, arms), cont)` のアーム本体であるとき、`A(m_0)` について
        前半が成り立つならば `A(m)` についても成り立つ。
    BY <2>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `insert_into_match` は `live_cont = insert_into_expr(cont, live_after)` を取り、
    `live_after_match = live_cont \ {x}` の下でアーム本体を書き換えるので `A(m) ⊆ live_cont` である。
    <2>2 より `live_cont` の各名前は `cont` の部分木の中で使われるか `A(m_0)` の元である。`cont` の
    部分木はアーム本体の部分木と交わらないので前者は `m` の部分木の外にあり、後者は仮定より
    `m_0` の部分木の外、したがって `m` の部分木の外にある位置で使われる。
  <2>6. QED
    BY <2>1, <2>3, <2>4, <2>5, A6, A11
    <2>1 の 3 通りについて、`insert_into_expr` の呼び出しの入れ子の深さについての帰納で前半が
    成り立つ -- 基底は <2>3、段は <2>4 と <2>5 である。後半は前半から出る。`m` の部分木の中で
    束縛される名前 `y` を `m` の部分木の外の位置が使うとすると、A11 よりその使用はその位置で
    スコープに入っている束縛に解決するが、A6 より `y` を束縛するのはその 1 つだけであり、その
    スコープ (D2) は `m` の部分木の中に収まる -- 矛盾である。よって `y` は前半の意味で使われず、
    `A(m)` に入らない。

<1>4. ASSUME NEW 骨格節点 `m`、および (帰納法の仮定) `m` の真の部分木である各骨格節点について言明が
              成り立つこと
      PROVE  `m` について言明が成り立つ
  <2>1. CASE `m = Ret(x)`。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この腕は `live = live_after.clone()` に `insert_if_local(&mut live, &x.name)` で `x` を足したものを
    返す。<1>1 より `free_locals(Ret(x))` は `x` が局所名なら `{x}`、そうでなければ空である。
  <2>2. CASE `m = Let(x, rhs, cont)` で `rhs` が `Match` でない。
    BY 帰納法の仮定, A6, A11, D2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: insert_if_local,
       <1>1, <1>3
    この関数は `insert_into_expr(cont, live_after)` を呼び、返った `live_cont` から `x` を除き、
    `rhs_operands(rhs)` の各オペランドの局所名を足したものを返す。帰納法の仮定より
    `live_cont = free_locals(cont) ∪ A(m)`。`rhs_operands` が挙げるオペランドは `rhs` が参照する変数の
    すべてである -- `Var` は被移動変数、`App` は callee と全引数、`Closure` は全 capture、`Llvm` は
    全オペランドを挙げ、`Match` はこの関数に来ない。よって `Λ(m)` は
    `((free_locals(cont) ∪ A(m)) \ {x}) ∪ ops` であり、<1>3 より `x ∉ A(m)` なので
    `(free_locals(cont) \ {x}) ∪ ops ∪ A(m)` に等しい。<1>1 より `free_locals(m)` は `m` の部分木が
    参照する局所名 `refs = ops ∪ refs(cont)` から、束縛する局所名 `bound = {x} ∪ bound(cont)` を
    落としたものである。A11 より `ops` の各使用はその位置でスコープに入っている束縛に解決し、A6 より
    その名前を束縛するものはプログラム全体で 1 つなので、D2 のスコープの規則より `ops` は `bound` と
    交わらない。よって `free_locals(m) = ops ∪ (free_locals(cont) \ {x})` であり、上の式の前 2 項が
    それである。
  <2>3. CASE `m = Destructure(container, fields, _, cont)`。
    BY 帰納法の仮定, A6, A11, D2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>3
    この関数は `live_cont` から各フィールド変数を除き、`container` の局所名を足したものを返す。
    フィールド変数は <1>3 より `A(m)` に入らない。<2>2 と同じく A6・A11・D2 のスコープの規則より
    `container` は `m` の部分木が束縛する名前ではないので、<1>1 の `free_locals(m)` は
    `free_locals(cont)` からフィールド変数を落として `container` を足したものである。
  <2>4. CASE `m = Eval(x, cont)`。
    BY 帰納法の仮定, A6, A11, D2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この関数は `live_cont` に `x` の局所名を足したものを返す。`Eval` は名前を束縛しないので `m` の
    部分木が束縛する名前は `cont` の部分木が束縛するものであり、<2>2 と同じく A6・A11・D2 のスコープの
    規則より `x` はそこに入らない。よって <1>1 の `free_locals(m)` は `free_locals(cont)` に `x` を
    足したものである。
  <2>5. CASE `m = Let(x, Match(scrut, arms), cont)`。
    BY 帰納法の仮定, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
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
    BY A25
    A25 より骨格はこの 2 種を含まないので、この場合は起きない。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, D2, A15
    場合が尽きることは `RcExpr` が 6 個の構成子を持つこと (D2) と、`insert_into_expr_inner` の
    `match` がその 6 個をこの 6 つの腕で覆うことによる。A15 より `insert_into_expr` は
    `insert_into_expr_inner` をちょうど 1 回呼ぶ。<2>2 から <2>5 の各場合で帰納法の仮定を当てるのは
    `cont` とアーム本体 -- どれも `m` の真の部分木 -- についてだけであり、<2>1 はそれを使わない。

<1>5. QED
  BY <1>3, <1>4, D2
  (a) について、D2 より骨格は有限の木なので、部分木の節点数についての整礎帰納が使える。<1>4 がその段で
  あり、それをすべての骨格節点に当てると (a) が出る。(b) は <1>3 である。

### 10.3 `L15` (`Λ` と `insert_rc` の出力についての 5 つの性質)

**言明**。骨格節点 `m` について次の 5 つが成り立つ。

- **(a)** `Λ(m)` の各名前は、`ρ` の上で `m` の検査点に至るまでに値を得ている。
- **(b)** `m` の核節点が名指す局所変数は、すべて `Λ(m)` に入る。「名指す」とは、`Ret(x)` の `x`、
  `Let` の右辺の各変数 (`Match` の scrutinee を含む)、`Destructure` の容器、`Eval` の変数を指す。
- **(c)** `needs_rc(v)` が偽の変数 `v` はスロットを持たない。
- **(d)** 局所でない名前は `vars.bindings` に束縛を持たない。したがって局所でない名前 `g` と `ty(g)` の
  inhabited な boxed leaf `λ` の対は、D6 の記号の位置であってスロットではない。
- **(e)** `insert_rc` の出力のすべての関数の `borrowed_units` は空であり、D14 よりそのすべての
  パラメータ・capture の unit はその関数が所有する。

**証明**

<1>1. (a)。
  BY L14, A11
  `L14` (a) より `Λ(m) = free_locals(m) ∪ A(m)`。`free_locals(m)` の各名前は `m` の部分木が参照して束縛
  しない名前なので、A11 よりその束縛は `m` を囲むスコープに在り、`m` の検査点までに値を得ている。
  `L14` (b) より `A(m)` の各名前は `m` の部分木の外にある位置で使われる名前であり、同じ理由でその束縛は
  `m` を囲むスコープに在る。

<1>2. (b)。
  BY L14, A6, A11, D2, CODE src/rc_ir/rc_insert.rs: free_locals,
     CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound
  核節点が名指す変数は、`collect_referenced_and_bound` が `refs` に入れる名前のうち、`m` 自身の
  節点が挙げるものである。`free_locals(m)` は `refs` から `m` の**部分木**が束縛する名前 `bound` を
  落とすので、名指す名前が `bound` に入らないことが要る。A11 より核節点の変数の使用はその位置で
  スコープに入っている束縛に解決し、A6 より同じ名前を束縛するものはプログラム全体で 1 つだけである。
  D2 のスコープの規則より、`m` 自身の束縛 (`Let` の `x`、`Destructure` のフィールド変数、`Match` の
  payload) のスコープは `m` の継続かアーム本体の部分木であり、`m` より深い節点の束縛のスコープは
  さらにその中である。どれも `m` の核節点の位置を含まないので、名指す名前は `bound` に入らない。
  よって `free_locals(m)` に入り、`L14` (a) より `Λ(m)` に入る。

<1>3. (c)。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, D4, D6
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` であり、D4 の第 1 規則より `is_fully_unboxed` が
  真の型は boxed leaf を持たない。D6 よりスロットは boxed leaf についてのみ在る。

<1>4. (d)。
  BY D6, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: VarTable::body_only,
     CODE src/rc_ir/lower.rs: Lowerer::fresh_var, CODE src/rc_ir/lower.rs:
     Lowerer::lower_lambda_as_function, CODE src/ast/name.rs: FullName::is_local,
     CODE src/ast/name.rs: FullName::local
  `vars.bindings` に鍵が入るのは 2 か所である -- `VarTable::of` が入れる関数のパラメータと capture の
  名前と、`collect_bindings` が入れる `Let`・`Destructure`・`Match` の束縛変数の名前である
  (グローバル初期化子については `VarTable::body_only` が後者だけを入れる)。lowering は
  この両方を `Lowerer::fresh_var` で作り (パラメータと capture は `lower_lambda_as_function` が、
  残りは各 `lower_*` が)、`fresh_var` は名前を `FullName::local(...)` で組む。`FullName::local` が作る
  名前の名前空間は空であり、`is_local` は名前空間が空かを答えるので、束縛を持つ名前は局所名である。
  対偶より局所でない名前は `vars.bindings` に束縛を持たない。D6 の「値を得る 3 つの形」の 3 つ目が
  この名前であり、D6 よりその対は記号の位置であってスロットではない。

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

<1>3. <1>1 の 13 個のうち `result_prov` を override するのは
      `InlineLLVMArrayCopyCapacityBoundsUnchecked`、`InlineLLVMStructGetBody`、
      `InlineLLVMUnionAsBody` の 3 個である。
  BY CODE src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov
  残る 10 個は `result_locality` を override するが `result_prov` は override しない。

<1>3a. `InlineLLVMArrayCopyCapacityBoundsUnchecked` の `result_prov` は、結果の各 boxed leaf に
       単一の `Fresh` を置く。単一の `Fresh` は単一の `Arg(i, σ)` ではない。
  BY CODE src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov,
     CODE src/rc_ir/provenance.rs: Provenance::uniform, CODE src/rc_ir/provenance.rs: sole_origin,
     CODE src/rc_ir/provenance.rs: LeafOrigin
  その `result_prov` は `Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)` を返す。
  `uniform` は `LeafMap::uniform(ty, type_env, sole_origin(src))` であり、`sole_origin` は 1 元の
  集合を作る。`LeafOrigin` の構成子は `Arg`・`Fresh`・`Unknown` であり、`Fresh` は `Arg` ではない。

<1>4. 残る 2 個 -- `InlineLLVMStructGetBody` と `InlineLLVMUnionAsBody` -- は、`borrows_operand(i)` が
      真であるとき結果の型が `is_fully_unboxed` である。
  BY A12, CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_operand
  前者の `borrows_operand` は
  `i == 0 && Self::borrows_container(&arg_tys[0].field_types(type_env)[self.field_idx], type_env)` で
  あり、`borrows_container(field_ty, ・)` は `field_ty.is_fully_unboxed(type_env)` である。A12 の
  `Llvm` 節点の型についての節は「`InlineLLVMStructGetBody` の `ty(x)` は `ty(args[0])` の第
  `field_idx` フィールドの型であり、`InlineLLVMUnionAsBody` の `ty(x)` は `ty(args[0])` の第
  `field_idx` 変位の payload の型である」と述べるので、この `field_ty` はこの op の結果の型である。
  後者も同じ形で、`borrows_union` は payload の型 -- A12 よりこの op の結果の型 -- について
  `is_fully_unboxed` を問う。どちらも `i == 0` 以外の `i` には偽を返す。

<1>5. 結果の型が `is_fully_unboxed` であるとき、宣言はどの leaf にも何も置かない。
  BY D4, CODE src/rc_ir/provenance.rs: Provenance
  D4 の第 1 規則より `is_fully_unboxed` が真の型は boxed leaf を持たない。`Provenance::build_shape` と
  `Provenance::uniform` はどちらも型の boxed leaf の上に `LeafMap` を作るので、leaf を持たない型では
  空である。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5
  `borrows_operand(i)` が真になるのは <1>1 の 13 個のいずれかであり、そのうち 10 個は既定の
  `result_prov` を持ち `Arg` を宣言しない (<1>2、<1>3)。`InlineLLVMArrayCopyCapacityBoundsUnchecked`
  は結果の各 leaf に単一の `Fresh` を置くので `Arg` を宣言しない (<1>3a)。残る 2 個は、
  `borrows_operand(i)` が真であるとき結果に leaf が無い (<1>4、<1>5) ので、やはり `Arg(i, σ)` を
  宣言する leaf を持たない。

**この補題が要る理由。** D9 の消費の表の `Llvm` の行は `borrows_operand(i)` が偽のオペランドだけを
挙げるが、移動の表の `Llvm` の行 (素通し leaf) はその条件を持たない。両方が同時に成り立つ op が在ると、
1 つの参照が結果へ移りながら呼び出し元にも残ることになり、`insert_rc` が置く「借用オペランドの最後の
使用の後の `Release`」がその参照を二重に処分する。`L16` はその形が無いことを言う。

### 10.4a `L16a` (終端の `Ret` を書き換える呼び出しの `live_after` は空である)

**言明**。関数本体・グローバル初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合で
あり、その骨格節点の前置 `Retain` 鎖は空である。

**証明**

<1>1. 本体の根を書き換える呼び出しの `live_after` は空集合である。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func, CODE src/rc_ir/rc_insert.rs: insert_rc
  関数については `insert_into_func` が `self.insert_into_expr(func.body, &Set::default())` を呼び、
  グローバル初期化子については `insert_rc` が `inserter.insert_into_expr(glob.init, &Set::default())` を
  呼ぶ。

<1>2. 骨格節点 `m` が `Ret` でないとき、`m` の継続を書き換える呼び出しの `live_after` は、`m` を
      書き換える呼び出しの `live_after` に等しい。
  BY A25, D2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  D2 より `RcExpr` は 6 種であり、A25 より骨格は `Retain` と `Release` を含まないので、`Ret` でない
  骨格節点は `Let`(`Match` でない右辺)、`Let`(`Match` の右辺)、`Destructure`、`Eval` の 4 種である。
  4 つの関数はいずれも継続について `self.insert_into_expr(cont, live_after)` を呼ぶ。

<1>3. 関数本体・初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合である。
  BY <1>1, <1>2, D2, D3
  D2 より `Ret` 以外の 5 種はちょうど 1 つの継続を持つので、関数本体・初期化子の終端の `Ret` (D3) は
  本体の根から継続だけを辿って着く節点である。その継続の鎖の長さについての帰納で、鎖の各節点を
  書き換える呼び出しの `live_after` は根のもの、すなわち空集合である。

<1>4. QED
  BY <1>3, CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕
  `Ret` の腕が前置 `Retain` 鎖を作るのは `retain_if_live(&x, live_after, ret)` の 1 か所だけであり、
  その条件 `live.contains(&var.name)` は空集合について偽である。

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
- **(c)** 遷移の中の各節点 `n` とスロット `s` について、`n` が `μ(s)` を下げる回数を `k_n(s)` とすると、
  `n` の入口で `μ(s) ≥ k_n(s)` である。

**証明**

<1>0. 局所名でない変数はスロットを持たない。したがって以下の各場合で `μ` を勘定するのは局所名の変数に
      ついてだけでよく、局所名でない変数を名指す節点はどのスロットの `μ` も動かさず、(b) の「読む値の
      各スロット」「触れる各スロット」の量化もその変数については空である。
  BY L15, DEF 割り当て
  `L15` (d) が局所でない名前についてこれを与える。`DEF 割り当て` の 6 種はいずれもスロットについて値を
  動かすので、スロットを持たない変数についてはどの行も発火しない。

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
  <2>4. 出力の各節点は、`insert_into_func` が積む `unused` の解放鎖に属するか、いずれかの骨格節点の
        塊 (10.1 節) に属する。
    BY A15, A25, CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `insert_rc` が節点を作るのは、`insert_into_func` の `build_releases(unused, body)` と、
    `insert_into_expr_inner` が骨格節点 1 つを受け取って走る 5 つの腕だけである (A15 より
    `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼び、A25 より
    `Retain`/`Release` の腕は通らない)。その 5 つの腕が作る節点は、10.1 節が挙げる 4 つの部分 --
    前置 `Retain` 鎖、核節点、アームの頭の `Release` 鎖、後置 `Release` 鎖 -- で尽きる。継続と
    アーム本体の節点は、それぞれについての `insert_into_expr` の呼び出しが作るものである。
  <2>4a. QED
    BY D2, D3, A25, L16a, <2>1, <2>2, <2>3, <2>4,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕
    D2 より骨格節点は 6 種であり、A25 より `Retain` と `Release` は骨格に無い。残る 4 種について、
    塊の節点が `ρ` の上でどの遷移に入るかを見る。
    `Let`(非 `Match`)、`Destructure`、`Eval` の塊はアームの頭の `Release` 鎖を持たないので、
    前置 `Retain` 鎖・核節点・後置 `Release` 鎖の 3 つであり、<2>1 よりそれがそのまま (T1) の遷移で
    ある。
    `Let(x, Match(scrut, arms), cont)` の塊は 2 つに分かれる。前置 `Retain` 鎖・核節点・`ρ` が選んだ
    アーム `j` の頭の `Release` 鎖は <2>2 より (T2) の遷移をなし、後置 `Release` 鎖は <2>3 より
    アーム `j` の本体の終端の `Ret` から始まる (T3) の遷移の末尾に入る。D3 より `ρ` は選ばれなかった
    アームの頭の `Release` 鎖を訪れない。
    `Ret` は木の中の位置で 2 つに分かれる。アーム本体の終端の `Ret` の塊は前置 `Retain` 鎖と核節点で
    あり (`Ret` の腕は `build_releases` を呼ばないので後置 `Release` 鎖が無い)、<2>3 よりその 2 つは
    その `Match` の後置 `Release` 鎖と合わせて (T3) の遷移をなす。関数本体・初期化子の終端の `Ret` は
    `ρ` の最後の骨格節点であって後続の検査点を持たないので遷移を成さないが、`L16a` よりその前置
    `Retain` 鎖は空であり、後置 `Release` 鎖も無いので、その塊は核節点 1 つ -- すなわちその検査点
    1 点 -- である。
    よって `ρ` の上の各節点の入口は、`unused` の解放鎖の中の点か、(T1)・(T2)・(T3) のいずれかの
    遷移の中の点か、関数本体・初期化子の終端の `Ret` の検査点かのいずれかであり、遷移はこの 3 種で
    尽きる。

<1>2. **CASE (T1)** で `m = Let(x, rhs, cont)`、`rhs` は `Match` でない。
  <2>1. `Λ(m) = (Λ(m') \ {x}) ∪ ops` である。ここで `ops` は `rhs_operands(rhs)` が挙げるオペランドの
        局所名の集合であり、`x ∉ ops` である。
    BY A6, A11, D2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: insert_if_local
    `insert_into_operation_let` は `live_cont` (= `Λ(m')`) から `x` を除き、各オペランドの局所名を
    `insert_if_local` で足したものを返す。`x ∉ ops` は A11 と D2 から出る -- `ops` の各名前は `rhs` の
    位置での使用であり、A11 よりその使用はその位置でスコープに入っている束縛に解決する。D2 より
    `Let(x, rhs, k)` が束縛する `x` のスコープは `k` の部分木であって `rhs` の位置を含まないので、
    `ops` の使用が `x` の束縛に解決することはない。A6 より `x` という名前を束縛するものはプログラム
    全体で 1 つだけなので、`ops` に `x` は現れない。
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
        `μ` を 1 下げる。`Borrow` の出現は `μ` を変えない。`ops` に入らないオペランド -- 局所名でない
        もの -- については、<1>0 より動くスロットが無い。
    BY <1>0, D9, L15, L16
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
    BY <1>0, <2>2, <2>3, <2>4, <2>5
    仮定より遷移の前は `μ(v, λ) = 1` (`v ∈ ops ⊆ Λ(m)`) であり `μ(x, μ') = 0` (`x` はまだ値を得て
    いない)。`v` について、前置 `Retain` が `n_v - [v ∉ Λ(m') ∧ 最後が Own]` 回上げ、核節点が `n_v`
    回下げ、後置 `Release` が `[v ∉ Λ(m') ∧ 最後が Borrow]` 回下げるので、
    `1 - [v ∉ Λ(m') ∧ 最後が Own] - [v ∉ Λ(m') ∧ 最後が Borrow] = 1 - [v ∉ Λ(m')] = [v ∈ Λ(m')]`。
    `x` について、核節点が 1 上げ、後置 `Release` が `[x ∉ Λ(m')]` 回下げるので `[x ∈ Λ(m')]`。
    <2>1 より `Λ(m)` と `Λ(m')` の差は `ops` と `{x}` の上にしか無いので、他の名前は両方に入るか
    両方に入らないかであり、`μ` も変わらない。
  <2>7. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <1>0, <2>2, <2>3, <2>4, <2>5, <2>6, A6
    前置 `Retain` 鎖の中では `μ` は上がるだけなので、`μ(v, λ) ≥ 1` (`v ∈ ops`) が保たれ、鎖の各
    `Retain` が触れるスロットは `μ ≥ 1` である。核節点の入口では `μ(v, λ) = 1 + (前置の回数) ≥ n_v`
    であり、核節点が読む値 (D7 の表: `Llvm` の各オペランド、`App` の callee と各引数、`Closure` の
    各 capture、`Destructure` の容器、`Eval` の変数) のうち `ops` の名前のものは `μ ≥ 1` であり、
    残るもの -- 局所名でないもの -- は <1>0 よりスロットを持たないので量化から外れる。核節点の後、
    後置 `Release` 鎖の各名前は相異なる -- `releases_after`
    に同じ名前が 2 度入ることはなく (`used_later` が偽になるのは最後の出現だけ)、`x` はオペランドとは
    別の名前である (A6) -- ので、鎖の `i` 番目の `Release` の入口での `μ` は、その名前について
    <2>6 の最終値 `[・ ∈ Λ(m')]` に、まだ実行していない自分自身の分 1 を足したもの以上であり、1 以上で
    ある。それらの `Release` の後の値は <2>6 の最終値であり非負である。
  <2>7a. (c)。
    BY <2>2, <2>3, <2>4, <2>5, <2>6, A6
    核節点は、<2>4 より
    `ops` の各名前 `v` の各スロット `(v, λ)` を `n_v` 回下げ、<2>5 より `x` のスロットは上げるだけ
    なので、`k_核(v, λ) = n_v`、`k_核(x, ・) = 0`、他のスロットについては 0 である。核節点の入口での
    `μ(v, λ)` は、<2>2 より `1 + n_v - [v ∉ Λ(m') ∧ 最後が Own] ≥ n_v` である。後置 `Release` 鎖の
    各節点は path が空列なのでその名前の各スロットを 1 回ずつ下げ、鎖の中に同じ名前は 2 度現れない
    (`releases_after` に同じ名前は 2 度入らず、`x` はオペランドとは別の名前である (A6))。<2>7 より
    その入口の値は 1 以上である。
  <2>8. QED
    BY <2>6, <2>7, <2>7a

<1>3. **CASE (T1)** で `m = Destructure(container, fields, _, cont)`。
  <2>1. `container` が局所名であるとき `Λ(m) = (Λ(m') \ {フィールド変数}) ∪ {container}`、そうでない
        とき `Λ(m) = Λ(m') \ {フィールド変数}` である。前置 `Retain` 鎖は `container` が局所名で
        `container ∈ Λ(m')` のとき `Retain(container, [])` を 1 つ置き、後置 `Release` 鎖は `Λ(m')` に
        入らない各フィールド変数の `Release` を 1 つずつ置く。以下 `container` は局所名とする --
        そうでないとき <1>0 よりこの変数はスロットを持たず、下の勘定はフィールド変数だけについて
        同じ形で走る。
    BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: insert_if_local,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live, L15
    `retain_if_live(&container, &live_cont, node)` は `var.name.is_local()` を要求したうえで
    `live_cont` (= `Λ(m')`) を見る。`live_before` へ `container` を足すのは `insert_if_local` なので、
    局所名でない `container` は `Λ(m)` に入らない。`dead` は
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
  <2>4a. (c)。
    BY <2>1, <2>2, <2>3, <2>4, A6
    前置 `Retain` は `μ` を上げるだけなので `k = 0` である。核節点は、<2>2 と <2>3 より容器の各
    スロットを 1 回だけ下げ、フィールド変数のスロットは上げるだけなので、`k_核 ≤ 1` であり、
    <2>4 より容器のスロットの入口の値は 1 以上である。後置 `Release` 鎖の各節点はその名前の各
    スロットを 1 回ずつ下げ、鎖の名前は相異なるフィールド変数 (A6) なので、<2>4 よりその入口の値は
    1 である。
  <2>5. QED
    BY <2>2, <2>3, <2>4, <2>4a, D5
    容器は boxed か unbox かのいずれかである。

<1>4. **CASE (T1)** で `m = Eval(x, cont)`。
  BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval, D9, D7, L15
  `Λ(m) = Λ(m') ∪ ({x} ∩ 局所名)` である。前置 `Retain` 鎖は空であり、後置 `Release` 鎖は
  `x` が局所名で `x ∉ Λ(m')` かつ `needs_rc(x)` のとき `Release(x, [])` を 1 つ置く。D9 の 2 つの表に `Eval` の行は無いので核節点は
  `μ` を変えない。よって遷移の後 `μ(x, λ) = 1 - [x ∉ Λ(m')] = [x ∈ Λ(m')]` であり、他の名前は変わら
  ない。核節点は `x` を読み (D7)、その入口で `μ(x, λ) = 1` である。`Release` の入口でも 1 である。
  (c) について、核節点は `μ` を下げないので `k = 0` であり、`Release(x, [])` は `x` の各スロットを
  1 回ずつ下げてその入口の値は 1 である。`x` が局所名でないときは <1>0 より `x` はスロットを持たず、
  `Release` も置かれないので、この場合の勘定は空である。

<1>5. **CASE (T2)**。`m = Let(x, Match(scrut, arms), cont)`、`ρ` が選ぶアームを `j` とする。
  <2>0. `scrut` が局所名でないとき、<1>0 より `scrut` はスロットを持たず、この場合の `scrut` の列の
        勘定は空である。`H` は `insert_if_local` が作る live 集合の和なので局所名だけからなり、
        `retain_if_live` は `var.name.is_local()` を要求するので前置 `Retain` は置かれず、
        `DB_j ⊆ H` にも `scrut` は入らない。頭の `Release(scrut, [])` は置かれうるが、動かすスロットが
        無い。以下 `scrut` は局所名とする。
    BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: insert_if_local
  <2>1. `H := live_at_arm_head`、`M := live_after_match`、`U_j := arm_free_locals(arm_j)`、
        `P_j := {payload_j} ∩ free_locals(arm_j.body)` と置くと、`Λ(m) = H ∪ {scrut}`、
        `Λ(m') = U_j ∪ P_j ∪ M` であり、`H = M ∪ (∪_i U_i)` である。
    BY <2>0, L14, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals
    `live_at_arm_head` は `live_after_match` に各アームの `arm_free_locals` を足したものである。
    `L14` (a) より `Λ(m) = free_locals(m) ∪ A(m)` であり、`free_locals(m)` は
    `{scrut} ∪ (∪_i U_i) ∪ (free_locals(cont) \ {x})`、`M = live_cont \ {x}` は
    `(free_locals(cont) ∪ A(m)) \ {x}` なので `Λ(m) = {scrut} ∪ (∪_i U_i) ∪ M = {scrut} ∪ H`。
    アーム本体は `insert_into_expr(arm.body, &live_after_match)` で書き換えられるので `L14` (a) より
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
    BY <2>0, <2>2, <2>4, <2>5, A6, D7
    前置 `Retain` が置かれるのは `scrut ∈ H ⊆ Λ(m)` のときであり、そのときその入口で
    `μ(scrut, λ) = 1` である。核節点は scrutinee を読み (D7)、その入口で
    `μ(scrut, λ) = 1 + [scrut ∈ H] ≥ 1`。頭の `Release` 鎖の名前のうち `scrut` は最大 2 回
    現れる -- `DB_j` に入るときと容器解放のときである。両方が起きるのは `scrut ∈ DB_j` のときであり、
    `DB_j = H \ (U_j ∪ M) ⊆ H` (<2>2) よりそのとき `scrut ∈ H` なので前置 `Retain` が置かれ、
    <2>4 より核節点は scrutinee の `μ` を下げないから、核節点の直後の `μ(scrut, λ)` は 2 である。
    よって 2 つの `Release` の入口の値は 2 と 1 である。片方だけのときは入口の値は 1 以上である。`payload_j` と `DB_j` の他の名前は互いに、また `scrut` とも異なり (A6)、
    それぞれ 1 度しか現れないので、その入口の値は 1 である。<2>4 と <2>5 の最終値はすべて非負である。
  <2>6a. (c)。
    BY <2>2, <2>4, <2>5, <2>6, A6
    前置 `Retain` は `μ` を上げるだけなので `k = 0` である。核節点は、<2>4 では scrutinee の `μ` を
    下げず (下げるのは頭の `Release` 鎖である)、<2>5 では移動によって scrutinee の各スロットを 1 回
    だけ下げるので、どちらでも `k_核 ≤ 1` であり、<2>6 よりその入口の値は `1 + [scrut ∈ H] ≥ 1` で
    ある。payload のスロットは上げるだけである。頭の `Release` 鎖の各節点は path が空列なので
    その名前の各スロットを 1 回ずつ下げ、<2>6 よりその入口の値は 1 以上である -- `scrut` が 2 度
    現れる場合の入口の値は 2 と 1 である。
  <2>7. QED
    BY <2>4, <2>5, <2>6, <2>6a
    `arm_j.tag` は `Some` か `None` のいずれかであり、`Some` の場合は `scrut.ty.is_box` の真偽で
    分かれる。3 つの場合を <2>4 と <2>5 が覆う。

<1>6. **CASE (T3)**。`m` はアーム本体の終端の `Ret(r)`、その `Match` を `Let(x, Match(s, arms), cont)`
      とする。
  <2>1. `Λ(m) = M ∪ ({r} ∩ 局所名)`、`Λ(m') = live_cont` であり `M = live_cont \ {x}` である。
    BY L14, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    アーム本体は `live_after = M` で書き換えられるので `L14` (a) より
    `Λ(m) = free_locals(Ret(r)) ∪ M`。
  <2>2. 前置 `Retain` 鎖は `r ∈ M` のとき `Retain(r, [])` を 1 つ置き、`Match` の核節点と `cont` の
        間の `Release` 鎖は `x ∉ live_cont` のとき `Release(x, [])` を 1 つ置く。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, L15
  <2>3. 核節点 `Ret(r)` は `μ(x, λ)` を 1 上げる。`r` が局所名であるときは併せて `r` の inhabited な
        各 boxed leaf について `μ(r, λ)` を 1 下げ、局所名でないときは下げる先を持たない。
    BY <1>0, D9, A12
    D9 の移動の表の「`Match` のアーム本体の `Ret(x)`」の行。A12 よりアームの結果と `Match` の束縛変数の
    型は等しいので leaf は対応する。`r` が局所名でないときは <1>0 より `(r, λ)` はスロットではないので
    `μ` を下げる項が無い。
  <2>4. QED
    BY <1>0, <2>1, <2>2, <2>3, A6
    A6 より `x ≠ r` であり、`r ∈ M ⟺ r ∈ live_cont` である。`r` が局所名であるとき、遷移の後
    `μ(r, λ) = 1 + [r ∈ M] - 1 = [r ∈ Λ(m')]` である。`r` が局所名でないときは <1>0 より `r` は
    スロットを持たず、<2>1 より `Λ(m) = M` であり、`r` についての勘定は空である。どちらでも
    `μ(x, λ) = 1 - [x ∉ live_cont] = [x ∈ Λ(m')]` である。他の名前は `Λ(m)` と `Λ(m')` の両方に入るか
    両方に入らないかである。遷移の中の各節点については、前置 `Retain` の入口で `μ(r, λ) = 1`
    (置かれるのは `r ∈ M` のときであり、`M` の名前は局所名である)、核節点は D7 の読む構文では
    ないが `r` を名指し、`r` が局所名なら `μ(r, λ) = 1 + [r ∈ M] ≥ 1` である。`Release(x, [])` の
    入口で `μ(x, λ) = 1` である。値はどこでも非負である。
    (c) について、前置 `Retain` は `μ` を下げないので `k = 0`、核節点は <2>3 より `r` が局所名なら
    その各スロットを 1 回ずつ下げてその入口の値は `1 + [r ∈ M] ≥ 1`、局所名でないなら `k_核 = 0` で
    ある。`Release(x, [])` は `x` の各スロットを 1 回ずつ下げてその入口の値は 1 である。

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
    BY A11, L14, D10, D1, CODE src/rc_ir/rc_insert.rs: insert_rc
    D1 より `init` はパラメータも capture も持たないので D10 の初期値は空であり、活性化の開始時の
    `μ` はすべて 0 である。`insert_rc` は `inserter.insert_into_expr(glob.init, &Set::default())` を
    呼ぶので `A(根) = ∅` であり、`L14` (a) より `Λ(根) = free_locals(init)` である。A11 は
    「グローバル初期化子の `init` は自由な局所名を持たない」と述べるので `Λ(根) = ∅` である。
    `insert_rc` はこの本体に `build_releases` を掛けないので、根の前に節点は無い。
  <2>2. CASE 本体が関数の `body` である。
    BY A11, L14, D10, L15, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: build_releases
    `L15` (e) より、すべてのパラメータ・capture の unit は所有される。D10 の初期値は、所有する
    各パラメータ・capture の inhabited な各 leaf の `μ` を 1 にする。`insert_into_func` は
    `insert_into_expr(func.body, &Set::default())` を呼ぶので `A(根) = ∅` であり、`L14` (a) より
    `Λ(根) = free_locals(func.body)` である。A11 は「関数の本体の自由な局所名は、その関数のパラメータと
    capture に限る」と述べるので、`Λ(根)` はパラメータと capture の名前だけからなる。
    `insert_into_func` は `func.body = build_releases(unused, body)` を作り、`unused` は
    `self.needs_rc(p) && !live.contains(&p.name)` を満たすパラメータと capture である。ここで
    `live = Λ(根)` である。`L15` (c) より `needs_rc(p)` が偽のパラメータはスロットを持たない。よって `unused` の解放鎖の後、パラメータ・capture の各スロットの `μ` は
    `[p ∈ Λ(根)]` である。`unused` の名前は相異なるので各 `Release` の入口で `μ = 1` であり、
    パラメータ・capture 以外の名前のスロットは値を得ておらず `μ = 0 = [・ ∈ Λ(根)]` である。
  <2>3. QED
    BY <2>1, <2>2, D23, A6
    D23 より本体は関数の `body` かグローバル初期化子の `init` かのいずれかである。A6 より
    `unused` の名前は相異なる。

<1>2. 各検査点で `μ` は `Λ` の指示関数である。
  BY <1>1, L17
  `ρ` の上の検査点の列についての帰納。基底は <1>1、段は `L17` である。

<1>3. QED
  BY <1>2, L13a, L15 DEF 割り当て
  `L15` (e) より `insert_rc` の出力ではすべての unit が所有されるので、`L13a` (c) が当たり、
  `L13a` (b) は `held_ρ(m, C) = Σ_{(v, λ) ∈ C} μ_m(v, λ)` になる。<1>2 よりこれは
  `Σ_{(v, λ) ∈ C, v ∈ Λ(m)} 1 = Σ_{v ∈ Λ(m)} κ_C(v) = N_ρ(m, C)` である。

### 10.6a `L18a` (終端の `Ret` の前に `Retain` は立たない)

**言明**。関数本体・グローバル初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合であり、
その検査点の前置 `Retain` 鎖は空である。したがって `insert_rc` の出力の各 `Retain` 節点の入口は、
`L17` (a) が挙げる 3 通りのうち、連続する 2 つの検査点の間の遷移の中の点である。

**証明**

<1>1. 関数本体・初期化子の終端の `Ret(x)` を書き換える呼び出しの `live_after` は空集合であり、
      その検査点の前置 `Retain` 鎖は空である。
  BY L16a

<1>3. 本体の根の検査点より前の `Release` 鎖に `Retain` 節点は無い。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: build_releases
  `insert_into_func` はこの位置に `build_releases(unused, body)` だけを置き、`build_releases` は
  `RcExpr::Release` の節点しか作らない。

<1>4. QED
  BY <1>1, <1>3, L17
  `L17` (a) より、出力の各節点の入口は、根の検査点より前の `Release` 鎖の中の点か、連続する 2 つの
  検査点の間の遷移の中の点か、終端の `Ret` の検査点かのいずれかである。<1>3 より第 1 の場所に
  `Retain` 節点は無い。第 3 の場所は 1 点であり、<1>1 より前置 `Retain` 鎖が空なのでその点は核節点
  `Ret` の入口であって `Retain` 節点の入口ではない。よって `Retain` 節点の入口は第 2 の場所にある。

### 10.7 `L19` ((O1))

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、次の 2 つが成り立つ。

- **(a)** `ρ` の上の各時点において、各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その節点の入口で `held_ρ(・, C) ≥ 1` である。
- **(c)** 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った後、各計数下の別名類 `C` について
  `held_ρ(・, C) = 0` である。

**証明**

<1>1. (a)。
  BY L13a, L15, L17, L18 DEF 割り当て
  `L17` (a) より各節点の入口は 3 通りである。`unused` の解放鎖の中では `L18` が `μ ≥ 0` を与え、
  遷移の中では `L17` (b) が与え、終端の `Ret` の検査点では `L18` が `μ` を `Λ` の指示関数と定める
  ので `μ ≥ 0` である。`L15` (e) と `L13a` (c) より `β ≡ 0` なので、`L13a` (b) は
  `held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ)` であり、これは非負の項の和である。

<1>2. (b)。
  BY L13a, L17, L18, L15 DEF 割り当て
  `L17` (b) より、遷移の中の各節点の入口で、その節点が読む値のスロットと `Retain`/`Release` が触れる
  スロットは `μ ≥ 1` である。検査点はその遷移の最初の節点の入口なので同じ主張に含まれる。
  `unused` の解放鎖の中の `Release` については `L18` が `μ = 1` を与える。終端の `Ret` の検査点に
  ついては、`x` が局所名なら `L18` よりその節点が名指す `x` のスロットの `μ` は `[x ∈ Λ]` であり、
  `L15` (b) より `x ∈ Λ` なので 1 であり、局所名でないなら `L15` (d) よりスロットが無い (終端の `Ret`
  は D7 の読む構文ではないので、この場合はいずれにせよ言明の対象外である)。
  そのスロット `(v, λ)` が計数下の類 `C` に属するとき、`L15` (e) と `L13a` (b)(c) と <1>1 より
  `held_ρ(・, C) = Σ_{(w, μ') ∈ C} μ(w, μ') ≥ μ(v, λ) ≥ 1` である (ほかの項は <1>1 の議論より
  非負である)。

<1>3. (c)。
  <2>1. 関数本体・初期化子の終端の `Ret(x)` を書き換える呼び出しの `live_after` は空集合であり、
        その検査点の前置 `Retain` 鎖は空である。
    BY L18a
  <2>2. QED
    BY <2>1, L13a, L15, L18, L14, D9, DEF 割り当て
    <2>1 と `L14` (a) より `Λ(終端の Ret) = free_locals(Ret(x))` であり、これは `x` が局所名なら
    `{x}`、そうでなければ空である。`L18` よりその検査点で `μ` は `Λ` の指示関数である。`x` が局所名の
    とき、`x` 以外のスロットの `μ` は 0 であり、D9 の消費の表の「関数本体の終端の `Ret(x)`」の行より
    核節点は `x` の inhabited な全 boxed leaf の参照を消費するので、その後 `μ(x, λ) = 0` である。
    `x` が局所名でないとき、`L15` (d) より `x` はスロットを持たないので消費は `μ` を動かさず、
    `Λ` が空なのですべてのスロットの `μ` は 0 のままである。どちらでもすべてのスロットで `μ = 0` で
    あり、`L15` (e) と `L13a` (b)(c) より
    `held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ) = 0` である。

<1>4. QED
  BY <1>1, <1>2, <1>3

**(c) が言っているもの。** (c) は D11 の (S-b) を別名類の粒度へ絞ったものである。A19 (ii-a) を活性化の
終わりの 1 点先まで読む読み手は、この点で `held ≥ 0` を要る -- (c) はそれを等式の形で与える。
**A19 (ii-b) はこの点へは延びない。** `held = 0` である一方、走査の `RcExpr::Ret` の腕は
`returns_from_func` が真のとき pending の要素を `needed_retains` に入れるだけで `pending` から
取り除かないので (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)、`bumps ≥ 1` のまま
`held = 0` になる時点が在る。第 12 節の `L38` がその形の `insert_rc` の出力を挙げる。

### 10.8 `L20` (各別名類の開始事象は `ρ` の上に高々 1 つ)

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各計数下の別名類 `C` の
開始事象 (`DEF 開始事象`) は `ρ` の上に高々 1 つである。

**証明**

<1>1. 開始事象は、パラメータ・capture の inhabited (D16) な leaf についての行と、D10 の
      生成の表の各行である。`insert_rc` の出力ではすべての unit が所有される (`L15` (e)) ので、
      前者は D10 の初期値の行そのものである。
  BY DEF 開始事象, D34, D10, D14, L15
  D34 の表で `held_ρ(・, C)` に開始値 1 を与えるのはこの 2 種であり、
  パラメータ・capture の leaf についての行は、その leaf の unit を関数が所有するか借用するかを
  問わず開始値 1 を与える。`L15` (e) より `insert_rc` の出力に借用する unit は無い。D10 は義務集合の
  初期値と生成の表を持ち、それ以外に参照を作る行を持たない。

<1>2. これらの事象が作る参照が属する類の ρ-終端は、その事象の位置のスロット自身である。
  BY <1>1, L13a
  `L13a` (d) がパラメータ・capture の leaf について、`L13a` (e) が D10 の生成の表の各行について、
  そのスロットが自分の属する類の ρ-終端であることを与える。

<1>3. QED
  BY <1>1, <1>2, D6, A6, A11
  <1>2 より、相異なる開始事象は相異なるスロットを ρ-終端とする -- 事象の位置が相異なれば、そこで値を
  得る変数が相異なる (A6) か、同じ変数の相異なる leaf である。よって相異なる開始事象は相異なる類に
  開始値を与える。A11 と D6 より 1 つのスロットが 2 度値を得ることはない。

**`L4` の前提が満たされること。** `L4` は「`ρ` の上で開始事象を 1 つだけ持つ各別名類」について述べる。
`L20` より開始事象は高々 1 つである。開始事象を 1 つも持たない類は `ρ` の上にスロットを持たない
(D6 -- スロットの変数はその実行路の上で値を得ている) ので、勘定の対象にならない。

## 11. (O2) の証明 -- 帳簿は処分に遅れない

`L7` より、(O2) は「`bumps ≥ 1` である各時点で `held ≥ 1 + bumps`」と同値である。この節はそれを示す。
支えるのは、**1 つの別名類のスロットに付く名前の全体 `Ids(C)` が作る木**についての不等式である。
`origin` の `identity` はスロットごとに 1 つの名前を与え、1 つの別名類の中でその名前は
`Binding::Join` が候補を 2 つ以上持つ位置で切り替わる。切り替わりの関係 `Anc` が `Ids(C)` を木にし
(`L21`)、走査の帳簿はその木の各名前に付く。示す不等式は **「1 つの部分木に付いている bump の総和は、
その部分木が保持している参照の個数より真に小さい」**(`L24`) であり、木の根で読むとそれがちょうど
`held ≥ 1 + bumps` になる。

この節は `DEF 割り当て` の `μ` と、本体について 11.1 節の前提 (S) が挙げる 4 つだけを読む。`insert_rc` の
出力がその 4 つを満たすことは `L20a` が第 10 節の `L15`・`L17`・`L18`・`L18a`・`L20` から出す。前提の形で
書くのは、`split_rc_units` の出力についても同じ 4 つを確かめれば同じ結論が出るからである
(第 13 節の `L32`)。

### 11.1 名前の鎖、名前の木、帳簿

本体 `B` と、その 1 つの活性化と実行路 `ρ` と計数下の別名類 `C` を固定する。以下、`C` のスロットを単に
**スロット**と書く。

**DEF `id(s)`**。スロット `s = (v, λ)` について `id(s) := origin(v, λ).identity()` と置く
(`CODE src/rc_ir/ownership.rs: origin`, `CODE src/rc_ir/ownership.rs: Origin`)。

**DEF `Anc(s)`**。D33 の ρ-歩みは、`s` から
ρ-終端まで辿るスロットの列 `s = s_0, s_1, …, s_n` である。`Anc(s) := {id(s_0), id(s_1), …, id(s_n)}`
と置き、**`s` の名前の鎖**と呼ぶ。

**DEF `Ids(C)`、`Sub(id)`**。`Ids(C) := {id(s) : s はスロット}`。`id ∈ Ids(C)` について
`Sub(id) := {id(s) : s はスロットで id ∈ Anc(s)}`。

**DEF `Down`、`Bmp`、`Bsub`**。`ρ` の上の時点 `τ` と `id ∈ Ids(C)` について
`Down_τ(id) := Σ_{s : id ∈ Anc(s)} μ_τ(s)`、
`Bmp_τ(id) := Σ_{p ∈ pending} B(p, ρ)[id]` (D27)、
`Bsub_τ(id) := Σ_{id' ∈ Sub(id)} Bmp_τ(id')` と置く。

**`Anc`・`id`・`Ids`・`Sub` は時点に依らない。** `origin` の答えは鍵ごとに 1 つに決まって memo の
状態に依らず (D13、P2a)、ρ-歩みは `ρ` が各 `Match` で選ぶアームだけで決まる (D17) ので、`id(s)` も `Anc(s)` も
`ρ` を辿る 1 つの活性化について固定される。上の 4 つの集合は `ρ` の上に**いつか**存在するすべての
スロットを走る。時点に依るのは `μ_τ`・`Down_τ`・`Bmp_τ`・`Bsub_τ` の 4 つだけである。
**まだ値を得ていないスロットは `μ_τ = 0` を寄せ** (`DEF 割り当て` の 6 種はいずれもその位置の
スロットについて値を動かす。D6 -- スロットはその変数が値を得た後に在る)、**その名前の bump も
`Bmp_τ = 0` である** (D27 より要素が `pending` に入るのは `Retain` 節点の訪問だけであり、
`B(p, ρ)[id] ≥ 1` にはその時点で `identity` が `id` である inhabited かつ計数下の leaf -- すなわち
D6 のスロット -- が要る)。よって時点ごとの量は、その時点までに値を得たスロットだけを数える。

**名前は類を決める (`L5b`)。** `origin(w, μ).identity()` が `Ids(C)` の元であり、`obj(w, μ)` が
計数下 (D26) である `ρ` の上のスロット `(w, μ)` は、`C` のスロットである。`Ids(C)` の元は `C` の
スロットの `identity` なので、これは `L5b` をその元に当てたものである。この節の補題はこの形で
`L5b` を引く。

**前提 (S)**。この節の補題が本体 `B` とそれが属するプログラムについて読むのは、次の 4 つだけである。

- **(S0)** `ρ` の上の各時点において、`B` の各スロットの `μ` は 0 以上である。
- **(S1)** そのプログラムのすべての関数の `borrowed_units` が空である。D14 より、すべての
  パラメータ・capture の unit はその関数が所有する。
- **(S2)** `B` の各 `Retain(v, π)` 節点の入口において、その節点が触れる各スロット -- `π` の下の
  inhabited (D16) な各 leaf `(v, λ)` -- の `μ` は 1 以上である。
- **(S3)** `ρ` の上で、各計数下の別名類に `held_ρ(・, ・)` の開始値 1 を与える事象は高々 1 つである。

### 11.1a `L20a` (`insert_rc` の出力は (S) を満たす)

**言明**。`insert_rc` の出力の各本体は (S0)・(S1)・(S2)・(S3) を満たす。

**証明**

<1>0. (S0)。
  BY L17, L18
  `L17` (a) より `ρ` の上の各節点の入口は 3 通りである。本体の根の検査点より前の `Release` 鎖の中では
  `L18` が「すべてのスロットの `μ` は非負」を与え、連続する 2 つの検査点の間の遷移の中では `L17` (b) が
  同じことを与え、関数本体・初期化子の終端の `Ret` の検査点では `L18` が `μ` を `Λ` の指示関数と定める
  のでどのスロットでも 0 か 1 である。

<1>1. (S1)。
  BY L15
  `L15` (e) が (S1) そのものである。

<1>2. (S2)。
  BY L17, L18a
  `L18a` より、出力の各 `Retain` 節点の入口は連続する 2 つの検査点の間の遷移の中の点である。
  `L17` (b) より、遷移の中の各節点の入口で、`Retain`/`Release` が触れる各スロットの `μ` は 1 以上で
  ある。

<1>3. (S3)。
  BY L20
  `L20` が (S3) そのものである。

<1>4. QED
  BY <1>0, <1>1, <1>2, <1>3

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

<1>5a. ρ-歩みの上で、候補が 2 つ以上の `Join` の腕を取るスロット `s_k` について、`id(s_k)` は
       `Anc(s_{k+1})` に入らない。したがって `Anc(s_k) = {id(s_k)} ∪ Anc(s_{k+1})` は
       `Anc(s_{k+1})` を真に含む。
  BY <1>2, <1>5, D2, D6, D33, A6, A11, CODE src/rc_ir/ownership.rs: origin_inner
  <1>2 より、候補が 2 つ以上の `Join` では `identity` は `s_k` 自身の位置に取り替わるので
  `id(s_k) = s_k` であり、`s_k` はその `Match` の束縛変数 `x` の leaf である。
  `Anc(s_{k+1})` の元は `s_{k+1}` の歩みの上の位置である (<1>5、D33)。歩みの各段は、いま居る位置の
  変数の束縛が名指す変数へ進む (`origin_inner` の各腕がその変数について `origin` を呼ぶ)。A11 より
  その名指しはその束縛の位置でスコープに入っている束縛に解決し、D2 より `x` のスコープはその `Match` の
  継続の部分木であって、`s_{k+1}` -- 選ばれたアーム本体の結果のスロット -- の位置もその歩みが辿る
  どの束縛の位置もその中に無い。よって歩みの上の位置の変数はどれも `x` の束縛に解決せず、A6 より
  `x` という名前を束縛するものはプログラム全体で 1 つだけなので、どの位置の変数の名前も `x` ではない。
  束縛を持たない名前を終端とする位置は記号の位置であり (D6)、その名前は局所名でないので `x` とは
  異なる。したがって `id(s_k) ∉ Anc(s_{k+1})` である。

<1>6. (d)。
  BY <1>3, <1>5, <1>5a
  `id ∈ Anc(s)` のとき、`Anc(s)` の定義より `id = id(s_j)` であるスロット `s_j` が `s` の歩みの上に
  ある。`s` の歩みの `s_j` 以降は `s_j` の歩みそのものなので `Anc(s_j) ⊆ Anc(s)` であり、(b) より
  `Anc(s_j) = Anc(id)` である。
  `Anc(s)` の相異なる 2 つの元は、歩みの上の相異なる位置で立つ名前である (<1>5)。<1>5a より、名前が
  切り替わる各段で `Anc` は真に大きくなるので、後で立つ名前 `id'` の `Anc(id')` は先に立つ名前の
  `Anc` の**真**部分集合である。よって `Anc(s)` の元は「`Anc` の包含」で線形順序をなし、相異なる
  2 つの元は相異なる `Anc` を持つ。すべての `Anc` は `id_0` を含む (<1>3) ので、`Ids(C)` の上の関係
  「`id' ∈ Anc(id)`」は反射的・推移的であり、いま示したとおり反対称でもある -- すなわち `id_0` を
  最大元とする半順序であり、各 `id` の上側は鎖である。すなわち `Ids(C)` は `id_0` を根とする木を
  なす。

<1>7. (e)。
  BY <1>2, <1>5, <1>5a, D9, D20, A6, CODE src/rc_ir/ownership.rs: collect_bindings
  D9 の移動の表は 6 行を持つ。`Let(x, Var(y), k)` の移動先は `Binding::Move`、unbox 容器の
  `Destructure` の名前付きフィールドは `Binding::Field` の unbox の枝、unbox union の変位アームの
  payload 束縛は `Binding::Payload` の `Some(tag)` かつ scrutinee が unbox の枝、catch-all アームの
  payload 束縛は `Binding::Payload` の `None` の枝、`Llvm` の素通し leaf は `Binding::Llvm` の
  単一 `Arg` の枝に束縛される (`collect_bindings`)。この 5 つは <1>2 の辿る腕であり、`origin(s')` は
  `origin(s)` に等しいので `Anc(s') = Anc(s)` である。
  残る 1 行 -- `Match` のアーム本体の `Ret(x)` -- の移動先は `Binding::Join` である。候補が 1 つの
  ときは <1>2 より辿る腕と同じ形で `Anc(s') = Anc(s)`、候補が 2 つ以上のときは `id(s')` が `s'` 自身
  であり、<1>5 の展開より `Anc(s') = {id(s')} ∪ Anc(s)` である。`id(s') ∉ Anc(s)` は <1>5a である
  -- `s'` の歩みは `s'` から `s` へ進むので、<1>5a の `s_k` が `s'`、`s_{k+1}` が `s` である。

<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

**`Sub` と `Down` の言い換え。** (b) より `id ∈ Anc(s)` と `id(s) ∈ Sub(id)` は同値である
(`id(s) ∈ Sub(id)` はあるスロット `s''` について `id(s'') = id(s)` かつ `id ∈ Anc(s'')` であること
であり、(b) より `Anc(s'') = Anc(s)`)。よって `Down_τ(id) = Σ_{s : id(s) ∈ Sub(id)} μ_τ(s)` である。

### 11.3 `L22` (木の根で読むと `held` と `bumps` になる)

**言明**。`ρ` の上の各時点 `τ` について `Down_τ(id_0) = held_ρ(τ, C)` かつ
`Bsub_τ(id_0) = bumps_ρ(τ, C)` である。

**証明**

<1>1. `Down_τ(id_0) = held_ρ(τ, C)`。
  BY L21, L13a, DEF 割り当て, 前提 (S1)
  `L21` (c) よりすべてのスロット `s` について `id_0 ∈ Anc(s)` なので、`Down_τ(id_0)` は `C` の全
  スロットにわたる `μ` の和である。前提 (S1) よりすべての unit が所有されるので `L13a` (c) が当たり、
  `L13a` (b) よりその和は `held_ρ(τ, C)` である。

<1>2. `Sub(id_0) = Ids(C)`。
  BY L21
  `L21` (c) より各スロット `s` について `id_0 ∈ Anc(s)` であり、`Sub(id_0)` の定義よりその `id(s)` は
  `Sub(id_0)` に入る。

<1>3. QED
  BY <1>2, A19, D27, L5b, 11.1 節の `Ids(C)` の定義
  A19 (ii-b) は `bumps` の帰属を定める -- 「`bumps_ρ(τ, C)` とは、時点 `τ` の `pending` の各要素 `p`
  について D27 が定める `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和で
  ある」。11.1 節の `Ids(C)` は `C` のスロットの `identity` の
  全体なので、その総和は `Σ_{id ∈ Ids(C)} Bmp_τ(id)` に等しい (`L5b` より、`Ids(C)` の名前が付く
  bump は `C` のスロットの bump に限る)。<1>2 より `Sub(id_0) = Ids(C)` なので、右辺は
  `Bsub_τ(id_0)` である。

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
前提 (S1) よりすべての unit が所有されるので `owns` は常に真であり、`L16` より借用オペランドの素通しは
無いので、この 2 つが挙げる leaf は D9 の消費の表の対応する行が挙げる leaf に一致する。
**関数本体・初期化子の終端の `Ret` の消費には、対応する呼び出しが無い。** `walk_inner` の
`RcExpr::Ret` の腕は `returns_from_func` が真のとき `needed_retains` に入れるだけで `pending` を
変えない。この消費の後には `ρ` の上に時点が無いので、`L24` と `L25` の言明はそこを扱わない
(`L19` (c) と `L38`)。

**言明**。(S1) の下で、1 つの処分の事象と各 `id ∈ Ids(C)` について、その事象が `Down(id)` を減らす量を
`d` とすると、次のどちらかが成り立つ。

- **(i)** その事象で `Bsub(id)` は `d` 以上減る。
- **(ii)** 事象の後、その事象が処分したスロットのうち `id ∈ Anc(s)` であるもの `s` について、
  `Anc(s) ∩ Sub(id)` の各名前 `id'` は `Bmp(id') = 0` である。

**証明**

<1>0. どの pending の要素 `p` と名前 `id'` についても `B(p, ρ)[id'] ≥ 0` であり、したがって
      `Bmp(id')` と `Bsub(id')` も 0 以上である。要素を `pending` から取り除くこと、および `un_bump` が
      `InBracket` で 1 つの要素の `B` から引くことは、どの `id'` についても `Bmp(id')` を増やさない。
  BY L5a, D27, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: un_bump
  `L5a` (b) が第 1 文を与え、`Bmp` と `Bsub` はその非負の項の和である。`consume_objects` は要素を
  取り除くだけであり、取り除かれた要素の `B` は和から落ちるので、非負の項が 1 つ減る。`un_bump` の
  `InBracket` は 1 つの要素の `B` から引くだけである。

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
  BY <1>0, <1>1, <1>2, L21, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, D15
  `consume` は `origin(vars, type_env, var, path).acted_on()` を `consume_objects` に渡す。`path` は
  boxed leaf の path であり、それが inhabited でなければ D9 の消費は参照を処分せず `d = 0` で (i) が
  成り立つ (`consume_objects` は要素を減らすだけなので `Bsub` は増えない)。inhabited なとき処分される
  スロットは `s = (var, path)` の 1 つであり、`d = [id ∈ Anc(s)]` である。`id ∉ Anc(s)` なら `d = 0`
  で同じく (i) が成り立つ。`id ∈ Anc(s)` なら、
  `L21` (a) より `Anc(s) ⊆ acted_on(s)` であり、<1>1 と <1>2 より `Anc(s)` のうち `Bmp ≥ 1` である
  名前を持つ要素はすべて取り除かれる。よって事象の後 `Anc(s)` の各名前について `Bmp ≤ 0` であり、
  <1>0 の非負性と合わせて `Bmp = 0` なので (ii) が成り立つ。

<1>4. CASE 事象が `Release(v, π)` の訪問である。
  <2>1. 訪問はまず `consume_objects(pending, other_objects(v, π))` を行う。
        `other_objects(v, π)` は `π` の下の各 boxed leaf の `candidates() \ {identity()}` を集める。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects
  <2>2. この段の後、処分される各スロット `s = (v, λ)` について、`Anc(s) \ {id(s)}` の各名前
        `id'` は `Bmp(id') = 0` である。
    BY <2>1, <1>0, <1>1, <1>2, L21, D15
    `L21` (a) より `Anc(s) ⊆ acted_on(s) = {id(s)} ∪ candidates(s)` (D15) なので
    `Anc(s) \ {id(s)} ⊆ candidates(s) \ {id(s)}` であり、それは `other_objects(v, π)` に含まれる。
  <2>3. 続けて `un_bump(pending, acted_references(v, π))` が呼ばれる。返り値が `InBracket` のとき、
        選ばれた要素の `B(p, ρ)` からこの `Release` が `ρ` で実際に処分する参照の多重集合が引かれる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: un_bump, D27
  <2>4. CASE `un_bump` が `InBracket` を返す。(i) が成り立つ。
    BY <2>3, L5a, D27, L5b, L21
    この `Release` が処分するのは `π` の下の inhabited な各 leaf の参照であり (D10)、`Down(id)` が
    減る量 `d` はそのうち `id ∈ Anc(s)` であるスロットの個数である。`B(p, ρ)` は `origin` の
    `identity` を鍵とする多重集合なので (D27)、そこから引かれる「実際に処分する参照の多重集合」も
    同じ鍵で数えたものであり、`Sub(id)` に落ちる分はちょうど
    `id(s) ∈ Sub(id)` すなわち `id ∈ Anc(s)` であるスロットの分、すなわち `d` である。`L5a` (d) より
    引き算は打ち切られない -- 選ばれた要素の `B(p, ρ)` は各名前でその量を下回らない -- ので、引かれる
    量はそのまま `B(p, ρ)` から落ちる。`L5b` より `Sub(id) ⊆ Ids(C)` の名前が付く分はほかに無い。よって
    `Bsub(id)` はちょうど `d` 減る。
  <2>5. CASE `un_bump` が `OutsideBracket` を返す。(ii) が成り立つ。
    BY <2>1, <2>2, <1>0, <1>1, <1>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
       `RcExpr::Release` の腕, CODE src/rc_ir/ownership.rs: acted_references
    この腕は `consume_objects(pending, un_bumped.objects())` を呼ぶ。`acted_references(v, π)` は
    `π` の下の各 boxed leaf を `origin` の `identity` で数えるので、処分される各スロット `s` の
    `id(s)` はその `objects()` に入る。よって <1>1 と <1>2 より、事象の後 `Bmp(id(s)) ≤ 0` であり、
    <1>0 の非負性と合わせて `Bmp(id(s)) = 0` である。<2>2 と合わせて `Anc(s)` の各名前について
    `Bmp = 0` であり、その部分集合 `Anc(s) ∩ Sub(id)` についても成り立つ。
  <2>6. CASE `un_bump` が `NoBracket` を返す。(ii) が成り立つ。
    BY <2>2, <1>0, <1>1, CODE src/rc_ir/borrow.rs: un_bump,
       CODE src/rc_ir/ownership.rs: acted_references
    `NoBracket` は、`outstanding` が `un_bumped` とオブジェクトを共有する要素が `pending` に無いこと
    である。処分される各スロット `s` の `id(s)` は `un_bumped` が名指すので、どの要素の `outstanding`
    も `id(s)` を名指さない。<1>1 より `Bmp(id(s)) ≤ 0` であり、<1>0 の非負性と合わせて
    `Bmp(id(s)) = 0` である。<2>2 と合わせて `Anc(s)` の各名前について成り立つ。
  <2>7. QED
    BY <2>4, <2>5, <2>6, CODE src/rc_ir/borrow.rs: un_bump
    `un_bump` は `UnBump::NoBracket`、`UnBump::OutsideBracket`、`UnBump::InBracket` のいずれかを
    返す。

<1>5. QED
  BY <1>3, <1>4
  処分の事象は `DEF 処分の事象` の 2 種で尽きる。

### 11.5 `L24` (名前の木の不等式)

**言明**。(S) の下で、`ρ` の上の各時点 `τ` と各 `id ∈ Ids(C)` について、`Bsub_τ(id) ≥ 1` ならば
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
    BY L5b, 前提 (S3), D2, D6, D27, D33, D34, A11, L13a, L21, DEF 割り当て
    まず、この事象より前に `C` はスロットを持たない。D33 の ρ-歩みの各段は、いま居る位置の変数の
    束縛が名指す変数へ進み、A11 と D2 よりその変数はいま居る位置の変数より前に値を得ている。よって
    `C` のスロットが在る時点では `C` の ρ-終端の位置も値を得ており、`L13a` (f) よりその位置は
    パラメータ・capture の leaf か D10 の生成の位置であって、D34 の「開始の時点」の段落よりそこで
    開始値が置かれる。(S3) よりその事象は `ρ` の上に高々 1 つであり、いまの事象がそれである。
    事象の直後に値を得ている `C` のスロットは、この事象が値を与える位置の leaf のうち `C` に属する
    ものである。`L13a` (d)(e) より、パラメータ・capture の leaf も D10 の生成の位置も自分が属する類の
    ρ-終端であり、ρ-終端はスロットごとに 1 つに決まるので、そのような leaf は `C` の ρ-終端 1 つ
    だけである。`L21` (c) よりその `id` は `id_0` である。
    `DEF 割り当て` の 6 種はいずれもその位置のスロットについて値を動かすので、D6 よりまだ値を得て
    いないスロット `s` は `μ_τ(s) = 0` を寄せる。よって `id ≠ id_0` である各 `id ∈ Ids(C)` について
    `Down(id) = 0` である。
    `Bsub` の側はどの `id ∈ Ids(C)` についても 0 である -- D27 より要素が `pending` に入るのは
    `Retain` 節点の訪問だけであり、`B(p, ρ)[id] ≥ 1` にはその訪問の時点で `identity` が `id` である
    inhabited かつ計数下の leaf が要る。その leaf は D6 のスロットであり、`L5b` よりそれは `C` の
    スロットである。上より `C` のスロットはこの事象より前には無い。よってどの `id ∈ Ids(C)` に
    ついても `Bsub(id) = 0` であり、言明は空虚に真である。
  <2>2. CASE (E-Retain)。
    BY D27, DEF 割り当て, 前提 (S0), 前提 (S2), L5b, L21
    `Retain(v, π)` の訪問は、`π` の下の inhabited かつ計数下の各 leaf `λ` について `μ(v, λ)` を
    1 上げ (`DEF 割り当て`)、同時に押し込まれる要素の `B(p, ρ)[id(v, λ)]` を 1 上げる (D27)。
    `id` を固定し、`k := #{λ : λ は π の下の inhabited な leaf で (v, λ) は C のスロットであり
    id ∈ Anc(v, λ)}` と置くと、`Down(id)` も `Bsub(id)` もちょうど `k` 増える
    (`id(v, λ) ∈ Sub(id)` と `id ∈ Anc(v, λ)` は同値、11.2 節)。`Bsub(id)` の側にこれ以外の分が
    無いのは `L5b` による -- `id(v, λ) ∈ Sub(id) ⊆ Ids(C)` である inhabited かつ計数下の leaf の
    スロット `(v, λ)` は `C` のスロットであり、`C` のスロットでない leaf の分は `Sub(id)` の名前に
    付かない。`k = 0` の `id` では何も変わらない。
    `k ≥ 1` のとき、差 `Down(id) - Bsub(id)` は変わらないので、事象の前に `Bsub(id) ≥ 1` であれば
    言明はそのまま保たれる。事象の前に `Bsub(id) ≤ 0` であったときは、(S2) よりその `Retain` が
    触れる `k` 個のスロットは訪問の入口で `μ ≥ 1` であり、(S0) より `Down(id)` が数える残りのスロットの
    `μ` は 0 以上なので `Down(id) ≥ k` であり、事象の後
    `Down(id) ≥ 2k` かつ `Bsub(id) ≤ k` である。`k ≥ 1` より `2k ≥ k + 1` なので
    `Bsub(id) ≤ Down(id) - 1` である。
  <2>3. CASE (E-移動-同名)。
    BY DEF 割り当て, L21
    `L21` (e) の前者の場合であり、`Anc(s') = Anc(s)` である。`μ(s)` が 1 減り `μ(s')` が 1 増えるので、どの `id` についても `Down(id)` は変わらない。
    `B(p, ρ)` は動かない (D27) ので `Bsub` も変わらない。
  <2>4. CASE (E-移動-新名)。
    BY DEF 割り当て, D2, D6, D27, D33, A6, A11, L5b, L21
    `L21` (e) の後者の場合であり、`Anc(s') = {id(s')} ∪ Anc(s)` かつ `id(s') ∉ Anc(s)` である。
    `id ∈ Anc(s)` については `Down(id)` は `-1` (移動元) `+1` (移動先) で変わらず、`Bsub(id)` も
    変わらない。`Anc(s) ∪ {id(s')}` に入らない `id` については、移動元も移動先もその `Sub(id)` に
    落ちないので両方が変わらない。
    残るのは `id = id(s')` である。`L21` (e) より `id(s')` は `s'` 自身の位置、すなわちこの移動が
    値を与える `Match` の束縛変数 `x` の leaf である。`Sub(id(s'))` の名前を `id(s'')` とする
    スロット `s''` は `id(s') ∈ Anc(s'')` を満たすので、D33 の ρ-歩みは `s''` から `s'` の位置を
    通る。歩みの各段は、いま居る位置の変数の束縛が名指す変数へ進み、A11 と D2 よりその変数はいま居る
    位置の変数より前に値を得ているので、`s''` の変数は `x` より後に値を得る (A6 より `x` を束縛する
    ものは 1 つだけであり、`s''` の歩みがその位置を通る)。よってこの移動の直後に値を得ている
    そのようなスロットは `s'` だけである。`DEF 割り当て` の 6 種はいずれもその位置のスロットについて
    値を動かすので、D6 よりまだ値を得ていないスロットは `μ = 0` を寄せる。すなわち `Down(id(s'))` は
    0 から 1 になる。
    `Bsub(id(s'))` の側は 0 である -- D27 より要素が `pending` に入るのは `Retain` 節点の訪問だけで
    あり、`Sub(id(s'))` のある名前について `B(p, ρ) ≥ 1` にはその訪問の時点で `identity` がその名前
    である inhabited かつ計数下の leaf が要る。その leaf は D6 のスロットであり、`L5b` より `C` の
    スロットであって、上よりその変数はこの移動より後に値を得る。よって言明は空虚に真である。
  <2>5. CASE (E-処分)。
    <3>1. `id ∉ Anc(s)` がすべての処分されるスロット `s` について成り立つ `id` では、`Down(id)` は
          変わらず `Bsub(id)` は増えない。
      BY L23, L5a, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
         CODE src/rc_ir/borrow.rs: un_bump, D27
      `consume_objects` は要素を取り除くだけ、`un_bump` は `InBracket` のとき 1 つの要素の `B` を
      引くだけである。`L5a` (b) より各要素の `B` の成分は 0 以上なので、要素が和から落ちることも
      1 つの成分が引かれることも `Bsub` を増やさない。
    <3>2. 残る `id` について、`L23` の (i) が成り立つ場合は言明が保たれる。
      BY L23
      `Down(id)` が `d` 減り `Bsub(id)` が `d` 以上減るので、`Bsub(id) ≤ Down(id) - 1` は
      `Bsub` が 1 以上である限り保たれる。
    <3>3. 残る `id` について、`L23` の (ii) が成り立つ場合も言明が保たれる。
      BY L23, L5a, <1>1, L21, 前提 (S0), 帰納法の仮定
      `S := ∪_s (Anc(s) ∩ Sub(id))` と置く。ここで `s` はこの事象が処分したスロットのうち
      `id ∈ Anc(s)` であるものを走る。`S` は `Sub(id)` の中で `Anc` について上に閉じている --
      `id' ∈ Anc(s) ∩ Sub(id)` かつ `id'' ∈ Anc(id') ∩ Sub(id)` ならば `L21` (d) より
      `id'' ∈ Anc(s)` である。`L23` の (ii) より、事象の後 `S` の各名前 `id'` は `Bmp(id') = 0` で
      ある。<1>1 より `Sub(id) \ S = Sub(r_1) ⊎ … ⊎ Sub(r_k)` と分かれる。`Bsub(id)` は
      `Sub(id)` の各名前の `Bmp` の和なので、事象の後 `Bsub(id) = Σ_i Bsub(r_i)` である。
      処分されたスロット `s` は `id(s) ∈ S` なので、<1>1 より どの `Down(r_i)` にも数えられない。
      したがって `Down(r_i)` はこの事象で変わらず、`Bsub(r_i)` は増えない (<3>1 と同じ理由) ので、
      帰納法の仮定 (事象の前の時点での言明) より `Bsub(r_i) ≥ 1` である `i` については事象の後も
      `Bsub(r_i) ≤ Down(r_i) - 1` である。`L5a` (b) より `Bsub(r_i) ≥ 1` でない `i` の
      `Bsub(r_i)` は 0 である。
      `Bsub(id) ≥ 1` とすると、`Bsub(r_i) ≥ 1` である `i` が少なくとも 1 つあり、その個数を `k'` と
      すると
      `Bsub(id) ≤ Σ_{i : Bsub(r_i) ≥ 1} Bsub(r_i) ≤ Σ_{i : Bsub(r_i) ≥ 1} (Down(r_i) - 1)
       ≤ Down(id) - k' ≤ Down(id) - 1`
      である。最後から 2 つ目の不等号は、<1>1 より `Down(r_i)` が数えるスロットが互いに素で
      すべて `Down(id)` に数えられることと、(S0) より `Down(id)` が数える残りのスロット -- どの
      `Bsub(r_i) ≥ 1` である `Sub(r_i)` にも属さない名前を `id(s)` とするもの -- の `μ` が 0 以上で
      あることによる。
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>6. CASE (E-落とし)。
    BY L5a, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, D27
    `merge` が返す `pending` は `pending_in` を `uniform` で絞ったものであり、`uniform` に入るのは
    すべてのアームの出口に同じ `outstanding` で現れる要素だけなので、`ρ` が選んだアームの出口の
    `pending` の部分集合である。よって要素は減るだけであり、D27 より残る要素の `B(p, ρ)` は運ばれる。
    処分に伴わない `consume_objects` も要素を取り除くだけである。`L5a` (b) より落ちる要素の `B` の
    成分は 0 以上なので、どちらも `Down` を変えず `Bsub` を増やさず、言明は保たれる。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <1>2

<1>4. QED
  BY <1>2, <1>3, CODE src/rc_ir/borrow.rs: cancel
  `ρ` の上の事象の列についての帰納。`ρ` の上の各時点は、この列のある接頭のすべての事象が起きた後に
  来る。活性化の開始時、`pending` は空であり (`cancel` は
  `analysis.walk(body, PendingRetains::default(), true)` で走査を始める)、`C` はスロットを持たない
  ので `Bsub ≡ 0` かつ `Down ≡ 0` で言明は空虚に真である。段は <1>3 であり、<1>2 が事象を尽くす。

### 11.6 `L25` ((O2))

**言明**。(S) を満たす各本体、各実行路 `ρ`、`ρ` を辿る各活性化、各時点、各計数下の
別名類 `C` について、`bumps_ρ(・, C) ≥ 1` ならば `held_ρ(・, C) ≥ 1 + bumps_ρ(・, C)` である。
`ρ` の上にスロットを持つ類については、`L7` よりこれは `U + X ≥ D` と同値である。

**証明**

<1>1. `C` がスロットを持たない時点では `bumps_ρ(・, C) = 0` である。
  BY D27, D6, L5b
  D27 より要素が `pending` に入るのは `Retain` 節点の訪問だけであり、`B(p, ρ)[id] ≥ 1` には、その
  訪問の時点で `identity` が `id` である inhabited かつ計数下の leaf -- D6 のスロット -- が要る。
  `L5b` より、`id` が `C` のスロットの `identity` であるとき、そのようなスロットは `C` のスロットで
  ある。D6 よりスロットはその変数が値を得た後に在るので、`C` がスロットを持たない時点より前にも
  `C` のスロットは無い。よってその時点で `C` の名前に付く bump は 1 つも無い。

<1>2. `C` がスロットを持つ時点では、`L24` を `id_0` に当てて `L22` で読み替えると言明が出る。
  BY L24, L22
  `L22` より `Down(id_0) = held_ρ(・, C)`、`Bsub(id_0) = bumps_ρ(・, C)`。`L24` を `id = id_0` に
  当てると、`bumps ≥ 1` のとき `bumps ≤ held - 1` である。

<1>2a. `C` について `L6` の前提 (I) が成り立つ。
  BY D6, D16, D26, p13 の L12a, D33
  (I) は「`C` のスロットは `ρ` の上で inhabited であり、`obj(C)` は計数下である」である。D6 より
  スロット `(x, λ)` が在るのは `λ` が `ty(x)` の inhabited (D16) な boxed leaf であるときに限る。
  言明が量化するのは計数下の別名類であり、p13 の `L12a` と D33 より `C` のすべてのスロットは
  `obj(C)` を指すので、その `obj(C)` が計数下 (D26) であることが「計数下の別名類」の意味である。

<1>2c. `C` が `ρ` の上にスロットを持つとき、`C` は開始事象をちょうど 1 つ持つ。
  BY <1>2a, 前提 (S3), D6, DEF 開始事象, L13a
  (S3) が「高々 1 つ」を与えるので、1 つ以上あることを言えばよい。`L13a` (f) より、計数下の別名類
  `C` の ρ-終端はパラメータ・capture の leaf か D10 の生成の表の位置のいずれかである。`DEF 開始事象`
  の 3 行がその 2 種を覆うので、`C` のスロットが在る時点までに開始事象が 1 つ起きている
  (D6 -- スロットはその変数が値を得た後に在る)。

<1>3. QED
  BY <1>1, <1>2, <1>2a, <1>2c, L7
  不等式は、`C` がスロットを持つ時点では <1>2 が、持たない時点では <1>1 が `bumps = 0` を与えるので
  空虚に、それぞれ成り立つ。同値の側は、スロットを持つ類について `L7` の恒等式
  `held - (1 + bumps) = U + X - D` から出る -- `L7` の前提のうち `L4` の前提 (`C` の開始事象が 1 つ)
  は <1>2c が、`L6` の前提 (I) は <1>2a が与える。

**`insert_rc` の出力について。** `L20a` より `insert_rc` の出力の各本体は (S) を満たすので、`L25` から
(O2) -- 第 8 節の言明 -- が出る。**`split_rc_units` の出力について。** 第 13 節の `L32` が、その出力も
(S) を満たすことを示す。

## 12. `L38`: A19 (ii-b) が延びない点

`L19` (c) は、関数本体・初期化子の終端の `Ret` の消費の後、各計数下の別名類の `held` が 0 であることを
示す。A19 (ii-a) はこの点でも成り立つ (`0 ≥ 0`)。この節は A19 (ii-b) がこの点では偽であることを、
`insert_rc` の出力を挙げて示す。

### 12.1 道具立て

4.1 節の道具立て (名前の取り方を含む) を使う。`Arr` を boxed な型とし、`Pair` を `Arr` を 2 つ持つ
unbox 構造体とする。`make_pair : (Arr, Arr) -> Pair` は `InlineLLVMMakeStructBody` であり、その
`result_prov` は unbox 構造体について、結果の leaf `[i] ++ σ` を単一の `Arg(i, σ)` と宣言する
(`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody`)。`borrows_operand` は既定の
偽である (`CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand`)。

骨格 `S_g` (関数 `g`、パラメータ `m : Arr`、`borrowed_units` は空、返り値の型 `Pair`):

```
Let(x, Llvm(make_pair, [m, m]), Ret(x))
```

### 12.2 `L38` (A19 (ii-b) は活性化の終わりの 1 点先では偽である)

**言明**。`insert_rc` は `S_g` を次の本体 `B_2` に書き換える。

```
Retain(m, [], RcState::Unknown,
Let(x, Llvm(make_pair, [m, m]),
Ret(x)))
```

`B_2` は D11 を満たす (したがって `g` だけからなるプログラムは D12 を満たす)。`B_2` の唯一の実行路と、
`m` が計数下のオブジェクトを受け取る活性化について、
終端の `Ret` の消費を行った直後の点では `held = 0` かつ `bumps = 1` であり、`held ≥ 1 + bumps` は
偽である。

**証明**

<1>1. `insert_rc` は `S_g` を `B_2` に書き換える。
  BY A15, 4.1 節の名前の取り方,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
  `Ret(x)` の腕は `live_after = ∅` の下で `live = {x}` を返し、`retain_if_live` は
  `live_after` が空なので発火しない。`insert_into_operation_let` は `live_cont = {x}` の下で
  `rhs_operands(Llvm(make_pair, [m, m])) = [(m, Own), (m, Own)]` を逆順に走る -- 第 2 の `m` は
  `used_later` が偽 (`live_after_operand` はまだ `live_cont = {x}` の写しであって `m` を含まない) なので
  何も置かず、第 1 の `m` は第 2 の `m` が `live_after_operand` に入った後なので `used_later` が真で
  `retains_before` に入る (`needs_rc(m)` は真である -- `is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `ty(m) = Arr` では偽である)。`x` は `live_cont` に在る
  ので `after` は空である。`live_before` は `({x} \ {x}) ∪ {m} = {m}` であり、`insert_into_func` の
  `unused` は `live` が `m` を含むので空である。`build_retains([m], ・)` が `Retain(m, [])` を被せる。

<1>2. `B_2` の実行路は 1 本であり、`C := {(m, []), (x, [0]), (x, [1])}` はその上の 1 つの別名類で、
      `obj(C)` は計数下であり、`id` はどのスロットについても `(m, [])` である。
  BY D3, D4, D26, A3, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     D33
  `Match` が無いので実行路は 1 本である。`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `Arr` では偽であり、`Pair` は
  boxed なフィールドを持つのでその再帰でも偽である。`is_unbox` は
  `self.is_closure() || toplevel_tycon_info(type_env).is_unbox` であり `is_box` はその否定なので、
  `is_box` が真の `Arr` では `is_closure` が偽である。`Pair` は 12.1 節より unbox 構造体であって
  クロージャではない。よって D4 の判定はどちらの型でも第 1 規則と第 2 規則を抜け、
  `boxed_leaf_paths(Arr) = {[]}` (第 3 規則)、`boxed_leaf_paths(Pair) = {[0], [1]}` (第 5 規則) で
  ある。`collect_bindings` は `x` に
  `Binding::Llvm(make_pair, [m, m], Pair)` を入れ、12.1 節の宣言と `as_arg_projection` より
  `origin(x, [0])` は `origin(m, [])`、`origin(x, [1])` も `origin(m, [])` である。`m` はパラメータ
  なので `origin_inner` の `Binding::Param` の腕が `Exactly((m, []))` を返し、`(m, [])` が 3 つの
  スロットの ρ-終端である。3 つのスロットは ρ-終端が等しいので 1 つの別名類をなす。`obj(C)` は
  `m` が受け取ったオブジェクトであり、言明はそれが計数下である活性化について述べる。

<1>3. `μ`、`held`、`bumps` は次のように動く。
  BY <1>1, <1>2, L13a, L15, DEF 割り当て, D9, D10, D27, A3,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/ownership.rs: acted_references, CODE src/rc_ir/ownership.rs: rhs_consumes,
     CODE src/rc_ir/ownership.rs: passthrough_arg_leaves

  | 時点 | `μ(m, [])` | `μ(x, [0])` | `μ(x, [1])` | `held` | `bumps` |
  |---|---|---|---|---|---|
  | `Retain(m, [])` の入口 | 1 | 0 | 0 | 1 | 0 |
  | `Let(x, …)` の入口 | 2 | 0 | 0 | 2 | 1 |
  | `Ret(x)` の入口 | 0 | 1 | 1 | 2 | 1 |
  | 終端の `Ret` の消費の後 | 0 | 0 | 0 | 0 | 1 |

  `μ` の初期値は D10 の初期値の行 -- `L15` (e) より `m` の unit は `g` が所有する -- で
  `μ(m, []) = 1` である。
  `Retain(m, [])` は `μ(m, [])` を 1 上げる。`Let(x, Llvm(make_pair, [m, m]), ・)` は、D9 の移動の表の
  `Llvm` の素通し leaf の行で `(m, [])` の参照を `(x, [0])` と `(x, [1])` へ移すので `μ(m, [])` が
  2 下がり `μ(x, [0])` と `μ(x, [1])` が 1 ずつ上がる。終端の `Ret(x)` の消費は `x` の全 boxed leaf の
  参照を消費する。`L15` (e) と `L13a` (c) より `β(C) = 0` なので、`held` は `L13a` (b) より 3 つの
  `μ` の和である。
  `bumps` について、`Retain(m, [])` の訪問が積む要素の `outstanding` は
  `acted_references(m, []) = {(m, []): 1}` であり、D27 より `B` も `{(m, []): 1}` である。
  `Let(x, Llvm(make_pair, [m, m]), ・)` の訪問は `consume_rhs` を呼ぶが、`rhs_consumes` の `Llvm` の腕は
  素通しの leaf を消費として報告しない -- `passthrough_arg_leaves` が `(0, [])` と `(1, [])` を返し、
  `m` の boxed leaf は `[]` だけだからである。よってこの要素は落ちない。終端の `Ret` の腕は
  `returns_from_func` が真なので要素を `needed_retains` に入れるが、`pending` からは取り除かない。

<1>4. `B_2` は D11 を満たす。
  BY <1>1, <1>2, <1>3, L15, D9, D10, D11, A3
  `Obl` と `H(O_m)` は、活性化の開始で `{O_m}, 1`、`Retain(m, [])` の後 `{O_m, O_m}, 2`、`make_pair` の
  後も `{O_m, O_m}, 2` (A3 の単一 `Arg` の行より素通しは新しい参照を作らず、D9 の移動の行より `Obl` を
  変えない)、終端の `Ret` の消費で `{}, 2` -- 2 つの参照は呼び出し元へ渡る。(S-a) は各除去がその時点の
  `Obl` に入っており、(S-b) は終端の消費の後 `Obl` が空であり、(S-c) は `H(O_m) ≥ 1` の下で読み・
  触れるので、それぞれ成り立つ。D12 は「すべての関数の本体とすべてのグローバル初期化子の `init` が
  D11 を満たす」ことなので、`g` だけからなるプログラムは D12 を満たす。

<1>5. QED
  BY <1>1, <1>3, <1>4
  <1>3 の表の最後の行で `held = 0` かつ `bumps = 1` なので `held ≥ 1 + bumps` は偽である。

**`L25` はこの本体でも成り立つ。** <1>3 の表の 1 つ前の行までは `held ≥ 1 + bumps` である (`2 ≥ 2`)。
`L25` の言明が量化するのは `ρ` の上の時点 -- 節点の入口 -- であり、終端の `Ret` の消費の後はその中に
無い。

## 13. `split_rc_units` の段

A19 の範囲は `borrow_ify` の入力、すなわち `split_rc_units` の出力である。第 10 節と第 11 節が
示すのは `insert_rc` の出力についてなので、`Retain(v, π)` を `units_under(ty(v), π)` の鎖へ割る段が残る。
この節がそれを扱う。

`split_rc_units` は `optimize_rc_program` の中で `insert_rc` の直後・`borrow_ify` の直前に走り、その間に
他のパスは無い (`CODE src/build/build_object_files.rs: lower_and_insert_rc`,
`CODE src/build/build_object_files.rs: optimize_rc_program`)。よってこの節の入力は `insert_rc` の出力で
ある。

### 13.1 記法

`P` を `insert_rc` の出力のプログラム、`P' := split_rc_units(P)` とする。`B` を `P` の 1 つの本体
(ある関数の `body` か、あるグローバル初期化子の `init`)、`B' := split_body(B)` を `P'` の対応する本体と
する。

**DEF `Leaves`**。型 `τ` と path `π` について
`Leaves(τ, π) := {λ ∈ boxed_leaf_paths(τ) : λ は π で始まる}` と置く
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。

### 13.2 `L26` (`split_rc_units` が行う書き換え)

**言明**。

- **(a)** `split_rc_units` は `prog.funcs` の各関数の `body` と `prog.globals` の各初期化子の `init` を
  `split_body` の像に置き換え、`RcProgram`・`RcFunc`・`RcGlobalInit` の他のフィールドを 1 つも変えない。
  とくに `borrowed_units` を変えない。
- **(b)** `split_body` は本体の木を次の規則で写す。
  - `Retain(v, π, s, k)` は、`units_under(ty(v), π) = [u_1, …, u_n]` として
    `Retain(v, u_1, s, Retain(v, u_2, s, … Retain(v, u_n, s, split_body(k)) …))` に写る。`n = 0` の
    ときは `split_body(k)` そのものに写る。この `n` 個の節点を `t` の**鎖**と呼ぶ。
  - `Release(v, π, s, k)` は、節点の種類が `Release` であることだけが違う同じ形に写る。
  - `Let(x, Match(scrut, arms), k)` は `Let(x, Match(scrut, arms'), split_body(k))` に写る。`arms'` の
    第 `j` アームは `arms` の第 `j` アームの `tag`・`payload`・`payload_state` をそのまま持ち、本体が
    `split_body(arms[j].body)` である。
  - `Let(x, rhs, k)` (`rhs` は `Match` でない) は `Let(x, rhs, split_body(k))`、
    `Destructure(c, fs, s, k)` は `Destructure(c, fs, s, split_body(k))`、`Eval(v, k)` は
    `Eval(v, split_body(k))`、`Ret(v)` は `Ret(v)` に写る。

  この 5 種から作られた節点を**写し**と呼ぶ。写しは `B` の `Retain`/`Release` でない節点と 1 対 1 に
  対応する。**群**とは、1 つの写しか、1 つの `Retain`/`Release` 節点の鎖のことである。群は `B` の節点と
  1 対 1 に対応する。群の**入口**とはその最初の節点の入口であり、鎖が空 (`n = 0`) の群は節点を持たない
  のでその入口は次の群の入口と同じ点である。D2 より `Retain` と `Release` は継続をちょうど 1 つ持ち、
  本体の終端の節点は `Ret` -- 写し -- なので、鎖の群の次には必ず別の群がある。

**証明**

<1>1. (a)。
  BY CODE src/rc_ir/borrow.rs: split_rc_units
  この関数の本体は `for func in prog.funcs.values_mut() { func.body = split_body(&func.body, type_env); }`
  と `for g in &mut prog.globals { g.init = split_body(&g.init, type_env); }` の 2 つのループだけで
  あり、`body` と `init` 以外のフィールドへの代入を持たない。

<1>2. (b) の `Retain` の行。
  BY CODE src/rc_ir/borrow.rs: split_body, CODE src/rc_ir/borrow.rs: split_body_inner,
     CODE src/rc_ir/borrow.rs: split_rc, CODE src/rc_ir/borrow.rs: rc_node,
     CODE src/rc_ir/borrow.rs: expr_node, A15
  `split_body_inner` の `RcExpr::Retain(v, path, state, k)` の腕は、まず `k` を `split_body` で写し、
  その結果を `split_rc(v, path, *state, false, k, &node.source, type_env)` に渡す。`split_rc` は
  `units_under(&v.ty, path, type_env).into_iter().rev().fold(k, |cont, unit| rc_node(is_release,
  v.clone(), unit, state, cont, source))` であり、`rev().fold` は `k` から始めて外へ向かって節点を
  積むので、第 1 の unit が最も外側に来る。`rc_node` は `is_release` が偽のとき
  `RcExpr::Retain(var, path, state, k)` を作る。`units_under` が空の列を返せば `fold` は `k` を
  そのまま返す。A15 より `split_body` は `split_body_inner` をちょうど 1 回呼ぶ。

<1>3. (b) の `Release` の行。
  BY CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/borrow.rs: split_rc,
     CODE src/rc_ir/borrow.rs: rc_node, A15
  `split_body_inner` の `RcExpr::Release(v, path, state, k)` の腕は同じ `split_rc` を
  `is_release = true` で呼び、`rc_node` はそのとき `RcExpr::Release(var, path, state, k)` を作る。

<1>4. (b) の残る 5 行。
  BY CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/ast.rs: MatchArm::with_body,
     CODE src/rc_ir/borrow.rs: expr_node, A15
  `split_body_inner` の残る 5 つの腕は、いずれも同じ構成子を、同じ変数・同じ右辺・同じ `RcState` と、
  `split_body` で写した継続 (と写したアーム本体) から作り直す。`Match` の腕は
  `arm.with_body(split_body(&arm.body, type_env))` を使い、`with_body` は `body` 以外のフィールドを
  複製する (`MatchArm { body, ..self.clone() }`)。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, D2
  D2 より `RcExpr` は 6 個の構成子を持ち、`split_body_inner` の `match` は
  `Retain`・`Release`・`Let`(`Match` の右辺)・`Let`(その他)・`Destructure`・`Eval`・`Ret` の 7 つの腕で
  それを覆う。写しが `B` の `Retain`/`Release` でない節点と 1 対 1 に対応するのは、この 5 行がどれも
  節点を 1 つずつ作り直すからである。

### 13.3 `L27` (unit は path の下の boxed leaf を分割する)

**言明**。**A10 を満たす**任意の型 `τ` と任意の path `π` について次が成り立つ。この補題を当てる先は
プログラムに現れる変数の型なので、A10 がその前提を与える。

- **(a)** `units_under(τ, π)` の各元は `π` で始まる。`Leaves(τ, π)` の各元はちょうど 1 つの
  `u ∈ units_under(τ, π)` について `Leaves(τ, u)` に入る。すなわち
  `Leaves(τ, π) = ⊎_{u ∈ units_under(τ, π)} Leaves(τ, u)` である。
- **(b)** `subtree_type(τ, π)` が `Some` を返すとき、`units_under(τ, π)` の各元 `u` について
  `Leaves(τ, u)` は空でない。
- **(c)** `units_under(τ, [])` は `rc_units(τ)` である。その長さは 0 にも 1 にも 2 以上にもなり、
  長さが 0 のときは (a) より `Leaves(τ, []) = boxed_leaf_paths(τ)` が空である。

**証明**

<1>1. `unit_step(σ) = Fields { held_fields, .. }` である型 `σ` について、`boxed_leaf_paths` の `go` は
      `σ` の位置で最初の 4 つの `if` をすべて抜けて `for (i, fty) in ty.unpunched_field_types(type_env)`
      のループに入り、その `(i, fty)` の列は `held_fields` に等しい。
  BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  `unit_step` が `Fields` を返すのは `is_fully_unboxed`・`is_closure`・`is_box`・`is_union`・`is_array`・
  `is_punched_array` がすべて偽のときであり、そのとき `held_fields` は `ty.unpunched_field_types(type_env)`
  である。`go` の最初の 4 つの `if` はそれぞれ `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` を
  問うので、4 つとも偽であり、`go` は同じ `ty.unpunched_field_types(type_env)` のループに入る。

<1>2. ASSUME NEW 型 `σ` (A10 を満たす)、および (帰納法の仮定) `unit_step(σ)` の `Fields` の腕が
              降りる各フィールドの型について言明の結論が成り立つこと
      PROVE  `boxed_leaf_paths(σ)` の各元はちょうど 1 つの `rc_units(σ)` の元で始まる
  <2>1. CASE `unit_step(σ) = NoUnit`。
    BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    `unit_step` がこれを返すのは `σ.is_fully_unboxed(type_env)` が真のときである。`rc_units_go` の
    `NoUnit` の腕は `out` に何も積まないので `rc_units(σ)` は空であり、`boxed_leaf_paths` の `go` は
    最初の `if ty.is_fully_unboxed(type_env) { return; }` で返るので `boxed_leaf_paths(σ)` も空である。
    量化する元が無いので空虚に真である。
  <2>2. CASE `unit_step(σ) = Capture { capture_idx, .. }`。
    BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    `unit_step` がこれを返すのは `is_fully_unboxed` が偽で `σ.is_closure()` が真のときであり、
    `capture_idx` は `CLOSURE_CAPTURE_IDX as usize` である。`rc_units_go` の `Capture` の腕は
    `path.push(capture_idx)` の後に `path.clone()` を積むので `rc_units(σ) = {[capture_idx]}` である。
    `go` は最初の `if` を抜けて `if ty.is_closure()` の枝に入り、`path.push(CLOSURE_CAPTURE_IDX as
    usize)` の後に積んで返るので `boxed_leaf_paths(σ) = {[capture_idx]}` である。唯一の leaf が唯一の
    unit で始まる。
  <2>3. CASE `unit_step(σ) = Unit`。
    BY CODE src/rc_ir/ownership.rs: rc_units_go
    `rc_units_go` の `Unit` の腕は `path.clone()` を積む。最上位の呼び出しでは `path` は空列なので
    `rc_units(σ) = {[]}` である。`boxed_leaf_paths(σ)` のどの元も空列で始まり、unit は 1 つしか
    無いので「ちょうど 1 つ」が成り立つ。
  <2>4. CASE `unit_step(σ) = Fields { held_fields, .. }`。
    BY <1>1, 帰納法の仮定, CODE src/rc_ir/ownership.rs: rc_units_go
    `rc_units_go` の `Fields` の腕は `held_fields` の各 `(i, fty)` について `path.push(i)` の後に
    再帰する。<1>1 より `go` も同じ `(i, fty)` の列を同じ形で走る。よって
    `rc_units(σ) = ⊎_i {[i] ++ u : u ∈ rc_units(fty_i)}`、
    `boxed_leaf_paths(σ) = ⊎_i {[i] ++ λ : λ ∈ boxed_leaf_paths(fty_i)}` である。`[i] ++ λ` が
    `[j] ++ u` で始まるのは `i = j` かつ `λ` が `u` で始まるときに限るので、帰納法の仮定を `fty_i` に
    当てると「ちょうど 1 つ」が出る。
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/ownership.rs: UnitStep
    場合が尽きることは `UnitStep` が `NoUnit`・`Capture`・`Unit`・`Fields` の 4 個の構成子を持つ
    ことによる。

<1>2a. A10 を満たす任意の型 `σ` について、`boxed_leaf_paths(σ)` の各元はちょうど 1 つの
       `rc_units(σ)` の元で始まる。
  BY <1>2, A10
  A10 より `boxed_leaf_paths` も `rc_units` も停止するので、`unit_step` の `Fields` の腕が降りる
  フィールドの型の降下は有限である。<1>2 を段とする整礎帰納で、その降下で到達する各型について結論が
  成り立つ。

<1>3. `truncate_to_unit(σ, λ)` が値を返すとき、その値は `λ` の接頭である。
  BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  `out` は空の `Vec` から始まり、`for &idx in path` のループの各回で行うのは、`NoUnit` の腕での panic、
  `Capture` の腕での `out.push(idx)` と `break`、`Unit` の腕での `break`、`Fields` の腕での
  `out.push(idx)` と続行のいずれかである。よって `out` は `path` の添字を順に並べたその接頭である。

<1>4. A10 を満たす任意の型 `σ` について、`rc_units(σ)` の各元 `u` について `u` で始まる
      `boxed_leaf_paths(σ)` の元が在る。
  BY P1, A10, <1>3
  P1 は「**A10 を満たす**任意の型 `τ`」についての言明であり、その後半より、`rc_units(σ)` の各 unit
  `u` は、ある leaf `λ ∈ boxed_leaf_paths(σ)` について `truncate_to_unit(σ, λ)` が返す値である。
  <1>3 よりその値は `λ` の接頭であり、すなわち `λ` は `u` で始まる。

<1>5. `subtree_type(τ, π) = Some(σ)` のとき、`Leaves(τ, π) = {π ++ ν : ν ∈ boxed_leaf_paths(σ)}` で
      ある。
  BY <1>1, CODE src/rc_ir/ownership.rs: subtree_type, CODE src/rc_ir/ownership.rs: held_field_type,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  `subtree_type` は `π` の各添字 `idx` について、`unit_step(cur)` が `Fields` を返すときだけ `cur` を
  `held_field_type(held_fields, idx, "subtree_type")` へ降ろし、`NoUnit`・`Capture`・`Unit` のときは
  `None` を返す。よって `Some(σ)` は、`π` の各接頭の位置で `unit_step` が `Fields` であり、`σ` が
  `π` を降り切った先の型であることをいう。<1>1 より、`go` はそれらの位置で
  `unpunched_field_types` のループに入り、その第 `idx` フィールドの型は
  `held_field_type(held_fields, idx, ・)` と同じものである。`go` はパスを `path.push(i)` で伸ばしながら
  降りるので `path = π` になる位置はちょうど 1 つでありそこでの型は `σ` である。`Fields` の枝は
  `out.push` を持たないので `π` の真の接頭の位置では leaf を積まず、`π` で始まる leaf はすべてこの
  位置から下で積まれる。その分はちょうど `π ++ boxed_leaf_paths(σ)` である。

<1>6. (a)。
  <2>1. CASE `subtree_type(τ, π) = None`。
    BY CODE src/rc_ir/ownership.rs: units_under
    `units_under` の `None` の腕は `vec![path.clone()]` を返す。唯一の元は `π` 自身なので `π` で
    始まり、`Leaves(τ, π)` の各元は `π` で始まるのでちょうど 1 つの元について `Leaves(τ, u)` に入る。
  <2>2. CASE `subtree_type(τ, π) = Some(σ)`。
    BY CODE src/rc_ir/ownership.rs: units_under, <1>5, <1>2a, A10
    `units_under` の `Some` の腕は `rc_units(σ)` の各 `u` について
    `let mut unit_path = path.clone(); unit_path.extend(u);` を返すので、各元は `π ++ u` の形であり
    `π` で始まる。<1>5 より `Leaves(τ, π) = {π ++ ν : ν ∈ boxed_leaf_paths(σ)}` であり、`π ++ ν` が
    `π ++ u` で始まるのは `ν` が `u` で始まるときに限る。A10 の最後の段落より `σ` -- `τ` から
    `unpunched_field_types` を繰り返し取って到達する型 -- も A10 を満たすので、<1>2a を `σ` に当てると、
    各 `ν` はちょうど 1 つの `u ∈ rc_units(σ)` で始まる。
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: subtree_type
    `subtree_type` の返り値は `Some` か `None` のいずれかである。

<1>7. (b)。
  BY <1>4, <1>5, A10, CODE src/rc_ir/ownership.rs: units_under
  `subtree_type(τ, π) = Some(σ)` のとき `units_under(τ, π)` の各元は `π ++ u_0` (`u_0 ∈ rc_units(σ)`)
  である。A10 の最後の段落より `σ` -- `τ` から `unpunched_field_types` を繰り返し取って到達する型 --
  も A10 を満たすので <1>4 が当たり、`u_0` で始まる `ν ∈ boxed_leaf_paths(σ)` が在る。<1>5 より
  `π ++ ν ∈ Leaves(τ, π)` であり、`π ++ ν` は `π ++ u_0` で始まる。

<1>8. (c)。
  BY CODE src/rc_ir/ownership.rs: subtree_type, CODE src/rc_ir/ownership.rs: units_under,
     CODE src/rc_ir/ownership.rs: rc_units_go, <1>6
  `subtree_type(τ, [])` は空のループを抜けて `Some(τ)` を返すので、`units_under(τ, [])` は
  `rc_units(τ)` の各元に空の接頭を足したもの、すなわち `rc_units(τ)` である。長さについては、
  `rc_units_go` の 4 つの腕より、`unit_step(τ)` が `NoUnit` なら 0、`Unit` か `Capture` なら 1、
  `Fields` なら各フィールドの `rc_units` の連結であって 0 にも 2 以上にもなる。長さが 0 のときは、
  <1>6 (a) より `boxed_leaf_paths(τ)` の各元がある unit で始まらねばならず、unit が無いので
  `boxed_leaf_paths(τ)` は空である。

<1>9. QED
  BY <1>6, <1>7, <1>8

### 13.4 `L28` (`insert_rc` の出す path は空列であり、割った後は unit である)

**言明**。

- **(a)** `insert_rc` の出力の各 `Retain`/`Release` 節点の path は空列である。
- **(b)** したがって `split_rc_units` がそれに掛けるのは `units_under(ty(v), []) = rc_units(ty(v))` で
  あり (`L27` (c))、`P'` の各 `Retain`/`Release` 節点の path はその変数の型の unit である。その unit の
  下には boxed leaf が在る (`L27` (b))。

**証明**

<1>1. `Retain` について (a)。
  BY L9
  `L9` は、`insert_rc` の出力の各 `Retain` 節点が `Retain(v, [], RcState::Unknown, k)` の形であることを
  述べる。

<1>2. `src/rc_ir/rc_insert.rs` で `RcExpr::Release` を構成する式は `build_releases` の中の 1 つだけで
      あり、そこでは path が `vec![]` である。
  BY A25, CODE src/rc_ir/rc_insert.rs: build_releases
  `build_releases` は `RcExpr::Release(v, vec![], RcState::Unknown, c)` を作る。A25 より骨格
  (第 1 節) は `Release` 節点を含まないので、出力の `Release` 節点はすべてこの式が作ったものである。

<1>3. (b)。
  BY <1>1, <1>2, L26, L27, A10
  `L26` (b) より `P'` の各 `Retain`/`Release` 節点は、`P` のある `Retain(v, π)`/`Release(v, π)` 節点の
  鎖の 1 つであり、その path は `units_under(ty(v), π)` の元である。A10 より `ty(v)` は `L27` の前提を
  満たす。<1>1 と <1>2 より `π` は空列なので `L27` (c) よりそれは `rc_units(ty(v))` の元、すなわち
  `ty(v)` の unit である。`L27` (b) を `π = []` (このとき `subtree_type(ty(v), []) = Some(ty(v))`) に
  当てると、その unit の下に boxed leaf が在る。

<1>4. QED
  BY <1>1, <1>2, <1>3

### 13.5 `L29` (割る段は束縛・`origin`・別名類・`held` を変えない)

**言明**。`B` を `P` の 1 つの本体、`B' = split_body(B)` とする。

- **(a)** `B` と `B'` の `VarTable` について、`bindings` の鍵の集合と各鍵の束縛の形、`var_tys`、
  `param_tys`、`closure_targets` は一致する。したがって、鍵が等しい 2 つの `origin` の呼び出しの
  答えは等しく、`Origin::acted_on` と `acted_references` も引数が等しければ等しい。
- **(b)** `B'` の実行路 (D3) と `B` の実行路は 1 対 1 に対応する。対応する 2 本の路は、写し
  (`L26` (b)) の列が同じであり、各 `Match` で同じアームを選ぶ。
- **(c)** 対応する路 `ρ` と `ρ'` について、`ρ'` を辿る `B'` の各活性化 `α'` に対し、次の 3 つを持つ
  `B` の活性化 `α` がちょうど 1 つ存在する。この 2 つを**対応する活性化**と呼ぶ。

  1. パラメータ・capture の値が `α'` のものと等しい。
  2. D21 が挙げる「オペランドから結果が決まらない 4 種」のうち、写し (`L26` (b)) の節点に付く位置
     では、`α` の結果が `α'` の対応する位置の結果と等しい。
  3. `B` の `Retain`/`Release` 節点 `t` に付く (F) の結果は、`α'` が `t` の鎖の各節点に与える (F) の
     結果を鎖の順に並べたものである。

  対応する活性化は `ρ` と `ρ'` を辿り、対応する各写しの節点で同じ値を割り当て、各 `Match` で同じアームを
  選ぶ。したがって ρ-歩み・ρ-終端・別名類・`Anc`・`Ids`・`id`・inhabited (D16) の判定も 2 つの側で
  一致する。

  **第 3 項が要るのは D21 が (F) をどの構文にも付けるからである。** D21 は「**(F) はどの構文でも
  起こりうる** -- 参照を処分する段はどれもオブジェクトの解放を起こしうるので、`Release` の節点も、
  消費を行う `App` や `Destructure` の節点も、この意味で外から与えられる量を持つ」と述べる。割る段は
  1 つの `Release(v, π)` を `n` 個へ割るので、この量を担う位置の個数が 2 つの側で違う。第 3 項は
  その `n` 個分をまとめて 1 つの粗い節点の分と読む。
- **(d)** 対応する活性化について、`DEF 割り当て` の `μ` は、`B'` の各**群**の入口 (`L26` (b)) と `B` の
  対応する節点の入口とで等しい。したがって各**計数下の**別名類 `C` について `held` も等しい
  (D34 は `held` を計数下の類にしか定めない)。
- **(e)** `B` の `Retain`/`Release` 節点 `t` の鎖の第 `i` 節点の入口における `μ` は、その群の入口の `μ`
  に、第 1 から第 `i-1` 節点が触れる inhabited なスロットの分を、`t` が `Retain` なら足し `Release` なら
  引いたものである。鎖の全体が動かすスロットの多重集合は、`B` の `t` が動かすものに等しい。

**証明**

<1>1. (a)。
  <2>1. 2 つの表の `bindings` の鍵の集合、`var_tys`、`param_tys`、`closure_targets` は等しい。
        対応する各鍵の `Binding` は同じ構成子であり、`Move`・`Field`・`Payload`・`Join` の欄は
        等しく、`Llvm` の欄は `args` の列と結果の型が等しく、`llvm_gen` は `B` の側のものを
        `rhs.clone()` が写したものである。
    BY L26, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only,
       CODE src/rc_ir/borrow.rs: split_body_inner
    `collect_bindings` の `RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k)`
    の腕は `collect_bindings(k, vars)` を呼ぶだけで、`bindings`・`var_tys`・`param_tys`・
    `closure_targets` に何も入れない。表に何かを入れるのは `Let` と `Destructure` の腕であり、`Match` の
    腕は各アームの `payload` と `tag` と `returned_var(&arm.body)` を読む。`returned_var` は
    `Retain`/`Release`/`Eval`/`Let`/`Destructure` の継続を辿って `Ret` の変数を返す。`L26` (b) より
    `B'` は `B` の `Let`・`Destructure`・`Match` の構造と、各アームの `tag`・`payload`・
    `payload_state` と、各 `Ret` の変数をそのまま持つので、2 つの表は上の形で一致する。`Llvm` の欄に
    ついては、`split_body_inner` の `Let(x, rhs, k)` の腕が右辺に `rhs.clone()` を据えるので、
    `B'` の側の `llvm_gen` は `B` の側のものの写しであり、`args` と結果の型 (`x.ty`) は等しい。
    `VarTable::of` はこれにパラメータと capture の `Param` 束縛を足すだけであり、`L26` (a) より
    `params` と `capture` は変わらない。
  <2>2. 対応する 2 つの `Binding::Llvm` の欄について、
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の返り値は等しい。ここで `arg_tys` は
        `args` の各元の型の列である。
    BY <2>1, A3
    <2>1 より `args`・結果の型・`type_env` は 2 つの側で等しい。`rhs.clone()` は右辺を欄ごとに写す
    ので、複製の各欄は原本のものに等しく、A3 の「`result_prov` と `borrows_operand` は自分の
    `FullName` の欄を読まない」の節より、答えは `self` が持つ名前にも依らない。A3 の決定性の節 --
    「`result_prov` と `borrows_operand` は決定的である -- 同じ引数に対して常に同じ値を返す」-- より、
    2 つの呼び出しは同じ `Provenance` を返す。
  <2>3. `Org_B(x, π)` を、`vars_B` と `type_env` を第 1・第 2 引数として `origin` が鍵 `(x, π)` に
        返す値とする。この値は 1 つに決まり、`origin_inner(vars_B, type_env, x, π)` の値に等しい --
        ただしその中の各 `origin` の呼び出しは、その鍵の `Org_B` を返すものとして読む。`vars_{B'}` に
        ついて同じものを `Org_{B'}` と書く。
    BY P2, P2a, CODE src/rc_ir/ownership.rs: origin
    P2 より `origin` はこの引数について答えを返して停止し、P2a より鍵が等しい 2 つの呼び出しの返り値は
    等しいので、値は `vars.origins` が保持する memo の状態に依らず 1 つに決まる。`origin` は memo に
    鍵が無いとき `origin_inner` を走らせてその値を返すので、その鍵についての最初の呼び出しが等式を
    与える。その中の各 `origin` の呼び出しが返す値は、P2a よりその鍵の `Org_B` である。
  <2>4. 鍵 `(y, σ)` が `origin_inner(vars_B, type_env, x, π)` の中の `origin` の呼び出しで問われる
        ことを `(y, σ) ≺ (x, π)` と書くと、`≺` は整礎である。
    BY P2, CODE src/rc_ir/ownership.rs: origin
    `origin` が memo に答えを書くのは `origin_inner` から戻った後である。`≺` に無限下降列があれば、
    その列の頭の鍵についての最初の呼び出しは memo に当たる前に無限に潜り、停止しない。P2 は停止すると
    述べる。
  <2>5. P2 が範囲に入れる各鍵 `(x, π)` について `Org_B(x, π) = Org_{B'}(x, π)` である。
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: Origin::acted_on,
       CODE src/rc_ir/ownership.rs: Origin::of_candidates
    `≺` (<2>4) の上の帰納である。鍵 `(x, π)` を取り、`(y, σ) ≺ (x, π)` である各鍵について
    `Org_B(y, σ) = Org_{B'}(y, σ)` を仮定する。<2>3 より示すのは、2 つの表について `origin_inner` が
    同じ値を返すことである。`origin_inner` はまず `vars.bindings.get(var)` で腕を選び、<2>1 より鍵の
    集合と構成子は等しいので、2 つの側は同じ腕に入る。
    - `None`・`Binding::Param`・`Binding::Producer` の腕は `Exactly((var, path))` を返す。引数だけで
      決まるので等しい。
    - `Binding::Move(y)` の腕は `origin(vars, type_env, &y.name, path)` を返す。<2>1 より `y` は等しく、
      帰納法の仮定よりこの値は等しい。
    - `Binding::Join(arm_results)` の腕は、各 `arm_result` について
      `origin(vars, type_env, &arm_result.name, path).acted_on()` の元を `candidates` に入れ、
      `Origin::of_candidates(candidates, &(var, path))` を返す。<2>1 より `arm_results` は等しく、
      帰納法の仮定より各 `origin` の値は等しい。`acted_on` は `Origin` の値から決まるので
      `candidates` は集合として等しく、`of_candidates` はその集合と第 2 引数から値を決めるので、
      返り値は等しい。
    - `Binding::Llvm(llvm_gen, args, result_ty)` の腕は、<2>2 より同じ `decl` を得る。
      `decl.leaf_origins_at(path).and_then(as_arg_projection)` は `decl` と `path` から決まる。
      `Some((j, p))` のときは `origin(vars, type_env, &args[j].name, &p)` を返し、<2>1 より `args` は
      等しいので帰納法の仮定より値は等しい。`None` のときは
      `origin_from_leaves_under(vars, type_env, &decl, args, path, &(var, path))` を呼び、それが
      `None` を返せば `Exactly((var, path))` を返す。`origin_from_leaves_under` は
      `decl.leaf_origins_under(path)` と `truncate_to_unit(&args[j].ty, leaf, type_env)` から
      `operand_units` と `produced_here` を作り、各 `(j, unit)` について
      `origin(vars, type_env, &args[j].name, &unit)` を集めて `reached` とし、`produced_here` のとき
      `Exactly(here)` を足す。`operand_units` と `produced_here` は `decl`・`args` の型・`type_env`
      から決まり、`origin` の値は帰納法の仮定より等しいので、`reached` の元の集合は 2 つの側で等しい。
      返り値はその集合から決まる -- `reached` の元がすべて等しいときはその共通の値であり、そうでない
      ときは各元の `acted_on` を集めた**集合**を `of_candidates` に渡したものである。どちらも
      `reached` の並べ方に依らない。
    - `Binding::Field(container, idx)` の腕は `container.ty.is_box(type_env)` で枝が分かれ、<2>1 より
      `container` と `idx` は等しいので 2 つの側は同じ枝に入る。boxed の枝は `Exactly((var, path))` を
      返し、unbox の枝は `origin(vars, type_env, &container.name, &([idx] ++ path))` を返すので、
      帰納法の仮定より等しい。
    - `Binding::Payload(scrut, variant)` の腕も同じ形である。<2>1 より `scrut` と `variant` は等しい
      ので同じ枝に入り、`None` と `Some(tag)` (unbox) の枝は `origin` の値を返すので帰納法の仮定より
      等しく、`Some(_)` (boxed) の枝は `Exactly((var, path))` を返す。
    どの腕でも 2 つの側は同じ鍵を問い、同じ値を返す。
  <2>6. QED
    BY <2>1, <2>3, <2>5, CODE src/rc_ir/ownership.rs: Origin::identity,
       CODE src/rc_ir/ownership.rs: Origin::acted_on,
       CODE src/rc_ir/ownership.rs: acted_references,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <2>1 が `bindings`・`var_tys`・`param_tys`・`closure_targets` についての節を、<2>3 と <2>5 が
    `origin` の答えについての節を与える。`Origin::acted_on` は `Origin` の値から決まるので、答えが
    等しければ等しい。`acted_references(vars, type_env, v, path)` は
    `boxed_leaf_paths(&v.ty, type_env)` のうち `path` で始まる各 leaf の
    `origin(vars, type_env, &v.name, &leaf).identity()` を数えたものであり、`boxed_leaf_paths` は型と
    `type_env` から決まるので、引数が等しい 2 つの呼び出しの値は等しい。

<1>2. (b)。
  BY L26, D3
  D3 の規則で路が分かれるのは `Let(x, Match(v, arms), k)` の行だけである。`L26` (b) より `B'` の
  `Match` 節点とそのアームは `B` のものと 1 対 1 に対応し、アームの本体も `split_body` の像として
  対応する。`Retain` と `Release` は継続をちょうど 1 つ持つ (D2) ので路を分けない。よってアームの
  選び方が路を決め、2 つの側で同じ選び方が対応する。

<1>2a. `B` の `Retain`/`Release` 節点 `t` の鎖の全体が動かすスロットの多重集合は、`B` の `t` が
       動かすものに等しい。
  BY L26, L27, A10, DEF 割り当て, D16
  `L26` (b) より鎖の第 `i` 節点は `Retain(v, u_i)` (または `Release(v, u_i)`) であり、`DEF 割り当て`
  より `u_i` の下の inhabited な各 leaf `(v, λ)` の `μ` を ±1 する。A10 より `ty(v)` は `L27` の前提を
  満たす。`L27` (a) より `Leaves(ty(v), π)` は
  `Leaves(ty(v), u_1), …, Leaves(ty(v), u_n)` の直和であり、直和は inhabited なものに制限しても
  直和である。よって鎖の全体が動かすスロットの多重集合は、`B` の `t = Retain(v, π)` (または
  `Release(v, π)`) が動かす「`π` の下の inhabited な各 leaf」に等しい。

<1>3. (c)。
  <2>0. `B` の `Retain`/`Release` 節点を実行する段は、D24 の (E7) -- グローバルの初期化の段 -- を
        含まない。
    BY D22, D24, D7, D3, A25, L8, L15,
       CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/ownership.rs: collect_bindings
    (E7) が走るのは、まだ初期化されていないグローバルを読む者が居るときである (D22 の
    「グローバルのアクセサ」は初期化済みの旗を見る)。局所名の変数を名指す `Retain`/`Release` は
    グローバルの記憶域を読まないので、(E7) を起こしうるのは局所名でない変数を名指す節点だけである。
    `src/rc_ir/rc_insert.rs` で `RcExpr::Retain` と `RcExpr::Release` を構成する式は `build_retains` と
    `build_releases` の中の 1 つずつであり、A25 より骨格はこの 2 種を含まないので、`insert_rc` の出力の
    `Retain`/`Release` 節点はこの 2 つが作ったものに限る。`L8` より `build_retains` の呼び出し元は
    4 か所であり、`build_releases` の呼び出し元は 6 か所である。
    そのうち門を持つのは次である -- `insert_into_operation_let` の `retains_before` と
    `releases_after` は `if v.name.is_local()` の門の中に在り、`retain_if_live` は
    `var.name.is_local()` を要求し、`insert_into_eval` の `Release` も `x.name.is_local()` を要求する。
    `insert_into_destructure` の `dead` はフィールド変数、`insert_into_func` の `unused` は
    パラメータと capture、`insert_into_match` の dead-branch の `Release` は
    `live_at_arm_head` の名前 (`insert_if_local` が作る集合なので局所名) 、payload の `Release` は
    アームの payload である。
    `is_local()` の門を持たない位置は 3 つある。`insert_into_operation_let` の
    `if !live_cont.contains(&x.name) && self.needs_rc(&x) { after.push(x.clone()); }` と、
    `insert_into_match` の
    `if !live_cont.contains(&x.name) && self.needs_rc(&x) { build_releases(vec![x.clone()], cont) }` は
    どちらも `Let` の束縛変数 `x` を名指す。`collect_bindings` は `Let`・`Destructure`・`Match` の
    束縛変数を `vars.bindings` に入れるので、`L15` (d) の対偶より `x` は局所名である。
    残る 1 つは、`insert_into_match` が boxed union の変位アームの頭に置く `Release(scrut, [])` --
    `release_container && arm.tag.is_some() && needs_rc(&scrut)` の枝 -- である。その `scrut` が
    局所名でないグローバルであるとき、D7 の読む構文の表の `Let(x, Match(v, arms), k)` の行より
    その `Match` の写し節点が `scrut` を読み、D3 よりその節点は `ρ` の上でこの `Release` より前に
    あるので、そのグローバルの初期化はその写しの段で済んでいる。
  <2>1. `B` の活性化は、言明の 3 項が与えるデータで 1 つに決まる。
    BY <2>0, D21, D24, D2, L26
    D21 は「1 つの本体の活性化は、パラメータ・capture の値と、この 4 種の各位置での結果を与えると
    決まり、辿る実行路もそれで決まる」と述べる。`L26` (b) より `B` の節点は写しに対応するものと
    `Retain`/`Release` のものに尽きる。写しの節点に付く 4 種の位置は第 2 項が与える。
    `Retain`/`Release` の節点に付きうる 4 種は (F) だけであり、それを第 3 項が与える。まず、残る
    3 種 (一意性の観測点、外部の状態を読む `Llvm` の演算、`unique_check_operand` を宣言する `Llvm` の
    演算) はいずれも `Llvm` 節点に付き、D2 より `Retain`/`Release` は `Llvm` の右辺を持たない。
    第 4 種「子の活性化を作る段」は D24 の活性化の林が 5 種と数え上げる -- (E1)、(E3)、(E7)、
    オペランドを適用する `Llvm` の段、(F) の解放が `Destructor` について作る段である。(E1) は環境が
    活性化の根を作る段であって節点に付かず、(E3) は `App` の段、オペランドを適用する `Llvm` の段は
    `Llvm` の段なので、D2 よりどれも `Retain`/`Release` 節点には付かない。(E7) は <2>0 が除く。
  <2>2. 第 3 項が与えるものは、`t` の位置の (F) の結果として成り立つ。
    BY <1>2a, D24, A4
    D24 の (F) は解放を段の一部と定め、参照を処分するどの段でも起きうるとする。<1>2a より `t` の鎖の
    全体が処分する参照の多重集合は `B` の `t` が処分するものに等しいので、鎖の各節点で起きた解放を
    鎖の順に並べたものは、`t` の 1 つの段の中で起きる解放の列である。A4 より、その解放が記憶域へ
    書き込む内容は節点とオペランドの値と D21 の 4 種の結果だけで決まるので、並べ替えずに運べる。
  <2>3. 対応する活性化は `ρ` と `ρ'` を辿り、対応する各写しの節点で同じ値を割り当て、各 `Match` で
        同じアームを選ぶ。
    BY <2>1, <2>2, <1>2, <1>2a, L26, D2, D21, A4
    群の境界についての帰納。基底は本体の根の群の入口であり、第 1 項よりパラメータ・capture の値は
    等しい。段は 2 つある。写しの群では、`L26` (b) より 2 つの節点は同じ種類・同じ変数・同じ右辺を
    持ち、第 2 項よりその位置の D21 の 4 種の結果も等しいので、A4 と D21 より同じ値を割り当て、
    同じアームを選ぶ (`Match` の節点では <1>2 が路の対応を与える)。鎖の群では、D2 より
    `Retain`/`Release` は変数を束縛しないので値を割り当てず、<1>2a と <2>2 より鎖の全体は `B` の `t` と
    同じ参照を処分し同じ解放を起こす。
  <2>4. QED
    BY <2>0, <2>1, <2>2, <2>3, <1>1, D33, D16
    <2>1 より `α` はちょうど 1 つ存在する。ρ-歩みは `origin` の再帰を活性化が選んだアームに沿って
    辿るものなので、<1>1 (`origin` の答えが等しい) と <2>3 (選ぶアームが等しい) より 2 つの側で
    一致し、ρ-終端も別名類も一致する。`Anc`・`Ids`・`id` は `origin` の `identity` と ρ-歩みから
    定まるので一致する。inhabited (D16) は値の実行時のタグで決まり、<2>3 より値が一致するので
    一致する。

<1>4. (e)。
  BY <1>2a, L26, D2, DEF 割り当て, D16
  `L26` (b) より鎖の第 `i` 節点は `Retain(v, u_i)` (または `Release(v, u_i)`) であり、`DEF 割り当て`
  より `u_i` の下の inhabited な各 leaf `(v, λ)` の `μ` を `+1` (または `-1`) する。D2 より
  `Retain`/`Release` は継続をちょうど 1 つ持つので鎖の節点は継続の順に実行され、第 `i` 節点の入口
  までに第 1 から第 `i-1` 節点の分が動いている。言明の第 2 文は <1>2a である。

<1>5. (d)。
  BY <1>3, <1>4, L13a, L15, DEF 割り当て, L26
  `DEF 割り当て` の 6 種の事象のうち `Retain` と `Release` の 2 種は鎖の節点にだけ付き、残る 4 種
  (D10 の初期値、D10 の生成、D9 の消費、D9 の移動) は写しの節点にだけ付く -- `L26` (b) より `B'` の
  `Retain`/`Release` 以外の節点は写しであり、D10 の生成の表と D9 の 2 つの表に `Retain`/`Release` の
  行は無い。<1>3 より対応する写しの節点は同じ値を割り当てるので、この 4 種は対応する位置で同じ leaf に
  ついて同じだけ動かす。<1>4 より鎖の群の全体は `B` の `t` と同じだけ動かす。よって群の入口についての
  帰納 -- 基底は本体の根の群の入口 (D10 の初期値は 2 つの側で同じ) -- で、`B'` の各群の入口の `μ` は
  `B` の対応する節点の入口の `μ` に等しい。`L15` (e) と `L26` (a) より 2 つの側のどちらでもすべての
  unit が所有されるので `L13a` (c) が当たり、`held` は `L13a` (b) より `Σ_{s ∈ C} μ(s)` である。
  <1>3 より別名類も等しいので `held` も等しい。

<1>6. QED
  BY <1>1, <1>2, <1>2a, <1>3, <1>4, <1>5

### 13.6 (A19 (i) はこの段に何も要求しない)

A19 (i) は仮定ではなく、D21 が活性化に課す制限である (README の A19、D21)。実行 (D24) が作る活性化が
その制限を満たすことは P28 (b) が示すので、この段について示すものは無い。この段の出力には借用する
unit が無い (`L15` (e)、`L26` (a)) ので、`L13a` (c) より `β ≡ 0` であり、その本体のどの別名類でも
(i) の角括弧は 0 である。

### 13.7 `L31` (A19 (ii-a) の保存)

**言明**。`P` が A19 (ii-a) を満たすならば、`P' = split_rc_units(P)` も A19 (ii-a) を満たす。

**証明**

`B'` の 1 つの実行路 `ρ'` と、それを辿る 1 つの活性化を固定し、`L29` (b)(c) が与える `B` の実行路 `ρ`
と対応する活性化を取る。計数下の別名類 `C` を固定する。`L29` (c) より `C` は 2 つの側で同じ類である。

<1>0. `B` と `B'` のどちらの側でも、各時点と各計数下の別名類 `C` について
      `held(・, C) = Σ_{s ∈ C} μ(s)` である。したがって、`C` のどのスロットの `μ` も下げない事象は
      `held(・, C)` を下げず、どのスロットの `μ` も上げない事象は `held(・, C)` を上げない。
  BY L13a, L15, L26
  `L15` (e) より `P` のすべての関数の `borrowed_units` は空であり、`L26` (a) より
  `split_rc_units` はそれを変えないので `P'` についても空である。よって 2 つの側のどちらでも
  すべてのパラメータ・capture の unit が所有され、`L13a` (c) が当たる (グローバル初期化子の `init` の
  本体も `L13a` (c) が覆う)。

<1>1. 群 (`L26` (b)) の入口では `held_{ρ'}(・, C) = held_ρ(・, C)` であり、その値は仮定より 0 以上で
      ある。
  BY L29, 仮定
  `L29` (d) が等式を与える。群の入口は `B` の対応する節点の入口に対応し、そこで (ii-a) の前半を
  `B` に当てると `held_ρ(・, C) ≥ 0` である。

<1>2. `Retain` の鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>0, <1>1, L29
  `L29` (e) より鎖の第 1 から第 `i-1` 節点は各スロットの `μ` を上げるだけなので、<1>0 より
  `held_{ρ'}(・, C)` は群の入口の値以上である。<1>1 よりその値は 0 以上である。

<1>3. `Release` の鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>0, <1>1, L26, L29
  `L29` (e) より鎖の第 `i` から第 `n` 節点は各スロットの `μ` を下げるだけなので、<1>0 より第 `i`
  節点の入口の `held_{ρ'}(・, C)` は鎖の直後の点の値以上である。`L26` (b) より鎖の群の次には必ず
  別の群があるので、鎖の直後の点は群の入口であり、<1>1 よりそこで `held ≥ 0` である。

<1>4. D7 の読む構文が `C` のスロットを読む時点では `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>1, L26, L29, D7, 仮定
  D7 の読む構文の表の 6 行はいずれも `Let`・`Destructure`・`Eval` の節点であり、`L26` (b) よりそれらは
  すべて写しであって、その入口は群の入口である。<1>1 よりその入口の `held` は `B` の対応する節点の
  入口のものに等しく、`L29` (a)(c) より読まれるスロットも同じなので、仮定 ((ii-a) の後半を `B` に
  当てる) より `held ≥ 1` である。

<1>5. 鎖の群の第 `i` 節点 (`1 ≤ i ≤ n`) が `Retain(v, u_i)` で `C` のスロットを触れる時点では
      `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>0, L27, A10, L29, 仮定
  A10 より `ty(v)` は `L27` の前提を満たす。`L27` (a) より `u_i` は `π` で始まるので、`u_i` の下の
  inhabited な leaf は `π` の下の inhabited な
  leaf である。よって `B` の `t = Retain(v, π)` も `C` のスロットを触れ、仮定 ((ii-a) の後半を `B` に
  当てる) より `t` の入口で `held_ρ(・, C) ≥ 1` である。`L29` (d) よりこの群の入口の
  `held_{ρ'}(・, C)` はその値に等しい。`L29` (e) より鎖の第 1 から第 `i-1` 節点は各スロットの `μ` を
  上げるだけなので、<1>0 より第 `i` 節点の入口の `held_{ρ'}(・, C)` は群の入口の値以上である。

<1>6. 鎖の群の第 `i` 節点 (`1 ≤ i ≤ n`) が `Release(v, u_i)` で `C` のスロットを触れる時点では
      `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>0, <1>1, L26, L29
  第 `j` 節点が処分する `C` のスロットの個数を `d_j` とすると、`L29` (e) と <1>0 より第 `i` 節点の
  入口の `held_{ρ'}(・, C)` は鎖の直後の点の値に `Σ_{j ≥ i} d_j` を足したものである。`L26` (b) より
  鎖の直後の点は群の入口であり、<1>1 よりそこで `held ≥ 0` である。この節点が `C` のスロットを
  触れるので `d_i ≥ 1` である。よって `held ≥ d_i ≥ 1`。

<1>6a. 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った直後の点で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>1, L26, L29, D9, D16, 仮定
  `L26` (b) より `B'` の終端の節点は `Ret` の写しであり、その入口は群の入口なので <1>1 より
  そこで `held_{ρ'}(・, C) = held_ρ(・, C)` である。D9 の消費の表の「本体の終端の `Ret(x)`」の行が
  挙げる leaf は 2 つの側で同じである -- `L26` (b) より 2 つの `Ret` は同じ変数を名指し、`L29` (c) より
  対応する活性化は対応する各写しの節点で同じ値を割り当てるので、D16 の inhabited の判定も一致する。
  よって消費が `held` から引く量も等しく、消費の直後の `held` も 2 つの側で等しい。仮定 ((ii-a) の
  「非負であることは、終端の `Ret` の消費を行った直後の時点についても言う」を `B` に当てる) より
  `held_ρ(・, C) ≥ 0` なので、`held_{ρ'}(・, C) ≥ 0` である。

<1>7. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>6a, L26
  `L26` (b) より `ρ'` の上の時点は群の入口か、鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) かの
  いずれかである。(ii-a) の前半は、群の入口では <1>1 が、鎖の途中では <1>2 と <1>3 が与える
  (`L26` (b) の 2 行より、鎖は `Retain` の鎖か `Release` の鎖かのいずれかである)。(ii-a) が
  節点の入口の外に置く 1 点 -- 終端の `Ret` の消費を行った直後 -- は <1>6a が与える。(ii-a) の後半が
  当たるのは D7 の読む構文と `Retain`/`Release` の節点であり、前者は <1>4 が、後者は `L26` (b) より
  すべて鎖の節点なので <1>5 と <1>6 が与える。

### 13.8 `L32` (A19 (ii-b))

**言明**。`P' = split_rc_units(P)` の各本体は第 11 節の前提 (S) を満たす。したがって `L25` より
`P'` は A19 (ii-b) を満たす。

**証明**

<1>0. (S0)。
  BY L26, L29, L20a
  `L26` (b) より `ρ'` の上の各時点は、群の入口か、鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) かの
  いずれかである。群の入口では `L29` (d) より各スロットの `μ` は `B` の対応する節点の入口のものに
  等しく、`L20a` の (S0) を `B` に当てるとそれは 0 以上である。鎖の途中については、`L29` (e) より
  `Retain` の鎖は `μ` を下げないので群の入口の値以上であり、`Release` の鎖は `μ` を上げないので鎖の
  直後の点の値以上である。`L26` (b) より鎖の群の次には必ず別の群があるので、鎖の直後の点は群の入口で
  あり、そこで `μ ≥ 0` である。

<1>1. (S1)。
  BY L26, L15
  `L26` (a) より `split_rc_units` は `borrowed_units` を変えない。`L15` (e) より `P` のすべての関数の
  `borrowed_units` は空である。

<1>2. (S2)。
  BY L26, L27, A10, L29, L20a
  `L26` (b) より `B'` の各 `Retain` 節点は `B` のある `Retain(v, π)` 節点 `t` の鎖の第 `i` 節点
  `Retain(v, u_i)` である。それが触れるスロットは `u_i` の下の inhabited な leaf であり、A10 より
  `ty(v)` は `L27` の前提を満たすので、`L27` (a) より
  `u_i` は `π` で始まるので、それらは `π` の下の inhabited な leaf、すなわち `t` が触れるスロットで
  ある。`L20a` の (S2) を `B` に当てると `t` の入口でそれらの `μ` は 1 以上であり、`L29` (d) より
  この鎖の群の入口でも 1 以上である。`L29` (e) より第 1 から第 `i-1` 節点は `Retain` なので `μ` を
  下げない。よって第 `i` 節点の入口でも `μ ≥ 1` である。

<1>3. (S3)。
  BY L26, L29, L20, DEF 開始事象, D10
  `DEF 開始事象` より開始事象は D10 の初期値の行と D10 の生成の表の各行に当たる事象であり、
  D10 の生成の表に `Retain`/`Release` の行は無いので、それらは写しの節点と本体の入口にだけ付く。
  `L26` (b) と `L29` (c) より写しと別名類は 2 つの側で対応するので、`ρ'` の上の開始事象は
  `ρ` の上の開始事象と 1 対 1 に対応する。`L20` より後者は各計数下の別名類について高々 1 つである。

<1>4. QED
  BY <1>0, <1>1, <1>2, <1>3, L25

### 13.9 2 つの節がそろうこと

`split_rc_units` の出力 -- `borrow_ify` の入力 -- について、A19 の 2 つの節はどちらも成り立つ。
(ii-a) は `L31`、(ii-b) は `L25` を `L32` の (S) の上に当てたものである。(i) は仮定ではなく D21 が
活性化に課す制限であり、実行が作る活性化がそれを満たすことは P28 (b) が示す (第 13.6 節)。
(ii-a) だけが保存の形であり、`insert_rc` の出力についてそれが成り立つこと (`L19`) を入力とする。
(ii-a) が節点の入口の外に置く 1 点 -- 終端の `Ret` の消費を行った直後 -- も `L31` の量化に入り、
`insert_rc` の出力についてその点を等式で与えるのは `L19` (c) である。
(ii-b) は保存の形を取らない。粗い 1 つの
`Retain(v, π)` が積む要素は `acted_references(v, π)` を `outstanding` に持ち、それは鎖の各節点が積む
要素の `outstanding` の和である。`consume_objects` は `outstanding` が名指されたオブジェクトを含む要素を
落とすので (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)、粗い側の 1 要素が落ちる場面で
細かい側は名指されたオブジェクトを持つ要素だけが落ち、残りは pending に留まる。よって割った後の
`bumps` が割る前より大きい時点がありうる。そのかわり出力そのものが第 11 節の前提 (S) を満たすので、
`L25` を出力に当てて閉じる。

併せて、`L28` (b) より出力の各 `Retain`/`Release` 節点の path はその変数の型の unit である。これが
A2 が `split_rc_units` について述べていることである。
