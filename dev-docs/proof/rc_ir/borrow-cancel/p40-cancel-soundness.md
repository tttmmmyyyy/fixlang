# P18c, P19 - P24: `cancel` が RC 規律を保存すること

この文書は README の 7 命題 P18c, P19, P20, P21, P22, P23, P24 を証明する。README の定義 D1 - D34
(D11a と D11 の (S-c) の接頭条件を含む)、仮定 A1 - A26、層 2 の命題 P30、および
命題 P1 - P18b と P14a の**言明**の上に立つ。主定理 T は `p70-main-theorem.md` の担当であり、この文書は
扱わない。

この文書が読んだコードのコミットは `d9b200c8cf12fdf02790d19b5f6c8c4ed9562617` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で `src/` に入った変更は、
README の第 1 節が挙げる 2 種である -- 「**対象コミットより後に `src/` へ入った変更は 2 種である。**
1 つは各証明が引く記号に付く `// PROOF:` コメントで、`dev-docs/proof/proof_links.py` が生成する。
もう 1 つは `Validator::check_rhs` が `result_prov` の宣言する source の個数を数える検査であり、A3 の
「1 つの結果 leaf に 2 つ以上の source を宣言しない」を果たす者を与える。どちらも
`borrow_ify`・`cancel`・`ownership.rs` の振る舞いを変えない。」

引用する外部の補題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF bump の帰属` (`B_ρ`)、`DEF ρ-歩みと ρ-終端`、`DEF N` (`N_ρ`) -- と、補題 `L7`
(boxed leaf の路は反鎖をなす)、`L9` (`identity` は inhabited を決める)、`L10a` (静的な数え上げと実行時の
作用が活性な名前で一致する)、`L11` (非活性な名前では `B` は空)、`L17`
(`N` は別名類ごとの `bumps` の和である)。
これらは `p30 の L10`、`p13 の L17` のようにファイル名を添えて引用する。

**外部の結果**は `EXT <名前>` の名札で引く。この文書が引くのは Rust の言語規則と標準ライブラリ、および
`dyn_clone` crate の 4 つで、その完全な言明は次のとおりである。

- **EXT Arc の割り当ての安定性** --- `std::sync::Arc<T>` の値は、制御ブロックと `T` を 1 つのヒープ
  割り当ての中に置く。`Arc::as_ref`(`Deref`) が返す `&T` の番地はその割り当ての中の `T` の番地であり、
  同じ割り当てを指す `Arc` から何度取っても等しい。`Arc` の値を move してもその番地は動かない。割り当てが
  解放されるのは最後の強参照が落ちたときであり、共有参照が生きている間は落ちない。
- **EXT Vec::clone** --- `Vec<T>` (`T: Clone`) の `clone` は、長さが原本と等しく、第 `i` 要素が原本の
  第 `i` 要素の `clone` である新しい `Vec` を返す。要素の並びは原本のものである。
- **EXT derive(Clone)** --- 構造体または列挙型に `#[derive(Clone)]` を付けて得られる `clone` は、原本と
  同じ変位の値であって、その各欄が原本の対応する欄の `Clone::clone` である値を返す。欄を落とすことも、
  並べ替えることも、別の値を置くこともない。
- **EXT dyn_clone の trait object の複製** --- `dyn_clone::clone_trait_object!(Tr)` は
  `impl Clone for Box<dyn Tr>` を与える。その `clone` は中身の具体型の `Clone::clone` を呼び、その値を
  新しい `Box` に置いて返す。**複製が原本と同じオブジェクトであるとは限らない。**

`B_ρ` の個数が 0 以上であることは README の P18b が言う -- P18b は `B(p, ρ)` を「`p.node` の `Retain` が
`ρ` で実際に作った参照のうち、`ρ` 上でまだ処分されていないものを、それを作った leaf の `origin` の
identity で名付けた多重集合」と定める。**`outstanding` に
ついて使うのは `covers`、すなわち `outstanding[o] ≥ B_ρ(・, ・)[o]` の向きだけである。**

別名類 (`obj(C)`、`T_ρ(C)`) は **D33**、類ごとの参照 `held_ρ` は **D34** であり、どちらも `README.md` に在る。

この文書が導入する補題は `L30` から番号を付ける。`p30` と `p13` の補題の番号と衝突させないためである。
補題は依存の順に並ぶので、`L43a` を読む `L41c` はその後に在る。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P18c | 証明済み (`L42`)。A19、P14a、`p13` の `L17` を読む。入力は `borrow_ify` の出力に限る |
| P19 | 証明済み |
| P20 | 証明済み |
| P21 | 証明済み。(a) は `L43` と `L44` の (b) に、(b) は `L44` の (e) に載る。`α` が D21 の制限を満たすことは `L44` の (f) が示す |
| P22 | 証明済み |
| P23 | 証明済み。(S-a) は `L42` に、(S-b) は `L44` の (c) に、(S-c) は `L44` の (e) に載る。(S-c) は D11a の接頭条件つきで示す |
| P24 | 証明済み。第 5 の箇条は `rewrite_inner` の 8 腕についての構造帰納で出る |

**各段が依拠するのは、冒頭が挙げる README の定義・仮定・命題と、そこに据えた 4 つの外部の結果 (`EXT`)
と、引用したコードだけである。** 局所の仮説はどの命題にも残っていない。**P18c・P21・P23 は入力を
`borrow_ify` の出力に限る** -- `L41a` が読む A19 (ii-b) と P14a の範囲がそこだからであり、第 4 節・
第 7 節・第 9 節の「証明する形」がその限定を書き出す。**README の P18c の言明はこの限定を持たないので、
そこへ差し戻す。**

**点はこの文書が定めるものではない。** 点は D24 の段内の点であり (`DEF 実行時の量`)、この文書は
`DEF 節点の入口の点` でその上に節点ごとの記法を置くだけである。

**`α` の参照カウントの推移は、D29 の第 2 行が `α'` のそれに欠損 `k` を足したものとして与える。**
`L44` の (b) と (e) の等式はその与件を読んだものであり、`L44` が示すのは 2 つの活性化の点が対応する
ことの方である。対応するオブジェクトが保持する値が対応することは D29 の第 5 行と A4 が、子の活性化を
作る段が参照カウントに与える変化を活性化の側のデータとすることは D21 の第 4 行が与える。

**A19 (i) は仮定ではなく、D21 が活性化に課す制限である。** `α'` は `B'` の活性化なのでそれを満たし、
`α` がそれを満たすことは `L44` の (f) が点ごとに示す。移し替えを行うのは `L43b` と `L43c` であり、
`α` についてこの制限を読むのは `L41`・`L41c`・`L41d`・`L42` である。

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

**この 3 つの値は、走査のどの段階で読んでも同じである。** `CancelAnalysis::acted_references` は
`acted_references(self.vars, self.type_env, v, path)` を、`CancelAnalysis::other_objects` は
`boxed_leaf_paths(&v.ty, self.type_env)` の各 leaf についての `origin(self.vars, self.type_env, v.name, leaf)`
を呼ぶだけである (`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`,
`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。D15 より `acted_references` の値はその引数と
`origin` の答えで決まる。`CancelAnalysis` の `vars` と `type_env` の欄は、`cancel` が据えた 1 つの
`VarTable` の値と 1 つの `TypeEnv` の値への共有参照であり、走査はその欄を差し替えない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis`, `CODE src/rc_ir/borrow.rs: cancel`)。よって走査の全体が、
**1 つの `VarTable` の値と 1 つの `TypeEnv` の値**を第 1・第 2 引数として `origin` を呼ぶ。P2a はその形の
主張であり、鍵 `(x, π)` が等しい 2 つの呼び出しの答えが等しいこと -- すなわち答えが `vars.origins` が
保持する memo の状態に依らないこと -- を与える。

**根拠を「共有参照だから値が動かない」に置くことはできない。** `VarTable` は `origins` を
`RefCell<Map<VarPath, Origin>>` で持ち、`origin` は共有参照から
`vars.origins.borrow_mut().insert(key, answer.clone())` を実行する
(`CODE src/rc_ir/ownership.rs: VarTable`, `CODE src/rc_ir/ownership.rs: origin`)。README の A3 は
「**`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変えない。** 到達できる型が
内部可変性を持つ欄を持つときは、その欄は**一度だけ書かれる memo であって、その値はその型の
`PartialEq` が読む成分の関数である**」と述べ、続けて「**「内部可変性を持たない」と書くと偽になる。**」と
書く。`type_env` から到達する `TypeNode` の `OnceLock` の欄についてはその節が、`vars.origins` については
P2a が答える。`origin` が `vars` から読む残りの欄 `bindings` は、`VarTable::of` と `VarTable::body_only` が
`collect_bindings` で作った後 書き込まれない -- `VarTable` を `&mut` で受け取るのはこの 3 つと
`collect_bindings` だけである (`CODE src/rc_ir/ownership.rs: VarTable::of`,
`CODE src/rc_ir/ownership.rs: VarTable::body_only`, `CODE src/rc_ir/ownership.rs: collect_bindings`)。

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
対応が全単射であることを示す。以下、`ρ` と `ρ'` は、対応する節点で並べて比べる。

### DEF 実行時の量

**点は D24 の段内の点である。この文書は点集合を定義しない。** D24 は「**段内の点は、この 6 種の列の
切れ目である。** 段の素動作を時間の順に並べたときの、最初の素動作の前の点と各素動作の後の点を、その段の
**段内の点**と呼ぶ。段と段のあいだの時点はその段の最初の段内の点であり、直前の段の最後の段内の点である。
(F) の解放がその段の中で始めた活性化の木の節点が行う動作も、この列の元である。」と定め、続けて
「**この定義は 1 か所にしかない** -- P28・D11・D30 がこの粒度で量化するので、証明ファイルが自分の
第 1 節で同じものを定義し直すと、そのファイルは別の点集合の上で命題を示すことになる。」と書く。以下、
**点**と書くのはこの段内の点であり、`τ`、`p` で表す。**時点**も同じものを指す。

6 種の素動作は D24 が挙げる「参照の受け渡し、生成、割り当て、処分、解放、グローバル化」である。切れ目を
除くのは 2 つだけであり、D24 は「**切れ目を除くのは、上の 2 つの節だけである。** 処分とそれが起こす解放の
あいだ、および素動作とそれに付随する書き込みのあいだには点が無い。**それ以外の切れ目を束ねてはならない。**」
と書き、「とくに**割り当てと、割り当てたオブジェクトの欄を埋める受け渡しのあいだには点が在る。**」と
続ける。**この文書はこの 2 つ以外の切れ目を束ねない。**

**活性化 `α` の点**とは、`α` が生きている (D23) 間の各点である。`α` が入れ子の呼び出しで中断中の間の点も
`α` の点である -- D23 は「**生きている活性化**とは、始まって終わっていない活性化である」と定め、A19 は
「**「各時点」は、その活性化が生きている (D23) 間のすべての時点であり、入れ子の呼び出しで中断中の時点を
含む。**」と述べる。

`α` の各点 `τ` とオブジェクト `O` について、`H(τ, O)` をその点の `O` の参照カウント (D7)、`Obl(τ, O)` を
その点の義務集合 (D10) が持つ `O` への参照の個数とする。**`H` は活性化の側のデータである** -- D21 は
「活性化はさらに、各時点の各計数下オブジェクトの参照カウント `H` を持つ。」と述べ、続けて「`H` も活性化の
側のデータである。」と書き、その増減のうち別の制御の流れの段から来る分はこの活性化の外から来ると述べる。
`B'` の活性化 `α'` については `H'`、`Obl'` と書く。

**D21 の制限は段内の点の粒度で読む。** D21 の「**活性化は、その各時点で A19 (i) の不等式を満たすものに
限る。**」の「各時点」は段内の点である。根拠は 3 つで、どれも README に在る。D21 自身が `H` を「各時点の
各計数下オブジェクトの参照カウント」として活性化の側のデータに置くこと、D34 が「**段内の点における
`held`。** P28 (b) は各時点と各段内の点 (D24) について A19 (i) の不等式を要求するので、`held` は段内の点
でも定まっていなければならない。」と述べて `held` を段内の点で定めること、そして P28 (b) が実行の作る
活性化についてその粒度で不等式を示すことである。

### DEF 節点の入口の点

`ρ` の上の節点 `q` について、`q` の実行が始まる直前の点を `q` の**入口の点**と呼び、そこでの量を
`H(q, O)`、`Obl(q, O)`、`d(q, O)`、`N(q, O)` と書く。`q` の実行が終わった点を `q'` と書く。`q` の実行の
終わりと、`ρ` の上で `q` の次にある節点の実行の始まりとのあいだに素動作は無いので、`q'` はその次の節点の
入口の点である。`q` が `ρ` の終端の `Ret` であるときは、`q'` を `ρ` の**最後の点** `q_end` と呼ぶ -- その
`Ret` の消費 (D9) を行った直後の点である。D3 より `ρ` は有限の列なので、入口の点は有限個である。

**`q` の実行の点**とは、`q` の入口の点から `q'` までの各点をいう (両端を含む)。

**この文書は時刻を「位置」と呼ばない。** 時刻は上の「点」であり、`ρ` の上の節点はその入口の点で代表
する。**「位置」と書くのは D6 の意味 -- スロットと記号の位置 -- のときだけである。** 本体の木の場所に
ついては D2 が「**この文書では、本体の木の位置を「節点」と呼び、位置が相異なれば節点も相異なるものと
する。**」と定めるので、この文書も「節点」と書く。P15 の言明を引くところだけは、P15 の語である「位置」を
そのまま写す。

### DEF 節点の実行の素動作

節点 `q` の実行が持つ素動作 (D24) は次で尽きる。**この一覧は D24 の段の記述の網羅から出る** -- D24 は
「**段の記述は `Obl` について網羅である。** (E1)-(E8) と (F) は、各段について `Obl` を離れる参照の
行き先と、作られる参照の持ち手と、`H` の動きを全部書いている。**すなわち、ここに挙がっていない動きは
起きない。**」と述べる。

- **D10 の行が定める作成と処分**。`q` について D10 の生成の表・消費 (D9)・`Retain` の行・`Release` の行が
  定めるものであり、D24 の (E2)・(E3)・(E4) がその行き先と `H` の動きを書く。**1 つの行が複数の leaf を
  名指すときは、leaf ごとに 1 つの素動作である** -- D34 は第 4 行を「`Retain(v, π)` であって
  `(v, λ) ∈ C` である `λ` を `π` の下に持つ」に対し「その `λ` 1 つにつき +1」と書き、その事象を
  「**それを運ぶ素動作の直後の段内の点**」で `held` に映すので、素動作は leaf ごとである。
- **割り当て**。D10 の生成の表のうち `Closure(f, caps)` の capture object と、`result_prov` が単一の
  `Fresh` を宣言する `Llvm` の結果の leaf は、新しいオブジェクトの割り当てを伴う。D24 の (E2) の `H` の
  表より、割り当てた直後のカウントは 1 であり、その 1 つの参照の持ち手は `Obl(α)` である (D10 の生成)。
  すなわちこの素動作は第 1 の箇条の生成でもある。**割り当てたオブジェクトの欄を埋める受け渡しは別の
  素動作であり、そのあいだに点が在る** -- D24 は「とくに**割り当てと、割り当てたオブジェクトの欄を
  埋める受け渡しのあいだには点が在る。**」と述べ、「**割り当てられた直後のオブジェクトの持ち手の単位は、
  参照を 1 つも持たない。**」と続ける。
- **子の活性化の素動作**。D24 の (E3)、(E7)、および (E2) のうちオペランドを適用する `Llvm` の段が作る
  活性化と、その子孫の活性化の素動作である。`α` はその間 中断中であり、`Obl(α)` は動かない (L39b)。
- **(F) の解放**。ある処分が計数下オブジェクトのカウントを 0 にしたとき、D24 の (F) の解放がその処分に
  付随して走る -- D24 は「**解放は、それを起こした処分に付随して起きる。** 処分とその処分が起こす解放の
  あいだに段内の点は挟まらない。」と述べる。解放は `o` が持つ参照を処分し、`o` が `Std::FFI::Destructor`
  のオブジェクトであるときは**活性化を 2 つ作る** -- D24 の「活性化の林」は「**(F) の解放が作る活性化は
  2 つである** -- `_dtor` の欄の関数を `_value` の欄の値へ適用するものと、それが返した `IO` の動作の
  runner を適用するものであり、2 つ目の入力は 1 つ目の結果である」と述べる。その活性化の木の節点が行う
  動作も段内の点の列の元であり、D24 は「**この節が縛るのは、解放の始まりである。**」に続けて「**解放の
  内側には段内の点が在る。**」と書く。D24 の (F) より、解放の連鎖はオブジェクトの個数で抑えられるので
  有限で終わる。**この連鎖の間の `Obl` の動きは L39a が述べる。**
- **グローバル化**。(E5) の段が行うもので、`q` の実行の中では (E7) が作る初期化子の活性化が終わる
  ところにだけ現れる。`Obl(α)` を動かさない (L39b)。

**素動作の順序をこの文書は定めない。** D24 は各段の素動作を「時間の順に並べたとき」の列として扱うだけで、
1 つの節点の中で処分と作成のどちらが先かを定めない。以下の議論は順序を使わず、素動作の列の上の帰納で
進む。

**解放の内側の点。** ある処分から、それに付随して走る (F) の解放の連鎖が終わるまでのあいだの点を、
**解放の内側の点**と呼ぶ。D24 は「**この節が縛るのは、解放の始まりである。**」に続けて「**解放の内側には
段内の点が在る。**」と書くので、この点は在りうる。`Std::FFI::Destructor` でないオブジェクトの解放は
活性化を作らないので、その解放の内側に点は無い。

**走査の状態との対応。** `q` の実行の各点には、`q` の訪問の中でそれまでに施された操作 (`push`、
`consume_objects`、`un_bump`) の後の `pending` が対応する。D10 の行が定める 1 つの作成・処分の素動作の
直後の点には、走査がその事象に対応する操作を行った直後の `pending` が対応する (A19、D27)。子の活性化の
素動作・(F) の解放・グローバル化には走査の操作が対応しない -- その間の点に対応するのは、その節点の訪問の
中でそれまでに施された操作の後の `pending` である。

### DEF 対応する活性化の量

