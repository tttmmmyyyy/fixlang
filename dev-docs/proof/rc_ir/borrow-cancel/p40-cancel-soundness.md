# P18c, P19 - P24: `cancel` が RC 規律を保存すること

この文書は README の 7 命題 P18c, P19, P20, P21, P22, P23, P24 を証明する。README の定義 D1 - D34、
仮定 A1 - A24、層 0 の命題 P28・P29、および命題 P1 - P18b と P14a の**言明**の上に立つ。主定理 T は
`p70-main-theorem.md` の担当であり、この文書は扱わない。

この文書が読んだコードのコミットは `0adefcea17796abd5bd7a949749b2bd704136caa` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で、`src/` に変わったのは
コメントだけである。

引用する外部の補題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF bump の帰属` (`B_ρ`)、`DEF ρ-歩みと ρ-終端` -- と、補題 `L7`
(boxed leaf の路は反鎖をなす)、`L9` (`identity` は inhabited を決める)、`L10a` (静的な数え上げと実行時の
作用が活性な名前で一致する)、`L11` (非活性な名前では `B` は空)、`L17`
(`N` は別名類ごとの `bumps` の和である)。

`B_ρ` の個数が 0 以上であることは README の P18b が言う -- P18b は `B(p, ρ)` を「その `Retain` が `ρ` で
実際に作った参照のうち、`ρ` 上でまだ処分されていないもの」を数えた多重集合と読む。**`outstanding` に
ついて使うのは `covers`、すなわち `outstanding[o] ≥ B_ρ(・, ・)[o]` の向きだけである。**
これらは `p30 の L10`、`p13 の L17` のようにファイル名を添えて引用する。

別名類 (`obj(C)`、`T_ρ(C)`) は **D33**、類ごとの参照 `held_ρ` は **D34** であり、どちらも枠に在る。

この文書が導入する補題は `L30` から番号を付ける。`p30` と `p13` の補題の番号と衝突させないためである。
補題は依存の順に並ぶので、`L43a` を読む `L41c` はその後に在る。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P18c | 証明済み (`L42`)。A19、P14a、`p13` の `L17` を読む |
| P19 | 証明済み |
| P20 | 証明済み |
| P21 | 証明済み。(a) は `L43` と `L44` の (b) に、(b) は `L44` の (e) に載る |
| P22 | 証明済み |
| P23 | 証明済み。(S-a) は `L42` に、(S-b) と (S-c) は `L44` に載る |
| P24 | 証明済み。第 4 の箇条は `rewrite_inner` の 8 腕についての構造帰納で出る |

**この文書に開いている点は無い。**

**対応する 2 つの活性化が開始時に等しい参照カウントを持つことは D29 の第 2 行が、対応するオブジェクトが
保持する値が対応することは D29 の第 5 行と A4 が、子の活性化を作る段が参照カウントに与える変化を活性化の
側のデータとすることは D21 の第 4 行が与える。** `L44` はこの 3 つをそこから読む。

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
渡し (`CODE src/build/build_object_files.rs: optimize_rc_program`)、`cancel` と `borrow_ify` はどちらも
`pub(crate)` なのでクレートの外から呼べない (`CODE src/rc_ir/borrow.rs: cancel`,
`CODE src/rc_ir/borrow.rs: borrow_ify`、P15)。よって本体 `B` について、A19 の
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

活性化の**時点**とは、参照の作成・処分の事象の間の区切りをいう。事象は 4 種である。その活性化の節点に
ついて D10 の行が定めるもの、その活性化が作った子の活性化 (D24 の (E3)、(E7)、および (E2) のうち
オペランドを適用する `Llvm` の段) の事象、D24 の (F) の解放が処分するもの、および **(F) の解放が作るもの**
-- `_dtor` の欄の関数に適用の分の参照を与える retain と、その適用が作る活性化の事象である。**子の活性化の
間の時点も、親の活性化の時点である。**

`B` の 1 回の活性化 `α` の各時点 `τ` とオブジェクト `O` について、`H(τ, O)` をその時点の `O` の参照
カウント (D7) とする。節点の時点 (下記) については、`Obl(τ, O)` をその時点の義務集合 (D10) が
持つ `O` への参照の個数とする。`B'` の活性化 `α'` については `H'`、`Obl'` と書く。

`ρ` の上の**位置**とは、`ρ` の上の節点 `q` の実行の直前の時点をいい、その時点の量を `H(q, O)`、
`Obl(q, O)` と書く。これに、`ρ` の終端の `Ret` の消費を行った後の時点を 1 つ加え、これも位置と呼ぶ。
D3 より `ρ` は有限の列なので、位置は有限個である。`d` と `H` と `Obl` はこの最後の位置でも定まる。
1 つの節点の実行が複数の事象を起こすことがあるので、時点は位置より細かい。

節点 `q` の実行が終わった時点を `q'` と書く。`q` の実行の終わりと、`ρ` の上で `q` の次にある節点の実行の
始まりとの間には事象が無いので、`q'` はその次の節点の位置である。`q` が `ρ` の終端の `Ret` であるときは、
`q'` は上で加えた最後の位置である。

対応する活性化 `α`、`α'` を固定したとき、`H'(q, O)`、`Obl'(q, O)` は、`α'` について `ρ'` の対応する
位置で読んだ値とする。`q ∈ Del` のときは `ρ'` にその節点が無いので、`ρ'` の上で `q` の直後にあたる位置の
値を取る。**`H` と `H'` を 1 つの `O` について並べて書くとき、2 つの活性化のオブジェクトを結ぶのは D29 の
全単射である。** `O` はその全単射が対応させるオブジェクトを渡る。D29 の第 5 行より、その定義域は 2 つの
活性化がそれぞれ到達できる (D25) オブジェクトの全体であり、どちらかの活性化が触れうる計数下オブジェクトは
すべてここに入る。

**節点の実行の扱い**。1 つの節点の実行の時点は、次の 4 群がこの順に並んだものである。

1. **子の活性化の事象**。D24 の (E3)、(E7)、および (E2) のうちオペランドを適用する `Llvm` の段が作る
   活性化の事象である。
2. **D10 の処分**。その節点について D10 の消費の行と `Release` の行が定める処分を、まとめて 1 つの
   遷移として適用する。
3. **D10 の作成**。その節点について D10 の生成の行と `Retain` の行が定める作成を、まとめて 1 つの
   遷移として適用する。
4. **解放の走査**。第 2 群と第 3 群を適用した後にカウントが 0 であり、位置 `q` ではそうでなかった
   計数下オブジェクト (D26) について D24 の (F) の解放が走り、解放が処分する参照についてこれを
   繰り返す。**この群は参照を作り、活性化も作る** -- 解放されるオブジェクトが `Std::FFI::Destructor` の
   ものであるとき、D24 の (F) より `_dtor` の欄の関数に retain が立ち、その適用が活性化を作る。
   D24 の (F) より、解放の連鎖はオブジェクトの個数で抑えられるので有限で終わる。

節点の実行の**節点の時点**とは、位置 `q` と、第 2 群の直後の時点と、第 3 群の直後の時点をいう。`Obl` を
読むのはこの 3 つの時点である。第 1 群と第 4 群の事象は、`α` 自身の節点について D10 の行が定めるもので
はない。

**処分を作成より前に置く。** D10 は義務集合を「実行路上の各位置における」量として定めるだけで、1 つの
節点の中の順序を定めない。この順に取るのは、D11 の (S-a) が `Obl` から参照を取り除いた直後の時点を
見るからである。まとめて 1 つの遷移とするのは、1 つの節点が同じオブジェクトへの参照を複数処分する
とき、(S-a) が要求するのはその総和が `Obl` に入っていることだからである。

**解放の走査を作成より後に置く。** boxed 容器の `Destructure` は 1 つの節点で処分と作成の両方を行う --
容器の参照を捨て、各名前付きフィールドの参照を作る (D10)。容器のカウントが 0 になると、D24 の (F) の
走査は容器が持つフィールドの参照を処分する。作成を走査より後に置くと、その走査でカウントが 0 に
なったフィールドのオブジェクトに対して D10 の生成が参照を作ることになり、D7 の解放後のオブジェクトに
触れる形になる。

**子の活性化の事象は、D10 の 2 つの群より前に置く。** (E3) の `App`
については順序が `H` を変えない -- (E3) が引数の参照を子へ渡し、(E4) が結果の参照を子から受け取るので、
`App` の節点自身の D10 の行はどちらも `H` を変えない (D24 の (E2) の表の `App` の行が「**変わらない**」と
述べる)。(E2) のうちオペランドを適用する `Llvm` の段については、D24 が「適用された関数の本体の活性化が
作られ、`a` はそれが終わるまで中断中である」と述べるので、その段が完了するのは子が終わった後である。
(E7) のグローバルの初期化は、その節点が値を読む前に走る。

**走査の状態との対応。** 実行時の 1 つの事象 (D10 の行が定める参照の作成・処分) の直後の時点には、
走査がその事象に対応する操作を行った直後の `pending` が対応する (A19、D27)。第 1 群と第 4 群の事象には
走査の操作が対応しない -- その間の時点に対応するのは、その節点の訪問の中でそれまでに施された操作の
後の `pending` である。

### DEF 解放されている

オブジェクト `O` が活性化のある時点 `τ` で**解放されている**とは、`τ` かそれより前の時点で `O` の参照
カウントが 0 であること、すなわち `H(τ0, O) = 0` を満たす `τ` 以前の時点 `τ0` が在ることをいう。D7 は
参照カウントが 0 になったオブジェクトが解放されると定めるので、これは D7 の「解放される」を時点の言葉で
書いたものである。`α'` についても `H'` で同じように定める。

### DEF 欠損

対応する活性化を固定する。`ρ` の上の位置 `q` と計数下 (D26) のオブジェクト `O` について、

