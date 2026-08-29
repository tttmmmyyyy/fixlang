# P19 - P24: `cancel` が RC 規律を保存すること

この文書は README の層 4 の 6 命題 P19, P20, P21, P22, P23, P24 を証明する。README の定義 D1 - D27、
仮定 A1 - A19、および命題 P1 - P18b の**言明**の上に立つ。主定理 T は `p60-main-theorem.md` の担当であり、
この文書は扱わない。

対象コミットは `7a9826ed` である。

引用する外部の補題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF N` (`N_ρ`)、`DEF ρ-歩みと ρ-終端`、`DEF 別名類` (`obj(C)`)、
`DEF 類ごとの参照` (`held_ρ`、`bumps_ρ`) -- と、補題 `L8`、`L9`、`L10a`、`L11`、第 7.5.1 節の `INV`、
第 7.5.4 節の <1>2。これらは `p30 の L10`、`p13 の L11` のようにファイル名を添えて引用する。

この文書が導入する補題は `L30` から番号を付ける。`p30` と `p13` の補題の番号と衝突させないためである。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P19 | 証明済み。`t` の `outstanding` は位置ごとの残量として読む (第 10 節の 差し戻し 1) |
| P20 | 証明済み |
| P21 | 証明済み。ただし入力の D12 (差し戻し 5) と、2 つの実行が同じアームを選ぶこと (`DEF 対応する活性化`、差し戻し 4) を前提に置く |
| P22 | 証明済み |
| P23 | 証明済み。(S-a) は L42 に立ち、L42 は README に無い言明である (差し戻し 3)。P21 と同じ前提を継ぐ |
| P24 | 証明済み |

**README に足りないものが 4 つ出た。** 第 10 節が述べる。要点だけ先に書く。

- **P18a はオブジェクトごとの形が要る。** README の P18a は名前 `o` ごとに `H(o) ≥ n(o) + 1` を言うが、
  1 つのオブジェクトを指す活性な名前は複数ありうるので、P21 が要る `H(O) ≥ Σ_o n(o) + 1` はそこから
  出ない。この形は `p13-disposals-and-pending.md` の第 7.5.1 節の `INV` そのものであり、同ファイルの
  第 7.5.4 節が A19 から示している。この文書は L41 でそれを引用する。
- **義務集合の側の言明が要る。** D11 の (S-a) は `Obl` についての条件であり、`H` についての A19 (i) からは
  出ない。A19 (ii) と `p13` の `DEF 類ごとの参照` から `Obl(τ, O) ≥ Σ_C bumps_ρ(τ, C)` が出る。L42 が
  それである。
- **2 つの実行が同じアームを選ぶことは前提である。** `unsafe_is_unique` (D18) は参照カウントを読むので、
  削除がカウントを下げた位置で観測値が倒れうる。倒れれば 2 つの実行は別のアームへ入り、位置の対応が
  切れる。
- **P21 の言明に入力の D12 が要る。** 解放されたオブジェクトのカウントが上がらないことを使うので、
  入力が D11 の (S-c) を満たすことが要る。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の本体 `B` を固定し、`B` から作られる `VarTable` を `vars`、
プログラムの `TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`,
`CODE src/rc_ir/ownership.rs: VarTable::body_only`)。この 2 つは本体ごとに 1 つなので、以下では
`origin` と `acted_references` の第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。
- `id(x, π)` は `origin(x, π).identity()` (`CODE src/rc_ir/ownership.rs: Origin::identity`)。
- `acted_on(x, π)` は `origin(x, π).acted_on()` の元の集合
  (`CODE src/rc_ir/ownership.rs: Origin::acted_on`)。D15 より
  `acted_on(x, π) = {id(x, π)} ∪ origin(x, π).candidates()` である。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より、これが
  「`v` の `π` の下の boxed leaf」の全体であり、inhabited (D16) でないものを含む。

**2 つの「オブジェクト」を書き分ける。** D15 と `References` が「オブジェクト」と呼ぶのは `VarPath` の
値であり、この文書ではこれを**名前**と呼び、`o` で表す。D7 が「オブジェクト」と呼ぶのは実行時のヒープの
オブジェクトであり、この文書ではこれを**オブジェクト**と呼び、`O` で表す。`p13` の `DEF 名前の活性` が
定める `obj_ρ(o)` が、活性な名前からオブジェクトへの写像である。

多重集合の記法。`References` は `Map<VarPath, usize>` を 1 つ持つ構造体であり
(`CODE src/rc_ir/ownership.rs: References`)、これを鍵を名前、値をその個数とする多重集合とみなす。
和 `R1 + R2`、差 `R1 - R2` を各鍵の個数の和・差とし、`R[o]` を名前 `o` の個数とする。

## 2. 局所の定義

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。節点 `n` の訪問における `pending` 引数の値
(その訪問がそれに変更を加える前の値) を `pending(n)` と書き、**入口状態**と呼ぶ。その訪問の戻り値を
`pending_out(n)` と書き、**出口状態**と呼ぶ。P15 の後半より、走査は `B` の各位置をちょうど 1 回訪問するので、
`pending(n)` と `pending_out(n)` は節点ごとに 1 つに定まる。

節点 `n` の**継続終端** `ret(n)` を、`n` から D2 の意味の継続 (`Match` の場合はアーム本体ではなく `k`) を
たどって到達する `Ret` 節点とする。D2 より継続の鎖は有限で `Ret` で終わるので、`ret(n)` は 1 つに定まる。

要素 `e ∈ pending(n)` について、`e.node` を `NodeId` に持つ `Retain` 節点を `e` の**由来**と呼ぶ。P16 の
(a) と P15 の前半より、由来はちょうど 1 つに定まる。`Retain` 節点 `t` が `pending(n)` の要素の由来である
とき、「`t` は `n` で pending である」といい、その要素を `e_t(n)`、その `outstanding` を `out(t, n)` と
書く。P16 の (c) より `t` に対応する要素は高々 1 つである。

### DEF 子と親

節点 `n` の**子**を次で定める。

| `n` の式 | 子 |
|---|---|
| `Let(_, Match(_, arms), k)` | `arms` の各 `arm.body`、および `k` |
| `Let(_, rhs, k)` (`rhs` は `Match` でない) | `k` |
| `Retain(_, _, _, k)` | `k` |
| `Release(_, _, _, k)` | `k` |
| `Destructure(_, _, _, k)` | `k` |
| `Eval(_, k)` | `k` |
| `Ret(_)` | 無し |

`n` を子に持つ節点を `n` の**親**と呼ぶ。節点 `n` の**部分木** `N(n)` を、`n` と、`n` の各子 `c` に
ついての `N(c)` との合併とする。D2 より本体は有限の木であり、位置が相異なれば節点も相異なるので、根で
ない各節点はちょうど 1 つの親を持ち、`n` はどの子の部分木にも入らない。

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、

- `ActRefs(t) :=` `self.acted_references(v, path)` の値、`ActRefs(r) :=` `self.acted_references(v, path)`
  の値 (`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。D15 より、これは
  `acted_references(vars, type_env, v, path)` すなわち `L(v, path)` の各 leaf を `id` で名付けて数えた
  多重集合である。
- `others(r) :=` `self.other_objects(v, path)` の値の元の集合
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。

`p30` の `DEF 節点の量` が、この 3 つの値が走査のどの時点で読んでも同じであることを示している。

### DEF 削除集合

`cancel_body` の 1 回の実行について、`analysis.cancelled()` が返す集合を `Del` と書く
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled`)。
`self.all_retains` の要素のうち、`self.needed_retains` に入らず、`self.un_bump_releases` の値が空でない
ものの全体を `CT` と書く。走査の終わりの `self.un_bump_releases` を `un_bump_releases` と書く。

`NodeId` は節点を一意に決める (P15 の前半) ので、以下では `Del`、`CT`、`un_bump_releases` の要素を節点と
同一視する。

### DEF 出力の本体

`B` に対する `cancel_body` の返り値 `drop_nodes(B, Del)` を `B'` と書く
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: drop_nodes`)。

### DEF 路の対応

L30 が示すとおり `B'` は `B` から `Del` の節点を抜いた木である。`B` の実行路 `ρ` (D3) から `Del` の節点を
除いた列を `ρ'` と書き、`ρ` に**対応する** `B'` の実行路と呼ぶ。L31 が、これが `B'` の実行路であることと、
対応が全単射であることを示す。以下、位置についての主張は、`ρ` と `ρ'` を、対応する節点で並べて比べる。

### DEF 対応する活性化

`ρ` を `B` の実行路、`ρ'` をそれに対応する `B'` の実行路とする。`α` を `ρ` を辿る `B` の 1 回の活性化
(D21)、`α'` を `ρ'` を辿る `B'` の 1 回の活性化とし、次の 2 つを満たすとき、この 2 つを**対応する活性化**と
呼ぶ。

- **(C1)** `α` と `α'` は同じ入力から始まる。すなわち、パラメータと capture に同じ値を受け取り、開始時の
  ヒープが同じである。
- **(C2)** 対応する各一意性の観測点 (D18) -- `observes_uniqueness()` が真である `LLVMGen` を持つ `Llvm`
  節点 (`CODE src/ast/inline_llvm.rs: LLVMGen::observes_uniqueness`) -- で、2 つの活性化の観測値が
  等しい。

(C2) が要る理由と、それを果たす者が誰も居ないことは、第 10 節の 差し戻し 4 が述べる。

### DEF 実行時の量

対応する活性化 `α`、`α'` を固定する。`ρ` の上の節点 `q` について、

- `H(q, O)` を、`α` が `q` の訪問に対応する実行の段に入る**直前**のオブジェクト `O` の参照カウント (D7)、
  `Obl(q, O)` を、その時点の義務集合 (D10) が持つ `O` への参照の個数とする。
- `H'(q, O)`、`Obl'(q, O)` を、`α'` について `ρ'` の対応する位置で同じように定める。`q ∈ Del` のときは、
  `ρ'` にその節点は無いので、`ρ'` の上で `q` の直後にあたる位置の値を取る。

`ρ` の上の位置とは、この意味で節点の実行の直前の時点をいう。これに、`ρ` の終端の `Ret` の消費を行った後の
時点を 1 つ加え、これも位置と呼ぶ。D3 より `ρ` は有限の列なので、位置は有限個である。`d` と `H` と `Obl` は
この最後の位置でも定まる。

### DEF 欠損

対応する活性化を固定する。`ρ` の上の位置 `q` と計数下 (D26) のオブジェクト `O` について、

