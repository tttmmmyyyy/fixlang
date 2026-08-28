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

### 3.1 中間表現

**D1 (プログラム)**
RC IR のプログラムとは、名前から関数への写像 `funcs`、グローバル値の初期化子の列 `globals`、外から到達される
名前の集合 `roots` の 3 つ組である (`CODE src/rc_ir/ast.rs: RcProgram`)。関数は、名前、パラメータの列
`params`、capture (無い場合がある)、本体 `body`、および借用する単位の集合 `borrowed_units` を持つ
(`CODE src/rc_ir/ast.rs: RcFunc`)。グローバル初期化子は、パラメータも capture も持たない本体である。

**D2 (本体の木)**
本体は式の木である。節点は次の 6 種である (`CODE src/rc_ir/ast.rs: RcExpr`)。

| 節点 | 意味 | 継続 |
|---|---|---|
| `Let(x, rhs, k)` | `rhs` の値を `x` に束縛する | `k` |
| `Retain(v, π, s, k)` | `v` の `π` の下の各 boxed leaf の参照カウントを 1 上げる | `k` |
| `Release(v, π, s, k)` | `v` の `π` の下の各 boxed leaf の参照カウントを 1 下げる | `k` |
| `Destructure(c, fs, s, k)` | 容器 `c` をフィールドに分解し、各 `(i, x)` の `x` に第 `i` フィールドを束縛する | `k` |
| `Eval(v, k)` | `v` を効果のために評価して捨てる | `k` |
| `Ret(v)` | この式の値は `v` である | 無し |

`Let` の右辺 `rhs` は、`Var`、`App`、`Closure`、`Llvm`、`Match` の 5 種である
(`CODE src/rc_ir/ast.rs: RcRhs`)。`Match(scrut, arms)` の各アームは、変位番号 (catch-all のときは無し)、
payload 変数、およびアーム本体を持つ。アーム本体もまた本体の木で、その `Ret` はアームの値を `Match` の
束縛変数へ渡す。

節点の継続はどれも 1 つで、分岐は `Match` のアームだけである。よって本体は木であり、繰り返しは呼び出しでしか
作れない。

**D3 (実行路)**
本体 `B` の**実行路**とは、`B` の根から始まり、各節点でその継続へ進み、`Let(x, Match(v, arms), k)` に
出会ったときはアームを 1 つ選んでそのアーム本体の路を辿ってから `k` へ進む、という節点の有限列である。
`Ret` に着いたら終わる。D2 より `B` は木なので、実行路は有限であり、その本数はアームの選び方の個数である。

「**`n` の後**」とは、`n` を含む実行路の上で `n` より後ろにある位置をいう。「**すべての路で**」とは、
その節点を含むすべての実行路について、という意味である。

### 3.2 値の構造

**D4 (boxed leaf)**
型 `τ` の値が保持する参照の位置を **boxed leaf** と呼び、その全体を `boxed_leaf_paths(τ)` が列挙する
(`CODE src/rc_ir/leaf_map.rs: boxed_leaf_paths`)。leaf は、値の根からの
フィールド添字の列 (`FieldPath`) で表す。列挙の規則は、参照を持たない値は leaf を持たず、クロージャは capture の
1 つ、boxed な値は自分自身の 1 つ、unbox 集約 (構造体・タプル・union) は各フィールド (union は各変位の payload)
の下へ降りる、`Array` は自分自身の 1 つ、である。

**D5 (RC unit)**
1 回の参照カウント操作が対象にできる位置を **RC unit** と呼び、型の全体を `rc_units(τ)` が列挙する
(`CODE src/rc_ir/ownership.rs: rc_units`)。どの型が unit を担うかは `unit_step` が 1 か所で決める
(`CODE src/rc_ir/ownership.rs: unit_step`)。leaf と unit の違いは union にある。unbox union は
**1 つの unit** であり (物理的な操作はタグで分岐しなければならない)、しかしその leaf は各変位の payload の中に
ある。

`truncate_to_unit(τ, π)` は leaf path `π` をそれが属する unit の path へ切り詰める
(`CODE src/rc_ir/ownership.rs: truncate_to_unit`)。`units_under(τ, π)` は path `π` の下にある unit を
列挙する (`CODE src/rc_ir/ownership.rs: units_under`)。

**D6 (スロット)**
実行のある時点における**スロット**とは、対 `(x, λ)` である。ここで `x` はその時点で束縛されている変数、
`λ ∈ boxed_leaf_paths(ty(x))` である。スロットが指すオブジェクトを `obj(x, λ)` と書く。

