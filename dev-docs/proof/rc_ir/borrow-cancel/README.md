# `borrow_ify` と `cancel` が RC 規律を保存することの証明

## 1. 対象

**この文書が証明するのは「保存 (preservation)」である。**「意味の保存 (correctness)」ではない。証明するのは
「入力が RC 規律 (D11) を満たすなら、出力も満たす」であって、「出力が入力と同じものを計算する」ではない。
compiler verification の慣行では、パスが意味を保つことを *correct* と呼び、*sound* は解析や論理について言う。
この文書で `健全` を使うのは `origin` と `infer_ownership` -- すなわち解析 -- についてだけである。

証明しないものを名前で挙げておく。**評価順の保存**、**FFI の副作用の列の保存**、**返り値の一致**。
これらは D11 の軸の上に無い。

**この証明は書きかけである。** 第 7 節の表が、どの命題がどこまで進んでいるかを述べる。第 8 節が、証明を書く
作業が見つけたコードの欠陥を述べる。

証明の対象は、コミット `39c41033cb436ea752b3a1231f67321a45dd1f87` の

- `src/rc_ir/borrow.rs` の `borrow_ify` と `cancel`、およびこの 2 つが呼ぶ同ファイル内の関数
- `src/rc_ir/ownership.rs` の全体 (この 2 つが参照の同一性と消費を決めるのに使うモデル)

である。`split_rc_units` は同じファイルに住むが、`borrow_ify` の前段であって対象ではない。その出力の性質は
仮定 A3 として置く。

この 2 つは、`src/build/build_object_files.rs` の `optimize_rc_program` の中で、`-O max` 以上のとき
`split_rc_units` の直後にこの順で走る。

対象コミットは、第 8 節が述べる欠陥のうち 5 件を直した後のものである。#551 の修正は `levelling-assert`
ブランチにあり、まだ main に入っていない。**#552 は直していない** -- 修正を書いて測ったところ実ベンチの
命令数が最大 +9.73% 増えたので外した。第 8 節がその経緯を述べる。

## 2. 証明の記法

各命題の証明は、`.claude/skills/code-proof/SKILL.md` が定める構造化証明の記法で書く。要点だけ再掲する。

- ステップは `<k>n.` で始まる。`k` は入れ子の深さ、`n` はその深さでの番号。各ステップの証明は、`<k+1>` の
  ステップの列 (最後は必ず `QED`) か、`BY` の 1 行かのどちらかである。
- `BY` の行に、そのステップが依拠するものを全部並べる。`D<n>` (定義)、`A<n>` (仮定)、`P<n>` (命題)、
  `<k>n` (同じ証明のステップ)、`DEF <名前>` (本文を展開した定義)、`CODE <ファイル>: <記号>` (読んだコード)。
- `<3>4` の証明から参照してよいのは、各祖先の深さの**先行する兄弟**だけである。兄弟の証明の内部は参照できない。
- ステップを後から挟むときは `<1>1a` と枝番を振る。番号は振り直さない。

## 3. 定義

定義は依存の順に並べる。番号は導入の順ではなく、文書中で固定された名札である。

### 3.1 中間表現

**D1 (プログラム)**
RC IR のプログラムとは、次の 3 つ組である (`CODE src/rc_ir/ast.rs: RcProgram`)。

- `funcs`: 名前 (`FuncRef`) から関数への写像。
- `globals`: グローバル値の初期化子の列。
- `roots`: 外から到達される名前の集合。

関数 `RcFunc` は 9 個のフィールドを持つ (`CODE src/rc_ir/ast.rs: RcFunc`)。`name`、`fn_ty`、`params`
(パラメータの列)、`capture` (無い場合がある)、`ret_ty`、`body` (本体)、`source`、`borrowed_units` (借用する
unit の集合)、`inline_into_callers`。グローバル初期化子 `RcGlobalInit` は 5 個のフィールドを持つ
(`CODE src/rc_ir/ast.rs: RcGlobalInit`)。`symbol`、`ty`、`init` (パラメータも capture も持たない本体)、
`owns_initializer`、`owns_storage`。

`borrow_ify` と `cancel` は、出力のすべてのグローバル初期化子の `owns_initializer` と `owns_storage` に
無条件に `true` を書く (`CODE src/rc_ir/borrow.rs: borrow_ify`, `cancel`)。この 2 つのフィールドに意味を
与える `divide_among_units` は、この 2 つのパスの**後**に走る (`CODE src/build/build_object_files.rs:
build_object_files` -- `optimize_rc_program` の呼び出しが `divide_among_units` の呼び出しより前にある)。
分割前のプログラムは 1 つであり、その 1 つがすべての初期化子と記憶域を持つので、`true` が正しい値である。

**D2 (本体の木)**
本体は式の節点 `RcExprNode` の木である。節点は式 `RcExpr` と source span からなり、式は次の 6 種である
(`CODE src/rc_ir/ast.rs: RcExpr`)。`s` はいずれも `RcState` 型の値で、コード生成が参照カウント操作をどう
特殊化するかを指示する (`CODE src/rc_ir/ast.rs: RcState`)。RC 規律 (D11) は `RcState` を読まないので、以下では
`s` を運ぶだけで参照しない。

| 節点 | 意味 | 継続 |
|---|---|---|
| `Let(x, rhs, k)` | `rhs` の値を `x` に束縛する | `k` |
| `Retain(v, π, s, k)` | 参照を作る (D10) | `k` |
| `Release(v, π, s, k)` | 参照を処分する (D10) | `k` |
| `Destructure(c, fs, s, k)` | 容器 `c` をフィールドに分解し、各 `(i, x)` の `x` に第 `i` フィールドを束縛する | `k` |
| `Eval(v, k)` | `v` を効果のために評価して捨てる | `k` |
| `Ret(v)` | この式の値は `v` である | 無し |

`Ret` を除く 5 種はちょうど 1 つの継続を持ち、`Ret` は継続を持たない。`Ret` は唯一の終端子である。

`Let` の右辺 `rhs` は次の 5 種である (`CODE src/rc_ir/ast.rs: RcRhs`)。`Var(y)`、`App(callee, args)`、
`Closure(f, caps)`、`Llvm(gen, args)`、`Match(scrut, arms)`。`Match` の各アーム `MatchArm` は 4 個の
フィールドを持つ (`CODE src/rc_ir/ast.rs: MatchArm`)。`tag` (変位番号、catch-all のときは無し)、`payload`
(payload 変数)、`payload_state` (`RcState`、上と同じ理由で以下では参照しない)、`body` (アーム本体)。

分岐は `Match` のアームだけであり、節点が自分自身を含むことはない。よって本体は有限の木であり、繰り返しは
関数呼び出しでしか作れない。

`RcExprNode` は式を `Arc` で共有するので、1 つの木の相異なる位置が同じ `Arc` を指すことがありうる。**この
文書では、本体の木の位置を「節点」と呼び、位置が相異なれば節点も相異なるものとする。** `Arc` のアドレスが
位置を一意に決めるかどうかは P15 が扱う。

**D3 (実行路)**
本体 `B` の**実行路**とは、次の規則で `B` の根から辿って得られる節点の有限列である。

- `Ret` を除く 5 種の節点では、その継続へ進む。
- `Let(x, Match(v, arms), k)` では、アームを 1 つ選び、そのアーム本体の実行路を辿り、その後 `k` へ進む。
- 関数本体の根から辿ってきて `Ret` に着いたら、そこで終わる。

アーム本体の `Ret` はそのアーム本体の実行路を終えるだけであり、関数本体の実行路は続く。関数本体の実行路の
最後の節点を**終端の `Ret`** と呼ぶ。D2 より `B` は有限の木なので、実行路は有限であり、その本数は有限である。

「**節点 `n` の後**」とは、`n` を含む実行路の上で `n` より後ろにある位置をいう。「**すべての路で**」とは、その
節点を含むすべての実行路について、という意味である。

**D21 (活性化と、それが辿る実行路)**
本体 `B` の 1 回の**活性化**とは、`B` の根から始まる 1 つの計算である。D9 と D10 が「1 回の活性化」と言う
のはこれである。活性化は各節点で D3 と同じ規則で進むが、
`Let(x, Match(v, arms), k)` では、選ぶアームが**決まっている**。`v` の値の実行時のタグに `tag` が等しい
アームであり、そのようなアームが無ければ、タグに `tag` が等しいアームを持たない `Match` に対する
コード生成の振る舞いに従う (`CODE src/rc_ir/codegen.rs`)。実行が節点を訪れる順序は、D3 の意味の実行路の
1 つである。これをその実行が**辿る実行路**と呼ぶ。

D3 は静的な概念であり、選ばれうるアームを列挙する。D21 は実行時の概念であり、1 つのアームが選ばれる。
D6 のスロット、D7 の参照カウント `H`、D10 の義務集合はいずれも活性化についての量であり、実行路だけでは
決まらない。「実行路 `ρ` の上で `H(o) ≥ 2`」のような言い方は、`ρ` を辿るすべての活性化についての主張として
読む。

活性化はアームの外の情報を使ってアームを選ぶので、1 つの実行路を辿る活性化が存在しないこともある。D11 が
すべての実行路について条件を課すのは、どの実行路が実現するかを決めないための安全側の近似である。

活性化が渡すのは、辿る実行路だけではなく、**各位置での値の割り当ても含む組**である。出力の活性化から
入力の活性化を作る段 (P14、P21) は、削除した `Retain`/`Release` が値を変えないことを使ってこの割り当てを
そのまま運ぶ。

**プログラムの**実行 -- 活性化の木、環境、スレッド -- は D22 以降が定める。1 つの本体の話をしている間は
「活性化」と書き、プログラム全体の話をするときだけ「実行」と書く。

### 3.2 値の構造

**D4 (boxed leaf)**
型 `τ` の値が参照を持ちうる位置を **boxed leaf** と呼び、その全体を `boxed_leaf_paths(τ)` が列挙する
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。leaf は値の根からのフィールド添字の列 (`FieldPath`) で
表す。列挙の規則は次のとおりで、上から順に判定する。

1. `is_fully_unboxed` が真の型は leaf を持たない。
2. クロージャは capture の位置 1 つを leaf とする。
3. `is_box` が真の型は、自分自身の位置 1 つを leaf とする。
4. `is_array` が真の型は、自分自身の位置 1 つを leaf とする。
5. それ以外 (unbox の構造体・タプル・union) は、`unpunched_field_types` が返すフィールドの下へ降りる。
   union のときは各変位の payload へ降りる。**穴 (punched field) は `unpunched_field_types` が返さないので
   降りない。**

**D16 (inhabited leaf)**
実行のある時点における値 `v` について、leaf `λ ∈ boxed_leaf_paths(ty(v))` が **inhabited** であるとは、
`λ` が通る unbox union の各節において、`λ` がその節で選ぶ変位番号が、その時点の `v` のその節のタグに等しい
ことをいう。unbox union を 1 つも通らない leaf は常に inhabited である。

unbox union は 1 つのタグしか持たないので、1 つの union の複数の変位の leaf が同時に inhabited になることは
ない。

**D5 (RC unit)**
1 回の参照カウント操作が対象にできる位置を **RC unit** と呼び、型 `τ` のすべてを `rc_units(τ)` が列挙する
(`CODE src/rc_ir/ownership.rs: rc_units`)。どの型が unit を担うかは `unit_step` が 1 か所で決める
(`CODE src/rc_ir/ownership.rs: unit_step`)。判定は上から順に、`is_fully_unboxed` なら unit 無し、
クロージャなら capture が 1 unit、`is_box` / `is_union` / `is_array` / `is_punched_array` のいずれかなら
自分自身が 1 unit、それ以外は unbox 集約としてフィールドの下へ降りる。

leaf と unit がずれるのは 2 か所である。**unbox union** は 1 つの unit だが、その leaf は各変位の payload の
中にある。**punched array** は 1 つの unit (`[]`) だが、その leaf は内側の配列 (`[0]`) である。