`d(q, O) :=` (`q` より前に実行された `Del` の `Retain` 節点が `O` への参照を作った個数)
`-` (`q` より前に実行された `Del` の `Release` 節点が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。

### DEF 消費点

D9 の消費の表の行が指す位置を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

## 3. 局所の補題

### L30 (`drop_nodes` の作用)

**言明** --- `S` を `NodeId` の集合とする。`drop_nodes(B, S)` は、`B` の木から、`NodeId` が `S` に入る
`Retain` 節点と `Release` 節点だけを取り除いた木を返す。残る各位置の式の変位、変数、path、`RcState`、
source、`Match` のアームの本数と並び、および継続の順序は変わらない。

**証明**

<1>1. `drop_nodes(node, to_delete)` は `grow_stack(|| drop_nodes_inner(node, to_delete))` であり、A15 より
      `drop_nodes_inner` をちょうど 1 回呼んでその値を返す。
  BY CODE src/rc_ir/borrow.rs: drop_nodes, A15
<1>2. `drop_nodes` が読む `NodeId` は入力の木のものであり、走査が読んだものと同じである。`cancel_body` は
      `drop_nodes(body, &analysis.cancelled())` を呼び、その `body` は走査が訪問した木そのものである。
      `cancel` の閉包の実行中、`body` は `prog` から借用されているので、その木の `Arc` の割り当てはすべて
      生存しており、`node_id` の値は走査のときと同じである。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: node_id
<1>3. 木 `N(node)` の構造についての帰納法で、`drop_nodes_inner(node, to_delete)` が言明のとおりの木を返す
      ことを示す。DEF 子と親 より子は真の部分木なので整礎である。
  <2>1. CASE `node` の式が `RcExpr::Retain(v, path, state, k)` である。この腕は `drop_nodes(k, to_delete)`
        を 1 回呼び、`to_delete` が `node_id(node)` を含むときはその値をそのまま返し、含まないときは
        `RcExpr::Retain(v.clone(), path.clone(), *state, k)` の節点を `&node.source` を付けて積んで返す。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Retain(v, path, state, k)` の腕
  <2>2. CASE `node` の式が `RcExpr::Release(v, path, state, k)` である。この腕は <2>1 と同じ形で、
        `RcExpr::Release(v.clone(), path.clone(), *state, k)` を積む。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Release(v, path, state, k)` の腕
  <2>3. CASE `node` の式が `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` である。この腕は各アームに
        ついて `drop_nodes(&arm.body, to_delete)` を 1 回、`drop_nodes(k, to_delete)` を 1 回呼び、
        `x`、`scrut`、各アームの `tag`/`payload`/`payload_state`、アームの本数と並びを変えずに節点を
        積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body
  <2>4. CASE `node` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
        `drop_nodes(k, to_delete)` を 1 回呼び、`x` と `rhs` を変えずに節点を積む。`to_delete` の検査は
        しない。`match` の腕はこの順に並んでいるので、この腕に落ちる `rhs` は `RcRhs::Match` ではない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, rhs, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner
  <2>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)` または `RcExpr::Eval(v, k)`
        である。この 2 つの腕は `drop_nodes(k, to_delete)` を 1 回呼び、他のフィールドを変えずに節点を
        積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Eval(v, k)` の腕
  <2>6. CASE `node` の式が `RcExpr::Ret(v)` である。この腕は `v` を変えずに 1 節点を作って返す。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Ret(v)` の腕
  <2>7. QED
    `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <2>1 から <2>6 が尽くす。これは
    `drop_nodes_inner` の `match` の 7 つの腕である。節点を落とすのは <2>1 と <2>2 だけであり、どちらも
    `Retain`/`Release` である。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner
<1>4. QED
  BY <1>1, <1>2, <1>3

### L31 (路の対応)

**言明** --- `Del` の要素がすべて `Retain` 節点か `Release` 節点であるとする。`B` の実行路 (D3) から
`Del` の節点を除く写像は、`B` の実行路の全体から `B'` の実行路の全体への全単射である。

**証明**

<1>1. `B'` の木は `B` の木から `Del` の `Retain`/`Release` 節点を抜いたものであり、`Match` のアームの
      本数と並び、および各節点の継続の順序は変わらない。
  BY L30, 本補題の仮定
<1>2. D3 の実行路は、根から継続をたどり、各 `Match` でアームを 1 つ選ぶことで決まる。<1>1 より `B` と
      `B'` は同じ `Match` 節点の同じアームの並びを持つので、アームの選択の全体は 2 つの木で同じである。
      1 つの選択に `B` の実行路 1 本と `B'` の実行路 1 本が対応し、後者は前者から `Del` の節点を除いた
      列である。
  BY D3, <1>1
<1>3. QED
  BY <1>1, <1>2

### L32 (削除集合の構造)

**言明** --- 次の 5 つが成り立つ。

1. `CT` の各要素は、走査が訪問した `Retain` 節点である。
2. `t ∈ CT` について、`un_bump_releases[t]` の各要素は、走査が訪問した `Release` 節点である。
3. `Del` は `CT` と `⋃_{t ∈ CT} un_bump_releases[t]` の非交和であり、後者の族も互いに素である。
4. `Release` 節点 `r` が `un_bump_releases[t]` に入るのは、`r` の訪問の中の `un_bump` の呼び出しが
   `InBracket(t)` を返したとき、かつそのときに限る。
5. `Del` の要素はすべて `Retain` 節点か `Release` 節点である。

**証明**

<1>1. `cancelled` は `self.all_retains` の各要素 `retain` を回り、`self.needed_retains` がそれを含むときは
      飛ばし、含まないときは `self.un_bump_releases.get(&retain)` を引き、その `Vec` が空でないときだけ
      `retain` とその `Vec` の全要素を `out` に入れる。よって `Del` は DEF 削除集合 の `CT` と、その各
      要素の `un_bump_releases` の値の合併である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, DEF 削除集合
<1>2. `self.all_retains` に値が入るのは、`walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕の
      `self.all_retains.push(retain)` だけであり、そこで `retain = node_id(node)` の `node` はいま訪問して
      いる `Retain` 節点である。よって 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: node_id, <1>1
<1>3. `self.un_bump_releases` の値の `Vec` に要素が入るのは、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の `UnBump::InBracket(retain)` の枝の
      `self.un_bump_releases.entry(retain).or_default().push(node_id(node))` だけであり、そこで `node` は
      いま訪問している `Release` 節点である。よって 2 と 4 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: node_id
<1>4. 1 つの `Release` 節点は、高々 1 つの `t` の `un_bump_releases[t]` に、高々 1 回入る。
  <2>1. P15 の後半より、走査は `Release` 節点 `r` をちょうど 1 回訪問する。
    BY P15
  <2>2. `r` の訪問は `un_bump(&mut pending, &un_bumped)` を 1 回だけ評価し、その値に対する `match` の
        `InBracket` の枝は高々 1 回実行される。`un_bump` の返り値は `UnBump` の 1 つの変位であり、
        `InBracket` は `NodeId` を 1 つだけ運ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: UnBump
  <2>3. QED
    BY <1>3, <2>1, <2>2
<1>5. `CT` の要素と `un_bump_releases[t]` の要素は相異なる節点である。前者は `Retain` 節点、後者は
      `Release` 節点であり、この 2 つは `B` の相異なる位置である。
  BY <1>2, <1>3
<1>6. QED
  3 は <1>1、<1>4、<1>5 から従う。5 は <1>2、<1>3、<1>1 から従う。1 は <1>2、2 は <1>3、4 は <1>3 で
  ある。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L33 (出口状態は継続終端の入口状態である)

**言明** --- 任意の節点 `n` について、`pending_out(n) = pending(ret(n))` である。

**証明** `n` から `ret(n)` への継続の鎖の長さについての帰納法で示す。D2 より鎖は有限である。

<1>1. CASE `n` の式が `RcExpr::Ret(_)` である。`walk_inner` のこの腕は `pending` をそのまま返すので
      `pending_out(n) = pending(n)` であり、DEF 訪問 より `ret(n) = n` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, DEF 訪問
<1>2. CASE `n` の式が `RcExpr::Ret(_)` でない。`walk_inner` の残る 6 つの腕はいずれも
      `self.walk(k, ・, ・)` の値をそのまま返す (`k` は `n` の継続)。`p30` の `L1` より `walk` は
      `walk_inner` を 1 回呼んでその値を返すので、`pending_out(n) = pending_out(k)` である。DEF 訪問 より
      `ret(n) = ret(k)` なので、帰納法の仮定が使える。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, p30 の L1, DEF 訪問
<1>3. QED
  `RcExpr` の 6 変位のうち `Ret` を <1>1 が、残る 5 変位 (`Let` は右辺で 2 つの腕に分かれるが、どちらも
  `self.walk(k, ・, ・)` の値を返す) を <1>2 が尽くす。
  BY <1>1, <1>2, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L34 (路に沿った状態の遷移)

**言明** --- 実行路 `ρ` (D3) の上の節点 `n` と、`ρ` の上で `n` の直後にある節点 `n'` について、次のいずれかが
成り立ち、この 3 つは場合を尽くす。

1. `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` である。
   このとき `n'` は `n` の継続 `k` であり、`pending(n')` は `pending(n)` に、その腕が行う
   `pending.push`、`consume_objects` の呼び出し、`un_bump` の呼び出しを、コードの順に施したものである。
2. `n` の式が `Let(_, Match(_, arms), k)` である。このとき `n'` は `ρ` が選んだアーム `arm_i` の本体で
   あり、`pending(n')` は `pending(n)` の複製、すなわち要素の `node`・`outstanding`・並びが等しい値で
   ある。
3. `n` がアーム本体の終端の `Ret` である。そのアームを `arm_i`、その `Match` 節点を `M` とすると
   `n'` は `M` の継続 `k_M` であり、`pending(n') = merge(pending(M), arm_exits)` の値である。ここで
   `arm_exits[j] = pending_out(arm_j.body)` であり、`arm_exits[i] = pending(n)` である。

**証明**

<1>1. CASE `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` で
      ある。D3 より `ρ` の上の `n` の直後の節点は `n` の継続 `k` である。`walk_inner` のこれらの腕は、
      `pending` に `push`・`consume_objects`・`un_bump` をコードの順に施したうえで
      `self.walk(k, pending, returns_from_func)` を呼ぶので、`pending(k)` はその施した後の値である。
      これらの腕が `pending` に触れる操作はこの 3 つだけである。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, p30 の L1, p30 の L6
<1>2. CASE `n` の式が `Let(_, Match(_, arms), k)` である。D3 より `ρ` の上の `n` の直後の節点は、`ρ` が
      選んだアームの本体である。`walk_inner` のこの腕は各アームについて
      `self.walk(&arm.body, pending.clone(), false)` を呼ぶ。`PendingRetain` は `Clone` を derive し、
      `Vec::clone` は要素をその並びのまま複製するので、アーム本体の入口状態は `pending(n)` と等しい値で
      ある。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     CODE src/rc_ir/borrow.rs: PendingRetain, p30 の L1
