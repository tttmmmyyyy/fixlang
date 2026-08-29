# P15 - P18: `cancel` の走査

この文書は README の層 3 の 4 命題 P15, P16, P17, P18 を証明する。README の定義 D1 - D19 と仮定
A1 - A14 の上に立つ。層 1 と層 2 の命題は引用しない。

## 0. この文書が使う記法

README の第 2 節の記法に、次の 2 つを加える。

- **局所の定義**。この文書の中だけで使う語を第 1 節と第 5 節で定める。`BY` の行では `DEF <名前>` で引用する。
- **局所の補題**。この文書の中だけで使う補題を `L1` - `L12` と番号を付けて述べ、`BY` の行では
  `L<n>` で引用する。各補題は、それより小さい番号の補題と、その補題より前に置かれた命題と、README の
  D/A だけを引用する。

`CODE` の引用はファイル名と記号の道で書く。1 か所だけこのリポジトリの外のコードを引用する
(`stacker` crate の `maybe_grow`)。

## 1. 局所の定義

### DEF 部分木

D2 の意味での本体の木の位置を**節点**と呼ぶ。節点 `n` の**子**を次で定める。

| `n` の式 | 子 |
|---|---|
| `Let(_, Match(_, arms), k)` | `arms` の各 `arm.body`、および `k` |
| `Let(_, rhs, k)` (`rhs` は `Match` でない) | `k` |
| `Retain(_, _, _, k)` | `k` |
| `Release(_, _, _, k)` | `k` |
| `Destructure(_, _, _, k)` | `k` |
| `Eval(_, k)` | `k` |
| `Ret(_)` | 無し |

節点 `n` の**部分木** `N(n)` を、`n` と、`n` の各子 `c` についての `N(c)` との合併とする。D2 より本体は
有限の木であり、位置が相異なれば節点も相異なるので、相異なる子の部分木は交わらず、`n` はどの子の部分木にも
入らない。

### DEF 継続終端

節点 `n` の**継続終端** `ret(n)` を、`n` から D2 の意味の継続 (`Match` の場合はアーム本体ではなく `k`) を
たどって到達する `Ret` 節点とする。D2 より継続の鎖は有限で `Ret` で終わるので、`ret(n)` は 1 つに定まる。
`ret` が `Ret` 節点に与える値はその節点自身である。

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。呼び出しの時間順を**訪問順序**と呼ぶ。

節点 `n` の訪問における `pending` 引数の値 (その訪問がそれに変更を加える前の値) を `pending(n)` と書き、
**入口状態**と呼ぶ。その訪問の戻り値を `pending_out(n)` と書き、**出口状態**と呼ぶ。

節点 `n` が時点 `τ` までに**訪問された**とは、`n` の訪問がその時点までに始まっていることをいう。節点 `m`
が節点 `n` より**前に訪問された**とは、`m` の訪問が `n` の訪問より前に始まっていることをいう。

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、
`CancelAnalysis` の走査中に次の値を定める。

- `ActRefs(t) :=` `self.acted_references(v, path)` の値、`ActRefs(r) :=` `self.acted_references(v, path)` の値。
- `others(r) :=` `self.other_objects(v, path)` の値。

`CancelAnalysis::acted_references(v, path)` は `ownership::acted_references(self.vars, self.type_env, v, path)`
の値を返す (`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。すなわち `ActRefs(t)` は D15 の
`ActRefs(v, path)` である。`ownership::acted_references` の値は `vars`、`type_env`、`v`、`path` だけから
定まる (`CODE src/rc_ir/ownership.rs: acted_references`)。`origin` は答えを `vars.origins` に記録するが、
記録するのは計算した答えそのものであり、記録の有無は返り値を変えない
(`CODE src/rc_ir/ownership.rs: origin`)。`other_objects` も `self.vars`、`self.type_env`、`v`、`path` だけ
から値が決まる (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。よって上の 3 つの量は、走査の
どの時点で読んでも同じ値である。`self.vars` と `self.type_env` は `CancelAnalysis` の構築のときに置かれ、
走査はこれらを共有参照でしか持たない (`CODE src/rc_ir/borrow.rs: CancelAnalysis`)。

### DEF 参照の多重集合

`References` は `Map<VarPath, usize>` を 1 つ持つ構造体である (`CODE src/rc_ir/ownership.rs: References`)。
これを、鍵をオブジェクトの名前、値をその個数とする多重集合とみなす。次の記法を使う。

- `R2 ⊆ R1` とは、各オブジェクトについて `R2` の個数が `R1` の個数以下であることをいう。
- `R1 - R2` とは、各オブジェクトの個数の差である (`R2 ⊆ R1` のときだけ書く)。
- **空**とは、参照を 1 つも持たないことをいう。
- 2 つの `References` の値が**等しい**とは、`PartialEq` が真を返すこと、すなわち各オブジェクトの個数が
  一致することをいう。`References` は `PartialEq` を derive し、その中身は `Map` (`FxHashMap`) なので、
  等しさは鍵と値の対の集合の一致である (`CODE src/rc_ir/ownership.rs: References`,
  `CODE src/misc.rs: Map`)。

`⊆` は推移的である (各オブジェクトの個数についての不等式の推移律)。

### DEF 割り当て

`Arc::new` は 1 つの**割り当て**を作る。その `Arc` またはその複製が 1 つでも生きている間、その割り当ては
生きている。生きている 2 つの相異なる割り当ては記憶域を共有しないので、それらの先頭アドレスは相異なる。

### DEF 基本操作

走査が `PendingRetains` の値を作る操作に、次の 6 つの名を与える。この 6 つで尽きることは L8 が示す。

| 名 | 作られ方 |
|---|---|
| 初期 | `cancel` の `cancel_body` の `analysis.walk(body, PendingRetains::default(), true)` の第 2 引数 |
| 複製 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `pending.clone()` |
| 追加 | `RcExpr::Retain(v, path, _, k)` の腕の `pending.push(PendingRetain { node, outstanding })` |
| 消費 | `CancelAnalysis::consume_objects` の `pending.retain(...)` |
| 引き | `RcExpr::Release(v, path, _, k)` の腕の `un_bump(&mut pending, &un_bumped)` |
| 併合 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `self.merge(&pending, &arm_exits)` |

(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/borrow.rs: un_bump`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)

走査が作る `PendingRetains` の値を**状態**と呼ぶ。「追加」「消費」「引き」は直前の状態をその場で書き換え、
「複製」と「併合」は新しい値を作る。状態には**生成順序** (走査がそれを作る時間順) があり、初期状態を除く各
状態は、それより前に作られた 1 つ以上の状態から上の操作 1 つで作られる。走査は有限回で終わるので、
生成順序は有限の全順序である。

### DEF 除去事象

基本操作 1 つが、状態の集まり `P1, ..., Pm` を入力として状態 `P'` を作り、ある `Pi` に `node` が `x` で
ある要素があり、`P'` にはそれが無いとき、この操作を `x` の**除去事象**と呼ぶ。「併合」の入力は
`pending_in` と各 `arm_exits[j]` の全部であり、ほかの操作の入力は 1 つである。「追加」「消費」「引き」は
状態をその場で書き換えるので、入力は書き換えの前の値、`P'` は後の値である。

