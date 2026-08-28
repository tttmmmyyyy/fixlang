# `borrow_ify` と `cancel` の健全性の証明

## 1. 対象

証明の対象は、コミット `b81cc2c8e859a00cbf007e4f43483a514c813c73` の

- `src/rc_ir/borrow.rs` の `borrow_ify` と `cancel`、およびこの 2 つが呼ぶ同ファイル内の関数
- `src/rc_ir/ownership.rs` の全体 (この 2 つが参照の同一性と消費を決めるのに使うモデル)

である。`split_rc_units` は同じファイルに住むが、`borrow_ify` の前段であって対象ではない。その出力の性質は
仮定 A3 として置く。

この 2 つは、`src/build/build_object_files.rs` の `optimize_rc_program` の中で、`-O max` 以上のとき
`split_rc_units` の直後にこの順で走る。

## 2. 証明の記法

各命題の証明は、`.claude/skills/soundness-proof/SKILL.md` が定める構造化証明の記法で書く。要点だけ再掲する。

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
特殊化するかを指示する (`CODE src/rc_ir/ast.rs: RcState`)。健全性 (D11) は `RcState` を読まないので、以下では
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
実行のある時点における**スロット**とは、対 `(x, λ)` である。ここで `x` はその時点で束縛されている変数、
`λ` は `ty(x)` の inhabited な boxed leaf である。`ty(x)` は `x` に束縛された値の型を表す。スロットが指す
オブジェクトを `obj(x, λ)` と書く。inhabited でない leaf にスロットは無い。

### 3.3 所有、参照、義務

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
だけである。`Retain` と `Release` は参照カウントと状態バイトを触り、`Release` はカウントが 0 になったときに
オブジェクトを走査するが、走査するのはそのオブジェクトが解放される時であって、D11 の (S-c) が禁じるのは他の
オブジェクトの解放後の読みである。

**D8 (参照)**
**参照**とは、1 つのオブジェクトに対する処分義務の 1 単位である。参照は D10 の**生成**によって作られ、D10 の
**消費**または `Release` によって処分される。オブジェクト `o` の参照カウント `H(o)` は、`o` への未処分の参照の
総数に等しい。

同じオブジェクトへの参照どうしは互いに区別されない。義務集合 (D10) はオブジェクトごとの個数を持つ多重集合で
あり、「その `Retain` が作った参照」のような言い方は、オブジェクトごとの個数として読む。移動 (D9) は、どの
参照が移ったかを決めない。

**D9 (消費と移動)**
関数の 1 回の活性化が保持する参照について、次の 2 つを区別する。

**消費**とは、活性化が保持する参照を活性化の外へ渡すか、捨てる構文である。次のものがある。

| 構文 | 消費される leaf |
|---|---|
| `App(callee, args)` | callee の全 boxed leaf、および呼び出し先がその位置の unit を所有する (D14) 引数の leaf |
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
  | `App(callee, args)` の結果の各 boxed leaf | 各 1 つ |
  | `Closure(f, caps)` の結果 (capture object) | 1 つ |
  | boxed 容器の `Destructure` の各名前付きフィールドの各 leaf | 各 1 つ |
  | boxed union の変位アームの payload の各 leaf | 各 1 つ |

  `Llvm` の行は、宣言が空集合 (bottom) のとき、`Fresh` や `Unknown` を含むとき、複数の元を持つときのすべてを
  含む。空集合と宣言された leaf は inhabited にならないので、参照は生じない (A3)。

- **消費** (D9): 消費される inhabited な各 leaf につき参照を 1 つ取り除く。`H` は変わらない (参照は渡された
  先が持つ)。
- **移動** (D9): `Obl` を変えない。

### 3.4 健全性

**D11 (健全な本体)**
本体 `B` が、所有と借用の割り当て (D14) の下で**健全**であるとは、`B` のすべての実行路について次の 3 つが
成り立つことをいう。

- **(S-a) 過剰処分が無い**: `Obl` から参照を取り除くすべての操作について、取り除かれる参照はその時点の `Obl` に
  入っている。
- **(S-b) 漏れが無い**: 実行路の終端の `Ret(v)` において、その `Ret` の消費を行った後の `Obl` は空である。
- **(S-c) 解放後の読みが無い**: D7 の読む構文がその位置で読みうる各オブジェクトは、その時点で解放されていない。

