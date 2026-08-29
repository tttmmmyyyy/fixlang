# P7c: 処分はすべて走査に届く

この文書は README の P7c、P7f、P18b、P18a を扱う。README の定義 D1-D27 と仮定 A1-A20、および命題
P1、P2、P5、P6、P7、P7a、P7e、P8、P9、P10、P11、P12、P13、P14a、P17、P18 の**言明**の上に立つ。それらの証明は
`p10-leaves-and-units.md`、`p12-identity-and-consumes.md`、`p20-borrow-ify.md`、`p30-cancel-walk.md` に
あり、この文書はその言明だけを使う。`p30-cancel-walk.md` の補題 `L10` も引く (第 7 節)。

**結論を先に書く。P7c と P7f と P18b は証明できた。P18a は A19 の上で証明できた。**

第 3 節が P7c を `P7c′` として書き直す。README の P7c の (a) と (b) はこの文書の `P7c′` の (a) と (b) に
一致するので、書き直しは表記だけである。第 5 節が `P7c′` を証明する。第 4 節の `L6` が P7f を与え、
その直後の段がその一致を述べる。第 7 節が P18b (第 7.4 節) と P18a (第 7.5 節) を扱う。
第 6 節と第 7.6 節が README への差し戻しをまとめる。

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
leaf `λ` について、`(v, λ)` は `VarPath` の対 `(v.name, λ)` を表す。この文書が「オブジェクト」と書くのは
`VarPath` の水準の名前であり、README の第 3.5 節の最後の段落がそう定める。

この文書は補題を `L1` から `L6`、証明する形の命題を `P7c′` と呼ぶ。`BY` の行ではそれらを名前で引用する。
補題の証明の内部のステップは引用しない。

外部の結果を 1 つ使う。**stacker の `maybe_grow`**: `stacker::maybe_grow(red_zone, stack_size, callback)`
は `callback` をちょうど 1 回呼び、その値を返す (`CODE stacker-0.1.23/src/lib.rs: maybe_grow`)。
`remaining_stack()` の値で分かれる 2 つの枝がどちらも `callback` を 1 回だけ評価し、`callback` の型が
`FnOnce` なので 2 回は評価できない。

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

**P7c′**。`B` の各節点 `n` の各訪問について、次が成り立つ。

- **(a)** `n` が `B` の終端の `Ret` でないとき。その訪問が行う `consume_objects` と `un_bump` の呼び出しが
  名指すオブジェクトの和は、`Obj(n)` を含む。とくに `Obj(n)` が空でなければ、その訪問はこの 2 つの
  どちらかを少なくとも 1 回呼ぶ。
- **(b)** `n` が `B` の終端の `Ret` のとき。その訪問は、渡された `pending` の**すべての**要素を
  `needed_retains` に入れる。とくに `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素はすべて入る。

(a) は README の P7c の言明そのものである。(b) が、README の言明が名指していない 3 つ目の仕組みである。

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

<1>5. QED
  <1>1 と <1>3 より、`n` の訪問が起これば `n` から出る各辺の先の節点の訪問がちょうど 1 回ずつ起こり、
  それ以外の訪問は起こらない。よって根からの辺の列と訪問が 1 対 1 に対応する。D2 より `B` は木なので、
  各節点への辺の列はちょうど 1 本であり、各節点の訪問はちょうど 1 回である。D2 より `B` は有限なので、
  この対応は辺の列の長さについての帰納で尽きる。基底は <1>4 である。
  BY <1>1, <1>3, <1>4, D2

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
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects
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
  BY L6, DEF 触れうるオブジェクト, DEF 処分 leaf, P7c′
  L6 の言明は「`n = Release(v, π, s, k)` の訪問において、`un_bump` の呼び出しが `UnBump::NoBracket` または
  `UnBump::OutsideBracket` を返すならば、訪問がその後 `self.walk(k, pending, ·)` に渡す `pending` の
  どの要素も、`Obj(n)` のどのオブジェクトも名指さない」である。「訪問の後の `pending`」とは `walk(k, ·, ·)`
  に渡すものである (L5 の 4)。「その `Release` が触れうるオブジェクト」とは、DEF 触れうるオブジェクトと
  DEF 処分 leaf の `Release` の行より `Obj(n) = ⋃_{λ ∈ L(v, π)} acted_on(v, λ)` である。これは P7c′ の
  言明が「その構文が触れうるオブジェクト」と呼ぶものと同じ量である。「要素が名指す」とは
  `outstanding.names` が真であることである (L6 の <1>1)。よって 2 つの言明は同じことを述べている。

## 5. P7c′ の証明

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
    空集合はどんな和にも含まれる。よって P7c′ (a) が成り立つ。
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
    `Obj(n)` のオブジェクトを名指す `outstanding` を持つ要素も入る。これが P7c′ (b) である。
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
      <4>1. `params` は呼び出し先の関数 `prog.funcs[fref]` の `params` である。
        BY CODE src/rc_ir/ownership.rs: resolve_callee_params
      <4>2. `Disp(n)` の引数の側の元 `(a_i, λ)` とは、`λ ∈ L(a_i)` であって、呼び出し先が
            `p_i = params[i]` の unit `u = truncate_to_unit(ty(p_i), λ, type_env)` を所有するもので
            ある。また A12 より `ty(a_i) = ty(p_i)` であり、`λ` は `ty(p_i)` の boxed leaf である。
        BY DEF 処分 leaf, A12, <4>1
        A12 は「`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型」の一致を挙げる。
      <4>3. D14 より、呼び出し先が `u` を所有するとは、`u` が `rc_units(ty(p_i))` の元であって
            `(p_i.name, u)` が呼び出し先の `borrowed_units` に入らないことである。
        BY D14
      <4>4. `u = truncate_to_unit(ty(p_i), λ, type_env)` は `rc_units(ty(p_i))` の元である。
        BY P1, <4>2
        P1 は、任意の型 `τ` の各 boxed leaf の `truncate_to_unit(τ, ・)` が `rc_units(τ)` の元であると
        述べる。<4>2 より `λ` は `ty(p_i)` の boxed leaf である。
      <4>5. `all_owned_units(prog, type_env)` は、`prog` の各関数の各パラメータ・capture `p` と各
            `u' ∈ rc_units(ty(p))` について、`(p.name, u')` がその関数の `borrowed_units` に入らない
            ならばそれを集合に入れる。
        BY CODE src/rc_ir/ownership.rs: all_owned_units
      <4>6. `Disp(n)` の引数の側の元 `(a_i, λ)` について `owns(&params[i], &λ)` は真である。
        呼び出し先は `prog.funcs` の関数であり (<4>1)、`p_i` はそのパラメータである。<4>3 と <4>4 より
        `u ∈ rc_units(ty(p_i))` かつ `(p_i.name, u) ∉ borrowed_units` なので、<4>5 より
        `(p_i.name, u) ∈ all_owned_units(prog, type_env)` である。<2>2 と <2>3 より `owns(&params[i], &λ)`
        はまさにこの所属を検査する。
        BY <2>2, <2>3, <4>1, <4>2, <4>3, <4>4, <4>5
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
    <2>6 の和に含まれる。これが P7c′ (a) である。
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
    含み、<2>4 よりその集合は `Obj(n)` を含む。これが P7c′ (a) である。
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
    1 つの場合に入る。`n` が `B` の終端の `Ret` である場合 (<1>7) は P7c′ (b) を、他の場合は
    P7c′ (a) を示している。
    BY <1>1, <2>1, <2>2, <2>3, <2>4, <2>5

