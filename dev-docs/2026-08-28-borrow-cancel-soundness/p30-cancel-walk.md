# P15 - P18: `cancel` の走査

この文書は README の層 3 の 4 命題 P15, P16, P17, P18 を証明する。README の定義 D1 - D16 と仮定
A1 - A8 の上に立つ。層 1 と層 2 の命題は引用しない。

## 0. この文書が使う記法

README の §2 の記法に、次の 2 つを加える。

- **局所の定義**。この文書の中だけで使う語を §1 で定める。`BY` の行では `DEF <名前>` で引用する。
- **局所の補題**。この文書の中だけで使う補題を `L1` - `L8` と番号を付けて述べ、`BY` の行では
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
- `key(r) := self.unit_key(&v.name, path)`、`dropped(r) := self.acted_references(v, path)`
- `others(r) := self.acted_unit_keys(&v.name, path)` の要素のうち `key(r)` と異なるもの

`CancelAnalysis::unit_key` は `ownership::unit_key` を、`CancelAnalysis::acted_unit_keys` は
`ownership::acted_unit_keys` を、`CancelAnalysis::acted_references` は `ownership::acted_references` を、
それぞれ `self.vars` と `self.type_env` を渡して呼ぶ (`CODE src/rc_ir/borrow.rs: CancelAnalysis::unit_key`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_unit_keys`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。D15 の 3 つの関数はどれも `vars`、`τenv`、
および渡された `(変数, path)` だけから値が決まるので、上の 4 つの量は走査のどの時点で読んでも同じ値である。

### DEF 参照の多重集合

`References` は `Map<VarPath, usize>` を 1 つ持つ構造体である (`CODE src/rc_ir/ownership.rs: References`)。
これを、鍵をオブジェクトの名前、値をその個数とする多重集合とみなす。多重集合の差 `R1 - R2` を各オブジェクトの
個数の差とする (`R1.covers(&R2)` が成り立つときだけ書く)。**空**とは、参照を 1 つも持たないことをいう。

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

状態 `P` から基本操作 1 つで状態 `P'` が作られ、`P` に `node` が `x` である要素があり、`P'` にはそれが無い
とき、この操作を `x` の**除去事象**と呼ぶ。

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
  BY DEF 割り当て (Rust の 1 つの割り当ては、生存している間、他の生存する割り当てと記憶域を共有しない)
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
      `analysis.walk(body, PendingRetains::default(), true)` と、`walk_inner` の 6 つの腕の中の 8 か所
      だけである。
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
