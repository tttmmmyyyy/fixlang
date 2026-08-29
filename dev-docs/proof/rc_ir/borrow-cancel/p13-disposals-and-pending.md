# P7c: 処分はすべて走査に届く

この文書は README の P7c を扱う。README の定義 D1-D19、仮定 A1-A14、および命題 P1 と P5 の**言明**の上に
立つ。それらの証明は `p10-leaves-and-units.md` と `p12-identity-and-consumes.md` にあり、この文書は
その言明だけを使う。P2、P3、P4、P6、P7、P7a、P7d は引用してよい位置にあるが、この文書は使わない。

**結論を先に書く。P7c は証明できた。** ただし言明が名指す仕組みは 2 つで、コードが持つ仕組みは 3 つである。
README の P7c は「`consume_objects` または `un_bump` を呼ぶ」と述べるが、終端の `Ret` の訪問はそのどちらも
呼ばず、その時点の `pending` の各要素を直接 `needed_retains` に入れる (`CODE src/rc_ir/borrow.rs:
CancelAnalysis::walk_inner` の `RcExpr::Ret` の腕)。第 3 節がこの 3 つ目の仕組みを含む形を `P7c′` として
書き、第 5 節がそれを証明する。第 6 節が README への差し戻しをまとめる。

第 4 節に補題 `L6` を置く。これは P7c の言明には無いが、`Release` の訪問の後に `pending` に何が残るかを
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

**D9 の `App` の行の読み。** D9 は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」と書く。
「その位置の unit」は、引数 `a_i` の型と呼び出し先のパラメータ `p_i` の型のどちらで取るとも読めるが、
D9 の行が意味を持つのは 2 つの型が対応するときだけである。この文書は、`ty(a_i)` と `ty(p_i)` が等しいと
読み、unit を `truncate_to_unit(ty(p_i), λ, type_env)` で取る。第 6 節の差し戻し 3 がこれを述べる。

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
            ある。また DEF 処分 leaf の「D9 の `App` の行の読み」より `ty(a_i) = ty(p_i)` であり、
            `λ` は `ty(p_i)` の boxed leaf である。
        BY DEF 処分 leaf, <4>1
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
第 5 節の <1>8 の <3>3 がこの対応の上に立っている。

### 差し戻し 4 (L6 を README の命題として置くか)

第 4 節の L6 は「`un_bump` が `NoBracket` か `OutsideBracket` を返したとき、`Release` の訪問の後の
`pending` のどの要素も `Obj(n)` のどのオブジェクトも名指さない」を述べる。P7c の言明はここまで言わないが、
P18a はこれを使う -- P7c だけでは「呼んだ」までしか出ず、「pending から消えた」が出ないからである。
L6 をこの文書の補題のまま置くか、README の命題に上げるかは、P18a を書く側の判断による。

### 差し戻し 5 (`stacker::maybe_grow` を外部の結果として README に置くか)

`grow_stack` を経由する関数 (`walk`、`rewrite`、`origin`、`drop_nodes`) の性質は、
`stacker::maybe_grow` が閉包をちょうど 1 回呼ぶことに立つ。この文書は第 1 節でそれを外部の結果として
述べた。`p20-borrow-ify.md`、`p30-cancel-walk.md`、`p40-cancel-soundness.md` も同じ事実を各自で述べて
いるので、README の第 3 節に 1 度だけ置くほうがよい。

## 7. P18b と P18a

**結論を先に書く。P18b は A1・A2・D12 から証明できた。P18a は前提 `A19` を 1 つ足すと証明できる。**

P18b は第 7.4 節が証明する。証明の要は補題 `L9` -- `origin` の `identity` が等しい 2 つの leaf は、
実行時に同時に inhabited (D16) であるか同時にそうでないか、のどちらかである -- であり、これが
「`outstanding` は静的な数え上げを引き、実行時は inhabited な分だけが上下する」というずれを消す。

P18a は A1・A2・D12 だけからは出ない。第 7.5 節が、足りない前提を `A19` として書き、そこから P18a を
出し、`A19` を果たす者のうち `borrow_ify` の側を示す。`A19` が要ることは D12 を満たす 2 つの反例 `C1` と
`C2` が示す (7.5.7 と 7.5.8)。どちらでも `cancel` が対を消して解放後の読みを作るので、この 2 つは
P21 と P23 の反例でもある。どちらも `insert_rc` の出力ではない。

