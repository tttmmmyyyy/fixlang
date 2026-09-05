# P5, P6, P7 -- identity とオブジェクト、`acted_references`、消費の網羅性

この文書は `README.md` の P5 (a)、P5 (b)、P5 (c)、P6、P7 を証明する。主語となる語彙は D13 が定める --
`origin` の返り値と、その 2 つの形 `Exactly(u, σ)` と `Join { identity, candidates }` である。段が `BY` で
引くのは、`README.md` の定義と仮定、および命題 P1、P2、P29 の**言明**である。
P1 と P2 の証明は `p10-leaves-and-units.md`、P29 の証明は `p51-runs.md` にあり、この文書はその 3 つの
言明だけを使う。

D17 と A5 は本文が引き合いに出す。D17 は第 2 節の補足が leaf の写り方を突き合わせる
相手であり、A5 は P6 の補足 3 -- 同じ `id` を持つ 2 つの leaf がどちらも `Linhc(v, π, p)` に入るときは
参照も 2 つある -- の根拠である。この 2 つを `BY` に挙げる段は無い。

A16 が要るのは L1b、L1 の E3 の場合、L4、P5 (a)、P6 (b)、R1 である。読むのは A16 の 2 つの節 --
`Match` のアームが scrutinee のタグを尽くすことと、catch-all アームが `arms` の最後にあること --
であり、その 2 つを直に読むのは L1b と R1 である。

読んだコードはコミット `160cb59b928673a3613841067ad1f3872e8c1a10` の版である。

## 0. 結論

| 命題 | 結果 |
|---|---|
| P5 (a) 対の健全性 | 証明した (A16 の下で) |
| P5 (b) 対の有効性 | 証明した |
| P5 (c) 被覆 | 証明した |
| P6 (`acted_references` は静的な上位近似である) | 証明した (A16 の下で) |
| P7 (消費の網羅性) | 証明した (前半は D14 の所有をちょうど報告する `own` について、後半はどの `own` についても) |
| L6 (報告しない箇所は D9 の消費ではない) | 証明した (P7 に添える命題。D14 の所有をちょうど報告する `own` について) |

**オブジェクトの同一性を運ぶのは参照ではなく値である。** D9 の移動の表は参照の持ち手が変わる構文を挙げる
が、グローバル状態のオブジェクト (D26) を指す leaf は D8 の意味の参照を持たないので、「同じ参照を持つ
2 つの leaf は同じオブジェクトを指す」の形の議論は両端が計数下であるときにしか通らない (A5)。L1 は代わりに D9 の
**値の水準の 6 行**を使い、辺の両端が同じ値の同じ位置であることからオブジェクトの一致を出す。この道筋は
計数下かグローバル状態かを問わない。

P5 (a) の要は L4 である。**L4 が渡るのは、解析が `origin` を呼ぶ鍵に限る** -- 帰納が立つ整礎性
(L0a (b)) は呼ばれた鍵の上でしか言えないからである。第 9 節にその選択の理由を書く。**位置 (D6) の path は boxed leaf で
あり、`origin` の再帰は boxed leaf の path から出ると boxed leaf の path しか訪れない。** `origin_from_leaves_under` が `truncate_to_unit` で path を
unit へ切り詰める枝は、leaf でない path でだけ通る。1 つの unit の下に複数の leaf を持つ値
(`Std::Option (a, b)` の payload) が 2 つのオブジェクトに届いても、その 2 つの leaf の `identity` が
1 つに潰れることは無い。第 9 節に、この結論がどの仮定に載っているかを書く。

P6 (b) の要は、`identity` が付ける名前とオブジェクトの間の写像 `ν` である。L4 の (ii) が
`ν(id(v, λ)) = obj(v, λ)` を与え、これ 1 つから、名前づけが 2 つのオブジェクトを 1 つに潰さないこと
(= P5 (a)) と、P6 (b) の等号の両方が出る。逆向き -- 1 つのオブジェクトが 2 つの名前を持つこと -- は
実際に起きる (R1) が、P6 (b) の等号を壊さない。P6 の節がこれを述べる。

## 1. 記法と前提

1 つの本体を固定し、その本体から作られた `VarTable` の値を 1 つ、プログラムの `TypeEnv` の値を 1 つ
固定して、それぞれ `vars`、`type_env` と書く。**固定する本体は、`borrow_ify` の入力プログラムの、
ある関数の `body` か、あるグローバル初期化子の `init` かのどちらかである** (D23)。前者の `vars` を作るのは
`VarTable::of`、後者の `vars` を作るのは `VarTable::body_only` である
(`CODE src/rc_ir/ownership.rs: VarTable::of`, `VarTable::body_only`)。以下では、この 2 つを固定した
うえで `origin` と `acted_references` の第 1・第 2 引数を落として書く。**`bindings` が等しい相異なる
2 つの `VarTable` の値について答えが等しいことは、この記法の主張ではない** (P2a)。

**A1・A6・A9・A11 は主語を `borrow_ify` の入力に置く仮定なので、この固定の下でそのまま当たる。** 層 1 の
命題である P5・P6・P7 を `borrow_ify` の出力について読む者がどう読むかは `README.md` の A6 の項が
定める --「層 1 の証明は依存の順で P9 を引けないので、A6 を読む段は入力について読み、出力について読む者は
P9 と合わせて読む」。A11 の項も同じ形を取り、「A6 と同じ形であり、出力について読む段は P9 と合わせて
読む。」と述べる。A9 についてはその項が「`borrow_ify` と `cancel` は
アームを持たない `Match` を作らないので (P22、P24)、`cancel` の入力と出力についても同じことが言える」と
述べる。

- `origin(x, π)` は `origin(vars, type_env, x, π)`。**解析**とは、この文書が固定した `vars` と
  `type_env` を引数として `origin` を呼ぶ計算の全体である -- `borrow_ify` の `infer_ownership` と
  `RewriteCtx`、`cancel` の `CancelAnalysis` がそれであり、`README.md` の P3・P4 が「解析」と呼ぶのも
  これである。**この記法が指すのは、解析が鍵 `(x, π)` で `origin` を
  呼ぶときの、その呼び出しが返す値である。** 値が呼び出しに依らず 1 つであることは L0 (a) が示す。
  **解析が呼ばない鍵についてこの記法は何も指さないので、この文書の言明は、その鍵で `origin` が呼ばれる
  ことを前提に置く。** README の P3・P4・P5 (a)・P5 (b) は同じ前件を持ち、P6 は「**その各 leaf について
  解析が `origin` を呼ぶ**」を本文で述べる。
- **この文書の「解析」は、枠の P5 が置く述語より広い。** P5 は「**解析が `origin` を呼ぶ鍵とは、
  `src/rc_ir/borrow.rs` の `origin(` の呼び出しがその実行で渡す `(変数, path)` の対の全体である**」と、
  呼び出し元の在りかで絞る。この文書の意味の鍵はそれに加えて、`origin_inner` の再帰と
  `ownership.rs` の `acted_references` が渡す対を含む。枠の意味の鍵はこの文書の意味の鍵でもあるので、
  前件を満たす鍵すべてについて述べるこの文書の言明は、枠の意味の鍵にも当たる。
- `id(x, π)` は `origin(x, π).identity()`、`cand(x, π)` は `origin(x, π).candidates()` を集合とみなした
  もの、`act(x, π)` は `origin(x, π).acted_on()` を集合とみなしたもの
  (`CODE src/rc_ir/ownership.rs: Origin::identity`, `Origin::candidates`, `Origin::acted_on`)。
- `ty(x)` は `x` が得る値の型である。A12 より同じ名前の `RcVar` が持つ型は一致するので、これは `x` の
  出現によらない。

**DEF `L`**
`L(v, π)` は `boxed_leaf_paths(ty(v), type_env)` の要素のうち `π` を前置に持つものの集合である。
inhabited (D16) でない元を含む。この集合を `v` の `π` の下の boxed leaf の全体として読む段は、D4 を
併せて `BY` に挙げる。

**DEF `Linh`**
実行路の位置 `p` (D23) において、`L(v, π)` の元のうち `v` の値で inhabited (D16) なものの集合を
`Linh(v, π, p)` と書く。

`ActRefs(v, π)` は D15 の `acted_references(v, π)` である。`VarPath` は対 `(FullName, FieldPath)` である
(`CODE src/rc_ir/ast.rs: VarPath`)。等号はこの対の等号である。

この文書は命題を `L0`、`L0a`、`L0b`、`L1b`、`L1`、`L1a`、`L2`、`L3`、`L3a`、`L4`、`L5a`、`L5`、`L6`
(この順に並べる)、反例を `R1` と呼ぶ。**`BY` の行で
引用してよいのは、それぞれの言明だけである。** 言明が複数の主張からなる命題は主張に (a)、(b)、… の名札を
付け、`L5 (c)` のように引用する。同じ規則を P6 (a)、P6 (b)、P7 (a)、P7 (b) にも使う -- これらは
この文書が P6 と P7 を分けた主張であり、引用してよいのはその言明である。

### 外部の結果

この文書が Rust の言語仕様と実行の意味論から引く結果に名札を付ける。`BY` の行では `EXT <名前>` として
挙げる。

- **EXT 呼び出しの入れ子**。1 つのスレッドの上では、呼び出された関数は呼び出し元へ返るまでの間だけ走り、
  その間 呼び出し元は走らない。したがって 1 つのスレッドの上の 2 つの関数呼び出しの実行区間は、互いに
  素であるか、一方が他方に含まれるかのどちらかである。
- **EXT `Send` と `Sync`**。Rust では `&T` が `Send` であることと `T` が `Sync` であることは同値であり、
  `RefCell<T>` は `Sync` を実装しない。**`Sync` は auto trait であり、明示の `unsafe impl` を持たない
  構造体は、その欄の型がすべて `Sync` であるときに限って `Sync` を実装する。** auto trait の明示の実装は
  `unsafe impl` の形でしか書けず、それを書けるのはその型を定義するクレートの中である。`Send` でない値は、
  スレッドをまたいで渡せない。
- **EXT 名前による数え上げ**。Rust では、欄を名指す式はその欄の名前を含み、関数を呼ぶ式はその関数の名前を
  含む。`use` が別名を導入する場合はその別名を含む。したがって、その項目が可視である範囲のソースについて
  その名前と、`use` が導入した別名とを検索すれば、その項目を名指す式の全体が得られる。
- **EXT 借用規則**。Rust の借用規則は、値を move する時点にその値への参照が生きていることを許さない。
  また、共有参照 `&T` を通じて `T` の値やその欄に代入することはできない -- 共有参照から書き換えられるのは
  内部可変性を持つ欄だけである。
- **EXT 可視性**。Rust では、`pub` の付かない欄を名指す式は、その欄を宣言するモジュールとその子孫の
  中にしか書けない。同じく、`pub(crate)` の項目を名指す式はそれを宣言するクレートの中にしか、可視性の
  指定を持たない項目を名指す式はそれを宣言するモジュールとその子孫の中にしか書けない。
- **EXT `derive` した `Clone`**。`#[derive(Clone)]` が生成する実装は、その型の各欄について
  `Clone::clone` を呼び、その結果から同じ形の値を組み立てるだけであり、ほかの関数を呼ばない。
- **EXT `derive` した `PartialEq`**。`#[derive(PartialEq)]` が生成する実装は、その型の各欄について
  `PartialEq::eq` を呼び、その連言を返すだけであり、ほかの関数を呼ばない。`#[derive(Eq)]` が生成する
  実装は本体を持たない。
- **EXT 値を落とす処理**。`Drop` を実装しない型の値が落ちるとき走るのは、その型の各欄の値を落とす
  処理だけである。
- **EXT `RefCell` の内部可変性**。`RefCell<T>` が包む値を変更するには、その `RefCell` を名指して
  `borrow_mut` を呼ぶか、`get_mut`・`replace`・`take` のように可変参照を要求する操作を呼ぶ必要がある。
  共有参照からの読み (`borrow`) はその値を変更しない。
- **EXT 有限の時間に有限の呼び出し**。プロセスがある時点までに実行した命令は有限個であり、1 つの
  関数呼び出しが始まるのも返るのも、そのうちの 1 つの命令の実行である。したがって、ある時点までに
  始まった呼び出しも、ある時点までに返った呼び出しも有限個である。
- **EXT 排中律**。任意の命題 `Q` について、`Q` が成り立つか、`Q` の否定が成り立つかのどちらかである。
  よって「`Q` が成り立つ場合」と「`Q` が成り立たない場合」の 2 つは、場合を尽くす。
- **EXT 等号の性質**。等号は反射的 (`a = a`)、対称的 (`a = b` ならば `b = a`)、推移的 (`a = b` かつ
  `b = c` ならば `a = c`) である。
- **EXT `Iterator::enumerate` と `Iterator::filter`**。`Iterator::enumerate` は、元の列の第 `n` 元に
  添字 `n` を対にした列を返す。添字は 0 から始まり、1 ずつ増える。`Iterator::filter` は、述語が真である
  元だけを元の順序のまま残す。
- **EXT `u32` の 10 進表記**。`u32::to_string` はその値を 10 進の数字だけからなる文字列として返し、
  `str::parse::<u32>` はその文字列を同じ値に戻す。

### 在りかの前提

**コードのどこに何が在るかの数え上げは、段の中で行わない。** 記号を名指す `CODE` の引用はその記号の
本体しか与えないので、「ほかの記号はそれをしない」の側はそこから出ない。以下を名前つきの前提として
置き、`BY` の行ではその名前で引く。**個数は書かない** -- 一覧が在れば個数は一覧の長さである。

**果たすのは走査である。** 在りかを走らせられる字面で書き、`dev-docs/proof/proof_links.py` がその字面を
走らせて下の一覧と突き合わせる。挙がった各項目が何であるかは `--` の後に書く。走査は字面の上位近似
なので、一覧には読みだけの項目も、別の識別子や散文の一部としてその字面を含む項目も入る。
`#[cfg(test)]` の下の項目は走査が除く。項目の名前は走査が呼ぶ名前である -- 自由関数がその直前の
`impl` の名前を冠して挙がる形を含む。

**前提 `origins` の欄を名指す在りか** --- `VarTable` の `origins` の欄を名指す式が在るのは、
`RefCell::default()` を置く `VarTable::empty` と、記録を読み書きする自由関数 `origin` の本体である。
その欄の宣言は `VarTable` に在る。

SCAN src/ `origins`
  = src/rc_ir/ownership.rs: VarTable -- 欄の宣言
  = src/rc_ir/ownership.rs: VarTable::empty -- `origins: RefCell::default()`
  = src/rc_ir/ownership.rs: Origin::origin -- 自由関数 `origin` の記録の読みと書き
  = src/rc_ir/ownership.rs: Origin::identity -- doc の散文
  = src/rc_ir/ownership.rs: origin_inner -- 別の識別子 `leaf_origins_at` と doc の散文
  = src/rc_ir/ownership.rs: origin_from_leaves_under -- 別の識別子 `leaf_origins_under` と `//` コメント
  = src/rc_ir/provenance.rs: LeafOrigins -- doc の散文
  = src/rc_ir/provenance.rs: sole_origin -- 局所変数 `origins`
  = src/rc_ir/provenance.rs: Provenance::arg_passthrough -- doc の散文
  = src/rc_ir/provenance.rs: Provenance::join -- doc の散文
  = src/rc_ir/provenance.rs: Provenance::leaf_origins_at -- 別の識別子 `leaf_origins_at` の宣言
  = src/rc_ir/provenance.rs: Provenance::leaf_origins_under -- 別の識別子 `leaf_origins_under` の宣言
  = src/rc_ir/provenance.rs: Provenance::compose -- 閉包の引数 `origins` と局所変数 `operand_origins`
  = src/rc_ir/provenance.rs: Provenance::fmt -- 局所束縛 `origins`
  = src/rc_ir/provenance.rs: Provenance::leaf_source_to_string -- 引数 `origins`
  = src/rc_ir/provenance.rs: Uniqueness::resolve_leaf -- 引数 `origins`
  = src/rc_ir/provenance.rs: resolve -- 閉包の引数 `origins`
  = src/rc_ir/provenance.rs: leaf_is_unique -- 局所変数 `origins`
  = src/rc_ir/provenance.rs: is_unique_result -- 別の識別子 `leaf_origins_at`
  = src/rc_ir/validate.rs: check_rhs -- ループ変数 `origins`

**前提 名前の複製と照合の実装の在りか** --- `src/ast/` の項目のうち字面 `origin` を含むのは、
deprecation の欄 `origin_namespace` を宣言する項目と、その欄を置く項目・読む項目である。
`src/ast/name.rs` のどの項目もこの字面を含まない。

SCAN src/ast/ `origin`
  = src/ast/deprecation.rs: DeprecationStatement -- 欄 `origin_namespace` の宣言と doc の散文
  = src/ast/program.rs: Program::add_deprecation -- `origin_namespace: NameSpace::local()`
  = src/ast/program.rs: Program::identify_deprecation_targets -- `stmt.origin_namespace` と doc の散文

**前提 書かれた `Drop` の在りか** --- `src/` の中に `Drop` の実装を書いた型は `Finally`・
`StopWatch`・`LspClient` である。

SCAN src/ `impl Drop`
  = src/misc.rs: Finally::defer -- `Finally` の `Drop`
  = src/tool/stopwatch.rs: StopWatch::end -- `StopWatch` の `Drop`
  = src/tests/test_lsp/lsp_client.rs: LspClient::finish -- `LspClient` の `Drop`

**前提 `unsafe impl` の在りか** --- `src/` に `unsafe impl` と書いた項目は無い。

SCAN src/ `unsafe impl`

**前提 消費の走査を呼ぶ在りか** --- `collect_consumes`・`infer_ownership`・`rhs_consumes` を呼ぶ式が
在る項目は次で尽きる。走査はそれぞれの宣言と、`collect_consumes` を部分文字列に持つ
`collect_consumes_go` も挙げる。

SCAN src/ `collect_consumes`
  = src/rc_ir/borrow.rs: OwnedLeaves::infer_ownership -- 自由関数 `infer_ownership` の中の呼び出し
  = src/rc_ir/ownership.rs: collect_consumes -- 宣言と `collect_consumes_go` の呼び出し
  = src/rc_ir/ownership.rs: collect_consumes_go -- 宣言と自身への再帰

SCAN src/ `infer_ownership`
  = src/rc_ir/borrow.rs: OwnedLeaves::infer_ownership -- 宣言
  = src/rc_ir/borrow.rs: borrow_ify -- 自分の第 1 引数を渡す呼び出し

SCAN src/ `rhs_consumes`
  = src/rc_ir/borrow.rs: consume_rhs -- `CancelAnalysis::consume_rhs` の中の呼び出し
  = src/rc_ir/ownership.rs: collect_consumes_go -- `RcExpr::Let` の腕の呼び出し
  = src/rc_ir/ownership.rs: rhs_consumes -- 宣言

**この前提は枠の仮定に置くべきものである。** 枠へ移すときは `SCAN` の走査ごと移し、引く段の `BY` を
枠の仮定の名前へ差し替える。

### A16 の 2 つの節

この文書が読む A16 の節を、引用の形で書き出す。

- **(網羅)** すべての `Match(s, arms)` について、`arms` が catch-all アーム (`tag` が `None`) を持つか、
  `s` の値が取りうる実行時のタグがいずれかのアームの `tag` である。
- **(位置)** catch-all アームは `arms` の最後にある。

(位置) を果たすのはコード生成である -- `Generator::eval_rc_match` は最後でない各アームについて
`arm.tag.expect(..)` を無条件に評価するので、最後でない位置に catch-all を持つプログラムはコード生成で
止まり、その本体の活性化は存在しない (A16)。`validate` の `check_rhs` も同じことを検査するが、そちらは
`develop_mode` の門を持つ。

読む者は L1b と R1 (`<1>1` が 2 つの節を直接引く) であり、L1b を通じて L1 の E3 の場合、L4、P5 (a)、
P6 (b) が載る。(網羅) が落ちると、
3 変位の union に対する `[tag 0, tag 1]` のような `Match` で、実行時のタグが 2 のときコード生成は
`tag = Some(1)` のアームへ入る。そのとき D9 の移動の表の「unbox union の変位アームの payload 束縛」の
行が名指す活性変位と、`origin` が辿る静的な変位番号が食い違う。

**DEF 路の位置**

1 つの活性化 (D21) -- 固定した本体の 1 回の計算 -- を固定し、それが辿る実行路 (D21) を `ρ` とする。

**`ρ` の位置は D6 が定める。** D6 は「実行路 `ρ` の上のスロット (`ρ` の位置) とは、`ρ` を辿るある時点で
スロットである対のことである」と述べ、同じ節で「記号の位置についても同じく、`ρ` を辿るある時点で記号の
位置であるものを `ρ` の記号の位置と呼ぶ」と述べる。スロットと記号の位置を合わせて**位置**と呼ぶのも
D6 である。以下、`ρ` の位置のうちスロットであるものを **`ρ` のスロット**と呼ぶ。

**この節が足すのは、名前が値を得る形の数え上げの語だけである。** 名前 `x` が `ρ` の上で**値を得る**とは、
次のいずれかをいう。

- **(S1)** `x` がこの本体のパラメータか capture である。
- **(S2)** `Let(x, rhs, k)` の形の節点、または `Destructure(c, fs, s, k)` で `(i, x) ∈ fs` である節点が、
  `ρ` の上にある。
- **(S3)** `Let(m, Match(s, arms), k)` の形の節点が `ρ` の上にあり、`ρ` を辿る実行がその `Match` で選ぶ
  アーム `a` の `payload` が `x` である。
- **(S4)** `x` が `vars.bindings` に無い名前であり、`ρ` の上のある節点に現れる。

`x` の値の位置 `λ` にあるオブジェクトを `obj(x, λ)` と書く (D6)。

**この 4 つの形が D6 の数える 3 つの形と同じものを数えること、`(x, λ)` が `ρ` の位置であることとの同値、
および `obj(x, λ)` が `ρ` の上の位置に依らないことは、定義ではなく L0b が示す。**

## 2. 別名の辺と、それが `ρ` の上で実行されること

D20 は、D9 の移動の表の 6 行をスロットの間の**別名の辺**と呼び、**別名の道**をその辺を向きを問わず辿った
道と定める。以下、その 6 行に名前を付ける。`λ` は leaf を渡る。

- **E1** (`Let(x, Var(y), k)`): `(y, λ)` と `(x, λ)`。
- **E2** (`Destructure(c, fs, s, k)` で `c` が unbox、`(i, f)` が `fs` の元): `(c, [i] ++ λ)` と `(f, λ)`。
- **E3** (`Let(m, Match(s, arms), k)` の unbox union の変位アームの payload 束縛、アームの `tag` が
  `Some(t)`、payload 変数が `p`): `(s, [t] ++ λ)` と `(p, λ)`。
- **E4** (catch-all アームの payload 束縛、payload 変数 `p`): `(s, λ)` と `(p, λ)`。
- **E5** (`Let(x, Llvm(gen, args), k)` の素通し leaf): 結果の leaf `λ` の `result_prov` の宣言が単一の
  `Arg(j, σ)` のとき、`(args[j], σ)` と `(x, λ)`。
- **E6** (`Match` のアーム本体の終端の `Ret(x)`): `Match` の束縛変数を `m` として、`(x, λ)` と `(m, λ)`。

D20 の 6 行との対応は、`Let(x, Var(y), k)` が E1、アーム本体の `Ret` が E6、unbox 容器の `Destructure` の
名前付きフィールドが E2、unbox union の変位アームの payload が E3、catch-all アームの payload が E4、
`Llvm` の素通し leaf が E5 である。

**DEF 辺の leaf 対応**
E1 から E6 の各行は、始点の位置の path と終点の位置の path を leaf `λ` の関数として書き下したもので
ある。その 2 つの path の対応を、その行の**辺の leaf 対応**と呼ぶ -- E1 は `λ` ↔ `λ`、E2 は
`[i] ++ λ` ↔ `λ`、E3 は `[t] ++ λ` ↔ `λ`、E4 は `λ` ↔ `λ`、E5 は `σ` ↔ `λ`、E6 は `λ` ↔ `λ` である。

**この対応が D9 の値の水準の 6 行の渡す位置の対応であることは、定義ではなく L1 の `<1>1a` が示す。**

**補足 (D17 の写り方との突き合わせ)**。D17 は `origin` の辿る辺について leaf の写り方を書く -- `Move`・
catch-all・`Join` は `λ` を変えず、`Destructure` のフィールドと変位アームの payload は先頭に添字を足し、
`Llvm` は宣言の `σ` へ置き換える。上の一覧はそれと一致する。この突き合わせを `BY` に挙げる段は無い
(§1)。

**DEF 辺の存在**
E1 から E6 の各行は、辺を定める節点の形と leaf `λ` の選び方から、2 つの対 `(u, α)`、`(w, β)` を作る。
**その 2 つがどちらも `ρ` の位置 (DEF 路の位置) であるとき、その節点と `λ` についてのその辺は `ρ` の上に
在るといい、一方でも `ρ` の位置でないときは無いという。** 以下、単に「辺が在る」と書くのはこの意味で
ある。

**この語と DEF `ρ` の上で実行された辺 とを併せたものが D20 の意味の「辺が在る」と同じものであることは、
定義ではなく L1 (b) が示す。**

**補足 (辺の端は位置である)**。辺の端を位置に取るのは D20 の本文である -- 「**辺の端はスロットに限らず、
位置 (D6) である。** D6 は別名の道が記号の位置で終わることを述べるので、D9 の移動の表の辺は記号の位置へ
着きうる」。上の語はその読みをこの文書の語に写したものである。

**DEF `ρ` の上で実行された辺**
辺が **`ρ` の上で実行された**とは、次をいう。

- E1、E2、E5 の辺: その辺を定める節点 (`Let(x, Var(y), k)`、`Destructure(c, fs, s, k)`、
  `Let(x, Llvm(gen, args), k)`) が `ρ` の上にある。
- E3、E4 の辺: その辺を定める `Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、`ρ` を辿る実行が
  その `Match` で選ぶアーム (D21) が、その辺の属するアームである。
- E6 の辺: その辺を定めるアーム本体の終端の `Ret(x)` が `ρ` の上にある。

**補足 (端が `ρ` の位置にならない形)**。端が `ρ` の位置でない形は辺の両側にある。E2 の `[i] ++ λ` は、
`i` が穴のとき `ty(c)` の boxed leaf ではない (D4 の規則 5)。E3 の payload 変数の型が boxed leaf を
1 つも持たなければ、その節点はどの `λ` についても E3 の辺を持たない。

**補足 (始点が記号の位置である形)**。E1 の `y` が `vars.bindings` に束縛を持たない名前であるとき、
始点は記号の位置であり、これは `ρ` の位置である (D6 はスロットと記号の位置を合わせて位置と呼ぶ) --
すなわちこの形では辺は在る。この形のために、L2 は辺ではなく `vars.bindings` の記録について述べる --
辺が在ることを先に言わなくても `origin` の等式が使えるようにするためである。

## L0 (`origin` は `origin_inner` の値を返す) <!--#ecdd35d-->

**言明**。`vars` と `type_env` を固定すると、次の 3 つが成り立つ。**鍵**とは `origin` の第 3・第 4 引数の
対 `(x, π)` であり、鍵 `(x, π)` の **cold な呼び出し**とは、`origin(vars, type_env, x, π)` の呼び出しの
うち `origin_inner` を評価するものである。

- **(a)** 鍵 `(x, π)` について `origin` の呼び出しが在るならば、その鍵のどの呼び出しも同じ値を返し、
  その値は `origin_inner(vars, type_env, x, π)` の 1 回の呼び出しが返した値である。§1 の記法
  `origin(x, π)` はこの共通の値を指す。
- **(b)** 鍵 `(x, π)` について `origin` の呼び出しが 1 つでも在るならば、その鍵の cold な呼び出しは
  ちょうど 1 つ在る。
- **(c)** この `vars` と `type_env` を第 1・第 2 引数に取る `origin` の呼び出しはどれも停止する。

**(c) は P2 の言明をこの文書が固定した `vars` に当てたものである。** (b) の証明がそれを使い、L0a も
使う -- 呼び出しの入れ子から「先に返る」を出す段は、その呼び出しが返ることを要る。

**(a) は `README.md` の P2a より強い。** P2a は、A6 と A11 を満たすプログラムの本体について
`VarTable::of` か `VarTable::body_only` が作った 1 つの `VarTable` の値と 1 つの `TypeEnv` の値を
固定したうえで、鍵が等しい 2 つの `origin` の呼び出しがどちらも値を返すならばその 2 つの返り値が
等しいと述べる。
(a) は同じ固定の下で、呼び出しの在る鍵について、その値が `origin_inner` の**1 回の呼び出しが返した値**で
あることを足す。その分が要るのは L2 と L4 であり、そこは `origin` の値を `origin_inner` の腕が返す式として
読む。**(a) が呼び出しの存在を前件に取るのは、呼び出しの無い鍵について `origin_inner` の呼び出しも無い
からである。** (b) は L0a と L4 の帰納が要るものであり、(a) と同じ分析から出る。

<1>1. `origin` は、`vars.origins` に鍵 `(x, π)` の記録があればその値を返し、無ければ
      `grow_stack(|| origin_inner(vars, type_env, x, π))` の値を鍵 `(x, π)` で記録して返す。
  BY CODE src/rc_ir/ownership.rs: origin

<1>1a. (c) が成り立つ。すなわち P2 は、この `vars` と `type_env` を引数に取る `origin` のどの
      呼び出しについても停止を与える。
  P2 は、`x` がプログラムの束縛変数であるか `vars.bindings` に束縛を持たない名前であるような
  すべての `(x, π)` について、`π` を問わず `origin(vars, type_env, x, π)` が停止すると述べる。
  「プログラムの束縛変数」が何を指すかは P2 の項が定める -- 節点が束縛する変数と、その本体の
  パラメータ・capture の両方である。同じ項が、`VarTable::of` の作る表について
  「`vars.bindings` に記録を持つ名前はちょうどこの 2 種である」と述べる。`VarTable::body_only` が
  作る表は `collect_bindings` の記録だけを持つので、その名前もこの 2 種の一方である。記録を持たない
  名前は P2 の第 2 の節に当たる。
  BY <ref id=0edb0ba/>, CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only,
     CODE src/rc_ir/ownership.rs: collect_bindings

<1>2. `grow_stack(f)` の値は `f()` の値である。
  BY <ref id=3e6b0e0/>

<1>2a. `origin(vars, type_env, x, π)` の 1 回の呼び出しの中で `origin` の呼び出しが始まるのは、その
      呼び出しが `origin_inner` を評価する場合の、その評価の中だけである。
  <2>1. `origin` の本体が行う呼び出しは、鍵 `(var.clone(), path.to_vec())` を組み立てる
        `FullName::clone` と `<[usize]>::to_vec`、記録を検査する `RefCell::borrow` と `Map::get`、
        記録の値と答えを複製する `Origin::clone`、`grow_stack(|| origin_inner(..))`、および記録を
        書き込む `RefCell::borrow_mut` と `Map::insert` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin
  <2>2. `grow_stack(|| origin_inner(..))` の実行の中で始まる `origin` の呼び出しは、`origin_inner` の
        評価の中で始まる。
    `grow_stack` の本体が行う呼び出しは `stacker::maybe_grow(64 * 1024, 1024 * 1024, f)` 1 つであり、
    `stacker` は外部クレートなので `src/` の外に在り、EXT 可視性 より `pub(crate)` の `origin` を名指す
    式を持たない。A15 より `grow_stack(f)` は `f` をちょうど 1 回呼びその返り値を返し、ここでの `f` は
    閉包 `|| origin_inner(vars, type_env, var, path)` であって、その本体は `origin_inner` の呼び出し
    1 つである。
    BY <ref id=3e6b0e0/>, EXT 可視性, CODE src/misc.rs: grow_stack, CODE src/rc_ir/ownership.rs: origin
  <2>3. `<2>1` の受け手のうち `grow_stack` を除くものの実行の中で入る `src/` の項目は、`FullName` と
        `NameSpace` と `Origin` の `Clone` の実装、および `FullName` と `NameSpace` の
        `Hash`・`PartialEq`・`Eq` の実装で尽きる。
    `<[usize]>::to_vec` と `RefCell` の `borrow`・`borrow_mut` は標準ライブラリの項目、`Map` すなわち
    `FxHashMap` の `get`・`insert` は外部クレートの項目である。この `get` と `insert` は鍵の型
    `VarPath = (FullName, FieldPath)` の `Hash` と `Eq` を呼び、`insert` は置き換えた値を落とす。
    対の `Hash` と `Eq`、`FieldPath` すなわち `Vec<usize>` の `Hash` と `Eq`、および `String`・
    `Vec<String>`・`bool` の同じトレイトは標準ライブラリに在り、`Set<VarPath>` すなわち `FxHashSet` の
    `Clone` は外部クレートに在る。`Origin` は `#[derive(Clone)]` を持ち、その欄の型は `VarPath` と
    `Set<VarPath>` である。`FullName` は `Clone` と `PartialEq` と `Eq` を derive し、その欄の型は
    `NameSpace` と `String` である。`NameSpace` は `Clone` を derive し、`Hash` と `PartialEq` と `Eq` は
    `impl Hash for NameSpace`・`impl PartialEq for NameSpace`・`impl Eq for NameSpace` として
    `src/ast/name.rs` に書かれ、その欄の型は `Vec<String>` と `bool` である。`FullName` の `Hash` も
    `impl Hash for FullName` として同じファイルに書かれている。EXT `derive` した `Clone` と
    EXT `derive` した `PartialEq` より、derive した実装が
    呼ぶのは欄の同じトレイトのメソッドだけである。前提 書かれた `Drop` の在りか より `FullName`・
    `NameSpace`・`Origin` はどれも `Drop` を実装しないので、EXT 値を落とす処理 より、その値が落ちるとき
    走るのは欄の値を落とす処理だけである。
    BY 前提 書かれた `Drop` の在りか, EXT `derive` した `Clone`, EXT `derive` した `PartialEq`,
       EXT 値を落とす処理, CODE src/rc_ir/ownership.rs: Origin,
       CODE src/rc_ir/ast.rs: VarPath, FieldPath, CODE src/ast/name.rs: FullName, NameSpace,
       CODE src/ast/name.rs: impl Hash for FullName, CODE src/ast/name.rs: impl Hash for NameSpace,
       CODE src/ast/name.rs: impl PartialEq for NameSpace,
       CODE src/ast/name.rs: impl Eq for NameSpace,
       CODE src/misc.rs: Map, Set
  <2>4. `<2>3` の項目はどれも `origin` を名指す式を持たない。
    derive した実装は EXT `derive` した `Clone` と EXT `derive` した `PartialEq` より欄の同じトレイトの
    メソッドだけを呼ぶ。書かれた実装 -- `impl Hash for FullName`、`impl Hash for NameSpace`、
    `impl PartialEq for NameSpace`、`impl Eq for NameSpace` -- は `src/ast/name.rs` の項目であり、
    前提 名前の複製と照合の実装の在りか より、`src/ast/` の項目の
    うち字面 `origin` を含むのは `deprecation.rs` と `program.rs` の deprecation についての項目だけで
    あって、`name.rs` の項目はそこに無い。EXT 名前による数え上げ より、`origin` を名指す式はその名前を
    含む。
    BY 前提 名前の複製と照合の実装の在りか, EXT `derive` した `Clone`, EXT `derive` した `PartialEq`,
       EXT 名前による数え上げ, CODE src/ast/name.rs: impl Hash for FullName,
       CODE src/ast/name.rs: impl Hash for NameSpace,
       CODE src/ast/name.rs: impl PartialEq for NameSpace,
       CODE src/ast/name.rs: impl Eq for NameSpace
  <2>5. QED
    `origin` は `ownership.rs` の `pub(crate)` の関数なので、EXT 可視性 より、それを呼ぶ式はこの
    クレートのソース `src/` の中にしかない。`<2>1` が本体の行う呼び出しの全体を与え、`<2>2` が
    `grow_stack` の側を、`<2>3` と `<2>4` が残る側を与える。`src/` の外の項目は EXT 可視性 より
    `origin` を名指す式を持たない。よって `origin` の呼び出しが始まるのは `origin_inner` の評価の中
    だけである。
    BY EXT 可視性, <2>1, <2>2, <2>3, <2>4