<1>3. CASE `n` がアーム本体の終端の `Ret` である。D3 より、`Ret` の後に実行路が続くのは、`n` が、その
      実行路が入ったアームの本体の実行路を終える `Ret` であるときに限り、そのとき直後の節点はその `Match`
      節点 `M` の継続 `k_M` である。`walk_inner` の `M` の腕は `arm_exits` を集めてから
      `let merged = self.merge(&pending, &arm_exits);` を作り、`self.walk(k, merged, returns_from_func)`
      を呼ぶので `pending(k_M) = merged` である。`arm_exits[j]` は `self.walk(&arm_j.body, ・, ・)` の
      返り値、すなわち `pending_out(arm_j.body)` である。L33 より
      `pending_out(arm_i.body) = pending(ret(arm_i.body)) = pending(n)` である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     p30 の L1, L33
<1>4. QED
  `ρ` の上に直後の節点があるのは、`n` の式が `Ret` でないか、`n` がアーム本体の終端の `Ret` であるかの
  どちらかである (D3)。前者を <1>1 と <1>2 が `RcExpr` の変位で尽くし、後者を <1>3 が扱う。
  BY <1>1, <1>2, <1>3, D3, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L35 (`pending` の要素はその位置を支配する)

**言明** --- 走査が訪問する節点 `n` と `pending(n)` の要素 `e` について、`e` の由来 (DEF 訪問) を `t` と
する。このとき、`n` を含むすべての実行路は `t` を含み、その路の上で `t` は `n` より真に前にある。

**証明** 訪問順序についての帰納法で示す。P15 の後半より走査は各位置をちょうど 1 回訪問するので、訪問順序は
有限の全順序であり、この帰納法は整礎である。

<1>1. 帰納法の仮定: `n` より前に訪問された各節点 `m` と `pending(m)` の各要素について、言明が成り立つ。
  BY 帰納法の仮定
<1>2. 木の各節点 `n` について、`n` を含むすべての実行路は、`n` の親 (DEF 子と親) を含み、その路の上で親は
      `n` より真に前にある。
  <2>1. D2 より本体は木であり、根でない各節点はちょうど 1 つの親を持つ。
    BY D2
  <2>2. D3 の規則のうち、実行路に `n` を加えるのは次の 3 つの場合だけである。`n` が根である場合、`n` が
        直前の節点の継続である場合、`n` がアーム本体の根である場合。第 2 の場合の直前の節点は `n` の親
        か、`n` の親が `Match` であるときそのアーム本体の終端の `Ret` であり、後者のときも D3 より
        その `Match` 節点自身が路の上でより前にある。第 3 の場合の直前の節点は `n` の親である `Match`
        節点である。
    BY D3
  <2>3. QED
    BY <2>1, <2>2
<1>3. CASE `n` が `B` の根である。`cancel_body` は `analysis.walk(body, PendingRetains::default(), true)`
      を呼ぶので `pending(n)` は空であり、言明は空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>4. CASE `n` が根でなく、その親 `m` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が
      `Match` でない `Let` である。
  <2>1. `walk_inner` の `m` の腕は `pending(m)` に操作を施して `self.walk(n, pending, ・)` を呼ぶ。
        `pending` に要素を加えるのは `RcExpr::Retain` の腕の `pending.push(PendingRetain { node: retain,
        outstanding })` 1 か所だけであり、そこで加わる要素の `node` は `m` 自身の `NodeId` である。
        `consume_objects` と `un_bump` は要素を加えない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
       CODE src/rc_ir/borrow.rs: un_bump, p30 の L5, p30 の L6
  <2>2. `pending(n)` の要素は、`pending(m)` の要素であるか、由来が `m` 自身 (`m` が `Retain` のとき) で
        ある。
    BY <2>1, L34
  <2>3. QED
    `pending(m)` の要素については <1>1 と <1>2 を合わせる。由来が `m` 自身のものについては <1>2 が直接
    与える。
    BY <1>1, <1>2, <2>2
<1>5. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n` がその
      アームの本体である。L34 の 2 より `pending(n)` の要素の `node` の集合は `pending(m)` のそれに等しい
      ので、<1>1 と <1>2 を合わせる。
  BY <1>1, <1>2, L34
<1>6. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n = k` で
      ある。L34 の 3 より `pending(n) = merge(pending(m), arm_exits)` である。P18 の第 1 の主張より、
      `merge` の返り値に残る `Retain` は `pending_in` すなわち `pending(m)` に在るものだけである。よって
      <1>1 と <1>2 を合わせる。
  BY <1>1, <1>2, L34, P18
<1>7. QED
  `n` は根であるか、親を持つ。親を持つ場合、DEF 子と親 の子の表より、親の式は <1>4、<1>5、<1>6 の
  いずれかの形である。
  BY <1>3, <1>4, <1>5, <1>6, D2

### L36 (`consume_objects` が呼ばれる位置)

**言明** --- 走査が `self.consume_objects(pending, objects)` を呼ぶのは次の 4 か所だけであり、いずれの
呼び出しも、`objects` のいずれかについて `outstanding.names` が真である `pending` の要素をすべて取り除き、
取り除いた各要素の `node` を `self.needed_retains` に入れる。残る要素の値と並びは変わらない。

1. 右辺が `Match` でない `Let(x, rhs, k)` の訪問: `rhs_consumes` が報告する各 `(w, μ)` について
   `objects = acted_on(w, μ)`。
2. `Destructure(c, fs, s, k)` の訪問: `destructure_consumes(c, fs, type_env)` が返す各 `μ` について
   `objects = acted_on(c.name, μ)`。
3. `Release(v, π, s, k)` の訪問: `objects = others(r)`。
4. `Release(v, π, s, k)` の訪問で `un_bump` が `OutsideBracket` を返したとき:
   `objects = ActRefs(r).objects()`。

**証明**

<1>1. `consume_objects` の作用は言明のとおりである。
  BY p30 の L6
<1>2. `consume_objects` を呼ぶのは、`CancelAnalysis::consume` と、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の `UnBump::OutsideBracket` の枝の 2 か所である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled
<1>3. `consume(pending, var, path)` は `origin(self.vars, self.type_env, var, path).acted_on()` の元を
      `objects` に集め、`self.consume_objects(pending, &objects)` を 1 回呼ぶ。すなわち
      `objects = acted_on(var, path)` である。`consume` を呼ぶのは、`consume_rhs` と、`walk_inner` の
      `RcExpr::Destructure(container, fields, _state, k)` の腕の 2 か所である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, 第 1 節の記法,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕
<1>4. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` が `consumed` に積んだ各 `(var, leaf)` に
      ついて `self.consume(pending, &var, &leaf)` を呼ぶ。`consume_rhs` を呼ぶのは `walk_inner` の
      `RcExpr::Let(x, rhs, k)` の腕 1 か所だけであり、その腕には右辺が `Match` の `Let` は入らない
      (`match` の腕がその先に置かれているため)。よって <1>3 と合わせて 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, <1>3
<1>5. `walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕は、
      `destructure_consumes(container, fields, self.type_env)` の各 `leaf` について
      `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。よって <1>3 と合わせて 2 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕, <1>3
<1>6. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、`let others = self.other_objects(v, path);`
      の直後に `self.consume_objects(&mut pending, &others)` を呼び、その後 `un_bump` の返り値が
      `UnBump::OutsideBracket` のときだけ `let objects = un_bumped.objects();` として
      `self.consume_objects(&mut pending, &objects)` を呼ぶ。ここで `un_bumped = self.acted_references(v,
      path) = ActRefs(r)` である。よって 3 と 4 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     DEF 節点の量
<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L37 (削除される `Release` は `t` より後にある)

**言明** --- `t ∈ CT` と `r ∈ un_bump_releases[t]` について、`t` は `r` で pending であり (DEF 訪問)、
`r` を含むすべての実行路は `t` を含み、その路の上で `t` は `r` より真に前にある。

**証明**

<1>1. L32 の 4 より、`r` の訪問の中の `un_bump(&mut pending, &un_bumped)` の呼び出しが `InBracket(t)` を
      返した。
  BY L32
<1>2. `p30` の `L5` の 3 より、`un_bump` が `InBracket(t)` を返すのは、その第 1 引数の `pending` に、
      `un_bumped` とオブジェクトを共有する要素があり、そのうち最も後ろの要素の `node` が `t` の `NodeId`
      であるときである。すなわちその要素は由来が `t` の要素である。
  BY p30 の L5, <1>1
<1>3. <1>2 の第 1 引数は、`r` の訪問が `un_bump` を呼ぶ時点の `pending` であり、それは `pending(r)` に、
      この腕がそれより前に行う `others(r)` についての `consume_objects` を施したものである。L36 より
      `consume_objects` は要素を取り除くだけで加えないので、由来が `t` の要素は `pending(r)` にも在る。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     L36, <1>2
<1>4. QED
  BY <1>3, L35

### L38 (`t` が pending である区間)

**言明** --- `t ∈ CT` と、`t` を含む実行路 `ρ` を取る。`ρ` の上の節点 `n` について「`t` が `n` で pending
である」が成り立つ `n` の全体は、`ρ` の上の空でない連続する区間であり、その最初の節点は `ρ` の上で `t` の
直後にある節点である。この区間を `I_ρ(t)`、その最後の節点を `n*(ρ)` と書くと、次の 4 つが成り立つ。

1. `n*(ρ)` は `Release` 節点であり、その訪問の `un_bump` は `InBracket(t)` を返し、`subtract` の後に
   由来が `t` の要素の `outstanding` が空になってその要素が取り除かれる。よって
   `n*(ρ) ∈ un_bump_releases[t]` である。
2. `n*(ρ)` は `ρ` の終端の `Ret` より真に前にある。
3. `I_ρ(t)` に入る `Release` 節点のうち、その訪問の `un_bump` が `InBracket(t)` を返すもの全体を
   `R_ρ(t)` と書くと、`n*(ρ) ∈ R_ρ(t) ⊆ un_bump_releases[t] ⊆ Del` であり、
   `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。