## 6. README を読み直した結果

**この文書がかつて差し戻した 5 点は、どれも README にすでに在る。** 対象コミット `6af3eb3b` の
README を読み直して確かめた。

| かつての差し戻し | 主張 | README の現在の文 |
|---|---|---|
| 1 | P7c の言明が名指す仕組みは 2 つで足りない | P7c は「**(a)** 終端の `Ret` 以外では、`consume_objects` または `un_bump` を … 呼ぶ。**(b)** 終端の `Ret` では、その時点の `pending` のすべての要素を `needed_retains` に入れる」と 2 節に分けている |
| 2 | P7c が `acted_on` の出典を D13 と書いている | P7c (a) は「その構文が触れうるオブジェクト (**D15 の `acted_on`**)」と書いている |
| 3 | D9 の `App` の行が unit を取る型を書いていない / A12 に引数とパラメータの型の一致が無い | D9 の `App` の行は「unit は**呼び出し先のパラメータの型**で取る」と書き、A12 は「**`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型**」を挙げている |
| 4 | L6 を README の命題に上げるか | **P7f** として在る。この文書の第 4 節が L6 を証明し、その直後の段が L6 と P7f の一致を述べる |
| 5 | `stacker::maybe_grow` を README に置くか | **A15** として在る -- 「`grow_stack(f)` は `f` をちょうど 1 回呼び、その返り値を返す」 |

第 3 の項について補う。この文書の第 2 節の `Disp` の表の `App` の行は、D9 のその行と A12 のその項を
そのまま読んだものであり、第 5 節の `<1>8` はその 2 つを引く。第 1 節が外部の結果として述べた
`stacker::maybe_grow` は A15 と同じ事実だが、A15 は `grow_stack` の水準で述べるので、`L1` は A15 を
直接引ける形にしてある。

## 7. P18b と P18a

**結論を先に書く。P18b は A1・A2・D12 から証明できた。P18a は前提 `A19` を 1 つ足すと証明できる。**

P18b は第 7.4 節が証明する。証明の要は補題 `L9` -- `origin` の `identity` が等しい 2 つの leaf は、
実行時に同時に inhabited (D16) であるか同時にそうでないか、のどちらかである -- であり、これが
「`outstanding` は静的な数え上げを引き、実行時は inhabited な分だけが上下する」というずれを消す。

P18a は A1・A2・D12 だけからは出ない。第 7.5 節が README の A19 を引いてそこから P18a を出し、A19 を
果たす者のうち `borrow_ify` の側を `L16` で示す。A19 が要ることは D12 を満たす 2 つの反例 `C1` と `C2` が
示す (7.5.7 と 7.5.8)。どちらでも `cancel` が対を消して解放後の読みを作るので、この 2 つは P21 と P23 の
反例でもある。どちらも `insert_rc` の出力ではない (README の A19 が引く `p60-insert-rc.md` の `L10`)。

第 7.5.4 節が引くのは A19 の (i)、(ii-a)、(ii-b) の 3 つである。(ii-b) は「`bumps ≥ 1` である時点では
`held ≥ 1 + bumps`」という条件付きの形であり、条件を外した無条件の `held ≥ bumps` が要る段では、
第 7.5.3 節の `L14b` が (ii-a) の非負性と (ii-b) から 1 段でそれを出す。

### 7.1 局所の定義

この節から先で使う補題を `L7` から `L17` と番号を付ける。後から挟んだものには `L10a` のように枝番を
振る。`p30-cancel-walk.md` の補題を引用するときは `p30 の L10` のように書く。

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
`ActRefs^inh_ρ(n)` は `ActRefs(v, π)` の部分多重集合である。P6 より、`n` が `Retain` のとき
`ActRefs^inh_ρ(n)` は `n` が `ρ` で実際に作る参照の多重集合であり、`n` が `Release` のとき
`ActRefs^inh_ρ(n)` は `n` が `ρ` で実際に処分する参照の多重集合である。

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

**言明**。任意の型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元は、一方が他方の前置に
なることが無い。

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
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types
  最初の呼び出しの `path` は空である。`unpunched_field_types` は
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

**言明**。`x` を `B` の変数、`λ ∈ boxed_leaf_paths(ty(x))` とし、実行路 `ρ` の上で `x` が値を得ていると
する。次の 2 つが成り立つ。

- **(A)** `origin_inner(vars, type_env, x.name, λ)` の本体が `origin(vars, type_env, x'.name, λ')` を
  呼ぶとき、次の 3 つが成り立つ。ただし `Binding::Join(arm_results)` の腕については、`ρ` が選んだ
  アームの結果変数 `x' = arm_results[j]` についてだけ主張する。
  - **(i)** `λ' ∈ boxed_leaf_paths(ty(x'))` である。
  - **(ii)** `ρ` の上で `x'` は値を得ている。
  - **(iii)** `ρ` の上で、`λ` が `x` の値の inhabited な leaf であることと、`λ'` が `x'` の値の
    inhabited な leaf であることは同値である。
  - **(iv)** その leaf が inhabited であるとき、`obj(x, λ) = obj(x', λ')` である。
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
  <2>3. `ty(x) = ty(arm_results[j])` である。
    BY A12
    A12 の第 2 項が「アームの結果と `Match` の束縛変数の型」の一致を述べる。
  <2>4. (i) と (ii) が成り立つ。
    BY <2>1, <2>2, <2>3, A11
    (i) は <2>3 から。(ii) は、`ρ` が `arm_j` の本体を通り、その終端の `Ret` が `arm_results[j]` を
    名指す (<2>1) ので、その変数は `ρ` の上で値を得ている。
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
      A12 の第 6 項が「`Destructure` の容器が構造体であること」、第 5 項が「`Destructure` のフィールド
      変数とフィールドの型」の一致を述べる。この場合は `is_box` が偽なので unbox である。
    <3>2. 第 `idx` フィールドは `container.ty` の穴でない。
      BY A12
      A12 が「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が実際に持つ
      (punched でない) ものであること」を述べる。
    <3>3. `[idx] ++ λ ∈ boxed_leaf_paths(ty(container))` である。これが (i) である。
      BY D4, <3>1, <3>2
      D4 の第 5 規則より、unbox 集約の leaf は `unpunched_field_types` が返す各フィールド `i` について
      `[i] ++ (そのフィールドの型の leaf)` である。`container.ty` は unbox の構造体 (<3>1) であり、
      leaf を持つので `is_fully_unboxed` ではなく、closure でも box でも array でもない。
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
      BY D21
      D21 より、活性化が `Let(x, Match(scrut, arms), k)` で選ぶアームは `tag` が `scrut` の値の実行時の
      タグに等しいアームである。固定した活性化はこのアームを選んでいる。
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