## 2. 予備の補題

### L1 (`grow_stack` は閉包を 1 回呼ぶ)

`grow_stack(f)` は `f` をちょうど 1 回呼び、その値を返す。よって
`CancelAnalysis::walk(node, pending, returns_from_func)` の 1 回の呼び出しは
`CancelAnalysis::walk_inner(node, pending, returns_from_func)` をちょうど 1 回呼んでその値を返し、
`RewriteCtx::rewrite(node)` の 1 回の呼び出しは `RewriteCtx::rewrite_inner(node)` をちょうど 1 回呼んで
その値を返す。

**証明**

<1>1. `grow_stack(f)` の本体は `stacker::maybe_grow(64 * 1024, 1024 * 1024, f)` である。
  BY CODE src/misc.rs: grow_stack
<1>2. `maybe_grow(red_zone, stack_size, callback)` は、`enough_space` が真のとき `callback()` を、偽の
      とき `grow(stack_size, callback)` を評価し、その値を返す。どちらの枝も `callback` をちょうど 1 回
      呼ぶ (`callback` の型は `FnOnce` なので 2 回は呼べない)。
  BY CODE stacker-0.1.23/src/lib.rs: maybe_grow
<1>3. `walk` の本体は `grow_stack(|| self.walk_inner(node, pending, returns_from_func))` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk
<1>4. `rewrite` の本体は `grow_stack(|| self.rewrite_inner(node))` である。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L2 (`References` の表現)

`CancelAnalysis` の走査が扱う `References` の値は、どのオブジェクトについても個数が 1 以上である。よって
この 4 つが成り立つ。

1. `R.is_empty()` が真であることと `R` が空 (DEF 参照の多重集合) であることは同値である。
2. `R1.covers(&R2)` が真であることと `R2 ⊆ R1` であることは同値である。
3. `R1.shares_an_object(&R2)` が真であることと、`R1` と `R2` の双方が参照を持つオブジェクトが在ることは
   同値である。また `R1.names(o)` が真であることと、`R1` が `o` の参照を持つことは同値である。
   `R1.objects()` は `R1` が参照を持つオブジェクトをちょうど 1 度ずつ並べた列である。
4. `R2 ⊆ R1` のとき `R1.subtract(&R2)` は `R1` を `R1 - R2` に書き換え、panic しない。また
   `R1 - R2 ⊆ R1` である。

**証明**

<1>1. `ownership::acted_references(vars, type_env, v, path)` が返す `References` の各鍵の値は 1 以上で
      ある。鍵の値が増えるのは `*references.entry(object).or_default() += 1` の 1 か所だけであり、鍵は
      その場で作られるので、値が 0 の鍵は残らない。
  BY CODE src/rc_ir/ownership.rs: acted_references
<1>2. `CancelAnalysis::acted_references(v, path)` は `ownership::acted_references` の値を返す。ただし
      その値が空のときは `assert!` が発火してコンパイラが停止する。よってこの関数が値を返すとき、その値は
      空でなく、各鍵の値は 1 以上である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references, <1>1
<1>3. `References::subtract(other)` は `other` の各鍵について自分の値をその分だけ減らし、0 になった鍵を
      取り除く。よって、各鍵の値が 1 以上の `References` を `subtract` した結果も、各鍵の値が 1 以上で
      あり、その値は各オブジェクトの個数を減らしたものである。
  BY CODE src/rc_ir/ownership.rs: References::subtract
<1>4. `References::covers(other)` は、`other` のどの鍵についても、自分がその鍵を持ちその値以上であることを
      言う。各鍵の値が 1 以上のとき、これは各オブジェクトの個数の不等式、すなわち `other ⊆ self` と同値で
      ある (`other` が持たないオブジェクトの個数は 0 で、不等式は自動的に成り立つ)。よって 2 が成り立つ。
      また `covers` が真のとき、<1>3 の `get_mut` の `expect` は発火せず、引き算の結果は `self - other`
      である。`R1 - R2` の各オブジェクトの個数は `R1` のそれ以下なので `R1 - R2 ⊆ R1` である。よって
      4 が成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::covers, CODE src/rc_ir/ownership.rs: References::subtract,
     <1>3
<1>5. `References::is_empty()` は内側の `Map` が空であることを言う。各鍵の値が 1 以上の `References` に
      ついて、これは参照を 1 つも持たないことと同値である。よって 1 が成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::is_empty, <1>1, <1>3
<1>6. `References::shares_an_object(other)` は `other` の鍵のいずれかが自分の鍵であることを言い、
      `References::names(object)` は `object` が自分の鍵であることを言い、`References::objects()` は
      自分の鍵を 1 度ずつ並べた列を返す。各鍵の値が 1 以上の `References` について、鍵であることと
      その参照を 1 つ以上持つことは同値である。よって 3 が成り立つ。
  BY CODE src/rc_ir/ownership.rs: References::shares_an_object,
     CODE src/rc_ir/ownership.rs: References::names, CODE src/rc_ir/ownership.rs: References::objects,
     <1>1, <1>3
<1>7. 走査が扱う `References` の値は、`CancelAnalysis::acted_references` が返したもの、それを `subtract`
      で減らしたもの、およびそれらの複製だけである。
  BY CODE src/rc_ir/ownership.rs: References (フィールドは非公開なので `ownership` の外では作れない),
     CODE src/rc_ir/ownership.rs: acted_references (`References(references)` がこのモジュールでの唯一の
     構築点), CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
     (`outstanding` は `self.acted_references` の値), CODE src/rc_ir/borrow.rs: un_bump
     (`innermost.outstanding.subtract(un_bumped)`), CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
     (`outstanding.clone()`)
<1>8. QED
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

### L3 (走査する本体は `RewriteCtx::rewrite` の出力である)

`cancel` が `cancel_body` に渡す本体 --- `prog.funcs` の各 `f` の `f.body` と `prog.globals` の各 `g` の
`g.init` --- は、いずれも `RewriteCtx::rewrite` の 1 回の呼び出しが返した木である。

**証明**

<1>1. `cancel` を呼ぶのは `optimize_rc_program` の 1 か所だけであり、その第 1 引数 `prog` は直前の文
      `prog = borrow_ify(&prog, type_env, config.develop_mode)` の値である。間にあるのは
      `validate(&prog, "after borrow_ify")` の呼び出しだけであり、`validate` は `prog` を共有参照で
      受け取るので `prog` を変えない。
  BY CODE src/build/build_object_files.rs: optimize_rc_program, CODE src/rc_ir/validate.rs: validate
<1>2. `borrow_ify` が返す `RcProgram` の `funcs` の各値の `body` は `ctx.rewrite(&f_own.body)` または
      `ctx.rewrite(&clone.body)` の値であり、`globals` の各値の `init` は `ctx.rewrite(&g.init)` の値で
      ある。返す直前に走る `for func in funcs.values_mut()` のループは `borrowed_units` だけを書き換える。
  BY CODE src/rc_ir/borrow.rs: borrow_ify
<1>3. `cancel` は `prog.funcs.values()` の各 `f` について `cancel_body(&vars, &f.body)` を呼び、
      `prog.globals` の各 `g` について `cancel_body(&vars, &g.init)` を呼ぶ。ほかに `cancel_body` を
      呼ばない。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>4. QED
  BY <1>1, <1>2, <1>3