`d(q, O) :=` (`q` より前に実行された `Del` の `Retain` 節点が `O` への参照を作った個数)
`-` (`q` より前に実行された `Del` の `Release` 節点が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。

**同じ数え上げを位置の間の時点へ広げる。** 位置 `q` の節点の実行の間の各時点 `τ` について、上の 2 つの
数え上げを「`τ` より前に実行された」と読んだ値を `d(τ, O)` と書く。位置も時点なので、この記法は位置に
ついては上の値に一致する。

**この 2 つの数え上げが渡るのは、`α` が `ρ` の上で実行した `Del` の節点だけである。** `DEF 実行時の量` の
第 1 群 (子の活性化の事象) と第 4 群 (解放の走査とそれが作る活性化の事象) は、どちらも `α` が `ρ` の節点を
実行する事象ではないので、`d` を動かさない。したがって

- `q ∉ Del` のとき、`q` の節点の実行のどの時点でも `d(τ, O) = d(q, O)` である。
- `q ∈ Del` のとき、`d` は 1 度だけ動く。`Retain` 節点では第 3 群の直後に、`Release` 節点では第 2 群の
  直後にである。動いた後の値は、`q` の節点の実行の後の位置 `q'` の値 `d(q', O)` である。

**`d(q, O)` は README の P21 (a) の `k(O)` である。** L43 の <1>8 が、`d(q, O)` が「`q` までに実行された
`Del` の `Retain` が `O` に作った参照のうち、その `Retain` と対になる `Del` の `Release` がまだ処分して
いないものの個数」であることを示す。`k(O)` を `Retain` の個数ではなく参照の個数で数えることが要る理由は
第 11 節が反例で述べる。

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

### DEF 類ごとの義務

別名類 `C` (D33) について、`β(C)` を、`C` の ρ-終端が借用する (D14) パラメータ・capture の leaf である
とき 1、そうでないとき 0 と定め、`held_ρ(τ, C)` (D34) が定まる各時点 `τ` について

`obl_ρ(τ, C) := held_ρ(τ, C) - β(C)`

と置く。**これは A19 (i) の `d(C)` である** -- A19 (i) は
`d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置き、その角括弧が `β(C)` である。
以下で `d` はこの文書の `DEF 欠損` の量を指すので、A19 (i) の `d(C)` はこの名前で書く。

`bumps_ρ(τ, C)` は、A19 (ii-b) が言う「走査がその類について `pending` に数えている bump の個数」で
ある。その帰属は D27 が定めるので、`τ` に対応する走査の状態 (`DEF 実行時の量`) の `pending` について
`bumps_ρ(τ, C) = Σ_{p ∈ pending} Σ_{o ∈ C} B_ρ(τ, p)[o]` である。`p13` の `L17` の `bumps_ρ` はこれで
ある。

計数下のオブジェクト `O` と時点 `τ` について、`S(τ, O)` を、`obj(C) = O` (D33) であり**開始の時点
(D34) が `τ` 以前である**計数下の別名類 `C` の全体とし、

`b(τ, O) := Σ_{C ∈ S(τ, O)} bumps_ρ(τ, C)`

と置く。D34 は `held_ρ(τ, C)` をその類の開始の時点以後の `τ` についてだけ定め、`C` を渡る総和にその
条件を付ける。以下の類を渡る総和はどれも `S(τ, O)` を渡る。

## 3. 局所の補題

### L30 (`drop_nodes` の作用)

**言明** --- `S` を `NodeId` の集合とする。`drop_nodes(B, S)` は、`B` の木から、`NodeId` が `S` に入る
`Retain` 節点と `Release` 節点だけを取り除いた木を返す。残る各位置の式の変位、変数、path、`RcState`、
source、`Match` のアームの本数と並び、および継続の順序は変わらない。

**証明**

<1>1. `drop_nodes(node, to_delete)` は `grow_stack(|| drop_nodes_inner(node, to_delete))` であり、A15 より
      `drop_nodes_inner` をちょうど 1 回呼んでその値を返す。
  BY CODE src/rc_ir/borrow.rs: drop_nodes, A15
<1>2. `drop_nodes` が読む `NodeId` は入力の木のものであり、走査が読んだものと同じである。すなわち、木の
      1 つの位置について、走査が計算した `node_id` の値と `drop_nodes` が計算する値は等しい。
  <2>1. `cancel_body` は 1 つの共有参照 `body: &RcExprNode` について `analysis.walk(body, ・, ・)` を
        呼び、その値から作った集合を持って `drop_nodes(body, &analysis.cancelled())` を呼ぶ。`body` は
        `prog: &RcProgram` から借用したものであり、この 2 つの呼び出しの間に木を変える操作は無い --
        `cancel` が持つのは共有参照だけで、`funcs` と `globals` を作る写像はそれぞれの `f.body` /
        `g.init` を読むだけである。
    BY CODE src/rc_ir/borrow.rs: cancel
  <2>2. `node_id(n)` は `n.expr` が指す `RcExpr` の番地である。`Arc<T>` は `T` をヒープの 1 つの割り当ての
        中に置き、`Arc::as_ref` はその割り当ての中の `T` の番地を返す。その番地は割り当てが生きている間
        変わらない -- `Arc` の値を動かして動くのはポインタだけであり、割り当てが返るのは最後の強参照が
        落ちたときである (Rust 標準ライブラリの `std::sync::Arc` の契約)。`NodeId` の doc がこの性質を
        「the address of its expression, stable while the tree is borrowed」と述べる。
    BY CODE src/rc_ir/borrow.rs: node_id, CODE src/rc_ir/borrow.rs: NodeId,
       CODE src/rc_ir/ast.rs: RcExprNode
  <2>3. QED
    BY <2>1, <2>2
    <2>1 より 2 つの走査は同じ木の同じ `Arc` を読み、その間 `body` の借用が生きているので、<2>2 より
    どの位置についても 2 つの `node_id` の値は等しい。
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
        `x`、`scrut`、各アームの `tag`/`payload`/`payload_state`、アームの本数と並びを変えずに、
        `&node.source` を付けて節点を積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm::with_body
  <2>4. CASE `node` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
        `drop_nodes(k, to_delete)` を 1 回呼び、`x` と `rhs` を変えずに `&node.source` を付けて節点を
        積む。`to_delete` の検査はしない。`match` の腕はこの順に並んでいるので、この腕に落ちる `rhs` は
        `RcRhs::Match` ではない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, rhs, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner
  <2>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)` または `RcExpr::Eval(v, k)`
        である。この 2 つの腕は `drop_nodes(k, to_delete)` を 1 回呼び、他のフィールドを変えずに
        `&node.source` を付けて節点を積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Eval(v, k)` の腕
  <2>6. CASE `node` の式が `RcExpr::Ret(v)` である。この腕は `v` を変えずに `&node.source` を付けて
        1 節点を作って返す。
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
  <2>1. その `get` は `Some` を返す。`self.all_retains` に `retain` を積むのは `walk_inner` の
        `RcExpr::Retain(v, path, _, k)` の腕だけであり、その腕は同じ `retain` について
        `self.un_bump_releases.entry(retain).or_default()` を評価するので、その鍵の項目が在る。
        `p30` の `L10` より走査は記録を取り除かない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, p30 の L10
  <2>2. QED
    BY <2>1, CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, DEF 削除集合
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
<1>2. 木の根でない各節点 `n` について、`n` を含むすべての実行路は、`n` の親 (DEF 子と親) を含み、その路の
      上で親は `n` より真に前にある。
  <2>1. D2 より本体は木であり、根でない各節点はちょうど 1 つの親を持つ。
    BY D2
  <2>2. D3 の規則のうち、実行路に `n` を加えるのは次の 3 つの場合だけである。`n` が根である場合、`n` が
        直前の節点の継続である場合、`n` がアーム本体の根である場合。`n` は根でないので第 1 の場合は
        起こらない。第 2 の場合の直前の節点は `n` の親か、`n` の親が `Match` であるときそのアーム本体の
        終端の `Ret` であり、後者のときも D3 よりその `Match` 節点自身が路の上でより前にある。第 3 の
        場合の直前の節点は `n` の親である `Match` 節点である。
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
      `self.consume_objects(&mut pending, &objects)` である。`merge`、`cancelled`、`un_bump` はこの
      呼び出しを持たない。`consume_rhs` がこの呼び出しに届くのは `consume` を通してであり、<1>3 が
      その 1 か所を展開する。
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

### L40 (歩みの位置は boxed leaf である)

**言明** --- `ty(u)` を変数 `u` に束縛された値の型 (D6) とする。次の 2 つが成り立つ。

1. `σ` が `boxed_leaf_paths(ty(u))` の要素であり、`u` の束縛が `Binding::Llvm` であるとき、
   `origin_inner(u, σ)` が呼ぶ `origin_from_leaves_under` は `origin` を呼ばない。
2. `σ` が `boxed_leaf_paths(ty(u))` の要素であるとき、`origin_inner(u, σ)` が呼ぶ `origin(u', σ')` の
   `σ'` は `boxed_leaf_paths(ty(u'))` の要素である。

したがって、スロット (D6) から始まる別名類の ρ-歩み (`p13` の `DEF ρ-歩みと ρ-終端`) の各位置 `(u, σ)`
について、`σ` は `ty(u)` の boxed leaf である。**とくに、`origin_from_leaves_under` が
`truncate_to_unit(ty(args[j]), σ')` を行き先の path として辿る辺 -- 行き先の path が leaf でないことが
ありうる唯一の辺であり、D17 の第 3 行の但し書きがそれを述べる -- は、歩みの上では取られない。**

**D17 との対応**。D17 は `origin` が辿る各辺を、`origin` に与えた path `π` の下の leaf `λ` の写り方として
述べる。`π` 自身が boxed leaf であるときは `λ = π` であり、そのとき辺の行き先の path は D17 が言う `λ` の
像そのものである。2 の証明の各場合でこれを使う。

**証明**

<1>1. `origin(u, σ)` の評価が呼ぶ `origin` は、`origin_inner(u, σ)` が呼ぶものだけである。`origin` は
      `vars.origins` の memo を引き、無ければ `grow_stack(|| origin_inner(vars, type_env, var, path))` を
      呼んでその値を記録するほかに何もしない。A15 より `grow_stack` は閉包をちょうど 1 回呼ぶ。
  BY CODE src/rc_ir/ownership.rs: origin, A15
<1>2. 1 が成り立つ。
  <2>1. `origin_inner` の `Binding::Llvm(llvm_gen, args, result_ty)` の腕が `origin_from_leaves_under` を
        呼ぶのは、`decl.leaf_origins_at(σ).and_then(as_arg_projection)` が `None` のときである。ここで
        `decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env)` であり、`collect_bindings` が
        `Binding::Llvm` の第 3 欄に置くのは `Let(x, RcRhs::Llvm(..), k)` の `x.ty` なので
        `result_ty = ty(u)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings, D6
  <2>2. `leaf_origins_at(σ)` は `Some` を返す。A3 は「`result_prov` は結果の
        leaf ごとに `LeafOrigin` の集合 (`LeafOrigins`) を宣言する」と述べるので、`decl` が持つ鍵は
        `boxed_leaf_paths(result_ty)` の各元である。`Provenance::build_shape` はその形の値を作る --
        `LeafMap::build_shape` が `boxed_leaf_paths(ty, type_env)` の各元を鍵に据える。
        `leaf_origins_at` はその写像を鍵 `σ` で引くだけであり (`LeafMap::get`)、`σ` は
        `boxed_leaf_paths(ty(u)) = boxed_leaf_paths(result_ty)` の元である。
    BY <2>1, A3, CODE src/rc_ir/provenance.rs: Provenance::build_shape,
       CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>3. `σ` の宣言は、空集合か、`Fresh` ただ 1 つか、`Unknown` ただ 1 つである。A3 より、このコミットの
        すべての op の宣言は結果の各 leaf に元数 0 か 1 の `LeafOrigins` を与える。元数 1 でその元が
        `LeafOrigin::Arg` ならば `as_arg_projection` は `Some` を返し、<2>1 の場合に入らない。
    BY <2>1, <2>2, A3, CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>4. `decl.leaf_origins_under(σ)` が渡す集合は、`σ` 自身の宣言 1 つだけである。`leaves_under(path)` は
        鍵が `path` を前置に持つ元を渡し、`p13` の `L7` より boxed leaf の路は反鎖をなすので、`σ` を
        前置に持つ boxed leaf は `σ` 自身だけである。
    BY <2>2, p13 の L7, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>5. QED
    BY <2>3, <2>4, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    `origin_from_leaves_under` の最初のループは、`leaf_origins_under(path)` が渡す各集合の
    `LeafOrigin::Arg` の元についてだけ `operand_units` に元を入れる。<2>3 と <2>4 よりその元は無いので
    `operand_units` は空である。`origin_from_leaves_under` が `origin` を呼ぶのは `operand_units` の各元に
    ついてだけである。
<1>3. 2 が成り立つ。
  <2>1. CASE `u` の束縛が `None`、`Binding::Param`、`Binding::Producer` のいずれかである。この 3 つの腕は
        `here()` を返し、`origin` を呼ばない。よって言明は空虚に成り立つ。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>2. CASE `u` の束縛が `Binding::Move(y)`、`Binding::Join(arm_results)`、または
        `Binding::Payload(scrut, None)` である。これらの腕が呼ぶのは、順に `origin(y, σ)`、各アームに
        ついての `origin(arm_result, σ)`、`origin(scrut, σ)` であり、いずれも path を変えない。D17 の
        第 1 行は「`Binding::Move`、catch-all アームの payload、`Binding::Join`: `λ` を変えない」と述べ、
        D17 はその像を「着く leaf」と呼ぶので、`σ` は行き先の変数の型の boxed leaf である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, D17, A12
  <2>3. CASE `u` の束縛が `Binding::Field(container, idx)` で `container.ty.is_box` が偽、または
        `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box` が偽である。これらの腕が呼ぶのは
        `origin(container, [idx] ++ σ)`、`origin(scrut, [tag] ++ σ)` である。D17 の第 2 行は「unbox 容器の
        `Destructure` のフィールド、unbox union の変位アームの payload: `λ` の先頭に添字を足す」と述べ、
        D17 はその像を「着く leaf」と呼ぶので、`[idx] ++ σ` と `[tag] ++ σ` は行き先の変数の型の boxed
        leaf である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, D17, A12
  <2>4. CASE `u` の束縛が `Binding::Field(container, idx)` で `container.ty.is_box` が真、または
        `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box` が真である。この 2 つの腕は `here()` を
        返し、`origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>5. CASE `u` の束縛が `Binding::Llvm` で、`decl.leaf_origins_at(σ).and_then(as_arg_projection)` が
        `Some((j, p))` である。この腕は `origin(args[j], p)` を呼ぶ。`as_arg_projection` が `Some((j, p))`
        を返すのは `σ` の宣言が単一の `LeafOrigin::Arg(j, p)` であるときであり、A3 の表の「単一の
        `Arg(j, σ)`」の行 (その `σ` はここでの `p` に当たる) は、それが「第 `j` オペランドの leaf `σ`」を
        名指すと述べる。よって `p` は `boxed_leaf_paths(ty(args[j]))` の要素である。D17 の第 3 行も同じ
        ことを「`λ` を、`λ` 自身の宣言 `Arg(j, σ')` の `σ'` へ置き換える」と述べる。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection, A3, D17
  <2>6. CASE `u` の束縛が `Binding::Llvm` で、同じ式が `None` である。<1>2 よりこの腕の
        `origin_from_leaves_under` は `origin` を呼ばない。
    BY <1>2, CODE src/rc_ir/ownership.rs: origin_inner
  <2>7. QED
    `origin_inner` の `match` は `vars.bindings.get(var)` の値について、`None | Some(Binding::Param) |
    Some(Binding::Producer)`、`Move`、`Join`、`Llvm`、`Field`、`Payload` の 6 つの腕を持ち、`Field` と
    `Payload` はさらに `is_box` と変位で分かれる。<2>1 から <2>6 がこれを尽くす。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: Binding
<1>4. QED
  BY <1>1, <1>2, <1>3, D6, p13 の DEF ρ-歩みと ρ-終端
  D6 よりスロット `(x, λ)` の `λ` は `ty(x)` の inhabited な boxed leaf である。歩みはスロットから始まり、
  各段で `origin_inner` が呼ぶ `origin` の引数へ進むので (`p13` の `DEF ρ-歩みと ρ-終端`)、<1>1 と <1>3 を
  歩みの長さについて繰り返せば、各位置の path は boxed leaf である。<1>2 より、boxed leaf の位置では
  `origin_from_leaves_under` は `origin` を呼ばないので、その辺は歩みの上では取られない。

### L40a (スロットを含む類は開始している)

**言明** --- 実行路 `ρ` を辿る活性化の時点 `τ` と別名類 `C` について、`C` が、`τ` までに値を得た変数の
スロット (D6) を 1 つでも含むならば、`C` の開始の時点 (D34) は `τ` 以前である。とくに次の 2 つが
成り立つ。

1. `bumps_ρ(τ, C) ≥ 1` である計数下の類 `C` は `τ` で開始している。よって
   `Σ_{C : obj(C) = O} bumps_ρ(τ, C)` は `b(τ, O)` に等しい (DEF 類ごとの義務)。
2. D10 の各事象が名指す leaf のスロットが属する類は、その事象の時点で開始している。

**証明**

<1>1. `ρ` の上で、`origin` の辺の行き先の変数は、辺の元の変数より後に値を得ることがない。
  <2>1. `origin_inner` が `origin` を呼ぶとき、その引数の変数は、いま見ている変数 `u` の束縛が名指す
        変数である -- `Binding::Move(y)` の `y`、`Binding::Join(arm_results)` の各アームの結果、
        `Binding::Payload(scrut, ・)` の `scrut`、`Binding::Field(container, ・)` の `container`、
        `Binding::Llvm` の `args[j]` である。`collect_bindings` はこれらを `u` を束縛する節点から
        取る。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: Binding
  <2>2. `Binding::Join` 以外では、その変数は `u` を束縛する節点の位置でスコープに入っている (A11)。
        D2 のスコープの規則より、その束縛はその節点の祖先であるか、パラメータ・capture である。D3 より
        実行路は祖先を先に通り、パラメータ・capture は活性化が始まる時点で値を持つ (D23)。
    BY A11, D2, D3, D23, <2>1
  <2>3. `Binding::Join` では、D17 より辺はその活性化が選んだアームの結果へ辿る。D3 より `ρ` は
        `Match` 節点でそのアーム本体を辿ってから継続へ進むので、アームの結果の変数は `Match` の
        束縛変数より前に値を得ている。
    BY D17, D3, D21, <2>1
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>2. `C` が `τ` までに値を得た変数のスロット `(u, σ)` を含むならば、`C` の ρ-終端 `T_ρ(C)` の変数も
      `τ` までに値を得ている。よって D34 の「開始の時点」より `C` は `τ` で開始している。
  BY <1>1, p13 の DEF ρ-歩みと ρ-終端, D33, D34
  D33 より `C` のスロットはどれも `ρ` を辿って同じ終端に着く。ρ-終端は `(u, σ)` から始まる ρ-歩みの
  最後の位置であり、歩みの各段は `origin_inner` が呼ぶ `origin` の引数へ進む。<1>1 をその段数について
  繰り返す。
<1>2a. スロット `(v, λ)` (D6) について、`id(v, λ)` は `(v, λ)` から始まる ρ-歩み
      (`p13` の `DEF ρ-歩みと ρ-終端`) の上の位置である。とくに <1>1 より、その位置の変数は `v` より
      後に値を得ることがない。
  <2>0. ρ-歩みの残りの長さについての帰納法で示す。L40 より歩みの各位置 `(u, σ)` の `σ` は `ty(u)` の
        boxed leaf であり、歩みは `origin_inner` が `origin` を呼ばない位置で終わる。P2 より
        `origin(v, λ)` は停止するので、その再帰の深さは有限であり、歩みも有限である。よってこの
        帰納法は整礎である。
    BY L40, P2, p13 の DEF ρ-歩みと ρ-終端
  <2>1. CASE 位置 `(u, σ)` で `origin_inner` が `origin` を呼ばない。このとき `identity` は `(u, σ)` で
        ある。
    <3>1. `None`/`Param`/`Producer` の腕、`container.ty.is_box` が真の `Binding::Field` の腕、
          `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` の腕は `here() = Exactly((u, σ))` を
          返す。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. `Binding::Llvm` で `decl.leaf_origins_at(σ).and_then(as_arg_projection)` が `None` である腕は、
          `origin_from_leaves_under(...)` の値を返し、`None` のときは `here()` を返す。L40 の 1 より
          この呼び出しは `origin` を呼ばず、`reached` は `operand_units` の各元について `origin` を
          呼んで作られるので `operand_units` は空である。よって `reached` は空であるか
          (`produced_here` が真のとき) `[Exactly(here_identity)]` である。空のときは `first?` が `None`
          を返して `here()` に落ち、1 元のときは全要素が `first` に等しいのでその元を返す。どちらでも
          値は `Exactly((u, σ))` である。
      BY L40, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::identity
  <2>2. CASE 位置 `(u, σ)` の腕が、次の位置の `origin` の値をそのまま返す。すなわち `Binding::Move(y)`、
        `Binding::Payload(scrut, None)`、`container.ty.is_box` が偽の `Binding::Field`、
        `scrut.ty.is_box` が偽の `Binding::Payload(_, Some(tag))`、`as_arg_projection` が `Some((j, p))`
        を返す `Binding::Llvm` である。`identity` は次の位置のものであり、その位置は ρ-歩みの次の位置で
        あるから、帰納法の仮定が当たる。
    BY CODE src/rc_ir/ownership.rs: origin_inner, p13 の DEF ρ-歩みと ρ-終端, 帰納法の仮定
  <2>3. CASE 位置 `(u, σ)` の腕が `Binding::Join(arm_results)` である。この腕は
        `Origin::of_candidates(candidates, (u, σ))` を返す。ここで `candidates` は各アームの結果 `w` に
        ついての `origin(w, σ).acted_on()` の合併である。
    <3>1. `candidates` の元が 2 つ以上のとき、`of_candidates` は `Join { identity: (u, σ), .. }` を
          返すので `identity` は `(u, σ)` である。
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: Origin::identity
    <3>2. `candidates` がただ 1 つの元 `c` を持つとき、`of_candidates` は `Exactly(c)` を返す。`w` を
          その活性化が選んだアームの結果とすると、`origin(w, σ).acted_on()` は空でなく `candidates` に
          含まれるので `{c}` である。D15 より `acted_on()` は `identity()` を先頭に持つので
          `c = id(w, σ)` である。ρ-歩みは `Binding::Join` の位置から選んだアームの結果へ進む (D17、D21)
          ので、帰納法の仮定より `c` は `(w, σ)` から始まる歩みの上の位置であり、したがって `(u, σ)` から
          始まる歩みの上の位置である。
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, CODE src/rc_ir/ownership.rs: origin_inner,
         D15, D17, D21, p13 の DEF ρ-歩みと ρ-終端, 帰納法の仮定
    <3>3. QED
      BY <3>1, <3>2
  <2>4. QED
    `origin_inner` の `match` は `vars.bindings.get(var)` の値について `None | Param | Producer`、
    `Move`、`Join`、`Llvm`、`Field`、`Payload` の 6 つの腕を持ち、`Field` と `Payload` は `is_box` で、
    `Llvm` は `as_arg_projection` の値でさらに分かれる。<2>1、<2>2、<2>3 がこれを尽くす。歩みの最初の
    位置は `(v, λ)` なので、`id(v, λ)` はその歩みの上の位置である。「とくに」は <1>1 を歩みの段数に
    ついて繰り返したものである。
    BY <2>1, <2>2, <2>3, <1>1, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: Binding
<1>3. 1 が成り立つ。
  <2>1. `bumps_ρ(τ, C) ≥ 1` とする。DEF 類ごとの義務 より、`τ` に対応する走査の状態 (DEF 実行時の量) の
        `pending` のある要素 `p` と、`C` に属するある名前 `o` について `B_ρ(τ, p)[o] ≥ 1` である。
    BY DEF 類ごとの義務, DEF 実行時の量, D27
  <2>2. `p` の由来 (DEF 訪問) を `t = Retain(v, π)` とすると、`o = id(v, λ)` である `λ ∈ L(v, π)` が
        在る。その `λ` は `π` の下の inhabited (D16) かつ計数下 (D26) の boxed leaf である。
    BY D27, P16, 第 1 節の記法, D4, D16, D26
    D27 は、`p` が `pending` に入るときの `B_ρ` を、`π` の下の inhabited かつ計数下の各 leaf を
    `origin` の identity で名付けて数えたものと定め、以後の操作はその多重集合から引くか、値をそのまま
    運ぶだけである。よって `B_ρ(τ, p)` が 1 以上を与える名前はその数え上げに現れる名前であり、`L(v, π)`
    の元の `id` である。P16 の (a) より `p` の由来は `Retain` 節点である。
  <2>3. `α` が `τ` に実行している節点を `q` とすると、`t` は `ρ` の上で `q` より前にあるか `q` 自身で
        ある。
    BY L35, L34, DEF 実行時の量, DEF 訪問
    DEF 実行時の量 より、`τ` に対応する走査の状態は `q` の訪問の中の `pending` である。L34 の 1 より
    `q` の訪問が `pending` に施すのは `push`・`consume_objects`・`un_bump` であり、要素を加えるのは
    `push` だけで、その要素の由来は `q` 自身である (`q` が `Retain` のとき)。それ以外の要素は
    `pending(q)` の要素であり、L35 よりその由来は `ρ` の上で `q` より真に前にある。
  <2>4. `v` は `τ` までに値を得ている。
    BY <2>3, A11, D2, D3, D23
    `t` は `v` を名指す節点である。A11 より `v` の使用はその位置でスコープに入っている束縛に解決し、
    D2 のスコープの規則よりその束縛は `t` の祖先であるか、パラメータ・capture である。D3 より実行路は
    祖先を先に通り、パラメータ・capture は活性化が始まる時点で値を持つ (D23)。<2>3 より `t` の位置は
    `τ` 以前である。
  <2>5. `o = (u, σ)` は `τ` におけるスロット (D6) である。
    BY <1>1, <1>2a, <2>1, <2>2, <2>4, p13 の L11, 第 1 節の記法, D6
    `p13` の `L11` より `B_ρ(τ, p)[o] ≥ 1` である `o` は `ρ` で活性であり、第 1 節の記法 より活性な
    名前 `o = (u, σ)` は `ρ` の上のスロットである -- すなわち `σ` は `ty(u)` の inhabited な boxed leaf
    である。残るのは `u` が `τ` までに値を得ていることである。<2>2 より `o = id(v, λ)` であり、`λ` は
    `π` の下の inhabited な leaf なので `(v, λ)` はスロットである。<1>2a より `o` は `(v, λ)` から
    始まる ρ-歩みの上の位置であり、<1>1 よりその変数は `v` より後に値を得ることがない。<2>4 より `v` は
    `τ` までに値を得ている。
  <2>6. QED
    BY <1>2, <2>1, <2>5, P18b, DEF 類ごとの義務
    <2>1 より `o ∈ C` であり、<2>5 より `o` は `τ` におけるスロットなので、<1>2 より `C` は `τ` で
    開始している。P18b より `B_ρ` の個数は 0 以上なので `bumps_ρ(τ, ・)` も 0 以上であり、
    `Σ_{C : obj(C) = O} bumps_ρ(τ, C)` の 0 でない項はすべて `S(τ, O)` の類のものである。よってその和は
    `b(τ, O)` に等しい。
<1>4. 2 が成り立つ。
  BY <1>2, D10, D6
  D10 の各行が名指すのは inhabited な leaf であり、その leaf を持つ値の変数はその事象の時点で値を得て
  いる。D6 よりその対はスロットである。
<1>5. QED
  BY <1>2, <1>3, <1>4

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

### L41a (類の義務は bump 以上である)

**言明** --- 実行路 `ρ` を辿る活性化の、終端の `Ret` の消費より前の各時点 `τ` と、`held_ρ(τ, ・)` が
定まる各計数下の別名類 `C` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である (DEF 類ごとの義務)。

**証明**

<1>1. `bumps_ρ(τ, C) ≥ 0` である。
  BY DEF 類ごとの義務, D27, P18b
  `bumps_ρ(τ, C)` は `B_ρ` の個数の総和であり、P18b より各個数は 0 以上である。
<1>2. CASE `β(C) = 0` かつ `bumps_ρ(τ, C) = 0`。A19 の (ii-a) より `held_ρ(τ, C) ≥ 0` であり、
      DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 0 = bumps_ρ(τ, C)` である。
  BY A19, DEF 類ごとの義務
  (ii-a) は「各時点と各計数下の別名類について、その類が持つ参照の個数は非負であり」と述べる。
<1>3. CASE `β(C) = 0` かつ `bumps_ρ(τ, C) ≥ 1`。A19 の (ii-b) より
      `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であり、`obl_ρ(τ, C) = held_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  BY A19, DEF 類ごとの義務
  (ii-b) は「`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である」と述べる。
<1>4. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) ≥ 1`。A19 の (ii-b) より
      `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であり、`obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ bumps_ρ(τ, C)` で
      ある。
  BY A19, DEF 類ごとの義務
