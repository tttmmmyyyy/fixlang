# P31: `insert_rc` と `split_rc_units` の出力と A19

この文書は、README の A19 が `insert_rc` に帰した義務 -- (ii-a)・(ii-b)・(ii-c) -- を扱う。README の
定義・仮定・命題の**言明**の上に立つ。**その一覧を手で書かない** -- 引く項目は
`python3 dev-docs/proof/proof_index.py --render` が生成するヘッダが挙げる。加えて
`p13-disposals-and-pending.md` から命題 `p13 の L9`・`p13 の L12a`・`p13 の L14` の**言明**を輸入する。
この文書自身も `L9`・`L12`・`L14` という名札を別の命題に使うので、修飾を落とさない。`p13 の L9` の
言明は次である。

- 「`x` を `B` の変数、`λ ∈ boxed_leaf_paths(ty(x))` とし、実行路 `ρ` の上で `x` が値を得ていると
  する。`(u, σ) = origin(x, λ).identity()` と置く。このとき `ρ` の上で `u` は値を得ており、
  `σ ∈ boxed_leaf_paths(ty(u))` であり、`λ` が `x` の値の inhabited な leaf であることと `σ` が
  `u` の値の inhabited な leaf であることは同値である。」

`p13 の L12a` は 1 つの別名類に属する 2 つのスロットが
同じオブジェクトを指すこと -- すなわち D33 の `obj(C)` が定まること -- を与える。`p13 の L14` の
言明は 2 つの節からなる。

- `ρ` の上のスロット `(x, λ)` について、`o = origin(x, λ).identity()` は `ρ` の上の位置 (D6) --
  スロットか記号の位置 -- であり、`obj(o) = obj(C_ρ(x, λ)) = obj(x, λ)` である。
- **`obj(x, λ)` が D26 の意味で計数下であるとき、`o` は `ρ` の上のスロットであって
  `C_ρ(o) = C_ρ(x, λ)` であり、`o` は `ρ` で活性 (p13 の `DEF 名前の活性`) であって
  `obj_ρ(o) = obj(x, λ)` である。** 第 6.1b 節の `L5b` が読むのは、この後半のうち `o` が `ρ` の上の
  スロットであって `C_ρ(o) = C_ρ(x, λ)` であるという部分である。

**`p13` の反例 `C1` については、本体だけを輸入する** -- `DEF C1 の本体` がそれを書き写し、この文書が
`C1` について読む事実 (別名類の分割、callee の位置、A19 (ii) を破る時点での `held` と `bumps` の値) は
どれも `L5` と `L12` がその本体から自分で示す。

証明対象のコードは `src/rc_ir/rc_insert.rs` の全体と、
`src/rc_ir/borrow.rs` の `split_rc_units`・`split_body`・`split_body_inner`・`split_rc`、およびこれらが
呼ぶ `src/rc_ir/ownership.rs` の `units_under`・`subtree_type`・`rc_units`・`rc_units_go`・`unit_step` と
`src/rc_ir/leaf_map.rs` の `boxed_leaf_paths` である。
対象コミットは README 第 1 節が据える `b6c51fb892746e493e155d9d59ea05d02d7357db` である。

**結論を先に書く。**

- **`(P-insert)` は偽である。** `insert_rc` が実際に出力する 5 節点の本体が反例である (第 4 節)。破れは
  1 か所の数え落としから来る -- 言明が数える「残りの消費と `Release`」には、まだ実行されていない
  `Retain` が作る参照の分が入っており、その分を引いていない。**これはコードの欠陥ではない。** 反例の本体は
  D12 を満たし、A19 (ii) の 3 つの節もすべて満たす (第 4 節の `L30`)。
- **数え落としを直した形だけでは A19 (ii) は出ない。** 直した形は `p13-disposals-and-pending.md` の
  反例 `C1` が満たし、`C1` は A19 (ii) を破る (第 5 節)。よって A19 (ii) へ渡るには、直した形に加えて
  `insert_rc` の出力を `C1` から区別する事実が要る。すなわち `(P-insert)` は、強すぎる読みでは
  `insert_rc` の出力について偽であり、弱めた読みでは `C1` を弾かない。
- **A19 (ii) が `insert_rc` に要求しているものは、走査の名前に触れずには書けない。** 第 6 節が A19 (ii) を
  台帳の形 `U + X ≥ D` に書き直す。`D` は別名類の処分の個数、`U` と `X` は走査がその類の名前について
  落とす bump の量である。`held` だけを主語にする言明はこの不等式を決めない。
- **`insert_rc` が与える形は、構文の形として書ける。** 第 7 節の `L9` -- `insert_rc` が置く `Retain` は、
  その変数を名指す構文の直前 (間に `Retain` 以外の節点を挟まない位置) にしか現れない -- を証明する。
  `C1` の `main` はこの形を破るので、**`insert_rc` の出力ではない** (第 7 節の `L10`)。
  この判定は節点 1 つを見れば済む。
- **A19 を読む 2 つの命題は、別のものを要求している。** P18a・P18c・P19・P21 が読む形 (走査の帳簿) と、
  P14 が読む形 (由来ごとの非負性と、名指す由来に参照が 1 つ以上残っていること) は、どちらも他方を
  導かない。`C1` が前者だけを破り、第 9 節の `B_1` が後者だけを破る (第 9 節)。**A19 は 2 つに割れる。**
- **(O1) は証明されている。** `insert_rc` の liveness は使用だけから作られる集合であり、`Retain` と
  `Release` の位置はそれだけで決まる。第 10 節が、**各検査点において 1 つのスロットに割り当たる参照の
  個数はその変数が live かどうかで決まる** (`L18`) ことを示し、そこから別名類の粒度の RC 規律
  (`L19` (a)-(c)) と、「各計数下の別名類 `C` の開始事象 (`DEF 開始事象`) は `ρ` の上に高々 1 つで
  ある」(`L20`) を出す。`L4` と `L7` の前提「ちょうど 1 つ」へは、`L13a` (f) が渡す (`L25` `<1>2c`)。
  (ii-a) が
  節点の入口の外に置く
  1 点 -- 終端の `Ret` の消費を行った直後 -- は `L19` (c) が `held = 0` の等式で与える。
  A19 の (ii-c) -- 節点の実行の途中の各点 (D24 の段内の点) で、その点で `held_ρ` が定まる各計数下の
  別名類についての非負性 -- は `L19` (d) が示す。
- **(O2) は証明されている。** 第 11 節が、1 つの別名類のスロットに `origin` の
  `identity` が付ける名前の全体 `Ids(C)` が木をなすこと (`L21`) を示し、**その木の各部分木について
  「付いている bump の総和が 1 以上であるならば、それはその部分木が保持している参照の個数より真に
  小さい」**(`L24`) を示す。**仮説を落とした無条件の形は偽である** -- README の A19 (ii-b) が
  「無条件の `held ≥ 1 + bumps` は偽である」と書くのと同じ理由で、`Bsub = Down = 0` の名前で破れる。
  木の根で読むと `held ≥ 1 + bumps` になる (`L25`)。**「名前は別名類を決める」は `L5b` が
  `p13` の `L14` の後半から出す** (6.1b 節)。第 11 節が本体について
  読むのは前提 (S) の 5 つだけであり、`insert_rc` の出力がそれを満たすことは `L20a` が与える。
- **A19 (i) はこの文書の義務ではない。** (i) は仮定ではなく、D21 が活性化に課す**制限**である。
  実行 (D24) が作る活性化がその制限を満たすことは P28 (b) が示す。この文書が (i) について読むのは、
  `held` と `μ` の差 -- 借用する (D14) unit の下の leaf を ρ-終端とする類の開始の 1 -- が (i) の
  `d(C)` が引く角括弧と同じものであることだけであり、それは `L13a` (b)(c) が与える。
- **`split_rc_units` の段も閉じている。** A19 (ii) の範囲は「**`borrow_ify` の入力の各本体と、
  `borrow_ify` がそれを写した各本体 (すなわち `cancel` の入力) の両方**」であり、この文書が扱うのは
  その第 1 の半分である。`borrow_ify` の入力は `split_rc_units` の出力なので、
  第 13 節が、`Retain(v, π)` を `units_under(ty(v), π)` の鎖へ割る段について、(ii-a) が
  保存されること (`L31`)、(ii-c) が保存されること (`L31a`)、および (ii-b) が出力について
  成り立つこと (`L32`) を示す。
  支えは、**unit が `π` の下の boxed leaf を分割する**こと (`L27`) と、**割る段が束縛を 1 つも作らない**
  ので `origin`・別名類・`held` が変わらないこと (`L29`) である。(i) はこの段に何も要求しない --
  D21 の制限であり、実行が作る活性化がそれを満たすことは P28 (b) が示す。
- **A19 (ii-b) は活性化の終わりの 1 点先へは延びない。** 終端の `Ret` の消費の後、`held` は 0 になるが
  (`L19` (c))、走査の `RcExpr::Ret` の腕は pending の要素を取り除かないので `bumps ≥ 1` が残りうる。
  第 12 節の `L38` がその形の `insert_rc` の出力を挙げる。(ii-a) はこの点でも成り立つ。

## 1. 記法

`insert_rc(prog, type_env)` は `prog` の各関数の `body` と各グローバル初期化子の `init` を書き換える
(`CODE src/rc_ir/rc_insert.rs: insert_rc`)。

- **DEF 骨格**。`insert_rc` の入力の本体を**骨格**と呼ぶ。A25 より骨格は `RcExpr::Retain` と
  `RcExpr::Release` を含まない。
- `ty(x)`、`origin(x, π)`、`acted_on(x, π)`、`L(v, π)`、`ActRefs(v, π)` は
  `p13-disposals-and-pending.md` が記法として定めるものと同じ意味で使う。
- `needs_rc(v)` は `RcInserter::needs_rc(v)`、すなわち `!v.ty.is_fully_unboxed(type_env)`
  (`CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc`)。

**DEF `nm(s)`**。`ρ` の上のスロット `s = (v, λ)` (D6) について `nm(s) := origin(v, λ).identity()` と
置く (`CODE src/rc_ir/ownership.rs: origin`, `CODE src/rc_ir/ownership.rs: Origin`,
`CODE src/rc_ir/ownership.rs: Origin::identity`)。**この記法をここに置くのは、第 6.1a 節がこれを
第 11 節より前に読むからである。**

**DEF 時点**。この文書では**時点**とは節点の訪問の入口を指す。`held_ρ(n, C)`、`bumps_ρ(n, C)` は、
`ρ` の上の節点 `n` の入口における値である。**この粒度は A19 (ii-a)・(ii-b) の「各時点」が指す点集合と
同じである** --
README の A19 は「**「各時点」は、その活性化が生きている (D23) 間の、その活性化の節点の訪問の入口で
ある時点である。**」と書き、その理由を「`bumps` を定める D27 が `B(p, ρ)` を走査中の位置 --
節点の訪問の入口 -- でしか定めないので、この 2 つを読める点はそこに限る」と述べる。
**節点の実行の途中の点での非負性は、A19 が (ii-c) として別に立てている** -- README の A19 は
「**(ii-c) (段内の点の非負性)。節点の実行の途中の各点 (D24 の段内の点) と、その点で `held_ρ` が定まる
各計数下の別名類について、`held ≥ 0` である。**」と書く。D34 は類の開始の点より前で `held_ρ` を定め
ないので、(ii-a) が D34 を通して受けるのと同じ条件がここにも掛かる。**量化の範囲はこの一文が限る** --
開始の点より前の類は範囲の外である。
果たす者について README は「果たす者は `insert_rc` (`p60-insert-rc.md` の `L19` (d)) と `borrow_ify`
(`p20-borrow-ify.md` の第 13 節) である。」と 2 人挙げ、この文書が果たすのは前者である。
第 10.7 節の `L19` (d) がその段であり、その言明も同じ範囲で量化する。

**外部の結果。** README が `EXT` の名札を与える群の項目を、この文書は次のとおり据える。

**EXT 呼び出しの入れ子**。1 つのスレッドの
計算において、関数の呼び出しは開始と終了について入れ子をなす。すなわち、呼び出し `c` の実行中に
始まった呼び出しは `c` が戻る前に戻り、`c` の実行中のどの時点でも戻っていない呼び出しは、`c` と
その祖先 -- `c` を実行している呼び出しの列 -- である。停止する計算が行う呼び出しは有限個であり、その
どれもが戻る。

**EXT `Iterator::fold`**。反復子 `it` が生む元を順に `a_1, …, a_n` とすると、`it.fold(init, f)` は
`f(… f(f(init, a_1), a_2) …, a_n)` を返す。`n = 0` のときは `init` をそのまま返す
(Rust の標準ライブラリ `core::iter::Iterator::fold`)。

**EXT `Iterator::rev`**。`it.rev()` は、`it` が生む元を逆順に生む反復子である。生む元の多重集合は
変わらない (`core::iter::Iterator::rev`)。

**EXT `Iterator::all`**。`it.all(p)` は、`it` が生むすべての元について `p` が真を返すとき、また
そのときに限り真を返す (`core::iter::Iterator::all`)。

**EXT 述語による除去**。`Vec::retain(p)` と `HashSet::retain(p)` は、`p` が偽を返す元をすべて
取り除き、真を返す元をすべて残す。`Vec` では残る元の順序も変わらない
(`alloc::vec::Vec::retain`, `std::collections::HashSet::retain`)。この文書が `Set` と書くのは
`FxHashSet` の別名である (`CODE src/misc.rs: Set`)。

**EXT `Vec::extend`**。`v.extend(it)` は、`it` が生む元を順に `v` の末尾へ足す
(`alloc::vec::Vec` の `Extend` の実装)。

**EXT `Iterator::filter_map`**。`it.filter_map(f)` は、`it` が生む各元 `a` について `f(a)` が
`Some(b)` のとき `b` を生み、`None` のとき何も生まない反復子である
(`core::iter::Iterator::filter_map`)。

**EXT 空の列の第 1 元**。`v.first()` は、`v` が空のとき `None` を返し、そうでないとき第 1 元への
参照を包んだ `Some` を返す (`core::slice::first`)。`Option` を返す関数の本体に書かれた `e?` は、
`e` が `None` のときその関数から `None` を返す (Rust Reference の "The question mark operator")。

**EXT 空を作る `Default`**。`Vec`・`Map`・`Set` の `Default::default()` はそれぞれ空の列・空の写像・
空の集合を返し、`RefCell<T>::default()` は `T::default()` を包んだ `RefCell` を返す
(`alloc::vec::Vec`・`std::collections::HashMap`・`std::collections::HashSet`・`core::cell::RefCell` の
`Default` の実装)。`#[derive(Default)]` が作る実装は、各欄にその型の `default()` を置く
(`core::default::Default` の derive マクロ)。

**EXT 導出した `PartialEq`**。`#[derive(PartialEq)]` が作る等号は、2 つの値が同じ構成子であって、
同じ位置の各欄がその欄の型の `PartialEq` で等しいとき、またそのときに限り真である。`HashSet` の
等号は、大きさが等しくどちらの元も他方に含まれること -- すなわち集合としての等しさ -- である
(`core::cmp::PartialEq` の derive マクロ、`std::collections::HashSet` の `PartialEq` の実装)。

**EXT クレートの項目**。Rust Reference は、クレートについて「A crate contains a tree of nested module
scopes. The top level of this tree is a module that is anonymous (from the point of view of paths
within the module) and any item within a crate has a canonical module path denoting its location
within the crate's module tree」と述べ、モジュールについて「A module is a container for zero or more
items」「A module without a body is loaded from an external file」「the module's contents are in a
file with the name of the module plus the `.rs` extension」と述べる (Rust Reference の
"Crates and source files" と "Modules")。すなわちクレートは、クレート根から `mod` 宣言をたどって
得られるモジュールの木であり、各モジュールの**内容**はブロックか、その宣言が読み込む 1 つのファイル
である。よって、あるモジュールの内容を持つファイルの全文を読めば、そのモジュールの内容 -- 項目、
トレイト実装、およびそれらの本体に書かれた式 -- をすべて読んだことになり、クレートの全ファイルの
全文を読めば、そのクレートの内容をすべて読んだことになる。**式まで届くのは「the module's contents
are in a file」の側である** -- 「A module is a container for zero or more items」が数えるのは
項目だけである。

**EXT ビルドの対象**。`Cargo.toml` は 2 つのターゲットを宣言する -- `[lib]` は
`path = "src/lib.rs"`、`[[bin]]` は `path = "src/main.rs"` である。`src/lib.rs` と `src/main.rs` は
どちらも同じ名前のモジュール (`ast`、`build`、… `rc_ir`、…) を `mod` 宣言でたどるので、**同じ
ファイルが 2 つのクレートに属する**。よって「この式はクレートに 1 つも無い」を確かめる走査の範囲は、
`src/` の全ファイルである。

**EXT 条件つきコンパイル**。Rust Reference は `cfg` 属性について「If the predicates are true, the
form is rewritten to not have the `cfg` attributes on it. If any predicate is false, the form is
removed from the source code」と述べ、`test` の構成について「Enabled when compiling the test
harness」と述べる (Rust Reference の "Conditional compilation")。すなわち `#[cfg(test)]` が付いた
項目は、テストのハーネスをビルドするときにだけコンパイルされ、`fix` の実行可能ファイルを作る
ビルドには入らない。

**EXT 非公開の項目の可視範囲**。Rust Reference は項目の可視性について「With the notion of an item
being either public or private, Rust allows item accesses in two cases: If an item is public,
then it can be accessed externally from some module `m` if you can access all the item's ancestor
modules from `m`. … If an item is private, it may be accessed by the current module and its
descendants」と述べる (Rust Reference の "Visibility and Privacy")。すなわち、`pub` を持たない
項目を名指せる式は、その項目を宣言するモジュールとその子孫のモジュールの中にしかない。

### 1.1 在りかの前提

**コードのどこに何が在るかの数え上げは、段の中で行わない。** 段が自分で在りかを数え上げると、その
数え上げには果たす者が居らず、検査するものも無い。在りかは名前つきの前提として置き、`BY` の行では
その名前で引く。**個数は書かない** -- 一覧が在れば個数は一覧の長さである。

**果たすのは走査である。** 在りかを走らせられる字面で書き、`dev-docs/proof/proof_links.py` がその字面を
`src/` の全体に走らせて、下の一覧と突き合わせる。挙がった各項目が何であるかは `--` の後に書く。走査は
字面の上位近似なので、一覧には構成でなくパターンとしてその字面を持つ項目も入る。`#[cfg(test)]` の下の
項目は走査が除く。

**前提 `LeafOrigin::Arg` の在りか** --- `LeafOrigin::Arg` の字面を持つ項目は次で尽きる。そのうち
`LLVMGen::result_prov` の実装は 6 つであり、`--` の後にそれを書く。

SCAN src/ `LeafOrigin::Arg`
  = src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov -- `result_prov` の実装
  = src/fixstd/builtin.rs: InlineLLVMMakeStructBody::result_prov -- `result_prov` の実装
  = src/fixstd/builtin.rs: InlineLLVMStructPunchBody::result_prov -- `result_prov` の実装
  = src/fixstd/builtin.rs: replaced_field_prov -- `InlineLLVMStructPlugInBody::result_prov` と `InlineLLVMStructSetBody::result_prov` が呼ぶ補助関数
  = src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov -- `result_prov` の実装
  = src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov -- `result_prov` の実装
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- パターン (宣言を読む側)
  = src/rc_ir/ownership.rs: as_arg_projection -- パターン (宣言を読む側)
  = src/rc_ir/provenance.rs: Provenance::arg_passthrough -- 構成。呼ぶのは `Provenance` の解析であって `result_prov` の実装ではない
  = src/rc_ir/provenance.rs: Provenance::compose -- パターン (宣言を読む側)
  = src/rc_ir/provenance.rs: leaf_source_to_string -- パターン (表示)
  = src/rc_ir/provenance.rs: resolve_leaf -- パターン (宣言を読む側)

**前提 節点の書き換えの不在** --- `RcExprNode` の値を可変に借りる式も、`Arc` の中身を可変に借りる式も、
下の走査が挙げるもので尽きる。

SCAN src/ `&mut RcExprNode`

SCAN src/ `Arc::get_mut`

SCAN src/ `Arc::make_mut`
  = src/elaboration/typecheck.rs: TypeCheckContext::instantiate_scheme -- 借りるのは `Arc<Vec<Predicate>>` と `Arc<Map<..>>` であって `Arc<RcExpr>` ではない

**前提 `VarTable::bindings` へ鍵を入れる式の在りか** --- `bindings` という名前の欄へ鍵を入れる式は次で
尽きる。そのうち `VarTable` の欄を書くものは `--` の後にそう書く。

SCAN src/ `bindings.insert(`
  = src/rc_ir/ownership.rs: VarTable::of -- `VarTable` の欄。パラメータと capture の名前
  = src/rc_ir/ownership.rs: collect_bindings -- `VarTable` の欄。`Let`・`Destructure` の束縛変数と、`Match` のアームの payload の名前の 2 つの式
  = src/rc_ir/provenance.rs: Interpreter::record -- `Interpreter` の同名の欄であって `VarTable` の欄ではない

**前提 `RcFunc::borrowed_units` の在りか** --- `borrowed_units` の字面を持つ項目は次で尽きる。この欄に
値を書くものは `--` の後にそう書く。

SCAN src/ `borrowed_units`
  = src/build/build_object_files.rs: lower_and_insert_rc -- doc コメントの中の言及
  = src/rc_ir/ast.rs: RcFunc -- 欄の宣言
  = src/rc_ir/borrow.rs: borrow_ify -- 書き込み。`param_capture_units` から `owned_units` を除いた集合
  = src/rc_ir/borrow.rs: clone_func -- 書き込み。`Set::default()`
  = src/rc_ir/borrow.rs: covered_leaves -- 読み
  = src/rc_ir/borrow.rs: node_id -- doc コメントの中の言及
  = src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function -- 書き込み。`Set::default()`
  = src/rc_ir/ownership.rs: acted_references -- doc コメントの中の言及
  = src/rc_ir/ownership.rs: all_owned_units -- 読み
  = src/rc_ir/specialization.rs: CloneRegistry::finish_clone -- 書き込み。複製の鍵を改名して組む式

**前提 `finish_clone` の呼び出し元** --- `finish_clone` の字面を持つ項目は次で尽きる。

SCAN src/ `finish_clone(`
  = src/rc_ir/locality.rs: specialize -- 呼び出し
  = src/rc_ir/specialization.rs: CloneRegistry::finish_clone -- 宣言
  = src/rc_ir/unique_check_elim.rs: Specializer::materialize_clone -- 呼び出し

**前提 `borrows_operand` の本体の在りか** --- `LLVMGen::borrows_operand` の本体が在る項目は次で尽きる。
既定の本体を除く 13 個が override である。

SCAN src/ `fn borrows_operand`
  = src/ast/inline_llvm.rs: borrows_operand -- trait の既定の本体
  = src/fixstd/builtin.rs: InlineLLVMArrayBorrowElementsBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMArrayGetCapacityBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMArrayGetPtrBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMArrayGetSizeBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMArrayUnsafeGetBoundsUnchecked::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMCaptureProjectBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMGetBoxedDataPtrFunctionBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_operand -- override
  = src/fixstd/builtin.rs: InlineLLVMUnionIsBody::borrows_operand -- override

**前提 `Origin::Join` の在りか** --- `Origin::Join` の字面を持つ項目は次で尽きる。構成する式を持つのは
1 つだけであり、残る 2 つはパターンである。

SCAN src/ `Origin::Join`
  = src/rc_ir/ownership.rs: Origin::candidates -- パターン
  = src/rc_ir/ownership.rs: Origin::identity -- パターン
  = src/rc_ir/ownership.rs: Origin::of_candidates -- 構成。`candidates.len()` が 2 以上のときだけ作る

**前提 走査が `consume` を呼ぶ位置** --- `CancelAnalysis::consume` を呼ぶ式を持つ項目は次で尽きる。

SCAN src/ `self.consume(`
  = src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs -- `rhs_consumes` が `consumed` に積む各 `(var, leaf)` について 1 回
  = src/rc_ir/borrow.rs: CancelAnalysis::walk_inner -- `Destructure` の腕が `destructure_consumes` の挙げる各 leaf について 1 回

**前提 `origins` の memo を書く式** --- `VarTable::origins` を可変に借りる式を持つ項目は次で尽きる。

SCAN src/ `origins.borrow_mut()`
  = src/rc_ir/ownership.rs: origin -- `origin_inner` の答えを同じ鍵で入れる 1 つの式

**前提 `pending` と `outstanding` を変える式** --- `PendingRetains` の列から要素を取り除く式と、
`References` の個数を引く式を持つ項目は次で尽きる。

SCAN src/ `pending.remove(`
  = src/rc_ir/borrow.rs: un_bump -- `InBracket` の subtract の後、`outstanding` が空になった要素を外す

SCAN src/ `pending.retain(`
  = src/rc_ir/borrow.rs: CancelAnalysis::consume_objects -- 名指されたオブジェクトを含む要素を落とす

SCAN src/ `.subtract(`
  = src/rc_ir/borrow.rs: un_bump -- `innermost.outstanding` から `un_bumped` を引く

**DEF ii-a と ii-b の読み**。README の形で A19 (ii) を読む。**(ii-a)** は、各時点と各**計数下の**別名類 `C` に
ついて `held_ρ(τ, C) ≥ 0` であり、読む構文と `Retain`/`Release` がその類を名指す時点では
`held_ρ(τ, C) ≥ 1` であることである。**非負であることは、終端の `Ret` の消費を行った直後の時点に
ついても言う。** **(ii-b)** は、`bumps_ρ(τ, C) ≥ 1` である時点では
`held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` であることである。この文書が単に「A19 (ii)」と書くときは (ii-b)
を指す。計数下の類に限るのは、グローバル値を ρ-終端とする類に D34 の表が開始値を
与えないからである (D26)。**`held` の帰属を定めるのは D34 の表、`bumps` の帰属を定めるのは
A19 (ii-b) の本文である** -- 「`bumps_ρ(τ, C)` とは、時点 `τ` の `pending` の各要素 `p` について
D27 が定める `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和である」。

**上流の仮定は A2 を通して読む。** A6 (名前の一意性)、A9 (`Match` はアームを持つ)、A11 (スコープの
規律)、A13 (名前の形) はいずれも `borrow_ify` の入力にかかる仮定である。この文書が主語にする本体の
うち、`split_rc_units` の出力はその `borrow_ify` の入力そのものだが、`insert_rc` の入力と出力
(後者は `split_rc_units` の入力でもある) はその上流にある。A2 は
「**`insert_rc` と `split_rc_units` は束縛を作らず、名前を替えず、`Match` のアームを消さない。**」と
「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の入力と出力についても読める。**」を
述べ、A6・A9・A13 のそれぞれが「**`insert_rc` の入力と出力について読む段は A2 を引く。**」と書く。
A11 は「**この仮定が語るのは `borrow_ify` の入力である。**」と範囲を述べるので、A11 を引く段も A2 を
併せて引く。
**この文書でこの 4 つを引く段は、どれも A2 を併せて引く。** 上流について読む段だけを一覧にすると、
段が増えるたびに一覧が古くなるので、規則として書く。**`L11` はこの規則の外にある** -- その主語は
`borrow_ify` の入力と出力であり、A13 の範囲にそのまま入る。

**この文書の命題は `L1` から `L38` と番号を付ける。** 番号は固定された名札であり、間に挟むときは
`L13a` のように枝番を振る。`L33` から `L37` までの番号は使わない。他のファイルの命題を引く
ときは `p13 の L14` のように書く。

**DEF 例の型と op**。この文書が挙げる例のプログラムは、次の型と `Llvm` 演算を使う。`Arr` を boxed な
型、`I` を `is_fully_unboxed` が真の型、`Bl` を 2 つの変位がどちらも payload を持たない unbox union
とする。`Llvm` 演算 `alloc : () -> Arr` と `mkbl : () -> Bl` は結果の leaf を単一の `Fresh` と宣言し、
`zero : () -> I` は boxed leaf を持たない結果を返す。どれもオペランドを持たない。

**DEF 例の名前の取り方**。例のプログラムのパラメータと束縛変数の名前は、いずれも名前空間を持たない
ものを取る。例の関数は、その名前を鍵として `prog.funcs` に置く。

**DEF `C1` の本体**。`C1` は 2 つの関数からなる。関数 `f` (パラメータ `b : Arr`、**capture 無し**、
`borrowed_units` は空、返り値の型 `I`) の本体は `Let(z, Llvm(zero, []), Release(b, [], s, Ret(z)))` であり、関数 `main`
(パラメータ無し、capture 無し、返り値の型 `I`) の本体は次である。

```
Let(p, Llvm(alloc, []),
Let(q, Llvm(alloc, []),
Let(c, Llvm(mkbl, []),
Let(m, Match(c, [ MatchArm { tag: Some(0), payload: y0, body: Ret(p) },
                  MatchArm { tag: Some(1), payload: y1, body: Ret(q) } ]),
Retain(m, [], s,
Let(u, App(f, [p]),
Let(w, App(f, [q]),
Eval(m,
Release(m, [], s,
Ret(u))))))))))
```

## 2. 言明

README の A19 は (ii-a) と (ii-b) の果たす者に `insert_rc` を挙げる。その義務を「各別名類は、これから
起きるその類の処分をすべて賄えるだけの参照を持つ」と読んだ形を `(P-insert)` と呼ぶ。

> **(P-insert)** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各時点の各別名類
> `C` は、`ρ` の上でその類のスロットを名指す残りの消費と `Release` の個数以上の参照を持つ。

「残りの消費と `Release` の個数」を次の量として読む。

**DEF `Fut`**。`ρ` の上の節点 `n` と別名類 `C` について、`Fut_ρ(n, C)` を、`ρ` の上で `n` より後にある
次の事象の個数の和とする。

- `(w, μ) ∈ C` である leaf の D9 の意味の消費。1 つの構文が `C` のスロットを `k` 個消費するときは `k` を
  数える。
- `Release(v, π)` 節点であって `(v, λ) ∈ C` である `λ` を `π` の下に持つもの。そのような `λ` 1 つにつき
  1 を数える。

時点は節点の訪問の入口なので、`n` 自身の実行が行う事象は「`n` より後」に数える。

**DEF P-insert の読み**。`(P-insert)` の「参照を持つ」は `held_ρ(n, C)` (D34) と読む。すなわち
`(P-insert)` は次を主張する。

> `ρ` の上の各節点 `n` と各別名類 `C` について、`held_ρ(n, C) ≥ Fut_ρ(n, C)`。

**DEF 量化の範囲**。D34 は「`held_ρ(τ, C)` はその時点以後の `τ` についてだけ定まり、**`C` を渡る総和は
どれもその条件を付ける**」と述べる。ここでいう「その時点」は `C` の ρ-終端の変数が値を得る時点であり、
**この文書が `held_ρ(・, C)` を含む式 -- 不等式も等式も -- はどれも、`C` についてはその時点以後の点を
走る。** 等式の側にもこの範囲が要るのは、D34 がその時点より前の `held_ρ(・, C)` に値を与えないので、
そこでは等式の左辺が定義を持たないからである。

## 3. この文書が示すこと

- **R1**、**L1** - **L3**、**L30** (第 4 節)。`insert_rc` が反例の骨格 `S_main` を本体 `B_0` へ写す
  こと (`L1`)、`B_0` の実行路と別名類 (`L2`)、その上の `held` と `Fut` (`L3`)、そこから出る
  `(P-insert)` の破れ (R1)、およびその出力が D12 と A19 (ii) の 3 つの節を満たすこと -- すなわち破れが言明の側に
  あること (`L30`)。
- **R2**、**L4**、**L5** (第 5 節)。`held` が開始と増減で決まること (`L4`)、`Fut` から未実行の
  `Retain` の分を引いた形 `(P-insert-net)` を `p13-disposals-and-pending.md` の反例 `C1` が満たすこと
  (`L5`)。`C1` は A19 (ii) を破るので、`(P-insert-net)` だけから A19 (ii) は出ない (R2)。
- **L5a**、**L5b**、**L6**、**L7** (第 6 節)。走査の帳簿 `B(p, ρ)` が `outstanding` の一部と一致する
  こと (`L5a`)、名前が別名類を決めること (`L5b`)、bump の残高 `bumps = R - U - X` (`L6`)、および
  A19 (ii) が別名類ごとの台帳の不等式 `U + X ≥ D` と同値であること (`L7`)。
- **L8** - **L11** (第 7 節)。出力の節点が 7 つの構成式から来ることと `Retain` を作る位置が 4 つで
  あること (`L8`)、`insert_rc` の出力の
  `Retain` の位置の形 (`L9`)、その直後の構文が `v` の参照を手放すこと (`L9a`)、それが `C1` を弾くこと
  (`L10`)、およびその形が `cancel` の入力まで残ること (`L11`)。
- **L12**、**L13** (第 9 節)。A19 を読む 2 つの形が、どちらも他方を導かないこと。
- **L13a** - **L20** (第 10 節)。(O1)。移動の辺の両端が同じ別名類に属し `held` がスロットごとの割り当ての
  和であること (`L13a`)、`live_before` が自由変数と `live_after` の和であること (`L14`)、`Λ` と
  出力についての 5 つの性質 (`L15`)、借用するオペランドの leaf が素通しを宣言されず、素通しの宣言が
  結果の leaf について単射であること (`L16`)、
  終端の `Ret` を書き換える呼び出しの `live_after` が空であること (`L16a`)、遷移が割り当てを liveness の
  指示関数へ運ぶこと (`L17`)、`insert_rc` の liveness と実行時の参照の分布が一致すること
  (`L18`)、終端の `Ret` の前に `Retain` が立たないこと (`L18a`)、そこから出る別名類の粒度の RC 規律
  (`L19` (a)-(c))、節点の実行の途中の点で `held_ρ` が定まる類についての非負性
  (`L19` (d) -- A19 の (ii-c) の分)、および開始事象の一意性 (`L20`)。
- **L20a** - **L25** (第 11 節)。(O2)。この節が本体について読む 5 つを前提 (S) として括り出し、
  `insert_rc` の出力がそれを満たすこと (`L20a`)、11.1 節の量についての 4 つ (`L20b`)、名前の鎖が
  作る木 (`L21`)、木の根で読むと `held` と `bumps` になること (`L22`)、処分の事象に対する走査の応答
  (`L23`)、その部分木についての不等式 (`L24`)、そこから出る `held ≥ 1 + bumps` (`L25`)。
- **L38** (第 12 節)。A19 (ii-b) が活性化の終わりの 1 点先では偽であること。
- **L26** - **L29**、**L31**、**L31a**、**L32** (第 13 節)。`split_rc_units` について。書き換えの形
  (`L26`)、unit による leaf の分割
  (`L27`)、`insert_rc` の出す path が空列であること (`L28`)、束縛・`origin`・別名類・`held` が変わらない
  こと (`L29`)、A19 (ii-a) の保存 (`L31`)、A19 (ii-c) の保存 (`L31a`)、出力が (S) を満たすので
  `L25` から A19 (ii-b) が出ること (`L32`)。

## 4. R1: `(P-insert)` は `insert_rc` の出力で偽である

### 4.1 反例のプログラム

`DEF 例の型と op` の `Arr`・`I`・`alloc`・`zero` と、`DEF 例の名前の取り方` を使う。

**DEF 骨格 `S_f` と `S_main`**。次の 2 つの木を、関数 `f` と `main` の本体として `insert_rc` の入力に
据える。`DEF 骨格` より、この 2 つは骨格である。

骨格 `S_f` (関数 `f`、パラメータ `b : Arr`、**capture 無し**、`borrowed_units` は空、返り値の型 `I`):

```
Let(z, Llvm(zero, []), Ret(z))
```

骨格 `S_main` (関数 `main`、パラメータ無し、capture 無し、返り値の型 `I`):

```
Let(p, Llvm(alloc, []),
Let(u, App(f, [p]),
Let(w, App(f, [p]),
Ret(w))))
```

**capture の有無を書くのは、`insert_into_func` が未使用の capture にも `Release` を被せるからである。**
どちらの関数も capture を持たないので、その `Release` はどちらの出力にも立たない。

### 4.2 `L1` (`insert_rc` が出力する本体) <!--#48a9624-->

**言明**。`insert_rc` は `S_main` を次の本体 `B_0` に書き換える。

```
Let(p, Llvm(alloc, []),
Retain(p, [], RcState::Unknown,
Let(u, App(f, [p]),
Let(w, App(f, [p]),
Ret(w)))))
```

また `S_f` を `Release(b, [], RcState::Unknown, Let(z, Llvm(zero, []), Ret(z)))` に書き換える。

**証明**

<1>1. `needs_rc(p)` と `needs_rc(b)` は真であり、`needs_rc(u)`、`needs_rc(w)`、`needs_rc(z)` は偽で
      ある。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, CODE src/ast/types.rs:
     TypeNode::is_fully_unboxed
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` である。`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので、boxed な型では偽である。`ty(p) = ty(b) = Arr`
  は boxed なので `needs_rc` は真であり、`ty(u) = ty(w) = ty(z) = I` は `DEF 例の型と op` より
  `is_fully_unboxed` が真なので `needs_rc` は偽である。

<1>2. `p`、`u`、`w`、`z`、`b` の名前は `is_local` が真であり、`f` の名前は偽である。
  BY <ref id=8e3aff3/>, <ref id=cb35ab1/>, DEF 例の名前の取り方, CODE src/ast/name.rs: FullName::is_local,
     CODE src/ast/name.rs: NameSpace::is_local
  `FullName::is_local` の本体は `self.namespace.is_local()` であり、`NameSpace::is_local` の本体は
  `self.names.len() == 0` なので、`is_local` は名前空間が空かを答える。`DEF 例の名前の取り方` より
  `p`、`u`、`w`、`z`、`b` の名前は
  名前空間を持たない。`f` は `prog.funcs` の鍵であり、A13 は「最上位の記号の名前は局所名
  ではない」-- `FullName::is_local` が偽であり、`prog.funcs` の鍵はそのような名前である -- と述べる。
  A13 は `borrow_ify` の入力について語るが、A13 自身が「**`insert_rc` の入力と出力について読む段は
  A2 を引く。**」と述べ、A2 が「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の
  入力と出力についても読める。**」を与えるので、この本体についてこれを読める。

<1>3. `insert_rc` は `main` について `RcInserter::insert_into_func(main)` を呼び、それは
      `self.insert_into_expr(func.body, &Set::default())` を呼ぶ。
  BY CODE src/rc_ir/rc_insert.rs: insert_rc, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func

<1>4. `insert_into_expr(Ret(w), ∅)` は `(Ret(w), {w})` を返す。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>2, <ref id=3e6b0e0/>
  `insert_into_expr` の本体は `grow_stack(|| self.insert_into_expr_inner(node, live_after))` であり、
  A15 より `grow_stack` はその閉包をちょうど 1 回呼んでその返り値を返すので、
  `insert_into_expr` の値は `insert_into_expr_inner` の値である。
  この腕は `live = live_after ∪ {x}` を作り (`insert_if_local` は `name.is_local()` のときだけ
  その名前を集合へ入れる。<1>2 より `w` は局所名なので入る)、
  `retain_if_live(&x, live_after, ret)` を呼ぶ。`live_after = ∅` は `w` を含まないので、`retain_if_live`
  の条件 `live.contains(&var.name)` が偽であり、節点はそのまま返る。

<1>5. `insert_into_expr(Let(w, App(f, [p]), Ret(w)), ∅)` は
      `(Let(w, App(f, [p]), Ret(w)), {p})` を返す。
  <2>1. この呼び出しは `insert_into_operation_let(w, App(f, [p]), Ret(w), source, ∅)` に入る。
    BY <ref id=3e6b0e0/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Let(x, rhs, cont)` の腕
    A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。右辺は `Match` では
    ないので、その腕は `insert_into_operation_let` へ振り分ける。
  <2>2. `live_cont = {w}` である。
    BY <1>4, <ref id=3e6b0e0/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    この関数はまず `self.insert_into_expr(cont, live_after)` を呼ぶ。A15 よりその呼び出しは
    `insert_into_expr_inner` をちょうど 1 回呼ぶので、返る値は <1>4 のものである。
  <2>3. `retains_before` と `releases_after` はどちらも空である。
    BY CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
       RcInserter::insert_into_operation_let, <1>2, <2>2
    `rhs_operands(App(f, [p]))` は `[(f, Own), (p, Own)]` である。逆順の走査はまず `p` を見る。
    `live_after_operand` は `live_cont = {w}` の写しなので `p` を含まず、`used_later` は偽である。
    所有は `Own` なので `releases_after` には入らず、`used_later` が偽なので `retains_before` にも
    入らない。次に `f` を見るが、<1>2 より `f.name.is_local()` が偽なので何も起きない。
  <2>4. `after` は空である。
    BY <2>2, <2>3, <1>1, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `after` は `releases_after` (<2>3 より空) に、`!live_cont.contains(&x.name) && self.needs_rc(&x)` の
    ときだけ `x = w` を足したものである。<2>2 より `live_cont = {w}` は `w` を含むので、この条件は偽で
    ある。
  <2>5. 返る節点は `Let(w, App(f, [p]), Ret(w))` である。
    BY <2>3, <2>4, CODE src/rc_ir/rc_insert.rs: build_releases,
       CODE src/rc_ir/rc_insert.rs: build_retains, EXT `Iterator::fold`, EXT `Iterator::rev`
    `build_releases(vars, cont)` と `build_retains(vars, cont)` はどちらも
    `vars.into_iter().rev().fold(cont, …)` である。`EXT Iterator::rev` より逆順の反復子も空であり、
    `EXT Iterator::fold` より元が 1 つも無い `fold` は初期値 `cont` をそのまま返す。
  <2>6. QED
    BY <2>2, <2>3, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `live_before` は `live_cont` から `x = w` を除き、各オペランドの局所名を足したものである。
    `{w} \ {w} = ∅` に `p` が入り、`f` は <1>2 より入らない。

<1>6. `insert_into_expr(Let(u, App(f, [p]), <5 の骨格>), ∅)` は
      `(Retain(p, [], Unknown, Let(u, App(f, [p]), Let(w, App(f, [p]), Ret(w)))), {p})` を返す。
  <2>1. この呼び出しは `insert_into_operation_let(u, App(f, [p]), ·, source, ∅)` に入り、
        `live_cont = {p}` である。
    BY <1>5, <ref id=3e6b0e0/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の
       `RcExpr::Let(x, rhs, cont)` の腕
    A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。右辺は `Match` では
    ないのでその腕は `insert_into_operation_let` へ振り分け、`live_cont` は継続についての
    `insert_into_expr` の返り値、すなわち <1>5 の `{p}` である。
  <2>2. `retains_before = [p]` であり、`releases_after` は空である。
    BY <2>1, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
       RcInserter::insert_into_operation_let
    逆順の走査はまず `p` を見る。`live_after_operand` は `live_cont = {p}` の写しなので `p` を含み、
    `used_later` は真である。所有は `Own` で `needs_rc(p)` が真 (<1>1) なので `retains_before` に `p` が
    入る。`f` は <1>2 より飛ばされる。
  <2>3. `after` は空である。
    BY <2>1, <2>2, <1>1, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `releases_after` は空であり、`x = u` については `needs_rc(u)` が偽 (<1>1) なので足されない。
  <2>4. 返る節点は `Retain(p, [], RcState::Unknown, Let(u, App(f, [p]), <5 の本体>))` である。
    BY <2>2, <2>3, CODE src/rc_ir/rc_insert.rs: build_retains
    `build_retains([p], node)` は `RcExpr::Retain(p, vec![], RcState::Unknown, node)` を作る。
  <2>5. QED
    BY <2>1, <2>2, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let
    `live_before` は `{p} \ {u} = {p}` に `p` を足したもので `{p}` である。

<1>7. `insert_into_expr(S_main, ∅)` は `(B_0, ∅)` を返す。
  BY <1>6, <1>1, <ref id=3e6b0e0/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs:
     RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases,
     EXT `Iterator::fold`, EXT `Iterator::rev`
  A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼び、右辺は `Match` では
  ないので `insert_into_expr_inner` の `RcExpr::Let(x, rhs, cont)` の腕は
  `insert_into_operation_let` へ振り分ける。継続についての呼び出しの返り値は
  <1>6 のものである。**空の `retains_before`・`releases_after` から鎖が被さらないことは
  `build_retains` と `build_releases` の本体による** -- どちらも
  `vars.into_iter().rev().fold(cont, …)` であり、`EXT Iterator::rev` より逆順の反復子も空、
  `EXT Iterator::fold` より元が 1 つも無い `fold` は初期値 `cont` をそのまま返す。
  `rhs_operands(Llvm(alloc, []))` は空の列である (`alloc` はオペランドを持たない)。よって
  `retains_before` と `releases_after` は空である。`x = p` については `live_cont = {p}` が `p` を含むので
  `after` に入らない。`live_before` は `{p} \ {p} = ∅` である。

<1>8. `main` について `insert_into_func` はこの節点をそのまま `func.body` にする。
  BY <1>7, EXT 空を作る `Default`, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func
  `live` は空なので `for name in &live` の表明は走らず、`main` はパラメータも capture も持たないので
  `unused` は空であり、`build_releases(∅, body)` は `body` を返す。

<1>9. `f` について `insert_rc` は `Release(b, [], RcState::Unknown, Let(z, Llvm(zero, []), Ret(z)))` を
      作る。
  BY <1>1, <1>2, <ref id=3e6b0e0/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: insert_if_local,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: rhs_operands,
     CODE src/rc_ir/rc_insert.rs: build_releases
  A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。
  `insert_into_expr_inner` の `RcExpr::Ret(x)` の腕は `live = live_after ∪ {z}` を作り
  (`insert_if_local`、<1>2 より `z` は入る)、`retain_if_live(&z, live_after, ret)` を呼ぶ。
  `live_after = ∅` は `z` を含まないので `live.contains(&var.name)` が偽であり、節点はそのまま返る。
  `Let(z, Llvm(zero, []), Ret(z))` はオペランドを持たず、`live_cont = {z}` が `x = z` を含むので
  `after` は空であり、`live_before = {z} \ {z} = ∅` である。`insert_into_func` はこの `live` が `b` を
  含まないことと `needs_rc(b)` が真であること (<1>1) から `unused = [b]` を作り、
  `build_releases([b], body)` が `Release(b, [], RcState::Unknown, ·)` を被せる。

<1>10. QED
  BY <1>3, <1>8, <1>9

### 4.3 `L2` (`B_0` の実行路と別名類) <!--#4eea96b-->

**言明**。`B_0` は実行路 `ρ_0` をちょうど 1 本持つ。`ρ_0` を辿る活性化について、`ρ_0` の上の
スロットは `(p, [])` ただ 1 つであり、`B_0` に現れる残りの `RcVar` -- `u`、`w`、`f` -- はスロットを
持たない (`f` の各 boxed leaf との対は D6 の記号の位置であり、そこが指すのは funptr かグローバル状態の
オブジェクトである)。`origin(p, []) = Exactly((p, []))` であり、`(p, [])` は
別名類 `C_p = {(p, [])}` の唯一のスロットであり、`C_p` の ρ-終端は `(p, [])` 自身、`obj(C_p)` は
`alloc` が割り当てたオブジェクトで、これは D26 の意味で計数下である。よって `ρ_0` の上の計数下の
別名類は `C_p` ただ 1 つである。

**証明**

<1>1. `B_0` は `Match` を含まないので、実行路は 1 本である。
  BY <ref id=ca36627/>
  D3 の規則で分岐が生じるのは `Let(x, Match(v, arms), k)` の行だけである。

<1>2. `ρ_0` の上のスロットのうち、変数が `p`、`u`、`w` のいずれかであるものは `(p, [])` だけである。
      `[]` は `ty(p) = Arr` の inhabited (D16) な boxed leaf であり、`p` が値を得た後 `(p, [])` は
      D6 のスロットである。
  BY <ref id=0594f24/>, <ref id=596a46d/>, <ref id=66c9670/>, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/ast/types.rs: TypeNode::is_unbox, CODE src/ast/types.rs: TypeNode::is_box
  `is_fully_unboxed` は `if self.is_box(type_env) { return false; }` で始まるので、boxed な `Arr` では
  偽である。`is_box` の本体は `!self.is_unbox(type_env)` であり、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` なので、`is_box` が真の `Arr` では
  `is_closure` が偽である。よって D4 の判定は
  第 1 規則 (`is_fully_unboxed`) と第 2 規則 (クロージャ) を抜けて第 3 規則 (`is_box`) に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。この leaf は第 3 規則が積んだものであって unbox union の節を
  1 つも通らないので、D16 より常に inhabited である。D6 よりスロットは、その実行路の上でその時点までに
  値を得た変数と、その型の inhabited な boxed leaf の対であり、`(p, [])` はその形である。
  `ty(u) = ty(w) = I` は `DEF 例の型と op` より `is_fully_unboxed` が真なので、D4 の第 1 規則より leaf を
  持たない。

<1>2a. `B_0` に現れる残りの `RcVar` は `App` の callee `f` であり、その各 boxed leaf `λ` について
       `(f, λ)` は記号の位置であってスロットではない。
  BY <ref id=596a46d/>, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/lower.rs: Lowerer::lower_var
  `collect_bindings` は `f` に束縛を入れない (`f` は `B_0` のどの `Let`、`Destructure`、`Match` の
  束縛変数でもない)。`f` はパラメータでも capture でもないので `vars.bindings` に束縛を持たず、
  D6 の「**値を得る形は 3 つあり、スロットが在るのはそのうち 2 つである。**」の 3 つ目 -- `Lowerer::lower_var` の `resolve` が `None` を返す腕が作る名前 --
  である。D6 よりその対は記号の位置であってスロットではない。

<1>3. `origin(p, []) = Origin::Exactly((p, []))` である。
  BY CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under, <ref id=e11772a/>, <ref id=b1f6e13/>, <ref id=3e6b0e0/>, DEF 例の型と op
  `origin` は `vars.origins` を鍵で引き、項が無ければ `origin_inner` を走らせて (A15 より
  `grow_stack` はその閉包をちょうど 1 回呼ぶ) その答えを返す。P2a より答えは memo の状態に依らない
  ので、`origin_inner` が返す値がそのまま `origin(p, [])` の値である。
  `collect_bindings` は `Let(p, Llvm(alloc, []), ·)` に `Binding::Llvm(alloc, [], Arr)` を入れる。
  `origin_inner` の `Binding::Llvm` の腕は、まず `decl.leaf_origins_at([])` を `as_arg_projection` に
  掛ける。A3 と `DEF 例の型と op` より宣言は単一の `Fresh` であり、`as_arg_projection` は
  `LeafOrigin::Fresh` に `None` を返す。次に
  `origin_from_leaves_under` を呼ぶ。オペランドが無いので `operand_units` は空、`Fresh` により
  `produced_here` が真になり、`reached = [Origin::Exactly((p, []))]` の 1 元である。`reached` の元が
  すべて等しいので、その値がそのまま返る。

<1>4. `(p, [])` は ρ-終端であり、`C_p = {(p, [])}` である。
  BY <1>2, <1>2a, <1>3, <ref id=596a46d/>, <ref id=d59f90b/>, <ref id=30d6238/>, <ref id=e11772a/>, DEF 例の型と op,
     CODE src/rc_ir/ownership.rs: collect_bindings
  D33 は ρ-歩みが止まる位置を 3 種で挙げ、その第 2 種は「`Binding::Llvm` であって、`λ` の宣言が
  単一の `Fresh` または単一の `Unknown` である位置」である。<1>3 より `p` の束縛は
  `collect_bindings` が入れた `Binding::Llvm(alloc, [], Arr)` であり、A3 と `DEF 例の型と op` より
  `alloc` の結果の leaf `[]` の宣言は単一の `Fresh` なので、`(p, [])` はこの第 2 種に当たる。
  D17 の第 2 項も、宣言が単一の `Fresh` のとき鎖はそこで止まり対応するスロットはその位置自身であると
  述べる。よって `(p, [])` は ρ-終端である。
  別名類は ρ-終端が等しいスロットの集まりなので (D33)、`(p, [])` を ρ-終端とするスロットを数えれば
  よい。`B_0` に現れる `RcVar` は `p`、`u`、`w`、`f` であり、<1>2 より前 3 者のスロットは `(p, [])`
  だけ、<1>2a より `f` はスロットを持たない。

<1>5. `obj(C_p)` は計数下である。
  BY <ref id=e11772a/>, <ref id=88a06de/>, DEF 例の型と op
  A3 が単一の `Fresh` の行を字義どおりに読ませないのは「実行時に参照カウントで分岐する op」-- その
  一意の腕がオペランドのオブジェクトをそのまま返す op -- についてであり、`DEF 例の型と op` より `alloc` は
  オペランドを持たないのでその形ではない。よってこの op についてはこの行を字義どおりに読めて、単一の
  `Fresh` を宣言する leaf には新しく割り当てたオブジェクトへの参照が置かれる。D26 より
  割り当てられたオブジェクトは計数下である。

<1>6. QED
  BY <1>1, <1>2, <1>2a, <1>3, <1>4, <1>5

### 4.4 `L3` (`B_0` の上の `held` と `Fut`) <!--#764c202-->

**言明**。`n_R` を `B_0` の `Retain(p, [])` 節点とする。`ρ_0` の上で `C_p` のスロットを名指す
D9 の消費は 2 つの `App(f, [p])` の引数の位置の 2 つであり、`ρ_0` の上に `Release` 節点は無い。
`held_ρ0(n_R, C_p) = 1` であり、`Fut_ρ0(n_R, C_p) = 2` である。

**証明**

<1>1. `f` のパラメータ `b` の unit は `f` が所有する。
  BY <ref id=48a9624/>, <ref id=dca1c02/>
  `L15` (e) より `insert_rc` の出力のすべての関数の `borrowed_units` は空であり、そのすべての
  パラメータ・capture の unit をその関数が所有する。`B_0` は `insert_rc` の出力の本体なので (`L1`)、
  同じプログラムの `f` にこれが当たる。

<1>2. `ρ_0` の上で `C_p` のスロットを名指す D9 の消費は、2 つの `App(f, [p])` の引数の位置の 2 つである。
  BY <ref id=48a9624/>, <ref id=4eea96b/>, <ref id=dca1c02/>, <1>1, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=ff5985d/>, DEF 例の型と op
  `L1` より `B_0` の 5 つの節点は `Let(p, Llvm(alloc, []), ・)`、`Retain(p, [])`、2 つの
  `Let(・, App(f, [p]), ・)`、`Ret(w)` である。D9 の消費の表の 6 行をこの 5 つに当てる。
  **`App` の行**: 呼び出し先が所有する位置の引数の leaf が消費される。D23 より D9 の `App` の行が
  言う呼び出し先はプログラムの `funcs` の関数であり、`L15` (e) よりその全パラメータの unit が
  所有されるので、どちらの `App(f, [p])` も引数の位置の leaf を消費する。`L2` より `ty(p)` の boxed
  leaf は `[]` の 1 つでそのスロットは `(p, []) ∈ C_p` なので、2 つの `App` はそれぞれ `C_p` の
  スロットを 1 つ消費する。同じ行が挙げる callee の leaf は `(f, ・)` であり、`L2` よりそれは D6 の
  記号の位置であってスロットではないので `C_p` のスロットではない。
  **`Llvm` の行**: `B_0` の唯一の `Llvm` 節点は `Llvm(alloc, [])` であり、`DEF 例の型と op` より
  `alloc` はオペランドを持たないので、この行が挙げる leaf は無い。
  **`Closure` の行と `Destructure` の 2 行**: `B_0` にその構文の節点は無い (`L1`)。
  **終端の `Ret(x)` の行**: 終端は `Ret(w)` であり、`ty(w) = I` は `DEF 例の型と op` より
  `is_fully_unboxed` が真なので D4 の第 1 規則より boxed leaf を持たない。
  よって `C_p` のスロットを名指す消費は 2 つの `App` の引数の位置の 2 つである。

<1>3. `ρ_0` の上に `Release` 節点は無い。
  BY <ref id=48a9624/>
  `B_0` の 5 つの節点は `Let`、`Retain`、`Let`、`Let`、`Ret` である。

<1>4. `Fut_ρ0(n_R, C_p) = 2` である。
  BY <1>2, <1>3, DEF `Fut`
  2 つの消費はどちらも `n_R` より後にある。

<1>5. `held_ρ0(n_R, C_p) = 1` である。
  BY <ref id=4eea96b/>, <ref id=f06144e/>, <ref id=9d5d254/>
  `C_p` の ρ-終端 `(p, [])` は D10 の生成の表の `Llvm` の行で作られる (宣言が単一の `Arg` でないため)。
  表の第 1 行より `held` は 1 から始まる。`n_R` の入口までに起きた事象はこの生成だけである -- `n_R` は
  `Let(p, Llvm(alloc, []), ·)` の直後の節点であり、その間に `Retain`、`Release`、消費は無い。

<1>6. QED
  BY <1>2, <1>3, <1>4, <1>5

### 4.5 R1 の結論

**R1**。`(P-insert)` は偽である。

**証明**

<1>1. `B_0` は `insert_rc` の出力の本体である。
  BY <ref id=48a9624/>

<1>2. `B_0` は実行路 `ρ_0` を持ち、`C_p` は `ρ_0` を辿る活性化の別名類である。
  BY <ref id=4eea96b/>

<1>3. `ρ_0` の上の節点 `n_R` の入口という時点において
      `held_ρ0(n_R, C_p) = 1 < 2 = Fut_ρ0(n_R, C_p)` である。
  BY <ref id=764c202/>

<1>4. QED
  BY <1>1, <1>2, <1>3, DEF P-insert の読み
  `DEF P-insert の読み` より `(P-insert)` は「`ρ` の上の各節点 `n` と各別名類 `C` について
  `held_ρ(n, C) ≥ Fut_ρ(n, C)`」である。<1>1 から <1>3 が、`insert_rc` の出力の 1 つの本体と、
  その 1 本の実行路と、その上の 1 つの別名類と、1 つの節点でその不等式が破れる例を与える。

### 4.6 `L30` (`B_0` の破れは言明の側にある) <!--#6e54a22-->

**言明**。`DEF 骨格 S_f と S_main` の 2 つを本体とする関数 `f`・`main` だけからなり、グローバル
初期化子を持たないプログラムに `insert_rc` を掛けた出力を `P_0` とする。

- **(a)** `P_0` は D12 の意味で RC 規律を満たす。
- **(b)** `B_0` は A19 (ii) の 3 つの節をすべて満たす -- (ii-a) と (ii-b) は
  `DEF ii-a と ii-b の読み` の形で、(ii-c) は第 1 節の `DEF 時点` が引く形で。

すなわち `(P-insert)` の破れ (R1) はコードの欠陥ではなく、言明の側の欠陥である。

**証明**

<1>1. `P_0` の関数は `main` と `f` の 2 つであり、その本体は `B_0` と
      `Release(b, [], RcState::Unknown, Let(z, Llvm(zero, []), Ret(z)))` である。`P_0` はグローバル
      初期化子を持たない。どちらの関数もすべてのパラメータ・capture の unit を所有する。
  BY <ref id=48a9624/>, <ref id=dca1c02/>, DEF 骨格 S_f と S_main, CODE src/rc_ir/rc_insert.rs: insert_rc
  `L1` が 2 つの本体を与え、`L15` (e) が所有を与える。`insert_rc` は `prog.funcs` の各値の `body` と
  `prog.globals` の各エントリの `init` を差し替えるだけで、関数も初期化子も足さないので、`P_0` の
  関数は入力の 2 つであり、初期化子は入力と同じく 1 つも無い。

<1>2. `f` の本体の実行路は 1 本であり、その上のスロットは `(b, [])` だけである。`(b, [])` は自分
      だけからなる別名類 `C_b` の ρ-終端である。`C_b` が計数下であるとき、`Obl` は
      `{obj(b, [])}` で始まり `Release(b, [])` がその 1 つを取り除いて空になり、
      `held_ρ(・, C_b)` は 1 で始まり `Release` の後 0 である。
  BY <1>1, <ref id=ca36627/>, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=94fad8e/>, DEF 骨格 S_f と S_main, DEF 例の型と op,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     CODE src/ast/types.rs: TypeNode::is_box
  `Match` が無いので実行路は 1 本である。`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `Arr` では偽であり、`is_box` の
  本体は `!self.is_unbox(type_env)`、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` なので、`is_box` が真の `Arr` では
  `is_closure` が偽である。よって D4 の判定は `Arr` について第 1 規則 (`is_fully_unboxed`) と
  第 2 規則 (クロージャ) を抜けて第 3 規則 (`is_box`) に着くので `boxed_leaf_paths(Arr) = {[]}` で
  あり、`ty(z) = I` は `DEF 例の型と op` より `is_fully_unboxed` が真なので第 1 規則より boxed leaf を
  持たない。`b` はパラメータなので
  `VarTable::of` が `Binding::Param` を入れ、D33 が歩みを止める第 1 種に当たるので `(b, [])` は
  自分の類の ρ-終端である。D10 の初期値は所有するパラメータの inhabited な各 leaf につき参照を
  1 つ入れる (<1>1)。`Llvm(zero, [])` はオペランドを持たず、その結果の型は boxed leaf を持たないので
  D9 の消費にも D10 の生成にも当たらない。終端の `Ret(z)` は `ty(z) = I` に boxed leaf が無いので
  何も消費しない。`held` の推移は `L4` の等式に開始 1 つと `Release` 1 つを当てたものである。

<1>3. `ρ_0` の上の計数下の別名類は `C_p` だけであり、`held_ρ0(・, C_p)` は、`Let(p, ・)` の段の後 1、
      `Retain(p, [])` の後 2、最初の `App(f, [p])` の消費の後 1、2 番目の `App(f, [p])` の消費の後
      0 である。`Obl` は `{O_p}` で始まり、同じ 4 つの点で `{O_p, O_p}`、`{O_p}`、空と動く。
  BY <ref id=4eea96b/>, <ref id=764c202/>, <ref id=94fad8e/>, <ref id=0594f24/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>
  `L2` が「`ρ_0` の上の計数下の別名類は `C_p` ただ 1 つ」を、`L3` が 2 つの消費と `Release` 節点が
  無いことを与える。`held` は `L4` の等式に生成 1 つ・`Retain` 1 つ・消費 2 つを順に当てたもので
  ある。`Obl` については、D10 の生成の `Llvm` の行が最初の参照を作り、`Retain` の行がもう 1 つ加え、
  `App` の引数の消費が 1 つずつ取り除いて呼び出し先へ渡す。終端の `Ret(w)` は `ty(w) = I` が
  D4 の第 1 規則より boxed leaf を持たないので何も消費しない。

<1>4. `P_0` のどちらの本体についても、その活性化の各時点で、計数下の別名類 `C` が
      `held(・, C) ≥ 1` を満たすならば `H(obj(C)) ≥ 1` であり、その活性化がその時点まで解放に
      ついて閉じている (D11a) ならば `obj(C)` はその時点で解放されていない。
  BY <1>1, <1>2, <1>3, <ref id=9f1cf6c/>, <ref id=859cf84/>, <ref id=c232680/>, <ref id=30d6238/>, <ref id=c2ea78f/>
  D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と
  定め、その不等式の本文 -- 角括弧、総和 `S`、`d(C)` -- は A19 (i) が置く。<1>1 と
  `L13a` (c) より `β ≡ 0` なので、A19 (i) の角括弧は 0 であり `d(C') = held(・, C')` である。
  <1>2 と <1>3 より、どちらの本体でもスロットを持つ別名類は 1 つだけなので、A19 (i) の総和 `S` は
  `C` を含むならその 1 項だけからなる。よって `H(obj(C)) ≥ held(・, C) ≥ 1` である。D11a は、
  解放について閉じている時点では `H(O) ≥ 1` である各計数下オブジェクトが解放されていないと定める。

<1>5. `f` の本体は D11 を満たす。
  BY <1>2, <1>4, <ref id=56c2068/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=fd95f12/>, <ref id=88a06de/>, <ref id=ec8d1a0/>, <ref id=b6673ca/>
  (S-a) と (S-b) は `C_b` が計数下であるかどうかで 2 つの場合に分かれる。**D26 よりこの 2 つで尽きる**
  -- オブジェクトは計数下かグローバル状態かのどちらかであり、1 つの活性化の間その区別は変わらない。
  `C_b` が計数下であるとき、(S-a): `Obl` から参照を取り除く操作は `Release(b, [])` の 1 つだけで
  あり、<1>2 よりその時点の `Obl` はその参照を持つ。(S-b): 終端の `Ret(z)` の消費 (何も消費しない) の
  後、`Obl` は空である。
  `obj(b, [])` がグローバル状態であるとき、D26 よりその leaf は D8 の意味の参照を持たないので、D10 の
  初期値は空であり、`Release(b, [])` は `Obl` から何も取り除かない。よって (S-a) は取り除く操作が
  1 つも無いので、(S-b) は `Obl` が空のままなので成り立つ。
  (S-c): D7 の読む構文はこの本体に `Let(z, Llvm(zero, []), ・)` の 1 つだけ在り、`zero` は
  オペランドを持たないので読むオブジェクトが無い (A26 の順序の節はこの節点について空虚に当たる)。
  触れるのは `Release(b, [])` である。D24 の「**読みの直前の点では、勘定は直前の段内の点のものである。**」
  より、触れる動作の直前の点の勘定は直前の段内の点のものである。**その段内の点は、この節点の入口の
  勘定を持つ** -- この節点の入口から触れる動作までのあいだに、この段は参照を処分しておらず
  (D10 の `Release` の行より、この節点の処分はその触れる動作そのものである)、D24 の (F) の解放を
  起こすのは参照の処分なので、そのあいだにどのオブジェクトも解放されない。<1>2 よりその入口で
  `held_ρ(・, C_b) = 1` なので、`C_b` が計数下なら <1>4 より
  `obj(b, [])` は解放されていない。`C_b` が計数下でなければ、`obj(b, [])` はグローバル状態であって
  解放されることが無い (D26、A8)。

<1>6. `B_0` は D11 を満たす。
  BY <1>3, <1>4, <ref id=4eea96b/>, <ref id=c2ea78f/>, <ref id=56c2068/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=88a06de/>, <ref id=b6673ca/>, <ref id=fd95f12/>
  (S-a): `Obl` から参照を取り除くのは 2 つの `App` の消費だけであり、<1>3 よりどちらの時点でも
  `Obl` は取り除かれる参照を持つ。(S-b): 終端の `Ret(w)` の消費の後、`Obl` は空である。
  (S-c): D7 の読む構文はこの本体に 3 つ在る -- `Let(p, Llvm(alloc, []), ・)` はオペランドを持たない
  ので読むオブジェクトが無く、2 つの `Let(・, App(f, [p]), ・)` は callee `f` と引数 `p` を読む。
  触れるのは `Retain(p, [])` である。callee の側は `L2` より記号の位置であり、`L13a` (i) より
  記号の位置が指すのはグローバル状態のオブジェクトであって、A8 と D26 よりそれが解放されることは
  無い (funptr 型の名前は `L13a` (i) より boxed leaf を持たないので記号の位置を作らず、読む
  オブジェクトを持たない)。
  `p` の側と `Retain` の側については、D24 の「**読みの直前の点では、勘定は直前の段内の点のもので
  ある。**」より、読み・触れる動作の直前の点の勘定は直前の段内の点のものである。**その段内の点は、
  その節点の入口の勘定を持つ** -- `App` の節点では A26 より読みはこの節点が行うどの手放しよりも前に
  起き、`Retain` の節点ではその段が参照を 1 つも処分しない (D10 の `Retain` の行) ので、どちらでも
  節点の入口からその点までのあいだにこの段は参照を処分しておらず、D24 の (F) の解放を起こすのは
  参照の処分なので、そのあいだにどのオブジェクトも解放されない。<1>3 より
  `Retain(p, [])` の入口で `held = 1`、最初の `App` の入口で `held = 2`、2 番目の `App` の入口で
  `held = 1` なので、<1>4 より `O_p` はどの点でも解放されていない。

<1>7. (a)。
  BY <1>1, <1>5, <1>6, <ref id=3d96eb8/>
  D12 は「`P` のすべての関数の本体と、すべてのグローバル初期化子の `init` が、`P` の
  `borrowed_units` が定める所有と借用の割り当て (D14) の下で RC 規律を満たす (D11) こと」である。
  <1>1 より `P_0` の本体は 2 つで尽き、グローバル初期化子は無い。

<1>8. `Retain(p, [])` の訪問が押し込む要素の `outstanding` は `{(p, []): 1}` であり、最初の
      `App(f, [p])` の訪問がその要素を `pending` から取り除く。よって `bumps_ρ0(・, C_p)` は
      `Retain(p, [])` の後から最初の `App` の消費までが 1 で、他の時点では 0 である。
  BY <ref id=4eea96b/>, <ref id=764c202/>, <ref id=dca1c02/>, <ref id=cbc4a1c/>, <ref id=8093b68/>, <ref id=9f1cf6c/>, <ref id=66c9670/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=ef8efc4/>, <ref id=9d74736/>, CODE src/rc_ir/ownership.rs: acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/ownership.rs: Origin::acted_on,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/ownership.rs: rhs_consumes,
     CODE src/rc_ir/ownership.rs: all_owned_units, CODE src/rc_ir/borrow.rs: cancel
  `walk_inner` の `RcExpr::Retain` の腕は `outstanding` に `self.acted_references(p, [])` を置く。
  **その呼び先は `CancelAnalysis` のメソッドであり、それは自分の `vars` と `type_env` を添えて
  `ownership::acted_references` を呼び、その返り値をそのまま返す** -- 間に挟まるのは
  `!references.is_empty()` の表明だけである。D15 より
  それは `[]` の下の各 boxed leaf の `origin` の `identity` を数えたものであり、`L2` より `ty(p)` の
  boxed leaf は `[]` の 1 つ、`origin(p, []) = Exactly((p, []))` なので
  `{(p, []): 1}` である。D27 が数えるのは inhabited (D16) かつ計数下 (D26) の leaf に限るが、
  `L2` より `ρ_0` を辿るどの活性化でも `(p, [])` は `ρ_0` の上のスロットであり、D6 よりスロットの
  leaf は inhabited であって、`L2` より `obj(C_p)` は計数下なので、押し込まれた要素の `B` も
  `{(p, []): 1}` である。最初の
  `App(f, [p])` の訪問は `consume_rhs` を通じて `rhs_consumes` が挙げる leaf について `consume` を
  呼ぶ。**その位置が所有位置であることは `owns` が決める** -- `consume_rhs` が `rhs_consumes` に渡す
  `owns` は `self.owned_units.contains(…)` であり、`cancel` は `owned_units` に
  `all_owned_units(prog, type_env)` を置く。D14 より `all_owned_units` が返すのは各関数の
  パラメータ・capture の unit のうち `borrowed_units` に入らないものであり、`L15` (e) より
  `borrowed_units` はどの関数でも空なので、`owns` は常に真である。よって `rhs_consumes` は
  `(p, [])` を挙げる (`L3` が D9 の側について同じことを述べる)。`consume` は
  `origin(p, []).acted_on() = [(p, [])]` を `consume_objects` へ渡し、その要素の `outstanding` は
  `(p, [])` を名指すので取り除かれる。`walk_inner` が `pending` に要素を足すのは `RcExpr::Retain` の
  腕だけであり、`B_0` の `Retain` 節点はこの 1 つである。
  **名前から類への集約は A19 (ii-b) の本文が定める** -- 「`bumps_ρ(τ, C)` とは、時点 `τ` の
  `pending` の各要素 `p` について D27 が定める `B(p, ρ)` のうち、`C` のスロットの `origin` の
  identity に付く分の総和である」。`L2` より `C_p` のスロットは `(p, [])` 1 つで、その `identity` は
  `(p, [])` なので、`bumps_ρ0(・, C_p)` はこの要素の `B` の `(p, [])` の分そのものである。

<1>9. (ii-a) と (ii-b)。
  BY <1>3, <1>6, <1>8, DEF ii-a と ii-b の読み
  `DEF ii-a と ii-b の読み` では (ii-a) は「各時点で `held ≥ 0`、読む構文と `Retain`/`Release` がその類を名指す
  時点では `held ≥ 1`、終端の `Ret` の消費を行った直後の時点でも `held ≥ 0`」であり、(ii-b) は
  「`bumps ≥ 1` である時点で `held ≥ 1 + bumps`」である。<1>3 より `held` は `1, 2, 1, 0` と動いて
  常に 0 以上であり、終端の `Ret(w)` は何も消費しないのでその直後も 0 である。この類を名指す
  読む構文と `Retain` は <1>6 が挙げる 3 つであり、そこで `held` は 1・2・1 である。<1>8 より
  `bumps ≥ 1` であるのは `Retain(p, [])` の後から最初の `App` の消費までであり、<1>3 より
  その間 `held = 2 = 1 + 1` である。

<1>9a. (ii-c)。
  BY <1>1, <ref id=48a9624/>, <ref id=f941b4f/>, DEF 時点
  `L1` と <1>1 より `B_0` は `insert_rc` の出力の本体である。`L19` (d) の第 2 文が、`insert_rc` の
  出力の各本体の各節点の実行の途中の各点 (D24 の段内の点) について、その点で `held_ρ` が定まる各計数下の
  別名類 `C` の `held_ρ(・, C)` が 0 以上であることを与える。`DEF 時点` が引く A19 (ii-c) の量化は
  その形である。

<1>10. QED
  BY <1>7, <1>9, <1>9a

**破れているのは言明の側である。** `Fut_ρ(n, C)` は `n` より後の処分だけを数え、`n` より後の `Retain` が
作る参照を数えない。`held_ρ(n, C)` はその時点の参照の個数なので、後で `Retain` が増やす分を先取りして
持ってはいない。第 5 節がこの数え落としを直した形を扱う。

## 5. R2: 数え落としを直した形は A19 (ii) を果たさない

**DEF 関数 `h`**。`DEF 例の型と op` に、関数 `h` (パラメータ `a : Arr`、**capture 無し**、
`borrowed_units` は空、返り値の型 `Arr`、本体 `Ret(a)`) を足す。`h` を使うのは `L13` である。

**DEF `Ret#`**。`ρ` の上の節点 `n` と別名類 `C` について、`Ret#_ρ(n, C)` を、`ρ` の上で `n` より後にある
`Retain(v, π)` 節点であって `(v, λ) ∈ C` である `λ` を `π` の下に持つものについて、そのような `λ` の
個数の総和とする。`DEF Fut` と同じく、`n` 自身が `Retain` であるときその `Retain` は「`n` より後」に
数える。

**DEF `(P-insert-net)`**。`ρ` の上の各節点 `n` と各**計数下の**別名類 `C` について、
`held_ρ(n, C) ≥ Fut_ρ(n, C) - Ret#_ρ(n, C)`。計数下の類に限るのは、D34 が `held` を計数下の類にしか
定めないからである。

**DEF 開始事象**。別名類 `C` の**開始事象**とは、
D34 の表で `held_ρ(・, C)` に開始値 1 を与える 3 行 -- `C` の終端が D10 の生成で
作られる行、`C` の終端が所有するパラメータ・capture の leaf である行、`C` の終端が借用する
パラメータ・capture の leaf である行 -- のいずれかに当たる `ρ` の上の事象をいう。

### 5.1 `L4` (`held` は開始と増減で決まる) <!--#94fad8e-->

**言明**。`ρ` の上で開始事象を 1 つだけ持つ各別名類 `C` と、`held_ρ(・, C)` が定まる `ρ` の上の各節点
`n` (`DEF 量化の範囲`) について、
`held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)` である。ここで `R_ρ(n, C)` は `n` より前に実行された各
`Retain(v, π)` 節点について、`(v, λ) ∈ C` であって `λ` を `π` の下に持つような `λ` の個数を数えた総和
であり (D34 の第 4 行の数え上げである)、`D_ρ(n, C)` は `n` より前に実行された各
`Release(v, π)` 節点の同じ数え上げ (D34 の第 5 行) と、`C` のスロットの D9 の消費の個数の和である。

**証明**

<1>1. `held_ρ(·, C)` を動かす事象は、D34 の表の 6 行で尽きる。
  BY <ref id=9d5d254/>
  その定義は `held_ρ(·, C)` を表で定めており、表に無い事象はこの量を動かさない。

<1>2. 表の最初の 3 行は `held_ρ(·, C)` に開始値 1 を与え、仮定よりそのうち 1 つだけが `ρ` の上で
      起きる。
  BY <1>1, DEF 開始事象, 言明の仮定 (`C` は `ρ` の上で開始事象を 1 つだけ持つ)
  `DEF 開始事象` はこの 3 行に当たる事象を開始事象と呼ぶ。

<1>3. 表の残る 3 行は **leaf ごと**に数える。第 4 行は、`Retain(v, π)` 節点であって `(v, λ) ∈ C` である
      `λ` を `π` の下に持つものについて、**その `λ` 1 つにつき `+1`** である。第 5 行は同じ形の
      `Release(v, π)` について、その `λ` 1 つにつき `-1` である。第 6 行は `(w, μ) ∈ C` のスロットの
      D9 の消費 1 つにつき `-1` である。
  BY <1>1, <ref id=9d5d254/>
  D34 の表の第 4 行と第 5 行は「その `λ` 1 つにつき」と書く。1 つの `Retain(v, π)` 節点の下に `C` の
  スロットが 2 つ在れば、その節点は `held_ρ(・, C)` を 2 上げる。

<1>4. QED
  BY <1>1, <1>2, <1>3
  `R_ρ(n, C)` の定義 (言明) は <1>3 の第 4 行の leaf ごとの数え上げを `n` より前について総和した
  ものであり、`D_ρ(n, C)` の定義は第 5 行の同じ数え上げと第 6 行の個数の和である。<1>2 の開始値 1 に
  <1>3 の増減を足すと `held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)` である。

### 5.2 `L5` (`C1` は `(P-insert-net)` を満たす) <!--#1aab97c-->

**言明**。`DEF C1 の本体` の `main` の実行路は、変位 0 を選ぶ `ρ_0` と変位 1 を選ぶ `ρ_1` の
2 本である。

- **(a)** `ρ_0` の上の計数下の別名類は `C_1 = {(p, []), (m, [])}` と `C_2 = {(q, [])}` の 2 つ、
  `ρ_1` の上のそれは `C_3 = {(q, []), (m, [])}` と `C_4 = {(p, [])}` の 2 つである。この 4 つは
  どれも、自分の実行路の上に開始事象 (`DEF 開始事象`) をちょうど 1 つ持つ。
- **(b)** どちらの実行路についても、その上の計数下の別名類のいずれについても、`(P-insert-net)` の
  不等式は、その類の開始の時点以後のすべての節点で成り立つ。

**証明**

<1>0. `C1` の `main` の本体は `Match` をちょうど 1 つ持ち、そのアームは変位 `Some(0)` (payload `y0`、
      本体 `Ret(p)`) と変位 `Some(1)` (payload `y1`、本体 `Ret(q)`) の 2 つである。D3 の分岐の規則は
      `Let(x, Match(v, arms), k)` の節点にだけ当たるので、`main` の実行路は、変位 0 のアームを選ぶ
      `ρ_0` と変位 1 のアームを選ぶ `ρ_1` のちょうど 2 本である。
  BY DEF `C1` の本体, <ref id=ca36627/>

<1>0a. `obj(p, [])` と `obj(q, [])` は D26 の意味で計数下である。
  BY <ref id=e11772a/>, <ref id=88a06de/>, DEF 例の型と op, DEF `C1` の本体
  `p` も `q` も `Llvm(alloc, [])` に束縛される。A3 の表の「単一の `Fresh`」の行は、生成コードが
  結果のその leaf に「新しく割り当てたオブジェクトへの新しい参照」を置くと述べる。A3 がこの行を
  字義どおりに読ませないのは「実行時に参照カウントで分岐する op」-- その一意の腕がオペランドの
  オブジェクトをそのまま返す op -- についてであり、`DEF 例の型と op` より `alloc` はオペランドを
  持たないのでその形ではない。よってこの op についてはこの行を字義どおりに読めて、`p` と `q` の
  leaf `[]` には新しく割り当てたオブジェクトへの参照が置かれる。D26 より割り当てられたオブジェクトは
  計数下である。

<1>1. `ρ_0` の上のスロットは `(p, [])`、`(q, [])`、`(m, [])` の 3 つであり、`(m, [])` の ρ_0 終端は
      `(p, [])` である。よって `ρ_0` の上の計数下の別名類は `C_1 = {(p, []), (m, [])}` と
      `C_2 = {(q, [])}` の 2 つである。
  BY <1>0, <1>0a, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=d59f90b/>, <ref id=9c7c27a/>, <ref id=c232680/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=83d98e9/>, DEF `C1` の本体, DEF 例の型と op,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     CODE src/ast/types.rs: TypeNode::is_box
  `main` が名指す変数は `p`、`q`、`c`、`m`、`u`、`w` と、アームの payload `y0`・`y1` と、`App` の
  callee `f` である。
  **`m`・`u`・`w` の型は A12 が与える** -- `DEF C1 の本体` は `Match` の束縛変数と `App` の束縛変数に
  型を書かない。A12 の「アームの結果と `Match` の束縛変数の型」の節より `ty(m)` は 2 つのアームの結果
  `p`・`q` の型に等しく `Arr` である。A12 の「`Let(x, App(callee, args), k)` の `ty(x)` は呼び出し先の
  返り値の型である」の節より `ty(u)` と `ty(w)` は `f` の返り値の型 `I` である。
`ty(p) = ty(q) = ty(m) = Arr` は boxed なので `is_fully_unboxed` は偽であり
  (`if self.is_box(type_env) { return false; }` で始まる)、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので
  `is_closure` も偽である。よって D4 の判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(c) = Bl` は 2 つの変位がどちらも payload を持たない
  unbox union なので boxed leaf を持たず、payload `y0`・`y1` も同じである。`ty(u) = ty(w) = I` は
  `is_fully_unboxed` が真なので D4 の第 1 規則より leaf を持たない。callee は束縛を持たない名前なので、
  その boxed leaf との対は D6 の記号の位置であってスロットではなく、そこが指すのは funptr かグローバル
  状態のオブジェクトであって D26 より計数下ではない。よってスロットは `(p, [])`、`(q, [])`、`(m, [])`
  の 3 つである。
  `p` と `q` はどちらも `Llvm(alloc, [])` に束縛されるので、D33 の第 2 種 (`Binding::Llvm` であって
  `λ` の宣言が単一の `Fresh` または単一の `Unknown` である位置) に当たり、それぞれ自分自身が ρ_0 終端
  である。`m` は `Match` の束縛変数であり、D20 の別名の辺 (アーム本体の `Ret(x)` の `x` から `Match` の
  束縛変数へ) を持つので D33 の止まる位置に当たらず、歩みを続ける。D20 はこの辺が在るのに
  「路がそのアームを選ぶことも要る」と述べ、D21 は活性化が辿る実行路をその活性化が訪れた節点の並びと
  定めるので、`ρ_0` を辿る活性化は変位 0 のアームへ入っており、この辺は `Ret(p)` の `p` から来る。
  `(p, [])` は上で ρ_0 終端なので D33 より `m` の ρ_0 歩みはそこで止まり、`(m, [])` の ρ_0 終端は
  `(p, [])` である。D33 より別名類は ρ-終端が等しいという関係でスロットを分けた同値類なので、
  `(p, [])` と `(m, [])` は同じ類に属し、`(q, [])` はその歩みの届かない別の類である。
  D33 より 1 つの別名類のすべてのスロットは同じオブジェクトを指すので、`obj(C_1) = obj(p, [])`、
  `obj(C_2) = obj(q, [])` であり、<1>0a よりどちらも計数下である。記号の位置を終端とする類は
  D26 より計数下でないので、計数下の類はこの 2 つで尽きる。

<1>2. `C_1` の上の事象は、`ρ_0` の順に、生成 (`Llvm(alloc, [])`)、`Retain(m, [])`、消費
      (`App(f, [p])` の引数)、`Release(m, [])` の 4 つである。`C_1` の開始の時点は
      `Let(p, Llvm(alloc, []), ·)` の実行の直後であり、`ρ_0` の上でそれ以後の節点は
      `Let(q, Llvm(alloc, []), ·)` から `Ret(u)` までの 10 個である。
  BY <1>1, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>, DEF `C1` の本体, DEF 量化の範囲, DEF 開始事象
  `Let(m, Match(c, ...), ·)` が変位 0 のアームを選んで辿る `Ret(p)` は D9 の移動であり、移動は
  `held` を変えない (D34 の表に移動の行は無い)。`Retain(m, [])` と `Release(m, [])` は
  `(m, []) ∈ C_1` を名指す。`App(f, [p])` は `(p, []) ∈ C_1` を消費し (D9、`f` は `b` を所有する)、
  `App(f, [q])` が消費するのは `(q, [])` であり <1>1 より `C_1` のスロットではない。`Eval(m, ·)` は
  D9 の消費でも移動でもない。終端の `Ret(u)` は `ty(u) = I` に boxed leaf が無いので何も消費しない。
  `C_1` の ρ-終端の変数は `p` であり、`p` が値を得るのは `Let(p, Llvm(alloc, []), ·)` を実行する段で
  ある (D6)。D34 より `held_ρ0(τ, C_1)` はその時点以後の `τ` についてだけ定まる。
  **`L4` の前提はここで満たされる** -- 上の 4 つが `ρ_0` の上で `C_1` について起きる事象のすべてで
  あり、そのうち `DEF 開始事象` に当たるのは生成の 1 つだけなので、`C_1` の開始事象は `ρ_0` の上に
  ちょうど 1 つである。

<1>3. 残る 10 個の節点の入口で `C_1` について `(held, Fut, Ret#)` は次のとおりである。
  BY <1>2, <ref id=94fad8e/>, DEF `Fut`, DEF `Ret#`

  | 入口 | `held` | `Fut` | `Ret#` |
  |---|---|---|---|
  | `Let(q, Llvm(alloc, []), ·)` から `Retain(m, [], s, ·)` まで (5 個) | 1 | 2 | 1 |
  | `Let(u, App(f, [p]), ·)` | 2 | 2 | 0 |
  | `Let(w, App(f, [q]), ·)` から `Release(m, [], s, ·)` まで (3 個) | 1 | 1 | 0 |
  | `Ret(u)` | 0 | 0 | 0 |

  `held` は `L4` の等式 `held = 1 + R - D` である。<1>2 の 4 つの事象のうち生成は開始値 1 を与え、
  以後の各行の `R` は `Retain(m, [])` が実行済みなら 1・そうでなければ 0、`D` は消費と `Release`
  のうち実行済みの個数である。よって `held` は 1 (生成の後、`Let(c, Llvm(mkbl, []), ·)`・
  `Let(m, Match(c, ...), ·)`・アーム本体の `Ret(p)`・`Retain(m, [])` の入口を含む)、2 (`Retain` の後)、
  1 (`App(f, [p])` の消費の後、`Let(w, App(f, [q]), ·)`・`Eval(m, ·)`・`Release(m, [])` の入口を
  含む)、0 (`Release` の後) と動く。
  `Fut` は残る消費と `Release` の個数である。`DEF Fut` より `n` 自身の実行が行う事象も「`n` より後」に
  数えるので、生成から `Retain(m, [])` までの各入口では消費と `Release` の 2 つ、`App(f, [p])` の
  消費の後から `Release(m, [])` までの各入口では `Release` の 1 つ、`Ret(u)` の入口では 0 である。
  `Ret#` は残る `Retain` の個数であり、`DEF Ret#` も `n` 自身が `Retain` であるときそれを数えるので、
  `Retain(m, [])` の入口までが 1、それ以後が 0 である。

<1>4. `C_2` の上の事象は、`ρ_0` の順に、生成 (`Llvm(alloc, [])`)、消費 (`App(f, [q])` の引数) の
      2 つであり、`Retain(m, [])` と `Release(m, [])` はどちらも `(q, [])` を名指さない。`C_2` の
      開始の時点は `Let(q, Llvm(alloc, []), ·)` の実行の直後であり、`ρ_0` の上でそれ以後の節点は
      `Let(c, Llvm(mkbl, []), ·)` から `Ret(u)` までの 9 個である。残るその 9 個の節点の入口で `C_2`
      について `held = Fut` であり `Ret# = 0` であるので `held ≥ Fut - Ret#` が成り立つ --
      `Let(c, Llvm(mkbl, []), ·)` から `Let(w, App(f, [q]), ·)` までの 6 個では `held = Fut = 1`、
      `Eval(m, ·)` から `Ret(u)` までの 3 個では `held = Fut = 0` である。
  BY <1>1, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>, <ref id=94fad8e/>, DEF `Fut`, DEF `Ret#`, DEF `C1` の本体, DEF 量化の範囲, DEF 開始事象
  D10 の生成の表の `Llvm` の行が `(q, [])` の参照を作り、D9 の `App` の行が `App(f, [q])` でそれを
  消費する。<1>1 より `Retain(m, [])` と `Release(m, [])` が名指す `(m, [])` は `C_2` のスロットでは
  ない。`C_2` の ρ-終端の変数は `q` であり、`q` が値を得るのは `Let(q, Llvm(alloc, []), ·)` を
  実行する段である (D6)。D34 より `held_ρ0(τ, C_2)` はその時点以後の `τ` についてだけ定まる。
  **`L4` の前提はここで満たされる** -- 上の 2 つが `ρ_0` の上で `C_2` について起きる事象のすべてで
  あり、そのうち `DEF 開始事象` に当たるのは生成の 1 つだけである。
  `R = 0` なので `L4` の等式より `held` は生成の後 1、消費の後 0 であり、`Fut` は残る消費の個数
  (消費の節点自身の入口を含めて 1、それ以後 0)、`Ret#` は `C_2` のスロットを名指す `Retain` が無いので
  どの入口でも 0 である。

<1>5. `ρ_1` の上のスロットは `(p, [])`、`(q, [])`、`(m, [])` の 3 つであり、`(m, [])` の ρ_1 終端は
      `(q, [])` である。よって `ρ_1` の上の計数下の別名類は `C_3 = {(q, []), (m, [])}` と
      `C_4 = {(p, [])}` の 2 つである。
  BY <1>0, <1>0a, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=d59f90b/>, <ref id=9c7c27a/>, <ref id=c232680/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=83d98e9/>, DEF `C1` の本体, DEF 例の型と op,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     CODE src/ast/types.rs: TypeNode::is_box
  `main` が名指す変数は `p`、`q`、`c`、`m`、`u`、`w` と、アームの payload `y0`・`y1` と、`App` の
  callee `f` である。
  **`m`・`u`・`w` の型は A12 が与える** -- `DEF C1 の本体` は `Match` の束縛変数と `App` の束縛変数に
  型を書かない。A12 の「アームの結果と `Match` の束縛変数の型」の節より `ty(m)` は 2 つのアームの結果
  `p`・`q` の型に等しく `Arr` である。A12 の「`Let(x, App(callee, args), k)` の `ty(x)` は呼び出し先の
  返り値の型である」の節より `ty(u)` と `ty(w)` は `f` の返り値の型 `I` である。
`ty(p) = ty(q) = ty(m) = Arr` は boxed なので `is_fully_unboxed` は偽であり
  (`if self.is_box(type_env) { return false; }` で始まる)、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので
  `is_closure` も偽である。よって D4 の判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(c) = Bl` は 2 つの変位がどちらも payload を持たない
  unbox union なので boxed leaf を持たず、payload `y0`・`y1` も同じである。`ty(u) = ty(w) = I` は
  `is_fully_unboxed` が真なので D4 の第 1 規則より leaf を持たない。callee は束縛を持たない名前なので、
  その boxed leaf との対は D6 の記号の位置であってスロットではなく、そこが指すのは funptr かグローバル
  状態のオブジェクトであって D26 より計数下ではない。よってスロットは `(p, [])`、`(q, [])`、`(m, [])`
  の 3 つである。
  `p` と `q` はどちらも `Llvm(alloc, [])` に束縛されるので、D33 の第 2 種 (`Binding::Llvm` であって
  `λ` の宣言が単一の `Fresh` または単一の `Unknown` である位置) に当たり、それぞれ自分自身が ρ_1 終端
  である。`m` は
  `Match` の束縛変数であり、D20 の別名の辺 (アーム本体の `Ret(x)` の `x` から `Match` の束縛変数へ) を
  持つ。`ρ_1` を辿る活性化は D21 より変位 1 のアーム -- payload が `y1`、本体が `Ret(q)` -- へ入って
  いるので、D20 が要る「路がそのアームを選ぶこと」を満たし、この辺は `Ret(q)` の `q` から来る。
  `(q, [])` は上で ρ_1 終端なので、`m` の ρ_1 歩みはそこで止まり、`(m, [])` の ρ_1 終端は `(q, [])`
  である。
  D33 より `(q, [])` と `(m, [])` は同じ類に属し、`(p, [])` はその歩みの届かない別の類である。
  D33 より 1 つの別名類のすべてのスロットは同じオブジェクトを指すので、`obj(C_3) = obj(q, [])`、
  `obj(C_4) = obj(p, [])` であり、<1>0a よりどちらも計数下である。記号の位置を終端とする類は
  D26 より計数下でないので、計数下の類はこの 2 つで尽きる。

<1>6. `C_3` の上の事象は、`ρ_1` の順に、生成 (`Llvm(alloc, [])`)、`Retain(m, [])`、消費
      (`App(f, [q])` の引数)、`Release(m, [])` の 4 つである。`C_3` の開始の時点は
      `Let(q, Llvm(alloc, []), ·)` の実行の直後であり、`ρ_1` の上でそれ以後の節点は
      `Let(c, Llvm(mkbl, []), ·)` から `Ret(u)` までの 9 個である。
  BY <1>5, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>, DEF `C1` の本体, DEF 量化の範囲, DEF 開始事象
  `Let(m, Match(c, ...), ·)` が変位 1 のアームを選んで辿る `Ret(q)` は D9 の移動であり、移動は
  `held` を変えない。`Retain(m, [])` と `Release(m, [])` は `(m, []) ∈ C_3` を名指す。
  `App(f, [q])` は `(q, []) ∈ C_3` を消費し、`App(f, [p])` が消費するのは `(p, [])` であり <1>5 より
  `C_3` のスロットではない。`Eval(m, ·)` は D9 の消費でも移動でもない。終端の `Ret(u)` は何も消費
  しない。`C_3` の ρ-終端の変数は `q` であり、`q` が値を得るのは `Let(q, Llvm(alloc, []), ·)` を
  実行する段である (D6)。D34 より `held_ρ1(τ, C_3)` はその時点以後の `τ` についてだけ定まる。
  **`L4` の前提はここで満たされる** -- 上の 4 つが `ρ_1` の上で `C_3` について起きる事象のすべてで
  あり、そのうち `DEF 開始事象` に当たるのは生成の 1 つだけである。

<1>7. 残る 9 個の節点の入口で `C_3` について `(held, Fut, Ret#)` は次のとおりである。
  BY <1>6, <ref id=94fad8e/>, DEF `Fut`, DEF `Ret#`

  | 入口 | `held` | `Fut` | `Ret#` |
  |---|---|---|---|
  | `Let(c, Llvm(mkbl, []), ·)` から `Retain(m, [], s, ·)` まで (4 個) | 1 | 2 | 1 |
  | `Let(u, App(f, [p]), ·)` から `Let(w, App(f, [q]), ·)` まで (2 個) | 2 | 2 | 0 |
  | `Eval(m, ·)` から `Release(m, [], s, ·)` まで (2 個) | 1 | 1 | 0 |
  | `Ret(u)` | 0 | 0 | 0 |

  `held` は `L4` の等式である。<1>6 の 4 つの事象のうち生成は開始値 1、`Retain(m, [])` が実行済みなら
  `R` に 1、`App(f, [q])` の消費と `Release(m, [])` のうち実行済みの個数が `D` である。
  `App(f, [p])` は `C_3` を動かさないので、`Retain(m, [])` の後から `App(f, [q])` の消費までの間、
  `held` は 2 のまま `Let(u, App(f, [p]), ·)` と `Let(w, App(f, [q]), ·)` の両方の入口を覆う。
  `Fut` は残る消費と `Release` の個数であり、`Let(w, App(f, [q]), ·)` の入口ではその節点自身の消費を
  数えるので、`Let(u, App(f, [p]), ·)` の入口と同じ 2 のままである。`Ret#` は残る `Retain` の個数で
  あり、`Retain(m, [])` の入口までが 1、それ以後が 0 である。

<1>8. `C_4` の上の事象は、`ρ_1` の順に、生成 (`Llvm(alloc, [])`)、消費 (`App(f, [p])` の引数) の
      2 つであり、`Retain(m, [])` と `Release(m, [])` はどちらも `(p, [])` を名指さない。`C_4` の
      開始の時点は `Let(p, Llvm(alloc, []), ·)` の実行の直後であり、`ρ_1` の上でそれ以後の節点は
      `Let(q, Llvm(alloc, []), ·)` から `Ret(u)` までの 10 個である。残るその 10 個の節点の入口で
      `C_4` について `held = Fut` であり `Ret# = 0` であるので `held ≥ Fut - Ret#` が成り立つ --
      `Let(q, Llvm(alloc, []), ·)` から `Let(u, App(f, [p]), ·)` までの 6 個では `held = Fut = 1`、
      `Let(w, App(f, [q]), ·)` から `Ret(u)` までの 4 個では `held = Fut = 0` である。
  BY <1>5, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>, <ref id=94fad8e/>, DEF `Fut`, DEF `Ret#`, DEF `C1` の本体, DEF 量化の範囲, DEF 開始事象
  D10 の生成の表の `Llvm` の行が `(p, [])` の参照を作り、D9 の `App` の行が `App(f, [p])` でそれを
  消費する。<1>5 より `Retain(m, [])` と `Release(m, [])` が名指す `(m, [])` は `C_4` のスロットでは
  ない。`C_4` の ρ-終端の変数は `p` であり、`p` が値を得るのは `Let(p, Llvm(alloc, []), ·)` を
  実行する段である (D6)。D34 より `held_ρ1(τ, C_4)` はその時点以後の `τ` についてだけ定まる。
  **`L4` の前提はここで満たされる** -- 上の 2 つが `ρ_1` の上で `C_4` について起きる事象のすべてで
  あり、そのうち `DEF 開始事象` に当たるのは生成の 1 つだけである。
  `R = 0` なので `L4` の等式より `held` は生成の後 1、消費の後 0 であり、`Fut` は残る消費の個数
  (消費の節点自身の入口を含めて 1、それ以後 0)、`Ret#` は `C_4` のスロットを名指す `Retain` が無いので
  どの入口でも 0 である。

<1>9. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8
  (a) は <1>0 (実行路が 2 本)、<1>1 と <1>5 (類の分割)、<1>2・<1>4・<1>6・<1>8 (開始事象が
  ちょうど 1 つ) である。(b) について、
  <1>0 より `main` の実行路は `ρ_0` と `ρ_1` の 2 本で尽きる。<1>1 より `ρ_0` の上の計数下の別名類は
  `C_1` と `C_2` の 2 つであり、<1>3 の表の各行で `held ≥ Fut - Ret#` である
  (`1 ≥ 1`、`2 ≥ 2`、`1 ≥ 1`、`0 ≥ 0`)。<1>4 より `C_2` についてもすべての節点で `held = Fut` かつ
  `Ret# = 0` なので `held ≥ Fut - Ret#` である。<1>5 より `ρ_1` の上の計数下の別名類は `C_3` と `C_4`
  の 2 つであり、<1>7 の表の各行で `held ≥ Fut - Ret#` である (`1 ≥ 1`、`2 ≥ 2`、`1 ≥ 1`、`0 ≥ 0`)。
  <1>8 より `C_4` についてもすべての節点で `held = Fut` かつ `Ret# = 0` なので
  `held ≥ Fut - Ret#` である。

### 5.3 R2 の結論

**R2**。`(P-insert-net)` を満たしながら A19 (ii) を破る本体が在る。

**証明**

<1>1. `C1` の `main` は `ρ_0` の上で A19 (ii) を破る。
  BY <ref id=eddc8aa/>
  `L12` の言明の第 2 文が、`Let(w, App(f, [q]), ·)` の入口で `held_ρ0(・, C_1) = 1` かつ
  `bumps_ρ0(・, C_1) = 1` であることを与える。`DEF ii-a と ii-b の読み` では `bumps ≥ 1` のとき
  `held ≥ 1 + bumps` が要る。
  **`L12` を引いても循環しない** -- `L12` の証明が引く命題は `L4`・`L5`・`L13a`・`L15` だけであり、
  そのどれも `R2` を引かない。

<1>2. その同じ `C1` の `main` は `(P-insert-net)` を満たす。
  BY <ref id=1aab97c/>
  `(P-insert-net)` は「`ρ` の上の各節点 `n` と各計数下の別名類 `C` について
  `held_ρ(n, C) ≥ Fut_ρ(n, C) - Ret#_ρ(n, C)`」である (5 節の `DEF (P-insert-net)`)。`L5` (b) は
  `C1` の `main` の 2 本の実行路のそれぞれについて、その上の計数下の別名類のすべてでこの不等式が
  各節点で成り立つことを与える。`held` が定まるのは計数下の類だけである (D34)。

<1>3. QED
  BY <1>1, <1>2

**この 2 つが挟んでいるもの。** `(P-insert)` は `insert_rc` の出力について偽であり (R1)、数え落としを
直した `(P-insert-net)` は `C1` を弾かない (R2)。`C1` は `insert_rc` の出力ではない (第 7.4 節の `L10`)
ので、R2 は `insert_rc` の出力について A19 (ii) が破れることを言うのではない。R2 が言うのは、
`(P-insert-net)` から A19 (ii) へ渡る段が、`insert_rc` の出力を `C1` から区別する事実を要ることである。
第 6 節がその段の形を書き、第 7 節がその区別を与える。

## 6. A19 (ii) の台帳形

### 6.1 局所の定義

`ρ` と活性化を固定し、`obj(C)` が計数下である別名類 `C` を固定する。

**DEF `C` の名前**。`C` のスロットの `origin` の `identity` の全体を、**`C` の名前**と呼ぶ。1 つの
別名類のスロットが持つ `identity` は 1 つとは限らない -- 第 11 節の `L21` はその全体 `Ids(C)` が
木をなすことを示す。以下「`C` の名前に付く分」と書くときは、その全体にわたる総和を指す。A19 (ii-b) が
`bumps` の帰属を「時点 `τ` の `pending` の各要素 `p` について D27 が定める `B(p, ρ)` のうち、`C` の
スロットの `origin` の identity に付く分の総和である」と定めるのと同じ読みである。

`ρ` の上の節点 `n` について:

- `R_ρ(n, C)`、`D_ρ(n, C)` は `L4` のもの。
- **DEF `U_ρ`**。`n` より前の `Release` の訪問で `un_bump` が `InBracket` を返し、選ばれた要素の
  `B_ρ` から引かれた量のうち、`C` のスロットの `identity` に付いていた分の総和を `U_ρ(n, C)` と置く
  (`CODE src/rc_ir/borrow.rs: un_bump`, D27)。
- **DEF `X_ρ`**。`n` より前に `ρ` の上の `pending` から取り除かれた要素の、取り除かれた時点の
  `B_ρ` のうち `C` のスロットの `identity` に付いていた分の総和を `X_ρ(n, C)` と置く。要素を
  `pending` から取り除く操作は 3 つである -- `consume_objects`、`merge`、そして `un_bump` が
  `InBracket` の subtract の後に置く `if innermost.outstanding.is_empty() { pending.remove(index); }`
  である (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`, `CODE src/rc_ir/borrow.rs:
  CancelAnalysis::merge`, `CODE src/rc_ir/borrow.rs: un_bump`)。
  **在りかは `前提 pending と outstanding を変える式` が与える。** `pending.remove(` の走査は
  `un_bump` を、`pending.retain(` の走査は `consume_objects` を挙げ、`merge` は返り値を
  `pending_in` から組み立てる (`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)。
  `PendingRetains` は `Vec<PendingRetain>` であり
  (`CODE src/rc_ir/borrow.rs: PendingRetains`)、その要素の型 `PendingRetain` は
  `src/rc_ir/borrow.rs` の非公開の項目なので、`EXT 非公開の項目の可視範囲` よりこの列を持つ値を
  名指す式はそのモジュールとその子孫の中にしかなく、`EXT クレートの項目` よりそのモジュールの項目は
  このファイルに書かれたものだけである。その全文を読むと、この列に要素を足す式は
  `walk_inner` の `RcExpr::Retain` の腕の `pending.push` だけである。

  **`merge` の分は `ρ` が選んだアームの出口の側で読む。** `merge` は返り値の要素を `pending_in` から
  組み立てるが、その `outstanding` は `uniform` の値、すなわちアームの出口の側の値である
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::merge`)。D27 の第 3 箇条はアームへの複製が
  `B(p, ρ)` をそのまま運ぶと定めるので、`ρ` の上で `bumps_ρ` の和に入っているのは `ρ` が選んだアームへ
  渡った複製であり、`merge` が落とす分もその側の `B_ρ` である。`pending_in` の側の値で読むと、
  アームの中で `consume_objects` が外した要素の分が 2 度数えられ、`L7` の恒等式が偽になる。
  `ρ` が選んだアームの出口に現れない要素は、その落ちる分をアームの中の除去としてすでに数えている。

### 6.1a `L5a` (`B` は `outstanding` の一部と一致する) <!--#2340772-->

1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。

**DEF `帳簿の名前`**。名前 `id` (`VarPath`) を、`ρ` の上に `nm(s) = id` であるスロット `s` -- D6 より
その leaf は inhabited (D16) -- が在り、かつその `s` が指すオブジェクトが計数下 (D26) であるとき
**帳簿の名前**と呼ぶ。**ここで `s` は、解析がその鍵で `origin` を呼んだスロットに限る** -- README の
P5 は「**`identity` は解析が呼んだ鍵についてしか定まらないので、この一覧に漏れがあると、その命題は
呼ばれていない鍵の答えを引く形になる。**」と述べるので、呼ばれていない鍵について `nm(s)` を読むことは
できない。

**言明**。`ρ` が至る走査中の各位置と、その位置での `pending` の各要素 `p` について次が成り立つ。
**位置を `ρ` が至るものに限るのは、D27 の `B(p, ρ)` が `ρ` の上の leaf を数えるからである** --
`ρ` が選ばなかったアームの中の走査位置では、そこで数える leaf が `ρ` の上に無い。README の P18b が
同じ量化を「走査中の各位置と、その位置に至る各実行路 `ρ` について」と書くのと同じ限定である。

- **(a)** `id` が帳簿の名前であるとき `B(p, ρ)[id] = p.outstanding[id]` であり、そうでないとき
  `B(p, ρ)[id] = 0` である。
- **(b)** したがって `B(p, ρ)[id] ≥ 0` である。
- **(c)** `p.outstanding` が空であるとき `B(p, ρ)` は空である。
- **(d)** `un_bump` が `InBracket` で `p` を選ぶとき、D27 がその訪問で `B(p, ρ)` から引く名前の
  多重集合 -- その `Release` が `ρ` で実際に処分する参照を、それを持つ leaf の `origin` の `identity` で
  名付けて数えたもの -- は、各名前で `B(p, ρ)` を超えない。すなわち D27 の引き算は打ち切られない。
  D27 が「**引くのも名前の多重集合である** -- 第 1 項が名前で数えているので、参照の多重集合を引くと
  水準が合わない」と述べるのがこの水準である。

**証明**

<1>1. **解析がその鍵で `origin` を呼んだ** `ρ` の上の 2 つの boxed leaf の `identity` が等しいとき、
      2 つは同時に inhabited (D16) であり、inhabited であるならば同じオブジェクトを指す。したがって
      帳簿の名前 `id` について、解析が鍵を呼んだ leaf のうち `identity` が `id` であるものはすべて
      inhabited であって計数下のオブジェクトを指し、帳簿の名前でない `id` について、解析が鍵を呼んだ
      inhabited かつ計数下の leaf で `identity` が `id` であるものは無い。
  BY <ref id=eabf11d/>, <ref id=0b3e0e1/> (a), <ref id=596a46d/>, <ref id=88a06de/>, DEF `帳簿の名前`
  **P5 (a) の前件をここで述べる。** README の P5 (a) は「**解析がその 2 つの鍵で `origin` を呼び**、
  1 つの実行路の 1 つの位置において `origin` の `identity` が等しい 2 つの leaf のスロットは、同じ
  オブジェクトを指す。」であり、README は「**`identity` を読む命題はこの節を持つ。** P3・P4・P5 (a)・
  P5 (b) は前件として、P5 (c)・P6・P18b は構成から満たされるものとして持つ。」と書く。この段の主語を
  解析が鍵を呼んだ leaf に限るのはそのためであり、帳簿の名前の定義も同じ制限の下にある (言明)。
  p13 の `L9` は identity が inhabited を決めることを与えるので、`identity` が等しい 2 つの leaf は
  同時に inhabited である。inhabited な leaf は D6 のスロットであり、P5 (a) より `identity` が等しい
  2 つのスロットは同じオブジェクトを指す。D26 より 1 つの活性化の間、そのオブジェクトが計数下で
  あるかどうかは変わらない。第 2 文の前半はこの 2 つを帳簿の名前の定義に当てたものである。後半は
  定義そのもの -- `identity` が `id` である inhabited かつ計数下の leaf はそれ自身がスロットであり
  計数下のオブジェクトを指すので、解析がその鍵を呼んでいれば `id` は帳簿の名前である。

<1>2. `Retain(v, π)` の訪問が `pending` に押し込む要素について、押し込んだ直後に (a) が成り立つ。
  BY <1>1, <ref id=cbc4a1c/>, <ref id=8093b68/>, <ref id=66c9670/>, <ref id=88a06de/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/ownership.rs: acted_references
  `walk_inner` の `RcExpr::Retain` の腕は `outstanding` に `self.acted_references(v, path)` を置く。
  **その呼び先は `CancelAnalysis` のメソッドであり、それは自分の `vars` と `type_env` を添えて
  `ownership::acted_references` を呼び、その返り値をそのまま返す** -- 間に挟まるのは
  `!references.is_empty()` の表明だけである。D15 より
  その値は `π` の下の**すべての** boxed leaf を `origin` の `identity` で数えた多重集合である。D27 より
  `B(p, ρ)` は `π` の下の inhabited かつ計数下の各 leaf を同じ鍵で数えたものである。
  **<1>1 の主語の制限はここで満たされる** -- `acted_references(v, π)` はその各 leaf を鍵として
  `origin` を呼ぶ呼び出しそのものである (D15)。<1>1 より、`id` が
  帳簿の名前であれば `π` の下の `identity` が `id` である leaf はすべて inhabited かつ計数下なので
  2 つの数は等しく、そうでなければ `B(p, ρ)[id] = 0` である。

<1>3. `p` が `pending` に在る間、`B(p, ρ)` と `p.outstanding` を動かすのは `un_bump` が `InBracket` で
      `p` を選ぶ `Release` の訪問だけであり、その訪問は `p.outstanding` から
      `R := acted_references(v, π)` を引き、`B(p, ρ)` から名前の多重集合 `A` -- その `Release` が `ρ` で
      実際に処分する参照を、それを持つ leaf の `origin` の `identity` で名付けて数えたもの -- を引く。
      `R` も `A` も `VarPath` を鍵とする多重集合である。
  BY <ref id=8093b68/>, 前提 `pending` と `outstanding` を変える式,
     EXT クレートの項目, EXT ビルドの対象, EXT 非公開の項目の可視範囲,
     CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge, CODE src/rc_ir/borrow.rs: PendingRetains,
     CODE src/rc_ir/ownership.rs: References
  **在りかは可視性で決める。** `PendingRetain` は `src/rc_ir/borrow.rs` の非公開の項目であり、その
  `outstanding` の欄も非公開なので、`EXT 非公開の項目の可視範囲` よりその欄を名指す式はそのモジュールと
  その子孫の中にしかなく、`EXT クレートの項目` よりそのモジュールの項目はこのファイルに書かれたものだけ
  である。`EXT ビルドの対象` より走査の範囲は `src/` の全ファイルであり、その全文を読むと、この欄を
  名指す式のうち既に `pending` に在る要素の欄を**書き換える**ものは
  `un_bump` の `innermost.outstanding.subtract(un_bumped)` の 1 つだけである -- 残りは
  `PendingRetain` の宣言、`shares_an_object`・`covers`・`is_empty`・`names` の読み、`merge` が
  `uniform` を作るときの読みと、`walk_inner` の `Retain` の腕・`merge` の返り値が新しい要素を
  組み立てるときの初期値である。**`References` の値を変えるメソッドは `subtract` だけである** --
  `Map<VarPath, usize>` の欄は非公開なので、`EXT 非公開の項目の可視範囲` よりそれを書き換える式は
  `References` を定めるモジュールの中にしかなく、`前提 pending と outstanding を変える式` の
  `.subtract(` の走査が挙げる項目は `un_bump` だけである。
  `un_bumped` は `Release` の腕が渡す `self.acted_references(v, path)` -- <1>2 より
  `ownership::acted_references` の返り値そのもの -- である。D27 の 2 行目が
  `B(p, ρ)` の側を定める。残る 2 つの行では両者が揃って運ばれる -- アームへの複製は `pending` を
  `clone` するので `outstanding` をそのまま写し、D27 より `B(p, ρ)` も運ばれる。`merge` が返り値に
  据える要素の `outstanding` は `uniform` の値であり、`uniform` に入るのはすべてのアームの出口に
  同じ `outstanding` で現れる要素だけなので、それは `ρ` が選んだアームの出口での値に等しく、D27 より
  `B(p, ρ)` もその側から運ばれる。

<1>4. `id` が帳簿の名前であるとき `A[id] = R[id]` であり、そうでないとき `A[id] = 0` である。
  BY <1>1, <ref id=f06144e/>, <ref id=cbc4a1c/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=8093b68/>, <ref id=4f4714c/>, <ref id=948f840/>
  D10 の `Release` の行より `Release(v, π)` が処分するのは `π` の下の inhabited な各 leaf の参照で
  あり、D26 よりそのうち計数下のオブジェクトを指すものだけが D8 の意味の参照を持つ。P6 がこの
  数え上げと実行時に処分される参照の多重集合の一致を与える。`A` はそれを `origin` の `identity` で
  名付けたものであり (D27 と P18b の言明が `B` をその鍵で数えるのと同じ鍵である)、`R` は D15 より
  同じ `π` の下の**すべての** boxed leaf を同じ鍵で数えたものである。**<1>1 の主語の制限はここでも
  満たされる** -- その `Release` の訪問が呼ぶ `acted_references(v, π)` が、`π` の下の各 boxed leaf を
  鍵として `origin` を呼ぶ (D15)。<1>1 より、帳簿の名前については
  両者が数える leaf が一致し、そうでない名前については `A` が数える leaf が無い。

<1>5. (a)。
  BY <1>2, <1>3, <1>4
  `p` が `pending` に在る間の事象の個数についての帰納。基底は <1>2、段は <1>3 と <1>4 -- 帳簿の名前で
  は両辺から同じ `R[id] = A[id]` が引かれ、そうでない名前では `B` の側が 0 のまま動かない。

<1>6. (b) と (c)。
  BY <1>5, CODE src/rc_ir/ownership.rs: References
  `outstanding` は `References`、すなわち `VarPath` から個数への写像であり、その値は 0 以上である。
  (a) より `B(p, ρ)[id]` はその値か 0 なので 0 以上である。`p.outstanding` が空 -- どの名前の個数も
  0 -- であれば、(a) より `B(p, ρ)` のどの名前の個数も 0 である。

<1>7. (d)。
  BY <1>4, <1>5, <ref id=cbc4a1c/>, CODE src/rc_ir/borrow.rs: un_bump,
     CODE src/rc_ir/ownership.rs: References
  `un_bump` が `InBracket` を返すのは `innermost.outstanding.covers(un_bumped)` が真のときであり、
  D15 より `covers(R)` は各オブジェクトについて自分の個数が `R` 以上かを答えるので、そのとき
  各名前で `p.outstanding[id] ≥ R[id]` である。帳簿の名前では <1>5 より
  `B(p, ρ)[id] = p.outstanding[id] ≥ R[id] = A[id]` (<1>4) であり、そうでない名前では
  `A[id] = 0 = B(p, ρ)[id]` である。

<1>8. QED
  BY <1>5, <1>6, <1>7

### 6.1b `L5b` (名前は別名類を決める) <!--#4910eaa-->

**言明**。1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。`C` を `obj(C)` が計数下
(D26) である別名類、`s_0 = (u, σ)` を `C` のスロットとする。`ρ` の上のスロット `(v, λ)` であって
`obj(v, λ)` が計数下であり `origin(v, λ).identity() = origin(u, σ).identity()` であるものは、`C` の
スロットである。

**この形で足りる。** この命題を読む段が主語にする leaf は、どれも D27 が数える leaf -- inhabited (D16)
かつ計数下 (D26) -- である。D6 より inhabited な leaf は `ρ` の上のスロットであり、それが指す
オブジェクトが計数下であることが第 2 の仮定である。

**証明**

<1>1. `o := origin(u, σ).identity()` は `ρ` の上のスロットであり `C_ρ(s_0) = C` かつ
      `C_ρ(o) = C` である。
  BY <ref id=2171d13/>, <ref id=80d7700/>, <ref id=30d6238/>, 言明の仮定
  `s_0` は `C` のスロットなので、D33 より `C_ρ(s_0) = C` であり、p13 の `L12a` より
  `obj(s_0) = obj(C)` である。言明の仮定よりそれは計数下 (D26) である。p13 の `L14` の後半を
  `s_0` に当てると、`o` は `ρ` の上のスロットであって `C_ρ(o) = C_ρ(s_0) = C` である。

<1>2. `o = origin(v, λ).identity()` であり、`o` は `ρ` の上のスロットであって
      `C_ρ(o) = C_ρ(v, λ)` である。
  BY <ref id=2171d13/>, 言明の仮定
  言明の仮定より `(v, λ)` は `ρ` の上のスロットであって `obj(v, λ)` は計数下なので、p13 の `L14` の
  後半が `(v, λ)` に当たる。2 つの `identity` が等しいことも言明の仮定である。

<1>3. QED
  BY <1>1, <1>2, <ref id=30d6238/>
  D33 より別名類は ρ-終端が等しいという関係でスロットを分けた同値類であり、ρ-終端はスロットごとに
  1 つに決まるので、1 つのスロットはちょうど 1 つの類に属する。よって `C_ρ(o)` は `o` がスロットで
  あることだけで定まる。<1>1 と <1>2 よりその 1 つの類は `C` にも `C_ρ(v, λ)` にも等しいので
  `C_ρ(v, λ) = C` であり、`(v, λ)` は `C` のスロットである。

**この命題が読むのは p13 の `L14` の言明だけである。** 第 1 節が輸入したその言明は `ρ` の上のスロットに
ついて述べるだけで、その本体が `borrow_ify` の出力であることも、その本体について A19 が成り立つことも
求めない。よってこの命題は `insert_rc` の出力と `split_rc_units` の出力にも当たり、A19 を示す段が
これを引いても循環しない。
「名前は別名類を決める」という性質を `p13-disposals-and-pending.md` の `L14` の後半に帰したうえで、
その性質について「**前提として置く者は居ない。**」と述べるのは `report.md` の第 7 節である。

### 6.2 `L6` (bump の残高) <!--#dfb8eac-->

**言明**。次の前提の下で、`bumps_ρ(n, C) = R_ρ(n, C) - U_ρ(n, C) - X_ρ(n, C)` である。

- **(I)** `C` のスロットは `ρ` の上で inhabited であり、`obj(C)` は計数下である。

**(I) の前半は D6 から出る。** D6 のスロットの leaf は inhabited なので、この前提が本体について
言うのは後半 -- `obj(C)` が計数下であること -- だけであり、6.1 節はその条件を満たす `C` を固定して
いる。

**証明**

<1>1. `B_ρ` を動かす事象は、D27 の 3 つの箇条が挙げるもので尽きる。
  BY <ref id=8093b68/>
  D27 は `B(p, ρ)` を 3 つの箇条で定め、最後の箇条が「ほかのどの操作も `B(p, ρ)` を変えない」と
  述べる。

<1>1a. `Retain(v, π)` の訪問が `B_ρ` に足す量のうち `C` の名前 (6.1 節 -- `C` のスロットの
       `identity` の全体) に付く分の総和は、`π` の下の `(v, λ) ∈ C` である `λ` の個数に等しい。
  BY <ref id=4910eaa/>, <ref id=8093b68/>, <ref id=596a46d/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=30d6238/>, 前提 (I), DEF `C` の名前
  D27 の第 1 箇条より、`Retain(v, π)` の訪問が押し込む要素の `B(p, ρ)` は、`π` の下の inhabited (D16)
  かつ計数下 (D26) の各 leaf を `origin` の `identity` で名付けて数えたものである。よって `C` の名前に
  付く分の総和は、`π` の下の inhabited かつ計数下であって `identity` が `C` のいずれかのスロットの
  `identity` に等しい leaf の個数である。D6 より inhabited な leaf は `ρ` の上のスロットなので、`L5b` よりそのような leaf は
  `C` のスロットである。逆に `(v, λ) ∈ C` で `λ` が `π` の下に在るならば、D6 より `λ` は inhabited で
  あり、D33 と前提 (I) より `obj(v, λ) = obj(C)` は計数下であり、その `identity` は `C` のスロットの
  `identity` である。よって 2 つの個数は等しい。

<1>2. `B_ρ` を増やすのは D27 の第 1 箇条 -- `Retain(v, π)` の訪問が要素を `pending` に押し込む段 --
      だけであり、`n` までにそれが `C` の名前に足す量の総和は `R_ρ(n, C)` である。
  BY <1>1, <1>1a, <ref id=8093b68/>, <ref id=94fad8e/>
  D27 の第 2 箇条は引く側であり、第 3 箇条は運ぶか定めないかである。<1>1a より、`Retain(v, π)` が
  `B_ρ` に足す量のうち `C` の名前に付く分は、`π` の下の `(v, λ) ∈ C` である `λ` の個数に等しい。
  `R_ρ(n, C)` (`L4`) はその個数の `n` より前の総和である。

<1>3. `B_ρ` を減らすのは D27 の第 2 箇条 -- `un_bump` が `InBracket` で要素を選ぶ `Release` の訪問 --
      だけであり、`n` までにそれが `C` の名前から引く量の総和は `U_ρ(n, C)` である。
  BY <1>1, <ref id=8093b68/>, DEF `U_ρ`

<1>4. 要素が `pending` を離れるとその `B_ρ` は `bumps` の和から落ち、`n` までに落ちた量のうち `C` の
      名前に付く分の総和は `X_ρ(n, C)` である。
  BY <1>1, <ref id=2340772/>, <ref id=8093b68/>, DEF `X_ρ`,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge
  `bumps_ρ(n, C)` は `pending` に在る要素についての総和なので、`pending` を離れた要素は数えられない。
  6.1 節の `X_ρ(n, C)` は 3 つの取り除く操作すべてについての総和であり、そのうち `un_bump` が
  `outstanding` の空になった要素を外す分は `L5a` (c) より 0 を寄せる。`un_bump` が同じ訪問で引いた分は
  `U_ρ(n, C)` が数えている (<1>3) ので、二重には数えられない。
  `merge` の分については、6.1 節の定義が `ρ` が選んだアームの出口の側の `B_ρ` を数えると定める。`ρ` の上で
  `bumps_ρ` の和に在るのはそのアームへ渡った複製であり (D27 の第 3 箇条)、`merge` はそのうち
  `uniform` に入らないものを落とす。`ρ` が選んだアームの出口に現れない要素は、その落ちる分をアームの
  中の `consume_objects` か `un_bump` の除去としてすでに数えているので、ここでは数えない。

<1>5. 残るもの -- アームへの複製と、`merge` が要素を返り値に据える段 -- は `B_ρ` を変えない。
  BY <1>1, <ref id=8093b68/>
  D27 の第 3 箇条が「アームへの複製と、`merge` が `p` を返り値に据えるときは、`B(p, ρ)` をそのまま
  運ぶ」と述べる。

<1>6. QED
  BY <1>1, <1>1a, <1>2, <1>3, <1>4, <1>5, <ref id=4910eaa/>, <ref id=8093b68/>, <ref id=9f1cf6c/>, DEF `C` の名前
  `bumps_ρ(n, C)` は `pending` の要素の `B_ρ` のうち `C` の名前に付く分の総和である -- A19 (ii-b) が
  `bumps` の帰属を「`bumps_ρ(τ, C)` とは、時点 `τ` の `pending` の各要素 `p` について D27 が定める
  `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和である」と定め、6.1 節の
  読みよりその identity は 1 つとは限らずその全体にわたる総和である。<1>2 から <1>5 より、その値は
  足した分 `R` から引いた分 `U` と落ちた分
  `X` を除いたものである。D27 が `B_ρ` に量を足すのは inhabited かつ計数下の leaf についてであり、
  `L5b` よりそのうち `C` の名前を持つものは `C` のスロットに限るので、`C` の外のスロットが `C` の
  名前へ量を足すことはない。

### 6.3 `L7` (A19 (ii) の台帳形) <!--#86b3f11-->

**言明**。`L6` の前提 (I) と `L4` の前提 (開始事象が 1 つ) の下で、`ρ` の上の節点 `n` について
次が成り立つ。

```
held_ρ(n, C) - (1 + bumps_ρ(n, C)) = U_ρ(n, C) + X_ρ(n, C) - D_ρ(n, C)
```

とくに `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)` と `U_ρ(n, C) + X_ρ(n, C) ≥ D_ρ(n, C)` は同値である。

**証明**

<1>1. `held_ρ(n, C) = 1 + R_ρ(n, C) - D_ρ(n, C)`。
  BY <ref id=94fad8e/>, 言明の前提 (`L4` の前提)

<1>2. `bumps_ρ(n, C) = R_ρ(n, C) - U_ρ(n, C) - X_ρ(n, C)`。
  BY <ref id=dfb8eac/>, 言明の前提 (`L6` の前提 (I))

<1>3. `held_ρ(n, C) - (1 + bumps_ρ(n, C)) = U_ρ(n, C) + X_ρ(n, C) - D_ρ(n, C)`。
  BY <1>1, <1>2
  <1>1 の右辺から <1>2 の右辺に 1 を足したものを引くと
  `(1 + R - D) - (1 + R - U - X) = U + X - D` である。

<1>4. QED
  BY <1>3
  <1>3 より 2 つの式は同じ整数である。よって一方が 0 以上であることと他方が 0 以上であることは
  同値であり、すなわち `held_ρ(n, C) ≥ 1 + bumps_ρ(n, C)` と
  `U_ρ(n, C) + X_ρ(n, C) ≥ D_ρ(n, C)` は同値である。

### 6.4 台帳形が言うもの

`D` は別名類の処分の個数であり、`U + X` は走査がその類の名前について落とす bump の量である。README の
A19 が「すなわち **(ii-b) は「走査の帳簿がその類の処分に遅れない」ことである。**」と書くのがこの形で
ある。

`p13-disposals-and-pending.md` の反例 `C1` は、この形で次のように読める。`Retain(m, [])` の要素の
`outstanding` は `{(m, []): 1}` である。`App(f, [p])` の消費が渡す `acted_on(p, []) = {(p, [])}` は
その鍵ではないので `consume_objects` は要素を落とさない。よって `D` が 1 増えて `U + X` は増えない。

**この不等式を決めるのは名前である。** `U` と `X` は、走査が持つ `outstanding` の鍵 -- `origin` の
`identity` -- と、処分の構文が渡す `acted_on` が名前を共有するかで決まる。`C1` は `(P-insert-net)` を
満たしながら (`L5`) この不等式を破るので、`held` の推移についての言明からこの不等式へ渡る段には、
名前についての事実が要る。第 7 節がその事実を与える。

## 7. `insert_rc` が置く `Retain` の位置

台帳形が名前を要求する以上、`insert_rc` の側で書けるのは「どの `Retain` がどの構文の直前に立つか」で
ある。この節はそれを証明する。

### 7.1 `L8` (`Retain` を作る位置は 4 つである) <!--#664b958-->

**言明**。

- **(0)** `insert_rc` の出力の各本体の各節点は、`src/rc_ir/rc_insert.rs` の 7 つの `RcExprNode` の
  構成式のいずれかが作ったものである。そのうち `RcExpr::Retain` を作るのは `build_retains` の中の
  1 つだけ、`RcExpr::Release` を作るのは `build_releases` の中の 1 つだけである。
- **(a)** `insert_rc` が `RcExpr::Retain` を作る位置は `build_retains` の呼び出し 2 か所だけであり、
  その呼び出し元は次の 4 か所である。

  1. `RcInserter::insert_into_operation_let` の `build_retains(retains_before, node)`。`node` は
     その呼び出しが作った `Let(x, rhs, cont)` である。
  2. `RcInserter::insert_into_expr_inner` の `RcExpr::Ret(x)` の腕の
     `retain_if_live(&x, live_after, ret)`。`ret` はその腕が作った `Ret(x)` である。
  3. `RcInserter::insert_into_destructure` の `retain_if_live(&container, &live_cont, node)`。`node` は
     その呼び出しが作った `Destructure(container, fields, Unknown, cont)` である。
  4. `RcInserter::insert_into_match` の `retain_if_live(&scrut, &live_at_arm_head, node)`。`node` は
     その呼び出しが作った `Let(x, Match(scrut, new_arms), cont)` である。

**証明**

<1>0. `insert_rc` の出力の各本体の各節点は、`src/rc_ir/rc_insert.rs` の 7 つの `RcExprNode` の構成式の
      いずれかが作ったものである。**走査の範囲を 1 つのファイルへ狭めていない** -- この段が数え上げるのは
      「クレートのどこに `RcExprNode` を作る式が在るか」ではなく、「`insert_rc` が出力に置く木の節点が
      どの式から来るか」であり、その式は `insert_rc` の呼び出しの木を辿って読める。
  BY <ref id=3e6b0e0/>, <ref id=d80dde9/>, EXT クレートの項目, CODE src/rc_ir/rc_insert.rs: insert_rc,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
  `insert_rc` が出力に置く本体は、関数については `insert_into_func` が `func.body` に代入する値、
  グローバル初期化子については `insert_rc` が `glob.init` に代入する値である。前者は
  `build_releases(unused, body)`、後者は `insert_into_expr` の返り値の第 1 成分である。A15 より
  `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。
  `insert_into_expr_inner` の 6 つの腕のうち、A25 より骨格 (`DEF 骨格`) は `Retain`/`Release` を
  含まないので `RcExpr::Retain(..) | RcExpr::Release(..)` の腕は通らない。残る 5 つの腕は、
  `RcExpr::Ret(x)` の腕が節点を 1 つ作って `retain_if_live` に渡し、他の 4 つは
  `insert_into_operation_let`・`insert_into_destructure`・`insert_into_eval`・`insert_into_match` へ
  振り分ける。**どの腕も入力の節点をそのまま返さず、`RcExpr` を新たに構成して `RcExprNode` に
  包む。** 継続とアーム本体の節点は、それぞれについての `insert_into_expr` の呼び出しが返す木の節点で
  あるから、入力の骨格の節点数についての帰納で、出力の木の各節点は
  `insert_into_expr_inner` の `Ret` の腕・`insert_into_eval`・`insert_into_operation_let`・
  `insert_into_destructure`・`insert_into_match`・`build_retains`・`build_releases` の 7 つの
  `RcExprNode` の構成式のいずれかが作ったものである。`EXT クレートの項目` より、このモジュールの項目は
  このファイルに書かれたものだけなので、ファイルの全文を読んで得たこの 7 つの一覧は完全である。

<1>1. (0)。したがって `insert_rc` の出力の `RcExpr::Retain` 節点は、すべて `build_retains` の中の
      1 つの式が作ったものである。
  BY <1>0, CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
  <1>0 が第 1 文を与える。その 7 つの構成式のうち、`RcExpr::Retain` を作るのは `build_retains` の中の
  `RcExpr::Retain(v, vec![], RcState::Unknown, c)` の 1 つだけ、`RcExpr::Release` を作るのは
  `build_releases` の中の `RcExpr::Release(v, vec![], RcState::Unknown, c)` の 1 つだけである --
  残る 5 つが作るのは `Ret`・`Eval`・`Let`・`Destructure`・`Let(Match)` である。

<1>2. `build_retains` を呼ぶのは `insert_into_operation_let` と `retain_if_live` の 2 か所である。
  BY EXT クレートの項目, EXT 非公開の項目の可視範囲,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live
  `build_retains` はこのモジュールの非公開の項目なので、`EXT 非公開の項目の可視範囲` よりそれを呼ぶ式は
  このモジュールの中にしかなく、`EXT クレートの項目` よりこのモジュールの項目はこのファイルに書かれた
  ものだけである。よってファイルの全文を読んで得たこの一覧は完全である。

<1>3. `retain_if_live` を呼ぶのは、言明の 2・3・4 に挙げた 3 か所である。
  BY EXT クレートの項目, EXT 非公開の項目の可視範囲,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `insert_into_expr_inner` の `RcExpr::Ret(x)` の腕、`insert_into_destructure`、`insert_into_match` の
  3 か所である。`retain_if_live` は `RcInserter` の非公開のメソッドなので、`EXT 非公開の項目の可視範囲`
  よりそれを呼ぶ式はこのモジュールの中にしかなく、`EXT クレートの項目` よりこのモジュールの項目は
  このファイルに書かれたものだけである。

<1>4. QED
  BY <1>0, <1>1, <1>2, <1>3
  (0) は <1>0 と <1>1 である。(a) について、<1>1 より `Retain` 節点を作る位置は `build_retains` の
  呼び出しに尽き、<1>2 と <1>3 よりその呼び出し元は言明の 4 か所である。

### 7.2 `L9` (`Retain` はその変数を名指す構文の直前に立つ) <!--#19c0e5a-->

**言明**。`insert_rc` の出力の各本体の各 `Retain` 節点 `t` について、`t` は `Retain(v, [],
RcState::Unknown, k)` の形であり、`k` から継続を辿って最初に現れる `RcExpr::Retain` でない節点 `n_t` は
次のいずれかである。いずれの場合も `n_t` は `v` を名指す。

- **(a)** `Let(x, rhs, k')` であって、`rhs_operands(rhs)` が `(v, Ownership::Own)` を含む。
- **(b)** `Ret(v)` であって、その `Ret` は `Match` のアーム本体の終端である。
- **(c)** `Destructure(v, fs, s, k')`。
- **(d)** `Let(x, Match(v, arms), k')`。

**証明**

<1>1. `build_retains(vars, cont)` が作る各 `Retain` 節点は `Retain(v, vec![], RcState::Unknown, ·)` の
      形であり、その継続は同じ呼び出しが作る次の `Retain` 節点か `cont` である。
  BY CODE src/rc_ir/rc_insert.rs: build_retains, EXT `Iterator::fold`, EXT `Iterator::rev`
  `build_retains` は `vars.into_iter().rev().fold(cont, |c, v| … RcExpr::Retain(v, vec![],
  RcState::Unknown, c) …)` である。`EXT Iterator::rev` より走る順は `vars` の逆順であり、
  `EXT Iterator::fold` より各段は直前の段の結果を継続に据えた `Retain` 節点を作るので、`cont` から
  始めて外へ向かって節点が積まれ、`vars` の第 1 元の節点が最も外側に来る。作られる節点の path は
  `vec![]`、`RcState` は `Unknown` である。

<1>2. `insert_rc` は、一度作った節点の継続を書き換えない。
  BY 前提 節点の書き換えの不在, CODE src/rc_ir/ast.rs: RcExprNode,
     CODE src/rc_ir/rc_insert.rs: insert_rc,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: build_retains,
     CODE src/rc_ir/rc_insert.rs: build_releases
  節点の継続は `RcExprNode` の `expr` 欄 (`Arc<RcExpr>`) が持つので、一度作った節点の
  継続を書き換える式は、その節点を可変に借りるか、`Arc` の中身を可変に借りるものである。
  `前提 節点の書き換えの不在` の 3 つの走査より、`&mut RcExprNode` を取る式も `Arc::get_mut` の
  呼び出しも無く、`Arc::make_mut` の呼び出しは `Arc<RcExpr>` を借りない 1 つだけである。
  よって `RcExprNode` の値は作られた後に変わらず、`insert_rc` が走らせるどの式も、返された節点を
  別の構成子の継続 (`cont` または `node`) として渡すか、本体の位置 -- `insert_into_func` の
  `func.body`、`insert_rc` の `glob.init` -- へ書き換えの結果の木そのものを置くだけである
  (`build_releases` と `build_retains` も渡された節点を継続として**包む**)。よって出力の
  `Retain` 節点の継続は、それが作られた時点の継続である。

<1>3. CASE `t` が `L8` (a) の 1 で作られた。
  BY <ref id=664b958/>, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner
  `build_retains(retains_before, node)` の `node` は `Let(x, rhs, cont)` であり、`rhs` は `Match` では
  ない -- `insert_into_expr_inner` は `RcExpr::Let(x, RcRhs::Match(..), cont)` を
  `insert_into_match` へ振り分けるので、`insert_into_operation_let` に来る右辺は `Match` でない。
  `n_t` はこの `Let` である。`retains_before` に入るのは、`rhs_operands(rhs)`
  が `Ownership::Own` を与えるオペランド `v` だけである (同関数のループの `else if` の枝)。これが (a) で
  ある。

<1>4. CASE `t` が `L8` (a) の 2 で作られた。
  <2>1. `n_t` は `Ret(v)` である。
    BY <ref id=664b958/>, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の
       `RcExpr::Ret(x)` の腕
    `retain_if_live(&x, live_after, ret)` に渡る `ret` はこの腕が作った `Ret(x)` であり、`v = x` で
    ある。
  <2>2. 本体の根を書き換える呼び出しの `live_after` は空集合である。
    BY EXT 空を作る `Default`, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: insert_rc
    関数については `insert_into_func` が `self.insert_into_expr(func.body, &Set::default())` を呼び、
    グローバル初期化子については `insert_rc` が `inserter.insert_into_expr(glob.init, &Set::default())` を
    呼ぶ。`EXT 空を作る Default` より `Set::default()` は空の集合である。
  <2>3. `Let`(`Match` でない右辺)、`Destructure`、`Eval`、`Let(x, Match(..), k)` のいずれについても、
        継続を書き換える呼び出しの `live_after` は、その節点を書き換える呼び出しの `live_after` に
        等しい。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    4 つとも `self.insert_into_expr(cont, live_after)` を呼ぶ。
  <2>4. 本体の終端の `Ret` を書き換える呼び出しの `live_after` は空集合である。
    BY <2>2, <2>3, <ref id=b3dfa37/>, <ref id=d80dde9/>
    D2 より `RcExpr` は 6 種であり、`Ret` を除く 5 種はちょうど 1 つの継続を持つので、本体の根から
    継続だけを辿ると終端の `Ret` に着く。A25 より骨格は `Retain` と `Release` を含まないので、その
    鎖の `Ret` でない各節点は `Let`(`Match` でない右辺)・`Let`(`Match` の右辺)・`Destructure`・`Eval` の
    4 種であり、<2>3 がその 4 種を覆う。鎖の長さについての帰納で、鎖の各節点を書き換える呼び出しの
    `live_after` は根のもの、すなわち <2>2 の空集合である。
  <2>5. 本体の終端の `Ret` では `retain_if_live` は節点をそのまま返す。
    BY <2>4, CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live
    `live.contains(&var.name)` は空集合について偽である。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>5, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `retain_if_live` がこの位置で発火する `Ret` は、本体の終端の `Ret` ではない。`Ret` を書き換える
    呼び出しに空でない `live_after` が渡るのは、`insert_into_match` が
    `self.insert_into_expr(arm.body, &live_after_match)` でアーム本体を書き換える枝を経由したときだけで
    あり (<2>2 と <2>3 より、他の枝は根の空集合を運ぶ)、その枝の下で終端になる `Ret` はアーム本体の
    終端の `Ret` である。これが (b) である。

<1>5. CASE `t` が `L8` (a) の 3 で作られた。
  BY <ref id=664b958/>, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure
  `node` は `Destructure(container, fields, RcState::Unknown, cont)` であり、`v = container` である。
  これが (c) である。

<1>6. CASE `t` が `L8` (a) の 4 で作られた。
  BY <ref id=664b958/>, <1>1, <1>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  `node` は `Let(x, RcRhs::Match(scrut, new_arms), cont)` であり、`v = scrut` である。これが (d) で
  ある。

<1>7. QED
  BY <ref id=664b958/>, <1>3, <1>4, <1>5, <1>6
  `L8` (a) が場合を尽くす。

### 7.3 `L9a` (`n_t` に着くと `v` の参照は手を離れる) <!--#d686a93-->

**言明**。**`insert_rc` の出力の各本体について**、`L9` の各場合の `n_t` は、場合 (d) で scrutinee が
boxed であって選ばれたアームが変位アーム (`tag` が `Some`) であるときは `v` の inhabited な各 boxed
leaf の参照を処分する `Release(v, [])` をそのアームの頭に持ち、そうでないときは `v` の inhabited な各
boxed leaf の参照を D9 の意味で消費するか移動させる。

**主語を `insert_rc` の出力に限るのは、下の `<1>1` `<2>3` が `L15` (e) -- 出力のすべての関数の
`borrowed_units` が空であること -- を読むからである。** `L11` は `L9` の構文の形を `cancel` の入力まで
運ぶが、そこには借用する unit を持つ借用版が在るので、この命題はそこへは延びない。
場合 (d) の変位アームの `Release(v, [])` を置くのは `insert_into_match` である。D9 は
「`Eval(v, k)` と `Let(x, Match(v, arms), k)` の `Match` 節点自身は、参照を作らず、移さず、手放さない」
と述べており、この場合の処分は `Match` 節点ではなくその `Release` が行う。

**読む者は README の A19 の脇である** -- 「その構文は D9 の消費か移動を行うか、boxed union の変位
アームの頭に置かれた `Release` がその参照を処分する。」がこの言明である。

**証明**

<1>1. CASE (a)。`rhs_operands` が `Ownership::Own` を与えるオペランドは、`Var` の被移動変数、`App` の
      callee と各引数、`Closure` の各 capture、`Llvm` の `borrows_operand(i)` が偽であるオペランドの
      いずれかである。
  BY CODE src/rc_ir/rc_insert.rs: rhs_operands
  `Var` の腕は唯一のオペランドに `Own` を与え、`App` の腕は callee と全引数に `Own` を与え、`Closure` の
  腕は全 capture に `Own` を与え、`Llvm` の腕は `borrows_operand(i)` が真のものにだけ `Borrow` を与える。
  `Match` の腕はこの関数に来ない。
  <2>1. `Var` の被移動変数の各 leaf は D9 の移動である。
    BY <ref id=9d74736/>
    移動の表の `Let(x, Var(y), k)` の行。
  <2>2. `App` の callee の各 leaf は D9 の消費である。
    BY <ref id=9d74736/>
    消費の表の `App` の行の前半。
  <2>3. `App` の引数の各 leaf は D9 の消費である。
    BY <ref id=9d74736/>, <ref id=dca1c02/>
    消費の表の `App` の行の後半は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」を
    挙げる。`L15` (e) より `insert_rc` の出力のすべての関数のすべてのパラメータ・capture の unit は
    その関数が所有するので、すべての位置が所有される。
  <2>4. `Closure` の capture の各 leaf は D9 の消費である。
    BY <ref id=9d74736/>
    消費の表の `Closure` の行。
  <2>5. `Llvm` の `borrows_operand(i)` が偽であるオペランドの各 leaf は、D9 の消費か移動である。
    BY <ref id=9d74736/>
    消費の表の `Llvm` の行は、`result_prov` が単一の `Arg(i, σ)` として素通しを宣言していない leaf を
    挙げる。素通しを宣言している leaf は移動の表の `Llvm` の行にある。
  <2>6. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5

<1>2. CASE (b)。アーム本体の終端の `Ret(v)` は D9 の移動である。
  BY <ref id=9d74736/>
  移動の表の「`Match` のアーム本体の `Ret(x)`」の行。

<1>3. CASE (c)。`Destructure(v, fs, s, k')` は `v` の各 leaf を消費するか移動させる。
  BY <ref id=9d74736/>
  `v` が boxed なら消費の表の「`Destructure(c, fs)` (`c` が boxed)」の行が全 leaf を挙げる。`v` が unbox なら、
  名前の付いていないフィールドの leaf は消費の表の行に、名前の付いたフィールドの leaf は移動の表の
  「unbox 容器の `Destructure` の名前付きフィールド」の行にある。

<1>4. CASE (d)。`Let(x, Match(v, arms), k')` の選ばれたアームの入口で、`v` の各 leaf の参照は移動するか、
      `insert_into_match` が置いた `Release(v, [])` が処分する。
  BY <ref id=9d74736/>, <ref id=66c9670/>, <ref id=c232680/>, <ref id=f769887/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
     CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  `v` が unbox union のとき、変位アームの payload 束縛と catch-all アームの payload 束縛は移動の表の
  行である。移動の表の行が名指すのは scrutinee の**活性**変位の参照なので、選ばれたアームの `tag` が
  実行時のタグに等しいことが要る -- D21 より活性化は実行時のタグに `tag` が等しいアームを選び、A16 より
  そのようなアームか catch-all アームが在る。D16 より scrutinee の inhabited な leaf はその変位の下に
  あるものだけである。`v` が boxed union のとき、`insert_into_match` はこの節点で
  `release_container = scrut.ty.is_box(self.type_env)` を真にし、
  `release_container && arm.tag.is_some() && self.needs_rc(&scrut)` の枝で各変位アームの先頭に
  `Release(v, [])` を置く (`head.push(scrut.clone())` と `build_releases(head, body)`)。
  第 3 項は真である -- `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` であり、
  `is_fully_unboxed` は `if self.is_box(type_env) { return false; }` で始まるので boxed な `ty(v)` では
  偽である。
  catch-all アームでは payload 束縛が移動の表の行である。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, <ref id=19c0e5a/>, <ref id=9d74736/>
  `L9` の 4 つの場合が尽くす。(a)(b)(c) と、(d) のうち scrutinee が unbox であるか catch-all アームが
  選ばれる場合は <1>1 から <1>4 が消費か移動を与える。(d) の残る場合 -- scrutinee が boxed で変位アームが
  選ばれる場合 -- は <1>4 が `Release(v, [])` による処分を与える。

### 7.4 `L10` (`C1` の `main` は `insert_rc` の出力ではない) <!--#cafef88-->

**言明**。`p13-disposals-and-pending.md` の反例 `C1` の `main` の本体は `insert_rc` の出力ではない。

**証明**

<1>1. `C1` の `main` の `Retain(m, [], s, k)` について、`k` は `Let(u, App(f, [p]), ·)` である。
  BY DEF `C1` の本体

<1>2. `C1` の `main` は `L9` を破る。
  BY <1>1, <ref id=19c0e5a/>, CODE src/rc_ir/rc_insert.rs: rhs_operands
  `n_t = Let(u, App(f, [p]), ·)` は `L9` の (a) の形だが、`rhs_operands(App(f, [p]))` は
  `[(f, Own), (p, Own)]` であって `m` を含まない。(b)(c)(d) の形でもない。

<1>3. QED
  BY <1>2, <ref id=19c0e5a/>
  `L9` は `insert_rc` の出力のすべての `Retain` について成り立つ。

### 7.5 `L11` (`L9` の形は `cancel` の入力まで残る) <!--#53bfb96-->

**言明**。`cancel` の入力の各本体の各 `Retain` 節点 `t = Retain(v, π, s, k)` について、`k` から継続を
辿って最初に現れる `RcExpr::Retain` でない節点 `n_t` は `v` を名指し、`L9` の (a)-(d) のいずれかの形で
ある。

**証明**

<1>1. `split_rc_units` は `Retain(v, π, s, k)` を `units_under(ty(v), π)` の各 unit の `Retain` 節点の
      鎖に置き換え、他の節点は継続を書き換えた上でその場に作り直す。
  BY CODE src/rc_ir/borrow.rs: split_body, CODE src/rc_ir/borrow.rs: split_body_inner,
     CODE src/rc_ir/borrow.rs: split_rc, CODE src/rc_ir/borrow.rs: expr_node,
     CODE src/rc_ir/ast.rs: MatchArm::with_body, <ref id=3e6b0e0/>
  A15 より `split_body` は `split_body_inner` をちょうど 1 回呼ぶ。
  `split_rc` は `fold` で `k` の外へ節点を積む。他の腕は同じ種類の節点を、同じ変数・同じ右辺・
  同じ `RcState` と `split_body` した継続から作り直し、`expr_node` がそれを `RcExprNode` に包むだけで
  ある。`Match` の腕は `arm.with_body(…)` を使い、`with_body` は `MatchArm { body, ..self.clone() }`
  なので `tag`・`payload`・`payload_state` をそのまま持つ。
  よって `Retain` の鎖の後に来る節点の種類・変数・並びは変わらない。

<1>2. `borrow_ify` の `rewrite_rc` は `Retain(v, π, s, k)` を、`v` を名指す `Retain` 節点の鎖
      (長さ 0 のこともある) に置き換え、その後に `k` の写しを置く。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_rc, CODE src/rc_ir/borrow.rs: rc_node
  この関数はまず `k` を `rewrite` で写す。`is_borrow_version` が偽のときは `rc_node` で同じ `v`・
  同じ `path`・同じ `state` の節点を 1 つ作り直してそこで返る -- この枝は `units_under` を呼ばない。
  真のときは `units_under(&v.ty, path, ・)` のうち `owns_unit(v, ・)` が真の unit だけを残し、
  `rev().fold` でその各 unit の `Retain` 節点を `k` の外へ積む。どちらの枝でも作られる節点は
  `v` を名指す `Retain` であり、鎖の後に来る節点は `k` の写しである。

<1>3. `borrow_ify` の `rewrite_inner` は、`Let(x, App(callee, args), k)` について `call_rc` の `before` の
      `Retain` 節点をその `Let` 節点の直前に積み、`after` の `Release` 節点をその `Let` 節点と継続の間に
      置く。他の節点は継続を書き換えた上でその場に作り直す。
  BY CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner, CODE src/rc_ir/borrow.rs: prepend_rc
  `prepend_rc(before, false, app)` は `app` の外へ `Retain` を積み、`prepend_rc(after, true,
  self.rewrite(k))` は継続の外へ `Release` を積む。

<1>3a. `rewrite_inner` の `App` の腕は callee を `route` の結果で差し替える。差し替わりうる名前は
       局所名ではないので、`insert_rc` が `Retain` を置いた変数ではありえない。
  BY <ref id=843e506/>, <ref id=cb35ab1/>, <ref id=664b958/>, CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner,
     CODE src/rc_ir/borrow.rs: borrow_funcref, CODE src/rc_ir/borrow.rs: clone_func,
     CODE src/rc_ir/rename.rs: fresh_rename_function,
     CODE src/rc_ir/rename.rs: assign_fresh_name,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/ast/name.rs: FullName::is_local, CODE src/ast/name.rs: NameSpace::is_local
  P12 より `route` が返す呼び出し先は、元の呼び出し先と同じ関数の版 (元の版そのものか、その
  `borrow_versions` の像) であるか、局所変数を経由する間接呼び出しでは呼び出し先そのものである。
  後者では名前が変わらない。前者で名前が変わるとき、元の名前は `borrow_ify` の入力の `funcs` の鍵で
  あり、A13 の「最上位の記号の名前は局所名ではない」の節よりそれは局所名ではない。返る名前は
  `borrow_funcref` が元の名前の最後の断片に `#borrow` を足したものであり、名前空間の欄を書き替えない。
  `FullName::is_local` の本体は `self.namespace.is_local()`、`NameSpace::is_local` の本体は
  `self.names.len() == 0` なので、`is_local` は名前空間が空かを答える。よって返る名前も局所名では
  ない。一方 `L8` (a) より
  `insert_rc` が `Retain` を置くのは `insert_into_operation_let` の `if v.name.is_local()` の門の中と、
  `retain_if_live` の `var.name.is_local()` を要求する枝の 2 種で尽きるので、`Retain(v, π)` の `v` は
  局所名である。**借用版の側でもそうである** -- `clone_func` が作る複製の束縛名は
  `fresh_rename_function` を通じて `assign_fresh_name` が付けたものであり、`assign_fresh_name` は
  `name.clone()` の `name` 欄だけを `format!("{}#{}{}", …)` で書き替えて名前空間の欄を元のままに
  するので、局所名の像は局所名である。よって
  差し替えが起きる callee の名前は `v` と異なり、`rhs_operands` はその `App` について `v` を
  `Ownership::Own` で挙げたままである。

<1>4. `before` の各 `Retain(a, u)` について、その直後の非 `Retain` 節点はその `App` の `Let` であり、
      `a` はその `args` の元である。
  BY <1>3, CODE src/rc_ir/borrow.rs: RewriteCtx::call_rc,
     CODE src/rc_ir/rc_insert.rs: rhs_operands
  `call_rc` は `args` の元だけを `before` に入れる。`rhs_operands` の `App` の腕は callee と `args` の
  各元に `Ownership::Own` を与えるので、これは `L9` の (a) の形である。

<1>5. `clone_func` が作る借用版は元の本体の束縛変数を一斉に付け替えたものである。
  BY <ref id=63eadd9/>
  P9 より複製はそれ以外の違いを持たないので、節点の種類も並びも変わらない。一斉の付け替えは
  1 つの束縛名を 1 つの新しい名前へ写す全単射であり、束縛とその使用の両方を同じ像へ写すので、
  `Retain` が名指す束縛と、その直後の構文が名指す束縛との一致は保たれる。よって `L9` の (a)-(d) の
  形は、名前を像へ読み替えた形で保たれる。

<1>5a. `cancel` の入力は、`insert_rc` の出力に `split_rc_units` と `borrow_ify` をこの順に掛けた
      ものであり、間にプログラムを書き換えるパスは無い。
  BY <ref id=24bf090/>, CODE src/build/build_object_files.rs: lower_and_insert_rc,
     CODE src/build/build_object_files.rs: optimize_rc_program,
     CODE src/build/build_object_files.rs: build_object_files,
     CODE src/rc_ir/validate.rs: validate
  P15 は「**`cancel` の入力が `borrow_ify` の出力であることは、`optimize_rc_program` の 1 か所を
  読めば決まる。**」と述べる。`build_object_files` は `lower_and_insert_rc` の返り値をそのまま
  `optimize_rc_program` に渡し、`lower_and_insert_rc` は `insert_rc` を掛けて返る。
  `optimize_rc_program` はその値に `split_rc_units` を掛け、続けて `borrow_ify`、`cancel` を掛ける。
  間に挟まるのは `config.develop_mode` のときだけ走る `validate` の呼び出しであり、それは
  `&RcProgram` を受け取って検査するだけで書き換えない。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5, <1>5a, <ref id=19c0e5a/>
  <1>5a より `cancel` の入力の各本体は、`insert_rc` の出力の本体に `split_rc_units` と `borrow_ify` を
  掛けたものである。`insert_rc` の出力が持つ形 (`L9`) は `split_rc_units` (<1>1) と `borrow_ify`
  (<1>2、<1>3、<1>3a、<1>5) を通って残り、`borrow_ify` が足す `Retain` も同じ形を持つ (<1>4)。

## 8. (O1) と (O2) の言明

`L7` より、**計数下の別名類であって開始事象を `ρ` の上にちょうど 1 つ持つもの** -- `L6` の前提 (I) と
`L4` の前提を満たすもの -- について、A19 (ii-b) は `bumps ≥ 1` である各時点で `U + X ≥ D` である。
`L9` と
`L11` は、`R` を増やす節点 -- `Retain` -- が、その変数を名指す構文の直前にしか立たないことを示す。
README の A19 が (O1) と (O2) と呼ぶ 2 つを、この節が言明として書く。

**(O1) 由来の形。** `insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化が、第 9 節の
`DEF 由来の形` の 3 つの節を満たすこと。すなわち (a) 各時点の
各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` であり、(b) D7 の読む構文が読む値の各スロット、および
`Retain(v, π)`・`Release(v, π)` が触れる各スロットについて、そのスロットが属する計数下の別名類は、その
節点の入口で `held ≥ 1` であり、(c) 関数本体・初期化子の終端の `Ret` の消費を行った直後の点でも
`held ≥ 0` である。これは D11 を別名類の粒度へ絞った主張であり、A19 (ii-a) が要求するものである
-- (c) は A19 (ii-a) の「非負であることは、終端の `Ret` の消費を行った直後の時点についても言う」の
分である。**第 10 節の `L19` (a)-(c) が示す。** 併せて `L20` が「各計数下の別名類 `C` の開始事象
(`DEF 開始事象`) は `ρ` の上に高々 1 つである」を示す。**`L4` と `L7` の前提はこれより 1 段強い** --
それは「ちょうど 1 つ」である。`ρ` の上にスロットを持つ計数下の類については、下の 1 つが渡る --
`L13a` (f) よりその類の ρ-終端はパラメータ・capture の leaf か D10 の生成の表の位置のいずれかであり、
`DEF 開始事象` の 3 行がその 2 種を覆うので、開始事象は少なくとも 1 つ在る。**その事象がその類の
スロットが在る時点以前であることは `L13a` (h) による** -- ρ-歩みの各段は、進む先の変数がいま居る
位置の変数より前に値を得る位置へ進むので、ρ-終端の変数はその類のどのスロットの変数以前にも値を得る。
`L25` `<1>2c` がその形で渡す。

**節点の実行の途中の点は (O1) の外にある。** A19 (ii-a) が量化するのは節点の訪問の入口である時点だけで
あり (`DEF 時点`)、その粒度は A19 が (ii-c) として別に立てている。README の A19 は (ii-c) の果たす者を
「果たす者は `insert_rc` (`p60-insert-rc.md` の `L19` (d)) と `borrow_ify`
(`p20-borrow-ify.md` の第 13 節) である。」と 2 人挙げる。この文書が果たすのは前者であり、`L19` (d) が
その段である。**その量化は README の (ii-c) の 2 文目に従う** -- 節点の実行の途中の各点で
`held ≥ 0` を出すのは、その点で `held_ρ` が定まる各計数下の別名類についてである (`DEF 時点`)。

**(O2) 帳簿の遅れが無いこと。** 各時点の各計数下の別名類について、`bumps ≥ 1` ならば
`held ≥ 1 + bumps` である。`L7` より `U + X ≥ D` と同値であり、A19 (ii-b) が要求するものである。
**第 11 節の `L25` が前提 (S) の下で示し、`L20a` が `insert_rc` の出力について (S) を与える。**

### 8.1 (O2) の証明が本体について読むもの

第 11 節が本体とそれが属するプログラムについて読む**量的な**前提は、11.1 節の (S) が挙げる 5 つ --
(S0) 各時点で各スロットの `μ` が非負であること、(S1) すべての unit が所有されること、
(S2) 各 `Retain` 節点の入口でその節点が触れる各スロットの `μ` が 1 以上であること、(S3) 各計数下の
別名類に開始値 1 を与える事象が高々 1 つであること、(S4) 各節点の入口でその節点が `μ` を下げる回数
以上の `μ` が在ること -- だけである。`insert_rc` の出力がその 5 つを
満たすことは `L20a` が第 10 節から出し、`split_rc_units` の出力については `L32` が同じ 5 つを
確かめる。
**構文と名前の規律はこの 5 つの外に在る。** 第 11 節の段は、A6 (名前の一意性)・A11 (スコープの
規律)・D2 (本体の木とスコープの規則)・A13 (名前の形) を本体について読む -- `L24` の各場合が
ρ-歩みが辿る変数の値を得る順序を `L13a` (h) 経由で読み、`L21` `<1>5a` が、束縛を持たない名前が
局所名でないことを A13 から読む。**A13 を引く段は A2 を併せて引く** (第 1 節の規則)。`L32` は
この 4 つを、`L26` (b) が写しと束縛をそのまま運ぶことから `split_rc_units` の出力へ渡す。

不等式の形はこうである。台帳が処分に遅れうるのは、処分されるスロットの名前を、pending な要素の
`outstanding` が名指さないときだけであり、`L23` がその場合を切り分ける。名前どうしの関係 `Anc` は
`Ids(C)` を木にし (`L21`)、走査の帳簿はその木の各名前に付く。第 11 節はこれを木の各部分木についての
1 つの不等式 (`L24`) に束ね、木の根で読んで `held ≥ 1 + bumps` を得る (`L22`、`L25`)。

### 8.2 `split_rc_units` の段

A19 (ii) の範囲は「**`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち
`cancel` の入力) の両方**」であり、この文書が扱うのはその第 1 の半分である。`borrow_ify` の入力は
`split_rc_units` の出力である。第 10 節と第 11 節が示すのは `insert_rc` の出力に
ついてなので、`Retain(v, [])` を unit ごとの鎖へ割る段が 1 つ残る。**第 13 節がそれを扱う。**

- **(ii-a)** は保存の形で示す (`L31`)。割る段は束縛を 1 つも作らないので `origin`・別名類・`held` は
  変わらず (`L29`)、鎖の途中に新しく生まれる時点だけが検査の対象になる。入力の側 -- `insert_rc` の
  出力 -- は `L19` (a)-(c) が与える。
- **(ii-c)** も保存の形で示す (`L31a`)。量化するのは節点の実行の途中の各点なので、写しの節点では
  素動作の列が 2 つの側で同じであることを、鎖の節点では `held` がその中で単調であることを使う。
  入力の側は `L19` (d) が与える。
- **(i)** はこの段に何も要求しない。(i) は仮定ではなく D21 が活性化に課す制限であり、実行が作る
  活性化がそれを満たすことは P28 (b) が示す。この段の出力には借用する unit が無い
  (`L15` (e)、`L26` (a)) ので、その本体のどの別名類でも (i) の角括弧は 0 である (`L13a` (c))。
- **(ii-b)** は、割る段の出力が第 11 節の前提 (S) を満たすことを示して `L25` を当てる (`L32`)。保存の
  形では通らない -- 鎖に割ると `pending` の要素が細かくなり、粗い 1 要素なら落ちた場面で細かい要素の
  一部が残ることがあるので、`bumps` は割る前より大きくなりうる。

## 9. A19 を読む 2 つの形

README の A19 は (ii-a) と (ii-b) に別々の読み手を挙げる -- (ii-a) の項は「**(ii-a) 由来の形** --
読む者: P14、P18a、P18c。」で始まり、(ii-b) の項は「**(ii-b) 帳簿の形** -- 読む者: P18a、P18c、P19、
P21。」で始まる。P18a と P18c はどちらの項にも挙がっており、A19 の脇はそれを「**P18a と P18c は
`cancel` の入力について (ii-a) も読む**」と述べる。(ii-b) は走査の `pending` を主語にする形であり、
(ii-a) は次の形である。

**DEF 由来の形**。次の 3 つが成り立つこと。

- **(a)** `ρ` の上の各時点 `τ` と各計数下の別名類 `C` について `held_ρ(τ, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その時点で `held_ρ(τ, C) ≥ 1` である。
- **(c)** 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った直後の点でも、各計数下の別名類 `C` に
  ついて `held_ρ(・, C) ≥ 0` である。

**3 つの節は A19 (ii-a) の 3 つの節そのものである。** (a) と (b) は `DEF ii-a と ii-b の読み` が引く A19 (ii-a) の本文
「各時点と各計数下の別名類について、その類が持つ参照の個数は非負であり、読む構文と
`Retain`/`Release` がその類を名指す時点では 1 以上である」の 2 つ、(c) は続く
「**非負であることは、終端の `Ret` の消費を行った直後の時点についても言う。**」である。
(c) を落とすと、この文書の量化は A19 (ii-a) より弱いものになる。

「由来」-- `origin` の再帰を実行が辿った枝に沿って追い切った先 -- は D33 の `ρ` 終端であり
(D34 も「**ここでいう終端は D33 の `ρ` 終端である。**」と書く)、別名類は ρ-終端が等しいスロットの
集まりなので (D33)、その類が持つ参照の個数は `held_ρ(·, C)` である。

**(ii-a) と (ii-b) はどちらも他方を導かない。** 9.1 と 9.2 がそれぞれの向きの反例を挙げる。

### 9.1 `L12` (`C1` は由来の形を満たし、A19 (ii) を破る) <!--#eddc8aa-->

**言明**。`DEF C1 の本体` の `main` は、変位 0 を選ぶ実行路 (`L5` の `ρ_0`) で由来の形を満たす。
同じ実行路の `Let(w, App(f, [q]), ·)` の入口では `held_ρ0(・, C_1) = 1` かつ
`bumps_ρ0(・, C_1) = 1` であり、`held ≥ 1 + bumps` は偽である。すなわち `C1` の `main` は
A19 (ii) を破る。

**証明**

<1>1. その実行路 -- `L5` の `ρ_0` -- の上のスロットは `(p, [])`、`(q, [])`、`(m, [])` の 3 つで
      あり、`c`・`u`・`w` とアームの payload `y0`・`y1` はスロットを持たない。`App` の callee `f` の
      boxed leaf との対は D6 の記号の位置であってスロットではなく、そこが指すオブジェクトは計数下では
      ない。`ρ_0` の上の計数下の別名類は `C_1 = {(p, []), (m, [])}` と `C_2 = {(q, [])}` の 2 つで
      あり、どちらも `ρ_0` の上に開始事象をちょうど 1 つ持つ。
  BY <ref id=1aab97c/>, <ref id=c2ea78f/>, DEF `C1` の本体, DEF 例の型と op,
     <ref id=0594f24/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=83d98e9/>,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     CODE src/ast/types.rs: TypeNode::is_box
  **類の分割と開始事象の個数は `L5` (a) が与える** -- `L5` の `ρ_0` は変位 0 を選ぶ実行路であり、
  この命題が主語にする実行路である。
  スロットの側は次のとおりである。`DEF C1 の本体` の `main` が名指す変数は `p`、`q`、`c`、`m`、`u`、
  `w` と、アームの payload `y0`・`y1` と、`App` の callee `f` である。`ty(p) = ty(q) = ty(m) = Arr` は
  boxed なので `is_fully_unboxed` は偽であり (`if self.is_box(type_env) { return false; }` で始まる)、
  `is_box` の本体は `!self.is_unbox(type_env)`、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` なので `is_closure` も偽である。
  よって D4 の判定は第 1 規則と第 2 規則を抜けて第 3 規則に
  着き、`boxed_leaf_paths(Arr) = {[]}` である。`ty(c) = Bl` は 2 つの変位がどちらも payload を
  持たない unbox union なので boxed leaf を持たず、payload `y0`・`y1` も同じである。
  `ty(u) = ty(w) = I` は `is_fully_unboxed` が真なので D4 の第 1 規則より leaf を持たない。
  **`m`・`u`・`w` の型は A12 が与える** -- `DEF C1 の本体` は `Match` の束縛変数と `App` の束縛変数に
  型を書かない。A12 の「アームの結果と `Match` の束縛変数の型」の節より `ty(m)` は 2 つのアームの結果
  `p`・`q` の型に等しく `Arr` である。A12 の「`Let(x, App(callee, args), k)` の `ty(x)` は呼び出し先の
  返り値の型である」の節より `ty(u)` と `ty(w)` は `f` の返り値の型 `I` である。
  `App` の callee `f` は束縛を持たない名前なので、その boxed leaf との対は D6 の記号の位置であって
  スロットではない。`L13a` (i) よりその位置が指すのはグローバル状態のオブジェクトであり、D26 より
  計数下ではない。

<1>2. `held(C_1)` は、割り当ての後 1、`Retain(m, [])` の後 2、`App(f, [p])` の消費の後 1、
      `Release(m, [])` の後 0 である。`held(C_2)` は、割り当ての後 1、`App(f, [q])` の消費の後 0 で
      ある。
  BY <1>1, DEF `C1` の本体, <ref id=94fad8e/>, <ref id=9d5d254/>, <ref id=9d74736/>, <ref id=f06144e/>
  `DEF C1 の本体` の `main` の節点を `ρ` の順に見る。`p` と `q` の割り当ては D10 の生成で
  あり、開始値 1 を与える。`Let(m, Match(c, ...), ·)` は変位 0 のアームを選び、そのアーム本体の
  `Ret(p)` が D9 の移動を行うが、移動は `held` を変えない (D34 の表に移動の行は無い)。
  `Retain(m, [])` は `(m, []) ∈ C_1` を名指し、`Release(m, [])` も同じである。`App(f, [p])` は
  `(p, []) ∈ C_1` を、`App(f, [q])` は `(q, []) ∈ C_2` を消費する (D9、`f` は `b` を所有する)。
  `Eval(m, ·)` は D9 の消費でも移動でもない。終端の `Ret(u)` は `ty(u) = I` に boxed leaf が無いので
  何も消費しない。

<1>3. 由来の形の (a) が成り立つ。
  BY <1>1, <1>2
  <1>1 より `ρ_0` の上の計数下の別名類は `C_1` と `C_2` で尽き、<1>2 よりその `held` の値はすべて
  0 以上である。

<1>4. 由来の形の (b) が成り立つ。
  BY <1>1, <1>2, DEF `C1` の本体, DEF 例の型と op, <ref id=56c2068/>
  `DEF C1 の本体` の `main` の読む構文と `Retain`/`Release` を順に見る (D7 の表)。
  `Let(p, Llvm(alloc, []), ·)`、`Let(q, Llvm(alloc, []), ·)`、`Let(c, Llvm(mkbl, []), ·)` は
  D7 の表の第 1 行に当たるが、`DEF 例の型と op` より `alloc` も `mkbl` もオペランドを持たないので、
  読む値が 1 つも無い。
  `Let(m, Match(c, ...))` が読むのは scrutinee `c` であり、<1>1 より `c` はスロットを持たない。
  `Retain(m, [])` は `(m, [])` に触れ、その時点の `held(C_1) = 1` である。
  `Let(u, App(f, [p]), ·)` は callee と `p` を読み、`p` の側は `held(C_1) = 2` である
  (callee の側は <1>1 より計数下でない)。`Let(w, App(f, [q]), ·)` は `q` を読み `held(C_2) = 1` で
  ある。`Eval(m)` は `m` を読み `held(C_1) = 1` である。`Release(m, [])` は `(m, [])` に触れ
  `held(C_1) = 1` である。終端の `Ret(u)` は D7 の読む構文ではない。

<1>4a. 由来の形の (c) が成り立つ。
  BY <1>1, <1>2, DEF `C1` の本体, DEF 例の型と op, <ref id=0594f24/>, <ref id=9d74736/>,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  `C1` の `main` の終端は `Ret(u)` であり、`ty(u) = I` は `DEF 例の型と op` より `is_fully_unboxed` が真
  なので D4 の第 1 規則より boxed leaf を持たない。よって D9 の消費の表の「本体の終端の `Ret(x)`」の
  行が挙げる leaf は無く、この消費は `held` を動かさない。<1>2 よりその点までに `held(C_1)` は
  `Release(m, [])` の後 0、`held(C_2)` は `App(f, [q])` の消費の後 0 であり、<1>1 より計数下の類は
  この 2 つで尽きるので、消費の直後の点でどちらも `0 ≥ 0` である。

<1>4b. `nm(m, []) = (m, [])` であり、`nm(p, []) = (p, [])` である。よって `C_1` のスロットの
       `identity` の全体は `{(m, []), (p, [])}` である。
  BY <1>1, <ref id=1aab97c/>, <ref id=e11772a/>, <ref id=b1f6e13/>, <ref id=3e6b0e0/>, DEF `C1` の本体, DEF 例の型と op, DEF `nm(s)`,
     CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var,
     CODE src/rc_ir/ownership.rs: Origin, CODE src/rc_ir/ownership.rs: Origin::of_candidates,
     CODE src/rc_ir/ownership.rs: Origin::identity, CODE src/rc_ir/ownership.rs: Origin::acted_on,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  `origin` は memo に鍵が無ければ `origin_inner` を走らせ (A15 より `grow_stack` はその閉包を
  ちょうど 1 回呼ぶ)、P2a より答えは memo の状態に依らない。
  `p` と `q` は `Llvm(alloc, [])` に束縛されるので `collect_bindings` は `Binding::Llvm` を入れ、
  A3 と `DEF 例の型と op` より `alloc` の結果の leaf `[]` の宣言は単一の `Fresh` なので
  `as_arg_projection` は `None` を返し、オペランドが無いので `origin_from_leaves_under` は
  `produced_here` の 1 元だけを積む。よって
  `origin(p, []) = Exactly((p, []))`、`origin(q, []) = Exactly((q, []))` である。
  `m` は `Match` の束縛変数なので `collect_bindings` は `Binding::Join(arm_results)` を入れ、
  `returned_var` が各アーム本体の `Ret` の変数を取るので `arm_results` は `[p, q]` である。
  `origin_inner` の `Binding::Join` の腕は各アームの結果の `origin` の `acted_on()` を集めるので
  `candidates = {(p, []), (q, [])}` であり、元数が 2 なので `of_candidates` は
  `Join { identity: (m, []), candidates }` を返す。`Origin::identity` はその `identity` を返すので
  `DEF nm(s)` より `nm(m, []) = (m, [])` である。`Exactly(t)` について `identity` は `t` なので
  `nm(p, []) = (p, [])` である。<1>1 より `C_1 = {(p, []), (m, [])}` なので、その `identity` の
  全体はこの 2 つである。

<1>5. `Retain(m, [])` の訪問が押し込む要素の `outstanding` は `{(m, []): 1}` であり、その `B` も
      `{(m, []): 1}` である。`App(f, [p])` の訪問はその要素を `pending` から取り除かず、`Eval(m, ·)`
      の訪問も取り除かない。よって `Let(w, App(f, [q]), ·)` の入口で `bumps_ρ0(・, C_1) = 1` で
      ある。<1>2 よりその入口で `held_ρ0(・, C_1) = 1` なので、`held ≥ 1 + bumps` は偽である。
  BY <1>1, <1>2, <1>4b, <ref id=cbc4a1c/>, <ref id=8093b68/>, <ref id=9f1cf6c/>, <ref id=66c9670/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=dca1c02/>, <ref id=ef8efc4/>, <ref id=9d74736/>,
     DEF `C1` の本体, DEF ii-a と ii-b の読み,
     CODE src/rc_ir/ownership.rs: acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/ownership.rs: Origin::acted_on, CODE src/rc_ir/ownership.rs: References,
     CODE src/rc_ir/ownership.rs: rhs_consumes, CODE src/rc_ir/ownership.rs: all_owned_units,
     CODE src/rc_ir/borrow.rs: cancel
  `walk_inner` の `RcExpr::Retain` の腕は `outstanding` に `self.acted_references(m, [])` を置く。
  **その呼び先は `CancelAnalysis` のメソッドであり、それは自分の `vars` と `type_env` を添えて
  `ownership::acted_references` を呼び、その返り値をそのまま返す** -- 間に挟まるのは
  `!references.is_empty()` の表明だけである。D15 よりその値は `[]` の下の各 boxed leaf の
  `origin` の `identity` を数えたものであり、<1>1 より `ty(m) = Arr` の boxed leaf は `[]` の 1 つ、
  <1>4b よりその `identity` は `(m, [])` なので `{(m, []): 1}` である。D27 が数えるのは inhabited
  (D16) かつ計数下 (D26) の leaf に限るが、<1>1 より `(m, [])` は `ρ_0` の上のスロットであり
  (D6 よりスロットの leaf は inhabited)、`obj(C_1)` は計数下なので、`B` も `{(m, []): 1}` である。
  `App(f, [p])` の訪問は `consume_rhs` を通じて `rhs_consumes` が挙げる leaf について `consume` を
  呼ぶ。**所有位置の判定は `owns` が決める** -- `consume_rhs` が `rhs_consumes` に渡す `owns` は
  `self.owned_units.contains(…)` であり、`cancel` は `owned_units` に
  `all_owned_units(prog, type_env)` を置く。D14 より `all_owned_units` が返すのは各関数の
  パラメータ・capture の unit のうち `borrowed_units` に入らないものであり、`DEF C1 の本体` より
  `f` の `borrowed_units` は空なので `owns` は真である。よって `rhs_consumes` が挙げるのは callee `f` の
  各 boxed leaf と `(p, [])` である。`consume` はそれぞれについて `origin(・).acted_on()` を
  `consume_objects` へ渡し、<1>4b より `(p, [])` の分は `[(p, [])]`、callee の分は `[(f, ・)]` である
  (<1>1 より callee は束縛を持たない名前なので `origin_inner` は `Exactly` を返す)。
  `consume_objects` が落とすのは `outstanding` がそれらの名前を含む要素であり (D15 の `names`)、
  この要素の `outstanding` は `(m, [])` だけを含むので落ちない。
  `walk_inner` の `RcExpr::Eval` の腕は `pending` をそのまま渡す。
  A19 (ii-b) が定める `bumps` の帰属 -- 「時点 `τ` の `pending` の各要素 `p` について D27 が定める
  `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和」-- を <1>4b の
  `{(m, []), (p, [])}` に当てると、この入口で `bumps_ρ0(・, C_1) = 1 + 0 = 1` である。
  `DEF ii-a と ii-b の読み` では `bumps ≥ 1` のとき `held ≥ 1 + bumps` が要るが、<1>2 より
  この入口の `held_ρ0(・, C_1) = 1` である。

<1>6. QED
  BY <1>3, <1>4, <1>4a, <1>4b, <1>5

### 9.2 `L13` (A19 (ii) を満たし、由来の形を破る本体 `B_1`) <!--#eab5fd1-->

`DEF 例の型と op`・`DEF 例の名前の取り方`・`DEF 関数 h` を使う。本体 `B_1` (`main`、パラメータ
無し、**capture 無し**、返り値の型 `I`) を次で定める。

```
Let(o, Llvm(alloc, []),
Let(y, App(h, [o]),
Eval(o,
Let(u, App(f, [y]),
Ret(u)))))
```

**言明**。`B_1` はどの活性化でも A19 (ii) を満たし、由来の形の (b) を破る。**`App(h, [o])` の段が
`o` の値をそのまま返す活性化について、`B_1` は D11 を満たす。**
`B_1` は `insert_rc` の出力ではない。

**D11 の側に活性化の制限が付く理由。** D21 は「子の活性化を作る段」の結果 -- 返る値と `H` に与える
変化 -- を活性化の側の与件に置く。制限を外すと、`App(h, [o])` が `obj(o, [])` とは別のオブジェクトを
返し、その段の子が `obj(o, [])` への参照を処分して `H` を 0 にする活性化が入り、`Eval(o)` が
解放されたオブジェクトを読む。**その活性化も A19 (i) を満たす** -- `App` の消費の後
`held_ρ(・, C_o) = 0` であり、`C_y` は別のオブジェクトを指す類なので、A19 (i) が `obj(o, [])` に
ついて要求するのは `H ≥ 0` だけである。
**制限を置いた側ではこの形が起きない。** `App(h, [o])` が `o` の値をそのまま返す活性化では
`obj(C_y) = obj(C_o) = O` であり、その消費の後 `held_ρ(・, C_y) = 1` なので、A19 (i) が
`H(O) ≥ 1` を要求し、D21 はそれを破る活性化を活性化と呼ばない。**子の `H` への効果を別に縛る要は
ない** -- 縛るのは A19 (i) であって、`h` の本体の形ではない。
**制限は `L13` の役目を変えない** -- 由来の形 (b) の破れも A19 (ii) の成立も、どの活性化でも
成り立つ。

**証明**

<1>1. `B_1` の実行路は 1 本であり、その上のスロットは `(o, [])` と `(y, [])` の 2 つ、別名類は
      `C_o = {(o, [])}` と `C_y = {(y, [])}` の 2 つである。`C_o` はどの活性化でも計数下である。
      `C_y` が計数下であるかどうかは活性化が決める -- `obj(y, [])` は `App(h, [o])` の段の結果で
      あり、D21 はそれを活性化の側の与件に置く。言明が限る活性化では `obj(C_y) = obj(C_o)` なので
      `C_y` も計数下である。
  BY <ref id=ca36627/>, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox
  `Match` が無いので実行路は 1 本である。`collect_bindings` は `Let(o, Llvm(alloc, []), ・)` に
  `Binding::Llvm(alloc, [], Arr)` を入れる。A3 と `DEF 例の型と op` より `alloc` の宣言は単一の `Fresh` なので
  `as_arg_projection` は `None` を返し、`origin_from_leaves_under` はオペランドを持たないこの op に
  ついて `Exactly((o, []))` を返す。よって `origin(o, []) = Exactly((o, []))` である。`y` は
  `RcRhs::App` に束縛されるので `collect_bindings` は `Binding::Producer` を入れる。
  D33 が ρ-歩みを止める 3 種のうち、第 1 種「辺を持たない束縛、すなわち `Binding::Param`、
  `Binding::Producer`、および束縛を持たない名前 (記号の位置)」が `(y, [])` に、第 2 種
  「`Binding::Llvm` であって、`λ` の宣言が単一の `Fresh` または単一の `Unknown` である位置」が
  `(o, [])` に当たるので、どちらも ρ-終端であり、別々の類である。
  `ty(o) = ty(y) = Arr` の leaf が `[]` の 1 つであることは D4 の判定から出る -- `is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `Arr` では偽であり、`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` で `is_box` はその否定なので `Arr` では
  `is_closure` が偽である。よって判定は第 1 規則と第 2 規則を抜けて第 3 規則に着き、
  `boxed_leaf_paths(Arr) = {[]}` である。`ty(u) = I` は `DEF 例の型と op` より `is_fully_unboxed` が真なので
  D4 の第 1 規則より leaf を持たない。
  `B_1` に現れる残りの `RcVar` は `App` の callee `h` と `f` であり、どちらも `collect_bindings` が
  束縛を入れない名前なので、その boxed leaf との対は D6 の記号の位置であってスロットではない。
  D6 よりそこが指すのは funptr かグローバル状態のオブジェクトであり、D26 よりグローバル状態の
  オブジェクトは計数下ではない。
  `C_o` の計数下性は `alloc` から出る。A3 の表の「単一の `Fresh`」の行は、生成コードが結果のその
  leaf に「新しく割り当てたオブジェクトへの新しい参照」を置くと述べる。A3 がこの行を字義どおりに
  読ませないのは「実行時に参照カウントで分岐する op」についてであり、`DEF 例の型と op` より `alloc` は
  オペランドを持たないのでその形ではない。D26 より割り当てられたオブジェクトは計数下である。
  D33 より 1 つの別名類のすべてのスロットは同じオブジェクトを指すので `obj(C_o) = obj(o, [])` で
  あり、`obj(C_y) = obj(y, [])` である。言明が限る活性化では `App(h, [o])` が `o` の値をそのまま
  返すので `obj(y, []) = obj(o, [])` であり、`C_y` も計数下である。

<1>2. どの活性化についても、`Obl` は割り当ての後 `{obj(o, [])}`、`App(h, [o])` が返った後
      `{obj(y, [])}`、`App(f, [y])` が返った後 空である。言明が限る活性化では
      `obj(y, []) = obj(o, []) = O` なので、この 3 つは `{O}`、`{O}`、空である。
  BY <1>1, <ref id=596a46d/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=88a06de/>, <ref id=c232680/>, 言明が限る活性化,
     DEF `C1` の本体 (関数 `f` の `borrowed_units` は空),
     DEF 関数 `h` (関数 `h` の `borrowed_units` は空)
  `f` と `h` の `borrowed_units` が空であることは、この 2 つの関数を据えた `DEF C1 の本体` と
  `DEF 関数 h` の取り決めである。
  D14 は借用する unit の集合を `RcFunc::borrowed_units` と定め、残りを所有すると定めるので、
  `h` の `a` も `f` の `b` も所有される。
  `App(h, [o])` は `(o, [])` を消費し、`H` を動かさない。同じ節点について D9 の `App` の行は callee の
  全 boxed leaf も消費として挙げるが、<1>1 より `(id, ・)` と `(f, ・)` は記号の位置であってスロットでは
  なく、D6 よりそこが指すのは funptr かグローバル状態のオブジェクトである。D26 よりそれらは D8 の意味の
  参照を持たないので、この消費は `Obl` から何も取り除かない。
  同じ節点の D10 の生成の表の `App` の行が結果の leaf `(y, [])` の参照を作る。
  `App(f, [y])` は `(y, [])` を消費する。`Ret(u)` は `ty(u) = I` に boxed leaf が無いので何も
  消費しない。**この推移はどの活性化でも同じである** -- 動かすのは節点の種類と D9・D10 の表だけで
  ある。言明が限る活性化では `App(h, [o])` の段が `o` の値をそのまま返すので
  `obj(y, []) = obj(o, []) = O` である。

<1>2a. `held_ρ(・, C_o)` は割り当ての後 1、`App(h, [o])` の消費の後 0 である。`C_y` が計数下で
       あるとき、`held_ρ(・, C_y)` は `App(h, [o])` の生成の後 1、`App(f, [y])` の消費の後 0 で
       ある。どちらもどの時点でも 0 以上である。**この推移は、その類が計数下である活性化ではどれでも
       同じである** -- D34 は計数下の類にしか `held` を定めないので、`C_y` が計数下でない活性化では
       `held_ρ(・, C_y)` は量化の外に落ちる。
  BY <1>1, <1>2, <ref id=94fad8e/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=9d5d254/>, DEF 開始事象
  `B_1` に `Retain`/`Release` 節点は無いので、D34 の表のうち動くのは開始値の行と消費の行だけで
  ある。`C_o` の開始は D10 の生成の表の `Llvm` の行、`C_y` の開始は同じ表の `App` の行であり、
  消費は D9 の `App` の行の 2 つである。**`L4` の前提はここで満たされる** -- `DEF 開始事象` に当たる
  事象はどちらの類についても 1 つだけである。`L4` の等式 `held = 1 + R - D` に `R = 0` を当てると上の
  値になる。どの事象も節点の種類と D9・D10 の表だけで決まるので、活性化に依らない。

<1>3. 言明が限る活性化について、`B_1` は D11 を満たす。
  <2>1. `App(h, [o])` が返った時点から `App(f, [y])` の消費までの各時点で `H(O) ≥ 1` であり、
        その活性化がその時点まで解放について閉じている (D11a) ならば `O` はその時点で解放されて
        いない。同じことが `Let(y, App(h, [o]), ·)` の入口についても成り立つ。
    BY <1>1, <1>2, <1>2a, <ref id=9f1cf6c/>, <ref id=859cf84/>, <ref id=c232680/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=c2ea78f/>
    D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と
    定め、その不等式の本文 -- 角括弧、総和 `S`、`d(C)` -- は A19 (i) が置く。`B_1` の関数 `main` は
    パラメータも capture も持たないので、`L13a` (c) より `β ≡ 0` であり、角括弧は 0、各類について
    `d(・) = held(・, ・)` である。言明が限る活性化について、<1>1 より `ρ` の上の計数下の別名類は
    `C_o` と `C_y` の 2 つであり、<1>2 より `obj(C_o) = obj(C_y) = O` なので、A19 (i) の総和 `S` は
    この 2 つのうち開始済みのものからなる。<1>2a よりどちらの `held` も非負なので、`H(O)` は
    その一方の値以上である。
    `Let(y, App(h, [o]), ·)` の入口では `held(C_o) = 1`、`App(h, [o])` が返った時点から
    `App(f, [y])` の消費までは `held(C_y) = 1` なので、どの点でも `H(O) ≥ 1` である。
    D11a は、解放について閉じている時点では `H(O) ≥ 1` である各計数下オブジェクトが解放されて
    いないと定める。
  <2>2. (S-a) と (S-b)。
    BY <1>2, <ref id=9d74736/>, <ref id=95427eb/>
    (S-a): `Obl` から参照を取り除くのは 2 つの `App` の消費だけであり、<1>2 よりどちらの時点でも
    `Obl` は取り除かれる参照を持つ。(S-b): 終端の `Ret(u)` の消費 -- `ty(u) = I` に boxed leaf が
    無いので何も消費しない -- の後、`Obl` は空である。
  <2>3. (S-c)。
    BY <1>1, <1>2a, <2>1, <ref id=b6673ca/>, <ref id=fd95f12/>, <ref id=56c2068/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=88a06de/>
    D7 の読む構文はこの本体に 4 つ在る。`Let(o, Llvm(alloc, []), ·)` は `DEF 例の型と op` より
    オペランドを持たないので読むオブジェクトが無い。残る 3 つのうち 2 つの `App` は callee を読むが、
    <1>1 より callee は記号の位置であり、そこが指すのは funptr かグローバル状態のオブジェクトなので、
    A8 と D26 よりそれが解放されることは無い。`Retain`/`Release` 節点はこの本体に無い。
    残るのは `Let(y, App(h, [o]), ·)` が読む `o`、`Eval(o, ·)` が読む `o`、
    `Let(u, App(f, [y]), ·)` が読む `y` の 3 つであり、<1>2 よりどれも `O` を指す。
    D24 の「**読みの直前の点では、勘定は直前の段内の点のものである。**」より、読みの直前の点の
    勘定は直前の段内の点のものである。**その段内の点は、その節点の入口の勘定を持つ** -- A26 より
    読みはその節点が行うどの手放しよりも前に起きるので、節点の入口からその点までのあいだにこの段は
    参照を処分しておらず、D24 の (F) の解放を起こすのは参照の処分なので、そのあいだにどのオブジェクトも
    解放されない。<2>1 より
    その 3 つの入口はいずれも `H(O) ≥ 1` の点であり、(S-c) の接頭条件 (D11a) の下で `O` は
    解放されていない。
  <2>4. QED
    BY <2>2, <2>3, <ref id=95427eb/>

<1>4. `B_1` はどの活性化でも A19 (ii) を満たす。
  BY DEF ii-a と ii-b の読み, EXT 空を作る `Default`,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: cancel
  `DEF ii-a と ii-b の読み` の最後の文より、この文書が単に「A19 (ii)」と書くときそれが指すのは
  (ii-b) --「`bumps_ρ(τ, C) ≥ 1` である時点では `held_ρ(τ, C) ≥ 1 + bumps_ρ(τ, C)` である」-- で
  ある。**走査の `pending` の初期値は空である** -- `cancel` は
  `analysis.walk(body, PendingRetains::default(), true)` で走査を始める。`B_1` に `Retain` 節点は
  無く、`walk_inner` が `pending` に要素を足すのは `RcExpr::Retain` の
  腕だけなので、走査の `pending` は空のままであり `bumps ≡ 0` である。よって `bumps ≥ 1` の時点は
  無く、(ii-b) はどの類についても空虚に成り立つ。

<1>5. `B_1` はどの活性化でも由来の形の (b) を破る。
  BY <1>1, <1>2a, <ref id=56c2068/>
  `Eval(o, ·)` は D7 の読む構文であり、`o` の leaf `[]` を読む。そのスロットの類は `C_o` であり、
  <1>1 よりそれはどの活性化でも計数下で、<1>2a よりその時点の `held(C_o) = 0` である。

<1>6. `B_1` は `insert_rc` の出力ではない。
  BY <ref id=3e6b0e0/>, <ref id=d80dde9/>, <ref id=596a46d/>, DEF 例の名前の取り方,
     CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, CODE src/ast/name.rs: FullName::is_local,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
  `Let(y, App(h, [o]), cont)` を書き換える呼び出しの `live_cont` は `o` を含む -- `insert_into_eval` は
  返す `live_before` に `o` を入れる。`o` は局所名である -- `o` は `B_1` の `Let` 節点が束縛する変数
  なので `collect_bindings` がその名前を `vars.bindings` に入れ、D6 は「**逆に、`vars.bindings` に
  束縛を持つ名前は局所名である。**」と述べる。`rhs_operands(App(h, [o]))` は `o` に
  `Ownership::Own` を与え、`needs_rc(o)` は真である -- `is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `ty(o) = Arr` では偽である。
  よって
  `retains_before` に `o` が入り、`build_retains` が `Retain(o, [])` をこの `Let` の直前に置く。
  `B_1` はその節点を持たない。**その骨格が `B_1` と同じ形であることは次のとおりである。** A25 より
  `insert_rc` の入力の本体は `Retain`/`Release` を含まない。A15 より `insert_into_expr` は
  `insert_into_expr_inner` をちょうど 1 回呼び、その 5 つの腕 (`Ret`、`Match` の右辺の `Let`、
  その他の `Let`、`Destructure`、`Eval`) はいずれも骨格節点と同じ構成子・同じ変数・同じ右辺の節点を
  1 つ作り直し、`build_retains` と `build_releases` が積む鎖を、その節点の外側 (前置)・継続の側
  (後置)・アーム本体の頭のいずれかに置く。`insert_into_func` が根の外に積む `unused` の鎖も
  `build_releases` である。積む位置がどこであれ積まれるのは `Retain`/`Release` 節点だけなので、
  出力から `Retain`/`Release` 節点を取り除くと、骨格と節点の種類・変数・右辺・並びのすべてで
  一致する木が戻る (`insert_into_destructure` は入力の `RcState` を捨てて `Unknown` を置くが、
  `B_1` に `Destructure` 節点は無い)。`B_1` は `Retain`/`Release` を 1 つも持たないので、
  `B_1` が `insert_rc` の出力であるならばその骨格は `B_1` とこの形で一致し、上の勘定がそれに当たる。

<1>7. QED
  BY <1>3, <1>4, <1>5, <1>6

### 9.3 2 つの形の関係

`L12` は「由来の形 ⇒ A19 (ii-b)」の反例であり、`L13` は「A19 (ii-b) ⇒ 由来の形」の反例である。よって
**2 つの形はどちらも他方を導かない。** 1 つの仮定として置くと、P14 が読む形は文面に無く、
P18a・P18c・P19・P21 が読む形は P14 には強すぎる。

`insert_rc` の側の義務は、この 2 つに分かれる。

- **A19 (ii-a)** は `DEF 由来の形` の 3 つの節であり、それが `insert_rc` の出力について成り立つという
  言明が第 8 節の **(O1)** である。走査を読まないので、`insert_rc` の liveness の規律だけで
  書ける。`B_1` はこれを破るが (`Eval(o, ·)` の時点で `held(C_o) = 0`、`L13`)、`L13` が示すとおり
  `B_1` は `insert_rc` の出力ではない。
- **A19 (ii-b)** は「**`bumps ≥ 1` である時点では `held ≥ 1 + bumps` である。**」であり、それが
  `insert_rc` の出力について成り立つという言明が第 8 節の **(O2)** である。`L7` の恒等式より、`L4` と `L6` の前提の下で
  この形は `U + X ≥ D` と同値である。`C1` は (ii-b) だけを破る (`L12`)。

**第 10 節が (O1) を、第 11 節が (O2) を示す。** よって `insert_rc` の出力について A19 の (ii-a) と
(ii-b) はどちらも成り立つ。

## 10. (O1) の証明 -- 別名類の粒度の RC 規律

この節は `insert_rc` の出力について (O1) を示す。支えるのは `insert_rc` が持つ 1 つの等式である --
**各時点において、1 つのスロットに割り当たる参照の個数は、その変数が `insert_rc` の liveness で live
かどうかで決まる。** その集合と実行時の参照の分布が一致する、というのがこの等式である。**liveness が
使用だけから作られることは `L14` (a) が示す** -- `Λ(m) = free_locals(m) ∪ A(m)` であり、
`free_locals` が集めるのは骨格が名指す変数と束縛する変数だけである。(O1) の 3 つの節はここから出る。
`L19` はそこへ A19 (ii-c) の分 (d) を足した 4 つの節を持つ。

### 10.1 塊、検査点、live 集合、割り当て

`insert_rc` は骨格 (`DEF 骨格`) の各節点 `m` を、次の 4 つの部分へ写す。**この 4 つの部分が
出力の節点を尽くすこと、骨格節点の種類が 5 つであること、そのうちアームの頭の `Release` 鎖を持つのが
`Let(x, Match(scrut, arms), cont)` だけ・後置 `Release` 鎖を持たないのが `Ret` だけであることは、
`L17` (a0) が示す。**

- **前置 `Retain` 鎖**: `build_retains` が積む `Retain` 節点の列 (空のこともある)。核節点の外側に立つ。
- **核節点**: 骨格節点と同じ種類の節点 (`Let`、`Destructure`、`Eval`、`Ret`)。
- **アームの頭の `Release` 鎖**: `m` が `Let(x, Match(scrut, arms), cont)` であるとき、
  `insert_into_match` が各アームの本体の頭に `build_releases(head, body)` で積む `Release` 節点の列
  (空のこともある)。骨格節点は
  `Let`(`Match` でない右辺)・`Let`(`Match` の右辺)・`Destructure`・`Eval`・`Ret` の 5 種であり、
  他の 4 種はこの部分を持たない。
- **後置 `Release` 鎖**: `build_releases` が積む `Release` 節点の列 (空のこともある)。核節点と `m` の
  継続の写しの間に立つ。`Ret` はこの部分を持たない。

骨格節点 `m` の**塊**とは、この 4 つの部分の節点の全体をいう。骨格節点 `m` の**検査点**とは、`m` の前置
`Retain` 鎖の最初の節点 (鎖が空なら核節点) の入口をいう。

**塊に属さない節点が 1 つの位置にある。** `insert_into_func` が `func.body = build_releases(unused, body)`
で本体の全体に被せる `Release` 鎖は、どの骨格節点の塊にも属さず、本体の根の検査点より**前**に立つ。
`L17` (a) はこの鎖を 3 通りの第 1 として明示的に扱い、`L18` はその鎖の各入口についての勘定を与える。

**DEF `Λ(m)`、`A(m)`**。骨格節点 `m` を書き換える `insert_into_expr(m, live_after)` の呼び出しが返す
第 2 成分 (`live_before`) を `Λ(m)`、渡された `live_after` を `A(m)` と書く
(`CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr`)。

**DEF 割り当て `μ`**。活性化を固定する。`ρ` の上の各時点 (節点の訪問の入口) `τ` と各スロット `(v, λ)`
(D6) について、整数 `μ_τ(v, λ)` を次の規則で定める。値は活性化の開始時にはすべて 0 であり、`ρ` の上の
次の 6 種の事象がそれを動かす。

- **D10 の初期値**: 所有する各パラメータ・capture の inhabited な各 leaf `(p, λ)` について `+1`。
- **D10 の生成**: 生じた inhabited な各 leaf `(v, λ)` について `+1`。
- **`Retain(v, π)`**: `π` の下の inhabited な各 leaf `λ` について `μ(v, λ)` を `+1`。
- **`Release(v, π)`**: 同じ leaf について `-1`。
- **D9 の消費**: 消費される inhabited な各 leaf `(v, λ)` について `-1`。
- **D9 の移動**: 移動元の leaf `(v, λ)` について `-1`、移動先の leaf `(x, λ')` について `+1`。

**この 6 種が、活性化が保持する参照を動かす事象を尽くすことは、`L13a` (g) が示す。**
**`μ` の類ごとの総和が `held_ρ(・, C)` とどれだけ違うかは、`L13a` (b) が示す** -- 違いは、
借用する (D14) unit の下のパラメータ・capture の leaf を ρ-終端とする類の開始の 1 だけである。

**DEF `N_ρ(m, C)`**。`κ_C(v) := #{λ : λ は ty(v) の inhabited (D16) な boxed leaf で (v, λ) ∈ C}` と
置き、骨格節点 `m` について `N_ρ(m, C) := Σ_{v ∈ Λ(m)} κ_C(v)` と置く。

### 10.1a `L13a` (移動の辺は類の中で閉じ、`held` は `μ` の類ごとの総和に借用の開始を足したものである) <!--#c2ea78f-->

**言明**。1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。別名類 `C` について、
`C` の ρ-終端がその本体の関数が**借用する** (D14) unit の下のパラメータ・capture の leaf であるとき
`β(C) := 1`、そうでないとき `β(C) := 0` と置く。

- **(a)** D9 の移動の各辺について、移動元が**スロット** `s` であるとき、`s` と移動先のスロット `s'` は
  同じ別名類に属する。移動元が**記号の位置** (D6) であるときは、`s'` が属する別名類の ρ-終端はその
  記号の位置であり、その類は計数下ではない。
- **(b)** 各**計数下の**別名類 `C` と、D34 が `held_ρ(・, C)` を定める各時点 `τ` -- D34 の「開始の
  時点」の段落より、`C` の ρ-終端の変数が値を得る時点以後の `τ` -- について
  `held_ρ(τ, C) = Σ_{s ∈ C} μ_τ(s) + β(C)` である。計数下の類に限るのは、D34 が `held` を計数下の類に
  しか定めないからである。時点の範囲を限るのは、D34 が「その時点以後の `τ` についてだけ定まり」と
  述べ、それより前の `held_ρ(・, C)` に値を与えないからである。
- **(c)** その本体の関数のパラメータ・capture のすべての unit が所有される (D14) とき、どの類でも
  `β(C) = 0` であり、(b) は `held_ρ(τ, C) = Σ_{s ∈ C} μ_τ(s)` になる。グローバル初期化子の `init` の
  本体もこれに当たる (D1 より `init` はパラメータも capture も持たない)。
- **(d)** パラメータ・capture の leaf `(p, λ)` の ρ-終端は `(p, λ)` 自身である。したがって 1 つの
  別名類はパラメータ・capture の leaf を高々 1 つ含み、`β(C) = 1` である類 `C` は、その借用する
  unit の下の leaf をスロットとして含む。
- **(e)** D10 の生成の表の各行が参照を作る位置のスロットは、それが属する別名類の ρ-終端である。
- **(f)** 計数下の別名類の ρ-終端は、パラメータ・capture の leaf か、D10 の生成の表の各行が参照を作る
  位置のスロットかのいずれかである。
- **(g)** 活性化が保持する参照を動かす `ρ` の上の事象は、`DEF 割り当て μ` が挙げる 6 種で尽きる。
- **(h)** D33 の ρ-歩みの 1 歩が**スロット**へ進むとき、その進む先のスロットの変数は、`ρ` の上で
  いま居る位置の変数より前に値を得る。
- **(i)** 最上位の tycon が `is_funptr_tycon` を満たす型は `is_fully_unboxed` が真であり、D4 の
  第 1 規則よりその型の値は boxed leaf を持たない。したがって、束縛を持たない名前 `g` について
  `(g, λ)` が D6 の記号の位置であるとき、`obj(g, λ)` はグローバル状態のオブジェクト (D26) であり、
  A8 よりそれが解放されることは無い。

**(g) を段として立てるのは、定義の中に置くと誰の検査も受けないからである** (README 第 3 節)。
`L17` (b)(c)、`L19` (a)(d)、`L24` `<1>2` がこの網羅の上に立つ。

**(c) の前提が成り立つ範囲。** この文書が扱うのは `insert_rc` の出力と
`split_rc_units` の出力であり、前者については `L15` (e) が、後者については `L15` (e) と `L26` (a) が
すべての unit が所有されることを与える。よってこの文書の中では (b) は (c) の形で読める。`β` つきの
(b) が要るのは、借用する unit を持つ本体 -- `borrow_ify` の出力 -- についてである。

**`held` の読みは A19 (i) の本文が固定している。** A19 (i) は、借用するパラメータを持つ借用版に
ついて「`g` の類も `f_borrow` の類も開始値 1 を持つ」と書き、`d(C) = held(C) - [C の ρ-終端が借用する
(D14) leaf ならば 1]` と置く。`β` はこの角括弧である。

**証明**

<1>0. (g)。
  BY <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, DEF 割り当て `μ`
  D9 は 2 つの表の後に「上の 2 つの表と D10 の生成の表で、参照を作る・移す・手放す構文はすべてで
  ある」と述べ、続けて「`Eval(v, k)` と `Let(x, Match(v, arms), k)` の `Match` 節点自身は、参照を
  作らず、移さず、手放さない」「`Retain` と `Release` は D10 が直接扱う」と述べる。D10 は活性化が
  始まる時点の参照を初期値の行で与え、`Retain(v, π)` と `Release(v, π)` の行がその 2 種を扱う。
  D8 より参照は D10 の生成で作られ D10 の消費か `Release` で処分されるので、活性化が保持する参照を
  動かす事象は、D10 の初期値・D10 の生成・`Retain`・`Release`・D9 の消費・D9 の移動の 6 つで尽きる。
  `DEF 割り当て μ` の 6 行がこの 6 つである。

<1>1. D9 の移動の表の 6 行の移動先のスロットの変数は、`collect_bindings` が次の 6 通りの束縛を
      入れる変数である。
  BY <ref id=9d74736/>, <ref id=9c7c27a/>, CODE src/rc_ir/ownership.rs: collect_bindings
  `Let(x, Var(y), k)` の `x` は `Binding::Move(y)`、unbox 容器の `Destructure` の名前付きフィールドの
  変数は `Binding::Field(container, idx)`、unbox union の変位アームの payload は
  `Binding::Payload(scrut, Some(tag))`、catch-all アームの payload は `Binding::Payload(scrut, None)`、
  `Llvm` の素通し leaf の結果 `x` は `Binding::Llvm(gen, args, ty)`、`Match` のアーム本体の `Ret(x)` が
  値を渡す先の `Match` の束縛変数は `Binding::Join(arm_results)` である。

<1>1a. (h)。
  BY <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=596a46d/>, <ref id=d59f90b/>, <ref id=30d6238/>, <ref id=9d74736/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=8e3aff3/>, <ref id=ff5985d/>,
     CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: collect_bindings
  D33 の ρ-歩みの 1 歩は、いま居る位置 `(w, μ)` から、`w` の束縛が `μ` について持つ D20 の別名の辺の
  行き先 (D17) へ進む。
  **進む先の変数の値の得方は 2 通りである** -- D6 は「**値を得る形は 3 つあり、スロットが在るのは
  そのうち 2 つである。**」と述べ、節点が束縛する変数とパラメータ・capture がスロットを持ち、
  `vars.bindings` に束縛を持たない名前は記号の位置であってスロットではないと定める。(h) が主語に
  するのはスロットへ進む 1 歩なので、第 3 の形は落ちる。
  **進む先がパラメータ・capture であるときは、束縛の節点が無くても言明が出る** -- D23 より
  パラメータ・capture は活性化が始まる時点で値を得るので、`ρ` の上のどの節点の実行よりも前である。
  以下、進む先が節点の束縛する変数である場合を、束縛の種類で分ける。
  - `Binding::Move(y)`、`Binding::Field(container, idx)` の unbox の枝、`Binding::Payload(scrut, ・)` の
    辿る枝、`Binding::Llvm` の単一 `Arg(j, σ)` の枝では、`origin_inner` が `origin` を呼ぶ相手は
    それぞれ `y`・`container`・`scrut`・`args[j]` であり、どれも `w` を束縛する節点がその位置で
    名指す変数である。A11 よりその使用はその位置でスコープに入っている束縛に解決し、A6 より
    その名前を束縛するものはプログラム全体で 1 つだけなので、D2 のスコープの規則よりその束縛の節点は
    `ρ` の上でこの節点より前にある。D6 よりスロットはその変数が値を得た後に在るので、進む先の変数は
    `w` より前に値を得る。A6 と A11 を `insert_rc` の入力と出力について読めるのは A2 による。
  - `Binding::Join(arm_results)` の辺は、D17 よりその活性化が選んだアームの結果 `Ret(a)` の `a` へ
    進む。**この場合は上の議論が当たらない** -- D6 が「`a` は `k` の位置ではスコープに無い」と明記する
    とおり、`a` は `w` を束縛する節点の位置でスコープに入っていない。代わりに D3 を読む。`w` を束縛
    するのは `Let(w, Match(scrut, arms), k)` であり、D3 より実行路はアームを 1 つ選んでそのアーム本体を
    辿り、その後 `k` へ進むので、`a` を束縛する節点 -- アーム本体の中に在るか、その `Match` を囲む
    スコープに在る -- は `ρ` の上でこの `Match` 節点が `w` を束縛するより前にある。D6 よりスロットは
    その変数が値を得た後に在るので、`a` は `w` より前に値を得る。**`a` がパラメータ・capture である
    場合は上の節が覆う** -- そのとき `a` を束縛する節点は無いが、`a` は活性化が始まる時点で値を得る
    (D23)。

<1>2. これらの束縛のもとで、移動先のスロット `s'` の ρ-歩みの次の位置 (D6) は移動元の位置である。
      移動元の変数が局所名であるときそれはスロットであり、局所名でないときは記号の位置である。
  BY <1>1, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=d59f90b/>, CODE src/rc_ir/ownership.rs: origin_inner
  `Binding::Move(y)` の腕は `origin(y, path)` を返し、D9 の値の水準の行より `x` の値は `y` の値な
  ので移動元は `(y, λ)` である。`Binding::Field` の unbox の枝は `origin(container, [idx] ++ path)` を
  返し、フィールド変数の値は容器のそのフィールドなので移動元は `(container, [idx] ++ λ)` である。
  `Binding::Payload` の `None` の枝は `origin(scrut, path)` を返し、catch-all の payload の値は
  scrutinee の値そのものなので移動元は `(scrut, λ)` である。`Binding::Payload` の `Some(tag)` かつ
  scrutinee が unbox の枝は `origin(scrut, [tag] ++ path)` を返し、payload の値は scrutinee の活性変位の
  payload なので移動元は `(scrut, [tag] ++ λ)` である。`Binding::Llvm` の単一 `Arg(j, σ')` の枝は
  `origin(args[j], σ')` を返し、D17 の `Binding::Llvm` の行より対応先は `(args[j], σ')` であって、
  結果のその leaf の値はオペランド `j` のその leaf の値である。`Binding::Join` の腕については D17 が
  「`Binding::Join` の辺は、その活性化が選んだアームの結果へ辿る」と定め、移動元はそのアーム本体の
  `Ret(x)` の `(x, λ)` である。

<1>2a. (i)。
  BY <ref id=0594f24/>, <ref id=596a46d/>, <ref id=88a06de/>, <ref id=b6673ca/>, <ref id=3d4be43/>,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_box,
     CODE src/ast/types.rs: TypeNode::is_unbox, CODE src/ast/types.rs: TypeNode::is_closure,
     CODE src/ast/types.rs: TypeNode::is_array, CODE src/ast/types.rs: TypeNode::is_funptr,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_info,
     CODE src/ast/name.rs: FullName::from_strs, CODE src/ast/name.rs: FullName::new,
     CODE src/constants.rs: ARRAY_NAME, CODE src/constants.rs: ARROW_NAME,
     CODE src/constants.rs: FUNPTR_NAME, CODE src/constants.rs: STD_NAME,
     CODE src/fixstd/builtin.rs: is_funptr_tycon, CODE src/fixstd/builtin.rs: is_array_tycon,
     CODE src/fixstd/builtin.rs: make_array_tycon, CODE src/fixstd/builtin.rs: make_array_name,
     CODE src/fixstd/builtin.rs: make_arrow_name_abs, CODE src/fixstd/builtin.rs: bulitin_tycons
  `is_fully_unboxed` は `if self.is_funptr() { return true; }` に着いて真を返す。**先の 3 つの早期
  return が発火しないことは次のとおりである。** `is_funptr` は `toplevel_tycon_satisfies` に
  `is_funptr_tycon` を渡したものであり、`is_funptr_tycon` は tycon の名前の名前空間が `Std` の
  1 段であって名前が `FUNPTR_NAME` (`"#FunPtr"`) で始まることを問う。**`toplevel_tycon_satisfies` が
  述語に渡すのは `toplevel_tycon()` が返す tycon である**ので、その tycon の鍵はその形である。
  A28 より `E.tycons()` のその鍵の項目は `bulitin_tycons()` が置いたものであり、
  その `is_unbox` は真、`variant` は `Primitive` である。`is_unbox` は
  `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` であり、`toplevel_tycon_info` は
  同じ `toplevel_tycon()` の値で `E.tycons()` を引くのでその項目を返すから、第 2 項で真になる。`is_box` はその否定なので
  偽であり、第 1 の early return は発火しない。`is_closure` は `toplevel_tycon_satisfies` に
  「tycon の名前が `make_arrow_name_abs()` に等しい」を渡したものであり、`make_arrow_name_abs()` は
  `FullName::from_strs(&[STD_NAME], ARROW_NAME)` を作って `set_absolute` を呼ぶものであり、
  `FullName::from_strs(ns, name)` は `FullName::new(&NameSpace::from_strs(ns), name)` なので
  その `name` 欄は `ARROW_NAME` (`"Arrow"`) である。`"Arrow"` は `#FunPtr` で始まらないので偽である。
  よって第 2 の early return も
  発火しない。`is_array` は `toplevel_tycon_satisfies` に `is_array_tycon` を渡したものであり、
  `is_array_tycon(tc)` は `*tc == make_array_tycon()`、`make_array_tycon()` は
  `TyCon::new(make_array_name())`、`make_array_name()` は
  `FullName::from_strs(&[STD_NAME], ARRAY_NAME)` なのでその `name` 欄は `ARRAY_NAME` (`"Array"`) で
  あり、`#FunPtr` で始まる名前の tycon とは異なる。よって第 3 の early return も発火しない。
  よって D4 の第 1 規則よりその型は boxed leaf を持たない。D6 の記号の位置は `ty(g)` の inhabited な
  boxed leaf についてしか在らないので、funptr 型の名前は記号の位置を作らない。D6 より記号の位置が
  指すのは funptr かグローバル状態のオブジェクトなので、記号の位置が在るときそれはグローバル状態の
  オブジェクトである。A8 と D26 より、グローバル状態のオブジェクトが解放されることは無い。

<1>3. (a)。
  BY <1>2, <1>2a, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=88a06de/>, <ref id=30d6238/>
  移動元がスロット `s` であるとき、`s'` の ρ-歩みは 1 歩で `s` に着き、以後は `s` の ρ-歩みそのもの
  である。よって 2 つの ρ-終端は等しい。別名類は ρ-終端が等しいスロットの集まりなので (D33)、`s` と
  `s'` は同じ類に属する。
  移動元が記号の位置 `(g, λ)` であるとき、D6 より別名類の歩みは記号の位置で終わるので `s'` の ρ-終端は
  `(g, λ)` である。D9 の値の水準の行より移動先の値は移動元の値なので `s'` が指すオブジェクトは
  `obj(g, λ)` であり、<1>2a よりそれはグローバル状態のオブジェクトである。D26 より
  グローバル状態のオブジェクトは計数下ではないので、`s'` が属する類は計数下ではない。

<1>4. D34 の表は 6 行を持つ。最初の 3 行は `held_ρ(・, C)` に開始値 1 を
      与え、そのうちパラメータ・capture の inhabited (D16) な leaf については、その leaf の unit を
      関数が所有する (D14) か借用する (D14) かを問わず開始値 1 が与えられる。残る 3 行は
      `(v, λ) ∈ C` である `λ` を `π` の下に持つ `Retain(v, π)` につき `+1`、同じ形の `Release(v, π)`
      につき `-1`、`C` のスロットの D9 の消費につき `-1` である。移動の行は無い。
  BY <ref id=9d5d254/>

<1>4a. パラメータ・capture の leaf `(p, λ)` の ρ-終端は `(p, λ)` 自身である。したがって 1 つの別名類は
       パラメータ・capture の leaf を高々 1 つ含み、`β(C) = 1` であるのはその leaf の unit を関数が
       借用するときちょうどそのときである。
  BY <ref id=30d6238/>, <ref id=3f68b95/>, <ref id=d59f90b/>,
     CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: origin_inner
  `VarTable::of` は `func.params` と `func.capture` の各名前に `Binding::Param` を入れる。
  `origin_inner` の `None | Some(Binding::Param) | Some(Binding::Producer)` の腕は
  `Origin::Exactly((var.clone(), path.to_vec()))` を返すので、`(p, λ)` の ρ-歩みは 1 歩も進まずに
  `(p, λ)` で止まる。別名類は ρ-終端が等しいスロットの集まりであり、ρ-終端はスロットごとに 1 つに
  決まるので、パラメータ・capture の 2 つの leaf は別々の類に属する。`β` の定義はその類の ρ-終端に
  ついての条件なので、この 1 つの leaf の unit が決める。

<1>4b. (e)。
  BY <ref id=f06144e/>, <ref id=d59f90b/>, <ref id=e11772a/>, <ref id=30d6238/>, <ref id=0594f24/>,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
     EXT 空の列の第 1 元,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
     CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  D10 の生成の表は 5 行を持つ。`App(callee, args)` と `Closure(f, caps)` の結果は `collect_bindings` が
  `Binding::Producer` を入れ、`origin_inner` の `Producer` の腕は `here()` を返す。boxed 容器の
  `Destructure` の名前付きフィールドは `Binding::Field` の容器が boxed の枝、boxed union の変位アームの
  payload は `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝で、どちらも `here()` を返す。
  `Llvm` の結果のうち `result_prov` の宣言が単一の `Arg` でない leaf は `Binding::Llvm` の腕の
  `as_arg_projection` が `None` を返す枝に入る。A3 よりこのコミットの宣言は単一の `Fresh`・単一の
  `Unknown`・空集合のいずれかである。前 2 者では D17 の第 2 項より鎖がそこで止まる。空集合のときは
  `origin_from_leaves_under` が `reached` に元を 1 つも積まず、`EXT 空の列の第 1 元` より
  `reached.first()?` が `None` を返すので、`origin_inner` は `unwrap_or_else(here)` で自分自身を
  答える。**`reached` が空であることは
  `leaf_origins_under` が渡す宣言がその leaf 自身の 1 つだけであることによる** --
  `leaf_origins_under(path)` は `LeafMap::leaves_under(path)`、すなわち path が `path` で始まる leaf の
  宣言の列であり、D4 の判定は leaf を置く位置でその位置の下へ降りないので boxed leaf の path は互いに
  他の接頭にならず、`path` が leaf であるときその列は `path` 自身の 1 つである。その宣言が空集合なので
  `operand_units` は空、`produced_here` は偽である。いずれの行でも `origin` を
  呼ばずに自分自身を答えるので、ρ-歩みはその位置で終わり、そのスロットは自分が属する類の ρ-終端で
  ある。

<1>4c. (f)。
  BY <1>4b, EXT 空の列の第 1 元, <ref id=596a46d/>, <ref id=f06144e/>, <ref id=d59f90b/>, <ref id=e11772a/>, <ref id=30d6238/>, CODE src/rc_ir/ownership.rs: origin_inner,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  ρ-歩みが止まるのは `origin_inner` が `origin` を呼ばずに `here()` を答える腕であり、それは
  束縛の無い腕、`Binding::Param`、`Binding::Producer`、`Binding::Field` の容器が boxed の枝、
  `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝、`Binding::Llvm` の
  `as_arg_projection` が `None` を返す枝 (A3 よりこのコミットの宣言は単一の `Fresh`・単一の
  `Unknown`・空集合のいずれかである。前 2 者では D17 の第 2 項よりそこで止まり、空集合のときは
  <1>4b のとおり `origin_from_leaves_under` が `reached` に元を 1 つも積まずに `None` を返すので
  (`EXT 空の列の第 1 元`)、`origin_inner` が `unwrap_or_else(here)` で自分自身を答える) の 6 つで
  ある。束縛の無い腕が
  答える位置は D6 の記号の位置であってスロットではなく、D6 よりそれを終端とする類は計数下の類の
  範囲の外である。残る 5 つのうち `Binding::Param` はパラメータ・capture の leaf であり、
  `Producer` は `App` と `Closure` の結果、`Field` の boxed の枝は boxed 容器の `Destructure` の
  名前付きフィールド、`Payload` の `Some(_)` かつ boxed の枝は boxed union の変位アームの payload、
  `Llvm` の止まる枝は素通しでない `Llvm` の結果 leaf であって、この 4 つが D10 の生成の表の 5 行が
  参照を作る位置である。

<1>5. <1>4 の開始値を与える 3 行のうち、借用する (D14) unit の下のパラメータ・capture の leaf に
      ついての分を除いたものは、`DEF 割り当て μ` の第 1 行と第 2 行と同じ事象であり、両辺を同じだけ
      動かす。借用する unit の下のパラメータ・capture の leaf の開始値 1 には、`DEF 割り当て μ` に
      対応する行が無いので、左辺だけが 1 増える。
  BY <1>4, <1>4a, <1>4b, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ef8efc4/>, DEF 割り当て `μ`
  <1>4 の 3 行は `C` の ρ-終端についての条件で開始値を置く。<1>4a と <1>4b より、パラメータ・capture の
  leaf と D10 の生成が作るスロットはいずれも自分が属する類の ρ-終端なので、その位置に置かれる開始値は
  そのスロットを含む類のものである。よって `DEF 割り当て μ` の第 1 行・第 2 行が同じ位置で `μ` を
  `+1` するのと同じ事象である。
  `DEF 割り当て μ` の第 1 行は「**D10 の初期値**: 所有する各パラメータ・capture の inhabited な各 leaf
  `(p, λ)` について `+1`」であり、D10 の初期値も「所有する (D14) パラメータ・capture の unit の下の
  inhabited な各 leaf につき 1 つ。借用する unit の下の leaf は入れない」と述べる。よって所有する
  leaf については両辺が 1 ずつ増え、借用する leaf については左辺だけが 1 増える。D10 の生成の事象に
  ついては `DEF 割り当て μ` の第 2 行が同じ leaf について `+1` する。活性化が始まる時点の参照を D10 の
  初期値が、それ以後に作られる参照を D10 の生成の表が尽くすことは、<1>0 が示す。

<1>5a. `Retain`・`Release`・D9 の消費の 3 種は、`DEF 割り当て μ` の第 3・第 4・第 5 行が <1>4 の残る
       3 行と同じ leaf について同じ向きに 1 だけ動かす。
  BY <1>4, <ref id=ec8d1a0/>, <ref id=f06144e/>, DEF 割り当て `μ`
  どちらの量も D8 の意味の参照を `C` について数えたものである。`DEF 割り当て μ` の第 3 行は
  `Retain(v, π)` の `π` の下の inhabited な各 leaf について `+1`、第 4 行は `Release(v, π)` の同じ
  leaf について `-1`、第 5 行は D9 の消費される inhabited な各 leaf について `-1` であり、<1>4 の
  残る 3 行と同じ leaf を同じ向きに 1 だけ動かす。

<1>6. 計数下の別名類 `C` について、移動はどちらの辺も動かさない。
  BY <1>3, <1>4, <ref id=88a06de/>, DEF 割り当て `μ`
  <1>4 より `held_ρ(・, C)` に移動の行は無い。右辺について、`DEF 割り当て μ` の第 6 行は移動元の `μ` を
  `-1`、移動先の `μ` を `+1` する。移動元がスロットであるとき、<1>3 より 2 つのスロットは同じ類に
  属するので、両方が `C` に入るか両方が入らないかであり、前者では `-1` と `+1` が相殺し、後者では
  和に現れない。移動元が記号の位置であるときは `-1` を受ける項が無いが、<1>3 より移動先のスロットが
  属する類は計数下ではないので、計数下の `C` の和には `+1` も現れない。

<1>6a. 計数下の別名類 `C` について、D34 が `held_ρ(・, C)` を定め始める時点 -- `C` の ρ-終端の変数が
       値を得る時点 -- において、値を得ている `C` のスロットは ρ-終端 1 つだけであり、そこで
       `held_ρ(・, C) = 1 = Σ_{s ∈ C} μ(s) + β(C)` である。
  BY <1>1a, <1>4, <1>4a, <1>4c, <ref id=b3dfa37/>, <ref id=596a46d/>, <ref id=ef8efc4/>, <ref id=30d6238/>, <ref id=9d5d254/>, DEF 割り当て `μ`, 言明の `β` の定義
  <1>1a より ρ-歩みの 1 歩が進む先のスロットの変数は、いま居る位置の変数より前に値を得る。
  よって `C` のスロットのうち ρ-終端でないものの変数は、ρ-終端の変数より後に値を得る
  (ρ-終端は自分の歩みの最後の位置であり、その歩みの各段が変数を前へ辿る)。
  D6 よりスロットはその変数が値を得た後に在り、`DEF 割り当て μ` の 6 種はいずれもその位置のスロットに
  ついて値を動かすので、まだ値を得ていないスロットの `μ` は 0 である。
  <1>4c より計数下の類の ρ-終端はパラメータ・capture の leaf か D10 の生成の表の位置のいずれかで
  あり、D34 の「開始の時点」の段落よりそのどちらでも `held` はこの時点に 1 から始まる (<1>4)。
  右辺は 3 つの場合に分かれる。ρ-終端が**所有する** (D14) パラメータ・capture の leaf であるとき、
  `DEF 割り当て μ` の第 1 行がその leaf の `μ` を 1 にし、<1>4a より `β(C) = 0` なので和は 1 である。
  **借用する** leaf であるとき、`DEF 割り当て μ` の第 1 行は所有する leaf にしか掛からないので `μ = 0`、
  <1>4a より `β(C) = 1` で和は 1 である。D10 の生成の位置であるとき、`DEF 割り当て μ` の第 2 行が
  `μ` を 1 にし、その位置はパラメータ・capture の leaf ではないので `β` の定義より `β(C) = 0` で
  あり、和は 1 である。

<1>7. QED
  BY <1>0, <1>1a, <1>2a, <1>3, <1>4, <1>4a, <1>4b, <1>4c, <1>5, <1>5a, <1>6, <1>6a, <ref id=a502f3e/>, <ref id=ef8efc4/>, <ref id=9d5d254/>, DEF 割り当て `μ`
  (a) は <1>3 である。(b) について、`C` を計数下の別名類とする。<1>6a が、D34 が
  `held_ρ(・, C)` を定め始める時点で等式が成り立つことを与える。その時点より後について、
  右辺を動かす事象は <1>0 より `DEF 割り当て μ` の 6 種で尽き、左辺を動かす事象は D34 の表の 6 行で
  尽きる。<1>5 より両者の差は「借用する unit の下のパラメータ・capture の leaf の
  開始値」の 1 種だけであり、<1>5・<1>5a・<1>6 が残る各事象について 2 つの増減の一致を与える。
  <1>4a よりその 1 種は `β(C) = 1` である類についてちょうど 1 回、`β(C) = 0` である
  類については 1 回も起きず、しかも <1>6a が基底に取った時点で既に起きている。よってその時点より後の
  事象の個数についての帰納で (b) の等式が成り立つ。
  (c) は `β` の定義である -- すべての unit が所有されるとき、どの類の ρ-終端も借用する unit の下の
  leaf ではないので `β(C) = 0` である。グローバル初期化子の `init` は D1 よりパラメータも capture も
  持たないので、その本体の類の ρ-終端はパラメータ・capture の leaf ではない。(d) は <1>4a、(e) は
  <1>4b、(f) は <1>4c、(g) は <1>0、(h) は <1>1a、(i) は <1>2a である。

### 10.2 `L14` (`live_before` は自由変数と `live_after` の和である) <!--#66e786e-->

**言明**。`insert_rc` が骨格節点 `m` を `live_after = A(m)` の下で書き換えるとき、次の 2 つが成り立つ。

- **(a)** `Λ(m) = free_locals(m) ∪ A(m)` である (`CODE src/rc_ir/rc_insert.rs: free_locals`)。
- **(b)** `A(m)` の各名前は、`m` の部分木の外にある位置で使われる名前である。したがって `m` の部分木の
  中で束縛される名前は `A(m)` に入らない。
- **(c)** 骨格節点を書き換える `insert_into_expr` の呼び出しは、本体の根についてのものか、ある骨格
  節点 `m_0` の継続についてのものか、ある `Let(x, Match(scrut, arms), cont)` のアーム本体についての
  もののいずれかである。

**証明**

<1>1. `free_locals(m)` は、`m` を根とする部分木が参照する局所名から、その部分木が束縛する局所名を
      除いたものである。
  BY CODE src/rc_ir/rc_insert.rs: free_locals,
     CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound,
     CODE src/rc_ir/rc_insert.rs: insert_if_local, EXT 述語による除去
  `free_locals` は `collect_referenced_and_bound` で `refs` と `bound` を集め、
  `refs.retain(|n| !bound.contains(n))` を掛けて `refs` を返す。`EXT 述語による除去` より、この
  呼び出しは `bound` に入る元をすべて取り除き、入らない元をすべて残す。
  `collect_referenced_and_bound` は `Ret` の変数、`Let` の右辺の各変数 (`Match` の
  scrutinee を含む)、`Destructure` の容器、`Retain`/`Release`/`Eval` の変数を `refs` に入れ、`Let` の
  束縛変数、`Match` の各アームの payload、`Destructure` の各フィールド変数を `bound` に入れ、継続と
  アーム本体へ降りる。局所名の判定は `insert_if_local` が行う。

<1>2. `m` が本体の根であるとき `A(m) = ∅` である。
  BY EXT 空を作る `Default`, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: insert_rc
  関数は `insert_into_expr(func.body, &Set::default())`、グローバル初期化子は
  `inserter.insert_into_expr(glob.init, &Set::default())` で呼ばれる。`EXT 空を作る Default` より
  `Set::default()` は空の集合である。

<1>3. `A(m)` の各名前は、`m` の部分木の外にある位置で使われる名前である。したがって `m` の部分木の中で
      束縛される名前は `A(m)` に入らない。
  <2>1. 骨格節点を書き換える `insert_into_expr` の呼び出しは、本体の根についてのものか、ある骨格節点
        `m_0` の継続についてのものか、ある `Let(x, Match(scrut, arms), cont)` のアーム本体についての
        ものかの 3 通りである。
    BY <ref id=b3dfa37/>, EXT クレートの項目, EXT ビルドの対象, EXT 非公開の項目の可視範囲,
       CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    **呼び出し元の集合は可視性で閉じる。** `insert_into_expr` は `RcInserter` の非公開のメソッドで
    あり、`RcInserter` 自身も `src/rc_ir/rc_insert.rs` の非公開の項目なので、
    `EXT 非公開の項目の可視範囲` よりそれを呼ぶ式はそのモジュールとその子孫の中にしかない。
    `EXT クレートの項目` よりそのモジュールの項目はこのファイルに書かれたものだけであり、
    `EXT ビルドの対象` より走査の範囲は `src/` の全ファイルである。よってファイルの全文を読んで得た
    呼び出し元の一覧は完全であり、`insert_into_expr` を呼ぶのは `insert_rc` と `insert_into_func`
    (根) と、`insert_into_operation_let`・`insert_into_destructure`・`insert_into_eval`・
    `insert_into_match` の継続についての呼び出しと、`insert_into_match` のアーム本体についての
    呼び出しである。
  <2>2. `insert_into_expr(n, L)` が返す集合の各名前は、`n` の部分木の中で使われる名前か `L` の元で
        ある。
    BY <ref id=3e6b0e0/>, <ref id=d80dde9/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: insert_if_local
    `n` の部分木の節点数についての帰納。`RcExpr::Ret(x)` の腕は `L` に `x` を足したものを返す。残る
    4 種の腕はいずれも、継続 (と `Match` ではアーム本体) についての呼び出しが返した集合から名前を
    取り除き、`n` 自身が名指す名前 -- `rhs_operands` が挙げるオペランド、`Destructure` の容器、
    `Eval` の変数、`Match` の scrutinee と各アームの `arm_free_locals` -- を足したものを返す。
    足される名前はいずれも `n` の部分木の中で使われる。A25 より骨格は `Retain`/`Release` を含まないので
    その腕は通らない。
  <2>3. 根についての呼び出しでは前半が成り立つ。
    BY <1>2
    `A(根) = ∅` なので量化する名前が無い。
  <2>4. `m` が `m_0` の継続であるとき、`A(m_0)` について前半が成り立つならば `A(m)` についても
        成り立つ。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    4 つの関数はいずれも `self.insert_into_expr(cont, live_after)` を呼ぶので `A(m) = A(m_0)` である。
    `m` の部分木は `m_0` の部分木に含まれるので、`m_0` の部分木の外にある位置は `m` の部分木の外にも
    ある。
  <2>5. `m` が `m_0 = Let(x, Match(scrut, arms), cont)` のアーム本体であるとき、`A(m_0)` について
        前半が成り立つならば `A(m)` についても成り立つ。
    BY <2>2, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    `insert_into_match` は `live_cont = insert_into_expr(cont, live_after)` を取り、
    `live_after_match = live_cont \ {x}` の下でアーム本体を書き換えるので `A(m) ⊆ live_cont` である。
    <2>2 より `live_cont` の各名前は `cont` の部分木の中で使われるか `A(m_0)` の元である。`cont` の
    部分木はアーム本体の部分木と交わらないので前者は `m` の部分木の外にあり、後者は仮定より
    `m_0` の部分木の外、したがって `m` の部分木の外にある位置で使われる。
  <2>6. QED
    BY <ref id=8e3aff3/>, <2>1, <2>3, <2>4, <2>5, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, EXT 呼び出しの入れ子
    <2>1 の 3 通りについて、`insert_into_expr` の呼び出しの入れ子の深さについての帰納で前半が
    成り立つ -- 基底は <2>3、段は <2>4 と <2>5 である。`EXT 呼び出しの入れ子` より入れ子は木をなし、
    <2>1 の後ろ 2 通りはどちらも `m_0` の真の部分木についての呼び出しなので、D2 (骨格は有限の木) より
    深さは有限である。後半は前半から出る。`m` の部分木の中で
    束縛される名前 `y` を `m` の部分木の外の位置が使うとすると、A11 よりその使用はその位置で
    スコープに入っている束縛に解決するが、A6 より `y` を束縛するのはその 1 つだけであり、その
    スコープ (D2) は `m` の部分木の中に収まる -- 矛盾である。よって `y` は前半の意味で使われず、
    `A(m)` に入らない。

<1>4. ASSUME NEW 骨格節点 `m`、および (帰納法の仮定) `m` の真の部分木である各骨格節点について言明が
              成り立つこと
      PROVE  `m` について言明が成り立つ
  <2>1. CASE `m = Ret(x)`。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この腕は `live = live_after.clone()` に `insert_if_local(&mut live, &x.name)` で `x` を足したものを
    返す。<1>1 より `free_locals(Ret(x))` は `x` が局所名なら `{x}`、そうでなければ空である。
  <2>2. CASE `m = Let(x, rhs, cont)` で `rhs` が `Match` でない。
    BY <ref id=8e3aff3/>, 帰納法の仮定, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: insert_if_local,
       <1>1, <1>3
    この関数は `insert_into_expr(cont, live_after)` を呼び、返った `live_cont` から `x` を除き、
    `rhs_operands(rhs)` の各オペランドの局所名を足したものを返す。帰納法の仮定より
    `live_cont = free_locals(cont) ∪ A(m)`。`rhs_operands` が挙げるオペランドは `rhs` が参照する変数の
    すべてである -- `Var` は被移動変数、`App` は callee と全引数、`Closure` は全 capture、`Llvm` は
    全オペランドを挙げ、`Match` はこの関数に来ない。よって `Λ(m)` は
    `((free_locals(cont) ∪ A(m)) \ {x}) ∪ ops` であり、<1>3 より `x ∉ A(m)` なので
    `(free_locals(cont) \ {x}) ∪ ops ∪ A(m)` に等しい。<1>1 より `free_locals(m)` は `m` の部分木が
    参照する局所名 `refs = ops ∪ refs(cont)` から、束縛する局所名 `bound = {x} ∪ bound(cont)` を
    落としたものである。A11 より `ops` の各使用はその位置でスコープに入っている束縛に解決し、A6 より
    その名前を束縛するものはプログラム全体で 1 つなので、D2 のスコープの規則より `ops` は `bound` と
    交わらない。よって `free_locals(m) = ops ∪ (free_locals(cont) \ {x})` であり、上の式の前 2 項が
    それである。
  <2>3. CASE `m = Destructure(container, fields, _, cont)`。
    BY <ref id=8e3aff3/>, 帰納法の仮定, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>3
    この関数は `live_cont` から各フィールド変数を除き、`container` の局所名を足したものを返す。
    フィールド変数は <1>3 より `A(m)` に入らない。<2>2 と同じく A6・A11・D2 のスコープの規則より
    `container` は `m` の部分木が束縛する名前ではないので、<1>1 の `free_locals(m)` は
    `free_locals(cont)` からフィールド変数を落として `container` を足したものである。
  <2>4. CASE `m = Eval(x, cont)`。
    BY <ref id=8e3aff3/>, 帰納法の仮定, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1
    この関数は `live_cont` に `x` の局所名を足したものを返す。`Eval` は名前を束縛しないので `m` の
    部分木が束縛する名前は `cont` の部分木が束縛するものであり、<2>2 と同じく A6・A11・D2 のスコープの
    規則より `x` はそこに入らない。よって <1>1 の `free_locals(m)` は `free_locals(cont)` に `x` を
    足したものである。
  <2>5. CASE `m = Let(x, Match(scrut, arms), cont)`。
    BY 帰納法の仮定, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, <1>1, <1>3, <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=1172c08/>, <ref id=3905b4e/>, <ref id=b3dfa37/>
    この関数は `live_cont = insert_into_expr(cont, live_after)` を取り、
    `live_after_match = live_cont \ {x}` の下で各アーム本体を書き換え、返った `body_live` をすべて
    `live_before_arms` に集めて各アームの payload を除き、最後に `x` を除いて `scrut` を足す。
    帰納法の仮定より `live_cont = free_locals(cont) ∪ A(m)` であり、
    `body_live_j = free_locals(arm_j.body) ∪ live_after_match` である。payload は <1>3 より
    `live_after_match` に入らないので、`live_before_arms` は
    `(∪_j (free_locals(arm_j.body) \ {payload_j})) ∪ live_after_match` である。
    **`live_after_match` の分が入るのは、アームが 1 つ以上あるからである** -- `live_before_arms` は
    `Set::default()` -- `EXT 空を作る Default` より空の集合 -- から始まり、各アームが返した
    `body_live` からしか名前を受け取らないので、アームが 0 個ならこの和は空になる。A9 は「`borrow_ify` の入力プログラムのすべての `Match` は 1 つ以上の
    アームを持つ」と述べ、A9 自身が「**`insert_rc` の入力と出力について読む段は A2 を引く。**」と
    書き、A2 が「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の入力と出力に
    ついても読める。**」を与えるので、`insert_rc` の入力の骨格の各 `Match` はアームを 1 つ以上持つ。
    A6 と A11 より、ある
    アームが束縛する名前を別のアームや `cont` が参照することはないので、この和は
    `collect_referenced_and_bound` がアームについて集める `refs \ bound` と一致する。
    最後の `x` の除去はこの和のうち `live_after_match` の分にしか掛からない --
    `x ∉ ∪_j free_locals(arm_j.body)` だからである。A11 よりアーム本体の中の `x` という名前の使用は
    その位置でスコープに入っている束縛に解決し、D2 のスコープの規則より `Let(x, Match(..), cont)` が
    束縛する `x` のスコープは `cont` の部分木であってアーム本体を含まず、A6 より `x` という名前を
    束縛するものはプログラム全体で 1 つだけなので、アーム本体は `x` を参照しない。よって `Λ(m)` は
    `(∪_j (free_locals(arm_j.body) \ {payload_j})) ∪ (free_locals(cont) \ {x}) ∪ {scrut} ∪ A(m)` で
    ある (`x ∉ A(m)` は <1>3)。
    **`scrut` が `m` の部分木の束縛する名前でないことが要る** -- <1>1 の `free_locals(m)` は `refs`
    から `m` の部分木が束縛する名前 `bound` を落とすので、`scrut` が `bound` に入ればこの項が消える。
    A11 より `scrut` の使用はその位置でスコープに入っている束縛に解決し、A6 よりその名前を束縛する
    ものはプログラム全体で 1 つだけである。D2 のスコープの規則より、`m` の部分木の束縛 -- `x`、各
    アームの payload、およびそれより深い節点の束縛 -- のスコープはどれも `cont` の部分木かアーム
    本体の部分木に収まり、scrutinee の位置を含まない。よって `scrut ∉ bound` である
    (兄弟の <2>2 と <2>3 が `ops`・`container` について置くのと同じ節である)。
    <1>1 よりこれは `free_locals(m) ∪ A(m)` である。
  <2>6. CASE `m` が `Retain` または `Release` である。
    BY <ref id=d80dde9/>
    A25 より骨格はこの 2 種を含まないので、この場合は起きない。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <ref id=b3dfa37/>, <ref id=3e6b0e0/>
    場合が尽きることは `RcExpr` が 6 個の構成子を持つこと (D2) と、`insert_into_expr_inner` の
    `match` がその 6 個をこの 6 つの腕で覆うことによる。A15 より `insert_into_expr` は
    `insert_into_expr_inner` をちょうど 1 回呼ぶ。<2>2 から <2>5 の各場合で帰納法の仮定を当てるのは
    `cont` とアーム本体 -- どれも `m` の真の部分木 -- についてだけであり、<2>1 はそれを使わない。

<1>5. QED
  BY <1>3, <1>4, <ref id=b3dfa37/>
  (a) について、D2 より骨格は有限の木なので、部分木の節点数についての整礎帰納が使える。<1>4 がその段で
  あり、それをすべての骨格節点に当てると (a) が出る。(b) は <1>3 であり、(c) は <1>3 の <2>1 である
  -- <2>1 は `<1>3` の証明の内側に在るので、その結論をここで (c) として言明に括り出す。

### 10.3 `L15` (`Λ` と `insert_rc` の出力についての 5 つの性質) <!--#dca1c02-->

**言明**。`insert_rc` の出力の 1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定し、
その本体の骨格 (`DEF 骨格`) の節点 `m` を取る。次の 5 つが成り立つ。(c)・(d)・(e) は `ρ` と活性化に
依らない。

- **(a)** `Λ(m)` の各名前を束縛するのは、`m` の真の祖先の骨格節点か、その本体のパラメータ・capture
  である。したがってその各名前は局所名であり、`ρ` の上で `m` の検査点に至るまでに値を得ている。
- **(b)** `m` の核節点が名指す局所変数は、すべて `Λ(m)` に入る。「名指す」とは、`Ret(x)` の `x`、
  `Let` の右辺の各変数 (`Match` の scrutinee を含む)、`Destructure` の容器、`Eval` の変数を指す。
- **(c)** `needs_rc(v)` が偽の変数 `v` はスロットを持たない。
- **(d)** 局所でない名前は `vars.bindings` に束縛を持たない。したがって局所でない名前 `g` と `ty(g)` の
  inhabited な boxed leaf `λ` の対は、D6 の記号の位置であってスロットではない。
- **(e)** `insert_rc` の出力のすべての関数の `borrowed_units` は空であり、D14 よりそのすべての
  パラメータ・capture の unit はその関数が所有する。

**証明**

<1>1. (a)。
  <2>1. `free_locals(m)` の各名前を束縛するのは、`m` の真の祖先の骨格節点か、その本体のパラメータ・
        capture である。
    BY <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>,
       CODE src/rc_ir/rc_insert.rs: free_locals,
       CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound
    `free_locals(m)` は `m` の部分木が参照する局所名から、その部分木が束縛する局所名を落としたもので
    ある。その各名前 `n` は `m` の部分木の中のある位置で使われる。A11 よりその使用はその位置で
    スコープに入っている束縛に解決し、A6 よりその名前を束縛するものはプログラム全体で 1 つだけである。
    D2 のスコープの規則より、その束縛は `Let`/`Destructure` の束縛 (スコープは継続の部分木)、`Match` の
    アームの payload (スコープはそのアーム本体の部分木)、パラメータ・capture (スコープは本体の全体) の
    いずれかである。前 2 者のスコープが `m` の部分木の中の位置を含むのは、その束縛の節点が `m` の
    祖先であるか `m` の部分木の中に在るときに限る。`n` は `m` の部分木が束縛する名前ではないので、
    残るのは `m` の真の祖先か、パラメータ・capture である。A6 と A11 を `insert_rc` の入力と出力に
    ついて読めるのは A2 による。
  <2>2. `A(m)` の各名前を束縛するのは、`m` の真の祖先の骨格節点か、その本体のパラメータ・capture で
        ある。
    BY <2>1, <ref id=66e786e/>, <ref id=b3dfa37/>, EXT 呼び出しの入れ子,
       CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
    骨格節点を書き換える `insert_into_expr` の呼び出しは `L14` (c) の 3 通りである。その呼び出しの
    入れ子の深さについての帰納
    (`EXT 呼び出しの入れ子` より入れ子は木をなし、D2 より骨格は有限の木なので深さは有限である)。
    本体の根では `A(根) = ∅` なので量化する名前が無い。`m` が骨格節点 `m_0` の継続であるとき、
    4 つの関数はいずれも `self.insert_into_expr(cont, live_after)` を呼ぶので `A(m) = A(m_0)` であり、
    `m_0` の真の祖先は `m` の真の祖先である。`m` が `m_0 = Let(x, Match(scrut, arms), cont)` の
    アーム本体であるとき、`insert_into_match` は `A(m) = live_cont \ {x}` の下でアーム本体を書き換え、
    `L14` (a) より `live_cont = free_locals(cont) ∪ A(m_0)` である。<2>1 を `cont` に当てると
    `free_locals(cont)` の各名前を束縛するのは `cont` の真の祖先 -- `m_0` かその祖先 -- か
    パラメータ・capture であり、`m_0` が束縛するのは `x` だけなのでそれを除くと `m_0` の真の祖先か
    パラメータ・capture が残る。`A(m_0)` の側は帰納法の仮定である。`m_0` の真の祖先も `m_0` 自身も
    `m` の真の祖先である。
  <2>3. QED
    BY <2>1, <2>2, <ref id=8e3aff3/>, <ref id=66e786e/>, <ref id=ca36627/>, <ref id=596a46d/>, <ref id=b3dfa37/>,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: collect_bindings
    `L14` (a) より `Λ(m) = free_locals(m) ∪ A(m)` なので、<2>1 と <2>2 よりその各名前を束縛するのは
    `m` の真の祖先の骨格節点かパラメータ・capture である。D3 より `ρ` は本体の根から `m` まで祖先を
    順に辿るので、`m` の真の祖先の節点は `m` の検査点より前に実行される。D6 より節点が束縛する変数は
    その節点を実行する段で値を得、パラメータ・capture は活性化が始まる時点で値を得る。よって
    `Λ(m)` の各名前は `m` の検査点に至るまでに値を得ている。
    局所名であることは D6 から出る -- `VarTable::of` はパラメータと capture の名前を、
    `collect_bindings` は節点が束縛する変数の名前を `vars.bindings` に入れ、D6 は
    「**逆に、`vars.bindings` に束縛を持つ名前は局所名である。**」と述べる。

<1>2. (b)。
  BY <ref id=8e3aff3/>, <ref id=66e786e/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: free_locals,
     CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound
  核節点が名指す変数は、`collect_referenced_and_bound` が `refs` に入れる名前のうち、`m` 自身の
  節点が挙げるものである。`free_locals(m)` は `refs` から `m` の**部分木**が束縛する名前 `bound` を
  落とすので、名指す名前が `bound` に入らないことが要る。A11 より核節点の変数の使用はその位置で
  スコープに入っている束縛に解決し、A6 より同じ名前を束縛するものはプログラム全体で 1 つだけである。
  D2 のスコープの規則より、`m` 自身の束縛 (`Let` の `x`、`Destructure` のフィールド変数、`Match` の
  payload) のスコープは `m` の継続かアーム本体の部分木であり、`m` より深い節点の束縛のスコープは
  さらにその中である。どれも `m` の核節点の位置を含まないので、名指す名前は `bound` に入らない。
  よって `free_locals(m)` に入り、`L14` (a) より `Λ(m)` に入る。

<1>3. (c)。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc, <ref id=0594f24/>, <ref id=596a46d/>
  `needs_rc(v)` は `!v.ty.is_fully_unboxed(type_env)` であり、D4 の第 1 規則より `is_fully_unboxed` が
  真の型は boxed leaf を持たない。D6 よりスロットは boxed leaf についてのみ在る。

<1>4. (d)。
  BY <ref id=8e3aff3/>, <ref id=cb35ab1/>, <ref id=b3dfa37/>, <ref id=596a46d/>, <ref id=3e6b0e0/>,
     前提 `VarTable::bindings` へ鍵を入れる式の在りか,
     EXT クレートの項目, EXT ビルドの対象, EXT 条件つきコンパイル,
     CODE src/rc_ir/rename.rs: rename_expr_inner,
     CODE src/rc_ir/rename.rs: assign_fresh_names_to_binders_inner,
     CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: VarTable::body_only,
     CODE src/rc_ir/lower.rs: Lowerer::fresh_var, CODE src/rc_ir/lower.rs:
     Lowerer::lower_lambda_as_function, CODE src/ast/name.rs: FullName::is_local,
     CODE src/ast/name.rs: NameSpace::is_local, CODE src/ast/name.rs: FullName::local,
     CODE src/ast/name.rs: NameSpace::local, CODE src/ast/name.rs: FullName::new,
     CODE src/build/build_object_files.rs: lower_and_insert_rc,
     CODE src/rc_ir/simplify.rs: simplify, CODE src/rc_ir/simplify.rs: case_of_known_union,
     CODE src/rc_ir/simplify.rs: destructure_of_struct, CODE src/rc_ir/simplify.rs: case_of_case,
     CODE src/rc_ir/simplify.rs: single_subst, CODE src/rc_ir/rename.rs: clone_fresh,
     CODE src/rc_ir/rename.rs: assign_fresh_name,
     CODE src/rc_ir/rename.rs: assign_fresh_names_to_binders,
     CODE src/rc_ir/rename.rs: substitute_expr,
     CODE src/rc_ir/rename.rs: rename_expr, CODE src/rc_ir/rename.rs: rename_var
  `vars.bindings` に鍵を入れる式の在りかは `前提 VarTable::bindings へ鍵を入れる式の在りか` が
  与える -- `VarTable::of` が入れる関数のパラメータと capture の名前と、`collect_bindings` が入れる
  2 つの式 (`Let`・`Destructure` の束縛変数と、`Match` のアームの payload の名前) である
  (グローバル初期化子については `VarTable::body_only` が後者だけを入れる)。走査が挙げる残る 1 つは
  `Interpreter` の同名の欄であって `VarTable` の欄ではない。D6 は
  「**逆に、`vars.bindings` に束縛を持つ名前は局所名である。**」と述べ、その道を
  「`VarTable::of` と `VarTable::body_only` がその表に入れる鍵は、パラメータ・capture の名前と節点が
  束縛する変数の名前だけで、どれも `Lowerer::fresh_var` が `FullName::local` で作ったものである」と
  書く。`FullName::local(name)` は `FullName::new(&NameSpace::local(), name)` であり、
  `NameSpace::local()` は `names` に空の列を置くので、その名前の名前空間は空である。
  `FullName::is_local` の本体は `self.namespace.is_local()`、`NameSpace::is_local` の本体は
  `self.names.len() == 0` なので、束縛を持つ名前は局所名である。対偶より局所でない名前は `vars.bindings` に束縛を持たない。
  **この本体は lowering の出力そのものではない。** `lower_and_insert_rc` は `lower_program` の後に
  `simplify` を掛けてから `insert_rc` を呼ぶので、束縛の位置に名前を書く式は lowering の外にもある。
  D6 はその 2 人を「その 2 人は `Lowerer::fresh_var` と `clone_fresh` であり、後者は `simplify` が
  束縛の位置に名前を書く道である。」と名指し、A13 の果たす者もこの 2 人である。
  `Lowerer::fresh_var` (パラメータと capture は `lower_lambda_as_function` が、残りは各 `lower_*` が
  呼ぶ) は `FullName::local` で名前を作るので名前空間は空である。`clone_fresh` は
  `assign_fresh_names_to_binders` を通じて各束縛変数に `assign_fresh_name` を当てる --
  A15 より `assign_fresh_names_to_binders` は `grow_stack` の閉包
  `assign_fresh_names_to_binders_inner` をちょうど 1 回呼び、その本体が `Let` の束縛変数・
  `Destructure` の各フィールド変数・`Match` の各アームの payload に `assign_fresh_name` を当てて
  継続とアーム本体へ降りる。`assign_fresh_name` は `name.clone()` の `name` 欄だけを
  `format!("{}#{}{}", ...)` で書き替えるので、名前空間の欄は元の名前のものである。よってどちらの
  作る名前も、元が局所名なら局所名である。
  **`simplify` のもう 1 つの書き換え `substitute_expr` は、束縛の位置の名前を替えない。**
  `substitute_expr` は `rename_expr` を呼び、A15 より `rename_expr` は `grow_stack` の閉包
  `rename_expr_inner` をちょうど 1 回呼ぶ。`rename_expr_inner` は `Let` の `x`、`Destructure` の
  フィールド変数、`Match` の payload にも `rename_var` を当てるが、`rename_var` が名前を替えるのは
  その名前が写像の鍵であるときだけである。`EXT クレートの項目` より
  `src/rc_ir/simplify.rs` の項目はそのファイルに書かれたものだけなので、その呼び出しの一覧は完全で
  あり、4 つある。**そのうち 3 つは `single_subst` を通るので定義域は 1 つの名前である** --
  `case_of_known_union` のアームの payload と `Match` の束縛変数、case-of-case の外側のアームの
  payload である。**残る 1 つ、`destructure_of_struct` の定義域は 1 つではない** --
  `for (idx, fv) in fields { subst.insert(fv.name.clone(), args[*idx].name.clone()); }` が、
  取り除く `Destructure` が名指す**各**フィールド変数を鍵に入れる。
  **4 つのどれでも、鍵はその書き換えが取り除く節点が束縛する名前であり、写す先はその束縛のスコープ
  (D2) の中の部分式である。** 写す先がその名前を改めて束縛しないことは、2 人の作り手から出る --
  `Lowerer::fresh_var` は `self.fresh_counter` を 1 つ進めてから
  `FullName::local(&format!("{}#{}{}", hint, self.symbol_tag, self.fresh_counter))` を作るので、
  lowering の出力の束縛名は互いに相異なる。case-of-case の写す先
  `moved = clone_fresh(&outer.body, PASS_TAG, counter)` の束縛名は、`fresh_var` ではなく
  `assign_fresh_name` が `counter` を 1 つ進めてから作ったものである。**その名前が写す先に既に在る
  どの名前とも異なることを置くのは A13 である** -- A13 は「**`clone_fresh` が作る名前は、それを
  写す本体に既に在るどの名前とも異なる。**」と述べる。
  よって `substitute_expr` は束縛の位置の名前を替えない。以上より `is_local` は
  この段を渡って保たれる。`insert_rc` 自身が束縛を作らないことは A2 が述べる。
  D6 の「**値を得る形は 3 つあり、スロットが在るのはそのうち 2 つである。**」の 3 つ目が
  この名前であり、D6 よりその対は記号の位置であってスロットではない。

<1>5. (e)。
  BY <ref id=ef8efc4/>, 前提 `RcFunc::borrowed_units` の在りか, 前提 `finish_clone` の呼び出し元,
     EXT クレートの項目, EXT ビルドの対象, EXT 条件つきコンパイル,
     CODE src/rc_ir/lower.rs: Lowerer::lower_lambda_as_function,
     CODE src/rc_ir/simplify.rs: simplify, CODE src/rc_ir/rc_insert.rs: insert_rc,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/build/build_object_files.rs: lower_and_insert_rc,
     CODE src/rc_ir/borrow.rs: borrow_ify, CODE src/rc_ir/borrow.rs: clone_func,
     CODE src/rc_ir/specialization.rs: CloneRegistry::finish_clone,
     CODE src/rc_ir/unique_check_elim.rs: Specializer::materialize_clone,
     CODE src/rc_ir/locality.rs: specialize,
     CODE src/build/build_object_files.rs: optimize_rc_program
  `RcFunc` の `borrowed_units` の欄に値を書く式は次の 8 つである。**在りかは 2 つで決める** --
  この欄の字面を持つ項目は `前提 RcFunc::borrowed_units の在りか` が挙げ、そのうち書き込みは
  `--` の後に書いてある。字面を持たずにこの欄を運ぶのは `..` の構造体更新の式であり、その式は
  `RcFunc` のリテラルの中にしかないので、`EXT ビルドの対象` と `EXT クレートの項目` より
  `src/` の全ファイルの全文を読んで得た下の一覧は完全である。

  1. `Lowerer::lower_lambda_as_function` が `RcFunc` を組むときに置く `Set::default()`。
  2. `borrow_ify` の末尾の `func.borrowed_units = param_capture_units(func, type_env)…`。
  3. `clone_func` が複製に置く `Set::default()`。
  4. `CloneRegistry::finish_clone` の `name == func.name` の枝が返す `RcFunc { body, ..func.clone() }`
     -- この欄には `func` の値がそのまま入る。
  5. 同じ関数の残る枝が、複製の鍵を改名して組む式。
  6. `#[cfg(test)]` の下の `src/rc_ir/validate.rs` のテスト用の `RcFunc` が置く `Set::default()`。
  7. `#[cfg(test)]` の下の `src/rc_ir/dead_code_elim.rs` のテスト用の `RcFunc` が置く
     `Set::default()`。
  8. `#[cfg(test)]` の下の `src/rc_ir/dead_code_elim.rs` のテスト用の
     `RcFunc { body, ..func(main.clone(), &[]) }` -- `..` の構造体更新でこの欄を運ぶ式である。

  6・7・8 は `EXT 条件つきコンパイル` より `fix` の実行可能ファイルを作るビルドに入らない。
  2 と 3 は `borrow_ify` の中に在る。4 と 5 は `CloneRegistry::finish_clone` の中に在り、
  **`finish_clone` を呼ぶ式は 2 つである** (`前提 finish_clone の呼び出し元` -- 走査が挙げる 3 つの
  うち 1 つはその関数自身の宣言であり、残る 2 つが `unique_check_elim::Specializer::materialize_clone`
  と `locality::specialize` の呼び出しである)。
  `optimize_rc_program` はこの 2 つのパスを `borrow_ify` より後に呼ぶので、2・3・4・5 はどれも
  `insert_rc` の出力には掛からない。
  `lower_and_insert_rc` は `lower_program` の後に `simplify` と `insert_rc` を掛けるだけであり、
  そのどちらもこの欄を書かない (`insert_into_func` が代入するのは `func.body` である)。よって
  `insert_rc` の出力の各関数の `borrowed_units` は 1 が置いた空集合のままで
  ある。D14 は借用する unit の集合を `RcFunc::borrowed_units` と定め、残りを所有すると定めるので、
  すべてのパラメータ・capture の unit が所有される。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5

**(e) は最適化の水準に依らない。** `borrow_ify` を呼ぶのは `optimize_rc_program` の
`if config.enable_borrow_optimization()` の中だけなので (`CODE src/build/build_object_files.rs:
optimize_rc_program`)、門が偽のとき `insert_rc` の出力は `borrow_ify` の入力にならず、A1 はそれに
当たらない。(e) の証明はコードの数え上げと D14 だけに立つので、門の真偽を問わない。
**(e) を引く段はこの文書のどこにも在りうる。循環しないことを述語で述べる** -- (e) の証明はこの文書の
命題を 1 つも引かないので、(e) を引く段がどれであっても循環は生じない。一覧でなく述語で書くのは、
段が増えるたびに一覧が古くなるからである。手で据えた本体 (`L12` の `C1`、`L13` の `B_1`) は
`insert_rc` の出力ではないので (e) の範囲の外であり、その `borrowed_units` は `DEF C1 の本体` と
`DEF 関数 h` が取り決める。

### 10.4 `L16` (素通しの宣言についての 2 つ) <!--#e885aa0-->

**言明**。

- **(a)** `Llvm(gen, args)` について、`gen.borrows_operand(i, ・, ・)` が真であるとき、結果のどの leaf も
  単一の `Arg(i, σ)` を宣言しない。
- **(b)** `Llvm(gen, args)` の結果の相異なる 2 つの boxed leaf が、同じ `Arg(i, σ)` を宣言することは
  ない。すなわち、結果の leaf からそれが宣言する `(i, σ)` への対応は単射である。

**(b) が要る理由。** D9 の移動の表の `Llvm` の行は素通し leaf ごとに 1 本の辺を挙げ、
`DEF 割り当て μ` はその辺 1 本につき移動元の `μ` を 1 下げる。2 つの結果 leaf が同じ `Arg(i, σ)` を
宣言すると、1 つのオペランドの leaf の `μ` が 1 つの節点で 2 下がる。**この形を A5 だけで排除する
ことはできない** -- A5 は、同じ参照を持つ 2 つの leaf は同じオブジェクトを指すという形で A5 を使う
議論が、両端が計数下であるときにだけ通ると述べるので、オペランドの leaf がグローバル状態の
オブジェクトを指す場合が残る。D6 のスロットは計数下に限らないので `μ` の勘定はその場合も走る。
そこで (b) は宣言の側の数え上げで示す。

**証明**

<1>1. `borrows_operand` を override する `impl LLVMGen for` は、`前提 borrows_operand の本体の在りか`
      が挙げるもので尽きる。既定は偽を返す。
  BY <ref id=e11772a/>, 前提 `borrows_operand` の本体の在りか,
     CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand
  その走査が挙げる項目のうち `src/ast/inline_llvm.rs` の 1 つは trait の既定の本体であり、その本体は
  `false` を返す。残るのが override である。

<1>2. `result_prov` の既定の実装は、結果のどの leaf にも `Unknown` だけを置く。
  BY CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, CODE src/rc_ir/provenance.rs: Provenance,
     CODE src/rc_ir/provenance.rs: Provenance::uniform, CODE src/rc_ir/provenance.rs: sole_origin,
     CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform,
     CODE src/rc_ir/ownership.rs: as_arg_projection
  既定は `Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)` であり、それは
  `LeafMap::uniform(ty, type_env, sole_origin(src))`、さらに
  `LeafMap::build_shape(ty, type_env, &|_| fact.clone())` である。`build_shape` は
  `boxed_leaf_paths(ty, type_env)` の各元に項を置くので、各 boxed leaf に `sole_origin(Unknown)` が
  入る。`as_arg_projection` は `LeafOrigin::Unknown` に `None` を返す。

<1>3. <1>1 の override のうち `result_prov` を override するのは
      `InlineLLVMArrayCopyCapacityBoundsUnchecked`、`InlineLLVMStructGetBody`、
      `InlineLLVMUnionAsBody` の 3 個である。
  BY <1>1, EXT クレートの項目,
     CODE src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov
  残る 10 個は `result_locality` を override するが `result_prov` は override しない。
  `EXT クレートの項目` より、<1>1 の各 `impl` ブロックはそれぞれ 1 か所にしかなく、その本体を
  読んで得たメソッドの一覧は完全である。

<1>3a. `InlineLLVMArrayCopyCapacityBoundsUnchecked` の `result_prov` は、結果の各 boxed leaf に
       単一の `Fresh` を置く。単一の `Fresh` は単一の `Arg(i, σ)` ではない。
  BY CODE src/fixstd/builtin.rs: InlineLLVMArrayCopyCapacityBoundsUnchecked::result_prov,
     CODE src/rc_ir/provenance.rs: Provenance::uniform, CODE src/rc_ir/provenance.rs: sole_origin,
     CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform,
     CODE src/rc_ir/provenance.rs: LeafOrigin
  その `result_prov` は `Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)` を返す。
  `uniform` は `LeafMap::uniform(ty, type_env, sole_origin(src))`、さらに
  `LeafMap::build_shape(ty, type_env, &|_| fact.clone())` であり、`build_shape` は
  `boxed_leaf_paths(ty, type_env)` の各元に項を置く。`sole_origin` は 1 元の
  集合を作る。`LeafOrigin` の構成子は `Arg`・`Fresh`・`Unknown` であり、`Fresh` は `Arg` ではない。

<1>4. 残る 2 個 -- `InlineLLVMStructGetBody` と `InlineLLVMUnionAsBody` -- は、`borrows_operand(i)` が
      真であるとき結果の型が `is_fully_unboxed` である。
  BY <ref id=83d98e9/>, CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_operand,
     CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::borrows_container,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_operand,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::borrows_union,
     CODE src/ast/types.rs: TypeNode::field_types
  `InlineLLVMStructGetBody::borrows_operand` は
  `i == 0 && Self::borrows_container(&arg_tys[0].field_types(type_env)[self.field_idx], type_env)` で
  あり、`InlineLLVMStructGetBody::borrows_container` の本体は
  `field_ty.is_fully_unboxed(type_env)` である。A12 の
  `Llvm` 節点の型についての節は「`InlineLLVMStructGetBody` の `ty(x)` は `ty(args[0])` の第
  `field_idx` フィールドの型であり、`InlineLLVMUnionAsBody` の `ty(x)` は `ty(args[0])` の第
  `field_idx` 変位の payload の型である」と述べる。**`arg_tys[0].field_types(type_env)[self.field_idx]`
  がその型であることは `field_types` の本体による** -- それは
  `self.instance_field_types(self.toplevel_tycon_info(type_env), type_env)`、すなわちその型が持つ
  フィールド (union では変位の payload) の型の列であり、その第 `field_idx` 元がその型である。
  よってこの `field_ty` はこの op の結果の型である。
  `InlineLLVMUnionAsBody::borrows_operand` は
  `i == 0 && Self::borrows_union(&arg_tys[0].field_types(type_env)[self.field_idx], type_env)` で
  あり、`InlineLLVMUnionAsBody::borrows_union` の本体は `payload_ty.is_fully_unboxed(type_env)` で
  ある。A12 の同じ節よりこの `payload_ty` はこの op の結果の型である。
  どちらの `borrows_operand` も `i == 0` を要求するので、`i == 0` 以外の `i` には偽を返す。

<1>5. 結果の型が `is_fully_unboxed` であるとき、宣言はどの leaf にも何も置かない。
  BY <ref id=0594f24/>, CODE src/rc_ir/provenance.rs: Provenance,
     CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform
  D4 の第 1 規則より `is_fully_unboxed` が真の型は boxed leaf を持たない。`Provenance::build_shape` と
  `Provenance::uniform` はどちらも `LeafMap::build_shape` を通り、それは
  `boxed_leaf_paths(ty, type_env)` の各元に項を置くので、leaf を持たない型では
  空である。

<1>5a. `LeafOrigin::Arg` を含む `Provenance` を返す `result_prov` の実装は下に挙げるもので尽き、
      そのどれでも結果の leaf からそれが宣言する `(i, σ)` への対応は単射である。
  BY <ref id=e11772a/>, <ref id=83d98e9/>, <ref id=0594f24/>, 前提 `LeafOrigin::Arg` の在りか,
     CODE src/fixstd/builtin.rs: PLUG_IN_PUNCHED_ARG,
     CODE src/fixstd/builtin.rs: PLUG_IN_FIELD_ARG,
     CODE src/fixstd/builtin.rs: STRUCT_SET_STRUCT_ARG,
     CODE src/fixstd/builtin.rs: STRUCT_SET_VALUE_ARG,
     CODE src/ast/inline_llvm.rs: LLVMGen::result_prov, CODE src/rc_ir/provenance.rs: LeafOrigin,
     CODE src/rc_ir/provenance.rs: Provenance::arg_passthrough,
     CODE src/rc_ir/provenance.rs: Provenance, CODE src/rc_ir/provenance.rs: sole_origin,
     CODE src/rc_ir/leaf_map.rs: LeafMap::build_shape, CODE src/rc_ir/leaf_map.rs: LeafMap::uniform,
     CODE src/fixstd/builtin.rs: InlineLLVMStructGetBody::result_prov,
     CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody,
     CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMStructPunchBody::arg_leaf_path,
     CODE src/fixstd/builtin.rs: replaced_field_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMStructPlugInBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMStructSetBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMMakeUnionBody::result_prov,
     CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov
  **在りかは前提 `LeafOrigin::Arg` の在りかが与える** (第 1.1 節)。その一覧のうち
  `LLVMGen::result_prov` の実装は 6 つであり、下に 1 つずつ当たる。残る項目は、宣言を**読む**側の
  パターン (`origin_from_leaves_under`・`as_arg_projection`・`Provenance::compose`・
  `leaf_source_to_string`・`resolve_leaf`) と、`Provenance::arg_passthrough` -- `Arg` を置くが
  `result_prov` の実装ではない -- である。既定の `result_prov`
  は `Unknown` を置く (<1>2) ので `Arg` を含まない。
  - `InlineLLVMStructGetBody::result_prov` の unbox の枝は、結果の leaf `π` に
    `Arg(0, [field_idx] ++ π)` を置く。`π ↦ [field_idx] ++ π` は単射である。
  - `impl LLVMGen for InlineLLVMMakeStructBody` の `result_prov` は、結果の leaf `[i] ++ rest` に
    `Arg(i, rest)` を置く。`[i] ++ rest ↦ (i, rest)` は単射である。
  - `InlineLLVMStructPunchBody::result_prov` の unbox の枝は、結果の leaf `π` に
    `Arg(0, arg_leaf_path(π))` を置く。結果の型は `(field, punched struct)` の 2 成分の tuple で
    あり (A12)、`arg_leaf_path` は `π = [PUNCHED_STRUCT_FIELD] ++ rest` を `rest` へ、
    残る成分の `π = [head] ++ rest` を `[field_idx] ++ rest` へ写す。各枝の中では単射である。
    枝をまたいで一致するのは `rest = [field_idx] ++ rest'` のときだが、A12 より punched struct の
    第 `field_idx` フィールドは穴であり、D4 の第 5 規則は `unpunched_field_types` が返さない穴の
    下へ降りないので、そのような結果 leaf は存在しない。
  - `InlineLLVMStructPlugInBody::result_prov` と `InlineLLVMStructSetBody::result_prov` は
    どちらも `replaced_field_prov` を呼ぶ。その unbox の枝は、結果の leaf `[field_idx] ++ rest` に
    `Arg(value_arg, rest)` を、残る leaf `[f] ++ rest` (`f ≠ field_idx`) に
    `Arg(struct_arg, [f] ++ rest)` を置く。各枝の中では単射であり、2 つの枝は相異なるオペランドの
    添字を置く -- `struct_plug_in` は `(PLUG_IN_PUNCHED_ARG, PLUG_IN_FIELD_ARG) = (0, 1)`、
    `struct_set` は `(STRUCT_SET_STRUCT_ARG, STRUCT_SET_VALUE_ARG) = (1, 0)` である
    (`CODE src/fixstd/builtin.rs: PLUG_IN_PUNCHED_ARG`,
    `CODE src/fixstd/builtin.rs: PLUG_IN_FIELD_ARG`,
    `CODE src/fixstd/builtin.rs: STRUCT_SET_STRUCT_ARG`,
    `CODE src/fixstd/builtin.rs: STRUCT_SET_VALUE_ARG`)。
  - `InlineLLVMMakeUnionBody::result_prov` の unbox の枝は、結果の leaf `[variant_idx] ++ rest` に
    `Arg(0, rest)` を置き、他の変位の leaf には空集合を置く。`Arg` を置く leaf の上で単射である。
  - `InlineLLVMUnionAsBody::result_prov` の unbox の枝は、結果の leaf `π` に
    `Arg(0, [variant_idx] ++ π)` を置く。`π ↦ [variant_idx] ++ π` は単射である。

  上の各項目の boxed の枝 -- `InlineLLVMStructGetBody`・`InlineLLVMStructPunchBody`・
  `replaced_field_prov`・`InlineLLVMUnionAsBody`・`InlineLLVMMakeUnionBody` のもの -- は
  `Unknown` か `Fresh` を置くので `Arg` を含まない。

<1>6. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>4, <1>5, <1>5a
  (a) について、`borrows_operand(i)` が真になるのは <1>1 の override のいずれかであり、そのうち 10 個は
  既定の `result_prov` を持ち `Arg` を宣言しない (<1>2、<1>3)。
  `InlineLLVMArrayCopyCapacityBoundsUnchecked`
  は結果の各 leaf に単一の `Fresh` を置くので `Arg` を宣言しない (<1>3a)。残る 2 個は、
  `borrows_operand(i)` が真であるとき結果に leaf が無い (<1>4、<1>5) ので、やはり `Arg(i, σ)` を
  宣言する leaf を持たない。(b) は <1>5a である。

**この命題が要る理由。** D9 の消費の表の `Llvm` の行は `borrows_operand(i)` が偽のオペランドだけを
挙げるが、移動の表の `Llvm` の行 (素通し leaf) はその条件を持たない。両方が同時に成り立つ op が在ると、
1 つの参照が結果へ移りながら呼び出し元にも残ることになり、`insert_rc` が置く「借用オペランドの最後の
使用の後の `Release`」がその参照を二重に処分する。`L16` (a) はその形が無いことを言う。

### 10.4a `L16a` (終端の `Ret` を書き換える呼び出しの `live_after` は空である) <!--#cd1f6fa-->

**言明**。関数本体・グローバル初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合で
あり、その骨格節点の前置 `Retain` 鎖は空である。

**証明**

<1>1. 本体の根を書き換える呼び出しの `live_after` は空集合である。
  BY EXT 空を作る `Default`, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: insert_rc
  関数については `insert_into_func` が `self.insert_into_expr(func.body, &Set::default())` を呼び、
  グローバル初期化子については `insert_rc` が `inserter.insert_into_expr(glob.init, &Set::default())` を
  呼ぶ。`EXT 空を作る Default` より `Set::default()` は空の集合である。

<1>2. 骨格節点 `m` が `Ret` でないとき、`m` の継続を書き換える呼び出しの `live_after` は、`m` を
      書き換える呼び出しの `live_after` に等しい。
  BY <ref id=d80dde9/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match
  D2 より `RcExpr` は 6 種であり、A25 より骨格は `Retain` と `Release` を含まないので、`Ret` でない
  骨格節点は `Let`(`Match` でない右辺)、`Let`(`Match` の右辺)、`Destructure`、`Eval` の 4 種である。
  4 つの関数はいずれも継続について `self.insert_into_expr(cont, live_after)` を呼ぶ。

<1>3. 関数本体・初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合である。
  BY <1>1, <1>2, <ref id=b3dfa37/>, <ref id=ca36627/>
  D2 より `Ret` 以外の 5 種はちょうど 1 つの継続を持つので、関数本体・初期化子の終端の `Ret` (D3) は
  本体の根から継続だけを辿って着く節点である。その継続の鎖の長さについての帰納で、鎖の各節点を
  書き換える呼び出しの `live_after` は根のもの、すなわち空集合である。

<1>4. QED
  BY <1>3, CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕
  `Ret` の腕が前置 `Retain` 鎖を作るのは `retain_if_live(&x, live_after, ret)` の 1 か所だけであり、
  その条件 `live.contains(&var.name)` は空集合について偽である。

### 10.5 `L17` (遷移は割り当てを liveness の指示関数へ運ぶ) <!--#18030ac-->

`ρ` の上で連続する 2 つの検査点の間に在る節点の列を**遷移**と呼ぶ。

**言明**。`insert_rc` の出力の 1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化を固定する。

- **(a0)** `insert_rc` の出力の各節点は、`insert_into_func` が本体の根の外に積む `Release` 鎖
  (`unused` の鎖) に属するか、いずれかの骨格節点の塊 (10.1 節) に属する。骨格節点は
  `Let`(`Match` でない右辺)・`Let`(`Match` の右辺)・`Destructure`・`Eval`・`Ret` の 5 種であり、
  アームの頭の `Release` 鎖を持つのは `Let`(`Match` の右辺) の塊だけ、後置 `Release` 鎖を持たないのは
  `Ret` の塊だけである。
- **(a)** `ρ` の上の各節点の入口は、本体の根の検査点より前の `Release` 鎖 (`insert_into_func` の
  `unused` の鎖) の中の点か、連続する 2 つの検査点の間の遷移の中の点 (先の検査点を含む) か、
  関数本体・初期化子の終端の `Ret` の検査点のいずれかである。遷移は下の (T1)・(T2)・(T3) の 3 種で
  尽きる。
- **(b)** 検査点 `m` から検査点 `m'` への遷移について、`m` の時点で「各スロット `(v, λ)` の `μ` は、
  `v ∈ Λ(m)` のとき 1、そうでないとき 0 である」が成り立つならば、`m'` の時点で同じことが `Λ(m')`
  について成り立つ。さらに遷移の中の各節点の入口で、すべてのスロットについて `μ ≥ 0` であり、その
  節点が読む値の各スロットと `Retain`/`Release` が触れる各スロットについて `μ ≥ 1` である。
- **(c)** 遷移の中の各節点 `n` とスロット `s` について、`n` が `μ(s)` を下げる回数を `k_n(s)` とすると、
  `n` の入口で `μ(s) ≥ k_n(s)` である。

**証明**

<1>0. 局所名でない変数はスロットを持たない。したがって以下の各場合で `μ` を勘定するのは局所名の変数に
      ついてだけでよく、`DEF 割り当て μ` の 6 種の行のうち**局所名でない変数のスロット**を主語にする
      ものは 1 つも発火せず、(b) の「読む値の各スロット」「触れる各スロット」の量化もその変数に
      ついては空である。**節点が他の変数のスロットの `μ` を動かすことは妨げない** -- `App` の callee が
      局所名でなくても、その `App` は引数の leaf の `μ` を下げ、結果の leaf の `μ` を上げる。
  BY <ref id=dca1c02/>, DEF 割り当て `μ`
  `L15` (d) が局所でない名前についてこれを与える。`DEF 割り当て μ` の 6 種はいずれもスロットについて値を
  動かすので、スロットを持たない変数についてはどの行も発火しない。

<1>1. (a)。遷移は次の 3 種で尽きる。
  <2>1. **(T1)** `m` が `Let(x, rhs, cont)` (`rhs` は `Match` でない)、`Destructure(c, fs, _, cont)`、
        `Eval(x, cont)` のいずれかであるとき。遷移は `m` の前置 `Retain` 鎖、核節点、後置 `Release` 鎖
        であり、`m'` は `cont` である。
    BY <ref id=ca36627/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval
    3 つとも、`insert_into_expr(cont, ・)` が返した節点の外へ後置 `Release` 鎖を積み、その外に核節点を
    作り、その外に前置 `Retain` 鎖を積む。**`ρ` の上の順序はこの入れ子から出る** -- D3 より
    `Ret` を除く 5 種の節点では実行路はその継続へ進むので、外側の節点ほど先に訪れられる。よって
    `ρ` はこの塊を、前置 `Retain` 鎖・核節点・後置 `Release` 鎖の順に訪れ、その後 `cont` の写しの
    根 -- `m'` の検査点 -- へ進む。
  <2>2. **(T2)** `m` が `Let(x, Match(scrut, arms), cont)` であるとき。`ρ` が選ぶアームを `j` とすると、
        遷移は `m` の前置 `Retain` 鎖 (`retain_if_live(&scrut, &live_at_arm_head, ・)` が置く高々 1 つの
        `Retain`)、核節点 `Let(x, Match(scrut, arms'), ・)`、アーム `j` の頭の `Release` 鎖であり、
        `m'` はアーム `j` の本体の根である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, <ref id=ca36627/>
    `insert_into_match` は各アームの本体を `build_releases(head, body)` で包む。D3 より
    `Let(x, Match(v, arms), k)` の実行路はアームを 1 つ選んでその本体へ進む。
  <2>3. **(T3)** `m` がアーム本体の終端の `Ret(r)` であるとき。遷移は `m` の前置 `Retain` 鎖
        (`retain_if_live(&r, live_after, ret)` が置く高々 1 つの `Retain`)、核節点 `Ret(r)`、および
        その `Match` の核節点と `cont` の間に置かれた `Release` 鎖 (`x` が `live_cont` に入らないときの
        `Release(x, [])`) であり、`m'` はその `Match` の `cont` である。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, <ref id=ca36627/>
    D3 より、アーム本体の実行路を辿り終えると `k` へ進む。`insert_into_match` は
    `!live_cont.contains(&x.name) && self.needs_rc(&x)` のとき `build_releases(vec![x], cont)` を置く。
  <2>4. (a0)。
    BY <ref id=3e6b0e0/>, <ref id=d80dde9/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
    `insert_rc` が節点を作るのは、`insert_into_func` の `build_releases(unused, body)` と、
    `insert_into_expr_inner` が骨格節点 1 つを受け取って走る 5 つの腕だけである (A15 より
    `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼び、A25 より
    `Retain`/`Release` の腕は通らない)。その 5 つの腕が作る節点は、10.1 節が挙げる 4 つの部分 --
    前置 `Retain` 鎖、核節点、アームの頭の `Release` 鎖、後置 `Release` 鎖 -- で尽きる。継続と
    アーム本体の節点は、それぞれについての `insert_into_expr` の呼び出しが作るものである。
    **骨格節点の種類が 5 つであることは D2 と A25 による** -- D2 より `RcExpr` は 6 種であり、A25 より
    骨格は `Retain` と `Release` を含まないので、残る 4 種の `Let` を右辺が `Match` かどうかで
    2 つに分けた 5 種である。**アームの頭の `Release` 鎖を積むのは `insert_into_match` だけで
    あり**、その関数は `Let(x, Match(scrut, arms), cont)` の腕からしか呼ばれない。**後置 `Release` 鎖を
    積まないのは `RcExpr::Ret(x)` の腕だけである** -- 残る 4 つの腕が振り分ける関数はどれも
    `build_releases` を呼び、`Ret` の腕は `retain_if_live` しか呼ばない。
  <2>4a. QED
    BY <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=d80dde9/>, <ref id=cd1f6fa/>, <2>1, <2>2, <2>3, <2>4,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕
    D2 より骨格節点は 6 種であり、A25 より `Retain` と `Release` は骨格に無い。残る 4 種について、
    塊の節点が `ρ` の上でどの遷移に入るかを見る。
    `Let`(非 `Match`)、`Destructure`、`Eval` の塊はアームの頭の `Release` 鎖を持たないので、
    前置 `Retain` 鎖・核節点・後置 `Release` 鎖の 3 つであり、<2>1 よりそれがそのまま (T1) の遷移で
    ある。
    `Let(x, Match(scrut, arms), cont)` の塊は 2 つに分かれる。前置 `Retain` 鎖・核節点・`ρ` が選んだ
    アーム `j` の頭の `Release` 鎖は <2>2 より (T2) の遷移をなし、後置 `Release` 鎖は <2>3 より
    アーム `j` の本体の終端の `Ret` から始まる (T3) の遷移の末尾に入る。D3 より `ρ` は選ばれなかった
    アームの頭の `Release` 鎖を訪れない。
    `Ret` は木の中の位置で 2 つに分かれる。アーム本体の終端の `Ret` の塊は前置 `Retain` 鎖と核節点で
    あり (`Ret` の腕は `build_releases` を呼ばないので後置 `Release` 鎖が無い)、<2>3 よりその 2 つは
    その `Match` の後置 `Release` 鎖と合わせて (T3) の遷移をなす。関数本体・初期化子の終端の `Ret` は
    `ρ` の最後の骨格節点であって後続の検査点を持たないので遷移を成さないが、`L16a` よりその前置
    `Retain` 鎖は空であり、後置 `Release` 鎖も無いので、その塊は核節点 1 つ -- すなわちその検査点
    1 点 -- である。
    よって `ρ` の上の各節点の入口は、`unused` の解放鎖の中の点か、(T1)・(T2)・(T3) のいずれかの
    遷移の中の点か、関数本体・初期化子の終端の `Ret` の検査点かのいずれかであり、遷移はこの 3 種で
    尽きる。(a0) は <2>4 である。

<1>2. **CASE (T1)** で `m = Let(x, rhs, cont)`、`rhs` は `Match` でない。
  <2>1. `Λ(m) = (Λ(m') \ {x}) ∪ ops` である。ここで `ops` は `rhs_operands(rhs)` が挙げるオペランドの
        局所名の集合であり、`x ∉ ops` である。
    BY <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: insert_if_local
    `insert_into_operation_let` は `live_cont` (= `Λ(m')`) から `x` を除き、各オペランドの局所名を
    `insert_if_local` で足したものを返す。`x ∉ ops` は A11 と D2 から出る -- `ops` の各名前は `rhs` の
    位置での使用であり、A11 よりその使用はその位置でスコープに入っている束縛に解決する。D2 より
    `Let(x, rhs, k)` が束縛する `x` のスコープは `k` の部分木であって `rhs` の位置を含まないので、
    `ops` の使用が `x` の束縛に解決することはない。A6 より `x` という名前を束縛するものはプログラム
    全体で 1 つだけなので、`ops` に `x` は現れない。
  <2>2. `ops` の各名前 `v` について、`n_v` を `v` の `Own` の出現回数とすると、前置 `Retain` 鎖が
        `v` を名指す回数は `n_v - [v ∉ Λ(m') かつ v の最後の出現が Own]` であり、後置 `Release` 鎖が
        `v` を名指す回数は `[v ∉ Λ(m') かつ v の最後の出現が Borrow]` である。コードはどちらも
        `needs_rc` で絞るが、`needs_rc` が偽の名前はスロットを持たない (`L15` (c)) ので、以下の
        勘定では区別しない。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: build_retains,
       CODE src/rc_ir/rc_insert.rs: build_releases, <ref id=dca1c02/>, EXT `Iterator::rev`
    ループは `operands.iter().rev()` を走り (`EXT Iterator::rev` よりそれは `operands` を逆順に
    走る)、`live_after_operand` を `live_cont`(= `Λ(m')`) の写しから
    始めて各局所オペランドの名前を足していく。よって `v` の出現 `i` における `used_later` は
    「`v ∈ Λ(m')` または `v` の出現が `i` より後ろにある」である。`Own` の出現は `used_later` が真で
    `needs_rc(v)` のとき `retains_before` に入り、`Borrow` の出現は `used_later` が偽で `needs_rc(v)`
    のとき `releases_after` に入る。`used_later` が偽になるのは `v ∉ Λ(m')` のときの最後の出現だけで
    ある。
  <2>3. 後置 `Release` 鎖は、さらに `x ∉ Λ(m')` のとき `x` を名指す `Release` を 1 つ置く。
        コードは `needs_rc` でも絞るが、`needs_rc` が偽の名前はスロットを持たない (`L15` (c)) ので、
        以下の勘定では区別しない。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let, <ref id=dca1c02/>
    `after` は `releases_after` に、`!live_cont.contains(&x.name) && self.needs_rc(&x)` のとき `x` を
    足したものである。
  <2>3a. `rhs = Llvm(gen, args)` のとき、素通しを宣言する結果の boxed leaf とそれが名指すオペランドの
         leaf は 1 対 1 に対応する。したがって 1 つのオペランドの leaf を名指す素通しの辺は高々 1 本で
         ある。
    BY <ref id=e885aa0/>, <ref id=e11772a/>, <ref id=9d74736/>
    `L16` (b) より、結果の相異なる 2 つの boxed leaf が同じ `Arg(i, σ)` を宣言することはない。
    D9 の移動の表の `Llvm` の行は、単一の `Arg(i, σ)` を宣言する結果の leaf 1 つにつき、
    オペランド `i` の leaf `σ` からその結果 leaf への辺を 1 本挙げる (A3 の表の
    「単一の `Arg(j, σ)`」の行が「第 `j` オペランドの leaf `σ` と**同じ参照**」と書く)。よって辺の
    集合は結果 leaf から
    オペランド leaf への単射のグラフであり、1 つのオペランドの leaf を始点とする辺は高々 1 本である。
  <2>4. 核節点は、`ops` の各 `Own` の出現ごとに、そのオペランドの inhabited な各 boxed leaf について
        `μ` を 1 下げる。`Borrow` の出現は `μ` を変えない。`ops` に入らないオペランド -- 局所名でない
        もの -- については、<1>0 より動くスロットが無い。
    BY <1>0, <2>3a, <ref id=9d74736/>, <ref id=dca1c02/>, <ref id=e885aa0/>
    `rhs = Var(y)`: D9 の移動の表の `Let(x, Var(y), k)` の行が `y` の全 leaf を移す。
    `rhs = App(callee, args)`: D9 の消費の表の `App` の行が callee の全 boxed leaf と、呼び出し先が
    所有する位置の引数の leaf を消費する。`L15` (e) よりすべての位置が所有される。
    `rhs = Closure(f, caps)`: 消費の表の `Closure` の行が全 capture の全 leaf を消費する。
    `rhs = Llvm(gen, args)`: `borrows_operand(i)` が偽のオペランドの各 leaf は、素通しを宣言されて
    いれば移動の表の `Llvm` の行で結果へ移り、されていなければ消費の表の `Llvm` の行で消費される。
    どちらでも `μ` は 1 下がる。**2 下がらないことは <2>3a による** -- 1 つのオペランドの leaf を
    始点とする素通しの辺は高々 1 本であり、素通しを宣言された leaf は消費の表の行に入らない。
    `borrows_operand(i)` が真のオペランドは、消費の表の行に入らず、`L16` (a)
    より移動の表の行にも入らない。
  <2>5. 核節点は、結果の inhabited な各 boxed leaf `(x, μ')` について `μ` を 1 上げる。
    BY <2>3a, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=e11772a/>, <ref id=0594f24/>
    `rhs = Var(y)`: 移動の表の行が `x` の各 leaf に移す。
    `rhs = App` / `Closure`: D10 の生成の表の対応する行が結果の各 leaf に参照を作る。`Closure` の
    結果は capture object 1 つであり、D4 の第 2 規則よりクロージャの boxed leaf は capture の位置の
    1 つだけなので、この行は結果の唯一の leaf に 1 つ置く。
    `rhs = Llvm`: 素通しの leaf は移動の表の行で結果へ入り、素通しでない leaf は D10 の生成の表の
    `Llvm` の行で参照を得る。A3 の表より結果の各 leaf の宣言は単一の `Arg` かそれ以外かのいずれかで
    あり、この 2 つは排他なので、結果の各 inhabited な leaf は素通しか生成かのちょうど一方で `μ` を
    1 得る。<2>3a より、素通しの辺が同じ結果 leaf に 2 本着くこともない。
  <2>6. 遷移の後、`ops` の各名前 `v` について `μ(v, λ) = [v ∈ Λ(m')]`、`x` について
        `μ(x, μ') = [x ∈ Λ(m')]` であり、他の名前の `μ` は変わらない。
    BY <1>0, <2>2, <2>3, <2>4, <2>5
    仮定より遷移の前は `μ(v, λ) = 1` (`v ∈ ops ⊆ Λ(m)`) であり `μ(x, μ') = 0` (`x` はまだ値を得て
    いない)。`v` について、前置 `Retain` が `n_v - [v ∉ Λ(m') ∧ 最後が Own]` 回上げ、核節点が `n_v`
    回下げ、後置 `Release` が `[v ∉ Λ(m') ∧ 最後が Borrow]` 回下げるので、
    `1 - [v ∉ Λ(m') ∧ 最後が Own] - [v ∉ Λ(m') ∧ 最後が Borrow] = 1 - [v ∉ Λ(m')] = [v ∈ Λ(m')]`。
    `x` について、核節点が 1 上げ、後置 `Release` が `[x ∉ Λ(m')]` 回下げるので `[x ∈ Λ(m')]`。
    <2>1 より `Λ(m)` と `Λ(m')` の差は `ops` と `{x}` の上にしか無いので、他の名前は両方に入るか
    両方に入らないかであり、`μ` も変わらない。
  <2>7. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <ref id=8e3aff3/>, <1>0, <2>2, <2>3, <2>4, <2>5, <2>6, <ref id=33c54dc/>, <ref id=56c2068/>
    前置 `Retain` 鎖の中では `μ` は上がるだけなので、`μ(v, λ) ≥ 1` (`v ∈ ops`) が保たれ、鎖の各
    `Retain` が触れるスロットは `μ ≥ 1` である。核節点の入口では `μ(v, λ) = 1 + (前置の回数) ≥ n_v`
    であり、核節点が読む値 (D7 の表: `Llvm` の各オペランド、`App` の callee と各引数、`Closure` の
    各 capture、`Destructure` の容器、`Eval` の変数) のうち `ops` の名前のものは `μ ≥ 1` であり、
    残るもの -- 局所名でないもの -- は <1>0 よりスロットを持たないので量化から外れる。核節点の後、
    後置 `Release` 鎖の各名前は相異なる -- `releases_after`
    に同じ名前が 2 度入ることはなく (`used_later` が偽になるのは最後の出現だけ)、`x` はオペランドとは
    別の名前である (A6) -- ので、鎖の `i` 番目の `Release` の入口での `μ` は、その名前について
    <2>6 の最終値 `[・ ∈ Λ(m')]` に、まだ実行していない自分自身の分 1 を足したもの以上であり、1 以上で
    ある。それらの `Release` の後の値は <2>6 の最終値であり非負である。
  <2>7a. (c)。
    BY <ref id=8e3aff3/>, <2>2, <2>3, <2>4, <2>5, <2>6, <ref id=33c54dc/>,
       CODE src/rc_ir/rc_insert.rs: build_releases
    核節点は、<2>4 より
    `ops` の各名前 `v` の各スロット `(v, λ)` を `n_v` 回下げ、<2>5 より `x` のスロットは上げるだけ
    なので、`k_核(v, λ) = n_v`、`k_核(x, ・) = 0`、他のスロットについては 0 である。核節点の入口での
    `μ(v, λ)` は、<2>2 より `1 + n_v - [v ∉ Λ(m') ∧ 最後が Own] ≥ n_v` である。後置 `Release` 鎖の
    各節点は path が空列 -- `build_releases` が作る式は
    `RcExpr::Release(v, vec![], RcState::Unknown, c)` である -- なのでその名前の各スロットを
    1 回ずつ下げ、鎖の中に同じ名前は 2 度現れない
    (`releases_after` に同じ名前は 2 度入らず、`x` はオペランドとは別の名前である (A6))。<2>7 より
    その入口の値は 1 以上である。
  <2>8. QED
    BY <2>6, <2>7, <2>7a

<1>3. **CASE (T1)** で `m = Destructure(container, fields, _, cont)`。
  <2>1. `container` が局所名であるとき `Λ(m) = (Λ(m') \ {フィールド変数}) ∪ {container}`、そうでない
        とき `Λ(m) = Λ(m') \ {フィールド変数}` である。前置 `Retain` 鎖は `container` が局所名で
        `container ∈ Λ(m')` のとき `Retain(container, [])` を 1 つ置き、後置 `Release` 鎖は `Λ(m')` に
        入らない各フィールド変数の `Release` を 1 つずつ置く。**以下 `container` は局所名とする。**
        局所名でないときは、<1>0 よりこの変数はスロットを持たないので、`container` のスロットを
        主語にする以下の各等式は量化する対象を 1 つも持たず、フィールド変数のスロットについての
        等式だけが残る。前置 `Retain` 鎖もそのとき空である -- `retain_if_live` は
        `var.name.is_local()` を要求する。コードは前置 `Retain` も後置 `Release` も `needs_rc` で
        絞るが、`needs_rc` が偽の名前はスロットを持たない (`L15` (c)) ので、以下の勘定では
        区別しない。
    BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: insert_if_local,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live, <ref id=dca1c02/>
    `retain_if_live(&container, &live_cont, node)` は `var.name.is_local()` を要求したうえで
    `live_cont` (= `Λ(m')`) を見る。`live_before` へ `container` を足すのは `insert_if_local` なので、
    局所名でない `container` は `Λ(m)` に入らない。`dead` は
    `!live_cont.contains(&fv.name) && self.needs_rc(fv)` を満たすフィールド変数である。
  <2>2. CASE `container` が boxed。
    BY <2>1, <ref id=9d74736/>, <ref id=f06144e/>
    D9 の消費の表の「`Destructure(c, fs)` (`c` が boxed)」の行より容器の全 boxed leaf が消費され (`μ` が各
    leaf で 1 下がる)、D10 の生成の表の「boxed 容器の `Destructure` の各名前付きフィールドの各 leaf」の
    行より各フィールドの leaf の `μ` が 1 上がる。よって遷移の後
    `μ(container, λ) = 1 + [container ∈ Λ(m')] - 1 = [container ∈ Λ(m')]`、
    `μ(fv, λ') = 1 - [fv ∉ Λ(m')] = [fv ∈ Λ(m')]`。
  <2>3. CASE `container` が unbox。
    BY <2>1, <ref id=9d74736/>, CODE src/rc_ir/ownership.rs: destructure_consumes
    D9 の消費の表の「`Destructure(c, fs)` (`c` が unbox)」の行より名前の付いていないフィールドの leaf が
    消費され、移動の表の「unbox 容器の `Destructure` の名前付きフィールド」の行より名前の付いた
    フィールドの leaf は容器からフィールド変数へ移る。どちらでも容器の各 leaf の `μ` は 1 下がり、
    名前の付いたフィールドの leaf では対応するフィールド変数の `μ` が 1 上がる。よって <2>2 と同じ
    最終値になる。
  <2>4. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <ref id=8e3aff3/>, <2>1, <2>2, <2>3, <ref id=33c54dc/>, <ref id=56c2068/>
    前置 `Retain` は `μ(container, ・)` を上げるだけである。D7 の読む構文の表の
    `Destructure(c, fs, s, k)` の行より核節点が読む値は容器であり、その入口で
    `μ(container, λ) = 1 + [container ∈ Λ(m')] ≥ 1`。後置 `Release` 鎖の名前は相異なるフィールド変数
    (A6) であり、各 `Release` の入口でその変数の `μ` は 1 である。
  <2>4a. (c)。
    BY <ref id=8e3aff3/>, <2>1, <2>2, <2>3, <2>4, <ref id=33c54dc/>,
       CODE src/rc_ir/rc_insert.rs: build_releases
    前置 `Retain` は `μ` を上げるだけなので `k = 0` である。核節点は、<2>2 と <2>3 より容器の各
    スロットを 1 回だけ下げ、フィールド変数のスロットは上げるだけなので、`k_核 ≤ 1` であり、
    <2>4 より容器のスロットの入口の値は 1 以上である。後置 `Release` 鎖の各節点は path が空列 --
    `build_releases` が作る式は `RcExpr::Release(v, vec![], RcState::Unknown, c)` である -- なので
    その名前の各スロットを 1 回ずつ下げ、鎖の名前は相異なるフィールド変数 (A6) なので、<2>4 より
    その入口の値は 1 である。
  <2>5. QED
    BY <2>2, <2>3, <2>4, <2>4a, CODE src/ast/types.rs: TypeNode::is_box
    `is_box` は `!is_unbox` を返すので、容器は boxed か unbox かのいずれかである。

<1>4. **CASE (T1)** で `m = Eval(x, cont)`。
  BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
     CODE src/rc_ir/rc_insert.rs: insert_if_local, CODE src/rc_ir/rc_insert.rs: build_releases,
     <ref id=9d74736/>, <ref id=56c2068/>, <ref id=dca1c02/>
  `insert_into_eval` は `live_cont` (= `Λ(m')`) に `insert_if_local(&mut live_before, &x.name)` で
  `x` を足したものを返し、`insert_if_local` は `name.is_local()` のときだけ集合へ入れるので
  `Λ(m) = Λ(m') ∪ ({x} ∩ 局所名)` である。前置 `Retain` 鎖は空であり、後置 `Release` 鎖は
  `x` が局所名で `x ∉ Λ(m')` かつ `needs_rc(x)` のとき `build_releases(vec![x], cont)` で
  `Release(x, [])` を 1 つ置く -- **その path が空列であることは `build_releases` の本体による**
  (`RcExpr::Release(v, vec![], RcState::Unknown, c)`)。D9 の 2 つの表に `Eval` の行は無いので核節点は
  `μ` を変えない。よって遷移の後 `μ(x, λ) = 1 - [x ∉ Λ(m')] = [x ∈ Λ(m')]` であり、他の名前は変わら
  ない。核節点は `x` を読み (D7)、その入口で `μ(x, λ) = 1` である。`Release` の入口でも 1 である。
  (c) について、核節点は `μ` を下げないので `k = 0` であり、`Release(x, [])` は path が空列なので
  `x` の各スロットを
  1 回ずつ下げてその入口の値は 1 である。`x` が局所名でないときは <1>0 より `x` はスロットを持たず、
  `Release` も置かれないので、この場合の勘定は空である。`needs_rc(x)` が偽のときも `L15` (c) より
  `x` はスロットを持たないので、勘定は同じく空である。

<1>5. **CASE (T2)**。`m = Let(x, Match(scrut, arms), cont)`、`ρ` が選ぶアームを `j` とする。
  <2>0. `scrut` が局所名でないとき、<1>0 より `scrut` はスロットを持たず、この場合の `scrut` の列の
        勘定は空である。`H` は局所名だけからなるので `scrut` を含まず、
        `retain_if_live` は `var.name.is_local()` を要求するので前置 `Retain` は置かれず、
        `DB_j ⊆ H` にも `scrut` は入らない。頭の `Release(scrut, [])` は置かれうるが、動かすスロットが
        無い。以下 `scrut` は局所名とする。
    BY <1>0, <ref id=dca1c02/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: free_locals,
       CODE src/rc_ir/rc_insert.rs: insert_if_local
    `insert_into_match` は `live_at_arm_head` を `live_after_match.clone()` から始めて各アームの
    `arm_free_locals` の名前を足したものとして作り、`live_after_match` は `live_cont` から `x` を
    除いたものである。`live_cont` は `insert_into_expr(cont, live_after)` の返り値、すなわち
    `Λ(cont)` であり、`L15` (a) よりその各名前は局所名である。`arm_free_locals` が返すのは
    `free_locals(arm.body)` から payload を除いたものであり、`free_locals` が集める名前は
    `insert_if_local` が `name.is_local()` の門を通したものだけである。よって `H` は局所名だけからなる。
  <2>1. `H := live_at_arm_head`、`M := live_after_match`、`U_j := arm_free_locals(arm_j)`、
        `P_j := {payload_j} ∩ free_locals(arm_j.body)` と置くと、`Λ(m) = H ∪ {scrut}`、
        `Λ(m') = U_j ∪ P_j ∪ M` であり、`H = M ∪ (∪_i U_i)` である。
    BY <2>0, <ref id=66e786e/>, <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=1172c08/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals,
       CODE src/rc_ir/rc_insert.rs: free_locals,
       CODE src/rc_ir/rc_insert.rs: collect_referenced_and_bound
    `live_at_arm_head` は `live_after_match.clone()` から始めて各アームの `arm_free_locals` の名前を
    足したものなので、`H = M ∪ (∪_i U_i)` である。
    **アームが 1 つ以上あることが要る。** `insert_into_match` が返す `live_before` は
    `Set::default()` -- `EXT 空を作る Default` より空の集合 -- から始まる `live_before_arms` を
    土台にするので、アームが 0 個ならそれは空になり、
    `Λ(m) = {scrut}` となって下の等式が破れる。A9 は「`borrow_ify` の入力プログラムのすべての `Match`
    は 1 つ以上のアームを持つ」と述べ、A9 自身が「**`insert_rc` の入力と出力について読む段は A2 を
    引く。**」と書き、A2 が「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の
    入力と出力についても読める。**」を与えるので、`insert_rc` の入力の骨格の各 `Match` は
    アームを 1 つ以上持つ。
    `L14` (a) より `Λ(m) = free_locals(m) ∪ A(m)` であり、`free_locals(m)` は
    `{scrut} ∪ (∪_i U_i) ∪ (free_locals(cont) \ {x})` である。**この分解はスコープの規律に立つ** --
    `free_locals` は `m` の部分木が参照する局所名から、その部分木が束縛する局所名を落としたもので
    あり (`collect_referenced_and_bound`)、A11 より各使用はその位置でスコープに入っている束縛に
    解決し、A6 より 1 つの名前を束縛するものはプログラム全体で 1 つなので、D2 のスコープの規則より
    `scrut` と各 `U_i` の名前は `m` の部分木が束縛する名前と交わらず、`x` の束縛のスコープは `cont` の
    部分木に収まる。`M = live_cont \ {x}` は
    `(free_locals(cont) ∪ A(m)) \ {x}` なので `Λ(m) = {scrut} ∪ (∪_i U_i) ∪ M = {scrut} ∪ H`。
    アーム本体は `insert_into_expr(arm.body, &live_after_match)` で書き換えられるので `L14` (a) より
    `Λ(m') = free_locals(arm_j.body) ∪ M = U_j ∪ P_j ∪ M`。
  <2>2. 前置 `Retain` 鎖は `scrut ∈ H` のとき `Retain(scrut, [])` を 1 つ置く。アーム `j` の頭の
        `Release` 鎖は、`DB_j := H \ (U_j ∪ M)` の各名前の `Release`、`scrut.ty.is_box` かつ
        `arm_j.tag` が `Some` のときの `Release(scrut, [])`、`payload_j ∉ Λ(m')` のときの
        `Release(payload_j, [])` をこの順に置く。コードはどれも `needs_rc` で絞るが、`needs_rc` が
        偽の名前はスロットを持たない (`L15` (c)) ので、以下の勘定では区別しない。
    BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: build_releases, <2>1, <ref id=dca1c02/>
    `retain_if_live(&scrut, &live_at_arm_head, node)` は `H` を見る。`head` は dead-branch
    (`!used.contains(n) && !live_after_match.contains(n)` を満たす `live_at_arm_head` の名前)、
    `release_container && arm.tag.is_some() && needs_rc(&scrut)`、
    `!body_live.contains(&payload.name) && needs_rc(&payload)` の順に積まれる。`body_live` は
    `Λ(m')` であり、`release_container` は `scrut.ty.is_box(self.type_env)` である。
  <2>3. `DB_j` の各名前 `n` (`scrut` を含みうる) について `n ∉ Λ(m')` であり、`Λ(m) \ Λ(m')` は
        `DB_j ∪ ({scrut} \ H)`、`Λ(m') \ Λ(m)` は `P_j` である。
    BY <ref id=8e3aff3/>, <2>1, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=dca1c02/>,
       CODE src/rc_ir/rc_insert.rs: RcInserter::arm_free_locals
    `DB_j = H \ (U_j ∪ M)` であり `Λ(m') = U_j ∪ P_j ∪ M`。**`payload_j` が `H` に入らないことは
    A6・A11・D2 のスコープの規則による** -- `H = M ∪ (∪_i U_i)` (<2>1) であり、`U_i` は
    `arm_free_locals(arm_i)` すなわち `free_locals(arm_i.body)` から `arm_i` の payload を除いたもので
    ある。`i = j` の分は `payload_j` を除いてある。`i ≠ j` の分について、A11 よりアーム `i` の本体の
    中の `payload_j` という名前の使用はその位置でスコープに入っている束縛に解決し、D2 のスコープの
    規則より `payload_j` のスコープはアーム `j` の本体の部分木であってアーム `i` の本体を含まず、
    A6 よりその名前を束縛するものはプログラム全体で 1 つだけなので、`payload_j ∉ U_i` である。
    `M` の分について、`M = live_cont \ {x}` であり `live_cont = Λ(cont)` なので、`L15` (a) より
    `Λ(cont)` の各名前を束縛するのは `cont` の真の祖先の骨格節点かパラメータ・capture であり、
    アームの payload はそのどちらでもない。よって `payload_j ∉ H` であり、
    `DB_j ∩ Λ(m') = ∅`。**`payload_j ≠ scrut` も同じ 3 つから出る** -- `scrut` は `Match` 節点の位置で
    使われる名前であり、A11 よりその使用はその位置でスコープに入っている束縛に解決し、D2 より
    `payload_j` のスコープはアーム `j` の本体であってその位置を含まず、A6 よりその名前を束縛するものは
    プログラム全体で 1 つだけである。
    `Λ(m) \ Λ(m') = (H ∪ {scrut}) \ (U_j ∪ P_j ∪ M)` であり、`payload_j` は
    `H ∪ {scrut}` に入らないので `= (H \ (U_j ∪ M)) ∪ ({scrut} \ (U_j ∪ M))`。`scrut ∈ H` のとき
    第 2 項は第 1 項に含まれ、`scrut ∉ H` のとき `scrut ∉ M ∪ (∪_i U_i)` なので第 2 項は `{scrut}` で
    ある。`Λ(m') \ Λ(m) = (U_j ∪ P_j ∪ M) \ (H ∪ {scrut}) = P_j` (`U_j ∪ M ⊆ H`)。
  <2>4. CASE `scrut.ty.is_box` かつ `arm_j.tag = Some(t)`。
    BY <2>2, <2>3, <ref id=9d74736/>, <ref id=f06144e/>
    D10 の生成の表の「boxed union の変位アームの payload の各 leaf」の行が `μ(payload_j, λ')` を
    1 上げる。scrutinee の側は D9 の 2 つの表のどの行にも当たらない -- 参照を処分するのは
    `insert_into_match` が置いた `Release(scrut, [])` である。よって遷移の後
    `μ(scrut, λ) = 1 + [scrut ∈ H] - [scrut ∈ DB_j] - 1`。`scrut ∈ H` のとき
    `[scrut ∈ DB_j] = [scrut ∉ U_j ∪ M]` なのでこれは `[scrut ∈ U_j ∪ M] = [scrut ∈ Λ(m')]`
    (`scrut ∉ P_j`)。`scrut ∉ H` のとき `scrut ∉ U_j ∪ M` すなわち `scrut ∉ Λ(m')` であり、
    `[scrut ∈ DB_j] = 0` なので値は 0 である。どちらでも `[scrut ∈ Λ(m')]` に等しい。
    `μ(payload_j, λ') = 1 - [payload_j ∉ Λ(m')] = [payload_j ∈ Λ(m')]`。
    `DB_j` の他の名前 `n` は `μ(n, ・) = 1 - 1 = 0 = [n ∈ Λ(m')]` (<2>3)。
  <2>5. CASE `arm_j.tag = None`、または (`arm_j.tag = Some(t)` かつ `scrut.ty.is_box` が偽)。
    BY <2>2, <2>3, <ref id=9d74736/>, <ref id=66c9670/>, <ref id=c232680/>, <ref id=f769887/>, <ref id=83d98e9/>
    どちらの場合も `release_container && arm.tag.is_some()` が偽なので容器の `Release` は置かれない。
    catch-all アームでは、D9 の移動の表の「catch-all アームの payload 束縛」の行より scrutinee の参照が
    payload へ移り、A12 より payload と scrutinee の型は等しいので、scrutinee の inhabited な各 leaf が
    payload の同じ path の leaf へ移る。unbox union の変位アームでは、D9 の移動の表の「unbox union の
    変位アームの payload 束縛」の行より活性変位の参照が payload へ移る。D21 と A16 より選ばれたアームの
    `tag` は scrutinee の実行時のタグに等しく、D16 より scrutinee の inhabited な leaf はその変位の下に
    あるものだけなので、どちらの場合も移るのは scrutinee のすべての inhabited な leaf である。
    よって遷移の後 `μ(scrut, λ) = 1 + [scrut ∈ H] - 1 - [scrut ∈ DB_j]` であり、<2>4 と同じ計算で
    `[scrut ∈ Λ(m')]` に等しい。`μ(payload_j, λ'') = 1 - [payload_j ∉ Λ(m')] = [payload_j ∈ Λ(m')]`。
    `DB_j` の他の名前は <2>4 と同じである。
  <2>6. 遷移の中の各節点の入口で `μ ≥ 0` であり、読む値と触れる先のスロットについて `μ ≥ 1` である。
    BY <ref id=8e3aff3/>, <2>0, <2>2, <2>4, <2>5, <ref id=33c54dc/>, <ref id=56c2068/>
    前置 `Retain` が置かれるのは `scrut ∈ H ⊆ Λ(m)` のときであり、そのときその入口で
    `μ(scrut, λ) = 1` である。核節点は scrutinee を読み (D7)、その入口で
    `μ(scrut, λ) = 1 + [scrut ∈ H] ≥ 1`。頭の `Release` 鎖の名前のうち `scrut` は最大 2 回
    現れる -- `DB_j` に入るときと容器解放のときである。両方が起きるのは `scrut ∈ DB_j` のときであり、
    `DB_j = H \ (U_j ∪ M) ⊆ H` (<2>2) よりそのとき `scrut ∈ H` なので前置 `Retain` が置かれ、
    <2>4 より核節点は scrutinee の `μ` を下げないから、核節点の直後の `μ(scrut, λ)` は 2 である。
    よって 2 つの `Release` の入口の値は 2 と 1 である。片方だけのときは入口の値は 1 以上である。`payload_j` と `DB_j` の他の名前は互いに、また `scrut` とも異なり (A6)、
    それぞれ 1 度しか現れないので、その入口の値は 1 である。<2>4 と <2>5 の最終値はすべて非負である。
  <2>6a. (c)。
    BY <ref id=8e3aff3/>, <2>2, <2>4, <2>5, <2>6, <ref id=33c54dc/>,
       CODE src/rc_ir/rc_insert.rs: build_releases
    前置 `Retain` は `μ` を上げるだけなので `k = 0` である。核節点は、<2>4 では scrutinee の `μ` を
    下げず (下げるのは頭の `Release` 鎖である)、<2>5 では移動によって scrutinee の各スロットを 1 回
    だけ下げるので、どちらでも `k_核 ≤ 1` であり、<2>6 よりその入口の値は `1 + [scrut ∈ H] ≥ 1` で
    ある。payload のスロットは上げるだけである。頭の `Release` 鎖の各節点は path が空列 --
    `build_releases` が作る式は `RcExpr::Release(v, vec![], RcState::Unknown, c)` である -- なので
    その名前の各スロットを 1 回ずつ下げ、<2>6 よりその入口の値は 1 以上である -- `scrut` が 2 度
    現れる場合の入口の値は 2 と 1 である。
  <2>7. QED
    BY <2>4, <2>5, <2>6, <2>6a
    `arm_j.tag` は `Some` か `None` のいずれかであり、`Some` の場合は `scrut.ty.is_box` の真偽で
    分かれる。3 つの場合を <2>4 と <2>5 が覆う。

<1>6. **CASE (T3)**。`m` はアーム本体の終端の `Ret(r)`、その `Match` を `Let(x, Match(s, arms), cont)`
      とする。
  <2>1. `A(m) = M` であり、`Λ(m) = M ∪ ({r} ∩ 局所名)`、`Λ(m') = live_cont`、
        `M = live_cont \ {x}` である。
    BY <ref id=66e786e/>, <ref id=b3dfa37/>, <ref id=ca36627/>,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: free_locals, CODE src/rc_ir/rc_insert.rs: insert_if_local
    `insert_into_match` はアーム本体の**根**について `insert_into_expr(arm.body, &live_after_match)` を
    呼ぶので、その根を書き換える呼び出しの `live_after` は `M := live_after_match` である。
    **`m` はアーム本体の終端の `Ret` であって根とは限らない** -- 根から `m` までの伝播が要る。
    `Ret` でない骨格節点 4 種を書き換える 4 つの関数はいずれも継続について
    `self.insert_into_expr(cont, live_after)` を呼ぶので、継続を書き換える呼び出しの `live_after` は
    その節点を書き換える呼び出しのものに等しい。D2 より `Ret` 以外の 5 種はちょうど 1 つの継続を持ち、
    D3 よりアーム本体の終端の `Ret` はその根から継続だけを辿って着く節点なので、その鎖の長さについての
    帰納で `A(m) = M` である。
    アーム本体は `live_after = M` で書き換えられるので `L14` (a) より
    `Λ(m) = free_locals(Ret(r)) ∪ M` であり、`free_locals(Ret(r))` は `insert_if_local` の門より
    `r` が局所名なら `{r}`、そうでなければ空である。
  <2>2. 前置 `Retain` 鎖は `r ∈ M` のとき `Retain(r, [])` を 1 つ置き、`Match` の核節点と `cont` の
        間の `Release` 鎖は `x ∉ live_cont` のとき `Release(x, [])` を 1 つ置く。コードはどちらも
        `needs_rc` で絞り、前者はさらに `r` が局所名であることを要求するが、`needs_rc` が偽の名前は
        スロットを持たず (`L15` (c))、局所名でない名前もスロットを持たない (<1>0) ので、以下の
        勘定では区別しない。
    BY <1>0, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner の `RcExpr::Ret(x)` の腕,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match, <ref id=dca1c02/>
  <2>3. 核節点 `Ret(r)` は `μ(x, λ)` を 1 上げる。`r` が局所名であるときは併せて `r` の inhabited な
        各 boxed leaf について `μ(r, λ)` を 1 下げ、局所名でないときは下げる先を持たない。
    BY <1>0, <ref id=9d74736/>, <ref id=83d98e9/>
    D9 の移動の表の「`Match` のアーム本体の `Ret(x)`」の行。A12 よりアームの結果と `Match` の束縛変数の
    型は等しいので leaf は対応する。`r` が局所名でないときは <1>0 より `(r, λ)` はスロットではないので
    `μ` を下げる項が無い。
  <2>4. QED
    BY <ref id=8e3aff3/>, <1>0, <2>1, <2>2, <2>3, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=56c2068/>
    **`x ≠ r` は A6・A11・D2 のスコープの規則から出る** -- `r` はアーム本体の終端の `Ret` の位置で
    使われる名前であり、A11 よりその使用はその位置でスコープに入っている束縛に解決する。D2 より
    `Let(x, Match(..), cont)` が束縛する `x` のスコープは `cont` の部分木であってアーム本体を含まない
    ので、`r` の使用が `x` の束縛に解決することはなく、A6 よりその名前を束縛するものはプログラム全体で
    1 つだけなので `x ≠ r` である。`M = live_cont \ {x}` なので
    `r ∈ M ⟺ r ∈ live_cont` である。`r` が局所名であるとき、遷移の後
    `μ(r, λ) = 1 + [r ∈ M] - 1 = [r ∈ Λ(m')]` である。`r` が局所名でないときは <1>0 より `r` は
    スロットを持たず、<2>1 より `Λ(m) = M` であり、`r` についての勘定は空である。どちらでも
    `μ(x, λ) = 1 - [x ∉ live_cont] = [x ∈ Λ(m')]` である。他の名前は `Λ(m)` と `Λ(m')` の両方に入るか
    両方に入らないかである。遷移の中の各節点については、前置 `Retain` の入口で `μ(r, λ) = 1`
    (置かれるのは `r ∈ M` のときであり、`M` の名前は局所名である)、核節点は D7 の読む構文の表の
    6 行のどれでもない `Ret` であって読む値を持たないが、`r` を名指すので、`r` が局所名なら
    `μ(r, λ) = 1 + [r ∈ M] ≥ 1` である。`Release(x, [])` の
    入口で `μ(x, λ) = 1` である。値はどこでも非負である。
    (c) について、前置 `Retain` は `μ` を下げないので `k = 0`、核節点は <2>3 より `r` が局所名なら
    その各スロットを 1 回ずつ下げてその入口の値は `1 + [r ∈ M] ≥ 1`、局所名でないなら `k_核 = 0` で
    ある。`Release(x, [])` は `x` の各スロットを 1 回ずつ下げてその入口の値は 1 である。

<1>7. QED
  BY <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### 10.6 `L18` (検査点では `held` は live なスロットの個数に等しい) <!--#0be730a-->

**言明**。`insert_rc` の出力の 1 つの本体、1 つの実行路 `ρ`、`ρ` を辿る 1 つの活性化について、`ρ` の
上の各検査点 `m` において、各スロット `(v, λ)` の `μ` は `[v ∈ Λ(m)]` に等しい。したがって、
`held_ρ(m, C)` が定まる (`DEF 量化の範囲`) 各計数下の
別名類 `C` について `held_ρ(m, C) = N_ρ(m, C)` である。また本体の根の検査点より前にある `Release` 節点
(`insert_into_func` の `unused` の鎖) の各入口で、すべてのスロットの `μ` は非負であり、その `Release`
が触れるスロットの `μ` は 1 である。

**証明**

<1>1. 最初の検査点 -- 本体の根の骨格節点の検査点 -- で言明が成り立つ。
  <2>1. CASE 本体がグローバル初期化子の `init` である。
    BY <ref id=8e3aff3/>, <ref id=3905b4e/>, <ref id=66e786e/>, <ref id=f06144e/>, <ref id=a502f3e/>, CODE src/rc_ir/rc_insert.rs: insert_rc
    D1 より `init` はパラメータも capture も持たないので D10 の初期値は空であり、活性化の開始時の
    `μ` はすべて 0 である。`insert_rc` は `inserter.insert_into_expr(glob.init, &Set::default())` を
    呼び、`EXT 空を作る Default` より `Set::default()` は空の集合なので `A(根) = ∅` であり、
    `L14` (a) より `Λ(根) = free_locals(init)` である。A11 は
    「グローバル初期化子の `init` は自由な局所名を持たない」と述べるので `Λ(根) = ∅` である。
    `insert_rc` はこの本体に `build_releases` を掛けないので、根の前に節点は無い。
  <2>2. CASE 本体が関数の `body` である。
    BY <ref id=8e3aff3/>, <ref id=3905b4e/>, <ref id=66e786e/>, <ref id=f06144e/>, <ref id=dca1c02/>, CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: build_releases
    `L15` (e) より、すべてのパラメータ・capture の unit は所有される。D10 の初期値は、所有する
    各パラメータ・capture の inhabited な各 leaf の `μ` を 1 にする。`insert_into_func` は
    `insert_into_expr(func.body, &Set::default())` を呼び、`EXT 空を作る Default` より
    `Set::default()` は空の集合なので `A(根) = ∅` であり、`L14` (a) より
    `Λ(根) = free_locals(func.body)` である。A11 は「関数の本体の自由な局所名は、その関数のパラメータと
    capture に限る」と述べるので、`Λ(根)` はパラメータと capture の名前だけからなる。
    `insert_into_func` は `func.body = build_releases(unused, body)` を作り、`unused` は
    `self.needs_rc(p) && !live.contains(&p.name)` を満たすパラメータと capture である。ここで
    `live = Λ(根)` である。`L15` (c) より `needs_rc(p)` が偽のパラメータはスロットを持たない。よって `unused` の解放鎖の後、パラメータ・capture の各スロットの `μ` は
    `[p ∈ Λ(根)]` である。`unused` の名前は相異なるので各 `Release` の入口で `μ = 1` であり、
    パラメータ・capture 以外の名前のスロットは値を得ておらず `μ = 0 = [・ ∈ Λ(根)]` である。
  <2>3. QED
    BY <ref id=8e3aff3/>, <2>1, <2>2, <ref id=ff5985d/>, <ref id=33c54dc/>
    D23 より本体は関数の `body` かグローバル初期化子の `init` かのいずれかである。A6 より
    `unused` の名前は相異なる。

<1>2. 各検査点で `μ` は `Λ` の指示関数である。
  BY <1>1, <ref id=18030ac/>
  `ρ` の上の検査点の列についての帰納。基底は <1>1、段は `L17` である。

<1>3. QED
  BY <1>2, <ref id=c2ea78f/>, <ref id=dca1c02/>, DEF 割り当て `μ`
  `L15` (e) より `insert_rc` の出力ではすべての unit が所有されるので、`L13a` (c) が当たり、
  `L13a` (b) は `held_ρ(m, C) = Σ_{(v, λ) ∈ C} μ_m(v, λ)` になる。<1>2 よりこれは
  `Σ_{(v, λ) ∈ C, v ∈ Λ(m)} 1 = Σ_{v ∈ Λ(m)} κ_C(v) = N_ρ(m, C)` である。

### 10.6a `L18a` (終端の `Ret` の前に `Retain` は立たない) <!--#af7088a-->

**言明**。関数本体・グローバル初期化子の終端の `Ret` を書き換える呼び出しの `live_after` は空集合であり、
その検査点の前置 `Retain` 鎖は空である。したがって `insert_rc` の出力の各 `Retain` 節点の入口は、
`L17` (a) が挙げる 3 通りのうち、連続する 2 つの検査点の間の遷移の中の点である。

**証明**

<1>1. 関数本体・初期化子の終端の `Ret(x)` を書き換える呼び出しの `live_after` は空集合であり、
      その検査点の前置 `Retain` 鎖は空である。
  BY <ref id=cd1f6fa/>

<1>2. 本体の根の検査点より前の `Release` 鎖に `Retain` 節点は無い。
  BY CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: build_releases
  `insert_into_func` はこの位置に `build_releases(unused, body)` だけを置き、`build_releases` は
  `RcExpr::Release` の節点しか作らない。

<1>3. QED
  BY <1>1, <1>2, <ref id=18030ac/>
  `L17` (a) より、出力の各節点の入口は、根の検査点より前の `Release` 鎖の中の点か、連続する 2 つの
  検査点の間の遷移の中の点か、終端の `Ret` の検査点かのいずれかである。<1>2 より第 1 の場所に
  `Retain` 節点は無い。第 3 の場所は 1 点であり、<1>1 より前置 `Retain` 鎖が空なのでその点は核節点
  `Ret` の入口であって `Retain` 節点の入口ではない。よって `Retain` 節点の入口は第 2 の場所にある。

### 10.7 `L19` ((O1)) <!--#f941b4f-->

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、次の 4 つが成り立つ。

- **(a)** `ρ` の上の各時点において、各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` である。
- **(b)** D7 の読む構文が読む値の各スロット、および `Retain(v, π)`・`Release(v, π)` が触れる各スロットに
  ついて、そのスロットが属する計数下の別名類 `C` は、その節点の入口で `held_ρ(・, C) ≥ 1` である。
- **(c)** 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った後、各計数下の別名類 `C` について
  `held_ρ(・, C) = 0` である。
- **(d)** `ρ` の上の各節点 `n` と各スロット `s` について、`n` の実行が `μ(s)` を下げる回数を `k_n(s)`
  とすると、`n` の入口で `μ(s) ≥ k_n(s)` である。したがって `n` の実行の途中の各点 (D24 の段内の点)
  において、**その点で `held_ρ` が定まる**各計数下の別名類 `C` について `held_ρ(・, C) ≥ 0` である。

**(c) の点も (a) の粒度の外に在る。** 第 1 節の**時点**は節点の訪問の入口であり、「終端の `Ret` の
消費を行った後」はその 1 つではない。この点を (c) が扱うのは、A19 (ii-a) が
「**非負であることは、終端の `Ret` の消費を行った直後の時点についても言う。**」と、その 1 点を
節点の入口の外に置くからである。**その点で `held_ρ` が定まるのは D34 による** -- D34 の第 1 の箇条は
表の第 6 行 (D9 の消費) の事象を**それを運ぶ素動作の直後の段内の点**に置くので、終端の `Ret` の消費の
直後は段内の点であり、そこで `held_ρ(・, C)` は値を持つ。

**(d) は (a) の粒度の外に在る。** (a) が言うのは節点の入口についてだけであり、A19 (ii-a) が量化する
のもその点集合である (第 1 節)。A19 の (ii-c) -- 節点の実行の途中の各点 (D24 の段内の点) での
非負性 -- を果たすために置くのが (d) であり、
その第 1 文は遷移については `L17` (c) そのもの、残る 2 か所については `L18` から出る。

**量化は A19 (ii-c) と同じである。** README の A19 は (ii-c) を「**(ii-c) (段内の点の非負性)。
節点の実行の途中の各点 (D24 の段内の点) と、その点で `held_ρ` が定まる各計数下の別名類について、
`held ≥ 0` である。**」と書く (第 1 節)。D34 は類の開始の点より前で `held_ρ` に値を与えないので、その点でまだ開始していない類は
(d) の第 2 文の外に在る。

**`μ` は節点の入口でしか定まらない。** `DEF 割り当て μ` が `μ_τ` を定めるのは `ρ` の上の各時点 -- 節点の
訪問の入口 -- についてだけであり、`L13a` (b) の等式もその点についてしか与えられない。よって第 2 文が
段内の点で読むのは `held` だけであり、入口の勘定をその点へ運ぶのは D34 の橋である -- D34 は
「**この 3 つの箇条が置かない点では、`held` は直前の置き場所の値のままである。**」と述べ、
「**したがって、節点の入口で勘定した値は、その節点の実行の途中の各点まで、上の 3 つの箇条が置く分だけを
足し引きして運べる。**」と結ぶ。

**証明**

<1>1. (a)。
  BY <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=18030ac/>, <ref id=0be730a/>, DEF 割り当て `μ`
  `L17` (a) より各節点の入口は 3 通りである。`unused` の解放鎖の中では `L18` が `μ ≥ 0` を与え、
  遷移の中では `L17` (b) が与え、終端の `Ret` の検査点では `L18` が `μ` を `Λ` の指示関数と定める
  ので `μ ≥ 0` である。`L15` (e) と `L13a` (c) より `β ≡ 0` なので、`L13a` (b) は
  `held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ)` であり、これは非負の項の和である。

<1>2. (b)。
  BY <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=18030ac/>, <ref id=0be730a/>, <ref id=56c2068/>, DEF 割り当て `μ`
  `L17` (b) より、遷移の中の各節点の入口で、その節点が読む値のスロットと `Retain`/`Release` が触れる
  スロットは `μ ≥ 1` である。検査点はその遷移の最初の節点の入口なので同じ主張に含まれる。
  `unused` の解放鎖の中の `Release` については `L18` が `μ = 1` を与える。終端の `Ret` の検査点に
  ついては、`x` が局所名なら `L18` よりその節点が名指す `x` のスロットの `μ` は `[x ∈ Λ]` であり、
  `L15` (b) より `x ∈ Λ` なので 1 であり、局所名でないなら `L15` (d) よりスロットが無い (終端の `Ret`
  は D7 の読む構文ではないので、この場合はいずれにせよ言明の対象外である)。
  そのスロット `(v, λ)` が計数下の類 `C` に属するとき、`L15` (e) と `L13a` (b)(c) と <1>1 より
  `held_ρ(・, C) = Σ_{(w, μ') ∈ C} μ(w, μ') ≥ μ(v, λ) ≥ 1` である (ほかの項は <1>1 の議論より
  非負である)。

<1>3. (c)。
  <2>1. 関数本体・初期化子の終端の `Ret(x)` を書き換える呼び出しの `live_after` は空集合であり、
        その検査点の前置 `Retain` 鎖は空である。
    BY <ref id=af7088a/>
  <2>2. QED
    BY <2>1, <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=0be730a/>, <ref id=66e786e/>, <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=9d5d254/>, DEF 割り当て `μ`
    **この点で読むのは `held` だけである** -- `DEF 割り当て μ` が `μ` を定めるのは節点の訪問の入口に
    ついてだけであり、消費を行った後はその 1 つではない。入口の勘定をその点へ運ぶのは D34 であり、
    D34 の第 1 の箇条は表の第 6 行 (D9 の消費) の事象を**それを運ぶ素動作の直後の段内の点**に置く。
    <2>1 と `L14` (a) より `Λ(終端の Ret) = free_locals(Ret(x))` であり、これは `x` が局所名なら
    `{x}`、そうでなければ空である。`L18` よりその検査点 -- これは `ρ` の上の時点である -- で `μ` は
    `Λ` の指示関数であり、`L15` (e) と `L13a` (b)(c) より
    `held_ρ(・, C) = Σ_{(v, λ) ∈ C} μ(v, λ)` である。
    `x` が局所名でないときは、`L15` (d) より `x` はスロットを持たないので、この検査点でどのスロットの
    `μ` も 0 であり、どの計数下の類でも `held_ρ(・, C) = 0` である。D34 の表の第 6 行が数えるのは
    `C` の**スロット**の消費であり、この節点が消費するのは `(x, λ)` の対 -- スロットではなく D6 の
    記号の位置 -- なので、どの類の `held` も動かず、消費の後も 0 である。
    `x` が局所名のときは、この検査点で `μ` は `x` の inhabited な各 boxed leaf のスロットについて 1、
    他のスロットについて 0 なので、計数下の類 `C` について
    `held_ρ(・, C) = #{λ : (x, λ) ∈ C かつ λ は inhabited}` である。D9 の消費の表の
    「本体 (D23) の終端の `Ret(x)`」の行 -- D23 より本体は関数の `body` とグローバル初期化子の `init` の
    両方を指すので、(c) が量化する初期化子の終端の `Ret` もこの行が覆う -- より、この節点は `x` の
    inhabited な各 boxed leaf の参照をちょうど 1 回ずつ消費する。D34 の表の第 6 行がその消費 1 つに
    つき `held_ρ(・, C)` を 1 下げる。この節点は `Retain` でも `Release` でもなく、D2 より変数を
    束縛しないので D10 の生成の表のどの行にも当たらず、D34 の表の他の行に当たる事象を持たない。
    よって消費を行った後 `held_ρ(・, C) = 0` である。

<1>3a. (d) の第 1 文。
  BY <ref id=8e3aff3/>, <ref id=66e786e/>, <ref id=dca1c02/>, <ref id=cd1f6fa/>, <ref id=18030ac/>, <ref id=0be730a/>, <ref id=af7088a/>, <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=ff5985d/>, <ref id=33c54dc/>, DEF 割り当て `μ`,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: build_releases
  `L17` (a) より `ρ` の上の各節点の入口は 3 通りである。
  遷移の中の節点については `L17` (c) がそのまま (d) の不等式を与える。
  本体の根の検査点より前の `Release` 鎖 (`insert_into_func` の `func.body = build_releases(unused,
  body)` が積む鎖) の中では、各節点は path が空列 -- `build_releases` が作る式は
  `RcExpr::Release(v, vec![], RcState::Unknown, c)` である -- なのでその名前の各スロットを 1 回ずつ
  下げ、A6 より鎖の名前は相異なるので、触れる
  スロットについて `k_n(s) = 1`、他のスロットについて `k_n(s) = 0` である。`L18` はその入口で
  触れるスロットの `μ` が 1、他のスロットの `μ` が非負であることを与える。
  関数本体・初期化子の終端の `Ret(x)` の検査点については、`L18a` と `L14` (a) より
  `Λ(終端の Ret) = free_locals(Ret(x))` であり、`L18` よりその検査点で `μ` は `Λ` の指示関数である。
  D9 の消費の表の「本体 (D23) の終端の `Ret(x)`」の行より、この節点は `x` の inhabited な各 boxed
  leaf の参照を 1 回ずつ消費するので `k_n(x, λ) = 1`、他のスロットについて `k_n(s) = 0` であり、
  `L15` (b) より `x` が局所名なら `x ∈ Λ` なので `μ(x, λ) = 1` である。`x` が局所名でないときは
  `L15` (d) より `x` はスロットを持たず、どのスロットについても `k_n(s) = 0` である。

<1>3b. 節点 `n` の実行の途中の各点における `held_ρ(・, C)` は、`n` の入口の値に、`n` がその点までに
       行った D34 の表の第 4 行の事象の個数を足し、第 5 行と第 6 行の事象の個数を引いたものである。
       ただし `held_ρ(・, C)` が `n` の途中で定まり始めるときは、その点の値は 1 であり、そこから先を
       同じ形で足し引きする。
  BY <ref id=e3436e8/>, <ref id=9d5d254/>
  D34 は段内の点における `held` について「**この 3 つの箇条が置かない点では、`held` は直前の置き場所の
  値のままである。**」と述べ、表に行を持たない素動作 -- 割り当て、記憶域を返す解放、グローバル化、
  `Destructor` が作る子の活性化の動作 -- が `held` を動かさないことを挙げ、
  「**したがって、節点の入口で勘定した値は、その節点の実行の途中の各点まで、上の 3 つの箇条が置く分だけを
  足し引きして運べる。**」と結ぶ。D34 の 3 つの箇条のうち、第 1 の箇条が表の第 4・第 5・第 6 行の事象を
  それを運ぶ素動作の直後の段内の点に置き、第 2・第 3 の箇条が開始値を置く。D24 の段内の点は素動作の
  切れ目なので、`n` の実行の途中の点はこの 3 つの箇条が置く事象で区切られる。

<1>3c. `n` の実行が行う D34 の表の第 5 行と第 6 行の事象のうち、`C` のスロットを名指すものの個数は
       `Σ_{s ∈ C} k_n(s)` 以下である。
  BY <ref id=9d5d254/>, DEF 割り当て `μ`
  D34 の表の第 5 行は「`Release(v, π)` であって `(v, λ) ∈ C` である `λ` を `π` の下に持つ」ものに
  その `λ` 1 つにつき `-1` を、第 6 行は「`(w, μ) ∈ C` の D9 の消費」に `-1` を置く。
  `DEF 割り当て μ` の第 4 行は `Release(v, π)` の `π` の下の inhabited な各 leaf について `μ(v, λ)` を
  `-1` し、第 5 行は D9 の消費される inhabited な各 leaf について `-1` するので、前者の事象 1 つは
  `k_n(v, λ)` に、後者の事象 1 つは `k_n(w, μ)` にそれぞれ 1 を寄せる。どちらの事象も `C` のスロットを
  名指すので、その総数は `Σ_{s ∈ C} k_n(s)` を超えない。

<1>3d. `held_ρ(・, C)` が `n` の入口で定まっているとき、`n` の実行の途中の各点で
       `held_ρ(・, C) ≥ 0` である。
  BY <1>3a, <1>3b, <1>3c, <ref id=c2ea78f/>, <ref id=dca1c02/>
  `L15` (e) と `L13a` (c) より `β ≡ 0` なので、`L13a` (b) より `n` の入口 -- これは `ρ` の上の時点で
  ある -- で `held_ρ(・, C) = Σ_{s ∈ C} μ(s)` である。<1>3b より途中の各点の値は入口の値から
  第 5 行と第 6 行の事象の個数を引いたもの以上であり、<1>3c よりその個数は `Σ_{s ∈ C} k_n(s)` 以下で
  ある。よって値は `Σ_{s ∈ C} (μ(s) - k_n(s))` 以上であり、<1>3a より各項は非負である。

<1>3e. `held_ρ(・, C)` が `n` の実行の途中で定まり始めるとき、その点から `n` の出口まで
       `held_ρ(・, C) ≥ 1` である。
  BY <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=b3dfa37/>, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ff5985d/>, <ref id=e3436e8/>, <ref id=30d6238/>, <ref id=9d5d254/>
  D34 が開始値を置く 3 行のうち、パラメータ・capture の leaf についての 2 行が置く点は、D34 の
  第 2・第 3 の箇条より、その類の終端の参照が初期 `Obl(a)` に入る素動作の直後の段内の点か、活性化が
  生きている活性化 (D23) になる点である。**その受け渡しを行う段は、活性化を作る段である。**
  D24 の「活性化の林」は活性化を作る段を 5 種と数え上げる -- (E1)、(E3)、(E7)、オペランドを適用する
  `Llvm` の段 (E2 の一部)、そして (F) の解放が `Destructor` について作る段である。**この 5 種は
  どれも、作られる活性化の節点を実行する段ではない** -- (E1) は環境の段、(E3) と (E7) と
  オペランドを適用する `Llvm` の段は呼び出し元の活性化の節点の段であり、(F) はその解放を起こした
  段の一部である。D24 の (E2) はその活性化自身の節点の段を別に定める。よって `n` -- この活性化の
  節点 -- の実行の途中で開始値が置かれるのは
  D10 の生成の行だけであり、その値は 1 である。`L13a` (e) より、その生成が参照を作る位置のスロット
  `(x, λ)` は `C` の ρ-終端である。D10 の生成の表の 5 行が参照を作る位置は、`Let(x, rhs, k)` の `x`、
  boxed 容器の `Destructure` のフィールド変数、boxed union の変位アームの payload 変数であり、D2 より
  どれも `n` が束縛する変数である。
  `C` の他のスロット `(w, μ)` については、D33 の ρ-歩みが `(w, μ)` から `(x, λ)` へ着く。`L13a` (h)
  より、歩みの 1 歩がスロットへ進むとき進む先の変数は `ρ` の上でいま居る位置の変数より前に値を得る。
  `n` が D34 の第 5 行・第 6 行で名指すスロットの変数は、`Release(v, π)` の `v` と D9 の消費が
  名指す変数であり、どれも `n` の位置で使われる名前なので、A11 と D2 より `n` より前に値を得ている。
  そのような変数から始まる歩みは `n` より前に値を得る変数の位置にしか着かないので、`(x, λ)` には
  着かない。A6 より `x` を束縛するものはプログラム全体で 1 つなので、その変数は `x` とも異なる。
  よって `n` は開始の点より後に `held_ρ(・, C)` を下げる事象を行わず、<1>3b より値は 1 以上のままで
  ある。

<1>4. QED
  BY <1>1, <1>2, <1>3, <1>3a, <1>3b, <1>3d, <1>3e, <ref id=9d5d254/>
  (a) は <1>1、(b) は <1>2、(c) は <1>3 である。(d) の第 1 文は <1>3a である。第 2 文について、
  D34 より段内の点で `held_ρ(・, C)` が定まるのは `C` の開始の点以後であり、その点が `n` の入口より
  前にあるときは <1>3d が、`n` の実行の途中にあるときは <1>3e が非負性を与える。

**(c) が言っているもの。** (c) は D11 の (S-b) を別名類の粒度へ絞ったものである。A19 (ii-a) を活性化の
終わりの 1 点先まで読む読み手が要るのはこの点での `held ≥ 0` であり、(c) はそれを等式の形で与える。
**A19 (ii-b) はこの点へは延びない。** `held = 0` である一方、走査の `RcExpr::Ret` の腕は
`returns_from_func` が真のとき pending の要素を `needed_retains` に入れるだけで `pending` から
取り除かないので (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)、`bumps ≥ 1` のまま
`held = 0` になる時点が在る。第 12 節の `L38` がその形の `insert_rc` の出力を挙げる。

### 10.8 `L20` (各別名類の開始事象は `ρ` の上に高々 1 つ) <!--#371ccc9-->

**言明**。`insert_rc` の出力の各本体、各実行路 `ρ`、`ρ` を辿る各活性化について、各計数下の別名類 `C` の
開始事象 (`DEF 開始事象`) は `ρ` の上に高々 1 つである。

**証明**

<1>1. 開始事象は、パラメータ・capture の inhabited (D16) な leaf についての行と、D10 の
      生成の表の各行である。`insert_rc` の出力ではすべての unit が所有される (`L15` (e)) ので、
      前者は D10 の初期値の行そのものである。
  BY DEF 開始事象, <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=dca1c02/>
  `DEF 開始事象` は D34 の表で `held_ρ(・, C)` に開始値 1 を与える 3 行に当たる事象を開始事象と呼ぶ。
  その 3 行のうちパラメータ・capture の leaf についての 2 行は、その leaf の unit を関数が所有するか
  借用するかを問わず開始値 1 を与える。`L15` (e) より `insert_rc` の出力に借用する unit は無いので、
  その 2 行は D10 の初期値の行そのものである。残る 1 行が D10 の生成の表の各行に当たる。

<1>2. これらの事象が作る参照が属する類の ρ-終端は、その事象の位置のスロット自身である。
  BY <1>1, <ref id=c2ea78f/>
  `L13a` (d) がパラメータ・capture の leaf について、`L13a` (e) が D10 の生成の表の各行について、
  そのスロットが自分の属する類の ρ-終端であることを与える。

<1>3. QED
  BY <ref id=8e3aff3/>, <1>1, <1>2, <ref id=596a46d/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=ca36627/>
  <1>2 より、相異なる開始事象は相異なるスロットを ρ-終端とする -- 事象の位置が相異なれば、そこで値を
  得る変数が相異なる (A6) か、同じ変数の相異なる leaf である。よって相異なる開始事象は相異なる類に
  開始値を与える。**1 つのスロットが `ρ` の上で 2 度値を得ることはない** -- D2 より本体は有限の木で
  節点が自分自身を含むことはなく、D3 の実行路は継続かアーム本体へ進むだけなので、1 本の実行路は
  各節点を高々 1 回訪れる。D6 よりスロットが値を得るのは、その変数を束縛する節点を実行する段か、
  パラメータ・capture については活性化が始まる時点であり、A6 より 1 つの名前を束縛する節点は
  プログラム全体で 1 つだけである。

**`L4` の前提が満たされること。** `L4` は「`ρ` の上で開始事象を 1 つだけ持つ各別名類」について述べる。
`L20` より開始事象は高々 1 つである。**「少なくとも 1 つ」の側を与えるのは `L13a` (f) と (h) と
`DEF 開始事象` の組である** -- 計数下の別名類の ρ-終端はパラメータ・capture の leaf か D10 の生成の
表の位置のいずれかであり (`L13a` (f))、`DEF 開始事象` の 3 行がその 2 種を覆う。**その事象がその類の
スロットが在る時点以前であることは `L13a` (h) が与える** -- ρ-歩みの各段は、進む先の変数がいま居る
位置の変数より前に値を得る位置へ進むからである。よって `ρ` の上に
スロットを持つ計数下の類は開始事象を 1 つ持つ。`ρ` の上にスロットを持たない類は勘定の対象に
ならない (D6 -- スロットの変数はその実行路の上で値を得ている)。`L25` `<1>2c` がこの 2 つを
合わせた形で `L4` の前提を渡す。

## 11. (O2) の証明 -- 帳簿は処分に遅れない

(O2) は「`bumps ≥ 1` である各時点で `held ≥ 1 + bumps`」である (第 8 節)。この節はそれを示す。
`L7` が与えるのは、その形と `U + X ≥ D` の同値である。
支えるのは、**1 つの別名類のスロットに付く名前の全体 `Ids(C)` が作る木**についての不等式である。
`origin` の `identity` はスロットごとに 1 つの名前を与え、1 つの別名類の中でその名前は
`Binding::Join` が候補を 2 つ以上持つ位置で切り替わる。切り替わりの関係 `Anc` が `Ids(C)` を木にし
(`L21`)、走査の帳簿はその木の各名前に付く。示す不等式は **「1 つの部分木に付いている bump の総和が 1 以上で
あるならば、それはその部分木が保持している参照の個数より真に小さい」**(`L24`) であり、木の根で読むと
それがちょうど「`bumps ≥ 1` ならば `held ≥ 1 + bumps`」になる。**仮説を落とすと偽になる** --
`Bsub = Down = 0` の名前で `0 ≤ -1` を要求することになる。

この節が本体について読む量的な前提は、11.1 節の (S) が挙げる 5 つだけである。`insert_rc` の
出力がその 5 つを満たすことは `L20a` が第 10 節の `L15`・`L17`・`L18`・`L18a`・`L19`・`L20` から出す。
前提の形で
書くのは、`split_rc_units` の出力についても同じ 5 つを確かめれば同じ結論が出るからである
(第 13 節の `L32`)。構文と名前の規律 -- A6・A11・D2 -- は (S) の外に在り、この節の段が本体について
直に読む (第 8.1 節)。

### 11.1 名前の鎖、名前の木、帳簿

本体 `B` と、その 1 つの活性化と実行路 `ρ` と計数下の別名類 `C` を固定する。以下、`C` のスロットを単に
**スロット**と書く。**`B` の `VarTable` の値も併せて固定する** -- `cancel` は `prog.funcs` の各関数に
`VarTable::of(f)` を、各グローバル初期化子に `VarTable::body_only(&g.init)` を作り、その 1 つを
`CancelAnalysis` の `vars` に据えて走査を回す (`CODE src/rc_ir/borrow.rs: cancel`)。`nm(s)` はその表を
第 1 引数として `origin` が返す値の `identity` である。

`nm(s)` は第 1 節の `DEF nm(s)` である。

**DEF `Anc(s)`**。D33 の ρ-歩みは、`s` から
ρ-終端まで辿るスロットの列 `s = s_0, s_1, …, s_n` である。`Anc(s) := {nm(s_0), nm(s_1), …, nm(s_n)}`
と置き、**`s` の名前の鎖**と呼ぶ。

**DEF `Ids`**。`Ids(C) := {nm(s) : s はスロット}` と置く。
**DEF `Sub`**。`id ∈ Ids(C)` について `Sub(id) := {nm(s) : s はスロットで id ∈ Anc(s)}` と置く。

**DEF `Down、Bmp、Bsub`**。`ρ` の上の時点 `τ` と `id ∈ Ids(C)` について
`Down_τ(id) := Σ_{s : id ∈ Anc(s)} μ_τ(s)`、
`Bmp_τ(id) := Σ_{p ∈ pending} B(p, ρ)[id]` (D27)、
`Bsub_τ(id) := Σ_{id' ∈ Sub(id)} Bmp_τ(id')` と置く。

上の 3 つの集合 `Anc(s)`・`Ids(C)`・`Sub(id)` と、スロットから名前への写像 `nm` は、`ρ` の上に
**いつか**存在するすべてのスロットを走る。**これらが時点に依らないこと、まだ値を得ていないスロットが
`μ_τ = 0` を寄せること、`Ids(C)` の名前に bump が付くにはその名前のスロットが値を得ていることが
要ること、そして `Ids(C)` の名前を持つ計数下のスロットが `C` のスロットに限ることは、`L20b` が
示す。** 時点に依るのは `μ_τ`・`Down_τ`・`Bmp_τ`・`Bsub_τ` の 4 つだけである。

**前提 (S)**。この節の命題が本体 `B` とそれが属するプログラムについて読む量的な前提は、次の 5 つ
だけである。

- **(S0)** `ρ` の上の各時点において、`B` の各スロットの `μ` は 0 以上である。
- **(S1)** そのプログラムのすべての関数の `borrowed_units` が空である。D14 より、すべての
  パラメータ・capture の unit はその関数が所有する。
- **(S2)** `B` の各 `Retain(v, π)` 節点の入口において、その節点が触れる各スロット -- `π` の下の
  inhabited (D16) な各 leaf `(v, λ)` -- の `μ` は 1 以上である。
- **(S3)** `ρ` の上で、各計数下の別名類に `held_ρ(・, ・)` の開始値 1 を与える事象は高々 1 つである。
- **(S4)** `ρ` の上の各節点 `n` と各スロット `s` について、`n` の実行が `μ(s)` を下げる回数を
  `k_n(s)` とすると、`n` の入口で `μ(s) ≥ k_n(s)` である。

**(S4) が (S0) と別に要る理由。** (S0) が縛るのは節点の訪問の入口だけであり、`L24` が帰納を回すのは
1 つの節点が行う事象の**列**の切れ目の上である (`<1>2a`)。その切れ目は節点の入口とは限らないので、
そこで `μ` の非負性を要る段は (S0) を引けない。(S4) は `L19` (d) の第 1 文と同じ形であり、そこから
節点の中の各切れ目での非負性が出る。

### 11.1a `L20a` (`insert_rc` の出力は (S) を満たす) <!--#0d8ae2f-->

**言明**。`insert_rc` の出力の各本体は (S0)・(S1)・(S2)・(S3)・(S4) を満たす。

**証明**

<1>0. (S0)。
  BY <ref id=18030ac/>, <ref id=0be730a/>
  `L17` (a) より `ρ` の上の各節点の入口は 3 通りである。本体の根の検査点より前の `Release` 鎖の中では
  `L18` が「すべてのスロットの `μ` は非負」を与え、連続する 2 つの検査点の間の遷移の中では `L17` (b) が
  同じことを与え、関数本体・初期化子の終端の `Ret` の検査点では `L18` が `μ` を `Λ` の指示関数と定める
  のでどのスロットでも 0 か 1 である。**`L17` (b) の前件はここで解除される** -- (b) が遷移について
  述べるのは「その遷移が始まる検査点 `m` で各スロット `(v, λ)` の `μ` が `[v ∈ Λ(m)]` である」ことを
  前提としてであり、`L18` がその等式を `ρ` の上の各検査点について与える。

<1>1. (S1)。
  BY <ref id=dca1c02/>
  `L15` (e) が (S1) そのものである。

<1>2. (S2)。
  BY <ref id=18030ac/>, <ref id=0be730a/>, <ref id=af7088a/>
  `L18a` より、出力の各 `Retain` 節点の入口は連続する 2 つの検査点の間の遷移の中の点である。
  `L17` (b) の後半 -- 遷移の中の各節点の入口で `Retain`/`Release` が触れる各スロットの `μ` が 1 以上で
  あること -- は、その遷移が始まる検査点 `m` で「各スロット `(v, λ)` の `μ` は `[v ∈ Λ(m)]` である」が
  成り立つことを前提とする。`L18` がその前提を各検査点について与える。

<1>3. (S3)。
  BY <ref id=371ccc9/>
  `L20` が (S3) そのものである。

<1>3a. (S4)。
  BY <ref id=f941b4f/>
  `L19` (d) の第 1 文が (S4) そのものである。

<1>4. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>3a

### 11.1b `L20b` (11.1 節の量についての 4 つ) <!--#2a415a3-->

**言明**。11.1 節が固定する本体・活性化・実行路 `ρ`・計数下の別名類 `C` について、次の 4 つが
成り立つ。

- **(a)** 各スロット `s` の `nm(s)` と `Anc(s)`、および `Ids(C)` と各 `Sub(id)` は、時点に依らない。
- **(b)** 時点 `τ` においてまだ値を得ていないスロット `s` は `μ_τ(s) = 0` である。
- **(b2)** `Bmp_τ(id) ≥ 1` である `id ∈ Ids(C)` については、`τ` までに走査が訪れたある `Retain` 節点の
  入口において、`nm(s) = id` である `C` のスロット `s` が既に値を得ている。
- **(c)** `origin(w, μ).identity()` が `Ids(C)` の元であり、`obj(w, μ)` が計数下 (D26) である
  `ρ` の上のスロット `(w, μ)` は、`C` のスロットである。

**証明**

<1>1. (a)。
  BY <ref id=596a46d/>, <ref id=3f68b95/>, <ref id=d59f90b/>, <ref id=30d6238/>, <ref id=b1f6e13/>, <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=3905b4e/>,
     CODE src/rc_ir/borrow.rs: cancel,
     CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only
  `nm(s) = origin(v, λ).identity()` であり、`origin` の答えは鍵ごとに 1 つに決まって
  `vars.origins` が保持する memo の状態に依らない (P2a)。**P2a の制限がここで満たされる。**
  P2a は「**1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の値を固定する。**」と述べ、
  「**`vars` は、A6 と A11 を満たすプログラムの本体について `VarTable::of` か `VarTable::body_only` が
  作った表である。**」と限る。11.1 節が固定するのは 1 つの本体とその表であり、`cancel` はその表を
  関数については `VarTable::of(f)`、グローバル初期化子については `VarTable::body_only(&g.init)` で
  作って `CancelAnalysis` の `vars` に据える。A6 と A11 は `borrow_ify` の入力に
  かかる仮定だが、A6 と A11 の脇と A2 の「**したがって、`borrow_ify` の入力について語る仮定は、
  `insert_rc` の入力と出力についても読める。**」により、この文書が扱う本体についてもそれを読める。
  ρ-歩みは D33 の規則で決まり、その各段の行き先は D17 が定める -- `Binding::Join` の辺だけが
  活性化の選んだアームに依り、活性化と `ρ` は固定されているので、`s` から ρ-終端までのスロットの列は
  1 つに決まる。よって `Anc(s)` も決まる。`Ids(C)` と `Sub(id)` はスロットの全体 -- `ρ` の上に
  いつか存在するもの (D6) -- を走る和なので、これも時点に依らない。

<1>2. (b)。
  BY <ref id=c2ea78f/>, <ref id=596a46d/>, DEF 割り当て `μ`
  `L13a` (g) より `μ` を動かす事象は `DEF 割り当て μ` の 6 種で尽き、その 6 種はいずれもその位置の
  スロットについて値を動かす。D6 よりスロットはその変数が
  値を得た後に在るので、まだ値を得ていないスロットを動かす事象は `ρ` の上に無い。`μ` の初期値は
  活性化の開始時に 0 である。

<1>3. (c)。
  BY <ref id=4910eaa/>, DEF `Ids`
  `Ids(C)` の元は `C` のあるスロット `s_0` の `identity` である (11.1 節の定義)。`L5b` を `C` と
  `s_0` に当てると、`obj(w, μ)` が計数下であって `origin(w, μ).identity() = nm(s_0)` である `ρ` の上の
  スロット `(w, μ)` は `C` のスロットである。

<1>4. (b2)。
  BY <1>1, <1>3, <ref id=596a46d/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=8093b68/>
  D27 より要素が `pending` に入るのは `Retain(v, π)` 節点の訪問だけであり、そのとき押し込まれる要素の
  `B(p, ρ)` は `π` の下の inhabited (D16) かつ計数下 (D26) の各 leaf を `origin` の `identity` で
  名付けて数えたものである。D27 の残る 2 つの箇条は `B(p, ρ)` を引くか運ぶか定めないかなので、
  `Bmp_τ(id) ≥ 1` には、`τ` までのある `Retain` 節点の訪問がその名前に量を積んだことが要る。その
  訪問の入口の時点で `identity` が `id` である inhabited かつ計数下の leaf -- D6 のスロット -- が
  在り、`id ∈ Ids(C)` のとき <1>3 よりそれは `C` のスロットである。D6 よりそのスロットはその変数が
  値を得た後に在るので、その時点でそのスロットは既に値を得ている。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4

### 11.2 `L21` (名前の鎖の形) <!--#347156f-->

**言明**。各スロット `s` について次の 5 つが成り立つ。

- **(a)** `Anc(s) ⊆ acted_on(s)`。ここで `acted_on(s) := origin(v, λ).acted_on()` の元の集合である
  (D15)。
- **(b)** `Anc(s)` は `nm(s)` だけで決まる。以下 `Anc(nm(s))` とも書く。
- **(c)** `nm(s) ∈ Anc(s)` であり、ρ-終端 `t` の名前 `id_0 := nm(t)` は `Anc(s)` に入る。
- **(d)** `id ∈ Anc(s)` ならば `Anc(id) ⊆ Anc(s)` である。したがって `Anc(id)` の元は
  「`Anc` に含まれる」の関係で線形順序をなし、`Ids(C)` は `id_0` を根とする木をなす。
- **(e)** D9 の移動の表の辺の移動元 `s` と移動先 `s'` について、`Anc(s') = Anc(s)` (したがって
  `nm(s') = nm(s)`) であるか、`Anc(s') = {nm(s')} ∪ Anc(s)` かつ `nm(s') ∉ Anc(s)` であるかの
  いずれかである。後者になるのはアーム本体の `Ret` の辺で、`Match` の束縛変数の `origin` が候補を
  2 つ以上持つときに限る。

**証明**

<1>1. ρ-歩みの 1 歩で `origin` の値がどう変わるかは、`origin_inner` の 6 つの腕で尽きる。
  BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Binding, <ref id=d59f90b/>
  `Binding` は 7 個の構成子を持ち (`Param`、`Move`、`Llvm`、`Producer`、`Field`、`Payload`、`Join`)、
  `origin_inner` の `match` は、束縛の無い場合 (`None`) と `Param` と `Producer` を 1 つの腕に
  まとめた 6 つの腕で、この 8 通りを覆う。

<1>2. 腕は 2 群に分かれる。**止まる腕**では `origin` は `here()` を返し、ρ-歩みはそこで終わる。
      **辿る腕**では `origin` は次のスロットの `origin` を返し、`identity` はそのまま受け継がれる。
      **`Join` の腕**だけが第 3 の形であり、候補が 2 つ以上のとき `identity` を `here()` に取り替えた
      うえで、その活性化が選んだアームの結果へ辿る。
  BY CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: Origin::of_candidates,
     CODE src/rc_ir/ownership.rs: Origin, CODE src/rc_ir/ownership.rs: Origin::acted_on,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: origin_from_leaves_under, <ref id=3f68b95/>, <ref id=cbc4a1c/>, <ref id=d59f90b/>, <ref id=e11772a/>,
     前提 `Origin::Join` の在りか, EXT 空の列の第 1 元
  止まる腕: `None`、`Binding::Param`、`Binding::Producer`、`Binding::Field` の容器が boxed の枝、
  `Binding::Payload` の `Some(_)` かつ scrutinee が boxed の枝、`Binding::Llvm` の
  `as_arg_projection` が `None` を返す枝 (スロットの path は boxed leaf なので `leaf_origins_at` は
  その leaf 自身の宣言を返し、A3 よりこのコミットの宣言は単一の `Fresh`・単一の `Unknown`・空集合の
  いずれかである。前 2 者では D17 の第 2 項より鎖がそこで止まり、空集合のときは
  `origin_from_leaves_under` が `reached` に元を 1 つも積まずに `None` を返すので
  (`EXT 空の列の第 1 元`)、`origin_inner` が `unwrap_or_else(here)` で自分自身を答える。いずれでも
  `origin` を呼ばない)。
  辿る腕: `Binding::Move`、`Binding::Field` の容器が unbox の枝、`Binding::Payload` の `None` の枝と
  `Some(tag)` かつ scrutinee が unbox の枝、`Binding::Llvm` の単一 `Arg` の枝。いずれも
  `origin(次のスロット)` をそのまま返す。
  `Binding::Join` の腕は、各アームの結果の `origin` の `acted_on()` の元をすべて集めた集合
  `candidates` について `of_candidates(candidates, here)` を返す。候補が 2 つ以上のとき返り値は
  `Join { identity: here, candidates }` であり、`identity` はそのスロット自身である。
  候補が 1 つのとき返り値は `Exactly(その候補)` であり、それは**選ばれたアームの結果の `origin` の値
  そのもの**である -- `origin` が返す `Origin::Join` の値は `of_candidates` が `candidates.len()` が
  2 以上のときにだけ作るものであり (`Origin::Join` を構成する式はその 1 つだけである)、
  `Origin::acted_on` は `identity()` を先頭に、それと異なる `candidates()` の元を続けた列なので、
  そのとき `acted_on()` は 2 元以上を持つ。**`Origin::Join` を構成する式が 1 つだけであることは
  `前提 Origin::Join の在りか` が与える** -- 走査が挙げる 3 つの項目のうち構成の式を持つのは
  `Origin::of_candidates` だけであり、残る 2 つはパターンである。ところが `candidates` は全アームの `acted_on()` の和であって
  1 元なので、選ばれたアームの結果の `origin` の `acted_on()` も 1 元であり、その値は `Join` では
  ありえず `Exactly` である。`Exactly(p)` の `acted_on()` は `[p]` なのでその `p` が唯一の候補であり、
  返り値と一致する。よってこの場合は辿る腕と同じ形になる。
  D17 より ρ-歩みは選ばれたアームの結果へ進む。

<1>3. (c)。
  BY <1>2
  ρ-歩みの各歩で `identity` は変わらないか (辿る腕、候補 1 つの `Join`)、そのスロット自身に
  取り替わるか (候補 2 つ以上の `Join`)、そこで止まる (止まる腕) かのいずれかである。よって `nm(s)` は
  歩みの上のどれかのスロットの位置であり、`Anc(s)` の元である。ρ-終端 `t` は歩みの最後の元なので
  `id_0 = nm(t) ∈ Anc(s)`。

<1>4. (a)。
  BY <1>2, <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: origin_inner
  歩みの長さについての帰納。長さ 0 (止まる腕) では `Anc(s) = {nm(s)}` であり
  `acted_on(s) ∋ identity()` なので成り立つ。辿る腕と候補 1 つの `Join` では `origin(s)` が次の
  スロットの `origin` に等しいので `Anc(s) = Anc(次)` かつ `acted_on(s) = acted_on(次)` であり、
  帰納法の仮定がそのまま渡る。候補 2 つ以上の `Join` では `Anc(s) = {nm(s)} ∪ Anc(次)` であり、
  `acted_on(s) = {nm(s)} ∪ candidates` で `candidates` は各アームの結果の `acted_on()` の和なので
  (`origin_inner` の `Binding::Join` の腕)、選ばれたアームの結果について
  `Anc(次) ⊆ acted_on(次) ⊆ candidates` が帰納法の仮定から出る。

<1>5. (b)。
  BY <1>2, <1>3
  <1>3 の議論より、`nm(s)` は歩みの上のスロット `s_k` の位置であり、`s_k` は「`origin(s_k)` の
  `identity` が `s_k` 自身である」最初のスロットである。歩みの `s_0` から `s_k` までの区間では
  `identity` は変わらないので `Anc(s)` のうち `nm(s)` より前の元は無く、`Anc(s) = {nm(s)} ∪ Anc(s_{k+1})`
  (`s_k` が候補 2 つ以上の `Join` のとき) または `Anc(s) = {nm(s)}` (`s_k` が止まる腕のとき) である。
  どちらも `s_k`、すなわち `nm(s)` だけで決まる。

<1>5a. ρ-歩みの上で、候補が 2 つ以上の `Join` の腕を取るスロット `s_k` について、`nm(s_k)` は
       `Anc(s_{k+1})` に入らない。したがって `Anc(s_k) = {nm(s_k)} ∪ Anc(s_{k+1})` は
       `Anc(s_{k+1})` を真に含む。
  BY <1>2, <1>5, <ref id=c2ea78f/>, <ref id=596a46d/>, <ref id=30d6238/>, <ref id=cb35ab1/>, <ref id=8e3aff3/>, CODE src/rc_ir/ownership.rs: origin_inner
  <1>2 より、候補が 2 つ以上の `Join` では `identity` は `s_k` 自身の位置に取り替わるので
  `nm(s_k) = s_k` であり、`s_k` はその `Match` の束縛変数 `x` の leaf である。
  `Anc(s_{k+1})` の元は `s_{k+1}` の歩みの上の位置である (<1>5、D33)。`L13a` (h) より、歩みの 1 歩が
  スロットへ進むとき進む先の変数は `ρ` の上でいま居る位置の変数より前に値を得るので、`s_{k+1}` の
  歩みの上のどのスロットの変数も、`s_{k+1}` の変数以前に値を得る。`s_k` から `s_{k+1}` への 1 歩も
  同じ向きなので、`s_{k+1}` の変数は `x` より前に値を得る。もし `s_{k+1}` の歩みの上のあるスロットの
  変数が `x` であれば、`x` は自分より前に値を得ることになって矛盾する。よってその歩みの上のどの
  スロットの変数も `x` ではない。
  束縛を持たない名前を終端とする位置は記号の位置であり (D6)、A13 よりその名前は局所名でなく、
  D6 より `x` は局所名なので、両者は異なる。**A13 を `insert_rc` の入力と出力について読めるのは
  A2 による** (第 1 節の規則) -- A13 の範囲は `borrow_ify` の入力であり、A2 が
  「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の入力と出力についても
  読める。**」を与える。したがって `nm(s_k) ∉ Anc(s_{k+1})` である。

<1>6. (d)。
  BY <1>3, <1>5, <1>5a, <ref id=b3dfa37/>, <ref id=0594f24/>, <ref id=596a46d/>
  `id ∈ Anc(s)` のとき、`Anc(s)` の定義より `id = nm(s_j)` であるスロット `s_j` が `s` の歩みの上に
  ある。`s` の歩みの `s_j` 以降は `s_j` の歩みそのものなので `Anc(s_j) ⊆ Anc(s)` であり、(b) より
  `Anc(s_j) = Anc(id)` である。
  `Anc(s)` の相異なる 2 つの元は、歩みの上の相異なる位置で立つ名前である (<1>5)。<1>5a より、名前が
  切り替わる各段で `Anc` は真に大きくなるので、後で立つ名前 `id'` の `Anc(id')` は先に立つ名前の
  `Anc` の**真**部分集合である。よって `Anc(s)` の元は「`Anc` の包含」で線形順序をなし、相異なる
  2 つの元は相異なる `Anc` を持つ。すべての `Anc` は `id_0` を含む (<1>3) ので、`Ids(C)` の上の関係
  「`id' ∈ Anc(id)`」は反射的・推移的であり、いま示したとおり反対称でもある -- すなわち `id_0` を
  最大元とする半順序であり、各 `id` の上側 `Anc(id)` は鎖である。
  **そこから木が出る一歩を書く。** `Ids(C)` は有限である -- D2 より本体は有限の木なので、`ρ` の上に
  値を得る変数は有限個であり、各変数の型の boxed leaf も有限個 (D4) なので、`C` のスロットは有限個で
  あり、`Ids(C)` はその像である。有限半順序で、最大元 `id_0` を持ち、各元 `id` の上側 `Anc(id)` が
  鎖であるとき、`id ≠ id_0` の各元について `Anc(id) \ {id}` は空でない有限の鎖なので最小元を
  ちょうど 1 つ持つ。それを `id` の**親**と置く。親の `Anc` は `Anc(id)` の真部分集合なので
  `|Anc(・)|` は親へ進むごとに真に減り、有限性より有限回で `id_0` に着く。すなわち各元から
  `id_0` への道がちょうど 1 本あり、`Ids(C)` は `id_0` を根とする木をなす。

<1>7. (e)。
  BY <ref id=8e3aff3/>, <1>2, <1>5, <1>5a, <ref id=9d74736/>, <ref id=9c7c27a/>, <ref id=33c54dc/>, CODE src/rc_ir/ownership.rs: collect_bindings
  D9 の移動の表は 6 行を持つ。`Let(x, Var(y), k)` の移動先は `Binding::Move`、unbox 容器の
  `Destructure` の名前付きフィールドは `Binding::Field` の unbox の枝、unbox union の変位アームの
  payload 束縛は `Binding::Payload` の `Some(tag)` かつ scrutinee が unbox の枝、catch-all アームの
  payload 束縛は `Binding::Payload` の `None` の枝、`Llvm` の素通し leaf は `Binding::Llvm` の
  単一 `Arg` の枝に束縛される (`collect_bindings`)。この 5 つは <1>2 の辿る腕であり、`origin(s')` は
  `origin(s)` に等しいので `Anc(s') = Anc(s)` である。
  残る 1 行 -- `Match` のアーム本体の `Ret(x)` -- の移動先は `Binding::Join` である。候補が 1 つの
  ときは <1>2 より辿る腕と同じ形で `Anc(s') = Anc(s)`、候補が 2 つ以上のときは `nm(s')` が `s'` 自身
  であり、<1>5 の展開より `Anc(s') = {nm(s')} ∪ Anc(s)` である。`nm(s') ∉ Anc(s)` は <1>5a である
  -- `s'` の歩みは `s'` から `s` へ進むので、<1>5a の `s_k` が `s'`、`s_{k+1}` が `s` である。

<1>8. QED
  BY <1>3, <1>4, <1>5, <1>6, <1>7

**`Sub` と `Down` の言い換え。** (b) より `id ∈ Anc(s)` と `nm(s) ∈ Sub(id)` は同値である
(`nm(s) ∈ Sub(id)` はあるスロット `s''` について `nm(s'') = nm(s)` かつ `id ∈ Anc(s'')` であること
であり、(b) より `Anc(s'') = Anc(s)`)。よって `Down_τ(id) = Σ_{s : nm(s) ∈ Sub(id)} μ_τ(s)` である。

### 11.3 `L22` (木の根で読むと `held` と `bumps` になる) <!--#045d249-->

**言明**。`held_ρ(τ, C)` が定まる (`DEF 量化の範囲`) `ρ` の上の各時点 `τ` について
`Down_τ(id_0) = held_ρ(τ, C)` かつ `Bsub_τ(id_0) = bumps_ρ(τ, C)` である。

**証明**

<1>1. `Down_τ(id_0) = held_ρ(τ, C)`。
  BY <ref id=347156f/>, <ref id=c2ea78f/>, DEF 割り当て `μ`, 前提 (S1)
  `L21` (c) よりすべてのスロット `s` について `id_0 ∈ Anc(s)` なので、`Down_τ(id_0)` は `C` の全
  スロットにわたる `μ` の和である。前提 (S1) よりすべての unit が所有されるので `L13a` (c) が当たり、
  `L13a` (b) よりその和は `held_ρ(τ, C)` である。

<1>2. `Sub(id_0) = Ids(C)`。
  BY <ref id=347156f/>
  `L21` (c) より各スロット `s` について `id_0 ∈ Anc(s)` であり、`Sub(id_0)` の定義よりその `nm(s)` は
  `Sub(id_0)` に入る。

<1>3. QED
  BY <1>2, <ref id=9f1cf6c/>, <ref id=8093b68/>, <ref id=4910eaa/>, DEF `Ids`
  A19 (ii-b) は `bumps` の帰属を定める -- 「`bumps_ρ(τ, C)` とは、時点 `τ` の `pending` の各要素 `p`
  について D27 が定める `B(p, ρ)` のうち、`C` のスロットの `origin` の identity に付く分の総和で
  ある」。11.1 節の `Ids(C)` は `C` のスロットの `identity` の
  全体なので、その総和は `Σ_{id ∈ Ids(C)} Bmp_τ(id)` に等しい (`L5b` より、`Ids(C)` の名前が付く
  bump は `C` のスロットの bump に限る)。<1>2 より `Sub(id_0) = Ids(C)` なので、右辺は
  `Bsub_τ(id_0)` である。

### 11.4 `L23` (処分の事象に対する走査の応答) <!--#62d00c2-->

**DEF 処分の事象**。走査の次の 2 種の事象と、それが `ρ` の上で対にする実行時の事象との組を、
**処分の事象**と呼ぶ。

- `CancelAnalysis` の `consume` の 1 回の呼び出し (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume`)
  と、それに対になる D9 の消費 1 つ。
- `RcExpr::Release` の腕の 1 回の訪問 (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`) と、
  それに対になる `Release(v, π)` の実行時の処分。

**その対が付くことは、この命題の言明の (0) が述べる。**

**関数本体・初期化子の終端の `Ret` の消費は、この定義の外に落ちる。** `walk_inner` の
`RcExpr::Ret` の腕は `returns_from_func` が真のとき `needed_retains` に入れるだけで `pending` を
変えない。この消費の後には `ρ` の上に時点が無いので、`L24` と `L25` の言明はそこを扱わない
(`L19` (c) と `L38`)。

**言明**。(S1) の下で次が成り立つ。

- **(0)** `ρ` の上の D9 の消費のうち、関数本体・初期化子の終端の `Ret` の消費でないものそれぞれに、
  同じ leaf を名指す走査の `consume` の呼び出しがちょうど 1 つ対応する。相異なる消費には相異なる
  呼び出しが対応する。**逆向きは主張しない** -- 走査は `Match` の両方のアームを歩くので、`ρ` が
  選ばなかったアームの中の `consume` の呼び出しには対応する `ρ` の上の消費が無い。`ρ` の上の各
  `Release(v, π)` 節点の実行時の処分には、走査の `RcExpr::Release` の腕の訪問がちょうど 1 つ対応し、
  相異なる処分には相異なる訪問が対応する。
- 1 つの処分の事象と各 `id ∈ Ids(C)` について、その事象が `Down(id)` を減らす量を
  `d` とすると、次のどちらかが成り立つ。
  - **(i)** その事象で `Bsub(id)` は `d` 以上減る。
  - **(ii)** 事象の後、その事象が処分したスロットのうち `id ∈ Anc(s)` であるもの `s` について、
    `Anc(s) ∩ Sub(id)` の各名前 `id'` は `Bmp(id') = 0` である。

**証明**

<1>0. どの pending の要素 `p` と名前 `id'` についても `B(p, ρ)[id'] ≥ 0` であり、したがって
      `Bmp(id')` と `Bsub(id')` も 0 以上である。要素を `pending` から取り除くこと、および `un_bump` が
      `InBracket` で 1 つの要素の `B` から引くことは、どの `id'` についても `Bmp(id')` を増やさない。
  BY <ref id=2340772/>, <ref id=8093b68/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: un_bump
  `L5a` (b) が第 1 文を与え、`Bmp` と `Bsub` はその非負の項の和である。`consume_objects` は要素を
  取り除くだけであり、取り除かれた要素の `B` は和から落ちるので、非負の項が 1 つ減る。`un_bump` の
  `InBracket` は 1 つの要素の `B` から引くだけである。

<1>0a. (0)。
  BY 前提 (S1), 前提 走査が `consume` を呼ぶ位置, <ref id=9d74736/>, <ref id=ef8efc4/>, <ref id=ff5985d/>, <ref id=e885aa0/>,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: cancel,
     CODE src/rc_ir/ownership.rs: rhs_consumes,
     EXT `Iterator::filter_map`,
     CODE src/rc_ir/ownership.rs: all_owned_units,
     CODE src/rc_ir/ownership.rs: passthrough_arg_leaves,
     CODE src/rc_ir/ownership.rs: destructure_consumes
  `CancelAnalysis::consume` を呼ぶ式の在りかは `前提 走査が consume を呼ぶ位置` が与える -- 走査が
  挙げるのは `consume_rhs` と `walk_inner` の 2 項目であり、前者は `rhs_consumes` が `consumed` に
  積む各 `(var, leaf)` について、後者は `Destructure` の腕が `destructure_consumes` の挙げる各 leaf に
  ついて、どちらも 1 つの leaf につき 1 回呼ぶ。`RcExpr::Ret` の腕も `Retain`/`Release`/`Eval` の腕も
  `consume` を呼ばない。
  D9 の消費の表の 6 行のうち、`App`・`Closure`・`Llvm` の 3 行は `Let` の右辺に、`Destructure` の
  2 行は `Destructure` の節点に、残る 1 行は本体の終端の `Ret` に付く。
  **`App` の行の所有は両側で常に真である。** `consume_rhs` が `rhs_consumes` に渡す `owns` は
  `self.owned_units.contains(&(p.name.clone(), truncate_to_unit(…)))` であり、`cancel` は
  `owned_units` に `all_owned_units(prog, type_env)` を置く。D14 より `all_owned_units` が返すのは
  各関数のパラメータ・capture の unit のうち `borrowed_units` に入らないものであり、前提 (S1) より
  `borrowed_units` はどの関数でも空なので、その集合はパラメータ・capture の全 unit である。よって
  `owns` は常に真であり、D9 の `App` の行が言う所有 -- D23 よりその呼び出し先は
  プログラムの `funcs` の関数である -- も常に真である。
  **`Llvm` の行は `passthrough_arg_leaves` が決める。** `rhs_consumes` の `Llvm` の腕は
  `borrows_operand(i)` が真のオペランドを飛ばし、残るオペランドの leaf のうち
  `passthrough.contains(&(i, leaf))` でないものを積む。その `passthrough` は
  `passthrough_arg_leaves` が `decl.leaves().filter_map(as_arg_projection)` で作る集合であり、
  `EXT Iterator::filter_map` よりそれは `as_arg_projection` が `Some` を返す leaf の分だけを集めた
  もの、すなわち結果のいずれかの leaf が単一の `Arg(i, σ)` として素通しを宣言している `(i, σ)` の
  全体である。
  `L16` (a) より借用オペランドの leaf は素通しを
  宣言されないので、D9 の消費の表の `Llvm` の行が挙げる leaf -- `borrows_operand(i)` が偽の
  オペランドのうち素通しを宣言されていない leaf -- は `rhs_consumes` の `Llvm` の腕が積む leaf に
  一致する。よって前 5 行については、`ρ` の上の各 D9 の消費に対応する走査の `consume` の呼び出しが
  ちょうど 1 つ在り、同じ leaf を名指す。**逆は成り立たない** -- `walk_inner` は `Match` の両方の
  アームを歩くので、`ρ` が選ばなかったアームの本体で行われる `consume` の呼び出しには、対応する `ρ`
  の上の D9 の消費が無い。(0) が対応を `ρ` の上の消費の側から数えるのはこのためである。
  第 6 行 -- 本体の終端の `Ret` -- については走査が `consume` を呼ばないので、言明はそれを除いて
  いる。
  `RcExpr::Release` の腕は 1 つの `Release` 節点の訪問につき 1 回走り、`ρ` の上の `Release(v, π)`
  節点の実行と 1 対 1 に対応する。

<1>1. `Bmp(id') ≥ 1` である名前 `id'` は、ある pending の要素の `outstanding` が名指す。
  BY <ref id=948f840/>, <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: References
  P18b より各要素の `outstanding` は `B(p, ρ)` を `covers` するので、`B(p, ρ)[id'] ≥ 1` ならば
  `outstanding[id'] ≥ 1` である。D15 より `References` は `VarPath` から個数への写像であり、
  `names(o)` は `o` を含むかを答えるので、`outstanding.names(id')` が真である。

<1>2. `consume_objects(pending, objects)` は、`outstanding` が `objects` のいずれかを名指す要素を
      すべて `pending` から取り除く。
  BY CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: PendingRetains, EXT 述語による除去
  `PendingRetains` は `Vec<PendingRetain>` である。`consume_objects` が呼ぶ `pending.retain` の閉包は、
  `objects` のいずれかを `retain.outstanding.names` が真とするとき `false` を返し、そうでないとき
  `true` を返す。`EXT 述語による除去` より、その呼び出しは前者の要素をすべて取り除き、後者の要素を
  すべて残す。

<1>3. CASE 事象が `consume(var, path)` の呼び出しである。
  BY <1>0, <1>0a, <1>1, <1>2, <ref id=347156f/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/ownership.rs: rhs_consumes, <ref id=596a46d/>, <ref id=9d74736/>, <ref id=cbc4a1c/>, <ref id=66c9670/>, DEF 割り当て `μ`,
     DEF `Down、Bmp、Bsub`
  `consume` は `origin(vars, type_env, var, path).acted_on()` を `consume_objects` に渡す。
  <1>0a よりこの呼び出しに対になる D9 の消費は `(var, path)` の 1 つであり、`DEF 割り当て μ` の第 5 行
  より、その消費が `μ` を下げるのはその leaf が inhabited (D16) であるときの `(var, path)` 1 つに
  ついてだけである。`DEF Down、Bmp、Bsub` より `Down(id) = Σ_{s : id ∈ Anc(s)} μ(s)` なので、
  この事象が `Down(id)` を減らす量 `d` は、`(var, path)` が `C` のスロットであって
  `id ∈ Anc(var, path)` であるとき 1、そうでないとき 0 である。以下、場合で分ける。
  `path` は boxed leaf の path であり、それが inhabited でなければ D9 の消費は参照を処分せず `d = 0`
  で (i) が成り立つ (`consume_objects` は要素を減らすだけなので `Bsub` は増えない)。
  `(var, path)` が `C` のスロットでないとき -- D6 のスロットでない記号の位置であるときと、別の類の
  スロットであるときがある -- も `d = 0` で (i) が成り立つ。`DEF Down、Bmp、Bsub` の総和は `C` の
  スロットだけを走るので、その `μ` はどの `Down(id)` の項でもないからである。**`consume` はこの形でも
  呼ばれる** -- `rhs_consumes` の `App` の腕は callee の全 boxed leaf を挙げるので、callee が
  `vars.bindings` に束縛を持たない名前であるとき、その対は記号の位置である (D6)。
  残るのは `(var, path)` が `C` のスロットである場合であり、そのとき処分される
  スロットは `s = (var, path)` の 1 つであり、`d = [id ∈ Anc(s)]` である。`id ∉ Anc(s)` なら `d = 0`
  で同じく (i) が成り立つ。`id ∈ Anc(s)` なら、
  `L21` (a) より `Anc(s) ⊆ acted_on(s)` であり、<1>1 と <1>2 より `Anc(s)` のうち `Bmp ≥ 1` である
  名前を持つ要素はすべて取り除かれる。よって事象の後 `Anc(s)` の各名前について `Bmp ≤ 0` であり、
  <1>0 の非負性と合わせて `Bmp = 0` なので (ii) が成り立つ。

<1>4. CASE 事象が `Release(v, π)` の訪問である。
  <2>1. 訪問はまず `consume_objects(pending, other_objects(v, π))` を行う。
        `other_objects(v, π)` は `π` の下の各 boxed leaf の `candidates() \ {identity()}` を集める。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects
  <2>2. この段の後、処分される各スロット `s = (v, λ)` について、`Anc(s) \ {nm(s)}` の各名前
        `id'` は `Bmp(id') = 0` である。
    BY <2>1, <1>0, <1>1, <1>2, <ref id=347156f/>, <ref id=cbc4a1c/>
    `L21` (a) より `Anc(s) ⊆ acted_on(s) = {nm(s)} ∪ candidates(s)` (D15) なので
    `Anc(s) \ {nm(s)} ⊆ candidates(s) \ {nm(s)}` であり、それは `other_objects(v, π)` に含まれる。
  <2>3. 続けて `un_bump(pending, acted_references(v, π))` が呼ばれる。返り値が `InBracket` のとき、
        選ばれた要素の `B(p, ρ)` から、この `Release` が `ρ` で実際に処分する参照を、それを持つ leaf の
        `origin` の `identity` で名付けて数えた**名前の**多重集合が引かれる。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕,
       CODE src/rc_ir/borrow.rs: un_bump, <ref id=8093b68/>
  <2>4. CASE `un_bump` が `InBracket` を返す。(i) が成り立つ。
    BY <2>1, <2>3, <1>0, <ref id=2340772/>, <ref id=8093b68/>, <ref id=4910eaa/>, <ref id=347156f/>
    この `Release` が処分するのは `π` の下の inhabited な各 leaf の参照であり (D10)、`Down(id)` が
    減る量 `d` はそのうち `id ∈ Anc(s)` であるスロットの個数である。`B(p, ρ)` は `origin` の
    `identity` を鍵とする多重集合であり (D27)、<2>3 よりそこから引かれる名前の多重集合も
    同じ鍵で数えたものなので、`Sub(id)` に落ちる分はちょうど
    `nm(s) ∈ Sub(id)` すなわち `id ∈ Anc(s)` であるスロットの分、すなわち `d` である。`L5a` (d) より
    引き算は打ち切られない -- 選ばれた要素の `B(p, ρ)` は各名前でその量を下回らない -- ので、引かれる
    量はそのまま `B(p, ρ)` から落ちる。`L5b` より `Sub(id) ⊆ Ids(C)` の名前が付く分はほかに無い。
    **この訪問は `un_bump` の前に <2>1 の `consume_objects` も走らせるので、`Bsub(id)` の減り方は
    `d` を下回らない、という向きで読む。** <1>0 より要素が `pending` から取り除かれることは
    `Bmp` を増やさないので、その分は `Bsub(id)` をさらに下げるだけである。よって
    `Bsub(id)` は `d` 以上減り、(i) が成り立つ。
  <2>5. CASE `un_bump` が `OutsideBracket` を返す。(ii) が成り立つ。
    BY <2>1, <2>2, <1>0, <1>1, <1>2, CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の
       `RcExpr::Release` の腕, CODE src/rc_ir/ownership.rs: acted_references
    この腕は `consume_objects(pending, un_bumped.objects())` を呼ぶ。`acted_references(v, π)` は
    `π` の下の各 boxed leaf を `origin` の `identity` で数えるので、処分される各スロット `s` の
    `nm(s)` はその `objects()` に入る。よって <1>1 と <1>2 より、事象の後 `Bmp(nm(s)) ≤ 0` であり、
    <1>0 の非負性と合わせて `Bmp(nm(s)) = 0` である。<2>2 と合わせて `Anc(s)` の各名前について
    `Bmp = 0` であり、その部分集合 `Anc(s) ∩ Sub(id)` についても成り立つ。
  <2>6. CASE `un_bump` が `NoBracket` を返す。(ii) が成り立つ。
    BY <2>2, <1>0, <1>1, CODE src/rc_ir/borrow.rs: un_bump,
       CODE src/rc_ir/ownership.rs: acted_references
    `NoBracket` は、`outstanding` が `un_bumped` とオブジェクトを共有する要素が `pending` に無いこと
    である。処分される各スロット `s` の `nm(s)` は `un_bumped` が名指すので、どの要素の `outstanding`
    も `nm(s)` を名指さない。<1>1 より `Bmp(nm(s)) ≤ 0` であり、<1>0 の非負性と合わせて
    `Bmp(nm(s)) = 0` である。<2>2 と合わせて `Anc(s)` の各名前について成り立つ。
  <2>7. QED
    BY <2>4, <2>5, <2>6, CODE src/rc_ir/borrow.rs: un_bump
    `un_bump` は `UnBump::NoBracket`、`UnBump::OutsideBracket`、`UnBump::InBracket` のいずれかを
    返す。

<1>5. QED
  BY <1>0a, <1>3, <1>4
  (0) は <1>0a である。処分の事象は `DEF 処分の事象` の 2 種で尽きる。

### 11.5 `L24` (名前の木の不等式) <!--#889bad0-->

**言明**。(S) の下で、`ρ` の上の各時点 `τ` と各 `id ∈ Ids(C)` について、`Bsub_τ(id) ≥ 1` ならば
`Bsub_τ(id) ≤ Down_τ(id) - 1` である。

**証明**

<1>1. `Ids(C)` の木について、`id` を固定し、`Sub(id)` の部分集合 `S` が「`Anc` について上に閉じて
      いる」-- すなわち `id' ∈ S` かつ `id'' ∈ Anc(id') ∩ Sub(id)` ならば `id'' ∈ S` -- とする。
      このとき `Sub(id) \ S` は、互いに比較不能な名前 `r_1, …, r_k` について
      `Sub(r_1) ⊎ … ⊎ Sub(r_k)` に分かれる。さらに `Down(r_i)` が数えるスロットは互いに素であり、
      すべて `Down(id)` が数えるスロットであり、`S` の名前を `nm(s)` とするスロットはどの
      `Down(r_i)` にも数えられない。
  BY <ref id=347156f/>
  `L21` (d) より `Ids(C)` は `id_0` を根とする木であり、`Sub(id)` はその部分木である。`id'' ∈ Sub(id)`
  で `id'' ∉ S` であるものについて、`Anc(id'') ∩ Sub(id)` は `id''` から `id` へ至る鎖であり、`S` が
  上に閉じているのでその鎖は「`S` に入らない先頭部分」と「`S` に入る後続部分」に分かれる。先頭部分の
  最後の元を `r(id'')` と置くと、`r(id'')` の親は `S` に在るか `id''` の鎖が `id` で終わるかである。
  相異なる `r` は比較不能である -- `r_i ∈ Anc(r_j)` とすると `r_j` の鎖は `r_i` を通り、`r_i` の親から
  上はすべて `S` に在るので `r_i` は `r(r_j)` の定義に反する。`Sub(r_i)` は互いに素であり (ある名前が
  2 つの比較不能な名前を `Anc` に持つことは、`Anc` が線形順序であること (`L21` (d)) に反する)、
  `Sub(r_i) ⊆ Sub(id) \ S` である -- `id'' ∈ Sub(r_i)` は `r_i ∈ Anc(id'')` を意味し、`id'' ∈ S` と
  すると `S` が上に閉じていることから `r_i ∈ S` となって `r_i` の取り方に反する。逆に
  `id'' ∈ Sub(id) \ S` は `Sub(r(id''))` に入るので、この和は `Sub(id) \ S` に等しい。`Down(r_i)` が数えるスロット `s` は `r_i ∈ Anc(s)` を満たし、
  `id ∈ Anc(r_i) ⊆ Anc(s)` (`L21` (d)) なので `Down(id)` にも数えられる。`S` の名前を `nm(s)` と
  するスロット `s` は `Anc(s) ∋ nm(s) ∈ S` であり、`r_i ∈ Anc(s)` とすると `r_i` は `s` の鎖の上で
  `nm(s)` より上にあり、`S` が上に閉じているので `r_i ∈ S` -- 矛盾。

<1>2. `ρ` の上で `Down` と `Bsub` を動かす事象は次の 6 種で尽きる。
  BY <ref id=c2ea78f/>, <ref id=62d00c2/>, DEF 割り当て `μ`, DEF 処分の事象, <ref id=8093b68/>,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner
  `Down` を動かすのは `μ` を動かす事象であり、`L13a` (g) よりそれは `DEF 割り当て μ` の 6 種で尽きる。`Bsub` を動かすのは
  D27 が `B(p, ρ)` を動かすと述べる事象と、要素が `pending` を離れる事象である。両者を並べると:
  **(E-生成)** D10 の初期値と生成、**(E-Retain)** `Retain` 節点の訪問、**(E-移動-同名)** `L21` (e) の
  前者の移動、**(E-移動-新名)** `L21` (e) の後者の移動、**(E-処分)** `DEF 処分の事象`、
  **(E-落とし)** `merge` と、処分に伴わない `consume_objects` による要素の除去。
  `Eval` の訪問と `Match` 節点の訪問自身はどちらも動かさない (D9 の 2 つの表に行が無く、
  `walk_inner` の `Eval` の腕は `pending` を素通しし、`Match` の腕は `consume_rhs` を呼ばない)。
  関数本体・初期化子の終端の `Ret` の消費は `Down` を減らすが、その後に `ρ` の上の時点が無いので
  言明の対象にならない (`L23` の `DEF 処分の事象` の最後の段落)。

<1>2a. `<1>2` の 6 種の事象を、それぞれ 1 回分の適用の単位で切り出して起きた順に並べる -- **1 つの
       事象**とは、次のいずれかを指す。

       - **(E-生成)**: D10 の初期値または D10 の生成の表の 1 行が、1 つの inhabited な leaf
         `(v, λ)` に参照を 1 つ置く動作。1 つの節点が複数の leaf に置くときは、その leaf の数だけ
         事象が並ぶ。
       - **(E-Retain)**: `Retain(v, π)` の 1 回の訪問が `π` の下の該当 leaf の全体にまとめて行う
         適用。
       - **(E-移動-同名)・(E-移動-新名)**: D9 の移動の 1 つの辺の 1 つのインスタンス
         (`(s, s')` の対 1 組)。
       - **(E-処分)**: `DEF 処分の事象` が言う 1 回の `consume` の呼び出し、または 1 回の
         `Release` の訪問。
       - **(E-落とし)**: 1 つの要素が `pending` を離れる動作 -- `merge` が 1 つの要素を落とすこと、
         または処分に伴わない `consume_objects` が 1 つの要素を落とすこと。

       **1 つの節点の訪問がこの単位で複数の事象を行うことがある** -- `App(f, [a, b])` の
       2 つの引数がどちらも所有位置に立つときは `consume` の呼び出しが 2 回、すなわち事象が 2 つ
       並び、boxed 容器でない `Destructure` が名前付きフィールドを複数持つときは移動の事象がその数
       だけ並ぶ。局所の量 `μ^+`・`Down^+`・`Bmp^+`・`Bsub^+` を、この列の各切れ目 `υ` (最初の事象の
       前の点と、各事象の後の点) について次で定める -- `υ` が `ρ` の上の時点 (節点の訪問の入口) で
       あるときは `μ^+_υ := μ_υ`・`Down^+_υ := Down_υ`・`Bmp^+_υ := Bmp_υ`・`Bsub^+_υ := Bsub_υ`
       (`DEF 割り当て μ` の `μ` と `DEF Down、Bmp、Bsub` の `Down`・`Bmp`・`Bsub` そのもの)、
       それ以外の切れ目では、直前の切れ目の値に、その間に置かれた 1 つの事象が `<1>2` の分類に
       従って足し引きする量を足したものとする。**どの切れ目でも
       `Down^+_υ(id) = Σ_{s : id ∈ Anc(s)} μ^+_υ(s)` であり
       `Bsub^+_υ(id) = Σ_{id' ∈ Sub(id)} Bmp^+_υ(id')` である** -- 各事象が `Down` を動かすのは
       それが `μ` を動かすことによって、`Bsub` を動かすのは `Bmp` を動かすことによってであり、
       両辺は同じ増減を受ける。
       **`μ^+`・`Down^+`・`Bmp^+`・`Bsub^+` はこの証明の中だけで使う局所の
       量であり、`DEF 割り当て μ` の `μ` の定義域を広げるものではない。**
  BY DEF 割り当て `μ`, DEF `Down、Bmp、Bsub`, DEF 処分の事象, <1>2, <ref id=8093b68/>, <ref id=f06144e/>,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner, CODE src/rc_ir/ownership.rs: rhs_consumes
  **README の D34 は `held` についてこれと同じ形の橋を架けており、この段はその技法を `Down`・`Bsub`
  について繰り返す。** D34 は「**この 3 つの箇条が置かない点では、`held` は直前の置き場所の値の
  ままである。**」「**したがって、節点の入口で勘定した値は、その節点の実行の途中の各点まで、上の
  3 つの箇条が置く分だけを足し引きして運べる。**」と架ける。`DEF 割り当て μ` の各行と D27 の各箇条は、
  いずれも 1 つの事象 (この単位) が `μ`・`B(p, ρ)` に与える増減をそれ自身の本文で与えるので、事象の
  列の隣り合う 2 つの切れ目のあいだの増減はそれで定まり、`Down^+`・`Bsub^+` は列のどの切れ目でも
  定まる。節点の訪問の入口はこの列の切れ目の 1 つなので、そこでの値は定義よりそのまま `Down_υ`・
  `Bsub_υ` に一致する。**第 10.7 節が述べる「`μ` は節点の入口でしか定まらない」は `DEF 割り当て μ`
  自身の `μ` の定義域についてであり、この段の `Down^+`・`Bsub^+` は `μ` の値をそのまま運ぶ、この
  証明だけで使う橋であって `μ` 自身の定義を変えないので、この記述と衝突しない。**

<1>2b. 列のどの切れ目 `υ` でも、`B` の各スロット `s` について `μ^+_υ(s) ≥ 0` である。したがって
       `Down^+_υ(id) ≥ 0` であり、`Down^+_υ(id)` が数えるスロットの部分集合についての和も 0 以上で
       ある。
  BY <1>2a, 前提 (S0), 前提 (S4), DEF 割り当て `μ`, <ref id=9d5d254/>, <ref id=f06144e/>
  **`<1>2a` の列の切れ目は 3 通りである** -- 最初の節点の入口より前の切れ目、ある節点の訪問の入口、
  その訪問が行う事象の 1 つの後の点である。第 1 の場合が在るのは、D34 が D10 の初期値の行の置く
  開始値を活性化が始まる側に置くからであり、`<1>2a` の (E-生成) はその初期値の行を事象に数える。
  **第 1 の場合。** `DEF 割り当て μ` より `μ` の値は活性化の開始時にはすべて 0 であり、最初の節点の
  入口より前に置かれる事象は D10 の初期値の行のもの -- 所有する各パラメータ・capture の inhabited な
  各 leaf について `+1` -- だけなので、どのスロットの `μ^+` も 0 か 1 である。
  **第 2 の場合。** `n` の入口は `ρ` の上の時点なので、そこでは `μ^+ = μ` であり (S0) より 0 以上で
  ある。
  **第 3 の場合。** `υ` を含む節点の訪問を `n` とすると、`n` の入口から `υ`
  までに置かれた事象が `μ(s)` を下げた回数は、`n` の実行が `μ(s)` を下げる回数 `k_n(s)` 以下である。
  (S4) より `n` の入口で `μ(s) ≥ k_n(s)` なので、`μ^+_υ(s) ≥ μ(s) - k_n(s) ≥ 0` である。
  `Down^+_υ(id)` は `<1>2a` より `μ^+_υ` の非負の項の和なので 0 以上であり、その項の部分集合に
  ついての和も 0 以上である。

<1>2c. `<1>2a` の列のどの切れ目 `υ` でも、次の 2 つが成り立つ。
  - **(b+)** `υ` においてまだ値を得ていないスロット `s` は `μ^+_υ(s) = 0` である。
  - **(b2+)** `Bmp^+_υ(id) ≥ 1` である `id ∈ Ids(C)` については、`υ` までに走査が訪れたある `Retain`
    節点の入口において、`nm(s) = id` である `C` のスロット `s` が既に値を得ている。

  **`L20b` (b)(b2) を切れ目へ延ばすのがこの段である** -- `L20b` (b)(b2) は「時点 `τ`」で量化して
  いるので、節点の入口とは限らない切れ目では引けない。
  BY <1>2a, <ref id=2a415a3/>, <ref id=c2ea78f/>, <ref id=596a46d/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=8093b68/>, DEF 割り当て `μ`, DEF `Ids`
  (b+)。`DEF 割り当て μ` より `μ` の値は活性化の開始時にはすべて 0 である。`L13a` (g) より `μ` を
  動かす事象は `DEF 割り当て μ` の 6 種で尽き、その 6 種はいずれもその位置のスロットについて値を
  動かす。D6 よりスロットはその変数が値を得た後に在るので、まだ値を得ていないスロットを動かす事象は
  列の上に無い。よってその `μ^+` は 0 のままである。
  (b2+)。`<1>2` より `Bmp` を動かす事象は (E-Retain)・(E-処分)・(E-落とし) の 3 種であり、D27 より
  `B(p, ρ)` に量を足すのは `Retain(v, π)` 節点の訪問が要素を `pending` に押し込む段だけである
  (残る 2 種は引くか、要素を和から落とすかである)。よって `Bmp^+_υ(id) ≥ 1` には、`υ` までのある
  (E-Retain) の事象がその名前に量を積んだことが要る。D27 よりその訪問が押し込む要素の `B(p, ρ)` は、
  `π` の下の inhabited (D16) かつ計数下 (D26) の各 leaf を `origin` の `identity` で名付けて数えた
  ものなので、その訪問の入口の時点で `identity` が `id` である inhabited かつ計数下の leaf -- D6 の
  スロット -- が在る。`id ∈ Ids(C)` のとき `L20b` (c) よりそれは `C` のスロットであり、D6 より
  そのスロットはその変数が値を得た後に在る。

<1>3. `Down^+`・`Bsub^+` について、事象の前の切れ目で言明が成り立つならば、その事象の後の切れ目でも
      成り立つ。以下 `Down`・`Bsub` と書くのは `Down^+`・`Bsub^+` のことである。
  <2>1. CASE (E-生成)。
    BY <1>2a, <1>2c, <ref id=2a415a3/>, 前提 (S3), DEF 割り当て `μ`, DEF `Down、Bmp、Bsub`, <ref id=8093b68/>, <ref id=596a46d/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=c2ea78f/>, <ref id=347156f/>
    **この事象が `C` のスロットを作らないときは、`Down` も `Bsub` も動かない。** `<1>2a` より
    (E-生成) の 1 つの事象は 1 つの leaf `(v, λ)` に参照を置く動作であり、`DEF 割り当て μ` の第 1 行と
    第 2 行はその leaf の `μ` だけを動かす。`Down(id)` の総和が走るのは `C` のスロットだけなので
    (`DEF Down、Bmp、Bsub`)、`(v, λ)` が `C` のスロットでなければ `Down(id)` はどの `id` についても
    変わらない。`Bsub` の側は D27 の 3 つの箇条のどれにも当たらないので動かない。よって言明は
    前の切れ目のまま保たれる。以下、この事象が `C` のスロットを作る場合を見る。
    このとき、この事象より前に `C` はスロットを持たない。`L13a` (h) より ρ-歩みの 1 歩が進む先の
    スロットの変数はいま居る位置の変数より前に値を得るので、
    `C` のスロットが在る時点では `C` の ρ-終端の位置も値を得ており、`L13a` (f) よりその位置は
    パラメータ・capture の leaf か D10 の生成の位置であって、D34 の「開始の時点」の段落よりそこで
    開始値が置かれる。(S3) よりその事象は `ρ` の上に高々 1 つであり、いまの事象がそれである。
    事象の直後に値を得ている `C` のスロットは、この事象が値を与える位置の leaf のうち `C` に属する
    ものである。`L13a` (d)(e) より、パラメータ・capture の leaf も D10 の生成の位置も自分が属する類の
    ρ-終端であり、ρ-終端はスロットごとに 1 つに決まるので、そのような leaf は `C` の ρ-終端 1 つ
    だけである。`L21` (c) よりその `id` は `id_0` である。
    `<1>2c` (b+) より、まだ値を得ていないスロット `s` は `μ^+(s) = 0` を寄せる。よって
    `id ≠ id_0` である各 `id ∈ Ids(C)` について `Down(id) = 0` である。
    `Bsub` の側はどの `id ∈ Ids(C)` についても 0 である -- 上より `C` のスロットはこの事象より前には
    無いので、この事象までのどの `Retain` 節点の訪問の入口でも `C` のスロットは値を得ておらず、
    `<1>2c` (b2+) の対偶より各 `id ∈ Ids(C)` について `Bmp(id) = 0` である。よってどの
    `id ∈ Ids(C)` についても `Bsub(id) = 0` であり、言明は空虚に真である。
  <2>2. CASE (E-Retain)。
    BY <ref id=8093b68/>, DEF 割り当て `μ`, 前提 (S0), 前提 (S2), <ref id=4910eaa/>, <ref id=347156f/>
`Retain(v, π)` の訪問は、`π` の下の inhabited な各 leaf `λ` について `μ(v, λ)` を
    1 上げ (`DEF 割り当て μ` の第 3 行 -- **この行に計数下の条件は無い**)、そのうち計数下の leaf に
    ついて、押し込まれる要素の `B(p, ρ)[nm(v, λ)]` を 1 上げる (D27 -- **計数下の条件はこちらの
    側に在る**)。
    `id` を固定し、`k := #{λ : λ は π の下の inhabited な leaf で (v, λ) は C のスロットであり
    id ∈ Anc(v, λ)}` と置くと、`Down(id)` も `Bsub(id)` もちょうど `k` 増える
    (`nm(v, λ) ∈ Sub(id)` と `id ∈ Anc(v, λ)` は同値、11.2 節)。`Bsub(id)` の側にこれ以外の分が
    無いのは `L5b` による -- `nm(v, λ) ∈ Sub(id) ⊆ Ids(C)` である inhabited かつ計数下の leaf の
    スロット `(v, λ)` は `C` のスロットであり、`C` のスロットでない leaf の分は `Sub(id)` の名前に
    付かない。`k = 0` の `id` では何も変わらない。
    `k ≥ 1` のとき、差 `Down(id) - Bsub(id)` は変わらないので、事象の前に `Bsub(id) ≥ 1` であれば
    言明はそのまま保たれる。事象の前に `Bsub(id) ≤ 0` であったときは、(S2) よりその `Retain` が
    触れる `k` 個のスロットは訪問の入口で `μ ≥ 1` であり、(S0) より `Down(id)` が数える残りのスロットの
    `μ` は 0 以上なので `Down(id) ≥ k` であり、事象の後
    `Down(id) ≥ 2k` かつ `Bsub(id) ≤ k` である。`k ≥ 1` より `2k ≥ k + 1` なので
    `Bsub(id) ≤ Down(id) - 1` である。
  <2>3. CASE (E-移動-同名)。
    BY DEF 割り当て `μ`, <ref id=347156f/>
    `L21` (e) の前者の場合であり、`Anc(s') = Anc(s)` である。`μ(s)` が 1 減り `μ(s')` が 1 増えるので、どの `id` についても `Down(id)` は変わらない。
    `B(p, ρ)` は動かない (D27) ので `Bsub` も変わらない。
  <2>4. CASE (E-移動-新名)。
    BY <1>2c, <ref id=8e3aff3/>, <ref id=2a415a3/>, <ref id=b3dfa37/>, <ref id=596a46d/>, <ref id=30d6238/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=347156f/>, <ref id=c2ea78f/>
    `L21` (e) の後者の場合であり、`Anc(s') = {nm(s')} ∪ Anc(s)` かつ `nm(s') ∉ Anc(s)` である。
    `id ∈ Anc(s)` については `Down(id)` は `-1` (移動元) `+1` (移動先) で変わらず、`Bsub(id)` も
    変わらない。`Anc(s) ∪ {nm(s')}` に入らない `id` については、移動元も移動先もその `Sub(id)` に
    落ちないので両方が変わらない。
    残るのは `id = nm(s')` である。`L21` (e) より `nm(s')` は `s'` 自身の位置、すなわちこの移動が
    値を与える `Match` の束縛変数 `x` の leaf である。`Sub(nm(s'))` の名前を `nm(s'')` とする
    スロット `s''` は `nm(s') ∈ Anc(s'')` を満たすので、D33 の ρ-歩みは `s''` から `s'` の位置を
    通る。`L13a` (h) より、歩みの 1 歩がスロットへ進むとき進む先の変数は `ρ` の上でいま居る位置の
    変数より前に値を得るので、`s''` の変数は `s'` の変数 `x` より後に値を得る。よってこの移動の
    直後に値を得ているそのようなスロットは `s'` だけである。`<1>2c` (b+) より、まだ値を得ていない
    スロットは `μ^+ = 0` を寄せる。すなわち `Down(nm(s'))` は 0 から 1 になる。
    `Bsub(nm(s'))` の側は 0 である -- `Sub(nm(s'))` の名前を `id` とする `C` のスロットは、`s'` と
    `s'` より後に値を得るものだけである。`s'` が値を得るのはこの移動を行う節点の実行であり、`ρ` の
    上でこの移動までに訪れた各 `Retain` 節点の入口では `s'` もまだ値を得ていない。よって
    `<1>2c` (b2+) の対偶より `Sub(nm(s'))` の各名前の `Bmp` は 0 である。
    よって言明は空虚に真である。
  <2>5. CASE (E-処分)。
    <3>1. `id ∉ Anc(s)` がすべての処分されるスロット `s` について成り立つ `id` では、`Down(id)` は
          変わらず `Bsub(id)` は増えない。
      BY <ref id=62d00c2/>, <ref id=2340772/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects,
         CODE src/rc_ir/borrow.rs: un_bump, <ref id=8093b68/>
      `consume_objects` は要素を取り除くだけ、`un_bump` は `InBracket` のとき 1 つの要素の `B` を
      引くだけである。`L5a` (b) より各要素の `B` の成分は 0 以上なので、要素が和から落ちることも
      1 つの成分が引かれることも `Bsub` を増やさない。
    <3>2. 残る `id` について、`L23` の (i) が成り立つ場合は言明が保たれる。
      BY <ref id=62d00c2/>
      `Down(id)` が `d` 減り `Bsub(id)` が `d` 以上減るので、`Bsub(id) ≤ Down(id) - 1` は
      `Bsub` が 1 以上である限り保たれる。
    <3>3. 残る `id` について、`L23` の (ii) が成り立つ場合も言明が保たれる。
      BY <ref id=62d00c2/>, <ref id=2340772/>, <1>1, <1>2a, <1>2b, <ref id=347156f/>, 帰納法の仮定
      `S := ∪_s (Anc(s) ∩ Sub(id))` と置く。ここで `s` はこの事象が処分したスロットのうち
      `id ∈ Anc(s)` であるものを走る。`S` は `Sub(id)` の中で `Anc` について上に閉じている --
      `id' ∈ Anc(s) ∩ Sub(id)` かつ `id'' ∈ Anc(id') ∩ Sub(id)` ならば `L21` (d) より
      `id'' ∈ Anc(s)` である。`L23` の (ii) より、事象の後 `S` の各名前 `id'` は `Bmp(id') = 0` で
      ある。<1>1 より `Sub(id) \ S = Sub(r_1) ⊎ … ⊎ Sub(r_k)` と分かれる。`Bsub(id)` は
      `Sub(id)` の各名前の `Bmp` の和なので、事象の後 `Bsub(id) = Σ_i Bsub(r_i)` である。
      処分されたスロット `s` は `nm(s) ∈ S` なので、<1>1 より どの `Down(r_i)` にも数えられない。
      したがって `Down(r_i)` はこの事象で変わらず、`Bsub(r_i)` は増えない (<3>1 と同じ理由) ので、
      帰納法の仮定 (事象の前の時点での言明) より `Bsub(r_i) ≥ 1` である `i` については事象の後も
      `Bsub(r_i) ≤ Down(r_i) - 1` である。`L5a` (b) より `Bsub(r_i) ≥ 1` でない `i` の
      `Bsub(r_i)` は 0 である。
      `Bsub(id) ≥ 1` とすると、`Bsub(r_i) ≥ 1` である `i` が少なくとも 1 つあり、その個数を `k'` と
      すると
      `Bsub(id) ≤ Σ_{i : Bsub(r_i) ≥ 1} Bsub(r_i) ≤ Σ_{i : Bsub(r_i) ≥ 1} (Down(r_i) - 1)
       ≤ Down(id) - k' ≤ Down(id) - 1`
      である。最後から 2 つ目の不等号は、<1>1 より `Down(r_i)` が数えるスロットが互いに素で
      すべて `Down(id)` に数えられることと、`Down(id)` が数える残りのスロット -- どの
      `Bsub(r_i) ≥ 1` である `Sub(r_i)` にも属さない名前を `nm(s)` とするもの -- の `μ^+` が 0 以上で
      あることによる。**この切れ目は節点の入口とは限らないので、非負性を与えるのは (S0) ではなく
      <1>2b である** -- <1>2a より `Down^+` は `μ^+` の和であり、<1>2b がその各項の非負性を列の
      どの切れ目についても与える。
    <3>4. QED
      BY <3>1, <3>2, <3>3
  <2>6. CASE (E-落とし)。
    BY <ref id=2340772/>, CODE src/rc_ir/borrow.rs: CancelAnalysis::merge,
       CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects, <ref id=8093b68/>
    `merge` が返す `pending` は `pending_in` を `uniform` で絞ったものであり、`uniform` に入るのは
    すべてのアームの出口に同じ `outstanding` で現れる要素だけなので、`ρ` が選んだアームの出口の
    `pending` の部分集合である。よって要素は減るだけであり、D27 より残る要素の `B(p, ρ)` は運ばれる。
    処分に伴わない `consume_objects` も要素を取り除くだけである。`L5a` (b) より落ちる要素の `B` の
    成分は 0 以上なので、どちらも `Down` を変えず `Bsub` を増やさず、言明は保たれる。
  <2>7. QED
    BY <2>1, <2>2, <2>3, <2>4, <2>5, <2>6, <1>2

<1>4. QED
  BY <1>2, <1>2a, <1>2b, <1>2c, <1>3, DEF 割り当て `μ`, EXT 空を作る `Default`,
     CODE src/rc_ir/borrow.rs: cancel, CODE src/rc_ir/borrow.rs: PendingRetains
  `<1>2a` の事象の列についての帰納。列の最初の切れ目は活性化の開始時である。そこで `pending` は
  空であり (`cancel` は `analysis.walk(body, PendingRetains::default(), true)` で走査を始める) ので
  `Bmp^+ ≡ 0`、したがって `Bsub^+ ≡ 0` である。よって前件 `Bsub^+(id) ≥ 1` を満たす `id` が無く、
  言明は空虚に真である。**`Down^+` の側を読む要はない** -- `DEF 割り当て μ` より `μ` の値は活性化の
  開始時にはすべて 0 なので `Down^+ ≡ 0` だが、前件が空である以上どちらの値でも言明は成り立つ。
  **`C` がその点でスロットを持たないとは限らない** -- ρ-終端がパラメータ・capture の leaf である類は
  開始時に既にスロットを持つ (D6) が、そのスロットの `μ` を上げる D10 の初期値の事象はこの切れ目より
  後に並ぶ。段は <1>3 であり、
  <1>2 が事象の種類を尽くす。よって列のどの切れ目でも言明が成り立つ。`ρ` の上の言明が量化する
  各時点 `τ` (節点の訪問の入口) はこの列の切れ目の 1 つなので、`<1>2a` より
  `Down^+_τ = Down_τ` かつ `Bsub^+_τ = Bsub_τ` であり、言明の結論がそのまま出る。

### 11.6 `L25` ((O2)) <!--#e1e0e4a-->

**言明**。(S) を満たす各本体、各実行路 `ρ`、`ρ` を辿る各活性化、各時点、各計数下の
別名類 `C` について、`bumps_ρ(・, C) ≥ 1` ならば `held_ρ(・, C) ≥ 1 + bumps_ρ(・, C)` である。
`ρ` の上にスロットを持つ類については、`L7` よりこれは `U + X ≥ D` と同値である。

**証明**

<1>1. `C` がスロットを持たない時点では `bumps_ρ(・, C) = 0` である。
  BY <ref id=2a415a3/>, <ref id=596a46d/>, <ref id=9f1cf6c/>, DEF `C` の名前
  D6 よりスロットはその変数が値を得た後に在るので、`C` がスロットを持たない時点より前にも
  `C` のスロットは無く、その時点までのどの `Retain` 節点の訪問の入口でも `C` のスロットは値を得て
  いない。`L20b` (b2) の対偶より、`Ids(C)` の各名前について `Bmp = 0` である。A19 (ii-b) が定める
  `bumps` の帰属は `C` のスロットの `identity` に付く分の総和 -- `DEF C の名前` より `Ids(C)` の
  全体にわたる総和 -- なので、その時点で `bumps_ρ(・, C) = 0` である。

<1>2. `C` がスロットを持つ時点では、`L24` を `id_0` に当てて `L22` で読み替えると言明が出る。
  BY <ref id=889bad0/>, <ref id=045d249/>
  `L22` より `Down(id_0) = held_ρ(・, C)`、`Bsub(id_0) = bumps_ρ(・, C)`。`L24` を `id = id_0` に
  当てると、`bumps ≥ 1` のとき `bumps ≤ held - 1` である。

<1>2a. `C` について `L6` の前提 (I) が成り立つ。
  BY <ref id=596a46d/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=80d7700/>, <ref id=30d6238/>
  (I) は「`C` のスロットは `ρ` の上で inhabited であり、`obj(C)` は計数下である」である。D6 より
  スロット `(x, λ)` が在るのは `λ` が `ty(x)` の inhabited (D16) な boxed leaf であるときに限る。
  言明が量化するのは計数下の別名類であり、p13 の `L12a` と D33 より `C` のすべてのスロットは
  `obj(C)` を指すので、その `obj(C)` が計数下 (D26) であることが「計数下の別名類」の意味である。

<1>2c. `C` が `ρ` の上にスロットを持つとき、`C` は開始事象をちょうど 1 つ持つ。
  BY <1>2a, 前提 (S3), <ref id=596a46d/>, <ref id=30d6238/>, <ref id=9d5d254/>, DEF 開始事象, <ref id=c2ea78f/>
  (S3) が「高々 1 つ」を与えるので、1 つ以上あることを言えばよい。`L13a` (f) より、計数下の別名類
  `C` の ρ-終端はパラメータ・capture の leaf か D10 の生成の表の位置のいずれかである。`DEF 開始事象`
  の 3 行がその 2 種を覆うので、その位置の変数が値を得る時点で開始事象が 1 つ起きる (D34 の
  「開始の時点」の段落)。**その時点が `C` のスロットが在る時点以前であることは `L13a` (h) による**
  -- `C` のスロット `s` から ρ-終端までの D33 の ρ-歩みの各段は、進む先の変数がいま居る位置の変数より
  前に値を得る位置へ進むので、ρ-終端の変数は `s` の変数以前に値を得る。D6 よりスロット `s` は
  その変数が値を得た後に在るので、`s` が在る時点までに開始事象が 1 つ起きている。

<1>3. QED
  BY <1>1, <1>2, <1>2a, <1>2c, <ref id=86b3f11/>
  不等式は、`C` がスロットを持つ時点では <1>2 が、持たない時点では <1>1 が `bumps = 0` を与えるので
  空虚に、それぞれ成り立つ。同値の側は、スロットを持つ類について `L7` の恒等式
  `held - (1 + bumps) = U + X - D` から出る -- `L7` の前提のうち `L4` の前提 (`C` の開始事象が 1 つ)
  は <1>2c が、`L6` の前提 (I) は <1>2a が与える。

**`insert_rc` の出力について。** `L20a` より `insert_rc` の出力の各本体は (S) を満たすので、`L25` から
(O2) -- 第 8 節の言明 -- が出る。**`split_rc_units` の出力について。** 第 13 節の `L32` が、その出力も
(S) を満たすことを示す。

## 12. A19 (ii-b) が延びない点

`L19` (c) は、関数本体・初期化子の終端の `Ret` の消費の後、各計数下の別名類の `held` が 0 であることを
示す。A19 (ii-a) はこの点でも成り立つ (`0 ≥ 0`)。この節は A19 (ii-b) がこの点では偽であることを、
`insert_rc` の出力を挙げて示す。

### 12.1 道具立て

`DEF 例の型と op` と `DEF 例の名前の取り方` を使う。

**DEF `Pair` と `make_pair`**。`Pair` を `Arr` を 2 つ持つ unbox 構造体とし、その最上位の tycon は
`make_array_tycon()` とは異なるものとする (`Std::Array` は 1 引数の組み込みであって、この例の
2 フィールドの構造体宣言ではない)。
`make_pair : (Arr, Arr) -> Pair` は `InlineLLVMMakeStructBody` であり、その `result_prov` は
unbox 構造体について、結果の leaf `[i] ++ σ` を単一の `Arg(i, σ)` と宣言する
(`CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMMakeStructBody`)。`borrows_operand` は既定の
偽である (`CODE src/ast/inline_llvm.rs: LLVMGen::borrows_operand`)。

骨格 `S_g` (関数 `g`、パラメータ `m : Arr`、**capture 無し**、`borrowed_units` は空、返り値の型
`Pair`):

```
Let(x, Llvm(make_pair, [m, m]), Ret(x))
```

### 12.2 `L38` (A19 (ii-b) は活性化の終わりの 1 点先では偽である) <!--#d1f21f8-->

**言明**。`insert_rc` は `S_g` を次の本体 `B_2` に書き換える。

```
Retain(m, [], RcState::Unknown,
Let(x, Llvm(make_pair, [m, m]),
Ret(x)))
```

`B_2` は D11 を満たす (したがって `g` だけからなるプログラムは D12 を満たす)。`B_2` の唯一の実行路と、
`m` が計数下のオブジェクトを受け取る活性化について、
終端の `Ret` の消費を行った直後の点では `held = 0` かつ `bumps = 1` であり、`held ≥ 1 + bumps` は
偽である。

**証明**

<1>1. `insert_rc` は `S_g` を `B_2` に書き換える。
  BY <ref id=3e6b0e0/>, DEF 例の名前の取り方, DEF `Pair` と `make_pair`,
     EXT `Iterator::fold`, EXT `Iterator::rev`,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_expr_inner,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
     CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
     CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
     CODE src/rc_ir/rc_insert.rs: insert_if_local,
     CODE src/rc_ir/rc_insert.rs: rhs_operands, CODE src/rc_ir/rc_insert.rs: RcInserter::needs_rc,
     CODE src/ast/name.rs: FullName::is_local,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
     CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases
  A15 より `insert_into_expr` は `insert_into_expr_inner` をちょうど 1 回呼ぶ。
  `Ret(x)` の腕は `live_after = ∅` に `insert_if_local` で `x` を足した `live = {x}` を返し
  (`DEF 例の名前の取り方` より `x` の名前は名前空間を持たないので `is_local` は真である)、
  `retain_if_live` は
  `live_after` が空なので発火しない。`insert_into_operation_let` は `live_cont = {x}` の下で
  `rhs_operands(Llvm(make_pair, [m, m])) = [(m, Own), (m, Own)]` を逆順に走る -- 第 2 の `m` は
  `used_later` が偽 (`live_after_operand` はまだ `live_cont = {x}` の写しであって `m` を含まない) なので
  何も置かず、第 1 の `m` は第 2 の `m` が `live_after_operand` に入った後なので `used_later` が真で
  `retains_before` に入る (`needs_rc(m)` は真である -- `is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `ty(m) = Arr` では偽である)。`x` は `live_cont` に在る
  ので `after` は空である。`live_before` は `({x} \ {x}) ∪ {m} = {m}` であり (`m` も
  `DEF 例の名前の取り方` より局所名なので `insert_if_local` の門を通る)、`insert_into_func` の
  `unused` は `live` が `m` を含むので空である。`build_retains([m], ・)` は
  `EXT Iterator::rev` と `EXT Iterator::fold` より 1 元の `vars` について
  `RcExpr::Retain(m, vec![], RcState::Unknown, node)` を 1 つ作り、`Retain(m, [])` を被せる。
  `build_releases` に渡る列はどれも空なので、`EXT Iterator::fold` より初期値がそのまま返る。

<1>2. `B_2` の実行路は 1 本であり、その上のスロットは `(m, [])`・`(x, [0])`・`(x, [1])` の 3 つで、
      `C := {(m, []), (x, [0]), (x, [1])}` はその 3 つからなる 1 つの別名類であり、`nm` はどの
      スロットについても `(m, [])` である。`obj(C)` は `m` が受け取ったオブジェクトであり、それが
      計数下 (D26) であるかどうかは活性化ごとに決まる。
  BY <ref id=ca36627/>, <ref id=0594f24/>, <ref id=88a06de/>, <ref id=e11772a/>, <ref id=d59f90b/>, <ref id=9c7c27a/>, <ref id=596a46d/>,
     DEF `Pair` と `make_pair`, DEF `nm(s)`,
     CODE src/rc_ir/ownership.rs: collect_bindings,
     CODE src/rc_ir/ownership.rs: VarTable::of,
     CODE src/rc_ir/ownership.rs: origin_inner, CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/ownership.rs: Origin::identity,
     CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_unbox,
     CODE src/ast/types.rs: TypeNode::is_box, CODE src/ast/types.rs: TypeNode::is_array,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon,
     CODE src/rc_ir/leaf_map.rs: LeafMap::get,
     CODE src/fixstd/builtin.rs: is_array_tycon, CODE src/fixstd/builtin.rs: make_array_tycon,
     <ref id=30d6238/>
  `Match` が無いので実行路は 1 本である。`is_fully_unboxed` は
  `if self.is_box(type_env) { return false; }` で始まるので boxed な `Arr` では偽であり、`Pair` は
  boxed なフィールドを持つのでその再帰でも偽である。`is_box` の本体は `!self.is_unbox(type_env)` で
  あり、`is_unbox` は `self.is_closure() || self.toplevel_tycon_info(type_env).is_unbox` なので、
  `is_box` が真の `Arr` では `is_closure` が偽である。`Pair` は `DEF Pair と make_pair` より
  unbox 構造体であってクロージャではない。**`Pair` について第 4 規則 (`is_array`) も外れる** --
  `is_array` は `toplevel_tycon_satisfies` に `is_array_tycon` を渡したものであり、
  `toplevel_tycon_satisfies` が述語に渡すのは `toplevel_tycon()` が返す tycon である。
  `is_array_tycon(tc)` は `*tc == make_array_tycon()` を問うが、`DEF Pair と make_pair` より
  `Pair` の最上位の tycon はそれとは異なる。
  よって D4 の判定は `Arr` については第 1 規則と第 2 規則を抜けて第 3 規則に着き、`Pair` については
  第 1・第 2・第 3・第 4 規則を抜けて第 5 規則に着くので、
  `boxed_leaf_paths(Arr) = {[]}`、`boxed_leaf_paths(Pair) = {[0], [1]}` で
  ある。`collect_bindings` は `x` に
  `Binding::Llvm(make_pair, [m, m], Pair)` を入れる。`DEF Pair と make_pair` より `make_pair` の
  `result_prov` は結果の leaf `[i] ++ σ` を単一の `Arg(i, σ)` と宣言するので、
  `leaf_origins_at(π)` の本体は `self.0.get(π)`、`LeafMap::get` の本体は `self.0.get(path)` -- その
  path の leaf に置かれた項 -- なので、`leaf_origins_at([0])` は単一の `Arg(0, [])`、
  `leaf_origins_at([1])` は単一の `Arg(1, [])` であり、`as_arg_projection` はそれぞれ `Some((0, []))`・`Some((1, []))` を返す。
  よって `origin_inner` の `Binding::Llvm` の腕は `origin(args[0], [])`・`origin(args[1], [])` を
  返し、`args` は `[m, m]` なのでどちらも `origin(m, [])` である。
  **D20 の別名の辺と D17 の行き先はここに立つ** -- D20 の移動の表の `Llvm` の素通し leaf の行が
  `(m, [])` から `(x, [0])` へ、および `(m, [])` から `(x, [1])` への辺を挙げ、D17 の
  `Binding::Llvm` の行より、`λ` に対応する位置はその宣言 `Arg(j, σ')` の `(args[j], σ')`、すなわち
  どちらも `(m, [])` である。よって `(x, [0])` と `(x, [1])` の ρ-歩みはどちらも 1 歩で `(m, [])` へ
  進む。`m` はパラメータ
  なので `VarTable::of` が `Binding::Param` を入れ、D33 が ρ-歩みを止める第 1 種
  「辺を持たない束縛、すなわち `Binding::Param`、`Binding::Producer`、および束縛を持たない名前
  (記号の位置)」に当たるので、`(m, [])` が 3 つのスロットの ρ-終端である。
  3 つのスロットは ρ-終端が等しいので 1 つの別名類をなす。`Origin::identity` は
  `Origin::Exactly(p)` に `p` を返し、`origin_inner` の `Binding::Param` の腕は
  `Exactly((m, []))` を返すので、`DEF nm(s)` より 3 つのスロットの `nm` はどれも `(m, [])` である。
  D33 より 1 つの別名類のすべてのスロットは
  同じオブジェクトを指すので、`obj(C)` は `m` が受け取ったオブジェクトである。D26 より
  オブジェクトは計数下かグローバル状態かのどちらかであり、`m` に何が渡るかは活性化が決める。

<1>3. `m` が計数下のオブジェクト `O_m` を受け取る活性化について、`μ`、`held`、`bumps` は次のように
      動く。
  BY <1>1, <1>2, <ref id=c2ea78f/>, <ref id=dca1c02/>, DEF 割り当て `μ`, DEF `Pair` と `make_pair`, DEF `nm(s)`,
     <ref id=9d74736/>, <ref id=f06144e/>, <ref id=cbc4a1c/>, <ref id=8093b68/>, <ref id=e11772a/>, <ref id=66c9670/>, <ref id=88a06de/>, <ref id=e3436e8/>, <ref id=9d5d254/>, <ref id=9f1cf6c/>,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::consume,
     CODE src/rc_ir/borrow.rs: cancel,
     EXT `Iterator::filter_map`,
     CODE src/rc_ir/ownership.rs: acted_references, CODE src/rc_ir/ownership.rs: rhs_consumes,
     CODE src/rc_ir/ownership.rs: passthrough_arg_leaves,
     CODE src/rc_ir/ownership.rs: as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance, CODE src/rc_ir/provenance.rs: LeafOrigin

  | 点 | `μ(m, [])` | `μ(x, [0])` | `μ(x, [1])` | `held` | `bumps` |
  |---|---|---|---|---|---|
  | `Retain(m, [])` の入口 | 1 | 0 | 0 | 1 | 0 |
  | `Let(x, …)` の入口 | 2 | 0 | 0 | 2 | 1 |
  | `Ret(x)` の入口 | 0 | 1 | 1 | 2 | 1 |
  | 終端の `Ret` の消費の後 | 0 | 0 | 0 | 0 | 1 |

  `μ` の初期値は D10 の初期値の行 -- `L15` (e) より `m` の unit は `g` が所有する -- で
  `μ(m, []) = 1` である。
  `Retain(m, [])` は `μ(m, [])` を 1 上げる。`Let(x, Llvm(make_pair, [m, m]), ・)` は、D9 の移動の表の
  `Llvm` の素通し leaf の行で `(m, [])` の参照を `(x, [0])` と `(x, [1])` へ移すので `μ(m, [])` が
  2 下がり `μ(x, [0])` と `μ(x, [1])` が 1 ずつ上がる。終端の `Ret(x)` の消費は `x` の全 boxed leaf の
  参照を消費する。`L15` (e) と `L13a` (c) より `β(C) = 0` なので、`held` は `L13a` (b) より 3 つの
  `μ` の和である。
  `bumps` について、`Retain(m, [])` の訪問が `outstanding` に置くのは
  `self.acted_references(m, [])` -- `CancelAnalysis` のメソッドであり、`ownership::acted_references` の
  返り値をそのまま返す -- である。D15 よりそれは `[]` の下の各 boxed leaf の `origin` の `identity` を
  数えたもので、<1>2 より `ty(m) = Arr` の boxed leaf は `[]` の 1 つ、その `nm` は `(m, [])` なので
  `{(m, []): 1}` である。`m` が計数下のオブジェクトを受け取る活性化ではこの leaf は inhabited (D16)
  かつ計数下 (D26) なので、D27 より `B` も `{(m, []): 1}` である。
  `Let(x, Llvm(make_pair, [m, m]), ・)` の訪問は `consume_rhs` を呼ぶが、`rhs_consumes` の `Llvm` の腕は
  素通しの leaf を消費として報告しない。**`passthrough_arg_leaves` がその集合を作る** --
  それは `decl.leaves().filter_map(as_arg_projection)` であり (`EXT Iterator::filter_map`)、`decl` は
  `make_pair` の `result_prov` の返り値である。`DEF Pair と make_pair` よりその宣言は結果の leaf
  `[0]` に単一の `Arg(0, [])`、leaf `[1]` に単一の `Arg(1, [])` を置き、<1>2 より `Pair` の boxed
  leaf はその 2 つで尽きるので、`Provenance::leaves()` が渡すのはその 2 つであり、
  `as_arg_projection` は `LeafOrigin::Arg(j, p)` に `Some((j, p))` を返すので、
  `passthrough_arg_leaves` は `{(0, []), (1, [])}` を返す。
  `rhs_consumes` の `Llvm` の腕は `if !passthrough.contains(&(i, leaf))` の門で leaf を積むので、
  `m` の唯一の boxed leaf `[]` はオペランド 0 についても 1 についてもこの集合に入り、積まれない。
  **`consume_rhs` が `consume` を呼ぶのは `rhs_consumes` が積んだ各元についてだけである**ので、
  この訪問はどの要素も `pending` から落とさない。終端の `Ret` の腕は
  `returns_from_func` が真なので要素を `needed_retains` に入れるが、`pending` からは取り除かない。
  **最上位の走査が `returns_from_func = true` で始まることは `cancel` にある** --
  `analysis.walk(body, PendingRetains::default(), true)` である。
  **表の最終行の点で `bumps` が値を持つことは D34 と D27 による** -- D34 の第 1 の箇条は表の第 6 行
  (D9 の消費) の事象をそれを運ぶ素動作の直後の段内の点 (D24) に置くので、終端の `Ret` の消費の直後は
  段内の点であり、D27 の最後の段落より節点の実行の途中の点における `B(p, ρ)` はその節点の訪問の
  入口における値である。A19 (ii-b) が定める `bumps` の帰属をその値に当てると 1 である。

<1>4. `B_2` は D11 を満たす。
  <2>1. `B_2` の各活性化の各時点で、計数下の別名類 `C'` が `held(・, C') ≥ 1` を満たすならば
        `H(obj(C')) ≥ 1` であり、その活性化がその時点まで解放について閉じている (D11a) ならば
        `obj(C')` はその時点で解放されていない。
    BY <1>2, <ref id=9f1cf6c/>, <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=859cf84/>, <ref id=c232680/>, <ref id=30d6238/>
    D21 は「**活性化は、その各時点と各段内の点 (D24) で A19 (i) の不等式を満たすものに限る。**」と
    定め、その不等式の本文 -- 角括弧、総和 `S`、`d(C)` -- は A19 (i) が置く。
    `L15` (e) と `L13a` (c) より `β ≡ 0` なので、A19 (i) の角括弧は 0 であり
    各類について `d(・) = held(・, ・)` である。<1>2 より `B_2` の唯一の実行路の上でスロットを持つ
    別名類は `C` だけなので、A19 (i) の総和 `S` は `C'` を含むならその 1 項だけからなる。よって
    `H(obj(C')) ≥ held(・, C') ≥ 1` である。D11a は、解放について閉じている時点では `H(O) ≥ 1` である
    各計数下オブジェクトが解放されていないと定める。
  <2>2. `m` が計数下のオブジェクトを受け取る活性化について、(S-a) と (S-b) が成り立つ。
    BY <1>1, <1>2, <1>3, <ref id=dca1c02/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=95427eb/>, <ref id=e11772a/>
    `Obl` は活性化の開始で `{O_m}` である -- D10 の初期値は所有するパラメータの inhabited な各 leaf に
    つき参照を 1 つ入れ、`L15` (e) より `m` の unit は `g` が所有する。`Retain(m, [])` が 1 つ加えて
    `{O_m, O_m}`、`make_pair` は A3 の単一 `Arg` の行より新しい参照を作らず D9 の移動の行より `Obl` を
    変えないので `{O_m, O_m}` のまま、終端の `Ret(x)` の消費が `x` の 2 つの boxed leaf の参照を
    呼び出し元へ渡して空になる。(S-a): `Obl` から参照を取り除く操作は終端の `Ret` の消費だけであり、
    その時点の `Obl` はその 2 つの参照を持つ。(S-b): その消費の後 `Obl` は空である。
  <2>3. `m` が計数下のオブジェクトを受け取る活性化について、(S-c) が成り立つ。
    BY <2>1, <1>2, <1>3, <ref id=56c2068/>, <ref id=95427eb/>, <ref id=859cf84/>, <ref id=e3436e8/>, <ref id=fd95f12/>, <ref id=f06144e/>
    D7 の読む構文はこの本体に `Let(x, Llvm(make_pair, [m, m]), ・)` の 1 つだけ在り、その各オペランドの
    leaf `(m, [])` を読む。触れるのは `Retain(m, [])` である。D24 の「**読みの直前の点では、勘定は
    直前の段内の点のものである。**」より、読み・触れる動作の直前の点の勘定は直前の段内の点のもので
    ある。**その段内の点は、その節点の入口の勘定を持つ** -- `Llvm` の節点では A26 より読みはこの節点が
    行うどの手放しよりも前に起き、`Retain` の節点ではその段が参照を 1 つも処分しない
    (D10 の `Retain` の行) ので、どちらでも節点の入口からその点までのあいだにこの段は参照を処分して
    おらず、D24 の (F) の解放を起こすのは参照の処分なので、そのあいだにどのオブジェクトも解放されない。
    <1>3 より
    `Retain(m, [])` の入口で `held = 1`、`Let(x, …)` の入口で `held = 2` なので、<2>1 より
    `obj(m, [])` はどちらの点でも解放されていない。**(S-c) の接頭条件 (D11a) はここで満たされる** --
    <2>1 が結論を、その活性化がその時点まで解放について閉じている (D11a) ことを前件とする形で与え、
    (S-c) が課す条件がまさにそれだからである。終端の `Ret` は D7 の読む構文ではない。
  <2>4. `m` がグローバル状態のオブジェクトを受け取る活性化について、D11 の 3 つの節が成り立つ。
    BY <1>2, <ref id=56c2068/>, <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=95427eb/>, <ref id=88a06de/>, <ref id=30d6238/>, <ref id=b6673ca/>
    D26 よりグローバル状態のオブジェクトを指す leaf は D8 の意味の参照を持たないので、D10 の初期値は
    空であり、`Retain`・終端の `Ret` の消費・D9 の移動のどれも `Obl` を動かさない。(S-a) は `Obl` から
    参照を取り除く操作が 1 つも無いので、(S-b) は `Obl` が空のままなので成り立つ。(S-c) について、
    この本体が読み・触れるオブジェクトは `obj(m, [])` だけである -- <1>2 より `x` の 2 つの leaf も
    `(m, [])` と同じ別名類に属し同じオブジェクトを指す (D33)。A8 と D26 より、グローバル状態の
    オブジェクトが解放されることは無い。
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, <1>2, <ref id=0594f24/>, <ref id=95427eb/>, <ref id=3d96eb8/>, <ref id=66c9670/>, <ref id=88a06de/>
    D26 よりオブジェクトは計数下かグローバル状態かのどちらかであり、1 つの活性化の間その区別は
    変わらないので、`B_2` の活性化は `obj(m, [])` が計数下であるものとグローバル状態であるものに
    分かれる。前者は <2>2 と <2>3、後者は <2>4 が覆う。**`m` の leaf が inhabited でない活性化は無い**
    -- <1>2 より `boxed_leaf_paths(Arr) = {[]}` は D4 の第 3 規則で積まれた leaf であって unbox union の
    節を 1 つも通らないので、D16 より常に inhabited である。`Pair` の `[0]` と `[1]` も同じく
    D4 の第 5 規則が unbox 構造体のフィールドへ降りて積んだもので、union の節を通らない。
    D12 は「`P` のすべての関数の本体と、すべての
    グローバル初期化子の `init` が、`P` の `borrowed_units` が定める所有と借用の割り当て (D14) の下で
    RC 規律を満たす (D11) こと」なので、`g` だけからなるプログラムは D12 を満たす。

<1>5. QED
  BY <1>1, <1>3, <1>4
  <1>3 の表の最後の行で `held = 0` かつ `bumps = 1` なので `held ≥ 1 + bumps` は偽である。

**`L25` はこの本体でも成り立つ。** <1>3 の表の 1 つ前の行までは `held ≥ 1 + bumps` である (`2 ≥ 2`)。
`L25` の言明が量化するのは `ρ` の上の時点 -- 節点の入口 -- であり、終端の `Ret` の消費の後はその中に
無い。

## 13. `split_rc_units` の段

A19 (ii) の範囲の第 1 の半分 -- `borrow_ify` の入力の各本体 -- は `split_rc_units` の出力である。
第 10 節と第 11 節が
示すのは `insert_rc` の出力についてなので、`Retain(v, π)` を `units_under(ty(v), π)` の鎖へ割る段が残る。
この節がそれを扱う。

`split_rc_units` は `insert_rc` の直後・`borrow_ify` の直前に走り、その間にプログラムを書き換える
パスは無い。**隣接を与えるのは呼び出し元である** -- `build_object_files` は `lower_and_insert_rc` の
返り値をそのまま `optimize_rc_program` に渡し、`lower_and_insert_rc` は `insert_rc` を掛けて返り、
`optimize_rc_program` はその値に `split_rc_units` を掛けてから `borrow_ify` を呼ぶ
(`CODE src/build/build_object_files.rs: build_object_files`,
`CODE src/build/build_object_files.rs: lower_and_insert_rc`,
`CODE src/build/build_object_files.rs: optimize_rc_program`)。**間に走るのは `validate` の呼び出し
だけである** -- `optimize_rc_program` は `split_rc_units` の前後に `"after insert_rc"` と
`"after split_rc_units"` の 2 回それを呼び、どちらも `config.develop_mode` のときだけ走り、
`&RcProgram` を受け取って検査するだけで書き換えない (`CODE src/rc_ir/validate.rs: validate`)。
よってこの節の入力は `insert_rc` の出力である。

### 13.1 記法

`P` を `insert_rc` の出力のプログラム、`P' := split_rc_units(P)` とする。`B` を `P` の 1 つの本体
(ある関数の `body` か、あるグローバル初期化子の `init`)、`B' := split_body(B)` を `P'` の対応する本体と
する。

**DEF `Leaves`**。型 `τ` と path `π` について
`Leaves(τ, π) := {λ ∈ boxed_leaf_paths(τ) : λ は π で始まる}` と置く
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。

### 13.2 `L26` (`split_rc_units` が行う書き換え) <!--#1ab62dc-->

**言明**。

- **(a)** `split_rc_units` は `prog.funcs` の各関数の `body` と `prog.globals` の各初期化子の `init` を
  `split_body` の像に置き換え、`RcProgram`・`RcFunc`・`RcGlobalInit` の他のフィールドを 1 つも変えない。
  とくに `borrowed_units` を変えない。
- **(b)** `split_body` は本体の木を次の規則で写す。
  - `t := Retain(v, π, s, k)` は、`units_under(ty(v), π) = [u_1, …, u_n]` として
    `Retain(v, u_1, s, Retain(v, u_2, s, … Retain(v, u_n, s, split_body(k)) …))` に写る。`n = 0` の
    ときは `split_body(k)` そのものに写る。この `n` 個の節点を `t` の**鎖**と呼ぶ。
  - `Release(v, π, s, k)` は、節点の種類が `Release` であることだけが違う同じ形に写る。
  - `Let(x, Match(scrut, arms), k)` は `Let(x, Match(scrut, arms'), split_body(k))` に写る。`arms'` の
    第 `j` アームは `arms` の第 `j` アームの `tag`・`payload`・`payload_state` をそのまま持ち、本体が
    `split_body(arms[j].body)` である。
  - `Let(x, rhs, k)` (`rhs` は `Match` でない) は `Let(x, rhs, split_body(k))`、
    `Destructure(c, fs, s, k)` は `Destructure(c, fs, s, split_body(k))`、`Eval(v, k)` は
    `Eval(v, split_body(k))`、`Ret(v)` は `Ret(v)` に写る。

  この 5 種から作られた節点を**写し**と呼ぶ。写しは `B` の `Retain`/`Release` でない節点と 1 対 1 に
  対応する。**群**とは、1 つの写しか、1 つの `Retain`/`Release` 節点の鎖のことである。群は `B` の節点と
  1 対 1 に対応する。群の**入口**とはその最初の節点の入口であり、鎖が空 (`n = 0`) の群は節点を持たない
  のでその入口は次の群の入口と同じ点である。D2 より `Retain` と `Release` は継続をちょうど 1 つ持ち、
  本体の終端の節点は `Ret` -- 写し -- なので、鎖の群の次には必ず別の群がある。

**証明**

<1>1. (a)。
  BY CODE src/rc_ir/borrow.rs: split_rc_units
  この関数の本体は `for func in prog.funcs.values_mut() { func.body = split_body(&func.body, type_env); }`
  と `for g in &mut prog.globals { g.init = split_body(&g.init, type_env); }` の 2 つのループだけで
  あり、`body` と `init` 以外のフィールドへの代入を持たない。

<1>2. (b) の `Retain` の行。
  BY CODE src/rc_ir/borrow.rs: split_body, CODE src/rc_ir/borrow.rs: split_body_inner,
     CODE src/rc_ir/borrow.rs: split_rc, CODE src/rc_ir/borrow.rs: rc_node,
     CODE src/rc_ir/borrow.rs: expr_node, <ref id=3e6b0e0/>, EXT `Iterator::fold`, EXT `Iterator::rev`
  `split_body_inner` の `RcExpr::Retain(v, path, state, k)` の腕は、まず `k` を `split_body` で写し、
  その結果を `split_rc(v, path, *state, false, k, &node.source, type_env)` に渡す。`split_rc` は
  `units_under(&v.ty, path, type_env).into_iter().rev().fold(k, |cont, unit| rc_node(is_release,
  v.clone(), unit, state, cont, source))` である。`EXT Iterator::rev` より走る順は `units_under` が
  返す列の逆順であり、`EXT Iterator::fold` より各段は直前の段の結果を継続に据えた節点を作るので、
  `k` から始めて外へ向かって節点が積まれ、第 1 の unit が最も外側に来る。`rc_node` は `is_release` が
  偽のとき `RcExpr::Retain(var, path, state, k)` を作る。`units_under` が空の列を返せば逆順の
  反復子も空であり、`EXT Iterator::fold` より `fold` は初期値 `k` をそのまま返す。
  A15 より `split_body` は `split_body_inner` をちょうど 1 回呼ぶ。

<1>3. (b) の `Release` の行。
  BY <1>2, CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/borrow.rs: split_rc,
     CODE src/rc_ir/borrow.rs: rc_node, <ref id=3e6b0e0/>, EXT `Iterator::fold`, EXT `Iterator::rev`
  `split_body_inner` の `RcExpr::Release(v, path, state, k)` の腕は同じ `split_rc` を
  `is_release = true` で呼ぶので、<1>2 の `fold` についての読み -- 積む順、最も外側に来る unit、
  そして `units_under` が空の列を返すときに初期値 `k` がそのまま返ること -- がそのまま当たる。
  違うのは `rc_node` が作る節点だけで、`is_release` が真のときそれは
  `RcExpr::Release(var, path, state, k)` である。

<1>4. (b) の残る 5 行。
  BY CODE src/rc_ir/borrow.rs: split_body_inner, CODE src/rc_ir/ast.rs: MatchArm::with_body,
     CODE src/rc_ir/borrow.rs: expr_node, <ref id=3e6b0e0/>, <ref id=b3dfa37/>
  `split_body_inner` の残る 5 つの腕は、いずれも同じ構成子を、同じ変数・同じ右辺・同じ `RcState` と、
  `split_body` で写した継続 (と写したアーム本体) から作り直す。`Match` の腕は
  `arm.with_body(split_body(&arm.body, type_env))` を使い、`with_body` は `body` 以外のフィールドを
  複製する (`MatchArm { body, ..self.clone() }`)。**`body` 以外が `tag`・`payload`・`payload_state` の
  3 つで尽きることは D2 による** -- D2 は「`Match` の各アーム `MatchArm` は 4 個の
  フィールドを持つ」と述べ、その 4 つを `tag`・`payload`・`payload_state`・`body` と挙げる。

<1>5. QED
  BY <1>1, <1>2, <1>3, <1>4, <ref id=b3dfa37/>
  D2 より `RcExpr` は 6 個の構成子を持ち、`split_body_inner` の `match` は
  `Retain`・`Release`・`Let`(`Match` の右辺)・`Let`(その他)・`Destructure`・`Eval`・`Ret` の 7 つの腕で
  それを覆う。写しが `B` の `Retain`/`Release` でない節点と 1 対 1 に対応するのは、この 5 行がどれも
  節点を 1 つずつ作り直すからである。

### 13.3 `L27` (unit は path の下の boxed leaf を分割する) <!--#38dde17-->

**言明**。**A10 を満たす**任意の型 `τ` と任意の path `π` について次が成り立つ。この命題を当てる先は
プログラムに現れる変数の型なので、A10 がその前提を与える。

- **(a)** `units_under(τ, π)` の各元は `π` で始まる。`Leaves(τ, π)` の各元はちょうど 1 つの
  `u ∈ units_under(τ, π)` について `Leaves(τ, u)` に入る。すなわち
  `Leaves(τ, π) = ⊎_{u ∈ units_under(τ, π)} Leaves(τ, u)` である。
- **(b)** `subtree_type(τ, π)` が `Some` を返すとき、`units_under(τ, π)` の各元 `u` について
  `Leaves(τ, u)` は空でない。
- **(c)** `units_under(τ, [])` は `rc_units(τ)` である。その長さは 0 にも 1 にも 2 以上にもなり、
  長さが 0 のときは (a) より `Leaves(τ, []) = boxed_leaf_paths(τ)` が空である。

**証明**

<1>0. **接頭不変性。** A10 を満たす任意の型 `σ` と任意の path `p` について、`boxed_leaf_paths` の
      `go` を `(σ, path = p)` で呼んだときに `out` へ積まれる元は、`(σ, path = [])` で呼んだときに
      積まれる各元 `ν` に `p` を前置した `p ++ ν` の全体である。`rc_units_go` についても同じことが
      成り立つ。とくに `rc_units(σ)` は `rc_units_go` を `path = []` で呼んだときに積まれる元の
      全体であり、`boxed_leaf_paths(σ)` は `go` を `path = []` で呼んだときに積まれる元の全体である。
  BY <ref id=8412761/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths,
     CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go
  `go` が `path` に対して行うのは、`path.push(i)` -- 再帰 -- `path.pop()` の組と、
  `out.push(path.clone())` だけである。よって降下のどの位置でも `path` は呼び出し時の値にその位置まで
  降りた添字を順に足したものであり、そこで積まれる元はその写しである。A10 より
  `unpunched_field_types` を繰り返し取る歩みは有限なので、降下の深さについての帰納で、`p` から
  始めた歩みが積む元は `[]` から始めた歩みが積む元に `p` を前置したものである。`rc_units_go` の
  `Capture`・`Unit`・`Fields` の 3 腕も同じ形であり (`NoUnit` は何も積まない)、`unit_step` の
  `Fields` の腕が降りるフィールドについて同じ帰納が回る。
  `rc_units` の本体は `let mut out = vec![]; rc_units_go(ty, type_env, &mut vec![], &mut out); out`
  であり、`boxed_leaf_paths` の本体は
  `let mut out = Vec::new(); go(ty, type_env, &mut Vec::new(), &mut out); out` である。どちらも
  `path` に空の `Vec` を渡すので、最上位の呼び出しでは `path` は空列である。

<1>1. `unit_step(σ) = Fields { held_fields, .. }` である型 `σ` について、`boxed_leaf_paths` の `go` は
      `σ` の位置で最初の 4 つの `if` をすべて抜けて `for (i, fty) in ty.unpunched_field_types(type_env)`
      のループに入り、その `(i, fty)` の列は `held_fields` に等しい。
  BY CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  `unit_step` が `Fields` を返すのは `is_fully_unboxed`・`is_closure`・`is_box`・`is_union`・`is_array`・
  `is_punched_array` がすべて偽のときであり、そのとき `held_fields` は `ty.unpunched_field_types(type_env)`
  である。`go` の最初の 4 つの `if` はそれぞれ `is_fully_unboxed`・`is_closure`・`is_box`・`is_array` を
  問うので、4 つとも偽であり、`go` は同じ `ty.unpunched_field_types(type_env)` のループに入る。

<1>2. ASSUME NEW 型 `σ` (A10 を満たす)、および (帰納法の仮定) `unit_step(σ)` の `Fields` の腕が
              降りる各フィールドの型 `fty` について「`boxed_leaf_paths(fty)` の各元はちょうど 1 つの
              `rc_units(fty)` の元で始まる」が成り立つこと
      PROVE  `boxed_leaf_paths(σ)` の各元はちょうど 1 つの `rc_units(σ)` の元で始まる
  <2>1. CASE `unit_step(σ) = NoUnit`。
    BY <1>0, CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units,
       CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    `unit_step` がこれを返すのは `σ.is_fully_unboxed(type_env)` が真のときである。`rc_units_go` の
    `NoUnit` の腕は `out` に何も積まないので、<1>0 より `rc_units(σ)` は空であり、`boxed_leaf_paths`
    の `go` は最初の `if ty.is_fully_unboxed(type_env) { return; }` で返るので `boxed_leaf_paths(σ)` も
    空である。量化する元が無いので空虚に真である。
  <2>2. CASE `unit_step(σ) = Capture { capture_idx, .. }`。
    BY <1>0, CODE src/rc_ir/ownership.rs: unit_step, CODE src/rc_ir/ownership.rs: rc_units,
       CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    `unit_step` がこれを返すのは `is_fully_unboxed` が偽で `σ.is_closure()` が真のときであり、
    `capture_idx` は `CLOSURE_CAPTURE_IDX as usize` である。`rc_units_go` の `Capture` の腕は
    `path.push(capture_idx)` の後に `path.clone()` を積むので、<1>0 より最上位の呼び出しの `path` は
    空列であって `rc_units(σ) = {[capture_idx]}` である。
    `go` は最初の `if` を抜けて `if ty.is_closure()` の枝に入り、`path.push(CLOSURE_CAPTURE_IDX as
    usize)` の後に積んで返るので `boxed_leaf_paths(σ) = {[capture_idx]}` である。唯一の leaf が唯一の
    unit で始まる。
  <2>3. CASE `unit_step(σ) = Unit`。
    BY <1>0, CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go
    `rc_units_go` の `Unit` の腕は `path.clone()` を積む。<1>0 より最上位の呼び出しでは `path` は
    空列なので `rc_units(σ) = {[]}` である。`boxed_leaf_paths(σ)` のどの元も空列で始まり、unit は
    1 つしか無いので「ちょうど 1 つ」が成り立つ。
  <2>4. CASE `unit_step(σ) = Fields { held_fields, .. }`。
    BY <1>0, <1>1, 帰納法の仮定, CODE src/rc_ir/ownership.rs: rc_units,
       CODE src/rc_ir/ownership.rs: rc_units_go,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    `rc_units_go` の `Fields` の腕は `held_fields` の各 `(i, fty)` について `path.push(i)` の後に
    再帰する。<1>1 より `go` は `ty.unpunched_field_types(type_env)` -- `held_fields` に等しい列 --
    のループに入り、その各 `(i, fty)` について `path.push(i)` の後に再帰する。**その再帰が積む元を
    `[i]` の前置で読めるのは <1>0 の接頭不変性による** -- 最上位の呼び出しの `path` は空列なので、
    第 `i` フィールドについての再帰は `path = [i]` で始まり、そこが積む元は `fty_i` を最上位として
    呼んだときに積む元に `[i]` を前置したものである。よって
    `rc_units(σ) = ⊎_i {[i] ++ u : u ∈ rc_units(fty_i)}`、
    `boxed_leaf_paths(σ) = ⊎_i {[i] ++ λ : λ ∈ boxed_leaf_paths(fty_i)}` である。`[i] ++ λ` が
    `[j] ++ u` で始まるのは `i = j` かつ `λ` が `u` で始まるときに限るので、帰納法の仮定を `fty_i` に
    当てると「ちょうど 1 つ」が出る。
  <2>5. QED
    BY <2>1, <2>2, <2>3, <2>4, CODE src/rc_ir/ownership.rs: UnitStep
    場合が尽きることは `UnitStep` が `NoUnit`・`Capture`・`Unit`・`Fields` の 4 個の構成子を持つ
    ことによる。

<1>2a. A10 を満たす任意の型 `σ` について、`boxed_leaf_paths(σ)` の各元はちょうど 1 つの
       `rc_units(σ)` の元で始まる。
  BY <1>0, <1>2, <ref id=8412761/>
  A10 より `boxed_leaf_paths` も `rc_units` も停止するので、`unit_step` の `Fields` の腕が降りる
  フィールドの型の降下は有限である。<1>2 を段とする整礎帰納で、その降下で到達する各型 `σ` について
  「`boxed_leaf_paths(σ)` の各元はちょうど 1 つの `rc_units(σ)` の元で始まる」が成り立つ。

<1>3. `truncate_to_unit(σ, λ)` が値を返すとき、その値は `λ` の接頭である。
  BY CODE src/rc_ir/ownership.rs: truncate_to_unit
  `out` は空の `Vec` から始まり、`for &idx in path` のループの各回で行うのは、`NoUnit` の腕での panic、
  `Capture` の腕での `out.push(idx)` と `break`、`Unit` の腕での `break`、`Fields` の腕での
  `out.push(idx)` と続行のいずれかである。よって `out` は `path` の添字を順に並べたその接頭である。

<1>4. A10 を満たす任意の型 `σ` について、`rc_units(σ)` の各元 `u` について `u` で始まる
      `boxed_leaf_paths(σ)` の元が在る。
  BY <ref id=3597669/>, <ref id=8412761/>, <1>3
  P1 は「**A10 を満たす**任意の型 `τ`」についての言明であり、その後半より、`rc_units(σ)` の各 unit
  `u` は、ある leaf `λ ∈ boxed_leaf_paths(σ)` について `truncate_to_unit(σ, λ)` が返す値である。
  <1>3 よりその値は `λ` の接頭であり、すなわち `λ` は `u` で始まる。

<1>5. `subtree_type(τ, π) = Some(σ)` のとき、`Leaves(τ, π) = {π ++ ν : ν ∈ boxed_leaf_paths(σ)}` で
      ある。
  BY <1>0, <1>1, CODE src/rc_ir/ownership.rs: subtree_type, CODE src/rc_ir/ownership.rs: held_field_type,
     CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
  `subtree_type` は `π` の各添字 `idx` について、`unit_step(cur)` が `Fields` を返すときだけ `cur` を
  `held_field_type(held_fields, idx, "subtree_type")` へ降ろし、`NoUnit`・`Capture`・`Unit` のときは
  `None` を返す。よって `Some(σ)` は、`π` の各接頭の位置で `unit_step` が `Fields` であり、`σ` が
  `π` を降り切った先の型であることをいう。<1>1 より、`go` はそれらの位置で
  `unpunched_field_types` のループに入り、その第 `idx` フィールドの型は
  `held_field_type(held_fields, idx, ・)` と同じものである。`go` はパスを `path.push(i)` で伸ばしながら
  降りるので `path = π` になる位置はちょうど 1 つでありそこでの型は `σ` である。`Fields` の枝は
  `out.push` を持たないので `π` の真の接頭の位置では leaf を積まず、`π` で始まる leaf はすべてこの
  位置から下で積まれる。**その分が `π ++ boxed_leaf_paths(σ)` であることは <1>0 の接頭不変性による**
  -- その位置からの降下は `go` を `(σ, path = π)` で呼んだものであり、<1>0 よりそれが積む元は
  `(σ, path = [])` で呼んだときに積む元、すなわち `boxed_leaf_paths(σ)` の各元に `π` を前置した
  ものである。

<1>6. (a)。
  <2>1. CASE `subtree_type(τ, π) = None`。
    BY CODE src/rc_ir/ownership.rs: units_under
    `units_under` の `None` の腕は `vec![path.clone()]` を返す。唯一の元は `π` 自身なので `π` で
    始まり、`Leaves(τ, π)` の各元は `π` で始まるのでちょうど 1 つの元について `Leaves(τ, u)` に入る。
  <2>2. CASE `subtree_type(τ, π) = Some(σ)`。
    BY CODE src/rc_ir/ownership.rs: units_under, <1>5, <1>2a, <ref id=8412761/>, EXT `Vec::extend`
    `units_under` の `Some` の腕は `rc_units(σ)` の各 `u` について
    `let mut unit_path = path.clone(); unit_path.extend(u);` を返す。`EXT Vec::extend` より
    `unit_path` は `π` の後ろに `u` の添字を順に並べたもの、すなわち `π ++ u` であり、
    `π` で始まる。<1>5 より `Leaves(τ, π) = {π ++ ν : ν ∈ boxed_leaf_paths(σ)}` であり、`π ++ ν` が
    `π ++ u` で始まるのは `ν` が `u` で始まるときに限る。A10 の「`unpunched_field_types` を
    繰り返し取って到達する型」の節より `σ` -- `τ` から `unpunched_field_types` を繰り返し取って到達する
    型 -- も A10 を満たすので、<1>2a を `σ` に当てると、
    各 `ν` はちょうど 1 つの `u ∈ rc_units(σ)` で始まる。
  <2>3. QED
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: subtree_type
    `subtree_type` の返り値は `Some` か `None` のいずれかである。

<1>7. (b)。
  BY <1>4, <1>5, <ref id=8412761/>, CODE src/rc_ir/ownership.rs: units_under
  `subtree_type(τ, π) = Some(σ)` のとき `units_under(τ, π)` の各元は `π ++ u_0` (`u_0 ∈ rc_units(σ)`)
  である。A10 の「`unpunched_field_types` を繰り返し取って到達する型」の節より `σ` -- `τ` から
  `unpunched_field_types` を繰り返し取って到達する型 -- も A10 を満たすので <1>4 が当たり、`u_0` で
  始まる `ν ∈ boxed_leaf_paths(σ)` が在る。<1>5 より
  `π ++ ν ∈ Leaves(τ, π)` であり、`π ++ ν` は `π ++ u_0` で始まる。

<1>8. (c)。
  BY <1>0, <1>6, CODE src/rc_ir/ownership.rs: subtree_type, CODE src/rc_ir/ownership.rs: units_under,
     CODE src/rc_ir/ownership.rs: rc_units, CODE src/rc_ir/ownership.rs: rc_units_go,
     CODE src/rc_ir/ownership.rs: unit_step,
     CODE src/ast/types.rs: TypeNode::is_fully_unboxed, CODE src/ast/types.rs: TypeNode::is_box,
     DEF 例の型と op, DEF `Pair` と `make_pair`, EXT `Vec::extend`
  `subtree_type(τ, [])` は空のループを抜けて `Some(τ)` を返すので、`units_under(τ, [])` は
  `rc_units(τ)` の各元に空の接頭を足したもの、すなわち `rc_units(τ)` である
  (`EXT Vec::extend` より `unit_path` は `[]` の後ろに `u` の添字を順に並べたもの、すなわち `u` で
  ある)。長さについては、<1>0 より最上位の呼び出しの `path` は空列であり、
  `rc_units_go` の 4 つの腕より、`unit_step(τ)` が `NoUnit` なら 0、`Unit` か `Capture` なら 1、
  `Fields` なら各フィールドの `rc_units` の連結であって 0 にも 2 以上にもなる。
  **3 つの長さはどれも起きる。** `DEF 例の型と op` の `I` は `is_fully_unboxed` が真なので
  `unit_step` は `NoUnit` を返し、長さは 0 である。同じ定義の `Arr` は boxed なので
  `is_fully_unboxed` が偽・`is_closure` が偽・`is_box` が真であり、`unit_step` は `Unit` を返して
  長さは 1 である。`DEF Pair と make_pair` の `Pair` は `Arr` を 2 つ持つ unbox 構造体なので
  `unit_step` は `Fields` を返し、その 2 つのフィールドがそれぞれ長さ 1 を寄せるので長さは 2 である。
  長さが 0 のときは、
  <1>6 (a) より `boxed_leaf_paths(τ)` の各元がある unit で始まらねばならず、unit が無いので
  `boxed_leaf_paths(τ)` は空である。

<1>9. QED
  BY <1>0, <1>6, <1>7, <1>8

### 13.4 `L28` (`insert_rc` の出す path は空列であり、割った後は unit である) <!--#1ca2b50-->

**言明**。

- **(a)** `insert_rc` の出力の各 `Retain`/`Release` 節点の path は空列である。
- **(b)** したがって `split_rc_units` がそれに掛けるのは `units_under(ty(v), []) = rc_units(ty(v))` で
  あり (`L27` (c))、`P'` の各 `Retain`/`Release` 節点の path はその変数の型の unit である。その unit の
  下には boxed leaf が在る (`L27` (b))。

**証明**

<1>1. `Retain` について (a)。
  BY <ref id=19c0e5a/>
  `L9` は、`insert_rc` の出力の各 `Retain` 節点が `Retain(v, [], RcState::Unknown, k)` の形であることを
  述べる。

<1>2. `insert_rc` の出力の `Release` 節点はすべて `build_releases` の中の 1 つの式が作ったものであり、
      そこでは path が `vec![]` である。
  BY <ref id=664b958/>, CODE src/rc_ir/rc_insert.rs: build_releases
  `L8` (0) より、出力の各節点は `src/rc_ir/rc_insert.rs` の 7 つの `RcExprNode` の構成式のいずれかが
  作ったものであり、そのうち `RcExpr::Release` を作るのは `build_releases` の中の 1 つだけである。
  その式は `RcExpr::Release(v, vec![], RcState::Unknown, c)` であり、path は `vec![]` である。

<1>3. (b)。
  BY <1>1, <1>2, <ref id=1ab62dc/>, <ref id=38dde17/>, <ref id=8412761/>
  `L26` (b) より `P'` の各 `Retain`/`Release` 節点は、`P` のある `Retain(v, π)`/`Release(v, π)` 節点の
  鎖の 1 つであり、その path は `units_under(ty(v), π)` の元である。A10 より `ty(v)` は `L27` の前提を
  満たす。<1>1 と <1>2 より `π` は空列なので `L27` (c) よりそれは `rc_units(ty(v))` の元、すなわち
  `ty(v)` の unit である。`L27` (b) を `π = []` (このとき `subtree_type(ty(v), []) = Some(ty(v))`) に
  当てると、その unit の下に boxed leaf が在る。

<1>4. QED
  BY <1>1, <1>2, <1>3

### 13.5 `L29` (割る段は束縛・`origin`・別名類・`held` を変えない) <!--#abab95b-->

**言明**。`B` を `P` の 1 つの本体、`B' = split_body(B)` とする。

- **(a)** `B` と `B'` の `VarTable` について、`bindings` の鍵の集合と各鍵の束縛の形、`var_tys`、
  `param_tys`、`closure_targets` は一致する。したがって、鍵が等しい 2 つの `origin` の呼び出しの
  答えは等しく、`Origin::acted_on` と `acted_references` も引数が等しければ等しい。
- **(b)** `B'` の実行路 (D3) と `B` の実行路は 1 対 1 に対応する。対応する 2 本の路は、写し
  (`L26` (b)) の列が同じであり、各 `Match` で同じアームを選ぶ。
- **(c)** 対応する路 `ρ` と `ρ'` について、`ρ'` を辿る `B'` の各活性化 `α'` に対し、次の 4 つを持つ
  `B` の活性化 `α` が存在する。以下その 1 つを固定し、この 2 つを**分割対応の活性化**と呼ぶ。
  **README の D29 が定める「対応する活性化」は別の関係である** -- D29 が対にするのは `cancel` の出力の
  活性化と入力の活性化であり、ここで対にするのは `split_body` の前後の 2 つの本体の活性化である。
  同じ語が 2 つのものを指さないように、この文書は別の名前を置く。

  1. パラメータ・capture の値が `α'` のものと等しい。
  2. D21 が挙げる「オペランドから結果が決まらない 4 種」のうち、写し (`L26` (b)) の節点に付く位置
     では、`α` の結果が `α'` の対応する位置の結果と等しい。
  3. `B` の `Retain`/`Release` 節点の段の中で (F) の解放が `Std::FFI::Destructor` について作る
     活性化の返り値は、任意に 1 つ固定する。
  4. `B'` の各**群**の入口 (`L26` (b)) と `B` の対応する節点の入口とで、各計数下オブジェクトの参照
     カウント `H` が `α'` のものに等しい。`B` の写しの節点の段の中の各段内の点 (D24) では、`H` は
     `α'` の対応する段の対応する点のものに等しい -- **点の対応は、`L26` (b) より 2 つの節点が同じ
     種類・同じ変数・同じ右辺を持ち、第 2 項よりその位置の D21 の 4 種の結果も等しいので、A4 より
     その段の素動作の列が 2 つの側で同じであることによる。** `B` の `Retain`/`Release` 節点 `t` の
     段の中の、群の入口でも出口でもない各段内の点では、各計数下オブジェクト `O` について
     `H(O) := Σ_{C ∈ S(O)} held_α(・, C) + m(O)` と置く。ここで `S(O)` は A19 (i) の総和が走る類の
     集合 -- `obj(C) = O` であって開始の時点がその点以前であるもの -- であり、`m(O)` は、`α'` の側の
     群の入口と群の出口における `H_{α'}(O) - Σ_{C ∈ S(O)} held_{α'}(・, C)` の 2 つの値のうち
     小さい方である。

  分割対応の活性化は `ρ` と `ρ'` を辿り、対応する各写しの節点で同じ値を割り当て、各 `Match` で同じアームを
  選ぶ。したがって ρ-歩み・ρ-終端・別名類・`Anc`・`Ids`・`nm`・inhabited (D16) の判定も 2 つの側で
  一致する。

  **第 4 項の最後の文が鎖の群の中の点を点の対応によらずに置くのは、2 つの理由による。** 第 1 に、
  `B'` の鎖は `n` 個の節点、すなわち `n` 個の段に割れる (D24 の (E2)) のに対し、`B` の粗い節点は
  1 つの段である。D24 は段を不可分と定め、A17 (iii) は「環境の動作も D24 の段としてこの実行の 1 つの
  列に並ぶ。段は不可分なので、環境が動くのは段と段のあいだである」と述べるので、**D21 が挙げる
  第 3 の増減 -- 別の制御の流れの段による増減 -- が入りうる点の集合が、2 つの側で違う。** `α'` の側
  では鎖の段の境目にも入りうるが、`α` の側の 1 つの段の中には入らない。第 2 に、**2 つの側の素動作の
  列が同じ順に並ぶことを言う者が居ない** -- `t` の段の生成コードが leaf を辿る順序と、鎖が unit ごとに
  辿る順序を突き合わせるものは、この文書にも枠にも無い。**(e) が与えるのは多重集合の一致までで
  あって、順序ではない。**

  **この置き方が D21 と両立する。** D21 が `H` に課すのは「活性化自身の事象は D10 の表のとおりに `H` を
  動かす」ことと A19 (i) の制限の 2 つである。前者について、`t` の段の中で `held_α` を動かすのは
  `t` 自身の処分か生成の素動作だけであり (D34 の表の第 4 行と第 5 行。D2 より `Retain`/`Release` は
  変数を束縛しないので D10 の生成の表のどの行にも当たらず、D34 の開始値の 3 行にも当たらない)、
  その 1 つの素動作は D10 より `H(O)` を同じ向きに 1 だけ動かすので、上の等式は `held_α` と一緒に
  `H` を動かす。`m(O)` はその段の中で定数なので、群の入口・出口の値との差は D21 の第 3 の増減、
  すなわちこの活性化の外から来る分に落ちる。D21 はその分を無制約な与件に置く。後者は `<2>3b` が
  示す。

  **第 3 項が要るのは D21 が (F) をどの構文にも付けるからである。** D21 は「**(F) はどの構文でも
  起こりうる** -- 参照を処分する段はどれもオブジェクトの解放を起こしうるので、`Release` の節点も、
  消費を行う `App` や `Destructure` の節点も、この意味で外から与えられる量を持つ」と述べる。
  **どのオブジェクトが解放されるかは与件ではない** -- D21 は「`H` を与えれば、どの段でどの
  オブジェクトが解放されるかは決まる」と述べるので、第 4 項がそれを決める。与件として残るのは、その
  解放が `Std::FFI::Destructor` について作る活性化の返り値だけである。**この項の値は下流が読まない**
  -- `L31`・`L31a`・`L32` が `L29` から読むのは `held`・`μ`・別名類・inhabited の判定であり、D34 は
  記憶域を返す解放と `Destructor` が作る子の活性化の動作を、表に行を持たない素動作として挙げる。

  **「ちょうど 1 つ」ではなく「存在する」と書く。** D21 は活性化を「パラメータ・capture の値と、
  この 4 種の各位置での結果と、`H` の推移」で決めるが、第 3 項はその 1 つを任意に固定するので、
  条件を満たす `α` は複数ありうる。下流が要るのは 1 つ取れることだけである -- `L31`・`L31a`・`L32` は
  どれも「`L29` (b)(c) が与える `B` の実行路 `ρ` と分割対応の活性化を取る」と書く。

  **`α` が D21 の制限 (A19 (i) の不等式) を満たすことは、この 4 項から出る。** 下の `<1>3` `<2>3b` が
  示す。D21 はその制限を満たすものだけを活性化と呼ぶので、この段が無いと「`B` の活性化」と言えない。
- **(d)** 分割対応の活性化について、`DEF 割り当て μ` の `μ` は、`B'` の各**群**の入口 (`L26` (b)) と `B` の
  対応する節点の入口とで等しい。したがって各**計数下の**別名類 `C` について `held` も等しい
  (D34 は `held` を計数下の類にしか定めない)。
- **(e)** `B` の `Retain`/`Release` 節点 `t` の鎖の第 `i` 節点の入口における `μ` は、その群の入口の `μ`
  に、第 1 から第 `i-1` 節点が触れる inhabited なスロットの分を、`t` が `Retain` なら足し `Release` なら
  引いたものである。鎖の全体が動かすスロットの多重集合は、`B` の `t` が動かすものに等しい。

**証明**

<1>1. (a)。
  <2>1. 2 つの表の `bindings` の鍵の集合、`var_tys`、`param_tys`、`closure_targets` は等しい。
        対応する各鍵の `Binding` は同じ構成子であり、`Move`・`Field`・`Payload`・`Join` の欄は
        等しく、`Llvm` の欄は `args` の列と結果の型が等しく、`llvm_gen` は `B` の側のものを
        `rhs.clone()` が写したものである。
    BY <ref id=1ab62dc/>, CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: returned_var,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only,
       CODE src/rc_ir/borrow.rs: split_body_inner
    `collect_bindings` の `RcExpr::Retain(_, _, _, k) | RcExpr::Release(_, _, _, k) | RcExpr::Eval(_, k)`
    の腕は `collect_bindings(k, vars)` を呼ぶだけで、`bindings`・`var_tys`・`param_tys`・
    `closure_targets` に何も入れない。表に何かを入れるのは `Let` と `Destructure` の腕であり、`Match` の
    腕は各アームの `payload` と `tag` と `returned_var(&arm.body)` を読む。`returned_var` は
    `Retain`/`Release`/`Eval`/`Let`/`Destructure` の継続を辿って `Ret` の変数を返す。`L26` (b) より
    `B'` は `B` の `Let`・`Destructure`・`Match` の構造と、各アームの `tag`・`payload`・
    `payload_state` と、各 `Ret` の変数をそのまま持つので、2 つの表は上の形で一致する。`Llvm` の欄に
    ついては、`split_body_inner` の `Let(x, rhs, k)` の腕が右辺に `rhs.clone()` を据えるので、
    `B'` の側の `llvm_gen` は `B` の側のものの写しであり、`args` と結果の型 (`x.ty`) は等しい。
    `VarTable::of` はこれにパラメータと capture の `Param` 束縛を足すだけであり、`L26` (a) より
    `params` と `capture` は変わらない。
  <2>2. 対応する 2 つの `Binding::Llvm` の欄について、
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の返り値は等しい。ここで `arg_tys` は
        `args` の各元の型の列である。
    BY <2>1, <ref id=e11772a/>
    <2>1 より `args`・結果の型・`type_env` は 2 つの側で等しい。`rhs.clone()` は右辺を欄ごとに写す
    ので、複製の各欄は原本のものに等しく、A3 の「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は自分の
    `FullName` の欄を読まない。**」の節より、答えは `self` が持つ名前にも依らない。A3 の決定性の節 --
    「**`result_prov`、`borrows_operand`、`applies_a_function_operand` は決定的である** -- 同じ引数に対して常に同じ値を返す」-- より、
    2 つの呼び出しは同じ `Provenance` を返す。
  <2>3. `Org_B(x, π)` を、`vars_B` と `type_env` を第 1・第 2 引数として `origin` が鍵 `(x, π)` に
        返す値とする。この値は 1 つに決まり、`origin_inner(vars_B, type_env, x, π)` の値に等しい --
        ただしその中の各 `origin` の呼び出しは、その鍵の `Org_B` を返すものとして読む。`vars_{B'}` に
        ついて同じものを `Org_{B'}` と書く。
    BY <ref id=0edb0ba/>, <ref id=b1f6e13/>, <ref id=8e3aff3/>, <ref id=33c54dc/>, <ref id=3905b4e/>, <ref id=1ab62dc/>, CODE src/rc_ir/ownership.rs: origin,
       EXT 空を作る `Default`,
       CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: VarTable::empty,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only
    **P2a の制限がここで満たされる。** P2a は「**1 つの `VarTable` の値 `vars` と 1 つの `TypeEnv` の
    値を固定する。**」と述べ、「**`vars` は、A6 と A11 を満たすプログラムの本体について
    `VarTable::of` か `VarTable::body_only` が作った表である。**」と限る。`vars_B` と `vars_{B'}` は
    それぞれ `B` と `B'` についてその 2 つの構成子が作る表であり、この段はどちらか一方を固定した中で
    P2a を読む -- P2a は「**表を跨ぐ形はこの命題の主張ではない。**」と述べ、
    「**`bindings` が等しい相異なる 2 つの `VarTable` について答えが等しいことは別の主張であり、
    それを要る段は自分で示す。**」と続けるので、2 つの表を跨ぐ等式はこの段の主張ではない。
    A6 と A11 は `borrow_ify` の入力にかかる仮定だが、A6 と A11 の脇と A2 の
    「**したがって、`borrow_ify` の入力について語る仮定は、`insert_rc` の入力と出力についても
    読める。**」より `B` -- `insert_rc` の出力 -- についてそれを読め、`L26` (b) より `B'` は `B` の
    束縛変数と `Match` のアームをそのまま持つので `B'` についても読める。
    P2 より `origin` はこの引数について答えを返して停止し、P2a より鍵が等しい 2 つの呼び出しの返り値は
    等しいので、値は `vars.origins` が保持する memo の状態に依らず 1 つに決まる。`origin` は memo に
    鍵が無いとき `origin_inner` を走らせてその値を返す。**空の memo からその鍵について呼ぶ計算が
    在ることは、表の作られ方による** -- `VarTable::of` と `VarTable::body_only` はどちらも
    `VarTable::empty()` から始め、`empty` は `origins` に `RefCell::default()` を置く。
    `EXT 空を作る Default` よりそれは `Map::default()` -- 空の写像 -- を包んだ `RefCell` なので、
    その表を作った直後にはどの鍵の項も無い。よってその鍵についての最初の呼び出しが
    `origin_inner` を走らせ、その値が等式を与える。その中の各 `origin` の呼び出しが返す値は、
    P2a よりその鍵の `Org_B` である。
  <2>4. 1 つの計算の中で memo に当たった `origin` の呼び出しが読む項は、**同じ鍵についてそれより先に
        戻った呼び出し**が入れたものである。
    BY <ref id=3e6b0e0/>, 前提 `origins` の memo を書く式, EXT 非公開の項目の可視範囲,
       CODE src/rc_ir/ownership.rs: origin, CODE src/rc_ir/ownership.rs: VarTable,
       CODE src/rc_ir/ownership.rs: VarTable::of, CODE src/rc_ir/ownership.rs: VarTable::body_only
    `origin` は `vars.origins` を鍵で引き、項が在ればその値を返し、無ければ `origin_inner` を走らせて
    (A15 より `grow_stack` はその閉包をちょうど 1 回呼ぶ) その答えを同じ鍵で表に入れ、続けて返る。
    **`origins` が非公開の欄であることは `VarTable` の宣言による** -- その欄は
    `origins: RefCell<Map<VarPath, Origin>>` と書かれていて `pub` を持たない (同じ宣言の
    `param_tys` と `var_tys` は `pub(crate)` を持つ)。`EXT 非公開の項目の可視範囲` よりそれに項を
    入れる式は `VarTable` を定めるモジュールの中にしかない。`前提 origins の memo を書く式` より、
    その欄を可変に借りる式を持つ項目は `origin` だけであり、そこに在るのは `origin_inner` の答えを
    同じ鍵で入れる 1 つの式である。よって読まれた項を入れたのは同じ鍵についての呼び出しである。
    その呼び出しは `insert` の直後に返るので、項が読まれる時点で既に戻っている。
  <2>5. P2 が範囲に入れる各鍵 `(x, π)` について `Org_B(x, π) = Org_{B'}(x, π)` である。
    BY <2>1, <2>2, <2>3, <2>4, <ref id=0edb0ba/>, <ref id=b1f6e13/>, <ref id=3e6b0e0/>, EXT 呼び出しの入れ子,
       CODE src/rc_ir/ownership.rs: origin,
       CODE src/rc_ir/ownership.rs: origin_inner,
       CODE src/rc_ir/ownership.rs: origin_from_leaves_under,
       CODE src/rc_ir/ownership.rs: as_arg_projection,
       CODE src/rc_ir/ownership.rs: truncate_to_unit,
       CODE src/rc_ir/ownership.rs: Origin,
       CODE src/rc_ir/ownership.rs: Origin::acted_on,
       CODE src/rc_ir/ownership.rs: Origin::of_candidates,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at,
       CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       EXT `Iterator::all`, EXT 導出した `PartialEq`
    **これは 2 つの `VarTable` を跨ぐ形なので、P2a は使えない。** P2a は
    「**表を跨ぐ形はこの命題の主張ではない。** `bindings` が等しい相異なる 2 つの `VarTable` について
    答えが等しいことは別の主張であり、それを要る段は自分で示す。」と述べる。この段がその義務を果たす --
    P2a を引くのは、`vars_B` を固定した中と `vars_{B'}` を固定した中の、それぞれ 1 つの表の内側だけで
    ある (<2>3)。
    鍵 `(x, π)` を取る。P2 の言明は `vars.origins` の状態に条件を置かず、範囲に入る `(x, π)` について
    `origin(x, π)` が panic せずに答えを返して停止すると述べるので、どの memo の状態から呼んでも
    それが成り立つ。とくに空の memo と `vars_B` から `(x, π)` について `origin` を呼ぶ計算は
    停止し、`EXT 呼び出しの入れ子` よりその計算が行う呼び出しは有限個で、そのどれもが戻る。**この計算の
    各呼び出しについて、その返り値がその鍵の `Org_{B'}` に等しいことを、戻る順の帰納で示す。** 根の
    呼び出しにこれを読むと、P2a よりその返り値は `Org_B(x, π)` なので言明が出る。
    呼び出しは 2 種である。memo に当たった呼び出しについては、<2>4 よりその項を入れたのは同じ鍵に
    ついてそれより先に戻った呼び出しであり、帰納法の仮定よりその返り値はその鍵の `Org_{B'}` である。
    残るのは `origin_inner(vars_B, type_env, y, σ)` を走らせる呼び出しであり (A15 より `grow_stack` は
    その閉包をちょうど 1 回呼ぶ)、その中で行われる各 `origin` の呼び出しは
    `EXT 呼び出しの入れ子` よりこの呼び出しより先に戻るので、帰納法の仮定よりその返り値はその鍵の
    `Org_{B'}` である。<2>3 を `vars_{B'}` に読むと、示すべきことは、2 つの側の `origin_inner` が同じ腕を
    通り、同じ鍵を問い、問うた鍵の答えが同じであれば同じ値を返すことである -- それが言えれば、`B` の側の
    返り値は `origin_inner(vars_{B'}, type_env, y, σ)` を `Org_{B'}` で読んだ値、すなわち
    `Org_{B'}(y, σ)` に等しい。以下、腕ごとにそれを見る。`origin_inner` はまず
    `vars.bindings.get(var)` で腕を選び、<2>1 より鍵の集合と構成子は等しいので、2 つの側は同じ腕に
    入る。
    - `None`・`Binding::Param`・`Binding::Producer` の腕は `Exactly((var, path))` を返す。引数だけで
      決まるので等しい。
    - `Binding::Move(y)` の腕は `origin(vars, type_env, &y.name, path)` を返す。<2>1 より `y` は等しく、
      帰納法の仮定よりこの値は等しい。
    - `Binding::Join(arm_results)` の腕は、各 `arm_result` について
      `origin(vars, type_env, &arm_result.name, path).acted_on()` の元を `candidates` に入れ、
      `Origin::of_candidates(candidates, &(var, path))` を返す。<2>1 より `arm_results` は等しく、
      帰納法の仮定より各 `origin` の値は等しい。`acted_on` は `Origin` の値から決まるので
      `candidates` は集合として等しく、`of_candidates` はその集合と第 2 引数から値を決めるので、
      返り値は等しい。
    - `Binding::Llvm(llvm_gen, args, result_ty)` の腕は、<2>2 より同じ `decl` を得る。
      `decl.leaf_origins_at(path).and_then(as_arg_projection)` は `decl` と `path` から決まる。
      `Some((j, p))` のときは `origin(vars, type_env, &args[j].name, &p)` を返し、<2>1 より `args` は
      等しいので帰納法の仮定より値は等しい。`None` のときは
      `origin_from_leaves_under(vars, type_env, &decl, args, path, &(var, path))` を呼び、それが
      `None` を返せば `Exactly((var, path))` を返す。`origin_from_leaves_under` は
      `decl.leaf_origins_under(path)` と `truncate_to_unit(&args[j].ty, leaf, type_env)` から
      `operand_units` と `produced_here` を作り、各 `(j, unit)` について
      `origin(vars, type_env, &args[j].name, &unit)` を集めて `reached` とし、`produced_here` のとき
      `Exactly(here)` を足す。`operand_units` と `produced_here` は `decl`・`args` の型・`type_env`
      から決まり、`origin` の値は帰納法の仮定より等しいので、`reached` の元の集合は 2 つの側で等しい。
      返り値はその集合から決まる -- `reached` の元がすべて等しいときはその共通の値であり、そうでない
      ときは各元の `acted_on` を集めた**集合**を `of_candidates` に渡したものである。どちらも
      `reached` の並べ方に依らない。`reached.iter().all(|o| o == first)` は、`EXT Iterator::all` より
      「`reached` のすべての元が第 1 元に等しい」であり、`EXT 導出した PartialEq` より `Origin` の
      等号は同値関係 -- `Set<VarPath>` の欄も集合として比べる -- なので、これは「元がすべて互いに
      等しい」と同値であって、並べ方に依らない。
    - `Binding::Field(container, idx)` の腕は `container.ty.is_box(type_env)` で枝が分かれ、<2>1 より
      `container` と `idx` は等しいので 2 つの側は同じ枝に入る。boxed の枝は `Exactly((var, path))` を
      返し、unbox の枝は `origin(vars, type_env, &container.name, &([idx] ++ path))` を返すので、
      帰納法の仮定より等しい。
    - `Binding::Payload(scrut, variant)` の腕も同じ形である。<2>1 より `scrut` と `variant` は等しい
      ので同じ枝に入り、`None` と `Some(tag)` (unbox) の枝は `origin` の値を返すので帰納法の仮定より
      等しく、`Some(_)` (boxed) の枝は `Exactly((var, path))` を返す。
    どの腕でも 2 つの側は同じ鍵を問い、同じ値を返す。
  <2>6. QED
    BY <2>1, <2>3, <2>5, CODE src/rc_ir/ownership.rs: Origin::identity,
       CODE src/rc_ir/ownership.rs: Origin::acted_on,
       CODE src/rc_ir/ownership.rs: acted_references,
       CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths
    <2>1 が `bindings`・`var_tys`・`param_tys`・`closure_targets` についての節を、<2>3 と <2>5 が
    `origin` の答えについての節を与える。`Origin::acted_on` は `Origin` の値から決まるので、答えが
    等しければ等しい。`acted_references(vars, type_env, v, path)` は
    `boxed_leaf_paths(&v.ty, type_env)` のうち `path` で始まる各 leaf の
    `origin(vars, type_env, &v.name, &leaf).identity()` を数えたものであり、`boxed_leaf_paths` は型と
    `type_env` から決まるので、引数が等しい 2 つの呼び出しの値は等しい。

<1>2. (b)。
  BY <ref id=1ab62dc/>, <ref id=ca36627/>
  D3 の規則で路が分かれるのは `Let(x, Match(v, arms), k)` の行だけである。`L26` (b) より `B'` の
  `Match` 節点とそのアームは `B` のものと 1 対 1 に対応し、アームの本体も `split_body` の像として
  対応する。`Retain` と `Release` は継続をちょうど 1 つ持つ (D2) ので路を分けない。よってアームの
  選び方が路を決め、2 つの側で同じ選び方が対応する。

<1>2a. `B` の `Retain`/`Release` 節点 `t` の鎖の全体が動かすスロットの多重集合は、`B` の `t` が
       動かすものに等しい。
  BY <ref id=1ab62dc/>, <ref id=38dde17/>, <ref id=8412761/>, DEF 割り当て `μ`, <ref id=66c9670/>
  `L26` (b) より鎖の第 `i` 節点は `Retain(v, u_i)` (または `Release(v, u_i)`) であり、`DEF 割り当て μ`
  より `u_i` の下の inhabited な各 leaf `(v, λ)` の `μ` を ±1 する。A10 より `ty(v)` は `L27` の前提を
  満たす。`L27` (a) より `Leaves(ty(v), π)` は
  `Leaves(ty(v), u_1), …, Leaves(ty(v), u_n)` の直和であり、直和は inhabited なものに制限しても
  直和である。よって鎖の全体が動かすスロットの多重集合は、`B` の `t = Retain(v, π)` (または
  `Release(v, π)`) が動かす「`π` の下の inhabited な各 leaf」に等しい。

<1>3. (c)。
  <2>0. `B` の `Retain`/`Release` 節点を実行する段は、D24 の (E7) -- グローバルの初期化の段 -- を
        含まない。
    BY <ref id=a502f3e/>, <ref id=243ae2c/>, <ref id=e3436e8/>, <ref id=56c2068/>, <ref id=ca36627/>, <ref id=d80dde9/>, <ref id=664b958/>, <ref id=dca1c02/>, EXT クレートの項目, EXT 非公開の項目の可視範囲,
       CODE src/rc_ir/rc_insert.rs: build_retains, CODE src/rc_ir/rc_insert.rs: build_releases,
       CODE src/rc_ir/rc_insert.rs: insert_rc,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_operation_let,
       CODE src/rc_ir/rc_insert.rs: RcInserter::retain_if_live,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_eval,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_destructure,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_func,
       CODE src/rc_ir/rc_insert.rs: RcInserter::insert_into_match,
       CODE src/rc_ir/rc_insert.rs: insert_if_local, CODE src/rc_ir/rc_insert.rs: free_locals,
       CODE src/rc_ir/ownership.rs: collect_bindings, CODE src/rc_ir/ownership.rs: VarTable::of
    (E7) が走るのは、まだ初期化されていないグローバルを読む者が居るときである (D22 の
    「グローバルのアクセサ」は初期化済みの旗を見る)。局所名の変数を名指す `Retain`/`Release` は
    グローバルの記憶域を読まないので、(E7) を起こしうるのは局所名でない変数を名指す節点だけである。
    `L8` (0) より、`insert_rc` の出力の `Retain`/`Release` 節点は `build_retains` と
    `build_releases` の中の 1 つずつの式が作ったものに限る。
    **数えるのは、その 2 つの関数へ変数を渡す式である。** `L8` (a) より `build_retains` の呼び出し元は
    4 か所であり、そのうち `insert_into_operation_let` の呼び出しは `retains_before` を、残る
    3 か所 (`Ret` の腕・`insert_into_destructure`・`insert_into_match`) は `retain_if_live` の
    唯一の引数を渡すので、渡す式は 4 つである。`build_releases` の呼び出し元は 6 か所であり
    (`build_releases` はこのモジュールの非公開の項目なので、`EXT 非公開の項目の可視範囲` より
    それを呼ぶ式はこのモジュールの中にしかなく、`EXT クレートの項目` よりこのモジュールの項目は
    このファイルに書かれたものだけである)、
    渡す式は 9 つである -- `insert_into_func` の `unused` (1)、`insert_into_operation_let` の
    `after` に入る `releases_after` と `x` (2)、`insert_into_eval` の `vec![x]` (1)、
    `insert_into_destructure` の `dead` (1)、`insert_into_match` の `head` に入る dead-branch・
    容器解放・payload (3)、`insert_into_match` の `Let` の束縛変数 `vec![x]` (1)。合わせて 13 の式が
    あり、これが `Retain`/`Release` 節点の名指す変数の出どころを尽くす。13 は次の 3 群に分かれる。
    **第 1 群 (7 つ): `is_local()` の門の中に在るもの。** `insert_into_operation_let` の
    `retains_before` と `releases_after` は `if v.name.is_local()` の門の中に在り (2)、
    `retain_if_live` の 3 か所は `var.name.is_local()` を要求し (3)、`insert_into_eval` の
    `vec![x]` も `x.name.is_local()` を要求し (1)、`insert_into_match` の dead-branch が名指すのは
    `live_at_arm_head` の名前で、その集合は `insert_if_local` と `free_locals` が作るので局所名だけ
    からなる (1)。
    **第 2 群 (5 つ): 名指す名前が `vars.bindings` に束縛を持つもの。**
    `insert_into_operation_let` の `after` に入る `x` と `insert_into_match` の `Let` の束縛変数は
    どちらも `Let` の束縛変数 (2)、`insert_into_destructure` の `dead` は `Destructure` のフィールド
    変数 (1)、`insert_into_match` の payload はアームの payload 変数 (1) であり、`collect_bindings` は
    この 3 種を `vars.bindings` に入れる。`insert_into_func` の `unused` はパラメータと capture を
    名指し (1)、`VarTable::of` がその名前を `vars.bindings` に入れる (グローバル初期化子の `init` は
    パラメータも capture も持たないので、この位置は関数についてだけ在る)。よって `L15` (d) の対偶より、
    この 5 つが名指す名前はどれも局所名である。
    **第 3 群 (1 つ): 残る `insert_into_match` の容器解放。** boxed union の変位アームの頭に置く
    `Release(scrut, [])` (`release_container && arm.tag.is_some() && needs_rc(&scrut)` の枝) である。
    その `scrut` が局所名でないグローバルであるとき、D7 の読む構文の表の
    `Let(x, Match(v, arms), k)` の行より
    その `Match` 節点が `scrut` を読み、D3 よりその節点は `ρ` の上でこの `Release` より前に
    あるので、そのグローバルの初期化はその `Match` 節点の段で済んでいる。
    第 1 群と第 2 群が名指すのは局所名であり、第 3 群の 1 つは (E7) を起こさない。
  <2>1. `B` の活性化を決めるデータは、言明の 4 項がすべて与える。
    BY <2>0, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=b3dfa37/>, <ref id=1ab62dc/>, <ref id=c422d87/>
    D21 は「すなわち 1 つの本体の活性化は、**パラメータ・capture の値と、この 4 種の各位置での結果と、
    `H` の推移**を与えると決まり、辿る実行路もそれで決まる」と述べる。言明の第 4 項が `H` の推移を、
    第 1 項がパラメータ・capture の値を与えるので、残るのは 4 種の各位置での結果である。
    `L26` (b) より `B` の節点は写しに対応するものと
    `Retain`/`Release` のものに尽きる。写しの節点に付く 4 種の位置は第 2 項が与える。
    `Retain`/`Release` の節点に付きうる 4 種は (F) だけであり、その解放が作る活性化の返り値を
    第 3 項が、解放がどのオブジェクトについて起きるかを第 4 項の `H` が与える。まず、残る
    3 種 (一意性の観測点、外部の状態を読む `Llvm` の演算、実行時の参照カウントで分岐する `Llvm` の
    演算) はいずれも `Llvm` 節点に付く -- **一意性の観測点については D18 が
    「RC IR ではそれぞれ 1 つの `Llvm` 演算として現れる」と述べ**、残る 2 つは D21 の本文が
    `Llvm` の演算と書く -- ので、D2 より `Retain`/`Release` は `Llvm` の右辺を持たないから、
    どれも `Retain`/`Release` 節点には付かない。
    第 4 種「子の活性化を作る段」は D24 の活性化の林が 5 種と数え上げる -- (E1)、(E3)、(E7)、
    オペランドを適用する `Llvm` の段、(F) の解放が `Destructor` について作る段である。(E1) は環境が
    活性化の根を作る段であって節点に付かず、(E3) は `App` の段、オペランドを適用する `Llvm` の段は
    `Llvm` の段なので、D2 よりどれも `Retain`/`Release` 節点には付かない。(E7) は <2>0 が除く。
  <2>2. 第 3 項と第 4 項が与えるものは、`t` の位置の (F) の結果として成り立つ。
    BY <ref id=e3436e8/>, <ref id=c232680/>, 言明の第 3 項, 言明の第 4 項
    D24 の (F) は解放を段の一部と定め、参照を処分するどの段でも起きうるとする。**どのオブジェクトが
    解放されるかは与件ではない** -- D21 は「**解放が起きること自体は与件ではない。** 4 種目が与件に
    するのは、子の活性化が返す値と、それが `H` に与える変化である。(F) の解放はその結果として `H` が
    0 になった段で起きる (D24 の (F))。すなわち `H` を与えれば、どの段でどのオブジェクトが解放されるかは
    決まる」と述べる。言明の第 4 項が `t` の段の中の各段内の点の `H` を与えるので、その段で解放される
    オブジェクトはそれで決まる。残る与件は、その解放が `Std::FFI::Destructor` について作る活性化の
    返り値であり、言明の第 3 項がそれを 1 つ固定する。
  <2>3. 分割対応の活性化は `ρ` と `ρ'` を辿り、対応する各写しの節点で同じ値を割り当て、各 `Match` で
        同じアームを選ぶ。
    BY <2>1, <2>2, <1>2, <1>2a, <ref id=1ab62dc/>, <ref id=b3dfa37/>, <ref id=c232680/>, <ref id=3f1bb47/>
    群の境界についての帰納。基底は本体の根の群の入口であり、第 1 項よりパラメータ・capture の値は
    等しい。段は 2 つある。写しの群では、`L26` (b) より 2 つの節点は同じ種類・同じ変数・同じ右辺を
    持ち、第 2 項よりその位置の D21 の 4 種の結果も等しいので、A4 と D21 より同じ値を割り当て、
    同じアームを選ぶ (`Match` の節点では <1>2 が路の対応を与える)。鎖の群では、D2 より
    `Retain`/`Release` は変数を束縛しないので値を割り当てず、<1>2a より鎖の全体は `B` の `t` と
    同じスロットの多重集合を動かす。
  <2>3a. 群の入口では、各計数下オブジェクト `O` について
         `Σ_{s : obj(s) = O} μ(s)` が 2 つの側で等しい。
    BY <2>3, <1>2a, <ref id=1ab62dc/>, <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=f06144e/>, DEF 割り当て `μ`
    群の入口についての帰納。基底は本体の根の群の入口であり、そこで `μ` を動かした事象は D10 の
    初期値だけで、言明の第 1 項よりパラメータ・capture の値は等しいので両側で同じである。段は
    2 つある。写しの群では、`L26` (b) より 2 つの節点は同じ種類・同じ変数・同じ右辺を持ち、<2>3 より
    同じ値を割り当てるので、`DEF 割り当て μ` の残る 4 種 (D10 の生成、D9 の消費、D9 の移動、および
    D10 の初期値) は対応する位置で同じ leaf を同じだけ動かす。鎖の群では、<1>2a より鎖の全体が
    動かすスロットの多重集合は `B` の粗い節点が動かすものに等しく、`DEF 割り当て μ` の第 3 行と第 4 行より
    向きも同じである。D2 より `Retain`/`Release` は変数を束縛しないので、`obj(s)` も両側で同じである
    (<2>3)。
  <2>3b. `α` は D21 が活性化に課す制限 -- 各時点と各段内の点 (D24) での A19 (i) の不等式 -- を
         満たす。したがって `α` は `B` の活性化である。
    BY <1>2a, <2>2, <2>3, <2>3a, <ref id=9f1cf6c/>, <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=1ab62dc/>, <ref id=f06144e/>, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=88a06de/>, <ref id=b3dfa37/>, <ref id=30d6238/>, <ref id=9d5d254/>, <ref id=3f1bb47/>, DEF 割り当て `μ`,
       言明の第 2 項, 言明の第 4 項
    `L15` (e) と `L26` (a) より 2 つの側のどちらでもすべての unit が所有されるので、`L13a` (c) より
    どの類でも `β(C) = 0` であり、A19 (i) の角括弧は 0、`d(C) = held(・, C)` である。A19 (i) の総和
    `S` は「`obj(C) = O` であり開始の時点がその時点以前であるもの」を走るので、A19 (i) は各計数下
    オブジェクト (D26) `O` について `H(O) ≥ Σ_{C ∈ S(O)} held(・, C)` と読める。以下この差
    `Δ(O) := H(O) - Σ_{C ∈ S(O)} held(・, C)` を追う。**追うのは `μ` ではなく `held` である** --
    `DEF 割り当て μ` が `μ` を定めるのは節点の訪問の入口についてだけであり、D34 は `held` を段内の点でも
    定める。
    **群の入口では `Δ(O)` が両側で等しい** -- 群の入口は節点の入口 (`L26` (b)) すなわち `ρ` の上の
    時点なので、`L13a` (b)(c) より `held(・, C) = Σ_{s ∈ C} μ(s)` であり、開始していない類のスロットは
    まだ値を得ていないので `DEF 割り当て μ` の 6 種のどれも動かさず `μ = 0` を寄せる。よって
    `Σ_{C ∈ S(O)} held(・, C) = Σ_{s : obj(s) = O} μ(s)` であり、<2>3a がその等しさを、言明の第 4 項が
    `H(O)` の等しさを与える。`α'` は `B'` の活性化なので D21 の制限を満たし、その点で `Δ(O) ≥ 0` で
    ある。
    **写しの群の中の段内の点でも等しい** -- `L26` (b) より 2 つの節点は同じ種類・同じ変数・同じ右辺を
    持ち、<2>3 より同じ値を割り当て、言明の第 2 項よりその位置の D21 の 4 種の結果も等しいので、
    その段の素動作の列は両側で同じである。D34 は表の各行の事象を**それを運ぶ素動作の直後の段内の点**に
    置き、開始値を参照が `Obl(a)` に入る素動作の直後の段内の点に置くので、`held` と `S(O)` の動きは
    その素動作の列で決まり、`H` の動きも同じ列で決まる。よって群の入口で等しい
    `Δ(O)` はその段の各点でも等しく、`α'` の側で 0 以上なので `α` の側でも 0 以上である。
    **鎖の群の中の、群の入口でも出口でもない段内の点では `Δ(O) = m(O) ≥ 0` である。**
    言明の第 4 項がその点の `H(O)` を `Σ_{C ∈ S(O)} held_α(・, C) + m(O)` と置くので、その点の
    `Δ(O)` はちょうど `m(O)` である。`m(O)` は `α'` の側の群の入口と群の出口における `Δ(O)` の
    小さい方であり、`L26` (b) より鎖の群の次には必ず別の群があるので群の出口は次の群の入口、すなわち
    `ρ'` の上の時点である。群の入口も `ρ'` の上の時点なので、`α'` が `B'` の活性化であること
    (D21 の制限) からその 2 つの値はどちらも 0 以上であり、`m(O) ≥ 0` である。
    **この形にするのは、2 つの側の素動作の列が同じ順に並ぶことを言う者が居ないからである** (言明の
    脇) -- `<1>2a` と `L29` (e) が与えるのは動かすスロットの多重集合の一致までであって、順序では
    ない。上の置き方は `α` の側の値を `α'` の側の点と対応させずに決めるので、順序を要らない。
    **群の出口の `Δ(O)` が両側で等しいことは、群の入口についての議論をそこへ当てたものである** --
    群の出口は次の群の入口だからである。
    D24 より段内の点はその段の素動作の列の切れ目であり、段と段のあいだの時点はその段の最初の段内の点
    なので、以上で `α` の各時点と各段内の点が尽きる。

  <2>4. QED
    BY <2>0, <2>1, <2>2, <2>3, <2>3a, <2>3b, <1>1, <ref id=30d6238/>, <ref id=d59f90b/>, <ref id=9c7c27a/>, <ref id=66c9670/>,
       CODE src/rc_ir/ownership.rs: Origin::identity
    <2>1 と <2>3b より、言明の 4 項を持つ `B` の活性化 `α` が存在する。
    **ρ-歩みが 2 つの側で一致することは D33・D20・D17 から出る。** D33 は歩みの 1 歩を「`x` の束縛が
    `λ` について D20 の別名の辺を持つ間は、その辺の行き先 -- D17 が定める、`λ` に対応する位置 -- へ
    進む」と定める。D20 の辺の一覧は束縛の 6 種で決まり、<1>1 (a) より 2 つの表は `bindings` の鍵の
    集合と各鍵の束縛の形を共有するので、どの位置でも辺の有無と種類が一致する。D17 の行き先のうち
    活性化に依るのは `Binding::Join` の辺だけであり、それは活性化が選んだアームの結果へ進むので、
    <2>3 より 2 つの側で同じである。よって歩みは一致し、ρ-終端も別名類も一致する。
    `nm(s) = origin(v, λ).identity()` は `origin` の答えから決まり、<1>1 (a) よりその答えは 2 つの
    側で等しいので `nm` は一致し、`Anc`・`Ids` は `nm` と歩みから定まるので一致する。
    inhabited (D16) は値の実行時のタグで決まり、<2>3 より値が一致するので
    一致する。

<1>4. (e)。
  BY <1>2a, <ref id=1ab62dc/>, <ref id=b3dfa37/>, DEF 割り当て `μ`, <ref id=66c9670/>
  `L26` (b) より鎖の第 `i` 節点は `Retain(v, u_i)` (または `Release(v, u_i)`) であり、`DEF 割り当て μ`
  より `u_i` の下の inhabited な各 leaf `(v, λ)` の `μ` を `+1` (または `-1`) する。D2 より
  `Retain`/`Release` は継続をちょうど 1 つ持つので鎖の節点は継続の順に実行され、第 `i` 節点の入口
  までに第 1 から第 `i-1` 節点の分が動いている。言明の第 2 文は <1>2a である。

<1>5. (d)。
  BY <1>3, <1>4, <ref id=c2ea78f/>, <ref id=dca1c02/>, DEF 割り当て `μ`, <ref id=1ab62dc/>
  `DEF 割り当て μ` の 6 種の事象のうち `Retain` と `Release` の 2 種は鎖の節点にだけ付き、残る 4 種
  (D10 の初期値、D10 の生成、D9 の消費、D9 の移動) は写しの節点にだけ付く -- `L26` (b) より `B'` の
  `Retain`/`Release` 以外の節点は写しであり、D10 の生成の表と D9 の 2 つの表に `Retain`/`Release` の
  行は無い。<1>3 より対応する写しの節点は同じ値を割り当てるので、この 4 種は対応する位置で同じ leaf に
  ついて同じだけ動かす。<1>4 より鎖の群の全体は `B` の `t` と同じだけ動かす。よって群の入口についての
  帰納 -- 基底は本体の根の群の入口 (D10 の初期値は 2 つの側で同じ) -- で、`B'` の各群の入口の `μ` は
  `B` の対応する節点の入口の `μ` に等しい。`L15` (e) と `L26` (a) より 2 つの側のどちらでもすべての
  unit が所有されるので `L13a` (c) が当たり、`held` は `L13a` (b) より `Σ_{s ∈ C} μ(s)` である。
  <1>3 より別名類も等しいので `held` も等しい。

<1>6. QED
  BY <1>1, <1>2, <1>2a, <1>3, <1>4, <1>5

### 13.6 (A19 (i) はこの段に何も要求しない)

A19 (i) は仮定ではなく、D21 が活性化に課す制限である (README の A19、D21)。実行 (D24) が作る活性化が
その制限を満たすことは P28 (b) が示すので、この段について示すものは無い。

**P28 が言明を限る 2 つの前提は、この段の出力について満たされる。** P28 は「**D12 の意味で RC 規律を
満たすプログラム**であって、**借用する unit を持つ本体の活性化を作る段が (E3) に限られる**もの」に
言明を限る。後者はこの段の出力について空虚に成り立つ -- `L15` (e) と `L26` (a) より
`split_rc_units` の出力のすべての関数の `borrowed_units` は空であり、D14 より借用する unit を持つ
本体が 1 つも無い。前者は A1 が与える -- A1 は「`borrow_ify` に渡されるプログラムは D12 の意味で
RC 規律を満たす」と述べ、その脇が「**「`borrow_ify` に渡されるプログラム」と「`split_rc_units` の
出力」は同じプログラムである**」と述べる。P28 が範囲を「`insert_rc` の入力から `cancel` の出力までの
どこかに現れるもの」に限ることも、この出力は満たす。

この段の出力には借用する
unit が無い (`L15` (e)、`L26` (a)) ので、`L13a` (c) より `β ≡ 0` であり、その本体のどの別名類でも
(i) の角括弧は 0 である。

### 13.7 `L31` (A19 (ii-a) の保存) <!--#f970a93-->

**言明**。`P` が A19 (ii-a) を満たすならば、`P' = split_rc_units(P)` も A19 (ii-a) を満たす。

**証明**

`B'` の 1 つの実行路 `ρ'` と、それを辿る 1 つの活性化を固定し、`L29` (b)(c) が与える `B` の実行路 `ρ`
と分割対応の活性化を取る。計数下の別名類 `C` を固定する。`L29` (c) より `C` は 2 つの側で同じ類である。

<1>0. `B` と `B'` のどちらの側でも、各時点と各計数下の別名類 `C` について
      `held(・, C) = Σ_{s ∈ C} μ(s)` である。したがって、`C` のどのスロットの `μ` も下げない事象は
      `held(・, C)` を下げず、どのスロットの `μ` も上げない事象は `held(・, C)` を上げない。
  BY <ref id=c2ea78f/>, <ref id=dca1c02/>, <ref id=1ab62dc/>
  `L15` (e) より `P` のすべての関数の `borrowed_units` は空であり、`L26` (a) より
  `split_rc_units` はそれを変えないので `P'` についても空である。よって 2 つの側のどちらでも
  すべてのパラメータ・capture の unit が所有され、`L13a` (c) が当たる (グローバル初期化子の `init` の
  本体も `L13a` (c) が覆う)。

<1>1. 群 (`L26` (b)) の入口では `held_{ρ'}(・, C) = held_ρ(・, C)` であり、その値は仮定より 0 以上で
      ある。
  BY <ref id=abab95b/>, 仮定
  `L29` (d) が等式を与える。群の入口は `B` の対応する節点の入口に対応し、そこで (ii-a) の前半を
  `B` に当てると `held_ρ(・, C) ≥ 0` である。

<1>2. `Retain` の鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>0, <1>1, <ref id=abab95b/>
  `L29` (e) より鎖の第 1 から第 `i-1` 節点は各スロットの `μ` を上げるだけなので、<1>0 より
  `held_{ρ'}(・, C)` は群の入口の値以上である。<1>1 よりその値は 0 以上である。

<1>3. `Release` の鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>0, <1>1, <ref id=1ab62dc/>, <ref id=abab95b/>
  `L29` (e) より鎖の第 `i` から第 `n` 節点は各スロットの `μ` を下げるだけなので、<1>0 より第 `i`
  節点の入口の `held_{ρ'}(・, C)` は鎖の直後の点の値以上である。`L26` (b) より鎖の群の次には必ず
  別の群があるので、鎖の直後の点は群の入口であり、<1>1 よりそこで `held ≥ 0` である。

<1>4. D7 の読む構文が `C` のスロットを読む時点では `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>1, <ref id=1ab62dc/>, <ref id=abab95b/>, <ref id=56c2068/>, 仮定
  D7 の読む構文の表の 6 行はいずれも `Let`・`Destructure`・`Eval` の節点であり、`L26` (b) よりそれらは
  すべて写しであって、その入口は群の入口である。<1>1 よりその入口の `held` は `B` の対応する節点の
  入口のものに等しく、`L29` (a)(c) より読まれるスロットも同じなので、仮定 ((ii-a) の後半を `B` に
  当てる) より `held ≥ 1` である。

<1>5. 鎖の群の第 `i` 節点 (`1 ≤ i ≤ n`) が `Retain(v, u_i)` で `C` のスロットを触れる時点では
      `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>0, <ref id=38dde17/>, <ref id=8412761/>, <ref id=abab95b/>, 仮定
  A10 より `ty(v)` は `L27` の前提を満たす。`L27` (a) より `u_i` は `π` で始まるので、`u_i` の下の
  inhabited な leaf は `π` の下の inhabited な
  leaf である。よって `B` の `t = Retain(v, π)` も `C` のスロットを触れ、仮定 ((ii-a) の後半を `B` に
  当てる) より `t` の入口で `held_ρ(・, C) ≥ 1` である。`L29` (d) よりこの群の入口の
  `held_{ρ'}(・, C)` はその値に等しい。`L29` (e) より鎖の第 1 から第 `i-1` 節点は各スロットの `μ` を
  上げるだけなので、<1>0 より第 `i` 節点の入口の `held_{ρ'}(・, C)` は群の入口の値以上である。

<1>6. 鎖の群の第 `i` 節点 (`1 ≤ i ≤ n`) が `Release(v, u_i)` で `C` のスロットを触れる時点では
      `held_{ρ'}(・, C) ≥ 1` である。
  BY <1>0, <1>1, <ref id=1ab62dc/>, <ref id=abab95b/>
  第 `j` 節点が処分する `C` のスロットの個数を `d_j` とすると、`L29` (e) と <1>0 より第 `i` 節点の
  入口の `held_{ρ'}(・, C)` は鎖の直後の点の値に `Σ_{j ≥ i} d_j` を足したものである。`L26` (b) より
  鎖の直後の点は群の入口であり、<1>1 よりそこで `held ≥ 0` である。この節点が `C` のスロットを
  触れるので `d_i ≥ 1` である。よって `held ≥ d_i ≥ 1`。

<1>6a. 関数本体・初期化子の終端の `Ret` の消費 (D9) を行った直後の点で `held_{ρ'}(・, C) ≥ 0` である。
  BY <1>1, <ref id=1ab62dc/>, <ref id=abab95b/>, <ref id=9d74736/>, <ref id=66c9670/>, 仮定
  `L26` (b) より `B'` の終端の節点は `Ret` の写しであり、その入口は群の入口なので <1>1 より
  そこで `held_{ρ'}(・, C) = held_ρ(・, C)` である。D9 の消費の表の「本体の終端の `Ret(x)`」の行が
  挙げる leaf は 2 つの側で同じである -- `L26` (b) より 2 つの `Ret` は同じ変数を名指し、`L29` (c) より
  分割対応の活性化は対応する各写しの節点で同じ値を割り当てるので、D16 の inhabited の判定も一致する。
  よって消費が `held` から引く量も等しく、消費の直後の `held` も 2 つの側で等しい。仮定 ((ii-a) の
  「非負であることは、終端の `Ret` の消費を行った直後の時点についても言う」を `B` に当てる) より
  `held_ρ(・, C) ≥ 0` なので、`held_{ρ'}(・, C) ≥ 0` である。

<1>7. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>6a, <ref id=1ab62dc/>
  `L26` (b) より `ρ'` の上の時点は群の入口か、鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) かの
  いずれかである。(ii-a) の前半は、群の入口では <1>1 が、鎖の途中では <1>2 と <1>3 が与える
  (`L26` (b) の 2 行より、鎖は `Retain` の鎖か `Release` の鎖かのいずれかである)。(ii-a) が
  節点の入口の外に置く 1 点 -- 終端の `Ret` の消費を行った直後 -- は <1>6a が与える。(ii-a) の後半が
  当たるのは D7 の読む構文と `Retain`/`Release` の節点であり、前者は <1>4 が、後者は `L26` (b) より
  すべて鎖の節点なので <1>5 と <1>6 が与える。

### 13.7a `L31a` (A19 (ii-c) の保存) <!--#934a2b0-->

**言明**。`P` が A19 (ii-a) と (ii-c) を満たすならば、`P' = split_rc_units(P)` も (ii-c) を満たす。
すなわち `P'` の各本体、各実行路 `ρ'`、`ρ'` を辿る各活性化について、各節点の実行の途中の各点
(D24 の段内の点) と、その点で `held_{ρ'}` が定まる各計数下の別名類 `C` について
`held_{ρ'}(・, C) ≥ 0` である。

**証明**

`B'` の 1 つの実行路 `ρ'` と、それを辿る 1 つの活性化を固定し、`L29` (b)(c) が与える `B` の実行路 `ρ`
と分割対応の活性化を取る。計数下の別名類 `C` を固定する。`L29` (c) より `C` は 2 つの側で同じ類で
ある。`L26` (b) より `B'` の各節点は写しか鎖の節点かのいずれかである。

<1>1. `B'` の写しの節点 `n'` の実行の途中の各点で `held_{ρ'}(・, C) ≥ 0` である。
  BY <ref id=1ab62dc/>, <ref id=abab95b/>, <ref id=3f1bb47/>, <ref id=c232680/>, <ref id=e3436e8/>, <ref id=9d5d254/>, 仮定
  `L26` (b) より `n'` は `B` のある節点 `n` の写しであり、2 つは同じ種類・同じ変数・同じ右辺を持つ。
  `L29` (c) より分割対応の活性化は対応する各写しの節点で同じ値を割り当て、D21 の 4 種の各位置での
  結果も等しい。A4 より、ある段が記憶域へ書き込む内容と、その段が作る子の活性化の個数・順序・
  それぞれに渡す値は、その段の節点とオペランドの値と D21 の 4 種の結果だけで決まるので、
  `n` と `n'` の段の素動作の列は 2 つの側で同じであり、その切れ目 -- 段内の点 (D24) -- も対応する。
  D34 は表の各行の事象を**それを運ぶ素動作の直後の段内の点**に置き、開始値も素動作の直後の点に
  置くので、`held` の動きはその素動作の列で決まる。`n'` の入口は群の入口なので `L29` (d) より
  そこで `held` は 2 つの側で等しく、以後も対応する各点で等しい。仮定 ((ii-c) を `B` に当てる) より
  `B` の側は 0 以上なので、`B'` の側も 0 以上である。

<1>2. `B'` の `Retain(v, u_i)` の節点の実行の途中の各点で `held_{ρ'}(・, C) ≥ 0` である。
  BY <ref id=1ab62dc/>, <ref id=f970a93/>, <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=b3dfa37/>, 仮定
  D34 の表のうち `Retain` 節点の実行が行う事象は第 4 行 (`+1`) だけである -- D2 より `Retain` は
  変数を束縛しないので D10 の生成の表のどの行にも当たらず、開始値の 3 行にも当たらない。D9 の
  消費でも移動でもない。よって `held_{ρ'}(・, C)` はこの節点の中で減らない。この節点の入口は
  `ρ'` の上の時点なので、`L31` (この命題の仮定の下で `P'` が (ii-a) を満たすこと) よりそこで
  `held_{ρ'}(・, C) ≥ 0` であり、途中の各点でもそれ以上である。
  **この節点の中で `held_{ρ'}(・, C)` が定まり始めることはない** -- 開始値を置く 3 行はどれも
  この節点の事象ではない。

<1>3. `B'` の `Release(v, u_i)` の節点の実行の途中の各点で `held_{ρ'}(・, C) ≥ 0` である。
  BY <ref id=1ab62dc/>, <ref id=f970a93/>, <ref id=9d5d254/>, <ref id=f06144e/>, <ref id=e3436e8/>, <ref id=b3dfa37/>, 仮定
  D34 の表のうち `Release` 節点の実行が行う事象は第 5 行 (`-1`) だけである -- D2 より `Release` は
  変数を束縛しないので D10 の生成の表のどの行にも当たらず、開始値の 3 行にも当たらない。D9 の消費でも
  移動でもない。よって
  `held_{ρ'}(・, C)` はこの節点の中で増えない。D24 より段と段のあいだの時点はその段の最後の段内の
  点であり次の段の最初の段内の点なので、この節点の出口は次の節点の入口、すなわち `ρ'` の上の時点で
  ある -- `L26` (b) より鎖の群の次には必ず別の群があり、鎖の中では次の節点が続くので、この節点の
  後には必ず別の節点がある。`L31` よりその時点で `held_{ρ'}(・, C) ≥ 0` であり、この節点の中の
  各点ではそれ以上である。

<1>4. QED
  BY <1>1, <1>2, <1>3, <ref id=1ab62dc/>
  `L26` (b) より `B'` の各節点は写しか鎖の節点かであり、鎖は `Retain` の鎖か `Release` の鎖かの
  いずれかである。3 つの場合を <1>1・<1>2・<1>3 が覆う。

### 13.8 `L32` (A19 (ii-b)) <!--#783a939-->

**言明**。`P' = split_rc_units(P)` の各本体は第 11 節の前提 (S) を満たす。したがって `L25` より
`P'` は A19 (ii-b) を満たす。

**証明**

`B` を `P` の 1 つの本体、`B' = split_body(B)` とする。`B'` の 1 つの実行路 `ρ'` と、それを辿る
1 つの活性化を固定し、`L29` (b)(c) が与える `B` の実行路 `ρ` と分割対応の活性化を取る。`L29` (c) より
別名類・`Anc`・`Ids`・`nm`・inhabited の判定は 2 つの側で一致する。

<1>0. (S0)。
  BY <ref id=1ab62dc/>, <ref id=abab95b/>, <ref id=0d8ae2f/>
  `L26` (b) より `ρ'` の上の各時点は、群の入口か、鎖の群の第 `i` 節点の入口 (`2 ≤ i ≤ n`) かの
  いずれかである。群の入口では `L29` (d) より各スロットの `μ` は `B` の対応する節点の入口のものに
  等しく、`L20a` の (S0) を `B` に当てるとそれは 0 以上である。鎖の途中については、`L29` (e) より
  `Retain` の鎖は `μ` を下げないので群の入口の値以上であり、`Release` の鎖は `μ` を上げないので鎖の
  直後の点の値以上である。`L26` (b) より鎖の群の次には必ず別の群があるので、鎖の直後の点は群の入口で
  あり、そこで `μ ≥ 0` である。

<1>1. (S1)。
  BY <ref id=1ab62dc/>, <ref id=dca1c02/>
  `L26` (a) より `split_rc_units` は `borrowed_units` を変えない。`L15` (e) より `P` のすべての関数の
  `borrowed_units` は空である。

<1>2. (S2)。
  BY <ref id=1ab62dc/>, <ref id=38dde17/>, <ref id=8412761/>, <ref id=abab95b/>, <ref id=0d8ae2f/>
  `L26` (b) より `B'` の各 `Retain` 節点は `B` のある `Retain(v, π)` 節点 `t` の鎖の第 `i` 節点
  `Retain(v, u_i)` である。それが触れるスロットは `u_i` の下の inhabited な leaf であり、A10 より
  `ty(v)` は `L27` の前提を満たすので、`L27` (a) より
  `u_i` は `π` で始まるので、それらは `π` の下の inhabited な leaf、すなわち `t` が触れるスロットで
  ある。`L20a` の (S2) を `B` に当てると `t` の入口でそれらの `μ` は 1 以上であり、`L29` (d) より
  この鎖の群の入口でも 1 以上である。`L29` (e) より第 1 から第 `i-1` 節点は `Retain` なので `μ` を
  下げない。よって第 `i` 節点の入口でも `μ ≥ 1` である。

<1>3. (S3)。
  BY <1>1, <ref id=dca1c02/>, <ref id=1ab62dc/>, <ref id=abab95b/>, <ref id=371ccc9/>, DEF 開始事象, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=9d5d254/>
  `DEF 開始事象` は開始事象を D34 の 3 行 -- `C` の終端が D10 の生成で作られる行、`C` の終端が
  所有するパラメータ・capture の leaf である行、`C` の終端が借用するパラメータ・capture の leaf で
  ある行 -- に当たる事象と定める。**第 3 の行はこの段の主語には当たらない** -- <1>1 の (S1) より
  `P'` のすべての関数の `borrowed_units` は空であり、D14 より借用する unit が 1 つも無い
  (`L15` (e) と `L26` (a) がそれを与える)。残る 2 行のうち第 2 の行は D10 の初期値の行であり、
  第 1 の行は D10 の生成の表の各行である。
  D10 の生成の表に `Retain`/`Release` の行は無いので、それらは写しの節点と本体の入口にだけ付く。
  `L26` (b) と `L29` (c) より写しと別名類は 2 つの側で対応するので、`ρ'` の上の開始事象は
  `ρ` の上の開始事象と 1 対 1 に対応する。`L20` より後者は各計数下の別名類について高々 1 つである。

<1>3a. (S4)。
  BY <ref id=1ab62dc/>, <ref id=38dde17/>, <ref id=8412761/>, <ref id=abab95b/>, <ref id=0d8ae2f/>, <ref id=b3dfa37/>, DEF 割り当て `μ`
  `L26` (b) より `B'` の各節点は、写しか、`B` のある `Retain`/`Release` 節点 `t` の鎖の 1 つの節点で
  ある。
  **写しの節点**については、`L26` (b) より `B` の対応する節点と同じ種類・同じ変数・同じ右辺を持ち、
  `L29` (c) より分割対応の活性化はそこで同じ値を割り当てるので、`DEF 割り当て μ` の 6 種のうち
  その節点が行う事象は 2 つの側で同じ leaf を同じ向きに動かす。よって `k_{n'}(s) = k_n(s)` である。
  写しの入口は群の入口なので `L29` (d) より `μ` も等しく、`L20a` の (S4) を `B` に当てると
  `μ(s) ≥ k_n(s)` である。
  **鎖の第 `i` 節点** `Retain(v, u_i)` は `μ` を下げないので `k = 0` であり、不等式は <1>0 の
  `μ ≥ 0` から出る。`Release(v, u_i)` は `u_i` の下の inhabited な各 leaf `(v, λ)` の `μ` を 1 回
  ずつ下げるので `k(v, λ) = 1`、他のスロットについて 0 である。A10 より `ty(v)` は `L27` の前提を
  満たし、`L27` (a) より `u_i` は `π` で始まるので、その leaf は `B` の `t = Release(v, π)` が
  触れる leaf であり、`t` は各 leaf を 1 回ずつ下げるので `k_t(v, λ) = 1` である。`L20a` の (S4) を
  `B` に当てると `t` の入口で `μ(v, λ) ≥ 1` であり、`L29` (d) よりこの鎖の群の入口でも 1 以上で
  ある。`L27` (a) より `u_1, …, u_n` の下の leaf の集合は互いに素なので、`L29` (e) より第 1 から
  第 `i-1` 節点は `u_i` の下の leaf の `μ` を動かさない。よって第 `i` 節点の入口でも
  `μ(v, λ) ≥ 1 = k(v, λ)` である。

<1>4. QED
  BY <1>0, <1>1, <1>2, <1>3, <1>3a, <ref id=e1e0e4a/>

### 13.9 3 つの節がそろうこと

A19 は「**`borrow_ify` の入力の各本体と、`borrow_ify` がそれを写した各本体 (すなわち `cancel` の
入力) の両方**について、**(ii-a)・(ii-b)・(ii-c) を仮定する。**」と範囲と項を定める。
`split_rc_units` の出力 --
`borrow_ify` の入力 -- について、その 3 つの節はどれも成り立つ。

- **(ii-a)** は `L31` である。保存の形で通り、`insert_rc` の出力についてそれが成り立つこと
  (`L19` (a)-(c)) を入力とする。(ii-a) が節点の入口の外に置く 1 点 -- 終端の `Ret` の消費を行った
  直後 -- も `L31` の量化に入り、`insert_rc` の出力についてその点を等式で与えるのは `L19` (c) で
  ある。
- **(ii-b)** は `L25` を `L32` の (S) の上に当てたものである。保存の形は取らない (下記)。
- **(ii-c)** は `L31a` である。これも保存の形で通り、`insert_rc` の出力についてそれが成り立つこと
  (`L19` (d)) を入力とする。
- **(i)** は仮定ではなく D21 が活性化に課す制限であり、実行が作る活性化がそれを満たすことは
  P28 (b) が示す。**P28 が言明を限る 2 つの前提 -- D12 の意味で RC 規律を満たすことと、借用する
  unit を持つ本体の活性化を作る段が (E3) に限られること -- がこの出力について満たされることは、
  第 13.6 節が述べる。**

**(ii-b) が保存の形を取らない理由。** 粗い 1 つの
`Retain(v, π)` が積む要素は `acted_references(v, π)` を `outstanding` に持ち、それは鎖の各節点が積む
要素の `outstanding` の和である -- D15 より `acted_references(v, π)` は `π` の下の各 boxed leaf を
数え、`L27` (a) より `π` の下の boxed leaf は `units_under(ty(v), π)` の各 unit の下の boxed leaf の
直和なので、鎖の各節点の `acted_references(v, u_i)` を足したものが粗い側のそれに等しい
(`CODE src/rc_ir/ownership.rs: acted_references`)。`consume_objects` は `outstanding` が名指された
オブジェクトを含む要素を
落とすので (`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_objects`)、粗い側の 1 要素が落ちる場面で
細かい側は名指されたオブジェクトを持つ要素だけが落ち、残りは pending に留まる。よって割った後の
`bumps` が割る前より大きい時点がありうる。そのかわり出力そのものが第 11 節の前提 (S) を満たすので、
`L25` を出力に当てて閉じる。

併せて、`L28` (b) より出力の各 `Retain`/`Release` 節点の path はその変数の型の unit である。これが
A2 が `split_rc_units` について述べていることである。