### L4 (走査する本体の `Match` はアームを 1 つ以上持つ)

`cancel` が走査する本体のすべての `Match` は、1 つ以上のアームを持つ。

**証明**

<1>1. `RewriteCtx::rewrite_inner(node)` が返す木の各 `Match` は、`node` の木のある `Match` から、
      アームの本体だけを差し替えて作られたものである。アームの個数は等しい。
  <2>1. `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕は、新しいアームの列を
        `arms.iter().map(|arm| arm.with_body(self.rewrite(&arm.body))).collect()` で作る。
        `Iterator::map` と `collect` は要素数を保ち、`MatchArm::with_body` は `body` だけを差し替えた
        `MatchArm` を返す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body
  <2>2. ほかの腕は `RcRhs::Match` を作らない。`RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕が作る
        右辺は `RcRhs::App` であり、`RcExpr::Let(x, rhs, k)` の腕は `rhs.clone()` をそのまま運ぶ。
        `match` の腕はこの順に並んでいるので、この第 3 の腕に落ちる `rhs` は `RcRhs::Match` ではなく、
        `RcRhs` の残る 4 変位 (`Var`, `App`, `Closure`, `Llvm`) はアームを持たない。`rc_node`、
        `prepend_rc` が作る式は `Retain` と `Release` であり、`expr_node` はほかの腕で `Let` (右辺は
        `App` または複製した `rhs`)、`Destructure`、`Eval`、`Ret` を作る。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: rc_node,
       CODE src/rc_ir/borrow.rs: prepend_rc, CODE src/rc_ir/ast.rs: RcRhs
  <2>3. QED
    `rewrite_inner` は各子について `rewrite` を呼び、L1 よりそれは `rewrite_inner` を 1 回呼ぶ。木の
    構造についての帰納法 (DEF 部分木より整礎) と <2>1、<2>2 より、返る木の `Match` は入力の木の `Match`
    と 1 対 1 に対応し、アームの個数が等しい。
    BY <2>1, <2>2, L1, DEF 部分木
<1>2. `borrow_ify` が `rewrite` に渡す本体は、入力プログラムの関数の本体 `f_own.body`、その複製
      `clone.body`、または入力プログラムのグローバル初期化子 `g.init` である。`f_own` は
      `func.clone()` であり、`clone` は `clone_func` の値である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify
<1>3. `clone_func` が作る本体は `fresh_rename_function` が返す `body` であり、それは
      `rename_expr(body, &renaming)` の値である。`rename_expr_inner` は各 `Match` を、`rename_rhs` の
      `RcRhs::Match(scrut, arms)` の腕が `arms.iter().map(...).collect()` で作ったアームの列を持つ
      `Match` に写す。この写像は要素数を保つので、アームの個数は等しい。
  BY CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: fresh_rename_function,
     CODE src/rc_ir/rename.rs: rename_expr_inner, CODE src/rc_ir/rename.rs: rename_rhs
<1>4. `borrow_ify` の入力プログラムのすべての `Match` は 1 つ以上のアームを持つ。
  BY A9
<1>5. QED
  L3 より `cancel` が走査する本体は `rewrite` の出力である。<1>2 よりその入力は `borrow_ify` の入力
  プログラムの本体か、その複製である。<1>3 より複製はアームの個数を保ち、<1>1 より `rewrite` も保つ。
  <1>4 より元の個数は 1 以上である。
  BY L3, <1>1, <1>2, <1>3, <1>4

## 3. P15 (節点と `NodeId`)

**言明** --- `cancel` の入力すなわち `borrow_ify` の出力の各本体について、相異なる位置は相異なる `NodeId`
を持つ。また `CancelAnalysis::walk` は本体の各位置をちょうど 1 回訪れる。

前半は、`cancel` が走査する本体、すなわち `cancel` の入力プログラムの各関数の `body` と各グローバル
初期化子の `init` について示す。D2 が述べるとおり `RcExprNode` は式を `Arc` で共有できるので、これは
`RcExprNode` 一般の性質ではなく、`cancel` に渡される木の性質である。

### 3.1 前半 (相異なる位置は相異なる `NodeId` を持つ)

<1>1. `node_id(node)` は `node.expr` が指す `RcExpr` の割り当てのアドレスである。
  BY CODE src/rc_ir/borrow.rs: node_id, CODE src/rc_ir/ast.rs: RcExprNode
<1>2. 同時に生きている相異なる割り当てはアドレスが相異なる。
  BY DEF 割り当て
