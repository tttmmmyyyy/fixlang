# P7c: 処分はすべて走査に届く

**対象コミット**: `b6c51fb892746e493e155d9d59ea05d02d7357db`

この文書は README の P7c、P7f、P18b、P18a を扱う。README の定義 D1-D32 と仮定 A1-A21、および命題
P1、P2、P5、P6、P7、P7a、P7e、P8、P9、P10、P11、P12、P13、P14a、P16、P17、P18、P24、P28、P29 の**言明**の
上に立つ。それらの証明は `p05-holders.md`、`p10-leaves-and-units.md`、`p12-identity-and-consumes.md`、
`p20-borrow-ify.md`、`p30-cancel-walk.md`、`p40-cancel-soundness.md`、`p51-runs.md` にあり、この文書は
その言明だけを使う。

**結論を先に書く。P7c、P7f、P18b、P18a はいずれも証明できた。** P18a が立つのは README の
A19 (i)・(ii-a)・(ii-b) と P14a の上である。A19 (i) は、各活性化の計数下の別名類について
`d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` の総和と、借用終端の類が在るときの `+1` の
形で `H(O)` を下から抑える。第 7.5.4 節がその形から P18a を出す。

**P7c と `L16` は前提を 1 つ引く。** 第 4 節の (R) -- 出力の本体で `resolve_callee_params` が解決する
関数が実行時の呼び出し先である -- がそれで、README の P29 は同じ言明を `borrow_ify` の**入力**について
述べるだけである。最後の節がこれを差し戻す。

第 3 節が P7c の言明を README から引き、第 5 節がそれを証明する。第 4 節の `L6` が P7f を与え、
同じ節が (R) を述べる。第 7 節が P18b (第 7.4 節) と P18a (第 7.5 節) を扱う。第 6 節と第 7.6 節が、
かつてこの文書が README へ差し戻した点の現状をまとめる。

## 1. 記法

`cancel` の入力プログラムを `p` と書く (`CODE src/rc_ir/borrow.rs: cancel`)。`cancel` は `p` の各関数の
`body` と各グローバル初期化子の `init` のそれぞれについて `cancel_body` を 1 回呼ぶ。以下ではその 1 つを
固定し、**本体** `B` と書く。`B` に対応する `VarTable` を `vars` (`VarTable::of(f)` または
`VarTable::body_only(g.init)`)、プログラムの `TypeEnv` を `type_env` と書く。この 2 つは `B` ごとに 1 つ
なので、以下では `origin` と `acted_references` の第 1・第 2 引数を落として書く。

- `ty(x)` は変数 `x` の型、すなわち `x` を表す `RcVar` の `ty` フィールド (`CODE src/rc_ir/ast.rs: RcVar`)。
- `origin(x, π)` は `origin(vars, type_env, x, π)` (D13)。
- `acted_on(x, π)` は `origin(x, π).acted_on()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::acted_on`)。D15 より
  `acted_on(x, π) = {origin(x, π).identity()} ∪ origin(x, π).candidates()` である。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より、これが
  「`v` の `π` の下の boxed leaf」の全体である。inhabited (D16) でないものを含む。`L(v) = L(v, [])` と書く。
- `ActRefs(v, π)` は `acted_references(vars, type_env, v, π)` (D15)。
- `Others(v, π)` は `CancelAnalysis::other_objects(v, π)` の返す列を集合とみなしたもの
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。

`VarPath` は対 `(FullName, FieldPath)` である (`CODE src/rc_ir/ast.rs: VarPath`)。変数 `v` (`RcVar`) と
leaf `λ` について、`(v, λ)` は `VarPath` の対 `(v.name, λ)` を表す。**`VarPath` はオブジェクトの名前で
ある** -- README の第 3.5 節の最後の段落が「この文書はオブジェクトの名前を `VarPath` の水準で扱う」と
定める。`origin` の identity、`acted_references`、`PendingRetain::outstanding`、`consume_objects` の引数は
どれも `VarPath` を要素とする。第 6 節までがそれらについて「オブジェクト」と書くとき指すのはこの名前で
あり、走査が読む量はどれもこの水準にある。

第 7 節からは実行時のオブジェクト (D7) を相手にするので、2 つを**名前**と**オブジェクト**に書き分ける。
名前が指すオブジェクトは `obj(x, λ)` (D6) と `obj_ρ(o)` (第 7.2 節の DEF 名前の活性) が与える。1 つの
オブジェクトを 2 つの名前が指す本体は在る (第 7.5.8 節の `C2`) ので、この区別は表記だけのものではない。

この文書は補題を `L1` から `L6` と呼ぶ。`BY` の行ではそれらを名前で引用する。補題の証明の内部の
ステップは引用しない。第 4 節の最後に置く **(R)** は補題ではなく、この文書が引く前提である。

外部の結果を 2 つ使う。

- **stacker の `maybe_grow`**: `stacker::maybe_grow(red_zone, stack_size, callback)`
  は `callback` をちょうど 1 回呼び、その値を返す (`CODE stacker-0.1.23/src/lib.rs: maybe_grow`)。
  `remaining_stack()` の値で分かれる 2 つの枝がどちらも `callback` を 1 回だけ評価し、`callback` の型が
  `FnOnce` なので 2 回は評価できない。
- **Rust 標準ライブラリの `Vec::retain`**: `v.retain(f)` は、`v` の各要素に `f` を前から順に
  ちょうど 1 回ずつ適用し、`f` が偽を返した要素を `v` から取り除く。残る要素は値も相対順序も変わらない。
  この文書は `PendingRetains` の `retain` としてこれを使う -- `PendingRetains` は
  `Vec<PendingRetain>` の別名である (`CODE src/rc_ir/borrow.rs: PendingRetains`)。

## 2. 局所の定義

### DEF 訪問

走査は `cancel_body` の `analysis.walk(body, PendingRetains::default(), true)` から始まる
(`CODE src/rc_ir/borrow.rs: cancel`)。`B` の節点 `n` の**訪問**とは、`walk_inner(n, P, r)` の 1 回の
呼び出しをいう。`P` はその呼び出しに渡された `PendingRetains`、`r` は渡された `returns_from_func` である。

「訪問が**行う**呼び出し」とは、`walk_inner(n, P, r)` の本体が直接行う呼び出しだけをいう。継続の訪問や
アーム本体の訪問が行うものは含めない。

### DEF 呼び出しが名指すオブジェクト

- `consume_objects(pending, objects)` が**名指すオブジェクト**とは、`objects` の元の全体である
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)。
- `un_bump(pending, un_bumped)` が**名指すオブジェクト**とは、`un_bumped.objects()` の元の全体、すなわち
  `un_bumped` が個数を持つオブジェクトの全体である (`CODE src/rc_ir/borrow.rs: un_bump`,
  `CODE src/rc_ir/ownership.rs: References::objects`)。

### DEF 処分 leaf

`B` の各節点 `n` について、**処分 leaf の集合** `Disp(n) ⊆ VarPath` を次の表で定める。D9 の消費の表の
6 行と、D10 の `Release` の行が、参照を義務集合から取り除く構文のすべてであり、表の 7 行はその 7 つに
1 対 1 に対応する。表に現れない形の節点の `Disp` は空集合である。

| `n` | `Disp(n)` |
|---|---|
| `Let(x, App(callee, args), k)` | `(callee, λ)` for `λ ∈ L(callee)`、および `(a_i, λ)` for `λ ∈ L(a_i)` であって呼び出し先が第 `i` パラメータ `p_i` の unit `truncate_to_unit(ty(p_i), λ)` を所有する (D14) もの |
| `Let(x, Closure(f, caps), k)` | `(c, λ)` for `c ∈ caps`, `λ ∈ L(c)` |
| `Let(x, Llvm(gen, args), k)` | `(a_i, λ)` for `λ ∈ L(a_i)` であって、`borrows_operand(i)` が偽であり、かつ `result_prov` のどの結果 leaf も 1 元集合 `{Arg(i, λ)}` としては宣言していないもの |
| `Destructure(c, fs, s, k)`、`c` が boxed | `(c, λ)` for `λ ∈ L(c)` |
| `Destructure(c, fs, s, k)`、`c` が unbox | `(c, λ)` for `λ ∈ L(c)` であって `λ` の先頭の添字が `fs` の名前付きフィールドの添字でないもの |
| `B` の終端の `Ret(x)` | `(x, λ)` for `λ ∈ L(x)` |
| `Release(v, π, s, k)` | `(v, λ)` for `λ ∈ L(v, π)` |

D9 の表は「消費される leaf」を inhabited (D16) の限定なしに挙げ、D10 の `Release` の行は inhabited な
leaf だけを取り除く。`Disp` は inhabited でない leaf も入れる。A5 より inhabited でない leaf は参照を
持たないので、`Disp` は実行時に処分される参照が属する leaf の上位集合である。上位集合を取るのは、以下の
主張を強くする向きである。

**D9 の `App` の行が unit を取る型。** D9 の消費の表の `App` の行は「unit は**呼び出し先のパラメータの
型**で取る」と明記しており (`CODE src/rc_ir/ownership.rs: rhs_consumes`)、`ty(a_i) = ty(p_i)` は A12 の
「`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型」が与える。上の表の `App` の行は
その 2 つをそのまま読んだものである。

**D9 の終端の `Ret` の行の読み。** D9 は「関数本体の終端の `Ret(x)`」と書く。D12 はグローバル初期化子の
`init` を関数の本体と同じ資格で扱うので、`B` がグローバル初期化子の `init` であるときも、その終端の
`Ret` を同じ行で読む。

### DEF 触れうるオブジェクト

`B` の節点 `n` について、`Obj(n) = ⋃_{(w, λ) ∈ Disp(n)} acted_on(w, λ)` と定める。P7c の言明の
「その構文が触れうるオブジェクト (`acted_on`)」がこれである。

## 3. 証明する形

**P7c** (README 第 5 節)。

> - **(a)** 終端の `Ret` 以外では、その節点の訪問が行う `consume_objects` と `un_bump` の呼び出しが名指す
>   オブジェクトの**和**が、その構文が触れうるオブジェクト (D15 の `acted_on`) をすべて含む。とくに触れうる
>   オブジェクトが在れば、訪問はこの 2 つのどちらかを少なくとも 1 回呼ぶ。
> - **(b)** 終端の `Ret` では、その時点の `pending` のすべての要素を `needed_retains` に入れる。

この文書の記法で書くと次のとおりである。第 5 節が示すのはこの形である。

- **(a)** `n` が `B` の終端の `Ret` でないとき。その訪問が行う `consume_objects` と `un_bump` の呼び出しが
  名指すオブジェクトの和は、`Obj(n)` を含む。とくに `Obj(n)` が空でなければ、その訪問はこの 2 つの
  どちらかを少なくとも 1 回呼ぶ。
- **(b)** `n` が `B` の終端の `Ret` のとき。その訪問は、渡された `pending` の**すべての**要素を
  `needed_retains` に入れる。とくに `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素はすべて入る。

「その構文が触れうるオブジェクト (D15 の `acted_on`)」が `Obj(n)` であることは DEF 触れうるオブジェクトが、
「その時点の `pending`」が `pending(n)` であることは DEF 訪問が定める。

**和で述べる 2 つの場所。** README は「**和で述べるのは、1 回の呼び出しでは足りないからである。**」と書く。
その 2 つはこれであり、どちらも第 5 節の証明に現れる。

- **leaf ごとに呼ぶ構文。** `App`、`Closure`、`Destructure` の訪問は、`rhs_consumes` /
  `destructure_consumes` が挙げる leaf の 1 つずつについて `consume` を呼び、`consume` の 1 回の
  呼び出しが名指すのはその 1 つの leaf の `acted_on` だけである (`L4`)。パラメータ `a` の型が boxed な
  フィールドを 2 つ持つ unbox 構造体であり、呼び出し先がその 2 つの unit をどちらも所有する
  直接呼び出し `App(f, [a])` を考える。`f` は funptr の型を持ち、`is_funptr` の型は `is_fully_unboxed`
  が真なので boxed leaf を持たない (D4 の第 1 規則、`CODE src/ast/types.rs:
  TypeNode::is_fully_unboxed`)。よって `Disp(n) = {(a, [0]), (a, [1])}` であり、`origin` は
  パラメータについて `Exactly` を返すので `Obj(n) = {(a, [0]), (a, [1])}` である。訪問が行う 2 回の
  `consume_objects` は一方が `{(a, [0])}`、他方が `{(a, [1])}` を名指すので、どちらの引数も
  `Obj(n)` を含まない。
- **`Release` の 2 つの腕。** `Release(v, π)` の訪問は `Others(v, π)` を `consume_objects` へ、
  `ActRefs(v, π)` を `un_bump` へ渡す (`L5`)。`origin(v, λ)` が `Join` を返す leaf では、`ActRefs` は
  `identity` だけを数え (D15)、`other_objects` は `identity` と異なる候補だけを返す
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。第 7.5.7 節の `C1` の
  `Release(m, [])` がその形である -- `acted_on(m, []) = {(m, []), (p, []), (q, [])}` に対して
  `ActRefs(m, []).objects() = {(m, [])}`、`Others(m, []) = {(p, []), (q, [])}` であり、どちらの
  引数も `Obj(n)` を含まない。両者の**和**が含むことを述べるのが P5 (c) であり、`Release` の腕は
  その分担の上に立っている。

## 4. 予備の補題

### L1 (`walk` は `walk_inner` を 1 回呼ぶ)

**言明**。`CancelAnalysis::walk(node, pending, returns_from_func)` の 1 回の呼び出しは、
`CancelAnalysis::walk_inner(node, pending, returns_from_func)` をちょうど 1 回呼んでその値を返す。

**証明**

<1>1. `walk` の本体は `grow_stack(|| self.walk_inner(node, pending, returns_from_func))` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk
<1>2. `grow_stack(f)` の本体は `stacker::maybe_grow(64 * 1024, 1024 * 1024, f)` である。
  BY CODE src/misc.rs: grow_stack
<1>3. QED
  BY <1>1, <1>2, 外部の結果 stacker の `maybe_grow`

### L2 (走査の再帰)

**言明**。次の 2 つが成り立つ。

- **(i)** `B` の節点 `n` の訪問 `walk_inner(n, P, r)` が行う `walk` の呼び出しは、次のものだけである。
  `n` の式が `RcExpr::Ret` でないときは、`n` の継続 `k` について `walk(k, ·, r)` が 1 回。`n` の式が
  `RcExpr::Let(_, RcRhs::Match(_, arms), k)` のときは、それに加えて `arms` の各 `arm` について
  `walk(&arm.body, ·, false)` が 1 回ずつ。`n` の式が `RcExpr::Ret` のときは 1 回も呼ばない。
- **(ii)** `B` の各節点 `n` の訪問はちょうど 1 回起こる。その訪問へ至る `walk` の呼び出しの入れ子は、
  `B` の根から `n` への D2 の木の辺の列をちょうど辿る。

**証明**

<1>1. `B` の木の辺は次のものである。`Let(x, rhs, k)` から `k` へ、`rhs` が `Match(v, arms)` のときは
      さらに各 `arm.body` へ。`Retain(v, π, s, k)` / `Release(v, π, s, k)` / `Eval(v, k)` /
      `Destructure(c, fs, s, k)` から `k` へ。`Ret(v)` からはどこへも行かない。
  BY D2, CODE src/rc_ir/ast.rs: for_each_node_inner
  D2 は本体を、継続の辺と `Match` のアーム本体の辺だけを持つ木と定める。`for_each_node_inner` の `match`
  の 3 つの腕が辿る辺の集合がこれと一致する。

<1>2. `walk_inner(n, P, r)` の 7 つの腕が行う `walk` の呼び出しは次のとおりである。
  <2>1. `RcExpr::Retain(v, path, _, k)` の腕は `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `RcExpr::Release(v, path, _, k)` の腕は `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕は、`arms` の各 `arm` について
        `self.walk(&arm.body, pending.clone(), false)` を 1 回ずつ呼び、その後
        `self.walk(k, merged, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>4. `RcExpr::Let(x, rhs, k)` の腕 (`rhs` が `RcRhs::Match` でない場合) は
        `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>5. `RcExpr::Destructure(container, fields, _state, k)` の腕は
        `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>6. `RcExpr::Eval(_, k)` の腕は `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>7. `RcExpr::Ret(_)` の腕は `walk` を呼ばない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>8. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7
    `RcExpr` の 6 種のうち `Let` が 2 つの腕に分かれ、合わせて 7 つの腕である
    (`CODE src/rc_ir/ast.rs: RcExpr`)。

<1>3. (i) が成り立つ。
  BY <1>2, L1
  L1 より `walk(m, ·, ·)` の呼び出しは `m` の訪問 1 回に対応する。

<1>4. `B` の訪問が 1 回起こり、それは辺を 0 本辿ったものである。
  BY CODE src/rc_ir/borrow.rs: cancel, L1
  `cancel_body` が `analysis.walk(body, PendingRetains::default(), true)` を 1 回呼ぶ。

<1>4a. 走査で起こる訪問 -- DEF 訪問の意味の `walk_inner` の呼び出し -- と `walk` の呼び出しは、
       1 対 1 に対応する。`walk(m, P, r)` の呼び出しに対応する訪問は `m` の訪問 `walk_inner(m, P, r)`
       である。
  BY L1, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, DEF 訪問
  L1 より `walk` の 1 回の呼び出しは `walk_inner` をちょうど 1 回、同じ引数で呼ぶ。逆向きは、
  `walk_inner` を呼ぶ位置が `walk` の本体の
  `grow_stack(|| self.walk_inner(node, pending, returns_from_func))` ただ 1 か所だからである
  (`src/rc_ir/borrow.rs` に `walk_inner` の呼び出しはこの 1 か所しかない)。

<1>5. QED
  <1>1 と <1>3 と <1>4a より、`n` の訪問が起これば `n` から出る各辺の先の節点の訪問がちょうど 1 回ずつ
  起こり、それ以外の訪問は起こらない -- <1>3 は `walk` の呼び出しについて述べ、<1>4a がそれを訪問に
  読み替える。よって根からの辺の列と訪問が 1 対 1 に対応する。D2 より `B` は木なので、
  各節点への辺の列はちょうど 1 本であり、各節点の訪問はちょうど 1 回である。D2 より `B` は有限なので、
  この対応は辺の列の長さについての帰納で尽きる。基底は <1>4 と <1>4a である。
  BY <1>1, <1>3, <1>4, <1>4a, L1, D2

### L3 (`returns_from_func` が真になる節点)

**言明**。`B` の節点 `n` の訪問 `walk_inner(n, P, r)` について、`r` が真であることと、`n` が `B` の根から
継続の辺だけで到達できること (アーム本体の辺を 1 度も使わないこと) は同値である。とくに、`B` の終端の
`Ret` の訪問では `r` は真であり、アーム本体の中の `Ret` の訪問では `r` は偽である。

**証明**

<1>1. 根 `B` の訪問では `r` は真である。根へは辺を 0 本辿って到達する。
  BY CODE src/rc_ir/borrow.rs: cancel, L1

<1>2. 継続の辺で進む `walk` の呼び出しは `returns_from_func` に呼び出し元の `r` をそのまま渡し、アーム
      本体の辺で進む `walk` の呼び出しは `false` を渡す。
  BY L2

<1>3. `B` の根から `n` への辺の列の長さについての帰納法により、`r` が真であることと、その列が継続の辺
      だけからなることは同値である。
  <2>1. 基底 (長さ 0)。<1>1 より `r` は真であり、列は空なのでアーム本体の辺を含まない。
    BY <1>1
  <2>2. 帰納段。列の最後の辺が継続の辺のとき、<1>2 より `r` はその 1 つ前の節点の値を引き継ぐので、
        帰納法の仮定により、`r` が真であることと 1 つ前までの列が継続の辺だけからなることが同値であり、
        これは列全体についても同値である。列の最後の辺がアーム本体の辺のとき、<1>2 より `r` は偽であり、
        列はアーム本体の辺を含む。
    BY <1>2
  <2>3. QED
    BY <2>1, <2>2, L2
    L2 (ii) より、各節点の訪問はちょうど 1 回であり、その訪問へ至る呼び出しの入れ子は根からの辺の列に
    一致するので、`r` はその列だけで決まる。

<1>4. QED
  D3 より、`B` の実行路の最後の節点である終端の `Ret` は、`B` の根から継続の辺だけで到達する節点である
  (D3 の第 1 の規則が継続へ進み、第 2 の規則がアーム本体を辿った後に `Match` の継続 `k` へ戻る)。
  アーム本体の中の `Ret` へは、そのアーム本体の辺を少なくとも 1 度使う。
  BY <1>3, D3

### L4 (`consume` は leaf の `acted_on` を渡す)

**言明**。`CancelAnalysis::consume(pending, var, path)` は `consume_objects` を 1 回呼び、その呼び出しが
名指すオブジェクトは `acted_on(var, path)` である。

**証明**

<1>1. `consume` の本体は、`origin(self.vars, self.type_env, var, path).acted_on()` の元を複製して
      `Vec<VarPath>` にし、それを `objects` として `self.consume_objects(pending, &objects)` を呼ぶ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume

<1>2. `self.vars` と `self.type_env` は、`B` について `cancel_body` が作った `vars` と `type_env` である。
  BY CODE src/rc_ir/borrow.rs: cancel

<1>3. QED
  <1>1 の `objects` の元の全体は、第 1 節の記法と <1>2 より `acted_on(var, path)` である。DEF 呼び出しが
  名指すオブジェクトより、`consume_objects` の呼び出しが名指すオブジェクトは `objects` の元の全体である。
  BY <1>1, <1>2, DEF 呼び出しが名指すオブジェクト

### L5 (`Release` の訪問が行う呼び出し)

**言明**。`n = Release(v, π, s, k)` の訪問は、次の順で呼び出しを行う。

1. `others = self.other_objects(v, π)` を求め、`self.consume_objects(&mut pending, &others)` を呼ぶ。
   `others` の元の全体は `Others(v, π)` である。
2. `un_bumped = self.acted_references(v, π)` を求め、`un_bump(&mut pending, &un_bumped)` を呼ぶ。
   `un_bumped` は `ActRefs(v, π)` である。
3. `un_bump` が `UnBump::OutsideBracket` を返したときにかぎり、`self.consume_objects(&mut pending,
   &un_bumped.objects())` を呼ぶ。`UnBump::InBracket` と `UnBump::NoBracket` のときは `consume_objects` も
   `un_bump` もこれ以上呼ばない (`InBracket` の腕が行うのは `un_bump_releases` への記録だけである)。
4. `self.walk(k, pending, returns_from_func)` を呼ぶ。

**証明**

<1>1. 訪問は 1、2、4 の呼び出しをこの順で行い、2 の返り値による 3 つの腕のうち `UnBump::OutsideBracket`
      の腕だけが `self.consume_objects(&mut pending, &objects)` を呼ぶ (`objects` は
      `un_bumped.objects()`)。`UnBump::InBracket(retain)` の腕は `self.un_bump_releases` への `push` だけ
      を行い、`UnBump::NoBracket` の腕は何も行わない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕

<1>2. `other_objects(v, π)` の返り値の元の全体は `Others(v, π)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects, 第 1 節の記法

<1>3. `CancelAnalysis::acted_references(v, π)` は `acted_references(self.vars, self.type_env, v, π)` の
      値をそのまま返す (空でないことを表明した後で)。`self.vars` と `self.type_env` は `B` のものである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references, CODE src/rc_ir/borrow.rs: cancel

<1>4. QED
  BY <1>1, <1>2, <1>3, 第 1 節の記法

### L6 (`Release` の訪問の後に `pending` に残るもの)

**言明**。`n = Release(v, π, s, k)` の訪問において、`un_bump` の呼び出しが `UnBump::NoBracket` または
`UnBump::OutsideBracket` を返すならば、訪問がその後 `self.walk(k, pending, ·)` に渡す `pending` の
どの要素も、`Obj(n)` のどのオブジェクトも名指さない。

この補題は P7c の言明には無い。P18a が要る形として置く。

**証明**

<1>1. `consume_objects(pending, objects)` の呼び出しの後、`pending` のどの要素も `objects` のどの
      オブジェクトも名指さない。またこの呼び出しは `pending` の要素を落とすだけで、残る要素の
      `outstanding` を変えない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, 外部の結果 `Vec::retain`
  本体は `pending.retain(...)` であり、述語は `objects` のいずれかを `outstanding.names` が真とする要素に
  ついて `false` を返す。`retain` は `false` を返した要素を落とし、他の要素はそのまま残す。

<1>2. `Obj(n) ⊆ ActRefs(v, π).objects() ∪ Others(v, π)` である。
  BY P5, DEF 処分 leaf, DEF 触れうるオブジェクト
  DEF 処分 leaf の `Release` の行と DEF 触れうるオブジェクトより
  `Obj(n) = ⋃_{λ ∈ L(v, π)} acted_on(v, λ)` である。P5 (c) が、`ActRefs(v, π).objects()` と
  `other_objects(v, π)` の和が `π` の下の各 boxed leaf `λ` について `origin(v, λ).acted_on()` をすべて
  含むと述べる。`π` の下の boxed leaf の全体は `L(v, π)` である (第 1 節の記法)。

<1>3. `un_bumped.objects()` の元の全体は `ActRefs(v, π).objects()` の元の全体である。
  BY L5

<1>4. CASE `un_bump` が `UnBump::NoBracket` を返す。
  <2>1. `un_bump` は `pending` を変えずに返る。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `NoBracket` を返すのは `pending.iter().rposition(...)` が `None` を返した枝であり、そこまでに
    `pending` への書き込みは無い。
  <2>2. `pending` のどの要素 `r` についても `r.outstanding.shares_an_object(un_bumped)` は偽である。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `Iterator::rposition` が `None` を返すのは、述語がどの要素についても偽のときである。
  <2>3. `r.outstanding.shares_an_object(un_bumped)` が偽であることは、`un_bumped.objects()` のどの
        オブジェクト `o` についても `r.outstanding.names(o)` が偽であることと同値である。
    BY CODE src/rc_ir/ownership.rs: References::shares_an_object, References::objects, References::names
    `shares_an_object` は `other.0.keys().any(|object| self.0.contains_key(object))`、`objects` は
    `self.0.keys()` の複製、`names` は `self.0.contains_key` である。
  <2>4. 訪問が `walk(k, pending, ·)` に渡す `pending` は、L5 の 1 の呼び出しの後で、かつ <2>1 により
        変わらなかったものである。
    BY L5, <2>1
    L5 の 3 はこの場合には呼び出しを行わない。
  <2>5. QED
    その `pending` のどの要素も、<2>2 と <2>3 より `un_bumped.objects()` のどのオブジェクトも名指さず、
    <1>1 より `Others(v, π)` のどのオブジェクトも名指さない (L5 の 1 の `others` の元の全体が
    `Others(v, π)` である)。<1>3 より前者は `ActRefs(v, π).objects()` であり、<1>2 より
    `Obj(n)` はこの 2 つの和に含まれる。
    BY <1>1, <1>2, <1>3, <2>2, <2>3, <2>4, L5

<1>5. CASE `un_bump` が `UnBump::OutsideBracket` を返す。
  <2>1. `un_bump` は `pending` を変えずに返る。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `OutsideBracket` を返すのは `!innermost.outstanding.covers(un_bumped)` の枝であり、そこまでに
    `pending` への書き込みは無い (`&mut pending[index]` を取るだけである)。
  <2>2. 訪問は続けて `self.consume_objects(&mut pending, &un_bumped.objects())` を呼び、その後
        `walk(k, pending, ·)` を呼ぶ。
    BY L5
  <2>3. <2>2 の `consume_objects` の後、`pending` のどの要素も `un_bumped.objects()` のどのオブジェクトも
        名指さない。
    BY <1>1, <2>2
  <2>4. <2>2 の `consume_objects` の後も、`pending` のどの要素も `Others(v, π)` のどのオブジェクトも
        名指さない。
    BY <1>1, <2>1, <2>2, L5
    L5 の 1 の呼び出しの後にこの性質が成り立ち (<1>1、`others` の元の全体は `Others(v, π)`)、その後に
    `pending` に起きたのは <2>1 の「変えない」と <2>2 の `consume_objects` による要素の除去だけである。
    要素を落とす操作は「どの要素も名指さない」を壊さない (<1>1 の後半より、残る要素の `outstanding` は
    変わらない)。
  <2>5. QED
    <2>3 と <1>3 より、その `pending` のどの要素も `ActRefs(v, π).objects()` のどのオブジェクトも
    名指さない。<2>4 より `Others(v, π)` のどのオブジェクトも名指さない。<1>2 より `Obj(n)` はこの
    2 つの和に含まれる。
    BY <1>2, <1>3, <2>2, <2>3, <2>4

<1>6. QED
  BY <1>4, <1>5
  言明が `un_bump` の返り値を `NoBracket` か `OutsideBracket` かに限っており、この 2 つが場合を尽くす。

### P7f (L6 から)

**言明** (README 第 5 節)。「`Release` の訪問について、`un_bump` が `NoBracket` か `OutsideBracket` を
返したとき、その訪問の後の `pending` のどの要素の `outstanding` も、その `Release` が触れうるオブジェクトの
どれも名指さない。」

**証明**

<1>1. QED
  BY L6, L5, DEF 触れうるオブジェクト, DEF 処分 leaf, P7c
  L6 の言明は「`n = Release(v, π, s, k)` の訪問において、`un_bump` の呼び出しが `UnBump::NoBracket` または
  `UnBump::OutsideBracket` を返すならば、訪問がその後 `self.walk(k, pending, ·)` に渡す `pending` の
  どの要素も、`Obj(n)` のどのオブジェクトも名指さない」である。「訪問の後の `pending`」とは `walk(k, ·, ·)`
  に渡すものである (L5 の 4)。「その `Release` が触れうるオブジェクト」とは、DEF 触れうるオブジェクトと
  DEF 処分 leaf の `Release` の行より `Obj(n) = ⋃_{λ ∈ L(v, π)} acted_on(v, λ)` である。これは P7c の
  言明が「その構文が触れうるオブジェクト」と呼ぶものと同じ量である。「要素が名指す」とは
  `outstanding.names` が真であることである (L6 の <1>1)。よって 2 つの言明は同じことを述べている。

### (R) 出力の本体における呼び出し先の解決

**言明**。`prog` を `borrow_ify` の出力とする。`prog` の 1 つの版 `V` の本体の `App` 節点
`n = Let(x, App(c, args), k)` と、`V` の本体の `VarTable` `vars_V` について、
`resolve_callee_params(c, vars_V, prog)` が `Some(params)` を返すならば、`params` は `n` の段の実行時の
呼び出し先 (D23) のパラメータの列である。

**これはこの文書が引く前提であって、この文書が示すものではない。** README の P29 が同じ言明を
`borrow_ify` の**入力**について述べ、「`resolve_callee_params` が `None` を返す場合について何も言わないのは、
そのとき `rhs_consumes` が全位置を所有として扱う -- 安全側 -- からである。出力についての同じ性質は、P9 と
P12 と合わせて読む」と続ける。P9・P12・P24 が出力について与えるのは次の 3 つである。

- 出力の各 `App` 節点は入力のちょうど 1 つの `App` 節点から作られ、その callee は `route` がその節点の
  callee から作った `RcVar` である。ほかの節点は名前替えのほかは変わらない (P24、P9)。
- `route` が返す呼び出し先は元の呼び出し先と同じ関数の版であり、呼び出し先が入力の関数を名指すときは
  返る名前は出力の `funcs` の鍵である。局所変数を経由する間接呼び出しでは `route` は呼び出し先を
  そのまま返し、その名前はどちらの `funcs` の鍵でもない (P12)。
- `Closure` を右辺とする `Let` 節点は `FuncRef` を変えずに複製されるので、`vars_V.closure_targets` は
  入力の同じ写像を鍵について名前替えで写したものであり、値の `FuncRef` は入力のものと同じである
  (P24、P9、`CODE src/rc_ir/ownership.rs: collect_bindings`,
  `CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner`)。

この 3 つは、出力の `App` 節点の callee が**どの名前であるか**を決める。残るのは、**その名前が実行時に
指す関数が、そのプログラムの `funcs` のその名前の関数であること**である。P29 はそれを入力について述べ、
その文は入力の `funcs` を読む。出力の実行についての同じ文を与える者が README に無い。最後の
「新しく差し戻す点」の節がこれを差し戻す。

第 5 節の `<1>8` と第 7.5.5 節の `L16` がこれを引く。

## 5. P7c の証明

<1>1. `B` の各節点 `n` の訪問はちょうど 1 回起こる。
  BY L2

<1>2. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9, D10
    DEF 処分 leaf の表に `Retain` の行は無い。D9 の消費の表にも移動の表にも `Retain` の行は無く、D9 の
    最後の段落が `Retain` は D10 が直接扱うと述べる。D10 の `Retain` の行は参照を**加える**。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
    D2 より `Retain` と `Ret` は `RcExpr` の相異なる種である。
  <2>3. QED
    空集合はどんな和にも含まれる。よって P7c (a) が成り立つ。
    BY <2>1, <2>2

<1>3. CASE `n` の式が `RcExpr::Eval(v, k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    DEF 処分 leaf の表に `Eval` の行は無い。D9 の最後の段落が「`Eval(v, k)` … は、参照を作らず、移さず、
    手放さない」と述べる。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>4. CASE `n` の式が `RcExpr::Let(x, RcRhs::Var(y), k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    DEF 処分 leaf の表に `Var` の行は無い。D9 の移動の表の第 1 行が `Let(x, Var(y), k)` を移動とし、
    消費の表に `Var` の行は無い。移動は義務集合を変えない。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>5. CASE `n` の式が `RcExpr::Let(x, RcRhs::Match(v, arms), k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    DEF 処分 leaf の表に `Match` の行は無い。D9 の最後の段落が「`Let(x, Match(v, arms), k)` の `Match`
    節点自身は、参照を作らず、移さず、手放さない」と述べる。アームの payload 束縛とアーム本体の `Ret` は
    この節点ではなくアームの側の構文であり、それぞれ D10 の生成の表と D9 の移動の表が扱う。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>6. CASE `n` の式が `RcExpr::Ret(v)` であり、`n` が `B` の終端の `Ret` でない。
  <2>1. `B` の継続の鎖 -- `B` の根から始まる鎖と、各 `Match` の各アーム本体の根から始まる鎖 -- は、
        どれもちょうど 1 つの `Ret` で終わり、その鎖の他の節点は `Ret` ではない。`B` の各 `Ret` 節点は、
        ちょうど 1 つの鎖の最後の節点である。
    BY D2
    D2 より `Ret` は継続を持たない唯一の種であり、他の 5 種はちょうど 1 つの継続を持つ。D2 より `B` は
    有限なので、各鎖は有限であり、`Ret` で終わる。
  <2>2. `B` の根から始まる鎖の最後の `Ret` が `B` の終端の `Ret` である。
    BY D3, <2>1
    D3 の第 1 の規則が継続へ進み、第 2 の規則がアーム本体を辿った後に `Match` の継続へ戻るので、実行路の
    最後の節点は `B` の根から始まる鎖の最後の節点である。
  <2>3. `n` はあるアーム本体の根から始まる鎖の最後の `Ret` である。
    BY <2>1, <2>2
    仮定より `n` は `B` の終端の `Ret` ではないので、<2>2 より `B` の根から始まる鎖の最後の節点ではなく、
    <2>1 よりその鎖の他の位置にも `Ret` は無い。<2>1 より `n` はどれか 1 つの鎖の最後の節点なので、
    その鎖はアーム本体の根から始まる鎖である。
  <2>4. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9, <2>3
    DEF 処分 leaf の `Ret` の行は `B` の終端の `Ret` だけを挙げる。D9 の移動の表の第 2 行が、アーム本体の
    `Ret(x)` を移動とする。
  <2>5. QED
    BY <2>4

<1>7. CASE `n` の式が `RcExpr::Ret(v)` であり、`n` が `B` の終端の `Ret` である。
  <2>1. `n` の訪問 `walk_inner(n, P, r)` では `r` が真である。
    BY L3
  <2>2. `RcExpr::Ret(_)` の腕は、`returns_from_func` が真のとき `pending` の各要素 `retain` について
        `self.needed_retains.insert(retain.node)` を行う。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. QED
    <2>1 と <2>2 より、渡された `pending` のすべての要素が `needed_retains` に入る。部分集合として、
    `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素も入る。これが P7c (b) である。
    BY <2>1, <2>2

<1>8. CASE `n` の式が `RcExpr::Let(x, RcRhs::App(callee, args), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
    `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕が先に置かれているので、右辺が `Match` でない `Let` は
    次の `RcExpr::Let(x, rhs, k)` の腕に来る。
  <2>2. `consume_rhs(pending, rhs, result_ty)` は、空の `consumed` を作り、`rhs_consumes(rhs, result_ty,
        self.vars, self.prog, self.type_env, &owns, &mut consumed)` を呼び、`consumed` の各元
        `(var, leaf)` について `self.consume(pending, &var, &leaf)` を呼ぶ。`consumed` は `rhs_consumes`
        の `out` 引数なので、その元の全体は `rhs_consumes` が `out` に入れた元の全体である。ここで `owns` は
        `|p, leaf| self.owned_units.contains(&(p.name.clone(), truncate_to_unit(&p.ty, leaf, self.type_env)))`
        である。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>3. `self.owned_units` は `all_owned_units(prog, type_env)` である。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>4. `rhs_consumes` の `RcRhs::App(callee, args)` の腕は、`callee` の各 boxed leaf `λ` について
        `(callee.name, λ)` を `out` に入れ、各 `i` と `a_i` の各 boxed leaf `λ` について
        `is_owning_position` が真ならば `(a_i.name, λ)` を `out` に入れる。`is_owning_position` は、
        `resolve_callee_params(callee, vars, prog)` が `Some(params)` を返すときは `owns(&params[i], &λ)`、
        `None` を返すときは真である。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/ownership.rs: push_boxed_leaves, A14
    `push_boxed_leaves(var, ty, type_env, out)` は `boxed_leaf_paths(ty, type_env)` の各元 `p` について
    `(var.clone(), p)` を `out` に入れる。A14 より `args.len() <= params.len()` なので `params[i]` は
    範囲内である。
  <2>5. `Disp(n)` は <2>4 が `out` に入れる集合に含まれる。
    <3>1. `Disp(n)` の `callee` の側 -- `(callee, λ)` for `λ ∈ L(callee)` -- は <2>4 の第 1 の部分に
          等しい。
      BY <2>4, DEF 処分 leaf
      第 1 節の記法より `L(callee) = boxed_leaf_paths(ty(callee), type_env)` である。
    <3>2. CASE `resolve_callee_params` が `None` を返す。
      <4>1. <2>4 より `is_owning_position` はすべての `(i, λ)` について真なので、<2>4 の第 2 の部分は
            `(a_i, λ)` for `λ ∈ L(a_i)` の全体である。
        BY <2>4
      <4>2. QED
        `Disp(n)` の引数の側はその部分集合である。
        BY <4>1, DEF 処分 leaf
    <3>3. CASE `resolve_callee_params` が `Some(params)` を返す。
      <4>1. `params` は、`resolve_callee_params` が `vars.closure_targets` か `prog.funcs` の鍵から
            静的に引いた関数 `prog.funcs[fref]` の `params` である。
        BY CODE src/rc_ir/ownership.rs: resolve_callee_params
      <4>1a. `n` の段の**実行時の**呼び出し先 (D23) を `g` と書くと、`g` は `prog` の `funcs` の関数で
             あり、そのパラメータの列は `params` である。
        BY (R), <2>4, <4>1, D23
        第 1 節より `B` は `cancel` の入力、すなわち `borrow_ify` の出力の本体であり、`prog` はその
        プログラムである。<2>4 と <4>1 より `params` は `resolve_callee_params(callee, vars, prog)` の
        返り値であり、(R) はそれが実行時の呼び出し先のパラメータの列であることを述べる。`g` が `prog` の
        `funcs` の関数であることは D23 の「D9 の `App` の行が読む所有は D14 が
        `RcFunc::borrowed_units` から定めるものなので、**その呼び出し先はプログラムの `funcs` の関数で
        ある**」による。
      <4>2. `Disp(n)` の引数の側の元 `(a_i, λ)` とは、`λ ∈ L(a_i)` であって、`g` が `g` の第 `i`
            パラメータ `p_i = params[i]` の unit `u = truncate_to_unit(ty(p_i), λ, type_env)` を所有する
            ものである。また A12 より `ty(a_i) = ty(p_i)` であり、`λ` は `ty(p_i)` の boxed leaf である。
        BY DEF 処分 leaf, A12, <4>1a
        DEF 処分 leaf の `App` の行は D9 の `App` の行を読んだものであり、そこで言う「呼び出し先」は
        実行時のもの `g` である (D23)。<4>1a より `g` のパラメータの列は `params` なので、その第 `i`
        パラメータは `params[i]` である。A12 は「`App(callee, args)` の各引数と呼び出し先の対応する
        パラメータの型」の一致を挙げる。
      <4>3. D14 より、`g` が `u` を所有するとは、`u` が `rc_units(ty(p_i))` の元であって
            `(p_i.name, u)` が `g` の `borrowed_units` に入らないことである。
        BY D14
      <4>4. `u = truncate_to_unit(ty(p_i), λ, type_env)` は `rc_units(ty(p_i))` の元である。
        BY P1, A10, <4>2
        P1 は、**A10 を満たす**任意の型 `τ` の各 boxed leaf の `truncate_to_unit(τ, ・)` が
        `rc_units(τ)` の元であると述べる。`ty(p_i)` は `prog` の関数のパラメータの型、すなわち
        プログラムに現れる型なので A10 を満たす -- A10 は「プログラムに現れる型は ground であり、その
        tycon は `type_env` にあり、`no_size_in_place` の in-place の降下は有限である」と述べ、
        「P1 の定義域はこの広い方であり、型の歩みを扱う命題が P1 を部分木の型に当てるのはこの節に
        よる」と続ける。<4>2 より `λ` は `ty(p_i)` の boxed leaf である。
      <4>5. `all_owned_units(prog, type_env)` は、`prog` の各関数の各パラメータ・capture `p` と各
            `u' ∈ rc_units(ty(p))` について、`(p.name, u')` がその関数の `borrowed_units` に入らない
            ならばそれを集合に入れる。
        BY CODE src/rc_ir/ownership.rs: all_owned_units
      <4>6. `Disp(n)` の引数の側の元 `(a_i, λ)` について `owns(&params[i], &λ)` は真である。
        <4>1a より `g` は `prog.funcs` の関数であり、`p_i = params[i]` はそのパラメータである。<4>3 と
        <4>4 より `u ∈ rc_units(ty(p_i))` かつ `(p_i.name, u)` は `g` の `borrowed_units` に入らないので、
        <4>5 より `(p_i.name, u) ∈ all_owned_units(prog, type_env)` である。<2>2 と <2>3 より
        `owns(&params[i], &λ)` はまさにこの所属を検査する。
        BY <2>2, <2>3, <4>1a, <4>2, <4>3, <4>4, <4>5
      <4>7. QED
        BY <2>4, <4>6
        `is_owning_position` が真になるので、`(a_i.name, λ)` は `out` に入る。
    <3>4. QED
      BY <3>1, <3>2, <3>3
      `resolve_callee_params` の返り値は `None` か `Some` かのどちらかであり、この 2 つで尽きる
      (`Option` の 2 種)。`Disp(n)` は `callee` の側と引数の側の和であり、前者は <3>1、後者は <3>2 と
      <3>3 が扱う。
  <2>6. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>4 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, <2>4, L4
    <2>2 より `consumed` の各元について `consume` が呼ばれ、L4 よりその各呼び出しは `acted_on(w, λ)` を
    名指す `consume_objects` を 1 回行う。
  <2>7. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>8. QED
    <2>5 より `Disp(n)` は <2>4 の集合に含まれるので、`Obj(n) = ⋃_{(w, λ) ∈ Disp(n)} acted_on(w, λ)` は
    <2>6 の和に含まれる。これが P7c (a) である。
    BY <2>5, <2>6, <2>7, DEF 触れうるオブジェクト

<1>9. CASE `n` の式が `RcExpr::Let(x, RcRhs::Closure(f, caps), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び、`consume_rhs` は空の
        `consumed` を `rhs_consumes` の `out` 引数として渡した後、`consumed` の各元 `(var, leaf)` に
        ついて `self.consume(pending, &var, &leaf)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>2. `rhs_consumes` の `RcRhs::Closure(_, caps)` の腕は、`caps` の各 `c` の各 boxed leaf `λ` について
        `(c.name, λ)` を `out` に入れる。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/ownership.rs: push_boxed_leaves
  <2>3. `Disp(n)` は <2>2 が `out` に入れる集合に等しい。
    BY <2>2, DEF 処分 leaf
    DEF 処分 leaf の `Closure` の行は `(c, λ)` for `c ∈ caps`, `λ ∈ L(c)` であり、第 1 節の記法より
    `L(c) = boxed_leaf_paths(ty(c), type_env)` である。
  <2>4. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>2 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, L4
  <2>5. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>6. QED
    BY <2>3, <2>4, <2>5, DEF 触れうるオブジェクト

<1>10. CASE `n` の式が `RcExpr::Let(x, RcRhs::Llvm(gen, args), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼ぶ。`consume_rhs` は
        `rhs_consumes` に `result_ty` として `x.ty` を渡し、空の `consumed` を `out` 引数として渡した
        後、`consumed` の各元 `(var, leaf)` について `self.consume(pending, &var, &leaf)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>2. `rhs_consumes` の `RcRhs::Llvm(llvm_gen, args)` の腕は、
        `passthrough = passthrough_arg_leaves(&**llvm_gen, result_ty, args, type_env)` を求め、各 `i` に
        ついて `llvm_gen.borrows_operand(i, &arg_tys, type_env)` が真ならその `i` を飛ばし、偽なら
        `a_i` の各 boxed leaf `λ` について `passthrough` が `(i, λ)` を含まないときにかぎり
        `(a_i.name, λ)` を `out` に入れる。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes
  <2>3. `passthrough_arg_leaves(llvm_gen, result_ty, args, type_env)` が返す集合は、
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の結果 leaf のうち、その `LeafOrigins` が
        ちょうど 1 元であってその元が `LeafOrigin::Arg(j, σ)` であるものについての `(j, σ)` の全体である。
    BY CODE src/rc_ir/ownership.rs: passthrough_arg_leaves,
       CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/provenance.rs: Provenance::leaves
    `leaves()` は各 boxed leaf の `LeafOrigins` を渡し、`as_arg_projection` は元の個数が 1 でない集合と、
    唯一の元が `Fresh` か `Unknown` である集合に `None` を返す。
  <2>4. `Disp(n)` は <2>2 が `out` に入れる集合に等しい。
    BY <2>2, <2>3, DEF 処分 leaf
    DEF 処分 leaf の `Llvm` の行の条件「`borrows_operand(i)` が偽であり、かつ `result_prov` のどの結果
    leaf も 1 元集合 `{Arg(i, λ)}` としては宣言していない」は、<2>3 により <2>2 の条件
    「`borrows_operand(i)` が偽であり、かつ `passthrough` が `(i, λ)` を含まない」に一致する。
  <2>5. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>2 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, L4
  <2>6. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>7. QED
    BY <2>4, <2>5, <2>6, DEF 触れうるオブジェクト

<1>11. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。
  <2>1. `n` の訪問は、`destructure_consumes(container, fields, self.type_env)` の各元 `leaf` について
        `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `destructure_consumes(container, fields, type_env)` は、`container.ty.is_box(type_env)` が真の
        とき `boxed_leaf_paths(container.ty, type_env)` をそのまま返し、偽のときはそのうち先頭の添字が
        `fields` の名前付きフィールドの添字でないものだけを返す。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes
  <2>3. `(container.name, leaf)` for `leaf ∈ destructure_consumes(container, fields, type_env)` の全体は
        `Disp(n)` に等しい。
    BY <2>2, DEF 処分 leaf
    DEF 処分 leaf の `Destructure` の 2 行が、boxed と unbox のそれぞれについて <2>2 と同じ集合を挙げる。
  <2>4. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、
        `⋃_{leaf ∈ destructure_consumes(...)} acted_on(container.name, leaf)` を含む。
    BY <2>1, L4
  <2>5. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>6. QED
    BY <2>3, <2>4, <2>5, DEF 触れうるオブジェクト

<1>12. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。
  <2>1. `n` の訪問は `self.consume_objects(&mut pending, &others)` を呼び、その後
        `un_bump(&mut pending, &un_bumped)` を呼ぶ。`others` の元の全体は `Others(v, path)` であり、
        `un_bumped` は `ActRefs(v, path)` である。
    BY L5
  <2>2. この 2 つの呼び出しが名指すオブジェクトの和は `Others(v, path) ∪ ActRefs(v, path).objects()` に
        等しい。
    BY <2>1, DEF 呼び出しが名指すオブジェクト
  <2>3. `Obj(n) = ⋃_{λ ∈ L(v, path)} acted_on(v, λ)` である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト
  <2>4. `Obj(n) ⊆ ActRefs(v, path).objects() ∪ Others(v, path)` である。
    BY P5, <2>3
    P5 (c) が、`ActRefs(v, π).objects()` と `other_objects(v, π)` の和が `π` の下の各 boxed leaf `λ` に
    ついて `origin(v, λ).acted_on()` をすべて含むと述べる。`π` の下の boxed leaf の全体は `L(v, path)` で
    ある (第 1 節の記法)。
  <2>5. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>6. QED
    <2>1 の 2 つの呼び出しは、訪問が行う `consume_objects` と `un_bump` の呼び出しのうちの 2 つである
    (L5 によれば 3 つ目がありうる)。よって訪問が行う呼び出しが名指すオブジェクトの和は <2>2 の集合を
    含み、<2>4 よりその集合は `Obj(n)` を含む。これが P7c (a) である。
    BY <2>1, <2>2, <2>4, <2>5, L5

<1>13. QED
  <2>1. `RcExpr` はちょうど 6 種を持つ。`Let`、`Retain`、`Release`、`Destructure`、`Eval`、`Ret`。
    BY CODE src/rc_ir/ast.rs: RcExpr
  <2>2. `RcRhs` はちょうど 5 種を持つ。`Var`、`App`、`Closure`、`Llvm`、`Match`。
    BY CODE src/rc_ir/ast.rs: RcRhs
  <2>3. `Retain` は <1>2、`Eval` は <1>3、`Destructure` は <1>11、`Release` は <1>12 が扱う。
    BY <1>2, <1>3, <1>11, <1>12
  <2>4. `Let` は右辺の 5 種で分かれ、`Var` は <1>4、`Match` は <1>5、`App` は <1>8、`Closure` は <1>9、
        `Llvm` は <1>10 が扱う。
    BY <2>2, <1>4, <1>5, <1>8, <1>9, <1>10
  <2>5. `Ret` は「`n` が `B` の終端の `Ret` である」の真偽で 2 つに分かれ、偽は <1>6、真は <1>7 が扱う。
        この 2 つで尽きるのは排中律による。
    BY <1>6, <1>7
  <2>6. QED
    <1>1 より `B` の各節点の訪問が起こり、<2>1 から <2>5 より、どの訪問も <1>2 から <1>12 のちょうど
    1 つの場合に入る。`n` が `B` の終端の `Ret` である場合 (<1>7) は P7c (b) を、他の場合は
    P7c (a) を示している。
    BY <1>1, <2>1, <2>2, <2>3, <2>4, <2>5

## 6. README を読み直した結果

**この文書がかつて差し戻した 6 点は、どれも README にすでに在る。** 対象コミットの README を
読み直して確かめた。

| かつての差し戻し | 主張 | README の現在の文 |
|---|---|---|
| 1 | P7c の言明が名指す仕組みは 2 つで足りない | P7c は「**(a)** 終端の `Ret` 以外では … **(b)** 終端の `Ret` では、その時点の `pending` のすべての要素を `needed_retains` に入れる」と 2 節に分けている |
| 9 | P7c (a) が「1 回の呼び出しの引数がすべて含む」形であり、その形は偽である | P7c (a) は「その節点の訪問が行う `consume_objects` と `un_bump` の呼び出しが名指すオブジェクトの**和**が … すべて含む」と和の形で書き、「**和で述べるのは、1 回の呼び出しでは足りないからである。**」と理由も添えている。第 5 節が示すのはこの文そのものである |
| 2 | P7c が `acted_on` の出典を D13 と書いている | P7c (a) は「その構文が触れうるオブジェクト (**D15 の `acted_on`**)」と書いている |
| 3 | D9 の `App` の行が unit を取る型を書いていない / A12 に引数とパラメータの型の一致が無い | D9 の `App` の行は「unit は**呼び出し先のパラメータの型**で取る」と書き、A12 は「**`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型**」を挙げている |
| 4 | L6 を README の命題に上げるか | **P7f** として在る。この文書の第 4 節が L6 を証明し、その直後の段が L6 と P7f の一致を述べる |
| 5 | `stacker::maybe_grow` を README に置くか | **A15** として在る -- 「`grow_stack(f)` は `f` をちょうど 1 回呼び、その返り値を返す」 |

第 3 の項について補う。この文書の第 2 節の `Disp` の表の `App` の行は、D9 のその行と A12 のその項を
そのまま読んだものであり、第 5 節の `<1>8` はその 2 つを引く。第 1 節が外部の結果として述べた
`stacker::maybe_grow` は A15 と同じ事実だが、A15 は `grow_stack` の水準で述べるので、`L1` は A15 を
直接引ける形にしてある。

## 7. P18b と P18a

**結論を先に書く。P18b は A1・A2・D12 から証明できた。P18a は A19 と P14a の上で証明できた。**

P18b は第 7.4 節が証明する。証明の要は補題 `L9` -- `origin` の `identity` が等しい 2 つの leaf は、
実行時に同時に inhabited (D16) であるか同時にそうでないか、のどちらかである -- であり、これが
「`outstanding` は静的な数え上げを引き、実行時は inhabited な分だけが上下する」というずれを消す。

P18a は A1・A2・D12 だけからは出ない。第 7.5 節が README の A19 を引く。A19 が要ることは D12 を満たす
2 つの反例 `C1` と `C2` が示す (7.5.7 と 7.5.8)。どちらでも `cancel` が対を消して解放後の読みを作るので、
この 2 つは P21 と P23 の反例でもある。どちらも `insert_rc` の出力ではない (README の A19 が引く
`p60-insert-rc.md` の `L10`)。A19 を果たす者のうち `borrow_ify` の側は `L16` が示す。

**A19 (i) が `H(O)` の側を与える。** (i) は、各類について
`d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置いたときの `Σ d(C)` と、借用終端の類が
在るときの `+1` の和で `H(O)` を下から抑える。(ii-a)・(ii-b)・P14a が各類について `d(C) ≥ bumps` を与え、
`bumps` が正である類 1 つがそこに `+1` を足すので、2 つを繋ぐと `H(O) ≥ N + 1` になる。第 7.5.4 節が、
その類が借用終端であるかどうかで場合を分けて書く。

### 7.1 局所の定義

この節から先で使う補題を `L7` から `L17` と番号を付ける (`L14a` と `L15` は使わない)。後から挟んだものには
`L8a` のように枝番を振る。`p60-insert-rc.md` の補題の名前は、README の A19 の文面を引用する箇所にだけ現れ、そこでは
`p60` の `L9` のように書く。この文書の `BY` の行はそれらを引かない。

#### 固定するもの

以下では、本体 `B` の 1 つの実行路 `ρ` (D3) と、`ρ` を辿る 1 回の活性化 (D21) を固定する。値、
inhabited (D16)、参照カウント `H` (D7)、義務集合 `Obl` (D10) はいずれも活性化についての量であり、
D21 の最後から 2 つ目の段落より、「`ρ` の上で」と書いたものは `ρ` を辿るすべての活性化についての主張と
して読む。D21 より、`Let(x, Match(v, arms), k)` で活性化が選ぶアームは、`tag` が `v` の値の実行時のタグに
等しいアームである。

**inhabited の読み。** この文書が leaf について「inhabited」と書くとき、D16 の条件に加えて、A5 の例外 --
capture が空のクロージャの capture の leaf は null であり、A5 はそれを inhabited でない leaf と同じに
扱う -- を織り込む。すなわち、null の leaf は inhabited でない。

#### 計数下は活性化の間 変わらない

D26 の最後の段落がこれを述べる。「**1 つの活性化の間、そこに現れるオブジェクトが計数下であるかどうかは
変わらない。** `mark_global` の呼び出しはコード生成に 1 か所しかなく、グローバル初期化子の本体を評価した
結果に対してだけ走る … 命題が『各時点の計数下オブジェクト』を量化するとき、その集合は活性化の間ずっと
同じである。」よってこの文書は「計数下」と書くときに時点を添えない。

#### DEF 実行時の作用

実行路 `ρ` と、`ρ` の上の節点 `n` を固定する。`n` が `Retain(v, π, s, k)` または `Release(v, π, s, k)`
であるとき、

- `Inh_ρ(v, π, n)` を、`L(v, π)` の元のうち、`n` の時点で `v` の値の inhabited (D16) な leaf であって、
  その leaf が指すオブジェクトが D26 の意味で**計数下**であるもの全体とする。グローバル状態のオブジェクトを
  指す leaf を外すのは、D26 より、そこに D8 の参照が無く、`Retain`/`Release` が `H` を動かさないから
  である。
- `ActRefs^inh_ρ(n)` を、`Inh_ρ(v, π, n)` の各 `λ` を `origin(v, λ).identity()` で名付けて数えた多重集合
  とする。

D15 より `ActRefs(v, π)` は `L(v, π)` の**すべての** leaf を同じ名付けで数えたものなので、
`ActRefs^inh_ρ(n)` は `ActRefs(v, π)` の部分多重集合である。

**`ActRefs^inh_ρ(n)` は名前 (`VarPath`) ごとの多重集合であり、参照の多重集合ではない。** D8 より参照の
多重集合はオブジェクトごとであるのに対し、この数え上げは `VarPath` ごとである。P6 が等式を述べるのは
**各名前をそれが指すオブジェクトへ写した後**である -- 「この数え上げを inhabited (D16) かつ計数下 (D26) の
leaf に制限し、**各名前をそれが指すオブジェクトへ写して**得られる多重集合は、実行時に `Retain(v, π)` が
作る参照の多重集合に等しく、`Release(v, π)` が処分する参照の多重集合にも等しい」。1 つのオブジェクトを
2 つの名前が指す本体は在る (第 7.5.8 節の `C2` の `(o, [])` と `(y, [])`) ので、この写しは省けない。

以下では、名前の多重集合 `M` の各名前 `(u, σ)` を `obj(u, σ)` (D6) へ写して得られる多重集合を `M^obj` と
書く。`ActRefs^inh_ρ(n)` が個数を付ける名前がどれも `ρ` の上のスロットであり、したがって `obj` が定まる
ことは `L14` が与える。P6 より、`n` が `Retain` のとき `(ActRefs^inh_ρ(n))^obj` は `n` が `ρ` で実際に作る
参照の多重集合であり、`n` が `Release` のとき `(ActRefs^inh_ρ(n))^obj` は `n` が `ρ` で実際に処分する参照の
多重集合である。

#### DEF bump の帰属

D8 は「同じオブジェクトへの参照どうしは互いに区別されない」と述べ、「その `Retain` が作った参照」は
オブジェクトごとの個数として読むと定める。したがって P18b の言明の「`p.node` の `Retain` が `ρ` で実際に
作った参照のうち、`ρ` 上でまだ処分されていないもの」は、処分をどの `Retain` の分として数えるかを決めて
初めて定まる。この文書は**走査自身が行う帰属**を取る。すなわち、`un_bump` が `InBracket` で対にした
`Release` の処分だけを、その対の相手の分として引く。

固定した実行路 `ρ` と、`ρ` の上の各節点 `n` について、`n` の訪問の入口状態 `pending(n)`
(`walk_inner(n, P, r)` の `P`) の各要素 `p` に、`VarPath` 上の多重集合 `B_ρ(n, p)` を、`ρ` に沿った次の
規則で定める。`ρ` の最初の節点は `B` の根であり、そこでの `pending` は空なので (`CODE
src/rc_ir/borrow.rs: cancel`)、定めるものが無い。`ρ` の上で `n` の直後にある節点を `n'` と書く。

| `n` の形 | `pending(n')` の各要素の `B_ρ(n', ·)` |
|---|---|
| `Retain(v, π, s, k)` | 押し込まれた新しい要素 `p_new` について `B_ρ(k, p_new) = ActRefs^inh_ρ(n)`。それ以外の要素は `B_ρ(n, ·)` のまま |
| `Release(v, π, s, k)` で `un_bump` が `InBracket` を返す | `un_bump` が選んだ要素 `p_i` について `B_ρ(k, p_i) = B_ρ(n, p_i) - ActRefs^inh_ρ(n)`。残る他の要素は `B_ρ(n, ·)` のまま。`p_i` が取り除かれたときは定めるものが無い |
| `Release(v, π, s, k)` で `un_bump` が `NoBracket` か `OutsideBracket` を返す | 残る各要素は `B_ρ(n, ·)` のまま |
| `Let(x, App/Closure/Llvm, k)`、`Destructure`、`Eval`、`Let(x, Var(y), k)` | 残る各要素は `B_ρ(n, ·)` のまま |
| `Let(x, Match(v, arms), k)` (`n'` は `ρ` が選んだアーム `arm_j` の本体) | 複製された各要素は、`node` が等しい元の要素の `B_ρ(n, ·)` のまま |
| `arm_j` の本体の `Ret` (`n'` はその `Match` の継続 `k_M`) | `merged` の各要素 `p` について、`arm_exits[j]` の中で `node` が `p.node` に等しい要素 `p^{(j)}` の `B_ρ(n, p^{(j)})` |

`consume_objects` が取り除いた要素については定めるものが無い。この表が場合を尽くすことと、最後の 2 行の
`p^{(j)}` が存在することは、`L11` の証明の中で示す。

`B_ρ(n, p)` を、P18b と P18a の言明の `B(p, ρ)` として読む。この帰属は README の D27 と同じものである --
D27 は「`Retain(v, π)` の訪問で `pending` に入るとき … `π` の下の inhabited (D16) かつ計数下 (D26) の各
leaf を `origin` の identity で名付けて数えたもの」「`un_bump` が `InBracket` で `p` を選ぶ `Release` の
訪問で、その `Release` が `ρ` で実際に処分する参照の多重集合を引く」「アームへの複製と `merge` は
`B(p, ρ)` をそのまま運ぶ」「`consume_objects` が `p` を取り除いたときは定めるものが無い」と述べており、
上の表の 6 行はその 4 つを `ρ` の上の節点ごとに並べたものである。

### 7.2 leaf と inhabited の補題

#### L7 (boxed leaf の路は反鎖をなす)

**言明**。A10 を満たす任意の型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元は、
一方が他方の前置になることが無い。

**証明**

<1>1. `boxed_leaf_paths` の内部関数 `go` が `out` に路を push するのは 3 か所であり、そのいずれの直後にも
      `go` は `return` する。すなわち push した路より深い位置へ降りない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  3 か所は `is_closure` の枝 (`path.push(CLOSURE_CAPTURE_IDX)` の後に push して `return`)、`is_box` の枝、
  `is_array` の枝である。

<1>2. `go` が再帰するのは最後の `for (i, fty) in ty.unpunched_field_types(type_env)` の中だけであり、
      その再帰は `path` に添字を 1 つ足してから行われる。この枝では `out` への push は行われない。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>3. `go` の各呼び出しの入口の `path` は、その祖先の呼び出しが足した添字の列である。相異なる 2 つの
      呼び出しの入口の `path` は相異なる。
  BY A10, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types
  A10 が `go` の再帰の停止性を与える -- 「`unpunched_field_types` を繰り返し取って到達する型についても
  同じことが成り立ち、その歩みは有限である」「これが無いと `boxed_leaf_paths` も `rc_units` も停止
  しない」。最初の呼び出しの `path` は空である。`unpunched_field_types` は
  `instance_field_types(...).into_iter().enumerate().filter(...)` を返すので、返る添字は相異なる。<1>2 より、子の呼び出しの入口の `path` は親の入口の `path` に
  添字を 1 つ足したものであり、1 つの親が子に足す添字は `unpunched_field_types` が返す相異なる添字で
  ある。よって呼び出しと入口の `path` は 1 対 1 に対応する。

<1>4. `out` に push される路 `λ` について、`λ` を push した呼び出しの入口の `path` は、`is_box` と
      `is_array` の枝では `λ` そのもの、`is_closure` の枝では `λ` の最後の添字を落としたものである。
  BY <1>1, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>5. QED
  BY <1>1, <1>3, <1>4
  `out` の 2 元 `λ1 ≠ λ2` を取り、`λ1` が `λ2` の真の前置であるとする。<1>4 より `λ2` を push した
  呼び出し `I2` の入口の `path` は `λ2` か `λ2` の最後の添字を落としたものであり、どちらも `λ1` を前置に
  持つ。<1>3 より入口の `path` が `λ1` である呼び出し `I` がただ 1 つ在り、`I2` は `I` 自身か `I` の
  子孫である。一方 <1>4 より `λ1` を push した呼び出しの入口の `path` は `λ1` か `λ1` の最後の添字を
  落としたものである。前者ならその呼び出しは `I` であり、<1>1 より `I` は push の直後に `return` する
  ので子孫を持たず、`I2 = I` となるが `I` が push したのは `λ1` であって `λ2` ではない。後者なら
  その呼び出しは `I` の親であり、<1>1 よりそれも push の直後に `return` するので `I` は存在しない。
  どちらも矛盾である。よって `λ1` は `λ2` の真の前置ではない。`λ1` と `λ2` を入れ替えても同じである。

#### L8 (`origin` の 1 段は inhabited を保つ)

**言明**。`x` を `B` の変数、`λ ∈ boxed_leaf_paths(ty(x))` とする。次の 2 つが成り立つ。

- **(A)** `origin_inner(vars, type_env, x.name, λ)` の本体が `origin(vars, type_env, x'.name, λ')` を
  呼ぶとき、次の 4 つが成り立つ。
  - **(i)** `λ' ∈ boxed_leaf_paths(ty(x'))` である。**`Binding::Join(arm_results)` の腕については、
    どのアームの結果変数 `x' = arm_results[j']` についても成り立つ。**
  - **(ii)** `ρ` の上で `x'` は値を得ている。
  - **(iii)** `ρ` の上で、`λ` が `x` の値の inhabited な leaf であることと、`λ'` が `x'` の値の
    inhabited な leaf であることは同値である。
  - **(iv)** その leaf が inhabited であるとき、`obj(x, λ) = obj(x', λ')` である。

  **(ii)、(iii)、(iv) は、実行路 `ρ` の上で `x` が値を得ているときに主張する。`Binding::Join` の腕に
  ついては、`ρ` が選んだアームの結果変数 `x' = arm_results[j]` についてだけ主張する。(i) と (B) は
  `ρ` を読まない。**
- **(B)** `origin_inner(vars, type_env, x.name, λ)` の本体が `origin` を 1 回も呼ばないとき、その値は
  `Origin::Exactly((x.name, λ))` である。

(iii) の「inhabited」は第 7.1 節の読みのものである。以下の各腕は、leaf の位置に**同じ値**を置くこと
(`Move`、`Join`、`Field`、`Payload` の腕) か、**同じ参照**を置くこと (`Llvm` の単一の `Arg(j, σ)` の腕)
のどちらかを示す。(iii) と (iv) はどちらもそこから出る -- 同じ値の同じ leaf は同じ union のタグを通り
(D16)、同じポインタを持つ (D6 の `obj`)。null についての A5 の例外も同じ理由で保たれる。

**証明** `vars.bindings.get(x.name)` の 7 つの場合で分ける (`CODE src/rc_ir/ownership.rs: Binding`)。

<1>1. CASE `None`、`Some(Binding::Param)`、`Some(Binding::Producer)`。
  <2>1. この 3 つの腕は `here()` すなわち `Origin::Exactly((var.clone(), path.to_vec()))` を返し、
        `origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. QED
    BY <2>1
    呼ぶものが無いので (A) は空虚に成り立ち、`here()` の値が `Origin::Exactly((x.name, λ))` なので
    (B) が成り立つ。

<1>2. CASE `Some(Binding::Move(y))`。この腕は `origin(vars, type_env, &y.name, path)` を呼ぶ。
      すなわち `x' = y`、`λ' = λ` である。
  <2>1. `x` の値は `y` の値である。
    BY D2, D9
    D2 の表より `Let(x, rhs, k)` は `rhs` の値を `x` に束縛し、`RcRhs::Var(y)` の値は `y` の値である。
    D9 の移動の表の第 1 行がこの束縛を移動とする。
  <2>2. `ty(x) = ty(y)` である。
    BY A12
    A12 の第 1 項が move-bind の両辺の型の一致を述べる。
  <2>3. (i) が成り立つ。
    BY <2>2
    `λ ∈ boxed_leaf_paths(ty(x)) = boxed_leaf_paths(ty(y))`。
  <2>4. (ii) が成り立つ。
    BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
    `Binding::Move(y)` は `Let(x, RcRhs::Var(y), k)` からだけ作られる。A11 より `y` の使用はその位置で
    スコープに入っている束縛に解決するので、`x` が値を得た `ρ` の上で `y` は先に値を得ている。
  <2>5. (iii) と (iv) が成り立つ。
    BY <2>1, <2>2, D16, D6
    D16 の inhabited は値とその型だけで決まり、D6 の `obj(x, λ)` はその leaf が指すオブジェクトである。
    <2>1 と <2>2 より `x` と `y` は同じ値と同じ型を持つので、同じ leaf について同じ答えになる。
  <2>6. QED
    BY <2>3, <2>4, <2>5
    この腕は `origin` を呼ぶので (B) は空虚に成り立つ。

<1>3. CASE `Some(Binding::Join(arm_results))`。この腕は各 `arm_result` について
      `origin(vars, type_env, &arm_result.name, path)` を呼ぶ。`ρ` が選んだアームを `arm_j`、
      `x' = arm_results[j]`、`λ' = λ` とする。
  <2>1. `arm_results[j]` は `arms[j].body` の終端の `Ret` が名指す変数である。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var
    `RcRhs::Match(scrut, arms)` の腕が各 `arm` について `returned_var(&arm.body)` を `arm_results` に
    push する。`returned_var` は継続を辿って `RcExpr::Ret(v)` の `v` を返す。
  <2>2. `x` の値は `arm_results[j]` の値である。
    BY D2, D3, <2>1
    D3 より `ρ` はアーム `arm_j` の本体の実行路を辿ってからその `Match` の継続へ進む。D2 の表より
    `Ret(v)` の値は `v` の値であり、`Let(x, Match(v, arms), k)` は `Match` の値を `x` に束縛する。
  <2>3. どの `j'` についても `ty(x) = ty(arm_results[j'])` である。
    BY A12
    A12 の第 2 項が「アームの結果と `Match` の束縛変数の型」の一致を述べる。この項はどのアームに
    ついても言う。
  <2>4. (i) と (ii) が成り立つ。(i) はどのアームの結果変数についても成り立つ。
    BY <2>1, <2>2, <2>3, A11
    (i) は <2>3 から -- `λ' = λ` であり、<2>3 よりどの `j'` についても
    `boxed_leaf_paths(ty(arm_results[j'])) = boxed_leaf_paths(ty(x))` である。(ii) は、`ρ` が `arm_j` の
    本体を通り、その終端の `Ret` が `arm_results[j]` を名指す (<2>1) ので、その変数は `ρ` の上で値を
    得ている。
  <2>5. (iii) と (iv) が成り立つ。
    BY <2>2, <2>3, D16, D6
    <2>2 より `x` と `arm_results[j]` は同じ値を持ち、<2>3 より同じ型を持つ。D16 の inhabited と D6 の
    `obj` はどちらも値とその型だけで決まる。
  <2>6. QED
    BY <2>4, <2>5
    この腕は `origin` を呼ぶので (B) は空虚に成り立つ。

<1>4. CASE `Some(Binding::Llvm(llvm_gen, args, result_ty))` であって
      `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `Some((j, p))` を返す。この腕は
      `origin(vars, type_env, &args[j].name, &p)` を呼ぶ。すなわち `x' = args[j]`、`λ' = p` である。
  <2>1. `decl` は `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` であり、`result_ty` は `ty(x)` で
        ある。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings
    `collect_bindings` は `RcRhs::Llvm(llvm_gen, args)` について `Binding::Llvm(llvm_gen.clone(),
    args.clone(), x.ty.clone())` を作る。
  <2>2. `as_arg_projection(sources)` が `Some((j, p))` を返すのは、`sources` がちょうど 1 元の集合で
        あって、その元が `LeafOrigin::Arg(j, p)` であるときに限る。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection
    `sources.len() != 1` のとき `None`、唯一の元が `Fresh` か `Unknown` のとき `None` を返す。
  <2>3. `p` は `boxed_leaf_paths(ty(args[j]), type_env)` の元であり、`ρ` の上で `λ` が `x` の値の
        inhabited な leaf であることと、`p` が `args[j]` の値の inhabited な leaf であることは同値で
        ある。さらに inhabited であるとき `obj(x, λ) = obj(args[j], p)` である。
    BY A3, D6, <2>1, <2>2
    A3 の表の「単一の `Arg(j, σ)`」の行が、その leaf に置かれるのが「第 `j` オペランドの leaf `σ` と
    同じ参照」であり、「結果のその leaf が inhabited であることと、第 `j` オペランドの leaf `σ` が
    inhabited であることは同値である」と述べる。同じ参照は同じオブジェクトを指すので (D8)、D6 の
    `obj` は一致する。
  <2>4. (ii) が成り立つ。
    BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
    `Binding::Llvm` は `Let(x, RcRhs::Llvm(gen, args), k)` からだけ作られ、A11 より各 `args[j]` は
    その位置でスコープに入っている束縛に解決する。
  <2>5. QED
    BY <2>3, <2>4
    (i) と (iii) と (iv) は <2>3 である。この腕は `origin` を呼ぶので (B) は空虚に成り立つ。

<1>5. CASE `Some(Binding::Llvm(llvm_gen, args, result_ty))` であって
      `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `None` を返す。この腕は
      `origin_from_leaves_under(vars, type_env, &decl, args, path, &here_identity)
      .unwrap_or_else(here)` を返す。ここで `here_identity = (x.name, λ)` である。
  <2>1. `decl` は `boxed_leaf_paths(ty(x), type_env)` の各元を鍵に持つ `LeafMap` を包み、
        `decl.leaf_origins_at(path)` は `Some(sources)` である。`sources` は元数 0 か 1 の集合である。
    <3>1. `decl` の鍵の集合は `boxed_leaf_paths(ty(x), type_env)` である。
      BY A3, CODE src/rc_ir/ownership.rs: collect_bindings,
         CODE src/rc_ir/provenance.rs: Provenance::build_shape,
         CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape
      `collect_bindings` は `Binding::Llvm` の第 3 成分に `x.ty` を置くので、`origin_inner` が
      `result_prov` に渡す `result_ty` は `ty(x)` である。A3 より `result_prov` は結果の leaf ごとに
      `LeafOrigins` を宣言する。`LeafMap::build_shape` は `boxed_leaf_paths(ty, type_env)` の各元を鍵に
      して値を置く。
    <3>2. `leaf_origins_at(path)` は `self.0.get(path)` であり、`path ∈ boxed_leaf_paths(ty(x))` なので
          `Some` を返す。
      BY <3>1, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
         CODE src/rc_ir/leaf_map.rs: LeafMap::get
    <3>3. QED
      BY <3>1, <3>2, A3
      A3 は「その 29 個が leaf に置く集合はすべて要素数 0 か 1 である」と述べ、複数の元を宣言する op が
      このコミットのプログラムに存在しないことを仮定する。
  <2>2. `decl.leaf_origins_under(path)` が渡す集合は、<2>1 の `sources` ただ 1 つである。
    BY <2>1, L7, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
    `leaves_under(path)` は鍵が `path` を前置に持つ元だけを渡す。<2>1 より鍵は
    `boxed_leaf_paths(ty(x))` であり、L7 よりその中で `path` を前置に持つのは `path` 自身だけである。
  <2>3. `origin_from_leaves_under` の中の `operand_units` は空である。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection
    `operand_units` に元が入るのは `LeafOrigin::Arg(j, leaf)` の場合だけである。<2>1 より `sources` は
    元数 0 か 1 であり、元数 1 でその元が `Arg` ならば `as_arg_projection` は `Some` を返すので、この
    場合には来ない。よって `sources` は空集合か、`Fresh` か `Unknown` ただ 1 つである。
  <2>4. この腕は `origin` を呼ばない。
    BY <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    `origin_from_leaves_under` が `origin` を呼ぶのは `operand_units` の各元についてだけである。
  <2>5. この腕の値は `Origin::Exactly((x.name, λ))` である。
    BY <2>3, CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: origin_inner
    <2>3 より `reached` は、`produced_here` が偽なら空、真なら `[Origin::Exactly(here_identity)]` で
    ある。空のとき `let first = reached.first()?;` が `None` を返し、`origin_inner` の
    `unwrap_or_else(here)` が `here()` すなわち `Origin::Exactly((x.name, λ))` を返す。1 元のときは
    `reached.iter().all(|reached_origin| reached_origin == first)` が真なので `Some(first.clone())` を
    返し、その値は `Origin::Exactly(here_identity) = Origin::Exactly((x.name, λ))` である。
  <2>6. QED
    BY <2>4, <2>5
    (A) は空虚に成り立ち、(B) は <2>5 である。

<1>6. CASE `Some(Binding::Field(container, idx))`。
  <2>1. CASE `container.ty.is_box(type_env)` が真。この腕は `here()` すなわち
        `Origin::Exactly((x.name, λ))` を返し、`origin` を呼ばない。よって (A) は空虚に成り立ち、
        (B) が成り立つ。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. CASE `container.ty.is_box(type_env)` が偽。この腕は
        `origin(vars, type_env, &container.name, &container_path)` を呼ぶ。ここで
        `container_path = [idx] ++ path` である。すなわち `x' = container`、`λ' = [idx] ++ λ` である。
    <3>1. `container` の型は unbox の構造体であり、`ty(x)` はその第 `idx` フィールドの型である。
      BY A12
      A12 の第 8 項が「`Destructure` の容器が構造体であること」、第 5 項が「`Destructure` のフィールド
      変数とフィールドの型」の一致を述べる。この場合は `is_box` が偽なので unbox である。
    <3>2. 第 `idx` フィールドは `container.ty` の穴でない。
      BY A12
      A12 が「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が実際に持つ
      (punched でない) ものであること」を述べる。
    <3>2a. `(idx, ty(x))` は `unpunched_field_types(container.ty, type_env)` の元であり、
           `container.ty` は D4 の第 5 規則に来る -- `is_fully_unboxed`、`is_closure`、`is_box`、
           `is_array` がいずれも偽である。
      <4>1. `(idx, ty(x))` は `unpunched_field_types(container.ty, type_env)` の元である。
        BY <3>1, <3>2, CODE src/ast/types.rs: TypeNode::unpunched_field_types
        <3>1 より `ty(x)` は `container.ty` の第 `idx` フィールドの型であり、<3>2 よりそのフィールドは
        穴でない。`unpunched_field_types` は穴でないフィールドを添字とともに返す。
      <4>2. `is_fully_unboxed(ty(x), type_env)` は偽である。
        BY D4
        L8 の仮定より `λ ∈ boxed_leaf_paths(ty(x))` であり、D4 の第 1 規則は `is_fully_unboxed` が
        真の型が leaf を持たないと述べる。
      <4>3. `container.ty` について `is_closure`、`is_funptr`、`is_array` はいずれも偽である。
        BY <3>1, CODE src/ast/types.rs: TypeNode::is_closure,
           CODE src/ast/types.rs: TypeNode::is_funptr, CODE src/ast/types.rs: TypeNode::is_array,
           CODE src/ast/types.rs: TyConVariant, CODE src/fixstd/builtin.rs: bulitin_tycons
        この 3 つの判定は、最上位 tycon がそれぞれ `Std::->`、`Std::FunPtr_*` のいずれか、
        `Std::Array` であることを問う。`bulitin_tycons` はこの 3 種にそれぞれ `TyConVariant::Arrow`、
        `TyConVariant::Primitive`、`TyConVariant::Array` を与える。<3>1 より `container.ty` の最上位
        tycon は構造体として宣言されたもの、すなわち `TyConVariant::Struct` である。1 つの tycon の
        変位は 1 つなので、`container.ty` の最上位 tycon はこの 3 種のどれとも異なる。
      <4>4. QED
        BY <2>2, <4>1, <4>2, <4>3, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
        `is_fully_unboxed` は、`is_box` が真なら偽、`is_closure` が真なら偽、`is_array` が真なら偽、
        `is_funptr` が真なら真を返し、そのいずれでもないとき `unpunched_field_types` の各フィールドの
        型がすべて `is_fully_unboxed` であるかを答える。`is_box` はこの CASE の仮定 (<2>2) より偽、
        残る 3 つは <4>3 より偽なので、判定は最後の行に来る。<4>1 と <4>2 よりその全称は偽なので
        `is_fully_unboxed(container.ty, type_env)` は偽である。
    <3>3. `[idx] ++ λ ∈ boxed_leaf_paths(ty(container))` である。これが (i) である。
      BY D4, <3>2a
      D4 の第 5 規則より、この規則に来る型の leaf は `unpunched_field_types` が返す各フィールド `i` に
      ついて `[i] ++ (そのフィールドの型の leaf)` である。<3>2a より `(idx, ty(x))` はそのフィールドの
      1 つであり、L8 の仮定より `λ ∈ boxed_leaf_paths(ty(x))` である。
    <3>4. `container` の値の第 `idx` フィールドは `x` の値である。
      BY D2, D9
      D2 の表が `Destructure(c, fs, s, k)` を「容器 `c` をフィールドに分解し、各 `(i, x)` の `x` に
      第 `i` フィールドを束縛する」と述べる。D9 の移動の表の第 3 行がこの束縛を移動とする。
    <3>5. (ii) が成り立つ。
      BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
      `Binding::Field(container, idx)` は `Destructure(container, fields, _, k)` からだけ作られ、A11 より
      `container` はその位置でスコープに入っている束縛に解決する。
    <3>6. QED
      BY D16, D6, <3>1, <3>3, <3>4
      D16 より `[idx] ++ λ` が `container` の値の inhabited な leaf であることは、その路が通る unbox
      union の各節で選ぶ変位番号がその節のタグに等しいことである。<3>1 より `container.ty` は構造体なので
      根の節は union ではなく、先頭の添字 `idx` は union の節を通らない。残りの節は `λ` が `x` の値の中で
      通る節そのものである (<3>4)。よって (iii) が成り立つ。<3>4 より `container` の値の `[idx] ++ λ` に
      在るポインタは `x` の値の `λ` に在るポインタと同じものなので、D6 の `obj` も一致し (iv) が
      成り立つ。この腕は `origin` を呼ぶので (B) は空虚に成り立つ。
  <2>3. QED
    BY <2>1, <2>2
    `is_box` の真偽で場合を尽くす。

<1>7. CASE `Some(Binding::Payload(scrut, variant))`。
  <2>1. CASE `variant` が `None` (catch-all)。この腕は `origin(vars, type_env, &scrut.name, path)` を
        呼ぶ。すなわち `x' = scrut`、`λ' = λ` である。
    <3>1. `ty(x) = ty(scrut)` であり、`x` の値は `scrut` の値である。
      BY A12, D9
      A12 の第 4 項が「catch-all アームの payload と scrutinee の型」の一致を述べる。D9 の移動の表の
      第 5 行が「catch-all アームの scrutinee から payload 変数へ」を移動とする。
    <3>2. (ii) が成り立つ。
      BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
      `Binding::Payload(scrut, arm.tag)` は `RcRhs::Match(scrut, arms)` からだけ作られ、A11 より
      `scrut` はその位置でスコープに入っている束縛に解決する。
    <3>3. QED
      BY <3>1, <3>2, D16, D6
      同じ値と同じ型は同じ leaf について同じ答えを与えるので (i) と (iii) と (iv) が成り立つ。この腕は
      `origin` を呼ぶので (B) は空虚に成り立つ。
  <2>2. CASE `variant` が `Some(tag)` であって `scrut.ty.is_box(type_env)` が偽。この腕は
        `origin(vars, type_env, &scrut.name, &scrut_path)` を呼ぶ。ここで
        `scrut_path = [tag] ++ path` である。すなわち `x' = scrut`、`λ' = [tag] ++ λ` である。
    <3>1. `scrut.ty` は unbox union であり、`ty(x)` はその第 `tag` 変位の payload の型である。
      BY A12
      A12 の第 7 項が「`Match` の scrutinee が union であること」、第 3 項が「payload と変位の型」の
      一致を述べる。この場合は `is_box` が偽なので unbox である。
    <3>2. `[tag] ++ λ ∈ boxed_leaf_paths(ty(scrut))` である。これが (i) である。
      BY D4, <3>1
      D4 の第 5 規則が「union のときは各変位の payload へ降りる」と述べる。
    <3>3. `ρ` の上で、`scrut` の値のタグは `tag` である。
      <4>1. 固定した活性化は、この `Binding::Payload(scrut, Some(tag))` を作った `Match` 節点
            `Let(x, Match(scrut, arms), k)` で、`payload` が `x` であるアーム `arm_t` を選んでいる。
            `arm_t.tag` は `Some(tag)` である。
        BY A6, A11, D3, CODE src/rc_ir/ownership.rs: collect_bindings
        `collect_bindings` は `RcRhs::Match(scrut, arms)` の各 `arm` について、`arm.payload` の名前を
        鍵に `Binding::Payload(scrut, arm.tag)` を作る。A6 より名前は束縛を一意に決めるので、
        `x` に値を与える構文はこのアームの payload 束縛だけである。L8 の仮定より `ρ` の上で `x` は
        値を得ているので、D3 より `ρ` はこのアームの本体を通る。
      <4>2. `scrut` の値の実行時のタグを `t_run` と書く。CASE `arms` のあるアームの `tag` が
            `Some(t_run)` である。このとき `tag = t_run` である。
        BY D21, <4>1
        D21 より、そのようなアームが在るとき活性化が選ぶアームは `tag` が `t_run` に等しいアームで
        ある。<4>1 よりそれは `arm_t` なので、`arm_t.tag = Some(tag)` は `Some(t_run)` に等しい。
      <4>3. CASE `arms` のどのアームの `tag` も `Some(t_run)` でない。この場合は起こらない。
        <5>1. `arms` は catch-all アーム (`tag` が `None`) を持つ。
          BY A16
          A16 は「`arms` が catch-all アームを持つか、`s` の値が取りうる実行時のタグがいずれかの
          アームの `tag` である」と述べる。この CASE の仮定より後者ではない。
        <5>2. `arms` の最後のアームが catch-all アームである。
          BY <5>1, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
          `eval_rc_match` は最後のアーム以外の各アームについて
          `arm.tag.expect("a non-final match arm must be a variant arm")` を読むので、catch-all
          アームが最後でない本体はコード生成が停止し、活性化を持たない。
        <5>3. 実行はこの `Match` で最後のアームに入る。
          BY D21, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match
          この CASE の仮定より `t_run` に等しい `tag` を持つアームが無いので、D21 はコード生成の
          振る舞いに従う。`eval_rc_match` は最後のアーム以外の `tag` を case、最後のアームのブロックを
          default とする switch を出すので、どの case にも一致しない `t_run` は default すなわち
          最後のアームへ行く。アームが 1 つのときは無条件にそのアームへ跳ぶので、やはり最後の
          アームである。
        <5>4. QED
          BY <4>1, <5>2, <5>3
          <5>2 より最後のアームの `tag` は `None` であり、`arm_t.tag` は `Some(tag)` なので `arm_t` は
          最後のアームではない。よって <5>3 より実行は `arm_t` に入らず、<4>1 に矛盾する。
      <4>4. QED
        BY <4>2, <4>3
        排中律による。<4>3 の場合は起こらないので、<4>2 の結論が成り立つ。
    <3>4. `x` の値は `scrut` の値の第 `tag` 変位の payload である。
      BY D9, <3>3
      D9 の移動の表の第 4 行が「scrutinee の活性変位の参照が payload 変数へ」を移動とする。<3>3 より
      活性変位は `tag` である。
    <3>5. (ii) が成り立つ。
      BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>6. QED
      BY D16, D6, <3>2, <3>3, <3>4
      D16 より `[tag] ++ λ` が `scrut` の値の inhabited な leaf であることは、その路が通る unbox union の
      各節で選ぶ変位番号がその節のタグに等しいことである。根の節は `scrut` 自身であり、そこで選ぶ変位は
      `tag`、タグも `tag` (<3>3) なので一致する。残りの節は `λ` が `x` の値の中で通る節そのものである
      (<3>4)。よって (iii) が成り立つ。<3>4 より `scrut` の値の `[tag] ++ λ` に在るポインタは `x` の値の
      `λ` に在るポインタと同じものなので、D6 の `obj` も一致し (iv) が成り立つ。この腕は `origin` を
      呼ぶので (B) は空虚に成り立つ。
  <2>3. CASE `variant` が `Some(_)` であって `scrut.ty.is_box(type_env)` が真。この腕は `here()` すなわち
        `Origin::Exactly((x.name, λ))` を返し、`origin` を呼ばない。よって (A) は空虚に成り立ち、
        (B) が成り立つ。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. QED
    BY <2>1, <2>2, <2>3
    `variant` が `None` か `Some` か、`Some` のときは `is_box` の真偽か、で場合を尽くす。

<1>8. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, CODE src/rc_ir/ownership.rs: Binding
  `Binding` は 7 変位を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `origin_inner` の `match` はそれに `None` を加えた腕を持つ。<1>1 が `None`/`Param`/`Producer`、
  <1>2 が `Move`、<1>3 が `Join`、<1>4 と <1>5 が `Llvm`、<1>6 が `Field`、<1>7 が `Payload` を扱う。

#### L8a (`origin` の memo を使わない展開は有限である)

**言明**。`vars` をある本体の `VarTable`、`x` をその本体の変数、`λ` を `FieldPath` とする。次の 2 つが
成り立つ。

- **(i)** `origin(vars, type_env, x.name, λ)` の値は、`vars.origins` の memo が当たるかどうかで
  変わらない。
- **(ii)** その計算を、memo を使わずに `origin_inner` の再帰呼び出しをそのまま展開して得られる木は
  有限である。すなわち、`origin_inner` が `origin` を呼ぶ相手を辿る対の列はどれも有限であり、
  `origin_inner` が `origin` を呼ばない対で終わる。

**(ii) を別に言う必要がある。** `origin` は答えを `vars.origins` に記録して次から返すので、実際の再帰は
memo が当たった位置で止まる。`origin_inner` が呼ぶ相手を辿る議論はその先も辿るので、実際の呼び出しの木の
有限性ではなく、memo を使わない展開の有限性が要る。

**証明**

<1>1. `origin` のどの呼び出しも停止し、したがってその実際の呼び出しの木は有限である。
  BY P2, A11, CODE src/rc_ir/ownership.rs: origin_inner
  P2 は「`origin(x, π)` は、`x` がプログラムの束縛変数であるようなすべての `(x, π)` について、`π` を
  問わず panic せずに答えを返し、停止する」と述べる。停止する呼び出しが直接行う `origin` の呼び出しは
  有限個であり、その入れ子も有限である。A11 (スコープの規律) が P2 の立つ仮定である。`x` が束縛を
  持たない名前であるときは P2 の範囲の外だが、`origin_inner` の `None` の腕が `origin` を呼ばずに
  `here()` を返すので、その木は根 1 つである。

<1>2. `origin` は `vars.origins` に記録された答えが在ればそれを返し、無ければ `origin_inner` を呼んで
      その答えを記録してから返す。記録するのは `origin_inner` が計算した答えそのものであり、記録は
      `origin_inner` から戻った後に行う。
  BY CODE src/rc_ir/ownership.rs: origin

<1>3. (i) が成り立つ。
  BY <1>2
  記録される答えは `origin_inner` が計算したものそのものなので、memo が当たる位置で返る答えは、
  同じ引数について `origin_inner` を呼んで得られる答えと等しい。

<1>4. QED
  BY <1>1, <1>2, <1>3
  memo を使わない展開は、実際の呼び出しの木の memo が当たった各葉を、その答えを記録した呼び出しの
  展開で置き換えて得られる (<1>3 より置き換えても値は変わらない)。<1>2 より記録は `origin_inner` から
  戻った後に行われるので、ある記録の元となった呼び出しの内側で当たる memo は、それより前に記録された
  ものだけである。記録の順序についての帰納法で、各記録の元となった呼び出しの展開が有限であることが
  出る -- 帰納法の仮定が内側の各置き換えの有限性を与え、<1>1 よりその呼び出し自身の木が有限だから
  である。`origin(vars, type_env, x.name, λ)` 自身の展開についても同じ置き換えが当たる。有限の木の
  枝は有限であり、その最後の対では `origin_inner` が `origin` を呼ばない。

#### L9 (`identity` は inhabited を決める)

**言明**。`x` を `B` の変数、`λ ∈ boxed_leaf_paths(ty(x))` とし、実行路 `ρ` の上で `x` が値を得ていると
する。`(u, σ) = origin(x, λ).identity()` と置く。このとき `ρ` の上で `u` は値を得ており、
`σ ∈ boxed_leaf_paths(ty(u))` であり、`λ` が `x` の値の inhabited な leaf であることと `σ` が `u` の値の
inhabited な leaf であることは同値である。

**証明** `origin(x, λ)` の計算の、memo を使わない展開 (L8a (ii)) についての帰納法で示す。L8a (ii) より
その展開は有限なので、この帰納法は整礎である。L8a (i) より、答えは memo が当たるかどうかで変わらない。

<1>1. CASE `origin_inner(vars, type_env, x.name, λ)` が `origin` を 1 回も呼ばない。
  <2>1. `origin(x, λ) = Origin::Exactly((x.name, λ))` である。
    BY L8, CODE src/rc_ir/ownership.rs: origin
    L8 の (B) がこの場合の `origin_inner` の値を与える。
  <2>2. QED
    BY <2>1, D15
    D15 より `Origin::Exactly(p).identity()` は `p` である。よって `(u, σ) = (x.name, λ)` であり、
    3 つの主張はいずれも仮定そのものである。

<1>2. CASE `origin_inner` が `Binding::Move`、`Binding::Llvm` の単一 `Arg` の腕、unbox 容器の
      `Binding::Field`、`Binding::Payload` の catch-all か unbox 変位の腕のいずれかを取る。
  <2>1. これらの腕は `origin(x', λ')` をちょうど 1 回呼び、その値をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner
    `Binding::Move(y)` の腕は `origin(vars, type_env, &y.name, path)`、`Binding::Llvm` の
    `Some((j, p))` の枝は `origin(vars, type_env, &args[j].name, &p)`、unbox 容器の `Binding::Field` の
    枝は `origin(vars, type_env, &container.name, &container_path)`、`Binding::Payload` の `None` の枝は
    `origin(vars, type_env, &scrut.name, path)`、`Some(tag)` かつ unbox の枝は
    `origin(vars, type_env, &scrut.name, &scrut_path)` を、それぞれ腕の値として返す。
  <2>2. `λ' ∈ boxed_leaf_paths(ty(x'))` であり、`ρ` の上で `x'` は値を得ており、`λ` が `x` の値の
        inhabited な leaf であることと `λ'` が `x'` の値の inhabited な leaf であることは同値である。
    BY L8
    L8 の (A) の (i)(ii)(iii) である。
  <2>3. QED
    BY <2>1, <2>2, 帰納法の仮定
    <2>1 より `origin(x, λ).identity() = origin(x', λ').identity() = (u, σ)` である。<2>2 で帰納法の
    仮定が `(x', λ')` に適用でき、`u` が `ρ` の上で値を得ていること、`σ ∈ boxed_leaf_paths(ty(u))` で
    あること、`λ'` が inhabited であることと `σ` が inhabited であることが同値であることが出る。
    <2>2 の同値と繋いで言明が成り立つ。

<1>3. CASE `origin_inner` が `Binding::Join(arm_results)` の腕を取り、
      `Origin::of_candidates(candidates, &(x.name, λ))` が `Origin::Exactly` を返す。
  <2>1. このとき `candidates` は 1 元集合であり、返り値はその唯一の元 `c` についての
        `Origin::Exactly(c)` である。よって `(u, σ) = c` である。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: origin_inner, D15
    `of_candidates` は `candidates.len()` が 1 のとき `Origin::Exactly(その元)` を返す。
  <2>2. `ρ` が選んだアームの結果変数を `x' = arm_results[j]` とすると、`origin(x', λ).acted_on()` の元は
        すべて `c` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, <2>1
    この腕は各 `arm_result` について `origin(vars, type_env, &arm_result.name, path).acted_on()` の
    各元を `candidates` に入れる。<2>1 より `candidates = {c}` なので、どの `arm_result` の `acted_on`
    の元も `c` である。
  <2>3. `origin(x', λ).identity() = c` である。
    BY <2>2, D15
    D15 より `acted_on()` は `identity()` を先頭に持つ列であり、空でない。
  <2>4. `λ ∈ boxed_leaf_paths(ty(x'))` であり、`ρ` の上で `x'` は値を得ており、`λ` が `x` の値の
        inhabited な leaf であることと `λ` が `x'` の値の inhabited な leaf であることは同値である。
    BY L8
    L8 の (A) の `Binding::Join` の腕についての主張は `ρ` が選んだアームの結果変数についてのもので
    ある。
  <2>5. QED
    BY <2>1, <2>3, <2>4, 帰納法の仮定
    <2>4 で帰納法の仮定が `(x', λ)` に適用でき、その `identity` は <2>3 より `c = (u, σ)` である。

<1>4. CASE `origin_inner` が `Binding::Join(arm_results)` の腕を取り、`of_candidates` が
      `Origin::Join` を返す。
  <2>1. このとき返り値は `Origin::Join { identity: (x.name, λ), candidates }` であり、
        `(u, σ) = (x.name, λ)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Origin::of_candidates, D15
    この腕は `Origin::of_candidates(candidates, &(var.clone(), path.to_vec()))` を返し、
    `of_candidates` は元数が 1 でないとき `Origin::Join { identity, candidates }` を返す。D15 より
    `Origin::Join` の `identity()` はその `identity` フィールドである。
  <2>2. QED
    BY <2>1
    `(u, σ) = (x.name, λ)` なので、3 つの主張はいずれも仮定そのものである。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: Binding, CODE src/rc_ir/ownership.rs: Origin
  `origin_inner` の `match` の腕は `None`/`Param`/`Producer`、`Move`、`Join`、`Llvm`、`Field`、
  `Payload` である。`origin` を呼ばない腕 -- `None`/`Param`/`Producer`、`Llvm` の
  `origin_from_leaves_under` の枝、boxed 容器の `Field` の枝、boxed scrutinee の `Payload` の枝 -- は
  <1>1 が扱う。`origin` を 1 回呼んでその値を返す腕は <1>2 が扱う。`Binding::Join` の腕は
  `of_candidates` の返り値が `Origin` の 2 変位のどちらかなので <1>3 と <1>4 で尽きる。

#### L10 (名前の活性)

**言明**。実行路 `ρ` を固定する。`(x, λ)` と `(w, μ)` を、`λ ∈ boxed_leaf_paths(ty(x))`、
`μ ∈ boxed_leaf_paths(ty(w))` であって `ρ` の上で `x` と `w` がともに値を得ている 2 つの対とし、
`origin(x, λ).identity() = origin(w, μ).identity()` とする。このとき、`λ` が `x` の値の inhabited な
leaf であることと、`μ` が `w` の値の inhabited な leaf であることは同値である。

**証明**

<1>1. `(u, σ) = origin(x, λ).identity() = origin(w, μ).identity()` と置く。`λ` が `x` の値の inhabited な
      leaf であることと `σ` が `u` の値の inhabited な leaf であることは同値である。
  BY L9

<1>2. `μ` が `w` の値の inhabited な leaf であることと `σ` が `u` の値の inhabited な leaf であることは
      同値である。
  BY L9

<1>3. QED
  BY <1>1, <1>2
  同値の推移律による。

#### DEF 名前の活性

実行路 `ρ` を固定する。名前 `o ∈ VarPath` が `ρ` で**活性**であるとは、
`origin(x, λ).identity() = o` を満たす対 `(x, λ)` で、`λ ∈ boxed_leaf_paths(ty(x))` であり `ρ` の上で
`x` が値を得ているものが在って、`λ` が `x` の値の inhabited な leaf であり、かつ `obj(x, λ)` が D26 の
意味で計数下であることをいう。活性な名前 `o` が指すオブジェクトを `obj_ρ(o)` と書く。

**この述語は対の選び方によらず、`ρ` の上の時点にもよらない。** 対の選び方によらないのは、inhabited に
ついては L10 が、`obj(x, λ)` については P5 (a) が与えるからである (identity が等しい 2 つの leaf の
スロットは同じオブジェクトを指す)。時点によらないのは、2 つの条件がどちらも時点によらないからである。
inhabited が時点によらないのは、値が束縛の後に変わらないことによる -- D2 より本体は木であり、D3 の
実行路は根から各節点を高々 1 度通るので、1 つの束縛は 1 つの実行路の上で高々 1 回実行される。計数下が
時点によらないのは D26 の最後の段落による。

値は束縛の後に変わらない -- D2 より本体は木であり、D3 の実行路は根から各節点を高々 1 度通るので、
1 つの束縛は 1 つの実行路の上で高々 1 回実行される。よって活性かどうかは `ρ` の上の時点によらない。

#### L10a (静的な数え上げと実行時の作用が活性な名前で一致する)

**言明**。実行路 `ρ` と、`ρ` の上の節点 `n` を固定する。`n` の式が `RcExpr::Retain(v, π, s, k)` または
`RcExpr::Release(v, π, s, k)` であるとき、各名前 `o` について次が成り立つ。

- `o` が `ρ` で活性ならば `ActRefs(v, π)[o] = ActRefs^inh_ρ(n)[o]` である。
- `o` が `ρ` で活性でないならば `ActRefs^inh_ρ(n)[o] = 0` である。

**証明**

<1>1. `ActRefs(v, π)` は `L(v, π)` の各 leaf `λ` を `origin(v, λ).identity()` で名付けて数えた多重集合で
      あり、`ActRefs^inh_ρ(n)` は `Inh_ρ(v, π, n)` の各 leaf を同じ名付けで数えた多重集合である。
      `Inh_ρ(v, π, n) ⊆ L(v, π)` である。
  BY D15, DEF 実行時の作用, 第 1 節の記法
  D15 より `acted_references(v, π)` は `π` の下のすべての boxed leaf の `origin(v, ・).identity()` を
  数えた多重集合であり、第 1 節の記法より `π` の下の boxed leaf の全体は `L(v, π)` である。

<1>2. `L(v, π)` の各元 `λ` は `boxed_leaf_paths(ty(v), type_env)` の元であり、`ρ` の上で `v` は値を
      得ている。
  BY 第 1 節の記法, D3, A11
  第 1 節の記法より `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の部分集合である。`n` は `ρ` の上の
  節点であり、`v` を名指す。A11 より `v` はその位置でスコープに入っている束縛に解決するので、`ρ` の上で
  `v` は先に値を得ている。

<1>3. CASE `o` が `ρ` で活性である。
  <2>1. `L(v, π)` の元 `λ` で `origin(v, λ).identity() = o` となるものは、すべて `v` の値の inhabited な
        leaf である。
    BY DEF 名前の活性, L10, <1>2
    `o` が活性なので、`origin(x, λ0).identity() = o` を満たす対 `(x, λ0)` で `λ0` が inhabited なものが
    在る。<1>2 より `(v, λ)` も L10 の仮定を満たす対なので、L10 より `λ` は inhabited である。
  <2>1a. <2>1 の各 `λ` について `obj(v, λ)` は計数下である。
    BY DEF 名前の活性, P5, <2>1
    `o` が活性なので、`origin(x, λ0).identity() = o` を満たす対 `(x, λ0)` で `obj(x, λ0)` が計数下である
    ものが在る。<2>1 より `(v, λ)` のスロットは在り、P5 (a) より identity が等しい 2 つの leaf のスロットは
    同じオブジェクトを指すので `obj(v, λ) = obj(x, λ0)` である。
  <2>2. QED
    BY <2>1, <2>1a, <1>1, DEF 実行時の作用
    `Inh_ρ(v, π, n)` は `L(v, π)` の元のうち inhabited かつ計数下のものの全体なので (DEF 実行時の作用)、
    <2>1 と <2>1a より、`o` で名付けられる `L(v, π)` の元はすべて `Inh_ρ(v, π, n)` に入る。よって
    2 つの数え上げの `o` の個数は等しい。

<1>4. CASE `o` が `ρ` で活性でない。
  <2>1. `Inh_ρ(v, π, n)` の元 `λ` で `origin(v, λ).identity() = o` となるものは無い。
    BY DEF 名前の活性, DEF 実行時の作用, <1>2
    そのような `λ` が在れば、DEF 実行時の作用より `λ` は inhabited で `obj(v, λ)` は計数下なので、
    `(v, λ)` は DEF 名前の活性の 3 つの条件をすべて満たす対であり、`o` は活性である。
  <2>2. QED
    BY <2>1, <1>1
    `ActRefs^inh_ρ(n)[o]` は `Inh_ρ(v, π, n)` の元で `o` と名付けられるものの個数である。

<1>5. QED
  BY <1>3, <1>4
  排中律による。

### 7.3 `outstanding` と `B` の関係

#### L11 (活性な名前では一致し、非活性な名前では `B` は空)

**言明**。実行路 `ρ` と、`ρ` の上の節点 `n` を固定する。`pending(n)` の各要素 `p` と各名前 `o`
について、次が成り立つ。

- **(i)** `o` が `ρ` で活性ならば `p.outstanding[o] = B_ρ(n, p)[o]` である。
- **(ii)** `o` が `ρ` で活性でないならば `B_ρ(n, p)[o] = 0` である。

また、DEF bump の帰属の表は `ρ` の上の場合を尽くし、その最後の 2 行の `p^{(j)}` は存在する。

**証明** `ρ` の上の節点の並びについての帰納法で示す。D3 より実行路は有限の列である。

<1>1. 基底。`ρ` の最初の節点は `B` の根であり、`pending` は `PendingRetains::default()` すなわち空で
      ある。
  BY CODE src/rc_ir/borrow.rs: cancel
  `cancel_body` は `analysis.walk(body, PendingRetains::default(), true)` を呼ぶ。要素が無いので (i) と
  (ii) は空虚に成り立つ。

<1>1a. `consume_objects(pending, objects)` は、`objects` のいずれかについて `outstanding.names(object)`
       が真である要素を取り除いてその `node` を `self.needed_retains` に入れ、残る要素の `node`・
       `outstanding`・並びを変えない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, 外部の結果 `Vec::retain`
  本体は `pending.retain(|retain| { if objects.iter().any(|object| retain.outstanding.names(object))
  { self.needed_retains.insert(retain.node); return false; } true })` である。

<1>2. 帰納段。`ρ` の上の節点 `n` について (i) と (ii) が成り立つとし、`ρ` の上の `n` の直後の節点 `n'` に
      ついて示す。`n` の式で場合を分ける。

  <2>1. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。`n'` は `k` である。
    <3>1. `pending(k)` は `pending(n)` の末尾に `PendingRetain { node: node_id(n), outstanding:
          ActRefs(v, path) }` を足したものである。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
         CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references, L2, 第 1 節の記法
      この腕は `pending.push(PendingRetain { node: retain, outstanding })` を行い、`outstanding` は
      `self.acted_references(v, path)` である。`PendingRetain` はこの 2 つのフィールドだけを持つ
      (`CODE src/rc_ir/borrow.rs: PendingRetain`)。L2 (i) よりこの腕は `walk(k, pending, ·)` を 1 回呼ぶ。
    <3>2. 元からあった要素については (i) と (ii) が成り立つ。
      BY 帰納法の仮定, DEF bump の帰属, <3>1
      表の第 1 行より、これらの要素の `B_ρ` は変わらない。<3>1 より `outstanding` も変わらない。
    <3>3. 新しい要素 `p_new` について、`p_new.outstanding = ActRefs(v, path)` であり、
          `B_ρ(k, p_new) = ActRefs^inh_ρ(n)` である。
      BY <3>1, DEF bump の帰属
    <3>4. QED
      BY <3>2, <3>3, L10a
      L10a より、活性な `o` について `ActRefs(v, path)[o] = ActRefs^inh_ρ(n)[o]` なので `p_new` に
      ついて (i) が成り立ち、活性でない `o` について `ActRefs^inh_ρ(n)[o] = 0` なので (ii) が成り立つ。
      <3>2 が他の要素を扱う。

  <2>2. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。`n'` は `k` である。
    <3>1. `n` の訪問は、`consume_objects` を 1 回か 2 回、`un_bump` を 1 回行い、その後
          `walk(k, pending, ·)` を呼ぶ。
      BY L5
    <3>2. CASE `un_bump` が `NoBracket` か `OutsideBracket` を返す。
      <4>1. `un_bump` は `pending` を変えない。
        BY P17
        P17 は、`NoBracket` と `OutsideBracket` のどちらの場合も `pending` は変わらないと述べる。
      <4>2. `pending(k)` の各要素は `pending(n)` の要素であり、`node` と `outstanding` は変わらない。
        BY <3>1, <4>1, <1>1a
      <4>3. QED
        BY <4>2, 帰納法の仮定, DEF bump の帰属
        表の第 3 行よりこれらの要素の `B_ρ` は変わらない。
    <3>3. CASE `un_bump` が `InBracket` を返す。
      <4>1. `un_bump` が選んだ要素 `p_i` の `outstanding` は `p_i.outstanding - ActRefs(v, path)` に
            なり、それが空ならば `p_i` は取り除かれる。他の要素の `node` と `outstanding` は変わらない。
            この場合 `consume_objects` は `others` について 1 回だけ呼ばれる。
        BY P17, L5
        P17 は `InBracket(t)` の場合の作用を述べる。L5 は、`OutsideBracket` のときにだけ 2 回目の
        `consume_objects` が呼ばれることを述べる。
      <4>2. `p_i.outstanding` は `ActRefs(v, path)` を `covers` する。すなわち各 `o` について
            `p_i.outstanding[o] ≥ ActRefs(v, path)[o]` である。
        BY P17, D15
        P17 は `covers` するときにだけ `InBracket` を返すと述べる。D15 より `covers(R)` は各オブジェクトに
        ついて自分の個数が `R` 以上かを答える。
      <4>3. 活性な `o` について、`p_i.outstanding[o] - ActRefs(v, path)[o] = B_ρ(n, p_i)[o] -
            ActRefs^inh_ρ(n)[o]` であり、この値は 0 以上である。
        BY 帰納法の仮定, <4>2, L10a
        帰納法の仮定 (i) より `p_i.outstanding[o] = B_ρ(n, p_i)[o]`、L10a より引く量が等しい。
        <4>2 より差は 0 以上である。
      <4>4. 活性でない `o` について、`B_ρ(n, p_i)[o] - ActRefs^inh_ρ(n)[o] = 0` である。
        BY 帰納法の仮定, L10a
        帰納法の仮定 (ii) より `B_ρ(n, p_i)[o] = 0`、L10a より `ActRefs^inh_ρ(n)[o] = 0`。
      <4>5. QED
        BY <4>1, <4>3, <4>4, 帰納法の仮定, DEF bump の帰属, <1>1a
        表の第 2 行より `B_ρ(k, p_i) = B_ρ(n, p_i) - ActRefs^inh_ρ(n)` であり、<4>3 と <4>4 が
        `p_i` について (i) と (ii) を与える。他の要素は <4>1 と <1>1a より `outstanding` が変わらず、
        表より `B_ρ` も変わらないので帰納法の仮定がそのまま使える。
    <3>4. QED
      BY <3>2, <3>3, P17
      P17 は `un_bump` の返り値が `NoBracket`、`OutsideBracket`、`InBracket` のいずれかであると述べる。

  <2>3. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match` でない、または
        `RcExpr::Destructure(container, fields, _state, k)`、または `RcExpr::Eval(_, k)` である。
        `n'` は `k` である。
    <3>1. これらの腕が `pending` に行うのは `consume_objects` の呼び出しだけである。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
         CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, L4
      `RcExpr::Let(x, rhs, k)` の腕は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び、`consume_rhs`
      は `rhs_consumes` が集めた各元について `self.consume` を呼び、L4 より `consume` は
      `consume_objects` を 1 回呼ぶ。`RcExpr::Destructure` の腕は `self.consume` を 0 回以上呼ぶ。
      `RcExpr::Eval(_, k)` の腕は `pending` に触れない。
    <3>2. `pending(k)` の各要素は `pending(n)` の要素であり、`node` と `outstanding` は変わらない。
      BY <3>1, <1>1a
    <3>3. QED
      BY <3>2, 帰納法の仮定, DEF bump の帰属
      表の第 4 行よりこれらの要素の `B_ρ` は変わらない。

  <2>4. CASE `n` の式が `RcExpr::Let(x, RcRhs::Match(v, arms), k)` である。D3 より `n'` は `ρ` が選んだ
        アーム `arm_j` の本体である。
    <3>1. `pending(arm_j.body)` は `pending(n).clone()` であり、要素の `node`・`outstanding`・並びは
          `pending(n)` と等しい。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
         `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, CODE src/rc_ir/borrow.rs: PendingRetain, L2
      この腕は各 `arm` について `self.walk(&arm.body, pending.clone(), false)` を呼ぶ。
      `PendingRetain` は `Clone` を derive し、`Vec::clone` は要素をその並びのまま複製する。
    <3>2. QED
      BY <3>1, 帰納法の仮定, DEF bump の帰属
      表の第 5 行より、複製された要素の `B_ρ` は元の要素のものである。

  <2>5. CASE `n` の式が `RcExpr::Ret(_)` であり、`n` は `B` の終端の `Ret` ではない。
    <3>1. `n` はある `Match` 節点 `M` のあるアーム `arm_j` の本体の実行路を終える `Ret` であり、
          `ρ` の上の `n` の直後の節点 `n'` は `M` の継続 `k_M` である。
      BY D3
      D3 の第 2 の規則が `Let(x, Match(v, arms), k)` について「アームを 1 つ選び、そのアーム本体の
      実行路を辿り、その後 `k` へ進む」と述べ、続く段落が「アーム本体の `Ret` はそのアーム本体の
      実行路を終えるだけであり、関数本体の実行路は続く」と述べる。仮定より `n` は関数本体の実行路の
      最後の節点ではないので、`n` はアーム本体の実行路を終える `Ret` である。
    <3>2. `M` の訪問は、各アームについて `walk(&arm.body, pending.clone(), false)` を呼んでその値を
          `arm_exits` に集め、`merged = self.merge(&pending, &arm_exits)` を作り、
          `walk(k_M, merged, ·)` を呼ぶ。すなわち `pending(k_M) = merged` である。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
         `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, L2
    <3>3. `arm_exits[j] = pending(n)` である。
      BY L2, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, <3>1, <3>2
      `RcExpr::Ret(_)` の腕は `pending` を変えずに `pending` を返す。L2 (i) より、`arm_j.body` の訪問
      から `n` の訪問までは継続の辺だけを辿る `walk` の呼び出しの入れ子であり、L2 (i) が挙げる腕は
      いずれも継続の訪問の返り値をそのまま自分の返り値とする。
    <3>4. `merged` の各要素 `p` について、`p.node` は `pending(M)` のある要素の `node` であり、
          `p.outstanding` は各 `arm_exits[j']` の中の `node` が `p.node` に等しい要素の `outstanding`
          と等しい。とくに `arm_exits[j]` にそのような要素 `p^{(j)}` が在る。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, P18
      `merge` の返り値は `pending_in.iter().filter_map(|retain| uniform.get(&retain.node).map(
      |outstanding| PendingRetain { node: retain.node, outstanding: outstanding.clone() }))` であり、
      `uniform` に `retain` が入るのは `is_uniform` すなわち
      `entered_with.contains(&retain) && arm_states.iter().all(|other| other.get(&retain) ==
      Some(&outstanding))` が真のときだけで、入る値は `outstanding.clone()` である。P18 も、残る
      `Retain` が「すべてのアームの出口に同じ `outstanding` で現れる」ものだけであると述べる。
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4, 帰納法の仮定, DEF bump の帰属
      表の第 6 行より `B_ρ(k_M, p) = B_ρ(n, p^{(j)})` である。<3>3 と <3>4 より `p^{(j)}` は
      `pending(n)` の要素であり、その `outstanding` は `p.outstanding` に等しい。帰納法の仮定が
      `(n, p^{(j)})` について (i) と (ii) を与えるので、`(k_M, p)` についても成り立つ。

  <2>6. CASE `n` の式が `RcExpr::Ret(_)` であり、`n` は `B` の終端の `Ret` である。
    BY D3
    D3 より終端の `Ret` は実行路の最後の節点なので、`ρ` の上に `n'` は無い。示すものが無い。

  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
    `RcExpr` は 6 種を持ち、`Let` を右辺が `Match` かどうかで 2 つに分け、`Ret` を `B` の終端かどうかで
    2 つに分けた 8 つの場合を <2>1 から <2>6 が尽くす。DEF bump の帰属の表の 6 行はこの 8 つの場合に
    対応する (第 4 行が 3 種をまとめ、第 6 行がアーム本体の `Ret` を扱い、終端の `Ret` には行が要らない)。

<1>3. QED
  BY <1>1, <1>1a, <1>2


### 7.4 P18b の証明

**P18b**。走査中の各位置と、その位置に至る各実行路 `ρ` について、`pending` の各要素 `p` を考える。
`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上でまだ処分されていないものを、**それを
作った leaf の `origin` の identity で名付けた**多重集合を `B(p, ρ)` とする (D27)。このとき
`p.outstanding` は `B(p, ρ)` を `covers` する。とくに `p.outstanding` が空ならば `B(p, ρ)` も空である。

**証明** 走査中の位置を `B` の節点 `n` の訪問の入口とし、`B(p, ρ)` を DEF bump の帰属の `B_ρ(n, p)` と
読む。**この読みでは `B(p, ρ)` は名前 (`VarPath`) ごとの多重集合である。** D27 が `B(p, ρ)` を「`π` の下の
inhabited (D16) かつ計数下 (D26) の各 leaf を `origin` の identity で名付けて数えたもの」と定めており、
`p.outstanding` (`References`) も `covers` も `VarPath` を鍵とするので (D15)、`covers` の両辺が同じ水準に
あるのはこの読みである。示すのはこの水準の言明であり、`<1>1` がそれと「実際に作った参照」との関係を
述べる。

<1>1. `B_ρ(n, p)` を各名前が指すオブジェクトへ写した多重集合 `(B_ρ(n, p))^obj` は、`p.node` の `Retain` が
      `ρ` で実際に作った参照の多重集合から、`ρ` の上でその `Retain` より後にあって `un_bump` がその要素と
      対にした `Release` が実際に処分した参照の多重集合を引いたものである。
  BY DEF bump の帰属, DEF 実行時の作用, P6, D26, A8, L11
  L11 より DEF bump の帰属の表は `ρ` の上の場合を尽くすので、`pending(n)` の各要素 `p` の `B_ρ(n, p)` は
  表の第 1 行が置いた初期値から表を辿って得られる。第 1 行が `ActRefs^inh_ρ` を初期値に置き、第 2 行が
  それを引く唯一の行であり、第 3 行から第 6 行は値を引き継ぐだけである。第 2 行が引くのは、`un_bump` が
  `InBracket` でその要素と対にした `Release` の分だけである。名前ごとの足し引きは写した後でも足し引きなので、
  `(・)^obj` は表の各行と可換である。

  `(ActRefs^inh_ρ(n))^obj` が実際に作る / 処分する参照の多重集合であることは、P6 と D26 の両方が要る。
  P6 は「この数え上げを inhabited (D16) かつ計数下 (D26) の leaf に制限し、**各名前をそれが指す
  オブジェクトへ写して**得られる多重集合は、実行時に `Retain(v, π)` が作る参照の多重集合に等しく、
  `Release(v, π)` が処分する参照の多重集合にも等しい」と述べる。`ActRefs^inh_ρ(n)` は DEF 実行時の作用より
  その制限を掛けた数え上げそのものであり、`(・)^obj` が P6 の言う写しである。グローバル状態のオブジェクトを
  指す leaf を落としてよいのは、それが D8 の参照を持たず (D26)、`Retain`/`Release` が `H` を変えない (A8)
  からである。

<1>2. 各名前 `o` について、`o` が `ρ` で活性ならば `p.outstanding[o] = B_ρ(n, p)[o]` であり、
      活性でなければ `B_ρ(n, p)[o] = 0` である。
  BY L11

<1>3. 各名前 `o` について `p.outstanding[o] ≥ B_ρ(n, p)[o]` である。
  BY <1>2
  活性なら等しく、活性でなければ右辺が 0 で左辺は 0 以上である。

<1>4. `p.outstanding` は `B_ρ(n, p)` を `covers` する。
  BY <1>3, D15, CODE src/rc_ir/ownership.rs: References::covers
  D15 より `covers(R)` は各オブジェクトについて自分の個数が `R` 以上かを答える。`covers` の本体は
  `other.0.iter().all(|(object, count)| self.0.get(object).is_some_and(|held_count| held_count >=
  count))` であり、`References` の `Map` に鍵が無いことは個数 0 と同じ意味なので、これは <1>3 の
  不等式である。

<1>5. `p.outstanding` が空ならば `B_ρ(n, p)` も空である。
  BY <1>2
  `p.outstanding` が空ならば、活性な `o` について `B_ρ(n, p)[o] = p.outstanding[o] = 0` であり、活性で
  ない `o` についても `B_ρ(n, p)[o] = 0` である。

<1>6. QED
  BY <1>1, <1>4, <1>5, DEF bump の帰属
  `<1>4` が `covers` を、`<1>5` が「空ならば空」を、どちらも名前ごとの多重集合 `B_ρ(n, p)` について与える。
  これが D27 の帰属で読んだ P18b である。`<1>1` は、その `B_ρ(n, p)` をオブジェクトへ写したものが P18b の
  文面「`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上でまだ処分されていないもの」に一致する
  ことを述べる。

**この証明の要**。`outstanding` から引かれるのは静的な数え上げ `ActRefs(v, path)` であり、`B_ρ` から
引かれるのは実行時の `ActRefs^inh_ρ`、すなわち inhabited な leaf の分だけである。左辺と右辺で引かれる量が
違うので `covers` が保たれるとは限らないが、L10a が、活性な `o` についてはこの 2 つが
**一致する**ことを与える。それを与えるのが L9 -- `origin` の `identity` が等しい leaf は同時に inhabited
であるか同時にそうでないか -- である。unbox union の `Retain` の `ActRefs` は変位すべての leaf を数えるが、
数えられた名前のうち活性でないものについては `B_ρ` の側が常に 0 なので、そこで `outstanding` がいくら
減っても不等式は破れない。

### 7.5 P18a

**結論を先に書く。**

- P18a は A1・A2・D12 だけからは**出ない**。反例が 2 つある。7.5.7 の `C1` (`Join` が 1 つのオブジェクトに
  2 つ目の名前を与える形) と 7.5.8 の `C2` (呼び出しに渡した参照が返り値として別の名前で戻る形) である。
  どちらも D12 を満たし、どちらでも `cancel` が対を消して解放後の読みを作る。
- 足りない前提は README の **A19** である。7.5.3 がそれを引く。示す形は**オブジェクトごとの形** --
  計数下オブジェクト `O` について `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` として `H(O) ≥ n(O) + 1` --
  であり、README の P18a の言明そのものである。名前ごとの弱い形を経由しない (7.5.1 と 7.5.4 の <1>3)。
- 7.5.4 が引くのは A19 の (i)、(ii-a)、(ii-b) と P14a である。(ii-b) は条件付きの形
  「`bumps ≥ 1` である時点では `held ≥ 1 + bumps`」であり、7.5.4 の `<1>3` と `<1>6` はそれを
  `bumps ≥ 1` の類に直接当てる。他の類に要る無条件の `held ≥ bumps` は、7.5.3 の `L14b` が (ii-a) の
  非負性と (ii-b) から 1 段で出す。借用終端の類で `bumps` が 0 である場合を埋めるのが P14a である。
- **`+1` は 1 回だけ立つ。** A19 (i) の右辺は、各類の `d(C)` の総和と、借用終端の類が在るときの `+1` で
  ある。`bumps` が正である類 `C0` が借用終端でなければ (ii-b) の `+1` が、借用終端であれば (i) の
  角括弧の `+1` が立つ。7.5.4 の `<1>6` と `<1>7` がその 2 つの場合である。
- `A19` を果たす者は `insert_rc` と `borrow_ify` である。**この文書が示すのは `borrow_ify` の側だけで
  ある。** 7.5.5 の `L16` がそれを示す -- `rewrite_rc` が借用版で落とす `Retain` は、その leaf を消費する
  構文が残っている限り落ちない (P8 と P7a)。落ちる場合には `call_rc` が消費の直前に同じ `Retain` を
  置く。`insert_rc` の側は `p60-insert-rc.md` が部分的に示しており、7.5.6 が残る義務を指す。

#### 7.5.1 示す不変条件

**DEF `N`**。`ρ` の上の節点 `n` と計数下 (D26) のオブジェクト `O` について、
`N_ρ(n, O) = Σ_{p ∈ pending(n)} Σ_{o} B_ρ(n, p)[o]` と定める。内側の和は、`ρ` で活性 (DEF 名前の活性) で
あって `obj_ρ(o) = O` である名前 `o` を渡る。L11 (ii) より活性でない名前の `B_ρ` は 0 なので、この制限は
和を変えない。

**INV(n)**。`ρ` の上の節点 `n` の訪問の入口において、計数下の各オブジェクト `O` について、
`N_ρ(n, O) ≥ 1` ならば `H(O) ≥ N_ρ(n, O) + 1` である。ここで `H(O)` は `n` の時点の参照カウント (D7)
であり、走査における `n` の訪問の入口と実行時のその時点とを同じものとして扱うことは、第 7.5.3 節の
`DEF 節点の時点` が定める。

**INV は P18a そのものである。** P18a の `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` は、走査中の位置を
節点の訪問の入口に取り、`B(p, ρ)` を D27 の帰属 (第 7.1 節の DEF bump の帰属) で読むと `N_ρ(n, O)` で
ある。内側の和を活性な名前に制限してよいのは、L11 (ii) より活性でない名前の `B_ρ` が 0 だからであり、
`obj_ρ(o)` が定まるのは DEF 名前の活性による。よって 7.5.4 が INV を示せば P18a が出る。

**オブジェクトごとに和を取るところは落とせない。** 名前ごとの `H(o) ≥ n(o) + 1` を足し合わせても
`H(O) ≥ Σ_o n(o) + 1` は出ない -- `+1` が名前の個数だけ立つからである。7.5.4 の `<1>3` は余りを
別名類ごとに 1 つ数え、`<1>6` と `<1>7` が `bumps` の正である類 1 つについて `+1` を立てるので、
オブジェクトごとの形をそのまま与える。

#### 7.5.2 別名の歩みと別名類

##### DEF ρ-歩みと ρ-終端

`ρ` の上で値を得ている変数 `x` と `λ ∈ boxed_leaf_paths(ty(x))` をとる。L8 (A) が挙げる対 `(x', λ')` --
`origin_inner(vars, type_env, x.name, λ)` が `origin` を呼ぶ相手、`Binding::Join` の腕については `ρ` が
選んだアームの結果変数のもの -- を `(x, λ)` の **ρ-歩み**と呼ぶ。`origin_inner` が `origin` を呼ばない
とき、`(x, λ)` を **ρ-終端**と呼ぶ。

##### L12 (ρ-歩みは終端で終わる)

**言明**。`ρ` の上で値を得ている `x` と `λ ∈ boxed_leaf_paths(ty(x))` について、次の 2 つが成り立つ。

- **(i)** `(x, λ)` から ρ-歩みを辿る列は有限で、ρ-終端で終わる。その終端を `T_ρ(x, λ)` と書く。列の各対は
  L8 (A) の (i)-(iv) を満たす。
- **(ii)** 列の各対の変数は、1 つ前の対の変数が `ρ` の上で値を得る段より前に値を得ている。とくに
  `T_ρ(x, λ)` の変数は、`x` が値を得る段以前に値を得ている (`(x, λ)` 自身が終端であるときは同じ段で
  ある)。

**証明**

<1>0. ρ-歩みの各段 `(x, λ) → (x', λ')` について、`ρ` の上で `x'` は `x` より前に値を得ている。
  BY A11, D3, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: collect_bindings
  L8 (A) が挙げる `origin` を呼ぶ 6 つの腕が渡す `x'` は、`Binding::Move(y)` の `y`、`Binding::Llvm` の
  `args[j]`、`Binding::Field(container, _)` の `container`、`Binding::Payload(scrut, _)` の `scrut`、
  そして `Binding::Join(arm_results)` の `ρ` が選んだアームの結果変数である。`collect_bindings` より、
  前の 4 つでは `x'` は `x` を束縛する節点 (`Let`、`Destructure`、`Match`) がその右辺・容器・scrutinee
  として名指す変数であり、A11 よりその使用はその位置でスコープに入っている束縛に解決するので、`ρ` の
  上でその節点より前に値を得ている。`Binding::Join` では、`collect_bindings` が `arm_results` に置くのは
  各アーム本体の `returned_var`、すなわちそのアーム本体の終端の `Ret` が名指す変数であり、D3 より `ρ` は
  そのアーム本体を辿ってから `Match` の継続へ進むので、その変数が値を得るのは `x` が値を得る前である
  (アーム本体の中で束縛されるか、A11 よりアームの外の、スコープに入っている束縛である)。

<1>1. `origin` のどの呼び出しも停止し、したがってその実際の呼び出しの木は有限である。
  BY P2, A11
  P2 は「`origin(x, π)` は … `π` を問わず panic せずに答えを返し、停止する」と述べる。停止する呼び出しが
  直接行う `origin` の呼び出しは有限個であり、その入れ子も有限である。A11 (スコープの規律) が P2 の立つ
  仮定である。

<1>1a. `origin(x, λ)` の計算を、memo を使わずに `origin_inner` の再帰呼び出しをそのまま展開して得られる
       木も有限である。
  BY L8a
  L8a (ii) がこれである。ρ-歩み (DEF ρ-歩みと ρ-終端) は `origin_inner` が呼ぶ相手を辿るので、memo が
  当たった位置でも止まらない。この展開の有限性が要るのはそのためである。

<1>2. ρ-歩みの列は <1>1a の木の 1 本の枝である。
  BY DEF ρ-歩みと ρ-終端, CODE src/rc_ir/ownership.rs: origin_inner
  `origin_inner` が呼ぶ `origin` の相手のうち 1 つを選んで進むので、列は木の根から下る 1 本の枝である。

<1>3. QED
  BY <1>0, <1>1a, <1>2, L8
  有限の木の枝は有限で、その最後の対では `origin_inner` が `origin` を呼ばない。各段が L8 (A) の
  (i)-(iv) を満たすことは、L8 の言明が `Binding::Join` の腕について `ρ` が選んだアームを主語に
  することによる。これが (i) である。(ii) は <1>0 を列の各段に当て、列の長さについての帰納法で得られる。

##### DEF 別名類

`ρ` の上のスロット (D6) -- `ρ` の上で値を得ている変数の inhabited な boxed leaf の対 -- を、`T_ρ` が
等しいという関係で分けた同値類を**別名類**と呼ぶ。スロット `(x, λ)` が属する別名類を `C_ρ(x, λ)` と書く。

D20 と L8 (A) より、ρ-歩みの各段は D20 の別名の辺である -- D20 が挙げる 6 種の辺は、L8 (A) が挙げる
`origin` を呼ぶ 6 つの腕 (`Move`、`Join` の取ったアーム、unbox 容器の `Field`、unbox union の変位
`Payload`、catch-all の `Payload`、`Llvm` の単一 `Arg`) にちょうど対応する。

##### L12a (別名類はオブジェクトを決める)

**言明**。1 つの別名類 `C` に属する 2 つのスロットは同じオブジェクトを指す。そのオブジェクトを `obj(C)`
と書き、`obj(C) = obj(T_ρ(C))` である。

**証明**

<1>1. `(x, λ)` が `ρ` の上のスロット (D6) であるとき、その ρ-歩み `(x', λ')` も `ρ` の上のスロットで
      あり、`obj(x, λ) = obj(x', λ')` である。
  BY L8, L12, D6
  D6 よりスロットとは、`ρ` の上で値を得た変数と、その値の inhabited な boxed leaf の対である。L12 より
  ρ-歩みの各段は L8 (A) の (i)-(iv) を満たす。(i) と (ii) より `λ' ∈ boxed_leaf_paths(ty(x'))` であり
  `x'` は `ρ` の上で値を得ている。(iii) より `λ` が inhabited であることと `λ'` が inhabited であることは
  同値であり、`(x, λ)` がスロットなので `λ` は inhabited、よって `λ'` も inhabited である。すなわち
  `(x', λ')` はスロットである。(iv) は leaf が inhabited であるときに `obj(x, λ) = obj(x', λ')` を
  与える条件付きの主張であり、その条件はいま満たされている。

<1>2. QED
  BY <1>1, L12, DEF 別名類
  L12 より `(x, λ)` から `T_ρ(x, λ)` への ρ-歩みの列は有限である。列の長さについての帰納法で、
  <1>1 がその各段について「次の対もスロットであり、指すオブジェクトが等しい」を与えるので、列の
  各対はスロットであり、`obj(x, λ) = obj(T_ρ(x, λ))` である。とくに `T_ρ(x, λ)` はスロットなので
  `obj(T_ρ(x, λ))` は定まる。1 つの別名類の 2 つのスロットは同じ `T_ρ` を持つので、どちらも同じ
  オブジェクトを指す。

##### L13 (`acted_on` は ρ-歩みで縮まない)

**言明**。`(x, λ)` の ρ-歩みが `(x', λ')` であるとき、`acted_on(x, λ) ⊇ acted_on(x', λ')` である。

**証明** L8 の場合分けのうち `origin` を呼ぶ腕で分ける。

<1>1. CASE `Binding::Move`、`Binding::Llvm` の単一 `Arg` の腕、unbox 容器の `Binding::Field`、
      `Binding::Payload` の catch-all か unbox 変位の腕。これらの腕は `origin(x', λ')` の値をそのまま
      返すので `origin(x, λ) = origin(x', λ')` であり、両辺の `acted_on` は等しい。
  BY CODE src/rc_ir/ownership.rs: origin_inner

<1>2. CASE `Binding::Join(arm_results)` の腕。この腕は
      `C = ⋃_{arm_result} acted_on(arm_result, λ)` を作り、`Origin::of_candidates(C, &(x.name, λ))` を
      返す。`x' = arm_results[j]`、`λ' = λ` である。
  <2>1. `acted_on(x', λ') ⊆ C` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
    この腕は各 `arm_result` について `origin(vars, type_env, &arm_result.name, path).acted_on()` の各元を
    `candidates` に入れる。`x'` はその `arm_result` の 1 つである。
  <2>2. CASE `C` の元数が 1 である。返り値は `Origin::Exactly(c)` (`c` は `C` の唯一の元) なので
        `acted_on(x, λ) = {c} = C` である。D15 より `acted_on(x', λ')` は空でないので、<2>1 と
        合わせて `acted_on(x', λ') = {c}` であり、包含は等号として成り立つ。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, D15, <2>1
  <2>3. CASE `C` の元数が 2 以上である。返り値は `Origin::Join { identity: (x.name, λ), candidates: C }`
        なので、D15 より `acted_on(x, λ) = {(x.name, λ)} ∪ C ⊇ C ⊇ acted_on(x', λ')` である。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, D15, <2>1
  <2>4. QED
    BY <2>2, <2>3
    `of_candidates` は `candidates.len()` が 1 かそれ以外かで分かれる。

<1>3. QED
  BY <1>1, <1>2, L8
  L8 の (A) が挙げる `origin` を呼ぶ腕はこの 2 つの場合で尽きる。

##### L14 (`identity` は自分の別名類のスロットである)

**言明**。`ρ` の上のスロット `(x, λ)` について、`o = origin(x, λ).identity()` は `ρ` の上のスロットで
あり、`C_ρ(o) = C_ρ(x, λ)` である。とくに `obj_ρ(o) = obj(C_ρ(x, λ))` である。

**証明**

<1>1. `o` は `ρ` の上のスロットである。
  BY L9
  L9 より `o = (u, σ)` について `u` は `ρ` の上で値を得ており、`σ ∈ boxed_leaf_paths(ty(u))` であり、
  `λ` が inhabited であることと `σ` が inhabited であることは同値である。`(x, λ)` はスロットなので
  `λ` は inhabited であり、よって `σ` も inhabited である。

<1>2. `o` は `(x, λ)` から ρ-歩みを辿る列の上の対である。
  <2>1. `origin_inner` が `origin` を呼ばない対では、その値は `Origin::Exactly` であり、その
        `identity()` はその対自身である。
    BY L8, D15
    L8 (B) が値を、D15 が `Origin::Exactly(p).identity() = p` を与える。
  <2>2. `origin_inner` が `Binding::Join` 以外の腕で `origin(x', λ')` を呼ぶとき、腕の値は
        `origin(x', λ')` の値そのものなので、`identity()` は `(x', λ')` のそれと等しい。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Binding::Join` の腕では、`of_candidates` が返す値の `identity()` は、候補が 2 元以上のときは
        `(x.name, λ)` 自身であり、1 元のときは `ρ` が選んだアームの結果対の `identity()` と等しい。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Origin::of_candidates, D15
    候補が 1 元 `{c}` のとき返り値は `Origin::Exactly(c)` である。この腕は各 `arm_result` の
    `acted_on()` の元を候補に入れるので、`ρ` が選んだアームの結果対の `acted_on()` は `{c}` に含まれ、
    D15 より `acted_on()` は `identity()` を先頭に持つ空でない列なので、その `identity()` は `c` である。
  <2>4. QED
    BY <2>1, <2>2, <2>3, L12
    `(x, λ)` から ρ-歩みを辿ると、<2>2 と <2>3 の 1 元の場合では `identity()` が次の対のそれに等しく、
    <2>1 と <2>3 の 2 元以上の場合ではその対自身になる。L12 より列は有限なので、`o` はこの列の上の
    どれかの対である。

<1>3. QED
  BY <1>2, L12, DEF 別名類, L12a
  <1>2 より `o` は `(x, λ)` から ρ-歩みを辿る列の上にあるので、L12 より `T_ρ(o) = T_ρ(x, λ)` であり、
  DEF 別名類より同じ別名類に属する。L12a より `obj_ρ(o) = obj(C_ρ(x, λ))` である。

#### 7.5.3 要る前提

##### DEF 節点の時点

`ρ` の上の節点 `n` について、`τ(n)` を、`ρ` を辿る活性化の**位置** (D23) が `n` に着いた直後の時点 --
すなわち、その活性化が `n` を実行する段の直前の時点 -- とする。D23 より活性化の位置は段ごとに実行路に
沿って 1 つ進み、D2 より `B` は木で D3 の実行路は根から各節点を高々 1 度通るので、位置が `n` に着く段は
`ρ` の上にちょうど 1 つあり、`τ(n)` は 1 つに定まる。走査の側では、`L2` (ii) より `n` の訪問はちょうど
1 回起こり、その入口の `pending` -- 第 7.1 節の `pending(n)` -- も 1 つに定まる。

**「位置が `n` にある時点」は 1 つとは限らない。** `n` が `App` の節点であるとき、(E3) より活性化は
呼び出し先の活性化が終わるまで中断中であり、その間ずっと位置は `n` にある。オペランドを適用する `Llvm` の
節点も同じである ((E2))。A19 の「各時点」はその中断中の時点を含む -- README は「**「各時点」は、その
活性化が生きている (D23) 間のすべての時点であり、入れ子の呼び出しで中断中の時点を含む。**」と書く。
`τ(n)` はその区間の**最初の**時点であり、**この文書が A19 を当てるのはこの点だけである。**

**以下では、実行時の時点 `τ(n)` と、走査における `n` の訪問の入口とを、`n` の 1 つの時点として扱う。**
D27 が走査中の位置を「節点の訪問の入口」と定めるので、この対応が 2 つの水準の量を同じ添字で書けるように
する。時点を引数に取る量については `held_ρ(n, C) = held_ρ(τ(n), C)` と略記する。

##### DEF 類ごとの参照

**計数下の**別名類 `C` -- `obj(C)` が D26 の意味で計数下である類 -- について、`ρ` を辿る活性化の各時点
`τ` における `held_ρ(τ, C)` を次の規則で定める。グローバル状態の類には定めない。

| 事象 | `held_ρ(·, C)` の変化 |
|---|---|
| `C` の ρ-終端が D10 の生成で作られる | 1 から始まる |
| `C` の ρ-終端が、所有する (D14) パラメータ・capture の leaf である | 1 から始まる (D10 の初期値) |
| `C` の ρ-終端が、借用する (D14) パラメータ・capture の leaf である | 1 から始まる (呼び出し元が持つ参照、D14) |
| `Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき +1 |
| `Release(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき -1 |
| `(w, μ) ∈ C` の D9 の消費 | -1 |

**開始の時点。** 最初の 3 行が置く開始値は、`C` の ρ-終端 `T_ρ(C) = (u, σ)` の変数 `u` が値を得る時点で
置かれる -- `u` がパラメータ・capture であるときは活性化が始まる時点、そうでないときは `u` を束縛する節点を
実行する段の直後である。`held_ρ(τ, C)` はその時点以後の `τ` についてだけ定まる。**`ρ` の上の類でも、
`n` の時点でまだ終端が値を得ていないものは在る** -- 第 7.5.8 節の `C2` の類 `C_y` は
`Let(y, App(id, [o]), ・)` の段の後に始まるので、それより前の節点では値を持たない。**この文書が
`held_ρ(n, C)` と書くのは `C` の開始の時点が `τ(n)` 以前であるときだけであり、`C` を渡る総和はどれも
その条件を付ける。**

**3 つの開始行は、計数下の類を作りうる ρ-終端を尽くす。** `L8` の (B) より ρ-終端とは `origin_inner` が
`origin` を呼ばない対であり、そうなる腕は 6 つある -- 束縛を持たない名前 (`None`)、`Binding::Param`、
`Binding::Producer`、`Binding::Llvm` の `origin_from_leaves_under` の枝、boxed 容器の `Binding::Field`、
boxed scrutinee の `Binding::Payload` である (`CODE src/rc_ir/ownership.rs: origin_inner`)。

- `Binding::Param` は第 2 行と第 3 行が扱う。D14 より、パラメータ・capture の各 unit はその関数が所有するか
  借用するかのどちらかであり、この 2 行がその 2 つである。
- 残る 4 つの腕は第 1 行が扱う。`collect_bindings` は `RcRhs::App(..)` と `RcRhs::Closure(..)` にだけ
  `Binding::Producer` を置くので (`CODE src/rc_ir/ownership.rs: collect_bindings`)、この 4 つの腕が名指す
  位置は 5 つあり、D10 の生成の表はそのそれぞれに 1 行を持つ -- 「`App(callee, args)` の結果の各 boxed
  leaf」「`Closure(f, caps)` の結果 (capture object)」「`Llvm(gen, args)` の結果の leaf のうち、
  `result_prov` の宣言が単一の `Arg(j, σ)` **でない**もの」「boxed 容器の `Destructure` の各名前付き
  フィールドの各 leaf」「boxed union の変位アームの payload の各 leaf」である。`Llvm` の行がこの腕に対応するのは、`as_arg_projection` が `None` を
  返すことと宣言が単一の `Arg(j, σ)` でないことが同値だからである (第 5 節の `<2>3`)。
- **束縛を持たない名前 (`None` の腕) を ρ-終端とする類は、計数下ではない。** `vars.bindings` に束縛を持たない
  `RcVar` の名前は最上位の記号の名前である -- A13 が「**束縛名に限らない** -- 直接呼び出しが名指す関数の
  名前と、グローバル値を読む `RcVar` の名前 (`origin_inner` の束縛を持たない腕が扱う) も含む」と、この腕が
  扱う名前をその 2 種として挙げる。A8 より、グローバル値が到達するオブジェクトは記憶域にグローバルを表す
  状態を持ち、D26 よりそれは計数下ではない。よって `obj(C)` が計数下である類の ρ-終端はこの腕に来ず、
  この表が行を持つ必要はない。README の A19 が「計数下の類に限るのは、グローバル値を終端とする類に `held` の
  開始値を与える行が無いからである (D26)」と述べるのがこの点である。

**表の 6 行は開始値と増減だけを定める。** 借用する終端の行が定めるのも開始値だけであり、その後 `held` が
どう動くかは残りの 3 行が決める。**この行についての命題が P14a である。** README 第 5 節の文面は次の
とおりである。

> **P14a** (借用する終端の類は活性化の間 参照を持つ)。`borrow_ify` の出力の各本体、各実行路、各活性化に
> ついて、ρ-終端が借用する (D14) パラメータ・capture の leaf である**計数下**の別名類 (D26) は、活性化の
> 間ずっと参照を少なくとも 1 つ持つ。

`計数下` の限定は、この表が計数下の類にしか `held_ρ` を定めないことと合っている -- README も「計数下に
限るのは、グローバル状態のオブジェクトを指す leaf が D8 の意味の参照を持たないからである (D26)。
グローバル値を借用位置の引数に渡す活性化では、限定の無い言明は偽になる」と述べる。ここに引くのは、
借用する終端の行が開始値だけを与えることと、その後の下限が別の命題で与えられることを分けて示すためで
ある。**第 7.5.4 節がこの下限を読む** -- A19 (i) の `d` は借用終端の類から 1 を引くので、その類の
`d` が非負であることに `held_ρ(n, C) ≥ 1` が要る。

D8 は同じオブジェクトへの参照を区別しないので、この勘定は「どの参照がどの類のものか」を決める取り決めで
ある。取り決めが実行時のカウントと整合することを A19 (i) が要求する。

##### DEF bumps

計数下の別名類 `C` と `ρ` の上の節点 `n` について、
`bumps_ρ(n, C) = Σ_{p ∈ pending(n)} Σ_{o : C_ρ(o) = C} B_ρ(n, p)[o]` と定める。内側の和は、`ρ` の上の
スロットである名前 `o` のうち `C_ρ(o) = C` であるものを渡る。L14 より、`B_ρ(n, p)` が個数を付けている
名前はどれも `ρ` の上のスロットであり、ちょうど 1 つの別名類に属する。

これが A19 (ii-b) の言う「走査がその類について `pending` に数えている bump の個数」である。走査の
`pending` は節点の訪問の入口ごとに定まるので、この量の添字は節点であり、DEF 節点の時点により
`bumps_ρ(τ(n), C) = bumps_ρ(n, C)` と読む。

##### A19 (README 第 4 節)

**この文書は README の A19 を前提として引く。** 使う 3 つの節の文面は次のとおりである (README 第 4 節)。

> - **(i)** 各時点、各計数下オブジェクト `O`、その時点で生きている各活性化 `a` について、次が成り立つ。
>   `a` の計数下の別名類のうち `obj(C) = O` であり開始の時点がその時点以前であるものの全体を `S` とし、
>   各類について `d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置くと、
>
>   > `H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]`
> - **(ii-a) 由来の形** -- 読む者: P14、P18c。各時点と各計数下の別名類について、その類が持つ参照の個数は
>   非負であり、読む構文と `Retain`/`Release` がその類を名指す時点では 1 以上である。**非負であることは、
>   終端の `Ret` の消費を行った直後の時点についても言う。**
> - **(ii-b) 帳簿の形** -- 読む者: P18a、P18c、P19、P21。各時点と各計数下の別名類について、走査がその類に
>   ついて `pending` に数えている bump の個数を `bumps`、その類が持つ参照の個数を `held` とすると、
>   **`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である。**

README は「**`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の
入力) の両方**について、次の 3 つを仮定する」と範囲を定めているので、`cancel` の入力についてこの 3 つの
節がどれも使える。この文書の記法では、(ii-b) は「`bumps_ρ(n, C) ≥ 1` である節点 `n` では
`held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)`」である。

**(i) の `d` と角括弧が第 7.5.4 節に効く。** (i) の総和が渡るのは、固定した活性化の計数下の別名類の
うち `obj(C) = O` であり開始の時点がその時点以前であるものの全体であり、これは第 7.5.4 節の `S(n, O)`
すなわち `L17` の範囲と同じものである。各項が `held` そのものではなく
`d(C) = held(C) - [借用終端ならば 1]` であることと、`S` に借用終端の類が在るときに右辺へ `+1` が立つ
ことが、第 7.5.4 節の 2 つの場合を分ける。

条件を落とした形が使えないことも README が述べる。無条件の `held ≥ 1 + bumps` は `bumps = 0` の時点で
`0 ≥ 1` を要求するので偽であり (第 7.5.7 節の `f` の `Release(b, [])` の直後がその時点である)、
`held ≥ bumps` は真だが弱すぎて P18a が要る形を出さない (第 7.5.8 節の `C2` の類 `C_o` が
`held = bumps = 1` の時点を持つ)。

##### DEF A19 の `held` の読み

A19 は `held` を「その類が持つ参照の個数」と書く。D8 は同じオブジェクトへの参照を区別しないので、
**どの参照がどの類のものかは帰属の取り決めを定めて初めて定まる。** `bumps` について D27 が走査自身の帰属を
採ったのと同じ事情である。README の A19 はその取り決めをこの文書に委ねている -- 「**別名類と `held` の
定義がこの文書でなく `p13-disposals-and-pending.md` に在るのは、どちらも走査の帳簿と噛み合う形でしか
書けず、その帳簿を定めるのがその文書だからである。**」

**この文書は `DEF 類ごとの参照` の表をその取り決めとして採り、A19 (i)・(ii-a)・(ii-b) の `held` を
`held_ρ` と読む。** A19 を引く段 -- `L14b` と第 7.5.4 節 -- は `DEF 類ごとの参照` をこの読みとして
併せて引く。

**(i) はこの読みの下でも (ii) と同じ量を足さない。** (i) が足すのは、この読みの `held_ρ` そのもの
ではなく、ρ-終端が借用するパラメータ・capture の leaf である類について 1 を引いた `d(C)` である。
README はその理由を「(ii-a)・(ii-b) と P14a は、借用する終端の類が `held ≥ 1` を持つ読みを要求する --
借用位置へ渡した参照を呼び出し先が読んでよいことがそこに載っているからである。(i) はその参照を
呼び出し元の類が既に数えている。`d` と角括弧が、その食い違いを (i) の側で解く」と述べる。**類ごと落とす
形ではない** -- README は「**借用終端の類を丸ごと落とす形は弱すぎる。** … `d` が引くのは開始の 1 だけで
ある」と続ける。この文書はその差を第 7.5.4 節で扱う。

##### L14b (`held` は `bumps` 以上である)

**言明**。`ρ` の上の節点 `n` と、開始の時点 (DEF 類ごとの参照) が `τ(n)` 以前である計数下の別名類 `C` に
ついて、`bumps_ρ(n, C) ≥ 0` であり、`held_ρ(n, C) ≥ bumps_ρ(n, C)` である。**開始の条件が要る** --
`held_ρ(n, C)` はその条件の下でだけ定まる。

**証明**

<1>1. CASE `bumps_ρ(n, C) = 0`。A19 (ii-a) より `held_ρ(n, C) ≥ 0` である。
  BY A19, DEF A19 の `held` の読み, DEF 類ごとの参照, DEF 節点の時点
  A19 (ii-a) は「各時点と各計数下の別名類について、その類が持つ参照の個数は非負である」と述べる。
  DEF A19 の `held` の読みより、その個数はこの文書の `held_ρ` であり、DEF 節点の時点より `n` の
  時点は A19 の言う時点の 1 つである。

<1>2. CASE `bumps_ρ(n, C) ≥ 1`。A19 (ii-b) より `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C) ≥ bumps_ρ(n, C)` で
      ある。
  BY A19, DEF A19 の `held` の読み, DEF 類ごとの参照, DEF bumps, DEF 節点の時点
  A19 (ii-b) は「`bumps ≥ 1` である時点では `held ≥ 1 + bumps`」と述べる。DEF bumps より、走査が
  `pending` に数えている bump の個数はこの文書の `bumps_ρ(n, C)` である。

<1>3. QED
  BY <1>1, <1>2, L11, D15, DEF bumps
  `bumps_ρ(n, C)` は `B_ρ(n, p)[o]` の有限和であり (DEF bumps)、L11 より各項は非負である -- 活性な
  `o` では `p.outstanding[o]` に等しく (L11 (i))、`References` は `VarPath` から個数への写像なので
  これは非負であり (D15)、活性でない `o` では 0 である (L11 (ii))。よって `bumps_ρ(n, C)` は非負整数で
  あり、これが言明の前半である。後半は、`bumps_ρ(n, C)` が 0 か 1 以上かで場合が尽きるので、
  <1>1 と <1>2 が与える。

##### L17 (`N` は別名類ごとの `bumps` の和である)

**言明**。`ρ` の上の節点 `q` と**計数下** (D26) のオブジェクト `O` について、

`N_ρ(q, O) = Σ_{C ∈ S(q, O)} bumps_ρ(q, C)`

である。`S(q, O)` は、`obj(C) = O` であって開始の時点 (DEF 類ごとの参照) が `τ(q)` 以前である計数下の
別名類 `C` の全体である。**開始の条件が要る** -- `held_ρ(q, C)` はその条件の下でだけ定まるので、
第 7.5.4 節が同じ範囲で `held_ρ` を足せるのはこの形による。

**`O` の「計数下」に時点は要らない。** D26 の最後の段落が「1 つの活性化の間、そこに現れるオブジェクトが
計数下であるかどうかは変わらない … 命題が『各時点の計数下オブジェクト』を量化するとき、その集合は
活性化の間ずっと同じである」と述べるからである。`obj(C)` が各別名類について定まることは `L12a` が与える。

**証明**

<1>1. `obj(C)` は各別名類 `C` について定まる。
  BY L12a
  L12a は「1 つの別名類 `C` に属する 2 つのスロットは同じオブジェクトを指す。そのオブジェクトを
  `obj(C)` と書き、`obj(C) = obj(T_ρ(C))` である」と述べる。

<1>2. 両辺はどちらも、`pending(q)` の各要素 `p` と名前 `o` にわたる `B_ρ(q, p)[o]` の和であり、`o` の
      動く範囲だけが違う。左辺は「活性 (DEF 名前の活性) であって `obj_ρ(o) = O` である `o`」、右辺は
      「`ρ` の上のスロットであって `C_ρ(o) ∈ S(q, O)` である `o`」を渡る。
  BY DEF `N`, DEF bumps
  DEF `N` は `N_ρ(q, O) = Σ_{p ∈ pending(q)} Σ_{o} B_ρ(q, p)[o]` (内側は活性で `obj_ρ(o) = O` である
  `o`)、DEF bumps は `bumps_ρ(q, C) = Σ_{p ∈ pending(q)} Σ_{o : C_ρ(o) = C} B_ρ(q, p)[o]` (内側は `ρ` の
  上のスロットである `o`) である。右辺は `S(q, O)` の類を渡るので、その二重和は `C_ρ(o) ∈ S(q, O)` で
  あるスロット `o` を渡ることと同じである。

<1>3. `B_ρ(q, p)[o] ≥ 1` であるどの名前 `o` も、<1>2 の 2 つの範囲の両方に入る。
  <2>1. `o` は、`ρ` の上で `q` より前にある `Retain(v, π)` の `π` の下の inhabited かつ計数下の leaf `λ`
        の `origin(v, λ).identity()` である。
    BY DEF bump の帰属, DEF 実行時の作用, P16, L11
    DEF bump の帰属より `B_ρ(·, p)` の初期値は `ActRefs^inh_ρ` であり、その後の操作は名前を増やさない。
    DEF 実行時の作用より `ActRefs^inh_ρ` が名前を付けるのは `Inh_ρ(v, π, ·)` の leaf、すなわち
    inhabited かつ計数下の leaf である。その初期値を置くのは `p` を `pending` に入れた `Retain` の訪問で
    あり、P16 (a) より `p.node` は「その位置までに訪れた `Retain` 節点」である。L11 より帰属の表は `ρ` の
    上の場合を尽くすので、その訪問は `ρ` の上で `q` より前にある。
  <2>2. `(v, λ)` は `ρ` の上のスロットであり、`o` も `ρ` の上のスロットであって
        `C_ρ(o) = C_ρ(v, λ)` である。
    BY L14, <2>1, DEF 別名類
    L14 は「`ρ` の上のスロット `(x, λ)` について、`o = origin(x, λ).identity()` は `ρ` の上のスロットで
    あり、`C_ρ(o) = C_ρ(x, λ)` である」と述べる。
  <2>3. `obj(C_ρ(o)) = obj_ρ(o) = obj(v, λ)` であり、これは計数下である。
    BY L14, L12a, <2>1, <2>2
    L14 の後半が `obj_ρ(o) = obj(C_ρ(x, λ))` を与え、<1>1 の `obj(C)` と一致する。<2>1 より
    `obj(v, λ)` は計数下である。
  <2>4. `o` は活性である。
    BY DEF 名前の活性, <2>1, <2>2, <2>3
    DEF 名前の活性は、`origin(x, λ).identity() = o` を満たす対 `(x, λ)` で、`λ` が inhabited であり
    `obj(x, λ)` が計数下であるものが在ることを要求する。`(v, λ)` がその対である。
  <2>4a. `C_ρ(o)` の開始の時点は `τ(q)` 以前である。
    BY L12, A11, DEF 類ごとの参照, DEF 節点の時点, <2>1, <2>2
    DEF 類ごとの参照より `C_ρ(o)` の開始の時点は `T_ρ(C_ρ(o))` の変数が値を得る時点である。<2>2 より
    `C_ρ(o) = C_ρ(v, λ)` なのでその終端は `T_ρ(v, λ)` であり、L12 (ii) よりその変数は `v` が値を得る段
    以前に値を得ている。<2>1 の `Retain(v, π)` は `ρ` の上で `q` より前にあり、A11 より `v` はその節点で
    スコープに入っている束縛に解決するので、`v` が値を得るのは `τ(q)` より前である。
  <2>5. QED
    BY <2>2, <2>3, <2>4, <2>4a
    <2>4 と <2>3 より `o` は左辺の範囲に入り、<2>2 と <2>3 と <2>4a より右辺の範囲に入る。

<1>4. QED
  BY <1>2, <1>3
  <1>2 の 2 つの範囲は、<1>3 より、`B_ρ(q, p)[o] ≥ 1` である名前をすべて含む。範囲の外の項は
  `B_ρ(q, p)[o] = 0` なので和に寄与しない。よって 2 つの和は等しい。

#### 7.5.4 A19 から P18a を出す段

**証明**

<1>1. SUFFICES ASSUME NEW `ρ` の上の節点 `n`、NEW 計数下 (D26) オブジェクト `O`、`N_ρ(n, O) ≥ 1`
      PROVE `H(O) ≥ N_ρ(n, O) + 1`
  BY DEF `N`, DEF INV, DEF bump の帰属, DEF 名前の活性, D27, L11
  7.5.1 が述べるとおり、この形が INV(n) であり、INV は P18a そのものである。P18a の
  `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` は、走査中の位置を節点の訪問の入口に取り (D27)、
  `B(p, ρ)` を DEF bump の帰属で読むと `N_ρ(n, O)` である。内側の和を活性な名前に制限してよいのは
  L11 (ii) が活性でない名前の `B_ρ` を 0 とするからであり、`obj_ρ(o)` が定まるのは DEF 名前の活性に
  よる。**以下のステップは、この ASSUME が導入した `n`、`O` と仮定 `N_ρ(n, O) ≥ 1` の下で書く。**

<1>2. `N_ρ(n, O) = Σ_{C ∈ S(n, O)} bumps_ρ(n, C)` である。`S(n, O)` は L17 の範囲、すなわち固定した
      活性化の計数下の別名類のうち `obj(C) = O` であり開始の時点が `τ(n)` 以前であるものの全体である。
  BY L17, <1>1
  <1>1 の `O` は計数下である。

<1>2a. DEFINE `d(C) == held_ρ(n, C) - [C の ρ-終端が借用する (D14) パラメータ・capture の leaf
       ならば 1]`
       以下の各ステップはこの記法を `S(n, O)` の類についてだけ使う。そこで `held_ρ(n, C)` が定まる
       ことは <1>3 が述べる。

<1>3. ASSUME NEW `C ∈ S(n, O)`
      PROVE  `held_ρ(n, C)` は定まり、`d(C) ≥ bumps_ρ(n, C)` である。
  <2>1. `held_ρ(n, C)` は定まる。
    BY L17, DEF 類ごとの参照
    L17 より `S(n, O)` の各類は計数下であり、その開始の時点は `τ(n)` 以前である。DEF 類ごとの参照は
    計数下の類について、開始の時点以後の各時点で `held_ρ` を定める。
  <2>2. CASE `C` の ρ-終端が借用する (D14) パラメータ・capture の leaf でない。
    BY <2>1, L14b, L17
    このとき `d(C) = held_ρ(n, C)` である。L17 より `C` は計数下で開始の時点が `τ(n)` 以前なので
    L14b の仮定を満たし、L14b が `held_ρ(n, C) ≥ bumps_ρ(n, C)` を与える。
  <2>3. CASE `C` の ρ-終端が借用する (D14) パラメータ・capture の leaf であり、`bumps_ρ(n, C) ≥ 1` で
        ある。
    BY A19, DEF A19 の `held` の読み, DEF 類ごとの参照, DEF bumps, DEF 節点の時点, <2>1
    A19 (ii-b) は「`bumps ≥ 1` である時点では `held ≥ 1 + bumps`」と述べる。DEF A19 の `held` の読み
    より A19 の `held` はこの文書の `held_ρ` であり、DEF bumps より A19 の `bumps` は `bumps_ρ` で
    あり、DEF 節点の時点より `τ(n)` は A19 の言う時点の 1 つである。よって
    `d(C) = held_ρ(n, C) - 1 ≥ bumps_ρ(n, C)` である。
  <2>4. CASE `C` の ρ-終端が借用する (D14) パラメータ・capture の leaf であり、`bumps_ρ(n, C) = 0` で
        ある。
    BY P14a, DEF 類ごとの参照, DEF 節点の時点, <2>1, L17
    P14a は「`borrow_ify` の出力の各本体、各実行路、各活性化について、ρ-終端が借用する (D14)
    パラメータ・capture の leaf である**計数下**の別名類 (D26) は、活性化の間ずっと参照を少なくとも
    1 つ持つ」と述べる。第 1 節より `B` は `cancel` の入力、すなわち `borrow_ify` の出力の本体であり、
    L17 より `C` は計数下である。DEF 類ごとの参照がその「参照の個数」の帰属を定め、DEF 節点の時点より
    `τ(n)` はこの活性化が生きている間の時点である。よって `held_ρ(n, C) ≥ 1` であり、
    `d(C) = held_ρ(n, C) - 1 ≥ 0 = bumps_ρ(n, C)` である。
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, L14b, L17
    3 つの CASE は、`C` の ρ-終端が借用するパラメータ・capture の leaf であるかどうか (排中律) と、
    そうであるときの `bumps_ρ(n, C)` が 0 か 1 以上か (L17 より `C` は L14b の仮定を満たし、L14b より
    `bumps_ρ(n, C)` は非負整数なのでこの 2 つで尽きる) で尽きる。<2>1 より `held_ρ(n, C)`、したがって
    `d(C)` は定まる。

<1>4. `bumps_ρ(n, C0) ≥ 1` である類 `C0 ∈ S(n, O)` が少なくとも 1 つ在る。
  BY <1>1, <1>2, L14b, L17
  L17 より `S(n, O)` の各類は L14b の仮定を満たすので、L14b より `bumps_ρ(n, ・)` は非負整数である。
  <1>1 と <1>2 より和が 1 以上なので、ある項が 1 以上である。

<1>5. `H(O) ≥ Σ_{C ∈ S(n, O)} d(C) + [S(n, O) に ρ-終端が借用するパラメータ・capture の leaf である類が
      在るならば 1]` である。
  BY A19, DEF A19 の `held` の読み, DEF 類ごとの参照, DEF 節点の時点, L12a, L17, <1>1, <1>2, <1>2a
  A19 (i) を、時点 `τ(n)` と、`ρ` を辿るこの活性化 -- `n` の位置に在るので `τ(n)` に生きている (D23)
  -- と、計数下オブジェクト `O` に当てる。(i) の `S` は「その活性化の計数下の別名類のうち
  `obj(C) = O` であり開始の時点がその時点以前であるものの全体」であり、L17 の `S(n, O)` がこれで
  ある。(i) の `d(C)` は `held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` であり、
  DEF A19 の `held` の読みより A19 の `held` はこの文書の `held_ρ` なので、<1>2a の `d` と同じもので
  ある。`obj(C)` が各別名類について定まることは L12a が与える。

<1>6. CASE <1>4 の `C0` の ρ-終端が借用する (D14) パラメータ・capture の leaf でない。
  <2>1. `d(C0) ≥ 1 + bumps_ρ(n, C0)` である。
    BY A19, DEF A19 の `held` の読み, DEF 類ごとの参照, DEF bumps, DEF 節点の時点, <1>3, <1>4
    この CASE の仮定より `d(C0) = held_ρ(n, C0)` である。<1>4 より `bumps_ρ(n, C0) ≥ 1` なので、
    A19 (ii-b) が `held_ρ(n, C0) ≥ 1 + bumps_ρ(n, C0)` を与える。
  <2>2. `Σ_{C ∈ S(n, O)} d(C) ≥ N_ρ(n, O) + 1` である。
    BY <1>2, <1>3, <1>4, <2>1
    `C0` の項に <2>1 を、`S(n, O)` の他の各項に <1>3 を当てて足すと、下界は
    `Σ_{C ∈ S(n, O)} bumps_ρ(n, C) + 1` であり、<1>2 よりこれは `N_ρ(n, O) + 1` である。
  <2>3. QED
    BY <1>5, <2>2
    <1>5 の角括弧は 0 以上なので `H(O) ≥ Σ_{C ∈ S(n, O)} d(C) ≥ N_ρ(n, O) + 1` である。

<1>7. CASE <1>4 の `C0` の ρ-終端が借用する (D14) パラメータ・capture の leaf である。
  <2>1. `Σ_{C ∈ S(n, O)} d(C) ≥ N_ρ(n, O)` である。
    BY <1>2, <1>3
    `S(n, O)` の各項に <1>3 を当てて足すと、下界は `Σ_{C ∈ S(n, O)} bumps_ρ(n, C)` であり、
    <1>2 よりこれは `N_ρ(n, O)` である。
  <2>2. <1>5 の角括弧は 1 である。
    BY <1>4
    <1>4 より `C0 ∈ S(n, O)` であり、この CASE の仮定よりその ρ-終端は借用するパラメータ・capture の
    leaf である。
  <2>3. QED
    BY <1>5, <2>1, <2>2
    `H(O) ≥ N_ρ(n, O) + 1` である。

<1>8. QED
  BY <1>1, <1>4, <1>6, <1>7
  <1>4 が取った `C0` の ρ-終端が借用するパラメータ・capture の leaf であるかどうかで、<1>6 と <1>7 は
  場合を尽くす (排中律)。どちらも <1>1 の PROVE を与える。

**`<1>7` の場合は空虚ではない。** 借用版 `V` がパラメータ `b` の unit `u` を借用し、その本体が
`App(h, [b])` (`h` は対応するパラメータの `u` を所有する) を持つとき、`call_rc` はその呼び出しの直前に
`Retain(b, u)` を置く (P11)。その `App` の節点 `n` の入口ではその要素が `pending` に在るので
`bumps_ρ(n, C_b) = 1` であり、`C_b` は借用終端の類である。`held_ρ(n, C_b) = 2` -- 開始の 1 と `Retain`
の 1 -- なので `d(C_b) = 1` であり、`<1>5` の角括弧の 1 と合わせて `H(O) ≥ 2 = N_ρ(n, O) + 1` になる。
実行時のカウントも 2 である (呼び出し元が持つ 1 と `Retain` の 1) ので、この位置は等号で立つ。

#### 7.5.5 A19 (ii-b) の `borrow_ify` の側

A19 (ii-b) が破れるのは、ある類の参照が減って bump の数が減らないときである。`held_ρ(·, C)` を減らす
事象は 2 つ -- `C` のスロットを名指す `Release` と、`C` のスロットの消費である。`Release` の側は L6 と
P17 が扱う (7.5.4 の前の第 4 節と、L11 の <2>2 の場合分け)。**消費の側が、依頼された問いの場所である。**

問いは次の形になる。`cancel` の入力 -- `borrow_ify(split_rc_units(insert_rc の出力))` -- において、
`(w, μ) ∈ C` の消費が起きるとき、その消費で減る分を埋める参照が `C` に在るか。`insert_rc` はそれを
「使うたびに 1 つ用意する」ことで作る。**`borrow_ify` がそれを壊しうるかを問う。**壊しうる経路は
`rewrite_rc` である。借用版では、その版が所有しない unit の `Retain`/`Release` を丸ごと落とすからである
(P10)。

##### L16 (消費が残る leaf の `Retain` は落ちないか、`call_rc` が補う)

**言明**。`borrow_ify` の出力のある版 `V` の本体において、inhabited な leaf `(p, μ)` が D9 の意味で
消費されるとする。`u = truncate_to_unit(ty(p), μ, type_env)` と置く。このとき次のどちらかが成り立つ。

- **(A)** `V` の本体に `Retain(p, u)` 節点が在り、その節点は `rewrite_rc` に落とされない。
- **(B)** その消費は `App(callee, args)` の所有位置の引数であって、`call_rc` がその呼び出しの直前に
  `Retain(p, u)` を置く。
- **(C)** `V` の本体に `Retain(p, u)` 節点が無く、その消費は `App` の所有位置の引数でもない。

(C) は `insert_rc` の側の義務に属する -- その消費が処分する参照を用意した `Retain` が本体のどこかに
在ることは、`insert_rc` の使用回数の勘定が与えるものであり、`borrow_ify` はそれを写すだけである
(第 7.5.6 節)。この補題が述べるのは、(A) と (B) の場合に `borrow_ify` が余りを壊さないことである。

**証明**

<1>1. `V` が借用版でないとき、`rewrite_rc` は節点をそのまま返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  `if !self.is_borrow_version { return rc_node(...); }` である。

<1>1a. A10 を満たす任意の型 `τ` と `t ∈ rc_units(τ, type_env)` について、`t` は次のどちらかの形で
       あり、どちらでも `truncate_to_unit(τ, t, type_env) = t` である。
       - **(α)** `t` の各添字の位置で `unit_step` が `UnitStep::Fields` を返し、`t` が名指す型 `sty` に
         ついて `unit_step(sty) = UnitStep::Unit` である。
       - **(β)** `t` の最後の添字より前の各位置で `unit_step` が `UnitStep::Fields` を返し、最後の添字の
         位置の型について `unit_step` が `UnitStep::Capture { capture_idx, .. }` を返して `t` の最後の
         添字が `capture_idx` である。
  BY A10, CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go,
     CODE src/rc_ir/ownership.rs: truncate_to_unit, CODE src/rc_ir/ownership.rs: unit_step
  A10 は `rc_units` と `truncate_to_unit` が辿る型の歩みが有限であることを与える -- 「`unpunched_field_types`
  を繰り返し取って到達する型についても同じことが成り立ち、その歩みは有限である」。
  `rc_units_go` が `out` に積むのは 2 か所である。`UnitStep::Unit` の枝は `path` をそのまま積み、
  `UnitStep::Capture` の枝は `capture_idx` を 1 つ足して積む。どちらの `path` も、`UnitStep::Fields` の枝が
  添字を 1 つずつ足しながら下って作ったものである。これが (α) と (β) である。`truncate_to_unit(τ, t)` は
  `t` の添字を順に見て、`UnitStep::Fields` のときその添字を `out` に積んで下り、`UnitStep::Capture` の
  ときその添字を積んで打ち切り、`UnitStep::Unit` のときは何も積まずに打ち切る。(α) では全添字が
  `Fields` の枝を通って積まれるので `out = t`、(β) では最後の添字で `Capture` に来てそれも積まれるので
  `out = t` である。

<1>1b. A10 を満たす任意の型 `τ` と `t ∈ rc_units(τ, type_env)` について
       `units_under(τ, t, type_env) = [t]` である。
  BY <1>1a, A10, CODE src/rc_ir/ownership.rs: units_under, CODE src/rc_ir/ownership.rs: subtree_type,
     CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go
  `units_under(τ, t)` は、`subtree_type(τ, t)` が `Some(sty)` のとき `rc_units(sty)` の各元を `t` の後ろに
  繋いだ列、`None` のとき `[t]` である。`subtree_type` は path の添字を順に見て `UnitStep::Fields` の
  ときだけ下り、それ以外では `None` を返す。<1>1a の (α) では全添字が `Fields` を通るので `subtree_type` は
  `Some(sty)` を返し、`unit_step(sty) = UnitStep::Unit` より `rc_units_go` は空の `path` を 1 つだけ積むので
  `rc_units(sty) = [[]]` であり、`units_under(τ, t) = [t]` である。(β) では最後の添字の位置で
  `UnitStep::Capture` に来るので `subtree_type` は `None` を返し、`units_under(τ, t) = [t]` である。

<1>2. 以下 `V` が借用版であるとする。`rewrite_rc` が `Retain(p, u)` を落とすのは `owns_unit(p, u)` が
      偽のときに限る。
  BY <1>1b, A2, P1, A10, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  `rewrite_rc` は `units_under(&v.ty, path, type_env)` を `self.owns_unit(v, unit)` で絞り、残った各 unit に
  ついて節点を積む。`ty(p)` は本体に現れる変数の型、すなわちプログラムに現れる型なので A10 を満たし、
  P1 が `u = truncate_to_unit(ty(p), μ, type_env) ∈ rc_units(ty(p))` を与える。A2 より `Retain(p, u)`
  節点の `path` も `ty(p)` の `rc_units` の元である。よって <1>1b より `units_under(ty(p), u) = [u]` で
  あり、絞りが落とすのは `owns_unit(p, u)` が偽のときである。

<1>3. `V` の本体に `Retain(p, u)` 節点が在るとき、`(p, u)` は `levelled_sites(V の本体, type_env)` の
      元である。また消費が `App(callee, args)` の引数 `p` の位置であるとき、`(p, u)` は `u` が
      `rc_units(ty(p))` の元であるかぎり `levelled_sites` の元である。
  BY CODE src/rc_ir/borrow.rs: levelled_sites, A2, P1, A10
  `levelled_sites` は `RcExpr::Retain(v, path, _, _) | RcExpr::Release(v, path, _, _)` の腕で
  `(v, path)` を、`RcExpr::Let(_, RcRhs::App(_, args), _)` の腕で各 `arg` と各
  `unit ∈ rc_units(&arg.ty, type_env)` について `(arg, unit)` を挙げる。`ty(p)` はプログラムに現れる型
  なので A10 を満たし、`u = truncate_to_unit(ty(p), μ)` は P1 より `rc_units(ty(p))` の元である。
  `Retain(p, u)` 節点の `path` が `u` であることは A2 による。

<1>4. CASE 消費が `App` の所有位置の引数以外 -- `App` の callee、`Closure` の capture、
      boxed/unbox の `Destructure`、終端の `Ret` -- であり、かつ `V` の本体に `Retain(p, u)` 節点が在る。
  <2>0a. `V` は入力のちょうど 1 つの関数から作られる。その関数を `F` と書き、`rename` を `V` の本体が
         `F` の本体から受けた名前替え (P9) とする。`V` が原本の版であるときは `rename` は恒等写像で
         あり (`borrow_ify` が `clone_func` を呼ぶのは借用版についてだけである)、`F = V` である。
    BY P24, P9, CODE src/rc_ir/borrow.rs: borrow_ify
    P24 は「**出力の各関数は入力のちょうど 1 つの関数から作られ**、その `fn_ty` / `ret_ty` / `params` の
    型 / `inline_into_callers` は元の関数のものに等しい」と述べる。P9 は「`clone_func` が作る借用版の
    本体は、元の本体の束縛変数を一斉に付け替えたものであり、それ以外の違いを持たない」と述べる。
  <2>0b. `V` の本体は、`F` の本体を `rename` で写したものに `RewriteCtx::rewrite` を当てたものであり、
         その書き換えが変えるのは `Retain`/`Release` 節点 (P10、P11) と `App` の callee の名前 (P12) だけ
         である。ほかの節点は種・変数・path・並びを変えずに組み直される。
    BY <2>0a, P10, P11, P12, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
       CODE src/rc_ir/borrow.rs: borrow_ify
    `rewrite_inner` の 7 つの腕を読む。`RcExpr::Retain` と `RcExpr::Release` の腕は `rewrite_rc` へ行く。
    `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は `callee` を `route` の結果に差し替え、`call_rc` が
    返す `before` と `after` の節点を前後に置き、`x` と `args` はそのまま複製する。残る腕 --
    `Let` の `Match` とそれ以外、`Destructure`、`Eval`、`Ret` -- は、束縛変数・右辺・容器・フィールド・
    scrutinee・返す変数をそのまま複製し、継続とアーム本体を書き換えた木で組み直す。
  <2>1. `collect_consumes` は `(p, μ)` を報告する。
    BY P7, CODE src/rc_ir/ownership.rs: collect_consumes_go, CODE src/rc_ir/ownership.rs: rhs_consumes
    P7 より D9 の意味で消費する構文はすべて `collect_consumes` が報告する。この 5 種について
    `collect_consumes_go` と `rhs_consumes` の判定は `own` を読まないので (`rhs_consumes` の
    `RcRhs::App` の腕は `callee` の leaf を無条件に `out` に入れ、`RcRhs::Closure` の腕は各 capture の
    leaf を無条件に入れる)、報告はどの所有の割り当てでも起きる。

  <2>1a. 次の 2 つが成り立つ。
         - `p` が `rename` の像に無いとき、`V` の本体で `p` は束縛を持たず、`origin_V(p, μ)` は
           `Origin::Exactly((p, μ))` であり、`p` は `V` の `vars.param_tys` の鍵ではない。
         - `p = rename[p0]` であるとき、`F` の本体でも `(p0, μ)` が D9 の意味で消費され、
           `origin_V(p, μ)` の候補の全体は `origin_F(p0, μ)` の候補を `rename` で写したものである。
           `V` のパラメータ `r = rename[r0]` を根に持つ候補 `(r, q)` には、`F` のパラメータ `r0` を
           根に持つ候補 `(r0, q)` が対応し、`ty(r) = ty(r0)` である。
    BY <2>0a, <2>0b, <2>1, P9, A6, A13,
       CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: VarTable::of
    `collect_bindings` は `Retain`/`Release` の腕で束縛を作らず、`RcRhs::App(..)` にはその callee に
    よらず `Binding::Producer` を置くので、<2>0b の 2 つの違いは束縛表を動かさない。よって `V` の束縛表は
    `F` の束縛表を `rename` で写したものである (`VarTable::of` はこれにパラメータ・capture の
    `Binding::Param` を足す)。`origin_inner` は変数を名前で引くだけなので、`origin` の答えも同じ写像で
    写る。A6 と A13 より複製の名前は原本のどの名前とも衝突しないので、この写しは単射である。`rename` の
    像に無い名前は `V` の本体で束縛を持たないので、`origin_inner` の `None` の腕が `here()` すなわち
    `Exactly((p, μ))` を返す。`V` のパラメータ名と capture 名 -- `vars.param_tys` の鍵 -- はどれも
    `rename` の像なので、像に無い名前はその鍵ではない。この CASE の 5 種の消費は `own` を読まずに本体の
    形だけで決まる (<2>1) ので、名前替えで保たれる。
  <2>2. `p = rename[p0]` であるとき、`origin_F(p0, μ)` の候補であるパラメータ leaf はすべて
        `owned_leaves` に入っている。ここで `owned_leaves = infer_ownership(prog, type_env)` であり、
        その鍵は原本の名前である (`CODE src/rc_ir/borrow.rs: borrow_ify`)。
    BY P8, <2>1, <2>1a
    <2>1a より `F` の本体で `(p0, μ)` は D9 の意味で消費される。P8 は、ある関数のある leaf の参照が
    その関数のある実行路で D9 の意味で消費されるならば、その leaf の `origin` の候補であるパラメータ
    leaf はすべて `owned_leaves` に入っていると述べる。
  <2>3. `origin(p, μ)` の各候補 `(r, q)` について `owns_object(r, q)` は真である。
    <3>1. `owns_object(r, q)` は、`r` が `V` の `vars.param_tys` の鍵でないとき -- すなわち `V` の
          パラメータでも capture でもないとき -- 真である。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object,
         CODE src/rc_ir/ownership.rs: VarTable::of
      `self.vars.param_tys.get(root)` が `None` の腕が `true` を返す。`VarTable::of` は
      `func.params` と `func.capture` の名前だけを `param_tys` の鍵にする。
    <3>2. `r` が `V` の `vars.param_tys` の鍵であるとき、`owns_object(r, q)` は
          `units_under(ty(r), q)` の各 unit `q'` について
          `(r, truncate_to_unit(ty(r), q'))` が `owned_units` に入るかを見る。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
    <3>3. `V` が借用版であるとき、`V` は capture を持たず、`V` のパラメータ名は複製の名前
          `rename[&p0.name]` である。`owned_units` が `V` の名前について持つのは、原本のパラメータ
          `p0` と `owned_leaves.owns(&p0.name, &leaf)` が真である `leaf` についての
          `(rename[&p0.name], truncate_to_unit(&p0.ty, &leaf, type_env))` がすべてである。
      BY CODE src/rc_ir/borrow.rs: borrow_ify, P9
      `borrow_ify` が借用版を作るのは `func.capture.is_none()` の関数についてだけである。その版に
      ついて `borrow_ify` は
      `for p in &func.params { for leaf in boxed_leaf_paths(&p.ty, type_env) {
      if owned_leaves.owns(&p.name, &leaf) { let unit = truncate_to_unit(&p.ty, &leaf, type_env);
      owned_units.insert((rename[&p.name].clone(), unit)); } } }` を行い、`owned_units` へ `V` の名前で
      書き込むのはこの繰り返しだけである。**鍵は複製後の名前で、`owned_leaves` は原本の名前で
      引かれる。**この改名が P8 と `V` を繋ぐ橋である。P9 より `ty(rename[&p0.name]) = ty(p0)` である。
    <3>3a. `origin_V(p, μ)` の各候補 `(r, q)` について `q ∈ boxed_leaf_paths(ty(r), type_env)` である。
      <4>1. `λ ∈ boxed_leaf_paths(ty(x))` であるすべての対 `(x, λ)` について、`origin(x, λ).acted_on()`
            の各元 `(w, ν)` は `ν ∈ boxed_leaf_paths(ty(w))` を満たす。
        `origin(x, λ)` の計算の、memo を使わない展開 (L8a (ii)) についての帰納法で示す。L8a (ii) より
        その展開は有限なので、この帰納法は整礎であり、L8a (i) より答えは memo が当たるかどうかで
        変わらない。L8 (A) の (i) と (B) はどちらも `ρ` を読まないので、`ρ` の上で値を得ていない
        変数 -- `ρ` が選ばなかったアームの結果変数と、その先 -- についても使える。
        <5>1. CASE `origin_inner` が `origin` を呼ばない。このとき `acted_on()` の元は `(x.name, λ)`
              だけであり、仮定より `λ ∈ boxed_leaf_paths(ty(x))` である。
          BY L8, D15
          L8 (B) より値は `Origin::Exactly((x.name, λ))` であり、D15 より `Origin::Exactly` の
          `acted_on()` はその 1 元だけからなる列である。
        <5>2. CASE `origin_inner` が `Binding::Join` 以外の腕で `origin(x', λ')` を呼ぶ。このとき
              腕の値は `origin(x', λ')` そのものであり、L8 (A) の (i) より
              `λ' ∈ boxed_leaf_paths(ty(x'))` である。
          BY L8, CODE src/rc_ir/ownership.rs: origin_inner
          `Binding::Move`、`Binding::Llvm` の単一 `Arg` の枝、unbox 容器の `Binding::Field`、
          `Binding::Payload` の 2 つの枝は、いずれも `origin` を 1 回呼んでその値を腕の値とする。
        <5>3. CASE `origin_inner` が `Binding::Join(arm_results)` の腕を取る。このとき `acted_on()` は
              `{(x.name, λ)} ∪ ⋃_{j'} origin(arm_results[j'], λ).acted_on()` に含まれ、各 `j'` に
              ついて `λ ∈ boxed_leaf_paths(ty(arm_results[j']))` である。
          BY L8, D15, CODE src/rc_ir/ownership.rs: origin_inner,
             CODE src/rc_ir/ownership.rs: Origin::of_candidates
          この腕は各 `arm_result` について `origin(vars, type_env, &arm_result.name, path).acted_on()`
          の各元を `candidates` に入れ、`Origin::of_candidates(candidates, &(x.name, λ))` を返す。
          `of_candidates` は候補が 1 元のときその元の `Origin::Exactly`、そうでないとき
          `Origin::Join { identity: (x.name, λ), candidates }` を返し、D15 より
          `acted_on() = {identity()} ∪ candidates()` である。leaf の側は L8 (A) の (i) が、この腕に
          ついてどのアームの結果変数についても成り立つ形で与える。
        <5>4. QED
          BY <5>1, <5>2, <5>3, L8
          L8 (A) と (B) より `origin_inner` の腕はこの 3 つの場合に分かれる。<5>1 では `acted_on()` の
          元は仮定を満たす対 1 つである。<5>2 と <5>3 では、`acted_on()` の元は `(x.name, λ)` か、
          行き先の型の boxed leaf を第 2 成分に持つ再帰の相手の `acted_on()` の元であり、後者には
          帰納法の仮定が適用できる。
      <4>2. QED
        BY <4>1, D15, D9
        L16 の仮定より `(p, μ)` は D9 の意味で消費される leaf であり、D9 の消費の表が挙げるのは
        boxed leaf なので `μ ∈ boxed_leaf_paths(ty(p))` である。D15 より候補は `acted_on()` の元で
        あるから、<4>1 が結論を与える。
    <3>4. QED
      BY <1>1a, <1>1b, <2>1a, <2>2, <3>1, <3>2, <3>3, <3>3a, P1, A10, P7e, P9
      `origin_V(p, μ)` の候補 `(r, q)` を取る。`r` が `V` の `vars.param_tys` の鍵でないときは
      <3>1 が結論を与える。以下 `r` がその鍵であるとし、`t = truncate_to_unit(ty(r), q, type_env)` と
      置く。`r` は `V` のパラメータか capture なので `ty(r)` はプログラムに現れる型であり、A10 を
      満たす。<3>3a より `q ∈ boxed_leaf_paths(ty(r))` なので、P1 より `t ∈ rc_units(ty(r))` である。
      P7e (a) より `owns_object(r, q) = owns_object(r, t)` であり、<3>2 より `owns_object(r, t)` は
      `units_under(ty(r), t)` の各 unit `q'` について `(r, truncate_to_unit(ty(r), q'))` が
      `owned_units` に入るかを見る。<1>1b より `units_under(ty(r), t) = [t]`、<1>1a より
      `truncate_to_unit(ty(r), t) = t` なので、検査は `(r, t) ∈ owned_units` の 1 件である。
      `V` が借用版でないときは、`owned_units` がその版の各パラメータ・capture `p'` と各
      `unit ∈ rc_units(ty(p'))` の対を含むので (`CODE src/rc_ir/borrow.rs: borrow_ify` の
      `owned_units.extend(param_capture_units(func, type_env))`、`CODE src/rc_ir/borrow.rs:
      param_capture_units`)、`(r, t)` はそこに在る。`V` が借用版であるときは、<3>3 より `V` は
      capture を持たないので `r` は `V` のパラメータであり、<2>1a より `r = rename[r0]`、`(r0, q)` は
      `origin_F(p0, μ)` の候補、`ty(r) = ty(r0)` である。<3>3a より `q ∈ boxed_leaf_paths(ty(r0))` で
      あり、<2>2 より `(r0, q)` は `owned_leaves` に入るので、<3>3 の繰り返しは `leaf = q` の回で
      `(rename[r0], truncate_to_unit(ty(r0), q)) = (r, t)` を `owned_units` に入れる。
  <2>3a. `μ` は `u` を前置に持つ。すなわち `μ` は `u` の下の leaf である。
    BY CODE src/rc_ir/ownership.rs: truncate_to_unit
    `truncate_to_unit(ty(p), μ)` は `μ` の添字を先頭から順に見て `out` に積み、`UnitStep::Capture` か
    `UnitStep::Unit` で打ち切る。積む添字は `μ` の添字を順に取ったものなので、返る `u` は `μ` の前置で
    ある。
  <2>4. QED
    BY <2>3, <2>3a, <1>3, P7a, <1>2
    この CASE の仮定より `V` の本体に `Retain(p, u)` 節点が在るので、<1>3 より `(p, u)` は
    `levelled_sites` の元である。`μ` は inhabited であり、<2>3a より `u` の下の leaf である。よって
    <2>3 は P7a の節 2 である。P7a より `owns_unit(p, u)` は真であり、<1>2 より節点は落ちない。これが
    (A) である。

<1>5. CASE 消費が `App(callee, args)` の引数の位置である。
  <2>0. `p` を第 `i` 引数とし、`u = truncate_to_unit(ty(p), μ, type_env)` と置く。`V` の本体のこの `App`
        の段の実行時の呼び出し先 (D23) を `g`、`g` の第 `i` パラメータを `p_i` と書く。`g` は
        `borrow_ify` の出力の `funcs` の関数である。また `call_rc` が引く `params` が `Some` であるとき、
        `params[i]` は `p_i` の名前と型である。
    BY D23, (R), A6, A13, P9, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc,
       CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/ownership.rs: resolve_callee_params
    D23 は `App` の呼び出し先を実行時に `callee` の値が指す関数と定め、「**D9 の `App` の行と D10 の
    生成の `App` の行が「呼び出し先」と言うのは、この実行時の関数である。**」と述べたうえで、「D9 の
    `App` の行が読む所有は D14 が `RcFunc::borrowed_units` から定めるものなので、**その呼び出し先は
    プログラムの `funcs` の関数である**」と続ける。`call_rc` は
    `self.callee_params.get(&FuncRef { name: callee.name })` を引き、`borrow_ify` は出力の各版 `func` に
    ついて `callee_params.insert(func.name, param_names_and_types(func))` を行うので、`Some` が返るのは
    `callee.name` が出力の `funcs` の鍵であるときであり、返るのはその関数のパラメータの名前と型の列で
    ある。そのとき A6 と A13 と P9 より `callee.name` は `V` の本体の束縛名ではないので
    `resolve_callee_params` の `closure_targets` の枝は当たらず、`resolve_callee_params` も同じ関数の
    `params` を返す。(R) より、その `params` は実行時の呼び出し先 `g` のパラメータの列である。
  <2>1. この位置で `(p, μ)` の消費が起きるならば、`(p_i.name, u)` は
        `all_owned_units(borrow_ify の出力, type_env)` に入る。
    <3>1. D9 の `App` の行より、引数 `p` の leaf `μ` が消費されるのは、`g` がその位置の unit を D14 の
          意味で所有するときであり、unit は**呼び出し先のパラメータの型**で取る。
      BY D9, D23, <2>0
    <3>2. その unit は `u` であり、`u ∈ rc_units(ty(p_i), type_env)` である。
      BY A12, P1, A10, <3>1
      A12 の第 6 項より `ty(p) = ty(p_i)` なので
      `truncate_to_unit(ty(p_i), μ) = truncate_to_unit(ty(p), μ) = u` である。`ty(p_i)` は
      `borrow_ify` の出力の関数のパラメータの型、すなわちプログラムに現れる型なので A10 を満たす。
      P1 より、A10 を満たす型の boxed leaf の `truncate_to_unit` は `rc_units` の元である。
    <3>3. `g` が `u` を所有することと `(p_i.name, u) ∈ all_owned_units(borrow_ify の出力, type_env)` は
          同値である。
      BY D14, <2>0, <3>2, CODE src/rc_ir/ownership.rs: all_owned_units
      D14 より、`g` が `u` を所有するとは `(p_i.name, u)` が `g` の `borrowed_units` に入らないことで
      ある。`all_owned_units` は、各関数の各パラメータ・capture `p'` と各 `unit ∈ rc_units(ty(p'))` に
      ついて、`(p'.name, unit)` が `borrowed_units` に入らないときにそれを集合に入れる。<2>0 より `g` は
      出力の `funcs` の関数であり、<3>2 より `u` はその `unit` の 1 つである。
    <3>5. QED
      BY <3>1, <3>2, <3>3
  <2>2. この位置で `(p, μ)` の消費が起きるならば、`call_rc` の `callee_owns` は真である。
    BY <2>0, <2>1, P13, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc,
       CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/ownership.rs: all_owned_units
    `call_rc` は `params` が `None` のとき `callee_owns` を真とし、`Some(params)` のとき
    `self.owned_units.contains(&(params[arg_idx].0.clone(), unit.clone()))` を見る。`None` の場合は
    無条件に真である。`Some` の場合は <2>0 より `params[i].0 = p_i.name` であり、`owned_units` は出力の
    各版のパラメータ・capture の所有 unit を持ち、P13 よりそれは出力の `borrowed_units` に入らない
    unit の全体、すなわち `all_owned_units(出力)` のパラメータ・capture の部分に一致するので、
    <2>1 の所属がその検査を真にする。
  <2>3. CASE `owns_unit(p, u)` が真である。`V` の本体に `Retain(p, u)` 節点が在れば <1>2 より落ちない
        ので (A) であり、無ければ (C) である。
    BY <1>2
  <2>4. CASE `owns_unit(p, u)` が偽である。このとき `call_rc` は `(p, u)` を `before` に入れる。
    BY <2>2, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
    `call_rc` は `callee_owns && !arg_owned` のとき `before.push((arg.clone(), unit))` を行う。L16 の
    仮定よりこの位置で `(p, μ)` の消費は起きるので、<2>2 より `callee_owns` は真であり、この CASE の
    仮定より `arg_owned = owns_unit(p, u)` は偽である。
  <2>5. QED
    BY <2>3, <2>4, P11
    <2>4 の場合、`call_rc` が `before` に入れた `(p, u)` について、P11 よりこの呼び出しの前に
    `Retain(p, u)` が置かれる。これが (B) である。<2>3 と <2>4 は `owns_unit(p, u)` の真偽で場合を
    尽くす。

<1>5a. CASE 消費が `App` の所有位置の引数以外であり、`V` の本体に `Retain(p, u)` 節点が無い。
  BY この CASE の仮定
  この CASE の 2 つの仮定は、言明の (C) の 2 つの条件 --「`V` の本体に `Retain(p, u)` 節点が無く、
  その消費は `App` の所有位置の引数でもない」-- そのものである。

<1>6. QED
  BY <1>1, <1>2, <1>4, <1>5, <1>5a
  D9 の消費の表は 6 行を持つ -- `App` の callee、`App` の所有位置の引数、`Closure` の capture、
  boxed 容器の `Destructure`、unbox 容器の `Destructure`、終端の `Ret`。`App` の所有位置の引数の行は
  <1>5 が扱い、(A) か (B) か (C) を与える。**残りの 5 行 -- `App` の callee を含む -- は、`Retain(p, u)`
  節点が在るときは <1>4 が (A) を、無いときは <1>5a が (C) を与える。**この 5 行はどれも
  `collect_consumes_go` が `own` を読まずに報告するからである (`CODE src/rc_ir/ownership.rs:
  collect_consumes_go`、`CODE src/rc_ir/ownership.rs: rhs_consumes` の `RcRhs::App` の腕は `callee` の
  leaf を無条件に `out` に入れる)。この 2 つの分け方で場合は尽きる。

**この補題が答えるもの。** 疑い -- 「`rewrite_rc` がアームの `Retain(p)` を落としながら `p` の消費が
残ることがあるか」-- は、当たらない。`Retain(p, u)` 節点が在る場合、それが落ちるのは `owns_unit(p, u)` が
偽のときだけであり、それが起きるのは `App` の所有位置の引数に限られる (L16 の (A) の CASE が他の 5 行を
`owns_unit` 真にするので)。そこでは `call_rc` が消費の直前に同じ `Retain(p, u)` を置くので (L16 の (B))、
`Retain` の直後に消費が来て `held_ρ(·, C)` はこの節点をまたいで変わらない。

**この補題は A19 (ii-b) を果たさない。** 果たすのは「`borrow_ify` が `insert_rc` の出した `Retain` を
消費から引き離さない」ことだけである。(C) の場合 -- 消費に対応する `Retain` 節点が本体に無い場合 -- は
`insert_rc` の側の義務であり、第 7.5.6 節が述べる。README も果たす者を 2 人挙げ、そのうち
「`borrow_ify` の側は示されている (`p13-disposals-and-pending.md` の `L16`)」と書き、`insert_rc` の側を
別に数えている。

#### 7.5.6 `insert_rc` の側 -- この文書が示せないもの

A19 (ii-b) が要求するのは、`insert_rc` の出力の各別名類が、走査が `pending` にその類の bump を数えて
いる時点では、その bump より 1 つ多い参照を持つことである。`insert_rc` はこれを使用回数の勘定で作る --
`Own` オペランドの最後でない使用の前に `Retain` を置き (`CODE src/rc_ir/rc_insert.rs:
RcInserter::insert_into_operation_let` の `retains_before`)、アーム本体の `Ret(x)` の `x` が `Match` の
後でも live であるときそのアームの中に `Retain(x)` を置き (`CODE src/rc_ir/rc_insert.rs:
RcInserter::insert_into_expr_inner` の `RcExpr::Ret(x)` の腕と
`CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live`)、容器と scrutinee についても同じことをする
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure`,
`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match`)。

**これは `insert_rc` についての命題であり、この証明の対象 (`borrow_ify` と `cancel`) の外にある。**
README はこの側の進み具合を A19 の中に書いている -- 「**`insert_rc` の側は部分的に示されている**
(`p60-insert-rc.md`)」であり、示されたのは「`insert_rc` が出す `Retain` は、間に `Retain` 以外の節点を
挟まず、同じ変数を名指す構文の直前に立つ」(`p60` の `L9`) と「この形は `split_rc_units` と `borrow_ify` を
通って `cancel` の入力まで残る」(`p60` の `L11`)、そして「`C1` と `C2` はどちらも `insert_rc` の出力では
ない」(`p60` の `L10`) である。残る義務は README が **(O1)** と **(O2)** として挙げている --
(O1) は別名類の粒度で D11 を `insert_rc` の出力について示すこと、(O2) は台帳の不等式 `U + X ≥ D` で
ある。**この文書はどちらも扱わない。**

#### 7.5.7 反例 `C1` (A19 (ii-b) が A1・A2・D12 から出ないことを示す)

A19 を仮定として置かねばならないこと -- すなわち A1・A2・D12 では P18a が出ないこと -- を、D12 を満たす
本体で示す。`Arr` を boxed な型、`I` を `is_fully_unboxed` が真の型、`Bl` を 2 つの変位がどちらも payload を
持たない unbox union とする。`alloc : () -> Arr`、`mkbl : () -> Bl`、`zero : () -> I` はそれぞれ結果の
leaf を `Fresh` と宣言する / boxed leaf を持たない `Llvm` 演算とする。

関数 `f` (パラメータ `b : Arr`、`borrowed_units` は空、返り値の型 `I`):
`Let(z, Llvm(zero, []), Release(b, [], s, Ret(z)))`。

関数 `main` (パラメータ無し、返り値の型 `I`):

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

変位 0 を選ぶ実行路で `m` の値は `p` の値である。`p` のオブジェクトを `O_p`、`q` のを `O_q` と書く。
`main` の各節点を実行した直後の `Obl` と参照カウントは次のとおりである。

| 実行した節点 | `Obl` | `H(O_p)` | `H(O_q)` |
|---|---|---|---|
| `Let(p, Llvm(alloc, []), ・)` | `{O_p}` | 1 | -- |
| `Let(q, Llvm(alloc, []), ・)` | `{O_p, O_q}` | 1 | 1 |
| `Let(c, Llvm(mkbl, []), ・)` | `{O_p, O_q}` | 1 | 1 |
| `Let(m, Match(c, ...), ・)` (移動) | `{O_p, O_q}` | 1 | 1 |
| `Retain(m, [], s, ・)` | `{O_p, O_p, O_q}` | 2 | 1 |
| `Let(u, App(f, [p]), ・)` | `{O_p, O_q}` | 1 | 1 |
| `Let(w, App(f, [q]), ・)` | `{O_p}` | 1 | 0 |
| `Eval(m, ・)` | `{O_p}` | 1 | 0 |
| `Release(m, [], s, ・)` | `{}` | 0 | 0 |

`App` の 2 行では、D9 の `App` の行が引数の leaf を消費して `Obl` から取り除き (`f` は `b` を所有する)、
`H` は呼び出しの段では動かず、`f` の本体の `Release(b, [])` が 1 下げる。`Ret(u)` の消費は
`ty(u) = I` に boxed leaf が無いので何も取り除かない。

(S-a) は各除去が `Obl` に入っているので、(S-b) は終端で `Obl` が
空で `ty(u) = I` に boxed leaf が無いので、(S-c) は読む構文 (`App(f, [p])` で `H(O_p) = 2`、
`App(f, [q])` で `H(O_q) = 1`、`Eval(m)` で `H(O_p) = 1`) と触れる構文 (`Retain(m, [])` と
`Release(m, [])` でどちらも `H(O_p) = 1`) がいずれも解放されていないオブジェクトを相手にするので、
成り立つ。`App` の `callee` は funptr の型を持ち、`is_funptr` の型は `is_fully_unboxed` が真なので
boxed leaf を持たない (D4 の第 1 規則、`CODE src/ast/types.rs: TypeNode::is_fully_unboxed`)。よって
`callee` には D9 が消費する leaf も D6 のスロットも無い。よって `C1` は D12 を満たし、A2 も満たす。

**A19 (ii-b) が破れる。** `m` と `p` は 1 つの別名類 `C` に属し (アーム本体の `Ret` の辺は D20 の
別名の辺であり、L8 (A) の `Binding::Join` の腕がそれを ρ-歩みとする)、`held_ρ(·, C)` は `p` の割り当てで
1、`Retain(m, [])` で 2、`App(f, [p])` の消費で 1 になる。`Retain(m, [])` の要素はそのとき `pending` に
在り `bumps_ρ(·, C) = 1` なので、(ii-b) が要求する `held ≥ 1 + bumps = 2` を `held_ρ(·, C) = 1` は
破る。P18a も同じ位置で破れる -- `N_ρ(·, O_p) = 1` に対して `H(O_p) = 1` で
ある。

**`cancel` は対を消す。** `origin(m, [])` は `Binding::Join([p, q])` の腕を取り、候補 `{(p, []), (q, [])}`
が 2 元なので `Join { identity: (m, []), candidates }` である。よって `ActRefs(m, []) = {(m, []): 1}` で
あり、`pending` の要素の `outstanding` は `(m, [])` だけを鍵に持つ (`CODE src/rc_ir/borrow.rs:
CancelAnalysis::walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕)。`App(f, [p])` の消費は
`consume_objects` を `acted_on(p, []) = {(p, [])}` で呼ぶ (L4)。`consume_objects` の述語は
`objects.iter().any(|object| retain.outstanding.names(object))` であり、`(m, [])` を鍵に持つ
`outstanding` は `(p, [])` を `names` としないので、要素は取り除かれない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)。続く `Release(m, [])` が `un_bump` で
`InBracket` を返し、`cancelled()` が `Retain(m, [])` とその `Release` を返す。消した後は `App(f, [p])` が
`O_p` を解放し、`Eval(m)` が解放済みの `O_p` を読む -- (S-c) 違反である。

**この形は `insert_rc` の出力ではない。** README の A19 が「`C1` と `C2` はどちらも `insert_rc` の出力では
ない (`p60` の `L10`)。`Retain` の直後の節点がその変数を名指さないので、節点 1 つを見るだけで弾ける」と
述べる。`C1` の `Retain(m, [])` の直後の節点は `Let(u, App(f, [p]), ...)` であり、`m` を名指さない。

#### 7.5.8 反例 `C2` (A19 (ii-b) の破れ方は `Join` に限らない)

`C1` は `Join` が 1 つのオブジェクトに 2 つ目の名前を与える形である。A19 (ii-b) の破れ方はそれだけでは
ない。次の `C2` には `Match` が無く、`origin` はどの leaf についても `Origin::Exactly` を返す。

関数 `id` (パラメータ `a : Arr`、`borrowed_units` は空、返り値の型 `Arr`): `Ret(a)`。
関数 `f` は 7.5.7 のもの。

関数 `main` (パラメータ無し、返り値の型 `I`):

```
Let(o, Llvm(alloc, []),
Let(y, App(id, [o]),
Retain(o, [], s,
Let(u, App(f, [y]),
Eval(o,
Release(o, [], s,
Ret(u)))))))
```

`o` のオブジェクトを `O` と書く。`Obl` と `H(O)` は `{O}, 1` → `{O}, 1` → `{O, O}, 2` → `{O}, 1` →
`{O}, 1` → `{}, 0` と動く。第 2 行は D9 の `App` の行 (`id` は `a` を所有するので `o` の leaf を消費し、
`H` は動かない) と D10 の生成の表 (`App` の結果の leaf が参照を生じ、`H` はここでは動かない) による。
`main` の 3 つの節はどれも成り立つので `C2` は D12 を満たす。

**A19 (ii-b) が破れる。** `o` と `y` は別の別名類である -- `y` は `RcRhs::App(..)` に束縛されるので
`Binding::Producer` であり、`origin(y, []) = Origin::Exactly((y, []))` で ρ-歩みを持たない
(`CODE src/rc_ir/ownership.rs: collect_bindings`, `CODE src/rc_ir/ownership.rs: origin_inner`)。
`o` の類を `C_o`、`y` の類を `C_y` と書くと、`held_ρ(·, C_o)` は割り当てで 1、`App(id, [o])` の消費で 0、
`Retain(o, [])` で 1 になる。`Retain(o, [])` の要素が `pending` に在る間 `bumps_ρ(·, C_o) = 1` なので、
(ii-b) が要求する `held ≥ 1 + bumps = 2` を `held_ρ(·, C_o) = 1` は破る。**この類は
`held = bumps = 1` の時点を持つので、README が A19 で「`held ≥ bumps` は真だが弱すぎて、P18a が要る形を
出さない」と述べるときに引く例でもある** -- `held ≥ bumps` はこの時点を許すが (ii-b) は許さない。

**`cancel` は対を消す。** `App(f, [y])` の消費が渡す `acted_on(y, []) = {(y, [])}` は
`t.outstanding = {(o, []): 1}` の鍵ではない。よって要素は残り、`Release(o, [])` が `un_bump` で `InBracket` を返し、対が消える。消した後は
`App(f, [y])` の中で `f` の `Release(b, [])` が `O` を解放し、`Eval(o)` が解放済みの `O` を読む。

**`C2` が示すこと。** ずれを作っているのは `Join` ではなく `App` である。`o` の参照は `id` の活性化を
通って `y` に戻り、`origin` は `y` を `Producer` として `Exactly((y, []))` と答えるので、同じ参照が
呼び出しの前と後で別の別名類に属する。`Join` の候補を `pending` の要素に持たせても、この形には
触れない -- `Others(o, [])` は空だからである。

**この形も `insert_rc` の出力ではない。** README の A19 が引く `p60` の `L9` -- 「`insert_rc` が出す
`Retain` は、間に `Retain` 以外の節点を挟まず、同じ変数を名指す構文の直前に立つ」-- と `L10` が
これを与える。`C2` の `Retain(o, [])` の直後の節点は `Let(u, App(f, [y]), ...)` であり、`o` を
名指さない。**「消費された変数はその後 live でない」という理由づけはこの本体では偽である** -- `o` は
`Eval(o)` で使われるので `App(id, [o])` の時点で live であり、`insert_rc` の liveness はこの形を
弾かない。弾くのは `L9` の「直前に立つ」の側である。

### 7.6 README を読み直した結果 (第 7 節の分)

**この文書がかつて差し戻した 7 点は、どれも README にすでに在る。** 直近の 4 点 (計数下の安定性、
A19 (ii-b) の形、`held` の帰属の委ね先、A19 (i) の `d` の形) は、この文書が差し戻した後に README が
答えたものである。

| かつての差し戻し | 主張 | README の現在の文 |
|---|---|---|
| 6 | bump の帰属を定義に置くか、「README にはまだ無い」 | **D27 (bump の帰属)** として第 5 節に在り、位置も「走査中の位置 -- **節点の訪問の入口**」と定めている。第 7.1 節の DEF bump の帰属はこれと同じものである |
| 10 | A19 の `held` に帰属の取り決めが無い | A19 が「**別名類と `held` の定義がこの文書でなく `p13-disposals-and-pending.md` に在るのは、どちらも走査の帳簿と噛み合う形でしか書けず、その帳簿を定めるのがその文書だからである。**」と、`held` の定義をこの文書へ委ねている。(ii-a) の側にも「**`held` は帰属である。** … その帰属は `p13-disposals-and-pending.md` の `DEF 類ごとの参照` が定める」と在る。この文書の `DEF A19 の held の読み` がその委任を受ける |
| 7 | A19 を README の第 4 節に置くか | **A19 (bump の下に余りが在る)** として第 4 節に在り、果たす者は「`insert_rc` (使用回数の勘定) と `borrow_ify` (`rewrite_rc` が落とさないこと、落とす場合に `call_rc` が補うこと)」と書かれている |
| 8 | `C1`/`C2` を第 8 節でなく A19 の根拠として引くか | A19 の中で「`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` と第 7.5.8 節の `C2` が、D12 を満たしながら P18a を破る本体である。この 2 つはコードの欠陥ではなく、この仮定が要ることの証拠である」と引かれている |
| 1 (計数下の安定性を D26 に書く) | D26 が、遷移が 1 つの活性化の中で起きるかどうかを書いていない | D26 の最後の段落が「**1 つの活性化の間、そこに現れるオブジェクトが計数下であるかどうかは変わらない。** `mark_global` の呼び出しはコード生成に 1 か所しかなく、グローバル初期化子の本体を評価した結果に対してだけ走る (`CODE src/rc_ir/codegen.rs: Generator::implement_rc_global`)。初期化子は引数を持たない (`CODE src/rc_ir/ast.rs: RcGlobalInit`) ので … 命題が『各時点の計数下オブジェクト』を量化するとき、その集合は活性化の間ずっと同じである」と述べている |
| 3 (A19 (i) が P18a の要る段を与えない) | (i) が借用終端の類を総和から除くので、`Σ_C held_ρ(n, C)` の上界を与えない | (i) が `d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` の総和と、「`S` に借用終端の類が在るならば 1」の和の形になった。第 7.5.4 節の `<1>5` がこれを引き、`<1>6` と `<1>7` が `bumps` の正である類が借用終端であるかどうかで場合を分ける |
| 2 (A19 の導かれた形は不等式から出ない) | (ii-b) を `held ≥ bumps` と書き、条件付きの形をそこから導いていた | (ii-b) 自身が条件付きの形になった -- 「`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である」。README は続けて「`held ≥ bumps` は真だが弱すぎて、P18a が要る形を出さない。`held = bumps` かつ `bumps ≥ 1` である時点を `held ≥ bumps` は許し、`p13-disposals-and-pending.md` の `C2` の類 `C_o` がまさにその時点だからである」と、台帳での 1 単位差 (`U + X - D ≥ -1` と `≥ 0`) とともに述べている |

差し戻し 8 に添えた「`C1` の形は実際の入力に 11 件ある」には出典が無かった。この文書はその数を使わない。

### 新しく差し戻す点 -- P29 を出力について読む段が無い

**第 4 節の (R) を与える者が README に無い。** (R) は「`borrow_ify` の出力の版 `V` の本体の `App` 節点に
ついて `resolve_callee_params(c, vars_V, prog)` が `Some(params)` を返すならば、`params` はその段の
実行時の呼び出し先 (D23) のパラメータの列である」であり、第 5 節の `<1>8` と第 7.5.5 節の `L16` が
これを引く。README の P29 の現在の文は次のとおりで、**入力**についての言明である。

> **P29** (静的に決めた呼び出し先は実行時の呼び出し先である)。`borrow_ify` の入力の
> `Let(x, App(callee, args), k)` について、`resolve_callee_params` が解決する関数が `Some` であるならば、
> **それはその段の実行時の呼び出し先 (D23) と同じ `RcFunc` である**。したがってその `params` も
> `borrowed_units` も、実行時の呼び出し先のものである。

出力については「`resolve_callee_params` が `None` を返す場合について何も言わないのは、そのとき
`rhs_consumes` が全位置を所有として扱う -- 安全側 -- からである。出力についての同じ性質は、P9 と P12 と
合わせて読む」と続くだけである。P9・P12・P24 が与えるのは第 4 節の (R) が挙げる 3 つ -- 節点の対応、
`route` が返す名前の在りか、`closure_targets` の像 -- であり、これは出力の `App` の callee が**どの名前で
あるか**までを決める。残るのは**その名前が実行時に指す関数が、そのプログラムの `funcs` のその名前の
関数であること**であり、これは P9 と P12 のどちらの言明にも無い。P29 の言明はそれを入力について言い、
その文は入力の `funcs` を読む。

**要る形は、P29 を 2 つのプログラムについて読める形にすることである。** README の第 7 節は P29 の証明に
ついて「局所補題 `L0b` を仮説つきで立て、P29 は入力に、`L0` は出力に当てる」と書いており、その仮説つきの
形を命題として出すか、P29 の言明を入力と出力の両方にかかる形にすれば、(R) は前提でなくなる。