<1>3. `origins` の欄を変更するのは `<1>1` の記録だけである。
  <2>1. `origins` は `VarTable` の非公開の欄である (`pub` が付かない)。EXT 可視性 より、この欄を
        名指す式は、それを宣言するモジュールとその子孫、すなわちこのクレートの中にしかなく、その
        ソースは `src/` である。
    BY EXT 可視性, CODE src/rc_ir/ownership.rs: VarTable
  <2>2. `src/` の中でこの欄を名指す式は、`VarTable::empty` の `origins: RefCell::default()`、
        `origin` の `vars.origins.borrow()`、`origin` の `vars.origins.borrow_mut()` である。
        前提 `origins` の欄を名指す在りか が、この欄を名指す式が在るのはその 2 つの項目であると述べ、
        走査が `src/` の全体について字面 `origins` を含む項目を挙げてそれを果たす。EXT 名前による
        数え上げ より、この欄を名指す式はその名前を含む。
    BY 前提 `origins` の欄を名指す在りか, EXT 名前による数え上げ,
       CODE src/rc_ir/ownership.rs: VarTable, CODE src/rc_ir/ownership.rs: VarTable::empty,
       CODE src/rc_ir/ownership.rs: origin
  <2>3. QED
    EXT `RefCell` の内部可変性 より、`RefCell<Map<..>>` が包む写像を変更するには、その `RefCell` を
    名指す式を通る必要がある。`<2>1` と `<2>2` よりその式は `<2>2` が挙げたものに限られ、`origin` の
    `vars.origins.borrow()` は読みなので変更しない。残るのは `VarTable::empty` の初期化 (空の写像) と
    `origin` の `vars.origins.borrow_mut()` に続く `insert` だけであり、後者が `<1>1` の記録である。
    BY EXT `RefCell` の内部可変性, <2>1, <2>2

<1>3a. 鍵 `(x, π)` について、cold な呼び出しは高々 1 つであり、`<1>1` の記録も高々 1 度しか書かれない。
  <2>1. 記録は取り除かれない。`<1>3` より `origins` を変更するのは `<1>1` の `insert` だけである。
    BY <1>3
  <2>1b. 鍵 `(x, π)` の cold な呼び出しの全体は、その鍵の記録を書く呼び出しの全体に等しい。
    `<1>1` より、記録を書くのは自分の検査で記録を見つけなかった呼び出し、すなわち `origin_inner` を
    評価する呼び出しであり、書き込みは `origin_inner` が返った後、その呼び出しが返る直前にある。
    `<1>1a` より cold な呼び出しは返るので、その書き込みに着く。
    BY <1>1, <1>1a
  <2>2. 1 つの `vars` を引数に取る 2 つの `origin` の呼び出しが別々のスレッドの上にあるならば、その
        2 つの実行区間は互いに素である。
    <3>1. 1 つの `VarTable` への参照を 2 つのスレッドが同時に持つことはない。`VarTable` は
          `origins: RefCell<Map<VarPath, Origin>>` の欄を持つ。EXT `Send` と `Sync` より、auto trait の
          明示の実装を書けるのはその型を定義するクレートの中に限られ、`VarTable` を定義するのは
          このコンパイラのクレートであって、そのソースは `src/` である。前提 `unsafe impl` の在りか
          より `src/` に `unsafe impl` と書いた項目は無いので、`VarTable` は `Sync` の `unsafe impl` を
          持たない。
          EXT `Send` と `Sync` より `RefCell<T>` は `Sync` を実装せず、`Sync` は auto trait なので
          その欄を持つ `VarTable` も `Sync` を実装しない。同じ結果より `&VarTable` は `Send` では
          ないので、スレッドをまたいで渡せない。
      BY 前提 `unsafe impl` の在りか, EXT `Send` と `Sync`, CODE src/rc_ir/ownership.rs: VarTable
    <3>2. `origin` は `vars: &VarTable` を引数に取り、その呼び出しの実行区間の間ずっとこの参照を保持
          する。
      BY CODE src/rc_ir/ownership.rs: origin
    <3>3. `VarTable` の値を別のスレッドへ move する時点では、`origin` の呼び出しは 1 つも走っていない。
          EXT 借用規則 より、値を move する時点にその値への参照が生きていることはなく、`<3>2` より
          走っている `origin` の呼び出しはその参照を保持している。
      BY EXT 借用規則, <3>2
    <3>4. QED
      `<3>1` より 2 つのスレッドが同時に `&VarTable` を持つことはないので、この `vars` を引数に取る
      呼び出しがスレッドをまたぐには、その `VarTable` の値が move されている。`<3>3` よりその時点に
      走っている呼び出しは無いので、move より前のスレッドの呼び出しはすべて move より前に終わり、
      move より後のスレッドの呼び出しはすべて move より後に始まる。
      BY <3>1, <3>3
  <2>3. 1 つの `vars` を引数に取る 2 つの `origin` の呼び出しの実行区間は、互いに素であるか、一方が
        他方に含まれるかのどちらかである。
        同じスレッドの上にある 2 つについては EXT 呼び出しの入れ子 がこれを与える。別々のスレッドの上に
        ある 2 つについては `<2>2` が互いに素であることを与える。
    BY EXT 呼び出しの入れ子, <2>2
  <2>4. 鍵 `(x, π)` の記録を書く呼び出しが 2 つあるとすると、一方 `C` は他方 `C'` の `origin_inner` の
        評価の中で始まり、どちらも自分の検査で記録を見つけていない。
    `<1>1` より、記録を書くのは自分の検査で記録を見つけなかった呼び出しであり、書き込みは `origin_inner`
    が返った後、その呼び出しが返る直前にある。2 つのうち先に終わる方を `C`、後に終わる方を `C'` とする。
    `C` の書き込みは `C` が終わる直前にあり、`<2>1` よりその記録は以後残るので、`C'` が記録を見つけて
    いない以上、`C'` の検査は `C` の終わりより前にある。`C'` の書き込みは `C` の終わりより後なので、
    2 つの実行区間は重なり、`<2>3` より `C` が `C'` に含まれる。`<1>2a` より、`C'` の中で始まる `origin` の
    呼び出しは `C'` の `origin_inner` の評価の中で始まるので、`C` はその中で始まる。
    BY <1>1, <1>2a, <2>1, <2>3
  <2>4a. 記録を見つけない 2 つの `origin` の呼び出しが同じ鍵を持ち、一方が他方の `origin_inner` の
         評価の中で始まるとき、この対を**対 (つい)** と呼ぶ。対 `(A, B)` -- `A` が外側 -- が在るならば、
         対 `(A', B')` であって `A'` が `A` の `origin_inner` の評価の中で始まるものが在る。
    <3>1. `origin_inner(vars, type_env, x, π)` が直接行う `origin` の呼び出しの**鍵の集合**は、
          `vars.bindings` の記録・`type_env`・`(x, π)` だけで決まり、`vars.origins` を読まない。
          `None | Param | Producer` の腕は呼び出しを行わず、`Move` の腕は 1 つの鍵を、`Join` の腕は
          `arm_results` の各元の鍵を、`Field` と `Payload` の腕は高々 1 つの鍵を、`Llvm` の腕は `decl` から
          決まる鍵を渡す。`Llvm` の腕の `decl` は
          `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の値であり、A3 より `result_prov` は
          決定的である -- 同じ引数に対して常に同じ値を返す -- ので、`decl` は
          `vars.bindings` が持つ `llvm_gen`・`args`・`result_ty` と `type_env` だけで決まる。
          `origin_from_leaves_under` も、`decl` から
          `operand_units` を先に集め終えてからその全部について `origin` を呼ぶ。どの腕も、行った
          `origin` の呼び出しの返り値で次の呼び出しの有無や鍵を変えない。
      BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_inner, origin_from_leaves_under
    <3>2. 対 `(A, B)` を取る。`D_0 := A` とし、`D_i ≠ B` である間、`D_{i+1}` を「`D_i` の `origin_inner` の
          評価の中で始まる `origin` の呼び出しのうち、`B` を実行区間に含む (または `B` 自身である) もので
          最も外側のもの」と定めると、`A = D_0 ⊋ D_1 ⊋ … ⊋ D_k = B` (`k ≥ 1`) が得られる。`D_0` から
          `D_{k-1}` はどれも記録を見つけず `origin_inner` を評価し、`D_{i+1}` は `D_i` の `origin_inner` が
          `<3>1` の意味で直接行う呼び出しである。
      `<2>3` より 2 つの呼び出しの実行区間は互いに素か一方が他方に含まれるかなので、`D_i` の中で始まり
      `B` を含む呼び出しどうしは入れ子に並び、最も外側のものが 1 つ定まる。`<1>2a` より、`D_i` の中で
      始まる `origin` の呼び出しは `D_i` の `origin_inner` の評価の中で始まるので、`D_{i+1}` はその中で
      始まる。列が有限で `B` に着いて止まるのは
      次による。各 `D_i` は `B` を実行区間に含む (または `B` 自身である) ので、`B` が始まる瞬間に走って
      いる呼び出しである。それらは `<2>3` より互いに素にならないので、`<2>2` より `B` と同じスレッドの
      上にあり、EXT 呼び出しの入れ子 より入れ子に並ぶ -- すなわち `B` が始まる瞬間の呼び出しスタックの
      元である。EXT 有限の時間に有限の呼び出し より、`B` が始まる時点までに始まった呼び出しは有限個で
      あり、そのスタックの元はそのうちのものなので、スタックの深さは有限である。`D_{i+1}` の実行区間は
      `D_i` の実行区間の真の部分なので
      `D_{i+1}` はそのスタックの上でより深い位置にある。よって列は有限で終わり、終わるのは
      `D_i` の中で始まり `B` を含む呼び出しが `B` 自身しか無いところ、すなわち `D_i = B` である。
      `<1>1` より、`origin_inner` を評価するのは記録を見つけなかった呼び出しだけである。
      BY EXT 呼び出しの入れ子, EXT 有限の時間に有限の呼び出し, <1>1, <1>2a, <2>2, <2>3, <3>1
    <3>3. `B` の `origin_inner` の評価は、`D_1` の鍵と同じ鍵の呼び出し `E` を直接行う。
      `B` は記録を見つけないので `origin_inner` を評価し、その鍵は `A` の鍵に等しい。`<3>1` より
      直接行う呼び出しの鍵の集合は鍵と `vars.bindings` と `type_env` だけで決まるので (`Llvm` の腕に
      ついては A3 の決定性がそれを与える)、`B` の `origin_inner` が直接行う呼び出しの鍵の集合は
      `A` の `origin_inner` のそれに等しい。`D_1` は `A` の `origin_inner` が直接行った呼び出しなので、
      その鍵はその集合に入っている。
      BY <ref id=e11772a/>, <3>1, <3>2
    <3>4. `E` は `D_1` の `origin_inner` の評価の中で始まる。
      `k = 1` のとき `B = D_1` であり、`E` は `B` の `origin_inner` の中で始まる。`k ≥ 2` のとき
      `B` は `D_1` の `origin_inner` の評価の中で始まるので、その中で始まる `E` も同じである。
      BY <3>2, <3>3
    <3>5. CASE `E` は記録を見つけない。`<3>3` より `E` の鍵は `D_1` の鍵に等しく、`<3>2` より `D_1` は
          記録を見つけないので、`<3>4` より `(D_1, E)` は対である。
      BY <3>2, <3>3, <3>4
    <3>6. CASE `E` は記録を見つける。この鍵の記録を書いた呼び出しを `F` とすると、`(D_1, F)` は対である。
      `<3>2` より `D_1` は自分の検査でこの鍵の記録を見つけていないので、`D_1` の検査の時点にその記録は
      無い。`<2>1` より記録は取り除かれないので、`F` の書き込みは `D_1` の検査より後にある。`<1>1` より
      書き込みは `F` が返る直前にあり、それは `E` の検査より前にある。`<3>4` より `E` は `D_1` の中に
      あるので、`F` が返るのは `D_1` が返るより前である。すなわち `F` が返る時点は `D_1` の実行区間の
      内側にあるので、2 つの実行区間は互いに素ではない。`<2>3` より一方が他方に含まれ、`D_1` が `F` に
      含まれるならば `F` は `D_1` より後に返るのでこれに反する。よって `F` は `D_1` に含まれ、`D_1` は
      まだ返っていないので `F ≠ D_1` である。
      `<1>2a` より、`D_1` の中で始まる `origin` の呼び出しは `D_1` の `origin_inner` の評価の中で始まるので、
      `F` はその中で始まる。`<1>1` より記録を書く `F` は記録を見つけなかった呼び出しである。
      BY <1>1, <1>2a, <2>1, <2>3, <3>2, <3>4
    <3>7. QED
      `<3>5` と `<3>6` は `E` が記録を見つけるか否かの 2 つの場合で尽きており、どちらも外側が `D_1` で
      ある対を与える。`<3>2` より `D_1` は `A` の `origin_inner` の評価の中で始まる。
      BY <3>2, <3>5, <3>6
  <2>4b. 対 `(A, B)` が在るならば、`A` は返らない。
    `<2>4a` を繰り返し当てると、`A = A_0` と、各 `A_{n+1}` が `A_n` の `origin_inner` の評価の中で
    始まるような呼び出しの無限列が得られる。`A_{n+1}` の実行区間は `A_n` の実行区間の真の部分なので、
    `A_1`、`A_2`、… は相異なる呼び出しであり、どれも `A` の中で始まる。EXT 有限の時間に有限の呼び出し
    より、`A` が返るならば `A` が返る時点までに始まった呼び出しは有限個であり、`A` の中で始まる
    呼び出しはそのうちのものである。よって `A` は返らない。
    BY EXT 有限の時間に有限の呼び出し, <2>4a
  <2>5. QED
    鍵 `(x, π)` の cold な呼び出しが 2 つあるとすると、`<2>1b` よりどちらも記録を書くので、`<2>4` より
    一方 `C` が他方 `C'` の `origin_inner` の評価の中で始まり、どちらも記録を見つけない同じ鍵の
    呼び出しである。すなわち `(C', C)` は `<2>4a` の意味の対であり、`<2>4b` より `C'` は返らない。
    これは `<1>1a` に反する。ゆえに cold な呼び出しは高々 1 つであり、`<2>1b` より記録を書く呼び出しも
    高々 1 つである。
    BY <1>1a, <2>1b, <2>4, <2>4a, <2>4b

<1>3b. 鍵 `(x, π)` について `origin` の呼び出しが 1 つでも在るならば、その鍵の cold な呼び出しは
       少なくとも 1 つ在る。
  その鍵の呼び出しのどれかが検査で記録を見つけなければ、`<1>1` よりその呼び出しは `origin_inner` を
  評価するので cold である。どれもが記録を見つけるとする。`VarTable::empty` は `origins` を
  `RefCell::default()`、すなわち空の写像で初期化し、`VarTable::of` と `VarTable::body_only` はどちらも
  `VarTable::empty()` から表を作る。`<1>3` より `origins` を変更するのは `<1>1` の `insert` だけなので、
  その鍵の記録を書いた呼び出しが在る。`<1>1` より
  記録を書くのは検査で記録を見つけなかった呼び出しであり、それはその鍵の呼び出しなので、どれもが記録を
  見つけるという仮定に反する。
  BY <1>1, <1>3, CODE src/rc_ir/ownership.rs: VarTable::empty, VarTable::of, VarTable::body_only

<1>4. QED
  (a) について。鍵 `(x, π)` について呼び出しが在るとする。`<1>1` より各呼び出しの返り値は、記録の値か、
  いま行った `grow_stack(|| origin_inner(..))` の値で
  ある。`<1>2` より後者は `origin_inner(vars, type_env, x, π)` の値であり、`<1>3` より前者もある時点の
  同じ呼び出しの値である。`<1>3a` より鍵 `(x, π)` について記録は高々 1 度しか書かれないので、同じ
  `(x, π)` についてどの呼び出しも同じ値を返す。`<1>3b` よりその鍵の cold な呼び出しが在り、その値が
  この共通の値である。(b) について。`<1>3a` が高々 1 つを、`<1>3b` が少なくとも 1 つを与える。
  (c) は `<1>1a` である。
  BY <1>1, <1>1a, <1>2, <1>3, <1>3a, <1>3b

## L0a (呼び出しは辺に沿って伝わり、鍵の関係は整礎である) <!--#cabeb3c-->

**DEF 鍵の関係**
鍵 `k = (x, π)` と `k' = (y, σ)` について、**`k ⇝ k'`** とは、`origin_inner(vars, type_env, x, π)` の
評価が `origin(vars, type_env, y, σ)` の呼び出しを**直接**行うことをいう。「直接」とは、その呼び出しが
`origin_inner` の本体 (それが呼ぶ `origin_from_leaves_under` の中を含む) が行うものであって、別の
`origin` の呼び出しの実行区間の中で始まるものではないことをいう。

**言明**。次の 2 つが成り立つ。

- **(a)** `k ⇝ k'` であり、鍵 `k` について `origin` の呼び出しが在るならば、鍵 `k'` についても `origin` の
  呼び出しが在る。
- **(b)** 鍵 `k_0` について `origin` の呼び出しが在るとき、`k_0` から始まる `⇝` の無限列
  `k_0 ⇝ k_1 ⇝ k_2 ⇝ …` は無い。すなわち `k_0` から `⇝` で到達できる鍵の上で `⇝` は整礎であり、それに
  ついての帰納法が使える。

**(a) が要るのは L4 である。** L4 は前件として「解析がその鍵で `origin` を呼ぶ」を置くので、帰納の各段は
その前件を後続の鍵について立て直す要がある。

**この命題が要るのは `origin` が memo を持つからである。** `origin(y, σ)` が memo に当たれば
`origin_inner` を評価せず、下位の呼び出しを 1 つも始めない。よって「呼び出しの中で始まる呼び出し」と
いう動的な関係は `⇝` と一致せず、動的な関係の整礎性からは `⇝` の整礎性が出ない。

<1>1. `origin(vars, type_env, x, π)` は、`vars.origins` に鍵 `(x, π)` の記録があればその値を返し、
      無ければ `origin_inner` を評価し、その値を鍵 `(x, π)` で記録してから返す。すなわち鍵 `(x, π)` の
      記録を書くのはその鍵の cold な呼び出しだけであり、書き込みはその呼び出しが返る直前にある。
  BY CODE src/rc_ir/ownership.rs: origin

<1>2. `k ⇝ k'` であり `k` の cold な呼び出し `C(k)` が在るならば、`k'` の cold な呼び出し `C(k')` が
      ちょうど 1 つ在り、`C(k')` は `C(k)` より前に返る。
  <2>1. `C(k)` は `origin_inner` を評価し、DEF 鍵の関係 よりその評価は `origin(k')` の呼び出し `B` を
        直接行う。`B` はこの `vars` と `type_env` を引数に取る `origin` の呼び出しなので L0 (c) より
        停止し、`C(k)` も同じ理由で停止する。`B` は `C(k)` の実行区間の中で始まって終わるので、
        EXT 呼び出しの入れ子 より `B` の実行区間は `C(k)` の実行区間に含まれ、`B` は `C(k)` より前に
        返る。
    BY DEF 鍵の関係, EXT 呼び出しの入れ子, <ref id=ecdd35d/> (b), <ref id=ecdd35d/> (c)
  <2>2. `k'` の cold な呼び出し `C(k')` がちょうど 1 つ在る。`<2>1` より `origin(k')` の呼び出しが
        在るので L0 (b) が当たる。
    BY <ref id=ecdd35d/> (b), <2>1
  <2>3. CASE `B` が記録を見つけない。
    `<1>1` より `B` は `origin_inner` を評価するので `B` は `k'` の cold な呼び出しであり、`<2>2` の
    一意性より `B = C(k')` である。`<2>1` より `B` は `C(k)` より前に返る。
    BY <ref id=ecdd35d/> (b), <1>1, <2>1, <2>2
  <2>4. CASE `B` が記録を見つける。
    `B` の検査の時点に鍵 `k'` の記録が在るので、それを書いた呼び出しが在る。`<1>1` よりそれは `k'` の
    cold な呼び出しであり、`<2>2` の一意性よりそれは `C(k')` である。`<1>1` より書き込みは `C(k')` が
    返る直前にあるので、`C(k')` は `B` の検査より前に返る。`<2>1` より `B` は `C(k)` の実行区間に
    含まれ、`C(k)` は返るので、`B` の検査は `C(k)` が返るより前にある。よって `C(k')` は
    `C(k)` より前に返る。
    BY <ref id=ecdd35d/> (b), <1>1, <2>1, <2>2
  <2>5. QED
    `<2>3` と `<2>4` は `B` が記録を見つけるか否かの 2 つの場合で尽きている。
    BY <2>2, <2>3, <2>4

<1>2a. (a) が成り立つ。
  `k ⇝ k'` であり鍵 `k` について `origin` の呼び出しが在るとする。L0 (b) よりその鍵の cold な呼び出し
  `C(k)` が在るので、`<1>2` より `k'` の cold な呼び出しが在る。L0 の言明より cold な呼び出しは
  `origin` の呼び出しである。
  BY <ref id=ecdd35d/> (b), <1>2

<1>3. QED
  (a) は `<1>2a` である。(b) について。無限列 `k_0 ⇝ k_1 ⇝ k_2 ⇝ …` が在るとする。前提より `k_0` に
  ついて `origin` の呼び出しが在るので、
  L0 (b) より `k_0` の cold な呼び出し `C(k_0)` が在り、L0 (c) より `C(k_0)` は返る。`<1>2` を
  繰り返すと、各 `k_i` の cold な
  呼び出し `C(k_i)` の無限列であって、`C(k_{i+1})` が `C(k_i)` より前に返るものが得られる。返る時点は
  狭義に早くなっていくので、この列の呼び出しは互いに相異なる。ところが EXT 有限の時間に有限の呼び出し
  より、`C(k_0)` が返る時点までに返った呼び出しは有限個である。矛盾。
  BY EXT 有限の時間に有限の呼び出し, <ref id=ecdd35d/> (b), <ref id=ecdd35d/> (c), <1>2, <1>2a

## L0b (`ρ` の位置の数え上げ) <!--#09fabad-->

**言明**。`ρ` を実行路とし、名前が `ρ` の上で値を得る 4 つの形 (S1)-(S4) を DEF 路の位置 のとおりと
する。次の 3 つが成り立つ。

- **(a)** (S1)-(S4) は D6 が数える 3 つの形と同じものを数える。(S1) がパラメータ・capture、(S2) と (S3)
  が節点が束縛する変数、(S4) が `vars.bindings` に束縛を持たない名前である。
- **(b)** 名前 `x` が `ρ` の上で値を得ることと、`ρ` を辿るある時点までに `x` が値を得ていることは同値で
  ある。そのとき、`x` の値の inhabited (D16) な boxed leaf `λ` との対 `(x, λ)` は D6 の意味の `ρ` の
  位置である -- (S1) から (S3) の `(x, λ)` は `ρ` のスロット、(S4) の `(x, λ)` は `ρ` の記号の位置で
  ある。**逆向きは 2 つに分かれる。** `ρ` の位置 `(x, λ)` について `λ` は `ty(x)` の inhabited な
  boxed leaf である。`(x, λ)` が `ρ` の**スロット**であるならば、`x` は (S1) から (S3) のいずれかの形で
  `ρ` の上で値を得る。**記号の位置については逆向きを主張しない** -- D6 の記号の位置は `g` が `ρ` の上に
  現れることを条件に持たないので、`ρ` のどの節点も名指さない記号の位置が在りうる。
- **(c)** `obj(x, λ)` は `x` が得る値と `λ` だけで決まり、`ρ` の上の位置に依らない。参照を経由しない。

<1>1. (S1) の `x` は活性化の**入力の束縛** (D23) から値を得ており、本体の最初の節点の時点で既に値を持つ。
      本体がグローバル初期化子の `init` であるときは、D1 より `init` はパラメータも capture も持たない
      ので、(S1) は起きない。
  BY <ref id=a502f3e/>, <ref id=ff5985d/>

<1>2. D2 より、本体が変数を束縛する節点は `Let`、`Destructure`、`Match` のアームの payload の 3 つだけ
      なので、(S2) と (S3) は本体の束縛をすべて尽くす。(S2) と (S3) の `x` はその節点が束縛する変数で
      あり、その節点を実行した段の後は値を持つ。
  BY <ref id=b3dfa37/>

<1>3. (S4) の `x` は本体が束縛しない名前であり、D6 より必ず最上位の記号の名前 -- D1 の `globals` の
      記号か `funcs` の記号 -- である。逆に (S1) から (S3) の `x` は `vars.bindings` に記録を持つ。
  表を作るのは、本体が関数の `body` なら `VarTable::of`、グローバル初期化子の `init` なら
  `VarTable::body_only` である (§1)。前者はパラメータ・capture を `Binding::Param` として記録した
  うえで `collect_bindings` を呼び、後者は `collect_bindings` を呼ぶだけである -- D1 より `init` は
  パラメータも capture も持たないので、記録しうるものがそれで尽きる。`collect_bindings` は歩いた本体の
  すべての束縛を表に入れるので、`vars.bindings` に無い名前は本体が束縛しない名前であり、逆に
  パラメータ・capture と束縛節点の変数は表に入る (A11)。
  BY <ref id=3905b4e/>, <ref id=a502f3e/>, <ref id=596a46d/>, CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only,
     CODE src/rc_ir/ownership.rs: collect_bindings

<1>3a. (S4) の `x` について、`(x, λ)` は `ρ` を辿るある時点 -- `x` が現れる節点の段より後 -- で記号の
      位置である。
  D6 は、記号の位置が値を持つのはその記号のグローバル化の段より後の時点であると述べ、同じ節で、`x` を
  読む節点は必ず値を読むこと -- まだ初期化されていなければその節点の段が先に初期化を走らせること -- を
  述べる。
  BY <ref id=596a46d/>

<1>4. (a) が成り立つ。
  D6 は「値を得る形は 3 つあり、スロットが在るのはそのうち 2 つである」と数え、その 3 つは節点が束縛する
  変数、パラメータ・capture、`vars.bindings` に束縛を持たない名前である。`<1>1` の (S1) がパラメータ・
  capture に、`<1>2` の (S2) と (S3) が節点が束縛する変数に、`<1>3` の (S4) が束縛を持たない名前に
  当たり、`<1>2` よりこの 4 つは本体の束縛を尽くす。
  BY <ref id=596a46d/>, <1>1, <1>2, <1>3

<1>5. (b) が成り立つ。
  D6 のスロットは「その実行路の上でその時点までに値を得た変数」を主語に取り、記号の位置も同じ形で
  `ρ` へ広げられる (D6)。`<1>4` より (S1)-(S4) は D6 の 3 つの形を尽くすので、`x` が `ρ` の上で値を
  得ることと、`ρ` を辿るある時点までに `x` が値を得ていることは同値である -- (S1) は `<1>1` より本体の
  最初の節点の時点で、(S2) と (S3) は `<1>2` よりその束縛節点の段の後で、値を持つ。(S4) は `<1>3a` より、
  その節点の段より後のある時点で記号の位置であり、記号の位置は値を持つ (D6) ので、その時点で値を持つ。
  D6 より、値を得た変数とその値の inhabited (D16) な boxed leaf との対が `ρ` の位置で
  あり、束縛を持つ名前の対はスロット、束縛を持たない名前の対は記号の位置である。`<1>3` より (S4) の
  名前は束縛を持たず、(S1) から (S3) の名前は束縛を持つ。
  逆向きについて。D6 より `ρ` の位置の第 2 成分はその変数の値の inhabited な boxed leaf である。
  `ρ` のスロットの第 1 成分は、D6 より束縛を持つ名前 -- 節点が束縛する変数か、パラメータ・capture --
  であって、`ρ` を辿るある時点までに値を得たものである。前者は D2 の 3 種の束縛節点のいずれかが束縛し、
  その節点は `ρ` の上にある (値を得た時点までに実行されている)。`Match` のアームの payload の場合は
  D2 よりその名前のスコープはそのアームの `body` の部分木なので、その名前が値を得るのは `ρ` がその
  アーム本体を辿るときである。D3 の第 2 の規則より `ρ` がアーム本体を辿るのは `ρ` がその `Match` で
  そのアームを選ぶときなので、`ρ` はそのアームを選んでいる -- すなわち (S2) か (S3) である。後者は
  (S1) である。
  BY <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=596a46d/>, <ref id=66c9670/>, <1>1, <1>2, <1>3,
     <1>3a, <1>4

<1>6. (c) が成り立つ。
  A6 より束縛名は相異なるので、`x` を束縛する節点は高々 1 つである。D2 は「節点が自分自身を含むことは
  ない」「本体は有限の木であり、繰り返しは関数呼び出しでしか作れない」と述べるので、1 つの活性化の中で
  1 つの節点が 2 度実行されることはない。D6 は「変数の値は、それを束縛する節点の後は変わらない」と述べ、
  `obj(x, λ)` を位置に依らない記法で書けるのはこれによるとする。(S4) の `x` の値はその記号の値であって
  1 つである (D6)。よって 1 つの活性化・1 つの路について `x` が得る値は 1 つである。D4 より leaf は値の
  根からのフィールド添字の列なので、
  `(x, λ)` は `x` の値の位置 `λ` を名指し、D6 よりその位置にあるオブジェクトが `obj(x, λ)` である。
  記号の位置は値とオブジェクトを持ち (D6)、グローバル状態のオブジェクト (D26) を指す leaf もそれが
  指すオブジェクトを持つので、この量は参照を経由しない。
  BY <ref id=33c54dc/>, <ref id=b3dfa37/>, <ref id=0594f24/>, <ref id=596a46d/>, <ref id=88a06de/>

<1>7. QED
  BY <1>4, <1>5, <1>6

## L1b (変位アームは scrutinee の活性変位のアームである) <!--#71210de-->