#### L9 (`identity` は inhabited を決める)

**言明**。`x` を `B` の変数、`λ ∈ boxed_leaf_paths(ty(x))` とし、実行路 `ρ` の上で `x` が値を得ていると
する。`(u, σ) = origin(x, λ).identity()` と置く。このとき `ρ` の上で `u` は値を得ており、
`σ ∈ boxed_leaf_paths(ty(u))` であり、`λ` が `x` の値の inhabited な leaf であることと `σ` が `u` の値の
inhabited な leaf であることは同値である。

**証明** `origin(x, λ)` の計算の再帰についての帰納法で示す。P2 より `origin(x, λ)` は停止するので、その
再帰の木は有限であり、この帰納法は整礎である。`origin` は答えを `vars.origins` に記録して次から返すが、
記録するのは `origin_inner` が計算した答えそのものなので、答えは記録の有無で変わらない
(`CODE src/rc_ir/ownership.rs: origin`)。

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

実行路 `ρ` を固定する。オブジェクト `o ∈ VarPath` が `ρ` で**活性**であるとは、
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
`RcExpr::Release(v, π, s, k)` であるとき、各オブジェクト `o` について次が成り立つ。

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

**言明**。実行路 `ρ` と、`ρ` の上の節点 `n` を固定する。`pending(n)` の各要素 `p` と各オブジェクト `o`
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
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     DEF `Vec::retain` (Rust 標準ライブラリの規約: 閉包が偽を返した要素を取り除き、残る要素の値と
     相対順序を保つ)
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
`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上でまだ処分されていないものの多重集合を
`B(p, ρ)` とする。このとき `p.outstanding` は `B(p, ρ)` を `covers` する。とくに `p.outstanding` が
空ならば `B(p, ρ)` も空である。

**証明** 走査中の位置を `B` の節点 `n` の訪問の入口とし、`B(p, ρ)` を DEF bump の帰属の `B_ρ(n, p)` と
読む。

<1>1. `B_ρ(n, p)` は、`p.node` の `Retain` が `ρ` で実際に作った参照の多重集合から、`ρ` の上でその
      `Retain` より後にあって `un_bump` がその要素と対にした `Release` が実際に処分した参照の多重集合を
      引いたものである。
  BY DEF bump の帰属, DEF 実行時の作用, P6, D26, A8, L11
  L11 より DEF bump の帰属の表は `ρ` の上の場合を尽くすので、`pending(n)` の各要素 `p` の `B_ρ(n, p)` は
  表の第 1 行が置いた初期値から表を辿って得られる。第 1 行が `ActRefs^inh_ρ` を初期値に置き、第 2 行が
  それを引く唯一の行であり、第 3 行から第 6 行は値を引き継ぐだけである。第 2 行が引くのは、`un_bump` が
  `InBracket` でその要素と対にした `Release` の分だけである。

  `ActRefs^inh_ρ(n)` が実際に作る / 処分する参照の多重集合であることは、P6 と D26 の両方が要る。P6 は
  「実行時に `Retain(v, π)` が作る参照の多重集合は、この数え上げを inhabited な leaf に制限したものに
  等しく、`Release(v, π)` が処分する参照の多重集合も同じものに等しい」と述べるが、`ActRefs^inh_ρ(n)` は
  それをさらに**計数下**の leaf に制限したものである (DEF 実行時の作用)。落とした分 -- グローバル状態の
  オブジェクトを指す leaf -- は D8 の参照を持たず (D26)、`Retain`/`Release` は `H` を変えない (A8) ので、
  作る参照にも処分する参照にも数えられない。よって 2 つの制限は同じ多重集合を与える。

<1>2. 各オブジェクト `o` について、`o` が `ρ` で活性ならば `p.outstanding[o] = B_ρ(n, p)[o]` であり、
      活性でなければ `B_ρ(n, p)[o] = 0` である。
  BY L11

<1>3. 各オブジェクト `o` について `p.outstanding[o] ≥ B_ρ(n, p)[o]` である。
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
  BY <1>1, <1>4, <1>5

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
- 足りない前提は README の **A19** である。7.5.3 がそれを引き、7.5.4 が A19 から P18a を出す。出る形は
  **オブジェクトごとの形** -- 計数下オブジェクト `O` について `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]`
  として `H(O) ≥ n(O) + 1` -- であり、README の P18a の言明そのものである。名前ごとの弱い形を経由しない
  (7.5.1 と 7.5.4 の <1>3)。
- 7.5.4 が引くのは A19 の (i)、(ii-a)、(ii-b) である。(ii-b) は条件付きの形
  「`bumps ≥ 1` である時点では `held ≥ 1 + bumps`」であり、7.5.4 の `<1>3` はそれを `bumps ≥ 1` の類に
  直接当てる。他の類に要る無条件の `held ≥ bumps` は、7.5.3 の `L14b` が (ii-a) の非負性と (ii-b) から
  1 段で出す。
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
`N_ρ(n, O) ≥ 1` ならば `H(O) ≥ N_ρ(n, O) + 1` である。

**INV は P18a そのものである。** P18a の `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` は、走査中の位置を
節点の訪問の入口に取り、`B(p, ρ)` を D27 の帰属 (第 7.1 節の DEF bump の帰属) で読むと `N_ρ(n, O)` で
ある。内側の和を活性な名前に制限してよいのは、L11 (ii) より活性でない名前の `B_ρ` が 0 だからであり、
`obj_ρ(o)` が定まるのは DEF 名前の活性による。よって 7.5.4 が INV を示せば P18a が出る。

**オブジェクトごとに和を取るところは落とせない。** 名前ごとの `H(o) ≥ n(o) + 1` を足し合わせても
`H(O) ≥ Σ_o n(o) + 1` は出ない -- `+1` が名前の個数だけ立つからである。7.5.4 の <1>3 は、余りを
別名類ごとに 1 つ数えたうえで、`bumps` が正である類が 1 つあれば足りるという形で和を取るので、
オブジェクトごとの形をそのまま与える。

#### 7.5.2 別名の歩みと別名類

##### DEF ρ-歩みと ρ-終端

`ρ` の上で値を得ている変数 `x` と `λ ∈ boxed_leaf_paths(ty(x))` をとる。L8 (A) が挙げる対 `(x', λ')` --
`origin_inner(vars, type_env, x.name, λ)` が `origin` を呼ぶ相手、`Binding::Join` の腕については `ρ` が
選んだアームの結果変数のもの -- を `(x, λ)` の **ρ-歩み**と呼ぶ。`origin_inner` が `origin` を呼ばない
とき、`(x, λ)` を **ρ-終端**と呼ぶ。

##### L12 (ρ-歩みは終端で終わる)

**言明**。`ρ` の上で値を得ている `x` と `λ ∈ boxed_leaf_paths(ty(x))` について、`(x, λ)` から ρ-歩みを
辿る列は有限で、ρ-終端で終わる。その終端を `T_ρ(x, λ)` と書く。列の各対は L8 (A) の (i)-(iv) を満たす。

**証明**

<1>1. `origin(x, λ)` の計算の再帰は有限の木である。
  BY P2
  P2 より `origin(x, λ)` は panic せずに停止する。

<1>2. ρ-歩みの列は <1>1 の再帰の木の 1 本の枝である。
  BY DEF ρ-歩みと ρ-終端, CODE src/rc_ir/ownership.rs: origin_inner
  `origin_inner` が呼ぶ `origin` の相手のうち 1 つを選んで進むので、列は木の根から下る 1 本の枝である。

<1>3. QED
  BY <1>1, <1>2, L8
  有限の木の枝は有限で、その最後の対では `origin_inner` が `origin` を呼ばない。各段が L8 (A) の
  (i)-(iv) を満たすことは、L8 の言明が `Binding::Join` の腕について `ρ` が選んだアームを主語に
  することによる。

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

<1>1. `(x, λ)` とその ρ-歩み `(x', λ')` について `obj(x, λ) = obj(x', λ')` である。
  BY L8, L12
  L8 (A) の (iv) がこれを述べる。L12 より ρ-歩みの各段は L8 (A) を満たす。

<1>2. QED
  BY <1>1, L12, DEF 別名類
  L12 より `(x, λ)` から `T_ρ(x, λ)` への ρ-歩みの列は有限であり、<1>1 をその各段に当てると
  `obj(x, λ) = obj(T_ρ(x, λ))` である。1 つの別名類の 2 つのスロットは同じ `T_ρ` を持つので、
  どちらも同じオブジェクトを指す。

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

##### L14a (1 つの別名類のスロットは名前を共有する)

**言明**。同じ別名類に属する 2 つのスロット `(x, λ)` と `(w, μ)` について、
`acted_on(x, λ) ∩ acted_on(w, μ) ≠ ∅` である。

**証明**

<1>1. `T_ρ(x, λ) = (u, σ)` と置くと `origin(u, σ) = Origin::Exactly((u.name, σ))` であり、
      `acted_on(u, σ) = {(u.name, σ)}` である。
  BY L8, D15
  ρ-終端では `origin_inner` が `origin` を呼ばないので、L8 (B) よりその値は `Origin::Exactly` である。
  D15 より `Origin::Exactly(p)` の `acted_on()` は `{p}` である。

<1>2. `acted_on(x, λ) ∋ (u.name, σ)` である。
  BY <1>1, L12, L13
  L12 より `(x, λ)` から `(u, σ)` への ρ-歩みの列は有限であり、L13 をその各段に当てると
  `acted_on(x, λ) ⊇ acted_on(u, σ) = {(u.name, σ)}` である。

<1>3. QED
  BY <1>2, DEF 別名類
  `T_ρ(w, μ) = (u, σ)` でもあるので、<1>2 と同じ議論が `(w, μ)` についても成り立つ。よって
  `(u.name, σ)` は両方の `acted_on` に入る。

#### 7.5.3 要る前提

##### DEF 類ごとの参照

**計数下の**別名類 `C` -- `obj(C)` が D26 の意味で計数下である類 -- について、`ρ` を辿る活性化の各時点
`τ` における `held_ρ(τ, C)` を次の規則で定める。グローバル状態の類には定めない。

| 事象 | `held_ρ(·, C)` の変化 |
|---|---|
| `C` の ρ-終端が D10 の生成で作られる | 1 から始まる |
| `C` の ρ-終端が、所有する (D14) パラメータ・capture の leaf である | 1 から始まる (D10 の初期値) |
| `C` の ρ-終端が、借用する (D14) パラメータ・capture の leaf である | 1 から始まり、活性化の間ずっと 1 以上である (P14a) |
| `Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき +1 |
| `Release(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき -1 |
| `(w, μ) ∈ C` の D9 の消費 | -1 |

計数下の類に限るのは、README の A19 が「計数下の類に限るのは、グローバル値を終端とする類に `held` の
開始値を与える行が無いからである (D26)」と述べるとおり、ρ-終端が束縛を持たない名前 (グローバル) である
類にこの表が開始値を与えないからである。`origin_inner` は `vars.bindings.get(var)` が `None` のとき
`here()` を返すので、そういう ρ-終端は在りうる (`CODE src/rc_ir/ownership.rs: origin_inner`)。

D8 は同じオブジェクトへの参照を区別しないので、この勘定は「どの参照がどの類のものか」を決める取り決めで
ある。取り決めが実行時のカウントと整合することを A19 (i) が要求する。

##### bumps

計数下の別名類 `C` と時点 `τ` について、`bumps_ρ(τ, C) = Σ_{p ∈ pending} Σ_{o : C_ρ(o) = C} B_ρ(p)[o]`
と定める。内側の和は、`ρ` の上のスロットである名前 `o` のうち `C_ρ(o) = C` であるものを渡る。L14 より、
`B_ρ(p)` が個数を付けている名前はどれも `ρ` の上のスロットであり、ちょうど 1 つの別名類に属する。

##### A19 (README 第 4 節)

**この文書は README の A19 を前提として引く。** 使う 2 つの節の文面は次のとおりである (README 第 4 節)。

> - **(i)** 各時点と各計数下オブジェクト `O` について、`H(O)` は `O` を指す別名類が持つ参照の総数以上で
>   ある。すなわち相異なる類の参照は相異なる。
> - **(ii-a) 由来の形** -- 読む者: P14。各時点と各計数下の別名類について、その類が持つ参照の個数は非負で
>   あり、読む構文と `Retain`/`Release` がその類を名指す時点では 1 以上である。
> - **(ii-b) 帳簿の形** -- 読む者: P18a、P19、P21。各時点と各計数下の別名類について、走査がその類に
>   ついて `pending` に数えている bump の個数を `bumps`、その類が持つ参照の個数を `held` とすると、
>   **`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である。**