4. `I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に在る時点に走る
   `consume_objects(pending, objects)` の呼び出しはどれも、その時点のその要素の `outstanding` が名指す
   名前を `objects` に含まない。よってその要素は `consume_objects` に取り除かれない。

**証明**

<1>1. `t` の訪問は `pending` の末尾に `PendingRetain { node: node_id(t), outstanding: ActRefs(t) }` を
      積んでから継続へ進む。よって `ρ` の上で `t` の直後にある節点 `n0` について `t` は `n0` で pending で
      あり、`out(t, n0) = ActRefs(t)` である。`t` は `Retain` 節点なので継続を持ち、`ρ` の上で `n0` は
      存在する。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     DEF 節点の量, L34, D2
<1>2. `ρ` の上で `t` より後の 2 つの節点 `n`、`n'` が `ρ` の上で隣り合い、`t` が `n` で pending でない
      ならば、`t` は `n'` でも pending でない。
  <2>1. CASE L34 の 1。`pending(n')` は `pending(n)` に `push`・`consume_objects`・`un_bump` を施した
        ものである。要素を加えるのは `push` だけであり、そこで加わる要素の `node` は `n` 自身の
        `NodeId` である。`n` は `Retain` 節点 `t` とは相異なる位置なので、P15 の前半よりその `NodeId` は
        `t` の `NodeId` と異なる。`consume_objects` と `un_bump` は要素を加えない。
    BY L34, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       P15, L36, p30 の L5
  <2>2. CASE L34 の 2。`pending(n')` は `pending(n)` の複製であり、要素の `node` の集合は等しい。
    BY L34
  <2>3. CASE L34 の 3。P18 の第 1 の主張より、`merge` の返り値に残る `Retain` は、いずれのアームの出口
        にも現れるものだけである。`arm_exits[i] = pending(n)` は由来が `t` の要素を持たないので、`t` は
        `merge` の返り値にも入らない。
    BY L34, P18
  <2>4. QED
    BY <2>1, <2>2, <2>3, L34
<1>3. `t` が pending である `ρ` の上の節点の全体は、<1>1 の `n0` から始まる連続する区間である。
  BY <1>1, <1>2
<1>4. `t` は `ρ` の終端の `Ret` では pending でない。
  <2>1. `cancel_body` は `analysis.walk(body, ・, true)` を呼び、`walk_inner` は `Match` のアーム本体に
        だけ `false` を渡し、ほかの継続には自分が受け取った `returns_from_func` をそのまま渡す。よって
        `returns_from_func` が真である節点の全体は、`B` の根から継続だけをたどって得られる鎖であり、
        その鎖に入る唯一の `Ret` 節点は `ret(B の根)` である。
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       DEF 訪問
  <2>2. D3 より、実行路の終端の `Ret` は `ret(B の根)` である。実行路は根から継続をたどり、`Match` では
        アーム本体の実行路を辿ってから継続へ戻るので、最後の節点は根から継続だけをたどって着く `Ret` で
        ある。
    BY D3, DEF 訪問
  <2>3. `walk_inner` の `RcExpr::Ret(_)` の腕は、`returns_from_func` が真のとき `pending` の全要素の
        `node` を `self.needed_retains` に入れる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
  <2>4. QED
    `t` がそこで pending なら <2>3 より `t` が `needed_retains` に入り、`p30` の `L10` より走査の終わりまで
    残る。これは `t ∈ CT` (DEF 削除集合) に反する。
    BY <2>1, <2>2, <2>3, p30 の L10, DEF 削除集合
<1>5. 4 が成り立つ。
  <2>1. `I_ρ(t)` に入る節点 `n` の訪問の中で `consume_objects(pending, objects)` が走る時点に、由来が
        `t` の要素 `e` が走査の `pending` に在り、`objects` のいずれかの名前を `e.outstanding` が名指す
        と仮定する。
    BY 仮定
  <2>2. L36 よりこの呼び出しは `e` を取り除き、`t` を `self.needed_retains` に入れる。
    BY L36, <2>1
  <2>3. QED
    `p30` の `L10` より `t` は走査の終わりまで `needed_retains` に残り、`t ∈ CT` に反する。よって <2>1 の
    仮定は成り立たない。L36 より `consume_objects` が要素を取り除くのはその条件が成り立つときだけなので、
    `e` は `consume_objects` に取り除かれない。
    BY <2>2, p30 の L10, DEF 削除集合, L36
<1>6. 1 が成り立つ。
  <2>1. <1>3 と <1>4 より区間は `ρ` の終端の `Ret` より前で終わるので、区間の最後の節点 `n*` があり、
        `ρ` の上でその直後の節点 `n'` について `t` は `n'` で pending でない。
    BY <1>3, <1>4
  <2>2. CASE L34 の 3 (`n*` がアーム本体の終端の `Ret`)。`arm_exits[i] = pending(n*)` が由来 `t` の要素を
        持ち、`merge` の返り値が持たない。P18 の第 2 の主張より、この `merge` の呼び出しは `t` を
        `self.needed_retains` に入れる。`p30` の `L10` よりそれは走査の終わりまで残るので `t ∈ CT` に
        反する。よってこの場合は起こらない。
    BY L34, P18, p30 の L10, DEF 削除集合
  <2>3. CASE L34 の 2 (`n*` が `Match` 節点)。`pending(n')` は `pending(n*)` の複製なので `t` は `n'` で
        pending であり、`n*` が区間の最後であることに反する。よってこの場合は起こらない。
    BY L34
  <2>4. CASE L34 の 1。`walk_inner` のこれらの腕が `pending` から要素を取り除くのは、`consume_objects` と
        `un_bump` の 2 つだけである。<1>5 より `consume_objects` は由来 `t` の要素を取り除かない。よって
        取り除いたのは `un_bump` である。`p30` の `L5` より `un_bump` が要素を取り除くのは第 3 の場合、
        すなわち `InBracket` を返し、選んだ要素の `outstanding` から `un_bumped` を引いた結果が空に
        なったときだけであり、そのとき返る `NodeId` は取り除かれた要素の `node` すなわち `t` である。
        `un_bump` を呼ぶのは `RcExpr::Release(v, path, _, k)` の腕だけなので、`n*` は `Release` 節点で
        ある。
    BY L34, <1>5, p30 の L5, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  <2>5. QED
    `n*` は `ρ` の上に直後の節点を持つので、L34 の 3 つの場合のいずれかである。<2>2 と <2>3 がそのうち
    2 つを排除する。L32 の 4 より `n*(ρ) ∈ un_bump_releases[t]` である。
    BY <2>1, <2>2, <2>3, <2>4, L34, L32
<1>7. 2 が成り立つ。
  BY <1>3, <1>4, <1>6
<1>8. 3 が成り立つ。
  <2>1. `R_ρ(t)` の各要素は L32 の 4 より `un_bump_releases[t]` の要素であり、`t ∈ CT` なので L32 の 3 より
        `Del` の要素である。<1>6 より `n*(ρ) ∈ R_ρ(t)` である。
    BY L32, <1>6
  <2>2. 区間の上で、由来が `t` の要素の `outstanding` は、区間の最初の節点で `ActRefs(t)` であり
        (<1>1)、L34 の各遷移で次のように変わる。L34 の 1 では、`consume_objects` は残る要素の値を変えず
        (L36)、`un_bump` は `InBracket` で選んだ要素の `outstanding` からだけ `un_bumped` を引く
        (`p30` の `L5`)。よって `un_bump` が `InBracket(t)` を返す遷移では `ActRefs(r)` が引かれ、
        それ以外では変わらない。L34 の 2 では複製なので変わらない。L34 の 3 では、`merge` が
        `uniform.get(&retain.node)` の複製を新しい `outstanding` に据え、P18 よりその値はすべてのアームの
        出口での共通の値、すなわち `arm_exits[i] = pending(n*)` におけるこの要素の `outstanding` に
        等しいので、やはり変わらない。
    BY <1>1, L34, L36, p30 の L5, P18, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
  <2>3. 区間の最後で、`n*(ρ)` の `subtract` の後にこの `outstanding` は空になる。
    BY <1>6
  <2>4. QED
    <2>2 より、`ActRefs(t)` から `R_ρ(t)` の各要素の `ActRefs` を順に引いた結果が <2>3 で空になる。
    多重集合の差なので `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。
    BY <2>1, <2>2, <2>3
<1>9. QED
  BY <1>3, <1>5, <1>6, <1>7, <1>8

### L39 (静的な収支は実行時の収支である)

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`t ∈ CT` が `ρ` の上にあるとき、名前の
多重集合として

`ActRefs^inh_ρ(t) = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)`

である (`ActRefs^inh_ρ` は `p13` の `DEF 実行時の作用`)。したがって、`t` が `ρ` で実際に作る参照の
多重集合は、`R_ρ(t)` の各要素が `ρ` で実際に処分する参照の多重集合の和に等しい。

**証明**

<1>1. `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。`R_ρ(t)` の各要素は `I_ρ(t)` に入るので `ρ` の上に
      あり、`ρ` の上で実行される。
  BY L38
<1>2. `ρ` の上の各 `Retain`/`Release` 節点 `m` と各名前 `o` について、`o` が `ρ` で活性 (`p13` の
      `DEF 名前の活性`) ならば `ActRefs(m)[o] = ActRefs^inh_ρ(m)[o]` であり、活性でなければ
      `ActRefs^inh_ρ(m)[o] = 0` である。
  BY p13 の L10a
<1>3. 活性な名前 `o` について、<1>1 の等式の両辺の `o` の個数は <1>2 より `ActRefs^inh_ρ` の側の個数と
      等しいので、`ActRefs^inh_ρ(t)[o] = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)[o]` である。活性でない名前に
      ついては <1>2 より両辺とも 0 である。よって言明の第 1 の等式が成り立つ。
  BY <1>1, <1>2
<1>4. QED
  `p13` の `DEF 実行時の作用` より、`Retain` 節点 `m` について `ActRefs^inh_ρ(m)` は `m` が `ρ` で実際に
  作る参照の多重集合であり、`Release` 節点 `m` について `m` が `ρ` で実際に処分する参照の多重集合で
  ある。`p13` の `DEF 名前の活性` より、活性な名前 `o` の参照はオブジェクト `obj_ρ(o)` への参照である。
  よって <1>3 の名前ごとの等式は、参照の多重集合の等式である。
  BY <1>3, p13 の DEF 実行時の作用, p13 の DEF 名前の活性, P6

### L40 (`N` は類ごとの bump の和である)

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`ρ` の上の節点 `n` と計数下 (D26) の
オブジェクト `O` について、`N_ρ(n, O) = Σ_{C : obj(C) = O} bumps_ρ(n, C)` である。ここで `N_ρ` は `p13` の
`DEF N`、`bumps_ρ` と `obj(C)` は `p13` の `DEF 類ごとの参照` と `DEF 別名類` のものである。

**証明**

<1>1. QED
  BY p13 の 7.5.4 の <1>2
  `p13-disposals-and-pending.md` の第 7.5.4 節の <1>2 が、この等式をそのまま述べて示している --- 活性な
  各名前 `o` が、`o` を `identity` とする leaf のスロットの属する別名類 `C(o)` を 1 つ決め、
  `obj(C(o)) = obj_ρ(o)` であり、その対応で `N_ρ` の和を並べ替えると `bumps_ρ` の和になる、という形で
  ある。

