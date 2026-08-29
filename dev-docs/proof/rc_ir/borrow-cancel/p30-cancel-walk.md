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