`truncate_to_unit(τ, π)` は path `π` をそれが属する unit の path へ切り詰める
(`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。`units_under(τ, π)` は、`subtree_type(τ, π)` が型を
返すときはその型の unit を `π` の下に並べたものを返し、`None` を返すときは `π` 自身だけからなる列を返す
(`CODE src/rc_ir/ownership.rs: units_under`, `subtree_type`)。

**D6 (スロット)**
実行のある時点における**スロット**とは、対 `(x, λ)` である。ここで `x` はその実行路の上でその時点までに
値を得た変数、`λ` は `ty(x)` の inhabited な boxed leaf である。`ty(x)` は `x` に束縛された値の型を表す。
スロットが指すオブジェクトを `obj(x, λ)` と書く。inhabited でない leaf にスロットは無い。

`x` を「その時点で束縛されている変数」に限らないのは、`origin` の答えがスコープを出た変数を名指すから
である。1 つのアームを持つ `Match` の本体 `Let(m, Match(s, [Let(a, App(f, []), Ret(a))]), k)` では
`origin(m, [])` の identity は `(a, [])` であり、`a` は `k` の位置ではスコープに無い。D17 の対応するスロットも
この広さを要求する。

### 3.3 所有、参照、義務

**D20 (別名の辺と別名の道)**
D9 の移動の表の 6 行を、スロットの間の**別名の辺**と呼ぶ。すなわち、`Let(x, Var(y), k)` の `y` から `x` へ、
アーム本体の `Ret(x)` の `x` から `Match` の束縛変数へ、unbox 容器の `Destructure` の名前付きフィールドの
容器からフィールド変数へ、unbox union の変位アームの scrutinee から payload 変数へ、catch-all アームの
scrutinee から payload 変数へ、`Llvm` の素通し leaf のオペランドから結果へ。**別名の道**とは、この辺を
向きを無視して辿る道である。

**D17 (対応するスロット)**
`origin` が `(x, π)` から `(u, σ)` へ辿った別名の辺の列を、`π` の下の leaf `λ` について辿ったときに着く leaf の
スロットを、`λ` に**対応するスロット**と呼ぶ。各辺での `λ` の写り方は次のとおりである
(`CODE src/rc_ir/ownership.rs: origin_inner`, `origin_from_leaves_under`)。

- `Binding::Move`、catch-all アームの payload、`Binding::Join`: `λ` を変えない。
- unbox 容器の `Destructure` のフィールド、unbox union の変位アームの payload: `λ` の先頭に添字を足す。
- `Binding::Llvm` の 2 つの道 (`leaf_origins_at(π)` が単一の `Arg` の場合と `origin_from_leaves_under` の
  場合): `λ` を、`λ` 自身の宣言 `Arg(j, σ')` の `σ'` へ置き換える。

path の連結ではなく宣言の辿り着く先で定義するのは、構築の演算 (`struct_make`、`union_make`) の宣言が接頭辞を
**外す**からである。`struct_make(m, m)` を `union_make_1` で包んだ値の leaf `[1, 0]` は、宣言 `Arg(0, [0])` を
経て `m` の leaf `[]` に対応する。path を連結する規則が指す `[1, 0]` は、boxed な `m` の leaf ではない。

**D14 (所有と借用)**
関数の各パラメータ・capture の各 unit は、その関数が**所有する**か**借用する**かのどちらかである。借用する
ものの集合が `RcFunc::borrowed_units` であり、残りが所有するものである
(`CODE src/rc_ir/ownership.rs: all_owned_units`)。所有する unit の参照はその関数が処分し、借用する unit の
参照は呼び出し元が処分する。

**D7 (オブジェクトと参照カウント、読み)**
実行時のヒープは**オブジェクト**の集合であり、各オブジェクト `o` は**参照カウント** `H(o) ≥ 0` を持つ。
`H(o)` が 0 になったオブジェクトは**解放される**。オブジェクトを**読む**とは、そのオブジェクトが占める記憶域の
うち、参照カウントと状態バイトを除いた部分を読むことをいう。解放されたオブジェクトを読むことを**解放後の読み**
と呼ぶ。

値を**読む構文**とは、次の 6 つである。読む構文は、名指した値の inhabited な各 boxed leaf が指すオブジェクトを
読みうる。

| 構文 | 読まれる値 |
|---|---|
| `Let(x, Llvm(gen, args), k)` | 各オペランド (`borrows_operand` の真偽によらない) |
| `Let(x, App(callee, args), k)` | callee と各引数 |
| `Let(x, Closure(f, caps), k)` | 各 capture |
| `Let(x, Match(v, arms), k)` | scrutinee `v` |
| `Destructure(c, fs, s, k)` | 容器 `c` |
| `Eval(v, k)` | `v` |

残る 4 種 (`Let(x, Var(y), k)`、`Retain`、`Release`、`Ret`) は読む構文ではない。`Var` と `Ret` は値を渡す
だけである。

`Retain(v, π)` と `Release(v, π)` は読まないが、`π` の下の inhabited な各 leaf が指すオブジェクトの参照
カウントと状態バイトに**触れる**。触れることは読むこと (記憶域のうちカウントと状態バイトを**除いた**部分を
読むこと) ではないので、この 2 つは読む構文ではない。それでも D11 の (S-c) は触れる先も扱う -- 解放された
オブジェクトのカウントに触れることは、読むことと同じく未定義の動作だからである。`Release` はカウントが 0 に
なったときにオブジェクトを走査するが、走査するのはそのオブジェクトが解放される時であって、(S-c) が禁じるのは
他のオブジェクトの解放後の読みである。

**D8 (参照)**
**参照**とは、1 つのオブジェクトに対する処分義務の 1 単位である。参照は D10 の**生成**によって作られ、D10 の
**消費**または `Release` によって処分される。オブジェクト `o` の参照カウント `H(o)` は、`o` への未処分の参照の
総数に等しい。

同じオブジェクトへの参照どうしは互いに区別されない。義務集合 (D10) はオブジェクトごとの個数を持つ多重集合で
あり、「その `Retain` が作った参照」のような言い方は、オブジェクトごとの個数として読む。移動 (D9) は、どの
参照が移ったかを決めない。

**D26 (計数下のオブジェクトとグローバル状態のオブジェクト)**
オブジェクトは**計数下**か**グローバル状態**かのどちらかである。割り当てられたオブジェクトは計数下であり、
グローバル値が到達するグラフに `mark_global` が印を付けた時点でグローバル状態になる。逆向きの遷移は無い。
A8 より、グローバル状態のオブジェクトへの `Retain`/`Release` は `H` を変えず、それが解放されることは無い
(`CODE src/generator.rs: Generator::build_release_boxed_with` -- `global_bb` はカウントを下げず
`destruction_bb` へも行かず `end_bb` へ跳ぶ)。

**D8 の参照、D10 の義務集合、D11 の (S-a) と (S-b) は、どれも計数下のオブジェクトへの参照だけを対象と
する。** グローバル状態のオブジェクトを指す leaf は、D8 の意味の参照を持たない。

この制限は無いと矛盾する。グローバル値 `g` を所有位置の引数として関数に渡す本体を考える。RC IR では `g` は
束縛を持たない `RcVar` として現れる (`CODE src/rc_ir/lower.rs: Lowerer::lower_var`)。D10 の初期値は
パラメータと capture だけを入れるので `g` の leaf の参照は `Obl` に入らず、D10 の生成の表にもグローバルの
行は無い。ところが `App(f, [g])` は D9 の `App` の行によって `g` の leaf を消費し、消費は `Obl` から参照を
取り除くので、制限が無いと (S-a) が破れる。**この形は実プログラムにいくらでもある。**

コード生成もこの読みと合っている。boxed なグローバルの読みは retain を伴わず
(`CODE src/generator.rs: Generator::add_global_object`)、呼び出し先が出す `Release` はグローバル状態の
オブジェクトに対して何もしない。カウントの上でも義務の上でも、グローバル状態のオブジェクトは勘定の外に居る。
A8 が言っているのはこのことであり、D26 はそれを D10 と D11 の本文に届く形に書き直したものである。

**D9 (消費と移動)**
関数の 1 回の活性化が保持する参照について、次の 2 つを区別する。

**消費**とは、活性化が保持する参照を活性化の外へ渡すか、捨てる構文である。次のものがある。

| 構文 | 消費される leaf |
|---|---|
| `App(callee, args)` | callee の全 boxed leaf、および呼び出し先がその位置の unit を所有する (D14) 引数の leaf。unit は**呼び出し先のパラメータの型**で取る (`CODE src/rc_ir/ownership.rs: rhs_consumes`) |
| `Closure(f, caps)` | 各 capture の全 boxed leaf |
| `Llvm(gen, args)` | `borrows_operand(i)` が偽のオペランドのうち、`result_prov` が**単一の** `Arg(i, σ)` として素通しを宣言していない leaf |
| `Destructure(c, fs)` (`c` が boxed) | `c` の全 boxed leaf |
| `Destructure(c, fs)` (`c` が unbox) | 名前が付いていないフィールドの leaf |
| 関数本体の終端の `Ret(x)` | `x` の全 boxed leaf (呼び出し元へ渡る) |

**移動**とは、参照の持ち手が活性化の中で変わるだけの構文である。移動は義務集合 (D10) を変えない。次のものが
ある。

| 構文 | 移動 |
|---|---|
| `Let(x, Var(y), k)` | `y` の参照が `x` へ |
| `Match` のアーム本体の `Ret(x)` | `x` の参照が `Match` の束縛変数へ |
| `Destructure(c, fs)` (`c` が unbox) の名前付きフィールド | `c` のそのフィールドの参照がフィールド変数へ |
| unbox union の変位アームの payload 束縛 | scrutinee の活性変位の参照が payload 変数へ |
| catch-all アームの payload 束縛 | scrutinee の参照が payload 変数へ |
| `Llvm` の素通し leaf (`result_prov` が単一の `Arg(i, σ)`) | オペランド `i` の参照が結果へ |

上の 2 つの表と D10 の生成の表で、参照を作る・移す・手放す構文はすべてである。`Eval(v, k)` と
`Let(x, Match(v, arms), k)` の `Match` 節点自身は、参照を作らず、移さず、手放さない。`Retain` と `Release` は
D10 が直接扱う。

`collect_consumes` が報告するのは、消費に加えて**アーム本体の `Ret` も含めた集合**である
(`CODE src/rc_ir/ownership.rs: collect_consumes_go` の `RcExpr::Ret` の腕)。すなわち報告される集合は消費の
上位集合である。この過剰報告の読み手は `infer_ownership` だけであり、そこでは所有を増やす向きに働くので安全側で
ある。`cancel` はこの関数を読まず、`rhs_consumes` と `destructure_consumes`、および終端の `Ret` の扱いを
自分で持つ (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner` の `RcExpr::Ret` の腕の
`returns_from_func`)。

**D10 (義務集合)**
関数の 1 回の活性化について、実行路上の各位置における**義務集合** `Obl` を、参照の多重集合として次で定める。

- 初期値: 所有する (D14) パラメータ・capture の unit の下の inhabited な各 leaf につき 1 つ。借用する unit の
  下の leaf は入れない。
- `Retain(v, π)`: `π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ加える。同時に
  `H(obj(v, λ))` を 1 上げる。
- `Release(v, π)`: `π` の下の inhabited な各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ取り除く。同時に
  `H(obj(v, λ))` を 1 下げる。
- **生成**: 次の構文が新しい参照を作る。生じた inhabited な各 leaf につき参照を 1 つ加える。`H` はその場で
  上がる (割り当てなら 1 から始まり、読み出しなら 1 上がる)。

  | 構文 | 生じる参照 |
  |---|---|
  | `Llvm(gen, args)` の結果の leaf のうち、`result_prov` の宣言が単一の `Arg(j, σ)` **でない**もの | 各 1 つ |
  | `App(callee, args)` の結果の各 boxed leaf | 各 1 つ。ただし `H` はここでは動かない -- その参照は呼び出し先の中で作られており、返りで持ち手が移るだけである |
  | `Closure(f, caps)` の結果 (capture object) | 1 つ |
  | boxed 容器の `Destructure` の各名前付きフィールドの各 leaf | 各 1 つ |
  | boxed union の変位アームの payload の各 leaf | 各 1 つ |

  `Llvm` の行は、宣言が空集合 (bottom) のとき、`Fresh` や `Unknown` を含むとき、複数の元を持つときのすべてを
  含む。空集合と宣言された leaf は inhabited にならないので、参照は生じない (A3)。

- **消費** (D9): 消費される inhabited な各 leaf につき参照を 1 つ取り除く。渡す先のある消費 (`App` の引数、
  `Closure` の capture、終端の `Ret`) では `H` は変わらず、参照は渡された先が持つ。**捨てる**消費 --
  boxed 容器の `Destructure` は容器の参照を捨て (`CODE src/object.rs:
  ObjectFieldType::get_struct_fields`)、unbox 容器の `Destructure` は名前の付いていないフィールドの leaf を
  捨てる -- では `H` が 1 下がる。D9 は消費を「外へ渡すか、捨てる」と定めており、この行はその 2 つを分ける。
- **移動** (D9): `Obl` を変えない。

### 3.4 RC 規律

**D11 (RC 規律を満たす本体)**
本体 `B` が、所有と借用の割り当て (D14) の下で **RC 規律を満たす**とは、`B` のすべての実行路について次の 3 つが
成り立つことをいう。

- **(S-a) 過剰処分が無い**: `Obl` から参照を取り除くすべての操作について、取り除かれる参照はその時点の `Obl` に
  入っている。
- **(S-b) 漏れが無い**: 実行路の終端の `Ret(v)` において、その `Ret` の消費を行った後の `Obl` は空である。
- **(S-c) 解放後の読みが無い**: D7 の読む構文がその位置で読みうる各オブジェクト、および `Retain(v, π)` と
  `Release(v, π)` が触れる (D7) 各オブジェクトは、その時点で解放されていない。

  `Release` の側は自明に見えて自明でない。カウントが 1 のオブジェクトを `Release` するとき、そのオブジェクトは
  **その時点では**解放されておらず、解放するのはその `Release` である。よって条件は満たされる。禁じているのは、
  すでに解放されたオブジェクトのカウントに触れることである。

  `Retain` の行が要る理由は較正 (第 6 節) が述べる。この行が無いと、3 つの節をすべて満たしながら 1 つの
  オブジェクトを 2 回解放する本体が書ける。

**D12 (RC 規律を満たすプログラム)**
プログラム `P` が **RC 規律を満たす**とは、`P` のすべての関数の本体と、すべてのグローバル初期化子の `init` が、
`P` の `borrowed_units` が定める所有と借用の割り当て (D14) の下で RC 規律を満たす (D11) ことをいう。

`p*.md` の本文は D11 と D12 を「健全」とも呼ぶ。指すものは同じである。

D11 と D12 は `RcProgram` の残りの部分について何も言わない。`roots`、`RcFunc` の `fn_ty` / `ret_ty` / `source` /
`inline_into_callers`、`RcGlobalInit` の `symbol` / `ty` / `owns_initializer` / `owns_storage` である。
これらを `borrow_ify` と `cancel` がどう扱うかは P13 と P24 が扱う。

**D18 (一意性の観測点)**
`unsafe_is_unique` は、boxed な値がただ 1 つの名前から参照されているかを `Bool` として返す std の公開関数で
ある (`CODE src/fixstd/stdlib.rs: unsafe_is_unique`)。RC IR では 1 つの `Llvm` 演算として現れる。この演算が
現れる位置を**一意性の観測点**と呼び、その位置でその演算が返す `Bool` を**観測値**と呼ぶ。

観測値はプログラムの振る舞いを変える。`Debug::assert_unique` は観測値が偽のとき `undefined` を呼んで
プログラムを止め、`Destructor::mutate_unique_io` は偽のとき資源の複製 (`ctor : a -> IO a`) を走らせる
(`CODE src/fixstd/std.fix: assert_unique`, `mutate_unique_io`)。

**D19 (観測点の対応)**
`borrow_ify` と `cancel` は、値を計算する構文を作らず、消さず、並べ替えない。よって出力の各一意性の観測点は、
入力のちょうど 1 つの観測点から来る。この対応を**観測点の対応**と呼ぶ。`borrow_ify` が関数の借用版を作る
とき、借用版の本体の観測点は、複製元の本体の同じ位置の観測点に対応する。

### 3.5 同一性のモデル

**D13 (origin)**
`origin(vars, τenv, x, π)` は、変数 `x` の path `π` にある値が、どの変数のどの path で作られた参照を持つかを
答える関数である (`CODE src/rc_ir/ownership.rs: origin`)。返り値は `Exactly(u, σ)` か
`Join { identity, candidates }` のどちらかである (`CODE src/rc_ir/ownership.rs: Origin`)。`identity` は
`VarPath` を 1 つ、`candidates` は `VarPath` の空でない集合を持つ。以下では `vars` と `τenv` が文脈から
定まるとき `origin(x, π)` と書く。

D13 は `origin` が何を返すかを述べるだけであり、その返り値が実行時の参照とどう対応するかは P3 と P4 が述べる。

**D15 (触れる参照)**
`acted_references(v, π)` は、`π` の下の**すべての** boxed leaf (inhabited でないものを含む) について、その
leaf の `origin(v, leaf).identity()` を数えた多重集合を返す (`CODE src/rc_ir/ownership.rs: acted_references`)。
以下ではこれを `ActRefs(v, π)` と書く。これは静的な数え上げであり、実行時に触れる参照との関係は P6 が述べる。

返り値の型 `References` は `VarPath` から個数への写像であり、次の演算を持つ
(`CODE src/rc_ir/ownership.rs: References`)。`covers(R)` は各オブジェクトについて自分の個数が `R` 以上か。
`names(o)` は `o` を含むか。`objects()` は含むオブジェクトの列 (個数は落ちる)。`shares_an_object(R)` は
`R` と共通のオブジェクトを持つか。`subtract(R)` は `R` の個数を引く (`covers(R)` が成り立つときにだけ
呼ばれる)。`is_empty()` は空か。

`Origin::acted_on()` は、`identity()` を先頭に、それと異なる `candidates()` の元を続けた列である
(`CODE src/rc_ir/ownership.rs: Origin::acted_on`)。すなわち
`acted_on() = {identity()} ∪ candidates()` であり、`Exactly(u, σ)` のときは 1 元の列 `[(u, σ)]` である。

**この文書はオブジェクトの名前を `VarPath` の水準で扱う。**`cancel` が `Retain` と `Release` を対にする
のは、両者が触れるオブジェクトが共通するかどうかによる。かつてこの位置に置かれていた `unit_key` --
`origin` の identity を `truncate_to_unit` で unit へ丸めた名前 -- は、第 8 節が述べる 3 件の
miscompile の共通の根であり、コミット `a924f115` で取り除かれた。

## 4. 仮定

各仮定には、それを果たす者を書く。

**A1 (入力が RC 規律を満たす)** -- 果たす者: 前段のパス (`insert_rc`)。
`borrow_ify` に渡されるプログラムは D12 の意味で RC 規律を満たす。またそのプログラムのすべての関数の
`borrowed_units` は空である。すなわちすべてのパラメータ・capture の unit が所有される。

**A2 (単位への正規化)** -- 果たす者: `insert_rc` と `split_rc_units`。
`borrow_ify` に渡されるプログラムのすべての `Retain`/`Release` 節点の path は、その変数の型の `rc_units` の
要素である。`insert_rc` が出す `Retain`/`Release` の path はすべて空列であり、`split_rc_units` はそれを
`units_under(τ, [])` で分解する (`CODE src/rc_ir/borrow.rs: split_rc`)。`subtree_type(τ, [])` は常に
`Some(τ)` を返すので、この分解が返すのは `rc_units(τ)` そのものである。

**A3 (宣言されたモデルの忠実さ)** -- 果たす者: 誰も。
各 `LLVMGen` の `result_prov` と `borrows_operand` は、その演算が生成するコードを正しく述べている
(`CODE src/ast/inline_llvm.rs: LLVMGen::result_prov`, `LLVMGen::borrows_operand`)。`result_prov` は結果の
leaf ごとに `LeafOrigin` の集合 (`LeafOrigins`) を宣言する。宣言と生成コードの対応は次のとおりである。

| 宣言 | 生成コードが結果のその leaf に置くもの |
|---|---|
| 空集合 | 何も置かない。その leaf は inhabited にならない (存在しない union 変位、または中断する演算の結果) |
| 単一の `Arg(j, σ)` | 第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を作らない。結果のその leaf が inhabited であることと、第 `j` オペランドの leaf `σ` が inhabited であることは同値である |
| 単一の `Fresh` | 新しく割り当てたオブジェクトへの新しい参照 |
| 単一の `Unknown` | 既存のオブジェクトへの新しい参照 (retain を伴う読み出し)。そのオブジェクトは、この op のオペランドの leaf が指すオブジェクトから到達できるか、グローバル値が到達する (`CODE src/rc_ir/provenance.rs: LeafOrigin`) |
| 複数の元 | 実行路ごとにそのいずれか。いずれの路でも新しい参照 |

`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分しない。

複数の元を宣言する op は、このコミットのプログラムには存在しない。`impl LLVMGen for` は 78 個あり、
`result_prov` を override するのは 29 個、その 29 個が leaf に置く集合はすべて要素数 0 か 1 である
(`sole_origin` / `Set::default()` / `uniform` / `uniform_bottom` / `fresh_under` のいずれかで作られる)。
複数元を作るのは `Provenance::join` と `compose` であり、これらは解析の側であって宣言ではない。表がこの行を
持つのは、`LLVMGen::result_prov` の型と doc がそれを許すからである。

**P5 (a) はこの数え上げに載っている。** 元数 2 以上の宣言を持つ op が現れると、`origin` は boxed leaf の
path から `origin_from_leaves_under` の `truncate_to_unit` を通る枝に入りうる。そのとき 1 つの unbox union の
下の 2 つの leaf がどちらもその union の unit path へ切り詰められ、`identity` が 1 つに潰れる。潰れた
2 つの leaf は別々のオブジェクトを指しうるので、P5 (a) が破れる。`Std::Option (a, b)` の payload が
この形の値である。すなわち、複数元を宣言する op を足すことは、`cancel` の対の健全性を壊す変更である。

この仮定は誰も果たさない。宣言と実装の乖離は、証明ではなくテストと valgrind が捕まえる。
`dev-docs/2026-06-28-unique-check-elim/audit-2026-07-20-op-declarations.md` が、ある時点での全 op の宣言を
人手で照合した記録である。

**A4 (コード生成の忠実さ)** -- 果たす者: 誰も。
コード生成は、`Retain(v, π)` / `Release(v, π)` を、`π` の下の **inhabited な** 各 boxed leaf の参照カウントの
±1 として実装する。unbox union に対しては実行時のタグで分岐し、活性な変位の payload だけを数える
(`CODE src/object.rs: ObjectFieldType::retain_release_mark_union`)。`Destructure` と `Match` の変位アームに
ついては、D9 の消費・移動の表と D10 の生成の表のとおりに実装する。

**A5 (型が leaf の上位近似)** -- 果たす者: `leaf_map.rs` の設計。
値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf のうち inhabited (D16) なものにちょうど
1 つずつある。inhabited でない leaf は参照を持たない。

例外が 1 つある。capture が空のクロージャの capture の leaf は null ポインタであり、そこにオブジェクトも
参照も無い (`CODE src/rc_ir/codegen.rs: Generator::build_rc_closure`, `CODE src/generator.rs:
Generator::build_traverser_work` -- boxed の腕は `build_if_nonnull` で包む)。この文書は null の leaf を
inhabited でない leaf と同じに扱う。

**A6 (名前の一意性)** -- 果たす者: lowering。
`borrow_ify` の入力のすべての束縛変数の名前は相異なり、**どの関数の名前とも異なる**。後半が要るのは、
`call_rc` が呼び出し先を `callee_params` から名前で引くからである。よって変数名は束縛を一意に決める。出力についての同じ
性質は仮定ではなく P9 が示す -- `fresh_rename_function` を呼ぶのは証明対象の `borrow_ify` 自身なので、
それを仮定に置くと証明対象が自分を支えることになる。

**A13 (束縛名の形)** -- 果たす者: `Lowerer::fresh_var` と `clone_fresh`。検査: 無し。
`borrow_ify` の入力のすべての束縛名について、その `name` フィールドを `#` で区切った最後の断片は、文字 `b` の
後に 10 進数字だけが続く形ではない。

`clone_func` が導入する名前が入力のどの束縛名とも異なること (P9 の後半) は、これが無いと言えない。
`fresh_rename_function` は入力の名前を 1 つも読まないので、衝突しないことは入力の名前の形からしか出ない
(`CODE src/rc_ir/rename.rs: fresh_rename_function`)。名前が衝突すると `origin` が複製の変数を原本の束縛へ
辿り、`cancel` が誤った対を消す。

**A14 (過適用が無い)** -- 果たす者: 型検査と lowering。
`App(callee, args)` の `args` の個数は、呼び出し先のパラメータの個数以下である。`call_rc` と `rhs_consumes` は
`params[arg_idx]` を引くので、これが無いと範囲外になる。

**A15 (`grow_stack` は閉包をちょうど 1 回呼ぶ)** -- 果たす者: `stacker` crate。
`grow_stack(f)` は `f` をちょうど 1 回呼び、その返り値を返す (`CODE src/misc.rs: grow_stack`)。`origin`、
`CancelAnalysis::walk`、`RewriteCtx::rewrite`、`drop_nodes`、`rename_expr` はいずれも本体を `grow_stack` で
包むので、これが無いと「各位置をちょうど 1 回訪れる」がどれも言えない。

**A16 (`Match` のアームは scrutinee のタグを尽くす)** -- 果たす者: lowering
(`CODE src/rc_ir/lower.rs: Lowerer::lower_match`, `Lowerer::lower_if`) と、アームの列を保つ後段のパス。
検査: 無し。
すべての `Match(s, arms)` について、`arms` が catch-all アーム (`tag` が `None`) を持つか、`s` の値が
取りうる実行時のタグがいずれかのアームの `tag` である。

D21 は「タグが等しいアームが無ければコード生成の振る舞いに従う」と書いており、コード生成は最後のアームの
ブロックを switch の default とする (`CODE src/rc_ir/codegen.rs: Generator::eval_rc_match`)。最後の
アームが変位アームで、どのアームも名指さない変位が在ると、実行はその変位の値をもって `tag = Some(t)` の
アームに入る。そのとき D9 の移動の表の「unbox union の変位アームの payload 束縛」の行が名指す**活性**変位と、
`origin_inner` の `Binding::Payload(scrut, Some(t))` の腕が辿る静的な変位番号 `t` が食い違い、P5 (a) と
P6 が偽になる。`validate` が見るのは、アームが 1 つ以上あること、catch-all アームが最後にあること、
2 つのアームが同じ変位を担わないことだけである (`CODE src/rc_ir/validate.rs: Validator::check_rhs`)。

**A19 (bump の下に余りが在る)** -- 果たす者: `insert_rc` (使用回数の勘定) と `borrow_ify`
(`rewrite_rc` が落とさないこと、落とす場合に `call_rc` が補うこと)。検査: 無し。

別名類とは、1 つの実行路の上で同じスロットに辿り着くスロットの集まりである
(`p13-disposals-and-pending.md` の第 7.5 節が定める)。`cancel` の入力の各本体、各実行路 `ρ`、`ρ` を辿る
各活性化について、次の 2 つを仮定する。

- **(i)** 各時点と各計数下オブジェクト `O` について、`H(O)` は `O` を指す別名類が持つ参照の総数以上である。
  すなわち相異なる類の参照は相異なる。
- **(ii-a) 由来の形** -- 読む者: P14。各時点と各計数下の別名類について、その類が持つ参照の個数は非負で
  あり、読む構文と `Retain`/`Release` がその類を名指す時点では 1 以上である。
- **(ii-b) 帳簿の形** -- 読む者: P18a、P19、P21。各時点と各計数下の別名類について、その類が持つ参照の
  個数は、**走査がその類について `pending` に数えている bump の個数以上**である。

  **(ii-a) と (ii-b) は独立である。どちらも他方を導かない** (`p60-insert-rc.md` の `L12` と `L13` が
  両向きの反例を与える)。`C1` は (ii-a) を満たして (ii-b) を破り、
  `Let(o, alloc, Let(y, id(o), Eval(o, Let(u, App(f,[y]), Ret(u)))))` は (ii-b) を満たして (ii-a) を破る。
  よって 1 つの仮定にまとめられない。

  (ii-b) を「bump より 1 つ多い」と書くと、字義どおりには偽である -- その類の最後の参照が処分された後の
  時点では `0 ≥ 1` になる。要るのは上の形であり、P18a が使うのは「bump が 1 以上ある時点では、参照は
  bump より 1 つ多い」という導かれた形である。`bumps ≥ 1` ならその bump を作った `Retain` の対象が
  参照を 1 つ持っていたので、`held ≥ bumps` と合わせて `held ≥ 1 + bumps` が出る。

  計数下の類に限るのは、グローバル値を終端とする類に `held` の開始値を与える行が無いからである (D26)。

この仮定は 2 つの命題が読む。P18a の `+1` はここから来る。P14 も同じ仮定を読む -- D11 はオブジェクトごとの
多重集合しか見ないので、1 つのオブジェクトを指す 2 つの別名類の間で収支を融通する本体は D12 を満たしながら
これを破り、そのとき借用版が落とす節点と残す節点の対応が崩れる。

**P18a は A1・A2・D12 だけからは出ない** --
`p13-disposals-and-pending.md` の第 7.5.7 節の `C1` と第 7.5.8 節の `C2` が、D12 を満たしながら P18a を
破る本体である。この 2 つはコードの欠陥ではなく、この仮定が要ることの証拠である。

**(ii-b) が何を要求しているかは `p60-insert-rc.md` の `L7` が正確にした。** 恒等式
`held - (1 + bumps) = U + X - D` が成り立つ。`D` はその類の処分の個数、`U` は `un_bump` が引く量、
`X` は `consume_objects` と `merge` が要素ごと落とす量である。すなわち **(ii-b) は「走査の帳簿がその類の
処分に遅れない」ことに等しい。** 決めるのは `held` の推移ではなく、`origin` の `identity` と `acted_on`
が名前を共有するかどうかである。`C1` は名前が届かない例、`C2` は `Retain` が処分より後に来る例と、
統一的に読める。

**果たす者の 2 人のうち、`borrow_ify` の側は示されている** (`p13-disposals-and-pending.md` の `L16`)。借用版が `Retain` を落と
すのは `owns_unit` が偽のときだけで、`App` の引数以外の消費では P8 と P7a がそれを真にし、`App` の引数では
`call_rc` が同じ `Retain` を呼び出しの直前に置き直す。**`insert_rc` の側は部分的に示されている** (`p60-insert-rc.md`)。示されたのは次の 2 つである。

- **`insert_rc` が出す `Retain` は、間に `Retain` 以外の節点を挟まず、同じ変数を名指す構文の直前に立つ**
  (`L9`)。その構文は D9 の消費か移動を行う。この形は `split_rc_units` と `borrow_ify` を通って `cancel` の
  入力まで残る (`L11`)。
- したがって **`C1` と `C2` はどちらも `insert_rc` の出力ではない** (`L10`)。`Retain` の直後の節点が
  その変数を名指さないので、節点 1 つを見るだけで弾ける。

残る義務は 2 つである (`p60-insert-rc.md` の第 8 節)。**(O1)** 別名類の粒度で D11 を `insert_rc` の出力に
ついて示すこと -- これが (ii-a) を与える。**(O2)** 台帳の不等式 `U + X ≥ D` -- (O1) と合わせて (ii-b) を
与える。(O2) で開いているのは `Join` の束縛変数の場合だけで、埋め合わせの機構は特定されている --
アーム内の `retain_if_live` が置く `Retain` の要素を `merge` が `entered_with` のゲートで必ず落とすので
`X` が増える -- が、常に足りることは示されていない。**どちらの形についても、`insert_rc` の出力の反例は
見つかっていない。**

`L6` は「名前は別名類を決める」を前提に持つ。`p13-disposals-and-pending.md` の `L9`/`L12`/`L13` と
P5 (a) が与える形だが、`p60-insert-rc.md` はそれを証明していない。

**A20 (借りた参照は活性化の間 生きている)** -- 果たす者: 呼び出し元。検査: 無し。
`borrow_ify` の出力の関数が借用する (D14) unit について、呼び出し元はその参照を、呼び出しが返るまで
処分しない。

D14 は「借用する unit の参照は呼び出し元が処分する」としか言わず、呼び出しの**間**に処分しないことを
言わない。**これは D11 の (S-c) が呼び出し元に触れる唯一の点である** -- (S-a) と (S-b) は 1 つの本体の
中で閉じるが、借用したものを読んでよいかは呼び出し元の振る舞いで決まる。P14 の (S-c) がこれを読む。

**A17 (環境の契約)** -- 果たす者: 環境のコード (`build_main_function`、`ExportStatement::implement`、
`implement_rc_global`)。検査: 無し。
環境とは、RC IR プログラムの外側にあってその本体を起動するコードである。環境について次の 3 つを仮定する。
(i) 環境が活性化を作るとき、D10 の初期値が要求する参照を渡し、それ以後それを持たない。(ii) 環境が読む
オブジェクトは、その時点で環境が持つ参照が指すオブジェクトか、そこから到達できるオブジェクトである。
(iii) 環境が動くのは、生きている活性化がどれも動いていないときだけである。

**A18 (残るものについての 2 つの仮定)** -- 果たす者: 誰も。
(a) **生きているオブジェクトのグラフは非巡回である。** 検査: valgrind の下で走るテスト (閉路になった
計数下のオブジェクトは `definitely lost` として報告される)。
(b) **グローバル状態のオブジェクトは計数下のオブジェクトへの参照を持たない。** 検査: 無し。`mark_global` は
印を付ける時点で到達できるグラフ全体に印を付ける (`CODE src/generator.rs: Generator::mark_global`)。

この 2 つを使うのは P27 の (R3) だけである。(R1) と (R2) はどちらにも依らない。

**A7 (呼び出し先の解決)** -- 果たす者: `resolve_callee_params` の設計 (`CODE src/rc_ir/ownership.rs:
resolve_callee_params`)。
`prog.funcs` に無い呼び出し先は、全パラメータの全 unit を所有するものとして扱われる。これは所有を増やす向きの
近似である。

**A8 (グローバルは線形規律の外)** -- 果たす者: `mark_global`。
グローバル値が到達するオブジェクトは、記憶域に「グローバル」を表す状態を持ち、それらへの `Retain`/`Release` は
参照カウントを変えない。よってそれらのオブジェクトが解放されることはない。

**A9 (`Match` はアームを持つ)** -- 果たす者: lowering。検査: `validate` の `check_rhs`
(`CODE src/rc_ir/validate.rs: Validator::check_rhs`)、ただし `develop_mode` のときだけ走る。
プログラムのすべての `Match` は 1 つ以上のアームを持つ。

**A10 (型の well-formedness)** -- 果たす者: `validate_layouts` (elaboration で必ず走る)。ただし最適化が
作る型を再検査するのは develop build だけである。
プログラムに現れる型は ground であり、その tycon は `type_env` にあり、`no_size_in_place` の in-place の
降下は有限である。これが無いと `boxed_leaf_paths` も `rc_units` も停止しない。

**A11 (スコープの規律)** -- 果たす者: lowering。検査: `validate` の `check_expr_inner` と `check_rhs`
(`CODE src/rc_ir/validate.rs: Validator::check_expr_inner`, `Validator::check_rhs`)、ただし
`develop_mode` のときだけ走る。
変数の使用は、その位置でスコープに入っている束縛に解決する。A6 は「同じ名前が 2 度束縛されない」までしか
言わず、`x` の束縛が `x` 自身を参照しないことは言わない。`origin` の停止性はこの仮定に立つ
(`VarTable::origins` の memo は答えを再帰から戻った後に記録するので、閉路があれば memo が当たる前に
無限に潜る)。

**A12 (束縛の形と型が合っている)** -- 果たす者: 誰も。
move-bind の両辺の型、アームの結果と `Match` の束縛変数の型、payload と変位の型、**catch-all アームの
payload と scrutinee の型**、`Destructure` のフィールド変数とフィールドの型、**`App(callee, args)` の各引数と
呼び出し先の対応するパラメータの型**、`Match` の scrutinee が union であること、`Destructure` の容器が
構造体であること、**`Destructure` が名指すフィールドと `Match` が名指す変位が、その型が実際に持つ
(punched でない) ものであること**、同じ名前の `RcVar` が持つ型が一致すること。

punched でないことが要るのは、`held_field_type` が持たないフィールドを問われると panic するからである
(`CODE src/rc_ir/ownership.rs: held_field_type`)。**このコミットにこれを検査するコードは無い**
(`validate` は構造だけを見る)。

引数とパラメータの型の一致は、`rhs_consumes` が引数の leaf を呼び出し先のパラメータの型で
`truncate_to_unit` に掛けるので、停止性にも要る -- 型に合わない path は panic する
(`CODE src/rc_ir/ownership.rs: rhs_consumes`, `truncate_to_unit`)。

## 5. 命題

依存の順に並べる。各命題は自分より前の命題だけを引用してよい。番号は導入の順ではなく固定された名札なので、
後から間に挟むときは `P7a` のように枝番を振り、既存の番号は振り直さない。

### 層 1 -- 所有権モデル (`ownership.rs`)

- **P1** (leaf と unit の対応)。任意の型 `τ` について、`boxed_leaf_paths(τ)` の各 leaf の
  `truncate_to_unit(τ, ・)` は `rc_units(τ)` の要素であり、`rc_units(τ)` の各 unit はある leaf の
  `truncate_to_unit(τ, ・)` である。
- **P2** (`origin` の全域性と停止性)。`origin(x, π)` は、`x` がプログラムの束縛変数であるようなすべての
  `(x, π)` について、`π` を問わず panic せずに答えを返し、停止する。

  `π` に制限を置かないのは、置いた制限が再帰について閉じないからである。`Result e (Option a)` を match して
  payload に `Retain` を置くと、`origin` はその payload の `[]` から scrutinee の `[0]` を問い、`[0]` は
  scrutinee の型の leaf でも unit でもない。
- **P3** (`origin` の健全性 -- `Exactly`)。`origin(x, π) = Exactly(u, σ)` のとき、すべての実行路のすべての
  位置において、`π` の下の inhabited な各 leaf `λ` について、`obj(x, λ)` を指す参照は、`λ` に対応するスロット
  (D17) が持つ参照と同一である。
- **P4** (`origin` の健全性 -- `Join`)。`origin(x, π) = Join { identity, candidates }` のとき、各実行路の
  各位置において、`π` の下の inhabited な各 leaf のスロットが持つ参照は、`candidates` のいずれかの下の
  対応するスロット (D17) が持つ参照と同一である。
- **P5** (identity とオブジェクトの関係)。1 つの関数の 1 回の活性化について、次の 3 つが成り立つ。
  - **(a)** (対の健全性) 1 つの実行路の 1 つの位置において `origin` の `identity` が等しい 2 つの leaf の
    スロットは、同じオブジェクトを指す。
  - **(b)** (対の有効性) 同じオブジェクトを指す 2 つの leaf のスロットで、一方から他方への別名の道が
    `Match` のアーム本体の `Ret` の辺を含まないならば、両者の `identity` は等しい。
  - **(c)** (被覆) `Release(v, π)` の走査が `un_bump` と `consume_objects` に渡すオブジェクトの和 --
    すなわち `ActRefs(v, π).objects()` と `other_objects(v, π)` の和 -- は、`π` の下の各 boxed leaf `λ` に
    ついて `origin(v, λ).acted_on()` をすべて含む。

  (a) が `cancel` の健全性を支える。`un_bump` が対にするのは identity が共通する `Retain` と `Release` で
  あり、(a) が成り立たなければ対でないものを対にする。(b) は有効性 (対が実際に見つかること) の側であり、
  破れても `Retain` が `needed_retains` に入るだけである。制限は外せない。アーム本体の `Ret` の辺は
  identity を保たず、`m` と `x` が同じオブジェクトを指すのに identity が `(m, λ)` と `(x, λ)` に分かれる
  本体が作れる (`p12-identity-and-consumes.md` の反例 R1)。走査はこの辺を `merge` で扱う (P18)。

  (c) は `Release` の 2 つの腕の役割分担を述べる。`origin` が `Join` を返す leaf について、実行時に処分
  される参照が属するオブジェクトは `candidates` のどれでもありうるが、`ActRefs` は `identity` しか名指さ
  ない。残る `candidates \ {identity}` を `other_objects` が拾い、`consume_objects` へ渡す
  (`CODE src/rc_ir/borrow.rs: CancelAnalysis::other_objects`)。
- **P6** (`acted_references` は静的な上位近似である)。1 つの関数の 1 回の活性化について、`acted_references(v, π)` が返す `Map` は、`π` の下の
  すべての boxed leaf を `origin` の identity で名付けて数えたものである。実行時に `Retain(v, π)` が作る
  参照の多重集合は、この数え上げを inhabited な leaf に制限したものに等しく、`Release(v, π)` が処分する
  参照の多重集合も同じものに等しい。
- **P7** (消費の網羅性)。D9 の意味で消費する構文はすべて `collect_consumes` が報告する。また
  `collect_consumes` が報告して D9 が消費としないものは、アーム本体の `Ret` に限る。

### 層 2 -- `borrow_ify` が RC 規律を保存すること

- **P7e** (`owns_object` は unit ごとに答える)。任意の root `r` と path `p` について、
  `owns_object(r, p) = owns_object(r, truncate_to_unit(ty(r), p))` である。`r` がこの版のパラメータ・
  capture でないときは、両辺とも真である。

- **P7d** (所有は site ごとに一様である)。`infer_ownership` の不動点において、`levelled_sites` が挙げる
  各 site `(v, u)` について、`origin(v, u)` の候補は、すべて `owns_object` が真であるか、すべて偽であるかの
  どちらかである。

  `owns_unit(v, u)` は候補すべてに `owns_object` を要求して真偽値を 1 つ返し、`rewrite_rc` と `call_rc` は
  その 1 つの答えで unit 全体の `Retain`/`Release` を決める。答えが候補ごとに割れる site があると、真の側
  では借りている参照を処分し、偽の側では所有している参照を処分しない。どちらも誤りであり、それが #530 で
  ある。`level_ownership` はこの一様性を、候補が 1 つでも所有されていれば候補すべての leaf を所有へ倒す
  ことで作る (`CODE src/rc_ir/borrow.rs: level_ownership`, `owns_object_yet`)。

- **P7c** (処分はすべて走査に届く)。実行時に参照を処分するか、処分の義務を活性化の外へ渡す構文 --
  D9 の消費、`Release`、終端の `Ret` -- はすべて、`cancel` の走査で次のどちらかを行う。

  - **(a)** 終端の `Ret` 以外では、`consume_objects` または `un_bump` を、その構文が触れうるオブジェクト
    (D15 の `acted_on`) をすべて含む引数で呼ぶ。
  - **(b)** 終端の `Ret` では、その時点の `pending` のすべての要素を `needed_retains` に入れる。

  (b) を別に書くのは、`walk_inner` の `RcExpr::Ret` の腕がそのどちらの関数も呼ばず、`needed_retains` へ
  直接入れるからである (`CODE src/rc_ir/borrow.rs: CancelAnalysis::walk_inner`)。印を付ける範囲は
  `consume_objects` より広く、要素を `pending` から取り除かない点だけが狭い。取り除かないことが後段に
  響かないのは、終端の `Ret` の訪問が返す `pending` が走査全体の返り値であり、`cancel` がそれを捨てるから
  である (`CODE src/rc_ir/borrow.rs: cancel`)。

  P5 (a) では足りない。P5 (a) は同じ identity を持つ 2 つのスロットについての主張だが、`cancel` が要るのは
  **1 つのオブジェクトを指す相異なる 2 つの参照**についてであり、こちらの方が広い。

- **P7f** (処分の後、pending はそのオブジェクトを名指さない)。`Release` の訪問について、`un_bump` が
  `NoBracket` か `OutsideBracket` を返したとき、その訪問の後の `pending` のどの要素の `outstanding` も、
  その `Release` が触れうるオブジェクトのどれも名指さない。

  P7c は「走査が呼んだ」までしか言わない。P18a が要るのは「pending から消えた」であり、その 1 段が
  ここにある。`Release` の腕は `other_objects` を先に `consume_objects` へ渡してから `un_bump` を呼ぶので、
  この 2 つの順序が load-bearing である。

- **P7a** (site の所有は、その leaf の所有と一致する)。`(v, u)` を `levelled_sites` が挙げる site とし、
  `Λ(u)` を `u` の下の boxed leaf の集合とする。`infer_ownership` の不動点の下で、次の 3 つは同値である。

  1. `owns_unit(v, u)` が真である。
  2. `Λ(u)` の**ある inhabited な** leaf `λ` の**すべての**候補 `(r, p)` について `owns_object(r, p)` が
     真である。
  3. `Λ(u)` の**すべての inhabited な** leaf のすべての候補について `owns_object` が真である。

  成り立つのは **1 ⟹ 3** と **2 ⟹ 1** の 2 つであり、この 2 つで足りる。1 ⟹ 3 が「節点を残すのが安全で
  ある」を、2 ⟹ 1 の対偶が「節点を落とすのが安全である」を与える。**3 ⟹ 2 と 3 ⟹ 1 は偽である。**
  `Λ(u)` に inhabited な leaf が 1 つも無い活性化では節 3 が空虚に真になり、そこから何も出ない
  (`p15-ownership-uniformity.md` の反例 R2)。`Inh(v, u) ≠ ∅` を足せば 3 つは同値になるが、下流はそれを
  要らない。

  節 2 と節 3 が inhabited (D16) な leaf に限るのは、inhabited でない leaf が参照を持たないからである
  (A5)。限定を外すと節 2 が偽になる本体が作れる: `unbox union` の変位のうち、`result_prov` が `⊥` と
  宣言する側の leaf は `origin` が自分自身を名指すので、`owns_object` はそれを所有と答える。その leaf は
  実行時に存在せず、`Retain`/`Release` は触れない (同ファイルの反例 R1)。

  1 ⟺ 3 は「節点を残すのが安全である」を与え、¬1 ⟹ ¬2 は「節点を落とすのが安全である」を与える。借用版の
  `rewrite_rc` は `owns_unit` が偽の unit の `Retain`/`Release` を丸ごと落とすので (P10)、落とした先に所有
  している leaf が 1 つでも残っていれば参照が漏れる。**包含では言えない。** `origin_inner` は
  `Binding::Param` の腕で path を降りずに `Exactly((v, path))` を返すので、`v` がパラメータのとき
  `origin(v, u) = Exactly((v, u))` と `origin(v, λ) = Exactly((v, λ))` は候補が別である。両者が一致するのは
  `truncate_to_unit` を掛けた後であり、`owns_object` がその truncate を内側で行う (P7e)。

  **site と不動点に限るのは、この同値が `level_ownership` が作るものだからである。** 任意の `(v, u)` に
  ついては成り立たない。`origin_inner` の `Binding::Llvm` の腕で、unit の側は
  `origin_from_leaves_under` がオペランドの leaf を `truncate_to_unit(args[j].ty, ・)` で unit へ丸めてから
  辿るのに対し、leaf の側は `as_arg_projection` が leaf のままオペランドを辿る。オペランドの unit の下に、
  どの結果 leaf も名指さない leaf があると、2 と 3 が離れる。`union_as` が unbox union に対してその形の
  宣言を出す -- 結果の各 leaf を `Arg(0, [variant_idx] ++ path)` と宣言し、他の変位の leaf をどこにも
  名指さない (`CODE src/fixstd/builtin.rs: InlineLLVMUnionAsBody::result_prov`)。

  `owns_unit` を呼ぶ位置が `levelled_sites` の挙げる site を出ないことは、この命題の証明の中で示す
  (`CODE src/rc_ir/borrow.rs: levelled_sites`, `RewriteCtx::rewrite_rc`, `call_rc`, `any_owned_unit`)。

- **P8** (推論の停止性と安全性)。`infer_ownership` は停止する。その不動点が返す集合 `owned_leaves` は、
  次を満たす。ある関数のある leaf の参照が、その関数のある実行路で D9 の意味で消費されるならば、その leaf の
  `origin` の候補であるパラメータ leaf はすべて `owned_leaves` に入っている。

  「候補である」の形で述べるのは、D8 より同じオブジェクトへの参照が互いに区別されないからである。2 つの
  パラメータ leaf が同じ参照を持つ位置では、消費されたのがどちらの leaf の参照かは決まらない。

  この言明が読む所有と借用の割り当ては、**`infer_ownership` が計算している途中の割り当て**である。A1 の
  割り当て (全所有) で読むと偽になる入力プログラムが作れる (`p20-borrow-ify.md` の L7)。

  **消費のうち `App` の引数の位置は除く。** そこは `rhs_consumes` が leaf の粒度で `owned_leaves` を引く
  のに D14 の所有は unit の粒度なので、leaf を 2 つ以上持つ unit -- unbox union -- で 2 つが食い違う。
  P14 も `p13-disposals-and-pending.md` の `L16` も、この位置を P8 では扱わず、`call_rc` が置く節点で扱う。
- **P9** (複製は名前替えである)。`clone_func` が作る借用版の本体は、元の本体の束縛変数を一斉に付け替えた
  ものであり、それ以外の違いを持たない。さらに、複製が導入する名前は、入力プログラムのどの束縛名とも異なる。

  後半が要るのは、A6 (名前の一意性) の果たす者が `fresh_rename_function` であり、それを呼ぶのが証明対象の
  `borrow_ify` 自身だからである。A6 は `borrow_ify` の入力にかかる仮定であり、その出力について名前が一意で
  あることは、ここで示すべき事柄である。オブジェクトの名前 (`VarPath`) は束縛名を含むので、複製と原本の
  名前が衝突すれば別のオブジェクトが 1 つの名前を共有し、`cancel` は誤った対を消す。
- **P10** (借用版が落とす RC 節点)。借用版の `rewrite_rc` は、`Retain(v, π)` / `Release(v, π)` を、
  `units_under(ty(v), π)` のうち `owns_unit(v, ・)` が真である unit の節点の列に置き換える。所有しない unit の
  節点は残らない。
- **P11** (呼び出し側の補正)。`call_rc` は、呼び出し元が借用し呼び出し先が所有する unit には前に `Retain` を、
  呼び出し元が所有し呼び出し先が借用する unit には後に `Release` を置き、それ以外には何も置かない。

  この節点が義務集合 (D10) の食い違いを**ちょうど**埋めることは、別の主張であり、P14 が示す。`Retain(a, u)` は
  `u` の下の inhabited なすべての leaf の参照を作るので、`owns_unit` が返す 1 つの真偽値が leaf ごとの所有と
  食い違うと収支が合わない。それが #530 であり、P7d がその食い違いが起きないことを述べる。
- **P12** (振り分けの安全性)。`route` が借用版へ回すのは、末尾位置でない呼び出しか、所有する unit を持つ引数を
  1 つも持たない呼び出しだけである。また `route` が返す呼び出し先は、元の呼び出し先と同じ関数の版である
  (元の版そのものか、その `borrow_versions` の像)。呼び出し先が入力の関数を名指すとき、返る名前は出力の
  `funcs` の鍵である。局所変数を経由する間接呼び出しでは `route` は呼び出し先をそのまま返し、その名前は
  どちらの `funcs` の鍵でもない。
- **P13** (注釈の一致)。出力の各版の `borrowed_units` は、その版のパラメータ・capture の unit のうち
  `owned_units` に入らないものの集合に一致する。
- **P14** (`borrow_ify` は RC 規律を保存する)。D12 の意味で RC 規律を満たし、かつ A1 と A2 を満たすプログラムを
  入力とすると、`borrow_ify` の出力は D12 の意味で RC 規律を満たす。

### 層 3 -- `cancel` の走査

- **P15** (節点と `NodeId`)。`cancel` の入力すなわち `borrow_ify` の出力の各本体について、相異なる位置は
  相異なる `NodeId` を持つ。また `CancelAnalysis::walk` は本体の各位置をちょうど 1 回訪れる。

  前半は `RcExprNode` 一般の性質ではない。`RcExprNode` は式を `Arc` で共有できるので、1 つの木の 2 つの位置が
  同じ `Arc` を指す本体は表現できる。成り立つのは `RewriteCtx::rewrite` が出力の各位置に `expr_node`
  (`Arc::new`) で新しい割り当てを作るからである (`CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner`)。
- **P16** (`pending` の不変条件)。走査中の各位置において、`pending` は次を満たす。
  (a) 各要素の `node` は、その位置までに訪れた `Retain` 節点である。(b) 各要素の `outstanding` は空でなく、
  その `Retain` の `ActRefs` に含まれる。(c) 1 つの `Retain` 節点は `pending` に高々 1 回現れる。
  (d) `pending` の並びは、訪れた順である (後ろほど新しい)。(e) `pending` から取り除かれた `Retain` は、
  次の 3 つのいずれかである。(e1) `outstanding` が空になった。(e2) `needed_retains` に入った。
  (e3) その除去は `merge` によるものであり、各アームへ渡った複製の側に同じ `Retain` の除去事象があって、
  それらがすべて (e1)(e2)(e3) のいずれかである。

  (e3) は落とせない。`Retain` の後の `Match` のすべてのアームがその `Retain` を完全に un-bump すると、
  `merge` はそれを `uniform` にも `needed_retains` にも入れず、`merged` にも入れない。このとき
  `pending_in` の側の `outstanding` は空でない (減ったのは各アームに渡った複製である)。A9 より アームは
  1 つ以上あるので、(e3) の展開は有限で、葉は (e1) か (e2) である。
- **P17** (`un_bump` の正しさ)。`un_bump(pending, R)` の返り値は次で決まる。`R` とオブジェクトを共有する
  要素が `pending` に無ければ `NoBracket` で、`pending` は変わらない。あって、そのうち最も後ろの要素
  (最内) の `outstanding` が `R` を `covers` しなければ `OutsideBracket` で、`pending` は変わらない。
  covers すれば `InBracket(t)` で、`t` はその要素の `node` であり、その要素の `outstanding` から `R` が
  引かれ、空になればその要素が取り除かれる。他の要素は変わらない。
- **P18** (`merge` の後に残るもの)。`merge` の返す `pending` に残る `Retain` は、`pending_in` に在り、
  いずれかのアームの出口に現れ、かつすべてのアームの出口に同じ `outstanding` で現れるものだけである。
  いずれかのアームの出口に現れてこの条件を満たさない `Retain` は `needed_retains` に入る。どのアームの出口にも
  現れない `Retain` は、この呼び出しでは `needed_retains` にも返り値にも入らない (走査の他の位置が
  `needed_retains` に入れることは妨げない)。

- **D27 (bump の帰属)** -- 定義であって命題ではないが、P18b と P18a の言明がこれに立つのでここに置く。
  実行路 `ρ` と、走査中の位置 -- **節点の訪問の入口** -- を固定する。`pending` の各要素 `p` について、
  多重集合 `B(p, ρ)` を次で定める。

  - `p` が `Retain(v, π)` の訪問で `pending` に入るとき、`B(p, ρ)` は、`π` の下の inhabited (D16) かつ
    計数下 (D26) の各 leaf を `origin` の identity で名付けて数えたものである。P6 より、これはその
    `Retain` が `ρ` で実際に作る参照の多重集合である。
  - `un_bump` が `InBracket` で `p` を選ぶ `Release` の訪問で、その `Release` が `ρ` で実際に処分する
    参照の多重集合を引く。
  - アームへの複製と `merge` は `B(p, ρ)` をそのまま運ぶ。`consume_objects` が `p` を取り除いたときは
    定めるものが無い。ほかのどの操作も `B(p, ρ)` を変えない。

  D8 は同じオブジェクトへの参照を区別しないので、「その `Retain` が作った参照のうちまだ処分されていない
  もの」は帰属を決めないと定まらない。**走査自身の帰属を採るのは、削除の可否を決めるのが走査の
  `outstanding` だからである。**「そのオブジェクトのどの処分も引く」と読むと、P18a は A19 の反例でも
  空虚に成り立ってしまい、P21 を支えない。

- **P18b** (`outstanding` は実行時の bump の上界である)。走査中の各位置と、その位置に至る各実行路 `ρ` に
  ついて、`pending` の各要素 `p` を考える。`p.node` の `Retain` が `ρ` で実際に作った参照のうち、`ρ` 上で
  まだ処分されていないものの多重集合を `B(p, ρ)` とする。このとき `p.outstanding` は `B(p, ρ)` を
  `covers` する。とくに `p.outstanding` が空ならば `B(p, ρ)` も空である。

  `outstanding` は静的な数え上げ (`ActRefs`、D15) から始まり、`un_bump` が静的な数え上げを引く。実行時に
  bump され un-bump されるのは inhabited な leaf の分だけである (P6)。unbox union は 1 つの unit なので、
  その `Retain` の `ActRefs` は変位すべての leaf を数えるのに対し、実行時に bump されるのは活性変位の分
  だけである。左辺と右辺で引かれる量が違うので、この不等式は自明ではない。

- **P18a** (pending の bump は値自身の参照の上に積まれている)。走査中の各位置と、その位置に至る各実行路
  `ρ` について、各**計数下オブジェクト** `O` (D26) を考える。D27 の `B(p, ρ)` を使って
  `n(O) = Σ_p Σ_{o : obj(o) = O} B(p, ρ)[o]` と置き、`n(O) ≥ 1` とする。このとき、その位置での実行時の
  参照カウントは `H(O) ≥ n(O) + 1` である。

  **オブジェクトごとに総和を取るところが load-bearing である。** 名前ごとに `H(o) ≥ n(o) + 1` と書くと
  P21 を支えない。1 つのオブジェクトを指す活性な名前は複数ありうる (第 8 節の #552 の形) ので、名前ごとの
  不等式を足し合わせても `+1` が名前の個数だけ余分に立ってしまい、必要な `H(O) ≥ Σ n(o) + 1` は出ない。

  `+1` が主張の要である。`n(O)` 個の bump を消しても `H(O) ≥ 1` が残る、すなわち削除がオブジェクトの解放を
  早めないこと (P21) は、この `+1` からしか出ない。`+1` の出どころは、`Retain(v, π)` が bump するのが
  `v` の leaf が**すでに持っている**参照だということであり、それを述べるのが A19 である。

  P16 は `pending` の形についての主張で、実行時の参照カウントに触れない。P21 は「削除しても解放が早まらない」
  を示すので、この 2 つを結ぶ段が要る。層 4 の実質はここにある。

- **P18c** (義務集合の側の同じ不等式)。走査中の各位置と各実行路について、各計数下オブジェクト `O` に
  ついて `Obl(O) ≥ n(O)` である。ここで `Obl(O)` は義務集合が持つ `O` への参照の個数、`n(O)` は P18a の
  ものである。

  P18a は参照カウント `H` の話であり、D11 の (S-a) は義務集合 `Obl` の話である。削除は `Obl` からも bump
  を取り除くので、(S-a) を保つにはこちらが要る。A19 (ii) が与える `+1` は、借用する終端の類が初期値 1 を
  `Obl` に持たない分をちょうど吸収するので、`Obl` の側では `+1` が付かない。

### 層 4 -- `cancel` が RC 規律を保存すること

- **P19** (削除される retain の性質)。`cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含む
  すべての実行路において、`t` より後にある、**その位置での** `t` の `outstanding` のオブジェクトを
  `acted_on` に含む消費より前、かつ終端の `Ret` より前に、削除される `Release` 群が `t` の `outstanding` を
  空にする。さらに、`t` とともに削除される各 `Release` は、実行路の上で `t` より後ろにある。

  「その位置での」を落とすと偽になる。`outstanding` を初期値 `ActRefs(t)` と読むと、unbox union の
  `Retain` を `Release` が部分的に un-bump した後で、un-bump 済みの名前を消費する点が `t` の最後の
  `Release` より前に来る本体が作れる。健全性には響かない -- その名前についての欠損は 0 だからである。

  「`t` より後にある」の限定が要る。`t` が `pending` に入るのは `t` の訪問なので、`t` より前にある消費に
  ついては un-bump が起きようがない。

  後半が要るのは、(S-a) が各時点についての条件だからである。路全体での収支が合っていても、`Release` が
  `Retain` より先に実行される路があれば、その時点で `Obl` に無い参照を取り除くことになる。
- **P20** (削除は収支を保つ)。各実行路において、削除される `Retain` が実行時に作る参照の多重集合は、その路で
  実行される削除される `Release` が実行時に処分する参照の多重集合に一致する。
- **P21** (削除は解放後の読みを作らない)。入力が D12 を満たすとき、削除の前後で、対応する 2 つの活性化の
  各読む構文の各位置において解放されているオブジェクトの集合は変わらない。

  「入力が D12 を満たす」が要るのは、解放されたオブジェクトのカウントが後から上がらないことに (S-c) を
  使うからである。

  **「対応する活性化」は、同じ実行路を辿る 2 つの活性化である。**入力と出力で選ばれるアームが同じである
  ことは、この命題の前提であって結論ではない。倒れうる経路が 1 つある -- 一意性の観測 (D18) は参照カウントを
  読むので、削除がカウントを下げると観測値が変わり、その先で選ばれるアームが変わりうる。P26 がその観測が
  悪化しないことを述べるが、P26 は改善の向き (偽から真へ) を許すので、路の対応そのものは別に前提される。
- **P22** (`drop_nodes` の正しさ)。`drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release`
  節点だけを取り除いた木を返し、他の節点の種類・変数・path・並びを変えない。
- **P23** (`cancel` は RC 規律を保存する)。D12 の意味で RC 規律を満たすプログラムを入力とすると、`cancel` の
  出力も D12 の意味で RC 規律を満たす。
- **P24** (D12 が見ない部分の保存)。`borrow_ify` と `cancel` は、D12 が見ない部分について次を満たす。
  `roots` を変えない。各関数の `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` を変えない。
  各グローバル初期化子の `symbol` と `ty` を変えず、`owns_initializer` と `owns_storage` に `true` を書く。
  D1 が述べる呼び出し順により、この書き込みは正しい値を書く。

### 層 5 -- 観測の保存

- **P26** (一意性は悪化しない)。入力の一意性の観測点 (D18) で観測値が真であるすべての実行について、出力の
  対応する観測点 (D19) の観測値も真である。

  逆向きは許す。`unsafe_is_unique` の doc が「最適化の水準によって返り値は変わりうる。最適化が不要な計算を
  取り除き、共有されていた値が一意になることがあるため」と述べており、共有から一意への変化は言語が認めて
  いる (`CODE src/fixstd/std.fix: unsafe_is_unique` の doc)。認められていないのは逆で、一意だったものが
  共有になることである。これは `cancel` の逆写像 -- 観測点を `Retain`/`Release` の対で囲むだけのパス -- が
  D11 を満たしてしまうことへの歯止めである。そのパスはすべての観測値を偽に固定し、`Debug::assert_unique` を
  持つプログラムを止める。

  **この命題が `borrow_ify` について真かどうかは自明ではない。** `call_rc` は、呼び出し元が借用し呼び出し先が
  所有する unit の前に `Retain` を置く。カウントが上がるので、呼び出し先の中の観測が真から偽へ倒れうる形に
  見える。入力では呼び出し元がその参照を所有していたので差し引きが合う可能性はあるが、それは証明が決める。
  偽であれば、`borrow_ify` の欠陥か、この命題を弱める必要があるかの判断になる。

- **P27** (実行の合成)。プログラムのすべての本体が D11 を満たし、A17 と A18 が成り立つならば、その
  プログラムのどの実行においても、次の 3 つが成り立つ。

  - **(R1)** 解放されたオブジェクトの読みは起きない。
  - **(R2)** どのオブジェクトも**高々 1 回**しか解放されない。
  - **(R3)** 正常終了する実行では、最後まで解放されずに残る計数下のオブジェクト (D26) は、環境が持つ参照が
    指すオブジェクトから到達できるものに限る。

  D11 は 1 つの本体の、D3 の意味の静的な実行路についての述語である。依頼者が求めているのは実行についての
  主張なので、本体ごとの述語から実行の主張へ渡る段が要る。呼び出し元が渡した参照が呼び出し先の活性化の初期
  `Obl` になること (D10) を使い、活性化の入れ子に関する帰納で示す。

  **「ちょうど 1 回解放される」とは書けない。** 反例が 3 つある。(i) グローバル状態のオブジェクト (D26) は
  1 度も解放されない (A8)。(ii) 発散する実行と中断する実行では、終端の `Ret` に着かない活性化があるので
  (S-b) が何も言わず、生きたままのオブジェクトが残る。(iii) 正常終了する実行でも、C のエントリ点は
  `run_ios_runner` が作る `IOState` を処分せずに 0 を返す (`CODE src/build/build_object_files.rs:
  build_main_function`, `CODE src/fixstd/builtin.rs: run_ios_runner`)。プロセスが終わるので実害は無いが、
  「ちょうど 1 回」はこの `IOState` について偽である。

  依頼者の言葉での結論 -- 「この 2 つのパスが原因の segv と二重解放は起きない」-- には (R1) と (R2) で
  届く。届かないのは「漏れが無い」の側であり、(R3) が「漏れるとすれば環境が持っているものだけである」と
  いう形でそこを答える。

### 主定理

- **T** (パイプラインが保存するもの)。`split_rc_units` の出力が A1 と A2 を満たすとき、
  `cancel(borrow_ify(・))` の出力について次が成り立つ。

  1. 出力は D12 の意味で RC 規律を満たす (P14 と P23 の合成)。
  2. よって出力のどの実行でも、解放されたオブジェクトの読みは起きず、どのオブジェクトも高々 1 回しか
     解放されない。正常終了する実行で残る計数下のオブジェクトは、環境が持つ参照から到達できるものに限る
     (1 と P27)。
  3. 入力で一意だった観測点は、出力でも一意である (P26)。**この節は現在偽である** -- 第 8 節の #551。
  4. D12 が見ない部分は P24 のとおりに保たれる。

  2 が依頼者の言葉での結論である -- 「この 2 つのパスが原因の segv と二重解放は起きない」。1 だけでは
  そこまで届かない。

## 6. 較正

RC 規律の定義 (D11) が弱すぎないことを、このコードが実際に持っていたバグで確かめる。

**較正に使うバグ**: issue #519 (修正は `853f9756`、マージは `b81cc2c8`)。構造体のフィールドに入った unbox
union (`Option` や `Result`) の payload を 2 回読むと、`-O max` で読んでいる最中に解放されていた。原因は
`origin_from_leaves_under` が、leaf の辿り着いた origin を、読み出した値自身の名前で組み直していたことである。
1 つのオブジェクトが道ごとに 3 つの名前を持ち、`release` が括弧を閉じず、最初の `Retain` が最後の `Release` と
対になり、その対を消した結果、最初の `Release` が読んでいる最中の payload を解放した。

**この定義がそれを弾くこと**: 修正前のコードは D11 の (S-c) に違反する。payload のオブジェクトが解放された後に
読まれるからである。よって P23 (`cancel` は RC 規律を保存する) は修正前のコードについては偽であり、証明は閉じない。
定義は #519 を弾く。

**この定義が弾く先**: 同じ形の誤りは (S-a) にも現れる。参照が 2 度処分されれば (S-a) が破れる。#519 の症状に
二重解放が含まれていたのはこのためである。

**実行できる形**: この違反を捕まえるテストが `src/tests/test_union_rc_shapes.rs` にある。
`test_field_read_twice_memory_safety` (valgrind の下で走る) と `test_field_read_twice_correctness` である。
`853f9756` の `src/rc_ir/ownership.rs` への変更を戻すと、この 2 つが落ちる。

**節ごとの較正。** 性質は 4 つの節を持つ -- (S-a)(S-b)(S-c) と P26 の観測。節ごとに、その節**だけ**を破る
バグで較正しなければ、較正されていない節が残る。現状は次のとおりである。

| 節 | 較正するバグ | 較正するテスト | 状態 |
|---|---|---|---|
| (S-c) 解放後の読み | #519、#529、#545 | `test_field_read_twice_*`、`test_join_payload_*`、`test_union_payload_two_units_*` | 較正済み |
| (S-a) 過剰処分 | #519 (二重解放の症状) | 同上 | 較正済み。ただし #519 は (S-c) も破るので、この節だけを破る例ではない |
| (S-b) 漏れ | #530、#530 の残穴 | `test_split_ownership_unit_memory_safety`、`test_one_variant_owned_memory_safety` | 較正済み |
| P26 観測 | #551 | 無し (`Debug::assert_unique` が `-O max` でだけ止まる) | 較正済み |

**P26 の節は #551 で較正された。** この節は critic が「D11 を満たす間違ったパス」を挙げたことで足された
もので、長らく実在のバグを持たなかった。P26 を証明しようとして見つかった #551 -- 借用化が値を一意から共有へ
倒し、`-O max` でだけ `Debug::assert_unique` が止まる -- がその例である。**その入力は D11 を満たす**
(valgrind でエラー 0、漏れ 0 バイト) のに P26 だけを破るので、この節だけを破る例として要件を満たす。

### (S-c) を強めた記録

**(S-c) はもとは「読む構文が読みうるオブジェクト」だけを扱っていた。** その形では、3 つの節をすべて満たし
ながら 1 つのオブジェクトを 2 回解放する本体が書ける。P27 を証明する作業がこれを見つけた。

```
Let(x, Var(y), Release(y, [], Retain(x, [], Release(x, [], Ret(u)))))
```

`y` は所有するパラメータであり、`x` は `y` の move-bind なので同じオブジェクトを指す。追うと:
初期の `Obl` は `y` の参照 1 つ。`Release(y)` がそれを取り除き、`H` は 0 になってオブジェクトが解放される。
`Retain(x)` は解放されたオブジェクトのカウントを 1 に上げ、`Obl` に参照を 1 つ**加える**。`Release(x)` が
それを取り除き、`H` は再び 0 になってオブジェクトが 2 度目の解放を受ける。

3 つの節はどれも破れない。(S-a) は `Obl` から**取り除く**操作しか見ないので、`Retain` が解放済みの
オブジェクトに触れることを見ない。(S-b) は終端で `Obl` が空なので満たされる。(S-c) は、もとの形では
`Retain` と `Release` を読む構文に数えていなかったので、何も言わない。

**穴は 1 か所であり、直したのは (S-c) の側である。** `Retain` と `Release` が触れる先を (S-c) に入れた。
仮定を足す道 -- 「`Retain` は解放されたオブジェクトに触れない」を A16 として置く -- は採らなかった。それを
果たす者が居らず、穴を動かすだけだからである。(S-c) に入れれば、`borrow_ify` と `cancel` がそれを保つことが
証明の義務になる。

この強化は較正をやり直させない。強めた性質は、弱い形が弾いていたものを弾き続ける。#519、#529、#530、#545 は
どれも強めた (S-c) の下でも規律を破る。

較正をやり直す条件: D11 または P26 を**弱めた**とき。弱めた定義の下で修正前のコードが規律を満たすなら、その
変更は却下する。

## 7. 検証状況

`T への寄与`は、その命題が主定理の鎖に入るかどうかである。入らない命題は、証明できていても T を進めない。

`証明` の欄は**対象コミット `a924f115` に対する**状態である。第 8 節の 4 件の修正 -- とくに `unit_key` を
取り除いた設計変更 -- で、キーを主語にしていた命題は言明ごと書き換わった。書き換わった言明の証明は、その
時点で書き直しになる。

| 命題 | ファイル | T への寄与 | 証明 | 検証 |
|---|---|---|---|---|
| P1, P2 | `p10-leaves-and-units.md` | 有 | 証明済み。`<1>35`/`<1>36` (`unit_key` についての観察) は対象を失った | 未着手 |
| P3, P4 | `p11-origin-soundness.md` | 有 | 証明済み | 未着手 |
| P5 (a), (b) | `p12-identity-and-consumes.md` | 有 | 証明済み (A16 の下で) | 検証済み (指摘 26 件を反映) |
| P5 (c) | `p12-identity-and-consumes.md` | 有 | 証明済み | 検証済み |
| P6, P7 | `p12-identity-and-consumes.md` | 有 | 証明済み (P6 は A16 の下で) | 検証済み |
| P7e, P7d | `p15-ownership-uniformity.md` | 有 | 証明済み | 未着手 |
| P7a | `p15-ownership-uniformity.md` | 有 | 証明済み (1 ⟹ 3 と 2 ⟹ 1。言明は 3 度書き直した) | 未着手 |
| P7c, P7f | `p13-disposals-and-pending.md` | 有 | 証明済み (P7c の言明は 1 度書き直した) | 検証済み |

| P8 | `p20-borrow-ify.md` | 有 | 証明済み (`App` の引数の位置を除く形に狭めた) | 検証済み (指摘 1 件) |
| P9 - P13 | `p20-borrow-ify.md` | 有 | 証明済み | 検証済み (P12 (d) の 1 段が旧版のコードを述べている) |
| P14 | `p20-borrow-ify.md` | 有 | **証明されていない** -- 4 段が偽または不完全 (第 9 節) | 検証済み (指摘 26 件) |
| P15 - P18 | `p30-cancel-walk.md` | 有 | 証明済み | 検証済み (指摘 9 件を反映) |
| P18b | `p13-disposals-and-pending.md` | 有 | 証明済み | 検証済み |
| P18c | `p40-cancel-soundness.md` | 有 | **証明されていない** -- P18a と同じ誤読 (`L42`) | 検証済み |
| P18a | `p13-disposals-and-pending.md` | 有 | **証明されていない** -- 導出が使う A19 (ii) の形は、どのプログラムも満たさない (第 9 節) | 検証済み (指摘 30 件超) |

| P19, P20, P22, P24 | `p40-cancel-soundness.md` | 有 | 証明済み | 検証済み (指摘 18 件) |
| P21, P23 | `p40-cancel-soundness.md` | 有 | **証明されていない** -- P18a に載っており、(S-a) の段も同じ A19 の誤読の上に立つ (第 9 節) | 検証済み |
| P26 (`cancel` の半分) | `p50-observation.md` | 有 | 証明済み | 未着手 |
| P26 (`borrow_ify` の半分) | `p50-observation.md` | 有 | **書き直し待ち** -- 第 2 の反例が出てコードを直した (#551) | 未着手 |
| P27 | `p51-runs.md` | 有 | 証明済み (言明を 1 度書き直した。実行のモデル D22-D28 は同ファイル) | 未着手 |
| (P-insert) | `p60-insert-rc.md` | 有 (A19 の残る半分) | **走行中** | 未着手 |
| T | `p70-main-theorem.md` | -- | 未着手 | 未着手 |

**この証明の唯一の実質的な主張 -- 「節点を消しても RC 規律が保たれる」-- は、まだ証明されていない。**
一度は証明済みと記録したが、検証が覆した。第 9 節が経緯を述べる。

## 8. 発見

証明を書く作業が、対象のコードに 6 件の欠陥を見つけた。4 件は miscompile であり、いずれもバグハントでは
見つかっていなかった。すべて修正済みである。

**#529 (miscompile、修正 `be26b396`、PR #531)。** P3/P4 の証明が閉じない原因はコードにあった。
`origin_from_leaves_under` と `origin_inner` の `Binding::Join` の腕が、内側の `Origin` を `candidates()` で
平坦化するとき、その `identity` を落とす。落ちた名前は消費の側の名前に現れず、`cancel` の `Release` の腕が
その名前の pending な `Retain` に印を付けないので、対でない retain と release が消える。`-O max` で解放後の
メモリを読む Fix プログラムを作って確かめた (`-O none` と `-O basic` は正しい)。

**#530 (漏れ、修正 `211b2d3c`、PR #544)。** P14 の証明が閉じない原因もコードにあった。`owns_unit` は unit
ごとに真偽値を 1 つ答えるが、所有は leaf ごとの性質である。1 つの unit の下の leaf が別々の root に由来して
所有が分かれると、`rewrite_rc` は `Release` を丸ごと落とすか、借りている参照まで処分するかの二択になり、
どちらも誤りである。leaf を 2 つ以上持つ unit は unbox union だけであり、`-O max` で 32 バイトが漏れる Fix
プログラムを作って確かめた (`-O basic` は漏れない)。修正は `level_ownership`: 割れた候補を所有の側へ倒す。

**#530 の残穴 (漏れ、修正 `50e7712d`、PR #546)。** `level_ownership` の発火判定は leaf 粒度で所有を読み、
書き換えが読む `owns_object` は unit 粒度で読む。パラメータの unbox union の一方の変位の leaf だけが所有
されているときに食い違い、`Release` が落ちて参照が漏れる。修正は `owns_object_yet`: 発火判定を
`owns_object` と同じ粒度で書き、`level_ownership` がそれを読む。

**#545 (miscompile、修正 `a924f115`、PR #546)。** `origin_from_leaves_under` は、unit の path の下の leaf が
別々のオペランド unit に辿り着くとき、答えを「読み出した値自身の名前」を identity とする `Join` にする。
すると union 自身のキーはその値の名前になり、payload の leaf を消費する構文のキーは payload の元の名前に
なる。1 つのオブジェクトが 2 つのキーを持ち、`Retain` と消費が別のキーに分かれるので、`cancel` が消費を
またいで対を消す。`Std::Option (a, b)` と `Std::Result e (a, b)` がこの形である。

**同じ族の 3 件目が設計をやり直させた。** #519、#529、#545 はどれも「1 つのオブジェクトが道ごとに 2 つ以上の
名前を持ち、`Retain` と `Release` が別のキーに分かれる」形である。3 件目の応急処置は、`Etc::Works` を
+6.40% 遅くした。根は、キー -- `origin` の identity を unit へ丸めた名前 -- が何を名指す量なのか定まって
いなかったことである。`a924f115` はキーを取り除いた。`cancel` は `Retain` を 1 本の `Vec<PendingRetain>` に
積み、`Release` はオブジェクトを共有する最内の要素と対になる。**対にするのに要る情報は最初から
`outstanding` にあり、キーがそれを潰していた。** LangArena のコーパスで RC 操作は 28 減り、速度は雑音の中で
ある。

**この設計変更で言明が変わった命題。** P5、P7c、P16、P17、P18a、P19。いずれもキーを主語にしていた。
新しい言明はオブジェクト (`VarPath`) を主語にする。

**#551 (一意性が倒れる、修正済み)。** P26 の証明が閉じない原因はコードにあった。借用化は、本体のどこでも
消費されないパラメータ leaf を借用に倒す。借用版はその leaf の `Release` を落とすので、呼び出し元がその
参照を呼び出しの後まで持つ。その結果、借用版の中の一意性の観測点でカウントが入力より 1 大きくなり、
一意だった値が共有として観測される。`Debug::assert_unique` を持つプログラムが `-O max` でだけ止まる
(`-O none` と `-O basic` は止まらない)。`unsafe_is_unique` の doc が認めているのは共有から一意への向き
だけなので、この向きは認められていない。

振り分けの判定 `routing_saves_retain` が引数についての `any` であることが、この欠陥を目立たなくしている。
条件を満たす引数は、`Release` が落ちる引数とも観測される引数とも別でよい。

修正は「一意性の観測点に到達する関数の借用版を作らない」である。代償は実測でゼロだった -- LangArena と
speedtest のどちらでも借用版を 1 つも止めず、命令数はベースラインと一致する。

**この修正には穴が 3 度あり、いずれも証明が見つけた。** 3 度とも症状は同じ「`-O max` でだけ
`Debug::assert_unique` が止まる」であり、テストは 1 度も見つけていない。

1. 到達を**直接呼び出しのグラフだけ**で取っていたので、局所変数が持つクロージャを経た間接呼び出しの先に
   ある観測点に届かなかった。直したのは、間接呼び出しを持つ関数にクロージャが運びうる全関数への辺を張る
   ことである (A7 が呼び出し先の解決を放棄している以上、これが健全な近似)。
2. その「クロージャが運びうる全関数」を `prog.funcs` の本体からしか集めておらず、**グローバル初期化子の
   中で作られるクロージャ**が落ちた。
3. 3 度目に形を変えた。走査を 1 つの閉包に括り出し、**プログラムが持つ本体を全部そこへ通す**
   (`prog.funcs` の各本体と `prog.globals` の各初期化子)。手で在りかを並べる形をやめた。

広げた分の代償はゼロだった (クロージャを多用する 10 ケースで命令数が一致)。回帰テストは 3 本あり、いずれも
対応する辺を外すと落ちる。

**この 3 件は「列挙を手で並べると落とす」という族である。** 同じ族が、キャッシュの鍵が手書きの列挙で
4 つ漏れていた件にある。落とした列挙が安全側に倒れるなら気付かないが、この門は落とすと**借用版を通して
しまう**向きなので、落とした分だけ穴が開く。

**#552 (`cancel` の対の判定の非対称、修正しない)。** P18a の証明が閉じない原因はコードにあった。`Release` の
訪問は `other_objects` -- その節点が触れうる、identity 以外の候補 -- を `consume_objects` へ渡すのに、
`Retain` の訪問はそれを記録しない。よって `Join` の値の `Retain` は identity (`Match` の束縛変数の名前) だけを
名指し、アームが作った値を名指す消費がその `Retain` を素通りする。括弧の中で消費が起きたまま対が消え、
消費先が解放したオブジェクトを束縛変数が読む形になる。反例 `C1` は `p13-disposals-and-pending.md` の
第 7.5 節にある。

**実在の入力にこの形が現れることは測った。** `cancel` が消した retain のうち、pending の間に候補を消費された
ものは、LangArena で 11 件ある (`HashMap::insert` の 5 つの特殊化、`Std::loop` のクロージャ特殊化、
`CompressArith::_code`)。**ただし観測可能な障害は再現できていない。** `insert_rc` はアーム本体の値が
`Match` の後で live なら、そのアームの中に `Retain` を置く。その参照が P18a の `+1` を与えるので、
`C1` の形をそのまま Fix のソースから作ることはできなかった (2 通り試した)。

**修正を書いて測り、外した。** `PendingRetain` に候補を持たせて `consume_objects` に見せると、
`cp_lib_dijkstra` の命令数が **+9.73%**、`cp_lib_bipartite` が +1.93%、`cp_lib_lsegtree` が +1.05% 増えた
(`-O experimental`、変更を 1 つずつ入れて属性を取った)。出どころは増える参照カウント操作 (静的に +0.85%)
ではなく、消せなかった retain が値を Unique のまま残さないので `unique_check_elim` が特殊化を諦めることで
ある。

**その後、この修正では P18a が閉じないことが分かった。** `p13-disposals-and-pending.md` の第 7.5.8 節の
反例 `C2` は `Match` を 1 つも含まず、`origin` はどの leaf についても `Exactly` を返すので `other_objects`
はどこでも空である。ずれを作るのは `Join` ではなく `App` -- 所有する引数を取って返す関数が、参照を別の
別名類へ移すこと -- であり、候補を記録しても届かない。**すなわち外した修正は、代償を払ったうえで問題を
閉じもしなかった。**

P18a が要る前提は A19 として書き、その半分 (`borrow_ify` の側) は証明されている。残る半分は `insert_rc`
についての命題であり、この証明の対象の外にある。**この issue は「コードを直す」ではなく「A19 の
`insert_rc` の側を示す」に置き換わる。**

**定義の側の欠陥も 2 件出た。** どちらも P27 を証明する作業が見つけた。1 つは (S-c) が `Retain` の触れる先を
見ておらず、3 つの節をすべて満たしながら二重解放する本体が書けたこと (第 6 節)。もう 1 つは D10 と D11 が
グローバル状態のオブジェクトで場合分けしておらず、**グローバル値を所有位置の引数に渡す本体がすべて (S-a) を
破っていた**こと (D26)。後者は実プログラムにいくらでもある形なので、直す前の定義の下では A1 が偽であった。

## 9. 検証が覆したもの

**P18a、P18c、P21、P23 は、一度「証明済み」と記録した後、検証によって取り消された。** 覆した指摘は 1 つの
形に集約される。

### A19 (ii) を、どのプログラムも満たさない形で使っていた

A19 (ii-b) が言うのは `held ≥ bumps` である。README はそのすぐ後に、なぜ `1 +` を付けられないかを書いて
いる -- **その類の最後の参照が処分された後の時点では `0 ≥ 1` になる**からである。

ところが `p13-disposals-and-pending.md` の第 7.5.3 節も `p40-cancel-soundness.md` の `L42` も、A19 (ii) を
**各時点について `held ≥ 1 + bumps`** の形で置き、その上に P18a と P18c を建てていた。この形は病的な本体で
はなく**正しい本体すべて**で破れる。`p13` 自身が第 7.5.7 節に書いた `f`
(`Let(z, Llvm(zero,[]), Release(b,[],s,Ret(z)))`) で、`Release(b,[])` の直後に `held = 0`、`bumps = 0` と
なり `0 ≥ 1` を要求する。

**すなわち P18a は、どのプログラムも満たさない前提から導かれていた。** 導出の各段は妥当でも、結論は空虚
である。P21 と P23 はその P18a に載り、P23 の (S-a) は `L42` を通じて同じ誤読に直接載る。

**直し方の見当**: `p13` の第 7.5.4 節の `<1>3` は、`bumps ≥ 1` の類を 1 つ選んで他の類には `held ≥ bumps`
だけを使う形で和を取っており、README の弱い形でも通る形をしている。前提の側を弱い形に直し、導出がその形
だけを使うようにすれば閉じる見込みがある。閉じなければ A19 (ii-b) を強める判断になる。

### 併せて崩れた 2 つの土台

- **`obj(C)` が定まっていない。** 別名類の全スロットが同じオブジェクトを指すことを P5 (a) から出している
  が、P5 (a) は **identity** が等しいスロットについての主張であり、別名類は identity ではなく ρ-終端で
  分けた類である。1 つの類が相異なる identity を持つ例は `p13` 自身の `C1` にある (`m` と `p`)。`obj(C)` は
  A19 (i) と P18a の導出が全面的に載る量である。
- **「名前の活性は時点によらない」が偽。** 論拠は値の不変性だけだが、活性の定義は inhabited **かつ計数下**
  を要求する。D26 より計数下からグローバル状態への遷移は在るので、`ρ` の前半で活性・後半で非活性になる
  余地が残る。

### P14 も取り消した

`p20-borrow-ify.md` の検証は、骨格に届く指摘を 4 件出した。

- **L16 の「節点の列は等しい」が偽。** `rewrite_inner` の `App` の腕は `route` の結果で callee を差し替える
  ので、振り分けが起きた節点の callee は両側で別の名前である。10.3 と 10.7 の 2 か所がこの等号に載る。
  救える見込みはある (直接呼び出しの callee は funptr で boxed leaf を持たない) が、その 2 行が書かれて
  いない。
- **「所有されない由来の `n_out` は 0 か 1」が偽。** `App(callee, [x, x])` -- 同じ変数を 2 つの引数位置へ
  渡す呼び出し -- では `call_rc` が `before` に同じ対を 2 回積むので `n_out` は 2 に達する。言明の側
  (非負であること) は保たれるので下流は倒れない。
- **(S-b) の位置を不変条件が覆っていない。** INV は「各節点の入口」についてしか述べないのに、`<1>3` は
  「終端の `Ret` の消費の後」に当てている。(S-b) はまさにその位置の条件である。
- **QED が仮定を落としている。** 10.5/10.6/10.7 は局所仮説 H1・H2 の下でしか示されていないのに、QED は
  仮定の付かない P14 を主張している。

うち H1 は **A19 (ii-a)**、H2 は **A20** と同じものであり、独自に置く必要は無かった。P14 の帰納が立つ
D21 の読み方 (活性化が値の割り当ても運ぶこと) も、README は既に採っている。

### 記録として

**検証を入れるまで、P14・P18a・P18c・P21・P23 は「証明済み」と書かれていた。** A19 については、証明者
2 人が互いに独立に書いて**同じ誤読**をしており、README の A19 の脇に「字義どおりには偽である」と書いて
あっても防げなかった。**引用の突き合わせを課した検証者だけがこれを見つけた。**

3 ファイルの検証で、証明者が「README に無い」と差し戻した項目のうち **16 件が既に在るもの**だった。
差し戻しを読む前に README を読み直させる手順が要る。