### L41 (余りの下界)

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`ρ` の上の節点 `n` と計数下のオブジェクト
`O` について、`N_ρ(n, O) ≥ 1` ならば、`n` の位置での参照カウントは `H(n, O) ≥ N_ρ(n, O) + 1` である。

**証明**

<1>1. QED
  BY A19, p13 の 7.5.1 の INV
  `p13-disposals-and-pending.md` の第 7.5.1 節が `INV(n)` としてこの言明を置き、第 7.5.4 節が A19 の
  (i) と (ii) からそれを示している。

### L42 (義務は pending の bump を覆う)

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。活性化の各時点 `τ` と計数下のオブジェクト
`O` について、`Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。とくに `ρ` の上の節点 `n` の入口では
`Obl(n, O) ≥ N_ρ(n, O)` である。

時点と走査の状態の対応は `p13` の `A19` (ii) が置くものを使う。すなわち、実行時の 1 つの事象 (D10 の行が
定める参照の作成・処分) の直後の時点には、走査がその事象に対応する操作を行った直後の `pending` が対応する。

**局所の定義 (類ごとの義務)**。`ρ` を辿る活性化の各時点 `τ` と別名類 `C` について、`obl_ρ(τ, C)` を
`p13` の `DEF 類ごとの参照` の表と同じ規則で定める。ただし第 3 行 (`C` の ρ-終端が借用する (D14)
パラメータ・capture の leaf である場合) の初期値だけを 1 でなく 0 とする。すなわち
`obl_ρ(τ, C) = held_ρ(τ, C) - β(C)` であり、`β(C)` は `C` の ρ-終端が借用するパラメータ・capture の
leaf のとき 1、そうでないとき 0 である。

**証明**

<1>1. 計数下のオブジェクト `O` と各時点 `τ` について `Obl(τ, O) = Σ_{C : obj(C) = O} obl_ρ(τ, C)` である。
  <2>1. D10 が `Obl` を変える事象は、初期値、`Retain`、`Release`、生成、消費の 5 種である。移動は
        `Obl` を変えない。D26 より、数えるのは計数下のオブジェクトへの参照だけである。
    BY D10, D26
  <2>2. これらの事象はいずれも 1 つの inhabited な leaf に紐づき、その leaf は `ρ` の上のスロット (D6)
        であって、ちょうど 1 つの別名類に属する。
    BY D6, D10, p13 の DEF 別名類
  <2>3. `p13` の `DEF 類ごとの参照` の表の 6 行は、この 5 種の事象と次のように対応する。第 1 行が生成、
        第 2 行と第 3 行が初期値 (所有する場合と借用する場合)、第 4 行が `Retain`、第 5 行が `Release`、
        第 6 行が消費である。
    BY p13 の DEF 類ごとの参照, D10
  <2>4. D10 の生成が作る leaf は、その類の ρ-終端である。
    BY p13 の L8, p13 の DEF ρ-歩みと ρ-終端, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: origin_inner, D10
    D10 の生成の 5 行が名指す leaf の束縛は順に、`Binding::Llvm` の宣言が単一の `Arg` でない場合、
    `RcRhs::App` の `Binding::Producer`、`RcRhs::Closure` の `Binding::Producer`、boxed 容器の
    `Binding::Field`、boxed scrutinee の `Binding::Payload(_, Some(tag))` である
    (`collect_bindings` がこの 5 つの束縛を作る)。`p13` の `L8` の (B) とその証明の場合分けより、
    `origin_inner` はこの 5 つの腕で `origin` を呼ばず `Origin::Exactly` を返す。`p13` の
    `DEF ρ-終端` はこれを ρ-終端という。
  <2>5. 計数下の `O` について `obj(C) = O` である類 `C` の ρ-終端は、生成の leaf か、パラメータ・capture の
        leaf である。
    BY p13 の DEF ρ-歩みと ρ-終端, CODE src/rc_ir/ownership.rs: origin_inner, D26, A8, <2>4
    ρ-終端は `origin_inner` が `origin` を呼ばない腕に当たる対であり、その腕は
    `None`/`Param`/`Producer`、単一 `Arg` でない `Binding::Llvm`、boxed 容器の `Field`、boxed
    scrutinee の `Payload` である。`None` は束縛表が持たない名前、すなわちグローバル値であり、A8 と
    D26 よりその指すオブジェクトはグローバル状態で計数下ではない。`Param` はパラメータ・capture の
    leaf であり、残る 4 つは <2>4 の生成の leaf である。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, D10, DEF 類ごとの義務 (本補題)
    <2>3 の対応で、D10 の各事象は `obl_ρ` の同じ増減を起こす。ただし第 3 行の初期値については、D10 は
    借用する unit の下の leaf を `Obl` に入れないので `Obl` の側は 0 であり、`obl_ρ` の定義もそこを 0 と
    している。<2>5 より計数下の `O` を指す類の ρ-終端は第 1 行から第 3 行のいずれかである。よって
    2 つの勘定は各時点で一致する。
<1>2. 各時点 `τ` と各別名類 `C` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  BY A19, 本補題の局所の定義
  A19 の (ii) は各時点について `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` を言う。`β(C) ≤ 1` なので
  `obl_ρ(τ, C) = held_ρ(τ, C) - β(C) ≥ bumps_ρ(τ, C)` である。
<1>3. QED
  BY <1>1, <1>2, L40
  `Obl(τ, O) = Σ_C obl_ρ(τ, C) ≥ Σ_C bumps_ρ(τ, C)` である。節点 `n` の入口ではこの右辺は L40 より
  `N_ρ(n, O)` である。

### L43 (欠損は pending の bump の一部である)

**言明** --- 対応する活性化を固定する。`ρ` の上の位置 `q` と計数下のオブジェクト `O` について

`d(q, O) = Σ_{t} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]`

である。ここで外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り、内側の和は `ρ` で
活性な名前 `o` を渡る。とくに `0 ≤ d(q, O) ≤ N_ρ(q, O)` である。

**証明**

<1>1. `q` より前に実行された `Del` の `Retain` 節点は、`ρ` の上で `q` より前にある `CT` の要素である。
  BY L32, DEF 欠損
<1>2. `q` より前に実行された `Del` の `Release` 節点 `r` は、ある `t ∈ CT` について `r ∈ R_ρ(t)` であり、
      その `t` は `ρ` の上で `r` より前にある。
  BY L32, L37, L38
  L32 の 3 より `r` はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に属する。L37 より `t` は `ρ` の
  上で `r` より真に前にあり、`t` は `r` で pending である。よって `r ∈ I_ρ(t)` であり、L32 の 4 より
  `r` の訪問の `un_bump` は `InBracket(t)` を返すので、L38 の 3 の定義より `r ∈ R_ρ(t)` である。
<1>3. `t ∈ CT` が `ρ` の上で `q` より前にあり、`q` で pending でないとき、`R_ρ(t)` の全要素は `q` より前に
      実行されており、`t` と `R_ρ(t)` の寄与の和は 0 である。
  BY L38, L39, <1>2
  L38 より `I_ρ(t)` は `t` の直後から始まる連続した区間であり、`t` が `q` で pending でないので区間は
  `q` より前で終わる。`R_ρ(t) ⊆ I_ρ(t)` なのでその全要素は `q` より前にある。L39 より
  `ActRefs^inh_ρ(t) = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)` なので、作った参照と処分した参照は打ち消し合う。
<1>4. `t ∈ CT` が `q` で pending であるとき、`t` と、`q` より前に実行された `R_ρ(t)` の要素の寄与の和は、
      名前ごとに `ActRefs^inh_ρ(t) - Σ_{r ∈ R_ρ(t), r は q より前} ActRefs^inh_ρ(r)` であり、これは
      `B_ρ(q, e_t(q))` に等しい。
  BY D27, L38, <1>2
  D27 は、`Retain` の訪問で押し込まれた要素の `B_ρ` を `ActRefs^inh_ρ(t)` と定め、`un_bump` が
  `InBracket` でその要素を選ぶ `Release` の訪問でだけ `ActRefs^inh_ρ` を引き、複製・`merge`・その他の
  節点では値を運ぶだけである。その要素を選ぶ `Release` は L38 の 3 の `R_ρ(t)` の要素である。
<1>5. 名前ごとの寄与をオブジェクトごとの寄与に直す。`Retain`/`Release` が実際に作る (処分する) 参照は、
      その名前 `o` について、オブジェクト `obj_ρ(o)` への参照である。よって `O` への寄与は
      `obj_ρ(o) = O` である名前 `o` の分の和である。
  BY p13 の DEF 名前の活性, p13 の DEF 実行時の作用, P5
<1>6. 言明の等式が成り立つ。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, DEF 欠損
<1>7. QED
  BY <1>6, p13 の L11, p13 の DEF N
  `p13` の `L11` より、活性な `o` について `B_ρ(q, e)[o] = e.outstanding[o] ≥ 0` であり、活性でない `o`
  について `B_ρ(q, e)[o] = 0` である。よって <1>6 の和は 0 以上である。`p13` の `DEF N` の `N_ρ(q, O)` は
  同じ内側の和を `pending(q)` の**すべての**要素について取ったものなので、`d(q, O) ≤ N_ρ(q, O)` で
  ある。

### L44 (2 つの実行の対応)

**言明** --- `cancel` の入力プログラム `P` が D12 を満たすとする。`ρ` を `B` の実行路、`ρ'` をそれに
対応する `B'` の実行路、`α` と `α'` を対応する活性化 (DEF 対応する活性化) とする。このとき、`ρ` の上の
各位置 `q` と各計数下オブジェクト `O` について次の 5 つが成り立つ。`ρ` の終端の `Ret` の消費を行った後の
時点についても同じである。

- **(a)** `α` と `α'` は、`q` までに、`Del` の節点を除いて同じ節点を同じ値に対して実行し、その時点までに
  値を得ている各変数の値は 2 つの実行で等しい。
- **(b)** `H'(q, O) = H(q, O) - d(q, O)` である。
- **(c)** `Obl'(q, O) = Obl(q, O) - d(q, O)` である。
- **(d)** `d(q, O) ≥ 1` ならば `H(q, O) ≥ d(q, O) + 1` かつ `Obl(q, O) ≥ d(q, O)` である。
- **(e)** `H(q, O) = 0` と `H'(q, O) = 0` は同値である。

**節点の実行の扱い**。D10 は義務集合を「実行路上の各位置における」量として定めるので、この証明は 1 つの
節点の実行を 1 つの遷移として扱う。すなわち、その節点について D10 の行が定める参照の作成と処分をまとめて
適用し、その結果カウントが 0 になったオブジェクトについて D7 の走査を行い、走査が処分する参照について
これを繰り返す。A18 (a) より生きているオブジェクトのグラフは非巡回なので、この繰り返しは有限で終わる。