### 7.1 局所の定義

この節から先で使う補題を `L7` から `L16` と番号を付ける。後から挟んだものには `L10a` のように枝番を
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

`B_ρ(n, p)` を、P18b と P18a の言明の `B(p, ρ)` として読む。第 7.6 節の差し戻し 6 が、この帰属を README に
書くことを提案する。

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
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  最初の呼び出しの `path` は空である。<1>2 より、子の呼び出しの入口の `path` は親の入口の `path` に
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
- **(B)** `origin_inner(vars, type_env, x.name, λ)` の本体が `origin` を 1 回も呼ばないとき、その値は
  `Origin::Exactly((x.name, λ))` である。

(iii) の「inhabited」は第 7.1 節の読みのものである。null についての A5 の例外がこの同値を壊さないのは、
以下の各腕が示す関係が、leaf の位置に**同じ値**を置くか (`Move`、`Join`、`Field`、`Payload`)、**同じ
参照**を置くか (A3 の単一の `Arg(j, σ)` の行) のどちらかだからである。どちらでも一方が null であることと
他方が null であることは同値である。

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
  <2>5. (iii) が成り立つ。
    BY <2>1, <2>2, D16
    D16 の inhabited は値とその型だけで決まる。<2>1 と <2>2 より `x` と `y` は同じ値と同じ型を持つので、
    同じ leaf について同じ答えになる。
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
  <2>5. (iii) が成り立つ。
    BY <2>2, <2>3, D16
    <1>2 の議論と同じく、同じ値と同じ型は同じ leaf について同じ答えを与える。
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
        ある。
    BY A3, <2>1, <2>2
    A3 の表の「単一の `Arg(j, σ)`」の行が、その leaf に置かれるのが「第 `j` オペランドの leaf `σ` と
    同じ参照」であり、「結果のその leaf が inhabited であることと、第 `j` オペランドの leaf `σ` が
    inhabited であることは同値である」と述べる。
  <2>4. (ii) が成り立つ。
    BY A11, CODE src/rc_ir/ownership.rs: collect_bindings
    `Binding::Llvm` は `Let(x, RcRhs::Llvm(gen, args), k)` からだけ作られ、A11 より各 `args[j]` は
    その位置でスコープに入っている束縛に解決する。
  <2>5. QED
    BY <2>3, <2>4
    (i) と (iii) は <2>3 である。この腕は `origin` を呼ぶので (B) は空虚に成り立つ。

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
      BY D16, <3>1, <3>3, <3>4
      D16 より `[idx] ++ λ` が `container` の値の inhabited な leaf であることは、その路が通る unbox
      union の各節で選ぶ変位番号がその節のタグに等しいことである。<3>1 より `container.ty` は構造体なので
      根の節は union ではなく、先頭の添字 `idx` は union の節を通らない。残りの節は `λ` が `x` の値の中で
      通る節そのものである (<3>4)。よって (iii) が成り立つ。この腕は `origin` を呼ぶので (B) は空虚に
      成り立つ。
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
      BY <3>1, <3>2, D16
      同じ値と同じ型は同じ leaf について同じ答えを与えるので (i) と (iii) が成り立つ。この腕は
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
      BY D16, <3>2, <3>3, <3>4
      D16 より `[tag] ++ λ` が `scrut` の値の inhabited な leaf であることは、その路が通る unbox union の
      各節で選ぶ変位番号がその節のタグに等しいことである。根の節は `scrut` 自身であり、そこで選ぶ変位は
      `tag`、タグも `tag` (<3>3) なので一致する。残りの節は `λ` が `x` の値の中で通る節そのものである
      (<3>4)。よって (iii) が成り立つ。この腕は `origin` を呼ぶので (B) は空虚に成り立つ。
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
意味で計数下であることをいう。inhabited であるかどうかが対の選び方によらないことは L10 が、`obj(x, λ)` が
対の選び方によらないことは P5 (a) が与える (identity が等しい 2 つの leaf のスロットは同じオブジェクトを
指す)。活性な名前 `o` が指すオブジェクトを `obj_ρ(o)` と書く。

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
  <2>2. QED
    BY <2>1, <1>1, DEF 実行時の作用
    `Inh_ρ(v, π, n)` は `L(v, π)` の元のうち inhabited なものの全体なので、<2>1 より、`o` で名付け
    られる `L(v, π)` の元はすべて `Inh_ρ(v, π, n)` に入る。よって 2 つの数え上げの `o` の個数は等しい。