対応する活性化 `α`、`α'` を固定したとき、`H'(τ, O)`、`Obl'(τ, O)` は、`α'` について `τ` に対応する点で
読んだ値とする。`q ∈ Del` のときは `ρ'` にその節点が無いので、`q` の実行の各点に対応する `α'` の点は、
`ρ'` の上で `q` の直後にあたる節点の入口の点である。**`H` と `H'` を 1 つの `O` について並べて書くとき、
2 つの活性化のオブジェクトを結ぶのは D29 の全単射である。** `O` はその全単射が対応させるオブジェクトを
渡る。D29 の第 5 行より、その定義域は 2 つの活性化がそれぞれ到達できる (D25) オブジェクトの全体であり、
どちらかの活性化が触れうる計数下オブジェクトはすべてここに入る。

**2 つの活性化の点の対応。** 2 つの活性化は同じ点で始まり、対応する節点の実行が起こす素動作を順に
突き合わせて、点を 1 対 1 に対応させる。`Del` の節点の実行が作る点は `α` の側にしかないので、`α'` の側の
値として上の約束の点の値を取る。**この対応が `α` のどの点についても定まることは `L44` が示す** -- (a) が
2 つの活性化が `Del` の節点を除いて同じ節点を実行することを、(e) が 1 つの節点の実行の中で 2 つが同じ
素動作の列を持つことを与える。

**その対応の上で `H` と `H'` の関係を与えるのは D29 の第 2 行である。** D29 は「`α` の参照カウントの
推移は、`α'` のそれに欠損 `k` (P21 (a)) を足したものとして与える」と述べ、その `k` は `DEF 欠損` の `d`
である (L43)。すなわち `L44` の (b) と (e) の等式は、証明が素動作ごとに積み上げる量ではなく、D29 が
`α` に与えるデータである。

### DEF 解放されている

オブジェクト `O` が活性化のある時点 `τ` で**解放されている**とは、`τ` かそれより前の時点で `O` の参照
カウントが 0 であること、すなわち `H(τ0, O) = 0` を満たす `τ` 以前の時点 `τ0` が在ることをいう。D7 は
参照カウントが 0 になったオブジェクトが解放されると定めるので、これは D7 の「解放される」を時点の言葉で
書いたものである。`α'` についても `H'` で同じように定める。

**この語を使うのは、`O` を割り当てた素動作 (D24) より後の時点についてだけである。** それより前の時点で
`O` は生きていない (D25)。環境が持ち込んだオブジェクトは実行の最初の時点から生きている (D25)。

**これは D11a が読む「解放されている」と同じものを指す。** D11a は「解放されていない (D24 の (F))」の形
で書くので、2 つが同じであることを述べる者が要る。

- D24 の (F) が `O` を解放したならば、それを起こした処分がカウントを 0 にしているので、その処分の直後の
  時点で `H = 0` である。(F) は「ある段が参照を処分して計数下のオブジェクト (D26) `o` の `H(o)` が 0 に
  なったとき、`o` は**その同じ段の中で解放される**」と述べる。
- 逆に `H(τ0, O) = 0` である時点 `τ0` が在るならば、`O` は (F) の解放を経ている。割り当ての直後の
  カウントは 1 であり (D24 の (E2) の `H` の表の「単一の `Fresh` ならこの段が新しく割り当てる
  オブジェクトで `H` = 1」と `Closure(f, caps)` の行)、環境が持ち込んだオブジェクトは実行の最初の時点で
  持ち手を少なくとも 1 つ持つ (A17 の (i-c) -- 「**その時点に生きている各計数下オブジェクトは、少なくとも
  1 つの持ち手を持つ**」)。カウントを下げるのは参照の処分だけである -- D24 は「**段の記述は `Obl` に
  ついて網羅である。** (E1)-(E8) と (F) は、各段について `Obl` を離れる参照の行き先と、作られる参照の
  持ち手と、`H` の動きを全部書いている。**すなわち、ここに挙がっていない動きは起きない。**」と述べ、
  そこで `H` を下げる行は処分だけである。よって `H` が 0 になった最初の時点はある処分の直後であり、(F) は
  その処分に付随して `O` を解放する。

読む者は第 9 節の `<1>7` の `<2>2` である -- そこで `α'` の D11a の接頭条件から `α` の接頭条件を出す。

### DEF 欠損

対応する活性化を固定する。`α` の点 `τ` と計数下 (D26) のオブジェクト `O` について、

