# P18c, P19 - P24: `cancel` が RC 規律を保存すること

この文書は README の 7 命題 P18c, P19, P20, P21, P22, P23, P24 を証明する。README の定義 D1 - D32、
仮定 A1 - A20、および命題 P1 - P18b と P14a の**言明**の上に立つ。主定理 T は `p70-main-theorem.md` の
担当であり、この文書は扱わない。

対象コミットは `10bb7e118ce88fbed8b66d20a7b8d8fea2e82e92` である。README が名指す `6af3eb3b` との差の
うち、この文書が引用する記号に届くのは `borrow_ify` だけであり、そこでの変更は
`funcs_observing_uniqueness` の呼び出しの引数である。P24 が `borrow_ify` について読むのは
`RcProgram` の組み立てと `f_own` の作り方であり、どちらも動いていない。

引用する外部の補題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF bump の帰属` (`B_ρ`)、`DEF ρ-歩みと ρ-終端`、`DEF 別名類` (`obj(C)`)、
`DEF 類ごとの参照` (`held_ρ`、`bumps_ρ`) -- と、補題 `L7` (boxed leaf の路は反鎖をなす)、`L9`
(`identity` は inhabited を決める)、`L10a` (静的な数え上げと実行時の作用が活性な名前で一致する)、`L11`
(活性な名前では一致し、非活性な名前では `B` は空)、`L17` (`N` は別名類ごとの `bumps` の和である)。
これらは `p30 の L10`、`p13 の L17` のようにファイル名を添えて引用する。

この文書が導入する補題は `L30` から番号を付ける。`p30` と `p13` の補題の番号と衝突させないためである。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P18c | 証明済み (`L42`)。A19、P14a、`p13` の `L17` を読む |
| P19 | 証明済み |
| P20 | 証明済み |
| P21 | **G1 と G2 の下で証明した。** どちらも README の D29 と D21 に足すことを求める点である (第 11 節) |
| P22 | 証明済み |
| P23 | **G1 と G2 の下で証明した。** (S-a) は `L42` に、(S-b) と (S-c) は `L44` に載る |
| P24 | 証明済み |

**G1 と G2 は、どちらも対応する 2 つの活性化に何が与えられているかについての点である。** 第 2 節の
`DEF README へ差し戻す 2 点` が言明を、第 11 節が求める直しを述べる。ほかに開いている点は無い。

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

`cancel` の入力は `borrow_ify` の出力である。`optimize_rc_program` が `borrow_ify` の返り値を `cancel` に
渡す (`CODE src/build/build_object_files.rs: optimize_rc_program`)。よって本体 `B` について、A19 の
「`borrow_ify` がそれを写した各本体」の側と P14a が使える。

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

### DEF 実行時の量

活性化の**時点**とは、参照の作成・処分の事象の間の区切りをいう。事象は 3 種である。その活性化の節点に
ついて D10 の行が定めるもの、その活性化が作った子の活性化 (D24 の (E3)、(E7)、および (E2) のうち
オペランドを適用する `Llvm` の段) の事象、および D24 の (F) の解放が処分するものである。**子の活性化の
間の時点も、親の活性化の時点である。**

`B` の 1 回の活性化 `α` の各時点 `τ` とオブジェクト `O` について、`H(τ, O)` をその時点の `O` の参照
カウント (D7) とする。`α` 自身の節点についての時点については、`Obl(τ, O)` をその時点の義務集合 (D10) が
持つ `O` への参照の個数とする。`B'` の活性化 `α'` については `H'`、`Obl'` と書く。

`ρ` の上の**位置**とは、`ρ` の上の節点 `q` の実行の直前の時点をいい、その時点の量を `H(q, O)`、
`Obl(q, O)` と書く。これに、`ρ` の終端の `Ret` の消費を行った後の時点を 1 つ加え、これも位置と呼ぶ。
D3 より `ρ` は有限の列なので、位置は有限個である。`d` と `H` と `Obl` はこの最後の位置でも定まる。
1 つの節点の実行が複数の事象を起こすことがあるので、時点は位置より細かい。

対応する活性化 `α`、`α'` を固定したとき、`H'(q, O)`、`Obl'(q, O)` は、`α'` について `ρ'` の対応する
位置で読んだ値とする。`q ∈ Del` のときは `ρ'` にその節点が無いので、`ρ'` の上で `q` の直後にあたる位置の
値を取る。**`H` と `H'` を 1 つの `O` について並べて書くとき、2 つの活性化のオブジェクトを結ぶのは D29 の
全単射である。** `O` はその全単射が対応させるオブジェクトを渡る。

**節点の実行の扱い**。D10 は義務集合を「実行路上の各位置における」量として定めるので、この文書は 1 つの
節点について D10 の行が定める参照の作成と処分を 1 つの遷移としてまとめて適用する。その遷移でカウントが
0 になったオブジェクトについて D24 の (F) の解放が走り、解放が処分する参照についてこれを繰り返す。
A18 (a) より生きているオブジェクトのグラフは非巡回なので、この繰り返しは有限で終わる。すなわち 1 つの
節点の実行の時点は、その節点が作る子の活性化の事象、D10 の遷移、解放の走査の 3 群である。

**子の活性化の事象は、D10 の遷移より前に置く。** 子の活性化を作る段は D24 に 3 つある。(E3) の `App`
については順序が `H` を変えない -- (E3) が引数の参照を子へ渡し、(E4) が結果の参照を子から受け取るので、
`App` の節点自身の D10 の行はどちらも `H` を変えない (D24 の (E2) の表の `App` の行が「**変わらない**」と
述べる)。(E2) のうちオペランドを適用する `Llvm` の段については、D24 が「適用された関数の本体の活性化が
作られ、`a` はそれが終わるまで中断中である」と述べるので、その段が完了するのは子が終わった後である。
(E7) のグローバルの初期化は、その節点が値を読む前に走る。よって解放の走査は節点の実行の最後にあり、
その最初の時点から `q'` まで `H` は下がるだけである。

### DEF 解放されている

オブジェクト `O` が活性化のある時点 `τ` で**解放されている**とは、`τ` かそれより前の時点で `O` の参照
カウントが 0 であること、すなわち `H(τ0, O) = 0` を満たす `τ` 以前の時点 `τ0` が在ることをいう。D7 は
参照カウントが 0 になったオブジェクトが解放されると定めるので、これは D7 の「解放される」を時点の言葉で
書いたものである。`α'` についても `H'` で同じように定める。

### DEF 欠損

対応する活性化を固定する。`ρ` の上の位置 `q` と計数下 (D26) のオブジェクト `O` について、