<1>5. CASE `β(C) = 1` かつ `bumps_ρ(τ, C) = 0`。P14a より `held_ρ(τ, C) ≥ 1` であり、
      `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ 0 = bumps_ρ(τ, C)` である。
  BY P14a, DEF 類ごとの義務, 第 1 節の記法, D34
  P14a は「`borrow_ify` の出力の各本体、各実行路、各活性化について、ρ-終端が借用する (D14) パラメータ・
  capture の leaf である**計数下**の別名類 (D26) は、活性化の間ずっと参照を少なくとも 1 つ持つ」と
  述べる。`B` は `borrow_ify` の出力の本体である (第 1 節)。`β(C) = 1` の計数下の類はその形であり、類が
  持つ参照の個数は `held_ρ` である (D34)。
<1>6. QED
  DEF 類ごとの義務 より `β(C)` は 0 か 1 であり、<1>1 より `bumps_ρ(τ, C)` は 0 か 1 以上である。
  <1>2 から <1>5 がこの 4 つの場合を尽くす。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

**終端の `Ret` の消費より前に限るのは、A19 の (ii-b) がその位置で偽だからである。** A19 は「延ばすのは
(ii-a) だけである。(ii-b) はこの位置で偽である」と述べ、反例を挙げる。

### L41b (類ごとの余りは 1 つ立つ)