**言明**。`Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、`ρ` を辿る実行がその `Match` で
`tag = Some(t)` のアームを選ぶとする。このとき、その位置での `s` の値の実行時のタグは `t` である。

<1>1. `ρ` を辿る実行がこの `Match` で選ぶアームは、`s` の値の実行時のタグに `tag` が等しいアームであり、
      そのようなアームが無ければ、コード生成の振る舞いが決めるアームである。
  BY <ref id=c232680/>

<1>2. CASE `arms` に、`s` の値の実行時のタグ `t*` に `tag` が等しいアームがある。
  <2>1. `<1>1` より、選ばれるのはそのアームである。前提より選ばれたアームの `tag` は `Some(t)` なので、
        `Some(t) = Some(t*)` すなわち `t = t*` である。
    BY <1>1
  <2>2. QED
    BY <2>1

<1>3. CASE `arms` に、`s` の値の実行時のタグに `tag` が等しいアームが無い。
  <2>1. `arms` は catch-all アームを持つ。
    A16 の (網羅) より、`arms` が catch-all アームを持つか、`s` の値の実行時のタグがいずれかのアームの
    `tag` である。後者はこの CASE の前提に反する。
    BY <ref id=f769887/>
  <2>2. catch-all アームは `arms` の最後である。
    `<2>1` の catch-all アームについて A16 の (位置) がこれを与える。
    BY <ref id=f769887/>, <2>1
  <2>3. QED
    コード生成は、最後のアームのブロックを `else_bb` とし、アームが 1 つのときはそこへ無条件に分岐し、
    2 つ以上のときは最後を除く各アームをその `tag` の case とする switch の default にそれを据える。
    どちらでも `<1>1` の第 2 の場合に実行が入るのは最後のアームであり、`<2>1` と `<2>2` より
    それは catch-all アーム、すなわち `tag` が `None` のアームである。これは前提の `Some(t)` に反するので、
    この CASE は起きない。
    BY <1>1, <2>1, <2>2, CODE src/rc_ir/codegen.rs: Generator::eval_rc_match

<1>4. QED
  `<1>2` と `<1>3` は「`s` の値の実行時のタグに `tag` が等しいアームがある」という命題について、
  それが成り立つ場合と成り立たない場合であり、EXT 排中律 より場合を尽くしている。
  BY EXT 排中律, <1>2, <1>3

## L1 (実行された辺の両端は同じオブジェクトを指す) <!--#4c886c1-->

**言明**。次の 2 つが成り立つ。

- **(a)** この文書の意味で在り (DEF 辺の存在)、かつ `ρ` の上で実行された (DEF `ρ` の上で実行された辺)
  E1 から E6 の辺の両端の位置は、同じ値の同じ位置を名指し、したがって同じオブジェクトを指す。
- **(b)** 「この文書の意味で在り (DEF 辺の存在)、かつ `ρ` の上で実行された辺」と「D20 の意味で在る辺」は
  同じものである。

<1>1. 2 つの位置 `(u, α)` と `(w, β)` について、`u` の値の位置 `α` と `w` の値の位置 `β` が同じ値の
      同じ位置であれば、`obj(u, α) = obj(w, β)` である。
  L0b (c) より `obj(u, α)` は `u` が得る値と `α` だけで決まる量であり、D6 の `obj` と同じものである。
  同じ値の同じ位置は 1 つのオブジェクトを持つ。
  BY <ref id=0594f24/>, <ref id=596a46d/>, <ref id=09fabad/>, DEF 路の位置

<1>1a. E1 から E6 の各行の辺の leaf 対応 (DEF 辺の leaf 対応) は、D9 の値の水準の対応する行が始点の値と
       終点の値のあいだに定める位置の対応である。
  D9 の移動の表の 6 行には値の水準の行が 1 つずつ対応する (D9 の「値の水準では、移動の各行は次を渡す」の
  6 行)。値の水準の行は始点の値と終点の値の関係を述べ、D4 より leaf は値の根からのフィールド添字の列
  なので、この関係は始点の値の位置と終点の値の位置のあいだの対応を与える。E1 から E4 と E6 の 5 行に
  ついては、その対応は行の文言そのものである -- 「`x` の値は `y` の値である」は `λ` ↔ `λ`、
  「フィールド変数の値は容器の値のそのフィールドである」は `[i] ++ λ` ↔ `λ`、「payload 変数の値は
  scrutinee の値の活性変位の payload である」は `[t] ++ λ` ↔ `λ`、「payload 変数の値は scrutinee の値
  そのものである」は `λ` ↔ `λ`、「`Match` の束縛変数の値は `x` の値である」は `λ` ↔ `λ` である。
  E5 の行は対にする leaf を宣言の path として名指す -- 「結果の leaf `λ` の宣言が単一の `Arg(i, σ)` で
  あるとき、その leaf の値は**オペランド `i` の leaf `σ` の値**である」であり、続けて「**`λ` と `σ` は
  一般に別の path である**」と述べる。よって E5 の leaf 対応は `σ` ↔ `λ` である。
  BY <ref id=0594f24/>, <ref id=9d74736/>, DEF 辺の leaf 対応

<1>2. E1 の辺 `(y, λ)`-`(x, λ)`。
  <2>1. `x` の値は `y` の値である。
    D9 の値の水準の第 1 行 (`Let(x, Var(y), k)`) が「`x` の値は `y` の値である」と述べる。
    BY <ref id=9d74736/>
  <2>2. QED
    `<2>1` より `x` の値の位置 `λ` と `y` の値の位置 `λ` は同じ値の同じ位置である。DEF 辺の leaf 対応 が
    この行の leaf の対応を `λ` ↔ `λ` と読む。
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>1

<1>3. E2 の辺 `(c, [i] ++ λ)`-`(f, λ)`。
  <2>1. `f` の値は `c` の値の第 `i` フィールドである。
    D9 の値の水準の第 3 行 (unbox 容器の `Destructure` の名前付きフィールド) が「フィールド変数の値は
    容器の値のそのフィールドである」と述べる。
    BY <ref id=9d74736/>
  <2>2. QED
    `<2>1` より `f` の値の位置 `λ` は `c` の値の位置 `[i] ++ λ` と同じ値の同じ位置である。DEF 辺の
    leaf 対応 がこの行の leaf の対応を `[i] ++ λ` ↔ `λ` と読む。
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>1

<1>4. E3 の辺 `(s, [t] ++ λ)`-`(p, λ)`。
  <2>1. この辺が `ρ` の上で実行されたとき、`ρ` を辿る実行はこの `Match` で `tag = Some(t)` のアームを
        選んでおり、この位置での `s` の値の実行時のタグは `t` である。
    BY <ref id=71210de/>, DEF `ρ` の上で実行された辺
  <2>2. `p` の値は `s` の値の変位 `t` の payload である。
    D9 の値の水準の第 4 行 (unbox union の変位アームの payload 束縛) が「payload 変数の値は scrutinee の
    値の活性変位の payload である」と述べる。`<2>1` よりこの位置での活性変位は `t` である。
    BY <ref id=9d74736/>, <2>1
  <2>3. QED
    `<2>2` より `p` の値の位置 `λ` は `s` の値の位置 `[t] ++ λ` と同じ値の同じ位置である。DEF 辺の
    leaf 対応 がこの行の leaf の対応を `[t] ++ λ` ↔ `λ` と読む。
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>2

<1>5. E4 の辺 `(s, λ)`-`(p, λ)`。
  <2>1. `p` の値は `s` の値そのものである。
    この辺が `ρ` の上で実行されたとき、DEF `ρ` の上で実行された辺 の第 2 の場合より、その辺を定める
    `Let(m, Match(s, arms), k)` の節点は `ρ` の上にあり、`ρ` を辿る実行はその `Match` で `p` を payload
    とする catch-all アームを選ぶ。D9 の値の水準の第 5 行 (catch-all アームの payload 束縛) が
    「payload 変数の値は scrutinee の値そのものである」と述べ、その行が当たるのは、その活性化が選んだ
    アームの payload 束縛についてである (D21)。選ばれなかったアームの payload 変数はその路で値を得ない。
    BY <ref id=9d74736/>, <ref id=c232680/>, DEF `ρ` の上で実行された辺
  <2>2. QED
    `<2>1` より `p` の値の位置 `λ` と `s` の値の位置 `λ` は同じ値の同じ位置である。DEF 辺の leaf 対応 が
    この行の leaf の対応を `λ` ↔ `λ` と読む。
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>1

<1>6. E5 の辺 `(args[j], σ)`-`(x, λ)`。
  <2>1. `x` の値の位置 `λ` の値は、`args[j]` の値の位置 `σ` の値である。
    D9 の値の水準の第 6 行 (`Llvm` の素通し leaf) が「結果の leaf `λ` の宣言が単一の `Arg(i, σ)` である
    とき、その leaf の値は**オペランド `i` の leaf `σ` の値**である」と述べる。第 2 節の E5 は、結果の
    leaf `λ` の宣言が単一の `Arg(j, σ)` であるときの辺を `(args[j], σ)`-`(x, λ)` と定めるので、この行が
    オペランドの側で名指す leaf は E5 の辺の始点そのものである (`<1>1a`)。
    BY <ref id=9d74736/>, DEF 辺の leaf 対応, <1>1a
  <2>2. QED
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>1

<1>7. E6 の辺 `(x, λ)`-`(m, λ)`。
  <2>1. `m` の値は `x` の値である。
    この辺が `ρ` の上で実行されたとき、DEF `ρ` の上で実行された辺 の第 3 の場合より、その辺を定める
    アーム本体の終端の `Ret(x)` は `ρ` の上にあり、D3 より `ρ` はそのアームを通る。D9 の値の水準の
    第 2 行 (`Match` のアーム本体の `Ret(x)`) が「`Match` の束縛変数の値は `x` の値である」と述べ、
    その行が当たるのは、その活性化が選んだアームの `Ret` についてである (D21)。選ばれなかったアームの
    `Ret(x')` の `x'` はその路で値を得ない。
    BY <ref id=ca36627/>, <ref id=9d74736/>, <ref id=c232680/>, DEF `ρ` の上で実行された辺
  <2>2. QED
    `<2>1` より `m` の値の位置 `λ` と `x` の値の位置 `λ` は同じ値の同じ位置である。DEF 辺の leaf 対応 が
    この行の leaf の対応を `λ` ↔ `λ` と読む。
    BY DEF 辺の leaf 対応, <1>1, <1>1a, <2>1

<1>7a. (b) が成り立つ。
  D20 は、辺が在るのは「その辺を定める節点が実行路の上に在り、かつその節点と leaf `λ` が作る 2 つの対が
  どちらもその路の位置であるとき」であり、アームの中の行 (変位アームの payload 束縛、catch-all アームの
  payload 束縛、アーム本体の `Ret`) については路がそのアームを選ぶことも要ると述べる。この文書はその
  3 つの条件を 2 つの語に割る -- 2 つの対が `ρ` の位置であることを DEF 辺の存在 が、節点が `ρ` の上に
  在ることと路がそのアームを選ぶことを DEF `ρ` の上で実行された辺 が担う。E1、E2、E5 の行はアームの中に
  無いので、DEF `ρ` の上で実行された辺 が課すのは節点が `ρ` の上に在ることだけである。E3、E4 の行は
  その `Match` の節点が `ρ` の上に在り `ρ` がそのアームを選ぶことを課す。E6 の行はそのアーム本体の
  終端の `Ret` が `ρ` の上に在ることを課し、D3 より実行路はアームを 1 つ選んでそのアーム本体の実行路を
  辿るので、これはその `Match` の節点が `ρ` の上に在り `ρ` がそのアームを通ることと同じである。よって
  2 つの語の連言は D20 の 3 つの条件に一致する。
  BY <ref id=ca36627/>, <ref id=9c7c27a/>, DEF 辺の存在, DEF `ρ` の上で実行された辺

<1>8. QED
  (a) について。E1 から E6 は D20 (すなわち D9 の移動の表) の 6 行に 1 対 1 で対応し、値の水準の 6 行も
  その 6 行に 1 対 1 で対応する (D9)。`<1>2` から `<1>7` がその 6 つであり、どれも両端が同じ値の同じ
  位置を名指すことを与える。辺が在る (DEF 辺の存在) ことは両端が `ρ` の位置であることなので、`<1>1` の
  前提が揃い、両端は同じオブジェクトを指す。(b) は `<1>7a` である。
  BY <ref id=9d74736/>, <ref id=9c7c27a/>, DEF 辺の存在, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>7a

## L1a (`Binding` の形は `ρ` の上の束縛節点の形である) <!--#cc9c0f1-->

**言明**。`(x, λ)` を `ρ` の位置 (DEF 路の位置) とし、`vars.bindings.get(x)` が `Some(b)` であるとする。
このとき次が成り立つ。

- **(a)** `b = Param` であることと、`x` がこの本体のパラメータか capture であることは同値である。
- **(b)** `b ≠ Param` のとき、`x` を束縛する節点が `ρ` の上にちょうど 1 つあり、その形は `b` の構成子に
  応じて次のとおりである。さらに、その節点が定める辺が**在れば** (すなわち両端が `ρ` の位置で
  あれば)、それは `ρ` の上で実行された辺である。

| `b` | `ρ` の上の節点 | 定める辺 |
|---|---|---|
| `Move(y)` | `Let(x, Var(y), k)` | E1 の `(y, λ)`-`(x, λ)` |
| `Producer` | `Let(x, App(callee, args), k)` または `Let(x, Closure(f, caps), k)` | 無し |
| `Field(c, i)` | `Destructure(c, fs, s, k)` で `(i, x) ∈ fs` | `c` が unbox のとき E2 の `(c, [i] ++ λ)`-`(x, λ)` |
| `Payload(s, Some(t))` | `Let(m, Match(s, arms), k)` で、`ρ` が選ぶアームが `tag = Some(t)`、`payload = x` のもの | `s` が unbox のとき E3 の `(s, [t] ++ λ)`-`(x, λ)` |
| `Payload(s, None)` | `Let(m, Match(s, arms), k)` で、`ρ` が選ぶアームが catch-all、`payload = x` のもの | E4 の `(s, λ)`-`(x, λ)` |
| `Llvm(gen, args, ty)` | `Let(x, Llvm(gen, args), k)` で `ty = ty(x)` | 宣言が単一の `Arg(j, σ)` のとき E5 の `(args[j], σ)`-`(x, λ)` |
| `Join(rs)` | `Let(x, Match(s, arms), k)` | `ρ` が選ぶアーム `a` について `r_0 := returned_var(a.body)` とおくと、`r_0 ∈ rs` であり、E6 の `(r_0, λ)`-`(x, λ)` |

<1>1. `vars` が関数の本体から作られたものであるとき (`VarTable::of`)、それはパラメータと capture の
      それぞれについて `Binding::Param` を記録し、その後 `collect_bindings` を呼ぶ。`vars` が
      グローバル初期化子の `init` から作られたものであるとき (`VarTable::body_only`)、それは
      `collect_bindings` を呼ぶだけであり、`Binding::Param` を 1 つも記録しない。どちらの場合も
      `collect_bindings` は `Param` を記録しない。
  BY CODE src/rc_ir/ownership.rs: VarTable::of, VarTable::body_only,
     CODE src/rc_ir/ownership.rs: collect_bindings

<1>2. `collect_bindings` が `x` について記録を作るのは、次の 3 つの節点でだけであり、記録される構成子は
      次のとおりである。
      `Let(x, rhs, k)`: `rhs` が `Var(y)` なら `Move(y)`、`Llvm(gen, args)` なら
      `Llvm(gen, args, x.ty)`、`Closure(..)` または `App(..)` なら `Producer`、`Match(s, arms)` なら
      `Join(arm_results)` で `arm_results` は各アームの `returned_var(&arm.body)` の列。
      `Destructure(c, fs, _, k)` の各 `(i, fv)`: `fv` について `Field(c, i)`。
      `Let(m, Match(s, arms), k)` の各アーム: `arm.payload` について `Payload(s, arm.tag)`。
      `returned_var` の本体は `grow_stack` に包まれているので、その値を読むのに A15 が要る。
  BY <ref id=3e6b0e0/>, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var

<1>3. (a) が成り立つ。
  <2>1. CASE `vars` が `VarTable::of` で作られた (本体が関数の `body` である)。
    `x` がパラメータか capture であるとき、`<1>1` が `x` について `Param` を記録し、A6 より本体はその
    名前を束縛し直さないので、`<1>2` の記録がそれを上書きすることはない。よって `b = Param` である。
    逆に `b = Param` であるとき、`<1>2` より `collect_bindings` が作る構成子は `Move`、`Llvm`、
    `Producer`、`Join`、`Field`、`Payload` の 6 つで `Param` を含まないので、この記録は `<1>1` が
    作ったものであり、`x` はパラメータか capture である。
    BY <ref id=33c54dc/>, <1>1, <1>2
  <2>2. CASE `vars` が `VarTable::body_only` で作られた (本体がグローバル初期化子の `init` である)。
    D1 より `init` はパラメータも capture も持たないので、(a) の第 2 の側 -- `x` がこの本体のパラメータか
    capture であること -- は偽である。`<1>1` より `body_only` は `Param` を 1 つも記録せず、`<1>2` より
    `collect_bindings` の作る構成子も `Param` を含まないので、`b = Param` も偽である。よって 2 つは
    同値である。
    BY <ref id=a502f3e/>, <1>1, <1>2
  <2>3. QED
    D23 より、固定した本体は関数の `body` かグローバル初期化子の `init` かのどちらかであり、`vars` は
    それぞれ `VarTable::of` と `VarTable::body_only` が作る (§1)。
    BY <ref id=ff5985d/>, <2>1, <2>2

<1>4. `b ≠ Param` のとき、`x` を束縛する節点が `ρ` の上にある。
  前提より `vars.bindings.get(x)` は `Some(b)` なので `x` は束縛を持つ名前であり、D6 より束縛を持つ名前の
  対はスロットである。よって `(x, λ)` は `ρ` のスロットであり、L0b (b) の逆向きより `x` は (S1) から
  (S3) のいずれかの形で `ρ` の上で値を得る。(S1) は `<1>3` より `b = Param` を与えるので前提に反する。
  残るのは (S2) と (S3) であり、どちらも `x` を束縛する節点が `ρ` の上にあることを言う。
  BY <ref id=596a46d/>, <ref id=09fabad/>, DEF 路の位置, <1>3

<1>5. `x` を束縛する節点は高々 1 つであり、`b` はその節点が `<1>2` の規則で作る構成子である。
  BY <ref id=33c54dc/>, <1>2

<1>6. `b ≠ Param` のとき、`x` を束縛する節点が `ρ` の上に**ちょうど 1 つ**あり、(b) の表の節点の形が
      成り立つ。
  `<1>4` が少なくとも 1 つあることを、`<1>5` が高々 1 つであることを与える。`<1>2` の対応を逆に読むと、記録された
  構成子ごとにその節点の形が表のとおりに定まる。`Payload(s, tag)` の場合、`x` を束縛するのは (S3) の
  意味であり、`ρ` が選ぶアームの `payload` が `x` である。`Join(rs)` の場合、`rs` は各アームの
  `returned_var(&arm.body)` の列なので、`ρ` が選ぶアーム `a` の `returned_var(a.body)` は `rs` の元で
  ある。
  BY <ref id=09fabad/>, DEF 路の位置, <1>2, <1>4, <1>5

<1>7. (b) の表の辺が在れば、それは `ρ` の上で実行された辺である。DEF `ρ` の上で実行された辺 は、辺が
      実行されたことを、その辺を定める節点 (と、選ばれるアーム) だけで判定する。
  <2>1. `Move(y)`、`Llvm(gen, args, ty)`、`Field(c, i)` の場合、`<1>6` の節点はそれぞれ
        `Let(x, Var(y), k)`、`Let(x, Llvm(gen, args), k)`、`Destructure(c, fs, s, k)` であって `ρ` の
        上にある。DEF `ρ` の上で実行された辺 の第 1 の場合より、E1・E5・E2 の辺は在れば `ρ` の上で
        実行された。
    BY DEF `ρ` の上で実行された辺, <1>6
  <2>2. `Payload(s, Some(t))` と `Payload(s, None)` の場合、`<1>6` の節点は `Let(m, Match(s, arms), k)`
        であって `ρ` の上にあり、`ρ` が選ぶアームが `x` を payload とするアームである。DEF `ρ` の上で
        実行された辺 の第 2 の場合より、E3・E4 の辺は在れば `ρ` の上で実行された。
    BY DEF `ρ` の上で実行された辺, <1>6
  <2>3. `Join(rs)` の場合、`<1>6` の節点は `Let(x, Match(s, arms), k)` であって `ρ` の上にあり、`ρ` は
        アーム `a` を通る。D3 より `ρ` は `a.body` の実行路を辿り、その終端は `a.body` の終端の `Ret` で
        ある。`returned_var` は本体を `grow_stack` に包み (A15)、`Ret` に着くまで継続を辿る。D2 より
        `Ret` は唯一の終端子であり `Ret` 以外の 5 種は継続を 1 つ持つので、`returned_var`
        が着く `Ret` はその終端の `Ret` であり、それが名指す変数が `r_0` である。DEF `ρ` の上で実行された辺
        の第 3 の場合より、E6 の辺は在れば `ρ` の上で実行された。
    BY <ref id=3e6b0e0/>, <ref id=b3dfa37/>, <ref id=ca36627/>, DEF `ρ` の上で実行された辺, CODE src/rc_ir/ownership.rs: returned_var, <1>6
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>8. QED
  (a) は `<1>3` である。(b) の「ちょうど 1 つ」と表の節点の形は `<1>6` (その支えが `<1>4` と `<1>5`)、
  表の辺が `ρ` の上で実行されたものであることは `<1>7` である。
  BY <1>3, <1>4, <1>5, <1>6, <1>7

## L2 (5 種の束縛は `origin` を保つ) <!--#97d41ba-->

**言明**。次の 6 つが成り立つ。(E1) から (E5) の主語は `vars.bindings` の記録であって辺ではない --
辺が在ることを先に言わなくても `origin` の等式が使えるようにするためである (DEF 辺の存在)。
`λ` と `σ` は任意の path を渡る。**(E1) から (E5) はいずれも、解析が鍵 `(u, λ)` で `origin` を呼ぶ場合に
ついて述べ、右辺の鍵でも呼ぶことを併せて言う** -- 呼ばれない鍵について `origin` は答えを持たない (§1)。

- **(E1)** 解析が鍵 `(u, λ)` で `origin` を呼び、`vars.bindings.get(u) = Some(Move(y))` であるとき、
  解析は鍵 `(y, λ)` でも `origin` を呼び、`origin(u, λ) = origin(y, λ)` である。
- **(E2)** 解析が鍵 `(u, λ)` で `origin` を呼び、`vars.bindings.get(u) = Some(Field(c, i))` かつ
  `c.ty.is_box(type_env)` が偽であるとき、解析は鍵 `(c, [i] ++ λ)` でも `origin` を呼び、
  `origin(u, λ) = origin(c, [i] ++ λ)` である。
- **(E3)** 解析が鍵 `(u, λ)` で `origin` を呼び、`vars.bindings.get(u) = Some(Payload(s, Some(t)))` かつ
  `s.ty.is_box(type_env)` が偽であるとき、解析は鍵 `(s, [t] ++ λ)` でも `origin` を呼び、
  `origin(u, λ) = origin(s, [t] ++ λ)` である。
- **(E4)** 解析が鍵 `(u, λ)` で `origin` を呼び、`vars.bindings.get(u) = Some(Payload(s, None))` である
  とき、解析は鍵 `(s, λ)` でも `origin` を呼び、`origin(u, λ) = origin(s, λ)` である。
- **(E5)** 解析が鍵 `(u, λ)` で `origin` を呼び、`vars.bindings.get(u) = Some(Llvm(gen, args, ty))` で
  あって、`decl := gen.result_prov(ty, arg_tys, type_env)` の `decl.leaf_origins_at(λ)` が単一の
  `Arg(j, σ)` からなる集合であるとき、解析は鍵 `(args[j], σ)` でも `origin` を呼び、
  `origin(u, λ) = origin(args[j], σ)` である。ここで **`arg_tys` は `args` の各元の型の列**
  `args.iter().map(|a| a.ty.clone()).collect()` である。
- **(B)** 本体の節点が E1 から E5 のいずれかの辺を定めるとき、その辺の終点の変数の `vars.bindings` の
  記録は、その辺の種に応じてそれぞれ `Some(Move(y))`、`Some(Field(c, i))`、`Some(Payload(s, Some(t)))`、
  `Some(Payload(s, None))`、`Some(Llvm(gen, args, ty(x)))` である。

<1>1. `collect_bindings` は、`Let(x, Var(y), k)` に対し `x` の `Binding` を `Move(y)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Var(y)` の場合

<1>2. `collect_bindings` は、`Destructure(container, fields, _state, k)` の各 `(idx, fv)` に対し `fv` の
      `Binding` を `Field(container, idx)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Destructure` の腕

<1>3. `collect_bindings` は、`Let(x, Match(scrut, arms), k)` の各アームの `payload` に対し `Binding` を
      `Payload(scrut, arm.tag)` とする。`arm.tag` は catch-all のとき `None`、変位アームのとき
      `Some(t)` である。
  BY <ref id=b3dfa37/>, CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>4. `collect_bindings` は、`Let(x, Llvm(llvm_gen, args), k)` に対し `x` の `Binding` を
      `Llvm(llvm_gen, args, x.ty)` とする。
  BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合

<1>4a. 解析が鍵 `(u, λ)` で `origin` を呼ぶとき、その鍵の cold な呼び出しがちょうど 1 つ在り、
      `origin(u, λ)` の値はその呼び出しの `origin_inner(vars, type_env, u, λ)` が返した値である。
      よって `origin_inner` の腕が返す式を読めば `origin(u, λ)` の値が決まり、その腕が直接行う `origin` の
      呼び出しは解析が行う呼び出しである。
  BY <ref id=ecdd35d/> (a), <ref id=ecdd35d/> (b)

<1>5. (E1) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Move(y))` の腕は `origin(vars, type_env, &y.name, path)` を
        そのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. QED
    `<1>4a` より鍵 `(u, λ)` の cold な呼び出しが `origin_inner` を評価し、`<2>1` よりその腕は鍵 `(y, λ)` の
    `origin` を直接呼んでその値を返す。よって解析は鍵 `(y, λ)` でも `origin` を呼び、
    `origin(u, λ) = origin(y, λ)` である。
    BY <1>4a, <2>1

<1>6. (E2) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Field(container, idx))` の腕は、`container.ty.is_box(type_env)`
        が偽のとき `container_path` を `[*idx] ++ path` として作り
        `origin(vars, type_env, &container.name, &container_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕
  <2>2. QED
    `<1>4a` より鍵 `(u, λ)` の cold な呼び出しが `origin_inner` を評価し、`<2>1` よりその腕は
    鍵 `(c, [i] ++ λ)` の `origin` を直接呼んでその値を返す。よって解析はその鍵でも `origin` を呼び、
    `origin(u, λ) = origin(c, [i] ++ λ)` である。
    BY <1>4a, <2>1

<1>7. (E3) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の
        `Some(tag) if !scrut.ty.is_box(type_env)` の場合は、`scrut_path` を `[*tag] ++ path` として作り
        `origin(vars, type_env, &scrut.name, &scrut_path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. QED
    `<1>4a` より鍵 `(u, λ)` の cold な呼び出しが `origin_inner` を評価し、`<2>1` よりその腕は
    鍵 `(s, [t] ++ λ)` の `origin` を直接呼んでその値を返す。よって解析はその鍵でも `origin` を呼び、
    `origin(u, λ) = origin(s, [t] ++ λ)` である。
    BY <1>4a, <2>1

<1>8. (E4) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
        `origin(vars, type_env, &scrut.name, path)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. QED
    `<1>4a` より鍵 `(u, λ)` の cold な呼び出しが `origin_inner` を評価し、`<2>1` よりその腕は鍵 `(s, λ)` の
    `origin` を直接呼んでその値を返す。よって解析は鍵 `(s, λ)` でも `origin` を呼び、
    `origin(u, λ) = origin(s, λ)` である。
    BY <1>4a, <2>1

<1>9. (E5) が成り立つ。
  <2>1. `origin_inner` の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕は、`decl` を
        `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` として作り、
        `decl.leaf_origins_at(path).and_then(as_arg_projection)` が `Some((j, p))` のとき
        `origin(vars, type_env, &args[j].name, &p)` をそのまま返す。
    BY CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>2. `as_arg_projection(sources)` が `Some((j, p))` を返すのは、`sources` がちょうど 1 元からなり、
        その元が `LeafOrigin::Arg(j, p)` のときに限る。
    BY CODE src/rc_ir/ownership.rs: as_arg_projection
  <2>2a. `<2>1` の呼び出しが作る `decl` は、言明の `decl` と同じ値である。どちらも、記録
         `Llvm(gen, args, ty)` が持つ op に、記録が持つ結果の型 `ty`、`args` の各元の型の列、そして
         `type_env` を渡した `result_prov` の値である (`<2>1` の腕が `arg_tys` をそう作る)。引数が
         同じなので、A3 の決定性の節 -- `result_prov` は同じ引数に対して常に同じ値を返す -- が 2 つの
         値を等しくする。
    BY <ref id=e11772a/>, <2>1
  <2>3. 前提より `decl.leaf_origins_at(λ)` は 1 元集合 `{Arg(j, σ)}` なので、`as_arg_projection` はそれを
        `Some((j, σ))` に写す。`<2>2a` より、`<2>1` の呼び出しが `leaf_origins_at` を掛ける `decl` は
        言明の `decl` である。
    BY <2>2, <2>2a
  <2>4. QED
    `<2>3` よりこの腕は `Some((j, σ))` の枝を取る。`<1>4a` より鍵 `(u, λ)` の cold な呼び出しが
    `origin_inner` を評価し、`<2>1` よりその枝は鍵 `(args[j], σ)` の `origin` を直接呼んでその値を返す。
    よって解析はその鍵でも `origin` を呼び、`origin(u, λ) = origin(args[j], σ)` である。
    BY <1>4a, <2>1, <2>2a, <2>3

<1>9a. (B) が成り立つ。
  E1 から E5 の辺を定める節点は、第 2 節の一覧より、それぞれ `Let(x, Var(y), k)`、
  `Destructure(c, fs, s, k)` で `(i, f) ∈ fs`、`Let(m, Match(s, arms), k)` の `tag = Some(t)` のアーム、
  同じ節点の catch-all アーム、`Let(x, Llvm(gen, args), k)` である。`<1>1` から `<1>4` より
  `collect_bindings` はそのそれぞれについて、辺の終点の変数に (B) の構成子を記録する。辺の終点の変数は
  この本体が束縛する変数であり、A6 より束縛変数の名前は相異なるので、`collect_bindings` の他の記録は
  同じ名前に別の値を入れない。本体が関数の `body` であるとき `VarTable::of` が作る `Param` の記録は
  パラメータ・capture の名前に付き、A6 より本体はその名前を束縛し直さないので、これも同じ名前に当たら
  ない。本体がグローバル初期化子の `init` であるときは `VarTable::body_only` が `Param` を 1 つも
  記録しない -- D1 より `init` はパラメータも capture も持たない。
  BY <ref id=33c54dc/>, <ref id=a502f3e/>, <1>1, <1>2, <1>3, <1>4

<1>10. QED
  BY <1>5, <1>6, <1>7, <1>8, <1>9, <1>9a

**補足 (E1-E5 に入らない `origin_inner` の腕)**。`origin_inner` が `here()` を返して値自身を origin と
するのは、`Binding` が `None` / `Param` / `Producer` のとき、`Field` で容器が boxed のとき、`Payload` で
scrutinee が boxed のとき、`Llvm` で `as_arg_projection` も `origin_from_leaves_under` も答えを出さない
ときである (`CODE src/rc_ir/ownership.rs: origin_inner`)。このうち `Param` は D10 の初期値、`Producer` は
D10 の生成の表の `App` と `Closure` の行 (`CODE src/rc_ir/ownership.rs: collect_bindings` がこの 2 つに
`Producer` を付ける)、boxed の `Field` と boxed の `Payload` は D10 の生成の表の対応する行、`Llvm` は
D10 の生成の表の `Llvm` の行に当たる。残る `Binding::Join` の腕は E6 の辺であり、L2 の言明はそれを外して
いる。

## L3 (boxed leaf は互いに前置にならない) <!--#742afee-->

**言明**。**A10 を満たす**型 `τ` について、`boxed_leaf_paths(τ, type_env)` の相異なる 2 元の一方が他方の
前置になることはない。とくに、変数 `v` の型 `ty(v)` が A10 を満たし、`π` が
`boxed_leaf_paths(ty(v), type_env)` の元であるとき、`L(v, π)` は `{π}` である。

A10 を満たすことを言明が要求するのは、証明が `go` の再帰の整礎性を A10 から取るからである。README の P1 が
同じ限定を置くのも同じ理由による。

<1>1. `boxed_leaf_paths` の内部の走査 `go(ty, type_env, path, out)` は、3 つの腕で `out` に path を積んだ
      直後に `return` する。`ty.is_closure()` の腕は `path ++ [CLOSURE_CAPTURE_IDX]` を積み、
      `ty.is_box(type_env)` の腕と `ty.is_array()` の腕は `path` を積む。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2. `go` が path を積むのはその 3 つの腕だけであり、`ty.is_fully_unboxed(type_env)` の腕は積まずに
      `return` し、最後の腕は `unpunched_field_types` が返す各 `(i, fty)` について `path ++ [i]` を
      引数として `go` を呼ぶ (他の再帰呼び出しは無い)。
  BY <ref id=0594f24/>, CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>2a. 引数 `path` が `q` である `go` の 1 回の呼び出しが、その再帰呼び出しを含めて `out` に積む path の
       列を `Push(q)` と書く。以下、path `a` が path `b` の**前置**であるとは `b` が `a` で始まることを
       いい、`a = b` の場合を含む。`<1>1` と `<1>2` より `Push(q)` は次のいずれかである。
       空の列 (`is_fully_unboxed` の腕)、`[q ++ [CLOSURE_CAPTURE_IDX]]` (`is_closure` の腕)、
       `[q]` (`is_box` の腕と `is_array` の腕)、`Push(q ++ [i])` を `i` について連ねた列 (最後の腕、
       `i` は `unpunched_field_types` が返す添字を渡る)。
  BY <1>1, <1>2

<1>2aa. `go` の再帰呼び出しの列は有限である。すなわち `<1>2a` の関係は整礎である。
  <2>1. `go` が再帰するのは、`is_fully_unboxed`・`is_closure`・`is_box`・`is_array` の 4 つの早期 `return`
        をすべて外れた型についてだけであり、その再帰先は `unpunched_field_types` が返す各フィールドの
        型である。
    BY <1>1, <1>2
  <2>2. QED
    A10 は、A10 を満たす型について「`unpunched_field_types` を繰り返し取って到達する型」の歩みが有限で
    あるとする。`<2>1` より `go` の再帰の辺はその歩みの辺なので、`go` の再帰呼び出しの列も有限である。
    BY <ref id=8412761/>, <2>1

<1>2b. `Push(q)` の各成分は `q` を前置に持つ。
  `<1>2aa` より `<1>2a` の再帰は整礎であり、それについての帰納法が使える。
  空の列には成分が無い。`[q ++ [CLOSURE_CAPTURE_IDX]]` と `[q]` の成分は `q` を前置に持つ。最後の場合、
  帰納法の仮定より `Push(q ++ [i])` の各成分は `q ++ [i]` を前置に持ち、`q` はその前置である。
  BY <1>2a, <1>2aa

<1>2c. `unpunched_field_types` が返す対の第 1 成分は互いに相異なる。
  この関数は `instance_field_types` が返す列に `enumerate()` を掛けて添字を付け、`filter` で穴の
  フィールドを落とす。EXT `Iterator::enumerate` と `Iterator::filter` より、付く添字は 0 から始まって
  1 ずつ増える狭義単調増加の列であり、`filter` は順序を保つので、残った対の第 1 成分も狭義単調増加で
  あり、したがって互いに相異なる。よって
  `<1>2a` の最後の場合で連ねられる `Push(q ++ [i])` の `i` は互いに相異なる。
  BY EXT `Iterator::enumerate` と `Iterator::filter`,
     CODE src/ast/types.rs: TypeNode::unpunched_field_types

<1>3. `Push(q)` の異なる 2 つの位置にある成分 `P`、`P'` について、`P` は `P'` の前置ではない。
      とくに (前置が等号を含むので) `P ≠ P'` であり、`Push(q)` の成分は互いに相異なる。
  `<1>2b` と同じ整礎な関係についての帰納法による。空の列と長さ 1 の列には異なる 2 つの位置が無い。
  最後の場合、`<1>2c` より連ねられる `Push(q ++ [i])` の `i` は互いに相異なるので、異なる 2 つの位置に
  ある成分は、同じ `i` の `Push(q ++ [i])` の異なる 2 つの位置から来たか、相異なる `i ≠ i'` から来たか
  のどちらかである。前者は帰納法の仮定であり、後者については、`<1>2b` より一方は位置 `|q|` に `i` を、
  他方は `i'` を持つので、どちらも他方の前置ではない。
  BY <1>2a, <1>2aa, <1>2b, <1>2c

<1>3a. `boxed_leaf_paths(τ, type_env)` が返す列は `Push([])` である。
  `boxed_leaf_paths` は空の `path` と空の `out` で `go` を 1 度呼び、`out` を返す。
  BY CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths

<1>4. QED
  `<1>3` と `<1>3a` より `boxed_leaf_paths(τ, type_env)` の成分は互いに相異なり、どの 2 つも一方が他方の
  前置ではない。`τ = ty(v)` と置くと、`π` が `boxed_leaf_paths(ty(v), type_env)` の元であるとき、その中で
  `π` を前置に持つのは `π` 自身だけである。DEF `L` は `L(v, π)` を
  `boxed_leaf_paths(ty(v), type_env)` の元のうち `π` を前置に持つものの集合と定めるので、
  `L(v, π) = {π}` である。
  BY DEF `L`, <1>3, <1>3a

## L3a (`Std::Array` と `Std::#FunPtr{n}` の `TyConInfo`) <!--#f68ae1c-->

**言明**。`type_env` をプログラムの `TypeEnv` とする。次の 3 つが成り立つ。

- **(a)** 型 `τ` の最上位の tycon が `type_env.tycons()` の鍵であるとき、`is_funptr(τ)` は panic せずに
  真か偽を返す。
- **(b)** `is_array(τ)` が真であり `toplevel_tycon_info(τ, type_env)` が値を返すならば、その値の
  `variant` は `TyConVariant::Array` である。
- **(c)** `is_funptr(τ)` が真であり `toplevel_tycon_info(τ, type_env)` が値を返すならば、その値の
  `variant` は `TyConVariant::Primitive` である。

**主語を鍵ではなく型に取るのは、読む 3 段が持っているのが型だからである。** その 3 段は
`is_struct` か `is_union` から `toplevel_tycon_info` の値を得ており、`is_array` と `is_funptr` の
真偽をその `variant` と突き合わせる。**この命題が要るのは、`is_array` と `is_funptr` が tycon の
名前で決まるのに対し、`is_struct` と `is_union` が `type_env` の項の `variant` で決まるからである。**
2 つを突き合わせる段は L4 の `<1>9` `<3>2`、L4 の `<1>12` `<3>2`、L5a の `<1>1` である。

**funptr の側を `bulitin_tycons` が入れる範囲でなく `is_funptr` の真偽で書くのは、`is_funptr` が
その範囲の外の `#FunPtr{n}` にも真を返すからである。** 範囲で書くと、読む 3 段が要る
「`is_funptr(τ)` が真ならば `variant` は `Primitive`」が範囲の外で覆われない。

<1>1. `TypeNode::is_array` は、最上位の tycon が在ってそれが `is_array_tycon` を満たすこと --
      すなわちその tycon が `make_array_tycon()`、名前空間が `STD_NAME` (`Std`) ただ 1 つで名前が
      `ARRAY_NAME` (`Array`) である `TyCon` に等しいこと -- であり、最上位の tycon が無ければ偽である。
      `is_funptr_tycon` は、渡された tycon の名前空間が `STD_NAME` ただ 1 つでないか、名前が
      `FUNPTR_NAME` (`#FunPtr`) で始まらないときは `None` を返し、始まるときは名前の残りを `u32` として
      parse し、成功すればその値を包んで返す。**残りが `u32` として読めなければ `unwrap` が panic する**
      -- `is_funptr_tycon` は全域関数ではない。`TypeNode::is_funptr` は、最上位の tycon が在れば
      その tycon についてのこの呼び出しの `is_some()` であり、無ければ偽である。
      `TypeNode::toplevel_tycon_info(type_env)` が返すのは `type_env.tycons()` のその tycon の項であり、
      その鍵が無ければ (または最上位の tycon がクロージャならば) panic する。
  BY CODE src/ast/types.rs: TypeNode::is_array, TypeNode::is_funptr,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_satisfies,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon, TypeNode::toplevel_tycon_info,
     CODE src/fixstd/builtin.rs: make_array_tycon, make_array_name, is_array_tycon, is_funptr_tycon,
     CODE src/constants.rs: STD_NAME, ARRAY_NAME, FUNPTR_NAME

<1>2. `bulitin_tycons()` が返す写像は、鍵 `make_array_tycon()` に `variant` が `TyConVariant::Array` の
      `TyConInfo` を、`1` から `FUNPTR_ARGS_MAX` までの各 `n` について鍵 `make_funptr_tycon(n)` に
      `variant` が `TyConVariant::Primitive` の `TyConInfo` を持つ。`make_funptr_tycon(n)` の名前空間は
      `STD_NAME` ただ 1 つであり、その名前は `FUNPTR_NAME` の後ろに `n` の 10 進表記
      (`u32::to_string` の値) を続けたものである。
  BY CODE src/fixstd/builtin.rs: bulitin_tycons, make_array_tycon, make_funptr_tycon, make_funptr_name,
     CODE src/constants.rs: STD_NAME, FUNPTR_ARGS_MAX, FUNPTR_NAME

<1>3. `type_env.tycons()` の項のうち、鍵が `bulitin_tycons()` の置く鍵のいずれかであるものは、
      `bulitin_tycons()` がその鍵の下に置いた項である。とくに鍵 `make_array_tycon()` の項がそうであり、
      名前空間が `STD_NAME` ただ 1 つで名前が `FUNPTR_NAME` で始まる鍵は `make_funptr_tycon(n)`
      (`n` は 1 以上 `FUNPTR_ARGS_MAX` 以下) であって、その項もそうである。
  BY <ref id=3d4be43/>

<1>4. (a) が成り立つ。
  `τ` の最上位の tycon を `tc` とし、`tc` が `type_env.tycons()` の鍵であるとする。`<1>1` より
  `is_funptr(τ)` は `is_funptr_tycon(tc)` の `is_some()` であり、`is_funptr_tycon` が panic しうるのは
  `tc` の名前空間が `STD_NAME` ただ 1 つであって名前が `FUNPTR_NAME` で始まる場合だけである。その場合、
  `<1>3` より `tc` は `1` 以上 `FUNPTR_ARGS_MAX` 以下のある `n` についての `make_funptr_tycon(n)` で
  あり、`<1>2` よりその名前は `FUNPTR_NAME` の後ろに `n` の 10 進表記を続けたものなので、
  EXT `u32` の 10 進表記 より `FUNPTR_NAME` の後ろの残りは `u32` として読める。よって parse は成功し、
  `unwrap` は panic しない。
  BY EXT `u32` の 10 進表記, <1>1, <1>2, <1>3

<1>5. (b) が成り立つ。
  `is_array(τ)` が真であるとする。`<1>1` より `τ` の最上位の tycon は `make_array_tycon()` である。
  `toplevel_tycon_info(τ, type_env)` が値を返すならば、`<1>1` よりその値は `type_env.tycons()` の
  鍵 `make_array_tycon()` の項であり、その鍵が写像に在る。`<1>3` よりその項は `bulitin_tycons()` が
  その鍵の下に置いた項であり、`<1>2` よりその `variant` は `TyConVariant::Array` である。
  BY <1>1, <1>2, <1>3

<1>6. (c) が成り立つ。
  `is_funptr(τ)` が真であるとする。`<1>1` より `τ` の最上位の tycon `tc` が在って `is_funptr_tycon(tc)`
  は `Some` を返すので、`tc` の名前空間は `STD_NAME` ただ 1 つであり、その名前は `FUNPTR_NAME` で
  始まる。`toplevel_tycon_info(τ, type_env)` が値を返すならば、`<1>1` よりその値は
  `type_env.tycons()` の鍵 `tc` の項であり、その鍵が写像に在る。`<1>3` より `tc` は `1` 以上
  `FUNPTR_ARGS_MAX` 以下のある `n` についての `make_funptr_tycon(n)` であって、その項は
  `bulitin_tycons()` がその鍵の下に置いた項であり、`<1>2` よりその `variant` は
  `TyConVariant::Primitive` である。
  BY <1>1, <1>2, <1>3

<1>7. QED
  BY <1>4, <1>5, <1>6

## L4 (identity の位置) <!--#8253e68-->

**言明**。`ρ` を実行路、`(x, λ)` を `ρ` の位置 (DEF 路の位置) であって、**解析が鍵 `(x, λ)` で `origin` を
呼ぶ**ものとする。`id(x, λ) = (w, σ)` とおくと、次が成り立つ。

- (i) `σ` は `ty(w)` の boxed leaf であり、`(w, σ)` は `ρ` の位置である。
- (ii) `obj(x, λ) = obj(w, σ)`。

言明が `ρ` のスロットではなく `ρ` の位置を渡るのは、`origin` の再帰が記号の位置へ着くからである --
`Let(x, Var(g), k)` (`g` はグローバル値) の `(g, λ)` がそれであり、そこがこの再帰の終端の 1 つである
(D6)。

**前件「解析が鍵 `(x, λ)` で `origin` を呼ぶ」を置くのは、この証明の帰納が立つ整礎性がその範囲でしか
言えないからである** -- L0a (b) が `⇝` の整礎性を与えるのは呼び出しの在る鍵についてであり、`id(x, λ)` の
値そのものも呼び出しの在る鍵についてしか定まらない (L0 (a)、§1)。**前件を落として `ρ` のすべての位置を
渡る形にすると、解析が呼ばない鍵について `id` の値を引くことになる。** `README.md` の P3・P4・P5 (a)・
P5 (b) が同じ前件を持ち、P6 は「**その各 leaf について解析が `origin` を呼ぶ**」を本文で述べる。この
命題を読む 2 人はそこから前件を得る -- P5 (a) は自分の前件から、P6 (b) は P6 (a) の前半から取る。

証明は、DEF 鍵の関係 の `⇝` の上の帰納法による。前提より鍵 `(x, λ)` について `origin` の呼び出しが在るので、
L0a (b) より、この鍵から始まる `⇝` の無限列は無い。
以下、帰納法の仮定を「IH」と書く。**IH を当てる各段は、当てる先の鍵が `(x, λ)` の `⇝` の像に入ることと、
解析がその鍵でも `origin` を呼ぶこととを併せて述べる。** `origin` は memo を持つので、`origin_inner` の腕が
行う `origin` の呼び出しが下位の計算を始めるとは限らず、動的な呼び出しの入れ子ではこの帰納法は立たない。

<1>1. `origin(x, λ)` の値は `origin_inner(vars, type_env, x, λ)` の 1 回の呼び出しが返した値である。
  前提より鍵 `(x, λ)` について `origin` の呼び出しが在るので、L0 (a) が当たる。
  BY <ref id=ecdd35d/> (a)

<1>2. `origin_inner` の腕は、`vars.bindings.get(x)` の値について次の 6 本で尽きている。
      `None | Some(Param) | Some(Producer)`、`Some(Move(y))`、`Some(Join(arm_results))`、
      `Some(Llvm(gen, args, result_ty))`、`Some(Field(c, i))`、`Some(Payload(s, variant))`。
  BY CODE src/rc_ir/ownership.rs: Binding (構成子は `Param`、`Move`、`Llvm`、`Producer`、`Field`、
     `Payload`、`Join` の 7 つ), origin_inner

<1>3. `x` の `Binding` が名指す変数 -- `Move(y)` の `y`、`Field(c, i)` の `c`、`Payload(s, ·)` の `s`、
      `Llvm(gen, args, ·)` の `args` の元、`Join(rs)` の `rs` のうち `ρ` が通ったアームの元 -- は、
      `ρ` の上で値を得ている。
  <2>1. ここで挙げた 5 つの構成子はいずれも `Param` ではないので L1a (b) が使え、`x` を束縛する節点は
        `ρ` の上にあり、その形は `Binding` の構成子で決まる。`collect_bindings` はこれらの変数を
        その節点から取る。`Move` と `Llvm` は `Let(x, rhs, k)` の
        `rhs` から、`Field` は `Destructure` の容器から、`Payload` は `Match` の scrutinee から、
        `Join` の元は各アーム本体の `returned_var` から取る。`returned_var` の本体は `grow_stack` に
        包まれているので、その値を読むのに A15 が要る。
    BY <ref id=3e6b0e0/>, <ref id=cc9c0f1/>, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
  <2>2. 節点に現れる変数の使用は、その節点でスコープに入っている束縛に解決する。
    BY <ref id=3905b4e/>
  <2>3. D2 の「束縛の及ぶ範囲 (スコープ)」より、`Let(x, rhs, k)` と `Destructure(c, fs, s, k)` が束縛する
        変数のスコープは `k` の部分木、`Match` のアームの `payload` のスコープはそのアームの `body` の
        部分木であり、どれも束縛の節点の子孫である。パラメータと capture のスコープは本体の全体である。
        よって節点でスコープに入っている束縛は、その節点の祖先が束縛したものか、パラメータ・capture で
        ある。
    BY <ref id=b3dfa37/>
  <2>4. D3 より実行路は根から辿るので、`ρ` が着いた節点の祖先はすべて `ρ` の上にある。パラメータと
        capture は活性化の入力の束縛 (D23) から値を得ており、本体の最初の節点の時点で既に値を持つ
        (DEF 路の位置 の (S1))。
    BY <ref id=ca36627/>, <ref id=ff5985d/>, <ref id=09fabad/>, DEF 路の位置
  <2>4a. `ρ` の上の節点 `N` が `Let(m, Match(s, arms), k)` のアーム `a` の本体の部分木にあるとき、`ρ` を
         辿る実行はその `Match` で `a` を選ぶ。D3 より実行路は `Match` の節点でアームを 1 つ選び、その
         アーム本体の実行路を辿ってから `k` へ進むので、`ρ` が `a` の本体の節点を含むのは `ρ` がその
         アームを選んだときに限る。
    BY <ref id=ca36627/>
  <2>5. `Join(rs)` の元のうち `ρ` が通ったアームのものは、L1a (b) の `Join` の行より、そのアーム本体の
        終端の `Ret` が名指す変数である。その `Ret` は `ρ` の上の節点なので、`<2>2` から `<2>4a` をその
        位置に適用できる。
    BY <ref id=ca36627/>, <ref id=cc9c0f1/>, <2>1, <2>2, <2>3, <2>4, <2>4a
  <2>6. `ρ` の上の節点に現れる名前が `vars.bindings` に無いとき、それは DEF 路の位置 の (S4) であり、
        その名前は `ρ` の上で値を得ている。
    BY <ref id=09fabad/>, DEF 路の位置
  <2>7. QED
    `u` を `x` の `Binding` が名指す変数の 1 つとする。`<2>1` より `u` は `ρ` の上のある節点 `N` に現れる
    (`Join(rs)` の元については `<2>5` がその `N` を与える)。`<2>2` より `u` のその出現は `N` でスコープに
    入っている束縛に解決する。`u` が `vars.bindings` に無い名前であれば、`u` は `ρ` の上の節点 `N` に
    現れるので `<2>6` の (S4) である。在る場合、`<2>3` よりその束縛はパラメータ・capture か、`N` の
    祖先の節点が作ったものである。前者は
    DEF 路の位置 の (S1) であり、`<2>4` より本体の最初の節点の時点で値を持つ。後者について、D2 の
    束縛節点は `Let`・`Destructure`・`Match` のアームの `payload` の 3 種であり、`<2>4` よりその節点は
    `ρ` の上にある。`Let(u, rhs, k)` と `Destructure(c, fs, s, k)` は (S2) である。`Match` のアームの
    `payload` が `u` である場合、D2 よりその `payload` のスコープはそのアームの本体の部分木なので `N` は
    その部分木にあり、`<2>4a` より `ρ` を辿る実行はそのアームを選ぶ。すなわち (S3) である。DEF 路の
    位置 より、(S1) から (S4) のいずれでも `u` は `ρ` の上で値を得ている。
    BY <ref id=b3dfa37/>, <ref id=09fabad/>, DEF 路の位置, <2>1, <2>2, <2>3, <2>4, <2>4a, <2>5, <2>6

<1>4. `x` の `Binding` が `Llvm(gen, args, result_ty)` であるとき、`result_ty` は `ty(x)` であり、
      `decl := gen.result_prov(result_ty, &arg_tys, type_env)` について `decl.leaf_origins_at(λ)` は
      `Some(S)` である。さらにこの `decl` は、`<1>1` の `origin_inner` の呼び出しが `Llvm` の腕で作る
      `decl` と同じ値である。
  <2>1. `collect_bindings` は `Let(x, Llvm(llvm_gen, args), k)` に `Binding::Llvm(llvm_gen, args, x.ty)` を
        作る。
    BY CODE src/rc_ir/ownership.rs: collect_bindings の `RcExpr::Let` の腕の `RcRhs::Llvm` の場合
  <2>2. `result_prov` は結果の型の boxed leaf ごとに `LeafOrigins` を 1 つ宣言する。すなわち `decl` の
        鍵の全体は `boxed_leaf_paths(result_ty, type_env)` である。
    BY <ref id=e11772a/>
  <2>2a. `<1>1` の `origin_inner` の呼び出しが `Llvm` の腕で作る `decl` は、この言明の `decl` と同じ
         値である。その腕は `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` を評価し、
         `llvm_gen`・`args`・`result_ty` は `vars.bindings.get(x)` の記録 `Llvm(gen, args, result_ty)` が
         持つ値 -- `llvm_gen` は `gen` である -- で、`arg_tys` は `args` の各元の型の列である。引数が
         同じなので、A3 の決定性の節 -- `result_prov` は同じ引数に対して常に同じ値を返す -- が 2 つの
         値を等しくする。
    BY <ref id=e11772a/>, <1>1, CODE src/rc_ir/ownership.rs: origin_inner の
       `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>3. `leaf_origins_at(π)` は、`π` に記録がある場合に `Some`、無い場合に `None` を返す。
    BY CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_at, CODE src/rc_ir/leaf_map.rs: LeafMap::get
  <2>4. QED
    `λ` は `ty(x)` の boxed leaf である (L0b (b))。`<2>1` より `result_ty = ty(x)` なので、
    `<2>2` より `λ` は `decl` の鍵であり、`<2>3` より `leaf_origins_at(λ)` は `Some` を返す。後半は
    `<2>2a` である。
    BY <ref id=09fabad/>, DEF 路の位置, <2>1, <2>2, <2>2a, <2>3

<1>5. `<1>4` の `S` の元数は 0 か 1 である。
  A3 は「複数の元を宣言する op は存在しない」と述べ、`result_prov` を override する 29 個が leaf に置く
  集合はすべて要素数 0 か 1 であるとする。`origin_inner` が読む `decl` は `llvm_gen.result_prov(..)` の
  返り値そのものであって、`Provenance::join` や `compose` を通していない。
  BY <ref id=e11772a/>, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(..))` の腕

<1>5a. `id(x, λ) = (x, λ)` であるとき、(i) と (ii) が成り立つ。
  (i): 前提より `(x, λ)` は `ρ` の位置であり、L0b (b) より `λ` は `ty(x)` の boxed leaf で
  ある。(ii): `obj(x, λ) = obj(x, λ)` は EXT 等号の性質 の反射性である。
  BY EXT 等号の性質, <ref id=09fabad/>, DEF 路の位置

<1>6. CASE `None | Some(Param) | Some(Producer)`。
  <2>1. 答えは `here()` すなわち `Exactly((x, λ))` であり、`id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の
       `None | Some(Binding::Param) | Some(Binding::Producer)` の腕, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>7. CASE `Some(Move(y))`。
  <2>1. 解析は鍵 `(y, λ)` でも `origin` を呼び、`origin(x, λ) = origin(y, λ)` であり、よって
        `id(x, λ) = id(y, λ)` である。また `(x, λ) ⇝ (y, λ)` である。
    この CASE の前提は `vars.bindings.get(x) = Some(Move(y))` であり、L4 の前提より解析は鍵 `(x, λ)` で
    `origin` を呼ぶので、L2 (E1) が呼び出しと等式の両方を与える。`origin_inner` の
    `Some(Binding::Move(y))` の腕は `origin(vars, type_env, &y.name, path)` を直接呼ぶので、
    DEF 鍵の関係 よりこの鍵は `(x, λ)` の `⇝` の像に入る。
    BY DEF 鍵の関係, <ref id=97d41ba/> (E1),
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Move(y))` の腕
  <2>2. `(y, λ)` は `ρ` の位置である。
    A12 の move-bind の行より `ty(y) = ty(x)` なので `λ` は `ty(y)` の boxed leaf である。D9 の値の水準の
    第 1 行より `x` の値は `y` の値であり、D16 の inhabited は値で決まるので `λ` は `y` の値でも
    inhabited である。`y` が `ρ` の上で値を得ていることは `<1>3` である。
    BY <ref id=83d98e9/>, <ref id=9d74736/>, <ref id=66c9670/>, <ref id=09fabad/>, DEF 路の位置, <1>3
  <2>3. `obj(x, λ) = obj(y, λ)`。
    L1a (b) の `Move(y)` の行より、`x` を束縛する節点 `Let(x, Var(y), k)` が `ρ` の上にあり、それが
    定める E1 の辺は `(y, λ)`-`(x, λ)` である。`<2>2` と前提よりその両端は `ρ` の位置なので、その辺は
    在る (DEF 辺の存在)。L1a (b) の同じ行より、それは `ρ` の上で実行された辺である。
    BY <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <2>2
  <2>4. QED
    `<2>1` より `(y, λ)` は `(x, λ)` の `⇝` の像に入り、解析はその鍵でも `origin` を呼び、`<2>2` より
    それは `ρ` の位置なので、IH を当てられる。IH を `(y, λ)` に適用
    すると、`id(y, λ) = (w, σ)` について (i) と `obj(y, λ) = obj(w, σ)` が出る。`<2>1` より
    `id(x, λ) = id(y, λ)` であり、`<2>3` と EXT 等号の性質 の推移性より `obj(x, λ) = obj(w, σ)` である。
    BY EXT 等号の性質, <2>1, <2>2, <2>3, IH

<1>8. CASE `Some(Field(c, i))` で `c` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕の
       `container.ty.is_box(type_env)` が真の枝, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>9. CASE `Some(Field(c, i))` で `c` が unbox。
  <2>1. 解析は鍵 `(c, [i] ++ λ)` でも `origin` を呼び、`origin(x, λ) = origin(c, [i] ++ λ)` であり、
        よって `id(x, λ) = id(c, [i] ++ λ)` である。また `(x, λ) ⇝ (c, [i] ++ λ)` である。
    この CASE の前提は `vars.bindings.get(x) = Some(Field(c, i))` かつ `c.ty.is_box(type_env)` が偽で
    あり、L4 の前提より解析は鍵 `(x, λ)` で `origin` を呼ぶので、L2 (E2) が呼び出しと等式の両方を
    与える。`origin_inner` の `Some(Binding::Field(container, idx))` の腕は
    `origin(vars, type_env, &container.name, &container_path)` を直接呼ぶので、DEF 鍵の関係 より
    この鍵は `(x, λ)` の `⇝` の像に入る。
    BY DEF 鍵の関係, <ref id=97d41ba/> (E2),
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Field(container, idx))` の腕
  <2>2. フィールド `i` は `ty(c)` が実際に持つ (穴でない) フィールドである。
    L1a (b) の `Field(c, i)` の行より、`x` を束縛する節点は `Destructure(c, fs, s, k)` であって
    `(i, x) ∈ fs` である。A12 の「`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が
    実際に持つ (punched でない) ものであること」の行が、`i` が穴でないことを与える。
    BY <ref id=83d98e9/>, <ref id=cc9c0f1/>
  <2>3. `[i] ++ λ` は `ty(c)` の boxed leaf である。
    <3>1. `ty(c)` はクロージャではなく、その tycon の `variant` は `TyConVariant::Struct` である。
      A12 の「`Destructure` の容器が構造体であること」の行が `is_struct(ty(c))` を与え、`is_struct` は
      `toplevel_tycon_info` が返す `TyConInfo` の `variant` が `Struct` であることである。**A12 は、
      型の `variant` を述べる各節ではその型の `is_closure()` が偽であると述べ、`Destructure` の容器が
      構造体であることをその節の 1 つに挙げる。**
      BY <ref id=83d98e9/>, CODE src/ast/types.rs: TypeNode::is_struct, TypeNode::toplevel_tycon_info
    <3>2. `is_array(ty(c))` と `is_funptr(ty(c))` は panic せずに計算でき、どちらも偽である。
      `TyConVariant` は `Primitive`・`Arrow`・`Array`・`Struct`・`Union`・`DynamicObject`・
      `ArrayStorage`・`Opaque` のいずれか 1 つである。`<3>1` は `is_struct(ty(c))` が真であることを
      `toplevel_tycon_info` 経由で示すので、`toplevel_tycon_info` は値を返し、`ty(c)` の最上位の
      tycon は `type_env.tycons()` の鍵である。よって L3a (a) より `is_funptr(ty(c))` は panic せずに
      真か偽のいずれかの値を返す。L3a (b) より `is_array(ty(c))` が真ならばその項の `variant` は
      `Array` であり、L3a (c) より `is_funptr(ty(c))` が真ならばその `variant` は `Primitive` である。
      `<3>1` より `ty(c)` の `variant` は `Struct` なので、どちらも真ではあり得ず、
      `is_array(ty(c))` と `is_funptr(ty(c))` はどちらも偽である。
      BY <ref id=f68ae1c/> (a), <ref id=f68ae1c/> (b), <ref id=f68ae1c/> (c), <3>1,
         CODE src/ast/types.rs: TyConVariant, TypeNode::toplevel_tycon_info
    <3>3. `is_fully_unboxed(ty(c))` は偽である。
      この CASE の前提より `is_box(ty(c))` は偽であり、`<3>1` と `<3>2` より `is_closure`・`is_array`・
      `is_funptr` も偽なので、`is_fully_unboxed(ty(c))` は `unpunched_field_types(ty(c))` の各
      フィールドの型の `is_fully_unboxed` の連言である。A12 の `Destructure` の行より `ty(x)` は `c` の
      フィールド `i` の型であり、`<2>2` よりそのフィールドは穴でないので、`(i, ty(x))` はその連言の
      1 項である。`(x, λ)` は `ρ` の位置なので `λ` は `ty(x)` の boxed leaf であり
      (L0b (b))、`boxed_leaf_paths(ty(x), type_env)` は空でない。D4 の規則 1 より
      `is_fully_unboxed(ty(x))` は偽である。よって連言は偽である。
      BY <ref id=83d98e9/>, <ref id=0594f24/>, <ref id=09fabad/>, DEF 路の位置, <2>2, <3>1, <3>2,
         CODE src/ast/types.rs: TypeNode::is_fully_unboxed
    <3>4. QED
      `<3>3` より D4 の規則 1 は当たらず、`<3>1` より規則 2 (クロージャ) も、`<3>2` より規則 4
      (`is_array`) も当たらず、CASE の前提より規則 3 (`is_box`) も当たらない。よって規則 5 が当たり、
      `ty(c)` の leaf は `unpunched_field_types` が返す各フィールドの添字を、そのフィールドの型の leaf に
      前置したものである。`<2>2` と `<3>3` よりフィールド `i` はその中にあってその型は `ty(x)` であり、
      `λ` は `ty(x)` の boxed leaf である。
      BY <ref id=0594f24/>, <2>2, <3>1, <3>2, <3>3
  <2>4. `[i] ++ λ` は `c` の値で inhabited である。
    D9 の値の水準の第 3 行より `x` の値は `c` の値の第 `i` フィールドなので、`c` の値の位置
    `[i] ++ λ` は `x` の値の位置 `λ` である。前提より `(x, λ)` は `ρ` の位置であり、L0b (b) より
    その `λ` は `x` の値で inhabited である。A12 の
    `Destructure` の容器が構造体であることの行より `[i]` は unbox union の節を通らず、`[i] ++ λ` が通る
    unbox union の節は `λ` が通る節と同じである。
    BY <ref id=83d98e9/>, <ref id=9d74736/>, <ref id=66c9670/>, <ref id=09fabad/>, DEF 路の位置, <2>3
  <2>5. `(c, [i] ++ λ)` は `ρ` の位置である。
    BY <ref id=09fabad/>, DEF 路の位置, <1>3, <2>3, <2>4
  <2>6. `obj(x, λ) = obj(c, [i] ++ λ)`。
    L1a (b) の `Field(c, i)` の行より、`x` を束縛する節点 `Destructure(c, fs, s, k)` が `ρ` の上に
    あり、`c` が unbox のときそれが定める E2 の辺は `(c, [i] ++ λ)`-`(x, λ)` である。`<2>5` と前提より
    その両端は `ρ` の位置なので、その辺は在る (DEF 辺の存在)。L1a (b) の同じ行より、それは `ρ` の上で
    実行された辺である。
    BY <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <2>5
  <2>7. QED
    `<2>1` より `(c, [i] ++ λ)` は `(x, λ)` の `⇝` の像に入り、解析はその鍵でも `origin` を呼び、
    `<2>5` よりそれは `ρ` の位置なので、IH を当てられる。IH を `(c, [i] ++ λ)` に適用すると、
    `id(c, [i] ++ λ) = (w, σ)` について (i) と `obj(c, [i] ++ λ) = obj(w, σ)` が出る。`<2>1` より
    `id(x, λ) = id(c, [i] ++ λ)` であり、`<2>6` と EXT 等号の性質 の推移性より
    `obj(x, λ) = obj(w, σ)` である。
    BY EXT 等号の性質, <2>1, <2>5, <2>6, IH