<1>3. `RewriteCtx::rewrite(node)` の 1 回の呼び出しが返す木の各位置は、その呼び出しの間に `expr_node` が
      `Arc::new` で作った割り当てを持ち、相異なる位置は相異なる割り当てを持つ。さらに、それらの割り当ては
      その呼び出しが返るまで 1 つも解放されない。
  <2>1. `expr_node(expr, source)` は `Arc::new(expr)` を 1 つ作り、それを `expr` フィールドに持つ
        `RcExprNode` を返す。
    BY CODE src/rc_ir/borrow.rs: expr_node
  <2>2. `rc_node(is_release, var, path, state, k, source)` は `expr_node` を 1 回呼んでその値を返す。
        `prepend_rc(units, is_release, k)` は `units` を逆順にたたみ込み、`units` の要素ごとに `rc_node`
        を 1 回呼ぶ。`units` が空なら `k` をそのまま返す。
    BY CODE src/rc_ir/borrow.rs: rc_node, CODE src/rc_ir/borrow.rs: prepend_rc, <2>1
  <2>3. `RewriteCtx::rewrite_rc(v, path, state, is_release, k, source)` は `self.rewrite(k)` を 1 回呼び、
        その値の上に `rc_node` で 0 個以上の節点を積んだものを返す。ほかに `RcExprNode` を作らない。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, <2>2
  <2>4. `rewrite_inner(node)` の 8 つの腕はいずれも、`self.rewrite` を `node` の各子についてちょうど
        1 回ずつ呼び、その戻り値の上に `expr_node` / `rc_node` / `prepend_rc` で有限個の節点を積んだ木を
        返す。ほかに `RcExprNode` を作らない。
    <3>1. `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕は `self.rewrite(k)` を 1 回呼び、その上に
          `prepend_rc(after, true, ...)`、`expr_node(RcExpr::Let(...))`、`prepend_rc(before, false, ...)`
          で節点を積む。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::App(callee, args), k)` の腕
    <3>2. `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕は、`arms` の各 `arm` について
          `self.rewrite(&arm.body)` を 1 回ずつ呼び、`self.rewrite(k)` を 1 回呼び、`expr_node` で
          1 節点を積む。`arm.with_body(body)` は `body` をそのアームの本体に据えたアームを返す。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
         CODE src/rc_ir/ast.rs: MatchArm::with_body
    <3>3. `RcExpr::Let(x, rhs, k)` の腕は `self.rewrite(k)` を 1 回呼び、`expr_node` で 1 節点を積む。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, rhs, k)` の腕
    <3>4. `RcExpr::Retain(v, path, state, k)` の腕と `RcExpr::Release(v, path, state, k)` の腕は
          `self.rewrite_rc` を 1 回呼び、その値を返す。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Retain(v, path, state, k)` の腕,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Release(v, path, state, k)` の腕,
         <2>3
    <3>5. `RcExpr::Destructure(container, fields, state, k)` の腕と `RcExpr::Eval(v, k)` の腕は
          `self.rewrite(k)` を 1 回呼び、`expr_node` で 1 節点を積む。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Eval(v, k)` の腕
    <3>6. `RcExpr::Ret(v)` の腕は `expr_node` で 1 節点を作って返す。この節点に子は無い。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Ret(v)` の腕, DEF 部分木
    <3>7. QED
      `RcExpr` の変位は `Let`, `Retain`, `Release`, `Destructure`, `Eval`, `Ret` の 6 つで、`RcRhs` の
      変位は `Var`, `App`, `Closure`, `Llvm`, `Match` の 5 つである。<3>1 から <3>6 は
      `Let` (右辺が `App`)、`Let` (右辺が `Match`)、`Let` (右辺がそれ以外)、`Retain`、`Release`、
      `Destructure`、`Eval`、`Ret` の 8 つを尽くし、これは `rewrite_inner` の `match` の 8 つの腕である。
      BY <3>1, <3>2, <3>3, <3>4, <3>5, <3>6, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
  <2>5. `rewrite` の 1 回の呼び出しが作る `Arc<RcExpr>` は、その呼び出しが返るまで 1 つも解放されない。
    <3>1. <2>4 の各腕が作った `RcExprNode` は、直後にその上に積む節点の継続として、または戻り値として
          保持される。どの腕も、作った `RcExprNode` を捨てない。
      BY <2>4, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
    <3>2. `arm.with_body(body)` の中の `self.clone()` は入力側のアームの本体の `Arc` を複製し、その複製は
          `MatchArm` の構築で `body` に置き換えられて捨てられる。捨てられるのは入力の木の節点への参照で
          あり、入力の木は `rewrite` の引数として借用されている間ずっと生きているので、解放は起きない。
      BY CODE src/rc_ir/ast.rs: MatchArm::with_body,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕
    <3>3. QED
      BY <3>1, <3>2
  <2>6. QED
    木 `N(node)` の構造についての帰納法で示す。DEF 部分木より子は真の部分木なので、この帰納法は整礎で
    ある。帰納法の仮定より、各子について `rewrite` が返す木の相異なる位置は相異なる割り当てを持ち、それらは
    その子の呼び出しが返るまで解放されない。<2>5 よりそれらは `rewrite(node)` が返るまで解放されない。
    <2>4 より `rewrite_inner(node)` が返す木は、各子について `rewrite` が返した木と、新しく `expr_node`
    で作った有限個の節点からなる。相異なる子の木は相異なる呼び出しの中で作られ、<2>5 よりそれらは同時に
    生きているので、<1>2 より全体として相異なる割り当てを持つ。新しく作った節点は `Arc::new` の新しい
    割り当てなので、そのとき生きている他のどの割り当てとも相異なる。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <1>2, L1, DEF 部分木
<1>4. `cancel_body(vars, body)` の実行中、`body` の木の割り当てはすべて生きている。
  <2>1. `cancel` の `funcs` を作る閉包は、`prog.funcs.values()` の各 `f` について
        `clone.body = cancel_body(&vars, &f.body)` を実行する。`cancel_body` の実行中、`f` は `prog` から
        借用されており、`f.body` の木は `f` から到達可能である。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>2. `cancel` の `globals` を作る閉包は、`prog.globals` の各 `g` について
        `cancel_body(&vars, &g.init)` を実行する。`cancel_body` の実行中、`g` は `prog` から借用されて
        おり、`g.init` の木は `g` から到達可能である。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>3. QED
    BY <2>1, <2>2
<1>5. QED
  L3 より、`cancel` が走査する各本体は `RewriteCtx::rewrite` の 1 回の呼び出しが返した木である。<1>3 より
  その木の相異なる位置は相異なる割り当てを持ち、<1>4 よりそれらは走査の間ずっと生きている。<1>1 と <1>2 より
  それらの `NodeId` は相異なる。
  BY L3, <1>1, <1>2, <1>3, <1>4

### 3.2 後半 (`walk` は各位置をちょうど 1 回訪れる)

<1>1. `CancelAnalysis::walk` を呼ぶのは、`cancel` の中の
      `analysis.walk(body, PendingRetains::default(), true)` と、`walk_inner` の中の 7 か所だけである。
      その 7 か所は、`Retain` の腕、`Release` の腕、`Match` の腕の 2 か所 (アームと継続)、
      右辺が `Match` でない `Let` の腕、`Destructure` の腕、`Eval` の腕である。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
<1>2. `walk_inner` が呼ぶ `walk` 以外の関数 --- `node_id`, `CancelAnalysis::acted_references`,
      `CancelAnalysis::other_objects`, `CancelAnalysis::consume_objects`, `CancelAnalysis::consume`,
      `CancelAnalysis::consume_rhs`, `CancelAnalysis::merge`, `un_bump`, `destructure_consumes` ---
      はどれも `walk` を呼ばない。
  <2>1. `walk` は `CancelAnalysis` の非公開のメソッドであり、`CancelAnalysis` は `borrow.rs` の非公開の型
        である。よって `ownership.rs` の関数 (`destructure_consumes`) はこれを呼べない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis, CODE src/rc_ir/ownership.rs: destructure_consumes
  <2>2. 残る 8 つの本体はどれも `self.walk` も `walk` も含まない。
    BY CODE src/rc_ir/borrow.rs: node_id, CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, CODE src/rc_ir/borrow.rs: un_bump
  <2>3. QED
    BY <1>1, <2>1, <2>2