**言明** --- 実行路 `ρ` を辿る活性化を固定する。終端の `Ret` の消費より前の各時点 `τ` と計数下の
オブジェクト `O` について、`b(τ, O) ≥ 1` ならば

`Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1] ≥ b(τ, O) + 1`

である。総和も角括弧も `S(τ, O)`、すなわち `obj(C) = O` であって `τ` までに開始した計数下の別名類を
渡る (DEF 類ごとの義務)。

**証明**

<1>1. PICK `C0 ∈ S(τ, O)` SUCH THAT `bumps_ρ(τ, C0) ≥ 1`。
  BY 本補題の仮定, DEF 類ごとの義務, D27, P18b
  `b(τ, O)` は `S(τ, O)` を渡る `bumps_ρ` の和であり、P18b より各項は 0 以上なので、和が
  1 以上ならば 1 以上の項が在る。
<1>2. `C0` 以外の各 `C ∈ S(τ, O)` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  BY L41a
<1>3. CASE `β(C0) = 0`。A19 の (ii-b) より `held_ρ(τ, C0) ≥ 1 + bumps_ρ(τ, C0)` であり、
      DEF 類ごとの義務 より `obl_ρ(τ, C0) = held_ρ(τ, C0) ≥ bumps_ρ(τ, C0) + 1` である。角括弧は 0 以上
      なので、<1>2 を残りの類について足すと言明の不等式になる。
  BY A19, DEF 類ごとの義務, <1>1, <1>2
<1>4. CASE `β(C0) = 1`。A19 の (ii-b) より `held_ρ(τ, C0) ≥ 1 + bumps_ρ(τ, C0)` であり、
      DEF 類ごとの義務 より `obl_ρ(τ, C0) = held_ρ(τ, C0) - 1 ≥ bumps_ρ(τ, C0)` である。`C0 ∈ S(τ, O)`
      は `β(C0) = 1` の類なので角括弧は 1 である。<1>2 を残りの類について足すと言明の不等式になる。
  BY A19, DEF 類ごとの義務, <1>1, <1>2
<1>5. QED
  DEF 類ごとの義務 より `β(C0)` は 0 か 1 なので、<1>3 と <1>4 が場合を尽くす。**`+1` が 1 回しか
  立たないのは、`C0` を `β` で分けたどちらの場合でもそれを出すのが 1 つだけだからである** -- `β(C0) = 0`
  では `C0` の `obl_ρ`、`β(C0) = 1` では角括弧である。
  BY <1>1, <1>2, <1>3, <1>4

### L42 (義務は pending の bump を覆う) --- これが P18c である

**言明** --- 実行路 `ρ` を辿る 1 回の活性化を固定する。終端の `Ret` の消費より前の、その活性化自身の
節点の時点 (DEF 実行時の量) `τ` と各計数下オブジェクト `O` について、`Obl(τ, O) ≥ b(τ, O)` である。
とくに `ρ` の上の節点 `q` の入口では `Obl(q, O) ≥ N(q, O)` である。

時点と走査の状態の対応は `DEF 実行時の量` が置く。

**証明**

<1>1. 計数下のオブジェクト `O` と各時点 `τ` について `Obl(τ, O) = Σ_{C ∈ S(τ, O)} obl_ρ(τ, C)` である。
  <2>1. D10 が `Obl` を変える事象は、初期値、`Retain`、`Release`、生成、消費の 5 種である。移動は `Obl` を
        変えない。D26 より、数えるのは計数下のオブジェクトへの参照だけである。
    BY D10, D26
  <2>2. これらの事象はいずれも 1 つの inhabited な leaf に紐づき、その leaf は `ρ` の上のスロット (D6)
        であって、ちょうど 1 つの別名類に属する。その類は事象の時点で開始している (L40a の 2) ので、
        `obj(C) = O` であるものは `S(τ, O)` に入る。
    BY D6, D10, D33, L40a, DEF 類ごとの義務
  <2>3. D34 の表の 6 行は、この 5 種の事象と次のように対応する。第 1 行が生成、
        第 2 行と第 3 行が初期値 (所有する場合と借用する場合)、第 4 行が `Retain`、第 5 行が `Release`、
        第 6 行が消費である。
    BY D34, D10
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
      BY CODE src/rc_ir/ownership.rs: origin_inner, L40, D10
      D10 の生成の `Llvm` の行が名指すのは結果の leaf なので、`λ` は結果を束縛する変数の型の boxed leaf
      である。L40 の 1 がこの呼び出しについて言明のとおりを述べる。
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
          与える (アームが 1 つ以上あるので `arm_results` は空でない)。`Binding::Llvm` の宣言が単一の
          `Arg` でない腕が ρ-終端であるのは、歩みの各位置の path が boxed leaf だからである -- L40 の
          1 より、boxed leaf の位置ではこの腕の `origin_from_leaves_under` は `origin` を呼ばない。
      BY CODE src/rc_ir/ownership.rs: origin_inner, A9, L40, <2>4, p13 の DEF ρ-歩みと ρ-終端
    <3>2. `None` の腕に当たるのは `vars.bindings` に束縛を持たない名前である。D6 より、その値はその記号の
          値であり、そこが指すのは funptr かグローバル状態のオブジェクトのどちらかであって、どちらも
          D8 の意味の参照を持たない。D34 は、束縛を持たない名前を ρ-終端とする類が計数下でないことを
          述べる。よってこの腕に当たる ρ-終端を持つ類は、計数下の `O` を指す類ではない。
      BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: VarTable::of,
         D6, D8, D34, A8, D26
    <3>3. QED
      BY <3>1, <3>2, <2>4, L40, D10, CODE src/rc_ir/ownership.rs: VarTable::of,
         CODE src/rc_ir/ownership.rs: collect_bindings
      <3>1 が挙げる 4 つの腕のうち、`None` の腕に当たる leaf は <3>2 が計数下でないとして除く。
      `Binding::Param` を置くのは `VarTable::of` がパラメータと capture について行う 1 か所だけなので、
      `Param` の腕に当たるのはパラメータ・capture の leaf である。残るのは `Binding::Producer` の腕と、
      宣言が単一の `Arg` でない `Binding::Llvm`、boxed 容器の `Binding::Field`、boxed scrutinee の
      `Binding::Payload(_, Some(_))` の 3 つの腕である。L40 より歩みの各位置の path は boxed leaf なので、
      この 4 つに当たる leaf は、<2>4 の <3>1 が挙げる D10 の生成の 5 行が名指す位置である --
      `collect_bindings` が `Binding::Producer` を置くのは `RcRhs::App` と `RcRhs::Closure` の結果、
      `Binding::Field` を置くのは `Destructure` の名前付きフィールド、
      `Binding::Payload(_, Some(tag))` を置くのは `Match` の変位アームの payload だけである。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, D10, D14, D34, DEF 類ごとの義務
    <2>3 の対応で、D10 の各事象は `held_ρ` と `Obl` に同じ増減を起こす。違うのは初期値の 2 行だけで
    ある。所有する (D14) パラメータ・capture の leaf では、D10 の初期値が参照を 1 つ入れ、`held_ρ` の
    第 2 行も 1 から始まるので、両者は等しい (`β = 0`)。借用する leaf では、D10 は参照を入れないのに
    `held_ρ` の第 3 行は 1 から始まるので、`Obl` の側は 1 少ない (`β = 1`)。生成の leaf では D10 が
    参照を 1 つ入れ、`held_ρ` の第 1 行も 1 から始まる (`β = 0`)。<2>5 より計数下の `O` を指す類の
    ρ-終端はこの 3 つのいずれかなので、`obl_ρ = held_ρ - β` の総和が `Obl` である。総和が `S(τ, O)` を
    渡るのは、<2>2 より `Obl` に参照を持つ類が `τ` までに開始しているからであり、D34 が `held_ρ` を
    定めるのもその範囲である。
<1>2. 各時点 `τ` と各 `C ∈ S(τ, O)` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  BY L41a
  本補題が量化するのは終端の `Ret` の消費より前の時点なので、L41a の範囲に入る。
<1>3. QED
  BY <1>1, <1>2, p13 の L17, L40a, DEF 類ごとの義務
  <1>1 と <1>2 より `Obl(τ, O) = Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) ≥ b(τ, O)` である。節点 `q` の入口では
  `p13` の `L17` より `Σ_{C : obj(C) = O} bumps_ρ(q, C) = N(q, O)` であり、L40a の 1 よりその和は
  `b(q, O)` に等しい。

### L43 (欠損は pending の bump の一部である)

**言明** --- 対応する活性化を固定する。`ρ` の上の**節点** `q` と計数下のオブジェクト `O` について

`d(q, O) = Σ_{t} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]`

である。ここで外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り、内側の和は `ρ` で
活性な名前 `o` を渡る。とくに `0 ≤ d(q, O) ≤ N(q, O)` である。`ρ` の終端の `Ret` の消費を行った後の位置を
`q_end` と書くと、`d(q_end, O) = 0` である。

等式と境界を節点について述べるのは、`DEF N` の `N(q, O)` と `DEF 訪問` の `pending(q)`・`e_t(q)` が節点に
ついての量だからである。`DEF 欠損` の `d` は位置についての量であり、節点を持たない `q_end` での値を
最後の文が与える。

さらに `d(q, O)` は README の P21 (a) の `k(O)` である。すなわち、`q` より前に実行された `Del` の
`Retain` 節点が `O` に作った参照のうち、その `Retain` と対になる `Del` の `Release` 節点がまだ処分して
いないものの個数である。

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
  BY <1>6, P18b, p13 の L11, DEF N
  P18b より `B_ρ(q, e)` は「その `Retain` が `ρ` で実際に作った参照のうち、まだ処分されていないもの」を
  数えた多重集合なので、どの名前についても個数は 0 以上である。`p13` の `L11` より、活性でない `o` に
  ついては `B_ρ(q, e)[o] = 0` である。よって <1>6 の和は 0 以上である。DEF N の `N(q, O)` は同じ内側の和を
  `pending(q)` の**すべての**要素について取ったものなので、`d(q, O) ≤ N(q, O)` である。
<1>7a. `d(q_end, O) = 0` である。
  BY <1>6, L38, L32, DEF 欠損
  L38 の 2 より、各 `t ∈ CT` が pending である区間は `ρ` の終端の `Ret` より真に前で終わる。よって終端の
  `Ret` の入口で pending な `CT` の要素は無く、<1>6 の等式の右辺の和は空で `d = 0` である。L32 の 5 より
  `Del` の要素は `Retain` 節点か `Release` 節点だけなので、終端の `Ret` は `Del` に入らず、その消費は
  DEF 欠損 の 2 つの数え上げのどちらも変えない。
<1>8. QED
  BY <1>2, <1>6, <1>7, <1>7a, L32, DEF 欠損
  等式は <1>6、境界は <1>7、`q_end` の値は <1>7a である。P21 (a) の `k(O)` が `d(q, O)` に等しいことは
  次から出る。
  DEF 欠損 より `d(q, O)` は、`q` より前に実行された `Del` の `Retain` 節点が `O` に作った参照の個数から、
  `q` より前に実行された `Del` の `Release` 節点が `O` の参照を処分した個数を引いたものである。L32 の 3 と
  <1>2 より、`Del` の各 `Release` 節点 `r` はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に属し、
  その `t` は `ρ` の上で `r` より前にある。すなわち `Del` の各 `Release` は `Del` のちょうど 1 つの
  `Retain` と対になる。よって引かれる個数は対になる `Retain` ごとに分けて数えられ、`d(q, O)` は「`Del` の
  各 `Retain` が `O` に作った参照のうち、対になる `Del` の `Release` がまだ処分していないもの」の総数で
  ある。これが P21 (a) の `k(O)` である。

### L43a (`Del` に入らない節点の実行の間、pending の bump は欠損を覆う)