<1>10. CASE `Some(Payload(s, None))` (catch-all)。
  <2>1. 解析は鍵 `(s, λ)` でも `origin` を呼び、`origin(x, λ) = origin(s, λ)` であり、よって
        `id(x, λ) = id(s, λ)` である。また `(x, λ) ⇝ (s, λ)` である。
    この CASE の前提は `vars.bindings.get(x) = Some(Payload(s, None))` であり、L4 の前提より解析は
    鍵 `(x, λ)` で `origin` を呼ぶので、L2 (E4) が呼び出しと等式の両方を与える。`origin_inner` の
    `Some(Binding::Payload(scrut, variant))` の腕の `None` の場合は
    `origin(vars, type_env, &scrut.name, path)` を直接呼ぶので、DEF 鍵の関係 よりこの鍵は `(x, λ)` の
    `⇝` の像に入る。
    BY DEF 鍵の関係, <ref id=97d41ba/> (E4),
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. `(s, λ)` は `ρ` の位置である。
    A12 の catch-all アームの payload と scrutinee の行より `ty(s) = ty(x)` なので `λ` は `ty(s)` の
    boxed leaf である。D9 の値の水準の第 5 行より、payload 変数 `x` が得る値は scrutinee `s` の値
    そのものである。よって `λ` は `s` の値でも inhabited である (D16)。`s` が値を得ていることは
    `<1>3` である。
    BY <ref id=83d98e9/>, <ref id=9d74736/>, <ref id=66c9670/>, <ref id=09fabad/>, DEF 路の位置, <1>3
  <2>3. `obj(x, λ) = obj(s, λ)`。
    L1a (b) の `Payload(s, None)` の行より、`x` を payload とする catch-all アームを `ρ` が選ぶ
    `Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、それが定める E4 の辺は `(s, λ)`-`(x, λ)` で
    ある。`<2>2` と前提よりその両端は `ρ` の位置なので、その辺は在る (DEF 辺の存在)。L1a (b) の同じ行
    より、それは `ρ` の上で実行された辺である。
    BY <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <2>2
  <2>4. QED
    `<2>1` より `(s, λ)` は `(x, λ)` の `⇝` の像に入り、解析はその鍵でも `origin` を呼び、`<2>2` より
    それは `ρ` の位置なので、IH を当てられる。IH を `(s, λ)` に適用すると、`id(s, λ) = (w, σ)` に
    ついて (i) と `obj(s, λ) = obj(w, σ)` が出る。`<2>1` より `id(x, λ) = id(s, λ)` であり、`<2>3` と
    EXT 等号の性質 の推移性より `obj(x, λ) = obj(w, σ)` である。
    BY EXT 等号の性質, <2>1, <2>2, <2>3, IH

<1>11. CASE `Some(Payload(s, Some(t)))` で `s` が boxed。
  <2>1. 答えは `here()` であり `id(x, λ) = (x, λ)` である。
    BY <1>1, CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕の
       `Some(_)` の枝, Origin::identity
  <2>2. QED
    BY <1>5a, <2>1

<1>12. CASE `Some(Payload(s, Some(t)))` で `s` が unbox。
  <2>1. 解析は鍵 `(s, [t] ++ λ)` でも `origin` を呼び、`origin(x, λ) = origin(s, [t] ++ λ)` であり、
        よって `id(x, λ) = id(s, [t] ++ λ)` である。また `(x, λ) ⇝ (s, [t] ++ λ)` である。
    この CASE の前提は `vars.bindings.get(x) = Some(Payload(s, Some(t)))` かつ `s.ty.is_box(type_env)` が
    偽であり、L4 の前提より解析は鍵 `(x, λ)` で `origin` を呼ぶので、L2 (E3) が呼び出しと等式の両方を
    与える。`origin_inner` の `Some(Binding::Payload(scrut, variant))` の腕の
    `Some(tag) if !scrut.ty.is_box(type_env)` の場合は `origin(vars, type_env, &scrut.name, &scrut_path)`
    を直接呼ぶので、DEF 鍵の関係 よりこの鍵は `(x, λ)` の `⇝` の像に入る。
    BY DEF 鍵の関係, <ref id=97d41ba/> (E3),
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Payload(scrut, variant))` の腕
  <2>2. `[t] ++ λ` は `ty(s)` の boxed leaf である。
    <3>1. `ty(s)` はクロージャではなく、その tycon の `variant` は `TyConVariant::Union` である。
      A12 の「`Match` の scrutinee が union であること」の行が `is_union(ty(s))` を与え、`is_union` は
      `toplevel_tycon_info` が返す `TyConInfo` の `variant` が `Union` であることである。**A12 は、
      型の `variant` を述べる各節ではその型の `is_closure()` が偽であると述べ、`Match` の scrutinee が
      union であることをその節の 1 つに挙げる。**
      BY <ref id=83d98e9/>, CODE src/ast/types.rs: TypeNode::is_union, TypeNode::toplevel_tycon_info
    <3>2. `is_array(ty(s))` と `is_funptr(ty(s))` は panic せずに計算でき、どちらも偽である。
      `TyConVariant` は `Primitive`・`Arrow`・`Array`・`Struct`・`Union`・`DynamicObject`・
      `ArrayStorage`・`Opaque` のいずれか 1 つである。`<3>1` は `is_union(ty(s))` が真であることを
      `toplevel_tycon_info` 経由で示すので、`toplevel_tycon_info` は値を返し、`ty(s)` の最上位の
      tycon は `type_env.tycons()` の鍵である。よって L3a (a) より `is_funptr(ty(s))` は panic せずに
      真か偽のいずれかの値を返す。L3a (b) より `is_array(ty(s))` が真ならばその項の `variant` は
      `Array` であり、L3a (c) より `is_funptr(ty(s))` が真ならばその `variant` は `Primitive` である。
      `<3>1` より `ty(s)` の `variant` は `Union` なので、どちらも真ではあり得ず、
      `is_array(ty(s))` と `is_funptr(ty(s))` はどちらも偽である。
      BY <ref id=f68ae1c/> (a), <ref id=f68ae1c/> (b), <ref id=f68ae1c/> (c), <3>1,
         CODE src/ast/types.rs: TyConVariant, TypeNode::toplevel_tycon_info
    <3>3. `is_fully_unboxed(ty(s))` は偽である。
      この CASE の前提より `is_box(ty(s))` は偽であり、`<3>1` と `<3>2` より `is_closure`・`is_array`・
      `is_funptr` も偽なので、`is_fully_unboxed(ty(s))` は `unpunched_field_types(ty(s))` の各
      フィールドの型の `is_fully_unboxed` の連言である。A12 の payload と変位の行より `ty(x)` は `ty(s)` の
      変位 `t` の型であり、A12 の「`Match` が名指す変位が、その型が実際に持つ (punched でない) もので
      あること」の行よりその変位は穴でないので、`(t, ty(x))` はその連言の 1 項である。`(x, λ)` は `ρ` の
      位置なので `λ` は `ty(x)` の boxed leaf であり (L0b (b))、
      `boxed_leaf_paths(ty(x), type_env)` は空でない。D4 の規則 1 より `is_fully_unboxed(ty(x))` は
      偽である。よって連言は偽である。
      BY <ref id=83d98e9/>, <ref id=0594f24/>, <ref id=09fabad/>, DEF 路の位置, <3>1, <3>2, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
    <3>4. QED
      `<3>3` より D4 の規則 1 は当たらず、`<3>1` より規則 2 (クロージャ) も、`<3>2` より規則 4
      (`is_array`) も当たらず、CASE の前提より規則 3 (`is_box`) も当たらない。よって規則 5 が当たり、
      union の leaf は各変位の payload の leaf に変位番号を前置したものである。`<3>3` より変位 `t` の
      型は `ty(x)` であり、`λ` は `ty(x)` の boxed leaf である。
      BY <ref id=0594f24/>, <3>1, <3>2, <3>3
  <2>3. この位置で `s` の値の実行時のタグは `t` である。
    L1a (b) の `Payload(s, Some(t))` の行より、`Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、
    `ρ` を辿る実行が選ぶアームの `tag` は `Some(t)` である。L1b がその位置での `s` の値の実行時のタグを
    `t` と定める。
    BY <ref id=cc9c0f1/>, <ref id=71210de/>
  <2>4. `[t] ++ λ` は `s` の値で inhabited である。
    D9 の値の水準の第 4 行より `x` の値は `s` の値の活性変位の payload であり、`<2>3` よりその活性変位は
    `t` なので、`s` の値の位置 `[t] ++ λ` は `x` の値の位置 `λ` である。`[t] ++ λ` が通る unbox union の
    節は、`ty(s)` の根の節 (`<2>3` よりタグは `t`) と、`λ` が通る節である。後者について、前提より
    `(x, λ)` は `ρ` の位置であり、L0b (b) よりその `λ` は `x` の値で inhabited である。
    BY <ref id=9d74736/>, <ref id=66c9670/>, <ref id=09fabad/>, DEF 路の位置, <2>2, <2>3
  <2>5. `(s, [t] ++ λ)` は `ρ` の位置である。
    BY <ref id=09fabad/>, DEF 路の位置, <1>3, <2>2, <2>4
  <2>6. `obj(x, λ) = obj(s, [t] ++ λ)`。
    L1a (b) の `Payload(s, Some(t))` の行より、`x` を payload とする `tag = Some(t)` のアームを `ρ` が
    選ぶ `Let(m, Match(s, arms), k)` の節点が `ρ` の上にあり、`s` が unbox のときそれが定める E3 の辺は
    `(s, [t] ++ λ)`-`(x, λ)` である。`<2>5` と前提よりその両端は `ρ` の位置なので、その辺は在る
    (DEF 辺の存在)。L1a (b) の同じ行より、それは `ρ` の上で実行された辺である。
    BY <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <2>5
  <2>7. QED
    `<2>1` より `(s, [t] ++ λ)` は `(x, λ)` の `⇝` の像に入り、解析はその鍵でも `origin` を呼び、
    `<2>5` よりそれは `ρ` の位置なので、IH を当てられる。IH を `(s, [t] ++ λ)` に適用すると、
    `id(s, [t] ++ λ) = (w, σ)` について (i) と `obj(s, [t] ++ λ) = obj(w, σ)` が出る。`<2>1` より
    `id(x, λ) = id(s, [t] ++ λ)` であり、`<2>6` と EXT 等号の性質 の推移性より
    `obj(x, λ) = obj(w, σ)` である。
    BY EXT 等号の性質, <2>1, <2>5, <2>6, IH

<1>13. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が単一の `Arg(j, σ')`。
  <2>1. 解析は鍵 `(args[j], σ')` でも `origin` を呼び、`origin(x, λ) = origin(args[j], σ')` であり、
        よって `id(x, λ) = id(args[j], σ')` である。また `(x, λ) ⇝ (args[j], σ')` である。
    この CASE の前提は `vars.bindings.get(x) = Some(Llvm(gen, args, result_ty))` かつ
    `decl.leaf_origins_at(λ) = Some({Arg(j, σ')})` である (`<1>4`)。L4 の前提より解析は鍵 `(x, λ)` で
    `origin` を呼ぶので、L2 (E5) が呼び出しと等式の両方を与える。`<1>4` より、`<1>1` の呼び出しの腕が
    作る `decl` はこの `decl` と同じ値なので、その腕の `leaf_origins_at(λ)` も同じ答えを返す。
    `origin_inner` の
    `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕は `origin(vars, type_env, &args[j].name, &p)` を
    直接呼ぶので、DEF 鍵の関係 よりこの鍵は `(x, λ)` の `⇝` の像に入る。
    BY DEF 鍵の関係, <ref id=97d41ba/> (E5), <1>4,
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Llvm(llvm_gen, args, result_ty))` の腕
  <2>2. `σ'` は `ty(args[j])` の boxed leaf であり、`args[j]` の値で inhabited である。
    A3 の「単一の `Arg(j, σ)`」の行は、宣言が第 `j` オペランドの leaf `σ` を名指すこと、および結果の
    その leaf が inhabited であることと第 `j` オペランドの leaf `σ` が inhabited であることが同値である
    ことを述べる。L4 の前提より `(x, λ)` は `ρ` の位置であり、L0b (b) より `λ` は inhabited である。
    BY <ref id=e11772a/>, <ref id=09fabad/>, DEF 路の位置
  <2>3. `(args[j], σ')` は `ρ` の位置である。
    BY <ref id=09fabad/>, DEF 路の位置, <1>3, <2>2
  <2>4. `obj(x, λ) = obj(args[j], σ')`。
    L1a (b) の `Llvm(gen, args, ty)` の行より、`x` を束縛する節点 `Let(x, Llvm(gen, args), k)` が
    `ρ` の上にあり、宣言が単一の `Arg(j, σ')` であるときそれが定める E5 の辺は
    `(args[j], σ')`-`(x, λ)` である。`<2>3` と前提よりその両端は `ρ` の位置なので、その辺は在る
    (DEF 辺の存在)。L1a (b) の同じ行より、それは `ρ` の上で実行された辺である。
    BY <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <2>3
  <2>5. QED
    `<2>1` より `(args[j], σ')` は `(x, λ)` の `⇝` の像に入り、解析はその鍵でも `origin` を呼び、
    `<2>3` よりそれは `ρ` の位置なので、IH を当てられる。IH を `(args[j], σ')` に適用すると、
    `id(args[j], σ') = (w, σ)` について (i) と `obj(args[j], σ') = obj(w, σ)` が出る。`<2>1` より
    `id(x, λ) = id(args[j], σ')` であり、`<2>4` と EXT 等号の性質 の推移性より
    `obj(x, λ) = obj(w, σ)` である。
    BY EXT 等号の性質, <2>1, <2>3, <2>4, IH

<1>14. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が空集合。
  A3 の空集合の行より、結果のその leaf は inhabited にならない。前提より `λ` は inhabited なので、この
  場合は起きない。
  BY <ref id=e11772a/>, <ref id=09fabad/>, DEF 路の位置

<1>15. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` が単一の `Fresh` または単一の `Unknown`。
  <2>1. `as_arg_projection(S)` は `None` を返すので、腕は
        `origin_from_leaves_under(vars, type_env, &decl, args, λ, &here_identity)` の値を
        `unwrap_or_else(here)` に渡し、その結果を返す。`here_identity` は `(x, λ)` であり、
        `here()` は `Exactly((x, λ))` である。`<1>4` より、その腕が作る `decl` はこの CASE が主語にする
        `decl` と同じ値である。
    BY <1>1, <1>4, CODE src/rc_ir/ownership.rs: as_arg_projection, origin_inner の
       `Some(Binding::Llvm(..))` の腕の `None =>` の枝
  <2>2. `decl.leaf_origins_under(λ)` が返すのは `S` 1 つだけである。
    `leaves_under(path)` は写像の鍵のうち `path` を前置に持つものの値を返す。`<1>4` より `decl` の鍵の
    全体は `boxed_leaf_paths(ty(x), type_env)` であり、`λ` はその 1 つで、その値は `S` である。`ty(x)` は
    プログラムに現れる型なので A10 を満たし、L3 より `ty(x)` の boxed leaf のうち `λ` を前置に持つものは
    `λ` 自身だけである。
    BY <ref id=8412761/>, <ref id=742afee/>, <1>4, CODE src/rc_ir/provenance.rs: Provenance::leaf_origins_under,
       CODE src/rc_ir/leaf_map.rs: LeafMap::leaves_under
  <2>3. `origin_from_leaves_under` は `Some(Origin::Exactly((x, λ)))` を返す。
    `S` の元は `Fresh` か `Unknown` なので、ループは `operand_units` に何も入れず `produced_here` を
    立てる。よって `reached` は `Exactly((x, λ))` 1 つだけからなり、`reached.iter().all(..)` の枝が
    その値を返す。
    BY <2>1, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. QED
    `<2>3` より `origin_from_leaves_under` は `Some(Origin::Exactly((x, λ)))` を返すので、`<2>1` の
    `unwrap_or_else(here)` はその中身をそのまま返し、腕の値は `Exactly((x, λ))` である。`<1>1` より
    `origin(x, λ)` の値はその `origin_inner` の呼び出しが返した値なので、`Origin::identity` より
    `id(x, λ) = (x, λ)` である。
    BY <1>1, <1>5a, <2>1, <2>3, CODE src/rc_ir/ownership.rs: Origin::identity

<1>16. CASE `Some(Llvm(gen, args, result_ty))` で `<1>4` の `S` の元数が 2 以上。
  `<1>5` よりこの場合は起きない。
  BY <1>5

<1>17. CASE `Some(Join(arm_results))`。
  <2>1. 答えは `Origin::of_candidates(C, (x, λ))` である。ここで `C` は各 `r ∈ arm_results` についての
        `act(r, λ)` の合併である。また各 `r ∈ arm_results` について `(x, λ) ⇝ (r, λ)` であり、解析は
        その各鍵でも `origin` を呼ぶ。
    `origin_inner` の `Some(Binding::Join(arm_results))` の腕は、各 `r` について
    `origin(vars, type_env, &r.name, path)` を直接呼び、その `acted_on()` を集める。DEF 鍵の関係 より
    その鍵は `(x, λ)` の `⇝` の像に入る。L4 の前提より解析は鍵 `(x, λ)` で `origin` を呼ぶので、
    L0a (a) より各 `(r, λ)` でも呼ぶ。
    BY DEF 鍵の関係, <ref id=cabeb3c/> (a), <1>1,
       CODE src/rc_ir/ownership.rs: origin_inner の `Some(Binding::Join(arm_results))` の腕
  <2>2. `r_0` を `ρ` が通ったアームの結果とすると、`(r_0, λ)` は `ρ` の位置であり
        `obj(x, λ) = obj(r_0, λ)` である。
    A12 のアームの結果と `Match` の束縛変数の行より両者の型は一致するので `λ` は `ty(r_0)` の boxed leaf
    である。D9 の値の水準の第 2 行より `x` が得る値は `r_0` の値なので、`λ` は `r_0` の値でも inhabited
    である (D16)。`r_0` が値を得ていることは `<1>3` である。L1a (b) の `Join(rs)` の行より、`ρ` が選ぶ
    アーム `a` について `r_0 ∈ rs` であり、`a` の本体の終端の `Ret(r_0)` が定める E6 の辺は
    `(r_0, λ)`-`(x, λ)` である。その両端は `ρ` の位置なので、その辺は在る (DEF 辺の存在)。L1a (b) の
    同じ行よりそれは `ρ` の上で実行された辺なので、L1 (a) よりオブジェクトが一致する。
    BY <ref id=83d98e9/>, <ref id=9d74736/>, <ref id=66c9670/>, <ref id=4c886c1/> (a), <ref id=cc9c0f1/>, DEF 辺の存在, <ref id=09fabad/>, DEF 路の位置, <1>3
  <2>2a. 解析が鍵 `(u, μ)` で `origin` を呼ぶとき、`act(u, μ)` は `id(u, μ)` を含み、したがって空でない。
    `Origin::acted_on()` は `identity()` を先頭に置く列である。
    BY <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: Origin::acted_on
  <2>3. `C` は空でない。
    A9 よりアームは 1 つ以上あり、`<2>1` より解析は各 `r ∈ arm_results` について鍵 `(r, λ)` で `origin` を
    呼ぶので、`<2>2a` より各 `act(r, λ)` は空でない。
    BY <ref id=1172c08/>, <2>1, <2>2a
  <2>4. CASE `|C| ≥ 2`。
    <3>1. `of_candidates(C, (x, λ))` は `Join { identity: (x, λ), candidates: C }` を返すので
          `id(x, λ) = (x, λ)` である。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity
    <3>2. QED
      BY <1>5a, <3>1
  <2>5. CASE `|C| = 1`。
    <3>1. `C = {c}` とおくと `of_candidates(C, (x, λ))` は `Exactly(c)` を返すので `id(x, λ) = c` で
          ある。
      BY <2>1, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity
    <3>2. `id(r_0, λ) = c` である。
      L1a (b) の `Join(rs)` の行より `r_0 ∈ arm_results` なので、`<2>1` より `act(r_0, λ) ⊆ C = {c}` で
      ある。`<2>2a` より `act(r_0, λ)` は空でなく `id(r_0, λ)` を含む。よって `act(r_0, λ) = {c}` であり、
      `id(r_0, λ) = c` である。
      BY <ref id=cc9c0f1/>, <2>1, <2>2a
    <3>3. QED
      L1a (b) の `Join(rs)` の行より `r_0 ∈ arm_results` なので、`<2>1` より `(r_0, λ)` は `(x, λ)` の
      `⇝` の像に入り、解析はその鍵でも `origin` を呼び、`<2>2` よりそれは `ρ` の位置なので、IH を
      当てられる。IH を `(r_0, λ)` に適用すると、`<3>2` より `id(r_0, λ) = c` なので (i) は `c` に
      ついて成り立ち、`obj(r_0, λ) = obj(c)` が出る。`<2>2` と EXT 等号の性質 の推移性より
      `obj(x, λ) = obj(c)` であり、`<3>1` より `id(x, λ) = c` である。
      BY EXT 等号の性質, <ref id=cc9c0f1/>, <2>1, <2>2, <3>1, <3>2, IH
  <2>6. QED
    `<2>3` より `|C| ≥ 1` であり、`<2>4` と `<2>5` がその 2 つの場合を尽くす。
    BY <2>3, <2>4, <2>5

<1>18. QED
  `<1>2` の 6 本の腕のうち、`Field` は容器の boxed / unbox で `<1>8` と `<1>9` に、`Payload` は
  `variant` が `None` か `Some` か、`Some` のときの scrutinee の boxed / unbox で `<1>10`、`<1>11`、
  `<1>12` に分かれる。`Llvm` は `<1>4` の `S` の形で `<1>13` から `<1>16` に分かれる -- `<1>4` が
  `decl.leaf_origins_at(λ)` が `Some(S)` であることを与え、`S` は `LeafOrigin` の集合であり、`LeafOrigin` の
  構成子は `Fresh`、`Unknown`、`Arg(j, σ)` の 3 つなので、`<1>5` が「元数 2 以上」を消した後に残るのは、
  空集合、単一の `Arg`、単一の `Fresh`、単一の `Unknown` の 4 つである。残りは `<1>6`、`<1>7`、`<1>17` で
  ある。
  BY <1>1, <1>2, <1>4, <1>5, <1>6, <1>7, <1>8, <1>9, <1>10, <1>11, <1>12, <1>13, <1>14, <1>15, <1>16,
     <1>17, CODE src/rc_ir/provenance.rs: LeafOrigin

**補足 (切り詰めを通る枝に入らないこと)**。`origin_from_leaves_under` が `truncate_to_unit` を呼ぶのは
`LeafOrigin::Arg(j, leaf)` の元を `operand_units` に入れるときである
(`CODE src/rc_ir/ownership.rs: origin_from_leaves_under`)。`<1>13` から `<1>16` より、`λ` が boxed leaf で
あるときにこの関数へ入るのは `S` が空集合か単一の `Fresh` / `Unknown` のときだけである。`ty(x)` は
プログラムに現れる型なので A10 を満たし、L3 より `ty(x)` の
boxed leaf のうち `λ` を前置に持つものは `λ` 自身だけなので、`decl.leaf_origins_under(λ)` が返す宣言は `S`
だけであり、どの場合も `Arg` の元を持たない。よって leaf の path から出た再帰は切り詰めを 1 度も通らない。

## P5 (a) -- 対の健全性

**言明** (README の P5 (a))。**1 つの本体 (D23)** の 1 回の活性化について、**解析がその 2 つの鍵で
`origin` を呼び**、1 つの実行路の 1 つの位置において `origin` の `identity` が等しい 2 つの leaf の
スロットは、同じオブジェクトを指す。

<1>0. この文書が固定する本体は、ある関数の `body` かあるグローバル初期化子の `init` かのどちらかであり
      (§1)、D23 の「本体」はその 2 つで尽きる。以下の 2 段はその固定の下で述べられているので、言明の
      主語である本体を尽くす。
  BY <ref id=ff5985d/>

<1>1. 位置 `p` (D23) の 2 つのスロットを `(x, λ)`、`(y, μ)` とし、`id(x, λ) = id(y, μ) = (w, σ)` とする。
      この 2 つはこの実行路 `ρ` のスロットであり、したがって `ρ` の位置 (D6) である。
  D6 は「実行路 `ρ` の上のスロット (`ρ` の位置) とは、`ρ` を辿るある時点でスロットである対のことである」と
  定めるので、位置 `p` のスロットは `ρ` のスロットであり、`ρ` の位置である。
  BY <ref id=596a46d/>

<1>2. `obj(x, λ) = obj(w, σ)` であり、`obj(y, μ) = obj(w, σ)` である。
  `<1>1` より `(x, λ)` と `(y, μ)` は `ρ` の位置であり、前提より解析はその 2 つの鍵で `origin` を呼ぶので、
  L4 が両方に当たる。L4 の (ii) を `(x, λ)` に当てると `obj(x, λ) = obj(id(x, λ))`、`(y, μ)` に当てると
  `obj(y, μ) = obj(id(y, μ))` であり、前提より `id(x, λ) = id(y, μ) = (w, σ)` である。
  BY <ref id=8253e68/>, <1>1

<1>3. QED
  `<1>2` の 2 つの等式に EXT 等号の性質 の対称性と推移性を当てると `obj(x, λ) = obj(y, μ)` である。
  `<1>0` よりこの結論は言明の主語である本体を尽くす。
  BY EXT 等号の性質, <1>0, <1>2

**補足 (この主張がどこで危ういか)**。`identity` が 2 つの leaf の間で潰れるのは、`origin` の再帰が
`origin_from_leaves_under` の切り詰めを通るときである。`Std::Option (a, b)` のように 1 つの unit
(unbox union) の下に 2 つの boxed leaf を持つ値では、その 2 つの leaf が別々のオブジェクトに届きながら、
切り詰めた先が同じ unit path になりうる。`L4 の補足` が、位置の path が boxed leaf である限りこの枝に
入らないことを述べる。この結論は A3 の「複数の元を宣言する op は存在しない」に載っている。第 9 節に
書き出す。

## P5 (b) -- 対の有効性

**言明** (README の P5 (b))。**1 つの本体 (D23)** の 1 回の活性化について、
**解析がその 2 つの鍵で `origin` を呼び**、同じオブジェクトを指す 2 つの leaf のスロットで、一方から
他方への別名の道が `Match` のアーム本体の `Ret` の辺を含まないならば、両者の `identity` は等しい。

**前件が要るのは P5 (a) と同じ理由による** -- `identity` は解析が呼ぶ鍵についてしか定まらない
(L0 (a)、§1)。

<1>0. この文書が固定する本体は、ある関数の `body` かあるグローバル初期化子の `init` かのどちらかであり
      (§1)、D23 の「本体」はその 2 つで尽きる。以下の各段はその固定の下で述べられているので、言明の
      主語である本体を尽くす。
  BY <ref id=ff5985d/>

<1>1. 言明が前提に置く道 (E6 の辺を含まない別名の道) のうち、辺の本数が最小のものを 1 つ取る。その各辺は
      E1 から E5 のいずれかである。
  `Match` のアーム本体の `Ret` の辺は E6 であり、D20 の別名の辺は E1 から E6 である。前提の道は有限個の
  辺を並べたものなので、辺の本数は自然数であり、最小のものが在る。取り替えた道の両端は言明の 2 つの
  スロットのままである。
  BY <ref id=9c7c27a/>

<1>1a. 道の各辺について、その辺の終点の変数の `vars.bindings` の記録は、L2 (B) が挙げる構成子である。
  `<1>1` より各辺は E1 から E5 のいずれかであり、L2 (B) がその 5 種について記録を与える。
  BY <ref id=97d41ba/> (B), <1>1

<1>1b. 1 つの位置 `(x, λ)` を終点とする E1 から E5 の辺は高々 1 つである。
  L2 (B) より、E1 から E5 の辺の終点の変数の `vars.bindings` の記録は、辺の種に応じて `Move(y)`、
  `Field(c, i)`、`Payload(s, Some(t))`、`Payload(s, None)`、`Llvm(gen, args, ty(x))` のいずれかで
  ある。`vars.bindings` は名前から `Binding` への写像なので `x` の記録は高々 1 つであり、辺の種は
  それで決まる -- `Payload` の 2 種は記録の `variant` が `Some` か `None` かで分かれる。始点はその記録と
  leaf `λ` で決まる -- `Move(y)` は `(y, λ)`、
  `Field(c, i)` は `(c, [i] ++ λ)`、`Payload(s, Some(t))` は `(s, [t] ++ λ)`、`Payload(s, None)` は
  `(s, λ)`、`Llvm(gen, args, ty)` は `decl.leaf_origins_at(λ)` が単一の `Arg(j, σ)` であるときの
  `(args[j], σ)` であり、その `decl` は記録が持つ op・引数・結果の型と `type_env` だけから決まる
  (A3 の決定性の節)。始点の path を記録と `λ` から読む一覧は、第 2 節の E1 から E5 の定義そのもので
  ある (DEF 辺の leaf 対応 がその対応を書き下す)。
  BY <ref id=e11772a/>, <ref id=9c7c27a/>, <ref id=97d41ba/> (B), DEF 辺の leaf 対応, CODE src/rc_ir/ownership.rs: VarTable

<1>1c. 道の上のどの位置についても、解析はその鍵で `origin` を呼ぶ。
  道を `v_0, …, v_n` とし、`v_0` と `v_n` を言明の 2 つのスロットとする。`v_i` と `v_{i+1}` を結ぶ辺を、
  `v_i` が終点であるとき**上り**、`v_{i+1}` が終点であるとき**下り**と呼ぶ。下りの直後に上りは来ない --
  来るとすると `v_{i+1}` が 2 つの辺の終点になり、`<1>1b` よりその 2 つは同じ辺なので `v_i = v_{i+2}` と
  なる。その 2 歩を除いた列は前提を満たすより短い道であり、`<1>1` の最小性に反する。よって道は上りの
  区間と下りの区間がこの順に並ぶ。L2 の (E1) から (E5) は、解析が辺の終点の鍵で `origin` を呼ぶならば
  始点の鍵でも呼ぶことを与えるので、前提の `v_0` の呼び出しから上りの区間の各位置へ、`v_n` の呼び出しから
  下りの区間を逆に辿って各位置へ、呼び出しが伝わる。L2 の (E2)・(E3)・(E5) が置く「容器が unbox」
  「scrutinee が unbox」「宣言が単一の `Arg(j, σ)`」は、第 2 節の E2・E3・E5 の定義に含まれている。
  BY <ref id=9c7c27a/>, <ref id=97d41ba/> (E1), <ref id=97d41ba/> (E2), <ref id=97d41ba/> (E3), <ref id=97d41ba/> (E4), <ref id=97d41ba/> (E5), <1>1, <1>1a, <1>1b

<1>2. 道の各辺の両端の位置は同じ `origin` を持つ。
  `<1>1c` より辺の終点の鍵で解析は `origin` を呼ぶので、`<1>1a` の記録に L2 の (E1) から (E5) を
  当てられる。E2 と E3 が要る「容器が unbox」「scrutinee が unbox」は
  第 2 節の E2・E3 の定義に含まれており、E5 が要る「宣言が単一の `Arg(j, σ)`」も E5 の定義に含まれて
  いる。
  BY <ref id=97d41ba/> (E1), <ref id=97d41ba/> (E2), <ref id=97d41ba/> (E3), <ref id=97d41ba/> (E4), <ref id=97d41ba/> (E5), <1>1, <1>1a, <1>1c

<1>3. 道の両端の位置は同じ `origin` を持つ。
  道は有限個の辺を並べたものであり、`<1>2` より各辺の両端は同じ `origin` を持つ。EXT 等号の性質 の
  推移性を辺の本数について繰り返すと、両端が同じ `origin` を持つ。
  BY EXT 等号の性質, <1>2

<1>4. QED
  `Origin::identity` は `Origin` の関数なので、等しい `origin` は等しい `identity` を持つ。`<1>1` より
  取り替えた道の両端は言明の 2 つのスロットなので、この結論はその 2 つについてのものである。`<1>0` より
  この結論は言明の主語である本体を尽くす。
  BY <1>0, <1>1, <1>3, CODE src/rc_ir/ownership.rs: Origin::identity

**補足 (前提のうち使っていないもの)**。この証明は前提の「同じオブジェクトを指す」を使わない。E1 から E5 の
別名の道で結ばれていれば、オブジェクトが同じかどうかを問わず `identity` は等しい。

## R1 (E6 を除く限定が外せないこと)

**言明**。同じオブジェクトを指す 2 つのスロット (D6) で、**解析がその 2 つの鍵で `origin` を呼び**、
`identity` が異なるものがある。さらに、その 2 つは
別名の道 (D20) で結ばれており、結ぶ道はすべて E6 の辺を含む。よって P5 (b) の「別名の道が `Match` の
アーム本体の `Ret` の辺を含まない」という限定は外せない。

**解析の呼び出しを言明に入れるのは、P5 (b) がそれを前件に持つからである。** 前件を満たさない組は、
限定を外した形の反例にならない。

**両端がスロットであることが要点である。** P5 (b) が量化するのはスロットなので、記号の位置 (D6) を端に
持つ組はこの限定の反例にならない。下の本体はアームの中で値を作り、その変数を返す。

<1>1. 次の関数 `f` を考える。op と型は、このコンパイラが実在に持つものを取る。

      - `gen`: `InlineLLVMStringBuf`。**この op はこのコンパイラの `impl LLVMGen for` の 1 つである**
        (A3 がその全体を 78 個と数え上げている)。`free_vars_mut` は空の列を返す。`result_prov` は
        `Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)` を返すので、結果の型の各 boxed leaf
        に**単一の `Fresh`** を宣言する。
      - `T`: `Array U8`。`make_string_lit` が、この op を持つ `Llvm` 節点に
        `type_tyapp(make_array_ty(), make_u8_ty())` を結果の型として与える。
      - `Bool`: `Std::Bool`。`unbox union { _false : (), _true : () }` であり、2 つの変位の payload の
        型はどちらも `()` である。

      A3 の表の「単一の `Fresh`」の行より、`gen` が結果のその leaf に置くのは、新しく割り当てた
      オブジェクトへの新しい参照である。**A3 が同じ節に置く但し書き -- 実行時に参照カウントで分岐する
      op の `Fresh` の行は、オブジェクトの同一性については字義どおりではない -- は `gen` に当たらない。**
      A3 はその但し書きで、そうした op の一意の腕が**オペランドの**オブジェクトをそのまま返すと述べる。
      `gen` の `free_vars_mut` は空の列を返し、`LLVMGen::free_vars` はその複製の `free_vars_mut` を
      写した列なので `gen.free_vars()` も空である。A12 の `Llvm` 節点の型についての第 1 の節より
      `Let(x, Llvm(gen, args), k)` の `args` の名前の列は `gen.free_vars()` に等しいので、`gen` を持つ
      `Llvm` 節点はオペランドを 1 つも持たない。返せるオペランドのオブジェクトが無いので、`gen` は
      A3 の但し書きが述べる op ではない。

      `f` のパラメータは `c : Bool` の 1 つ、capture は無く、`borrowed_units` は空 (A1) である。本体は
      次のとおりで、`m`・`x_0`・`x_1` は型 `T`、`p_0`・`p_1` は `()` である。

      ```
      Let(m, Match(c, [ arm(tag=0, payload=p_0, body=Let(x_0, Llvm(gen, []), Ret(x_0))),
                        arm(tag=1, payload=p_1, body=Let(x_1, Llvm(gen, []), Ret(x_1))) ]), Ret(m))
      ```

      これは D2 の形の本体であり、次の仮定を満たす。A1 の後半 (`borrowed_units` が空であること)、
      A6 (`c`・`m`・`x_0`・`x_1`・`p_0`・`p_1` は相異なる名前)、A9 (アームは 2 つ)、
      A12 (アームの結果と `Match` の束縛変数の型、payload と変位の型、`Llvm` 節点の `args` の名前の列が
      `gen.free_vars()` -- 空の列 -- に等しいこと)、A16 の (網羅) (2 つのアームが `Bool` の 2 変位を
      尽くす) と (位置) (catch-all アームが無いので空虚に真)。**A10 と A11 も満たす。** `T`・`Bool`・`()`
      はどれも実在の Fix プログラムに現れる型 -- 文字列リテラルの `Array U8`、`Std::Bool`、0 要素の
      タプル -- なので、A10 がそれらについて ground・飽和・tycon が `type_env` にあること・
      `unpunched_field_types` の歩みが有限であることを与える。各変数の使用はその位置でスコープに入って
      いる束縛に解決するので A11 を満たす -- `c` はパラメータ、`m` は `Let(m, ..)` の継続 `Ret(m)` に、
      `x_j` は同じアーム本体の `Ret(x_j)` に現れる。**この 2 つをここで確かめるのは、この反例が
      `origin` の答えと `boxed_leaf_paths` の数え上げの両方を使うからである。** A11 は `origin` の
      停止性 (P2) がその仮定に立つと `README.md` が述べる分であり、A10 は D4 の規則を `T`・`Bool`・`()`
      に当てるのに要る分である。以下、`j` は 0 と 1 を渡り、`ρ_j` は変位 `j`
      のアームを選ぶ実行路を表す。
  BY <ref id=627e117/>, <ref id=e11772a/>, <ref id=33c54dc/>, <ref id=1172c08/>, <ref id=8412761/>, <ref id=3905b4e/>, <ref id=83d98e9/>, <ref id=f769887/>, <ref id=b3dfa37/>,
     CODE src/fixstd/builtin.rs: impl LLVMGen for InlineLLVMStringBuf, make_string_lit,
     CODE src/ast/inline_llvm.rs: LLVMGen::free_vars,
     CODE src/fixstd/std.fix: Bool

<1>2. `boxed_leaf_paths(T, type_env)` は `{[]}` であり、`[]` は `T` の値で inhabited である。`p_0` と
      `p_1` の型の `boxed_leaf_paths` は空であり、`boxed_leaf_paths(Bool, type_env)` も空である。
  <2>1. `is_array(T)` は真、`is_closure(T)` は偽、`is_fully_unboxed(T)` は偽である。
    `<1>1` より `T = Array U8` であり、その tycon は `Std::Array` である。`is_array` は tycon が
    `Std::Array` であること、`is_closure` は tycon が関数型のものであることなので、前者は真、後者は
    偽である。`is_fully_unboxed` は `is_array` が真の型に対して偽を返す。
    BY <1>1, CODE src/ast/types.rs: TypeNode::is_array, TypeNode::is_closure,
       TypeNode::is_fully_unboxed
  <2>2. `boxed_leaf_paths(T, type_env) = {[]}` である。D4 の規則 1 は `<2>1` より、規則 2 も `<2>1`
        (`is_closure(T)` が偽) より当たらない。残る規則 3 (`is_box`) と規則 4 (`is_array`) は、どちらも
        自分自身の位置 1 つを leaf とする。`<2>1` より `is_array(T)` は真なので、規則 3 が当たっても
        規則 4 が当たっても leaf は `[]` 1 つである。
    BY <ref id=0594f24/>, <2>1
  <2>3. `[]` は `T` の値で inhabited である。`[]` は unbox union の節を 1 つも通らない。
    BY <ref id=66c9670/>, <2>2
  <2>4. `p_0` と `p_1` の型 `()` は leaf を持たない。`()` は `tuple_defn(0)` が定める型、すなわち
        フィールドを 1 つも持たない構造体であり、その `is_unbox` は `TUPLE_UNBOX` すなわち真である。
        よって `is_box`・`is_closure`・`is_array`・`is_funptr` がいずれも偽であり、
        `unpunched_field_types` は空の列を返す。よって `is_fully_unboxed` は空の連言として真であり、
        D4 の規則 1 より leaf を持たない。
    BY <ref id=0594f24/>, <1>1, CODE src/ast/types.rs: TypeNode::is_fully_unboxed,
       CODE src/fixstd/builtin.rs: tuple_defn, CODE src/constants.rs: TUPLE_UNBOX
  <2>5. `boxed_leaf_paths(Bool, type_env)` は空である。`<1>1` より `Bool` は unbox union なので
        `is_box` が偽であり、`is_closure`・`is_array`・`is_funptr` も偽なので、`is_fully_unboxed(Bool)` は
        `unpunched_field_types(Bool)` が返す各型の `is_fully_unboxed` の連言である。`<1>1` よりそれは
        2 つの payload の型 `()` についての連言であり、`<2>4` よりどちらも真である。よって
        `is_fully_unboxed(Bool)` は真であり、D4 の規則 1 より `Bool` は leaf を持たない。
    BY <ref id=0594f24/>, <1>1, <2>4, CODE src/ast/types.rs: TypeNode::is_fully_unboxed
  <2>6. QED
    BY <2>2, <2>3, <2>4, <2>5

<1>3. `ρ_j` の上の節点は `Let(m, Match(c, arms), Ret(m))`、アーム `j` の本体の
      `Let(x_j, Llvm(gen, []), Ret(x_j))`、そのアーム本体の終端の `Ret(x_j)`、関数本体の終端の `Ret(m)` の
      4 つである。本体は `Retain`・`Release`・`Destructure`・`Eval` の節点を持たず、`Let` の右辺は
      `Match` 1 つと `Llvm` 2 つだけである。
  BY <ref id=ca36627/>, <1>1

<1>4. `ρ_j` について、`(x_j, [])` と `(m, [])` はどちらも `ρ_j` のスロットであり、同じオブジェクトを
      指す。そのオブジェクトは計数下 (D26) である。
  <2>1. `(x_j, [])` は `ρ_j` のスロットである。`x_j` は `ρ_j` の上の節点 `Let(x_j, Llvm(gen, []), Ret(x_j))`
        が束縛するので DEF 路の位置 の (S2) で値を得ており (L0b (b))、`<1>2` より `[]` は `ty(x_j) = T` の
        inhabited な boxed leaf である。
    BY <ref id=09fabad/>, DEF 路の位置, <1>1, <1>2, <1>3
  <2>2. `(m, [])` は `ρ_j` のスロットである。`m` は `ρ_j` の上の節点 `Let(m, Match(c, arms), Ret(m))` が
        束縛するので (S2) で値を得ており、`<1>2` より `[]` は `ty(m) = T` の inhabited な boxed leaf で
        ある。
    BY <ref id=09fabad/>, DEF 路の位置, <1>1, <1>2, <1>3
  <2>3. E6 の辺 `(x_j, [])`-`(m, [])` は在り、`ρ_j` の上で実行された辺である。
    `<1>3` よりアーム `j` の本体の終端の `Ret(x_j)` は `ρ_j` の上にあり、`<1>1` よりその `Match` の
    束縛変数は `m` なので、それが定める E6 の辺は `(x_j, [])`-`(m, [])` である。`<2>1` と `<2>2` より
    両端は `ρ_j` の位置なので、辺は在る (DEF 辺の存在)。DEF `ρ` の上で実行された辺 の第 3 の場合より
    その辺は実行された。
    BY DEF 辺の存在, DEF `ρ` の上で実行された辺, <1>1, <1>3, <2>1, <2>2
  <2>4. QED
    L1 (a) より `<2>3` の辺の両端は同じオブジェクトを指す。`<1>1` より A3 の但し書き -- 実行時に参照カウントで
    分岐する op の `Fresh` の行はオブジェクトの同一性については字義どおりでない -- は `gen` に当たらない
    ので、A3 の「単一の `Fresh`」の行を字義どおりに読める。すなわち `obj(x_j, [])` はこの op が新しく
    割り当てたオブジェクトであり、D26 より割り当てられたオブジェクトは計数下である。
    BY <ref id=e11772a/>, <ref id=88a06de/>, <ref id=4c886c1/> (a), <1>1, <2>1, <2>2, <2>3

<1>5. `ρ_j` を辿る活性化について次が成り立つ。`Obl` は `Let(x_j, Llvm(gen, []), Ret(x_j))` の実行までは
      空で、その実行の後は `obj(x_j, [])` への参照 1 つだけからなり、終端の `Ret(m)` の消費を行った後は
      再び空である。`Obl` から参照を取り除く操作は終端の `Ret(m)` の消費だけである。
  <2>1. 初期値は空である。D10 の初期値は、所有するパラメータ・capture の unit の下の inhabited な各 leaf に
        つき参照 1 つであり、`<1>1` より `f` のパラメータは `c` だけで capture は無く、`<1>2` より
        `ty(c) = Bool` は boxed leaf を持たない。
    BY <ref id=f06144e/>, <1>1, <1>2
  <2>2. `Match` 節点自身とアーム本体の `Ret(x_j)` は `Obl` を変えない。D9 は `Let(x, Match(v, arms), k)` の
        `Match` 節点自身が参照を作らず、移さず、手放さないと述べる。アーム本体の `Ret(x_j)` は D9 の
        移動の表の第 2 行であり、D10 の移動の行より `Obl` を変えない。
    BY <ref id=9d74736/>, <ref id=f06144e/>, <1>3
  <2>3. `Let(x_j, Llvm(gen, []), Ret(x_j))` は `Obl` に `obj(x_j, [])` への参照を 1 つ加え、取り除かない。
    D10 の生成の表の `Llvm` の行は、`result_prov` の宣言が単一の `Arg(j, σ)` でない結果の leaf につき
    参照を 1 つ加える。`<1>1` より宣言は単一の `Fresh` であり、`<1>2` より `T` の leaf は `[]` 1 つで
    inhabited であり、`<1>4` よりそのオブジェクトは計数下である。D9 の `Llvm` の行が消費するのは
    `borrows_operand(i)` が偽のオペランドの leaf であり、`<1>1` より `gen` はオペランドを 1 つも
    取らないので、消費される leaf は無い。
    BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=88a06de/>, <1>1, <1>2, <1>4
  <2>4. 終端の `Ret(m)` の消費は `obj(m, []) = obj(x_j, [])` への参照を 1 つ取り除く。D9 の終端の `Ret` の
        行が消費するのは `m` の inhabited な全 boxed leaf であり、`<1>2` と `<1>4` よりそれは `[]` 1 つで、
        そのオブジェクトは `obj(x_j, [])` である。
    BY <ref id=9d74736/>, <ref id=f06144e/>, <1>2, <1>4
  <2>5. `Obl` から参照を取り除く操作は終端の `Ret(m)` の消費だけである。D10 で `Obl` から参照を取り除くのは
        `Release` の行と消費の行であり、`<1>3` より本体に `Release` 節点は無く、D9 の消費の表の 6 行の
        うち `<1>3` の 4 つの節点に当たるのは `Llvm` の行 (`<2>3` より消費する leaf は無い) と終端の
        `Ret` の行だけである。
    BY <ref id=9d74736/>, <ref id=f06144e/>, <1>3, <2>3
  <2>6. QED
    `<1>3` の 4 つの節点のほかに `Obl` を動かす機会は無い。`<2>1` より初期値は空、`<2>2` より `Match` 節点と
    アーム本体の `Ret` は `Obl` を変えず、`<2>3` が 1 つ加え、`<2>4` がそれを取り除き、`<2>5` が取り除く
    操作を尽くす。
    BY <1>3, <2>1, <2>2, <2>3, <2>4, <2>5

<1>6. `f` の本体は D11 の意味で RC 規律を満たす。
  <2>1. (S-a) が成り立つ。`<1>5` より `Obl` から参照を取り除く操作は終端の `Ret(m)` の消費だけであり、
        それが取り除く参照はその時点の `Obl` の唯一の元である。
    BY <ref id=95427eb/>, <1>5
  <2>2. (S-b) が成り立つ。`<1>5` より終端の `Ret(m)` の消費を行った後の `Obl` は空である。
    BY <ref id=95427eb/>, <1>5
  <2>3. (S-c) が成り立つ。`<1>3` より、この本体に現れる D7 の読む構文は `Let(x, Llvm(gen, args), k)` の
        行と `Let(x, Match(v, arms), k)` の行だけである。前者が読むのは各オペランドであり、`<1>1` より
        `gen` はオペランドを 1 つも取らない。後者が読むのは scrutinee `c` であり、`<1>2` より `c` は
        boxed leaf を持たない。よって読まれうるオブジェクトは無い。`<1>3` より `Retain` と `Release` の
        節点は無いので、触れるオブジェクトも無い。よって条件は空虚に成り立つ。
    BY <ref id=56c2068/>, <ref id=95427eb/>, <1>1, <1>2, <1>3
  <2>4. QED
    D3 より実行路はアームの選び方で尽くされ、`<2>1` から `<2>3` は `j` について量化した主張なので
    どちらのアームについても成り立つ。
    BY <ref id=ca36627/>, <2>1, <2>2, <2>3

<1>6a. 解析は鍵 `(x_j, [])` と `(m, [])` で `origin` を呼ぶ。したがってその 2 つについて
       `origin(x_j, [])` と `origin(m, [])` の値は `origin_inner` の 1 回の呼び出しが返した値である。
  `borrow_ify` は自分の入力プログラムについて `infer_ownership` を呼ぶ。`infer_ownership` は
  `prog.funcs` の各関数 `func` について `VarTable::of(func)` を作り、`collect_consumes(&func.body, ..)` を
  呼び、報告された各 `(var, path)` に
  ついて `origin(vars, type_env, &var, &path)` を呼ぶ。その `vars` は `f` については `VarTable::of(f)`
  であり、§1 が固定する表である。`collect_consumes` は `collect_consumes_go` を
  呼び、その `RcExpr::Ret(x)` の腕は `push_boxed_leaves` で `boxed_leaf_paths(ty(x), type_env)` の各 `p`
  について `(x.name, p)` を積み、`RcExpr::Let` の腕は `rhs` が `RcRhs::Match` のとき各アーム本体へも
  降りる。`<1>3` より `f` の本体は終端の `Ret(m)` を持ち、アーム `j` の本体は終端の `Ret(x_j)` を持つ
  ので、走査はその 2 つを訪れる。`<1>1` より `ty(m)` も `ty(x_j)` も `T` であり、`<1>2` より `T` の
  boxed leaf は `[]` 1 つなので、積まれるものの中に `(m, [])` と `(x_j, [])` がある。呼び出しが在るので
  L0 (a) が当たる。
  BY <ref id=ecdd35d/> (a), <1>1, <1>2, <1>3, CODE src/rc_ir/borrow.rs: borrow_ify, infer_ownership,
     CODE src/rc_ir/ownership.rs: collect_consumes, collect_consumes_go, push_boxed_leaves

<1>7. `id(x_j, []) = (x_j, [])` であり、`act(x_j, []) = {(x_j, [])}` である。
  <2>1. `x_j` の `Binding` は `Llvm(gen, [], T)` である。`collect_bindings` は
        `Let(x, Llvm(llvm_gen, args), k)` に対し `x` の `Binding` を `Llvm(llvm_gen, args, x.ty)` と
        する。
    BY <1>1, CODE src/rc_ir/ownership.rs: collect_bindings
  <2>2. `origin_inner` の `Llvm` の腕は `origin_from_leaves_under` の値を `unwrap_or_else(here)` に
        渡し、その結果を返す。`<1>1` より `decl.leaf_origins_at([])` は単一の `Fresh` からなる集合で
        あり、`as_arg_projection` はそれに `None` を返すので、腕はこの枝に入る。
    BY <1>1, <2>1, CODE src/rc_ir/ownership.rs: origin_inner, as_arg_projection
  <2>3. `origin_from_leaves_under` は `Some(Origin::Exactly((x_j, [])))` を返す。`<1>2` より `T` の
        boxed leaf は `[]` だけなので `decl.leaf_origins_under([])` が返すのはその 1 つの宣言だけであり、
        その元は `Fresh` なので、ループは `operand_units` に何も入れず `produced_here` を立てる。よって
        `reached` は `Exactly((x_j, []))` 1 つだけからなり、`reached.iter().all(..)` の枝がその値を返す。
    BY <1>2, <2>2, CODE src/rc_ir/ownership.rs: origin_from_leaves_under
  <2>4. QED
    `<2>3` より `origin_from_leaves_under` の値は `Some(Origin::Exactly((x_j, [])))` なので、`<2>2` の
    `unwrap_or_else(here)` はその中身をそのまま返し、腕の値は `Exactly((x_j, []))` である。`<1>6a` より
    `origin(x_j, [])` の値はその `origin_inner` の呼び出しが返した値であり、`Exactly(p)` の
    `identity()` は `p`、`acted_on()` は `[p]` である。
    BY <1>6a, <2>2, <2>3, CODE src/rc_ir/ownership.rs: Origin::identity, Origin::acted_on

<1>8. `id(m, []) = (m, [])` である。
  <2>1. `m` の `Binding` は `Join([x_0, x_1])` である。`collect_bindings` は `Let(m, Match(s, arms), k)` に
        対し `m` の `Binding` を `Join(arm_results)` とし、`arm_results` は各アームの
        `returned_var(&arm.body)` の列である。`<1>1` より 2 つのアーム本体の終端はそれぞれ `Ret(x_0)`、
        `Ret(x_1)` なので、`returned_var` が返すのは `x_0` と `x_1` である。`returned_var` の本体は
        `grow_stack` に包まれているので、その値を読むのに A15 が要る。
    BY <ref id=3e6b0e0/>, <1>1, CODE src/rc_ir/ownership.rs: collect_bindings, returned_var
  <2>2. `origin_inner` の `Join` の腕が集める候補集合は `{(x_0, []), (x_1, [])}` であり、`<1>1` の A6 より
        `x_0` と `x_1` は相異なる名前なので 2 元である。
    BY <1>1, <1>6a, <1>7, <2>1, CODE src/rc_ir/ownership.rs: origin_inner
  <2>3. QED
    `<2>2` より `of_candidates` は元数 2 の集合を受け取るので `Join { identity: (m, []), .. }` を返す。
    BY <2>2, CODE src/rc_ir/ownership.rs: Origin::of_candidates, Origin::identity

<1>9. `ρ_j` の上に在り (DEF 辺の存在)、かつ `ρ_j` の上で実行された `f` の本体の別名の辺は、E6 の辺
      `(x_j, [])`-`(m, [])` の 1 本だけである。L1 (b) より、これは D20 の意味で在る辺の全体である。
  <2>1. E1 の辺は `Let(_, Var(_), _)` の節点を要し、E2 の辺は `Destructure` の節点を要する。`<1>3` の
        本体はこの 2 種の節点を持たない。
    BY <ref id=9c7c27a/>, <1>3
  <2>2. E5 の辺は、結果の leaf の宣言が単一の `Arg(j, σ)` であることを要する。`<1>1` より `Llvm` 節点の
        結果の唯一の leaf の宣言は単一の `Fresh` なので、E5 の辺は無い。
    BY <ref id=9c7c27a/>, <1>1, <1>2
  <2>3. E4 の辺は catch-all アームを要する。`<1>1` の 2 つのアームはどちらも `tag` を持つので、E4 の辺は
        無い。E3 の辺は payload 変数 `p_0` / `p_1` の位置を端に持つが、`<1>2` より `p_0` と `p_1` の
        型は boxed leaf を持たないので、この 2 つを端とする `ρ_j` の位置は無く、DEF 辺の存在 よりその辺も
        無い。
    BY <ref id=9c7c27a/>, DEF 辺の存在, <1>1, <1>2
  <2>4. E6 の辺は 2 つのアーム本体の終端の `Ret` から来る。`<1>3` より `ρ_j` の上にあるのはアーム `j` の
        本体の終端の `Ret(x_j)` だけであり、`<1>2` より `T` の boxed leaf は `[]` だけなので `λ` の
        選び方は 1 つである。よって `ρ_j` の上に在る E6 の辺は `(x_j, [])`-`(m, [])` の 1 本である
        (`<2>3` と同じく DEF 辺の存在 による)。その `Ret(x_j)` は `ρ_j` の上にあるので、
        DEF `ρ` の上で実行された辺 の第 3 の場合よりその辺は実行された。
    BY <ref id=9c7c27a/>, DEF 辺の存在, DEF `ρ` の上で実行された辺, <1>1, <1>2, <1>3
  <2>5. QED
    `<2>1` から `<2>3` が E1 から E5 の辺が 1 本も在らないことを、`<2>4` が E6 の辺が 1 本であることを
    与える。L1 (b) より、在ってかつ実行された辺の全体が D20 の意味で在る辺の全体である。
    BY <ref id=9c7c27a/>, <ref id=4c886c1/> (b), <2>1, <2>2, <2>3, <2>4

<1>10. QED
  `<1>6` より `f` の本体は RC 規律を満たす。`<1>4` より `ρ_0` の上で `(x_0, [])` と `(m, [])` はどちらも
  スロットであり、同じオブジェクトを指す。`<1>6a` より解析はその 2 つの鍵で `origin` を呼ぶ。
  `<1>7` と `<1>8` より `id(x_0, []) = (x_0, [])`、
  `id(m, []) = (m, [])` であり、`<1>1` の A6 より `x_0` と `m` は相異なる名前なので、この 2 つの
  `VarPath` は異なる。`<1>9` を `j = 0` に当てると、`ρ_0` の上に D20 の意味で在る別名の辺は E6 の
  `(x_0, [])`-`(m, [])` の 1 本だけなので、`(x_0, [])` と `(m, [])` はその 1 本からなる別名の道 (D20) で
  結ばれ、この 2 つを結ぶどの道も E6 の辺だけからなる。
  BY <ref id=9c7c27a/>, <1>1, <1>4, <1>6, <1>6a, <1>7, <1>8, <1>9

**補足 (両端は計数下である)**。`<1>4` の 2 つのスロットが指すオブジェクトは計数下 (D26) である。よって
この本体は、P5 (b) を計数下のスロットに限った形への反例にもなっている。

## P5 (c) -- 被覆

**言明** (README の P5 (c))。**1 つの本体 (D23)** の 1 回の活性化について、`Release(v, π)` の走査が
`un_bump` と `consume_objects` に渡すオブジェクトの
和 -- すなわち `ActRefs(v, π).objects()` と `other_objects(v, π)` の和 -- は、`π` の下の各 boxed leaf `λ` に
ついて `origin(v, λ).acted_on()` をすべて含む。

**この節が「オブジェクト」と呼ぶものは位置 (`VarPath`) である。** `References::objects` と
`References::names` と `References::shares_an_object` の鍵、`CancelAnalysis::other_objects` の返り値、
`consume_objects` の第 2 引数は、いずれも `VarPath` である
(`CODE src/rc_ir/ownership.rs: References`, `CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`,
`CancelAnalysis::consume_objects`)。`VarPath` は D6 の位置であって D7 の実行時のオブジェクトではなく、
2 つが `obj(・)` で写り合うことは P5 (a) が述べる。以下、この節の各集合は `VarPath` の集合である。

<1>0. この文書が固定する本体は、ある関数の `body` かあるグローバル初期化子の `init` かのどちらかであり
      (§1)、D23 の「本体」はその 2 つで尽きる。以下の各段はその固定の下で述べられているので、言明の
      主語である本体を尽くす。
  BY <ref id=ff5985d/>

<1>1. `walk_inner` の `RcExpr::Release(v, path, _, k)` の腕は、`other_objects(v, path)` を
      `consume_objects` に渡し、`acted_references(v, path)` を `un_bump` に渡す。`un_bump` が読むのは
      その `References` の鍵ごとの個数であり、それが名指す鍵の全体は `ActRefs(v, π).objects()` である。
  <2>1. 腕は `let others = self.other_objects(v, path); self.consume_objects(&mut pending, &others);` の
        のち `let un_bumped = self.acted_references(v, path);` を `un_bump(&mut pending, &un_bumped)` に
        渡す。`UnBump::OutsideBracket` の枝はさらに `un_bumped.objects()` を `consume_objects` に渡す。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner の `RcExpr::Release` の腕
  <2>2. `CancelAnalysis::acted_references(v, path)` は `ownership::acted_references(vars, type_env, v, path)`
        の値である (空でないことを表明するほかに何もしない)。
    BY CODE src/rc_ir/borrow.rs: CancelAnalysis::acted_references
  <2>3. `un_bump(pending, un_bumped)` が `un_bumped` から読むのは `shares_an_object`、`covers`、
        `subtract` であり、いずれも `un_bumped` の鍵 (`VarPath`) ごとの個数についての演算である。
    BY <ref id=cbc4a1c/>, CODE src/rc_ir/borrow.rs: un_bump, CODE src/rc_ir/ownership.rs: References
  <2>4. QED
    BY <2>1, <2>2, <2>3

<1>1a. `boxed_leaf_paths(ty(v), type_env)` の要素のうち `leaf.starts_with(π)` を満たすものの全体は
       `L(v, π)` である。
  BY <ref id=0594f24/>, DEF `L`

<1>1b. `Release(v, π)` の訪問が呼ぶ `acted_references(v, π)` と `other_objects(v, π)` は、どちらも
       `L(v, π)` の各 `λ` について `origin(vars, type_env, &v.name, &λ)` を呼ぶ。よってこの節が書く
       `id(v, λ)`・`cand(v, λ)`・`act(v, λ)` は、いずれも解析が呼ぶ鍵についての値である (§1)。
  どちらの関数も `boxed_leaf_paths(&v.ty, type_env)` の要素のうち `leaf.starts_with(path)` を満たす各
  `leaf` について `origin` を呼び、`<1>1a` よりその `leaf` の全体は `L(v, π)` である。
  BY <1>1, <1>1a, CODE src/rc_ir/ownership.rs: acted_references,
     CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects

<1>2. `ActRefs(v, π).objects()` は `{ id(v, λ) : λ ∈ L(v, π) }` である。
  <2>1. `acted_references(v, π)` は、`boxed_leaf_paths(ty(v), type_env)` の要素 `leaf` のうち
        `leaf.starts_with(π)` を満たすものについて、`origin(v, leaf).identity()` をキーとする計数を
        1 ずつ増やした `Map<VarPath, usize>` を `References` に包んで返す。
    BY <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: acted_references
  <2>2. `References::objects()` はその `Map` のキーの列を返す。
    BY <ref id=cbc4a1c/>, CODE src/rc_ir/ownership.rs: References::objects
  <2>3. QED
    BY <1>1a, <1>1b, <2>1, <2>2

<1>3. `other_objects(v, π)` は `∪_{λ ∈ L(v, π)} (cand(v, λ) \ {id(v, λ)})` を含む。
  `other_objects` は `boxed_leaf_paths(ty(v), type_env)` のうち `leaf.starts_with(path)` を満たす各 `leaf`
  について `where_from = origin(v, leaf)` を取り、その `candidates()` のうち `identity()` と異なるものを
  すべて `out` に積む。回る `leaf` の全体は `<1>1a` より `L(v, π)` である。
  BY <1>1a, <1>1b, CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects

<1>4. `L(v, π)` の各 `λ` について `act(v, λ) = {id(v, λ)} ∪ (cand(v, λ) \ {id(v, λ)})` である。
  `<1>1b` より解析は鍵 `(v, λ)` で `origin` を呼ぶので、この 3 つの値が定まる。`Origin::acted_on()` は
  `identity()` を先頭に、`candidates()` のうち `identity()` と異なるものを続けた
  列である。
  BY <ref id=cbc4a1c/>, <1>1b, CODE src/rc_ir/ownership.rs: Origin::acted_on

<1>5. QED
  `<1>4` より `∪_{λ ∈ L(v, π)} act(v, λ) = { id(v, λ) : λ ∈ L(v, π) } ∪
  ∪_{λ ∈ L(v, π)} (cand(v, λ) \ {id(v, λ)})` であり、第 1 項は `<1>2` より `ActRefs(v, π).objects()` に
  等しく、第 2 項は `<1>3` より `other_objects(v, π)` に含まれる。`<1>1` より、この 2 つが走査の
  `un_bump` と `consume_objects` に渡るオブジェクトである。`<1>0` よりこの結論は言明の主語である
  本体を尽くす。
  BY <1>0, <1>1, <1>2, <1>3, <1>4

**補足 (逆向きも成り立つ)**。`<1>2` と `<1>3` はどちらも等号で成り立つので、和はちょうど
`∪_{λ ∈ L(v, π)} act(v, λ)` である。P5 (c) の言明が包含の向きだけを述べるのは `README.md` の書き方で
あり、この証明はその強い側も与える。

## P6 (`acted_references` は静的な上位近似である)

P6 の 2 つの主張を次のように書く。

- **(a)** `acted_references(v, π)` は `L(v, π)` の各元 `λ` について鍵 `(v, λ)` で `origin` を呼び、
  返す `References` は、`L(v, π)` のすべての元 `λ` を `id(v, λ)` で名付けて数えた多重集合である。
- **(b)** 位置 `p` (D23) において `Retain(v, π)` が実行時に作る参照の多重集合は、(a) の数え上げを
  `Linhc(v, π, p)` に制限し、各名前をそれが指すオブジェクトへ写して得られる多重集合に等しい。
  `Release(v, π)` が実行時に処分する参照の多重集合も同じものに等しい。

**DEF `Linhc`**
`Linh(v, π, p)` の元のうち、`obj(v, λ)` が計数下 (D26) であるものの集合を `Linhc(v, π, p)` と書く。

(a) は README の P6 の第 1 文と第 2 文、(b) は第 3 文である。第 3 文が「**inhabited (D16)
かつ計数下 (D26)**」の
leaf への制限と「**各名前をそれが指すオブジェクトへ写して**」の段を持つのは、D8 と D26 による。D26 より、
グローバル状態のオブジェクトを指す leaf は D8 の意味の参照を持たないので、`Retain` はその leaf について
参照を作らない。また D8 より参照の多重集合は**オブジェクトごと**の個数であり、(a) の数え上げは `VarPath`
ごとの個数なので、2 つを比べるには名前をオブジェクトへ写す写像が要る。この写像を先に定める。

**DEF 名前の指すオブジェクト**
`ρ` の位置 `(w, σ)` に対し `ν(w, σ) := obj(w, σ)` と置く。**`ν` が `ρ` の位置である `VarPath` の集合から
オブジェクトの集合への写像として定まることは、定義ではなく P6 (b) の `<1>0` が示す。**`VarPath` の多重集合 `M` の各元が `ρ` の位置であるとき、
`ν` による**押し出し** `ν_*M` を、各オブジェクト `o` について
`(ν_*M)(o) := Σ_{n : ν(n) = o} M(n)` で定める。

(b) の「等しい」は、右辺 (`VarPath` の多重集合) を `ν` で押し出したものが、左辺 (オブジェクトごとの個数) に
等しいことである。

### P6 (a)

<1>1. `acted_references(v, path)` は、`boxed_leaf_paths(&v.ty, type_env)` の要素 `leaf` のうち
      `leaf.starts_with(path)` を満たすものについて `origin(vars, type_env, &v.name, &leaf)` を呼び、
      その `identity()` をキーとする計数を 1 ずつ増やした `Map<VarPath, usize>` を組み立て、それを
      `References` に包んで返す。
  BY CODE src/rc_ir/ownership.rs: acted_references, References

<1>2. `<1>1` の走査が回る `leaf` の全体は `L(v, π)` である。inhabited かどうかは判定していない。
  BY <ref id=0594f24/>, <ref id=cbc4a1c/>, DEF `L`, <1>1

<1>3. QED
  `<1>1` と `<1>2` より、`acted_references(v, π)` は `L(v, π)` の各元 `λ` について鍵 `(v, λ)` で
  `origin` を呼び、返り値の `References` は `L(v, π)` の各元 `λ` を `id(v, λ)` で名付けて数えた
  多重集合である。
  BY <1>1, <1>2

### P6 (b)

<1>0. `ν` は、`ρ` の位置である `VarPath` の集合からオブジェクトの集合への写像として定まる。したがって
      DEF 名前の指すオブジェクト の押し出し `ν_*` も定まる。
  L0b (c) より `obj(w, σ)` は `w` が得る値と `σ` だけで決まり `ρ` の上の位置に依らないので、`ρ` の
  位置 `(w, σ)` にオブジェクトが 1 つ対応する。記号の位置も値とオブジェクトを持つ (D6) ので、`ρ` の
  位置のうちスロットでないものにも対応するオブジェクトが在る。
  BY <ref id=596a46d/>, <ref id=09fabad/>, DEF 名前の指すオブジェクト

<1>1. `Retain(v, π)` が `Obl` に加える参照は、`Linhc(v, π, p)` の各 `λ` について `obj(v, λ)` への参照
      1 つずつであり、それが全部である。
  <2>1. D10 の `Retain` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        加える」である。
    BY <ref id=f06144e/>
  <2>2. `<2>1` の行が渡る leaf -- `π` の下の inhabited な各 leaf -- の全体は `Linh(v, π, p)` である。
    BY <ref id=0594f24/>, <ref id=66c9670/>, DEF `L`, DEF `Linh`
  <2>3. `Linh(v, π, p)` の元のうち `obj(v, λ)` がグローバル状態であるものについては、参照は加わらない。
    D26 より、D8 の参照と D10 の義務集合は計数下のオブジェクトへの参照だけを対象とし、グローバル状態の
    オブジェクトを指す leaf は D8 の意味の参照を持たない。
    BY <ref id=ec8d1a0/>, <ref id=f06144e/>, <ref id=88a06de/>
  <2>4. QED
    `<2>2` から `<2>3` を除いた残りが `Linhc(v, π, p)` である。
    BY DEF `Linhc`, <2>1, <2>2, <2>3

<1>2. `Release(v, π)` が `Obl` から取り除く参照は、`Linhc(v, π, p)` の各 `λ` について `obj(v, λ)` への
      参照 1 つずつであり、それが全部である。
  <2>1. D10 の `Release` の行は「`π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ
        取り除く」である。
    BY <ref id=f06144e/>
  <2>2. `<2>1` の行が渡る leaf -- `π` の下の inhabited な各 leaf -- の全体は `Linh(v, π, p)` である。
    BY <ref id=0594f24/>, <ref id=66c9670/>, DEF `L`, DEF `Linh`
  <2>3. `Linh(v, π, p)` の元のうち `obj(v, λ)` がグローバル状態であるものについては、取り除く参照が
        無い。
    D26 より、D8 の参照と D10 の義務集合は計数下のオブジェクトへの参照だけを対象とし、グローバル状態の
    オブジェクトを指す leaf は D8 の意味の参照を持たない。
    BY <ref id=ec8d1a0/>, <ref id=f06144e/>, <ref id=88a06de/>
  <2>4. QED
    `<2>2` から `<2>3` を除いた残りが `Linhc(v, π, p)` である。
    BY DEF `Linhc`, <2>1, <2>2, <2>3

