# P19 - P24 と主定理 T: `cancel` が RC 規律を保存すること

この文書は README の層 4 の 6 命題 P19, P20, P21, P22, P23, P24 と、主定理 T を扱う。README の定義
D1 - D19、仮定 A1 - A14、および命題 P1 - P18 の**言明**の上に立つ。P1 - P14 の証明は
`p10-leaves-and-units.md`、`p11-origin-soundness.md`、`p12-identity-and-consumes.md`、`p20-borrow-ify.md`
にあり、P15 - P18 の証明は `p30-cancel-walk.md` にある。この文書はそれらの言明だけを使う。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P19 | 証明済み。ただし言明に「`t` より後の」の限定が要る (第 11 節の 差し戻し 1) |
| P20 | 証明済み。A3 の単一 `Arg` の行を inhabited についての同値として読む (第 11 節の 差し戻し 2) |
| P21 | **閉じない。** 局所の補題 L22 に帰着したが、L22 自身が閉じない。L22 は L20、L21、および pending と実行時の参照カウントを結ぶ不変条件 (N4) を要し、この 3 つは README の D/A/P からは出ない |
| P22 | 証明済み |
| P23 | 構成は書けた。(S-a)、(S-b)、(S-c) のいずれも L22 を引用するので、P21 と同じ所で止まる |
| P24 | 証明済み |
| T | 1 は P14 がコードで偽 (#530) なので閉じない。2 は P27 が未証明、3 は P26 が未証明。4 は閉じる |

**P5 (c) を使った所**: 第 6 節の「L22 について分かっていること」の項目 1 と、その次の段落 (消費が
`d(q, o) ≥ 1` の区間に現れないこと) である。どちらも、ある構文が処分する参照の名前がその構文の
`acted_unit_keys` に (`unit_of` を通して) 現れることを言うのに使う。README は P5 (c) を「このコミットの
コードでは偽」と書くが、その原因の #529 は `be26b396` で直っており、言明は使ってよい。

## 1. 記法

1 つの関数 (またはグローバル初期化子) の本体 `B` を固定し、`B` から作られる `VarTable` を `vars`、
プログラムの `TypeEnv` を `type_env` と書く (`CODE src/rc_ir/ownership.rs: VarTable::of`,
`CODE src/rc_ir/ownership.rs: VarTable::body_only`)。この 2 つは本体ごとに 1 つなので、以下では
`origin`、`unit_key`、`acted_references`、`acted_unit_keys` の第 1・第 2 引数を落として書く。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。
- `id(x, π)` は `origin(x, π).identity()` (`CODE src/rc_ir/ownership.rs: Origin::identity`)。
- `key(x, π)` は `unit_key(vars, type_env, x, π)` (D15)。
- `unit_of(n)` は `unit_of(vars, type_env, n)` (`CODE src/rc_ir/ownership.rs: unit_of`)。よって
  `key(x, π) = unit_of(id(x, π))` である。
- `L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合。D4 より、これが
  「`v` の `π` の下の boxed leaf」の全体であり、inhabited (D16) でないものを含む。
- 1 つの活性化の 1 つの実行路の上で `v` が束縛された後、`L(v, π)` の元のうち inhabited なものの集合を
  `Linh(v, π)` と書く。位置を引数に取らない理由は L18 の <1>1 が述べる。

補題の番号について。**`L1` から `L10` は `p30-cancel-walk.md` の補題**を指す。この文書が導入する補題は
`L11` から始める。`DEF <名前>` はこの文書の第 2 節が導入する語を指す。

多重集合の記法。`References` は `Map<VarPath, usize>` を 1 つ持つ構造体であり
(`CODE src/rc_ir/ownership.rs: References`)、これを鍵を `VarPath`、値をその個数とする多重集合とみなす。
和 `R1 + R2`、差 `R1 - R2` を各鍵の個数の和・差とする。

## 2. 局所の定義

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。節点 `n` の訪問における `pending` 引数の値
(その訪問がそれに変更を加える前の値) を `pending(n)` と書き、**入口状態**と呼ぶ。その訪問の戻り値を
`pending_out(n)` と書き、**出口状態**と呼ぶ。P15 の後半より、走査は `B` の各位置をちょうど 1 回訪問するので、
`pending(n)` と `pending_out(n)` は節点ごとに 1 つに定まる。

節点 `n` の**継続終端** `ret(n)` を、`n` から D2 の意味の継続 (`Match` の場合はアーム本体ではなく `k`) を
たどって到達する `Ret` 節点とする。D2 より継続の鎖は有限で `Ret` で終わるので、`ret(n)` は 1 つに定まる。

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

### DEF 名前づけ

`Retain(v, π)` が inhabited な leaf `λ` について作る参照の**名前**を `id(v, λ)` とし、`Release(v, π)` が
leaf `λ` について処分する参照の名前も `id(v, λ)` とする。P6 (b) はこの対応の下で、実行時の参照の
多重集合と `VarPath` 上の多重集合の一致を述べる。

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、

- `key(t) := key(v.name, path)`、`bumped(t) := acted_references(v, path)`
- `key(r) := key(v.name, path)`、`un_bumped(r) := acted_references(v, path)`
- `others(r) := acted_unit_keys(v.name, path)` の要素のうち `key(r)` と異なるもの

`CancelAnalysis::unit_key`、`CancelAnalysis::acted_unit_keys`、`CancelAnalysis::acted_references` は
`ownership` の同名関数に `self.vars` と `self.type_env` を渡して呼ぶ
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::unit_key`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_unit_keys`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。これらの値は `vars`、`type_env`、
および渡された `(変数, path)` だけで決まるので、走査のどの時点で読んでも同じ値である。

### DEF 削除集合

`cancel_body` の 1 回の実行について、`analysis.cancelled()` が返す集合を `Del` と書く
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled`)。
`self.all_retains` の要素のうち、`self.needed_retains` に入らず、`self.un_bump_releases` の値が空でない
ものの全体を `CT` と書く。

### DEF 出力の本体

`B` に対する `cancel_body` の返り値 `drop_nodes(B, Del)` を `B'` と書く
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: drop_nodes`)。

### DEF 路の対応

P22 が示すとおり `B'` は `B` から `Del` の節点を抜いた木である。`B` の実行路 `p` (D3) から `Del` の節点を
除いた列を `p'` と書き、`p` に**対応する** `B'` の実行路と呼ぶ。P22 の <1>5 がこれが `B'` の実行路であること
と、対応が全単射であることを示す。以下、位置についての主張は、`p` と `p'` を、対応する節点で並べて比べる。

### DEF 欠損

1 つの活性化と 1 つの実行路 `p` を固定する。`p` の上の位置 `q` とオブジェクト `o` について、