<1>4. CASE `o` が `ρ` で活性でない。
  <2>1. `Inh_ρ(v, π, n)` の元 `λ` で `origin(v, λ).identity() = o` となるものは無い。
    BY DEF 名前の活性, <1>2
    そのような `λ` が在れば、`(v, λ)` は DEF 名前の活性の条件を満たす対なので `o` は活性である。
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
  BY DEF bump の帰属, DEF 実行時の作用, P6, L11
  L11 より DEF bump の帰属の表は `ρ` の上の場合を尽くすので、`pending(n)` の各要素 `p` の `B_ρ(n, p)` は
  表の第 1 行が置いた初期値から表を辿って得られる。第 1 行が `ActRefs^inh_ρ` を初期値に置き、第 2 行が
  それを引く唯一の行であり、第 3 行から第 6 行は値を引き継ぐだけである。DEF 実行時の作用と P6 より、
  `Retain` の `ActRefs^inh_ρ` はそれが実際に作る参照の多重集合であり、`Release` の `ActRefs^inh_ρ` は
  それが実際に処分する参照の多重集合である。第 2 行が引くのは、`un_bump` が `InBracket` でその要素と
  対にした `Release` の分だけである。

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
- 足りない前提は 1 つの形にまとまる。7.5.3 の **`A19`** -- 「別名類は、走査が `pending` に持っている
  bump より 1 つ多い参照を持つ」-- である。7.5.4 が `A19` から P18a を出す。出る形は
  **オブジェクトごとの形** -- 計数下オブジェクト `O` について `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]`
  として `H(O) ≥ n(O) + 1` -- であり、README の P18a の言明そのものである。名前ごとの弱い形を経由しない
  (7.5.1 と 7.5.4 の <1>3)。
- `A19` を果たす者は `insert_rc` と `borrow_ify` である。**この文書が示せるのは `borrow_ify` の側だけで
  ある。** 7.5.5 がそれを示す -- `rewrite_rc` が借用版で落とす `Retain` は、その leaf を消費する構文が
  残っている限り落ちない (P8 と P7a)。落ちる場合には `call_rc` が消費の直前に同じ `Retain` を置く。
  すなわち**依頼された疑い -- `borrow_ify` が余りを壊す -- は当たらない**。
- `insert_rc` の側 (アームの `retain_if_live` と使用回数の勘定) は、この証明の対象の外にある `insert_rc`
  についての命題であり、7.5.6 がそれを言明として書き出す。

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
辿る列は有限で、ρ-終端で終わる。その終端を `T_ρ(x, λ)` と書く。列の各対は L8 (A) の (i)(ii)(iii) を
満たす。

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
  (i)(ii)(iii) を満たすことは、L8 の言明が `Binding::Join` の腕について `ρ` が選んだアームを主語に
  することによる。

##### DEF 別名類

`ρ` の上で値を得ている変数の inhabited な boxed leaf の対 -- すなわち `ρ` の上のスロット (D6) -- を、
`T_ρ` が等しいという関係で分けた同値類を**別名類**と呼ぶ。スロット `(x, λ)` が属する別名類を
`C_ρ(x, λ)` と書く。D20 と L8 (A) より、ρ-歩みの各段は D20 の別名の辺であり、逆に `ρ` の上で実際に
値を渡す別名の辺は ρ-歩みである。すなわち別名類とは、`ρ` の上で別名の辺で結ばれたスロットの集まりで
ある。

L8 (A) の (iii) より、1 つの別名類のスロットは同時に inhabited であるかそうでないかであり、P5 (a) と
L9 より、1 つの別名類のスロットはすべて同じオブジェクトを指す。その オブジェクトを `obj(C)` と書く。

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

##### L14 (1 つの別名類のスロットは名前を共有する)

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

##### L15 (走査は名前を共有する処分でその要素を落とす)