**言明** --- `Del` に入らない `ρ` の上の節点 `q` と計数下のオブジェクト `O` について、位置 `q` から
`q` の節点の実行が終わるまでの各時点 `τ` について `b(τ, O) ≥ d(q, O)` である。

**証明**

<1>1. 位置 `q` について `b(q, O) ≥ d(q, O)` である。
  BY p13 の L17, L40a, DEF N, L43
  `p13` の `L17` は `N_ρ(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` を述べ、L40a の 1 よりその和は
  `b(q, O)` である。DEF N より `N(q, O) = N_ρ(q, O)` であり、L43 より `d(q, O) ≤ N(q, O)` である。
<1>2. `q` の節点の実行の各時点 `τ` に対応する走査の状態 (DEF 実行時の量) には、`q` で pending である
      `CT` の各要素が、`q` の入口での `B_ρ` のまま入っている。
  <2>1. CASE `q` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let`
        である。DEF 実行時の量 より、対応する走査の状態は `q` の訪問の途中または終わりの `pending` で
        ある。L34 の 1 より、`q` の訪問が `pending` に施す操作は `push`、`consume_objects`、
        `un_bump` の 3 つである。
    <3>1. `push` は末尾に新しい要素を足すだけで、既に在る要素の `node` と `outstanding` を変えない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
    <3>2. `consume_objects` はそれらの要素を取り除かず、その `B_ρ` も変えない。
      BY L38, L36, D27
      L38 の 4 が、`I_ρ(t)` に入る節点の訪問の中で `consume_objects` がそれらの要素を取り除かない
      ことを述べる。D27 は `consume_objects` が要素を取り除いたときの `B_ρ` を定めないだけで、取り
      除かない要素の `B_ρ` を変える操作を挙げていない。
    <3>3. `un_bump` はそれらの要素の `outstanding` を引かず、取り除きもしない。
      BY p30 の L5, L32, 本補題の仮定, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
      `un_bump` を呼ぶのは `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕だけなので、`un_bump`
      が走るとき `q` は `Release` 節点である。`p30` の `L5` より、`un_bump` が `pending` を変えるのは
      `InBracket` を返すときだけであり、そのとき変わるのは選ばれた 1 つの要素の `outstanding`
      (`subtract` の分) と、それが空になったときのその要素の除去だけである。返る `NodeId` はその要素の
      `node` である。それが `CT` の要素 `t` であるとすると、L32 の 4 より `q` は `un_bump_releases[t]`
      に入り、`t ∈ CT` なので L32 の 3 より `q ∈ Del` である。本補題の仮定に反するので、選ばれる要素は
      `CT` の要素の由来を持たない。
    <3>4. QED
      BY <3>1, <3>2, <3>3, D27, DEF 実行時の量
      D27 は、`un_bump` が `InBracket` でその要素を選ぶ `Release` の訪問でだけ `B_ρ` を引き、ほかの
      どの操作も `B_ρ` を変えないと定める。
  <2>2. CASE `q` の式が `Let(_, RcRhs::Match(_, arms), k)` であるか、`q` がアーム本体の終端の `Ret` で
        ある。この 2 つは D10 の行を 1 つも持たない (D9、D10) ので、DEF 実行時の量 より、この節点の
        実行のどの時点に対応する走査の状態も `pending(q)` である。`CT` の要素はそこに在る。
    BY L34, D9, D10, DEF 実行時の量, DEF 訪問
  <2>3. CASE `q` が `ρ` の終端の `Ret` である。L38 の 2 より `q` で pending な `CT` の要素は無いので、
        言明は空虚に成り立つ。
    BY L38
  <2>4. QED
    BY <2>1, <2>2, <2>3, L34, L33a, D3, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
    `RcExpr` の 6 変位のうち、`Ret` 以外は L34 の 1 か 2 の形であり <2>1 と <2>2 が扱う。`Ret` は、
    L33a と D3 より、アーム本体の終端 (<2>2) か `ρ` の終端 (<2>3) である。
<1>3. `q` が `ρ` の終端の `Ret` であるとき `d(q, O) = 0` であり、`b(τ, O) ≥ 0` である。
  BY L38, L43, D27, P18b
  L38 の 2 より `q` で pending な `CT` の要素は無いので、L43 の等式より `d(q, O) = 0` である。
  `b(τ, O)` は `B_ρ` の個数の総和 (DEF 類ごとの義務、D27) であり、P18b より各個数は 0 以上である。
<1>4. `B_ρ(τ, e)[o] ≥ 1` であって `o` が活性かつ `obj_ρ(o) = O` である各名前 `o` は、`ρ` の上の
      スロットであり `obj(C_ρ(o)) = O` である。その類は `S(τ, O)` に入る。
  BY p13 の L9, P5, p13 の DEF 名前の活性, D33, L40a, 第 1 節の記法, DEF 類ごとの義務
  第 1 節の記法 より、活性な名前 `o` は `ρ` の上のスロットであって `obj_ρ(o)` はそのスロットが指す
  オブジェクトである。D33 より `o` はちょうど 1 つの別名類 `C_ρ(o)` に属し、その類の `obj(C_ρ(o))` は
  類の各スロットが指すオブジェクト、すなわち `obj_ρ(o) = O` である。L40a より `C_ρ(o)` は `τ` で
  開始している。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, L43, D27, DEF 類ごとの義務, P18b
  `q` が `ρ` の終端の `Ret` である場合は <1>3 が言明を与える。そうでない場合、L43 より `d(q, O)` は、
  `q` で pending である `CT` の要素の `B_ρ(q, ・)` を、活性かつ `obj_ρ(o) = O` である名前 `o` について
  足したものである。<1>2 よりその要素と値は各時点に対応する `pending` にそのまま在り、<1>4 よりそれが
  数える名前は `S(τ, O)` の類の名前である。`b(τ, O)` は `pending` の**すべての**要素の `B_ρ` を
  `S(τ, O)` の類の名前について足したもの (DEF 類ごとの義務、D27) なので、前者の項はそのまま後者に
  現れる。残る項は P18b より 0 以上である。位置 `q` 自身については <1>1 が同じことを述べる。

### L41c (節点の実行の間も余りは残る)

**言明** --- 実行路 `ρ` を辿る活性化 `α` を固定する。`Del` に入らない `ρ` の上の節点 `q` と計数下の
オブジェクト `O` について `d(q, O) ≥ 1` とする。このとき、位置 `q` から `q` の節点の実行が終わるまでの
各時点 `τ` について `H(τ, O) ≥ d(q, O) + 1` である。

**証明**

<1>1. `q` は `ρ` の終端の `Ret` ではない。よって `q` の節点の実行の各時点は、終端の `Ret` の消費より
      前にある。
  BY L38, L43, 本補題の仮定, D3, L33a
  L38 の 2 より、各 `t ∈ CT` が pending である区間は `ρ` の終端の `Ret` より真に前で終わるので、終端の
  `Ret` で pending な `CT` の要素は無く、L43 の等式より `d = 0` である。これは本補題の仮定に反する。
  D3 と L33a より終端の `Ret` は `ρ` の最後の節点なので、`q` の節点の実行はその消費より前に終わる。
<1>2. `q` の節点の実行の各時点 `τ` について `b(τ, O) ≥ d(q, O) ≥ 1` である。
  BY L43a, 本補題の仮定
<1>3. `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1] ≥ d(q, O) + 1` である。
  BY L41b, <1>1, <1>2
<1>4. `H(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1]` である。
  BY A19, D23, D33, D34, DEF 類ごとの義務
  A19 の (i) は「各時点、各計数下オブジェクト `O`、その時点で生きている各活性化 `a` について、次が
  成り立つ。`a` の計数下の別名類のうち `obj(C) = O` であり開始の時点がその時点以前であるものの全体を
  `S` とし、各類について `d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置くと、
  `H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]`」と述べる。A19 は「各時点」がその活性化が
  生きている間のすべての時点であり、入れ子の呼び出しで中断中の時点を含むことを明記する。`α` は `τ` で
  生きている (D23 -- 始まって終わっていない)。A19 の `S` は `S(τ, O)`、A19 の `d(C)` は `obl_ρ(τ, C)`、
  角括弧は `β(C) = 1` の類が在るかである (DEF 類ごとの義務)。
<1>5. QED
  BY <1>3, <1>4

### L44 (2 つの実行の対応)

**言明** --- `cancel` の入力プログラム `P` が D12 を満たすとする。`ρ` を `B` の実行路、`ρ'` をそれに
対応する `B'` の実行路、`α'` を `ρ'` を辿る `B'` の活性化、`α` を D29 が `α'` に対応させる `B` の
活性化とする。`ρ` の上の各位置 `q` と各計数下オブジェクト `O` について次の 5 つが成り立つ。`ρ` の終端の
`Ret` の消費を行った後の位置についても同じである。

- **(a)** `α` と `α'` は、`q` までに、`Del` の節点を除いて同じ節点を実行し、その時点までに値を得ている
  各変数の値は、D29 の全単射のもとで対応する。
- **(b)** `H'(q, O) = H(q, O) - d(q, O)` である。
- **(c)** `Obl'(q, O) = Obl(q, O) - d(q, O)` である。
- **(d)** `d(q, O) ≥ 1` ならば `H(q, O) ≥ d(q, O) + 1` である。
- **(e)** `q` の節点の実行の間の各時点 `τ` -- 位置 `q` 自身と、その実行が終わった時点 `q'` を含む
  (DEF 実行時の量) -- について `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と `H'(τ, O) = 0` は
  同値である。`q ∉ Del` のときは `d(τ, O) = d(q, O)` である (DEF 欠損)。

**証明**

<1>1. (d) が成り立つ。
  <2>1. `q` が節点の入口であるとき。`d(q, O) ≥ 1` とする。L43 より `N(q, O) ≥ d(q, O) ≥ 1` である。
        L41 より `H(q, O) ≥ N(q, O) + 1 ≥ d(q, O) + 1` である。
    BY L41, L43
  <2>2. `q` が最後の位置 (終端の `Ret` の消費を行った後) であるとき、`d(q, O) = 0` なので (d) は空虚に
        成り立つ。
    BY L43
    L43 の最後の文が `d(q_end, O) = 0` を述べる。
  <2>3. QED
    BY <2>1, <2>2
<1>2. `Del` に入らない `ρ` の上の節点 `q` について、`q` の節点の実行の各時点 `τ` において、
      `d(q, O) ≥ 1` ならば `H(τ, O) ≥ d(q, O) + 1` である。
  BY L41c
<1>3. `cancel` の出力の各関数の `name`・`params`・`capture`・`borrowed_units` は、入力の同じ名前の関数の
      ものに等しい。とくに `B'` の所有と借用の割り当て (D14) は `B` のものと同じである。`cancel` は
      `prog.funcs.values()` の各 `f` について `f.clone()` を作って `clone.body` にだけ書き込み、鍵に
      `f.name.clone()` を据えるので、この 4 つは変わらない。グローバル初期化子はパラメータも capture も
      持たない (D1)。
  BY CODE src/rc_ir/borrow.rs: cancel, D14, D1
