# P19 - P24: `cancel` が RC 規律を保存すること

この文書は README の層 4 の 6 命題 P19, P20, P21, P22, P23, P24 を証明し、README が層 3 に置く P18c を
`L42` として扱う。README の定義 D1 - D21、D26、D27、仮定 A1 - A20、および命題 P1 - P18b の**言明**の
上に立つ。主定理 T は `p70-main-theorem.md` の担当であり、この文書は扱わない。

対象コミットは `38412a37` である。README が名指す `39c41033` との差は `funcs_observing_uniqueness` の
書き換えだけであり、この文書が引用する記号はどれも動いていない。

引用する外部の補題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF bump の帰属` (`B_ρ`)、`DEF ρ-歩みと ρ-終端`、`DEF 別名類` (`obj(C)`)、
`DEF 類ごとの参照` (`held_ρ`、`bumps_ρ`) -- と、補題 `L7` (boxed leaf の路は反鎖をなす)、`L8`
(`origin` の 1 段は inhabited を保つ)、`L9` (`identity` は inhabited を決める)、`L10a`、`L11`。これらは
`p30 の L10`、`p13 の L11` のようにファイル名を添えて引用する。

この文書が導入する補題は `L30` から番号を付ける。`p30` と `p13` の補題の番号と衝突させないためである。
この文書が置く補助前提は `H1` - `H3` と呼び、第 2 節の末尾に並べる。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P19 | 証明済み |
| P20 | 証明済み |
| P21 | **H3 の下でだけ証明した。** 呼び出し先の活性化の対応を前提に置いている (第 10 節の 差し戻し 3) |
| P22 | 証明済み |
| P23 | **証明されていない。** (S-b) と (S-c) は P21 と同じ H3 の下で閉じるが、(S-a) は `L42` に載り、`L42` は H1 と H2 を要する |
| P24 | 証明済み |
| P18c | **証明されていない。** `L42` が H1 と H2 の下でだけ閉じる (第 10 節の 差し戻し 1 と 2) |

**開いている 3 点はどれもこの文書の外に在る。** H1 は `p13` の補題に、H2 は `borrow_ify` の出力の性質に、
H3 は活性化の木を跨ぐ相互帰納に属する。第 10 節が、それぞれを誰がどこで果たせるかを述べる。

**A19 (ii-b) の読みについて。** README は `held ≥ bumps` を置き、そのすぐ後に「bump より 1 つ多い」と
書くと字義どおりには偽であることを述べ、P18a が使う導かれた形を「bump が 1 以上ある時点では、参照は
bump より 1 つ多い」と書く。この文書はこの 2 つの形だけを使い、各時点についての `held ≥ 1 + bumps` は
使わない。`L42` の <1>2 が、その 2 つの形で足りる場合と足りない場合を分ける。

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
オブジェクトであり、この文書ではこれを**オブジェクト**と呼び、`O` で表す。名前 `o` が実行路 `ρ` で
**活性**であること、および活性な名前が指すオブジェクト `obj_ρ(o)` は、`p13` の `DEF 名前の活性` が
定める。**README の P18a が `obj(o)` と書くのはこの `obj_ρ(o)` である** -- 名前 `o = (u, σ)` は活性で
あるとき `ρ` の上のスロット (D6) であり、`obj_ρ(o) = obj(u, σ)` だからである (`p13 の L9`、P5 (a))。

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
`ret` が `Ret` 節点に与える値はその節点自身である。

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

`ρ` を `B` の実行路、`ρ'` をそれに対応する `B'` の実行路とする。`B` の 1 回の活性化 `α` (D21) と `B'` の
1 回の活性化 `α'` が**対応する**とは、次の 2 つが成り立つことをいう。

- **(C1)** `α` と `α'` は同じ入力から始まる。すなわち、パラメータと capture に同じ値を受け取り、開始時の
  ヒープが同じである。
- **(C2)** `α` が辿る実行路 (D21) は `ρ` であり、`α'` が辿る実行路は `ρ'` である。