<1>3. 任意の節点 `n`、任意の `pending`、任意の `returns_from_func` について、
      `walk(n, pending, returns_from_func)` の 1 回の呼び出しの実行中、`N(n)` の各節点はちょうど 1 回
      訪問され、`N(n)` の外の節点は訪問されない。
  木 `N(n)` の構造についての帰納法で示す。DEF 部分木より子は真の部分木なので、この帰納法は整礎である。
  <2>1. 帰納法の仮定: `n` の各子 `c` と任意の引数について、`walk(c, ・, ・)` の 1 回の呼び出しの実行中、
        `N(c)` の各節点はちょうど 1 回訪問され、`N(c)` の外の節点は訪問されない。
    BY 帰納法の仮定
  <2>2. `walk(n, pending, returns_from_func)` は `walk_inner` を、`node` 引数を `n` としてちょうど
        1 回呼ぶ。すなわちこの呼び出しは `n` をちょうど 1 回訪問する。
    BY L1
  <2>3. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。
      BY DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <3>1, <3>2
  <2>4. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。
      BY DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <3>1, <3>2
  <2>5. CASE `n` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。
    <3>1. この腕は `arms.iter().map(|arm| self.walk(&arm.body, pending.clone(), false)).collect()` で
          各 `arm` について `self.walk(&arm.body, ・, ・)` を呼び、その後
          `self.walk(k, merged, returns_from_func)` を 1 回呼ぶ。ほかに `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
         <1>2
    <3>2. `Iterator::map` の閉包は `collect` によって各要素についてちょうど 1 回、先頭から順に呼ばれる。
          よって <3>1 の `self.walk(&arm.body, ・, ・)` は `arms` の各要素についてちょうど 1 回である。
      BY DEF `Iterator::map` と `collect` (Rust 標準ライブラリの規約)
    <3>3. `n` の子は `arms` の各 `arm.body` と `k` であり、`N(n)` は `{n}` とそれらの部分木の非交和で
          ある。
      BY DEF 部分木
    <3>4. QED
      BY <2>1, <2>2, <3>1, <3>2, <3>3
  <2>6. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。
    <3>1. この腕は `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び、その後
          `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。ほかに `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。
      BY DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <3>1, <3>2
  <2>7. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。
    <3>1. この腕は `destructure_consumes(container, fields, self.type_env)` の各要素について
          `self.consume` を呼び、その後 `self.walk(k, pending, returns_from_func)` を 1 回呼ぶ。ほかに
          `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
         <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。
      BY DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <3>1, <3>2
  <2>8. CASE `n` の式が `RcExpr::Eval(_, k)` である。
    <3>1. この腕は `self.walk(k, pending, returns_from_func)` を 1 回呼び、ほかに `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕, <1>2
    <3>2. `n` の子は `k` だけであり、`N(n)` は `{n}` と `N(k)` の非交和である。
      BY DEF 部分木
    <3>3. QED
      BY <2>1, <2>2, <3>1, <3>2
  <2>9. CASE `n` の式が `RcExpr::Ret(_)` である。
    <3>1. この腕は `walk` を呼ばない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, <1>2
    <3>2. `n` に子は無く、`N(n)` は `{n}` である。
      BY DEF 部分木
    <3>3. QED
      BY <2>2, <3>1, <3>2
  <2>10. QED
    `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を、<2>3 から <2>9 が尽くす。
    `walk_inner` の `match` の腕もこの 7 つであり、`Let` の 2 つの腕はこの順に並んでいるので、右辺が
    `Match` の `Let` は第 1 の腕に、それ以外の `Let` は第 2 の腕に入る。
    BY <2>3, <2>4, <2>5, <2>6, <2>7, <2>8, <2>9, CODE src/rc_ir/ast.rs: RcExpr,
       CODE src/rc_ir/ast.rs: RcRhs, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
<1>4. QED
  <1>3 を `n` を本体の根として適用する。`cancel_body` は `analysis.walk(body, ・, ・)` を 1 回だけ呼ぶ。
  BY <1>3, CODE src/rc_ir/borrow.rs: cancel

## 4. 基本操作の補題

### L5 (`un_bump` の作用)

要素 `e` が `References` の値 `R` と**オブジェクトを共有する**とは、`e.outstanding.shares_an_object(R)`
が真であること、すなわち L2 の 3 より `e.outstanding` と `R` の双方が参照を持つオブジェクトが在ることを
いう。

`un_bump(pending, un_bumped)` の 1 回の呼び出しについて、次の 3 つが成り立ち、この 3 つは場合を尽くす。

1. `un_bumped` とオブジェクトを共有する要素が `pending` に無いとき、返り値は `NoBracket` であり、
   `pending` は変わらない。
2. あるとき、そのような要素の添字のうち最大のものを `i` とする。`pending[i].outstanding.covers(un_bumped)`
   が偽のとき、返り値は `OutsideBracket` であり、`pending` は変わらない。
3. 真のとき、返り値は `InBracket(pending[i].node)` である。`pending[i].outstanding` は
   `pending[i].outstanding - un_bumped` になり、それが空なら `pending[i]` が取り除かれる。ほかの要素の
   `node` と `outstanding` は変わらず、要素の相対順序も変わらない。

**証明**

<1>1. `pending.iter().rposition(|retain| retain.outstanding.shares_an_object(un_bumped))` は、述語を
      満たす要素の添字のうち最大のものを `Some` で返し、そのような要素が無ければ `None` を返す。
      `let Some(index) = ... else { return UnBump::NoBracket; };` により、`None` のとき `NoBracket` を
      返す。この文までに `pending` を変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump,
     DEF `Iterator::rposition` (Rust 標準ライブラリの規約: 後ろから探して最初に述語を満たす要素の添字を
     返す)
<1>2. `let innermost = &mut pending[index];` は添字 `index` の要素への可変参照である。
      `if !innermost.outstanding.covers(un_bumped) { return UnBump::OutsideBracket; }` により、`covers`
      が偽のとき `OutsideBracket` を返す。この文までに `pending` を変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>1
<1>3. `covers` が真のとき、`innermost.outstanding.subtract(un_bumped)` は panic せず、
      `innermost.outstanding` を `pending[index].outstanding - un_bumped` に書き換える。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2
<1>4. `let retain = innermost.node;` は <1>3 の後に実行されるが、`subtract` は `outstanding` しか変え
      ないので、`retain` は書き換え前後で同じ `pending[index].node` である。
  BY CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: References::subtract,
     CODE src/rc_ir/borrow.rs: PendingRetain
<1>5. `if innermost.outstanding.is_empty() { pending.remove(index); }` は、L2 の 1 より <1>3 の差が空の
      ときちょうど添字 `index` の要素を取り除き、空でないとき何もしない。`Vec::remove` は後続の要素を
      1 つずつ前へ詰めるだけなので、残る要素の値と相対順序は変わらない。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2, DEF `Vec::remove` (Rust 標準ライブラリの規約)
<1>6. `UnBump::InBracket(retain)` を返す。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>4
<1>7. <1>1 から <1>6 の間に `pending` に触れるのは <1>3 と <1>5 だけであり、どちらも添字 `index` の
      要素にしか触れない。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>3, <1>5
<1>8. QED
  場合分けは「共有する要素が無い」「あって `covers` が偽」「あって `covers` が真」であり、尽くしている。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7

### L6 (消費の作用)

`CancelAnalysis::consume_objects(pending, objects)` が `pending` に対して行うのは次のことである。
`objects` のいずれかについて `outstanding.names` が真である要素をすべて取り除き、取り除いた各要素の
`node` を `self.needed_retains` に入れる。残る要素の値と並びは変わらず、残る要素の `node` はこの呼び出しで
`needed_retains` に入らない。

さらに、`CancelAnalysis::consume`、`CancelAnalysis::consume_rhs`、および `walk_inner` の
`RcExpr::Destructure(container, fields, _state, k)` の腕が `pending` に対して行うのは、
`consume_objects` の呼び出しだけである。

**証明**

<1>1. `consume_objects` の本体は `pending.retain(|retain| { if objects.iter().any(|object|
      retain.outstanding.names(object)) { self.needed_retains.insert(retain.node); return false; } true })`
      である。`Vec::retain` は閉包が偽を返した要素を取り除き、残る要素の値と相対順序を保つ。閉包が
      `self.needed_retains` に入れるのは偽を返す枝でだけであり、その枝を通るのは
      `objects.iter().any(...)` が真の要素である。この関数はほかに `pending` にも `needed_retains` にも
      触れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     DEF `Vec::retain` (Rust 標準ライブラリの規約)
<1>2. `consume(pending, var, path)` は `origin(self.vars, self.type_env, var, path).acted_on()` から
      `objects` を作り、`self.consume_objects(pending, &objects)` を 1 回呼ぶ。ほかに `pending` に
      触れない。`origin` と `Origin::acted_on` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, CODE src/rc_ir/ownership.rs: origin,
     CODE src/rc_ir/ownership.rs: Origin::acted_on
<1>3. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` を呼んで `consumed` を集め、その各要素
      `(var, leaf)` について `self.consume(pending, &var, &leaf)` を呼ぶ。ほかに `pending` に触れない。
      `rhs_consumes` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/ownership.rs: rhs_consumes