<1>4. (a)、(b)、(c)、(e) を `ρ` の上の位置についての帰納法で示す。D3 より `ρ` は有限の列なので整礎で
      ある。
  <2>1. 基底。`ρ` の最初の位置は `B` の根の入口である。DEF 欠損 より `d = 0` である。D29 より 2 つの
        活性化はパラメータと capture に対応する値を受け取るので、そこで値を得ている変数の値は対応する。
        D29 の第 2 行は「**活性化の最初の時点において、全単射で対応する各計数下オブジェクトの参照カウントは
        等しい。**」と述べるので、`H'` は `H` に等しい。D10 の初期値は所有する unit の下の inhabited な
        leaf で決まり、<1>3 より割り当ては同じなので `Obl'` は `Obl` に等しい。
    BY DEF 欠損, D29, D10, <1>3
  <2>2. 帰納法の仮定: `ρ` の上の位置 `q` について (a)、(b)、(c) が成り立つ。`q` の節点の実行の後の位置を
        `q'` とする。
    BY 帰納法の仮定
  <2>3. CASE `q` の節点が `Del` の `Retain` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Retain` の行により `O` への参照を `m(O)` 個作り、`H` と `Obl` をそれぞれ `m(O)` 増やす。
        DEF 欠損 より `d` も `m(O)` 増える。D9 の 2 つの表より `Retain` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。よって (a)、(b)、(c) が `q'` で成り立つ。
    <3>1. この節点は参照を処分しないので、D24 の (F) の解放は起きず、解放が作る活性化も参照も無い。
          `Retain` 節点は残る 3 つの子を作る段にも当たらない -- (E3) は `App` の節点、(E2) のうち
          オペランドを適用する `Llvm` の段は `Llvm` の節点であり、(E7) が起きるのは「まだ初期化されて
          いないグローバルを読む節点の位置」であって、D7 は `Retain` を読む構文に数えない。よって
          DEF 実行時の量 の 4 群のうち第 1 群・第 2 群・第 4 群は空であり、この節点の実行の時点は
          `q'` だけである。
      BY D10, D24, DEF 実行時の量, D7
    <3>2. QED
      BY <2>2, <1>1, <3>1, D10, D9, DEF 欠損, L30, L31, L32
      `q` は `Del` の `Retain` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
      しない。(e) が見る時点は <3>1 より `q` と `q'` の 2 つであり、DEF 欠損 より `d` の値はそれぞれ
      `d(q, O)` と `d(q', O) = d(q, O) + m(O)` である。位置 `q` では <2>2 の (b) が、位置 `q'` では本場合が
      示した (b) が、その値についての等式を与える。この 2 つはどちらも `ρ` の上の位置なので <1>1 の (d) が
      当たり、`d = 0` のときは両辺が等しく、`d ≥ 1` のときは `H ≥ d + 1 ≥ 2 > 0` かつ
      `H' = H - d ≥ 1 > 0` である。よって 0 になることは同値である。
  <2>4. CASE `q` の節点が `Del` の `Release` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Release` の行により `O` への参照を `c(O)` 個処分し、`H` と `Obl` をそれぞれ `c(O)` 減らす。
        DEF 欠損 より `d` も `c(O)` 減る。D9 の 2 つの表より `Release` は値を作らず、移さず、手放さない
        ので、変数の値は変わらない。
    <3>1. この節点の実行はどのオブジェクトも解放しない。
      <4>1. この節点が解放を起こすとすれば、`c(O0) ≥ 1` かつ `H(q, O0) - c(O0) = 0` である計数下
            オブジェクト `O0` が在る。
        BY D24, DEF 実行時の量, D7
        D24 の (F) より解放が起きるのは、段が参照を処分してカウントが 0 になったときである。`Release`
        節点が処分するのは D10 の `Release` の行の分だけなので、DEF 実行時の量 の第 2 群を適用した後の
        カウントは `H(q, O) - c(O)` であり、第 3 群は空である。最初に解放されるオブジェクトについて
        これが 0 であり、位置 `q` ではそうでない (`H(q, O0) ≥ 1`) ので `c(O0) ≥ 1` である。走査が
        起こす連鎖はその最初の解放から始まる。
      <4>2. `d(q, O0) = c(O0)` である。
        BY <4>1, DEF 欠損, L43, <2>2
        L43 より `d(q', O0) ≥ 0` であり、DEF 欠損 より `d(q', O0) = d(q, O0) - c(O0)` なので
        `d(q, O0) ≥ c(O0)`。一方 <2>2 の (b) より
        `H'(q, O0) = H(q, O0) - d(q, O0) = c(O0) - d(q, O0) ≥ 0` なので `d(q, O0) ≤ c(O0)`。
      <4>3. QED
        BY <4>1, <4>2, <1>1
        <4>1 より `d(q, O0) = c(O0) ≥ 1` なので、<1>1 の (d) より
        `H(q, O0) ≥ d(q, O0) + 1 = c(O0) + 1` である。これは <4>1 の `H(q, O0) = c(O0)` に反する。
    <3>2. QED
      `q` は `Del` の `Release` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
      しない。<3>1 より解放は起きないので、解放の走査も、それが作る活性化と参照も無い。`Release` 節点は
      残る 3 つの子を作る段にも当たらない -- (E3) は `App` の節点、(E2) のうちオペランドを適用する
      `Llvm` の段は `Llvm` の節点であり、
      (E7) が起きるのは「まだ初期化されていないグローバルを読む節点の位置」であって、D7 は `Release` を
      読む構文に数えない。よって DEF 実行時の量 の 4 群のうち第 1 群・第 3 群・第 4 群は空であり、この
      節点の実行の時点は `q'` だけである。`H` の変化は D10 の `Release` の行の分だけなので (a)、(b)、(c)
      が `q'` で成り立つ。
      (e) が見る時点は `q` と `q'` の 2 つであり、DEF 欠損 より `d` の値はそれぞれ `d(q, O)` と
      `d(q', O) = d(q, O) - c(O)` である。位置 `q` では <2>2 の (b) が、位置 `q'` では本場合が示した (b)
      が、その値についての等式を与える。この 2 つはどちらも `ρ` の上の位置なので <1>1 の (d) が当たり、
      `d = 0` のときは両辺が等しく、`d ≥ 1` のときは `H ≥ d + 1 ≥ 2 > 0` かつ `H' = H - d ≥ 1 > 0` で
      ある。よって 0 になることは同値である。
      BY <2>2, <1>1, D10, D9, D24, DEF 実行時の量, DEF 欠損, <3>1, L30, L31, L32
  <2>5. CASE `q` の節点が `Del` に入らない。`ρ'` の対応する位置にも同じ節点がある (L30、L31)。
    <3>1. 2 つの実行はこの節点について D10 の同じ行を同じ値に対して適用し、この節点が束縛する変数に
          対応する値を置く。さらに、この節点が子の活性化を作るとき、その子の事象が参照カウントに与える
          変化は 2 つの実行で同じである。`d` は変わらない。
      <4>1. `q` は `Del` に入らないので、L30 と L31 より `ρ'` の対応する位置の節点は `q` と同じ式の
            変位・変数・path・`RcState` を持つ。<2>2 の (a) より、その時点までに値を得ている各変数の値は
            2 つの実行で対応する。
        BY L30, L31, <2>2
      <4>1a. `q` の式が `Let(_, RcRhs::Match(v, arms), k)` であるとき、2 つの活性化は同じアームへ進む。
        BY D21, D29, <4>1
        D21 は `Match` のアームを `v` の値の実行時のタグで決める。D29 の第 5 行は「**スカラの成分に
        ついては、対応は等号である。** boxed leaf でない成分 -- unbox union のタグ、整数、浮動小数、
        funptr の番地 -- は、対応する 2 つの値で等しい。D21 は `Match` のアームを `v` の値の実行時の
        タグで決めるので、この節が、対応する 2 つの活性化が同じアームへ進むことを与える」と述べる。
        <4>1 より `v` の値は 2 つの実行で対応する。
      <4>2. この節点が束縛する値は 2 つの実行で対応する。
        <5>1. CASE その値が、節点の形と、それが名指す変数の値と、その値から到達できる (D25) オブジェクト
              が保持する値だけで決まる。D2 の節点の表と D9 の移動の表と D10 の生成の表がその値を定める。
              <4>1 より名指す変数の値は 2 つの実行で対応し、D29 の第 5 行より対応するオブジェクトが
              保持する値も対応するので、この値も 2 つの実行で対応する。オブジェクトの中から読み出される
              3 つ -- boxed 容器の `Destructure` のフィールド、boxed union の変位アームの payload、単一の
              `Unknown` を宣言する `Llvm` の結果の leaf (A3 より、そのオブジェクトはオペランドの leaf が
              指すオブジェクトから到達できるか、グローバル値が到達する) -- がこの場合に入る。
          BY D2, D9, D10, A3, A4, A5, D25, D29, <4>1
        <5>2. CASE それ以外である。すなわちこの構文はオペランドから結果が決まらない構文である。D21 は
              「その割り当ては、オペランドから結果が決まらない構文の結果を含む」と述べ、D29 はその各位置
              での結果を 2 つの活性化に同じものとして与える。
          BY D21, D29, <4>1
        <5>3. QED
          BY <5>1, <5>2
          値は、<5>1 の 3 つ (節点の形、名指す変数の値、そこから到達できるオブジェクトが保持する値) で
          決まるか、決まらないかのどちらかである。決まらないものは、A3 の但し書きが挙げる
          `InlineLLVMBoxedFromRetainedPtrIOS` のように到達できる元を持たない `Unknown` を含めて、
          オペランドから結果が決まらない構文であり、<5>2 が扱う。
      <4>3. この節点が子の活性化を作るとき、その子の事象が参照カウントに与える変化は 2 つの実行で同じ
            である。
        BY D24, D21, D29, <4>1
        D24 の「活性化の林」より、子の活性化を作る段は 4 つである。(E3) の `App`、(E7) のグローバルの
        初期化、(E2) のうちオペランドを適用する `Llvm` の段、そして (F) の解放が `Destructor` に
        ついて作る段である。D21 の第 4 行は「**子の活性化を作る段** (D24 の「活性化の林」)。`App` の段
        (E3)、オペランドを適用する `Llvm` の段 (E2)、グローバルの初期化の段 (E7)、そして `Destructor` の
        オブジェクトを解放する段 (F) である。返る値と、参照カウントに与える変化は、子の本体が決める。
        1 つの本体だけを見ている間、これは外から与えられる量である。**(F) はどの構文でも起こりうる** --
        参照を処分する段はどれもオブジェクトの解放を起こしうるので、`Release` の節点も、消費を行う
        `App` や `Destructure` の節点も、この意味で外から与えられる量を持つ」と述べ、この 4 つを 1 つの
        行で扱う。D29 の第 4 行はそのデータを対応する 2 つの活性化に同じものとして与える。
      <4>3a. `App` の節点について、D9 の `App` の行が名指す leaf は 2 つの実行で同じである。
        BY <4>1, <1>3, D9, D14, D23, D29
        D9 の `App` の行は、callee の全 boxed leaf と、**呼び出し先がその位置の unit を所有する (D14)**
        引数の leaf を名指す。D23 より、その呼び出し先はその段で `callee` の値が指す実行時の関数で
        ある。<4>1 より `callee` の値は 2 つの実行で対応し、D29 の第 5 行はスカラの成分 -- funptr の
        番地を含む -- が対応する 2 つの値で等しいと述べるので、2 つの実行の呼び出し先は同じ名前の関数
        である。<1>3 よりその関数の `params` と `borrowed_units` は 2 つのプログラムで等しいので、
        D14 の所有も等しい。
      <4>4. QED
        BY <4>1, <4>1a, <4>2, <4>3, <4>3a, D9, D10, D16, A4, DEF 欠損,
           CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
        言明の前半は <4>2 と <4>1a であり、後半は <4>3 である。D10 の行が名指す leaf は、節点の形と値
        (D16、A4) と、`App` については呼び出し先の所有 (<4>3a) で決まるので、2 つの実行で同じである。
        `q` は `Del` に入らないので DEF 欠損 の 2 つの数え上げは変わらない。
    <3>2. `q` の節点の実行の間の各時点 `τ` について `H'(τ, O) = H(τ, O) - d(q, O)` であり、
          `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。位置 `q` 自身と、その実行が終わった時点 `q'` に
          ついても同じである。本場合の仮定より `q` は `Del` に入らないので、DEF 欠損 より
          `d(τ, O) = d(q, O)` であり、これは (e) である。
      <4>1. 位置 `q` 自身について、`H'(q, O) = H(q, O) - d(q, O)` であり、`H(q, O) = 0` と
            `H'(q, O) = 0` は同値である。
        BY <2>2, <1>1
        <2>2 の (b) が等式を与える。`d(q, O) = 0` のとき両辺は等しい。`d(q, O) ≥ 1` のとき <1>1 の (d)
        より `H(q, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、`H'(q, O) ≥ 1 > 0` なので、どちらも 0 でない。
      <4>2. DEF 実行時の量 より、この節点の実行の時点は 4 群であり、この順に並ぶ。子の活性化の事象、
            D10 の処分、D10 の作成、解放の走査である。子の活性化を作らない節点では第 1 群が空であり、
            解放が起きない節点では第 4 群が空である。本場合の仮定より `q` は `Del` に入らないので、
            DEF 欠損 よりこの実行のどの時点でも `d(τ, O) = d(q, O)` である。
        BY DEF 実行時の量, D10, D24, DEF 欠損, 本場合の仮定
      <4>2a. `d(q, O) ≥ 1` であるとき、この節点の実行の各時点 `τ` について `H(τ, O) ≥ d(q, O) + 1` で
             ある。とくに `H(τ, O) > 0` である。
        BY <1>2, 本場合の仮定
        本場合の仮定は `q` が `Del` に入らないことであり、<1>2 はその形の節点について、その実行の
        各時点でこれを述べる。
      <4>3. 第 1 群の各時点 `τ` について、`H'(τ, O) = H(τ, O) - d(q, O)` である。
        BY <3>1, <4>1, <4>2
        `τ` までの `H` の変化は、子の活性化の事象が与えるものだけである。<3>1 の後半よりそれは 2 つの
        実行で同じなので、<4>1 と合わせて差は `d(q, O)` である。
      <4>4. 第 2 群・第 3 群・第 4 群の各時点 `τ` について、`H'(τ, O) = H(τ, O) - d(q, O)` である。
        <5>1. 事象の個数についての帰納法で示す。基底は第 1 群の終わりの時点であり、<4>3 (第 1 群が空の
              節点では <4>1) が差を `d(q, O)` と与える。
          BY <4>1, <4>3
        <5>2. 第 2 群と第 3 群の事象は 2 つの実行で `H` を同じだけ変える。
          BY <3>1, D10
        <5>3. 差が `d(q, O)` である時点で、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。
          <6>1. CASE `d(q, O) = 0`。2 つのカウントは等しい。
            BY <5>1
          <6>2. CASE `d(q, O) ≥ 1`。<4>2a より `H(τ, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、
                `H'(τ, O) = H(τ, O) - d(q, O) ≥ 1 > 0` である。どちらも 0 でない。
            BY <4>2a, <5>1
          <6>3. QED
            BY <6>1, <6>2
        <5>4. 第 4 群の事象は 2 つの実行で対応し、`H` を同じだけ変える。
          BY <5>3, <3>1, D24, D25, D21, D29, A5
          D24 の (F) より、解放されるのはカウントが 0 になった計数下オブジェクトであり、<5>3 より
          その集合は 2 つの実行で等しい。解放が処分するのは、そのオブジェクトが保持する参照である
          (D24 の (F)、D25)。A5 より、その参照は inhabited であって計数下のオブジェクトを指す各 boxed
          leaf に 1 つずつ在り、D29 の第 5 行より対応するオブジェクトが保持する値は対応するので、
          処分される参照は対応する。**`#ArrayStorage a` のオブジェクトは A5 の例外であり、持ち手の単位は
          leaf ではなく要素の位置である** -- その時点の `size` 個の各位置が、要素の型の
          `boxed_leaf_paths` が列挙する leaf を 1 組ずつ持ち、解放の走査はそれを 1 つずつ処分する (A5、
          `CODE src/object.rs: ObjectFieldType::loop_over_array_buf`)。`size` が 2 つの実行で等しいことは
          D29 の第 5 行の「スカラの成分については、対応は等号である」が与え、各要素の位置の leaf が対応
          するオブジェクトを指すことは同じ行の「対応する各位置において、inhabited な各 boxed leaf が
          全単射で対応するオブジェクトを指し」が与える。よってこの場合も処分される参照は対応する。
          `Destructor` のオブジェクトの解放が作る retain は
          `_dtor` の欄の値に当たり、その欄の値も D29 の第 5 行より対応する。その適用が作る活性化の事象が
          2 つの実行で同じであることは <3>1 の後半が述べる -- D21 の第 4 行が (F) の作る活性化を
          活性化の側のデータに数え、D29 の第 4 行がそれを 2 つの活性化に同じものとして与える。
        <5>5. QED
          BY <5>1, <5>2, <5>3, <5>4, D24
          <5>2 と <5>4 より各事象は差を保ち、<5>1 の帰納法が進む。D24 の (F) より解放の連鎖は有限で
          終わるので、事象の個数は有限である。
      <4>5. QED
        BY <4>1, <4>2, <4>2a, <4>3, <4>4
        <4>2 の 4 群の時点を、<4>3 が第 1 群について、<4>4 が第 2 群・第 3 群・第 4 群について尽くし、
        位置 `q` 自身は <4>1 が扱う。0 になることが同値であるのは、`d(q, O) = 0` のときは 2 つの
        カウントが等しく、`d(q, O) ≥ 1` のときは <4>2a より `H(τ, O) ≥ 2` かつ
        `H'(τ, O) = H(τ, O) - d(q, O) ≥ 1` だからである。
    <3>2a. `q` の節点の実行の第 4 群 -- 解放の走査 -- が `Obl` に与える変化は、2 つの実行で同じである。
      <4>1. 第 4 群で解放されるオブジェクトの集合と、それが解放される時点は、2 つの実行で対応する。
        BY <3>2, D24, D26, D29, DEF 実行時の量
        DEF 実行時の量 の第 4 群は、第 2 群と第 3 群を適用した後にカウントが 0 であり位置 `q` では
        そうでなかった計数下オブジェクトについて D24 の (F) の解放が走る、と定める。<3>2 より各時点で
        `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。D29 の第 5 行より、計数下かグローバル状態か
        (D26) の区別も対応する 2 つのオブジェクトで一致する。
      <4>2. 解放が処分する参照は `Obl` の元ではないので、その処分は `Obl` を変えない。
        BY D24, D25, P28
        D24 の (F) は解放を「`o` が持つ参照 (D25) をすべて処分し、それから `o` の記憶域を返すこと」と
        定める。その参照の持ち手は D25 の 2 つ目 -- 生きているオブジェクト `o` -- であり、P28 より
        1 つの参照の持ち手はちょうど 1 つなので、その参照はどの活性化の `Obl` にも属さない。
      <4>3. 解放されるオブジェクト `o` が `Std::FFI::Destructor` のものであるとき、その解放が `Obl` に
            与える変化は 2 つの実行で同じである。
        BY D24, D21, D29, D9, D16, A5
        D24 の (F) より、この解放は `_dtor` の欄の関数に retain を立てて `Obl` に参照を入れ、その関数を
        `_value` の欄の値へ適用し (D9 の `App` の行の消費が `Obl` からその参照を出す)、返った `IO` の
        動作の runner を適用し、返りの参照を `Obl` へ受け取って `o` の `_value` の欄へ書き込む。この列の
        各点で `Obl` に入る参照と `Obl` を出る参照は、A5 より、そこで動く値の inhabited (D16) であって
        計数下のオブジェクトを指す各 boxed leaf に 1 つずつである。動く値は、`o` の `_dtor` と `_value` の
        欄の値と、2 つの活性化が返す値である。D29 の第 5 行より対応する `o` が保持する値は対応し、
        どの leaf が inhabited であるかを決めるタグも等しい。D21 の第 4 行と D29 の第 4 行より、2 つの
        活性化の結果は対応する 2 つの活性化に同じものとして与えられる。よって各点で動く参照は 2 つの
        実行で対応する。
      <4>4. QED
        BY <4>1, <4>2, <4>3, D24
        D24 の (F) より第 4 群で起きるのは解放の連鎖であり、その各解放は <4>2 の処分と、`o` が
        `Destructor` のオブジェクトであるときの <4>3 の動作からなる。<4>1 よりその連鎖は 2 つの実行で
        対応するので、`Obl` に与える変化は 2 つの実行で同じである。
    <3>3. QED
      `Obl` を動かすのは、D10 の行 (DEF 実行時の量 の第 2 群と第 3 群) と、第 4 群の解放だけである。
      第 1 群は子の活性化自身の事象であって `Obl(α)` を動かさない -- D24 の (E3) の消費と (E4) の
      受け取り、および (E2) のうちオペランドを適用する `Llvm` の段が渡す参照と受け取る参照は、いずれも
      `α` 自身の節点について D10 の行が定めるものであり、第 2 群と第 3 群に在る。(E7) のグローバルの
      初期化が返す参照の行き先は `E` であり、グローバルを読む節点は参照を得ない (D24 の (E7)、A8、
      D26)。前者が 2 つの実行で同じであることは <3>1 が、第 4 群については <3>2a が述べる。<3>1 と <3>2 より `H` の変化も 2 つの実行で同じで
      あり、束縛する変数には対応する値が置かれる。`d` は変わらないので (a)、(b)、(c) が `q'` で成り立ち、
      (e) は <3>2 である。
      BY <3>1, <3>2, <3>2a, <2>2, D10, D24, A8, D26, DEF 実行時の量
  <2>6. QED
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかである (L32 の 5)。
    節点を持たない最後の位置については、L43 の最後の文より `d = 0` なので (b) から `H' = H` であり、
    (e) の 2 つの言明がその位置について成り立つ。
    BY <2>1, <2>3, <2>4, <2>5, <1>1, L43, L32
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
          D9 の `App` の行が読む所有は実行時の呼び出し先 (D23) のものなので、静的に解決した関数が
          その呼び出し先であることが要る。`cancel` は `borrow_ify` の出力を入力に取るので (第 1 節)、
          P29 の (b) がそれを与える -- README は「`cancel` の中で `CancelAnalysis::consume_rhs` が
          `rhs_consumes` を呼ぶ位置がこれを読む」と、この位置を名指す。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: resolve_callee_params, D9, D23, A7, A14, P29, <3>1
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
  `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` が `O` に作った参照のうち、**その `Retain`
  と対になる削除済みの `Release` がまだ処分していないもの**の個数である。
