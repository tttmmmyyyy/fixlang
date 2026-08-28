# P15 - P18: `cancel` の走査

この文書は README の層 3 の 4 命題 P15, P16, P17, P18 を証明する。README の定義 D1 - D16 と仮定
A1 - A8 の上に立つ。層 1 と層 2 の命題は引用しない。

## 0. この文書が使う記法

README の第 2 節の記法に、次の 2 つを加える。

- **局所の定義**。この文書の中だけで使う語を第 1 節と第 5 節で定める。`BY` の行では `DEF <名前>` で引用する。
- **局所の補題**。この文書の中だけで使う補題を `L1` - `L10` と番号を付けて述べ、`BY` の行では
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

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、
`CancelAnalysis` の走査中に次の値を定める。

- `key(t) := self.unit_key(&v.name, path)`、`bumped(t) := self.acted_references(v, path)`
- `key(r) := self.unit_key(&v.name, path)`
- `others(r) := self.acted_unit_keys(&v.name, path)` の要素のうち `key(r)` と異なるもの

`CancelAnalysis::unit_key` は `ownership::unit_key` を、`CancelAnalysis::acted_unit_keys` は
`ownership::acted_unit_keys` を、`CancelAnalysis::acted_references` は `ownership::acted_references` を、
それぞれ `self.vars` と `self.type_env` を渡して呼ぶ (`CODE src/rc_ir/borrow.rs: CancelAnalysis::unit_key`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_unit_keys`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。D15 の 3 つの関数はどれも `vars`、`type_env`、
および渡された `(変数, path)` だけから値が決まるので、上の 4 つの量は走査のどの時点で読んでも同じ値である。

### DEF 参照の多重集合

`References` は `Map<VarPath, usize>` を 1 つ持つ構造体である (`CODE src/rc_ir/ownership.rs: References`)。
これを、鍵をオブジェクトの名前、値をその個数とする多重集合とみなす。多重集合の差 `R1 - R2` を各オブジェクトの
個数の差とする (`R1.covers(&R2)` が成り立つときだけ書く)。**空**とは、参照を 1 つも持たないことをいう。

### DEF 割り当て

`Arc::new` は 1 つの**割り当て**を作る。その `Arc` またはその複製が 1 つでも生きている間、その割り当ては
生きている。生きている 2 つの相異なる割り当ては記憶域を共有しないので、それらの先頭アドレスは相異なる。

### DEF 基本操作

走査が `PendingRetains` の値を作るのは、次の 6 種だけである。

| 名 | 作られ方 |
|---|---|
| 初期 | `cancel_body` の `analysis.walk(body, PendingRetains::default(), true)` の第 2 引数 |
| 複製 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `pending.clone()` |
| 追加 | `RcExpr::Retain(v, path, _, k)` の腕の `pending.entry(...).or_default().push(...)` |
| 鍵除去 | `consume_unit` の `pending.remove(&key)` |
| 引き | `RcExpr::Release(v, path, _, k)` の腕の `un_bump(&mut pending, &key, &un_bumped)` |
| 併合 | `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕の `self.merge(&pending, &arm_exits)` |

(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit`, `CODE src/rc_ir/borrow.rs: un_bump`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)

走査が作る `PendingRetains` の値を**状態**と呼ぶ。「追加」「鍵除去」「引き」は直前の状態をその場で書き換え、
「複製」と「併合」は新しい値を作る。状態には**生成順序** (走査がそれを作る時間順) があり、初期状態を除く各
状態は、それより前に作られた 1 つまたは 2 つの状態から上の操作 1 つで作られる。走査は有限回で終わるので、
生成順序は有限の全順序である。

### DEF 除去事象

基本操作 1 つが、状態 `P` を入力の 1 つとして状態 `P'` を作り、`P` に `node` が `x` である要素があり、
`P'` にはそれが無いとき、この操作を `x` の**除去事象**と呼ぶ。「併合」の入力は `pending_in` と `arm_exits`
であり、ここで `P` とするのは `pending_in` である。「追加」「鍵除去」「引き」は状態をその場で書き換えるので、
`P` は書き換えの前の値、`P'` は後の値である。

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
`R.is_empty()` が真であることと `R` が空 (DEF 参照の多重集合) であることは同値である。また
`R1.covers(&R2)` が真のとき、`R1.subtract(&R2)` は `R1` を `R1 - R2` に書き換え、panic しない。

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
      ある。
  BY CODE src/rc_ir/ownership.rs: References::subtract
<1>4. `References::covers(other)` は、`other` のどの鍵についても、自分がその鍵を持ちその値以上であることを
      言う。よって `covers` が真のとき、<1>3 の `get_mut` の `expect` は発火しない。
  BY CODE src/rc_ir/ownership.rs: References::covers, CODE src/rc_ir/ownership.rs: References::subtract
<1>5. `References::is_empty()` は内側の `Map` が空であることを言う。各鍵の値が 1 以上の `References` に
      ついて、これは参照を 1 つも持たないことと同値である。
  BY CODE src/rc_ir/ownership.rs: References::is_empty, <1>1, <1>3
<1>6. 走査が扱う `References` の値は、`CancelAnalysis::acted_references` が返したもの、それを `subtract`
      で減らしたもの、およびそれらの複製だけである。
  BY CODE src/rc_ir/ownership.rs: References (フィールドは非公開なので `ownership` の外では作れない),
     CODE src/rc_ir/ownership.rs: acted_references (`References(references)` がこのモジュールでの唯一の
     構築点), CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
     (`outstanding` は `self.acted_references` の値), CODE src/rc_ir/borrow.rs: un_bump
     (`innermost.outstanding.subtract(un_bumped)`), CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
     (`outstanding.clone()`)
<1>7. QED
  BY <1>2, <1>3, <1>4, <1>5, <1>6

## 3. P15 (節点と `NodeId`)

**言明** --- 1 つの本体の木の相異なる位置は相異なる `NodeId` を持つ。また `CancelAnalysis::walk` は本体の
各位置をちょうど 1 回訪れる。

前半は、`cancel` が走査する本体、すなわち `cancel` の入力プログラムの各関数の `body` と各グローバル
初期化子の `init` について示す。D2 が述べるとおり `RcExprNode` は式を `Arc` で共有できるので、これは
`RcExprNode` 一般の性質ではなく、`cancel` に渡される木の性質である。

### 3.1 前半 (相異なる位置は相異なる `NodeId` を持つ)

<1>1. `node_id(node)` は `node.expr` が指す `RcExpr` の割り当てのアドレスである。
  BY CODE src/rc_ir/borrow.rs: node_id
<1>2. 同時に生きている相異なる割り当てはアドレスが相異なる。
  BY DEF 割り当て
<1>3. `cancel` に渡されるプログラムの各関数の `body` と各グローバル初期化子の `init` は、
      `RewriteCtx::rewrite` の 1 回の呼び出しが返した木である。
  <2>1. `optimize_rc_program` は `config.enable_borrow_optimization()` が真の枝でだけ
        `cancel(&prog, type_env, config.develop_mode)` を呼ぶ。その直前の文は
        `prog = borrow_ify(&prog, type_env)` であり、間にあるのは `validate(&prog, "after borrow_ify")`
        だけである。`validate` はプログラムを変えない。
    BY CODE src/build/build_object_files.rs: optimize_rc_program
  <2>2. `borrow_ify` が返す `RcProgram` の `funcs` の各値の `body` は `ctx.rewrite(&f_own.body)` または
        `ctx.rewrite(&clone.body)` の値であり、`globals` の各値の `init` は `ctx.rewrite(&g.init)` の
        値である。
    BY CODE src/rc_ir/borrow.rs: borrow_ify
  <2>3. QED
    BY <2>1, <2>2
<1>4. `RewriteCtx::rewrite(node)` の 1 回の呼び出しが返す木の各位置は、その呼び出しの間に `expr_node` が
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
  <2>4. `rewrite_inner(node)` の 7 つの腕はいずれも、`self.rewrite` を `node` の各子についてちょうど
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
      `Destructure`、`Eval`、`Ret` を尽くす。
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
<1>5. `cancel_body(vars, body)` の実行中、`body` の木の割り当てはすべて生きている。
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
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### 3.2 後半 (`walk` は各位置をちょうど 1 回訪れる)

<1>1. `CancelAnalysis::walk` を呼ぶのは、`cancel` の中の
      `analysis.walk(body, PendingRetains::default(), true)` と、`walk_inner` の中の 7 か所だけである。
      その 7 か所は、`Retain` の腕、`Release` の腕、`Match` の腕の 2 か所 (アームと継続)、
      右辺が `Match` でない `Let` の腕、`Destructure` の腕、`Eval` の腕である。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
<1>2. `walk_inner` が呼ぶ `walk` 以外の関数 --- `CancelAnalysis::unit_key`,
      `CancelAnalysis::acted_unit_keys`, `CancelAnalysis::acted_references`,
      `CancelAnalysis::consume_rhs`, `CancelAnalysis::consume`, `CancelAnalysis::consume_unit`,
      `CancelAnalysis::merge`, `un_bump`, `check_one_key_per_object`, `node_id`,
      `destructure_consumes` --- はどれも `walk` を呼ばない。
  BY <1>1
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

### L3 (`un_bump` の作用)

`un_bump(pending, key, un_bumped)` の 1 回の呼び出しについて、`pending` が鍵 `key` を持つならばその `Vec`
が空である場合を除く。このとき次の 3 つが成り立ち、この 3 つは場合を尽くす。

1. `pending` が鍵 `key` を持たないとき、返り値は `NoBracket` であり、`pending` は変わらない。
2. `pending` が鍵 `key` を持ち、その `Vec` の最後の要素 `e` について `e.outstanding.covers(un_bumped)` が
   偽のとき、返り値は `OutsideBracket` であり、`pending` は変わらない。
3. `pending` が鍵 `key` を持ち、その `Vec` の最後の要素 `e` について `e.outstanding.covers(un_bumped)` が
   真のとき、返り値は `InBracket(e.node)` である。`pending` は次のように変わる。`e.outstanding` が
   `e.outstanding - un_bumped` になる。それが空ならその要素が `Vec` の末尾から取り除かれ、`Vec` が空に
   なれば鍵 `key` が `pending` から取り除かれる。ほかの鍵、`Vec` のほかの要素、要素の並びは変わらない。

**証明**

<1>1. `let Some(stack) = pending.get_mut(key) else { return UnBump::NoBracket; };` により、`pending` が
      鍵 `key` を持たないとき `NoBracket` を返す。この文までに `pending` を変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump
<1>2. `let innermost = stack.last_mut().expect("a stack kept in `pending` is non-empty");` は、仮定
      (`Vec` が空でない) により `Vec` の最後の要素への可変参照を返し、`expect` は発火しない。
  BY CODE src/rc_ir/borrow.rs: un_bump, 本補題の仮定
<1>3. `if !innermost.outstanding.covers(un_bumped) { return UnBump::OutsideBracket; }` により、covers が
      偽のとき `OutsideBracket` を返す。この文までに `pending` を変える操作は無い。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>1, <1>2
<1>4. covers が真のとき、`innermost.outstanding.subtract(un_bumped)` は `innermost.outstanding` を
      `e.outstanding - un_bumped` に書き換え、panic しない。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2
<1>5. `let retain = innermost.node;` は `e.node` を控える。`subtract` は `node` を変えない。
  BY CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: References::subtract
<1>6. `if innermost.outstanding.is_empty() { stack.pop(); }` は、<1>4 の差が空のときちょうど `Vec` の
      最後の要素を取り除き、空でないとき何もしない。
  BY CODE src/rc_ir/borrow.rs: un_bump, L2
<1>7. `if stack.is_empty() { pending.remove(key); }` は、`Vec` が空になったときだけ鍵 `key` を取り除く。
  BY CODE src/rc_ir/borrow.rs: un_bump
<1>8. `UnBump::InBracket(retain)` を返す。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>5
<1>9. <1>1 から <1>8 の間に `pending` に触れるのは <1>4、<1>6、<1>7 だけであり、これらは鍵 `key` の
      `Vec` の最後の要素と、その `Vec`、その鍵にしか触れない。`Vec::pop` は末尾だけを取り除くので、
      残る要素の並びは変わらない。
  BY CODE src/rc_ir/borrow.rs: un_bump, <1>4, <1>6, <1>7
<1>10. QED
  場合分けは「鍵を持たない」「鍵を持ち covers が偽」「鍵を持ち covers が真」であり、尽くしている。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9

### L4 (消費の作用)

`CancelAnalysis::consume_unit`、`CancelAnalysis::consume`、`CancelAnalysis::consume_rhs`、および
`walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕が `pending` に対して行うのは
「鍵除去」だけである。1 回の「鍵除去」は、鍵とその `Vec` を丸ごと取り除き、取り除いた `Vec` の各要素の
`node` を `self.needed_retains` に入れる。

**証明**

<1>1. `consume_unit(pending, key)` の本体は
      `if let Some(stack) = pending.remove(&key) { for retain in stack { self.needed_retains.insert(retain.node); } }`
      である。よって、鍵 `key` があればその鍵と `Vec` を取り除き、取り除いた `Vec` の各要素の `node` を
      `needed_retains` に入れる。鍵が無ければ `pending` を変えない。ほかの鍵には触れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit
<1>2. `consume(pending, var, path)` は `self.acted_unit_keys(var, path)` の各要素 `key` について
      `self.consume_unit(pending, key)` を呼ぶ。ほかに `pending` に触れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume
<1>3. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` を呼んで `consumed` を集め、その各要素
      `(var, leaf)` について `self.consume(pending, &var, &leaf)` を呼ぶ。ほかに `pending` に触れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs
<1>4. `rhs_consumes` は `pending` を引数に取らないので、`pending` を変えない。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes
<1>5. `walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕は、
      `destructure_consumes(container, fields, self.type_env)` の各 `leaf` について
      `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。ほかに `pending` に触れない。
      `destructure_consumes` は `pending` を引数に取らない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     CODE src/rc_ir/ownership.rs: destructure_consumes
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L5 (`merge` の作用)

`self.merge(pending_in, arm_exits)` の 1 回の呼び出しについて、各 `arm_exits[j]` の相異なる要素は相異なる
`node` を持つとする。このとき次が成り立つ。

1. `arm_states[j]` は `arm_exits[j]` の各要素 `e` を `e.node` から `&e.outstanding` へ写す表であり、
   その鍵の集合は `arm_exits[j]` の要素の `node` の集合に等しい。
2. `entered_with` は `pending_in` の要素の `node` の集合である。
3. `NodeId` の値 `x` について、呼び出しの終わりに `uniform` が `x` を鍵に持つことと、次が成り立つことは
   同値である。`x` が `entered_with` の要素であり、ある `j` について `arm_states[j]` が `x` を鍵に持ち、
   すべての `j'` について `arm_states[j']` が `x` を鍵に持って、それらの値が互いに等しい。このとき
   `uniform[x]` はその共通の値の複製である。
4. この呼び出しが `self.needed_retains` に入れる `NodeId` は、ある `j` について `arm_states[j]` が鍵に
   持ち、かつ呼び出しの終わりに `uniform` が鍵に持たないもの全部である。
5. 返り値 `merged` は、`pending_in` の各鍵 `key` について、`pending_in[key]` の要素のうち `node` を
   `uniform` が鍵に持つものだけを、その並びのまま、`outstanding` を `uniform[node]` の複製に差し替えて
   並べた `Vec` を持つ。その `Vec` が空になる鍵は `merged` に入らない。`merged` はほかに鍵も要素も
   持たない。

**証明**

<1>1. `arm_states` は
      `arm_exits.iter().map(|exit| exit.values().flatten().map(|retain| (retain.node, &retain.outstanding)).collect()).collect()`
      である。`exit.values().flatten()` は `exit` のすべての要素を渡すので、`arm_states[j]` の鍵の集合は
      `arm_exits[j]` の要素の `node` の集合であり、仮定より 2 つの要素が 1 つの鍵に落ちることはない。
      よって 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, 本補題の仮定
<1>2. `entered_with` は `pending_in.values().flatten().map(|retain| retain.node).collect()` である。
      よって 2 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>3. 二重ループの各反復で計算される `is_uniform` の値は `retain` だけで決まり、それが真であることは、
      `retain` が `entered_with` の要素であり、かつすべての `j'` について `arm_states[j']` が `retain` を
      鍵に持ってそれらの値が互いに等しいこと、と同値である。
  <2>1. 反復は `for states in &arm_states { for (&retain, &outstanding) in states { ... } }` であり、
        `outstanding` は `states` すなわちある `arm_states[j]` の `retain` の値である。`is_uniform` は
        `entered_with.contains(&retain)` と
        `arm_states.iter().all(|other| other.get(&retain) == Some(&outstanding))` の連言である。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
  <2>2. CASE すべての `j'` について `arm_states[j']` が `retain` を鍵に持ち、その値が互いに等しい。
        このとき、`retain` を鍵に持つどの `arm_states[j]` の反復から見ても、`outstanding` はその共通の
        値であり、第 2 の連言肢は真である。
    BY <2>1
  <2>3. CASE ある `j'` について `arm_states[j']` が `retain` を鍵に持たない。このとき
        `other.get(&retain)` は `None` であり、`Some(&outstanding)` と等しくないので、`retain` を鍵に
        持つどの `arm_states[j]` の反復から見ても第 2 の連言肢は偽である。
    BY <2>1
  <2>4. CASE すべての `j'` が `retain` を鍵に持つが、2 つの `j'` の値が異なる。このとき、`retain` を鍵に
        持つどの `arm_states[j]` の反復から見ても、値の異なるもう一方の `j'` で `all` が偽になるので、
        第 2 の連言肢は偽である。
    BY <2>1
  <2>5. QED
    <2>2、<2>3、<2>4 は、`retain` を鍵に持つ `arm_states[j']` の有無と値の一致について場合を尽くし、
    第 2 の連言肢の値が `retain` だけで決まること、およびそれが真であるのは <2>2 の場合に限ることを
    与える。第 1 の連言肢 `entered_with.contains(&retain)` も `retain` だけで決まる。
    BY <2>1, <2>2, <2>3, <2>4
<1>4. 二重ループの `retain` は、ある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を渡る。
      `uniform` は `insert` でだけ変わり、要素を失わない。`is_uniform` が真の反復では
      `uniform.insert(retain, outstanding.clone())` が、偽の反復では `self.needed_retains.insert(retain)`
      が実行される。この二重ループの外で `uniform` と `self.needed_retains` は変えられない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>5. 3 が成り立つ。
  <2>1. `uniform` が `x` を鍵に持つのは、`x` を `retain` とする反復で `is_uniform` が真になったとき、
        かつそのときに限る。
    BY <1>4
  <2>2. `x` を `retain` とする反復があるのは、ある `j` について `arm_states[j]` が `x` を鍵に持つとき、
        かつそのときに限る。
    BY <1>4
  <2>3. `is_uniform` が真であることは、`x` が `entered_with` の要素であり、すべての `j'` について
        `arm_states[j']` が `x` を鍵に持ってその値が互いに等しいことと同値である。
    BY <1>3
  <2>4. 真の反復で `uniform` に入る値は `outstanding.clone()` であり、<2>3 よりそれは共通の値である。
    BY <1>4, <2>3
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4
<1>6. 4 が成り立つ。<1>4 より、この呼び出しが `self.needed_retains` に入れるのは `is_uniform` が偽の
      反復の `retain` であり、`retain` はある `j` について `arm_states[j]` が鍵に持つ `NodeId` の全体を
      渡る。<1>3 より `is_uniform` の真偽は本補題の 3 の右辺の条件と一致し、<1>5 が示した 3 より、その
      条件は `uniform` が `retain` を鍵に持つことと同値である。よって `needed_retains` に入るのは、ある
      `j` について `arm_states[j]` が鍵に持ち、かつ `uniform` が鍵に持たないもの全部である。
  BY <1>3, <1>4, <1>5
<1>7. 5 が成り立つ。再構築のループは `for (key, stack) in pending_in` を回り、
      `stack.iter().filter_map(|retain| uniform.get(&retain.node).map(|outstanding| PendingRetain { node: retain.node, outstanding: outstanding.clone() }))`
      を `kept` に集め、`kept` が空でないときだけ `merged.insert(key.clone(), kept)` を行う。
      `Iterator::filter_map` は要素の順序を保ち、`uniform.get` が `Some` を返す要素だけを残す。
      `merged` はこのループの外では変えられない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>8. `arm_states` と `pending_in` は `Map` (ハッシュ表) なので反復の順序は定まらないが、<1>3 により
      `is_uniform` は反復の順序によらず、<1>4 により `uniform` は要素を失わないので、`uniform` の
      最終的な内容は反復の順序によらない。`merged` は鍵ごとに独立に作られるので、その内容も
      `pending_in` の反復の順序によらない。
  BY <1>3, <1>4, <1>7, CODE src/misc.rs: Map (`FxHashMap` の別名)
<1>9. QED
  BY <1>1, <1>2, <1>5, <1>6, <1>7, <1>8

### L6 (出口状態は入口状態から基本操作で得られる)

各節点 `n` について、`pending_out(n)` は `pending(n)` から有限個の基本操作 (追加・鍵除去・引き・併合) の
列で得られる。この列を `n` の**状態の鎖**と呼ぶ。「複製」はこの列に現れない。

**証明** 木 `N(n)` の構造についての帰納法で示す。DEF 部分木より子は真の部分木なので整礎である。

<1>1. 帰納法の仮定: `n` の各子 `c` について、`pending_out(c)` は `pending(c)` から有限個の基本操作の列で
      得られる。
  BY 帰納法の仮定
<1>2. CASE `n` の式が `RcExpr::Retain(v, path, _, k)` である。この腕は `pending` に「追加」を 1 回行い、
      `self.walk(k, pending, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「追加」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, <1>1
<1>3. CASE `n` の式が `RcExpr::Release(v, path, _, k)` である。この腕は、`others` の各要素について
      `consume_unit` を呼び (L4 より「鍵除去」)、`un_bump` を 1 回呼び (「引き」)、その返り値が
      `OutsideBracket` のときさらに `consume_unit` を 1 回呼ぶ。その後
      `self.walk(k, pending, returns_from_func)` の値を返す。`check_one_key_per_object` は `pending` を
      共有参照で受け取り、`assert!` を評価するだけなので `pending` を変えない。よって `pending(k)` は
      `pending(n)` に有限個の基本操作を行ったものであり、`pending_out(n) = pending_out(k)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: check_one_key_per_object, L4, <1>1
<1>4. CASE `n` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。この腕は各アームについて
      `pending.clone()` (「複製」) を渡して `walk` を呼び、`pending` 自身は変えない。その後
      `self.merge(&pending, &arm_exits)` (「併合」) で `merged` を作り、
      `self.walk(k, merged, returns_from_func)` の値を返す。よって `pending(k)` は `pending(n)` に
      「併合」を 1 回行ったものであり、`pending_out(n) = pending_out(k)` である。アーム本体の入口状態は
      「複製」で作られた別の値であり、この列には現れない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, <1>1
<1>5. CASE `n` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
      `self.consume_rhs(&mut pending, rhs, &x.ty)` を呼び (L4 より 0 個以上の「鍵除去」)、
      `self.walk(k, pending, returns_from_func)` の値を返す。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, L4, <1>1
<1>6. CASE `n` の式が `RcExpr::Destructure(container, fields, _state, k)` である。この腕は
      `self.consume` を 0 回以上呼び (L4 より「鍵除去」)、`self.walk(k, pending, returns_from_func)` の
      値を返す。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     L4, <1>1
<1>7. CASE `n` の式が `RcExpr::Eval(_, k)` である。この腕は `pending` を変えずに
      `self.walk(k, pending, returns_from_func)` の値を返す。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕, <1>1
<1>8. CASE `n` の式が `RcExpr::Ret(_)` である。この腕は `self.needed_retains` に要素を入れることが
      あるが、`pending` を変えずに `pending` を返す。よって `pending_out(n) = pending(n)` であり、
      鎖は空である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
<1>9. QED
  `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <1>2 から <1>8 が尽くす。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

## 5. 状態の不変条件

### DEF INV

状態 `P` (DEF 基本操作) について、次の 5 つの連言を `INV(P)` と書く。(ii) の「すでに訪問した」は、`P` が
作られた時点までに、という意味である。

- **(i)** `P` のどの鍵についても、その `Vec` は空でない。
- **(ii)** `P` の各要素 `e` について、`e.node` は走査がすでに訪問した `Retain` 節点 `t` の `node_id` で
  あり、`e` が入っている鍵は `key(t)` に等しい。以下、この `t` を `e` の**由来**と呼ぶ。
- **(iii)** `P` の各要素の `outstanding` は空でない。
- **(iv)** `P` の相異なる 2 つの要素 (鍵をまたぐ 2 つを含む) は相異なる `node` を持つ。
- **(v)** 各鍵 `k` と添字 `i < j` について、`P[k][i]` の由来は `P[k][j]` の由来より前に訪問された。

### L7 (走査が作る状態は `INV` を満たす)

`cancel_body` の 1 回の実行の中で走査が作るすべての状態 `P` について、`INV(P)` が成り立つ。

**証明** 状態の生成順序 (DEF 基本操作) についての帰納法で示す。生成順序は有限の全順序であり、初期状態を
除く各状態は、それより前に作られた状態から基本操作 1 つで作られるので、この帰納法は整礎である。

<1>1. 帰納法の仮定: この状態より前に作られたすべての状態について `INV` が成り立つ。
  BY 帰納法の仮定
<1>2. CASE 状態が「初期」で作られた。`PendingRetains::default()` は鍵を 1 つも持たない `Map` である。
      (i) から (v) はどれも `P` の鍵または要素についての全称なので、空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel, DEF INV
<1>3. CASE 状態が「複製」で作られた。`pending.clone()` は元の状態と等しい値である。(i)、(iii)、(iv)、
      (v) は値だけで決まるので、<1>1 より成り立つ。(ii) は「すでに訪問した」を含むが、走査が訪問した
      節点の集合は時が進んでも要素を失わないので、元の状態で成り立てば複製の時点でも成り立つ。
  BY <1>1, DEF INV, DEF 訪問
<1>4. CASE 状態が「追加」で作られた。すなわち `Retain` 節点 `t = Retain(v, path, _, k)` の訪問が
      `pending.entry(self.unit_key(&v.name, path)).or_default().push(PendingRetain { node: retain,
      outstanding })` を実行した。ここで `retain = node_id(node)` であり、
      `outstanding = self.acted_references(v, path)` である。
  <2>1. この操作は、鍵 `key(t)` の `Vec` の末尾に要素を 1 つ加えるだけである。鍵が無ければ空の `Vec` を
        作ってから加える。ほかの鍵と、その `Vec` のほかの要素には触れない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
  <2>2. (i) が成り立つ。<2>1 より、触れた鍵の `Vec` は要素を 1 つ加えた後なので空でない。ほかの鍵は
        <1>1 の (i) のまま。
    BY <1>1, <2>1
  <2>3. (ii) が成り立つ。新しい要素の `node` は `node_id(t)` であり、`t` は今訪問されている `Retain`
        節点である。その要素が入る鍵は `self.unit_key(&v.name, path)` すなわち `key(t)` である。既存の
        要素は <1>1 の (ii) のまま。
    BY <1>1, <2>1, DEF 節点の量
  <2>4. (iii) が成り立つ。新しい要素の `outstanding` は `CancelAnalysis::acted_references` の値なので
        空でない。既存の要素は <1>1 の (iii) のまま。
    BY <1>1, L2
  <2>5. (iv) が成り立つ。
    <3>1. 追加の前の状態のどの要素も `node` が `node_id(t)` と等しくない。
      <4>1. 追加の前の状態のある要素 `e` が `e.node = node_id(t)` を満たすと仮定する。
        BY 背理法の仮定
      <4>2. <1>1 の (ii) より、`e` の由来 `t'` は走査がすでに訪問した `Retain` 節点であり、
            `node_id(t') = e.node = node_id(t)` である。
        BY <1>1, <4>1
      <4>3. P15 の前半より、この本体の相異なる位置は相異なる `NodeId` を持つ。よって `t'` と `t` は
            同じ位置である。
        BY P15, <4>2
      <4>4. `t` の訪問がこの「追加」より前に `pending` に触れることはないので、`e` は `t` の入口状態
            `pending(t)` の要素である。`pending(t)` は `t` の訪問が始まる前に作られた状態なので、それに
            <1>1 の (ii) を適用すると、`t'` は `t` の訪問が始まる前にすでに訪問されていた。<4>3 より
            `t'` は `t` と同じ位置なので、`t` は 2 回訪問されたことになる。
        BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
           <1>1, <4>2, <4>3, DEF 訪問
      <4>5. QED (矛盾)
        P15 の後半は `walk` が本体の各位置をちょうど 1 回訪れることを言うので、<4>4 と矛盾する。
        BY P15, <4>4
    <3>2. QED
      BY <1>1, <2>1, <3>1
  <2>6. (v) が成り立つ。新しい要素は鍵 `key(t)` の `Vec` の末尾に置かれる。同じ鍵の既存の要素の由来は
        <1>1 の (ii) よりすでに訪問された節点であり、`t` の訪問は今なので、既存の要素の由来はどれも
        `t` より前に訪問された。ほかの鍵の並びは変わらない。
    BY <1>1, <2>1
  <2>7. QED
    BY <2>2, <2>3, <2>4, <2>5, <2>6
<1>5. CASE 状態が「鍵除去」で作られた。L4 より、これは鍵とその `Vec` を丸ごと取り除く。残る鍵と要素は
      変わらないので、(i)、(ii)、(iii)、(iv)、(v) はいずれも <1>1 の対応する節から遺伝する。
  BY <1>1, L4, DEF INV
<1>6. CASE 状態が「引き」で作られた。すなわち `Release` 節点の訪問が
      `un_bump(&mut pending, &key, &un_bumped)` を実行した。
  <2>1. `pending` が鍵 `key` を持つならその `Vec` は空でない。よって L3 の仮定が満たされ、L3 の 3 つの
        場合分けが使える。
    BY <1>1 (直前の状態の (i))
  <2>2. CASE L3 の 1 または 2。`pending` は変わらないので、<1>1 の各節がそのまま成り立つ。
    BY <1>1, L3, <2>1
  <2>3. CASE L3 の 3。鍵 `key` の `Vec` の最後の要素の `outstanding` が減り、それが空になればその要素が
        末尾から取り除かれ、`Vec` が空になれば鍵が取り除かれる。ほかは変わらない。
    <3>1. (i) が成り立つ。`Vec` が空になる場合だけ鍵が取り除かれるので、残る鍵の `Vec` は空でない。
      BY L3, <2>1
    <3>2. (ii) が成り立つ。要素の `node` も鍵も変わらず、要素は減るだけである。
      BY <1>1, L3
    <3>3. (iii) が成り立つ。差が空になった要素はちょうどそのとき取り除かれるので、残る要素の
          `outstanding` は空でない。
      BY <1>1, L3, L2
    <3>4. (iv) が成り立つ。要素は減るだけで `node` は変わらない。
      BY <1>1, L3
    <3>5. (v) が成り立つ。取り除かれるのは `Vec` の末尾の 1 要素だけなので、残る要素の並びは変わらない。
      BY <1>1, L3
    <3>6. QED
      BY <3>1, <3>2, <3>3, <3>4, <3>5
  <2>4. QED
    BY L3, <2>1, <2>2, <2>3
<1>7. CASE 状態が「併合」で作られた。すなわち `Match` 節点の訪問が `self.merge(&pending, &arm_exits)` を
      実行し、その返り値 `merged` が新しい状態である。
  <2>1. 各 `arm_exits[j]` は `pending_out(arm_j.body)` であり、`merged` より前に作られた状態である。
        よって <1>1 より `INV(arm_exits[j])` が成り立ち、その (iv) より `arm_exits[j]` の相異なる要素は
        相異なる `node` を持つ。すなわち L5 の仮定が満たされる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       <1>1, DEF 訪問
  <2>2. `pending_in` は `merged` より前に作られた状態なので、<1>1 より `INV(pending_in)` が成り立つ。
    BY <1>1
  <2>3. (i) が成り立つ。L5 の 5 より、`merged` に入る鍵は `kept` が空でないものだけである。
    BY L5, <2>1
  <2>4. (ii) が成り立つ。L5 の 5 より、`merged` の各要素の `node` は `pending_in` のある要素の `node`
        であり、入る鍵も `pending_in` でのその要素の鍵である。<2>2 の (ii) より、その `node` は
        すでに訪問した `Retain` 節点の `node_id` であり、鍵はその由来の `key` である。
    BY L5, <2>1, <2>2
  <2>5. (iii) が成り立つ。L5 の 5 より、`merged` の要素の `outstanding` は `uniform[node]` の複製で
        ある。L5 の 3 より、それはある `arm_states[j]` の値、すなわち `arm_exits[j]` のある要素の
        `outstanding` の複製である。<2>1 の (iii) よりそれは空でない。
    BY L5, <2>1
  <2>6. (iv) が成り立つ。L5 の 5 より、`merged` の要素は `pending_in` の要素の一部と `node` を共有し、
        `pending_in` の 1 つの要素からは高々 1 つの `merged` の要素ができる。<2>2 の (iv) より
        `pending_in` の相異なる要素は相異なる `node` を持つ。
    BY L5, <2>1, <2>2
  <2>7. (v) が成り立つ。L5 の 5 より、各鍵の `Vec` は `pending_in` の同じ鍵の `Vec` の部分列であり、
        順序は保たれる。要素の `node` は変わらないので由来も変わらない。<2>2 の (v) よりその並びは
        由来の訪問順である。
    BY L5, <2>1, <2>2
  <2>8. QED
    BY <2>3, <2>4, <2>5, <2>6, <2>7
<1>8. QED
  DEF 基本操作より、状態の作られ方は「初期」「複製」「追加」「鍵除去」「引き」「併合」の 6 種で尽きる。
  BY <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, DEF 基本操作

## 6. P16 (`pending` の不変条件)

**言明** --- 走査中の各位置において、`pending` は次を満たす。(a) `pending[k]` の各要素の `node` は、その
位置までに訪れた `Retain` 節点であり、その `unit_key` は `k` である。(b) 各要素の `outstanding` は空で
ない。(c) 1 つの `Retain` 節点は `pending` 全体で高々 1 か所に現れる。(d) `pending[k]` の並びは、訪れた
順である (後ろほど新しい)。(e) `pending` から取り除かれた `Retain` は、`outstanding` が空になったか、
`needed_retains` に入ったかのいずれかである。

(e) を証明できる形にすると次になる。**各除去事象 (DEF 除去事象) について、次の 3 つのいずれかが成り立つ。
(e1) その事象は「引き」であり、取り除かれた要素の `outstanding` はその事象の直前に空になった。
(e2) その事象は、取り除かれた要素の `node` を `self.needed_retains` に入れる。
(e3) その事象は「併合」であり、各アームの状態の鎖の中に、同じ `node` の除去事象がある。
そして (e3) の展開は有限で終わり、その葉は (e1)、(e2)、またはアームが 0 個の `Match` の「併合」の
いずれかである。** (e3) が要る理由は第 10 節の注記 1 に、第 3 の葉については注記 5 に書く。

**証明**

<1>1. (a) が成り立つ。
  BY L7 の (ii), DEF INV, DEF 節点の量 (由来 `t` の `key(t)` は `self.unit_key` の値である)
<1>2. (b) が成り立つ。
  BY L7 の (iii), DEF INV
<1>3. (c) が成り立つ。
  BY L7 の (iv), DEF INV
<1>4. (d) が成り立つ。
  BY L7 の (v), DEF INV
<1>5. 除去事象を起こしうる基本操作は「鍵除去」「引き」「併合」の 3 つだけである。
  <2>1. 「初期」は前の状態を持たないので、除去事象ではない。
    BY DEF 除去事象, DEF 基本操作
  <2>2. 「複製」で作られた状態は元の状態と等しいので、要素を失わない。
    BY DEF 基本操作 (`pending.clone()`)
  <2>3. 「追加」は `Vec` の末尾に要素を 1 つ加えるだけで、要素を取り除かない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
  <2>4. QED
    BY <2>1, <2>2, <2>3, DEF 基本操作
<1>6. CASE 除去事象が「鍵除去」である。L4 より、この操作は取り除いた `Vec` の各要素の `node` を
      `self.needed_retains` に入れる。よって (e2) が成り立つ。
  BY L4
<1>7. CASE 除去事象が「引き」である。
  <2>1. L7 の (i) より L3 の仮定が満たされる。L3 の 1 と 2 では `pending` が変わらないので、除去事象は
        L3 の 3 の場合に限る。
    BY L3, L7
  <2>2. L3 の 3 で要素が取り除かれるのは `stack.pop()` の 1 か所だけであり、それは
        `innermost.outstanding.subtract(un_bumped)` の後にその `outstanding` が空になったときに限る。
    BY L3
  <2>3. L3 の 3 の `pending.remove(key)` は、`Vec` が空になったときに鍵を取り除くだけであり、そのとき
        その `Vec` には要素が無いので、要素を取り除かない。
    BY L3
  <2>4. QED (e1) が成り立つ。
    BY <2>1, <2>2, <2>3
<1>8. CASE 除去事象が「併合」である。取り除かれた要素を `e` とする。
  <2>1. L7 の (iv) を各 `arm_exits[j]` に適用すると L5 の仮定が満たされる。L5 の 5 より、`pending_in` の
        要素 `e` が `merged` に入らないのは、`uniform` が `e.node` を鍵に持たないとき、かつそのときに
        限る。
    BY L5, L7
  <2>2. CASE ある `j` について `arm_states[j]` が `e.node` を鍵に持つ。このとき <2>1 と L5 の 4 より、
        この `merge` の呼び出しは `e.node` を `self.needed_retains` に入れる。すなわち (e2) が成り立つ。
    BY L5, <2>1
  <2>3. CASE どの `j` についても `arm_states[j]` が `e.node` を鍵に持たない。
    <3>1. L5 の 1 より、`e.node` はどの `arm_exits[j]` の要素の `node` でもない。
      BY L5, <2>1
    <3>2. 各アーム `j` の入口状態は `pending_in` の「複製」なので、`e.node` を `node` とする要素を持つ。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
         DEF 基本操作
    <3>3. L6 より `arm_exits[j] = pending_out(arm_j.body)` はアーム `j` の入口状態から基本操作の有限列
          (状態の鎖) で得られる。<3>2 でその鎖の最初の状態は `e.node` の要素を持ち、<3>1 でその鎖の
          最後の状態は持たない。よってその鎖の中に、`e.node` の要素を持つ状態から持たない状態を作る
          操作、すなわち `e.node` の除去事象がある。
      BY L6, <3>1, <3>2, DEF 除去事象
    <3>4. QED (e3) が成り立つ。
      BY <3>3
  <2>4. QED
    BY <2>2, <2>3
<1>9. (e3) の展開は有限で終わる。その葉は (e1)、(e2)、またはアームが 0 個の `Match` の「併合」の
      いずれかである。
  <2>1. (e3) が指す各除去事象は、`Match` 節点 `n` のアーム `j` の走査の中で起きる。その走査は `n` の
        訪問の中で `self.merge` の呼び出しより前に完了しているので、そこで作られる状態は `merged` より
        前に作られている。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       DEF 基本操作
  <2>2. よって (e3) の展開は、状態の生成順序について真に減少する。生成順序は有限の全順序なので、展開は
        有限回で終わる。
    BY <2>1, DEF 基本操作
  <2>3. QED
    展開が止まるのは、その除去事象が (e1) か (e2) であるとき、または (e3) が指す除去事象が 1 つも無い
    ときである。<1>8 より (e3) はアームごとに 1 つの除去事象を指すので、後者はアームが 0 個のときに
    限る。除去事象が (e1)、(e2)、(e3) のいずれかであることは <1>6、<1>7、<1>8 が与える。
    BY <2>1, <2>2, <1>6, <1>7, <1>8
<1>10. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9

## 7. P17 (`un_bump` の正しさ)

**言明** --- `un_bump(pending, k, R)` の返り値は次で決まる。`pending` にキー `k` の項目が無ければ
`NoBracket` で、`pending` は変わらない。あって、最内の要素の `outstanding` が `R` を `covers` しなければ
`OutsideBracket` で、`pending` は変わらない。covers すれば `InBracket(t)` で、`t` は最内の要素の `node`
であり、その要素の `outstanding` から `R` が引かれ、空になればその要素が取り除かれ、スタックが空になれば
キーが取り除かれる。

ここで「最内の要素」とは、鍵 `k` の `Vec` の最後の要素である。P16 の (d) より、それは鍵 `k` の要素のうち
最も後に訪問された `Retain` 節点を由来に持つ。

**証明**

<1>1. `un_bump` を呼ぶのは `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕 1 か所だけであり、その
      第 1 引数は走査の状態である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: un_bump
<1>2. L7 の (i) より、その状態が鍵 `k` を持つならその `Vec` は空でない。すなわち L3 の仮定が満たされる。
  BY L7, <1>1
<1>3. `Vec` の最後の要素を `last_mut` が返すので、L3 の 2 と 3 の「最後の要素」は言明の「最内の要素」で
      ある。
  BY CODE src/rc_ir/borrow.rs: un_bump, L3
<1>4. QED
  L3 の 3 つの場合は、言明の 3 つの場合と条件も結論も一致する。L3 の 3 の「`e.outstanding` が
  `e.outstanding - un_bumped` になる」は、言明の「`outstanding` から `R` が引かれ」である。
  BY L3, <1>2, <1>3, DEF 参照の多重集合

## 8. P18 (`merge` の後に残るもの)

**言明** --- `merge` の返す `pending` に残る `Retain` は、`pending_in` に在り、かつすべてのアームの出口に
同じ `outstanding` で現れるものだけである。いずれかのアームの出口に現れてこの条件を満たさない `Retain` は
`needed_retains` に入る。どのアームの出口にも現れない `Retain` は、`needed_retains` にも返り値にも
入らない。

第 3 の主張の「入らない」は、「この `merge` の呼び出しがそれを入れない」と読む。`needed_retains` は走査の
間ずっと残る集合なので、以前の訪問がすでに入れた `NodeId` はそこに在りうる。

**証明**

<1>1. L7 の (iv) を各 `arm_exits[j]` に適用すると、L5 の仮定が満たされる。以下 L5 の 1 から 5 を使う。
  BY L7, L5
<1>2. `merged` の要素の `node` の集合は、呼び出しの終わりの `uniform` の鍵の集合に等しい。
  <2>1. L5 の 5 より、`merged` の要素の `node` は `pending_in` の要素の `node` のうち `uniform` が鍵に
        持つもの全部である。
    BY L5, <1>1
  <2>2. L5 の 3 より、`uniform` の鍵はすべて `entered_with` の要素であり、L5 の 2 より `entered_with` は
        `pending_in` の要素の `node` の集合である。
    BY L5, <1>1
  <2>3. QED
    BY <2>1, <2>2
<1>3. 第 1 の主張が成り立つ。L5 の 3 と <1>2 より、`merged` に残る `node` は、`pending_in` の要素の
      `node` であり、ある `j` について `arm_states[j]` が鍵に持ち、すべての `j'` について
      `arm_states[j']` が鍵に持ってその値が互いに等しいもの、である。L5 の 1 より `arm_states[j]` が
      `x` を鍵に持つことは `arm_exits[j]` に `node` が `x` の要素があることであり、その値はその要素の
      `outstanding` である。
  BY L5, <1>1, <1>2
<1>4. CASE `arm_exits` が空でない。このとき、「ある `j` について `arm_states[j]` が鍵に持つ」は
      「すべての `j'` について `arm_states[j']` が鍵に持つ」から従う。よって <1>3 の条件は、言明の
      「すべてのアームの出口に同じ `outstanding` で現れる」と一致する。
  BY <1>3
<1>5. CASE `arm_exits` が空である。このとき `arm_states` も空なので、二重ループの本体は 1 度も実行されず、
      `uniform` は鍵を持たない。<1>2 より `merged` は要素を持たない。言明の「すべてのアームの出口に
      同じ `outstanding` で現れる」は空虚に真なので、この場合には <1>3 の条件のうち「ある `j` について
      `arm_states[j]` が鍵に持つ」が効いている。
  BY L5, <1>2, <1>3, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>6. 第 2 の主張が成り立つ。`x` がいずれかの `arm_exits[j]` の要素の `node` であり、<1>3 の条件を
      満たさないとする。L5 の 1 より `x` はその `arm_states[j]` の鍵であり、<1>3 の条件を満たさないので
      L5 の 3 より `uniform` は `x` を鍵に持たない。よって L5 の 4 より、この呼び出しは `x` を
      `self.needed_retains` に入れる。
  BY L5, <1>1, <1>3
<1>7. 第 3 の主張が成り立つ。`x` がどの `arm_exits[j]` の要素の `node` でもないとする。L5 の 1 より `x`
      はどの `arm_states[j]` の鍵でもない。よって L5 の 4 より、この呼び出しは `x` を
      `self.needed_retains` に入れない。また L5 の 3 より `uniform` は `x` を鍵に持たないので、<1>2 より
      `x` は `merged` の要素の `node` ではない。
  BY L5, <1>1, <1>2
<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

## 9. 層 4 へ渡す補題

次の 3 つは P15 - P18 の証明には使わないが、`cancel` の走査の性質なのでここで示す。P19 と P23 がこれらを
使う。

### L8 (記録は増えるだけ)

走査の実行中、`self.needed_retains` は要素を失わず、`self.all_retains` は要素を失わず、
`self.un_bump_releases` は鍵を失わず、その各値の `Vec` も要素を失わない。また、`Retain` 節点 `t` の訪問の
後、走査が終わるまで、`node_id(t)` は `self.all_retains` の要素であり、`self.un_bump_releases` は
`node_id(t)` を鍵に持つ。

**証明**

<1>1. `self.needed_retains` に触れるのは、`walk_inner` の `RcExpr::Ret(_)` の腕、`consume_unit`、
      `merge` の 3 か所であり、どれも `insert` だけである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
<1>2. `self.all_retains` に触れるのは、`walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕の
      `self.all_retains.push(retain)` だけである。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
<1>3. `self.un_bump_releases` に触れるのは、`walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕の
      `self.un_bump_releases.entry(retain).or_default()` と、`RcExpr::Release(v, path, _, k)` の腕の
      `self.un_bump_releases.entry(retain).or_default().push(node_id(node))` の 2 か所だけである。
      どちらも鍵を取り除かず、値の `Vec` から要素を取り除かない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕
<1>4. `Retain` 節点 `t` の訪問は `self.all_retains.push(node_id(t))` と
      `self.un_bump_releases.entry(node_id(t)).or_default()` を実行する。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: node_id
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L9 (訪問順序は実行路の順序を含む)

実行路 `p` (D3) の上で節点 `m` が節点 `n` より真に前にあるならば、走査は `m` を `n` より前に訪問する。

**証明**

<1>1. 「`p` の上で真に前」は「`p` の上で直後」の推移閉包である。よって、`p` の上で `m'` が `m` の直後に
      あるすべての対について「走査は `m` を `m'` より前に訪問する」を示せば足りる。
  BY D3 (実行路は節点の有限列である)
<1>2. CASE `m` の式が `RcExpr::Let(_, RcRhs::Match(_, arms), k)` である。
  <2>1. D3 より、`p` の上の `m` の直後の節点 `m'` は、`p` が選んだアーム `arm_i` の本体 `arm_i.body` で
        ある。
    BY D3
  <2>2. `m` の訪問は `self.walk(&arm_i.body, pending.clone(), false)` を呼び、その呼び出しの中で
        `arm_i.body` が訪問される。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
       L1
  <2>3. QED
    BY <2>1, <2>2
<1>3. CASE `m` の式が `RcExpr::Retain(..)`、`RcExpr::Release(..)`、`RcExpr::Destructure(..)`、
      `RcExpr::Eval(..)`、または `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でないもの、の
      いずれかである。
  <2>1. D3 より、`p` の上の `m` の直後の節点は `m` の継続 `k` である。
    BY D3
  <2>2. `m` の訪問は `self.walk(k, ・, ・)` を呼び、その呼び出しの中で `k` が訪問される。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Eval(_, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, L1
  <2>3. QED
    BY <2>1, <2>2
<1>4. CASE `m` の式が `RcExpr::Ret(_)` であり、`p` の上に `m` の直後の節点 `m'` がある。
  <2>1. D3 より、`Ret` の後に実行路が続くのは、`m` が、その実行路が入ったアームの本体の実行路を終える
        `Ret` であるときに限る。そのアームを `arm_i`、その `Match` 節点を `M` とすると、
        `m = ret(arm_i.body)` であり、`m'` は `M` の継続 `k_M` である。
    BY D3, DEF 継続終端
  <2>2. `M` の訪問は、まず `arms` の各アームについて `self.walk(&arm.body, pending.clone(), false)` を
        呼び、それらが返った後で `self.merge` を呼び、その後で
        `self.walk(k_M, merged, returns_from_func)` を呼ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕
  <2>3. `m` は `N(arm_i.body)` の要素なので、<2>2 の `self.walk(&arm_i.body, ・, ・)` の呼び出しの中で
        訪問される。`k_M` は <2>2 の `self.walk(k_M, ・, ・)` の呼び出しの中で訪問される。前者は後者
        より前に完了する。
    BY P15, DEF 部分木, <2>2
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>5. QED
  <1>2、<1>3、<1>4 は、`p` の上に直後の節点がある場合を尽くす。`RcExpr` の 6 変位のうち `Ret` 以外の
  5 つを <1>2 と <1>3 が尽くし、`Ret` を <1>4 が扱う。
  BY <1>1, <1>2, <1>3, <1>4, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L10 (`OutsideBracket` の後始末)

`un_bump` が `OutsideBracket` を返したとき、`walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、鍵
`key` の `Vec` を丸ごと取り除き、その各要素の `node` を `self.needed_retains` に入れる。`un_bump` が
調べるのはその `Vec` の最後の要素だけだが、`needed_retains` に入るのはその `Vec` の全要素である。

**証明**

<1>1. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕の `match un_bump(...)` の
      `UnBump::OutsideBracket` の枝は `self.consume_unit(&mut pending, key)` を実行する。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕
<1>2. L4 より、`consume_unit(pending, key)` は鍵 `key` とその `Vec` を丸ごと取り除き、取り除いた `Vec` の
      各要素の `node` を `self.needed_retains` に入れる。
  BY L4
<1>3. L3 の 2 より、`un_bump` が `OutsideBracket` を返すとき `pending` は変わらないので、<1>2 が取り除く
      `Vec` は `un_bump` が見たものと同じである。`un_bump` が `covers` を検査するのはその `Vec` の最後の
      要素だけである。
  BY L3
<1>4. QED
  BY <1>1, <1>2, <1>3

## 10. 言明についての注記

**注記 1 (P16 の (e) に第 3 の場合が要ること)**。(e) を「取り除かれた要素の `outstanding` がその時点で
空である」か「取り除かれた要素の `node` がその時点で `needed_retains` に入っている」かの二択にすると、
次の形で偽になる。`Retain` 節点 `t` が鍵 `k` の下で `pending` に入り、その後の `Match` の各アームが `t` を
完全に un-bump する (各アームの中の `Release` が L3 の 3 で `t` の `outstanding` を空にする) 場合である。
このとき `arm_exits` のどれも `t` の要素を持たないので、L5 の 3 と 4 より `merge` は `t` を `uniform` にも
`needed_retains` にも入れず、L5 の 5 より `merged` にも入れない。この「併合」は `pending_in` の要素の
除去事象だが、その要素の `outstanding` は (`pending_in` の中では減っていないので) 空でなく、`t` は
`needed_retains` にも入っていない。P16 の (e3) は、この除去がどこで解消されたか --- 各アームの状態の鎖の
中の除去事象 --- を名指す。この展開が終わる先は P16 の <1>9 が示す。

**注記 2 (P17 の `OutsideBracket` の条件)**。`un_bump` が検査するのは
`innermost.outstanding.covers(un_bumped)` であって、最内の要素の由来である `Retain` 節点が作った参照
(`bumped`) と `un_bumped` の関係ではない。この 2 つは食い違いうる。ある `Release` が L3 の 3 で最内の要素の
`outstanding` を減らした後、`bumped` は覆うが `outstanding` は覆わない `un_bumped` を持つ `Release` が
来ると、`covers` は偽になる。よって `OutsideBracket` の条件は `outstanding` で述べる必要がある。

また `un_bump` は鍵 `k` の `Vec` の最後の要素しか調べないので、より外側の要素の `outstanding` が
`un_bumped` を覆っていても `OutsideBracket` を返す。この場合の後始末は L10 が述べる。`un_bump` が
調べなかった外側の要素も `needed_retains` に入るので、`un_bump` が見なかった要素の上に打ち消しが立つことは
ない。

**注記 3 (P18 の第 1 の主張と、アームが 0 個の `Match`)**。P18 の第 1 の主張の「すべてのアームの出口に
同じ `outstanding` で現れる」は、`arms` が空のとき空虚に真になるが、そのとき `merged` は空である
(P18 の <1>5)。正確な条件は「**ある**アームの出口に現れ、かつ**すべての**アームの出口に同じ `outstanding`
で現れる」である (P18 の <1>3)。アームが 1 つ以上あればこの 2 つは一致する。`validate` はアームが 0 個の
`Match` を拒む (`CODE src/rc_ir/validate.rs: check_rhs` の `RcRhs::Match(scrutinee, arms)` の腕) が、
`validate` は `config.develop_mode` のときだけ走る (`CODE src/build/build_object_files.rs:
optimize_rc_program`) ので、P18 はこの場合を含めて述べてある。

**注記 4 (P15 の前半が何の性質か)**。D2 が述べるとおり `RcExprNode` は式を `Arc` で共有できるので、1 つの
木の相異なる位置が同じ `NodeId` を持つ木は作れる。P15 の前半が成り立つのは、`cancel` に渡される木が
`RewriteCtx::rewrite` の出力だからである (P15 の 3.1 の <1>3 と <1>4)。`rewrite` は入力の節点を出力に
そのまま置かず、出力の各位置に `expr_node` で新しい割り当てを作る。したがって P15 の前半は
`borrow_ify` が保つべき性質であり、`borrow_ify` の実装を変えるときはこの性質を壊さないことを確かめる
必要がある。

**注記 5 (アームが 0 個の `Match` と、P16 の (e) の届く範囲)**。`arms` が空のとき、`merge` は
`pending_in` の全要素を落とし、`needed_retains` には何も入れない (P18 の <1>5)。この「併合」は除去事象で
あり、P16 の (e3) は「各アームの状態の鎖の中に除去事象がある」を空虚に満たすので、展開はそこで止まり、
(e1) にも (e2) にも行き着かない。すなわち P16 の (e) はこの場合に何も言わない。D3 の実行路もこの `Match`
を越えないので、この `Match` の継続を含む実行路は無く、その本体には終端の `Ret` に至る実行路が 1 本も無い。
層 4 が `cancelled()` の返す集合を実行路の上で論じるときは、この形が入力に無いことが要る。それを
`validate` の検査 (注記 3) より前に置くなら、「すべての `Match` はアームを 1 つ以上持つ」を lowering が
果たす仮定として README に置くのが素直である。

**注記 6 (`check_one_key_per_object` が見ない `Retain`)**。`walk_inner` の
`RcExpr::Release(v, path, _, k)` の腕は、`others(r)` の各鍵について `consume_unit` を呼んだ**後**に
`check_one_key_per_object` を呼ぶ (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の
`RcExpr::Release(v, path, _, k)` の腕)。L4 よりその `consume_unit` は鍵ごと要素を取り除くので、
`others(r)` の鍵の下に在った `Retain` は検査の対象にならない。それらの `node` は `needed_retains` に
入っているので (L4)、打ち消しの対象からは外れている。

**注記 7 (層 4 が必要とする形)**。P19 は実行路で量化した言明である。P16 は実行路を量化しないので、P19 を
示すには次の 2 つを繋ぐ必要がある。第 1 に、L9 (訪問順序は実行路の順序を含む) が、走査が状態を作る順序と
実行路の順序を繋ぐ。第 2 に、P16 の (e) の (e3) の展開が、`Match` のアームごとに分かれるので、「すべての
実行路について」の場合分けの構造をそのまま与える。1 つの実行路は各 `Match` でアームを 1 つ選ぶので、
(e3) の展開のうちその選択に沿った枝が、その実行路の上でどこで解消されたかを名指す。