README は「**`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の
入力) の両方**について、各実行路 `ρ`、`ρ` を辿る各活性化について、次の 2 つを仮定する」と範囲を定めて
いるので、`cancel` の入力についてこの 3 つの節がどれも使える。この文書の記法では、(ii-b) は
「`bumps_ρ(τ, C) ≥ 1` である時点では `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)`」である。

条件を落とした形が使えないことも README が述べる。無条件の `held ≥ 1 + bumps` は `bumps = 0` の時点で
`0 ≥ 1` を要求するので偽であり (第 7.5.7 節の `f` の `Release(b, [])` の直後がその時点である)、
`held ≥ bumps` は真だが弱すぎて P18a が要る形を出さない (第 7.5.8 節の `C2` の類 `C_o` が
`held = bumps = 1` の時点を持つ)。

##### L14b (`held` は `bumps` 以上である)

**言明**。計数下の別名類 `C` と時点 `τ` について `held_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。

**証明**

<1>1. CASE `bumps_ρ(τ, C) = 0`。A19 (ii-a) より `held_ρ(τ, C) ≥ 0` である。
  BY A19

<1>2. CASE `bumps_ρ(τ, C) ≥ 1`。A19 (ii-b) より `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C) ≥ bumps_ρ(τ, C)` で
      ある。
  BY A19