- **(b)** 2 つの活性化は同じ位置で同じオブジェクトを解放する。とくに各読む構文の各位置で解放されて
  いるオブジェクトの集合は等しい。

**証明する形**。`cancel` の入力プログラム `P` が D12 を満たすとする。`ρ'` を `B'` の実行路、`α'` を
それを辿る `B'` の活性化、`ρ` を `ρ'` に対応する `B` の実行路、`α` を D29 が `α'` に対応させる `B` の
活性化とする。`k(O)` を `Retain` の個数ではなく参照の個数で数えることが要る理由は、第 11 節が反例で
述べる。

**証明**

<1>1. (a) が成り立つ。
  BY L44, L43
  L43 の最後の文より P21 (a) の `k(O)` は DEF 欠損 の `d(q, O)` であり、L44 の (b) がその等式である。
<1>2. 計数下のオブジェクトについて、2 つの活性化は同じ位置で同じオブジェクトを解放する。ここで `O` は
      D29 の全単射が対応させるオブジェクトを渡り、D29 の第 5 行よりその定義域は 2 つの活性化がそれぞれ
      到達できる (D25) オブジェクトの全体なので、どちらかの活性化が解放しうる計数下オブジェクトはすべて
      `O` の範囲に入る。
  <2>1. `ρ` の上の位置 `q` の節点の実行の間に `α` が `O` を解放するのは、その実行の間のある時点 `τ` で
        `H(τ, O) = 0` となり、位置 `q` ではそうでないときである。`α'` についても `H'` で同じである。
    BY D7, DEF 解放されている, DEF 実行時の量, D29, D25
  <2>2. QED
    BY <2>1, L44
    L44 の (e) より、`q` の節点の実行の間の各時点 -- 位置 `q` 自身を含む -- で `H(τ, O) = 0` と
    `H'(τ, O) = 0` は同値である。よって <2>1 の条件は 2 つの活性化で同時に成り立つ。
<1>3. グローバル状態のオブジェクト (D26) は 2 つの活性化のどちらでも解放されない。
  BY D26, A8
<1>4. (b) が成り立つ。
  BY <1>2, <1>3, DEF 解放されている, L44, L30, L31, D29, D26
  DEF 解放されている より、`α` の時点 `τ` で `O` が解放されているとは、`H(τ0, O) = 0` を満たす `τ` 以前の
  時点 `τ0` が在ることであり、`α'` については `H'` で同じである。L44 の (e) をその各 `τ0` に当てると、
  この 2 つは同値である。計数下でないオブジェクトについては <1>3 が両方とも解放されないことを与える。
  D29 の第 5 行より計数下かグローバル状態か (D26) の区別は対応する 2 つのオブジェクトの間で一致するので、
  この場合分けは 2 つの活性化で同じに分かれる。
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

**言明 (README)** --- **`borrow_ify` の出力**を入力とすると、`cancel` の出力も D12 の意味で RC 規律を
満たす。出力の各活性化に対応する入力の活性化が在ることは D29 が与える。

README はこの限定の理由を続けて述べる。「入力を `borrow_ify` の出力に限るのは、この命題の証明が読む
2 つがそこにしか無いからである。A19 (ii-b) の範囲は「`borrow_ify` の入力の各本体と、`borrow_ify` が
それを写した各本体」であり、P14a の範囲は「`borrow_ify` の出力の各本体」である。D12 を満たすだけの
プログラムにはどちらも付いてこない。」

**証明する形**。入力プログラムを `P`、出力を `P'` と書く。**`P` は `borrow_ify` の出力であり、D12 を
満たすとする。**この 2 つが要る所は次のとおりである。`borrow_ify` の出力であることは `<1>5` が読む
`L42` に要る -- `L42` が読む `L41a` が P14a と A19 (ii-b) を読み、どちらも範囲が
`borrow_ify` の出力の本体だからである (第 1 節)。D12 を満たすことは `<1>3` が読む。D11 は `B'` の
すべての実行路についての述語なので、示すのは、`B'` の各実行路 `ρ'` と、それを辿る各活性化 `α'` に
ついて (S-a)、(S-b)、(S-c) が成り立つことである。

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
  <2>1a. CASE その操作が `ρ` の終端の `Ret` の消費である。L38 の 2 より終端の `Ret` で pending な `CT` の
         要素は無いので、L43 の等式より `d(q, O) = 0` であり、<1>4 より `Obl'(q, O) = Obl(q, O)` である。
         <1>3 の (S-a) が `Obl(q, O) ≥ c` を与える。
    BY L38, L43, <1>3, <1>4
  <2>2. CASE それ以外。その操作が位置 `q` の節点で `O` への参照を `c` 個取り除くとする。DEF 実行時の量 の
        第 2 群がその処分をまとめて適用し、その直後の時点 `τ` について `Obl(τ, O) = Obl(q, O) - c` で
        ある。この場合 `q` は `ρ` の終端の `Ret` ではなく、D3 と L33a より終端の `Ret` は `ρ` の最後の
        節点なので、`τ` は終端の `Ret` の消費より前の節点の時点である。よって L42 より
        `Obl(τ, O) ≥ b(τ, O)` である。
    BY L42, DEF 実行時の量, D10, D3, L33a
  <2>3. その `τ` について `b(τ, O) ≥ d(q, O)` である。
    BY L43a, <2>1
    <2>1 より `q` は `Del` に入らないので L43a が使え、`τ` は `q` の節点の実行の間の時点である。
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3, <1>4
    <2>1a 以外の操作について、<2>2 と <2>3 より `Obl(q, O) - c = Obl(τ, O) ≥ d(q, O)` なので
    `Obl'(q, O) = Obl(q, O) - d(q, O) ≥ c` である。すなわち取り除かれる参照は `Obl'` に入っている。
    (S-a) が見る操作の位置は D9 の消費の表の 6 行と `Release` 節点であり、そのうち終端の `Ret` を
    <2>1a が、残りを <2>2 と <2>3 が扱う。
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
        あり、各位置で名指す変数と path は同じである。L44 の (a) よりその位置の変数の値も対応し、
        D29 の第 5 行より対応する値の inhabited な各 boxed leaf は全単射で対応するオブジェクトを指す
        ので、読みうるオブジェクトと触れるオブジェクトは 2 つの実行で対応する。
    BY L30, L31, L32, L44, D7, A5, D29
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