### 3.3 参照と義務

**D7 (オブジェクトと参照カウント)**
実行時のヒープは**オブジェクト**の集合であり、各オブジェクト `o` は**参照カウント** `H(o) ≥ 0` を持つ。
`H(o)` が 0 になったオブジェクトは**解放される**。オブジェクトを**読む**とは、そのオブジェクトが占める記憶域を
読むことをいう。解放されたオブジェクトを読むことを**解放後の読み**と呼ぶ。

**D8 (参照)**
**参照**とは、1 つのオブジェクトに対する処分義務の 1 単位である。参照は、割り当て、`Retain`、または boxed な
容器からの読み出しによって作られ、`Release` または消費 (D9) によって処分される。`H(o)` は `o` への未処分の
参照の総数である。

**D9 (消費)**
関数の活性化が保持する参照を、他所へ渡すか捨てるかして手放す構文を**消費**と呼ぶ。どの構文がどの leaf の参照を
消費するかは `collect_consumes` / `rhs_consumes` / `destructure_consumes` が定める
(`CODE src/rc_ir/ownership.rs: collect_consumes`, `rhs_consumes`, `destructure_consumes`)。

**D10 (義務集合)**
関数の 1 回の活性化について、実行路上の各位置における**義務集合** `Obl` を、参照の多重集合として次で定める。

- 初期値: 所有するパラメータ・capture の unit (D14) の各 leaf につき 1 つ。借用する unit の leaf は入れない。
- `Retain(v, π)`: `π` の下の各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ加える。同時に `H` を 1 上げる。
- `Release(v, π)`: `π` の下の各 leaf `λ` につき `obj(v, λ)` への参照を 1 つ取り除く。同時に `H` を 1 下げる。
- 生成 (割り当て、boxed 容器からの読み出し、呼び出しの返り値): 生じた各 leaf につき参照を 1 つ加える。`H` は
  その場で上がる。
- 消費 (D9): 消費される各 leaf につき参照を 1 つ取り除く。`H` は変わらない (参照は渡された先が持つ)。

### 3.4 健全性

**D11 (健全な本体)**
本体 `B` が、所有権の割り当て (D14) の下で**健全**であるとは、`B` のすべての実行路について次の 3 つが
成り立つことをいう。

- **(S-a) 過剰処分が無い**: `Obl` から参照を取り除くすべての操作について、取り除かれる参照はその時点の `Obl` に
  入っている。
- **(S-b) 漏れが無い**: 実行路の終端の `Ret(v)` において、`Ret` 自身の消費を行った後の `Obl` は空である。
- **(S-c) 解放後の読みが無い**: 値を読むすべての操作について、読まれる各オブジェクトはその時点で解放されていない。

**D12 (健全なプログラム)**
プログラムが**健全**であるとは、そのすべての関数の本体とすべてのグローバル初期化子が、そのプログラムが定める
所有権の割り当て (D14) の下で健全であることをいう。

### 3.5 同一性のモデル

**D13 (origin)**
`origin(vars, τenv, x, π)` は、変数 `x` の path `π` にある値が、どの変数のどの path で作られた参照を持つかを
答える (`CODE src/rc_ir/ownership.rs: origin`)。答えは `Exactly(u, σ)` (ちょうど 1 つのオブジェクト) か
`Join { identity, candidates }` (通った路によって変わり、`identity` はどの路でも一致する 1 つの名前) の
どちらかである (`CODE src/rc_ir/ownership.rs: Origin`)。

**D14 (所有と借用、unit key)**
関数の各パラメータ・capture の各 unit は、その関数が**所有する**か**借用する**かのどちらかである。借用する
ものの集合が `RcFunc::borrowed_units` であり、残りが所有するものである
(`CODE src/rc_ir/ownership.rs: all_owned_units`)。

`unit_key(x, π) = unit_of(origin(x, π).identity())` を、その leaf の参照が数えられる**キー**と呼ぶ
(`CODE src/rc_ir/ownership.rs: unit_key`)。`acted_references(v, π)` は、`(v, π)` への RC 操作が触れる
参照を、オブジェクトごとの個数として返す (`CODE src/rc_ir/ownership.rs: acted_references`)。

## 4. 仮定

各仮定には、それを果たす者を書く。

