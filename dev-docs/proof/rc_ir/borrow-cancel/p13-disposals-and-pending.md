# P7c: 処分はすべて走査に届く

この文書は README の P7c を扱う。README の定義 D1-D19、仮定 A1-A14、および命題 P1 と P5 の**言明**の上に
立つ。それらの証明は `p10-leaves-and-units.md` と `p12-identity-and-consumes.md` にあり、この文書は
その言明だけを使う。P2、P3、P4、P6、P7、P7a、P7d は引用してよい位置にあるが、この文書は使わない。

**結論を先に書く。P7c は証明できた。** ただし言明が名指す仕組みは 2 つで、コードが持つ仕組みは 3 つである。
README の P7c は「`consume_objects` または `un_bump` を呼ぶ」と述べるが、終端の `Ret` の訪問はそのどちらも
呼ばず、その時点の `pending` の各要素を直接 `needed_retains` に入れる (`CODE src/rc_ir/borrow.rs:
CancelAnalysis::walk_inner` の `RcExpr::Ret` の腕)。第 3 節がこの 3 つ目の仕組みを含む形を `P7c′` として
書き、この文書はそれを証明する。第 6 節が README への差し戻しをまとめる。

第 5 節に補題 `L5` を置く。これは P7c の言明には無いが、`Release` の訪問の後に `pending` に何が残るかを
述べるもので、P18a が要る形である。

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
  「`v` の `π` の下の boxed leaf」の全体である。inhabited (D16) でないものを含む。
  `L(v) = L(v, [])` と書く。
- `ActRefs(v, π)` は `acted_references(vars, type_env, v, π)` (D15)。
- `Others(v, π)` は `CancelAnalysis::other_objects(v, π)` の返す列を集合とみなしたもの
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。

`VarPath` は対 `(FullName, FieldPath)` である (`CODE src/rc_ir/ast.rs: VarPath`)。この文書が
「オブジェクト」と書くのは `VarPath` の水準の名前であり、README の第 3.5 節の最後の段落がそう定める。

この文書は補題を `L1` から `L5`、証明する形の命題を `P7c′` と呼ぶ。`BY` の行ではそれらを名前で引用し、
その中のステップを指すときは `L1 の <1>3` と書く。

外部の結果を 1 つ使う。**stacker の `maybe_grow`**: `stacker::maybe_grow(red_zone, stack_size, callback)`
は `callback` をちょうど 1 回呼び、その値を返す (`CODE stacker-0.1.23/src/lib.rs: maybe_grow`)。

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

`B` の各節点 `n` について、**処分 leaf の集合** `Disp(n) ⊆ VarPath` を次で定める。D9 の消費の表の 6 行と、
D10 の `Release` の行が、参照を `Obl` から取り除く構文のすべてである。

| `n` | `Disp(n)` |
|---|---|
| `Let(x, App(callee, args), k)` | `{(callee, λ) : λ ∈ L(callee)}` と `{(a_i, λ) : λ ∈ L(a_i)、呼び出し先が第 `i` パラメータの unit `truncate_to_unit(ty(p_i), λ)` を所有する (D14)}` の和 |
| `Let(x, Closure(f, caps), k)` | `{(c, λ) : c ∈ caps, λ ∈ L(c)}` |
| `Let(x, Llvm(gen, args), k)` | `{(a_i, λ) : λ ∈ L(a_i)、`borrows_operand(i)` が偽、かつ `result_prov` のどの結果 leaf も `{Arg(i, λ)}` を単独では宣言しない}` |
| `Destructure(c, fs, s, k)` (`c` が boxed) | `{(c, λ) : λ ∈ L(c)}` |
| `Destructure(c, fs, s, k)` (`c` が unbox) | `{(c, λ) : λ ∈ L(c)、`λ` の先頭の添字が `fs` の名前付きフィールドでない}` |
| 終端の `Ret(x)` | `{(x, λ) : λ ∈ L(x)}` |
| `Release(v, π, s, k)` | `{(v, λ) : λ ∈ L(v, π)}` |
| 上のどれでもない `n` | 空集合 |

D9 の表は「消費される leaf」を inhabited (D16) の限定なしに挙げ、D10 の `Release` の行は inhabited な
leaf だけを取り除く。`Disp` は inhabited でない leaf も入れる。A5 より inhabited でない leaf は参照を
持たないので、`Disp` は実行時に処分される参照が属する leaf の上位集合である。上位集合を取るのは、以下の
主張を強くする向きである。