**証明**

<1>1. (d) が成り立つ。
  <2>1. `q` が節点の入口であるとき。`d(q, O) ≥ 1` とする。L43 より `N_ρ(q, O) ≥ d(q, O) ≥ 1` である。
        L41 より `H(q, O) ≥ N_ρ(q, O) + 1 ≥ d(q, O) + 1` であり、L42 より
        `Obl(q, O) ≥ N_ρ(q, O) ≥ d(q, O)` である。
    BY L41, L42, L43
  <2>2. `q` が最後の位置 (終端の `Ret` の消費を行った後) であるとき、`d(q, O) = 0` なので (d) は空虚に
        成り立つ。
    BY L38, L43, L32, DEF 欠損
    L38 より各 `t ∈ CT` が pending である区間は終端の `Ret` より真に前で終わるので、終端の `Ret` の
    入口で pending な `CT` の要素は無く、L43 よりそこで `d = 0` である。L32 の 5 より `Del` は
    `Retain` と `Release` だけなので終端の `Ret` は `Del` に入らず、その消費の後も `d` は変わらない。
  <2>3. QED
    BY <2>1, <2>2
<1>2. (e) は (b) と (d) から出る。
  `d(q, O) = 0` のとき (b) より `H'(q, O) = H(q, O)` である。`d(q, O) ≥ 1` のとき (d) より
  `H(q, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、(b) より `H'(q, O) = H(q, O) - d(q, O) ≥ 1 > 0` である。
  BY <1>1
<1>3. `B'` の所有と借用の割り当て (D14) は `B` のものと同じである。`cancel` は各関数の `clone()` の
      `body` にだけ書き込むので `borrowed_units`・`params`・`capture` を変えず、グローバル初期化子は
      パラメータも capture も持たない。
  BY CODE src/rc_ir/borrow.rs: cancel, D14, D1