**言明 (README)** --- `borrow_ify` と `cancel` は、D12 が見ない部分について次を満たす。

- `roots` を変えない。
- **出力の各関数は入力のちょうど 1 つの関数から作られ**、その `fn_ty` / `ret_ty` / `params` の型 /
  `inline_into_callers` は元の関数のものに等しい。
- 出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の第 `i` 要素の
  ものに等しい。`owns_initializer` と `owns_storage` には `true` を書き、D1 が述べる呼び出し順により
  この書き込みは正しい値を書く。
- **本体について書き換えが変えるのは、`Retain`/`Release` の節点と、`App` の callee の名前だけである。**
  節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の op とオペランド・
  `Destructure` のフィールドは、いずれも元の本体のものに等しい (複製の名前替えを P9 で戻したうえで)。

**証明**

<1>1. `roots` は変わらない。`borrow_ify` の返す `RcProgram` の `roots` は `prog.roots.clone()` であり、
      `cancel` の返す `RcProgram` の `roots` も `prog.roots.clone()` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel
<1>2. `cancel` の出力の各関数は、入力の関数の `clone()` に `body` だけを書き込んだものである。よって
      `fn_ty`、`ret_ty`、`params`、`inline_into_callers` は変わらない。
  BY CODE src/rc_ir/borrow.rs: cancel
<1>3. `borrow_ify` の出力の各関数は入力のちょうど 1 つの関数から作られ、その `fn_ty`、`ret_ty`、
      `params` の型、`inline_into_callers` は元の関数のものと等しい。
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
    BY <2>1, <2>2, CODE src/rc_ir/borrow.rs: borrow_ify
    `borrow_ify` が出力の `funcs` に入れるのは、入力の各関数について作る元の版 <2>1 と、借用版を作る
    関数について作る借用版 <2>2 だけであり、そのどちらも 1 つの入力の関数から作られる。
<1>3a. `borrow_ify` の出力の各版の本体について、書き換えが変えるのは `Retain`/`Release` の節点と、`App` の
      callee の名前だけである。節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の
      op とオペランド・`Destructure` のフィールドは、`RewriteCtx` が読む本体のものに等しい。借用版に
      ついてその本体は `clone_func` が作る複製であり、P9 がそれを元の本体の名前替えとして述べる。
  <2>1. `RewriteCtx::rewrite(node)` は `grow_stack(|| self.rewrite_inner(node))` であり、A15 より
        `rewrite_inner` をちょうど 1 回呼んでその値を返す。
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, A15
  <2>2. 木 `N(node)` の構造についての帰納法で示す。DEF 子と親 より子は真の部分木なので整礎である。
    <3>1. CASE `node` の式が `RcExpr::Let(x, RcRhs::App(callee, args), k)` である。この腕は `x` と
          `args.clone()` を据えた `Let` の節点を `&node.source` を付けて作り、その継続を
          `prepend_rc(after, true, self.rewrite(k))` に、callee を `self.route(x, callee, args, k)` の
          値にする。さらに `prepend_rc(before, false, ・)` でその節点を包む。P12 より `route` が返すのは
          元の呼び出し先と同じ関数の版であり、変わりうるのはその名前だけである。P11 より `call_rc` が
          置くのは `Retain` と `Release` だけであり、`prepend_rc` はその各要素を `rc_node` で節点にする。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: prepend_rc,
         CODE src/rc_ir/borrow.rs: rc_node, P11, P12
    <3>2. CASE `node` の式が `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` である。この腕は `x`、
          `scrut`、各アームの `tag`/`payload`/`payload_state`、アームの本数と並びを変えず、各アーム本体を
          `self.rewrite(&arm.body)` に、継続を `self.rewrite(k)` に置き換えて `&node.source` を付ける。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/ast.rs: MatchArm::with_body
    <3>3. CASE `node` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::App` でも `RcRhs::Match` でも
          ない。この腕は `x` と `rhs` を `clone()` し、継続を `self.rewrite(k)` に置き換えて
          `&node.source` を付ける。`match` の腕はこの順に並んでいるので、この腕に落ちる `rhs` は `App`
          でも `Match` でもない。すなわち `Llvm` の op とオペランド、`Closure`、`Var` は元のままである。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/ast.rs: RcRhs
    <3>4. CASE `node` の式が `RcExpr::Retain(v, path, state, k)` または
          `RcExpr::Release(v, path, state, k)` である。この 2 つの腕は `rewrite_rc` を呼ぶ。`rewrite_rc`
          は継続を `self.rewrite(k)` に置き換え、借用版でない版ではその継続を `rc_node` の節点 1 つで
          包み、借用版では `owns_unit` が真である unit ごとの節点の列で包む (P10)。`rc_node` が作るのは
          `Retain` か `Release` である。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, CODE src/rc_ir/borrow.rs: rc_node, P10
    <3>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)`、`RcExpr::Eval(v, k)`、
          または `RcExpr::Ret(v)` である。この 3 つの腕は、`container`/`fields`/`state`、`v` を
          `clone()` し、継続を `self.rewrite(k)` に置き換えて `&node.source` を付ける (`Ret` は継続を
          持たない)。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
    <3>6. QED
      BY <3>1, <3>2, <3>3, <3>4, <3>5, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/rc_ir/borrow.rs: expr_node, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
      `RcExpr` の 6 変位のうち `Let` を右辺で 3 つに分けた 8 つの場合を <3>1 から <3>5 が尽くす。これは
      `rewrite_inner` の `match` の 8 つの腕である。どの腕も `expr_node` で節点を作って `&node.source` を
      据え、継続には `self.rewrite` の値を置くので、節点の並びは元の並びである。節点を足すのは <3>1 と
      <3>4 だけであり、足す節点はいずれも `Retain` か `Release` である。節点を落とすのは <3>4 の借用版
      だけであり、落とす節点は `Retain` か `Release` である。ほかのどの腕も、式の変位と、`Let` の束縛
      変数、`Match` のアームの構成、`Llvm` の op とオペランド、`Destructure` のフィールドを `clone()` で
      運ぶ。
  <2>3. QED
    BY <2>1, <2>2
<1>3b. `cancel` の出力の各本体について、書き換えが変えるのは `Retain`/`Release` の節点だけである。
  BY L30, L32
  L30 が、`drop_nodes` が `NodeId` の集合に入る `Retain`/`Release` 節点だけを取り除き、残る各位置の式の
  変位・変数・path・`RcState`・source・`Match` のアームの本数と並び・継続の順序を変えないことを述べる。
  L32 の 5 より `Del` の要素はすべて `Retain` 節点か `Release` 節点である。
<1>4. グローバル初期化子について、`borrow_ify` と `cancel` はどちらも入力の `globals` を写して、
      第 `i` 要素について `symbol: g.symbol.clone()`、`ty: g.ty.clone()`、`owns_initializer: true`、
      `owns_storage: true` を持つ `RcGlobalInit` を作る。写像なので列の長さは変わらない。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel
<1>5. `true` が正しい値である。`build_object_files` は `optimize_rc_program` を呼び、その返り値を
      `divide_into_units` と `divide_among_units` に渡す。すなわち `borrow_ify` と `cancel` が走るのは
      分割の前であり、そのときプログラムは 1 つで、その 1 つがすべての初期化子と記憶域を持つ。
  BY D1, CODE src/build/build_object_files.rs: build_object_files,
     CODE src/build/build_object_files.rs: optimize_rc_program
<1>6. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>3b, <1>4, <1>5
  第 1 の箇条は <1>1、第 2 の箇条は <1>2 と <1>3、第 3 の箇条は <1>4 と <1>5、第 4 の箇条は <1>3a
  (`borrow_ify`) と <1>3b (`cancel`) である。

## 11. P21 (a) の `k(O)` を参照ごとに数える理由

README の P21 (a) は `k(O)` を、削除される `Retain` が `O` に作った参照のうち対になる削除される
`Release` がまだ処分していないものの**個数**として書く。`Retain` の個数で数える形が偽であることを、
本体で示す。**この文書に閉じられなかった点は無い。**

**`Retain` の個数で数える形は 2 通りに読め、どちらの読みでも (a) が偽になる。** 1 つの削除される
`Retain` `t` と対になる削除される `Release` は 1 つとは限らない。`L38` の 3 が `R_ρ(t)` と書く集合で
あり、`Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` を満たす。`|R_ρ(t)| ≥ 2` であって、その一部だけが
実行済みである位置で、`Retain` を数える形は `t` を数えるとも数えないとも読める。

**その形の本体は書ける。** `v` をパラメータとし、その型を、変位を 1 つだけ持ち、その payload が boxed な
値を 2 つ持つ unbox 構造体である unbox union とする。D5 よりこの union は 1 つの unit `[]` なので、A2 の
下で `Retain(v, [])` が書ける。D4 の第 5 規則は変位の payload の下へ降りるので
`boxed_leaf_paths(ty(v)) = {[0, 0], [0, 1]}` であり、`v` はパラメータなので `origin` はこの 2 つの leaf に
相異なる名前 `(v, [0, 0])`、`(v, [0, 1])` を与える。すなわち `ActRefs(t)` はこの 2 つの名前を 1 つずつ
数える。本体を次の形に取る。

```
Retain(v, [], s,
  Let(m, Match(v, [変位 0 のアーム (payload `p`):
        Destructure(p, [(0, f0), (1, f1)], s,
          Release(f0, [], s,
            Release(f1, [], s,
              Let(z, Llvm(zero, []), Ret(z)))))]),
    ...))
```

`origin` は `f0` を `Binding::Field` (unbox 容器) と `Binding::Payload(v, Some(0))` (unbox union) の
2 つの辺を経て `(v, [0, 0])` へ、`f1` を同じ 2 つの辺を経て `(v, [0, 1])` へ辿るので、
`ActRefs(Release(f0, []))` と `ActRefs(Release(f1, []))` はその名前を 1 つずつ数える。よって最初の
`Release` の `un_bump` は `InBracket(t)` を返して `outstanding` から第 1 の名前を引き (残りは空でない)、
2 つめの `Release` の `un_bump` が第 2 の名前を引いて空にする。`R_ρ(t)` は 2 元であり、`t` を含めた
3 つが `Del` に入る。この間に消費は無い -- `Match` の節点は消費せず、名前の付いた 2 つのフィールドだけを
取り出す unbox 容器の `Destructure` も消費しない (D9)。アームが 1 つなので、`merge` は `t` を
`needed_retains` に入れない (P18 の第 3 の主張)。

変位が 1 つなので 2 つの leaf はどちらも常に inhabited (D16) であり、`t` は 2 つのオブジェクト `O0`、
`O1` へ 1 つずつ参照を作る (`O0 ≠ O1` である活性化を取る)。位置 `q` を `Release(f0, [])` の後、
`Release(f1, [])` の前に取ると、`H_α(O0) - H_{α'}(O0) = 0` (処分済み)、`H_α(O1) - H_{α'}(O1) = 1`
(未処分) である。

- **「対になる削除済みの `Release` を 1 つも実行していない `t` を数える」と読む場合。** `t` は
  `Release(f0, [])` を実行済みなので数から落ち、`k(O1) = 0` になる。差は 1 なので **(a) が偽である。**
- **「対になる削除済みの `Release` を全部は実行していない `t` を数え、その `t` が `O` に作った参照を
  全部数える」と読む場合。** `k(O0) = 1` になる。差は 0 なので **(a) が偽である。**

**README の P21 (a) は参照ごとに数える形である。**

> `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` が `O` に作った参照のうち、その `Retain`
> と対になる削除済みの `Release` がまだ処分していないものの個数である。

この形では、上の本体の位置 `q` で `k(O0) = 0`、`k(O1) = 1` であり、どちらも差に一致する。

**この形は証明が出す量である。** `L43` の <1>8 が、`DEF 欠損` の `d(q, O)` -- 削除される `Retain` が
作った個数から削除される `Release` が処分した個数を引いたもの -- がこの形の量に等しいことを示す。
`L44` の (b) が `H'(q, O) = H(q, O) - d(q, O)` を与えるので、P21 (a) はそのまま出る。

**対になることは `L32` の 3 が与える。** `Del` の各 `Release` はちょうど 1 つの `t ∈ CT` の
`un_bump_releases[t]` に属するので、「その `Retain` と対になる `Release`」は 1 つに定まる。