**言明**。`n` を `ρ` の上の節点で、その式が `RcExpr::Let(x, RcRhs::App(..), k)`、
`RcExpr::Let(x, RcRhs::Closure(..), k)`、`RcExpr::Let(x, RcRhs::Llvm(..), k)`、
`RcExpr::Destructure(..)` のいずれかであるとする (D9 の消費を行う節点)。`(w, μ) ∈ Disp(n)` とし、
`p ∈ pending(n)` の由来を `t = Retain(v, π)`、`λ ∈ L(v, π)` とする。`identity(v, λ)` が `ρ` で活性で
あり `B_ρ(n, p)[identity(v, λ)] ≥ 1` であり、かつ `identity(v, λ) ∈ acted_on(w, μ)` であるとする。
このとき、`ρ` の上で `n` の直後の節点 `k` について `pending(k)` に `node = p.node` の要素は無く、
`p.node` は `n` の訪問の後 `self.needed_retains` の元である。

**証明**

<1>1. `p.outstanding` は `identity(v, λ)` を鍵に持つ。
  BY L11, D15, CODE src/rc_ir/ownership.rs: References::names
  仮定と L11 (i) より `p.outstanding[identity(v, λ)] = B_ρ(n, p)[identity(v, λ)] ≥ 1` である。D15 より
  `names(o)` は `o` を含むかを答え、`References::names` の本体は `self.0.contains_key(object)` である。

<1>2. `n` の訪問は `self.consume(&mut pending, &w.name, &μ)` を 1 回呼ぶ。
  BY 第 5 節の <1>8、<1>9、<1>10、<1>11
  第 5 節のこの 4 つの場合が、`Disp(n)` の各元について `consume` が呼ばれることを示す。

<1>3. <1>2 の呼び出しは `consume_objects` を 1 回呼び、その `objects` の元の全体は `acted_on(w, μ)` で
      ある。
  BY L4

<1>4. その `consume_objects` の呼び出しは `p` を `pending` から取り除き、`p.node` を
      `self.needed_retains` に入れる。
  BY <1>1, <1>3, L11 の <1>1a
  仮定より `identity(v, λ) ∈ acted_on(w, μ)` であり、<1>1 より述語の第 1 の選言肢
  `retain.outstanding.names(object)` が真である。

<1>5. `n` の訪問はその後 `pending` に要素を足さず、`walk(k, pending, ·)` を呼ぶ。
  BY L11 の <2>3 の <3>1

<1>6. `self.needed_retains` は走査の間 要素を失わない。
  BY p30-cancel-walk.md の L10

<1>7. QED
  BY <1>4, <1>5, <1>6

**この補題が使うのは `outstanding` の鍵だけである。** `consume_objects` の述語は
`retain.outstanding.names(object)` であり、`pending` の要素はこの 1 つの名前の集合でしか照合されない。
`Retain(v, π)` の訪問は `other_objects` を呼ばないので、その `Retain` が触れうる他の名前 -- `Join` の
候補 -- は `pending` の要素に残らない (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の
`RcExpr::Retain(v, path, _, k)` の腕、`CODE src/rc_ir/borrow.rs: PendingRetain`)。7.5.7 の `C1` が
その帰結である。

#### 7.5.3 要る前提

##### DEF 類ごとの参照

`ρ` を辿る活性化の各時点 `τ` と別名類 `C` について、`held_ρ(τ, C)` を次の規則で定める。