<1>3. QED
  BY <1>1, <1>2
  `bumps_ρ(τ, C)` は非負整数なので、この 2 つで場合を尽くす。

#### 7.5.4 A19 から P18a が出る

**証明**

<1>1. ASSUME `ρ` の上の節点 `n` と計数下オブジェクト `O` をとり、`N_ρ(n, O) ≥ 1` とする。
      PROVE `H(O) ≥ N_ρ(n, O) + 1`。
  この形が INV(n) であり、7.5.1 より P18a である。

<1>2. `N_ρ(n, O) = Σ_{C : obj(C) = O} bumps_ρ(n, C)` である。
  BY DEF `N`, DEF bumps, DEF 別名類, L12a, L14
  `N_ρ(n, O)` の内側の和は、活性で `obj_ρ(o) = O` である名前 `o` を渡り、`B_ρ(n, p)[o]` を足す。L14 より
  各名前 `o` は `ρ` の上のスロットであり、ちょうど 1 つの別名類 `C_ρ(o)` に属し、L12a より
  `obj(C_ρ(o)) = obj_ρ(o)` である。逆に計数下の類 `C` について、`C_ρ(o) = C` である名前 `o` は
  `obj_ρ(o) = obj(C)` を満たし、DEF 名前の活性の 3 条件 (スロットであること、inhabited、計数下) を
  満たすので活性である。よって「`obj_ρ(o) = O` である活性な名前」と「`obj(C) = O` である類 `C` に属する
  名前」は同じ集合であり、和を類ごとに括ると `bumps_ρ(n, C)` の和になる (DEF bumps)。