**D9 の `App` の行の読み。** D9 は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」と書き、
その unit をどちらの型で取るかを書いていない。この文書は、呼び出し先のパラメータ `p_i` の型で取った
`truncate_to_unit(ty(p_i), λ, type_env)` と読む。第 6 節がこれを差し戻す。

**D9 の終端の `Ret` の行の読み。** D9 は「関数本体の終端の `Ret(x)`」と書く。D12 はグローバル初期化子の
`init` を関数の本体と同じ資格で扱うので、`B` がグローバル初期化子の `init` であるときも、その終端の
`Ret` を同じ行で読む。

### DEF 触れうるオブジェクト

`B` の節点 `n` について、`Obj(n) = ⋃_{(w, λ) ∈ Disp(n)} acted_on(w, λ)` と定める。P7c の言明の
「その構文が触れうるオブジェクト (`acted_on`)」がこれである。

## 3. 証明する形

**P7c′**。`B` の各節点 `n` の各訪問について、次が成り立つ。

- **(a)** `n` が `B` の終端の `Ret` でないとき。その訪問が行う `consume_objects` と `un_bump` の呼び出しが
  名指すオブジェクトの和は、`Obj(n)` を含む。とくに `Obj(n)` が空でなければ、その訪問はこの 2 つの
  どちらかを少なくとも 1 回呼ぶ。