<1>4. (a)、(b)、(c) を `ρ` の上の位置についての帰納法で示す。D3 より `ρ` は有限の列なので整礎である。
  <2>1. 基底。`ρ` の最初の位置は `B` の根の入口である。DEF 欠損 より `d = 0` である。DEF 対応する活性化 の
        (C1) より 2 つの活性化は同じ入力から始まるので、そこで値を得ている変数 (パラメータと capture) の
        値は等しく、`H'` は `H` に等しい。D10 の初期値は所有する unit の下の inhabited な leaf で決まり、
        <1>3 より割り当ては同じなので `Obl'` は `Obl` に等しい。
    BY DEF 欠損, DEF 対応する活性化, D10, <1>3
  <2>2. 帰納法の仮定: `ρ` の上の位置 `q` について (a)、(b)、(c) が成り立つ。`q` の節点の実行の後の位置を
        `q'` とする。
    BY 帰納法の仮定
  <2>3. CASE `q` の節点が `Del` の `Retain` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Retain` の行により `O` への参照を `m(O)` 個作り、`H` と `Obl` をそれぞれ `m(O)` 増やす。
        DEF 欠損 より `d` も `m(O)` 増える。D9 の 2 つの表より `Retain` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。よって (a)、(b)、(c) が `q'` で成り立つ。この節点はカウントを上げる
        だけなので、オブジェクトを解放しない。
    BY <2>2, D10, D9, DEF 欠損
  <2>4. CASE `q` の節点が `Del` の `Release` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Release` の行により `O` への参照を `c(O)` 個処分し、`H` と `Obl` をそれぞれ `c(O)` 減らす。
        DEF 欠損 より `d` も `c(O)` 減る。D9 の 2 つの表より `Release` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。
    <3>1. この節点はオブジェクトを解放しない。
      <4>1. `O` が解放されたとすると `H(q', O) = 0` である。
        BY D7
      <4>2. `d(q, O) = c(O)` かつ `H(q, O) = c(O)` である。
        BY <4>1, DEF 欠損
        `H(q', O) = H(q, O) - c(O) = 0` なので `H(q, O) = c(O)`。また L43 より `d(q', O) ≥ 0` であり、
        `d(q', O) = d(q, O) - c(O)` なので `d(q, O) ≥ c(O)`。一方 (b) と <2>2 より
        `H'(q, O) = H(q, O) - d(q, O) = c(O) - d(q, O) ≥ 0` なので `d(q, O) ≤ c(O)`。
      <4>3. QED
        BY <4>2, <1>1
        `c(O) ≥ 1` (解放が起きたので処分がある) なので `d(q, O) = c(O) ≥ 1` であり、<1>1 の (d) より
        `H(q, O) ≥ d(q, O) + 1 = c(O) + 1` である。これは <4>2 の `H(q, O) = c(O)` に反する。
    <3>2. QED
      <3>1 より走査は起きないので、`H` の変化は D10 の `Release` の行の分だけである。よって (a)、(b)、
      (c) が `q'` で成り立つ。
      BY <2>2, D10, D9, DEF 欠損, <3>1
  <2>5. CASE `q` の節点が `Del` に入らない。`ρ'` の対応する位置にも同じ節点がある (L30、L31)。
    <3>1. 2 つの実行はこの節点について D10 の同じ行を同じ値に対して適用する。`d` は変わらない。
      BY <2>2, D10, A4, L30, DEF 欠損
      <2>2 の (a) より変数の値が等しく、L30 より節点の式の変位・変数・path・`RcState` も等しいので、
      D10 の行が名指す leaf と、A4 よりコード生成が行う参照カウントの操作は 2 つの実行で同じである。
      `q` は `Del` に入らないので DEF 欠損 の 2 つの数え上げは変わらない。
    <3>2. D10 の行を適用した後、カウントが 0 になるオブジェクトの集合は 2 つの実行で等しく、その各要素
          `O` について `d(q, O) = 0` である。さらに走査が処分する参照についても同じことが繰り返し成り立つ。
      <4>1. 走査の段数についての帰納法で示す。各段で、2 つの実行はそこまでに同じ参照を処分しており、
            各計数下 `O` についてその段の直前のカウントの差は `d(q, O)` である。
        BY <3>1, <2>2
      <4>2. ある段で `α` の `O` のカウントが 0 になったとする。走査の間カウントは下がるだけであり、
            この節点の実行が終わった後の値が `H(q', O)` なので `H(q', O) = 0` である。<1>1 の (d) より
            `d(q', O) = 0` であり、`q` は `Del` に入らないので `d(q, O) = 0` である。<4>1 より `α'` の
            カウントも同じ段で 0 になる。
        BY <4>1, <1>1, DEF 欠損
      <4>3. ある段で `α'` の `O` のカウントが 0 になり、`α` のそれが 0 でないとする。<4>1 よりその段の
            `α` のカウントは `d(q, O)` であり、`d(q, O) ≥ 1` である。走査の間カウントは下がるだけなので
            `H(q', O) ≤ d(q, O)` である。一方 <1>1 の (d) を `q'` に当てると、`d(q', O) = d(q, O) ≥ 1`
            なので `H(q', O) ≥ d(q, O) + 1` である。矛盾する。よってこの場合は起こらない。
        BY <4>1, <1>1, DEF 欠損
      <4>4. QED
        BY <4>1, <4>2, <4>3
        <4>2 と <4>3 より、各段で 0 になるオブジェクトの集合は 2 つの実行で等しい。よって次の段で処分
        される参照も等しく、<4>1 の帰納法が進む。A18 (a) より段数は有限である。
    <3>3. QED
      <3>1 と <3>2 より、この節点の実行は 2 つの実行で `H` を同じだけ変え、`Obl` も D10 の行の分だけ
      同じく変え、束縛する変数に同じ値を置く。`d` は変わらないので (a)、(b)、(c) が `q'` で成り立つ。
      BY <3>1, <3>2, <2>2, D10
  <2>6. QED
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかである (L32 の 5)。
    BY <2>1, <2>3, <2>4, <2>5, L32
<1>5. QED
  BY <1>1, <1>2, <1>4

## 4. P19 (削除される retain の性質)

**言明 (README)** --- `cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含むすべての実行路に
おいて、`t` より後にある、`t` の `outstanding` のオブジェクトを `acted_on` に含む消費より前、かつ終端の
`Ret` より前に、削除される `Release` 群が `t` の `outstanding` を空にする。さらに、`t` とともに削除される
各 `Release` は、実行路の上で `t` より後ろにある。

**証明する形**。「`t` の `outstanding`」は、その位置での残量 `out(t, ・)` (DEF 訪問) として読む。`t` の
`outstanding` は `Release` 群が順に減らしていく量なので、位置ごとに値が違う。第 10 節の 差し戻し 1 が、
初期値 `ActRefs(t)` として読むと言明が偽になることを述べる。示すのは次の 4 つである。

`t ∈ Del` が `Retain` 節点であるとき、`t` を含むすべての実行路 `ρ` について、

1. `ρ` の上に `t` より後の `Release` 節点の有限集合 `R_ρ(t) ⊆ Del` があり、
   `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。その最後の要素 `n*(ρ)` の訪問で `t` の `outstanding`
   が空になる。
2. `n*(ρ)` は `ρ` の終端の `Ret` より真に前にある。
3. `ρ` の上で `t` より後にある消費点 `c` であって、`t` が `c` で pending であり、`c` で消費されるある
   leaf `(w, μ)` の `acted_on(w, μ)` が `out(t, c)` の名指すオブジェクトを含むものは、存在しない。よって、
   `t` より後にあってそのようなオブジェクトを名指す消費点は、`t` が pending でなくなった後、すなわち
   `n*(ρ)` より後にある。
4. `r ∈ un_bump_releases[t]` である各 `Release` 節点について、`r` を含むすべての実行路の上で `t` は `r`
   より真に前にある。

**証明**

<1>1. `t ∈ Del` が `Retain` 節点であることと `t ∈ CT` であることは同値である。
  BY L32
<1>2. 1 が成り立つ。
  BY L38, L32, <1>1
  L38 の 3 が `R_ρ(t) ⊆ un_bump_releases[t] ⊆ Del` と静的な収支を与え、L38 の 1 が `n*(ρ)` の訪問で
  `outstanding` が空になることを与える。L37 より `R_ρ(t)` の各要素は `ρ` の上で `t` より後にある。
<1>3. 2 が成り立つ。
  BY L38, <1>1
<1>4. 3 が成り立つ。
  <2>1. そのような消費点 `c` があると仮定する。`t` が `c` で pending なので、L38 より `c ∈ I_ρ(t)` で
        ある。
    BY 仮定, L38
  <2>2. CASE `c` が D9 の消費の表の `App`、`Closure`、`Llvm` の行の位置である。この位置は右辺が `Match`
        でない `Let` 節点である。P7 の前半より D9 が消費とする leaf はすべて `collect_consumes` が報告し、
        `collect_consumes_go` はこの 3 つの右辺について `rhs_consumes` にそのまま委ね、その位置でほかに
        `out` へ積まない。よって `rhs_consumes` はこの位置で `(w, μ)` を報告し、L36 の 1 より `c` の訪問は
        `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
    BY P7, CODE src/rc_ir/ownership.rs: collect_consumes_go, L36, D9
  <2>3. CASE `c` が D9 の消費の表の `Destructure` の 2 行の位置である。P7 の前半と
        `collect_consumes_go` の `Destructure` の腕より、`destructure_consumes` はこの位置で `μ` を返し、
        L36 の 2 より `c` の訪問は `consume_objects(pending, acted_on(c の容器の名前, μ))` を呼ぶ。
    BY P7, CODE src/rc_ir/ownership.rs: collect_consumes_go, L36, D9
  <2>4. CASE `c` が D9 の消費の表の「関数本体の終端の `Ret(x)`」の行の位置である。L38 より `t` は `ρ` の
        終端の `Ret` では pending でないので、仮定に反する。よってこの場合は起こらない。
    BY L38
  <2>5. QED
    <2>2 と <2>3 の呼び出しは、`I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に
    在る時点に走る。仮定よりその `objects` は `out(t, c)` が名指すオブジェクトを含むので、L38 の 4 に
    反する。D9 の消費の表の 6 行を <2>2、<2>3、<2>4 が尽くす。よって <2>1 の仮定は成り立たない。
    L38 より `t` が pending である区間は `n*(ρ)` で終わるので、`t` より後にあってそのようなオブジェクトを
    名指す消費点は `n*(ρ)` より後にある。
    BY <2>1, <2>2, <2>3, <2>4, L38, D9
<1>5. 4 が成り立つ。
  BY L37
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 5. P20 (削除は収支を保つ)

**言明 (README)** --- 各実行路において、削除される `Retain` が実行時に作る参照の多重集合は、その路で
実行される削除される `Release` が実行時に処分する参照の多重集合に一致する。

**証明** 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。

<1>1. `ρ` の上にある `Del` の `Release` 節点の全体は、`ρ` の上にある `Del` の `Retain` 節点 `t` に
      ついての `R_ρ(t)` の非交和である。
  <2>1. L32 の 3 より、`Del` の各 `Release` 節点はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に
        属する。
    BY L32
  <2>2. `r ∈ un_bump_releases[t]` が `ρ` の上にあるならば、L37 より `t` も `ρ` の上にあって `r` より前に
        あり、`t` は `r` で pending である。よって `r ∈ I_ρ(t)` であり、L32 の 4 より `r` の訪問の
        `un_bump` は `InBracket(t)` を返すので、L38 の 3 より `r ∈ R_ρ(t)` である。
    BY L37, L38, L32
  <2>3. 逆に `t` が `ρ` の上にある `CT` の要素ならば、L38 の 3 より `R_ρ(t)` の各要素は `Del` の
        `Release` 節点であり、`I_ρ(t)` に入るので `ρ` の上にある。
    BY L38
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>2. `ρ` の上にある各 `t ∈ CT` について、`t` が `ρ` で実際に作る参照の多重集合は、`R_ρ(t)` の各要素が
      `ρ` で実際に処分する参照の多重集合の和に等しい。
  BY L39
<1>3. QED
  BY <1>1, <1>2, L32
  L32 の 5 と 1 より、`ρ` の上にある `Del` の `Retain` 節点は `ρ` の上にある `CT` の要素の全体である。
  <1>2 をそのすべてについて足し、<1>1 で右辺をまとめると言明の等式になる。

## 6. P21 (削除は解放後の読みを作らない)

**言明 (README)** --- 削除の前後で、各読む構文の各位置において解放されているオブジェクトの集合は変わらない。

**証明する形**。`cancel` の入力プログラム `P` が D12 を満たすとする (P23 の仮定)。`ρ` を `B` の実行路、
`ρ'` を対応する `B'` の実行路、`α` と `α'` を対応する活性化 (DEF 対応する活性化) とする。示すのは、
`ρ` の上の**すべての**位置について、解放されているオブジェクトの集合が `α` と `α'` で等しいことである。
読む構文の位置はその特別な場合であり、`Retain`/`Release` 節点の位置も含まれる -- D11 の (S-c) は
その 2 つが触れる先も条件に入れるので、後者が要る。

**証明**

<1>1. `ρ` の上の各位置 `q` と各計数下オブジェクト `O` について、`H(q, O) = 0` と `H'(q, O) = 0` は
      同値である。
  BY L44
<1>2. グローバル状態のオブジェクト (D26) は 2 つの実行のどちらでも解放されない。
  BY D26, A8
<1>3. QED
  D7 より、ある位置で解放されているオブジェクトはその位置で参照カウントが 0 のものである。<1>1 と <1>2 より
  その集合は 2 つの実行で等しい。L30 と L31 より `ρ'` の各位置の節点は `ρ` の対応する位置の節点と同じ式の
  変位・変数・path を持ち、L44 の (a) よりその位置の変数の値も等しいので、読む構文とその読む値は 2 つの
  実行で同じである。
  BY <1>1, <1>2, D7, L30, L31, L44

## 7. P22 (`drop_nodes` の正しさ)

**言明 (README)** --- `drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release` 節点だけを
取り除いた木を返し、他の節点の種類・変数・path・並びを変えない。

**証明**

<1>1. QED
  BY L30
  L30 がこの言明である。`cancel_body` が渡す `S = Del` の要素がすべて `Retain` 節点か `Release` 節点で
  あることは L32 の 5 が述べるので、L31 の仮定も満たされ、実行路の対応 (DEF 路の対応) が定まる。

## 8. P23 (`cancel` は RC 規律を保存する)

**言明 (README)** --- D12 の意味で RC 規律を満たすプログラムを入力とすると、`cancel` の出力も D12 の意味で
RC 規律を満たす。

**証明** 入力プログラムを `P`、出力を `P'` と書く。`P` は D12 を満たし、A19 を満たすとする。

<1>1. `P'` の所有と借用の割り当て (D14) は `P` のものと同じである。
  <2>1. `cancel` は `prog.funcs.values()` の各 `f` について `let mut clone = f.clone();` を作り、
        `clone.body` にだけ書き込んで `(f.name.clone(), clone)` を `funcs` に入れる。よって
        `borrowed_units`、`params`、`capture` は変わらない。グローバル初期化子はパラメータも capture も
        持たない (D1)。
    BY CODE src/rc_ir/borrow.rs: cancel, D1
  <2>2. QED
    D14 の割り当ては `RcFunc::borrowed_units` が定めるので、<2>1 より変わらない。
    BY D14, <2>1
<1>2. `P'` の各関数の本体と各グローバル初期化子の `init` は、`P` の対応するものに `cancel_body` を適用した
      ものであり、DEF 出力の本体 の `B'` である。
  BY CODE src/rc_ir/borrow.rs: cancel, DEF 出力の本体
<1>3. `B'` の各実行路 `ρ'` と、それを辿る各活性化 `α'` について、対応する `B` の実行路 `ρ` と活性化 `α`
      を取る (DEF 路の対応、DEF 対応する活性化)。`ρ` について D11 の (S-a)、(S-b)、(S-c) が成り立つ。
  BY D12, L31, L32, DEF 路の対応, DEF 対応する活性化
<1>4. `ρ'` の各位置において `Obl'(q, O) = Obl(q, O) - d(q, O)` であり、`d(q, O) ≥ 0` である。
  BY L44, L43
<1>5. (S-a) が `ρ'` で成り立つ。
  <2>1. `ρ'` の上で `Obl'` から参照を取り除く操作は、`ρ` の上の同じ操作から `Del` の `Release` を除いた
        ものである。`ρ` と `ρ'` は `Del` の節点を除いて同じ節点を同じ値に対して実行するので (L44 の (a))、
        残る各操作が取り除く参照の個数は 2 つの実行で等しい。
    BY L44, L30, L31, D10
  <2>2. そのような操作の 1 つが位置 `q` の節点で `O` への参照を `c` 個取り除くとする。その直後の時点を
        `τ` とすると、L42 より `Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。
    BY L42
  <2>3. `Σ_{C : obj(C) = O} bumps_ρ(τ, C) ≥ d(q, O)` である。
    BY L43, L40, L38, D27, L32
    L43 より `d(q, O)` は、`q` で pending である `CT` の要素の `B_ρ` を `obj_ρ` が `O` を指す名前に
    ついて足したものである。それらの要素は `τ` の時点でも走査の `pending` に在る -- `q` は `Del` に
    入らないので (`Del` の `Release` は <2>1 で除いてある)、L38 の 4 より `consume_objects` はそれらを
    取り除かず、`un_bump` がそれらを取り除けばその `Release` は `un_bump_releases` に入って `Del` の
    要素になる (L32 の 4 と 3) からである。D27 より `consume_objects` はそれらの `B_ρ` を変えない。
    L40 の対応で名前ごとの和を類ごとの和に並べ替えると、`bumps_ρ(τ, ・)` の和はこれ以上である。
  <2>4. QED
    BY <2>2, <2>3, <1>4
    `Obl(q, O) - c = Obl(τ, O) ≥ d(q, O)` なので `Obl'(q, O) = Obl(q, O) - d(q, O) ≥ c` である。すなわち
    取り除かれる参照は `Obl'` に入っている。
<1>6. (S-b) が `ρ'` で成り立つ。
  <2>1. `ρ` の終端の `Ret` では、`CT` の要素はどれも pending でない。
    BY L38
    L38 より各 `t ∈ CT` が pending である区間は `n*(ρ)` で終わり、`n*(ρ)` は終端の `Ret` より真に前に
    ある。
  <2>2. 終端の `Ret` の位置とその消費を行った後の時点で `d = 0` である。
    BY <2>1, L43, DEF 欠損
    L43 より `d` は `CT` の pending な要素の `B_ρ` の和であり、<2>1 よりその和は空である。終端の `Ret` は
    `Del` に入らない (L32 の 5 より `Del` は `Retain` と `Release` だけ) ので、その消費の後も `d` は
    変わらない。
  <2>3. QED
    BY <1>3, <2>2, L44
    <1>3 の (S-b) より `ρ` の終端の `Ret` の消費の後の `Obl` は空である。L44 の (c) と <2>2 より
    その時点の `Obl'` は `Obl` に等しいので、`Obl'` も空である。L30 と L31 より `ρ'` の終端の `Ret` は
    `ρ` の終端の `Ret` と同じ節点であり、L44 の (a) より消費する値も同じである。
<1>7. (S-c) が `ρ'` で成り立つ。
  <2>1. `ρ'` の読む構文 (D7) と `Retain`/`Release` 節点は、`ρ` のそれらから `Del` の節点を除いたもので
        あり、各位置で名指す変数と path は同じである。L44 の (a) よりその位置の変数の値も同じなので、
        読みうるオブジェクトと触れるオブジェクトは 2 つの実行で同じである。
    BY L30, L31, L32, L44, D7
  <2>2. P21 より、`ρ` の各位置で解放されているオブジェクトの集合は 2 つの実行で等しい。
    BY P21
  <2>3. QED
    <1>3 の (S-c) より、`ρ` の各読む構文の各位置で読みうるオブジェクトと、各 `Retain`/`Release` が触れる
    オブジェクトは、その時点で解放されていない。<2>1 と <2>2 より `ρ'` についても同じである。
    BY <1>3, <2>1, <2>2
<1>8. QED
  <1>2 より `P'` のすべての本体は `B'` の形であり、<1>5、<1>6、<1>7 よりそのすべての実行路で D11 の
  3 つが成り立つ。<1>1 より読む割り当ても同じである。D12 はこれを言う。
  BY <1>1, <1>2, <1>5, <1>6, <1>7, D11, D12

## 9. P24 (D12 が見ない部分の保存)

**言明 (README)** --- `borrow_ify` と `cancel` は、D12 が見ない部分について次を満たす。`roots` を変えない。
各関数の `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` を変えない。各グローバル初期化子の
`symbol` と `ty` を変えず、`owns_initializer` と `owns_storage` に `true` を書く。D1 が述べる呼び出し順に
より、この書き込みは正しい値を書く。

**証明**

<1>1. `roots` は変わらない。`borrow_ify` の返す `RcProgram` の `roots` は `prog.roots.clone()` であり、
      `cancel` の返す `RcProgram` の `roots` も `prog.roots.clone()` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel
<1>2. `cancel` の出力の各関数は、入力の関数の `clone()` に `body` だけを書き込んだものである。よって
      `fn_ty`、`ret_ty`、`params`、`inline_into_callers` は変わらない。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>3. `borrow_ify` の出力の各関数について、`fn_ty`、`ret_ty`、`params` の型、`inline_into_callers` は
      入力の対応する関数のものと等しい。
  <2>1. 元の版 `f_own` は `func.clone()` に `body` を書き込んだものであり、その後
        `for func in funcs.values_mut()` のループが `borrowed_units` だけを書き換える。よってこの 4 つは
        変わらない。
    BY CODE src/rc_ir/borrow.rs: borrow_ify
  <2>2. 借用版は `clone_func(func, ・, ・)` が作る `RcFunc` に `body` を書き込んだものであり、
        `clone_func` は `fn_ty: func.fn_ty.clone()`、`ret_ty: func.ret_ty.clone()`、
        `inline_into_callers: func.inline_into_callers` を据える。`params` は `fresh_rename_function` が
        返すもので、その各要素は `rename_var(p, &renaming)` すなわち `p` の複製の `name` だけを差し替えた
        ものなので、型は変わらない。
    BY CODE src/rc_ir/borrow.rs: clone_func, CODE src/rc_ir/rename.rs: fresh_rename_function,
       CODE src/rc_ir/rename.rs: rename_var
  <2>3. QED
    BY <2>1, <2>2
<1>4. グローバル初期化子について、`borrow_ify` と `cancel` はどちらも `symbol: g.symbol.clone()`、
      `ty: g.ty.clone()`、`owns_initializer: true`、`owns_storage: true` を持つ `RcGlobalInit` を作る。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel
<1>5. `true` が正しい値である。`build_object_files` は `optimize_rc_program` を呼び、その返り値を
      `divide_into_units` と `divide_among_units` に渡す。すなわち `borrow_ify` と `cancel` が走るのは
      分割の前であり、そのときプログラムは 1 つで、その 1 つがすべての初期化子と記憶域を持つ。
  BY D1, CODE src/build/build_object_files.rs: build_object_files,
     CODE src/build/build_object_files.rs: optimize_rc_program
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 10. README へ差し戻す点

### 差し戻し 1 (P19 の「`t` の `outstanding`」は位置ごとの残量である)

P19 の言明の「`t` の `outstanding`」を、`t` が bump した参照の全体 `ActRefs(t)` として読むと、言明は偽に
なる。反例の形はこうである。`t = Retain(v, π)` で `π` が unbox union の unit であり、`ActRefs(t)` が
2 つの名前 `o1`、`o2` を持つとする。`ActRefs` が `{o1}` である `Release` `r1` が `t` を部分的に un-bump
すると (`p30` の `L5` の 3 は `covers` が成り立てば引くので、これは起こる)、`out(t, ・)` は `{o2}` に
なる。その後に `o1` を `acted_on` に含む消費点が来ても、`consume_objects` の述語
`retain.outstanding.names(object)` は偽なので `t` は `pending` に残り、さらに後の `Release` が `o2` を
un-bump して `t` は削除されうる。この消費点は `ActRefs(t)` の名前を含むが、`R_ρ(t)` の最後の要素より前に
ある。

**これは欠陥ではない。** その消費点の時点で `o1` の bump はすでに `r1` が un-bump しており、`r1` も
`t` と一緒に削除されるので、欠損 `d` は `o1` について 0 である。P23 が使うのも残量の形である。README の
P19 の本文に「その位置での `outstanding`」と書き足すのがよい。

### 差し戻し 2 (P18a はオブジェクトごとの形でなければ P21 を支えない)

README の P18a は、名前 `o` ごとに `n(o) = Σ_p B(p, ρ)[o]` を取り `H(o) ≥ n(o) + 1` を言う。P21 が要る
のは、1 つのオブジェクト `O` についての `H(O) ≥ (Σ_{o : obj_ρ(o) = O} n(o)) + 1` である。1 つの
オブジェクトを指す活性な名前が 2 つ以上あるとき、後者は前者から出ない。名前が分かれる形は README の
第 8 節が #552 として記録しているものである。

この強い形は `p13-disposals-and-pending.md` の第 7.5.1 節に `INV(n)` として書かれており、同ファイルの
第 7.5.4 節が A19 の (i) と (ii) から示している。この文書は L41 としてそれを引用した。README の P18a を
`INV` の形に書き換えるか、`INV` を別の命題として置くのがよい。

### 差し戻し 3 (義務集合の側の言明が README に無い)

D11 の (S-a) は `Obl` についての条件であり、A19 の (i) が言うのは `H` についての不等式である。P23 の
(S-a) が要るのは

> 各時点 `τ` と各計数下オブジェクト `O` について `Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。

であり、これは A19 の (ii) と `p13` の `DEF 類ごとの参照` から出る -- 借用する終端を持つ類だけが
`held_ρ` の初期値 1 を `Obl` に持たないので、`obl_ρ = held_ρ - β`、`β ≤ 1` となり、A19 (ii) の `+1` が
ちょうどその `β` を吸収する。この文書は L42 としてそれを示した。層 3 の命題 (たとえば `P18c`) として
README に置くのがよい。

### 差し戻し 4 (2 つの実行が同じアームを選ぶことは前提である)

P21 と P23 は、`B` の実行と `B'` の実行を対応する位置で並べて比べる (README の P21 の「各読む構文の各
位置において」がすでにこの対応を前提にしている)。この対応が切れるのは、一意性の観測点 (D18) の観測値が
2 つの実行で食い違うときである。観測値は `Bool` であり、`Match` の scrutinee になるので、食い違えば
2 つの実行は別のアームへ入る。この文書は `DEF 対応する活性化` の (C2) としてこの一致を前提に置いた。
**この前提を果たす者は居ない。**