| 事象 | `held_ρ(·, C)` の変化 |
|---|---|
| `C` の ρ-終端が D10 の生成で作られる | 1 から始まる |
| `C` の ρ-終端が、所有する (D14) パラメータ・capture の leaf である | 1 から始まる (D10 の初期値) |
| `C` の ρ-終端が、借用する (D14) パラメータ・capture の leaf である | 1 から始まる (呼び出し元が持つ参照、D14) |
| `Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき +1 |
| `Release(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ | その `λ` 1 つにつき -1 |
| `(w, μ) ∈ C` の D9 の消費 | -1 |

D8 は同じオブジェクトへの参照を区別しないので、この勘定は「どの参照がどの類のものか」を決める取り決めで
ある。取り決めが実行時のカウントと整合することを `A19` の (i) が要求する。

##### A19 (bump の下に余りが在る)

**この文書は次を前提として置く。README には無い。第 7.6 節の差し戻し 8 が、これを README の仮定として
置くことを提案する。果たす者は `insert_rc` と `borrow_ify` である。**

> `cancel` の入力の各本体 `B`、`B` の各実行路 `ρ`、`ρ` を辿る各活性化について、次の 2 つが成り立つ。
>
> - **(i)** 各時点 `τ` と各計数下オブジェクト `O` について、
>   `H(O) ≥ Σ_{C : obj(C) = O} held_ρ(τ, C)` である。
> - **(ii)** 各時点 `τ` と各別名類 `C` について、`held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` である。ここで
>   `bumps_ρ(τ, C)` は、その時点で走査が `pending` に持つ要素 `p` のうち、由来の `Retain(v, π)` が
>   `C` のスロットを名指すものについて、`B_ρ(p)` が `C` のスロットの `identity` に付けている個数の総和で
>   ある。

(i) は「相異なる類の参照は相異なる」であり、D8 と D10 が定める勘定がこの取り決めと合っていることを言う。
(ii) が主張の実質である -- **どの類も、走査が数えている bump より 1 つ多い参照を持っている。**

#### 7.5.4 A19 から P18a が出る

**証明**

<1>1. `ρ` の上の節点 `n` と計数下オブジェクト `O` をとり、`N_ρ(n, O) ≥ 1` とする。

<1>2. `N_ρ(n, O) = Σ_{C : obj(C) = O} bumps_ρ(n, C)` である。
  BY DEF `N`, DEF 別名類, DEF 類ごとの参照, L9, P5
  `N_ρ(n, O)` の内側の和は、活性で `obj_ρ(o) = O` である名前 `o` を渡り、`B_ρ(n, p)[o]` を足す。名前 `o`
  は、それを `identity` とする leaf のスロットが属する別名類 `C(o)` を 1 つ決める -- L9 より `o` 自身が
  スロットであり、L12 と L13 より そのスロットと `o` を `identity` とする leaf のスロットは同じ ρ-終端を
  持つ。P5 (a) より `obj(C(o)) = obj_ρ(o) = O` である。この対応で和を並べ替えると
  `bumps_ρ(n, C)` の和になる。

<1>3. `Σ_{C : obj(C) = O} held_ρ(n, C) ≥ N_ρ(n, O) + 1` である。
  BY A19 (ii), <1>1, <1>2
  A19 (ii) より各 `C` について `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C) ≥ 0` である。<1>1 と <1>2 より
  `bumps_ρ(n, C) ≥ 1` である類 `C0` が少なくとも 1 つ在る。その `C0` について
  `held_ρ(n, C0) ≥ bumps_ρ(n, C0) + 1` であり、他の類 `C` について `held_ρ(n, C) ≥ bumps_ρ(n, C)` で
  ある。足し合わせると `Σ_C held_ρ(n, C) ≥ Σ_C bumps_ρ(n, C) + 1 = N_ρ(n, O) + 1` である。

<1>4. QED
  BY A19 (i), <1>3
  `H(O) ≥ Σ_C held_ρ(n, C) ≥ N_ρ(n, O) + 1` である。これが INV(n) であり、7.5.1 より P18a が出る。

#### 7.5.5 A19 (ii) の `borrow_ify` の側

`A19` (ii) が破れるのは、ある類の参照が減って bump の数が減らないときである。`held_ρ(·, C)` を減らす
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

- **(A)** `owns_unit(p, u)` が真であり、`V` の本体に `Retain(p, u)` 節点が在ればその節点は `rewrite_rc`
  に落とされない。
- **(B)** `owns_unit(p, u)` は偽であり、その消費は `App(callee, args)` の引数の位置であって、`call_rc` が
  その呼び出しの直前に `Retain(p, u)` を置く。

**証明**

<1>1. `V` が借用版でないとき、`rewrite_rc` は節点をそのまま返す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  `if !self.is_borrow_version { return rc_node(...); }` である。

<1>2. 以下 `V` が借用版であるとする。`rewrite_rc` が `Retain(p, u)` を落とすのは `owns_unit(p, u)` が
      偽のときに限る。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc
  `units_under(&v.ty, path, type_env)` を `self.owns_unit(v, unit)` で絞る。A2 より `path` は `ty(p)` の
  unit なので `units_under` はその 1 元であり、絞りが落とすのは `owns_unit` が偽のときである。

<1>3. `(p, u)` は `levelled_sites(V の本体, type_env)` の元である。
  BY CODE src/rc_ir/borrow.rs: levelled_sites
  `levelled_sites` は「the target of every reference-count node, and every argument unit of every call」
  を挙げる。`Retain(p, u)` は `V` の本体の reference-count 節点である。

<1>4. CASE 消費が `App` の腕以外 -- `Closure` の capture、`Destructure`、終端の `Ret` -- である。
  <2>1. `collect_consumes` は `(p, μ)` を報告する。
    BY P7, CODE src/rc_ir/ownership.rs: collect_consumes_go
    P7 より D9 の意味で消費する構文はすべて `collect_consumes` が報告する。この 3 種について
    `collect_consumes_go` の判定は `own` を読まないので、報告はどの所有の割り当てでも起きる。
  <2>2. `origin(p, μ)` の候補であるパラメータ leaf はすべて `owned_leaves` に入っている。
    BY P8, <2>1
  <2>3. `origin(p, μ)` の各候補 `(r, q)` について `owns_object(r, q)` は真である。
    BY <2>2, P7e, CODE src/rc_ir/borrow.rs: RewriteCtx::owns_object,
       CODE src/rc_ir/borrow.rs: borrow_ify
    `owns_object` は `r` が この版のパラメータでないとき真を返す。パラメータであるときは
    `units_under(ty(r), q)` の各 unit が `owned_units` に入るかを見る。`borrow_ify` は借用版の
    `owned_units` を、`owned_leaves.owns(&p.name, &leaf)` が真である各 leaf について
    `truncate_to_unit(&p.ty, &leaf, type_env)` を入れて作る。P7e より
    `owns_object(r, q) = owns_object(r, truncate_to_unit(ty(r), q))` であり、<2>2 の
    `owned_leaves` の元 `q` はまさにその unit を `owned_units` に入れている。
  <2>4. QED
    BY <2>3, <1>3, P7a, <1>2
    `μ` は inhabited であり、`u` の下の leaf である (`u = truncate_to_unit(ty(p), μ)` と P1)。よって
    <2>3 は P7a の節 2 である。P7a より `owns_unit(p, u)` は真であり、<1>2 より節点は落ちない。

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
  <2>3. CASE `owns_unit(p, u)` が真である。<1>2 より節点は落ちない。これが (A) である。
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

<1>6. QED
  BY <1>1, <1>2, <1>4, <1>5
  D9 の消費の表の 6 行のうち、`App` の引数の行は <1>5 が扱う。残りの 5 行 -- `App` の callee、
  `Closure` の capture、boxed 容器の `Destructure`、unbox 容器の `Destructure`、終端の `Ret` -- は
  <1>4 が扱う。この 5 行はどれも `collect_consumes_go` が `own` を読まずに報告するからである。
  <1>4 は (A) を、<1>5 は (A) か (B) を与える。`V` が借用版でないときは <1>1 より (A) である。

**この補題が答えるもの。** 依頼された疑い -- 「`rewrite_rc` がアームの `Retain(p)` を落としながら `p` の
消費が残ることがあるか」-- は、当たらない。落ちるのは `owns_unit(p, u)` が偽のときだけであり、それは
`App` の引数の位置に限られる (L16 の (B))。そこでは `call_rc` が消費の直前に同じ `Retain(p, u)` を置くので、
`Retain` の直後に消費が来て `held_ρ(·, C)` はこの節点をまたいで変わらない。(A) の場合は `insert_rc` が
置いた `Retain` がそのまま残る。**どちらでも `A19` (ii) の余りは保たれる。** よって `A19` を果たす者の
うち `borrow_ify` の側は果たされる。

#### 7.5.6 `insert_rc` の側 -- この文書が示せないもの

`A19` (ii) が要求するのは、`insert_rc` の出力の各別名類が、その類のスロットを名指す `Retain` のうち
走査が `pending` に持つものより 1 つ多い参照を持つことである。`insert_rc` はこれを使用回数の勘定で作る --
`Own` オペランドの最後でない使用の前に `Retain` を置き (`CODE src/rc_ir/rc_insert.rs:
RcInserter::insert_into_operation_let` の `retains_before`)、アーム本体の `Ret(x)` の `x` が `Match` の
後でも live であるときそのアームの中に `Retain(x)` を置き (`CODE src/rc_ir/rc_insert.rs:
RcInserter::insert_into_expr_inner` の `RcExpr::Ret(x)` の腕と
`CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live`)、容器と scrutinee についても同じことをする
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure`,
`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match`)。

**これは `insert_rc` についての命題であり、この証明の対象 (`borrow_ify` と `cancel`) の外にある。**
言明の形にすると次のものになる。

> **(P-insert)** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各時点の各別名類
> `C` は、`ρ` の上でその類のスロットを名指す残りの消費と `Release` の個数以上の参照を持つ。

この命題が要るのは P18a、P19、P21 のすべてである。**果たす者を書いた仮定として置くか、`insert_rc` に
ついての命題として証明するかは、この文書の外の判断である。**

#### 7.5.7 反例 `C1` (`A19` (ii) が A1・A2・D12 から出ないことを示す)

`A19` を仮定として置かねばならないこと -- すなわち A1・A2・D12 では P18a が出ないこと -- を、D12 を満たす
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

`A19` (ii) が破れる。`m` と `p` は 1 つの別名類 `C` に属し (アーム本体の `Ret` の辺、D20)、
`held_ρ(·, C)` は `p` の割り当てで 1、`Retain(m, [])` で 2、`App(f, [p])` の消費で 1 になる。
`Retain(m, [])` の要素はそのとき `pending` に在り `bumps_ρ(·, C) = 1` なので、
`held_ρ(·, C) = 1 < 1 + 1` である。P18a も同じ位置で破れる -- `n((m, [])) = 1` に対して `H(O_p) = 1` で
ある。

**`cancel` は対を消す。** `origin(m, [])` は `Binding::Join([p, q])` の腕を取り、候補 `{(p, []), (q, [])}`
が 2 元なので `Join { identity: (m, []), candidates }` である。`ActRefs(m, []) = {(m, []): 1}` なので、
`App(f, [p])` の消費が渡す `acted_on(p, []) = {(p, [])}` は `outstanding` の鍵ではない。よって L15 の
仮定 (`identity(v, λ) ∈ acted_on(w, μ)`) が成り立たず、要素は残る。続く `Release(m, [])` が `un_bump` で
`InBracket` を返し、`cancelled()` が `Retain(m, [])` とその `Release` を返す。消した後は `App(f, [p])` が
`O_p` を解放し、`Eval(m)` が解放済みの `O_p` を読む -- (S-c) 違反である。

**この形は実際の入力には現れない。** `App(f, [p])` は `Match` の後の `p` の使用なので、`insert_rc` は
アーム 0 の `Ret(p)` の前に `Retain(p)` を置く。`C1` の `main` はその `Retain(p)` を持たないので
`insert_rc` の出力ではない。その `Retain(p)` が在れば `held_ρ(·, C)` は 1 大きく、`A19` (ii) が成り立つ。
`borrow_ify` がその `Retain(p)` を落とさないことは L16 が示す。

#### 7.5.8 反例 `C2` (`A19` (ii) の破れ方は `Join` に限らない)

`C1` は `Join` が 1 つのオブジェクトに 2 つ目の名前を与える形である。`A19` (ii) の破れ方はそれだけでは
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

**`A19` (ii) が破れる。** `o` と `y` は別の別名類である -- `y` は `RcRhs::App(..)` に束縛されるので
`Binding::Producer` であり、`origin(y, []) = Origin::Exactly((y, []))` で ρ-歩みを持たない
(`CODE src/rc_ir/ownership.rs: collect_bindings`, `CODE src/rc_ir/ownership.rs: origin_inner`)。
`o` の類を `C_o`、`y` の類を `C_y` と書くと、`held_ρ(·, C_o)` は割り当てで 1、`App(id, [o])` の消費で 0、
`Retain(o, [])` で 1 になる。`Retain(o, [])` の要素が `pending` に在る間 `bumps_ρ(·, C_o) = 1` なので
`held_ρ(·, C_o) = 1 < 2` である。

**`cancel` は対を消す。** `App(f, [y])` の消費が渡す `acted_on(y, []) = {(y, [])}` は
`t.outstanding = {(o, []): 1}` の鍵ではない。よって要素は残り、`Release(o, [])` が `un_bump` で `InBracket` を返し、対が消える。消した後は
`App(f, [y])` の中で `f` の `Release(b, [])` が `O` を解放し、`Eval(o)` が解放済みの `O` を読む。

**`C2` が示すこと。** ずれを作っているのは `Join` ではなく `App` である。`o` の参照は `id` の活性化を
通って `y` に戻り、`origin` は `y` を `Producer` として `Exactly((y, []))` と答えるので、同じ参照が
呼び出しの前と後で別の別名類に属する。`Join` の候補を `pending` の要素に持たせても、この形には
触れない -- `Others(o, [])` は空だからである。**`C2` も実際の入力には現れない** -- `insert_rc` は変数が live な位置にしか `Retain` を
置かず (`CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live`)、消費された変数はその消費より後では
live でないので、`App(id, [o])` の後に `Retain(o, [])` は置かれない。`borrow_ify` の `call_rc` が足す
`Retain` も呼び出しの直前に置かれる (P11)。

### 7.6 README へ差し戻す点 (第 7 節の分)

#### 差し戻し 6 (bump の帰属と「走査中の位置」を定義に置く)

P18b と P18a の言明の `B(p, ρ)` -- 「その `Retain` が `ρ` で実際に作った参照のうち、まだ処分されて
いないもの」-- は、D8 が「同じオブジェクトへの参照どうしは互いに区別されない」と定めるので、処分をどの
`Retain` の分として数えるかを決めないと定まらない。第 7.1 節の DEF bump の帰属がその決め方を書いた。
README にはまだ無い。次の文を README の第 5 節、P18b の言明の直前に置くことを提案する。

> **D27 (bump の帰属)**
> 実行路 `ρ` と、走査中の位置 -- 節点の訪問の入口 -- を固定する。`pending` の各要素 `p` について、
> `VarPath` 上の多重集合 `B(p, ρ)` を、`ρ` に沿った次の規則で定める。
>
> - `p` が `Retain(v, π)` の訪問で `pending` に入るとき、`B(p, ρ)` は、`π` の下の inhabited (D16) かつ
>   計数下 (D26) の各 leaf を `origin` の identity で名付けて数えた多重集合である。P6 より、これは
>   その `Retain` が `ρ` で実際に作る参照の多重集合である。
> - `un_bump` が `InBracket` で `p` を選ぶ `Release(v', π')` の訪問で、`B(p, ρ)` から、その `Release` が
>   `ρ` で実際に処分する参照の多重集合を引く。
> - アームへの複製と `merge` は `B(p, ρ)` をそのまま運ぶ。`consume_objects` が `p` を取り除いたときは
>   定めるものが無い。ほかのどの操作も `B(p, ρ)` を変えない。
>
> D8 は同じオブジェクトへの参照を区別しないので、「その `Retain` が作った参照のうちまだ処分されて
> いないもの」は帰属を決めないと定まらない。**走査自身の帰属を採るのは、削除の可否を決めるのが走査の
> `outstanding` だからである。**「そのオブジェクトのどの処分も引く」と読むと、P18a は 7.5.7 の `C1` でも
> 空虚に成り立ってしまい、P21 を支えない。

あわせて、P18b と P18a が量化する「走査中の各位置」を、**節点の訪問の入口**と定めることを提案する。
走査は 1 つの節点の訪問の中でも `pending` を数回書き換えるので、位置をどちらに取るかで言明の強さが
変わる。第 7 節は入口に取った。

#### 差し戻し 7 (P18a に要る前提を仮定として置く)

第 7.5 節が `A19` として置いた前提は README に無い。P18a は A1・A2・D12 だけからは出ない -- 7.5.7 の
`C1` と 7.5.8 の `C2` が、D12 を満たしながら P18a を破る本体である。次の仮定を README の第 4 節に置く
ことを提案する。文面は第 7.5 節の `A19` のとおりである。

> **A19 (bump の下に余りが在る)** -- 果たす者: `insert_rc` (使用回数の勘定) と `borrow_ify`
> (`rewrite_rc` が落とさないこと、落とす場合に `call_rc` が補うこと)。検査: 無し。

果たす者の 2 人のうち、**`borrow_ify` の側はこの文書が示した** (7.5.5 の `L16`)。`insert_rc` の側は
`insert_rc` についての命題であり、この証明の対象の外にある。7.5.6 がその言明 `(P-insert)` を書き出した。
`A19` は P18a のほか P19 と P21 も読むので、置く場所は README の第 4 節がよい。

#### 差し戻し 8 (`C1` と `C2` を第 8 節に記録するか)

`C1` と `C2` はコードの欠陥ではなく、**言明が A1・A2・D12 より強い前提を必要とすることの証拠**である。
README の第 8 節は「証明を書く作業が見つけたコードの欠陥」を挙げる節なので、この 2 つはそこではなく、
`A19` の根拠として第 4 節に短く引くのがよい。`C1` の形は実際の入力に 11 件あると計測されているが、
その 11 件はどれも `insert_rc` のアームの `Retain` を伴うので `A19` を破らない。