**A1 (入力の健全性)** -- 果たす者: 前段のパス (`insert_rc` と `split_rc_units`)。
`borrow_ify` に渡されるプログラムは D12 の意味で健全である。そのときすべてのパラメータ・capture の unit は
所有される (`borrowed_units` が空である)。

**A2 (単位への正規化)** -- 果たす者: `split_rc_units` (`CODE src/rc_ir/borrow.rs: split_rc_units`)。
`borrow_ify` に渡されるプログラムのすべての `Retain`/`Release` 節点の path は、その変数の型の `rc_units` の
要素である。

**A3 (宣言されたモデルの忠実さ)** -- 果たす者: 誰も。
各 `LLVMGen` の `result_prov` と `borrows_operand` は、その演算が生成するコードを正しく述べている。すなわち、
`result_prov` が結果の leaf を `Arg(j, σ)` と宣言するとき、生成コードはその leaf に第 `j` オペランドの
leaf `σ` と**同じ参照**を置き、新しい参照を作らない。`Fresh` と宣言するときは新しいオブジェクトを割り当て、
`Unknown` と宣言するときは新しい参照を作る。`borrows_operand(i)` が真のとき、生成コードは第 `i` オペランドの
参照を処分しない。

この仮定は誰も果たさない。宣言と実装の乖離は、証明ではなくテストと valgrind が捕まえる。
`dev-docs/2026-06-28-unique-check-elim/audit-2026-07-20-op-declarations.md` が、ある時点での全 op の
宣言を人手で照合した記録である。

**A4 (コード生成の忠実さ)** -- 果たす者: 誰も。
コード生成は、`Retain(v, π)` / `Release(v, π)` を、`π` の下の各 boxed leaf の参照カウントの ±1 として実装し、
`Destructure` と `Match` を D9 の消費モデルの通りに実装する。

**A5 (型が leaf の権威)** -- 果たす者: `leaf_map.rs` の設計。
値が保持する参照は、その型の `boxed_leaf_paths` が列挙する leaf にちょうど 1 つずつある。

**A6 (名前の一意性)** -- 果たす者: lowering と `fresh_rename_function`。
プログラム中のすべての束縛変数の名前は相異なる。よって変数名は束縛を一意に決める。

**A7 (呼び出し先の解決)** -- 果たす者: `resolve_callee_params` の設計 (`CODE src/rc_ir/ownership.rs:
resolve_callee_params`)。
`prog.funcs` に無い呼び出し先は、全パラメータを所有するものとして扱われる。これは安全側の近似である。

**A8 (グローバルは線形規律の外)** -- 果たす者: `mark_global`。
グローバル値が到達するオブジェクトは参照カウントの対象外であり、それらへの `Retain`/`Release` は何もしない。

## 5. 命題

依存の順に並べる。各命題は自分より小さい番号の命題だけを引用してよい。

### 層 1 -- 所有権モデル (`ownership.rs`)

- **P1** (leaf と unit の対応)。任意の型 `τ` について、`boxed_leaf_paths(τ)` の各 leaf の
  `truncate_to_unit` は `rc_units(τ)` の要素であり、`rc_units(τ)` の各 unit はある leaf の
  `truncate_to_unit` である。
- **P2** (`origin` の全域性と停止性)。`origin` はすべての `(x, π)` に対して答えを返し、停止する。
- **P3** (`origin` の健全性 -- `Exactly`)。`origin(x, π) = Exactly(u, σ)` のとき、すべての実行路の
  その位置において、スロット `(x, π)` の下の各 leaf が持つ参照は、スロット `(u, σ)` の対応する leaf が持つ
  参照と同一である。
- **P4** (`origin` の健全性 -- `Join`)。`origin(x, π) = Join { identity, candidates }` のとき、各実行路に
  おいてスロット `(x, π)` が持つ参照は `candidates` のいずれかのスロットが持つ参照と同一であり、かつ
  `identity` は、その参照に対してどの実行路でも同じ名前である。
- **P5** (キーは参照の関数である)。同じ参照を持つ 2 つのスロットは、同じ `unit_key` を持つ。
- **P6** (`acted_references` の正しさ)。`acted_references(v, π)` が返す多重集合は、`Retain(v, π)` が作る
  参照の多重集合、および `Release(v, π)` が処分する参照の多重集合に一致する。
- **P7** (消費の網羅性)。関数の活性化が保持する参照を手放す構文はすべて、`collect_consumes` が報告する。

### 層 2 -- `borrow_ify`

- **P8** (推論の停止性と安全性)。`infer_ownership` は停止し、その不動点において、ある実行路で消費される
  パラメータ leaf はすべて `Own` である。