<1>2a. `Linhc(v, π, p)` の各 `λ` について `(v, λ)` は `ρ` の位置である。
  DEF `Linhc` と DEF `Linh` より `λ` は `ty(v)` の boxed leaf であって `v` の値で inhabited である。
  `v` が `ρ` の上で値を得ていることは次による。この節が固定する位置 `p` には `Retain(v, π)` または
  `Release(v, π)` の節点が在り、A11 よりその `v` の使用はその位置でスコープに入っている束縛に解決する。
  D2 の「束縛の及ぶ範囲」よりその束縛は、この本体のパラメータ・capture か、`p` の祖先の節点が作った
  ものである。前者は DEF 路の位置 の (S1) である。後者について、D3 より実行路は根から辿るので `p` の
  祖先はすべて `ρ` の上にあり、D2 の束縛節点は `Let`・`Destructure`・`Match` のアームの `payload` の
  3 種なので、前 2 者は (S2)、`payload` は (S3) である -- `payload` のスコープはそのアームの本体の
  部分木なので `p` はその部分木にあり、D3 より `ρ` はそのアームを選んでいる。`v` が `vars.bindings` に
  束縛を持たない名前であるときは、`v` は `ρ` の上の節点 `p` に現れるので (S4) である。
  BY <ref id=3905b4e/>, <ref id=b3dfa37/>, <ref id=ca36627/>, <ref id=09fabad/>, DEF 路の位置, DEF `Linh`, DEF `Linhc`