<1>3. `Σ_{C : obj(C) = O} held_ρ(n, C) ≥ N_ρ(n, O) + 1` である。
  BY A19, L14b, <1>1, <1>2
  <1>1 と <1>2 より `bumps_ρ(n, C0) ≥ 1` である類 `C0` が少なくとも 1 つ在る。その `C0` については
  A19 (ii-b) が `held_ρ(n, C0) ≥ 1 + bumps_ρ(n, C0)` を直接与える。他の各類 `C` については L14b が
  `held_ρ(n, C) ≥ bumps_ρ(n, C)` を与える。足し合わせると
  `Σ_C held_ρ(n, C) ≥ Σ_C bumps_ρ(n, C) + 1 = N_ρ(n, O) + 1` である。**`+1` は 1 回しか立たない** --
  (ii-b) を使うのは `C0` の 1 つだけで、他の類には L14b を使うからである。

<1>4. QED
  BY A19, <1>3
  A19 (i) より `H(O) ≥ Σ_{C : obj(C) = O} held_ρ(n, C)` である。<1>3 と合わせて
  `H(O) ≥ N_ρ(n, O) + 1` であり、これが <1>1 の PROVE である。

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

<1>2. 以下 `V` が借用版であるとする。`rewrite_rc` が `Retain(p, u)` を落とすのは `owns_unit(p, u)` が
      偽のときに限る。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  `units_under(&v.ty, path, type_env)` を `self.owns_unit(v, unit)` で絞る。A2 より `path` は `ty(p)` の
  unit なので `units_under` はその 1 元であり、絞りが落とすのは `owns_unit` が偽のときである。