`d(τ, O) :=` (`τ` より前に実行された `Del` の `Retain` 節点の素動作が `O` への参照を作った個数)
`-` (`τ` より前に実行された `Del` の `Release` 節点の素動作が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。節点 `q` の
入口の点での値を `d(q, O)` と書く。

**この 2 つの数え上げが渡るのは、`α` が `ρ` の上で実行した `Del` の節点の素動作だけである。**
`DEF 節点の実行の素動作` の 5 種のうち、子の活性化の素動作・(F) の解放・グローバル化は、どれも `α` が
`ρ` の節点について D10 の `Retain`/`Release` の行を実行する素動作ではないので、`d` を動かさない。
`Del` の要素は `Retain` 節点か `Release` 節点だけである (L32 の 5) ので、したがって

- `q ∉ Del` のとき、`q` の実行のどの点でも `d(τ, O) = d(q, O)` である。
- `q ∈ Del` のとき、`d` が動くのは `q` 自身の D10 の行の素動作の直後の点だけである。`Retain` 節点では
  leaf ごとに 1 上がり、`Release` 節点では leaf ごとに 1 下がる。`q` の実行が終わった点での値が
  `d(q', O)` である。

**`d(q, O)` は README の P21 (a) の `k(O)` である。** L43 の <1>8 が、`d(q, O)` が「`q` の入口の点までに
実行された
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

D9 の消費の表の行が指す節点を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

### DEF 類ごとの義務

別名類 `C` (D33) について、`β(C)` を、`C` の ρ-終端が借用する (D14) パラメータ・capture の leaf である
とき 1、そうでないとき 0 と定め、`held_ρ(τ, C)` (D34) が定まる各時点 `τ` について

`obl_ρ(τ, C) := held_ρ(τ, C) - β(C)`

と置く。**これは A19 (i) の `d(C)` である** -- A19 (i) は
`d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置き、その角括弧が `β(C)` である。
以下で `d` はこの文書の `DEF 欠損` の量を指すので、A19 (i) の `d(C)` はこの名前で書く。

`bumps_ρ(τ, C)` は、A19 (ii-b) が言う「走査がその類について `pending` に数えている bump の個数」で
ある。その帰属は D27 が定めるので、`τ` に対応する走査の状態 (`DEF 節点の実行の素動作`) の `pending` に
ついて
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
  <2>2. `node_id(n)` は `n.expr` が指す `RcExpr` の番地である。`RcExprNode` の `expr` は
        `Arc<RcExpr>` であり、EXT Arc の割り当ての安定性 より、その番地は同じ割り当てについて何度取っても
        等しく、`Arc` の値を move しても動かず、共有参照が生きている間は割り当ても落ちない。
    BY CODE src/rc_ir/borrow.rs: node_id, CODE src/rc_ir/borrow.rs: NodeId,
       CODE src/rc_ir/ast.rs: RcExprNode, EXT Arc の割り当ての安定性
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
      `self.walk(&arm.body, pending.clone(), false)` を呼ぶ。`PendingRetains` は `Vec<PendingRetain>` で
      あり、`PendingRetain` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の `node` と
      `outstanding` は原本のものの `clone` である。EXT Vec::clone より、複製の
      長さと各位置の要素は原本のものに等しいので、アーム本体の入口状態は `pending(n)` と等しい値である。
  BY D3, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     CODE src/rc_ir/borrow.rs: PendingRetain, CODE src/rc_ir/borrow.rs: PendingRetains,
     EXT Vec::clone, EXT derive(Clone), p30 の L1
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
<1>3. <1>2 の第 1 引数は、`r` の訪問が `un_bump` を呼ぶところの `pending` であり、それは `pending(r)` に、
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
4. `I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に在るところで走る
   `consume_objects(pending, objects)` の呼び出しはどれも、その要素のそのときの `outstanding` が名指す
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
  <2>1. `I_ρ(t)` に入る節点 `n` の訪問の中で `consume_objects(pending, objects)` が走るところで、由来が
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

### L39a (解放の走査は義務集合を戻す)

**言明** --- `ρ` の上の節点 `q` の実行の中で、ある処分に付随して走る D24 の (F) の解放の連鎖
(`DEF 節点の実行の素動作`) を取る。`τ0` をその処分の直前の点、`c(O)` をその処分が `Obl` から取り除く
`O` への参照の個数とする。このとき、連鎖が終わった点 `τ1` について `Obl(τ1, O) = Obl(τ0, O) - c(O)` で
あり、`τ0` と `τ1` のあいだの各点 `τ` について `Obl(τ, O) ≥ Obl(τ0, O) - c(O)` である。すなわち**解放は
`Obl` を正味で動かさず、その内側でも下げない。**

**支えは D24 の網羅である。** D24 は「**段の記述は `Obl` について網羅である。** (E1)-(E8) と (F) は、
各段について `Obl` を離れる参照の行き先と、作られる参照の持ち手と、`H` の動きを全部書いている。
**すなわち、ここに挙がっていない動きは起きない。**」と述べ、その節を要る段として
「`p40-cancel-soundness.md` の `L39a` のように、D21 の意味の活性化 -- 実行に実現するとは限らないもの --
の `Obl` を論じる段は、P28 を引けないのでこの網羅で迂回する」と、この補題を名指す。解放の連鎖の素動作に
ついて D24 が `Obl` に触れると述べるのは、(F) の retain と (E4) の 2 つの返り、および D9 の消費の行と
(E2) の行き先の一覧だけであり、以下はその数え上げである。
**主語は D21 の意味の 1 つの活性化であり、実行 (D24) が作る活性化に限らない。**

**証明**

<1>1. 連鎖で起きるのは D24 の (F) の解放である。各解放は、`o` が持つ参照の処分と、`o` が
      `Std::FFI::Destructor` のオブジェクトであるときの**2 つの活性化**とその返りからなる。D24 の
      「活性化の林」は「**(F) の解放が作る活性化は 2 つである** -- `_dtor` の欄の関数を `_value` の欄の
      値へ適用するものと、それが返した `IO` の動作の runner を適用するものであり、2 つ目の入力は 1 つ目の
      結果である」と述べる。
  BY DEF 節点の実行の素動作, D24
<1>2. `o` が持つ参照の処分は `Obl` を変えない。
  BY D24, D25, D10
  D24 の (F) は解放を「`o` が持つ参照 (D25) をすべて処分し、それから `o` の記憶域を返すこと」と定め、
  D25 の 2 つ目より、その参照の持ち手は生きているオブジェクト `o` である。D24 の「**段の記述は `Obl` に
  ついて網羅である。**」の節より、各段について `Obl` を離れる参照の行き先は全部書かれており、
  「**すなわち、ここに挙がっていない動きは起きない。**」。(F) の本文がこの処分を `Obl` を離れる参照として
  挙げることはない --
  (F) の本文が `Obl` について述べるのは、`_dtor` の retain が作る参照の持ち手がこの解放を含む段を実行して
  いる活性化であることと、`_value` の参照が `o` を離れて子の `Obl` の初期値に入ることの 2 つであり、
  どちらもこの処分ではない。
  D10 も、`Obl` を動かす行を活性化が実行する構文に付けており、この処分を名指す構文は無い (D24 の (F)
  -- 「適用される関数も、それが返す `IO` の動作も、オブジェクトの欄から来るものであって、どの構文も
  それを名指さない」)。
<1>3. `_value` の欄の参照は `Obl` を通らない。D24 の (F) より、`o` の `_value` の leaf が持つ参照は
      `o` を離れて、適用が作る活性化 `b` の `Obl(b)` の初期値に入る。
  BY D24
<1>4. この解放が `Obl` に入れる参照は 3 種であり、どれも同じ連鎖の中で、それを入れた素動作より後の
      素動作で `Obl` を離れる。
  <2>1. `_dtor` の欄の関数に retain が与える参照。D24 の (F) より、その持ち手はこの解放を含む段を
        実行している活性化であり、その retain は `_dtor` の欄の関数に**適用の分**の参照を与えるもので
        ある。その適用は D9 の `App` の行により callee の全 boxed leaf の参照を消費するので、この参照は
        同じ連鎖の中で `Obl` を離れ、1 つ目の活性化の `Obl` の初期値に入る。retain が適用より前に立つ
        ことは D24 の (F) が「**この段は参照も作る。** `_dtor` の欄の関数に適用の分の参照を与える retain が
        それである」と述べ、コードの位置を `build_retain(dtor, one, ...)` が `apply_lambda` の前に立つ
        こととして書いていることから出る。
    BY D24, D9
  <2>2. 1 つ目の活性化が返す参照。D24 の (E4) は「(F) が作る 1 つ目の活性化は `_dtor` の欄の関数を
        `_value` に適用したものであり、その返り値 `io_act` は 2 つ目の活性化 -- 返った `IO` の runner の
        適用 -- の入力になる」と述べ、「どちらの返りでも、参照はその解放を含む段を実行した活性化の
        `Obl` に入る」と続ける。よってこの参照は `Obl` に入り、続く 2 つ目の適用が D9 の `App` の行に
        より消費するので、同じ連鎖の中で `Obl` を離れて 2 つ目の活性化の `Obl` の初期値に入る。返りが
        2 つ目の適用より前にあることは、(E4) の「その返り値 `io_act` は 2 つ目の活性化 -- 返った `IO` の
        runner の適用 -- の入力になる」から出る。
    BY D24, D9
  <2>2a. 2 つ目の活性化が返す参照。D24 の (E4) より、これも `Obl` に入る。**`o` の `_value` の欄へ
        書き込まれるのはこの 2 つ目の返り値だけである** -- (E4) は「2 つ目の返り値は `o` の `_value` の
        欄へ書き込まれる」と述べる。D24 の (E2) は「既に在るオブジェクトの leaf」を `Obl` を離れる参照の
        行き先の 1 つとして挙げるので、この参照は書き込みによって `Obl` を離れ、その持ち手は `o` に
        なる (D25 の 2 つ目)。
    BY D24, D25
  <2>3. QED
    BY <2>1, <2>2, <2>2a, D24
    D24 の (F) がこの解放について参照を作ると述べるのは <2>1 の retain だけであり、解放の中で `Obl` へ
    参照が入るもう 1 つの道は (E4) の 2 つの返りだけである。作られた活性化自身の事象が動かすのは、その
    活性化の `Obl` である (D24 の (E2)、(E3)、(E4))。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, D24
  D24 の (F) より解放の連鎖は有限で終わる。その各解放について <1>2 から <1>4 が、`Obl` に入った参照が
  すべて同じ連鎖の中で `Obl` を離れ、それ以外に `Obl` を動かす素動作が無いことを述べる。よって連鎖が
  終わった点の `Obl` は、処分が取り除いた分だけ `τ0` より少ない。連鎖の内側の各点については、<1>4 の
  3 種のどれも、それを `Obl` に入れる素動作が、それを `Obl` から離す素動作より前にあるので、`Obl` は
  その値を下回らない。

### L39b (子の活性化の事象は義務集合を動かさない)

**言明** --- `ρ` の上の節点 `q` の実行の素動作のうち、子の活性化の素動作とグローバル化
(`DEF 節点の実行の素動作`) は、`Obl(α)` を動かさない。

**証明**

<1>1. 子の活性化の素動作は、D24 の (E3)、(E7)、および (E2) のうちオペランドを適用する `Llvm` の段が作る
      活性化と、その子孫の活性化のものである。子の活性化自身の素動作が動かすのは、その子の `Obl` である
      (D24 の (E2)、(E3)、(E4))。
  BY DEF 節点の実行の素動作, D24
<1>2. (E3) の `App` について、`α` の `Obl` を動かすのは D9 の `App` の行の消費と (E4) の受け取りであり、
      どちらも `α` 自身の節点について D10 の行が定めるものなので、`DEF 節点の実行の素動作` の第 1 の
      箇条に在る。子の活性化の素動作には無い。
  BY D24, D9, D10, DEF 節点の実行の素動作
<1>3. (E2) のうちオペランドを適用する `Llvm` の段についても同じである。渡る参照は D9 の `Llvm` の行が、
      受け取る参照は D10 の生成の `Llvm` の行が定める -- D24 の (E4) は「`b` を作ったのが (E2) のうち
      オペランドを適用する `Llvm` の段であれば、それらの参照はその段を実行した活性化の `Obl` に入り」と
      述べる。どちらも `α` 自身の節点について D10 の行が定めるものである。
  BY D24, D9, D10, DEF 節点の実行の素動作
<1>4. (E7) のグローバルの初期化は `α` の `Obl` を動かさない。D24 の (E7) より、初期化子の活性化の終端の
      `Ret` が消費する参照の行き先は呼び出し元ではなく `E` である。グローバルを読む節点が参照を得ない
      ことは A8 と D26 が言う (D24 の (E7))。同じ段の中で走る (E5) のグローバル化も、`Obl` を離れる参照も
      作られる参照も持たない -- D24 の (E5) は印を付けることだけを述べる。
  BY D24, A8, D26
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, DEF 節点の実行の素動作
  `DEF 節点の実行の素動作` より、子の活性化の素動作はこの 3 種の段が作る活性化とその子孫のものに尽き、
  グローバル化は (E7) が作る活性化が終わるところにだけ現れる。

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
  <2>2. `decl` が持つ鍵は `boxed_leaf_paths(result_ty, type_env)` の各元である。
    <3>1. `Provenance` の値の鍵の集合は、それを作った `LeafMap::build_shape` に渡した型の
          `boxed_leaf_paths` である。`LeafMap::build_shape(ty, type_env, leaf)` は
          `boxed_leaf_paths(ty, type_env)` の各元を鍵に据えた写像を作る。`Provenance::build_shape` は
          それをそのまま包み、`Provenance::uniform` は `LeafMap::uniform` を経てそれを呼び、
          `Provenance::uniform_bottom` は `Provenance::build_shape` を呼ぶ。`Provenance::fresh_under` は
          `uniform` の値に `set_leaves_under` を掛けるが、それは `LeafMap::map_leaves_under` で各鍵の値を
          写すだけで鍵を増減しない。
      BY CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform,
         CODE src/rc_ir/provenance.rs: Provenance::build_shape,
         CODE src/rc_ir/provenance.rs: Provenance::uniform,
         CODE src/rc_ir/provenance.rs: Provenance::uniform_bottom,
         CODE src/rc_ir/provenance.rs: Provenance::fresh_under,
         CODE src/rc_ir/provenance.rs: Provenance::set_leaves_under,
         CODE src/rc_ir/leaf_map.rs: LeafMap::map_leaves_under
    <3>2. `result_prov` の値は、`result_ty` を渡した `<3>1` の 4 つの構成子のいずれかが作る。
          `LLVMGen::result_prov` の既定の実装は `Provenance::uniform(result_ty, type_env,
          LeafOrigin::Unknown)` を返す。これを override するのは `src/fixstd/builtin.rs` の 29 個であり
          (A3 がその個数を述べる)、そのどれもが `Provenance::uniform(result_ty, ・, ・)`、
          `Provenance::uniform_bottom(result_ty, ・)`、`Provenance::fresh_under(result_ty, ・, ・)`、
          `Provenance::build_shape(result_ty, ・, ・)`、または同じ 2 つを `result_ty` に掛ける
          `replaced_field_prov` を返す。**`Provenance::empty()` と `Provenance` の `Default` は
          `result_prov` の実装に現れない** -- `Provenance::empty()` を呼ぶのは
          `src/rc_ir/provenance.rs` の単体テストだけであり、`Default` を呼ぶ生産コードは無い。この
          2 つは鍵を 1 つも持たない値を作るので、この数え上げが要る。
      BY CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, CODE src/fixstd/builtin.rs: replaced_field_prov,
         CODE src/rc_ir/provenance.rs: Provenance::empty, CODE src/rc_ir/provenance.rs: Provenance, A3
    <3>3. QED
      BY <3>1, <3>2, <2>1
      `<2>1` より `result_ty = ty(u)` である。
  <2>2a. `leaf_origins_at(σ)` は `Some` を返す。`leaf_origins_at` は `decl` の写像を鍵 `σ` で引くだけで
         あり (`LeafMap::get`)、本補題の仮定より `σ` は `boxed_leaf_paths(ty(u))` の元なので、`<2>2` より
         それは `decl` の鍵である。
    BY <2>1, <2>2, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>3. `σ` の宣言は、空集合か、`Fresh` ただ 1 つか、`Unknown` ただ 1 つである。A3 より、このコミットの
        すべての op の宣言は結果の各 leaf に元数 0 か 1 の `LeafOrigins` を与える。元数 1 でその元が
        `LeafOrigin::Arg` ならば `as_arg_projection` は `Some` を返し、<2>1 の場合に入らない。
    BY <2>1, <2>2a, A3, CODE src/rc_ir/ownership.rs: as_arg_projection
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
        `origin(container, [idx] ++ σ)`、`origin(scrut, [tag] ++ σ)` である。**呼ばれる側の変数を `w`、
        足される添字を `i` と書く** -- `Binding::Field` では `w = container`、`i = idx` であり、
        `Binding::Payload` では `w = scrut`、`i = tag` である。`u` は `w` から取り出された値の変数で
        あって、この場合の 2 つの型は `ty(w)` (容器・scrutinee の型) と `ty(u)` (フィールド・payload
        変数の型) である。
    <3>1. `ty(w)` について、D4 の第 5 規則が当たる。すなわち `ty(w)` は `is_fully_unboxed` でも
          `is_closure` でも `is_box` でも `is_array` でもない。
      BY A12, A10, D4, 本場合の仮定, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
         CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_struct,
         CODE src/ast/types.rs: TypeNode::is_union, CODE src/ast/types.rs: TypeNode::is_array,
         CODE src/ast/types.rs: TypeNode::is_funptr, CODE src/fixstd/builtin.rs: bulitin_tycons
      `ty(w).is_box` が偽であることは本場合の仮定である。A12 は「`Match` の scrutinee が union であること、
      `Destructure` の容器が構造体であること」を述べ、README の A12 は「**この仮定が型の `variant` を
      述べる各節では、その型の `is_closure()` は偽である**」と続けるので `ty(w).is_closure` は偽である。
      `is_struct` と `is_union` はその型の `TyConInfo` の `variant` が `Struct` か `Union` であることで
      あり、`Std::Array` の `variant` は `Array`、`Std::#FunPtr{n}` の `variant` は `Primitive` なので、
      `ty(w)` については `is_array` も `is_funptr` も偽である
      (`CODE src/fixstd/builtin.rs: bulitin_tycons` -- `make_array_tycon` と `make_funptr_tycon` に
      与える `TyConInfo`)。`is_fully_unboxed` は、`is_box`・`is_closure`・`is_array` のいずれでもなく
      `is_funptr` でもない型について、`unpunched_field_types` の各フィールドの型が `is_fully_unboxed` で
      あることと同値である。A10 より、`ty(w)` の `unpunched_field_types` の歩みは abort せず有限である。
      A12 より `Destructure` が名指すフィールドと `Match` が名指す変位は punched でなく、その型は `ty(u)` に
      等しいので、`unpunched_field_types(ty(w))` は `ty(u)` を含む。本補題の仮定より
      `σ ∈ boxed_leaf_paths(ty(u))` であり、`boxed_leaf_paths` は `is_fully_unboxed` の型について空の列を
      返すので、`ty(u)` は `is_fully_unboxed` ではない。よって `ty(w)` も `is_fully_unboxed` ではない。
    <3>2. QED
      BY <3>1, D4, D17, A12, CODE src/rc_ir/ownership.rs: origin_inner
      D4 の第 5 規則は「それ以外 (unbox の構造体・タプル・union) は、`unpunched_field_types` が返す
      フィールドの下へ降りる。union のときは各変位の payload へ降りる」と述べるので、
      `boxed_leaf_paths(ty(w))` は `[i] ++ boxed_leaf_paths(ty(u))` を含む。よって `[i] ++ σ` は
      `ty(w)` の boxed leaf である。D17 の第 2 行が同じことを「unbox 容器の `Destructure` のフィールド、
      unbox union の変位アームの payload: `λ` の先頭に添字を足す」と述べる。
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
  <2>1. `bumps_ρ(τ, C) ≥ 1` とする。DEF 類ごとの義務 より、`τ` に対応する走査の状態
        (`DEF 節点の実行の素動作`) の
        `pending` のある要素 `p` と、`C` に属するある名前 `o` について `B_ρ(τ, p)[o] ≥ 1` である。
    BY DEF 類ごとの義務, DEF 節点の実行の素動作, D27
  <2>2. `p` の由来 (DEF 訪問) を `t = Retain(v, π)` とすると、`o = id(v, λ)` である `λ ∈ L(v, π)` が
        在る。その `λ` は `π` の下の inhabited (D16) かつ計数下 (D26) の boxed leaf である。
    BY D27, P16, 第 1 節の記法, D4, D16, D26
    D27 は、`p` が `pending` に入るときの `B_ρ` を、`π` の下の inhabited かつ計数下の各 leaf を
    `origin` の identity で名付けて数えたものと定め、以後の操作はその多重集合から引くか、値をそのまま
    運ぶだけである。よって `B_ρ(τ, p)` が 1 以上を与える名前はその数え上げに現れる名前であり、`L(v, π)`
    の元の `id` である。P16 の (a) より `p` の由来は `Retain` 節点である。
  <2>3. `α` が `τ` に実行している節点を `q` とすると、`t` は `ρ` の上で `q` より前にあるか `q` 自身で
        ある。
    BY L35, L34, DEF 節点の実行の素動作, DEF 訪問
    `DEF 節点の実行の素動作` より、`τ` に対応する走査の状態は `q` の訪問の中の `pending` である。
    L34 の 1 より
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
`O` について、`N(q, O) ≥ 1` ならば、`q` の入口の点での参照カウントは `H(q, O) ≥ N(q, O) + 1` である。

**主語は D21 の意味の活性化である。** D21 は各時点で A19 (i) の不等式を満たすものだけを活性化とするので、
この補題を `α` について読む段は、`α` がその制限を満たすことに立つ (`L44` の (f))。

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

**主語は D21 の意味の活性化である。** この補題が読む A19 の (ii-a)・(ii-b) と P14a は、どれも各実行路と
それを辿る各活性化についての言明であり、D21 は各時点で A19 (i) の不等式を満たすものだけを活性化とする。
`α` についてこの補題を読む段は、`α` がその制限を満たすことに立つ (`L44` の (f))。この注は、この補題を
読む `L41b` と `L42` にも掛かる。

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

**終端の `Ret` の消費より前に限るのは、A19 の (ii-b) がその点で偽だからである。** A19 は
「**延ばすのは (ii-a) だけである。(ii-b) はこの位置で偽である。**」と述べ、反例を挙げる。

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

**言明** --- 実行路 `ρ` を辿る 1 回の活性化 `α` を固定する。終端の `Ret` の消費より前の `α` の各点
(`DEF 実行時の量`) `τ` と各計数下オブジェクト `O` について、`Obl(τ, O) ≥ b(τ, O)` である。とくに `ρ` の
上の節点 `q` の入口の点では `Obl(q, O) ≥ N(q, O)` である。

点と走査の状態の対応は `DEF 節点の実行の素動作` が置く。

**証明**

<1>1. 計数下のオブジェクト `O` と `α` の各点 `τ` について
      `Obl(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C)` であり、解放の内側でない点では等号が成り立つ。
  <2>1. `α` の点で `Obl(α)` を動かす素動作は、D10 の行が定める作成と処分、および (F) の解放が
        `Obl` に出し入れするものに尽きる。D10 が `Obl` を変える事象は、初期値、`Retain`、`Release`、
        生成、消費の 5 種である。移動は `Obl` を変えない。D26 より、数えるのは計数下のオブジェクトへの
        参照だけである。`DEF 節点の実行の素動作` の残る 3 種のうち、子の活性化の素動作とグローバル化は
        `Obl(α)` を動かさず (L39b)、割り当ては D10 の生成の行と同じ素動作である (D24 の (E2) の `H` の
        表 -- 単一の `Fresh` と `Closure(f, caps)` の行が、割り当てたオブジェクトの `H` を 1 とする)。
        (F) の解放については L39a が、`Obl` に入れる参照がすべて同じ連鎖の中で `Obl` を離れること、
        連鎖の内側で `Obl` が下がらないことを述べる。
    BY D10, D26, L39a, L39b, D24, DEF 節点の実行の素動作
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
      BY <3>1, <3>2, <3>3, D33, p13 の DEF ρ-歩みと ρ-終端
      D33 は「**歩みは有限である** -- 各段は `origin_inner` が `origin` を呼ぶ腕に 1 対 1 で
      対応し、P2 よりその再帰は停止する。」と述べるので、歩みが止まる位置は `origin_inner` が `origin` を
      呼ばない位置である。`p13` の `DEF ρ-歩みと ρ-終端` が同じことを述べる。
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
  <2>5a. D10 の 1 つの事象は、`Obl(・, O)` と `Σ_{C ∈ S(・, O)} obl_ρ(・, C)` を同じだけ動かす。
    BY <2>2, <2>3, <2>4, <2>5, D10, D14, D34, D26, DEF 類ごとの義務
    <2>2 より、事象はちょうど 1 つの inhabited な leaf に紐づき、その leaf のスロットはちょうど 1 つの
    別名類 `C` に属する。D26 より `obj(C)` が計数下のときだけ両辺が動き、そのとき `Obl(・, obj(C))` と
    `held_ρ(・, C)` は <2>3 の対応で同じだけ動く -- D34 の第 4 行が `Retain`、第 5 行が `Release`、
    第 6 行が消費、第 1 行が生成である。D34 は「表の第 4・第 5・第 6 行の事象は、**それを運ぶ素動作の
    直後の段内の点**で `held` を動かす。」と述べ、「第 1 行と第 2 行の開始値は、**その類の終端の参照が
    `Obl(a)` に入る素動作の直後の段内の点**に置く。」と続けるので、両辺は同じ点で動く。`β(C)` は点に
    依らないので `obl_ρ` の動きは `held_ρ` の動きに等しい。初期値の 2 行については、所有する (D14) パラメータ・capture の leaf では D10 の初期値が
    参照を 1 つ入れ `held_ρ` の第 2 行も 1 から始まるので両辺が 1 ずつ増え (`β = 0`)、借用する leaf では
    D10 が参照を入れず、`held_ρ` の第 3 行の 1 は `β = 1` が打ち消すのでどちらも動かない。<2>5 より
    計数下の `O` を指す類の ρ-終端はこの 3 つのいずれかなので、ほかの開始行は無い。
  <2>5b. `α` の素動作の列の上の帰納で `<1>1` を示す。
    <3>1. 基底。`α` が生きている活性化 (D23) になる点で、両辺は 0 である。
      BY D10, D23, D34, D24, DEF 類ごとの義務
      その点で `Obl(α)` は空である -- D10 の初期値が要求する参照は D24 の (E1)・(E3)・(E7) が受け渡しの
      素動作で `Obl(α)` に入れるものであり、その素動作はこの点より後にある。右辺の側では、D34 の第 1 行と
      第 2 行の開始値がまだ置かれておらず、第 3 行の開始値 (借用する終端の類) は
      「**その活性化が生きている活性化 (D23) になる点**」に置かれるが、その類は `β = 1` なので
      `obl_ρ = 0` である。D34 は「**置き方は自由ではない。** 開始値をまとめて活性化の初期 `Obl` を作る
      受け渡しの列の**直前**に置くと、所有するパラメータの boxed leaf を 2 つ以上持つ関数では、その列の
      途中の段内の点で `Σ d` が `Obl(a)` の個数を上回り、**A19 (i) がその点で偽になる**。」と述べ、
      この置き方を指定する。
    <3>2. 帰納段。ある点で `<1>1` が成り立つとき、次の素動作の後の点でも成り立つ。
      BY <2>1, <2>5a, L39a, L39b, D34
      <2>1 の数え上げより、次の素動作は次のいずれかである。D10 の行が定める作成・処分のとき、<2>5a より
      両辺が同じだけ動くので、不等号も等号も保たれる。子の活性化の素動作とグローバル化のとき、左辺は
      L39b より動かず、右辺も D34 の表がその素動作に行を持たないので動かない。(F) の解放の連鎖の素動作の
      とき、右辺は D34 の表がその素動作に行を持たないので動かず、左辺は L39a より連鎖が終わった点の値
      以上であって、連鎖が終わった点で
      その値に戻る。連鎖を起こした処分は D10 の行の事象なので <2>5a が両辺を同じだけ下げており、その
      下げは D34 が「それを運ぶ素動作の直後の段内の点」に置くので、連鎖の内側の各点で右辺は既に下がって
      いる。よって連鎖の内側では左辺が右辺以上、連鎖が終わった点では等号が戻る。
    <3>3. QED
      BY <3>1, <3>2, D24
      D24 より実行の素動作は 1 つの列をなすので、`α` のどの点についてもその前の素動作は有限個であり、
      その接頭についてこの帰納が届く。
  <2>6. QED
    BY <2>5a, <2>5b
<1>2. 各点 `τ` と各 `C ∈ S(τ, O)` について `obl_ρ(τ, C) ≥ bumps_ρ(τ, C)` である。
  BY L41a
  本補題が量化するのは終端の `Ret` の消費より前の点なので、L41a の範囲に入る。
<1>3. QED
  BY <1>1, <1>2, p13 の L17, L40a, DEF 類ごとの義務
  <1>1 と <1>2 より `Obl(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) ≥ b(τ, O)` である。節点 `q` の入口の点では
  `p13` の `L17` より `Σ_{C : obj(C) = O} bumps_ρ(q, C) = N(q, O)` であり、L40a の 1 よりその和は
  `b(q, O)` に等しい。

### L43 (欠損は pending の bump の一部である)

**言明** --- 対応する活性化を固定する。`ρ` の上の**節点** `q` と計数下のオブジェクト `O` について

`d(q, O) = Σ_{t} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]`

である。ここで外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り、内側の和は `ρ` で
活性な名前 `o` を渡る。とくに `0 ≤ d(q, O) ≤ N(q, O)` である。`ρ` の最後の点 `q_end`
(`DEF 節点の入口の点`) については `d(q_end, O) = 0` である。

等式と境界を節点の入口の点について述べるのは、`DEF N` の `N(q, O)` と `DEF 訪問` の `pending(q)`・
`e_t(q)` が節点についての量だからである。`DEF 欠損` の `d` は `α` の各点についての量であり、節点を持たない
`q_end` での値を最後の文が与える。

さらに `d(q, O)` は README の P21 (a) の `k(O)` である。すなわち、`q` の入口の点より前に実行された `Del` の
`Retain` 節点が `O` に作った参照のうち、その `Retain` と対になる `Del` の `Release` 節点がまだ処分して
いないものの個数である。

**証明**

<1>1. `q` の入口の点より前に実行された `Del` の `Retain` 節点は、`ρ` の上で `q` より前にある `CT` の
      要素である。
  BY L32, DEF 欠損
<1>2. `q` の入口の点より前に実行された `Del` の `Release` 節点 `r` は、ある `t ∈ CT` について
      `r ∈ R_ρ(t)` であり、
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
  P18b は `B(p, ρ)` を「`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上で
  まだ処分されていないものを、それを作った leaf の `origin` の identity で名付けた多重集合」と定めるので、
  `B_ρ(q, e)` はどの名前についても個数が 0 以上である。`p13` の `L11` より、活性でない `o` に
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

**言明** --- `Del` に入らない `ρ` の上の節点 `q` と計数下のオブジェクト `O` について、`q` の実行の
各点 (`DEF 節点の入口の点`) `τ` について `b(τ, O) ≥ d(q, O)` である。

**証明**

<1>1. `q` の入口の点について `b(q, O) ≥ d(q, O)` である。
  BY p13 の L17, L40a, DEF N, L43
  `p13` の `L17` は `N_ρ(q, O) = Σ_{C : obj(C) = O} bumps_ρ(q, C)` を述べ、L40a の 1 よりその和は
  `b(q, O)` である。DEF N より `N(q, O) = N_ρ(q, O)` であり、L43 より `d(q, O) ≤ N(q, O)` である。
<1>2. `q` の実行の各点 `τ` に対応する走査の状態 (`DEF 節点の実行の素動作`) には、`q` で pending である
      `CT` の各要素が、`q` の入口の点での `B_ρ` のまま入っている。
  <2>1. CASE `q` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let`
        である。`DEF 節点の実行の素動作` より、対応する走査の状態は `q` の訪問の途中または終わりの
        `pending` である。L34 の 1 より、`q` の訪問が `pending` に施す操作は `push`、`consume_objects`、
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
      BY <3>1, <3>2, <3>3, D27, DEF 節点の実行の素動作
      D27 は、`un_bump` が `InBracket` でその要素を選ぶ `Release` の訪問でだけ `B_ρ` を引き、ほかの
      どの操作も `B_ρ` を変えないと定める。
  <2>2. CASE `q` の式が `Let(_, RcRhs::Match(_, arms), k)` であるか、`q` がアーム本体の終端の `Ret` で
        ある。この 2 つは D10 の行を 1 つも持たない (D9、D10) ので、`DEF 節点の実行の素動作` より、この
        節点の実行のどの点に対応する走査の状態も `pending(q)` である。`CT` の要素はそこに在る。
    BY L34, D9, D10, DEF 節点の実行の素動作, DEF 訪問
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
  現れる。残る項は P18b より 0 以上である。`q` の入口の点自身については <1>1 が同じことを述べる。

### L41c (節点の実行の間も余りは残る)

**言明** --- 実行路 `ρ` を辿る活性化 `α` を固定する。`Del` に入らない `ρ` の上の節点 `q` と計数下の
オブジェクト `O` について `d(q, O) ≥ 1` とする。このとき、`q` の実行の各点 (`DEF 節点の入口の点`) `τ` に
ついて `H(τ, O) ≥ d(q, O) + 1` である。

**主語は D21 の意味の活性化である。** `<1>4` が読むのは D21 が活性化に課す制限であり、`α` について
それが成り立つことは `L44` の (f) が点ごとに与える。

**証明**

<1>1. `q` は `ρ` の終端の `Ret` ではない。よって `q` の実行の各点は、終端の `Ret` の消費より
      前にある。
  BY L38, L43, 本補題の仮定, D3, L33a
  L38 の 2 より、各 `t ∈ CT` が pending である区間は `ρ` の終端の `Ret` より真に前で終わるので、終端の
  `Ret` で pending な `CT` の要素は無く、L43 の等式より `d = 0` である。これは本補題の仮定に反する。
  D3 と L33a より終端の `Ret` は `ρ` の最後の節点なので、`q` の実行はその消費より前に終わる。
<1>2. `q` の実行の各点 `τ` について `b(τ, O) ≥ d(q, O) ≥ 1` である。
  BY L43a, 本補題の仮定
<1>3. `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1] ≥ d(q, O) + 1` である。
  BY L41b, <1>1, <1>2
<1>4. `H(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1]` である。
  BY D21, A19, D23, D33, D34, DEF 類ごとの義務
  D21 は「**活性化は、その各時点で A19 (i) の不等式を満たすものに限る。**」と述べる。すなわちこの
  不等式は仮定ではなく、D21 が活性化に課す制限である。A19 (i) は「`a` の計数下の別名類のうち
  `obj(C) = O` であり開始の時点がその時点以前であるものの全体を `S` とし、各類について
  `d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置くと、」として、引用の形で
  「`H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]`」を置く。A19 は「各時点」が
  その活性化が生きている間のすべての時点であり、入れ子の呼び出しで中断中の時点を含むことを明記する。
  `α` は `τ` で生きている (D23 -- 始まって終わっていない)。A19 (i) の `S` は `S(τ, O)`、`d(C)` は
  `obl_ρ(τ, C)`、角括弧は `β(C) = 1` の類が在るかである (DEF 類ごとの義務)。
<1>5. QED
  BY <1>3, <1>4

### L41d (参照を持つ類のオブジェクトはカウントを持つ)

**言明** --- 実行路 `ρ` を辿る活性化を固定する。終端の `Ret` の消費より前の各点 `τ` と各計数下オブジェクト
`O` について、`S(τ, O)` に `held_ρ(τ, C) ≥ 1` である類 `C` が在るならば `H(τ, O) ≥ 1` である。とくに、
`ρ` の上の `Retain(v, π)` 節点または `Release(v, π)` 節点 `q` の入口の点では、`π` の下の inhabited (D16)
かつ計数下 (D26) の各 leaf `λ` について `H(q, obj(v, λ)) ≥ 1` である。

**主語は D21 の意味の活性化である。** `<1>3` が読むのは D21 が活性化に課す制限であり、`α` について
それが成り立つことは `L44` の (f) が点ごとに与える。

**証明**

<1>1. 各 `C' ∈ S(τ, O)` について `obl_ρ(τ, C') ≥ 0` である。
  BY L41a, DEF 類ごとの義務, D27, P18b
  L41a より `obl_ρ(τ, C') ≥ bumps_ρ(τ, C')` であり、`bumps_ρ` は `B_ρ` の個数の総和なので P18b より
  0 以上である。
<1>2. `Σ_{C' ∈ S(τ, O)} obl_ρ(τ, C') + [S(τ, O) に β(C') = 1 の類が在るならば 1] ≥ 1` である。
  <2>1. CASE `β(C) = 0`。DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 1` であり、<1>1 より
        残りの項は 0 以上、角括弧も 0 以上なので、総和は 1 以上である。
    BY DEF 類ごとの義務, 本補題の仮定, <1>1
  <2>2. CASE `β(C) = 1`。`C ∈ S(τ, O)` なので角括弧は 1 であり、<1>1 より総和は 0 以上である。
    BY DEF 類ごとの義務, 本補題の仮定, <1>1
  <2>3. QED
    BY <2>1, <2>2, DEF 類ごとの義務
    DEF 類ごとの義務 より `β(C)` は 0 か 1 である。
<1>3. `H(τ, O) ≥ Σ_{C' ∈ S(τ, O)} obl_ρ(τ, C') + [S(τ, O) に β(C') = 1 の類が在るならば 1]` である。
  BY D21, A19, D23, DEF 類ごとの義務
  D21 は「**活性化は、その各時点で A19 (i) の不等式を満たすものに限る。**」と述べる。A19 (i) の `S` は
  `S(τ, O)`、`d(C)` は `obl_ρ(τ, C)`、角括弧は `β(C) = 1` の類が在るかである (DEF 類ごとの義務)。
  `α` は `τ` で生きている (D23)。
<1>4. 「とくに」の節が成り立つ。
  BY A19, L40a, D6, D33, D34, D26, <1>1, <1>2, <1>3
  `λ` は `π` の下の inhabited かつ計数下の leaf なので、`(v, λ)` は `ρ` の上のスロット (D6) であり、
  D33 よりちょうど 1 つの別名類 `C` に属して `obj(C) = obj(v, λ)` である。L40a の 2 より `C` は `q` の
  入口の点で開始しているので `C ∈ S(q, obj(v, λ))` である。A19 の (ii-a) は「各時点と各計数下の別名類に
  ついて、その類が持つ参照の個数は非負であり、読む構文と `Retain`/`Release` がその類を名指す時点では
  1 以上である。」と述べるので、`held_ρ(q, C) ≥ 1` である。よって言明の仮定が満たされ、<1>1 から <1>3 が
  `H(q, obj(v, λ)) ≥ 1` を与える。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L43b (対応する 2 つの活性化の別名類)

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`α` の時点 `τ` を固定する。`τ` までに 2 つの活性化が
`Del` の節点を除いて同じ節点を実行し、その時点までに値を得ている各変数の値が D29 の全単射のもとで
対応しているとする。このとき次の 3 つが成り立つ。

1. `B` から作られる `VarTable` と `B'` から作られる `VarTable` は、`var_tys`・`param_tys`・
   `closure_targets` が等しく、`bindings` は鍵の集合と各鍵の `Binding` の変位・`RcVar`・添字・変位番号・
   型が等しい (`Binding::Llvm` が運ぶ op は同じ原本の複製であって同じオブジェクトではない)。鍵が等しい
   2 つの `origin` の答えは等しい。さらに `B` を本体に持つ関数と `B'` を本体に持つ関数の `params`・
   `capture`・`borrowed_units` は等しい。
2. `ρ` の上の位置 (D6) と `ρ'` の上の位置は、変数と leaf を同じくする対応で 1 対 1 に対応し、その対応は
   別名類 (D33) の 1 対 1 の対応を導く。対応する 2 つの類は `β` (DEF 類ごとの義務) が等しく、`τ` までに
   開始しているか (D34) が一致し、計数下であるか (D26) が一致する。計数下の類については `obj` が D29 の
   全単射で対応する。とくに各計数下オブジェクト `O` について `S(τ, O)` と `S'(τ, O)` はこの対応で
   移り合う。
3. 各計数下オブジェクト `O` について
   `Σ_{C ∈ S(τ, O)} held_ρ(τ, C) = Σ_{C' ∈ S'(τ, O)} held_{ρ'}(τ, C') + d(τ, O)` である。

**証明**

<1>1. 1 が成り立つ。
  <2>1. L30 と L32 の 5 より、`B'` は `B` から `Retain` 節点と `Release` 節点をいくつか取り除いた木で
        あり、残る各位置の式の変位・変数・path・`RcState`・`Match` のアームの本数と並び・継続の順序は
        変わらない。
    BY L30, L32
  <2>2. `collect_bindings` は `RcExpr::Retain` と `RcExpr::Release` の腕で継続へ降りるだけであり、
        `bindings`・`var_tys`・`closure_targets` に何も入れない。`returned_var` も同じ 2 つの腕で継続へ
        降りる。よって <2>1 の木の変形は `var_tys` と `closure_targets` を変えず、`bindings` については
        鍵の集合と、各鍵の `Binding` の変位・`RcVar`・添字・変位番号・型を変えない。**`Binding::Llvm` が
        運ぶ op はこの水準では等しいと言えない** -- <2>3a がそれを扱う。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var,
       CODE src/rc_ir/ownership.rs: Binding, <2>1
  <2>3. `cancel` は `prog.funcs.values()` の各 `f` について `f.clone()` を作って `clone.body` にだけ
        書き込み、鍵に `f.name.clone()` を据えるので、`params`・`capture`・`borrowed_units` は
        変わらない。グローバル初期化子はパラメータも capture も持たない (D1)。`VarTable::of` は
        パラメータと capture について `Binding::Param` と `param_tys`・`var_tys` を置き、残りを
        `collect_bindings` から取る。`VarTable::body_only` は `collect_bindings` だけを取る。
        `RcFunc` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より `f.clone()` の各欄は `f` の
        対応する欄の `clone` である。
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc,
       CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only, D1, EXT derive(Clone)
  <2>3a. 2 つの表の対応する `Binding::Llvm` が運ぶ op は、同じ引数に対して同じ `Provenance` を返す。
    <3>1. `B'` の `RcRhs::Llvm` が持つ op は `B` のものの複製であり、同じオブジェクトであるとは限らない。
          `drop_nodes_inner` の右辺が `Match` でない `Let` の腕は
          `RcExpr::Let(x.clone(), rhs.clone(), drop_nodes(k, to_delete))` を積む。`RcRhs` は
          `#[derive(Clone)]` を持つので、EXT derive(Clone) より `rhs.clone()` は `RcRhs::Llvm` の第 1 欄に
          `Box<dyn LLVMGen>` の `clone` を置く。`LLVMGen` は `DynClone` を継承し、
          `dyn_clone::clone_trait_object!(LLVMGen)` がその `Clone` を与えるので、EXT dyn_clone の trait
          object の複製 より、その複製が原本と同じオブジェクトであるとは限らない。`collect_bindings` は
          さらに `llvm_gen.clone()` を `Binding::Llvm` の第 1 欄に置くので、2 つの表の op は 2 段の複製で
          隔たっている。
      BY CODE src/rc_ir/borrow.rs: drop_nodes_inner, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/ast/inline_llvm.rs: LLVMGen, CODE src/rc_ir/ownership.rs: collect_bindings,
         EXT derive(Clone), EXT dyn_clone の trait object の複製
    <3>2. QED
      BY <3>1, A3
      A3 は「**`result_prov` と `borrows_operand` は決定的である** -- 同じ引数に対して常に同じ値を
      返す」と述べ、続けて「**この 2 節を合わせると「op の複製は原本と同じ宣言を返す」が出る。**
      `rhs.clone()` や `fresh_rename_function` が作る複製の op は、原本と同じ引数を渡されれば同じ
      `Provenance` を返す」と述べる。A3 はこの 1 文を要る段の見分け方を「複製された op の宣言を原本の
      ものと同じだと読む段が、それである」と書いており、<3>1 よりこの段がそれである。
  <2>3b. 鍵 `(x, π)` が等しい 2 つの表の `origin` の答えは等しい。
    <3>0. P2a より、1 つの表を固定すれば `origin` の答えは `vars.origins` の memo の状態に依らないので、
          各表の `origin` は鍵の関数である。よって 2 つの答えを比べるのに、memo が空の状態からの評価を
          取ってよい。その評価では `origin` は memo に当たらず `grow_stack(|| origin_inner(...))` を
          1 回呼ぶ (A15 より `grow_stack` は閉包をちょうど 1 回呼ぶ) ので、呼び出しの木は
          `origin_inner` の再帰の木である。P2 よりその評価は停止するので木は有限であり、その高さに
          ついての帰納法が整礎である。
      BY P2, P2a, A15, CODE src/rc_ir/ownership.rs: origin
    <3>1. `origin_inner` が `vars` から読むのは `bindings.get(var)` だけである。`var_tys`・`param_tys`・
          `closure_targets`・`origins` を読まない。
      BY CODE src/rc_ir/ownership.rs: origin_inner
    <3>2. CASE 腕が `None`/`Param`/`Producer`、`container.ty.is_box` が真の `Binding::Field`、または
          `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` である。これらは
          `Exactly((var, path))` を返し、`origin` を呼ばない。<2>2 より 2 つの表は同じ鍵で同じ腕へ入り、
          `is_box` が読む型も <2>2 より等しいので、返り値は等しい。
      BY CODE src/rc_ir/ownership.rs: origin_inner, <2>2
    <3>3. CASE 腕が `Binding::Move(y)`、`Binding::Join(arm_results)`、`Binding::Payload(scrut, None)`、
          `container.ty.is_box` が偽の `Binding::Field(container, idx)`、または `scrut.ty.is_box` が
          偽の `Binding::Payload(scrut, Some(tag))` である。これらが呼ぶ `origin` の引数 -- 変数と
          path -- は、<2>2 より 2 つの表で等しい。帰納法の仮定よりその答えは等しく、`Join` の腕が
          `Origin::of_candidates(candidates, (var, path))` に渡す `candidates` はそれらの答えの
          `acted_on()` の合併なので、これも等しい。
      BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Origin::of_candidates,
         CODE src/rc_ir/ownership.rs: Origin::acted_on, <2>2, 帰納法の仮定
    <3>4. CASE 腕が `Binding::Llvm(llvm_gen, args, result_ty)` である。この腕は `arg_tys` を `args` の
          型から作り、`decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を取る。<2>2 より
          `args` と `result_ty` は 2 つの表で等しく、第 1 節の記法 より `type_env` は 1 つなので、
          <2>3a より `decl` は 2 つで等しい。`as_arg_projection` は `decl` と `path` だけを読む。
          `origin_from_leaves_under` は `decl`・`args`・`path`・`type_env` から `operand_units` と
          `produced_here` を作り、`operand_units` の各元について `origin(vars, type_env, args[j].name,
          unit)` を呼び、その答えから返り値を組む。呼び出しの引数は 2 つの表で等しいので、帰納法の仮定
          よりその答えも等しく、返り値も等しい。
      BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: truncate_to_unit, <2>2, <2>3a, 第 1 節の記法, 帰納法の仮定
    <3>5. QED
      BY <3>0, <3>1, <3>2, <3>3, <3>4, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Binding
      `origin_inner` の `match` は `bindings.get(var)` の値について `None | Param | Producer`、`Move`、
      `Join`、`Llvm`、`Field`、`Payload` の 6 つの腕を持ち、`Field` と `Payload` は `is_box` で
      さらに分かれる。<3>2、<3>3、<3>4 がこれを尽くす。
  <2>4. QED
    BY <2>2, <2>3, <2>3a, <2>3b, 第 1 節の記法, CODE src/rc_ir/borrow.rs: cancel
    第 1 節の記法 より `type_env` はプログラムの `TypeEnv` であり、`cancel` は受け取った `type_env` を
    そのまま `CancelAnalysis` と `all_owned_units` に渡して型を作らないので、2 つの本体の `origin` は
    同じ `type_env` の下で読む。**P2a は 1 つの `VarTable` の値を固定した形の主張であり、相異なる
    2 つの表について答えを比べる形はその主張ではない** -- その形を <2>3b が `origin` の再帰の上の帰納で
    示す。
<1>2. 2 が成り立つ。
  <2>1. 位置 (D6) が対応する。D6 より位置は対 `(x, λ)` であり、`x` は値を得た変数か束縛を持たない名前、
        `λ` は `ty(x)` の inhabited (D16) な boxed leaf である。<1>1 より 2 つの `VarTable` の
        `var_tys` は等しく、A12 より束縛を持たない名前の型はその記号の型なので、`ty(x)` は 2 つで同じで
        あり、`boxed_leaf_paths(ty(x))` も同じである (D4)。D16 の inhabited は値が通る unbox union の
        タグで決まる。本補題の仮定より `τ` までに値を得ている変数は 2 つの活性化で同じであり、その値は
        D29 の全単射のもとで対応する。D29 の第 5 行より、対応する 2 つの値のスカラの成分 -- unbox union
        のタグを含む -- は等しく、inhabited な各 boxed leaf は全単射で対応するオブジェクトを指す。
        束縛を持たない名前 (記号の位置) は 2 つの本体で同じであり、そこが指すのは funptr かグローバル
        状態のオブジェクトである (D6)。
    BY D6, D4, D16, D29, A12, <1>1, 本補題の仮定
  <2>2. 別名の辺 (D20) と ρ-歩み・ρ-終端 (D33) が対応する。D33 の歩みは各位置で `origin_inner` が呼ぶ
        `origin` の引数へ進み、`origin` を呼ばない位置で止まる。<1>1 より `origin` の答えは 2 つの本体で
        等しく、<2>1 より位置が対応する。`Binding::Join` の辺はその活性化が選んだアームの結果へ進む
        (D17) が、本補題の仮定より 2 つの活性化は `τ` までに同じ `Match` の節点を実行しており、
        L30 と L31 よりその節点は同じ位置にあるので、選んだアームも同じである。
    BY D20, D33, D17, L30, L31, <1>1, <2>1, 本補題の仮定
  <2>3. 別名類が対応し、計数下の類の `obj` が対応する。D33 は 1 つの実行路の上の位置を ρ-終端が等しい
        という関係で分けた同値類を別名類と定め、`obj(C)` を類の各位置が指すオブジェクトとする。
        <2>1 と <2>2 より位置と ρ-終端が対応するので、類も対応する。ρ-終端が記号の位置である類の
        オブジェクトはグローバル状態であって計数下ではない (D33)。それ以外の類については、<2>1 より
        対応する 2 つの位置が指すオブジェクトが D29 の全単射で対応するので `obj` が対応し、D29 の
        第 5 行より計数下かグローバル状態か (D26) の区別も対応する 2 つのオブジェクトで一致する。
    BY D33, D26, D29, <2>1, <2>2
  <2>4. `β` が等しい。DEF 類ごとの義務 の `β(C)` は、`C` の ρ-終端が借用する (D14) パラメータ・capture の
        leaf であるとき 1、そうでないとき 0 である。D14 の借用は `RcFunc::borrowed_units` が定め、
        <1>1 よりそれは 2 つで等しい。<2>2 より ρ-終端の変数と leaf も 2 つで同じである。
    BY DEF 類ごとの義務, D14, <1>1, <2>2
  <2>5. `τ` までに開始しているかが一致する。D34 の開始の時点は、ρ-終端 `(u, σ)` の変数 `u` が値を得る
        時点である -- `u` がパラメータ・capture なら活性化が始まる時点、そうでなければ `u` を束縛する
        節点を実行する段の直後である。L32 の 5 より `Del` の要素は `Retain` 節点か `Release` 節点だけで
        あり、D2 よりこの 2 つは変数を束縛しない。本補題の仮定より `τ` までに値を得ている変数は 2 つの
        活性化で同じなので、`τ` までに開始している類も対応する。
    BY D34, L32, D2, <2>2, 本補題の仮定
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, DEF 類ごとの義務
    DEF 類ごとの義務 より `S(τ, O)` は、`obj(C) = O` であり `τ` までに開始した計数下の別名類の全体で
    ある。<2>3 と <2>5 がこの 3 つの条件を対応させる。
<1>3. 3 が成り立つ。
  <2>1. D34 の表の 6 行のうち、`Retain` の行と `Release` の行を除く 4 行 -- 生成、所有する初期値、
        借用する初期値、消費 -- が 2 つの活性化で起こす増減は等しい。
    BY D34, D10, D9, D14, L32, <1>1, <1>2, 本補題の仮定
    生成の行と消費の行が名指す leaf は、D10 の生成の表と D9 の消費の表が定めるとおり、節点の形と、
    それが名指す変数の値と、`App` については呼び出し先の所有 (D14) で決まる。L32 の 5 より `Del` の
    要素は `Retain` 節点と `Release` 節点だけなので、この 2 つの行を起こす節点はどちらの活性化も同じ
    だけ実行している (本補題の仮定)。値が対応することは本補題の仮定が、`borrowed_units` が等しいことは
    <1>1 が与える。初期値の 2 行はパラメータ・capture の leaf についてであり、<1>1 より `params` と
    `capture` は等しい。増減が掛かる類が対応することは <1>2 である。
  <2>2. `Retain` の行と `Release` の行について、`α` の側が `α'` の側より多く数えるのは、`τ` までに
        `α` が実行した `Del` の `Retain`/`Release` 節点の分だけである。
    BY L30, L31, L32, 本補題の仮定
    L30 と L31 より `ρ'` は `ρ` から `Del` の節点を除いた列であり、本補題の仮定より 2 つの活性化は
    それ以外の節点を同じだけ実行している。
  <2>3. `Del` の `Retain(v, π)` 節点 1 つが、`obj(C) = O` である計数下の類の `held_ρ` の総和に与える
        増分は、その節点が `O` への参照を作った個数に等しい。`Del` の `Release(v, π)` 節点についても、
        減分について同じことが成り立つ。
    BY D34, D10, D6, D33, D26, L40a
    D34 の第 4 行は「`Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ」類の `held_ρ` を
    その `λ` 1 つにつき 1 上げる。D6 より `(v, λ)` が位置であるのは `λ` が inhabited なときであり、
    D10 の `Retain` の行は `π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ作る。
    D33 より `(v, λ)` が属する類 `C` の `obj(C)` は `obj(v, λ)` である。D26 より参照を持つのは計数下の
    オブジェクトを指す leaf だけであり、D34 は計数下の類についてだけ `held_ρ` を定める。L40a の 2 より
    その類はその事象の時点で開始しているので `S(τ, O)` に入る。D34 の第 5 行と D10 の `Release` の行に
    ついても同じである。
  <2>4. QED
    BY <2>1, <2>2, <2>3, <1>2, DEF 欠損
    DEF 欠損 は `d(τ, O)` を「`τ` より前に実行された `Del` の `Retain` 節点が `O` への参照を作った
    個数」から「`τ` より前に実行された `Del` の `Release` 節点が `O` への参照を処分した個数」を引いた
    ものと定める。<2>3 をその全節点について足し、<2>1 と <2>2 と合わせると、2 つの総和の差は
    ちょうど `d(τ, O)` である。総和が渡る 2 つの類の集合が対応することは <1>2 である。
<1>4. QED
  BY <1>1, <1>2, <1>3

### L43c (対応する入力の活性化は D21 の制限を満たす)

**言明** --- 対応する活性化 `α`、`α'` (D29) と `α` の時点 `τ` を固定する。L43b の仮定が `τ` について
成り立ち、`α'` が `τ` に対応する時点で生きており (D23)、かつ各計数下オブジェクト `O` について
`H(τ, O) = H'(τ, O) + d(τ, O)` であるとする。このとき `α` は `τ` で D21 の制限 -- A19 (i) の不等式 --
を満たす。

**証明**

<1>1. `α'` は `τ` に対応する時点で A19 (i) の不等式を満たす。すなわち各計数下オブジェクト `O` に
      ついて
      `H'(τ, O) ≥ Σ_{C' ∈ S'(τ, O)} obl_{ρ'}(τ, C') + [S'(τ, O) に β(C') = 1 の類が在るならば 1]`
      である。
  BY D21, A19, D23, D29, DEF 類ごとの義務, DEF 対応する活性化の量
  D21 は「**活性化は、その各時点で A19 (i) の不等式を満たすものに限る。**」と述べる。`α'` は `B'` の
  活性化であり (D29)、`DEF 対応する活性化の量` の点の対応より `τ` に対応する `α'` の点が定まり、
  本補題の仮定
  より `α'` はそこで生きている (D23)。DEF 類ごとの義務 より、A19 (i) の `S` は `S'(τ, O)`、`d(C)` は
  `obl_{ρ'}(τ, C')`、角括弧は `β(C') = 1` の類が在るかである。
<1>2. 角括弧の値は 2 つの活性化で等しく、`Σ_{C ∈ S(τ, O)} β(C) = Σ_{C' ∈ S'(τ, O)} β(C')` である。
  BY L43b, 本補題の仮定
  L43b の 2 より `S(τ, O)` と `S'(τ, O)` は 1 対 1 に対応し、対応する類の `β` は等しい。
<1>3. `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) = Σ_{C' ∈ S'(τ, O)} obl_{ρ'}(τ, C') + d(τ, O)` である。
  BY L43b, <1>2, DEF 類ごとの義務, 本補題の仮定
  DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) - β(C)` である。L43b の 3 が `held_ρ` の総和の差を
  `d(τ, O)` と与え、<1>2 が `β` の総和を打ち消す。
<1>4. QED
  BY <1>1, <1>2, <1>3, 本補題の仮定
  本補題の仮定と <1>1 と <1>3 より
  `H(τ, O) = H'(τ, O) + d(τ, O) ≥ Σ_{C'} obl_{ρ'}(τ, C') + [角括弧] + d(τ, O) = Σ_{C} obl_ρ(τ, C) + [角括弧]`
  である。<1>2 より角括弧は 2 つで同じ値である。これが `α` についての A19 (i) の不等式である。

### L44 (2 つの実行の対応)

**言明** --- `ρ` を `B` の実行路、`ρ'` をそれに対応する `B'` の実行路、`α'` を `ρ'` を辿る `B'` の
活性化、`α` を D29 が `α'` に対応させる `B` の活性化とする。`ρ` の上の各節点 `q` と各計数下オブジェクト
`O` について次の 6 つが成り立つ。`ρ` の最後の点 `q_end` (`DEF 節点の入口の点`) についても同じである。

**`B` が `borrow_ify` の出力の本体であることを読む。** `<1>2` が読む `L41`・`L41c`・`L41d` は P18a を、
`L41c`・`L41d` が読む `L41b`・`L41a` は A19 の (ii-a)・(ii-b) と P14a を `α` に当てる。その範囲が
`borrow_ify` の出力の本体だからである (第 1 節)。
**入力プログラムが D12 を満たすことは、この補題のどの段も読まない。**

- **(a)** `α` と `α'` は、`q` の入口の点までに、`Del` の節点を除いて同じ節点を実行し、その点までに値を
  得ている各変数の値は、D29 の全単射のもとで対応する。
- **(b)** `H'(q, O) = H(q, O) - d(q, O)` である。
- **(c)** `Obl'(q, O) = Obl(q, O) - d(q, O)` である。
- **(d)** `d(q, O) ≥ 1` ならば `H(q, O) ≥ d(q, O) + 1` である。
- **(e)** `q` の実行の各点 `τ` -- `q` の入口の点と、その実行が終わった点 `q'` を含む
  (`DEF 節点の入口の点`) -- について、2 つの活性化の点が対応し、`H'(τ, O) = H(τ, O) - d(τ, O)` であり、
  `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。`q ∉ Del` のときは `d(τ, O) = d(q, O)` である
  (DEF 欠損)。
- **(f)** `α` は、`q` の入口の点までの各点と `q` の実行の各点で、D21 の制限 -- A19 (i) の不等式 -- を
  満たす。

**(b) と (e) の等式は D29 の与件である。** D29 の第 2 行は「`α` の参照カウントの推移は、`α'` のそれに
欠損 `k` (P21 (a)) を足したものとして与える」と述べ、L43 の最後の文よりその `k` は `DEF 欠損` の `d` で
ある。`H` は活性化の側のデータなので (D21)、これは証明が素動作ごとに積み上げる量ではなく、`α` を決める
データの一部である。**この補題が示すのは、その与件を読む先 -- 2 つの活性化の点の対応
(`DEF 対応する活性化の量`) -- が `ρ` の全体にわたって定まることと、(c)(d)(e)(f) である。**

**与件は 2 つの活性化の自分の素動作と整合する。** D21 より、活性化自身の事象は D10 の表のとおりに `H` を
動かし、それ以外の増減はこの活性化の外から来る。対応する 2 つの点の間に 2 つの活性化が実行する自分の
素動作は `Del` の節点の分だけ違い ((a))、`Del` の `Retain`/`Release` 節点が D10 の表によって `O` の `H` に
与える増減の総和は `DEF 欠損` の `d(・, O)` の増減そのものである。すなわち `H = H' + d` が言うのは、
2 つの活性化の外から来る増減が対応することであり、D29 はそれを `α` に与える。

**(f) が要る所。** D21 は制限を満たすものだけを活性化とするので、`α` について A19 の (ii-a)・(ii-b)、
P14a、P18a を読む段はこれに立つ。**この 4 つはどれも「各実行路と、それを辿る各活性化の各時点」に
ついての言明であり、点 `τ` についての結論が結ぶのは `τ` までの量 -- その点の `H`・`Obl`・`held`・
`bumps` と、そこに至る `ρ` の接頭が決める走査の状態 -- である。** よってこの補題は (f) を点ごとに
示し、`q` の入口の点までの制限の上で `q` についてこれらを読む。README の第 2 節が「**各時点についての
言明は、その時点までの接頭で読む。**「各実行路と、それを辿る各活性化の各時点について …」の形の言明 --
A19 (ii-a)、A19 (ii-b)、P14a、P18a、P18b、P18c、D11 の 3 つの節 -- は、時点 `τ` について `τ` までの量
だけを結ぶ」と定めるのがこの読みであり、README の P21 の脇が「位置ごとの帰納にするのはこのためであり、
各位置で (a) から `α` の許容性を出してから (b) を出す」と述べるのがこの形である -- README がそこで
「位置」と呼ぶのは、この文書の節点の入口の点である。

**証明**

<1>1. `cancel` の出力の各関数の `name`・`params`・`capture`・`borrowed_units` は、入力の同じ名前の関数の
      ものに等しい。とくに `B'` の所有と借用の割り当て (D14) は `B` のものと同じである。`cancel` は
      `prog.funcs.values()` の各 `f` について `f.clone()` を作って `clone.body` にだけ書き込み、鍵に
      `f.name.clone()` を据えるので、この 4 つは変わらない -- `RcFunc` は `#[derive(Clone)]` を持つので、
      EXT derive(Clone) より `f.clone()` の各欄は `f` の対応する欄の `clone` である。グローバル初期化子は
      パラメータも capture も持たない (D1)。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, D14, D1, EXT derive(Clone)
<1>1a. `Del` の節点はどれも `ρ` の終端の `Ret` より真に前にある。よって、`q_end` を除く各点に
       ついて、それに対応する点で `α'` は生きている (D23)。`q_end` では
       2 つの活性化はどちらも終わっており、D21 の制限はその点について何も課さない。
  BY L38, L37, L32, L30, L31, D23, D21, DEF 節点の入口の点
  L32 の 5 より `Del` の要素は `Retain` 節点か `Release` 節点だけである。`ρ` の上にある `Del` の
  `Retain` 節点は `CT` の要素であり (L32 の 1)、L38 より `t ∈ CT` が pending である区間 `I_ρ(t)` は
  `ρ` の上で `t` の直後から始まり、その最後の節点 `n*(ρ)` は終端の `Ret` より真に前にある (L38 の 2)。
  `ρ` の上にある `Del` の `Release` 節点は、L32 の 3 と L37 よりある `t ∈ CT` について `R_ρ(t)` の要素で
  あり、L38 の 3 より `R_ρ(t) ⊆ I_ρ(t)` なので `n*(ρ)` 以前にある。よってどの `Del` の節点も終端の
  `Ret` より真に前にある。L30 と L31 より `ρ'` の終端の `Ret` は `ρ` の終端の `Ret` と同じ節点であり、
  `α'` はその消費を行った時点で終わる (D23)。
<1>2. (a)、(b)、(c)、(d)、(e)、(f) を `ρ` の上の節点についての帰納法で示す。D3 より `ρ` は有限の列なので
      整礎である。各節点では (a) と (b) と (c) から (f) を出し、(f) の上で (d) と (e) を出し、(e) が
      次の節点の (a) を与える。
  <2>1. 基底。`ρ` の最初の節点は `B` の根であり、その入口の点は `α` が生きている活性化 (D23) になる点で
        ある。DEF 欠損 より `d = 0` である。どちらの活性化も
        まだ節点を実行しておらず、D29 より 2 つの活性化はパラメータと capture に対応する値を受け取るので
        (a) が成り立つ。D29 の第 2 行より最初の点では `k = 0` であり、2 つの活性化は対応する各計数下
        オブジェクトについて等しいカウントで始まるので (b) が成り立つ。D10 の初期値は所有する unit の
        下の inhabited な leaf で決まり、<1>1 より割り当ては同じなので `Obl'` は `Obl` に等しく、(c) が
        成り立つ。
    BY DEF 欠損, D29, D10, D23, <1>1
  <2>2. 帰納法の仮定: `ρ` の上の節点 `q` の入口の点について (a)、(b)、(c) が成り立つ。`q` の実行が
        終わった点を `q'` とする。基底の節点についてこれを与えるのは <2>1 であり、それ以外の節点に
        ついては、`ρ` の上でその直前にある節点についての <2>3・<2>4・<2>5 が与える。
    BY 帰納法の仮定, <2>1
  <2>2a. `α` は `q` の入口の点で D21 の制限を満たす。
    BY L43c, <2>2, <1>1a
    `q` の入口の点が `q_end` であるとき、<1>1a より D21 の制限はその点について何も課さず、言明は空虚に
    成り立つ。それ以外の点では <1>1a より `α'` は対応する点で生きている。L43b の仮定 -- `q` の入口の点
    までに 2 つの活性化が `Del` の節点を除いて同じ節点を実行し、その点までに値を得ている各変数の値が
    対応すること -- は <2>2 の (a) が与え、`H(q, O) = H'(q, O) + d(q, O)` は <2>2 の (b) が与える。
  <2>2b. (d) が `q` の入口の点で成り立つ。
    <3>1. CASE `q` が `ρ` の上の節点である。`d(q, O) ≥ 1` とする。L43 より `N(q, O) ≥ d(q, O) ≥ 1` で
          ある。L41 より `H(q, O) ≥ N(q, O) + 1 ≥ d(q, O) + 1` である。L41 を `α` について読むのに要る
          D21 の制限は <2>2a が与える。
      BY L41, L43, <2>2a
    <3>2. CASE `q` が `q_end` (終端の `Ret` の消費を行った直後の点) である。L43 の最後の文より
          `d(q_end, O) = 0` なので (d) は空虚に成り立つ。
      BY L43
    <3>3. QED
      BY <3>1, <3>2, DEF 節点の入口の点
      `DEF 節点の入口の点` より、この帰納が渡るのは `ρ` の上の節点の入口の点と `q_end` である。
  <2>3. CASE `q` の節点が `Del` の `Retain` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Retain` の行が、`π` の下の inhabited (D16) かつ計数下 (D26) の leaf ごとに 1 つの生成の素動作を
        行う (`DEF 節点の実行の素動作`)。`q` の実行の点 `τ` について、`τ` までにこの節点が `O` への参照を
        作った個数を `m_τ(O)` と書き、`m(O) := m_{q'}(O)` と置く。DEF 欠損 より
        `d(τ, O) = d(q, O) + m_τ(O)` である。`H` と `Obl` がそれぞれ `m_τ(O)` 増えることは、この節点の
        実行の素動作がその生成だけであることから出る (`<3>1`)。
    <3>1. この節点の実行の素動作は、D10 の `Retain` の行が定める生成だけである。
          この節点は参照を処分しないので、D24 の (F) の解放は起きず、解放が作る活性化も参照も無い。
          割り当ても伴わない -- D24 の (E2) の `H` の表で `Retain(v, π, s, k)` の行が述べるのは各 leaf の
          +1 だけである。`Retain` 節点は子の活性化を作る 3 種の段にも当たらない -- (E3) は `App` の節点、
          (E2) のうちオペランドを適用する `Llvm` の段は `Llvm` の節点であり、(E7) が走るのは D24 の
          「**(E7) グローバルの初期化の段。** まだ初期化されていないグローバル `g` を読む者が居るとき、
          `g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。読む者は、`g` を読む節点の位置に
          ある生きている活性化 `a` か、**環境**である」の形であって、D7 は `Retain` を読む構文に
          数えない。グローバル化は (E7) が作る活性化が終わるところにしか現れないので (`DEF 節点の実行の
          素動作`)、これも無い。
      BY D10, D24, DEF 節点の実行の素動作, D7
    <3>1a. `m(O) ≥ 1` である各計数下オブジェクト `O` について `H(q, O) ≥ 1` である。
      BY L41d, <2>2a, D16, D26
      `m(O) ≥ 1` であるのは、`π` の下の inhabited かつ計数下の leaf `λ` であって `obj(v, λ) = O` である
      ものが在るときである。L41d の「とくに」の節がその `O` について `H(q, O) ≥ 1` を与える。L41d を
      `α` について読むのに要る D21 の制限は <2>2a が与える。
    <3>2. `q` の実行の各点 `τ` で (a) が成り立ち、`τ` の点は 2 つの活性化で対応する。とくに `q'` で
          (a)、(b)、(c) が成り立つ。
      BY <2>2, <3>1, D10, D9, DEF 欠損, DEF 対応する活性化の量, L30, L31, L32, D29
      `q` は `Del` の `Retain` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
      しない。D9 の 2 つの表より `Retain` は値を作らず、移さず、手放さないので、`q` の実行の各点で値を
      得ている変数とその値は `q` の入口の点のものと同じであり、(a) が成り立つ。`DEF 対応する活性化の量`
      の点の対応より、`q` の実行の各点に対応する `α'` の点は `q` の入口の点に対応するものと同じである。
      D29 の第 2 行がその点の (b) を与える。`Obl` は D10 の `Retain` の行の分だけ増え、`α'` の側は
      変わらず、`d` も同じだけ増えるので (c) が `q'` で成り立つ。
    <3>3. (f) が `q` の実行の各点で成り立つ。
      BY L43c, <3>1, <3>2, <2>2a, <1>1a
      `q` の入口の点については <2>2a が与える。残る各点については、L43b の仮定を <3>2 の (a) が、
      `H(τ, O) = H'(τ, O) + d(τ, O)` を <3>2 の (b) が与え、`α'` が対応する点で生きていることを
      <1>1a が与える -- `q` は `Del` の節点なので `ρ` の終端の `Ret` より真に前にある。
    <3>4. (e) が成り立つ。
      <4>1. `q` の実行の各点 `τ` について、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。
        BY <3>1, <3>1a, <3>2, <2>2, <2>2b, DEF 欠損
        `H(τ, O) = H(q, O) + m_τ(O)`、`d(τ, O) = d(q, O) + m_τ(O)`、`H'(τ, O) = H'(q, O)` である
        (<3>1、DEF 欠損、<3>2)。`H(τ, O) = 0` ならば、`m_τ(O)` も `H(q, O)` も 0 以上なので
        `H(q, O) = 0` であり、<2>2 の (b) と <2>2b より `d(q, O) = 0` (`d(q, O) ≥ 1` なら
        `H(q, O) ≥ d(q, O) + 1 ≥ 2`) なので `H'(q, O) = 0`、すなわち `H'(τ, O) = 0` である。逆に
        `H'(τ, O) = 0` ならば `H'(q, O) = 0` である。<2>2 の (b) より `H(q, O) = d(q, O)` であり、
        `d(q, O) ≥ 1` ならば <2>2b の (d) が `H(q, O) ≥ d(q, O) + 1` を与えて矛盾するので
        `d(q, O) = 0` かつ `H(q, O) = 0` である。
        <3>1a より `m(O) ≥ 1` なら `H(q, O) ≥ 1` なので、`H(q, O) = 0` のとき `m(O) = 0`、したがって
        `m_τ(O) = 0` であり `H(τ, O) = 0` である。
      <4>2. QED
        BY <3>1, <3>2, <4>1, DEF 欠損
        (e) の点の対応と等式は <3>2 が、0 になることの同値は <4>1 が与える。`d(τ, O)` は DEF 欠損 の
        とおり `d(q, O) + m_τ(O)` である。
    <3>5. QED
      BY <3>2, <3>3, <3>4
  <2>4. CASE `q` の節点が `Del` の `Release` 節点である。`α'` はこの節点を実行しない。`α` では D10 の
        `Release` の行が、`π` の下の inhabited (D16) かつ計数下 (D26) の leaf ごとに 1 つの処分の素動作を
        行う (`DEF 節点の実行の素動作`)。`q` の実行の点 `τ` について、`τ` までにこの節点が `O` への参照を
        処分した個数を `c_τ(O)` と書き、`c(O) := c_{q'}(O)` と置く。DEF 欠損 より
        `d(τ, O) = d(q, O) - c_τ(O)` である。`H` と `Obl` がそれぞれ `c_τ(O)` 減ることは、この節点の
        実行の素動作がその処分だけであることから出る (`<3>1`)。
    <3>0. `q` の実行の各点 `τ` について `d(τ, O) ≥ 0` である。
      BY L43, DEF 欠損, D2
      `q` は `Release` 節点なので継続を 1 つ持ち (D2)、`q'` はその継続の節点の入口の点である。L43 より
      `d(q', O) ≥ 0` であり、DEF 欠損 より `d(τ, O) = d(q, O) - c_τ(O) ≥ d(q, O) - c(O) = d(q', O)` で
      ある。
    <3>0a. `Release` 節点は子の活性化を作る 3 種の段にも当たらず、グローバル化も伴わない。
      BY D24, D7, DEF 節点の実行の素動作
      (E3) は `App` の節点、(E2) のうちオペランドを適用する `Llvm` の段は `Llvm` の節点であり、(E7) が
      走るのは D24 の「**(E7) グローバルの初期化の段。** まだ初期化されていないグローバル `g` を読む者が
      居るとき、`g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。読む者は、`g` を読む節点の
      位置にある生きている活性化 `a` か、**環境**である」の形であって、D7 は `Release` を読む構文に
      数えない。グローバル化は (E7) が作る活性化が終わるところにしか現れない (`DEF 節点の実行の素動作`)。
    <3>1. この節点の実行はどのオブジェクトも解放しない。よってその素動作は D10 の `Release` の行が定める
          処分だけである。
      <4>1. この節点の実行が解放を起こすとすれば、その最初の解放について、`q` の実行のある点 `τ` と
            計数下オブジェクト `O0` があって、`c_τ(O0) ≥ 1`、`H(τ, O0) = 0`、かつ
            `H(τ, O0) = H(q, O0) - c_τ(O0)` である。
        BY D24, DEF 節点の実行の素動作, <3>0a, D7
        D24 の (F) より解放が起きるのは、参照を処分して計数下オブジェクトのカウントが 0 になったときで
        あり、D24 は「**解放は、それを起こした処分に付随して起きる。**」と述べる。最初の解放を起こす
        処分の直後の点を `τ`、そのオブジェクトを `O0` とすると `H(τ, O0) = 0` であり、その処分は
        この節点のものなので `c_τ(O0) ≥ 1` である。`τ` より前にこの節点の実行が持つ素動作は、`<3>0a` と
        `DEF 節点の実行の素動作` より D10 の `Release` の行が定める処分だけである -- 解放は `τ` が最初
        なので、それより前に解放の素動作は無い。よって `q` の入口の点から `τ` までの `H(・, O0)` の変化は
        この節点の処分の分だけであり、`H(τ, O0) = H(q, O0) - c_τ(O0)` である。
      <4>2. `d(q, O0) ≥ c_τ(O0) ≥ 1` である。
        BY <4>1, <3>0, DEF 欠損
        <3>0 より `d(τ, O0) = d(q, O0) - c_τ(O0) ≥ 0` である。
      <4>3. QED
        BY <4>1, <4>2, <2>2b, DEF 欠損
        <4>2 と <2>2b の (d) より `H(q, O0) ≥ d(q, O0) + 1 ≥ c_τ(O0) + 1` である。ところが
        `H(τ, O0) = H(q, O0) - c_τ(O0) = 0` より `H(q, O0) = c_τ(O0)` であり、矛盾する。よって <4>1 の
        形は起きない。解放が無いので、解放が作る活性化も参照もこの実行に無い。
    <3>2. `q` の実行の各点 `τ` で (a) が成り立ち、`τ` の点は 2 つの活性化で対応する。とくに `q'` で
          (a)、(b)、(c) が成り立つ。
      BY <2>2, <3>0a, <3>1, D10, D9, DEF 欠損, DEF 対応する活性化の量, L30, L31, L32, D29
      `q` は `Del` の `Release` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
      しない。D9 の 2 つの表より `Release` は値を作らず、移さず、手放さないので、`q` の実行の各点で値を
      得ている変数とその値は `q` の入口の点のものと同じであり、(a) が成り立つ。
      `DEF 対応する活性化の量` の点の対応より、`q` の実行の各点に対応する `α'` の点は `q` の入口の点に
      対応するものと同じである。D29 の第 2 行がその点の (b) を与える。`Obl` は D10 の `Release` の行の
      分だけ減り、`α'` の側は変わらず、`d` も同じだけ減るので (c) が `q'` で成り立つ。
    <3>3. (f) が `q` の実行の各点で成り立つ。
      BY L43c, <3>2, <2>2a, <1>1a
      `q` の入口の点については <2>2a が与える。残る各点については、L43b の仮定を <3>2 の (a) が、
      `H(τ, O) = H'(τ, O) + d(τ, O)` を <3>2 の (b) が与え、`α'` が対応する点で生きていることを
      <1>1a が与える -- `q` は `Del` の節点なので `ρ` の終端の `Ret` より真に前にある。
    <3>4. (e) が成り立つ。
      <4>1. `q` の実行の各点 `τ` について、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。
        BY <3>0, <3>1, <3>2, <2>2, <2>2b, DEF 欠損
        `H(τ, O) = H(q, O) - c_τ(O)`、`d(τ, O) = d(q, O) - c_τ(O)`、`H'(τ, O) = H'(q, O)` である
        (<3>1、DEF 欠損、<3>2)。`H(τ, O) = 0` とする。`c_τ(O) ≥ 1` ならば <3>0 より
        `d(q, O) ≥ c_τ(O) ≥ 1` であり、<2>2b の (d) より `H(q, O) ≥ d(q, O) + 1 ≥ c_τ(O) + 1` となって
        `H(q, O) = c_τ(O)` に反する。よって `c_τ(O) = 0` かつ `H(q, O) = 0` であり、<2>2 の (b) と
        <2>2b より `d(q, O) = 0`、したがって `H'(τ, O) = H'(q, O) = 0` である。逆に `H'(τ, O) = 0` なら
        `H'(q, O) = 0` である。<2>2 の (b) より `H(q, O) = d(q, O)` であり、`d(q, O) ≥ 1` ならば
        <2>2b の (d) が `H(q, O) ≥ d(q, O) + 1` を与えて矛盾するので `d(q, O) = 0` かつ
        `H(q, O) = 0` である。`H(τ, O)` は 0 以上で
        `H(q, O) - c_τ(O)` に等しいので `c_τ(O) = 0` であり、`H(τ, O) = 0` である。
      <4>2. QED
        BY <3>1, <3>2, <4>1, DEF 欠損
        (e) の点の対応と等式は <3>2 が、0 になることの同値は <4>1 が与える。`d(τ, O)` は DEF 欠損 の
        とおり `d(q, O) - c_τ(O)` である。
    <3>5. QED
      BY <3>0, <3>0a, <3>1, <3>2, <3>3, <3>4
  <2>5. CASE `q` の節点が `Del` に入らない。`ρ'` の対応する節点も同じ節点である (L30、L31)。
    <3>1. 2 つの実行はこの節点について D10 の同じ行を同じ値に対して適用し、この節点が束縛する変数に
          対応する値を置く。さらに、この節点が子の活性化を作るとき、その子の事象が参照カウントに与える
          変化は 2 つの実行で同じである。`d` は変わらない。
      <4>1. `q` は `Del` に入らないので、L30 と L31 より `ρ'` の対応する節点は `q` と同じ式の
            変位・変数・path・`RcState` を持つ。<2>2 の (a) より、その点までに値を得ている各変数の値は
            2 つの実行で対応する。
        BY L30, L31, <2>2
      <4>1a. `q` の式が `Let(_, RcRhs::Match(v, arms), k)` であるとき、2 つの活性化は同じアームへ進む。
        BY D21, D29, <4>1
        D21 は `Match` のアームを `v` の値の実行時のタグで決める。D29 の第 5 行は「**スカラの成分に
        ついては、対応は等号である。** boxed leaf でない成分 -- unbox union のタグ、整数、
        浮動小数 -- は、対応する 2 つの値で等しい。」と述べ、続けて「D21 は `Match` のアームを `v` の値の
        実行時のタグで決めるので、この節が、対応する 2 つの活性化が同じアームへ進むことを与える。」と
        述べる。<4>1 より `v` の値は 2 つの実行で対応するので、unbox union のタグは 2 つで等しい。
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
        <5>1. 2 つの実行の呼び出し先は、互いに対応する 2 つの版である。
          BY D23, D29, <4>1
          D23 より `Let(x, App(callee, args), k)` の呼び出し先は、その段で `callee` の値が指す関数で
          あり、`callee` の値がクロージャならその funptr の指す関数、funptr ならそれ自身である。<4>1 より
          `callee` の値は 2 つの実行で対応する。**その成分は等号では読めない** -- D29 の第 5 行は
          「**funptr の番地は等号では読めない** -- 2 つのプログラムは別々にコンパイルされるので、同じ
          関数の番地が同じ値になるとは限らない。」と述べ、続けて「**その成分については、対応する
          2 つの番地が対応する版を名指す**」と述べる。よって 2 つの番地は対応する版を名指す。
        <5>2. `B` の関数と `B'` の関数が対応する版であるならば、その 2 つは同じ `name` を持ち、その
              `params` と `borrowed_units` は等しい。
          BY <1>1, P24, CODE src/rc_ir/borrow.rs: cancel
          `cancel` は入力の各関数 `f` について `f.clone()` の `body` だけを差し替え、出力の `funcs` の
          鍵に `f.name.clone()` を据えるので、出力の版は入力の版と同じ `name` を持ち、対応はその名前で
          決まる。P24 は「**`cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。** とくに
          `borrowed_units` と `capture` は入力のものに等しい」と述べ、<1>1 が `params` と
          `borrowed_units` について同じことをこの文書の中で述べる。
        <5>3. QED
          BY <5>1, <5>2, D9, D14
          D9 の `App` の行は、callee の全 boxed leaf と、**呼び出し先がその位置の unit を所有する (D14)**
          引数の leaf を名指す。D14 の所有は `RcFunc::borrowed_units` が定めるので、<5>1 と <5>2 より
          2 つの実行で等しい。
      <4>4. QED
        BY <4>1, <4>1a, <4>2, <4>3, <4>3a, D9, D10, D16, A4, DEF 欠損,
           CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
        言明の前半は <4>2 と <4>1a であり、後半は <4>3 である。D10 の行が名指す leaf は、節点の形と値
        (D16、A4) と、`App` については呼び出し先の所有 (<4>3a) で決まるので、2 つの実行で同じである。
        `q` は `Del` に入らないので DEF 欠損 の 2 つの数え上げは変わらない。
    <3>2. `q` の実行の各点で 2 つの活性化の点が対応し、(f) が成り立ち、
          `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。`q` の
          入口の点と、その実行が終わった点 `q'` についても同じである。本場合の仮定より `q` は `Del` に
          入らないので、DEF 欠損 より `d(τ, O) = d(q, O)` である。すなわちこれは (e) と、(f) の
          「`q` の実行の各点で」の半分である。
      <4>1. `q` の入口の点について、点は対応し、`H'(q, O) = H(q, O) - d(q, O)` であり、`H(q, O) = 0` と
            `H'(q, O) = 0` は同値であり、(f) が成り立つ。
        BY <2>2, <2>2a, <2>2b
        <2>2 の (a) が点の対応を、<2>2 の (b) が等式を、<2>2a が (f) を与える。`d(q, O) = 0` のとき
        両辺は等しい。`d(q, O) ≥ 1` のとき <2>2b の (d) より `H(q, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、
        `H'(q, O) = H(q, O) - d(q, O) ≥ 1 > 0` なので、どちらも 0 でない。
      <4>2. `DEF 節点の実行の素動作` より、この節点の実行の素動作は 5 種のいずれかである。D10 の行が
            定める作成と処分、割り当て、子の活性化の素動作、(F) の解放、グローバル化である。素動作の
            順序はこの文書が定めず、以下の帰納は順序を使わない。本場合の仮定より `q` は `Del` に入らない
            ので、DEF 欠損 よりこの実行のどの点でも `d(τ, O) = d(q, O)` である。
        BY DEF 節点の実行の素動作, D10, D24, DEF 欠損, 本場合の仮定
      <4>3. この節点の実行の素動作の個数についての帰納法で、各点について、2 つの活性化の点が対応し、
            (f) が成り立ち、`H(τ, O) = 0` と `H'(τ, O) = 0` が同値であることを示す。
        <5>1. 基底は `q` の入口の点であり、<4>1 がこの 3 つを与える。
          BY <4>1
        <5>2. 対応が定まっている点 `τ` について、`τ` の次の素動作は 2 つの活性化で対応する。
          <6>1. CASE 次の素動作が子の活性化のもの、またはグローバル化である。<3>1 の後半より、この節点が
                作る子の活性化の事象が参照カウントに与える変化は 2 つの実行で同じである。D21 の第 4 行が
                それを活性化の側のデータとし、D29 の第 4 行が 2 つの活性化に同じものとして与える。
                グローバル化は (E7) が作る活性化が終わるところに現れ、その活性化の事象も同じ行が扱う。
            BY <3>1, D21, D29, D24
          <6>2. CASE 次の素動作が D10 の行が定める作成・処分、または割り当てである。<3>1 より 2 つの
                実行はこの節点について D10 の同じ行を同じ値に対して適用し、割り当てを伴う行 -- 単一の
                `Fresh` を宣言する `Llvm` の結果の leaf と `Closure(f, caps)` の結果 -- は 2 つの実行で
                同じである (D24 の (E2) の `H` の表)。**割り当てたオブジェクトが対応することは D29 の
                第 3 行が与える** -- 「D10 の生成の表が割り当てを行う位置 -- `Closure(f, caps)` の
                capture object と、`result_prov` が単一の `Fresh` を宣言する `Llvm` の結果の leaf --
                について、対応する位置が割り当てる 2 つのオブジェクトは対応する。」
            BY <3>1, D10, D24, D29
          <6>3. CASE 次の素動作が (F) の解放である。D24 の (F) より、解放されるのは
                その段が参照を処分してカウントが 0 になった計数下オブジェクトである。帰納法の仮定より
                `τ` で `H(τ, O) = 0` と `H'(τ, O) = 0` は同値なので、解放されるオブジェクトの集合は
                2 つの実行で対応する。解放が処分するのは、そのオブジェクトが保持する参照である
                (D24 の (F)、D25)。A5 より、その参照は inhabited であって計数下のオブジェクトを指す各
                boxed leaf に 1 つずつ在り、D29 の第 5 行より対応するオブジェクトが保持する値は対応
                するので、処分される参照は対応する。**`#ArrayStorage a` のオブジェクトは A5 の例外で
                あり、持ち手の単位は leaf ではなく要素の位置である** -- その時点の `size` 個の各位置が、
                要素の型の `boxed_leaf_paths` が列挙する leaf を 1 組ずつ持ち、解放の走査はそれを 1 つ
                ずつ処分する (A5、`CODE src/object.rs: ObjectFieldType::loop_over_array_buf`)。`size` が
                2 つの実行で等しいことは D29 の第 5 行の「スカラの成分については、対応は等号である」が
                与え、各要素の位置の leaf が対応するオブジェクトを指すことは同じ行の「対応する各位置に
                おいて、inhabited (D16) な各 boxed leaf が全単射で対応するオブジェクトを指し」が
                与える。
                `Destructor` のオブジェクトの解放が作る retain は `_dtor` の欄の値に当たり、その欄の値も
                D29 の第 5 行より対応する。その適用が作る活性化の事象が 2 つの実行で同じであることは
                <3>1 の後半が述べる -- D21 の第 4 行が (F) の作る活性化を活性化の側のデータに数え、
                D29 の第 4 行がそれを 2 つの活性化に同じものとして与える。
            BY 帰納法の仮定, <3>1, D21, D24, D25, D29, A5,
               CODE src/object.rs: ObjectFieldType::loop_over_array_buf
          <6>4. QED
            BY <6>1, <6>2, <6>3, <4>2
            <4>2 より、この節点の実行の素動作は 5 種のいずれかに属する。
        <5>3. `τ` の次の点で、2 つの活性化の点が対応し、(f) が成り立ち、`H = 0` と `H' = 0` が
              同値である。
          BY <5>2, L43c, L41c, D29, <2>2, <3>1, <4>2, <1>1a, 本場合の仮定
          <5>2 より次の点でも 2 つの活性化の点は対応し、D29 の第 2 行がその点で
          `H'(τ, O) = H(τ, O) - d(τ, O)` を与える。L43b の仮定 -- その点までに 2 つの活性化が `Del` の
          節点を除いて同じ節点を実行し、値を得ている各変数の値が対応すること -- は <2>2 の (a) と <3>1 が
          与える。その点が `q_end` -- `q` が `ρ` の終端の `Ret` であるときの `q'` -- であるときは、
          <1>1a より D21 の制限はその点について何も課さないので (f) は空虚に成り立つ。それ以外の
          点では <1>1a より `α'` は対応する点で生きているので、L43c より (f) が成り立つ。<4>2 より
          `d(τ, O) = d(q, O)` である。`d(q, O) = 0` の
          とき 2 つのカウントは等しい。`d(q, O) ≥ 1` のとき、本場合の仮定より `q` は `Del` に入らない
          ので L41c が当たり、`H(τ, O) ≥ d(q, O) + 1 ≥ 2 > 0` かつ
          `H'(τ, O) = H(τ, O) - d(q, O) ≥ 1 > 0` である。L41c を `α` について読むのに要る D21 の制限は
          この段が示した (f) である。
        <5>4. QED
          BY <5>1, <5>2, <5>3, D24, D2
          D24 の (F) より解放の連鎖は有限で終わり、D10 の各行が名指す leaf は有限個なので (D2 より本体は
          有限の木で、型の leaf は有限個である)、この節点の実行の素動作の個数は有限であり、この
          帰納法は整礎である。
      <4>4. QED
        BY <4>1, <4>2, <4>3
        <4>3 が各点について 3 つを与え、<4>2 が `d(τ, O) = d(q, O)` を与える。
    <3>2a. `q` の実行が `Obl` に与える変化は、D10 の行が定める分だけである。
      BY L39a, L39b, D10, DEF 節点の実行の素動作
      `DEF 節点の実行の素動作` より、この節点の実行の素動作は 5 種である。D10 の行が定める作成と処分は
      そのままであり、割り当てはその生成と同じ素動作である。子の活性化の素動作とグローバル化は L39b より
      `Obl` を動かさず、(F) の解放は L39a より `Obl` を正味で動かさない。
    <3>3. QED
      BY <3>1, <3>2, <3>2a, <2>2, D10, DEF 欠損
      <3>1 と <3>2a より、この節点の実行は 2 つの実行で `Obl` を D10 の行の分だけ同じく変えるので、
      <2>2 の (c) と合わせて (c) が `q'` で成り立つ。<3>1 より束縛する変数には対応する値が置かれるので
      (a) が `q'` で成り立つ。`q` は `Del` に入らないので `d` は変わらない (DEF 欠損)。(b) は <3>2 の
      `q'` の点についての等式であり、(e) と (f) の後半も <3>2 である。
  <2>6. QED
    BY <2>1, <2>2, <2>2a, <2>2b, <2>3, <2>4, <2>5, L43, L32, DEF 節点の入口の点
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかである
    (L32 の 5)。節点を持たない `q_end` については、L43 の最後の文より `d = 0` なので (b) から
    `H' = H` であり、(d) は <2>2b が、(f) は <2>2a が、(e) の 2 つの言明はその点について <2>2 の (b) と
    <2>2a が与える。
    **(f) の「`q` の入口の点までの各点」の半分は、節点についての帰納がそのまま与える。**
    `DEF 節点の入口の点` より、`q` の入口の点より前の各点は `ρ` の上で `q` より前にある節点の実行の点で
    あり、その節点についての <2>3・<2>4・<2>5 が (f) の「その節点の実行の各点で」の半分をそこで与えて
    いる。`q` の入口の点自身については <2>2a である。
<1>3. QED
  BY <1>1, <1>2, D3, D21, D29
  <1>2 が `ρ` の各節点について (a) から (f) を与える。D3 より `ρ` の節点は有限個なので、(f) を
  すべての節点について集めると `α` はその全点で D21 の制限を満たす。すなわち `α` は D21 の意味の
  活性化であり、D29 が「`α'` に対応する `B` の活性化はちょうど 1 つ存在する」と述べるのはこれに立つ。

## 4. P18c (義務集合の側の同じ不等式)

**言明 (README)** --- 走査中の各位置と各実行路について、各計数下オブジェクト `O` について
`Obl(O) ≥ n(O)` である。ここで `Obl(O)` は義務集合が持つ `O` への参照の個数、`n(O)` は P18a のものである。

**証明する形**。示すのは、**`borrow_ify` の出力**の各本体 `B` の各実行路 `ρ` と、それを辿る各活性化
(D21) と、`ρ` の上の各節点 `q` と各計数下オブジェクト `O` について `Obl(q, O) ≥ N(q, O)` である。
**入力を `borrow_ify` の出力に限るのは、P21・P23 と同じ理由による** -- `<1>1` が読む L42 は L41a を経て
A19 の (ii-a)・(ii-b) と P14a を活性化に当てるので、その範囲を出られない。A19 の (ii-b) の範囲は
「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体」、P14a の範囲は「`borrow_ify` の
出力の各本体」であり、この文書では第 1 節が `optimize_rc_program` からその限定を出している。
**この限定は主定理の鎖を切らない** -- P18c を読むのは `cancel` の健全性の鎖であり、その入力は
まさに `borrow_ify` の出力だからである。

活性化が D21 の制限 (A19 (i) の不等式) を満たすことは、D21 が制限を満たすものだけを活性化とすることから
出る -- ここで量化するのは `B` の活性化そのものであって、`L44` が構成する入力側の活性化ではない。

**証明**

<1>1. QED
  BY L42, DEF N, D27, D21
  走査中の位置は D27 に従って節点の訪問の入口であり、DEF N より P18a の `n(O)` はその位置の `N(q, O)` で
  ある。節点の入口は終端の `Ret` の消費より前にあるので L42 の範囲に入り、L42 の「とくに」の節が
  `Obl(q, O) ≥ N(q, O)` を述べる。L42 を読むのに要る D21 の制限は、量化する活性化が D21 の意味の活性化で
  あることから出る。

## 5. P19 (削除される retain の性質)

**言明 (README)** --- `cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含むすべての実行路に
おいて、`t` より後にある、**その位置での** `t` の `outstanding` の位置 (`VarPath`) を `acted_on` に含む
消費より前、かつ終端の `Ret` より前に、削除される `Release` 群が `t` の `outstanding` を空にする。さらに、
`t` とともに削除される各 `Release` は、実行路の上で `t` より後ろにある。

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
  <2>2. CASE `c` が D9 の消費の表の `App`、`Closure`、`Llvm` の行の節点である。この節点は右辺が `Match`
        でない `Let` 節点であり、`c` の訪問は `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
    <3>1. `consume_rhs` が `rhs_consumes` に渡す `owns(p, leaf)` は
          `self.owned_units.contains(&(p.name, truncate_to_unit(&p.ty, leaf, self.type_env)))` であり、
          `self.owned_units` は `all_owned_units(prog, type_env)` の値である。`all_owned_units` は各関数の
          各パラメータ・capture の各 unit のうち `borrowed_units` に入らないものを集めるので、この述語が
          真であることは、呼び出し先がその leaf の unit を所有する (D14) ことに等しい。unit は
          呼び出し先のパラメータの型 `p.ty` で取る。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/borrow.rs: cancel,
         CODE src/rc_ir/ownership.rs: all_owned_units, D14
    <3>2. `rhs_consumes` が `RcRhs::App(callee, args)` について報告する leaf の集合は、D9 の `App` の行が
          名指す leaf をすべて含む。`rhs_consumes` は callee の全 boxed leaf と、
          `resolve_callee_params` が返すパラメータについて `owns` が真である引数 leaf を報告し、
          `resolve_callee_params` が `None` を返すときは `is_owning_position` を無条件に真とする。
          A14 より `params[i]` は範囲内である。callee の全 boxed leaf は D9 の `App` の行の第 1 項その
          ものなので、残るのは引数の leaf である。
          - `resolve_callee_params` が `Some` を返す場合。<3>1 より `owns` が真であることは、呼び出し先が
            その leaf の unit を所有する (D14) ことに等しい。D9 の `App` の行が読む所有は実行時の呼び出し先 (D23) の
            ものなので、静的に解決した関数がその呼び出し先であることが要る。`cancel` は `borrow_ify` の
            出力を入力に取るので (第 1 節)、それを与えるのは P30 -- 「`borrow_ify` の出力の
            `Let(x, App(callee, args), k)` について、`resolve_callee_params` が解決する関数が `Some` で
            あるならば、それはその段の実行時の呼び出し先 (D23) と同じ `RcFunc` である」-- である。P30 は
            続けて「`cancel` の中で `CancelAnalysis::consume_rhs` が `rhs_consumes` を呼ぶ位置がこれを
            読む」と、この節点を名指す。**P29 は `borrow_ify` の入力についての命題なので、ここには
            当たらない。** よってこの場合、報告する集合は D9 の行が名指す集合に等しい。
          - `resolve_callee_params` が `None` を返す場合。報告するのは全引数の全 boxed leaf であり、
            D9 の行が名指す集合はその部分集合である。**2 つは等しいとは限らない** -- A7 は
            「`prog.funcs` に無い呼び出し先は、全パラメータの全 unit を所有するものとして扱われる。これは
            所有を増やす向きの近似である」と述べ、その近似の差がここに出る。D9 の行が読むのは実行時の
            呼び出し先が所有する位置だけなので、実行時の呼び出し先が借用する unit の引数 leaf は D9 の
            行に無く、`rhs_consumes` の報告には在る。
          **この段が与えるのは包含だけである。** `<3>5` が使うのは「`c` で消費される leaf は
          `rhs_consumes` がこの節点で報告する」という向きだけであり、逆向きは要らない。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: resolve_callee_params, D9, D23, A7, A14, P30, <3>1
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
      <3>4 よりそれは `rhs_consumes` がこの節点で報告する leaf である。L36 の 1 より `c` の訪問は
      `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
  <2>3. CASE `c` が D9 の消費の表の `Destructure` の 2 行の節点である。`destructure_consumes(container,
        fields, type_env)` は、容器が boxed のとき容器の全 boxed leaf を、unbox のとき名前の付いていない
        フィールドの leaf を返す。これは D9 の `Destructure` の 2 行そのものである。L36 の 2 より、`c` の
        訪問はその各 leaf `μ` について `consume_objects(pending, acted_on(container.name, μ))` を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes, D4, D9, L36
  <2>4. CASE `c` が D9 の消費の表の「本体 (D23) の終端の `Ret(x)`」の行の節点である。L38 より `t` は `ρ` の
        終端の `Ret` では pending でないので、仮定に反する。よってこの場合は起こらない。
    BY L38
  <2>5. QED
    <2>2 と <2>3 の呼び出しは、`I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に
    在るところで走る。仮定よりその `objects` は `out(t, c)` が名指す名前を含むので、L38 の 4 に反する。
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

**証明する形**。入力プログラムを `P` と書く。**`P` は `borrow_ify` の出力であり、D12 を満たすとする。**
`ρ'` を `B'` の実行路、`α'` をそれを辿る `B'` の活性化、`ρ` を `ρ'` に対応する `B` の実行路、`α` を
D29 が `α'` に対応させる `B` の活性化とする。`k(O)` を `Retain` の個数ではなく参照の個数で数えることが
要る理由は、第 11 節が反例で述べる。

**この 2 つが要る所は次のとおりである。** `borrow_ify` の出力であることは `<1>1` と `<1>1a` が読む
`L44` に要る -- `L44` が読む `L41`・`L41c` が P18a を、`L41c` が読む `L41b`・`L41a` が A19 の
(ii-a)・(ii-b) と P14a を `α` に当てるので、その範囲が要る。この文書では `B` が `borrow_ify` の出力の
本体であることを第 1 節が `optimize_rc_program` から出しており、この節はそれを言明の側にも書き出す。
D12 は README の言明が置く前提であり、(a) と (b) の導出はどの段もそれを読まない -- `DEF 解放されている`
が「その時点かそれより前の時点でカウントが 0 である」の形で解放を定めるので、(b) はカウントが後から
上がる場合も覆う。**D12 を読むのは第 9 節の `<1>3` である** -- そこで `α` について D11 の 3 つを得る。

**`α` が D21 の制限を満たすことも、この命題の証明が示す** (README の P21 の脇)。`<1>1a` がそれである。

**証明**

<1>1. (a) が成り立つ。
  BY L44, L43
  L43 の最後の文より P21 (a) の `k(O)` は DEF 欠損 の `d(q, O)` であり、L44 の (b) がその等式である。
<1>1a. `α` は D21 の制限 -- A19 (i) の不等式 -- をその全時点で満たす。すなわち `α` は D21 の意味の
       活性化であり、D29 が「`α'` に対応する `B` の活性化はちょうど 1 つ存在する」と述べるのはこれに
       立つ。
  BY L44, D21, D29
  L44 の (f) が点ごとにこれを示し、L44 の `<1>3` がそれを `ρ` の全点について集める。
<1>2. 計数下のオブジェクトについて、2 つの活性化は同じ点で同じオブジェクトを解放する。ここで `O` は
      D29 の全単射が対応させるオブジェクトを渡り、D29 の第 5 行よりその定義域は 2 つの活性化がそれぞれ
      到達できる (D25) オブジェクトの全体なので、どちらかの活性化が解放しうる計数下オブジェクトはすべて
      `O` の範囲に入る。
  <2>1. `ρ` の上の節点 `q` の実行の間に `α` が `O` を解放するのは、その実行の間のある点 `τ` で
        `H(τ, O) = 0` となり、`q` の入口の点ではそうでないときである。`α'` についても `H'` で同じで
        ある。
    BY D7, DEF 解放されている, DEF 節点の入口の点, D29, D25
  <2>2. QED
    BY <2>1, L44
    L44 の (e) より、`q` の実行の各点 -- `q` の入口の点自身を含む -- で `H(τ, O) = 0` と
    `H'(τ, O) = 0` は同値である。よって <2>1 の条件は 2 つの活性化で同時に成り立つ。
<1>3. グローバル状態のオブジェクト (D26) は 2 つの活性化のどちらでも解放されない。
  BY D26, A8
<1>4. (b) が成り立つ。
  BY <1>2, <1>3, DEF 解放されている, L44, L30, L31, D29, D26
  DEF 解放されている より、`α` の点 `τ` で `O` が解放されているとは、`H(τ0, O) = 0` を満たす `τ` 以前の
  点 `τ0` が在ることであり、`α'` については `H'` で同じである。L44 の (e) をその各 `τ0` に当てると、
  この 2 つは同値である。計数下でないオブジェクトについては <1>3 が両方とも解放されないことを与える。
  D29 の第 5 行より計数下かグローバル状態か (D26) の区別は対応する 2 つのオブジェクトの間で一致するので、
  この場合分けは 2 つの活性化で同じに分かれる。
  L30 と L31 より `ρ'` の各節点は `ρ` の対応する節点と同じ式の変位・変数・path を持ち、
  L44 の (a) よりその節点で名指す変数の値も対応するので、読む構文とその読む値は 2 つの実行で対応する。
<1>5. QED
  BY <1>1, <1>1a, <1>4

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

**(S-c) は接頭条件つきの節である。** D11 の (S-c) は「**その活性化がその時点まで解放について閉じている
(D11a) とき**、D7 の読む構文がその位置で読みうる各オブジェクト、および `Retain(v, π)` と `Release(v, π)`
が触れる (D7) 各オブジェクトは、**その読み・その触れる動作の直前の点で**解放されていない」と述べる。
よって `α'` について示すのは条件つきの言明であり、`<1>7` が置いてよい仮定は「`α'` がその時点まで閉じて
いる」である。`α` の側の (S-c) も同じ形で条件つきなので、`α'` の条件から `α` の条件を出す段が要る --
`<1>7` の `<2>2` がそれである。**「直前の点」は節点の入口の点ではないので、そこで使う対応は節点の
粒度の P21 (b) ではなく、点の粒度の `L44` の (e) と、D24 の「**読みの直前の点では、勘定は直前の段内の点の
ものである。**」の節である。**

**証明**

<1>1. `P'` の所有と借用の割り当て (D14) は `P` のものと同じである。
  <2>1. `cancel` は `prog.funcs.values()` の各 `f` について `let mut clone = f.clone();` を作り、
        `clone.body` にだけ書き込んで `(f.name.clone(), clone)` を `funcs` に入れる。よって
        `borrowed_units`、`params`、`capture` は変わらない -- `RcFunc` は `#[derive(Clone)]` を持つので、
        EXT derive(Clone) より `f.clone()` の各欄は `f` の対応する欄の `clone` である。グローバル初期化子は
        パラメータも capture も持たない (D1)。
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, D1, EXT derive(Clone)
  <2>2. QED
    D14 の割り当ては `RcFunc::borrowed_units` が定めるので、<2>1 より変わらない。
    BY D14, <2>1
<1>2. `P'` の各関数の本体と各グローバル初期化子の `init` は、`P` の対応するものに `cancel_body` を適用した
      ものであり、DEF 出力の本体 の `B'` である。
  BY CODE src/rc_ir/borrow.rs: cancel, DEF 出力の本体
<1>3. `ρ'` に対応する `B` の実行路 `ρ` と、`α'` に対応する `B` の活性化 `α` が定まる。`α` は D21 の
      制限を満たし、`ρ` と `α` について D11 の (S-a) と (S-b) が成り立ち、(S-c) が D11a の接頭条件つきで
      成り立つ -- すなわち `α` が時点 `τ` まで解放について閉じている (D11a) ならば、`τ` の読み・触れる
      動作の直前の点で、それが読みうる・触れるオブジェクトは解放されていない。
  BY D12, D11, D11a, D21, L31, L32, L44, DEF 路の対応, D29
  L32 の 5 より `Del` は `Retain`/`Release` 節点だけなので L31 が使え、`ρ'` に対応する `B` の実行路 `ρ` が
  ちょうど 1 つ定まる。D29 は `α'` に対応する `B` の活性化がちょうど 1 つ存在することを述べ、`α` が
  D21 の制限をその全時点で満たすことは L44 の (f) が示す (L44 の `<1>3`)。**D11 と D12 が課す条件は
  D21 の意味の活性化についてのものであり、D21 は制限を満たすものだけを活性化とするので、この引用が要る**
  -- 制限を満たさない対は D11 の範囲に入らない。`P` が D12 を満たすので、`B` のすべての実行路と、
  それを辿るすべての活性化について D11 の 3 つが成り立ち、`α` はその 1 つである。(S-c) の節が接頭条件を
  持つことは D11 の本文が述べる。
<1>4. `ρ'` の各節点の入口の点において `Obl'(q, O) = Obl(q, O) - d(q, O)` であり、`d(q, O) ≥ 0` で
      ある。
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
  <2>2. CASE それ以外。その操作が節点 `q` の実行の中で `O` への参照を `c` 個取り除くとする。
        `DEF 節点の実行の素動作` より、その操作は D10 の行が定める処分の素動作であり、それらを施した
        直後の点 `τ` について `Obl(τ, O) = Obl(q, O) - c` である。この場合 `q` は `ρ` の終端の `Ret`
        ではなく、D3 と L33a より終端の `Ret` は `ρ` の最後の節点なので、`τ` は終端の `Ret` の消費より
        前にある。よって L42 より `Obl(τ, O) ≥ b(τ, O)` である。L42 を `α` について読むのに要る、`α` が
        D21 の意味の活性化であること -- L42 が読む L41a が A19 の (ii-a)・(ii-b) と P14a を `α` に
        当てるので要る -- は <1>3 が与える。
    BY L42, DEF 節点の実行の素動作, D10, D3, L33a, <1>3, D21
  <2>3. その `τ` について `b(τ, O) ≥ d(q, O)` である。
    BY L43a, <2>1
    <2>1 より `q` は `Del` に入らないので L43a が使え、`τ` は `q` の実行の点である。
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3, <1>4
    <2>1a 以外の操作について、<2>2 と <2>3 より `Obl(q, O) - c = Obl(τ, O) ≥ d(q, O)` なので
    `Obl'(q, O) = Obl(q, O) - d(q, O) ≥ c` である。すなわち取り除かれる参照は `Obl'` に入っている。
    (S-a) が見る操作は D10 が `Obl` から参照を取り除くと定めるもの -- D9 の消費の表の 6 行と `Release`
    節点 -- であり、そのうち終端の `Ret` を <2>1a が、残りを <2>2 と <2>3 が扱う。
<1>6. (S-b) が `ρ'` で成り立つ。
  <2>1. `ρ` の終端の `Ret` では、`CT` の要素はどれも pending でない。
    BY L38
    L38 より各 `t ∈ CT` が pending である区間は `n*(ρ)` で終わり、`n*(ρ)` は終端の `Ret` より真に前に
    ある。
  <2>2. 終端の `Ret` の入口の点と `q_end` で `d = 0` である。
    BY <2>1, L43, L32, DEF 欠損
    L43 より `d` は `CT` の pending な要素の `B_ρ` の和であり、<2>1 よりその和は空である。終端の `Ret` は
    `Del` に入らない (L32 の 5 より `Del` は `Retain` と `Release` だけ) ので、その消費の後も `d` は
    変わらない。
  <2>3. QED
    BY <1>3, <2>2, L44, L30, L31
    <1>3 の (S-b) より `ρ` の終端の `Ret` の消費の後の `Obl` は空である。L44 の (c) と <2>2 より
    その点の `Obl'` は `Obl` に等しいので、`Obl'` も空である。L30 と L31 より `ρ'` の終端の `Ret` は
    `ρ` の終端の `Ret` と同じ節点であり、L44 の (a) より消費する値も対応する。
<1>7. (S-c) が `ρ'` で成り立つ。すなわち `α'` が `ρ'` の上の点 `p'` まで解放について閉じている
      (D11a) とき、`p'` で起きる読み・触れる動作の直前の点で、それが読みうる・触れる各オブジェクトは
      解放されていない。
  <2>1. `ρ'` の読む構文 (D7) と `Retain`/`Release` 節点は、`ρ` のそれらから `Del` の節点を除いたもので
        あり、対応する各節点で名指す変数と path は同じである。L44 の (a) よりその変数の値も対応し、
        D29 の第 5 行より対応する値の inhabited な各 boxed leaf は全単射で対応するオブジェクトを指す
        ので、読みうるオブジェクトと触れるオブジェクトは 2 つの実行で対応する。
    BY L30, L31, L32, L44, D7, A5, D29
  <2>1a. `α` の各点 `τ` と、それに対応する `α'` の点について、各計数下オブジェクト `O` が `τ` で
         解放されている (DEF 解放されている) ことと、対応する点で `O` が解放されていることは同値で
         ある。
    BY L44, DEF 解放されている, DEF 節点の入口の点, D29, D26, A8
    L44 の (e) は、`ρ` の上の各節点の実行の各点 -- `q` の入口の点とその実行が終わった点を含む --
    について、2 つの活性化の点が対応し `H(τ, O) = 0` と `H'(τ, O) = 0` が同値であることを述べる。
    **これは段内の点の粒度の言明である** -- (e) が量化するのは節点の実行の各点であって、節点の入口の点
    だけではない。DEF 解放されている は「`τ` かそれより前の時点で参照カウントが 0 であること」を解放されて
    いることと定めるので、`τ` 以前の各点にこの同値を当てれば、2 つの活性化で解放されているオブジェクト
    は対応する。`O` はどちらの活性化のものも D29 の全単射の定義域に入る (D29 の第 5 行)。計数下でない
    オブジェクトはどちらの活性化でも解放されない (D26、A8)。
  <2>1b. **(S-c) が見る「直前の点」でも、この同値が成り立つ。** その点は一般に段内の点ではないが、D24 が
         その点の勘定を直前の段内の点のものと定める。
    BY D24, <2>1a
    D24 は「**読みの直前の点では、勘定は直前の段内の点のものである。** D11 の (S-c) は「その読み・その
    触れる動作の直前の点」で条件を課すが、読みは 6 種の素動作のどれでもないので、その点は一般に段内の点
    ではない。」と述べ、続けて「**その点と直前の段内の点のあいだに素動作は 1 つも無いので、`H` も `Obl` も
    `held` (D34) も動かず、解放も起きない。** よって段内の点について示した勘定は、その点へそのまま
    移る。**この節が無いと、読みの直前の点を扱う証明ファイルがそれぞれ同じ橋を自分で架けることになる。**」
    と書く。`H` が動かず解放も起きないので、その点で `O` が解放されていることは、直前の段内の点でそう
    であることと同じであり、<2>1a がその点についてこの同値を与える。**この橋を自分で架けてはならない** --
    この文書の点集合は D24 の段内の点であり (`DEF 実行時の量`)、読みの直前の点はそこに入らない。
  <2>2. `α'` が点 `p'` まで解放について閉じている (D11a) ならば、`α` は `p'` に対応する `α` の点まで
        閉じている。
    BY <2>1a, D11a, D26, DEF 解放されている
    D11a は、時点が解放について閉じているとは、その時点で `H(O) ≥ 1` である各計数下オブジェクト `O` が
    その時点で解放されていない (D24 の (F)) ことと定め、`τ` まで閉じているとは `τ` 以前の各時点が
    閉じていることと定める。D11a のこの「解放されていない」と DEF 解放されている の否定が同じものを
    指すことは、DEF 解放されている の「**これは D11a が読む「解放されている」と同じものを指す。**」の節が
    述べる。`α` の時点 `σ` と `H(σ, O) ≥ 1` である計数下 `O` を取る。<2>1a の同値 (`H = 0` と `H' = 0`)
    より対応する時点で `H'(σ, O) ≥ 1` であり、`α'` はその時点まで閉じているので `O` はそこで解放されて
    いない。<2>1a より `O` は `σ` でも解放されていない。よって `σ` は閉じている。
  <2>3. QED
    BY <1>3, <2>1, <2>1a, <2>1b, <2>2
    <2>2 より、`α'` が `p'` まで閉じているとき `α` は対応する点まで閉じているので、`<1>3` の (S-c) の
    接頭条件が満たされ、`ρ` の対応する読み・触れる動作の直前の点で、読みうるオブジェクトと触れる
    オブジェクトは解放されていない。<2>1 よりその読み・触れる動作とその対象は 2 つの実行で対応し、
    <2>1b よりその点で解放されていることも 2 つで同値なので、`ρ'` の側でも解放されていない。
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
- **`cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。** とくに `borrowed_units` と `capture` は
  入力のものに等しい。`borrow_ify` は `borrowed_units` を書くので、この節は `cancel` についてだけで
  ある。**この節は P14b の結論を運ばない** -- どの種の段がどの本体の活性化を作るかは実行の上の言明で
  あり、欄と本体の一致からは出ない。それは P14b が自分の範囲に `cancel` の出力を入れて述べる。
- **本体について書き換えが変えるのは、`Retain`/`Release` の節点と、`App` の callee の名前だけである。**
  節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の op とオペランド・
  `Destructure` のフィールドは、いずれも元の本体のものに等しい (複製の名前替えを P9 で戻したうえで)。

**証明**

<1>1. `roots` は変わらない。`borrow_ify` の返す `RcProgram` の `roots` は `prog.roots.clone()` であり、
      `cancel` の返す `RcProgram` の `roots` も `prog.roots.clone()` である。
  BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: cancel
<1>2. `cancel` の出力の各関数は、入力の関数の `clone()` に `body` だけを書き込んだものである。`RcFunc` は
      `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の各欄は原本の対応する欄の `clone` であり、
      `fn_ty`、`ret_ty`、`params`、`inline_into_callers` は変わらない。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, EXT derive(Clone)
<1>2a. `cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。とくに `borrowed_units` と `capture` は
       入力のものに等しい。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, D1, P14b, EXT derive(Clone)
  D1 より `RcFunc` は `name`・`fn_ty`・`params`・`capture`・`ret_ty`・`body`・`source`・
  `borrowed_units`・`inline_into_callers` の 9 個の欄を持つ。`cancel` は `prog.funcs.values()` の各 `f`
  について `let mut clone = f.clone();` を作って `clone.body` にだけ書き込み、`(f.name.clone(), clone)` を
  出力の `funcs` に入れる。`RcFunc` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の各欄は
  原本の対応する欄の `clone` であり、残る 8 個は入力のものに等しい。`borrow_ify` は `borrowed_units` を書く
  (<1>3 の `<2>1` のループ) ので、この節は `cancel` についてだけである。
  **この節は P14b の結論を運ばない。** P14b が述べる「借用する unit を持つ本体の活性化を作る段は (E3) に
  限る」は実行 (D24) の上の言明であり、欄と本体の一致からは出ない。P14b は `cancel` の出力を自分の範囲に
  入れて述べる。
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
         CODE src/rc_ir/borrow.rs: expr_node, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
         CODE src/rc_ir/borrow.rs: prepend_rc, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, D11, D12
      `RcExpr` の 6 変位のうち `Let` を右辺で 3 つに分けた 8 つの場合を <3>1 から <3>5 が尽くす。これは
      `rewrite_inner` の `match` の 8 つの腕である。どの腕も、入力の節点に対応する節点を `expr_node` で
      作って `&node.source` を据え、継続には `self.rewrite` の値を置くので、節点の並びは元の並びである
      (`<3>4` の借用版が `rewrite_rc` で作る unit ごとの節点も、その節点の `source` を据える)。節点を
      足すのは <3>1 と <3>4 だけであり、足す節点はいずれも `Retain` か `Release` である。**`<3>1` の
      `prepend_rc` が足す節点の `source` は `None` である** (`CODE src/rc_ir/borrow.rs: prepend_rc` --
      `rc_node(is_release, var, path, RcState::Unknown, cont, &None)`)。P24 の第 5 の箇条が数えるのは
      節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の op とオペランド・
      `Destructure` のフィールドであって `source` ではなく、D11 と D12 も `RcExprNode` の `source` を
      読まないので、この違いは言明に触れない。節点を落とすのは <3>4 の借用版
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
  BY <1>1, <1>2, <1>2a, <1>3, <1>3a, <1>3b, <1>4, <1>5
  第 1 の箇条は <1>1、第 2 の箇条は <1>2 と <1>3、第 3 の箇条は <1>4 と <1>5、第 4 の箇条は <1>2a、
  第 5 の箇条は <1>3a (`borrow_ify`) と <1>3b (`cancel`) である。

## 11. P21 (a) の `k(O)` を参照ごとに数える理由

README の P21 (a) は `k(O)` を、削除される `Retain` が `O` に作った参照のうち対になる削除される
`Release` がまだ処分していないものの**個数**として書く。`Retain` の個数で数える形が偽であることを、
本体で示す。**この節は反例だけからなり、ほかの節が読む結論を持たない。** 第 0 節の表の 7 命題はすべて、
冒頭が挙げるものの上で閉じている。

**`Retain` の個数で数える形は 2 通りに読め、どちらの読みでも (a) が偽になる。** 1 つの削除される
`Retain` `t` と対になる削除される `Release` は 1 つとは限らない。`L38` の 3 が `R_ρ(t)` と書く集合で
あり、`Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` を満たす。`|R_ρ(t)| ≥ 2` であって、その一部だけが
実行済みである点で、`Retain` を数える形は `t` を数えるとも数えないとも読める。

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
`O1` へ 1 つずつ参照を作る (`O0 ≠ O1` である活性化を取る)。`Release(f0, [])` の継続の節点 -- すなわち
`Release(f1, [])` -- の入口の点を `q` と取ると、`H_α(O0) - H_{α'}(O0) = 0` (処分済み)、
`H_α(O1) - H_{α'}(O1) = 1` (未処分) である。

- **「対になる削除済みの `Release` を 1 つも実行していない `t` を数える」と読む場合。** `t` は
  `Release(f0, [])` を実行済みなので数から落ち、`k(O1) = 0` になる。差は 1 なので **(a) が偽である。**
- **「対になる削除済みの `Release` を全部は実行していない `t` を数え、その `t` が `O` に作った参照を
  全部数える」と読む場合。** `k(O0) = 1` になる。差は 0 なので **(a) が偽である。**

**README の P21 (a) は参照ごとに数える形である。**

> `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` が `O` に作った参照のうち、その `Retain`
> と対になる削除済みの `Release` がまだ処分していないものの個数である。

この形では、上の本体の点 `q` で `k(O0) = 0`、`k(O1) = 1` であり、どちらも差に一致する。

**この形は証明が出す量である。** `L43` の <1>8 が、`DEF 欠損` の `d(q, O)` -- 削除される `Retain` が
作った個数から削除される `Release` が処分した個数を引いたもの -- がこの形の量に等しいことを示す。
`L44` の (b) が `H'(q, O) = H(q, O) - d(q, O)` を与えるので、P21 (a) はそのまま出る。

**対になることは `L32` の 3 が与える。** `Del` の各 `Release` はちょうど 1 つの `t ∈ CT` の
`un_bump_releases[t]` に属するので、「その `Retain` と対になる `Release`」は 1 つに定まる。