`d(q, o) :=` (`q` までに実行された `Del` の `Retain` 節点が `o` への参照を作った個数)
`-` (`q` までに実行された `Del` の `Release` 節点が `o` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が「作る」「処分する」個数を定める。

### DEF 消費点

D9 の消費の表の行が指す位置を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

## 3. 局所の補題

### L11 (削除集合の構造)

**言明** — 次の 4 つが成り立つ。

1. `CT` の各要素は、走査が訪問した `Retain` 節点の `NodeId` である。
2. `t ∈ CT` について、`un_bump_releases[t]` の各要素は、走査が訪問した `Release` 節点の `NodeId` である。
3. `Del` は `CT` と `⋃_{t ∈ CT} un_bump_releases[t]` の非交和であり、後者の族も互いに素である。
4. `Release` 節点 `r` の `NodeId` が `un_bump_releases[t]` に入るのは、`r` の訪問の中の `un_bump` の
   呼び出しが `InBracket(t)` を返したとき、かつそのときに限る。

**証明**

<1>1. `cancelled` は `self.all_retains` の各要素 `retain` を回り、`self.needed_retains` がそれを含むときは
      飛ばし、含まないときは `self.un_bump_releases.get(&retain)` を引き、その `Vec` が空でないときだけ
      `retain` とその `Vec` の全要素を `out` に入れる。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled
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
<1>4. 1 つの `Release` 節点の `NodeId` は、高々 1 つの `t` の `un_bump_releases[t]` に、高々 1 回入る。
  <2>1. P15 の後半より、走査は `Release` 節点 `r` をちょうど 1 回訪問する。
    BY P15
  <2>2. `r` の訪問は `un_bump(&mut pending, &key, &un_bumped)` を 1 回だけ評価し、その値に対する `match`
        の `InBracket` の枝は高々 1 回実行される。`un_bump` の返り値は `UnBump` の 1 つの変位であり、
        `InBracket` は `NodeId` を 1 つだけ運ぶ。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: UnBump
  <2>3. QED
    BY <1>3, <2>1, <2>2
<1>5. `CT` の要素と `un_bump_releases[t]` の要素は相異なる。前者は `Retain` 節点の、後者は `Release` 節点の
      `NodeId` であり、この 2 つは `B` の相異なる位置なので、P15 の前半より相異なる `NodeId` を持つ。
  BY P15, <1>2, <1>3
<1>6. QED
  3 は <1>1、<1>4、<1>5 から従う。1 は <1>2、2 は <1>3、4 は <1>3 である。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L12 (出口状態は継続終端の入口状態である)

**言明** — 任意の節点 `n` について、`pending_out(n) = pending(ret(n))` である。

**証明** `n` から `ret(n)` への継続の鎖の長さについての帰納法で示す。D2 より鎖は有限である。

<1>1. CASE `n` の式が `RcExpr::Ret(_)` である。`walk_inner` のこの腕は `pending` をそのまま返すので
      `pending_out(n) = pending(n)` であり、DEF 訪問 より `ret(n) = n` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, DEF 訪問
<1>2. CASE `n` の式が `RcExpr::Ret(_)` でない。`walk_inner` の残る 6 つの腕はいずれも
      `self.walk(k, ・, ・)` の値をそのまま返す (`k` は `n` の継続)。L1 より `walk` は `walk_inner` を
      1 回呼んでその値を返すので、`pending_out(n) = pending_out(k)` である。DEF 訪問 より
      `ret(n) = ret(k)` なので、帰納法の仮定が使える。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, L1, DEF 訪問
<1>3. QED
  `RcExpr` の 6 変位のうち `Ret` を <1>1 が、残る 5 変位 (`Let` は右辺で 2 つの腕に分かれるが、どちらも
  `self.walk(k, ・, ・)` を返す) を <1>2 が尽くす。
  BY <1>1, <1>2, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L13 (路に沿った状態の遷移)

**言明** — 実行路 `p` (D3) の上の節点 `n` と、`p` の上で `n` の直後にある節点 `n'` について、次のいずれかが
成り立ち、この 3 つは場合を尽くす。

1. `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` である。
   このとき `n'` は `n` の継続 `k` であり、`pending(n')` は `pending(n)` にその腕が行う基本操作の列を
   施したものである。
2. `n` の式が `Let(_, Match(_, arms), k)` である。このとき `n'` は `p` が選んだアーム `arm_i` の本体で
   あり、`pending(n') = pending(n)` である。
3. `n` がアーム本体の終端の `Ret` である。そのアームを `arm_i`、その `Match` 節点を `M` とすると
   `n'` は `M` の継続 `k_M` であり、`pending(n') = merge(pending(M), arm_exits)` の値である。ここで
   `arm_exits[j] = pending_out(arm_j.body)` であり、`arm_exits[i] = pending(n)` である。

**証明**

<1>1. CASE `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` で
      ある。D3 より `p` の上の `n` の直後の節点は `n` の継続 `k` である。`walk_inner` のこれらの腕は、
      `pending` に 0 個以上の操作を施したうえで `self.walk(k, pending, returns_from_func)` を呼ぶので、
      `pending(k)` はその施した後の値である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, L1
<1>2. CASE `n` の式が `Let(_, Match(_, arms), k)` である。D3 より `p` の上の `n` の直後の節点は、`p` が
      選んだアームの本体である。`walk_inner` のこの腕は各アームについて
      `self.walk(&arm.body, pending.clone(), false)` を呼ぶので、アーム本体の入口状態は `pending(n)` の
      複製、すなわち `pending(n)` と等しい値である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕, L1
<1>3. CASE `n` がアーム本体の終端の `Ret` である。D3 より、`Ret` の後に実行路が続くのは、`n` が、その
      実行路が入ったアームの本体の実行路を終える `Ret` であるときに限り、そのとき直後の節点はその `Match`
      節点 `M` の継続 `k_M` である。`walk_inner` の `M` の腕は `arm_exits` を集めてから
      `let merged = self.merge(&pending, &arm_exits);` を作り、`self.walk(k, merged, returns_from_func)`
      を呼ぶので `pending(k_M) = merged` である。`arm_exits[j]` は `self.walk(&arm_j.body, ・, ・)` の
      返り値、すなわち `pending_out(arm_j.body)` である。L12 より
      `pending_out(arm_i.body) = pending(ret(arm_i.body)) = pending(n)` である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     L1, L12
<1>4. QED
  `p` の上に直後の節点があるのは、`n` の式が `Ret` でないか、`n` がアーム本体の終端の `Ret` であるかの
  どちらかである (D3)。前者を <1>1 と <1>2 が `RcExpr` の変位で尽くし、後者を <1>3 が扱う。
  BY <1>1, <1>2, <1>3, D3, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L14 (`pending` の要素はその位置を支配する)

**言明** — 走査が訪問する節点 `n` と `pending(n)` の要素 `e` について、`e.node` を `NodeId` に持つ `Retain`
節点を `t` とする (P16 の (a) と P15 の前半より、そのような節点はちょうど 1 つある)。このとき、`n` を含む
すべての実行路は `t` を含み、その路の上で `t` は `n` より真に前にある。

**証明** 訪問順序についての帰納法で示す。P15 の後半より走査は各位置をちょうど 1 回訪問するので、訪問順序は
有限の全順序であり、この帰納法は整礎である。

<1>1. 帰納法の仮定: `n` より前に訪問された各節点 `m` と `pending(m)` の各要素について、言明が成り立つ。
  BY 帰納法の仮定
<1>2. 木の各節点 `n` について、`n` を含むすべての実行路は、`n` の親 (DEF 子と親 の意味での親、すなわち
      `n` を子に持つ唯一の節点) を含み、その路の上で親は `n` より真に前にある。
  <2>1. D2 より本体は木であり、根でない各節点はちょうど 1 つの親を持つ。
    BY D2
  <2>2. D3 の 3 つの規則のうち、実行路に `n` を加えるのは次の 3 つの場合だけである。`n` が根である
        場合、`n` が直前の節点の継続である場合、`n` がアーム本体の根である場合。第 2 の場合の直前の節点は
        `n` の親か、`n` の親が `Match` であるときそのアーム本体の終端の `Ret` であり、後者のときも
        D3 よりその `Match` 節点自身が路の上でより前にある。第 3 の場合の直前の節点は `n` の親である
        `Match` 節点である。
    BY D3
  <2>3. QED
    BY <2>1, <2>2
<1>3. CASE `n` が `B` の根である。`cancel_body` は `analysis.walk(body, PendingRetains::default(), true)`
      を呼ぶので `pending(n)` は鍵を 1 つも持たず、言明は空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>4. CASE `n` が根でなく、その親 `m` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が
      `Match` でない `Let` である。
  <2>1. `walk_inner` の `m` の腕は `pending(m)` に操作を施して `self.walk(n, pending, ・)` を呼ぶ。
        `pending` に要素を加えるのは `RcExpr::Retain` の腕の `push` 1 か所だけであり、そこで加わる要素の
        `node` は `m` 自身の `NodeId` である。ほかの腕 (`Release` の `consume_unit` と `un_bump`、`Let` の
        `consume_rhs`、`Destructure` の `consume`) は要素を加えない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit,
       CODE src/rc_ir/borrow.rs: un_bump,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume
  <2>2. `pending(n)` の要素は、`pending(m)` の要素であるか、`m` 自身 (`m` が `Retain` のとき) である。
    BY <2>1
  <2>3. QED
    `pending(m)` の要素については <1>1 と <1>2 を合わせる。`m` 自身については <1>2 が直接与える。
    BY <1>1, <1>2, <2>2
<1>5. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n` がその
      アームの本体である。L13 の 2 より `pending(n) = pending(m)` なので、<1>1 と <1>2 を合わせる。
  BY <1>1, <1>2, L13
<1>6. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n = k` で
      ある。L13 の 3 より `pending(n) = merge(pending(m), arm_exits)` である。P18 の第 1 の主張より、
      `merge` の返り値に残る要素の `node` は `pending(m)` の要素の `node` である。よって <1>1 と <1>2 を
      合わせる。
  BY <1>1, <1>2, L13, P18
<1>7. QED
  `n` は根であるか、親を持つ。親を持つ場合、DEF 子と親 の子の表より、親の式は <1>4、<1>5、<1>6 の
  いずれかの形である。
  BY <1>3, <1>4, <1>5, <1>6, D2

### L15 (削除される `Release` は `t` より後にある)

**言明** — `t ∈ CT` と `r ∈ un_bump_releases[t]` について、`r` の訪問の時点で `t` は `pending(r)` の要素で
あり、`r` を含むすべての実行路は `t` を含み、その路の上で `t` は `r` より真に前にある。

**証明**

<1>1. L11 の 4 より、`r` の訪問の中の `un_bump(&mut pending, &key(r), &un_bumped(r))` の呼び出しが
      `InBracket(t)` を返した。
  BY L11
<1>2. P17 より、`un_bump` が `InBracket(t)` を返すのは、その第 1 引数が鍵 `key(r)` の項目を持ち、その最内の
      要素の `node` が `t` であるときである。その第 1 引数は `r` の訪問が `un_bump` を呼ぶ時点の状態で
      あり、それは `pending(r)` に、この腕がそれより前に行う `others(r)` についての `consume_unit` を
      施したものである。`consume_unit` は要素を加えないので、`t` は `pending(r)` の要素でもある。
  BY P17, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit
<1>3. QED
  BY <1>2, L14

### L16 (`t` が pending である区間)

**言明** — `t ∈ CT` と、`t` を含む実行路 `p` を取る。`p` の上の節点 `n` について「`t` が `pending(n)` の
要素である」が成り立つ `n` の全体は、`p` の上の空でない連続する区間であり、その最初の節点は `p` の上で
`t` の直後にある節点である。この区間の最後の節点を `n*(p)` と書くと、次が成り立つ。

1. `n*(p)` は `Release` 節点であり、その訪問の `un_bump` は `InBracket(t)` を返し、その `subtract` の後に
   `t` の要素の `outstanding` が空になる。よって `n*(p) ∈ un_bump_releases[t]` である。
2. `n*(p)` は `p` の終端の `Ret` より真に前にある。
3. `p` の上でこの区間に入る `Release` 節点のうち、その訪問の `un_bump` が `InBracket(t)` を返すもの全体を
   `R_p(t)` と書くと、`R_p(t) ⊆ un_bump_releases[t]` であり、`Σ_{r ∈ R_p(t)} un_bumped(r) = bumped(t)`
   である。
4. `p` の上でこの区間に入る位置では、`consume_unit(・, key(t))` は 1 度も呼ばれない。

**証明**

<1>1. `t` の訪問は、`pending` の鍵 `key(t)` の `Vec` の末尾に `PendingRetain { node: node_id(t),
      outstanding: bumped(t) }` を積んでから継続へ進む。よって `t` の直後の節点 `n0` について
      `t` は `pending(n0)` の要素であり、その `outstanding` は `bumped(t)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     DEF 節点の量, L13
<1>2. `p` の上で `t` より後の 2 つの節点 `n`、`n'` が `p` の上で隣り合い、`t` が `pending(n)` の要素で
      ないならば、`t` は `pending(n')` の要素でもない。
  <2>1. CASE L13 の 1。`pending(n')` は `pending(n)` に基本操作を施したものである。要素を加えるのは
        `Retain` の腕の `push` だけであり、そこで加わる要素の `node` は `n` 自身の `NodeId` である。
        `n` は `Retain` 節点 `t` とは相異なる位置なので、P15 の前半よりその `NodeId` は `t` の
        `NodeId` と異なる。
    BY L13, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕, P15
  <2>2. CASE L13 の 2。`pending(n') = pending(n)` である。
    BY L13
  <2>3. CASE L13 の 3。P18 の第 1 の主張より、`merge` の返り値に残る要素の `node` は、いずれのアームの
        出口にも現れるものだけである。`pending(n) = arm_exits[i]` は `t` を持たないので、`t` は
        `merge` の返り値にも入らない。
    BY L13, P18
  <2>4. QED
    BY <2>1, <2>2, <2>3, L13
<1>3. `t` が `pending(n)` の要素である `p` の上の節点 `n` の全体は、<1>1 の `n0` から始まる連続する区間で
      ある。
  BY <1>1, <1>2
<1>4. `t` が `p` の終端の `Ret` の入口状態の要素であることはない。
  <2>1. `cancel_body` は `analysis.walk(body, ・, true)` を呼び、`walk_inner` は `Match` のアーム本体に
        だけ `false` を渡し、ほかの継続には自分が受け取った `returns_from_func` をそのまま渡す。よって
        `returns_from_func` が真である節点の全体は、`B` の根から継続だけをたどって得られる鎖であり、
        その鎖に入る唯一の `Ret` 節点は `ret(B の根)` である。
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       DEF 訪問
  <2>2. D3 より、実行路の終端の `Ret` は `ret(B の根)` である。
    BY D3, DEF 訪問
  <2>3. `walk_inner` の `RcExpr::Ret(_)` の腕は、`returns_from_func` が真のとき `pending` の全要素の
        `node` を `self.needed_retains` に入れる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
  <2>4. QED
    `t` がそこで `pending` の要素なら <2>3 より `node_id(t)` が `needed_retains` に入り、L8 より走査の
    終わりまで残る。これは `t ∈ CT` に反する。
    BY <2>1, <2>2, <2>3, L8, DEF 削除集合
<1>5. 区間の最後の節点 `n*(p)` は `Release` 節点であり、その訪問の `un_bump` が `InBracket(t)` を返して
      `outstanding` を空にすることによって `t` が取り除かれる。
  <2>1. <1>3 と <1>4 より区間は `p` の終端の `Ret` より前で終わるので、区間の最後の節点 `n*` があり、
        `p` の上でその直後の節点 `n'` について `t` は `pending(n')` の要素でない。
    BY <1>3, <1>4
  <2>2. CASE L13 の 3 (`n*` がアーム本体の終端の `Ret`)。`pending(n*) = arm_exits[i]` が `t` を持ち、
        `merge` の返り値が持たない。P18 の第 2 の主張より、この `merge` の呼び出しは `node_id(t)` を
        `self.needed_retains` に入れる。L8 よりそれは走査の終わりまで残るので `t ∈ CT` に反する。
        よってこの場合は起こらない。
    BY L13, P18, L8, DEF 削除集合
  <2>3. CASE L13 の 2 (`n*` が `Match` 節点)。`pending(n') = pending(n*)` なので `t` は `pending(n')` の
        要素であり、`n*` が区間の最後であることに反する。よってこの場合は起こらない。
    BY L13
  <2>4. CASE L13 の 1。`walk_inner` のこれらの腕が `pending` から要素を取り除くのは、`consume_unit` と
        `un_bump` の 2 つだけである。
    <3>1. `consume_unit(pending, K)` は鍵 `K` があればその鍵と `Vec` を丸ごと取り除き、取り除いた `Vec`
          の各要素の `node` を `self.needed_retains` に入れる。よってこれが `t` を取り除いたなら
          `node_id(t)` が `needed_retains` に入り、L8 より走査の終わりまで残るので `t ∈ CT` に反する。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit, L8, DEF 削除集合
    <3>2. `un_bump` が `pending` から要素を取り除くのは P17 の第 3 の場合だけであり、そのとき取り除かれる
          のは鍵 `key` の最内の要素で、`outstanding` から `R` を引いた結果が空になったときに限る。
          その呼び出しは `InBracket` を返し、その `NodeId` は取り除かれた要素の `node` である。
      BY P17
    <3>3. QED
      BY <3>1, <3>2
  <2>5. QED
    `n*` は `p` の上に直後の節点を持つので、L13 の 3 つの場合のいずれかである。<2>2 と <2>3 がそのうち
    2 つを排除する。
    BY <2>1, <2>2, <2>3, <2>4, L13
<1>6. 1 が成り立つ。<1>5 の `un_bump` の呼び出しは `InBracket(t)` を返すので、L11 の 4 より
      `node_id(n*(p)) ∈ un_bump_releases[t]` である。
  BY <1>5, L11
<1>7. 2 が成り立つ。
  BY <1>3, <1>4, <1>5
<1>8. 3 が成り立つ。
  <2>1. `R_p(t)` の各要素は L11 の 4 より `un_bump_releases[t]` の要素である。
    BY L11
  <2>2. 区間の上で、`t` の要素の `outstanding` の値は、区間の最初で `bumped(t)` であり、L13 の各遷移で
        次のように変わる。L13 の 1 の `un_bump` が `InBracket(t)` を返す遷移では `un_bumped(r)` が引かれる
        (P17)。L13 の 1 のそれ以外の遷移と L13 の 2 の遷移では変わらない。L13 の 3 の遷移では、
        `merge` が `uniform.get(&retain.node)` の複製を新しい `outstanding` に据えるので、その値は
        `arm_states` の共通の値、すなわち `arm_exits[i]` すなわち `pending(n)` におけるこの要素の
        `outstanding` に等しく、やはり変わらない。
    BY <1>1, L13, P17, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
  <2>3. 区間の最後で `outstanding` は空になる (<1>5)。
    BY <1>5
  <2>4. QED
    <2>2 より `bumped(t)` から `R_p(t)` の各要素の `un_bumped` を引いた結果が空である。
    BY <2>1, <2>2, <2>3
<1>9. 4 が成り立つ。区間の上の位置で `consume_unit(・, key(t))` が呼ばれたとすると、P16 の (a) より
      `t` は鍵 `key(t)` の下に在るので、`consume_unit` はそれを取り除いて `node_id(t)` を
      `needed_retains` に入れる。L8 よりそれは走査の終わりまで残るので `t ∈ CT` に反する。
  BY P16, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit, L8, DEF 削除集合
<1>10. QED
  BY <1>3, <1>6, <1>7, <1>8, <1>9

### L17 (名前は inhabited を決める)

**言明** — 1 つの活性化の 1 つの実行路を固定する。その上で束縛された値 `v` と
`λ ∈ boxed_leaf_paths(ty(v), type_env)` について `id(v, λ) = (u, σ)` とする。このとき、`λ` が `v` に
おいて inhabited (D16) であることと、`σ` が `u` において inhabited であることは同値である。

**証明** `origin(v, λ)` の再帰についての帰納法で示す。P2 より `origin` は停止するので、この再帰の木は有限で
あり、帰納法は整礎である。以下、`origin_inner` の腕ごとに場合を分ける
(`CODE src/rc_ir/ownership.rs: origin_inner`)。

<1>1. CASE `vars.bindings.get(&v)` が `None`、`Some(Binding::Param)`、または `Some(Binding::Producer)` で
      ある。この腕は `here()` すなわち `Origin::Exactly((v, λ))` を返すので `(u, σ) = (v, λ)` であり、
      言明は同語反復である。
  BY CODE src/rc_ir/ownership.rs: origin_inner
<1>2. CASE `Some(Binding::Move(y))` である。この腕は `origin(y, λ)` を返す。D2 より
      `Let(x, Var(y), k)` は `y` の値を `x` に束縛するので、`v` の値と `y` の値は等しく、D16 の判定が
      読む union のタグも等しい。よって `λ` が `v` において inhabited であることと `y` において
      inhabited であることは同値であり、帰納法の仮定を `(y, λ)` に適用する。
  BY CODE src/rc_ir/ownership.rs: origin_inner, D2, D16, 帰納法の仮定
<1>3. CASE `Some(Binding::Join(arm_results))` である。この腕は各 `arm_result` について
      `origin(arm_result, λ).acted_on()` を集めて `Origin::of_candidates(candidates, (v, λ))` を返す。
  <2>1. CASE 返り値が `Origin::Join { identity, .. }` である。`of_candidates` はこのとき
        `identity` に第 2 引数をそのまま据えるので `(u, σ) = (v, λ)` であり、言明は同語反復である。
    BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
  <2>2. CASE 返り値が `Origin::Exactly((u, σ))` である。
    <3>1. `of_candidates` が `Exactly` を返すのは `candidates` が 1 元集合のときなので、各
          `arm_result` について `origin(arm_result, λ).acted_on()` は `{(u, σ)}` に含まれる。
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Origin::acted_on` は `identity` を先頭に置き、そこに `candidates` のうち `identity` と
          異なるものを足す。`Origin::Join` の `candidates` は `of_candidates` が 2 元以上のときにだけ
          作るので、`Join` の `acted_on` は 2 元以上を持つ。よって <3>1 より、各 `arm_result` の
          `origin(arm_result, λ)` は `Origin::Exactly((u, σ))` である。
      BY CODE src/rc_ir/ownership.rs: Origin::acted_on, CODE src/rc_ir/ownership.rs: Origin::of_candidates, <3>1
    <3>3. この実行路が選んだアームを `arm_i`、その結果変数を `x_i` とすると、D2 と D9 の移動の表の
          「`Match` のアーム本体の `Ret(x)`」の行より、`v` の値は `x_i` の値である。よって `λ` が `v` に
          おいて inhabited であることと `x_i` において inhabited であることは同値である。
      BY D2, D9, D16
    <3>4. QED
      BY <3>2, <3>3, 帰納法の仮定
  <2>3. QED
    `Origin` の変位は `Exactly` と `Join` の 2 つである。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: Origin
<1>4. CASE `Some(Binding::Llvm(llvm_gen, args, result_ty))` である。この腕は
      `decl.leaf_origins_at(λ).and_then(as_arg_projection)` で分岐する。
  <2>1. `λ` は `result_ty` の boxed leaf なので、`decl.leaf_origins_at(λ)` が `Some(s)` を返すとき、
        `s` は宣言がその leaf に与えた `LeafOrigins` である。A3 の末尾の段落より、このコミットの
        すべての op の宣言は各 leaf に 0 元または 1 元の集合を与えるので `|s| ≤ 1` である。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, A3
  <2>2. CASE `as_arg_projection` が `Some((j, σ'))` を返す。すなわち `s` は単一の `Arg(j, σ')` である。
        この腕は `origin(args[j], σ')` を返す。
    <3>1. `λ` が `v` において inhabited ならば、`σ'` は `args[j]` において inhabited である。
      <4>1. A5 より、inhabited な leaf `λ` に `v` は参照をちょうど 1 つ持つ。
        BY A5
      <4>2. A3 の「単一の `Arg(j, σ)`」の行より、生成コードが結果のその leaf に置くのは第 `j` オペランドの
            leaf `σ'` と同じ参照である。よって `args[j]` はその leaf `σ'` に参照を持つ。
        BY A3, <4>1
      <4>3. QED
        A5 の「inhabited でない leaf は参照を持たない」の対偶より `σ'` は inhabited である。
        BY A5, <4>2
    <3>2. `σ'` が `args[j]` において inhabited ならば、`λ` は `v` において inhabited である。
      <4>1. A5 より `args[j]` は leaf `σ'` に参照をちょうど 1 つ持つ。
        BY A5
      <4>2. A3 の「単一の `Arg(j, σ)`」の行より、生成コードは結果のその leaf `λ` にその参照を置く。よって
            `v` は leaf `λ` に参照を持つ。
        BY A3, <4>1
      <4>3. QED
        A5 の「inhabited でない leaf は参照を持たない」の対偶より `λ` は inhabited である。
        BY A5, <4>2
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: origin_inner, 帰納法の仮定
  <2>3. CASE `decl.leaf_origins_at(λ).and_then(as_arg_projection)` が `None` である。この腕は
        `origin_from_leaves_under(vars, type_env, &decl, args, λ, &(v, λ)).unwrap_or_else(here)` を返す。
    <3>1. `leaf_origins_under(λ)` が渡すのは `λ` 自身の `LeafOrigins` だけである。`λ` は leaf なので、
          その下に別の leaf は無い。
      BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under (doc: 「A path that is itself a leaf
         yields that leaf」), D4
    <3>2. <2>1 と <2>2 の場合分けより、この場合の `λ` の `LeafOrigins` は、空集合であるか、単一の
          `LeafOrigin::Fresh` または `LeafOrigin::Unknown` である。
      BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: as_arg_projection
    <3>3. CASE `λ` の `LeafOrigins` が空集合である。`operand_units` は空、`produced_here` は偽、
          `reached` は空なので `reached.first()?` が `None` を返し、`unwrap_or_else(here)` により
          `Origin::Exactly((v, λ))` になる。よって `(u, σ) = (v, λ)` であり、言明は同語反復である。
      BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under, <3>1, <3>2
    <3>4. CASE `λ` の `LeafOrigins` が単一の `Fresh` または `Unknown` である。`operand_units` は空、
          `produced_here` は真なので `reached = [Origin::Exactly((v, λ))]` であり、
          `reached.iter().all(...)` が真になって `Some(Origin::Exactly((v, λ)))` を返す。よって
          `(u, σ) = (v, λ)` であり、言明は同語反復である。
      BY CODE src/rc_ir/ownership.rs: origin_from_leaves_under, <3>1, <3>2
    <3>5. QED
      BY <3>2, <3>3, <3>4
  <2>4. QED
    BY <2>2, <2>3
<1>5. CASE `Some(Binding::Field(container, idx))` である。
  <2>1. CASE `container.ty.is_box(type_env)` が真。この腕は `here()` を返すので `(u, σ) = (v, λ)` で
        あり、言明は同語反復である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. CASE `container.ty.is_box(type_env)` が偽。この腕は `origin(container, [idx] ++ λ)` を返す。
    <3>1. A12 より `Destructure` の容器は構造体であり、その `idx` 番目のフィールドの型は `ty(v)` である。
          D9 の移動の表の「`Destructure(c, fs)` (`c` が unbox) の名前付きフィールド」の行より、`v` の値は
          `container` の値の第 `idx` フィールドである。
      BY A12, D9
    <3>2. D16 の判定は、leaf の道が通る unbox union の節のタグを読む。`[idx] ++ λ` が `container` に
          おいて通る unbox union の節は、`idx` が unbox 構造体のフィールド添字であることから、`λ` が
          `v` において通る節と同じ値の同じ節である。
      BY D16, D4, <3>1
    <3>3. QED
      BY <3>2, CODE src/rc_ir/ownership.rs: origin_inner, 帰納法の仮定
  <2>3. QED
    BY <2>1, <2>2
<1>6. CASE `Some(Binding::Payload(scrut, variant))` である。
  <2>1. CASE `variant` が `None` (catch-all)。この腕は `origin(scrut, λ)` を返す。D9 の移動の表の
        「catch-all アームの payload 束縛」の行より、payload 変数の値は scrutinee の値である。よって
        D16 の判定が読むタグも等しい。
    BY CODE src/rc_ir/ownership.rs: origin_inner, D9, D16, 帰納法の仮定
  <2>2. CASE `variant` が `Some(tag)` で `scrut.ty.is_box(type_env)` が偽。この腕は
        `origin(scrut, [tag] ++ λ)` を返す。
    <3>1. A11 より、payload 変数 `v` の使用はその変位アームの本体の中にあり、この実行路はそのアームを
          選んでいる。A4 より `Match` は実行時のタグで分岐するので、この実行路の上で `scrut` の値の
          タグは `tag` である。
      BY A11, A4, D3
    <3>2. D16 より、`[tag] ++ λ` が `scrut` において inhabited であることは、`scrut` の値のタグが
          `tag` に等しく、かつ `λ` が payload において inhabited であることと同値である。<3>1 より前者は
          この実行路の上で成り立つ。D9 の移動の表の「unbox union の変位アームの payload 束縛」の行より
          `v` の値は `scrut` の値の第 `tag` 変位の payload である。
      BY D16, D9, <3>1
    <3>3. QED
      BY <3>2, CODE src/rc_ir/ownership.rs: origin_inner, 帰納法の仮定
  <2>3. CASE `variant` が `Some(_)` で `scrut.ty.is_box(type_env)` が真。この腕は `here()` を返すので
        `(u, σ) = (v, λ)` であり、言明は同語反復である。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>7. QED
  `origin_inner` の `match` は `vars.bindings.get(var)` の値で分岐し、その腕は `None` と
  `Binding` の 7 変位 (`Param`, `Move`, `Llvm`, `Producer`, `Field`, `Payload`, `Join`) を尽くす。
  <1>1 から <1>6 はその全部を扱う。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, CODE src/rc_ir/ownership.rs: Binding,
     CODE src/rc_ir/ownership.rs: origin_inner

### L18 (名前はオブジェクトを決める)

**言明** — 1 つの活性化の 1 つの実行路を固定する。inhabited な leaf `λ` について `id(v, λ) = (u, σ)` と
するとき、`(u, σ)` はスロット (D6) であり、`obj(v, λ) = obj(u, σ)` である。さらに `obj(u, σ)` の値は、
`u` が束縛された後のどの位置で読んでも同じである。

**証明**

<1>1. `obj(u, σ)` は `u` の値だけで決まり、`u` が束縛された後は変わらない。
  <2>1. D4 の leaf の列挙は、`is_box` が真の型と `is_array` が真の型で止まり、その内側へ降りない。よって
        leaf の道が通るのは unbox の構造体・タプル・union だけであり、`obj(u, σ)` は `u` の値そのものの
        中の位置が指すオブジェクトである。
    BY D4
  <2>2. `RcExpr` の 6 変位のどれも、すでに束縛された変数に別の値を結び直さない。`Let`、`Destructure`、
        `Match` のアームの payload だけが変数を束縛し、A6 より束縛名は相異なる。
    BY CODE src/rc_ir/ast.rs: RcExpr, D2, A6
  <2>3. QED
    BY <2>1, <2>2
<1>2. `id(v, λ)` は、`(v, λ)` から D17 の別名の辺を 0 本以上たどって着く対である。
  `origin(v, λ)` の再帰についての帰納法で示す (P2 より再帰の木は有限である)。腕の分け方は L17 の
  <1>1 から <1>6 と同じである。
  <2>1. CASE 腕が `Origin::Exactly((v, λ))` を返す。すなわち L17 の <1>1、<1>4 の <2>3、<1>5 の <2>1、
        <1>6 の <2>3 の場合である。このとき `id(v, λ) = (v, λ)` であり、辺を 0 本たどったものである。
    BY L17
  <2>2. CASE 腕が D17 の別名の辺 1 本の先の `origin` の値をそのまま返す。すなわち L17 の <1>2
        (`Move`)、<1>4 の <2>2 (`Llvm` の単一 `Arg`)、<1>5 の <2>2 (unbox 容器の `Field`)、<1>6 の
        <2>1 と <2>2 (`Payload`) の場合である。D17 はこの 5 種の辺を数え上げており、`id` はその先の
        `origin` の `identity` なので、帰納法の仮定が使える。
    BY L17, D17, 帰納法の仮定
  <2>3. CASE 腕が `Binding::Join(arm_results)` のものである (L17 の <1>3)。
    <3>1. CASE 返り値が `Origin::Join` である。`of_candidates` は `identity` に第 2 引数 `(v, λ)` を
          据えるので、辺を 0 本たどったものである。
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates
    <3>2. CASE 返り値が `Origin::Exactly((u, σ))` である。L17 の <1>3 の <2>2 より、この実行路が選んだ
          アームの結果変数 `x_i` について `origin(x_i, λ) = Origin::Exactly((u, σ))` である。D17 は
          `Binding::Join` の辺を `λ` を変えない辺として数え上げているので、`(v, λ)` から `(x_i, λ)` へ
          辺 1 本で行ける。帰納法の仮定を `(x_i, λ)` に適用する。
      BY L17, D17, 帰納法の仮定
    <3>3. QED
      `Origin` の変位は `Exactly` と `Join` の 2 つである。
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin
  <2>4. QED
    L17 の <1>7 より、<2>1、<2>2、<2>3 で `origin_inner` の腕は尽きている。
    BY <2>1, <2>2, <2>3, L17
<1>3. QED
  <1>2 より `(u, σ)` は `(v, λ)` から D17 の別名の辺を 0 本以上たどって着く対である。L17 よりその leaf は
  inhabited なので `(u, σ)` はスロットであり (D6)、P3 と P4 よりその道の各辺で結ばれた 2 つのスロットは
  同じ参照を持つので、指すオブジェクトも等しい。位置によらないことは <1>1 が与える。
  BY <1>1, <1>2, L17, D6, D17, P3, P4

### L19 (`consume_unit` が呼ばれる位置)

**言明** — `cancel_body` の走査が `consume_unit(pending, K)` を呼ぶのは次の 4 か所だけであり、いずれの
呼び出しも、鍵 `K` があればその鍵と `Vec` を丸ごと取り除き、取り除いた `Vec` の各要素の `node` を
`self.needed_retains` に入れる。

1. 右辺が `Match` でない `Let(x, rhs, k)` の訪問: `rhs_consumes` が報告する各 `(w, μ)` について、
   `acted_unit_keys(w, μ)` の各要素。
2. `Destructure(c, fs, s, k)` の訪問: `destructure_consumes(c, fs, type_env)` が返す各 `μ` について、
   `acted_unit_keys(c.name, μ)` の各要素。
3. `Release(v, π, s, k)` の訪問: `others(r)` の各要素。
4. `Release(v, π, s, k)` の訪問で `un_bump` が `OutsideBracket` を返したとき: `key(r)`。

**証明**

<1>1. `consume_unit(pending, key)` の本体は
      `if let Some(stack) = pending.remove(&key) { for retain in stack { self.needed_retains.insert(retain.node); } }`
      である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_unit
<1>2. `consume_unit` を呼ぶのは、`CancelAnalysis::consume` と、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の 2 か所である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled
<1>3. `consume(pending, var, path)` は `self.acted_unit_keys(var, path)` の各要素について
      `consume_unit` を呼ぶ。`consume` を呼ぶのは、`consume_rhs` と、`walk_inner` の
      `RcExpr::Destructure(container, fields, _state, k)` の腕の 2 か所である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕
<1>4. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` が `consumed` に積んだ各 `(var, leaf)` に
      ついて `consume(pending, &var, &leaf)` を呼ぶ。`consume_rhs` を呼ぶのは `walk_inner` の
      `RcExpr::Let(x, rhs, k)` の腕 1 か所だけであり、その腕には右辺が `Match` の `Let` は入らない
      (その腕が `match` の先に置かれているため)。よって 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕, <1>3
<1>5. `walk_inner` の `RcExpr::Destructure(container, fields, _state, k)` の腕は、
      `destructure_consumes(container, fields, self.type_env)` の各 `leaf` について
      `self.consume(&mut pending, &container.name, &leaf)` を呼ぶ。よって <1>3 と合わせて 2 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕, <1>3
<1>6. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、`self.acted_unit_keys(&v.name, path)` の
      各 `other` について `other != key` のときだけ `self.consume_unit(&mut pending, other)` を呼び、
      その後 `un_bump` の返り値が `UnBump::OutsideBracket` のときだけ
      `self.consume_unit(&mut pending, key)` を呼ぶ。よって 3 と 4 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     DEF 節点の量
<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### L20 (単位の鍵は、その下の leaf の鍵である) --- 未証明

**言明** — `π` が `ty(v)` の RC unit (D5) であり、`λ ∈ L(v, π)` が inhabited であるとき、
`unit_of(id(v, λ)) = key(v, π)` である。

**この補題は証明できていない。** 第 11 節の 差し戻し 3 が、証明が閉じない場合と、その場合が現に起こりうるか
どうかについて分かったことを述べる。P21 と P23 はこの補題を仮定として引用する。

### L21 (1 つのオブジェクト、1 つの鍵) --- 未証明

**言明** — 1 つの活性化の 1 つの実行路の 1 つの位置において、同じオブジェクトを指す 2 つのスロット
`(v, λ)`、`(w, μ)` について、`unit_of(id(v, λ)) = unit_of(id(w, μ))` である。

**この補題も証明できていない。** P5 (a) は、2 つのスロットが**同じ参照**を持ち、かつ両者を結ぶ別名の道が
`Match` のアーム本体の `Ret` の辺を含まないときにだけ、`unit_key` の一致を与える。同じオブジェクトを指す
相異なる 2 つの参照については何も言わない。第 11 節の 差し戻し 4 が詳しい。P21 と P23 はこの補題を仮定として
引用する。

## 4. P19 (削除される retain の性質)

**言明 (README)** --- `cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含むすべての実行路に
おいて、`t` の `unit_key` を `acted_unit_keys` に含む消費より前、かつ終端の `Ret` より前に、削除される
`Release` 群が `t` の `outstanding` を空にする。さらに、`t` とともに削除される各 `Release` は、実行路の上で
`t` より後ろにある。

**証明する形**。「消費より前」は「`t` より後にある消費より前」と読む。`p` の上で `t` より前にある消費に
ついては何も主張しない。第 11 節の 差し戻し 1 がこの限定が要る理由を述べる。すなわち示すのは次である。

`t ∈ Del` が `Retain` 節点であるとき、`t` を含むすべての実行路 `p` について、

1. `p` の上に `t` より後の `Release` 節点の有限集合 `R_p(t) ⊆ Del` があり、
   `Σ_{r ∈ R_p(t)} un_bumped(r) = bumped(t)` である。
2. `R_p(t)` の最後の要素は、`p` の上で、`t` より後にある消費点のうち消費される leaf `(w, μ)` が
   `key(t) ∈ acted_unit_keys(w, μ)` を満たすもののどれよりも前にあり、`p` の終端の `Ret` よりも前にある。
3. `r ∈ un_bump_releases[t]` である各 `Release` 節点について、`r` を含むすべての実行路の上で `t` は `r` より
   真に前にある。

**証明**

<1>1. `t ∈ Del` が `Retain` 節点であることと `t ∈ CT` であることは同値である。
  BY L11
<1>2. 1 が成り立つ。L16 の 3 の `R_p(t)` を取ればよい。L16 の 1 よりその要素は `un_bump_releases[t]` の
      要素であり、`t ∈ CT` なので L11 の 3 よりそれらは `Del` の要素である。L15 より各要素は `p` の上で
      `t` より後にある。
  BY L16, L15, L11, <1>1
<1>3. 2 の前半が成り立つ。`p` の上で `t` より後にある消費点 `c` で、消費される leaf `(w, μ)` が
      `key(t) ∈ acted_unit_keys(w, μ)` を満たすものを取る。
  <2>1. CASE `c` が D9 の消費の表の `App`、`Closure`、`Llvm` の行の位置である。この位置は右辺が `Match`
        でない `Let` 節点であり、L19 の 1 より、その訪問は `rhs_consumes` が報告する各 leaf について
        `acted_unit_keys` の各要素を `consume_unit` に渡す。`collect_consumes_go` は右辺が `Var`、`App`、
        `Closure`、`Llvm` のとき `rhs_consumes` にそのまま委ね、その位置でほかに `out` へ積まないので、
        P7 の前半より `rhs_consumes` はこの 3 つの右辺について D9 の消費の表のとおりの leaf を報告する。
        よってこの訪問は `consume_unit(・, key(t))` を呼ぶ。
    BY L19, P7, D9, CODE src/rc_ir/ownership.rs: collect_consumes_go
  <2>2. CASE `c` が D9 の消費の表の `Destructure` の 2 行の位置である。L19 の 2 より、その訪問は
        `destructure_consumes` が返す各 leaf について `acted_unit_keys` の各要素を `consume_unit` に
        渡す。`collect_consumes_go` は `Destructure` の腕で `destructure_consumes` の各 leaf を `out` へ
        積み、その位置でほかに積まないので、P7 の前半より `destructure_consumes` はこの 2 行のとおりの
        leaf を返す。よってこの訪問は `consume_unit(・, key(t))` を呼ぶ。
    BY L19, P7, D9, CODE src/rc_ir/ownership.rs: collect_consumes_go
  <2>3. CASE `c` が D9 の消費の表の「関数本体の終端の `Ret(x)`」の行の位置である。これは `p` の終端の
        `Ret` であり、2 の後半が扱う。
    BY D9
  <2>4. QED
    <2>1 と <2>2 の位置は、L16 の 4 より、`t` が `pending` の要素である区間には入らない。区間の最初の
    節点は `p` の上で `t` の直後にあるので (L16)、それらの位置は区間の最後の節点 `n*(p)` より後にある。
    L16 の 1 と 3 より `n*(p)` は `R_p(t)` の最後の要素である。D9 の消費の表の 6 行を <2>1、<2>2、
    <2>3 が尽くす。
    BY <2>1, <2>2, <2>3, L16, D9
<1>4. 2 の後半が成り立つ。
  BY L16
<1>5. 3 が成り立つ。
  BY L15
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 5. P20 (削除は収支を保つ)

**言明 (README)** --- 各実行路において、削除される `Retain` が実行時に作る参照の多重集合は、その路で
実行される削除される `Release` が実行時に処分する参照の多重集合に一致する。

**証明** 実行路 `p` と 1 つの活性化を固定する。

<1>1. `p` の上にある `Del` の `Release` 節点の全体は、`p` の上にある `Del` の `Retain` 節点 `t` について
      `R_p(t)` を集めた非交和である。
  <2>1. L11 の 3 より、`Del` の各 `Release` 節点はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に
        属する。
    BY L11
  <2>2. `r ∈ un_bump_releases[t]` が `p` の上にあるならば、L15 より `t` も `p` の上にあり、`r` は `p` の
        上で `t` より後にある。さらに L15 より `t` は `pending(r)` の要素なので、`r` は L16 の区間に
        入り、L11 の 4 と L16 の 3 より `r ∈ R_p(t)` である。
    BY L15, L16, L11
  <2>3. QED
    BY <2>1, <2>2, L16
<1>2. 静的な収支。`p` の上にある `Del` の `Retain` 節点 `t` について
      `Σ_{r ∈ R_p(t)} un_bumped(r) = bumped(t)` である。よって <1>1 より、`p` の上にある `Del` の
      `Retain` 節点の `bumped` の総和は、`p` の上にある `Del` の `Release` 節点の `un_bumped` の総和に
      等しい。これは `VarPath` を鍵とする多重集合としての等式である。
  BY P19, <1>1
<1>3. 静的な数え上げから実行時の数え上げへ。`VarPath` の値 `n = (u, σ)` について、`u` がこの活性化の
      この実行路の上で束縛されており、かつ `σ` が `u` において inhabited であるとき `χ(n) := 1`、
      そうでないとき `χ(n) := 0` と置く。このとき、`p` の上のどの `Retain`/`Release` 節点 `m` (作用する
      対を `(v, π)` とする) についても、`m` が実行時に作る (あるいは処分する) 参照の多重集合は、
      `acted_references(v, π)` の各鍵 `n` の個数に `χ(n)` を掛けた多重集合と、P6 の DEF 名前づけ の下で
      一致する。
  <2>1. P6 (a) より、`acted_references(v, π)` は `L(v, π)` の各元 `λ` を `id(v, λ)` で名付けて数えた
        多重集合である。
    BY P6
  <2>2. P6 (b) より、`m` が実行時に作る (処分する) 参照の多重集合は、この数え上げを `Linh(v, π)` に
        制限したものに、DEF 名前づけ の下で一致する。
    BY P6
  <2>3. `λ ∈ L(v, π)` について、`λ ∈ Linh(v, π)` であることと `χ(id(v, λ)) = 1` であることは同値で
        ある。
    BY L17
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>4. 名前から参照へ。<1>3 の多重集合の鍵 `n` は、L18 よりこの活性化のこの実行路の上でただ 1 つの
      オブジェクト `obj(n)` を指し、その対応は位置によらない。D8 より同じオブジェクトへの参照は互いに
      区別されないので、参照の多重集合はオブジェクトごとの個数で決まる。
  BY L18, D8
<1>5. QED
  <1>2 の等式の両辺の各鍵 `n` の個数に `χ(n)` を掛けると、<1>3 より、左辺は `p` の上の `Del` の `Retain`
  が実行時に作る参照の多重集合の名前づけ、右辺は `p` の上の `Del` の `Release` が実行時に処分する参照の
  多重集合の名前づけになる。<1>4 よりこの 2 つの名前づけの一致は参照の多重集合の一致である。
  BY <1>1, <1>2, <1>3, <1>4

## 6. P21 (削除は解放後の読みを作らない)

**言明 (README)** --- 削除の前後で、各読む構文の各位置において解放されているオブジェクトの集合は変わらない。

**証明する形**。1 つの活性化と `B` の実行路 `p`、および対応する `B'` の実行路 `p'` (DEF 路の対応) を、活性化の
入力を同じにし、各 `Match` で同じアームを選ぶという条件の下で比べる。この条件は D11 が静的な実行路について
述べる述語であることに対応する。D3 の実行路はアームの選択の列であり、D11 はそのすべてについての主張なので、
出力の各実行路をこの対応で入力の実行路と突き合わせれば足りる。2 つの実行が実際に同じアームを選ぶかどうかは
D18 の観測値が絡む別の主張であり、P26 が扱う。

`B` の実行における位置 `q` でのオブジェクト `o` の参照カウントを `H(q, o)`、`B'` の実行における対応する位置
での参照カウントを `H'(q, o)` と書く。D7 より、`o` が解放されているとはその参照カウントが 0 であることなので、
示すのは次である。`p` のすべての位置 `q` とすべてのオブジェクト `o` について、`H(q, o) = 0` と
`H'(q, o) = 0` は同値である。

### DEF pending 総量

`p` の上の位置 `q` とオブジェクト `o` について、`S(q, o)` を次で定める。`q` の訪問の入口状態 `pending` に
在る各 `Retain` 節点 `t''` について、その `outstanding` が名指す `o` への参照の個数 (`outstanding` の各鍵の
うち L18 の意味で `o` を指すものの個数を、L17 の意味で inhabited な leaf に制限して数えたもの) を足し
上げたものである。

### L22 (打ち消される bump は余剰である) --- 未証明

**言明** — `p` のすべての位置 `q` とすべてのオブジェクト `o` について、次の 3 つが成り立つ。

(a) `Obl(q, o) ≥ d(q, o)` である。
(b) `d(q, o) ≥ 1` ならば `H(q, o) ≥ d(q, o) + 1` である。
(c) 位置 `q` にある削除されない構文が `Obl` から `o` への参照を `c` 個取り除くならば、
    `Obl(q, o) ≥ d(q, o) + c` である。

**この補題は証明できていない。** 直後の「L22 について分かっていること」が、証明のどこまでが閉じ、何が
足りないかを述べる。P21 と P23 はこれを仮定として引用する。

### L22 について分かっていること

**閉じる部分**。`d(q, o)` は、`q` において `pending` に在る `CT` の `Retain` 節点の `outstanding` が名指す
`o` への参照の総数に等しい。これは L16 の 3 と L17 から出る。`t ∈ CT` の実行時の bump は `bumped(t)` を
inhabited な leaf に制限したもの (P6) であり、その後に実行される削除された `Release` の実行時の処分は
`un_bumped(r)` を同じ制限で読んだものであって、`outstanding` はまさにその差だからである。したがって
`d(q, o) ≤ S(q, o)` であり、`d(q, o) ≥ 0` である。

**足りない部分**。上より、L22 は `S` についての次の 2 つに帰着する。

- (S1) `S(q, o) ≥ 1` ならば `H(q, o) ≥ S(q, o) + 1`。
- (S2) `S(q, o) ≥ 1` ならば `Obl(q, o) ≥ S(q, o) + 1`、かつ削除されない構文が `Obl` から `o` を `c` 個
  取り除くならば `Obl(q, o) ≥ S(q, o) + c`。

(S1)(S2) は `p` の上の位置についての帰納法で示すことになる。`H` を下げる構文は `Release` だけである
(D10 の表: 生成は `H` を上げ、消費は `H` を変えず、移動は変えない)。よって帰結を壊しうる遷移は 2 つ、
すなわち `Retain` 節点で `S` が増える遷移と、削除されない `Release` 節点で `H` が下がる遷移である。
後者について、L19、L20、L21、P5 (c) は次のところまでを与える。

削除されない `Release` 節点 `r` が位置 `q` で `o` への参照を処分し、かつ `d(q, o) ≥ 1` とする。`o` を
outstanding に持つ pending な `CT` の `Retain` 節点を `t` とする。

1. `r` が処分する `o` への参照は、`r` の inhabited な leaf `λ'` が持つものであり、その名前 `id(v_r, λ')` に
   ついて P5 (c) より `unit_of(id(v_r, λ')) ∈ acted_unit_keys(v_r, π_r)` である。L20 を `r` に適用すると
   `unit_of(id(v_r, λ')) = key(r)` である。
2. 同じオブジェクト `o` は `t` の側でも leaf `λ''` として名指されており、L21 より
   `unit_of(id(v_r, λ')) = unit_of(id(v_t, λ''))`、L20 を `t` に適用して
   `unit_of(id(v_t, λ'')) = key(t)` である。よって `key(r) = key(t)` である。
3. したがって `r` の訪問の `un_bump(&mut pending, &key(t), &un_bumped(r))` は、`t` を含むスタックに
   作用する。P16 の (a) より `t` は鍵 `key(t)` の下に在るので、P17 の `NoBracket` は起こらない。
   `OutsideBracket` なら L19 の 4 より `consume_unit(・, key(t))` が呼ばれて `t` が `needed_retains` に
   入り、L8 と DEF 削除集合 より `t ∈ CT` に反する。よって `InBracket(t')` である。`t' = t` なら
   L11 の 4 より `r ∈ un_bump_releases[t] ⊆ Del` となり、`r` が削除されないことに反する。よって
   `t'` は `t` より内側の (すなわち `t` より後に訪問された) pending な `Retain` 節点であり、`t'` は
   `CT` に入らない。P17 より `un_bumped(r)` は `t'` の `outstanding` に覆われる。

ここまでで、削除されない `Release` の処分は必ず「`t` より内側の、打ち消されない `Retain` の bump」の
範囲に収まることが言える。残るのは、この事実から (S1) の不等式を保つ段である。それには、`S(q, o)` を
超える参照が `q` において少なくとも 1 つあること --- すなわち「pending な `Retain` の bump は、その値
自身が持つ参照の**上に**積まれている」--- を不変条件として運ぶ必要がある。この不変条件は README の
D1 - D19、A1 - A14、P1 - P18 のどれにも無い。D8 が同じオブジェクトへの参照を互いに区別しないので、
「値自身が持つ参照」を個別に追う言い方も使えない。

(S2) について。`Obl` から参照を取り除く構文は `Release` と消費の 2 つである。消費に
ついては、L19 の 1 と 2 がその位置で `acted_unit_keys` の各要素が `consume_unit` に渡されることを、
P7 の前半が `rhs_consumes` と `destructure_consumes` が D9 の消費の表のとおりの leaf を報告することを、
P5 (c) と L20 と L21 が消費される leaf の鍵が `key(t)` であることを与える。よって消費は
`consume_unit(・, key(t))` を呼び、`t` は `needed_retains` に入って `t ∈ CT` に反する。すなわち消費は
`d(q, o) ≥ 1` の区間には現れない。残るのは `Release` であり、そこは (S1) と同じ形の残りを持つ。

**まとめ**。L22 を閉じるには次の 3 つが要る。L20、L21、および「pending な bump は値自身の参照の上に
積まれている」という実行時の不変条件。第 11 節の 差し戻し 3、4、5 がそれぞれを述べる。

### P21 の証明 (L22 を仮定して)

<1>1. `p` の上の各位置 `q` と各オブジェクト `o` について `H'(q, o) = H(q, o) - d(q, o)` であり、かつ
      `H(q, o) = 0` と `H'(q, o) = 0` は同値である。
  `p` の上の位置についての帰納法で示す。D3 より `p` は有限の列なので整礎である。
  <2>1. 帰納法の仮定: `q` より前の `p` の上の各位置について、この主張が成り立つ。
    BY 帰納法の仮定
  <2>2. 活性化の入口では `d = 0` であり、2 つの実行は同じ入力を受け取るので `H' = H` である。
    BY DEF 欠損
  <2>3. 2 つの実行は、`q` までに、`Del` の節点を除いて同じ構文を同じ値に対して実行する。
    <3>1. P22 より `B'` は `B` から `Del` の節点を抜いた木であり、他の節点の種類・変数・path・並びは
          変わらない。DEF 路の対応 より `p'` は `p` から `Del` の節点を除いた列である。
      BY P22, DEF 路の対応
    <3>2. `Del` の節点は `Retain` と `Release` だけである (L11)。D7 より `Retain` と `Release` は読む構文
          ではなく、参照カウントと状態バイトしか触らない。D9 と D10 より、この 2 つは値を作らず、移さず、
          変えない。よって 2 つの実行の各変数の値は等しい。
      BY L11, D7, D9, D10
    <3>3. QED
      BY <3>1, <3>2
  <2>4. CASE `q` の節点が `Del` の `Retain` 節点である。この節点は `B'` の実行では実行されない。`B` の
        実行では `o` への参照を `m` 個作り (D10)、`H` を `m` 増やす。DEF 欠損 より `d` も `m` 増える。
        よって <2>1 の第 1 の等式が保たれる。第 2 の同値は、`d ≥ 1` になる位置では L22 (b) が
        `H ≥ d + 1 > d` を与えるので `H' = H - d ≥ 1` であり、`H ≥ m ≥ 1` でもあることから従う。
    BY <2>1, D10, DEF 欠損, L22
  <2>5. CASE `q` の節点が `Del` の `Release` 節点である。この節点は `B'` の実行では実行されない。`B` の
        実行では `o` への参照を `c` 個処分し (D10)、`H` を `c` 減らす。DEF 欠損 より `d` も `c` 減る。
        よって <2>1 の第 1 の等式が保たれる。第 2 の同値は、`d` が減った後も L22 (b) が成り立つことから
        従う。
    BY <2>1, D10, DEF 欠損, L22
  <2>6. CASE `q` の節点が `Del` に入らない。この節点は 2 つの実行で同じ構文を同じ値に対して実行するので
        (<2>3)、`H` と `H'` を同じだけ変える。`d` は変わらない (DEF 欠損)。よって第 1 の等式が保たれる。
        第 2 の同値は、`d ≥ 1` の位置では L22 (b) より `H - d ≥ 1` であり、`d = 0` の位置では
        `H' = H` であることから従う。
    BY <2>1, <2>3, D10, DEF 欠損, L22
  <2>7. 解放が誘発する走査も 2 つの実行で一致する。<2>4 から <2>6 の各段の後で `H = 0` と `H' = 0` が
        同値なので、どちらの実行でも同じオブジェクトが同じ位置で解放される。D7 より解放されたオブジェクト
        の走査はそのオブジェクトが解放されるときに起き、その走査が出す `Release` も同じである。
    BY <2>4, <2>5, <2>6, D7
  <2>8. QED
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかであり (L11)、
    <2>4、<2>5、<2>6 がこれを尽くす。
    BY <2>2, <2>4, <2>5, <2>6, <2>7, L11
<1>2. QED
  D7 より、各読む構文の各位置において解放されているオブジェクトの集合は、その位置で `H` が 0 である
  オブジェクトの集合である。<1>1 よりこの集合は 2 つの実行で等しい。
  BY <1>1, D7

## 7. P22 (`drop_nodes` の正しさ)

**言明 (README)** --- `drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release` 節点だけを
取り除いた木を返し、他の節点の種類・変数・path・並びを変えない。

**証明**

<1>1. `drop_nodes(node, to_delete)` は `grow_stack(|| drop_nodes_inner(node, to_delete))` であり、L1 より
      `drop_nodes_inner` をちょうど 1 回呼んでその値を返す。
  BY CODE src/rc_ir/borrow.rs: drop_nodes, L1
<1>2. `drop_nodes` が読む `NodeId` は入力の木のものである。`cancel_body` は
      `drop_nodes(body, &analysis.cancelled())` を呼び、その `body` は走査が訪問した木そのものである。
      `cancel` の閉包の実行中、`body` は `prog` から借用されているので、その木の割り当てはすべて生きて
      いる。よって `node_id` の値は走査のときと同じである。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: node_id
<1>3. 木 `N(node)` の構造についての帰納法で、`drop_nodes_inner(node, to_delete)` が言明のとおりの木を返す
      ことを示す。子は真の部分木なので整礎である。
  <2>1. CASE `node` の式が `RcExpr::Retain(v, path, state, k)` である。この腕は `drop_nodes(k, to_delete)`
        を 1 回呼び、`to_delete` が `node_id(node)` を含むときはその値をそのまま返し、含まないときは
        `RcExpr::Retain(v.clone(), path.clone(), *state, k)` の節点を積んで返す。すなわち `v`、`path`、
        `state`、`node.source` は変わらない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Retain(v, path, state, k)` の腕
  <2>2. CASE `node` の式が `RcExpr::Release(v, path, state, k)` である。<2>1 と同じ形である。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Release(v, path, state, k)` の腕
  <2>3. CASE `node` の式が `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` である。この腕は各アームに
        ついて `drop_nodes(&arm.body, to_delete)` を 1 回、`drop_nodes(k, to_delete)` を 1 回呼び、
        `x`、`scrut`、各アームの `tag`/`payload`/`payload_state` を変えずに節点を積む。`to_delete` の
        検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body
  <2>4. CASE `node` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
        `drop_nodes(k, to_delete)` を 1 回呼び、`x` と `rhs` を変えずに節点を積む。`to_delete` の検査は
        しない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, rhs, k)` の腕
  <2>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)` または `RcExpr::Eval(v, k)`
        である。この 2 つの腕は `drop_nodes(k, to_delete)` を 1 回呼び、他のフィールドを変えずに節点を
        積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Eval(v, k)` の腕
  <2>6. CASE `node` の式が `RcExpr::Ret(v)` である。この腕は `v` を変えずに 1 節点を作って返す。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Ret(v)` の腕
  <2>7. QED
    `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <2>1 から <2>6 が尽くす。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
<1>4. `Del` の要素はすべて `Retain` 節点か `Release` 節点の `NodeId` なので、<1>3 の 7 つの腕のうち
      `to_delete` を検査する 2 つだけが節点を落とす。
  BY L11, <1>3
<1>5. `B` の実行路 (D3) と `B'` の実行路の間に、`Del` の節点を除く対応が全単射としてある。
  <2>1. <1>3 より `B'` の木は `B` の木から `Del` の `Retain`/`Release` 節点を抜いたものであり、`Match` の
        アームの本数と並び、および各節点の継続の順序は変わらない。
    BY <1>3, <1>4
  <2>2. D3 の実行路は、根から継続をたどり、各 `Match` でアームを 1 つ選ぶことで決まる。<2>1 より
        `B` と `B'` は同じ `Match` 節点の同じアームの集合を持つので、アームの選択の全体は 2 つの木で
        同じである。
    BY D3, <2>1
  <2>3. QED
    BY <2>1, <2>2
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 8. P23 (`cancel` は RC 規律を保存する)

**言明 (README)** --- D12 の意味で RC 規律を満たすプログラムを入力とすると、`cancel` の出力も D12 の意味で
RC 規律を満たす。

**証明** 入力プログラムを `P`、出力を `P'` と書く。

<1>1. `P'` の所有と借用の割り当て (D14) は `P` のものと同じである。
  <2>1. `cancel` は `prog.funcs.values()` の各 `f` について `let mut clone = f.clone();` を作り、
        `clone.body` にだけ書き込んで `(f.name.clone(), clone)` を `funcs` に入れる。よって
        `borrowed_units`、`params`、`capture` は変わらない。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>2. D14 の割り当ては `RcFunc::borrowed_units` が定めるので、<2>1 より変わらない。
    BY D14, <2>1
  <2>3. QED
    BY <2>1, <2>2
<1>2. `P'` の各関数の本体と各グローバル初期化子の `init` は、`P` の対応するものに `cancel_body` を適用した
      ものであり、DEF 出力の本体 の `B'` である。
  BY CODE src/rc_ir/borrow.rs: cancel, DEF 出力の本体
<1>3. `B'` の各実行路 `p'` について、対応する `B` の実行路 `p` (DEF 路の対応) を取る。`p` について D11 の
      (S-a)、(S-b)、(S-c) が成り立つ (仮定より `P` は D12 を満たす)。
  BY D12, P22, DEF 路の対応
<1>4. `p'` の各位置において `Obl'(q) = Obl(q) - d(q)` である。
  <2>1. L22 (a) より `Obl(q, o) ≥ d(q, o)` なので、この差は多重集合として定義できる。
    BY L22
  <2>2. `p` と `p'` は `Del` の節点を除いて同じ構文を同じ値に対して実行し (P21 の <1>1 の <2>3)、
        `Del` の節点だけが `Obl` への作用の差を生む。D10 より `Retain` は `Obl` に加え、`Release` は
        取り除くので、その差は DEF 欠損 の `d(q)` である。
    BY D10, DEF 欠損, P21
  <2>3. QED
    BY <2>1, <2>2
<1>5. (S-a) が `p'` で成り立つ。`p'` の上で `Obl'` から参照を取り除く操作は、`p` の上の同じ操作から
      `Del` の `Release` を除いたものである。そのそれぞれについて、L22 (c) より
      `Obl(q, o) ≥ d(q, o) + c` すなわち `Obl'(q, o) ≥ c` である。
  BY <1>4, L22, D10
<1>6. (S-b) が `p'` で成り立つ。`p` の終端の `Ret` の消費の後、`Obl` は空である (<1>3 の (S-b))。P19 の 2
      より、`p` の上の `Del` の各 `Retain` はその `Ret` より前に完全に un-bump されているので、その位置で
      `d = 0` である。よって <1>4 より `Obl'` も空である。
  BY <1>3, <1>4, P19, P20
<1>7. (S-c) が `p'` で成り立つ。P21 より、各読む構文の各位置で解放されているオブジェクトの集合は `p` と
      `p'` で等しい。`p` ではその集合がその位置の読む構文が読みうるオブジェクトを含まない (<1>3 の
      (S-c))。P22 より `p'` の読む構文とその位置の値は `p` のものと同じである。
  BY <1>3, P21, P22
<1>8. QED
  <1>2 より `P'` のすべての本体は `B'` の形であり、<1>5、<1>6、<1>7 よりそのすべての実行路で D11 の 3 つが
  成り立つ。<1>1 より読む割り当ても同じである。D12 はこれを言う。
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
  <2>1. 元の版 `f_own` は `func.clone()` に `body` を書き込んだものであり、その後 `borrowed_units` だけが
        書き換えられる。よってこの 4 つは変わらない。
    BY CODE src/rc_ir/borrow.rs: borrow_ify
  <2>2. 借用版は `clone_func(func, ・, ・)` が作る `RcFunc` に `body` を書き込んだものであり、
        `clone_func` は `fn_ty: func.fn_ty.clone()`、`ret_ty: func.ret_ty.clone()`、
        `inline_into_callers: func.inline_into_callers` を据える。`params` は
        `fresh_rename_function` が返すもので、その各要素は `rename_var(p, &renaming)` すなわち `p` の
        複製の `name` だけを差し替えたものなので、型は変わらない。
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

## 10. 主定理 T

**言明 (README)** --- `split_rc_units` の出力が A1 と A2 を満たすとき、`cancel(borrow_ify(・))` の出力に
ついて次が成り立つ。

1. 出力は D12 の意味で RC 規律を満たす (P14 と P23 の合成)。
2. よって出力のどの実行でも、解放されたオブジェクトの読みは起きず、どのオブジェクトもちょうど 1 回
   解放される (1 と P27)。
3. 入力で一意だった観測点は、出力でも一意である (P26)。
4. D12 が見ない部分は P24 のとおりに保たれる。

**証明**

<1>1. パスの順序。`optimize_rc_program` は `split_rc_units(&mut prog, type_env)` を呼び、その後
      `config.enable_borrow_optimization()` が真のときにだけ `prog = borrow_ify(&prog, type_env);` と
      `prog = cancel(&prog, type_env, config.develop_mode);` をこの順で呼ぶ。この 2 つの間にあるのは
      `validate(&prog, "after borrow_ify")` だけであり、`validate` はプログラムを変えない。よって
      `cancel` の入力は `borrow_ify` の出力である。
  BY CODE src/build/build_object_files.rs: optimize_rc_program
<1>2. 1 は閉じない。P14 は「D12 の意味で RC 規律を満たし、かつ A1 と A2 を満たすプログラムを入力とすると、
      `borrow_ify` の出力は D12 の意味で RC 規律を満たす」と言うが、README の第 8 節が述べるとおり P14 は
      このコミットのコードでは偽である (#530)。したがって、P23 が要求する「D12 を満たす入力」が
      `borrow_ify` の出力について言えない。
  BY P14, P23, README の第 8 節
<1>3. 1 の残りの形。P14 が真になるようにコードが直った後は、A1 と A2 を満たす `split_rc_units` の出力に
      P14 を適用して `borrow_ify` の出力が D12 を満たすことを得、それに P23 を適用して `cancel` の出力が
      D12 を満たすことを得る。P23 は L20、L21、L22 に懸かっているので、そこも同時に要る。
  BY <1>1, P14, P23, L20, L21, L22
<1>4. 2 は閉じない。P27 (実行の合成) は `p50-observation-and-runs.md` に置かれる予定の命題であり、
      未証明である。P27 は「プログラムのすべての本体が D11 を満たすならば、そのプログラムのどの実行に
      おいても、解放されたオブジェクトの読みは起きず、どのオブジェクトもちょうど 1 回解放される」と
      言う。1 が閉じた上でこれを引用すれば 2 が出る。
  BY P27, <1>2, <1>3
<1>5. 3 は閉じない。P26 (一意性は悪化しない) も `p50-observation-and-runs.md` に置かれる予定の命題で
      あり、未証明である。README の P26 の注記が述べるとおり、`borrow_ify` の `call_rc` が置く `Retain`
      について P26 が真かどうかは自明ではない。
  BY P26
<1>6. 4 が成り立つ。
  BY P24
<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

## 11. 開いている段と README へ差し戻す点

### 差し戻し 1 (P19 の言明に「`t` より後の」が要る)

P19 の言明は「`t` の `unit_key` を `acted_unit_keys` に含む消費より前 ... `outstanding` を空にする」と
述べるが、`t` より前にある消費についてはこれを満たしようがない。`t` が `pending` に入るのは `t` の訪問で
あり、un-bump はその後にしか起きないからである。実行路の上で `t` より前に、`key(t)` を
`acted_unit_keys` に含む消費がある本体は書ける。P19 の言明を

> `t` を含むすべての実行路において、**`t` より後にある**、`t` の `unit_key` を `acted_unit_keys` に
> 含む消費より前、かつ終端の `Ret` より前に、削除される `Release` 群が `t` の `outstanding` を空にする。

と直すのが素直である。この文書はこの形を証明した。

### 差し戻し 2 (A3 の単一 `Arg` の行を inhabited について同値と読んでいる)

L17 の <1>4 の <2>2 は、A3 の「単一の `Arg(j, σ)`」の行を両向きに使う。すなわち、結果の leaf が
inhabited なら第 `j` オペランドの leaf `σ` も inhabited であり、逆に `σ` が inhabited なら結果の leaf も
inhabited である、と読む。A3 の現在の文面「第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を
作らない」は、A5 の「inhabited でない leaf は参照を持たない」と合わせれば両向きを与えるが、読みが
1 つに定まる書き方ではない。A3 の表のこの行に

> 結果のその leaf が inhabited であることと、第 `j` オペランドの leaf `σ` が inhabited であることは
> 同値である。

を足すのがよい。

### 差し戻し 3 (L20 --- 単位の鍵は、その下の leaf の鍵である)

**言明**。`π` が `ty(v)` の RC unit (D5) であり、`λ ∈ L(v, π)` が inhabited であるとき、
`unit_of(id(v, λ)) = key(v, π)` である。

これは `unit_key` の doc が述べていること (「A leaf below an unboxed union keys to the union itself, so a
whole-union retain and a consume of the payload get the same key」) の形式化であり、`cancel` の走査が
`Retain`/`Release` を unit の path で鍵付けしながら、消費を leaf の path で鍵付けしていることの整合性で
ある。README にはこの言明が無い。

**証明が閉じる場合**。`origin(v, π)` の再帰が `here()` で止まる場合 (`Param`、`Producer`、束縛の無い
名前、boxed 容器の `Field`、boxed union の `Payload`)、`id(v, π) = (v, π)` と `id(v, λ) = (v, λ)` で
あり、`truncate_to_unit(ty(v), λ) = π` が成り立つ (`π` は unit なので `truncate_to_unit` の走査は
`π` を消費し終えた所で `Unit` または `Capture` に当たって止まる)。よって両辺とも `(v, π)` になる。
`Move`、unbox 容器の `Field`、`Payload`、`Llvm` の単一 `Arg`、および `Join` の両辺が同じ形になる場合は、
帰納法で閉じる。

**閉じない場合**。次の 2 つである。

- `Binding::Join` の腕で、`origin(v, π)` が `Join` (アームの `acted_on` の合併が 2 元以上) になり、かつ
  `origin(v, λ)` が `Exactly((u, σ))` (アームがその leaf で一致) になる場合。このとき
  `unit_of(id(v, π)) = (v, π)` だが `unit_of(id(v, λ)) = unit_of((u, σ))` であり、`u` は `v` と別の
  変数でありうる。
- `Binding::Llvm` の腕で、`origin_from_leaves_under` が `π` について `Join` を返し (`reached` が
  2 元以上)、かつ `λ` の宣言が単一の `Arg` である場合。同じずれが起きる。

どちらの場合も、成立には「1 つの unbox union の 2 つ以上の変位の leaf が、相異なる origin を持つ」ことが
要る。このコミットの `result_prov` の実装でそれが起きる op を挙げることはできなかった (union を作る op は
作らない変位の leaf を空集合と宣言し、空集合の leaf は `origin_from_leaves_under` の数え上げに寄与しない)。
`Binding::Join` についても、アームがある leaf で一致しながら unit で食い違う形を作るには、やはり同じ形の
値が要る。すなわち**この 2 つの場合が現に起こるかどうかは決着していない**。

L20 を README に置くとすれば、P5 (a)(b)(c) と同じ層 1 の命題として、`origin` についての主張として置くのが
筋である。証明が上の 2 つの場合で閉じないなら、そこはコードの側 (`unit_key` が unit の path から鍵を作る
こと) を見直す点になる。

### 差し戻し 4 (L21 --- 1 つのオブジェクト、1 つの鍵)

**言明**。1 つの活性化の 1 つの実行路の 1 つの位置において、同じオブジェクトを指す 2 つのスロット
`(v, λ)`、`(w, μ)` について `unit_of(id(v, λ)) = unit_of(id(w, μ))` である。

P5 (a) はこれを、2 つのスロットが**同じ参照**を持ち、かつ両者を結ぶ別名の道が E6 (`Match` のアーム本体の
`Ret` の辺) を含まないときにだけ与える。`cancel` が要るのはそれより広い形である。位置 `q` で
`Release` が処分する参照と、pending な `Retain` が bump した参照は、同じオブジェクトを指すが同じ参照とは
限らないからである (retain がまさに 2 つ目の参照を作っている)。

P5 (a) の反例 R1 (`p12-identity-and-consumes.md`) が E6 の辺で `unit_key` が分かれることを示しているので、
L21 をそのままの強さで置くことはできない。置くなら、`cancel` が実際に必要とする形 --- 「pending な
`Retain` の `outstanding` に在るオブジェクトを処分する構文は、その `Retain` の鍵の下で `consume_unit` か
`un_bump` を呼ぶ」--- を、`origin` と `acted_unit_keys` の言葉で述べた命題として置くのがよい。

### 差し戻し 5 (L22 --- 打ち消される bump は余剰である)

第 6 節の「L22 について分かっていること」が述べたとおり、L22 は `S(q, o)` についての 2 つの不等式に帰着
する。その帰納法を回すには、「pending な `Retain` の bump は、その値自身が持つ参照の上に積まれている」と
いう実行時の不変条件が要る。D10 は `Obl` を、D7 は `H` を定めるが、`pending` (走査の側の量) と `H`/`Obl`
(実行時の量) を結ぶ言明は README に無い。この結び付けが、層 4 の実質である。

置くべき形は、たとえば次のような命題である。

> **(N4)** 走査の状態 `pending` が位置 `q` で保持する `Retain` 節点 `t''` と、その `outstanding` が
> 名指すオブジェクト `o` について、`q` において `H(q, o)` は、`pending` 全体の `outstanding` が名指す
> `o` の総数より真に大きい。

この命題は、L20 と L21 と、`Retain` が bump する前にその値が参照を持つこと (A5) から示せる見込みがある。
第 6 節の 1 から 3 がその半分 (削除されない `Release` の側) を与えている。

### 気づいたコードの欠陥

新しいコードの欠陥は見つからなかった。差し戻し 3 が述べる 2 つの場合は、成立すれば
`Retain(v, π)` と、その `π` の下の leaf を消費する構文とが別の鍵に分かれ、打ち消してはならない対が
打ち消されることになるが、それを起こす `result_prov` の宣言も束縛の形も、このコミットのコードには
見つけられなかった。#529 (`be26b396` で修正済み) と #530 (未修正) は、README の第 8 節が既に記録して
いる。

### P5 (c) を使った所

- 第 6 節の「L22 について分かっていること」の 1。削除されない `Release` が処分する参照の名前が、その
  `Release` の `acted_unit_keys` に (`unit_of` を通して) 現れることを言うのに使った。
- L21 の言明の役割の説明。P5 (c) は `acted_unit_keys` が inhabited な leaf 由来のオブジェクトを覆うことを
  言うが、覆う先の鍵が pending な `Retain` の鍵と一致することまでは言わない。その隙間が L20 と L21 で
  ある。