`d(q, O) :=` (`q` より前に実行された `Del` の `Retain` 節点が `O` への参照を作った個数)
`-` (`q` より前に実行された `Del` の `Release` 節点が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。`Del` の
要素は `α` 自身の節点なので、`q` の節点の実行の間の時点では `d` は `d(q, O)` のままである。

**`d(q, O)` は README の P21 (a) の `k(O)` である。** P21 (a) の `k(O)` は「その位置までに `α` が実行した
削除済みの `Retain` のうち、対になる削除済みの `Release` をまだ実行していないものが `O` に作った参照の
個数」であり、L43 がこの 2 つが同じ量であることを示す。

### DEF N

対応する活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト `O` について、

`N(q, O) := Σ_{p ∈ pending(q)} Σ_{o} B_ρ(q, p)[o]`

と定める。内側の和は、`ρ` で活性であって `obj_ρ(o) = O` である名前 `o` を渡る。`B_ρ(q, p)` は D27 の
`B(p, ρ)` を節点 `q` の訪問の入口で読んだものであり、`p13` の `DEF bump の帰属` が同じ規則を表で書く。

**`N(q, O)` は README の P18a の `n(O)` であり、`p13` の `DEF N` の `N_ρ(q, O)` である。** P18a は
「走査中の位置」を D27 に従って節点の訪問の入口に取り、`n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と
置く。内側の和を活性な名前に制限してよいのは、`p13` の `L11` (ii) より活性でない名前の `B_ρ` が 0 だから
であり、`obj_ρ(o)` が定まるのも活性な名前についてだけである。

### DEF 消費点

D9 の消費の表の行が指す位置を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

### DEF README へ差し戻す 2 点

次の 2 つは README の D・A・P で名指せない言明である。**どちらも README の側の直しを求める点であり、
この文書はそれを第 11 節に述べる。**読む所は `L44` だけである。

- **G1 (対応する活性化は同じ参照カウントから始まる)**。D29 が `α'` に対応させる `B` の活性化 `α` は、
  活性化の最初の時点において、D29 の全単射で対応する各オブジェクトについて `α'` と等しい参照カウントを
  持つ。読む所: `L44` の <1>4 の <2>1。
- **G2 (子の活性化を作る段は、参照カウントに与える変化も活性化の側のデータである)**。D24 が子の活性化を
  作る段として挙げる 3 つ -- (E3) の `App`、(E2) のうちオペランドを適用する `Llvm` の段、(E7) の
  グローバルの初期化 -- のいずれについても、その段が参照カウントに与える変化は活性化の側のデータであり、
  D29 は対応する 2 つの活性化に同じものとして与える。D21 はこれを `App` の行について述べており、G2 は
  残る 2 つについて同じ扱いを求める。読む所: `L44` の <1>4 の <2>5 の <3>1 の <4>3。

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

### L41a (類の参照は bump 以上である)

**言明** --- 実行路 `ρ` を辿る活性化の各時点 `τ` と各計数下の別名類 `C` について
`held_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。

**証明**

<1>1. CASE `bumps_ρ(τ, C) = 0`。A19 の (ii-a) より `held_ρ(τ, C) ≥ 0` である。
  BY A19
  (ii-a) は「各時点と各計数下の別名類について、その類が持つ参照の個数は非負であり」と述べる。
<1>2. CASE `bumps_ρ(τ, C) ≥ 1`。A19 の (ii-b) より `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であり、これは
      `bumps_ρ(τ, C)` 以上である。
  BY A19
  (ii-b) は「`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である」と述べる。
<1>3. QED
  `bumps_ρ(τ, C)` は `B_ρ` の個数の総和であり、`p13` の `L11` より各個数は 0 以上なので、この 2 つの
  場合が尽くす。
  BY <1>1, <1>2, p13 の L11, p13 の DEF 類ごとの参照

### L41b (類ごとの余りは 1 つ立つ)

**言明** --- 実行路 `ρ` を辿る活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト `O` について、
`N(q, O) ≥ 1` ならば `Σ_{C : obj(C) = O} held_ρ(q, C) ≥ N(q, O) + 1` である。和は `obj(C) = O` である
計数下の別名類 `C` を渡る。

**証明**

<1>1. `N(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` である。
  BY p13 の L17
  `p13` の `L17` は、`ρ` の上の節点 `q` と計数下のオブジェクト `O` について
  `N_ρ(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` を述べる。DEF N より `N(q, O) = N_ρ(q, O)` である。
<1>2. PICK `C0` SUCH THAT `obj(C0) = O` かつ `bumps_ρ(q, C0) ≥ 1`。
  BY <1>1, 本補題の仮定, p13 の L11
  `p13` の `L11` より各 `bumps_ρ` は 0 以上なので、和が 1 以上ならば 1 以上の項が在る。
<1>3. `held_ρ(q, C0) ≥ 1 + bumps_ρ(q, C0)` である。
  BY A19, <1>2
  A19 の (ii-b) が `bumps ≥ 1` の時点についてこれを述べる。
<1>4. `C0` 以外の各 `C` について `held_ρ(q, C) ≥ bumps_ρ(q, C)` である。
  BY L41a
<1>5. QED
  BY <1>1, <1>3, <1>4
  <1>3 と <1>4 を `obj(C) = O` である類すべてについて足すと
  `Σ_C held_ρ(q, C) ≥ Σ_C bumps_ρ(q, C) + 1` であり、<1>1 よりその右辺は `N(q, O) + 1` である。
  `+1` が 1 回しか立たないのは、<1>3 を使うのが `C0` の 1 つだけだからである。

### L41c (中断中も余りは残る)

**言明** --- 実行路 `ρ` を辿る活性化 `α` を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト `O` に
ついて、`N(q, O) ≥ 1` とする。このとき、`q` の位置から `q` の節点の実行が終わるまでの各時点 `τ` のうち、
`α` が子の活性化を待って中断中である時点について、`H(τ, O) ≥ N(q, O) + 1` である。

**証明**

<1>1. `α` が中断中の間、`held_ρ(・, C)` と `bumps_ρ(・, C)` はどの類 `C` についても動かない。よって
      そのような時点 `τ` について `held_ρ(τ, C) = held_ρ(q, C)` である。
  BY A19
  A19 は「**「各時点」は、その活性化が生きている (D23) 間のすべての時点であり、入れ子の呼び出しで中断中の
  時点を含む。** … 中断中はその活性化の節点が走らないので `held` も `bumps` も動かず」と述べる。DEF 実行時の量
  より、子の活性化の事象は `q` の節点の D10 の遷移より前に置かれるので、`α` が中断中である時点までに
  `α` 自身の事象は 1 つも起きておらず、`held_ρ` は `q` の位置の値のままである。
<1>2. `Σ_{C : obj(C) = O} held_ρ(q, C) ≥ N(q, O) + 1` である。
  BY L41b, 本補題の仮定
<1>3. `H(τ, O) ≥ Σ_{C : obj(C) = O} held_ρ(τ, C)` である。
  BY A19
  A19 の (i) は「各時点と各計数下オブジェクト `O` について、`H(O)` は、**その時点で生きているすべての
  活性化**の別名類のうち `O` を指すものが持つ参照の総数以上である」と述べる。`α` は `τ` で生きている
  (D23 -- 中断中の活性化は始まって終わっていない) ので、その和は `α` の類の分を含む。
<1>4. QED
  BY <1>1, <1>2, <1>3

### L42 (義務は pending の bump を覆う) --- これが P18c である

**言明** --- 実行路 `ρ` を辿る 1 回の活性化を固定する。その活性化自身の節点についての各時点 `τ` と
各計数下オブジェクト `O` について、`Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。とくに
`ρ` の上の節点 `q` の入口では `Obl(q, O) ≥ N(q, O)` である。

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
  <2>2. CASE `β(C) = 0`。L41a より `held_ρ(τ, C) ≥ bumps_ρ(τ, C)` であり、局所の定義より
        `obl_ρ(τ, C) = held_ρ(τ, C)` である。
    BY L41a, 本補題の局所の定義
  <2>3. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) ≥ 1`。A19 の (ii-b) より
        `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であり、局所の定義より
        `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ bumps_ρ(τ, C)` である。
    BY A19, 本補題の局所の定義
  <2>4. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) = 0`。P14a より `held_ρ(τ, C) ≥ 1` であり、局所の定義より
        `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ 0 = bumps_ρ(τ, C)` である。
    BY P14a, 本補題の局所の定義, 第 1 節の記法, p13 の DEF 類ごとの参照
    P14a は「`borrow_ify` の出力の各本体、各実行路、各活性化について、ρ-終端が借用する (D14) パラメータ・
    capture の leaf である**計数下**の別名類 (D26) は、活性化の間ずっと参照を少なくとも 1 つ持つ」と
    述べる。`B` は `borrow_ify` の出力の本体である (第 1 節)。`C` はその形の計数下の類であり、類が持つ
    参照の個数は `held_ρ` である (`p13` の `DEF 類ごとの参照`)。
  <2>5. QED
    局所の定義より `β(C)` は 0 か 1 であり、<2>1 より `bumps_ρ(τ, C)` は 0 か 1 以上である。<2>2、
    <2>3、<2>4 はこの 3 つの場合を尽くす。
    BY <2>1, <2>2, <2>3, <2>4
<1>3. QED
  BY <1>1, <1>2, p13 の L17
  <1>1 と <1>2 より `Obl(τ, O) = Σ_C obl_ρ(τ, C) ≥ Σ_C bumps_ρ(τ, C)` である。節点 `q` の入口では
  `p13` の `L17` よりこの右辺は `N(q, O)` である。

### L43 (欠損は pending の bump の一部である)

**言明** --- 対応する活性化を固定する。`ρ` の上の位置 `q` と計数下のオブジェクト `O` について

`d(q, O) = Σ_{t} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]`

である。ここで外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り、内側の和は `ρ` で
活性な名前 `o` を渡る。とくに `0 ≤ d(q, O) ≤ N(q, O)` である。さらに `d(q, O)` は README の P21 (a) の
`k(O)` である。

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
<1>7. `0 ≤ d(q, O) ≤ N(q, O)` である。
  BY <1>6, p13 の L11, DEF N
  `p13` の `L11` より、活性な `o` について `B_ρ(q, e)[o] = e.outstanding[o] ≥ 0` であり、活性でない `o`
  について `B_ρ(q, e)[o] = 0` である。よって <1>6 の和は 0 以上である。DEF N の `N(q, O)` は同じ内側の和を
  `pending(q)` の**すべての**要素について取ったものなので、`d(q, O) ≤ N(q, O)` である。
<1>8. QED
  BY <1>1, <1>2, <1>3, <1>6, <1>7, L38
  README の P21 (a) の `k(O)` は「`q` までに `α` が実行した削除済みの `Retain` のうち、対になる削除済みの
  `Release` をまだ実行していないものが `O` に作った参照の個数」である。L32 の 3 より削除済みの `Retain`
  `t` と対になる削除済みの `Release` は `un_bump_releases[t]` の要素であり、`ρ` の上でそれらが実行され
  終わるのは L38 の 1 より `t` が pending でなくなる時点である。よって「対になる削除済みの `Release` を
  まだ実行していない」`t` の全体は <1>1 と <1>3 が除いた残り、すなわち `q` で pending である `CT` の要素
  であり、その `t` が作った参照のうち処分されていない個数は <1>4 の量である。すなわち `k(O) = d(q, O)`
  である。

### L44 (2 つの実行の対応)

**言明** --- `cancel` の入力プログラム `P` が D12 を満たすとする。`ρ` を `B` の実行路、`ρ'` をそれに
対応する `B'` の実行路、`α'` を `ρ'` を辿る `B'` の活性化、`α` を D29 が `α'` に対応させる `B` の
活性化とする。G1 と G2 の下で、`ρ` の上の各位置 `q` と各計数下オブジェクト `O` について次の 5 つが
成り立つ。`ρ` の終端の `Ret` の消費を行った後の位置についても同じである。