<1>4. `walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕は、
      `destructure_consumes(container, fields, self.type_env)` の各 `leaf` について
      `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。ほかに `pending` に触れない。
      `destructure_consumes` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     CODE src/rc_ir/ownership.rs: destructure_consumes
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L7 (`merge` の作用)

`self.merge(pending_in, arm_exits)` の 1 回の呼び出しについて、各 `arm_exits[j]` の相異なる要素は相異なる
`node` を持つとする。このとき次が成り立つ。

1. `arm_states[j]` は `arm_exits[j]` の各要素 `e` を `e.node` から `&e.outstanding` へ写す表であり、
   その鍵の集合は `arm_exits[j]` の要素の `node` の集合に等しい。
2. `entered_with` は `pending_in` の要素の `node` の集合である。
3. `NodeId` の値 `x` について、二重ループの中で `x` を `retain` とする反復の `is_uniform` の値は、その
   反復がどの `j` のものであっても等しい。その値が真であることは、次の条件 `U(x)` と同値である。
   **`x` が `entered_with` の要素であり、すべての `j'` について `arm_states[j']` が `x` を鍵に持ち、
   それらの値が互いに等しい。**
4. 呼び出しの終わりに `uniform` が `x` を鍵に持つことは、「ある `j` について `arm_states[j]` が `x` を鍵に
   持ち、かつ `U(x)`」と同値である。このとき `uniform[x]` は、その共通の値と等しい `References` である。
5. この呼び出しが `self.needed_retains` に入れる `NodeId` は、「ある `j` について `arm_states[j]` が鍵に
   持ち、かつ `U(x)` が成り立たない」もの全部である。
6. 返り値は、`pending_in` の要素のうち `node` を `uniform` が鍵に持つものを、その並びのまま、`node` は
   そのまま、`outstanding` を `uniform[node]` と等しい値に差し替えて並べた `Vec` である。ほかの要素を
   持たない。

**証明**

<1>1. `arm_states` は
      `arm_exits.iter().map(|exit| exit.iter().map(|retain| (retain.node, &retain.outstanding)).collect()).collect()`
      である。`exit.iter()` は `arm_exits[j]` のすべての要素を渡すので、`arm_states[j]` の鍵の集合は
      `arm_exits[j]` の要素の `node` の集合であり、仮定より 2 つの要素が 1 つの鍵に落ちることはない。
      よって 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, 本補題の仮定
<1>2. `entered_with` は `pending_in.iter().map(|retain| retain.node).collect()` である。よって 2 が
      成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>3. 3 が成り立つ。
  <2>1. 反復は `for states in &arm_states { for (&retain, &outstanding) in states { ... } }` であり、
        `outstanding` は `states` すなわちある `arm_states[j]` の `retain` の値である。`is_uniform` は
        `entered_with.contains(&retain)` と
        `arm_states.iter().all(|other| other.get(&retain) == Some(&outstanding))` の連言である。
        この等式は `References` の `PartialEq` による値の比較である (DEF 参照の多重集合)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, DEF 参照の多重集合
  <2>2. 第 1 の連言肢 `entered_with.contains(&retain)` は `retain` だけで決まる。
    BY <2>1
  <2>3. CASE すべての `j'` について `arm_states[j']` が `retain` を鍵に持ち、その値が互いに等しい。
        このとき、`retain` を鍵に持つどの `arm_states[j]` の反復から見ても、`outstanding` はその共通の
        値と等しいので、第 2 の連言肢は真である。
    BY <2>1
  <2>4. CASE ある `j'` について `arm_states[j']` が `retain` を鍵に持たない。このとき
        `other.get(&retain)` は `None` であり、`Some(&outstanding)` と等しくないので、`retain` を鍵に
        持つどの `arm_states[j]` の反復から見ても第 2 の連言肢は偽である。
    BY <2>1
  <2>5. CASE すべての `j'` が `retain` を鍵に持つが、2 つの `j'` の値が互いに等しくない。このとき、
        `retain` を鍵に持つどの `arm_states[j]` の反復についても、その `outstanding` と等しくない値を
        持つ `j'` が在るので (すべての `j'` の値が `arm_states[j]` の値と等しければ、それらは互いに
        等しい)、`all` は偽であり、第 2 の連言肢は偽である。
    BY <2>1
  <2>6. QED
    <2>3、<2>4、<2>5 は、`retain` を鍵に持つ `arm_states[j']` の有無と値の一致について場合を尽くす。
    3 つのどの場合でも第 2 の連言肢の値は `j` によらず、それが真であるのは <2>3 の場合に限る。<2>2 と
    合わせて、`is_uniform` の値は `j` によらず、真であることは `U(retain)` と同値である。
    BY <2>1, <2>2, <2>3, <2>4, <2>5
<1>4. 二重ループの `retain` は、ある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を渡る。
      `uniform` は `insert` でだけ変わり、要素を失わない。`is_uniform` が真の反復では
      `uniform.insert(retain, outstanding.clone())` が、偽の反復では `self.needed_retains.insert(retain)`
      が実行される。この二重ループの外で `uniform` と `self.needed_retains` は変えられない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>5. 4 が成り立つ。<1>4 より `uniform` が `x` を鍵に持つのは、`x` を `retain` とする反復があってその
      `is_uniform` が真のとき、かつそのときに限る。<1>4 より `x` を `retain` とする反復があるのは、ある
      `j` について `arm_states[j]` が `x` を鍵に持つときに限り、<1>3 よりその反復の `is_uniform` が真で
      あることは `U(x)` と同値である。真の反復で `uniform` に入るのは `outstanding.clone()` であり、
      `uniform` の値の型は `References` なので、これは `outstanding` と等しい `References` の値である。
      <1>3 の `U(x)` よりそれは各 `arm_states[j']` の共通の値である。
  BY <1>3, <1>4, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>6. 5 が成り立つ。<1>4 より `self.needed_retains` に入るのは `is_uniform` が偽の反復の `retain` で
      あり、`retain` はある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を渡る。<1>3 より
      `is_uniform` が偽であることは `U(retain)` が成り立たないことと同値である。
  BY <1>3, <1>4
<1>7. 6 が成り立つ。返り値は
      `pending_in.iter().filter_map(|retain| uniform.get(&retain.node).map(|outstanding| PendingRetain { node: retain.node, outstanding: outstanding.clone() })).collect()`
      である。`Iterator::filter_map` は要素の順序を保ち、`uniform.get` が `Some` を返す要素だけを残す。
      作られる要素は `node` が元の要素の `node`、`outstanding` が `uniform[node]` と等しい `References`
      である。`merge` はほかに返り値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     DEF `Iterator::filter_map` (Rust 標準ライブラリの規約)