**D12 (健全なプログラム)**
プログラム `P` が**健全**であるとは、`P` のすべての関数の本体と、すべてのグローバル初期化子の `init` が、`P` の
`borrowed_units` が定める所有と借用の割り当て (D14) の下で健全 (D11) であることをいう。

健全性は `RcProgram` の残りの部分について何も言わない。`roots`、`RcFunc` の `fn_ty` / `ret_ty` / `source` /
`inline_into_callers`、`RcGlobalInit` の `symbol` / `ty` / `owns_initializer` / `owns_storage` である。
これらを `borrow_ify` と `cancel` がどう扱うかは P13 と P24 が扱う。

### 3.5 同一性のモデル

**D13 (origin)**
`origin(vars, τenv, x, π)` は、変数 `x` の path `π` にある値が、どの変数のどの path で作られた参照を持つかを
答える関数である (`CODE src/rc_ir/ownership.rs: origin`)。返り値は `Exactly(u, σ)` か
`Join { identity, candidates }` のどちらかである (`CODE src/rc_ir/ownership.rs: Origin`)。`identity` は
`VarPath` を 1 つ、`candidates` は `VarPath` の空でない集合を持つ。以下では `vars` と `τenv` が文脈から
定まるとき `origin(x, π)` と書く。

D13 は `origin` が何を返すかを述べるだけであり、その返り値が実行時の参照とどう対応するかは P3 と P4 が述べる。

**D15 (キーと触れる参照)**
`unit_key(x, π) = unit_of(origin(x, π).identity())` を、その leaf の参照が数えられる**キー**と呼ぶ
(`CODE src/rc_ir/ownership.rs: unit_key`, `unit_of`)。`unit_of(root, path)` は、`root` の型が `vars` に
記録されていればその型で `path` を `truncate_to_unit` し、記録されていなければ `path` をそのまま返す。

`acted_references(v, π)` は、`π` の下の**すべての** boxed leaf (inhabited でないものを含む) について、その
leaf の `origin(v, leaf).identity()` を数えた `Map<VarPath, usize>` を返す
(`CODE src/rc_ir/ownership.rs: acted_references`)。これは静的な数え上げであり、実行時に触れる参照との関係は
P6 が述べる。

`acted_unit_keys(x, π)` は、`origin(x, π).acted_on()` の各要素を `unit_of` で写したものである
(`CODE src/rc_ir/ownership.rs: acted_unit_keys`, `Origin::acted_on`)。

## 4. 仮定

各仮定には、それを果たす者を書く。

**A1 (入力の健全性)** -- 果たす者: 前段のパス (`insert_rc`)。
`borrow_ify` に渡されるプログラムは D12 の意味で健全である。またそのプログラムのすべての関数の
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
| 単一の `Arg(j, σ)` | 第 `j` オペランドの leaf `σ` と**同じ参照**。新しい参照を作らない |
| 単一の `Fresh` | 新しく割り当てたオブジェクトへの新しい参照 |
| 単一の `Unknown` | 既存のオブジェクトへの新しい参照 (retain を伴う読み出し) |
| 複数の元 | 実行路ごとにそのいずれか。いずれの路でも新しい参照 |