- **(a)** `α` と `α'` は、`q` までに、`Del` の節点を除いて同じ節点を実行し、その時点までに値を得ている
  各変数の値は、D29 の全単射のもとで対応する。
- **(b)** `H'(q, O) = H(q, O) - d(q, O)` である。
- **(c)** `Obl'(q, O) = Obl(q, O) - d(q, O)` である。
- **(d)** `d(q, O) ≥ 1` ならば `H(q, O) ≥ d(q, O) + 1` である。
- **(e)** `q` の節点の実行の間の各時点 `τ` について `H'(τ, O) = H(τ, O) - d(q, O)` であり、
  `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。位置 `q` 自身についても同値である。

**証明**

<1>1. (d) が成り立つ。
  <2>1. `q` が節点の入口であるとき。`d(q, O) ≥ 1` とする。L43 より `N(q, O) ≥ d(q, O) ≥ 1` である。
        L41 より `H(q, O) ≥ N(q, O) + 1 ≥ d(q, O) + 1` である。
    BY L41, L43
  <2>2. `q` が最後の位置 (終端の `Ret` の消費を行った後) であるとき、`d(q, O) = 0` なので (d) は空虚に
        成り立つ。
    BY L38, L43, L32, DEF 欠損
    L38 より各 `t ∈ CT` が pending である区間は終端の `Ret` より真に前で終わるので、終端の `Ret` の
    入口で pending な `CT` の要素は無く、L43 よりそこで `d = 0` である。L32 の 5 より `Del` は
    `Retain` と `Release` だけなので終端の `Ret` は `Del` に入らず、その消費の後も `d` は変わらない。
  <2>3. QED
    BY <2>1, <2>2
<1>2. `q` の節点の実行の間、`α` が中断中である各時点 `τ` について、`d(q, O) ≥ 1` ならば
      `H(τ, O) ≥ d(q, O) + 1` である。
  BY L41c, L43
  L43 より `N(q, O) ≥ d(q, O) ≥ 1` であり、L41c より `H(τ, O) ≥ N(q, O) + 1 ≥ d(q, O) + 1` である。
<1>3. `B'` の所有と借用の割り当て (D14) は `B` のものと同じである。`cancel` は各関数の `clone()` の
      `body` にだけ書き込むので `borrowed_units`・`params`・`capture` を変えず、グローバル初期化子は
      パラメータも capture も持たない。
  BY CODE src/rc_ir/borrow.rs: cancel, D14, D1