<1>8. `arm_states` の各表と `pending_in` の反復について、`arm_states` の表は `Map` (ハッシュ表) なので
      反復の順序は定まらないが、<1>3 により `is_uniform` は反復の順序によらず、<1>4 により `uniform` は
      要素を失わないので、`uniform` の最終的な内容は反復の順序によらない。返り値は `pending_in` の順序で
      作られる。
  BY <1>3, <1>4, <1>7, CODE src/misc.rs: Map
<1>9. QED
  BY <1>1, <1>2, <1>3, <1>5, <1>6, <1>7, <1>8

### L8 (出口状態は入口状態から基本操作で得られる)

各節点 `n` について、`pending_out(n)` は `pending(n)` から有限個の基本操作 (追加・消費・引き・併合) の
列で得られる。この列を `n` の**状態の鎖**と呼ぶ。「複製」はこの列に現れない。また、走査が
`PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけである。

**証明** 木 `N(n)` の構造についての帰納法で示す。DEF 部分木より子は真の部分木なので整礎である。

<1>1. 帰納法の仮定: `n` の各子 `c` について、`pending_out(c)` は `pending(c)` から有限個の基本操作の列で
      得られ、その走査が `PendingRetains` の値を作るのは DEF 基本操作 の 6 種だけである。
  BY 帰納法の仮定
<1>2. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。この腕は `pending` に「追加」を 1 回行い、
      `self.walk(k, pending, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「追加」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。この腕はほかに
      `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, <1>1
<1>3. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。この腕は `self.other_objects` の値を
      `others` として `self.consume_objects(&mut pending, &others)` を 1 回呼び (「消費」)、
      `un_bump(&mut pending, &un_bumped)` を 1 回呼び (「引き」)、その返り値が `OutsideBracket` のとき
      さらに `self.consume_objects(&mut pending, &objects)` を 1 回呼ぶ (「消費」)。その後
      `self.walk(k, pending, returns_from_func)` の値を返す。`InBracket` の枝が触れるのは
      `self.un_bump_releases` だけである。よって `pending(k)` は `pending(n)` に有限個の基本操作を
      行ったものであり、`pending_out(n) = pending_out(k)` である。この腕はほかに `PendingRetains` の値を
      作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕, L6, <1>1
<1>4. CASE `n` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。この腕は各アームについて
      `pending.clone()` (「複製」) を渡して `walk` を呼び、`pending` 自身は変えない。その後
      `self.merge(&pending, &arm_exits)` (「併合」) で `merged` を作り、
      `self.walk(k, merged, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「併合」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。アーム本体の入口状態は
      「複製」で作られた別の値であり、この列には現れない。この腕はほかに `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, <1>1
<1>5. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
      `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び (L6 より 0 個以上の「消費」)、
      `self.walk(k, pending, returns_from_func)` の値を返す。ほかに `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, L6, <1>1
<1>6. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。この腕は
      `self.consume` を 0 回以上呼び (L6 より「消費」)、`self.walk(k, pending, returns_from_func)` の
      値を返す。ほかに `PendingRetains` の値を作らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     L6, <1>1
<1>7. CASE `n` の式が `RcExpr::Eval(_, k)` である。この腕は `pending` を変えずに
      `self.walk(k, pending, returns_from_func)` の値を返す。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕, <1>1
<1>8. CASE `n` の式が `RcExpr::Ret(_)` である。この腕は `returns_from_func` が真のとき `pending` の各
      要素の `node` を `self.needed_retains` に入れるが、`pending` を変えずに `pending` を返す。よって
      `pending_out(n) = pending(n)` であり、鎖は空である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
<1>9. QED
  `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <1>2 から <1>8 が尽くす。走査が
  `PendingRetains` の値を作るのは、`cancel` の「初期」と、これら 7 つの腕が行う操作だけである。7 つの腕が
  呼ぶ関数のうち `PendingRetains` に触れるのは `consume_rhs`、`consume`、`consume_objects` (L6 より
  「消費」)、`un_bump` (L5 より「引き」)、`merge` (L7 より「併合」) であり、腕自身が行うのは「追加」
  (<1>2) と「複製」(<1>4) である。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
     CODE src/rc_ir/borrow.rs: cancel, L5, L6, L7

## 5. 状態の不変条件

### DEF INV

状態 `P` (DEF 基本操作) について、次の 4 つの連言を `INV(P)` と書く。「`P` の時点」とは、`P` を作った
基本操作が実行された時点をいう。

- **(i)** `P` の各要素 `e` について、`e.node` は、`P` の時点までに訪問された `Retain` 節点 `t` の
  `node_id(t)` である。P15 の前半より、1 つの本体の中でこの `t` は一意に定まる。これを `e` の**由来**と
  呼び、`orig(e)` と書く。
- **(ii)** `P` の各要素 `e` について、`e.outstanding` は空でなく、`e.outstanding ⊆ ActRefs(orig(e))` で
  ある。
- **(iii)** `P` の相異なる 2 つの要素は相異なる `node` を持つ。
- **(iv)** 添字 `i < j` について、`P[i]` の由来は `P[j]` の由来より前に訪問された。

### L9 (走査が作る状態は `INV` を満たす)

`cancel_body` の 1 回の実行の中で走査が作るすべての状態 `P` について、`INV(P)` が成り立つ。

**証明** 状態の生成順序 (DEF 基本操作) についての帰納法で示す。生成順序は有限の全順序であり、初期状態を
除く各状態は、それより前に作られた状態から基本操作 1 つで作られるので、この帰納法は整礎である。

<1>1. 帰納法の仮定: この状態より前に作られたすべての状態について `INV` が成り立つ。
  BY 帰納法の仮定
<1>2. CASE 状態が「初期」で作られた。`PendingRetains::default()` は `Vec` の `default()` すなわち空の
      `Vec` である。(i) から (iv) はどれも `P` の要素についての全称なので、空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: PendingRetains, DEF INV
<1>3. CASE 状態が「複製」で作られた。`pending.clone()` は元の状態と等しい値である。(ii)、(iii)、(iv) は
      値だけで決まるので、<1>1 より成り立つ。(i) は「`P` の時点までに訪問された」を含むが、訪問された
      節点の集合は時が進んでも要素を失わないので、元の状態で成り立てば複製の時点でも成り立ち、由来も
      変わらない。
  BY <1>1, DEF INV, DEF 訪問
<1>4. CASE 状態が「追加」で作られた。すなわち `Retain` 節点 `t = Retain(v, path, _, k)` の訪問が
      `pending.push(PendingRetain { node: retain, outstanding })` を実行した。ここで
      `retain = node_id(node)` であり `node` は `t` の節点、`outstanding = self.acted_references(v, path)`
      すなわち `ActRefs(t)` である。書き換え前の状態を `P0` とする。
  <2>1. この操作は `Vec` の末尾に要素を 1 つ加えるだけであり、既存の要素の値と並びを変えない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       DEF `Vec::push` (Rust 標準ライブラリの規約)
  <2>2. `P0` は `t` の訪問が始まる前に作られた状態である。よって <1>1 の (i) より、`P0` の各要素の由来は
        `t` の訪問が始まる前に訪問されており、P15 の後半 (各位置はちょうど 1 回訪問される) よりそれは
        `t` ではない。
    BY <1>1, P15, DEF 訪問,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
  <2>3. (i) が成り立つ。新しい要素の `node` は `node_id(t)` であり、`t` の訪問はこの時点で始まっている。
        既存の要素は <1>1 の (i) のままで、由来も変わらない。
    BY <1>1, <2>1, DEF 訪問
  <2>4. (ii) が成り立つ。新しい要素の由来は <2>3 より `t` であり、その `outstanding` は `ActRefs(t)` で
        ある。L2 よりそれは空でなく、`ActRefs(t) ⊆ ActRefs(t)` である。既存の要素は <1>1 の (ii) のまま
        である。
    BY <1>1, <2>1, <2>3, L2, DEF 節点の量
  <2>5. (iii) が成り立つ。
    <3>1. `P0` のどの要素も `node` が `node_id(t)` と等しくない。
      <4>1. `P0` のある要素 `e` が `e.node = node_id(t)` を満たすと仮定する。
        BY 背理法の仮定
      <4>2. <1>1 の (i) より、`orig(e)` は `P0` の時点までに訪問された `Retain` 節点であり、
            `node_id(orig(e)) = e.node = node_id(t)` である。
        BY <1>1, <4>1
      <4>3. P15 の前半より、この本体の相異なる位置は相異なる `NodeId` を持つ。よって `orig(e)` と `t` は
            同じ位置である。
        BY P15, <4>2
      <4>4. QED (矛盾)
        <2>2 より `P0` の各要素の由来は `t` ではない。<4>3 はそれに反する。
        BY <2>2, <4>3
    <3>2. QED
      BY <1>1, <2>1, <3>1
  <2>6. (iv) が成り立つ。新しい要素は `Vec` の末尾に置かれる。<2>2 より既存の要素の由来はどれも `t` の
        訪問が始まる前に訪問されており、新しい要素の由来は `t` である。既存の要素どうしの並びは <1>1 の
        (iv) のままである。
    BY <1>1, <2>1, <2>2, <2>3
  <2>7. QED
    BY <2>3, <2>4, <2>5, <2>6
<1>5. CASE 状態が「消費」で作られた。L6 より、これは要素を取り除くだけであり、残る要素の値と並びを変え
      ない。よって (i)、(ii)、(iii)、(iv) はいずれも <1>1 の対応する節から遺伝する ((i) の「`P` の時点
      までに訪問された」は時が進んでも保たれる)。
  BY <1>1, L6, DEF INV, DEF 訪問
<1>6. CASE 状態が「引き」で作られた。すなわち `Release` 節点の訪問が `un_bump(&mut pending, &un_bumped)`
      を実行した。
  <2>1. CASE L5 の 1 または 2。`pending` は変わらないので、<1>1 の各節がそのまま成り立つ ((i) の
        「`P` の時点までに訪問された」は時が進んでも保たれる)。
    BY <1>1, L5, DEF 訪問
  <2>2. CASE L5 の 3。添字 `i` の要素の `outstanding` が `outstanding - un_bumped` になり、それが空なら
        その要素が取り除かれる。ほかの要素の値と相対順序は変わらない。
    <3>1. (i) が成り立つ。要素の `node` は変わらず、要素は減るだけである。
      BY <1>1, L5, DEF 訪問
    <3>2. (ii) が成り立つ。差が空になった要素はそのとき取り除かれるので、残る要素の `outstanding` は
          空でない。L2 の 4 より `outstanding - un_bumped ⊆ outstanding` であり、`⊆` は推移的なので
          (DEF 参照の多重集合)、<1>1 の (ii) から `outstanding - un_bumped ⊆ ActRefs(orig(e))` が従う。
      BY <1>1, L5, L2, DEF 参照の多重集合
    <3>3. (iii) が成り立つ。要素は減るだけで `node` は変わらない。
      BY <1>1, L5
    <3>4. (iv) が成り立つ。要素の相対順序と由来は変わらない。
      BY <1>1, L5
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4
  <2>3. QED
    L5 の 3 つの場合は尽くしている。
    BY L5, <2>1, <2>2
<1>7. CASE 状態が「併合」で作られた。すなわち `Match` 節点の訪問が `self.merge(&pending, &arm_exits)` を
      実行し、その返り値 `merged` が新しい状態である。`pending_in` を `&pending` の指す状態とする。
  <2>1. 各 `arm_exits[j]` は `pending_out(arm_j.body)` であり、`merged` より前に作られた状態である。
        よって <1>1 より `INV(arm_exits[j])` が成り立ち、その (iii) より `arm_exits[j]` の相異なる要素は
        相異なる `node` を持つ。すなわち L7 の仮定が満たされる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       <1>1, DEF 基本操作
  <2>2. `pending_in` は `merged` より前に作られた状態なので、<1>1 より `INV(pending_in)` が成り立つ。
    BY <1>1
  <2>3. (i) が成り立つ。L7 の 6 より、`merged` の各要素の `node` は `pending_in` のある要素の `node` で
        ある。<2>2 の (i) より、それは `pending_in` の時点までに訪問された `Retain` 節点の `node_id` で
        あり、`merged` の時点はそれより後である。由来は `pending_in` のその要素の由来と同じである。
    BY L7, <2>1, <2>2, DEF 訪問
  <2>4. (ii) が成り立つ。
    <3>1. `merged` の各要素 `e` の `outstanding` は、L7 の 6 と 4 より、ある `j` について
          `arm_states[j]` が `e.node` に与える値、すなわち `arm_exits[j]` の要素 `e'` で
          `e'.node = e.node` であるものの `outstanding` と等しい。
      BY L7, <2>1
    <3>2. `orig(e') = orig(e)` である。`e'.node = e.node` であり、由来は `node` から P15 の前半で一意に
          定まる。
      BY P15, DEF INV, <3>1, <2>1, <2>3
    <3>3. QED
      <2>1 の `INV(arm_exits[j])` の (ii) より `e'.outstanding` は空でなく
      `e'.outstanding ⊆ ActRefs(orig(e'))` である。<3>1 と <3>2 より `e.outstanding` はそれと等しい値で
      あり、その由来も同じである。
      BY <2>1, <3>1, <3>2, DEF 参照の多重集合
  <2>5. (iii) が成り立つ。L7 の 6 より、`merged` の要素は `pending_in` の要素から `filter_map` で作られ、
        1 つの入力要素からは高々 1 つの出力要素ができ、`node` は変わらない。<2>2 の (iii) より
        `pending_in` の相異なる要素は相異なる `node` を持つ。
    BY L7, <2>1, <2>2
  <2>6. (iv) が成り立つ。L7 の 6 より、`merged` は `pending_in` の部分列であり、順序は保たれる。要素の
        `node` は変わらないので由来も変わらない。<2>2 の (iv) よりその並びは由来の訪問順である。
    BY L7, <2>1, <2>2, <2>3
  <2>7. QED
    BY <2>3, <2>4, <2>5, <2>6
<1>8. QED
  DEF 基本操作 と L8 より、状態の作られ方は「初期」「複製」「追加」「消費」「引き」「併合」の 6 種で
  尽きる。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, DEF 基本操作, L8