`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの参照を処分しない。

複数の元を宣言する op は、このコミットのプログラムには存在しない。`impl LLVMGen for` は 78 個あり、
`result_prov` を override するのは 29 個、その 29 個が leaf に置く集合はすべて要素数 0 か 1 である
(`sole_origin` / `Set::default()` / `uniform` / `uniform_bottom` / `fresh_under` のいずれかで作られる)。
複数元を作るのは `Provenance::join` と `compose` であり、これらは解析の側であって宣言ではない。表がこの行を
持つのは、`LLVMGen::result_prov` の型と doc がそれを許すからである。

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

**A6 (名前の一意性)** -- 果たす者: lowering と `fresh_rename_function`。
プログラム中のすべての束縛変数の名前は相異なる。よって変数名は束縛を一意に決める。

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
move-bind の両辺の型、アームの結果と `Match` の束縛変数の型、payload と変位の型、`Destructure` の
フィールド変数とフィールドの型、`Match` の scrutinee が union であること、`Destructure` の容器が構造体で
あること、同じ名前の `RcVar` が持つ型が一致すること。**このコミットにこれを検査するコードは無い**
(`validate` は構造だけを見る)。

## 5. 命題

依存の順に並べる。各命題は自分より小さい番号の命題だけを引用してよい。

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
- **P5** (キーと参照の関係)。1 つの関数の 1 回の活性化について、次の 3 つが成り立つ。
  - **(a)** 1 つの実行路の 1 つの位置において同じ参照を持つ 2 つのスロットで、一方から他方への別名の道が
    `Match` のアーム本体の `Ret` の辺を含まないならば、両者の `unit_key` は等しい。
  - **(b)** アーム本体の `Ret(x)` が `x` の参照を `Match` の束縛変数 `m` へ移す辺について、
    `origin(x, λ).candidates()` は `origin(m, λ).candidates()` に含まれる。
  - **(c)** (N) `acted_unit_keys(v, π)` は、`acted_references(v, π)` が名指すオブジェクトのうち inhabited な
    leaf に由来するものを、`unit_of` で写した上ですべて含む。

  (a) の制限は外せない。アーム本体の `Ret` の辺は identity を保たず、`m` と `x` が同じ参照を持つのに
  `unit_key` が `(m, λ)` と `(x, λ)` に分かれる本体が作れる (`p12-keys-and-consumes.md` の反例 R1)。
  `cancel` が要るのは (a) ではなく (b) と (c) である。
  **(c) はこのコミットのコードでは偽であり、この命題は閉じない** (`p11-origin-soundness.md` の第 4 節)。
- **P6** (`acted_references` は静的な上位近似である)。1 つの関数の 1 回の活性化について、`acted_references(v, π)` が返す `Map` は、`π` の下の
  すべての boxed leaf を `origin` の identity で名付けて数えたものである。実行時に `Retain(v, π)` が作る
  参照の多重集合は、この数え上げを inhabited な leaf に制限したものに等しく、`Release(v, π)` が処分する
  参照の多重集合も同じものに等しい。
- **P7** (消費の網羅性)。D9 の意味で消費する構文はすべて `collect_consumes` が報告する。また
  `collect_consumes` が報告して D9 が消費としないものは、アーム本体の `Ret` に限る。

### 層 2 -- `borrow_ify`

- **P8** (推論の停止性と安全性)。`infer_ownership` は停止する。その不動点が返す集合 `owned_leaves` は、
  次を満たす。ある関数のあるパラメータ leaf の参照が、その関数のある実行路で D9 の意味で消費されるならば、
  その leaf は `owned_leaves` に入っている。
- **P9** (複製は名前替えである)。`clone_func` が作る借用版の本体は、元の本体の束縛変数を一斉に付け替えた
  ものであり、それ以外の違いを持たない。
- **P10** (借用版が落とす RC 節点)。借用版の `rewrite_rc` は、`Retain(v, π)` / `Release(v, π)` を、
  `units_under(ty(v), π)` のうち `owns_unit(v, ・)` が真である unit の節点の列に置き換える。所有しない unit の
  節点は残らない。
- **P11** (呼び出し側の補正)。`call_rc` が置く前後の RC 節点は、呼び出し元と呼び出し先の所有権の食い違いを
  ちょうど埋める。すなわち、呼び出し元が借用し呼び出し先が所有する unit には前に `Retain` を、呼び出し元が
  所有し呼び出し先が借用する unit には後に `Release` を置き、それ以外には何も置かない。
- **P12** (振り分けの安全性)。`route` が借用版へ回すのは、末尾位置でない呼び出しか、所有する unit を持つ引数を
  1 つも持たない呼び出しだけである。
- **P13** (注釈の一致)。出力の各版の `borrowed_units` は、その版のパラメータ・capture の unit のうち
  `owned_units` に入らないものの集合に一致する。
- **P14** (`borrow_ify` の健全性)。D12 の意味で健全で、かつ A1 と A2 を満たすプログラムを入力とすると、
  `borrow_ify` の出力は D12 の意味で健全である。

### 層 3 -- `cancel` の走査

- **P15** (節点と `NodeId`)。`cancel` の入力すなわち `borrow_ify` の出力の各本体について、相異なる位置は
  相異なる `NodeId` を持つ。また `CancelAnalysis::walk` は本体の各位置をちょうど 1 回訪れる。

  前半は `RcExprNode` 一般の性質ではない。`RcExprNode` は式を `Arc` で共有できるので、1 つの木の 2 つの位置が
  同じ `Arc` を指す本体は表現できる。成り立つのは `RewriteCtx::rewrite` が出力の各位置に `expr_node`
  (`Arc::new`) で新しい割り当てを作るからである (`CODE src/rc_ir/borrow.rs: RewriteCtx::rewrite_inner`)。
- **P16** (`pending` の不変条件)。走査中の各位置において、`pending` は次を満たす。
  (a) `pending[k]` の各要素の `node` は、その位置までに訪れた `Retain` 節点であり、その `unit_key` は `k` で
  ある。(b) 各要素の `outstanding` は空でない。(c) 1 つの `Retain` 節点は `pending` 全体で高々 1 か所に
  現れる。(d) `pending[k]` の並びは、訪れた順である (後ろほど新しい)。(e) `pending` から取り除かれた
  `Retain` は、次の 3 つのいずれかである。(e1) `outstanding` が空になった。(e2) `needed_retains` に入った。
  (e3) その除去は `merge` によるものであり、各アームへ渡った複製の側に同じ `Retain` の除去事象があって、
  それらがすべて (e1)(e2)(e3) のいずれかである。

  (e3) は落とせない。`Retain` の後の `Match` のすべてのアームがその `Retain` を完全に un-bump すると、
  `merge` はそれを `uniform` にも `needed_retains` にも入れず、`merged` にも入れない。このとき
  `pending_in` の側の `outstanding` は空でない (減ったのは各アームに渡った複製である)。A9 より アームは
  1 つ以上あるので、(e3) の展開は有限で、葉は (e1) か (e2) である。
- **P17** (`un_bump` の正しさ)。`un_bump(pending, k, R)` の返り値は次で決まる。`pending` にキー `k` の項目が
  無ければ `NoBracket` で、`pending` は変わらない。あって、最内の要素の `outstanding` が `R` を `covers`
  しなければ `OutsideBracket` で、`pending` は変わらない。covers すれば `InBracket(t)` で、`t` は最内の要素の
  `node` であり、その要素の `outstanding` から `R` が引かれ、空になればその要素が取り除かれ、スタックが空に
  なればキーが取り除かれる。
- **P18** (`merge` の後に残るもの)。`merge` の返す `pending` に残る `Retain` は、`pending_in` に在り、
  いずれかのアームの出口に現れ、かつすべてのアームの出口に同じ `outstanding` で現れるものだけである。
  いずれかのアームの出口に現れてこの条件を満たさない `Retain` は `needed_retains` に入る。どのアームの出口にも
  現れない `Retain` は、この呼び出しでは `needed_retains` にも返り値にも入らない (走査の他の位置が
  `needed_retains` に入れることは妨げない)。

### 層 4 -- `cancel` の健全性

- **P19** (削除される retain の性質)。`cancelled()` が返す集合に含まれる `Retain` `t` について、`t` を含む
  すべての実行路において、`t` の `unit_key` を `acted_unit_keys` に含む消費より前、かつ終端の `Ret` より前に、
  削除される `Release` 群が `t` の `outstanding` を空にする。
- **P20** (削除は収支を保つ)。各実行路において、削除される `Retain` が実行時に作る参照の多重集合は、その路で
  実行される削除される `Release` が実行時に処分する参照の多重集合に一致する。
- **P21** (削除は解放後の読みを作らない)。削除の前後で、各読む構文の各位置において解放されているオブジェクトの
  集合は変わらない。
- **P22** (`drop_nodes` の正しさ)。`drop_nodes(B, S)` は、`B` の `NodeId` が `S` に入る `Retain`/`Release`
  節点だけを取り除いた木を返し、他の節点の種類・変数・path・並びを変えない。
- **P23** (`cancel` の健全性)。D12 の意味で健全なプログラムを入力とすると、`cancel` の出力は D12 の意味で
  健全である。
- **P24** (健全性が見ない部分の保存)。`borrow_ify` と `cancel` は、D12 が見ない部分について次を満たす。
  `roots` を変えない。各関数の `fn_ty` / `ret_ty` / `params` の型 / `inline_into_callers` を変えない。
  各グローバル初期化子の `symbol` と `ty` を変えず、`owns_initializer` と `owns_storage` に `true` を書く。
  D1 が述べる呼び出し順により、この書き込みは正しい値を書く。

### 主定理

- **T** (パイプラインの健全性)。`split_rc_units` の出力が A1 と A2 を満たすとき、`cancel(borrow_ify(・))` の
  出力は D12 の意味で健全であり、D12 が見ない部分について P24 を満たす。P14、P23、P24 の合成である。

## 6. 較正

健全性の定義 (D11) が弱すぎないことを、このコードが実際に持っていたバグで確かめる。

**較正に使うバグ**: issue #519 (修正は `853f9756`、マージは `b81cc2c8`)。構造体のフィールドに入った unbox
union (`Option` や `Result`) の payload を 2 回読むと、`-O max` で読んでいる最中に解放されていた。原因は
`origin_from_leaves_under` が、leaf の辿り着いた origin を、読み出した値自身の名前で組み直していたことである。
1 つのオブジェクトが道ごとに 3 つの名前を持ち、`release` が括弧を閉じず、最初の `Retain` が最後の `Release` と
対になり、その対を消した結果、最初の `Release` が読んでいる最中の payload を解放した。

**この定義がそれを弾くこと**: 修正前のコードは D11 の (S-c) に違反する。payload のオブジェクトが解放された後に
読まれるからである。よって P23 (`cancel` の健全性) は修正前のコードについては偽であり、証明は閉じない。
定義は #519 を弾く。

**この定義が弾く先**: 同じ形の誤りは (S-a) にも現れる。参照が 2 度処分されれば (S-a) が破れる。#519 の症状に
二重解放が含まれていたのはこのためである。

**実行できる形**: この違反を捕まえるテストが `src/tests/test_union_rc_shapes.rs` にある。
`test_field_read_twice_memory_safety` (valgrind の下で走る) と `test_field_read_twice_correctness` である。
`853f9756` の `src/rc_ir/ownership.rs` への変更を戻すと、この 2 つが落ちる。

較正をやり直す条件: D11 を変えたとき。変えた定義の下で修正前のコードが健全になるなら、その変更は却下する。

## 7. 検証状況

| 命題 | ファイル | 証明 | 検証 |
|---|---|---|---|
| P1, P2 | `p10-leaves-and-units.md` | 証明済み | 未着手 |
| P3, P4 | `p11-origin-soundness.md` | **閉じない** -- P5 (c) がコードで偽 | 未着手 |
| P5, P6, P7 | `p12-keys-and-consumes.md` | P6, P7 は証明済み。P5 は (a) を制限つきで証明し (c) は未 | 未着手 |
| P8 - P14 | `p20-borrow-ify.md` | 未着手 | 未着手 |
| P15 - P18 | `p30-cancel-walk.md` | 証明済み | 未着手 |
| P19 - P24, T | `p40-cancel-soundness.md` | 未着手 | 未着手 |

## 8. 発見

**#529 (miscompile)。** P3/P4 の証明が閉じない原因はコードにあった。`origin_from_leaves_under` と
`origin_inner` の `Binding::Join` の腕が、内側の `Origin` を `candidates()` で平坦化するとき、その
`identity` を落とす。落ちた名前は `acted_unit_keys` に現れず、`cancel` の `Release` の腕がその名前の
pending な `Retain` に印を付けないので、対でない retain と release が消える。`-O max` で解放後のメモリを
読む Fix プログラムを作って確かめた (`-O none` と `-O basic` は正しい)。P5 (c) はこのコードでは偽であり、
層 4 (P19-P24) はこれが直るまで着手しない。

**測定の記録。** (c) を `develop_mode` の assertion にしたブランチが `n-probe` (`5245625a`) にある。
#519 の修正を戻すと発火し (較正)、修正を入れた状態では全スイート 1,638 + 178 本で 1 度も発火せず、
#529 の再現プログラムで発火する。