<1>4. (a)、(b)、(c)、(e) を `ρ` の上の位置についての帰納法で示す。D3 より `ρ` は有限の列なので整礎で
      ある。
  <2>1. 基底。`ρ` の最初の位置は `B` の根の入口である。DEF 欠損 より `d = 0` である。D29 より 2 つの
        活性化はパラメータと capture に対応する値を受け取るので、そこで値を得ている変数の値は対応し、
        G1 より各対応するオブジェクトの参照カウントは等しいので `H'` は `H` に等しい。D10 の初期値は
        所有する unit の下の inhabited な leaf で決まり、<1>3 より割り当ては同じなので `Obl'` は `Obl`
        に等しい。
    BY DEF 欠損, D29, G1, D10, <1>3
  <2>2. 帰納法の仮定: `ρ` の上の位置 `q` について (a)、(b)、(c) が成り立つ。`q` の節点の実行の後の位置を
        `q'` とする。
    BY 帰納法の仮定
  <2>3. CASE `q` の節点が `Del` の `Retain` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Retain` の行により `O` への参照を `m(O)` 個作り、`H` と `Obl` をそれぞれ `m(O)` 増やす。
        DEF 欠損 より `d` も `m(O)` 増える。D9 の 2 つの表より `Retain` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。よって (a)、(b)、(c) が `q'` で成り立つ。
    <3>1. この節点はカウントを上げるだけなので、解放の走査は走らない。`Retain` 節点は子の活性化も
          作らない -- D24 で子を作る段は (E3) の `App`、(E2) のうちオペランドを適用する `Llvm` の段、
          (E7) のグローバルの初期化の 3 つであり、(E7) が起きるのは「まだ初期化されていないグローバルを
          読む節点の位置」であって、D7 は `Retain` を読む構文に数えない。よって DEF 実行時の量 の 3 群の
          うち第 1 群と第 3 群は空であり、この節点の実行の時点は `q'` だけである。
      BY D10, D24, DEF 実行時の量, D7
    <3>2. QED
      BY <2>2, <1>1, <3>1, D10, D9, DEF 欠損
      (e) が見る時点は <3>1 より `q` と `q'` の 2 つである。どちらでも (b) が等式を与え、`d = 0` の
      ときは両辺が等しく、`d ≥ 1` のときは <1>1 の (d) より `H ≥ d + 1 ≥ 2 > 0` かつ `H' ≥ 1 > 0` な
      ので、0 になることは同値である。
  <2>4. CASE `q` の節点が `Del` の `Release` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Release` の行により `O` への参照を `c(O)` 個処分し、`H` と `Obl` をそれぞれ `c(O)` 減らす。
        DEF 欠損 より `d` も `c(O)` 減る。D9 の 2 つの表より `Release` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。
    <3>1. この節点はオブジェクトを解放しない。
      <4>1. `O` が解放されたとすると `H(q', O) = 0` である。すなわちこの節点の D10 の遷移の後に `O` の
            カウントが 0 になったということであり、その後に走る解放の走査はカウントを上げないので、
            節点の実行が終わった後の値も 0 である。
        BY D7, D24, DEF 実行時の量
      <4>2. `d(q, O) = c(O)` かつ `H(q, O) = c(O)` である。
        BY <4>1, DEF 欠損, L43, <2>2
        `H(q', O) = H(q, O) - c(O) = 0` なので `H(q, O) = c(O)`。また L43 より `d(q', O) ≥ 0` であり、
        `d(q', O) = d(q, O) - c(O)` なので `d(q, O) ≥ c(O)`。一方 <2>2 の (b) より
        `H'(q, O) = H(q, O) - d(q, O) = c(O) - d(q, O) ≥ 0` なので `d(q, O) ≤ c(O)`。
      <4>3. QED
        BY <4>2, <1>1
        `c(O) ≥ 1` (解放が起きたので処分がある) なので `d(q, O) = c(O) ≥ 1` であり、<1>1 の (d) より
        `H(q, O) ≥ d(q, O) + 1 = c(O) + 1` である。これは <4>2 の `H(q, O) = c(O)` に反する。
    <3>2. QED
      <3>1 より解放の走査は走らず、`Release` 節点は子の活性化も作らない -- D24 で子を作る段は (E3) の
      `App`、(E2) のうちオペランドを適用する `Llvm` の段、(E7) のグローバルの初期化の 3 つであり、
      (E7) が起きるのは「まだ初期化されていないグローバルを読む節点の位置」であって、D7 は `Release` を
      読む構文に数えない。よって DEF 実行時の量 の 3 群のうち第 1 群と第 3 群は空であり、この節点の
      実行の時点は `q'` だけである。`H` の変化は D10 の `Release` の行の分だけなので (a)、(b)、(c) が `q'` で成り立つ。
      (e) が見る時点は `q` と `q'` の 2 つであり、どちらでも (b) が等式を与え、`d = 0` のときは両辺が
      等しく、`d ≥ 1` のときは <1>1 の (d) より `H ≥ d + 1 ≥ 2 > 0` かつ `H' ≥ 1 > 0` なので、0 に
      なることは同値である。
      BY <2>2, <1>1, D10, D9, D24, DEF 実行時の量, DEF 欠損, <3>1
  <2>5. CASE `q` の節点が `Del` に入らない。`ρ'` の対応する位置にも同じ節点がある (L30、L31)。
    <3>1. 2 つの実行はこの節点について D10 の同じ行を同じ値に対して適用し、この節点が束縛する変数に
          対応する値を置く。さらに、この節点が子の活性化を作るとき、その子の事象が参照カウントに与える
          変化は 2 つの実行で同じである。`d` は変わらない。
      <4>1. `q` は `Del` に入らないので、L30 と L31 より `ρ'` の対応する位置の節点は `q` と同じ式の
            変位・変数・path・`RcState` を持つ。<2>2 の (a) より、その時点までに値を得ている各変数の値は
            2 つの実行で対応する。
        BY L30, L31, <2>2
      <4>2. この節点が束縛する値は 2 つの実行で対応する。
        <5>1. CASE その値が、節点の形とそれが名指す変数の値だけで決まる。D2 の節点の表と D9 の移動の表が
              その値を定めるので、<4>1 より 2 つの実行で対応する。
          BY D2, D9, A3, A4, <4>1
        <5>2. CASE それ以外である。すなわちこの構文はオペランドから結果が決まらない構文である。D21 は
              「その割り当ては、オペランドから結果が決まらない構文の結果を含む」と述べ、D29 はその各位置
              での結果を 2 つの活性化に同じものとして与える。
          BY D21, D29, <4>1
        <5>3. QED
          BY <5>1, <5>2
          値は、節点の形と名指す変数の値だけで決まるか、決まらないかのどちらかである。
      <4>3. この節点が子の活性化を作るとき、その子の事象が参照カウントに与える変化は 2 つの実行で同じ
            である。
        BY D24, D21, G2, D29, <4>1
        D24 より子の活性化を作る段は 3 つである。(E3) の `App`、(E2) のうちオペランドを適用する `Llvm` の
        段、(E7) のグローバルの初期化である。D21 は `App` について「返す値と、参照カウントに与える変化」
        を活性化の側のデータに置き、G2 は残る 2 つについて同じものを置く。D29 はそのデータを対応する
        2 つの活性化に同じものとして与える。
      <4>4. QED
        BY <4>1, <4>2, <4>3, D10, D16, A4, DEF 欠損,
           CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
        言明の前半は <4>2 であり、後半は <4>3 である。D10 の行が名指す leaf は節点の形と値で決まる
        (D16、A4) ので、2 つの実行で同じである。`q` は `Del` に入らないので DEF 欠損 の 2 つの数え上げは
        変わらない。
    <3>2. `q` の節点の実行の間の各時点 `τ` について `H'(τ, O) = H(τ, O) - d(q, O)` であり、
          `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。位置 `q` 自身についても同値である。
      <4>1. 位置 `q` 自身について、`H'(q, O) = H(q, O) - d(q, O)` であり、`H(q, O) = 0` と
            `H'(q, O) = 0` は同値である。
        BY <2>2, <1>1
        <2>2 の (b) が等式を与える。`d(q, O) = 0` のとき両辺は等しい。`d(q, O) ≥ 1` のとき <1>1 の (d)
        より `H(q, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、`H'(q, O) ≥ 1 > 0` なので、どちらも 0 でない。
      <4>2. DEF 実行時の量 より、この節点の実行の時点は 3 群であり、この順に並ぶ。子の活性化の事象、
            D10 の遷移、解放の走査である。子の活性化を作らない節点では第 1 群が空である。
        BY DEF 実行時の量, D10, D24
      <4>3. 子の活性化の間の各時点 `τ` について、`H'(τ, O) = H(τ, O) - d(q, O)` であり、
            `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。
        <5>1. `τ` までの `H` の変化は、子の活性化の事象が与えるものだけである。<3>1 の後半よりそれは
              2 つの実行で同じなので、<4>1 と合わせて `H'(τ, O) = H(τ, O) - d(q, O)` である。
          BY <3>1, <4>1, <4>2
        <5>2. CASE `d(q, O) = 0`。<5>1 より `H'(τ, O) = H(τ, O)` である。
          BY <5>1
        <5>3. CASE `d(q, O) ≥ 1`。D24 の (E3)、(E7)、および (E2) のうちオペランドを適用する `Llvm` の
              段のいずれについても、子が段を持つ間 `α` は中断中であるから、<1>2 より
              `H(τ, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、<5>1 より
              `H'(τ, O) = H(τ, O) - d(q, O) ≥ 1 > 0` である。どちらも 0 でない。
          BY <1>2, <5>1, D24
        <5>4. QED
          BY <5>1, <5>2, <5>3
      <4>4. D10 の遷移の直後の時点と、解放の走査の間の各時点 `τ` について、
            `H'(τ, O) = H(τ, O) - d(q, O)` であり、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。
        <5>1. 走査の段数についての帰納法で示す。各段の直前で、2 つの実行はそこまでに対応する参照を
              処分しており、各計数下 `O` についてカウントの差は `d(q, O)` である。基底は D10 の遷移の
              直後の時点であり、<3>1 よりその遷移は 2 つの実行で `H` を同じだけ変えるので、<4>3 と
              合わせて差は `d(q, O)` である (子の活性化を作らない節点では <4>1 と合わせる)。
          BY <3>1, <4>1, <4>3, D10
        <5>2. ある時点で `α` の `O` のカウントが 0 になったとする。DEF 実行時の量 より解放の走査は節点の
              実行の最後にあり、その間カウントは下がるだけなので、この節点の実行が終わった後の値
              `H(q', O)` は 0 である。<1>1 の (d) を `q'` に当てると `d(q', O) = 0` であり、`q` は `Del`
              に入らないので `d(q, O) = 0` である。<5>1 より `α'` のカウントも同じ時点で 0 になる。
          BY <5>1, <1>1, DEF 欠損, DEF 実行時の量
        <5>3. ある時点で `α'` の `O` のカウントが 0 になったとする。<5>1 よりその時点の `α` のカウントは
              `d(q, O)` である。`d(q, O) = 0` ならば `α` のカウントも 0 である。`d(q, O) ≥ 1` とすると、
              走査の間カウントは下がるだけなので `H(q', O) ≤ d(q, O)` であり、一方 <1>1 の (d) を `q'`
              に当てると `d(q', O) = d(q, O) ≥ 1` なので `H(q', O) ≥ d(q, O) + 1` である。矛盾するので
              `d(q, O) = 0` であり、`α` のカウントも同じ時点で 0 になる。
          BY <5>1, <1>1, DEF 欠損, DEF 実行時の量
        <5>4. QED
          BY <5>1, <5>2, <5>3, A18
          <5>2 と <5>3 より、各段で 0 になるオブジェクトの集合は 2 つの実行で等しい。よって次の段で処分
          される参照も対応し、<5>1 の帰納法が進む。A18 (a) より生きているオブジェクトのグラフは非巡回で
          あり、段数は有限である。
      <4>5. QED
        BY <4>1, <4>2, <4>3, <4>4
        <4>2 の 3 群の時点を、<4>3 が第 1 群について、<4>4 が第 2 群と第 3 群について尽くし、位置 `q`
        自身は <4>1 が扱う。
    <3>3. QED
      <3>1 と <3>2 より、この節点の実行は 2 つの実行で `H` を同じだけ変え、`Obl` も D10 の行の分だけ
      同じく変え、束縛する変数に対応する値を置く。`d` は変わらないので (a)、(b)、(c) が `q'` で成り立ち、
      (e) は <3>2 である。
      BY <3>1, <3>2, <2>2, D10
  <2>6. QED
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかである (L32 の 5)。
    節点を持たない最後の位置については、<1>1 の (d) より `d = 0` なので (b) から `H' = H` であり、
    (e) の「位置 `q` 自身についても同値である」が成り立つ。
    BY <2>1, <2>3, <2>4, <2>5, <1>1, L32
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

## 4. P18c (義務集合の側の同じ不等式)

**言明 (README)** --- 走査中の各位置と各実行路について、各計数下オブジェクト `O` について
`Obl(O) ≥ n(O)` である。ここで `Obl(O)` は義務集合が持つ `O` への参照の個数、`n(O)` は P18a のものである。

**証明**

<1>1. QED
  BY L42, DEF N, D27
  走査中の位置は D27 に従って節点の訪問の入口であり、DEF N より P18a の `n(O)` はその位置の `N(q, O)` で
  ある。L42 の「とくに」の節が `Obl(q, O) ≥ N(q, O)` を述べる。

## 5. P19 (削除される retain の性質)

**言明 (README)** --- `cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含むすべての実行路に
おいて、`t` より後にある、**その位置での** `t` の `outstanding` のオブジェクトを `acted_on` に含む消費より
前、かつ終端の `Ret` より前に、削除される `Release` 群が `t` の `outstanding` を空にする。さらに、`t` と
ともに削除される各 `Release` は、実行路の上で `t` より後ろにある。

**証明する形**。「その位置での `t` の `outstanding`」は `out(t, ・)` (DEF 訪問) である。示すのは次の
4 つである。`t ∈ Del` が `Retain` 節点であるとき、`t` を含むすべての実行路 `ρ` について、

1. `ρ` の上に `t` より後の `Release` 節点の有限集合 `R_ρ(t) ⊆ Del` があり、
   `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。その最後の要素 `n*(ρ)` の訪問で `t` の `outstanding`
   が空になる。
2. `n*(ρ)` は `ρ` の終端の `Ret` より真に前にある。
3. `ρ` の上で `t` より後にある消費点 `c` であって、`t` が `c` で pending であり、`c` で消費されるある
   leaf `(w, μ)` の `acted_on(w, μ)` が `out(t, c)` の名指す名前を含むものは、存在しない。よって、
   `t` より後にあってそのような名前を名指す消費点は、`t` が pending でなくなった後、すなわち `n*(ρ)` より
   後にある。
4. `r ∈ un_bump_releases[t]` である各 `Release` 節点について、`r` を含むすべての実行路の上で `t` は `r`
   より真に前にある。

**証明**

<1>1. `t ∈ Del` が `Retain` 節点であることと `t ∈ CT` であることは同値である。
  BY L32
<1>2. 1 が成り立つ。
  BY L38, L32, L37, <1>1
  L38 の 3 が `R_ρ(t) ⊆ un_bump_releases[t] ⊆ Del` と静的な収支を与え、L38 の 1 が `n*(ρ)` の訪問で
  `outstanding` が空になることを与える。L37 より `R_ρ(t)` の各要素は `ρ` の上で `t` より後にある。
<1>3. 2 が成り立つ。
  BY L38, <1>1
<1>4. 3 が成り立つ。
  <2>1. そのような消費点 `c` があると仮定する。`t` が `c` で pending なので、L38 より `c ∈ I_ρ(t)` で
        ある。
    BY 仮定, L38
  <2>2. CASE `c` が D9 の消費の表の `App`、`Closure`、`Llvm` の行の位置である。この位置は右辺が `Match`
        でない `Let` 節点であり、`c` の訪問は `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
    <3>1. `consume_rhs` が `rhs_consumes` に渡す `owns(p, leaf)` は
          `self.owned_units.contains(&(p.name, truncate_to_unit(&p.ty, leaf, self.type_env)))` であり、
          `self.owned_units` は `all_owned_units(prog, type_env)` の値である。`all_owned_units` は各関数の
          各パラメータ・capture の各 unit のうち `borrowed_units` に入らないものを集めるので、この述語は
          「呼び出し先がその leaf の unit を所有する (D14)」に等しい。unit は呼び出し先のパラメータの型
          `p.ty` で取る。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/borrow.rs: cancel,
         CODE src/rc_ir/ownership.rs: all_owned_units, D14
    <3>2. `rhs_consumes` は `RcRhs::App(callee, args)` について、callee の全 boxed leaf と、
          `resolve_callee_params` が返すパラメータについて `owns` が真である引数 leaf を報告する。
          呼び出し先が解決しないときは全位置を所有とみなす。<3>1 と合わせて、これは D9 の `App` の行が
          名指す leaf そのものである (解決しない場合は A7 が扱う。A14 より `params[i]` は範囲内である)。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: resolve_callee_params, D9, A7, A14, <3>1
    <3>3. `rhs_consumes` は `RcRhs::Closure(_, caps)` について各 capture の全 boxed leaf を報告する。
          これは D9 の `Closure` の行である。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/ownership.rs: push_boxed_leaves, D9
    <3>4. `rhs_consumes` は `RcRhs::Llvm(llvm_gen, args)` について、`borrows_operand(i)` が偽の
          オペランドの boxed leaf のうち `passthrough_arg_leaves` に入らないものを報告する。
          `passthrough_arg_leaves` は、結果のある leaf の宣言が単一の `Arg(i, σ)` であるような
          `(i, σ)` の全体である。これは D9 の `Llvm` の行である。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: passthrough_arg_leaves,
         CODE src/rc_ir/ownership.rs: as_arg_projection, D9
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4, L36
      仮定より `c` で消費される leaf `(w, μ)` は D9 のこの 3 行のいずれかが名指すものなので、<3>2 から
      <3>4 よりそれは `rhs_consumes` がこの位置で報告する leaf である。L36 の 1 より `c` の訪問は
      `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
  <2>3. CASE `c` が D9 の消費の表の `Destructure` の 2 行の位置である。`destructure_consumes(container,
        fields, type_env)` は、容器が boxed のとき容器の全 boxed leaf を、unbox のとき名前の付いていない
        フィールドの leaf を返す。これは D9 の `Destructure` の 2 行そのものである。L36 の 2 より、`c` の
        訪問はその各 leaf `μ` について `consume_objects(pending, acted_on(container.name, μ))` を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes, D4, D9, L36
  <2>4. CASE `c` が D9 の消費の表の「関数本体の終端の `Ret(x)`」の行の位置である。L38 より `t` は `ρ` の
        終端の `Ret` では pending でないので、仮定に反する。よってこの場合は起こらない。
    BY L38
  <2>5. QED
    <2>2 と <2>3 の呼び出しは、`I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に
    在る時点に走る。仮定よりその `objects` は `out(t, c)` が名指す名前を含むので、L38 の 4 に反する。
    D9 の消費の表の 6 行を <2>2、<2>3、<2>4 が尽くす。よって <2>1 の仮定は成り立たない。L38 より `t` が
    pending である区間は `n*(ρ)` で終わるので、`t` より後にあってそのような名前を名指す消費点は
    `n*(ρ)` より後にある。
    BY <2>1, <2>2, <2>3, <2>4, L38, D9
<1>5. 4 が成り立つ。
  BY L37
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

## 6. P20 (削除は収支を保つ)

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

## 7. P21 (削除がカウントと解放に与えるもの)

**言明 (README)** --- 入力が D12 を満たすとき、`cancel` の出力の各活性化 `α'` と、それに対応する入力の
活性化 `α` (D29) について、次の 2 つが成り立つ。

- **(a)** 各位置において、各計数下オブジェクト `O` について `H_{α'}(O) = H_α(O) - k(O)` である。
  `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` のうち、対になる削除済みの `Release` を
  まだ実行していないものが `O` に作った参照の個数である。
- **(b)** 2 つの活性化は同じ位置で同じオブジェクトを解放する。とくに各読む構文の各位置で解放されて
  いるオブジェクトの集合は等しい。

**証明する形**。`cancel` の入力プログラム `P` が D12 を満たすとする。`ρ'` を `B'` の実行路、`α'` を
それを辿る `B'` の活性化、`ρ` を `ρ'` に対応する `B` の実行路、`α` を D29 が `α'` に対応させる `B` の
活性化とする。G1 と G2 の下で示す。

**証明**

<1>1. (a) が成り立つ。
  BY L44, L43
  L43 の後半より README の `k(O)` は DEF 欠損 の `d(q, O)` であり、L44 の (b) がその等式である。
<1>2. 計数下のオブジェクトについて、2 つの活性化は同じ位置で同じオブジェクトを解放する。
  <2>1. `ρ` の上の位置 `q` の節点の実行の間に `α` が `O` を解放するのは、その実行の間のある時点 `τ` で
        `H(τ, O) = 0` となり、位置 `q` ではそうでないときである。`α'` についても `H'` で同じである。
    BY D7, DEF 解放されている, DEF 実行時の量
  <2>2. QED
    BY <2>1, L44
    L44 の (e) より、`q` の節点の実行の間の各時点で `H(τ, O) = 0` と `H'(τ, O) = 0` は同値であり、位置
    `q` 自身でも同値である。よって <2>1 の条件は 2 つの活性化で同時に成り立つ。
<1>3. グローバル状態のオブジェクト (D26) は 2 つの活性化のどちらでも解放されない。
  BY D26, A8
<1>4. (b) が成り立つ。
  BY <1>2, <1>3, DEF 解放されている, L44, L30, L31
  DEF 解放されている より、`α` の時点 `τ` で `O` が解放されているとは、`H(τ0, O) = 0` を満たす `τ` 以前の
  時点 `τ0` が在ることであり、`α'` については `H'` で同じである。L44 の (e) をその各 `τ0` に当てると、
  この 2 つは同値である。計数下でないオブジェクトについては <1>3 が両方とも解放されないことを与える。
  L30 と L31 より `ρ'` の各位置の節点は `ρ` の対応する位置の節点と同じ式の変位・変数・path を持ち、
  L44 の (a) よりその位置の変数の値も対応するので、読む構文とその読む値は 2 つの実行で対応する。
<1>5. QED
  BY <1>1, <1>4

## 8. P22 (`drop_nodes` の正しさ)

**言明 (README)** --- `drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release` 節点だけを
取り除いた木を返し、他の節点の種類・変数・path・並びを変えない。

**証明**

<1>1. QED
  BY L30, L32, L31
  L30 がこの言明である。`cancel_body` が渡す `S = Del` の要素がすべて `Retain` 節点か `Release` 節点で
  あることは L32 の 5 が述べるので、L31 の仮定も満たされ、実行路の対応 (DEF 路の対応) が定まる。

## 9. P23 (`cancel` は RC 規律を保存する)

**言明 (README)** --- D12 の意味で RC 規律を満たすプログラムを入力とすると、`cancel` の出力も D12 の意味で
RC 規律を満たす。出力の各活性化に対応する入力の活性化が在ることは D29 が与える。

**証明する形**。入力プログラムを `P`、出力を `P'` と書く。`P` は D12 を満たし、A19 を満たすとする。D11 は
`B'` のすべての実行路についての述語なので、示すのは、`B'` の各実行路 `ρ'` と、それを辿る各活性化 `α'` に
ついて (S-a)、(S-b)、(S-c) が成り立つことである。G1 と G2 の下で示す。

**証明**

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
<1>3. `ρ'` に対応する `B` の実行路 `ρ` と、`α'` に対応する `B` の活性化 `α` が定まる。`ρ` について
      D11 の (S-a)、(S-b)、(S-c) が成り立つ。
  BY D12, L31, L32, DEF 路の対応, D29
  L32 の 5 より `Del` は `Retain`/`Release` 節点だけなので L31 が使え、`ρ'` に対応する `B` の実行路 `ρ` が
  ちょうど 1 つ定まる。D29 は `α'` に対応する `B` の活性化がちょうど 1 つ存在することを述べる。`P` が
  D12 を満たすので、`B` のすべての実行路と、それを辿るすべての活性化について D11 の 3 つが成り立つ。
<1>4. `ρ'` の各位置において `Obl'(q, O) = Obl(q, O) - d(q, O)` であり、`d(q, O) ≥ 0` である。
  BY L44, L43
<1>5. (S-a) が `ρ'` で成り立つ。
  <2>1. `ρ'` の上で `Obl'` から参照を取り除く操作は、`ρ` の上の同じ操作から `Del` の `Release` を除いた
        ものである。`ρ` と `ρ'` は `Del` の節点を除いて同じ節点を対応する値に対して実行するので
        (L44 の (a))、残る各操作が取り除く参照の個数は 2 つの実行で等しい。
    BY L44, L30, L31, D10
  <2>2. そのような操作の 1 つが位置 `q` の節点で `O` への参照を `c` 個取り除くとする。その操作の直後の
        時点を `τ` とすると、L42 より `Obl(τ, O) ≥ Σ_{C : obj(C) = O} bumps_ρ(τ, C)` である。
    BY L42
  <2>3. `Σ_{C : obj(C) = O} bumps_ρ(τ, C) ≥ d(q, O)` である。
    <3>1. `q` で pending である `CT` の要素は `τ` の時点でも走査の `pending` に在り、その `B_ρ` は
          `q` の入口での値と等しい。
      BY L38, L32, D27, <2>1
      `q` は `Del` に入らない (`Del` の `Release` は <2>1 で除いてある)。L38 の 4 より `consume_objects`
      はそれらの要素を取り除かない。`un_bump` がそれらを取り除けば、その `Release` は
      `un_bump_releases` に入って `Del` の要素になる (L32 の 4 と 3) ので、`q` が `Del` に入らないことに
      反する。D27 より `consume_objects` はそれらの `B_ρ` を変えない。
    <3>2. `B_ρ(q, e_t(q))[o] ≥ 1` であって `o` が活性かつ `obj_ρ(o) = O` である各名前 `o` は、`ρ` の上の
          スロットであり `obj(C_ρ(o)) = O` である。
      BY p13 の L9, P5, p13 の DEF 名前の活性, p13 の DEF 別名類, 第 1 節の記法
      第 1 節の記法 より、活性な名前 `o` は `ρ` の上のスロットであって `obj_ρ(o)` はそのスロットが指す
      オブジェクトである。`p13` の `DEF 別名類` より `o` はちょうど 1 つの別名類 `C_ρ(o)` に属し、その
      類の `obj(C_ρ(o))` は類の各スロットが指すオブジェクト、すなわち `obj_ρ(o) = O` である。
    <3>3. QED
      BY <3>1, <3>2, L43, p13 の DEF 類ごとの参照
      L43 より `d(q, O)` は、`q` で pending である `CT` の要素の `B_ρ(q, ・)` を、活性かつ
      `obj_ρ(o) = O` である名前 `o` について足したものである。`p13` の `DEF 類ごとの参照` の `bumps` の
      定義より `Σ_{C : obj(C) = O} bumps_ρ(τ, C)` は、`τ` の `pending` の**すべての**要素の
      `B_ρ(τ, ・)` を、`ρ` の上のスロットであって `obj(C_ρ(o)) = O` である名前 `o` について足したもので
      ある。<3>1 より前者の要素と値は後者の和にそのまま現れ、<3>2 より前者が数える名前は後者が数える
      名前に含まれる。残る項は `p13` の `L11` より 0 以上なので、後者は前者以上である。
  <2>4. QED
    BY <2>2, <2>3, <1>4
    `Obl(q, O) - c = Obl(τ, O) ≥ d(q, O)` なので `Obl'(q, O) = Obl(q, O) - d(q, O) ≥ c` である。すなわち
    取り除かれる参照は `Obl'` に入っている。
<1>6. (S-b) が `ρ'` で成り立つ。
  <2>1. `ρ` の終端の `Ret` では、`CT` の要素はどれも pending でない。
    BY L38
    L38 より各 `t ∈ CT` が pending である区間は `n*(ρ)` で終わり、`n*(ρ)` は終端の `Ret` より真に前に
    ある。
  <2>2. 終端の `Ret` の位置とその消費を行った後の位置で `d = 0` である。
    BY <2>1, L43, L32, DEF 欠損
    L43 より `d` は `CT` の pending な要素の `B_ρ` の和であり、<2>1 よりその和は空である。終端の `Ret` は
    `Del` に入らない (L32 の 5 より `Del` は `Retain` と `Release` だけ) ので、その消費の後も `d` は
    変わらない。
  <2>3. QED
    BY <1>3, <2>2, L44, L30, L31
    <1>3 の (S-b) より `ρ` の終端の `Ret` の消費の後の `Obl` は空である。L44 の (c) と <2>2 より
    その時点の `Obl'` は `Obl` に等しいので、`Obl'` も空である。L30 と L31 より `ρ'` の終端の `Ret` は
    `ρ` の終端の `Ret` と同じ節点であり、L44 の (a) より消費する値も対応する。
<1>7. (S-c) が `ρ'` で成り立つ。
  <2>1. `ρ'` の読む構文 (D7) と `Retain`/`Release` 節点は、`ρ` のそれらから `Del` の節点を除いたもので
        あり、各位置で名指す変数と path は同じである。L44 の (a) よりその位置の変数の値も対応するので、
        読みうるオブジェクトと触れるオブジェクトは 2 つの実行で対応する。
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

## 10. P24 (D12 が見ない部分の保存)

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

## 11. README へ差し戻す 2 点

第 2 節の `DEF README へ差し戻す 2 点` が言明を置いた `G1` と `G2` について、README のどこが何を述べて
いて、何を足すことを求めるかを書く。**この 2 つのほかに、この文書が閉じられなかった点は無い。**

### 差し戻し 1 (G1 -- D29 に開始時の参照カウントの条項が要る)

**README の D29 が述べていること。** 第 3.1 節の D29 は次の 3 つを述べる。

> `B'` の活性化 `α'` に**対応する** `B` の活性化とは、D21 の 2 つのデータ -- パラメータ・capture の値と、
> オペランドから結果が決まらない 3 種の構文の各位置での結果 -- を `α'` と共有する `B` の活性化である。

> **値とオブジェクトは 2 つの活性化が別々に割り当てる。**「同じ値」「同じオブジェクト」と言うとき指すのは、
> 次の 3 つの行が生成する全単射のもとで対応するものである。

> D21 より、この 2 つのデータを与えれば本体の活性化は 1 つに決まる。よって **`α'` に対応する `B` の活性化は
> ちょうど 1 つ存在する。対応は前提ではなく、D21 と P22 から作られる。**

**参照カウントについての条項は無い。** D29 が対応させるのは値とオブジェクトであり、オブジェクトの参照
カウントが 2 つの活性化で等しいことはどこにも述べていない。

**それが要る所。** P21 (a) は 2 つの活性化の `H` を絶対値で比べる `H_{α'}(O) = H_α(O) - k(O)` である。
活性化の最初の位置では `k(O) = 0` なので、この等式はそこで `H_{α'}(O) = H_α(O)` を要求する。D29 のデータ
はそれを与えない。

**求めるのは、D29 に次の 1 文を足すことである。** 「対応する 2 つの活性化は、活性化の最初の時点において、
この全単射で対応する各オブジェクトについて等しい参照カウントを持つ。」

**これは仮定ではなく、対にする相手の選び方である。** D11 は `B` のすべての活性化について条件を課すので、
`α'` の開始時のカウントから始まる `B` の活性化もその中に在り、それを `α` に選べる。D29 が `App` の結果を
「外から与えるデータ」として扱うのと同じ扱いであり、果たす者は要らない。D21 が「この 2 つのデータを与えれば
活性化は 1 つに決まる」と言うところは、「この 2 つのデータと開始時の参照カウントを与えれば」に変わる --
各節点がカウントに与える変化は、D10 の行と、子の活性化を作る 3 つの段について与えられるデータ (G2) が
決めるので、開始時のカウントを与えれば以後のカウントは決まる。

**絶対の形を採り、相対の形を採らなかった理由。** 「活性化の最初の時点からの `H` の変化を比べる」相対の形
なら開始時のカウントは要らない。しかしそれでは P21 (b) が出ない。(b) は `H` が 0 になる時点についての
主張であり、開始時のカウントに差 `δ` があれば、`H_α` が 0 にならない位置で `H_{α'}` が 0 になりうる。
実際、`H_{α'}(q, O) = H_α(q, O) - k(O) + δ(O)` となるので、`δ(O) < 0` のとき P18a の `+1` は差を埋め
ない。**(b) は絶対の形からしか出ない。**

**A19 (i) だけに載せる形も採らなかった。** A19 (i) は「その時点で生きているすべての活性化」を数えるので、
`k(O)` を「生きているすべての活性化の欠損の総和」に取れば、開始時のカウントを与えずに `H_{α'} ≥ 1` が
出る。しかしその `k(O)` は活性化の林についての量であり、1 つの本体についての命題では書けない。README の
層 5 は「**2 つの実行を突き合わせる命題は要らない。** … よって P14 と P23 は本体ごとに閉じ」と述べて
いるので、この向きは層 5 の方針と合わない。加えてその形は A19 (i) を `cancel` の**出力**についても要求
するが、A19 の範囲は「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel`
の入力) の両方について」であって出力を含まない。

**A19 (i) の「中断中を含む」は別の所で効いている。** `L41c` が、`α` が子の活性化を待って中断中である
時点について `H(τ, O) ≥ N(q, O) + 1` を出すのに、A19 (i) の「その時点で生きているすべての活性化」と
「中断中はその活性化の節点が走らないので `held` も `bumps` も動かず」の 2 つを読む。この段が無いと、
子の活性化の間に `H'` が 0 に落ちないことが言えない。

### 差し戻し 2 (G2 -- 子の活性化を作る段は 3 つあり、D21 が扱っているのは 1 つだけ)

**README の D21 が述べていること。** 第 3.1 節の D21 は次のとおりである。

> **その割り当ては、オペランドから結果が決まらない構文の結果を含む。** 3 種ある。
>
> - **一意性の観測点** (D18)。返す `Bool` はその時点の参照カウントで決まり、オペランドでは決まらない。
> - **外部の状態を読む `Llvm` の演算**。環境 (A17) を読む。
> - **`App`**。返す値と、参照カウントに与える変化は、呼び出し先の本体が決める。1 つの本体だけを見ている
>   間、これは外から与えられる量である。

**同じ README の D24 は、子の活性化を作る段を 3 つ挙げている。** 第 3.6 節の (E2) の後にある段落と、
(E7) の段の文面は次のとおりである。

> **`Let(x, Llvm(gen, args), k)` の段は、活性化を作りうる。** その op の生成コードがオペランドを関数として
> 適用するとき (`LLVMGen::applies_a_function_operand` が真を宣言する op)、適用された関数の本体の活性化が
> 作られ、`a` はそれが終わるまで中断中である。(E3) と同じ形であり、違うのは呼び出し先を決めるのが `callee`
> の値ではなく op の生成コードだということだけである。

> **(E7) グローバルの初期化の段。** 生きている活性化 `a` が、まだ初期化されていないグローバル `g` を読む
> 節点の位置にあるとき、`g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。

そして「**活性化の林。** (E1) が作る活性化を**根**、(E3) と (E7)、および (E2) のうちオペランドを適用する
`Llvm` の段が作る活性化を、それを作った活性化の**子**と呼ぶ」と述べる。

**それが要る所。** `L44` の (b) と (e) は、節点が作る子の活性化が参照カウントに与える変化が 2 つの活性化
で同じであることを使う。その変化を決めるのは子の活性化であり、その本体は `P` と `P'` で別である。
`App` とまったく同じ形であって、`App` については D21 の行が「返す値と、参照カウントに与える変化」を
活性化の側のデータに置いている。残る 2 つの段についてその 1 文が無いと、`L44` はそこで閉じない。

**求めるのは、D21 の列挙に次の 2 行を足すことである。** どちらも `App` の行と同じ文である。

- **オペランドを適用する `Llvm` の演算** (`LLVMGen::applies_a_function_operand` が真を宣言する op)。
  返す値と、参照カウントに与える変化は、適用された関数の本体が決める。
- **まだ初期化されていないグローバルの読み** (D24 の (E7))。読める値と、参照カウントに与える変化は、
  そのグローバルの初期化子の本体が決める。

**この 2 つは実在する。** 前者について、README 第 8 節の #551 の 4 件目が `Option::mod_some`、`Std::fix`、
各 union の `mod_{変位}`、`with_retained`、`Array::borrow_elements`、`mutate_boxed` 系、
`Array::mutate_elements` 系の 8 つを挙げている。後者は、グローバル値を持つプログラムのすべてで起きる。

**(E7) が起きる位置は D7 の読む構文に限る。** (E7) の文面が「まだ初期化されていないグローバル `g` を読む
節点の位置にあるとき」と述べるからである。`Retain`、`Release`、`Let(x, Var(y), k)`、`Ret` は D7 の読む
構文ではないので、`L44` の `Del` の `Retain`/`Release` の場合はこの段を持たない。

**値の側は D21 の規則で足りる。** `L44` は、束縛される値について D21 の列挙を読まず、「オペランドから
結果が決まらない構文の結果は活性化の側のデータである」という D21 の規則だけを読む。よって値の側は
列挙が尽きていなくても閉じる。**ただし列挙は尽きていない。** 上の 2 つに加えて、`unique_check_operand`
を宣言する op がある -- README の A3 が「そうした op は実行時に参照カウントで分岐し、一意の腕では
オペランドのオブジェクトをそのまま返す」と述べるとおり、その結果はオペランドから決まらないが、
`unsafe_is_unique` ではないので D18 の一意性の観測点ではなく、環境 (A17) を読むわけでもないので
「外部の状態を読む `Llvm` の演算」でもない。「3 種ある」を数え直すことを求める。

### `p20-borrow-ify.md` が名指す `p40` の `H2`

`p20-borrow-ify.md` は 3 か所で `p40-cancel-soundness.md` の `H2` を名指す。`H2` はこの文書から消え、
その言明は README の P14a になった (`L42` の <1>2 の <2>4 が P14a を読む)。`p20` の側の名指しを P14a へ
書き替えることを求める。