<1>3. `Retain(v, π)` が作る参照の、オブジェクトごとの多重集合は、各オブジェクト `o` について
      `#{λ ∈ Linhc(v, π, p) : obj(v, λ) = o}` である。`Release(v, π)` が処分する参照の多重集合も同じで
      ある。
  D8 より、同じオブジェクトへの参照どうしは互いに区別されず、参照の多重集合はオブジェクトごとの個数と
  して読む。`<1>1` と `<1>2` は `λ` ごとに `obj(v, λ)` への参照を 1 つずつ数えている。
  BY <ref id=ec8d1a0/>, <1>1, <1>2

<1>4. 各 `λ ∈ Linhc(v, π, p)` について、`id(v, λ)` は `ρ` の位置であり、`ν(id(v, λ)) = obj(v, λ)` で
      ある。
  `<1>2a` より `(v, λ)` は `ρ` の位置である。(b) の言明が数えるのは `acted_references(v, π)` が返した
  多重集合を制限したものなので、その呼び出しは行われており、P6 (a) よりその呼び出しは `L(v, π)` の各元に
  ついて鍵 `(v, ・)` で `origin` を呼ぶ。DEF `Linhc` と DEF `Linh` と DEF `L` より
  `Linhc(v, π, p) ⊆ L(v, π)` なので、解析は鍵 `(v, λ)` で `origin` を呼ぶ。よって L4 の 2 つの前件が
  揃い、L4 の (i) が `id(v, λ)` が `ρ` の位置であることを、(ii) が `obj(v, λ) = obj(id(v, λ))` を与え、
  DEF 名前の指すオブジェクト より `obj(id(v, λ)) = ν(id(v, λ))` である。
  BY <ref id=8253e68/>, <ref id=4f4714c/> (a), DEF `L`, DEF `Linh`, DEF `Linhc`, DEF 名前の指すオブジェクト, <1>2a

