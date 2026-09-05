# P18c, P19, P20, P21, P22, P23, P24 -- `cancel` が RC 規律を保存すること

この文書は README の 7 命題 P18c, P19, P20, P21, P22, P23, P24 を証明する。README の命題については
その**言明**だけを使う。**この文書が立つ README の定義・仮定・命題の一覧はここに書かない** --
`python3 dev-docs/proof/proof_index.py --depends dev-docs/proof/rc_ir/borrow-cancel/p40-cancel-soundness.md`
が引用のグラフからそれを出す。主定理 T は `p70-main-theorem.md` の担当であり、この文書は扱わない。

この文書が読んだコードのコミットは `d9b200c8cf12fdf02790d19b5f6c8c4ed9562617` である。README が証明の
対象として名指すコミット `b6c51fb892746e493e155d9d59ea05d02d7357db` との間で `src/` に入った変更は、
README の第 1 節が挙げる 2 種である -- 「**対象コミットより後に `src/` へ入った変更は 2 種である。**
1 つは各証明が引く記号に付く `// PROOF:` コメントで、`dev-docs/proof/proof_links.py` が生成する。
もう 1 つは `Validator::check_rhs` が `result_prov` の宣言する source の個数を数える検査であり、A3 の
「1 つの結果 leaf に 2 つ以上の source を宣言しない」を果たす者を与える。どちらも
`borrow_ify`・`cancel`・`ownership.rs` の振る舞いを変えない。」

引用する外部の命題は 2 つのファイルにある。`p30-cancel-walk.md` の `L1` (`walk` と `rewrite` は内側を
1 回呼ぶ)、`L5` (`un_bump` の作用)、`L6` (消費の作用)、`L10` (記録は増えるだけ)。
`p13-disposals-and-pending.md` の第 7 節の局所の定義 -- `DEF 実行時の作用` (`Inh_ρ`、`ActRefs^inh_ρ`)、
`DEF 名前の活性` (`obj_ρ`)、`DEF 名前をオブジェクトへ写す` (`(・)^obj`)、`DEF bump の帰属` (`B_ρ`)、
`DEF N` (`N_ρ`) -- と、命題 `L7`
(boxed leaf の路は反鎖をなす)、`L9` (`identity` は inhabited を決める)、`L10a` (静的な数え上げと実行時の
作用が活性な名前で一致する)、`L10c` (`ActRefs^inh_ρ` の名前は位置であり、写した先が実行時の作用で
ある)、`L8a` (`origin` の memo を使わない展開は有限である)、`L11` (非活性な名前では `B` は空)、
`L11a` (`InBracket` の引き算は切り捨てを起こさない)、`L14` (`identity` は自分の別名類の
位置である)、`L17`
(`N` は別名類ごとの `bumps` の和である)。
これらは `p30 の L10`、`p13 の L17` のようにファイル名を添えて引用する。

**外部の結果**は `EXT <名前>` の名札で引く。この文書が据えるのは Rust の言語規則と標準ライブラリ、および
`dyn_clone` crate の 10 個で、その完全な言明は次のとおりである。Rust の言語規則の 4 つは Rust Reference の
原文を引く。

- **EXT 可視性と私有性** --- Rust Reference の "Visibility and Privacy" が次を述べる。

  > By default, everything is *private*, with two exceptions: Associated items in a `pub` Trait are
  > public by default; Enum variants in a `pub` enum are also public by default.

  > With the notion of an item being either public or private, Rust allows item accesses in two cases:
  >
  > 1. If an item is public, then it can be accessed externally from some module `m` if you can access
  >    all the item's ancestor modules from `m`. You can also potentially be able to name the item
  >    through re-exports.
  > 2. If an item is private, it may be accessed by the current module and its descendants.

  同じ節が `pub(crate)` について次を述べる。

  > `pub(crate)` makes an item visible within the current crate.

  すなわち、`pub` の付かない項目 -- 自由関数、型、inherent な `impl` の中のメソッド、構造体の
  フィールド -- を名指せるのは、それを宣言したモジュールとその子孫のモジュールの中だけである。
  `pub(crate)` の付いた項目を名指せるのは、そのクレートの中だけである。
- **EXT モジュールは `mod` が導入する** --- Rust Reference の "Modules" が次を述べる。

  > A module is a container for zero or more items.

  > A *module item* is a module, surrounded in braces, named, and prefixed with the keyword `mod`. A
  > module item introduces a new, named module into the tree of modules making up a crate.

  > Modules can nest arbitrarily.

  すなわち、1 つのファイルの本体が `mod` の項目を 1 つも書かなければ、そのファイルのモジュールは子孫の
  モジュールを持たない。EXT 可視性と私有性 と合わせると、そのファイルが宣言する非公開の項目を名指せるのは
  そのファイルの中だけである。書かれた `mod` の項目が `#[cfg(test)] mod tests` の 1 つだけであれば、
  子孫はそのモジュールだけである。
- **EXT trait の実装は既定と再定義で尽きる** --- Rust Reference の "Implementations" が次を述べる。

  > An implementation is an item that associates items with an *implementing type*.

  > A trait implementation must define all non-default associated items declared by the implemented
  > trait, may redefine default associated items defined by the implemented trait, and cannot define
  > any other items.

  すなわち、既定の本体を持つ trait のメソッドについて、ある型のためにそれが実行する本体は、その型の
  `impl Tr for T` ブロックがそのメソッドを再定義していればその定義、していなければ trait 側の既定の
  本体である。よって、その本体の全体は、trait の既定の本体と、そのメソッドを再定義する `impl` の
  一覧とで尽きる。
- **EXT 共有参照は代入を許さない** --- Rust Reference の "Pointer types" が共有参照について次を述べる。

  > When a shared reference to a value is created, it prevents direct mutation of the value. Interior
  > mutability provides an exception for this in certain circumstances. As the name suggests, any
  > number of shared references to a value may exist. A shared reference type is written `&type`, or
  > `&'a type` when you need to specify an explicit lifetime.

  すなわち `&T` を通じて `T` の欄へ代入することはできない。`&T` から値を動かす道として残るのは、その型が
  `UnsafeCell` を通じて持つ内部可変性だけである。
- **EXT Arc の割り当ての安定性** --- `std::sync::Arc<T>` の値は、制御ブロックと `T` を 1 つのヒープ
  割り当ての中に置く。`Arc::as_ref`(`Deref`) が返す `&T` の番地はその割り当ての中の `T` の番地であり、
  同じ割り当てを指す `Arc` から何度取っても等しい。`Arc` の値を move してもその番地は動かない。割り当てが
  解放されるのは最後の強参照が落ちたときであり、共有参照が生きている間は落ちない。
- **EXT Vec::clone** --- `Vec<T>` (`T: Clone`) の `clone` は、長さが原本と等しく、第 `i` 要素が原本の
  第 `i` 要素の `clone` である新しい `Vec` を返す。要素の並びは原本のものである。
- **EXT derive(Clone)** --- 構造体または列挙型に `#[derive(Clone)]` を付けて得られる `clone` は、原本と
  同じ変位の値であって、その各欄が原本の対応する欄の `Clone::clone` である値を返す。欄を落とすことも、
  並べ替えることも、別の値を置くこともない。
- **EXT 標準の型の clone は等しい値を返す** --- `usize` と `bool` は `Copy` であり、その `clone` は
  原本そのものを返す。`String` の `clone` は原本と同じ文字列を持つ新しい `String` を返す。
  `Vec<T>` (`T: Clone`) については EXT Vec::clone のとおりである。
  `std::collections::HashMap<K, V, S>` (`K: Clone`、`V: Clone`、`S: Clone`) の `clone` は、鍵の集合が
  原本の鍵をそれぞれ `clone` したものからなり、各鍵の値が原本の値の `clone` である写像を返す。
  タプルの `clone` は、各成分が原本の対応する成分の `clone` であるタプルを返す。`Option<T>`
  (`T: Clone`) の `clone` は、`None` には `None` を、`Some(x)` には `Some(x.clone())` を返す。
  したがって、これらの型を組み合わせた値を `PartialEq` が成分ごとに比べる限り、`clone` の値は原本と
  等しい。
  `rustc_hash::FxHashMap<K, V>` は `HashMap<K, V, FxBuildHasher>` の別名であり、`crate::misc::Map<K, V>`
  はさらにその別名である (`CODE src/misc.rs: Map`)。
- **EXT Iterator::map と collect** --- `Vec<T>` の `iter()` が返す反復子は、その要素を先頭から順に
  1 つずつ渡す。`Iterator::map(f)` が返す反復子は、元の反復子と同じ個数の要素を
  同じ順に渡し、その第 `i` 要素は元の第 `i` 要素に `f` を当てた値である。その反復子を
  `collect::<Vec<_>>()` で集めると、長さがその個数に等しく、第 `i` 要素がその第 `i` 要素である `Vec` が
  得られる。
- **EXT dyn_clone の trait object の複製** --- `dyn_clone` crate は、その `DynClone` を継承する trait
  `Tr` について `impl Clone for Box<dyn Tr>` を与える (`clone_trait_object!(Tr)`)。その `clone` は中身の
  具体型の `Clone::clone` を呼び、その値を新しい `Box` に置いて返す。**複製が原本と同じオブジェクトで
  あるとは限らない。**

`B_ρ` の個数が 0 以上であることは README の P18b が言う -- P18b は `B(p, ρ)` を「`p.node` の `Retain` が
`ρ` で実際に作った参照のうち、`ρ` 上でまだ処分されていないものを、それを作った leaf の `origin` の
identity で名付けた多重集合」と定める。**`outstanding` に
ついて使うのは `covers`、すなわち `outstanding[o] ≥ B_ρ(・, ・)[o]` の向きだけである。**

別名類 (`obj(C)`、`T_ρ(C)`) は **D33**、類ごとの参照 `held_ρ` は **D34** であり、どちらも `README.md` に在る。
**この文書が ρ-歩み・ρ-終端と書くのは、D33 の `ρ` 歩み・`ρ` 終端である。** D33 は「**この 3 つの語も
ここにしかない**」と定めるので、指すものはそこで定まる 1 つである。

この文書が導入する命題は `L30` から番号を付ける。`p30` と `p13` の命題の番号と衝突させないためである。
**番号は固定された名札であって、ファイルの中の並びではない。** 命題どうしの引用に循環は無い。

## 0. 到達した所

| 命題 | 結果 |
|---|---|
| P18c | 証明済み (`L42`)。A19、P14a、`p13` の `L17` を読む。入力は `borrow_ify` の出力に限る |
| P19 | 証明済み |
| P20 | 証明済み |
| P21 | 証明済み。(a) は `L43` と `L44` の (b) に、(b) は `L44` の (e) に載る。`α` が D21 の制限を満たすことは `L44` の (f) が示す |
| P22 | 証明済み |
| P23 | 証明済み。(S-a) は `L41c` の 1 に、(S-b) は `L44` の (c) に、(S-c) は `L44` の (e) に載る。(S-c) は D11a の接頭条件つきで示す |
| P24 | 証明済み。第 5 の箇条は `rewrite_inner` の 8 腕についての構造帰納で出る |

**各段が依拠するのは、冒頭が挙げる README の定義・仮定・命題と、第 1 節が挙げる他のファイルの命題と
定義 -- `p30-cancel-walk.md` の 4 つの命題と、`p13-disposals-and-pending.md` の `DEF` と命題 -- と、
そこに据えた外部の結果 (`EXT`) と、引用したコードと、第 2 節が名前つきで置く前提だけである。**
その前提は 4 種である。1 つは **`borrow_ify` が A19 の (ii-c) を保つ
こと**であり、第 2 節の `前提 (ii-c) の保存` が置き、果たす者を `p20-borrow-ify.md` の第 13 節と書く。
1 つは第 2 節の `在りかの前提` が置く、コードのどこに何が在るかの前提であり、果たすのはそこに
書いた `SCAN` の走査である。残る 2 つは **D29 が `α` に与える与件の読み** -- 第 2 節の
`前提 対応は素動作の粒度で読む` と `前提 記号の位置の値の対応` -- であり、どちらも枠 (D29) に置くのが
本来である。**P18c・P21・P23 は
入力を `borrow_ify` の出力に限る** -- `L41a` が読む A19 (ii-a)・(ii-b)・(ii-c) と P14a の範囲がそこだから
であり、第 4 節・第 7 節・第 9 節の「証明する形」がその限定を書き出す。README の P18c・P21・P23 の言明も
同じ限定を持つ。

**点はこの文書が定めるものではない。** 点は D24 の段内の点であり (`DEF 実行時の量`)、この文書は
`DEF 節点の入口の点` でその上に節点ごとの記法を置くだけである。**走査の状態は点ではなく節点の訪問に
付く** (`DEF 節点の実行の素動作`) -- D27 が `B(p, ρ)` を節点の訪問の入口で定めるからであり、
`bumps_ρ` と `b` はその節点を引数に取る。

**`α` の参照カウントの推移は、D29 の第 2 行が `α'` のそれに欠損 `k` を足したものとして与える。**
`L44` の (b) と (e) の等式はその与件を読んだものであり、`L44` が示すのは 2 つの活性化の点が対応する
ことの方である。対応するオブジェクトが保持する値が対応することは D29 の第 5 行と A4 が、子の活性化を
作る段が参照カウントに与える変化を活性化の側のデータとすることは D21 の第 4 行が与える。

**A19 (i) は仮定ではなく、D21 が活性化に課す制限である。** `α'` は `B'` の活性化なのでそれを満たし、
`α` がそれを満たすことは `L44` の (f) が点ごとに示す。移し替えを行うのは `L43b` と `L43c` であり、
`α` についてこの制限を読むのは `L41`・`L41c`・`L41d` である。

## 1. 記法

**DEF 本体ごとの記法** --- 1 つの関数 (またはグローバル初期化子) の本体 `B` を固定し、
`B` から作られる `VarTable` を `vars`、
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

**DEF 名前とオブジェクト** --- D15 と `References` が「オブジェクト」と呼ぶのは `VarPath` の
値であり、この文書ではこれを**名前**と呼び、`o` で表す。D7 が「オブジェクト」と呼ぶのは実行時のヒープの
オブジェクトであり、この文書ではこれを**オブジェクト**と呼び、`O` で表す。名前 `o` が実行路 `ρ` で
**活性**であること、および活性な名前が指すオブジェクト `obj_ρ(o)` は、`p13` の `DEF 名前の活性` が
定める。活性な名前と `ρ` の上のスロットの対応、および README の P18a の `obj(o)` がこの `obj_ρ(o)` で
あることは `L45` が述べる。

多重集合の記法。`References` は `Map<VarPath, usize>` を 1 つ持つ構造体であり
(`CODE src/rc_ir/ownership.rs: References`)、これを鍵を名前、値をその個数とする多重集合とみなす。
和 `R1 + R2`、差 `R1 - R2` を各鍵の個数の和・差とし、`R[o]` を名前 `o` の個数とする。

`cancel` の入力は `borrow_ify` の出力である。`optimize_rc_program` が `borrow_ify` の返り値を `cancel` に
渡し (`CODE src/build/build_object_files.rs: optimize_rc_program`)、`cancel` と `borrow_ify` はどちらも
`pub(crate)` なのでクレートの外から呼べない (`CODE src/rc_ir/borrow.rs: cancel`,
`CODE src/rc_ir/borrow.rs: borrow_ify`、EXT 可視性と私有性、P15)。よって本体 `B` について、A19 の
「`borrow_ify` がそれを写した各本体」の側と P14a が使える。

## 2. 局所の定義

### DEF 訪問

`walk_inner` の 1 回の呼び出しを**訪問**と呼び、その `node` 引数が指す節点を訪問した、という
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。節点 `n` の訪問における `pending` 引数の値
(その訪問がそれに変更を加える前の値) を `pending(n)` と書き、**入口状態**と呼ぶ。その訪問の戻り値を
`pending_out(n)` と書き、**出口状態**と呼ぶ。

節点 `n` の**継続終端** `ret(n)` を、`n` から D2 の意味の継続 (`Match` の場合はアーム本体ではなく `k`) を
たどって到達する `Ret` 節点とする。`ret` が `Ret` 節点に与える値はその節点自身である。

要素 `e ∈ pending(n)` について、`e.node` を `NodeId` に持つ `Retain` 節点を `e` の**由来**と呼ぶ。
`Retain` 節点 `t` が `pending(n)` の要素の由来である
とき、「`t` は `n` で pending である」といい、その要素を `e_t(n)`、その `outstanding` を `out(t, n)` と
書く。

この 3 つの記法が 1 つに定まることは `L50` が述べる。

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
ついての `N(c)` との合併とする。根でない各節点が親をちょうど 1 つ持ち、`n` がどの子の部分木にも入らない
ことは `L50` が述べる。

### DEF 節点の量

`Retain` 節点 `t = Retain(v, path, _, _)` と `Release` 節点 `r = Release(v, path, _, _)` について、

- `ActRefs(t) :=` `self.acted_references(v, path)` の値、`ActRefs(r) :=` `self.acted_references(v, path)`
  の値 (`CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references`)。D15 より、これは
  `acted_references(vars, type_env, v, path)` すなわち `L(v, path)` の各 leaf を `id` で名付けて数えた
  多重集合である。
- `others(r) :=` `self.other_objects(v, path)` の値の元の集合
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。

この 3 つの値が走査のどの段階で読んでも同じであることは `L46` が述べる。

### DEF 削除集合

`cancel_body` の 1 回の実行について、`analysis.cancelled()` が返す集合を `Del` と書く
(`CODE src/rc_ir/borrow.rs: cancel`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled`)。
`self.all_retains` の要素のうち、`self.needed_retains` に入らず、`self.un_bump_releases` の値が空でない
ものの全体を `CT` と書く。走査の終わりの `self.un_bump_releases` を `un_bump_releases` と書く。

以下では `Del`、`CT`、`un_bump_releases` の要素を節点と同一視する。この同一視が定まることは `L50` が
述べる。

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
**点**と書くのはこの段内の点であり、`τ`、`p` で表す。

**時点は段内の点の一部である。** D24 は「段と段のあいだの時点はその段の最初の段内の点であり、直前の段の
最後の段内の点である。」と述べるので、D24 の意味の各時点はこの文書の点であり、逆は成り立たない。README が
「各時点と各段内の点」と 2 つを並べるところを、この文書は後者だけで読む -- 前者は後者に含まれるので、
段内の点について示した言明は時点についても成り立つ。**時点についてしか言えない言明を段内の点へ広げる
向きは、この読みからは出ない** -- A19 の (ii-a)・(ii-b) がその形であり、下の節がその範囲を書く。

6 種の素動作は D24 が挙げる「参照の受け渡し、生成、割り当て、処分、解放、グローバル化」である。切れ目を
除くのは 2 つだけであり、D24 は「**切れ目を除くのは、上の 2 つの節だけである。** 処分とそれが起こす解放の
あいだ、および素動作とそれに付随する書き込みのあいだには点が無い。**それ以外の切れ目を束ねてはならない。**」
と書き、「とくに**割り当てと、割り当てたオブジェクトの欄を埋める受け渡しのあいだには点が在る。**」と
続ける。**この文書はこの 2 つ以外の切れ目を束ねない。**

**活性化 `α` の点**とは、`α` が生きている (D23) 間の各点である。`α` が入れ子の呼び出しで中断中の間の点も
`α` の点である -- D23 は「**生きている活性化**とは、始まって終わっていない活性化である」と定める。

`α` の各点 `τ` とオブジェクト `O` について、`H(τ, O)` をその点の `O` の参照カウント (D7)、`Obl(τ, O)` を
その点の義務集合 (D10) が持つ `O` への参照の個数とする。**`H` は活性化の側のデータである** -- D21 は
「活性化はさらに、各時点の各計数下オブジェクトの参照カウント `H` を持つ。」と述べ、続けて「`H` も活性化の
側のデータである。」と書き、その増減のうち別の制御の流れの段から来る分はこの活性化の外から来ると述べる。
`B'` の活性化 `α'` については `H'`、`Obl'` と書く。

**D21 の制限は各点で読む。** D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を
満たすものに限る。**」と述べ、続けて「**粒度が段内の点までなのは、`held` (D34) がその粒度で定まり、
P28 (b) がその粒度でこの制限を示すからである**」と書く。この文書の「点」は D24 の段内の点なので (上記)、
D21 の制限はこの文書の各点に掛かる。

**A19 の (ii-a) と (ii-b) を読める点はこれより狭い。** A19 は「**「各時点」は、その活性化が生きている
(D23) 間の、その活性化の節点の訪問の入口である時点である。**」と述べ、その理由を「`bumps` を定める D27 が
`B(p, ρ)` を走査中の位置 -- 節点の訪問の入口 -- でしか定めないので、この 2 つを読める点はそこに限る。」と
書く。さらに「**入れ子の呼び出しで中断中の時点も、その活性化の節点の入口であるものは含む。**」と続ける。
この文書ではその点は `DEF 節点の入口の点` の入口の点である。**段内の点で言えるのは A19 の (ii-c) の
非負性までである** -- A19 は「**(ii-b) は段内の点では読めない。** 右辺の `bumps` は節点ごとに定まり
(D27)、左辺の `held` は節点の実行の途中で下がりうるので、入口で成り立つことは途中の点で成り立つことを
与えない。」と述べる。

### DEF 節点の入口の点

`ρ` の上の節点 `q` について、`q` の実行が始まる直前の点を `q` の**入口の点**と呼び、そこでの量を
`H(q, O)`、`Obl(q, O)`、`d(q, O)`、`N(q, O)` と書く。`q` の実行が終わった点を `q'` と書く。
`q` が `ρ` の終端の `Ret` であるときは、`q'` を `ρ` の**最後の点** `q_end` と呼ぶ -- その
`Ret` の消費 (D9) を行った直後の点である。`ρ` の上の入口の点が有限個であることは `L50` が述べる。

**`q` の実行の点**とは、`q` の入口の点から `q'` までの各点をいう (両端を含む)。

**`q'` と、`ρ` の上で `q` の次にある節点の入口の点は、同じ点とは限らない。** その 2 つのあいだに在る点を
**節点の間の点**と呼ぶ。`L50` の 4 より、そこで `α` は素動作を持たず、挟まる段はどれも `α` が実行する段
ではない -- 環境の段と、別の制御の流れの活性化の段である。**`ρ` の上の点は、節点の実行の点と節点の間の
点で尽きる。** 節点の間の点を越える対応は `L44g` が扱う。

**`q'` は `q` の実行が終わるときにだけ定まる。** 停止しない呼び出し先を持つ `App` の節点の実行は
終わらず (D24 は実行を「段の有限または無限の列」と定める)、そのとき `q'` も、`ρ` の上で `q` より後に
ある節点の入口の点も `α` に無い。**以下、`q'` と `q` より後の節点について述べる言明は、その点が `α` に
在るときの言明である。**

**この文書は時刻を「位置」と呼ばない。** 時刻は上の「点」であり、`ρ` の上の節点はその入口の点で代表
する。**「位置」と書くのは D6 の意味 -- スロットと記号の位置 -- のときだけである。** 本体の木の場所に
ついては D2 が「**この文書では、本体の木の位置を「節点」と呼び、位置が相異なれば節点も相異なるものと
する。**」と定めるので、この文書も「節点」と書く。P15 と D30 の言明を引くところ、および README の A19 と
P21 の脇を引くところだけは、引く側の語である「位置」をそのまま写す。

### DEF 節点の実行の素動作

**節点 `q` の実行に属する段**を次で定める -- `α` が `q` の位置で実行する段、`q` の実行が作る活性化と
その子孫の活性化が実行する段、および `q` の中で D24 の (E7) が作る初期化子の活性化が終わるところで走る
(E5) の段である。節点 `q` の実行に属する段が `q` の実行の点で行う素動作 (D24) のうち、次の 6 種に
それぞれ名前を与える。**この 6 種が `q` の実行に属する素動作を尽くすことは `L47` が示す。**
**`q` の実行の点には、`q` の実行に属さない段 -- 別の制御の流れの活性化の段と、環境の段 -- の素動作も
在りうる。** `q` が子の活性化を作る節点であるあいだ `α` は中断中であり、A17 (iii) はその間に環境の段が
並びうると述べる。その素動作をこの 6 種は数えず、それが `Obl(α)` と `held_ρ` を動かさないことを
`L49` の 2 が述べる。

**`α` の素動作**とは、`α` が実行する段が行う素動作と、`α` が作った活性化とその子孫の活性化が実行する段が
行う素動作と、`α` の中で D24 の (E7) が作る初期化子の活性化が終わるところで走る (E5) の段が行う素動作を
いう。すなわち、この定義が「節点 `q` の実行に属する段」として挙げる 3 種を、`ρ` の全節点について集めた
ものである。それが `ρ` の上のどれかの節点の実行に属する段のものであることは `L47` の 3 が述べる。

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
- **op の生成コードが出す retain と release**。D24 は「**この網羅は段の境界についてである。** 1 つの段の
  生成コードは、この表に行を持たない素動作を段の中で出しうる。**素動作の粒度で勘定する段は、その op の
  `generate` が出す retain と release を読む。**」と述べ、「**在りかは述語で決める** --
  `Generator::retain`・`Generator::build_retain`・`Generator::release` の
  呼び出しを出す生成コードの全体であり、一覧で書くと op が 1 つ増えるたびに古くなる。」と続け、
  「**述語は名前の綴りでなく、呼ばれる項目で書く。**」「形は 2 つに分かれる。」と置く。
  **その 2 つの形は `Obl(α)` について別に扱う。** **この 2 つは (E2) の段が中で出す素動作の分類で
  あり、環境が段そのものとして行う (E9) の retain は入らない** -- D24 は「**この 2 つの形は (E2) の
  段が中で出す素動作の分類である。(E9) の retain はどちらでもない。**」と述べる。(E9) が節点の実行の
  素動作でないことは `L47` の 1 が述べる。
  - **段の中で相殺するもの**。D24 は「`InlineLLVMWithRetainedFunctionBody` はオペランドを retain し、
    適用の後に release する。段の出口では相殺するので表には現れないが、段内の点では見える。」と述べ、
    「**その参照の持ち手は、その段を実行している活性化である** (D25 の 1 番目) -- retain と適用のあいだの
    段内の点で `Obl(a)` に在り、適用がそれを呼び出し先へ渡し、release が元の分を処分する。
    **素動作の粒度で `Obl` を勘定する段は、この 2 つを数える。**」と続ける。すなわち retain は
    `Obl(α)` に参照を 1 つ加える生成であり、適用はそれを呼び出し先へ渡す受け渡しであり、release は
    D9 の `Llvm` の行の消費を行う処分である。**この 3 つを合わせた `Obl(α)` の正味の動きは、D10 の行が
    定める分に等しい。**
  - **相殺しないもの**。D24 は「複製を作る腕が複製の欄へ retain する形である
    (`make_struct_union_unique` の共有の腕、`clone_array_buf`)。」とし、「**その参照の持ち手は、その
    生成コードが書き込むオブジェクトの持ち手の
    単位である** (D25 の 2 番目)。」と述べ、「**書き込む先が新しいか既存かをこの節は問わない。**」と
    続ける。**持ち手が `Obl(α)` ではないので、この素動作は `Obl(α)` を動かさない。**

**素動作の順序をこの文書は定めない。** D24 は各段の素動作を「時間の順に並べたとき」の列として扱うだけで、
1 つの節点の中で処分と作成のどちらが先かを定めない。以下の議論は順序を使わず、素動作の列の上の帰納で
進む。

**解放の内側の点。** ある処分から、それに付随して走る (F) の解放の連鎖が終わるまでのあいだの点を、
**解放の内側の点**と呼ぶ。D24 は「**この節が縛るのは、解放の始まりである。**」に続けて「**解放の内側には
段内の点が在る。**」と書くので、この点は在りうる。`Std::FFI::Destructor` でないオブジェクトの解放は
活性化を作らないので、その解放の内側に点は無い。

**走査の状態との対応。** **走査の状態は点ではなく節点の訪問に付く。** `ρ` の上の節点 `n` を取り、`n` の
実行の各点 `τ` について、**`n` の下で `τ` に対応する走査の状態**を `n` の入口状態 `pending(n)`
(`DEF 訪問`) と定める。**この置き方は D27 が定める** -- D27 は「**節点の実行の途中の点 (D24 の段内の点)
における `B(p, ρ)` は、その節点の訪問の入口における値である。** 走査は節点を単位に進むので、1 つの節点の
実行の途中に対応する走査の状態は無い。」と述べる。走査が節点を単位に進む以上、`B` の値だけでなく
`pending` の元も節点の入口のものを取るほかにない。よって `bumps_ρ` と `b` (`DEF 類ごとの義務`) は、
`n` の下では `n` の実行のどの点でも `n` の入口の点の値である。

**1 つの点が 2 つの節点について読まれることがある。** `q` が `ρ` の終端の `Ret` でなければ、`q'` と
`ρ` の上で `q` の次にある節点 `m` の入口の点のあいだに `α` の素動作は無い (`L50` の 4)。その範囲の点で
`B_ρ` を読む段は、`q` の下では `pending(q)`、`m` の下では `pending(m)` を取る。**どちらを取るかは、
その量を読む段が節点を名指して決める** --
`L41e` が点 `p(q)` と一緒に節点 `n(q)` を定めるのはこのためであり、`L41a`・`L41b`・`L42`・`L43a` は
`bumps_ρ` と `b` を節点を引数にして書く。

**`pending_out(q)` はこの対応に現れない。** `L33` が `pending_out(n) = pending(ret(n))` を示すとおり、
それは `n` の**継続終端**の入口状態であって、`ρ` の上で `n` の次にある節点の入口状態ではない。
`Retain(v,[],s, Let(z, Llvm(zero,[]), Release(v,[],s, Ret(z))))` の `Let` 節点がその 2 つを分ける --
その継続終端は `Ret(z)` であり、`Release` の `un_bump` がそこまでに `v` の要素を取り除くので、
`pending_out` はその要素を持たず、`ρ` の上で次にある `Release` 節点の入口状態はそれを持つ。

### DEF 対応する活性化の量

**D29 の箇条は本文の並び順で数える。** D29 の本文は「次の 4 つの行」と書きながら箇条を 5 つ持つので、
この文書が「D29 の第 `n` 行」と書くときの `n` は、その並びの第 `n` 箇条を指す。第 1 行は「入力の束縛
(D23) が受け取る値は対応する」、第 2 行は参照カウントの推移、第 3 行は割り当ての位置、第 4 行は子の
活性化を作る段の結果、第 5 行は「**対応するオブジェクトが保持する値は対応する。**」である。

対応する活性化 `α`、`α'` を固定したとき、`H'(τ, O)`、`Obl'(τ, O)` は、`α'` について `τ` に対応する点で
読んだ値とする。`q ∈ Del` のときは `ρ'` にその節点が無いので、`q` の実行の各点に対応する `α'` の点は、
`ρ'` の上で `q` の直後にあたる節点の入口の点である。**`H` と `H'` を 1 つの `O` について並べて書くとき、
2 つの活性化のオブジェクトを結ぶのは D29 の全単射である。** `O` はその全単射が対応させるオブジェクトを
渡る。D29 の第 5 行より、その定義域は、対応する 2 つの活性化がそれぞれ到達できる (D25) オブジェクトの
全体である。

**2 つの活性化の点の対応。** 2 つの活性化は同じ点で始まり、対応する節点の実行が起こす素動作を順に
突き合わせて、点を 1 対 1 に対応させる。`Del` の節点の実行が作る点は `α` の側にしかないので、`α'` の側の
値として上の約束の点の値を取る。**この対応が `α` のどの点についても定まることは `L44` が示す** -- (a) が
2 つの活性化が `Del` の節点を除いて同じ節点を実行することを、(e) が 1 つの節点の実行の中で 2 つが同じ
素動作の列を持つことを与える。

**その対応の上で `H` と `H'` の関係を与えるのは D29 の第 2 行である。** D29 は「`α` の参照カウントの
推移は、`α'` のそれに欠損 `k` (P21 (a)) を足したものとして与える」と述べ、その `k` は `DEF 欠損` の `d`
である (L43)。すなわち `L44` の (b) と (e) の等式は、証明が素動作ごとに積み上げる量ではなく、D29 が
`α` に与えるデータである。

### DEF 対応の 3 つ

対応する活性化 `α`、`α'` (D29) を固定する。`α` の点 `τ` について、次の 3 つが `τ` で成り立つことを、
`τ` で**対応の 3 つ**が成り立つ、という。`L44` の (a)・(b)・(c) を 1 つの点について読んだものである。

- **(a)** `α` と `α'` は `τ` までに、`Del` の節点を除いて同じ節点を実行し、`τ` までに値を得ている各変数の
  値は、D29 の全単射のもとで対応する。**対応する 2 つの値の Fix の関数型の成分は、入力と出力の対応する
  版を名指す。** ここで入力の関数と出力の関数が**対応する版**であるとは、P24 の第 2 の箇条が言う
  「**出力の各関数は入力のちょうど 1 つの関数から作られ**」の関係にあること、すなわち `cancel` が入力の
  `f` から作った出力の関数であることをいう。`L44d` より対応する 2 つの版は同じ `name` を持つので、
  `prog.funcs` の同じ鍵が対応する版を引く。
- **(b)** 各計数下オブジェクト `O` について `H'(τ, O) = H(τ, O) - d(τ, O)` である。
- **(c)** 各計数下オブジェクト `O` について `Obl'(τ, O) = Obl(τ, O) - d(τ, O)` である。

`τ` が `ρ` の上の節点 `q` の入口の点であるとき、`H(τ, O)`・`Obl(τ, O)`・`d(τ, O)` は
`DEF 節点の入口の点` の `H(q, O)`・`Obl(q, O)`・`d(q, O)` である。`α'` の側の量は
`DEF 対応する活性化の量` が定める。

### DEF 解放されている

オブジェクト `O` が活性化のある点 `τ` (`DEF 実行時の量`) で**解放されている**とは、`τ` かそれより前の
点で `O` の参照カウントが 0 であること、すなわち `H(τ0, O) = 0` を満たす `τ` 以前の点 `τ0` が在ることを
いう。D7 は参照カウントが 0 になったオブジェクトが解放されると定めるので、これは D7 の「解放される」を
点の言葉で書いたものである。`α'` についても `H'` で同じように定める。

**量化を点で置くのは、この語を読む段がどれも段内の点で読むからである。** `DEF 実行時の量` は
「**時点についてしか言えない言明を段内の点へ広げる向きは、この読みからは出ない**」と定めるので、
時点で量化した語をそのまま段内の点で読むことはできない。時点は段内の点の一部なので (`DEF 実行時の量`)、
点で定めたこの語は時点についても読める。

**この語を使うのは、`O` を割り当てた素動作 (D24) より後の点についてだけである。** それより前の点で
`O` は生きていない (D25)。環境が持ち込んだオブジェクトは実行の最初の時点から生きている (D25)。

この語が D11a の読む「解放されている」と同じものを指すことは `L48` が述べる。

### DEF 欠損

対応する活性化を固定する。`α` の点 `τ` と計数下 (D26) のオブジェクト `O` について、

`d(τ, O) :=` (`τ` より前に実行された `Del` の `Retain` 節点の素動作が `O` への参照を作った個数)
`-` (`τ` より前に実行された `Del` の `Release` 節点の素動作が `O` への参照を処分した個数)

を**欠損**と呼ぶ。D10 の `Retain` と `Release` の行が、作られる個数と処分される個数を定める。節点 `q` の
入口の点での値を `d(q, O)` と書く。

`d` が `ρ` の上のどの節点の実行の中で動くかは `L47` の 2 が述べる。`d(q, O)` が README の P21 (a) の
`k(O)` であることは `L43` が述べる。`k(O)` を `Retain` の個数ではなく参照の個数で数えることが要る理由は
第 11 節が反例で述べる。

### DEF N

対応する活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト `O` について、

`N(q, O) := Σ_{p ∈ pending(q)} Σ_{o} B_ρ(q, p)[o]`

と定める。内側の和は、`ρ` で活性であって `obj_ρ(o) = O` である名前 `o` を渡る。`B_ρ(q, p)` は D27 の
`B(p, ρ)` を節点 `q` の訪問の入口で読んだものであり、`p13` の `DEF bump の帰属` が同じ規則を表で書く。

この量が README の P18a の `n(O)` であり `p13` の `DEF N` の `N_ρ(q, O)` であることは `L41` の 1 が述べる。

### DEF 消費点

D9 の消費の表の行が指す節点を**消費点**と呼び、その行が指す leaf を**消費される leaf** と呼ぶ。

### DEF 類ごとの義務

別名類 `C` (D33) について、`β(C)` を、`C` の ρ-終端が借用する (D14) パラメータ・capture の leaf である
とき 1、そうでないとき 0 と定め、`held_ρ(τ, C)` (D34) が定まる各時点 `τ` について

`obl_ρ(τ, C) := held_ρ(τ, C) - β(C)`

と置く。**これは A19 (i) の `d(C)` である** -- A19 (i) は
`d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置き、その角括弧が `β(C)` である。
以下で `d` はこの文書の `DEF 欠損` の量を指すので、A19 (i) の `d(C)` はこの名前で書く。

`bumps_ρ` は、A19 (ii-b) が言う「走査がその類について `pending` に数えている bump の個数」で
ある。その帰属は D27 が定める。**走査の状態は節点の訪問に付く (`DEF 節点の実行の素動作`) ので、この量の
引数は点ではなく `ρ` の上の節点である** -- `ρ` の上の節点 `n` と計数下の別名類 `C` について

`bumps_ρ(n, C) := Σ_{p ∈ pending(n)} Σ_{o ∈ C} B_ρ(n, p)[o]`

と置く。`p13` の `L17` の `bumps_ρ` はこれである。`n` の下では、`n` の実行のどの点でもこの値を読む
(`DEF 節点の実行の素動作`)。

計数下のオブジェクト `O` と点 `τ` について、`S(τ, O)` を、`obj(C) = O` (D33) であり**開始の時点
(D34) が `τ` 以前である**計数下の別名類 `C` の全体とする。`ρ` の上の節点 `n` について、`n` の入口の点を
`τ_n` と書き、

`b(n, O) := Σ_{C ∈ S(τ_n, O)} bumps_ρ(n, C)`

と置く。D34 は `held_ρ(τ, C)` をその類の開始の時点以後の `τ` についてだけ定め、`C` を渡る総和にその
条件を付ける。以下の類を渡る総和はどれも `S(τ, O)` を渡る。**`S` と `obl_ρ` と `held_ρ` の第 1 引数に
節点 `n` を書いたときは、`n` の入口の点 `τ_n` を指す** -- 例えば `S(q, O)` は `S(τ_q, O)` である。

**削除される `Retain` が積んだ bump の類ごとの内訳。** `ρ` の上の節点 `q` と計数下の別名類 `C` に
ついて、

`d_C(q) := Σ_{t} Σ_{o ∈ C} B_ρ(q, e_t(q))[o]`

と置く。外側の和は、`CT` に属し `q` で pending である `Retain` 節点 `t` を渡り (DEF 訪問)、内側の和は
`ρ` で活性な名前 `o` を渡る。`d(q, O) = Σ_{C ∈ S(q, O)} d_C(q)` であることは `L43a` が述べる。

### 前提 (ii-c) の保存

**`borrow_ify` が写した各本体についても A19 の (ii-c) が成り立つことを、この文書は前提として置く。**
すなわち `cancel` の入力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、**節点の実行の途中の各点
(D24 の段内の点) と、その点で `held_ρ` が定まる各計数下の別名類 `C`** について `held_ρ(・, C) ≥ 0` で
ある。**量化は A19 の (ii-c) のものである** -- A19 は「**(ii-c) (段内の点の非負性)。節点の実行の途中の
各点 (D24 の段内の点) と、その点で `held_ρ` が定まる各計数下の別名類について、`held ≥ 0` である。**」と
述べる。

**この前提は保守的であって、果たす者は在る。** A19 は (ii-c) の果たす者を「`insert_rc`
(`p60-insert-rc.md` の `L19` (d)) と `borrow_ify` (`p20-borrow-ify.md` の第 13 節) である。」と挙げ、
`report.md` の第 7.3 節は果たされていない義務を「**無し。** A19 (ii-c) は 3 人とも果たす --
`insert_rc` の側は `p60-insert-rc.md` の `L30` (b) が、`split_rc_units` の側は同じファイルの `L31a` が、
`borrow_ify` の側は `p20-borrow-ify.md` の第 13 節が示す。`p40-cancel-soundness.md` が第 2 節に置いた
`前提 (ii-c) の保存` は、その 3 つが満たす。」と述べる。この文書が扱う本体は `borrow_ify` の出力なので、
その半分をここで名前つきの前提として置き、`BY` の行ではこれを `前提 (ii-c) の保存` の名で引く。

### 前提 対応は素動作の粒度で読む

**D29 が `α` に与える与件を、この文書は素動作 (D24) の列の粒度で読む。** 対応する活性化 `α`、`α'`
(D29) について次の 2 つを前提として置く。

1. `Del` に入らない `ρ` の上の節点 `q` と、`ρ'` の上でそれに対応する節点について、2 つの節点の実行が
   行う素動作のうち、**D24 の (F) の解放とその連鎖が行うものを除いたもの**は 1 対 1 に対応し、対応する
   2 つの素動作は D24 の 6 種のうち同じ種であって、参照カウントに与える変化が等しく、名指すオブジェクトは
   D29 の全単射で対応する。**その節点の実行が作る子の活性化とその子孫が行う素動作も、この列の元に
   数える。**

   **(F) の解放とその連鎖は与件に数えない。** D24 の (F) は解放を「ある段が参照を処分して計数下の
   オブジェクト (D26) `o` の `H(o)` が 0 になったとき」に起きるものと定めるので、どの処分がどの
   オブジェクトの解放を起こすかは `H` から決まり、連鎖が行う処分はそのオブジェクトが保持する参照の
   ものである (D25)。それが 2 つの活性化で対応することは `L44c` の `<1>2` `<2>3` `<3>2` `<4>3` が示す。
2. `α` が実行する段でも `α` が作った活性化とその子孫が実行する段でもない段 -- 環境の段と、別の制御の
   流れの活性化の段 (`DEF 節点の入口の点`) -- は、2 つの活性化の段の列の対応する位置に、同じ並びで置く。

**D29 の与件は結果しか言わない。** D29 の第 4 行は「子の活性化を作る段 (D24) の結果は、この定義が外から
与えるデータである」と述べ、D21 の第 4 行は「返る値と、参照カウントに与える変化は、子の本体が決める」と
述べる。ところが D24 の段内の点は、(F) の解放が作る活性化の節点が行う動作も素動作の列の元に数えるので、
返る値と `H` への変化だけでは 2 つの活性化の点が 1 対 1 に並ばない。**2 も同じ形である** -- D30 は
2 つの実行について「複数の制御の流れがある場合は段の並び方を同じにして取る」と定めるが、D29 に対応する
節が無い。

**この前提は `α` の存在を狭めない。** D29 は「**子の活性化を作る段の結果を共有することが実際の実行で
成り立つかは、この定義の主張ではないし、どの命題の主張でもない。**」と述べ、D21 の意味の活性化は実行に
実現するとは限らない。`α` は `α'` の素動作の列に `Del` の節点の素動作を挿し込んだものとして取れるので、
この粒度の与件を課しても `α` はちょうど 1 つ定まる。

**枠に置くのが本来である。** D29 は 1 つの本体の 2 つの活性化についての定義であり、D30 は 2 つの実行に
ついて同じ形の節 -- 「**段の素動作の列が対応する**」と「段の並び方を同じにして取る」-- を既に持つ。
**果たす者は要らない** -- D29 は `α` を構成する定義であって、果たされるべき性質ではない。
**読む者は `L44c` の `<1>1` `<2>3`・`<1>2` `<2>3`・`<1>2a` と `L44g` の `<1>2` である。**

### 前提 記号の位置の値の対応

**記号の位置 (D6) が持つ値は、2 つの活性化で対応する。** すなわち、束縛を持たない名前 `g` と `ty(g)` の
inhabited (D16) な boxed leaf `λ` について、`α` の側の位置 `(g, λ)` と `α'` の側の同じ位置は、値の
スカラの成分が等しく、Fix の関数型の成分が対応する版 (`DEF 対応の 3 つ` の (a)) を名指し、
計数下 (D26) のオブジェクトを指すときはそのオブジェクトが D29 の全単射で対応する。

**D29 の 4 つの行はこの値を生成しない。** 記号の位置は入力の束縛でも、割り当ての位置でも、子の活性化の
結果でもない。その値を作るのはその記号のグローバル初期化子の活性化であって (D24 の (E7)、(E5))、この
本体の活性化の外にある。D21 が「1 つの本体だけを見ている間、これは外から与えられる量である」と書くのと
同じ形で、この文書はこの値も与件に数える。**2 つのプログラムの記号が 1 対 1 に対応することは P24 の
第 3 の箇条が与える** -- 出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` は入力の
第 `i` 要素のものに等しい。**関数の名前についてはこの前提は要らない** -- そこが指すのは funptr であり、
D24 が「実行時に名前 `n` が指す関数は `P.funcs[FuncRef{n}]` の本体を実装したものである」と定めるので、
`L44d` が 2 つの版の対応を与える。

**枠に置くのが本来である。読む者は `L44c` の `<1>1` `<2>2` `<3>2a` である。**

### 在りかの前提

**コードのどこに何が在るかの数え上げは、段の中で行わない。** 段が自分で「クレートに 3 か所」と数えると、
その数え上げには果たす者が居らず、検査するものも無い。**記号を名指す `CODE` の引用はその記号の本体しか
与えないので、「ほかの記号はそれをしない」の側はそこから出ない。** 以下の 6 つを名前つきの前提として置き、
`BY` の行ではその名前で引く。**個数は書かない** -- 一覧が在れば個数は一覧の長さである。

**果たすのは走査である。** 在りかを走らせられる字面で書き、`dev-docs/proof/proof_links.py` がその字面を
`src/` の全体に走らせて、下の一覧と突き合わせる。挙がった各項目が何であるかは `--` の後に書く。
走査は字面の上位近似なので、一覧には読みだけの項目も入る。

**前提 走査の記録の書き込みの在りか** --- `CancelAnalysis` の `all_retains` に値を積む式と、
`un_bump_releases` の値の `Vec` に要素を入れる式は、それぞれ `walk_inner` の中の 1 つだけである。

SCAN src/ `self.all_retains`
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `RcExpr::Retain` の腕の `push`
  = src/rc_ir/borrow.rs: CancelAnalysis::cancelled -- 読み

SCAN src/ `un_bump_releases`
  = src/rc_ir/borrow.rs: cancel -- 構成子が空の写像を置く
  = src/rc_ir/borrow.rs: CancelAnalysis -- 欄の宣言
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `Retain` の腕の `entry(..).or_default()` と `Release` の腕の `push`
  = src/rc_ir/borrow.rs: CancelAnalysis::cancelled -- 読み

**前提 消費の呼び出しの在りか** --- `consume_objects`・`consume`・`consume_rhs` を呼ぶ式が在る項目は
次で尽きる。

SCAN src/ `self.consume_objects(`
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `RcExpr::Release` の腕の 2 つ
  = src/rc_ir/borrow.rs: CancelAnalysis::consume -- 末尾の 1 つ

SCAN src/ `self.consume(`
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `RcExpr::Destructure` の腕
  = src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs -- `rhs_consumes` が積んだ各対について

SCAN src/ `self.consume_rhs(`
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- 右辺が `Match` でない `RcExpr::Let` の腕

**前提 `un_bump` の呼び出しの在りか** --- 自由関数 `un_bump` を呼ぶ式が在る項目は `walk_inner` だけである。

SCAN src/ `un_bump(`
  = src/rc_ir/borrow.rs: un_bump -- 定義
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `RcExpr::Release` の腕

**前提 解析の値と表の作り手の在りか** --- `CancelAnalysis` の値を作る式、`VarTable` を可変に借りる関数、
その関数を呼ぶ式、および `vars`・`type_env` の欄へ代入する式が在る項目は次で尽きる。`.vars = ` の走査は
何も挙げない。

SCAN src/ `CancelAnalysis {`
  = src/rc_ir/borrow.rs: cancel -- `cancel_body` に渡す値を作る

SCAN src/ `&mut VarTable`
  = src/rc_ir/ownership.rs: collect_bindings -- 唯一の受け取り手

SCAN src/ `collect_bindings(`
  = src/rc_ir/ownership.rs: VarTable::of -- 自分の作った局所の表に対して
  = src/rc_ir/ownership.rs: VarTable::body_only -- 自分の作った局所の表に対して
  = src/rc_ir/ownership.rs: collect_bindings -- 定義と、その中の継続・アーム本体への再帰

SCAN src/ `.vars = `

SCAN src/ `.type_env = `
  = src/ast/program.rs: Program::calculate_type_env -- `Program` の欄であって `CancelAnalysis` の欄ではない

**前提 束縛を置く在りか** --- `Binding` の各変位を置く式が在る項目は次で尽きる。走査は
`origin_inner` のパターンも挙げる。`#[cfg(test)]` の下の項目は走査が除く。

SCAN src/ `Binding::Param`
  = src/rc_ir/ownership.rs: VarTable::of -- パラメータと capture について置く
  = src/rc_ir/ownership.rs: origin_inner -- パターン

SCAN src/ `Binding::Producer`
  = src/rc_ir/ownership.rs: collect_bindings -- `RcRhs::App` と `RcRhs::Closure` の結果について置く
  = src/rc_ir/ownership.rs: origin_inner -- パターン

SCAN src/ `Binding::Field(`
  = src/rc_ir/ownership.rs: collect_bindings -- `Destructure` の名前付きフィールドについて置く
  = src/rc_ir/ownership.rs: origin_inner -- パターン

SCAN src/ `Binding::Payload(`
  = src/rc_ir/ownership.rs: collect_bindings -- `Match` のアームの payload について置く
  = src/rc_ir/ownership.rs: origin_inner -- パターン

**前提 `result_prov` の本体の在りか** --- `LLVMGen::result_prov` の本体を書く項目は次で尽きる。
**そのどれもが、`Provenance::uniform`・`Provenance::uniform_bottom`・`Provenance::fresh_under`・
`Provenance::build_shape`・`replaced_field_prov` のいずれかを第 1 引数 `result_ty` に掛けた値を返す。**
各項目が返す形は `--` の後に書く。

SCAN src/ `fn result_prov`
  = src/ast/inline_llvm.rs: result_prov -- 既定の本体。`uniform(result_ty, ・, Unknown)`
  = src/fixstd/builtin.rs: InlineLLVMStringBuf::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayUnsafeEmpty::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayTruncateBoundsUnchecked::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayAppendValueCapacityUnchecked::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArraySetCapacityBoundsUnchecked::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayAppendCapacityUnchecked::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayGrowSizeBody::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArraySetBody::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArraySwapBody::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMArrayPunchBody::result_prov -- `fresh_under(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMPunchedArrayPlugBody::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov -- `uniform(result_ty, ・, Unknown)` か `build_shape(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov -- `build_shape(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMArrayLitBody::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMStructPunchBody::result_prov -- `fresh_under(result_ty, ・, ・)` か `build_shape(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMStructPlugInBody::result_prov -- `replaced_field_prov(result_ty, ・, ・, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMStructSetBody::result_prov -- `replaced_field_prov(result_ty, ・, ・, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov -- `build_shape(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov -- `uniform(result_ty, ・, Unknown)` か `build_shape(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMUndefinedInternalBody::result_prov -- `uniform_bottom(result_ty, ・)`
  = src/fixstd/builtin.rs: InlineLLVMIsUniqueFunctionBody::result_prov -- `uniform(result_ty, ・, Unknown)`
  = src/fixstd/builtin.rs: InlineLLVMArrayIsStorageUniqueBody::result_prov -- `uniform(result_ty, ・, Unknown)`
  = src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedInternalFunctionBody::result_prov -- `fresh_under(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMUnsafeMutateBoxedIOSInternalBody::result_prov -- `fresh_under(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsInternalBody::result_prov -- `fresh_under(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMArrayMutateElementsIosInternalBody::result_prov -- `fresh_under(result_ty, ・, ・)`
  = src/fixstd/builtin.rs: InlineLLVMDestructorMake::result_prov -- `uniform(result_ty, ・, Fresh)`
  = src/fixstd/builtin.rs: InlineLLVMMarkThreadedFunctionBody::result_prov -- `uniform(result_ty, ・, Unknown)`

**この 6 つは枠の仮定に置くのが本来である。** ここに置いているのは、この文書が自分の段からそれを引ける
ようにするためであり、`前提 (ii-c) の保存` と同じ形である。

## 3. 局所の命題

### L45 (活性な名前は `ρ` の上のスロットである) <!--#b154692-->

**言明** --- 名前 `o` が実行路 `ρ` で活性 (`p13` の `DEF 名前の活性`) であるとき、`o = (u, σ)` は `ρ` の
上のスロット (D6) であり、`obj_ρ(o) = obj(u, σ)` である。逆に、`ρ` の上のスロット `(x, λ)` について
`obj(x, λ)` が計数下 (D26) であるとき、`id(x, λ)` は `ρ` で活性であって
`obj_ρ(id(x, λ)) = obj(x, λ)` である。したがって README の P18a が `obj(o)` と書く量はこの `obj_ρ(o)` で
ある。

**証明**

<1>1. `o` が `ρ` で活性であるとき、`origin(x, λ).identity() = o` を満たす対 `(x, λ)` で、
      `λ ∈ boxed_leaf_paths(ty(x))` であり、`ρ` の上で `x` が値を得ており、`λ` が `x` の値の inhabited な
      leaf であり、`obj(x, λ)` が計数下であるものが在る。この対は `ρ` の上のスロット (D6) である。
  BY p13 の DEF 名前の活性, <ref id=596a46d/>, <ref id=e683eae/>
  D6 は「実行のある時点における**スロット**とは、対 `(x, λ)` である。ここで `x` はその実行路の上で
  その時点までに値を得た変数、`λ` は `ty(x)` の inhabited な boxed leaf である。」と定め、その脇が
  「**実行路 `ρ` の上のスロット (`ρ` の位置) とは、`ρ` を辿るある時点でスロットである対のことである。**」
  と広げる。`p13` の `DEF 名前の活性` が挙げる対はこの条件を満たす。
<1>2. 逆向きが成り立つ。`ρ` の上のスロット `(x, λ)` について `obj(x, λ)` が計数下であるとき、
      `id(x, λ)` は `ρ` の上のスロットであって `ρ` で活性であり、`obj_ρ(id(x, λ)) = obj(x, λ)` である。
  BY <ref id=2171d13/>, <ref id=88a06de/>, <ref id=596a46d/>
  `p13` の `L14` は「`ρ` の上のスロット `(x, λ)` について、`o = origin(x, λ).identity()` は `ρ` の上の
  位置 (D6) -- スロットか記号の位置 -- であり、D6 の `obj` について
  `obj(o) = obj(C_ρ(x, λ)) = obj(x, λ)` である。さらに `obj(x, λ)` が D26 の意味で計数下であるとき、
  `o` は `ρ` の上のスロットであって `C_ρ(o) = C_ρ(x, λ)` であり、`o` は `ρ` で活性 (DEF 名前の活性) で
  あって `obj_ρ(o) = obj(x, λ)` である。」と述べる。
<1>3. 前半が成り立つ。
  BY <1>1, <1>2
  <1>1 の対 `(x, λ)` はスロットであり `obj(x, λ)` は計数下なので、<1>2 がその `id(x, λ) = o` について
  `ρ` の上のスロットであることと `obj_ρ(o) = obj(x, λ)` を与える。
<1>4. QED
  BY <1>2, <1>3, <ref id=97bdd4e/>, <ref id=8093b68/>
  README の P18a は `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と置き、その `o` は D27 の `B(p, ρ)` の
  鍵である。D27 は `B(p, ρ)` を「`π` の下の inhabited (D16) かつ計数下 (D26) の各 leaf を `origin` の
  identity で名付けて数えたもの」と定めるので、その鍵は <1>2 の形の `id(x, λ)` であって活性であり、
  P18a の `obj(o)` は `obj_ρ(o)` である。

### L46 (節点の量は走査のどの段階で読んでも同じである) <!--#72bac67-->

**言明** --- `DEF 節点の量` の `ActRefs(t)`、`ActRefs(r)`、`others(r)` は、走査のどの段階で読んでも
同じ値である。

**証明**

<1>1. `CancelAnalysis::acted_references` は `acted_references(self.vars, self.type_env, v, path)` を、
      `CancelAnalysis::other_objects` は `boxed_leaf_paths(&v.ty, self.type_env)` の各 leaf についての
      `origin(self.vars, self.type_env, v.name, leaf)` を呼ぶだけである。D15 より
      `acted_references` の値はその引数と `origin` の答えで決まる。よってこの 3 つの値は、引数と、
      `self.vars`・`self.type_env` の欄と、`origin` の答えで決まる。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects,
     CODE src/rc_ir/ownership.rs: acted_references, <ref id=cbc4a1c/>
<1>2. `CancelAnalysis` の `vars` と `type_env` の欄は、`cancel_body` が据えた 1 つの `VarTable` の値と
      1 つの `TypeEnv` の値への共有参照であり、走査はその欄を差し替えない。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis, CODE src/rc_ir/borrow.rs: cancel,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: origin,
     CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: Binding,
     CODE src/ast/types.rs: TypeNode, CODE src/parse/sourcefile.rs: SourceFile,
     CODE src/rc_ir/ast.rs: RcVar, FuncRef, CODE src/ast/name.rs: FullName, NameSpace,
     <ref id=e11772a/>, <ref id=b1f6e13/>,
     前提 解析の値と表の作り手の在りか, EXT 共有参照は代入を許さない
  `前提 解析の値と表の作り手の在りか` の走査より、`CancelAnalysis` の値を作るのは `cancel` の中の
  1 か所だけであり、`vars` と `type_env` の欄へ代入する式はどこにも無い -- `.vars = ` の走査は何も
  挙げず、`.type_env = ` の走査が挙げる 1 項目は `Program` の同名の欄についてのものである。
  **欄の型は共有参照なので、その先の値の欄へ代入することもできない**
  (EXT 共有参照は代入を許さない)。**残るのは内部可変性であり、その在りかは型で決める** -- A3 は
  「**在りかは型で決める。** `RefCell`・`Cell`・`OnceCell`・`OnceLock`・`Mutex`・`RwLock`・`UnsafeCell`・
  `Atomic*` のいずれかを含む欄の宣言を走査し、その値から到達できるものを取る。**特定のメソッド名や特定の
  欄で数え上げると、別の型を経て届く道が落ちる。**」と述べる。`VarTable` の 5 欄からその走査が取るのは
  次の 3 つである。
  - `origins` の `RefCell<Map<VarPath, Origin>>` (`CODE src/rc_ir/ownership.rs: VarTable`)。`origin` は
    共有参照から `vars.origins.borrow_mut().insert(key, answer.clone())` を実行する
    (`CODE src/rc_ir/ownership.rs: origin`)。この欄が動いても `origin` の答えが動かないことは P2a が言う。
  - `param_tys` と `var_tys` の `Arc<TypeNode>`、および `bindings` の `Binding::Llvm` の第 3 欄の
    `Arc<TypeNode>` から届く `TypeNode` の `hash_cache`・`ground_cache`・`depth_cache` の 3 つの
    `OnceLock` (`CODE src/ast/types.rs: TypeNode`)。`bindings` の `Binding` が運ぶ `RcVar` の `ty` も
    同じ型へ届き (`CODE src/rc_ir/ast.rs: RcVar`, `CODE src/rc_ir/ownership.rs: Binding`)、
    `Binding::Llvm` の第 1 欄の `Box<dyn LLVMGen>` が欄に持つ型も同じである。
  - `TypeNode` の `info.source` から `Span.input` を経て届く `SourceFile` の `string` と `hash` の
    `Arc<Mutex<Option<String>>>` (`CODE src/parse/sourcefile.rs: SourceFile`)。`RcVar` の `source` も
    同じ型へ届く。

  **残る 1 欄 `closure_targets: Map<FullName, FuncRef>` からは、この走査は何も取らない。**
  `FuncRef` は `name: FullName` の 1 欄を持ち、`FullName` は `namespace: NameSpace` と `name: String`
  を、`NameSpace` は `names: Vec<String>` と `is_absolute: bool` を持つ
  (`CODE src/rc_ir/ast.rs: FuncRef`, `CODE src/ast/name.rs: FullName`,
  `CODE src/ast/name.rs: NameSpace`)。ここに現れる欄の型 -- `FullName`・`NameSpace`・`String`・
  `Vec<String>`・`bool` -- はどれも、A3 が挙げる 8 つの型のいずれかを含む欄を持たない。

  この 3 つのどれも `origin` の答えを動かさない。1 つ目は P2a が、残る 2 つは A3 が
  「到達できる型が内部可変性を持つ欄を持つときは、その欄は**一度だけ
  書かれる memo であって、その値はその型の `PartialEq` が読む成分の関数である**」と述べ、`TypeNode` の
  3 つの `OnceLock` と `SourceFile` の 2 つの `Mutex` をその形の欄として名指す。
  `origin` が `vars` から読む残りの欄 `bindings` については、写像そのもの -- 鍵の集合と、各鍵に据わる
  `Binding` の変位とその欄 -- が、表が構成子から返った後は動かない。鍵を足し引きすることも、ある鍵に
  別の `Binding` を据え直すことも、その写像の欄への代入であり、共有参照からはできない
  (EXT 共有参照は代入を許さない)。**`bindings` から到達できる内部可変性の欄は、上の第 2 と第 3 の箇条が
  挙げる `TypeNode` の 3 つの `OnceLock` と `SourceFile` の 2 つの `Mutex` である。** その 5 つへの
  書き込みは一度だけ書かれる memo を埋めるだけで (A3)、写像の鍵の集合も各鍵の `Binding` も据え直さない。
  よって写像そのものを動かす道は `VarTable` の `&mut` の借用に限る。`CancelAnalysis` が持つのは共有参照
  だけであり (`CODE src/rc_ir/borrow.rs: CancelAnalysis`)、共有参照から `&mut` の借用は作れない。
  **`&mut VarTable` を受け取る関数は `collect_bindings` ただ 1 つであり、それを呼ぶのは `VarTable::of`
  と `VarTable::body_only` と `collect_bindings` 自身である** -- `前提 解析の値と表の作り手の在りか` の
  `&mut VarTable` の走査が挙げるのは `collect_bindings` の 1 項目であり、`collect_bindings(` の走査が
  挙げるのはその 3 項目である。前の 2 つはどちらも自分の作った局所の値に対して呼び、`collect_bindings`
  自身の呼び出しは受け取った同じ借用を継続とアーム本体へ渡すだけである
  (`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`, `collect_bindings`)。
<1>3. `self.vars` は、A6 と A11 を満たす本体について `VarTable::of` か `VarTable::body_only` が作った
      表である。
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: VarTable::body_only, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=63eadd9/>
  `cancel` は各関数について `VarTable::of(f)` を、各グローバル初期化子について
  `VarTable::body_only(&g.init)` を作って `cancel_body` に渡す。`B` は `borrow_ify` の出力の本体で
  ある (第 1 節)。A6 は「出力についての同じ性質は仮定ではなく P9 が示す -- `fresh_rename_function` を
  呼ぶのは証明対象の `borrow_ify` 自身なので、それを仮定に置くと証明対象が自分を支えることになる。」と
  述べ、A11 は「**この仮定が語るのは `borrow_ify` の入力である。** 出力についての同じ性質は、この仮定と
  P9 から出る -- 複製の本体は原本の束縛変数を一斉に付け替えたものでそれ以外の違いを持たないので、束縛と
  使用の対応がそのまま写る。」と述べる。すなわち `B` について A6 と A11 を読む段は P9 と合わせて読む。
<1>4. QED
  BY <1>1, <1>2, <1>3, <ref id=b1f6e13/>
  P2a は「**1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定する。** その 2 つを第 1・第 2
  引数とし、鍵 `(x, π)` が等しい 2 つの `origin` の呼び出しがどちらも値を返すならば、その 2 つの返り値は
  等しい」と述べ、その `vars` を「A6 と A11 を満たすプログラムの本体について `VarTable::of` か
  `VarTable::body_only` が作った表」に限る。<1>2 と <1>3 よりこの制限は満たされ、走査の全体が 1 つの
  `VarTable` の値と 1 つの `TypeEnv` の値を第 1・第 2 引数として `origin` を呼ぶので、鍵が等しい呼び出しの
  答えは走査のどの段階でも等しい。<1>1 よりこの 3 つの値も等しい。

### L47 (節点の実行の素動作) <!--#ba12699-->

**言明** --- 次の 3 つが成り立つ。

1. `ρ` の上の節点 `q` の実行に属する段 (`DEF 節点の実行の素動作`) が `q` の実行の点で行う素動作
   (D24) は、`DEF 節点の実行の素動作` の 6 種で尽きる。**(E1)・(E8)・(E9) の段は `q` の実行に属する
   段ではない。** その 3 種を含め、**`q` の実行に属さない段の素動作はこの数え上げに入らない**
   (`DEF 節点の実行の素動作`)。
2. `q ∉ Del` のとき、`q` の実行のどの点でも `d(τ, O) = d(q, O)` である (DEF 欠損)。`q ∈ Del` のとき、
   `d` が動くのは `q` 自身の D10 の行の素動作の直後の点だけであり、`Retain` 節点では leaf ごとに 1 上がり、
   `Release` 節点では leaf ごとに 1 下がる。
3. `α` の素動作 (`DEF 節点の実行の素動作`) は、`ρ` の上のどれかの節点の実行に属する段のものである。

**証明**

<1>1. 段の境界について、`q` の実行が `Obl` と `H` に与える動きは D24 の (E2)・(E3)・(E4)・(E7) が挙げる
      ものだけである。
  BY <ref id=e3436e8/>, <ref id=a89f403/>, <ref id=5b4974e/>, <ref id=8c40929/>, <ref id=7e70ffa/>, <ref id=f06144e/>, <ref id=9d74736/>, <ref id=1df9ec0/>, <ref id=c8f928a/>, <ref id=98680d9/>, <ref id=8e052e9/>, DEF 節点の実行の素動作
  D24 は「**段の記述は `Obl` について網羅である。** (E1)-(E9) と (F) は、各段について `Obl` を離れる
  参照の行き先と、作られる参照の持ち手と、`H` の動きを全部書いている。**すなわち、ここに挙がっていない
  動きは起きない。**」と述べる。`DEF 節点の実行の素動作` より、節点 `q` の実行に属する段は (E2)・(E3)・
  (E4)、`q` の中で走る (E7)、その (E7) が作る初期化子の活性化が終わるところで走る (E5)、そして `q` が
  プロセスを終える `Llvm` の節点であるときの (E6) である。**そのうち (E5) と (E6) は `Obl` も `H` も
  動かさない** -- D24 は (E5) について「**この段は参照を作らず、渡さず、処分しない**」、(E6) について
  「**この段は参照を作らず、渡さず、処分しない。**」と述べる。**残る (E1)・(E8)・(E9) は `q` の実行に
  属さない。** `DEF 節点の実行の素動作` は `q` の実行に属する段を 3 種 -- `α` が `q` の位置で実行する段、
  `q` の実行が作る活性化とその子孫の活性化が実行する段、`q` の中で (E7) が作る初期化子の活性化が終わる
  ところで走る (E5) の段 -- と定める。**(E8) と (E9) を実行するのは環境である** -- D24 は (E8) を
  「**(E8) 環境の読み書きの段。** 環境が、Fix の側から渡された番地の指す記憶域を読むか、そこへ書き込む」、
  (E9) を「**(E9) 環境の参照の操作の段。** 環境が、`get_funptr_retain` か `get_funptr_release` が渡した
  番地を呼ぶ」と定めるので、どちらも活性化が実行する段ではない。**(E1) を実行するのも環境であり、
  そこで作られる活性化は林の根である** -- D24 は (E1) を「**(E1) 環境が活性化を作る段。** C のエントリ点
  または `FFI_EXPORT` のエントリ点が、関数の本体 `B` の活性化 `a` を作り」と定め、「活性化の林」は
  「(E1) が作る活性化を**根**」と呼ぶ。根は `q` の実行が作る活性化でもその子孫でもない。
  **(E9) は `Obl` を動かさない** --
  D24 は「**retain の段はその番地が指すオブジェクトへの参照を 1 つ作り、
  環境の持ち分に足す** -- `H` はその分だけ上がり、`Obl` は動かない。**release の段は環境の持ち分から
  参照を 1 つ処分する** -- `H` はその分だけ下がり、`Obl` は動かない。」と述べる。
<1>1a. **この網羅は段の境界についてであり、段の中には表に行を持たない素動作が在る。**
  BY <ref id=e3436e8/>, DEF 節点の実行の素動作
  D24 は <1>1 の節に続けて「**この網羅は段の境界についてである。** 1 つの段の生成コードは、この表に行を
  持たない素動作を段の中で出しうる。**素動作の粒度で勘定する段は、その op の `generate` が出す retain と
  release を読む。**」と述べ、「**在りかは述語で決める** -- `Generator::retain`・
  `Generator::build_retain`・`Generator::release` の呼び出しを出す生成コードの全体であり、一覧で書くと
  op が 1 つ増えるたびに古くなる。」と続ける。`DEF 節点の実行の素動作` の第 6 の箇条がその 2 つの形を
  写す。
<1>1ab. (E6) の段は素動作を 1 つも持たない。
  BY <1>1, <ref id=98680d9/>, <ref id=8e052e9/>, <ref id=ff38f3b/>, <ref id=1df9ec0/>, <ref id=e3436e8/>, DEF 節点の実行の素動作
  D24 の (E6) の節は「**この段は参照を作らず、渡さず、処分しない。**」と述べる。すなわち受け渡し・
  生成・処分の 3 種はこの段に無い。**割り当てもない** -- `DEF 節点の実行の素動作` の第 2 の箇条は
  割り当ての素動作を「すなわちこの素動作は第 1 の箇条の生成でもある」と述べるので、生成の無い段は
  割り当てを持たない。**解放もない** -- D24 の (F) は解放を「ある段が参照を処分して計数下のオブジェクト
  (D26) `o` の `H(o)` が 0 になったとき」に起きるものと定めるので、処分の無い段に解放は無い。
  **グローバル化もない** -- `DEF 節点の実行の素動作` の第 5 の箇条はグローバル化を「(E5) の段が行う
  もの」と定め、D24 は段を (E1) から (E9) の 9 種と定めるので、(E6) の段は (E5) の段ではない。
  **op の生成コードが出す retain と release もない** -- `DEF 節点の実行の素動作` の第 6 の箇条の
  2 つの形はどちらも参照を 1 つ作る素動作を含み (前者は `Obl(α)` の、後者は書き込む先のオブジェクトの
  持ち手の単位の)、この段は参照を作らない。D24 も (E6) について「`abort` を呼ぶ op は release を 1 つも
  出さない」と述べる。
<1>1b. 1 が成り立つ。
  BY <1>1, <1>1a, <1>1ab, <ref id=e3436e8/>, <ref id=f06144e/>, <ref id=9d74736/>, <ref id=9d5d254/>, <ref id=c8f928a/>, DEF 節点の実行の素動作
  <1>1 の各段について D24 が挙げる動きは、D10 の行が定める作成と処分 (第 1 の箇条)、その
  うち割り当てを伴うもの (第 2 の箇条)、この段が作る子の活性化の素動作 (第 3 の箇条)、処分に付随する
  (F) の解放 (第 4 の箇条)、(E5) のグローバル化 (第 5 の箇条) で尽きる。<1>1a の素動作が第 6 の箇条で
  ある。**D24 が「在りかは述語で決める」と定めるので、この 6 つ目は op の一覧ではなく、
  `Generator::retain`・`Generator::build_retain`・`Generator::release` の呼び出しを出す生成コードの
  全体として数える。**(E5) の段が行うのはグローバル化だけである -- D24 は「**この段は参照を作らず、
  渡さず、処分しない** -- `mark_global` はオブジェクトを辿って印を付けるだけである。」と述べる。
  (E6) の段は <1>1ab より素動作を持たない。
<1>2. 2 が成り立つ。
  BY <1>1b, <ref id=7d5a1de/>, DEF 欠損, <ref id=f06144e/>, <ref id=e3436e8/>, <ref id=b3dfa37/>
  `DEF 欠損` の 2 つの数え上げが渡るのは、`α` が `ρ` の上で実行した `Del` の節点について D10 の
  `Retain`/`Release` の行が定める素動作だけである。<1>1b の 6 種のうち、子の活性化の素動作・(F) の解放・
  グローバル化・op の生成コードが出す retain と release はそのどれでもない -- 最後のものが `Del` の節点で
  起きないことは、`Del` の要素が `Retain` 節点か `Release` 節点だけであり (L32 の 5)、その 2 種が
  `Llvm` の op を持たないことによる (D2)。L32 の 5 より `Del` の要素は `Retain` 節点か `Release` 節点
  だけなので、`d` を動かす素動作を持つ節点は `Del` の要素に限る。
<1>2a. 3 が成り立つ。
  BY <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=7e70ffa/>, <ref id=ca36627/>, DEF 節点の実行の素動作
  `DEF 節点の実行の素動作` は `α` の素動作を 3 種 -- `α` が実行する段が行うもの、`α` が作った活性化と
  その子孫の活性化が実行する段が行うもの、`α` の中で (E7) が作る初期化子の活性化が終わるところで走る
  (E5) の段が行うもの -- と定める。**第 1 の種について。** D23 は活性化の**位置**を「`B(a)` の 1 つの
  実行路 (D3) の上の位置。活性化が始まった時点では `B(a)` の根であり、節点を 1 つ実行するごとにその路に
  沿って 1 つ進む。」と定めるので、`α` は常に `ρ` の上のある節点の位置にある。`α` が実行する段は
  D24 の (E2)・(E3)・(E4)・(E7) であり、いずれもその位置の節点を実行する段か、その節点の位置で
  中断・再開する段である。よってそれは `DEF 節点の実行の素動作` の第 1 の種、すなわちその節点の実行に
  属する段である。**第 2 の種について。** D24 の「活性化の林」は活性化を作る段を (E1)・(E3)・(E7)・
  (E2) のうちオペランドを適用する `Llvm` の段・(F) の解放の 5 種と数え、「**活性化を作る段はこの 5 種で
  尽きる。**」と述べる。そのうち `α` が実行するのは (E3)・(E7)・(E2) と、`α` の段の中で起きる (F) の
  解放であり、どれも第 1 の種の段なので、その節点の実行に属する。したがってそれが作る活性化とその子孫が
  実行する段も、`DEF 節点の実行の素動作` の第 2 の種としてその節点の実行に属する。(E1) が作る活性化は
  林の根であって `α` が作った活性化ではない (<1>1)。**第 3 の種について。** その (E5) の段は、
  `DEF 節点の実行の素動作` がその (E7) を持つ節点の実行に属する段として挙げるものである。
<1>3. QED
  BY <1>1, <1>1a, <1>1b, <1>2, <1>2a

### L48 (`DEF 解放されている` は D11a の「解放されている」と同じものを指す) <!--#4798985-->

**言明** --- 活性化の点 `τ` (`DEF 実行時の量`) と計数下 (D26) のオブジェクト `O` について、`O` が
`DEF 解放されている` の意味で `τ` で解放されていることと、`O` が D11a の読む意味で -- すなわち D24 の
(F) の解放を `τ` までに受けて -- 解放されていることは同値である。**時点は点の一部なので
(`DEF 実行時の量`)、この同値は D11a が量化する各時点についても読める。**

**証明**

<1>1. D24 の (F) が `τ` までに `O` を解放したならば、`H(τ0, O) = 0` を満たす `τ` 以前の点 `τ0` が在る。
  BY <ref id=e3436e8/>, <ref id=b065d17/>, DEF 解放されている, DEF 実行時の量
  (F) は「ある段が参照を処分して計数下のオブジェクト (D26) `o` の `H(o)` が 0 になったとき、`o` は
  **その同じ段の中で解放される**」と述べる。それを起こした処分がカウントを 0 にしているので、その処分の
  直後の点で `H = 0` である。**その点はこの文書の点である** -- D24 は段内の点を「段の素動作を時間の順に
  並べたときの、最初の素動作の前の点と各素動作の後の点」と定め、`DEF 実行時の量` はこの文書の点をその
  段内の点と定める。
<1>2. 逆に `H(τ0, O) = 0` である `τ` 以前の点 `τ0` が在るならば、`O` は `τ` までに (F) の解放を経ている。
  BY <ref id=e3436e8/>, <ref id=c9e4cca/>, <ref id=0b850c9/>, <ref id=ec8d1a0/>, DEF 解放されている, DEF 節点の実行の素動作
  割り当ての直後のカウントは 1 であり (D24 の (E2) の `H` の表の「単一の `Fresh` ならこの段が新しく
  割り当てるオブジェクトで `H` = 1」と `Closure(f, caps)` の行)、環境が持ち込んだオブジェクトは実行の
  最初の時点で持ち手を少なくとも 1 つ持つ (A17 の (i-c) -- 「**その時点に生きている各計数下オブジェクトは、
  少なくとも 1 つの持ち手を持つ**」)。**カウントを下げるのは参照の処分だけである。** D8 は `H(O)` を
  `O` への未処分の参照の総数と定めるので、`H` が下がるのは参照が処分される素動作だけである。段の境界に
  ついて D24 は「**段の記述は
  `Obl` について網羅である。** (E1)-(E9) と (F) は、各段について `Obl` を離れる参照の行き先と、作られる
  参照の持ち手と、`H` の動きを全部書いている。**すなわち、ここに挙がっていない動きは起きない。**」と
  述べ、そこで `H` を下げる行は処分だけである。**段の中の素動作についても同じである** -- D24 は
  「**この網羅は段の境界についてである。** 1 つの段の生成コードは、この表に行を持たない素動作を段の中で
  出しうる。**素動作の粒度で勘定する段は、その op の `generate` が出す retain と release を読む。**」と
  述べ、`DEF 節点の実行の素動作` の第 6 の箇条がその 2 つの形を写すが、そのうち `H` を下げるのは
  release、すなわち参照の処分である。**環境が `H` を下げる (E9) の段も処分である** -- D24 は
  「**release の段は環境の持ち分から参照を 1 つ処分する** -- `H` はその分だけ下がり、`Obl` は
  動かない。」と述べ、A17 (ii-c) は「**環境は、自分が持たない参照を処分しない。**」と述べる。よって
  `H` が 0 になった最初の点はある処分の直後であり、(F) はその処分に付随して `O` を解放する。
<1>3. QED
  BY <1>1, <1>2

### L50 (第 2 節の記法は 1 つに定まる) <!--#941af96-->

**言明** --- 次の 4 つが成り立つ。

1. `pending(n)`、`pending_out(n)`、`ret(n)`、および `pending(n)` の要素 `e` の由来 (`DEF 訪問`) は、
   いずれも 1 つに定まる。`Retain` 節点 `t` が `n` で pending であるとき、その要素 `e_t(n)` も 1 つに
   定まる。
2. 本体の木の根でない各節点はちょうど 1 つの親 (`DEF 子と親`) を持ち、節点はどの子の部分木にも入らない。
3. `Del`、`CT`、`un_bump_releases` の要素を節点と同一視してよい (`DEF 削除集合`)。
4. 実行路 `ρ` を辿る活性化 `α` と、`ρ` の上の節点 `q` を取る。`q` が `ρ` の終端の `Ret` でないとき、
   `q'` (`DEF 節点の入口の点`) と、`ρ` の上で `q` の次にある節点の入口の点とのあいだに `α` の素動作は
   無く、そこに挟まる段はどれも `α` が実行する段ではない -- 環境の段か、別の制御の流れの活性化の段で
   ある。`ρ` の上の入口の点は有限個である。

**証明**

<1>1. 1 が成り立つ。
  BY <ref id=24bf090/>, <ref id=a423f41/>, <ref id=b3dfa37/>, DEF 訪問
  P15 の後半より走査は `B` の各位置をちょうど 1 回訪問するので、`pending(n)` と `pending_out(n)` は
  節点ごとに 1 つに定まる。D2 より継続の鎖は有限で `Ret` で終わるので `ret(n)` は 1 つに定まる。
  P16 の (a) より `pending` の各要素の `node` は走査が訪れた `Retain` 節点であり、P15 の前半より
  相異なる位置は相異なる `NodeId` を持つので、由来は 1 つに定まる。P16 の (c) より 1 つの `Retain`
  節点は `pending` に高々 1 回現れるので、`e_t(n)` も 1 つに定まる。
<1>2. 2 が成り立つ。
  BY <ref id=b3dfa37/>, DEF 子と親
  D2 より本体は有限の木であり、位置が相異なれば節点も相異なる。`DEF 子と親` の子の表は各節点の子を
  その継続とアーム本体に取るので、木の親子関係そのものである。
<1>3. 3 が成り立つ。
  BY <ref id=24bf090/>, DEF 削除集合
  P15 の前半より `NodeId` は節点を一意に決める。
<1>4. `q'` と、`ρ` の上で `q` の次にある節点の入口の点とのあいだに `α` の素動作は無い。
  BY <ref id=ca36627/>, <ref id=ff5985d/>, <ref id=ba12699/>, DEF 節点の入口の点, DEF 節点の実行の素動作
  `L47` の 3 より、`α` の素動作は `ρ` の上のどれかの節点の実行に属する段のものである。`DEF 節点の入口の
  点` より、節点 `n` の実行の点は `n` の入口の点から `n'` までであり、`q'` は `q` の実行が終わった点、
  `ρ` の上で `q` の次にある節点の実行の点はその節点の入口の点から始まる。D3 より `ρ` は節点の列で
  あって、`α` は節点を 1 つ実行するごとにその路に沿って進む (D23) ので、`q` より前の節点の実行の点は
  `q` の入口の点より前にあり、`q` の次の節点より後の節点の実行の点はその入口の点より後にある。よって
  `q'` と次の節点の入口の点のあいだの点は、`ρ` の上のどの節点の実行の点でもなく、そこに `α` の素動作は
  無い。
<1>4a. そのあいだに挟まる段はどれも `α` が実行する段ではない。すなわち、環境の段か、別の制御の流れの
       活性化の段である。
  BY <1>4, <ref id=c9e4cca/>, <ref id=e3436e8/>
  A17 (iii) は「環境の動作も D24 の段としてこの実行の 1 つの列に並ぶ。段は不可分なので、環境が動くのは
  段と段のあいだである。」と述べ、続けて「**(iii) は「生きている活性化が動いていないときだけ環境が
  動く」ではない。** 複数の制御の流れがある実行では、環境の段は生きている活性化の段と交互に並ぶ」と
  書く。D24 は実行を段の列と定め、複数の制御の流れがある実行ではそれらの段が 1 つの列に並ぶと述べる。
  <1>4 よりそのあいだの段は `α` が実行する段ではないので、環境の段か、`α` 以外の活性化の段である。
  **`α` の子の活性化の段はここに入らない** -- 子が動くのは `α` が中断中の間、すなわち `α` のある節点の
  実行の中である (`DEF 節点の実行の素動作` の第 3 の箇条)。
<1>4b. `ρ` の上の入口の点は有限個である。
  BY <ref id=ca36627/>, DEF 節点の入口の点
  D3 より `ρ` は有限の列である。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>4a, <1>4b

### L30 (`drop_nodes` の作用) <!--#4a0c14c-->

**言明** --- `S` を `NodeId` の集合とする。`drop_nodes(B, S)` は、`B` の木から、`NodeId` が `S` に入る
`Retain` 節点と `Release` 節点だけを取り除いた木を返す。残る各位置について、式の変位、変数、path、
`RcState`、source、**`Let` の右辺**(`App` の callee と引数、`Closure` の `FuncRef` と capture、`Llvm` の
op とオペランド、`Var` の変数、`Match` の scrutinee)、**`Destructure` のフィールドの列**、`Match` の
アームの本数と並びとその `tag`・`payload`・`payload_state`、および継続の順序は変わらない。

**`Llvm` の op については、この「変わらない」は複製の関係である。** `Box<dyn LLVMGen>` の複製が原本と
同じオブジェクトであるとは限らない (EXT dyn_clone の trait object の複製) ので、この言明が op について
言うのは、出力の op が入力の op の複製であり、したがって A3 より同じ引数に対して原本と同じ
`Provenance`・`borrows_operand`・`applies_a_function_operand` を返すことである。

**証明**

<1>1. `drop_nodes(node, to_delete)` は `grow_stack(|| drop_nodes_inner(node, to_delete))` であり、A15 より
      `drop_nodes_inner` をちょうど 1 回呼んでその値を返す。
  BY CODE src/rc_ir/borrow.rs: drop_nodes, <ref id=3e6b0e0/>
<1>2. `drop_nodes` が読む `NodeId` は入力の木のものであり、走査が読んだものと同じである。すなわち、木の
      1 つの位置について、走査が計算した `node_id` の値と `drop_nodes` が計算する値は等しい。
  <2>1. `cancel_body` は 1 つの共有参照 `body: &RcExprNode` について `analysis.walk(body, ・, ・)` を
        呼び、その値から作った集合を持って `drop_nodes(body, &analysis.cancelled())` を呼ぶ。`body` は
        `prog: &RcProgram` から借用したものであり、この 2 つの呼び出しの間に木を変える操作は無い --
        `cancel` が持つのは共有参照 `prog: &RcProgram` だけで、EXT 共有参照は代入を許さない より
        その先の値へ代入することはできず、`funcs` と `globals` を作る写像はそれぞれの `f.body` /
        `g.init` を読むだけである。**内部可変性の例外はここでは働かない** -- A3 は
        「**`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が変えない。** 到達できる型が
        内部可変性を持つ欄を持つときは、その欄は**一度だけ書かれる memo であって、その値はその型の
        `PartialEq` が読む成分の関数である**」と述べ、D1a は「**内部可変性の memo も成分ではない** (A3)。」と
        書く。すなわちその欄が埋まっても D1a の意味の木は変わらず、D2 が節点と呼ぶ位置の集合も各位置の
        内容も動かない。
    BY CODE src/rc_ir/borrow.rs: cancel, EXT 共有参照は代入を許さない, <ref id=e11772a/>, <ref id=1c00537/>, <ref id=b3dfa37/>
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
        `RcVar` は `#[derive(Clone)]` を持ち、`FieldPath` は `Vec<usize>` なので、EXT derive(Clone)・
        EXT Vec::clone・EXT 標準の型の clone は等しい値を返す より、積む節点の変数と path は元のものに
        等しい。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Retain(v, path, state, k)` の腕,
       CODE src/rc_ir/ast.rs: RcVar, CODE src/rc_ir/ast.rs: FieldPath,
       EXT derive(Clone), EXT Vec::clone, EXT 標準の型の clone は等しい値を返す
  <2>2. CASE `node` の式が `RcExpr::Release(v, path, state, k)` である。この腕は
        `drop_nodes(k, to_delete)` を 1 回呼び、`to_delete` が `node_id(node)` を含むときはその値を
        そのまま返し、含まないときは `RcExpr::Release(v.clone(), path.clone(), *state, k)` の節点を
        `&node.source` を付けて積んで返す。変数と path が元のものに等しいことは <2>1 と同じ 3 つの
        名札が与える。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Release(v, path, state, k)` の腕,
       CODE src/rc_ir/ast.rs: RcVar, CODE src/rc_ir/ast.rs: FieldPath,
       EXT derive(Clone), EXT Vec::clone, EXT 標準の型の clone は等しい値を返す
  <2>3. CASE `node` の式が `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` である。この腕は各アームに
        ついて `drop_nodes(&arm.body, to_delete)` を 1 回、`drop_nodes(k, to_delete)` を 1 回呼び、
        `x`、`scrut`、各アームの `tag`/`payload`/`payload_state`、アームの本数と並びを変えずに、
        `&node.source` を付けて節点を積む。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, RcRhs::Match(scrut, arms), k)` の腕,
       CODE src/rc_ir/ast.rs: MatchArm, CODE src/rc_ir/ast.rs: MatchArm::with_body,
       CODE src/rc_ir/ast.rs: RcVar, EXT derive(Clone), EXT Iterator::map と collect,
       EXT 標準の型の clone は等しい値を返す
    `x` と `scrut` は `x.clone()`・`scrut.clone()` で運ばれ、`RcVar` は `#[derive(Clone)]` を持つので
    EXT derive(Clone) と EXT 標準の型の clone は等しい値を返す より元のものに等しい。アームは
    `arms.iter().map(|arm| arm.with_body(..)).collect()` で作られ、`MatchArm::with_body` は
    `MatchArm { body, ..self.clone() }` を返すので、EXT derive(Clone) より `tag`・`payload`・
    `payload_state` は元のものの `clone` であり、EXT Iterator::map と collect より本数と並びは
    元のものである。
  <2>4. CASE `node` の式が `RcExpr::Let(x, rhs, k)` で `rhs` が `RcRhs::Match(..)` でない。この腕は
        `drop_nodes(k, to_delete)` を 1 回呼び、`x` と `rhs` を `clone()` で運んで `&node.source` を
        付けて節点を積む。すなわち `App` の callee と引数、`Closure` の `FuncRef` と capture、`Var` の
        変数は元のものである。`to_delete` の検査はしない。`match` の腕はこの順に
        並んでいるので、この腕に落ちる `rhs` は `RcRhs::Match` ではない。
        **`Llvm` の op については、複製の関係が得られる** -- `RcRhs` は `#[derive(Clone)]` を持つので
        EXT derive(Clone) より `rhs.clone()` は `RcRhs::Llvm` の第 1 欄に `Box<dyn LLVMGen>` の
        `Clone::clone` の値を置き、`LLVMGen` の宣言は `pub trait LLVMGen: DynClone + Send + Sync` で
        あって `dyn_clone` の `DynClone` を継承するので、EXT dyn_clone の trait object の複製 が
        その `Clone` を述べる。**複製が原本と同じオブジェクトであるとは限らない**ので、言明が op に
        ついて言うのはこの複製の関係と、A3 の「**この 2 節を合わせると「op の複製は原本と同じ宣言を
        返す」が出る。** `rhs.clone()` や `fresh_rename_function` が作る複製の op は、原本と同じ引数を
        渡されれば同じ `Provenance` を返す」である。オペランドは `Vec<RcVar>` なので EXT Vec::clone と
        EXT derive(Clone) より元のものに等しい。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Let(x, rhs, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner, CODE src/rc_ir/ast.rs: RcRhs,
       CODE src/rc_ir/ast.rs: RcVar, CODE src/ast/inline_llvm.rs: LLVMGen, <ref id=e11772a/>,
       EXT derive(Clone), EXT Vec::clone, EXT 標準の型の clone は等しい値を返す,
       EXT dyn_clone の trait object の複製
  <2>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)` または `RcExpr::Eval(v, k)`
        である。この 2 つの腕は `drop_nodes(k, to_delete)` を 1 回呼び、`container`・`fields`・`state`・
        `v` を `clone()` で運んで `&node.source` を付けて節点を積む。すなわち `Destructure` の
        フィールドの列は元のものである。`to_delete` の検査はしない。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Destructure(container, fields, state, k)` の腕,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Eval(v, k)` の腕,
       EXT Vec::clone, EXT derive(Clone)
  <2>6. CASE `node` の式が `RcExpr::Ret(v)` である。この腕は `RcExpr::Ret(v.clone())` の 1 節点を
        `&node.source` を付けて作って返す。`RcVar` は `#[derive(Clone)]` を持つので、
        EXT derive(Clone) と EXT 標準の型の clone は等しい値を返す より、その変数は元のものに等しい。
    BY CODE src/rc_ir/borrow.rs: drop_nodes_inner の `RcExpr::Ret(v)` の腕,
       CODE src/rc_ir/ast.rs: RcVar, EXT derive(Clone), EXT 標準の型の clone は等しい値を返す
  <2>7. QED
    `RcExpr` の 6 変位のうち `Let` を右辺で 2 つに分けた 7 つの場合を <2>1 から <2>6 が尽くす。これは
    `drop_nodes_inner` の `match` の 7 つの腕である。節点を落とすのは <2>1 と <2>2 だけであり、どちらも
    `Retain`/`Release` である。**継続とアーム本体については帰納法の仮定を当てる** -- <2>1 から <2>6 の
    各腕が `drop_nodes` に渡すのは `node` の子 (`DEF 子と親`) であり、それは `N(node)` より真に小さい
    部分木なので、その返り値が言明のとおりの木であることは帰納法の仮定が与える。各腕はその返り値を
    そのまま継続 (アームの場合はアーム本体) に据えるので、言明は `N(node)` の全体について成り立つ。
    **残る各位置の source が変わらないのは、どの腕も `expr_node(・, &node.source)` で節点を作るから
    である** -- `expr_node(expr, source)` は `RcExprNode { expr: Arc::new(expr), source: source.clone() }`
    を返す。
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, 帰納法の仮定, CODE src/rc_ir/ast.rs: RcExpr,
       CODE src/rc_ir/ast.rs: RcRhs, CODE src/rc_ir/ast.rs: RcExprNode,
       CODE src/rc_ir/borrow.rs: drop_nodes_inner, CODE src/rc_ir/borrow.rs: expr_node,
       DEF 子と親, EXT derive(Clone), EXT 標準の型の clone は等しい値を返す
<1>4. QED
  BY <1>1, <1>2, <1>3

### L31 (路の対応) <!--#2bb344b-->

**言明** --- `Del` の要素がすべて `Retain` 節点か `Release` 節点であるとする。`B` の実行路 (D3) から
`Del` の節点を除く写像は、`B` の実行路の全体から `B'` の実行路の全体への全単射である。

**証明**

<1>1. `B'` の木は `B` の木から `Del` の `Retain`/`Release` 節点を抜いたものであり、`Match` のアームの
      本数と並び、および各節点の継続の順序は変わらない。
  BY <ref id=4a0c14c/>, 本命題の仮定
<1>2. D3 の実行路は、根から継続をたどり、各 `Match` でアームを 1 つ選ぶことで決まる。<1>1 より `B` と
      `B'` は同じ `Match` 節点の同じアームの並びを持つので、アームの選択の全体は 2 つの木で同じである。
      1 つの選択に `B` の実行路 1 本と `B'` の実行路 1 本が対応し、後者は前者から `Del` の節点を除いた
      列である。
  BY <ref id=ca36627/>, <1>1
<1>3. QED
  BY <1>1, <1>2

### L32 (削除集合の構造) <!--#7d5a1de-->

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
        `p30` の `L10` より走査は記録を取り除かない。**「その腕だけ」を与えるのは
        `前提 走査の記録の書き込みの在りか` である** -- その走査が `self.all_retains` について挙げるのは
        `walk_inner` と `cancelled` の 2 項目であり、`cancelled` はそれを読むだけである。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, <ref id=3b81b08/>,
       前提 走査の記録の書き込みの在りか
  <2>2. QED
    BY <2>1, CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, DEF 削除集合
<1>2. `self.all_retains` に値が入るのは、`walk_inner` の `RcExpr::Retain(v, path, _, k)` の腕の
      `self.all_retains.push(retain)` だけであり、そこで `retain = node_id(node)` の `node` はいま訪問して
      いる `Retain` 節点である。よって 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: node_id, <1>1, 前提 走査の記録の書き込みの在りか
  `前提 走査の記録の書き込みの在りか` の走査が `self.all_retains` について挙げるのは `walk_inner` と
  `cancelled` の 2 項目であり、`cancelled` はそれを読むだけである (`CODE src/rc_ir/borrow.rs:
  CancelAnalysis::cancelled`)。`walk_inner` の中でそれへ書き込む式はこの `push` である。
<1>3. `self.un_bump_releases` の値の `Vec` に要素が入るのは、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の `UnBump::InBracket(retain)` の枝の
      `self.un_bump_releases.entry(retain).or_default().push(node_id(node))` だけであり、そこで `node` は
      いま訪問している `Release` 節点である。よって 2 と 4 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::cancelled, CODE src/rc_ir/borrow.rs: CancelAnalysis,
     CODE src/rc_ir/borrow.rs: cancel,
     CODE src/rc_ir/borrow.rs: node_id, 前提 走査の記録の書き込みの在りか
  `前提 走査の記録の書き込みの在りか` の走査が `un_bump_releases` について挙げるのは `cancel`・
  `CancelAnalysis`・`walk_inner`・`cancelled` の 4 項目である。`cancel` は構成子で空の写像を置き、
  `CancelAnalysis` は欄の宣言であり、`cancelled` の `get` は読むだけである。残る `walk_inner` の中で
  値の `Vec` に要素を入れるのはこの `push` だけであり、同じ腕の `entry(retain).or_default()` は
  空の `Vec` を据えるだけである。
<1>4. 1 つの `Release` 節点は、高々 1 つの `t` の `un_bump_releases[t]` に、高々 1 回入る。
  <2>1. P15 の後半より、走査は `Release` 節点 `r` をちょうど 1 回訪問する。
    BY <ref id=24bf090/>
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
  BY <1>2, <1>3, <ref id=24bf090/>
<1>6. QED
  3 は <1>1、<1>4、<1>5 から従う。5 は <1>2、<1>3、<1>1 から従う。1 は <1>2、2 は <1>3、4 は <1>3 で
  ある。
  BY <1>1, <1>2, <1>3, <1>4, <1>5

### L33 (出口状態は継続終端の入口状態である) <!--#4fe00a7-->

**言明** --- 任意の節点 `n` について、`pending_out(n) = pending(ret(n))` である。

**証明** `n` から `ret(n)` への継続の鎖の長さについての帰納法で示す。D2 より鎖は有限である。

<1>1. CASE `n` の式が `RcExpr::Ret(_)` である。`walk_inner` のこの腕は `pending` をそのまま返すので
      `pending_out(n) = pending(n)` であり、DEF 訪問 より `ret(n) = n` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕, DEF 訪問
<1>2. CASE `n` の式が `RcExpr::Ret(_)` でない。`walk_inner` の残る 6 つの腕はいずれも
      `self.walk(k, ・, ・)` の値をそのまま返す (`k` は `n` の継続)。`p30` の `L1` より `walk` は
      `walk_inner` を 1 回呼んでその値を返すので、`pending_out(n) = pending_out(k)` である。DEF 訪問 より
      `ret(n) = ret(k)` なので、帰納法の仮定が使える。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, <ref id=dad309f/>, DEF 訪問
<1>3. QED
  `RcExpr` の 6 変位のうち `Ret` を <1>1 が、残る 5 変位 (`Let` は右辺で 2 つの腕に分かれるが、どちらも
  `self.walk(k, ・, ・)` の値を返す) を <1>2 が尽くす。
  BY <1>1, <1>2, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L33a (節点から辿った列の最後は継続終端である) <!--#68112c9-->

**言明** --- 節点 `n` から D3 の 3 つの規則を適用して得られる列は有限であり、その最後の節点は `ret(n)`
である。とくに、本体 `B` の実行路の最後の節点は `ret(B の根)` であり、アーム本体 `arm.body` から辿った
列の最後の節点は `ret(arm.body)` である。

**証明** 部分木 `N(n)` の節点数についての帰納法で示す。DEF 子と親 より子の部分木は `N(n)` より真に
小さく、D2 より本体は有限の木なので、この帰納法は整礎である。

<1>1. CASE `n` の式が `RcExpr::Ret(_)` である。D3 より `Ret` からは辿る先が無いので列は `n` だけで
      あり、DEF 訪問 より `ret(n) = n` である。
  BY <ref id=ca36627/>, DEF 訪問
<1>2. CASE `n` の式が `Let(_, Match(_, arms), k)` である。D3 のこの規則より、列はアームを 1 つ選んで
      その本体から辿った列を挟み、その後 `k` から辿った列へ続く。前者は帰納法の仮定より有限であり、
      後者の最後の節点は帰納法の仮定より `ret(k)` である。DEF 訪問 より `ret(n) = ret(k)` である。
  BY <ref id=ca36627/>, DEF 訪問, DEF 子と親, 帰納法の仮定
<1>3. CASE `n` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が `Match` でない `Let` で
      ある。D3 のこの規則より、列は `n` の継続 `k` から辿った列へ続く。帰納法の仮定よりその最後の節点は
      `ret(k)` であり、DEF 訪問 より `ret(n) = ret(k)` である。
  BY <ref id=ca36627/>, DEF 訪問, DEF 子と親, 帰納法の仮定
<1>4. QED
  `RcExpr` の 6 変位のうち `Ret` を <1>1、右辺が `Match` の `Let` を <1>2、残り (`Let` の残る形、
  `Retain`、`Release`、`Destructure`、`Eval`) を <1>3 が尽くす。本体の実行路は根から辿った列であり
  (D3)、アーム本体の実行路はそのアーム本体の根から辿った列である (D3)。
  BY <1>1, <1>2, <1>3, <ref id=ca36627/>, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L34 (路に沿った状態の遷移) <!--#7af2e2e-->

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
  BY <ref id=ca36627/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, <ref id=dad309f/>, <ref id=13b0da2/>, <ref id=941af96/>
  `pending(n)` と `pending(k)` が節点ごとに 1 つに定まることは L50 の 1 が述べる。
<1>2. CASE `n` の式が `Let(_, Match(_, arms), k)` である。D3 より `ρ` の上の `n` の直後の節点は、`ρ` が
      選んだアームの本体である。`walk_inner` のこの腕は各アームについて
      `self.walk(&arm.body, pending.clone(), false)` を呼ぶ。`PendingRetains` は `Vec<PendingRetain>` で
      あり、`PendingRetain` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の `node` と
      `outstanding` は原本のものの `clone` である。**その 2 つの `clone` は原本と等しい値を返す** --
      `node` の型 `NodeId` は `usize` であり、`outstanding` の型 `References` は
      `Map<VarPath, usize>` を 1 つ持つ構造体であって `#[derive(Clone)]` を持ち、`VarPath` は
      `(FullName, FieldPath)`、`FieldPath` は `Vec<usize>` である。この型の組み合わせについて
      EXT 標準の型の clone は等しい値を返す がそれを与える。EXT Vec::clone より、複製の
      長さと各位置の要素は原本のものに等しいので、アーム本体の入口状態は `pending(n)` と等しい値である。
  BY <ref id=ca36627/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     CODE src/rc_ir/borrow.rs: PendingRetain, CODE src/rc_ir/borrow.rs: PendingRetains,
     CODE src/rc_ir/borrow.rs: NodeId, CODE src/rc_ir/ownership.rs: References,
     CODE src/rc_ir/ast.rs: VarPath, FieldPath,
     EXT Vec::clone, EXT derive(Clone), EXT 標準の型の clone は等しい値を返す, <ref id=dad309f/>
<1>3. CASE `n` がアーム本体の終端の `Ret` である。D3 より、`Ret` の後に実行路が続くのは、`n` が、その
      実行路が入ったアームの本体の実行路を終える `Ret` であるときに限り、そのとき直後の節点はその `Match`
      節点 `M` の継続 `k_M` である。L33a より `n = ret(arm_i.body)` である。`walk_inner` の `M` の腕は
      `arm_exits` を集めてから `let merged = self.merge(&pending, &arm_exits);` を作り、
      `self.walk(k, merged, returns_from_func)` を呼ぶので `pending(k_M) = merged` である。
      `arm_exits[j]` は `self.walk(&arm_j.body, ・, ・)` の返り値、すなわち `pending_out(arm_j.body)` で
      ある。L33 より `pending_out(arm_i.body) = pending(ret(arm_i.body)) = pending(n)` である。
  BY <ref id=ca36627/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(_, RcRhs::Match(_, arms), k)` の腕,
     <ref id=dad309f/>, <ref id=4fe00a7/>, <ref id=68112c9/>
<1>4. QED
  `ρ` の上に直後の節点があるのは、`n` の式が `Ret` でないか、`n` がアーム本体の終端の `Ret` であるかの
  どちらかである (D3)。前者を <1>1 と <1>2 が `RcExpr` の変位で尽くし、後者を <1>3 が扱う。
  BY <1>1, <1>2, <1>3, <ref id=ca36627/>, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs

### L35 (`pending` の要素はその節点を支配する) <!--#5b098d9-->

**言明** --- 走査が訪問する節点 `n` と `pending(n)` の要素 `e` について、`e` の由来 (DEF 訪問) を `t` と
する。このとき、`n` を含むすべての実行路は `t` を含み、その路の上で `t` は `n` より真に前にある。

**証明** 訪問順序についての帰納法で示す。P15 の後半より走査は各位置をちょうど 1 回訪問するので、訪問順序は
有限の全順序であり、この帰納法は整礎である。

<1>1. 帰納法の仮定: `n` より前に訪問された各節点 `m` と `pending(m)` の各要素について、言明が成り立つ。
  BY 帰納法の仮定
<1>2. 木の根でない各節点 `n` について、`n` を含むすべての実行路は、`n` の親 (DEF 子と親) を含み、その路の
      上で親は `n` より真に前にある。
  <2>1. D2 より本体は木であり、根でない各節点はちょうど 1 つの親を持つ。
    BY <ref id=b3dfa37/>
  <2>2. D3 の規則のうち、実行路に `n` を加えるのは次の 3 つの場合だけである。`n` が根である場合、`n` が
        直前の節点の継続である場合、`n` がアーム本体の根である場合。`n` は根でないので第 1 の場合は
        起こらない。第 2 の場合の直前の節点は `n` の親か、`n` の親が `Match` であるときそのアーム本体の
        終端の `Ret` であり、後者のときも D3 よりその `Match` 節点自身が路の上でより前にある。第 3 の
        場合の直前の節点は `n` の親である `Match` 節点である。
    BY <ref id=ca36627/>
  <2>3. QED
    BY <2>1, <2>2
<1>3. CASE `n` が `B` の根である。`cancel_body` は `analysis.walk(body, PendingRetains::default(), true)`
      を呼ぶので `pending(n)` は空であり、言明は空虚に成り立つ。
  BY CODE src/rc_ir/borrow.rs: cancel, <ref id=dad309f/>, DEF 訪問
  `DEF 訪問` は `pending(n)` を `n` の訪問における `walk_inner` の `pending` 引数と定め、`p30` の
  `L1` は `walk` が受け取った引数をそのまま渡して `walk_inner` を 1 回呼び、その値を返すことを
  述べるので、`walk` に渡した `PendingRetains::default()` がそのまま `pending(n)` である。
<1>4. CASE `n` が根でなく、その親 `m` の式が `Retain`、`Release`、`Destructure`、`Eval`、または右辺が
      `Match` でない `Let` である。
  <2>1. `walk_inner` の `m` の腕は `pending(m)` に操作を施して `self.walk(n, pending, ・)` を呼ぶ。
        `pending` に要素を加えるのは `RcExpr::Retain` の腕の `pending.push(PendingRetain { node: retain,
        outstanding })` 1 か所だけであり、そこで加わる要素の `node` は `m` 自身の `NodeId` である。
        `consume_objects` と `un_bump` は要素を加えない。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
       CODE src/rc_ir/borrow.rs: un_bump, <ref id=19296b2/>, <ref id=13b0da2/>
  <2>2. `pending(n)` の各要素の `node` は、`pending(m)` のある要素の `node` であるか、`m` 自身の
        `NodeId` である (`m` が `Retain` のとき)。したがってその要素の由来 (DEF 訪問) は、
        `pending(m)` のある要素の由来であるか、`m` 自身である。
    BY <2>1, <ref id=7af2e2e/>, <ref id=19296b2/>, <ref id=24bf090/>, DEF 訪問
    **`outstanding` の値については同じことを言わない** -- L34 の 1 の `un_bump` は、選んだ要素の
    `outstanding` から引くので、要素の値は `pending(m)` のものと等しいとは限らない。由来は `node` だけで
    決まり (DEF 訪問、P15 の前半)、`un_bump` は `node` を書き替えないので、この段が言うのは `node` に
    ついてである。
  <2>3. QED
    `pending(m)` のある要素の由来であるものについては <1>1 と <1>2 を合わせる。由来が `m` 自身のものに
    ついては <1>2 が直接与える。
    BY <1>1, <1>2, <2>2
<1>5. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n` がその
      アームの本体である。L34 の 2 より `pending(n)` の要素の `node` の集合は `pending(m)` のそれに等しい
      ので、<1>1 と <1>2 を合わせる。
  BY <1>1, <1>2, <ref id=7af2e2e/>
<1>6. CASE `n` が根でなく、その親 `m` の式が `Let(_, RcRhs::Match(_, arms), k)` であり、`n = k` で
      ある。L34 の 3 より `pending(n) = merge(pending(m), arm_exits)` である。P18 の第 1 の主張より、
      `merge` の返り値に残る `Retain` は `pending_in` すなわち `pending(m)` に在るものだけである。よって
      <1>1 と <1>2 を合わせる。
  BY <1>1, <1>2, <ref id=7af2e2e/>, <ref id=5116349/>
<1>7. QED
  `n` は根であるか、親を持つ。親を持つ場合、DEF 子と親 の子の表より、親の式は <1>4、<1>5、<1>6 の
  いずれかの形である。
  BY <1>3, <1>4, <1>5, <1>6, <ref id=b3dfa37/>

### L36 (`consume_objects` の呼び出し箇所) <!--#7855e90-->

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
  BY <ref id=13b0da2/>
<1>2. `self.consume_objects(...)` と書かれているのは 3 か所である。`CancelAnalysis::consume`
      の末尾の `self.consume_objects(pending, &objects)`、`walk_inner` の
      `RcExpr::Release(v, path, _, k)` の腕の `let others = self.other_objects(v, path);` の直後の
      `self.consume_objects(&mut pending, &others)`、および同じ腕の `UnBump::OutsideBracket` の枝の
      `self.consume_objects(&mut pending, &objects)` である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, 前提 消費の呼び出しの在りか
  `前提 消費の呼び出しの在りか` の走査が `self.consume_objects(` について挙げるのは `walk_inner` と
  `consume` の 2 項目である。`consume` の中の 1 つと、`walk_inner` の `RcExpr::Release` の腕の 2 つが
  その全部である。
<1>3. `consume(pending, var, path)` は `origin(self.vars, self.type_env, var, path).acted_on()` の元を
      `objects` に集め、`self.consume_objects(pending, &objects)` を 1 回呼ぶ。すなわち
      `objects = acted_on(var, path)` である。`consume` を呼ぶのは、`consume_rhs` と、`walk_inner` の
      `RcExpr::Destructure(container, fields, _state, k)` の腕の 2 か所である。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume, DEF 本体ごとの記法,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Destructure(container, fields, _state, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, 前提 消費の呼び出しの在りか
  `前提 消費の呼び出しの在りか` の走査が `self.consume(` について挙げるのは `walk_inner` と
  `consume_rhs` の 2 項目であり、`walk_inner` の中でその呼び出しを持つのは `RcExpr::Destructure` の
  腕だけである。
<1>4. `consume_rhs(pending, rhs, result_ty)` は `rhs_consumes` が `consumed` に積んだ各 `(var, leaf)` に
      ついて `self.consume(pending, &var, &leaf)` を呼ぶ。`consume_rhs` を呼ぶのは `walk_inner` の
      `RcExpr::Let(x, rhs, k)` の腕 1 か所だけであり、その腕には右辺が `Match` の `Let` は入らない
      (`match` の腕がその先に置かれているため)。よって <1>3 と合わせて 1 が成り立つ。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Let(x, rhs, k)` の腕,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, <1>3, 前提 消費の呼び出しの在りか
  `前提 消費の呼び出しの在りか` の走査が `self.consume_rhs(` について挙げるのは `walk_inner` の
  1 項目であり、その中でその呼び出しを持つのは `RcExpr::Let` の腕だけである。
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
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, 前提 消費の呼び出しの在りか

### L37 (削除される `Release` は `t` より後にある) <!--#23ac734-->

**言明** --- `t ∈ CT` と `r ∈ un_bump_releases[t]` について、`t` は `r` で pending であり (DEF 訪問)、
`r` を含むすべての実行路は `t` を含み、その路の上で `t` は `r` より真に前にある。

**証明**

<1>1. L32 の 4 より、`r` の訪問の中の `un_bump(&mut pending, &un_bumped)` の呼び出しが `InBracket(t)` を
      返した。
  BY <ref id=7d5a1de/>
<1>2. `p30` の `L5` の 3 より、`un_bump` が `InBracket(t)` を返すのは、その第 1 引数の `pending` に、
      `un_bumped` と**位置を共有する**要素があり、そのうち最も後ろの要素の `node` が `t` の `NodeId`
      であるときである。すなわちその要素は由来が `t` の要素である。`p30` の `L5` は「要素 `e` が
      `References` の値 `R` と**位置 (D6) を共有する**とは、`e.outstanding.shares_an_object(R)` が真で
      あることをいう」と定め、その鍵が `VarPath` -- この文書の名前 (`DEF 名前とオブジェクト`) -- で
      あることを述べる。
  BY <ref id=19296b2/>, <1>1, DEF 名前とオブジェクト
<1>3. <1>2 の第 1 引数は、`r` の訪問が `un_bump` を呼ぶところの `pending` であり、それは `pending(r)` に、
      この腕がそれより前に行う `others(r)` についての `consume_objects` を施したものである。L36 より
      `consume_objects` は要素を取り除くだけで加えないので、由来が `t` の要素は `pending(r)` にも在る。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release(v, path, _, k)` の腕,
     <ref id=7855e90/>, <ref id=941af96/>, <1>2
  `pending(r)` が 1 つに定まり、`un_bump_releases[t]` の要素を節点と同一視してよいことは L50 の
  1 と 3 が述べる。
<1>4. QED
  BY <1>3, <ref id=5b098d9/>

### L38 (`t` が pending である区間) <!--#5adaf7f-->

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
     DEF 節点の量, <ref id=7af2e2e/>, <ref id=b3dfa37/>, <ref id=72bac67/>, <ref id=941af96/>
  `t` の訪問が積む `outstanding` は `DEF 節点の量` の `ActRefs(t)` であり、その値が走査のどの段階で
  読んでも同じであることは L46 が述べる。`pending(n0)` と `e_t(n0)` が 1 つに定まることは L50 の 1 で
  ある。
<1>2. `ρ` の上で `t` より後の 2 つの節点 `n`、`n'` が `ρ` の上で隣り合い、`t` が `n` で pending でない
      ならば、`t` は `n'` でも pending でない。
  <2>1. CASE L34 の 1。`pending(n')` は `pending(n)` に `push`・`consume_objects`・`un_bump` を施した
        ものである。要素を加えるのは `push` だけであり、そこで加わる要素の `node` は `n` 自身の
        `NodeId` である。`n` は `Retain` 節点 `t` とは相異なる位置なので、P15 の前半よりその `NodeId` は
        `t` の `NodeId` と異なる。`consume_objects` と `un_bump` は要素を加えない。
    BY <ref id=7af2e2e/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕,
       <ref id=24bf090/>, <ref id=7855e90/>, <ref id=19296b2/>
  <2>2. CASE L34 の 2。`pending(n')` は `pending(n)` の複製であり、要素の `node` の集合は等しい。
    BY <ref id=7af2e2e/>
  <2>3. CASE L34 の 3。P18 の第 1 の主張より、`merge` の返り値に残る `Retain` は、いずれのアームの出口
        にも現れるものだけである。`arm_exits[i] = pending(n)` は由来が `t` の要素を持たないので、`t` は
        `merge` の返り値にも入らない。
    BY <ref id=7af2e2e/>, <ref id=5116349/>
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=7af2e2e/>
<1>3. `t` が pending である `ρ` の上の節点の全体は、<1>1 の `n0` から始まる連続する区間である。
  BY <1>1, <1>2
<1>4. `t` は `ρ` の終端の `Ret` では pending でない。
  <2>1. `returns_from_func` が真である節点の全体は、`B` の根から継続だけをたどって得られる鎖であり、
        その鎖に入る唯一の `Ret` 節点は `ret(B の根)` である。
    <3>1. `B` の根の訪問が受け取る `returns_from_func` は真である。`cancel_body` は
          `analysis.walk(body, ・, true)` を呼び、`p30` の `L1` より `walk` は受け取った引数を
          そのまま渡して `walk_inner` を 1 回呼ぶ。
      BY CODE src/rc_ir/borrow.rs: cancel, <ref id=dad309f/>, DEF 訪問
    <3>2. `walk_inner` は `Match` のアーム本体には `false` を渡し、ほかの継続には自分が受け取った
          `returns_from_func` をそのまま渡す。`p30` の `L1` より `walk` はその値をそのまま
          `walk_inner` へ渡す。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, <ref id=dad309f/>
    <3>3. QED
      BY <3>1, <3>2, <ref id=24bf090/>, <ref id=b3dfa37/>, DEF 子と親, DEF 訪問
      訪問順序についての帰納法で示す。P15 の後半より走査は各位置をちょうど 1 回訪問するので、訪問順序は
      有限の全順序であり、この帰納法は整礎である。根については <3>1 が真を与える。根でない節点 `n` は
      その親 `m` の訪問から訪問される (`DEF 子と親`、D2)。`n` が `Match` のアーム本体であれば <3>2 より
      `returns_from_func` は偽であり、そうでなければ `n` は `m` の継続であって、<3>2 より `n` の
      `returns_from_func` は `m` のものに等しい。よって `returns_from_func` が真である節点は、根から
      継続だけをたどって得られる鎖の元に限られ、その各元では真である。D2 より継続の鎖は有限で `Ret` で
      終わるので、その鎖に入る `Ret` 節点は `ret(B の根)` ただ 1 つである (`DEF 訪問`)。
  <2>2. L33a より、`ρ` の最後の節点は `ret(B の根)` である。
    BY <ref id=68112c9/>
  <2>3. `walk_inner` の `RcExpr::Ret(_)` の腕は、`returns_from_func` が真のとき `pending` の全要素の
        `node` を `self.needed_retains` に入れる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Ret(_)` の腕
  <2>4. QED
    `t` がそこで pending なら <2>3 より `t` が `needed_retains` に入り、`p30` の `L10` より走査の終わりまで
    残る。これは `t ∈ CT` (DEF 削除集合) に反する。
    BY <2>1, <2>2, <2>3, <ref id=3b81b08/>, DEF 削除集合
<1>5. 4 が成り立つ。
  <2>1. `I_ρ(t)` に入る節点 `n` の訪問の中で `consume_objects(pending, objects)` が走るところで、由来が
        `t` の要素 `e` が走査の `pending` に在り、`objects` のいずれかの名前を `e.outstanding` が名指す
        と仮定する。
    BY 仮定
  <2>2. L36 よりこの呼び出しは `e` を取り除き、`t` を `self.needed_retains` に入れる。
    BY <ref id=7855e90/>, <2>1
  <2>3. QED
    `p30` の `L10` より `t` は走査の終わりまで `needed_retains` に残り、`t ∈ CT` に反する。よって <2>1 の
    仮定は成り立たない。L36 より `consume_objects` が要素を取り除くのはその条件が成り立つときだけなので、
    `e` は `consume_objects` に取り除かれない。
    BY <2>2, <ref id=3b81b08/>, DEF 削除集合, <ref id=7855e90/>
<1>6. 1 が成り立つ。
  <2>1. <1>3 と <1>4 より区間は `ρ` の終端の `Ret` より前で終わるので、区間の最後の節点 `n*` があり、
        `ρ` の上でその直後の節点 `n'` について `t` は `n'` で pending でない。
    BY <1>3, <1>4
  <2>2. CASE L34 の 3 (`n*` がアーム本体の終端の `Ret`)。`arm_exits[i] = pending(n*)` が由来 `t` の要素を
        持ち、`merge` の返り値が持たない。P18 の第 2 の主張より、この `merge` の呼び出しは `t` を
        `self.needed_retains` に入れる。`p30` の `L10` よりそれは走査の終わりまで残るので `t ∈ CT` に
        反する。よってこの場合は起こらない。
    BY <ref id=7af2e2e/>, <ref id=5116349/>, <ref id=3b81b08/>, DEF 削除集合
  <2>3. CASE L34 の 2 (`n*` が `Match` 節点)。`pending(n')` は `pending(n*)` の複製なので `t` は `n'` で
        pending であり、`n*` が区間の最後であることに反する。よってこの場合は起こらない。
    BY <ref id=7af2e2e/>
  <2>4. CASE L34 の 1。`walk_inner` のこれらの腕が `pending` から要素を取り除くのは、`consume_objects` と
        `un_bump` の 2 つだけである。<1>5 より `consume_objects` は由来 `t` の要素を取り除かない。よって
        取り除いたのは `un_bump` である。`p30` の `L5` より `un_bump` が要素を取り除くのは第 3 の場合、
        すなわち `InBracket` を返し、選んだ要素の `outstanding` から `un_bumped` を引いた結果が空に
        なったときだけであり、そのとき返る `NodeId` は取り除かれた要素の `node` すなわち `t` である。
        `un_bump` を呼ぶのは `RcExpr::Release(v, path, _, k)` の腕だけなので、`n*` は `Release` 節点で
        ある -- `前提 `un_bump` の呼び出しの在りか` の走査が挙げるのは自由関数 `un_bump` 自身の定義と
        `walk_inner` の 2 項目であり、`walk_inner` の中でその呼び出しを持つのはその腕だけである。
    BY <ref id=7af2e2e/>, <1>5, <ref id=19296b2/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
       CODE src/rc_ir/borrow.rs: un_bump, 前提 `un_bump` の呼び出しの在りか
  <2>5. QED
    `n*` は `ρ` の上に直後の節点を持つので、L34 の 3 つの場合のいずれかである。<2>2 と <2>3 がそのうち
    2 つを排除する。L32 の 4 より `n*(ρ) ∈ un_bump_releases[t]` である。
    BY <2>1, <2>2, <2>3, <2>4, <ref id=7af2e2e/>, <ref id=7d5a1de/>
<1>7. 2 が成り立つ。
  BY <1>3, <1>4, <1>6
<1>8. 3 が成り立つ。
  <2>1. `R_ρ(t)` の各要素は L32 の 4 より `un_bump_releases[t]` の要素であり、`t ∈ CT` なので L32 の 3 より
        `Del` の要素である。<1>6 より `n*(ρ) ∈ R_ρ(t)` である。
    BY <ref id=7d5a1de/>, <1>6
  <2>2. `I_ρ(t)` の上で、由来が `t` の要素の `outstanding` は、区間の最初の節点で `ActRefs(t)` であり
        (<1>1)、`ρ` の上で隣り合う 2 つの節点 `n`、`n'` (どちらも区間に入る) の間で次のように変わる。
        L34 の 1 では、`consume_objects` は残る要素の値を変えず (L36)、`un_bump` は `InBracket` で選んだ
        要素の `outstanding` からだけ引く (`p30` の `L5`)。よって `un_bump` が `InBracket(t)` を返す
        遷移では `ActRefs(n)` が引かれ、それ以外の遷移では変わらない。L34 の 2 では複製なので変わらない。
        L34 の 3 では、`n` はアーム `arm_i` の本体の終端の `Ret` であり、`merge` が
        `uniform.get(&retain.node)` の複製を新しい `outstanding` に据える。P18 よりその値はすべての
        アームの出口に現れる共通の値であり、L34 の 3 より `arm_exits[i] = pending(n)` なので、その共通の
        値は `out(t, n)` に等しい。よって変わらない。**`merge` が据える `outstanding.clone()` は原本と
        等しい値である** -- `References` は `Map<VarPath, usize>` を 1 つ持つ構造体であって
        `#[derive(Clone)]` を持ち、`VarPath` は `(FullName, FieldPath)`、`FieldPath` は `Vec<usize>` で
        あるので、EXT derive(Clone)・EXT Vec::clone・EXT 標準の型の clone は等しい値を返す がそれを
        与える。
    BY <1>1, <ref id=7af2e2e/>, <ref id=7855e90/>, <ref id=19296b2/>, <ref id=5116349/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, DEF 節点の量,
       <ref id=72bac67/>, CODE src/rc_ir/ownership.rs: References, CODE src/rc_ir/ast.rs: VarPath, FieldPath,
       EXT derive(Clone), EXT Vec::clone, EXT 標準の型の clone は等しい値を返す
    引かれる `ActRefs(n)` と、区間の最初に置かれる `ActRefs(t)` が、走査のどの段階で読んでも同じ値で
    あることは L46 が述べる。
  <2>3. 区間の最後で、`n*(ρ)` の `subtract` の後にこの `outstanding` は空になる。
    BY <1>6
  <2>4. QED
    <2>2 より、`ActRefs(t)` から `R_ρ(t)` の各要素の `ActRefs` を順に引いた結果が <2>3 で空になる。
    **各回の引き算は切り捨てを起こさない** -- `p30` の `L5` の 3 より、`un_bump` が `InBracket` を
    返して `subtract` を呼ぶのは、選んだ要素の `outstanding` がその `References` を `covers` する
    ときだけであり、D15 は `covers(R)` を「各オブジェクトについて自分の個数が `R` 以上か」と定める。
    すなわち各名前について、引かれる個数はその時点の個数以下である。よって多重集合の差として順に引いた
    結果が空になることから `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` が出る。
    BY <2>1, <2>2, <2>3, <ref id=19296b2/>, <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: References
<1>9. QED
  BY <1>3, <1>5, <1>6, <1>7, <1>8

### L39 (静的な収支は実行時の収支である) <!--#2307c1e-->

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`t ∈ CT` が `ρ` の上にあるとき、名前の
多重集合として

`ActRefs^inh_ρ(t) = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)`

である (`ActRefs^inh_ρ` は `p13` の `DEF 実行時の作用`)。したがって、`t` が `ρ` で実際に作る参照の
多重集合は、`R_ρ(t)` の各要素が `ρ` で実際に処分する参照の多重集合の和に等しい。

**証明**

<1>1. `Σ_{r ∈ R_ρ(t)} ActRefs(r) = ActRefs(t)` である。`R_ρ(t)` の各要素は `I_ρ(t)` に入るので `ρ` の上に
      あり、`ρ` の上で実行される。
  BY <ref id=5adaf7f/>, <ref id=72bac67/>
  この等式の両辺の `ActRefs` が走査のどの段階で読んでも同じ値であることは L46 が述べる。
<1>2. `ρ` の上の各 `Retain`/`Release` 節点 `m` と各名前 `o` について、`o` が `ρ` で活性 (`p13` の
      `DEF 名前の活性`) ならば `ActRefs(m)[o] = ActRefs^inh_ρ(m)[o]` であり、活性でなければ
      `ActRefs^inh_ρ(m)[o] = 0` である。
  BY <ref id=77a3def/>
<1>3. 活性な名前 `o` について、<1>1 の等式の両辺の `o` の個数は <1>2 より `ActRefs^inh_ρ` の側の個数と
      等しいので、`ActRefs^inh_ρ(t)[o] = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)[o]` である。活性でない名前に
      ついては <1>2 より両辺とも 0 である。よって言明の第 1 の等式が成り立つ。
  BY <1>1, <1>2
<1>4. QED
  BY <1>3, <ref id=126135d/>, p13 の DEF 名前をオブジェクトへ写す, p13 の DEF 実行時の作用
  **`ActRefs^inh_ρ` そのものは参照の多重集合ではない。** `p13` の `DEF 実行時の作用` は
  「**`ActRefs^inh_ρ(n)` は名前 (`VarPath`) ごとの多重集合であり、参照の多重集合ではない。** D8 より参照の
  多重集合はオブジェクトごとであるのに対し、この数え上げは `VarPath` ごとである。」と述べる。参照の
  多重集合になるのは、各名前をそれが指すオブジェクトへ写した後 -- `p13` の
  `DEF 名前をオブジェクトへ写す` の `(・)^obj` を掛けた後 -- である。
  `p13` の `L10c` の (ii) は「`n` が `Retain` のとき `(ActRefs^inh_ρ(n))^obj` は `n` が `ρ` で実際に作る
  参照の多重集合であり、`n` が `Release` のとき `n` が `ρ` で実際に処分する参照の多重集合である」と
  述べ、(i) がその写しが定まることを述べる。`(・)^obj` は名前ごとの個数をオブジェクトごとに束ねる写しな
  ので多重集合の和を保つ。よって <1>3 の名前ごとの等式に `(・)^obj` を掛けると、言明の第 2 の文 --
  参照の多重集合の等式 -- になる。

### L39a (解放の走査は義務集合を戻す) <!--#78073d2-->

**言明** --- `ρ` の上の節点 `q` の実行の中で、ある処分に付随して走る D24 の (F) の解放の連鎖
(`DEF 節点の実行の素動作`) を取る。`τ0` をその処分の直前の点、`c(O)` をその処分が `Obl` から取り除く
`O` への参照の個数とする。このとき、連鎖が終わった点 `τ1` について `Obl(τ1, O) = Obl(τ0, O) - c(O)` で
あり、`τ0` と `τ1` のあいだの各点 `τ` について `Obl(τ, O) ≥ Obl(τ0, O) - c(O)` である。すなわち**解放は
`Obl` を正味で動かさず、その内側でも下げない。さらに、連鎖の中で `Obl` から参照を取り除く各素動作に
ついて、取り除かれる参照はその直前の点の `Obl` に入っている。**

**支えは D24 の網羅である。** D24 は「**段の記述は `Obl` について網羅である。** (E1)-(E9) と (F) は、
各段について `Obl` を離れる参照の行き先と、作られる参照の持ち手と、`H` の動きを全部書いている。
**すなわち、ここに挙がっていない動きは起きない。**」と述べ、その節を要る段として
「`p40-cancel-soundness.md` の `L39a` のように、D21 の意味の活性化 -- 実行に実現するとは限らないもの --
の `Obl` を論じる段は、P28 を引けないのでこの網羅で迂回する」と、この命題を名指す。解放の連鎖の素動作に
ついて D24 が `Obl` に触れると述べるのは、(F) の retain と (E4) の 2 つの返り、および D9 の消費の行と
(E2) の行き先の一覧だけであり、以下はその数え上げである。
**主語は D21 の意味の 1 つの活性化であり、実行 (D24) が作る活性化に限らない。**

**証明**

<1>1. 連鎖で起きるのは D24 の (F) の解放である。1 つの解放が持つ動作は次の 6 つで尽きる。
  BY DEF 節点の実行の素動作, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=9d74736/>
  - **`o` が持つ参照の処分。** D24 の (F) は解放を「`o` が持つ参照 (D25) をすべて処分し、それから `o` の
    記憶域を返すこと」と定める。
  - **`_dtor` の欄の関数に適用の分の参照を与える retain。** `o` が `Std::FFI::Destructor` の
    オブジェクトであるときに在る。D24 の (F) は「**この段は参照も作る。** `_dtor` の欄の関数に適用の分の
    参照を与える retain がそれである」と述べる。
  - **2 つの活性化とその返り。** D24 の「活性化の林」は「**(F) の解放が作る活性化は
    2 つである** -- `_dtor` の欄の関数を `_value` の欄の値へ適用するものと、それが返した `IO` の動作の
    runner を適用するものであり、2 つ目の入力は 1 つ目の結果である」と述べる。1 つ目の適用は `_value` の
    欄の参照を消費し、2 つ目の適用は 1 つ目の返り値を消費する (D9 の `App` の行)。
  - **2 つ目の返り値を `o` の `_value` の欄へ書き戻す受け渡し。** D24 の (F) は「**この段は `_value` の
    欄の参照を動かす。**」に続けて「`b` が返した `IO` の動作の結果は `o` の `_value` の leaf へ戻る」と
    述べ、「したがって**「`o` が持つ参照をすべて処分し」が指すのは、この往復の後に `o` が持つ参照で
    ある**」と続ける。
  - **その 2 つの活性化の子孫の活性化の素動作。** D24 の「活性化の林」は活性化を親子で並べ、子の活性化が
    作る活性化もその林に入る。その素動作が動かすのはその活性化の `Obl` である (D24 の (E2)、(E3)、(E4))。
  - **`o` の記憶域の返却。** D24 の (F) の「それから `o` の記憶域を返すこと」である。**これは D24 の
    素動作の 6 種のどれでもない** -- 参照を作らず、渡さず、割り当てず、処分せず、解放を起こさず、
    グローバル化もしないので、`Obl` も `H` も動かさない。
<1>2. `o` が持つ参照の処分は `Obl` を変えない。
  BY <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=f06144e/>
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
  BY <ref id=e3436e8/>
<1>4. この解放が `Obl` に入れる参照は 3 種であり、どれも同じ連鎖の中で、それを入れた素動作より後の
      素動作で `Obl` を離れる。
  <2>1. `_dtor` の欄の関数に retain が与える参照。D24 の (F) より、その持ち手はこの解放を含む段を
        実行している活性化であり、その retain は `_dtor` の欄の関数に**適用の分**の参照を与えるもので
        ある。その適用は D9 の `App` の行により callee の全 boxed leaf の参照を消費するので、この参照は
        同じ連鎖の中で `Obl` を離れ、1 つ目の活性化の `Obl` の初期値に入る。retain が適用より前に立つ
        ことは D24 の (F) が「**この段は参照も作る。** `_dtor` の欄の関数に適用の分の参照を与える retain が
        それである」と述べ、コードの位置を `build_retain(dtor, one, ...)` が `apply_lambda` の前に立つ
        こととして書いていることから出る。
    BY <ref id=e3436e8/>, <ref id=9d74736/>
  <2>2. 1 つ目の活性化が返す参照。D24 の (E4) は「(F) が作る 1 つ目の活性化は `_dtor` の欄の関数を
        `_value` に適用したものであり、その返り値 `io_act` は 2 つ目の活性化 -- 返った `IO` の runner の
        適用 -- の入力になる」と述べ、「どちらの返りでも、参照はその解放を含む段を実行した活性化の
        `Obl` に入る」と続ける。よってこの参照は `Obl` に入る。**それを離すのは 2 つ目の適用である。**
        D24 の「活性化の林」は 2 つ目の活性化を「それが返した `IO` の動作の runner を適用するもの」と
        定め、その生成コードとして `run_ios_runner` を名指す。`run_ios_runner` はその runner を
        `apply_lambda` の**呼び出し先**に据え、`run_io_or_ios_runner` はその runner に `io_act` 自身か、
        `io_act` の第 0 フィールドから取り出した値を渡す
        (`CODE src/fixstd/builtin.rs: run_io_or_ios_runner`, `run_io`, `run_ios_runner`,
        `CODE src/generator.rs: Generator::build_run_destructor` -- `apply_lambda` の結果 `io_act` を
        `run_io_or_ios_runner` に渡す)。D9 の `App` の行は callee の全 boxed leaf を消費し、フィールドの
        取り出しは D9 の `Destructure` の 2 行のどちらか -- unbox の名前付きフィールドの移動か、boxed
        容器の消費 -- なので、いずれにせよ `Obl` に入ったこの参照は同じ連鎖の中で `Obl` を離れる。返りが
        2 つ目の適用より前にあることは、(E4) の「その返り値 `io_act` は 2 つ目の活性化 -- 返った `IO` の
        runner の適用 -- の入力になる」から出る。
    BY <ref id=e3436e8/>, <ref id=9d74736/>, <ref id=7e70ffa/>, <ref id=9e3e401/>,
       CODE src/generator.rs: Generator::build_run_destructor,
       CODE src/fixstd/builtin.rs: run_io_or_ios_runner, run_io, run_ios_runner
  <2>2a. 2 つ目の活性化が返す参照。D24 の (E4) より、これも `Obl` に入る。**`o` の `_value` の欄へ
        書き込まれるのはこの 2 つ目の返り値だけである** -- (E4) は「2 つ目の返り値は `o` の `_value` の
        欄へ書き込まれる」と述べる。**この書き戻しは (F) の解放の中の動作である** -- D24 の (F) は
        「**この段は `_value` の欄の参照を動かす。**」に続けて「`b` が返した `IO` の動作の結果は `o` の
        `_value` の leaf へ戻る」と述べ、「したがって**「`o` が持つ参照をすべて処分し」が指すのは、この
        往復の後に `o` が持つ参照である**」と続ける。よってこの参照は書き戻しによって `Obl` を離れ、その
        持ち手は `o` になる (D25 の 2 つ目)。**その参照は同じ解放が処分する** -- <1>2 より `o` が持つ
        参照の処分は `Obl` を変えない。
    BY <ref id=e3436e8/>, <ref id=0b850c9/>, <1>2
  <2>3. QED
    BY <2>1, <2>2, <2>2a, <1>1, <ref id=e3436e8/>
    D24 の (F) がこの解放について参照を作ると述べるのは <2>1 の retain だけであり、解放の中で `Obl` へ
    参照が入るもう 1 つの道は (E4) の 2 つの返りだけである。<1>1 の 6 つの動作のうち、残る 3 つは
    `Obl` に参照を入れない -- `o` が持つ参照の処分は <1>2 が `Obl` を変えないと述べ、作られた 2 つの
    活性化とその子孫の素動作が動かすのはその活性化の `Obl` であり (D24 の (E2)、(E3)、(E4))、記憶域の
    返却は素動作ではない (<1>1)。
<1>4b. 連鎖の中で `Obl` から参照を取り除く各素動作について、取り除かれる参照はその直前の点の `Obl` に
       入っている。
  BY <1>1, <1>2, <1>3, <1>4, <ref id=e3436e8/>, <ref id=9d74736/>, <ref id=f06144e/>
  <1>1 の 6 つの動作のうち `Obl` から参照を取り除きうるのは、2 つの適用が D9 の `App` の行により行う
  消費と、2 つ目の返り値を `o` の `_value` の欄へ書き戻す受け渡しである。**残る 4 つは `Obl` から
  取り除かない** -- `o` が持つ参照の処分は `Obl` を変えず (<1>2)、`_value` の欄の参照は `Obl` を
  通らず (<1>3)、`_dtor` の retain は `Obl` へ入れる側であり (<1>4)、`o` の記憶域の返却は素動作では
  ない (<1>1)。2 つの活性化とその子孫が実行する段が動かすのはその活性化の `Obl` である
  (D24 の (E2)・(E3)・(E4))。<1>4 は、この連鎖が `Obl` に入れる 3 種の参照のどれもが、それを入れた
  素動作より**後**の素動作で `Obl` を離れることを述べるので、取り除く素動作の直前の点でその参照は
  `Obl` に在る。
<1>4a. 連鎖のあいだに環境の段は入らない。
  BY <ref id=c9e4cca/>, <ref id=e3436e8/>
  連鎖は 1 つの段の中で走る (D24 の (F) -- 「`o` は**その同じ段の中で解放される**」)。A17 (iii) は
  「環境の動作も D24 の段としてこの実行の 1 つの列に並ぶ。段は不可分なので、環境が動くのは段と段の
  あいだである。」と述べるので、環境の参照の操作の段 (E9) は連鎖の内側の点では起きない。**この段が
  要るのは、(E9) が `H` を動かす段だからである** -- D24 は (E9) について「`H` はその分だけ上がり、
  `Obl` は動かない。」「`H` はその分だけ下がり、`Obl` は動かない。」と述べるので `Obl` の勘定は
  変わらないが、連鎖の内側で `H` が動くと <1>1 の「連鎖で起きるのは D24 の (F) の解放である」が
  読めなくなる。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>4a, <1>4b, <ref id=e3436e8/>
  D24 の (F) より解放の連鎖は有限で終わる。その各解放について <1>2 から <1>4 が、`Obl` に入った参照が
  すべて同じ連鎖の中で `Obl` を離れ、それ以外に `Obl` を動かす素動作が無いことを述べる。よって連鎖が
  終わった点の `Obl` は、処分が取り除いた分だけ `τ0` より少ない。連鎖の内側の各点については、<1>4 の
  3 種のどれも、それを `Obl` に入れる素動作が、それを `Obl` から離す素動作より前にあるので、`Obl` は
  その値を下回らない。

### L39b (子の活性化の事象は義務集合を動かさない) <!--#9d3dd4d-->

**言明** --- `ρ` の上の節点 `q` の実行の素動作のうち、子の活性化の素動作とグローバル化
(`DEF 節点の実行の素動作`) は、`Obl(α)` を動かさない。

**証明**

<1>1. 子の活性化の素動作は、D24 の (E3)、(E7)、および (E2) のうちオペランドを適用する `Llvm` の段が作る
      活性化と、その子孫の活性化のものである。子の活性化自身の素動作が動かすのは、その子の `Obl` である
      (D24 の (E2)、(E3)、(E4))。
  BY DEF 節点の実行の素動作, <ref id=e3436e8/>
<1>2. (E3) の `App` について、`α` の `Obl` を動かすのは D9 の `App` の行の消費と (E4) の受け取りであり、
      どちらも `α` 自身の節点について D10 の行が定めるものなので、`DEF 節点の実行の素動作` の第 1 の
      箇条に在る。子の活性化の素動作には無い。
  BY <ref id=e3436e8/>, <ref id=9d74736/>, <ref id=f06144e/>, DEF 節点の実行の素動作
<1>3. (E2) のうちオペランドを適用する `Llvm` の段についても同じである。渡る参照は D9 の `Llvm` の行が、
      受け取る参照は D10 の生成の `Llvm` の行が定める -- D24 の (E4) は「`b` を作ったのが (E2) のうち
      オペランドを適用する `Llvm` の段であれば、それらの参照はその段を実行した活性化の `Obl` に入り」と
      述べる。どちらも `α` 自身の節点について D10 の行が定めるものである。
  BY <ref id=e3436e8/>, <ref id=9d74736/>, <ref id=f06144e/>, DEF 節点の実行の素動作
<1>4. (E7) のグローバルの初期化は `α` の `Obl` を動かさない。D24 の (E7) より、初期化子の活性化の終端の
      `Ret` が消費する参照の行き先は呼び出し元ではなく `E` である。グローバルを読む節点が参照を得ない
      ことは A8 と D26 が言う (D24 の (E7))。同じ段の中で走る (E5) のグローバル化も、`Obl` を離れる参照も
      作られる参照も持たない -- D24 の (E5) は印を付けることだけを述べる。
  BY <ref id=e3436e8/>, <ref id=b6673ca/>, <ref id=88a06de/>
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, DEF 節点の実行の素動作
  `DEF 節点の実行の素動作` より、子の活性化の素動作はこの 3 種の段が作る活性化とその子孫のものに尽き、
  グローバル化は (E7) が作る活性化が終わるところにだけ現れる。

### L40 (歩みの位置は boxed leaf である) <!--#6fffd4c-->

**言明** --- `ty(u)` を変数 `u` に束縛された値の型 (D6) とする。次の 2 つが成り立つ。

1. `σ` が `boxed_leaf_paths(ty(u))` の要素であり、`u` の束縛が `Binding::Llvm` であるとき、
   `origin_inner(u, σ)` が呼ぶ `origin_from_leaves_under` は `origin` を呼ばない。
2. `σ` が `boxed_leaf_paths(ty(u))` の要素であるとき、`origin_inner(u, σ)` が呼ぶ `origin(u', σ')` の
   `σ'` は `boxed_leaf_paths(ty(u'))` の要素である。

したがって、スロット (D6) から始まる別名類の ρ-歩み (D33) の各位置 `(u, σ)`
について、`σ` は `ty(u)` の boxed leaf である。**とくに、`origin_from_leaves_under` が
`truncate_to_unit(ty(args[j]), σ')` を行き先の path として辿る辺 -- 行き先の path が leaf でないことが
ありうる唯一の辺であり、D17 の**辺の行き先についての第 3 項**の但し書きがそれを述べる -- は、歩みの
上では取られない。**

**D17 との対応**。D17 は `origin` が辿る各辺を、`origin` に与えた path `π` の下の leaf `λ` の写り方として
述べる。`π` 自身が boxed leaf であるときは `λ = π` であり、そのとき辺の行き先の path は D17 が言う `λ` の
像そのものである。2 の証明の各場合でこれを使う。

**証明**

<1>1. `origin(u, σ)` の評価が呼ぶ `origin` は、`origin_inner(u, σ)` が呼ぶものだけである。`origin` は
      `vars.origins` の memo を引き、無ければ `grow_stack(|| origin_inner(vars, type_env, var, path))` を
      呼んでその値を記録するほかに何もしない。A15 より `grow_stack` は閉包をちょうど 1 回呼ぶ。
  BY CODE src/rc_ir/ownership.rs: origin, <ref id=3e6b0e0/>
<1>2. 1 が成り立つ。
  <2>1. `origin_inner` の `Binding::Llvm(llvm_gen, args, result_ty)` の腕が `origin_from_leaves_under` を
        呼ぶのは、`decl.leaf_origins_at(σ).and_then(as_arg_projection)` が `None` のときである。ここで
        `decl = llvm_gen.result_prov(result_ty, &arg_tys, type_env)` であり、`collect_bindings` が
        `Binding::Llvm` の第 3 欄に置くのは `Let(x, RcRhs::Llvm(..), k)` の `x.ty` なので
        `result_ty = ty(u)` である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings, <ref id=596a46d/>
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
          前提 `result_prov` の本体の在りか の走査が挙げる各項目は、`Provenance::uniform`・
          `Provenance::uniform_bottom`・`Provenance::fresh_under`・`Provenance::build_shape`・
          `replaced_field_prov` のいずれかを第 1 引数 `result_ty` に掛けた値を返す。`replaced_field_prov`
          はその中で `Provenance::uniform(result_ty, ・, ・)` か
          `Provenance::build_shape(result_ty, ・, ・)` を返す
          (`CODE src/fixstd/builtin.rs: replaced_field_prov`)。**数え上げの範囲を閉じるのは
          EXT trait の実装は既定と再定義で尽きる である** -- ある op の `result_prov` が実行する本体は、
          その型の `impl LLVMGen for` がそれを再定義していればその定義、していなければ trait 側の既定の
          本体であり、ほかに本体は無い。走査の literal `fn result_prov` はその両方を挙げるので、
          `result_prov` が実行しうる本体はこの一覧で尽きる。
      BY CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, CODE src/fixstd/builtin.rs: replaced_field_prov,
         前提 `result_prov` の本体の在りか, EXT trait の実装は既定と再定義で尽きる
    <3>3. QED
      BY <3>1, <3>2, <2>1
      `<2>1` より `result_ty = ty(u)` である。
  <2>2a. `leaf_origins_at(σ)` は `Some` を返す。`leaf_origins_at` は `decl` の写像を鍵 `σ` で引くだけで
         あり (`LeafMap::get`)、本命題の仮定より `σ` は `boxed_leaf_paths(ty(u))` の元なので、`<2>2` より
         それは `decl` の鍵である。
    BY <2>1, <2>2, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>3. `σ` の宣言は、空集合か、`Fresh` ただ 1 つか、`Unknown` ただ 1 つである。A3 より、このコミットの
        すべての op の宣言は結果の各 leaf に元数 0 か 1 の `LeafOrigins` を与える。元数 1 でその元が
        `LeafOrigin::Arg` ならば `as_arg_projection` は `Some` を返し、<2>1 の場合に入らない。
    BY <2>1, <2>2a, <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>4. `decl.leaf_origins_under(σ)` が渡す集合は、`σ` 自身の宣言 1 つだけである。`leaves_under(path)` は
        鍵が `path` を前置に持つ元を渡し、`p13` の `L7` より boxed leaf の路は反鎖をなすので、`σ` を
        前置に持つ boxed leaf は `σ` 自身だけである。
    BY <2>2, <ref id=efe0c77/>, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
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
        **各辺での `λ` の写り方の第 1 項**は「`Binding::Move`、catch-all アームの payload、
        `Binding::Join`: `λ` を変えない」と述べ、
        D17 はその像を「着く leaf」と呼ぶので、`σ` は行き先の変数の型の boxed leaf である。
    BY CODE src/rc_ir/ownership.rs: origin_inner, <ref id=d59f90b/>, <ref id=83d98e9/>
  <2>3. CASE `u` の束縛が `Binding::Field(container, idx)` で `container.ty.is_box` が偽、または
        `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box` が偽である。これらの腕が呼ぶのは
        `origin(container, [idx] ++ σ)`、`origin(scrut, [tag] ++ σ)` である。**呼ばれる側の変数を `w`、
        足される添字を `i` と書く** -- `Binding::Field` では `w = container`、`i = idx` であり、
        `Binding::Payload` では `w = scrut`、`i = tag` である。`u` は `w` から取り出された値の変数で
        あって、この場合の 2 つの型は `ty(w)` (容器・scrutinee の型) と `ty(u)` (フィールド・payload
        変数の型) である。
    <3>1. `ty(w)` について、D4 の第 5 規則が当たる。すなわち `ty(w)` は `is_fully_unboxed` でも
          `is_closure` でも `is_box` でも `is_array` でもない。
      BY <ref id=83d98e9/>, <ref id=8412761/>, <ref id=3d4be43/>, <ref id=0594f24/>, 本場合の仮定, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
         CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_struct,
         CODE src/ast/types.rs: TypeNode::is_union, CODE src/ast/types.rs: TypeNode::is_array,
         CODE src/ast/types.rs: TypeNode::is_funptr,
         CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
         CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
         CODE src/fixstd/builtin.rs: is_array_tycon, is_funptr_tycon, make_array_tycon, make_funptr_tycon,
         CODE src/fixstd/builtin.rs: bulitin_tycons
      `ty(w).is_box` が偽であることは本場合の仮定である。A12 は「`Match` の scrutinee が union であること、
      `Destructure` の容器が構造体であること」を述べ、README の A12 は「**この仮定が型の `variant` を
      述べる各節では、その型の `is_closure()` は偽である。**」と続けるので `ty(w).is_closure` は偽で
      ある。
      `is_struct` と `is_union` の本体は `toplevel_tycon_info(type_env).variant` を `Struct` か `Union` と
      照合するので、A12 より `ty(w)` の最上位の tycon の項目の `variant` は `Struct` か `Union` である。
      **`is_array` と `is_funptr` の本体は tycon そのものを見る** -- `is_array` の本体は
      `self.toplevel_tycon_satisfies(is_array_tycon)`、`is_funptr` の本体は
      `self.toplevel_tycon_satisfies(|tc| is_funptr_tycon(tc).is_some())` であり、
      `toplevel_tycon_satisfies` は最上位の tycon にその述語を当て、`is_array_tycon(tc)` は
      `*tc == make_array_tycon()`、`is_funptr_tycon(tc)` は `tc.name.namespace` が `Std` の 1 段で
      `tc.name.name` が `FUNPTR_NAME` で始まり残りが数として読めることである。**その 2 つが真ならば
      `variant` は `Struct` でも `Union` でもない** -- A28 は「`E.tycons()` の項目のうち、鍵が
      `bulitin_tycons()` の置く鍵のいずれかであるものは、`bulitin_tycons()` がその鍵の下に置いた項目で
      ある。**とくに `make_array_tycon()` の項目と、`tc.name.namespace` が `Std` の 1 段であって
      `tc.name.name` が `FUNPTR_NAME` (`"#FunPtr"`) で始まる鍵の項目 (`make_funptr_tycon(n)`、`n` は
      1 以上 `FUNPTR_ARGS_MAX` 以下) がそうである。**」と述べ、`bulitin_tycons()` が
      `make_array_tycon()` に与える `TyConInfo` の `variant` は `Array`、`make_funptr_tycon(arity)` に
      与えるものは `Primitive` である。よって `ty(w)` については
      `is_array` も `is_funptr` も偽である。`is_fully_unboxed` は、`is_box`・`is_closure`・`is_array` のいずれでもなく
      `is_funptr` でもない型について、`unpunched_field_types` の各フィールドの型が `is_fully_unboxed` で
      あることと同値である。A10 より、`ty(w)` の `unpunched_field_types` の歩みは abort せず有限である。
      A12 より `Destructure` が名指すフィールドと `Match` が名指す変位は punched でなく、その型は `ty(u)` に
      等しいので、`unpunched_field_types(ty(w))` は `ty(u)` を含む。本命題の仮定より
      `σ ∈ boxed_leaf_paths(ty(u))` であり、`boxed_leaf_paths` は `is_fully_unboxed` の型について空の列を
      返すので、`ty(u)` は `is_fully_unboxed` ではない。よって `ty(w)` も `is_fully_unboxed` ではない。
    <3>2. QED
      BY <3>1, <ref id=0594f24/>, <ref id=d59f90b/>, <ref id=83d98e9/>, CODE src/rc_ir/ownership.rs: origin_inner
      D4 の第 5 規則は「それ以外 (unbox の構造体・タプル・union) は、`unpunched_field_types` が返す
      フィールドの下へ降りる。union のときは各変位の payload へ降りる」と述べるので、
      `boxed_leaf_paths(ty(w))` は `[i] ++ boxed_leaf_paths(ty(u))` を含む。よって `[i] ++ σ` は
      `ty(w)` の boxed leaf である。D17 の**各辺での `λ` の写り方の第 2 項**が同じことを「unbox 容器の
      `Destructure` のフィールド、
      unbox union の変位アームの payload: `λ` の先頭に添字を足す」と述べる。
  <2>4. CASE `u` の束縛が `Binding::Field(container, idx)` で `container.ty.is_box` が真、または
        `Binding::Payload(scrut, Some(tag))` で `scrut.ty.is_box` が真である。この 2 つの腕は `here()` を
        返し、`origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>5. CASE `u` の束縛が `Binding::Llvm` で、`decl.leaf_origins_at(σ).and_then(as_arg_projection)` が
        `Some((j, p))` である。この腕は `origin(args[j], p)` を呼ぶ。`as_arg_projection` が `Some((j, p))`
        を返すのは `σ` の宣言が単一の `LeafOrigin::Arg(j, p)` であるときであり、A3 の表の「単一の
        `Arg(j, σ)`」の行 (その `σ` はここでの `p` に当たる) は、それが「第 `j` オペランドの leaf `σ`」を
        名指すと述べる。よって `p` は `boxed_leaf_paths(ty(args[j]))` の要素である。D17 の**各辺での `λ` の
        写り方の第 3 項**も同じ
        ことを「`λ` を、`λ` 自身の宣言 `Arg(j, σ')` の `σ'` へ置き換える」と述べる。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection, <ref id=e11772a/>, <ref id=d59f90b/>
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
  BY <1>1, <1>2, <1>3, <ref id=596a46d/>, <ref id=30d6238/>
  D6 よりスロット `(x, λ)` の `λ` は `ty(x)` の inhabited な boxed leaf である。歩みはスロットから始まり、
  各段で `origin_inner` が呼ぶ `origin` の引数へ進むので (D33)、<1>1 と <1>3 を
  歩みの長さについて繰り返せば、各位置の path は boxed leaf である。<1>2 より、boxed leaf の位置では
  `origin_from_leaves_under` は `origin` を呼ばないので、その辺は歩みの上では取られない。

### L40a (スロットを含む類は開始している) <!--#2ea7903-->

**言明** --- 実行路 `ρ` を辿る活性化の点 `τ` (`DEF 実行時の量`) と**計数下** (D26) の別名類 `C` に
ついて、`C` が、`τ` までに値を得た変数のスロット (D6) を 1 つでも含むならば、`C` の開始の点 -- D34 の
3 つの箇条が `C` の開始値を置く点 -- は `τ` 以前である。とくに次の 3 つが成り立つ。

1. `ρ` の上の節点 `n` とその入口の点 `τ_n` について、`bumps_ρ(n, C) ≥ 1` である計数下の類 `C` は
   `τ_n` で開始している。よって
   `Σ_{C : obj(C) = O} bumps_ρ(n, C)` は `b(n, O)` に等しい (DEF 類ごとの義務)。
2. D34 の表の各事象について、その事象が名指す leaf のスロットが属する類は、それが計数下であるとき、
   **その事象を運ぶ素動作の直後の点で**開始している。
3. D10 の生成の表の 5 行が名指す leaf のスロットは、それが属する別名類の ρ-終端 (D33) である。

**計数下の類に限るのは、D34 がそこにしか開始の時点を定めないからである。** D34 は「**計数下の**別名類
`C` について、`ρ` を辿る活性化の各時点 `τ` における `held_ρ(τ, C)` を次で定める。グローバル状態の類には
定めない。」と述べ、「開始の時点」の節はその表の最初の 3 行が置く開始値の置き場所を定めるものである。

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
    BY <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=ff5985d/>, <2>1
  <2>3. `Binding::Join` では、D17 より辺はその活性化が選んだアームの結果へ辿る。D3 より `ρ` は
        `Match` 節点でそのアーム本体を辿ってから継続へ進むので、アームの結果の変数は `Match` の
        束縛変数より前に値を得ている。
    BY <ref id=d59f90b/>, <ref id=ca36627/>, <ref id=c232680/>, <2>1
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>2. 計数下の類 `C` が `τ` までに値を得た変数のスロット `(u, σ)` を含むならば、`C` の ρ-終端
      `T_ρ(C)` の変数も `τ` までに値を得ている。よって D34 の「開始の時点」より `C` は `τ` で
      開始している -- D34 が開始の時点を定めるのは計数下の類についてであり、`C` はそれである。
  BY <1>1, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=3054e88/>, <ref id=ff5985d/>
  D34 は「最初の 3 行が置く開始値は、`T_ρ(C) = (u, σ)` の変数 `u` が値を得る時点で置かれる --
  `u` がパラメータ・capture なら活性化が始まる時点、そうでなければ `u` を束縛する節点を実行する段の
  直後である。」と述べる。`T_ρ(C)` の変数が `τ` までに値を得ているならば、その時点は `τ` 以前で
  ある -- パラメータ・capture なら活性化が始まる時点であり (D23)、そうでなければその変数を束縛する
  節点を実行する段は `τ` までに終わっているからである。
  D33 より `C` のスロットはどれも `ρ` を辿って同じ終端に着く。ρ-終端は `(u, σ)` から始まる ρ-歩みの
  最後の位置であり、歩みの各段は `origin_inner` が呼ぶ `origin` の引数へ進む。<1>1 をその段数について
  繰り返す。
<1>2a. スロット `(v, λ)` (D6) について、`id(v, λ)` は `(v, λ)` から始まる ρ-歩み
      (D33) の上の位置である。とくに <1>1 より、その位置の変数は `v` より
      後に値を得ることがない。
  <2>0. ρ-歩みの残りの長さについての帰納法で示す。L40 より歩みの各位置 `(u, σ)` の `σ` は `ty(u)` の
        boxed leaf であり、歩みは `origin_inner` が `origin` を呼ばない位置で終わる。P2 より
        `origin(v, λ)` は停止するので、その再帰の深さは有限であり、歩みも有限である。よってこの
        帰納法は整礎である。
    BY <ref id=6fffd4c/>, <ref id=0edb0ba/>, <ref id=30d6238/>
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
      BY <ref id=6fffd4c/>, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under
    <3>3. QED
      BY <3>1, <3>2, CODE src/rc_ir/ownership.rs: Origin::identity
  <2>2. CASE 位置 `(u, σ)` の腕が、次の位置の `origin` の値をそのまま返す。すなわち `Binding::Move(y)`、
        `Binding::Payload(scrut, None)`、`container.ty.is_box` が偽の `Binding::Field`、
        `scrut.ty.is_box` が偽の `Binding::Payload(_, Some(tag))`、`as_arg_projection` が `Some((j, p))`
        を返す `Binding::Llvm` である。`identity` は次の位置のものであり、その位置は ρ-歩みの次の位置で
        あるから、帰納法の仮定が当たる。
    BY CODE src/rc_ir/ownership.rs: origin_inner, <ref id=30d6238/>, 帰納法の仮定
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
         <ref id=cbc4a1c/>, <ref id=d59f90b/>, <ref id=c232680/>, <ref id=30d6238/>, 帰納法の仮定
    <3>2a. `candidates` は空でない。
      BY CODE src/rc_ir/ownership.rs: Origin::of_candidates, <ref id=0edb0ba/>
      `of_candidates` は `assert!(!candidates.is_empty(), ...)` で始まる。P2 は `origin(x, π)` が
      「`π` を問わず panic せずに答えを返し、停止する」と述べるので、この表明は破れない。
      **この段が要るのは、`candidates` が空の場合が `of_candidates` の `match` の
      `_ => Origin::Join { .. }` の腕に落ちるからである** -- 表明が無ければ元数 0 の `Join` が
      返りうる。
    <3>3. QED
      BY <3>1, <3>2, <3>2a
      <3>2a より `candidates` の元数は 1 以上であり、1 のときを <3>2 が、2 以上のときを <3>1 が扱う。
  <2>4. QED
    `origin_inner` の `match` は `vars.bindings.get(var)` の値について `None | Param | Producer`、
    `Move`、`Join`、`Llvm`、`Field`、`Payload` の 6 つの腕を持ち、`Field` と `Payload` は `is_box` で、
    `Llvm` は `as_arg_projection` の値でさらに分かれる。<2>1、<2>2、<2>3 がこれを尽くす。歩みの最初の
    位置は `(v, λ)` なので、`id(v, λ)` はその歩みの上の位置である。「とくに」は <1>1 を歩みの段数に
    ついて繰り返したものである。
    BY <2>1, <2>2, <2>3, <1>1, CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: Binding
<1>3. 1 が成り立つ。
  <2>1. `bumps_ρ(n, C) ≥ 1` とする。DEF 類ごとの義務 より、`pending(n)` のある要素 `p` と、
        `C` に属するある名前 `o` について `B_ρ(n, p)[o] ≥ 1` である。
    BY DEF 類ごとの義務, DEF 節点の実行の素動作, <ref id=8093b68/>
  <2>2. `p` の由来 (DEF 訪問) を `t = Retain(v, π)` とすると、`o = id(v, λ)` である `λ ∈ L(v, π)` が
        在る。その `λ` は `π` の下の inhabited (D16) かつ計数下 (D26) の boxed leaf である。
    BY <ref id=8093b68/>, <ref id=a423f41/>, DEF 本体ごとの記法, <ref id=0594f24/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=b154692/>
    D27 は、`p` が `pending` に入るときの `B_ρ` を、`π` の下の inhabited かつ計数下の各 leaf を
    `origin` の identity で名付けて数えたものと定め、以後の操作はその多重集合から引くか、値をそのまま
    運ぶだけである。よって `B_ρ(n, p)` が 1 以上を与える名前はその数え上げに現れる名前であり、`L(v, π)`
    の元の `id` である。P16 の (a) より `p` の由来は `Retain` 節点である。
  <2>3. `t` は `ρ` の上で `n` より真に前にある。
    BY <ref id=5b098d9/>, DEF 訪問
    L35 は、走査が訪問する節点 `n` と `pending(n)` の要素 `p` について、`n` を含むすべての実行路が
    `p` の由来を含み、その路の上で由来が `n` より真に前にあることを述べる。
  <2>4. `v` は `τ_n` までに値を得ている。
    BY <2>3, <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=ff5985d/>
    `t` は `v` を名指す節点である。A11 より `v` の使用はその位置でスコープに入っている束縛に解決し、
    D2 のスコープの規則よりその束縛は `t` の祖先であるか、パラメータ・capture である。D3 より実行路は
    祖先を先に通り、パラメータ・capture は活性化が始まる時点で値を持つ (D23)。<2>3 より `t` は `ρ` の
    上で `n` より前にあるので、その実行は `τ_n` より前に終わっている。
  <2>5. `o = (u, σ)` は `τ_n` におけるスロット (D6) である。
    BY <1>1, <1>2a, <2>1, <2>2, <2>4, <ref id=c4ea962/>, <ref id=b154692/>, <ref id=596a46d/>
    `p13` の `L11` より `B_ρ(n, p)[o] ≥ 1` である `o` は `ρ` で活性であり、`L45` より活性な
    名前 `o = (u, σ)` は `ρ` の上のスロットである -- すなわち `σ` は `ty(u)` の inhabited な boxed leaf
    である。残るのは `u` が `τ_n` までに値を得ていることである。<2>2 より `o = id(v, λ)` であり、`λ` は
    `π` の下の inhabited な leaf なので `(v, λ)` はスロットである。<1>2a より `o` は `(v, λ)` から
    始まる ρ-歩みの上の位置であり、<1>1 よりその変数は `v` より後に値を得ることがない。<2>4 より `v` は
    `τ_n` までに値を得ている。
  <2>6. QED
    BY <1>2, <2>1, <2>5, <ref id=948f840/>, DEF 類ごとの義務
    <2>1 より `o ∈ C` であり、<2>5 より `o` は `τ_n` におけるスロットなので、<1>2 より `C` は `τ_n` で
    開始している。P18b より `B_ρ` の個数は 0 以上なので `bumps_ρ(n, ・)` も 0 以上であり、
    `Σ_{C : obj(C) = O} bumps_ρ(n, C)` の 0 でない項はすべて `S(τ_n, O)` の類のものである。よってその和は
    `b(n, O)` に等しい。
<1>3a. 3 が成り立つ。
  <2>1. D10 の生成の 5 行が名指す値の束縛は順に、`Binding::Llvm` (宣言が単一の `Arg` でない leaf)、
        `RcRhs::App` の `Binding::Producer`、`RcRhs::Closure` の `Binding::Producer`、boxed 容器の
        `Binding::Field`、boxed scrutinee の `Binding::Payload(_, Some(tag))` である。
    BY <ref id=f06144e/>, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. `Binding::Producer` の腕、`container.ty.is_box` が真の `Binding::Field` の腕、
        `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` の腕は、いずれも `here()` を返し
        `origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. `Binding::Llvm` の腕は、boxed leaf `λ` について
        `decl.leaf_origins_at(λ).and_then(as_arg_projection)` が `None` のとき
        `origin_from_leaves_under(vars, type_env, &decl, args, λ, &here_identity)` を呼ぶ。この呼び
        出しは `origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner, <ref id=6fffd4c/>, <ref id=f06144e/>
    D10 の生成の `Llvm` の行が名指すのは結果の leaf なので、`λ` は結果を束縛する変数の型の boxed leaf
    である。L40 の 1 がこの呼び出しについて言明のとおりを述べる。
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=30d6238/>
    D33 は「**歩みは有限である** -- 各段は `origin_inner` が `origin` を呼ぶ腕に 1 対 1 で
    対応し、P2 よりその再帰は停止する。」と述べるので、歩みが止まる位置は `origin_inner` が `origin` を
    呼ばない位置である。<2>2 と <2>3 よりその leaf の位置で歩みは止まるので、その位置が ρ-終端である。
<1>4. 2 が成り立つ。
  <2>1. CASE 事象が D34 の第 4 行 (`Retain`)、第 5 行 (`Release`)、第 6 行 (D9 の消費) のものである。
        その事象が名指す leaf を持つ値の変数を `v` とすると、`v` はその事象を起こす節点が名指す変数で
        ある (D10 の `Retain`/`Release` の行、D9 の消費の表)。`v` はその節点の入口の点までに値を得て
        いる -- A11 より `v` の使用はその位置でスコープに入っている束縛に解決し、D2 のスコープの規則
        よりその束縛はその節点の祖先であるか、パラメータ・capture である。D3 より実行路は祖先を先に
        通り、パラメータ・capture は活性化が始まる時点で値を持つ (D23)。D10 の各行が名指すのは
        inhabited な leaf なので、その対はスロットである (D6)。よって `<1>2` より `C` はその節点の
        入口の点で開始しており、その事象を運ぶ素動作の直後の点はそれ以後である。
    BY <1>2, <ref id=f06144e/>, <ref id=9d74736/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=ff5985d/>, <ref id=596a46d/>, <ref id=9d5d254/>, <ref id=88a06de/>
  <2>2. CASE 事象が D34 の第 2 行・第 3 行 (パラメータ・capture の leaf の初期値) のものである。
        その leaf を持つ値の変数はパラメータか capture であり、D23 よりその値は活性化が始まる時点で
        在るので、その対はスロットである (D6)。D34 は第 2 行の開始値を「**その類の終端の参照が
        `Obl(a)` に入る素動作の直後の段内の点**」に、第 3 行の開始値を「**その活性化が生きている
        活性化 (D23) になる点**」に置く。第 2 行の事象はまさにその受け渡しの素動作であり、第 3 行の
        事象は活性化が生きている活性化になる点なので、どちらでも開始の点はその事象の点である。
    BY <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=ff5985d/>, <ref id=596a46d/>, <ref id=88a06de/>
  <2>3. CASE 事象が D34 の第 1 行 (D10 の生成) のものである。3 (<1>3a) より、その事象が名指す leaf の
        スロットは `C` の ρ-終端である。D34 は第 1 行の開始値を「**その類の終端の参照が `Obl(a)` に
        入る素動作の直後の段内の点**」に置き、D10 の生成の行はその leaf につき参照を 1 つ `Obl` に
        加えるので、その素動作がまさにこの事象である。よって開始の点はこの事象を運ぶ素動作の直後の点で
        ある。
    BY <1>3a, <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=596a46d/>, <ref id=88a06de/>
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=9d5d254/>
    D34 の表は 6 行を持ち、開始値を置くのは最初の 3 行である。第 1 行を <2>3、第 2 行と第 3 行を
    <2>2、第 4 行・第 5 行・第 6 行を <2>1 が尽くす。
<1>5. QED
  BY <1>2, <1>3, <1>3a, <1>4

### L49 (義務集合は類ごとの義務の総和を覆う) <!--#561fd05-->

**言明** --- 実行路 `ρ` を辿る 1 回の活性化 `α` を固定する。次の 2 つが成り立つ。

1. 計数下のオブジェクト `O` と `α` の各点 `τ` について `Obl(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C)` で
   ある。
2. `α` が実行する段でも、`α` が作った活性化とその子孫が実行する段でもない段 -- 環境の段と、別の制御の
   流れの活性化とその子孫の段 -- の素動作は、`Obl(α)` も `held_ρ` も動かさない。

**等号は主張しない。** 左辺が右辺を超える点が 2 種ある。(F) の解放の内側 (`DEF 節点の実行の素動作`) と、
op の生成コードが出す retain が作った参照が `Obl(α)` に在るあいだ (`DEF 節点の実行の素動作` の第 6 の箇条)
である。どちらの素動作も D34 の表に行を持たないので、右辺はそこで動かない。**この命題を読む段が使うのは
不等号だけである** (`L42` と `L41c`)。

**この命題は A19 を読まない。** 支えは D10 と D34 の帳簿の対応と、D24 の段の記述の網羅、およびその
網羅が段の境界についてであることを述べる節と、D25 の持ち手の 3 種である。

**証明**

<1>1. `α` の点で `Obl(α)` を動かす素動作は、D10 の行が定める作成と処分、(F) の解放が `Obl` に出し入れ
      するもの、および op の生成コードが出す retain が作る参照とその受け渡しに尽きる。D10 が `Obl` を
      変える事象は、初期値、`Retain`、`Release`、
      生成、消費の 5 種である。移動は `Obl` を変えない。D26 より、数えるのは計数下のオブジェクトへの
      参照だけである。`DEF 節点の実行の素動作` の残る 4 種のうち、子の活性化の素動作とグローバル化は
      `Obl(α)` を動かさず (L39b)、割り当ては D10 の生成の行と同じ素動作である (D24 の (E2) の `H` の
      表 -- 単一の `Fresh` と `Closure(f, caps)` の行が、割り当てたオブジェクトの `H` を 1 とする)。
      (F) の解放については L39a が、`Obl` に入れる参照がすべて同じ連鎖の中で `Obl` を離れること、
      連鎖の内側で `Obl` が下がらないことを述べる。
      **第 6 の箇条については、D24 が形を 2 つに分ける。**相殺しない形の持ち手はその生成コードが
      書き込むオブジェクトの持ち手の単位なので (D24、D25 の 2 番目)、`Obl(α)` を動かさない。
      相殺する形については D24 が
      「**その参照の持ち手は、その段を実行している活性化である** (D25 の 1 番目) -- retain と適用のあいだの
      段内の点で `Obl(a)` に在り、適用がそれを呼び出し先へ渡し、release が元の分を処分する。
      **素動作の粒度で `Obl` を勘定する段は、この 2 つを数える。**」と述べる。すなわち `Obl(α)` を動かす
      のは、retain が参照を 1 つ加える素動作と、適用がそれを呼び出し先へ渡す素動作の 2 つであり、
      release が処分する「元の分」は D9 の `Llvm` の行の消費、すなわち D10 の行が定める処分である。
      **`α` の点には `α` 自身の段でない段の中の点も在るが、そこで `Obl(α)` は動かない。** D24 は
      (E5) について「**この段は参照を作らず、渡さず、処分しない**」、(E6) と (E8) について
      「**この段は参照を作らず、渡さず、処分しない。**」と述べ、(E9) に
      ついては「**retain の段はその番地が指すオブジェクトへの参照を 1 つ作り、環境の持ち分に足す** --
      `H` はその分だけ上がり、`Obl` は動かない。**release の段は環境の持ち分から参照を 1 つ処分する** --
      `H` はその分だけ下がり、`Obl` は動かない。」と述べる。(E1) が `α` の初期 `Obl` へ渡す参照は
      D10 の初期値の事象であり、上の 5 種に既に在る。**`α` 以外の活性化を作る (E1) の段は `Obl(α)` を
      動かさない** -- D24 は (E1) を「C のエントリ点または `FFI_EXPORT` のエントリ点が、関数の本体 `B` の
      活性化 `a` を作り、`a` の初期 `Obl` (D10 の初期値) の各参照を `E` から渡す。」と定めるので、
      渡る先はその段が作る活性化の `Obl` であって `Obl(α)` ではない。ほかの活性化が実行する
      (E2)・(E3)・(E4)・(E7) の
      段が動かすのはその活性化の `Obl` である (D24) -- `α` の子の活性化についてこれを言うのが L39b で
      あり、別の制御の流れの活性化についても D24 の同じ行が当たる。
  BY <ref id=f06144e/>, <ref id=88a06de/>, <ref id=78073d2/>, <ref id=9d3dd4d/>, <ref id=e3436e8/>, <ref id=a89f403/>, <ref id=0b850c9/>, <ref id=9d74736/>, DEF 節点の実行の素動作
<1>1b. 2 が成り立つ。
  BY <1>1, <ref id=9d5d254/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=e3436e8/>, DEF 類ごとの義務
  `Obl(α)` の側は <1>1 が述べる。`held_ρ` の側は D34 の帳簿から出る -- 表の 6 行は `Retain`/`Release`
  の構文と、D10 の生成・初期値・D9 の消費の事象を主語にし、3 つの開始行はその類の終端の参照が
  `Obl(a)` に入る素動作と、その活性化が生きている活性化になる点に開始値を置くので、どれも `α` の
  構文と `α` の事象についての行である。D34 は「**この 3 つの箇条が置かない点では、`held` は直前の
  置き場所の値のままである。**」と述べるので、それ以外の段の素動作は `held_ρ` を動かさない。
  `β(C)` は点に依らないので `obl_ρ` も動かない (`DEF 類ごとの義務`)。
<1>2. これらの事象はいずれも 1 つの inhabited な leaf に紐づき、その leaf は `ρ` の上のスロット (D6)
      であって、ちょうど 1 つの別名類に属する。その類の `obj(C)` が計数下であるとき、その類は事象の
      時点で開始している (L40a の 2) ので、`obj(C) = O` であるものは `S(τ, O)` に入る。**計数下でない
      類は勘定に入らない** -- D26 より `Obl` が数えるのは計数下のオブジェクトへの参照だけであり、
      `S(τ, O)` も計数下の類だけを渡る (DEF 類ごとの義務)。
  BY <ref id=596a46d/>, <ref id=f06144e/>, <ref id=30d6238/>, <ref id=2ea7903/>, DEF 類ごとの義務
<1>3. D34 の表の 6 行は、この 5 種の事象と次のように対応する。第 1 行が生成、
      第 2 行と第 3 行が初期値 (所有する場合と借用する場合)、第 4 行が `Retain`、第 5 行が `Release`、
      第 6 行が消費である。
  BY <ref id=9d5d254/>, <ref id=f06144e/>
<1>4. D10 の生成の表の 5 行が名指す leaf は、いずれもその類の ρ-終端 (D33) である。また D10 の生成の
      5 行が名指す値の束縛は順に、`Binding::Llvm` (宣言が単一の `Arg` でない leaf)、`RcRhs::App` の
      `Binding::Producer`、`RcRhs::Closure` の `Binding::Producer`、boxed 容器の `Binding::Field`、
      boxed scrutinee の `Binding::Payload(_, Some(tag))` である。
  BY <ref id=2ea7903/>, <ref id=f06144e/>, CODE src/rc_ir/ownership.rs: collect_bindings
  前半は L40a の 3 が述べる。後半は D10 の生成の表と `collect_bindings` から出る -- `collect_bindings` は
  `RcRhs::Llvm` の結果に `Binding::Llvm` を、`RcRhs::App` と `RcRhs::Closure` の結果に
  `Binding::Producer` を、`Destructure` の名前付きフィールドに `Binding::Field(container, idx)` を、
  `Match` のアームの payload に `Binding::Payload(scrut, arm.tag)` を置く。
<1>5. 計数下の `O` について `obj(C) = O` である類 `C` の ρ-終端は、D10 の生成が作る leaf か、パラメータ・
      capture の leaf である。
  <2>1. ρ-終端に当たる `origin_inner` の腕は、`None`/`Param`/`Producer` の腕、`Binding::Llvm` の
        宣言が単一の `Arg` でない腕、`container.ty.is_box` が真の `Binding::Field` の腕、
        `scrut.ty.is_box` が真の `Binding::Payload(_, Some(_))` の腕の 4 つである。残る腕 --
        `Move`、`Join`、単一 `Arg` の `Llvm`、`is_box` が偽の `Field`、catch-all と `is_box` が偽の
        変位の `Payload` -- はいずれも `origin` を呼ぶ。**`Join` の腕が `origin` を呼ぶことは A9 が
        与える。** `Binding::Join(arm_results)` の腕は `arm_results` の各元について `origin` を呼ぶので、
        `arm_results` が空でなければ `origin` を呼ぶ。**`arm_results` はアームと 1 対 1 である** --
        `collect_bindings` は `Match` の各アームについて
        `arm_results.push(returned_var(&arm.body).clone())` を行い、その列を `Binding::Join(arm_results)`
        に据える。A9 より `Match` は 1 つ以上のアームを持つので `arm_results` は空でない。
        **A9 が `borrow_ify` の出力に
        当たることは A9 自身が述べる** -- 「`borrow_ify` と `cancel` は
        アームを持たない `Match` を作らないので (P22、P24)、`cancel` の入力と出力についても同じことが
        言える。」であり、`B` は `cancel` の入力すなわち `borrow_ify` の出力である (第 1 節)。
        `Binding::Llvm` の宣言が単一の
        `Arg` でない腕が ρ-終端であるのは、歩みの各位置の path が boxed leaf だからである -- L40 の
        1 より、boxed leaf の位置ではこの腕の `origin_from_leaves_under` は `origin` を呼ばない。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings,
       CODE src/rc_ir/ownership.rs: returned_var, CODE src/rc_ir/ownership.rs: Binding,
       <ref id=1172c08/>, <ref id=6fffd4c/>, <1>4, <ref id=30d6238/>
  <2>2. `None` の腕に当たるのは `vars.bindings` に束縛を持たない名前である。D6 より、その値はその記号の
        値であり、そこが指すのは funptr かグローバル状態のオブジェクトのどちらかであって、どちらも
        D8 の意味の参照を持たない。D34 は、束縛を持たない名前を ρ-終端とする類が計数下でないことを
        述べる。よってこの腕に当たる ρ-終端を持つ類は、計数下の `O` を指す類ではない。
    BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: VarTable::of,
       <ref id=596a46d/>, <ref id=ec8d1a0/>, <ref id=9d5d254/>, <ref id=b6673ca/>, <ref id=88a06de/>
  <2>3. QED
    BY <2>1, <2>2, <1>4, <ref id=6fffd4c/>, <ref id=f06144e/>, <ref id=b1f6e13/>, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: collect_bindings, 前提 束縛を置く在りか
    <2>1 が挙げる 4 つの腕のうち、`None` の腕に当たる leaf は <2>2 が計数下でないとして除く。
    `Binding::Param` を置くのは `VarTable::of` がパラメータと capture について行うところだけなので、
    `Param` の腕に当たるのはパラメータ・capture の leaf である。残るのは `Binding::Producer` の腕と、
    宣言が単一の `Arg` でない `Binding::Llvm`、boxed 容器の `Binding::Field`、boxed scrutinee の
    `Binding::Payload(_, Some(_))` の 3 つの腕である。L40 より歩みの各位置の path は boxed leaf なので、
    この 4 つに当たる leaf は、D10 の生成の表の 5 行が名指す位置である --
    `collect_bindings` が `Binding::Producer` を置くのは `RcRhs::App` と `RcRhs::Closure` の結果、
    `Binding::Field` を置くのは `Destructure` の名前付きフィールド、
    `Binding::Payload(_, Some(tag))` を置くのは `Match` の変位アームの payload だけである。
    **この 4 つの変位を置く在りかは `前提 束縛を置く在りか` が与える。** その走査が挙げるのは、
    `Binding::Param` について `VarTable::of` と `origin_inner`、残る 3 つについて
    `collect_bindings` と `origin_inner` であり、`origin_inner` に在るのはどれもパターンであって
    値を置く式ではない (`CODE src/rc_ir/ownership.rs: origin_inner`)。**`#[cfg(test)]` の下の作り手が
    範囲の外であることは P2a が置く** -- 「製品のコードが作る表はこの 2 つの構成子を通るものだけであり」
    であり、走査もその下の項目を除く。
<1>5a. D10 の 1 つの事象は、`Obl(・, O)` と `Σ_{C ∈ S(・, O)} obl_ρ(・, C)` を同じだけ動かす。
  BY <1>2, <1>3, <1>4, <1>5, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=9d5d254/>, <ref id=88a06de/>,
     CODE src/rc_ir/ownership.rs: VarTable::of, 前提 束縛を置く在りか, DEF 類ごとの義務
  <1>2 より、事象はちょうど 1 つの inhabited な leaf に紐づき、その leaf のスロットはちょうど 1 つの
  別名類 `C` に属する。D26 より `obj(C)` が計数下のときだけ両辺が動き、そのとき `Obl(・, obj(C))` と
  `held_ρ(・, C)` は <1>3 の対応で同じだけ動く -- D34 の第 4 行が `Retain`、第 5 行が `Release`、
  第 6 行が消費、第 1 行が生成である。D34 は「表の第 4・第 5・第 6 行の事象は、**それを運ぶ素動作の
  直後の段内の点**で `held` を動かす。」と述べ、「第 1 行と第 2 行の開始値は、**その類の終端の参照が
  `Obl(a)` に入る素動作の直後の段内の点**に置く。」と続けるので、両辺は同じ点で動く。`β(C)` は点に
  依らないので `obl_ρ` の動きは `held_ρ` の動きに等しい。**生成の行 (第 1 行) については `β(C) = 0` で
  ある** -- <1>4 よりその leaf は類の ρ-終端であり、`DEF 類ごとの義務` の `β(C)` はその ρ-終端が
  借用する (D14) パラメータ・capture の leaf であるときにだけ 1 であるところ、<1>4 の束縛の一覧より
  その leaf を持つ変数の束縛は `Binding::Llvm`・`Binding::Producer`・`Binding::Field`・
  `Binding::Payload` のいずれかであって `Binding::Param` ではなく、`VarTable::of` が `Binding::Param` を
  置くのはパラメータと capture についてだけである (`前提 束縛を置く在りか`)。よって生成の事象では
  D10 の生成の行が `Obl` に参照を 1 つ入れ、`held_ρ` の第 1 行も 1 から始まるので、両辺が 1 ずつ増える。
  初期値の 2 行については、所有する (D14) パラメータ・capture の leaf では D10 の初期値が
  参照を 1 つ入れ `held_ρ` の第 2 行も 1 から始まるので両辺が 1 ずつ増え (`β = 0`)、借用する leaf では
  D10 が参照を入れず、`held_ρ` の第 3 行の 1 は `β = 1` が打ち消すのでどちらも動かない。<1>5 より
  計数下の `O` を指す類の ρ-終端はこの 3 つのいずれかなので、ほかの開始行は無い。
<1>5b. `α` の素動作の列の上の帰納で本命題の言明を示す。**帰納の不変条件は言明より強い形に取る** --
      `e(τ, O)` を、op の生成コードが出す retain (`DEF 節点の実行の素動作` の第 6 の箇条の相殺する形) が
      `O` への参照として `Obl(α)` に加え、まだ `Obl(α)` を離れていないものの個数とし、

      `Obl(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + e(τ, O)`

      を示す。`e(τ, O) ≥ 0` なので、これから言明の不等式が出る。**この強め方が要るのは、その retain と
      それを呼び出し先へ渡す受け渡しの 2 つの素動作のあいだに、ほかの素動作が挟まりうるからである** --
      不等号だけを不変条件にすると、受け渡しが左辺を 1 下げる段が閉じない。
  <2>1. 基底。`α` が生きている活性化 (D23) になる点で、両辺は 0 である。`e` も 0 である -- `α` はまだ
        1 つも節点を実行していないので、op の生成コードは走っていない。
    BY <ref id=f06144e/>, <ref id=ff5985d/>, <ref id=9d5d254/>, <ref id=e3436e8/>, <ref id=a502f3e/>, <ref id=9d74736/>, DEF 類ごとの義務
    その点で `Obl(α)` は空である -- D10 の初期値が要求する参照は、`α` を作った段が受け渡しの素動作で
    `Obl(α)` に入れるものであり、その素動作はこの点より後にある。**活性化を作る段は 5 種である** --
    D24 の「活性化の林」は「(E1) が作る活性化を**根**、(E3) と (E7)、(E2) のうちオペランドを適用する
    `Llvm` の段、および (F) の解放が `Destructor` について作る段が作る活性化を、それを作った活性化の
    **子**と呼ぶ。」と述べ、「**活性化を作る段はこの 5 種で尽きる。**」と続ける。**5 種それぞれについて
    D24 が書くのは次のとおりである。** (E1) は「`a` の初期 `Obl` (D10 の初期値) の各参照を `E` から
    渡す。」、(E3) は「D9 の `App` の行が消費する各参照が `Obl(a)` を離れ、呼び出し先 (D23) の本体の
    新しい活性化 `b` が作られて、それらの参照が `Obl(b)` の初期値になる。」、(E7) は
    「`Obl(b)` は空で始まる (D1 より `init` はパラメータも capture も持たない)。」、
    (F) の `Destructor` については「`o` の `_value` の leaf が持つ参照は `o` を離れ、適用が作る
    活性化 `b` の `Obl(b)` の初期値に入る (D9 の `App` の行の消費)。」である。(E2) のうちオペランドを
    適用する `Llvm` の段については、D24 は「**作る活性化の初期 `Obl` は、この段が離した参照とは
    限らない。**」と書くが、その参照が `Obl(α)` に入るのはやはり受け渡しであり、D24 の
    「**切れ目を除くのは、上の 2 つの節だけである。**」より、活性化が作られる素動作とその受け渡しの
    あいだには点が在る。(E7) の場合は初期 `Obl` が空なので受け渡しそのものが無い。いずれの種でも、
    `α` が生きている活性化になる点で `Obl(α)` は空である。右辺の側では、D34 の第 1 行と
    第 2 行の開始値がまだ置かれておらず、第 3 行の開始値 (借用する終端の類) は
    「**その活性化が生きている活性化 (D23) になる点**」に置かれるが、その類は `β = 1` なので
    `obl_ρ = 0` である。D34 は「**置き方は自由ではない。** 開始値をまとめて活性化の初期 `Obl` を作る
    受け渡しの列の**直前**に置くと、所有するパラメータの boxed leaf を 2 つ以上持つ関数では、その列の
    途中の段内の点で `Σ d` が `Obl(a)` の個数を上回り、**A19 (i) がその点で偽になる**。」と述べ、
    この置き方を指定する。
  <2>2. 帰納段。ある点で <1>5b の不変条件が成り立つとき、次の素動作の後の点でも成り立つ。
    BY <1>1, <1>5a, <ref id=78073d2/>, <ref id=9d3dd4d/>, <ref id=9d5d254/>, <ref id=e3436e8/>, <ref id=a89f403/>, <ref id=1df9ec0/>, <ref id=8e052e9/>, <ref id=5b4974e/>, <ref id=8c40929/>, <ref id=c232680/>, DEF 節点の実行の素動作
    <1>1 の数え上げより、次の素動作は次のいずれかである。D10 の行が定める作成・処分のとき、<1>5a より
    第 1 項と第 2 項が同じだけ動き、`e` は動かないので不変条件が保たれる。子の活性化の素動作と
    グローバル化のとき、左辺は
    L39b より動かず、右辺も D34 の表がその素動作に行を持たず `e` も動かないので動かない。
    **op の生成コードが出す retain のとき**、左辺が 1 上がり `e` も 1 上がるので、両辺が同じだけ動く。
    **それが作った参照を呼び出し先へ渡す受け渡しのとき**、左辺が 1 下がり `e` も 1 下がるので、
    両辺が同じだけ動く -- D24 は「retain と適用のあいだの段内の点で `Obl(a)` に在り、適用がそれを
    呼び出し先へ渡し」と述べるので、この受け渡しはその retain より後にあり、`e ≥ 1` である。
    相殺しない形の retain は `Obl(α)` も `e` も動かさない (<1>1)。**`α` 自身の段でない段の素動作のとき**、
    左辺は <1>1 より動かず、右辺も D34 の表が `Retain`/`Release` の構文と D10 の事象にしか行を持たない
    ので動かず、`e` も動かない。**その段は D24 の (E1) から (E9) のいずれかである。**
    (E2)・(E3)・(E4)・(E7) が動かすのはその段を実行している活性化の `Obl` であり (D24)、`α` の子の
    活性化のものは `L39b` が扱い、**別の制御の流れの活性化のものも同じ行が当たる** -- D21 は
    「**別の制御の流れの段による増減は、この活性化の外から来る**」と述べて、それを `α` の `Obl` では
    なく `H` の側の与件に置く。(E1) が渡す参照はその段が作る活性化の `Obl` の初期値に入るのであって
    `Obl(α)` ではなく、(E5)・(E6)・(E8) について D24 は「**この段は参照を作らず、渡さず、処分しない。**」
    と述べ、(E9) について「`H` はその分だけ上がり、`Obl` は動かない。」「`H` はその分だけ下がり、
    `Obl` は動かない。」と述べる (<1>1)。
    (F) の解放の連鎖の素動作の
    とき、右辺は D34 の表がその素動作に行を持たず `e` も動かないので動かず、左辺は L39a より連鎖が
    終わった点の値以上であって、連鎖が終わった点で
    その値に戻る。連鎖を起こした処分は D10 の行の事象なので <1>5a が第 1 項と第 2 項を同じだけ下げており、その
    下げは D34 が「それを運ぶ素動作の直後の段内の点」に置くので、連鎖の内側の各点で右辺は既に下がって
    いる。よって連鎖の内側でも不変条件は保たれる。
  <2>3. QED
    BY <2>1, <2>2, <ref id=e3436e8/>
    D24 より実行の素動作は 1 つの列をなすので、`α` のどの点についてもその前の素動作は有限個であり、
    その接頭についてこの帰納が届く。`e(τ, O) ≥ 0` なので、不変条件から言明の不等式が出る。
<1>6. QED
  BY <1>1b, <1>5a, <1>5b

### L41 (余りの下界) <!--#217aebd-->

**言明** --- 実行路 `ρ` と、`ρ` を辿る 1 回の活性化を固定する。`ρ` の上の節点 `q` と計数下のオブジェクト
`O` について、次の 2 つが成り立つ。

1. `DEF N` の `N(q, O)` は、README の P18a の「走査中の位置」を `q` の訪問の入口に取ったときの `n(O)`
   であり、`p13` の `DEF N` の `N_ρ(q, O)` である。
2. `N(q, O) ≥ 1` ならば、`q` の入口の点での参照カウントは `H(q, O) ≥ N(q, O) + 1` である。

**主語は D21 の意味の活性化である。** D21 は各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものだけを
活性化とするので、この命題の 2 を `α` について読む段は、`α` がその制限を満たすことに立つ (`L44` の (f))。
**2 は各時点についての言明 (P18a) を読むので、README 第 2 節の接頭の規則に従う** -- `q` の入口の点に
ついて 2 を読むには、その点までの各点で `α` が D21 の制限を満たしていることが要る。

**証明**

<1>1. 1 が成り立つ。すなわち P18a の「走査中の位置」を節点 `q` の訪問の入口に取ると、その `n(O)` は
      `N(q, O)` であり、`p13` の `DEF N` の `N_ρ(q, O)` である。
  BY <ref id=97bdd4e/>, <ref id=8093b68/>, DEF N, <ref id=b154692/>, <ref id=c4ea962/>, p13 の DEF N
  D27 は `B(p, ρ)` を節点の訪問の入口で定め、P18a はその `B(p, ρ)` を使って
  `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と置く。`L45` より P18a の `obj(o)` は `obj_ρ(o)` であり、
  内側の和を活性な名前に制限してよいのは、`p13` の `L11` (ii) より活性でない名前の `B_ρ` が 0 だから
  である。DEF N がその和である。
<1>2. 2 が成り立つ。
  BY <ref id=97bdd4e/>, <1>1
  P18a は `n(O) ≥ 1` のとき `H(O) ≥ n(O) + 1` を述べる。
<1>3. QED
  BY <1>1, <1>2

### L41a (類の義務は非負であり、節点の入口では bump 以上である) <!--#89569b4-->

**言明** --- 実行路 `ρ` を辿る活性化を固定する。次の 2 つが成り立つ (DEF 類ごとの義務)。

1. 終端の `Ret` の消費より前の**各点** `τ` と、`held_ρ(τ, ・)` が定まる各計数下の別名類 `C` について
   `obl_ρ(τ, C) ≥ 0` である。
2. 終端の `Ret` の消費より前にある `ρ` の上の**節点** `n` とその入口の点 `p` (`DEF 節点の入口の点`)、
   および `held_ρ(p, ・)` が
   定まる各計数下の別名類 `C` について `obl_ρ(p, C) ≥ bumps_ρ(n, C)` である。

**2 が節点の入口の点に限るのは、A19 の (ii-b) がそこでしか読めないからである。** A19 は
「**「各時点」は、その活性化が生きている (D23) 間の、その活性化の節点の訪問の入口である時点である。**」と
定め、「**(ii-b) は段内の点では読めない。**」と続ける。段内の点で `obl_ρ` に言えるのは 1 の非負性までで
あり、それを与えるのは (ii-c) と P14a である。**2 の主語に節点 `n` を置くのは、走査の状態が節点の訪問に
付くからである** (`DEF 節点の実行の素動作`) -- 1 つの点が 2 つの節点の入口の点でありうるので、
`bumps_ρ` を読むときは節点を名指す。A19 の (ii-b) は各節点の訪問の入口についての言明なので、`n` の
訪問について読める。

**主語は D21 の意味の活性化である。** この命題が読む A19 の (ii-a)・(ii-b)・(ii-c) と P14a は、どれも
各実行路とそれを辿る各活性化についての言明であり、D21 は各時点と各段内の点で A19 (i) の不等式を満たす
ものだけを活性化とする。`α` についてこの命題を読む段は、`α` がその制限を満たすことに立つ (`L44` の (f))。
この注は、この命題を読む `L41b`・`L41c`・`L41d`・`L42` にも掛かる。

**証明**

<1>1. `ρ` の上の各節点 `n` と各計数下の別名類 `C` について `bumps_ρ(n, C) ≥ 0` である。
  BY DEF 類ごとの義務, <ref id=8093b68/>, <ref id=948f840/>
  `bumps_ρ(n, C)` は `B_ρ` の個数の総和であり、P18b より各個数は 0 以上である。
<1>2. 1 が成り立つ。
  <2>1. CASE `β(C) = 0` かつ `τ` が節点の入口の点である。A19 の (ii-a) より `held_ρ(τ, C) ≥ 0` であり、
        DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 0` である。
    BY <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 節点の入口の点
    (ii-a) は「各時点と各計数下の別名類について、その類が持つ参照の個数は非負であり」と述べ、A19 は
    その「各時点」を「その活性化が生きている (D23) 間の、その活性化の節点の訪問の入口である時点」に
    限る。
  <2>2. CASE `β(C) = 0` かつ `τ` が、`ρ` の上の節点 `n` の実行の点であって `n` の入口の点でも `n'` でも
        ない。A19 の (ii-c) より `held_ρ(τ, C) ≥ 0` であり、
        DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 0` である。
    BY <ref id=9f1cf6c/>, 前提 (ii-c) の保存, DEF 類ごとの義務, DEF 節点の入口の点
    A19 の (ii-c) は「**(ii-c) (段内の点の非負性)。節点の実行の途中の各点 (D24 の段内の点) と、その点で
    `held_ρ` が定まる各計数下の別名類について、`held ≥ 0` である。**」と述べる。**量化は太字の中に在り、
    「その点で `held_ρ` が定まる」の制限もその中に在る** -- 本場合の `C` は `held_ρ(τ, ・)` が定まる類
    なのでその制限を満たす。本場合の `τ` は `n` の実行の途中の点なので、(ii-c) の量化に入る。この文書が
    扱う本体は `borrow_ify` の出力なので、その範囲について (ii-c) を持つのは `前提 (ii-c) の保存` で
    ある。
  <2>2a. CASE `β(C) = 0` かつ `τ` が、`ρ` の上の節点 `q` について `q'` であるか、`q` の後の節点の間の
         点である。
    BY <ref id=9f1cf6c/>, <ref id=941af96/>, <ref id=561fd05/>, <ref id=9d5d254/>, <ref id=ca36627/>, <ref id=68112c9/>, DEF 類ごとの義務, DEF 節点の入口の点
    **このとき `q` は `ρ` の終端の `Ret` ではない** -- 本命題が量化するのは終端の `Ret` の消費より前の
    点であり、終端の `Ret` の `q'` は `q_end`、すなわちその消費の直後の点だからである
    (`DEF 節点の入口の点`)。D3 と L33a より `ρ` の最後の節点は終端の `Ret` なので、`ρ` の上で `q` の
    次にある節点 `m` が在る。その入口の点を `τ_m` と書く。`L50` の 4 より `q'` から `τ_m` までのあいだに
    `α` の素動作は無く、挟まる段はどれも `α` が実行する段ではないので、`L49` の 2 よりその素動作は
    `held_ρ` を動かさない。D34 は「**この 3 つの箇条が置かない点では、`held` は直前の置き場所の値の
    ままである。**」と述べるので `held_ρ(τ, C) = held_ρ(τ_m, C)` である。`τ_m` は節点の入口の点なので
    A19 の (ii-a) が当たり `held_ρ(τ_m, C) ≥ 0` であり、DEF 類ごとの義務 より
    `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 0` である。
  <2>3. CASE `β(C) = 1`。P14a より `held_ρ(τ, C) ≥ 1` であり、
        `obl_ρ(τ, C) = held_ρ(τ, C) - 1 ≥ 0` である。
    BY <ref id=1e8ca86/>, DEF 類ごとの義務, <ref id=9d5d254/>, DEF 実行時の量
    P14a は「`borrow_ify` の出力の各本体、各実行路、各活性化について、ρ-終端が借用する (D14) パラメータ・
    capture の leaf である**計数下**の別名類 (D26) は、活性化の間ずっと参照を少なくとも 1 つ持つ」と
    述べる。`B` は `borrow_ify` の出力の本体である (第 1 節)。`β(C) = 1` の計数下の類はその形であり、
    類が持つ参照の個数は `held_ρ` である (D34)。**その「活性化の間ずっと」が段内の点まで読めることは
    P14a の脇が定める** -- 「**「活性化の間ずっと」は、各時点と各段内の点 (D24) である。** `held` は
    その粒度で定まる (D34) ので、この節はそこまで読める。」。`DEF 実行時の量` よりこの文書の点は
    その段内の点なので、`τ` が節点の入口の点であるかを問わずこの節が当たる。
  <2>4. QED
    BY <2>1, <2>2, <2>2a, <2>3, <ref id=ff5985d/>, DEF 類ごとの義務, DEF 節点の入口の点
    DEF 類ごとの義務 より `β(C)` は 0 か 1 である。`β(C) = 1` のときは <2>3 が点を問わず与える。
    `β(C) = 0` のときは点で場合分けする -- `DEF 節点の入口の点` は「**`ρ` の上の点は、節点の実行の点と
    節点の間の点で尽きる。**」と述べ、節点 `n` の実行の点を「`n` の入口の点から `n'` までの各点
    (両端を含む)」と定めるので、点は「ある節点の入口の点」「ある節点の実行の点であって入口でも `n'` でも
    ない点」「ある節点の `n'`」「節点の間の点」で尽きる。第 1 を <2>1、第 2 を <2>2、第 3 と第 4 を
    <2>2a が扱う。**節点の間の点はどれかの節点の `q'` の後に在る** -- `DEF 節点の入口の点` はそれを
    「`q'` と、`ρ` の上で `q` の次にある節点の入口の点とのあいだに在る点」と定める。
<1>3. 2 が成り立つ。
  <2>1. CASE `bumps_ρ(n, C) = 0`。<1>2 より `obl_ρ(p, C) ≥ 0 = bumps_ρ(n, C)` である。
    BY <1>2
  <2>2. CASE `bumps_ρ(n, C) ≥ 1` かつ `β(C) = 0`。A19 の (ii-b) より
        `held_ρ(p, C) ≥ 1 + bumps_ρ(n, C)` であり、`obl_ρ(p, C) = held_ρ(p, C) ≥ bumps_ρ(n, C)` で
        ある。
    BY <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 節点の入口の点, DEF 節点の実行の素動作
    (ii-b) は「`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である」と述べ、A19 はその「各時点」を
    節点の訪問の入口である時点に限る。`p` は `n` の訪問の入口の時点であり、その訪問について走査が
    数えている bump が `bumps_ρ(n, C)` である (`DEF 節点の実行の素動作`)。
  <2>3. CASE `bumps_ρ(n, C) ≥ 1` かつ `β(C) = 1`。A19 の (ii-b) より
        `held_ρ(p, C) ≥ 1 + bumps_ρ(n, C)` であり、`obl_ρ(p, C) = held_ρ(p, C) - 1 ≥ bumps_ρ(n, C)` で
        ある。
    BY <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 節点の入口の点, DEF 節点の実行の素動作
  <2>4. QED
    BY <1>1, <2>1, <2>2, <2>3, DEF 類ごとの義務
    <1>1 より `bumps_ρ(n, C)` は 0 か 1 以上であり、DEF 類ごとの義務 より `β(C)` は 0 か 1 である。
<1>4. QED
  BY <1>2, <1>3

**終端の `Ret` の消費より前に限るのは、A19 の (ii-b) がその点で偽だからである。** A19 は
「**延ばすのは (ii-a) だけである。(ii-b) はこの位置で偽である。**」と述べ、反例を挙げる。

### L41b (節点の入口では、類ごとの余りが 1 つ立つ) <!--#3efe52c-->

**言明** --- 実行路 `ρ` を辿る活性化を固定する。終端の `Ret` の消費より前にある `ρ` の上の**節点** `n`
とその入口の点 `p` (`DEF 節点の入口の点`)、計数下のオブジェクト `O`、および `S(p, O)` の部分集合 `S` に
ついて、`Σ_{C ∈ S} bumps_ρ(n, C) ≥ 1` ならば

`Σ_{C ∈ S} obl_ρ(p, C) + [S(p, O) に β(C) = 1 の類が在るならば 1] ≥ Σ_{C ∈ S} bumps_ρ(n, C) + 1`

である。`S(p, O)` は `obj(C) = O` であって `p` までに開始した計数下の別名類の全体である
(DEF 類ごとの義務)。**`S = S(p, O)` と取れば左辺の総和は `S(p, O)` を渡り、右辺は `b(n, O) + 1` に
なる。** 部分集合の形で述べるのは、`L41c` がその形で読むからである。

**証明**

<1>1. PICK `C0 ∈ S` SUCH THAT `bumps_ρ(n, C0) ≥ 1`。
  BY 本命題の仮定, DEF 類ごとの義務, <ref id=8093b68/>, <ref id=948f840/>
  仮定の総和は `S` を渡る `bumps_ρ` の和であり、P18b より各項は 0 以上なので、和が 1 以上ならば 1 以上の
  項が在る。
<1>2. `C0` 以外の各 `C ∈ S` について `obl_ρ(p, C) ≥ bumps_ρ(n, C)` である。
  BY <ref id=89569b4/>
  `p` は `n` の入口の点なので L41a の 2 が当たる。
<1>3. CASE `β(C0) = 0`。A19 の (ii-b) より `held_ρ(p, C0) ≥ 1 + bumps_ρ(n, C0)` であり、
      DEF 類ごとの義務 より `obl_ρ(p, C0) = held_ρ(p, C0) ≥ bumps_ρ(n, C0) + 1` である。角括弧は 0 以上
      なので、<1>2 を残りの類について足すと言明の不等式になる。
  BY <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 節点の入口の点, DEF 節点の実行の素動作, <1>1, <1>2
<1>4. CASE `β(C0) = 1`。A19 の (ii-b) より `held_ρ(p, C0) ≥ 1 + bumps_ρ(n, C0)` であり、
      DEF 類ごとの義務 より `obl_ρ(p, C0) = held_ρ(p, C0) - 1 ≥ bumps_ρ(n, C0)` である。`C0 ∈ S ⊆ S(p, O)`
      は `β(C0) = 1` の類なので角括弧は 1 である。<1>2 を残りの類について足すと言明の不等式になる。
  BY <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 節点の入口の点, DEF 節点の実行の素動作, <1>1, <1>2
<1>5. QED
  DEF 類ごとの義務 より `β(C0)` は 0 か 1 なので、<1>3 と <1>4 が場合を尽くす。**`+1` が 1 回しか
  立たないのは、`C0` を `β` で分けたどちらの場合でもそれを出すのが 1 つだけだからである** -- `β(C0) = 0`
  では `C0` の `obl_ρ`、`β(C0) = 1` では角括弧である。
  BY <1>1, <1>2, <1>3, <1>4

### L41e (節点の実行の各点の類ごとの義務は、ある節点の入口の点のものを下回らない) <!--#5c7bebf-->

**言明** --- 実行路 `ρ` を辿る活性化と、`ρ` の上の節点 `q` を固定する。`q` が `ρ` の終端の `Ret` で
ないとき、`ρ` の上の節点 `n(q)` を次で定める -- `q` が `Retain` 節点であれば `n(q) := q`、そうでなければ
`n(q) :=` `ρ` の上で `q` の次にある節点。`p(q)` を `n(q)` の入口の点とする。`τ` を `q` の実行の各点
(`DEF 節点の入口の点`) とすると、次の 3 つが成り立つ。

1. `n(q)` は 1 つに定まる。`q` が `Retain` 節点であれば `p(q)` は `q` の入口の点そのものであり、
   そうでなければ `p(q)` は `q'` かそれより後の点である -- そのあいだに在るのは節点の間の点だけである
   (`DEF 節点の入口の点`)。
2. `q` の入口の点までに開始した各計数下の別名類 `C` について `obl_ρ(τ, C) ≥ obl_ρ(p(q), C)` である。
3. 各計数下オブジェクト `O` について `S(q, O) ⊆ S(τ, O) ⊆ S(p(q), O)` であり、`S(q, O)` に `β = 1` の類が
   在ることと `S(τ, O)` にそれが在ることは同値である。

**点と節点を対にして定めるのは、走査の状態が節点の訪問に付くからである** (`DEF 節点の実行の素動作`)。
`bumps_ρ` は節点を引数に取るので、`q` の実行の点でそれを読む段は節点 `n(q)` を名指す。`obl_ρ` と `S` は
点だけで定まるので、2 と 3 は点 `p(q)` について述べる。

**証明**

<1>1. 1 が成り立つ。
  BY DEF 節点の入口の点, DEF 節点の実行の素動作, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=68112c9/>, <ref id=941af96/>
  `q` が `Retain` 節点のときは `n(q) = q` なので `p(q)` は `q` の入口の点そのものである。そうでない
  とき、`q` は `ρ` の終端の `Ret` でないので、D3 と L33a より `ρ` の最後の節点は終端の `Ret` であり、
  `q` は `ρ` の上に直後の節点をちょうど 1 つ持つ。`DEF 節点の入口の点` より、`q'` とその節点の入口の点の
  あいだに在るのは節点の間の点だけである。
<1>2. 開始した類の `held_ρ` を増やす事象は、D34 の表の第 4 行 -- `Retain(v, π)` -- だけである。
  BY <ref id=9d5d254/>, <ref id=561fd05/>, DEF 節点の入口の点
  D34 の表の 6 行のうち、第 1・第 2・第 3 行はその類の**開始**の値を置く行であり、開始した後の類には
  当たらない。第 5 行 (`Release`) と第 6 行 (消費) は `held_ρ` を下げる。よって開始した類の `held_ρ` を
  上げるのは第 4 行だけである。**表に行を持たない素動作は `held_ρ` を動かさない** -- D34 は
  「**この 3 つの箇条が置かない点では、`held` は直前の置き場所の値のままである。**」と述べる。
  **`α` が実行する段でない段の素動作もそこに入る** -- D34 の 6 行はどれも `α` の構文と `α` の事象を
  主語にするので、環境の段と別の制御の流れの活性化の段 (`DEF 節点の入口の点`) は行を持たない
  (`L49` の 2)。
<1>3. 2 が成り立つ。
  <2>1. CASE `q` が `Retain` 節点である。D34 の表の第 5 行が当たるのは `Release` 節点、第 6 行が当たるのは
        消費点 (`DEF 消費点`) であり、`q` はそのどちらでもない。よって `q` の実行の間 `held_ρ(・, C)` は
        下がらず、`obl_ρ(τ, C) = held_ρ(τ, C) - β(C) ≥ held_ρ(q, C) - β(C) = obl_ρ(p(q), C)` である。
    BY <1>2, <ref id=9d5d254/>, <ref id=9d74736/>, DEF 消費点, DEF 類ごとの義務
  <2>2. CASE `q` が `Retain` 節点でない。D34 の表の第 4 行が当たるのは `Retain` 節点だけなので、
        <1>2 より `held_ρ(・, C)` は `q` の実行の間 下がるだけである。`q'` から `p(q)` までに在るのは
        節点の間の点だけであり、そこで `α` は素動作を持たないので `held_ρ(・, C)` は動かない (<1>2)。
        よって `τ` から `p(q)` まで `held_ρ(・, C)` は下がるだけであり、
        `held_ρ(τ, C) ≥ held_ρ(p(q), C)`、すなわち `obl_ρ(τ, C) ≥ obl_ρ(p(q), C)` である。
    BY <1>2, <1>1, <ref id=9d5d254/>, <ref id=941af96/>, DEF 類ごとの義務, DEF 節点の入口の点
  <2>3. QED
    BY <2>1, <2>2
<1>4. 3 が成り立つ。
  BY <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=b3dfa37/>, <ref id=ff5985d/>, <1>1, DEF 類ごとの義務, DEF 節点の入口の点
  DEF 類ごとの義務 より `S(・, O)` は `obj(C) = O` であってその点までに開始した計数下の類の全体で
  あり、開始の点は点の列の上で 1 つに決まる (D34) ので、点が進めば集合は増えるだけである。
  **`q` が `Retain` 節点でないとき。** <1>1 より `p(q)` は `q'` 以後にあるので、
  `S(q, O) ⊆ S(τ, O) ⊆ S(p(q), O)` である。
  **`q` が `Retain` 節点のとき。** <1>1 より `p(q)` は `q` の入口の点そのものなので、単調性が与えるのは
  `S(p(q), O) = S(q, O) ⊆ S(τ, O)` の向きである。**残る包含は、`q` の実行の間に開始する類が無いことから
  出る** -- D34 の 3 つの箇条が開始値を置くのは、第 1 行 (D10 の生成) と第 2 行 (所有するパラメータ・
  capture の leaf) についてはその類の終端の参照が `Obl(a)` に入る素動作の直後の段内の点、第 3 行
  (借用する leaf) についてはその活性化が生きている活性化 (D23) になる点である。`Retain` 節点は D10 の
  生成の表にも初期値の行にも当たらず (D10 はその節点を別の行で扱う)、活性化が生きている活性化になる点は
  `ρ` の最初の節点の入口の点以前にある (D23 -- 活性化が始まった時点では位置は `B(a)` の根である)。
  よって `q` の実行の点で開始値が置かれることはなく、`S(τ, O) = S(q, O) = S(p(q), O)` である。
  `β(C) = 1` の類の ρ-終端はパラメータ・capture の leaf なので、その開始の点は D34 の第 3 行が
  「**その活性化が生きている活性化 (D23) になる点**」に置く。すなわちその類は `q` の入口の点で既に
  開始しており、`S(q, O)` に在ることと `S(τ, O)` に在ることは同値である。

### L42 (節点の入口では義務が pending の bump を覆う) --- これが P18c である <!--#0669029-->

**言明** --- 実行路 `ρ` を辿る 1 回の活性化 `α` を固定する。終端の `Ret` の消費より前にある `ρ` の上の
節点 `q` の入口の点と各計数下オブジェクト `O` について、`Obl(q, O) ≥ b(q, O) = N(q, O)` である。

**主語は D21 の意味の活性化である。** `L41a` の脇の注がこの命題にも掛かる。

**証明**

<1>1. `Obl(q, O) ≥ Σ_{C ∈ S(q, O)} obl_ρ(q, C)` である。
  BY <ref id=561fd05/>
<1>2. 各 `C ∈ S(q, O)` について `obl_ρ(q, C) ≥ bumps_ρ(q, C)` である。
  BY <ref id=89569b4/>
  L41a の 2 を、節点を `q`、その入口の点を `q` の入口の点として読む。本命題が量化するのは終端の `Ret` の
  消費より前の節点なので、その範囲に入る。
<1>3. QED
  BY <1>1, <1>2, <ref id=5c1f4e7/>, <ref id=2ea7903/>, <ref id=217aebd/>, DEF 類ごとの義務, DEF N
  <1>1 と <1>2 より `Obl(q, O) ≥ Σ_{C ∈ S(q, O)} obl_ρ(q, C) ≥ Σ_{C ∈ S(q, O)} bumps_ρ(q, C) = b(q, O)`
  である。`p13` の `L17` より `N_ρ(q, O) = Σ_{C ∈ S(q, O)} bumps_ρ(q, C)` であり、`L41` の 1 より
  `N(q, O) = N_ρ(q, O)` である。L40a の 1 が、`obj(C) = O` である類を渡る和と `S(q, O)` を渡る和が
  等しいことを与える。

### L43 (欠損は pending の bump の一部である) <!--#4ff3e8d-->

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
  BY <ref id=7d5a1de/>, DEF 欠損
<1>2. `q` の入口の点より前に実行された `Del` の `Release` 節点 `r` は、ある `t ∈ CT` について
      `r ∈ R_ρ(t)` であり、
      その `t` は `ρ` の上で `r` より前にある。
  BY <ref id=7d5a1de/>, <ref id=23ac734/>, <ref id=5adaf7f/>
  L32 の 3 より `r` はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に属する。L37 より `t` は `ρ` の
  上で `r` より真に前にあり、`t` は `r` で pending である。よって `r ∈ I_ρ(t)` であり、L32 の 4 より
  `r` の訪問の `un_bump` は `InBracket(t)` を返すので、L38 の 3 の定義より `r ∈ R_ρ(t)` である。
<1>3. `t ∈ CT` が `ρ` の上で `q` より前にあり、`q` で pending でないとき、`R_ρ(t)` の全要素は `q` より前に
      実行されており、`t` と `R_ρ(t)` の寄与の和は 0 である。
  BY <ref id=5adaf7f/>, <ref id=2307c1e/>, <1>2
  L38 より `I_ρ(t)` は `t` の直後から始まる連続した区間であり、`t` が `q` で pending でないので区間は
  `q` より前で終わる。`R_ρ(t) ⊆ I_ρ(t)` なのでその全要素は `q` より前にある。L39 より
  `ActRefs^inh_ρ(t) = Σ_{r ∈ R_ρ(t)} ActRefs^inh_ρ(r)` なので、作った参照と処分した参照は打ち消し合う。
<1>4. `t ∈ CT` が `q` で pending であるとき、`t` と、`q` より前に実行された `R_ρ(t)` の要素の寄与の和は、
      名前ごとに `ActRefs^inh_ρ(t) - Σ_{r ∈ R_ρ(t), r は q より前} ActRefs^inh_ρ(r)` であり、これは
      `B_ρ(q, e_t(q))` に等しい。
  BY <ref id=8093b68/>, <ref id=5adaf7f/>, <ref id=941af96/>, <1>2
  `e_t(q)` が 1 つに定まることは L50 の 1 である。
  D27 は、`Retain` の訪問で押し込まれた要素の `B` を `ActRefs^inh_ρ(t)` と定め、`un_bump` が
  `InBracket` でその要素を選ぶ `Release` の訪問でだけ `ActRefs^inh_ρ` を引き、複製・`merge`・その他の
  節点では値を運ぶだけである。その要素を選ぶ `Release` は L38 の 3 の `R_ρ(t)` の要素である。
<1>5. 名前ごとの寄与をオブジェクトごとの寄与に直す。`Retain`/`Release` が実際に作る (処分する) 参照は、
      その名前 `o` について、オブジェクト `obj_ρ(o)` への参照である。よって `O` への寄与は
      `obj_ρ(o) = O` である名前 `o` の分の和である。
  BY p13 の DEF 名前の活性, p13 の DEF 実行時の作用, <ref id=0b3e0e1/>, <ref id=4f4714c/>, DEF 節点の量
  **この段が P5 (a) を当てる対は、その前件の述語に入る。** README の P5 は「**前件を満たす側は述語で
  決める。**」に続けて「**解析が `origin` を呼ぶ鍵とは、`src/rc_ir/borrow.rs` の `origin(` の呼び出しが
  その実行で渡す `(変数, path)` の対の全体である**」と定め、「**前件を満たすことを示す段は、その対が
  この述語に入ることを言う。**」と続ける。`Retain(v, π)` 節点と `Release(v, π)` 節点について走査が
  読む量は `self.acted_references(v, π)` の値であり (`DEF 節点の量`)、P6 は「**その各 leaf について
  解析が `origin` を呼ぶ** -- `acted_references` 自身がその呼び出しである。」と述べる。この段が
  identity を比べる対は `L(v, π)` の各 leaf `λ` についての `(v, λ)` なので、述語に入る。
<1>6. 言明の等式が成り立つ。
  BY <1>1, <1>2, <1>3, <1>4, <1>5, DEF 欠損
<1>7. `0 ≤ d(q, O) ≤ N(q, O)` である。
  BY <1>6, <ref id=948f840/>, <ref id=c4ea962/>, DEF N
  P18b は `B(p, ρ)` を「`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上で
  まだ処分されていないものを、それを作った leaf の `origin` の identity で名付けた多重集合」と定めるので、
  `B_ρ(q, e)` はどの名前についても個数が 0 以上である。`p13` の `L11` より、活性でない `o` に
  ついては `B_ρ(q, e)[o] = 0` である。よって <1>6 の和は 0 以上である。DEF N の `N(q, O)` は同じ内側の和を
  `pending(q)` の**すべての**要素について取ったものなので、`d(q, O) ≤ N(q, O)` である。
<1>7a. `d(q_end, O) = 0` である。
  BY <1>6, <ref id=5adaf7f/>, <ref id=7d5a1de/>, DEF 欠損
  L38 の 2 より、各 `t ∈ CT` が pending である区間は `ρ` の終端の `Ret` より真に前で終わる。よって終端の
  `Ret` の入口で pending な `CT` の要素は無く、<1>6 の等式の右辺の和は空で `d = 0` である。L32 の 5 より
  `Del` の要素は `Retain` 節点か `Release` 節点だけなので、終端の `Ret` は `Del` に入らず、その消費は
  DEF 欠損 の 2 つの数え上げのどちらも変えない。
<1>8. QED
  BY <1>2, <1>6, <1>7, <1>7a, <ref id=7d5a1de/>, DEF 欠損
  等式は <1>6、境界は <1>7、`q_end` の値は <1>7a である。P21 (a) の `k(O)` が `d(q, O)` に等しいことは
  次から出る。
  DEF 欠損 より `d(q, O)` は、`q` より前に実行された `Del` の `Retain` 節点が `O` に作った参照の個数から、
  `q` より前に実行された `Del` の `Release` 節点が `O` の参照を処分した個数を引いたものである。L32 の 3 と
  <1>2 より、`Del` の各 `Release` 節点 `r` はちょうど 1 つの `t ∈ CT` の `un_bump_releases[t]` に属し、
  その `t` は `ρ` の上で `r` より前にある。すなわち `Del` の各 `Release` は `Del` のちょうど 1 つの
  `Retain` と対になる。よって引かれる個数は対になる `Retain` ごとに分けて数えられ、`d(q, O)` は「`Del` の
  各 `Retain` が `O` に作った参照のうち、対になる `Del` の `Release` がまだ処分していないもの」の総数で
  ある。これが P21 (a) の `k(O)` である。

### L43a (`Del` に入らない節点を越えても、pending の bump は欠損を覆う) <!--#dd86b2c-->

**言明** --- `ρ` の上の節点 `q` を取り、`q` が `ρ` の終端の `Ret` でないときは
`n(q)` を `L41e` が定める節点とする。1 は任意の `q` について、2 と 3 は `q` が `Del` に入らないときに
成り立つ。

1. 各計数下オブジェクト `O` について `d(q, O) = Σ_{C ∈ S(q, O)} d_C(q)` である (DEF 類ごとの義務)。
2. `q` が `ρ` の終端の `Ret` でないとき、`q` で pending である `CT` の各要素は `n(q)` でも pending で
   あり、その `B_ρ` は `q` の入口の点での値に等しい。
3. `q` が `ρ` の終端の `Ret` でないとき、各計数下の別名類 `C` について
   `d_C(q) ≤ bumps_ρ(n(q), C)` である。

**証明**

<1>1. `q` が `ρ` の終端の `Ret` でないとき、`n(q)` は `ρ` の上の節点であり、`q` が `Retain` 節点なら
      `n(q) = q`、そうでなければ `n(q)` は `ρ` の上で `q` の次にある節点である。
  BY <ref id=5c7bebf/>
<1>2. 2 の前半が成り立つ。すなわち `q ∉ Del` のとき、`q` で pending である `t ∈ CT` は `n(q)` でも
      pending である。
  <2>1. CASE `q` が `Retain` 節点である。<1>1 より `n(q) = q` なので、示すべきは「`q` で pending で
        ある `t ∈ CT` は `q` でも pending である」であり、これはその前提そのものである。
    BY <1>1
  <2>2. CASE `q` が `Retain` 節点でない。<1>1 より `n(q)` は `ρ` の上で `q` の次にある節点である。
        L38 より、`t` が pending である `ρ` の上の節点の全体 `I_ρ(t)` は `ρ` の上の連続する区間で
        あり、その最後の節点 `n*(ρ)` は `un_bump_releases[t]` の要素である (L38 の 1)。`t ∈ CT` なので
        L32 の 3 より `n*(ρ) ∈ Del` であり、本命題の仮定より `q ∉ Del` なので `q ≠ n*(ρ)` である。
        よって `q` は区間の最後の節点ではなく、`n(q) ∈ I_ρ(t)` である。
    BY <ref id=5adaf7f/>, <ref id=7d5a1de/>, 本命題の仮定, <1>1
  <2>3. QED
    BY <2>1, <2>2
<1>3. 2 の後半が成り立つ。すなわちその要素の `B_ρ` は `q` の入口の点での値に等しい。
  <2>1. CASE `q` が `Retain` 節点である。<1>1 より `n(q) = q` なので、`B_ρ(n(q), ・)` は `q` の訪問の
        入口で読んだ値そのものである (`DEF N`)。
    BY <1>1, DEF N
  <2>2. CASE `q` が `Retain` 節点でない。`q` の訪問が `pending` に施す操作は、L34 の 3 つの場合で
        尽きる -- 1 の場合は `push`・`consume_objects`・`un_bump`、2 の場合はアームへの複製、3 の
        場合は `merge` である。
    <3>1. `push` は末尾に新しい要素を足すだけで、既に在る要素の `node` と `outstanding` を変えない。
      BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Retain(v, path, _, k)` の腕
    <3>2. `consume_objects` は由来が `t` の要素を取り除かない。
      BY <ref id=5adaf7f/>, <ref id=7855e90/>
      L38 の 4 が、`I_ρ(t)` に入る節点の訪問の中で `consume_objects` がその要素を取り除かない
      ことを述べる。
    <3>3. `un_bump` は由来が `t` の要素の `outstanding` を引かず、取り除きもしない。
      BY <ref id=19296b2/>, <ref id=7d5a1de/>, 本命題の仮定, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
      `un_bump` を呼ぶのは `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕だけなので、`un_bump`
      が走るとき `q` は `Release` 節点である。`p30` の `L5` より、`un_bump` が `pending` を変えるのは
      `InBracket` を返すときだけであり、そのとき変わるのは選ばれた 1 つの要素の `outstanding`
      (`subtract` の分) と、それが空になったときのその要素の除去だけである。返る `NodeId` はその要素の
      `node` である。それが `CT` の要素 `t` であるとすると、L32 の 4 より `q` は `un_bump_releases[t]`
      に入り、`t ∈ CT` なので L32 の 3 より `q ∈ Del` である。本命題の仮定に反するので、選ばれる要素は
      `CT` の要素の由来を持たない。
    <3>4. QED
      BY <3>1, <3>2, <3>3, <ref id=8093b68/>, <ref id=7af2e2e/>
      D27 は、`un_bump` が `InBracket` でその要素を選ぶ `Release` の訪問でだけ `B_ρ` を引き、アームへの
      複製と `merge` が `p` を返り値に据えるときは `B_ρ` をそのまま運び、ほかのどの操作も `B_ρ` を
      変えないと定める。<3>1 から <3>3 より、`q` の訪問の 3 つの操作はどれも由来が `t` の要素の `B_ρ` を
      変えない。
  <2>3. QED
    BY <2>1, <2>2
<1>4. `B_ρ(n, e)[o] ≥ 1` であって `o` が活性かつ `obj_ρ(o) = O` である各名前 `o` は、`ρ` の上の
      スロットであり `obj(C_ρ(o)) = O` である。その類は `S(τ_n, O)` に入る。
  BY <ref id=eabf11d/>, <ref id=0b3e0e1/>, <ref id=4f4714c/>, p13 の DEF 名前の活性, <ref id=8093b68/>, <ref id=30d6238/>, <ref id=2ea7903/>, <ref id=b154692/>, DEF 節点の量, DEF 類ごとの義務
  `L45` より、活性な名前 `o` は `ρ` の上のスロットであって `obj_ρ(o)` はそのスロットが指す
  オブジェクトである。D33 より `o` はちょうど 1 つの別名類 `C_ρ(o)` に属し、その類の `obj(C_ρ(o))` は
  類の各スロットが指すオブジェクト、すなわち `obj_ρ(o) = O` である。L40a より `C_ρ(o)` は `τ_n` で
  開始している。
  **この段が P5 (a) を当てる対は、その前件の述語に入る。** README の P5 は「**解析が `origin` を呼ぶ
  鍵とは、`src/rc_ir/borrow.rs` の `origin(` の呼び出しがその実行で渡す `(変数, path)` の対の全体で
  ある**」と定め、「**前件を満たすことを示す段は、その対がこの述語に入ることを言う。**」と続ける。
  D27 は `B_ρ` の名前を `Retain(v, π)` 節点の `acted_references(v, π)` が付ける identity と定め、
  P6 は「**その各 leaf について解析が `origin` を呼ぶ** -- `acted_references` 自身がその呼び出しで
  ある。」と述べるので、`B_ρ` に現れる名前 `o` はその呼び出しが渡した対 `(v, λ)` の identity であり、
  述語に入る (`DEF 節点の量`)。
<1>5. 1 が成り立つ。
  BY <1>4, <ref id=4ff3e8d/>, <ref id=8093b68/>, DEF 類ごとの義務
  L43 の等式は `d(q, O)` を、`q` で pending である `CT` の要素の `B_ρ(q, ・)` を、活性かつ
  `obj_ρ(o) = O` である名前 `o` について足したものと与える。<1>4 よりその名前はどれも `S(q, O)` の
  ちょうど 1 つの類に属するので、名前ごとの和を類ごとに分けると `DEF 類ごとの義務` の `d_C(q)` の和に
  なる。
<1>6. 2 が成り立つ。
  BY <1>2, <1>3
<1>7. 3 が成り立つ。
  BY <1>6, DEF 類ごとの義務, DEF 節点の実行の素動作, <ref id=8093b68/>, <ref id=948f840/>, <1>4
  `bumps_ρ(n(q), C)` は `pending(n(q))` の**すべての**要素の `B_ρ` を `C` に属する名前に
  ついて足したものである (DEF 類ごとの義務)。<1>6 より `CT` の要素はそこに `q` の入口の点での `B_ρ` の
  まま在るので、`d_C(q)` の項はそのまま現れる。残る項は P18b より 0 以上である。
<1>8. QED
  BY <1>5, <1>6, <1>7

### L41c (節点の実行の間も余りは残る) <!--#965e588-->

**言明** --- 実行路 `ρ` を辿る活性化 `α` を固定する。`Del` に入らない `ρ` の上の節点 `q` と計数下の
オブジェクト `O` を取り、`τ` を `q` の実行の各点 (`DEF 節点の入口の点`) とする。次の 2 つが成り立つ。

1. `q` が `ρ` の終端の `Ret` でないとき、`Obl(τ, O) ≥ d(q, O)` である。
2. `d(q, O) ≥ 1` ならば `H(τ, O) ≥ d(q, O) + 1` である。

**主語は D21 の意味の活性化である。** `<1>6` が読むのは D21 が活性化に課す制限であり、`α` について
それが成り立つことは `L44` の (f) が点ごとに与える。`L41a` の脇の注も掛かる。

**証明**

<1>1. `d(q, O) ≥ 1` ならば `q` は `ρ` の終端の `Ret` ではない。`q` が `ρ` の終端の `Ret` でないとき、
      `p(q)` (`L41e`) は `ρ` の上の節点の入口の点であり、終端の `Ret` の消費より前にある。
  BY <ref id=5adaf7f/>, <ref id=4ff3e8d/>, <ref id=ca36627/>, <ref id=68112c9/>, <ref id=5c7bebf/>, DEF 節点の入口の点
  L38 の 2 より、各 `t ∈ CT` が pending である区間は `ρ` の終端の `Ret` より真に前で終わるので、終端の
  `Ret` で pending な `CT` の要素は無く、L43 の等式より `d = 0` である。D3 と L33a より終端の `Ret` は
  `ρ` の最後の節点なので、`q` がそれでなければ `q` の実行はその消費より前に終わり、`L41e` の 1 より
  `n(q)` は `ρ` の上の節点で `p(q)` はその入口の点である。
<1>2. `q` が `ρ` の終端の `Ret` でないとき、
      `Σ_{C ∈ S(q, O)} obl_ρ(τ, C) ≥ Σ_{C ∈ S(q, O)} obl_ρ(p(q), C) ≥ d(q, O)` である。
  BY <ref id=5c7bebf/>, <ref id=89569b4/>, <ref id=dd86b2c/>, <1>1
  `L41e` の 2 が第 1 の不等式を類ごとに与える。第 2 の不等式は、`L41a` の 2 を節点 `n(q)` とその入口の
  点 `p(q)` について各類に当てて
  `obl_ρ(p(q), C) ≥ bumps_ρ(n(q), C)` を得て、`L43a` の 3 で `bumps_ρ(n(q), C) ≥ d_C(q)` へ落とし、
  `L43a` の 1 でその和を `d(q, O)` にまとめたものである。`L41a` の 2 が使えるのは、<1>1 より `n(q)` が
  終端の `Ret` の消費より前の節点だからである。`L41e` の 3 より `S(q, O) ⊆ S(p(q), O)` なので、
  この各類について `held_ρ(p(q), ・)` は定まる。
<1>3. `q` が `ρ` の終端の `Ret` でないとき、
      `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) ≥ Σ_{C ∈ S(q, O)} obl_ρ(τ, C)` である。
  BY <ref id=5c7bebf/>, <ref id=89569b4/>, <1>1
  `L41e` の 3 より `S(q, O) ⊆ S(τ, O)` であり、差の各類については `L41a` の 1 が `obl_ρ(τ, C) ≥ 0` を
  与える。`L41a` の 1 が使えるのは、<1>1 より `τ` が終端の `Ret` の消費より前の点だからである。
<1>4. 1 が成り立つ。
  BY <ref id=561fd05/>, <1>2, <1>3
  `L49` より `Obl(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C)` である。
<1>5. `d(q, O) ≥ 1` のとき
      `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1] ≥ d(q, O) + 1` である。
  BY <ref id=3efe52c/>, <ref id=5c7bebf/>, <ref id=dd86b2c/>, <1>1, <1>3
  `L43a` の 1 と 3 より `Σ_{C ∈ S(q, O)} bumps_ρ(n(q), C) ≥ d(q, O) ≥ 1` なので、`L41b` を節点 `n(q)`
  とその入口の点 `p(q)`、オブジェクト `O`、部分集合 `S = S(q, O)` について読むと
  `Σ_{C ∈ S(q, O)} obl_ρ(p(q), C) + [S(p(q), O) に β(C) = 1 の類が在るならば 1] ≥ d(q, O) + 1` である。
  `L41e` の 2 で左辺の総和を `τ` の側へ移し、<1>3 で `S(τ, O)` へ広げる。角括弧は 3 つの点で同じ値で
  ある -- `L41e` の 3 より `S(q, O) ⊆ S(τ, O) ⊆ S(p(q), O)` であり、`β = 1` の類の開始の時点は
  活性化が生きている活性化になる点なので (D34 の第 3 行)、その類は `S(q, O)` に在ることと
  `S(p(q), O)` に在ることが同値である。
<1>6. `d(q, O) ≥ 1` のとき、`τ` は `α` が生きている (D23) 点であり、
      `H(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1]` である。
  BY <ref id=c232680/>, <ref id=9f1cf6c/>, <ref id=ff5985d/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=ca36627/>, <ref id=68112c9/>, <1>1, DEF 類ごとの義務, DEF 実行時の量, DEF 節点の入口の点
  **`α` が `τ` で生きていることを先に出す。** <1>1 より `d(q, O) ≥ 1` ならば `q` は `ρ` の終端の `Ret`
  ではないので、D3 と L33a より `q` は `ρ` の最後の節点ではなく、`q` の実行の点はどれも終端の `Ret` の
  消費より前にある。D23 より `α` が終わるのはその消費の時点なので、`τ` で `α` は始まって終わっておらず、
  生きている。**`τ = q_end` の場合はここに入らない** -- `q_end` は終端の `Ret` の消費を行った直後の点で
  あり、そこでは `α` は終わっている (D23) ので D21 の制限はその点について何も課さない。その `q` に
  ついては <1>1 より `d(q, O) = 0` なので、本命題の 2 は空虚に成り立つ。
  D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と
  述べる。すなわちこの不等式は仮定ではなく、D21 が活性化に課す制限であり、`DEF 実行時の量` より
  この文書の点はその段内の点である。A19 (i) は「`a` の計数下の別名類のうち
  `obj(C) = O` であり開始の時点がその時点以前であるものの全体を `S` とし、各類について
  `d(C) = held(C) - [C の ρ-終端が借用する (D14) leaf ならば 1]` と置くと、」として、引用の形で
  「`H(O) ≥ Σ_{C ∈ S} d(C) + [S に借用終端の類が在るならば 1]`」を置く。
  A19 (i) の `S` は `S(τ, O)`、`d(C)` は
  `obl_ρ(τ, C)`、角括弧は `β(C) = 1` の類が在るかである (DEF 類ごとの義務)。
<1>7. QED
  BY <1>4, <1>5, <1>6, <1>1
  1 は <1>4 である。2 は、`d(q, O) ≥ 1` のとき <1>5 と <1>6 を合わせて得られる。`d(q, O) = 0` のとき
  2 は空虚に成り立つ。

### L41d (参照を持つ類のオブジェクトはカウントを持つ) <!--#6358834-->

**言明** --- 実行路 `ρ` を辿る活性化を固定する。終端の `Ret` の消費より前の各点 `τ` と各計数下オブジェクト
`O` について、`S(τ, O)` に `held_ρ(τ, C) ≥ 1` である類 `C` が在るならば `H(τ, O) ≥ 1` である。とくに、
`ρ` の上の `Retain(v, π)` 節点または `Release(v, π)` 節点 `q` の入口の点では、`π` の下の inhabited (D16)
かつ計数下 (D26) の各 leaf `λ` について `H(q, obj(v, λ)) ≥ 1` である。

**主語は D21 の意味の活性化である。** `<1>3` が読むのは D21 が活性化に課す制限であり、`α` について
それが成り立つことは `L44` の (f) が点ごとに与える。`L41a` の脇の注も掛かる。

**証明**

<1>1. 各 `C' ∈ S(τ, O)` について `obl_ρ(τ, C') ≥ 0` である。
  BY <ref id=89569b4/>
  L41a の 1 がこれを各点について述べる。
<1>2. `Σ_{C' ∈ S(τ, O)} obl_ρ(τ, C') + [S(τ, O) に β(C') = 1 の類が在るならば 1] ≥ 1` である。
  <2>1. CASE `β(C) = 0`。DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) ≥ 1` であり、<1>1 より
        残りの項は 0 以上、角括弧も 0 以上なので、総和は 1 以上である。
    BY DEF 類ごとの義務, 本命題の仮定, <1>1
  <2>2. CASE `β(C) = 1`。`C ∈ S(τ, O)` なので角括弧は 1 であり、<1>1 より総和は 0 以上である。
    BY DEF 類ごとの義務, 本命題の仮定, <1>1
  <2>3. QED
    BY <2>1, <2>2, DEF 類ごとの義務
    DEF 類ごとの義務 より `β(C)` は 0 か 1 である。
<1>3. `H(τ, O) ≥ Σ_{C' ∈ S(τ, O)} obl_ρ(τ, C') + [S(τ, O) に β(C') = 1 の類が在るならば 1]` である。
  BY <ref id=c232680/>, <ref id=9f1cf6c/>, <ref id=ff5985d/>, DEF 類ごとの義務, DEF 実行時の量
  D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と
  述べ、`DEF 実行時の量` よりこの文書の点はその段内の点である。A19 (i) の `S` は
  `S(τ, O)`、`d(C)` は `obl_ρ(τ, C)`、角括弧は `β(C) = 1` の類が在るかである (DEF 類ごとの義務)。
  `α` は `τ` で生きている (D23)。
<1>4. 「とくに」の節が成り立つ。
  BY <ref id=9f1cf6c/>, <ref id=2ea7903/>, <ref id=596a46d/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=88a06de/>, <ref id=b3dfa37/>, <ref id=3905b4e/>, <ref id=ff5985d/>, <ref id=ca36627/>, <ref id=68112c9/>, <1>1, <1>2, <1>3
  `Retain`/`Release` 節点は継続を 1 つ持つので (D2) `ρ` の最後の節点ではなく、D3 と L33a より `ρ` の
  最後の節点は終端の `Ret` なので、その入口の点は終端の `Ret` の消費より前にあり、本命題の範囲に入る。
  `λ` は `π` の下の inhabited かつ計数下の leaf なので、`(v, λ)` は `ρ` の上のスロット (D6) であり、
  D33 よりちょうど 1 つの別名類 `C` に属して `obj(C) = obj(v, λ)` である。**`C` が `q` の入口の点で
  開始していることは L40a の主文が与える** -- `v` は `q` の入口の点までに値を得ている (A11 より `v` の
  使用はその位置でスコープに入っている束縛に解決し、D2 のスコープの規則よりその束縛は `q` の祖先で
  あるかパラメータ・capture であり、D3 より実行路は祖先を先に通り、パラメータ・capture は活性化が
  始まる時点で値を持つ (D23)) ので、`C` は `τ` までに値を得た変数のスロットを含む。よって
  `C ∈ S(q, obj(v, λ))` である。A19 の (ii-a) は「各時点と各計数下の別名類に
  ついて、その類が持つ参照の個数は非負であり、読む構文と `Retain`/`Release` がその類を名指す時点では
  1 以上である。」と述べるので、`held_ρ(q, C) ≥ 1` である。よって言明の仮定が満たされ、<1>1 から <1>3 が
  `H(q, obj(v, λ)) ≥ 1` を与える。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L43b (対応する 2 つの活性化の別名類) <!--#0c081dc-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`α` の点 `τ` (`DEF 実行時の量`) を固定する。`τ` までに
2 つの活性化が `Del` の節点を除いて同じ節点を実行し、その点までに値を得ている各変数の値が D29 の全単射の
もとで対応しているとする。このとき次の 4 つが成り立つ。

1. `B` から作られる `VarTable` と `B'` から作られる `VarTable` は、`var_tys`・`param_tys`・
   `closure_targets` が等しく、`bindings` は鍵の集合と各鍵の `Binding` の変位・`RcVar`・添字・変位番号・
   型が等しい (`Binding::Llvm` が運ぶ op は同じ原本の複製であって同じオブジェクトではない)。
   **対応する 2 つの `Binding::Llvm` が運ぶ op は、同じ引数に対して同じ `Provenance` を返し、
   `borrows_operand` にも同じ値を返す。**
   鍵が等しい
   2 つの `origin` の答えは等しい。さらに `B` を本体に持つ関数と `B'` を本体に持つ関数の `params`・
   `capture`・`borrowed_units` は等しい。
2. `ρ` の上の位置 (D6) と `ρ'` の上の位置は、変数と leaf を同じくする対応で 1 対 1 に対応し、その対応は
   別名類 (D33) の 1 対 1 の対応を導く。対応する 2 つの類は `β` (DEF 類ごとの義務) が等しく、`τ` までに
   開始しているか (D34) が一致し、計数下であるか (D26) が一致する。計数下の類については `obj` が D29 の
   全単射で対応する。とくに各計数下オブジェクト `O` について `S(τ, O)` と `S'(τ, O)` はこの対応で
   移り合う。
3. 各計数下オブジェクト `O` について
   `Σ_{C ∈ S(τ, O)} held_ρ(τ, C) = Σ_{C' ∈ S'(τ, O)} held_{ρ'}(τ, C') + d(τ, O)` である。
4. D29 の全単射が対応させる 2 つのオブジェクトは、同じ Fix の型を持つ。

**証明**

<1>1. 1 が成り立つ。
  <2>1. L30 と L32 の 5 より、`B'` は `B` から `Retain` 節点と `Release` 節点をいくつか取り除いた木で
        あり、残る各位置の式の変位・変数・path・`RcState`・`Match` のアームの本数と並び・継続の順序は
        変わらない。
    BY <ref id=4a0c14c/>, <ref id=7d5a1de/>
  <2>2. `collect_bindings` は `RcExpr::Retain` と `RcExpr::Release` の腕で継続へ降りるだけであり、
        `bindings`・`var_tys`・`closure_targets` に何も入れない。`returned_var` も同じ 2 つの腕で継続へ
        降りる。よって <2>1 の木の変形は `var_tys` と `closure_targets` を変えず、`bindings` については
        鍵の集合と、各鍵の `Binding` の変位・`RcVar`・添字・変位番号・型を変えない。**`Binding::Llvm` が
        運ぶ op はこの水準では等しいと言えない** -- 2 つの表の op は複製で隔たっているので、この段は
        それについて何も述べない。
    BY CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var,
       CODE src/rc_ir/ownership.rs: Binding, <2>1
  <2>2a. `B` と `B'` はどちらも A6 と A11 を満たし、その `VarTable` は `VarTable::of` か
         `VarTable::body_only` が作った表である。すなわち P2a を読む条件が両側で満たされる。
    BY <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=63eadd9/>, <ref id=4a0c14c/>, <ref id=7d5a1de/>, <ref id=b3dfa37/>, CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only, CODE src/rc_ir/borrow.rs: cancel
    `B` は `borrow_ify` の出力の本体である (第 1 節)。A6 は「出力についての同じ性質は仮定ではなく P9 が
    示す」と述べ、A11 は「**この仮定が語るのは `borrow_ify` の入力である。** 出力についての同じ性質は、
    この仮定と P9 から出る」と述べるので、`B` について A6 と A11 は P9 と合わせて読む。`B'` は L30 と
    L32 の 5 より `B` から `Retain`/`Release` 節点をいくつか取り除いた木であり、D2 よりこの 2 種は変数を
    束縛しないので、束縛名の集合も束縛と使用の対応も `B` のものである。よって `B'` も A6 と A11 を
    満たす。表を作るのは `VarTable::of` (関数の本体) か `VarTable::body_only` (グローバル初期化子の
    `init`) であり、`cancel` はその 2 つしか呼ばない。
  <2>3. `cancel` は `prog.funcs.values()` の各 `f` について `f.clone()` を作って `clone.body` にだけ
        書き込み、鍵に `f.name.clone()` を据えるので、`params`・`capture`・`borrowed_units` は
        変わらない。グローバル初期化子はパラメータも capture も持たない (D1)。`VarTable::of` は
        パラメータと capture について `Binding::Param` と `param_tys`・`var_tys` を置き、残りを
        `collect_bindings` から取る。`VarTable::body_only` は `collect_bindings` だけを取る。
        `RcFunc` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より `f.clone()` の各欄は `f` の
        対応する欄の `clone` である。
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc,
       CODE src/rc_ir/ownership.rs: VarTable::of,
       CODE src/rc_ir/ownership.rs: VarTable::body_only, <ref id=a502f3e/>, EXT derive(Clone)
  <2>3a. 2 つの表の対応する `Binding::Llvm` が運ぶ op は、同じ引数に対して同じ `Provenance` を返し、
         `borrows_operand` にも同じ値を返す。
    <3>1. `B'` の `RcRhs::Llvm` が持つ op は `B` のものの複製であり、同じオブジェクトであるとは限らない。
          `drop_nodes_inner` の右辺が `Match` でない `Let` の腕は
          `RcExpr::Let(x.clone(), rhs.clone(), drop_nodes(k, to_delete))` を積む。`RcRhs` は
          `#[derive(Clone)]` を持つので、EXT derive(Clone) より `rhs.clone()` は `RcRhs::Llvm` の第 1 欄に
          `Box<dyn LLVMGen>` の `Clone::clone` の値を置く。`collect_bindings` は
          さらに `llvm_gen.clone()` を `Binding::Llvm` の第 1 欄に置くので、2 つの表の op は 2 段の複製で
          隔たっている。**その複製が原本と同じオブジェクトであるとは限らない** -- `LLVMGen` の宣言は
          `pub trait LLVMGen: DynClone + Send + Sync` であって `dyn_clone` の `DynClone` を継承するので、
          EXT dyn_clone の trait object の複製 がその `Box<dyn LLVMGen>` の `Clone` を述べる。
          **この段が与えるのは複製の関係だけであり、<3>2 が読むのもそれである。**
      BY CODE src/rc_ir/borrow.rs: drop_nodes_inner, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/ast/inline_llvm.rs: LLVMGen, CODE src/rc_ir/ownership.rs: collect_bindings,
         EXT derive(Clone), EXT dyn_clone の trait object の複製
    <3>2. QED
      BY <3>1, <ref id=e11772a/>
      A3 は「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である** -- 同じ引数に対して常に同じ値を
      返す」と述べ、続けて「**この 2 節を合わせると「op の複製は原本と同じ宣言を返す」が出る。**
      `rhs.clone()` や `fresh_rename_function` が作る複製の op は、原本と同じ引数を渡されれば同じ
      `Provenance` を返す」と述べる。A3 はこの 1 文を要る段の見分け方を「複製された op の宣言を原本の
      ものと同じだと読む段が、それである」と書いており、<3>1 よりこの段がそれである。決定性は
      `borrows_operand` についても同じ文が述べ、`FullName` の欄を読まないことも A3 が
      「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は自分の `FullName` の欄を読まない。**」として述べる。
  <2>3b. 鍵 `(x, π)` が等しい 2 つの表の `origin` の答えは等しい。
    <3>0. 各表の `origin` の答えは、`origin_inner` の再帰呼び出しを memo を使わずに展開して得られる
          有限の木 `E(x, π)` (`p13` の `L8a`) の値であり、その木の高さについての帰納法は整礎である。
          以下の帰納はその高さで回す。
      BY <ref id=0edb0ba/>, <ref id=b1f6e13/>, <ref id=df74e59/>, CODE src/rc_ir/ownership.rs: origin, <2>2a
      P2a より、1 つの表を固定すれば `origin` の答えは `vars.origins` の memo の状態に依らないので、
      各表の `origin` は鍵の関数である。**P2a はその `vars` を限る** -- 「**`vars` は、A6 と A11 を
      満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が作った表である。**」と
      述べ、「**この制限は言明の一部であって、読む段が自分で補うものではない。**」と続ける。その制限が
      両側で満たされることは <2>2a が与える。**表を跨ぐ形は P2a の主張ではない** -- P2a は
      「**表を跨ぐ形はこの命題の主張ではない。** `bindings` が等しい相異なる 2 つの `VarTable` に
      ついて答えが等しいことは別の主張であり、それを要る段は自分で示す。」と述べ、その形を要る段を
      「相異なる 2 つの `VarTable` の値について `origin` の答えを比べる段が、それである」と見分ける。
      この `<2>3b` がその段であり、以下の帰納がそれを示す。
      **帰納を回すのは実際の呼び出しの木ではない。** `origin` は答えを**計算した後に** memo へ
      書き込むので、1 つの評価の中でも鍵が重なれば 2 度目は memo に当たり、実際の呼び出しの木は
      そこで止まる (`CODE src/rc_ir/ownership.rs: origin`)。`p13` の `L8a` がその差を埋める --
      (i) は「`origin(vars, type_env, x.name, λ)` が返す値は、`origin_inner` の再帰呼び出しを memo を
      使わずにそのまま展開して得られる木 … `E(x.name, λ)` … の値に等しい」、(ii) は「その計算を、
      memo を使わずに `origin_inner` の再帰呼び出しをそのまま展開して得られる木は有限である」と
      述べる。`L8a` は `vars` を「プログラムのいずれかの本体 (D23) の `VarTable`」に取り、その証明は
      A11 の下の P2 に立つので、<2>2a より 2 つの表のどちらについても読める。よって 2 つの答えを
      比べるのに `E` の値を比べればよく、`E` は有限の木なのでその高さについての帰納法は整礎である。
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
          `args` と `result_ty` は 2 つの表で等しく、`DEF 本体ごとの記法` より `type_env` は 1 つなので、
          <2>3a より `decl` は 2 つで等しい。`as_arg_projection` は `decl` と `path` だけを読む。
          `origin_from_leaves_under` は `decl`・`args`・`path`・`type_env` から `operand_units` と
          `produced_here` を作り、`operand_units` の各元について `origin(vars, type_env, args[j].name,
          unit)` を呼び、その答えから返り値を組む。呼び出しの引数は 2 つの表で等しいので、帰納法の仮定
          よりその答えも等しく、返り値も等しい。
      BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection,
         CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
         CODE src/rc_ir/ownership.rs: truncate_to_unit, <2>2, <2>3a, DEF 本体ごとの記法, 帰納法の仮定
    <3>5. QED
      BY <3>0, <3>1, <3>2, <3>3, <3>4, CODE src/rc_ir/ownership.rs: origin_inner,
         CODE src/rc_ir/ownership.rs: Binding
      `origin_inner` の `match` は `bindings.get(var)` の値について `None | Param | Producer`、`Move`、
      `Join`、`Llvm`、`Field`、`Payload` の 6 つの腕を持ち、`Field` と `Payload` は `is_box` で
      さらに分かれる。<3>2、<3>3、<3>4 がこれを尽くす。**<3>3 と <3>4 が引く帰納法の仮定は、
      `E(x, π)` の部分木についてのものである** -- `p13` の `L8a` の展開の定め方より、`E(x, π)` の
      部分木は `origin_inner(x, π)` が `origin` に渡す各鍵 `b` の `E(b)` であり、その高さは
      `E(x, π)` の高さより小さい (<3>0)。
  <2>4. QED
    BY <2>2, <2>3, <2>3a, <2>3b, DEF 本体ごとの記法, CODE src/rc_ir/borrow.rs: cancel
    `DEF 本体ごとの記法` より `type_env` はプログラムの `TypeEnv` であり、`cancel` は受け取った `type_env` を
    そのまま `CancelAnalysis` と `all_owned_units` に渡して型を作らないので、2 つの本体の `origin` は
    同じ `type_env` の下で読む。**P2a は 1 つの `VarTable` の値を固定した形の主張であり、相異なる
    2 つの表について答えを比べる形はその主張ではない** -- その形を <2>3b が、`p13` の `L8a` の
    memo を使わない展開の高さについての帰納で示す。
<1>2. 2 が成り立つ。
  <2>1. 位置 (D6) が対応する。D6 より位置は対 `(x, λ)` であり、`x` は値を得た変数か束縛を持たない名前、
        `λ` は `ty(x)` の inhabited (D16) な boxed leaf である。<1>1 より 2 つの `VarTable` の
        `var_tys` は等しく、A12 より束縛を持たない名前の型はその記号の型なので、`ty(x)` は 2 つで同じで
        あり、`boxed_leaf_paths(ty(x))` も同じである (D4)。D16 の inhabited は値が通る unbox union の
        タグで決まる。本命題の仮定より `τ` までに値を得ている変数は 2 つの活性化で同じであり、その値は
        D29 の全単射のもとで対応する。D29 の第 5 行より、対応する 2 つの値のスカラの成分 -- unbox union
        のタグを含む -- は等しく、inhabited な各 boxed leaf は全単射で対応するオブジェクトを指す。
        束縛を持たない名前 (記号の位置) は 2 つの本体で同じであり、そこが指すのは funptr かグローバル
        状態のオブジェクトである (D6)。
    BY <ref id=596a46d/>, <ref id=0594f24/>, <ref id=66c9670/>, <ref id=7218f92/>, <ref id=83d98e9/>, <1>1, 本命題の仮定
  <2>2. 別名の辺 (D20) と ρ-歩み・ρ-終端 (D33) が対応する。D33 の歩みは各位置で `origin_inner` が呼ぶ
        `origin` の引数へ進み、`origin` を呼ばない位置で止まる。<1>1 より `origin` の答えは 2 つの本体で
        等しく、<2>1 より位置が対応する。`Binding::Join` の辺はその活性化が選んだアームの結果へ進む
        (D17) が、本命題の仮定より 2 つの活性化は `τ` までに同じ `Match` の節点を実行しており、
        L30 と L31 よりその節点は同じ位置にあるので、選んだアームも同じである。
    BY <ref id=9c7c27a/>, <ref id=30d6238/>, <ref id=d59f90b/>, <ref id=4a0c14c/>, <ref id=2bb344b/>, <1>1, <2>1, 本命題の仮定
  <2>3. 別名類が対応し、計数下の類の `obj` が対応する。D33 は「1 つの実行路 `ρ` の上のスロット (D6) を、
        `ρ` 終端が等しいという関係で分けた同値類を**別名類**と呼ぶ」と定め、`obj(C)` を類の各スロットが
        指すオブジェクトとする。**主語はスロットであって位置ではない** -- 記号の位置はどの別名類にも
        属さない。<2>1 は位置の対応を与え、そのうちどれがスロットであるかも 2 つで一致する -- D6 より
        位置がスロットであるのは第 1 成分が値を得た変数であるときであり、<1>1 より 2 つの表の
        `bindings` の鍵の集合は等しく、<2>1 より `τ` までに値を得ている変数も 2 つで同じだからである。
        <2>2 より ρ-終端も対応するので、類も対応する。**ρ-終端が記号の位置である類のオブジェクトは
        グローバル状態であって計数下ではない** -- D33 は「**歩みは記号の位置で終わりうる。** その類の
        スロットが指すのはグローバル状態のオブジェクトであって計数下ではないので (D26)、参照を数える節は
        その類に掛からない。」と述べる。それ以外の類については、<2>1 より
        対応する 2 つのスロットが指すオブジェクトが D29 の全単射で対応するので `obj` が対応し、D29 の
        第 5 行より計数下かグローバル状態か (D26) の区別も対応する 2 つのオブジェクトで一致する。
    BY <ref id=30d6238/>, <ref id=88a06de/>, <ref id=7218f92/>, <ref id=596a46d/>, <1>1, <2>1, <2>2
  <2>4. `β` が等しい。DEF 類ごとの義務 の `β(C)` は、`C` の ρ-終端が借用する (D14) パラメータ・capture の
        leaf であるとき 1、そうでないとき 0 である。D14 の借用は `RcFunc::borrowed_units` が定め、
        <1>1 よりそれは 2 つで等しい。<2>2 より ρ-終端の変数と leaf も 2 つで同じである。
    BY DEF 類ごとの義務, <ref id=ef8efc4/>, <1>1, <2>2
  <2>5. `τ` までに開始しているかが一致する。D34 の開始の時点は、ρ-終端 `(u, σ)` の変数 `u` が値を得る
        時点である -- `u` がパラメータ・capture なら活性化が始まる時点、そうでなければ `u` を束縛する
        節点を実行する段の直後である。L32 の 5 より `Del` の要素は `Retain` 節点か `Release` 節点だけで
        あり、D2 よりこの 2 つは変数を束縛しない。本命題の仮定より `τ` までに値を得ている変数は 2 つの
        活性化で同じなので、`τ` までに開始している類も対応する。
    BY <ref id=9d5d254/>, <ref id=7d5a1de/>, <ref id=b3dfa37/>, <2>2, 本命題の仮定
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, DEF 類ごとの義務
    DEF 類ごとの義務 より `S(τ, O)` は、`obj(C) = O` であり `τ` までに開始した計数下の別名類の全体で
    ある。<2>3 と <2>5 がこの 3 つの条件を対応させる。
<1>3. 3 が成り立つ。
  <2>1. D34 の表の 6 行のうち、`Retain` の行と `Release` の行を除く 4 行 -- 生成、所有する初期値、
        借用する初期値、消費 -- が 2 つの活性化で起こす増減は等しい。
    BY <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=9d74736/>, <ref id=ef8efc4/>, <ref id=7d5a1de/>, <1>1, <1>2, <ref id=e11772a/>, <ref id=ff5985d/>, <ref id=7218f92/>, <ref id=1dbc8d1/>, 本命題の仮定
    生成の行と消費の行が名指す leaf は、D10 の生成の表と D9 の消費の表が定めるとおり、節点の形と、
    それが名指す変数の値と、`App` については呼び出し先の所有 (D14) で決まる。**`Llvm` の節点については
    その 2 つの行が `result_prov` と `borrows_operand` を読むので、2 つの本体の op が同じ宣言を返すことが
    要る** -- `B'` の op は `B` の op の複製であって同じオブジェクトではない。それを与えるのは <1>1 の
    第 2 文であり、その支えは A3 の「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である** -- 同じ引数に
    対して常に同じ値を返す」と「**この 2 節を合わせると「op の複製は原本と同じ宣言を返す」が出る。**」で
    ある。**`App` の節点については、2 つの活性化の呼び出し先が対応する版である。** D23 より呼び出し先は
    その段で `callee` の値が指す関数であり、本命題の仮定より `callee` の値は 2 つで対応する。D29 の
    第 5 行は「**funptr の番地は等号では読めない** -- 2 つのプログラムは別々にコンパイルされるので、
    同じ関数の番地が同じ値になるとは限らない。」に続けて「**その成分については、対応する 2 つの番地が
    対応する版を名指す**」と述べ、さらに「**スカラの成分についてのこの読みは、オブジェクトが保持する値に
    限らない。** … 呼び出し先を決める値も、記憶域へ格納する値もである。」と広げる。`L44d` より、対応する
    2 つの版は同じ `name` を持ち、その `params` と `borrowed_units` は等しい。よって D9 の `App` の行が
    名指す leaf は 2 つで同じである。L32 の 5 より `Del` の
    要素は `Retain` 節点と `Release` 節点だけなので、この 2 つの行を起こす節点はどちらの活性化も同じ
    だけ実行している (本命題の仮定)。値が対応することは本命題の仮定が与える。初期値の 2 行は
    パラメータ・capture の leaf についてであり、<1>1 より `params` と
    `capture` は等しい。増減が掛かる類が対応することは <1>2 である。
  <2>2. `Retain` の行と `Release` の行について、`α` の側が `α'` の側より多く数えるのは、`τ` までに
        `α` が実行した `Del` の `Retain`/`Release` 節点の**素動作**の分だけである。
    BY <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=7d5a1de/>, <ref id=9d5d254/>, 本命題の仮定, DEF 節点の実行の素動作
    L30 と L31 より `ρ'` は `ρ` から `Del` の節点を除いた列であり、本命題の仮定より 2 つの活性化は
    それ以外の節点を同じだけ実行している。**素動作の粒度で数えるのは、`τ` が `Del` の節点の実行の
    途中の点でありうるからである** -- D34 は第 4 行と第 5 行の事象を「**それを運ぶ素動作の直後の
    段内の点**」で `held` に映し、`DEF 節点の実行の素動作` の第 1 の箇条は 1 つの行が複数の leaf を
    名指すときその素動作を leaf ごとと定める。
  <2>3. `Del` の `Retain(v, π)` 節点の 1 つの leaf についての素動作が、`obj(C) = O` である計数下の類の
        `held_ρ` の総和に与える増分は、その素動作が `O` への参照を作った個数に等しい。`Del` の
        `Release(v, π)` 節点の 1 つの leaf についての素動作は、その素動作が `O` への参照を処分した
        個数だけその総和を下げる。
    BY <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=596a46d/>, <ref id=30d6238/>, <ref id=88a06de/>, <ref id=2ea7903/>, DEF 節点の実行の素動作
    **`Retain` の側。** D34 の第 4 行は「`Retain(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に
    持つ」類の `held_ρ` をその `λ` 1 つにつき 1 上げる。D6 より `(v, λ)` が位置であるのは `λ` が
    inhabited なときであり、D10 の `Retain` の行は `π` の下の inhabited な各 leaf `λ` につき
    `obj(v, λ)` への参照を 1 つ作る。D33 より `(v, λ)` が属する類 `C` の `obj(C)` は `obj(v, λ)` で
    ある。D26 より参照を持つのは計数下のオブジェクトを指す leaf だけであり、D34 は計数下の類に
    ついてだけ `held_ρ` を定める。L40a の 2 よりその類はその素動作の直後の点で開始しているので
    `S(τ, O)` に入る。よって leaf `λ` の素動作は `obj(v, λ) = O` のときに限り総和を 1 上げ、その
    素動作が `O` へ作る参照もちょうど 1 つである。
    **`Release` の側。** D34 の第 5 行は「`Release(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の
    下に持つ」類の `held_ρ` をその `λ` 1 つにつき 1 下げ、D10 の `Release` の行は `π` の下の
    inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ取り除く。`λ` から類 `C` への対応、
    `obj(C) = obj(v, λ)` であること、計数下への制限、`S(τ, O)` に入ることは `Retain` の側と同じ 4 つの
    定義 (D6、D33、D26、L40a の 2) が与える。よって leaf `λ` の素動作は `obj(v, λ) = O` のときに限り
    総和を 1 下げ、その素動作が `O` の参照を処分する個数もちょうど 1 つである。
  <2>4. QED
    BY <2>1, <2>2, <2>3, <1>2, DEF 欠損
    DEF 欠損 は `d(τ, O)` を「`τ` より前に実行された `Del` の `Retain` 節点の素動作が `O` への参照を
    作った個数」から「`τ` より前に実行された `Del` の `Release` 節点の素動作が `O` への参照を処分した
    個数」を引いたものと定める。<2>3 をその全素動作について足し、<2>1 と <2>2 と合わせると、2 つの総和の
    差はちょうど `d(τ, O)` である。総和が渡る 2 つの類の集合が対応することは <1>2 である。
<1>4. 4 が成り立つ。
  <2>1. D29 の全単射が対応させるオブジェクトを生成するのは、D29 の第 1 行 (入力の束縛が受け取る値)、
        第 3 行 (割り当ての位置)、第 4 行 (子の活性化を作る段の結果)、第 5 行 (対応するオブジェクトが
        保持する値) である。第 2 行は参照カウントの推移についてであり、オブジェクトを対応させない。
    BY <ref id=7218f92/>, DEF 対応する活性化の量
  <2>2. 2 つの活性化の対応する位置 (D6) は、同じ Fix の型の値を持つ。
    BY <1>1, <ref id=1dbc8d1/>, <ref id=596a46d/>, <ref id=83d98e9/>, <ref id=746e87a/>, <1>2
    スロットの第 1 成分が節点の束縛する変数であるときは、その型は `VarTable` の `var_tys` が持つ値で
    あり、<1>1 よりそれは 2 つの表で等しい。パラメータ・capture であるときは、その型は
    `params`・`capture` の型であり、`L44d` よりそれは 2 つの本体を持つ関数で等しい。記号の位置で
    あるときは、A12 より束縛を持たない `RcVar` の型はその名前の記号の型であり、P24 の第 3 の箇条より
    2 つのプログラムの第 `i` のグローバル初期化子の `symbol` と `ty` は等しく、関数の名前については
    `L44d` より 2 つの版の `fn_ty` の元である `params`・`capture` が等しい。<1>2 より 2 つの位置は
    対応する。
  <2>3. QED
    BY <2>1, <2>2, <ref id=0594f24/>, <ref id=7218f92/>, <ref id=0b850c9/>, <ref id=f06144e/>, <1>2
    D25 の到達の関係についての帰納で示す。**基底。** 対応する 2 つの位置 (D6) が指すオブジェクトは、
    <2>2 よりその位置の値の型と leaf `λ` で型が決まり (D4)、2 つで同じである。D29 の第 1 行・第 3 行・
    第 4 行が対応させるオブジェクトはどれもこの形である -- 入力の束縛が受け取る値も、割り当ての位置が
    置く値も、子の活性化が返す値も、その節点が束縛する変数かパラメータ・capture の値の leaf として
    現れる (D10 の生成の表、D23)。**帰納段。** 対応する 2 つのオブジェクトが同じ型を持つとき、その
    型の各 boxed leaf の型も同じであり (D4)、D29 の第 5 行はその leaf が指す 2 つのオブジェクトを
    対応させる。よって全単射の定義域 -- 2 つの活性化がそれぞれ到達できる (D25) オブジェクトの全体 --
    の全体でこの性質が成り立つ。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L43c (対応する入力の活性化は D21 の制限を満たす) <!--#c6ed0c5-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と `α` の点 `τ` (`DEF 実行時の量`) を固定する。L43b の
仮定が `τ` について成り立ち、`α'` が `τ` に対応する点で生きており (D23)、かつ各計数下オブジェクト `O` に
ついて `H(τ, O) = H'(τ, O) + d(τ, O)` であるとする。このとき `α` は `τ` で D21 の制限 --
A19 (i) の不等式 -- を満たす。

**証明**

<1>1. `α'` は `τ` に対応する点で A19 (i) の不等式を満たす。すなわち各計数下オブジェクト `O` に
      ついて
      `H'(τ, O) ≥ Σ_{C' ∈ S'(τ, O)} obl_{ρ'}(τ, C') + [S'(τ, O) に β(C') = 1 の類が在るならば 1]`
      である。
  BY <ref id=c232680/>, <ref id=9f1cf6c/>, <ref id=ff5985d/>, <ref id=7218f92/>, DEF 類ごとの義務, DEF 対応する活性化の量, DEF 実行時の量
  D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と述べ、
  `DEF 実行時の量` よりこの文書の点はその段内の点である。`α'` は `B'` の
  活性化であり (D29)、`DEF 対応する活性化の量` の点の対応より `τ` に対応する `α'` の点が定まり、
  本命題の仮定
  より `α'` はそこで生きている (D23)。DEF 類ごとの義務 より、A19 (i) の `S` は `S'(τ, O)`、`d(C)` は
  `obl_{ρ'}(τ, C')`、角括弧は `β(C') = 1` の類が在るかである。
<1>2. 角括弧の値は 2 つの活性化で等しく、`Σ_{C ∈ S(τ, O)} β(C) = Σ_{C' ∈ S'(τ, O)} β(C')` である。
  BY <ref id=0c081dc/>, 本命題の仮定
  L43b の 2 より `S(τ, O)` と `S'(τ, O)` は 1 対 1 に対応し、対応する類の `β` は等しい。
<1>3. `Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) = Σ_{C' ∈ S'(τ, O)} obl_{ρ'}(τ, C') + d(τ, O)` である。
  BY <ref id=0c081dc/>, <1>2, DEF 類ごとの義務, 本命題の仮定
  DEF 類ごとの義務 より `obl_ρ(τ, C) = held_ρ(τ, C) - β(C)` である。L43b の 3 が `held_ρ` の総和の差を
  `d(τ, O)` と与え、<1>2 が `β` の総和を打ち消す。
<1>4. QED
  BY <1>1, <1>2, <1>3, 本命題の仮定
  本命題の仮定と <1>1 と <1>3 より
  `H(τ, O) = H'(τ, O) + d(τ, O) ≥ Σ_{C'} obl_{ρ'}(τ, C') + [角括弧] + d(τ, O) = Σ_{C} obl_ρ(τ, C) + [角括弧]`
  である。<1>2 より角括弧は 2 つで同じ値である。これが `α` についての A19 (i) の不等式である。

### L44d (`cancel` の出力の関数の欄は入力のものである) <!--#1dbc8d1-->

**言明** --- `cancel` の出力の各関数の `name`・`params`・`capture`・`borrowed_units` は、入力の同じ名前の
関数のものに等しい。とくに `B'` の所有と借用の割り当て (D14) は `B` のものと同じである。

**証明**

<1>1. QED
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, <ref id=ef8efc4/>, <ref id=a502f3e/>, EXT derive(Clone)
  `cancel` は `prog.funcs.values()` の各 `f` について `f.clone()` を作って `clone.body` にだけ書き込み、
  鍵に `f.name.clone()` を据えるので、この 4 つは変わらない -- `RcFunc` は `#[derive(Clone)]` を持つので、
  EXT derive(Clone) より `f.clone()` の各欄は `f` の対応する欄の `clone` である。グローバル初期化子は
  パラメータも capture も持たない (D1)。D14 の所有と借用の割り当ては `RcFunc::borrowed_units` が定めるので、
  それも変わらない。

### L44e (`Del` の節点は終端の `Ret` より前にある) <!--#18416e9-->

**言明** --- 対応する活性化 `α`、`α'` (D29) を固定する。`Del` の節点はどれも `ρ` の終端の `Ret` より
真に前にある。よって、`ρ` の最後の点 `q_end` (`DEF 節点の入口の点`) を除く `α` の各点について、それに
対応する `α'` の点で `α'` は生きている (D23)。`q_end` では 2 つの活性化はどちらも終わっており、D21 の
制限はその点について何も課さない。

**証明**

<1>1. QED
  BY <ref id=5adaf7f/>, <ref id=23ac734/>, <ref id=7d5a1de/>, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=ff5985d/>, <ref id=c232680/>, <ref id=24bf090/>, <ref id=9d74736/>, DEF 節点の入口の点, DEF 対応する活性化の量, DEF 実行時の量
  L32 の 5 より `Del` の要素は `Retain` 節点か `Release` 節点だけである。**`Del` の `Retain` 節点は
  `CT` の要素である** -- L32 の 3 より `Del` は `CT` と `⋃_{t ∈ CT} un_bump_releases[t]` の非交和であり、
  L32 の 2 より後者の要素は `Release` 節点なので、P15 の前半 (相異なる位置は相異なる `NodeId` を持つ) と
  合わせて、`Retain` 節点である `Del` の要素は `CT` の側に在る。L38 より `t ∈ CT` が pending である区間
  `I_ρ(t)` は
  `ρ` の上で `t` の直後から始まり、その最後の節点 `n*(ρ)` は終端の `Ret` より真に前にある (L38 の 2)。
  `ρ` の上にある `Del` の `Release` 節点は、L32 の 3 と L37 よりある `t ∈ CT` について `R_ρ(t)` の要素で
  あり、L38 の 3 より `R_ρ(t) ⊆ I_ρ(t)` なので `n*(ρ)` 以前にある。よってどの `Del` の節点も終端の
  `Ret` より真に前にある。L30 と L31 より `ρ'` の終端の `Ret` は `ρ` の終端の `Ret` と同じ節点であり、
  `α'` はその消費を行った時点で終わる (D23)。**点の対応が `q_end` より前の点を `α'` が終わる前の点へ
  写すのは、この 2 つからである** -- `DEF 対応する活性化の量` の点の対応は、対応する節点の実行が起こす
  素動作を順に突き合わせて作る。`q_end` より前の `α` の点は、`ρ` の終端の `Ret` の消費より前の素動作の
  切れ目であり、`Del` の節点の実行が作る点を除けばその相手は `ρ'` の同じ節点の同じ素動作の直後の点で
  ある。`ρ'` の終端の `Ret` の消費はその後にあるので、その点で `α'` はまだ終わっていない。`Del` の
  節点の実行が作る点については、`DEF 対応する活性化の量` が `ρ'` の上で `q` の直後にあたる節点の入口の
  点を相手に取り、その節点は終端の `Ret` かそれより前にあるので、やはり `α'` は終わっていない。
  **`q_end` では 2 つの活性化はどちらも終わっている。** D23 は「活性化 `a` が**終わる**とは、`a` の位置が
  `B(a)` の終端の `Ret` に着き、その `Ret` の消費 (D9) を行うことをいう」と定め、`DEF 節点の入口の点` は
  `q_end` を「その `Ret` の消費 (D9) を行った直後の点」と定めるので、`α` は `q_end` で終わっている。
  `ρ'` の終端の `Ret` は `ρ` のそれと同じ節点であり (L30、L31)、`DEF 対応する活性化の量` の点の対応が
  `q_end` に写す点はその消費の直後なので、`α'` もそこで終わっている。**D21 の制限はその点について何も
  課さない** -- `DEF 実行時の量` は「**活性化 `α` の点**とは、`α` が生きている (D23) 間の各点である」と
  定めるので、`q_end` は `α` の点でも `α'` の点でもなく、D21 が「その各時点と各段内の点 (D24) で
  A19 (i) の不等式を満たすものに限る」と課す条件はそこに掛からない。

### L44p (節点の入口の点での帰結) <!--#efe22a7-->

**言明** --- 対応する活性化 `α`、`α'` (D29) を固定し、`p` を `ρ` の上の節点 `q` の入口の点、または
`ρ` の最後の点 `q_end` (`DEF 節点の入口の点`) とする。`p` で `DEF 対応の 3 つ` の (a) と (b) が成り立ち、
**`p` より前の `α` の各点で `α` が D21 の制限 -- A19 (i) の不等式 -- を満たす**とすると、次の 2 つが
成り立つ。

1. `α` は `p` で D21 の制限 -- A19 (i) の不等式 -- を満たす。
2. 各計数下オブジェクト `O` について、`d(p, O) ≥ 1` ならば `H(p, O) ≥ d(p, O) + 1` である。

**この命題は `α` が D21 の意味の活性化であることを仮定しない。** 仮定するのは `p` より**前**の各点での
制限だけであり、`p` 自身については 1 がそれを示す。立つのは `α'` の側の制限である (`L43c`)。
**2 が `L41` を読むのに要る `α` の側の制限は、README 第 2 節の接頭の規則により `p` までの各点のもので
ある** -- `p` より前は本命題の仮定が、`p` 自身は 1 が与える。

**証明**

<1>1. 1 が成り立つ。
  BY <ref id=c6ed0c5/>, 本命題の仮定, <ref id=18416e9/>
  `p` が `q_end` であるとき、`L44e` より D21 の制限はその点について何も課さず、言明は空虚に成り立つ。
  それ以外の点では `L44e` より `α'` は対応する点で生きている。L43b の仮定 -- `p` までに 2 つの活性化が
  `Del` の節点を除いて同じ節点を実行し、その点までに値を得ている各変数の値が対応すること -- は
  本命題の仮定の (a) が与え、`H(p, O) = H'(p, O) + d(p, O)` は本命題の仮定の (b) が与える。
<1>2. 2 が成り立つ。
  <2>1. CASE `p` が `ρ` の上の節点 `q` の入口の点である。`d(q, O) ≥ 1` とする。L43 より
        `N(q, O) ≥ d(q, O) ≥ 1` である。L41 の 2 より `H(q, O) ≥ N(q, O) + 1 ≥ d(q, O) + 1` である。
        **L41 の 2 を `α` について読むのに要る D21 の制限は、`q` の入口の点までの各点のものである** --
        README の第 2 節は「**各時点についての言明は、その時点までの接頭で読む。**「各実行路と、それを
        辿る各活性化の各時点について …」の形の言明 -- A19 (ii-a)、A19 (ii-b)、P14a、P18a、P18b、P18c、
        D11 の 3 つの節 -- は、時点 `τ` について `τ` までの量だけを結ぶ。」と定め、`L41` の 2 が読むのは
        その一覧に在る P18a である。`q` の入口の点より前の各点については本命題の仮定が、`q` の入口の点
        自身については <1>1 がその制限を与える。
    BY <ref id=217aebd/>, <ref id=4ff3e8d/>, <1>1, 本命題の仮定
  <2>2. CASE `p` が `q_end` である。L43 の最後の文より `d(q_end, O) = 0` なので、言明は空虚に成り立つ。
    BY <ref id=4ff3e8d/>
  <2>3. QED
    BY <2>1, <2>2, DEF 節点の入口の点
    `DEF 節点の入口の点` より、`p` はこの 2 つのどちらかである。
<1>3. QED
  BY <1>1, <1>2

### L44a (`Del` の `Retain` 節点を越える対応) <!--#fdadbd9-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`Del` の `Retain(v, π, s, k)` 節点である `ρ` の上の節点
`q` を取る。`q` の入口の点で `DEF 対応の 3 つ` が成り立ち、**`q` の入口の点より前の `α` の各点で `α` が
D21 の制限 -- A19 (i) の不等式 -- を満たす**とすると、次の 3 つが成り立つ。

1. `q` の実行の各点 `τ` (`DEF 節点の入口の点`) で 2 つの活性化の点が対応し、`DEF 対応の 3 つ` の (a) と
   (c) が `τ` で成り立つ。とくに `q'` で `DEF 対応の 3 つ` が成り立つ。
2. `q` の実行の各点 `τ` について `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と
   `H'(τ, O) = 0` は同値である。
3. `α` は `q` の実行の各点で D21 の制限 -- A19 (i) の不等式 -- を満たす。

**記法。** `α'` はこの節点を実行しない。`α` では D10 の `Retain` の行が、`π` の下の inhabited (D16)
かつ計数下 (D26) の leaf ごとに 1 つの生成の素動作を行う (`DEF 節点の実行の素動作`)。`q` の実行の点 `τ`
について、`τ` までにこの節点が `O` への参照を作った個数を `m_τ(O)` と書き、`m(O) := m_{q'}(O)` と置く。
DEF 欠損 より `d(τ, O) = d(q, O) + m_τ(O)` である。`H` と `Obl` がそれぞれ `m_τ(O)` 増えることは、
この節点の実行の素動作がその生成だけであることから出る (`<1>1`)。

**証明**

<1>1. この節点の実行の素動作は、D10 の `Retain` の行が定める生成だけである。
      この節点は参照を処分しないので、D24 の (F) の解放は起きず、解放が作る活性化も参照も無い。
      割り当ても伴わない -- D24 の (E2) の `H` の表で `Retain(v, π, s, k)` の行が述べるのは各 leaf の
      +1 だけである。`Retain` 節点は子の活性化を作る 3 種の段にも当たらない -- (E3) は `App` の節点、
      (E2) のうちオペランドを適用する `Llvm` の段は `Llvm` の節点であり、(E7) が走るのは D24 の
      「**(E7) グローバルの初期化の段。** まだ初期化されていないグローバル `g` を読む者が居るとき、
      `g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。読む者は、`g` を読む節点の位置に
      ある生きている活性化 `a` か、**環境**である」の形であって、D7 は `Retain` を読む構文に
      数えない。グローバル化は (E7) が作る活性化が終わるところにしか現れないので (`DEF 節点の実行の
      素動作`)、これも無い。**op の生成コードが出す retain と release も無い** -- それを出すのは
      `Llvm` 節点の op の `generate` であり (`DEF 節点の実行の素動作` の第 6 の箇条)、D2 より
      `Retain` 節点は op を持たない。
  BY <ref id=f06144e/>, <ref id=e3436e8/>, DEF 節点の実行の素動作, <ref id=56c2068/>, <ref id=b3dfa37/>
<1>1b. `q` の実行の点に、`q` の実行に属さない段の素動作は無い。よって `q` の実行の点で `H`・`Obl`・`d` を
       動かすのは、この節点の素動作だけである。
  BY <1>1, <ref id=e3436e8/>, <ref id=c9e4cca/>, <ref id=99174c1/>, DEF 節点の実行の素動作
  <1>1 よりこの節点は子の活性化を作らないので、その実行は D24 の (E2) のちょうど 1 つの段である --
  D24 の (E2) は「生きている活性化 `a` が、自分の位置の節点を実行し、D3 に従って次の位置へ進む」段で
  あり、その脇は「**この段は活性化を 1 つ作るごとに区切られる。**」として区切りが活性化の生成で
  起きることを述べる。D24 は「段は不可分であり」と述べ、A17 (iii) は「段は不可分なので、環境が動くのは
  段と段のあいだである。」と述べるので、その段の中に別の段の素動作は並ばない。
<1>1a. `m(O) ≥ 1` である各計数下オブジェクト `O` について `H(q, O) ≥ 1` である。
  BY <ref id=6358834/>, <ref id=efe22a7/>, <ref id=66c9670/>, <ref id=88a06de/>, 本命題の仮定
  `m(O) ≥ 1` であるのは、`π` の下の inhabited かつ計数下の leaf `λ` であって `obj(v, λ) = O` である
  ものが在るときである。L41d の「とくに」の節がその `O` について `H(q, O) ≥ 1` を与える。
  **L41d を `α` について読むのに要る D21 の制限は、`q` の入口の点までの各点のものである** --
  `L41d` が読む `L41a`・`L41b` は A19 の (ii-a)・(ii-b)・(ii-c) と P14a を `α` に当て、README の
  第 2 節はその形の言明を「時点 `τ` について `τ` までの量だけを結ぶ」と読む。`q` の入口の点より前の
  各点については本命題の仮定が、`q` の入口の点自身については `L44p` の 1 がその制限を与える。
<1>2. `q` の実行の各点 `τ` で 2 つの活性化の点が対応し、(a)・(b)・(c) が `τ` で成り立つ。とくに `q'` で
      `DEF 対応の 3 つ` が成り立つ。
  BY 本命題の仮定, <1>1, <1>1b, <ref id=f06144e/>, <ref id=9d74736/>, DEF 欠損, DEF 対応の 3 つ, DEF 対応する活性化の量, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=7d5a1de/>, <ref id=7218f92/>
  `q` は `Del` の `Retain` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
  しない。D9 の 2 つの表より `Retain` は値を作らず、移さず、手放さないので、`q` の実行の各点で値を
  得ている変数とその値は `q` の入口の点のものと同じであり、(a) が成り立つ。`DEF 対応する活性化の量`
  の点の対応より、`q` の実行の各点に対応する `α'` の点は `q` の入口の点に対応するものと同じである --
  <1>1b よりその区間に `α` の外の素動作は無く、この節点の素動作はどれも `α'` の側に相手を持たない。
  D29 の第 2 行がその点の (b) を与える。`Obl` は D10 の `Retain` の行の分だけ増えて
  `Obl(τ, O) = Obl(q, O) + m_τ(O)` であり、`α'` の側は変わらず、`d` も同じだけ増えるので、(c) が
  `q` の実行の各点で成り立つ。
<1>3. 3 が成り立つ。すなわち `α` は `q` の実行の各点で D21 の制限を満たす。
  BY <ref id=c6ed0c5/>, <1>1, <1>2, <ref id=efe22a7/>, <ref id=18416e9/>
  `q` の入口の点については L44p の 1 が与える -- その仮定のうち `q` の入口の点より前の各点での制限は
  本命題の仮定が与える。残る各点については、L43b の仮定を <1>2 の (a) が、
  `H(τ, O) = H'(τ, O) + d(τ, O)` を <1>2 の (b) が与え、`α'` が対応する点で生きていることを
  L44e が与える -- `q` は `Del` の節点なので `ρ` の終端の `Ret` より真に前にある。
<1>4. 2 が成り立つ。
  <2>1. `q` の実行の各点 `τ` について、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。**L44p の 2 を
        読むのに要る `q` の入口の点より前の各点での制限は、本命題の仮定が与える。**
    BY <1>1, <1>1a, <1>1b, <1>2, 本命題の仮定, <ref id=efe22a7/>, DEF 欠損
    `H(τ, O) = H(q, O) + m_τ(O)`、`d(τ, O) = d(q, O) + m_τ(O)`、`H'(τ, O) = H'(q, O)` である
    (<1>1、<1>1b、DEF 欠損、<1>2)。`H(τ, O) = 0` ならば、`m_τ(O)` も `H(q, O)` も 0 以上なので
    `H(q, O) = 0` であり、本命題の仮定の (b) と L44p の 2 より `d(q, O) = 0` (`d(q, O) ≥ 1` なら
    `H(q, O) ≥ d(q, O) + 1 ≥ 2`) なので `H'(q, O) = 0`、すなわち `H'(τ, O) = 0` である。逆に
    `H'(τ, O) = 0` ならば `H'(q, O) = 0` である。本命題の仮定の (b) より `H(q, O) = d(q, O)` であり、
    `d(q, O) ≥ 1` ならば L44p の 2 が `H(q, O) ≥ d(q, O) + 1` を与えて矛盾するので
    `d(q, O) = 0` かつ `H(q, O) = 0` である。
    <1>1a より `m(O) ≥ 1` なら `H(q, O) ≥ 1` なので、`H(q, O) = 0` のとき `m(O) = 0`、したがって
    `m_τ(O) = 0` であり `H(τ, O) = 0` である。
  <2>2. QED
    BY <1>1, <1>2, <2>1, DEF 欠損
    等式 `H'(τ, O) = H(τ, O) - d(τ, O)` は <1>2 が (b) として与え、0 になることの同値は <2>1 が
    与える。`d(τ, O)` は DEF 欠損 のとおり `d(q, O) + m_τ(O)` である。
<1>5. QED
  BY <1>2, <1>3, <1>4

### L44b (`Del` の `Release` 節点を越える対応) <!--#dcf3c88-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`Del` の `Release(v, π, s, k)` 節点である `ρ` の上の
節点 `q` を取る。`q` の入口の点で `DEF 対応の 3 つ` が成り立ち、**`q` の入口の点より前の `α` の各点で
`α` が D21 の制限 -- A19 (i) の不等式 -- を満たす**とすると、次の 3 つが成り立つ。

1. `q` の実行の各点 `τ` (`DEF 節点の入口の点`) で 2 つの活性化の点が対応し、`DEF 対応の 3 つ` の (a) と
   (c) が `τ` で成り立つ。とくに `q'` で `DEF 対応の 3 つ` が成り立つ。
2. `q` の実行の各点 `τ` について `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と
   `H'(τ, O) = 0` は同値である。
3. `α` は `q` の実行の各点で D21 の制限 -- A19 (i) の不等式 -- を満たす。

**記法。** `α'` はこの節点を実行しない。`α` では D10 の `Release` の行が、`π` の下の inhabited (D16)
かつ計数下 (D26) の leaf ごとに 1 つの処分の素動作を行う (`DEF 節点の実行の素動作`)。`q` の実行の点 `τ`
について、`τ` までにこの節点が `O` への参照を処分した個数を `c_τ(O)` と書き、`c(O) := c_{q'}(O)` と
置く。DEF 欠損 より `d(τ, O) = d(q, O) - c_τ(O)` である。`H` と `Obl` がそれぞれ `c_τ(O)` 減ることは、
この節点の実行の素動作がその処分だけであることから出る (`<1>1`)。

**証明**

<1>0. `q` の実行の各点 `τ` と各計数下オブジェクト `O` について `c_τ(O) ≤ d(q, O)` であり、したがって
      `d(τ, O) = d(q, O) - c_τ(O) ≥ 0` である。
  <2>1. `q` は、ある `t ∈ CT` の `un_bump_releases[t]` にちょうど 1 つ属する。その `t` は `q` で
        pending であり、`q` の訪問の `un_bump` はその要素 `e_t(q)` を選んで `InBracket(t)` を返す。
    BY <ref id=7d5a1de/>, <ref id=23ac734/>, <ref id=19296b2/>, <ref id=941af96/>, 本命題の仮定, DEF 訪問
    L32 の 3 より `Del` は `CT` と `⋃_{t ∈ CT} un_bump_releases[t]` の非交和であり、**L32 の 1 より
    `CT` の各要素は `Retain` 節点である**ので、`Release` 節点である `q` は `CT` に属さず、後者の
    ちょうど 1 つに属する。
    L37 がその `t` が `q` で pending であることを述べ、L32 の 4 が `q` の訪問の `un_bump` が
    `InBracket(t)` を返すことを述べる。`p30` の `L5` の 3 より、`InBracket` が返す `NodeId` は
    `un_bump` が選んだ要素の `node` なので、選ばれた要素は由来が `t` の要素であり、L50 の 1 より
    それは `e_t(q)` ただ 1 つである。
  <2>2. 各名前 `o` について `B_ρ(q, e_t(q))[o] ≥ ActRefs^inh_ρ(q)[o]` である。
    BY <2>1, <ref id=d9d155f/>, p13 の DEF 実行時の作用, <ref id=8093b68/>
    `p13` の `L11a` は、`Release` 節点の訪問の `un_bump` が `InBracket` を返して `pending` の要素
    `p_i` を選ぶとき、各名前についてこの不等式が成り立つと述べる。<2>1 よりその `p_i` は `e_t(q)` で
    ある。
  <2>3. `c_τ(O) ≤ Σ_{o : obj_ρ(o) = O} ActRefs^inh_ρ(q)[o]` である。和は `ρ` で活性な名前 `o` を渡る。
    BY <ref id=f06144e/>, <ref id=4f4714c/>, <ref id=b154692/>, p13 の DEF 実行時の作用,
       p13 の DEF 名前をオブジェクトへ写す, p13 の DEF 名前の活性, DEF 節点の実行の素動作
    D10 の `Release` の行より、この節点が処分する参照は `π` の下の inhabited かつ計数下の各 leaf に
    つき 1 つであり (`DEF 節点の実行の素動作` の第 1 の箇条)、`c_τ(O)` はそのうち `τ` までに済んだ
    `O` への分の個数なので、この節点が処分する `O` への参照の総数を超えない。P6 は「この数え上げを
    **inhabited (D16) かつ計数下 (D26)** の leaf に制限し、**各名前をそれが指すオブジェクトへ写して**
    得られる多重集合は … `Release(v, π)` が処分する参照の多重集合にも等しい」と述べ、`p13` の
    `DEF 実行時の作用` の `ActRefs^inh_ρ(q)` がその制限された数え上げである。写しは `p13` の
    `DEF 名前をオブジェクトへ写す` の `(・)^obj` であり、`L45` よりその像の `O` の個数は活性な名前 `o`
    について `obj_ρ(o) = O` の分の和である。
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=4ff3e8d/>, <ref id=948f840/>, DEF 欠損
    L43 より `d(q, O) = Σ_{t'} Σ_{o : obj_ρ(o) = O} B_ρ(q, e_{t'}(q))[o]` であり、外側の和は `CT` に
    属し `q` で pending である `Retain` 節点 `t'` を渡る。<2>1 の `t` はその 1 つである。P18b の
    `B(p, ρ)` は多重集合なので各名前の個数は 0 以上であり、残る項は
    0 以上であって `d(q, O) ≥ Σ_{o : obj_ρ(o) = O} B_ρ(q, e_t(q))[o]` である。<2>2 よりこれは
    `Σ_{o : obj_ρ(o) = O} ActRefs^inh_ρ(q)[o]` 以上であり、<2>3 より `c_τ(O)` 以上である。
    DEF 欠損 より `d(τ, O) = d(q, O) - c_τ(O)` なので `d(τ, O) ≥ 0` である。
<1>0a. `Release` 節点は割り当てを伴わず、子の活性化を作る 3 種の段にも当たらず、グローバル化も伴わず、
       op の生成コードが出す retain と release も持たない。
  BY <ref id=e3436e8/>, <ref id=56c2068/>, DEF 節点の実行の素動作, <ref id=b3dfa37/>
  (E3) は `App` の節点、(E2) のうちオペランドを適用する `Llvm` の段は `Llvm` の節点であり、(E7) が
  走るのは D24 の「**(E7) グローバルの初期化の段。** まだ初期化されていないグローバル `g` を読む者が
  居るとき、`g` のアクセサが `g` の初期化子の `init` の活性化 `b` を作る。読む者は、`g` を読む節点の
  位置にある生きている活性化 `a` か、**環境**である」の形であって、D7 は `Release` を読む構文に
  数えない。グローバル化は (E7) が作る活性化が終わるところにしか現れない (`DEF 節点の実行の素動作`)。
  op の生成コードが出す retain と release を出すのは `Llvm` 節点の op の `generate` であり
  (`DEF 節点の実行の素動作` の第 6 の箇条)、D2 より `Release` 節点は op を持たない。**割り当ても無い**
  -- `DEF 節点の実行の素動作` の第 2 の箇条は割り当てを伴う素動作を D10 の生成の表の `Closure(f, caps)`
  の capture object と単一の `Fresh` を宣言する `Llvm` の結果の leaf に限り、D2 より `Release` 節点は
  そのどちらでもない。
<1>1. この節点の実行はどのオブジェクトも解放しない。よってその素動作は D10 の `Release` の行が定める
      処分だけであり、`q` の実行の点に `q` の実行に属さない段の素動作も無い。
  <2>1. この節点の実行が解放を起こすとすれば、その最初の解放について、`q` の実行のある点 `τ` と
        計数下オブジェクト `O0` があって、`c_τ(O0) ≥ 1`、`H(τ, O0) = 0`、かつ
        `H(τ, O0) = H(q, O0) - c_τ(O0)` である。
    BY <ref id=e3436e8/>, DEF 節点の実行の素動作, <1>0a, <ref id=56c2068/>
    D24 の (F) より解放が起きるのは、参照を処分して計数下オブジェクトのカウントが 0 になったときで
    あり、D24 は「**解放は、それを起こした処分に付随して起きる。**」と述べる。最初の解放を起こす
    処分の直後の点を `τ`、そのオブジェクトを `O0` とすると `H(τ, O0) = 0` であり、その処分は
    この節点のものなので `c_τ(O0) ≥ 1` である。`τ` より前にこの節点の実行が持つ素動作は、`<1>0a` と
    `DEF 節点の実行の素動作` より D10 の `Release` の行が定める処分だけである -- 解放は `τ` が最初
    なので、それより前に解放の素動作は無い。よって `q` の入口の点から `τ` までの `H(・, O0)` の変化は
    この節点の処分の分だけであり、`H(τ, O0) = H(q, O0) - c_τ(O0)` である。
  <2>2. `d(q, O0) ≥ c_τ(O0) ≥ 1` である。
    BY <2>1, <1>0
    <1>0 より `c_τ(O0) ≤ d(q, O0)` であり、<2>1 より `c_τ(O0) ≥ 1` である。
  <2>3. QED
    BY <2>1, <2>2, <1>0a, <ref id=efe22a7/>, <ref id=e3436e8/>, <ref id=99174c1/>, <ref id=c9e4cca/>, 本命題の仮定, DEF 欠損, DEF 節点の実行の素動作
    <2>2 と L44p の 2 より `H(q, O0) ≥ d(q, O0) + 1 ≥ c_τ(O0) + 1` である -- L44p の 2 を読むのに要る
    `q` の入口の点より前の各点での制限は本命題の仮定が与える。ところが
    `H(τ, O0) = H(q, O0) - c_τ(O0) = 0` より `H(q, O0) = c_τ(O0)` であり、矛盾する。よって <2>1 の
    形は起きない。解放が無いので、解放が作る活性化も参照もこの実行に無い。**したがってこの節点は
    子の活性化を 1 つも作らず** (<1>0a と解放の不在)、**その実行は D24 の (E2) のちょうど 1 つの段で
    ある** -- D24 の (E2) の脇は「**この段は活性化を 1 つ作るごとに区切られる。**」として区切りが
    活性化の生成で起きることを述べる。D24 は「段は不可分であり」と述べ、A17 (iii) は「段は不可分なので、
    環境が動くのは段と段のあいだである。」と述べるので、その段の中に別の段の素動作は並ばない。
<1>2. `q` の実行の各点 `τ` で 2 つの活性化の点が対応し、(a)・(b)・(c) が `τ` で成り立つ。とくに `q'` で
      `DEF 対応の 3 つ` が成り立つ。
  BY 本命題の仮定, <1>0a, <1>1, <ref id=f06144e/>, <ref id=9d74736/>, DEF 欠損, DEF 対応の 3 つ, DEF 対応する活性化の量, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=7d5a1de/>, <ref id=7218f92/>
  `q` は `Del` の `Release` 節点なので、L30 と L31 より `ρ'` にはこの節点が無く、`α'` はこれを実行
  しない。D9 の 2 つの表より `Release` は値を作らず、移さず、手放さないので、`q` の実行の各点で値を
  得ている変数とその値は `q` の入口の点のものと同じであり、(a) が成り立つ。
  `DEF 対応する活性化の量` の点の対応より、`q` の実行の各点に対応する `α'` の点は `q` の入口の点に
  対応するものと同じである -- <1>1 よりその区間に `α` の外の素動作は無く、この節点の素動作はどれも
  `α'` の側に相手を持たない。D29 の第 2 行がその点の (b) を与える。`Obl` は D10 の `Release` の行の
  分だけ減って `Obl(τ, O) = Obl(q, O) - c_τ(O)` であり、`α'` の側は変わらず、`d` も同じだけ減るので、
  (c) が `q` の実行の各点で成り立つ。
<1>3. 3 が成り立つ。すなわち `α` は `q` の実行の各点で D21 の制限を満たす。
  BY <ref id=c6ed0c5/>, <1>2, <ref id=efe22a7/>, <ref id=18416e9/>, 本命題の仮定
  `q` の入口の点については L44p の 1 が与える -- その仮定のうち `q` の入口の点より前の各点での制限は
  本命題の仮定が与える。残る各点については、L43b の仮定を <1>2 の (a) が、
  `H(τ, O) = H'(τ, O) + d(τ, O)` を <1>2 の (b) が与え、`α'` が対応する点で生きていることを
  L44e が与える -- `q` は `Del` の節点なので `ρ` の終端の `Ret` より真に前にある。
<1>4. 2 が成り立つ。
  <2>1. `q` の実行の各点 `τ` について、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。**L44p の 2 を
        読むのに要る `q` の入口の点より前の各点での制限は、本命題の仮定が与える。**
    BY <1>0, <1>1, <1>2, 本命題の仮定, <ref id=efe22a7/>, DEF 欠損
    `H(τ, O) = H(q, O) - c_τ(O)`、`d(τ, O) = d(q, O) - c_τ(O)`、`H'(τ, O) = H'(q, O)` である
    (<1>1、DEF 欠損、<1>2)。`H(τ, O) = 0` とする。`c_τ(O) ≥ 1` ならば <1>0 より
    `d(q, O) ≥ c_τ(O) ≥ 1` であり、L44p の 2 より `H(q, O) ≥ d(q, O) + 1 ≥ c_τ(O) + 1` となって
    `H(q, O) = c_τ(O)` に反する。よって `c_τ(O) = 0` かつ `H(q, O) = 0` であり、本命題の仮定の (b) と
    L44p の 2 より `d(q, O) = 0`、したがって `H'(τ, O) = H'(q, O) = 0` である。逆に `H'(τ, O) = 0` なら
    `H'(q, O) = 0` である。本命題の仮定の (b) より `H(q, O) = d(q, O)` であり、`d(q, O) ≥ 1` ならば
    L44p の 2 が `H(q, O) ≥ d(q, O) + 1` を与えて矛盾するので `d(q, O) = 0` かつ
    `H(q, O) = 0` である。`H(τ, O)` は 0 以上で
    `H(q, O) - c_τ(O)` に等しいので `c_τ(O) = 0` であり、`H(τ, O) = 0` である。
  <2>2. QED
    BY <1>1, <1>2, <2>1, DEF 欠損
    等式 `H'(τ, O) = H(τ, O) - d(τ, O)` は <1>2 が (b) として与え、0 になることの同値は <2>1 が
    与える。`d(τ, O)` は DEF 欠損 のとおり `d(q, O) - c_τ(O)` である。
<1>5. QED
  BY <1>0, <1>0a, <1>1, <1>2, <1>3, <1>4

### L44c (`Del` に入らない節点を越える対応) <!--#45214bd-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`Del` に入らない `ρ` の上の節点 `q` を取る。`q` の
入口の点で `DEF 対応の 3 つ` が成り立ち、**`q` の入口の点より前の `α` の各点で `α` が D21 の制限 --
A19 (i) の不等式 -- を満たす**とすると、次の 3 つが成り立つ。

1. `q` の実行の各点 `τ` (`DEF 節点の入口の点`) で 2 つの活性化の点が対応し、`DEF 対応の 3 つ` の (a) と
   (c) が `τ` で成り立つ。とくに `q'` で `DEF 対応の 3 つ` が成り立つ。
2. `q` の実行の各点 `τ` について `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と
   `H'(τ, O) = 0` は同値である。
3. `α` は `q` の実行の各点で D21 の制限 -- A19 (i) の不等式 -- を満たす。

**記法。** `ρ'` の対応する節点も同じ節点である (L30、L31)。本命題の仮定より `q` は `Del` に入らないので、
DEF 欠損 よりこの実行のどの点でも `d(τ, O) = d(q, O)` である。

**証明**

<1>1. 2 つの実行はこの節点について D10 の同じ行を同じ値に対して適用し、この節点が束縛する変数に
      対応する値を置く。さらに、この節点が子の活性化を作るとき、その子の事象が参照カウントに与える
      変化は 2 つの実行で同じである。`d` は変わらない。
  <2>1. `q` は `Del` に入らないので、L30 と L31 より `ρ'` の対応する節点は `q` と同じ式の
        変位・変数・path・`RcState` を持ち、**`Let` の右辺 (`Llvm` の op とオペランド、`Closure` の
        `FuncRef` と capture、`App` の callee と引数、`Var` の変数、`Match` の scrutinee)、
        `Destructure` のフィールドの列、`Match` のアームの本数と並びとその `tag`・`payload`・
        `payload_state` も `q` のものである** -- L30 の言明がこの一覧を持つ。本命題の仮定の (a) より、
        その点までに値を得ている各変数の値は 2 つの実行で対応する。
    BY <ref id=4a0c14c/>, <ref id=2bb344b/>, 本命題の仮定
  <2>1a. `q` の式が `Let(_, RcRhs::Match(v, arms), k)` であるとき、2 つの活性化は同じアームへ進む。
    BY <ref id=c232680/>, <ref id=7218f92/>, <2>1
    D21 は `Match` のアームを `v` の値の実行時のタグで決める。D29 の第 5 行は「**スカラの成分に
    ついては、対応は等号である。** boxed leaf でない成分 -- unbox union のタグ、整数、
    浮動小数 -- は、対応する 2 つの値で等しい。」と述べ、続けて「D21 は `Match` のアームを `v` の値の
    実行時のタグで決めるので、この節が、対応する 2 つの活性化が同じアームへ進むことを与える。」と
    述べる。<2>1 より `v` の値は 2 つの実行で対応するので、unbox union のタグは 2 つで等しい。
  <2>2. この節点が束縛する値は 2 つの実行で対応する。
    <3>1. CASE その値が、節点の形と、それが名指す変数の値と、その値から到達できる (D25) オブジェクト
          が保持する値だけで決まる。D2 の節点の表と D9 の移動の表と D10 の生成の表がその値を定める。
          **この場合に入る構文は次で尽きる。** D9 の値の水準の 6 行が渡す値 -- `Let(x, Var(y), k)` の
          `x`、アーム本体の `Ret(x)` が渡す `Match` の束縛変数、unbox 容器の `Destructure` の
          名前付きフィールド変数、unbox union の変位アームの payload 変数、catch-all アームの payload
          変数、`Llvm` の素通し leaf --、D10 の生成の表のうちオブジェクトの中から読み出す 3 つ --
          boxed 容器の `Destructure` のフィールド、boxed union の変位アームの payload、単一の
          `Unknown` を宣言する `Llvm` の結果の leaf (A3 より、そのオブジェクトはオペランドの leaf が
          指すオブジェクトから到達できるか、グローバル値が到達する) --、そして `Ret(v)` が渡す値で
          ある。
          **その決定の規則は D29 の全単射と可換である。** D9 の値の水準の 6 行はどれも、名指す値の
          成分を取り出すか、その値をそのまま渡すかであり (D9 -- 「**値の水準では、移動の各行は次を
          渡す。**」)、D10 の生成の 3 つはオブジェクトが保持する値の成分を取り出す。D29 の第 5 行は
          「**対応するオブジェクトが保持する値は対応する。**」を置き、「対応する各位置において、
          inhabited (D16) な各 boxed leaf が全単射で対応するオブジェクトを指し」「**スカラの成分に
          ついては、対応は等号である。**」と成分ごとに対応を定め、続けて「**スカラの成分についての
          この読みは、オブジェクトが保持する値に限らない。** 対応する 2 つの活性化の対応する位置に
          おける、対応する 2 つの値のどれについても同じく読む」と広げる。すなわち対応は値の成分ごとに
          定まるので、対応する 2 つの値から同じ成分を取り出した 2 つの値もまた対応する。<2>1 より
          名指す変数の値は 2 つの実行で対応し、D29 の第 5 行より対応するオブジェクトが保持する値も
          対応するので、この値も 2 つの実行で対応する。
      BY <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=3a95067/>, <ref id=f06144e/>, <ref id=e11772a/>, <ref id=3f1bb47/>, <ref id=4f63121/>, <ref id=0b850c9/>, <ref id=7218f92/>, <ref id=66c9670/>, <2>1
    <3>1a. CASE その値が、この段が割り当てるオブジェクトを指す leaf を持つ。すなわち
           `Closure(f, caps)` の結果 (capture object) と、`result_prov` が単一の `Fresh` を宣言する
           `Llvm` の結果の leaf である。D29 の第 3 行は「D10 の生成の表が割り当てを行う位置 --
           `Closure(f, caps)` の capture object と、`result_prov` が単一の `Fresh` を宣言する `Llvm` の
           結果の leaf -- について、対応する位置が割り当てる 2 つのオブジェクトは対応する。」と述べる。
           <2>1 より 2 つの節点は同じ位置のものなので、この行がその 2 つのオブジェクトを対応させる。
           値の残る成分は <3>1 の形で決まる。
      BY <ref id=7218f92/>, <ref id=f06144e/>, <3>1, <2>1
    <3>2. CASE それ以外である。すなわちこの構文はオペランドから結果が決まらない構文である。D21 は
          「その割り当ては、オペランドから結果が決まらない構文の結果を含む」と述べ、D29 はその各位置
          での結果を 2 つの活性化に同じものとして与える。
      BY <ref id=c232680/>, <ref id=7218f92/>, <2>1
    <3>2a. この節点が束縛する値の Fix の関数型の成分は、`DEF 対応の 3 つ` の (a) が定める意味で
           **対応する版**を名指す。
      BY <3>1, <3>1a, <3>2, <2>1, <ref id=1dbc8d1/>, <ref id=ebec376/>, <ref id=7218f92/>, <ref id=596a46d/>, <ref id=cb35ab1/>, <ref id=8d3e4af/>, <ref id=0ca4686/>, <ref id=4a0c14c/>, <ref id=746e87a/>, <ref id=88a06de/>,
         CODE src/rc_ir/borrow.rs: drop_nodes_inner, CODE src/rc_ir/borrow.rs: cancel,
         前提 記号の位置の値の対応, DEF 対応の 3 つ
      **「対応する版」の語を定めるのは `DEF 対応の 3 つ` の (a) である** -- 入力の関数と出力の関数が
      対応する版であるとは、P24 の第 2 の箇条が言う「**出力の各関数は入力のちょうど 1 つの関数から
      作られ**」の関係にあること、すなわち `cancel` が入力の `f` から作った出力の関数であることを
      いう。A21 は「Fix の関数型の値に LLVM 関数の番地を書き込むのは、クロージャを作る段
      (`build_rc_closure`)、funptr のグローバルを読む段 (`ValueAccessor::get` の `is_funptr` の枝)、
      そして `InlineLLVMFixBody` の 3 か所だけである。ほかのどの構文も op も、既にある関数の値を写す
      だけである。」と述べるので、場合は次の 4 つで尽きる。
      - **既にある関数の値を写す場合。** その成分は、この節点が名指す変数の値の成分か、オブジェクトが
        保持する値の成分か、記号の位置 (D6) の値の成分から来る。前の 2 つについては <2>1 と <3>1 が
        言明を与える -- D29 の第 5 行が「**funptr の番地は等号では読めない** …**その成分については、
        対応する 2 つの番地が対応する版を名指す**」と読み、その読みを「対応する 2 つの活性化の対応する
        位置における、対応する 2 つの値のどれについても同じく読む」と広げる。3 つ目は下の第 3 の箇条と
        同じである。
      - **クロージャを作る段。** その節点は `Let(x, Closure(f, caps), k)` であり、L30 より `B'` の
        対応する節点の `f` は `B` のものと同じ `FuncRef` である。`cancel` は入力の各関数 `f` に
        ついて `f.clone()` の `body` だけを差し替え、出力の `funcs` の鍵に `f.name.clone()` を据える
        ので (`L44d`)、鍵 `f` が入力で引く関数と出力で引く関数は対応する版である。
      - **funptr のグローバルを読む段。** その節点が読むのは束縛を持たない名前 `g` の記号の位置で
        ある (D6)。**その位置が funptr を指すとき、`g` は `prog.funcs` の鍵である** -- D6 は
        「**束縛を持たない名前は、必ず最上位の記号の名前である。**」と述べ、A13 はその 2 種を
        「直接呼び出しが名指す関数の名前と、グローバル値を読む `RcVar` の名前」と挙げる。前者は関数の
        名前であり、A22 より `prog.funcs` の鍵はその関数の `name` である。D24 は「**したがって、
        実行時に名前 `n` が指す関数は `P.funcs[FuncRef{n}]` の本体を実装したものである**」と定めるので、
        その funptr が名指すのは 2 つのプログラムの `funcs` の同じ鍵の関数であり、`L44d` よりその 2 つは
        対応する版である。**その位置がグローバル状態のオブジェクト (D26) を指すときは、その値の関数型の
        成分について `前提 記号の位置の値の対応` が言明を与える。**
      - **`InlineLLVMFixBody`。** この op はオペランドを適用するので、その結果はオペランドから決まらず、
        <3>2 の場合に入る。D29 はその位置での結果を 2 つの活性化に同じものとして与え、値が「対応する」
        ことの読みは D29 の第 5 行の funptr の但し書きに従う。
    <3>3. QED
      BY <3>1, <3>1a, <3>2, <3>2a
      値は、<3>1 の 3 つ (節点の形、名指す変数の値、そこから到達できるオブジェクトが保持する値) で
      決まるか、この段が割り当てるオブジェクトを指す leaf を持つか (<3>1a)、そのどちらでもないかの
      いずれかである。どちらでもないものは、A3 の但し書きが挙げる
      `InlineLLVMBoxedFromRetainedPtrIOS` のように到達できる元を持たない `Unknown` を含めて、
      オペランドから結果が決まらない構文であり、<3>2 が扱う。Fix の関数型の成分については <3>2a が
      「対応する版を名指す」の形で述べる。
  <2>3. この節点が子の活性化を作るとき、その子の事象が参照カウントに与える変化は 2 つの実行で同じ
        である。
    BY <ref id=e3436e8/>, <ref id=c232680/>, <ref id=7218f92/>, <2>1
    D24 の「活性化の林」より、子の活性化を作る段は 4 つである。(E3) の `App`、(E7) のグローバルの
    初期化、(E2) のうちオペランドを適用する `Llvm` の段、そして (F) の解放が `Destructor` に
    ついて作る段である。D21 の第 4 行は「**子の活性化を作る段** (D24 の「活性化の林」)。`App` の段
    (E3)、オペランドを適用する `Llvm` の段 (E2)、グローバルの初期化の段 (E7)、そして `Destructor` の
    オブジェクトを解放する段 (F) である。返る値と、参照カウントに与える変化は、子の本体が決める。
    1 つの本体だけを見ている間、これは外から与えられる量である。**(F) はどの構文でも起こりうる** --
    参照を処分する段はどれもオブジェクトの解放を起こしうるので、`Release` の節点も、消費を行う
    `App` や `Destructure` の節点も、この意味で外から与えられる量を持つ」と述べ、この 4 つを 1 つの
    行で扱う。D29 の第 4 行はそのデータを対応する 2 つの活性化に同じものとして与える。
  <2>3a. `App` の節点について、D9 の `App` の行が名指す leaf は 2 つの実行で同じである。
    <3>1. 2 つの実行の呼び出し先は、互いに対応する 2 つの版である。
      BY <ref id=ff5985d/>, <ref id=7218f92/>, <2>1, 本命題の仮定
      D23 より `Let(x, App(callee, args), k)` の呼び出し先は、その段で `callee` の値が指す関数で
      あり、`callee` の値がクロージャならその funptr の指す関数、funptr ならそれ自身である。<2>1 より
      `callee` の値は 2 つの実行で対応する。**その成分は等号では読めない** -- D29 の第 5 行は
      「**funptr の番地は等号では読めない** -- 2 つのプログラムは別々にコンパイルされるので、同じ
      関数の番地が同じ値になるとは限らない。」と述べ、続けて「**その成分については、対応する
      2 つの番地が対応する版を名指す**」と述べる。**この読みは呼び出し先を決める値にも当たる** --
      D29 は続けて「**スカラの成分についてのこの読みは、オブジェクトが保持する値に限らない。** 対応する
      2 つの活性化の対応する位置における、対応する 2 つの値のどれについても同じく読む -- 入力の束縛が
      受け取る値も、段が作る値も、呼び出し先を決める値も、記憶域へ格納する値もである。**funptr の
      成分を「対応する版を名指す」と読む段は、その値がオブジェクトの中から来たかどうかを問わない。**」と
      述べる。よって 2 つの番地は対応する版を名指す。
    <3>2. `B` の関数と `B'` の関数が対応する版であるならば、その 2 つは同じ `name` を持ち、その
          `params` と `borrowed_units` は等しい。
      BY <ref id=1dbc8d1/>, <ref id=746e87a/>, CODE src/rc_ir/borrow.rs: cancel
      `cancel` は入力の各関数 `f` について `f.clone()` の `body` だけを差し替え、出力の `funcs` の
      鍵に `f.name.clone()` を据えるので、出力の版は入力の版と同じ `name` を持ち、対応はその名前で
      決まる。P24 は「**`cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。** とくに
      `borrowed_units` と `capture` は入力のものに等しい」と述べ、L44d が `params` と
      `borrowed_units` について同じことをこの文書の中で述べる。
    <3>3. QED
      BY <3>1, <3>2, <ref id=9d74736/>, <ref id=ef8efc4/>
      D9 の `App` の行は、callee の全 boxed leaf と、**呼び出し先がその位置の unit を所有する (D14)**
      引数の leaf を名指す。D14 の所有は `RcFunc::borrowed_units` が定めるので、<3>1 と <3>2 より
      2 つの実行で等しい。
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3, <2>3a, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=66c9670/>, <ref id=3f1bb47/>, <ref id=7218f92/>, DEF 欠損,
       CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs
    言明の前半は <2>2 と <2>1a であり、後半は <2>3 である。D10 の行が名指す leaf は、節点の形と値
    (D16、A4) と、`App` については呼び出し先の所有 (<2>3a) で決まるので、2 つの実行で同じである。
    **同じ leaf が 2 つの実行で inhabited (D16) になる** -- D16 は inhabited を「`λ` が通る unbox union
    の各節において、`λ` がその節で選ぶ変位番号が、その時点の `v` のその節のタグに等しいこと」と定め、
    D29 の第 5 行は「**スカラの成分については、対応は等号である。** boxed leaf でない成分 -- unbox
    union のタグ、整数、浮動小数 -- は、対応する 2 つの値で等しい。」と述べるので、<2>1 と <2>2 で
    対応する 2 つの値のタグは等しい。
    `q` は `Del` に入らないので DEF 欠損 の 2 つの数え上げは変わらない。
<1>2. `q` の実行の各点 `τ` で 2 つの活性化の点が対応し、`α` は `τ` で D21 の制限を満たし、
      `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。`q` の
      入口の点と、その実行が終わった点 `q'` もその範囲に入る。本命題の仮定より `q` は `Del` に
      入らないので、DEF 欠損 より `d(τ, O) = d(q, O)` である。すなわちこれは本命題の 2 と 3 である。
  <2>1. `q` の入口の点について、点は対応し、`H'(q, O) = H(q, O) - d(q, O)` であり、`H(q, O) = 0` と
        `H'(q, O) = 0` は同値であり、`α` はその点で D21 の制限を満たす。
    BY 本命題の仮定, <ref id=efe22a7/>
    本命題の仮定の (a) が点の対応を、本命題の仮定の (b) が等式を、L44p の 1 が D21 の制限を与える --
    L44p の仮定のうち `q` の入口の点より前の各点での制限は本命題の仮定が与える。`d(q, O) = 0` のとき
    両辺は等しい。`d(q, O) ≥ 1` のとき L44p の 2 より `H(q, O) ≥ d(q, O) + 1 ≥ 2 > 0` であり、
    `H'(q, O) = H(q, O) - d(q, O) ≥ 1 > 0` なので、どちらも 0 でない。
  <2>2. この節点の実行の点に在る素動作は、次の 7 種のいずれかである。`q` の実行に属する段
        (`DEF 節点の実行の素動作`) が行うものは
        `DEF 節点の実行の素動作` の 6 種 -- D10 の行が定める作成と処分、割り当て、子の活性化の素動作、
        (F) の解放、グローバル化、op の生成コードが出す retain と release -- で尽き (`L47` の 1)、
        残る 1 種は `q` の実行に属さない段 -- 別の制御の流れの活性化の段と、環境の段 -- の素動作である。
        **この節点が子の活性化を作るとき `α` は中断中であり、そのあいだにその段が並びうる** --
        A17 (iii) は「環境の動作も D24 の段としてこの実行の 1 つの列に並ぶ」と述べ、「複数の制御の
        流れがある実行では、環境の段は生きている活性化の段と交互に並ぶ」と続ける。素動作の
        順序はこの文書が定めず、以下の帰納は順序を使わない。本命題の仮定より `q` は `Del` に入らない
        ので、DEF 欠損 よりこの実行のどの点でも `d(τ, O) = d(q, O)` である。
    BY DEF 節点の実行の素動作, <ref id=ba12699/>, <ref id=c9e4cca/>, <ref id=f06144e/>, <ref id=e3436e8/>, DEF 欠損, 本命題の仮定
  <2>3. この節点の実行の素動作の個数についての帰納法で、各点について、2 つの活性化の点が対応し、
        `α` がその点で D21 の制限を満たし、`H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H(τ, O) = 0` と
        `H'(τ, O) = 0` が同値であることを示す。
    <3>1. 基底は `q` の入口の点であり、<2>1 がこの 4 つを与える。
      BY <2>1
    <3>2. 対応が定まっている点 `τ` について、`τ` の次の素動作は 2 つの活性化で対応する。
      <4>1. CASE 次の素動作が子の活性化のもの、またはグローバル化である。<1>1 の後半より、この節点が
            作る子の活性化の事象が参照カウントに与える変化は 2 つの実行で同じである。D21 の第 4 行が
            それを活性化の側のデータとし、D29 の第 4 行が 2 つの活性化に同じものとして与える。
            グローバル化は (E7) が作る活性化が終わるところに現れ、その活性化の事象も同じ行が扱う。
            **その素動作の列が 1 対 1 に対応することは `前提 対応は素動作の粒度で読む` の 1 が与える**
            -- D21 の第 4 行と D29 の第 4 行が与件にするのは返る値と参照カウントへの変化であって、
            子の活性化の節点が行う動作の列ではない。
        BY <1>1, <ref id=c232680/>, <ref id=7218f92/>, <ref id=e3436e8/>, 前提 対応は素動作の粒度で読む
      <4>2. CASE 次の素動作が D10 の行が定める作成・処分、または割り当てである。<1>1 より 2 つの
            実行はこの節点について D10 の同じ行を同じ値に対して適用し、割り当てを伴う行 -- 単一の
            `Fresh` を宣言する `Llvm` の結果の leaf と `Closure(f, caps)` の結果 -- は 2 つの実行で
            同じである (D24 の (E2) の `H` の表)。**割り当てたオブジェクトが対応することは D29 の
            第 3 行が与える** -- 「D10 の生成の表が割り当てを行う位置 -- `Closure(f, caps)` の
            capture object と、`result_prov` が単一の `Fresh` を宣言する `Llvm` の結果の leaf --
            について、対応する位置が割り当てる 2 つのオブジェクトは対応する。」
        BY <1>1, <ref id=f06144e/>, <ref id=e3436e8/>, <ref id=7218f92/>
      <4>3. CASE 次の素動作が処分であって、それが D24 の (F) の解放を起こす。D24 の (F) より、解放
            されるのはその処分によってカウントが 0 になった計数下オブジェクトであり、D24 は
            「**解放は、それを起こした処分に付随して起きる。** 処分とその処分が起こす解放のあいだに
            段内の点は挟まらない。」と述べるので、この場合の素動作は処分とその解放の対である。
            **解放されるオブジェクトの集合は 2 つの実行で対応する。** その処分が 2 つの実行で対応する
            ことは <4>2 が与えるので、処分の直後の 2 つのカウントは `τ` のものから同じだけ下がり、
            帰納法の仮定の `H(τ, O) = H'(τ, O) + d(q, O)` はそこでも成り立つ。D7 より参照カウントは
            0 以上なので、`d(q, O) ≥ 1` の `O` では処分の直後のカウントが `d(q, O) ≥ 1` 以上であって
            解放は起きず、`d(q, O) = 0` の `O` では 2 つのカウントが等しいので、片方で 0 になることと
            他方で 0 になることは同値である。**この議論が点についての命題を読まないのは、処分とその
            解放のあいだに点が無いからである。**
            解放が処分するのは、そのオブジェクトが保持する参照である
            (D24 の (F)、D25)。A5 より、その参照は inhabited であって計数下のオブジェクトを指す各
            boxed leaf に 1 つずつ在り、D29 の第 5 行より対応するオブジェクトが保持する値は対応
            するので、処分される参照は対応する。**`#ArrayStorage a` のオブジェクトは A5 の例外で
            あり、持ち手の単位は leaf ではなく要素の位置である** -- 各位置が、要素の型の
            `boxed_leaf_paths` が列挙する leaf を 1 組ずつ持ち、解放の走査はそれを 1 つずつ処分する
            (A5)。**その走査は位置を 1 つ飛ばすことがある** -- `traverse_array_buf` は
            `hole: Option<IntValue>` を取り、`None` の腕は `traverse_array_range` を `[0, size)` に
            ついて 1 回呼び、`Some(hole)` の腕は `[0, hole)` と穴の後ろの残りについて 2 回呼んで位置
            `hole` を飛ばす (`CODE src/object.rs: ObjectFieldType::traverse_array_buf`,
            `ObjectFieldType::traverse_array_range`)。**どちらの腕を取るかは、その走査を出す生成コードの
            `obj.ty` -- そのオブジェクトの Fix の型 -- が決める。** `build_traverse` は
            `obj.ty.is_array()` と
            `obj.ty.is_punched_array()` で枝を選び、`traverse_array_buf` に渡す `hole` を `None` か
            `Some(idx)` に決める (`CODE src/object.rs: build_traverse`)。**対応する 2 つのオブジェクトの
            Fix の型は同じである** -- `L43b` の 4 がそれを述べ、その仮定は帰納法の仮定と <1>1 が
            与える。**連鎖が解放するオブジェクトはこの節点が名指すものに限らない**ので、`q` の構文から
            その型を出すことはできず、この段はそこを `L43b` の 4 に置く。**`size` と `idx` は解放される
            オブジェクトが保持する値のスカラの成分であり、2 つの実行で等しい** -- D29 の第 5 行は
            「**スカラの成分については、対応は等号である。**」と述べる。
            よって走査する位置の集合も 2 つの実行で同じである。各要素の位置の leaf が対応する
            オブジェクトを指すことは同じ行の
            「対応する各位置において、inhabited (D16) な各 boxed leaf が全単射で対応するオブジェクトを
            指し」が与える。
            **`Destructor` のオブジェクトの解放は活性化を 2 つ作る** -- D24 の「活性化の林」は
            「**(F) の解放が作る活性化は 2 つである** -- `_dtor` の欄の関数を `_value` の欄の値へ
            適用するものと、それが返した `IO` の動作の runner を適用するものであり、2 つ目の入力は
            1 つ目の結果である」と述べる。その解放が作る retain は `_dtor` の欄の値に当たり、1 つ目の
            適用が受け取るのは `_value` の欄の値であり、どちらも D29 の第 5 行より 2 つの実行で対応
            する。2 つ目の活性化の入力は 1 つ目の結果であり、その結果と、2 つの活性化の事象が
            2 つの実行で同じであることは <1>1 の後半が述べる -- D21 の第 4 行が (F) の作る活性化を
            活性化の側のデータに数え、D29 の第 4 行がそれを 2 つの活性化に同じものとして与える。
        BY 帰納法の仮定, <1>1, <4>2, 本命題の仮定, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=a4961ed/>, <ref id=56c2068/>, <ref id=0b850c9/>, <ref id=7218f92/>, <ref id=4f63121/>, <ref id=0c081dc/>,
           CODE src/object.rs: ObjectFieldType::traverse_array_buf,
           CODE src/object.rs: ObjectFieldType::traverse_array_range,
           CODE src/object.rs: build_traverse, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=746e87a/>, <ref id=3f1bb47/>
      <4>3a. CASE 次の素動作が op の生成コードが出す retain か release、またはそれが作った参照を
             呼び出し先へ渡す受け渡しである。この素動作を出すのは `q` の `Llvm` 節点の op の
             `generate` であり (`DEF 節点の実行の素動作` の第 6 の箇条)、L30 と L31 より `ρ'` の対応する
             節点は `q` と同じ位置にあって、その `Llvm` の op は `q` の op の複製、オペランドは `q` の
             ものである。**その素動作が 1 対 1 に対応する
             ことは `前提 対応は素動作の粒度で読む` の 1 が与える。** A3 が縛るのは `result_prov`・
             `borrows_operand`・`applies_a_function_operand` の 3 つだけであり、D24 はこの retain と
             release を「この表に行を持たない素動作」と呼ぶので、A3 と A4 からは個数と対象が出ない。
             **参照カウントで分岐する腕がその代表である** -- 2 つの実行のカウントは `d(q, O)` だけ
             ずれるので、op がそれを読んで分岐すれば別の腕を取りうる。D21 の第 3 の箇条はその op の
             返り値と参照カウントへの変化を活性化の側のデータに置き、D29 がそれを 2 つの活性化に同じ
             ものとして与えるが、素動作の列まではその与件に入らない。対象のオブジェクトが対応することは、
             本命題の仮定の (a) と <1>1 よりオペランドの値が対応し、D29 の第 5 行より対応するオブジェクトが
             保持する値も対応することから出る。
        BY <1>1, 本命題の仮定, <ref id=e11772a/>, <ref id=3f1bb47/>, <ref id=c232680/>, <ref id=7218f92/>, <ref id=e3436e8/>, <ref id=f71c295/>, <ref id=4a0c14c/>, <ref id=2bb344b/>, 前提 対応は素動作の粒度で読む, DEF 節点の実行の素動作
      <4>3b. CASE 次の素動作が、`q` の実行に属さない段 -- 別の制御の流れの活性化の段、または
             環境の段 -- のものである。`α` はその段で節点を実行せず、変数を束縛しないので、値を得ている
             変数とその値の対応は動かない -- D29 の第 5 行が「**対応するオブジェクトが保持する値は
             対応する。**」を全単射の与件として置き、A17 は「**(ii-b) 環境が書き込むのは、計数下
             オブジェクト (D26) の inhabited な boxed leaf ではない。**」と述べる。`H` の動きは
             D29 の第 2 行が `α` に与える -- D21 は「**別の制御の流れの段による増減は、この活性化の
             外から来る**」と述べて、それを活性化の側のデータに数える。`Obl(α)` は動かず (`L49` の 2)、
             `d` も動かない (`DEF 欠損` の 2 つの数え上げが渡るのは `α` が実行した `Del` の
             節点の素動作だけである)。**その素動作が作る点に対応する `α'` の点が在ることは
             `前提 対応は素動作の粒度で読む` の 2 が与える** -- `DEF 対応する活性化の量` の突き合わせ
             規則は「対応する節点の実行が起こす素動作を順に」であって、`q` の実行に属さない段はその外に
             あるので、2 つの活性化の段の列の対応する位置に同じ並びで置くことをその前提が定める。
        BY <ref id=c9e4cca/>, <ref id=c232680/>, <ref id=7218f92/>, <ref id=e3436e8/>, <ref id=561fd05/>, 前提 対応は素動作の粒度で読む, DEF 欠損, DEF 対応する活性化の量, DEF 節点の実行の素動作
      <4>4. QED
        BY <4>1, <4>2, <4>3, <4>3a, <4>3b, <2>2
        <2>2 より、この節点の実行の点に在る素動作は 7 種のいずれかに属する。`q` の実行に属する段の
        6 種を <4>1 から <4>3a が尽くし、残る 1 種を <4>3b が扱う。
    <3>3. `τ` の次の点で、2 つの活性化の点が対応し、`α` がその点で D21 の制限を満たし、
          `H'(τ, O) = H(τ, O) - d(τ, O)` であり、`H = 0` と `H' = 0` が同値である。
      BY <3>2, <ref id=c6ed0c5/>, <ref id=965e588/>, <ref id=7218f92/>, 本命題の仮定, <1>1, <2>1, <2>2, <ref id=18416e9/>
      <3>2 より次の点でも 2 つの活性化の点は対応し、D29 の第 2 行 -- 「`α` の参照カウントの推移は、
      `α'` のそれに欠損 `k` (P21 (a)) を足したものとして与える」-- がその点で
      `H'(τ, O) = H(τ, O) - d(τ, O)` を与える。L43b の仮定 -- その点までに 2 つの活性化が `Del` の
      節点を除いて同じ節点を実行し、値を得ている各変数の値が対応すること -- は 本命題の仮定の (a) と <1>1 が
      与える。その点が `q_end` -- `q` が `ρ` の終端の `Ret` であるときの `q'` -- であるときは、
      L44e より D21 の制限はその点について何も課さないので、その節は空虚に成り立つ。それ以外の
      点では L44e より `α'` は対応する点で生きているので、L43c より `α` はその点で D21 の制限を満たす。
      <2>2 より `d(τ, O) = d(q, O)` である。`d(q, O) = 0` の
      とき 2 つのカウントは等しい。`d(q, O) ≥ 1` のとき、本命題の仮定より `q` は `Del` に入らない
      ので L41c が当たり、`H(τ, O) ≥ d(q, O) + 1 ≥ 2 > 0` かつ
      `H'(τ, O) = H(τ, O) - d(q, O) ≥ 1 > 0` である。**L41c を `α` について読むのに要る D21 の制限は、
      その点までの各点のものである** -- `q` の入口の点より前は本命題の仮定が、`q` の入口の点は
      <2>1 が、`q` の実行の途中の各点は帰納法の仮定とこの段が与える。
    <3>4. QED
      BY <3>1, <3>2, <3>3, <ref id=e3436e8/>
      **帰納が渡るのは各点までの接頭であって、素動作の列の全体ではない。** D24 は実行を「段の有限
      または無限の列」と定め、段内の点をその段の素動作の列の切れ目と定める。よって `q` の実行の各点に
      ついてその前にある素動作は有限個であり、その個数についての帰納が届く。**列そのものは有限とは
      限らない** -- 停止しない呼び出し先を持つ `App` の節点の実行は終わらず、無限の素動作を持ちうる。
      そのときも各点までの接頭は有限なので、この帰納法はどの点にも届く。
  <2>4. QED
    BY <2>1, <2>2, <2>3, <ref id=7218f92/>, DEF 欠損
    <2>3 が各点について 4 つ -- 点の対応、D21 の制限、`H'(τ, O) = H(τ, O) - d(τ, O)`、`H = 0` と
    `H' = 0` の同値 -- を与え、<2>2 が `d(τ, O) = d(q, O)` を与える。等式を与えるのは D29 の第 2 行で
    ある。
<1>2a. `q` の実行が `Obl` に与える変化は、D10 の行が定める分と、op の生成コードが出す retain が作った
       参照が `Obl` に在るあいだの分だけであり、後者は 2 つの実行で同じ素動作の対から来る。
  BY <ref id=78073d2/>, <ref id=9d3dd4d/>, <ref id=561fd05/>, <ref id=ba12699/>, <ref id=f06144e/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=e11772a/>, <ref id=3f1bb47/>, <1>1, <1>2, DEF 節点の実行の素動作
  `L47` の 1 より、この節点の実行に属する段の素動作は `DEF 節点の実行の素動作` の 6 種で尽き、その
  ほかにこの節点の実行の点に在るのは `q` の実行に属さない段の素動作だけである
  (`DEF 節点の実行の素動作`)。6 種のうち、D10 の行が
  定める作成と処分はそのままであり、割り当てはその生成と同じ素動作である。子の活性化の素動作と
  グローバル化は L39b より `Obl` を動かさず、(F) の解放は L39a より `Obl` を正味で動かさない。
  **`q` の実行に属さない段の素動作も `Obl` を動かさない** (`L49` の 2)。**第 6 の箇条のうち `Obl` を
  動かすのは、相殺する形の retain とその受け渡しの 2 つである** -- 相殺しない形の持ち手はその生成
  コードが書き込むオブジェクトの持ち手の単位なので `Obl` を動かさず (D24、D25 の 2 番目)、相殺する形の release は D9 の
  `Llvm` の行の消費である (D24)。その 2 つが 2 つの実行で同じ個数・同じ対象で起きることは、
  `前提 対応は素動作の粒度で読む` の 1 が与える (<1>2 の `<2>3` `<3>2` `<4>3a`)。
<1>3. QED
  BY <1>1, <1>2, <1>2a, 本命題の仮定, <ref id=f06144e/>, DEF 対応の 3 つ, DEF 欠損
  <1>1 と <1>2a より、この節点の実行は 2 つの実行で `Obl` を同じだけ同じ順に変える。
  <1>2 より `q` の実行の各点で 2 つの活性化の点が対応するので、対応する各点までに施された素動作は
  2 つの実行で同じであり (<1>1、<1>2a)、`Obl` と `Obl'` は `q` の入口の点からの増減が等しい。
  本命題の仮定の (c) と合わせて (c) が `q` の実行の各点で成り立つ。<1>1 より束縛する変数には対応する値が
  置かれるので (a) が `q'` で成り立ち、`q'` での (b) は <1>2 の等式である。`q` は `Del` に入らないので
  `d` は変わらない (DEF 欠損)。すなわち本命題の 1 は <1>1・<1>2 と上の (c) から、2 と 3 は <1>2 から
  出る。

### L44g (節点の間の点を越える対応) <!--#c445993-->

**言明** --- 対応する活性化 `α`、`α'` (D29) と、`ρ` の終端の `Ret` でない `ρ` の上の節点 `q` を取る。
`m` を `ρ` の上で `q` の次にある節点とし、`τ` を `q'` から `m` の入口の点までの各点 (両端と、そのあいだの
節点の間の点、`DEF 節点の入口の点`) とする。`q'` で `DEF 対応の 3 つ` が成り立つとすると、次の 4 つが
成り立つ。

1. 各計数下オブジェクト `O` について `d(τ, O) = d(m, O)` であり、各計数下の別名類 `C` について
   `held_ρ(τ, C) = held_ρ(m, C)` であり、`S(τ, O) = S(m, O)` である。
2. `τ` で 2 つの活性化の点が対応し、`DEF 対応の 3 つ` が `τ` で成り立つ。とくに `m` の入口の点で
   `DEF 対応の 3 つ` が成り立つ。
3. `α` は `τ` で D21 の制限 -- A19 (i) の不等式 -- を満たす。
4. 各計数下オブジェクト `O` について `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。

**この命題が要るのは、`q` の実行が終わった点と `m` の入口の点が同じ点とは限らないからである**
(`DEF 節点の入口の点`)。そのあいだの点で `α` は素動作を持たないが、環境の (E9) の段と別の制御の流れの
活性化の段は `H` を動かす -- D24 は「**環境が `H` を動かす道は (E9) の 1 つだけである。**」と述べる。

**証明**

<1>1. 1 が成り立つ。
  BY <ref id=941af96/>, <ref id=9d5d254/>, DEF 欠損, DEF 類ごとの義務, DEF 節点の入口の点
  `L50` の 4 より、`q'` から `m` の入口の点までのあいだに `α` の素動作は無く、挟まる段はどれも `α` が
  実行する段ではない。`DEF 欠損` の 2 つの数え上げが渡るのは `α` が `ρ` の上で実行した `Del` の節点に
  ついて D10 の行が定める素動作だけなので、`d` はそのあいだ動かない。D34 の表の 6 行と 3 つの開始行は
  どれも `α` の構文と `α` の事象を主語にするので、`held_ρ` も動かず、開始の時点もそのあいだには置かれ
  ない。よって `S(・, O)` も動かない。
<1>2. 2 が成り立つ。
  BY 本命題の仮定, <1>1, <ref id=7218f92/>, <ref id=c9e4cca/>, <ref id=561fd05/>, <ref id=f06144e/>, <ref id=941af96/>, <ref id=71d67e3/>, 前提 対応は素動作の粒度で読む, DEF 対応の 3 つ, DEF 対応する活性化の量
  **(a)。** `α` はそのあいだ節点を実行せず、変数を束縛しない (`L50` の 4) ので、値を得ている変数は
  `q'` のものと同じである。その値が対応することは D29 の第 5 行が全単射の与件として置き、A17 は
  「**(ii-b) 環境が書き込むのは、計数下オブジェクト (D26) の inhabited な boxed leaf ではない。**」と
  述べる。
  **(b)。** **点の対応を与えるのは `前提 対応は素動作の粒度で読む` の 2 である** -- そのあいだに
  挟まる段はどれも `α` が実行する段でも `α` が作った活性化とその子孫が実行する段でもない (`L50` の 4)
  ので、その前提が 2 つの活性化の段の列の対応する位置に同じ並びでそれを置き、その素動作が作る点が
  1 対 1 に対応する。その各点で `H = H' + d` であることは D29 の第 2 行が `α` の参照カウントの推移を
  `α'` のそれに `d` を足したものとして与えることによる。**この等式は `α` を決めるデータの一部で
  あって、証明が素動作ごとに積み上げる量ではない** (`L44` の脇)。
  **(c)。** `L49` の 2 より、`α` が実行する段でない段の素動作は `Obl(α)` を動かさない。**`α'` の側も
  同じ区間で `Obl(α')` を動かさない** -- `ρ'` の上で `q` に対応する節点の `q''` から `m` に対応する
  節点の入口の点までのあいだに `α'` の素動作は無く、挟まる段はどれも `α'` が実行する段ではないことを、
  `L50` の 4 を `ρ'` と `α'` について読んだものが述べ、`L49` の 2 を `α'` について読んだものが
  その素動作が `Obl(α')` を動かさないことを述べる。<1>1 より `d` も動かないので、`q'` の (c) が
  そのまま移る。
<1>3. 3 が成り立つ。
  BY <ref id=c6ed0c5/>, <ref id=18416e9/>, <1>2
  L43b の仮定は <1>2 の (a) が、`H(τ, O) = H'(τ, O) + d(τ, O)` は <1>2 の (b) が与える。`α'` が対応する
  点で生きていることは `L44e` が与える -- `q` は `ρ` の終端の `Ret` でないので、`τ` は `q_end` では
  ない。
<1>4. 4 が成り立つ。
  <2>1. `Σ_{C ∈ S(m, O)} bumps_ρ(m, C) ≥ d(m, O)` である。
    BY <ref id=dd86b2c/>, <ref id=948f840/>, <ref id=8093b68/>, DEF 類ごとの義務
    `L43a` の 1 より `d(m, O) = Σ_{C ∈ S(m, O)} d_C(m)` である。`DEF 類ごとの義務` より `d_C(m)` は、
    `CT` に属し `m` で pending である `Retain` 節点 `t` についての `B_ρ(m, e_t(m))` を `C` の名前に
    ついて足したものであり、`bumps_ρ(m, C)` は `pending(m)` の**すべての**要素について同じ和を取った
    ものである。P18b より残る項は 0 以上なので `bumps_ρ(m, C) ≥ d_C(m)` であり、`S(m, O)` を渡って
    足すと言明の不等式になる。
  <2>2. CASE `d(τ, O) ≥ 1`。このとき `H(τ, O) ≥ d(τ, O) + 1` である。
    <3>1. `Σ_{C ∈ S(m, O)} obl_ρ(m, C) + [S(m, O) に β(C) = 1 の類が在るならば 1] ≥ d(m, O) + 1` で
          ある。
      BY <ref id=3efe52c/>, <ref id=ca36627/>, <ref id=68112c9/>, <1>1, <2>1, 本場合の仮定
      <1>1 と本場合の仮定より `d(m, O) = d(τ, O) ≥ 1` なので、<2>1 より
      `Σ_{C ∈ S(m, O)} bumps_ρ(m, C) ≥ 1` である。よって `L41b` を節点 `m`、その入口の点、
      オブジェクト `O`、部分集合 `S = S(m, O)` について読める。`m` の入口の点が終端の `Ret` の消費より
      前にあることは、D3 と L33a より `ρ` の最後の節点が終端の `Ret` であることから出る。
    <3>2. QED
      BY <1>1, <1>3, <3>1, <ref id=c232680/>, <ref id=9f1cf6c/>, DEF 類ごとの義務, DEF 実行時の量
      <1>3 より `α` は `τ` で D21 の制限を満たすので、A19 (i) より
      `H(τ, O) ≥ Σ_{C ∈ S(τ, O)} obl_ρ(τ, C) + [S(τ, O) に β(C) = 1 の類が在るならば 1]` である。
      <1>1 より `S(τ, O) = S(m, O)` であり `held_ρ(τ, ・) = held_ρ(m, ・)` なので `obl_ρ` の値も等しく、
      右辺は <3>1 の左辺である。<1>1 より `d(τ, O) = d(m, O)` なので、<3>1 が
      `H(τ, O) ≥ d(τ, O) + 1` を与える。
  <2>3. CASE `d(τ, O) = 0`。<1>2 の (b) より `H'(τ, O) = H(τ, O)` なので、4 は成り立つ。
    BY <1>2
  <2>4. QED
    BY <2>2, <2>3, <1>1, <1>2, <ref id=4ff3e8d/>
    `L43` より `d(m, O) ≥ 0` であり、<1>1 より `d(τ, O) = d(m, O)` なので、場合は <2>2 と <2>3 で
    尽きる。<2>2 の場合、`H(τ, O) ≥ d(τ, O) + 1 ≥ 2 > 0` であり、<1>2 の (b) より
    `H'(τ, O) = H(τ, O) - d(τ, O) ≥ 1 > 0` なので、どちらも 0 でない。
<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### L44 (2 つの実行の対応) <!--#329a0ee-->

**言明** --- `ρ` を `B` の実行路、`ρ'` をそれに対応する `B'` の実行路、`α'` を `ρ'` を辿る `B'` の
活性化、`α` を D29 が `α'` に対応させる `B` の活性化とする。`ρ` の上の各節点 `q` と各計数下オブジェクト
`O` について次の 6 つが成り立つ。`ρ` の最後の点 `q_end` (`DEF 節点の入口の点`) についても同じである。

**`B` が `borrow_ify` の出力の本体であることを読む。** `L44p`・`L44a`・`L44b`・`L44c` が読む `L41`・`L41c`・`L41d` は P18a を、
`L41c`・`L41d` が読む `L41b`・`L41a` は A19 の (ii-a)・(ii-b)・(ii-c) と P14a を `α` に当てる。その範囲が
`borrow_ify` の出力の本体だからである (第 1 節)。**(ii-c) の `borrow_ify` の側は
`前提 (ii-c) の保存` が置く。**
**入力プログラムが D12 を満たすことは、この命題のどの段も読まない。**

- **(a)** `α` と `α'` は、`q` の入口の点までに、`Del` の節点を除いて同じ節点を実行し、その点までに値を
  得ている各変数の値は、D29 の全単射のもとで対応する。**対応する 2 つの値の Fix の関数型の成分は、
  入力と出力の対応する版を名指す。** ここで入力の関数と出力の関数が**対応する版**であるとは、P24 の
  第 2 の箇条が言う「**出力の各関数は入力のちょうど 1 つの関数から作られ**」の関係にあること、すなわち
  `cancel` が入力の `f` から作った出力の関数であることをいう。`L44d` より対応する 2 つの版は同じ
  `name` を持つので、`prog.funcs` の同じ鍵が対応する版を引く。
- **(b)** `H'(q, O) = H(q, O) - d(q, O)` である。
- **(c)** `q` の実行の各点 `τ` について `Obl'(τ, O) = Obl(τ, O) - d(τ, O)` である。とくに `q` の入口の
  点で `Obl'(q, O) = Obl(q, O) - d(q, O)` である。
- **(d)** `d(q, O) ≥ 1` ならば `H(q, O) ≥ d(q, O) + 1` である。
- **(e)** `q` の実行の各点 `τ` -- `q` の入口の点と、その実行が終わった点 `q'` を含む
  (`DEF 節点の入口の点`) -- と、`q'` から `ρ` の上で `q` の次にある節点の入口の点までの各点 (節点の間の
  点) について、2 つの活性化の点が対応し、`H'(τ, O) = H(τ, O) - d(τ, O)` であり、
  `H(τ, O) = 0` と `H'(τ, O) = 0` は同値である。`q ∉ Del` のときは `q` の実行の各点で
  `d(τ, O) = d(q, O)` である (DEF 欠損)。
- **(f)** `α` は、`q` の入口の点までの各点と `q` の実行の各点と、その後の節点の間の点で、D21 の制限 --
  A19 (i) の不等式 -- を満たす。

**(e) と (f) が `ρ` の全点を覆うのは、`DEF 節点の入口の点` の「`ρ` の上の点は、節点の実行の点と節点の
間の点で尽きる」による。** 節点の間の点を扱うのは `L44g` である。

**とくに `α` はその全点で D21 の制限を満たし、D21 の意味の活性化である。** (f) を `ρ` の全節点について
集めたものであり、`<1>2` がそれを述べる。

**(b) と (e) の等式は D29 の与件である。** D29 の第 2 行は「`α` の参照カウントの推移は、`α'` のそれに
欠損 `k` (P21 (a)) を足したものとして与える」と述べ、L43 の最後の文よりその `k` は `DEF 欠損` の `d` で
ある。`H` は活性化の側のデータなので (D21)、これは証明が素動作ごとに積み上げる量ではなく、`α` を決める
データの一部である。**この命題が示すのは、その与件を読む先 -- 2 つの活性化の点の対応
(`DEF 対応する活性化の量`) -- が `ρ` の全体にわたって定まることと、(c)(d)(e)(f) である。**

**与件は 2 つの活性化の自分の素動作と整合する。** D21 より、活性化自身の事象は D10 の表のとおりに `H` を
動かし、それ以外の増減はこの活性化の外から来る。対応する 2 つの点の間に 2 つの活性化が実行する自分の
素動作は `Del` の節点の分だけ違い ((a))、`Del` の `Retain`/`Release` 節点が D10 の表によって `O` の `H` に
与える増減の総和は `DEF 欠損` の `d(・, O)` の増減そのものである。すなわち `H = H' + d` が言うのは、
2 つの活性化の外から来る増減が対応することであり、D29 はそれを `α` に与える。

**(f) が要る所。** D21 は制限を満たすものだけを活性化とするので、`α` について A19 の
(ii-a)・(ii-b)・(ii-c)、P14a、P18a を読む段はこれに立つ。**この 5 つはどれも点ごとの言明であり、点 `τ` に
ついての結論が結ぶのは `τ` までの量 -- その点の `H`・`Obl`・`held`・`bumps` と、そこに至る `ρ` の接頭が
決める走査の状態 -- である。** 量化する点は 3 通りに分かれる -- A19 の (ii-a)・(ii-b) と P18a は
「各実行路と、それを辿る各活性化の各時点」であり、A19 はその「各時点」を「その活性化が生きている
(D23) 間の、その活性化の節点の訪問の入口である時点」に限る。A19 の (ii-c) は「節点の実行の途中の各点
(D24 の段内の点)」である。P14a は「各時点と各段内の点 (D24)」である (P14a の脇)。よってこの命題は
(f) を点ごとに
示し、`q` の入口の点までの制限の上で `q` についてこれらを読む。README の第 2 節が「**各時点についての
言明は、その時点までの接頭で読む。**「各実行路と、それを辿る各活性化の各時点について …」の形の言明 --
A19 (ii-a)、A19 (ii-b)、P14a、P18a、P18b、P18c、D11 の 3 つの節 -- は、時点 `τ` について `τ` までの量
だけを結ぶ」と定めるのがこの読みであり、README の P21 の脇が「位置ごとの帰納にするのはこのためであり、
各位置で (a) から `α` の許容性を出してから (b) を出す」と述べるのがこの形である -- README がそこで
「位置」と呼ぶのは、この文書の節点の入口の点である。**A19 (ii-c) はその一覧に載っていないが、同じ
読みが当たる** -- A19 は「**(ii-c) (段内の点の非負性)。節点の実行の途中の各点 (D24 の段内の点) と、
その点で `held_ρ` が定まる各計数下の別名類について、`held ≥ 0` である。**」と、点ごとの言明として
置き、その点の `held` のほかに何も結ばない。この文書がその点で (ii-c) を読むのは `L41a` の
`<1>2` である。

**証明**

<1>1. `ρ` の上の各節点の入口の点と `ρ` の最後の点 `q_end` について `DEF 対応の 3 つ` が成り立つことを、
      `ρ` の上の節点についての帰納法で示す。D3 より `ρ` は有限の列なので整礎である。各節点では
      `L44p` が (d) と (f) の入口の点の分を出し、`L44a`・`L44b`・`L44c` が (e) と (f) の残りと、
      次の節点の `DEF 対応の 3 つ` を出す。
  <2>1. 基底。`ρ` の最初の節点は `B` の根であり、その入口の点は `α` が生きている活性化 (D23) になる点で
        ある。DEF 欠損 より `d = 0` である。どちらの活性化も
        まだ節点を実行しておらず、D29 より 2 つの活性化はパラメータと capture に対応する値を受け取るので
        (a) が成り立つ。(a) の第 2 文もその与件の一部である -- D29 の第 1 行は「入力の束縛 (D23) が
        受け取る値は対応する。これは定義の与件である。」と述べ、その「対応する」が D29 の置く 1 つの
        全単射のもとでの対応であることは、D29 が「**値とオブジェクトは 2 つの活性化が別々に割り当てる。**
        「同じ値」「同じオブジェクト」と言うとき指すのは、次の 4 つの行が生成する全単射のもとで対応する
        ものである。」と書くことによる。**その全単射のもとで、Fix の関数型の成分について「対応する」が
        何を意味するかを定めるのが第 5 行の但し書きである** -- 「**funptr の番地は等号では
        読めない** -- 2 つのプログラムは別々にコンパイルされるので、同じ関数の番地が同じ値になるとは
        限らない。」に続けて「**その成分については、対応する 2 つの番地が対応する版を名指す**」と述べる。
        **その但し書きの本文はオブジェクトが保持する値についての行に置かれているが、定める先は全単射の
        読み方であり、D30 も「段が作る値・呼び出し先を決める値・記憶域へ格納する値のいずれについても
        同じく読む。」として同じ読みを局所の値へ広げる。**
        D29 の第 2 行より最初の点では `k = 0` であり、2 つの活性化は対応する各計数下
        オブジェクトについて等しいカウントで始まるので (b) が成り立つ。D10 の初期値は所有する unit の
        下の inhabited な leaf で決まり、`L44d` より割り当ては同じなので `Obl'` は `Obl` に等しく、(c) が
        成り立つ。
    BY DEF 欠損, <ref id=7218f92/>, <ref id=081e39f/>, <ref id=f06144e/>, <ref id=ff5985d/>, <ref id=1dbc8d1/>
  <2>2. **帰納法の仮定。** `ρ` の上の節点 `q` を取り、`q` の入口の点で `DEF 対応の 3 つ` が成り立ち、
        **`q` の入口の点より前の `α` の各点で `α` が D21 の制限を満たす**とする。基底の節点について
        これを与えるのは <2>1 であり (根の入口の点より前に `α` の点は無いので後半は空虚に成り立つ)、
        それ以外の節点については、`ρ` の上でその直前にある節点について本帰納法が示したもの --
        その節点の `q'` での `DEF 対応の 3 つ` を `L44g` の 2 が `q` の入口の点へ運んだもの、および
        その節点の実行の各点と その後の節点の間の点での D21 の制限 (`L44a`・`L44b`・`L44c` の 3 と
        `L44g` の 3) -- である。
        **この帰納が渡る点は `ρ` の上の節点の入口の点と `q_end` である** -- `DEF 節点の入口の点` より
        `q_end` は `ρ` の終端の `Ret` の `q'` であり、その `Ret` についての本帰納法が
        そこで `DEF 対応の 3 つ` を与える。
    BY 帰納法の仮定, <2>1, <ref id=c445993/>, <ref id=fdadbd9/>, <ref id=dcf3c88/>, <ref id=45214bd/>, <ref id=ff5985d/>, DEF 節点の入口の点, DEF 対応の 3 つ
  <2>3. CASE `q` の節点が `Del` の `Retain` 節点である。`L44a` の 1 と 2 が、`q` の実行の各点で
        (a)・(c)・(e) が成り立つことと、`q'` で `DEF 対応の 3 つ` が成り立つことを与え、`L44a` の 3 が
        その各点で (f) を与える。`q` の入口の点の (d) と (f) は `L44p` の 2 と 1 が与える。`q` が
        `ρ` の終端の `Ret` でないとき、`L44g` が `q'` の後の節点の間の点で (e) と (f) を与え、`ρ` の
        上で `q` の次にある節点の入口の点へ `DEF 対応の 3 つ` を運ぶ。
    BY <2>2, <ref id=fdadbd9/>, <ref id=efe22a7/>, <ref id=c445993/>, DEF 対応の 3 つ
    `L44a` と `L44p` の 2 つの仮定 -- `q` の入口の点で `DEF 対応の 3 つ` が成り立つことと、`q` の入口の
    点より前の `α` の各点で `α` が D21 の制限を満たすこと -- は <2>2 が与え、`L44g` の仮定 -- `q'` で
    `DEF 対応の 3 つ` が成り立つこと -- は `L44a` の 1 が与える。
  <2>4. CASE `q` の節点が `Del` の `Release` 節点である。`L44b` の 1・2・3 が同じものを与え、`L44p` と
        `L44g` が <2>3 と同じものを与える。2 つの命題の仮定は <2>3 と同じく <2>2 と `L44b` の 1 が
        与える。
    BY <2>2, <ref id=dcf3c88/>, <ref id=efe22a7/>, <ref id=c445993/>, DEF 対応の 3 つ
  <2>5. CASE `q` の節点が `Del` に入らない。`L44c` の 1・2・3 が同じものを与え、`L44p` と `L44g` が
        <2>3 と同じものを与える。2 つの命題の仮定は <2>3 と同じく <2>2 と `L44c` の 1 が与える。
    BY <2>2, <ref id=45214bd/>, <ref id=efe22a7/>, <ref id=c445993/>, DEF 対応の 3 つ
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <ref id=efe22a7/>, <ref id=4ff3e8d/>, <ref id=7d5a1de/>, DEF 節点の入口の点
    `q` の節点は `Del` の `Retain`、`Del` の `Release`、`Del` に入らないもののいずれかである
    (L32 の 5)。この 3 つを <2>3・<2>4・<2>5 が尽くし、どれも `q'` での `DEF 対応の 3 つ` を出して
    `L44g` がそれを次の節点の入口の点へ運ぶので、帰納は次の節点へ進む。**節点を持たない `q_end` は
    この帰納の最後の点である** -- L43 の最後の文よりそこで `d(q_end, O) = 0` なので、<2>2 の (b) から
    `H'(q_end, O) = H(q_end, O)` であり、(e) の 2 つの言明 -- 等式と、`H = 0` と `H' = 0` の同値 --
    はどちらもその等式から出る。点の対応は <2>2 の (a) が与える。その点の (d) と (f) は `L44p` が
    与える。
    **(f) の「`q` の入口の点までの各点」の半分は、節点についての帰納がそのまま与える。**
    `DEF 節点の入口の点` より、`q` の入口の点より前の各点は `ρ` の上で `q` より前にある節点の実行の点か、
    その後の節点の間の点であり、前者についてはその節点についての <2>3・<2>4・<2>5 が、後者については
    `L44g` の 3 が (f) をそこで与えている。`q` の入口の点自身については `L44p` の 1 である。
<1>2. QED
  BY <1>1, <ref id=ca36627/>, <ref id=c232680/>, <ref id=7218f92/>, <ref id=efe22a7/>, <ref id=fdadbd9/>, <ref id=dcf3c88/>, <ref id=45214bd/>, <ref id=c445993/>, DEF 節点の入口の点
  <1>1 が `ρ` の各節点について (a) から (f) を与える。D3 より `ρ` の節点は有限個であり、
  `DEF 節点の入口の点` より `ρ` の上の点は節点の実行の点と節点の間の点で尽きるので、(f) を
  すべての節点について集めると `α` はその全点で D21 の制限を満たす。すなわち `α` は D21 の意味の
  活性化であり、D29 が「`α'` に対応する `B` の活性化はちょうど 1 つ存在する」と述べるのはこれに立つ。


## 4. P18c (義務集合の側の同じ不等式)

**言明 (README)** --- **`borrow_ify` の出力を入力とし**、走査中の各位置と各実行路について、各計数下
オブジェクト `O` について `Obl(O) ≥ n(O)` である。ここで `Obl(O)` は義務集合が持つ `O` への参照の個数、
`n(O)` は P18a のものである。

**証明する形**。示すのは、**`borrow_ify` の出力**の各本体 `B` の各実行路 `ρ` と、それを辿る各活性化
(D21) と、`ρ` の上の各節点 `q` と各計数下オブジェクト `O` について `Obl(q, O) ≥ N(q, O)` である。
**入力を `borrow_ify` の出力に限るのは、P21・P23 と同じ理由による** -- `<1>1` が読む L42 は L41a を経て
A19 の (ii-a)・(ii-b)・(ii-c) と P14a を活性化に当てるので、その範囲を出られない。A19 の (ii-b) の範囲は
「`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体」、P14a の範囲は「`borrow_ify` の
出力の各本体」であり、この文書では第 1 節が `optimize_rc_program` からその限定を出している。(ii-c) の
`borrow_ify` の側は `前提 (ii-c) の保存` が置く。**この限定は主定理の鎖を切らない** -- P18c を読むのは
`cancel` の健全性の鎖であり、その入力はまさに `borrow_ify` の出力だからである。README の P18c の言明も
「**`borrow_ify` の出力を入力とし**、」で始まる。

活性化が D21 の制限 (A19 (i) の不等式) を満たすことは、D21 が制限を満たすものだけを活性化とすることから
出る -- ここで量化するのは `B` の活性化そのものであって、`L44` が構成する入力側の活性化ではない。

**証明**

<1>1. QED
  BY <ref id=0669029/>, <ref id=217aebd/>, <ref id=ca36627/>, <ref id=68112c9/>, <ref id=b3dfa37/>, DEF N, <ref id=8093b68/>, <ref id=c232680/>, DEF 節点の入口の点
  走査中の位置は D27 に従って節点の訪問の入口であり、**その位置の P18a の `n(O)` が `N(q, O)` で
  あることは `L41` の 1 が述べる**。**節点の入口は終端の `Ret` の消費より前にある** -- D3 と L33a より
  `ρ` の最後の節点は終端の `Ret` であり、それより前の節点の入口の点はその実行より前にある。終端の
  `Ret` 自身の入口の点も、`DEF 節点の入口の点` よりその消費を行った直後の点 `q_end` より前にある。
  よって `q` は L42 の範囲に入り、L42 が
  `Obl(q, O) ≥ b(q, O) = N(q, O)` を述べる。L42 を読むのに要る D21 の制限は、量化する活性化が D21 の
  意味の活性化であることから出る。

## 5. P19 (削除される retain の性質)

**言明 (README)** --- `cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含むすべての実行路に
おいて、`t` より後にある、**その位置での** `t` の `outstanding` の位置 (`VarPath`) を `acted_on` に含む
消費より前、かつ終端の `Ret` より前に、削除される `Release` 群が `t` の `outstanding` を空にする。さらに、
`t` とともに削除される各 `Release` は、実行路の上で `t` より後ろにある。

**証明する形**。「**その位置での** `t` の `outstanding`」は `out(t, ・)` (DEF 訪問) である。示すのは次の
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
  BY <ref id=7d5a1de/>
<1>2. 1 が成り立つ。
  BY <ref id=5adaf7f/>, <ref id=7d5a1de/>, <ref id=23ac734/>, <1>1
  L38 の 3 が `R_ρ(t) ⊆ un_bump_releases[t] ⊆ Del` と静的な収支を与え、L38 の 1 が `n*(ρ)` の訪問で
  `outstanding` が空になることを与える。L37 より `R_ρ(t)` の各要素は `ρ` の上で `t` より後にある。
<1>3. 2 が成り立つ。
  BY <ref id=5adaf7f/>, <1>1
<1>4. 3 が成り立つ。
  <2>1. そのような消費点 `c` があると仮定する。`t` が `c` で pending なので、L38 より `c ∈ I_ρ(t)` で
        ある。
    BY 仮定, <ref id=5adaf7f/>
  <2>2. CASE `c` が D9 の消費の表の `App`、`Closure`、`Llvm` の行の節点である。この節点は右辺が `Match`
        でない `Let` 節点であり、`c` の訪問は `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
    <3>1. `consume_rhs` が `rhs_consumes` に渡す `owns(p, leaf)` は
          `self.owned_units.contains(&(p.name, truncate_to_unit(&p.ty, leaf, self.type_env)))` であり、
          `self.owned_units` は `all_owned_units(prog, type_env)` の値である。`all_owned_units` は各関数の
          各パラメータ・capture の各 unit のうち `borrowed_units` に入らないものを集めるので、この述語が
          真であることは、呼び出し先がその leaf の unit を所有する (D14) ことに等しい。unit は
          呼び出し先のパラメータの型 `p.ty` で取る。**この同値には 3 つが要る。** `all_owned_units` は
          **全関数**のパラメータ・capture の unit を 1 つの `Set<VarPath>` に集め、鍵は
          `(p.name, unit)` なので、(i) **パラメータ名がプログラム全体で一意**でなければ、別の関数の
          同名のパラメータの unit がこの述語を真にしうる。それを与えるのは A6 -- 「`borrow_ify` の入力の
          すべての束縛変数の名前は相異なり、**どの関数の名前とも異なる**」-- と、出力についての同じ
          性質を与える P9 である。**パラメータがそこでいう束縛変数に数えられることは P2 が書き出して
          いる** -- 「**「プログラムの束縛変数」は、節点が束縛する変数と、その本体のパラメータ・capture の
          両方である。**」であり、D6 も「`VarTable::of` と `VarTable::body_only` がその表に入れる鍵は、
          パラメータ・capture の名前と節点が束縛する変数の名前だけで、どれも `Lowerer::fresh_var` が
          `FullName::local` で作ったものである」として同じ 2 種を挙げる。(ii) `leaf` は `p.ty` の
          boxed leaf でなければ `truncate_to_unit` が
          型に合わない path を歩いて panic する。`rhs_consumes` が渡す `leaf` は引数の型の boxed leaf で
          あり、A12 の「**`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型**」がその型を
          `p.ty` に一致させる。(iii) `truncate_to_unit(p.ty, leaf)` の値が `rc_units(p.ty)` の元で
          なければ、`all_owned_units` が集めた鍵のどれとも一致しない。それを与えるのは P1 である --
          「**A10 を満たす**任意の型 `τ` について、`boxed_leaf_paths(τ)` の各 leaf の
          `truncate_to_unit(τ, ・)` は `rc_units(τ)` の要素であり、`rc_units(τ)` の各 unit はある leaf の
          `truncate_to_unit(τ, ・)` である。」。
      BY <ref id=596a46d/>, <ref id=ef8efc4/>, <ref id=33c54dc/>, <ref id=83d98e9/>, <ref id=8412761/>, <ref id=3597669/>, <ref id=0edb0ba/>, <ref id=63eadd9/>,
         CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs, CODE src/rc_ir/borrow.rs: cancel,
         CODE src/rc_ir/ownership.rs: all_owned_units,
         CODE src/rc_ir/ownership.rs: truncate_to_unit, CODE src/rc_ir/ownership.rs: rhs_consumes
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
          **この段が与えるのは包含だけである。** この場合の QED が使うのは「`c` で消費される leaf は
          `rhs_consumes` がこの節点で報告する」という向きだけであり、逆向きは要らない。
      BY <ref id=9d74736/>, <ref id=ff5985d/>, <ref id=4517a7a/>, <ref id=f8ae607/>, <ref id=561540d/>, <3>1,
         CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: resolve_callee_params
    <3>3. `rhs_consumes` は `RcRhs::Closure(_, caps)` について各 capture の全 boxed leaf を報告する。
          これは D9 の `Closure` の行である。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/ownership.rs: push_boxed_leaves, <ref id=9d74736/>
    <3>4. `rhs_consumes` は `RcRhs::Llvm(llvm_gen, args)` について、`borrows_operand(i)` が偽の
          オペランドの boxed leaf のうち `passthrough_arg_leaves` に入らないものを報告する。
          `passthrough_arg_leaves` は、結果のある leaf の宣言が単一の `Arg(i, σ)` であるような
          `(i, σ)` の全体である。これは D9 の `Llvm` の行である。
      BY CODE src/rc_ir/ownership.rs: rhs_consumes,
         CODE src/rc_ir/ownership.rs: passthrough_arg_leaves,
         CODE src/rc_ir/provenance.rs: Provenance::leaves,
         CODE src/rc_ir/leaf_map.rs: LeafMap::leaves,
         CODE src/rc_ir/ownership.rs: as_arg_projection, <ref id=9d74736/>
      `passthrough_arg_leaves` は `decl.leaves().filter_map(as_arg_projection)` を集める。
      `Provenance::leaves` の本体は `self.0.leaves()` の 1 つの呼び出しであり、`LeafMap::leaves` の
      本体は `self.0.values()` である。すなわち渡るのは、`decl` の写像が結果の各 boxed leaf に据えた
      `LeafOrigins` の全体であって、鍵は落ちる。`as_arg_projection` はそのうち単一の
      `LeafOrigin::Arg(i, σ)` であるものに `Some((i, σ))` を返すので、集まるのは `(i, σ)` の全体で
      ある。
    <3>5. QED
      BY <3>1, <3>2, <3>3, <3>4, <ref id=7855e90/>
      仮定より `c` で消費される leaf `(w, μ)` は D9 のこの 3 行のいずれかが名指すものなので、<3>2 から
      <3>4 よりそれは `rhs_consumes` がこの節点で報告する leaf である。L36 の 1 より `c` の訪問は
      `consume_objects(pending, acted_on(w, μ))` を呼ぶ。
  <2>3. CASE `c` が D9 の消費の表の `Destructure` の 2 行の節点である。`destructure_consumes(container,
        fields, type_env)` は、容器が boxed のとき容器の全 boxed leaf を、unbox のとき名前の付いていない
        フィールドの leaf を返す。これは D9 の `Destructure` の 2 行そのものである。L36 の 2 より、`c` の
        訪問はその各 leaf `μ` について `consume_objects(pending, acted_on(container.name, μ))` を呼ぶ。
    BY CODE src/rc_ir/ownership.rs: destructure_consumes, <ref id=0594f24/>, <ref id=9d74736/>, <ref id=7855e90/>
  <2>4. CASE `c` が D9 の消費の表の「本体 (D23) の終端の `Ret(x)`」の行の節点である。L38 より `t` は `ρ` の
        終端の `Ret` では pending でないので、仮定に反する。よってこの場合は起こらない。
    BY <ref id=5adaf7f/>
  <2>5. QED
    <2>2 と <2>3 の呼び出しは、`I_ρ(t)` に入る節点の訪問の中で、由来が `t` の要素が走査の `pending` に
    在るところで走る。仮定よりその `objects` は `out(t, c)` が名指す名前を含むので、L38 の 4 に反する。
    D9 の消費の表の 6 行を <2>2、<2>3、<2>4 が尽くす。よって <2>1 の仮定は成り立たない。L38 より `t` が
    pending である区間は `n*(ρ)` で終わるので、`t` より後にあってそのような名前を名指す消費点は
    `n*(ρ)` より後にある。
    BY <2>1, <2>2, <2>3, <2>4, <ref id=5adaf7f/>, <ref id=9d74736/>
<1>5. 4 が成り立つ。
  BY <ref id=23ac734/>, <1>1
  L37 の前件は `t ∈ CT` であり、<1>1 がそれを `t ∈ Del` が `Retain` 節点であることから与える。
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
    BY <ref id=7d5a1de/>
  <2>2. `r ∈ un_bump_releases[t]` が `ρ` の上にあるならば、L37 より `t` も `ρ` の上にあって `r` より前に
        あり、`t` は `r` で pending である。よって `r ∈ I_ρ(t)` であり、L32 の 4 より `r` の訪問の
        `un_bump` は `InBracket(t)` を返すので、L38 の 3 より `r ∈ R_ρ(t)` である。
    BY <ref id=23ac734/>, <ref id=5adaf7f/>, <ref id=7d5a1de/>
  <2>3. 逆に `t` が `ρ` の上にある `CT` の要素ならば、L38 の 3 より `R_ρ(t)` の各要素は `Del` の
        `Release` 節点であり、`I_ρ(t)` に入るので `ρ` の上にある。
    BY <ref id=5adaf7f/>
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>2. `ρ` の上にある各 `t ∈ CT` について、`t` が `ρ` で実際に作る参照の多重集合は、`R_ρ(t)` の各要素が
      `ρ` で実際に処分する参照の多重集合の和に等しい。
  BY <ref id=2307c1e/>
<1>3. QED
  BY <1>1, <1>2, <ref id=7d5a1de/>
  `ρ` の上にある `Del` の `Retain` 節点は `ρ` の上にある `CT` の要素の全体である -- L32 の 3 より `Del` は
  `CT` と `⋃_{t ∈ CT} un_bump_releases[t]` の非交和であり、L32 の 2 より後者の要素は `Release` 節点なので、
  `Retain` 節点である `Del` の要素は `CT` の側に在り、逆に `CT` の要素は L32 の 3 より `Del` の要素で
  ある。<1>2 をそのすべてについて足し、<1>1 で右辺をまとめると言明の等式になる。

## 7. P21 (削除がカウントと解放に与えるもの)

**言明 (README)** --- **`borrow_ify` の出力**を入力とし、それが D12 を満たすとき、`cancel` の出力の
各活性化 `α'` と、それに対応する入力の活性化 `α` (D29) について、次の 2 つが成り立つ。

- **(a)** 各位置において、各計数下オブジェクト `O` について `H_{α'}(O) = H_α(O) - k(O)` である。
  `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` が `O` に作った参照のうち、**その `Retain`
  と対になる削除済みの `Release` がまだ処分していないもの**の個数である。
- **(b)** 2 つの活性化は同じ位置で同じオブジェクトを解放する。とくに各読む構文の各位置で解放されて
  いるオブジェクトの集合は等しい。

**証明する形**。入力プログラムを `P` と書く。**`P` は `borrow_ify` の出力であり、D12 を満たすとする。**
`ρ'` を `B'` の実行路、`α'` をそれを辿る `B'` の活性化、`ρ` を `ρ'` に対応する `B` の実行路、`α` を
D29 が `α'` に対応させる `B` の活性化とする。`k(O)` を `Retain` の個数ではなく参照の個数で数えることが
要る理由は、第 11 節が反例で述べる。**言明の「各位置」は、この文書では `ρ` の上の各節点の入口の点と
`ρ` の最後の点 `q_end` である** (`DEF 節点の入口の点`)。(b) はさらに細かく、`α` の各点について示す --
「各読む構文の各位置で解放されているオブジェクトの集合は等しい」が見るのは読みの直前の点であり、それは
節点の入口の点ではない (D24)。

**この 2 つが要る所は次のとおりである。** `borrow_ify` の出力であることは `<1>1` と `<1>1a` が読む
`L44` に要る -- `L44` が読む `L41`・`L41c`・`L41d` が P18a を、`L41c` と `L41d` が読む `L41b`・`L41a` が
A19 の (ii-a)・(ii-b)・(ii-c) と P14a を `α` に当てるので、その範囲が要る。(ii-c) の `borrow_ify` の側は
`前提 (ii-c) の保存` が置く。この文書では `B` が `borrow_ify` の出力の
本体であることを第 1 節が `optimize_rc_program` から出しており、この節はそれを言明の側にも書き出す。
D12 は README の言明が置く前提であり、(a) と (b) の導出はどの段もそれを読まない -- `DEF 解放されている`
が「`τ` かそれより前の点で `O` の参照カウントが 0 であること」の形で解放を定めるので、(b) は
カウントが後から上がる場合も覆う。**D12 を読むのは第 9 節の `<1>3` である** -- そこで `α` について D11 の 3 つを得る。

**`α` が D21 の制限を満たすことも、この命題の証明が示す** (README の P21 の脇)。`<1>1a` がそれである。

**証明**

<1>1. (a) が成り立つ。
  BY <ref id=329a0ee/>, <ref id=4ff3e8d/>
  L43 の最後の文より P21 (a) の `k(O)` は DEF 欠損 の `d(q, O)` であり、L44 の (b) がその等式である。
<1>1a. `α` は D21 の制限 -- A19 (i) の不等式 -- をその全点で満たす。すなわち `α` は D21 の意味の
       活性化であり、D29 が「`α'` に対応する `B` の活性化はちょうど 1 つ存在する」と述べるのはこれに
       立つ。
  BY <ref id=329a0ee/>, <ref id=c232680/>, <ref id=7218f92/>
  L44 の言明の「**とくに `α` はその全点で D21 の制限を満たし、D21 の意味の活性化である。**」がこれで
  ある。
<1>2. 計数下のオブジェクトについて、2 つの活性化は同じ点で同じオブジェクトを解放する。ここで `O` は
      D29 の全単射が対応させるオブジェクトを渡る。
  <2>0. どちらかの活性化が解放するオブジェクトは、その全単射の定義域に入る。
    BY <ref id=7218f92/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=f06144e/>, <ref id=596a46d/>
    D29 の第 5 行より、その定義域は対応する 2 つの活性化がそれぞれ到達できる (D25) オブジェクトの
    全体である。**`α` が解放するオブジェクトは `α` が到達できるオブジェクトである** -- D24 の (F) が
    解放を起こすのは参照の処分であり、`α` の素動作が処分する参照は、D10 の行が名指す leaf のもの --
    その leaf は `α` の位置 (D6) のものである -- か、既に解放を始めたオブジェクトが保持する参照の
    もの (D24 の (F)) である。前者が指すオブジェクトは `α` の位置から届き、後者はその前者から D25 の
    意味で到達できる。
  <2>1. `α` が点 `τ` で `O` を解放するのは、`H(τ, O) = 0` となり、`τ` より前の点ではそうでないときで
        ある。`α'` についても `H'` で同じである。
    BY <ref id=56c2068/>, DEF 解放されている, DEF 節点の入口の点, <ref id=7218f92/>, <ref id=0b850c9/>
  <2>2. QED
    BY <2>0, <2>1, <ref id=329a0ee/>, DEF 節点の入口の点
    `DEF 節点の入口の点` より `ρ` の上の点は節点の実行の点と節点の間の点で尽きる。L44 の (e) は
    その両方について `H(τ, O) = 0` と `H'(τ, O) = 0` の同値を述べるので、<2>1 の条件は 2 つの活性化で
    同時に成り立つ。
<1>3. グローバル状態のオブジェクト (D26) は 2 つの活性化のどちらでも解放されない。
  BY <ref id=88a06de/>, <ref id=b6673ca/>
<1>4. (b) が成り立つ。
  BY <1>2, <1>3, DEF 解放されている, <ref id=329a0ee/>, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=7218f92/>, <ref id=88a06de/>
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
  BY <ref id=4a0c14c/>, <ref id=7d5a1de/>, <ref id=2bb344b/>
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
`L41c` に要る -- `L41c` が読む `L41b`・`L41a` が P14a と A19 の (ii-a)・(ii-b)・(ii-c) を読み、どれも
範囲が `borrow_ify` の出力の本体だからである (第 1 節)。**`<1>3`・`<1>4`・`<1>5`・`<1>6`・`<1>7` が読む
`L44` も同じ範囲を要る** -- `L44` は自分の脇で、それが読む `L41`・`L41c`・`L41d` が P18a を、
`L41c` と `L41d` が読む `L41b`・`L41a` が A19 の (ii-a)・(ii-b)・(ii-c) と P14a を `α` に当てると書く。
(ii-c) の `borrow_ify` の側は `前提 (ii-c) の保存` が置く。D12 を満たすことは `<1>3` が読む。D11 は `B'` の
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
    BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, <ref id=a502f3e/>, EXT derive(Clone)
  <2>2. QED
    D14 の割り当ては `RcFunc::borrowed_units` が定めるので、<2>1 より変わらない。
    BY <ref id=ef8efc4/>, <2>1
<1>2. `P'` の各関数の本体と各グローバル初期化子の `init` は、`P` の対応するものに `cancel_body` を適用した
      ものであり、DEF 出力の本体 の `B'` である。
  BY CODE src/rc_ir/borrow.rs: cancel, DEF 出力の本体
<1>3. `ρ'` に対応する `B` の実行路 `ρ` と、`α'` に対応する `B` の活性化 `α` が定まる。`α` は D21 の
      制限を満たし、`ρ` と `α` について D11 の (S-a) と (S-b) が成り立ち、(S-c) が D11a の接頭条件つきで
      成り立つ -- すなわち `α` が時点 `τ` まで解放について閉じている (D11a) ならば、`τ` の読み・触れる
      動作の直前の点で、それが読みうる・触れるオブジェクトは解放されていない。
  BY <ref id=3d96eb8/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=c232680/>, <ref id=2bb344b/>, <ref id=7d5a1de/>, <ref id=329a0ee/>, DEF 路の対応, <ref id=7218f92/>
  L32 の 5 より `Del` は `Retain`/`Release` 節点だけなので L31 が使え、`ρ'` に対応する `B` の実行路 `ρ` が
  ちょうど 1 つ定まる。D29 は `α'` に対応する `B` の活性化がちょうど 1 つ存在することを述べ、`α` が
  D21 の制限をその全点で満たすことは L44 の言明の「**とくに `α` はその全点で D21 の制限を満たし、
  D21 の意味の活性化である。**」が述べる。**D11 と D12 が課す条件は
  D21 の意味の活性化についてのものであり、D21 は制限を満たすものだけを活性化とするので、この引用が要る**
  -- 制限を満たさない対は D11 の範囲に入らない。`P` が D12 を満たすので、`B` のすべての実行路と、
  それを辿るすべての活性化について D11 の 3 つが成り立ち、`α` はその 1 つである。(S-c) の節が接頭条件を
  持つことは D11 の本文が述べる。
<1>4. `ρ'` の各節点の入口の点において `Obl'(q, O) = Obl(q, O) - d(q, O)` であり、`d(q, O) ≥ 0` で
      ある。
  BY <ref id=329a0ee/>, <ref id=4ff3e8d/>
<1>5. (S-a) が `ρ'` で成り立つ。
  <2>1. `ρ'` の上で `Obl'` から参照を取り除く操作は、`ρ` の上の同じ操作から `Del` の `Release` を除いた
        ものである。`ρ` と `ρ'` は `Del` の節点を除いて同じ節点を対応する値に対して実行するので
        (L44 の (a))、残る各操作が取り除く参照の個数は 2 つの実行で等しい。
    BY <ref id=329a0ee/>, <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=f06144e/>
  <2>1a. CASE その素動作が `ρ` の終端の `Ret` の消費のものである。**終端の `Ret` で pending な `CT` の
         要素は無い** -- L35 より、`n` で pending である要素の由来は `n` を含むすべての実行路の上に
         あって `n` より真に前にあるので、終端の `Ret` で pending な `CT` の要素 `t` は `ρ` の上に
         あり、L38 の 2 よりその pending の区間は終端の `Ret` より真に前で終わる。よって L43 の等式より
         終端の `Ret` の入口の点で `d = 0` である。`Del` の節点は
         どれも終端の `Ret` より真に前にある -- L32 の 5 より `Del` の要素は `Retain` 節点か `Release`
         節点だけであり、L32 の 2 と 3 より `un_bump_releases` の側の要素は `Release` 節点なので、`ρ` の
         上にある `Del` の `Retain` 節点は `CT` の要素であり、L38 の 2 より
         その pending の区間は終端の `Ret` より真に前で終わる。`ρ` の上にある `Del` の `Release` 節点は、
         L32 の 3 と L37 よりある `t ∈ CT` について `R_ρ(t)` の要素であり、L38 の 3 より `I_ρ(t)` に
         入る。よって終端の `Ret` の実行の各点でも `d = 0` である (DEF 欠損)。L44 の (c) よりその各点で
         `Obl' = Obl` であり、<1>3 の (S-a) が `α` について「取り除かれる参照はその時点の `Obl` に
         入っている」を与えるので、`α'` についても同じである。
    BY <ref id=014f689/>, <ref id=5adaf7f/>, <ref id=5b098d9/>, <ref id=23ac734/>, <ref id=4ff3e8d/>, <ref id=7d5a1de/>, <ref id=329a0ee/>, <ref id=24bf090/>, DEF 欠損, DEF 訪問, <1>3, <1>4
  <2>2. CASE その素動作が D10 の行が定める処分であって、`ρ` の終端の `Ret` の消費のものでない。
        1 つの素動作が取り除くのは 1 つの leaf の参照である (`DEF 節点の実行の素動作` の第 1 の箇条)。
        その素動作を持つ節点を `q`、その素動作の
        直後の点を `τ` とする。この場合 `q` は `ρ` の終端の `Ret` ではなく、D3 と L33a より終端の `Ret`
        は `ρ` の最後の節点なので、`τ` は終端の `Ret` の消費より前にある。<2>1 より `q` は `Del` に
        入らず、`τ` は `q` の実行の点なので、L41c の 1 より `Obl(τ, O) ≥ d(q, O)` である。
        `q` が `Del` に入らないので DEF 欠損 より `d(τ, O) = d(q, O)` であり、
        `Obl(τ, O) ≥ d(τ, O)` である。L41c を `α` について読むのに要る、`α` が D21 の意味の活性化で
        あること -- L41c が読む `L41b`・`L41a` が A19 の (ii-a)・(ii-b)・(ii-c) と P14a を `α` に当てる
        ので要る -- は <1>3 が与える。
    BY <ref id=965e588/>, DEF 節点の実行の素動作, <ref id=f06144e/>, <ref id=ca36627/>, <ref id=68112c9/>, DEF 欠損, <2>1, <1>3, <ref id=c232680/>
  <2>3. <2>2 の各素動作の直後の点 `τ` について `Obl'(τ, O) ≥ 0` であり、その直前の点 `τ_-` では
        `Obl'(τ_-, O) ≥ 1` である。よって取り除かれる参照は `τ_-` の `Obl'` に入っている。
    BY <2>2, <1>4, <ref id=329a0ee/>, DEF 欠損
    <2>2 より `Obl(τ, O) ≥ d(τ, O)` であり、L44 の (c) を `τ` に当てると
    `Obl'(τ, O) = Obl(τ, O) - d(τ, O) ≥ 0` である。この素動作が取り除くのは 1 つの leaf の参照なので
    (<2>2)、`Obl'(τ_-, O) = Obl'(τ, O) + 1 ≥ 1` である。
  <2>3a. CASE その操作が、op の生成コードが出す retain が作った参照を呼び出し先へ渡す受け渡しである。
         この操作は `Obl'` から参照を 1 つ取り除くが、その参照は同じ節点の実行の中でその retain が
         `Obl'` に加えたものであり、D24 は「retain と適用のあいだの段内の点で `Obl(a)` に在り、適用が
         それを呼び出し先へ渡し」と述べるので、受け渡しはその retain より後にある。よって取り除かれる
         参照はその直前の点の `Obl'` に入っている。
    BY <ref id=e3436e8/>, DEF 節点の実行の素動作
  <2>3b. CASE その操作が (F) の解放の連鎖の中で `Obl'` から参照を取り除くものである。`L39a` の第 2 の
         言明が、取り除かれる参照はその直前の点の `Obl'` に入っていることを述べる。
    BY <ref id=78073d2/>, <ref id=e3436e8/>, DEF 節点の実行の素動作
    `L39a` は「**さらに、連鎖の中で `Obl` から参照を取り除く各素動作について、取り除かれる参照はその
    直前の点の `Obl` に入っている。**」と述べ、その主語を「D21 の意味の 1 つの活性化であり、実行 (D24)
    が作る活性化に限らない」と書く。`α'` はその意味の活性化である。
  <2>4. QED
    BY <2>1, <2>1a, <2>2, <2>3, <2>3a, <2>3b, <1>4, <ref id=329a0ee/>, <ref id=e3436e8/>, <ref id=78073d2/>, <ref id=9d3dd4d/>, <ref id=f06144e/>, DEF 節点の実行の素動作
    (S-a) が見る操作は `Obl` から参照を取り除くすべての操作である。`DEF 節点の実行の素動作` の 6 種の
    うちそれに当たるのは、D10 が `Obl` から参照を取り除くと定めるもの -- D9 の消費の表の 6 行と
    `Release` 節点 --、**第 4 の箇条の (F) の解放の連鎖が `Obl` から取り除くもの**、そして第 6 の箇条の
    受け渡しである。**この 3 つで場合は尽き、4 つの CASE は互いに素である。** 第 1 を、`ρ` の終端の
    `Ret` の消費のものと、そうでないものに分け、前を <2>1a が、後を <2>2 と <2>3 が扱う。第 2 を
    <2>3b が、第 3 を <2>3a が扱う。第 6 の箇条の残る素動作は `Obl` から参照を取り除かない --
    retain は加える側であり、相殺しない形の持ち手はその生成コードが書き込むオブジェクトの持ち手の
    単位であり、相殺する形の release は D9 の `Llvm` の行の消費なので第 1 に入る (D24)。残る 3 種 --
    割り当て、子の活性化の素動作、グローバル化 -- のうち、割り当ては D10 の生成と同じ素動作であって
    `Obl` に加える側であり、残る 2 つは `Obl` を動かさない (`L39b`)。
<1>6. (S-b) が `ρ'` で成り立つ。
  <2>1. `ρ` の終端の `Ret` では、`CT` の要素はどれも pending でない。
    BY <ref id=5adaf7f/>, <ref id=5b098d9/>, DEF 訪問
    L35 より、終端の `Ret` で pending である要素の由来は、終端の `Ret` を含むすべての実行路 -- `ρ` を
    含む -- の上にあって、その路の上で終端の `Ret` より真に前にある。よってその由来が `CT` の要素で
    あれば `ρ` の上にあり、L38 が当たる。L38 より各 `t ∈ CT` が pending である区間は `n*(ρ)` で終わり、
    `n*(ρ)` は終端の `Ret` より真に前にある。
  <2>2. 終端の `Ret` の入口の点と `q_end` で `d = 0` である。
    BY <2>1, <ref id=4ff3e8d/>, <ref id=7d5a1de/>, DEF 欠損
    L43 より `d` は `CT` の pending な要素の `B_ρ` の和であり、<2>1 よりその和は空である。終端の `Ret` は
    `Del` に入らない (L32 の 5 より `Del` は `Retain` と `Release` だけ) ので、その消費の後も `d` は
    変わらない。
  <2>3. QED
    BY <1>3, <2>2, <ref id=329a0ee/>, <ref id=4a0c14c/>, <ref id=2bb344b/>
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
    BY <ref id=4a0c14c/>, <ref id=2bb344b/>, <ref id=7d5a1de/>, <ref id=329a0ee/>, <ref id=56c2068/>, <ref id=4f63121/>, <ref id=7218f92/>
  <2>1a. `α` の各点 `τ` と、それに対応する `α'` の点について、各計数下オブジェクト `O` が `τ` で
         解放されている (DEF 解放されている) ことと、対応する点で `O` が解放されていることは同値で
         ある。
    BY <ref id=329a0ee/>, DEF 解放されている, DEF 節点の入口の点, <ref id=7218f92/>, <ref id=e3436e8/>, <ref id=0b850c9/>, <ref id=f06144e/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=b6673ca/>
    L44 の (e) は、`ρ` の上の各節点の実行の各点 -- `q` の入口の点とその実行が終わった点を含む --
    について、2 つの活性化の点が対応し `H(τ, O) = 0` と `H'(τ, O) = 0` が同値であることを述べる。
    **これは段内の点の粒度の言明である** -- (e) が量化するのは節点の実行の各点であって、節点の入口の点
    だけではない。DEF 解放されている は「`τ` かそれより前の点で `O` の参照カウントが 0 であること」を
    解放されていることと定めるので、`τ` 以前の各点にこの同値を当てれば、2 つの活性化で解放されているオブジェクト
    は対応する。**どちらかの活性化が解放するオブジェクトは D29 の全単射の定義域に入る** -- D29 の
    第 5 行よりその定義域は 2 つの活性化がそれぞれ到達できる (D25) オブジェクトの全体であり、D24 の
    (F) が解放を起こすのは参照の処分であって、活性化の素動作が処分する参照は D10 の行が名指す leaf の
    もの -- その leaf はその活性化の位置 (D6) のものである -- か、既に解放を始めたオブジェクトが保持する
    参照のもの (D24 の (F)) だからである。計数下でない
    オブジェクトはどちらの活性化でも解放されない (D26、A8)。
  <2>1b. **(S-c) が見る「直前の点」でも、この同値が成り立つ。** その点は一般に段内の点ではないが、D24 が
         その点の勘定を直前の段内の点のものと定める。
    BY <ref id=e3436e8/>, <2>1a
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
    BY <2>1a, <ref id=329a0ee/>, <ref id=4798985/>, <ref id=859cf84/>, <ref id=88a06de/>, DEF 解放されている
    D11a は、時点が解放について閉じているとは、その時点で `H(O) ≥ 1` である各計数下オブジェクト `O` が
    その時点で解放されていない (D24 の (F)) ことと定め、`τ` まで閉じているとは `τ` 以前の各時点が
    閉じていることと定める。D11a のこの「解放されていない」と DEF 解放されている の否定が同じものを
    指すことは `L48` が述べる。`α` の時点 `σ` と `H(σ, O) ≥ 1` である計数下 `O` を取る。**`H = 0` と
    `H' = 0` の同値を与えるのは `L44` の (e) である** -- `<2>1a` が述べるのは解放されていることの同値で
    あって、カウントが 0 であることの同値ではない。その (e) より対応する点で `H'(σ, O) ≥ 1` であり、
    `α'` はその時点まで閉じているので `O` はそこで解放されて
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
  BY <1>1, <1>2, <1>5, <1>6, <1>7, <ref id=95427eb/>, <ref id=3d96eb8/>

## 10. P24 (D12 が見ない部分の保存)

**言明 (README)** --- `borrow_ify` と `cancel` は、D12 が見ない部分について次を満たす。

- `roots` を変えない。
- **出力の各関数は入力のちょうど 1 つの関数から作られ**、その `fn_ty` / `ret_ty` / `params` の型 /
  `inline_into_callers` は元の関数のものに等しい。鍵の一致では述べられない -- `borrow_ify` は入力に無い
  関数 (借用版) を足すからである。
- 出力のグローバル初期化子の列は入力と同じ長さで、第 `i` 要素の `symbol` と `ty` は入力の第 `i` 要素の
  ものに等しい。`owns_initializer` と `owns_storage` には `true` を書き、D1 が述べる呼び出し順により
  この書き込みは正しい値を書く。
- **`cancel` は `RcFunc` の `body` 以外の欄を 1 つも変えない。** とくに `borrowed_units` と `capture` は
  入力のものに等しい (`CODE src/rc_ir/borrow.rs: cancel` -- `f.clone()` の `body` だけを差し替える)。
  `borrow_ify` は `borrowed_units` を書くので、この節は `cancel` についてだけで
  ある。**この節は P14b の結論を運ばない** -- どの種の段がどの本体の活性化を作るかは実行の上の言明で
  あり、欄と本体の一致からは出ない。それは P14b が自分の範囲に `cancel` の出力を入れて述べる。
- **本体について書き換えが変えるのは、`Retain`/`Release` の節点と、`App` の callee の名前だけである。**
  節点の種類・その順序・`Let` の束縛変数・`Match` のアームの構成・`Llvm` の op とオペランド・
  `Destructure` のフィールドは、いずれも元の本体のものに等しい (複製の名前替えを P9 で戻したうえで)。
  `origin` が書き換えの前後で対応することを言う議論がこれを読む。

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
  BY CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/ast.rs: RcFunc, <ref id=a502f3e/>, <ref id=dbdbf7e/>, EXT derive(Clone)
  D1 より `RcFunc` は `name`・`fn_ty`・`params`・`capture`・`ret_ty`・`body`・`source`・
  `borrowed_units`・`inline_into_callers` の 9 個の欄を持つ。`cancel` は `prog.funcs.values()` の各 `f`
  について `let mut clone = f.clone();` を作って `clone.body` にだけ書き込み、`(f.name.clone(), clone)` を
  出力の `funcs` に入れる。`RcFunc` は `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の各欄は
  原本の対応する欄の `clone` であり、残る 8 個は入力のものに等しい。`borrow_ify` は `borrowed_units` を
  書き替える (`CODE src/rc_ir/borrow.rs: borrow_ify` -- `for func in funcs.values_mut()` のループ) ので、
  この節は `cancel` についてだけである。
  **この節は P14b の結論を運ばない。** P14b が述べるのは「**`borrow_ify` の出力と、`cancel` がそれを写した
  プログラムの両方**について、その実行 (D24) において、借用する unit を持つ本体の活性化を作る段は、
  (E3) の呼び出しの段に限る。」であり、これは実行 (D24) の上の言明で、欄と本体の一致からは出ない。P14b は
  `cancel` の出力を自分の範囲に入れて述べる。
<1>3. `borrow_ify` の出力の各関数は入力のちょうど 1 つの関数から作られ、その `fn_ty`、`ret_ty`、
      `params` の型、`inline_into_callers` は元の関数のものと等しい。
  <2>1. 元の版 `f_own` は `func.clone()` に `body` を書き込んだものであり、その後
        `for func in funcs.values_mut()` のループが `borrowed_units` だけを書き換える。`RcFunc` は
        `#[derive(Clone)]` を持つので、EXT derive(Clone) より複製の各欄は原本の対応する欄の `clone` で
        ある。よってこの 4 つは変わらない。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/ast.rs: RcFunc, EXT derive(Clone)
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
    BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite, <ref id=3e6b0e0/>
  <2>2. 木 `N(node)` の構造についての帰納法で示す。DEF 子と親 より子は真の部分木なので整礎である。
    <3>1. CASE `node` の式が `RcExpr::Let(x, RcRhs::App(callee, args), k)` である。この腕は `x` と
          `args.clone()` を据えた `Let` の節点を `&node.source` を付けて作り、その継続を
          `prepend_rc(after, true, self.rewrite(k))` に、callee を `self.route(x, callee, args, k)` の
          値にする。さらに `prepend_rc(before, false, ・)` でその節点を包む。P12 より `route` が返すのは
          元の呼び出し先と同じ関数の版である。**変わりうるのがその名前だけであることは `route` の
          コードが与える** -- `route` は振り替えるとき `callee.clone()` を作ってその `name` の欄だけを
          借用版の `name` で置き替え、振り替えないときは `callee.clone()` をそのまま返す
          (`CODE src/rc_ir/borrow.rs: route`)。`RcVar` は `#[derive(Clone)]` を持つので、
          EXT derive(Clone) より複製の各欄は原本の対応する欄の `clone` である。P11 より `call_rc` が
          置くのは `Retain` と `Release` だけであり、`prepend_rc` はその各要素を `rc_node` で節点にする。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: prepend_rc,
         CODE src/rc_ir/borrow.rs: rc_node, CODE src/rc_ir/borrow.rs: route,
         CODE src/rc_ir/ast.rs: RcVar, <ref id=eaf9b51/>, <ref id=843e506/>, EXT derive(Clone)
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
         CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, CODE src/rc_ir/borrow.rs: rc_node, <ref id=a985128/>
    <3>5. CASE `node` の式が `RcExpr::Destructure(container, fields, state, k)`、`RcExpr::Eval(v, k)`、
          または `RcExpr::Ret(v)` である。この 3 つの腕は、`container`/`fields`/`state`、`v` を
          `clone()` し、継続を `self.rewrite(k)` に置き換えて `&node.source` を付ける (`Ret` は継続を
          持たない)。
      BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner
    <3>6. QED
      BY <3>1, <3>2, <3>3, <3>4, <3>5, CODE src/rc_ir/ast.rs: RcExpr, CODE src/rc_ir/ast.rs: RcRhs,
         CODE src/rc_ir/borrow.rs: expr_node, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
         CODE src/rc_ir/borrow.rs: prepend_rc, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, <ref id=95427eb/>, <ref id=3d96eb8/>
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
  <2>2a. 借用版が書き換える本体は `clone_func` が作る複製であり、P9 がそれを元の本体の名前替えとして
         述べる。
    BY CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: clone_func,
       CODE src/rc_ir/rename.rs: fresh_rename_function, <ref id=63eadd9/>, <ref id=181fe37/>
    `borrow_ify` は借用版について `clone_func(func, ・, ・)` の返り値を作り、その `body` を
    `RewriteCtx` の `rewrite` の値に据える。D35 は書き換えの前の本体を `Pre(V)` と呼び、その版の
    `RewriteCtx` の `vars` がその表であると定める。P9 は「`clone_func` が作る借用版の本体は、元の本体の
    束縛変数を一斉に付け替えたものであり、それ以外の違いを持たない。」と述べる。
  <2>3. QED
    BY <2>1, <2>2, <2>2a
<1>3b. `cancel` の出力の各本体について、書き換えが変えるのは `Retain`/`Release` の節点だけである。
  BY <ref id=4a0c14c/>, <ref id=7d5a1de/>
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
  BY <ref id=a502f3e/>, CODE src/build/build_object_files.rs: build_object_files,
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

> `k(O)` は、その位置までに `α` が実行した削除済みの `Retain` が `O` に作った参照のうち、**その `Retain`
> と対になる削除済みの `Release` がまだ処分していないもの**の個数である。

この形では、上の本体の点 `q` で `k(O0) = 0`、`k(O1) = 1` であり、どちらも差に一致する。

**この形は証明が出す量である。** `L43` が、`DEF 欠損` の `d(q, O)` -- 削除される `Retain` が
作った個数から削除される `Release` が処分した個数を引いたもの -- がこの形の量に等しいことを示す。
`L44` の (b) が `H'(q, O) = H(q, O) - d(q, O)` を与えるので、P21 (a) はそのまま出る。

**対になることは `L32` の 3 が与える。** `Del` の各 `Release` はちょうど 1 つの `t ∈ CT` の
`un_bump_releases[t]` に属するので、README の P21 (a) が「**その `Retain`
と対になる削除済みの `Release` がまだ処分していないもの**」と書く対応は 1 つに定まる。