食い違いが起こる範囲は次のところまで絞れる。`unsafe_is_unique` の `result_prov` は `Unknown` を宣言し、
`borrows_operand` は既定の偽なので、`rhs_consumes` はそのオペランドの boxed leaf をすべて消費として報告する
(`CODE src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov`,
`CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand`)。よって観測点の訪問は
`consume_objects(pending, acted_on(オペランド, λ))` を呼び、その名前を `outstanding` に持つ pending な
`Retain` は `needed_retains` に入って削除の対象から外れる。残るのは、同じオブジェクトを**別の名前**で
名指す `Retain` が pending なままになる形 -- README の第 8 節が #552 として記録している形 -- であり、
そこでのみ観測されるオブジェクトについて `d(q, O) ≥ 1` になりうる。そのとき入力では
`H(q, O) ≥ d(q, O) + 1 ≥ 2` なので観測値は偽、出力では `H'(q, O) = H(q, O) - d(q, O)` が 1 になりうるので
真である。

README にこの前提を書くか、P26 の軸 (観測の保存) でそれを閉じる必要がある。

### 差し戻し 5 (P21 の言明に入力の D12 が要る)

P21 の証明 (L44) は、入力の実行で解放されたオブジェクトのカウントが上がらないことを使い、その根拠は
D11 の (S-c) である。README の P21 の言明には前提が書かれていない。「D12 を満たすプログラムを入力と
すると」を足すのがよい。P23 は同じ前提を持つので、鎖は閉じたままである。

### 差し戻し 6 (`p13` の `L11` の引用が対象コミットのコードと合わない)

`p13-disposals-and-pending.md` の第 7.3 節の `L11` は、`consume_objects` の述語を
`retain.outstanding.names(object) || retain.others.contains(object)` と引用し (<1>1a)、`PendingRetain` に
`others` フィールドがあるものとして書かれている (<2>1 の <3>1、<2>5 の <3>4)。対象コミットの
`PendingRetain` は `node` と `outstanding` の 2 フィールドで、`consume_objects` の述語は
`retain.outstanding.names(object)` だけである
(`CODE src/rc_ir/borrow.rs: PendingRetain`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)。

`L11` の言明は変わらない。選言肢が 1 つ減ると `consume_objects` が取り除く要素が減るだけであり、`L11` は
残る要素についての主張だからである。直すのは引用である。

### 差し戻し 7 (節点の実行を 1 つの遷移として扱っている)

L44 は、1 つの節点の実行を、D10 の行をまとめて適用してから解放の走査 (D7) を行う 1 つの遷移として扱う。
D10 が義務集合を「実行路上の各位置における」量として定めるので、これは D10 の粒度に合わせた扱いである。
節点の実行の途中 -- とくに `App` が呼び出す活性化の内部 -- で参照カウントがどう動くかは、1 つの本体に
ついての述語である D11 の外にあり、P27 (実行の合成) が扱う軸である。README の D10 か D21 に、この粒度を
1 文で書いておくと、層 4 と層 5 の境界がはっきりする。

### 気づいたコードの欠陥

新しいコードの欠陥は見つからなかった。差し戻し 4 が述べる観測値の食い違いは、README の第 8 節が #552 と
して既に記録している形の帰結であり、そこに書かれた「観測可能な障害は再現できていない」と同じ位置にある。