**(C2) は前提であって結論ではない。** README の P21 がそう述べる -- 一意性の観測 (D18) は参照カウントを
読むので、削除がカウントを下げると観測値が変わり、その先で選ばれるアームが変わりうる。第 10 節の
差し戻し 4 が、この前提が P23 の言明にも要ることを述べる。

### DEF 実行時の量

対応する活性化 `α`、`α'` を固定する。`ρ` の上の節点 `q` について、

- `H(q, O)` を、`α` が `q` の実行に入る**直前**のオブジェクト `O` の参照カウント (D7)、`Obl(q, O)` を、
  その時点の義務集合 (D10) が持つ `O` への参照の個数とする。
- `H'(q, O)`、`Obl'(q, O)` を、`α'` について `ρ'` の対応する位置で同じように定める。`q ∈ Del` のときは、
  `ρ'` にその節点は無いので、`ρ'` の上で `q` の直後にあたる位置の値を取る。

`ρ` の上の**位置**とは、この意味で節点の実行の直前の時点をいう。これに、`ρ` の終端の `Ret` の消費を
行った後の時点を 1 つ加え、これも位置と呼ぶ。D3 より `ρ` は有限の列なので、位置は有限個である。`d` と
`H` と `Obl` はこの最後の位置でも定まる。

活性化の**時点**とは、D10 の行が定める参照の作成・処分の事象の間の区切りをいう。位置は時点の一部で
ある -- 1 つの節点の実行が複数の事象を起こすことがあるので、時点の方が細かい。

### DEF 解放されている

オブジェクト `O` が活性化のある位置 `q` で**解放されている**とは、`q` かそれより前の位置で `O` の参照
カウントが 0 であること、すなわち `H(q0, O) = 0` を満たす `q` 以前の位置 `q0` が在ることをいう。D7 は
参照カウントが 0 になったオブジェクトが解放されると定めるので、これは D7 の「解放される」を位置の言葉で
書いたものである。`α'` についても `H'` で同じように定める。

### DEF 欠損

対応する活性化を固定する。`ρ` の上の位置 `q` と計数下 (D26) のオブジェクト `O` について、