<1>3. `V` の本体に `Retain(p, u)` 節点が在るとき、`(p, u)` は `levelled_sites(V の本体, type_env)` の
      元である。また消費が `App(callee, args)` の引数 `p` の位置であるとき、`(p, u)` は `u` が
      `rc_units(ty(p))` の元であるかぎり `levelled_sites` の元である。
  BY CODE src/rc_ir/borrow.rs: levelled_sites, A2, P1
  `levelled_sites` は `RcExpr::Retain(v, path, _, _) | RcExpr::Release(v, path, _, _)` の腕で
  `(v, path)` を、`RcExpr::Let(_, RcRhs::App(_, args), _)` の腕で各 `arg` と各
  `unit ∈ rc_units(&arg.ty, type_env)` について `(arg, unit)` を挙げる。`u = truncate_to_unit(ty(p), μ)`
  は P1 より `rc_units(ty(p))` の元である。`Retain(p, u)` 節点の `path` が `u` であることは A2 による。

<1>4. CASE 消費が `App` の所有位置の引数以外 -- `App` の callee、`Closure` の capture、
      boxed/unbox の `Destructure`、終端の `Ret` -- であり、かつ `V` の本体に `Retain(p, u)` 節点が在る。
  <2>1. `collect_consumes` は `(p, μ)` を報告する。
    BY P7, CODE src/rc_ir/ownership.rs: collect_consumes_go, CODE src/rc_ir/ownership.rs: rhs_consumes
    P7 より D9 の意味で消費する構文はすべて `collect_consumes` が報告する。この 5 種について
    `collect_consumes_go` と `rhs_consumes` の判定は `own` を読まないので (`rhs_consumes` の
    `RcRhs::App` の腕は `callee` の leaf を無条件に `out` に入れ、`RcRhs::Closure` の腕は各 capture の
    leaf を無条件に入れる)、報告はどの所有の割り当てでも起きる。
  <2>2. `origin(p, μ)` の候補であるパラメータ leaf はすべて `owned_leaves` に入っている。
    BY P8, <2>1
  <2>3. `origin(p, μ)` の各候補 `(r, q)` について `owns_object(r, q)` は真である。
    <3>1. `owns_object(r, q)` は、`r` が `V` のパラメータでないとき真である。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
      `self.vars.param_tys.get(root)` が `None` の腕が `true` を返す。
    <3>2. `r` が `V` のパラメータであるとき、`owns_object(r, q)` は
          `units_under(ty(r), q)` の各 unit `q'` について
          `(r, truncate_to_unit(ty(r), q'))` が `owned_units` に入るかを見る。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object
    <3>3. `V` が借用版であるとき、`V` のパラメータ名は複製の名前 `rename[&p0.name]` であり、
          `owned_units` に入るのは `(rename[&p0.name], truncate_to_unit(&p0.ty, &leaf, type_env))` --
          原本のパラメータ `p0` と、`owned_leaves.owns(&p0.name, &leaf)` が真である `leaf` -- である。
      BY CODE src/rc_ir/borrow.rs: borrow_ify, P9
      `borrow_ify` は `for p in &func.params { for leaf in boxed_leaf_paths(&p.ty, type_env) {
      if owned_leaves.owns(&p.name, &leaf) { ... owned_units.insert((rename[&p.name].clone(), unit)); } } }`
      を行う。**鍵は複製後の名前で、`owned_leaves` は原本の名前で引かれる。**この改名が P8 と `V` を
      繋ぐ橋である。P9 より複製の本体は原本の束縛変数を `rename` で一斉に付け替えたものなので、`V` の
      本体の `origin` が返す候補の名前も `rename` の像であり、`ty(rename[&p0.name]) = ty(p0)` である。
    <3>4. QED
      BY <2>2, <3>1, <3>2, <3>3, P7e, P9
      <2>2 は原本の名前で「`origin(p, μ)` の候補であるパラメータ leaf はすべて `owned_leaves` に入る」を
      与える。P9 の改名でその候補は `V` の候補に写り、<3>3 の `owned_units` はまさにその leaf の unit を
      鍵に持つ。P7e より `owns_object(r, q) = owns_object(r, truncate_to_unit(ty(r), q))` なので、
      <3>2 の検査は通る。`V` が借用版でないときは `owned_units` がすべてのパラメータ unit を持つので
      (`CODE src/rc_ir/borrow.rs: borrow_ify` の `owned_units.extend(param_capture_units(func, type_env))`)
      やはり真である。
  <2>4. QED
    BY <2>3, <1>3, P7a, <1>2
    この CASE の仮定より `V` の本体に `Retain(p, u)` 節点が在るので、<1>3 より `(p, u)` は
    `levelled_sites` の元である。`μ` は inhabited であり、`u` の下の leaf である
    (`u = truncate_to_unit(ty(p), μ)` と P1)。よって <2>3 は P7a の節 2 である。P7a より
    `owns_unit(p, u)` は真であり、<1>2 より節点は落ちない。これが (A) である。