- **(b)** `n` が `B` の終端の `Ret` のとき。その訪問は、渡された `pending` の**すべての**要素を
  `needed_retains` に入れる。とくに `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素はすべて入る。

(a) は README の P7c の言明そのものである。(b) が、README の言明が名指していない 3 つ目の仕組みである。

## 4. 予備の補題

### L1 (`grow_stack` は閉包を 1 回呼ぶ)

**言明**。`CancelAnalysis::walk(node, pending, returns_from_func)` の 1 回の呼び出しは、
`CancelAnalysis::walk_inner(node, pending, returns_from_func)` をちょうど 1 回呼んでその値を返す。

**証明**

<1>1. `walk` の本体は `grow_stack(|| self.walk_inner(node, pending, returns_from_func))` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk
<1>2. `grow_stack(f)` の本体は `stacker::maybe_grow(64 * 1024, 1024 * 1024, f)` である。
  BY CODE src/misc.rs: grow_stack
<1>3. QED
  BY <1>1, <1>2, 外部の結果 stacker の `maybe_grow`

### L2 (走査は本体の各節点を訪れる)

**言明**。`B` の各節点 `n` について、`n` の訪問が少なくとも 1 回起こる。

**証明**

<1>1. `B` の節点の全体は、根 `B` から次の辺を辿って到達できる節点の全体である。`Let(x, rhs, k)` から `k` へ、
      `rhs` が `Match(v, arms)` のときはさらに各 `arm.body` へ。`Retain(v, π, s, k)` /
      `Release(v, π, s, k)` / `Eval(v, k)` / `Destructure(c, fs, s, k)` から `k` へ。`Ret(v)` からは
      どこへも行かない。
  BY D2, CODE src/rc_ir/ast.rs: for_each_node_inner
  D2 は本体を、継続の辺と `Match` のアーム本体の辺だけを持つ木と定める。`for_each_node_inner` が辿る辺の
  集合がこれと同じであることを、その `match` の 3 つの腕が示す。

<1>2. `walk_inner(n, P, r)` は、`n` の式の種別ごとに次の `walk` の呼び出しを行う。
  <2>1. `RcExpr::Retain(v, path, _, k)` の腕は `self.walk(k, pending, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `RcExpr::Release(v, path, _, k)` の腕は `self.walk(k, pending, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕は、`arms` の各 `arm` について
        `self.walk(&arm.body, pending.clone(), false)` を呼び、その後 `self.walk(k, merged, returns_from_func)`
        を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>4. `RcExpr::Let(x, rhs, k)` の腕 (`rhs` が `Match` でない場合) は
        `self.walk(k, pending, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>5. `RcExpr::Destructure(container, fields, _state, k)` の腕は
        `self.walk(k, pending, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>6. `RcExpr::Eval(_, k)` の腕は `self.walk(k, pending, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>7. `RcExpr::Ret(_)` の腕は `walk` を呼ばない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>8. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <2>7

<1>3. `n` の訪問が起これば、`n` から <1>1 の辺で 1 歩で行ける各節点の訪問が起こる。
  BY <1>2, L1

<1>4. `B` の訪問が起こる。
  BY CODE src/rc_ir/borrow.rs: cancel, L1
  `cancel_body` が `analysis.walk(body, PendingRetains::default(), true)` を呼ぶ。

<1>5. QED
  <1>1 の辺による到達可能性についての帰納法。基底は <1>4、帰納段は <1>3 である。D2 より `B` は有限の木
  なので、根からの到達可能性は辺の本数についての帰納で尽きる。
  BY <1>1, <1>3, <1>4, D2

### L3 (`returns_from_func` が真になる節点)

**言明**。`B` の節点 `n` の訪問 `walk_inner(n, P, r)` について、`r` が真であることと、`n` が `B` の根から
継続の辺だけで到達できること (アーム本体の辺を 1 度も使わないこと) は同値である。とくに、`B` の終端の
`Ret` の訪問では `r` は真であり、アーム本体の中の `Ret` の訪問では `r` は偽である。

**証明**

<1>1. 根 `B` の訪問では `r` は真である。根は継続の辺を 0 本使って到達できる。
  BY CODE src/rc_ir/borrow.rs: cancel, L1

<1>2. `walk_inner(n, P, r)` が継続 `k` について行う `walk` の呼び出しは、`returns_from_func` に `r` を
      そのまま渡す。
  BY L2 の <1>2

<1>3. `walk_inner(n, P, r)` がアーム本体について行う `walk` の呼び出しは、`returns_from_func` に `false`
      を渡す。
  BY L2 の <1>2 の <2>3

<1>4. 節点 `n` へ至る `walk` の呼び出しの列は、`B` の根から `n` への辺の列と 1 対 1 に対応する。
  BY L2 の <1>2, L1
  L2 の <1>2 が挙げる `walk` の呼び出しは、L2 の <1>1 の辺と 1 対 1 である。

<1>5. QED
  <1>4 の辺の列の長さについての帰納法。<1>1 が基底であり、<1>2 と <1>3 が帰納段である。継続の辺だけの列
  では `r` は真のまま運ばれ (<1>2)、アーム本体の辺を 1 度でも使うと以後は偽である (<1>3 と <1>2)。
  `B` の終端の `Ret` は、D3 より `B` の根から継続の辺だけで到達する節点であり、アーム本体の中の `Ret` は
  アーム本体の辺を少なくとも 1 度使う。
  BY <1>1, <1>2, <1>3, <1>4, D3

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
  <1>1 の `objects` の元の全体は `acted_on(var, path)` である (第 1 節の記法と <1>2)。DEF 呼び出しが
  名指すオブジェクトより、`consume_objects` の呼び出しが名指すオブジェクトは `objects` の元の全体である。
  BY <1>1, <1>2, DEF 呼び出しが名指すオブジェクト

### L5 (`Release` の訪問の後に `pending` に残るもの)

**言明**。`n = Release(v, π, s, k)` の訪問において、`un_bump` の呼び出しが `UnBump::NoBracket` または
`UnBump::OutsideBracket` を返すならば、`walk_inner` がその後 `self.walk(k, pending, ...)` に渡す
`pending` のどの要素も、`Obj(n)` のどのオブジェクトも名指さない。

この補題は P7c の言明には無い。P18a が要る形として置く。

**証明**

<1>1. 訪問は、`others = self.other_objects(v, path)` を求め、`self.consume_objects(&mut pending, &others)`
      を呼び、`un_bumped = self.acted_references(v, path)` を求め、`un_bump(&mut pending, &un_bumped)` を
      呼ぶ。この順である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕

<1>2. `others` の元の全体は `Others(v, π)` であり、`un_bumped` は `ActRefs(v, π)` である。
  <2>1. `other_objects(v, path)` は第 1 節の記法の `Others(v, π)` を返す。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects
  <2>2. `CancelAnalysis::acted_references(v, path)` は `acted_references(self.vars, self.type_env, v, path)`
        の値をそのまま返す (空でないことを表明した後で)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references
  <2>3. QED
    BY <2>1, <2>2, L4 の <1>2

<1>3. `consume_objects(&mut pending, &others)` の後、`pending` のどの要素も `others` のどのオブジェクトも
      名指さない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects
  `consume_objects` は `pending.retain(...)` を呼び、`objects` のいずれかを `outstanding.names` が真と
  する要素を落とす。残るのはどれも名指さない要素だけである。

<1>4. `consume_objects` は `pending` の要素を落とすだけで、残る要素の `outstanding` を変えない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects

<1>5. CASE `un_bump` が `UnBump::NoBracket` を返す。
  <2>1. `un_bump` は `pending` を変えずに返る。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `NoBracket` を返すのは `rposition` が `None` を返した枝であり、そこまでに `pending` への書き込みは無い。
  <2>2. `rposition` が `None` を返したので、`pending` のどの要素 `r` についても
        `r.outstanding.shares_an_object(un_bumped)` は偽である。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `Iterator::rposition` が `None` を返すのは、述語がどの要素についても偽のときである。
  <2>3. `r.outstanding.shares_an_object(un_bumped)` が偽であることは、`un_bumped.objects()` のどの
        オブジェクト `o` についても `r.outstanding.names(o)` が偽であることと同値である。
    BY CODE src/rc_ir/ownership.rs: References::shares_an_object, References::objects, References::names
    `shares_an_object` は `other.0.keys().any(|object| self.0.contains_key(object))` であり、`objects` は
    `self.0.keys()` の複製、`names` は `self.0.contains_key` である。
  <2>4. `walk_inner` が `walk(k, ...)` に渡す `pending` は、<1>3 の後の `pending` である。
    BY <1>1, <2>1
  <2>5. QED
    <2>2 と <2>3 より、その `pending` のどの要素も `ActRefs(v, π).objects()` のどのオブジェクトも
    名指さない (<1>2)。<1>3 より、`Others(v, π)` のどのオブジェクトも名指さない。P5 (c) より
    `Obj(n) ⊆ ActRefs(v, π).objects() ∪ Others(v, π)` である。
    BY <1>2, <1>3, <2>2, <2>3, <2>4, P5, DEF 触れうるオブジェクト
    P5 (c) の言明の `π` の下の各 boxed leaf は `L(v, π)` の元であり、`Obj(n)` の定義 (DEF 処分 leaf の
    `Release` の行と DEF 触れうるオブジェクト) はその `acted_on` の和である。

<1>6. CASE `un_bump` が `UnBump::OutsideBracket` を返す。
  <2>1. `un_bump` は `pending` を変えずに返る。
    BY CODE src/rc_ir/borrow.rs: un_bump
    `OutsideBracket` を返すのは `!innermost.outstanding.covers(un_bumped)` の枝であり、そこまでに
    `pending` への書き込みは無い (`&mut pending[index]` を取るだけである)。
  <2>2. `walk_inner` は続けて `let objects = un_bumped.objects();` と
        `self.consume_objects(&mut pending, &objects)` を行う。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕
  <2>3. <2>2 の呼び出しの後、`pending` のどの要素も `un_bumped.objects()` のどのオブジェクトも名指さない。
    BY <2>2, <1>3 の議論と同じ CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects
  <2>4. <2>2 の呼び出しの後も、`pending` のどの要素も `others` のどのオブジェクトも名指さない。
    BY <1>3, <1>4, <2>1, <2>2
    <1>3 の性質は、要素を落とすだけの操作 (<1>4) と `pending` を変えない操作 (<2>1) を通しても保たれる。
  <2>5. `walk_inner` が `walk(k, ...)` に渡す `pending` は、<2>2 の呼び出しの後の `pending` である。
    BY <1>1, <2>2
  <2>6. QED
    BY <1>2, <2>3, <2>4, <2>5, P5, DEF 触れうるオブジェクト
    <1>5 の <2>5 と同じ被覆の議論による。

<1>7. QED
  BY <1>5, <1>6
  場合分けは `un_bump` の返り値が `NoBracket` か `OutsideBracket` かの 2 つであり、言明がその 2 つを
  仮定している。

## 5. P7c′

**証明**

<1>1. `B` の各節点 `n` の訪問が少なくとも 1 回起こる。
  BY L2

<1>2. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    D9 の消費の表にも移動の表にも `Retain` の行は無く、D9 の最後の段落が `Retain` は D10 が直接扱うと
    述べる。D10 の `Retain` の行は参照を**加える**。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
    D2 より `Retain` と `Ret` は `RcExpr` の相異なる種である。
  <2>3. QED
    空集合は任意の和に含まれる。よって P7c′ (a) が成り立つ。
    BY <2>1, <2>2

<1>3. CASE `n` の式が `RcExpr::Eval(v, k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    D9 の最後の段落が「`Eval(v, k)` … は、参照を作らず、移さず、手放さない」と述べる。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>4. CASE `n` の式が `RcExpr::Let(x, RcRhs::Var(y), k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    D9 の移動の表の第 1 行が `Let(x, Var(y), k)` を移動とし、消費の表に `Var` の行は無い。移動は義務集合を
    変えない。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>5. CASE `n` の式が `RcExpr::Let(x, RcRhs::Match(v, arms), k)` である。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D9
    D9 の最後の段落が「`Let(x, Match(v, arms), k)` の `Match` 節点自身は、参照を作らず、移さず、
    手放さない」と述べる。アームの payload 束縛とアーム本体の `Ret` は、この節点ではなくアームの側の
    構文であり、それぞれ D10 の生成の表と D9 の移動の表が扱う。
  <2>2. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>3. QED
    BY <2>1, <2>2

<1>6. CASE `n` の式が `RcExpr::Ret(v)` であり、`n` が `B` の終端の `Ret` でない。
  <2>1. `Disp(n)` は空であり、よって `Obj(n)` も空である。
    BY DEF 処分 leaf, DEF 触れうるオブジェクト, D3, D9
    DEF 処分 leaf の `Ret` の行は終端の `Ret` だけを挙げる。D3 より、終端でない `Ret` はアーム本体の
    `Ret` である (`B` の根から継続の辺だけで到達する `Ret` は終端の `Ret` である)。D9 の移動の表の第 2 行が
    アーム本体の `Ret(x)` を移動とする。
  <2>2. QED
    BY <2>1

<1>7. CASE `n` の式が `RcExpr::Ret(v)` であり、`n` が `B` の終端の `Ret` である。
  <2>1. `n` の訪問 `walk_inner(n, P, r)` では `r` が真である。
    BY L3
  <2>2. `RcExpr::Ret(_)` の腕は、`returns_from_func` が真のとき `pending` の各要素 `retain` について
        `self.needed_retains.insert(retain.node)` を行う。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>3. QED
    <2>1 と <2>2 より、渡された `pending` のすべての要素が `needed_retains` に入る。部分集合として、
    `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素も入る。これが P7c′ (b) である。
    BY <2>1, <2>2

<1>8. CASE `n` の式が `RcExpr::Let(x, RcRhs::App(callee, args), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
    `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕が先に置かれているので、`rhs` が `Match` でない
    `Let` はこの腕に来る。
  <2>2. `consume_rhs(pending, rhs, result_ty)` は、`rhs_consumes(rhs, result_ty, self.vars, self.prog,
        self.type_env, &owns, &mut consumed)` を呼び、`consumed` の各元 `(var, leaf)` について
        `self.consume(pending, &var, &leaf)` を呼ぶ。ここで `owns` は
        `|p, leaf| self.owned_units.contains(&(p.name.clone(), truncate_to_unit(&p.ty, leaf, self.type_env)))`
        である。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>3. `self.owned_units` は `all_owned_units(prog, type_env)` である。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>4. `rhs_consumes` の `RcRhs::App(callee, args)` の腕は、`callee` の各 boxed leaf `λ` について
        `(callee.name, λ)` を `out` に入れ、各 `i` と `a_i` の各 boxed leaf `λ` について、
        `is_owning_position` が真ならば `(a_i.name, λ)` を `out` に入れる。`is_owning_position` は、
        `resolve_callee_params(callee, vars, prog)` が `Some(params)` を返すときは `owns(&params[i], &λ)`、
        `None` を返すときは真である。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes, push_boxed_leaves
    `push_boxed_leaves(var, ty, type_env, out)` は `boxed_leaf_paths(ty, type_env)` の各元 `p` について
    `(var.clone(), p)` を `out` に入れる。A14 より `args.len() <= params.len()` なので `params[i]` は
    範囲内である。
  <2>5. `Disp(n)` は <2>4 が `out` に入れる集合に含まれる。
    <3>1. `Disp(n)` の第 1 の部分 `{(callee, λ) : λ ∈ L(callee)}` は <2>4 の第 1 の部分に等しい。
      BY <2>4, DEF 処分 leaf
      第 1 節の記法より `L(callee) = boxed_leaf_paths(ty(callee), type_env)` である。
    <3>2. `resolve_callee_params` が `None` を返すとき、`Disp(n)` の第 2 の部分は <2>4 の第 2 の部分に
          含まれる。
      BY <2>4
      このとき `is_owning_position` はすべての `(i, λ)` について真なので、<2>4 の第 2 の部分は
      `{(a_i, λ) : λ ∈ L(a_i)}` であり、`Disp(n)` の第 2 の部分はその部分集合である。
    <3>3. `resolve_callee_params` が `Some(params)` を返すとき、`params` は呼び出し先の関数
          `prog.funcs[fref]` の `params` である。
      BY CODE src/rc_ir/ownership.rs: resolve_callee_params
    <3>4. `Disp(n)` の第 2 の部分の元 `(a_i, λ)` について、`owns(&params[i], &λ)` は真である。
      <4>1. `Disp(n)` の第 2 の部分の元であるとは、呼び出し先が第 `i` パラメータ `p_i = params[i]` の
            unit `u = truncate_to_unit(ty(p_i), λ, type_env)` を所有することである。
        BY DEF 処分 leaf, <3>3
      <4>2. D14 より、呼び出し先が `u` を所有するとは、`u` が `p_i` の型の RC unit であって
            `(p_i.name, u)` が呼び出し先の `borrowed_units` に入らないことである。
        BY D14
      <4>3. `λ` は `ty(p_i)` の boxed leaf であり、P1 より `u = truncate_to_unit(ty(p_i), λ)` は
            `rc_units(ty(p_i))` の元である。
        BY P1, <4>1
        DEF 処分 leaf の `App` の行を上のように読むことが、`λ` を `ty(p_i)` の boxed leaf として扱うことを
        含んでいる (第 2 節の「D9 の `App` の行の読み」)。
      <4>4. `all_owned_units(prog, type_env)` は、`prog` の各関数の各パラメータ・capture `p` の各
            `u ∈ rc_units(ty(p))` について、`(p.name, u)` がその関数の `borrowed_units` に入らないならば
            それを集合に入れる。
        BY CODE src/rc_ir/ownership.rs: all_owned_units
      <4>5. QED
        呼び出し先は `prog.funcs` の関数であり (<3>3)、`p_i` はそのパラメータである。<4>2 と <4>3 より
        `u ∈ rc_units(ty(p_i))` かつ `(p_i.name, u) ∉ borrowed_units` なので、<4>4 より
        `(p_i.name, u) ∈ all_owned_units(prog, type_env)` である。<2>2 と <2>3 より
        `owns(&params[i], &λ)` はこの所属を検査する。
        BY <2>2, <2>3, <3>3, <4>1, <4>2, <4>3, <4>4
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4, <2>4
      `resolve_callee_params` の返り値が `None` か `Some` かで場合を尽くす。`None` は <3>2、`Some` は
      <3>4 と <2>4 の `is_owning_position` の定義による。
  <2>6. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>4 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, <2>4, L4
    <2>2 より `consumed` の各元について `consume` が呼ばれ、L4 よりその各呼び出しは
    `acted_on(w, λ)` を名指す `consume_objects` を 1 回行う。
  <2>7. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>8. QED
    <2>5 より `Disp(n)` は <2>4 の集合に含まれるので、`Obj(n) = ⋃_{(w, λ) ∈ Disp(n)} acted_on(w, λ)` は
    <2>6 の和に含まれる。これが P7c′ (a) である。
    BY <2>5, <2>6, <2>7, DEF 触れうるオブジェクト

<1>9. CASE `n` の式が `RcExpr::Let(x, RcRhs::Closure(f, caps), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `rhs_consumes` の `RcRhs::Closure(_, caps)` の腕は、`caps` の各 `c` の各 boxed leaf `λ` について
        `(c.name, λ)` を `out` に入れる。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes, push_boxed_leaves
  <2>3. `Disp(n)` は <2>2 が `out` に入れる集合に等しい。
    BY <2>2, DEF 処分 leaf
    DEF 処分 leaf の `Closure` の行は `{(c, λ) : c ∈ caps, λ ∈ L(c)}` であり、
    `L(c) = boxed_leaf_paths(ty(c), type_env)` である。
  <2>4. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>2 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, L4, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>5. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>6. QED
    BY <2>3, <2>4, <2>5, DEF 触れうるオブジェクト

<1>10. CASE `n` の式が `RcExpr::Let(x, RcRhs::Llvm(gen, args), k)` である。
  <2>1. `n` の訪問は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `rhs_consumes` の `RcRhs::Llvm(llvm_gen, args)` の腕は、
        `passthrough = passthrough_arg_leaves(&**llvm_gen, result_ty, args, type_env)` を求め、各 `i` に
        ついて `llvm_gen.borrows_operand(i, &arg_tys, type_env)` が真ならその `i` を飛ばし、偽なら
        `a_i` の各 boxed leaf `λ` について `passthrough` が `(i, λ)` を含まないときに `(a_i.name, λ)` を
        `out` に入れる。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes
  <2>3. `passthrough_arg_leaves` が返す集合は、`llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の
        結果 leaf のうち、その `LeafOrigins` がちょうど 1 元でその元が `LeafOrigin::Arg(j, σ)` である
        ものについての `(j, σ)` の全体である。
    BY CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection,
       CODE src/rc_ir/provenance.rs: Provenance::leaves
    `leaves()` は各 boxed leaf の `LeafOrigins` を渡し、`as_arg_projection` は 1 元でない集合と
    `Fresh`/`Unknown` に `None` を返す。
  <2>4. `Disp(n)` は <2>2 が `out` に入れる集合に等しい。
    BY <2>2, <2>3, DEF 処分 leaf
    DEF 処分 leaf の `Llvm` の行の条件「`borrows_operand(i)` が偽、かつ `result_prov` のどの結果 leaf も
    `{Arg(i, λ)}` を単独では宣言しない」は、<2>2 の条件「`borrows_operand(i)` が偽、かつ `passthrough` が
    `(i, λ)` を含まない」に、<2>3 によって一致する。
  <2>5. `consume_rhs` が `rhs_consumes` に渡す `result_ty` は `x.ty` である。
    BY <2>1, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>6. 訪問が行う `consume_objects` の呼び出しが名指すオブジェクトの和は、<2>2 が `out` に入れる各
        `(w, λ)` についての `acted_on(w, λ)` の和を含む。
    BY <2>1, <2>2, L4, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
  <2>7. `n` は `B` の終端の `Ret` ではない。
    BY D2
  <2>8. QED
    BY <2>4, <2>6, <2>7, DEF 触れうるオブジェクト

<1>11. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。
  <2>1. `n` の訪問は、`destructure_consumes(container, fields, self.type_env)` の各元 `leaf` について
        `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>2. `destructure_consumes(container, fields, type_env)` は、`container.ty.is_box(type_env)` が真の
        とき `boxed_leaf_paths(container.ty, type_env)` をそのまま返し、偽のときはそのうち先頭の添字が
        `fields` の名前付きフィールドの添字でないものだけを返す。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes
  <2>3. `{(container.name, leaf) : leaf ∈ destructure_consumes(...)}` は `Disp(n)` に等しい。
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
    BY L5 の <1>1, L5 の <1>2
  <2>2. この 2 つの呼び出しが名指すオブジェクトの和は
        `Others(v, path) ∪ ActRefs(v, path).objects()` を含む。
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
    BY <2>2, <2>4, <2>5

<1>13. QED
  <1>2 から <1>12 の場合分けが尽きている。
  <2>1. `RcExpr` はちょうど 6 種を持つ。`Let`、`Retain`、`Release`、`Destructure`、`Eval`、`Ret`。
    BY CODE src/rc_ir/ast.rs: RcExpr
  <2>2. `RcRhs` はちょうど 5 種を持つ。`Var`、`App`、`Closure`、`Llvm`、`Match`。
    BY CODE src/rc_ir/ast.rs: RcRhs
  <2>3. `Retain` は <1>2、`Eval` は <1>3、`Destructure` は <1>11、`Release` は <1>12 が扱う。
    BY <1>2, <1>3, <1>11, <1>12
  <2>4. `Let` は右辺の 5 種で分かれ、`Var` は <1>4、`Match` は <1>5、`App` は <1>8、`Closure` は <1>9、
        `Llvm` は <1>10 が扱う。
    BY <2>2, <1>4, <1>5, <1>8, <1>9, <1>10
  <2>5. `Ret` は「`n` が `B` の終端の `Ret` である」の真偽で分かれ、偽は <1>6、真は <1>7 が扱う。
        この 2 つで尽きるのは排中律による。
    BY <1>6, <1>7
  <2>6. QED
    BY <1>1, <2>1, <2>3, <2>4, <2>5
    <1>1 より各節点の訪問が起こり、<2>1 と <2>3 から <2>5 より、どの訪問も <1>2 から <1>12 の
    ちょうど 1 つの場合に入る。各場合が P7c′ の (a) か (b) のうちその節点に当たる方を示している。

## 6. README へ差し戻す点

### 差し戻し 1 (P7c の言明が名指す仕組みは 2 つ、コードの仕組みは 3 つ)

P7c は「`consume_objects` または `un_bump` を呼ぶ」と述べる。終端の `Ret` の訪問はそのどちらも呼ばず、
`pending` の各要素を `self.needed_retains.insert(retain.node)` で直接印を付ける
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Ret` の腕)。

言明を次の形にすることを提案する。

> **P7c** (処分はすべて走査に届く)。実行時に参照を処分するか、処分の義務を活性化の外へ渡す構文 --
> D9 の消費、`Release`、終端の `Ret` -- はすべて、`cancel` の走査で次のどちらかを行う。(a) 終端の `Ret`
> 以外では、`consume_objects` または `un_bump` を、その構文が触れうるオブジェクト (`acted_on`) をすべて
> 含む引数で呼ぶ。(b) 終端の `Ret` では、その時点の `pending` のすべての要素を `needed_retains` に入れる。

(b) の側は、印を付ける範囲としては `consume_objects` を `Obj(n)` で呼んだ場合より広い。狭いのは、
要素を `pending` から取り除かないことである。取り除かないことが後段に響かないのは、終端の `Ret` の訪問が
返す `pending` が走査全体の返り値であり、`cancel_body` がそれを捨てるからである
(`CODE src/rc_ir/borrow.rs: cancel` -- `analysis.walk(body, PendingRetains::default(), true);` の値は
束縛されない)。

### 差し戻し 2 (`acted_on` を定めるのは D15 である)

P7c の言明は「その構文が触れうるオブジェクト (D13 の `acted_on`)」と書く。`Origin::acted_on()` を定めるのは
D15 の最後の段落であり、D13 は `origin` と `Origin` を定める。引用先を D15 にすることを提案する。

### 差し戻し 3 (D9 の `App` の行が、unit をどちらの型で取るかを書いていない)

D9 の消費の表の `App` の行は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」と書く。引数の型と
パラメータの型は別の型なので、「その位置の unit」は 2 通りに読める。コードは呼び出し先のパラメータの型で
取る (`CODE src/rc_ir/ownership.rs: rhs_consumes` が `owns(&params[i], &leaf)` を呼び、
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs` の `owns` が
`truncate_to_unit(&p.ty, leaf, self.type_env)` を取る)。この文書はコードと同じ読みを採った (第 2 節)。
D9 にどちらの型かを書くことを提案する。

あわせて、**呼び出しの引数の型が呼び出し先のパラメータの型に一致するという仮定が A12 に無い**。A12 が挙げる
のは move-bind の両辺、アームの結果と `Match` の束縛変数、payload と変位、`Destructure` のフィールド変数と
フィールド、`Match` の scrutinee が union、`Destructure` の容器が構造体、同じ名前の `RcVar` の型の一致の
7 つで、`App` の引数とパラメータの対応はその中に無い。`truncate_to_unit` は型に合わない path で panic する
(`CODE src/rc_ir/ownership.rs: truncate_to_unit` の `UnitStep::NoUnit` の腕と
`CODE src/rc_ir/ownership.rs: held_field_type`) ので、この対応は `rhs_consumes` が停止するためにも要る。

### 差し戻し 4 (L5 を README の命題として置くか)

第 4 節の L5 は「`un_bump` が `NoBracket` か `OutsideBracket` を返したとき、`Release` の訪問の後の
`pending` のどの要素も `Obj(n)` のどのオブジェクトも名指さない」を述べる。P7c の言明はここまで言わないが、
P18a はこれを使う -- P7c だけでは「呼んだ」までしか出ず、「pending から消えた」が出ないからである。
L5 をこの文書の補題のまま置くか、README の命題に上げるかは、P18a を書く側の判断による。

### 差し戻し 5 (`stacker::maybe_grow` を外部の結果として README に置くか)

`grow_stack` を経由する関数 (`walk`、`rewrite`、`origin`、`drop_nodes`) の性質は、
`stacker::maybe_grow` が閉包をちょうど 1 回呼ぶことに立つ。この文書は第 1 節でそれを外部の結果として
述べた。`p20-borrow-ify.md`、`p30-cancel-walk.md`、`p40-cancel-soundness.md` も同じ事実を各自で述べて
いるので、README の第 3 節に 1 度だけ置くほうがよい。

## 7. P18a

未着手。README 第 5 節の言明を、この文書の第 4 節の L5 と、第 3 節の P7c′ の上に置く。
README 第 7 節の表がその状態を持つ。