`d(q, O) :=` (`q` より前に実行された `Del` の `Retain` 節点が `O` への参照を作った個数)
`-` (`q` より前に実行された `Del` の `Release` 節点が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。

### DEF N

対応する活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト `O` について、

`N(q, O) := Σ_{p ∈ pending(q)} Σ_{o} B_ρ(q, p)[o]`

と定める。内側の和は、`ρ` で活性であって `obj_ρ(o) = O` である名前 `o` を渡る。`B_ρ(q, p)` は D27 の
`B(p, ρ)` を節点 `q` の訪問の入口で読んだものであり、`p13` の `DEF bump の帰属` が同じ規則を表で書く。

**`N(q, O)` は README の P18a の `n(O)` である。** P18a は「走査中の位置」を D27 に従って節点の訪問の
入口に取り、`n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と置く。内側の和を活性な名前に制限してよいのは、
`p13` の `L11` (ii) より活性でない名前の `B_ρ` が 0 だからであり、`obj_ρ(o)` が定まるのも活性な名前に
ついてだけである。

### DEF 消費点

D9 の消費の表の行が指す位置を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

### この文書が置く補助前提

次の 3 つは README の D・A・P で名指せない言明であり、この文書は証明しない。第 10 節が、それぞれを誰が
どこで果たせるかを述べる。

- **H1 (類への分解)**。`ρ` の上の節点 `q` と計数下オブジェクト `O` について
  `N(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` である。和は `obj(C) = O` である別名類 (`p13` の
  `DEF 別名類`) を渡る。読む所: `L42`。
- **H2 (借りた終端の類は参照を持ち続ける)**。ρ-終端が借用する (D14) パラメータ・capture の leaf である
  計数下の別名類 `C` について、活性化の各時点 `τ` で `held_ρ(τ, C) ≥ 1` である。読む所: `L42` の
  <1>2 の <2>3。
- **H3 (呼び出し先の活性化の対応)**。`Let(x, App(callee, args), k)` の実行について、`α` が作る呼び出し
  先の活性化と `α'` が作る呼び出し先の活性化は、同じ値を返し、完了した時点で各計数下オブジェクトの参照
  カウントを同じだけ変え、同じオブジェクトの集合を解放する。読む所: `L44` の <1>4 の <2>5 の <3>1。

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
<1>5. `CT` の要素の `NodeId` と `un_bump_releases[t]` の要素の `NodeId` は相異なる。前者は `Retain`
      節点、後者は `Release` 節点であり (<1>2、<1>3)、この 2 つは `B` の相異なる位置なので、P15 の
      前半より相異なる `NodeId` を持つ。
  BY <1>2, <1>3, P15
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

### L33a (節点から辿った列の最後は継続終端である)

**言明** --- 節点 `n` から D3 の 3 つの規則を適用して得られる列は有限であり、その最後の節点は `ret(n)`
である。とくに、本体 `B` の実行路の最後の節点は `ret(B の根)` であり、アーム本体 `arm.body` から辿った
列の最後の節点は `ret(arm.body)` である。

**証明** 部分木 `N(n)` の節点数についての帰納法で示す。DEF 子と親 より子の部分木は `N(n)` より真に
小さく、D2 より本体は有限の木なので、この帰納法は整礎である。

<1>1. CASE `n` の式が `RcExpr::Ret(_)` である。D3 より `Ret` からは辿る先が無いので列は `n` だけで
      あり、DEF 訪問 より `ret(n) = n` である。
  BY D3, DEF 訪問
<1>2. CASE `n` の式が `Let(_, Match(_, arms), k)` である。D3 のこの規則より、列はアームを 1 つ選んで
      その本体から辿った列を挟み、その後 `k` から辿った列へ続く。前者は帰納法の仮定より有限であり、
      後者の最後の節点は帰納法の仮定より `ret(k)` である。DEF 訪問 より `ret(n) = ret(k)` である。
  BY D3, DEF 訪問, DEF 子と親, 帰納法の仮定
<1>3. CASE `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` で
      ある。D3 のこの規則より、列は `n` の継続 `k` から辿った列へ続く。帰納法の仮定よりその最後の節点は
      `ret(k)` であり、DEF 訪問 より `ret(n) = ret(k)` である。
  BY D3, DEF 訪問, DEF 子と親, 帰納法の仮定
<1>4. QED
  `RcExpr` の 6 変位のうち `Ret` を <1>1、右辺が `Match` の `Let` を <1>2、残り (`Let` の残る形、
  `Retain`、`Release`、`Destructure`、`Eval`) を <1>3 が尽くす。本体の実行路は根から辿った列であり
  (D3)、アーム本体の実行路はそのアーム本体の根から辿った列である (D3)。
  BY <1>1, <1>2, <1>3, D3, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

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
      節点 `M` の継続 `k_M` である。L33a より `n = ret(arm_i.body)` である。`walk_inner` の `M` の腕は
      `arm_exits` を集めてから `let merged = self.merge(&pending, &arm_exits);` を作り、
      `self.walk(k, merged, returns_from_func)` を呼ぶので `pending(k_M) = merged` である。
      `arm_exits[j]` は `self.walk(&arm_j.body, ・, ・)` の返り値、すなわち `pending_out(arm_j.body)` で
      ある。L33 より `pending_out(arm_i.body) = pending(ret(arm_i.body)) = pending(n)` である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     p30 の L1, L33, L33a
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
<1>2. 走査のコードで `self.consume_objects(...)` と書かれているのは 3 か所である。`CancelAnalysis::consume`
      の末尾の `self.consume_objects(pending, &objects)`、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の `let others = self.other_objects(v, path);` の直後の
      `self.consume_objects(&mut pending, &others)`、および同じ腕の `UnBump::OutsideBracket` の枝の
      `self.consume_objects(&mut pending, &objects)` である。`consume_rhs`、`merge`、`cancelled`、
      `un_bump` は `consume_objects` を呼ばない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled,
     CODE src/rc_ir/borrow.rs: un_bump
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
<1>6. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕の 2 つの呼び出しは、<1>2 が挙げた形である。
      前者の `others` は `self.other_objects(v, path)` の値、すなわち DEF 節点の量 の `others(r)` で
      ある。後者の `objects` は `un_bumped.objects()` であり、`un_bumped = self.acted_references(v, path)
      = ActRefs(r)` である。よって 3 と 4 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     DEF 節点の量, <1>2
<1>7. QED
  <1>2 の 3 か所のうち `consume` の中の 1 か所を <1>3 が展開し、それが呼ばれる 2 か所を <1>4 と <1>5 が
  1 と 2 として尽くす。残る 2 か所を <1>6 が 3 と 4 として尽くす。
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
  <2>2. L33a より、`ρ` の最後の節点は `ret(B の根)` である。
    BY L33a
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
  <2>2. `I_ρ(t)` の上で、由来が `t` の要素の `outstanding` は、区間の最初の節点で `ActRefs(t)` であり
        (<1>1)、`ρ` の上で隣り合う 2 つの節点 `n`、`n'` (どちらも区間に入る) の間で次のように変わる。
        L34 の 1 では、`consume_objects` は残る要素の値を変えず (L36)、`un_bump` は `InBracket` で選んだ
        要素の `outstanding` からだけ引く (`p30` の `L5`)。よって `un_bump` が `InBracket(t)` を返す
        遷移では `ActRefs(n)` が引かれ、それ以外の遷移では変わらない。L34 の 2 では複製なので変わらない。
        L34 の 3 では、`n` はアーム `arm_i` の本体の終端の `Ret` であり、`merge` が
        `uniform.get(&retain.node)` の複製を新しい `outstanding` に据える。P18 よりその値はすべての
        アームの出口に現れる共通の値であり、L34 の 3 より `arm_exits[i] = pending(n)` なので、その共通の
        値は `out(t, n)` に等しい。よって変わらない。
    BY <1>1, L34, L36, p30 の L5, P18, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, DEF 節点の量
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

### L40 (`N` は類ごとの bump の和である) --- H1 として置く

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`ρ` の上の節点 `q` と計数下 (D26) の
オブジェクト `O` について、`N(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` である。ここで `bumps_ρ` と
`obj(C)` は `p13` の `DEF 類ごとの参照` と `DEF 別名類` のものである。

**この文書はこれを証明しない。** 第 2 節の H1 として置き、`L42` だけが読む。同じ言明は
`p13-disposals-and-pending.md` の第 7.5.4 節の <1>2 が述べて示しているが、それは別の命題の証明の内部の
ステップであり、記法の規約はそれを引用することを禁じる。第 10 節の 差し戻し 1 が、これを `p13` の
名前付きの補題に上げることを求める。

### L41 (余りの下界)

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト
`O` について、`N(q, O) ≥ 1` ならば、`q` の位置での参照カウントは `H(q, O) ≥ N(q, O) + 1` である。

**証明**

<1>1. P18a の「走査中の位置」を節点 `q` の訪問の入口に取ると、その `n(O)` は `N(q, O)` である。
  BY P18a, D27, DEF N
  D27 は `B(p, ρ)` を節点の訪問の入口で定め、P18a はその `B(p, ρ)` を使って
  `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と置く。DEF N がその和である。
<1>2. QED
  BY P18a, <1>1
  P18a は `n(O) ≥ 1` のとき `H(O) ≥ n(O) + 1` を述べる。

### L42 (義務は pending の bump を覆う) --- H1 と H2 の下で

**言明** --- H1 と H2 の下で、実行路 `ρ` を辿る活性化の各時点 `τ` と各計数下オブジェクト `O` について、
`Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。とくに `ρ` の上の節点 `q` の入口では
`Obl(q, O) ≥ N(q, O)` である。**これは README の P18c である。**

時点と走査の状態の対応は、`p13` の `DEF 類ごとの参照` と A19 が置くものを使う。すなわち、実行時の 1 つの
事象 (D10 の行が定める参照の作成・処分) の直後の時点には、走査がその事象に対応する操作を行った直後の
`pending` が対応する。

**局所の定義 (類ごとの義務)**。別名類 `C` について、`β(C)` を、`C` の ρ-終端が借用する (D14) パラメータ・
capture の leaf であるとき 1、そうでないとき 0 と定め、各時点 `τ` について
`obl_ρ(τ, C) := held_ρ(τ, C) - β(C)` と置く。

**証明**

<1>1. 計数下のオブジェクト `O` と各時点 `τ` について `Obl(τ, O) = Σ_{C : obj(C) = O} obl_ρ(τ, C)` である。
  <2>1. D10 が `Obl` を変える事象は、初期値、`Retain`、`Release`、生成、消費の 5 種である。移動は `Obl` を
        変えない。D26 より、数えるのは計数下のオブジェクトへの参照だけである。
    BY D10, D26
  <2>2. これらの事象はいずれも 1 つの inhabited な leaf に紐づき、その leaf は `ρ` の上のスロット (D6)
        であって、ちょうど 1 つの別名類に属する。
    BY D6, D10, p13 の DEF 別名類
  <2>3. `p13` の `DEF 類ごとの参照` の表の 6 行は、この 5 種の事象と次のように対応する。第 1 行が生成、
        第 2 行と第 3 行が初期値 (所有する場合と借用する場合)、第 4 行が `Retain`、第 5 行が `Release`、
        第 6 行が消費である。
    BY p13 の DEF 類ごとの参照, D10
  <2>4. D10 の生成の表の 5 行が名指す leaf は、いずれもその類の ρ-終端 (`p13` の `DEF ρ-歩みと ρ-終端`)
        である。
    <3>1. D10 の生成の 5 行が名指す値の束縛は順に、`Binding::Llvm` (宣言が単一の `Arg` でない leaf)、
          `RcRhs::App` の `Binding::Producer`、`RcRhs::Closure` の `Binding::Producer`、boxed 容器の
          `Binding::Field`、boxed scrutinee の `Binding::Payload(_, Some(tag))` である。
      BY D10, CODE src/rc_ir/ownership.rs: collect_bindings
    <3>2. `Binding::Producer` の腕、`container.ty.is_box` が真の `Binding::Field` の腕、
          `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` の腕は、いずれも `here()` を返し
          `origin` を呼ばない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>3. `Binding::Llvm` の腕は、boxed leaf `λ` について
          `decl.leaf_origins_at(λ).and_then(as_arg_projection)` が `None` のとき
          `origin_from_leaves_under(vars, type_env, &decl, args, λ, &here_identity)` を呼ぶ。この呼び
          出しは `origin` を呼ばない。
      <4>1. A3 より、このコミットのすべての op の宣言は結果の各 leaf に元数 0 か 1 の `LeafOrigins` を
            与える。元数 1 でその元が `LeafOrigin::Arg` ならば `as_arg_projection` は `Some` を返すので、
            この場合の `λ` の宣言は空集合か、`Fresh` か `Unknown` ただ 1 つである。
        BY A3, CODE src/rc_ir/ownership.rs: as_arg_projection
      <4>2. `decl.leaf_origins_under(λ)` が渡す集合は `λ` 自身の宣言だけである。
        BY p13 の L7, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
           CODE src/rc_ir/provenance.rs: Provenance::build_shape
        `build_shape` は型の各 boxed leaf を鍵に値を置き、`leaf_origins_under(path)` は鍵が `path` を
        前置に持つ元を渡す (その doc が「A path that is itself a leaf yields that leaf」と述べる)。
        `p13` の `L7` より boxed leaf の路は反鎖をなすので、`λ` を前置に持つ boxed leaf は `λ` 自身
        だけである。
      <4>3. QED
        BY <4>1, <4>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
        <4>1 と <4>2 より `operand_units` は空である。`origin_from_leaves_under` が `origin` を呼ぶのは
        `operand_units` の各元についてだけである。
    <3>4. QED
      BY <3>1, <3>2, <3>3, p13 の DEF ρ-歩みと ρ-終端
      `p13` の `DEF ρ-終端` は「`origin_inner` が `origin` を呼ばないとき」を ρ-終端という。
  <2>5. 計数下の `O` について `obj(C) = O` である類 `C` の ρ-終端は、D10 の生成が作る leaf か、パラメータ・
        capture の leaf である。
    <3>1. ρ-終端に当たる `origin_inner` の腕は、`None`/`Param`/`Producer` の腕、`Binding::Llvm` の
          宣言が単一の `Arg` でない腕、`container.ty.is_box` が真の `Binding::Field` の腕、
          `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` の腕の 4 つである。残る腕 --
          `Move`、`Join`、単一 `Arg` の `Llvm`、`is_box` が偽の `Field`、catch-all と `is_box` が偽の
          変位の `Payload` -- はいずれも `origin` を呼ぶ。`Join` の腕が `origin` を呼ぶことは A9 が
          与える (アームが 1 つ以上あるので `arm_results` は空でない)。
      BY CODE src/rc_ir/ownership.rs: origin_inner, A9, <2>4, p13 の DEF ρ-歩みと ρ-終端
    <3>2. `None` の腕に当たるのは束縛表が持たない名前、すなわちグローバル値である。A8 と D26 より、その
          leaf が指すオブジェクトはグローバル状態であり、計数下ではない。
      BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: VarTable::of, A8, D26
    <3>3. QED
      BY <3>1, <3>2, <2>4, CODE src/rc_ir/ownership.rs: VarTable::of
      `Binding::Param` を置くのは `VarTable::of` がパラメータと capture について行う 1 か所だけなので、
      `Param` の腕に当たるのはパラメータ・capture の leaf である。残る 3 つの腕は <2>4 の生成の leaf で
      ある。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, D10, D14, 本補題の局所の定義
    <2>3 の対応で、D10 の各事象は `held_ρ` と `Obl` に同じ増減を起こす。違うのは初期値の 2 行だけで
    ある。所有する (D14) パラメータ・capture の leaf では、D10 の初期値が参照を 1 つ入れ、`held_ρ` の
    第 2 行も 1 から始まるので、両者は等しい (`β = 0`)。借用する leaf では、D10 は参照を入れないのに
    `held_ρ` の第 3 行は 1 から始まるので、`Obl` の側は 1 少ない (`β = 1`)。生成の leaf では D10 が
    参照を 1 つ入れ、`held_ρ` の第 1 行も 1 から始まる (`β = 0`)。<2>5 より計数下の `O` を指す類の
    ρ-終端はこの 3 つのいずれかなので、`obl_ρ = held_ρ - β` の総和が `Obl` である。
<1>2. 各時点 `τ` と、`obj(C) = O` である各別名類 `C` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  <2>1. `bumps_ρ(τ, C) ≥ 0` である。
    BY p13 の DEF 類ごとの参照, p13 の L11
    `bumps_ρ(τ, C)` は `B_ρ` の個数の総和であり、`p13` の `L11` より各個数は 0 以上である。
  <2>2. CASE `β(C) = 0`。A19 の (ii-b) より `held_ρ(τ, C) ≥ bumps_ρ(τ, C)` であり、局所の定義より
        `obl_ρ(τ, C) = held_ρ(τ, C)` である。
    BY A19, 本補題の局所の定義
  <2>3. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) ≥ 1`。A19 が (ii-b) の脇に書く導かれた形 --「bump が 1 以上
        ある時点では、参照は bump より 1 つ多い」-- より `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であり、
        局所の定義より `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ bumps_ρ(τ, C)` である。
    BY A19, 本補題の局所の定義
  <2>4. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) = 0`。H2 より `held_ρ(τ, C) ≥ 1` であり、局所の定義より
        `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ 0 = bumps_ρ(τ, C)` である。
    BY H2, 本補題の局所の定義
  <2>5. QED
    局所の定義より `β(C)` は 0 か 1 であり、<2>1 より `bumps_ρ(τ, C)` は 0 か 1 以上である。<2>2、
    <2>3、<2>4 はこの 3 つの場合を尽くす。
    BY <2>1, <2>2, <2>3, <2>4
<1>3. QED
  BY <1>1, <1>2, H1
  <1>1 と <1>2 より `Obl(τ, O) = Σ_C obl_ρ(τ, C) ≥ Σ_C bumps_ρ(τ, C)` である。節点 `q` の入口では
  H1 よりこの右辺は `N(q, O)` である。

#### H2 が要る所

<1>2 の 3 つの場合のうち、A19 だけで閉じるのは <2>2 と <2>3 である。<2>4 -- 借用する終端を持つ類で、
その時点に pending な bump が 1 つも無い場合 -- が残る。A19 の (ii-a) はその類の `held_ρ` が非負であることしか
言わないので、`held_ρ(τ, C) = 0` が許される。そのとき `obl_ρ(τ, C) = -1` である。

**それが総和を割るのは 1 つの形である。** 1 つの計数下オブジェクト `O` を指す類が 3 つあり、2 つは借用する
終端を持って `held_ρ = 0` に落ちており、1 つは `bumps_ρ = 1`、`held_ρ = 2` であるとする。このとき
`Obl(τ, O) = (-1) + (-1) + 2 = 0` であるのに `Σ_C bumps_ρ(τ, C) = 1` であり、言明が破れる。`bumps ≥ 1` の
類を 1 つ選んで残りに `held ≥ bumps` だけを使う場合分け (`p13` の第 7.5.4 節の <1>3 がとる形) では、
この差は埋まらない -- そこでの余りは 1 つだけで、`β` が 1 の類の個数だけ引かれるからである。

**H2 は `borrow_ify` の出力についての言明として証明できる見込みがある。** 借用する終端を持つ類の
`held_ρ` を減らす事象は 2 つしかない -- その類のスロットを名指す `Release` と、その類のスロットの消費で
ある。前者は借用版では `rewrite_rc` が丸ごと落とし (P10、P7a)、後者には `call_rc` が直前に `Retain` を
置く (P11、`p13` の `L16`)。第 10 節の 差し戻し 2 が、これをどこに置くかを述べる。

### L43 (欠損は pending の bump の一部である)

**言明** --- 対応する活性化を固定する。`ρ` の上の位置 `q` と計数下のオブジェクト `O` について

`d(q, O) = Σ_{t} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]`

である。ここで外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り、内側の和は `ρ` で
活性な名前 `o` を渡る。とくに `0 ≤ d(q, O) ≤ N(q, O)` である。

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
  D27 は、`Retain` の訪問で押し込まれた要素の `B` を `ActRefs^inh_ρ(t)` と定め、`un_bump` が
  `InBracket` でその要素を選ぶ `Release` の訪問でだけ `ActRefs^inh_ρ` を引き、複製・`merge`・その他の
  節点では値を運ぶだけである。その要素を選ぶ `Release` は L38 の 3 の `R_ρ(t)` の要素である。
<1>5. 名前ごとの寄与をオブジェクトごとの寄与に直す。`Retain`/`Release` が実際に作る (処分する) 参照は、
      その名前 `o` について、オブジェクト `obj_ρ(o)` への参照である。よって `O` への寄与は
      `obj_ρ(o) = O` である名前 `o` の分の和である。
  BY p13 の DEF 名前の活性, p13 の DEF 実行時の作用, P5
<1>6. 言明の等式が成り立つ。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, DEF 欠損
<1>7. QED
  BY <1>6, p13 の L11, DEF N
  `p13` の `L11` より、活性な `o` について `B_ρ(q, e)[o] = e.outstanding[o] ≥ 0` であり、活性でない `o`
  について `B_ρ(q, e)[o] = 0` である。よって <1>6 の和は 0 以上である。DEF N の `N(q, O)` は同じ内側の和を
  `pending(q)` の**すべての**要素について取ったものなので、`d(q, O) ≤ N(q, O)` である。