<1>5. CASE 消費が `App(callee, args)` の引数の位置である。
  <2>1. その位置で消費が起きるのは、`rhs_consumes` の `is_owning_position` が真のとき、すなわち
        `owned_units` が `(callee のパラメータ, u)` を含むときである。
    BY CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
    `consume_rhs` の `owns` は `self.owned_units.contains(&(p.name.clone(),
    truncate_to_unit(&p.ty, leaf, self.type_env)))` である。
  <2>2. `V` の本体の `App` の呼び出し先は `route` が決めた版であり、`call_rc` はその版の
        `owned_units` を `callee_owns` として読む。`callee_owns` は <2>1 の判定と同じものである。
    BY P12, P13, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc, CODE src/rc_ir/borrow.rs: borrow_ify,
       CODE src/rc_ir/ownership.rs: all_owned_units
    `call_rc` は `self.owned_units.contains(&(params[arg_idx].0.clone(), unit.clone()))` を見る。
    `callee_params` は出力の各版のパラメータ名と型を持ち、`owned_units` は出力の各版の所有 unit を持つ。
    <2>1 の判定が読む集合は `cancel` の `all_owned_units(prog, type_env)` -- 出力の各版のパラメータ
    unit のうち `borrowed_units` に入らないもの -- であり、P13 よりこれは `borrow_ify` の `owned_units` の
    パラメータ・capture の部分に一致する。
  <2>3. CASE `owns_unit(p, u)` が真である。`V` の本体に `Retain(p, u)` 節点が在れば <1>2 より落ちない
        ので (A) であり、無ければ (C) である。
    BY <1>2
  <2>4. CASE `owns_unit(p, u)` が偽である。このとき `call_rc` は `(p, u)` を `before` に入れる。
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc
    `call_rc` は `callee_owns && !arg_owned` のとき `before.push((arg.clone(), unit))` を行う。
    <2>1 と <2>2 より `callee_owns` は真であり、仮定より `arg_owned = owns_unit(p, u)` は偽である。
  <2>5. QED
    BY <2>3, <2>4, P11
    <2>4 の場合、`call_rc` が `before` に入れた `(p, u)` について、P11 よりこの呼び出しの前に
    `Retain(p, u)` が置かれる。これが (B) である。<2>3 と <2>4 は `owns_unit(p, u)` の真偽で場合を
    尽くす。

<1>5a. CASE 消費が `App` の所有位置の引数以外であり、`V` の本体に `Retain(p, u)` 節点が無い。
  BY DEF (C)
  これが (C) である。

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
`Obl` と `H` は `{O_p}, 1` → `{O_p, O_q}, 1` → (移動) → `{O_p, O_p, O_q}, 2` → `{O_p, O_q}, 1` →
`{O_p}, 1` → `{O_p}, 1` → `{}, 0` と動く。(S-a) は各除去が `Obl` に入っているので、(S-b) は終端で `Obl` が
空で `ty(u) = I` に boxed leaf が無いので、(S-c) は読む構文 (`App(f, [p])` で `H(O_p) = 2`、
`App(f, [q])` で `H(O_q) = 1`、`Eval(m)` で `H(O_p) = 1`) と触れる構文 (`Retain(m, [])` と
`Release(m, [])` でどちらも `H(O_p) = 1`) がいずれも解放されていないオブジェクトを相手にするので、
成り立つ。`App` の `callee` の leaf が指すオブジェクトは D26 のグローバル状態のものであり勘定の外に
居る。よって `C1` は D12 を満たし、A2 も満たす。

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

**この文書がかつて差し戻した 5 点は、どれも README にすでに在る。** 直近の 2 点 (計数下の安定性と
A19 (ii-b) の形) は、この文書が差し戻した後に README が答えたものである。

| かつての差し戻し | 主張 | README の現在の文 |
|---|---|---|
| 6 | bump の帰属を定義に置くか、「README にはまだ無い」 | **D27 (bump の帰属)** として第 5 節に在り、位置も「走査中の位置 -- **節点の訪問の入口**」と定めている。第 7.1 節の DEF bump の帰属はこれと同じものである |
| 7 | A19 を README の第 4 節に置くか | **A19 (bump の下に余りが在る)** として第 4 節に在り、果たす者は「`insert_rc` (使用回数の勘定) と `borrow_ify` (`rewrite_rc` が落とさないこと、落とす場合に `call_rc` が補うこと)」と書かれている |
| 8 | `C1`/`C2` を第 8 節でなく A19 の根拠として引くか | A19 の中で「`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` と第 7.5.8 節の `C2` が、D12 を満たしながら P18a を破る本体である。この 2 つはコードの欠陥ではなく、この仮定が要ることの証拠である」と引かれている |
| 1 (計数下の安定性を D26 に書く) | D26 が、遷移が 1 つの活性化の中で起きるかどうかを書いていない | D26 の最後の段落が「**1 つの活性化の間、そこに現れるオブジェクトが計数下であるかどうかは変わらない。** `mark_global` の呼び出しはコード生成に 1 か所しかなく、グローバル初期化子の本体を評価した結果に対してだけ走る (`CODE src/rc_ir/codegen.rs: Generator::implement_rc_global`)。初期化子は引数を持たない (`CODE src/rc_ir/ast.rs: RcGlobalInit`) ので … 命題が『各時点の計数下オブジェクト』を量化するとき、その集合は活性化の間ずっと同じである」と述べている |
| 2 (A19 の導かれた形は不等式から出ない) | (ii-b) を `held ≥ bumps` と書き、条件付きの形をそこから導いていた | (ii-b) 自身が条件付きの形になった -- 「`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である」。README は続けて「`held ≥ bumps` は真だが弱すぎて、P18a が要る形を出さない。`held = bumps` かつ `bumps ≥ 1` である時点を `held ≥ bumps` は許し、`p13-disposals-and-pending.md` の `C2` の類 `C_o` がまさにその時点だからである」と、台帳での 1 単位差 (`U + X - D ≥ -1` と `≥ 0`) とともに述べている |

差し戻し 8 に添えた「`C1` の形は実際の入力に 11 件ある」には出典が無かった。この文書はその数を使わない。

**第 7 節から新しく差し戻す点は無い。**