- **P9** (複製は名前替えである)。`clone_func` が作る借用版の本体は、元の本体の束縛変数を一斉に付け替えた
  ものであり、それ以外の違いを持たない。
- **P10** (借用版が落とす RC 節点)。借用版の `rewrite_rc` は、その版が所有しない unit の `Retain`/`Release`
  だけを落とし、所有する unit のものは残す。
- **P11** (呼び出し側の補正)。`call_rc` が置く前後の RC 節点は、呼び出し元と呼び出し先の所有権の食い違いを
  ちょうど埋める。すなわち、呼び出し元が借用し呼び出し先が所有する unit には前に `Retain` を、呼び出し元が
  所有し呼び出し先が借用する unit には後に `Release` を置く。
- **P12** (振り分けの安全性)。`route` が借用版へ回すのは、末尾位置でない呼び出しか、所有する引数を 1 つも
  持たない呼び出しだけである。よって後置の `Release` が末尾呼び出しの末尾性を壊すことはない。
- **P13** (注釈の一致)。出力の各版の `borrowed_units` は、その版が実際に処分しない unit の集合に一致する。
- **P14** (`borrow_ify` の健全性)。健全なプログラムを入力とすると、`borrow_ify` の出力は健全である。

### 層 3 -- `cancel` の走査

- **P15** (走査は各節点を 1 度だけ訪れる)。`CancelAnalysis::walk` は本体の各節点をちょうど 1 回訪れる。
- **P16** (`pending` の不変条件)。走査中の各位置において、`pending[k]` は、キー `k` の下でその位置までに
  実行され、まだ完全には un-bump されていない `Retain` の列であり、内側のものほど後ろにある。各要素の
  `outstanding` は、その `Retain` が作った参照のうちまだ処分されていないものの多重集合である。
- **P17** (`un_bump` の正しさ)。`un_bump(pending, k, R)` が `InBracket(t)` を返すとき、`t` は `R` の各参照を
  作った `Retain` であり、`pending` から `R` の分だけが引かれる。`OutsideBracket` を返すとき、その
  `Release` は最内の `Retain` が作っていない参照に触れる。`NoBracket` を返すとき、キー `k` の下に
  pending な `Retain` は無い。
- **P18** (`merge` の安全性)。`merge` の後に `pending` に残る `Retain` は、すべてのアームの出口で
  同じ `outstanding` を持つものだけである。それ以外はすべて `needed_retains` に入る。

### 層 4 -- `cancel` の健全性

- **P19** (削除される retain の性質)。`cancelled()` が返す集合に含まれる `Retain` `t` は、`t` を含むすべての
  実行路において、`t` のキーの消費より前、かつ関数からの復帰より前に、削除される `Release` 群によって
  完全に un-bump される。
- **P20** (削除は収支を保つ)。各実行路において、削除される `Retain` が作る参照の多重集合は、その路で実行される
  削除される `Release` が処分する参照の多重集合に一致する。
- **P21** (削除は解放後の読みを作らない)。`Retain` `t` とそれを un-bump する `Release` 群を削除しても、
  削除の前後で解放されるオブジェクトの集合と時点は変わらない。
- **P22** (`drop_nodes` の正しさ)。`drop_nodes` は、記録された節点をちょうど取り除き、他は変えない。
- **P23** (`cancel` の健全性)。健全なプログラムを入力とすると、`cancel` の出力は健全である。

### 主定理

- **T** (パイプラインの健全性)。`split_rc_units` の出力が健全であるとき、`cancel(borrow_ify(・))` の出力は
  健全である。P14 と P23 の合成である。

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

較正をやり直す条件: D11 を変えたとき。変えた定義の下で修正前のコードが健全になるなら、その変更は却下する。

## 7. 検証状況

| 命題 | ファイル | 証明 | 検証 |
|---|---|---|---|
| P1, P2 | `p10-leaves-and-units.md` | 未着手 | 未着手 |
| P3, P4 | `p11-origin-soundness.md` | 未着手 | 未着手 |
| P5, P6, P7 | `p12-keys-and-consumes.md` | 未着手 | 未着手 |
| P8 - P14 | `p20-borrow-ify.md` | 未着手 | 未着手 |
| P15 - P18 | `p30-cancel-walk.md` | 未着手 | 未着手 |
| P19 - P23, T | `p40-cancel-soundness.md` | 未着手 | 未着手 |