<1>5. P6 (a) の数え上げを `Linhc(v, π, p)` に制限した多重集合を `M := Σ_{λ ∈ Linhc(v, π, p)} [id(v, λ)]` と
      おくと、その押し出しは各オブジェクト `o` について
      `(ν_*M)(o) = #{λ ∈ Linhc(v, π, p) : obj(v, λ) = o}` である。
  P6 (a) より `M` は `Linhc(v, π, p)` の各元 `λ` を `id(v, λ)` で名付けて数えた多重集合である。`<1>4` より
  `M` の各元は `ρ` の位置であり、`<1>0` よりその上で押し出しが定まる。その定義より
  `(ν_*M)(o) = #{λ ∈ Linhc(v, π, p) : ν(id(v, λ)) = o}` である。`<1>4` の
  `ν(id(v, λ)) = obj(v, λ)` を代入する。
  BY <ref id=4f4714c/> (a), DEF 名前の指すオブジェクト, <1>0, <1>4

<1>6. QED
  `<1>3` の 2 つの多重集合と `<1>5` の `ν_*M` は、どのオブジェクト `o` についても同じ値
  `#{λ ∈ Linhc(v, π, p) : obj(v, λ) = o}` を取る。`ν` で写す段は DEF 名前の指すオブジェクト の
  押し出しそのものである。
  BY DEF 名前の指すオブジェクト, <1>3, <1>5

### P6 の結論

<1>1. README の P6 の第 1 文 --「**1 つの本体 (D23)** の 1 回の活性化について、`acted_references(v, π)` が返す `References` は、`π` の下の
      すべての boxed leaf を `origin` の identity で名付けて数えたものである」-- が成り立つ。
  DEF `L` より `L(v, π)` が「`π` の下のすべての boxed leaf」である。この文書が固定するのは、ある関数の
  `body` かあるグローバル初期化子の `init` かのどちらかであり (§1)、D23 の「本体」はその 2 つで尽きる
  ので、固定した本体を渡ると第 1 文の主語である本体を尽くす。
  BY <ref id=0594f24/>, <ref id=ff5985d/>, <ref id=4f4714c/> (a), DEF `L`
<1>1a. README の P6 の第 2 文 --「**その各 leaf について解析が
      `origin` を呼ぶ** -- `acted_references` 自身がその呼び出しである」-- が成り立つ。
  P6 (a) の前半がこれを与える -- `acted_references(v, π)` は `L(v, π)` の各元について鍵 `(v, ・)` で
  `origin` を呼ぶ。`<1>1` より `L(v, π)` が第 1 文の言う leaf の全体である。
  BY <ref id=4f4714c/> (a), DEF `L`, <1>1
<1>2. README の P6 の第 3 文 --「この数え上げを **inhabited (D16)
      かつ計数下 (D26)** の leaf に制限し、**各名前をそれが指すオブジェクトへ写して**得られる多重集合は、実行時に
      `Retain(v, π)` が作る参照の多重集合に等しく、`Release(v, π)` が処分する参照の多重集合にも等しい」--
      が成り立つ。
  DEF `Linhc` が「**inhabited (D16) かつ計数下 (D26)** の leaf」の全体であり、
  DEF 名前の指すオブジェクト の押し出し `ν_*` が
  「**各名前をそれが指すオブジェクトへ写して**得られる多重集合」である。
  BY <ref id=4f4714c/> (b), DEF `Linhc`, DEF 名前の指すオブジェクト
<1>3. QED
  README の P6 は 3 つの文からなり、第 1 文が `<1>1`、第 2 文が `<1>1a`、第 3 文が `<1>2` である。
  3 つの文の主語は
  「**1 つの本体 (D23)** の 1 回の活性化」であり、`<1>1` がその範囲を尽くすことを述べる -- 第 2 文と
  第 3 文は第 1 文の主語を引き継ぐ。
  BY <ref id=ff5985d/>, <1>1, <1>1a, <1>2

**補足 1 (名前づけが 2 つのオブジェクトを 1 つに潰さないこと)**。`References` の鍵は `VarPath` であり、
`covers` などの演算は鍵ごとに数える (D15)。鍵をオブジェクトの名前として読めるためには、1 つの鍵が 2 つの
オブジェクトを名指さないことと、`id` が 2 つのオブジェクトを 1 つの鍵に潰さないことの両方が要る。前者は
`ν` が写像であることそのものである (DEF 名前の指すオブジェクト)。後者は L4 の (ii) から出る --
`ν(id(v, λ)) = obj(v, λ)` なので、`id(v, λ) = id(v, μ)` ならば `obj(v, λ) = ν(id(v, λ)) = ν(id(v, μ)) =
obj(v, μ)` である。これは P5 (a) を 1 つの `v` の 2 つの leaf に限った形であり、P5 (a) がここで果たす
役割はこれである。

**補足 2 (1 つのオブジェクトが 2 つの鍵を持つこと)**。逆向きは成り立たない。`ν` は単射ではなく、同じ
オブジェクトを指す 2 つのスロットが相異なる `identity` を持つ本体がある (R1)。よって鍵ごとの多重集合は、
オブジェクトごとの多重集合より細かい情報を持つ。P6 (b) の等号がこの細かさに耐えるのは、両辺を `ν` で
押し出してから比べるからであり、その押し出しを `λ` ごとに与えるのが L4 の (ii) である。鍵の粒度で
比べる読み手 -- `un_bump` の `covers` -- が R1 の形をどう扱うかは P17 と P18b が扱う。

**補足 3 (2 つの leaf が 1 つの名前を持つとき)**。`L(v, π)` の相異なる 2 つの leaf が同じ `id` を持つことが
ある。返る `References` はそのとき計数を 2 にする (P6 (a))。その 2 つがどちらも `Linhc(v, π, p)` に入るときは、
A5 の下で参照は inhabited かつ計数下の leaf ごとに 1 つなので、参照としても 2 つある。

**補足 4 (上位近似のずれは片側だけである)**。`acted_references` は `L(v, π)` を数え、実行時に触れるのは
`Linhc(v, π, p)` である。`Linhc(v, π, p)` は `L(v, π)` の部分集合なので、`References` の数はつねに実行時に
触れる参照の数以上である。この差を読むのは `un_bump` の `covers` と `subtract`、`consume_objects` の
`names`、`merge` の `References` の等号である (`CODE src/rc_ir/borrow.rs: un_bump`,
`CancelAnalysis::consume_objects`, `CancelAnalysis::merge`)。差が対の判定にどう効くかは P18b が扱う --
`outstanding` が実行時の bump を `covers` することがその言明である。

## P7 (消費の網羅性)

P7 の 2 つの主張を次のように書く。README の P7 と同じく、どちらも**関数の本体を第 1 引数に渡した**
`collect_consumes` の呼び出しについての言明である。量化の範囲は 2 つで違う。

- **(a)** DEF leaf 粒度の所有 を満たす `own` を渡した呼び出しについて、D9 の消費の表の各行が指す leaf は
  `collect_consumes` が `out` に積む。
- **(b)** **どの `own` を渡した呼び出しについても**、`collect_consumes` が `out` に積むもののうち D9 の
  消費の表に無いものは、`Match` のアーム本体の終端の `Ret` が積むものに限る。

これに命題 L6 を添える。README の P7 の言明には無いが、報告しない箇所が参照の収支を狂わせないことを
述べるので、ここで併せて示す。L6 は (a) と同じく DEF leaf 粒度の所有 を満たす `own` についての言明で
ある。

第 1 引数を関数の本体に限るのは、`collect_consumes` の呼び出しがその形のものだけだからであり、それを
与えるのは `L5 (b')` と `L5 (n)` である。**D9 の消費の表の最後の行は「本体 (D23) の終端の `Ret(x)`」で
あり、D23 の「本体」は関数の `body` とグローバル初期化子の `init` の両方を指す。** この文書が示すのは
関数の `body` を渡した呼び出しについてであり、グローバル初期化子の `init` については `collect_consumes` が
呼ばれない。

**D9 の `App` の行が言う「呼び出し先」は実行時の関数であり、`rhs_consumes` が読むのは
`resolve_callee_params` が静的に決める呼び出し先である。** D23 が前者を定め、後者が前者と食い違わない
ことを P29 が述べる。P29 は `borrow_ify` の**入力**の `App` についての言明であり、`L5 (n)` より
`collect_consumes` が走るのはまさに `borrow_ify` の入力の本体についてなので、P7 はそのまま P29 を使える。

`collect_consumes` は `own` 引数を取り、`owns(p, λ)` を `own.contains(&(p.name, λ))` として使う
(`CODE src/rc_ir/ownership.rs: collect_consumes`)。D9 の `App` の行が言う所有は D14 の unit 粒度の所有
なので、(a) は次の `own` について述べる。

**DEF leaf 粒度の所有**
`p` を関数のパラメータ、`λ` を `ty(p)` の boxed leaf とする。`own` が DEF leaf 粒度の所有 を満たすとは、
`(p.name, λ) ∈ own` であることが「`p` の unit `truncate_to_unit(ty(p), λ, type_env)` が D14 の意味で
所有される」ことと同値であることをいう。**`truncate_to_unit(ty(p), λ, type_env)` が `rc_units(ty(p))` の
要素であること -- すなわちこの対応が定まること -- は、定義ではなく P7 (a) の `<2>2b` が示す。**

`infer_ownership` が渡す `owned_leaves` は不動点計算の途中の集合であり、それが DEF leaf 粒度の所有 に
一致するかどうかは P8 が扱う。`cancel` の側の `CancelAnalysis::consume_rhs` は `collect_consumes` を
呼ばず `rhs_consumes` を直接呼ぶので (`L5 (n)`)、P7 はそちらについて何も述べない
(`CODE src/rc_ir/borrow.rs: CancelAnalysis::consume_rhs`)。

### L5a (unbox 容器の boxed leaf はフィールドの添字で始まる) <!--#9d4ff56-->

**言明**。本体の `Destructure(c, fs, s, k)` の節点であって `c.ty.is_box(type_env)` が偽であるものに
ついて、次の 2 つが成り立つ。

- **(a)** `boxed_leaf_paths(ty(c), type_env)` の各元は空でない path であり、その先頭の添字は、その leaf が
  属するフィールドの添字である。
- **(b)** D9 の `Destructure` (unbox) の行が消費とする**名前が付いていないフィールドの leaf**の全体は、
  `boxed_leaf_paths(ty(c), type_env)` の元のうち**先頭の添字が `fs` の名前付きフィールドの添字でない**
  ものの全体に等しい。

(b) が成り立つのは D4 の規則 5 による -- unbox 集約の leaf は、フィールドの添字をそのフィールドの型の
leaf に前置したものだからである。

<1>1. `ty(c)` はクロージャではなく、その tycon の `variant` は `TyConVariant::Struct` である。したがって
      `is_array(ty(c))` は偽である。
  A12 の「`Destructure` の容器が構造体であること」の行が `is_struct(ty(c))` を与え、`is_struct` は
  `toplevel_tycon_info` が返す `TyConInfo` の `variant` が `Struct` であることである。A12 は、型の
  `variant` を述べる各節ではその型の `is_closure()` が偽であると述べ、`Destructure` の容器が構造体で
  あることをその節の 1 つに挙げる。
  `is_struct` が `toplevel_tycon_info` 経由で真であることから `toplevel_tycon_info(ty(c), type_env)` は
  値を返すので、L3a (b) より `is_array(ty(c))` が真ならばその `variant` は `Array` である。`variant` が
  `Struct` である `ty(c)` には当たらない。
  BY <ref id=83d98e9/>, <ref id=f68ae1c/> (b), CODE src/ast/types.rs: TypeNode::is_struct,
     CODE src/ast/types.rs: TypeNode::toplevel_tycon_info, TyConVariant

<1>2. CASE `is_fully_unboxed(ty(c))` が真。
  D4 の規則 1 より `boxed_leaf_paths(ty(c), type_env)` は空なので、(a) は空虚に成り立ち、(b) の 2 つの
  集合はどちらも空である。
  BY <ref id=0594f24/>

<1>3. CASE `is_fully_unboxed(ty(c))` が偽。
  D4 の規則 1 は当たらず、`<1>1` より規則 2 (クロージャ) と規則 4 (`is_array`) も当たらず、前提より
  規則 3 (`is_box`) も当たらない。よって規則 5 が当たり、`ty(c)` の各 leaf は `unpunched_field_types` が
  返すフィールドの添字を、そのフィールドの型の leaf に前置したものである。すなわち各 leaf は空でない
  path を持ち、その先頭の添字はその leaf が属するフィールドの添字であって、これが (a) である。よって
  「名前が付いていないフィールドの leaf」と「先頭の添字が名前付きフィールドの添字でない leaf」は同じ
  集合であり、これが (b) である。
  BY <ref id=0594f24/>, <1>1

<1>4. QED
  `<1>2` と `<1>3` は `is_fully_unboxed(ty(c))` の真偽の 2 つの場合で尽きており、どちらでも (a) と (b) が
  成り立つ。
  BY <1>2, <1>3

### L5 (`collect_consumes` が積むもの) <!--#c5547e4-->

**言明**。次が成り立つ。

- **(a)** `collect_consumes` は `owns` を作って、渡された本体について `collect_consumes_go` を呼ぶだけで
  ある。
- **(b)** `collect_consumes_go(e, ..)` の走査は、`e` の各節点をちょうど 1 度訪れる。走査が訪れる
  `RcExpr::Ret` の節点は、`e` の終端の `Ret` か、走査が通った `Let(x, Match(_, arms), k)` のあるアームの
  `arm.body` の終端の `Ret` かのどちらかである。
- **(b')** `infer_ownership` は `collect_consumes` を、プログラムの各関数の `func.body` を第 1 引数として
  呼ぶ。
- **(c)** `RcExpr::Ret(x)` の腕は、`boxed_leaf_paths(ty(x), type_env)` の各 `p` について `(x.name, p)` を
  積む。
- **(d)** `RcExpr::Destructure(c, fs, _, k)` の腕は、`destructure_consumes(c, fs, type_env)` の各 `leaf` に
  ついて `(c.name, leaf)` を積む。
- **(e)** 本体の `Destructure(c, fs, s, k)` の節点について、`destructure_consumes(c, fs, type_env)` は、
  `c.ty.is_box(type_env)` が真のとき `boxed_leaf_paths(ty(c), type_env)` の全部を返し、偽のときはその
  うち先頭の添字が `fs` の名前付きフィールドの添字でないものだけを返す。
- **(f)** `RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は何も積まない。`RcExpr::Let(x, rhs, k)` の
  腕は、`rhs` が `RcRhs::Match` のとき自身は何も積まず (各アーム本体へ再帰する)、それ以外の 4 種のとき
  `rhs_consumes(rhs, &x.ty, ..)` を呼ぶ。
- **(g)** `rhs_consumes` の `RcRhs::Var(_) | RcRhs::Match(..)` の腕は何も積まない。
- **(h)** `rhs_consumes` の `RcRhs::Closure(_, caps)` の腕は、`caps` の各元 `c` の各 boxed leaf `p` に
  ついて `(c.name, p)` を積む。
- **(i)** `rhs_consumes` の `RcRhs::App(callee, args)` の腕は、`callee` の全 boxed leaf を積み、さらに
  各引数 `args[i]` の各 boxed leaf `leaf` について、`resolve_callee_params` が `Some(params)` を返した
  ときは `owns(&params[i], &leaf)` が真のときだけ、`None` を返したときは常に `(args[i].name, leaf)` を
  積む。
- **(j)** `resolve_callee_params` が `None` を返すのは、`callee.name` が `vars.closure_targets` にも
  `prog.funcs` にも無いときである。
- **(k)** `rhs_consumes` の `RcRhs::Llvm(llvm_gen, args)` の腕は、`llvm_gen.borrows_operand(i, ..)` が真の
  オペランドを飛ばし、それ以外の各オペランド `args[i]` の各 boxed leaf `leaf` について、
  `passthrough_arg_leaves(llvm_gen, result_ty, args, type_env)` が `(i, leaf)` を含まないときだけ
  `(args[i].name, leaf)` を積む。
- **(l)** `passthrough_arg_leaves` は「結果のある leaf の宣言が単一の `Arg(j, p)` である」ような
  `(j, p)` の集合である。
- **(m)** 積まれるものの出どころは (c)、(d)、(h)、(i)、(k) の 5 か所で全部である。
- **(n)** `collect_consumes` を呼ぶ式はリポジトリに 1 つで、`infer_ownership` の中にある。
  `infer_ownership` を呼ぶ式も 1 つで、`borrow_ify` の中にあり、渡される `prog` は `borrow_ify` の
  第 1 引数である。よって `collect_consumes` とその走査が読む `prog` は、`borrow_ify` の入力プログラムに
  **等しい**。`rhs_consumes` にはもう 1 人の呼び出し元
  `CancelAnalysis::consume_rhs` があるが、そちらは `collect_consumes` を通らない。

<1>1. (a) が成り立つ。`collect_consumes` は `owns` を作って `collect_consumes_go` を呼ぶだけである。
  BY CODE src/rc_ir/ownership.rs: collect_consumes

<1>2. `RcExpr` は `Let`、`Retain`、`Release`、`Destructure`、`Eval`、`Ret` の 6 種である。
  BY <ref id=b3dfa37/>, CODE src/rc_ir/ast.rs: RcExpr

<1>3. `collect_consumes_go` の `match` の腕は `RcExpr::Ret(x)`、`RcExpr::Let(x, rhs, k)`、
      `RcExpr::Destructure(container, fields, _state, k)`、
      `RcExpr::Retain(..) | RcExpr::Release(..) | RcExpr::Eval(..)` の 4 つであり、`<1>2` の 6 種を尽くす。
  BY <1>2, CODE src/rc_ir/ownership.rs: collect_consumes_go

<1>4. `Ret` 以外の 3 つの腕は、いずれも継続 `k` について `collect_consumes_go` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go

<1>5. `Let` の腕は、`rhs` が `RcRhs::Match(_, arms)` のとき各 `arm.body` についても `collect_consumes_go` を
      呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Let` の腕の `RcRhs::Match` の場合

<1>6. `Ret` の腕は再帰しない。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret(x)` の腕

<1>7. (b) が成り立つ。
  <2>1. `collect_consumes_go(e, ..)` の走査は `e` の各節点をちょうど 1 度訪れる。
    D2 より本体は木であり、`Ret` を除く 5 種の節点の継続は 1 つ、分岐は `Match` のアームだけである。
    `<1>4` は `Ret` 以外の 3 つの腕がその継続を 1 度たどることを、`<1>5` は `Let` の腕が `RcRhs::Match`
    のとき各 `arm.body` を 1 度たどることを言い、`<1>6` が終端である。`<1>3` の 4 つの腕が `RcExpr` の
    6 種を尽くすので、これで `e` の節点が尽きる。
    BY <ref id=b3dfa37/>, <1>3, <1>4, <1>5, <1>6
  <2>2. 走査が訪れる `RcExpr::Ret` の節点は、`e` の終端の `Ret` か、走査が通った
        `Let(x, Match(_, arms), k)` のあるアームの `arm.body` の終端の `Ret` かのどちらかである。
    `<1>4` の継続への再帰は同じ本体の中を進み、`<1>6` より `Ret` で止まる。D2 より `Ret` は唯一の
    終端子なので、継続をたどって着く `Ret` はその本体の終端の `Ret` である。`<1>5` の `arm.body` への
    再帰は新しい本体を始めるので、その中で着く `Ret` はその `arm.body` の終端の `Ret` である。
    BY <ref id=b3dfa37/>, <1>4, <1>5, <1>6, <2>1
  <2>3. QED
    BY <2>1, <2>2

<1>7a. (b') が成り立つ。`infer_ownership` は `prog.funcs` の各関数について `collect_consumes` を
       `&func.body` を第 1 引数として呼ぶ。
  BY CODE src/rc_ir/borrow.rs: infer_ownership

<1>8. (c) が成り立つ。`RcExpr::Ret(x)` の腕は `push_boxed_leaves(&x.name, &x.ty, type_env, out)` を呼び、
      `boxed_leaf_paths(x.ty, type_env)` の各 `p` について `(x.name, p)` を積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Ret(x)` の腕, push_boxed_leaves

<1>9. (d) が成り立つ。`RcExpr::Destructure(container, fields, _state, k)` の腕は
      `destructure_consumes(container, fields, type_env)` の各 `leaf` について `(container.name, leaf)` を
      積む。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Destructure` の腕

<1>10. (e) が成り立つ。`destructure_consumes` は、`container.ty.is_box(type_env)` が真のとき
       `boxed_leaf_paths(container.ty, type_env)` の全部を返す。偽のときは、その各 `leaf` について
       `leaf.first()` を取り、それが `fields` の名前付きフィールドの添字の集合に入らないものだけを残す。
       **その `leaf.first()` の `expect` は発火しない** -- `L5a (a)` より、`is_box` が偽である本体の
       `Destructure` の容器の boxed leaf は空でない path を持つ。
  BY <ref id=9d4ff56/>, CODE src/rc_ir/ownership.rs: destructure_consumes

<1>11. (f) の前半が成り立つ。`RcExpr::Retain | RcExpr::Release | RcExpr::Eval` の腕は何も積まない。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の
     `RcExpr::Retain(..) | RcExpr::Release(..) | RcExpr::Eval(..)` の腕

<1>12. (f) の後半が成り立つ。`RcExpr::Let(x, rhs, k)` の腕は、`rhs` が `RcRhs::Match` のとき自身は何も
       積まず、それ以外の 4 種のとき `rhs_consumes(rhs, &x.ty, vars, prog, type_env, owns, out)` を呼ぶ。
  BY CODE src/rc_ir/ownership.rs: collect_consumes_go の `RcExpr::Let` の腕

<1>13. `RcRhs` は `Var`、`App`、`Closure`、`Llvm`、`Match` の 5 種である。
  BY <ref id=b3dfa37/>, CODE src/rc_ir/ast.rs: RcRhs

<1>14. `rhs_consumes` の `match` の腕は `RcRhs::Var(_) | RcRhs::Match(..)`、`RcRhs::Closure(_, caps)`、
       `RcRhs::App(callee, args)`、`RcRhs::Llvm(llvm_gen, args)` の 4 つであり、`<1>13` の 5 種を尽くす。
  BY <1>13, CODE src/rc_ir/ownership.rs: rhs_consumes

<1>15. (g) が成り立つ。`RcRhs::Var(_) | RcRhs::Match(..)` の腕は何も積まない。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Var(_) | RcRhs::Match(..)` の腕

<1>16. (h) が成り立つ。`RcRhs::Closure(_, caps)` の腕は、`caps` の各元 `c` について
       `boxed_leaf_paths(c.ty, type_env)` の各 `p` を `(c.name, p)` として積む。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Closure(_, caps)` の腕, push_boxed_leaves

<1>17. (i) が成り立つ。`RcRhs::App(callee, args)` の腕は、`callee` の全 boxed leaf を積み、さらに各引数
       `args[i]` の各 boxed leaf `leaf` について、`resolve_callee_params` が `Some(params)` を返したときは
       `owns(&params[i], &leaf)` が真のときだけ、`None` を返したときは常に `(args[i].name, leaf)` を積む。
       A14 より `params[i]` は範囲内である。
  BY <ref id=f8ae607/>, CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::App(callee, args)` の腕, push_boxed_leaves

<1>18. (j) が成り立つ。`resolve_callee_params` が `None` を返すのは、`callee.name` が
       `vars.closure_targets` にも `prog.funcs` にも無いときである。
  BY CODE src/rc_ir/ownership.rs: resolve_callee_params

<1>19. (k) が成り立つ。`RcRhs::Llvm(llvm_gen, args)` の腕は、`llvm_gen.borrows_operand(i, &arg_tys,
       type_env)` が真のオペランドを飛ばし、それ以外の各オペランド `args[i]` の各 boxed leaf `leaf` に
       ついて、`passthrough_arg_leaves(llvm_gen, result_ty, args, type_env)` が `(i, leaf)` を含まない
       ときだけ `(args[i].name, leaf)` を積む。
  BY CODE src/rc_ir/ownership.rs: rhs_consumes の `RcRhs::Llvm(llvm_gen, args)` の腕

<1>20. (l) が成り立つ。`passthrough_arg_leaves` は `llvm_gen.result_prov(result_ty, &arg_tys, type_env)` の
       各 leaf の `LeafOrigins` に `as_arg_projection` をかけて `Some((j, p))` になったものを集める。
       すなわち「結果のある leaf の宣言が単一の `Arg(j, p)` である」ような `(j, p)` の集合である。
  BY CODE src/rc_ir/ownership.rs: passthrough_arg_leaves, as_arg_projection,
     CODE src/rc_ir/provenance.rs: Provenance::leaves

<1>20a. (n) が成り立つ。
  <2>1. `collect_consumes` を呼ぶ式が在るのは `infer_ownership` であり、`infer_ownership` を呼ぶ式が
        在るのは `borrow_ify` であって、`borrow_ify` はそれに自分の第 1 引数 `prog` を渡す。
        `infer_ownership` の `prog` の型は `&RcProgram` である。`rhs_consumes` を呼ぶ式が在るのは
        `collect_consumes_go` と `CancelAnalysis::consume_rhs` である。
    `collect_consumes` と `rhs_consumes` は `pub(crate)`、`infer_ownership` は `borrow.rs` の
    非公開の関数なので、EXT 可視性 より、それらを呼ぶ式はこのクレートのソース `src/` の中にしかない。
    EXT 名前による数え上げ より、それらを呼ぶ式はその名前を含む。前提 消費の走査を呼ぶ在りか の走査が
    挙げる残りの項目が持つのは、各関数の宣言と、`collect_consumes_go` -- `collect_consumes` とは別の
    識別子 -- の宣言および自身への再帰である。
    BY 前提 消費の走査を呼ぶ在りか, EXT 可視性, EXT 名前による数え上げ,
       CODE src/rc_ir/ownership.rs: collect_consumes, collect_consumes_go, rhs_consumes,
       CODE src/rc_ir/borrow.rs: infer_ownership, borrow_ify, CancelAnalysis::consume_rhs
  <2>2. `collect_consumes` とその走査が読む `prog` は、`borrow_ify` の入力プログラムに等しい。
    `<2>1` より、`borrow_ify` は自分の第 1 引数を `infer_ownership` へ渡し、`infer_ownership` はそれを
    `&RcProgram` として受け取って `collect_consumes` へ渡す。EXT 借用規則 より、共有参照を通じて
    `RcProgram` の値やその欄に代入することはできず、書き換えられるのは内部可変性を持つ欄だけである。
    A3 の memo の節 --「`RcProgram` から到達できる値の等しさは、それを共有参照で受け取る計算が
    変えない」、到達できる型が内部可変性を持つ欄を持つときその欄は「一度だけ書かれる memo であって、
    その値はその型の `PartialEq` が読む成分の関数である」-- より、その書き換えは値の等しさを動かさない。
    BY <ref id=e11772a/>, EXT 借用規則, <2>1
  <2>3. QED
    BY <2>1, <2>2

<1>21. QED
  (a) は `<1>1`、(b) は `<1>7`、(b') は `<1>7a`、(c) は `<1>8`、(d) は `<1>9`、(e) は `<1>10`、(f) は
  `<1>11` と `<1>12`、
  (g) は `<1>15`、(h) は `<1>16`、(i) は `<1>17`、(j) は `<1>18`、(k) は `<1>19`、(l) は `<1>20`、
  (n) は `<1>20a` で
  ある。(m) は次のとおりである。`<1>3` の 4 つの腕のうち積むのは `Ret` (`<1>8`) と `Destructure`
  (`<1>9`) であり、`Retain | Release | Eval` は積まず (`<1>11`)、`Let` は自身では積まずに `rhs_consumes`
  を呼ぶ (`<1>12`)。`<1>14` の 4 つの腕のうち積むのは `Closure` (`<1>16`)、`App` (`<1>17`)、`Llvm`
  (`<1>19`) であり、`Var | Match` は積まない (`<1>15`)。
  BY <1>1, <1>3, <1>7, <1>7a, <1>8, <1>9, <1>10, <1>11, <1>12, <1>14, <1>15, <1>16, <1>17, <1>18,
     <1>19, <1>20, <1>20a

### P7 (a) D9 の消費はすべて報告される

この節は `own` が DEF leaf 粒度の所有 を満たすことを前提に置く。`own` を読む出どころは (i) だけで
あり (`L5 (i)`、`L5 (m)`)、その前提を使うのは `<1>1` の `<2>3` である。

<1>1. CASE D9 の `App(callee, args)` の行。
  <2>1. `App` は `RcRhs` の 1 種なので `Let(x, App(callee, args), k)` の形でだけ現れ、`L5 (f)` より
        `rhs_consumes` が呼ばれる。
    BY <ref id=b3dfa37/>, <ref id=c5547e4/> (f)
  <2>2. D9 の行の前半「callee の全 boxed leaf」は、`L5 (i)` の前半が積む。
    BY <ref id=9d74736/>, <ref id=c5547e4/> (i)
  <2>2a. `resolve_callee_params` が `Some(params)` を返すとき、`params` は D9 の `App` の行が言う
         呼び出し先 (D23) のパラメータの列である。
    D23 は「D9 の `App` の行と D10 の生成の `App` の行が「呼び出し先」と言うのは、この実行時の関数で
    ある」と定め、P29 が、`resolve_callee_params` が `Some(params)` を返すならば `params` はその段の
    実行時の呼び出し先のパラメータの列であると述べる。P29 は `borrow_ify` の入力の `App` についての
    言明であり、`L5 (n)` よりこの呼び出しが走るのは `borrow_ify` の入力の本体についてである。
    BY <ref id=ff5985d/>, <ref id=c5547e4/> (n), <ref id=7a4d9dc/>
  <2>2b. `L5 (i)` が回る `leaf` は `ty(params[i])` の boxed leaf でもあり、`(params[i], leaf)` について
         DEF leaf 粒度の所有 の対応が定まる。
    `L5 (i)` が回るのは `boxed_leaf_paths(ty(args[i]), type_env)` の元である。A12 の
    「`App(callee, args)` の各引数と呼び出し先の対応するパラメータの型」の行より
    `ty(args[i]) = ty(params[i])` なので、`leaf` は `ty(params[i])` の boxed leaf でもある。
    `ty(params[i])` はプログラムに現れる型なので A10 を満たし、P1 より
    `truncate_to_unit(ty(params[i]), leaf, type_env)` は `rc_units(ty(params[i]))` の要素である。
    BY <ref id=8412761/>, <ref id=83d98e9/>, <ref id=c5547e4/> (i), <ref id=3597669/>, DEF leaf 粒度の所有, <2>2a
  <2>3. D9 の行の後半「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」は、`resolve_callee_params` が
        `Some(params)` のとき `owns(&params[i], &leaf)` が積む。`<2>2a` より `params` は D9 の行が言う
        呼び出し先のパラメータであり、`<2>2b` より DEF leaf 粒度の所有 の対応が定まる。その対応より、
        この述語は「`params[i]` の leaf `leaf` の unit が D14 の意味で所有される」ことと同値である。
    BY <ref id=9d74736/>, <ref id=ef8efc4/>, DEF leaf 粒度の所有, <ref id=c5547e4/> (i), <2>2a, <2>2b
  <2>4. `resolve_callee_params` が `None` のとき、`L5 (i)` は各引数の**全** boxed leaf を積む。D9 の行の
        後半が指すのは引数の boxed leaf の一部なので、積まれるものはそれを尽くす。
    BY <ref id=9d74736/>, <ref id=c5547e4/> (i)
  <2>5. QED
    `<2>2` が D9 の行の前半を、`<2>3` と `<2>4` が後半を、`resolve_callee_params` の 2 つの場合ごとに
    与える。
    BY <2>1, <2>2, <2>2a, <2>2b, <2>3, <2>4
<1>2. CASE D9 の `Closure(f, caps)` の行。D9 のこの行「各 capture の全 boxed leaf」は `L5 (h)` が積む。
      `Closure` は `RcRhs` の 1 種なので `L5 (f)` より `rhs_consumes` が呼ばれる。
  BY <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=c5547e4/> (f), <ref id=c5547e4/> (h)
<1>3. CASE D9 の `Llvm(gen, args)` の行。D9 のこの行「`borrows_operand(i)` が偽のオペランドのうち、
      `result_prov` が**単一の** `Arg(i, σ)` として素通しを宣言していない leaf」は `L5 (k)` の条件そのもので
      あり、その「単一の `Arg(i, σ)`」は `L5 (l)` の条件そのものである。`L5 (l)` の
      `passthrough_arg_leaves` が読む `result_prov` の値と、D9 の行が言う宣言が同じものであることは、
      A3 の決定性の節 -- `result_prov` は同じ引数に対して常に同じ値を返す -- による。`Llvm` は `RcRhs` の
      1 種なので `L5 (f)` より `rhs_consumes` が呼ばれる。
  BY <ref id=e11772a/>, <ref id=b3dfa37/>, <ref id=9d74736/>, <ref id=c5547e4/> (f), <ref id=c5547e4/> (k), <ref id=c5547e4/> (l)
<1>4. CASE D9 の `Destructure(c, fs)` (`c` が boxed) の行。D9 のこの行「`c` の全 boxed leaf」は
      `L5 (e)` の `is_box` が真の場合が返し、`L5 (d)` が積む。
  BY <ref id=9d74736/>, <ref id=c5547e4/> (d), <ref id=c5547e4/> (e)
<1>5. CASE D9 の `Destructure(c, fs)` (`c` が unbox) の行。D9 のこの行「名前が付いていない
      フィールドの leaf」の全体は、`L5a (b)` より `L5 (e)` の `is_box` が偽の場合が返す集合に等しく、
      それを `L5 (d)` が積む。
  BY <ref id=9d74736/>, <ref id=c5547e4/> (d), <ref id=c5547e4/> (e), <ref id=9d4ff56/>
<1>6. CASE D9 の「本体 (D23) の終端の `Ret(x)`」の行。
  <2>1. `collect_consumes` に渡された式は関数の `body` であり、D23 よりそれは本体である。
    P7 の前提が第 1 引数を関数の本体に限り、`L5 (b')` と `L5 (n)` が `collect_consumes` の呼び出しが
    その形のものだけであることを与える。
    BY <ref id=ff5985d/>, <ref id=c5547e4/> (b'), <ref id=c5547e4/> (n)
  <2>2. その本体の終端の `Ret(x)` は `RcExpr::Ret` の節点であり、`L5 (b)` より走査はそれを訪れる。
        `L5 (b)` の 2 つの場合のうち第 1 の場合 (渡された式の終端の `Ret`) がこの節点である。
    BY <ref id=ca36627/>, <ref id=c5547e4/> (b), <2>1
  <2>3. D9 のこの行「`x` の全 boxed leaf」は `L5 (c)` が積む。
    BY <ref id=9d74736/>, <ref id=c5547e4/> (c)
  <2>4. QED
    BY <2>1, <2>2, <2>3
<1>7. QED
  D9 の消費の表は 6 行からなり、`<1>1` から `<1>6` がその 6 行である。
  BY <ref id=9d74736/>, <1>1, <1>2, <1>3, <1>4, <1>5, <1>6

### P7 (b) 余分に報告されるのはアーム本体の `Ret` に限る

この節は `own` に条件を置かない。`own` を読む出どころは (i) だけであり (`L5 (i)`、`L5 (m)`)、`<1>3` は
その出どころについて `own` を渡って主張する。

<1>1. `L5 (m)` の出どころ (d) が積むものは D9 の `Destructure` の 2 行のいずれかである。`L5 (e)` の
      2 つの場合は `container.ty.is_box(type_env)` の真偽で尽きている。真の場合が返すのは `ty(c)` の
      全 boxed leaf であって D9 の `Destructure` (boxed) の行に等しく、偽の場合が返す集合が D9 の
      `Destructure` (unbox) の行が指す leaf に等しいことは `L5a (b)` である。
  BY <ref id=9d74736/>, <ref id=c5547e4/> (d), <ref id=c5547e4/> (e), <ref id=c5547e4/> (m), <ref id=9d4ff56/>

<1>2. 出どころ (h) が積むものは D9 の `Closure` の行である。`L5 (h)` が積むのは各 capture の全 boxed
      leaf であり、D9 の `Closure` の行と同じである。
  BY <ref id=9d74736/>, <ref id=c5547e4/> (h), <ref id=c5547e4/> (m)

<1>3. 出どころ (i) が積むものは、`own` が何であれ、D9 の `App` の行が指す leaf である。
  <2>1. `L5 (i)` が積むのは、callee の全 boxed leaf と、各引数 `args[i]` の boxed leaf のうち
        `resolve_callee_params` が `Some(params)` のときは `owns(&params[i], &leaf)` が真のもの、`None` の
        ときは全部である。どちらの場合も、積まれる引数 leaf は
        `boxed_leaf_paths(ty(args[i]), type_env)` の部分集合である。
    BY <ref id=c5547e4/> (i)
  <2>2. callee の全 boxed leaf は D9 の `App` の行の前半である。
    BY <ref id=9d74736/>, <2>1
  <2>3. この `collect_consumes` の呼び出しは `borrow_ify` の入力プログラムの関数の本体についてのもので
        あり、それが読む `prog` は `borrow_ify` の入力プログラムに等しい。
    BY <ref id=c5547e4/> (b'), <ref id=c5547e4/> (n)
  <2>4. D9 の `App` の行の後半が指すのは、各引数の**全** boxed leaf である。
    <3>1. D9 の `App` の行が言う呼び出し先 -- D23 より、その段の実行時の関数 -- は、プログラムの
          `funcs` の関数である。D23 は、D9 の `App` の行が読む所有を D14 が `RcFunc::borrowed_units` から
          定めることを理由に、これを本文で述べる。
      BY <ref id=9d74736/>, <ref id=ef8efc4/>, <ref id=ff5985d/>
    <3>2. `<2>3` と A1 より、`prog` のすべての関数の `borrowed_units` は空である -- A1 は
          `borrow_ify` に渡されるプログラムについての仮定であり、`prog` はその値に等しい。D14 より
          借用する unit の集合が `borrowed_units` で残りが所有する unit なので、どの関数もその全
          パラメータの全 unit を所有する。
      BY <ref id=627e117/>, <ref id=ef8efc4/>, <2>3, <3>1
    <3>3. QED
      `<3>1` の呼び出し先は `<3>2` の関数の 1 つなので、その位置の unit をすべて所有する。よって D9 の
      行の後半「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」は、各引数の全 boxed leaf で
      ある。
      BY <ref id=9d74736/>, <3>1, <3>2
  <2>5. QED
    `<2>1` の引数 leaf は `<2>4` が挙げる集合の部分集合であり、`<2>2` の callee leaf は D9 の行の前半で
    ある。よって積まれるものはすべて D9 の `App` の行が指す leaf である。
    BY <2>1, <2>2, <2>4

<1>4. 出どころ (k) が積むものは D9 の `Llvm` の行である。`L5 (k)` が積むのは `borrows_operand(i)` が偽の
      オペランドの leaf のうち `passthrough_arg_leaves` に入らないものであり、`L5 (l)` よりその条件は
      「結果のどの leaf の宣言も単一の `Arg(i, leaf)` でない」ことである。これは D9 の `Llvm` の行と
      同じである -- `L5 (l)` の `passthrough_arg_leaves` が読む `result_prov` の値と D9 の行が言う宣言が
      同じものであることは、A3 の決定性の節による。
  BY <ref id=e11772a/>, <ref id=9d74736/>, <ref id=c5547e4/> (k), <ref id=c5547e4/> (l), <ref id=c5547e4/> (m)

<1>5. 出どころ (c) が積むものは、その `Ret` 節点が `collect_consumes` に渡された式の終端のものなら D9 の
      `Ret` の行であり、そうでないなら `Match` のアーム本体の終端の `Ret` である。
  <2>1. 走査が訪れる `RcExpr::Ret` の節点は、`collect_consumes` に渡された式の終端の `Ret` か、走査が
        通った `Let(x, Match(_, arms), k)` のあるアームの `arm.body` の終端の `Ret` かのどちらかである。
    `L5 (a)` より `collect_consumes` は渡された式について `collect_consumes_go` を呼ぶ。
    BY <ref id=c5547e4/> (a), <ref id=c5547e4/> (b)
  <2>2. `collect_consumes` に渡された式は関数本体である。P7 の前提 (第 1 引数が関数の本体であること) が
        それであり、`L5 (b')` がその形の呼び出しであることを与える。
    BY <ref id=c5547e4/> (b')
  <2>3. QED
    `<2>1` の第 1 の場合は、`<2>2` より関数本体の終端の `Ret` であり、D9 の終端の `Ret` の行に当たる。
    第 2 の場合は `Match` のアーム本体の終端の `Ret` である。
    BY <ref id=ca36627/>, <ref id=9d74736/>, <2>1, <2>2

<1>6. QED
  出どころ (d)、(h)、(k) が積むものは D9 の消費の行そのものであり (`<1>1`、`<1>2`、`<1>4`)、(i) が積む
  ものは `own` が何であれ D9 の `App` の行が指す leaf であり (`<1>3`)、(c) が積むものは関数本体の終端の
  `Ret` (D9 の行) か `Match` のアーム本体の終端の `Ret` かのどちらかである (`<1>5`)。`L5 (m)` より
  出どころはこの 5 つで全部である。よって積まれるもののうち D9 の消費の表に無いものは、`Match` の
  アーム本体の終端の `Ret` が積むものに限る。
  BY <ref id=c5547e4/> (m), <1>1, <1>2, <1>3, <1>4, <1>5

### L6 (報告しない箇所は D9 の消費ではない) <!--#03f9c40-->

**言明**。関数の本体を第 1 引数に渡し、DEF leaf 粒度の所有 を満たす `own` を渡した `collect_consumes` の
呼び出しについて、それが報告しない箇所は、いずれも D9 の消費ではない。それらが義務集合 `Obl` に
対して行うのは、何もしないか、D9 の移動 (`Obl` を変えない) か、D10 が `Retain` / `Release` の行で直接
定める増減かのどれかである。

<1>1. `rhs_consumes` の `RcRhs::Var(_)` の腕 (`L5 (g)`)。D9 の消費の表の 6 行 (`App`、`Closure`、
      `Llvm`、`Destructure` の 2 行、終端の `Ret`) に `Var` の行は無いので、これは消費ではない。
      `Let(x, Var(y), k)` は D9 の移動の表の第 1 行であり、`y` の参照は活性化の中で `x` へ移る。
      D10 の移動の行より `Obl` は変わらない。
  BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (g)

<1>2. `rhs_consumes` の `RcRhs::Match(..)` の腕 (`L5 (g)`)、および `collect_consumes_go` の `RcExpr::Let` の
      腕の `RcRhs::Match` の場合 (`L5 (f)`)。D9 の消費の表に `Match` の行は無いので、これは消費ではない。
      D9 は `Match` 節点自身が参照を作らず、移さず、手放さないと述べる。D10 は `Obl` を動かす事象を
      `Retain`・`Release`・生成・消費・移動で尽くすので、この節点は `Obl` を変えない。アームの
      中の消費は `L5 (b)` の再帰が報告する。
  BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (b), <ref id=c5547e4/> (f), <ref id=c5547e4/> (g)

<1>3. `collect_consumes_go` の `RcExpr::Retain` の腕 (`L5 (f)`)。D9 の消費の表に `Retain` の行は無いので、
      これは消費ではない。`Retain` は D10 が直接扱う構文であり、D10 の `Retain` の行は `Obl` への追加で
      ある。
  BY <ref id=ec8d1a0/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (f)

<1>4. `collect_consumes_go` の `RcExpr::Release` の腕 (`L5 (f)`)。D9 の消費の表に `Release` の行は
      無いので、これは消費ではない。`Release` は参照を処分するが、D10 は `Release` の行を消費の行とは
      別に持ち、その増減を直接定める。
  BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (f)

<1>5. `collect_consumes_go` の `RcExpr::Eval` の腕 (`L5 (f)`)。D9 の消費の表に `Eval` の行は無いので、
      これは消費ではない。D9 は `Eval(v, k)` が参照を作らず、移さず、手放さないと述べる。D10 は `Obl` を
      動かす事象を `Retain`・`Release`・生成・消費・移動で尽くすので、この節点は `Obl` を変えない。
      D7 の読む構文の表には入っているが、読みは `Obl` を変えない。
  BY <ref id=56c2068/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (f)

<1>6. `rhs_consumes` の `RcRhs::Llvm` の腕が `borrows_operand(i)` が真のときに飛ばすオペランド
      (`L5 (k)`)。D9 の `Llvm` の行は消費を `borrows_operand(i)` が偽のオペランドに限るので、これは
      消費ではない。A3 が「`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分
      しない」と置き、D10 は `Obl` を動かす事象を `Retain`・`Release`・生成・消費・移動で尽くすので、
      このオペランドについて `Obl` は変わらない。
  BY <ref id=e11772a/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (k)

<1>7. `rhs_consumes` の `RcRhs::Llvm` の腕が `passthrough` に入るとして飛ばす leaf (`L5 (k)`, `L5 (l)`)。
      D9 の `Llvm` の行は消費から素通し leaf を外しているので、これは消費ではない -- `L5 (l)` の
      `passthrough_arg_leaves` が読む `result_prov` の値と D9 の行が言う宣言が同じものであることは、
      A3 の決定性の節による。A3 の表の「単一の
      `Arg(j, σ)`」の行が、生成コードはそこに第 `j` オペランドの leaf `σ` と同じ参照を置き、新しい参照を
      作らないと述べる。D9 の移動の表の最後の行がこれを移動とし、D10 の移動の行より `Obl` は変わらない。
  BY <ref id=e11772a/>, <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (k), <ref id=c5547e4/> (l)

<1>8. `destructure_consumes` が unbox 容器について落とす名前付きフィールドの leaf (`L5 (e)`)。
      `L5a (b)` より `L5 (e)` が返す集合は D9 の `Destructure` (unbox) の行が指す leaf に等しいので、
      落とされる leaf は名前が付いたフィールドの leaf である。D9 のその行は消費を名前が付いていない
      フィールドの leaf に限るので、これは消費ではない。D9 の移動の表の第 3 行がこれを移動とし、
      D10 の移動の行より `Obl` は変わらない。
  BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=c5547e4/> (e), <ref id=9d4ff56/>

<1>8a. `rhs_consumes` の `RcRhs::App` の腕が、`resolve_callee_params` が `Some(params)` を返し
       `owns(&params[i], &leaf)` が偽のときに積まない引数 leaf (`L5 (i)`)。D23 は D9 の `App` の行が言う
       「呼び出し先」を実行時の関数と定め、P29 は、`resolve_callee_params` が `Some(params)` を返すならば
       `params` はその実行時の呼び出し先のパラメータの列であると述べる。`L5 (n)` よりこの呼び出しは
       `borrow_ify` の入力の本体について走るので、P29 をそのまま当てられる。DEF leaf 粒度の所有 より、
       この述語が偽であることは、`params[i]` のその leaf の unit を呼び出し先が**借用する** (D14) ことと
       同値である。D9 の `App` の行は消費を呼び出し先が所有する位置の leaf に
       限っているので、これは消費ではない。D14 より借用する unit の参照は呼び出し元が処分し、D10 は
       `Obl` を動かす事象を `Retain`・`Release`・生成・消費・移動で尽くすので、この leaf について
       `Obl` は変わらない。
  BY <ref id=9d74736/>, <ref id=f06144e/>, <ref id=ef8efc4/>, <ref id=ff5985d/>, DEF leaf 粒度の所有, <ref id=c5547e4/> (i), <ref id=c5547e4/> (n), <ref id=7a4d9dc/>

<1>9. QED
  報告しない箇所は次で全部である。`L5 (m)` より積む出どころは L5 の (c)、(d)、(h)、(i)、(k) の 5 つ
  なので、報告しない箇所は (1) 積まない腕 -- `collect_consumes_go` の `Retain | Release | Eval` の腕
  (`<1>3`、`<1>4`、`<1>5`)、`Let` の腕の `RcRhs::Match` の場合 (`<1>2`)、`rhs_consumes` の
  `Var | Match` の腕 (`<1>1`、`<1>2`) -- と、(2) 積む腕の中で落とされる leaf --
  `destructure_consumes` が unbox 容器について落とす名前付きフィールドの leaf (`<1>8`)、`App` の腕が
  `owns` の偽で落とす引数 leaf (`<1>8a`)、`Llvm` の腕が `borrows_operand` で飛ばすオペランド
  (`<1>6`) と `passthrough` で落とす leaf (`<1>7`) -- である。L5 の (c) と (h) は落とす条件を持たない。
  `<1>1` から `<1>8a` は、そのそれぞれについて、D9 の消費でないこと、および `Obl` への働きが「何も
  しない」「D9 の移動」「D10 の `Retain` / `Release` の行」のどれかであることを述べている。
  BY <ref id=c5547e4/> (c), <ref id=c5547e4/> (h), <ref id=c5547e4/> (m), <1>1, <1>2, <1>3, <1>4, <1>5, <1>6, <1>7, <1>8, <1>8a

### P7 の結論

<1>1. README の P7 の第 1 文 --「関数の本体を第 1 引数に渡し、**D14 の所有をちょうど報告する `own` を渡した**
      `collect_consumes` の呼び出しについて、D9 の意味で消費する構文はすべてそれが報告する」--
      が成り立つ。
  「D14 の所有をちょうど報告する `own`」が DEF leaf 粒度の所有 である。P7 (a) がその `own` について、
  D9 の消費の表の各行が指す leaf を `collect_consumes` が積むことを与える。
  BY <ref id=5b24ac0/> (a), DEF leaf 粒度の所有
<1>2. README の P7 の第 2 文 --「また報告して D9 が消費としないものは、アーム本体の `Ret` に限る」--
      が成り立つ。とくに README が続けて置く「どの `own` についても」の形でも成り立つ。
  P7 (b) は `own` に条件を置かずにこれを与える。第 1 文の `own` はその特別な場合である。
  BY <ref id=5b24ac0/> (b)
<1>3. QED
  README の P7 は 2 つの文からなり、第 1 文が `<1>1`、第 2 文が `<1>2` である。
  BY <1>1, <1>2

## 9. `README.md` との突き合わせ

### 定義が定めるのは語の意味だけである

`README.md` の第 3 節は「定義の中に、支えの要る主張を置かない」と定める。この文書の `DEF` が定めるのは
語の意味だけであり、その語について示すことは段が持つ -- DEF 路の位置 の数え上げと `obj` の一意性は
L0b、DEF 辺の leaf 対応 が D9 の値の水準の 6 行と一致することは L1 の `<1>1a`、DEF 辺の存在 と
DEF `ρ` の上で実行された辺 の連言が D20 の「辺が在る」と一致することは L1 (b)、
DEF 名前の指すオブジェクト の `ν` が写像として定まることは P6 (b) の `<1>0`、DEF leaf 粒度の所有 の
対応が定まることは P7 (a) の `<2>2b` である。

### D9 の値の水準の行を leaf の粒度で読む規則

D9 の移動の表も値の水準の 6 行も、構文の粒度で書かれている -- 「フィールド変数の値は容器の値のその
フィールドである」。ところが `origin` も `Retain`/`Release` も leaf の粒度で動くので、この行を使う証明は
「始点のどの leaf が終点のどの leaf に対応するか」を要る。この文書は第 2 節の DEF 辺の leaf 対応 でそれを
与え、L1 の `<1>1a` がそれを D9 の値の水準の 6 行と突き合わせる。E1 から E6 の 6 行はいずれも、対応する
leaf を行の文言そのものから読める。

`Llvm` の行は、対にする leaf を宣言の path として名指す -- 「結果の leaf `λ` の宣言が単一の
`Arg(i, σ)` であるとき、その leaf の値は**オペランド `i` の leaf `σ` の値**である」であり、続けて
「**`λ` と `σ` は一般に別の path である**」と述べる。すなわち `λ` と対にする leaf を `σ` と名指して
いるのは、この文書ではなく D9 の本文である。

### オブジェクトの同一性は参照ではなく値が運ぶ

D26 が A5 を計数下のオブジェクトへ制限した結果、「同じ参照を持つ 2 つの leaf は同じオブジェクトを指す」の
形の議論は両端が計数下であるときにしか通らない。**この文書の L1 は A5 を使わない。** 使うのは D9 の値の
水準の 6 行と、`obj(x, λ)` が `x` が得る値と `λ` だけで決まる量であること (L0b (c)、D6) である。
したがって L1 (a) に載る P5 (a) と P6 (b) は、両端が計数下であるかどうかに依らずに成り立つ。
P5 (b) は L1 を読まない -- その証明が命題として引くのは L2 だけである。

`Let(x, Var(g), k)` (`g` はグローバル値) がその境目である。`(g, [])` は記号の位置 (D6) であって D8 の
意味の参照を持たず、`(x, [])` の指すオブジェクトもグローバル状態なので参照を持たない (D26)。よって参照を
経由する議論は何も言わない。値を経由する議論は、D9 の値の水準の第 1 行 -- 「`x` の値は `y` の値で
ある」-- をこの節点に当てて、`obj(x, []) = obj(g, [])` を直ちに出す。この形が、L1 (a)・L4・P6 (b) を `ρ` のスロットではなく `ρ` の位置の
上で述べる理由である -- `origin` の再帰は記号の位置で終わるので、スロットだけを渡る言明では帰納が閉じない。

### P5 (b) の「同じオブジェクトを指す」は使っていない

P5 (b) の証明は前提のうち別名の道の条件と、解析が 2 つの鍵で `origin` を呼ぶことだけを使う。E1 から E5 の
別名の道で結ばれた 2 つのスロットは、オブジェクトが同じかどうかを問わず `identity` が等しい。

### 別名の辺の両端をスロットではなく位置で述べること

D20 は「別名の辺の両端のスロットは、同じオブジェクトを指す」を述べる。L1 (a) はその文を 1 点で強めた形を
示す。主語が `ρ` のスロットではなく `ρ` の位置 (D6) であることがそれで、別名の辺は記号の位置へ着きうる
ので、スロットだけを渡る言明では L4 の帰納が閉じない。**辺の端を位置に取ること自体は D20 が
本文で述べる** -- 「**辺の端はスロットに限らず、位置 (D6) である。**」-- が、上に引いた文の主語は
「両端のスロット」である。L4 が使うのはこの強めた形である。

**辺の選び方は D20 と同じである。** L1 (a) が主語に置くのは、この文書の意味で在り、かつ `ρ` の上で
実行された辺であり、L1 (b) よりこれは D20 の意味で在る辺の全体である -- D20 の存在の条件が
「アームの中の行 (変位アームの payload 束縛、catch-all アームの payload 束縛、アーム本体の `Ret`) に
ついては、路がそのアームを選ぶことも要る」を既に持つ。すなわちこの軸では L1 (a) は D20 より広くも
狭くもない。

**辺が在る条件は D20 が持ち、この文書はそれを 2 つの語に割る。** D20 は、辺が在るのは、その辺を定める
節点が実行路の上に在り、かつその節点と leaf `λ` が作る 2 つの対がどちらもその路の位置であるときで
あり、アームの中の行については路がそのアームを選ぶことも要る、と述べる。この文書は、2 つの対が `ρ` の
位置であることを DEF 辺の存在 に、節点が `ρ` の上に在ることと路がそのアームを選ぶことを
DEF `ρ` の上で実行された辺 に置く。L4 の各段が 2 つを別に言うのは、`ρ` の位置であることを段ごとに
示しているからである -- `λ` が終点の型の boxed leaf であって inhabited であること
(A12 と D9 と D16)、始点の path が始点の型の boxed leaf であって inhabited であること、そして両端の
変数が `ρ` の上で値を得ていること (`<1>3`) である。**辺の端が位置であることは D20 が本文で述べ**、
その根拠に D6 -- 別名の道が記号の位置で終わること -- を挙げる。

### 「条件を外した形」の 2 つの節と `identity` の側

P3 は条件を外した形で「**条件を外した形では、2 つの位置 (D6) は同じオブジェクトを指す。**」を述べ、
P4 は「**条件を外した形では、その位置 (D6) と対応する位置のいずれかは同じオブジェクトを指す。**」を
述べる -- P4 の側は選言である。**P5 (a) が要るのは `identity` の側であり、そこに P3 は届かない** --
`README.md` の P3 は「**この節は P5 (a) を支えない。**」を持ち、その理由に「P5 (a) が要るのは
`identity` の側についてのオブジェクトの一致であり、この節が名指すのは対応する位置である」を挙げる。P3 の主語は
`Exactly(u, σ)` の `(u, σ)` であって、対応する位置の path は `σ` の下の leaf であって `σ` そのものとは
限らない。P4 の主語は `candidates` の側であり、`Join` の `identity` は `candidates` に
入るとは限らない (`Origin::of_candidates` は `identity` に `here` を据える)。L4 の (ii) は `identity` に
ついて両方の場合を覆うので、この文書は P5 (a) を L4 に載せる -- `README.md` の P3 も
「P5 (a) は `p12-identity-and-consumes.md` の `L4` が `identity` について直に示す」と書く。

L4 は (i) -- `identity` の path が `ty(w)` の boxed leaf であり、`(w, σ)` が `ρ` の位置であること -- も
同じ帰納で示す。P3 の言う「対応するスロット」の path は `σ` の下の leaf であって `σ` そのものとは
限らないので (P3 の本文)、`identity` の位置を名指すにはこの (i) が要る。

### P5 (a) が載っている仮定 -- A3 の「複数の元」の行

P5 (a) の証明は L4 を通り、L4 は `result_prov` が leaf に置く集合の元数が 0 か 1 であること (A3) に
載っている。`README.md` の A3 はこの経路を本文に持ち、その節を果たすのは `validate` の `check_rhs` の
develop mode の検査である。`README.md` の「仮定」の節は、その強さについて「`develop_mode` でだけ走る
表明は 3 段目より弱い」と述べる。

### `is_array` / `is_funptr` と `variant` を突き合わせるには `type_env` が要る

`TypeNode::is_array` と `TypeNode::is_funptr` は最上位の tycon の**名前**で決まるのに対し、`is_struct` と
`is_union` は `toplevel_tycon_info(type_env)` が返す項の `variant` で決まる。2 つを突き合わせる段 --
L4 の `<1>9` `<3>2`、L4 の `<1>12` `<3>2`、L5a の `<1>1` -- は、その実行の `TypeEnv` の `Std::Array` の
項と `is_funptr_tycon` を満たす鍵の項が `bulitin_tycons` が入れたものであることを要る。L3a がそれを、
`Program::calculate_type_env` の種と、`TypeEnv` の `tycons` の欄に書く 6 つの式の数え上げから示す。
**funptr の側を範囲 (`1` から `FUNPTR_ARGS_MAX`) でなく述語で書くのは、`is_funptr` がその述語で決まり、
範囲の外の `#FunPtr{n}` にも真を返すからである。**

### 前件を、解析が `origin` を呼ぶ鍵に限ること

`README.md` の P3・P4・P5 (a)・P5 (b) が持つ、解析がその鍵で `origin` を呼ぶという前件を、この文書は L4 の
言明にも置いた。**言明を絞る道を取ったのは、L4 の帰納が `⇝` の整礎性 (L0a (b)) の上に立ち、その整礎性が呼び出しの
在る鍵についてしか言えないからである。** 絞らずに `ρ` のすべての位置を渡る形にすると、解析が呼ばない鍵に
ついて `id` の値を引くことになる。帰納の各段は、後続の鍵についてこの前件を立て直す -- E1 から E5 の辺の
場合は L2 が、`Join` の場合は L0a (a) が、それを与える。**下流は狭まらない** -- L4 を読む P5 (a) は
README の同じ前件から、P6 (b) は P6 (a) の前半から、前件を得る。

### A12 の「構造体である」「union である」の読み

L4 の `<1>9` と `<1>12` と L5a の `<1>1` は、`ty(c)` と `ty(s)` がクロージャでないことを A12 から出す。
A12 は「この仮定が型の `variant` を述べる各節では、その型の `is_closure()` は偽である」を持ち、
`Match` の scrutinee が union であることと `Destructure` の容器が構造体であることをその節に数える。
この 3 つの段はその節をそのまま引く。A12 がその節の理由に置くのは、`is_union` も `is_struct` も
`toplevel_tycon_info` を通り、それが `assert!(!self.is_closure())` で始まることである。

### D9 の `App` の行と `collect_consumes` の粒度が違う

D9 の `App` の行は「呼び出し先がその位置の unit を所有する (D14) 引数の leaf」と **unit** の粒度で述べ、
`collect_consumes` の `owns` は leaf 粒度の集合への所属である
(`CODE src/rc_ir/ownership.rs: collect_consumes`, `CODE src/rc_ir/borrow.rs: OwnedLeaves`)。P7 は
DEF leaf 粒度の所有 でこの 2 つを橋渡ししている。`README.md` の P8 は、同じ食い違いのために `App` の
引数の位置を言明から除き、その位置は `call_rc` が置く節点で扱うとしている。

### 第 1 引数が関数の本体であること

D9 の消費の表の最後の行は「本体 (D23) の終端の `Ret(x)`」であり、D23 の「本体」は関数の `body` と
グローバル初期化子の `init` の両方を指す。この文書が示すのは前者を渡した呼び出しについてであり、
`collect_consumes` の呼び出しがその形のものだけであることは `L5 (b')` と `L5 (n)` -- 呼び出しは
`infer_ownership` が `prog.funcs` の各関数の `func.body` について行うものだけである -- から取る。
`README.md` の P7 の項も同じ限定を同じ根拠で置く。
